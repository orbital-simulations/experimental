use glam::{vec3, Vec3};
use itertools::Itertools;

pub fn generate_mesh_plane(height: u32, width: u32, quad_size: f32) -> Vec<Vec3> {
    // Generate vertices
    let mut vertices = Vec::with_capacity((height * width) as usize);
    for y in 0..height {
        for x in 0..width {
            let x_1 = x as f32 * quad_size;
            let x_2 = (x as f32 + 1.) * quad_size;
            let y_1 = y as f32 * quad_size;
            let y_2 = (y as f32 + 1.) * quad_size;

            // Triangle 1
            vertices.push(vec3(x_1, y_1, 0.));
            vertices.push(vec3(x_1, y_2, 0.));
            vertices.push(vec3(x_2, y_1, 0.));

            // Triangle 2
            vertices.push(vec3(x_2, y_1, 0.));
            vertices.push(vec3(x_1, y_2, 0.));
            vertices.push(vec3(x_2, y_2, 0.));
        }
    }
    vertices
}

pub fn generate_mesh_normals(vertices: &[Vec3]) -> Vec<Vec3> {
    // Generate vertices
    let mut normals = Vec::with_capacity(vertices.len());
    for (p1, p2, p3) in vertices.iter().tuples() {
        let dir1 = *p2 - *p1;
        let dir2 = *p3 - *p1;
        let norm = dir1.cross(dir2).normalize();
        normals.push(norm);
        normals.push(norm);
        normals.push(norm);
    }
    normals
}
