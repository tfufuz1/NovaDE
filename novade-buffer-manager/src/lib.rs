//! # Novade Buffer Manager
//!
//! This crate provides core functionalities for managing buffer objects within the
//! Novade compositor. It handles buffer registration, reference counting, and provides
//! structures for describing buffer properties like type, format, and dimensions.
//!
//! It is designed to be used by other components of the compositor, such as the
//! surface management system, to associate buffers with surfaces for rendering.

pub mod buffer;

// Re-export key types for convenience.
pub use buffer::{BufferManager, BufferDetails, BufferId, BufferType, BufferFormat, ClientId};
