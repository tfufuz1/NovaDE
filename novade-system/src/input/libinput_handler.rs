// src/input/libinput_handler.rs
use input::{Libinput, LibinputInterface, Device, event::Event}; // Changed to input
use udev::{Context as UdevContext, Enumerator};
use std::fs::{File, OpenOptions};
use std::os::unix::io::{RawFd, AsRawFd, FromRawFd};
use std::path::Path;
use std::os::unix::fs::OpenOptionsExt;
use tracing::{error, info, warn};

// Minimal interface for udev interaction with libinput
struct MinimalInterface;

impl LibinputInterface for MinimalInterface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read(true)
            .write(true) // Required by input, even if not strictly used for all devices
            .open(path)
            .map(|file| file.into_raw_fd())
            .map_err(|err| {
                error!("Failed to open path {}: {}", path.display(), err);
                err.raw_os_error().unwrap_or(libc::EIO)
            })
    }

    fn close_restricted(&mut self, fd: RawFd) {
        unsafe {
            // Consumes the File object, which closes the fd upon drop
            drop(File::from_raw_fd(fd));
        }
    }
}

pub struct LibinputUdevHandler {
    libinput_context: Libinput, // This type comes from input
    // We might need to store the udev context if we plan to use it later,
    // e.g., for monitoring udev events for hotplugging.
    // _udev_context: UdevContext,
}

impl LibinputUdevHandler {
    pub fn new() -> Result<Self, String> {
        info!("LibinputUdevHandler: Initializing...");

        let udev_context = UdevContext::new().map_err(|e| {
            let err_msg = format!("Failed to create udev context: {}", e);
            error!("{}", err_msg);
            err_msg
        })?;
        info!("LibinputUdevHandler: Udev context created successfully.");

        let mut libinput_context = Libinput::new_with_udev(MinimalInterface); // Libinput::new_with_udev from input
        info!("LibinputUdevHandler: Libinput context created with udev interface.");

        // Assign the udev backend. This tells input to use udev for device discovery.
        // The seat ID is "seat0".
        if let Err(_e) = libinput_context.udev_assign_seat("seat0") {
            let err_msg = "Failed to assign udev backend to libinput context (seat0).".to_string();
            error!("{}", err_msg);
            return Err(err_msg);
        }
        info!("LibinputUdevHandler: Udev backend assigned to seat0 successfully.");

        info!("LibinputUdevHandler: Initialization successful.");
        Ok(Self {
            libinput_context,
        })
    }

    pub fn context(&self) -> &Libinput { // Libinput from input
        &self.libinput_context
    }

    // Expose a mutable context if needed for operations like dispatch
    pub fn context_mut(&mut self) -> &mut Libinput { // Libinput from input
        &mut self.libinput_context
    }

    // Placeholder for future event dispatching, if this handler manages the loop
    pub fn dispatch_events(&mut self) -> Result<(), ()> {
        match self.libinput_context.dispatch() { // dispatch from input
            Ok(_) => Ok(()),
            Err(_) => {
                error!("LibinputUdevHandler: Error during libinput dispatch.");
                Err(())
            }
        }
    }
}

// Optional: A default implementation for when initialization might fail or is not critical.
impl Default for LibinputUdevHandler {
    fn default() -> Self {
        warn!("LibinputUdevHandler: Attempting to create default instance. This might indicate an issue if a real handler was expected.");
        Self::new().expect("Failed to create a default LibinputUdevHandler. Check udev/input availability.")
    }
}
