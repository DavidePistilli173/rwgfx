//! Error types.

use std::{error::Error, fmt};

/// Possible errors during context initialisation.
#[derive(Debug, Copy, Clone)]
pub enum RendererCreationError {
    /// Error while creating the rendering surface.
    SurfaceCreation,
    /// Error while retrieving a compatible rendering device (graphics card or other).
    NoPhysicalGraphicsDevice,
    /// Error while creating a logical rendering device or the command queue.
    DeviceOrQueueCreation,
    /// Font library creation failed.
    FontLibraryCreation,
}

impl Error for RendererCreationError {}

impl fmt::Display for RendererCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::SurfaceCreation => write!(f, "Failed to create the rendering surface."),
            Self::NoPhysicalGraphicsDevice => {
                write!(f, "Failed to get a compatible physical rendering device.")
            }
            Self::DeviceOrQueueCreation => write!(
                f,
                "Failed to create a logical rendering device or a command queue."
            ),
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
