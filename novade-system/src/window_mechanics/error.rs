// novade-system/src/window_mechanics/error.rs

use thiserror::Error;
// Import standard I/O error, as it was mentioned as an example in the spec,
// though it might not be used immediately.
use std::io;

/// Defines specific errors that can occur within the window management system.
///
/// These errors are used to provide context about failures in operations like
/// layout calculations, window identification, or interactions with the
/// underlying compositor state.
#[derive(Error, Debug)]
pub enum WindowManagerError {
    /// Occurs when an operation is attempted with an invalid or unknown window ID.
    ///
    /// Contains the problematic window ID.
    #[error("Invalid window ID: {0}")]
    InvalidWindowId(String),

    /// Occurs when a window layout calculation fails.
    ///
    /// Contains a message describing the reason for the layout failure.
    #[error("Failed to calculate window layout: {0}")]
    LayoutCalculation(String),

    /// Occurs when there's an issue accessing or manipulating the compositor's state.
    ///
    /// This could be due to locking issues, unexpected state, or other
    /// problems related to the underlying `smithay` compositor integration.
    /// Contains a message describing the access error.
    #[error("Failed to access compositor state: {0}")]
    CompositorStateAccess(String),

    /// Occurs when an unsupported or currently unimplemented window operation is requested.
    ///
    /// Contains a message describing the unsupported operation.
    #[error("Unsupported window operation: {0}")]
    UnsupportedOperation(String),

    /// Placeholder for I/O errors, if any operations directly cause them.
    /// For example, if window configurations were read from/written to files.
    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    /// A generic error type for other, less specific window management issues.
    ///
    /// Contains a general message describing the error.
    #[error("Generic window manager error: {0}")]
    Other(String),
}

// It can be useful to implement `From` for common error types that might
// be encountered and need to be converted into `WindowManagerError`.
// For instance, if a function returns `Result<_, SomeOtherError>`, you might
// want to convert `SomeOtherError` into `WindowManagerError::Other` or a more
// specific variant.

// Example: Implementing From for a hypothetical external error type
// (This is commented out as no specific external errors are defined yet beyond std::io::Error)
/*
#[derive(Error, Debug)]
pub enum ExternalError {
    #[error("An external problem occurred: {0}")]
    Problem(String),
}

impl From<ExternalError> for WindowManagerError {
    fn from(err: ExternalError) -> Self {
        WindowManagerError::Other(format!("External error: {}", err))
    }
}
*/

// Unit tests for the error enum
#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error; // Required for source() and Display assertions

    #[test]
    fn invalid_window_id_formats_correctly() {
        let error = WindowManagerError::InvalidWindowId("test-id-123".to_string());
        assert_eq!(format!("{}", error), "Invalid window ID: test-id-123");
    }

    #[test]
    fn layout_calculation_formats_correctly() {
        let error = WindowManagerError::LayoutCalculation("Algorithm failure".to_string());
        assert_eq!(format!("{}", error), "Failed to calculate window layout: Algorithm failure");
    }

    #[test]
    fn compositor_state_access_formats_correctly() {
        let error = WindowManagerError::CompositorStateAccess("Failed to acquire lock".to_string());
        assert_eq!(format!("{}", error), "Failed to access compositor state: Failed to acquire lock");
    }

    #[test]
    fn unsupported_operation_formats_correctly() {
        let error = WindowManagerError::UnsupportedOperation("Minimize not implemented".to_string());
        assert_eq!(format!("{}", error), "Unsupported window operation: Minimize not implemented");
    }

    #[test]
    fn io_error_formats_correctly() {
        let original_io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        // Store the Display output of original_io_error BEFORE it's moved into WindowManagerError
        let original_io_error_str = format!("{}", original_io_error);

        let error = WindowManagerError::IoError(original_io_error);
        // The format of an io::Error is platform-dependent and may vary.
        // We check if the Display output of WindowManagerError::IoError contains the original error's string.
        // This is a common way to test this kind of wrapping.
        assert!(format!("{}", error).contains(&original_io_error_str));
        assert!(format!("{}", error).starts_with("I/O error:"));

        // Additionally, check the source
        match error.source() {
            Some(source_err) => {
                // Check if the source error is indeed an io::Error
                assert!(source_err.is::<io::Error>());
                // And that its description matches (or is contained within) the original
                 assert!(source_err.to_string().contains("File not found"));
            }
            None => panic!("Source error not found for IoError variant"),
        }
    }

    #[test]
    fn other_error_formats_correctly() {
        let error = WindowManagerError::Other("A miscellaneous error occurred".to_string());
        assert_eq!(format!("{}", error), "Generic window manager error: A miscellaneous error occurred");
    }

    // Example of testing `From` implementation if one were active
    /*
    #[test]
    fn from_external_error() {
        let external_err = ExternalError::Problem("dependency failed".to_string());
        let wm_error = WindowManagerError::from(external_err);
        assert_eq!(format!("{}", wm_error), "Generic window manager error: External error: An external problem occurred: dependency failed");
    }
    */
}
