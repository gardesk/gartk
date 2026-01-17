use crate::rect::Point;
use serde::{Deserialize, Serialize};

/// Keyboard modifier state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        super_key: false,
        caps_lock: false,
        num_lock: false,
    };

    pub fn is_empty(self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.super_key
    }
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    Back,
    Forward,
    Unknown(u8),
}

impl MouseButton {
    pub fn from_x11(button: u8) -> Self {
        match button {
            1 => Self::Left,
            2 => Self::Middle,
            3 => Self::Right,
            4 => Self::ScrollUp,
            5 => Self::ScrollDown,
            6 => Self::ScrollLeft,
            7 => Self::ScrollRight,
            8 => Self::Back,
            9 => Self::Forward,
            n => Self::Unknown(n),
        }
    }
}

/// Special keys (non-printable)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Key {
    Escape,
    Return,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Left,
    Right,
    Up,
    Down,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Space,
    /// A printable character
    Char(char),
    /// Unknown keycode
    Unknown(u8),
}

/// Keyboard event data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key: Key,
    pub keycode: u8,
    pub modifiers: Modifiers,
    pub pressed: bool,
}

impl KeyEvent {
    pub fn is_printable(&self) -> bool {
        matches!(self.key, Key::Char(_) | Key::Space)
    }

    pub fn char(&self) -> Option<char> {
        match self.key {
            Key::Char(c) => Some(c),
            Key::Space => Some(' '),
            _ => None,
        }
    }
}

/// Mouse event data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MouseEvent {
    pub position: Point,
    pub button: Option<MouseButton>,
    pub modifiers: Modifiers,
}

/// Scroll event data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScrollEvent {
    pub position: Point,
    pub delta_x: i32,
    pub delta_y: i32,
    pub modifiers: Modifiers,
}

/// All input event types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    /// Key pressed or released
    Key(KeyEvent),
    /// Mouse button pressed
    MousePress(MouseEvent),
    /// Mouse button released
    MouseRelease(MouseEvent),
    /// Mouse moved
    MouseMove(MouseEvent),
    /// Mouse entered window
    MouseEnter(Point),
    /// Mouse left window
    MouseLeave,
    /// Scroll wheel
    Scroll(ScrollEvent),
    /// Window needs redraw
    Expose,
    /// Window resized
    Resize { width: u32, height: u32 },
    /// Window focus gained
    FocusIn,
    /// Window focus lost
    FocusOut,
    /// Window close requested
    CloseRequested,
}

impl InputEvent {
    pub fn is_key_press(&self, key: Key) -> bool {
        matches!(self, InputEvent::Key(e) if e.pressed && e.key == key)
    }

    pub fn is_mouse_press(&self, button: MouseButton) -> bool {
        matches!(self, InputEvent::MousePress(e) if e.button == Some(button))
    }
}
