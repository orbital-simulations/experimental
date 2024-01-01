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

impl FilledCircle {
    fn buffer_description<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<FilledCircle>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &CIRCLE_VERTEX_ATTRIBUTES,
        }
    }
}

#[derive(Debug)]
pub struct FilledCircleRenderer {
    circles: Vec<FilledCircle>,
    circle_vertex_buffer: Buffer,
    circle_index_buffer: Buffer,
    circle_instance_buffer: Buffer,
    circle_pipeline: RenderPipeline,
}

impl FilledCircleRenderer {
    pub fn new(
        windowed_device: &mut WindowedDevice,
        projection_bind_group_layout: &BindGroupLayout,
    ) -> Self {
        let circle_shader = windowed_device
            .device
            .create_shader_module(include_wgsl!("../shaders/filled_circle.wgsl"));
        let render_pipeline_layout =
            windowed_device
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[projection_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let circle_pipeline =
            windowed_device
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Full circle Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &circle_shader,
                        entry_point: "vs_main",
                        buffers: &[
                            vec2_buffer_description(),
                            FilledCircle::buffer_description(),
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &circle_shader,
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
            windowed_device
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Circle Vertex Buffer"),
                    contents: CIRCLE_VERTICES.get_raw(),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let circle_index_buffer =
            windowed_device
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Circle Index Buffer"),
                    contents: CIRCLE_INDICES.get_raw(),
                    usage: wgpu::BufferUsages::INDEX,
                });

        // This will probably fial....
        let circle_instance_buffer = windowed_device.device.create_buffer(&BufferDescriptor {
            label: Some("Circle Index Buffer"),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            size: 0,
            mapped_at_creation: false,
        });

        Self {
            circles: vec![],
            circle_vertex_buffer,
            circle_index_buffer,
            circle_instance_buffer,
            circle_pipeline,
        }
    }

    pub fn add_circle(&mut self, circle: FilledCircle) {
        self.circles.push(circle);
    }

    pub fn render(
        &mut self,
        windowed_device: &mut WindowedDevice,
        projection_bind_group: &BindGroup,
        view: &TextureView,
        encoder: &mut CommandEncoder,
    ) {
        if (self.circle_instance_buffer.size() as usize) < self.circles.len() {
            self.circle_instance_buffer =
                windowed_device
                    .device
                    .create_buffer_init(&BufferInitDescriptor {
                        label: Some("Circle Index Buffer"),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        contents: self.circles.get_raw(),
                    });
        } else {
            windowed_device.queue.write_buffer(
                &self.circle_instance_buffer,
                0,
                self.circles.get_raw(),
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Rectangle Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.circle_pipeline);
            render_pass.set_bind_group(0, projection_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.circle_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.circle_instance_buffer.slice(..));
            render_pass.set_index_buffer(
                self.circle_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(
                0..(CIRCLE_INDICES.len() as u32),
                0,
                0..(self.circles.len() as u32),
            );
        }

        // TODO: Think about some memory releasing strategy. Spike in number of
        // circles will lead to space leak.
        self.circles.clear();
    }
}
