//! X11 window implementation

use anyhow::Context;
use std::cell::RefCell;
use std::num::NonZeroU32;
use std::rc::Rc;

use crate::os::x11::connection::XConnection;
use xcb::Xid;

const NET_WM_STATE_REMOVE: u32 = 0;
const NET_WM_STATE_ADD: u32 = 1;
const NET_WM_STATE_TOGGLE: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct XcbWindowHandleData {
    pub window: NonZeroU32,
    pub visual_id: Option<NonZeroU32>,
}

#[derive(Debug)]
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
        let window = connection
            .create_simple_window(0, 0, width as u16, height as u16)
            .context("create X11 toplevel window")?;

        let win = Self {
            connection: connection.clone(),
            window,
            width: RefCell::new(width),
            height: RefCell::new(height),
        };

        connection.map_window(window).context("map X11 window")?;
        connection
            .set_window_title(window, "Kaku")
            .context("set initial X11 window title")?;

        Ok(win)
    }

    pub fn set_size(&self, width: u32, height: u32) -> anyhow::Result<()> {
        *self.width.borrow_mut() = width;
        *self.height.borrow_mut() = height;

        self.connection.configure_window(
            self.window,
            None,
            None,
            Some(width.min(u16::MAX as u32) as u16),
            Some(height.min(u16::MAX as u32) as u16),
        )
    }

    pub fn set_position(&self, x: i32, y: i32) -> anyhow::Result<()> {
        self.connection.configure_window(
            self.window,
            Some(x.clamp(i16::MIN as i32, i16::MAX as i32) as i16),
            Some(y.clamp(i16::MIN as i32, i16::MAX as i32) as i16),
            None,
            None,
        )
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
        self.connection.destroy_window(self.window)
    }

    pub fn focus(&self) -> anyhow::Result<()> {
        self.connection.map_window(self.window)?;
        self.connection.raise_window(self.window)?;
        self.connection.set_input_focus(self.window)
    }

    pub fn maximize(&self) -> anyhow::Result<()> {
        self.connection.change_net_wm_state(
            self.window,
            NET_WM_STATE_ADD,
            self.connection.net_wm_state_maximized_horz_atom()?,
            self.connection.net_wm_state_maximized_vert_atom()?,
        )
    }

    pub fn restore(&self) -> anyhow::Result<()> {
        self.connection.change_net_wm_state(
            self.window,
            NET_WM_STATE_REMOVE,
            self.connection.net_wm_state_maximized_horz_atom()?,
            self.connection.net_wm_state_maximized_vert_atom()?,
        )
    }

    pub fn toggle_fullscreen(&self) -> anyhow::Result<()> {
        self.connection.change_net_wm_state(
            self.window,
            NET_WM_STATE_TOGGLE,
            self.connection.net_wm_state_fullscreen_atom()?,
            xcb::x::ATOM_NONE,
        )
    }

    pub fn destroy(&self) -> anyhow::Result<()> {
        self.connection.destroy_window(self.window)
    }

    pub fn get_geometry(&self) -> anyhow::Result<(i32, i32, u32, u32)> {
        let (x, y, width, height) = self.connection.get_window_geometry(self.window)?;
        Ok((x as i32, y as i32, width as u32, height as u32))
    }

    pub fn xcb_window_handle_data(&self) -> XcbWindowHandleData {
        XcbWindowHandleData {
            window: NonZeroU32::new(self.window.resource_id())
                .expect("xcb window ids are non-zero"),
            visual_id: NonZeroU32::new(self.connection.root_visual),
        }
    }
}
