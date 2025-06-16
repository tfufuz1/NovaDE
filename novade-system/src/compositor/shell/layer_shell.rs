// novade-system/src/compositor/shell/layer_shell.rs

#![allow(unused_imports)] // Allow unused imports for now, as we are just defining types

use wayland_protocols_wlr::wlr_layer_shell_unstable_v1::server::{
    zwlr_layer_shell_v1::{self, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{self, Layer, Anchor, KeyboardInteractivity, ZwlrLayerSurfaceV1},
};
use wayland_server::{Dispatch, DisplayHandle, GlobalDispatch, New}; // Common server items

// Placeholder for future LayerShellState or similar struct
pub struct LayerShellState;

// Placeholder for future ZwlrLayerShellV1 global handler
impl<D> GlobalDispatch<ZwlrLayerShellV1, (), D> for LayerShellState
where
    D: GlobalDispatch<ZwlrLayerShellV1, ()> + Dispatch<ZwlrLayerShellV1, ()> + 'static,
{
    fn bind(
        _state: &mut D,
        _handle: &DisplayHandle,
        _client: &wayland_server::Client,
        new_global: New<ZwlrLayerShellV1>,
        _data: &(),
    ) {
        // new_global.implement_closure(|request, _dispatch_data| {
        //     // Handle ZwlrLayerShellV1 requests here in the future
        //     tracing::info!("ZwlrLayerShellV1 request: {:?}", request);
        // }, None); // No specific DispatchData for the global itself for now
        tracing::warn!("ZwlrLayerShellV1 global bound, but not yet implemented.");
        let _ = new_global; // Avoid unused variable warning for now
    }
}

// Further implementation will go here (handling requests, managing layer surfaces, etc.)
// For now, we are just checking if the types can be imported and the module compiles.

#[cfg(test)]
mod tests {
    use super::*;

    // A simple test to check if types are accessible.
    // This doesn't test functionality, just type resolution.
    #[test]
    fn types_are_accessible() {
        let _layer: Option<Layer> = None;
        let _anchor: Option<Anchor> = None;
        let _keyboard_interactivity: Option<KeyboardInteractivity> = None;
        // The global and surface types themselves are ZSTs or require more setup to instantiate,
        // so we don't try to create instances here, just ensure the type names are resolved.
        // For example, ZwlrLayerShellV1 is a marker type for the global.
    }
}
