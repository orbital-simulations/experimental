use wgpu::{
    BindGroupLayout, ColorTargetState, CompareFunction, DepthBiasState, DepthStencilState,
    RenderPipeline, ShaderModule, StencilState, TextureFormat, VertexBufferLayout,
};

use crate::context::{Context, RenderingContext};

#[derive(Debug, Clone)]
pub struct RenderTargetDescription {
    pub multisampling: u32,
    pub depth_texture: Option<TextureFormat>,
    pub targets: Vec<TextureFormat>,
}

pub struct CreatePipeline<'a> {
    pub shader: &'a ShaderModule,
    pub vertex_buffer_layouts: Vec<VertexBufferLayout<'static>>,
    pub bind_group_layouts: Vec<&'a BindGroupLayout>,
    pub name: String,
}

#[derive(Debug)]
pub struct Pipeline {
    #[allow(dead_code)]
    name: String,
    pipeline: RenderPipeline,
}

pub trait PipelineCreator {
    fn create_pipeline<'a>(&'a self, rendering_context: &'a RenderingContext)
        -> CreatePipeline<'a>;
}

impl Pipeline {
    pub fn new(
        context: &Context,
        pipeline_creator: &impl PipelineCreator,
        render_target_description: &RenderTargetDescription,
        rendering_context: &RenderingContext,
    ) -> Self {
        let parameters = pipeline_creator.create_pipeline(rendering_context);
        let mut pipeline_layout_descriptor_name = parameters.name.clone();
        pipeline_layout_descriptor_name.push_str("layout descriptor");

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(pipeline_layout_descriptor_name.as_str()),
                    bind_group_layouts: &parameters.bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let targets: Vec<Option<ColorTargetState>> = render_target_description
            .targets
            .iter()
            .map(|target_texture_format| {
                Some(wgpu::ColorTargetState {
                    format: *target_texture_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })
            })
            .collect();

        let depth_stencil =
            render_target_description
                .depth_texture
                .map(|format| DepthStencilState {
                    format,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                });
        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&parameters.name),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: parameters.shader,
                    entry_point: "vs_main",
                    buffers: &parameters.vertex_buffer_layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: parameters.shader,
                    entry_point: "fs_main",
                    targets: &targets,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: render_target_description.multisampling,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });
        Self {
            name: parameters.name.to_string(),
            pipeline,
        }
    }

    pub fn render_pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }
}
