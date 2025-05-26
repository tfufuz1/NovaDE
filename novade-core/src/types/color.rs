//! Color representation and manipulation.

use crate::error::ColorParseError; // Use ColorParseError from crate::error
use serde::{de::Error as SerdeError, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// Represents an RGBA color with components clamped to the `[0.0, 1.0]` range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component (0.0 to 1.0).
    pub r: f32,
    /// Green component (0.0 to 1.0).
    pub g: f32,
    /// Blue component (0.0 to 1.0).
    pub b: f32,
    /// Alpha (opacity) component (0.0 for fully transparent, 1.0 for fully opaque).
    pub a: f32,
}

impl Default for Color {
    /// Returns `Color::TRANSPARENT` by default.
    fn default() -> Self {
        Color::TRANSPARENT
    }
}

// ColorParseError is now defined in crate::error

impl Color {
    // --- Constants ---
    /// Fully transparent color (R:0, G:0, B:0, A:0).
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    /// Opaque black color (R:0, G:0, B:0, A:1).
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    /// Opaque white color (R:1, G:1, B:1, A:1).
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    /// Opaque red color (R:1, G:0, B:0, A:1).
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    /// Opaque green color (R:0, G:1, B:0, A:1).
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    /// Opaque blue color (R:0, G:0, B:1, A:1).
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };

    /// Creates a new `Color` with the given RGBA components.
    ///
    /// Values are clamped to the `[0.0, 1.0]` range.
    ///
    /// # Arguments
    /// * `r`: Red component (0.0 to 1.0).
    /// * `g`: Green component (0.0 to 1.0).
    /// * `b`: Blue component (0.0 to 1.0).
    /// * `a`: Alpha component (0.0 to 1.0).
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    /// Creates a new `Color` from 8-bit RGBA components.
    ///
    /// # Arguments
    /// * `r8`: Red component (0 to 255).
    /// * `g8`: Green component (0 to 255).
    /// * `b8`: Blue component (0 to 255).
    /// * `a8`: Alpha component (0 to 255).
    pub fn from_rgba8(r8: u8, g8: u8, b8: u8, a8: u8) -> Self {
        Color {
            r: r8 as f32 / 255.0,
            g: g8 as f32 / 255.0,
            b: b8 as f32 / 255.0,
            a: a8 as f32 / 255.0,
        }
    }

    /// Converts the `Color` to 8-bit RGBA components.
    pub fn to_rgba8(&self) -> (u8, u8, u8, u8) {
        (
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
            (self.a * 255.0).round() as u8,
        )
    }

    /// Returns a new `Color` with the same RGB values but a different alpha component.
    ///
    /// The new alpha value is clamped to `[0.0, 1.0]`.
    pub fn with_alpha(&self, alpha: f32) -> Self {
        Color::new(self.r, self.g, self.b, alpha)
    }

    /// Blends this color with another color using alpha compositing (self over other).
    ///
    /// The formula used is: `C_out = C_self * A_self + C_other * A_other * (1 - A_self)`
    /// And for alpha: `A_out = A_self + A_other * (1 - A_self)`
    pub fn blend(&self, other: &Self) -> Self {
        let self_a = self.a;
        let other_a = other.a;
        let out_a = self_a + other_a * (1.0 - self_a);

        if out_a == 0.0 {
            // Avoid division by zero if resulting alpha is zero
            return Color::TRANSPARENT;
        }

        let out_r = (self.r * self_a + other.r * other_a * (1.0 - self_a)) / out_a;
        let out_g = (self.g * self_a + other.g * other_a * (1.0 - self_a)) / out_a;
        let out_b = (self.b * self_a + other.b * other_a * (1.0 - self_a)) / out_a;

        Color::new(out_r, out_g, out_b, out_a)
    }

    /// Lightens the color by a given factor.
    ///
    /// The factor should be between 0.0 (no change) and 1.0 (approaching white).
    /// This increases RGB components towards 1.0. Alpha is unchanged.
    pub fn lighten(&self, factor: f32) -> Self {
        let clamped_factor = factor.clamp(0.0, 1.0);
        Color::new(
            self.r + (1.0 - self.r) * clamped_factor,
            self.g + (1.0 - self.g) * clamped_factor,
            self.b + (1.0 - self.b) * clamped_factor,
            self.a,
        )
    }

    /// Darkens the color by a given factor.
    ///
    /// The factor should be between 0.0 (no change) and 1.0 (approaching black).
    /// This decreases RGB components towards 0.0. Alpha is unchanged.
    pub fn darken(&self, factor: f32) -> Self {
        let clamped_factor = factor.clamp(0.0, 1.0);
        Color::new(
            self.r * (1.0 - clamped_factor),
            self.g * (1.0 - clamped_factor),
            self.b * (1.0 - clamped_factor),
            self.a,
        )
    }

    /// Interpolates linearly between this color and another color.
    ///
    /// The `t` parameter is the interpolation factor, clamped to `[0.0, 1.0]`.
    /// `t = 0.0` returns `self`, `t = 1.0` returns `other`.
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        let clamped_t = t.clamp(0.0, 1.0);
        Color::new(
            self.r + (other.r - self.r) * clamped_t,
            self.g + (other.g - self.g) * clamped_t,
            self.b + (other.b - self.b) * clamped_t,
            self.a + (other.a - self.a) * clamped_t,
        )
    }

    // --- Hex String Methods ---

    /// Creates a `Color` from a hex string (e.g., "#RRGGBB", "#RGB", "#RRGGBBAA", "#RGBA").
    ///
    /// The leading '#' is required.
    /// Returns `crate::error::ColorParseError` if the string is invalid.
    pub fn from_hex(hex_string: &str) -> Result<Self, ColorParseError> {
        if !hex_string.starts_with('#') {
            return Err(ColorParseError::MissingPrefix);
        }

        let code = &hex_string[1..];
        let len = code.len();

        let (r_str, g_str, b_str, a_str) = match len {
            3 => { // #RGB
                let r = code.get(0..1).ok_or(ColorParseError::InvalidLength(len))?;
                let g = code.get(1..2).ok_or(ColorParseError::InvalidLength(len))?;
                let b = code.get(2..3).ok_or(ColorParseError::InvalidLength(len))?;
                (format!("{}{}",r,r), format!("{}{}",g,g), format!("{}{}",b,b), "FF".to_string())
            }
            4 => { // #RGBA
                let r = code.get(0..1).ok_or(ColorParseError::InvalidLength(len))?;
                let g = code.get(1..2).ok_or(ColorParseError::InvalidLength(len))?;
                let b = code.get(2..3).ok_or(ColorParseError::InvalidLength(len))?;
                let a = code.get(3..4).ok_or(ColorParseError::InvalidLength(len))?;
                (format!("{}{}",r,r), format!("{}{}",g,g), format!("{}{}",b,b), format!("{}{}",a,a))
            }
            6 => { // #RRGGBB
                (code.get(0..2).ok_or(ColorParseError::InvalidLength(len))?.to_string(),
                 code.get(2..4).ok_or(ColorParseError::InvalidLength(len))?.to_string(),
                 code.get(4..6).ok_or(ColorParseError::InvalidLength(len))?.to_string(),
                 "FF".to_string())
            }
            8 => { // #RRGGBBAA
                (code.get(0..2).ok_or(ColorParseError::InvalidLength(len))?.to_string(),
                 code.get(2..4).ok_or(ColorParseError::InvalidLength(len))?.to_string(),
                 code.get(4..6).ok_or(ColorParseError::InvalidLength(len))?.to_string(),
                 code.get(6..8).ok_or(ColorParseError::InvalidLength(len))?.to_string())
            }
            _ => return Err(ColorParseError::InvalidLength(len)),
        };

        let r8 = u8::from_str_radix(&r_str, 16).map_err(|e| ColorParseError::HexDecodingError(format!("R: {}", e)))?;
        let g8 = u8::from_str_radix(&g_str, 16).map_err(|e| ColorParseError::HexDecodingError(format!("G: {}", e)))?;
        let b8 = u8::from_str_radix(&b_str, 16).map_err(|e| ColorParseError::HexDecodingError(format!("B: {}", e)))?;
        let a8 = u8::from_str_radix(&a_str, 16).map_err(|e| ColorParseError::HexDecodingError(format!("A: {}", e)))?;
        
        Ok(Color::from_rgba8(r8, g8, b8, a8))
    }

    /// Converts the `Color` to a hex string (e.g., "#RRGGBB" or "#RRGGBBAA").
    ///
    /// # Arguments
    /// * `include_alpha`: If true, includes the alpha component in the string (RRGGBBAA).
    ///                    If false, omits alpha (RRGGBB), unless alpha is not 1.0, in which case it's always included.
    ///                    More precisely: if `include_alpha` is false and alpha is 1.0 (opaque), format is #RRGGBB.
    ///                    Otherwise (if `include_alpha` is true OR alpha is not 1.0), format is #RRGGBBAA.
    pub fn to_hex_string(&self, include_alpha_param: bool) -> String {
        let (r8, g8, b8, a8) = self.to_rgba8();

        if !include_alpha_param && a8 == 255 {
            format!("#{:02X}{:02X}{:02X}", r8, g8, b8)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", r8, g8, b8, a8)
        }
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize as hex string, always including alpha if not fully opaque for clarity.
        let hex_string = self.to_hex_string(self.a < 1.0);
        serializer.serialize_str(&hex_string)
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Color::from_hex(&s).map_err(SerdeError::custom)
    }
}

