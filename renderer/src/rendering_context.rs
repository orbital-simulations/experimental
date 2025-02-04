use std::sync::Arc;

use crate::{
    camera::{Camera, PrimaryCamera},
    gpu_context::GpuContext,
    resource_store::ResourceStore,
};

pub struct RenderingContext {
    pub gpu_context: Arc<GpuContext>,
    pub primary_camera: Camera,
    pub resource_store: ResourceStore,
}

impl RenderingContext {
    pub fn new(gpu_context: &Arc<GpuContext>, primary_camera: PrimaryCamera) -> eyre::Result<Self> {
        let mut resource_store = ResourceStore::new(gpu_context)?;
        let primary_camera = Camera::new(
            gpu_context,
            &mut resource_store,
            primary_camera.projection,
            primary_camera.surface_format,
            primary_camera.size,
            primary_camera.depth_buffer,
        );
        Ok(Self {
            gpu_context: gpu_context.clone(),
            primary_camera,
            resource_store,
        })
    }

    pub fn wgpu_limits() -> wgpu::Limits {
        wgpu::Limits::default()
    }
}
