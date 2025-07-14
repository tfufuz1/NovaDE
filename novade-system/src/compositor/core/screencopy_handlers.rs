// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

use smithay::{
    reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    wayland::screencopy::{ScreencopyHandler, ScreencopyManagerState, FrameState, BufferDamage, CaptureError},
    wayland::shm,
    output::Output,
    utils::{Rectangle, BufferCoord, Transform}, // BufferCoord and Transform might be needed for damage or specific captures
};
use crate::compositor::state::NovadeCompositorState;
// AsRawFd might not be strictly needed if shm::with_buffer_data_and_format is used, which abstracts fd handling.
// use std::os::unix::io::AsRawFd; 

impl ScreencopyHandler for NovadeCompositorState {
    fn screencopy_state(&mut self) -> &mut ScreencopyManagerState {
        &mut self.screencopy_state
    }

    fn capture_output_frame(
        &mut self,
        output: &Output,
        damage: Option<Vec<Rectangle<i32, BufferCoord>>>,
        frame_state: &FrameState, // Contains the WlBuffer and other frame details
    ) -> Result<(), CaptureError> {
        tracing::info!(
            "Attempting dummy capture for output: {}, buffer: {:?}, damage: {:?}",
            output.name(),
            frame_state.buffer.id(), // Access buffer via FrameState
            damage
        );

        let buffer = &frame_state.buffer;

        // First, get buffer properties without holding the slice mutable for too long.
        let frame_info = match shm::with_buffer_data_and_format(buffer, |_slice, data| {
            // data contains format, width, height, stride
            Ok(Some((data.format, data.width, data.height, data.stride)))
        }) {
            Ok(Some(info)) => info,
            Ok(None) => {
                tracing::error!("Buffer {:?} is not an SHM buffer or SHM data access failed early.", buffer.id());
                return Err(CaptureError::UnsupportedBuffer); // Or Temporary if it might become available
            }
            Err(e) => {
                tracing::error!("Failed to access buffer data for {:?} (e.g. not an SHM buffer or other error): {:?}", buffer.id(), e);
                return Err(CaptureError::Temporary); // Indicates a transient issue
            }
        };
        
        let (buffer_format, buffer_width, buffer_height, buffer_stride) = frame_info;

        let pixel_size_bytes = match buffer_format {
            shm::Format::Argb8888 | shm::Format::Xrgb8888 => 4,
            shm::Format::Abgr8888 | shm::Format::Xbgr8888 => 4,
            // Add other common formats if necessary for testing, e.g., RGB formats
            _ => {
                tracing::error!("Unsupported buffer format for dummy capture: {:?} on buffer {:?}", buffer_format, buffer.id());
                return Err(CaptureError::UnsupportedBuffer);
            }
        };

        // Now, get mutable access to the slice to fill it.
        shm::with_buffer_data_and_format(buffer, |slice, data| {
            // It's good practice to re-check properties if concerned about TOCTOU, though less likely here.
            if data.width != buffer_width || data.height != buffer_height || data.stride != buffer_stride {
                tracing::error!("Buffer {:?} properties changed between reads. Aborting dummy capture.", buffer.id());
                // This indicates a more severe issue, possibly client misbehavior or race.
                return Err(shm::BufferAccessError::BadHandle); 
            }

            tracing::info!(
                "Filling buffer {:?} ({}x{} stride {}) with dummy color (magenta). Format: {:?}",
                buffer.id(), buffer_width, buffer_height, buffer_stride, buffer_format
            );

            // Fill the buffer with magenta (or another test color)
            // The exact byte order for magenta (R=FF, G=00, B=FF, A=FF) depends on the format.
            // For ARGB8888: 0xFFFF00FF (Bytes: BB GG RR AA -> FF 00 FF FF)
            // For XRGB8888: 0xFFFFFF00 (Bytes: BB GG RR XX -> FF 00 FF FF, if X is most significant)
            // For ABGR8888: 0xFFFF00FF (Bytes: RR GG BB AA -> FF 00 FF FF)
            // For XBGR8888: 0xFFFFFF00 (Bytes: RR GG BB XX -> FF 00 FF FF)
            // Magenta: R=255, G=0, B=255
            
            let (r, g, b, a) = (255u8, 0u8, 255u8, 255u8); // Magenta

            for y in 0..buffer_height {
                for x in 0..buffer_width {
                    let offset = (y * buffer_stride + x * pixel_size_bytes) as usize;
                    if offset + pixel_size_bytes as usize <= slice.len() {
                        let pixel_slice = &mut slice[offset..(offset + pixel_size_bytes as usize)];
                        match buffer_format {
                            shm::Format::Argb8888 => { // B G R A
                                pixel_slice[0] = b; // Blue
                                pixel_slice[1] = g; // Green
                                pixel_slice[2] = r; // Red
                                pixel_slice[3] = a; // Alpha
                            }
                            shm::Format::Xrgb8888 => { // B G R X (X usually ignored or opaque alpha)
                                pixel_slice[0] = b;
                                pixel_slice[1] = g;
                                pixel_slice[2] = r;
                                pixel_slice[3] = 0xFF; // X (often treated as Alpha, set to opaque)
                            }
                             shm::Format::Abgr8888 => { // R G B A
                                pixel_slice[0] = r; 
                                pixel_slice[1] = g; 
                                pixel_slice[2] = b; 
                                pixel_slice[3] = a;
                            }
                            shm::Format::Xbgr8888 => { // R G B X
                                pixel_slice[0] = r;
                                pixel_slice[1] = g;
                                pixel_slice[2] = b;
                                pixel_slice[3] = 0xFF; 
                            }
                            _ => { /* Should have been caught by pixel_size_bytes check, but defensive */ }
                        }
                    }
                }
            }
            Ok(())
        }).map_err(|e| {
            tracing::error!("Failed to write to buffer {:?} for dummy capture: {:?}", buffer.id(), e);
            // Map shm::BufferAccessError to CaptureError appropriately
            match e {
                shm::BufferAccessError::BadHandle => CaptureError::UnsupportedBuffer, // Or another specific error
                _ => CaptureError::Temporary,
            }
        })?;

        // Smithay's ScreencopyState will call frame_state.ready() internally upon Ok(()) return.
        Ok(())
    }

    // fn frame_destroyed(&mut self, frame: Frame) -> bool;
    // This is now part of ScreencopyManagerState internal logic for Smithay 0.10+
    // No need to implement it here unless custom behavior beyond default logging is needed.
}
