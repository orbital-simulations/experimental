use std::env;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::{borrow::Cow, io::Read};

use wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource};

use crate::context::Context;
use crate::resource_watcher::{ResourceChangeObserver, ResourceWatcher};
use crate::store::{FatStoreID, RebuildableEntry, StorableResource, Store, StoreID, UnlockedStore};

pub struct ShaderStoreContext {
    pub gpu_context: Arc<Context>,
    pub resource_watcher: Arc<Mutex<ResourceWatcher>>
}

pub type ShaderID = StoreID<ShaderModule>;
pub type FatShaderID = FatStoreID<ShaderModule>;
pub type ShaderStore = Store<ShaderModule>;
pub type UnlockedShaderStore<'a> = UnlockedStore<'a, ShaderModule>;

#[derive(Clone, Debug)]
pub struct ShaderStatic {
    pub unique_shader_name: String,
    pub static_shader_module_descriptor: ShaderModuleDescriptor<'static>,
}

#[derive(Clone, Debug)]
pub enum ShaderDescription {
    ShaderFromFile(String),
    ShaderStatic(ShaderStatic),
}

impl PartialEq for ShaderDescription {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ShaderFromFile(a), Self::ShaderFromFile(b)) => a == b,
            (Self::ShaderStatic(a), Self::ShaderStatic(b)) => {
                a.unique_shader_name == b.unique_shader_name
            }
            _ => false,
        }
    }
}

impl Eq for ShaderDescription {}

impl Hash for ShaderDescription {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::ShaderFromFile(file_name) => {
                file_name.hash(state);
                false.hash(state)
            }
            Self::ShaderStatic(a) => {
                a.unique_shader_name.hash(state);
                true.hash(state)
            }
        }
    }
}

impl StorableResource for ShaderModule {
    type Context = ShaderStoreContext;
    type Description = ShaderDescription;

    fn build(context: &ShaderStoreContext, description: &ShaderDescription) -> ShaderModule {
        match description {
            ShaderDescription::ShaderFromFile(file_name) => {
                // TODO: Maybe this should just make the shader not work instead of terminating the
                // app.

                // TODO: In future. There probably should be some
                // configuration for directories where to look for shaders.
                let pwd = env::current_dir()
                    .unwrap_or_else(|_| panic!("can't get current working directory"));
                let file_path = pwd.join(file_name);
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
                let shader_description = ShaderModuleDescriptor {
                    label: Some(file_path.as_os_str().to_str().unwrap()),
                    source: ShaderSource::Wgsl(Cow::Owned(source)),
                };
                context
                    .gpu_context
                    .device
                    .create_shader_module(shader_description)
            }
            ShaderDescription::ShaderStatic(static_module) => context
                .gpu_context
                .device
                .create_shader_module(static_module.static_shader_module_descriptor.clone()),
        }
    }

    fn register_dependences(
        context: &ShaderStoreContext,
        description: &ShaderDescription,
        fat_id: FatStoreID<ShaderModule>,
    ) {
        match description {
            ShaderDescription::ShaderFromFile(file_name) => {
                let pwd = env::current_dir()
                    .unwrap_or_else(|_| panic!("can't get current working directory"));
                let file_path = pwd.join(file_name).canonicalize().unwrap();
                context.resource_watcher.lock().unwrap().watch_file(file_path, Box::new(fat_id))
            },
            ShaderDescription::ShaderStatic(_) => {},
        }

    }
}

impl ResourceChangeObserver for FatShaderID {
    fn file_changed(&self) {
        self.rebuild()
    }
}
