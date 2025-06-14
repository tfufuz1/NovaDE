// novade-domain/src/system_health_service/mod.rs

pub mod service;
pub mod error;
// pub mod types; // Potentially for domain-specific types related to system health, if any, beyond core types

pub use service::{SystemHealthService, SystemHealthServiceTrait};
pub use error::SystemHealthError;
