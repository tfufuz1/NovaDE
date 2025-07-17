// novade-system/src/compositor/backend/mod.rs

use anyhow::Result;
use calloop::LoopHandle;
use smithay::reexports::wayland_server::DisplayHandle;

use crate::compositor::state::DesktopState; // Assuming DesktopState is here

// Forward declare drm_backend module
pub mod drm_backend;
