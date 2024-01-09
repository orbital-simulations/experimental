pub mod buffers;
pub mod colors;
pub mod context;
pub mod filled_circle;
pub mod stroke_circle;
pub mod filled_rectangle;
pub mod line_segment;
pub mod raw;

use context::Context;
use filled_circle::{FilledCircle, FilledCircleRenderer};
use filled_rectangle::{FilledRectangle, FilledRectangleRenderer};
use glam::Vec2;
use line_segment::{LineSegment, LineSegmentRenderer};
use raw::Raw;

use tracing::info;
use wgpu::util::DeviceExt;
use wgpu::{StoreOp, Texture};

pub struct Renderer {
    pub context: Context,
    projection_bind_group: wgpu::BindGroup,
    projection_buffer: wgpu::Buffer,

    filled_circle_renderer: FilledCircleRenderer,
    filled_rectangle_renderer: FilledRectangleRenderer,
    line_segment_renderer: LineSegmentRenderer,

    scale_factor: f64,
    size: Vec2,
}

impl Renderer {
    pub fn new(context: Context, scale_factor: f64, size: Vec2) -> eyre::Result<Self> {
        let (projection_buffer, projection_bind_group_layout, projection_bind_group) =
            Self::create_projection(&context, scale_factor, size);

        let filled_circle_renderer =
            FilledCircleRenderer::new(&context, &projection_bind_group_layout);
        let filled_rectangle_renderer =
            FilledRectangleRenderer::new(&context, &projection_bind_group_layout);
        let line_segment_renderer =
            LineSegmentRenderer::new(&context, &projection_bind_group_layout);

        Ok(Self {
            context,
            projection_bind_group,
            projection_buffer,
            filled_circle_renderer,
            filled_rectangle_renderer,
            line_segment_renderer,
            scale_factor,
            size,
        })
    }

    /// Return a orthographics projection matrix which will place the (0,0) into the left top
    /// corner.
    fn create_projection(
        context: &Context,
        scale_factor: f64,
        size: Vec2,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let projection_matrix = Self::generate_projection_matrix(size, scale_factor);

        let projection_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Projection Buffer"),
                    contents: projection_matrix.get_raw(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    // TODO: Check if the COPY_DST is needed.
                });

        let projection_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Projection Bind Group Descriptor"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let projection_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &projection_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: projection_buffer.as_entire_binding(),
                }],
                label: Some("Projection Bind Group"),
            });

        (
            projection_buffer,
            projection_bind_group_layout,
            projection_bind_group,
        )
    }

    fn generate_projection_matrix(size: Vec2, scale_factor: f64) -> glam::Mat4 {
        let half_width = size.x / (2. * scale_factor as f32);
        let half_height = size.y / (2. * scale_factor as f32);
        glam::Mat4::orthographic_lh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            0.0,
            -1.0,
        )
    }

    pub fn draw_full_circle(&mut self, full_circle: FilledCircle) {
        self.filled_circle_renderer.add_circle(full_circle);
    }
    pub fn draw_full_rectangle(&mut self, full_rectangle: FilledRectangle) {
        self.filled_rectangle_renderer.add_rectangle(full_rectangle);
    }

    pub fn draw_line_segment(&mut self, line_segment: LineSegment) {
        self.line_segment_renderer.add_line_segment(line_segment);
    }

    pub fn on_resize(&mut self, new_size: Vec2) {
        info!("on resize event received new_size: {:?}", new_size);
        self.size = new_size;
        self.context.queue.write_buffer(
            &self.projection_buffer,
            0,
            Self::generate_projection_matrix(self.size, self.scale_factor).get_raw(),
        );
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f64) {
        info!("on scale factor change scale_factor: {}", scale_factor);
        self.scale_factor = scale_factor;
    }

    pub fn render(&mut self, texture: &Texture) {
        info!("creating view from the texture");
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        info!("getting command encoder");
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("GPU Encoder"),
                });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shapes Renderer Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
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

            self.filled_circle_renderer.render(
                &self.context,
                &self.projection_bind_group,
                &mut render_pass,
            );
            self.filled_rectangle_renderer.render(
                &self.context,
                &self.projection_bind_group,
                &mut render_pass,
            );
            self.line_segment_renderer.render(
                &self.context,
                &self.projection_bind_group,
                &mut render_pass,
            );
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
    }
}
