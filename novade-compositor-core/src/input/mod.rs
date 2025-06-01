//! Handles Wayland input protocols like `wl_seat`, `wl_pointer`, `wl_keyboard`,
//! `wl_touch`, and manages input focus logic for the Novade compositor.
//!
//! This module is responsible for:
//! - Defining and managing seat capabilities.
//! - Handling client requests for input device objects (`wl_pointer`, `wl_keyboard`, `wl_touch`).
//! - Tracking input focus for pointer and keyboard events.
//! - Defining structures for input events (though actual event dispatch is typically
//!   handled by a higher-level compositor loop interacting with a Wayland backend).

pub mod seat;
pub mod pointer;
pub mod keyboard;
pub mod touch;
pub mod focus;
