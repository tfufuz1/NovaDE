use crate::SystemWindowInfoProvider;
use crate::compositor::state::DesktopState;
use crate::FocusedWindowDetails;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

pub struct CompositorWindowInfoProvider {
    desktop_state: Arc<Mutex<DesktopState>>,
}

impl CompositorWindowInfoProvider {
    pub fn new(desktop_state: Arc<Mutex<DesktopState>>) -> Self {
        Self { desktop_state }
    }
}

use novade_domain::window_management::Window;

#[async_trait]
impl SystemWindowInfoProvider for CompositorWindowInfoProvider {
    fn get_focused_window_details(&self) -> FocusedWindowDetails {
        let state = self.desktop_state.lock().unwrap();
        let seat = &state.primary_seat;
        let focused_window = seat.get_keyboard_focus();

        if let Some(focused) = focused_window {
             let space = state.space.lock().unwrap();
             let window = space.elements().find(|w| w.wl_surface().as_ref() == Some(&focused)).cloned();
             if let Some(window) = window {
                 return FocusedWindowDetails {
                     title: window.title.clone(),
                     app_id: window.app_id.clone(),
                     icon_name: None, // We don't have this information yet
                 };
             }
        }

        FocusedWindowDetails::default()
    }

    async fn get_windows(&self) -> Vec<FocusedWindowDetails> {
        let state = self.desktop_state.lock().unwrap();
        let space = state.space.lock().unwrap();
        space
            .elements()
            .map(|w| FocusedWindowDetails {
                title: w.title.clone(),
                app_id: w.app_id.clone(),
                icon_name: None,
            })
            .collect()
    }
}
