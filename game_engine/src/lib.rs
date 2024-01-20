pub mod camera;
mod egui_integration;
pub mod inputs;
pub mod mesh;

use camera::{Camera, CameraController};
use egui_integration::EguiIntegration;
use glam::{vec2, vec3, Vec2};
use inputs::Inputs;
use renderer::projection::{OrtographicProjection, PerspectiveProjection, Projection};
use renderer::Renderer;
use std::f32::consts::PI;
use std::time::Instant;
use tracing::{debug, info, warn};
use wgpu::{
    Backends, DeviceDescriptor, Features, Gles3MinorVersion, Instance, InstanceDescriptor,
    InstanceFlags, Limits, PowerPreference, PresentMode, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureUsages,
};

use winit::event::WindowEvent::{
    CloseRequested, KeyboardInput, MouseInput, RedrawRequested, Resized, ScaleFactorChanged,
};
use winit::keyboard::NamedKey;

use winit::window::Window;
use winit::{dpi::PhysicalSize, event::Event, event_loop::EventLoop};

use renderer::context::Context;

pub struct GameEngine<'a> {
    window: &'a Window,
    pub last_frame_delta: f32,
    timer: Instant,
    pub renderer: Renderer,
    surface_configuration: SurfaceConfiguration,
    surface: Surface<'a>,
    size: PhysicalSize<u32>,
    inputs: Inputs,
    camera_controler: CameraController,
    camera: Camera,
    egui_integration: EguiIntegration,
}

fn size_to_vec2(size: &PhysicalSize<u32>) -> Vec2 {
    vec2(size.width as f32, size.height as f32)
}

pub struct MkGameEngine {
    projection: ProjectionInit,
    camera: Camera,
}

pub fn game_engine_3d_parameters() -> MkGameEngine {
    MkGameEngine {
        projection: ProjectionInit::Perspective,
        camera: Camera::new(vec3(0., 10., 0.), 0., 0.),
    }
}

pub fn game_engine_2_5d_parameters() -> MkGameEngine {
    MkGameEngine {
        projection: ProjectionInit::Ortographic,
        camera: Camera::new(vec3(0., 0., 10.), 0., -PI / 2.),
    }
}

enum ProjectionInit {
    Perspective,
    Ortographic,
}

