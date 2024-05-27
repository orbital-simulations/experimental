@group(0) @binding(0)
var<uniform> perspective: mat4x4<f32>;
@group(0) @binding(1)
var<uniform> camera: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
}
struct InstanceInput {
    @location(1) transform_matrix_1: vec4<f32>,
    @location(2) transform_matrix_2: vec4<f32>,
    @location(3) transform_matrix_3: vec4<f32>,
    @location(4) transform_matrix_4: vec4<f32>,
    @location(5) color: vec3<f32>,
    @location(6) radius: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv_coords: vec2<f32>,
    @location(1) color: vec3<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = mat4x4<f32>(
        instance.transform_matrix_1,
        instance.transform_matrix_2,
        instance.transform_matrix_3,
        instance.transform_matrix_4,
    );
    let world_position = model_matrix * vec4<f32>(model.position.x * instance.radius, model.position.y * instance.radius, 0.0, 1.0);

    out.clip_position = perspective * camera * world_position;
    out.uv_coords = model.position;
    out.color = instance.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let circle_sd: f32 = abs(length(in.uv_coords)) - 1.0;

    if circle_sd > 0.0 {
        discard;
    }
    return vec4<f32>(in.color, 1.0);
}
