use std::{any::Any, ops::Deref};

use glam::{Vec2, Vec3};
use wgpu::{include_wgsl, vertex_attr_array, RenderPass, VertexBufferLayout, VertexStepMode};

use crate::{
    buffers::{IndexBuffer, WriteableBuffer},
    context::{Context, RenderingContext},
    pipeline::{
        CreatePipeline, Pipeline, PipelineDescriptable, PipelineStore, RenderTargetDescription,
    },
    raw::Gpu,
    shader_store::{Shader, ShaderCreator, ShaderDescriptable, ShaderStore},
    store::EntryLabel,
};

#[derive(Debug)]
#[repr(C, packed)]
pub struct FilledCircle {
    pub pos: Vec2,
    pub radius: f32,
    pub color: Vec3,
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for FilledCircle {}

impl FilledCircle {
    pub fn new(pos: Vec2, radius: f32, color: Vec3) -> Self {
        Self { pos, radius, color }
    }
}

const CIRCLE_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] =
    vertex_attr_array![1 => Float32x2, 2 => Float32, 3 => Float32x3];

const CIRCLE_VERTICES: [Vec2; 4] = [
    Vec2 { x: -1.0, y: -1.0 },
    Vec2 { x: 1.0, y: -1.0 },
    Vec2 { x: -1.0, y: 1.0 },
    Vec2 { x: 1.0, y: 1.0 },
];

const CIRCLE_INDICES: &[u16] = &[0, 1, 3, 3, 2, 0];

pub struct FilledCircleRenderer {
    circles: Vec<FilledCircle>,
    vertex_buffer: WriteableBuffer<Vec2>,
    index_buffer: IndexBuffer<u16>,
    instance_buffer: WriteableBuffer<FilledCircle>,
    pipeline: Option<Pipeline>,
    shader: Shader,
}

const BUFFER_LAYOUTS: [VertexBufferLayout<'static>; 2] = [
    VertexBufferLayout {
        array_stride: std::mem::size_of::<Vec2>() as u64,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_array![0 => Float32x2],
    },
    VertexBufferLayout {
        array_stride: std::mem::size_of::<FilledCircle>() as u64,
        step_mode: VertexStepMode::Instance,
        attributes: &CIRCLE_VERTEX_ATTRIBUTES,
    },
];

impl PipelineDescriptable for FilledCircleRenderer {
    fn pipeline_description(&self, rendering_context: &RenderingContext) -> CreatePipeline {
        CreatePipeline {
            shader: self.shader.clone(),
            vertex_buffer_layouts: BUFFER_LAYOUTS.to_vec(),
            bind_group_layouts: vec![rendering_context.camera().bind_group_layout()],
            name: "filled circle renderer".to_string(),
        }
    }
}

struct FilledCircleShaderLabel;

impl EntryLabel for FilledCircleShaderLabel {
    fn unique_label(&self) -> (std::any::TypeId, u64) {
        (self.type_id(), 0)
    }
}

impl ShaderDescriptable for FilledCircleShaderLabel {
    fn shader_description(&self) -> ShaderCreator {
        ShaderCreator::ShaderStatic(include_wgsl!("../shaders/filled_circle.wgsl"))
    }
}

impl EntryLabel for FilledCircleRenderer {
    fn unique_label(&self) -> (std::any::TypeId, u64) {
        (self.type_id(), 0)
    }
}

impl FilledCircleRenderer {
    pub fn new(context: &Context, shader_store: &mut ShaderStore) -> Self {
        let shader = shader_store.get_entry(&FilledCircleShaderLabel);

        let index_buffer = IndexBuffer::new(context, "circle index buffer", CIRCLE_INDICES);
        let vertex_buffer = WriteableBuffer::new(
            context,
            "circle vertex buffer",
            &CIRCLE_VERTICES,
            wgpu::BufferUsages::VERTEX,
        );

        let instance_buffer = WriteableBuffer::new(
            context,
            "circle instance buffer",
            &[],
            wgpu::BufferUsages::VERTEX,
        );

        Self {
            circles: vec![],
            vertex_buffer,
            index_buffer,
            instance_buffer,
            shader,
            pipeline: None,
        }
    }

    pub fn add_circle(&mut self, circle: FilledCircle) {
        self.circles.push(circle);
    }

    pub fn render<'a>(
        &'a mut self,
        context: &Context,
        rendering_context: &'a RenderingContext,
        render_pass: &mut RenderPass<'a>,
        render_target_description: &RenderTargetDescription,
        pipeline_store: &mut PipelineStore,
    ) {
        if !self.circles.is_empty() {
            self.instance_buffer.write_data(context, &self.circles);

            if self.pipeline.is_none() {
                self.pipeline = Some(pipeline_store.get_entry(self, render_target_description));
            }

            let pipeline = self
                .pipeline
                .as_ref()
                .expect("pipeline should be created by now");

            render_pass.set_pipeline(&pipeline.deref().pipeline);
            rendering_context.camera().bind(render_pass, 0);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            self.index_buffer.set_index_buffer(render_pass);
            render_pass.draw_indexed(
                0..self.index_buffer.draw_count(),
                0,
                0..(self.circles.len() as u32),
            );

            // TODO: Think about some memory releasing strategy. Spike in number of
            // circles will lead to space leak.
            self.circles.clear();
        }
    }
}
