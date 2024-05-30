pub mod buffers;
pub mod camera;
pub mod circle_rendering;
pub mod colors;
pub mod gpu_context;
pub mod line_rendering;
pub mod mesh_rendering;
pub mod primitives;
pub mod projection;
pub mod raw;
pub mod rectangle_rendering;
pub mod rendering_context;
pub mod resource_store;
pub mod transform;

use std::sync::Arc;

use glam::{Mat4, Vec2, Vec3};
use mesh_rendering::{MeshBundle, MeshRendering};
use resource_store::{gpu_mesh::GpuMeshId, render_pipeline::PipelineId};

use crate::{
    camera::PrimaryCamera,
    circle_rendering::{Circle, CircleLine, CircleRendering},
    gpu_context::GpuContext,
    line_rendering::{Line, LineRenderering},
    projection::CameraProjection,
    rectangle_rendering::{Rectangle, RectangleLine, RectangleRendering},
    rendering_context::RenderingContext,
    resource_store::shader::ShaderSource,
    transform::Transform,
};

pub struct CameraId;

pub struct Renderer {
    // TODO: This needs a bit of an discusion... I is public beccause you need
    // to be able to request stuff like shader or buffer layout etc. But on the
    // other hand. Primary camera has function which allow modification witthout
    // getting a reference to RenderingContext which holds the primary camera...
    pub rendering_context: RenderingContext,
    circle_rendering: CircleRendering,
    rectangle_rendering: RectangleRendering,
    line_rendering: LineRenderering,
    mesh_rendering: MeshRendering,
}

impl Renderer {
    pub fn new(gpu_context: &Arc<GpuContext>, primary_camera: PrimaryCamera) -> Self {
        let mut rendering_context = RenderingContext::new(gpu_context, primary_camera);
        let circle_rendering = CircleRendering::new(&mut rendering_context);
        let rectangle_rendering = RectangleRendering::new(&mut rendering_context);
        let line_rendering = LineRenderering::new(&mut rendering_context);
        let mesh_rendering = MeshRendering::new(&mut rendering_context);
        Self {
            rendering_context,
            circle_rendering,
            rectangle_rendering,
            line_rendering,
            mesh_rendering,
        }
    }

    // Thinking about consuming the Circle because it needs to be recreated in
    // the next render cycle anyway. On the other hand if it is an reference
    // then user can draw the same circle multiple times without much hassle.
    pub fn draw_circle(&mut self, transform: &Transform, circle: &Circle) {
        self.circle_rendering.add_circle(transform, circle);
    }

    pub fn draw_circle_line(&mut self, transform: &Transform, circle_line: &CircleLine) {
        self.circle_rendering
            .add_circle_line(transform, circle_line);
    }

    pub fn draw_rectangle(&mut self, transform: &Transform, rectangle: &Rectangle) {
        self.rectangle_rendering.add_rectangle(transform, rectangle);
    }

    pub fn draw_rectangle_line(&mut self, transform: &Transform, rectangle_line: &RectangleLine) {
        self.rectangle_rendering
            .add_rectangle_line(transform, rectangle_line);
    }

    pub fn draw_line(&mut self, line_segment: &Line) {
        self.line_rendering.add_line_segment(line_segment);
    }

    // This is probably something that could be made transparent.
    pub fn add_mesh(
        &mut self,
        vertices: &Vec<Vec3>,
        normals: &Vec<Vec3>,
        indices: &Vec<u32>,
    ) -> GpuMeshId {
        self.rendering_context
            .resource_store
            .build_gpu_mesh(vertices, normals, indices)
    }

    // This is probably something that could be made transparent.
    pub fn create_3d_pipeline(&mut self, shader: &ShaderSource) -> PipelineId {
        self.mesh_rendering
            .create_3d_pipeline(&mut self.rendering_context, shader)
    }

    pub fn draw_mesh(&mut self, transform: &Transform, mesh_bundle: &MeshBundle) {
        self.mesh_rendering.add_mesh_bundle(transform, mesh_bundle);
    }

    pub fn draw_instanced_mesh(&mut self, _transform: &[Transform], _mesh_bundle: &MeshBundle) {
        todo!()
    }

    // There are two options:
    //  * Either we implement `on_resize` and `on_scale_factor_change` callbacks,
    //  * or we try to determin the change in render funcion from texture size
    //    and from scale that would be passed in.
    //  Not sure which design is better.
    pub fn on_resize(&mut self, new_size: Vec2) {
        self.rendering_context
            .primary_camera
            .on_resize(new_size, &self.rendering_context.gpu_context);
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f64) {
        self.rendering_context
            .primary_camera
            .on_scale_factor_change(scale_factor as f32);
    }

    pub fn render(&mut self, target_texture: &wgpu::Texture) {
        let texture_view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .rendering_context
            .gpu_context
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("GPU Encoder"),
            });
        {
            let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })];

            let depth_stencil_attachment = self
                .rendering_context
                .primary_camera
                .depth_buffer()
                .as_ref()
                .map(
                    |(_depth_texture_config, _depth_texture, depth_texture_view)| {
                        wgpu::RenderPassDepthStencilAttachment {
                            view: depth_texture_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }
                    },
                );
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shapes Renderer Pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.circle_rendering
                .render(&self.rendering_context, &mut render_pass);
            self.rectangle_rendering
                .render(&self.rendering_context, &mut render_pass);
            self.line_rendering
                .render(&self.rendering_context, &mut render_pass);
            self.mesh_rendering.render(&self.rendering_context, &mut render_pass);

        }

        self.rendering_context
            .gpu_context
            .queue()
            .submit(std::iter::once(encoder.finish()));
    }

    // For later use???
    pub fn create_camera(
        &mut self,
        _transform: &Transform,
        _projection: CameraProjection,
    ) -> CameraId {
        todo!()
    }

    pub fn set_primary_camera_projection(&mut self, projection: &CameraProjection) {
        self.rendering_context
            .primary_camera
            .set_camera_projection(projection);
    }

    pub fn set_primary_camera_matrix(&mut self, matrix: &Mat4) {
        self.rendering_context
            .primary_camera
            .set_camera_matrix(matrix)
    }

    pub fn set_camera_projection(&mut self, _camera_id: &CameraId, _projection: &CameraProjection) {
        todo!()
    }

    pub fn set_camera_matrix(&mut self, _camera_id: &CameraId, _matrix: &Mat4) {
        todo!()
    }

    pub fn wgpu_limits() -> wgpu::Limits {
        RenderingContext::wgpu_limits()
    }
}
