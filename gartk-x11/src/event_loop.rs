use crate::atoms::Atoms;
use crate::connection::Connection;
use crate::error::Result;
use crate::keyboard::{key_event_from_x11, modifiers_from_x11};
use crate::window::Window;
use gartk_core::{
    InputEvent, MouseButton, MouseEvent, Point, ScrollEvent, SelectionNotifyEvent,
    SelectionRequestEvent,
};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use x11rb::protocol::xproto::{
    self, AtomEnum, ButtonPressEvent, ConnectionExt as XprotoExt, PropMode,
};
use x11rb::protocol::Event;
use x11rb::wrapper::ConnectionExt;

/// XDND protocol version
const XDND_VERSION: u32 = 5;

/// Target frames per second for the event loop
const DEFAULT_FPS: u32 = 60;

/// Event loop configuration
#[derive(Debug, Clone)]
pub struct EventLoopConfig {
    /// Target frames per second
    pub fps: u32,
    /// Whether to redraw every frame or only on expose
    pub continuous_redraw: bool,
}

impl Default for EventLoopConfig {
    fn default() -> Self {
        Self {
            fps: DEFAULT_FPS,
            continuous_redraw: false,
        }
    }
}

/// XDND drag state
#[derive(Default)]
struct XdndState {
    /// Source window for current drag
    source: Option<xproto::Window>,
    /// Available data types from source
    types: Vec<xproto::Atom>,
    /// Last position from XdndPosition
    position: Option<(i16, i16)>,
    /// Waiting for selection data
    waiting_for_drop: bool,
}

/// Blocking event loop for a window
pub struct EventLoop {
    conn: Connection,
    atoms: Atoms,
    window_id: xproto::Window,
    config: EventLoopConfig,
    running: bool,
    needs_redraw: bool,
    frame_duration: Duration,
    xdnd_enabled: bool,
    xdnd_state: XdndState,
}

impl EventLoop {
    /// Create a new event loop for a window
    pub fn new(window: &Window, config: EventLoopConfig) -> Result<Self> {
        let conn = window.connection().clone();
        let atoms = Atoms::new(&conn)?;
        let frame_duration = Duration::from_secs_f64(1.0 / config.fps as f64);

        Ok(Self {
            conn,
            atoms,
            window_id: window.id(),
            config,
            running: true,
            needs_redraw: true,
            frame_duration,
            xdnd_enabled: false,
            xdnd_state: XdndState::default(),
        })
    }

    /// Stop the event loop
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Request a redraw
    pub fn request_redraw(&mut self) {
        self.needs_redraw = true;
    }

    /// Check if a redraw is needed
    pub fn needs_redraw(&self) -> bool {
        self.needs_redraw || self.config.continuous_redraw
    }

    /// Clear the redraw flag
    pub fn redraw_done(&mut self) {
        self.needs_redraw = false;
    }

    /// Enable XDND (drag and drop) support for this window
    pub fn enable_xdnd(&mut self) -> Result<()> {
        // Set XdndAware property with version 5
        self.conn.inner().change_property32(
            PropMode::REPLACE,
            self.window_id,
            self.atoms.xdnd_aware,
            AtomEnum::ATOM,
            &[XDND_VERSION],
        )?;
        self.conn.flush()?;
        self.xdnd_enabled = true;
        Ok(())
    }

    /// Run the event loop, calling the handler for each event
    pub fn run<F>(&mut self, mut handler: F) -> Result<()>
    where
        F: FnMut(&mut Self, InputEvent) -> Result<bool>,
    {
        let mut last_frame = Instant::now();

        while self.running {
            // Collect all pending events first for coalescing
            let mut pending_events: Vec<Event> = Vec::new();
            while let Some(event) = self.conn.poll_event()? {
                pending_events.push(event);
            }

            // Coalesce motion events - keep only the last one
            let events = coalesce_motion_events(pending_events, self.window_id);

            // Process coalesced events
            for event in events {
                if let Some(input_event) = self.translate_event(event) {
                    let should_continue = handler(self, input_event)?;
                    if !should_continue {
                        self.running = false;
                        break;
                    }
                }
            }

            if !self.running {
                break;
            }

            // Send Idle event once per frame for animations and timers
            let should_continue = handler(self, InputEvent::Idle)?;
            if !should_continue {
                self.running = false;
                break;
            }

            // Frame timing
            let now = Instant::now();
            let elapsed = now.duration_since(last_frame);

            if elapsed < self.frame_duration {
                std::thread::sleep(self.frame_duration - elapsed);
            }

            last_frame = Instant::now();
        }

        Ok(())
    }

