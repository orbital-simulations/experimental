use tracing::info;
use wgpu::Adapter;
use winit::{dpi::PhysicalSize, window::Window};

// TODO: Try to think of a better name.
pub struct WindowedDevice<'a> {
    pub instance: wgpu::Instance,
    // Configuration of the droweable surface. It is here so we can reconfigure
    // resolution on resize event.
    pub config: wgpu::SurfaceConfiguration,
    // Logical representaion of the GPU device crates encoders and it used to
    // reconfigure the window.
    pub device: wgpu::Device,
    // Sends data and encoded commands to GPU
    pub queue: wgpu::Queue,
    pub window: &'a Window,
    // Drawable surface in the window.
    pub surface: wgpu::Surface<'a>,
    pub adapter: Adapter,
    pub size: PhysicalSize<u32>,
}

impl<'a> WindowedDevice<'a> {
    pub async fn new(window: &'a Window) -> eyre::Result<Self> {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        let size = window.inner_size();

        let surface = instance.create_surface(window)?;
        use eyre::OptionExt;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // Default is LowPower what is why I change it.
                // power_preference: wgpu::PowerPreference::default(),
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_eyre("Could not request adapter")?;
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("GPU device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await?;

        let swap_chain_capablities = surface.get_capabilities(&adapter);
        info!("surface formats: {:?}", swap_chain_capablities.formats);
        let swapchain_format = swap_chain_capablities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(swap_chain_capablities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            // vsync off
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: swap_chain_capablities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        Ok(Self {
            instance,
            surface,
            device,
            queue,
            config,
            adapter,
            size,
            window,
        })
    }

    pub fn prepare_encoder(
        &mut self,
    ) -> Result<
        (
            wgpu::CommandEncoder,
            wgpu::TextureView,
            wgpu::SurfaceTexture,
        ),
        wgpu::SurfaceError,
    > {
        info!("getting current surface texture");
        let output = self.surface.get_current_texture()?;
        info!("creating view from the texture");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        info!("getting command encoder");
        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("GPU Encoder"),
            });
        Ok((encoder, view, output))
    }
}
