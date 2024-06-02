use std::{
    marker::PhantomData,
    ops::{Range, RangeBounds},
};

use bytemuck::{bytes_of, must_cast_slice, NoUninit};
use wgpu::{util::DeviceExt, Buffer, BufferUsages, IndexFormat};

use crate::gpu_context::GpuContext;

#[derive(Debug)]
pub struct WriteableBuffer<T: NoUninit> {
    buffer: Buffer,
    phantom_data: PhantomData<T>,
}

impl<T: NoUninit> WriteableBuffer<T> {
    pub fn new(gpu_context: &GpuContext, name: &str, data: &T, usage: BufferUsages) -> Self {
        let usage = usage | BufferUsages::COPY_DST;
        let buffer = gpu_context
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: bytes_of(data),
                usage,
            });
        Self {
            buffer,
            phantom_data: PhantomData,
        }
    }

    pub fn write_data(&mut self, gpu_context: &GpuContext, new_data: &T) {
        gpu_context
            .queue()
            .write_buffer(&self.buffer, 0, bytes_of(new_data));
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn slice<S: RangeBounds<wgpu::BufferAddress>>(&self, bounds: S) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(bounds)
    }
}

#[derive(Debug)]
pub struct WriteableVecBuffer<T: NoUninit> {
    buffer: Buffer,
    count: usize,
    name: String,
    usage: BufferUsages,
    phantom_data: PhantomData<T>,
}

impl<T: NoUninit> WriteableVecBuffer<T> {
    pub fn new(gpu_context: &GpuContext, name: &str, data: &[T], usage: BufferUsages) -> Self {
        let usage = usage | BufferUsages::COPY_DST;
        let byte_data = must_cast_slice(data);
        let buffer = gpu_context
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: byte_data,
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

    pub fn write_data(&mut self, gpu_context: &GpuContext, new_data: &[T]) {
        let new_len = new_data.len();
        let byte_data: &[u8] = must_cast_slice(new_data);
        if self.count < new_len {
            let buffer =
                gpu_context
                    .device()
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&self.name),
                        contents: byte_data,
                        usage: self.usage,
                    });
            self.buffer = buffer;
            self.count = new_data.len();
        } else {
            gpu_context.queue().write_buffer(&self.buffer, 0, byte_data);
        }
    }

    pub fn write_data_shrinking(&mut self, gpu_context: &GpuContext, new_data: &[T]) {
        let new_len = new_data.len();
        let byte_data: &[u8] = must_cast_slice(new_data);
        if self.count != new_len {
            let buffer =
                gpu_context
                    .device()
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(&self.name),
                        contents: byte_data,
                        usage: self.usage,
                    });
            self.buffer = buffer;
            self.count = new_data.len();
        } else {
            gpu_context.queue().write_buffer(&self.buffer, 0, byte_data);
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn slice<S: RangeBounds<wgpu::BufferAddress>>(&self, bounds: S) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(bounds)
    }
}

pub trait IndexFormatTrait {
    fn index_format() -> wgpu::IndexFormat
    where
        Self: Sized;
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
pub struct IndexBuffer<T: IndexFormatTrait + NoUninit> {
    buffer: Buffer,
    phantom_data: PhantomData<T>,
    count: u32,
}

impl<T: IndexFormatTrait + NoUninit> IndexBuffer<T> {
    pub fn new(gpu_context: &GpuContext, name: &str, data: &[T]) -> IndexBuffer<T> {
        let buffer = gpu_context
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(name),
                contents: must_cast_slice(data),
                usage: BufferUsages::INDEX,
            });
        Self {
            buffer,
            count: data.len() as u32,
            phantom_data: PhantomData,
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
    pub fn index_format(&self) -> wgpu::IndexFormat {
        T::index_format()
    }

    pub fn draw_count(&self) -> Range<u32> {
        0..self.count
    }

    pub fn slice<S: RangeBounds<wgpu::BufferAddress>>(&self, bounds: S) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(bounds)
    }
}
