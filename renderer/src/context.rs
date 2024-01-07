use tracing::info;
use wgpu::{Device, Queue, Texture, TextureFormat};

pub struct Context {
    pub device: wgpu::Device,
    // Sends data and encoded commands to GPU
    pub queue: wgpu::Queue,
    pub texture_format: TextureFormat,
}

impl Context {
    pub fn new(device: Device, queue: Queue, texture_format: TextureFormat) -> eyre::Result<Self> {
        Ok(Self {
            device,
            queue,
            texture_format,
        })
    }

    pub fn prepare_encoder(
        &self,
        texture: &Texture,
    ) -> Result<(wgpu::CommandEncoder, wgpu::TextureView), wgpu::SurfaceError> {
        info!("creating view from the texture");
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        info!("getting command encoder");
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("GPU Encoder"),
            });
        Ok((encoder, view))
    }
}
