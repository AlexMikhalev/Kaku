use crate::WindowOps;
use crate::connection::ConnectionOps;
use anyhow::Context;
use async_trait::async_trait;
use promise::Future;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle,
    RawWindowHandle, WindowHandle, XcbDisplayHandle, XcbWindowHandle,
};
use std::any::Any;
use std::cell::RefCell;
use std::ffi::c_void;
use std::num::NonZeroU32;
use std::ptr::NonNull;
use std::rc::Rc;
use xcb::Xid;

const DEFAULT_WINDOW_WIDTH: u32 = 800;
const DEFAULT_WINDOW_HEIGHT: u32 = 600;

fn default_window_size() -> (u32, u32) {
    (DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
}

pub(crate) struct WindowInner {
    pub xwindow: crate::os::x11::XWindow,
    pub events: crate::WindowEventSender,
    pub should_close: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct Window {
    id: usize,
}

impl Window {
    pub(crate) fn from_id(id: usize) -> Self {
        Self { id }
    }

    fn create_with_event_sender(
        width: u32,
        height: u32,
        mut events: crate::WindowEventSender,
    ) -> anyhow::Result<Self> {
        let conn = crate::Connection::get()
            .ok_or_else(|| anyhow::anyhow!("connection not initialized"))?;
        let window_id = conn.next_window_id();
        let xwindow = crate::os::x11::XWindow::create(conn.x11()?, width, height)
            .context("create X11 window backing object")?;
        let window = Self { id: window_id };
        events.assign_window(window.clone());

        conn.windows.borrow_mut().insert(
            window_id,
            Rc::new(RefCell::new(WindowInner {
                xwindow,
                events,
                should_close: false,
            })),
        );

        Ok(window)
    }

    fn with_window_inner<R>(&self, f: impl FnOnce(&WindowInner) -> R) -> Option<R> {
        let conn = crate::Connection::get()?;
        let handle = conn.window_by_id(self.id)?;
        let inner = handle.try_borrow().ok()?;
        Some(f(&inner))
    }

    fn with_window_inner_mut<R>(&self, f: impl FnOnce(&mut WindowInner) -> R) -> Option<R> {
        let conn = crate::Connection::get()?;
        let handle = conn.window_by_id(self.id)?;
        let mut inner = handle.try_borrow_mut().ok()?;
        Some(f(&mut inner))
    }

    pub fn new(
        _config: &crate::os::linux::Connection,
    ) -> anyhow::Result<Box<dyn crate::WindowOps>> {
        let (width, height) = default_window_size();
        Ok(Box::new(Self::create_with_event_sender(
            width,
            height,
            crate::WindowEventSender::new(|_, _| {}),
        )?))
    }

    pub fn create_terminal_window(
        _config: &crate::os::linux::Connection,
        _geometry: crate::os::parameters::Parameters,
    ) -> anyhow::Result<Box<dyn crate::WindowOps>> {
        let (width, height) = default_window_size();
        Ok(Box::new(Self::create_with_event_sender(
            width,
            height,
            crate::WindowEventSender::new(|_, _| {}),
        )?))
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
        let conn = crate::Connection::get()
            .ok_or_else(|| anyhow::anyhow!("connection not initialized"))?;
        let resolved = conn.resolve_geometry(geometry);
        let width = resolved.width.max(1) as u32;
        let height = resolved.height.max(1) as u32;
        let window = Self::create_with_event_sender(
            width,
            height,
            crate::WindowEventSender::new(event_handler),
        )
        .context("create Linux X11 window")?;
        if let (Some(x), Some(y)) = (resolved.x, resolved.y) {
            window.set_window_position(crate::ScreenPoint::new(x as isize, y as isize));
        }
        window.set_title(name);
        Ok(window)
    }
}

impl HasDisplayHandle for Window {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let conn = crate::Connection::get().ok_or(HandleError::Unavailable)?;
        let x11 = conn.x11().map_err(|_| HandleError::Unavailable)?;
        let raw_conn =
            NonNull::new(x11.conn.get_raw_conn().cast()).ok_or(HandleError::Unavailable)?;
        let handle = XcbDisplayHandle::new(Some(raw_conn), x11.default_screen);
        unsafe { Ok(DisplayHandle::borrow_raw(RawDisplayHandle::Xcb(handle))) }
    }
}

impl HasWindowHandle for Window {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let (window_id, visual_id) = self
            .with_window_inner(|inner| {
                (
                    inner.xwindow.window.resource_id(),
                    inner.xwindow.connection.root_visual,
                )
            })
            .ok_or(HandleError::Unavailable)?;

        let mut handle =
            XcbWindowHandle::new(NonZeroU32::new(window_id).ok_or(HandleError::Unavailable)?);
        handle.visual_id = NonZeroU32::new(visual_id);
        unsafe { Ok(WindowHandle::borrow_raw(RawWindowHandle::Xcb(handle))) }
    }
}

