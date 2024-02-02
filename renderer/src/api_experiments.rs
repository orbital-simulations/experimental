use std::{ops::RangeBounds, marker::PhantomData, rc::Rc};

use wgpu::{Buffer, BufferAddress, BufferSlice, util::DeviceExt, BufferUsages, RenderPassDescriptor};

use crate::{raw::{Raw, Gpu}, context::Context};

//struct Buff<T, S>{
//    name: String,
//    buffer: Option<Buffer>,
//    count: usize,
//    buffer_data_type: PhantomData<T>,
//    buffer_step: PhantomData<U>,
//}
//
//impl <T, S>Buff<T, S> {
////    pub fn new(context: &Context, name: &str, data: &[T]) -> Self {
////        let buffer = context
////            .device
////            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
////                label: Some(name),
////                contents: data.get_raw(),
////                usage,
////            });
////        Self {
////            count: data.len(),
////            buffer,
////            name: name.to_string(),
////            usage,
////            phantom_data: PhantomData,
////        }
////    }
//
//    fn recreate_buffer(&mut self, context: &Context, new_len: usize, data: &[T]) where T: Raw+Gpu {
//        let buffer = context
//            .device
//            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                label: Some(&self.name),
//                contents: data.get_raw(),
//                usage: BufferUsages::VERTEX,
//            });
//        self.buffer = Some(buffer);
//        self.count = new_len;
//    }
//    fn write(&mut self, context: &Context, data: &[T]) {
//        let new_len = data.len();
//        if let Some(buffer) = &self.buffer {
//            if self.buffer.is_none() || self.count < new_len {
//                self.recreate_buffer(context, new_len, data);
//            } else {
//                context
//                    .queue
//                    .write_buffer(buffer, 0, data.get_raw());
//            }
//        } else {
//            self.recreate_buffer(context, new_len, data);
//        }
//    }
//
//    fn clear(&mut self) {
//        self.buffer = None;
//        self.count = 0;
//    }
//}
//
//pub trait GetVertextAttribute {
//    fn getVertextBufferLayout() -> VertexAttribute {
//    }
//}
//
//struct VertexStep;
//type VertextBuffer<T> = Buff<T, VertextBufferType>;
//
//struct InstanceStep;
//type InstanceBuffer<T> = Buff<T, InstanceStep>;
//
//struct UniformType;
//type UniformBuffer = Buff<T, UniformType>;
//
//pub enum VertexStepMode {
//    Vertex = 0,
//    Instance = 1,
//}
//
//pub struct VertexAttribute {
//    pub format: VertexFormat,
//    pub offset: u64,
//    pub shader_location: u32,
//}
//
//struct Vec3;
//struct Vec4;
//struct Mat4;
//struct Mat3;
//
//struct RendererBase;
//
//struct DepthTexture;
//
//struct RenderPass;
//
//impl <'a> RenderPass {
//    fn description(&'a self) -> &RenderPassDescriptor<'a, 'a> {
//        todo!()
//    }
//}
//
//struct RenderGraph;
//
//struct Material;
//
//struct Pipeline {
//    device_pipeline: Option<wgpu::RenderPipeline>,
//}
//
//struct PipelineParams;
//
//type BufferBundle<T> = [(u32, Buff<T>)];
//
//impl Pipeline {
//    pub fn new(render_pass_descriptor: &RenderPass, buffer: &BufferBundle) {
//    }
//    pub fn draw() {
//    }
//}
//
//impl RendererBase {
//    pub fn get_pipeline() -> PipelineId {
//    }
//}
//
//pub struct ShaderStages: u32 {
//    const NONE = 0;
//    const VERTEX = 1 << 0;
//    const FRAGMENT = 1 << 1;
//    const COMPUTE = 1 << 2;
//    const VERTEX_FRAGMENT = Self::VERTEX.bits() | Self::FRAGMENT.bits();
//}
//
//// Device::create_pipeline_layout
//pub struct PipelineLayoutDescriptor<'a> {
//    pub label: Label<'a>,
//    pub bind_group_layouts: &'a [&'a BindGroupLayout],
//    pub push_constant_ranges: &'a [PushConstantRange],
//}
//pub struct PushConstantRange {
//    pub stages: ShaderStages,
//    pub range: Range<u32>,
//}
//
//// Device::create_bind_group_layout() -> BindGroupLayout
//pub struct BindGroupLayoutDescriptor<'a> {
//    pub label: Label<'a>,
//    pub entries: &'a [BindGroupLayoutEntry],
//}
//
//pub struct BindGroupLayoutEntry {
//    pub binding: u32,
//    pub visibility: ShaderStages,
//    pub ty: BindingType,
//    pub count: Option<NonZeroU32>,
//}
//
//pub enum BindingType {
//    Buffer {
//        ty: BufferBindingType,
//        has_dynamic_offset: bool,
//        min_binding_size: Option<NonZeroU64>,
//    },
//    Sampler(SamplerBindingType),
//    Texture {
//        sample_type: TextureSampleType,
//        view_dimension: TextureViewDimension,
//        multisampled: bool,
//    },
//    StorageTexture {
//        access: StorageTextureAccess,
//        format: TextureFormat,
//        view_dimension: TextureViewDimension,
//    },
//    AccelerationStructure,
//}
//
//// Device::create_render_pipeline() -> RenderPipeline
//pub struct RenderPipelineDescriptor<'a> {
//    pub label: Label<'a>,
//    pub layout: Option<&'a PipelineLayout>,
//    pub vertex: VertexState<'a>,
//    pub primitive: PrimitiveState,
//    pub depth_stencil: Option<DepthStencilState>,
//    pub multisample: MultisampleState,
//    pub fragment: Option<FragmentState<'a>>,
//    pub multiview: Option<NonZeroU32>,
//}
//
//pub struct VertexState<'a> {
//    pub module: &'a ShaderModule,
//    pub entry_point: &'a str,
//    pub buffers: &'a [VertexBufferLayout<'a>],
//}
//
//pub struct VertexBufferLayout<'a> {
//    pub array_stride: BufferAddress,
//    pub step_mode: VertexStepMode,
//    pub attributes: &'a [VertexAttribute],
//}
//
//pub enum VertexStepMode {
//    Vertex = 0,
//    Instance = 1,
//}
//
//pub struct VertexAttribute {
//    pub format: VertexFormat,
//    pub offset: u64,
//    pub shader_location: u32,
//}
//
//pub enum VertexFormat {
//    Uint8x2 = 0,
//    Uint8x4 = 1,
//    Sint8x2 = 2,
//    Sint8x4 = 3,
//    Unorm8x2 = 4,
//    Unorm8x4 = 5,
//    Snorm8x2 = 6,
//    Snorm8x4 = 7,
//    Uint16x2 = 8,
//    Uint16x4 = 9,
//    Sint16x2 = 10,
//    Sint16x4 = 11,
//    Unorm16x2 = 12,
//    Unorm16x4 = 13,
//    Snorm16x2 = 14,
//    Snorm16x4 = 15,
//    Float16x2 = 16,
//    Float16x4 = 17,
//    Float32 = 18,
//    Float32x2 = 19,
//    Float32x3 = 20,
//    Float32x4 = 21,
//    Uint32 = 22,
//    Uint32x2 = 23,
//    Uint32x3 = 24,
//    Uint32x4 = 25,
//    Sint32 = 26,
//    Sint32x2 = 27,
//    Sint32x3 = 28,
//    Sint32x4 = 29,
//    Float64 = 30,
//    Float64x2 = 31,
//    Float64x3 = 32,
//    Float64x4 = 33,
//}


struct Buff<T> {
    buffer: Buffer,
    data_type: PhantomData<T>,
    element_count: usize,
    name: String,
}

struct MeshObject {
    vertertices: Buff,
    normals: Buff,
    indices: Buff,
    instances: Buff,
    pipeline: Arc<Pipeline>,
}

struct Pipeline {
}

struct PipelineId {
}

struct DrawBundle {
    vertext_buffers: &[(u32, Buffer)],
    bind_groups: &[(u32, BindGroup)],
    pipeline: PipelineId, // or Pipeline directly???
    use_camera_bundle: bool,
}

impl Renderer {
    pub fn draw() {
    }
}
