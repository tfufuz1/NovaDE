//! SHM (Shared Memory) buffer handling for the NovaDE compositor.
//!
//! This module provides utilities and handlers for managing SHM buffers,
//! allowing clients to share memory regions with the compositor for rendering.

pub mod buffer_access;
pub mod errors;

// Re-export key types for convenience
pub use buffer_access::with_shm_buffer_contents;
pub use errors::ShmError;
