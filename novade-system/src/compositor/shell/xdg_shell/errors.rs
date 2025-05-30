use thiserror::Error;
use smithay::{
    reexports::wayland_server::{protocol::wl_surface::WlSurface, Serial},
    wayland::shell::xdg::ToplevelConfigureError,
};
use uuid::Uuid;

// Assuming this path is correct based on project structure
use crate::compositor::core::errors::CompositorCoreError;

#[derive(Error, Debug)]
pub enum XdgShellError {
    #[error("Invalid surface role for XDG shell operation on WlSurface: {0:?}")]
    InvalidSurfaceRole(WlSurface),

    #[error("Window handling error for window ID {0}: {1}")]
    WindowHandlingError(Uuid, String),

    #[error("Popup positioning error: {0}")]
    PopupPositioningError(String),

    #[error("Client acknowledged configure with invalid serial. Client: {client_serial:?}, Expected: {expected_serial:?}")]
    InvalidAckConfigureSerial {
        client_serial: Serial,
        expected_serial: Serial,
    },

    #[error("Toplevel with ID {0} not found.")]
    ToplevelNotFound(Uuid),

    #[error("Failed to configure XDG Toplevel: {0}")]
    ToplevelConfigureFailed(#[from] ToplevelConfigureError),

    #[error("Core compositor error: {0}")]
    CoreError(#[from] CompositorCoreError),

    #[error("XDG WM Base client data missing. This typically means the client is not bound to XdgWmBase or data was improperly initialized.")]
    XdgWmBaseClientDataMissing,
}
