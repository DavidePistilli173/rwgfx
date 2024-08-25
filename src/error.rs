//! Error types.

use std::{
    error::Error,
    fmt::{self, write},
};

/// Possible errors during context initialisation.
#[derive(Debug, Copy, Clone)]
pub enum RendererCreationError {
    /// Error while creating the rendering surface.
    SurfaceCreation,
    /// Error while creating the graphics compute context.
    GraphicsContextCreation,
    /// Font library creation failed.
    FontLibraryCreation,
}

impl Error for RendererCreationError {}

impl fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::SurfaceCreation => write!(f, "Failed to create the rendering surface."),
            Self::GraphicsContextCreation => {
                write!(f, "Failed to create the graphics compute context.")
            }
            Self::FontLibraryCreation => write!(f, "Failed to initialise the font library."),
        }
    }
}

/// Possible errors during rendering.
#[derive(Debug, Copy, Clone)]
pub enum RenderError {
    /// The surface has become invalid and it needs to be recreated.
    SurfaceInvalid,
    /// The graphics device is out of memory.
    OutOfMemory,
    /// The graphics device is not responding.
    GraphicsDeviceNotResponding,
}

impl Error for RenderError {}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::SurfaceInvalid => write!(f, "The rendering surface has become invalid and it needs to be recreated (eg. by calling resize)."),
            Self::OutOfMemory => write!(f, "The graphics device has run out of memory."),
            Self::GraphicsDeviceNotResponding => write!(f, "The graphics device is not responding."),
        }
    }
}

/// Possible errors during asset loading.
#[derive(Debug, Clone, Copy)]
pub enum AssetCreationError {
    /// Failed to load a font.
    FontLoading,
    /// Failed to load a texture.
    TextureLoading,
}

impl Error for AssetCreationError {}

impl fmt::Display for AssetCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::FontLoading => write!(f, "Failed to load the default font."),
            Self::TextureLoading => write!(f, "Failed to load texture."),
        }
    }
}
