use crate::connection::Connection;
use crate::error::Result;
use x11rb::protocol::xproto::{self, ConnectionExt, Cursor, Font};

/// Standard cursor shapes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CursorShape {
    /// Default arrow cursor
    Default,
    /// Text input cursor (I-beam)
    Text,
    /// Hand/pointer cursor for clickable elements
    Pointer,
    /// Wait/busy cursor
    Wait,
    /// Crosshair cursor
    Crosshair,
    /// Move cursor (four arrows)
    Move,
    /// Resize cursors
    ResizeN,
    ResizeS,
    ResizeE,
    ResizeW,
    ResizeNE,
    ResizeNW,
    ResizeSE,
    ResizeSW,
    /// Not allowed cursor
    NotAllowed,
}

impl CursorShape {
    /// Get the X11 cursor font glyph index
    fn glyph(self) -> u16 {
        // These are from the cursor font (see X11/cursorfont.h)
        match self {
            CursorShape::Default => 68,     // XC_left_ptr
            CursorShape::Text => 152,       // XC_xterm
            CursorShape::Pointer => 60,     // XC_hand2
            CursorShape::Wait => 150,       // XC_watch
            CursorShape::Crosshair => 34,   // XC_crosshair
            CursorShape::Move => 52,        // XC_fleur
            CursorShape::ResizeN => 138,    // XC_top_side
            CursorShape::ResizeS => 16,     // XC_bottom_side
            CursorShape::ResizeE => 96,     // XC_right_side
            CursorShape::ResizeW => 70,     // XC_left_side
            CursorShape::ResizeNE => 136,   // XC_top_right_corner
            CursorShape::ResizeNW => 134,   // XC_top_left_corner
            CursorShape::ResizeSE => 14,    // XC_bottom_right_corner
            CursorShape::ResizeSW => 12,    // XC_bottom_left_corner
            CursorShape::NotAllowed => 0,   // XC_X_cursor
        }
    }
}

/// Manages cursor creation and caching
pub struct CursorManager {
    conn: Connection,
    cursor_font: Font,
    cursors: std::collections::HashMap<CursorShape, Cursor>,
}

impl CursorManager {
    /// Create a new cursor manager
    pub fn new(conn: Connection) -> Result<Self> {
        let cursor_font = conn.generate_id()?;
        conn.inner()
            .open_font(cursor_font, b"cursor")?;

        Ok(Self {
            conn,
            cursor_font,
            cursors: std::collections::HashMap::new(),
        })
    }

    /// Get or create a cursor for the given shape
    pub fn get(&mut self, shape: CursorShape) -> Result<Cursor> {
        if let Some(&cursor) = self.cursors.get(&shape) {
            return Ok(cursor);
        }

        let cursor = self.conn.generate_id()?;
        let glyph = shape.glyph();

        self.conn.inner().create_glyph_cursor(
            cursor,
            self.cursor_font,
            self.cursor_font,
            glyph,
            glyph + 1,
            0, 0, 0,       // foreground RGB (black)
            0xFFFF, 0xFFFF, 0xFFFF, // background RGB (white)
        )?;

        self.cursors.insert(shape, cursor);
        Ok(cursor)
    }

    /// Set the cursor for a window
    pub fn set_window_cursor(
        &mut self,
        window: xproto::Window,
        shape: CursorShape,
    ) -> Result<()> {
        let cursor = self.get(shape)?;
        self.conn.inner().change_window_attributes(
            window,
            &xproto::ChangeWindowAttributesAux::new().cursor(cursor),
        )?;
        Ok(())
    }
}

impl Drop for CursorManager {
    fn drop(&mut self) {
        // Free all cursors
        for &cursor in self.cursors.values() {
            let _ = self.conn.inner().free_cursor(cursor);
        }
        // Close the cursor font
        let _ = self.conn.inner().close_font(self.cursor_font);
    }
}
