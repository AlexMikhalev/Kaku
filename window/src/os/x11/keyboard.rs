//! Minimal X11 keyboard placeholder.
//! Full xkbcommon integration is deferred until the X11 backend compiles cleanly.

#[derive(Debug, Default, Clone)]
pub struct Keyboard;

impl Keyboard {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self)
    }

    /// Placeholder hook for future X11 keymap updates.
    pub fn update_from_x11(&mut self, _keymap_str: &str) -> anyhow::Result<()> {
        Ok(())
    }

    /// For now we only expose the raw keycode back to higher layers.
    pub fn key_press(&mut self, keycode: u32) -> Option<u32> {
        Some(keycode)
    }

    /// For now we only expose the raw keycode back to higher layers.
    pub fn key_release(&mut self, keycode: u32) -> Option<u32> {
        Some(keycode)
    }

    pub fn get_modifiers(&self) -> u32 {
        0
    }

    pub fn is_mod_active(&self, _modifier: u32) -> bool {
        false
    }

    pub fn is_ctrl_active(&self) -> bool {
        false
    }

    pub fn is_shift_active(&self) -> bool {
        false
    }

    pub fn is_alt_active(&self) -> bool {
        false
    }

    pub fn is_super_active(&self) -> bool {
        false
    }
}
