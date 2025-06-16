use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum InputError {
    #[error("Failed to initialize libinput: {0}")]
    LibinputInitialization(String),
    #[error("Failed to create xkbcommon context: {0}")]
    XkbCommonContext(String),
    #[error("Failed to configure xkbcommon state: {0}")]
    XkbCommonState(String),
    #[error("Input device error: {0}")]
    Device(String),
    #[error("Error processing input event: {0}")]
    EventProcessing(String),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    // Weitere spezifische Fehler hier hinzufügen
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    #[test]
    fn test_libinput_initialization_error_display() {
        let error = InputError::LibinputInitialization("Failed to open device".to_string());
        assert_eq!(format!("{}", error), "Failed to initialize libinput: Failed to open device");
    }

    #[test]
    fn test_xkb_common_context_error_display() {
        let error = InputError::XkbCommonContext("Could not create context".to_string());
        assert_eq!(format!("{}", error), "Failed to create xkbcommon context: Could not create context");
    }

    #[test]
    fn test_xkb_common_state_error_display() {
        let error = InputError::XkbCommonState("Failed to compile keymap".to_string());
        assert_eq!(format!("{}", error), "Failed to configure xkbcommon state: Failed to compile keymap");
    }

    #[test]
    fn test_device_error_display() {
        let error = InputError::Device("Device not found".to_string());
        assert_eq!(format!("{}", error), "Input device error: Device not found");
    }

    #[test]
    fn test_event_processing_error_display() {
        let error = InputError::EventProcessing("Malformed event".to_string());
        assert_eq!(format!("{}", error), "Error processing input event: Malformed event");
    }

    #[test]
    fn test_io_error_from_implementation() {
        let io_err = Error::new(ErrorKind::NotFound, "File not found");
        let input_err = InputError::from(io_err); // Relies on #[from]
        assert_eq!(format!("{}", input_err), "IO error: File not found");
    }

    #[test]
    fn test_io_error_direct_construction_display() {
        // This test case is a bit redundant if the From trait is tested,
        // but it ensures the direct formatting also works as expected.
        let io_err = Error::new(ErrorKind::PermissionDenied, "Access denied");
        let error = InputError::Io(io_err);
        assert_eq!(format!("{}", error), "IO error: Access denied");
    }

    // Test Debug formatting for one variant as an example
    #[test]
    fn test_input_error_debug_format() {
        let error = InputError::LibinputInitialization("Debug test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("LibinputInitialization"));
        assert!(debug_str.contains("Debug test"));
    }
}
