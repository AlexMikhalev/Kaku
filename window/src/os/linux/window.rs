use async_trait::async_trait;
use std::any::Any;
use std::rc::Rc;

pub struct Window;

impl Window {
    pub fn new(
        _config: &crate::os::linux::Connection,
    ) -> anyhow::Result<Box<dyn crate::WindowOps>> {
        Ok(Box::new(Window))
    }

    pub fn create_terminal_window(
        _config: &crate::os::linux::Connection,
        _geometry: crate::parameters::Parameters,
    ) -> anyhow::Result<Box<dyn crate::WindowOps>> {
        Ok(Box::new(Window))
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

    fn get_clipboard(&self, _clipboard: crate::Clipboard) -> promise::Future<String> {
        Box::pin(async { Ok(String::new()) })
    }

    fn get_clipboard_data(
        &self,
        _clipboard: crate::Clipboard,
    ) -> promise::Future<crate::ClipboardData> {
        Box::pin(async { Ok(crate::ClipboardData::try_from(String::new()).unwrap_or_default()) })
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

// Make Connection available
pub use crate::os::linux::connection::Connection;
