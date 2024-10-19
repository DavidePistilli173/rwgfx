//! Main rendering manager and API.

use crate::error::{RendererAddMeshError, RendererCreationError, ShaderCreationError};
use crate::mesh::{Mesh, MeshDescriptor};
use crate::shader::{Shader, ShaderDescriptor};
use glium::glutin::surface::WindowSurface;
use glium::{Display, Surface};
use rwlog::sender::Logger;

/// Integrated UI shader.
pub const SHADER_ID_UI: usize = 0;

/// Parameters for the default shaders, in order of ID.
const DEFAULT_SHADER_PARAMS: &'static [ShaderDescriptor] = &[ShaderDescriptor {
    vertex_shader: include_str!("shader/ui.vert"),
    fragment_shader: include_str!("shader/ui.frag"),
}];

/// Parameters for the renderer creation.
#[derive(Debug)]
pub struct RendererDescriptor {
    /// Rendering surface.
    pub display: Display<WindowSurface>,
    /// Logger.
    pub logger: Logger,
}

/// Graphics renderer.
pub struct Renderer {
    /// Display to draw on.
    display: Display<WindowSurface>,
    /// Logger.
    logger: Logger,
    /// Available shaders and meshes for each shader.
    shaders_meshes: Vec<(Shader, Vec<Mesh>)>,
}

impl Renderer {
    /// Add a mesh to the renderer and get back its ID.
    /// # Arguments
    /// * `shader_id` - ID of the shader that will be used for rendering the mesh.
    /// * `descriptor` - Mesh creation parameters.
    pub fn add_mesh(
        &mut self,
        shader_id: usize,
        descriptor: &MeshDescriptor,
    ) -> Result<(usize, usize), RendererAddMeshError> {
        // Find the specified shader.
        let shader = self
            .shaders_meshes
            .get_mut(shader_id)
            .ok_or(RendererAddMeshError::InvalidShader)?;

        // Create and add the mesh.
        let mesh = Mesh::new(&self.display, descriptor).map_err(|e| {
            rwlog::err!(&self.logger, "Failed to create a mesh: {e}.");
            RendererAddMeshError::MeshCreationFailed
        })?;
        shader.1.push(mesh);

        // Return the shader ID and the mesh ID.
        Ok((shader_id, shader.1.len() - 1))
    }

    /// Add a shader to the renderer and get back its ID.
    pub fn add_shader(
        &mut self,
        descriptor: &ShaderDescriptor,
    ) -> Result<usize, ShaderCreationError> {
        let shader = Shader::new(&self.display, descriptor)?;
        self.shaders_meshes.push((shader, Vec::new()));
        Ok(self.shaders_meshes.len() - 1)
    }

    /// Draw a single frame.
    pub fn draw(&self) {
        let mut target = self.display.draw();
        target.clear_color(0.35, 0.05, 0.05, 0.75);

        for shader in self.shaders_meshes.iter() {
            for mesh in shader.1.iter() {
                if let Err(e) = target.draw(
                    mesh.vertex_buffer(),
                    mesh.index_buffer(),
                    shader.0.program(),
                    &glium::uniforms::EmptyUniforms,
                    &Default::default(),
                ) {
                    rwlog::err!(&self.logger, "Failed to draw mesh: {e}.");
                }
            }
        }

        target.finish().unwrap_or_else(|e| {
            rwlog::err!(&self.logger, "Failed to draw frame: {e}.");
        });
    }

    /// Initialise the default library shaders.
    fn init_shaders(
        logger: &Logger,
        display: &Display<WindowSurface>,
    ) -> Result<Vec<(Shader, Vec<Mesh>)>, ShaderCreationError> {
        // Create the output variable.
        let mut shaders = Vec::new();

        // Iterate over all descriptors for the default shaders and create them.
        for shader_info in DEFAULT_SHADER_PARAMS {
            let new_shader = Shader::new(display, shader_info).inspect_err(|e| {
                rwlog::err!(
                    logger,
                    "Failed to create default shader ({:?}) because of {e}.",
                    shader_info
                );
            })?;
            shaders.push((new_shader, Vec::new()));
        }

        // Return the newly created shaders.
        Ok(shaders)
    }

    /// Create a renderer from a window surface and return a handle that allows to send data and commands to the renderer.
    pub fn new(descriptor: RendererDescriptor) -> Result<Renderer, RendererCreationError> {
        let shaders_meshes = Renderer::init_shaders(&descriptor.logger, &descriptor.display)
            .map_err(|e| {
                rwlog::err!(
                    &descriptor.logger,
                    "Failed to create the default shaders: {e}."
                );
                RendererCreationError::ShaderCreation
            })?;

        Ok(Renderer {
            display: descriptor.display,
            logger: descriptor.logger,
            shaders_meshes,
        })
    }

    /// Set the surface to draw on.
    pub fn set_display(&mut self, display: Display<WindowSurface>) {
        self.display = display;
    }
}
