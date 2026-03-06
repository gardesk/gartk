use crate::connection::Connection;
use crate::error::{Result, X11Error};
use gartk_core::Rect;
use x11rb::protocol::randr::{self, ConnectionExt as RandrExt};
use x11rb::protocol::xproto::{self, ConnectionExt as XprotoExt};

/// Information about a monitor
#[derive(Debug, Clone)]
pub struct Monitor {
    /// Monitor name (e.g., "eDP-1", "HDMI-1")
    pub name: String,
    /// Monitor bounds in screen coordinates
    pub rect: Rect,
    /// Whether this is the primary monitor
    pub primary: bool,
    /// Monitor width in millimeters (for DPI calculation)
    pub width_mm: u32,
    /// Monitor height in millimeters
    pub height_mm: u32,
}

impl Monitor {
    /// Calculate horizontal DPI
    pub fn dpi_x(&self) -> f64 {
        if self.width_mm == 0 {
            96.0 // Default DPI
        } else {
            (self.rect.width as f64 * 25.4) / self.width_mm as f64
        }
    }

    /// Calculate vertical DPI
    pub fn dpi_y(&self) -> f64 {
        if self.height_mm == 0 {
            96.0
        } else {
            (self.rect.height as f64 * 25.4) / self.height_mm as f64
        }
    }

    /// Calculate average DPI
    pub fn dpi(&self) -> f64 {
        (self.dpi_x() + self.dpi_y()) / 2.0
    }

    /// Get center point of monitor
    pub fn center(&self) -> gartk_core::Point {
        self.rect.center()
    }
}

/// Detect all connected monitors using RandR
pub fn detect_monitors(conn: &Connection) -> Result<Vec<Monitor>> {
    // Check if RandR extension is available
    let randr_info = conn
        .inner()
        .randr_query_version(1, 5)
        .map_err(|_| X11Error::RandRNotAvailable)?
        .reply()
        .map_err(|_| X11Error::RandRNotAvailable)?;

    tracing::debug!(
        "RandR version: {}.{}",
        randr_info.major_version,
        randr_info.minor_version
    );

    let resources = conn
        .inner()
        .randr_get_screen_resources(conn.root())?
        .reply()?;

    let mut monitors = Vec::new();
    let mut primary_output = None;

    // Try to get primary output
    if let Ok(primary) = conn.inner().randr_get_output_primary(conn.root()) {
        if let Ok(reply) = primary.reply() {
            if reply.output != 0 {
                primary_output = Some(reply.output);
            }
        }
    }

    for output in &resources.outputs {
        let output_info = match conn
            .inner()
            .randr_get_output_info(*output, resources.config_timestamp)?
            .reply()
        {
            Ok(info) => info,
            Err(_) => continue,
        };

        // Skip disconnected outputs
        if output_info.connection != randr::Connection::CONNECTED {
            continue;
        }

        // Skip outputs without a CRTC
        if output_info.crtc == 0 {
            continue;
        }

        let crtc_info = match conn
            .inner()
            .randr_get_crtc_info(output_info.crtc, resources.config_timestamp)?
            .reply()
        {
            Ok(info) => info,
            Err(_) => continue,
        };

        // Skip disabled CRTCs
        if crtc_info.width == 0 || crtc_info.height == 0 {
            continue;
        }

        let name = String::from_utf8_lossy(&output_info.name).to_string();
        let is_primary = primary_output == Some(*output);

        monitors.push(Monitor {
            name,
            rect: Rect::new(
                crtc_info.x as i32,
                crtc_info.y as i32,
                crtc_info.width as u32,
                crtc_info.height as u32,
            ),
            primary: is_primary,
            width_mm: output_info.mm_width,
            height_mm: output_info.mm_height,
        });
    }

    // Sort monitors: primary first, then by x position
    monitors.sort_by(|a, b| {
        if a.primary != b.primary {
            b.primary.cmp(&a.primary)
        } else {
            a.rect.x.cmp(&b.rect.x)
        }
    });

    // Fallback if no monitors detected
    if monitors.is_empty() {
        tracing::warn!("No monitors detected via RandR, using screen dimensions");
        monitors.push(Monitor {
            name: "default".to_string(),
            rect: Rect::new(0, 0, conn.screen_width() as u32, conn.screen_height() as u32),
            primary: true,
            width_mm: 0,
            height_mm: 0,
        });
    }

    Ok(monitors)
}

