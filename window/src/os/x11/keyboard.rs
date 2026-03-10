//! X11 Keyboard handling with xkbcommon

use xkbcommon::xkb::{Context, Keymap, State};

const KEYCODE_OFFSET: u32 = 8;

pub struct Keyboard {
    context: Context,
    keymap: Option<Keymap>,
    state: Option<State>,
    keymap_str: String,
}

impl Keyboard {
    pub fn new() -> anyhow::Result<Self> {
        let context = Context::new(xkbcommon::xkb::CONTEXT_NO_FLAGS);

        // Try to load keymap from X11 server
        let keymap_str = String::new();

        let mut keyboard = Self {
            context,
            keymap: None,
            state: None,
            keymap_str,
        };

        // Try to compile a default keymap
        keyboard.try_compile_keymap()?;

        Ok(keyboard)
    }

    fn try_compile_keymap(&mut self) -> anyhow::Result<()> {
        // Try common layouts
        let layouts = ["us", "gb", "de", "fr", "es"];

        for layout in layouts {
            if let Ok(keymap) =
                self.context
                    .new_keymap_from_names(None, Some(layout), None, None, None, true)
            {
                let state = State::new(&keymap);
                self.keymap = Some(keymap);
                self.state = state;
                return Ok(());
            }
        }

        // Fallback: try empty (system default)
        if let Ok(keymap) = self
            .context
            .new_keymap_from_names(None, None, None, None, None, true)
        {
            let state = State::new(&keymap);
            self.keymap = Some(keymap);
            self.state = state;
        }

        Ok(())
    }

    /// Update keymap from X11 (for dynamic layout changes)
    pub fn update_from_x11(&mut self, keymap_str: &str) -> anyhow::Result<()> {
        match self.context.new_keymap_from_string(keymap_str) {
            Ok(keymap) => {
                let state = State::new(&keymap);
                self.keymap = Some(keymap);
                self.state = Some(state);
                self.keymap_str = keymap_str.to_string();
                Ok(())
            }
            Err(e) => {
                log::warn!("Failed to update keymap from X11: {:?}", e);
                Ok(())
            }
        }
    }

    /// Process a key press event
    pub fn key_press(&mut self, keycode: u32) -> Option<(&str, xkbcommon::xkb::KeyDirection)> {
        let state = self.state.as_mut()?;

        // xcb keycodes are 8-255, xkb expects 8-255
        let keycode = keycode + KEYCODE_OFFSET;

        let direction = state.update_key(keycode, xkbcommon::xkb::KeyDirection::Down);

        if let Some(keysym) = state.lookup_keysym(keycode, 0) {
            let name = keysym.name();
            if name != "NoSymbol" {
                return Some((name, direction));
            }
        }

        None
    }

    /// Process a key release event
    pub fn key_release(&mut self, keycode: u32) -> Option<(&str, xkbcommon::xkb::KeyDirection)> {
        let state = self.state.as_mut()?;

        let keycode = keycode + KEYCODE_OFFSET;

        let direction = state.update_key(keycode, xkbcommon::xkb::KeyDirection::Up);

        if let Some(keysym) = state.lookup_keysym(keycode, 0) {
            let name = keysym.name();
            if name != "NoSymbol" {
                return Some((name, direction));
            }
        }

        None
    }

    /// Get the current modifier state
    pub fn get_modifiers(&self) -> xkbcommon::xkb::ModMask {
        self.state.as_ref().map(|s| s.s_mods_active()).unwrap_or(0)
    }

    /// Check if a modifier is active
    pub fn is_mod_active(&self, modifier: xkbcommon::xkb::ModIndex) -> bool {
        self.state
            .as_ref()
            .map(|s| s.mod_index_is_active(modifier, xkbcommon::xkb::StateFlags::EMPTY))
            .unwrap_or(false)
    }

    /// Check if Control is active
    pub fn is_ctrl_active(&self) -> bool {
        self.is_mod_active(xkbcommon::xkb::ModIndex::new(0)) // Control
    }

    /// Check if Shift is active
    pub fn is_shift_active(&self) -> bool {
        self.is_mod_active(xkbcommon::xkb::ModIndex::new(1)) // Shift
    }

    /// Check if Alt is active
    pub fn is_alt_active(&self) -> bool {
        self.is_mod_active(xkbcommon::xkb::ModIndex::new(3)) // Mod1 (usually Alt)
    }

    /// Check if Super/Win is active
    pub fn is_super_active(&self) -> bool {
        self.is_mod_active(xkbcommon::xkb::ModIndex::new(6)) // Mod4 (Super/Win)
    }
}

impl Default for Keyboard {
    fn default() -> Self {
        Self::new().expect("Failed to create keyboard")
    }
}
