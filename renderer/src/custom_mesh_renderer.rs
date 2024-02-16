use std::rc::Rc;

use wgpu::{RenderPass, ShaderModule};

use crate::{
    context::{Context, RenderingContext},
    mesh::GpuMesh,
    pipeline::{CreatePipeline, Pipeline},
    render_pass::RenderTargetDescription,
};

#[derive(Debug)]
pub struct CustomMeshRenderer {
    shader: Rc<ShaderModule>,
    pipeline: Option<Pipeline>,
    mesh: GpuMesh,
}

impl CustomMeshRenderer {
    pub fn new(mesh: GpuMesh, shader: Rc<ShaderModule>) -> Self {
        Self {
            shader,
            pipeline: None,
            mesh,
        }
    }

    pub fn render<'a>(
        &'a mut self,
        rendering_context: &'a RenderingContext,
        context: &Context,
        render_pass: &mut RenderPass<'a>,
        render_target_description: &RenderTargetDescription,
    ) {
        let pipeline = self.pipeline.get_or_insert_with(|| {
            let pipeline_create_parameters = CreatePipeline {
                shader: &self.shader,
                vertex_buffer_layouts: &[
                    self.mesh.vertex_buffer_layout.clone(),
                    self.mesh.normal_buffer_layout.clone(),
                ],
                bind_group_layouts: &[rendering_context.camera().bind_group_layout()],
                name: "custom mesh renderer".to_string(),
            };

            Pipeline::new(
                context,
                &pipeline_create_parameters,
                render_target_description,
            )
        });

        render_pass.set_pipeline(pipeline.render_pipeline());
        rendering_context.camera().bind(render_pass, 0);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.mesh.normal_buffer.slice(..));
        self.mesh.index_buffer.set_index_buffer(render_pass);
        render_pass.draw_indexed(0..self.mesh.index_buffer.draw_count(), 0, 0..1);
    }
}
