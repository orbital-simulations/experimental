use std::rc::Rc;

use wgpu::{RenderPass, ShaderModule};

use crate::{context::{Context, RenderingContext}, mesh::GpuMesh, pipeline::{Pipeline, CreatePipeline}, render_pass::RenderTarget};

#[derive(Debug)]
pub struct CustomMeshRenderer {
    pipeline: Pipeline,
    mesh: GpuMesh,
}

impl CustomMeshRenderer {
    pub fn new(
        context: &Context,
        mesh: GpuMesh,
        shader: Rc<ShaderModule>,
        render_target: &RenderTarget,
        rendering_context: &RenderingContext,
    ) -> Self {

        let pipeline_create_parameters = CreatePipeline {
            shader,
            vertex_buffer_layouts: &[mesh.vertex_buffer_layout.clone(), mesh.normal_buffer_layout.clone()],
            bind_group_layouts: &[rendering_context.camera().bind_group_layout()],
            name: "custom mash renderer".to_string(),
        };

        let pipeline = Pipeline::new(context, &pipeline_create_parameters, render_target);
        Self { pipeline, mesh }
    }

    pub fn render<'a>(
        &'a mut self,
        rendering_context: &'a RenderingContext,
        render_pass: &mut RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.pipeline.render_pipeline());
        rendering_context.camera().bind(render_pass, 0);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.mesh.normal_buffer.slice(..));
        self.mesh.index_buffer.set_index_buffer(render_pass);
        render_pass.draw_indexed(0..self.mesh.index_buffer.draw_count(), 0, 0..1);
    }
}
