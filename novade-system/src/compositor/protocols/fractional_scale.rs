// novade-system/src/compositor/protocols/fractional_scale.rs
// Implementation of the wp_fractional_scale_v1 Wayland protocol

use smithay::{
    delegate_fractional_scale_manager,
    reexports::{
        wayland_protocols::wp::fractional_scale::v1::server::{
            wp_fractional_scale_manager_v1::{self, WpFractionalScaleManagerV1},
            wp_fractional_scale_v1::{self, WpFractionalScaleV1, Request as FractionalScaleRequest, Event as FractionalScaleEvent},
        },
        wayland_server::{
            protocol::{wl_surface, wl_output}, // wl_surface is key, wl_output for per-output scaling
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    utils::{Serial, Scale}, // Smithay's Scale type can represent fractional scales
    wayland::{
        compositor::get_role, // To check if a surface has a role
        output::Output, // Smithay's Output abstraction
        fractional_scale::{
            FractionalScaleHandler, FractionalScaleManagerState, FractionalScale, // Smithay's wrapper
            SurfaceData as FractionalScaleSurfaceData, // UserData for surfaces that get WpFractionalScaleV1
        },
        shell::xdg::XdgShellSurfaceData, // Example: XDG toplevels are common candidates for scaling
    },
    desktop::Window, // If we tie scaling preferences to Window objects
};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to hold `FractionalScaleManagerState`.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For Fractional Scaling, it would need to manage or access:
    // - FractionalScaleManagerState
    // - Output list and their individual scale factors (integer and fractional).
    // - Renderer capabilities regarding fractional scaling (e.g., viewport scaling).
    // - Per-window scaling preferences if applicable.
}

#[derive(Debug, Error)]
pub enum FractionalScaleError {
    #[error("Surface already has a fractional scaling object or does not support it")]
    SurfaceError,
    #[error("Invalid scale value: {0}")]
    InvalidScale(String),
}

// The main compositor state (e.g., NovaCompositorState) would implement FractionalScaleHandler
// and store FractionalScaleManagerState.
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub fractional_scale_manager_state: FractionalScaleManagerState,
//     // Access to outputs, windows, renderer
//     ...
// }
//
// impl FractionalScaleHandler for NovaCompositorState {
//     fn fractional_scale_manager_state(&mut self) -> &mut FractionalScaleManagerState {
//         &mut self.fractional_scale_manager_state
//     }
//
//     fn new_fractional_scale(
//         &mut self,
//         scale_obj: WpFractionalScaleV1, // The new Wayland object
//         surface: wl_surface::WlSurface,  // The surface this scale object is for
//     ) {
//         info!("New fractional scale object {:?} created for surface {:?}", scale_obj, surface);
//         // A client has requested fractional scaling information for a surface.
//         // We need to store this `scale_obj` and its association with the `surface`.
//         // Smithay's `FractionalScaleManagerState` and `FractionalScaleSurfaceData` handle this.
//
//         // Determine the current preferred scale for this surface. This might depend on:
//         // - The output(s) the surface is on.
//         // - Global scaling settings.
//         // - Per-window user preferences.
//         let preferred_scale_value = self.get_preferred_scale_for_surface(&surface); // Your logic
//         scale_obj.preferred_scale(preferred_scale_value); // Send initial preferred scale
//
//         // Store FractionalScaleSurfaceData on the surface's UserDataMap
//         with_states(&surface, |states| {
//             states.data_map.insert_if_missing_threadsafe(|| FractionalScaleSurfaceData::new(scale_obj));
//         });
//     }
// }
// delegate_fractional_scale_manager!(NovaCompositorState);

impl FractionalScaleHandler for DesktopState { // Replace DesktopState with NovaCompositorState
    fn fractional_scale_manager_state(&mut self) -> &mut FractionalScaleManagerState {
        // TODO: Properly integrate FractionalScaleManagerState with DesktopState or NovaCompositorState.
        panic!("FractionalScaleHandler::fractional_scale_manager_state() needs proper integration.");
        // Example: &mut self.nova_compositor_state.fractional_scale_manager_state
    }

    fn new_fractional_scale(
        &mut self,
        scale_obj: WpFractionalScaleV1, // The WpFractionalScaleV1 resource created by the client
        surface: wl_surface::WlSurface,  // The wl_surface this scale object is for
    ) {
        info!(
            "New fractional scale object {:?} requested for surface {:?}",
            scale_obj, surface
        );
        // A client application has created a `WpFractionalScaleV1` object for a `surface`.
        // This means the client is interested in receiving preferred fractional scaling information
        // for this surface, to render its content appropriately.

        // We need to:
        // 1. Associate `scale_obj` with `surface`. Smithay's `FractionalScaleSurfaceData`
        //    is used for this, typically stored in the surface's `UserDataMap`.
        //    The `delegate_fractional_scale_manager` macro and `FractionalScaleManagerState`
        //    handle the creation of `FractionalScaleSurfaceData` and its storage.
        //    This handler `new_fractional_scale` is called by Smithay after that setup.

        // 2. Determine the current "preferred scale" for this `surface`.
        //    This scale is a factor (e.g., 1.0, 1.25, 1.5) that the client should use
        //    to scale its content. The value is represented as `numerator / 120`.
        //    So, 1.0 is 120/120, 1.25 is 150/120, 1.5 is 180/120.
        //    The calculation might depend on:
        //    - The scale of the output(s) the surface is currently on.
        //    - Global desktop scaling settings.
        //    - Per-window or per-application user preferences for scaling.
        //    - Compositor's rendering strategy (e.g., if compositor does all scaling, preferred might always be 1.0).

        // TODO: Implement `get_preferred_scale_for_surface` logic.
        // This requires access to output information, window state, and compositor configuration.
        let preferred_scale_numerator = {
            // Placeholder logic: Assume a fixed scale or find the scale of the primary output.
            // For a real implementation:
            // - Find which output(s) the `surface` (or its window) is on.
            // - Get the fractional scale set for that output (e.g., from Output::current_scale()).
            // - If on multiple outputs with different scales, a choice needs to be made (e.g., largest, primary).
            // - Convert this scale factor to the protocol's numerator format (scale * 120).
            let example_scale_factor = 1.25; // e.g., 125% scaling
            (example_scale_factor * 120.0).round() as u32 // = 150
        };
        warn!(
            "Placeholder preferred scale for surface {:?}: {}/120. Implement dynamic calculation.",
            surface, preferred_scale_numerator
        );

        // 3. Send the initial `preferred_scale` event to the client via `scale_obj`.
        scale_obj.preferred_scale(preferred_scale_numerator);
        debug!(
            "Sent initial preferred_scale {} to WpFractionalScaleV1 {:?}",
            preferred_scale_numerator, scale_obj
        );

        // Note: `FractionalScaleSurfaceData` (which holds `scale_obj`) should already be
        // in the surface's UserDataMap, put there by Smithay's manager dispatch logic
        // before this handler is called. We don't need to insert it here again.
        // This handler is more for acting upon the creation, like sending initial state.
    }

    // The `WpFractionalScaleV1` interface does not have any requests from the client,
    // only the `preferred_scale` event from the server. So, no request handlers are needed
    // for `WpFractionalScaleV1` objects themselves in this `FractionalScaleHandler`.
    // The `delegate_fractional_scale_manager` handles destruction.
}

// delegate_fractional_scale_manager!(DesktopState); // Needs to be NovaCompositorState

/// Call this function when the preferred fractional scale for a surface changes.
/// For example, if the surface is moved to an output with a different scale factor,
/// or if global/per-window scaling settings are changed by the user.
///
/// - `surface`: The `wl_surface` whose preferred scale has changed.
/// - `new_preferred_scale_numerator`: The new preferred scale (numerator for factor of 120).
///
/// `D` is your main compositor state which holds `FractionalScaleManagerState`.
pub fn on_surface_scale_changed<D>(
    compositor_state: &mut D, // Your main compositor state (e.g., NovaCompositorState)
    surface: &wl_surface::WlSurface,
    new_preferred_scale_numerator: u32,
) where
    D: AsMut<FractionalScaleManagerState> + 'static, // AsMut to get state. Handler might not be needed if state is public.
{
    // Smithay's `FractionalScaleManagerState::surface_preferred_scale_changed` can be used here.
    // It finds the `WpFractionalScaleV1` object associated with the surface (if any)
    // and sends the `preferred_scale` event.

    let manager_state = compositor_state.as_mut(); // Get &mut FractionalScaleManagerState

    info!(
        "Preferred scale changed for surface {:?} to {}/120. Notifying client.",
        surface, new_preferred_scale_numerator
    );

    manager_state.surface_preferred_scale_changed(surface, new_preferred_scale_numerator);

    // Additionally, the compositor itself needs to react to this scale change:
    // - The surface's buffer scale might change (if client re-renders at new scale).
    // - The compositor might need to adjust how it scales this surface if it's doing client-side scaling.
    // - Layout of other elements might be affected if the surface's effective size changes.
    // This usually involves marking the surface/window for redraw and re-layout.
}


/// Initializes and registers the WpFractionalScaleManagerV1 global.
/// `D` is your main compositor state type.
pub fn init_fractional_scale_manager<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed by FractionalScaleManagerState
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<WpFractionalScaleManagerV1, ()> +
       Dispatch<WpFractionalScaleManagerV1, (), D> +
       Dispatch<WpFractionalScaleV1, FractionalScaleSurfaceData, D> + // UserData for WpFractionalScaleV1
       FractionalScaleHandler + 'static,
       // D must also own FractionalScaleManagerState.
{
    info!("Initializing WpFractionalScaleManagerV1 global (fractional-scale-v1)");

    // Create FractionalScaleManagerState. This state needs to be managed by your compositor (in D).
    // Example: state.fractional_scale_manager_state = FractionalScaleManagerState::new();

    display.create_global::<D, WpFractionalScaleManagerV1, _>(
        1, // protocol version
        () // GlobalData for the manager (unit)
    )?;

    // Ensure `delegate_fractional_scale_manager!(D)` is called in your main compositor state setup.
    // This macro sets up the necessary Dispatch implementations for:
    // - WpFractionalScaleManagerV1
    // - WpFractionalScaleV1 (using FractionalScaleSurfaceData as UserData)
    // It relies on `D` implementing `FractionalScaleHandler` and having `FractionalScaleManagerState`.

    info!("WpFractionalScaleManagerV1 global initialized.");
    Ok(())
}

// TODO:
// - Dynamic Scale Calculation (`get_preferred_scale_for_surface`):
//   - Implement robust logic to determine the correct preferred scale for any given surface.
//     This involves querying output scales, global settings, and potentially per-window settings.
//     It needs to handle cases where a window spans multiple outputs with different scales.
// - Compositor-Side Scaling:
//   - If clients render at a scale different from the output's physical scale (or if the compositor
//     prefers clients to render at logical pixels, i.e., scale 1.0), the compositor's renderer
//     must correctly scale the surface content. This interacts with `wp_viewport_v1`.
//   - The `wl_surface.set_buffer_scale` interface is for integer scales. Fractional scaling
//     usually implies the client renders at the fractional scale, or the compositor scales a
//     buffer rendered at an integer scale (often 1.0) using viewports.
// - Output Scale Management:
//   - The compositor needs a way to manage and update the scale factors of its outputs
//     (e.g., through a configuration file, D-Bus interface, or settings panel).
//   - When an output's scale changes, `on_surface_scale_changed` must be called for all
//     affected surfaces.
// - State Integration:
//   - `FractionalScaleManagerState` must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `FractionalScaleHandler`.
//   - `delegate_fractional_scale_manager!(NovaCompositorState);` macro must be used.
// - Testing:
//   - Test with applications that support fractional scaling (e.g., modern GTK/Qt apps, browsers).
//   - Verify that `preferred_scale` events are sent correctly when surfaces are created or moved
//     between outputs with different scales.
//   - Check visual correctness: sharp text and UI elements at various fractional scales.
//   - Ensure correct interaction with `wl_surface.set_buffer_scale` (integer scaling) and `wp_viewport_v1`.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod fractional_scale;
