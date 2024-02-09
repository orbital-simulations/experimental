use std::slice::from_ref;

use glam::{Mat4, Vec2};
use wgpu::{ShaderStages, BindGroupLayoutEntry};
use wgpu::{BindGroupLayout, RenderPass};
use crate::buffers::{DescriptiveBindGroupEntry, BindGroup};

use crate::{
    context::Context,
    projection::{Projection, ProjectionManipulation},
    buffers::WriteableBuffer,
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
        let projection_matrix_buffer: WriteableBuffer<Mat4> = WriteableBuffer::new(&context, "projectino matrix buffer", &[projection.make_projection_matrix()], wgpu::BufferUsages::UNIFORM);
        let camera_identity_matrix = glam::Mat4::IDENTITY;
        let camera_matrix_buffer: WriteableBuffer<Mat4> = WriteableBuffer::new(&context, "camera matrix buffer", &[camera_identity_matrix], wgpu::BufferUsages::UNIFORM);
        let bind_group = BindGroup::new(context, "camera", &[(0, ShaderStages::VERTEX, &projection_matrix_buffer), (1, ShaderStages::VERTEX, &camera_matrix_buffer)]);

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

    pub fn set_projection_matrix(&self, context: &Context) {
        self.projection_matrix_buffer.write_data(context, &[self.projection.make_projection_matrix()]);
    }

    pub fn set_camera_matrix(&self, context: &Context, camera_matrix: &Mat4) {
        self.projection_matrix_buffer.write_data(context, from_ref(camera_matrix));
    }

    fn bind_group_layout(&self) -> &BindGroupLayout {
        self.bind_group.layout()
    }
}

impl DescriptiveBindGroupEntry for Camera {
    fn bind_group_entry_description(
        &self,
        binding: u32,
        shader_stage: ShaderStages,
    ) -> BindGroupLayoutEntry {
        todo!()
    }
}
