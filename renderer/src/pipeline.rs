use std::{any::{Any, TypeId}, cell::RefCell, collections::HashMap, ops::Deref, rc::{Rc, Weak}};

use wgpu::{
    BindGroupLayout, ColorTargetState, CompareFunction, DepthBiasState, DepthStencilState,
    RenderPipeline, ShaderModule, StencilState, TextureFormat, VertexBufferLayout,
};

use crate::context::{Context, RenderingContext};

#[derive(Debug, Clone)]
pub struct RenderTargetDescription {
    pub multisampling: u32,
    pub depth_texture: Option<TextureFormat>,
    pub targets: Vec<TextureFormat>,
}

pub struct CreatePipeline<'a> {
    pub shader: &'a ShaderModule,
    pub vertex_buffer_layouts: Vec<VertexBufferLayout<'static>>,
    pub bind_group_layouts: Vec<&'a BindGroupLayout>,
    pub name: String,
}

#[derive(Clone)]
pub struct Pipeline {
    // TODO: This will need to be Arc in future
    value_rc: Rc<PipelineStoreEntryWrapper>,
}

impl Deref for Pipeline {
    type Target = RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.value_rc
    }
}

pub trait PipelineDescriptable {
    fn pipeline_description<'a>(&'a self, rendering_context: &'a RenderingContext) -> CreatePipeline<'a>;
}

struct PipelineStoreEntryWrapper {
    type_label: TypeId,
    // TODO: RefCell needs to be replaced with something for thread safety.
    pipeline_store: Weak<RefCell<HashMap<TypeId, Weak<PipelineStoreEntryWrapper>>>>,
    pipeline: RenderPipeline,
}

impl Deref for PipelineStoreEntryWrapper {
    type Target = RenderPipeline;

    fn deref(&self) -> &Self::Target {
        &self.pipeline
    }
}

impl Drop for PipelineStoreEntryWrapper {
    fn drop(&mut self) {
        let shader_store = Weak::upgrade(&self.pipeline_store);
        if let Some(shader_store) = shader_store {
            (*shader_store).borrow_mut().remove(&self.type_label);
        }
    }
}

#[derive(Debug)]
pub struct PipelineStore {
    // TODO: Later this will need to be something like Arc<TwLock<HashMap<..>>>
    // to prevent race conditions.
    shaders: Rc<RefCell<HashMap<TypeId, Weak<PipelineStoreEntryWrapper>>>>,
}

impl Default for PipelineStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineStore {
    pub fn new() -> Self {
        Self {
            shaders: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    fn create_pipeline<T>(
        &self,
        context: &Context,
        pipeline_label: &T,
        render_target_description: &RenderTargetDescription,
        rendering_context: &RenderingContext,
    ) -> (Rc<PipelineStoreEntryWrapper>, Weak<PipelineStoreEntryWrapper>)
    where
        T: PipelineDescriptable + Any,
    {
        let parameters = pipeline_label.pipeline_description(rendering_context);
        let mut pipeline_layout_descriptor_name = parameters.name.clone();
        pipeline_layout_descriptor_name.push_str("layout descriptor");

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(pipeline_layout_descriptor_name.as_str()),
                    bind_group_layouts: &parameters.bind_group_layouts,
                    push_constant_ranges: &[],
                });

        let targets: Vec<Option<ColorTargetState>> = render_target_description
            .targets
            .iter()
            .map(|target_texture_format| {
                Some(wgpu::ColorTargetState {
                    format: *target_texture_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })
            })
            .collect();

        let depth_stencil =
            render_target_description
                .depth_texture
                .map(|format| DepthStencilState {
                    format,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                });
        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&parameters.name),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: parameters.shader,
                    entry_point: "vs_main",
                    buffers: &parameters.vertex_buffer_layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: parameters.shader,
                    entry_point: "fs_main",
                    targets: &targets,
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: render_target_description.multisampling,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });

        let strong_pipeline = Rc::new(PipelineStoreEntryWrapper {
            pipeline_store: Rc::downgrade(&self.shaders),
            pipeline,
            type_label: pipeline_label.type_id(),
        });
        let weak_pipeline = Rc::downgrade(&strong_pipeline);
        (strong_pipeline, weak_pipeline)
    }

    pub fn get_pipeline<T>(&mut self, context: &Context, pipeline_label: &T,
        render_target_description: &RenderTargetDescription,
        rendering_context: &RenderingContext,
    ) -> Pipeline
    where
        T: PipelineDescriptable + Any,
    {
        let mut borrowed = self.shaders.borrow_mut();
        let possible_shader = borrowed.get(&pipeline_label.type_id());
        match possible_shader.as_ref() {
            Some(weak_shader) => match weak_shader.upgrade() {
                None => {
                    let (strong_pipeline, weak_pipeline) = self.create_pipeline(context, pipeline_label, render_target_description, rendering_context);
                    borrowed.insert(pipeline_label.type_id(), weak_pipeline);
                    Pipeline {
                        value_rc: strong_pipeline,
                    }
                }
                Some(strong_shader) => Pipeline {
                    value_rc: strong_shader,
                },
            },
            None => {
                    let (strong_pipeline, weak_pipeline) = self.create_pipeline(context, pipeline_label, render_target_description, rendering_context);
                borrowed.insert(pipeline_label.type_id(), weak_pipeline);
                Pipeline {
                    value_rc: strong_pipeline,
                }
            }
        }
    }
}
