use std::{any::{TypeId, Any}, collections::HashMap, ops::Deref, rc::{Rc, Weak}, io::Read, borrow::Cow};

use wgpu::{ShaderModuleDescriptor, ShaderSource, ShaderModule};

use crate::context::Context;

pub struct Shader {
    // TODO: This will need to be Arc in future
    shader_ref: Rc<ShaderStoreEntryWrapper>,
}

#[derive(Debug, Clone)]
pub enum ShaderCreator {
    ShaderFromFile(String),
    ShaderStatic(ShaderModuleDescriptor<'static>),
}

pub trait ShaderDescriptable {
    fn shader_description() -> ShaderCreator;
}

struct ShaderStoreEntryWrapper {
    type_label: TypeId,
    shader: ShaderModule,
}

#[derive(Debug)]
pub struct ShaderStore {
    // TODO: Later this will need to be something like Arc<TwLock<HashMap<..>>>
    // to prevent race conditions.
    shaders: HashMap<TypeId, Weak<ShaderStoreEntryWrapper>>,
}

impl Default for ShaderStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ShaderStore {
    pub fn new() -> Self {
        Self{
            shaders: HashMap::new(),
        }
    }

    fn create_shader<T>(context: &Context, shader_label: &T) -> (Rc<ShaderStoreEntryWrapper>, Weak<ShaderStoreEntryWrapper>) where T: ShaderDescriptable + Any {
        let shader = match T::shader_description() {
            ShaderCreator::ShaderFromFile(file_name) => {
                // TODO: Maybe this should just make the shader not work instead of termnating the
                // app.
                let mut source_file = std::fs::File::open(&file_name).unwrap_or_else(|_| panic!("missing shader file: {}", file_name));
                let mut source = String::new();
                // TODO: Maybe this should just make the shader not work instead of termnating the
                // app.
                source_file.read_to_string(&mut source).unwrap_or_else(|_| panic!("Can't read the shader: {}", file_name));
                let shader_description = ShaderModuleDescriptor {
                    label: Some(&file_name),
                    source: ShaderSource::Wgsl(Cow::Owned(source)),
                };
                context.device.create_shader_module(shader_description)
            },
            ShaderCreator::ShaderStatic(module) => {
                context.device.create_shader_module(module)
            },
        };

        let strong_shader = Rc::new(ShaderStoreEntryWrapper {
            shader,
            type_label: shader_label.type_id(),
        });
        let weak_shader = Rc::downgrade(&strong_shader);
        (strong_shader, weak_shader)
    }

    pub fn get_shader<T>(&mut self, context: &Context, shader_label: T)  -> Shader where T: ShaderDescriptable + Any {
        let possible_shader = self.shaders.get(&shader_label.type_id());
        match possible_shader.as_ref() {
            Some(weak_shader) => {
                match weak_shader.upgrade() {
                    None => {
                        let (strong_shader, weak_shader) = ShaderStore::create_shader(context, &shader_label);
                        self.shaders.insert(shader_label.type_id(), weak_shader);
                        Shader { shader_ref: strong_shader }
                    },
                    Some(strong_shader) =>
                        Shader { shader_ref: strong_shader }
,
                }
            },
            None => {
                        let (strong_shader, weak_shader) = ShaderStore::create_shader(context, &shader_label);
                        self.shaders.insert(shader_label.type_id(), weak_shader);
                        Shader { shader_ref: strong_shader }
            },
        }
    }
}

impl Deref for Shader {
    type Target = ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.shader_ref
    }
}

impl Deref for ShaderStoreEntryWrapper {
    type Target = ShaderModule;

    fn deref(&self) -> &Self::Target {
        &self.shader
    }
}
