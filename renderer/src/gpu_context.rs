use std::sync::Arc;

use wgpu::{Device, Queue};

#[derive(Clone)]
pub struct GpuContext {
    context: Arc<GpuContextInner>,
}

struct GpuContextInner {
    pub device: wgpu::Device,
    // Sends data and encoded commands to GPU
    pub queue: wgpu::Queue,
}

impl GpuContext {
    pub fn new(device: Device, queue: Queue) -> Self {
        Self {
            context: Arc::new(GpuContextInner { device, queue }),
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.context.device
    }
    pub fn queue(&self) -> &wgpu::Queue {
        &self.context.queue
    }
}
