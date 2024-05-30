use glam::Vec3;

use super::store_base::{StoreBase, StoreEntityId};
use crate::{
    buffers::{IndexBuffer, WriteableBuffer},
    gpu_context::GpuContext,
};

#[derive(Debug)]
pub struct GpuMesh {
    pub vertex_buffer: WriteableBuffer<Vec<Vec3>>,
    pub normal_buffer: WriteableBuffer<Vec<Vec3>>,
    pub index_buffer: IndexBuffer<u32>,
}

pub type GpuMeshId = StoreEntityId<GpuMesh>;

pub struct GpuMeshStore {
    store: StoreBase<GpuMesh>,
    gpu_context: GpuContext,
}

impl GpuMeshStore {
    pub fn new(gpu_context: &GpuContext) -> Self {
        GpuMeshStore {
            store: StoreBase::new(),
            gpu_context: gpu_context.clone(),
        }
    }

    pub fn build_gpu_mesh(
        &mut self,
        vertices: &Vec<Vec3>,
        normals: &Vec<Vec3>,
        indices: &[u32],
    ) -> GpuMeshId {
        let vertex_buffer = WriteableBuffer::new(
            &self.gpu_context,
            "mesh vertex buffer",
            vertices,
            wgpu::BufferUsages::VERTEX,
        );
        let normal_buffer = WriteableBuffer::new(
            &self.gpu_context,
            "mesh normals buffer",
            normals,
            wgpu::BufferUsages::VERTEX,
        );

        let index_buffer = IndexBuffer::new(&self.gpu_context, "gpu mesh", indices);
        self.store.add(GpuMesh {
            vertex_buffer,
            normal_buffer,
            index_buffer,
        })
    }

    pub fn get_gpu_mesh(&self, gpu_mesh_id: &GpuMeshId) -> &GpuMesh {
        self.store.get(gpu_mesh_id)
    }
}
