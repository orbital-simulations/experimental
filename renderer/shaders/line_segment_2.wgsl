@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct InstanceInput {
    @location(1) p1: vec2<f32>,
    @location(2) p2: vec2<f32>,
    @location(3) color: vec3<f32>,
    @location(4) width: f32,
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
    let center = (delta / 2) + instance.p1;

    let angle = atan2(delta.x, delta.y);
    let cos_angle = cos(angle);
    let sin_angle = sin(angle);

    let translation_matrix = mat4x4<f32>(
        vec4(cos_angle, -sin_angle, 0.0, 0.0),
        vec4(sin_angle, cos_angle, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(center.x, center.y, 0.0, 1.0)
    );

    let scale_matrix = mat4x4<f32>(
        vec4((instance.width / 2.), 0.0, 0.0, 0.0),
        vec4(0.0, (length(delta) / 2.) , 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(0, 0, 0, 1.0)
    );

    let world_position = translation_matrix * scale_matrix * vec4<f32>(model.position, -0.5, 1.0);

    out.clip_position = projection * world_position;
    out.color = instance.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
