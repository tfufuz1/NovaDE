// system/src/input/libinput_handler/session_interface.rs
use smithay::backend::input::LibinputInterface;
use std::fs::OpenOptions;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::fs::OpenOptionsExt; // For custom_flags
use std::path::Path;

#[derive(Debug)]
pub struct LibinputSessionManager {
    // No fields needed for this basic implementation
    _placeholder: (),
}

impl LibinputSessionManager {
    pub fn new() -> Self {
        tracing::info!(
            "LibinputSessionManager initialized. This basic version attempts direct device access."
        );
        Self { _placeholder: () }
    }
}

impl LibinputInterface for LibinputSessionManager {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, std::io::Error> {
        tracing::debug!(
            "LibinputSessionManager: open_restricted called for path {:?}, flags {}",
            path,
            flags
        );
        // flags are typically O_RDWR | O_NONBLOCK | O_CLOEXEC from libinput
        // We use custom_flags to pass them directly.
        match OpenOptions::new()
            .custom_flags(flags)
            .read(true) // libinput needs to read
            .write(true) // and sometimes write (e.g. for LEDS, disabling touchpad while typing)
            .open(path)
        {
            Ok(file) => {
                let fd = file.as_raw_fd();
                // Important: Prevent closing the file when `file` goes out of scope.
                // Libinput will manage the fd lifetime via close_restricted.
                std::mem::forget(file);
                tracing::info!("Successfully opened device {:?} with fd {}", path, fd);
                Ok(fd)
            }
            Err(e) => {
                tracing::error!(
                    "Failed to open device {:?} with flags {}: {}. Check permissions or udev rules.",
                    path,
                    flags,
                    e
                );
                Err(e)
            }
        }
    }

    fn close_restricted(&mut self, fd: RawFd) {
        tracing::debug!("LibinputSessionManager: close_restricted called for fd {}", fd);
        // Natively close the file descriptor
        let result = unsafe { libc::close(fd) };
        if result != 0 {
            tracing::error!("Failed to close file descriptor {}: {}", fd, std::io::Error::last_os_error());
        } else {
            tracing::info!("Successfully closed file descriptor {}", fd);
        }
    }
}

impl Default for LibinputSessionManager {
    fn default() -> Self {
        Self::new()
    }
}
