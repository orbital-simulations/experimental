use std::rc::Rc;

use glam::Vec3;
use wgpu::{vertex_attr_array, RenderPass, ShaderModule, VertexBufferLayout, VertexStepMode};

use crate::{
    context::{Context, RenderingContext},
    mesh::GpuMesh,
    pipeline::{CreatePipeline, Pipeline, PipelineCreator, RenderTargetDescription},
};

#[derive(Debug)]
pub struct CustomMeshRenderer {
    shader: Rc<ShaderModule>,
    pipeline: Option<Pipeline>,
    mesh: GpuMesh,
}

impl PipelineCreator for CustomMeshRenderer {
    fn create_pipeline<'a>(
        &'a self,
        rendering_context: &'a RenderingContext,
    ) -> CreatePipeline<'a> {
        CreatePipeline {
            shader: &self.shader,
            vertex_buffer_layouts: vec![
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec3>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x3],
                },
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec3>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![1 => Float32x3],
                },
            ],
            bind_group_layouts: vec![rendering_context.camera().bind_group_layout()],
            name: "custom mesh renderer".to_string(),
        }
    }
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
        if self.pipeline.is_none() {
            let pipeline =
                Pipeline::new(context, self, render_target_description, rendering_context);

            self.pipeline = Some(pipeline);
        }

        let pipeline = &self
            .pipeline
            .as_ref()
            .expect("pipeline should be created by now");

        render_pass.set_pipeline(pipeline.render_pipeline());
        rendering_context.camera().bind(render_pass, 0);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.mesh.normal_buffer.slice(..));
        self.mesh.index_buffer.set_index_buffer(render_pass);
        render_pass.draw_indexed(0..self.mesh.index_buffer.draw_count(), 0, 0..1);
    }
}
