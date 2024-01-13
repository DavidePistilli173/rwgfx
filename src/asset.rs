//! Asset management (loading/unloading/retrieval).

use rwlog::sender;
use rwlog::sender::Logger;

use crate::texture::Texture;
use glyphon::FontSystem;
use std::collections::HashMap;

/// Asset manager.
pub struct Manager {
    /// Logger.
    logger: Logger,
    /// Map of available textures ordered by ID.
    textures: HashMap<u64, Texture>,
    /// Collection of available fonts.
    font_system: FontSystem,
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

    /// Create a new asset manager with no assets loaded, except for the system fonts.
    pub fn new(logger: Logger) -> Self {
        Self {
            logger,
            textures: HashMap::new(),
            font_system: FontSystem::new(),
        }
    }
}
