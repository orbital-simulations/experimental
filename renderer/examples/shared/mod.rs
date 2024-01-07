use renderer::context::Context;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use wgpu::Surface;
use winit::{event_loop::EventLoop, window::Window};

pub async fn setup<'a>() -> color_eyre::eyre::Result<(EventLoop<()>, Surface<'a>, Context<'a>)> {
    let fmt_layer = fmt::layer().pretty();
    let filter_layer = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
    color_eyre::install()?;

    let event_loop = EventLoop::new().expect("Can't create the event loop");
    let window = Window::new(&event_loop).expect("Can't create the window");
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
    let context = Context::new(&device, &queue, swapchain_format)?;

    Ok((event_loop, surface, context))
}


pub fn run<State, FSetup, FUpdate>(
    event_loop: EventLoop<()>,
    setup: FSetup,
    update: &FUpdate,
) -> eyre::Result<()>
where
    FSetup: FnOnce() -> State,
    FUpdate: Fn(&mut State, &mut GameEngine),
{
    let mut state = setup();
    // Restart timer just in case.
    self.timer = Instant::now();
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                ScaleFactorChanged {
                    scale_factor,
                    inner_size_writer: _,
                } => {
                    self.on_scale_factor_change(scale_factor);
                }
                Resized(physical_size) => self.on_resize(physical_size),
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
                    self.redraw_requested(&mut state, update);
                }
                _ => {
                    debug!("UNKNOWN WINDOW EVENT RECEIVED: {:?}", event);
                }
            },
            Event::AboutToWait => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                //self.windowed_device.window.request_redraw();
            }
            _ => {
                debug!("UNKNOWN EVENT RECEIVED: {:?}", event);
            }
        }
    })?;
    Ok(())
}
