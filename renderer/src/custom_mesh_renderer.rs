use wgpu::{BindGroup, BindGroupLayout, RenderPass, RenderPipeline, ShaderModule};

use crate::{context::Context, mesh::GpuMesh};

macro_rules! prefix_label {
    () => {
        "Custom mesh "
    };
}
#[derive(Debug)]
pub struct CustomMashRenderer {
    pipeline: RenderPipeline,
    mesh: GpuMesh,
}

impl CustomMashRenderer {
    pub fn new(
        context: &Context,
        common_bind_group_layout: &BindGroupLayout,
        mesh: GpuMesh,
        shader: ShaderModule,
    ) -> Self {
        let render_pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(concat!(prefix_label!(), "render pipeline layout")),
                    bind_group_layouts: &[common_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(concat!(prefix_label!(), "render pipeline")),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        mesh.vertex_buffer_layout.clone(),
                        mesh.normal_buffer_layout.clone(),
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
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
                    front_face: wgpu::FrontFace::Cw,
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

        Self { pipeline, mesh }
    }

    pub fn render<'b>(
        &'b mut self,
        common_bind_group: &'b BindGroup,
        render_pass: &mut RenderPass<'b>,
    ) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, common_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.mesh.normal_buffer.slice(..));
        render_pass.draw(0..self.mesh.vertex_count, 0..1);
    }
}
