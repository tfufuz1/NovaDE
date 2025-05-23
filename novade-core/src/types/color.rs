//! Color representations and manipulation utilities.
//!
//! This module provides the [`Color`] struct for representing RGBA colors,
//! the [`ColorFormat`] enum for specifying different color model types,
//! and [`ColorParseError`] for handling errors during color string parsing.
//!
//! It supports creating colors from various formats (RGB, RGBA, HSL, HSLA, Hex),
//! converting between these formats, and performing common color operations like
//! blending, interpolation, and adjusting lightness/saturation.
//!
//! # Examples
//!
//! ```
//! use novade_core::types::{Color, ColorParseError};
//! use std::str::FromStr;
//!
//! // Creating colors
//! let red = Color::rgb(1.0, 0.0, 0.0);
//! let semi_transparent_blue = Color::new(0.0, 0.0, 1.0, 0.5);
//! let white_from_hex = Color::from_hex("#FFFFFF").unwrap_or_default();
//!
//! // Parsing from string
//! let green: Result<Color, ColorParseError> = Color::from_str("rgb(0, 255, 0)");
//! assert_eq!(green.unwrap(), Color::rgb(0.0, 1.0, 0.0));
//!
//! // Converting to hex
//! assert_eq!(red.to_hex(), "#ff0000");
//! assert_eq!(semi_transparent_blue.to_hex_with_alpha(), "#0000ff80");
//!
//! // Interpolation
//! let gray = red.interpolate(Color::rgb(0.0,0.0,0.0), 0.5); // Interpolates from red to black
//! assert_eq!(gray, Color::rgb(0.5, 0.0, 0.0));
//! ```

use std::fmt;
use std::str::FromStr;
use thiserror::Error; 
use std::num::ParseIntError; 
use serde::{Serialize, Deserialize, Deserializer, Serializer};
use serde::de::Error as SerdeError;

/// Error type for color parsing operations.
///
/// This enum defines errors that can occur when parsing color strings,
/// particularly hexadecimal color codes or other string formats.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ColorParseError {
    /// Indicates an invalid overall format for a hex color string.
    /// For example, missing the leading '#' or an unexpected structure.
    /// Contains the input string that caused the error.
    #[error("Invalid hex color string format: '{0}'. Expected #RGB, #RGBA, #RRGGBB, or #RRGGBBAA.")]
    InvalidHexFormat(String),

    /// Indicates an invalid hexadecimal digit was encountered within a component.
    /// Contains the problematic part of the input string and the source parsing error.
    #[error("Invalid hex digit in '{input_str}': {source}")]
    InvalidHexDigit {
        input_str: String,
        #[source]
        source: ParseIntError,
    },

    /// Indicates that a hex color string has an incorrect number of characters
    /// after the leading '#'. Expected lengths are 3 (RGB), 4 (RGBA), 6 (RRGGBB), or 8 (RRGGBBAA).
    /// Contains the input string that caused the error.
    #[error("Invalid hex color string length: '{0}'. Expected 3, 4, 6, or 8 characters after '#'.")]
    InvalidHexLength(String),

    /// A general error for non-hex string formats (like "rgb()", "hsl()") if they are malformed
    /// or contain unparsable components. Also used as a catch-all for unsupported color string types.
    /// Contains a descriptive message of the parsing failure.
    #[error("Invalid color string format: {0}")]
    InvalidFormat(String),
}

/// Specifies different color model formats.
///
/// This enum is used to indicate the format of color components, particularly
/// when converting to or from different color models.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    /// Red, Green, Blue color format.
    RGB,
    /// Red, Green, Blue, Alpha color format.
    RGBA,
    /// Hue, Saturation, Lightness color format.
    HSL,
    /// Hue, Saturation, Lightness, Alpha color format.
    HSLA,
}

/// Represents a color in RGBA (Red, Green, Blue, Alpha) format.
///
/// Components `r`, `g`, `b` (color channels) and `a` (alpha/opacity) are stored as `f32` values,
/// nominally in the range `[0.0, 1.0]`.
/// - `0.0` typically means no intensity or fully transparent.
/// - `1.0` typically means full intensity or fully opaque.
///
/// The `Color` struct provides methods for creation from various formats (e.g., hex, HSL, 8-bit RGB/RGBA),
/// conversion, and common color operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component, typically in the range `[0.0, 1.0]`.
    pub r: f32,
    /// Green component, typically in the range `[0.0, 1.0]`.
    pub g: f32,
    /// Blue component, typically in the range `[0.0, 1.0]`.
    pub b: f32,
    /// Alpha (opacity) component, in the range `[0.0, 1.0]`.
    /// `0.0` is fully transparent, `1.0` is fully opaque.
    pub a: f32,
}

