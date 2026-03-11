//! X11 Connection management - minimal compile-first implementation.

use crate::screen::{ScreenInfo, Screens};
use anyhow::Context;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt;
use std::ptr::NonNull;
use std::rc::Rc;

use xcb::x::{Atom, Window as XcbWindow};
use xcb::Connection;
use xcb::Xid;

use super::events::X11Event;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct XcbDisplayHandleData {
    pub connection: Option<NonNull<c_void>>,
    pub screen: i32,
}

pub struct XConnection {
    pub conn: Connection,
    pub default_screen: i32,
    pub root_window: XcbWindow,
    pub root_visual: u32,
    pub root_width: u16,
    pub root_height: u16,
    pub white_pixel: u32,
    pub windows: RefCell<HashMap<XcbWindow, XWindowInfo>>,
    pub atom_cache: RefCell<HashMap<String, Atom>>,
}

#[derive(Clone, Debug)]
pub struct XWindowInfo {
    pub window: XcbWindow,
    pub width: u32,
    pub height: u32,
    pub visual_id: u32,
    pub colormap: u32,
}

impl fmt::Debug for XConnection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XConnection")
            .field("default_screen", &self.default_screen)
            .field("root_window", &self.root_window)
            .field("root_visual", &self.root_visual)
            .field("root_width", &self.root_width)
            .field("root_height", &self.root_height)
            .field("white_pixel", &self.white_pixel)
            .finish_non_exhaustive()
    }
}

impl XConnection {
    pub fn create_new() -> anyhow::Result<Rc<Self>> {
        let (conn, screen_num) = Connection::connect(None)?;
        let (root_window, root_visual, root_width, root_height, white_pixel) = {
            let setup = conn.get_setup();
            let screen = setup
                .roots()
                .nth(screen_num as usize)
                .ok_or_else(|| anyhow::anyhow!("No X11 screens available"))?;
            (
                screen.root(),
                screen.root_visual(),
                screen.width_in_pixels(),
                screen.height_in_pixels(),
                screen.white_pixel(),
            )
        };

        Ok(Rc::new(Self {
            conn,
            default_screen: screen_num,
            root_window,
            root_visual,
            root_width,
            root_height,
            white_pixel,
            windows: RefCell::new(HashMap::new()),
            atom_cache: RefCell::new(HashMap::new()),
        }))
    }

    pub fn generate_id(&self) -> XcbWindow {
        self.conn.generate_id()
    }

    pub fn flush(&self) {
        let _ = self.conn.flush();
    }

    pub fn wait_for_event(&self) -> Option<xcb::Event> {
        self.conn.wait_for_event().ok()
    }

    pub fn poll_for_event(&self) -> Option<xcb::Event> {
        self.conn.poll_for_event().ok().flatten()
    }

    pub fn wait_for_x11_event(&self) -> Option<X11Event> {
        self.wait_for_event().map(|event| self.decode_event(&event))
    }

    pub fn poll_for_x11_event(&self) -> Option<X11Event> {
        self.poll_for_event().map(|event| self.decode_event(&event))
    }

    pub fn decode_event(&self, event: &xcb::Event) -> X11Event {
        X11Event::from_xcb_event(
            event,
            self.wm_protocols_atom().ok(),
            self.wm_delete_window_atom().ok(),
        )
    }

    pub fn get_atom(&self, name: &str) -> anyhow::Result<Atom> {
        if let Some(atom) = self.atom_cache.borrow().get(name) {
            return Ok(*atom);
        }

        let cookie = self.conn.send_request(&xcb::x::InternAtom {
            only_if_exists: false,
            name: name.as_bytes(),
        });
        let reply = self
            .conn
            .wait_for_reply(cookie)
            .map_err(anyhow::Error::from)
            .with_context(|| format!("InternAtom({name})"))?;
        let atom = reply.atom();
        self.atom_cache.borrow_mut().insert(name.to_string(), atom);
        Ok(atom)
    }

    pub fn wm_protocols_atom(&self) -> anyhow::Result<Atom> {
        self.get_atom("WM_PROTOCOLS")
    }

