use crate::Clipboard;

pub struct LinuxClipboard;

impl LinuxClipboard {
    pub fn new() -> Self {
        Self
    }

    pub fn get(&self, _clipboard: Clipboard) -> Option<String> {
        None
    }

    pub fn put(&self, _clipboard: Clipboard, _text: String) -> anyhow::Result<()> {
        Ok(())
    }
}
