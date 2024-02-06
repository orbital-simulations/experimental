use wgpu::{Device, Queue, TextureFormat};

pub struct Context {
    pub device: wgpu::Device,
    // Sends data and encoded commands to GPU
    pub queue: wgpu::Queue,
    pub output_texture_format: TextureFormat,
}

impl Context {
    pub fn new(device: Device, queue: Queue, texture_format: TextureFormat) -> Self {
        Self {
            device,
            queue,
            output_texture_format: texture_format,
        }
    }
}
