//! Basic button widget.

use crate::shader::general::MeshUniform;
use crate::vertex;
use crate::{animation::Animated, shader::general};
use cgmath::{Point2, Vector2};
use chrono::Duration;
use std::cell::RefCell;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, MouseButton, WindowEvent};

/// Index buffer data.
const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0];

/// Rectangular object that can be interacted with.
pub struct Button {
    /// Position of the button in screen coordinates.
    position: Animated<Point2<f32>>,
    /// Size of the button
    size: Animated<Vector2<f32>>,
    /// Z-index of the button, determines which UI element is drawn on top.
    z_index: f32,
    /// If true, the mouse cursor is hovering over the button.
    hovered: bool,
    /// If true, the user is clicking the button.
    pressed: bool,
    /// Background colour of the button.
    back_colour: [f32; 4],
    /// Alpha value of the white overlay of the button (for hovered-pressed animations).
    overlay_alpha: Animated<f32>,
    /// Vertex buffer data expressed in the local coordinate frame of the button.
    vertices: [vertex::Plain; 4],
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
    /// If true, signals that the vertex buffer needs to be updated.
    /// Interior mutability is used to allow drawing calls to not require &mut self.
    vertex_buffer_to_update: RefCell<bool>,
    /// If true, signals that the mesh uniform buffer needs to be updated.
    /// Interior mutability is used to allow drawing calls to not require &mut self.
    mesh_uniform_buffer_to_update: RefCell<bool>,
}

impl Button {
    /// Compute the vertex data.
    fn compute_vertices(size: &Vector2<f32>) -> [vertex::Plain; 4] {
        [
            vertex::Plain {
                position: [0.0, 0.0],
            },
            vertex::Plain {
                position: [0.0, size.y],
            },
            vertex::Plain {
                position: [size.x, size.y],
            },
            vertex::Plain {
                position: [size.x, 0.0],
            },
        ]
    }

    /// Process an event.
    /// If the event is directed at this button, true is returned to signal that the event was consumed.
    /// Otherwise, false is returned.
    pub fn consume_event(&mut self, event: &WindowEvent) -> bool {
        let mut event_consumed = false;

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let current_button_position = self.position.current();
                let current_button_size = self.size.current();
                let right_coord = current_button_position.x + current_button_size.x;
                let down_coord = current_button_position.y + current_button_size.y;
                // If the cursor is on the button.
                if current_button_position.x <= position.x as f32
                    && position.x as f32 <= right_coord
                    && current_button_position.y <= position.y as f32
                    && position.y as f32 <= down_coord
                {
                    if !self.hovered {
                        self.hovered = true;
                        self.overlay_alpha
                            .set_target(self.overlay_alpha.target() + 0.1);
                        event_consumed = true;
                    }
                } else {
                    if self.hovered {
                        self.hovered = false;
                        self.overlay_alpha
                            .set_target(self.overlay_alpha.target() - 0.1);
                        event_consumed = true;
                    }
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // Only process the left mouse button.
                if *button == MouseButton::Left {
                    // If the button is already pressed, check for the mouse release.
                    if self.pressed {
                        if *state == ElementState::Released {
                            self.pressed = false;
                            self.overlay_alpha
                                .set_target(self.overlay_alpha.target() - 0.1);
                            event_consumed = true;
                        }
                    } else {
                        if self.hovered && *state == ElementState::Pressed {
                            self.pressed = true;
                            self.overlay_alpha
                                .set_target(self.overlay_alpha.target() + 0.1);
                            event_consumed = true;
                        }
                    }
                }
            }
            _ => (),
        }

        event_consumed
    }

    /// Draw the button.
    pub fn draw<'a, 'b>(&'a self, queue: &wgpu::Queue, render_pass: &'b mut wgpu::RenderPass<'a>) {
        // Update the vertex buffer.
        if *self.vertex_buffer_to_update.borrow() {
            //queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
            *self.vertex_buffer_to_update.borrow_mut() = false;
        }

        // Update the mesh uniform buffer.
        if *self.mesh_uniform_buffer_to_update.borrow() {
            queue.write_buffer(
                &self.mesh_uniform_buffer,
                0,
                bytemuck::cast_slice(&[self.mesh_uniform]),
            );
            *self.mesh_uniform_buffer_to_update.borrow_mut() = false;
        }

        // Perform the draw calls.
        render_pass.set_bind_group(1, &self.mesh_uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }

    /// Create a new button.
    pub fn new(
        device: &wgpu::Device,
        position: Point2<f32>,
        size: Vector2<f32>,
        z_index: f32,
        back_colour: [f32; 4],
    ) -> Self {
        let vertices = Button::compute_vertices(&size);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Button vertex buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Button index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mesh_uniform = general::MeshUniform::new(position.into(), z_index, 0.0, back_colour);

        let mesh_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Button uniform buffer"),
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
            position: Animated::new(position, Duration::milliseconds(200)),
            size: Animated::new(size, Duration::milliseconds(200)),
            z_index,
            hovered: false,
            pressed: false,
            back_colour,
            overlay_alpha: Animated::new(0.0, Duration::milliseconds(100)),
            vertices,
            mesh_uniform,
            vertex_buffer,
            index_buffer,
            mesh_uniform_buffer,
            mesh_uniform_layout,
            mesh_uniform_bind_group,
            vertex_buffer_to_update: false.into(),
            mesh_uniform_buffer_to_update: false.into(),
        }
    }

    /// Update the button's logic.
    pub fn update(&mut self, elapsed: &Duration) {
        // Position update.
        if !self.position.complete() {
            self.position.update(elapsed);
            self.mesh_uniform.position = (*self.position.current()).into();
            *self.mesh_uniform_buffer_to_update.borrow_mut() = true;
        }

        // Size update.
        if !self.size.complete() {
            self.size.update(elapsed);
            self.vertices = Button::compute_vertices(self.size.current());
            *self.vertex_buffer_to_update.borrow_mut() = true;
        }

        // Overlay alpha update.
        if !self.overlay_alpha.complete() {
            self.overlay_alpha.update(elapsed);
            self.mesh_uniform.overlay_alpha = *self.overlay_alpha.current();
            *self.mesh_uniform_buffer_to_update.borrow_mut() = true;
        }
    }
}
