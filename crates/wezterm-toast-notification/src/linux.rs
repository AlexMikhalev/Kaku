use crate::ToastNotification;

pub fn show_notif(_notif: ToastNotification) -> Result<(), String> {
    // On Linux, we could use libnotify or dbus
    // For now, just log and do nothing (graceful fallback)
    log::info!(
        "Toast notification (Linux stub): {} - {}",
        _notif.title,
        _notif.message
    );
    Ok(())
}
