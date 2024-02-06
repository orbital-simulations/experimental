use wgpu::{
    BindGroupLayout, PipelineLayout, RenderPipeline, ShaderModule, VertexBufferLayout,
    VertexStepMode,
};

use crate::{
    buffers::{BindGroup, DescriptiveBuffer, IndexFormatTrait},
    context::Context,
};

struct CreatePipeline<'a> {
    shader: ShaderModule,
    instance_buffer: Option<(u32, &'a dyn DescriptiveBuffer)>,
    vertex_buffers: &'a [(u32, &'a dyn DescriptiveBuffer)],
    index_buffer: &'a dyn IndexFormatTrait,
    bind_groups: &'a [(u32, BindGroup)],
    name: String,
}

struct Pipeline {
    pipeline_layout: PipelineLayout,
    name: String,
    vertex_buffer_layouts: Vec<VertexBufferLayout<'static>>,
    pipeline: RenderPipeline,
    shader: ShaderModule,
}

impl Pipeline {
    pub fn new<'a>(
        context: &Context,
        parameters: &CreatePipeline<'a>,
    ) -> Self {
        let mut pipeline_layout_descriptor_name = parameters.name;
        pipeline_layout_descriptor_name.push_str("layout descriptor");

        let bind_group_layouts: Vec<&BindGroupLayout> = parameters
            .bind_groups
            .iter()
            .map(|b| b.1.layout())
            .collect();

        let pipeline_layout =
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(pipeline_layout_descriptor_name.as_str()),
                    bind_group_layouts: &bind_group_layouts,
                    push_constant_ranges: &[],
                });
        let mut vertex_buffer_layouts: Vec<VertexBufferLayout<'static>> = parameters
            .vertex_buffers
            .iter()
            .map(|(binding, buffer)| buffer.describe_vertex_buffer(VertexStepMode::Vertex))
            .collect();
        if let Some(instance_buffer) = parameters.instance_buffer {
            vertex_buffer_layouts.push(
                instance_buffer
                    .1
                    .describe_vertex_buffer(VertexStepMode::Instance),
            );
        }
        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&parameters.name),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &parameters.shader,
                    entry_point: "vs_main",
                    buffers: &vertex_buffer_layouts,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &parameters.shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: context.output_texture_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
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
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });
        Self {
            name: parameters.name,
            pipeline,
            shader: parameters.shader,
            pipeline_layout,
            vertex_buffer_layouts,
        }
    }
}
