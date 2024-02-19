use std::{any::TypeId, collections::HashMap, ops::Deref, rc::{Rc, Weak}};

use wgpu::ShaderModel;

use crate::context::Context;


#[derive(Debug)]
struct ShaderStore {
    shaders: HashMap<TypeId, Weak<ShaderStoreEntryWrapper>>,
}

struct Shader {
    shader_ref: Rc<ShaderStoreEntryWrapper>,
}

trait DescriptableShader {

}

impl ShaderStore {
    fn new() -> Self {
        Self{
            shaders: HashMap::new(),
        }
    }

    fn get_shader(&mut self, context: &Context, ) -> Shader {
        self.shaders.entry(key)
    }
}

struct ShaderStoreEntryWrapper {
    type_id: TypeId,
    shader: ShaderModel,
}

impl Deref for ShaderStoreEntryWrapper {
    type Target = ShaderModel;

    fn deref(&self) -> &Self::Target {
        &self.shader
    }
}
