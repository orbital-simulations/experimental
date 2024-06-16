#define_import_path model_matrix

fn to_model_matrix(affine_matrix_1: vec3<f32>, affine_matrix_2: vec3<f32>, affine_matrix_3: vec3<f32>, translation_vector: vec3<f32>) -> mat4x4<f32> {
    return mat4x4(vec4<f32>(affine_matrix_1, 0.0),
        vec4<f32>(affine_matrix_2, 0.0),
        vec4<f32>(affine_matrix_3, 0.0),
        vec4<f32>(translation_vector, 1.0),
    );
}
