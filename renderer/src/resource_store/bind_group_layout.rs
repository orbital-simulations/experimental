use slotmap::{new_key_type, SlotMap};

use crate::gpu_context::GpuContext;

new_key_type! {
    pub struct BindGroupLayoutId;
}

pub struct BindGroupLayoutStore {
    store: SlotMap<BindGroupLayoutId, wgpu::BindGroupLayout>,
    gpu_context: GpuContext,
}

impl BindGroupLayoutStore {
    pub fn new(gpu_context: &GpuContext) -> Self {
        Self {
            store: SlotMap::with_key(),
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
        self.store.insert(bind_group_layout)
    }

    pub fn get_bing_group_layout(
        &self,
        bind_group_id: BindGroupLayoutId,
    ) -> &wgpu::BindGroupLayout {
        // SAFETY: This works fine because we don't remove element and when we start removing them
        // it will be done in a way that doesn't leave keys (ids) dangling.
        unsafe {
            self.store.get_unchecked(bind_group_id)
        }
    }
}
