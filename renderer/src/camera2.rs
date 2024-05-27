use glam::{Mat4, Vec2};
use wgpu::BufferUsages;
use wgpu::{BindGroupLayoutEntry, ShaderStages};

use crate::buffers2::WriteableBuffer;
use crate::gpu_context::GpuContext;
use crate::projection2::CameraProjection;
use crate::resource_store::bind_group_layout::BindGroupLayoutId;
use crate::resource_store::ResourceStore;

#[derive(Debug, Clone)]
pub struct PrimaryCamera {
    pub projection: CameraProjection,
    pub surface_format: wgpu::TextureFormat,
    pub size: Vec2,
}

pub struct Camera {
    projection_matrix_buffer: WriteableBuffer<Mat4>,
    camera_transform_buffer: WriteableBuffer<Mat4>,
    bing_group_layout_id: BindGroupLayoutId,
    bing_group: wgpu::BindGroup, // TODO: Make it into BindGrpuId
    projection: CameraProjection,
    gpu_context: GpuContext,
    size: Vec2,
    surface_format: wgpu::TextureFormat,
}

impl Camera {
    pub fn new(
        gpu_context: &GpuContext,
        resource_store: &mut ResourceStore,
        projection: CameraProjection,
        surface_format: wgpu::TextureFormat,
        size: Vec2,
    ) -> Self {
        let projection_matrix_buffer: WriteableBuffer<Mat4> = WriteableBuffer::new(
            gpu_context,
            "projectino matrix buffer",
            &projection.make_projection_matrix(size),
            BufferUsages::UNIFORM,
        );
        let camera_identity_matrix = glam::Mat4::IDENTITY;
        let camera_transform_buffer: WriteableBuffer<Mat4> = WriteableBuffer::new(
            gpu_context,
            "camera matrix buffer",
            &camera_identity_matrix,
            BufferUsages::UNIFORM,
        );

        let bind_group_layout_descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bind group"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        };

        let bing_group_layout_id =
            resource_store.build_bind_group_layout(&bind_group_layout_descriptor);
        let bing_group = gpu_context
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("camera bind group"),
                layout: resource_store.get_bing_group_layout(&bing_group_layout_id),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: projection_matrix_buffer.buffer().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: camera_transform_buffer.buffer().as_entire_binding(),
                    },
                ],
            });

        Self {
            projection_matrix_buffer,
            camera_transform_buffer,
            projection,
            bing_group_layout_id,
            bing_group,
            gpu_context: gpu_context.clone(),
            size,
            surface_format,
        }
    }

    pub fn on_resize(&mut self, new_size: Vec2) {
        self.size = new_size;
        self.projection_matrix_buffer.write_data(
            &self.gpu_context,
            &self.projection.make_projection_matrix(new_size),
        );
    }

    pub fn on_scale_factor_change(&mut self, scale_factor: f32) {
        self.projection.set_scale(scale_factor);
        self.projection_matrix_buffer.write_data(
            &self.gpu_context,
            &self.projection.make_projection_matrix(self.size),
        );
    }

    pub fn set_camera_matrix(&mut self, matrix: &Mat4) {
        self.camera_transform_buffer
            .write_data(&self.gpu_context, matrix);
    }

    pub fn set_camera_projection(&mut self, projection: &CameraProjection) {
        self.projection = projection.clone();
        self.projection_matrix_buffer.write_data(
            &self.gpu_context,
            &self.projection.make_projection_matrix(self.size),
        );
    }

    pub fn bing_group(&self) -> &wgpu::BindGroup {
        &self.bing_group
    }

    pub fn bing_group_layout(&self) -> &BindGroupLayoutId {
        &self.bing_group_layout_id
    }

    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }
}
