//! gartk-core: Core types for the gartk toolkit
//!
//! This crate provides fundamental types used throughout gartk:
//! - [`Color`]: RGBA color with parsing and manipulation
//! - [`Rect`], [`Point`], [`Size`]: Geometry types
//! - [`InputEvent`]: Input event abstraction
//! - [`Theme`]: UI theming configuration

mod color;
mod event;
mod rect;
mod theme;

pub use color::{Color, ColorError};
pub use event::{InputEvent, Key, KeyEvent, Modifiers, MouseButton, MouseEvent, ScrollEvent, SelectionRequestEvent};
pub use rect::{Edges, Point, Rect, Size};
pub use theme::{Theme, ThemeBuilder};
