pub mod errors;
pub mod buffer_access;

pub use errors::ShmError;
pub use buffer_access::with_shm_buffer_contents;

use crate::compositor::core::state::DesktopState;
use smithay::reexports::wayland_server::{DisplayHandle, GlobalDispatch, DataInit, Client, New};
use smithay::reexports::wayland_server::protocol::wl_shm::WlShm;
use smithay::wayland::shm::ShmState;


// Ensures and logs that the wl_shm global, managed by DesktopState.shm_state, is active.
pub fn create_shm_global(
    desktop_state: &DesktopState,
    _display_handle: &DisplayHandle,
) -> Result<(), String> {

    let _shm_global_ref = desktop_state.shm_state.global();

    tracing::info!(
        "wl_shm Global (managed by ShmState within DesktopState) is active. Supported additional formats by ShmState: {:?}. Standard formats ARGB8888 and XRGB8888 are always available.",
        desktop_state.shm_state.additional_formats()
    );

    Ok(())
}
