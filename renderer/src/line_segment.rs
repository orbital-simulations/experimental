use glam::{Vec2, Vec3};
use wgpu::{
    include_wgsl, vertex_attr_array, BindGroupLayoutEntry, FrontFace, MultisampleState,
    PrimitiveState, PrimitiveTopology, RenderPass, ShaderStages, TextureFormat, VertexStepMode,
};

use crate::{
    buffers::{IndexBuffer, WriteableBuffer},
    context::{Context, RenderingContext},
    pipeline::{
        BindGroupLayoutDescription, FragmentStateDescription, PipelineDescription, PipelineID, PipelineLayoutDescription, PipelineStore, UnlockedPipelineStore, VertexBufferLayoutDescriptor, VertexStateDescription
    },
    raw::Gpu,
    shader_store::{ShaderDescription, ShaderStatic, ShaderStore},
};

#[derive(Debug)]
#[repr(C, packed)]
pub struct LineSegment {
    pub from: Vec2,
    pub to: Vec2,
    pub color: Vec3,
    pub width: f32,
}

impl LineSegment {
    pub fn new(from: Vec2, to: Vec2, color: Vec3, width: f32) -> Self {
        Self {
            from,
            to,
            color,
            width,
        }
    }
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for LineSegment {}

const LINE_SEGMENT_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 4] =
    vertex_attr_array![1 => Float32x2, 2 => Float32x2, 3 => Float32x3, 4 => Float32];

const LINE_SEGMENT_VERTICES: [Vec2; 4] = [
    Vec2 { x: -1.0, y: -1.0 },
    Vec2 { x: 1.0, y: -1.0 },
    Vec2 { x: -1.0, y: 1.0 },
    Vec2 { x: 1.0, y: 1.0 },
];

const LINE_SEGMENT_INDICES: &[u16] = &[0, 1, 3, 3, 2, 0];

pub struct LineSegmentRenderer {
    line_segments: Vec<LineSegment>,
    pipeline: Option<PipelineID>,
    vertex_buffer: WriteableBuffer<Vec2>,
    index_buffer: IndexBuffer<u16>,
    instance_buffer: WriteableBuffer<LineSegment>,
}

impl LineSegmentRenderer {
    pub fn new(
        context: &Context,
        shader_store: &ShaderStore,
        pipeline_store: &PipelineStore,
        target_texture_format: &TextureFormat,
    ) -> Self {
        let shader_description = ShaderDescription::ShaderStatic(ShaderStatic {
            unique_shader_name: "line_segment_renderer".to_string(),
            static_shader_module_descriptor: include_wgsl!("../shaders/line_segment.wgsl"),
        });
        let shader = {
            let mut shader_store = shader_store.lock();
            shader_store.get_or_create(&shader_description)
        };

        let pipeline_description = PipelineDescription {
            label: "line segment renderer pipeline".to_string(),
            layout: Some(PipelineLayoutDescription {
                label: "line segmen renderer layout".to_string(),
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
            }),
            vertex: VertexStateDescription {
                buffers: vec![
                    VertexBufferLayoutDescriptor {
                        array_stride: std::mem::size_of::<Vec2>() as u64,
                        step_mode: VertexStepMode::Vertex,
                        attributes: vertex_attr_array![0 => Float32x2].to_vec(),
                    },
                    VertexBufferLayoutDescriptor {
                        array_stride: std::mem::size_of::<LineSegment>() as u64,
                        step_mode: VertexStepMode::Instance,
                        attributes: LINE_SEGMENT_VERTEX_ATTRIBUTES.to_vec(),
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
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentStateDescription {
                module: shader,
                targets: vec![Some(wgpu::ColorTargetState {
                    format: *target_texture_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        };

        let mut pipeline_store = pipeline_store.lock();

        let pipeline = pipeline_store.get_or_create(&pipeline_description);

        let index_buffer = IndexBuffer::new(context, "circle index buffer", LINE_SEGMENT_INDICES);
        let vertex_buffer = WriteableBuffer::new(
            context,
            "circle vertex buffer",
            &LINE_SEGMENT_VERTICES,
            wgpu::BufferUsages::VERTEX,
        );

        let instance_buffer = WriteableBuffer::new(
            context,
            "circle instance buffer",
            &[],
            wgpu::BufferUsages::VERTEX,
        );

        Self {
            line_segments: vec![],
            vertex_buffer,
            index_buffer,
            instance_buffer,
            pipeline: Some(pipeline),
        }
    }

    pub fn add_line_segment(&mut self, line_segment: LineSegment) {
        self.line_segments.push(line_segment);
    }

    pub fn render<'a>(
        &'a mut self,
        context: &Context,
        rendering_context: &'a RenderingContext,
        render_pass: &mut RenderPass<'a>,
        // TODO:        render_target_description: &RenderTargetDescription,
        pipeline_store: &'a UnlockedPipelineStore<'a>,
    ) {
        if !self.line_segments.is_empty() {
            self.instance_buffer
                .write_data(context, &self.line_segments);

            if self.pipeline.is_none() {
                //    let pipeline = pipeline_store.get_entry(self);

                //    self.pipeline = Some(pipeline);
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
                0..(self.line_segments.len() as u32),
            );

            //            render_pass.draw(0..3, 0..1);
            // TODO: Think about some memory releasing strategy. Spike in number of
            // circles will lead to space leak.
            self.line_segments.clear();
        }
    }
}
