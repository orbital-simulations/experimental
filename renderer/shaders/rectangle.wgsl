#import model_matrix::to_model_matrix;

@group(0) @binding(0)
var<uniform> perspective: mat4x4<f32>;
@group(0) @binding(1)
var<uniform> camera: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
}
struct InstanceInput {
    @location(1) affine_matrix_1: vec3<f32>,
    @location(2) affine_matrix_2: vec3<f32>,
    @location(3) affine_matrix_3: vec3<f32>,
    @location(4) translation_vector: vec3<f32>,
    @location(5) size: vec2<f32>,
    @location(6) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = to_model_matrix(
        instance.affine_matrix_1,
        instance.affine_matrix_2,
        instance.affine_matrix_3,
        instance.translation_vector
    );
    let world_position = model_matrix * vec4<f32>(model.position.x * (instance.size.x/2.0), model.position.y * (instance.size.y/2.0), 0.0, 1.0);

    out.clip_position = perspective * camera * world_position;
    out.color = instance.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
