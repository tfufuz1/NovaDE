// In novade-system/src/compositor/wayland_server/protocols/core/wl_shm_pool.rs
use crate::compositor::wayland_server::objects::Interface;
use std::os::unix::io::RawFd; // For storing the fd

pub fn wl_shm_pool_interface() -> Interface { Interface::new("wl_shm_pool") }
pub const WL_SHM_POOL_VERSION: u32 = 1;

// Request Opcodes
pub const REQ_CREATE_BUFFER_OPCODE: u16 = 0;
pub const REQ_DESTROY_OPCODE: u16 = 1;
pub const REQ_RESIZE_OPCODE: u16 = 2;

// Represents the server-side state of a wl_shm_pool
#[derive(Debug)]
pub struct ShmPoolState {
    pub fd: RawFd, // The file descriptor for the shared memory
    pub size: i32, // The size of the shared memory pool
                   // In a real implementation, this might hold a memory map (e.g., Mmap)
                   // or be validated more thoroughly.
}

impl Drop for ShmPoolState {
    fn drop(&mut self) {
        // Close the file descriptor when the pool is no longer needed (e.g., when its ObjectEntry ref_count drops to 0)
        // This should be done carefully; the ObjectEntry's destruction should trigger this.
        // For now, we just note that the FD needs closing.
        // nix::unistd::close(self.fd).ok(); // Example, actual closing needs care
        tracing::debug!("ShmPoolState dropped for fd {}. FD should be closed by owner of ObjectEntry if held.", self.fd);
    }
}

// TODO: Define handlers for create_buffer, destroy, resize
