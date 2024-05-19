use std::sync::Arc;

use glam::{Vec2, Vec3};

#[derive(Clone)]
struct Transform;

#[derive(Clone)]
struct GpuContext;

#[derive(Clone)]
struct Circle{
    radius: f32,
    stroke: bool,
    color: Vec3,
}

#[derive(Clone)]
struct Rectangle{
    size: Vec2,
    stroke: bool,
    color: Vec3,
}

#[derive(Clone)]
struct LineSegment{
    width: f32,
    color: Vec3,
}

#[derive(Clone)]
struct MeshId;

#[derive(Clone)]
struct MeshBundle {
    shader: ShaderId,
    mesh: MeshId,
}

#[derive(Clone)]
struct Mesh{
    vertex_buffer: Vec<Vec3>,
    normal_buffer: Vec<Vec3>,
    index_buffer: Vec<u32>,
}

#[derive(Clone)]
struct Shader;

#[derive(Clone)]
struct ShaderId;

#[derive(Clone)]
struct CameraId;

#[derive(Clone)]
enum CameraProjection {
    Perspective(Perspective),
    Orthographic,
}

#[derive(Clone)]
pub struct Perspective {
    aspect: f32,
    fovy: f32, // In radians
    znear: f32,
    zfar: f32,
    scale: f32,
}

#[derive(Clone)]
pub struct Ortographic {
    width: f32,
    height: f32,
    depth: f32,
    scale_factor: f32,
}

#[derive(Clone)]
struct PrimaryCamera {
    projection: CameraProjection,
    transform: Transform,
    output_surface: wgpu::TextureFormat,
}

struct Renderer {
    gpu_context: Arc<GpuContext>
    // ...
}

impl Renderer {
    pub fn new(gpu_context: Arc<GpuContext>, primary_camera: &PrimaryCamera) -> Self{
        todo!()
    }

    // Thinking about consuming the Circle because it needs to be recreated in
    // the next render cycle anyway. On the other hand if it is an reference
    // then user can draw the same circle multiple times without much hassle.
    pub fn draw_circle(&mut self, transform: &Transform, full_circle: &Circle) {
        todo!()
    }

    pub fn draw_full_rectangle(&mut self, transform: &Transform,  rectangle: &Rectangle) {
        todo!()
    }

    pub fn draw_line_segment(&mut self, p1: &Vec3, p2: &Vec3, line_segment: &LineSegment) {
        todo!()
    }

    // This is probably something that could be made transparent.
    pub fn add_mesh(&mut self, stroke_rectangle: &Mesh) -> MeshId {
        todo!()
    }

    // This is probably something that could be made transparent.
    pub fn add_shader(&mut self, shader: &Shader) -> ShaderId {
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
        todo!()
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f64) {
        todo!()
    }

    pub fn render(&mut self, target_texture: &wgpu::Texture) {
        todo!()
    }

    pub fn get_pripmary_camera_id(&self) -> CameraId {
        todo!()
    }

    // For later use???
    pub fn create_camera(&mut self, transform: &Transform, projection: CameraProjection) -> CameraId {
        todo!()
    }

    pub fn set_camera_projection(&mut self, camera_id: &CameraId, projection: CameraProjection) {
        todo!()
    }

    pub fn set_camera_transform(&mut self, camera_id: &CameraId, transform: &Transform) {
        todo!()
    }
}