    /// Translate X11 event to InputEvent
    fn translate_event(&mut self, event: Event) -> Option<InputEvent> {
        match event {
            Event::Expose(e) if e.window == self.window_id => {
                self.needs_redraw = true;
                Some(InputEvent::Expose)
            }

            Event::KeyPress(e) if e.event == self.window_id => {
                Some(InputEvent::Key(key_event_from_x11(&e, true)))
            }

            Event::KeyRelease(e) if e.event == self.window_id => {
                Some(InputEvent::Key(key_event_from_x11(&e, false)))
            }

            Event::ButtonPress(e) if e.event == self.window_id => {
                let button = MouseButton::from_x11(e.detail);

                // Handle scroll events
                match button {
                    MouseButton::ScrollUp
                    | MouseButton::ScrollDown
                    | MouseButton::ScrollLeft
                    | MouseButton::ScrollRight => {
                        let (dx, dy) = match button {
                            MouseButton::ScrollUp => (0, -1),
                            MouseButton::ScrollDown => (0, 1),
                            MouseButton::ScrollLeft => (-1, 0),
                            MouseButton::ScrollRight => (1, 0),
                            _ => (0, 0),
                        };
                        Some(InputEvent::Scroll(ScrollEvent {
                            position: Point::new(e.event_x as i32, e.event_y as i32),
                            delta_x: dx,
                            delta_y: dy,
                            modifiers: modifiers_from_x11(e.state),
                        }))
                    }
                    _ => Some(InputEvent::MousePress(mouse_event_from_x11(&e))),
                }
            }

            Event::ButtonRelease(e) if e.event == self.window_id => {
                let button = MouseButton::from_x11(e.detail);
                // Don't emit release for scroll buttons
                if matches!(
                    button,
                    MouseButton::ScrollUp
                        | MouseButton::ScrollDown
                        | MouseButton::ScrollLeft
                        | MouseButton::ScrollRight
                ) {
                    None
                } else {
                    Some(InputEvent::MouseRelease(MouseEvent {
                        position: Point::new(e.event_x as i32, e.event_y as i32),
                        button: Some(button),
                        modifiers: modifiers_from_x11(e.state),
                    }))
                }
            }

            Event::MotionNotify(e) if e.event == self.window_id => {
                Some(InputEvent::MouseMove(MouseEvent {
                    position: Point::new(e.event_x as i32, e.event_y as i32),
                    button: None,
                    modifiers: modifiers_from_x11(e.state),
                }))
            }

            Event::EnterNotify(e) if e.event == self.window_id => Some(InputEvent::MouseEnter(
                Point::new(e.event_x as i32, e.event_y as i32),
            )),

            Event::LeaveNotify(e) if e.event == self.window_id => Some(InputEvent::MouseLeave),

            Event::FocusIn(e) if e.event == self.window_id => Some(InputEvent::FocusIn),

            Event::FocusOut(e) if e.event == self.window_id => Some(InputEvent::FocusOut),

            Event::ConfigureNotify(e) if e.window == self.window_id => {
                self.needs_redraw = true;
                Some(InputEvent::Resize {
                    width: e.width as u32,
                    height: e.height as u32,
                })
            }

            Event::ClientMessage(e) if e.window == self.window_id => {
                // Check for WM_DELETE_WINDOW
                if e.type_ == self.atoms.wm_protocols {
                    let data = e.data.as_data32();
                    if data[0] == self.atoms.wm_delete_window {
                        return Some(InputEvent::CloseRequested);
                    }
                }

                // XDND handling
                if self.xdnd_enabled {
                    if let Some(event) = self.handle_xdnd_client_message(&e) {
                        return Some(event);
                    }
                }

                None
            }

            // Selection notify for XDND drop data
            Event::SelectionNotify(e) if e.requestor == self.window_id => {
                if self.xdnd_enabled && self.xdnd_state.waiting_for_drop {
                    if let Some(event) = self.handle_xdnd_selection_notify(&e) {
                        return Some(event);
                    }
                    return None;
                }
                Some(InputEvent::SelectionNotify(SelectionNotifyEvent {
                    requestor: e.requestor,
                    selection: e.selection,
                    target: e.target,
                    property: e.property,
                    time: e.time,
                }))
            }

            // Selection events for clipboard support
            Event::SelectionRequest(e) => {
                Some(InputEvent::SelectionRequest(SelectionRequestEvent {
                    requestor: e.requestor,
                    selection: e.selection,
                    target: e.target,
                    property: e.property,
                    time: e.time,
                }))
            }

            Event::SelectionClear(e) if e.owner == self.window_id => {
                Some(InputEvent::SelectionClear)
            }

            _ => None,
        }
    }

    /// Handle XDND client messages
    fn handle_xdnd_client_message(&mut self, e: &xproto::ClientMessageEvent) -> Option<InputEvent> {
        let data = e.data.as_data32();

        if e.type_ == self.atoms.xdnd_enter {
            // XdndEnter: source window in data[0], flags in data[1]
            // data[2..5] contain up to 3 supported types (or more if flag bit 0 set)
            self.xdnd_state.source = Some(data[0]);
            self.xdnd_state.types.clear();

            // Collect offered types (simplified: just use the 3 in the message)
            for &atom in &data[2..5] {
                if atom != 0 {
                    self.xdnd_state.types.push(atom);
                }
            }
            return None;
        }

        if e.type_ == self.atoms.xdnd_position {
            // XdndPosition: data[0] = source window, data[2] = position, data[3] = time
            let x = (data[2] >> 16) as i16;
            let y = (data[2] & 0xFFFF) as i16;
            self.xdnd_state.position = Some((x, y));

            // Send XdndStatus reply
            if let Some(source) = self.xdnd_state.source {
                let _ = self.send_xdnd_status(source, true);
            }
            return None;
        }

        if e.type_ == self.atoms.xdnd_drop {
            // XdndDrop: request the data
            self.xdnd_state.waiting_for_drop = true;

            // Request text/uri-list data
            let target = self.atoms.text_uri_list;
            let _ = self.conn.inner().convert_selection(
                self.window_id,
                self.atoms.xdnd_selection,
                target,
                self.atoms.xdnd_selection, // property to store result
                x11rb::CURRENT_TIME,
            );
            let _ = self.conn.flush();
            return None;
        }

        if e.type_ == self.atoms.xdnd_leave {
            // XdndLeave: cancel the drag
            self.xdnd_state = XdndState::default();
            return None;
        }

        None
    }

