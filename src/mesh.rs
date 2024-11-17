//! Graphics mesh data and functions.

use glium::glutin::surface::WindowSurface;
use glium::{Display, IndexBuffer, VertexBuffer};

use crate::error::MeshCreationError;
use crate::vertex::Vertex;

/// Data for creating a mesh.
pub struct MeshDescriptor {
    /// ID of the mesh.
    pub id: usize,
    /// List of vertices that compose the mesh.
    pub vertices: Vec<Vertex>,
    /// Order that will be used for rendering the vertices.
    pub indices: Vec<u32>,
}

/// Mesh.
pub struct Mesh {
    /// ID of the mesh.
    id: usize,
    /// Vertex buffer containing all vertices of the mesh.
    vertex_buffer: VertexBuffer<Vertex>,
    /// Index buffer containing the rendering order for each vertex.
    index_buffer: IndexBuffer<u32>,
}

impl Mesh {
    /// Get the ID of the mesh.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the index buffer of the mesh.
    pub fn index_buffer(&self) -> &IndexBuffer<u32> {
        &self.index_buffer
    }

    /// Create a new mesh.
    pub fn new(
        display: &Display<WindowSurface>,
        descriptor: &MeshDescriptor,
    ) -> Result<Mesh, MeshCreationError> {
        let vertex_buffer = VertexBuffer::new(display, &descriptor.vertices)
            .map_err(|_| MeshCreationError::VertexBufferCreation)?;
        let index_buffer = IndexBuffer::new(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &descriptor.indices,
        )
        .map_err(|_| MeshCreationError::IndexBufferCreation)?;

        Ok(Mesh {
            id: descriptor.id,
            vertex_buffer,
            index_buffer,
        })
    }

    /// Update the data stored in the mesh's vertex buffer.
    pub fn update_vertex_buffer(&mut self, vertices: &Vec<Vertex>) {
        self.vertex_buffer.write(&vertices);
    }

    /// Get the vertex buffer of the mesh.
    pub fn vertex_buffer(&self) -> &VertexBuffer<Vertex> {
        &self.vertex_buffer
    }
}
