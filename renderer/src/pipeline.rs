use std::{num::NonZeroU32, rc::Rc, sync::Arc};

use wgpu::{
    BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BufferAddress,
    ColorTargetState, DepthStencilState, FragmentState, MultisampleState, PipelineLayout,
    PipelineLayoutDescriptor, PrimitiveState, PushConstantRange, RenderPipeline, TextureFormat,
    VertexAttribute, VertexStepMode,
};

use crate::{
    context::Context,
    shader_store::{ShaderID, ShaderStore},
    store::{FatStoreID, UnlockedStore, StorableResource, Store, StoreID},
};

#[derive(Debug, Clone)]
pub struct RenderTargetDescription {
    pub multisampling: u32,
    pub depth_texture: Option<TextureFormat>,
    pub targets: Vec<TextureFormat>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BindGroupLayoutDescription {
    pub label: String,
    pub entries: Vec<BindGroupLayoutEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PipelineLayoutDescription {
    pub label: String,
    pub bind_group_layouts: Vec<BindGroupLayoutDescription>,
    pub push_constant_ranges: Vec<PushConstantRange>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexBufferLayoutDescriptor {
    /// The stride, in bytes, between elements of this buffer.
    pub array_stride: BufferAddress,
    /// How often this vertex buffer is "stepped" forward.
    pub step_mode: VertexStepMode,
    /// The list of attributes which comprise a single vertex.
    pub attributes: Vec<VertexAttribute>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexStateDescription {
    /// The compiled shader module for this stage.
    pub module: ShaderID,
    /// The format of any vertex buffers used with this pipeline.
    pub buffers: Vec<VertexBufferLayoutDescriptor>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FragmentStateDescription {
    /// The compiled shader module for this stage.
    pub module: ShaderID,
    /// The color state of the render targets.
    pub targets: Vec<Option<ColorTargetState>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PipelineDescription {
    pub label: String,
    /// The layout of bind groups for this pipeline.
    pub layout: Option<PipelineLayoutDescription>,
    /// The compiled vertex stage, its entry point, and the input buffers layout.
    pub vertex: VertexStateDescription,
    /// The properties of the pipeline at the primitive assembly and rasterization level.
    pub primitive: PrimitiveState,
    /// The effect of draw calls on the depth and stencil aspects of the output target, if any.
    pub depth_stencil: Option<DepthStencilState>,
    /// The multi-sampling properties of the pipeline.
    pub multisample: MultisampleState,
    /// The compiled fragment stage, its entry point, and the color targets.
    pub fragment: Option<FragmentStateDescription>,
    /// If the pipeline will be used with a multiview render pass, this indicates how many array
    /// layers the attachments will have.
    pub multiview: Option<NonZeroU32>,
}

pub struct BindGroupLayoutContext {
    pub gpu_context: Arc<Context>,
}

pub type BindGroupLayoutStore = Store<BindGroupLayout>;
pub type BindGroupLayoutID = StoreID<BindGroupLayout>;

impl StorableResource for BindGroupLayout {
    type Context = BindGroupLayoutContext;
    type Description = BindGroupLayoutDescription;

    fn build(
        context: &BindGroupLayoutContext,
        description: &BindGroupLayoutDescription,
    ) -> BindGroupLayout {
        context
            .gpu_context
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some(&description.label),
                entries: &description.entries,
            })
    }

    fn register_dependences(
        _context: &Self::Context,
        _description: &Self::Description,
        _fat_id: FatStoreID<Self>,
    ) {
        // BindGroupLayouts don't have any dependences
    }
}

pub struct PipelineContext {
    pub gpu_context: Arc<Context>,
    pub shader_store: ShaderStore,
    pub bind_group_layout_store: BindGroupLayoutStore,
}

pub type PipelineStore = Store<RenderPipeline>;
pub type UnlockedPipelineStore<'a> = UnlockedStore<'a, RenderPipeline>;
pub type PipelineID = StoreID<RenderPipeline>;

impl StorableResource for RenderPipeline {
    type Context = PipelineContext;
    type Description = PipelineDescription;

    fn build(context: &PipelineContext, description: &PipelineDescription) -> RenderPipeline {
        let mut bind_group_layout_store = context.bind_group_layout_store.lock();

        let layout: Option<PipelineLayout> =
            description.layout.as_ref().map(|layout_description| {
                let mut bind_group_layout_ids: Vec<BindGroupLayoutID> = vec![];
                // FIXME: Is it possible to merge the two fors making the
                // `bind_group_layout_refs` into one????
                for bind_group_layout_description in layout_description.bind_group_layouts.iter() {
                    let id = bind_group_layout_store.get_or_create(bind_group_layout_description);
                    bind_group_layout_ids.push(id);
                }
                let bind_group_layout_refs: Vec<&BindGroupLayout> = bind_group_layout_ids
                    .iter()
                    .map(|id| bind_group_layout_store.get_ref(id))
                    .collect();
                context
                    .gpu_context
                    .device
                    .create_pipeline_layout(&PipelineLayoutDescriptor {
                        label: Some(&layout_description.label),
                        bind_group_layouts: &bind_group_layout_refs,
                        push_constant_ranges: &layout_description.push_constant_ranges,
                    })
            });
        let shader_store = context.shader_store.lock();

        let fragment: Option<FragmentState> =
            description
                .fragment
                .as_ref()
                .map(|fragment_description| FragmentState {
                    module: shader_store.get_ref(&fragment_description.module),
                    entry_point: "fs_main",
                    targets: &fragment_description.targets,
                });

        let bind_group_layout_refs: Vec<wgpu::VertexBufferLayout> = description
            .vertex
            .buffers
            .iter()
            .map(|bind_group_layout_description| wgpu::VertexBufferLayout {
                array_stride: bind_group_layout_description.array_stride,
                step_mode: bind_group_layout_description.step_mode,
                attributes: &bind_group_layout_description.attributes,
            })
            .collect();
        let vertex: wgpu::VertexState = {
            wgpu::VertexState {
                module: shader_store.get_ref(&description.vertex.module),
                entry_point: "vs_main",
                buffers: bind_group_layout_refs.as_slice(),
            }
        };

        context
            .gpu_context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&description.label),
                layout: layout.as_ref(),
                vertex,
                fragment,
                primitive: description.primitive,
                depth_stencil: description.depth_stencil.clone(),
                multisample: description.multisample,
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: description.multiview,
            })
    }

    fn register_dependences(
        context: &Self::Context,
        description: &Self::Description,
        fat_id: FatStoreID<Self>,
    ) {
        let mut guarded_shader_store = context.shader_store.lock();
        guarded_shader_store.add_dependant(Rc::new(fat_id.clone()), &description.vertex.module);
    }
}
