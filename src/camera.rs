use cgmath::{Matrix4, Point3, Vector3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

/// 3D orthographic camera
pub struct Camera {
    /// Position of the camera in world coordinates.
    pub eye: Point3<f32>,
    /// Point in world coordinates the camera is looking at.
    pub target: Point3<f32>,
    /// Up direction.
    pub up: Vector3<f32>,
    /// Nearest drawn Z coordinate.
    pub znear: f32,
    /// Farthest drawn Z coordinate.
    pub zfar: f32,
    /// Leftmost drawn coordinate.
    pub left: f32,
    /// Rightmost drawn coordinate.
    pub right: f32,
    /// Bottom drawn coordinate.
    pub bottom: f32,
    /// Top drawn coordinate.
    pub top: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::ortho(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.znear,
            self.zfar,
        );

        proj
    }
}
