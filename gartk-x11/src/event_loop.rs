use crate::atoms::Atoms;
use crate::connection::Connection;
use crate::error::Result;
use crate::keyboard::{key_event_from_x11, modifiers_from_x11};
use crate::window::Window;
use gartk_core::{InputEvent, MouseButton, MouseEvent, Point, ScrollEvent, SelectionRequestEvent};
use std::time::{Duration, Instant};
use x11rb::protocol::xproto::{self, ButtonPressEvent};
use x11rb::protocol::Event;

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

/// Blocking event loop for a window
pub struct EventLoop {
    conn: Connection,
    atoms: Atoms,
    window_id: xproto::Window,
    config: EventLoopConfig,
    running: bool,
    needs_redraw: bool,
    frame_duration: Duration,
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

            Event::EnterNotify(e) if e.event == self.window_id => {
                Some(InputEvent::MouseEnter(Point::new(
                    e.event_x as i32,
                    e.event_y as i32,
                )))
            }

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
                None
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