impl fmt::Display for Color {
    /// Formats the color as a hex string (e.g., "#RRGGBBAA").
    /// Always includes alpha if it's not 1.0.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex_string(self.a < 1.0))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;
    // Make sure ColorParseError is accessible for tests if tests are in a submodule
    use crate::error::ColorParseError as TestColorParseError;
    use std::fmt; // Required for fmt::Display, fmt::Debug


    assert_impl_all!(Color: std::fmt::Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize<'static>, Send, Sync, std::fmt::Display);

    #[test]
    fn color_new_clamps_values() {
        let color = Color::new(1.5, -0.5, 0.5, 2.0);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.5);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn color_default_is_transparent() {
        assert_eq!(Color::default(), Color::TRANSPARENT);
    }

    #[test]
    fn color_constants_are_correct() {
        assert_eq!(Color::TRANSPARENT, Color::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0, 1.0));
    }

    #[test]
    fn color_from_rgba8_and_to_rgba8() {
        let color = Color::from_rgba8(255, 128, 0, 64);
        assert!((color.r - 1.0).abs() < f32::EPSILON);
        assert!((color.g - 128.0 / 255.0).abs() < f32::EPSILON);
        assert!((color.b - 0.0).abs() < f32::EPSILON);
        assert!((color.a - 64.0 / 255.0).abs() < f32::EPSILON);

        let (r8, g8, b8, a8) = color.to_rgba8();
        assert_eq!(r8, 255);
        assert_eq!(g8, 128);
        assert_eq!(b8, 0);
        assert_eq!(a8, 64);
    }

