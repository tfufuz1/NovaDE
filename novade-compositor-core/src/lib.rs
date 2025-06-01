//! # Novade Compositor Core
//!
//! This crate forms the core logic for the Novade Wayland compositor.
//! It includes management of:
//! - Surfaces (`wl_surface`): Their state, attributes, pending vs. current state,
//!   damage tracking, and commit lifecycle.
//! - Subsurfaces (`wl_subsurface`): Hierarchy, synchronization, and stacking order.
//! - Buffers: (Via `novade-buffer-manager`) Association with surfaces.
//!
//! The core is responsible for maintaining the scene graph and the state of
//! various Wayland objects that clients interact with. It provides the foundational
//! building blocks upon which protocol handling and rendering are built.

pub mod surface;
pub mod subcompositor;
