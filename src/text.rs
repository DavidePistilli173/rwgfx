//! Text rendering context and objects.

use cgmath::{Point2, Vector2};
use rusttype::gpu_cache::{Cache, CachedBy};
use rusttype::{Font, PositionedGlyph};
use rwlog::sender::Logger;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use wgpu::util::DeviceExt;

use crate::asset;
use crate::renderer::FrameCtx;
use crate::shader::general::MeshUniform;
use crate::texture::{Origin3d, Texture, TextureFormat};
use crate::vertex;
use crate::{color, pipeline};

pub const TEXTURE_FORMAT: TextureFormat = TextureFormat::R8Uint;

/// Invalid font ID.
pub const ID_INVALID: u64 = 0;
/// ID of the default font.
pub const ID_DEFAULT: u64 = 1;

/// Possible errors during text operations.
#[derive(Debug, Clone, Copy)]
pub enum TextError {
    /// Failed to create a font cache.
    CacheCreation,
    /// Failed to load a font.
    FontLoading,
    /// Failed to get a set of pre-positioned glyphs.
    GlyphRetrieval,
}

impl Error for TextError {}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::CacheCreation => write!(f, "Failed to create a font cache."),
            Self::FontLoading => write!(f, "Failed to load a font."),
            Self::GlyphRetrieval => write!(f, "Failed to get pre-positioned glyphs."),
        }
    }
}

/// Internal structure for holding all data associated with a font.
struct FontData {
    /// Actual font.
    font: Font<'static>,
    /// Cache of recently used glyphs.
    cache: RefCell<Cache<'static>>,
    /// Flag to signal that the cache needs to be enlarged.
    enlarge_cache: RefCell<bool>,
    /// Texture for storing the cache on the GPU.
    cache_texture: Texture,
    /// Next valid ID for the pre-positioned glyphs.
    next_glyph_id: u64,
    /// Pre-positioned glyphs, identified by a unique ID provided by the TextHandler.
    positioned_glyphs: HashMap<u64, Vec<PositionedGlyph<'static>>>,
}

impl FontData {
    /// Add a new set of pre-positioned glyphs. Returns the ID of the glyphs.
    fn add_glyphs(&mut self, glyphs: Vec<PositionedGlyph<'static>>) -> u64 {
        let id = self.next_glyph_id;
        self.positioned_glyphs.insert(id, glyphs);
        self.next_glyph_id += 1;
        id
    }

    /// Update an already existing set of pre-positioned glyphs.
    fn update_glyphs(&mut self, id: u64, new_glyphs: Vec<PositionedGlyph<'static>>) {
        self.positioned_glyphs.insert(id, new_glyphs);
    }
}

/// Data required for creating a text object.
pub struct TextDescriptor {
    /// ID of the font to use.
    pub font_id: u64,
    /// Size of the font in pixels.
    pub font_size: f32,
    /// Colour of the text.
    pub color: color::Decimal,
    /// Position of the text.
    pub position: Point2<f32>,
    /// Size of the text.
    pub size: Vector2<f32>,
    /// Z-index of the text.
    pub z: f32,
}

/// Loads and stores all font rendering data.
pub struct TextHandler {
    /// Font data ordered by font ID.
    fonts: HashMap<u64, FontData>,
}

impl TextHandler {
    /// Create a cache and its GPU texture.
    fn create_cache(
        logger: &Logger,
        ctx: &rwcompute::Context,
        width: u32,
        height: u32,
    ) -> Result<(Cache<'static>, Texture), TextError> {
        let cache: Cache<'static> = Cache::builder().dimensions(width, height).build();
        let empty_cache_data = vec![128u8; width as usize * height as usize];

        let cache_texture = Texture::from_bytes(
            ctx,
            &empty_cache_data,
            Vector2::<u32> {
                x: width,
                y: height,
            },
            TEXTURE_FORMAT,
            "font_cache",
        )
        .map_err(|err| {
            rwlog::err!(logger, "Failed to create the font cache texture: {err}.");
            TextError::CacheCreation
        })?;

        Ok((cache, cache_texture))
    }

