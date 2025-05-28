// Re-export key components for easier access from parent modules if needed
pub mod errors;
pub mod state;
pub mod globals; 
pub mod handlers; // Ensure handlers module is declared

// Potentially re-export specific structs or traits if they form the public API of the core module
pub use self::state::{DesktopState, ClientCompositorData};
pub use self::errors::CompositorCoreError;
// Globals and Handlers are mostly for internal wiring, so direct re-export might not be needed.
