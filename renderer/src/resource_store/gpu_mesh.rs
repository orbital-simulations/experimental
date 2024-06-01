use glam::Vec3;
use slotmap::{new_key_type, SlotMap};

use crate::{
    buffers::{IndexBuffer, WriteableVecBuffer},
    gpu_context::GpuContext,
};

#[derive(Debug)]
pub struct GpuMesh {
    pub vertex_buffer: WriteableVecBuffer<Vec3>,
    pub normal_buffer: WriteableVecBuffer<Vec3>,
    pub index_buffer: IndexBuffer<u32>,
}

new_key_type! {
    pub struct GpuMeshId;
}

pub struct GpuMeshStore {
    store: SlotMap<GpuMeshId, GpuMesh>,
    gpu_context: GpuContext,
}

impl GpuMeshStore {
    pub fn new(gpu_context: &GpuContext) -> Self {
        GpuMeshStore {
            store: SlotMap::with_key(),
            gpu_context: gpu_context.clone(),
        }
    }

    pub fn build_gpu_mesh(
        &mut self,
        vertices: &Vec<Vec3>,
        normals: &Vec<Vec3>,
        indices: &[u32],
    ) -> GpuMeshId {
        let vertex_buffer = WriteableVecBuffer::new(
            &self.gpu_context,
            "mesh vertex buffer",
            &vertices,
            wgpu::BufferUsages::VERTEX,
        );
        let normal_buffer = WriteableVecBuffer::new(
            &self.gpu_context,
            "mesh normals buffer",
            &normals,
            wgpu::BufferUsages::VERTEX,
        );

        let index_buffer = IndexBuffer::new(&self.gpu_context, "gpu mesh", indices);
        self.store.insert(GpuMesh {
            vertex_buffer,
            normal_buffer,
            index_buffer,
        })
    }

    pub fn get_gpu_mesh(&self, gpu_mesh_id: GpuMeshId) -> &GpuMesh {
        &self.store[gpu_mesh_id]
    }
}
