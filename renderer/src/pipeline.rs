use std::{
    cell::{Ref, RefCell}, fs::File, io::Read, rc::Rc
};

use wgpu::{ShaderModuleDescriptor, TextureFormat};

use crate::{
    context::Context,
    resource_watcher::{Reloadable, ResourceWatcher},
    web_gpu::{RenderPipelineDescription, VertexBufferLayout},
};

#[derive(Debug, Clone)]
pub struct RenderTargetDescription {
    pub multisampling: u32,
    pub depth_texture: Option<TextureFormat>,
    pub targets: Vec<TextureFormat>,
}

#[derive(Debug)]
struct PipelineInner {
    #[allow(dead_code)]
    name: String,
    pipeline: RefCell<wgpu::RenderPipeline>,
    pipeline_description: RenderPipelineDescription,
}

impl PipelineInner {
    fn new(context: &Context, render_pipeline_description: &RenderPipelineDescription) -> Self {
        let pipeline = Self::build_pipeline(context, render_pipeline_description);
        Self {
            name: render_pipeline_description.label.clone(),
            pipeline: RefCell::new(pipeline),
            pipeline_description: render_pipeline_description.clone(),
        }
    }

    fn build_pipeline(
        context: &Context,
        render_pipeline_description: &RenderPipelineDescription,
    ) -> wgpu::RenderPipeline {
        println!("building pipeline!");
        let (shader_code, label) = match &render_pipeline_description.shader {
            crate::web_gpu::Shader::CompiledIn(module, label) => (module.clone(), label.clone()),
            crate::web_gpu::Shader::Path(path) => {
                let mut file = File::open(path.clone()).unwrap(); // FIXME: This unwrap...
                let mut shader_code = String::new();
                file.read_to_string(&mut shader_code).unwrap();
                (shader_code, path.clone().to_str().unwrap().into()) // FIXME: This unwrap...
            }
        };
        let shader_module = context.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&label),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });
        let fragment = render_pipeline_description
            .fragment
            .as_ref()
            .map(|fragment_description| wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &fragment_description.targets,
            });
        let vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout> = render_pipeline_description
            .vertex
            .buffers
            .iter()
            .map(
                |vertex_buffer_layout: &VertexBufferLayout| wgpu::VertexBufferLayout {
                    array_stride: vertex_buffer_layout.array_stride,
                    step_mode: vertex_buffer_layout.step_mode,
                    attributes: &vertex_buffer_layout.attributes,
                },
            )
            .collect();

        let vertex = wgpu::VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &vertex_buffer_layouts,
        };

        let pipeline_layout = render_pipeline_description.layout.as_ref().map(|layout| {
            let mut pipeline_layout_descriptor_name = render_pipeline_description.label.clone();
            pipeline_layout_descriptor_name.push_str("layout descriptor");
            let bind_group_layouts: Vec<wgpu::BindGroupLayout> = layout
                .bind_group_layouts
                .iter()
                .map(|bind_group_layout| {
                    context
                        .device
                        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                            label: Some(&bind_group_layout.label),
                            entries: &bind_group_layout.entries,
                        })
                })
                .collect();
            let mut pipeline_layout_descriptor_name = render_pipeline_description.label.clone();
            pipeline_layout_descriptor_name.push_str("layout descriptor");
            context
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(pipeline_layout_descriptor_name.as_str()),
                    bind_group_layouts: &bind_group_layouts
                        .iter()
                        .collect::<Vec<&wgpu::BindGroupLayout>>(),
                    push_constant_ranges: &[],
                })
        });

        context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&render_pipeline_description.label),
                layout: pipeline_layout.as_ref(),
                vertex,
                fragment,
                primitive: render_pipeline_description.primitive,
                depth_stencil: render_pipeline_description.depth_stencil.clone(),
                multisample: render_pipeline_description.multisample,
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: render_pipeline_description.multiview,
            })
    }

    fn rebuild(&self, context: &Context) {
        self.pipeline.replace(PipelineInner::build_pipeline(context, &self.pipeline_description));
    }

    fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        unsafe {
            &*self.pipeline.as_ptr()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pipeline {
    pipeline: Rc<PipelineInner>,
}

impl Pipeline {
    pub fn new(
        context: &Context,
        render_pipeline_description: &RenderPipelineDescription,
        resource_watcher: &mut ResourceWatcher,
    ) -> Self {
        let pipeline = Pipeline {
            pipeline: Rc::new(PipelineInner::new(context, render_pipeline_description)),
        };
        if let crate::web_gpu::Shader::Path(path) = &render_pipeline_description.shader {
            resource_watcher.watch_resource(path, Box::new(pipeline.clone()))
        }
        pipeline
    }
    pub fn render_pipeline(&self) -> &wgpu::RenderPipeline {
        self.pipeline.render_pipeline()
    }
}

impl Reloadable for Pipeline {
    fn reload(&mut self, context: &Context) {
        println!("asdfasdf");
        self.pipeline.rebuild(context)
    }
}
