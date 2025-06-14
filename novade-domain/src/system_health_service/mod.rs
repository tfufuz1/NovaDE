// novade-domain/src/system_health_service/mod.rs

pub mod service;
// pub mod types; // Potentially for domain-specific types related to system health

pub use service::{DefaultSystemHealthService, SystemHealthService};
// SystemHealthError is now in crate::error, so no local error module to re-export here.
// If SystemHealthError was intended to be crate::error::SystemHealthError, no re-export is needed here,
// consumers will use crate::error::SystemHealthError or crate::DomainError::SystemHealth.
// If a module alias is desired, it could be:
// pub use crate::error::SystemHealthError; // This would make it accessible as system_health_service::SystemHealthError
// However, it's generally better to use the canonical path from crate::error.
