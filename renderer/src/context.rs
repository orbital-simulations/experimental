use wgpu::{Device, Queue};

use crate::{camera::Camera, projection::Projection};

pub struct Context {
    pub device: wgpu::Device,
    // Sends data and encoded commands to GPU
    pub queue: wgpu::Queue,
}

impl Context {
    pub fn new(device: Device, queue: Queue) -> Self {
        Self { device, queue }
    }
}

pub struct RenderingContext {
    camera: Camera,
}

impl RenderingContext {
    pub fn new(context: &Context, projection: Projection) -> Self {
        let camera = Camera::new(context, projection);
        Self { camera }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }
}
