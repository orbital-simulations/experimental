use glam::Vec3;
use wgpu::{
    util::DeviceExt, Buffer, BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat,
};

use crate::{context::Context, raw::Raw};

#[derive(Debug)]
pub struct GpuMesh {
    pub vertex_buffer: Buffer,
    pub vertex_buffer_layout: VertexBufferLayout<'static>,
    pub normal_buffer_layout: VertexBufferLayout<'static>,
    pub normal_buffer: Buffer,
    pub vertex_count: u32,
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
    pub fn new(context: &Context, vertices: &[Vec3], normals: &[Vec3]) -> GpuMesh {
        let vertex_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(concat!(prefix_label!(), "vertex buffer")),
                contents: vertices.get_raw(),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let vertex_buffer_layout = mesh_vertext_buffer_description();
        let normal_buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(concat!(prefix_label!(), "normals buffer")),
                contents: normals.get_raw(),
                usage: wgpu::BufferUsages::VERTEX,
            });

        GpuMesh {
            vertex_buffer,
            vertex_buffer_layout,
            normal_buffer,
            normal_buffer_layout: mesh_normal_description(),
            vertex_count: vertices.len() as u32,
        }
    }
}
