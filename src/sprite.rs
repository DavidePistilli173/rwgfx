//! Basic graphics element.

use crate::context::{Context, FrameContext};
use crate::shader::general;
use crate::shader::general::MeshUniform;
use crate::{texture, vertex};
use cgmath::{Point2, Vector2};
use std::cell::RefCell;
use wgpu::util::DeviceExt;

/// Index buffer data.
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

/// Rectangular element that can be drawn.
pub struct Sprite {
    /// Vertex buffer data expressed in the local coordinate frame of the button.
    vertices: [vertex::Textured; 4],
    /// Mesh data for the shader.
    mesh_uniform: MeshUniform,
    /// Vertex buffer.
    vertex_buffer: wgpu::Buffer,
    /// Index buffer.
    index_buffer: wgpu::Buffer,
    /// Mesh uniform buffer.
    mesh_uniform_buffer: wgpu::Buffer,
    /// Layout of the mesh uniform.
    mesh_uniform_layout: wgpu::BindGroupLayout,
    /// Bind group for the mesh uniform.
    mesh_uniform_bind_group: wgpu::BindGroup,
    /// ID of the texture to use when drawing the sprite.
    texture_id: u64,
    /// If true, signals that the vertex buffer needs to be updated.
    /// Interior mutability is used to allow drawing calls to not require &mut self.
    vertex_buffer_to_update: RefCell<bool>,
    /// If true, signals that the mesh uniform buffer needs to be updated.
    /// Interior mutability is used to allow drawing calls to not require &mut self.
    mesh_uniform_buffer_to_update: RefCell<bool>,
}

impl Sprite {
    /// Compute the vertex data.
    fn compute_vertices(size: &Vector2<f32>) -> [vertex::Textured; 4] {
        [
            vertex::Textured {
                position: [0.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            vertex::Textured {
                position: [0.0, size.y],
                tex_coords: [0.0, 1.0],
            },
            vertex::Textured {
                position: [size.x, size.y],
                tex_coords: [1.0, 1.0],
            },
            vertex::Textured {
                position: [size.x, 0.0],
                tex_coords: [1.0, 0.0],
            },
        ]
    }

    /// Draw the button.
    pub fn draw<'a, 'b>(&'a self, frame_context: &mut FrameContext<'b, 'a>)
    where
        'a: 'b,
    {
        // Update the vertex buffer.
        if *self.vertex_buffer_to_update.borrow() {
            frame_context.queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.vertices),
            );
            *self.vertex_buffer_to_update.borrow_mut() = false;
        }

        // Update the mesh uniform buffer.
        if *self.mesh_uniform_buffer_to_update.borrow() {
            frame_context.queue.write_buffer(
                &self.mesh_uniform_buffer,
                0,
                bytemuck::cast_slice(&[self.mesh_uniform]),
            );
            *self.mesh_uniform_buffer_to_update.borrow_mut() = false;
        }

        let texture = frame_context
            .textures
            .get(&self.texture_id).unwrap_or(frame_context.textures.get(&texture::ID_EMPTY)
            .expect("There should be at least the empty texture always loaded. If not, there is no way to make the program not crash."));

        // Perform the draw calls.
        frame_context
            .render_pass
            .set_bind_group(1, &self.mesh_uniform_bind_group, &[]);
        frame_context
            .render_pass
            .set_bind_group(2, &texture.bind_group, &[]);
        frame_context
            .render_pass
            .set_vertex_buffer(0, self.vertex_buffer.slice(..));
        frame_context
            .render_pass
            .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        frame_context
            .render_pass
            .draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }

    /// Create a new sprite.
    pub fn new(
        context: &Context,
        position: Point2<f32>,
        size: Vector2<f32>,
        z_index: f32,
        back_colour: [f32; 4],
        texture_id: Option<u64>,
    ) -> Self {
        let vertices = Sprite::compute_vertices(&size);
        let device = context.device();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mesh_uniform = general::MeshUniform::new(position.into(), z_index, 0.0, back_colour);

        let mesh_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sprite uniform buffer"),
            contents: bytemuck::cast_slice(&[mesh_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let mesh_uniform_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &MeshUniform::layout_descriptor(),
                label: Some("mesh_bind_group_layout"),
            });

        let mesh_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mesh_uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mesh_uniform_buffer.as_entire_binding(),
            }],
            label: Some("mesh_uniform_bind_group"),
        });

        Self {
            vertices,
            mesh_uniform,
            vertex_buffer,
            index_buffer,
            mesh_uniform_buffer,
            mesh_uniform_layout,
            mesh_uniform_bind_group,
            vertex_buffer_to_update: false.into(),
            mesh_uniform_buffer_to_update: false.into(),
            texture_id: texture_id.unwrap_or(texture::ID_EMPTY),
        }
    }

    /// Set a new alpha value for the sprite's overlay.
    pub fn set_overlay_alpha(&mut self, alpha: f32) {
        self.mesh_uniform.overlay_alpha = alpha;
        *self.mesh_uniform_buffer_to_update.borrow_mut() = true;
    }

    /// Set a new position for the sprite.
    pub fn set_position(&mut self, position: Point2<f32>) {
        self.mesh_uniform.position = position.into();
        *self.mesh_uniform_buffer_to_update.borrow_mut() = true;
    }

    /// Set a new size for the sprite.
    pub fn set_size(&mut self, size: Vector2<f32>) {
        self.vertices = Sprite::compute_vertices(&size);
        *self.vertex_buffer_to_update.borrow_mut() = true;
    }
}
