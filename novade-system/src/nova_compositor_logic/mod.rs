// This file makes the modules accessible as a library.

pub mod error;
pub mod protocols;
pub mod state;
pub mod utils;
pub mod main; // Add main as a module

pub use main::start_nova_compositor_server; // Expose the function
