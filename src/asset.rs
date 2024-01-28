//! Asset management (loading/unloading/retrieval).

use rwlog::sender::Logger;
use std::collections::HashMap;

use crate::error::{self, AssetCreationError};
use crate::text::TextHandler;
use crate::texture;
use crate::texture::Texture;

/// Asset manager.
pub struct Manager {
    /// Logger.
    logger: Logger,
    /// Map of available textures ordered by ID.
    textures: HashMap<u64, Texture>,
    /// Collection of text rendering data.
    text_handler: TextHandler,
}

impl Manager {
    /// Get a texture with a given ID, if available.
    pub fn get_texture(&self, id: u64) -> Option<&Texture> {
        self.textures.get(&id)
    }

    /// Load a texture object into memory from raw bytes.
    /// Return true if the texture was loaded successfully, false otherwise.
    pub fn load_texture_from_bytes(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
        id: u64,
        label: &str,
    ) -> bool {
        let tex_res = Texture::from_bytes(device, queue, data, label);
        if let Ok(tex) = tex_res {
            self.textures.insert(id, tex);
            true
        } else {
            rwlog::rel_err!(
                &self.logger,
                "Failed to load texture {} from raw bytes: {}",
                label,
                tex_res.err().unwrap()
            );
            false
        }
    }

    /// Create a new asset manager with the default assets loaded.
    pub fn new(
        logger: Logger,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self, AssetCreationError> {
        let text_handler = TextHandler::new(&logger, "font/gnu-free=font/FreeMono.ttf")?;
        let mut result = Self {
            logger,
            textures: HashMap::new(),
            text_handler,
        };

        let empty_data = include_bytes!("texture/empty.png");
        result.load_texture_from_bytes(device, queue, empty_data, texture::ID_EMPTY, "empty");

        Ok(result)
    }
}
