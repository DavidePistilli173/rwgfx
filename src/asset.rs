//! Asset management (loading/unloading/retrieval).

use cgmath::Vector2;
use rwlog::sender::Logger;
use std::collections::HashMap;
use wgpu::TextureFormat;

use crate::error::AssetCreationError;
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

    /// Get a texture with a given ID, if available, or the default texture.
    pub fn get_texture_or_default(&self, id: u64) -> &Texture {
        self
            .get_texture(id).unwrap_or(self.get_texture(texture::ID_EMPTY)
            .expect("There should be at least the empty texture always loaded. If not, there is no way to make the program not crash."))
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
            rwlog::err!(
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
            rwlog::err!(
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
        let text_handler = TextHandler::new(&logger, ctx).map_err(|err| {
            rwlog::err!(&logger, "Failed to create text handler: {err}.");
            AssetCreationError::FontLoading
        })?;
        let mut result = Self {
            logger,
            textures: HashMap::new(),
            text_handler,
        };

        let empty_image =
            image::load_from_memory(include_bytes!("texture/empty.png")).map_err(|err| {
                rwlog::err!(
                    &result.logger,
                    "Failed to load the empty image texture: {}.",
                    err
                );
                AssetCreationError::TextureLoading
            })?;
        result.load_texture_from_image(ctx, empty_image, texture::ID_EMPTY, "empty");

        Ok(result)
    }

    /// Create the asset manager and load the default assets.
    pub fn new_with_defaults(
        logger: &rwlog::sender::Logger,
        ctx: &rwcompute::Context,
    ) -> Result<Self, AssetCreationError> {
        let mut asset_manager = Manager::new(logger.clone(), ctx)?;

        let hamburger_img = image::load_from_memory(include_bytes!("texture/hamburger.png"))
            .map_err(|err| {
                rwlog::err!(&logger, "Failed to load hamburger texture: {err}.");
                AssetCreationError::TextureLoading
            })?;
        if !asset_manager.load_texture_from_image(
            ctx,
            hamburger_img,
            texture::ID_HAMBURGER,
            "hamburger",
        ) {
            rwlog::err!(&logger, "Failed to load embedded hamburger texture.");
            return Err(AssetCreationError::TextureLoading);
        }

        Ok(asset_manager)
    }

    /// Get a reference to the text handler.
    pub fn text_handler(&self) -> &TextHandler {
        &self.text_handler
    }

    /// Get a mutable reference to the text handler.
    pub fn text_handler_mut(&mut self) -> &mut TextHandler {
        &mut self.text_handler
    }
}
