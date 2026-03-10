//! X11 Event handling

use xcb::{x::EventMask, Event};

use crate::os::x11::connection::XConnection;

#[derive(Debug, Clone)]
pub enum X11Event {
    KeyPress(u32, u32),
    KeyRelease(u32, u32),
    ButtonPress(u32, i16, i16),
    ButtonRelease(u32, i16, i16),
    MotionNotify(i16, i16),
    Expose(u32, u32),
    FocusIn,
    FocusOut,
    ConfigureNotify(i32, i32, u32, u32),
    EnterNotify,
    LeaveNotify,
    Destroy,
    Unknown,
}

impl X11Event {
    pub fn from_xcb_event(event: &Event, window: xcb::x::Window) -> Self {
        match event {
            Event::X(xevent) => match xevent {
                xcb::x::Event::KeyPress(e) => X11Event::KeyPress(e.detail(), e.sequence() as u32),
                xcb::x::Event::KeyRelease(e) => {
                    X11Event::KeyRelease(e.detail(), e.sequence() as u32)
                }
                xcb::x::Event::ButtonPress(e) => {
                    X11Event::ButtonPress(e.detail(), e.event_x(), e.event_y())
                }
                xcb::x::Event::ButtonRelease(e) => {
                    X11Event::ButtonRelease(e.detail(), e.event_x(), e.event_y())
                }
                xcb::x::Event::MotionNotify(e) => X11Event::MotionNotify(e.event_x(), e.event_y()),
                xcb::x::Event::Expose(e) => X11Event::Expose(e.width(), e.height()),
                xcb::x::Event::FocusIn(_) => X11Event::FocusIn,
                xcb::x::Event::FocusOut(_) => X11Event::FocusOut,
                xcb::x::Event::ConfigureNotify(e) => {
                    X11Event::ConfigureNotify(e.x(), e.y(), e.width(), e.height())
                }
                xcb::x::Event::EnterNotify(_) => X11Event::EnterNotify,
                xcb::x::Event::LeaveNotify(_) => X11Event::LeaveNotify,
                xcb::x::Event::DestroyNotify(_) => X11Event::Destroy,
                _ => X11Event::Unknown,
            },
            _ => X11Event::Unknown,
        }
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
    pub fn run<F>(self, window: xcb::x::Window, mut callback: F)
    where
        F: FnMut(X11Event),
    {
        loop {
            // Wait for event
            if let Some(event) = self.connection.wait_for_event() {
                let x11_event = X11Event::from_xcb_event(&event, window);

                // Handle destroy specially - exit loop
                if matches!(x11_event, X11Event::Destroy) {
                    break;
                }

                callback(x11_event);
            }
        }
    }

    /// Process pending events (non-blocking)
    pub fn poll<F>(window: xcb::x::Window, mut callback: F)
    where
        F: FnMut(X11Event),
    {
        while let Some(event) = self.connection.poll_for_event() {
            let x11_event = X11Event::from_xcb_event(&event, window);

            if matches!(x11_event, X11Event::Destroy) {
                break;
            }

            callback(x11_event);
        }
    }
}