    /// Get the loaded font with the given ID.
    pub fn font(&self, id: u64) -> Option<&Font> {
        Some(&self.fonts.get(&id)?.font)
    }

    /// Create a new TextHandler with only the default font loaded.
    pub fn new(logger: &Logger, ctx: &rwcompute::Context) -> Result<Self, TextError> {
        // Load the default font.
        let default_font = Font::try_from_bytes(include_bytes!("font/gnu-free-font/FreeMono.ttf"))
            .ok_or_else(|| {
                rwlog::err!(logger, "Failed to load the default font.");
                TextError::FontLoading
            })?;

        let (cache, cache_texture) = TextHandler::create_cache(logger, ctx, 1024, 1024)?;

        let mut fonts = HashMap::new();
        fonts.insert(
            ID_DEFAULT,
            FontData {
                font: default_font,
                cache: RefCell::new(cache),
                cache_texture,
                enlarge_cache: RefCell::new(false),
                next_glyph_id: 1,
                positioned_glyphs: HashMap::new(),
            },
        );

        Ok(Self { fonts })
    }

    /// Check if any of the caches needs to be resized and resize it.
    pub fn resize_caches(
        &mut self,
        logger: &Logger,
        ctx: &rwcompute::Context,
    ) -> Result<(), TextError> {
        for font_data in &mut self.fonts {
            if *font_data.1.enlarge_cache.borrow() {
                let (cache, cache_texture) = TextHandler::create_cache(
                    logger,
                    ctx,
                    font_data.1.cache_texture.size.width * 2,
                    font_data.1.cache_texture.size.height * 2,
                )?;
                *font_data.1.cache.borrow_mut() = cache;
                font_data.1.cache_texture = cache_texture;
                *font_data.1.enlarge_cache.borrow_mut() = false;
            }
        }

        Ok(())
    }
}

// TODO: Implement Drop trait for Text.

/// Drawable text object.
pub struct Text {
    /// Displayed text.
    text: String,
    /// Position on the screen.
    position: Point2<f32>,
    /// ID of the used font.
    font_id: u64,
    /// ID of the pre-positioned glyphs.
    positioned_glyphs_id: u64,
    /// Vertex buffer data expressed in the local coordinate frame of the button.
    vertices: Vec<vertex::Textured>,
    /// Indices used in the index buffer.
    indices: Vec<u16>,
    /// Mesh data for the shader.
    mesh_uniform: MeshUniform,
    /// Vertex buffer.
    vertex_buffer: wgpu::Buffer,
    /// Index buffer.
    index_buffer: wgpu::Buffer,
    /// Mesh uniform buffer.
    mesh_uniform_buffer: wgpu::Buffer,
    /// Layout of the mesh uniform.
    mesh_uniform_layout: wgpu::BindGroupLayout,
    /// Bind group for the mesh uniform.
    mesh_uniform_bind_group: wgpu::BindGroup,
    /// If true, signals that the vertex buffer needs to be updated.
    /// Interior mutability is used to allow drawing calls to not require &mut self.
    vertex_buffer_to_update: RefCell<bool>,
    /// If true, signals that the mesh uniform buffer needs to be updated.
    /// Interior mutability is used to allow drawing calls to not require &mut self.
    mesh_uniform_buffer_to_update: RefCell<bool>,
    /// Logger
    logger: Logger,
}

impl Text {
    pub fn draw<'txt, 'pass>(
        &'txt self,
        ctx: &FrameCtx<'pass>,
        asset_manager: &'pass asset::Manager,
    ) where
        'txt: 'pass,
    {
        if ctx.active_pipeline_id() != pipeline::ID_TEXT {
            return;
        }

        // Update the vertex buffer.
        if *self.vertex_buffer_to_update.borrow() {
            ctx.gpu_ctx().queue().write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::cast_slice(&self.vertices),
            );
            *self.vertex_buffer_to_update.borrow_mut() = false;
        }

