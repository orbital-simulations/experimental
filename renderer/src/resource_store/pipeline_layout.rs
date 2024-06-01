use slotmap::{new_key_type, SlotMap};

use super::bind_group_layout::{BindGroupLayoutId, BindGroupLayoutStore};
use crate::gpu_context::GpuContext;

#[derive(Clone, Debug, Default)]
pub struct PipelineLayoutDescriptor {
    /// Debug label of the pipeline layout. This will show up in graphics debuggers for easy identification.
    pub label: String,
    /// Bind groups that this pipeline uses. The first entry will provide all the bindings for
    /// "set = 0", second entry will provide all the bindings for "set = 1" etc.
    pub bind_group_layouts: Vec<BindGroupLayoutId>,
    /// Set of push constant ranges this pipeline uses. Each shader stage that uses push constants
    /// must define the range in push constant memory that corresponds to its single `layout(push_constant)`
    /// uniform block.
    ///
    /// If this array is non-empty, the [`Features::PUSH_CONSTANTS`] must be enabled.
    pub push_constant_ranges: Vec<wgpu::PushConstantRange>,
}

pub struct PipelineLayoutStore {
    store: SlotMap<PipelineLayoutId, wgpu::PipelineLayout>,
    gpu_context: GpuContext,
}

new_key_type! {
    pub struct PipelineLayoutId;
}

impl PipelineLayoutStore {
    pub fn new(gpu_context: &GpuContext) -> Self {
        Self {
            store: SlotMap::with_key(),
            gpu_context: gpu_context.clone(),
        }
    }

    pub fn build_pipeline_layout(
        &mut self,
        bind_group_layout_descriptor: &PipelineLayoutDescriptor,
        bind_group_layout_store: &BindGroupLayoutStore,
    ) -> PipelineLayoutId {
        let descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some(&bind_group_layout_descriptor.label),
            bind_group_layouts: &bind_group_layout_descriptor
                .bind_group_layouts
                .iter()
                .map(|v| bind_group_layout_store.get_bing_group_layout(*v))
                .collect::<Vec<&wgpu::BindGroupLayout>>(),
            push_constant_ranges: &bind_group_layout_descriptor.push_constant_ranges,
        };
        let pipeline_layout = self
            .gpu_context
            .device()
            .create_pipeline_layout(&descriptor);
        self.store.insert(pipeline_layout)
    }

    pub fn get_pipeline_layout(
        &self,
        pipeline_layout_id: PipelineLayoutId,
    ) -> &wgpu::PipelineLayout {
        &self.store[pipeline_layout_id]
    }
}
