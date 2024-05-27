use super::store_base::{StoreBase, StoreEntityId};
use crate::gpu_context::GpuContext;

pub type BindGroupLayoutId = StoreEntityId<wgpu::BindGroupLayout>;

pub struct BindGroupLayoutStore {
    store: StoreBase<wgpu::BindGroupLayout>,
    gpu_context: GpuContext,
}

impl BindGroupLayoutStore {
    pub fn new(gpu_context: &GpuContext) -> Self {
        Self {
            store: StoreBase::new(),
            gpu_context: gpu_context.clone(),
        }
    }

    pub fn build_bind_group_layout(
        &mut self,
        bind_group_layout_descriptor: &wgpu::BindGroupLayoutDescriptor,
    ) -> BindGroupLayoutId {
        let bind_group_layout = self
            .gpu_context
            .device()
            .create_bind_group_layout(bind_group_layout_descriptor);
        self.store.add(bind_group_layout)
    }

    pub fn get_bing_group_layout(
        &self,
        bind_group_id: &BindGroupLayoutId,
    ) -> &wgpu::BindGroupLayout {
        self.store.get(bind_group_id)
    }
}
