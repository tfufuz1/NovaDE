// novade-system/src/compositor/protocols/wlr_layer_shell.rs
// Implementation of the wlr-layer-shell Wayland protocol

use smithay::{
    delegate_layer_shell,
    desktop::{LayerSurface, LayerSurfaceData, LayerShellState, Space}, // Assuming Space might be needed for layout
    reexports::{
        wayland_protocols::wlr::layer_shell::v1::server::{
            zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
            zwlr_layer_surface_v1::{self, ZwlrLayerSurfaceV1},
        },
        wayland_server::{
            protocol::{wl_output, wl_surface},
            Client, DisplayHandle, GlobalDispatch, Dispatch, Resource, UserData,
        },
        calloop::LoopHandle,
    },
    utils::{Logical, Point, Rectangle, Size, Serial},
    wayland::{
        compositor::{CompositorState, CompositorHandler, with_states}, // May not be directly needed here but often related
        shell::wlr_layer::{WlrLayerShellHandler, Layer as WlrLayer, LayerShellRequest}, // Smithay's WlrLayerShellHandler
    },
};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tracing::{info, warn, error, debug};

// Placeholder for DesktopState - this will need to be defined elsewhere
// and properly integrated.
// TODO: Replace with actual DesktopState from the main compositor logic.
// This state would typically hold collections of LayerSurface objects.
#[derive(Debug, Default)]
pub struct DesktopState {
    // pub layer_surfaces: Vec<LayerSurface>, // Or managed by Smithay's LayerShellState
    // For now, we assume LayerShellState within a larger compositor state will manage these.
    // This DesktopState is the same placeholder as in xdg_shell.rs.
    // A unified DesktopState or a global CompositorState is needed.
}

#[derive(Debug, Error)]
pub enum WlrLayerShellError {
    #[error("Surface already has a role")]
    SurfaceHasRole,
    #[error("LayerSurface not found")]
    LayerSurfaceNotFound,
    // Add other specific errors as needed
}

// This struct would typically be part of your main compositor state.
// For example:
// pub struct NovaCompositorState {
//     ...
//     pub layer_shell_state: LayerShellState,
//     pub desktop_state: DesktopState, // Manages actual window/surface collections
//     ...
// }
//
// And NovaCompositorState would implement WlrLayerShellHandler.

impl WlrLayerShellHandler for DesktopState {
    fn layer_shell_state(&mut self) -> &mut LayerShellState {
        // TODO: Properly integrate LayerShellState with DesktopState or a global compositor state.
        // This is a temporary panic to highlight the need for proper integration.
        // Similar to xdg_shell, this state needs to be accessible.
        panic!("WlrLayerShellHandler::layer_shell_state() needs proper integration");
        // Example: &mut self.global_compositor_state.layer_shell_state
    }

    fn new_layer_surface(
        &mut self,
        surface: LayerSurface,
        output: Option<wl_output::WlOutput>,
        layer: WlrLayer, // zwlr_layer_shell_v1::Layer,
        namespace: String,
    ) {
        info!(
            "New layer surface requested: {:?}, output: {:?}, layer: {:?}, namespace: {}",
            surface, output, layer, namespace
        );
        // A new layer surface has been created by a client.
        // We need to add it to our list of managed layer surfaces and configure it.

        // Smithay's LayerSurface object (passed as `surface`) already wraps the wl_surface
        // and contains the initial state.
        // The `LayerShellState` (accessed via `self.layer_shell_state()`) will typically
        // store and manage these LayerSurface instances.

        // Example: (assuming DesktopState or a similar structure manages a Space or list of surfaces)
        // self.space.map_layer(&surface, (0,0)); // Or some initial position
        // self.layer_surfaces.push(surface); // If managing manually outside LayerShellState's direct store

        // The key is that `LayerShellState` tracks these surfaces. We need to ensure
        // our `DesktopState` (or equivalent) can interact with them for rendering and layout.
        // Often, you'd trigger a layout recalculation here.

        // TODO:
        // 1. Store the LayerSurface if not already handled by LayerShellState's internal list.
        //    (Smithay's LayerShellState does keep track of surfaces associated with it).
        // 2. Set its initial properties (output, layer, namespace are already part of LayerSurfaceData).
        // 3. Arrange/configure the surface. This often means calling `surface.send_configure(...)`.
        // 4. Trigger a re-layout of the desktop/output.

        // Example of sending an initial configure:
        let initial_size = Size::from((0, 0)); // Client should propose a size or calculate based on content
        surface.send_configure(initial_size, Serial::now()); // Serial should be tracked
        debug!("Sent initial configure for new layer surface.");

        // TODO: Integrate with a Space or layout manager if applicable.
        // If using smithay::desktop::Space, you might do something like:
        // self.space.map_layer(&surface, location, &mut UserData::default());
        // The location would depend on the layer, anchor, etc.
    }

