use std::time::Instant;

use glam::{Vec2, vec2};
use renderer::Renderer;
use tracing::{debug, info};
use wgpu::{Instance, InstanceDescriptor, Backends, InstanceFlags, Gles3MinorVersion, RequestAdapterOptions, PowerPreference, DeviceDescriptor, Features, Limits, SurfaceConfiguration, TextureUsages, PresentMode, Surface};
use wgpu::util::DeviceExt;

use winit::event::WindowEvent::{
    CloseRequested, KeyboardInput, MouseInput, RedrawRequested, Resized, ScaleFactorChanged,
};
use winit::keyboard::NamedKey;

use winit::window::Window;
use winit::{dpi::PhysicalSize, event::Event, event_loop::EventLoop};

use renderer::context::Context;

pub struct GameEngine<'a> {
    window: Window,
    pub last_frame_delta: f32,
    timer: Instant,
    renderer: Renderer<'a>,
    surface_configuration: SurfaceConfiguration,
    surface: Surface<'a>,
    size: PhysicalSize<u32>,
}

fn size_to_vec2(size: &PhysicalSize<u32>) -> Vec2 {
    vec2(size.width as f32, size.height as f32)
}

impl <'a> GameEngine<'a> {
    pub async fn new() -> eyre::Result<(Self, EventLoop<()>)> {
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
        let surface = instance.create_surface(&window)?;
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
        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            // vsync off
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: swap_chain_capablities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &surface_configuration);
        let context = Context::new(device, queue, swapchain_format)?;
        let renderer = Renderer::new(&context, scale_factor, size_to_vec2(&size))?;

        Ok((
            Self {
                last_frame_delta: 0.,
                timer: Instant::now(),
                window,
                renderer,
                surface_configuration,
                surface,
            },
            event_loop,
        ))
    }

    pub fn run<State, FSetup, FUpdate>(
        &mut self,
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

    fn redraw_requested<State, FUpdate>(&mut self, state: &mut State, update: FUpdate)
    where
        FUpdate: Fn(&mut State, &mut GameEngine),
    {
        info!("rendering as per the RedrawRequested was received");
        // TODO: It is possible this may cause some time shuttering in the
        // first frame. Maybe the first delta should be 0??
        self.last_frame_delta = self.timer.elapsed().as_secs_f32();
        self.timer = Instant::now();
        update(state, self);

        match self.windowed_device.prepare_encoder() {
            Ok((mut encoder, view, output)) => {
                self.renderer.render(texture)

                // This whould be moved into the renderer. Maybe for redering
                // we could create some guad object which would present the
                // render on drop???
                self.renderer.context
                    .queue
                    .submit(std::iter::once(encoder.finish()));
                output.present();
                self.window.request_redraw();
            }
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.on_resize(self.size);
            }
            Err(_) => {
                panic!("panicing in the panic");
            }
        }
    }

    pub fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
        info!("on resize event received new_size: {:?}", new_size);
        self.surface_configuration.width = new_size.width;
        self.surface_configuration.height = new_size.height;
        self.surface
            .configure(&self.renderer.context.device, &self.surface_configuration);
        self.renderer.on_resize(size_to_vec2(&new_size));
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f64) {
        info!("on scale factor change scale_factor: {}", scale_factor);
        self.renderer.on_scale_factor_change(scale_factor);
    }
}
