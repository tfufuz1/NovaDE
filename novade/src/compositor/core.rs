//! Core module for compositor initialization, Wayland global management,
//! and the main event loop.
//!
//! This module houses the primary entry point `run_nova_compositor` for starting
//! the NovaDE Wayland compositor.

#![allow(unused_variables)]
#![allow(dead_code)]

use crate::compositor::{
    backends::winit::{WinitBackendData, WinitBackendEvent}, // Winit backend integration (conceptual for now)
    state::NovaCompositorState, // The central compositor state
};
use smithay::{
    backend::input::InputEvent, // For processing input events
    reexports::{
        calloop::{channel as calloop_channel, EventLoop as CalloopEventLoop, PostAction, LoopSignal}, // Event loop
        wayland_server::{Display, DisplayHandle}, // Wayland display and handle
    },
    reexports::wayland_server::protocol::{wl_compositor::WlCompositor, wl_shm::WlShm, wl_seat::WlSeat}, // Core Wayland protocols
    wayland::{
        compositor::CompositorData,
        seat::{Seat, Capability}, // Seat and its capabilities
        shell::xdg::XdgWmBase,   // XDG Shell protocol
        shm::ShmData,
    },
    input::SeatCapabilities, // Helper for seat capabilities
    utils::{Log, SERIAL_COUNTER}, // Logging and serials for events
};
use std::{error::Error, time::Duration, cell::RefCell, rc::Rc}; // Standard library utilities
use winit::event_loop::EventLoopBuilder; // For Winit event loop (conceptual integration)


