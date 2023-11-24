//! Data for the default "general" shader.

/// Uniform used for the general shader camera data.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    /// View-projection matrix.
    pub view_proj: [[f32; 4]; 4],
}

/// Uniform used for the general shader mesh data.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshUniform {
    /// Position in world coordinates.
    pub position: [f32; 2],
    /// Z coordinate.
    pub z: f32,
    /// Padding bytes for 16-bytes alignment.
    padding: f32,
    /// Background colour.
    pub back_colour: [f32; 4],
}

impl MeshUniform {
    pub fn layout_descriptor() -> Vec<wgpu::BindGroupLayoutEntry> {
        vec![wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }]
    }

    pub fn new(position: [f32; 2], z: f32, back_colour: [f32; 4]) -> Self {
        Self {
            position,
            z,
            back_colour,
            padding: 0.0,
        }
    }
}
