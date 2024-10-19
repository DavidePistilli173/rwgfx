//! Data and code for GPU shaders.

use glium::glutin::surface::WindowSurface;
use glium::program::Program;
use glium::Display;

use crate::error::ShaderCreationError;

/// Data required for creating a GPU shader program.
#[derive(Debug)]
pub struct ShaderDescriptor<'a> {
    /// Source code for the vertex shader.
    pub vertex_shader: &'a str,
    /// Source code for the fragment shader.
    pub fragment_shader: &'a str,
}

/// GPU shader program.
pub struct Shader {
    /// GPU shader program.
    program: Program,
}

impl Shader {
    /// Create a new GPU shader.
    pub fn new(
        display: &Display<WindowSurface>,
        descriptor: &ShaderDescriptor,
    ) -> Result<Shader, ShaderCreationError> {
        let program = Program::from_source(
            display,
            descriptor.vertex_shader,
            descriptor.fragment_shader,
            None,
        )
        .map_err(|_| ShaderCreationError::FromSourceCreation)?;

        Ok(Shader { program })
    }

    /// Get the shader program handle.
    pub fn program(&self) -> &Program {
        &self.program
    }
}
