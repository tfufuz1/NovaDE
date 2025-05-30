use smithay::backend::input::LibinputInterface;
use std::{fs::OpenOptions, os::unix::io::{AsRawFd, RawFd, FromRawFd}};
use std::path::Path;
use std::os::unix::fs::OpenOptionsExt; // For custom_flags

#[derive(Debug)]
pub struct LibinputSessionManager; // Placeholder, no actual session state for now

impl LibinputSessionManager {
    pub fn new() -> Self {
        LibinputSessionManager
    }
}

impl LibinputInterface for LibinputSessionManager {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, std::io::Error> {
        tracing::warn!(
            "LibinputSessionManager: Attempting direct open_restricted for {:?} with flags {}. This may require root privileges if not using a proper session manager like logind.",
            path, flags
        );
        match OpenOptions::new()
            .read(true)
            .write( (flags & libc::O_RDWR != 0) || (flags & libc::O_WRONLY != 0) )
            .custom_flags(flags)
            .open(path)
        {
            Ok(file) => {
                // Keep the file alive, otherwise the FD will be closed upon drop.
                // Smithay's libinput backend will take ownership of this FD.
                // We need to ensure it's not closed by Rust's File destructor.
                let fd = file.as_raw_fd();
                std::mem::forget(file); // Prevent Rust from closing the FD.
                Ok(fd)
            },
            Err(e) => {
                tracing::error!("Failed to open {:?} directly: {}", path, e);
                Err(e)
            }
        }
    }

    fn close_restricted(&mut self, fd: RawFd) {
        tracing::warn!(
            "LibinputSessionManager: Attempting direct close_restricted for fd {}. Smithay's backend should manage this.",
            fd
        );
        // Smithay's LibinputInputBackend is expected to close the FDs it receives
        // from open_restricted when it is dropped or no longer needs them.
        // Therefore, we should not close it here to avoid a double-close.
        // If Smithay does not close it, it's a resource leak, but closing here
        // risks closing an FD that Smithay might still intend to use or has already closed.
        // Example: unsafe { libc::close(fd); } // Avoid this.
        // To be absolutely safe, one might re-take ownership and let Rust close it,
        // but this is only if Smithay explicitly states it won't close the FD.
        // For now, assume Smithay manages it.
        // Example of re-taking ownership (use with caution):
        // unsafe { std::fs::File::from_raw_fd(fd); } // This would close the FD when `_file` goes out of scope.
    }
}
