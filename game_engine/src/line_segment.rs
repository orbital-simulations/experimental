use glam::{Vec2, Vec3};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupLayout, Buffer, BufferAddress, BufferDescriptor,
    CommandEncoder, RenderPipeline, StoreOp, TextureView, VertexBufferLayout,
};

use crate::{
    buffers::vec2_buffer_description,
    raw::{Gpu, Raw},
    windowed_device::WindowedDevice,
};

pub struct LineSegment {
    pub from: Vec2,
    pub to: Vec2,
    pub color: Vec3,
}

#[derive(Debug)]
#[repr(C, packed)]
struct Color {
    color: Vec3,
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for Color {}

impl Color {
    fn buffer_description<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Color>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &LINE_SEGMENT_VERTEX_ATTRIBUTES,
        }
    }
}

#[derive(Debug)]
#[repr(C, packed)]
struct Endpoints {
    from: Vec2,
    to: Vec2,
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for Endpoints {}

impl LineSegment {
    pub fn new(from: Vec2, to: Vec2, color: Vec3) -> Self {
        Self { from, to, color }
    }
}

const LINE_SEGMENT_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 1] =
    vertex_attr_array![1 => Float32x3];

#[derive(Debug)]
pub struct LineSegmentRenderer {
    endpoints: Vec<Endpoints>,
    colors: Vec<Color>,
    line_segment_vertex_buffer: Buffer,
    line_segment_instance_buffer: Buffer,
    line_segment_pipeline: RenderPipeline,
    // Number of items that the vertex or index buffer can hold (they have the same capacity)
    buffer_capacity: usize,
}

impl LineSegmentRenderer {
    pub fn new(
        windowed_device: &mut WindowedDevice,
        projection_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let rectangle_shader = windowed_device
            .device
            .create_shader_module(include_wgsl!("../shaders/line_segment.wgsl"));
        let render_pipeline_layout =
            windowed_device
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Line Segment Render Pipeline Layout"),
                    bind_group_layouts: &[projection_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let line_segment_pipeline =
            windowed_device
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Line Segment Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &rectangle_shader,
                        entry_point: "vs_main",
                        buffers: &[vec2_buffer_description(), Color::buffer_description()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &rectangle_shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: windowed_device.config.format,
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

        let line_segment_vertex_buffer = windowed_device.device.create_buffer(&BufferDescriptor {
            label: Some("Line segment vertex buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: 0,
            mapped_at_creation: false,
        });

        let line_segment_instance_buffer =
            windowed_device.device.create_buffer(&BufferDescriptor {
                label: Some("Line segment instance buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                size: 0,
                mapped_at_creation: false,
            });

        Self {
            endpoints: vec![],
            colors: vec![],
            line_segment_vertex_buffer,
            line_segment_instance_buffer,
            line_segment_pipeline,
            buffer_capacity: 0,
        }
    }

    pub fn add_line_segment(&mut self, line_segment: LineSegment) {
        self.endpoints.push(Endpoints {
            from: line_segment.from,
            to: line_segment.to,
        });
        self.colors.push(Color {
            color: line_segment.color,
        });
    }

    pub fn render(
        &mut self,
        windowed_device: &mut WindowedDevice,
        projection_bind_group: &BindGroup,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        if self.buffer_capacity < self.endpoints.len() {
            self.line_segment_vertex_buffer =
                windowed_device
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: Some("Line segment vertex buffer"),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        contents: self.endpoints.get_raw(),
                    });
            self.line_segment_instance_buffer =
                windowed_device
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: Some("Line segment instance buffer"),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        contents: self.colors.get_raw(),
                    });
            self.buffer_capacity = self.endpoints.len()
        } else {
            windowed_device.queue.write_buffer(
                &self.line_segment_vertex_buffer,
                0,
                self.endpoints.get_raw(),
            );
            windowed_device.queue.write_buffer(
                &self.line_segment_instance_buffer,
                0,
                self.colors.get_raw(),
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Line segment render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.line_segment_pipeline);
            render_pass.set_bind_group(0, projection_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.line_segment_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.line_segment_instance_buffer.slice(..));
            let count = self.endpoints.len() as u32;
            render_pass.draw(0..(2 * count), 0..1);
        }

        self.endpoints = vec![];
        self.colors = vec![];
    }
}
