//! Color type for EPICX

use serde::{Deserialize, Serialize};

/// RGBA color representation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
    pub const YELLOW: Color = Color::rgb(1.0, 1.0, 0.0);
    pub const CYAN: Color = Color::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Color = Color::rgb(1.0, 0.0, 1.0);
    pub const TRANSPARENT: Color = Color::rgba(0.0, 0.0, 0.0, 0.0);

    /// Create a new color with RGBA values (0.0 - 1.0)
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new color with RGB values (alpha = 1.0)
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// Create a new color with RGBA values
    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::new(r, g, b, a)
    }

    /// Create a color from hex value (0xRRGGBB or 0xRRGGBBAA)
    pub fn from_hex(hex: u32) -> Self {
        if hex > 0xFFFFFF {
            // RGBA format
            let r = ((hex >> 24) & 0xFF) as f32 / 255.0;
            let g = ((hex >> 16) & 0xFF) as f32 / 255.0;
            let b = ((hex >> 8) & 0xFF) as f32 / 255.0;
            let a = (hex & 0xFF) as f32 / 255.0;
            Self::rgba(r, g, b, a)
        } else {
            // RGB format
            let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
            let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
            let b = (hex & 0xFF) as f32 / 255.0;
            Self::rgb(r, g, b)
        }
    }

    /// Create a color from HSL values
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c / 2.0;

        let (r, g, b) = match (h as i32) % 360 {
            0..=59 => (c, x, 0.0),
            60..=119 => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::rgb(r + m, g + m, b + m)
    }

    /// Convert to array [r, g, b, a]
    #[inline]
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Linearly interpolate between two colors
    pub fn lerp(self, other: Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::rgba(
            self.r + (other.r - self.r) * t,
            self.g + (other.g - self.g) * t,
            self.b + (other.b - self.b) * t,
            self.a + (other.a - self.a) * t,
        )
    }

    /// Return color with modified alpha
    pub fn with_alpha(self, a: f32) -> Color {
        Color::rgba(self.r, self.g, self.b, a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}

impl From<[f32; 4]> for Color {
    fn from(arr: [f32; 4]) -> Self {
        Self::rgba(arr[0], arr[1], arr[2], arr[3])
    }
}

impl From<[f32; 3]> for Color {
    fn from(arr: [f32; 3]) -> Self {
        Self::rgb(arr[0], arr[1], arr[2])
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        color.to_array()
    }
}
