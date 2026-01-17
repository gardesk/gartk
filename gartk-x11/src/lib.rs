//! gartk-x11: X11 integration for the gartk toolkit
//!
//! This crate provides X11 functionality:
//! - [`Connection`]: X11 connection wrapper
//! - [`Window`]: X11 window creation and management
//! - [`Monitor`]: RandR monitor detection
//! - [`EventLoop`]: Blocking event loop
//! - [`CursorManager`]: Cursor creation and management

mod atoms;
mod connection;
mod cursor;
mod error;
mod event_loop;
mod keyboard;
mod monitor;
mod window;

pub use atoms::Atoms;
pub use connection::Connection;
pub use cursor::{CursorManager, CursorShape};
pub use error::{Result, X11Error};
pub use event_loop::{EventLoop, EventLoopConfig};
pub use keyboard::{key_event_from_x11, key_from_keycode, modifiers_from_x11};
pub use monitor::{detect_monitors, monitor_at_point, primary_monitor, Monitor};
pub use window::{Window, WindowConfig, WindowType};

// Re-export x11rb types that users might need
pub use x11rb::protocol::xproto::{Window as XWindow, Visualid};
