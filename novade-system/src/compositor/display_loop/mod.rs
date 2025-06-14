// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! Main event loop for the Novade compositor.
//!
//! This module initializes the compositor state, sets up event sources
//! (Wayland, signals), and runs the main Calloop event loop.

use std::time::Duration;
use calloop::{EventLoop, LoopSignal};
use calloop_signal::{Signal, Signals};
use smithay::reexports::wayland_server::Display;
use smithay::wayland::source::WaylandSource; // Corrected import path
use tracing::{info, error, warn};

use crate::compositor::core::state::DesktopState;
use crate::compositor::core::globals::create_initial_output; // For MVP output

/// Initializes and runs the main compositor event loop.
pub fn run_compositor_event_loop() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    match tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
    {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to initialize tracing_subscriber (possibly already initialized): {}", e);
        }
    }

    info!("Starting Novade Compositor event loop...");

    // --- MVP Development Backend Setup (Winit + WGPU) ---
    // This Winit setup is conceptual for running the compositor in a window for development.
    // In a production scenario, this would be replaced by a DRM/KMS backend or similar.
    use winit::{event_loop::EventLoopBuilder as WinitEventLoopBuilder, window::WindowBuilder, platform::run_return::EventLoopExtRunReturn};
    use std::sync::Arc;
    use crate::renderer::wgpu_renderer::NovaWgpuRenderer;
    use crate::compositor::renderer_interface::abstraction::FrameRenderer; // For active_renderer trait object
    use smithay::utils::{Rectangle, Physical}; // For Rectangle used in render_frame

    // Create a Winit event loop *separate* from Calloop, for managing the host window.
    // For simplicity in MVP, we'll run its event processing also within the Calloop loop.
    let mut winit_event_loop = WinitEventLoopBuilder::<()>::with_user_event().build();
    let window = Arc::new(WindowBuilder::new()
        .with_title("Novade Compositor (MVP Winit/WGPU)")
        .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
        .build(&winit_event_loop)
        .expect("Failed to create Winit window for MVP renderer."));

    let initial_window_size_physical = window.inner_size();
    // This variable is used to track if Winit window size changed for renderer resize.
    let mut current_winit_size_physical_for_render = initial_window_size_physical;
    // --- End MVP Development Backend Setup ---

    // 1. Create the Wayland Display
    let display: Display<DesktopState> = Display::new()?;
    let display_handle = display.handle();

    // 2. Create the Calloop event loop
    let mut event_loop: EventLoop<DesktopState> = EventLoop::try_new()?;
    let loop_handle = event_loop.handle();
    let mut loop_signal = event_loop.get_signal();

    // 3. Initialize DesktopState
    // DesktopState::new expects LoopHandle<'static, Self> and DisplayHandle.
    // We need to ensure the LoopHandle's lifetime matches.
    // For a typical application structure where event_loop outlives or lives as long as DesktopState,
    // this is usually fine. Calloop's LoopHandle is designed to be 'static if the EventLoop is.
    // However, direct construction might be tricky if DesktopState needs to own things tied to the loop
    // that are not 'static.
    // Let's assume DesktopState::new is designed to work with a 'static LoopHandle for now.
    // If DesktopState itself is 'static, then loop_handle can be directly used.
    // The DesktopState::new method takes loop_handle: LoopHandle<'static, Self>
    // This means the DesktopState itself must be 'static or the LoopHandle must be correctly scoped.
    // A common pattern is to have the EventLoop own the DesktopState, or to ensure DesktopState
    // does not store the LoopHandle directly if DesktopState is not 'static.
    // Given DesktopState::new signature, it implies it might store it or pass it along.
    // The LoopHandle obtained from event_loop.handle() is tied to the lifetime of the event_loop.
    // If DesktopState is meant to be the 'data' for EventLoop<DesktopState>, it's fine.

    // We need a way to pass the loop_handle into DesktopState::new.
    // The current signature of DesktopState::new is:
    // pub fn new(loop_handle: LoopHandle<'static, Self>, display_handle: DisplayHandle) -> SystemResult<Self>
    // This is problematic if `Self` (DesktopState) is not 'static.
    // Let's assume for now that the intention is for DesktopState to be the data argument in EventLoop<DesktopState>,
    // and the 'static on LoopHandle inside DesktopState might be an issue to resolve if it stores it.
    // Smithay examples often pass the loop_handle to state.
    // For now, we proceed, but this 'static lifetime might need adjustment in DesktopState's definition
    // or how LoopHandle is stored/used if DesktopState is not 'static.

    let mut desktop_state = DesktopState::new(loop_handle.clone(), display_handle.clone())?;
    info!("DesktopState initialized.");

    // Initialize WGPU Renderer and store in DesktopState
    // This must happen after the Winit window (or other raw window handle provider) is created.
    match NovaWgpuRenderer::new(window.as_ref(), initial_window_size_physical) {
        Ok(renderer) => {
            let wgpu_renderer = Arc::new(std::sync::Mutex::new(renderer));
            desktop_state.active_renderer = Some(wgpu_renderer.clone() as Arc<std::sync::Mutex<dyn FrameRenderer>>);
            desktop_state.wgpu_renderer_concrete = Some(wgpu_renderer); // Store concrete type
            desktop_state.active_renderer_type = crate::compositor::core::state::ActiveRendererType::Wgpu;
            info!("NovaWgpuRenderer initialized and set as active_renderer in DesktopState.");
        }
        Err(e) => {
            error!("Failed to initialize NovaWgpuRenderer: {}", e);
            // Depending on policy, either return error or continue without a hardware renderer.
            // For MVP, if renderer fails, it's critical.
            return Err(Box::new(e));
        }
    };

    // 4. Create initial MVP Wayland output (e.g., "HEADLESS-1" for clients)
    // Even if rendering to a Winit window, Wayland clients need a wl_output.
    create_initial_output(&mut desktop_state);
    info!("Initial Wayland output (e.g., HEADLESS-1) created.");

    // 5. Initialize Input Backend (Libinput)
    info!("Initializing input backend...");
    use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
    // DirectSession might require root or specific permissions.
    // For a production compositor, consider alternatives like LogindSession if applicable.
    use smithay::backend::session::{Session, DirectSession};
    use std::path::Path;

    // Minimal interface for libinput opening/closing devices.
    struct MinimalLibinputInterface;
    impl libinput::Interface for MinimalLibinputInterface {
        fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<std::os::unix::io::RawFd, i32> {
            use std::fs::OpenOptions;
            use std::os::unix::fs::OpenOptionsExt;
            use std::os::unix::io::IntoRawFd;
            // Use libc flags directly for more control if needed, e.g. O_NONBLOCK, O_RDWR
            match OpenOptions::new().custom_flags(flags).read(true).write(true).open(path) {
                Ok(file) => Ok(file.into_raw_fd()),
                Err(e) => Err(e.raw_os_error().unwrap_or(libc::EIO)),
            }
        }
        fn close_restricted(&mut self, fd: std::os::unix::io::RawFd) {
            unsafe {
                // It's good practice to check the return of close, but for this interface, it's often ignored.
                let _ = libc::close(fd);
            };
        }
    }

    // Attempt to create a libinput context.
    let libinput_context_result = libinput::Libinput::new_from_path(MinimalLibinputInterface);

    if let Ok(mut ctx) = libinput_context_result {
        info!("Libinput context initialized successfully.");
        let seat_name_for_session = desktop_state.seat.name().to_string();
        // Assign devices to the seat.
        if ctx.udev_assign_seat(&seat_name_for_session).is_ok() {
            info!("Libinput context successfully assigned to seat: {}", seat_name_for_session);
        } else {
            // This might not be fatal if no devices are found or if udev is not fully available (e.g. in some containerized envs)
            warn!("Failed to assign libinput context to seat {}. Input might be limited or unavailable.", seat_name_for_session);
        }

        // LibinputInputBackend takes ownership of the context.
        let input_backend = LibinputInputBackend::new(ctx, Some(tracing::Span::current().into()));

        // Insert input backend event source
        match event_loop.handle().insert_source(input_backend, move |_event_readiness, _metadata, state| {
            // The LibinputInputBackend's EventSource::process method will call
            // smithay::backend::input::InputBackend::dispatch_input_events,
            // which in turn calls the appropriate methods on our DesktopState (as SeatHandler).
            // No explicit dispatch call is needed here from our side.
            // The `state.dispatch_input_events()` line was conceptual and is handled by the source.
            tracing::trace!("Input backend event source triggered and processed by LibinputInputBackend.");
        }) {
            Ok(_) => info!("Input backend event source inserted into event loop."),
            Err(e) => {
                error!("Failed to insert input backend event source: {}. Physical input will be unavailable.", e);
            }
        }
    } else {
        warn!("Failed to initialize Libinput context. Physical input processing will be skipped. Error: {:?}", libinput_context_result.err());
    }

    // 6. Insert Wayland event source
    let wayland_event_source = WaylandSource::new(display)?;
    event_loop.handle().insert_source(wayland_event_source, |event, _, state| {
        match state.display_handle.dispatch_clients(state) {
            Ok(_) => {}
            Err(e) => {
                error!("Error dispatching Wayland events: {}", e);
            }
        }
    })?;
    info!("Wayland event source inserted into event loop.");

    // 7. Insert Signal handling event source
    let signal_event_source = Signals::new(&[Signal::SIGINT, Signal::SIGTERM])?;
    event_loop.handle().insert_source(signal_event_source, move |event, _, _state| {
        // _state is &mut DesktopState here.
        match event {
            calloop_signal::Event::Signal(sig) => {
                warn!("Received signal {:?}, initiating graceful shutdown.", sig);
                // Set a flag or send a message to stop the event loop.
                // Using LoopSignal to stop the loop from within an event source handler.
                loop_signal.stop();
            }
        }
    })?;
    info!("Signal handling event source (SIGINT, SIGTERM) inserted.");

    // 7. Main event loop
    info!("Starting main event dispatch loop...");
    let mut running = true; // Controls the loop from outside signal handler perspective

    while running {
        match event_loop.dispatch(Some(Duration::from_millis(16)), &mut desktop_state) {
            Ok(_dispatched_count) => {
                // tracing::trace!("Dispatched {} events.", dispatched_count);
                // After dispatching, flush the display to send pending events to clients.
                desktop_state.display_handle.flush_clients()?; // Flush clients
            }
            Err(e) => {
                error!("Error during event loop dispatch: {}", e);
                running = false; // Stop the loop on dispatch error
            }
        }

        // Check if LoopSignal::stop() was called (e.g., by signal handler)
        if !event_loop.is_running() {
            info!("Event loop stop signal received, exiting loop.");
            running = false;
        }

        // TODO MVP: Implement rendering logic here or in a dedicated event source/timer.
        // For now, the loop primarily handles Wayland and signal events.
        // desktop_state.render_frame_if_needed(); // Conceptual: would check damage and render.
    }

    info!("Novade Compositor event loop finished. Shutting down.");
    // Cleanup is largely handled by Drop implementations of Display, EventLoop, DesktopState, etc.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use nix::sys::signal::{kill, Signal as NixSignal};
    use nix::unistd::Pid;
    use std::time::Duration;

    // This test is tricky because it involves an event loop that normally runs indefinitely
    // and signal handling. We'll try to run it for a very short duration or send a signal.

    #[test]
    #[ignore] // Ignored because it involves signals and timing, can be flaky in CI/headless
    fn test_event_loop_starts_and_handles_signal() {
        // To test signal handling, we need to run the loop in a thread
        // and send a signal to the current process.

        let (tx, rx) = std::sync::mpsc::channel();

        let test_thread = thread::spawn(move || {
            // Inform the main thread that we are about to start the loop
            tx.send(()).unwrap();

            // We expect this to block until a signal or error
            let result = run_compositor_event_loop();

            // Check if it exited due to signal (Ok, as loop_signal.stop() is called)
            // or an error during setup.
            if result.is_err() {
                error!("Test event loop failed: {:?}", result.as_ref().err().unwrap());
            }
            assert!(result.is_ok(), "run_compositor_event_loop should exit cleanly on signal or test end.");
        });

        // Wait for the thread to signal it's about to start the loop
        rx.recv_timeout(Duration::from_secs(5)).expect("Test thread did not start loop in time");

        // Give the loop a moment to fully initialize, especially WaylandSource and SignalSource
        thread::sleep(Duration::from_millis(500));

        info!("Sending SIGINT to self to test graceful shutdown...");
        match kill(Pid::this(), NixSignal::SIGINT) {
            Ok(_) => info!("SIGINT sent successfully."),
            Err(e) => error!("Failed to send SIGINT: {}", e), // Log error but continue, test might still pass if loop exits
        }

        // Wait for the event loop thread to finish
        match test_thread.join_timeout(Duration::from_secs(5)) { // Increased timeout
            Ok(_) => info!("Event loop thread joined successfully."),
            Err(_) => {
                panic!("Event loop thread did not terminate gracefully after SIGINT or timed out.");
            }
        }
    }

     #[test]
     fn test_desktop_state_creation_in_loop_context() {
         // This test focuses on the initialization part, not running the full loop.
         // It ensures DesktopState can be created as expected by run_compositor_event_loop.
        let display: Display<DesktopState> = Display::new().unwrap();
        let display_handle = display.handle();
        let mut event_loop: EventLoop<DesktopState> = EventLoop::try_new().unwrap();
        let loop_handle = event_loop.handle();

        match DesktopState::new(loop_handle, display_handle) {
            Ok(_desktop_state) => {
                info!("DesktopState created successfully for test.");
                // Further checks on _desktop_state could be done here if needed.
            }
            Err(e) => {
                panic!("Failed to create DesktopState for test: {}", e);
            }
        }
     }
}
