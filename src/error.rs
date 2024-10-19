//! Errors for the rwgfx library.

use std::{error::Error, fmt};

/// Possible errors during mesh creation.
#[derive(Debug, Copy, Clone)]
pub enum MeshCreationError {
    /// Error while creating the vertex buffer.
    VertexBufferCreation,
    /// Error while creating the index buffer.
    IndexBufferCreation,
}

impl Error for MeshCreationError {}

impl fmt::Display for MeshCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::VertexBufferCreation => {
                write!(f, "Failed to create the vertex buffer.")
            }
            Self::IndexBufferCreation => {
                write!(f, "Failed to create the index buffer.")
            }
        }
    }
}

/// Possible errors during renderer creation.
#[derive(Debug, Copy, Clone)]
pub enum RendererCreationError {
    /// Error while creating the default shaders.
    ShaderCreation,
}

impl Error for RendererCreationError {}

impl fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::ShaderCreation => {
                write!(f, "Failed to create the default shaders.")
            }
        }
    }
}

/// Possible errors when adding a mesh to a renderer.
#[derive(Debug, Copy, Clone)]
pub enum RendererAddMeshError {
    /// The specified shader ID does not exist.
    InvalidShader,
    /// Failed to create the mesh.
    MeshCreationFailed,
}

impl Error for RendererAddMeshError {}

impl fmt::Display for RendererAddMeshError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidShader => {
                write!(f, "The specified shader ID does not exist.")
            }
            Self::MeshCreationFailed => {
                write!(f, "Failed to create the mesh.")
            }
        }
    }
}

/// Possible errors during shader creation.
#[derive(Debug, Copy, Clone)]
pub enum ShaderCreationError {
    /// Error while creating the program from source.
    FromSourceCreation,
}

impl Error for ShaderCreationError {}

impl fmt::Display for ShaderCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::FromSourceCreation => {
                write!(f, "Failed to create the shader from source.")
            }
        }
    }
}
