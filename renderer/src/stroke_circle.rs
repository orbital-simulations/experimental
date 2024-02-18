use glam::{Vec2, Vec3};
use wgpu::{
    include_wgsl, vertex_attr_array, RenderPass, ShaderModule, VertexBufferLayout, VertexStepMode
};

use crate::{
    buffers::{IndexBuffer, WriteableBuffer}, context::{Context, RenderingContext}, pipeline::{CreatePipeline, Pipeline, PipelineCreator, RenderTargetDescription}, raw::Gpu
};

#[derive(Debug)]
#[repr(C, packed)]
pub struct StrokeCircle {
    pub pos: Vec2,
    pub radius: f32,
    pub border: f32,
    pub color: Vec3,
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for StrokeCircle {}

impl StrokeCircle {
    pub fn new(pos: Vec2, radius: f32, border: f32, color: Vec3) -> Self {
        Self {
            pos,
            radius,
            border,
            color,
        }
    }
}

const STROKE_CIRCLE_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 4] =
    vertex_attr_array![1 => Float32x2, 2 => Float32, 3 => Float32, 4 => Float32x3];

const STROKE_CIRCLE_VERTICES: [Vec2; 4] = [
    Vec2 { x: -1.0, y: -1.0 },
    Vec2 { x: 1.0, y: -1.0 },
    Vec2 { x: -1.0, y: 1.0 },
    Vec2 { x: 1.0, y: 1.0 },
];

const STROKE_CIRCLE_INDICES: &[u16] = &[0, 1, 3, 3, 2, 0];

#[derive(Debug)]
pub struct StrokeCircleRenderer {
    circles: Vec<StrokeCircle>,
    vertex_buffer: WriteableBuffer<Vec2>,
    index_buffer: IndexBuffer<u16>,
    instance_buffer: WriteableBuffer<StrokeCircle>,
    pipeline: Option<Pipeline>,
    shader: ShaderModule,
}

impl PipelineCreator for StrokeCircleRenderer {
    fn create_pipeline<'a>(&'a self, rendering_context: &'a RenderingContext) -> CreatePipeline<'a> {
        CreatePipeline {
            shader: &self.shader,
            vertex_buffer_layouts: vec![
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vec2>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &vertex_attr_array![0 => Float32x2],
                },
                VertexBufferLayout {
                    array_stride: std::mem::size_of::<StrokeCircle>() as u64,
                    step_mode: VertexStepMode::Instance,
                    attributes: &STROKE_CIRCLE_VERTEX_ATTRIBUTES,
                }
            ],
            bind_group_layouts: vec![rendering_context.camera().bind_group_layout()],
            name: "filled circle renderer".to_string(),
        }
    }
}

impl StrokeCircleRenderer {
    pub fn new(
        context: &Context,
    ) -> Self {
        let shader =
            context
                .device
                .create_shader_module(include_wgsl!("../shaders/stroke_circle.wgsl"));
        let index_buffer = IndexBuffer::new(context, "circle index buffer", STROKE_CIRCLE_INDICES);
        let vertex_buffer = WriteableBuffer::new(
            context,
            "circle vertex buffer",
            &STROKE_CIRCLE_VERTICES,
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
            pipeline: None,
            shader,
        }
    }

    pub fn add_stroke_circle(&mut self, circle: StrokeCircle) {
        self.circles.push(circle);
    }

    pub fn render<'a>(
        &'a mut self,
        context: &Context,
        rendering_context: &'a RenderingContext,
        render_pass: &mut RenderPass<'a>,
        render_target_description: &RenderTargetDescription
    ) {
        if !self.circles.is_empty() {
            self.instance_buffer.write_data(context, &self.circles);

            if self.pipeline.is_none() {
            let pipeline = Pipeline::new(
                    context,
                    self,
                    render_target_description,
                    rendering_context,
                );

            self.pipeline = Some(pipeline);
        }

        let pipeline = &self.pipeline.as_ref().expect("pipeline should be created by now");


            render_pass.set_pipeline(pipeline.render_pipeline());
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
