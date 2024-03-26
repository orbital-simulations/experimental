use std::{cell::RefCell, ops::Deref, rc::Rc};

use wgpu::{
    BindGroupLayout, ColorTargetState, CompareFunction, DepthBiasState, DepthStencilState,
    RenderPipeline, StencilState, TextureFormat, VertexBufferLayout,
};

use crate::{
    context::{Context, RenderingContext},
    shader_store::Shader,
    store::{Entry, EntryLabel, EntryRebuilder, Store},
};

#[derive(Debug, Clone)]
pub struct RenderTargetDescription {
    pub multisampling: u32,
    pub depth_texture: Option<TextureFormat>,
    pub targets: Vec<TextureFormat>,
}

#[derive(Clone)]
pub struct CreatePipeline {
    pub shader: Shader,
    pub vertex_buffer_layouts: Vec<VertexBufferLayout<'static>>,
    pub bind_group_layouts: Vec<Rc<BindGroupLayout>>,
    pub name: String,
}

pub struct InternalRenderPipeline {
    pub pipeline: RenderPipeline,
    pub render_target_description: RenderTargetDescription,
    pub parameters: CreatePipeline,
}

impl EntryRebuilder<PipelineContext> for InternalRenderPipeline {
    fn rebuild(&self, pipeline_context: &PipelineContext) -> Self {
        println!("pipeline rebuild: {}", &self.parameters.name);
        let mut pipeline_layout_descriptor_name = self.parameters.name.clone();
        pipeline_layout_descriptor_name.push_str("layout descriptor");

        let as_ref: Vec<&BindGroupLayout> = self
            .parameters
            .bind_group_layouts
            .iter()
            .map(|kwa| kwa.as_ref())
            .collect();

        let pipeline_layout = pipeline_context.context.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some(pipeline_layout_descriptor_name.as_str()),
                bind_group_layouts: &as_ref,
                push_constant_ranges: &[],
            },
        );

        let targets: Vec<Option<ColorTargetState>> = self
            .render_target_description
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
            self.render_target_description
                .depth_texture
                .map(|format| DepthStencilState {
                    format,
                    depth_write_enabled: true,
                    depth_compare: CompareFunction::Less,
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                });

        InternalRenderPipeline {
            pipeline: pipeline_context.context.device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some(&self.parameters.name),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: self.parameters.shader.deref(),
                        entry_point: "vs_main",
                        buffers: &self.parameters.vertex_buffer_layouts,
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: self.parameters.shader.deref(),
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
                        count: self.render_target_description.multisampling,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    // If the pipeline will be used with a multiview render pass, this
                    // indicates how many array layers the attachments will have.
                    multiview: None,
                },
            ),
            render_target_description: self.render_target_description.clone(),
            parameters: self.parameters.clone(),
        }
    }
}

pub type Pipeline = Entry<PipelineContext, InternalRenderPipeline>;

pub trait PipelineDescriptable {
    fn pipeline_description(&self, rendering_context: &RenderingContext) -> CreatePipeline;
}

pub struct PipelineContext {
    context: Rc<Context>,
    rendering_context: Rc<RefCell<RenderingContext>>,
}

pub struct PipelineStore {
    pipeline_store: Store<PipelineContext, InternalRenderPipeline>,
}

impl PipelineStore {
    pub fn new(context: Rc<Context>, rendering_context: Rc<RefCell<RenderingContext>>) -> Self {
        Self {
            pipeline_store: Store::new(PipelineContext {
                context,
                rendering_context,
            }),
        }
    }

    pub fn get_entry<L>(
        &mut self,
        label: &L,
        render_target_description: &RenderTargetDescription,
    ) -> Pipeline
    where
        L: PipelineDescriptable + EntryLabel,
    {
        let create_pipeline = |pipeline_context: &mut PipelineContext| {
            let rendering_context = pipeline_context.rendering_context.borrow();
            let parameters = label.pipeline_description(&rendering_context);
            let mut pipeline_layout_descriptor_name = parameters.name.clone();
            pipeline_layout_descriptor_name.push_str("layout descriptor");

            let as_ref: Vec<&BindGroupLayout> = parameters
                .bind_group_layouts
                .iter()
                .map(|kwa| kwa.as_ref())
                .collect();

            let pipeline_layout = pipeline_context.context.device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some(pipeline_layout_descriptor_name.as_str()),
                    bind_group_layouts: &as_ref,
                    push_constant_ranges: &[],
                },
            );

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
            let shader = parameters.shader.clone();
            (
                InternalRenderPipeline {
                    pipeline: pipeline_context.context.device.create_render_pipeline(
                        &wgpu::RenderPipelineDescriptor {
                            label: Some(&parameters.name),
                            layout: Some(&pipeline_layout),
                            vertex: wgpu::VertexState {
                                module: parameters.shader.deref(),
                                entry_point: "vs_main",
                                buffers: &parameters.vertex_buffer_layouts,
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: parameters.shader.deref(),
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
                        },
                    ),
                    render_target_description: render_target_description.clone(),
                    parameters,
                },
                shader,
            )
        };

        let after = |_context: &mut PipelineContext, entry: &Pipeline, metadata: Shader| {
            metadata.register_dep(Box::new(entry.downgrade()));
        };

        self.pipeline_store.get_entry(label, create_pipeline, after)
    }
}