impl<'a> GameEngine<'a> {
    pub async fn new(
        event_loop: EventLoop<()>,
        window: &'a Window,
        game_engine_parameters: MkGameEngine,
    ) -> eyre::Result<(Self, EventLoop<()>)> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: InstanceFlags::default(),
            gles_minor_version: Gles3MinorVersion::default(),
        });

        let size = window.inner_size();
        let scale_factor = window.scale_factor() as f32;
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
        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            // vsync off
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: swap_chain_capablities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };
        surface.configure(&device, &surface_configuration);
        let context = Context::new(device, queue, swapchain_format);
        let projection = match game_engine_parameters.projection {
            ProjectionInit::Perspective => Projection::Perspective(PerspectiveProjection::new(
                size.width as f32,
                size.height as f32,
                45.,
                0.01,
                1000.,
                scale_factor,
            )),
            ProjectionInit::Ortographic => Projection::Ortographic(OrtographicProjection::new(
                size.width as f32,
                size.height as f32,
                100.,
                scale_factor,
            )),
        };

        let egui_integration =
            EguiIntegration::new(window, &context.device, surface_configuration.format);

        let renderer = Renderer::new(context, size_to_vec2(&size), projection)?;

        Ok((
            Self {
                last_frame_delta: 0.,
                timer: Instant::now(),
                window,
                renderer,
                surface_configuration,
                surface,
                size,
                inputs: Inputs::new(),
                camera_controler: CameraController::new(100., 1.),
                camera: game_engine_parameters.camera,
                egui_integration,
            },
            event_loop,
        ))
    }

    pub fn run<State, FSetup, FUpdate, FRender>(
        &mut self,
        event_loop: EventLoop<()>,
        setup: FSetup,
        update: &FUpdate,
        render: &FRender,
    ) -> eyre::Result<()>
    where
        FSetup: FnOnce(&mut GameEngine) -> State,
        FUpdate: Fn(&mut State, &mut GameEngine),
        FRender: Fn(&State, &mut Renderer),
    {
        let mut state = setup(self);
        // Restart timer just in case the setup takes forever.
        self.timer = Instant::now();
        info!("rendering firs frame with initial state");
        render(&mut state, &mut self.renderer);
        event_loop.run(move |event, elwt| match event {
            Event::WindowEvent { event, .. } => {
                let res = self.egui_integration.on_window_event(self.window, &event);
                if res.consumed {
                } else {
                    match event {
                        ScaleFactorChanged {
                            scale_factor,
                            inner_size_writer: _,
                        } => {
                            self.on_scale_factor_change(scale_factor);
                            self.egui_integration
                                .on_scale_factor_change(scale_factor as f32);
                        }
                        Resized(physical_size) => {
                            self.on_resize(physical_size);
                            self.egui_integration.on_resize(physical_size);
                        }
                        CloseRequested => elwt.exit(),
                        KeyboardInput {
                            device_id: _,
                            event,
                            is_synthetic: _,
                        } => {
                            self.inputs.update_key(&event.physical_key, &event.state);
                            info!("Escape was pressed; terminating the event loop");
                            if let winit::keyboard::Key::Named(NamedKey::Escape) = event.logical_key
                            {
                                elwt.exit()
                            }
                        }
                        MouseInput {
                            device_id: _,
                            state,
                            button,
                        } => {
                            self.inputs.update_mouse_buttons(&button, &state);
                        }
                        winit::event::WindowEvent::CursorMoved {
                            device_id: _,
                            position,
                        } => {
                            let tmp: (f32, f32) = position.into();
                            self.inputs.update_cursor_move(tmp.into());
                        }
                        RedrawRequested => {
                            self.redraw_requested(&mut state, update, render);
                            self.inputs.reset_events();
                        }
                        //winit::event::WindowEvent::ActivationTokenDone { serial, token } => todo!(),
                        //winit::event::WindowEvent::Moved(_) => todo!(),
                        //winit::event::WindowEvent::Destroyed => todo!(),
                        //winit::event::WindowEvent::DroppedFile(_) => todo!(),
                        //winit::event::WindowEvent::HoveredFile(_) => todo!(),
                        //winit::event::WindowEvent::HoveredFileCancelled => todo!(),
                        //winit::event::WindowEvent::Focused(_) => todo!(),
                        //winit::event::WindowEvent::ModifiersChanged(_) => todo!(),
                        //winit::event::WindowEvent::Ime(_) => todo!(),
                        //winit::event::WindowEvent::CursorEntered { device_id } => todo!(),
                        //winit::event::WindowEvent::CursorLeft { device_id } => todo!(),
                        //winit::event::WindowEvent::MouseWheel { device_id, delta, phase } => todo!(),
                        //winit::event::WindowEvent::TouchpadMagnify { device_id, delta, phase } => todo!(),
                        //winit::event::WindowEvent::SmartMagnify { device_id } => todo!(),
                        //winit::event::WindowEvent::TouchpadRotate { device_id, delta, phase } => todo!(),
                        //winit::event::WindowEvent::TouchpadPressure { device_id, pressure, stage } => todo!(),
                        //winit::event::WindowEvent::AxisMotion { device_id, axis, value } => todo!(),
                        //winit::event::WindowEvent::Touch(_) => todo!(),
                        //winit::event::WindowEvent::ThemeChanged(_) => todo!(),
                        //winit::event::WindowEvent::Occluded(_) => todo!(),
                        _ => {
                            debug!("UNKNOWN WINDOW EVENT RECEIVED: {:?}", event);
                        }
                    }
                }
            }
            Event::AboutToWait => {}
            Event::DeviceEvent {
                device_id: _,
                event,
            } => {
                if let winit::event::DeviceEvent::MouseMotion { delta } = event {
                    self.inputs
                        .update_cursor_delta((delta.0 as f32, delta.1 as f32));
                }
            }
            _ => {
                debug!("UNKNOWN EVENT RECEIVED: {:?}", event);
            }
        })?;
        Ok(())
    }

    fn redraw_requested<State, FUpdate, FRender>(
        &mut self,
        state: &mut State,
        update: FUpdate,
        render: FRender,
    ) where
        FUpdate: Fn(&mut State, &mut GameEngine),
        FRender: Fn(&State, &mut Renderer),
    {
        info!("rendering as per the RedrawRequested was received");

        self.last_frame_delta = self.timer.elapsed().as_secs_f32();
        self.camera_controler
            .update_camera(&mut self.camera, self.last_frame_delta, &self.inputs);
        self.renderer
            .renderer_context
            .set_camera_matrix(&self.renderer.context, &self.camera.calc_matrix());
        warn!("camera: {:?}", self.camera);
        self.timer = Instant::now();

        self.egui_integration.prepare_frame(&self.window);
        update(state, self);
        render(state, &mut self.renderer);

        match self.surface.get_current_texture() {
            Ok(output) => {
                self.renderer.render(&output.texture);
                self.egui_integration.render(
                    &self.renderer.context.device,
                    &self.renderer.context.queue,
                    &output,
                );

                output.present();
            }
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.on_resize(self.size);
            }
            Err(err) => {
                panic!(
                    "Can't get current swapchain texture due to an error: {}",
                    err
                );
            }
        }

        self.window.request_redraw();
    }

    fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
        info!("on resize event received new_size: {:?}", new_size);
        self.surface_configuration.width = new_size.width;
        self.surface_configuration.height = new_size.height;
        self.surface
            .configure(&self.renderer.context.device, &self.surface_configuration);
        self.renderer.on_resize(size_to_vec2(&new_size));
    }

    fn on_scale_factor_change(&mut self, scale_factor: f64) {
        info!("on scale factor change scale_factor: {}", scale_factor);
        self.renderer.on_scale_factor_change(scale_factor);
    }

    pub fn egui(&self) -> &egui::Context {
        self.egui_integration.egui_context()
    }
}
