pub mod bind_group_layout;
pub mod gpu_mesh;
pub mod pipeline_layout;
pub mod render_pipeline;
pub mod shader;
pub mod store_base;

use glam::Vec3;

use crate::gpu_context::GpuContext;

use self::{
    bind_group_layout::{BindGroupLayoutId, BindGroupLayoutStore},
    gpu_mesh::{GpuMesh, GpuMeshId, GpuMeshStore},
    pipeline_layout::{PipelineLayoutDescriptor, PipelineLayoutId, PipelineLayoutStore},
    render_pipeline::{PipelineId, RenderPipelineDescriptor, RenderPipelineStore},
    shader::{ShaderId, ShaderSource, ShaderStore},
};

pub struct ResourceStore {
    shader_store: ShaderStore,
    render_pipeline_store: RenderPipelineStore,
    pipeline_layout_store: PipelineLayoutStore,
    bind_group_layout_store: BindGroupLayoutStore,
    gpu_mesh_store: GpuMeshStore,
}

impl ResourceStore {
    pub fn new(gpu_context: &GpuContext) -> Self {
        let bind_group_layout_store = BindGroupLayoutStore::new(gpu_context);
        let pipeline_layout_store = PipelineLayoutStore::new(gpu_context);
        let shader_store = ShaderStore::new(gpu_context);
        let render_pipeline_store = RenderPipelineStore::new(gpu_context);
        let gpu_mesh_store = GpuMeshStore::new(gpu_context);

        Self {
            shader_store,
            render_pipeline_store,
            pipeline_layout_store,
            bind_group_layout_store,
            gpu_mesh_store,
        }
    }

    pub fn build_bind_group_layout(
        &mut self,
        bind_group_layout_descriptor: &wgpu::BindGroupLayoutDescriptor,
    ) -> BindGroupLayoutId {
        self.bind_group_layout_store
            .build_bind_group_layout(bind_group_layout_descriptor)
    }

    pub fn get_bing_group_layout(
        &self,
        bind_group_id: &BindGroupLayoutId,
    ) -> &wgpu::BindGroupLayout {
        self.bind_group_layout_store
            .get_bing_group_layout(bind_group_id)
    }

    pub fn build_pipeline_layout(
        &mut self,
        bind_group_layout_descriptor: &PipelineLayoutDescriptor,
    ) -> PipelineLayoutId {
        self.pipeline_layout_store
            .build_pipeline_layout(bind_group_layout_descriptor, &self.bind_group_layout_store)
    }

    pub fn get_pipeline_layout(
        &self,
        pipeline_layout_id: &PipelineLayoutId,
    ) -> &wgpu::PipelineLayout {
        self.pipeline_layout_store
            .get_pipeline_layout(pipeline_layout_id)
    }

    pub fn build_shader(&mut self, shader_source: &ShaderSource) -> ShaderId {
        self.shader_store.build_shader(shader_source)
    }

    pub fn get_shader(&self, shader_id: &ShaderId) -> &wgpu::ShaderModule {
        self.shader_store.get_shader(shader_id)
    }

    pub fn build_render_pipeline(
        &mut self,
        bind_group_layout_descriptor: &RenderPipelineDescriptor,
    ) -> PipelineId {
        self.render_pipeline_store.build_render_pipeline(
            bind_group_layout_descriptor,
            &self.shader_store,
            &self.pipeline_layout_store,
        )
    }

    pub fn get_render_pipeline(&self, pipeline_id: &PipelineId) -> &wgpu::RenderPipeline {
        self.render_pipeline_store.get_render_pipeline(pipeline_id)
    }

    pub fn build_gpu_mesh(
        &mut self,
        vertices: &Vec<Vec3>,
        normals: &Vec<Vec3>,
        indices: &[u32],
    ) -> GpuMeshId {
        self.gpu_mesh_store
            .build_gpu_mesh(vertices, normals, indices)
    }

    pub fn get_gpu_mesh(&self, gpu_mesh_id: &GpuMeshId) -> &GpuMesh {
        self.gpu_mesh_store.get_gpu_mesh(gpu_mesh_id)
    }
}
