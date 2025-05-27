use std::cell::Cell; // For the counter

#[derive(Clone, Debug)]
pub struct ActiveWindowData {
    pub app_id: Option<String>,
    pub window_title: Option<String>,
    pub icon_name: Option<String>,
}

pub struct ActiveWindowService {
    counter: Cell<usize>, // Internal counter to cycle through data
    predefined_windows: Vec<ActiveWindowData>,
}

impl ActiveWindowService {
    pub fn new() -> Self {
        let predefined_windows = vec![
            ActiveWindowData {
                app_id: Some("org.gnome.TextEditor".to_string()),
                window_title: Some("Document 1 - Untitled".to_string()),
                icon_name: Some("accessories-text-editor-symbolic".to_string()),
            },
            ActiveWindowData {
                app_id: Some("org.mozilla.firefox".to_string()),
                window_title: Some("Mozilla Firefox".to_string()),
                icon_name: Some("web-browser-symbolic".to_string()),
            },
            ActiveWindowData {
                app_id: None, // Represents no specific app, maybe desktop focus
                window_title: Some("Desktop".to_string()),
                icon_name: Some("user-desktop-symbolic".to_string()),
            },
            ActiveWindowData { // Add another distinct state
                app_id: Some("org.gnome.Terminal".to_string()),
                window_title: Some("Terminal".to_string()),
                icon_name: Some("utilities-terminal-symbolic".to_string()),
            },
        ];
        Self {
            counter: Cell::new(0),
            predefined_windows,
        }
    }

    pub fn get_current_active_window(&self) -> ActiveWindowData {
        let current_index = self.counter.get();
        let data = self.predefined_windows[current_index].clone();
        self.counter.set((current_index + 1) % self.predefined_windows.len());
        data
    }
}

impl Default for ActiveWindowService {
    fn default() -> Self {
        Self::new()
    }
}
