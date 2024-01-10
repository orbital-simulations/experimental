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
pub struct StrokeCircle {
    pub pos: Vec2,
    pub radius: f32,
    pub border: f32,
    pub color: Vec3,
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for StrokeCircle {}

impl StrokeCircle {
    pub fn new(pos: Vec2, radius: f32, border: f32, color: Vec3) -> Self {
        Self {
            pos,
            radius,
            border,
            color,
        }
    }
}

const STROKE_CIRCLE_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 4] =
    vertex_attr_array![1 => Float32x2, 2 => Float32, 3 => Float32, 4 => Float32x3];

const STROKE_CIRCLE_VERTICES: [Vec2; 4] = [
    Vec2 { x: -1.0, y: -1.0 },
    Vec2 { x: 1.0, y: -1.0 },
    Vec2 { x: -1.0, y: 1.0 },
    Vec2 { x: 1.0, y: 1.0 },
];

const STROKE_CIRCLE_INDICES: &[u16] = &[0, 1, 3, 3, 2, 0];

const INITIAL_BUFFER_CAPACITY: usize = 4;

const INITIAL_BUFFER_SIZE: u64 = (INITIAL_BUFFER_CAPACITY * size_of::<StrokeCircle>()) as u64;

macro_rules! prefix_label {
    () => {
        "Stroke circle "
    };
}

impl StrokeCircle {
    fn buffer_description<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<StrokeCircle>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &STROKE_CIRCLE_VERTEX_ATTRIBUTES,
        }
    }
}

#[derive(Debug)]
pub struct StrokeCircleRenderer {
    circles: Vec<StrokeCircle>,
    circle_vertex_buffer: Buffer,
    circle_index_buffer: Buffer,
    circle_instance_buffer: Buffer,
    circle_pipeline: RenderPipeline,
    circle_instance_buffer_capacity: usize,
}

impl StrokeCircleRenderer {
    pub fn new(context: &Context, projection_bind_group_layout: &BindGroupLayout) -> Self {
        let circle_shader = context
            .device
            .create_shader_module(include_wgsl!("../shaders/stroke_circle.wgsl"));
        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(concat!(prefix_label!(), " render pipeline layout")),
                    bind_group_layouts: &[projection_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let circle_pipeline =
            context
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some(concat!(prefix_label!(), " render pipeline")),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &circle_shader,
                        entry_point: "vs_main",
                        buffers: &[
                            vec2_buffer_description(),
                            StrokeCircle::buffer_description(),
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &circle_shader,
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

        let circle_vertex_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(concat!(prefix_label!(), " vertex buffer")),
                    contents: STROKE_CIRCLE_VERTICES.get_raw(),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let circle_index_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(concat!(prefix_label!(), " index buffer")),
                    contents: STROKE_CIRCLE_INDICES.get_raw(),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let circle_instance_buffer = context.device.create_buffer(&BufferDescriptor {
            label: Some(concat!(prefix_label!(), " instance buffer")),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: INITIAL_BUFFER_SIZE,
            mapped_at_creation: false,
        });

        Self {
            circles: vec![],
            circle_vertex_buffer,
            circle_index_buffer,
            circle_instance_buffer,
            circle_pipeline,
            circle_instance_buffer_capacity: INITIAL_BUFFER_CAPACITY,
        }
    }

    pub fn add_stroke_circle(&mut self, circle: StrokeCircle) {
        self.circles.push(circle);
    }

    pub fn render<'a>(
        &'a mut self,
        context: &Context,
        projection_bind_group: &'a BindGroup,
        render_pass: &mut RenderPass<'a>,
    ) {
        if self.circle_instance_buffer_capacity < self.circles.len() {
            self.circle_instance_buffer_capacity = self.circles.len();
            self.circle_instance_buffer =
                context.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some(concat!(prefix_label!(), " instance buffer")),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    contents: self.circles.get_raw(),
                });
        } else {
            context
                .queue
                .write_buffer(&self.circle_instance_buffer, 0, self.circles.get_raw());
        }

        render_pass.set_pipeline(&self.circle_pipeline);
        render_pass.set_bind_group(0, projection_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.circle_vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.circle_instance_buffer.slice(..));
        render_pass.set_index_buffer(
            self.circle_index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(
            0..(STROKE_CIRCLE_INDICES.len() as u32),
            0,
            0..(self.circles.len() as u32),
        );

        // TODO: Think about some memory releasing strategy. Spike in number of
        // circles will lead to space leak.
        self.circles.clear();
    }
}
