use glam::Mat4;
use wgpu::{
    util::DeviceExt, BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPass, TextureFormat,
};

use crate::{
    buffers::simple_vertex_uniform_layout_entry,
    projection::{Projection, ProjectionManipulation},
    raw::Raw,
};

pub struct Context {
    pub device: wgpu::Device,
    // Sends data and encoded commands to GPU
    pub queue: wgpu::Queue,
    pub output_texture_format: TextureFormat,
}

impl Context {
    pub fn new(device: Device, queue: Queue, texture_format: TextureFormat) -> Self {
        Self {
            device,
            queue,
            output_texture_format: texture_format,
        }
    }
}

pub struct RenderingContext {
    // Contains projection and camra layous.
    pub common_bind_group_layout: BindGroupLayout,
    pub common_bind_group: BindGroup,
    pub projection_matrix_buffer: Buffer,
    pub camera_matrix_buffer: Buffer,
}

impl RenderingContext {
    pub fn new(context: &Context, projection: &Projection) -> Self {
        let common_bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Common bind group layout"),
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
        }
    }

    pub fn bind<'a>(&'a self, render_pass: &mut RenderPass<'a>, slot: u32) {
        render_pass.set_bind_group(slot, &self.common_bind_group, &[]);
    }

    pub fn set_projection_matrix(&self, context: &Context, projection_matrix: &Mat4) {
        context.queue.write_buffer(
            &self.projection_matrix_buffer,
            0,
            projection_matrix.get_raw(),
        );
    }

    pub fn set_camera_matrix(&self, context: &Context, projection_matrix: &Mat4) {
        context
            .queue
            .write_buffer(&self.camera_matrix_buffer, 0, projection_matrix.get_raw());
    }
}
