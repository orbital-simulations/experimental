use std::mem::size_of;

use glam::{Vec2, Vec3};
use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupLayout, Buffer, BufferAddress, BufferDescriptor,
    RenderPass, RenderPipeline, VertexBufferLayout,
};

use crate::{
    buffers::vec2_buffer_description,
    context::Context,
    raw::{Gpu, Raw},
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

const INITIAL_BUFFER_CAPACITY: usize = 4;

const INITIAL_BUFFER_SIZE: u64 = (INITIAL_BUFFER_CAPACITY * size_of::<LineSegment>()) as u64;

macro_rules! prefix_label {
    () => {
        "Line segment "
    };
}

const LINE_SEGMENT_VERTICES: [Vec2; 4] = [
    Vec2 { x: -1.0, y: -1.0 },
    Vec2 { x: 1.0, y: -1.0 },
    Vec2 { x: -1.0, y: 1.0 },
    Vec2 { x: 1.0, y: 1.0 },
];

const LINE_SEGMENT_INDICES: &[u16] = &[0, 1, 3, 3, 2, 0];

impl LineSegment {
    fn buffer_description<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<LineSegment>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &LINE_SEGMENT_VERTEX_ATTRIBUTES,
        }
    }
}

pub struct LineSegmentRenderer {
    line_segments: Vec<LineSegment>,
    line_segment_pipeline: RenderPipeline,
    line_segment_buffer_capacity: usize,
    line_segment_vertex_buffer: Buffer,
    line_segment_index_buffer: Buffer,
    line_segment_instance_buffer: Buffer,
}

impl LineSegmentRenderer {
    pub fn new(context: &Context, projection_bind_group_layout: &BindGroupLayout) -> Self {
        let line_segment_shader = context
            .device
            .create_shader_module(include_wgsl!("../shaders/line_segment.wgsl"));
        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(concat!(prefix_label!(), "render pipeline layout")),
                    bind_group_layouts: &[projection_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let line_segment_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(concat!(prefix_label!(), "render pipeline")),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &line_segment_shader,
                        entry_point: "vs_main",
                        buffers: &[vec2_buffer_description(), LineSegment::buffer_description()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &line_segment_shader,
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

        let line_segment_vertex_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(concat!(prefix_label!(), "vertex buffer")),
                    contents: LINE_SEGMENT_VERTICES.get_raw(),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let line_segment_index_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(concat!(prefix_label!(), "index buffer")),
                    contents: LINE_SEGMENT_INDICES.get_raw(),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // This will probably fial....
        let line_segment_instance_buffer = context.device.create_buffer(&BufferDescriptor {
            label: Some(concat!(prefix_label!(), "instance buffer")),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: INITIAL_BUFFER_SIZE,
            mapped_at_creation: false,
        });

        Self {
            line_segments: vec![],
            line_segment_vertex_buffer,
            line_segment_pipeline,
            line_segment_buffer_capacity: INITIAL_BUFFER_CAPACITY,
            line_segment_index_buffer,
            line_segment_instance_buffer,
        }
    }

    pub fn add_line_segment(&mut self, line_segment: LineSegment) {
        self.line_segments.push(line_segment);
    }

    pub fn render<'a>(
        &'a mut self,
        context: &Context,
        projection_bind_group: &'a BindGroup,
        render_pass: &mut RenderPass<'a>,
    ) {
        if self.line_segment_buffer_capacity < self.line_segments.len() {
            self.line_segment_vertex_buffer =
                context.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some(concat!(prefix_label!(), "instance buffer")),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    contents: self.line_segments.get_raw(),
                });
            self.line_segment_buffer_capacity = self.line_segments.len()
        } else {
            context.queue.write_buffer(
                &self.line_segment_instance_buffer,
                0,
                self.line_segments.get_raw(),
            );
        }

        render_pass.set_pipeline(&self.line_segment_pipeline);
        render_pass.set_bind_group(0, projection_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.line_segment_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.line_segment_instance_buffer.slice(..));
        render_pass.set_index_buffer(
            self.line_segment_index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(
            0..(LINE_SEGMENT_INDICES.len() as u32),
            0,
            0..(self.line_segments.len() as u32),
        );

        // TODO: Think about some memory releasing strategy. Spike in number of
        // circles will lead to space leak.
        self.line_segments.clear();
    }
}
