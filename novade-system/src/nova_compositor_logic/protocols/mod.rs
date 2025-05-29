//! Manages and groups all Wayland protocol implementations for the Nova compositor.
//!
//! This module serves as a central point for organizing the various Wayland
//! interfaces supported by the compositor, such as `wl_compositor`, `wl_shm`,
//! `wl_seat`, `wl_output`, `wl_surface`, and `wl_registry`. Each submodule
//! typically handles the logic for one Wayland global and its associated interfaces.

pub mod wl_registry;
pub mod wl_compositor;
pub mod wl_shm;
pub mod wl_surface;
pub mod wl_seat;
pub mod wl_output;
