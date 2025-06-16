use thiserror::Error;
use smithay::wayland::compositor::SurfaceRoleError;
use wayland_server::{backend::ClientId, protocol::wl_surface::WlSurface};

#[derive(Error, Debug)]
pub enum CompositorError { // Renamed from CompositorCoreError
    #[error("Feature is unavailable: {0}")]
    FeatureUnavailable(String),
    #[error("DMABUF import failed: {0}")]
    DmabufImportFailed(String),
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
    // Add other error variants as they become necessary
}
