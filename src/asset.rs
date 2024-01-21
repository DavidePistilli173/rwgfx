//! Asset management (loading/unloading/retrieval).

use freetype::face::Face;
use rwlog::sender::Logger;
use std::collections::HashMap;

use crate::texture::Texture;

/// Asset manager.
pub struct Manager {
    /// Logger.
    logger: Logger,
    /// Map of available textures ordered by ID.
    textures: HashMap<u64, Texture>,
    /// Collection of available fonts ordered by ID.
    fonts: HashMap<u64, Face>,
}

impl Manager {
    /// Get a texture with a given ID, if available.
    pub fn get_texture(&self, id: u64) -> Option<&Texture> {
        self.textures.get(&id)
    }

    /// Load a font from a TTF file.
    /// Return true if the font was loaded successfully, false otherwise.
    pub fn load_font_from_file(
        &mut self,
        font_library: &freetype::library::Library,
        path: &str,
        id: u64,
        logger: &Logger,
    ) -> bool {
        match font_library.new_face(path, 0) {
            Ok(font) => {
                self.fonts.insert(id, font);
                true
            }
            Err(err) => {
                rwlog::err!(logger, "Failed to load font {path}: {err}.");
                false
            }
        }
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

    /// Create a new asset manager with no assets loaded.
    pub fn new(logger: Logger) -> Self {
        Self {
            logger,
            textures: HashMap::new(),
            fonts: HashMap::new(),
        }
    }
}
