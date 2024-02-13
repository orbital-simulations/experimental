use std::rc::Rc;

use wgpu::{
    BindGroupLayout, PipelineLayout, RenderPipeline, ShaderModule, VertexBufferLayout,
    ColorTargetState,
};

use crate::{
    context::Context, render_pass::RenderTarget,
};

pub struct CreatePipeline<'a> {
    pub shader: Rc<ShaderModule>,
    pub vertex_buffer_layouts: &'a [VertexBufferLayout<'static>],
    pub bind_group_layouts: &'a [&'a BindGroupLayout],
    pub name: String,
}

#[derive(Debug)]
pub struct Pipeline {
    pipeline_layout: PipelineLayout,
    name: String,
    pipeline: RenderPipeline,
    shader: Rc<ShaderModule>,
}

impl Pipeline {
    pub fn new(
        context: &Context,
        parameters: &CreatePipeline,
        render_target: &RenderTarget,
    ) -> Self {
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

        let targets:Vec<Option<ColorTargetState>> = render_target.targets().iter().map(|(target_texture, _, _)|{
                Some(wgpu::ColorTargetState {
                        format: target_texture.format(),
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })
        }).collect();
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
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: render_target.multisampling(),
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
            shader: parameters.shader.clone(),
            pipeline_layout,
        }
    }

    pub fn render_pipeline(&self) -> &RenderPipeline{
        &self.pipeline
    }
}
