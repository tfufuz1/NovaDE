// Declare modules
pub mod types;
pub mod traits;
pub mod default_service;
pub mod mock_service; // Added mock_service module

// Re-export key items
pub use types::StdioProcess;
pub use traits::IMCPClientService;
pub use default_service::DefaultMCPClientService;
pub use mock_service::MockRealProcessMCPClientService; // Export the new mock
