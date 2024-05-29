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

pub struct MeshRendering {
}

impl MeshRendering {
    pub fn new(rendering_context: &mut RenderingContext) -> Self {
        let circle_shader_id =
            rendering_context
                .resource_store
                .build_shader::<PathBuf>(&ShaderSource::StaticFile(include_wgsl!(
                    "../shaders/circle.wgsl"
                )));
        let circle_line_shader_id =
            rendering_context
                .resource_store
                .build_shader::<PathBuf>(&ShaderSource::StaticFile(include_wgsl!(
                    "../shaders/circle_line.wgsl"
                )));

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

        let circle_piepeline_layout_id =
            rendering_context
                .resource_store
                .build_pipeline_layout(&PipelineLayoutDescriptor {
                    label: "circle pipeline layout".to_string(),
                    bind_group_layouts: vec![rendering_context
                        .primary_camera
                        .bing_group_layout()
                        .clone()],
                    push_constant_ranges: Vec::new(),
                });

        let circles_pipeline =
            rendering_context
                .resource_store
                .build_render_pipeline(&RenderPipelineDescriptor {
                    label: "circle pipeline".to_string(),
                    layout: Some(circle_piepeline_layout_id),
                    vertex: VertexState {
                        module: circle_shader_id.clone(),
                        buffers: vec![
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Vec2>() as u64,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: vertex_attr_array![0 => Float32x2].to_vec(),
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Mat4>() as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: vertex_attr_array![
                                    1 => Float32x4,
                                    2 => Float32x4,
                                    3 => Float32x4,
                                    4 => Float32x4,
                                ]
                                .to_vec(),
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Circle>() as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: vertex_attr_array![5 => Float32x3, 6 => Float32]
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
                        module: circle_shader_id.clone(),
                        targets: targets.clone(),
                    }),
                    multiview: None,
                });

        let circle_line_piepeline_layout_id = rendering_context
            .resource_store
            .build_pipeline_layout(&PipelineLayoutDescriptor {
                label: "circle line pipeline layout".to_string(),
                bind_group_layouts: vec![rendering_context
                    .primary_camera
                    .bing_group_layout()
                    .clone()],
                push_constant_ranges: Vec::new(),
            });

        let circle_lines_pipeline =
            rendering_context
                .resource_store
                .build_render_pipeline(&RenderPipelineDescriptor {
                    label: "circle line pipeline".to_string(),
                    layout: Some(circle_line_piepeline_layout_id),
                    vertex: VertexState {
                        module: circle_line_shader_id.clone(),
                        buffers: vec![
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Vec2>() as u64,
                                step_mode: wgpu::VertexStepMode::Vertex,
                                attributes: vertex_attr_array![0 => Float32x2].to_vec(),
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<Mat4>() as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes: vertex_attr_array![
                                    1 => Float32x4,
                                    2 => Float32x4,
                                    3 => Float32x4,
                                    4 => Float32x4,
                                ]
                                .to_vec(),
                            },
                            VertexBufferLayout {
                                array_stride: std::mem::size_of::<CircleLine>() as u64,
                                step_mode: wgpu::VertexStepMode::Instance,
                                attributes:
                                    vertex_attr_array![5 => Float32x3, 6 => Float32, 7 => Float32]
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
                        module: circle_line_shader_id.clone(),
                        targets: targets.clone(),
                    }),
                    multiview: None,
                });

        Self {
            circles_buffer,
            circles,
            circle_lines_buffer,
            circle_lines,
            quad_vertex_buffer,
            quad_index_buffer,
            circles_pipeline,
            circle_lines_pipeline,
            circles_transforms,
            circle_lines_transforms,
            circles_transforms_buffer,
            circle_lines_transforms_buffer,
        }
    }

    pub fn add_circle(&mut self, transform: &Transform, circle: &Circle) {
        self.circles.push(circle.clone());
        self.circles_transforms.push(transform.matrix());
    }

    pub fn add_circle_line(&mut self, transform: &Transform, circle: &CircleLine) {
        self.circle_lines.push(circle.clone());
        self.circle_lines_transforms.push(transform.matrix());
    }

    pub fn render<'a>(
        &'a mut self,
        rendering_context: &'a RenderingContext,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if !self.circles.is_empty() {
            self.circles_buffer
                .write_data(&rendering_context.gpu_context, &self.circles);
            self.circles_transforms_buffer
                .write_data(&rendering_context.gpu_context, &self.circles_transforms);

            let pipeline = &rendering_context
                .resource_store
                .get_render_pipeline(&self.circles_pipeline);

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, rendering_context.primary_camera.bing_group(), &[]);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.circles_transforms_buffer.slice(..));
            render_pass.set_vertex_buffer(2, self.circles_buffer.slice(..));
            render_pass.set_index_buffer(
                self.quad_index_buffer.slice(..),
                self.quad_index_buffer.index_format(),
            );
            render_pass.draw_indexed(
                self.quad_index_buffer.draw_count(),
                0,
                0..(self.circles.len() as u32),
            );

            // TODO: Think about some memory releasing strategy. Spike in number of
            // circles will lead to space leak.
            self.circles.clear();
            self.circles_transforms.clear();
        }

        if !self.circle_lines.is_empty() {
            self.circle_lines_buffer
                .write_data(&rendering_context.gpu_context, &self.circle_lines);
            self.circle_lines_transforms_buffer.write_data(
                &rendering_context.gpu_context,
                &self.circle_lines_transforms,
            );

            let pipeline = &rendering_context
                .resource_store
                .get_render_pipeline(&self.circle_lines_pipeline);

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, rendering_context.primary_camera.bing_group(), &[]);
            render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.circle_lines_transforms_buffer.slice(..));
            render_pass.set_vertex_buffer(2, self.circle_lines_buffer.slice(..));
            render_pass.set_index_buffer(
                self.quad_index_buffer.slice(..),
                self.quad_index_buffer.index_format(),
            );
            render_pass.draw_indexed(
                self.quad_index_buffer.draw_count(),
                0,
                0..(self.circle_lines.len() as u32),
            );

            // TODO: Think about some memory releasing strategy. Spike in number of
            // circles will lead to space leak.
            self.circle_lines.clear();
            self.circle_lines_transforms.clear();
        }
    }
}
