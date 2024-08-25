//! Crate for creating desktop GUI applications with a modern look and feel out of the box.
//! # Hello World
//! ```
//! let logger = rwlog::sender::Logger::to_console(rwlog::Level::Trace);
//! let app = rwgfx::application::App::new(logger.clone()).unwrap_or_else(|e| {
//!     rwlog::rel_fatal!(&logger, "Failed to create application: {e}.");
//! });
//! rwgfx::application::run(app);
//! ```

pub mod asset;
pub mod camera;
pub mod color;
pub mod drawable;
pub mod error;
pub mod pipeline;
pub mod renderer;
pub mod shader;
pub mod sprite;
pub mod text;
pub mod texture;
pub mod vertex;

pub use wgpu::Queue;
pub use wgpu::RenderPass;