    pub fn wm_delete_window_atom(&self) -> anyhow::Result<Atom> {
        self.get_atom("WM_DELETE_WINDOW")
    }

    pub fn net_active_window_atom(&self) -> anyhow::Result<Atom> {
        self.get_atom("_NET_ACTIVE_WINDOW")
    }

    pub fn net_wm_state_atom(&self) -> anyhow::Result<Atom> {
        self.get_atom("_NET_WM_STATE")
    }

    pub fn net_wm_state_maximized_horz_atom(&self) -> anyhow::Result<Atom> {
        self.get_atom("_NET_WM_STATE_MAXIMIZED_HORZ")
    }

    pub fn net_wm_state_maximized_vert_atom(&self) -> anyhow::Result<Atom> {
        self.get_atom("_NET_WM_STATE_MAXIMIZED_VERT")
    }

    pub fn net_wm_state_fullscreen_atom(&self) -> anyhow::Result<Atom> {
        self.get_atom("_NET_WM_STATE_FULLSCREEN")
    }

    pub fn get_atom_name(&self, atom: Atom) -> anyhow::Result<String> {
        let cookie = self.conn.send_request(&xcb::x::GetAtomName { atom });
        let reply = self
            .conn
            .wait_for_reply(cookie)
            .map_err(anyhow::Error::from)
            .with_context(|| format!("GetAtomName({})", atom.resource_id()))?;
        Ok(reply.name().to_utf8().into_owned())
    }

    pub fn create_simple_window(
        &self,
        x: i16,
        y: i16,
        width: u16,
        height: u16,
    ) -> anyhow::Result<XcbWindow> {
        let window_id = self.generate_id();

        let event_mask = xcb::x::EventMask::EXPOSURE
            | xcb::x::EventMask::KEY_PRESS
            | xcb::x::EventMask::KEY_RELEASE
            | xcb::x::EventMask::BUTTON_PRESS
            | xcb::x::EventMask::BUTTON_RELEASE
            | xcb::x::EventMask::POINTER_MOTION
            | xcb::x::EventMask::ENTER_WINDOW
            | xcb::x::EventMask::LEAVE_WINDOW
            | xcb::x::EventMask::FOCUS_CHANGE
            | xcb::x::EventMask::STRUCTURE_NOTIFY;

        let values = [
            xcb::x::Cw::BackPixel(self.white_pixel),
            xcb::x::Cw::EventMask(event_mask),
        ];

        let cookie = self.conn.send_request_checked(&xcb::x::CreateWindow {
            depth: xcb::x::COPY_FROM_PARENT as u8,
            wid: window_id,
            parent: self.root_window,
            x,
            y,
            width,
            height,
            border_width: 0,
            class: xcb::x::WindowClass::InputOutput,
            visual: self.root_visual,
            value_list: &values,
        });
        self.conn
            .check_request(cookie)
            .map_err(anyhow::Error::from)
            .context("CreateWindow")?;

        let wm_protocols = self.wm_protocols_atom()?;
        let wm_delete_window = self.wm_delete_window_atom()?;
        let cookie = self.conn.send_request_checked(&xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window: window_id,
            property: wm_protocols,
            r#type: xcb::x::ATOM_ATOM,
            data: &[wm_delete_window],
        });
        self.conn
            .check_request(cookie)
            .map_err(anyhow::Error::from)
            .context("ChangeProperty(WM_PROTOCOLS)")?;

