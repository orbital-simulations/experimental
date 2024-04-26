use glam::Vec3;
use wgpu::{
    vertex_attr_array, BindGroupLayoutEntry, FrontFace, MultisampleState, PrimitiveState,
    PrimitiveTopology, RenderPass, ShaderStages, TextureFormat, VertexStepMode,
};

use crate::{
    context::{Context, RenderingContext},
    mesh::GpuMesh,
    pipeline::{
        BindGroupLayoutDescription, FragmentStateDescription, PipelineDescription, PipelineID, PipelineLayoutDescription, PipelineStore, UnlockedPipelineStore, VertexBufferLayoutDescriptor, VertexStateDescription
    },
    shader_store::{ShaderDescription, ShaderStore},
};

pub struct CustomMeshRenderer {
    shader_description: ShaderDescription,
    pipeline: Option<PipelineID>,
    mesh: GpuMesh,
}

impl CustomMeshRenderer {
    pub fn new(mesh: GpuMesh, shader_description: &ShaderDescription) -> Self
where {
        Self {
            shader_description: shader_description.clone(),
            pipeline: None,
            mesh,
        }
    }

    pub fn build_pipeline(
        &mut self,
        shader_store: &ShaderStore,
        pipeline_store: &PipelineStore,
        target_texture_format: &TextureFormat,
    ) {
        if self.pipeline.is_none() {
            let shader = {
                let mut shader_store = shader_store.lock();
                shader_store.get_or_create(&self.shader_description)
            };
            let pipeline_description = PipelineDescription {
                label: "custom mesh renderer".to_string(),
                layout: Some(PipelineLayoutDescription {
                    label: "custom mesh renderer layout".to_string(),
                    // TODO: Camera bind broup description should be taken from camera
                    bind_group_layouts: vec![BindGroupLayoutDescription {
                        label: "camera bind group layout description".to_string(),
                        entries: vec![
                            BindGroupLayoutEntry {
                                binding: 0,
                                visibility: ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                            BindGroupLayoutEntry {
                                binding: 1,
                                visibility: ShaderStages::VERTEX,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                    }],
                    push_constant_ranges: vec![],
                }),
                vertex: VertexStateDescription {
                    buffers: vec![
                        VertexBufferLayoutDescriptor {
                            array_stride: std::mem::size_of::<Vec3>() as u64,
                            step_mode: VertexStepMode::Vertex,
                            attributes: vertex_attr_array![0 => Float32x3].to_vec(),
                        },
                        VertexBufferLayoutDescriptor {
                            array_stride: std::mem::size_of::<Vec3>() as u64,
                            step_mode: VertexStepMode::Vertex,
                            attributes: vertex_attr_array![1 => Float32x3].to_vec(),
                        },
                    ],
                    module: shader.clone(),
                },
                // TODO: This should be generated from target...
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                // TODO: This should be part of the target as well...
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(FragmentStateDescription {
                    module: shader,
                    targets: vec![Some(wgpu::ColorTargetState {
                        format: *target_texture_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            };

            let mut pipeline_store = pipeline_store.lock();
            self.pipeline = Some(pipeline_store.get_or_create(&pipeline_description));
        }
    }

    pub fn render<'a>(
        &'a mut self,
        rendering_context: &'a RenderingContext,
        _context: &Context,
        render_pass: &mut RenderPass<'a>,
        //        render_target_description: &RenderTargetDescription,
        pipeline_store: &'a UnlockedPipelineStore<'a>,
    ) {
        if self.pipeline.is_none() {
            //            self.pipeline = Some(pipeline_store.get_entry(self, render_target_description));
        }

        let pipeline = self
            .pipeline
            .as_ref()
            .expect("pipeline should be created by now");
        let pipeline = pipeline_store.get_ref(pipeline);

        render_pass.set_pipeline(pipeline);
        rendering_context.camera().bind(render_pass, 0);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.mesh.normal_buffer.slice(..));
        self.mesh.index_buffer.set_index_buffer(render_pass);
        render_pass.draw_indexed(0..self.mesh.index_buffer.draw_count(), 0, 0..1);
    }
}
