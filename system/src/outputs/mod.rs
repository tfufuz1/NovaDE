pub mod error;
pub mod device;
pub mod manager;

// Re-export key types for easier access
pub use self::error::OutputError;
pub use self::device::{OutputDevice, DpmsState};
pub use self::manager::{OutputManager, HotplugEvent};
