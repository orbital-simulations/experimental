use tracing::info;
use wgpu::{Device, Queue, Texture};

pub struct Context<'a> {
    pub device: &'a wgpu::Device,
    // Sends data and encoded commands to GPU
    pub queue: &'a wgpu::Queue,
    // Drawable surface in the window.
}

impl <'a> Context<'a> {
    pub async fn new(device: &'a Device, queue: &'a Queue) -> eyre::Result<Self> {
        Ok(Self {
            device,
            queue,
        })
    }

    pub fn prepare_encoder(
        &mut self,
        texture: &Texture,
    ) -> Result<
        (
            wgpu::CommandEncoder,
            wgpu::TextureView,
        ),
        wgpu::SurfaceError,
    > {
        info!("creating view from the texture");
        let view = texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        info!("getting command encoder");
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("GPU Encoder"),
            });
        Ok((encoder, view))
    }
}
