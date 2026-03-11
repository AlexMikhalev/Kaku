use super::window::WindowInner;
use crate::connection::ConnectionOps;
use crate::screen::Screens;
use crate::{Dimensions, WindowEvent, WindowState};
use anyhow::Context;
use promise::Future;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use wezterm_input_types::{MouseButtons, MouseEvent, MouseEventKind, MousePress, Point};
use x11::keysym;
use xkbcommon::xkb;

struct XkbKeyboard {
    _context: xkb::Context,
    _keymap: xkb::Keymap,
    state: xkb::State,
    _device_id: i32,
}

pub struct LinuxConnection {
    x11: RefCell<Option<Rc<crate::os::x11::XConnection>>>,
    pub(crate) windows: RefCell<HashMap<usize, Rc<RefCell<WindowInner>>>>,
    xkb: RefCell<Option<XkbKeyboard>>,
    next_window_id: AtomicUsize,
    terminate_requested: AtomicBool,
}

impl LinuxConnection {
    pub fn create_new() -> anyhow::Result<Self> {
        Ok(Self {
            x11: RefCell::new(None),
            windows: RefCell::new(HashMap::new()),
            xkb: RefCell::new(None),
            next_window_id: AtomicUsize::new(1),
            terminate_requested: AtomicBool::new(false),
        })
    }

    pub(crate) fn x11(&self) -> anyhow::Result<Rc<crate::os::x11::XConnection>> {
        if let Some(x11) = self.x11.borrow().as_ref() {
            return Ok(Rc::clone(x11));
        }

        let x11 = crate::os::x11::XConnection::create_new()?;
        self.x11.borrow_mut().replace(Rc::clone(&x11));
        Ok(x11)
    }

    fn current_x11(&self) -> Option<Rc<crate::os::x11::XConnection>> {
        self.x11.borrow().as_ref().map(Rc::clone)
    }

