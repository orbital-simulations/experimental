use std::{borrow::Cow, env, io::Read, path::PathBuf};

use slotmap::{new_key_type, SecondaryMap, SlotMap};
use wgpu::ShaderModuleDescriptor;

use crate::gpu_context::GpuContext;
use super::reload_command::RebuildCommand;
use crate::file_watcher::FileWatcher;

new_key_type! {
    pub struct ShaderId;
}

pub struct ShaderStore {
    store: SlotMap<ShaderId, wgpu::ShaderModule>,
    shader_sources: SecondaryMap<ShaderId, ShaderSource>,
    dependants: SecondaryMap<ShaderId, Vec<RebuildCommand>>,
    gpu_context: GpuContext,
}

#[derive(Clone, Debug)]
pub enum ShaderSource {
    ShaderFile(PathBuf),
    StaticFile(wgpu::ShaderModuleDescriptor<'static>),
}

impl ShaderStore {
    pub fn new(gpu_context: &GpuContext) -> Self {
        Self {
            store: SlotMap::with_key(),
            gpu_context: gpu_context.clone(),
            shader_sources: SecondaryMap::new(),
            dependants: SecondaryMap::new(),
        }
    }

    pub fn build_shader(
        &mut self,
        file_watcher: &mut FileWatcher,
        shader_source: &ShaderSource,
    ) -> ShaderId {
        let (shader_module, file_path) = self.build(shader_source);
        let shader_id = self.store.insert(shader_module);
        self.shader_sources.insert(shader_id, shader_source.clone());
        self.dependants.insert(shader_id, Vec::new());
        if let Some(file_path) = file_path {
            file_watcher.watch_file(file_path, RebuildCommand::Shader(shader_id));
        }
        shader_id
    }

    fn build(&self, shader_source: &ShaderSource) -> (wgpu::ShaderModule, Option<PathBuf>) {
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
                (shader_module, Some(file_path))
            }
            ShaderSource::StaticFile(shader_module_descriptor) => {
                let shader_module = self
                    .gpu_context
                    .device()
                    .create_shader_module(shader_module_descriptor.clone());
                (shader_module, None)
            }
        }
    }

    pub fn get_shader(&self, shader_id: ShaderId) -> &wgpu::ShaderModule {
        &self.store[shader_id]
    }

    pub fn rebuild(&mut self, shader_id: ShaderId) -> Vec<RebuildCommand> {
        let shader_source = &self.shader_sources[shader_id];
        let (shader_module, _) = self.build(shader_source);
        self.store[shader_id] = shader_module;
        self.dependants[shader_id].clone()
    }

    pub fn register_dependant(&mut self, shader_id: ShaderId, reload_command: RebuildCommand) {
        self.dependants[shader_id].push(reload_command);
    }
}
