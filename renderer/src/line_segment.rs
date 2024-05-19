use glam::{Vec2, Vec3};
use wgpu::{vertex_attr_array, RenderPass, VertexStepMode};

use crate::{
    buffers::{IndexBuffer, WriteableBuffer},
    context::{Context, RenderingContext},
    include_wgsl,
    pipeline::{Pipeline, RenderTargetDescription},
    raw::Gpu,
    resource_watcher::ResourceWatcher,
    web_gpu::{
        FragmentState, PipelineLayoutDescription, RenderPipelineDescription, VertexBufferLayout,
        VertexState,
    },
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
    pipeline: Option<Pipeline>,
    vertex_buffer: WriteableBuffer<Vec2>,
    index_buffer: IndexBuffer<u16>,
    instance_buffer: WriteableBuffer<LineSegment>,
}

impl LineSegmentRenderer {
    pub fn new(context: &Context) -> Self {
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
            pipeline: None,
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
        render_target_description: &RenderTargetDescription,
        resource_watcher: &mut ResourceWatcher,
    ) {
        if !self.line_segments.is_empty() {
            self.instance_buffer
                .write_data(context, &self.line_segments);

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
                    shader: include_wgsl!("../shaders/line_segment.wgsl"),
                    label: "line segment renderer".to_string(),
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
                                array_stride: std::mem::size_of::<LineSegment>() as u64,
                                step_mode: VertexStepMode::Instance,
                                attributes: LINE_SEGMENT_VERTEX_ATTRIBUTES.into(),
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
                    multisample: wgpu::MultisampleState {
                        count: render_target_description.multisampling,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(FragmentState { targets }),
                    multiview: None,
                };
                let pipeline =
                    Pipeline::new(context, &render_pipeline_description, resource_watcher);

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
                0..(self.line_segments.len() as u32),
            );

            // TODO: Think about some memory releasing strategy. Spike in number of
            // circles will lead to space leak.
            self.line_segments.clear();
        }
    }
}
