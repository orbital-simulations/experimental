use std::{num::NonZeroU32, path::PathBuf};

#[derive(Clone, Debug)]
pub enum Shader {
    CompiledIn(String, String),
    Path(PathBuf),
}

//#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct FragmentState {
    /// The color state of the render targets.
    pub targets: Vec<Option<wgpu::ColorTargetState>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct VertexBufferLayout {
    /// The stride, in bytes, between elements of this buffer.
    pub array_stride: wgpu::BufferAddress,
    /// How often this vertex buffer is "stepped" forward.
    pub step_mode: wgpu::VertexStepMode,
    /// The list of attributes which comprise a single vertex.
    pub attributes: Vec<wgpu::VertexAttribute>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct VertexState {
    /// The format of any vertex buffers used with this pipeline.
    pub buffers: Vec<VertexBufferLayout>,
}

//#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
#[derive(Clone, Debug, Default)]
pub struct BindGroupLayoutDescription {
    /// Debug label of the bind group layout. This will show up in graphics debuggers for easy identification.
    pub label: String,

    /// Array of entries in this BindGroupLayout
    pub entries: Vec<wgpu::BindGroupLayoutEntry>,
}

//#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
#[derive(Clone, Debug, Default)]
pub struct PipelineLayoutDescription {
    /// Bind groups that this pipeline uses. The first entry will provide all the bindings for
    /// "set = 0", second entry will provide all the bindings for "set = 1" etc.
    pub bind_group_layouts: Vec<BindGroupLayoutDescription>,
    /// Set of push constant ranges this pipeline uses. Each shader stage that uses push constants
    /// must define the range in push constant memory that corresponds to its single `layout(push_constant)`
    /// uniform block.
    ///
    /// If this array is non-empty, the [`Features::PUSH_CONSTANTS`] must be enabled.
    pub push_constant_ranges: Vec<wgpu::PushConstantRange>,
}

//#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[derive(Clone, Debug)]
pub struct RenderPipelineDescription {
    pub shader: Shader,
    /// Debug label of the pipeline. This will show up in graphics debuggers for easy identification.
    pub label: String,
    /// The layout of bind groups for this pipeline.
    pub layout: Option<PipelineLayoutDescription>,
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
