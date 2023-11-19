use crate::shader;
use cgmath::Matrix4;
use wgpu::util::DeviceExt;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

/// 3D orthographic camera
pub struct Camera {
    /// Nearest drawn Z coordinate.
    near: f32,
    /// Farthest drawn Z coordinate.
    far: f32,
    /// Leftmost drawn coordinate.
    left: f32,
    /// Rightmost drawn coordinate.
    right: f32,
    /// Bottom drawn coordinate.
    bottom: f32,
    /// Top drawn coordinate.
    top: f32,
    /// Uniform data that will be used by the shaders.
    uniform_data: shader::general::CameraUniform,
    /// Actual uniform buffer for the camera.
    buffer: wgpu::Buffer,
    /// Bind group layout for the camera uniform.
    bind_group_layout: wgpu::BindGroupLayout,
    /// Actual bind group for the camera uniform.
    bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn new_orthographic(
        device: &wgpu::Device,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let uniform_data = shader::general::CameraUniform {
            view_proj: cgmath::ortho(left, right, bottom, top, near, far).into(),
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[uniform_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Camera {
            left,
            right,
            bottom,
            top,
            near,
            far,
            uniform_data,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }
}
