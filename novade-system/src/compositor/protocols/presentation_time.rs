// novade-system/src/compositor/protocols/presentation_time.rs
// Implementation of the wp_presentation_time Wayland protocol

use smithay::{
    delegate_presentation_time,
    reexports::{
        wayland_protocols::wp::presentation_time::server::{
            wp_presentation::WpPresentation,
            wp_presentation_feedback::{self, WpPresentationFeedback}, // wp_presentation_feedback is per-surface
        },
        wayland_server::{
            protocol::{wl_surface, wl_output, wl_shm, wl_buffer}, // wl_surface is key
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    utils::{Serial, MonotonicTime, ergeben::Handle as ErgebenHandle}, // MonotonicTime for timestamps
    wayland::output::Output, // To associate presentation with an output
    wayland::presentation::{
        PresentationHandler, PresentationState, PresentationFeedbackData, FeedbackType, // FeedbackType for per-surface feedback
        RequestPresentationFeedbackOrigin, // To know who requested feedback
    },
    // For tracking buffer commits and presentation
    backend::renderer::Frame, // Frame is often used in rendering loop and can store presentation time
    desktop::space::SpaceElement, // If we need to find window for a surface
};
use std::{
    sync::{Arc, Mutex},
    time::Duration, // For converting to protocol timestamp format
};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to hold `PresentationState`.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For presentation time, it would need to manage or access:
    // - PresentationState
    // - Information about the rendering loop and vsync timing.
    // - Output information (refresh rate, last presentation time).
}

#[derive(Debug, Error)]
pub enum PresentationTimeError {
    #[error("Surface not found or not valid for presentation feedback")]
    SurfaceNotFound,
    #[error("Output not found or not valid for presentation feedback")]
    OutputNotFound,
    #[error("Clock ID not supported (only CLOCK_MONOTONIC is supported by this compositor)")]
    UnsupportedClock,
}

// The main compositor state (e.g., NovaCompositorState) would implement PresentationHandler
// and store PresentationState.
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub presentation_state: PresentationState,
//     // Access to rendering loop, output details, etc.
//     pub ergeben_handle: Option<ErgebenHandle<()>>, // For scheduling events on frame callbacks
//     ...
// }
//
// impl PresentationHandler for NovaCompositorState {
//     fn presentation_state(&mut self) -> &mut PresentationState {
//         &mut self.presentation_state
//     }
//
//     fn new_feedback(
//        &mut self,
//        feedback: WpPresentationFeedback,
//        surface: wl_surface::WlSurface,
//        origin: RequestPresentationFeedbackOrigin,
//     ) {
//        // Store this feedback object, associate with surface commit.
//        // When the surface is next presented, send feedback events.
//        let data = PresentationFeedbackData::new(feedback, surface, origin);
//        self.presentation_state.add_feedback(data);
//     }
//
//     fn get_feedback_type_for_surface(&self, surface: &wl_surface::WlSurface) -> Option<FeedbackType> {
//        // Determine if a surface should get feedback (e.g., if it's visible and mapped)
//        if is_surface_visible_and_mapped(surface) { // Your logic here
//            Some(FeedbackType::SurfaceThenOutput) // Example: get feedback from surface commit then output presentation
//        } else {
//            None
//        }
//     }
// }
// delegate_presentation_time!(NovaCompositorState);

impl PresentationHandler for DesktopState { // Replace DesktopState with NovaCompositorState
    fn presentation_state(&mut self) -> &mut PresentationState {
        // TODO: Properly integrate PresentationState with DesktopState or NovaCompositorState.
        panic!("PresentationHandler::presentation_state() needs proper integration. DesktopState must own PresentationState.");
        // Example: &mut self.nova_compositor_state.presentation_state
    }

    fn new_feedback(
        &mut self,
        feedback_obj: WpPresentationFeedback, // The Wayland object for this feedback request
        surface: wl_surface::WlSurface,     // The surface this feedback is for
        origin: RequestPresentationFeedbackOrigin, // Who requested it (client or compositor)
    ) {
        // A client (or the compositor itself) has requested presentation feedback for a surface.
        // We need to store this `feedback_obj` and associate it with the next commit/presentation
        // of the `surface`.
        info!(
            "New presentation feedback requested for surface {:?} by {:?}, object: {:?}",
            surface, origin, feedback_obj
        );

        // Smithay's `PresentationState` can manage these feedback objects.
        // We create `PresentationFeedbackData` and add it to the state.
        let feedback_data = PresentationFeedbackData::new(feedback_obj, surface.clone(), origin);

        // This assumes `self.presentation_state()` gives us the `PresentationState` instance.
        self.presentation_state().add_feedback(feedback_data);

        // When the `surface` is next committed and its buffer presented to an output,
        // we will iterate through the pending `PresentationFeedbackData` for that surface,
        // send the appropriate events (presented, discarded) on the `WpPresentationFeedback` object,
        // and then destroy the feedback object. This is typically done in the rendering loop
        // or post-render phase.
        debug!("Stored new feedback data for surface {:?}", surface);
    }


    fn get_feedback_type_for_surface(&self, surface: &wl_surface::WlSurface) -> Option<FeedbackType> {
        // Smithay calls this to determine if and how feedback should be generated for a surface
        // when it's committed.
        // We should return `Some(FeedbackType)` if the surface is eligible for feedback
        // (e.g., it's visible, mapped, and part of the scene graph).
        // Return `None` if feedback should not be generated for this surface at this time.

        // TODO: Implement logic to determine if the surface is currently visible and eligible.
        // This requires access to the window/surface list and their states (mapped, visible etc.).
        // For example, check if `surface` belongs to a mapped `Window` in our `Space`.
        let is_surface_eligible = {
            // Placeholder: Assume all surfaces are eligible for now for skeleton.
            // In reality:
            // - Find the Window associated with this wl_surface.
            // - Check if Window is mapped and visible on an output.
            // - Maybe check if it's not a cursor or drag icon surface if those shouldn't get it.
            true
        };

        if is_surface_eligible {
            // `FeedbackType` determines how feedback is generated:
            // - `Surface`: Feedback based on surface commit time only (less accurate).
            // - `Output`: Feedback based on output presentation time only.
            // - `SurfaceThenOutput`: Combines both; preferred for accuracy.
            // - `ZeroCopy`: For zero-copy direct scanout (implies immediate presentation).
            debug!("Surface {:?} is eligible for presentation feedback (type: SurfaceThenOutput).", surface);
            Some(FeedbackType::SurfaceThenOutput)
        } else {
            debug!("Surface {:?} is NOT eligible for presentation feedback at this time.", surface);
            None
        }
    }
}

// delegate_presentation_time!(DesktopState); // Needs to be NovaCompositorState

/// Call this function after a frame has been presented on an output.
/// It will find all relevant `WpPresentationFeedback` objects associated with the
/// surfaces presented in that frame and send the appropriate feedback events.
///
/// - `presented_surfaces`: A list of surfaces that were part of this frame presentation.
/// - `output`: The Smithay Output object representing the display where presentation occurred.
/// - `present_time`: The timestamp (CLOCK_MONOTONIC) when the frame was actually presented.
///                   This should be as accurate as possible (e.g., from DRM page flip event,
///                   or renderer's swap completion time).
/// - `refresh_cycle`: Duration of one refresh cycle of the output (e.g., 16.66ms for 60Hz).
/// - `seq`: The presentation sequence number (e.g., DRM vblank counter).
/// - `flags`: `wp_presentation_feedback::Kind` flags (vsync, hw_clock, hw_completion, zero_copy).
///
/// `D` is your main compositor state which holds `PresentationState`.
pub fn on_frame_presented<D>(
    compositor_state: &mut D, // Your main compositor state (e.g., NovaCompositorState)
    presented_surfaces_and_commits: &[(wl_surface::WlSurface, Serial)], // Surfaces and their commit serials for this frame
    output: &Output,
    present_time: MonotonicTime,
    refresh_cycle_duration: Duration,
    seq: u64, // Presentation sequence number (e.g. vblank count)
    feedback_flags: wp_presentation_feedback::Kind,
) where
    D: PresentationHandler + AsMut<PresentationState> + 'static, // AsMut might not be needed if PresentationHandler gives &mut
{
    let presentation_state = compositor_state.presentation_state(); // Get &mut PresentationState via handler

    for (surface, commit_serial) in presented_surfaces_and_commits {
        debug!(
            "Processing presentation feedback for surface {:?} (commit serial {:?}) on output '{}' at time {:?}",
            surface, commit_serial, output.name(), present_time
        );

        // Smithay's `PresentationState::surface_presented` handles the logic:
        // - Finds feedback objects for this surface and commit.
        // - Sends `wp_presentation_feedback.presented` or `discarded`.
        // - Marks feedback as completed.
        presentation_state.surface_presented(
            surface.clone(),
            *commit_serial,
            output.clone(), // Smithay Output object
            present_time,
            refresh_cycle_duration,
            seq,
            feedback_flags,
        );
    }

    // Clean up completed feedback objects
    presentation_state.cleanup_completed_feedback();
    debug!("Cleaned up completed presentation feedback objects.");
}


/// Initializes and registers the WpPresentation global.
/// `D` is your main compositor state type.
pub fn init_presentation_time<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed by PresentationState
    // clock_id: u32, // CLOCK_MONOTONIC is usually the only one supported
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<WpPresentation, u32> + Dispatch<WpPresentation, u32, D> + // UserData is clock_id
       Dispatch<WpPresentationFeedback, PresentationFeedbackData, D> +
       PresentationHandler + 'static,
       // D must also own PresentationState.
{
    info!("Initializing WpPresentation global (presentation-time)");

    // Create PresentationState. This state needs to be managed by your compositor (in D).
    // Example: state.presentation_state = PresentationState::new(CLOCK_MONOTONIC_RAW_ID);
    // Smithay's PresentationState takes the clock_id it will operate with.
    // CLOCK_MONOTONIC is specified by the protocol as the one to be used for timestamps.
    // The ID for CLOCK_MONOTONIC for Wayland is often 1.
    // wl_compositor.h defines WL_COMPOSITOR_CLOCK_MONOTONIC = 1
    // However, wayland-rs and wayland-protocols might use different constants or expect direct libc::CLOCK_MONOTONIC.
    // Smithay's `PresentationState::new()` takes the `clock_id` that will be advertised.
    // Let's use `libc::CLOCK_MONOTONIC`.
    let clock_id_to_advertise = libc::CLOCK_MONOTONIC;


    // The WpPresentation global is created. Its UserData is the clock_id.
    display.create_global::<D, WpPresentation, _>(
        1, // protocol version
        clock_id_to_advertise as u32 // Advertise CLOCK_MONOTONIC
    )?;

    // Ensure `delegate_presentation_time!(D)` is called in your main compositor state setup.
    // This macro sets up the necessary Dispatch implementations for:
    // - WpPresentation
    // - WpPresentationFeedback (using PresentationFeedbackData as UserData)
    // It relies on `D` implementing `PresentationHandler` and having `PresentationState`.

    info!("WpPresentation global initialized, advertising clock_id: {}", clock_id_to_advertise);
    Ok(())
}

// TODO:
// - Accurate Timestamps:
//   - The `present_time` provided to `on_frame_presented` must be as accurate as possible.
//     This usually comes from DRM page flip events or renderer's swap completion notifications
//     (e.g., EGL_KHR_partial_update with EGL_TIMESTAMP_PENDING_NV/EGL_TIMESTAMP_SUPPORTED_KHR).
// - Refresh Rate and Sequence Numbers:
//   - `refresh_cycle_duration` and `seq` must be correctly obtained from the display backend (DRM).
// - Feedback Flags:
//   - `feedback_flags` (e.g., `Kind::VSYNC`, `Kind::HW_CLOCK`) should reflect the hardware capabilities
//     and presentation method.
// - State Integration:
//   - `PresentationState` must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `PresentationHandler`.
//   - `delegate_presentation_time!(NovaCompositorState);` macro must be used.
//   - The `on_frame_presented` function must be called at the correct point in the rendering/output loop.
// - Determining Surface Eligibility (`get_feedback_type_for_surface`):
//   - Implement robust logic to check if a surface is visible, mapped, and should receive feedback.
//     This requires access to the compositor's scene graph / window management state.
// - Testing:
//   - Use Wayland clients that utilize presentation time (e.g., mpv, games, some toolkits for smooth animations)
//     to verify that feedback is being sent correctly and improves their behavior.
//   - Check `WAYLAND_DEBUG=1` output from clients to see presentation feedback events.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod presentation_time;