    #[test]
    fn color_with_alpha() {
        let color = Color::RED.with_alpha(0.5);
        assert_eq!(color, Color::new(1.0, 0.0, 0.0, 0.5));
        let color_clamped = Color::RED.with_alpha(2.0);
        assert_eq!(color_clamped.a, 1.0);
    }

    #[test]
    fn color_blend() {
        let fg = Color::new(1.0, 0.0, 0.0, 0.5); // Red, 50% alpha
        let bg = Color::new(0.0, 0.0, 1.0, 0.5); // Blue, 50% alpha
        
        // Alpha: 0.5 + 0.5 * (1-0.5) = 0.5 + 0.25 = 0.75
        // R: (1.0 * 0.5 + 0.0 * 0.5 * 0.5) / 0.75 = 0.5 / 0.75 = 2.0/3.0
        // G: (0.0 * 0.5 + 0.0 * 0.5 * 0.5) / 0.75 = 0.0 / 0.75 = 0.0
        // B: (0.0 * 0.5 + 1.0 * 0.5 * 0.5) / 0.75 = 0.25 / 0.75 = 1.0/3.0
        let blended = fg.blend(&bg);
        assert!((blended.r - 2.0/3.0).abs() < f32::EPSILON);
        assert!((blended.g - 0.0).abs() < f32::EPSILON);
        assert!((blended.b - 1.0/3.0).abs() < f32::EPSILON);
        assert!((blended.a - 0.75).abs() < f32::EPSILON);

        let opaque_fg = Color::RED;
        let blended_with_opaque_fg = opaque_fg.blend(&bg);
        assert_eq!(blended_with_opaque_fg, Color::RED); // Opaque foreground fully covers

        let transparent_fg = Color::TRANSPARENT;
        let blended_with_transparent_fg = transparent_fg.blend(&bg);
        assert_eq!(blended_with_transparent_fg, bg); // Transparent foreground reveals background
    }

    #[test]
    fn color_lighten_darken() {
        let color = Color::new(0.5, 0.5, 0.5, 1.0);
        let lightened = color.lighten(0.5); // (0.5 + (1-0.5)*0.5) = 0.5 + 0.25 = 0.75
        assert_eq!(lightened, Color::new(0.75, 0.75, 0.75, 1.0));
        let darkened = color.darken(0.5); // (0.5 * (1-0.5)) = 0.5 * 0.5 = 0.25
        assert_eq!(darkened, Color::new(0.25, 0.25, 0.25, 1.0));

        assert_eq!(Color::BLACK.lighten(1.0), Color::WHITE);
        assert_eq!(Color::WHITE.darken(1.0), Color::BLACK);
        assert_eq!(color.lighten(0.0), color);
        assert_eq!(color.darken(0.0), color);
    }

