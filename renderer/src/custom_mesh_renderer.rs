use std::any::Any;

use glam::Vec3;
use wgpu::{vertex_attr_array, RenderPass, VertexBufferLayout, VertexStepMode};

use crate::{
    context::{Context, RenderingContext},
    mesh::GpuMesh,
    pipeline::{CreatePipeline, Pipeline, PipelineDescriptable, PipelineStore, RenderTargetDescription},
    shader_store::{Shader, ShaderDescriptable, ShaderStore},
};

pub struct CustomMeshRenderer {
    shader: Shader,
    pipeline: Option<Pipeline>,
    mesh: GpuMesh,
}

impl PipelineDescriptable for CustomMeshRenderer {
    fn pipeline_description<'a>(&'a self, rendering_context: &'a RenderingContext) -> CreatePipeline<'a> {
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
    pub fn new<T>(
        mesh: GpuMesh,
        context: &Context,
        shader_store: &mut ShaderStore,
        shader_label: &T,
    ) -> Self
    where
        T: ShaderDescriptable + Any,
    {
        let shader = shader_store.get_shader(context, shader_label);
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
        pipeline_store: &mut PipelineStore,
    ) {
        if self.pipeline.is_none() {
            self.pipeline = Some(pipeline_store.get_pipeline(context, self, render_target_description, rendering_context));
        }

        let pipeline = &self
            .pipeline
            .as_ref()
            .expect("pipeline should be created by now");

        render_pass.set_pipeline(pipeline);
        rendering_context.camera().bind(render_pass, 0);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.mesh.normal_buffer.slice(..));
        self.mesh.index_buffer.set_index_buffer(render_pass);
        render_pass.draw_indexed(0..self.mesh.index_buffer.draw_count(), 0, 0..1);
    }
}
