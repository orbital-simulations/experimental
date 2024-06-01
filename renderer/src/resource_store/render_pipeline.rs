use std::num::NonZeroU32;

use slotmap::{new_key_type, SecondaryMap, SlotMap};

use super::pipeline_layout::{PipelineLayoutId, PipelineLayoutStore};
use super::reload_command::RebuildCommand;
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
    pipeline_descriptors: SecondaryMap<PipelineId, RenderPipelineDescriptor>,
    gpu_context: GpuContext,
}

impl RenderPipelineStore {
    pub fn new(gpu_context: &GpuContext) -> Self {
        Self {
            store: SlotMap::with_key(),
            gpu_context: gpu_context.clone(),
            pipeline_descriptors: SecondaryMap::new(),
        }
    }

    pub fn build_render_pipeline(
        &mut self,
        render_pipeline_descriptor: &RenderPipelineDescriptor,
        shader_store: &mut ShaderStore,
        pipeline_layout_store: &PipelineLayoutStore,
    ) -> PipelineId {
        let vertex_shader_id = render_pipeline_descriptor.vertex.module;
        let fragmen_shader_id = render_pipeline_descriptor
            .fragment
            .as_ref()
            .map(|v| v.module);

        let render_pipeline = self.build(
            render_pipeline_descriptor,
            shader_store,
            pipeline_layout_store,
        );
        let id = self.store.insert(render_pipeline);
        self.pipeline_descriptors
            .insert(id, render_pipeline_descriptor.clone());

        shader_store.register_dependant(vertex_shader_id, RebuildCommand::Pipeline(id));
        if let Some(fragmen_shader_id) = fragmen_shader_id {
            shader_store.register_dependant(fragmen_shader_id, RebuildCommand::Pipeline(id));
        }
        id
    }

    fn build(
        &self,
        render_pipeline_descriptor: &RenderPipelineDescriptor,
        shader_store: &ShaderStore,
        pipeline_layout_store: &PipelineLayoutStore,
    ) -> wgpu::RenderPipeline {
        let buffers = render_pipeline_descriptor
            .vertex
            .buffers
            .iter()
            .map(|v| wgpu::VertexBufferLayout {
                array_stride: v.array_stride,
                step_mode: v.step_mode,
                attributes: &v.attributes,
            })
            .collect::<Vec<wgpu::VertexBufferLayout>>();
        let vertex_shader_id = render_pipeline_descriptor.vertex.module;
        let vertex = wgpu::VertexState {
            module: shader_store.get_shader(vertex_shader_id),
            buffers: buffers.as_slice(),
            entry_point: "vs_main",
        };
        let fragment = render_pipeline_descriptor
            .fragment
            .as_ref()
            .map(|v| wgpu::FragmentState {
                module: shader_store.get_shader(v.module),
                entry_point: "fs_main",
                targets: &v.targets,
            });
        let bind_group_layout_descriptor = wgpu::RenderPipelineDescriptor {
            label: Some(&render_pipeline_descriptor.label),
            layout: render_pipeline_descriptor
                .layout
                .as_ref()
                .map(|v| pipeline_layout_store.get_pipeline_layout(*v)),
            vertex,
            primitive: render_pipeline_descriptor.primitive,
            depth_stencil: render_pipeline_descriptor.depth_stencil.clone(),
            multisample: render_pipeline_descriptor.multisample,
            fragment,
            multiview: render_pipeline_descriptor.multiview,
        };
        self.gpu_context
            .device()
            .create_render_pipeline(&bind_group_layout_descriptor)
    }

    pub fn get_render_pipeline(&self, pipeline_id: PipelineId) -> &wgpu::RenderPipeline {
        &self.store[pipeline_id]
    }

    pub fn rebuild(
        &mut self,
        shader_store: &ShaderStore,
        pipeline_layout_store: &PipelineLayoutStore,
        pipeline_id: PipelineId,
    ) {
        let render_pipeline_descriptor = &self.pipeline_descriptors[pipeline_id];
        let render_pipeline = self.build(
            render_pipeline_descriptor,
            shader_store,
            pipeline_layout_store,
        );
        self.store[pipeline_id] = render_pipeline;
    }
}
