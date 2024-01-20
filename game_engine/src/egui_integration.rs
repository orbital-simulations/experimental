use egui_winit::EventResponse;
use wgpu::{CommandEncoderDescriptor, Device, Queue, StoreOp, SurfaceTexture, TextureFormat};
use winit::{dpi::PhysicalSize, window::Window};

pub struct EguiIntegration {
    egui_winit_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    egui_screen_descriptor: egui_wgpu::renderer::ScreenDescriptor,
}

impl EguiIntegration {
    pub fn new(window: &Window, device: &Device, surface_format: TextureFormat) -> Self {
        let egui_context = egui::Context::default();
        let viewport_id = egui_context.viewport_id();
        let scale_factor = window.scale_factor() as f32;
        let egui_winit_state =
            egui_winit::State::new(egui_context, viewport_id, &window, Some(scale_factor), None);

        let egui_renderer = egui_wgpu::Renderer::new(device, surface_format, None, 1);
        let size = window.inner_size();
        let egui_screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
            size_in_pixels: [size.width, size.height],
            pixels_per_point: scale_factor,
        };

        Self {
            egui_winit_state,
            egui_renderer,
            egui_screen_descriptor,
        }
    }

    pub fn on_window_event(
        &mut self,
        window: &Window,
        event: &winit::event::WindowEvent,
    ) -> EventResponse {
        self.egui_winit_state.on_window_event(window, event)
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f32) {
        self.egui_screen_descriptor.pixels_per_point = scale_factor;
    }

    pub fn on_resize(&mut self, new_size: PhysicalSize<u32>) {
        self.egui_screen_descriptor.size_in_pixels = [new_size.width, new_size.height];
    }

    pub fn prepare_frame(&mut self, window: &Window) {
        let raw_input = self.egui_winit_state.take_egui_input(window);
        self.egui_winit_state.egui_ctx().begin_frame(raw_input);
    }

    pub fn render(&mut self, device: &Device, queue: &Queue, output: &SurfaceTexture) {
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Egui encoder"),
        });

        let egui_context = self.egui_winit_state.egui_ctx();

        let egui::FullOutput {
            shapes,
            textures_delta,
            ..
        } = egui_context.end_frame();

        let paint_jobs =
            egui_context.tessellate(shapes, self.egui_screen_descriptor.pixels_per_point);

        for id in textures_delta.free {
            self.egui_renderer.free_texture(&id);
        }

        for (id, image_delta) in textures_delta.set {
            self.egui_renderer
                .update_texture(device, queue, id, &image_delta);
        }

        let texture_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.egui_renderer.update_buffers(
            device,
            queue,
            &mut encoder,
            &paint_jobs,
            &self.egui_screen_descriptor,
        );

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Egui renderer pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // The egui rendering needs to go as the last step so
                        // we draw gui over everything else. And thus we CAN
                        // NOT clear the surface, so we use Load instead.
                        load: wgpu::LoadOp::Load,
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.egui_renderer
                .render(&mut render_pass, &paint_jobs, &self.egui_screen_descriptor);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn egui_context(&self) -> &egui::Context {
        self.egui_winit_state.egui_ctx()
    }
}
