use async_trait::async_trait;
use promise::Future;
use std::any::Any;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Window {
    id: usize,
}

impl Window {
    pub fn new(
        _config: &crate::os::linux::Connection,
    ) -> anyhow::Result<Box<dyn crate::WindowOps>> {
        Ok(Box::new(Window { id: 0 }))
    }

    pub fn create_terminal_window(
        _config: &crate::os::linux::Connection,
        _geometry: crate::parameters::Parameters,
    ) -> anyhow::Result<Box<dyn crate::WindowOps>> {
        Ok(Box::new(Window { id: 0 }))
    }

    pub async fn new_window<F>(
        _class_name: &str,
        _name: &str,
        _geometry: crate::RequestedWindowGeometry,
        _config: Option<&config::ConfigHandle>,
        _font_config: Rc<wezterm_font::FontConfiguration>,
        _event_handler: F,
    ) -> anyhow::Result<Window>
    where
        F: 'static + FnMut(crate::WindowEvent, &Window),
    {
        Ok(Window { id: 0 })
    }
}

#[async_trait(?Send)]
impl crate::WindowOps for Window {
    fn show(&self) {}

    fn notify<T: Any + Send + Sync>(&self, _t: T)
    where
        Self: Sized,
    {
    }

    async fn enable_opengl(&self) -> anyhow::Result<Rc<glium::backend::Context>> {
        Err(anyhow::anyhow!("OpenGL not implemented on Linux"))
    }

    fn finish_frame(&self, _frame: glium::Frame) -> anyhow::Result<()> {
        Ok(())
    }

    fn hide(&self) {}

    fn order_out(&self) {}

    fn close(&self) {}

    fn set_cursor(&self, _cursor: Option<crate::MouseCursor>) {}

    fn invalidate(&self) {}

    fn set_title(&self, _title: &str) {}

    fn set_inner_size(&self, _width: usize, _height: usize) {}

    fn set_maximize_button_position(&self, _rect: crate::ScreenRect) {}

    fn request_drag_move(&self) {}

    fn set_window_drag_position(&self, _coords: crate::ScreenPoint) {}

    fn set_window_position(&self, _coords: crate::ScreenPoint) {}

    fn set_text_cursor_position(&self, _cursor: crate::Rect) {}

    fn get_clipboard(&self, _clipboard: crate::Clipboard) -> Future<String> {
        Future::ok(String::new())
    }

    fn get_clipboard_data(&self, _clipboard: crate::Clipboard) -> Future<crate::ClipboardData> {
        Future::ok(crate::ClipboardData::Text(String::new()))
    }

    fn set_clipboard(&self, _clipboard: crate::Clipboard, _text: String) {}

    fn set_window_level(&self, _level: config::window::WindowLevel) {}

    fn set_icon(&self, _image: crate::Image) {}

    fn maximize(&self) {}

    fn restore(&self) {}

    fn focus(&self) {}

    fn toggle_fullscreen(&self) {}

    fn config_did_change(&self, _config: &config::ConfigHandle) {}

    fn is_zoom_animation_active(&self) -> bool {
        false
    }

    fn set_resize_increments(&self, _incr: crate::ResizeIncrement) {}

    fn get_os_parameters(
        &self,
        _config: &config::ConfigHandle,
        _window_state: crate::WindowState,
    ) -> anyhow::Result<Option<crate::os::parameters::Parameters>> {
        Ok(None)
    }
}

impl raw_window_handle::HasWindowHandle for Window {
    fn window_handle(
        &self,
    ) -> Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
        Err(raw_window_handle::HandleError::Unavailable)
    }
}

impl raw_window_handle::HasDisplayHandle for Window {
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Err(raw_window_handle::HandleError::Unavailable)
    }
}

// Make Connection available
pub use crate::os::linux::connection::Connection;
