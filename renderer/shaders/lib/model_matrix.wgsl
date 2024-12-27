#define_import_path model_matrix

fn to_model_matrix(affine_matrix_1: vec3<f32>, affine_matrix_2: vec3<f32>, affine_matrix_3: vec3<f32>, translation_vector: vec3<f32>) -> mat4x4<f32> {
    return mat4x4(vec4<f32>(affine_matrix_1, 0.0),
        vec4<f32>(affine_matrix_2, 0.0),
        vec4<f32>(affine_matrix_3, 0.0),
        vec4<f32>(translation_vector, 1.0),
    );
}

fn to_model_mesh_matrix(transform_affine1: vec4<f32>, transform_affine2: vec4<f32>, transform_affine3: vec4<f32>) -> mat4x4<f32> {
    return mat4x4<f32>(
        vec4<f32>(transform_affine1.xyz, 0.0),
        vec4<f32>(transform_affine1.w, transform_affine2.xy, 0.0),
        vec4<f32>(transform_affine2.zw, transform_affine3.x, 0.0),
        vec4<f32>(transform_affine3.yzw, 1.0),
    );
}