impl Color {
    /// Creates a new `Color` with the given RGBA components.
    ///
    /// Each component (`r`, `g`, `b`, `a`) should be in the range `[0.0, 1.0]`.
    /// Values outside this range will be clamped.
    ///
    /// # Arguments
    ///
    /// * `r`: Red component (0.0 to 1.0).
    /// * `g`: Green component (0.0 to 1.0).
    /// * `b`: Blue component (0.0 to 1.0).
    /// * `a`: Alpha (opacity) component (0.0 for transparent, 1.0 for opaque).
    ///
    /// # Returns
    ///
    /// A new `Color` instance.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    /// Creates a new opaque `Color` (alpha = 1.0) with the given RGB components.
    ///
    /// Each component (`r`, `g`, `b`) should be in the range `[0.0, 1.0]`.
    /// Values outside this range will be clamped by the call to `Color::new`.
    ///
    /// # Arguments
    ///
    /// * `r`: Red component (0.0 to 1.0).
    /// * `g`: Green component (0.0 to 1.0).
    /// * `b`: Blue component (0.0 to 1.0).
    ///
    /// # Returns
    ///
    /// A new opaque `Color` instance.
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color::new(r, g, b, 1.0)
    }

    /// Creates a new opaque `Color` (alpha = 1.0) from RGB components in the range `[0, 255]`.
    ///
    /// The 8-bit integer components are normalized to `f32` values in `[0.0, 1.0]`.
    ///
    /// # Arguments
    ///
    /// * `r`: Red component (0 to 255).
    /// * `g`: Green component (0 to 255).
    /// * `b`: Blue component (0 to 255).
    ///
    /// # Returns
    ///
    /// A new opaque `Color` instance.
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Color::rgb(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        )
    }

    /// Creates a new `Color` from RGBA components in the range `[0, 255]`.
    ///
    /// The 8-bit integer components are normalized to `f32` values in `[0.0, 1.0]`.
    ///
    /// # Arguments
    ///
    /// * `r`: Red component (0 to 255).
    /// * `g`: Green component (0 to 255).
    /// * `b`: Blue component (0 to 255).
    /// * `a`: Alpha component (0 to 255).
    ///
    /// # Returns
    ///
    /// A new `Color` instance.
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color::new(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            f32::from(a) / 255.0,
        )
    }

    /// Creates a new `Color` from a hexadecimal string (e.g., "#RRGGBB", "#RGB", "#RRGGBBAA").
    ///
    /// Supported formats:
    /// - `"#RGB"` (e.g., `"#F00"` for red)
    /// - `"#RGBA"` (e.g., `"#F008"` for semi-transparent red)
    /// - `"#RRGGBB"` (e.g., `"#FF0000"` for red)
    /// - `"#RRGGBBAA"` (e.g., `"#FF000080"` for semi-transparent red)
    ///
    /// The parsing is case-insensitive for hex digits.
    ///
    /// # Arguments
    ///
    /// * `hex_str`: The hexadecimal color string.
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `Color` if successful, or a [`ColorParseError`]
    /// if parsing failed due to an invalid format, length, or hex digits.
    pub fn from_hex(hex_str: &str) -> Result<Self, ColorParseError> {
        if !hex_str.starts_with('#') {
            return Err(ColorParseError::InvalidHexFormat(hex_str.to_string()));
        }
        let input = &hex_str[1..]; // Remove #

        let parse_hex_component = |s: &str| {
            u8::from_str_radix(s, 16)
                .map_err(|e| ColorParseError::InvalidHexDigit{ input_str: s.to_string(), source: e })
        };
        
        match input.len() {
            3 => { // #RGB
                let r_char = &input[0..1];
                let g_char = &input[1..2];
                let b_char = &input[2..3];

                let r = parse_hex_component(r_char)?;
                let g = parse_hex_component(g_char)?;
                let b = parse_hex_component(b_char)?;
                
                Ok(Color::from_rgb8((r << 4) | r, (g << 4) | g, (b << 4) | b))
            },
            4 => { // #RGBA
                let r_char = &input[0..1];
                let g_char = &input[1..2];
                let b_char = &input[2..3];
                let a_char = &input[3..4];

                let r = parse_hex_component(r_char)?;
                let g = parse_hex_component(g_char)?;
                let b = parse_hex_component(b_char)?;
                let a = parse_hex_component(a_char)?;

                Ok(Color::from_rgba8((r << 4) | r, (g << 4) | g, (b << 4) | b, (a << 4) | a))
            },
            6 => { // #RRGGBB
                let r_str = &input[0..2];
                let g_str = &input[2..4];
                let b_str = &input[4..6];

                let r = parse_hex_component(r_str)?;
                let g = parse_hex_component(g_str)?;
                let b = parse_hex_component(b_str)?;
                
                Ok(Color::from_rgb8(r, g, b))
            },
            8 => { // #RRGGBBAA
                let r_str = &input[0..2];
                let g_str = &input[2..4];
                let b_str = &input[4..6];
                let a_str = &input[6..8];

                let r = parse_hex_component(r_str)?;
                let g = parse_hex_component(g_str)?;
                let b = parse_hex_component(b_str)?;
                let a = parse_hex_component(a_str)?;
                
                Ok(Color::from_rgba8(r, g, b, a))
            },
            _ => Err(ColorParseError::InvalidHexLength(hex_str.to_string())),
        }
    }

    /// Creates a new opaque `Color` (alpha = 1.0) from HSL (Hue, Saturation, Lightness) components.
    ///
    /// # Arguments
    ///
    /// * `h`: Hue, in degrees. Typically in the range `[0.0, 360.0]`, but values outside this range
    ///   will be wrapped (e.g., 370.0 becomes 10.0).
    /// * `s`: Saturation, in the range `[0.0, 1.0]`. Values outside this range will be clamped.
    /// * `l`: Lightness, in the range `[0.0, 1.0]`. Values outside this range will be clamped.
    ///
    /// # Returns
    ///
    /// A new opaque `Color` instance converted from HSL to RGB.
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let h = h % 360.0; // Wrap hue
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);
        
        if s == 0.0 {
            // Achromatic (gray)
            return Color::rgb(l, l, l);
        }
        
        let q = if l < 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        
        let p = 2.0 * l - q;
        let h = h / 360.0; // Normalize hue to [0, 1]
        
        let r = hue_to_rgb(p, q, h + 1.0/3.0);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - 1.0/3.0);
        
        Color::rgb(r, g, b)
    }

    /// Creates a new `Color` from HSLA (Hue, Saturation, Lightness, Alpha) components.
    ///
    /// # Arguments
    ///
    /// * `h`: Hue, in degrees. Typically in the range `[0.0, 360.0]`, wrapped if outside.
    /// * `s`: Saturation, in the range `[0.0, 1.0]`. Clamped if outside.
    /// * `l`: Lightness, in the range `[0.0, 1.0]`. Clamped if outside.
    /// * `a`: Alpha, in the range `[0.0, 1.0]`. Clamped if outside.
    ///
    /// # Returns
    ///
    /// A new `Color` instance converted from HSLA to RGBA.
    pub fn from_hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        let mut color = Color::from_hsl(h, s, l);
        color.a = a.clamp(0.0, 1.0); // Clamp alpha
        color
    }

    /// Converts this `Color` (which is RGBA) to HSL (Hue, Saturation, Lightness) components.
    /// The alpha component of the original color is ignored in this conversion.
    ///
    /// # Returns
    ///
    /// A tuple `(h, s, l)` where:
    /// - `h`: Hue, in degrees `[0.0, 360.0)`. If saturation is 0 (achromatic), hue is 0.0.
    /// - `s`: Saturation, `[0.0, 1.0]`.
    /// - `l`: Lightness, `[0.0, 1.0]`.
    pub fn to_hsl(&self) -> (f32, f32, f32) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        
        let l = (max + min) / 2.0;
        
        if max == min {
            // Achromatic (gray)
            return (0.0, 0.0, l);
        }
        
        let d = max - min;
        let s = if l > 0.5 {
            d / (2.0 - max - min)
        } else {
            d / (max + min)
        };
        
        let h = if max == self.r {
            (self.g - self.b) / d + (if self.g < self.b { 6.0 } else { 0.0 })
        } else if max == self.g {
            (self.b - self.r) / d + 2.0
        } else {
            (self.r - self.g) / d + 4.0
        };
        
        (h * 60.0, s, l)
    }

    /// Converts the RGB components of this `Color` to 8-bit integer values in the range `[0, 255]`.
    /// The alpha component is ignored.
    ///
    /// # Returns
    ///
    /// A tuple `(r, g, b)` where each component is a `u8` from 0 to 255.
    pub fn to_rgb8(&self) -> (u8, u8, u8) {
        (
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
        )
    }

    /// Converts the RGBA components of this `Color` to 8-bit integer values in the range `[0, 255]`.
    ///
    /// # Returns
    ///
    /// A tuple `(r, g, b, a)` where each component is a `u8` from 0 to 255.
    pub fn to_rgba8(&self) -> (u8, u8, u8, u8) {
        (
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
            (self.a * 255.0).round() as u8,
        )
    }

    /// Converts this `Color` to a hexadecimal string in the format `"#RRGGBB"`.
    /// The alpha component is ignored.
    ///
    /// # Returns
    ///
    /// A hexadecimal color string (e.g., `"#FF0000"` for red).
    pub fn to_hex(&self) -> String {
        let (r, g, b) = self.to_rgb8();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    /// Converts this `Color` to a hexadecimal string in the format `"#RRGGBBAA"`.
    ///
    /// # Returns
    ///
    /// A hexadecimal color string including the alpha component (e.g., `"#FF000080"` for semi-transparent red).
    pub fn to_hex_with_alpha(&self) -> String {
        let (r, g, b, a) = self.to_rgba8();
        format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
    }

    /// Linearly interpolates RGBA components between this color and another color.
    /// This method is synonymous with `interpolate` but kept for compatibility or specific clarity.
    ///
    /// # Arguments
    ///
    /// * `other`: The target `Color` to interpolate towards.
    /// * `factor`: The interpolation factor, typically in `[0.0, 1.0]`.
    ///   - A factor of `0.0` returns this color.
    ///   - A factor of `1.0` returns the `other` color.
    ///   - Values outside `[0.0, 1.0]` are clamped.
    ///
    /// # Returns
    ///
    /// A new `Color` representing the interpolation result.
    pub fn blend(&self, other: &Color, factor: f32) -> Self {
        let factor = factor.clamp(0.0, 1.0);
        let inv_factor = 1.0 - factor;
        
        Color::new(
            self.r * inv_factor + other.r * factor,
            self.g * inv_factor + other.g * factor,
            self.b * inv_factor + other.b * factor,
            self.a * inv_factor + other.a * factor,
        )
    }

    /// Blends this color (foreground) with a `background` color using alpha compositing.
    ///
    /// This operation simulates placing this color over the `background` color,
    /// considering the alpha (opacity) of both.
    ///
    /// The formula for each color channel (e.g., red) is:
    /// `(fore_R * fore_A + back_R * back_A * (1 - fore_A)) / final_A`
    /// And the final alpha is:
    /// `final_A = fore_A + back_A * (1 - fore_A)`
    ///
    /// # Arguments
    ///
    /// * `background`: The `Color` to use as the background.
    ///
    /// # Returns
    ///
    /// A new `Color` representing the result of alpha blending.
    pub fn alpha_blend(&self, background: &Color) -> Self {
        let a = self.a + background.a * (1.0 - self.a);
        
        if a < 1e-6 {
            return Color::new(0.0, 0.0, 0.0, 0.0);
        }
        
        let r = (self.r * self.a + background.r * background.a * (1.0 - self.a)) / a;
        let g = (self.g * self.a + background.g * background.a * (1.0 - self.a)) / a;
        let b = (self.b * self.a + background.b * background.a * (1.0 - self.a)) / a;
        
        Color::new(r, g, b, a)
    }

    /// Creates a new `Color` with adjusted lightness.
    ///
    /// The color is converted to HSL, its lightness (`l`) component is modified by `amount`,
    /// and then it's converted back to RGBA. The original alpha is preserved.
    ///
    /// # Arguments
    ///
    /// * `amount`: The amount to adjust lightness by. Positive values lighten,
    ///   negative values darken. The resulting lightness is clamped to `[0.0, 1.0]`.
    ///
    /// # Returns
    ///
    /// A new `Color` with adjusted lightness.
    pub fn adjust_lightness(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_l = (l + amount).clamp(0.0, 1.0);
        let mut color = Color::from_hsl(h, s, new_l);
        color.a = self.a;
        color
    }

    /// Creates a new `Color` with adjusted saturation.
    ///
    /// The color is converted to HSL, its saturation (`s`) component is modified by `amount`,
    /// and then it's converted back to RGBA. The original alpha is preserved.
    ///
    /// # Arguments
    ///
    /// * `amount`: The amount to adjust saturation by. Positive values increase saturation,
    ///   negative values decrease it. The resulting saturation is clamped to `[0.0, 1.0]`.
    ///
    /// # Returns
    ///
    /// A new `Color` with adjusted saturation.
    pub fn adjust_saturation(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_s = (s + amount).clamp(0.0, 1.0);
        let mut color = Color::from_hsl(h, new_s, l);
        color.a = self.a;
        color
    }

    /// Linearly interpolates RGBA components between this color and another color.
    ///
    /// The interpolation factor `t` is clamped to the range `[0.0, 1.0]`.
    /// - A factor of `0.0` returns this color.
    /// - A factor of `1.0` returns the `other` color.
    ///
    /// # Arguments
    ///
    /// * `other`: The target `Color` to interpolate towards.
    /// * `t`: The interpolation factor.
    ///
    /// # Returns
    ///
    /// A new `Color` representing the interpolation result.
    #[must_use]
    pub fn interpolate(&self, other: Color, t: f32) -> Self {
        let t_clamped = t.clamp(0.0, 1.0);
        let r = self.r * (1.0 - t_clamped) + other.r * t_clamped;
        let g = self.g * (1.0 - t_clamped) + other.g * t_clamped;
        let b = self.b * (1.0 - t_clamped) + other.b * t_clamped;
        let a = self.a * (1.0 - t_clamped) + other.a * t_clamped;
        Color::new(r, g, b, a) // new() will clamp components
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Always serialize with Alpha for consistency, as per spec.
        serializer.serialize_str(&self.to_hex_with_alpha())
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

impl FromStr for Color {
    type Err = ColorParseError;

    /// Parses a color string into a `Color`.
    ///
    /// Supports various formats:
    /// - Hexadecimal: `"#RGB"`, `"#RGBA"`, `"#RRGGBB"`, `"#RRGGBBAA"` (parsed by `Color::from_hex`)
    /// - RGB: `"rgb(r, g, b)"` where r, g, b are u8 values (e.g., `"rgb(255, 0, 0)"`)
    /// - RGBA: `"rgba(r, g, b, a)"` where r, g, b are u8 and a is f32 (e.g., `"rgba(255, 0, 0, 0.5)"`)
    /// - HSL: `"hsl(h, s%, l%)"` or `"hsl(h, s, l)"` where h is f32 (degrees), s and l are f32 (0-1 or 0-100%)
    ///   (e.g., `"hsl(0, 100%, 50%)"` or `"hsl(0, 1.0, 0.5)"`)
    ///
    /// # Errors
    /// Returns a [`ColorParseError`] if the string format is unrecognized, or if parsing
    /// a recognized format fails (e.g., invalid numbers, incorrect component count).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('#') {
            Color::from_hex(s)
        } else if s.starts_with("rgb(") && s.ends_with(')') {
            let content = &s[4..s.len() - 1];
            let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();
            
            if parts.len() != 3 {
                return Err(ColorParseError::InvalidFormat(format!("Invalid RGB format: {}", s)));
            }
            
            let r = parts[0].parse::<u8>()
                .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid red component in '{}': {}", s, e)))?;
            let g = parts[1].parse::<u8>()
                .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid green component in '{}': {}", s, e)))?;
            let b = parts[2].parse::<u8>()
                .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid blue component in '{}': {}", s, e)))?;
            
            Ok(Color::from_rgb8(r, g, b))
        } else if s.starts_with("rgba(") && s.ends_with(')') {
            let content = &s[5..s.len() - 1];
            let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();
            
            if parts.len() != 4 {
                return Err(ColorParseError::InvalidFormat(format!("Invalid RGBA format: {}", s)));
            }
            
            let r = parts[0].parse::<u8>()
                .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid red component in '{}': {}", s, e)))?;
            let g = parts[1].parse::<u8>()
                .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid green component in '{}': {}", s, e)))?;
            let b = parts[2].parse::<u8>()
                .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid blue component in '{}': {}", s, e)))?;
            
            let a = parts[3].parse::<f32>()
                .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid alpha component in '{}': {}", s, e)))?;
            
            Ok(Color::new(
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
                a.clamp(0.0, 1.0), // Ensure alpha is clamped
            ))
        } else if s.starts_with("hsl(") && s.ends_with(')') {
            let content = &s[4..s.len() - 1];
            let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();
            
            if parts.len() != 3 {
                return Err(ColorParseError::InvalidFormat(format!("Invalid HSL format: {}", s)));
            }
            
            let h = parts[0].parse::<f32>()
                .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid hue component in '{}': {}", s, e)))?;
            
            let s_part = parts[1];
            let s = if s_part.ends_with('%') {
                s_part[..s_part.len() - 1].parse::<f32>()
                    .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid saturation component in '{}': {}", s, e)))? / 100.0
            } else {
                s_part.parse::<f32>()
                    .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid saturation component in '{}': {}", s, e)))?
            };
            
            let l_part = parts[2];
            let l = if l_part.ends_with('%') {
                l_part[..l_part.len() - 1].parse::<f32>()
                    .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid lightness component in '{}': {}", s, e)))? / 100.0
            } else {
                l_part.parse::<f32>()
                    .map_err(|e| ColorParseError::InvalidFormat(format!("Invalid lightness component in '{}': {}", s, e)))?
            };
            
            Ok(Color::from_hsl(h, s_val.clamp(0.0, 1.0), l_val.clamp(0.0, 1.0))) // Ensure s and l are clamped
        } else {
            Err(ColorParseError::InvalidFormat(format!("Unsupported color format: {}", s)))
        }
    }
}

