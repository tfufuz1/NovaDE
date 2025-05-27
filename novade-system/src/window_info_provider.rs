use async_trait::async_trait; // If you anticipate async methods later
use std::cell::Cell; // For simple counter

#[async_trait] // Keep async_trait for potential future async methods
pub trait SystemWindowInfoProvider: Send + Sync {
    fn get_focused_window_title(&self) -> Option<String>;
    // Potentially add methods for app_id, icon_name later
}

pub struct StubSystemWindowInfoProvider {
    counter: Cell<usize>,
    titles: Vec<Option<String>>,
}

impl StubSystemWindowInfoProvider {
    pub fn new() -> Self {
        Self {
            counter: Cell::new(0),
            titles: vec![
                Some("Document1 - Text Editor".to_string()),
                Some("Cool Game - Running".to_string()),
                None, // Simulate no specific window focused / desktop
                Some("Browser - Important Website".to_string()),
            ],
        }
    }
}

impl Default for StubSystemWindowInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait] // Keep async_trait for consistency, even if current methods are sync
impl SystemWindowInfoProvider for StubSystemWindowInfoProvider {
    fn get_focused_window_title(&self) -> Option<String> {
        let index = self.counter.get();
        self.counter.set((index + 1) % self.titles.len());
        self.titles[index].clone()
    }
}
