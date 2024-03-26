use std::cell::RefCell;
use std::env;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::{borrow::Cow, io::Read};

use wgpu::{ShaderModule, ShaderModuleDescriptor, ShaderSource};

use crate::context::Context;
use crate::resource_watcher::{ResourceWatcher, UpdateCommand, WatchedResource};
use crate::store::{self, EntryLabel, EntryRebuilder, Store, WeakEntry};

#[derive(Debug, Clone)]
pub enum ShaderCreator {
    ShaderFromFile(String),
    ShaderStatic(ShaderModuleDescriptor<'static>),
}

pub trait ShaderDescriptable {
    fn shader_description(&self) -> ShaderCreator;
}

pub type Shader = store::Entry<ShaderStoreContext, InternalShaderModule>;

pub struct InternalShaderModule {
    shader_module: ShaderModule,
    shader_path: PathBuf,
}

impl Deref for InternalShaderModule {
    type Target = ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.shader_module
    }
}

pub struct ShaderStoreContext {
    context: Rc<Context>,
    resource_watcher: Rc<RefCell<ResourceWatcher>>,
}

pub struct ShaderStore {
    shader_store: Store<ShaderStoreContext, InternalShaderModule>,
}

impl EntryRebuilder<ShaderStoreContext> for InternalShaderModule {
    fn rebuild(&self, context: &ShaderStoreContext) -> Self {
        let file_path = &self.shader_path;
        let mut source_file = std::fs::File::open(file_path).unwrap_or_else(|_| {
            panic!(
                "missing shader file: {}",
                file_path.as_os_str().to_str().unwrap()
            )
        });
        let mut source = String::new();
        // TODO: Maybe this should just make the shader not work instead of termnating the
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
        InternalShaderModule {
            shader_module: context
                .context
                .device
                .create_shader_module(shader_description),
            shader_path: file_path.clone(),
        }
    }
}

impl WatchedResource for WeakEntry<ShaderStoreContext, InternalShaderModule> {
    fn update(&mut self) -> UpdateCommand {
        if let Some(strong_ref) = self.upgrade() {
            strong_ref.rebuild();
            UpdateCommand::Keep
        } else {
            UpdateCommand::Remove
        }
    }
}

impl ShaderStore {
    pub fn new(context: Rc<Context>, resource_watcher: Rc<RefCell<ResourceWatcher>>) -> Self {
        Self {
            shader_store: Store::new(
                ShaderStoreContext {
                    context,
                    resource_watcher,
                },
                "shader store".to_string(),
            ),
        }
    }

    pub fn get_entry<L>(&mut self, label: &L) -> Shader
    where
        L: ShaderDescriptable + EntryLabel,
    {
        let construct_entry = |context: &mut ShaderStoreContext| {
            let description = label.shader_description();
            match description {
                ShaderCreator::ShaderFromFile(file_name) => {
                    // TODO: Maybe this should just make the shader not work instead of termnating the
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
                    // TODO: Maybe this should just make the shader not work instead of termnating the
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
                    (
                        InternalShaderModule {
                            shader_module: context
                                .context
                                .device
                                .create_shader_module(shader_description),
                            shader_path: file_path.clone(),
                        },
                        Some(file_path),
                    )
                }
                ShaderCreator::ShaderStatic(module) => (
                    InternalShaderModule {
                        shader_module: context.context.device.create_shader_module(module),
                        shader_path: PathBuf::from_str("").unwrap(),
                    },
                    None,
                ),
            }
        };

        let after =
            |context: &mut ShaderStoreContext, entry: &Shader, metadata: Option<PathBuf>| {
                if let Some(file_path) = metadata {
                    context
                        .resource_watcher
                        .borrow_mut()
                        .watch_file(&file_path, Box::new(entry.downgrade()));
                }
            };

        self.shader_store.get_entry(label, construct_entry, after)
    }
}
