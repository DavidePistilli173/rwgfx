//! Colour data.

/// Colour data, stored in normalised form ([0,1]).
pub struct Normalized {
    /// Red component.
    pub r: f32,
    /// Green component.
    pub g: f32,
    /// Blue component.
    pub b: f32,
    /// Alpha component.
    pub a: f32,
}

/// Colour data, stored in decimal form ([0,255]).
pub struct Decimal {
    /// Red component.
    pub r: u8,
    /// Green component.
    pub g: u8,
    /// Blue component.
    pub b: u8,
    /// Alpha component.
    pub a: u8,
}

impl From<Normalized> for Decimal {
    fn from(item: Normalized) -> Self {
        Decimal {
            r: (item.r * 255.0) as u8,
            g: (item.g * 255.0) as u8,
            b: (item.b * 255.0) as u8,
            a: (item.a * 255.0) as u8,
        }
    }
}

impl From<&Normalized> for Decimal {
    fn from(item: &Normalized) -> Self {
        Decimal {
            r: (item.r * 255.0) as u8,
            g: (item.g * 255.0) as u8,
            b: (item.b * 255.0) as u8,
            a: (item.a * 255.0) as u8,
        }
    }
}

impl From<Decimal> for Normalized {
    fn from(item: Decimal) -> Self {
        Normalized {
            r: item.r as f32 / 255.0,
            g: item.g as f32 / 255.0,
            b: item.b as f32 / 255.0,
            a: item.a as f32 / 255.0,
        }
    }
}

impl From<&Decimal> for Normalized {
    fn from(item: &Decimal) -> Self {
        Normalized {
            r: item.r as f32 / 255.0,
            g: item.g as f32 / 255.0,
            b: item.b as f32 / 255.0,
            a: item.a as f32 / 255.0,
        }
    }
}
