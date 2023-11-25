// Vertex shader
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

struct VertexInput {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>((model.position + mesh.position), mesh.z, 1.0);
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return mesh.back_colour + vec4<f32>(1.0 * mesh.overlay_alpha, 1.0 * mesh.overlay_alpha, 1.0 * mesh.overlay_alpha, 0.0);
}