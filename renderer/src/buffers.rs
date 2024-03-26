use std::{marker::PhantomData, ops::RangeBounds, rc::Rc};

use wgpu::{
    util::DeviceExt, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource,
    Buffer, BufferAddress, BufferSlice, BufferUsages, IndexFormat, RenderPass, ShaderStages,
};

use crate::{
    context::Context,
    raw::{Gpu, Raw},
};

#[derive(Debug)]
pub struct WriteableBuffer<T: Gpu> {
    buffer: Buffer,
    count: usize,
    name: String,
    usage: BufferUsages,
    phantom_data: PhantomData<T>,
}

impl<T: Gpu> WriteableBuffer<T> {
    pub fn new(context: &Context, name: &str, data: &[T], usage: BufferUsages) -> Self {
        let usage = usage | BufferUsages::COPY_DST;
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
    pub fn write_data(&mut self, context: &Context, new_data: &[T]) {
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

    pub fn write_data_shrinking(&mut self, context: &Context, new_data: &[T]) {
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

    pub fn slice<S: RangeBounds<BufferAddress>>(&self, bounds: S) -> BufferSlice<'_> {
        self.buffer.slice(bounds)
    }
}

pub trait BindableBuffer {
    fn as_entire_binding(&self) -> BindingResource<'_>;
}

impl<T: Gpu> BindableBuffer for WriteableBuffer<T> {
    fn as_entire_binding(&self) -> BindingResource<'_> {
        self.buffer.as_entire_binding()
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
pub struct IndexBuffer<T: IndexFormatTrait + Gpu> {
    buffer: WriteableBuffer<T>,
}

impl<T: IndexFormatTrait + Gpu> IndexBuffer<T> {
    pub fn new(context: &Context, base_name: &str, data: &[T]) -> Self {
        let mut name = base_name.to_string();
        name.push_str(" index buffer");
        let buffer = WriteableBuffer::new(
            context,
            &name,
            data,
            BufferUsages::INDEX | BufferUsages::COPY_DST,
        );
        Self { buffer }
    }

    pub fn buffer(&mut self) -> &mut WriteableBuffer<T> {
        &mut self.buffer
    }

    pub fn set_index_buffer<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        render_pass.set_index_buffer(self.buffer.buffer.slice(..), T::index_format());
    }

    pub fn draw_count(&self) -> u32 {
        self.buffer.count as u32
    }
}

pub trait DescriptiveBindGroupEntry {
    fn bind_group_entry_description(
        &self,
        binding: u32,
        shader_stage: ShaderStages,
    ) -> BindGroupLayoutEntry;
}

impl<T: Gpu> DescriptiveBindGroupEntry for WriteableBuffer<T> {
    fn bind_group_entry_description(
        &self,
        binding: u32,
        shader_stage: ShaderStages,
    ) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            visibility: shader_stage,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
}

pub struct BindGroup {
    bind_group: Rc<wgpu::BindGroup>,
    bind_group_layout: Rc<BindGroupLayout>,
}

pub trait DescriptiveBindableGroupEntry: DescriptiveBindGroupEntry + BindableBuffer {}
impl<T: DescriptiveBindGroupEntry + BindableBuffer> DescriptiveBindableGroupEntry for T {}

impl BindGroup {
    pub fn new(
        context: &Context,
        name: &str,
        buffers: &[(u32, wgpu::ShaderStages, &dyn DescriptiveBindableGroupEntry)],
    ) -> Self {
        let entires: Vec<BindGroupLayoutEntry> = buffers
            .iter()
            .map(|(binding, stage, buffer)| buffer.bind_group_entry_description(*binding, *stage))
            .collect();
        let mut layout_name: String = name.into();
        layout_name.push_str(" layout");
        let bind_group_layout =
            context
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&layout_name),
                    entries: &entires,
                });

        let entries: Vec<BindGroupEntry> = buffers
            .iter()
            .map(|(binding, _stage, buffer)| BindGroupEntry {
                binding: *binding,
                resource: buffer.as_entire_binding(),
            })
            .collect();

        let bind_group = context
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &entries,
                label: Some(name),
            });

        Self {
            bind_group: Rc::new(bind_group),
            bind_group_layout: Rc::new(bind_group_layout),
        }
    }

    pub fn layout(&self) -> Rc<BindGroupLayout> {
        self.bind_group_layout.clone()
    }

    pub fn bind<'a>(&'a self, render_pass: &mut RenderPass<'a>, slot: u32) {
        render_pass.set_bind_group(slot, &self.bind_group, &[]);
    }
}