        // Update the mesh uniform buffer.
        if *self.mesh_uniform_buffer_to_update.borrow() {
            ctx.gpu_ctx().queue().write_buffer(
                &self.mesh_uniform_buffer,
                0,
                bytemuck::cast_slice(&[self.mesh_uniform]),
            );
            *self.mesh_uniform_buffer_to_update.borrow_mut() = false;
        }

        let font_data = asset_manager.text_handler().fonts.get(&self.font_id);
        if let Some(font_data) = font_data {
            if let Some(positioned_glyphs) =
                font_data.positioned_glyphs.get(&self.positioned_glyphs_id)
            {
                for glyph in positioned_glyphs {
                    font_data
                        .cache
                        .borrow_mut()
                        .queue_glyph(self.font_id as usize, glyph.clone());
                }

                match font_data.cache.borrow_mut().cache_queued(|rect, data| {
                    font_data.cache_texture.write_data(
                        ctx.gpu_ctx().queue(),
                        data,
                        Vector2::<u32> {
                            x: rect.width(),
                            y: rect.height(),
                        },
                        Origin3d {
                            x: rect.min.x,
                            y: rect.min.y,
                            z: 0,
                        },
                    );
                }) {
                    Ok(CachedBy::Adding) => {
                        // Perform the draw calls.
                        ctx.bind_data(1, &self.mesh_uniform_bind_group);
                        ctx.bind_data(2, &font_data.cache_texture.bind_group);
                        ctx.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                        ctx.set_index_buffer(
                            self.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );
                        ctx.draw_indexed(0..self.indices.len() as u32, 0, 0..1);
                    }
                    Ok(CachedBy::Reordering) => {
                        rwlog::warn!(&self.logger, "Glyph queue reordered, the text in the next frame could be corrupted. Signalling cache resize for the next frames.");
                        *font_data.enlarge_cache.borrow_mut() = true;
                    }
                    Err(err) => {
                        rwlog::warn!(
                        &self.logger,
                        "Glyph queue for font {} is too small (error {err}), signalling need to resize.",
                        self.font_id
                    );
                        *font_data.enlarge_cache.borrow_mut() = true;
                    }
                }
            } else {
                rwlog::err!(
                    &self.logger,
                    "Failed to retrieve pre-positioned glyphs with id {} from memory.",
                    self.positioned_glyphs_id
                );
            }
        } else {
            rwlog::err!(
                &self.logger,
                "Failed to retrieve font {} from memory.",
                self.font_id
            );
        }
    }

    /// Create a new drawable text.
    pub fn new(
        logger: Logger,
        ctx: &rwcompute::Context,
        text_handler: &mut TextHandler,
        text: &str,
        descriptor: &TextDescriptor,
    ) -> Self {
        let mut positioned_glyphs = Vec::new();
        let mut font_id = descriptor.font_id;

        let font_data = match text_handler.fonts.get_mut(&descriptor.font_id) {
            Some(x) => x,
            None => {
                rwlog::warn!(
                    &logger,
                    "Failed to find font {font_id}, using default font."
                );
                font_id = ID_DEFAULT;
                text_handler.fonts.get_mut(&ID_DEFAULT).unwrap_or_else(|| {
                    rwlog::fatal!(&logger, "Default font not loaded.");
                    std::process::exit(1);
                })
            }
        };

        let scale = rusttype::Scale::uniform(descriptor.font_size);
        let v_metrics = font_data.font.v_metrics(scale);
        let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
        let mut caret = rusttype::Point {
            x: 0.0,
            y: v_metrics.ascent,
        };
        let mut last_glyph_id = None;

        for c in text.chars() {
            if c.is_control() {
                match c {
                    '\n' => {
                        caret = rusttype::Point {
                            x: 0.0,
                            y: caret.y + advance_height,
                        };
                    }
                    _ => {}
                }
            } else {
                let base_glyph = font_data.font.glyph(c);
                if let Some(id) = last_glyph_id.take() {
                    caret.x += font_data.font.pair_kerning(scale, id, base_glyph.id());
                }
                last_glyph_id = Some(base_glyph.id());
                let mut glyph = base_glyph.scaled(scale).positioned(caret);
                if let Some(bb) = glyph.pixel_bounding_box() {
                    if bb.max.x > descriptor.size.x as i32 {
                        caret = rusttype::Point {
                            x: 0.0,
                            y: caret.y + advance_height,
                        };
                        glyph.set_position(caret);
                        last_glyph_id = None;
                    }
                }
                caret.x += glyph.unpositioned().h_metrics().advance_width;
                positioned_glyphs.push(glyph);
            }
        }

        let origin = rusttype::Point { x: 0.0, y: 0.0 };
        let vertices: Vec<vertex::Textured> = positioned_glyphs
            .iter()
            .filter_map(|g| font_data.cache.borrow().rect_for(0, g).ok().flatten())
            .flat_map(|(uv_rect, screen_rect)| {
                let gl_rect = rusttype::Rect {
                    min: origin
                        + (rusttype::vector(
                            screen_rect.min.x as f32 / descriptor.size.x - 0.5,
                            1.0 - screen_rect.min.y as f32 / descriptor.size.y - 0.5,
                        )) * 2.0,
                    max: origin
                        + (rusttype::vector(
                            screen_rect.max.x as f32 / descriptor.size.x - 0.5,
                            1.0 - screen_rect.max.y as f32 / descriptor.size.y - 0.5,
                        )) * 2.0,
                };

                vec![
                    vertex::Textured {
                        position: [gl_rect.min.x, gl_rect.min.y],
                        tex_coords: [uv_rect.min.x, uv_rect.min.y],
                    },
                    vertex::Textured {
                        position: [gl_rect.min.x, gl_rect.max.y],
                        tex_coords: [uv_rect.min.x, uv_rect.max.y],
                    },
                    vertex::Textured {
                        position: [gl_rect.max.x, gl_rect.max.y],
                        tex_coords: [uv_rect.max.x, uv_rect.max.y],
                    },
                    vertex::Textured {
                        position: [gl_rect.max.x, gl_rect.min.y],
                        tex_coords: [uv_rect.max.x, uv_rect.min.y],
                    },
                ]
            })
            .collect();

        let vertex_buffer = ctx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Text vertex buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let mut indices: Vec<u16> = Vec::new();
        let quad_num = vertices.len() / 4;
        let index_num = 6 * quad_num;
        indices.reserve(index_num);
        for i in 1..quad_num {
            indices.push(0 + 4 * i as u16);
            indices.push(1 + 4 * i as u16);
            indices.push(2 + 4 * i as u16);
            indices.push(2 + 4 * i as u16);
            indices.push(3 + 4 * i as u16);
            indices.push(0 + 4 * i as u16);
        }

        let index_buffer = ctx
            .device()
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Text index buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let mesh_uniform = MeshUniform::new(
            descriptor.position.into(),
            descriptor.z,
            0.0,
            [0.0, 0.0, 0.0, 0.0],
        );

        let mesh_uniform_buffer =
            ctx.device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Text uniform buffer"),
                    contents: bytemuck::cast_slice(&[mesh_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let mesh_uniform_layout =
            ctx.device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &MeshUniform::layout_descriptor(),
                    label: Some("mesh_bind_group_layout"),
                });

        let mesh_uniform_bind_group = ctx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &mesh_uniform_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: mesh_uniform_buffer.as_entire_binding(),
            }],
            label: Some("mesh_uniform_bind_group"),
        });

        let positioned_glyphs_id = font_data.add_glyphs(positioned_glyphs);

        Self {
            position: Point2::<f32> { x: 0.0, y: 0.0 },
            text: text.to_string(),
            positioned_glyphs_id,
            font_id,
            logger,
            vertices,
            indices,
            mesh_uniform,
            vertex_buffer,
            index_buffer,
            mesh_uniform_buffer,
            mesh_uniform_layout,
            mesh_uniform_bind_group,
            vertex_buffer_to_update: false.into(),
            mesh_uniform_buffer_to_update: false.into(),
        }
    }

    /// Set a new position for the sprite.
    pub fn set_position(&mut self, position: Point2<f32>) {
        self.mesh_uniform.position = position.into();
        *self.mesh_uniform_buffer_to_update.borrow_mut() = true;
    }
}