    #[test]
    fn color_interpolate() {
        let c1 = Color::BLACK;
        let c2 = Color::WHITE;
        assert_eq!(c1.interpolate(&c2, 0.0), c1);
        assert_eq!(c1.interpolate(&c2, 1.0), c2);
        assert_eq!(c1.interpolate(&c2, 0.5), Color::new(0.5, 0.5, 0.5, 1.0));

        let c3 = Color::new(0.0,0.0,0.0,0.0);
        let c4 = Color::new(1.0,1.0,1.0,1.0);
        assert_eq!(c3.interpolate(&c4, 0.5), Color::new(0.5,0.5,0.5,0.5));
    }

    #[test]
    fn color_from_hex_valid() {
        assert_eq!(Color::from_hex("#FF0000").unwrap(), Color::RED);
        assert_eq!(Color::from_hex("#00FF00").unwrap(), Color::GREEN);
        assert_eq!(Color::from_hex("#0000FF").unwrap(), Color::BLUE);
        assert_eq!(Color::from_hex("#000").unwrap(), Color::BLACK); // Short RGB
        assert_eq!(Color::from_hex("#FFF").unwrap(), Color::WHITE);
        assert_eq!(Color::from_hex("#FF000080").unwrap(), Color::RED.with_alpha(128.0/255.0));
        assert_eq!(Color::from_hex("#F008").unwrap(), Color::RED.with_alpha(136.0/255.0)); // Short RGBA (0x88 = 136)
        assert_eq!(Color::from_hex("#123").unwrap(), Color::from_rgba8(0x11, 0x22, 0x33, 0xFF));
        assert_eq!(Color::from_hex("#1234").unwrap(), Color::from_rgba8(0x11, 0x22, 0x33, 0x44));
    }

    #[test]
    fn color_from_hex_invalid() {
        assert_eq!(Color::from_hex("FF0000"), Err(TestColorParseError::MissingPrefix));
        assert_eq!(Color::from_hex("#F0000"), Err(TestColorParseError::InvalidLength(5)));
        // For HexDecodingError, we check the variant type, not the exact string as it can be verbose
        assert!(matches!(Color::from_hex("#GG0000"), Err(TestColorParseError::HexDecodingError(_))));
        assert!(matches!(Color::from_hex("#F00G"), Err(TestColorParseError::HexDecodingError(_))));
    }

    #[test]
    fn color_to_hex_string() {
        assert_eq!(Color::RED.to_hex_string(false), "#FF0000");
        assert_eq!(Color::RED.to_hex_string(true), "#FF0000FF"); // include_alpha=true
        assert_eq!(Color::new(1.0,0.0,0.0,0.5).to_hex_string(false), "#FF000080"); // include_alpha=false, but alpha is not 1.0
        assert_eq!(Color::new(1.0,0.0,0.0,0.5).to_hex_string(true), "#FF000080");
        assert_eq!(Color::from_rgba8(0x12, 0x34, 0x56, 0x78).to_hex_string(true), "#12345678");
        assert_eq!(Color::BLACK.to_hex_string(false), "#000000");
        assert_eq!(Color::TRANSPARENT.to_hex_string(false), "#00000000"); // Alpha is 0, so always included
    }

    #[test]
    fn color_serde_serialization_deserialization() {
        let color = Color::new(0.1, 0.2, 0.3, 0.4);
        let expected_hex = color.to_hex_string(true); // Serializes with alpha if not 1.0
        
        let serialized = serde_json::to_string(&color).unwrap();
        assert_eq!(serialized, format!("\"{}\"", expected_hex));

        let deserialized: Color = serde_json::from_str(&serialized).unwrap();
        // Compare u8 tuples for robustness against float precision issues
        assert_eq!(deserialized.to_rgba8(), color.to_rgba8());


        let color_opaque = Color::RED;
        let expected_hex_opaque = color_opaque.to_hex_string(false); // Opaque, so #RRGGBB
        let serialized_opaque = serde_json::to_string(&color_opaque).unwrap();
        assert_eq!(serialized_opaque, format!("\"{}\"", expected_hex_opaque));
        let deserialized_opaque: Color = serde_json::from_str(&serialized_opaque).unwrap();
        assert_eq!(deserialized_opaque.to_rgba8(), color_opaque.to_rgba8());

        let json_rgb = "\"#FF0000\"";
        let deserialized_rgb: Color = serde_json::from_str(json_rgb).unwrap();
        assert_eq!(deserialized_rgb, Color::RED);

        let json_rgba = "\"#00FF0080\"";
        let deserialized_rgba: Color = serde_json::from_str(json_rgba).unwrap();
        assert_eq!(deserialized_rgba, Color::GREEN.with_alpha(128.0/255.0));
    }

    #[test]
    fn color_display_format() {
        assert_eq!(format!("{}", Color::RED), "#FF0000");
        assert_eq!(format!("{}", Color::RED.with_alpha(0.5)), "#FF000080");
        assert_eq!(format!("{}", Color::TRANSPARENT), "#00000000");
    }
}
