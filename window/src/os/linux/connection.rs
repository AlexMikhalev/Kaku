use crate::{Appearance, Connection, GeometryOrigin, ResolvedGeometry};
use anyhow::Result as Fallible;
use std::rc::Rc;

pub struct LinuxConnection;

impl LinuxConnection {
    pub fn create_new() -> Fallible<Rc<LinuxConnection>> {
        Ok(Rc::new(Self))
    }
}

impl crate::connection::ConnectionOps for LinuxConnection {
    fn name(&self) -> String {
        "linux".to_string()
    }

    fn set_event_handler(&self, _func: fn(crate::connection::ApplicationEvent)) {
        // Linux stub - events not implemented yet
    }

    fn dispatch_app_event(&self, _event: crate::connection::ApplicationEvent) {
        // Linux stub
    }

    fn default_dpi(&self) -> f64 {
        crate::DEFAULT_DPI
    }

    fn terminate_message_loop(&self) {
        // Linux stub
    }

    fn run_message_loop(&self) -> Fallible<()> {
        // Linux stub - just block forever
        // In a real implementation, this would run the event loop
        std::thread::park();
        Ok(())
    }

    fn get_appearance(&self) -> Appearance {
        Appearance::Dark
    }
}
