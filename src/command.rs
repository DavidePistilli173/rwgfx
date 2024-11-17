//! Commmands that can be given to the renderer through the renderer interface.

use crate::vertex::Vertex;

/// Available commands for the renderer.
pub enum RenderCmd {
    /// Create a new mesh.
    CreateMesh(CreateMeshData),
    /// Update the vertex data for a mesh. The number of vertices must not change.
    UpdateMeshVertices(UpdateMeshVerticesData),
}

/// Data for the RenderCmd::CreateMesh command.
pub struct CreateMeshData {
    /// ID of the shader to use when rendering the mesh.
    pub shader_id: usize,
    /// ID of the new mesh.
    pub mesh_id: usize,
    /// List of vertices that compose the mesh.
    pub vertices: Vec<Vertex>,
    /// Order that will be used for rendering the vertices.
    pub indices: Vec<u32>,
}

/// Data for the RenderCmd::UpdateMeshVertices command.
pub struct UpdateMeshVerticesData {
    /// ID of the shader to update.
    pub shader_id: usize,
    /// ID of the mesh to update.
    pub mesh_id: usize,
    /// List of vertices that compose the mesh.
    pub vertices: Vec<Vertex>,
}
