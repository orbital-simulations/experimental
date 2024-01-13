use std::marker::PhantomData;

use glam::{Vec2, Vec3};
use wgpu::{
    util::DeviceExt,
    BindGroupLayoutEntry, Buffer, BufferAddress, BufferUsages, IndexFormat, RenderPass,
    VertexAttribute, VertexBufferLayout, VertexFormat,
};

use crate::{
    context::Context,
    raw::{Gpu, Raw},
};

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

pub fn vec3_buffer_description() -> VertexBufferLayout<'static> {
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

pub fn simple_vertex_uniform_layout_entry(binding: u32) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

#[derive(Debug)]
struct WriteableBuffer<T: Gpu> {
    buffer: Buffer,
    count: usize,
    name: String,
    usage: BufferUsages,
    phantom_data: PhantomData<T>,
}

impl<T: Gpu> WriteableBuffer<T> {
    pub fn new(context: &Context, name: &str, data: &[T], usage: BufferUsages) -> Self {
        let buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: data.get_raw(),
                usage,
            });
        Self {
            count: data.len(),
            buffer,
            name: name.to_string(),
            usage,
            phantom_data: PhantomData,
        }
    }
}

pub trait BufferWritter<T> {
    fn write_data(&mut self, context: &Context, new_data: &[T]);
    fn write_data_shrinking(&mut self, context: &Context, new_data: &[T]);
}

impl<T: Gpu> BufferWritter<T> for WriteableBuffer<T> {
    fn write_data(&mut self, context: &Context, new_data: &[T]) {
        let new_len = new_data.len();
        if self.count < new_len {
            let buffer = context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&self.name),
                    contents: new_data.get_raw(),
                    usage: self.usage,
                });
            self.buffer = buffer;
            self.count = new_len;
        } else {
            context
                .queue
                .write_buffer(&self.buffer, 0, new_data.get_raw());
        }
    }

    fn write_data_shrinking(&mut self, context: &Context, new_data: &[T]) {
        let new_len = new_data.len();
        if self.count != new_len {
            let buffer = context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&self.name),
                    contents: new_data.get_raw(),
                    usage: self.usage,
                });
            self.buffer = buffer;
            self.count = new_len;
        } else {
            context
                .queue
                .write_buffer(&self.buffer, 0, new_data.get_raw());
        }
    }
}

pub trait IndexFormatTrait {
    fn index_format() -> wgpu::IndexFormat;
}

impl IndexFormatTrait for u32 {
    fn index_format() -> IndexFormat {
        IndexFormat::Uint32
    }
}

impl IndexFormatTrait for u16 {
    fn index_format() -> IndexFormat {
        IndexFormat::Uint16
    }
}

#[derive(Debug)]
pub struct IndexBuffer<T: IndexFormatTrait + Gpu> {
    buffer: WriteableBuffer<T>,
}

impl<T: IndexFormatTrait + Gpu> IndexBuffer<T> {
    pub fn new(context: &Context, data: &[T], base_name: &str) -> Self {
        let mut name = base_name.to_string();
        name.push_str(" index buffer");
        let buffer = WriteableBuffer::new(context, &name, data, BufferUsages::INDEX);
        Self { buffer }
    }

    pub fn set_index_buffer<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_index_buffer(self.buffer.buffer.slice(..), T::index_format());
    }

    pub fn draw_count(&self) -> u32 {
        self.buffer.count as u32
    }
}
