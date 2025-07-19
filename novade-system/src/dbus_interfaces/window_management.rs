use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.novade.WindowManager",
    default_service = "org.novade.WindowManager",
    default_path = "/org/novade/WindowManager"
)]
pub trait WindowManager {
    #[dbus_proxy(signal)]
    fn decoration_mode_changed(&self, window_id: &str, mode: &str) -> zbus::Result<()>;
}
