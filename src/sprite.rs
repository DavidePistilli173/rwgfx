//! Module containing the definition of a sprite.

use crate::command::{CreateMeshData, UpdateMeshVerticesData};
use crate::error::RenderInterfaceError;
use crate::render_interface::RenderInterface;
use crate::vertex;
use crate::vertex::Vertex;
use cgmath::{Point2, Vector2};

const INDICES: [u32; 6] = [0, 1, 2, 2, 3, 0];

const TOP_LEFT: usize = 0;
const TOP_RIGHT: usize = 1;
const BOTTOM_RIGHT: usize = 2;
const BOTTOM_LEFT: usize = 3;

/// Quad rendered on the screen.
pub struct Sprite {
    /// Interface towards the renderer.
    render_interface: RenderInterface,
    /// Vertex data.
    vertices: [Vertex; 4],
    /// ID of the shader used for rendering the mesh.
    shader_id: usize,
    /// ID of the renderer mesh.
    mesh_id: usize,
}

impl Sprite {
    /// Create a new sprite.
    /// /// # Arguments
    /// * `render_interface` - Interface to the renderer that will draw the sprite.
    /// * `shader_id` - ID of the shader to use for rendering the mesh.
    /// * `position` - Coordinates of the top-left corner of the sprite.
    /// * `size` - Size of the sprite.
    pub fn new(
        render_interface: RenderInterface,
        shader_id: usize,
        position: Point2<f32>,
        size: Vector2<f32>,
    ) -> Result<Self, RenderInterfaceError> {
        let vertices = [
            Vertex {
                position: [position.x, position.y],
            },
            Vertex {
                position: [position.x + size.x, position.y],
            },
            Vertex {
                position: [position.x + size.x, position.y + size.y],
            },
            Vertex {
                position: [position.x, position.y + size.y],
            },
        ];

        let mesh_id =
            render_interface.create_mesh(shader_id, vertices.to_vec(), INDICES.to_vec())?;

        Ok(Self {
            render_interface,
            vertices,
            shader_id,
            mesh_id,
        })
    }

    /// Set a new position for the sprite.
    pub fn set_position(&mut self, position: Point2<f32>) -> Result<(), RenderInterfaceError> {
        let delta = Vector2::<f32> {
            x: position.x - self.vertices[TOP_LEFT].position[vertex::X],
            y: position.y - self.vertices[TOP_LEFT].position[vertex::Y],
        };

        for vertex in self.vertices.iter_mut() {
            vertex.position[vertex::X] += delta.x;
            vertex.position[vertex::Y] += delta.y;
        }

        self.render_interface.update_mesh_vertices(
            self.shader_id,
            self.mesh_id,
            self.vertices.to_vec(),
        )
    }

    /// Set a new size for the sprite.
    pub fn set_size(&mut self, size: Vector2<f32>) -> Result<(), RenderInterfaceError> {
        let current_size = Vector2::<f32> {
            x: self.vertices[TOP_RIGHT].position[vertex::X]
                - self.vertices[TOP_LEFT].position[vertex::X],
            y: self.vertices[BOTTOM_RIGHT].position[vertex::Y]
                - self.vertices[TOP_RIGHT].position[vertex::Y],
        };
        let delta = size - current_size;

        self.vertices[TOP_RIGHT].position[vertex::X] += delta.x;
        self.vertices[BOTTOM_RIGHT].position[vertex::X] += delta.x;
        self.vertices[BOTTOM_RIGHT].position[vertex::Y] += delta.y;
        self.vertices[BOTTOM_LEFT].position[vertex::Y] += delta.y;

        self.render_interface.update_mesh_vertices(
            self.shader_id,
            self.mesh_id,
            self.vertices.to_vec(),
        )
    }
}
