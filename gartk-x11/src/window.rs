use crate::atoms::Atoms;
use crate::connection::Connection;
use crate::error::{Result, X11Error};
use gartk_core::{Point, Rect, Size};
use x11rb::protocol::xproto::{self, ConnectionExt, EventMask};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

/// Window type hint for the window manager
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowType {
    #[default]
    Normal,
    Dialog,
    Utility,
    PopupMenu,
    Splash,
}

/// Configuration for window creation
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window title
    pub title: String,
    /// Window class (for WM_CLASS)
    pub class: String,
    /// Position (None for centered or WM-managed)
    pub position: Option<Point>,
    /// Size
    pub size: Size,
    /// Whether to use override-redirect (bypass WM)
    pub override_redirect: bool,
    /// Window type
    pub window_type: WindowType,
    /// Background color (ARGB)
    pub background: Option<u32>,
    /// Border width
    pub border_width: u32,
    /// Whether the window should be mapped immediately
    pub map_on_create: bool,
    /// Whether to request ARGB visual for transparency
    pub transparent: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: String::new(),
            class: String::new(),
            position: None,
            size: Size::new(400, 300),
            override_redirect: false,
            window_type: WindowType::Normal,
            background: None,
            border_width: 0,
            map_on_create: true,
            transparent: false,
        }
    }
}

impl WindowConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn class(mut self, class: impl Into<String>) -> Self {
        self.class = class.into();
        self
    }

    pub fn position(mut self, x: i32, y: i32) -> Self {
        self.position = Some(Point::new(x, y));
        self
    }

    pub fn centered(mut self) -> Self {
        self.position = None;
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Size::new(width, height);
        self
    }

    pub fn override_redirect(mut self, value: bool) -> Self {
        self.override_redirect = value;
        self
    }

    pub fn window_type(mut self, wtype: WindowType) -> Self {
        self.window_type = wtype;
        self
    }

    pub fn background(mut self, color: u32) -> Self {
        self.background = Some(color);
        self
    }

    pub fn border_width(mut self, width: u32) -> Self {
        self.border_width = width;
        self
    }

    pub fn map_on_create(mut self, value: bool) -> Self {
        self.map_on_create = value;
        self
    }

    pub fn transparent(mut self, value: bool) -> Self {
        self.transparent = value;
        self
    }

    /// Create a popup window configuration
    pub fn popup() -> Self {
        Self::default()
            .override_redirect(true)
            .window_type(WindowType::PopupMenu)
    }

    /// Create a dialog window configuration
    pub fn dialog() -> Self {
        Self::default().window_type(WindowType::Dialog)
    }
}

/// An X11 window
pub struct Window {
    conn: Connection,
    atoms: Atoms,
    window: xproto::Window,
    colormap: Option<xproto::Colormap>,
    visual: xproto::Visualid,
    depth: u8,
    rect: Rect,
}

