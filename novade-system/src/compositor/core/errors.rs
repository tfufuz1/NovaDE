use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::reexports::wayland_server::ClientId;
use smithay::wayland::compositor::SurfaceRoleError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompositorCoreError {
    #[error("Failed to create global: {0}")]
    GlobalCreationFailed(String),
    #[error("Surface role error: {0}")]
    RoleError(#[from] SurfaceRoleError),
    #[error("Client data missing for client ID: {0:?}")]
    ClientDataMissing(ClientId),
    #[error("Surface data missing for surface: {0:?}")]
    SurfaceDataMissing(WlSurface),
    #[error("Invalid surface state: {0}")]
    InvalidSurfaceState(String),
    #[error("Renderer initialization failed: {0}")]
    RendererInitializationFailed(String),
    #[error("Display or event loop creation failed: {0}")]
    DisplayOrLoopCreationFailed(String),
    #[error("XWayland initialization failed: {0}")]
    XWaylandInitializationError(String),
}
