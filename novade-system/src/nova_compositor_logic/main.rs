//! The main entry point for the Nova Wayland Compositor.
//!
//! This module initializes the compositor, sets up Wayland globals,
//! and runs the main event loop to handle client requests and compositor events.

// Modules are now declared in nova_compositor_logic/mod.rs

use super::state::CompositorState; // Adjusted path
use smithay::reexports::calloop::{EventLoop, LoopHandle, Interest, Mode, PostAction};
use smithay::wayland::seat::Seat; // Smithay's Seat struct, used for type context
use wayland_server::{
    protocol::{wl_compositor, wl_shm, wl_seat, wl_output},
    Display, GlobalData, // Display for socket management, GlobalData for global user_data
};
use std::path::Path; // Used for socket path management (though add_socket_auto handles this mostly)
use std::process::Command; // Potentially for launching helper processes or cleanup, not directly used yet.
use super::protocols::wl_seat::SeatStateGlobalData; // Adjusted path: UserData for the wl_seat global itself
use super::protocols::wl_output::OutputGlobalData; // Adjusted path: UserData for the wl_output global itself

/// Starts the Nova Compositor logic.
pub fn start_nova_compositor_server() {
    // TODO: Initialize a proper logger (e.g., tracing-subscriber or slog)
    println!("Nova Compositor Logic starting (as part of novade-system)...");

    // Create the event loop that will drive the compositor.
    let mut event_loop = EventLoop::<CompositorState>::try_new()
        .expect("Failed to initialize event loop");

    let mut compositor_state = CompositorState::new(&event_loop.handle());

    // Initialize and register wl_compositor global
    let compositor_global = compositor_state.global_list.make_global::<wl_compositor::WlCompositor, GlobalData, CompositorState>(
        &compositor_state.display.handle(),
        4, 
        GlobalData,
    );
    compositor_state.compositor_global = Some(compositor_global);

    // Initialize and register wl_shm global
    let shm_global = compositor_state.global_list.make_global::<wl_shm::WlShm, GlobalData, CompositorState>(
        &compositor_state.display.handle(),
        1, 
        GlobalData,
    );
    compositor_state.shm_global = Some(shm_global);

    // Initialize and register wl_seat global
    // The GlobalDispatch for wl_seat on CompositorState will handle client binds.
    // The data for the global itself is SeatStateGlobalData.
    // The data for the resource created on bind is SeatData<CompositorState> (handled in wl_seat.rs).
    let seat_global = compositor_state.global_list.make_global_with_data::<wl_seat::WlSeat, GlobalData, CompositorState, SeatStateGlobalData>(
        &compositor_state.display.handle(),
        7, // wl_seat version (Smithay examples often use 7)
        GlobalData, // UserData for GlobalDispatch::bind on CompositorState (can be () or GlobalData)
        SeatStateGlobalData::default() // Data associated with the global itself
    );
    compositor_state.seat_global = Some(seat_global);
    println!("wl_seat global created (version 7).");
    
    // Note: Smithay's `Seat::new()` is not directly called here to create a *global* Seat object.
    // Instead, `SeatState` is in `CompositorState`. The `wl_seat` global is registered.
    // When a client binds, `GlobalDispatch::bind` (in wl_seat.rs) uses `SeatData` for the resource.
    // Input events will later be fed into `compositor_state.seat_state.get_seat(&display_handle)` or similar.

    // Initialize and register wl_output global
    let output_global = compositor_state.global_list.make_global_with_data::<wl_output::WlOutput, GlobalData, CompositorState, OutputGlobalData>(
        &compositor_state.display.handle(),
        3, // version for wl_output (version 3 adds scale and done events)
        GlobalData, // UserData for the GlobalDispatch on CompositorState for the wl_output global
        OutputGlobalData::default() // Data for this specific global instance
    );
    compositor_state.output_global = Some(output_global);
    println!("wl_output global created (version 3).");


    // Display Setup
    // The `Display` object is within `compositor_state`.
    // We need to borrow it mutably to call `add_socket_auto`.
    let display_handle = compositor_state.display.handle(); // Get display handle early if needed for seat
    
    // Create the Wayland socket
    // Note: add_socket_auto() adds the socket to the event loop sources internally.
    let listening_socket = match compositor_state.display.add_socket_auto() {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Fatal: Failed to create Wayland socket: {}", e);
            // Consider returning a Result instead of panicking if this is a library function
            panic!("Could not start compositor logic: socket creation failed."); 
        }
    };
    println!("Nova Compositor Logic listening on Wayland socket: {}", listening_socket.to_string_lossy());
    // Setting WAYLAND_DISPLAY is typically for client applications, 
    // but can be useful for testing or if other parts of novade-system expect it.
    std::env::set_var("WAYLAND_DISPLAY", listening_socket.as_os_str());

    // It's important that the LoopHandle is available for dispatching.
    // The EventLoop itself is consumed by run, so we use the handle.
    let loop_handle = event_loop.handle();

    println!("Nova Compositor Logic starting event loop...");
    // Event Loop Execution - this will block the current thread.
    // Consider if this needs to be spawned on a new thread if novade-system has other tasks.
    // For now, direct call as per typical compositor main.
    // The EventLoop `run` method is often preferred for dedicated compositor binaries.
    // However, since this is now a library function, we'll stick to the manual loop
    // for now, as it was structured before, but this is a key area for review.
    // If this function is expected to return, the loop needs a condition to break.
    // For a server, it might run indefinitely until an external signal or specific event.

    loop {
        // Process events from the event loop.
        if let Err(e) = event_loop.dispatch(std::time::Duration::from_millis(16), &mut compositor_state) {
            eprintln!("Error during event loop dispatch: {}", e);
            // Consider returning an error or specific shutdown based on error type
            break; 
        }
        
        // Flush events to clients.
        if let Err(e) = compositor_state.display.flush_clients() {
            eprintln!("Error flushing clients: {}", e);
            // Handle error, possibly break or log
        }
        // A mechanism to break this loop might be needed if `start_nova_compositor_server`
        // is not intended to run forever. For now, it mirrors the original loop.
    }
    
    println!("Nova Compositor Logic shutting down...");
    // Cleanup of the socket might be handled by Display dropping if using add_socket_auto,
    // but an explicit removal might be desired if a fixed path was used or for tests.
    // The original code correctly noted that add_socket_auto() sockets in XDG_RUNTIME_DIR
    // are often cleaned up by the OS.
}
