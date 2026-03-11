use crate::{Clipboard, ClipboardData};
use arboard::{Clipboard as SystemClipboard, GetExtLinux, LinuxClipboardKind, SetExtLinux};

fn kind(clipboard: Clipboard) -> LinuxClipboardKind {
    match clipboard {
        Clipboard::Clipboard => LinuxClipboardKind::Clipboard,
        Clipboard::PrimarySelection => LinuxClipboardKind::Primary,
    }
}

pub fn get(clipboard: Clipboard) -> ClipboardData {
    match SystemClipboard::new()
        .and_then(|mut clipboard_ctx| clipboard_ctx.get().clipboard(kind(clipboard)).text())
    {
        Ok(text) => ClipboardData::Text(text),
        Err(err) => {
            log::warn!("failed to read linux clipboard {:?}: {err:#}", clipboard);
            ClipboardData::Text(String::new())
        }
    }
}

pub fn set(clipboard: Clipboard, text: String) {
    if let Err(err) = SystemClipboard::new()
        .and_then(|mut clipboard_ctx| clipboard_ctx.set().clipboard(kind(clipboard)).text(text))
    {
        log::warn!("failed to set linux clipboard {:?}: {err:#}", clipboard);
    }
}