#[async_trait(?Send)]
impl crate::WindowOps for Window {
    fn show(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.xwindow.show()?;
            inner.events.dispatch(crate::WindowEvent::NeedRepaint);
            Ok(())
        });
    }

    fn notify<T: Any + Send + Sync>(&self, t: T)
    where
        Self: Sized,
    {
        Connection::with_window_inner(self.id, move |inner| {
            inner
                .events
                .dispatch(crate::WindowEvent::Notification(Box::new(t)));
            Ok(())
        });
    }

    async fn enable_opengl(&self) -> anyhow::Result<Rc<glium::backend::Context>> {
        let window = self
            .with_window_inner(|inner| inner.xwindow.window.resource_id())
            .ok_or_else(|| anyhow::anyhow!("invalid window"))? as usize
            as *const c_void;
        let backend = Rc::new(crate::egl::GlState::create(None, window)?);
        let behavior = if cfg!(debug_assertions) {
            glium::debug::DebugCallbackBehavior::DebugMessageOnError
        } else {
            glium::debug::DebugCallbackBehavior::Ignore
        };
        let context = unsafe { glium::backend::Context::new(Rc::clone(&backend), true, behavior) }?;
        Ok(context)
    }

    fn finish_frame(&self, _frame: glium::Frame) -> anyhow::Result<()> {
        _frame.finish()?;
        Ok(())
    }

    fn hide(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.xwindow.hide()?;
            Ok(())
        });
    }

    fn order_out(&self) {
        self.hide();
    }

    fn close(&self) {
        let id = self.id;
        promise::spawn::spawn_into_main_thread(async move {
            if let Some(conn) = crate::Connection::get() {
                if let Some(handle) = conn.window_by_id(id) {
                    let mut inner = handle.borrow_mut();
                    inner.should_close = true;
                    let _ = inner.xwindow.close();
                }
            }
        })
        .detach();
    }

    fn set_cursor(&self, _cursor: Option<crate::MouseCursor>) {}

    fn invalidate(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.events.dispatch(crate::WindowEvent::NeedRepaint);
            Ok(())
        });
    }

    fn set_title(&self, title: &str) {
        let title = title.to_string();
        Connection::with_window_inner(self.id, move |inner| {
            inner.xwindow.set_title(&title)?;
            Ok(())
        });
    }

    fn set_inner_size(&self, width: usize, height: usize) {
        Connection::with_window_inner(self.id, move |inner| {
            inner.xwindow.set_size(width as u32, height as u32)?;
            inner
                .events
                .dispatch(crate::WindowEvent::SetInnerSizeCompleted);
            Ok(())
        });
    }

    fn set_maximize_button_position(&self, _rect: crate::ScreenRect) {}

    fn request_drag_move(&self) {}

    fn set_window_drag_position(&self, _coords: crate::ScreenPoint) {}

    fn set_window_position(&self, coords: crate::ScreenPoint) {
        Connection::with_window_inner(self.id, move |inner| {
            inner
                .xwindow
                .set_position(coords.x as i32, coords.y as i32)?;
            Ok(())
        });
    }

    fn set_text_cursor_position(&self, _cursor: crate::Rect) {}

    fn get_clipboard(&self, clipboard: crate::Clipboard) -> Future<String> {
        let text = match crate::os::linux::clipboard::get(clipboard) {
            crate::ClipboardData::Text(text) => text,
            crate::ClipboardData::Files(_) => String::new(),
        };
        Future::ok(text)
    }

    fn get_clipboard_data(&self, clipboard: crate::Clipboard) -> Future<crate::ClipboardData> {
        Future::ok(crate::os::linux::clipboard::get(clipboard))
    }

    fn set_clipboard(&self, clipboard: crate::Clipboard, text: String) {
        crate::os::linux::clipboard::set(clipboard, text);
    }

    fn set_window_level(&self, _level: config::window::WindowLevel) {}

    fn set_icon(&self, _image: crate::Image) {}

    fn maximize(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.xwindow.maximize()?;
            Ok(())
        });
    }

    fn restore(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.xwindow.restore()?;
            Ok(())
        });
    }

    fn focus(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.xwindow.focus()?;
            Ok(())
        });
    }

    fn toggle_fullscreen(&self) {
        Connection::with_window_inner(self.id, |inner| {
            inner.xwindow.toggle_fullscreen()?;
            Ok(())
        });
    }

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

pub use crate::os::linux::connection::Connection;
