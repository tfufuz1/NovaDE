use async_trait::async_trait;
use std::cell::Cell;

// REVISED Trait and Struct Definition
#[derive(Clone, Debug, Default)] // Added Default for convenience if needed
pub struct FocusedWindowDetails {
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub icon_name: Option<String>,
}

#[async_trait]
pub trait SystemWindowInfoProvider: Send + Sync {
    fn get_focused_window_details(&self) -> FocusedWindowDetails;
}

pub struct StubSystemWindowInfoProvider {
    counter: Cell<usize>,
    details: Vec<FocusedWindowDetails>,
}

impl StubSystemWindowInfoProvider {
    pub fn new() -> Self {
        Self {
            counter: Cell::new(0),
            details: vec![
                FocusedWindowDetails {
                    title: Some("Text Editor - Document1".to_string()),
                    app_id: Some("org.gnome.TextEditor".to_string()),
                    icon_name: Some("accessories-text-editor-symbolic".to_string()),
                },
                FocusedWindowDetails {
                    title: Some("Firefox - NovaDE Docs".to_string()),
                    app_id: Some("org.mozilla.firefox".to_string()),
                    icon_name: Some("web-browser-symbolic".to_string()),
                },
                FocusedWindowDetails { // Simulates desktop focus or no specific app window
                    title: None, 
                    app_id: None,
                    icon_name: Some("user-desktop-symbolic".to_string()), // Could be a generic desktop icon
                },
                FocusedWindowDetails {
                    title: Some("Terminal".to_string()),
                    app_id: Some("org.gnome.Console".to_string()),
                    icon_name: Some("utilities-terminal-symbolic".to_string()),
                },
            ],
        }
    }
}

impl Default for StubSystemWindowInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemWindowInfoProvider for StubSystemWindowInfoProvider {
    fn get_focused_window_details(&self) -> FocusedWindowDetails {
        let index = self.counter.get();
        let details_to_return = self.details[index].clone();
        self.counter.set((index + 1) % self.details.len()); // Advance counter for next call
        details_to_return
    }
}