    fn layer_surface_configure(
        &mut self,
        surface: LayerSurface,
        configure: zwlr_layer_surface_v1::Configure, // This is the event from client
        serial: Serial, // Serial for ack_configure
    ) {
        info!("Layer surface configure request: {:?}, configure: {:?}", surface, configure);
        // The client is acknowledging a configure event sent by the server (us).
        // `configure` contains the serial of the server's configure event that is being acked.
        // `serial` is the serial for THIS ack_configure event itself.

        // Smithay's LayerSurface object handles ack_configure internally to update its pending state.
        // We might need to react to this, e.g., if the client now has a stable size and we can finalize layout.

        // This method is called when the client *requests* a new state by creating a new
        // zwlr_layer_surface_v1 object or by sending requests like `set_size`, `set_anchor`, etc.,
        // which are then committed. Smithay processes these into a `LayerSurfaceConfigure`
        // and then calls this handler method.
        //
        // No, looking at Smithay's `WlrLayerShellHandler::commit` is where changes are applied.
        // This `layer_surface_configure` seems to be more about the compositor *sending* a configure.
        // Let's re-check Smithay's documentation for `WlrLayerShellHandler`.

        // Re-evaluation:
        // The `layer_surface_configure` method in `WlrLayerShellHandler` is called when Smithay
        // determines a new configuration *should be sent* to the client.
        // The `configure` argument here is the `LayerSurfaceConfigure` struct from Smithay,
        // NOT the `zwlr_layer_surface_v1::Configure` event from the client.
        // The `serial` is the one we generate for this configure event.

        // This function is called by smithay when it wants you to send a zwlr_layer_surface_v1.configure event to the client.
        // The `configure` argument is smithay::shell::wlr_layer::LayerSurfaceConfigure
        // This was confusing. The plan refers to `zwlr_layer_surface_v1::Configure` which is an *event from client to server* (ack_configure).
        // Smithay's `WlrLayerShellHandler::layer_surface_configure` is for *server to client* configuration.

        // Let's align with the `smithay::shell::wlr_layer::WlrLayerShellHandler` trait definition:
        // fn layer_surface_configure(
        //     &mut self,
        //     surface: LayerSurface,
        //     configure: LayerSurfaceConfigure, // This is smithay's struct
        //     serial: Serial,
        // )

        // The plan's description for this step seems to be about `ack_configure`.
        // `ack_configure` is a request on `zwlr_layer_surface_v1`. Smithay handles this internally
        // when `Client::commit()` is called for the surface. We might not need a specific handler
        // method for `ack_configure` itself unless we want to react to it *after* Smithay processes it.

        // Let's assume the plan meant to handle the state changes that *lead* to sending a configure,
        // or reacting to a client's commit of new state.

        // Smithay's `LayerSurface::send_configure(size, serial)` is how we send a configure.
        // This handler method (`layer_surface_configure`) is called by Smithay when it has
        // aggregated changes (e.g. from client `set_size`, `set_anchor` requests followed by `commit`)
        // and determined a new configuration (`LayerSurfaceConfigure`) for the surface.
        // Our job here is to decide on the actual size/state and then call `surface.send_pending_configure()`.


        // Correct interpretation: This function is called by Smithay to ask US (the compositor)
        // to decide on the actual configuration for the layer surface.
        // `configure` (of type `smithay::shell::wlr_layer::LayerSurfaceConfigure`) contains the
        // client's desired state (size, anchor, margin, etc.).
        // We must then decide the actual size and send it.

        let new_size = configure.new_size; // This is client's desired size (width, height)
                                     // If (0,0), client wants us to decide.
        let actual_size: Size<i32, Logical>;

        // TODO: Implement actual layout logic here based on:
        // - `configure` (client's desired state: size, anchor, exclusive_zone, margins, layer, output)
        // - Output geometry
        // - Other layer surfaces
        // - Desktop policies (e.g., panel height limits)

        if new_size.w == 0 && new_size.h == 0 {
            // Client wants us to choose the size.
            // This might depend on the layer (e.g., a panel might span the width of the output).
            // For now, a placeholder size.
            actual_size = Size::from((100, 30)); // Placeholder
            warn!("Client requested (0,0) size for layer surface {:?}, using placeholder {:?}", surface, actual_size);
        } else {
            actual_size = new_size;
        }

        // This is the critical part: send the actual configuration to the client.
        // Smithay expects that after this handler, `surface.send_pending_configure()` is called
        // if we want to use the state accumulated in `configure`.
        // Or, we can directly call `surface.send_configure(actual_size, serial_for_send_configure)`.
        // The `serial` argument to *this* function is the serial Smithay generated for this configuration event.

        surface.send_configure(actual_size, serial);
        debug!("Sent configure for layer surface {:?} with size {:?}, serial {}", surface, actual_size, serial);

        // After this, the client is expected to draw into its buffer with `actual_size`
        // and then send `ack_configure` with the `serial` we just used.
    }

