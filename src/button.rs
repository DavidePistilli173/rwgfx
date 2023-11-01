use crate::vertex;
use crate::{animation::Animated, shader::general};
use cgmath::{Point2, Vector2};
use chrono::Duration;
use wgpu::util::DeviceExt;

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
    /// Background colour of the button.
    back_colour: wgpu::Color,
    /// Vertex buffer data expressed in the local coordinate frame of the button.
    vertices: [vertex::Plain; 4],
    /// Vertex buffer.
    vertex_buffer: wgpu::Buffer,
    /// Index buffer.
    index_buffer: wgpu::Buffer,
    /// Uniform buffer.
    uniform_buffer: wgpu::Buffer,
}

impl Button {
    /// Draw the button.
    pub fn draw<'a, 'b>(&'a self, render_pass: &'b mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
    }

    /// Create a new button.
    /// # Example
    /// ```
    /// use rwgfx::button::Button;
    ///
    /// let button = Button::new(Point2{x: 5.0, y: 10.0}, Vector2{x: 100.0, y: 25.0}, 1.0, [0.5, 0.0, 0.0]);
    /// ```
    pub fn new(
        device: &wgpu::Device,
        position: Point2<f32>,
        size: Vector2<f32>,
        z_index: f32,
        back_colour: wgpu::Color,
    ) -> Self {
        let vertices = [
            vertex::Plain {
                position: [0.0, 0.0],
            },
            vertex::Plain {
                position: [0.0, -size.y],
            },
            vertex::Plain {
                position: [size.x, -size.y],
            },
            vertex::Plain {
                position: [size.x, 0.0],
            },
        ];

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

        let uniform = general::MeshUniform {
            position: position.into(),
            z: z_index,
            back_colour: [
                back_colour.r as f32,
                back_colour.g as f32,
                back_colour.b as f32,
                back_colour.a as f32,
            ],
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Button uniform buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            position: Animated::new(position, Duration::milliseconds(200)),
            size: Animated::new(size, Duration::milliseconds(200)),
            z_index,
            back_colour,
            vertices,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
        }
    }

    /// Update the button's logic.
    pub fn update(&mut self, elapsed: Duration) {
        if !self.position.complete() {
            self.position.update(elapsed);
        }

        if !self.size.complete() {
            self.size.update(elapsed);
        }
    }
}
