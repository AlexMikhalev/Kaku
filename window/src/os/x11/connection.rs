//! X11 Connection management
//! Based on WezTerm's implementation patterns

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::unix::io::AsRawFd;
use std::rc::Rc;

use xcb::x::{Atom, Screen, Window as XWindow};
use xcb::{Connection, Event};

pub struct XConnection {
    pub conn: Connection,
    pub setup: xcb::x::Setup,
    pub screen: Screen,
    pub default_screen: i32,
    pub windows: RefCell<HashMap<XWindow, XWindowInfo>>,
    pub atom_cache: RefCell<HashMap<String, Atom>>,
}

#[derive(Clone)]
pub struct XWindowInfo {
    pub window: XWindow,
    pub width: u32,
    pub height: u32,
    pub visual_id: u32,
    pub colormap: u32,
}

impl XConnection {
    pub fn create_new() -> anyhow::Result<Rc<Self>> {
        // Connect to X11
        let conn = Connection::connect(None)?;

        // Get setup information
        let setup = conn.get_setup();

        // Get the default screen
        let screen = setup
            .roots()
            .nth(0)
            .ok_or_else(|| anyhow::anyhow!("No screens available"))?
            .clone();

        let xconn = Rc::new(Self {
            conn: conn.clone(),
            setup,
            screen: screen.clone(),
            default_screen: 0,
            windows: RefCell::new(HashMap::new()),
            atom_cache: RefCell::new(HashMap::new()),
        });

        // Setup atoms
        xconn.init_atoms()?;

        // Setup screen info
        xconn.query_screen_sizes()?;

        Ok(xconn)
    }

    fn init_atoms(&self) -> anyhow::Result<()> {
        // Pre-cache common atoms
        let atoms = [
            "WM_PROTOCOLS",
            "WM_DELETE_WINDOW",
            "WM_TAKE_FOCUS",
            "_NET_WM_NAME",
            "UTF8_STRING",
            "CLIPBOARD",
            "PRIMARY",
            "STRING",
        ];

        for name in atoms {
            let atom = self.get_atom(name)?;
            self.atom_cache.borrow_mut().insert(name.to_string(), atom);
        }

        Ok(())
    }

    pub fn get_atom(&self, name: &str) -> anyhow::Result<Atom> {
        // Check cache first
        if let Some(atom) = self.atom_cache.borrow().get(name) {
            return Ok(*atom);
        }

        // Request the atom
        let cookie = self.conn.send_request_checked(&xcb::x::InternAtom {
            name: name.as_bytes(),
            only_if_exists: false,
        });

        let reply = self.conn.wait_for_reply(cookie)?;
        let atom = reply.atom();

        self.atom_cache.borrow_mut().insert(name.to_string(), atom);

        Ok(atom)
    }

    fn query_screen_sizes(&self) -> anyhow::Result<()> {
        // Get current screen size
        let screen = &self.screen;
        let width = screen.width_in_pixels() as u32;
        let height = screen.height_in_pixels() as u32;

        log::info!("X11 screen size: {}x{}", width, height);
        Ok(())
    }

    pub fn generate_id(&self) -> XWindow {
        self.conn.generate_id()
    }

    pub fn flush(&self) {
        self.conn.flush();
    }

    pub fn wait_for_event(&self) -> Option<Event> {
        self.conn.wait_for_event().ok()
    }

    pub fn poll_for_event(&self) -> Option<Event> {
        self.conn.poll_for_event().ok().flatten()
    }

