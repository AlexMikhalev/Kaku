use async_trait::async_trait;
use promise::Future;
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Window {
    id: usize,
    xwindow: Option<crate::os::x11::XWindow>,
    connection: Option<Rc<crate::os::x11::XConnection>>,
}

impl Window {
    pub fn new(config: &crate::os::linux::Connection) -> anyhow::Result<Box<dyn crate::WindowOps>> {
        // Create X11 connection
        let xconn = crate::os::x11::XConnection::create_new()?;

        // Create X11 window
        let xwindow = crate::os::x11::XWindow::create(xconn.clone(), 800, 600)?;

        Ok(Box::new(Window {
            id: 0,
            xwindow: Some(xwindow),
            connection: Some(xconn),
        }))
    }

    pub fn create_terminal_window(
        config: &crate::os::linux::Connection,
        geometry: crate::parameters::Parameters,
    ) -> anyhow::Result<Box<dyn crate::WindowOps>> {
        let width = geometry.width.unwrap_or(800);
        let height = geometry.height.unwrap_or(600);

        // Create X11 connection
        let xconn = crate::os::x11::XConnection::create_new()?;

        // Create X11 window with specified size
        let xwindow = crate::os::x11::XWindow::create(xconn.clone(), width, height)?;

        Ok(Box::new(Window {
            id: 0,
            xwindow: Some(xwindow),
            connection: Some(xconn),
        }))
    }

    pub async fn new_window<F>(
        _class_name: &str,
        name: &str,
        geometry: crate::RequestedWindowGeometry,
        _config: Option<&config::ConfigHandle>,
        _font_config: Rc<wezterm_font::FontConfiguration>,
        event_handler: F,
    ) -> anyhow::Result<Window>
    where
        F: 'static + FnMut(crate::WindowEvent, &Window),
    {
        let width = geometry.width.0 as u32;
        let height = geometry.height.0 as u32;

        // Create X11 connection
        let xconn = crate::os::x11::XConnection::create_new()?;

        // Create X11 window
        let xwindow =
            crate::os::x11::XWindow::create(xconn.clone(), width.max(100), height.max(100))?;

        // Set window title
        let _ = xwindow.set_title(name);

        Ok(Window {
            id: 0,
            xwindow: Some(xwindow),
            connection: Some(xconn),
        })
    }
}

#[async_trait(?Send)]
impl crate::WindowOps for Window {
    fn show(&self) {
        if let Some(ref win) = self.xwindow {
            let _ = win.show();
        }
    }

    fn notify<T: Any + Send + Sync>(&self, _t: T)
    where
        Self: Sized,
    {
    }

    async fn enable_opengl(&self) -> anyhow::Result<Rc<glium::backend::Context>> {
        // For now, return error - needs EGL integration
        Err(anyhow::anyhow!("OpenGL via EGL not yet implemented"))
    }

    fn finish_frame(&self, _frame: glium::Frame) -> anyhow::Result<()> {
        Ok(())
    }

    fn hide(&self) {
        if let Some(ref win) = self.xwindow {
            let _ = win.hide();
        }
    }

    fn order_out(&self) {
        self.hide();
    }

    fn close(&self) {
        if let Some(ref win) = self.xwindow {
            let _ = win.close();
        }
    }

    fn set_cursor(&self, _cursor: Option<crate::MouseCursor>) {
        // TODO: Implement cursor
    }

    fn invalidate(&self) {
        // Trigger redraw
    }

    fn set_title(&self, title: &str) {
        if let Some(ref win) = self.xwindow {
            let _ = win.set_title(title);
        }
    }

    fn set_inner_size(&self, width: usize, height: usize) {
        if let Some(ref win) = self.xwindow {
            let _ = win.set_size(width as u32, height as u32);
        }
    }

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

// Make Connection available
pub use crate::os::linux::connection::Connection;
