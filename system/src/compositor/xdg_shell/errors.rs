use thiserror::Error;
use crate::compositor::core::surface_management::SurfaceRoleError;

#[derive(Debug, Error)]
pub enum XdgShellError {
    #[error("XDG surface role error: {0}")]
    RoleError(#[from] SurfaceRoleError),
    #[error("Window not found: {0:?}")]
    WindowNotFound(crate::compositor::xdg_shell::types::DomainWindowIdentifier), // Assuming DomainWindowIdentifier will be defined
    #[error("Tried to map a surface that is not an XDG toplevel or popup")]
    NotAnXdgSurface,
    #[error("XDG Toplevel surface already mapped")]
    ToplevelAlreadyMapped,
    #[error("XDG Popup surface already mapped")]
    PopupAlreadyMapped,
    #[error("Parent surface not found for popup or transient window: {0:?}")]
    ParentNotFound(crate::compositor::xdg_shell::types::DomainWindowIdentifier),
    #[error("Invalid popup parent: not an XDG surface")]
    InvalidPopupParent,
    #[error("Positioner logic error: {0}")]
    PositionerError(String),
    #[error("Client provided invalid XDG surface state: {0}")]
    ClientError(String),
    #[error("Internal XDG shell error: {0}")]
    Internal(String),
}
