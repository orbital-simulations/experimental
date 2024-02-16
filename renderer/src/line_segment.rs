use std::rc::Rc;

use glam::{Vec2, Vec3};
use wgpu::{
    include_wgsl, vertex_attr_array, BufferAddress, RenderPass, VertexBufferLayout, VertexStepMode,
};

use crate::{
    buffers::{DescriptiveBuffer, IndexBuffer, WriteableBuffer},
    context::{Context, RenderingContext},
    pipeline::{CreatePipeline, Pipeline},
    raw::Gpu,
    render_pass::RenderTargetDescription,
};

#[derive(Debug)]
#[repr(C, packed)]
pub struct LineSegment {
    pub from: Vec2,
    pub to: Vec2,
    pub color: Vec3,
    pub width: f32,
}

impl LineSegment {
    pub fn new(from: Vec2, to: Vec2, color: Vec3, width: f32) -> Self {
        Self {
            from,
            to,
            color,
            width,
        }
    }
}

// SAFETY: This is fine because we make sure the corresponding Attribute
// definitions are defined correctly.
unsafe impl Gpu for LineSegment {}

const LINE_SEGMENT_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 4] =
    vertex_attr_array![1 => Float32x2, 2 => Float32x2, 3 => Float32x3, 4 => Float32];

const LINE_SEGMENT_VERTICES: [Vec2; 4] = [
    Vec2 { x: -1.0, y: -1.0 },
    Vec2 { x: 1.0, y: -1.0 },
    Vec2 { x: -1.0, y: 1.0 },
    Vec2 { x: 1.0, y: 1.0 },
];

const LINE_SEGMENT_INDICES: &[u16] = &[0, 1, 3, 3, 2, 0];

impl DescriptiveBuffer for LineSegment {
    fn describe_vertex_buffer(step_mode: VertexStepMode) -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<LineSegment>() as BufferAddress,
            step_mode,
            attributes: &LINE_SEGMENT_VERTEX_ATTRIBUTES,
        }
    }
}

pub struct LineSegmentRenderer {
    line_segments: Vec<LineSegment>,
    pipeline: Pipeline,
    vertex_buffer: WriteableBuffer<Vec2>,
    index_buffer: IndexBuffer<u16>,
    instance_buffer: WriteableBuffer<LineSegment>,
}

impl LineSegmentRenderer {
    pub fn new(
        context: &Context,
        rendering_context: &RenderingContext,
        render_target_description: &RenderTargetDescription,
    ) -> Self {
        let shader = Rc::new(
            context
                .device
                .create_shader_module(include_wgsl!("../shaders/line_segment.wgsl")),
        );
        let index_buffer = IndexBuffer::new(context, "circle index buffer", LINE_SEGMENT_INDICES);
        let vertex_buffer = WriteableBuffer::new(
            context,
            "circle vertex buffer",
            &LINE_SEGMENT_VERTICES,
            wgpu::BufferUsages::VERTEX,
        );

        let instance_buffer = WriteableBuffer::new(
            context,
            "circle instance buffer",
            &[],
            wgpu::BufferUsages::VERTEX,
        );

        let pipeline_create_parameters = CreatePipeline {
            shader,
            vertex_buffer_layouts: &[
                Vec2::describe_vertex_buffer(VertexStepMode::Vertex),
                LineSegment::describe_vertex_buffer(VertexStepMode::Instance),
            ],
            bind_group_layouts: &[rendering_context.camera().bind_group_layout()],
            name: "custom mash renderer".to_string(),
        };
        let pipeline = Pipeline::new(
            context,
            &pipeline_create_parameters,
            render_target_description,
        );

        Self {
            line_segments: vec![],
            vertex_buffer,
            index_buffer,
            instance_buffer,
            pipeline,
        }
    }

    pub fn add_line_segment(&mut self, line_segment: LineSegment) {
        self.line_segments.push(line_segment);
    }

    pub fn render<'a>(
        &'a mut self,
        context: &Context,
        rendering_context: &'a RenderingContext,
        render_pass: &mut RenderPass<'a>,
    ) {
        if !self.line_segments.is_empty() {
            self.instance_buffer
                .write_data(context, &self.line_segments);

            render_pass.set_pipeline(self.pipeline.render_pipeline());
            rendering_context.camera().bind(render_pass, 0);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            self.index_buffer.set_index_buffer(render_pass);
            render_pass.draw_indexed(
                0..self.index_buffer.draw_count(),
                0,
                0..(self.line_segments.len() as u32),
            );

            // TODO: Think about some memory releasing strategy. Spike in number of
            // circles will lead to space leak.
            self.line_segments.clear();
        }
    }
}
