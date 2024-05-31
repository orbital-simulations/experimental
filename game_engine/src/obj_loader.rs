use eyre::Result;
use glam::Vec3;
use itertools::Itertools;
use renderer::{resource_store::GpuMeshId, Renderer};
use tobj::{load_mtl_buf, load_obj_buf, LoadError, LoadOptions};

pub fn load_model_static(
    renderer: &mut Renderer,
    data: &'static str,
    materials: &[(&'static str, &'static str)],
) -> Result<GpuMeshId> {
    let config = LoadOptions {
        single_index: true,
        triangulate: false,
        ignore_points: true,
        ignore_lines: true,
    };

    let data = load_obj_buf(&mut data.as_bytes(), &config, |path| {
        let name = path.to_str().ok_or(LoadError::OpenFileFailed)?;
        let data = materials
            .iter()
            .find_map(|v| if v.0 == name { Some(v.1) } else { None })
            .ok_or(LoadError::OpenFileFailed)?;
        load_mtl_buf(&mut data.as_bytes())
    })?;

    let model = &data.0[0];
    let vertices = model
        .mesh
        .positions
        .iter()
        .tuples()
        .map(|(x, y, z)| Vec3::new(*x, *y, *z))
        .collect::<Vec<Vec3>>();
    let normals = model
        .mesh
        .normals
        .iter()
        .tuples()
        .map(|(x, y, z)| Vec3::new(*x, *y, *z))
        .collect::<Vec<Vec3>>();

    Ok(renderer.rendering_context.resource_store.build_gpu_mesh(
        &vertices,
        &normals,
        &model.mesh.indices,
    ))
}
