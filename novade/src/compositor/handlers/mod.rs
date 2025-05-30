//! Provides handlers for various Wayland protocols.
//!
//! Each submodule in `handlers` is dedicated to one or more related Wayland
//! interfaces (e.g., `wl_compositor`, `xdg_shell`). These modules typically
//! implement Smithay's handler traits (such as `CompositorHandler`, `ShmHandler`,
//! `SeatHandler`, `XdgShellHandler`) for the main `NovaCompositorState`.
//!
//! The implementations define how the compositor reacts to client requests for
//! each protocol, managing resources, changing state, and generating events
//! as per the Wayland specifications. Smithay's `delegate_dispatch!` macros
//! (or more specific `delegate_compositor!`, etc.) are used to connect these
//! handler implementations to the Wayland dispatch mechanism.

// Allow dead code for now, as handlers will be progressively implemented
// and some handler methods might be placeholders initially.
#![allow(dead_code)]

pub mod wl_compositor;
pub mod wl_shm;
pub mod wl_seat;
pub mod xdg_shell;

// TODO: Add handler modules for other essential Wayland protocols as they are implemented:
// pub mod wl_output;          // For managing display outputs
// pub mod presentation_time;  // For synchronized presentation of frames
// pub mod xdg_decoration;     // For server-side and client-side window decorations
// pub mod data_device_manager; // For copy-paste and drag-and-drop
// ... and others as NovaDE's feature set grows.
```
