use wgpu::{
    BindGroupLayout, PipelineLayout, RenderPipeline, ShaderModule, VertexBufferLayout,
    VertexStepMode,
};

use crate::{
    buffers::{BindGroup, DescriptiveBuffer},
    context::Context,
};

pub struct CreatePipeline<'a> {
    shader: ShaderModule,
    vertex_buffer_layouts: Vec<VertexBufferLayout<'static>>,
    bind_group_layouts: &'a [&'a BindGroupLayout],
    name: String,
}

struct Pipeline {
    pipeline_layout: PipelineLayout,
    name: String,
    pipeline: RenderPipeline,
    shader: ShaderModule,
}

impl Pipeline {
    pub fn new<'a>(
        context: &Context,
        parameters: &CreatePipeline<'a>,
    ) -> Self {
        let mut pipeline_layout_descriptor_name = parameters.name;
        pipeline_layout_descriptor_name.push_str("layout descriptor");

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(pipeline_layout_descriptor_name.as_str()),
                    bind_group_layouts: &parameters.bind_group_layouts,
                    push_constant_ranges: &[],
                });
        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&parameters.name),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &parameters.shader,
                    entry_point: "vs_main",
                    buffers: &parameters.vertex_buffer_layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &parameters.shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.output_texture_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
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
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });
        Self {
            name: parameters.name,
            pipeline,
            shader: parameters.shader,
            pipeline_layout,
        }
    }
}
