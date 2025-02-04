use crate::buffers::WriteableBuffer;
use crate::include_wgsl;
use crate::primitives::quad::{QUAD_2D_INDICES, QUAD_2D_VERICES};
use crate::resource_store::PipelineId;
use crate::transform::{Transform, TransformGpu};
use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use wgpu::vertex_attr_array;

use crate::resource_store::pipeline_layout::PipelineLayoutDescriptor;
use crate::resource_store::render_pipeline::{
    FragmentState, RenderPipelineDescriptor, VertexBufferLayout, VertexState,
};
use crate::{
    buffers::{IndexBuffer, WriteableVecBuffer},
    rendering_context::RenderingContext,
};

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C, packed)]
pub struct Rectangle {
    size: Vec2,
    color: Vec3,
}

#[derive(Debug, Copy, Clone, Zeroable, Pod)]
#[repr(C, packed)]
pub struct RectangleLine {
    size: Vec2,
    color: Vec3,
    border: f32,
}

impl Rectangle {
    pub fn new(size: Vec2, color: Vec3) -> Self {
        Self { size, color }
    }
}

impl RectangleLine {
    pub fn new(size: Vec2, color: Vec3, border: f32) -> Self {
        Self {
            size,
            color,
            border,
        }
    }
}

pub struct RectangleRendering {
    rectangles_buffer: WriteableVecBuffer<Rectangle>,
    rectangles: Vec<Rectangle>,
    rectangles_transforms: Vec<TransformGpu>,
    rectangles_transforms_buffer: WriteableVecBuffer<TransformGpu>,
    rectangle_lines_buffer: WriteableVecBuffer<RectangleLine>,
    rectangle_lines: Vec<RectangleLine>,
    rectangle_lines_transforms: Vec<TransformGpu>,
    rectangle_lines_transforms_buffer: WriteableVecBuffer<TransformGpu>,
    quad_vertex_buffer: WriteableBuffer<[Vec2; 4]>,
    quad_index_buffer: IndexBuffer<u16>,
    rectangles_pipeline: PipelineId,
    rectangle_lines_pipeline: PipelineId,
}

