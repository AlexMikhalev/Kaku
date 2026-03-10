//! X11 Clipboard handling

use std::collections::HashMap;
use std::sync::Arc;

use xcb::x::{Atom, Property};

use crate::os::x11::connection::XConnection;

pub struct X11Clipboard {
    connection: Arc<XConnection>,
}

impl X11Clipboard {
    pub fn new(connection: Arc<XConnection>) -> Self {
        Self { connection }
    }

    /// Get clipboard text
    pub fn get(&self, clipboard: ClipboardSelection) -> Option<String> {
        let atom = match clipboard {
            ClipboardSelection::Clipboard => "CLIPBOARD",
            ClipboardSelection::Primary => "PRIMARY",
            ClipboardSelection::Secondary => "SECONDARY",
        };

        let selection_atom = self.connection.get_atom(atom).ok()?;
        let target_atom = self.connection.get_atom("UTF8_STRING").ok()?;

        // Request selection
        let window = self.connection.generate_id();

        // Create a temporary window for selection
        let _ = self.connection.create_window(0, 0, 1, 1);

        // Convert selection request
        let convert_req = xcb::x::ConvertSelection {
            selection: selection_atom,
            target: target_atom,
            property: xcb::x::Atom::from_reply(&self.connection.conn.send_request(
                &xcb::x::InternAtom {
                    name: b"VTK_SELECTION",
                    only_if_exists: false,
                },
            ))
            .atom(),
            requestor: window,
        };

        self.connection.conn.send_request(&convert_req);
        self.connection.flush();

        // TODO: Handle selection notify event
        // This is simplified - real implementation needs event loop integration

        None
    }

    /// Set clipboard text
    pub fn set(&self, clipboard: ClipboardSelection, text: &str) -> Option<()> {
        let atom = match clipboard {
            ClipboardSelection::Clipboard => "CLIPBOARD",
            ClipboardSelection::Primary => "PRIMARY",
            ClipboardSelection::Secondary => "SECONDARY",
        };

        let selection_atom = self.connection.get_atom(atom).ok()?;
        let target_atom = self.connection.get_atom("UTF8_STRING").ok()?;

        // Create a window to own the selection
        let window = self.connection.generate_id();

        // Set the selection
        let set_req = xcb::x::SetSelectionOwner {
            owner: window,
            selection: selection_atom,
            time: xcb::x::CURRENT_TIME,
        };

        self.connection.conn.send_request(&set_req);
        self.connection.flush();

        // Store the data (simplified - real impl needs to respond to selection requests)
        log::debug!("Set clipboard: {} bytes", text.len());

        Some(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ClipboardSelection {
    Clipboard,
    Primary,
    Secondary,
}

impl Default for ClipboardSelection {
    fn default() -> Self {
        ClipboardSelection::Clipboard
    }
}
