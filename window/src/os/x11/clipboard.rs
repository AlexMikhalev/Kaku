//! Minimal X11 clipboard placeholder.
//! Selection ownership and transfer protocol are deferred until the X11 backend
//! is compiling and basic runtime validation is in place.

use std::rc::Rc;

use crate::os::x11::connection::XConnection;

#[derive(Debug, Clone, Copy, Default)]
pub enum ClipboardSelection {
    #[default]
    Clipboard,
    Primary,
    Secondary,
}

pub struct X11Clipboard {
    #[allow(dead_code)]
    connection: Rc<XConnection>,
}

impl X11Clipboard {
    pub fn new(connection: Rc<XConnection>) -> Self {
        Self { connection }
    }

    /// Placeholder implementation.
    pub fn get(&self, _clipboard: ClipboardSelection) -> Option<String> {
        None
    }

    /// Placeholder implementation.
    pub fn set(&self, _clipboard: ClipboardSelection, _text: &str) -> Option<()> {
        Some(())
    }
}