/// Get the primary monitor, or the first one if no primary is set
pub fn primary_monitor(conn: &Connection) -> Result<Monitor> {
    let monitors = detect_monitors(conn)?;
    monitors
        .into_iter()
        .find(|m| m.primary)
        .or_else(|| detect_monitors(conn).ok()?.into_iter().next())
        .ok_or(X11Error::NoScreens)
}

/// Find the monitor containing a point
pub fn monitor_at_point(conn: &Connection, x: i32, y: i32) -> Result<Monitor> {
    let monitors = detect_monitors(conn)?;
    let point = gartk_core::Point::new(x, y);

    monitors
        .iter()
        .find(|m| m.rect.contains_point(point))
        .cloned()
        .or_else(|| monitors.into_iter().next())
        .ok_or(X11Error::NoScreens)
}

/// Get the monitor containing the mouse pointer
pub fn monitor_at_pointer(conn: &Connection) -> Result<Monitor> {
    let (x, y) = conn.query_pointer()?;
    monitor_at_point(conn, x as i32, y as i32)
}

/// Get the monitor containing the active (focused) window.
/// Reads _NET_ACTIVE_WINDOW from the root window, gets the window's geometry,
/// and finds which monitor contains its center point.
/// Cross-checks against _NET_CURRENT_DESKTOP to detect stale active windows
/// (e.g. when focus moved to an empty workspace on another monitor).
/// Falls back to monitor_at_pointer if no active window or mismatch detected.
pub fn monitor_of_active_window(conn: &Connection) -> Result<Monitor> {
    if let Ok(atom) = conn.intern_atom("_NET_ACTIVE_WINDOW", true) {
        if let Ok(reply) = conn
            .inner()
            .get_property(
                false,
                conn.root(),
                atom,
                xproto::AtomEnum::WINDOW,
                0,
                1,
            )?
            .reply()
        {
            if let Some(window_id) = reply.value32().and_then(|mut iter| iter.next()) {
                if window_id != 0 && window_id != conn.root() {
                    // Verify the window is on the current desktop.
                    // When focus moves to an empty workspace, _NET_ACTIVE_WINDOW
                    // may still point to a window on the previous workspace.
                    if let (Some(current_desktop), Some(window_desktop)) =
                        (get_cardinal_prop(conn, conn.root(), "_NET_CURRENT_DESKTOP"),
                         get_cardinal_prop(conn, window_id, "_NET_WM_DESKTOP"))
                    {
                        // 0xFFFFFFFF means "on all desktops" — always valid
                        if window_desktop != 0xFFFFFFFF && window_desktop != current_desktop {
                            return monitor_at_pointer(conn);
                        }
                    }

                    // Translate window origin to root coordinates and get size
                    if let Ok(translated) = conn
                        .inner()
                        .translate_coordinates(window_id, conn.root(), 0, 0)?
                        .reply()
                    {
                        if let Ok(geom) = conn.inner().get_geometry(window_id)?.reply() {
                            let center_x =
                                translated.dst_x as i32 + geom.width as i32 / 2;
                            let center_y =
                                translated.dst_y as i32 + geom.height as i32 / 2;
                            return monitor_at_point(conn, center_x, center_y);
                        }
                    }
                }
            }
        }
    }

    // Fallback: use pointer position
    monitor_at_pointer(conn)
}

/// Read a single CARDINAL (u32) property from a window.
fn get_cardinal_prop(conn: &Connection, window: u32, name: &str) -> Option<u32> {
    let atom = conn.intern_atom(name, true).ok()?;
    let reply = conn
        .inner()
        .get_property(false, window, atom, xproto::AtomEnum::CARDINAL, 0, 1)
        .ok()?
        .reply()
        .ok()?;
    reply.value32().and_then(|mut iter| iter.next())
}
