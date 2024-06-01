use glam::{Mat4, Vec2};
use wgpu::{BindGroupLayoutEntry, ShaderStages};
use wgpu::{BufferUsages, DepthStencilState};

use crate::buffers::WriteableBuffer;
use crate::gpu_context::GpuContext;
use crate::projection::CameraProjection;
use crate::resource_store::bind_group_layout::BindGroupLayoutId;
use crate::resource_store::ResourceStore;

#[derive(Debug, Clone)]
pub struct PrimaryCamera {
    pub projection: CameraProjection,
    pub surface_format: wgpu::TextureFormat,
    pub size: Vec2,
    pub depth_buffer: Option<wgpu::ColorTargetState>,
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
    depth_texture: Option<(wgpu::ColorTargetState, wgpu::Texture, wgpu::TextureView)>,
}

impl Camera {
    pub fn new(
        gpu_context: &GpuContext,
        resource_store: &mut ResourceStore,
        projection: CameraProjection,
        surface_format: wgpu::TextureFormat,
        size: Vec2,
        depth_texture_config: Option<wgpu::ColorTargetState>,
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
                layout: resource_store.get_bing_group_layout(bing_group_layout_id),
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

        let depth_texture = depth_texture_config.map(|depth_texture_config| {
            let depth_texture =
                Self::build_depth_texture(gpu_context, &size, &depth_texture_config);
            let depth_texture_view =
                depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
            (
                depth_texture_config.clone(),
                depth_texture,
                depth_texture_view,
            )
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
            depth_texture,
        }
    }

    pub fn on_resize(&mut self, new_size: Vec2, gpu_context: &GpuContext) {
        self.size = new_size;
        self.projection_matrix_buffer.write_data(
            &self.gpu_context,
            &self.projection.make_projection_matrix(new_size),
        );
        self.depth_texture.iter_mut().for_each(
            |(depth_texture_config, depth_texture, depth_texture_view)| {
                *depth_texture =
                    Self::build_depth_texture(gpu_context, &new_size, depth_texture_config);
                *depth_texture_view =
                    depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
            },
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

    pub fn depth_buffer(
        &self,
    ) -> &Option<(wgpu::ColorTargetState, wgpu::Texture, wgpu::TextureView)> {
        &self.depth_texture
    }

    fn build_depth_texture(
        gpu_context: &GpuContext,
        size: &Vec2,
        depth_buffer_config: &wgpu::ColorTargetState,
    ) -> wgpu::Texture {
        let depth_texture_size = wgpu::Extent3d {
            width: size.x as u32,
            height: size.y as u32,
            depth_or_array_layers: 1,
        };
        let depth_texture_description = wgpu::TextureDescriptor {
            label: Some("camera depth texture"),
            size: depth_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: depth_buffer_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[depth_buffer_config.format],
        };
        gpu_context
            .device()
            .create_texture(&depth_texture_description)
    }

    pub fn depth_stencil(&self) -> Option<DepthStencilState> {
        self.depth_texture
            .as_ref()
            .map(|(depth_texture_config, _, _)| wgpu::DepthStencilState {
                format: depth_texture_config.format,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            })
    }
}
