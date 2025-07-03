// novade-system/src/compositor/protocols/viewport.rs
// Implementation of the wp_viewport_v1 Wayland protocol

use smithay::{
    delegate_viewporter, // Assuming a delegate macro similar to others
    reexports::{
        wayland_protocols::wp::viewporter::server::{
            wp_viewport::{self, WpViewport, Request as ViewportRequest},
            wp_viewporter::{self, WpViewporter, Request as ViewporterRequest},
        },
        wayland_server::{
            protocol::wl_surface, // wl_surface is what a viewport is attached to
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    utils::{Serial, Logical, Physical, Rectangle, Size, Scale}, // For handling geometry
    wayland::{
        compositor::{get_role, CompositorHandler, CompositorState, SurfaceData as CompositorSurfaceData}, // SurfaceData for damage tracking
        viewporter::{ViewporterHandler, ViewporterState, Viewport, SurfaceDataExtension}, // Smithay's viewporter types
        // fractional_scale::FractionalScaleSurfaceData, // Might interact if scaling is done via viewport
    },
    desktop::Window, // If viewports affect window geometry calculations
};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState or the main compositor state (e.g., NovaCompositorState)
// This state will need to hold `ViewporterState`.
#[derive(Debug, Default)]
pub struct DesktopState {
    // This is the same placeholder.
    // For Viewporter, it would need to manage or access:
    // - ViewporterState
    // - Renderer capabilities for handling source/destination rectangles.
    // - Information about surface buffer scales and fractional scales.
}

#[derive(Debug, Error)]
pub enum ViewportError {
    #[error("Surface already has a viewport object or does not support it")]
    SurfaceError,
    #[error("Invalid viewport parameters: {0}")]
    InvalidParameters(String), // e.g., negative size, source rect out of bounds
}

// The main compositor state (e.g., NovaCompositorState) would implement ViewporterHandler
// and store ViewporterState.
//
// Example:
// pub struct NovaCompositorState {
//     ...
//     pub viewporter_state: ViewporterState,
//     // Access to renderer, surface states etc.
//     ...
// }
//
// impl ViewporterHandler for NovaCompositorState {
//     fn viewporter_state(&mut self) -> &mut ViewporterState {
//         &mut self.viewporter_state
//     }
//
//     fn new_viewport(&mut self, viewport_obj: WpViewport, surface: wl_surface::WlSurface) {
//         info!("New viewport object {:?} created for surface {:?}", viewport_obj, surface);
//         // A client has requested a viewport for a surface.
//         // Smithay's ViewporterState and its SurfaceDataExtension handle associating
//         // the viewport_obj with the surface and storing its state (source/destination rects).
//
//         // The SurfaceDataExtension (part of ViewporterState) is typically added to
//         // the wl_surface's UserDataMap when the WpViewporter global is bound, or when
//         // get_viewport is first called for a surface. Smithay's delegate macro handles this.
//
//         // When the client sets source/destination rectangles on viewport_obj, those
//         // requests are dispatched, and ViewporterState updates the SurfaceDataExtension.
//         // The compositor's rendering logic then reads this viewport information from
//         // SurfaceDataExtension when drawing the surface.
//     }
// }
// delegate_viewporter!(NovaCompositorState);

impl ViewporterHandler for DesktopState { // Replace DesktopState with NovaCompositorState
    fn viewporter_state(&mut self) -> &mut ViewporterState {
        // TODO: Properly integrate ViewporterState with DesktopState or NovaCompositorState.
        panic!("ViewporterHandler::viewporter_state() needs proper integration.");
        // Example: &mut self.nova_compositor_state.viewporter_state
    }

    fn new_viewport(
        &mut self,
        viewport_resource: WpViewport, // The WpViewport resource created by the client
        surface: wl_surface::WlSurface, // The wl_surface this viewport is for
    ) {
        info!(
            "New viewport object {:?} requested for surface {:?}",
            viewport_resource, surface
        );
        // A client application has created a `WpViewport` object for a `surface`.
        // This means the client intends to define a source rectangle from its buffer
        // and/or a destination rectangle on the surface, effectively allowing cropping and scaling.

        // Smithay's `ViewporterState` manages the state of viewports.
        // When `get_viewport` is called on `WpViewporter`, Smithay:
        // 1. Creates the `WpViewport` resource (`viewport_resource`).
        // 2. Ensures the `surface`'s `UserDataMap` has `SurfaceDataExtension` (from `ViewporterState`).
        //    This extension will store the actual viewport parameters (source rect, dest size).
        // 3. Calls this `new_viewport` handler.

        // The `SurfaceDataExtension` is the key piece of data that the compositor's rendering code
        // will query to apply the viewport. It's updated when the client sends requests
        // like `set_source` or `set_destination` on the `viewport_resource`.

        // No specific action is usually required in this handler *itself* beyond logging,
        // as Smithay has already set up the necessary data structures.
        // The actual viewport logic is applied during rendering by reading from
        // `SurfaceDataExtension` associated with the `wl_surface`.
        debug!(
            "Viewport resource {:?} associated with surface {:?}. Awaiting client configuration.",
            viewport_resource, surface
        );
    }

    // The `WpViewport` interface has requests like `set_source`, `set_destination`, and `destroy`.
    // These are dispatched by Smithay to internal handlers within `ViewporterState` which update
    // the `SurfaceDataExtension` for the associated surface.
    // The `ViewporterHandler` trait does not require us to re-implement these request handlers.
    // We only need to ensure `ViewporterState` is managed and `delegate_viewporter` is used.
}

// delegate_viewporter!(DesktopState); // Needs to be NovaCompositorState

/// Call this function during rendering to get the viewport parameters for a surface.
///
/// - `surface_states`: The `UserDataMap` of the `wl_surface` being rendered.
///
/// Returns `Option<(Option<Rectangle<f64, wl_buffer::Buffer>>, Option<Size<i32, Logical>>)>`
/// where the first element is the source rectangle (in buffer coordinates, fractional)
/// and the second is the destination size (in surface-local logical coordinates).
/// If `None` is returned, no viewport is set or active for this surface.
pub fn get_surface_viewport_info(
    surface_states: &UserDataMap,
) -> Option<(Option<Rectangle<f64, wl_buffer::Buffer>>, Option<Size<i32, Logical>>)> {
    surface_states.get::<SurfaceDataExtension>().and_then(|sde| {
        let state = sde.lock().unwrap(); // SurfaceDataExtension contains a Mutex<ViewportStateInternal>
        if state.is_enabled() {
            Some((state.src_rect, state.dst_size))
        } else {
            None
        }
    })
}

/// Initializes and registers the WpViewporter global.
/// `D` is your main compositor state type.
pub fn init_viewporter<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed by ViewporterState
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<WpViewporter, ()> +
       Dispatch<WpViewporter, (), D> +
       Dispatch<WpViewport, (), D> + // UserData for WpViewport is often unit, state is in SurfaceDataExtension
       ViewporterHandler + CompositorHandler + 'static, // CompositorHandler for SurfaceData access
       // D must also own ViewporterState and CompositorState.
{
    info!("Initializing WpViewporter global (viewporter)");

    // Create ViewporterState. This state needs to be managed by your compositor (in D).
    // Example: state.viewporter_state = ViewporterState::new();
    // ViewporterState needs access to CompositorState to attach its SurfaceDataExtension
    // to wl_surfaces when they are created. This is done via `CompositorState::add_surface_role_data_constructor`.

    // The `ViewporterState` needs to be initialized and told how to add its
    // `SurfaceDataExtension` to newly created `wl_surface`s.
    // This is typically done by passing a closure to `ViewporterState::new` or
    // by calling a method on `CompositorState` to register a role data constructor.
    //
    // Smithay's `ViewporterState::new()` does not take arguments.
    // The mechanism is that `delegate_viewporter!` and the `ViewporterHandler`
    // ensure that when `get_viewport` is called, the `SurfaceDataExtension` is added
    // to the surface's UserDataMap if not already present.
    // This means `CompositorState` doesn't need to be involved at construction time of ViewporterState.

    display.create_global::<D, WpViewporter, _>(
        1, // protocol version
        () // GlobalData for the manager (unit)
    )?;

    // Ensure `delegate_viewporter!(D)` is called in your main compositor state setup.
    // This macro sets up the necessary Dispatch implementations for WpViewporter and WpViewport.
    // It relies on `D` implementing `ViewporterHandler` and having `ViewporterState`.

    info!("WpViewporter global initialized.");
    Ok(())
}

// TODO:
// - Renderer Integration:
//   - The compositor's rendering logic MUST query the viewport state for each surface
//     (using `get_surface_viewport_info` or directly accessing `SurfaceDataExtension`).
//   - If a viewport is active, the renderer must use the source rectangle (for sampling from
//     the buffer texture) and render into the destination rectangle (on the surface).
//   - This affects texture coordinates and quad geometry used for rendering.
// - Interaction with Scaling:
//   - Viewports are often used in conjunction with scaling.
//     - For fractional scaling (`wp_fractional_scale_v1`): A client might render its buffer
//       at a pre-scaled size (e.g., 125% of logical), and the viewport's source rectangle
//       would be the full buffer. The destination size would be the logical size of the surface.
//       The compositor then scales this down during rendering.
//     - Alternatively, a client renders at logical size (scale 1.0), and the compositor uses
//       the viewport (source=full buffer, destination=scaled surface size) to upscale.
//   - Interaction with `wl_surface.set_buffer_scale` (integer scaling): If a buffer has an
//     integer scale factor, the source rectangle for the viewport is in buffer coordinates
//     at that scale, not logical pixels.
// - State Integration:
//   - `ViewporterState` must be part of `NovaCompositorState`.
//   - `NovaCompositorState` must implement `ViewporterHandler`.
//   - `delegate_viewporter!(NovaCompositorState);` macro must be used.
// - Damage Tracking:
//   - Viewport changes (source or destination) should ideally trigger damage on the surface
//     so the compositor re-renders it correctly. Smithay's `SurfaceDataExtension` might
//     handle some of this, or manual damage might be needed.
// - Testing:
//   - Test with applications that use viewports for scaling or cropping (e.g., video players,
//     some games, toolkits like GTK/Qt for certain widgets or fractional scaling).
//   - Verify correct rendering with various source/destination rectangle combinations.
//   - Test behavior when viewport settings change dynamically.
//   - Ensure correct rendering when combined with `wl_surface.set_buffer_scale` and
//     `wp_fractional_scale_v1`.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod viewport;
