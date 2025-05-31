// nova_shell/src/launcher.rs
use tracing::info;

pub struct Launcher {}

impl Launcher {
    pub fn new() -> Self {
        info!("Initializing Launcher component");
        Self {}
    }

    pub fn show(&self) {
        info!("Launcher showing (placeholder)");
    }

    pub fn launch_app(&self, app_id: &str) {
        info!("Attempting to launch app (placeholder): {}", app_id);
        // This will eventually need to communicate with the compositor
    }
}