impl Window {
    /// Create a new window with the given configuration
    pub fn create(conn: Connection, config: WindowConfig) -> Result<Self> {
        let atoms = Atoms::new(&conn)?;

        // Determine visual and depth
        let (visual, depth, colormap) = if config.transparent {
            if let Some(visual) = conn.find_argb_visual() {
                let colormap = conn.create_colormap(visual)?;
                (visual, 32, Some(colormap))
            } else {
                tracing::warn!("No ARGB visual found, falling back to default");
                (conn.default_visual(), conn.default_depth(), None)
            }
        } else {
            (conn.default_visual(), conn.default_depth(), None)
        };

        // Calculate position
        let position = config.position.unwrap_or_else(|| {
            // Center on screen
            Point::new(
                (conn.screen_width() as i32 - config.size.width as i32) / 2,
                (conn.screen_height() as i32 - config.size.height as i32) / 2,
            )
        });

        // Create window
        let window = conn.generate_id()?;

        let event_mask = EventMask::EXPOSURE
            | EventMask::KEY_PRESS
            | EventMask::KEY_RELEASE
            | EventMask::BUTTON_PRESS
            | EventMask::BUTTON_RELEASE
            | EventMask::POINTER_MOTION
            | EventMask::ENTER_WINDOW
            | EventMask::LEAVE_WINDOW
            | EventMask::STRUCTURE_NOTIFY
            | EventMask::FOCUS_CHANGE;

        let mut attrs = xproto::CreateWindowAux::new()
            .event_mask(event_mask)
            .override_redirect(if config.override_redirect { 1 } else { 0 })
            .border_pixel(0);

        if let Some(bg) = config.background {
            attrs = attrs.background_pixel(bg);
        }

        if let Some(cmap) = colormap {
            attrs = attrs.colormap(cmap);
        }

        conn.inner().create_window(
            depth,
            window,
            conn.root(),
            position.x as i16,
            position.y as i16,
            config.size.width as u16,
            config.size.height as u16,
            config.border_width as u16,
            xproto::WindowClass::INPUT_OUTPUT,
            visual,
            &attrs,
        )?;

        let win = Self {
            conn: conn.clone(),
            atoms: atoms.clone(),
            window,
            colormap,
            visual,
            depth,
            rect: Rect::new(
                position.x,
                position.y,
                config.size.width,
                config.size.height,
            ),
        };

        // Set window properties
        win.set_title(&config.title)?;
        win.set_class(&config.class)?;
        win.set_type(&atoms, config.window_type)?;
        win.set_protocols(&atoms)?;

        // Map window if requested
        if config.map_on_create {
            win.map()?;
        }

        conn.flush()?;

        Ok(win)
    }

    /// Get the window ID
    pub fn id(&self) -> xproto::Window {
        self.window
    }

    /// Get the connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get the visual ID
    pub fn visual(&self) -> xproto::Visualid {
        self.visual
    }

    /// Get the depth
    pub fn depth(&self) -> u8 {
        self.depth
    }

    /// Get the colormap (if custom)
    pub fn colormap(&self) -> xproto::Colormap {
        self.colormap.unwrap_or_else(|| self.conn.default_colormap())
    }

    /// Get the window bounds
    pub fn rect(&self) -> Rect {
        self.rect
    }

    /// Get the window size
    pub fn size(&self) -> Size {
        self.rect.size()
    }

    /// Map (show) the window
    pub fn map(&self) -> Result<()> {
        self.conn.inner().map_window(self.window)?;
        Ok(())
    }

    /// Unmap (hide) the window
    pub fn unmap(&self) -> Result<()> {
        self.conn.inner().unmap_window(self.window)?;
        Ok(())
    }

    /// Destroy the window
    pub fn destroy(&self) -> Result<()> {
        self.conn.inner().destroy_window(self.window)?;
        if let Some(cmap) = self.colormap {
            self.conn.inner().free_colormap(cmap)?;
        }
        Ok(())
    }

    /// Move the window
    pub fn move_to(&mut self, x: i32, y: i32) -> Result<()> {
        self.conn.inner().configure_window(
            self.window,
            &xproto::ConfigureWindowAux::new().x(x).y(y),
        )?;
        self.rect.x = x;
        self.rect.y = y;
        Ok(())
    }