impl fmt::Display for Color {
    /// Formats the `Color` as a string.
    ///
    /// - If alpha is 1.0 (opaque), formats as `"rgb(r, g, b)"`.
    /// - Otherwise, formats as `"rgba(r, g, b, a)"`.
    ///
    /// Components `r`, `g`, `b` are represented as 8-bit integers (0-255).
    /// Alpha `a` is represented as a float.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 1.0 {
            let (r, g, b) = self.to_rgb8();
            write!(f, "rgb({}, {}, {})", r, g, b)
        } else {
            let (r, g, b, _) = self.to_rgba8();
            write!(f, "rgba({}, {}, {}, {})", r, g, b, self.a)
        }
    }
}

/// Internal helper function to convert a HSL hue component to an RGB channel value.
///
/// This function is part of the HSL to RGB conversion algorithm.
///
/// # Arguments
///
/// * `p`: Calculated intermediate value from HSL conversion.
/// * `q`: Another calculated intermediate value from HSL conversion.
/// * `t`: A temporary hue value, normalized and potentially offset.
///        It's adjusted to be within `[0.0, 1.0]` before calculations.
///
/// # Returns
///
/// The calculated RGB channel component value in the range `[0.0, 1.0]`.
fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    
    if t < 1.0/6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0/2.0 {
        return q;
    }
    if t < 2.0/3.0 {
        return p + (q - p) * (2.0/3.0 - t) * 6.0;
    }
    
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let color = Color::new(0.5, 0.6, 0.7, 0.8);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.6);
        assert_eq!(color.b, 0.7);
        assert_eq!(color.a, 0.8);
    }

    #[test]
    fn test_color_rgb() {
        let color = Color::rgb(0.5, 0.6, 0.7);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.6);
        assert_eq!(color.b, 0.7);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_from_rgb8() {
        let color = Color::from_rgb8(128, 153, 179);
        assert!((color.r - 0.5).abs() < 0.01);
        assert!((color.g - 0.6).abs() < 0.01);
        assert!((color.b - 0.7).abs() < 0.01);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_color_from_rgba8() {
        let color = Color::from_rgba8(128, 153, 179, 204);
        assert!((color.r - 0.5).abs() < 0.01);
        assert!((color.g - 0.6).abs() < 0.01);
        assert!((color.b - 0.7).abs() < 0.01);
        assert!((color.a - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_color_from_hex() {
        // Test #RGB format
        let color = Color::from_hex("#8af").expect("Failed to parse #8af");
        assert_eq!(color.to_rgba8(), (0x88, 0xaa, 0xff, 0xff));
        
        // Test #RGBA format
        let color = Color::from_hex("#8afc").expect("Failed to parse #8afc");
        assert_eq!(color.to_rgba8(), (0x88, 0xaa, 0xff, 0xcc));
        
        // Test #RRGGBB format
        let color = Color::from_hex("#8899ff").expect("Failed to parse #8899ff");
        assert_eq!(color.to_rgba8(), (0x88, 0x99, 0xff, 0xff));
        
        // Test #RRGGBBAA format
        let color = Color::from_hex("#8899ffcc").expect("Failed to parse #8899ffcc");
        assert_eq!(color.to_rgba8(), (0x88, 0x99, 0xff, 0xcc));

        // Test error cases
        assert!(matches!(Color::from_hex("8899ff"), Err(ColorParseError::InvalidHexFormat(_))));
        assert!(matches!(Color::from_hex("#12345"), Err(ColorParseError::InvalidHexLength(_))));
        assert!(matches!(Color::from_hex("#12G"), Err(ColorParseError::InvalidHexDigit{..})));
    }

    #[test]
    fn test_color_from_hsl() {
        // Red
        let color = Color::from_hsl(0.0, 1.0, 0.5);
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.0).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
        
        // Green
        let color = Color::from_hsl(120.0, 1.0, 0.5);
        assert!((color.r - 0.0).abs() < 0.01);
        assert!((color.g - 1.0).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
        
        // Blue
        let color = Color::from_hsl(240.0, 1.0, 0.5);
        assert!((color.r - 0.0).abs() < 0.01);
        assert!((color.g - 0.0).abs() < 0.01);
        assert!((color.b - 1.0).abs() < 0.01);
        
        // Gray (saturation = 0)
        let color = Color::from_hsl(0.0, 0.0, 0.5);
        assert!((color.r - 0.5).abs() < 0.01);
        assert!((color.g - 0.5).abs() < 0.01);
        assert!((color.b - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_to_hsl() {
        // Red
        let color = Color::rgb(1.0, 0.0, 0.0);
        let (h, s, l) = color.to_hsl();
        assert!((h - 0.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
        
        // Green
        let color = Color::rgb(0.0, 1.0, 0.0);
        let (h, s, l) = color.to_hsl();
        assert!((h - 120.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
        
        // Blue
        let color = Color::rgb(0.0, 0.0, 1.0);
        let (h, s, l) = color.to_hsl();
        assert!((h - 240.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
        
        // Gray
        let color = Color::rgb(0.5, 0.5, 0.5);
        let (h, s, l) = color.to_hsl();
        assert!((s - 0.0).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_to_rgb8() {
        let color = Color::new(0.5, 0.6, 0.7, 0.8);
        let (r, g, b) = color.to_rgb8();
        assert_eq!(r, 128);
        assert_eq!(g, 153);
        assert_eq!(b, 179);
    }

    #[test]
    fn test_color_to_rgba8() {
        let color = Color::new(0.5, 0.6, 0.7, 0.8);
        let (r, g, b, a) = color.to_rgba8();
        assert_eq!(r, 128);
        assert_eq!(g, 153);
        assert_eq!(b, 179);
        assert_eq!(a, 204);
    }

    #[test]
    fn test_color_to_hex() {
        let color = Color::from_rgb8(136, 153, 255);
        assert_eq!(color.to_hex(), "#8899ff");
    }

    #[test]
    fn test_color_to_hex_with_alpha() {
        let color = Color::from_rgba8(136, 153, 255, 204);
        assert_eq!(color.to_hex_with_alpha(), "#8899ffcc");
    }

    #[test]
    fn test_color_blend() {
        let color1 = Color::rgb(1.0, 0.0, 0.0);
        let color2 = Color::rgb(0.0, 0.0, 1.0);
        
        // 50% blend
        let blended = color1.blend(&color2, 0.5);
        assert!((blended.r - 0.5).abs() < 0.01);
        assert!((blended.g - 0.0).abs() < 0.01);
        assert!((blended.b - 0.5).abs() < 0.01);
        
        // 25% blend
        let blended = color1.blend(&color2, 0.25);
        assert!((blended.r - 0.75).abs() < 0.01);
        assert!((blended.g - 0.0).abs() < 0.01);
        assert!((blended.b - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_color_alpha_blend() {
        let foreground = Color::new(1.0, 0.0, 0.0, 0.5);
        let background = Color::rgb(0.0, 0.0, 1.0);
        
        let blended = foreground.alpha_blend(&background);
        assert!((blended.r - 0.5).abs() < 0.01);
        assert!((blended.g - 0.0).abs() < 0.01);
        assert!((blended.b - 0.5).abs() < 0.01);
        assert!((blended.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_adjust_lightness() {
        let color = Color::from_hsl(0.0, 1.0, 0.5);
        
        // Lighten
        let lightened = color.adjust_lightness(0.2);
        let (h, s, l) = lightened.to_hsl();
        assert!((h - 0.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.7).abs() < 0.01);
        
        // Darken
        let darkened = color.adjust_lightness(-0.2);
        let (h, s, l) = darkened.to_hsl();
        assert!((h - 0.0).abs() < 0.01);
        assert!((s - 1.0).abs() < 0.01);
        assert!((l - 0.3).abs() < 0.01);
    }

    #[test]
    fn test_color_adjust_saturation() {
        let color = Color::from_hsl(0.0, 0.5, 0.5);
        
        // Increase saturation
        let more_saturated = color.adjust_saturation(0.2);
        let (h, s, l) = more_saturated.to_hsl();
        assert!((h - 0.0).abs() < 0.01);
        assert!((s - 0.7).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
        
        // Decrease saturation
        let less_saturated = color.adjust_saturation(-0.2);
        let (h, s, l) = less_saturated.to_hsl();
        assert!((h - 0.0).abs() < 0.01);
        assert!((s - 0.3).abs() < 0.01);
        assert!((l - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_color_from_str() {
        // Hex format
        let color = Color::from_str("#ff0000").expect("Failed to parse #ff0000 from str");
        assert_eq!(color.to_rgba8(), (0xff, 0x00, 0x00, 0xff));
        
        // RGB format
        let color = Color::from_str("rgb(255, 0, 0)").expect("Failed to parse rgb(255,0,0) from str");
        assert_eq!(color.to_rgba8(), (0xff, 0x00, 0x00, 0xff));
        
        // RGBA format
        let color = Color::from_str("rgba(255, 0, 0, 0.5)").expect("Failed to parse rgba(255,0,0,0.5) from str");
        assert_eq!(color.to_rgb8(), (0xff, 0x00, 0x00));
        assert!((color.a - 0.5).abs() < 0.001);

        // HSL format
        let color = Color::from_str("hsl(0, 100%, 50%)").expect("Failed to parse hsl(0,100%,50%) from str");
        assert_eq!(color.to_rgba8(), (0xff, 0x00, 0x00, 0xff));

        // Test error cases for FromStr
        assert!(matches!(Color::from_str("#12G"), Err(ColorParseError::InvalidHexDigit{..})));
        assert!(matches!(Color::from_str("rgb(255,0,bad)"), Err(ColorParseError::InvalidFormat(_))));
        assert!(matches!(Color::from_str("rgba(255,0,0,bad_alpha)"), Err(ColorParseError::InvalidFormat(_))));
        assert!(matches!(Color::from_str("hsl(bad,100%,50%)"), Err(ColorParseError::InvalidFormat(_))));
        assert!(matches!(Color::from_str("unsupported"), Err(ColorParseError::InvalidFormat(_))));
    }

    #[test]
    fn test_color_display() {
        let color = Color::rgb(1.0, 0.0, 0.0);
        assert_eq!(format!("{}", color), "rgb(255, 0, 0)");
        
        let color = Color::new(1.0, 0.0, 0.0, 0.5);
        assert_eq!(format!("{}", color), "rgba(255, 0, 0, 0.5)");
    }

    #[test]
    fn test_color_interpolate() {
        let c1 = Color::rgb(0.0, 0.0, 0.0); // Black
        let c2 = Color::rgb(1.0, 1.0, 1.0); // White

        // t = 0.0
        let interpolated1 = c1.interpolate(c2, 0.0);
        assert_eq!(interpolated1.r, 0.0);
        assert_eq!(interpolated1.g, 0.0);
        assert_eq!(interpolated1.b, 0.0);
        assert_eq!(interpolated1.a, 1.0);

        // t = 0.5
        let interpolated2 = c1.interpolate(c2, 0.5);
        assert_eq!(interpolated2.r, 0.5);
        assert_eq!(interpolated2.g, 0.5);
        assert_eq!(interpolated2.b, 0.5);
        assert_eq!(interpolated2.a, 1.0);

        // t = 1.0
        let interpolated3 = c1.interpolate(c2, 1.0);
        assert_eq!(interpolated3.r, 1.0);
        assert_eq!(interpolated3.g, 1.0);
        assert_eq!(interpolated3.b, 1.0);
        assert_eq!(interpolated3.a, 1.0);

        // t < 0.0 (clamped to 0.0)
        let interpolated4 = c1.interpolate(c2, -0.5);
        assert_eq!(interpolated4.r, 0.0);
        assert_eq!(interpolated4.g, 0.0);
        assert_eq!(interpolated4.b, 0.0);

        // t > 1.0 (clamped to 1.0)
        let interpolated5 = c1.interpolate(c2, 1.5);
        assert_eq!(interpolated5.r, 1.0);
        assert_eq!(interpolated5.g, 1.0);
        assert_eq!(interpolated5.b, 1.0);

        // Interpolate with alpha
        let c3 = Color::new(1.0, 0.0, 0.0, 0.0); // Transparent Red
        let c4 = Color::new(0.0, 0.0, 1.0, 1.0); // Opaque Blue
        let interpolated_alpha = c3.interpolate(c4, 0.5);
        assert_eq!(interpolated_alpha.r, 0.5);
        assert_eq!(interpolated_alpha.g, 0.0);
        assert_eq!(interpolated_alpha.b, 0.5);
        assert_eq!(interpolated_alpha.a, 0.5);
    }

    #[test]
    fn test_color_serde_serialization() {
        let color = Color::from_rgba8(0x12, 0x34, 0x56, 0x78);
        let serialized = serde_json::to_string(&color).unwrap();
        assert_eq!(serialized, "\"#12345678\"");

        let color_opaque = Color::from_rgb8(0xAB, 0xCD, 0xEF);
        let serialized_opaque = serde_json::to_string(&color_opaque).unwrap();
        assert_eq!(serialized_opaque, "\"#abcdefFF\""); // Always with alpha
    }

    #[test]
    fn test_color_serde_deserialization() {
        let hex_valid = "\"#12345678\"";
        let deserialized: Color = serde_json::from_str(hex_valid).unwrap();
        assert_eq!(deserialized, Color::from_rgba8(0x12, 0x34, 0x56, 0x78));

        let hex_short = "\"#ABC\""; // #ABC -> #AABBCCFF
        let deserialized_short: Color = serde_json::from_str(hex_short).unwrap();
        assert_eq!(deserialized_short, Color::from_rgba8(0xAA, 0xBB, 0xCC, 0xFF));
        
        let hex_short_alpha = "\"#ABCD\""; // #ABCD -> #AABBCCDD
        let deserialized_short_alpha: Color = serde_json::from_str(hex_short_alpha).unwrap();
        assert_eq!(deserialized_short_alpha, Color::from_rgba8(0xAA, 0xBB, 0xCC, 0xDD));

        let hex_invalid_format = "\"123456\""; // Missing '#'
        let result_invalid_format: Result<Color, _> = serde_json::from_str(hex_invalid_format);
        assert!(result_invalid_format.is_err());
        if let Err(e) = result_invalid_format {
            assert!(e.to_string().contains("Invalid hex color string format"));
        }

        let hex_invalid_digit = "\"#12345G\"";
        let result_invalid_digit: Result<Color, _> = serde_json::from_str(hex_invalid_digit);
        assert!(result_invalid_digit.is_err());
         if let Err(e) = result_invalid_digit {
            assert!(e.to_string().contains("Invalid hex digit"));
        }

        let hex_invalid_length = "\"#12345\"";
        let result_invalid_length: Result<Color, _> = serde_json::from_str(hex_invalid_length);
        assert!(result_invalid_length.is_err());
        if let Err(e) = result_invalid_length {
            assert!(e.to_string().contains("Invalid hex color string length"));
        }
    }
}