    pub(crate) fn next_window_id(&self) -> usize {
        self.next_window_id.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn window_by_id(&self, window_id: usize) -> Option<Rc<RefCell<WindowInner>>> {
        self.windows.borrow().get(&window_id).map(Rc::clone)
    }

    pub(crate) fn with_window_inner<
        R,
        F: FnOnce(&mut WindowInner) -> anyhow::Result<R> + Send + 'static,
    >(
        window_id: usize,
        f: F,
    ) -> Future<R>
    where
        R: Send + 'static,
    {
        let mut prom = promise::Promise::new();
        let future = prom.get_future().unwrap();
        promise::spawn::spawn_into_main_thread(async move {
            let result = match crate::Connection::get() {
                Some(conn) => match conn.window_by_id(window_id) {
                    Some(handle) => {
                        let mut inner = handle.borrow_mut();
                        f(&mut inner)
                    }
                    None => Err(anyhow::anyhow!("invalid window id {}", window_id)),
                },
                None => Err(anyhow::anyhow!("window connection is not initialized")),
            };
            prom.result(result);
        })
        .detach();

        future
    }

    fn window_id_by_xcb_window(&self, xcb_window: xcb::x::Window) -> Option<usize> {
        self.windows
            .borrow()
            .iter()
            .find_map(|(window_id, handle)| {
                let inner = handle.borrow();
                if inner.xwindow.window == xcb_window {
                    Some(*window_id)
                } else {
                    None
                }
            })
    }

    fn dispatch_event_to_window(
        &self,
        xcb_window: xcb::x::Window,
        event: WindowEvent,
    ) -> anyhow::Result<()> {
        let Some(window_id) = self.window_id_by_xcb_window(xcb_window) else {
            return Ok(());
        };

        let Some(handle) = self.window_by_id(window_id) else {
            return Ok(());
        };

        let mut inner = handle.borrow_mut();
        inner.events.dispatch(event);
        if inner.should_close {
            self.windows.borrow_mut().remove(&window_id);
        }
        Ok(())
    }

    fn modifiers_from_state(&self, state: xcb::x::KeyButMask) -> crate::Modifiers {
        let mut modifiers = crate::Modifiers::NONE;
        if state.contains(xcb::x::KeyButMask::SHIFT) {
            modifiers |= crate::Modifiers::SHIFT;
        }
        if state.contains(xcb::x::KeyButMask::CONTROL) {
            modifiers |= crate::Modifiers::CTRL;
        }
        if state.contains(xcb::x::KeyButMask::MOD1) {
            modifiers |= crate::Modifiers::ALT;
        }
        if state.contains(xcb::x::KeyButMask::MOD4) {
            modifiers |= crate::Modifiers::SUPER;
        }
        modifiers
    }

    fn mouse_buttons_from_state(&self, state: xcb::x::KeyButMask) -> MouseButtons {
        let mut buttons = MouseButtons::NONE;
        if state.contains(xcb::x::KeyButMask::BUTTON1) {
            buttons |= MouseButtons::LEFT;
        }
        if state.contains(xcb::x::KeyButMask::BUTTON2) {
            buttons |= MouseButtons::MIDDLE;
        }
        if state.contains(xcb::x::KeyButMask::BUTTON3) {
            buttons |= MouseButtons::RIGHT;
        }
        buttons
    }

    fn mouse_press_from_detail(&self, detail: u8) -> Option<MousePress> {
        match detail {
            1 => Some(MousePress::Left),
            2 => Some(MousePress::Middle),
            3 => Some(MousePress::Right),
            _ => None,
        }
    }

    fn wheel_kind_from_detail(&self, detail: u8) -> Option<MouseEventKind> {
        match detail {
            4 => Some(MouseEventKind::VertWheel(1)),
            5 => Some(MouseEventKind::VertWheel(-1)),
            6 => Some(MouseEventKind::HorzWheel(1)),
            7 => Some(MouseEventKind::HorzWheel(-1)),
            _ => None,
        }
    }

    fn dispatch_mouse_event(
        &self,
        xcb_window: xcb::x::Window,
        kind: MouseEventKind,
        state: xcb::x::KeyButMask,
        x: i16,
        y: i16,
        root_x: i16,
        root_y: i16,
    ) -> anyhow::Result<()> {
        let mouse_event = MouseEvent {
            kind,
            coords: Point::new(x as isize, y as isize),
            screen_coords: crate::ScreenPoint::new(root_x as isize, root_y as isize),
            mouse_buttons: self.mouse_buttons_from_state(state),
            modifiers: self.modifiers_from_state(state),
        };

        self.dispatch_event_to_window(xcb_window, WindowEvent::MouseEvent(mouse_event))
    }

    fn key_code_from_keysym(&self, keysym: u32, utf32: u32) -> crate::KeyCode {
        match keysym {
            keysym::XK_Return => crate::KeyCode::Char('\r'),
            keysym::XK_Tab => crate::KeyCode::Char('\t'),
            keysym::XK_Escape => crate::KeyCode::Char('\u{1b}'),
            keysym::XK_BackSpace => crate::KeyCode::Char('\u{8}'),
            keysym::XK_Delete => crate::KeyCode::Char('\u{7f}'),
            keysym::XK_Left => crate::KeyCode::LeftArrow,
            keysym::XK_Right => crate::KeyCode::RightArrow,
            keysym::XK_Up => crate::KeyCode::UpArrow,
            keysym::XK_Down => crate::KeyCode::DownArrow,
            keysym::XK_Home => crate::KeyCode::Home,
            keysym::XK_End => crate::KeyCode::End,
            keysym::XK_Page_Up => crate::KeyCode::PageUp,
            keysym::XK_Page_Down => crate::KeyCode::PageDown,
            keysym::XK_Insert => crate::KeyCode::Insert,
            keysym::XK_F1 => crate::KeyCode::Function(1),
            keysym::XK_F2 => crate::KeyCode::Function(2),
            keysym::XK_F3 => crate::KeyCode::Function(3),
            keysym::XK_F4 => crate::KeyCode::Function(4),
            keysym::XK_F5 => crate::KeyCode::Function(5),
            keysym::XK_F6 => crate::KeyCode::Function(6),
            keysym::XK_F7 => crate::KeyCode::Function(7),
            keysym::XK_F8 => crate::KeyCode::Function(8),
            keysym::XK_F9 => crate::KeyCode::Function(9),
            keysym::XK_F10 => crate::KeyCode::Function(10),
            keysym::XK_F11 => crate::KeyCode::Function(11),
            keysym::XK_F12 => crate::KeyCode::Function(12),
            keysym::XK_Shift_L => crate::KeyCode::LeftShift,
            keysym::XK_Shift_R => crate::KeyCode::RightShift,
            keysym::XK_Control_L => crate::KeyCode::LeftControl,
            keysym::XK_Control_R => crate::KeyCode::RightControl,
            keysym::XK_Alt_L | keysym::XK_Meta_L => crate::KeyCode::LeftAlt,
            keysym::XK_Alt_R | keysym::XK_Meta_R => crate::KeyCode::RightAlt,
            keysym::XK_Super_L => crate::KeyCode::LeftWindows,
            keysym::XK_Super_R => crate::KeyCode::RightWindows,
            _ => {
                if let Some(ch) = char::from_u32(utf32).filter(|ch| !ch.is_control()) {
                    crate::KeyCode::Char(ch)
                } else if (0x20..=0x7e).contains(&keysym) || (0xa0..=0xff).contains(&keysym) {
                    crate::KeyCode::Char(
                        char::from_u32(keysym).expect("checked printable keysym range"),
                    )
                } else {
                    crate::KeyCode::RawCode(keysym)
                }
            }
        }
    }

    fn init_xkb_keyboard(&self) -> anyhow::Result<XkbKeyboard> {
        let x11 = self.x11()?;
        let mut major = xkb::x11::MIN_MAJOR_XKB_VERSION;
        let mut minor = xkb::x11::MIN_MINOR_XKB_VERSION;
        let mut base_event = 0;
        let mut base_error = 0;
        anyhow::ensure!(
            xkb::x11::setup_xkb_extension(
                &x11.conn,
                xkb::x11::MIN_MAJOR_XKB_VERSION,
                xkb::x11::MIN_MINOR_XKB_VERSION,
                xkb::x11::SetupXkbExtensionFlags::NoFlags,
                &mut major,
                &mut minor,
                &mut base_event,
                &mut base_error,
            ),
            "XKB extension is unavailable on the X11 connection"
        );
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
        let device_id = match xkb::x11::get_core_keyboard_device_id(&x11.conn) {
            id if id >= 0 => id,
            _ => anyhow::bail!("unable to resolve XKB core keyboard device"),
        };
        let keymap = xkb::x11::keymap_new_from_device(
            &context,
            &x11.conn,
            device_id,
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        );
        anyhow::ensure!(
            !keymap.get_raw_ptr().is_null(),
            "unable to create XKB keymap from X11 device"
        );
        let state = xkb::x11::state_new_from_device(&keymap, &x11.conn, device_id);
        anyhow::ensure!(
            !state.get_raw_ptr().is_null(),
            "unable to create XKB state from X11 device"
        );
        Ok(XkbKeyboard {
            _context: context,
            _keymap: keymap,
            state,
            _device_id: device_id,
        })
    }

    fn key_code_from_keyboard_mapping(
        &self,
        detail: u8,
        state: xcb::x::KeyButMask,
    ) -> crate::KeyCode {
        let Ok(x11) = self.x11() else {
            return crate::KeyCode::RawCode(detail as u32);
        };
        let cookie = x11.conn.send_request(&xcb::x::GetKeyboardMapping {
            first_keycode: detail,
            count: 1,
        });
        let Ok(reply) = x11.conn.wait_for_reply(cookie) else {
            return crate::KeyCode::RawCode(detail as u32);
        };
        let keysyms = reply.keysyms();
        let shift_index = usize::from(state.contains(xcb::x::KeyButMask::SHIFT));
        let group_index = usize::from(state.contains(xcb::x::KeyButMask::MOD5));
        let keysym = keysyms
            .get(group_index * 2 + shift_index)
            .copied()
            .or_else(|| keysyms.get(shift_index).copied())
            .or_else(|| keysyms.first().copied())
            .unwrap_or_default();
        self.key_code_from_keysym(keysym, 0)
    }

    fn key_code_from_xkb_state(&self, detail: u8, key_is_down: bool) -> crate::KeyCode {
        let keycode = xkb::Keycode::new(detail.into());
        let mut xkb = self.xkb.borrow_mut();
        if xkb.is_none() {
            match self.init_xkb_keyboard() {
                Ok(keyboard) => {
                    *xkb = Some(keyboard);
                }
                Err(err) => {
                    log::warn!("failed to initialize XKB keyboard state: {err:#}");
                    return crate::KeyCode::RawCode(detail as u32);
                }
            }
        }
        let state = &mut xkb.as_mut().expect("xkb must be initialized").state;
        let utf32 = state.key_get_utf32(keycode);
        let keysym = state.key_get_one_sym(keycode).raw();
        let key = self.key_code_from_keysym(keysym, utf32);
        let direction = if key_is_down {
            xkb::KeyDirection::Down
        } else {
            xkb::KeyDirection::Up
        };
        state.update_key(keycode, direction);
        key
    }

    fn dispatch_key_event(
        &self,
        xcb_window: xcb::x::Window,
        detail: u8,
        state: xcb::x::KeyButMask,
        key_is_down: bool,
    ) -> anyhow::Result<()> {
        let Some(window_id) = self.window_id_by_xcb_window(xcb_window) else {
            return Ok(());
        };
        let Some(handle) = self.window_by_id(window_id) else {
            return Ok(());
        };

        let modifiers = self.modifiers_from_state(state);
        let raw_key = match self.key_code_from_xkb_state(detail, key_is_down) {
            crate::KeyCode::RawCode(_) => self.key_code_from_keyboard_mapping(detail, state),
            key => key,
        };
        let raw_handled = crate::Handled::new();
        let raw_event = crate::RawKeyEvent {
            key: raw_key.clone(),
            modifiers,
            leds: wezterm_input_types::KeyboardLedStatus::default(),
            phys_code: raw_key.to_phys(),
            raw_code: detail as u32,
            repeat_count: 1,
            key_is_down,
            handled: raw_handled.clone(),
        };

        let mut inner = handle.borrow_mut();
        inner
            .events
            .dispatch(crate::WindowEvent::RawKeyEvent(raw_event.clone()));

        if !raw_handled.is_handled() {
            let (key, modifiers) = raw_key.normalize_shift(modifiers);
            inner
                .events
                .dispatch(crate::WindowEvent::KeyEvent(crate::KeyEvent {
                    key,
                    modifiers,
                    leds: wezterm_input_types::KeyboardLedStatus::default(),
                    repeat_count: 1,
                    key_is_down,
                    raw: Some(raw_event),
                }));
        }

        Ok(())
    }

    fn pump_x11_events(&self) -> anyhow::Result<bool> {
        let Some(x11) = self.current_x11() else {
            return Ok(false);
        };
        let mut handled_any = false;
        let dpi = self.default_dpi() as usize;

        while let Some(event) = x11.poll_for_x11_event() {
            handled_any = true;
            match event {
                crate::os::x11::events::X11Event::Expose { window, .. } => {
                    self.dispatch_event_to_window(window, WindowEvent::NeedRepaint)
                        .context("dispatch Expose")?;
                }
                crate::os::x11::events::X11Event::ConfigureNotify {
                    window,
                    width,
                    height,
                    ..
                } => {
                    self.dispatch_event_to_window(
                        window,
                        WindowEvent::Resized {
                            dimensions: Dimensions {
                                pixel_width: width as usize,
                                pixel_height: height as usize,
                                dpi,
                            },
                            window_state: WindowState::default(),
                            live_resizing: false,
                        },
                    )
                    .context("dispatch ConfigureNotify")?;
                }
                crate::os::x11::events::X11Event::FocusIn { window } => {
                    self.dispatch_event_to_window(window, WindowEvent::FocusChanged(true))
                        .context("dispatch FocusIn")?;
                }
                crate::os::x11::events::X11Event::FocusOut { window } => {
                    self.dispatch_event_to_window(window, WindowEvent::FocusChanged(false))
                        .context("dispatch FocusOut")?;
                }
                crate::os::x11::events::X11Event::Destroy { window } => {
                    self.dispatch_event_to_window(window, WindowEvent::Destroyed)
                        .context("dispatch DestroyNotify")?;
                }
                crate::os::x11::events::X11Event::KeyPress {
                    window,
                    detail,
                    state,
                } => {
                    self.dispatch_key_event(window, detail, state, true)
                        .context("dispatch KeyPress")?;
                }
                crate::os::x11::events::X11Event::KeyRelease {
                    window,
                    detail,
                    state,
                } => {
                    self.dispatch_key_event(window, detail, state, false)
                        .context("dispatch KeyRelease")?;
                }
                crate::os::x11::events::X11Event::ButtonPress {
                    window,
                    detail,
                    state,
                    x,
                    y,
                    root_x,
                    root_y,
                } => {
                    if let Some(kind) = self.wheel_kind_from_detail(detail) {
                        self.dispatch_mouse_event(window, kind, state, x, y, root_x, root_y)
                            .context("dispatch ButtonPress wheel")?;
                    } else if let Some(press) = self.mouse_press_from_detail(detail) {
                        let state = match press {
                            MousePress::Left => state | xcb::x::KeyButMask::BUTTON1,
                            MousePress::Middle => state | xcb::x::KeyButMask::BUTTON2,
                            MousePress::Right => state | xcb::x::KeyButMask::BUTTON3,
                        };
                        self.dispatch_mouse_event(
                            window,
                            MouseEventKind::Press(press),
                            state,
                            x,
                            y,
                            root_x,
                            root_y,
                        )
                        .context("dispatch ButtonPress")?;
                    }
                }
                crate::os::x11::events::X11Event::ButtonRelease {
                    window,
                    detail,
                    state,
                    x,
                    y,
                    root_x,
                    root_y,
                } => {
                    if let Some(press) = self.mouse_press_from_detail(detail) {
                        let state = match press {
                            MousePress::Left => state & !xcb::x::KeyButMask::BUTTON1,
                            MousePress::Middle => state & !xcb::x::KeyButMask::BUTTON2,
                            MousePress::Right => state & !xcb::x::KeyButMask::BUTTON3,
                        };
                        self.dispatch_mouse_event(
                            window,
                            MouseEventKind::Release(press),
                            state,
                            x,
                            y,
                            root_x,
                            root_y,
                        )
                        .context("dispatch ButtonRelease")?;
                    }
                }
                crate::os::x11::events::X11Event::MotionNotify {
                    window,
                    state,
                    x,
                    y,
                    root_x,
                    root_y,
                } => {
                    self.dispatch_mouse_event(
                        window,
                        MouseEventKind::Move,
                        state,
                        x,
                        y,
                        root_x,
                        root_y,
                    )
                    .context("dispatch MotionNotify")?;
                }
                crate::os::x11::events::X11Event::EnterNotify {
                    window,
                    x,
                    y,
                    root_x,
                    root_y,
                } => {
                    self.dispatch_mouse_event(
                        window,
                        MouseEventKind::Move,
                        xcb::x::KeyButMask::empty(),
                        x,
                        y,
                        root_x,
                        root_y,
                    )
                    .context("dispatch EnterNotify")?;
                }
                crate::os::x11::events::X11Event::LeaveNotify { window, .. } => {
                    self.dispatch_event_to_window(window, WindowEvent::MouseLeave)
                        .context("dispatch LeaveNotify")?;
                }
                crate::os::x11::events::X11Event::CloseRequested { window } => {
                    self.dispatch_event_to_window(window, WindowEvent::CloseRequested)
                        .context("dispatch WM_DELETE_WINDOW")?;
                }
                crate::os::x11::events::X11Event::Unknown => {}
            }
        }

        Ok(handled_any)
    }
}

impl crate::connection::ConnectionOps for LinuxConnection {
    fn name(&self) -> String {
        "linux".to_string()
    }

    fn set_event_handler(&self, _func: fn(crate::connection::ApplicationEvent)) {}

    fn dispatch_app_event(&self, _event: crate::connection::ApplicationEvent) {}

    fn default_dpi(&self) -> f64 {
        crate::DEFAULT_DPI
    }

    fn terminate_message_loop(&self) {
        self.terminate_requested.store(true, Ordering::Release);
    }

    fn run_message_loop(&self) -> anyhow::Result<()> {
        while !self.terminate_requested.load(Ordering::Acquire) {
            let had_spawn_work = crate::drain_spawn_queue_burst(64);
            let had_x11_work = self.pump_x11_events().context("pump_x11_events")?;

            if !had_spawn_work && !had_x11_work {
                std::thread::sleep(Duration::from_millis(10));
            }
        }
        Ok(())
    }

    fn get_appearance(&self) -> crate::Appearance {
        crate::Appearance::Dark
    }

    fn screens(&self) -> anyhow::Result<Screens> {
        self.x11()?.screens(self.default_dpi())
    }
}

pub use self::LinuxConnection as Connection;
