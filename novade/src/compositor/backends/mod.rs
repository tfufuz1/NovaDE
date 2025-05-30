//! Backends Module
//!
//! This module provides abstractions and implementations for different
//! hardware and display backends that NovaDE can run on. This includes:
//! - Winit backend (for running in a window on existing desktops, for development)
//! - DRM/KMS backend (for direct hardware control on Linux)
//! - Headless backend (for testing)
//! - X11 backend (for running embedded in an X11 window, less common now)

// Allow dead code for now, as backends will be progressively implemented.
#![allow(dead_code)]

pub mod winit;
// TODO: pub mod drm;
// TODO: pub mod headless;

/// A generic trait for compositor backends.
///
/// Each backend will be responsible for:
/// - Setting up the display and input resources.
/// - Providing event sources for the main event loop.
/// - Handling output rendering (e.g., buffer swapping).
pub trait Backend {
    // TODO: Define common methods for backends, e.g.,
    // fn seat_name(&self) -> String;
    // fn GbmDevice(&self) -> Option<GbmDevice<Fd>>; // For DRM
    // fn signaller(&self) -> Signaller<BackendEvent>; // For Calloop
    // ... more methods as needed
}
```
