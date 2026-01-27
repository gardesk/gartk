//! X11 clipboard management for file operations.
//!
//! Implements the X11 selection protocol for copying/cutting files
//! so they can be pasted in other applications (file managers, etc).

use crate::atoms::Atoms;
use crate::connection::Connection;
use crate::error::Result;
use gartk_core::SelectionRequestEvent;
use std::path::PathBuf;
use x11rb::protocol::xproto::{self, Atom, ConnectionExt};
use x11rb::wrapper::ConnectionExt as WrapperConnectionExt;

/// Content stored in the clipboard.
#[derive(Debug, Clone)]
pub struct ClipboardContent {
    /// File URIs (file:///path/to/file)
    pub uris: Vec<String>,
    /// Whether this is a cut operation
    pub is_cut: bool,
}

/// Manages X11 clipboard (CLIPBOARD selection) for file operations.
pub struct ClipboardManager {
    conn: Connection,
    atoms: Atoms,
    /// Window that owns the selection
    window: xproto::Window,
    /// Current clipboard contents (if we own the selection)
    content: Option<ClipboardContent>,
    /// Timestamp when we acquired the selection
    selection_time: u32,
}

impl ClipboardManager {
    /// Create a new clipboard manager.
    ///
    /// # Arguments
    /// * `conn` - X11 connection
    /// * `window` - Window to use for selection ownership
    pub fn new(conn: Connection, window: xproto::Window) -> Result<Self> {
        let atoms = Atoms::new(&conn)?;
        Ok(Self {
            conn,
            atoms,
            window,
            content: None,
            selection_time: 0,
        })
    }

    /// Set clipboard contents with file paths.
    ///
    /// Claims ownership of the CLIPBOARD selection and stores the file paths.
    /// Other applications can then request this data via SelectionRequest.
    ///
    /// # Arguments
    /// * `paths` - File paths to copy
    /// * `is_cut` - Whether this is a cut operation (affects GNOME format)
    pub fn set_files(&mut self, paths: &[PathBuf], is_cut: bool) -> Result<()> {
        // Convert paths to file:// URIs
        let uris: Vec<String> = paths
            .iter()
            .filter_map(|p| {
                p.canonicalize().ok().map(|canonical| {
                    format!("file://{}", encode_uri_path(&canonical))
                })
            })
            .collect();

        if uris.is_empty() {
            return Ok(());
        }

        tracing::debug!("Setting clipboard with {} URIs: {:?}", uris.len(), uris);

        // Store content
        self.content = Some(ClipboardContent { uris: uris.clone(), is_cut });

        // Get current server time by making a property change
        // (CURRENT_TIME works for set_selection_owner but we need real time for TIMESTAMP)
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u32)
            .unwrap_or(0);
        self.selection_time = timestamp;

        // Claim CLIPBOARD selection ownership
        self.conn.inner().set_selection_owner(
            self.window,
            self.atoms.clipboard,
            x11rb::CURRENT_TIME,
        )?;
        self.conn.flush()?;

        // Verify we got ownership
        let owner = self.conn.inner()
            .get_selection_owner(self.atoms.clipboard)?
            .reply()?
            .owner;

        if owner != self.window {
            tracing::warn!("Failed to acquire CLIPBOARD ownership (owner={}, window={})", owner, self.window);
            self.content = None;
        } else {
            tracing::debug!("Successfully acquired CLIPBOARD ownership");
        }

