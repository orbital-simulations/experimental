use glam::{Vec2, Vec3};
use wgpu::{vertex_attr_array, MultisampleState, RenderPass, VertexStepMode};

use crate::{
    buffers::{IndexBuffer, WriteableBuffer}, context::{Context, RenderingContext}, include_wgsl, pipeline::{Pipeline, RenderTargetDescription}, raw::Gpu, resource_watcher::ResourceWatcher, web_gpu::{
        FragmentState, PipelineLayoutDescription, RenderPipelineDescription,
        VertexBufferLayout, VertexState,
    }
};

#[derive(Debug)]
#[repr(C, packed)]
pub struct FilledCircle {
    pub pos: Vec2,
    pub radius: f32,
    pub color: Vec3,
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for FilledCircle {}

impl FilledCircle {
    pub fn new(pos: Vec2, radius: f32, color: Vec3) -> Self {
        Self { pos, radius, color }
    }
}

const CIRCLE_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] =
    vertex_attr_array![1 => Float32x2, 2 => Float32, 3 => Float32x3];

const CIRCLE_VERTICES: [Vec2; 4] = [
    Vec2 { x: -1.0, y: -1.0 },
    Vec2 { x: 1.0, y: -1.0 },
    Vec2 { x: -1.0, y: 1.0 },
    Vec2 { x: 1.0, y: 1.0 },
];

const CIRCLE_INDICES: &[u16] = &[0, 1, 3, 3, 2, 0];

#[derive(Debug)]
pub struct FilledCircleRenderer {
    circles: Vec<FilledCircle>,
    vertex_buffer: WriteableBuffer<Vec2>,
    index_buffer: IndexBuffer<u16>,
    instance_buffer: WriteableBuffer<FilledCircle>,
    pipeline: Option<Pipeline>,
}

impl FilledCircleRenderer {
    pub fn new(context: &Context) -> Self {
        let index_buffer = IndexBuffer::new(context, "circle index buffer", CIRCLE_INDICES);
        let vertex_buffer = WriteableBuffer::new(
            context,
            "circle vertex buffer",
            &CIRCLE_VERTICES,
            wgpu::BufferUsages::VERTEX,
        );

        let instance_buffer = WriteableBuffer::new(
            context,
            "circle instance buffer",
            &[],
            wgpu::BufferUsages::VERTEX,
        );

        Self {
            circles: vec![],
            vertex_buffer,
            index_buffer,
            instance_buffer,
            pipeline: None,
        }
    }

    pub fn add_circle(&mut self, circle: FilledCircle) {
        self.circles.push(circle);
    }

    pub fn render<'a>(
        &'a mut self,
        context: &Context,
        rendering_context: &'a RenderingContext,
        render_pass: &mut RenderPass<'a>,
        render_target_description: &RenderTargetDescription,
        resource_watcher: &mut ResourceWatcher,
    ) {
        if !self.circles.is_empty() {
            self.instance_buffer.write_data(context, &self.circles);

            if self.pipeline.is_none() {
                let depth_stencil =
                    render_target_description
                        .depth_texture
                        .map(|format| wgpu::DepthStencilState {
                            format,
                            depth_write_enabled: true,
                            depth_compare: wgpu::CompareFunction::Less,
                            stencil: wgpu::StencilState::default(),
                            bias: wgpu::DepthBiasState::default(),
                        });

                let targets: Vec<Option<wgpu::ColorTargetState>> = render_target_description
                    .targets
                    .iter()
                    .map(|target_texture_format| {
                        Some(wgpu::ColorTargetState {
                            format: *target_texture_format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })
                    })
                    .collect();
                let render_pipeline_description = RenderPipelineDescription {
                    shader: include_wgsl!("../shaders/filled_circle.wgsl"),
                    label: "filled circle renderer".to_string(),
                    layout: Some(PipelineLayoutDescription {
                        bind_group_layouts: vec![rendering_context.camera().bind_group_layout()],
                        push_constant_ranges: vec![],
                    }),
                    vertex: VertexState {
                        buffers: vec![
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Vec2>() as u64,
                                step_mode: VertexStepMode::Vertex,
                                attributes: vertex_attr_array![0 => Float32x2].into(),
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<FilledCircle>() as u64,
                                step_mode: VertexStepMode::Instance,
                                attributes: CIRCLE_VERTEX_ATTRIBUTES.into(),
                            },
                        ],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                        // or Features::POLYGON_MODE_POINT
                        polygon_mode: wgpu::PolygonMode::Fill,
                        // Requires Features::DEPTH_CLIP_CONTROL
                        unclipped_depth: false,
                        // Requires Features::CONSERVATIVE_RASTERIZATION
                        conservative: false,
                    },
                    depth_stencil,
                    multisample: MultisampleState {
                        count: render_target_description.multisampling,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(FragmentState { targets }),
                    multiview: None,
                };
                let pipeline = Pipeline::new(context, &render_pipeline_description, resource_watcher);

                self.pipeline = Some(pipeline);
            }

            let pipeline = &self
                .pipeline
                .as_ref()
                .expect("pipeline should be created by now");

            let pipeline = pipeline.render_pipeline();
            render_pass.set_pipeline(&pipeline);
            rendering_context.camera().bind(render_pass, 0);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            self.index_buffer.set_index_buffer(render_pass);
            render_pass.draw_indexed(
                0..self.index_buffer.draw_count(),
                0,
                0..(self.circles.len() as u32),
            );

            // TODO: Think about some memory releasing strategy. Spike in number of
            // circles will lead to space leak.
            self.circles.clear();
        }
    }
}
