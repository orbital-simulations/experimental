use glam::Vec3;
use wgpu::{vertex_attr_array, RenderPass, RenderPipeline, VertexStepMode};

use crate::{
    context::{Context, RenderingContext},
    mesh::GpuMesh,
    pipeline::{Pipeline, RenderTargetDescription},
    resource_watcher::ResourceWatcher,
    web_gpu::{
        FragmentState, PipelineLayoutDescription, RenderPipelineDescription, Shader,
        VertexBufferLayout, VertexState,
    },
};
use std::cell::Ref;

#[derive(Debug)]
pub struct CustomMeshRenderer {
    pipeline: Option<Pipeline>,
    mesh: GpuMesh,
    shader_description: Shader,
}

impl CustomMeshRenderer {
    pub fn new(mesh: GpuMesh, shader_description: Shader) -> Self {
        Self {
            shader_description,
            pipeline: None,
            mesh,
        }
    }

    pub fn build<'a>(
        &'a mut self,
        rendering_context: &'a RenderingContext,
        context: &Context,
        render_target_description: &RenderTargetDescription,
        resource_watcher: &mut ResourceWatcher,
    ) {
        if self.pipeline.is_none() {
            let depth_stencil =
                render_target_description
                    .depth_texture
                    .map(|format| wgpu::DepthStencilState {
                        format,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    });

            let targets: Vec<Option<wgpu::ColorTargetState>> = render_target_description
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
            let render_pipeline_description = RenderPipelineDescription {
                shader: self.shader_description.clone(),
                label: "filled rectangle renderer".to_string(),
                layout: Some(PipelineLayoutDescription {
                    bind_group_layouts: vec![rendering_context.camera().bind_group_layout()],
                    push_constant_ranges: vec![],
                }),
                vertex: VertexState {
                    buffers: vec![
                        VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vec3>() as u64,
                            step_mode: VertexStepMode::Vertex,
                            attributes: vertex_attr_array![0 => Float32x3].into(),
                        },
                        VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vec3>() as u64,
                            step_mode: VertexStepMode::Vertex,
                            attributes: vertex_attr_array![1 => Float32x3].into(),
                        },
                    ],
                },
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
                fragment: Some(FragmentState { targets }),
                multiview: None,
            };
            let pipeline = Pipeline::new(context, &render_pipeline_description, resource_watcher);

            self.pipeline = Some(pipeline);
        }
    }

    pub fn pipeline(&self) -> Option<&RenderPipeline> {
        self.pipeline.as_ref().map(|p| p.render_pipeline())
    }

    pub fn render<'a>(
        &'a self,
        rendering_context: &'a RenderingContext,
        render_pass: &mut RenderPass<'a>,
    ) {
        render_pass.set_pipeline(self.pipeline().unwrap());
        rendering_context.camera().bind(render_pass, 0);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.mesh.normal_buffer.slice(..));
        self.mesh.index_buffer.set_index_buffer(render_pass);
        render_pass.draw_indexed(0..self.mesh.index_buffer.draw_count(), 0, 0..1);
    }
}
