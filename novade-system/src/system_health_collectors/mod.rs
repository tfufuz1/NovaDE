// novade-system/src/system_health_collectors/mod.rs

pub mod metric_collector;
pub mod log_harvester;
pub mod diagnostic_runner;
pub mod error; // Common errors for collectors

pub use metric_collector::SystemMetricCollector;
pub use log_harvester::SystemLogHarvester;
pub use diagnostic_runner::SystemDiagnosticRunner;
pub use error::CollectionError;
