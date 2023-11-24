//! Different types of vertices.

/// Trait common to all vertex types.
pub trait Vertex {
    /// Get the buffer layout for the vertex type.
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

/// Vertex with just position data.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Plain {
    /// Vertex coordinates (x, y).
    pub position: [f32; 2],
}

impl Vertex for Plain {
    /// Get the buffer layout for this type of vertex.
    /// # Example
    /// ```
    /// use rwgfx::vertex::{Vertex, Plain};
    ///
    /// let buffer_layout = Plain::desc();
    /// ```
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Plain>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

/// Vertex with position and colour information.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Coloured {
    /// Vertex coordinates (x, y).
    pub position: [f32; 2],
    /// Vertex colour (r, g, b).
    pub colour: [f32; 3],
}

impl Vertex for Coloured {
    /// Get the buffer layout for this type of vertex.
    /// # Example
    /// ```
    /// use rwgfx::vertex::{Vertex, Coloured};
    ///
    /// let buffer_layout = Coloured::desc();
    /// ```
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Coloured>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

/// Vertex with position and texture information.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Textured {
    /// Vertex coordinates (x, y).
    pub position: [f32; 2],
    /// Texture coordinates (x, y).
    pub tex_coords: [f32; 2],
}

impl Vertex for Textured {
    /// Get the buffer layout for this type of vertex.
    /// # Example
    /// ```
    /// use rwgfx::vertex::{Vertex, Textured};
    ///
    /// let buffer_layout = Textured::desc();
    /// ```
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Textured>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
