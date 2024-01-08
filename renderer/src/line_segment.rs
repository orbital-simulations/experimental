use glam::{Vec2, Vec3};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupLayout, Buffer, BufferAddress, BufferDescriptor,
    RenderPass, RenderPipeline, VertexBufferLayout,
};

use crate::{
    context::Context,
    raw::{Gpu, Raw},
};

pub struct LineSegment {
    pub from: Vec2,
    pub to: Vec2,
    pub color: Vec3,
}

impl LineSegment {
    pub fn new(from: Vec2, to: Vec2, color: Vec3) -> Self {
        Self { from, to, color }
    }
}

#[derive(Debug)]
#[repr(C, packed)]
struct Endpoint {
    position: Vec2,
    color: Vec3,
}

impl Endpoint {
    fn new(position: Vec2, color: Vec3) -> Endpoint {
        Endpoint { position, color }
    }
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for Endpoint {}

const LINE_SEGMENT_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] =
    vertex_attr_array![0 => Float32x2, 1 => Float32x3];

impl Endpoint {
    fn buffer_description<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Endpoint>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &LINE_SEGMENT_VERTEX_ATTRIBUTES,
        }
    }
}

#[repr(C, packed)]
struct Endpoints {
    from: Endpoint,
    to: Endpoint,
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for Endpoints {}

pub struct LineSegmentRenderer {
    endpoints: Vec<Endpoints>,
    line_segment_vertex_buffer: Buffer,
    line_segment_pipeline: RenderPipeline,
    // Number of items that the vertex or index buffer can hold (they have the same capacity)
    buffer_capacity: usize,
}

impl LineSegmentRenderer {
    pub fn new(context: &Context, projection_bind_group_layout: &BindGroupLayout) -> Self {
        let rectangle_shader = context
            .device
            .create_shader_module(include_wgsl!("../shaders/line_segment.wgsl"));
        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Line Segment Render Pipeline Layout"),
                    bind_group_layouts: &[projection_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let line_segment_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Line Segment Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &rectangle_shader,
                        entry_point: "vs_main",
                        buffers: &[Endpoint::buffer_description()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &rectangle_shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: context.texture_format,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::LineList,
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
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    // If the pipeline will be used with a multiview render pass, this
                    // indicates how many array layers the attachments will have.
                    multiview: None,
                });

        let line_segment_vertex_buffer = context.device.create_buffer(&BufferDescriptor {
            label: Some("Line segment vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: 0,
            mapped_at_creation: false,
        });

        Self {
            endpoints: vec![],
            line_segment_vertex_buffer,
            line_segment_pipeline,
            buffer_capacity: 0,
        }
    }

    pub fn add_line_segment(&mut self, line_segment: LineSegment) {
        self.endpoints.push(Endpoints {
            from: Endpoint::new(line_segment.from, line_segment.color),
            to: Endpoint::new(line_segment.to, line_segment.color),
        });
    }

    pub fn render<'a>(
        &'a mut self,
        context: &Context,
        projection_bind_group: &'a BindGroup,
        render_pass: &mut RenderPass<'a>,
    ) {
        if self.buffer_capacity < self.endpoints.len() {
            self.line_segment_vertex_buffer =
                context.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some("Line segment vertex buffer"),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    contents: self.endpoints.get_raw(),
                });
            self.buffer_capacity = self.endpoints.len()
        } else {
            context.queue.write_buffer(
                &self.line_segment_vertex_buffer,
                0,
                self.endpoints.get_raw(),
            );
        }

        render_pass.set_pipeline(&self.line_segment_pipeline);
        render_pass.set_bind_group(0, projection_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.line_segment_vertex_buffer.slice(..));
        let count = self.endpoints.len() as u32;
        render_pass.draw(0..(2 * count), 0..1);

        self.endpoints.clear();
    }
}
