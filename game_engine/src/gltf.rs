use std::path::{Path, PathBuf};

use glam::{Quat, Vec3};
use gltf::{buffer::Data, mesh::Mode, Primitive};
use renderer::{include_wgsl, mesh_rendering::MeshBundle, scene_node::SceneNode, transform::Transform, Renderer};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoadGltfError {
    #[error("Error during the import of the gltf file [{file}]: {error}")]
    GltfError{file: PathBuf, error: gltf::Error},
}


fn primitive_to_scene_node(renderer: &mut Renderer, transform: &Transform, primitive: &Primitive, buffers: &[Data]) -> eyre::Result<SceneNode> {
    let default_shader = include_wgsl!("test_shader/default_mesh.wgsl");
    let pipeline_id = renderer.create_3d_pipeline(&default_shader)?;

    if primitive.mode() != Mode::Triangles {
        panic!("Only supported primitive mode is `Mode::Triangles`, but {:?} was found", primitive.mode());
    }
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
    let indices: Vec<u32> = reader.read_indices().map(|iter| iter.into_u32().collect()).expect("indices missing in the model");

    let vertices: Vec<Vec3> = reader.read_positions().map(|iter| {
        iter.map(|v| v.into()).collect()
    }).expect("vertices missing in the model");

    let normals: Vec<Vec3> = reader.read_normals().map(|iter| {
        iter.map(|v| v.into()).collect()
    }).expect("normals missing in the model");

    let mesh_id = renderer.add_mesh(&vertices, &normals, &indices);
    let mesh_bundle = MeshBundle{pipeline_id, mesh_id};
    Ok(SceneNode::from_mesh_bundle(*transform, mesh_bundle))
}

fn to_scene_node(renderer: &mut Renderer, gltf_node: &gltf::Node, buffers: &[Data]) -> eyre::Result<SceneNode> {
    let transform = match gltf_node.transform() {
        gltf::scene::Transform::Matrix { matrix } => {
            Transform::from_columns(&matrix)
        },
        gltf::scene::Transform::Decomposed { translation, rotation, scale } => {
            Transform::from_translation_rotation_scale(&Vec3::from_array(translation), &Quat::from_array(rotation), scale[0])
        },
    };
    let mut child_nodes: Vec<SceneNode> = gltf_node.children().flat_map(|gltf_node| to_scene_node(renderer, &gltf_node, buffers)).collect();
    match gltf_node.mesh() {
        Some(gltf_mesh) => {
            match gltf_mesh.primitives().len() {
                0 => {
                    panic!("Number of meshe primitives in a the mesh is 0...");
                },
                1 => {
                    primitive_to_scene_node(renderer, &transform, &gltf_mesh.primitives().next().unwrap(), buffers)
                },
                _ => {
                    for primitive in gltf_mesh.primitives() {
                        let primitive = primitive_to_scene_node(renderer, &transform, &primitive, buffers)?;
                        child_nodes.push(primitive);
                    }
                    Ok(SceneNode::invisible(transform, child_nodes))
                },
            }
        },
        None => {
            Ok(SceneNode::invisible(transform, child_nodes))
        },
    }
}

pub fn load_gltf<P>(renderer: &mut Renderer, file: P) -> eyre::Result<Vec<SceneNode>>
    where P: AsRef<Path>
{
    let (document, buffers, images) = gltf::import(file.as_ref()).map_err(|e | LoadGltfError::GltfError { file: file.as_ref().to_path_buf(), error: e })?;
    let mut meshes: Vec<SceneNode> = Vec::new();
    for scene in document.scenes() {
        for gltf_node in scene.nodes() {
            meshes.push(to_scene_node(renderer, &gltf_node, &buffers)?);
        }
    }
    Ok(meshes)
}
