use glam::Vec2;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat};

pub fn vec2_buffer_description<'a>() -> VertexBufferLayout<'a> {
    VertexBufferLayout {
        array_stride: std::mem::size_of::<Vec2>() as BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[VertexAttribute {
            format: VertexFormat::Float32x2,
            offset: 0,
            shader_location: 0,
        }],
    }
}
