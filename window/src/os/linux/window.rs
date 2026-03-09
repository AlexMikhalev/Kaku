use crate::{KeyAssignment, KeyModifiers, MouseEvent, MouseEventKind, Point, ScrollAxis, Size};
use anyhow::Result as Fallible;
use std::sync::Arc;

pub struct Window;

impl Window {
    pub fn new(_config: &crate::Connection) -> Fallible<Box<dyn crate::Window>> {
        Ok(Box::new(Window))
    }

    pub fn create_terminal_window(
        _config: &crate::Connection,
        _geometry: crate::RequestedWindowGeometry,
    ) -> Fallible<Box<dyn crate::Window>> {
        Ok(Box::new(Window))
    }
}

impl crate::Window for Window {
    fn get_window_id(&self) -> crate::WindowId {
        crate::WindowId::new(0)
    }

    fn show(&self) -> Fallible<()> {
        Ok(())
    }

    fn hide(&self) -> Fallible<()> {
        Ok(())
    }

    fn set_title(&self, _title: &str) -> Fallible<()> {
        Ok(())
    }

    fn set_inner_size(&self, _size: Size) -> Fallible<()> {
        Ok(())
    }

    fn set_position(&self, _point: Point<i32>) -> Fallible<()> {
        Ok(())
    }

    fn set_always_on_top(&self, _always_on_top: bool) -> Fallible<()> {
        Ok(())
    }

    fn set_maximized(&self, _maximized: bool) -> Fallible<()> {
        Ok(())
    }

    fn set_fullscreen(&self, _fullscreen: bool) -> Fallible<()> {
        Ok(())
    }

    fn set_focus(&self) -> Fallible<()> {
        Ok(())
    }

    fn invalidate(&self) -> Fallible<()> {
        Ok(())
    }

    fn set_cursor(&self, _cursor: crate::CursorType) -> Fallible<()> {
        Ok(())
    }

    fn set_mouse_cursor(&self, _cursor: crate::CursorType) -> Fallible<()> {
        Ok(())
    }

    fn enable_mouse_capture(&self) -> Fallible<()> {
        Ok(())
    }

    fn disable_mouse_capture(&self) -> Fallible<()> {
        Ok(())
    }

    fn toggle_mouse_capture(&self) -> Fallible<()> {
        Ok(())
    }

    fn is_mouse_captured(&self) -> bool {
        false
    }

    fn set_progress(&self, _progress: crate::WindowProgress) -> Fallible<()> {
        Ok(())
    }

    fn attention(&self, _request: crate::AttentionRequest) -> Fallible<()> {
        Ok(())
    }

    fn resizable(&self, _resizable: bool) -> Fallible<()> {
        Ok(())
    }

    fn geometry(&self) -> Fallible<crate::Rect> {
        Ok(crate::Rect::new(Point::new(0, 0), Size::new(800, 600)))
    }

    fn open_terminal(&self, _spawn: crate::Spawn) -> Fallible<crate::Pane> {
        Err(anyhow::anyhow!("Not implemented on Linux"))
    }

    fn perform_key_assignment(&self, _key_assignment: KeyAssignment) -> Fallible<()> {
        Err(anyhow::anyhow!("Not implemented on Linux"))
    }

    fn spawn_command(&self, _command: Vec<String>, _ cwd: Option<String>) -> Fallible<crate::Pane> {
        Err(anyhow::anyhow!("Not implemented on Linux"))
    }

    fn get_appearance(&self) -> Fallible<Appearance> {
        Ok(Appearance::Dark)
    }

    fn set_appearance(&self, _appearance: Appearance) -> Fallible<()> {
        Ok(())
    }

    fn reveal_files(&self, _files: Vec<std::path::PathBuf>) -> Fallible<()> {
        Ok(())
    }

    fn paste(&self, _text: &str) -> Fallible<()> {
        Ok(())
    }

    fn paste_selection(&self, _clipboard: crate::Clipboard) -> Fallible<()> {
        Ok(())
    }

    fn copy(&self) -> Fallible<()> {
        Ok(())
    }

    fn select_all(&self) -> Fallible<()> {
        Ok(())
    }

    fn input_method_change_ime(&self, _active: bool, _cursor: Point) -> Fallible<bool> {
        Ok(false)
    }

    fn current_ime_state(&self) -> (bool, bool, String) {
        (false, false, String::new())
    }

    fn is_ime_active(&self) -> bool {
        false
    }

    fn keydown(&self, _key: crate::KeyCode, _mods: KeyModifiers) -> Fallible<bool> {
        Ok(false)
    }

    fn keyup(&self, _key: crate::KeyCode, _mods: KeyModifiers) -> Fallible<bool> {
        Ok(false)
    }

    fn mouse_event(&self, _event: MouseEvent) -> Fallible<()> {
        Ok(())
    }

    fn scroll(&self, _axis: ScrollAxis, _amount: f64, _mods: KeyModifiers) -> Fallible<()> {
        Ok(())
    }

    fn close(&self) -> Fallible<()> {
        Ok(())
    }

    fn destroy(&self) -> Fallible<()> {
        Ok(())
    }

    fn set_clipboard(&self, _clipboard: crate::Clipboard, _text: String) -> Fallible<()> {
        Ok(())
    }

    fn get_clipboard(&self, _clipboard: crate::Clipboard) -> Fallible<String> {
        Ok(String::new())
    }

    fn get_selection(&self) -> Fallible<String> {
        Ok(String::new())
    }

    fn future(&self) -> std::pin::Pin<Box<dyn Future<Output = Fallible<()>> + Send>> {
        Box::pin(async { Ok(()) })
    }
}
