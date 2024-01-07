use glam::{vec2, Vec2};
use renderer::{context::Context, Renderer};
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use wgpu::{
    Backends, DeviceDescriptor, Features, Gles3MinorVersion, Instance, InstanceDescriptor,
    InstanceFlags, Limits, PowerPreference, PresentMode, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureUsages,
};
use winit::{
    dpi::PhysicalSize, event::Event, event_loop::EventLoop, keyboard::NamedKey, window::Window,
};

fn physical_size_to_vec(physical_size: &PhysicalSize<u32>) -> Vec2 {
    vec2(physical_size.width as f32, physical_size.height as f32)
}

pub struct Loop<'a> {
    surface: Surface<'a>,
    surface_configuration: SurfaceConfiguration,
    renderer: Renderer,
}

impl<'a> Loop<'a> {
    pub async fn setup() -> color_eyre::eyre::Result<(Self, EventLoop<()>)> {
        let fmt_layer = fmt::layer().pretty();
        let filter_layer = EnvFilter::from_default_env();
        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(filter_layer)
            .init();
        color_eyre::install()?;

        let event_loop = EventLoop::new().expect("Can't create the event loop");
        let window = Window::new(&event_loop).expect("Can't create the window");
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: InstanceFlags::default(),
            gles_minor_version: Gles3MinorVersion::default(),
        });

        let size = window.inner_size();
        let scale_factor = window.scale_factor();
        let surface = instance.create_surface(window)?;
        use eyre::OptionExt;
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_eyre("Could not request adapter")?;
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("GPU device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
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
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            // vsync off
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: swap_chain_capablities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        let context = Context::new(device, queue, swapchain_format)?;
        let renderer = renderer::Renderer::new(context, scale_factor, physical_size_to_vec(&size))?;
        Ok((
            Self {
                surface,
                surface_configuration: config,
                renderer,
            },
            event_loop,
        ))
    }

    pub fn run<FRender>(&mut self, event_loop: EventLoop<()>, render: FRender) -> eyre::Result<()>
    where
        FRender: Fn(&mut Renderer),
    {
        use winit::event::WindowEvent::*;
        event_loop.run(move |event, elwt| {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    ScaleFactorChanged {
                        scale_factor,
                        inner_size_writer: _,
                    } => {
                        self.renderer.on_scale_factor_change(scale_factor);
                    }
                    Resized(new_size) => {
                        info!("on resize event received new_size: {:?}", new_size);
                        self.surface_configuration.width = new_size.width;
                        self.surface_configuration.height = new_size.height;
                        self.surface
                            .configure(&self.renderer.context.device, &self.surface_configuration);
                        self.renderer.on_resize(physical_size_to_vec(&new_size))
                    }
                    CloseRequested => elwt.exit(),
                    KeyboardInput {
                        device_id: _,
                        event,
                        is_synthetic: _,
                    } => {
                        info!("Escape was pressed; terminating the event loop");
                        if let winit::keyboard::Key::Named(NamedKey::Escape) = event.logical_key {
                            elwt.exit()
                        }
                    }
                    MouseInput {
                        device_id: _,
                        state: _,
                        button: _,
                    } => (),
                    RedrawRequested => {
                        render(&mut self.renderer);
                        let texture = self
                            .surface
                            .get_current_texture()
                            .expect("can't get current swapchain texture");
                        self.renderer
                            .render(&texture.texture)
                            .expect("The renderer failed to draw a frame");
                        texture.present();
                    }
                    _ => {}
                }
            }
        })?;
        Ok(())
    }
}
