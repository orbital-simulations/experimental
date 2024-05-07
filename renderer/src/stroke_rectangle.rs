use glam::{Vec2, Vec3};
use wgpu::{
    include_wgsl, vertex_attr_array, BindGroupLayoutEntry, FrontFace,
    PrimitiveState, PrimitiveTopology, RenderPass, ShaderStages, VertexStepMode,
};

use crate::{
    buffers::{IndexBuffer, WriteableBuffer},
    context::{Context, RenderingContext},
    pipeline::{
        BindGroupLayoutDescription, ColorTargetState, FragmentStateDescription, MultisampleStatePick, PipelineDescription, PipelineID, PipelineLayoutDescription, PipelineStore, TextureFormatPicker, UnlockedPipelineStore, VertexBufferLayoutDescriptor, VertexStateDescription
    },
    raw::Gpu,
    shader_store::{ShaderDescription, ShaderStatic, ShaderStore},
};

#[derive(Debug)]
#[repr(C, packed)]
pub struct StrokeRectangle {
    pub pos: Vec2,
    pub size: Vec2,
    pub border: f32,
    pub color: Vec3,
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for StrokeRectangle {}

impl StrokeRectangle {
    pub fn new(pos: Vec2, size: Vec2, border: f32, color: Vec3) -> Self {
        Self {
            pos,
            size,
            border,
            color,
        }
    }
}

const STROKE_RECTANGLE_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 4] =
    vertex_attr_array![1 => Float32x2, 2 => Float32x2, 3 => Float32, 4 => Float32x3];

const STROKE_RECTANGLE_VERTICES: [Vec2; 4] = [
    Vec2 { x: -1.0, y: -1.0 },
    Vec2 { x: 1.0, y: -1.0 },
    Vec2 { x: -1.0, y: 1.0 },
    Vec2 { x: 1.0, y: 1.0 },
];

const STROKE_RECTANGLE_INDICES: &[u16] = &[0, 1, 3, 3, 2, 0];

pub struct StrokeRectangleRenderer {
    rectangles: Vec<StrokeRectangle>,
    vertex_buffer: WriteableBuffer<Vec2>,
    index_buffer: IndexBuffer<u16>,
    instance_buffer: WriteableBuffer<StrokeRectangle>,
    pipeline: Option<PipelineID>,
}

impl StrokeRectangleRenderer {
    pub fn new(
        context: &Context,
        shader_store: &ShaderStore,
        pipeline_store: &PipelineStore,
    ) -> Self {
        let shader_description = ShaderDescription::ShaderStatic(ShaderStatic {
            unique_shader_name: "stroke_rectangle".to_string(),
            static_shader_module_descriptor: include_wgsl!("../shaders/stroke_rectangle.wgsl"),
        });
        let shader = {
            let mut shader_store = shader_store.lock();
            shader_store.get_or_create(&shader_description)
        };

        let pipeline_layout_description = PipelineLayoutDescription {
            label: "stroke rectangle renderer layout".to_string(),
            // TODO: Camera bind broup description should be taken from camera
            bind_group_layouts: vec![BindGroupLayoutDescription {
                label: "camera bind group layout description".to_string(),
                entries: vec![
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            }],
            push_constant_ranges: vec![],
        };
        let pipeline_description = PipelineDescription {
            label: "stroke rectangle renderer pipeline".to_string(),
            layout: Some(pipeline_layout_description),
            vertex: VertexStateDescription {
                buffers: vec![
                    VertexBufferLayoutDescriptor {
                        array_stride: std::mem::size_of::<Vec2>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        attributes: vertex_attr_array![0 => Float32x2].to_vec(),
                    },
                    VertexBufferLayoutDescriptor {
                        array_stride: std::mem::size_of::<StrokeRectangle>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: STROKE_RECTANGLE_VERTEX_ATTRIBUTES.to_vec(),
                    },
                ],
                module: shader.clone(),
            },
            // TODO: This should be generated from target...
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            // TODO: This should be part of the target as well...
            depth_stencil: None,
            multisample: MultisampleStatePick::Output,
            fragment: Some(FragmentStateDescription {
                module: shader,
            targets: vec![Some(ColorTargetState {
                    format: TextureFormatPicker::Output,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        };

        let mut pipeline_store = pipeline_store.lock();

        let pipeline = pipeline_store.get_or_create(&pipeline_description);

        let index_buffer =
            IndexBuffer::new(context, "circle index buffer", STROKE_RECTANGLE_INDICES);
        let vertex_buffer = WriteableBuffer::new(
            context,
            "circle vertex buffer",
            &STROKE_RECTANGLE_VERTICES,
            wgpu::BufferUsages::VERTEX,
        );

        let instance_buffer = WriteableBuffer::new(
            context,
            "circle instance buffer",
            &[],
            wgpu::BufferUsages::VERTEX,
        );

        Self {
            rectangles: vec![],
            vertex_buffer,
            index_buffer,
            instance_buffer,
            pipeline: Some(pipeline),
        }
    }

    pub fn add_rectangle(&mut self, rectangle: StrokeRectangle) {
        self.rectangles.push(rectangle);
    }

    pub fn render<'a>(
        &'a mut self,
        context: &Context,
        rendering_context: &'a RenderingContext,
        render_pass: &mut RenderPass<'a>,
        //        render_target_description: &RenderTargetDescription,
        pipeline_store: &'a UnlockedPipelineStore<'a>,
    ) {
        if !self.rectangles.is_empty() {
            self.instance_buffer.write_data(context, &self.rectangles);

            if self.pipeline.is_none() {
                //                self.pipeline = Some(pipeline_store.get_entry(self, render_target_description));
            }

            let pipeline = self
                .pipeline
                .as_ref()
                .expect("pipeline should be created by now");
            let pipeline = pipeline_store.get_ref(pipeline);

            render_pass.set_pipeline(pipeline);
            rendering_context.camera().bind(render_pass, 0);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            self.index_buffer.set_index_buffer(render_pass);
            render_pass.draw_indexed(
                0..self.index_buffer.draw_count(),
                0,
                0..(self.rectangles.len() as u32),
            );

            // TODO: Think about some memory releasing strategy. Spike in number of
            // circles will lead to space leak.
            self.rectangles.clear();
        }
    }
}
