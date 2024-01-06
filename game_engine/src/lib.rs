pub mod buffers;
pub mod colors;
pub mod filled_circle;
pub mod filled_rectangle;
pub mod line_segment;
pub mod raw;
pub mod windowed_device;

use std::time::Instant;

use filled_circle::{FilledCircle, FilledCircleRenderer};
use filled_rectangle::{FilledRectangle, FilledRectangleRenderer};
use line_segment::{LineSegment, LineSegmentRenderer};
use raw::Raw;

use tracing::{debug, info};
use wgpu::util::DeviceExt;
use windowed_device::WindowedDevice;

use winit::event::WindowEvent::{
    CloseRequested, KeyboardInput, MouseInput, RedrawRequested, Resized, ScaleFactorChanged,
};
use winit::keyboard::NamedKey;

use winit::window::Window;
use winit::{dpi::PhysicalSize, event::Event, event_loop::EventLoop};

pub struct GameEngine<'a> {
    windowed_device: WindowedDevice<'a>,
    #[allow(unused)]
    projection_bind_group: wgpu::BindGroup,
    projection_buffer: wgpu::Buffer,
    #[allow(unused)]
    projection_bind_group_layout: wgpu::BindGroupLayout,

    filled_circle_renderer: FilledCircleRenderer,
    filled_rectangle_renderer: FilledRectangleRenderer,
    line_segment_renderer: LineSegmentRenderer,
    pub last_frame_delta: f32,

    timer: Instant,
    scale_factor: f64,
}

impl <'a> GameEngine<'a> {
    pub async fn new(event_loop: EventLoop<()>, window: &'a Window) -> eyre::Result<(Self, EventLoop<()>)> {
        let scale_factor = window.scale_factor();
        let mut windowed_device = WindowedDevice::new(window).await?;
        let (projection_buffer, projection_bind_group_layout, projection_bind_group) =
            Self::create_projection(&mut windowed_device, scale_factor);

        let filled_circle_renderer =
            FilledCircleRenderer::new(&mut windowed_device, &projection_bind_group_layout);
        let filled_rectangle_renderer =
            FilledRectangleRenderer::new(&mut windowed_device, &projection_bind_group_layout);
        let line_segment_renderer =
            LineSegmentRenderer::new(&mut windowed_device, &projection_bind_group_layout);

        Ok((
            Self {
                windowed_device,
                projection_bind_group,
                projection_buffer,
                projection_bind_group_layout,
                filled_circle_renderer,
                last_frame_delta: 0.,
                filled_rectangle_renderer,
                line_segment_renderer,
                timer: Instant::now(),
                scale_factor,
            },
            event_loop,
        ))
    }

    /// Return a orthographics projection matrix which will place the (0,0) into the left top
    /// corner.
    fn create_projection(
        windowed_device: &mut WindowedDevice,
        scale_factor: f64,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let size = windowed_device.window.inner_size();
        let projection_matrix = Self::generate_projection_matrix(size, scale_factor);

        let projection_buffer =
            windowed_device
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Projection Buffer"),
                    contents: projection_matrix.get_raw(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    // TODO: Check if the COPY_DST is needed.
                });

        let projection_bind_group_layout =
            windowed_device
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Projection Bind Group Descriptor"),
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });

        let projection_bind_group =
            windowed_device
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &projection_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: projection_buffer.as_entire_binding(),
                    }],
                    label: Some("Projection Bind Group"),
                });

        (
            projection_buffer,
            projection_bind_group_layout,
            projection_bind_group,
        )
    }

    fn generate_projection_matrix(size: PhysicalSize<u32>, scale_factor: f64) -> glam::Mat4 {
        let half_width = size.width as f32 / (2. * scale_factor as f32);
        let half_height = size.height as f32 / (2. * scale_factor as f32);
        glam::Mat4::orthographic_lh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            0.0,
            -1.0,
        )
    }

    fn update_projection(&mut self) {
        self.windowed_device.queue.write_buffer(
            &self.projection_buffer,
            0,
            Self::generate_projection_matrix(self.windowed_device.size, self.scale_factor)
                .get_raw(),
        );
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
                self.filled_circle_renderer.render(
                    &mut self.windowed_device,
                    &self.projection_bind_group,
                    &view,
                    &mut encoder,
                );

                self.filled_rectangle_renderer.render(
                    &mut self.windowed_device,
                    &self.projection_bind_group,
                    &view,
                    &mut encoder,
                );

                self.line_segment_renderer.render(
                    &mut self.windowed_device,
                    &self.projection_bind_group,
                    &view,
                    &mut encoder,
                );

                self.windowed_device
                    .queue
                    .submit(std::iter::once(encoder.finish()));
                output.present();
                self.windowed_device.window.request_redraw();
            }
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.on_resize(self.windowed_device.size);
            }
            Err(_) => {
                panic!("panicing in the panic");
            }
        }
    }

    pub fn draw_full_circle(&mut self, full_circle: FilledCircle) {
        self.filled_circle_renderer.add_circle(full_circle);
    }
    pub fn draw_full_rectangle(&mut self, full_rectangle: FilledRectangle) {
        self.filled_rectangle_renderer.add_rectangle(full_rectangle);
    }

    pub fn draw_line_segment(&mut self, line_segment: LineSegment) {
        self.line_segment_renderer.add_line_segment(line_segment);
    }

    pub fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
        info!("on resize event received new_size: {:?}", new_size);
        self.windowed_device.size = new_size;
        self.windowed_device.config.width = new_size.width;
        self.windowed_device.config.height = new_size.height;
        self.windowed_device
            .surface
            .configure(&self.windowed_device.device, &self.windowed_device.config);
        self.update_projection();
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f64) {
        info!("on scale factor change scale_factor: {}", scale_factor);
        self.scale_factor = scale_factor;
    }
}
