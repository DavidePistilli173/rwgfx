//! Asset management (loading/unloading/retrieval).

use cgmath::Vector2;
use rwlog::sender::Logger;
use std::collections::HashMap;
use wgpu::TextureFormat;

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
        ctx: &rwcompute::Context,
        data: &[u8],
        size: Vector2<u32>,
        format: TextureFormat,
        id: u64,
        label: &str,
    ) -> bool {
        let tex_res = Texture::from_bytes(ctx, data, size, format, label);
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

    pub fn load_texture_from_image(
        &mut self,
        ctx: &rwcompute::Context,
        image: image::DynamicImage,
        id: u64,
        label: &str,
    ) -> bool {
        let tex_res = Texture::from_image(ctx, image, label);
        if let Ok(tex) = tex_res {
            self.textures.insert(id, tex);
            true
        } else {
            rwlog::rel_err!(
                &self.logger,
                "Failed to load texture {} from raw image: {}",
                label,
                tex_res.err().unwrap()
            );
            false
        }
    }

    /// Create a new asset manager with the default assets loaded.
    pub fn new(logger: Logger, ctx: &rwcompute::Context) -> Result<Self, AssetCreationError> {
        let text_handler = TextHandler::new(&logger, "font/gnu-free=font/FreeMono.ttf", ctx)?;
        let mut result = Self {
            logger,
            textures: HashMap::new(),
            text_handler,
        };

        let empty_image =
            image::load_from_memory(include_bytes!("texture/empty.png")).map_err(|err| {
                rwlog::rel_err!(
                    &result.logger,
                    "Failed to load the empty image texture: {}.",
                    err
                );
                AssetCreationError::TextureLoading
            })?;
        result.load_texture_from_image(ctx, empty_image, texture::ID_EMPTY, "empty");

        Ok(result)
    }
}
