//! Crate for creating desktop GUI applications with a modern look and feel out of the box.
//! # Hello World
//! ```
//! let logger = rwlog::sender::Logger::to_console(rwlog::Level::Trace);
//! let app = rwgfx::application::App::new(logger.clone()).unwrap_or_else(|e| {
//!     rwlog::rel_fatal!(&logger, "Failed to create application: {e}.");
//! });
//! rwgfx::application::run(app);
//! ```

#[macro_use]
extern crate glium;

pub mod error;
pub mod mesh;
pub mod renderer;
pub mod shader;
pub mod vertex;
