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

use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use context::{Context, RenderingContext};
use custom_mesh_renderer::CustomMeshRenderer;
use filled_circle::{FilledCircle, FilledCircleRenderer};
use filled_rectangle::{FilledRectangle, FilledRectangleRenderer};
use glam::Vec2;
use line_segment::{LineSegment, LineSegmentRenderer};
use pipeline::RenderTargetDescription;
use projection::Projection;

use stroke_circle::{StrokeCircle, StrokeCircleRenderer};
use stroke_rectangle::{StrokeRectangle, StrokeRectangleRenderer};
use tracing::info;
use wgpu::{
    Color, LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
    StoreOp, Texture, TextureFormat, TextureViewDescriptor,
};

pub struct Renderer {
    pub rendering_context: RenderingContext,
    filled_circle_renderer: FilledCircleRenderer,
    stroke_circle_renderer: StrokeCircleRenderer,
    filled_rectangle_renderer: FilledRectangleRenderer,
    stroke_rectangle_renderer: StrokeRectangleRenderer,
    line_segment_renderer: LineSegmentRenderer,
    custom_mesh_renderers: HashMap<TypeId, CustomMeshRenderer>,
    depth_texture: Option<Texture>,
    window_render_target_description: RenderTargetDescription,
}

#[derive(Default)]
pub struct Scene {
    filled_circles: Vec<FilledCircle>,
    stroke_circles: Vec<StrokeCircle>,
    filled_rectangles: Vec<FilledRectangle>,
    stroke_rectangles: Vec<StrokeRectangle>,
    line_segments: Vec<LineSegment>,
    // TODO expose custom mesh data
}

impl Scene {
    pub fn draw_filled_circle(&mut self, circle: FilledCircle) {
        self.filled_circles.push(circle);
    }

    pub fn draw_stroke_circle(&mut self, circle: StrokeCircle) {
        self.stroke_circles.push(circle);
    }

    pub fn draw_filled_rectangle(&mut self, rectangle: FilledRectangle) {
        self.filled_rectangles.push(rectangle);
    }

    pub fn draw_stroke_rectangle(&mut self, rectangle: StrokeRectangle) {
        self.stroke_rectangles.push(rectangle);
    }

    pub fn draw_line_segment(&mut self, line_segment: LineSegment) {
        self.line_segments.push(line_segment);
    }
}

pub trait CustomRenderer {}

impl Renderer {
    pub fn new(
        context: &Context,
        projection: Projection,
        main_surface_format: TextureFormat,
    ) -> eyre::Result<Self> {
        let window_render_target_description = RenderTargetDescription {
            multisampling: 1,
            depth_texture: Some(TextureFormat::Depth32Float),
            targets: vec![main_surface_format],
        };
        let rendering_context = RenderingContext::new(context, projection);
        let filled_circle_renderer = FilledCircleRenderer::new(context);
        let stroke_circle_renderer = StrokeCircleRenderer::new(context);
        let filled_rectangle_renderer = FilledRectangleRenderer::new(context);
        let stroke_rectangle_renderer = StrokeRectangleRenderer::new(context);
        let line_segment_renderer = LineSegmentRenderer::new(context);

        Ok(Self {
            rendering_context,
            filled_circle_renderer,
            stroke_circle_renderer,
            filled_rectangle_renderer,
            stroke_rectangle_renderer,
            line_segment_renderer,
            custom_mesh_renderers: HashMap::new(),
            depth_texture: None,
            window_render_target_description,
        })
    }

    pub fn add_custom_mesh_renderer<K>(
        &mut self,
        renderer_id: &K,
        custom_mesh_renderer: CustomMeshRenderer,
    ) where
        K: CustomRenderer + Any,
    {
        self.custom_mesh_renderers
            .insert(renderer_id.type_id(), custom_mesh_renderer);
    }

    pub fn remove_custom_mesh_renderer<K>(&mut self, renderer_id: &K)
    where
        K: CustomRenderer + Any,
    {
        self.custom_mesh_renderers.remove(&renderer_id.type_id());
    }

    pub fn on_resize(&mut self, context: &Context, new_size: Vec2) {
        info!("on resize event received new_size: {:?}", new_size);
        self.rendering_context
            .camera_mut()
            .on_resize(new_size, context);
        self.depth_texture = None;
    }

    pub fn on_scale_factor_change(&mut self, context: &Context, scale_factor: f64) {
        info!("on scale factor change scale_factor: {}", scale_factor);
        self.rendering_context
            .camera_mut()
            .on_scale_factor_change(scale_factor, context);
    }

    pub fn render(&mut self, context: &Context, texture: &Texture, size: Vec2, scene: &Scene) {
        info!("getting command encoder");
        let depth_texture = self.depth_texture.get_or_insert_with(|| {
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
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[TextureFormat::Depth32Float],
            };
            context.device.create_texture(&depth_texture_description)
        });

        let depth_texture_view = depth_texture.create_view(&TextureViewDescriptor::default());

        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("GPU Encoder"),
            });
        {
            let color_attachments = [Some(RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })];

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shapes Renderer Pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &depth_texture_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.filled_circle_renderer.render(
                context,
                &self.rendering_context,
                &mut render_pass,
                &self.window_render_target_description,
                &scene.filled_circles,
            );
            self.stroke_circle_renderer.render(
                context,
                &self.rendering_context,
                &mut render_pass,
                &self.window_render_target_description,
                &scene.stroke_circles,
            );
            self.filled_rectangle_renderer.render(
                context,
                &self.rendering_context,
                &mut render_pass,
                &self.window_render_target_description,
                &scene.filled_rectangles,
            );
            self.line_segment_renderer.render(
                context,
                &self.rendering_context,
                &mut render_pass,
                &self.window_render_target_description,
                &scene.line_segments,
            );
            self.stroke_rectangle_renderer.render(
                context,
                &self.rendering_context,
                &mut render_pass,
                &self.window_render_target_description,
                &scene.stroke_rectangles,
            );

            for custom_mesh_renderer in self.custom_mesh_renderers.values_mut() {
                custom_mesh_renderer.render(
                    &self.rendering_context,
                    context,
                    &mut render_pass,
                    &self.window_render_target_description,
                );
            }
        }

        context.queue.submit(std::iter::once(encoder.finish()));
    }
}
