//! Vertex data and functions.

/// Vertex data that will be replicated on the GPU.
#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
}
implement_vertex!(Vertex, position);

/// Index for accessing the X coordinate of the vertex.
pub const X: usize = 0;
/// Index for accessing the Y coordinate of the vertex.
pub const Y: usize = 1;
