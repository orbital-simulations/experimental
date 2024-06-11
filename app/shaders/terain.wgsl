@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;
@group(0) @binding(1)
var<uniform> camera: mat4x4<f32>;
@group(1) @binding(0)
var<uniform> transform: Transform;

struct Transform {
    affine1: vec4<f32>,
    affine2: vec4<f32>,
    affine3: vec4<f32>,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) color: vec3<f32>,
}
@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = mat4x4<f32>(
        vec4<f32>(transform.affine1.xyz, 0.0),
        vec4<f32>(transform.affine1.w, transform.affine2.xy, 0.0),
        vec4<f32>(transform.affine2.zw, transform.affine3.x, 0.0),
        vec4<f32>(transform.affine3.yzw, 1.0),
    );

    let world_position = model_matrix * vec4<f32>(model.position, 1.0);

    let world_matrix = projection * camera;

    let light_direction = normalize(vec3(0.3, -1.0, -1.0));
    let stupid_diffuse_strength = max(dot(model.normal, light_direction), 0.);
    let color = vec3(0., 1., 0.);
    out.color = color * 0.1 + color * stupid_diffuse_strength;
    out.clip_position = world_matrix * world_position;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
