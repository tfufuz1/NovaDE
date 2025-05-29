use smithay::reexports::wayland_server::{protocol::wl_surface::WlSurface, Serial};
use smithay::wayland::shell::xdg;
use thiserror::Error;
use uuid::Uuid;

use crate::compositor::core::errors::CompositorCoreError;

#[derive(Debug, Error)]
pub enum XdgShellError {
    #[error("Surface {0:?} already has a different role and cannot be used as an XDG surface.")]
    InvalidSurfaceRole(WlSurface),

    #[error("Window handling error for window ID {0}: {1}")]
    WindowHandlingError(Uuid, String),

    #[error("Popup positioning error: {0}")]
    PopupPositioningError(String),

    #[error("Invalid ACK configure serial from client. Client sent {client_serial:?}, expected {expected_serial:?}.")]
    InvalidAckConfigureSerial { client_serial: Serial, expected_serial: Serial },

    #[error("XDG Toplevel with ID {0} not found.")]
    ToplevelNotFound(Uuid),

    #[error("XDG Popup with ID {0} not found.")]
    PopupNotFound(Uuid),

    #[error("Failed to configure XDG Toplevel: {0}")]
    ToplevelConfigureFailed(#[from] xdg::ToplevelConfigureError),

    #[error("Core compositor error: {0}")]
    CoreError(#[from] CompositorCoreError),

    #[error("XDG WM Base client data missing. This typically means the client did not properly initialize its XDG shell globals.")]
    XdgWmBaseClientDataMissing,
}
