// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # XDG Shell Implementation
//!
//! This module implements the XDG shell protocol for the compositor.
//! It defines errors, handlers, and manages the XDG shell global.
//! Types are re-exported from the `types` submodule.

pub mod errors;
pub mod types;
pub mod handlers;

pub use errors::XdgShellError;
pub use types::*; // Re-export types like ManagedWindow

use crate::compositor::core::state::DesktopState;
use smithay::reexports::wayland_server::{
    Client, DisplayHandle, GlobalDispatch, DataInit, New, Resource,
};
use smithay::reexports::wayland_protocols::xdg::shell::server::xdg_wm_base::XdgWmBase;
use smithay::wayland::shell::xdg::{XdgShellState, XdgWmBaseClientData, XdgActivationState};
use std::sync::Arc;

// GlobalDispatch for XdgWmBase
// The GlobalData for xdg_wm_base is Arc<XdgWmBaseClientData> as per Smithay's XdgShellState::new()
impl GlobalDispatch<XdgWmBase, Arc<XdgWmBaseClientData>> for DesktopState {
    fn bind(
        _state: &mut Self,
        _handle: &DisplayHandle,
        _client: &Client,
        resource: New<XdgWmBase>,
        client_data: &Arc<XdgWmBaseClientData>,
        data_init: &mut DataInit<'_, Self>,
    ) {
        tracing::info!(client_id = ?_client.id(), "Client bound to XdgWmBase global");
        data_init.init(resource, client_data.clone());
    }

    fn can_view(_client: Client, _global_data: &Arc<XdgWmBaseClientData>) -> bool {
        true
    }
}

// Function to ensure XDG Shell global is active
pub fn create_xdg_shell_global(
    desktop_state: &DesktopState,
    _display_handle: &DisplayHandle,
) -> Result<(), String> {
    // XdgShellState::new() (called during DesktopState::new) already registers the xdg_wm_base global.
    // Smithay 0.10+ XdgShellState::new() returns (XdgShellState, Global<XdgWmBase>).
    // The Global<XdgWmBase> object would typically be stored by the caller (e.g., in DesktopState or a globals tracking struct)
    // and then passed to display_handle.create_global(...).
    // However, the current DesktopState directly holds XdgShellState.
    // We rely on XdgShellState having been initialized with an XdgActivationState,
    // and that its internal global was registered.
    // A call to desktop_state.xdg_shell_state.global() (if it existed) would confirm.
    // For now, just log that it's expected to be active.
    tracing::info!("XDG WM Base global (managed by XdgShellState within DesktopState) is assumed to be active.");
    Ok(())
}
