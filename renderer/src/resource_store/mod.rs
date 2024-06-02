pub mod bind_group_layout;
pub mod gpu_mesh;
pub mod pipeline_layout;
pub mod reload_command;
pub mod render_pipeline;
pub mod shader;

use std::env;

use glam::Vec3;

use crate::{file_watcher::FileWatcher, gpu_context::GpuContext};

use self::{
    bind_group_layout::BindGroupLayoutStore,
    gpu_mesh::{GpuMesh, GpuMeshStore},
    pipeline_layout::{PipelineLayoutDescriptor, PipelineLayoutStore},
    render_pipeline::{RenderPipelineDescriptor, RenderPipelineStore},
    shader::{ShaderSource, ShaderStore},
};

pub use self::bind_group_layout::BindGroupLayoutId;
pub use self::gpu_mesh::GpuMeshId;
pub use self::pipeline_layout::PipelineLayoutId;
pub use self::render_pipeline::PipelineId;
pub use self::shader::ShaderId;

pub struct ResourceStore {
    file_watcher: FileWatcher,
    shader_store: ShaderStore,
    render_pipeline_store: RenderPipelineStore,
    pipeline_layout_store: PipelineLayoutStore,
    bind_group_layout_store: BindGroupLayoutStore,
    gpu_mesh_store: GpuMeshStore,
}

impl ResourceStore {
    pub fn new(gpu_context: &GpuContext) -> eyre::Result<Self> {
        let bind_group_layout_store = BindGroupLayoutStore::new(gpu_context);
        let pipeline_layout_store = PipelineLayoutStore::new(gpu_context);
        let shader_store = ShaderStore::new(gpu_context);
        let render_pipeline_store = RenderPipelineStore::new(gpu_context);
        let gpu_mesh_store = GpuMeshStore::new(gpu_context);
        let pwd = env::current_dir()?;
        let file_watcher = FileWatcher::new(pwd)?;

        Ok(Self {
            shader_store,
            render_pipeline_store,
            pipeline_layout_store,
            bind_group_layout_store,
            gpu_mesh_store,
            file_watcher,
        })
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
        bind_group_id: BindGroupLayoutId,
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
        pipeline_layout_id: PipelineLayoutId,
    ) -> &wgpu::PipelineLayout {
        self.pipeline_layout_store
            .get_pipeline_layout(pipeline_layout_id)
    }

    pub fn build_shader(&mut self, shader_source: &ShaderSource) -> ShaderId {
        self.shader_store
            .build_shader(&mut self.file_watcher, shader_source)
    }

    pub fn get_shader(&self, shader_id: ShaderId) -> &wgpu::ShaderModule {
        self.shader_store.get_shader(shader_id)
    }

    pub fn build_render_pipeline(
        &mut self,
        bind_group_layout_descriptor: &RenderPipelineDescriptor,
    ) -> PipelineId {
        self.render_pipeline_store.build_render_pipeline(
            bind_group_layout_descriptor,
            &mut self.shader_store,
            &self.pipeline_layout_store,
        )
    }

    pub fn get_render_pipeline(&self, pipeline_id: PipelineId) -> &wgpu::RenderPipeline {
        self.render_pipeline_store.get_render_pipeline(pipeline_id)
    }

    pub fn build_gpu_mesh(
        &mut self,
        vertices: &[Vec3],
        normals: &[Vec3],
        indices: &[u32],
    ) -> GpuMeshId {
        self.gpu_mesh_store
            .build_gpu_mesh(vertices, normals, indices)
    }

    pub fn get_gpu_mesh(&self, gpu_mesh_id: GpuMeshId) -> &GpuMesh {
        self.gpu_mesh_store.get_gpu_mesh(gpu_mesh_id)
    }

    pub fn reload_if_necessary(&mut self) {
        let mut dependants = self.file_watcher.process_updates();
        while let Some(dependant) = dependants.pop() {
            let new_dependants = match dependant {
                reload_command::RebuildCommand::Shader(shader_id) => {
                    self.shader_store.rebuild(shader_id)
                }
                reload_command::RebuildCommand::Pipeline(pipeline_id) => {
                    self.render_pipeline_store.rebuild(
                        &self.shader_store,
                        &self.pipeline_layout_store,
                        pipeline_id,
                    );
                    Vec::new()
                }
            };
            for new_dependant in new_dependants {
                dependants.push(new_dependant.clone());
            }
        }
    }
}
