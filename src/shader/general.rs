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
    /// Background colour.
    pub back_colour: [f32; 4],
}
