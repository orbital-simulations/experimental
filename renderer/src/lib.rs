pub mod buffers;
pub mod camera;
pub mod colors;
pub mod context;
pub mod custom_mesh_renderer;
pub mod filled_circle;
pub mod filled_rectangle;
pub mod line_segment;
pub mod mesh;
pub mod pipeline;
pub mod projection;
pub mod raw;
pub mod stroke_circle;
pub mod stroke_rectangle;
pub mod render_pass;
//pub mod api_experiments;

use std::rc::Rc;

use context::{Context, RenderingContext};
use custom_mesh_renderer::CustomMeshRenderer;
use filled_circle::{FilledCircle, FilledCircleRenderer};
use filled_rectangle::{FilledRectangle, FilledRectangleRenderer};
use glam::Vec2;
use line_segment::{LineSegment, LineSegmentRenderer};
use projection::Projection;

use render_pass::{RenderTarget, RenderTargetDescription};
use stroke_circle::{StrokeCircle, StrokeCircleRenderer};
use stroke_rectangle::{StrokeRectangle, StrokeRectangleRenderer};
use tracing::info;
use wgpu::{
    Operations, StoreOp, Texture, TextureFormat,
};

pub struct Renderer {
    pub context: Context,
    rendering_context: RenderingContext,
    filled_circle_renderer: FilledCircleRenderer,
    stroke_circle_renderer: StrokeCircleRenderer,
    filled_rectangle_renderer: FilledRectangleRenderer,
    stroke_rectangle_renderer: StrokeRectangleRenderer,
    line_segment_renderer: LineSegmentRenderer,
    custom_mesh_renderers: Vec<CustomMeshRenderer>,
    size: Vec2,
    render_target: RenderTarget,
}

impl Renderer {
    pub fn new(context: Context, size: Vec2, projection: Projection, main_surface: Texture) -> eyre::Result<Self> {
        let rendering_context = RenderingContext::new(&context, projection);

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

        let render_target = RenderTarget::new(&context, &RenderTargetDescription{
            name: "main rendering target".to_string(),
            multisampling: 1,
            depth_texture: Some((Rc::new(depth_texture),
                     Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    })),
            targets: &[
                (Rc::new(main_surface), wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
})]});

        let filled_circle_renderer =
            FilledCircleRenderer::new(&context, &rendering_context, &render_target);
        let stroke_circle_renderer =
            StrokeCircleRenderer::new(&context, &rendering_context, &render_target);
        let filled_rectangle_renderer =
            FilledRectangleRenderer::new(&context, &rendering_context, &render_target);
        let stroke_rectangle_renderer =
            StrokeRectangleRenderer::new(&context, &rendering_context, &render_target);
        let line_segment_renderer =
            LineSegmentRenderer::new(&context, &rendering_context, &render_target);

        Ok(Self {
            context,
            filled_circle_renderer,
            stroke_circle_renderer,
            filled_rectangle_renderer,
            stroke_rectangle_renderer,
            line_segment_renderer,
            size,
            custom_mesh_renderers: vec![],
            rendering_context,
            render_target,
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

    pub fn add_custom_mesh_renderer(&mut self, custom_mesh_renderer: CustomMeshRenderer) {
        self.custom_mesh_renderers.push(custom_mesh_renderer);
    }

    pub fn on_resize(&mut self, new_size: Vec2) {
        info!("on resize event received new_size: {:?}", new_size);
        self.size = new_size;
        self.rendering_context.camera_mut().on_resize(new_size, &self.context);

        self.render_target.resize(new_size.as_uvec2());
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f64) {
        info!("on scale factor change scale_factor: {}", scale_factor);
        self.rendering_context.camera_mut()
            .on_scale_factor_change(scale_factor, &self.context);
    }

    pub fn render(&mut self, texture: Texture) {
        info!("getting command encoder");
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("GPU Encoder"),
                });
        {
            // FIXME: This can't be taking hold of the texture. And the
            // RenderTarget should have a simple way to change the texture it
            // refers to so we can support swapchians in a bit nicer way.
            self.render_target.targets[0].0 = Rc::new(texture);
            let mut render_pass = self.render_target.create_render_pass(&mut encoder);

            self.filled_circle_renderer
                .render(&self.context, &self.rendering_context, &mut render_pass);
            self.stroke_circle_renderer
                .render(&self.context, &self.rendering_context, &mut render_pass);
            self.filled_rectangle_renderer
                .render(&self.context,  &self.rendering_context,&mut render_pass);
            self.line_segment_renderer
                .render(&self.context,  &self.rendering_context,&mut render_pass);
            self.stroke_rectangle_renderer
                .render(&self.context,  &self.rendering_context,&mut render_pass);

            for custom_mesh_renderer in self.custom_mesh_renderers.iter_mut() {
                custom_mesh_renderer.render(&self.rendering_context, &mut render_pass);
            }
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
    }
}
