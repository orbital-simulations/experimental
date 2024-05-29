use glam::Vec3;

use crate::{
    buffers::{IndexBuffer, WriteableBuffer},
    gpu_context::GpuContext,
};

#[derive(Debug)]
pub struct GpuMesh {
    pub vertex_buffer: WriteableBuffer<Vec3>,
    pub normal_buffer: WriteableBuffer<Vec3>,
    pub index_buffer: IndexBuffer<u32>,
}

impl GpuMesh {
    pub fn new(context: &GpuContext, vertices: &[Vec3], normals: &[Vec3], indices: &[u32]) -> GpuMesh {
        let vertex_buffer = WriteableBuffer::new(
            context,
            "mesh vertex buffer",
            vertices,
            wgpu::BufferUsages::VERTEX,
        );
        let normal_buffer = WriteableBuffer::new(
            context,
            "mesh normals buffer",
            normals,
            wgpu::BufferUsages::VERTEX,
        );

        let index_buffer = IndexBuffer::new(context, "gpu mesh", indices);
        GpuMesh {
            vertex_buffer,
            normal_buffer,
            index_buffer,
        }
    }
}
