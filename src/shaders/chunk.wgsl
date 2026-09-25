// Vertex shader

struct Camera {
    view_pos: vec4<f32>,
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
}
@group(1) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) atlas_tile: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) @interpolate(flat) atlas_tile: vec2<f32>,
};

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = camera.view_proj * vec4<f32>(input.position, 1.0);

    output.tex_coords = input.tex_coords;
    output.world_normal = input.normal;
    output.atlas_tile = input.atlas_tile;

    return output;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

struct Sun {
    direction: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    ambient: f32,
}
@group(2) @binding(2)
var<uniform> sun: Sun;

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let atlas_grid_size = vec2<f32>(16.0, 16.0);
    let atlas_uv = (input.atlas_tile + fract(input.tex_coords)) / atlas_grid_size;
    let albedo = textureSample(t_diffuse, s_diffuse, atlas_uv);

    let normal = normalize(input.world_normal);
    let light_direction = normalize(sun.direction);

    let diffuse_strength = max(dot(normal, light_direction), 0.0);

    let lighting = vec3(sun.ambient) + sun.color * sun.intensity * diffuse_strength;

    return vec4(albedo.rgb * lighting, albedo.a);
}
