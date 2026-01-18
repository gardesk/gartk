use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ColorError {
    #[error("invalid hex color format: {0}")]
    InvalidHex(String),
    #[error("invalid rgb/rgba format: {0}")]
    InvalidRgb(String),
    #[error("unknown color name: {0}")]
    UnknownName(String),
}

/// RGBA color with components in 0.0-1.0 range
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub const fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self::new(r, g, b, 1.0)
    }

    /// Create from 0-255 integer components
    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            a as f64 / 255.0,
        )
    }

    /// Parse hex color: #RGB, #RRGGBB, #RRGGBBAA
    pub fn from_hex(s: &str) -> Result<Self, ColorError> {
        let s = s.trim_start_matches('#');

        match s.len() {
            3 => {
                // #RGB -> #RRGGBB
                let r = u8::from_str_radix(&s[0..1], 16)
                    .map_err(|_| ColorError::InvalidHex(s.to_string()))?;
                let g = u8::from_str_radix(&s[1..2], 16)
                    .map_err(|_| ColorError::InvalidHex(s.to_string()))?;
                let b = u8::from_str_radix(&s[2..3], 16)
                    .map_err(|_| ColorError::InvalidHex(s.to_string()))?;
                Ok(Self::from_u8(r * 17, g * 17, b * 17, 255))
            }
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16)
                    .map_err(|_| ColorError::InvalidHex(s.to_string()))?;
                let g = u8::from_str_radix(&s[2..4], 16)
                    .map_err(|_| ColorError::InvalidHex(s.to_string()))?;
                let b = u8::from_str_radix(&s[4..6], 16)
                    .map_err(|_| ColorError::InvalidHex(s.to_string()))?;
                Ok(Self::from_u8(r, g, b, 255))
            }
            8 => {
                let r = u8::from_str_radix(&s[0..2], 16)
                    .map_err(|_| ColorError::InvalidHex(s.to_string()))?;
                let g = u8::from_str_radix(&s[2..4], 16)
                    .map_err(|_| ColorError::InvalidHex(s.to_string()))?;
                let b = u8::from_str_radix(&s[4..6], 16)
                    .map_err(|_| ColorError::InvalidHex(s.to_string()))?;
                let a = u8::from_str_radix(&s[6..8], 16)
                    .map_err(|_| ColorError::InvalidHex(s.to_string()))?;
                Ok(Self::from_u8(r, g, b, a))
            }
            _ => Err(ColorError::InvalidHex(s.to_string())),
        }
    }

    /// Parse rgb(r, g, b) or rgba(r, g, b, a) format
    pub fn from_rgb_str(s: &str) -> Result<Self, ColorError> {
        let s = s.trim();

        if let Some(inner) = s.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() != 4 {
                return Err(ColorError::InvalidRgb(s.to_string()));
            }
            let r: u8 = parts[0]
                .parse()
                .map_err(|_| ColorError::InvalidRgb(s.to_string()))?;
            let g: u8 = parts[1]
                .parse()
                .map_err(|_| ColorError::InvalidRgb(s.to_string()))?;
            let b: u8 = parts[2]
                .parse()
                .map_err(|_| ColorError::InvalidRgb(s.to_string()))?;
            let a: f64 = parts[3]
                .parse()
                .map_err(|_| ColorError::InvalidRgb(s.to_string()))?;
            Ok(Self::from_u8(r, g, b, (a * 255.0) as u8))
        } else if let Some(inner) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
            let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
            if parts.len() != 3 {
                return Err(ColorError::InvalidRgb(s.to_string()));
            }
            let r: u8 = parts[0]
                .parse()
                .map_err(|_| ColorError::InvalidRgb(s.to_string()))?;
            let g: u8 = parts[1]
                .parse()
                .map_err(|_| ColorError::InvalidRgb(s.to_string()))?;
            let b: u8 = parts[2]
                .parse()
                .map_err(|_| ColorError::InvalidRgb(s.to_string()))?;
            Ok(Self::from_u8(r, g, b, 255))
        } else {
            Err(ColorError::InvalidRgb(s.to_string()))
        }
    }

    /// Parse any color format: hex, rgb(), rgba(), or named
    pub fn parse(s: &str) -> Result<Self, ColorError> {
        let s = s.trim();

        if s.starts_with('#') {
            Self::from_hex(s)
        } else if s.starts_with("rgb") {
            Self::from_rgb_str(s)
        } else {
            Self::from_name(s)
        }
    }

    /// Get color by name
    pub fn from_name(name: &str) -> Result<Self, ColorError> {
        match name.to_lowercase().as_str() {
            "black" => Ok(Self::BLACK),
            "white" => Ok(Self::WHITE),
            "red" => Ok(Self::RED),
            "green" => Ok(Self::GREEN),
            "blue" => Ok(Self::BLUE),
            "yellow" => Ok(Self::YELLOW),
            "cyan" => Ok(Self::CYAN),
            "magenta" => Ok(Self::MAGENTA),
            "transparent" => Ok(Self::TRANSPARENT),
            "gray" | "grey" => Ok(Self::GRAY),
            "darkgray" | "darkgrey" => Ok(Self::DARK_GRAY),
            "lightgray" | "lightgrey" => Ok(Self::LIGHT_GRAY),
            _ => Err(ColorError::UnknownName(name.to_string())),
        }
    }

    /// Return color with modified alpha
    pub const fn with_alpha(self, a: f64) -> Self {
        Self::new(self.r, self.g, self.b, a)
    }

    /// Lighten the color by a factor (0.0-1.0)
    pub fn lighten(self, factor: f64) -> Self {
        Self::new(
            (self.r + (1.0 - self.r) * factor).min(1.0),
            (self.g + (1.0 - self.g) * factor).min(1.0),
            (self.b + (1.0 - self.b) * factor).min(1.0),
            self.a,
        )
    }

    /// Darken the color by a factor (0.0-1.0)
    pub fn darken(self, factor: f64) -> Self {
        Self::new(
            (self.r * (1.0 - factor)).max(0.0),
            (self.g * (1.0 - factor)).max(0.0),
            (self.b * (1.0 - factor)).max(0.0),
            self.a,
        )
    }

    /// Convert RGB to HSV color space.
    /// Returns (hue: 0-360, saturation: 0-1, value: 0-1)
    pub fn to_hsv(&self) -> (f64, f64, f64) {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let delta = max - min;

        let h = if delta == 0.0 {
            0.0
        } else if max == self.r {
            60.0 * (((self.g - self.b) / delta) % 6.0)
        } else if max == self.g {
            60.0 * (((self.b - self.r) / delta) + 2.0)
        } else {
            60.0 * (((self.r - self.g) / delta) + 4.0)
        };
        let h = if h < 0.0 { h + 360.0 } else { h };
        let s = if max == 0.0 { 0.0 } else { delta / max };

        (h, s, max)
    }

    /// Create color from HSV values.
    /// hue: 0-360, saturation: 0-1, value: 0-1
    pub fn from_hsv(h: f64, s: f64, v: f64) -> Self {
        Self::from_hsva(h, s, v, 1.0)
    }

    /// Create color from HSVA values.
    /// hue: 0-360, saturation: 0-1, value: 0-1, alpha: 0-1
    pub fn from_hsva(h: f64, s: f64, v: f64, a: f64) -> Self {
        let c = v * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h_prime as u32 % 6 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Self::new(r + m, g + m, b + m, a)
    }

    /// Return color with modified hue (HSV)
    pub fn with_hue(self, h: f64) -> Self {
        let (_, s, v) = self.to_hsv();
        Self::from_hsva(h, s, v, self.a)
    }

    /// Return color with modified saturation (HSV)
    pub fn with_saturation(self, s: f64) -> Self {
        let (h, _, v) = self.to_hsv();
        Self::from_hsva(h, s, v, self.a)
    }

    /// Return color with modified value/brightness (HSV)
    pub fn with_value(self, v: f64) -> Self {
        let (h, s, _) = self.to_hsv();
        Self::from_hsva(h, s, v, self.a)
    }

    /// Convert to hex string (e.g., "#ff6600" or "#ff660080" with alpha)
    pub fn to_hex(&self) -> String {
        let r = (self.r * 255.0).round() as u8;
        let g = (self.g * 255.0).round() as u8;
        let b = (self.b * 255.0).round() as u8;
        if self.a >= 1.0 {
            format!("#{:02x}{:02x}{:02x}", r, g, b)
        } else {
            let a = (self.a * 255.0).round() as u8;
            format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
        }
    }

    /// Convert to ARGB u32 for X11 (alpha in high byte)
    pub fn to_argb_u32(self) -> u32 {
        let a = (self.a * 255.0) as u32;
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        (a << 24) | (r << 16) | (g << 8) | b
    }

    /// Convert to RGB u32 for X11 (no alpha)
    pub fn to_rgb_u32(self) -> u32 {
        let r = (self.r * 255.0) as u32;
        let g = (self.g * 255.0) as u32;
        let b = (self.b * 255.0) as u32;
        (r << 16) | (g << 8) | b
    }

    // Named color constants
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    pub const YELLOW: Self = Self::rgb(1.0, 1.0, 0.0);
    pub const CYAN: Self = Self::rgb(0.0, 1.0, 1.0);
    pub const MAGENTA: Self = Self::rgb(1.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const GRAY: Self = Self::rgb(0.5, 0.5, 0.5);
    pub const DARK_GRAY: Self = Self::rgb(0.25, 0.25, 0.25);
    pub const LIGHT_GRAY: Self = Self::rgb(0.75, 0.75, 0.75);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

impl FromStr for Color {
    type Err = ColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_parsing() {
        assert_eq!(Color::from_hex("#fff").unwrap(), Color::WHITE);
        assert_eq!(Color::from_hex("#000000").unwrap(), Color::BLACK);
        assert_eq!(
            Color::from_hex("#ff0000").unwrap(),
            Color::rgb(1.0, 0.0, 0.0)
        );
        assert_eq!(
            Color::from_hex("#00ff0080").unwrap(),
            Color::new(0.0, 1.0, 0.0, 128.0 / 255.0)
        );
    }

    #[test]
    fn test_rgb_parsing() {
        assert_eq!(
            Color::from_rgb_str("rgb(255, 0, 0)").unwrap(),
            Color::rgb(1.0, 0.0, 0.0)
        );
        assert_eq!(
            Color::from_rgb_str("rgba(0, 255, 0, 0.5)").unwrap(),
            Color::new(0.0, 1.0, 0.0, 0.5)
        );
    }

    #[test]
    fn test_named_colors() {
        assert_eq!(Color::from_name("black").unwrap(), Color::BLACK);
        assert_eq!(Color::from_name("WHITE").unwrap(), Color::WHITE);
    }
}
