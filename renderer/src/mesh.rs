use glam::Vec3;
use wgpu::{
    util::DeviceExt, Buffer, BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat,
};

use crate::{buffers::IndexBuffer, context::Context, raw::Raw};

#[derive(Debug)]
pub struct GpuMesh {
    pub vertex_buffer: Buffer,
    pub vertex_buffer_layout: VertexBufferLayout<'static>,
    pub normal_buffer_layout: VertexBufferLayout<'static>,
    pub normal_buffer: Buffer,
    pub index_buffer: IndexBuffer<u32>,
}

macro_rules! prefix_label {
    () => {
        "Mesh "
    };
}

pub fn mesh_vertext_buffer_description() -> VertexBufferLayout<'static> {
    VertexBufferLayout {
        array_stride: std::mem::size_of::<Vec3>() as BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 0,
            shader_location: 0,
        }],
    }
}

pub fn mesh_normal_description() -> VertexBufferLayout<'static> {
    VertexBufferLayout {
        array_stride: std::mem::size_of::<Vec3>() as BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[VertexAttribute {
            format: VertexFormat::Float32x3,
            offset: 0,
            shader_location: 1,
        }],
    }
}

impl GpuMesh {
    pub fn new(context: &Context, vertices: &[Vec3], normals: &[Vec3], indices: &[u32]) -> GpuMesh {
        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(concat!(prefix_label!(), "vertex buffer")),
                contents: vertices.get_raw(),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let normal_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(concat!(prefix_label!(), "normals buffer")),
                contents: normals.get_raw(),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = IndexBuffer::new(context, indices, "gpu mesh");
        GpuMesh {
            vertex_buffer,
            vertex_buffer_layout: mesh_vertext_buffer_description(),
            normal_buffer,
            normal_buffer_layout: mesh_normal_description(),
            index_buffer,
        }
    }
}
