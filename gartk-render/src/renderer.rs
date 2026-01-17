use crate::error::Result;
use crate::shapes;
use crate::surface::Surface;
use crate::text::{TextRenderer, TextStyle};
use cairo::Context;
use gartk_core::{Color, Point, Rect, Size, Theme};

/// High-level renderer that combines surface, shapes, and text rendering
pub struct Renderer {
    surface: Surface,
    text_renderer: TextRenderer,
    theme: Theme,
}

impl Renderer {
    /// Create a new renderer with the given size
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            surface: Surface::new(width, height)?,
            text_renderer: TextRenderer::new(),
            theme: Theme::default(),
        })
    }

    /// Create a new renderer with a theme
    pub fn with_theme(width: u32, height: u32, theme: Theme) -> Result<Self> {
        let default_style = TextStyle::new()
            .font_family(&theme.font_family)
            .font_size(theme.font_size)
            .color(theme.foreground);

        Ok(Self {
            surface: Surface::new(width, height)?,
            text_renderer: TextRenderer::with_style(default_style),
            theme,
        })
    }

    /// Get the underlying surface
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    /// Get the theme
    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Get the size
    pub fn size(&self) -> Size {
        self.surface.size()
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.surface.resize(width, height)
    }

    /// Get a Cairo context for custom drawing
    pub fn context(&self) -> Result<Context> {
        self.surface.context()
    }

    /// Clear with the theme background color
    pub fn clear(&self) -> Result<()> {
        self.surface.clear(self.theme.background)
    }

    /// Clear with a specific color
    pub fn clear_color(&self, color: Color) -> Result<()> {
        self.surface.clear(color)
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&self, rect: Rect, color: Color) -> Result<()> {
        let ctx = self.context()?;
        shapes::fill_rect(&ctx, rect, color);
        Ok(())
    }

    /// Draw a rounded rectangle
    pub fn fill_rounded_rect(&self, rect: Rect, radius: f64, color: Color) -> Result<()> {
        let ctx = self.context()?;
        shapes::fill_rounded_rect(&ctx, rect, radius, color);
        Ok(())
    }

    /// Draw a stroked rectangle
    pub fn stroke_rect(&self, rect: Rect, color: Color, line_width: f64) -> Result<()> {
        let ctx = self.context()?;
        shapes::stroke_rect(&ctx, rect, color, line_width);
        Ok(())
    }

    /// Draw a stroked rounded rectangle
    pub fn stroke_rounded_rect(
        &self,
        rect: Rect,
        radius: f64,
        color: Color,
        line_width: f64,
    ) -> Result<()> {
        let ctx = self.context()?;
        shapes::stroke_rounded_rect(&ctx, rect, radius, color, line_width);
        Ok(())
    }

    /// Draw a line
    pub fn line(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Color,
        line_width: f64,
    ) -> Result<()> {
        let ctx = self.context()?;
        shapes::line(&ctx, x1, y1, x2, y2, color, line_width);
        Ok(())
    }

    /// Draw a filled circle
    pub fn fill_circle(&self, cx: f64, cy: f64, radius: f64, color: Color) -> Result<()> {
        let ctx = self.context()?;
        shapes::fill_circle(&ctx, cx, cy, radius, color);
        Ok(())
    }

    /// Draw text at a position
    pub fn text(&self, text: &str, x: f64, y: f64, style: &TextStyle) -> Result<()> {
        let ctx = self.context()?;
        self.text_renderer.draw(&ctx, text, x, y, style);
        Ok(())
    }

    /// Draw text with the default style
    pub fn text_default(&self, text: &str, x: f64, y: f64, color: Color) -> Result<()> {
        let ctx = self.context()?;
        let style = TextStyle::new()
            .font_family(&self.theme.font_family)
            .font_size(self.theme.font_size)
            .color(color);
        self.text_renderer.draw(&ctx, text, x, y, &style);
        Ok(())
    }

    /// Draw text within a rectangle
    pub fn text_in_rect(&self, text: &str, rect: Rect, style: &TextStyle) -> Result<()> {
        let ctx = self.context()?;
        self.text_renderer.draw_in_rect(&ctx, text, rect, style);
        Ok(())
    }

    /// Draw text centered at a point
    pub fn text_centered(&self, text: &str, center: Point, style: &TextStyle) -> Result<()> {
        let ctx = self.context()?;
        self.text_renderer.draw_centered(&ctx, text, center, style);
        Ok(())
    }

    /// Measure text size
    pub fn measure_text(&self, text: &str, style: &TextStyle) -> Result<Size> {
        let ctx = self.context()?;
        Ok(self.text_renderer.measure(&ctx, text, style))
    }

    /// Flush the surface
    pub fn flush(&self) {
        self.surface.flush();
    }

    /// Get the text renderer for advanced text operations
    pub fn text_renderer(&self) -> &TextRenderer {
        &self.text_renderer
    }
}
