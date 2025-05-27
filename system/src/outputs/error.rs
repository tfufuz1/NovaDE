use thiserror::Error;

#[derive(Debug, Error)]
pub enum OutputError {
    #[error("Device access failed for device '{device}': {source}")]
    DeviceAccessFailed {
        device: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Wayland protocol error for '{protocol}': {message}")]
    ProtocolError { protocol: String, message: String },
    #[error("Output configuration conflict: {details}")]
    ConfigurationConflict { details: String },
    #[error("Failed to create resource '{resource}': {reason}")]
    ResourceCreationFailed { resource: String, reason: String },
    #[error("Smithay output error: {source}")]
    SmithayOutputError {
        #[from]
        source: smithay::output::OutputError,
    },
    #[error("Output '{name}' not found")]
    OutputNotFound { name: String },
    #[error("Mode '{mode_details}' not supported for output '{output_name}'")]
    ModeNotSupported {
        output_name: String,
        mode_details: String,
    },
    #[error("Internal output system error: {0}")]
    Internal(String),
}
