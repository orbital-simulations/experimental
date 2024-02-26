use glam::{vec3, Vec3};
use itertools::Itertools;

pub fn generate_mesh_plane(height: u32, width: u32, quad_size: f32) -> (Vec<Vec3>, Vec<u32>) {
    // Generate vertices
    let mut vertices = Vec::with_capacity((height * width) as usize);
    let mut indices = Vec::with_capacity((height * width) as usize);
    for y in 0..height {
        for x in 0..width {
            let x_1 = x as f32 * quad_size;
            let x_2 = (x as f32 + 1.) * quad_size;
            let y_1 = y as f32 * quad_size;
            let y_2 = (y as f32 + 1.) * quad_size;
            let index_base = vertices.len() as u32;

            // Triangle 1
            vertices.push(vec3(x_1, y_1, 0.));
            vertices.push(vec3(x_2, y_1, 0.));
            vertices.push(vec3(x_2, y_2, 0.));
            indices.push(index_base);
            indices.push(index_base + 1);
            indices.push(index_base + 2);

            // Triangle 2
            vertices.push(vec3(x_2, y_2, 0.));
            vertices.push(vec3(x_1, y_2, 0.));
            vertices.push(vec3(x_1, y_1, 0.));
            indices.push(index_base + 3);
            indices.push(index_base + 4);
            indices.push(index_base + 5);
        }
    }
    (vertices, indices)
}

pub fn generate_mesh_normals(vertices: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    // Generate vertices
    let mut normals = Vec::with_capacity(vertices.len());
    normals.resize(vertices.len(), Vec3::default());
    for (i1, i2, i3) in indices.iter().tuples() {
        let p1 = vertices[*i1 as usize];
        let p2 = vertices[*i2 as usize];
        let p3 = vertices[*i3 as usize];
        let dir1 = p2 - p1;
        let dir2 = p3 - p1;
        let norm = dir2.cross(dir1).normalize();
        normals[*i1 as usize] = norm;
        normals[*i2 as usize] = norm;
        normals[*i3 as usize] = norm;
    }
    normals
}
