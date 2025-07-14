// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! Main event loop for the Novade compositor.
//!
//! This module is responsible for:
//! 1. Initializing the Wayland display and the main `DesktopState`.
//! 2. Setting up the Calloop event loop.
//! 3. Inserting various event sources into the loop:
//!    - Wayland client communication.
//!    - OS signal handling (SIGINT, SIGTERM) for graceful shutdown.
//!    - Input backend events (e.g., from `libinput`).
//!    - Idle notification timer.
//! 4. Running the event dispatch loop.
//! 5. (Conceptually) Integrating a rendering backend for development (e.g., Winit + WGPU).

use std::time::Duration;
use calloop::{EventLoop, LoopSignal};
use calloop_signal::{Signal, Signals}; // For OS signal handling
use smithay::reexports::wayland_server::Display;
use smithay::wayland::source::WaylandSource;
use tracing::{info, error, warn};

use crate::compositor::state::DesktopState;
// For MVP, an initial output is created. In production, this would come from DRM/KMS.
use crate::compositor::core::globals::create_initial_output;

/// Initializes and runs the main compositor event loop.
///
/// This function sets up all necessary components for the compositor to run,
/// including Wayland display, event sources for input and signals, and the
/// main state. It then enters the Calloop event dispatch cycle.
pub fn run_compositor_event_loop() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging via tracing_subscriber.
    // Uses RUST_LOG environment variable for filtering.
    match tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
    {
        Ok(_) => (),
        Err(e) => {
            // e.g. if another part of the application already initialized it.
            eprintln!("Failed to initialize tracing_subscriber (possibly already initialized): {}", e);
        }
    }

    info!("Starting Novade Compositor event loop...");

    // --- MVP Development Backend Setup (Winit + WGPU) ---
    // This section is primarily for development, allowing the compositor to run
    // in a window on an existing desktop environment.
    // A production compositor would typically use a DRM/KMS backend directly.
    #[cfg(feature = "backend_winit")] // Conditional compilation for Winit backend
    let (mut winit_event_loop, window, initial_window_size_physical) = {
        use winit::{event_loop::EventLoopBuilder as WinitEventLoopBuilder, window::WindowBuilder, platform::run_return::EventLoopExtRunReturn};
        use std::sync::Arc;
        // Create a Winit event loop *separate* from Calloop for managing the host window.
        let winit_event_loop = WinitEventLoopBuilder::<()>::with_user_event().build();
        let window = Arc::new(WindowBuilder::new()
            .with_title("Novade Compositor (MVP Winit/WGPU)")
            .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
            .build(&winit_event_loop)
            .expect("Failed to create Winit window for MVP renderer."));
        let initial_size = window.inner_size();
        (winit_event_loop, window, initial_size)
    };
    // --- End MVP Development Backend Setup ---

    // 1. Create the Wayland Display object. This is the core of the Wayland server.
    let mut display: Display<DesktopState> = Display::new()?; // Changed to mut for init_client
    let display_handle = display.handle();

    // 2. Create the Calloop event loop. This will manage all event sources.
    let mut event_loop: EventLoop<DesktopState> = EventLoop::try_new()?;
    let loop_handle = event_loop.handle();
    let mut loop_signal = event_loop.get_signal(); // To stop the loop gracefully

    // 3. Initialize DesktopState, the main state container for the compositor.
    // Note: `DesktopState::new` takes `&mut EventLoop` to potentially insert sources
    // or get a correctly scoped LoopHandle.
    let mut desktop_state = DesktopState::new(&mut event_loop, &mut display)?; // Pass mut display
    info!("DesktopState initialized.");

    // Initialize WGPU Renderer if the feature is enabled and store in DesktopState
    #[cfg(feature = "backend_winit")]
    {
        use crate::renderer::wgpu_renderer::NovaWgpuRenderer;
        use crate::compositor::renderer_interface::abstraction::FrameRenderer;
        use std::sync::Arc;

        match NovaWgpuRenderer::new(&window, initial_window_size_physical) {
            Ok(renderer) => {
                let wgpu_renderer_arc = Arc::new(std::sync::Mutex::new(renderer));
                // desktop_state.active_renderer = Some(wgpu_renderer_arc.clone() as Arc<std::sync::Mutex<dyn FrameRenderer>>);
                // desktop_state.wgpu_renderer_concrete = Some(wgpu_renderer_arc);
                // desktop_state.active_renderer_type = crate::compositor::state::ActiveRendererType::Wgpu;
                info!("NovaWgpuRenderer initialized and would be set as active_renderer in DesktopState.");
            }
            Err(e) => {
                error!("Failed to initialize NovaWgpuRenderer: {}. Proceeding without WGPU renderer.", e);
                // For MVP, this might be critical, but allow continuing for core protocol testing.
            }
        };
    }


    // 4. Create initial Wayland output (e.g., "HEADLESS-1" or "WINIT-1") for clients.
    // Even if rendering to a Winit window, Wayland clients need a `wl_output`.
    create_initial_output(&mut desktop_state); // This function is in core/globals.rs
    info!("Initial Wayland output created and mapped to space.");

    // 5. Initialize Input Backend (e.g., Libinput).
    // This setup is simplified; a real compositor might use a session manager like `logind`.
    info!("Initializing input backend (libinput)...");
    #[cfg(feature = "backend_libinput")] // Assuming a feature flag for libinput
    {
        use smithay::backend::libinput::{LibinputInputBackend, LibinputSessionInterface};
        use std::path::Path;

        struct MinimalLibinputInterface;
        impl libinput::Interface for MinimalLibinputInterface {
            fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<std::os::unix::io::RawFd, i32> {
                use std::fs::OpenOptions;
                use std::os::unix::fs::OpenOptionsExt;
                use std::os::unix::io::IntoRawFd;
                match OpenOptions::new().custom_flags(flags).read(true).write(true).open(path) {
                    Ok(file) => Ok(file.into_raw_fd()),
                    Err(e) => Err(e.raw_os_error().unwrap_or(libc::EIO)),
                }
            }
            fn close_restricted(&mut self, fd: std::os::unix::io::RawFd) {
                unsafe { let _ = libc::close(fd); };
            }
        }

        match libinput::Libinput::new_from_path(MinimalLibinputInterface) {
            Ok(mut libinput_context) => {
                info!("Libinput context initialized successfully.");
                let seat_name = desktop_state.primary_seat.name().to_string();
                if libinput_context.udev_assign_seat(&seat_name).is_ok() {
                    info!("Libinput context successfully assigned to seat: {}", seat_name);
                } else {
                    warn!("Failed to assign libinput context to seat {}. Input may be limited.", seat_name);
                }
                let input_backend = LibinputInputBackend::new(libinput_context, Some(tracing::Span::current().into()));
                match event_loop.handle().insert_source(input_backend, |_, _, state| {
                    // Smithay's LibinputInputBackend dispatches events to SeatHandler methods on `state`
                    state.record_user_activity(true); // Record generic input activity here
                }) {
                    Ok(_) => info!("Input backend event source inserted."),
                    Err(e) => error!("Failed to insert input backend event source: {}. Physical input unavailable.", e),
                }
            }
            Err(e) => {
                warn!("Failed to initialize Libinput context: {:?}. Physical input processing will be skipped.", e);
            }
        }
    }


    // 6. Insert Wayland event source to handle client connections and requests.
    let wayland_event_source = WaylandSource::new(display).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?; // Ensure correct error type
    event_loop.handle().insert_source(wayland_event_source, |event_count, _, state| {
        // `dispatch_clients` processes all pending messages from clients.
        // It calls the appropriate `Dispatch::request` methods on `DesktopState`.
        match state.display_handle.dispatch_clients(state) {
            Ok(_) => { /* debug!("Dispatched {} Wayland client events.", event_count); */ } // event_count is from WaylandSource, not dispatch_clients
            Err(e) => error!("Error dispatching Wayland client events: {}", e),
        }
    })?;
    info!("Wayland event source inserted into event loop.");

    // 7. Insert OS Signal handling event source for graceful shutdown.
    let signal_event_source = Signals::new(&[Signal::SIGINT, Signal::SIGTERM])?;
    event_loop.handle().insert_source(signal_event_source, move |event, _, _state| {
        match event {
            calloop_signal::Event::Signal(sig) => {
                warn!("Received signal {:?}, initiating graceful shutdown.", sig);
                loop_signal.stop(); // Signal Calloop to stop the event loop.
            }
        }
    })?;
    info!("Signal handling event source (SIGINT, SIGTERM) inserted.");

    // 8. Insert Idle Check Timer.
    // The timer will periodically call `DesktopState::check_for_idle_state`.
    let idle_check_initial_delay = desktop_state.idle_timeout;
    let timer_source = calloop::timer::Timer::new()?;
    let idle_timer_handle = timer_source.handle(); // Get handle before moving timer_source

    event_loop.handle().insert_source(timer_source, move |_event_instant, _metadata, state| {
        state.check_for_idle_state(); // This method will also reschedule the timer.
    })?;
    idle_timer_handle.add_timeout(idle_check_initial_delay, ()); // Add initial timeout, data is ()
    desktop_state.idle_timer_handle = Some(idle_timer_handle); // Store the handle in DesktopState
    info!("Idle check timer inserted, initial timeout: {:?}", idle_check_initial_delay);


    // 9. Main event loop execution.
    info!("Starting main event dispatch loop...");
    let mut main_loop_running = true;

    while main_loop_running {
        // Dispatch events from all sources. Timeout of 16ms aims for ~60 FPS responsiveness.
        match event_loop.dispatch(Some(Duration::from_millis(16)), &mut desktop_state) {
            Ok(_dispatched_count) => {
                // After dispatching client messages, flush any pending events to clients.
                if let Err(e) = desktop_state.display_handle.flush_clients() {
                    error!("Error flushing Wayland clients: {}", e);
                    // This might indicate a client has disconnected improperly or a deeper issue.
                    // Depending on the error, might need to take action (e.g. disconnect client).
                }
            }
            Err(e) => {
                error!("Error during event loop dispatch: {}", e);
                main_loop_running = false; // Stop the loop on a fatal dispatch error
            }
        }

        // Check if LoopSignal::stop() was called (e.g., by OS signal handler).
        if !event_loop.is_running() {
            info!("Event loop stop signal received, exiting main_loop_running.");
            main_loop_running = false;
        }

        // TODO MVP: Implement rendering logic here, or better, in a dedicated timed event source
        // or driven by damage events from `Space::elements_for_output`.
        // For now, the loop primarily handles Wayland, OS signals, input, and idle timer events.
        // e.g., desktop_state.render_frame_if_needed();
    }

    info!("Novade Compositor event loop finished. Shutting down.");
    // Resources managed by Calloop (event sources) and Smithay (Display, Client, etc.)
    // are generally cleaned up when they are dropped.
    Ok(())
}

#[cfg(test)]
mod tests {
    // Basic test to ensure DesktopState can be created, which is a prerequisite
    // for the event loop to run with it.
    #[test]
     fn test_desktop_state_creation_for_loop_context() {
        use smithay::reexports::wayland_server::Display;
        use calloop::EventLoop;
        use crate::compositor::state::DesktopState;

        let mut display: Display<DesktopState> = Display::new().unwrap();
        let mut event_loop: EventLoop<DesktopState> = EventLoop::try_new().unwrap();

        match DesktopState::new(&mut event_loop, &mut display) {
            Ok(_desktop_state) => {
                info!("DesktopState created successfully for test context.");
            }
            Err(e) => {
                panic!("Failed to create DesktopState for test context: {}", e);
            }
        }
     }
}
