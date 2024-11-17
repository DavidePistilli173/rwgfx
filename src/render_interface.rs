//! Module containing the definition of the renderer interface that allows any arbitrary part
//! of the program (even other threads) to send commands and data to the renderer.

use crate::command::{CreateMeshData, RenderCmd, UpdateMeshVerticesData};
use crate::error::RenderInterfaceError;
use crate::vertex::Vertex;
use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Sender;
use std::sync::Arc;

/// Renderer interface that can be cloned throughout the program.
#[derive(Clone)]
pub struct RenderInterface {
    /// Channel used for sending commands to the renderer.
    channel: Sender<RenderCmd>,
    /// Next available shader ID.
    next_shader_id: Arc<AtomicCell<usize>>,
    /// Next available mesh ID.
    next_mesh_id: Arc<AtomicCell<usize>>,
}

impl RenderInterface {
    /// Ask the renderer to create a new mesh.
    pub fn create_mesh(
        &self,
        shader_id: usize,
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
    ) -> Result<usize, RenderInterfaceError> {
        let mesh_id = self.next_mesh_id.fetch_add(1);
        self.channel
            .send(RenderCmd::CreateMesh(CreateMeshData {
                shader_id,
                mesh_id,
                vertices,
                indices,
            }))
            .map_err(|_| RenderInterfaceError::CommunicationFailed)?;
        Ok(mesh_id)
    }

    /// Create a new renderer interface from a given sender data channel.
    pub fn new(
        channel: Sender<RenderCmd>,
        next_shader_id: Arc<AtomicCell<usize>>,
        next_mesh_id: Arc<AtomicCell<usize>>,
    ) -> Self {
        Self {
            channel,
            next_shader_id,
            next_mesh_id,
        }
    }

    /// Update the vertex data for an already existing mesh. The number of vertices must not change.
    pub fn update_mesh_vertices(
        &self,
        shader_id: usize,
        mesh_id: usize,
        vertices: Vec<Vertex>,
    ) -> Result<(), RenderInterfaceError> {
        self.channel
            .send(RenderCmd::UpdateMeshVertices(UpdateMeshVerticesData {
                shader_id,
                mesh_id,
                vertices,
            }))
            .map_err(|_| RenderInterfaceError::CommunicationFailed)
    }
}
