// This module defines traits (ports) that the domain logic expects
// to be implemented by outer layers (e.g., application or infrastructure).

pub mod config_service;
pub use config_service::ConfigServiceAsync; // Added this line
