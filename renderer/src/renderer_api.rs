use std::{path::Path, sync::Arc};

use glam::{Mat4, Vec2, Vec3};

use crate::{
    camera2::PrimaryCamera, circle_rendering::{Circle, CircleLine, CircleRendering}, gpu_context::GpuContext, projection2::CameraProjection, rectangle_rendering::{Rectangle, RectangleLine, RectangleRendering}, rendering_context::RenderingContext, resource_store::shader::{ShaderId, ShaderSource}, transform::Transform
};

#[derive(Clone)]
pub struct LineSegment {
    width: f32,
    color: Vec3,
}

#[derive(Clone)]
pub struct MeshId;

#[derive(Clone)]
pub struct MeshBundle {
    shader: ShaderId,
    mesh: MeshId,
}

pub struct CameraId;

pub struct Renderer {
    // TODO: This needs a bit of an discusion... I is public beccause you need
    // to be able to request stuff like shader or buffer layout etc. But on the
    // other hand. Primary camera has function which allow modification witthout
    // getting a reference to RenderingContext which holds the primary camera...
    pub rendering_context: RenderingContext,
    circle_rendering: CircleRendering,
    rectangle_rendering: RectangleRendering,
}

struct Mesh {}

impl Renderer {
    pub fn new(gpu_context: &Arc<GpuContext>, primary_camera: PrimaryCamera) -> Self {
        let mut rendering_context = RenderingContext::new(gpu_context, primary_camera);
        let circle_rendering = CircleRendering::new(&mut rendering_context);
        let rectangle_rendering = RectangleRendering::new(&mut rendering_context);
        Self { rendering_context, circle_rendering, rectangle_rendering }
    }

    // Thinking about consuming the Circle because it needs to be recreated in
    // the next render cycle anyway. On the other hand if it is an reference
    // then user can draw the same circle multiple times without much hassle.
    pub fn draw_circle(&mut self, transform: &Transform, circle: &Circle) {
        self.circle_rendering.add_circle(transform, circle);
    }

    pub fn draw_circle_line(&mut self, transform: &Transform, circle_line: &CircleLine) {
        self.circle_rendering.add_circle_line(transform, circle_line);
    }

    pub fn draw_rectangle(&mut self, transform: &Transform, rectangle: &Rectangle) {
        self.rectangle_rendering.add_rectangle(transform, rectangle);
    }

    pub fn draw_rectangle_line(&mut self, transform: &Transform, rectangle_line: &RectangleLine) {
        self.rectangle_rendering.add_rectangle_line(transform, rectangle_line);
    }

    pub fn draw_line_segment(&mut self, p1: &Vec3, p2: &Vec3, line_segment: &LineSegment) {
        todo!()
    }

    // This is probably something that could be made transparent.
    pub fn add_mesh(&mut self, stroke_rectangle: &Mesh) -> MeshId {
        todo!()
    }

    // This is probably something that could be made transparent.
    pub fn add_shader<P: AsRef<Path>>(&mut self, shader: &ShaderSource<P>) -> ShaderId {
        todo!()
    }

    pub fn draw_mesh(&mut self, transform: &Transform, mesh_bundle: &MeshBundle) {
        todo!()
    }

    pub fn draw_instanced_mesh(&mut self, transform: &Vec<Transform>, mesh_bundle: &MeshBundle) {
        todo!()
    }

    // There are two options:
    //  * Either we implement `on_resize` and `on_scale_factor_change` callbacks,
    //  * or we try to determin the change in render funcion from texture size
    //    and from scale that would be passed in.
    //  Not sure which design is better.
    pub fn on_resize(&mut self, new_size: Vec2) {
        self.rendering_context.primary_camera.on_resize(new_size);
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f64) {
        self.rendering_context
            .primary_camera
            .on_scale_factor_change(scale_factor as f32);
    }

    pub fn render(&mut self, target_texture: &wgpu::Texture) {
        let texture_view = target_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.rendering_context.gpu_context.device()
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Shapes Renderer Pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                //Some(RenderPassDepthStencilAttachment {
                //    view: &depth_texture_view,
                //    depth_ops: Some(Operations {
                //        load: LoadOp::Clear(1.0),
                //        store: StoreOp::Store,
                //    }),
                //    stencil_ops: None,
                //}),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.circle_rendering.render(&self.rendering_context, &mut render_pass);
            self.rectangle_rendering.render(&self.rendering_context, &mut render_pass);
        }

        self.rendering_context.gpu_context.queue().submit(std::iter::once(encoder.finish()));
    }

    // For later use???
    pub fn create_camera(
        &mut self,
        transform: &Transform,
        projection: CameraProjection,
    ) -> CameraId {
        todo!()
    }

    pub fn set_primary_camera_projection(&mut self, projection: &CameraProjection) {
        self.rendering_context.primary_camera.set_camera_projection(projection);
    }

    pub fn set_primary_camera_matrix(&mut self, matrix: &Mat4) {
        self.rendering_context.primary_camera.set_camera_matrix(matrix)
    }

    pub fn set_camera_projection(&mut self, camera_id: &CameraId, projection: &CameraProjection) {
        todo!()
    }

    pub fn set_camera_matrix(&mut self, camera_id: &CameraId, matrix: &Mat4) {
        todo!()
    }
}
