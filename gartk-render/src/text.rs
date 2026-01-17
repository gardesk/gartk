use crate::error::{RenderError, Result};
use crate::shapes::set_color;
use cairo::Context;
use gartk_core::{Color, Point, Rect, Size};
use pango::{EllipsizeMode, FontDescription, Layout, WrapMode};
use pangocairo::functions::{create_layout, show_layout};

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl From<TextAlign> for pango::Alignment {
    fn from(align: TextAlign) -> Self {
        match align {
            TextAlign::Left => pango::Alignment::Left,
            TextAlign::Center => pango::Alignment::Center,
            TextAlign::Right => pango::Alignment::Right,
        }
    }
}

/// Text rendering configuration
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f64,
    pub color: Color,
    pub align: TextAlign,
    pub ellipsize: bool,
    pub wrap: bool,
    pub max_width: Option<i32>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "sans-serif".to_string(),
            font_size: 14.0,
            color: Color::WHITE,
            align: TextAlign::Left,
            ellipsize: false,
            wrap: false,
            max_width: None,
        }
    }
}

impl TextStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }

    pub fn font_size(mut self, size: f64) -> Self {
        self.font_size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn ellipsize(mut self, value: bool) -> Self {
        self.ellipsize = value;
        self
    }

    pub fn wrap(mut self, value: bool) -> Self {
        self.wrap = value;
        self
    }

    pub fn max_width(mut self, width: i32) -> Self {
        self.max_width = Some(width);
        self
    }
}

/// Text renderer using Pango
pub struct TextRenderer {
    default_style: TextStyle,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self {
            default_style: TextStyle::default(),
        }
    }

    pub fn with_style(style: TextStyle) -> Self {
        Self {
            default_style: style,
        }
    }

    /// Create a Pango layout with the given text and style
    pub fn create_layout(&self, ctx: &Context, text: &str, style: &TextStyle) -> Layout {
        let layout = create_layout(ctx);

        // Set font
        let mut font_desc = FontDescription::new();
        font_desc.set_family(&style.font_family);
        font_desc.set_size((style.font_size * pango::SCALE as f64) as i32);
        layout.set_font_description(Some(&font_desc));

        // Set text
        layout.set_text(text);

        // Set alignment
        layout.set_alignment(style.align.into());

        // Set width constraint
        if let Some(width) = style.max_width {
            layout.set_width(width * pango::SCALE);

            if style.ellipsize {
                layout.set_ellipsize(EllipsizeMode::End);
            }

            if style.wrap {
                layout.set_wrap(WrapMode::Word);
            }
        }

        layout
    }

    /// Measure text size without rendering
    pub fn measure(&self, ctx: &Context, text: &str, style: &TextStyle) -> Size {
        let layout = self.create_layout(ctx, text, style);
        let (width, height) = layout.pixel_size();
        Size::new(width as u32, height as u32)
    }

    /// Draw text at the given position
    pub fn draw(&self, ctx: &Context, text: &str, x: f64, y: f64, style: &TextStyle) {
        let layout = self.create_layout(ctx, text, style);

        set_color(ctx, style.color);
        ctx.move_to(x, y);
        show_layout(ctx, &layout);
    }

    /// Draw text within a rectangle
    pub fn draw_in_rect(&self, ctx: &Context, text: &str, rect: Rect, style: &TextStyle) {
        let mut style = style.clone();
        style.max_width = Some(rect.width as i32);

        let layout = self.create_layout(ctx, text, &style);
        let (_, text_height) = layout.pixel_size();

        // Vertically center
        let y = rect.y as f64 + (rect.height as f64 - text_height as f64) / 2.0;

        set_color(ctx, style.color);
        ctx.move_to(rect.x as f64, y);
        show_layout(ctx, &layout);
    }

    /// Draw text centered at a point
    pub fn draw_centered(&self, ctx: &Context, text: &str, center: Point, style: &TextStyle) {
        let size = self.measure(ctx, text, style);
        let x = center.x as f64 - size.width as f64 / 2.0;
        let y = center.y as f64 - size.height as f64 / 2.0;
        self.draw(ctx, text, x, y, style);
    }

    /// Get the default style
    pub fn default_style(&self) -> &TextStyle {
        &self.default_style
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a font specification string
/// Supports formats like:
/// - "Sans 12"
/// - "Monospace:size=14"
/// - "JetBrains Mono 11"
pub fn parse_font_spec(spec: &str) -> Result<(String, f64)> {
    // Try polybar-style "Family:size=N"
    if let Some(idx) = spec.find(":size=") {
        let family = spec[..idx].trim().to_string();
        let size_str = &spec[idx + 6..];
        let size: f64 = size_str
            .parse()
            .map_err(|_| RenderError::InvalidFont(spec.to_string()))?;
        return Ok((family, size));
    }

    // Try pango-style "Family Size" or "Family Bold Italic Size"
    let parts: Vec<&str> = spec.split_whitespace().collect();
    if parts.is_empty() {
        return Err(RenderError::InvalidFont(spec.to_string()));
    }

    // Try to parse last part as size
    if let Some(last) = parts.last() {
        if let Ok(size) = last.parse::<f64>() {
            let family = parts[..parts.len() - 1].join(" ");
            return Ok((family, size));
        }
    }

    // Default size
    Ok((spec.to_string(), 12.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_font_spec() {
        let (family, size) = parse_font_spec("Sans 12").unwrap();
        assert_eq!(family, "Sans");
        assert_eq!(size, 12.0);

        let (family, size) = parse_font_spec("JetBrains Mono:size=14").unwrap();
        assert_eq!(family, "JetBrains Mono");
        assert_eq!(size, 14.0);

        let (family, size) = parse_font_spec("Monospace Bold 11").unwrap();
        assert_eq!(family, "Monospace Bold");
        assert_eq!(size, 11.0);
    }
}
