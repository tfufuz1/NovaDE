// novade-ui/src/assistant_ui/mod.rs

//! UI components and logic for the Context-Aware Smart Assistant.

pub mod widgets;
pub mod controller; // Added as per step 2, forward declaring
// pub mod state; // Potentially for managing UI-specific state of the assistant

// Re-export key components for easier access from other UI modules
pub use widgets::{AssistantMainWidget, AssistantDisplayItem}; // Added AssistantDisplayItem
pub use controller::{AssistantUIController, AssistantUIUpdate}; // Uncommented and added AssistantUIUpdate

// TODO: Assistant Integration: This module will house all UI elements related to the assistant.
// It needs to be initialized and integrated into the main application UI flow,
// possibly as an overlay, a dockable widget, or a popup.
// Ensure `novade-ui/src/lib.rs` declares `pub mod assistant_ui;` if this module is to be used externally.