        let info = XWindowInfo {
            window: window_id,
            width: width as u32,
            height: height as u32,
            visual_id: self.root_visual,
            colormap: 0,
        };
        self.windows.borrow_mut().insert(window_id, info);
        self.flush();
        Ok(window_id)
    }

    pub fn map_window(&self, window: XcbWindow) -> anyhow::Result<()> {
        let cookie = self
            .conn
            .send_request_checked(&xcb::x::MapWindow { window });
        self.conn
            .check_request(cookie)
            .map_err(anyhow::Error::from)
            .context("MapWindow")?;
        self.flush();
        Ok(())
    }

    pub fn unmap_window(&self, window: XcbWindow) -> anyhow::Result<()> {
        let cookie = self
            .conn
            .send_request_checked(&xcb::x::UnmapWindow { window });
        self.conn
            .check_request(cookie)
            .map_err(anyhow::Error::from)
            .context("UnmapWindow")?;
        self.flush();
        Ok(())
    }

    pub fn configure_window(
        &self,
        window: XcbWindow,
        x: Option<i16>,
        y: Option<i16>,
        width: Option<u16>,
        height: Option<u16>,
    ) -> anyhow::Result<()> {
        let mut values = Vec::new();
        if let Some(x) = x {
            values.push(xcb::x::ConfigWindow::X(x as i32));
        }
        if let Some(y) = y {
            values.push(xcb::x::ConfigWindow::Y(y as i32));
        }
        if let Some(width) = width {
            values.push(xcb::x::ConfigWindow::Width(width as u32));
        }
        if let Some(height) = height {
            values.push(xcb::x::ConfigWindow::Height(height as u32));
        }

        if !values.is_empty() {
            let cookie = self.conn.send_request_checked(&xcb::x::ConfigureWindow {
                window,
                value_list: &values,
            });
            self.conn
                .check_request(cookie)
                .map_err(anyhow::Error::from)
                .context("ConfigureWindow")?;
            self.flush();
        }

        Ok(())
    }

    pub fn set_window_title(&self, window: XcbWindow, title: &str) -> anyhow::Result<()> {
        let net_wm_name = self.get_atom("_NET_WM_NAME")?;
        let utf8_string = self.get_atom("UTF8_STRING")?;
        let cookie = self.conn.send_request_checked(&xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window,
            property: net_wm_name,
            r#type: utf8_string,
            data: title.as_bytes(),
        });
        self.conn
            .check_request(cookie)
            .map_err(anyhow::Error::from)
            .context("ChangeProperty(_NET_WM_NAME)")?;

        let cookie = self.conn.send_request_checked(&xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window,
            property: xcb::x::ATOM_WM_NAME,
            r#type: xcb::x::ATOM_STRING,
            data: title.as_bytes(),
        });
        self.conn
            .check_request(cookie)
            .map_err(anyhow::Error::from)
            .context("ChangeProperty(WM_NAME)")?;

        self.flush();
        Ok(())
    }

    pub fn destroy_window(&self, window: XcbWindow) -> anyhow::Result<()> {
        let cookie = self
            .conn
            .send_request_checked(&xcb::x::DestroyWindow { window });
        self.conn
            .check_request(cookie)
            .map_err(anyhow::Error::from)
            .context("DestroyWindow")?;
        self.windows.borrow_mut().remove(&window);
        self.flush();
        Ok(())
    }

    pub fn get_window_geometry(&self, window: XcbWindow) -> anyhow::Result<(i16, i16, u16, u16)> {
        let cookie = self.conn.send_request(&xcb::x::GetGeometry {
            drawable: xcb::x::Drawable::Window(window),
        });
        let reply = self
            .conn
            .wait_for_reply(cookie)
            .map_err(anyhow::Error::from)
            .context("GetGeometry")?;
        Ok((reply.x(), reply.y(), reply.width(), reply.height()))
    }

    pub fn root_rect(&self) -> anyhow::Result<crate::ScreenRect> {
        Ok(euclid::rect(
            0,
            0,
            self.root_width as isize,
            self.root_height as isize,
        ))
    }

    fn single_screen(&self, default_dpi: f64) -> anyhow::Result<Screens> {
        let main = ScreenInfo {
            name: format!("x11-screen-{}", self.default_screen),
            rect: self.root_rect()?,
            scale: 1.0,
            max_fps: None,
            effective_dpi: Some(default_dpi),
        };
        let mut by_name = HashMap::new();
        by_name.insert(main.name.clone(), main.clone());

        Ok(Screens {
            main: main.clone(),
            active: main.clone(),
            by_name,
            virtual_rect: main.rect,
        })
    }

    pub fn screens(&self, default_dpi: f64) -> anyhow::Result<Screens> {
        let cookie = self.conn.send_request(&xcb::randr::GetMonitors {
            window: self.root_window,
            get_active: true,
        });
        let reply = match self.conn.wait_for_reply(cookie) {
            Ok(reply) => reply,
            Err(err) => {
                log::debug!("RANDR GetMonitors unavailable: {err:#}");
                return self.single_screen(default_dpi);
            }
        };

        let mut by_name = HashMap::new();
        let mut virtual_rect: Option<crate::ScreenRect> = None;
        let mut main = None;

        for (idx, monitor) in reply.monitors().enumerate() {
            let name = self
                .get_atom_name(monitor.name())
                .unwrap_or_else(|_| format!("monitor-{idx}"));
            let info = ScreenInfo {
                name: name.clone(),
                rect: euclid::rect(
                    monitor.x() as isize,
                    monitor.y() as isize,
                    monitor.width() as isize,
                    monitor.height() as isize,
                ),
                scale: 1.0,
                max_fps: None,
                effective_dpi: Some(default_dpi),
            };
            virtual_rect = Some(match virtual_rect {
                Some(rect) => rect.union(&info.rect),
                None => info.rect,
            });
            if monitor.primary() {
                main = Some(info.clone());
            }
            by_name.insert(name, info);
        }

        let Some(active) = by_name.values().next().cloned() else {
            return self.single_screen(default_dpi);
        };
        let main = main.unwrap_or_else(|| active.clone());

        Ok(Screens {
            main,
            active: active.clone(),
            by_name,
            virtual_rect: virtual_rect.unwrap_or(active.rect),
        })
    }

    pub fn set_input_focus(&self, window: XcbWindow) -> anyhow::Result<()> {
        let cookie = self.conn.send_request_checked(&xcb::x::SetInputFocus {
            revert_to: xcb::x::InputFocus::PointerRoot,
            focus: window,
            time: xcb::x::CURRENT_TIME,
        });
        self.conn
            .check_request(cookie)
            .map_err(anyhow::Error::from)
            .context("SetInputFocus")?;
        self.flush();
        Ok(())
    }

    pub fn raise_window(&self, window: XcbWindow) -> anyhow::Result<()> {
        let cookie = self.conn.send_request_checked(&xcb::x::ConfigureWindow {
            window,
            value_list: &[xcb::x::ConfigWindow::StackMode(xcb::x::StackMode::Above)],
        });
        self.conn
            .check_request(cookie)
            .map_err(anyhow::Error::from)
            .context("ConfigureWindow(StackMode::Above)")?;
        self.flush();
        Ok(())
    }

    pub fn send_client_message(
        &self,
        destination: XcbWindow,
        window: XcbWindow,
        message_type: Atom,
        data: [u32; 5],
    ) -> anyhow::Result<()> {
        let event = xcb::x::ClientMessageEvent::new(
            window,
            message_type,
            xcb::x::ClientMessageData::Data32(data),
        );
        let cookie = self.conn.send_request_checked(&xcb::x::SendEvent {
            propagate: false,
            destination: xcb::x::SendEventDest::Window(destination),
            event_mask: xcb::x::EventMask::SUBSTRUCTURE_REDIRECT
                | xcb::x::EventMask::SUBSTRUCTURE_NOTIFY,
            event: &event,
        });
        self.conn
            .check_request(cookie)
            .map_err(anyhow::Error::from)
            .context("SendEvent(ClientMessage)")?;
        self.flush();
        Ok(())
    }

    pub fn change_net_wm_state(
        &self,
        window: XcbWindow,
        action: u32,
        first: Atom,
        second: Atom,
    ) -> anyhow::Result<()> {
        self.send_client_message(
            self.root_window,
            window,
            self.net_wm_state_atom()?,
            [action, first.resource_id(), second.resource_id(), 1, 0],
        )
    }

    pub fn request_activate_window(&self, window: XcbWindow) -> anyhow::Result<()> {
        self.send_client_message(
            self.root_window,
            window,
            self.net_active_window_atom()?,
            [1, xcb::x::CURRENT_TIME, window.resource_id(), 0, 0],
        )
    }

    pub fn xcb_display_handle_data(&self) -> XcbDisplayHandleData {
        XcbDisplayHandleData {
            connection: NonNull::new(self.conn.get_raw_conn().cast()),
            screen: self.default_screen,
        }
    }
}
