use std::{borrow::Cow, env, io::Read, path::PathBuf};

use slotmap::{new_key_type, SlotMap};
use wgpu::ShaderModuleDescriptor;

use crate::gpu_context::GpuContext;

new_key_type! {
    pub struct ShaderId;
}

pub struct ShaderStore {
    store: SlotMap<ShaderId, wgpu::ShaderModule>,
    gpu_context: GpuContext,
}

pub enum ShaderSource {
    ShaderFile(PathBuf),
    StaticFile(wgpu::ShaderModuleDescriptor<'static>),
}

impl ShaderStore {
    pub fn new(gpu_context: &GpuContext) -> Self {
        Self {
            store: SlotMap::with_key(),
            gpu_context: gpu_context.clone(),
        }
    }

    pub fn build_shader(&mut self, shader_source: &ShaderSource) -> ShaderId {
        match shader_source {
            ShaderSource::ShaderFile(file_path) => {
                // TODO: In future. We should start using some kind of an asset loader so we can
                // cross compile to web.
                // TODO: In future. There probably should be some
                // configuration for directories where to look for shaders.
                let pwd = env::current_dir()
                    .unwrap_or_else(|_| panic!("can't get current working directory"));
                let file_path = pwd.join(file_path);
                let mut source_file = std::fs::File::open(&file_path).unwrap_or_else(|_| {
                    panic!(
                        "missing shader file: {}",
                        file_path.as_os_str().to_str().unwrap()
                    )
                });
                let mut source = String::new();
                // TODO: Maybe this should just make the shader not work instead of terminating the
                // app.
                source_file.read_to_string(&mut source).unwrap_or_else(|_| {
                    panic!(
                        "Can't read the shader: {}",
                        file_path.as_os_str().to_str().unwrap()
                    )
                });
                let shader_module =
                    self.gpu_context
                        .device()
                        .create_shader_module(ShaderModuleDescriptor {
                            label: Some(file_path.as_os_str().to_str().unwrap()),
                            source: wgpu::ShaderSource::Wgsl(Cow::Owned(source)),
                        });
                self.store.insert(shader_module)
            }
            ShaderSource::StaticFile(shader_module_descriptor) => {
                let shader_module = self
                    .gpu_context
                    .device()
                    .create_shader_module(shader_module_descriptor.clone());
                self.store.insert(shader_module)
            }
        }
    }

    pub fn get_shader(&self, shader_id: ShaderId) -> &wgpu::ShaderModule {
        // SAFETY: This works fine because we don't remove element and when we start removing them
        // it will be done in a way that doesn't leave keys (ids) dangling.
        unsafe {
            self.store.get_unchecked(shader_id)
        }
    }
}
