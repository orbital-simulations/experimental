use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use wgpu::{include_wgsl, vertex_attr_array};

use crate::{
    buffers::{IndexBuffer, WriteableBuffer, WriteableVecBuffer},
    primitives::quad::{QUAD_2D_INDICES, QUAD_2D_VERICES},
    rendering_context::RenderingContext,
    resource_store::{
        pipeline_layout::PipelineLayoutDescriptor,
        render_pipeline::{
            FragmentState, RenderPipelineDescriptor, VertexBufferLayout, VertexState,
        },
        shader::ShaderSource,
        PipelineId,
    }, transform::{WorldTransform, WorldTransformGpuRepresentation},
};

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C, packed)]
pub struct Line {
    pub from: Vec3,
    pub to: Vec3,
    pub color: Vec3,
    pub width: f32,
}

impl Line {
    pub fn new(from: Vec3, to: Vec3, color: Vec3, width: f32) -> Self {
        Self {
            from,
            to,
            color,
            width,
        }
    }
}

pub struct LineRenderering {
    line_segments: Vec<Line>,
    line_segments_buffer: WriteableVecBuffer<Line>,
    line_segments_transforms: Vec<WorldTransformGpuRepresentation>,
    line_segments_transforms_buffer: WriteableVecBuffer<WorldTransformGpuRepresentation>,
    line_segment_pipeline: PipelineId,
    quad_vertex_buffer: WriteableBuffer<[Vec2; 4]>,
    quad_index_buffer: IndexBuffer<u16>,
}

impl LineRenderering {
    pub fn new(rendering_context: &mut RenderingContext) -> Self {
        let line_segments = Vec::new();
        let line_segments_buffer = WriteableVecBuffer::new(
            &rendering_context.gpu_context,
            "line segments buffer",
            &line_segments,
            wgpu::BufferUsages::VERTEX,
        );

        let line_segments_transforms = Vec::new();
        let line_segments_transforms_buffer = WriteableVecBuffer::new(
            &rendering_context.gpu_context,
            "line segments transforms buffer",
            &line_segments_transforms,
            wgpu::BufferUsages::VERTEX,
        );

        let line_segment_shader_id =
            rendering_context
                .resource_store
                .build_shader(&ShaderSource::StaticFile(include_wgsl!(
                    "../shaders/line_segment.wgsl"
                )));
        let quad_vertex_buffer = WriteableBuffer::new(
            &rendering_context.gpu_context,
            "quad index buffer",
            &QUAD_2D_VERICES,
            wgpu::BufferUsages::VERTEX,
        );
        let quad_index_buffer = IndexBuffer::new(
            &rendering_context.gpu_context,
            "quad index buffer",
            QUAD_2D_INDICES,
        );

        let targets: Vec<Option<wgpu::ColorTargetState>> = vec![Some(wgpu::ColorTargetState {
            format: rendering_context.primary_camera.surface_format(),
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent::REPLACE,
                alpha: wgpu::BlendComponent::REPLACE,
            }),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let line_segment_pipeline_layout_id = rendering_context
            .resource_store
            .build_pipeline_layout(&PipelineLayoutDescriptor {
                label: "line segment pipeline layout".to_string(),
                bind_group_layouts: vec![*rendering_context.primary_camera.bing_group_layout()],
                push_constant_ranges: Vec::new(),
            });

        let line_segment_pipeline =
            rendering_context
                .resource_store
                .build_render_pipeline(&RenderPipelineDescriptor {
                    label: "line segment pipeline".to_string(),
                    layout: Some(line_segment_pipeline_layout_id),
                    vertex: VertexState {
                        module: line_segment_shader_id,
                        buffers: vec![
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Vec2>() as u64,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: vertex_attr_array![0 => Float32x2].to_vec(),
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<WorldTransformGpuRepresentation>() as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: WorldTransformGpuRepresentation::vertex_attributes(1, 2, 3, 4)
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Line>() as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: vertex_attr_array![5 => Float32x3, 6 => Float32x3, 7 => Float32x3, 8 => Float32]
                                    .to_vec(),
                            },
                        ],
                    },
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: rendering_context.primary_camera.depth_stencil(),
                    multisample: wgpu::MultisampleState::default(),
                    fragment: Some(FragmentState {
                        module: line_segment_shader_id,
                        targets: targets.clone(),
                    }),
                    multiview: None,
                });

        Self {
            line_segments,
            line_segments_buffer,
            line_segment_pipeline,
            quad_vertex_buffer,
            quad_index_buffer,
            line_segments_transforms,
            line_segments_transforms_buffer,
        }
    }

    pub fn add_line_segment(&mut self, transform: &WorldTransform, line_segment: &Line) {
        self.line_segments.push(*line_segment);
        self.line_segments_transforms.push(transform.gpu());
    }

    pub fn render<'a>(
        &'a mut self,
        rendering_context: &'a RenderingContext,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if !self.line_segments.is_empty() {
            self.line_segments_buffer
                .write_data(&rendering_context.gpu_context, &self.line_segments);
            self.line_segments_transforms_buffer
                .write_data(&rendering_context.gpu_context, &self.line_segments_transforms);

            let pipeline = &rendering_context
                .resource_store
                .get_render_pipeline(self.line_segment_pipeline);

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, rendering_context.primary_camera.bing_group(), &[]);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.line_segments_transforms_buffer.slice(..));
            render_pass.set_vertex_buffer(2, self.line_segments_buffer.slice(..));
            render_pass.set_index_buffer(
                self.quad_index_buffer.slice(..),
                self.quad_index_buffer.index_format(),
            );
            render_pass.draw_indexed(
                self.quad_index_buffer.draw_count(),
                0,
                0..(self.line_segments.len() as u32),
            );

            // TODO: Think about some memory releasing strategy. Spike in number of
            // circles will lead to space leak.
            self.line_segments.clear();
            self.line_segments_transforms.clear();
        }
    }
}