impl RectangleRendering {
    pub fn new(rendering_context: &mut RenderingContext) -> eyre::Result<Self> {
        let rectangles = Vec::new();
        let rectangles_buffer = WriteableVecBuffer::new(
            &rendering_context.gpu_context,
            "rectangles buffer",
            &rectangles,
            wgpu::BufferUsages::VERTEX,
        );
        let rectangle_lines = Vec::new();
        let rectangle_lines_buffer = WriteableVecBuffer::new(
            &rendering_context.gpu_context,
            "rectangle lines buffer",
            &rectangle_lines,
            wgpu::BufferUsages::VERTEX,
        );

        let rectangles_transforms = Vec::new();
        let rectangles_transforms_buffer = WriteableVecBuffer::new(
            &rendering_context.gpu_context,
            "rectangle transforms buffer",
            &rectangles_transforms,
            wgpu::BufferUsages::VERTEX,
        );
        let rectangle_lines_transforms = Vec::new();
        let rectangle_lines_transforms_buffer = WriteableVecBuffer::new(
            &rendering_context.gpu_context,
            "rectangle line transforms buffer",
            &rectangle_lines_transforms,
            wgpu::BufferUsages::VERTEX,
        );

        let rectangle_shader_id = rendering_context
            .resource_store
            .build_shader(&include_wgsl!("../shaders/rectangle.wgsl"))?;
        let rectangle_line_shader_id = rendering_context
            .resource_store
            .build_shader(&include_wgsl!("../shaders/rectangle_line.wgsl"))?;

        let quad_vertex_buffer = WriteableBuffer::new(
            &rendering_context.gpu_context,
            "quad vertex buffer",
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

        let rectangle_pipeline_layout_id =
            rendering_context
                .resource_store
                .build_pipeline_layout(&PipelineLayoutDescriptor {
                    label: "rectangle pipeline layout".to_string(),
                    bind_group_layouts: vec![*rendering_context.primary_camera.bing_group_layout()],
                    push_constant_ranges: Vec::new(),
                });

        let rectangles_pipeline =
            rendering_context
                .resource_store
                .build_render_pipeline(&RenderPipelineDescriptor {
                    label: "rectangle pipeline".to_string(),
                    layout: Some(rectangle_pipeline_layout_id),
                    vertex: VertexState {
                        module: rectangle_shader_id,
                        buffers: vec![
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Vec2>() as u64,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: vertex_attr_array![0 => Float32x2].to_vec(),
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<TransformGpu>() as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: TransformGpu::vertex_attributes(1, 2, 3, 4),
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Rectangle>() as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: vertex_attr_array![5 => Float32x2, 6 => Float32x3]
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
                        module: rectangle_shader_id,
                        targets: targets.clone(),
                    }),
                    multiview: None,
                });

        let rectangle_line_pipeline_layout_id = rendering_context
            .resource_store
            .build_pipeline_layout(&PipelineLayoutDescriptor {
                label: "rectangle line pipeline layout".to_string(),
                bind_group_layouts: vec![*rendering_context.primary_camera.bing_group_layout()],
                push_constant_ranges: Vec::new(),
            });

        let rectangle_lines_pipeline =
            rendering_context
                .resource_store
                .build_render_pipeline(&RenderPipelineDescriptor {
                    label: "rectangle line pipeline".to_string(),
                    layout: Some(rectangle_line_pipeline_layout_id),
                    vertex: VertexState {
                        module: rectangle_line_shader_id,
                        buffers: vec![
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Vec2>() as u64,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: vertex_attr_array![0 => Float32x2].to_vec(),
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<TransformGpu>() as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: TransformGpu::vertex_attributes(1, 2, 3, 4),
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<RectangleLine>() as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes:
                                    vertex_attr_array![5 => Float32x2, 6 => Float32x3, 7 => Float32]
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
                        module: rectangle_line_shader_id,
                        targets: targets.clone(),
                    }),
                    multiview: None,
                });

        Ok(Self {
            rectangles_buffer,
            rectangles,
            rectangle_lines_buffer,
            rectangle_lines,
            quad_vertex_buffer,
            quad_index_buffer,
            rectangles_pipeline,
            rectangle_lines_pipeline,
            rectangles_transforms,
            rectangle_lines_transforms,
            rectangles_transforms_buffer,
            rectangle_lines_transforms_buffer,
        })
    }

    pub fn add_rectangle(&mut self, transform: &Transform, rectangle: &Rectangle) {
        self.rectangles.push(*rectangle);
        self.rectangles_transforms.push(transform.into());
    }

    pub fn add_rectangle_line(&mut self, transform: &Transform, rectangle: &RectangleLine) {
        self.rectangle_lines.push(*rectangle);
        self.rectangle_lines_transforms.push(transform.into());
    }

    pub fn render<'a>(
        &'a mut self,
        rendering_context: &'a RenderingContext,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if !self.rectangles.is_empty() {
            self.rectangles_buffer
                .write_data(&rendering_context.gpu_context, &self.rectangles);
            self.rectangles_transforms_buffer
                .write_data(&rendering_context.gpu_context, &self.rectangles_transforms);

            let pipeline = &rendering_context
                .resource_store
                .get_render_pipeline(self.rectangles_pipeline);

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, rendering_context.primary_camera.bing_group(), &[]);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.rectangles_transforms_buffer.slice(..));
            render_pass.set_vertex_buffer(2, self.rectangles_buffer.slice(..));
            render_pass.set_index_buffer(
                self.quad_index_buffer.slice(..),
                self.quad_index_buffer.index_format(),
            );
            render_pass.draw_indexed(
                self.quad_index_buffer.draw_count(),
                0,
                0..(self.rectangles.len() as u32),
            );

            // TODO: Think about some memory releasing strategy. Spike in number of
            // rectangles will lead to space leak.
            self.rectangles.clear();
            self.rectangles_transforms.clear();
        }

        if !self.rectangle_lines.is_empty() {
            self.rectangle_lines_buffer
                .write_data(&rendering_context.gpu_context, &self.rectangle_lines);
            self.rectangle_lines_transforms_buffer.write_data(
                &rendering_context.gpu_context,
                &self.rectangle_lines_transforms,
            );

            let pipeline = &rendering_context
                .resource_store
                .get_render_pipeline(self.rectangle_lines_pipeline);

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, rendering_context.primary_camera.bing_group(), &[]);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.rectangle_lines_transforms_buffer.slice(..));
            render_pass.set_vertex_buffer(2, self.rectangle_lines_buffer.slice(..));
            render_pass.set_index_buffer(
                self.quad_index_buffer.slice(..),
                self.quad_index_buffer.index_format(),
            );
            render_pass.draw_indexed(
                self.quad_index_buffer.draw_count(),
                0,
                0..(self.rectangle_lines.len() as u32),
            );

            // TODO: Think about some memory releasing strategy. Spike in number of
            // rectangles will lead to space leak.
            self.rectangle_lines.clear();
            self.rectangle_lines_transforms.clear();
        }
    }
}
