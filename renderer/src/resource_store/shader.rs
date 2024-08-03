use std::collections::HashMap;
use std::io;
use std::{borrow::Cow, env, io::Read, path::PathBuf};

use naga_oil::compose::{
    get_preprocessor_data, ComposableModuleDescriptor, Composer, ComposerError, ImportDefinition,
    NagaModuleDescriptor, ShaderLanguage, ShaderType,
};
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use thiserror::Error;
use wgpu::ShaderModuleDescriptor;

use super::reload_command::RebuildCommand;
use crate::file_watcher::FileWatcher;
use crate::gpu_context::GpuContext;

new_key_type! {
    pub struct ShaderId;
}

const DEFAULT_SHADER_LIB: &[&str] = &[include_str!("../../shaders/lib/model_matrix.wgsl")];

pub struct ShaderStore {
    store: SlotMap<ShaderId, wgpu::ShaderModule>,
    shader_sources: SecondaryMap<ShaderId, ShaderSource>,
    dependants: SecondaryMap<ShaderId, Vec<RebuildCommand>>,
    gpu_context: GpuContext,
    naga_oil_composer: Composer,
}

#[derive(Clone)]
pub struct StaticShaderFile {
    pub source: &'static str,
    pub file_path: &'static str,
}

#[derive(Clone)]
pub enum ShaderSource {
    ShaderFile(PathBuf),
    StaticFile(StaticShaderFile),
}

#[derive(Error, Debug)]
pub enum BuildShaderError {
    #[error("current working directory")]
    CurrentWorkingDirectory(#[from] io::Error),
    #[error("can't read shader file \"{file}\" because of internal error: {source}")]
    CantReadShaderFile { file: PathBuf, source: io::Error },
    #[error("shader file \"{file}\" hash incorent utf8 format")]
    NotValidUtf8 { file: PathBuf },
    #[error("naga oil composer filed for shader file \"{file}\" with error: {source}")]
    NagaComposerFailed {
        file: PathBuf,
        source: ComposerError,
    },
}

#[derive(Error, Debug)]
pub enum InitializationError {
    #[error("Oil composer error")]
    ComposerError(#[from] ComposerError),
    #[error("Imported module `{module_name}` not found. Import located in module `{caller}`")]
    ImportError { module_name: String, caller: String },
    #[error("Missing `#define_import_path` in a lib shader shader_str: `{shader_str}`")]
    MissingDefinitionOfImportPath { shader_str: String },
}

impl ShaderStore {
    pub fn new(gpu_context: &GpuContext) -> Result<Self, InitializationError> {
        let mut naga_oil_composer = Composer::default();
        Self::load_shader_lib(&mut naga_oil_composer, DEFAULT_SHADER_LIB)?;

        Ok(Self {
            store: SlotMap::with_key(),
            gpu_context: gpu_context.clone(),
            shader_sources: SecondaryMap::new(),
            dependants: SecondaryMap::new(),
            naga_oil_composer,
        })
    }

    fn load_shader_lib(
        naga_oil_composer: &mut Composer,
        shader_lib: &[&str],
    ) -> Result<(), InitializationError> {
        let mut default_shaders: HashMap<String, (&str, Vec<ImportDefinition>)> = HashMap::new();
        for shader_str in shader_lib {
            let (module_name, imports, _) = get_preprocessor_data(shader_str);
            if let Some(module_name) = module_name {
                default_shaders.insert(module_name, (shader_str, imports));
            } else {
                return Err(InitializationError::MissingDefinitionOfImportPath {
                    shader_str: shader_str.to_string(),
                });
            }
        }

        let mut stack: Vec<(&str, &str)> = Vec::new();

        for (module_name, _) in default_shaders.iter() {
            if naga_oil_composer.contains_module(module_name.as_str()) {
                continue;
            }
            stack.push(("lib_root", module_name.as_str()));

            while let Some((caller, module_name)) = stack.pop() {
                if let Some((module_source, imports)) = default_shaders.get(module_name) {
                    let mut missing_imports: Vec<&str> = Vec::new();
                    for import in imports.iter().map(|v| v.import.as_str()) {
                        if !naga_oil_composer.contains_module(import) {
                            missing_imports.push(import);
                        }
                    }

                    if missing_imports.is_empty() {
                        naga_oil_composer.add_composable_module(ComposableModuleDescriptor {
                            source: module_source,
                            file_path: format!("build-in-lib/{module_name}").as_str(),
                            language: ShaderLanguage::Wgsl,
                            as_name: None,
                            additional_imports: &[],
                            shader_defs: HashMap::new(),
                        })?;
                    } else {
                        stack.push((caller, module_name));
                        stack.append(
                            &mut missing_imports.iter().map(|v| (module_name, *v)).collect(),
                        );
                    }
                } else {
                    return Err(InitializationError::ImportError {
                        module_name: module_name.into(),
                        caller: caller.into(),
                    });
                }
            }
        }
        Ok(())
    }

