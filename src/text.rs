//! Text rendering context and objects.

use cgmath::{Point2, Vector2};

use crate::color;
use crate::renderer::{FrameContext, Renderer};
use crate::RenderPass;

/// Invalid font ID.
pub const ID_INVALID: u64 = 0;
/// ID of the default font.
pub const ID_DEFAULT: u64 = 1;

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
    /// Area bounds.
    bounds: TextBounds,
    /// Colour.
    color: glyphon::Color,
    /// Text data.
    buffer: Buffer,
}

impl Text {
    pub fn draw<'a>(
        &'a self,
        render_pass: &mut RenderPass<'a>,
        frame_context: &'a mut FrameContext<'a>,
    ) {
        let area = TextArea {
            bounds: self.bounds,
            buffer: &self.buffer,
            default_color: self.color,
            left: self.position.x,
            scale: 1.0,
            top: self.position.y,
        };

        frame_context.text_context.text_renderer.prepare_with_depth(
            frame_context.device,
            frame_context.queue,
            &mut frame_context.text_context.font_system,
            &mut frame_context.text_context.text_atlas,
            glyphon::Resolution {
                width: frame_context.window_size.x,
                height: frame_context.window_size.y,
            },
            [area],
            &mut frame_context.text_context.swash_cache,
            |z| (z as f32) / 1000.0,
        );

        frame_context
            .text_context
            .text_renderer
            .render(&frame_context.text_context.text_atlas, render_pass);
    }

    /// Create a new drawable text.
    /// Note that font_size <= 0 will panic.  
    pub fn new(renderer: &mut Renderer, text: &str, descriptor: &TextDescriptor) -> Self {
        let metrics = Metrics {
            font_size: descriptor.font_size,
            line_height: descriptor.font_size,
        };

        let bounds = TextBounds {
            left: descriptor.position.x as i32,
            bottom: (descriptor.position.y + descriptor.size.y) as i32,
            right: (descriptor.position.x + descriptor.size.x) as i32,
            top: descriptor.position.y as i32,
        };

        let color = glyphon::Color::rgba(
            descriptor.color.r,
            descriptor.color.g,
            descriptor.color.b,
            descriptor.color.a,
        );
        let mut buffer = Buffer::new(&mut renderer.text_context().font_system, metrics);
        let style = if descriptor.italic {
            glyphon::Style::Italic
        } else {
            glyphon::Style::Normal
        };
        let weight = if descriptor.bold { 4 } else { 1 };

        buffer.set_text(
            &mut renderer.text_context().font_system,
            text,
            glyphon::Attrs {
                color_opt: None,
                family: glyphon::Family::Name(descriptor.font_family),
                stretch: glyphon::Stretch::Normal,
                style,
                weight: glyphon::Weight(weight),
                metadata: (descriptor.z * 1000.0) as usize,
            },
            glyphon::Shaping::Advanced,
        );

        Self {
            position: descriptor.position,
            bounds,
            buffer,
            color,
        }
    }
}
