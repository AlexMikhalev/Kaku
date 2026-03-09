use crate::{KeyCode, KeyModifiers, MouseEventKind};

pub fn keycode_from_key_event(
    _keyval: u32,
    _keycode: u16,
    _state: u32,
) -> Option<(KeyCode, KeyModifiers)> {
    None
}

pub fn key_event_from_keycode(
    _keycode: KeyCode,
    _mods: KeyModifiers,
    _keymap: Option<*mut std::ffi::c_void>,
) -> Option<(u32, u32)> {
    None
}

pub fn mouse_event_kind_from_button(_button: u32) -> MouseEventKind {
    MouseEventKind::Up
}

pub fn mouse_event_kind_from_scroll(_delta: f64) -> (ScrollDirection, i32) {
    (ScrollDirection::Up, 0)
}

#[derive(Clone, Copy, Debug)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}