    pub fn build_shader(
        &mut self,
        file_watcher: &mut FileWatcher,
        shader_source: &ShaderSource,
    ) -> Result<ShaderId, BuildShaderError> {
        let (shader_module, file_path) = self.build(shader_source)?;
        let shader_id = self.store.insert(shader_module);
        self.shader_sources.insert(shader_id, shader_source.clone());
        self.dependants.insert(shader_id, Vec::new());
        if let Some(file_path) = file_path {
            file_watcher.watch_file(file_path, RebuildCommand::Shader(shader_id));
        }
        Ok(shader_id)
    }

    fn build(
        &mut self,
        shader_source: &ShaderSource,
    ) -> Result<(wgpu::ShaderModule, Option<PathBuf>), BuildShaderError> {
        use BuildShaderError::*;
        match shader_source {
            ShaderSource::ShaderFile(file_path) => {
                // TODO: In future. We should start using some kind of an asset loader so we can
                // cross compile to web.
                // TODO: In future. There probably should be some
                // configuration for directories where to look for shaders.
                let pwd = env::current_dir().map_err(CurrentWorkingDirectory)?;
                let file_path = pwd.join(file_path);
                let mut source_file =
                    std::fs::File::open(&file_path).map_err(|err| CantReadShaderFile {
                        file: file_path.clone(),
                        source: err,
                    })?;
                let mut source = String::new();
                // TODO: Maybe this should just make the shader not work instead of terminating the
                // app.
                source_file
                    .read_to_string(&mut source)
                    .map_err(|err| CantReadShaderFile {
                        file: file_path.clone(),
                        source: err,
                    })?;

                let naga_module = self
                    .naga_oil_composer
                    .make_naga_module(NagaModuleDescriptor {
                        source: source.as_str(),
                        file_path: file_path.as_os_str().to_str().ok_or_else(|| NotValidUtf8 {
                            file: file_path.clone(),
                        })?,
                        shader_type: ShaderType::Wgsl,
                        shader_defs: HashMap::new(),
                        additional_imports: &[],
                    })
                    .map_err(|err| NagaComposerFailed {
                        file: file_path.clone(),
                        source: err,
                    })?;

                let shader_module =
                    self.gpu_context
                        .device()
                        .create_shader_module(ShaderModuleDescriptor {
                            // TODO: Can this unwrap even fail?
                            label: Some(file_path.as_os_str().to_str().unwrap()),
                            source: wgpu::ShaderSource::Naga(Cow::Owned(naga_module)),
                        });
                Ok((shader_module, Some(file_path)))
            }
            ShaderSource::StaticFile(static_file) => {
                let naga_module = self
                    .naga_oil_composer
                    .make_naga_module(NagaModuleDescriptor {
                        source: static_file.source,
                        file_path: static_file.file_path,
                        shader_type: ShaderType::Wgsl,
                        shader_defs: HashMap::new(),
                        additional_imports: &[],
                    })
                    .map_err(|err| NagaComposerFailed {
                        file: static_file.file_path.into(),
                        source: err,
                    })?;

                let shader_module =
                    self.gpu_context
                        .device()
                        .create_shader_module(ShaderModuleDescriptor {
                            label: Some(static_file.file_path),
                            source: wgpu::ShaderSource::Naga(Cow::Owned(naga_module)),
                        });
                Ok((shader_module, None))
            }
        }
    }

    pub fn get_shader(&self, shader_id: ShaderId) -> &wgpu::ShaderModule {
        &self.store[shader_id]
    }

    pub fn rebuild(
        &mut self,
        shader_id: ShaderId,
    ) -> Result<Vec<RebuildCommand>, BuildShaderError> {
        let shader_source = self.shader_sources[shader_id].clone();
        let (shader_module, _) = self.build(&shader_source)?;
        self.store[shader_id] = shader_module;
        Ok(self.dependants[shader_id].clone())
    }

    pub fn register_dependant(&mut self, shader_id: ShaderId, reload_command: RebuildCommand) {
        self.dependants[shader_id].push(reload_command);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_missing_module_name() {
        use super::*;
        let mut naga_oil_composer = Composer::default();
        let test_shaders = ["fn test() {}"];
        let ret = ShaderStore::load_shader_lib(&mut naga_oil_composer, &test_shaders);
        assert!(ret.is_err());
        assert_eq!(
            ret.unwrap_err().to_string(),
            "Missing `#define_import_path` in a lib shader shader_str: `fn test() {}`"
        );
    }

    #[test]
    fn test_missing_import() {
        use super::*;
        let mut naga_oil_composer = Composer::default();
        let test_shaders = ["
            #define_import_path test_module
            fn test() { missing_module::foo() }"];
        let ret = ShaderStore::load_shader_lib(&mut naga_oil_composer, &test_shaders);
        assert!(ret.is_err());
        assert_eq!(
            ret.unwrap_err().to_string(),
            "Imported module `missing_module` not found. Import located in module `test_module`"
        );
    }

    #[test]
    fn test_existing_module_import() {
        use super::*;
        let mut naga_oil_composer = Composer::default();
        let test_shaders = [
            "#define_import_path test_module
             fn test() { existing_module::foo() }",
            "#define_import_path existing_module
             fn foo() {}",
        ];
        let ret = ShaderStore::load_shader_lib(&mut naga_oil_composer, &test_shaders);
        assert!(ret.is_ok());
    }
}
