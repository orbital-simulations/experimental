use std::{ops::RangeBounds, marker::PhantomData};

use wgpu::{Buffer, BufferAddress, BufferSlice, util::DeviceExt, BufferUsages};

use crate::{raw::{Raw, Gpu}, context::Context};

pub trait Buff {
    fn slice<S: RangeBounds<BufferAddress>>(&self, bounds: S) -> BufferSlice<'_>;
}

pub trait WriteBuff {
    type Item: Raw;
    fn write(&mut self, context: &Context, data: &[Self::Item]) where Self::Item: Raw;
    fn clear(&mut self);
}

pub trait ReadBuff {
    // This will need to change. I guess we will need two variants, one for
    // synch access one for async access.
    fn read<T>(&mut self) -> &[T];
}

// TODO: Most of them should take a gpu context as well???
// Maybe the context should be made into a global variable or some kind of
// pointer.
struct WriteVertextBuffer<T>{
    name: String,
    buffer: Option<Buffer>,
    count: usize,
    phantom_data: PhantomData<T>,
}

impl <T>WriteVertextBuffer<T> {
    pub fn new(context: &Context, name: &str, data: &[T]) -> Self {
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


    fn recreate_buffer(&mut self, context: &Context, new_len: usize, data: &[T]) where T: Raw+Gpu {
        let buffer = context
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&self.name),
                contents: data.get_raw(),
                usage: BufferUsages::VERTEX,
            });
        self.buffer = Some(buffer);
        self.count = new_len;
    }
}

impl <T>WriteBuff for WriteVertextBuffer<T> where T: Raw + Gpu {
    type Item = T;
    fn write(&mut self, context: &Context, data: &[Self::Item]) {
        let new_len = data.len();
        if let Some(buffer) = &self.buffer {
            if self.buffer.is_none() || self.count < new_len {
                self.recreate_buffer(context, new_len, data);
            } else {
                context
                    .queue
                    .write_buffer(buffer, 0, data.get_raw());
            }
        } else {
            self.recreate_buffer(context, new_len, data);
        }
    }

    fn clear(&mut self) {
        self.buffer = None;
        self.count = 0;
    }

}


struct ReadBuffer;
impl ReadBuffer {
    pub fn new() {
        todo!()
    }
    pub fn read<T: Raw>() -> Vec<T> {
        todo!()
    }
    pub fn map_to_gpu(/*/ maybe size should go here???*/) {
        todo!()
    }
}

struct StaticBuffer;
impl StaticBuffer {
    pub fn new<T: Raw>(data: &[T]) {
        todo!()
    }
    pub fn clear() {
        todo!()
    }
    pub fn splice() {
        todo!()
    }
}



struct VertextBuffer;
struct VertextMapping{
    buffer: VertextBuffer,
    shader_binding: u32,
}
struct IndexBuffer;

struct Vec3;
struct Vec4;
struct Mat4;
struct Mat3;

struct RendererBase;

struct DepthTexture;

struct MyRenderPass;