    fn layer_surface_closed(&mut self, surface: LayerSurface) {
        info!("Layer surface closed: {:?}", surface);
        // The layer surface has been destroyed by the client.
        // We need to remove it from our managed list and update layout.

        // Smithay's LayerShellState will remove it from its internal tracking.
        // We need to ensure our DesktopState (or equivalent) also reflects this.
        // Example:
        // self.space.unmap_layer(&surface);
        // self.layer_surfaces.retain(|s| s != &surface);

        // TODO:
        // 1. Remove from any layout manager (e.g., Smithay's Space).
        // 2. Remove from any explicit list of layer surfaces we might be keeping.
        // 3. Trigger a re-layout of the desktop/output.
        debug!("Layer surface {:?} removed from compositor.", surface);
    }

    // This method is called when a client commits new state to a layer surface.
    // It's the primary point where we react to client changes like `set_size`, `set_anchor`, etc.
    fn commit(&mut self, surface: &wl_surface::WlSurface) {
        // This `commit` is part of the general `WlrLayerShellHandler` trait,
        // but it's often tied to `CompositorHandler::commit` as well.
        // Smithay's `LayerSurfaceData::commit` handles the protocol details.
        // The `layer_surface_configure` handler is then typically invoked by Smithay
        // if the committed state requires a new configuration to be sent to the client.

        // We might need to trigger a layout update or other actions here
        // if the commit implies changes beyond what `layer_surface_configure` handles.
        // For example, if the content of the surface changed and requires re-rendering,
        // but the size/position (geometry) configuration remains the same.

        // Smithay's internal `dispatch_client` for `wl_surface.commit` will often call
        // `layer_shell_state.surface_commit(surface)` which then might call
        // `WlrLayerShellHandler::layer_surface_configure`.

        // It's usually sufficient to rely on `layer_surface_configure` for geometry changes.
        // If other reactions are needed on commit (e.g. damage tracking for rendering),
        // those would go here or in `CompositorHandler::commit`.
        debug!("Commit received for layer surface: {:?}", surface);
        // We can retrieve the LayerSurface instance using the wl_surface.
        if let Some(layer_surface) = self.layer_shell_state().layer_surface_for(surface) {
            // Now you have the `LayerSurface` and can inspect its current (pending) state.
            // For example, to manually trigger a configure based on its current state:
            // let current_data = layer_surface.get_data(); // Gets LayerSurfaceData
            // let pending_state = current_data.cached_state.pending(); // Access the pending LayerSurfaceState
            // let new_configure = LayerSurfaceConfigure { new_size: pending_state.size, .. }; // Construct as needed
            // self.layer_surface_configure(layer_surface, new_configure, Serial::now());

            // However, Smithay's design is that after a wl_surface.commit, its internal logic
            // for layer shell (specifically LayerShellState::surface_commit) will process the
            // pending state and call `WlrLayerShellHandler::layer_surface_configure` if necessary.
            // So, direct action here might be redundant for configuration, but useful for other effects.
        } else {
            warn!("Commit for a wl_surface not known to be a layer surface: {:?}", surface);
        }
    }
}

