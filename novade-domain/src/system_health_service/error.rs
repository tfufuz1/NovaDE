// novade-domain/src/system_health_service/error.rs

use thiserror::Error;
use novade_core::types::system_health::{DiagnosticTestId, AlertId, LogSourceIdentifier};

#[derive(Error, Debug, Clone)]
pub enum SystemHealthError {
    #[error("Configuration error for System Health Service: {0}")]
    ConfigurationError(String),

    #[error("Failed to initialize metric collector: {source_description}")]
    MetricCollectorInitializationError { source_description: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Failed to collect metric: {metric_name}")]
    MetricCollectionError { metric_name: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Failed to initialize log harvester for source '{source_id:?}': {source_description}")]
    LogHarvesterInitializationError { source_id: LogSourceIdentifier, source_description: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Failed to harvest logs from source '{source_id:?}': {source_description}")]
    LogHarvestingError { source_id: LogSourceIdentifier, source_description: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Diagnostic test '{test_id:?}' not found")]
    DiagnosticTestNotFound(DiagnosticTestId),

    #[error("Failed to execute diagnostic test '{test_id:?}': {source_description}")]
    DiagnosticTestExecutionError { test_id: DiagnosticTestId, source_description: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Alert rule engine error: {0}")]
    AlertRuleEngineError(String),

    #[error("Failed to send alert '{alert_id:?}': {source_description}")]
    AlertDispatchError{ alert_id: AlertId, source_description: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>> },

    #[error("Underlying system interface error: {0}")]
    SystemInterfaceError(String),

    #[error("An unexpected error occurred: {0}")]
    Unexpected(String),
}
