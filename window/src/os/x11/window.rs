//! X11 Window implementation

use std::cell::RefCell;
use std::rc::Rc;

use crate::os::x11::connection::{XConnection, XWindowInfo};

pub struct XWindow {
    pub connection: Rc<XConnection>,
    pub window: xcb::x::Window,
    pub width: RefCell<u32>,
    pub height: RefCell<u32>,
}

impl XWindow {
    pub fn new(connection: Rc<XConnection>, window: xcb::x::Window) -> Self {
        Self {
            connection,
            window,
            width: RefCell::new(800),
            height: RefCell::new(600),
        }
    }

    pub fn create(connection: Rc<XConnection>, width: u32, height: u32) -> anyhow::Result<Self> {
        let window = connection.create_window(0, 0, width, height)?;

        let win = Self {
            connection: connection.clone(),
            window,
            width: RefCell::new(width),
            height: RefCell::new(height),
        };

        // Map the window (make it visible)
        connection.map_window(window)?;

        // Set a default title
        connection.set_window_title(window, "Kaku")?;

        Ok(win)
    }

    pub fn set_size(&self, width: u32, height: u32) -> anyhow::Result<()> {
        *self.width.borrow_mut() = width;
        *self.height.borrow_mut() = height;

        self.connection
            .configure_window(self.window, None, None, Some(width), Some(height))
    }

    pub fn set_position(&self, x: i32, y: i32) -> anyhow::Result<()> {
        self.connection
            .configure_window(self.window, Some(x), Some(y), None, None)
    }

    pub fn set_title(&self, title: &str) -> anyhow::Result<()> {
        self.connection.set_window_title(self.window, title)
    }

    pub fn show(&self) -> anyhow::Result<()> {
        self.connection.map_window(self.window)
    }

    pub fn hide(&self) -> anyhow::Result<()> {
        self.connection.unmap_window(self.window)
    }

    pub fn close(&self) -> anyhow::Result<()> {
        // Send delete window event
        let atom = self.connection.get_atom("WM_DELETE_WINDOW")?;

        let protocols = xcb::x::ClientMessageData::new32([atom.resource_id(), 0, 0, 0, 0]);

        let event = xcb::x::ClientMessageEvent::new(32, self.window, atom, protocols);

        self.connection
            .conn
            .send_event(false, self.window, xcb::x::EventMask::NO_EVENT, event);

        self.connection.flush();

        Ok(())
    }

    pub fn destroy(&self) -> anyhow::Result<()> {
        self.connection.destroy_window(self.window)
    }

    pub fn get_geometry(&self) -> anyhow::Result<(i32, i32, u32, u32)> {
        self.connection.get_window_geometry(self.window)
    }
}
