// Copyright 2024 NovaDE Compsositor contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! XDG Shell Interface
//!
//! This module provides the XDG shell interface, which is used by clients
//! to create and manage surfaces.

use smithay::desktop::{Window, WindowSurfaceType};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::utils::Serial;

/// State of the XDG shell.
pub struct XdgShellState {}

/// Role of an XDG surface.
///
/// An XDG surface can either be a toplevel window or a popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XdgSurfaceRole {
    /// Toplevel window.
    Toplevel,
    /// Popup window.
    Popup,
}

/// Role-specific data for an XDG surface.
#[derive(Debug)]
pub enum XdgRoleSpecificData {
    /// Data for a toplevel surface.
    Toplevel(XdgToplevelData),
    /// Data for a popup surface.
    Popup(XdgPopupData),
    /// Surface has not yet been assigned a role.
    None,
}

/// Data associated with an XDG surface.
#[derive(Debug)]
pub struct XdgSurfaceData {
    /// Role of the surface.
    pub role: XdgSurfaceRole, // Keep this for quick role checks, though XdgRoleSpecificData also implies it
    /// Role-specific data.
    pub role_data: XdgRoleSpecificData,
    /// Parent surface, if any (especially for popups).
    pub parent: Option<WlSurface>,
    /// The underlying Smithay window object.
    pub window: Window,
    /// The domain-specific WindowId for NovaDE workspace management.
    pub domain_id: Option<novade_domain::workspaces::core::WindowId>,
    // TODO: Add more fields as needed, e.g., cached geometry, configure serials.
}

/// Data associated with an XDG toplevel surface.
#[derive(Debug, Default)]
pub struct XdgToplevelData {
    /// Title of the toplevel.
    pub title: Option<String>,
    /// Application ID of the toplevel.
    pub app_id: Option<String>,
    /// Parent toplevel, if any.
    pub parent: Option<WlSurface>, // This is the xdg_toplevel parent, distinct from popup parent.
    /// Current state of the toplevel (e.g., maximized, fullscreen).
    pub current_state: ToplevelState,
    /// Minimum size requested by the client.
    pub min_size: Option<(i32, i32)>,
    /// Maximum size requested by the client.
    pub max_size: Option<(i32, i32)>,
    /// Preferred decoration mode.
    pub decoration_mode: Option<smithay::reexports::wayland_protocols::unstable::xdg_decoration::v1::server::zxdg_toplevel_decoration_v1::Mode>,
    // TODO: Add more fields like pending_state.
}

/// State of an XDG toplevel surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToplevelState {
    /// Whether the toplevel is maximized.
    pub maximized: bool,
    /// Whether the toplevel is minimized.
    pub minimized: bool,
    /// Whether the toplevel is fullscreen.
    pub fullscreen: bool,
    /// Whether the toplevel is active (has focus).
    pub activated: bool,
    /// Whether the toplevel is resizing.
    pub resizing: bool,
    // TODO: Add more states like suspended.
}

/// Data associated with an XDG popup surface.
#[derive(Debug)]
pub struct XdgPopupData {
    /// Parent surface of the popup.
    pub parent: WlSurface,
    // TODO: Add positioner data (e.g., from xdg_positioner).
    // TODO: Add grab state.
    pub committed: bool, // To track if initial configure has been acked
}

// TODO: Implement XDG shell handlers in `handlers.rs`.

pub mod handlers;
