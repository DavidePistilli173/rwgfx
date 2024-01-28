//! Text rendering context and objects.

use cgmath::{Point2, Vector2};
use freetype::face::Face;
use rwlog::sender::Logger;
use std::collections::HashMap;

use crate::color;
use crate::error::AssetCreationError;
use crate::renderer::{FrameContext, Renderer};
use crate::texture::Texture;
use crate::RenderPass;

/// Invalid font ID.
pub const ID_INVALID: u64 = 0;
/// ID of the default font.
pub const ID_DEFAULT: u64 = 1;

/// Smallest loaded font size loaded by default.
const MIN_START_FONT_SIZE: u32 = 16;
/// Multiplication factor between two adjacent font sizes.
const FONT_SIZE_MULTIPLIER: u32 = 2;
/// Largest font size loaded by default.
const MAX_START_FONT_SIZE: u32 = 128;

/// Pre-rendered texture and coordinates for all characters in a font.
struct FontTable {
    pub texture: Texture,
    pub coordinates: HashMap<char, [f32; 8]>,
}

/// Loads and stores all font rendering data.
pub struct TextHandler {
    /// Font loading library.
    font_library: freetype::library::Library,
    /// Raw font faces, used for generating pre-rendered font textures.
    raw_fonts: HashMap<u64, Face>,
    /// Pre-rendered textures for all fonts and font-sizes.
    font_textures: HashMap<u64, Vec<FontTable>>,
}

impl TextHandler {
    pub fn new(logger: &Logger, default_font_path: &str) -> Result<Self, AssetCreationError> {
        // Initialise the font library.
        let font_library = freetype::library::Library::init().map_err(|err| {
            rwlog::err!(&logger, "Failed to initialise the font library: {err}.");
            AssetCreationError::TextLibraryCreation
        })?;

        // Load the default font.
        let default_font = match font_library.new_face(default_font_path, 0) {
            Ok(font) => font,
            Err(err) => {
                rwlog::err!(
                    logger,
                    "Failed to load the default font {default_font_path}: {err}."
                );
                return Err(AssetCreationError::DefaultFontLoading);
            }
        };

        // Generate the font textures.
        let mut font_size = MIN_START_FONT_SIZE;
        let mut font_tables_vec: Vec<FontTable> = Vec::new();
        font_tables_vec.reserve(FONT_SIZE_MULTIPLIER.ilog2() as usize);
        while font_size <= MAX_START_FONT_SIZE {
            default_font.set_pixel_sizes(0, font_size);
            font_size *= FONT_SIZE_MULTIPLIER;
        }

        let mut raw_fonts = HashMap::new();
        raw_fonts.insert(ID_DEFAULT, default_font);

        let mut font_textures = HashMap::new();

        Ok(Self {
            font_library,
            raw_fonts,
            font_textures,
        })
    }
}

/// Data required for creating a text object.
pub struct TextDescriptor<'a> {
    /// Size of the font in pixels.
    pub font_size: f32,
    /// Font family (must be a font installed in the system).
    pub font_family: &'a str,
    /// Colour of the text.
    pub color: color::Decimal,
    /// Position of the text.
    pub position: Point2<f32>,
    /// Size of the text area.
    pub size: Vector2<f32>,
    /// If true, the text will be italic.
    pub italic: bool,
    /// If true, the text will be bold.
    pub bold: bool,
    /// Z-index of the text.
    pub z: f32,
}

/// Drawable text object.
pub struct Text {
    /// Position on the screen.
    position: Point2<f32>,
}

impl Text {
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>, frame_context: &FrameContext<'a>) {}

    /// Create a new drawable text.
    /// Note that font_size <= 0 will panic.  
    pub fn new(renderer: &mut Renderer, text: &str, descriptor: &TextDescriptor) -> Self {
        Self {
            position: Point2::<f32> { x: 0.0, y: 0.0 },
        }
    }
}
