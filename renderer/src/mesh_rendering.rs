use std::mem::size_of;

use glam::{Mat4, Vec3};
use wgpu::vertex_attr_array;

use crate::{
    raw::Raw,
    rendering_context::RenderingContext,
    resource_store::{
        bind_group_layout::BindGroupLayoutId,
        gpu_mesh::GpuMeshId,
        pipeline_layout::PipelineLayoutDescriptor,
        render_pipeline::{
            FragmentState, PipelineId, RenderPipelineDescriptor, VertexBufferLayout, VertexState,
        },
        shader::ShaderSource,
    },
    transform::Transform,
};

#[derive(Debug, Clone)]
pub struct MeshBundle {
    pub mesh_id: GpuMeshId,
    pub pipeline_id: PipelineId,
}

pub struct MeshRendering {
    bundles: Vec<(Transform, MeshBundle)>,
    transform_uniform_bind_group_layout: BindGroupLayoutId,
    transform_uniform_bind_group: wgpu::BindGroup,
    transform_uniform_buffer: wgpu::Buffer,
    transform_uniform_buffer_size: usize,
}

fn ceil_to_next_multiple(value: usize, step: u32) -> u64 {
    let divide_and_ceil = value as u32 / step + if value as u32 % step == 0 { 0 } else { 1 };
    (step * divide_and_ceil) as u64
}

const TRANSFORMS_UNIFORM_BUFFER_NAME: &str = "3d mesh transforms uniform buffer";
const TRANSFORMS_UNIFORM_BIND_GROUP_NAME: &str = "3d mesh transforms bind group";

impl MeshRendering {
    pub fn new(rendering_context: &mut RenderingContext) -> Self {
        let transform_uniform_bind_group_layout = rendering_context
            .resource_store
            .build_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("3d mesh transforms uniform layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });
        let transform_uniform_buffer =
            rendering_context
                .gpu_context
                .device()
                .create_buffer(&wgpu::BufferDescriptor {
                    label: Some(TRANSFORMS_UNIFORM_BUFFER_NAME),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    size: 1, // This is a bit of a hack to make sure the first
                    // bind group created correctly, which means we
                    // don't need to use `Option` n bothw buffer and
                    // bind group.
                    mapped_at_creation: false,
                });
        let transform_uniform_bind_group = rendering_context
            .gpu_context
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(TRANSFORMS_UNIFORM_BIND_GROUP_NAME),
                layout: rendering_context
                    .resource_store
                    .get_bing_group_layout(&transform_uniform_bind_group_layout),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        transform_uniform_buffer.as_entire_buffer_binding(),
                    ),
                }],
            });

        Self {
            bundles: Vec::new(),
            transform_uniform_bind_group_layout,
            transform_uniform_buffer,
            transform_uniform_buffer_size: 0,
            transform_uniform_bind_group,
        }
    }

    pub fn add_mesh_bundle(&mut self, transform: &Transform, mesh_bundle: &MeshBundle) {
        self.bundles.push((transform.clone(), mesh_bundle.clone()));
    }

    pub fn create_3d_pipeline(
        &self,
        rendering_context: &mut RenderingContext,
        shader: &ShaderSource,
    ) -> PipelineId {
        let shader_id = rendering_context.resource_store.build_shader(shader);

        let pipeline_layout_id =
            rendering_context
                .resource_store
                .build_pipeline_layout(&PipelineLayoutDescriptor {
                    label: "3d mesh pipeline layout".to_string(),
                    bind_group_layouts: vec![
                        rendering_context.primary_camera.bing_group_layout().clone(),
                        self.transform_uniform_bind_group_layout.clone(),
                    ],
                    push_constant_ranges: Vec::new(),
                });

        let targets: Vec<Option<wgpu::ColorTargetState>> = vec![Some(wgpu::ColorTargetState {
            format: rendering_context.primary_camera.surface_format(),
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent::REPLACE,
                alpha: wgpu::BlendComponent::REPLACE,
            }),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        rendering_context
            .resource_store
            .build_render_pipeline(&RenderPipelineDescriptor {
                label: "3d mesh pipeline".to_string(),
                layout: Some(pipeline_layout_id),
                vertex: VertexState {
                    module: shader_id.clone(),
                    buffers: vec![
                        VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vec3>() as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: vertex_attr_array![0 => Float32x3].to_vec(),
                        },
                        VertexBufferLayout {
                            array_stride: std::mem::size_of::<Vec3>() as u64,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: vertex_attr_array![
                                1 => Float32x3,
                            ]
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
                    module: shader_id.clone(),
                    targets: targets.clone(),
                }),
                multiview: None,
            })
    }

    pub fn render<'a>(
        &'a mut self,
        rendering_context: &'a RenderingContext,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        if !self.bundles.is_empty() {
            let aligned_size = ceil_to_next_multiple(
                size_of::<Mat4>(),
                RenderingContext::wgpu_limits().min_uniform_buffer_offset_alignment,
            );
            if self.transform_uniform_buffer_size < self.bundles.len() {
                self.transform_uniform_buffer = rendering_context
                    .gpu_context
                    .device()
                    .create_buffer(&wgpu::BufferDescriptor {
                        label: Some(TRANSFORMS_UNIFORM_BUFFER_NAME),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        size: aligned_size * self.bundles.len() as u64,
                        mapped_at_creation: false,
                    });
                self.transform_uniform_buffer_size = self.bundles.len();
                self.transform_uniform_bind_group = rendering_context
                    .gpu_context
                    .device()
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(TRANSFORMS_UNIFORM_BIND_GROUP_NAME),
                        layout: rendering_context
                            .resource_store
                            .get_bing_group_layout(&self.transform_uniform_bind_group_layout),
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &self.transform_uniform_buffer,
                                offset: 0,
                                size: Some(
                                    std::num::NonZeroU64::new(size_of::<Mat4>() as u64).unwrap(),
                                ),
                            }),
                        }],
                    });
            }
            for (i, bundle) in self.bundles.iter().enumerate() {
                rendering_context.gpu_context.queue().write_buffer(
                    &self.transform_uniform_buffer,
                    aligned_size * i as u64,
                    bundle.0.matrix().get_raw(),
                );
            }

            for (i, bundle) in self.bundles.iter().enumerate() {
                let pipeline = &rendering_context
                    .resource_store
                    .get_render_pipeline(&bundle.1.pipeline_id);

                let gpu_mesh = rendering_context
                    .resource_store
                    .get_gpu_mesh(&bundle.1.mesh_id);

                render_pass.set_pipeline(pipeline);
                render_pass.set_bind_group(0, rendering_context.primary_camera.bing_group(), &[]);
                render_pass.set_bind_group(
                    1,
                    &self.transform_uniform_bind_group,
                    &[i as u32 * aligned_size as u32],
                );
                render_pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, gpu_mesh.normal_buffer.slice(..));
                render_pass.set_index_buffer(
                    gpu_mesh.index_buffer.slice(..),
                    gpu_mesh.index_buffer.index_format(),
                );
                render_pass.draw_indexed(gpu_mesh.index_buffer.draw_count(), 0, 0..1);
            }

            // TODO: Think about some memory releasing strategy. Spike in number of
            // circles will lead to space leak.
            self.bundles.clear();
        }
    }
}
