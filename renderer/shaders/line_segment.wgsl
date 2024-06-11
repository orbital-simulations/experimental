@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct InstanceInput {
    @location(1) affine_matrix_1: vec3<f32>,
    @location(2) affine_matrix_2: vec3<f32>,
    @location(3) affine_matrix_3: vec3<f32>,
    @location(4) translation_vector: vec3<f32>,
    @location(5) p1: vec3<f32>,
    @location(6) p2: vec3<f32>,
    @location(7) color: vec3<f32>,
    @location(8) width: f32,
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

    let delta = (instance.p2 - instance.p1);
    let center = (delta / 2.0) + instance.p1;

    let yaw = atan2(delta.y, delta.x);
    let pitch = atan2(delta.z, delta.x);

    let cos_yaw = cos(yaw);
    let sin_yaw = sin(yaw);
    let cos_pitch = cos(pitch);
    let sin_pitch = sin(pitch);

    let translation_matrix = mat4x4<f32>(
        vec4(cos_yaw * cos_pitch, sin_yaw, -cos_yaw * sin_pitch, 0.0),
        vec4(-sin_yaw * cos_pitch, cos_yaw, sin_yaw * sin_pitch, 0.0),
        vec4(sin_pitch, 0.0, cos_pitch, 0.0),
        vec4(center.x, center.y, center.z, 1.0)
    );

    let scale_matrix = mat4x4<f32>(
        vec4((length(delta) / 2.), 0.0, 0.0, 0.0),
        vec4(0.0, (instance.width / 2.) , 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(0.0, 0.0, 0.0, 1.0)
    );

    //let world_position = translation_matrix_pitch * translation_matrix_yaw * scale_matrix * vec4<f32>(model.position, 0.0, 1.0);
    let world_position = translation_matrix * scale_matrix * vec4<f32>(model.position, 0.0, 1.0);

    out.clip_position = projection * world_position;
    out.color = instance.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