    pub fn create_window(
        &self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> anyhow::Result<XWindow> {
        let window_id = self.generate_id();

        let screen = &self.screen;
        let root = screen.root();
        let visual_id = screen.root_visual();

        // Create window
        let cw_values = [
            (&xcb::x::Cw::BackPixel, &(screen.white_pixel() as u64)),
            (
                &xcb::x::Cw::EventMask,
                &[
                    xcb::x::EventMask::EXPOSURE,
                    xcb::x::EventMask::KEY_PRESS,
                    xcb::x::EventMask::KEY_RELEASE,
                    xcb::x::EventMask::BUTTON_PRESS,
                    xcb::x::EventMask::BUTTON_RELEASE,
                    xcb::x::EventMask::POINTER_MOTION,
                    xcb::x::EventMask::ENTER | xcb::x::EventMask::LEAVE,
                    xcb::x::EventMask::FOCUS_CHANGE,
                    xcb::x::EventMask::STRUCTURE_NOTIFY,
                ] as &[u32],
            ),
        ];

        let create_req = xcb::x::CreateWindow {
            depth: xcb::x::COPY_FROM_PARENT as u8,
            wid: window_id,
            parent: *root,
            x: x as i16,
            y: y as i16,
            width: width as u16,
            height: height as u16,
            border_width: 0,
            class: xcb::x::WindowClass::InputOutput as u16,
            visual: *visual_id,
            value_list: &cw_values,
        };

        self.conn.send_request(&create_req);
        self.conn.flush();

        // Store window info
        let info = XWindowInfo {
            window: window_id,
            width,
            height,
            visual_id: *visual_id,
            colormap: 0,
        };
        self.windows.borrow_mut().insert(window_id, info);

        log::info!("Created X11 window: {:?}", window_id);

        Ok(window_id)
    }

    pub fn destroy_window(&self, window: XWindow) -> anyhow::Result<()> {
        let destroy_req = xcb::x::DestroyWindow { window };
        self.conn.send_request(&destroy_req);
        self.windows.borrow_mut().remove(&window);
        Ok(())
    }

    pub fn map_window(&self, window: XWindow) -> anyhow::Result<()> {
        let map_req = xcb::x::MapWindow { window: window };
        self.conn.send_request(&map_req);
        self.conn.flush();
        Ok(())
    }

    pub fn unmap_window(&self, window: XWindow) -> anyhow::Result<()> {
        let unmap_req = xcb::x::UnmapWindow { window: window };
        self.conn.send_request(&unmap_req);
        self.conn.flush();
        Ok(())
    }

    pub fn configure_window(
        &self,
        window: XWindow,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<u32>,
        height: Option<u32>,
    ) -> anyhow::Result<()> {
        let mut values = Vec::new();

        if let Some(x) = x {
            values.push(xcb::x::ConfigWindow::X(x as i32));
        }
        if let Some(y) = y {
            values.push(xcb::x::ConfigWindow::Y(y as i32));
        }
        if let Some(w) = width {
            values.push(xcb::x::ConfigWindow::Width(w as u32));
        }
        if let Some(h) = height {
            values.push(xcb::x::ConfigWindow::Height(h as u32));
        }

        let configure_req = xcb::x::ConfigureWindow {
            window: window,
            value_list: &values,
        };

        self.conn.send_request(&configure_req);
        self.conn.flush();

        Ok(())
    }

    pub fn set_window_title(&self, window: XWindow, title: &str) -> anyhow::Result<()> {
        let title_atom = self.get_atom("_NET_WM_NAME")?;
        let utf8_atom = self.get_atom("UTF8_STRING")?;

        // Set _NET_WM_NAME (UTF-8)
        let wm_name = xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window: window,
            property: title_atom,
            r#type: utf8_atom,
            data: title.as_bytes(),
        };
        self.conn.send_request(&wm_name);

        // Also set legacy WM_NAME
        let wm_name_legacy = xcb::x::ChangeProperty {
            mode: xcb::x::PropMode::Replace,
            window: window,
            property: xcb::x::Atom::from_reply(&self.conn.send_request(&xcb::x::InternAtom {
                name: b"WM_NAME",
                only_if_exists: false,
            }))
            .atom(),
            r#type: utf8_atom,
            data: title.as_bytes(),
        };
        self.conn.send_request(&wm_name_legacy);

        self.conn.flush();
        Ok(())
    }

    pub fn get_window_geometry(&self, window: XWindow) -> anyhow::Result<(i32, i32, u32, u32)> {
        let cookie = self.conn.send_request(&xcb::x::GetGeometry {
            drawable: xcb::x::Drawable::Window(window),
        });

        let reply = self.conn.wait_for_reply(cookie)?;

        Ok((reply.x(), reply.y(), reply.width(), reply.height()))
    }

    pub fn intern_atom(&self, name: &str) -> anyhow::Result<Atom> {
        let cookie = self.conn.send_request_checked(&xcb::x::InternAtom {
            name: name.as_bytes(),
            only_if_exists: false,
        });

        let reply = self.conn.wait_for_reply(cookie)?;
        Ok(reply.atom())
    }
}

impl AsRawFd for XConnection {
    fn as_raw_fd(&self) -> std::os::unix::io::RawFd {
        self.conn.as_raw_fd()
    }
}
