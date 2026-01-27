use crate::error::{RenderError, Result};
use cairo::{Context, Format, ImageSurface};
use gartk_core::{Color, Size};
use gartk_x11::Window;

/// Cairo image surface for rendering
pub struct Surface {
    surface: ImageSurface,
    width: u32,
    height: u32,
}

impl Surface {
    /// Create a new ARGB surface
    pub fn new(width: u32, height: u32) -> Result<Self> {
        let surface = ImageSurface::create(Format::ARgb32, width as i32, height as i32)?;
        Ok(Self {
            surface,
            width,
            height,
        })
    }

    /// Get the underlying Cairo surface
    pub fn cairo_surface(&self) -> &ImageSurface {
        &self.surface
    }

    /// Get the width
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Get the height
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Get the size
    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }

    /// Create a Cairo context for this surface
    pub fn context(&self) -> Result<Context> {
        Context::new(&self.surface).map_err(|_| RenderError::ContextCreationFailed)
    }

    /// Resize the surface (creates a new surface)
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if width != self.width || height != self.height {
            self.surface = ImageSurface::create(Format::ARgb32, width as i32, height as i32)?;
            self.width = width;
            self.height = height;
        }
        Ok(())
    }

    /// Clear the surface with a color
    pub fn clear(&self, color: Color) -> Result<()> {
        let ctx = self.context()?;
        ctx.set_source_rgba(color.r, color.g, color.b, color.a);
        ctx.set_operator(cairo::Operator::Source);
        ctx.paint()?;
        ctx.set_operator(cairo::Operator::Over);
        Ok(())
    }

    /// Flush the surface
    pub fn flush(&self) {
        self.surface.flush();
    }

    /// Get the raw image data (ARGB format, native endian)
    pub fn data(&mut self) -> Result<Vec<u8>> {
        self.surface.flush();
        let data = self
            .surface
            .data()
            .map_err(|_| RenderError::SurfaceCreationFailed)?;
        Ok(data.to_vec())
    }

    /// Access raw image data via callback (avoids copy)
    pub fn with_data<F, T>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce(&[u8]) -> T,
    {
        self.surface.flush();
        let data = self
            .surface
            .data()
            .map_err(|_| RenderError::SurfaceCreationFailed)?;
        Ok(f(&data))
    }

    /// Get the stride (bytes per row)
    pub fn stride(&self) -> i32 {
        self.surface.stride()
    }

    /// Create a surface from RGBA pixel data
    pub fn from_rgba(data: &[u8], width: u32, height: u32) -> Result<Self> {
        let mut surface = Self::new(width, height)?;
        let stride = surface.stride() as usize;
        let w = width as usize;

        // Get the surface data for writing
        {
            let mut surface_data = surface
                .surface
                .data()
                .map_err(|_| RenderError::SurfaceCreationFailed)?;

            // Convert RGBA to Cairo's ARGB format (premultiplied alpha)
            for y in 0..height as usize {
                for x in 0..w {
                    let src_idx = (y * w + x) * 4;
                    let dst_idx = y * stride + x * 4;

                    if src_idx + 3 < data.len() && dst_idx + 3 < surface_data.len() {
                        let r = data[src_idx];
                        let g = data[src_idx + 1];
                        let b = data[src_idx + 2];
                        let a = data[src_idx + 3];

                        // Cairo uses BGRA on little-endian (which is most systems)
                        // Pre-multiply alpha
                        let alpha = a as f32 / 255.0;
                        surface_data[dst_idx] = (b as f32 * alpha) as u8;     // B
                        surface_data[dst_idx + 1] = (g as f32 * alpha) as u8; // G
                        surface_data[dst_idx + 2] = (r as f32 * alpha) as u8; // R
                        surface_data[dst_idx + 3] = a;                        // A
                    }
                }
            }
        }

        surface.surface.mark_dirty();
        Ok(surface)
    }

    /// Export surface as RGBA pixel data
    pub fn to_rgba(&mut self) -> Result<Vec<u8>> {
        self.surface.flush();

        let stride = self.stride() as usize;
        let width = self.width as usize;
        let height = self.height as usize;

        let data = self
            .surface
            .data()
            .map_err(|_| RenderError::SurfaceCreationFailed)?;

        let mut rgba = vec![0u8; width * height * 4];

        // Convert Cairo's BGRA (premultiplied) to RGBA
        for y in 0..height {
            for x in 0..width {
                let src_idx = y * stride + x * 4;
                let dst_idx = (y * width + x) * 4;

                if src_idx + 3 < data.len() {
                    let b = data[src_idx];
                    let g = data[src_idx + 1];
                    let r = data[src_idx + 2];
                    let a = data[src_idx + 3];

                    // Un-premultiply alpha
                    if a > 0 {
                        let alpha = a as f32 / 255.0;
                        rgba[dst_idx] = (r as f32 / alpha).min(255.0) as u8;
                        rgba[dst_idx + 1] = (g as f32 / alpha).min(255.0) as u8;
                        rgba[dst_idx + 2] = (b as f32 / alpha).min(255.0) as u8;
                        rgba[dst_idx + 3] = a;
                    } else {
                        rgba[dst_idx] = 0;
                        rgba[dst_idx + 1] = 0;
                        rgba[dst_idx + 2] = 0;
                        rgba[dst_idx + 3] = 0;
                    }
                }
            }
        }

        Ok(rgba)
    }
}

/// Double-buffered surface for flicker-free rendering
pub struct DoubleBufferedSurface {
    front: Surface,
    back: Surface,
}

impl DoubleBufferedSurface {
    /// Create a new double-buffered surface
    pub fn new(width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            front: Surface::new(width, height)?,
            back: Surface::new(width, height)?,
        })
    }

    /// Get the back buffer for drawing
    pub fn back(&self) -> &Surface {
        &self.back
    }

    /// Get the front buffer for display
    pub fn front(&self) -> &Surface {
        &self.front
    }

    /// Swap the buffers (copy back to front)
    pub fn swap(&mut self) -> Result<()> {
        self.back.flush();
        let ctx = self.front.context()?;
        ctx.set_source_surface(&self.back.surface, 0.0, 0.0)?;
        ctx.set_operator(cairo::Operator::Source);
        ctx.paint()?;
        self.front.flush();
        Ok(())
    }

    /// Resize both buffers
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.front.resize(width, height)?;
        self.back.resize(width, height)?;
        Ok(())
    }

    /// Get the size
    pub fn size(&self) -> Size {
        self.front.size()
    }
}

/// Copy a surface to an X11 window using PutImage
pub fn copy_surface_to_window(
    surface: &mut Surface,
    window: &Window,
    gc: u32,
    x: i32,
    y: i32,
) -> Result<()> {
    use x11rb::protocol::xproto::{ConnectionExt, ImageFormat};

    surface.flush();

    let conn = window.connection();
    let data = surface.data()?;

    // X11 PutImage expects the data in a specific format
    // Cairo uses native endian ARGB, X11 expects the same for depth 32
    conn.inner()
        .put_image(
            ImageFormat::Z_PIXMAP,
            window.id(),
            gc,
            surface.width() as u16,
            surface.height() as u16,
            x as i16,
            y as i16,
            0,
            window.depth(),
            &data,
        )
        .map_err(|e| RenderError::X11(gartk_x11::X11Error::Connection(e)))?;

    conn.flush()?;
    Ok(())
}
