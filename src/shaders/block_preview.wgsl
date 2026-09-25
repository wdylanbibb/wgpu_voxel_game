struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) brightness: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) brightness: f32,
};

@group(0) @binding(0)
var atlas: texture_2d<f32>;

@group(0) @binding(1)
var atlas_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let yaw = radians(38.0);
    let pitch = radians(28.0);
    let y_rotated = vec3<f32>(
        input.position.x * cos(yaw) - input.position.z * sin(yaw),
        input.position.y,
        input.position.x * sin(yaw) + input.position.z * cos(yaw),
    );
    let rotated = vec3<f32>(
        y_rotated.x,
        y_rotated.y * cos(pitch) - y_rotated.z * sin(pitch),
        y_rotated.y * sin(pitch) + y_rotated.z * cos(pitch),
    );

    var output: VertexOutput;
    output.clip_position = vec4<f32>(
        rotated.x * 1.1,
        rotated.y * 1.1,
        0.5 - rotated.z * 0.12,
        1.0,
    );
    output.tex_coord = input.tex_coord;
    output.brightness = input.brightness;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(atlas, atlas_sampler, input.tex_coord);
    return vec4<f32>(color.rgb * input.brightness, color.a);
}
