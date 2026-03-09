use std::rc::Rc;

pub struct LinuxConnection {
    // Empty for now - would need X11/Wayland connection
}

impl LinuxConnection {
    pub fn create_new() -> anyhow::Result<Rc<Self>> {
        Ok(Rc::new(Self))
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

    fn terminate_message_loop(&self) {}

    fn run_message_loop(&self) -> anyhow::Result<()> {
        // Just block - real implementation would run event loop
        std::thread::park();
        Ok(())
    }

    fn get_appearance(&self) -> crate::Appearance {
        crate::Appearance::Dark
    }
}

// Re-export Connection for use by other modules
pub use self::LinuxConnection as Connection;
