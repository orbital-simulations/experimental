use glam::{Mat4, Vec2};
use wgpu::{util::DeviceExt, BindGroup, BindGroupLayout, Buffer, RenderPass};

use crate::{
    buffers::simple_vertex_uniform_layout_entry,
    context::Context,
    projection::{Projection, ProjectionManipulation},
    raw::Raw,
};

pub struct Camera {
    // Contains projection and camra layous.
    common_bind_group_layout: BindGroupLayout,
    common_bind_group: BindGroup,
    projection_matrix_buffer: Buffer,
    camera_matrix_buffer: Buffer,
    projection: Projection,
}

impl Camera {
    pub fn new(context: &Context, projection: Projection) -> Self {
        let common_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Camera bind group layout"),
                    entries: &[
                        simple_vertex_uniform_layout_entry(0),
                        simple_vertex_uniform_layout_entry(1),
                    ],
                });
        let projection_matrix_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Projection buffer"),
                    contents: projection.make_projection_matrix().get_raw(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        let camera_matrix_buffer =
            context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Camera buffer"),
                    contents: Mat4::IDENTITY.transpose().get_raw(),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });
        let common_bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &common_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: projection_matrix_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: camera_matrix_buffer.as_entire_binding(),
                    },
                ],
                label: Some("Projection bind group"),
            });

        Self {
            common_bind_group_layout,
            common_bind_group,
            projection_matrix_buffer,
            camera_matrix_buffer,
            projection,
        }
    }
    pub fn on_resize(&mut self, new_size: Vec2, context: &Context) {
        self.projection.resize(new_size.x, new_size.y);
        self.set_projection_matrix(context);
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f64, context: &Context) {
        self.projection.scale(scale_factor as f32);
        self.set_projection_matrix(context);
    }

    pub fn bind<'a>(&'a self, render_pass: &mut RenderPass<'a>, slot: u32) {
        render_pass.set_bind_group(slot, &self.common_bind_group, &[]);
    }

    pub fn set_projection_matrix(&self, context: &Context) {
        context.queue.write_buffer(
            &self.projection_matrix_buffer,
            0,
            &self.projection.make_projection_matrix().get_raw(),
        );
    }

    pub fn set_camera_matrix(&self, context: &Context, projection_matrix: &Mat4) {
        context
            .queue
            .write_buffer(&self.camera_matrix_buffer, 0, projection_matrix.get_raw());
    }
}
