use crate::connection::Connection;
use crate::error::Result;
use x11rb::protocol::xproto::Atom;

/// Cached X11 atoms for common operations
#[derive(Debug, Clone)]
pub struct Atoms {
    // Window manager hints
    pub wm_protocols: Atom,
    pub wm_delete_window: Atom,
    pub wm_take_focus: Atom,
    pub wm_name: Atom,
    pub wm_class: Atom,
    pub wm_state: Atom,

    // EWMH atoms
    pub net_wm_name: Atom,
    pub net_wm_window_type: Atom,
    pub net_wm_window_type_normal: Atom,
    pub net_wm_window_type_dialog: Atom,
    pub net_wm_window_type_utility: Atom,
    pub net_wm_window_type_popup_menu: Atom,
    pub net_wm_window_type_splash: Atom,
    pub net_wm_state: Atom,
    pub net_wm_state_above: Atom,
    pub net_wm_state_sticky: Atom,
    pub net_wm_state_fullscreen: Atom,
    pub net_wm_state_focused: Atom,
    pub net_active_window: Atom,
    pub net_wm_pid: Atom,
    pub net_frame_extents: Atom,

    // Clipboard
    pub clipboard: Atom,
    pub primary: Atom,
    pub targets: Atom,
    pub utf8_string: Atom,
    pub text: Atom,
    pub text_uri_list: Atom,
    pub gnome_copied_files: Atom,
    pub text_plain: Atom,
    pub text_plain_utf8: Atom,
    pub multiple: Atom,
    pub incr: Atom,
    pub kde_cut_selection: Atom,
    pub uri_list: Atom,
    pub timestamp: Atom,

    // Misc
    pub cardinal: Atom,
    pub string: Atom,
    pub atom: Atom,
    pub window: Atom,
}

impl Atoms {
    /// Intern all commonly used atoms
    pub fn new(conn: &Connection) -> Result<Self> {
        Ok(Self {
            // WM hints
            wm_protocols: conn.intern_atom("WM_PROTOCOLS", false)?,
            wm_delete_window: conn.intern_atom("WM_DELETE_WINDOW", false)?,
            wm_take_focus: conn.intern_atom("WM_TAKE_FOCUS", false)?,
            wm_name: conn.intern_atom("WM_NAME", false)?,
            wm_class: conn.intern_atom("WM_CLASS", false)?,
            wm_state: conn.intern_atom("WM_STATE", false)?,

            // EWMH
            net_wm_name: conn.intern_atom("_NET_WM_NAME", false)?,
            net_wm_window_type: conn.intern_atom("_NET_WM_WINDOW_TYPE", false)?,
            net_wm_window_type_normal: conn.intern_atom("_NET_WM_WINDOW_TYPE_NORMAL", false)?,
            net_wm_window_type_dialog: conn.intern_atom("_NET_WM_WINDOW_TYPE_DIALOG", false)?,
            net_wm_window_type_utility: conn.intern_atom("_NET_WM_WINDOW_TYPE_UTILITY", false)?,
            net_wm_window_type_popup_menu: conn
                .intern_atom("_NET_WM_WINDOW_TYPE_POPUP_MENU", false)?,
            net_wm_window_type_splash: conn.intern_atom("_NET_WM_WINDOW_TYPE_SPLASH", false)?,
            net_wm_state: conn.intern_atom("_NET_WM_STATE", false)?,
            net_wm_state_above: conn.intern_atom("_NET_WM_STATE_ABOVE", false)?,
            net_wm_state_sticky: conn.intern_atom("_NET_WM_STATE_STICKY", false)?,
            net_wm_state_fullscreen: conn.intern_atom("_NET_WM_STATE_FULLSCREEN", false)?,
            net_wm_state_focused: conn.intern_atom("_NET_WM_STATE_FOCUSED", false)?,
            net_active_window: conn.intern_atom("_NET_ACTIVE_WINDOW", false)?,
            net_wm_pid: conn.intern_atom("_NET_WM_PID", false)?,
            net_frame_extents: conn.intern_atom("_NET_FRAME_EXTENTS", false)?,

            // Clipboard
            clipboard: conn.intern_atom("CLIPBOARD", false)?,
            primary: conn.intern_atom("PRIMARY", false)?,
            targets: conn.intern_atom("TARGETS", false)?,
            utf8_string: conn.intern_atom("UTF8_STRING", false)?,
            text: conn.intern_atom("TEXT", false)?,
            text_uri_list: conn.intern_atom("text/uri-list", false)?,
            gnome_copied_files: conn.intern_atom("x-special/gnome-copied-files", false)?,
            text_plain: conn.intern_atom("text/plain", false)?,
            text_plain_utf8: conn.intern_atom("text/plain;charset=utf-8", false)?,
            multiple: conn.intern_atom("MULTIPLE", false)?,
            incr: conn.intern_atom("INCR", false)?,
            kde_cut_selection: conn.intern_atom("application/x-kde-cutselection", false)?,
            uri_list: conn.intern_atom("x-special/URI", false)?,
            timestamp: conn.intern_atom("TIMESTAMP", false)?,

            // Types
            cardinal: conn.intern_atom("CARDINAL", false)?,
            string: conn.intern_atom("STRING", false)?,
            atom: conn.intern_atom("ATOM", false)?,
            window: conn.intern_atom("WINDOW", false)?,
        })
    }
}
