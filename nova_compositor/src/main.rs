//! The main entry point for the Nova Wayland Compositor.
//!
//! This module initializes the compositor, sets up Wayland globals,
//! and runs the main event loop to handle client requests and compositor events.

mod error;
mod protocols;
mod state;
mod utils;

use crate::state::CompositorState;
use smithay::reexports::calloop::{EventLoop, LoopHandle, Interest, Mode, PostAction};
use smithay::wayland::seat::Seat; // Smithay's Seat struct, used for type context
use wayland_server::{
    protocol::{wl_compositor, wl_shm, wl_seat, wl_output},
    Display, GlobalData, // Display for socket management, GlobalData for global user_data
};
use std::path::Path; // Used for socket path management (though add_socket_auto handles this mostly)
use std::process::Command; // Potentially for launching helper processes or cleanup, not directly used yet.
use crate::protocols::wl_seat::SeatStateGlobalData; // UserData for the wl_seat global itself
use crate::protocols::wl_output::OutputGlobalData; // UserData for the wl_output global itself

/// The main function that starts the Nova Compositor.
fn main() {
    // TODO: Initialize a proper logger (e.g., tracing-subscriber or slog)
    println!("Nova Compositor starting...");

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
    let display = &mut compositor_state.display;
    
    let socket_name = match display.add_socket_auto() {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Fatal: Failed to create Wayland socket: {}", e);
            panic!("Could not start compositor: socket creation failed.");
        }
    };
    println!("Listening on Wayland socket: {}", socket_name.to_string_lossy());
    std::env::set_var("WAYLAND_DISPLAY", socket_name.as_os_str());


    println!("Starting event loop...");
    // Event Loop Execution
    loop {
        // Process events from the event loop. This includes client requests.
        match event_loop.dispatch(std::time::Duration::from_millis(16), &mut compositor_state) {
            Ok(_) => {
                // Successfully dispatched events
            }
            Err(e) => {
                eprintln!("Error during event loop dispatch: {}", e);
                // Depending on the error, you might want to break the loop or handle it
                break; // For now, break on error
            }
        }
        
        // Flush events to clients.
        // This ensures that any events queued by the handlers are sent to the clients.
        match compositor_state.display.flush_clients() {
            Ok(_) => {
                // Successfully flushed clients
            }
            Err(e) => {
                eprintln!("Error flushing clients: {}", e);
                // Depending on the error, you might want to break or handle
            }
        }
    }
    
    // Cleanup of the socket file is typically handled by the OS when the process exits
    // for sockets created with add_socket_auto(), as they are often in XDG_RUNTIME_DIR.
    // If a fixed path was used, manual cleanup like before would be needed.
    println!("Shutting down Nova Compositor...");
    // No explicit socket_path.exists() and remove_file here as add_socket_auto() is used.
}
