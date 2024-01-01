pub mod buffers;
pub mod colors;
pub mod filled_circle;
pub mod filled_rectangle;
pub mod raw;
pub mod windowed_device;

use std::time::Instant;

use filled_circle::{FilledCircle, FilledCircleRenderer};
use filled_rectangle::{FilledRectangle, FilledRectangleRenderer};
use raw::Raw;
use tracing::{debug, info};
use wgpu::util::DeviceExt;
use windowed_device::WindowedDevice;
use winit::event::WindowEvent::{
    CloseRequested, KeyboardInput, MouseInput, RedrawRequested, Resized,
};
use winit::keyboard::NamedKey;
use winit::{dpi::PhysicalSize, event::Event, event_loop::EventLoop};

pub struct Renderer {
    windowed_device: WindowedDevice,
    #[allow(unused)]
    projection_bind_group: wgpu::BindGroup,
    projection_buffer: wgpu::Buffer,
    #[allow(unused)]
    projection_bind_group_layout: wgpu::BindGroupLayout,

    full_circle_renderer: FilledCircleRenderer,
    full_rectangle_renderer: FilledRectangleRenderer,
    pub last_frame_delta: f32,

    timer: Instant,
}

impl Renderer {
    pub async fn new() -> (Self, EventLoop<()>) {
        let mut event_loop = EventLoop::new().expect("can't create the event loop");
        let mut windowed_device = WindowedDevice::new(&mut event_loop).await;
        let (projection_buffer, projection_bind_group_layout, projection_bind_group) =
            Self::create_projection(&mut windowed_device);

        let full_circle_renderer =
            FilledCircleRenderer::new(&mut windowed_device, &projection_bind_group_layout);
        let full_rectangle_renderer =
            FilledRectangleRenderer::new(&mut windowed_device, &projection_bind_group_layout);

        (
            Self {
                windowed_device,
                projection_bind_group,
                projection_buffer,
                projection_bind_group_layout,
                full_circle_renderer,
                last_frame_delta: 0.,
                full_rectangle_renderer,
                timer: Instant::now(),
            },
            event_loop,
        )
    }

    /// Return a orthographics projection matrix which will place the (0,0) into the left top
    /// corner.
    fn create_projection(
        windowed_device: &mut WindowedDevice,
    ) -> (wgpu::Buffer, wgpu::BindGroupLayout, wgpu::BindGroup) {
        let size = windowed_device.window.inner_size();
        let perspective_matrix = Self::generate_projection_matrix(size);

        let projection_buffer =
            windowed_device
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Projection Buffer"),
                    contents: perspective_matrix.get_raw(),
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

    fn generate_projection_matrix(size: PhysicalSize<u32>) -> glam::Mat4 {
        let half_width = (size.width / 2) as f32;
        let half_height = (size.height / 2) as f32;
        glam::Mat4::orthographic_lh(
            -half_width,
            half_width,
            -half_height,
            half_height,
            0.0,
            -1.0,
        )
    }

    fn update_projection(&mut self, new_size: PhysicalSize<u32>) {
        self.windowed_device.queue.write_buffer(
            &self.projection_buffer,
            0,
            Self::generate_projection_matrix(new_size).get_raw(),
        );
    }

    pub fn run<State, FSetup, FUpdate>(
        &mut self,
        event_loop: EventLoop<()>,
        setup: FSetup,
        update: &FUpdate,
    ) where
        FSetup: FnOnce() -> State,
        FUpdate: Fn(&mut State, &mut Renderer),
    {
        let mut state = setup();
        // Restart timer just in case.
        self.timer = Instant::now();
        event_loop
            .run(move |event, elwt| {
                if let Event::WindowEvent { event, .. } = event {
                    match event {
                        Resized(new_size) => {
                            info!("updating the projection matric after resize");
                            self.windowed_device.config.width = new_size.width;
                            self.windowed_device.config.height = new_size.height;
                            self.windowed_device.surface.configure(
                                &self.windowed_device.device,
                                &self.windowed_device.config,
                            );
                            self.update_projection(new_size);
                            self.windowed_device.window.request_redraw();
                        }
                        CloseRequested => elwt.exit(),
                        KeyboardInput {
                            device_id: _,
                            event,
                            is_synthetic: _,
                        } => {
                            info!("Escape was pressed; terminating the event loop");
                            if let winit::keyboard::Key::Named(NamedKey::Escape) = event.logical_key
                            {
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
                    }
                } else {
                    debug!("UNKNOWN EVENT RECEIVED: {:?}", event);
                }
            })
            .unwrap();
    }

    fn redraw_requested<State, FUpdate>(&mut self, state: &mut State, update: FUpdate)
    where
        FUpdate: Fn(&mut State, &mut Renderer),
    {
        info!("rendering as per the RedrawRequested was received");
        // TODO: It is possible this may cause some time shuttering in the
        // first frame. Maybe the first delta should be 0??
        self.last_frame_delta = self.timer.elapsed().as_secs_f32();
        self.timer = Instant::now();
        update(state, self);

        let (mut encoder, view, output) = self
            .windowed_device
            .prepare_encoder()
            .expect("retreiving GPU command encoder");

        self.full_circle_renderer.render(
            &mut self.windowed_device,
            &self.projection_bind_group,
            &view,
            &mut encoder,
        );

        self.full_rectangle_renderer.render(
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

    pub fn draw_full_circle(&mut self, full_circle: FilledCircle) {
        self.full_circle_renderer.add_circle(full_circle);
    }
    pub fn draw_full_rectangle(&mut self, full_rectangle: FilledRectangle) {
        self.full_rectangle_renderer.add_rectangle(full_rectangle);
    }
}