    /// Resize the window
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        self.conn.inner().configure_window(
            self.window,
            &xproto::ConfigureWindowAux::new()
                .width(width)
                .height(height),
        )?;
        self.rect.width = width;
        self.rect.height = height;
        Ok(())
    }

    /// Move and resize the window
    pub fn set_geometry(&mut self, rect: Rect) -> Result<()> {
        self.conn.inner().configure_window(
            self.window,
            &xproto::ConfigureWindowAux::new()
                .x(rect.x)
                .y(rect.y)
                .width(rect.width)
                .height(rect.height),
        )?;
        self.rect = rect;
        Ok(())
    }

    /// Raise window to top
    pub fn raise(&self) -> Result<()> {
        self.conn.inner().configure_window(
            self.window,
            &xproto::ConfigureWindowAux::new().stack_mode(xproto::StackMode::ABOVE),
        )?;
        Ok(())
    }

    /// Request input focus
    pub fn focus(&self) -> Result<()> {
        self.conn.inner().set_input_focus(
            xproto::InputFocus::POINTER_ROOT,
            self.window,
            x11rb::CURRENT_TIME,
        )?;
        Ok(())
    }

    /// Set window title
    pub fn set_title(&self, title: &str) -> Result<()> {
        self.conn.inner().change_property8(
            xproto::PropMode::REPLACE,
            self.window,
            xproto::AtomEnum::WM_NAME,
            xproto::AtomEnum::STRING,
            title.as_bytes(),
        )?;
        self.conn.inner().change_property8(
            xproto::PropMode::REPLACE,
            self.window,
            self.atoms.net_wm_name,
            self.atoms.utf8_string,
            title.as_bytes(),
        )?;
        Ok(())
    }

    /// Set window class
    fn set_class(&self, class: &str) -> Result<()> {
        if class.is_empty() {
            return Ok(());
        }
        // WM_CLASS format: instance\0class\0
        let class_string = format!("{}\0{}\0", class, class);
        self.conn.inner().change_property8(
            xproto::PropMode::REPLACE,
            self.window,
            xproto::AtomEnum::WM_CLASS,
            xproto::AtomEnum::STRING,
            class_string.as_bytes(),
        )?;
        Ok(())
    }

    /// Set window type
    fn set_type(&self, atoms: &Atoms, wtype: WindowType) -> Result<()> {
        let type_atom = match wtype {
            WindowType::Normal => atoms.net_wm_window_type_normal,
            WindowType::Dialog => atoms.net_wm_window_type_dialog,
            WindowType::Utility => atoms.net_wm_window_type_utility,
            WindowType::PopupMenu => atoms.net_wm_window_type_popup_menu,
            WindowType::Splash => atoms.net_wm_window_type_splash,
        };
        self.conn.inner().change_property32(
            xproto::PropMode::REPLACE,
            self.window,
            atoms.net_wm_window_type,
            xproto::AtomEnum::ATOM,
            &[type_atom],
        )?;
        Ok(())
    }

    /// Set supported protocols (WM_DELETE_WINDOW, etc.)
    fn set_protocols(&self, atoms: &Atoms) -> Result<()> {
        self.conn.inner().change_property32(
            xproto::PropMode::REPLACE,
            self.window,
            atoms.wm_protocols,
            xproto::AtomEnum::ATOM,
            &[atoms.wm_delete_window, atoms.wm_take_focus],
        )?;
        Ok(())
    }

    /// Grab keyboard input
    pub fn grab_keyboard(&self) -> Result<()> {
        let reply = self.conn.inner().grab_keyboard(
            false,
            self.window,
            x11rb::CURRENT_TIME,
            xproto::GrabMode::ASYNC,
            xproto::GrabMode::ASYNC,
        )?.reply()?;

        if reply.status != xproto::GrabStatus::SUCCESS {
            return Err(X11Error::KeyboardGrabFailed);
        }
        Ok(())
    }

    /// Ungrab keyboard
    pub fn ungrab_keyboard(&self) -> Result<()> {
        self.conn.inner().ungrab_keyboard(x11rb::CURRENT_TIME)?;
        Ok(())
    }

    /// Grab pointer input
    pub fn grab_pointer(&self) -> Result<()> {
        let reply = self.conn.inner().grab_pointer(
            false,
            self.window,
            (EventMask::BUTTON_PRESS
                | EventMask::BUTTON_RELEASE
                | EventMask::POINTER_MOTION)
                .into(),
            xproto::GrabMode::ASYNC,
            xproto::GrabMode::ASYNC,
            self.window,
            0u32,
            x11rb::CURRENT_TIME,
        )?.reply()?;

        if reply.status != xproto::GrabStatus::SUCCESS {
            return Err(X11Error::PointerGrabFailed);
        }
        Ok(())
    }

    /// Ungrab pointer
    pub fn ungrab_pointer(&self) -> Result<()> {
        self.conn.inner().ungrab_pointer(x11rb::CURRENT_TIME)?;
        Ok(())
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        let _ = self.destroy();
    }
}