    /// Send XdndStatus message to source
    fn send_xdnd_status(&self, source: xproto::Window, accept: bool) -> Result<()> {
        let flags: u32 = if accept { 1 } else { 0 }; // bit 0 = accept

        let event = xproto::ClientMessageEvent::new(
            32,
            source,
            self.atoms.xdnd_status,
            [
                self.window_id,
                flags,
                0, // x, y of rectangle
                0, // w, h of rectangle
                self.atoms.xdnd_action_copy,
            ],
        );

        self.conn
            .inner()
            .send_event(false, source, xproto::EventMask::NO_EVENT, event)?;
        self.conn.flush()?;
        Ok(())
    }

    /// Handle SelectionNotify for XDND drop data
    fn handle_xdnd_selection_notify(
        &mut self,
        e: &xproto::SelectionNotifyEvent,
    ) -> Option<InputEvent> {
        if e.property == 0 {
            // Selection failed
            self.xdnd_state.waiting_for_drop = false;
            return None;
        }

        // Read the property containing the dropped data
        let paths = self.read_xdnd_data(e.property);

        // Send XdndFinished
        if let Some(source) = self.xdnd_state.source {
            let _ = self.send_xdnd_finished(source, !paths.is_empty());
        }

        // Reset state
        self.xdnd_state = XdndState::default();

        if paths.is_empty() {
            return None;
        }

        Some(InputEvent::FileDrop(paths))
    }

    /// Read dropped file paths from property
    fn read_xdnd_data(&self, property: xproto::Atom) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Get the property value
        let reply = match self.conn.inner().get_property(
            true, // delete after reading
            self.window_id,
            property,
            AtomEnum::ANY,
            0,
            1024 * 1024, // max length
        ) {
            Ok(cookie) => match cookie.reply() {
                Ok(reply) => reply,
                Err(_) => return paths,
            },
            Err(_) => return paths,
        };

        // Parse as text/uri-list (one URI per line)
        if let Ok(data) = std::str::from_utf8(&reply.value) {
            for line in data.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                // Convert file:// URI to path
                if let Some(path_str) = line.strip_prefix("file://") {
                    // URL decode the path
                    let decoded = urlencoded_decode(path_str);
                    paths.push(PathBuf::from(decoded));
                }
            }
        }

        paths
    }

    /// Send XdndFinished message to source
    fn send_xdnd_finished(&self, source: xproto::Window, success: bool) -> Result<()> {
        let flags: u32 = if success { 1 } else { 0 };

        let event = xproto::ClientMessageEvent::new(
            32,
            source,
            self.atoms.xdnd_finished,
            [
                self.window_id,
                flags,
                if success {
                    self.atoms.xdnd_action_copy
                } else {
                    0
                },
                0,
                0,
            ],
        );

        self.conn
            .inner()
            .send_event(false, source, xproto::EventMask::NO_EVENT, event)?;
        self.conn.flush()?;
        Ok(())
    }
}

/// Simple URL decoding for file paths
fn urlencoded_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            // Try to read two hex digits
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push('%');
            result.push_str(&hex);
        } else {
            result.push(c);
        }
    }

    result
}

fn mouse_event_from_x11(e: &ButtonPressEvent) -> MouseEvent {
    MouseEvent {
        position: Point::new(e.event_x as i32, e.event_y as i32),
        button: Some(MouseButton::from_x11(e.detail)),
        modifiers: modifiers_from_x11(e.state),
    }
}

/// Coalesce consecutive motion events for the same window, keeping only the last position.
/// This reduces lag during fast mouse movement by skipping intermediate positions.
fn coalesce_motion_events(events: Vec<Event>, window_id: xproto::Window) -> Vec<Event> {
    if events.is_empty() {
        return events;
    }

    let mut result: Vec<Event> = Vec::with_capacity(events.len());
    let mut last_motion: Option<Event> = None;

    for event in events {
        match &event {
            Event::MotionNotify(e) if e.event == window_id => {
                // Replace previous motion event with this one
                last_motion = Some(event);
            }
            _ => {
                // Flush any pending motion event before non-motion event
                if let Some(motion) = last_motion.take() {
                    result.push(motion);
                }
                result.push(event);
            }
        }
    }

    // Don't forget the last motion event
    if let Some(motion) = last_motion {
        result.push(motion);
    }

    result
}
