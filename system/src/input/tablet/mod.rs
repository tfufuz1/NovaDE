// system/src/input/tablet/mod.rs
pub mod types;
pub mod event_translator;
// pub mod handlers; // If specific handlers for Smithay traits are needed beyond DesktopState

pub use types::*; // Re-export types
// pub use event_translator::*; // Re-export handlers later
