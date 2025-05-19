//! Color module for the NovaDE core layer.
//!
//! This module provides color handling utilities used throughout the
//! NovaDE desktop environment, including RGBA color representation
//! and color format conversion.

use std::fmt;
use std::str::FromStr;

/// Color format types supported by the NovaDE desktop environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    /// RGB color format (Red, Green, Blue)
    RGB,
    /// RGBA color format (Red, Green, Blue, Alpha)
    RGBA,
    /// HSL color format (Hue, Saturation, Lightness)
    HSL,
    /// HSLA color format (Hue, Saturation, Lightness, Alpha)
    HSLA,
}

/// A color in RGBA format.
///
/// This struct represents a color with red, green, blue, and alpha components,
/// each in the range [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component [0.0, 1.0]
    pub r: f32,
    /// Green component [0.0, 1.0]
    pub g: f32,
    /// Blue component [0.0, 1.0]
    pub b: f32,
    /// Alpha component [0.0, 1.0]
    pub a: f32,
}

impl Color {
    /// Creates a new color with the given RGBA components.
    ///
    /// # Arguments
    ///
    /// * `r` - Red component [0.0, 1.0]
    /// * `g` - Green component [0.0, 1.0]
    /// * `b` - Blue component [0.0, 1.0]
    /// * `a` - Alpha component [0.0, 1.0]
    ///
    /// # Returns
    ///
    /// A new `Color` with the specified components.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    /// Creates a new color with the given RGB components and alpha set to 1.0.
    ///
    /// # Arguments
    ///
    /// * `r` - Red component [0.0, 1.0]
    /// * `g` - Green component [0.0, 1.0]
    /// * `b` - Blue component [0.0, 1.0]
    ///
    /// # Returns
    ///
    /// A new `Color` with the specified RGB components and alpha set to 1.0.
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Color::new(r, g, b, 1.0)
    }

    /// Creates a new color from RGB components in the range [0, 255].
    ///
    /// # Arguments
    ///
    /// * `r` - Red component [0, 255]
    /// * `g` - Green component [0, 255]
    /// * `b` - Blue component [0, 255]
    ///
    /// # Returns
    ///
    /// A new `Color` with the specified RGB components and alpha set to 1.0.
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Color::rgb(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
        )
    }

    /// Creates a new color from RGBA components in the range [0, 255].
    ///
    /// # Arguments
    ///
    /// * `r` - Red component [0, 255]
    /// * `g` - Green component [0, 255]
    /// * `b` - Blue component [0, 255]
    /// * `a` - Alpha component [0, 255]
    ///
    /// # Returns
    ///
    /// A new `Color` with the specified RGBA components.
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color::new(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            f32::from(a) / 255.0,
        )
    }

    /// Creates a new color from a hexadecimal string.
    ///
    /// Supports formats: "#RGB", "#RGBA", "#RRGGBB", "#RRGGBBAA"
    ///
    /// # Arguments
    ///
    /// * `hex` - Hexadecimal color string
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Color` if parsing was successful,
    /// or an error message if it failed.
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        
        match hex.len() {
            3 => {
                // RGB format
                let r = u8::from_str_radix(&hex[0..1], 16)
                    .map_err(|e| format!("Invalid red component: {}", e))?;
                let g = u8::from_str_radix(&hex[1..2], 16)
                    .map_err(|e| format!("Invalid green component: {}", e))?;
                let b = u8::from_str_radix(&hex[2..3], 16)
                    .map_err(|e| format!("Invalid blue component: {}", e))?;
                
                // Expand from 4-bit to 8-bit (e.g., "a" becomes "aa")
                let r = (r << 4) | r;
                let g = (g << 4) | g;
                let b = (b << 4) | b;
                
                Ok(Color::from_rgb8(r, g, b))
            },
            4 => {
                // RGBA format
                let r = u8::from_str_radix(&hex[0..1], 16)
                    .map_err(|e| format!("Invalid red component: {}", e))?;
                let g = u8::from_str_radix(&hex[1..2], 16)
                    .map_err(|e| format!("Invalid green component: {}", e))?;
                let b = u8::from_str_radix(&hex[2..3], 16)
                    .map_err(|e| format!("Invalid blue component: {}", e))?;
                let a = u8::from_str_radix(&hex[3..4], 16)
                    .map_err(|e| format!("Invalid alpha component: {}", e))?;
                
                // Expand from 4-bit to 8-bit
                let r = (r << 4) | r;
                let g = (g << 4) | g;
                let b = (b << 4) | b;
                let a = (a << 4) | a;
                
                Ok(Color::from_rgba8(r, g, b, a))
            },
            6 => {
                // RRGGBB format
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|e| format!("Invalid red component: {}", e))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|e| format!("Invalid green component: {}", e))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|e| format!("Invalid blue component: {}", e))?;
                
                Ok(Color::from_rgb8(r, g, b))
            },
            8 => {
                // RRGGBBAA format
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|e| format!("Invalid red component: {}", e))?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|e| format!("Invalid green component: {}", e))?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|e| format!("Invalid blue component: {}", e))?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|e| format!("Invalid alpha component: {}", e))?;
                
                Ok(Color::from_rgba8(r, g, b, a))
            },
            _ => Err(format!("Invalid hex color format: {}", hex)),
        }
    }

    /// Creates a new color from HSL components.
    ///
    /// # Arguments
    ///
    /// * `h` - Hue [0.0, 360.0]
    /// * `s` - Saturation [0.0, 1.0]
    /// * `l` - Lightness [0.0, 1.0]
    ///
    /// # Returns
    ///
    /// A new `Color` converted from HSL to RGB with alpha set to 1.0.
    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let h = h % 360.0;
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

    /// Creates a new color from HSLA components.
    ///
    /// # Arguments
    ///
    /// * `h` - Hue [0.0, 360.0]
    /// * `s` - Saturation [0.0, 1.0]
    /// * `l` - Lightness [0.0, 1.0]
    /// * `a` - Alpha [0.0, 1.0]
    ///
    /// # Returns
    ///
    /// A new `Color` converted from HSLA to RGBA.
    pub fn from_hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        let mut color = Color::from_hsl(h, s, l);
        color.a = a.clamp(0.0, 1.0);
        color
    }

    /// Converts this color to HSL components.
    ///
    /// # Returns
    ///
    /// A tuple of (hue, saturation, lightness) where:
    /// - hue is in the range [0.0, 360.0]
    /// - saturation is in the range [0.0, 1.0]
    /// - lightness is in the range [0.0, 1.0]
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

    /// Converts this color to RGB components in the range [0, 255].
    ///
    /// # Returns
    ///
    /// A tuple of (red, green, blue) where each component is in the range [0, 255].
    pub fn to_rgb8(&self) -> (u8, u8, u8) {
        (
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
        )
    }

    /// Converts this color to RGBA components in the range [0, 255].
    ///
    /// # Returns
    ///
    /// A tuple of (red, green, blue, alpha) where each component is in the range [0, 255].
    pub fn to_rgba8(&self) -> (u8, u8, u8, u8) {
        (
            (self.r * 255.0).round() as u8,
            (self.g * 255.0).round() as u8,
            (self.b * 255.0).round() as u8,
            (self.a * 255.0).round() as u8,
        )
    }

    /// Converts this color to a hexadecimal string in the format "#RRGGBB".
    ///
    /// # Returns
    ///
    /// A hexadecimal color string.
    pub fn to_hex(&self) -> String {
        let (r, g, b) = self.to_rgb8();
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    /// Converts this color to a hexadecimal string in the format "#RRGGBBAA".
    ///
    /// # Returns
    ///
    /// A hexadecimal color string with alpha.
    pub fn to_hex_with_alpha(&self) -> String {
        let (r, g, b, a) = self.to_rgba8();
        format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
    }

    /// Creates a new color by blending this color with another color.
    ///
    /// # Arguments
    ///
    /// * `other` - The color to blend with
    /// * `factor` - The blend factor in the range [0.0, 1.0], where 0.0 is this color
    ///   and 1.0 is the other color
    ///
    /// # Returns
    ///
    /// A new `Color` that is a blend of this color and the other color.
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

    /// Creates a new color by applying alpha blending with a background color.
    ///
    /// # Arguments
    ///
    /// * `background` - The background color
    ///
    /// # Returns
    ///
    /// A new `Color` that is the result of alpha blending this color over the background.
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

    /// Creates a new color with adjusted lightness.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount to adjust lightness by, positive to lighten, negative to darken
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

    /// Creates a new color with adjusted saturation.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount to adjust saturation by, positive to increase, negative to decrease
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
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('#') {
            Color::from_hex(s)
        } else if s.starts_with("rgb(") && s.ends_with(')') {
            let content = &s[4..s.len() - 1];
            let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();
            
            if parts.len() != 3 {
                return Err(format!("Invalid RGB format: {}", s));
            }
            
            let r = parts[0].parse::<u8>()
                .map_err(|e| format!("Invalid red component: {}", e))?;
            let g = parts[1].parse::<u8>()
                .map_err(|e| format!("Invalid green component: {}", e))?;
            let b = parts[2].parse::<u8>()
                .map_err(|e| format!("Invalid blue component: {}", e))?;
            
            Ok(Color::from_rgb8(r, g, b))
        } else if s.starts_with("rgba(") && s.ends_with(')') {
            let content = &s[5..s.len() - 1];
            let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();
            
            if parts.len() != 4 {
                return Err(format!("Invalid RGBA format: {}", s));
            }
            
            let r = parts[0].parse::<u8>()
                .map_err(|e| format!("Invalid red component: {}", e))?;
            let g = parts[1].parse::<u8>()
                .map_err(|e| format!("Invalid green component: {}", e))?;
            let b = parts[2].parse::<u8>()
                .map_err(|e| format!("Invalid blue component: {}", e))?;
            
            let a = parts[3].parse::<f32>()
                .map_err(|e| format!("Invalid alpha component: {}", e))?;
            
            Ok(Color::new(
                f32::from(r) / 255.0,
                f32::from(g) / 255.0,
                f32::from(b) / 255.0,
                a,
            ))
        } else if s.starts_with("hsl(") && s.ends_with(')') {
            let content = &s[4..s.len() - 1];
            let parts: Vec<&str> = content.split(',').map(|p| p.trim()).collect();
            
            if parts.len() != 3 {
                return Err(format!("Invalid HSL format: {}", s));
            }
            
            let h = parts[0].parse::<f32>()
                .map_err(|e| format!("Invalid hue component: {}", e))?;
            
            let s_part = parts[1];
            let s = if s_part.ends_with('%') {
                s_part[..s_part.len() - 1].parse::<f32>()
                    .map_err(|e| format!("Invalid saturation component: {}", e))? / 100.0
            } else {
                s_part.parse::<f32>()
                    .map_err(|e| format!("Invalid saturation component: {}", e))?
            };
            
            let l_part = parts[2];
            let l = if l_part.ends_with('%') {
                l_part[..l_part.len() - 1].parse::<f32>()
                    .map_err(|e| format!("Invalid lightness component: {}", e))? / 100.0
            } else {
                l_part.parse::<f32>()
                    .map_err(|e| format!("Invalid lightness component: {}", e))?
            };
            
            Ok(Color::from_hsl(h, s, l))
        } else {
            Err(format!("Unsupported color format: {}", s))
        }
    }
}

impl fmt::Display for Color {
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

/// Helper function to convert a hue component to RGB.
///
/// # Arguments
///
/// * `p` - First parameter for the conversion
/// * `q` - Second parameter for the conversion
/// * `t` - Hue component normalized to [0, 1]
///
/// # Returns
///
/// The RGB component value in the range [0.0, 1.0].
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
    use std::str::FromStr;

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
        let color = Color::from_hex("#8af").unwrap();
        assert!((color.r - 8.0/15.0).abs() < 0.01);
        assert!((color.g - 10.0/15.0).abs() < 0.01);
        assert!((color.b - 15.0/15.0).abs() < 0.01);
        assert_eq!(color.a, 1.0);
        
        // Test #RGBA format
        let color = Color::from_hex("#8afc").unwrap();
        assert!((color.r - 8.0/15.0).abs() < 0.01);
        assert!((color.g - 10.0/15.0).abs() < 0.01);
        assert!((color.b - 15.0/15.0).abs() < 0.01);
        assert!((color.a - 12.0/15.0).abs() < 0.01);
        
        // Test #RRGGBB format
        let color = Color::from_hex("#8899ff").unwrap();
        assert!((color.r - 136.0/255.0).abs() < 0.01);
        assert!((color.g - 153.0/255.0).abs() < 0.01);
        assert_eq!(color.b, 1.0);
        assert_eq!(color.a, 1.0);
        
        // Test #RRGGBBAA format
        let color = Color::from_hex("#8899ffcc").unwrap();
        assert!((color.r - 136.0/255.0).abs() < 0.01);
        assert!((color.g - 153.0/255.0).abs() < 0.01);
        assert_eq!(color.b, 1.0);
        assert!((color.a - 204.0/255.0).abs() < 0.01);
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
        let color = Color::from_str("#ff0000").unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.0).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
        
        // RGB format
        let color = Color::from_str("rgb(255, 0, 0)").unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.0).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
        
        // RGBA format
        let color = Color::from_str("rgba(255, 0, 0, 0.5)").unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.0).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
        assert!((color.a - 0.5).abs() < 0.01);
        
        // HSL format
        let color = Color::from_str("hsl(0, 100%, 50%)").unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.0).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_color_display() {
        let color = Color::rgb(1.0, 0.0, 0.0);
        assert_eq!(format!("{}", color), "rgb(255, 0, 0)");
        
        let color = Color::new(1.0, 0.0, 0.0, 0.5);
        assert_eq!(format!("{}", color), "rgba(255, 0, 0, 0.5)");
    }
}