        Ok(())
    }

    /// Clear clipboard contents.
    ///
    /// Called when we lose selection ownership or want to clear manually.
    pub fn clear(&mut self) {
        self.content = None;
    }

    /// Check if we currently own the clipboard and have content.
    pub fn has_content(&self) -> bool {
        self.content.is_some()
    }

    /// Check if current content is a cut operation.
    pub fn is_cut(&self) -> bool {
        self.content.as_ref().map_or(false, |c| c.is_cut)
    }

    /// Handle a SelectionRequest event from another application.
    ///
    /// Converts our clipboard content to the requested format and sends
    /// a SelectionNotify response.
    pub fn handle_selection_request(&self, event: &SelectionRequestEvent) -> Result<()> {
        let target = event.target;
        let property = if event.property == 0 {
            // Some old clients don't set property, use target as fallback
            target
        } else {
            event.property
        };

        tracing::debug!(
            "SelectionRequest: selection={}, target={}, property={}, requestor={}",
            event.selection, target, property, event.requestor
        );
        tracing::debug!(
            "Our atoms: clipboard={}, text_uri_list={}, gnome={}, utf8={}",
            self.atoms.clipboard, self.atoms.text_uri_list,
            self.atoms.gnome_copied_files, self.atoms.utf8_string
        );

        // Check if this is for CLIPBOARD selection
        if event.selection != self.atoms.clipboard {
            tracing::debug!("Not CLIPBOARD selection (expected {}, got {})", self.atoms.clipboard, event.selection);
            return self.send_selection_notify(event, xproto::AtomEnum::NONE.into());
        }

        // Check if we have content
        let content = match &self.content {
            Some(c) => c,
            None => {
                tracing::debug!("No clipboard content available");
                return self.send_selection_notify(event, xproto::AtomEnum::NONE.into());
            }
        };

        // Convert to requested format
        if target == self.atoms.targets {
            tracing::debug!("Sending TARGETS");
            self.send_targets(event, property)?;
        } else if target == self.atoms.timestamp {
            tracing::debug!("Sending TIMESTAMP: {}", self.selection_time);
            self.send_timestamp(event, property)?;
        } else if target == self.atoms.text_uri_list || target == self.atoms.uri_list {
            let data = self.format_uri_list(content);
            tracing::debug!("Sending text/uri-list: {}", data);
            self.send_data(event, property, target, data.as_bytes())?;
        } else if target == self.atoms.gnome_copied_files {
            let data = self.format_gnome_copied_files(content);
            tracing::debug!("Sending x-special/gnome-copied-files: {}", data);
            self.send_data(event, property, self.atoms.gnome_copied_files, data.as_bytes())?;
        } else if target == self.atoms.kde_cut_selection {
            // KDE cut selection: just "1" if cut, empty/absent if copy
            let data = if content.is_cut { "1" } else { "0" };
            tracing::debug!("Sending KDE cut selection: {}", data);
            self.send_data(event, property, self.atoms.kde_cut_selection, data.as_bytes())?;
        } else if target == self.atoms.utf8_string || target == self.atoms.text_plain_utf8 {
            // For UTF8_STRING, send full URIs so file managers can recognize them
            let data = self.format_uri_list_plain(content);
            tracing::debug!("Sending UTF8_STRING: {}", data);
            self.send_data(event, property, self.atoms.utf8_string, data.as_bytes())?;
        } else if target == self.atoms.text || target == self.atoms.text_plain {
            // For plain text, send full URIs
            let data = self.format_uri_list_plain(content);
            tracing::debug!("Sending TEXT: {}", data);
            self.send_data(event, property, self.atoms.text, data.as_bytes())?;
        } else if target == self.atoms.string {
            // STRING format (Latin-1, but we send UTF-8 which is compatible for ASCII)
            let data = self.format_uri_list_plain(content);
            tracing::debug!("Sending STRING: {}", data);
            self.send_data(event, property, self.atoms.string, data.as_bytes())?;
        } else {
            tracing::debug!("Unsupported clipboard target: {} (targets={}, uri_list={}, gnome={})",
                target, self.atoms.targets, self.atoms.text_uri_list, self.atoms.gnome_copied_files);
            return self.send_selection_notify(event, xproto::AtomEnum::NONE.into());
        }

        Ok(())
    }

    /// Handle SelectionClear event - we lost ownership.
    pub fn handle_selection_clear(&mut self) {
        tracing::debug!("Lost CLIPBOARD ownership");
        self.clear();
    }

    /// Get atoms reference (for event loop to check selection atom).
    pub fn atoms(&self) -> &Atoms {
        &self.atoms
    }

    /// Send TARGETS response (list of supported formats).
    fn send_targets(&self, event: &SelectionRequestEvent, property: Atom) -> Result<()> {
        // Match Dolphin's format order for maximum compatibility
        let mut targets: Vec<Atom> = vec![
            self.atoms.targets,
            self.atoms.timestamp,
            self.atoms.text_uri_list,
            self.atoms.gnome_copied_files,
        ];

        // Add KDE cut selection indicator if this is a cut operation
        if self.content.as_ref().map_or(false, |c| c.is_cut) {
            targets.push(self.atoms.kde_cut_selection);
        }

        // Add text formats (same order as Dolphin)
        targets.extend([
            self.atoms.utf8_string,
            self.atoms.text_plain_utf8,
            self.atoms.text_plain,
            self.atoms.text,
            self.atoms.string,
        ]);

        tracing::debug!(
            "TARGETS atoms: targets={}, text/uri-list={}, gnome={}, utf8={}",
            self.atoms.targets, self.atoms.text_uri_list,
            self.atoms.gnome_copied_files, self.atoms.utf8_string
        );
        tracing::debug!("Sending TARGETS list: {:?}", targets);

        self.conn.inner().change_property32(
            xproto::PropMode::REPLACE,
            event.requestor,
            property,
            xproto::AtomEnum::ATOM,
            &targets,
        )?;

        self.send_selection_notify(event, property)
    }

    /// Send TIMESTAMP response.
    fn send_timestamp(&self, event: &SelectionRequestEvent, property: Atom) -> Result<()> {
        // TIMESTAMP is sent as a 32-bit integer
        self.conn.inner().change_property32(
            xproto::PropMode::REPLACE,
            event.requestor,
            property,
            xproto::AtomEnum::INTEGER,
            &[self.selection_time],
        )?;
        self.conn.flush()?;
        self.send_selection_notify(event, property)
    }

    /// Send data in the requested format.
    fn send_data(
        &self,
        event: &SelectionRequestEvent,
        property: Atom,
        type_atom: Atom,
        data: &[u8],
    ) -> Result<()> {
        tracing::debug!(
            "send_data: property={}, type_atom={}, data_len={}, requestor={}",
            property, type_atom, data.len(), event.requestor
        );

        self.conn.inner().change_property8(
            xproto::PropMode::REPLACE,
            event.requestor,
            property,
            type_atom,
            data,
        )?;
        self.conn.flush()?;

        self.send_selection_notify(event, property)
    }

    /// Send SelectionNotify event to requestor.
    fn send_selection_notify(&self, event: &SelectionRequestEvent, property: Atom) -> Result<()> {
        tracing::debug!(
            "send_selection_notify: requestor={}, selection={}, target={}, property={}",
            event.requestor, event.selection, event.target, property
        );

        let notify = xproto::SelectionNotifyEvent {
            response_type: xproto::SELECTION_NOTIFY_EVENT,
            sequence: 0,
            time: event.time,
            requestor: event.requestor,
            selection: event.selection,
            target: event.target,
            property,
        };

        self.conn.inner().send_event(
            false,
            event.requestor,
            xproto::EventMask::NO_EVENT,
            notify,
        )?;
        self.conn.flush()?;

        tracing::debug!("SelectionNotify sent successfully");
        Ok(())
    }

    /// Format content as text/uri-list.
    ///
    /// Format: `file:///path\r\n` for each file
    fn format_uri_list(&self, content: &ClipboardContent) -> String {
        content.uris.iter()
            .map(|uri| format!("{}\r\n", uri))
            .collect()
    }

    /// Format content as x-special/gnome-copied-files.
    ///
    /// Format: `copy\nfile:///path1\nfile:///path2` (no trailing newline)
    /// or `cut\n...` for cut operations.
    fn format_gnome_copied_files(&self, content: &ClipboardContent) -> String {
        let action = if content.is_cut { "cut" } else { "copy" };
        // Join action and URIs with newlines, no trailing newline
        std::iter::once(action.to_string())
            .chain(content.uris.iter().cloned())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format content as plain text (newline-separated paths without file:// prefix).
    fn format_plain_text(&self, content: &ClipboardContent) -> String {
        content.uris.iter()
            .filter_map(|uri| uri.strip_prefix("file://"))
            .map(|path| decode_uri_path(path))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format content as newline-separated URIs (with file:// prefix).
    /// This is used for UTF8_STRING so file managers can still recognize URIs.
    fn format_uri_list_plain(&self, content: &ClipboardContent) -> String {
        content.uris.join("\n")
    }
}

/// Encode a path for use in a file:// URI.
///
/// Percent-encodes characters that are not allowed in URIs.
fn encode_uri_path(path: &std::path::Path) -> String {
    let path_str = path.to_string_lossy();
    let mut result = String::with_capacity(path_str.len() * 3);

    for byte in path_str.bytes() {
        match byte {
            // Unreserved characters (RFC 3986)
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' |
            b'-' | b'.' | b'_' | b'~' |
            // Path separators and common safe characters
            b'/' | b':' => {
                result.push(byte as char);
            }
            // Everything else gets percent-encoded
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }

    result
}

/// Decode a percent-encoded URI path.
fn decode_uri_path(encoded: &str) -> String {
    let mut result = Vec::with_capacity(encoded.len());
    let mut chars = encoded.bytes().peekable();

    while let Some(byte) = chars.next() {
        if byte == b'%' {
            // Try to read two hex digits
            let high = chars.next();
            let low = chars.next();

            if let (Some(h), Some(l)) = (high, low) {
                if let Ok(decoded) = u8::from_str_radix(
                    &format!("{}{}", h as char, l as char),
                    16,
                ) {
                    result.push(decoded);
                    continue;
                }
            }
            // Invalid encoding, keep literal
            result.push(b'%');
        } else {
            result.push(byte);
        }
    }

    String::from_utf8_lossy(&result).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_uri_path() {
        assert_eq!(
            encode_uri_path(std::path::Path::new("/home/user/file.txt")),
            "/home/user/file.txt"
        );
        assert_eq!(
            encode_uri_path(std::path::Path::new("/home/user/my file.txt")),
            "/home/user/my%20file.txt"
        );
        assert_eq!(
            encode_uri_path(std::path::Path::new("/home/user/file#1.txt")),
            "/home/user/file%231.txt"
        );
    }

    #[test]
    fn test_decode_uri_path() {
        assert_eq!(decode_uri_path("/home/user/file.txt"), "/home/user/file.txt");
        assert_eq!(decode_uri_path("/home/user/my%20file.txt"), "/home/user/my file.txt");
        assert_eq!(decode_uri_path("/home/user/file%231.txt"), "/home/user/file#1.txt");
    }
}