/// Main entry point to initialize and run the NovaDE Wayland compositor.
///
/// This function performs the following key steps:
/// 1.  **Initializes Logging:** Sets up a basic `slog` logger.
/// 2.  **Sets up Calloop Event Loop:** Creates a `calloop::EventLoop` to manage various event sources.
/// 3.  **Initializes Wayland Display:** Creates a Smithay `Display` object, which is the core of the Wayland server.
/// 4.  **Initializes Compositor State:** Creates an instance of `NovaCompositorState`, which holds all compositor-wide data.
/// 5.  **Registers Wayland Globals:** Advertises standard Wayland globals to clients, such as:
///     *   `wl_compositor` (via `CompositorData` and `delegate_compositor!`)
///     *   `wl_shm` (via `ShmData` and `delegate_shm!`)
///     *   `wl_seat` (via `SeatState::new_wl_seat` and `delegate_seat!`)
///     *   `xdg_wm_base` (XDG Shell, via `XdgShellState::new_wm_base` and `delegate_xdg_shell!`)
///     *   `wl_output` (via `Output::create_global` within `NovaCompositorState::new`)
/// 6.  **Sets up Backend Event Channel:** Creates a `calloop::channel` to receive events from a backend (conceptually Winit).
/// 7.  **Integrates Event Sources with Calloop:**
///     *   Adds the Wayland display's event source to Calloop for processing client messages.
///     *   Adds the backend event channel receiver to Calloop to handle events like input, resize, and refresh requests.
/// 8.  **Runs the Main Event Loop:** Enters a loop that:
///     *   Dispatches events from Calloop (Wayland messages, backend events).
///     *   Explicitly dispatches pending Wayland client messages.
///     *   (Conceptually) performs rendering based on backend refresh requests.
///     *   Flushes pending Wayland messages to clients.
///
/// # Returns
///
/// * `Ok(())` if the compositor runs and exits gracefully.
/// * `Err(Box<dyn Error>)` if there's an error during setup or execution.
///
/// # Panics
///
/// This function may panic if critical setup steps fail (e.g., Calloop event loop creation).
/// Error handling for fallible operations (like global creation or backend setup) is included.
///
/// # Notes on Backend Integration
///
/// The current implementation conceptually integrates a Winit backend via a channel.
/// A full Winit backend setup would typically involve running Winit's event loop on the
/// main thread and using tools like `calloop-winit` or a dedicated thread to bridge
/// Winit events into the Calloop loop that manages Wayland state. This function outlines
/// the Calloop side of such an integration.
pub fn run_nova_compositor() -> Result<(), Box<dyn std::error::Error>> {
    let logger = slog::Logger::root(slog::Discard, slog::o!()); // Basic stdout logger
    slog::info!(logger, "Starting NovaDE Compositor...");

    // 1. Initialize Calloop event loop
    let mut event_loop: CalloopEventLoop<NovaCompositorState> = CalloopEventLoop::try_new()
        .map_err(|e| format!("Failed to initialize Calloop event loop: {}", e))?;
    let loop_signal: LoopSignal = event_loop.get_signal(); // To gracefully stop the loop

    // 2. Initialize Wayland Display
    let mut display: Display<NovaCompositorState> = Display::new()?;
    let display_handle: DisplayHandle = display.handle();

    // 3. Initialize Compositor State
    let mut compositor_state = NovaCompositorState::new(display_handle.clone(), logger.clone());

    // 4. Register Wayland Globals
    display_handle.create_global::<NovaCompositorState, WlCompositor, _>(5, CompositorData::default());
    display_handle.create_global::<NovaCompositorState, WlShm, _>(1, ShmData::default());

    let seat_name = compositor_state.seat_name.clone();
    let seat_object: Seat<NovaCompositorState> = compositor_state.seat_state.new_wl_seat(&display_handle, &seat_name, logger.clone());
    compositor_state.seat = Some(seat_object); // Store the seat in the state

    if let Some(seat) = compositor_state.seat.as_mut() {
        seat.add_keyboard(Default::default(), 200, 25)?; // Default XKB config, repeat rate, delay
        seat.add_pointer()?;
        slog::info!(logger, "Seat '{}' configured with keyboard and pointer.", seat_name);
    } else {
        // This case should ideally not be reached if seat initialization is correct.
        return Err(Box::from("Failed to store and configure the seat object."));
    }

    compositor_state.xdg_shell_state.new_wm_base(&display_handle);
    // Note: wl_output global is created within NovaCompositorState::new() via output.create_global().

    slog::info!(logger, "Registered core Wayland globals.");

    // 5. Setup Backend Event Channel (conceptual Winit integration)
    let (winit_event_sender, winit_event_receiver) = calloop_channel::channel();
    // TODO: Spawn a thread for the Winit event loop that uses `winit_event_sender`
    //       to send `WinitBackendEvent`s back to this Calloop loop.
    //       This part is OS-dependent and non-trivial.

    // 6. Integrate Wayland events into Calloop
    let wayland_event_source = display.get_event_loop_source()?;
    event_loop.handle().insert_source(
        wayland_event_source,
        // This closure requires access to `display` to dispatch client messages.
        // A common pattern involves Rc<RefCell<Display<State>>> if Display needs to be shared
        // and mutated, or if the main loop data itself owns Display.
        // For simplicity, we assume dispatching happens in the main loop body.
        move |_event, _metadata, state: &mut NovaCompositorState| {
            slog::trace!(state.logger, "Wayland event received (actual dispatch in main loop).");
            // No explicit dispatch here; handled by display.dispatch_clients in the loop.
        },
    )?;

    // 7. Integrate Winit backend events into Calloop
    event_loop.handle().insert_source(
        winit_event_receiver,
        move |event, _metadata, state: &mut NovaCompositorState| {
            match event {
                calloop_channel::Event::Msg(backend_event) => match backend_event {
                    WinitBackendEvent::Input(input_event) => {
                        slog::trace!(state.logger, "Input event from Winit backend: {:?}", input_event);
                        if let Some(seat) = state.seat.as_mut() {
                            seat.process_input_event(state, input_event, SERIAL_COUNTER.next_serial());
                        } else {
                            slog::warn!(state.logger, "Received input event but seat is not initialized in state.");
                        }
                    }
                    WinitBackendEvent::Resized { new_size, new_scale_factor } => {
                        slog::info!(state.logger, "Winit resize event: size {:?}, scale {}", new_size, new_scale_factor);
                        // Output state is updated by WinitBackendData; here we ensure space knows.
                        state.space.map_output(&state.output, (0,0));
                        // TODO: Signal the renderer/backend in WinitBackendData about the resize for damage tracking.
                    }
                    WinitBackendEvent::Refresh => {
                        slog::trace!(state.logger, "Winit refresh request received.");
                        // Actual rendering is typically done in the main loop's idle phase or on a timer.
                        // This event can signal that the backend (e.g., Winit window) needs redrawing.
                        // state.render_needed = true; // Example flag
                    }
                    WinitBackendEvent::CloseRequested => {
                        slog::info!(state.logger, "Close requested from backend, stopping event loop.");
                        loop_signal.stop(); // Signal Calloop to gracefully exit.
                    }
                },
                calloop_channel::Event::Closed => {
                    slog::info!(state.logger, "Winit event channel closed; compositor may continue or stop based on policy.");
                    // loop_signal.stop(); // Optionally stop if backend channel closure is critical.
                }
            }
        },
    )?;

    slog::info!(logger, "Compositor setup complete, starting Calloop event loop...");

    // 8. Run the Main Event Loop
    // Placeholder for WinitBackendData if it were managed by Calloop (more complex setup needed)
    // let mut winit_backend_placeholder: Option<WinitBackendData> = None;

    let mut running = true;
    while running {
        // Dispatch events. A timeout of 0 means it checks for events and returns immediately if none.
        // A small timeout (e.g., 16ms for ~60fps) allows for periodic work like rendering.
        let dispatch_result = event_loop.dispatch(Duration::from_millis(16), &mut compositor_state);

        if let Err(e) = dispatch_result {
            slog::error!(logger, "Error dispatching Calloop event loop: {}", e);
            running = false; // Exit on critical error
        }

        // Explicitly dispatch Wayland client events after Calloop's dispatch.
        if let Err(e) = display.dispatch_clients(&mut compositor_state) {
            slog::error!(logger, "Error dispatching Wayland clients: {}", e);
            // Decide if this is fatal. Some client errors might be recoverable.
        }

        // TODO: Rendering Logic
        // This is where the actual rendering call to the Winit backend would happen.
        // Example:
        // if let Some(backend_data) = &mut winit_backend_placeholder { // If backend is part of Calloop state
        //     if backend_data.render(&compositor_state).is_err() {
        //         slog::warn!(compositor_state.logger, "Rendering failed.");
        //     }
        // } else {
        //     // If Winit runs on a separate thread, rendering is triggered by its refresh events.
        // }
        slog::trace!(compositor_state.logger, "Render cycle placeholder in main loop.");

        // Flush client event queues.
        if display.flush_clients().is_err() {
            slog::warn!(logger, "Error flushing Wayland clients.");
        }

        // Check if the loop signal has been triggered to stop.
        // Calloop's `dispatch` might not immediately exit if `loop_signal.stop()` was called
        // from within an event source's callback, so an explicit check might be desired
        // or rely on `dispatch` erroring out on a `LoopSignal::stop()`.
        // However, modern Calloop often exits from dispatch on signal.stop().
        // If `running` is set to false by a handler (e.g. CloseRequested), that also works.
        // For this sketch, `loop_signal.stop()` in CloseRequested handler is the primary exit.
    }

    slog::info!(logger, "NovaDE Compositor event loop finished.");
    Ok(())
}
```