// Delegate WlrLayerShell handling to DesktopState (or the main compositor state)
// Similar to xdg_shell, this macro call should be in the module where your main
// compositor state is defined and implements WlrLayerShellHandler.
//
// delegate_layer_shell!(DesktopState);
//
// Commented out for the same reasons as in xdg_shell.rs. It needs to be
// associated with the actual global compositor state structure.
//
// Example with `NovaCompositorState`:
//
// pub struct NovaCompositorState {
//     // ... other states ...
//     pub layer_shell_state: LayerShellState,
//     // ... desktop_state might hold Space, list of windows/layers etc.
// }
//
// impl WlrLayerShellHandler for NovaCompositorState {
//     fn layer_shell_state(&mut self) -> &mut LayerShellState {
//         &mut self.layer_shell_state
//     }
//     // ... other WlrLayerShellHandler methods ...
// }
//
// delegate_layer_shell!(NovaCompositorState);

/// Initializes and registers the WLR Layer Shell global.
///
/// Call this function during your compositor setup.
/// `D` is your main compositor state type.
pub fn init_wlr_layer_shell<D>(
    display: &DisplayHandle,
    // loop_handle: LoopHandle<'static, D>, // If needed by LayerShellState
) -> Result<(), Box<dyn std::error::Error>>
where
    D: GlobalDispatch<ZwlrLayerShellV1, ()> + Dispatch<ZwlrLayerShellV1, UserData, D> +
       Dispatch<ZwlrLayerSurfaceV1, UserData, D> + WlrLayerShellHandler + 'static,
{
    info!("Initializing WLR Layer Shell global");

    // Create LayerShellState. This state needs to be managed by your compositor.
    // let layer_shell_state = LayerShellState::new(); // Store in your global state.

    // The global for zwlr_layer_shell_v1 is created like other globals.
    // display.create_global::<D, ZwlrLayerShellV1, _>(
    //     4, // current version of wlr-layer-shell-unstable-v1 is 4
    //     ()
    // );
    //
    // Similar to XDG Shell, the `LayerShellState` should be part of your main compositor data `D`,
    // `D` should implement `WlrLayerShellHandler`, and `delegate_layer_shell!(D)` should be called.
    // The `display.create_global` call for `ZwlrLayerShellV1` then enables the protocol.

    Ok(())
}

// TODO: Add unit/integration tests for layer shell logic.
// Tests should cover:
// - Surface creation for different layers (top, bottom, overlay).
// - Correct handling of anchor and exclusive_zone.
// - Size negotiation (client proposes size, compositor decides).
// - Surface closure and resource cleanup.
// - Interactions between multiple layer surfaces on the same output.

// Ensure this module is declared in `novade-system/src/compositor/protocols/mod.rs`
// pub mod wlr_layer_shell;
// And that `protocols/mod.rs` is part of `compositor/mod.rs`
// And `compositor` is part of `lib.rs` or `main.rs`.
