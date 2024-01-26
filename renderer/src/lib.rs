pub mod buffers;
pub mod colors;
pub mod context;
pub mod custom_mesh_renderer;
pub mod filled_circle;
pub mod filled_rectangle;
pub mod line_segment;
pub mod mesh;
pub mod projection;
pub mod raw;
pub mod stroke_circle;
pub mod stroke_rectangle;
pub mod api_experiments;

use context::{Context, RenderingContext};
use custom_mesh_renderer::CustomMashRenderer;
use filled_circle::{FilledCircle, FilledCircleRenderer};
use filled_rectangle::{FilledRectangle, FilledRectangleRenderer};
use glam::Vec2;
use line_segment::{LineSegment, LineSegmentRenderer};
use projection::{Projection, ProjectionManipulation};

use stroke_circle::{StrokeCircle, StrokeCircleRenderer};
use stroke_rectangle::{StrokeRectangle, StrokeRectangleRenderer};
use tracing::{info, warn};
use wgpu::{
    Operations, RenderPassDepthStencilAttachment, StoreOp, Texture, TextureFormat, TextureView,
};

pub struct Renderer {
    pub context: Context,
    pub renderer_context: RenderingContext,

    filled_circle_renderer: FilledCircleRenderer,
    stroke_circle_renderer: StrokeCircleRenderer,
    filled_rectangle_renderer: FilledRectangleRenderer,
    stroke_rectangle_renderer: StrokeRectangleRenderer,
    line_segment_renderer: LineSegmentRenderer,
    custom_mesh_renderers: Vec<CustomMashRenderer>,
    projection: Projection,
    size: Vec2,

    depth_texture: Texture,
    depth_view: TextureView,
}

impl Renderer {
    pub fn new(context: Context, size: Vec2, projection: Projection) -> eyre::Result<Self> {
        let renderer_context = RenderingContext::new(&context, &projection);

        let filled_circle_renderer =
            FilledCircleRenderer::new(&context, &renderer_context.common_bind_group_layout);
        let stroke_circle_renderer =
            StrokeCircleRenderer::new(&context, &renderer_context.common_bind_group_layout);
        let filled_rectangle_renderer =
            FilledRectangleRenderer::new(&context, &renderer_context.common_bind_group_layout);
        let stroke_rectangle_renderer =
            StrokeRectangleRenderer::new(&context, &renderer_context.common_bind_group_layout);
        let line_segment_renderer =
            LineSegmentRenderer::new(&context, &renderer_context.common_bind_group_layout);

        let depth_texture_size = wgpu::Extent3d {
            width: size.x as u32,
            height: size.y as u32,
            depth_or_array_layers: 1,
        };
        let depth_texture_description = wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: depth_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[TextureFormat::Depth32Float],
        };
        let depth_texture = context.device.create_texture(&depth_texture_description);
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        Ok(Self {
            renderer_context,
            context,
            filled_circle_renderer,
            stroke_circle_renderer,
            filled_rectangle_renderer,
            stroke_rectangle_renderer,
            line_segment_renderer,
            size,
            custom_mesh_renderers: vec![],
            projection,
            depth_texture,
            depth_view,
        })
    }

    pub fn draw_full_circle(&mut self, full_circle: FilledCircle) {
        self.filled_circle_renderer.add_circle(full_circle);
    }

    pub fn draw_stroke_circle(&mut self, stroke_circle: StrokeCircle) {
        self.stroke_circle_renderer.add_stroke_circle(stroke_circle);
    }

    pub fn draw_full_rectangle(&mut self, full_rectangle: FilledRectangle) {
        self.filled_rectangle_renderer.add_rectangle(full_rectangle);
    }

    pub fn draw_stroke_rectangle(&mut self, stroke_rectangle: StrokeRectangle) {
        self.stroke_rectangle_renderer
            .add_rectangle(stroke_rectangle);
    }

    pub fn draw_line_segment(&mut self, line_segment: LineSegment) {
        self.line_segment_renderer.add_line_segment(line_segment);
    }

    pub fn add_custom_mesh_renderer(&mut self, custom_mesh_renderer: CustomMashRenderer) {
        self.custom_mesh_renderers.push(custom_mesh_renderer);
    }

    pub fn on_resize(&mut self, new_size: Vec2) {
        info!("on resize event received new_size: {:?}", new_size);
        self.size = new_size;
        self.projection.resize(new_size.x, new_size.y);
        self.renderer_context
            .set_projection_matrix(&self.context, &self.projection.make_projection_matrix());

        let depth_texture_size = wgpu::Extent3d {
            width: new_size.x as u32,
            height: new_size.y as u32,
            depth_or_array_layers: 1,
        };
        let depth_texture_description = wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: depth_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[TextureFormat::Depth32Float],
        };

        self.depth_texture = self
            .context
            .device
            .create_texture(&depth_texture_description);
        self.depth_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f64) {
        info!("on scale factor change scale_factor: {}", scale_factor);
        self.projection.scale(scale_factor as f32);
        self.renderer_context
            .set_projection_matrix(&self.context, &self.projection.make_projection_matrix());
    }

    pub fn render(&mut self, texture: &Texture) {
        warn!("projection: {:?}", self.projection);
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
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.renderer_context.bind(&mut render_pass, 0);

            self.filled_circle_renderer
                .render(&self.context, &mut render_pass);
            self.stroke_circle_renderer
                .render(&self.context, &mut render_pass);
            self.filled_rectangle_renderer
                .render(&self.context, &mut render_pass);
            self.line_segment_renderer
                .render(&self.context, &mut render_pass);
            self.stroke_rectangle_renderer
                .render(&self.context, &mut render_pass);

            for custom_mesh_renderer in self.custom_mesh_renderers.iter_mut() {
                custom_mesh_renderer
                    .render(&self.renderer_context.common_bind_group, &mut render_pass);
            }
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
    }
}
