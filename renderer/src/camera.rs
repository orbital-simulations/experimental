use std::slice::from_ref;

use crate::buffers::{BindGroup, DescriptiveBindGroupEntry};
use glam::{Mat4, Vec2};
use wgpu::{BindGroupLayout, BufferUsages, RenderPass};
use wgpu::{BindGroupLayoutEntry, ShaderStages};

use crate::{
    buffers::WriteableBuffer,
    context::Context,
    projection::{Projection, ProjectionManipulation},
};

pub struct Camera {
    // Contains projection and camra layous.
    bind_group: BindGroup,
    projection_matrix_buffer: WriteableBuffer<Mat4>,
    camera_matrix_buffer: WriteableBuffer<Mat4>,
    projection: Projection,
}

impl Camera {
    pub fn new(context: &Context, projection: Projection) -> Self {
        let projection_matrix_buffer: WriteableBuffer<Mat4> = WriteableBuffer::new(
            context,
            "projectino matrix buffer",
            &[projection.make_projection_matrix()],
            BufferUsages::UNIFORM,
        );
        let camera_identity_matrix = glam::Mat4::IDENTITY;
        let camera_matrix_buffer: WriteableBuffer<Mat4> = WriteableBuffer::new(
            context,
            "camera matrix buffer",
            &[camera_identity_matrix],
            BufferUsages::UNIFORM,
        );
        let bind_group = BindGroup::new(
            context,
            "camera",
            &[
                (0, ShaderStages::VERTEX, &projection_matrix_buffer),
                (1, ShaderStages::VERTEX, &camera_matrix_buffer),
            ],
        );

        Self {
            bind_group,
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
        self.bind_group.bind(render_pass, slot);
    }

    pub fn set_projection_matrix(&mut self, context: &Context) {
        self.projection_matrix_buffer
            .write_data(context, &[self.projection.make_projection_matrix()]);
    }

    pub fn set_camera_matrix(&mut self, context: &Context, camera_matrix: &Mat4) {
        self.camera_matrix_buffer
            .write_data(context, from_ref(camera_matrix));
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        self.bind_group.layout()
    }
}

impl DescriptiveBindGroupEntry for Camera {
    fn bind_group_entry_description(
        &self,
        _binding: u32,
        _shader_stage: ShaderStages,
    ) -> BindGroupLayoutEntry {
        todo!()
    }
}
