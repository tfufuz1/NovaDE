use thiserror::Error;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

#[derive(Debug, Error)]
pub enum XdgDecorationError {
    #[error("Toplevel surface {0:?} already has a decoration object.")]
    AlreadyDecorated(WlSurface),
    #[error("The requested decoration mode is not supported by the compositor.")]
    ModeNotSupported,
    #[error("Internal error in XDG decoration handling: {0}")]
    Internal(String),
}
