struct CameraUniform {
    view_proj: mat4x4<f32>,
}

struct MeshUniform {
    position: vec2<f32>,
    z: f32,
    overlay_alpha: f32,
    back_colour: vec4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> mesh: MeshUniform;

@group(2) @binding(0)
var t_diffuse: texture_2d<u32>;
@group(2) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>((model.position + mesh.position), mesh.z, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_sample: vec4<u32> = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var tex_background_combination: vec4<f32> = tex_sample + vec4<f32>(vec3<f32>(mesh.back_colour.x, mesh.back_colour.y, mesh.back_colour.z) * min(mesh.back_colour.w, 1.0 - tex_sample.w), min(mesh.back_colour.w, 1.0 - tex_sample.w));
    return vec4<f32>(vec3<f32>(tex_background_combination.x, tex_background_combination.y, tex_background_combination.z) * (1.0 - mesh.overlay_alpha), tex_background_combination.w) + vec4<f32>(vec3<f32>(1.0, 1.0, 1.0) * mesh.overlay_alpha, 0.0);
}