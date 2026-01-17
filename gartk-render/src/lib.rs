//! gartk-render: Cairo/Pango rendering for the gartk toolkit
//!
//! This crate provides rendering functionality:
//! - [`Surface`]: Cairo surface management with double-buffering
//! - [`Renderer`]: High-level renderer combining shapes and text
//! - [`TextRenderer`]: Pango text rendering
//! - Shape primitives (rectangles, rounded rects, circles, lines)

mod error;
mod renderer;
mod shapes;
mod surface;
mod text;

pub use error::{RenderError, Result};
pub use renderer::Renderer;
pub use shapes::{
    circle_path, fill_circle, fill_rect, fill_rounded_rect, hline, line, rect_path,
    rounded_rect_path, set_color, stroke_circle, stroke_rect, stroke_rounded_rect, vline,
};
pub use surface::{copy_surface_to_window, DoubleBufferedSurface, Surface};
pub use text::{parse_font_spec, TextAlign, TextRenderer, TextStyle};

// Re-export cairo for users who need direct access
pub use cairo;
pub use pango;
pub use pangocairo;
