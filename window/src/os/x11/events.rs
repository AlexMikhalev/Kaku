//! X11 Event handling

use std::rc::Rc;

use xcb::x::{Atom, ClientMessageData, KeyButMask, Window as XcbWindow};
use xcb::Event;
use xcb::Xid;

use crate::os::x11::connection::XConnection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum X11Event {
    KeyPress {
        window: XcbWindow,
        detail: u8,
        state: KeyButMask,
    },
    KeyRelease {
        window: XcbWindow,
        detail: u8,
        state: KeyButMask,
    },
    ButtonPress {
        window: XcbWindow,
        detail: u8,
        state: KeyButMask,
        x: i16,
        y: i16,
        root_x: i16,
        root_y: i16,
    },
    ButtonRelease {
        window: XcbWindow,
        detail: u8,
        state: KeyButMask,
        x: i16,
        y: i16,
        root_x: i16,
        root_y: i16,
    },
    MotionNotify {
        window: XcbWindow,
        state: KeyButMask,
        x: i16,
        y: i16,
        root_x: i16,
        root_y: i16,
    },
    Expose {
        window: XcbWindow,
        width: u32,
        height: u32,
    },
    FocusIn {
        window: XcbWindow,
    },
    FocusOut {
        window: XcbWindow,
    },
    ConfigureNotify {
        window: XcbWindow,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    EnterNotify {
        window: XcbWindow,
        x: i16,
        y: i16,
        root_x: i16,
        root_y: i16,
    },
    LeaveNotify {
        window: XcbWindow,
        x: i16,
        y: i16,
        root_x: i16,
        root_y: i16,
    },
    CloseRequested {
        window: XcbWindow,
    },
    Destroy {
        window: XcbWindow,
    },
    Unknown,
}

impl X11Event {
    pub fn from_xcb_event(
        event: &Event,
        wm_protocols: Option<Atom>,
        wm_delete_window: Option<Atom>,
    ) -> Self {
        match event {
            Event::X(xevent) => match xevent {
                xcb::x::Event::KeyPress(e) => X11Event::KeyPress {
                    window: e.event(),
                    detail: e.detail(),
                    state: e.state(),
                },
                xcb::x::Event::KeyRelease(e) => X11Event::KeyRelease {
                    window: e.event(),
                    detail: e.detail(),
                    state: e.state(),
                },
                xcb::x::Event::ButtonPress(e) => X11Event::ButtonPress {
                    window: e.event(),
                    detail: e.detail(),
                    state: e.state(),
                    x: e.event_x(),
                    y: e.event_y(),
                    root_x: e.root_x(),
                    root_y: e.root_y(),
                },
                xcb::x::Event::ButtonRelease(e) => X11Event::ButtonRelease {
                    window: e.event(),
                    detail: e.detail(),
                    state: e.state(),
                    x: e.event_x(),
                    y: e.event_y(),
                    root_x: e.root_x(),
                    root_y: e.root_y(),
                },
                xcb::x::Event::MotionNotify(e) => X11Event::MotionNotify {
                    window: e.event(),
                    state: e.state(),
                    x: e.event_x(),
                    y: e.event_y(),
                    root_x: e.root_x(),
                    root_y: e.root_y(),
                },
                xcb::x::Event::Expose(e) => X11Event::Expose {
                    window: e.window(),
                    width: e.width() as u32,
                    height: e.height() as u32,
                },
                xcb::x::Event::FocusIn(e) => X11Event::FocusIn { window: e.event() },
                xcb::x::Event::FocusOut(e) => X11Event::FocusOut { window: e.event() },
                xcb::x::Event::ConfigureNotify(e) => X11Event::ConfigureNotify {
                    window: e.window(),
                    x: e.x() as i32,
                    y: e.y() as i32,
                    width: e.width() as u32,
                    height: e.height() as u32,
                },
                xcb::x::Event::EnterNotify(e) => X11Event::EnterNotify {
                    window: e.event(),
                    x: e.event_x(),
                    y: e.event_y(),
                    root_x: e.root_x(),
                    root_y: e.root_y(),
                },
                xcb::x::Event::LeaveNotify(e) => X11Event::LeaveNotify {
                    window: e.event(),
                    x: e.event_x(),
                    y: e.event_y(),
                    root_x: e.root_x(),
                    root_y: e.root_y(),
                },
                xcb::x::Event::DestroyNotify(e) => X11Event::Destroy { window: e.window() },
                xcb::x::Event::ClientMessage(e)
                    if is_wm_delete_window_message(
                        e.r#type(),
                        e.data(),
                        wm_protocols,
                        wm_delete_window,
                    ) =>
                {
                    X11Event::CloseRequested { window: e.window() }
                }
                _ => X11Event::Unknown,
            },
            _ => X11Event::Unknown,
        }
    }
}

pub fn is_wm_delete_window_message(
    message_type: Atom,
    data: ClientMessageData,
    wm_protocols: Option<Atom>,
    wm_delete_window: Option<Atom>,
) -> bool {
    let (Some(wm_protocols), Some(wm_delete_window)) = (wm_protocols, wm_delete_window) else {
        return false;
    };

    if message_type != wm_protocols {
        return false;
    }

    match data {
        ClientMessageData::Data32(data) => data[0] == wm_delete_window.resource_id(),
        _ => false,
    }
}

/// X11 Event loop runner
pub struct X11EventLoop {
    connection: Rc<XConnection>,
}

impl X11EventLoop {
    pub fn new(connection: Rc<XConnection>) -> Self {
        Self { connection }
    }

    /// Run the event loop, calling callbacks for each event
    pub fn run<F>(self, mut callback: F)
    where
        F: FnMut(X11Event),
    {
        loop {
            if let Some(event) = self.connection.wait_for_event() {
                let x11_event = self.connection.decode_event(&event);

                if matches!(x11_event, X11Event::Destroy { .. }) {
                    break;
                }

                callback(x11_event);
            }
        }
    }

    /// Process pending events (non-blocking)
    pub fn poll<F>(&self, mut callback: F)
    where
        F: FnMut(X11Event),
    {
        while let Some(event) = self.connection.poll_for_event() {
            let x11_event = self.connection.decode_event(&event);

            if matches!(x11_event, X11Event::Destroy { .. }) {
                break;
            }

            callback(x11_event);
        }
    }
}
