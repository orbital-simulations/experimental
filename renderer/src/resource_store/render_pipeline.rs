use std::num::NonZeroU32;

use slotmap::{new_key_type, SlotMap};

use super::pipeline_layout::{PipelineLayoutId, PipelineLayoutStore};
use super::shader::{ShaderId, ShaderStore};
use crate::gpu_context::GpuContext;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct VertexBufferLayout {
    /// The stride, in bytes, between elements of this buffer.
    pub array_stride: wgpu::BufferAddress,
    /// How often this vertex buffer is "stepped" forward.
    pub step_mode: wgpu::VertexStepMode,
    /// The list of attributes which comprise a single vertex.
    pub attributes: Vec<wgpu::VertexAttribute>,
}

#[derive(Clone, Debug)]
pub struct VertexState {
    /// The compiled shader module for this stage.
    pub module: ShaderId,
    /// The format of any vertex buffers used with this pipeline.
    pub buffers: Vec<VertexBufferLayout>,
}

#[derive(Clone, Debug)]
pub struct FragmentState {
    /// The compiled shader module for this stage.
    pub module: ShaderId,
    /// The color state of the render targets.
    pub targets: Vec<Option<wgpu::ColorTargetState>>,
}

#[derive(Clone, Debug)]
pub struct RenderPipelineDescriptor {
    pub label: String,
    /// The layout of bind groups for this pipeline.
    pub layout: Option<PipelineLayoutId>,
    /// The compiled vertex stage, its entry point, and the input buffers layout.
    pub vertex: VertexState,
    /// The properties of the pipeline at the primitive assembly and rasterization level.
    pub primitive: wgpu::PrimitiveState,
    /// The effect of draw calls on the depth and stencil aspects of the output target, if any.
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    /// The multi-sampling properties of the pipeline.
    pub multisample: wgpu::MultisampleState,
    /// The compiled fragment stage, its entry point, and the color targets.
    pub fragment: Option<FragmentState>,
    /// If the pipeline will be used with a multiview render pass, this indicates how many array
    /// layers the attachments will have.
    pub multiview: Option<NonZeroU32>,
}

new_key_type! {
    pub struct PipelineId;
}

pub struct RenderPipelineStore {
    store: SlotMap<PipelineId, wgpu::RenderPipeline>,
    gpu_context: GpuContext,
}

impl RenderPipelineStore {
    pub fn new(gpu_context: &GpuContext) -> Self {
        Self {
            store: SlotMap::with_key(),
            gpu_context: gpu_context.clone(),
        }
    }

    pub fn build_render_pipeline(
        &mut self,
        bind_group_layout_descriptor: &RenderPipelineDescriptor,
        shader_store: &ShaderStore,
        pipeline_layout_store: &PipelineLayoutStore,
    ) -> PipelineId {
        let buffers = bind_group_layout_descriptor
            .vertex
            .buffers
            .iter()
            .map(|v| wgpu::VertexBufferLayout {
                array_stride: v.array_stride,
                step_mode: v.step_mode,
                attributes: &v.attributes,
            })
            .collect::<Vec<wgpu::VertexBufferLayout>>();
        let vertex = wgpu::VertexState {
            module: shader_store.get_shader(bind_group_layout_descriptor.vertex.module),
            buffers: buffers.as_slice(),
            entry_point: "vs_main",
        };
        let fragment =
            bind_group_layout_descriptor
                .fragment
                .as_ref()
                .map(|v| wgpu::FragmentState {
                    module: shader_store.get_shader(v.module),
                    entry_point: "fs_main",
                    targets: &v.targets,
                });
        let bind_group_layout_descriptor = wgpu::RenderPipelineDescriptor {
            label: Some(&bind_group_layout_descriptor.label),
            layout: bind_group_layout_descriptor
                .layout
                .as_ref()
                .map(|v| pipeline_layout_store.get_pipeline_layout(*v)),
            vertex,
            primitive: bind_group_layout_descriptor.primitive,
            depth_stencil: bind_group_layout_descriptor.depth_stencil.clone(),
            multisample: bind_group_layout_descriptor.multisample,
            fragment,
            multiview: bind_group_layout_descriptor.multiview,
        };
        let bind_group_layout = self
            .gpu_context
            .device()
            .create_render_pipeline(&bind_group_layout_descriptor);
        self.store.insert(bind_group_layout)
    }

    pub fn get_render_pipeline(&self, pipeline_id: PipelineId) -> &wgpu::RenderPipeline {
        // SAFETY: This works fine because we don't remove element and when we start removing them
        // it will be done in a way that doesn't leave keys (ids) dangling.
        unsafe {
            self.store.get_unchecked(pipeline_id)
        }
    }
}
