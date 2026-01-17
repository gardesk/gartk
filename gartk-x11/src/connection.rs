use crate::error::{Result, X11Error};
use std::sync::Arc;
use x11rb::connection::Connection as X11Connection;
use x11rb::protocol::xproto::{self, ConnectionExt, Screen};
use x11rb::rust_connection::RustConnection;

/// Wrapper around X11 connection with cached screen info
pub struct Connection {
    conn: Arc<RustConnection>,
    screen_num: usize,
    screen: Screen,
}

impl Connection {
    /// Connect to the X11 display
    pub fn connect(display: Option<&str>) -> Result<Self> {
        let (conn, screen_num) = RustConnection::connect(display)?;
        let conn = Arc::new(conn);

        let screen = conn
            .setup()
            .roots
            .get(screen_num)
            .ok_or(X11Error::InvalidScreen(screen_num))?
            .clone();

        Ok(Self {
            conn,
            screen_num,
            screen,
        })
    }

    /// Get the underlying X11 connection
    pub fn inner(&self) -> &RustConnection {
        &self.conn
    }

    /// Get the underlying X11 connection as Arc
    pub fn inner_arc(&self) -> Arc<RustConnection> {
        Arc::clone(&self.conn)
    }

    /// Get the screen number
    pub fn screen_num(&self) -> usize {
        self.screen_num
    }

    /// Get the screen info
    pub fn screen(&self) -> &Screen {
        &self.screen
    }

    /// Get the root window
    pub fn root(&self) -> xproto::Window {
        self.screen.root
    }

    /// Get the default depth
    pub fn default_depth(&self) -> u8 {
        self.screen.root_depth
    }

    /// Get the screen width
    pub fn screen_width(&self) -> u16 {
        self.screen.width_in_pixels
    }

    /// Get the screen height
    pub fn screen_height(&self) -> u16 {
        self.screen.height_in_pixels
    }

    /// Get the default visual ID
    pub fn default_visual(&self) -> xproto::Visualid {
        self.screen.root_visual
    }

    /// Get the default colormap
    pub fn default_colormap(&self) -> xproto::Colormap {
        self.screen.default_colormap
    }

    /// Generate a new X11 ID
    pub fn generate_id(&self) -> Result<u32> {
        self.conn.generate_id().map_err(X11Error::from)
    }

    /// Intern an atom
    pub fn intern_atom(&self, name: &str, only_if_exists: bool) -> Result<xproto::Atom> {
        let reply = self
            .conn
            .intern_atom(only_if_exists, name.as_bytes())?
            .reply()?;

        if reply.atom == xproto::AtomEnum::NONE.into() {
            Err(X11Error::AtomNotFound(name.to_string()))
        } else {
            Ok(reply.atom)
        }
    }

    /// Flush the connection
    pub fn flush(&self) -> Result<()> {
        self.conn.flush()?;
        Ok(())
    }

    /// Sync with the server (flush and wait for reply)
    pub fn sync(&self) -> Result<()> {
        // Use GetInputFocus as a sync point
        self.conn.get_input_focus()?.reply()?;
        Ok(())
    }

    /// Poll for an event (non-blocking)
    pub fn poll_event(&self) -> Result<Option<x11rb::protocol::Event>> {
        self.conn.poll_for_event().map_err(X11Error::from)
    }

    /// Wait for an event (blocking)
    pub fn wait_event(&self) -> Result<x11rb::protocol::Event> {
        self.conn.wait_for_event().map_err(X11Error::from)
    }

    /// Find a visual with the given depth that supports TrueColor
    pub fn find_visual(&self, depth: u8) -> Option<xproto::Visualid> {
        for allowed_depth in &self.screen.allowed_depths {
            if allowed_depth.depth == depth {
                for visual in &allowed_depth.visuals {
                    if visual.class == xproto::VisualClass::TRUE_COLOR {
                        return Some(visual.visual_id);
                    }
                }
            }
        }
        None
    }

    /// Find a 32-bit ARGB visual for transparency support
    pub fn find_argb_visual(&self) -> Option<xproto::Visualid> {
        self.find_visual(32)
    }

    /// Create a colormap for a visual
    pub fn create_colormap(&self, visual: xproto::Visualid) -> Result<xproto::Colormap> {
        let colormap = self.generate_id()?;
        self.conn.create_colormap(
            xproto::ColormapAlloc::NONE,
            colormap,
            self.root(),
            visual,
        )?;
        Ok(colormap)
    }

}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            screen_num: self.screen_num,
            screen: self.screen.clone(),
        }
    }
}
