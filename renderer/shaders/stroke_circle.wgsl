@group(0) @binding(0)
var<uniform> perspective: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
}
struct InstanceInput {
    @location(1) position: vec2<f32>,
    @location(2) radius: f32,
    @location(3) border_size: f32,
    @location(4) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) sdf_position: vec2<f32>,
    @location(1) color: vec3<f32>,
    @location(2) half_border: f32
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;

    let model_matrix = mat4x4<f32>(
        vec4(1.0, 0.0, 0.0, 0.0),
        vec4(0.0, 1.0, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(instance.position.x, instance.position.y, 0.0, 1.0)
    );
    let world_position = model_matrix * vec4<f32>(model.position.x * instance.radius, model.position.y * instance.radius, -0.5, 1.0);

    out.clip_position = perspective * world_position;
    out.sdf_position = model.position;
    out.color = instance.color;
    out.half_border = (instance.border_size/instance.radius)/2.0;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let circle_sd: f32 = abs(length(in.sdf_position) - 1.0 + in.half_border) - in.half_border;

    if circle_sd > 0.0 {
        discard;
    }
    return vec4<f32>(in.color, 1.0);

}
