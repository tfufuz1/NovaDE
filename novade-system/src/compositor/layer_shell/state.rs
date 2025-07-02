use smithay::{
    desktop::LayerSurface, // Changed from smithay::wayland::shell::wlr_layer::LayerSurface due to deprecation/renaming
    utils::{Physical, Point, Rectangle, Size},
    wayland::compositor, // For SurfaceData
    wayland::shell::wlr_layer::{Anchor, Layer, KeyboardInteractivity}, // KeyboardInteractivity might be needed
};
use wayland_server::protocol::wl_surface::WlSurface;

#[derive(Debug, Clone)]
pub struct LayerSurfaceData {
    pub layer_surface: LayerSurface, // Smithay's LayerSurface is an Arc<LayerSurfaceInner>
    pub current_layer: Layer,
    pub anchors: Anchor,
    pub margins: Rectangle<i32, Physical>,
    pub exclusive_zone: i32,
    pub desired_size: Size<i32, Physical>, // Size as (width, height)
    pub z_index: i32, // Added as per requirements, though not directly in LayerSurface state
    pub geometry: Rectangle<i32, Physical>, // Actual calculated geometry
    pub exclusive_zone_changed: bool,
    pub assigned_output_name: Option<String>, // Name of the SmithayOutput this layer is assigned to
    // Smithay's LayerSurface already handles namespace and client data internally
    // pub namespace: String, // To identify the client/surface for logging/debugging
}

impl LayerSurfaceData {
    pub fn new(layer_surface: LayerSurface) -> Self {
        let initial_state = layer_surface.current_state();
        Self {
            layer_surface,
            current_layer: initial_state.layer,
            anchors: initial_state.anchor,
            margins: initial_state.margin,
            exclusive_zone: initial_state.exclusive_zone,
            desired_size: initial_state.size,
            z_index: 0, // Default z_index, client can change this via set_z_index if supported
            geometry: Rectangle::from_loc_and_size(Point::from((0, 0)), initial_state.size), // Initial geometry
            exclusive_zone_changed: false,
            assigned_output_name: None, // Set by DesktopState during new_layer_surface
            // namespace: layer_surface.namespace().to_string(), // Get namespace if available
        }
    }

    /// Updates the internal state based on pending client commits.
    /// This is typically called when the underlying surface is committed.
    pub fn update_from_pending(&mut self) { // Removed surface_data argument as LayerSurface itself holds the state
        let pending_state = self.layer_surface.current_state(); // Use current_state which reflects committed state

        if self.current_layer != pending_state.layer {
            self.current_layer = pending_state.layer;
            // Potentially mark for re-layout or re-rendering based on layer change
        }
        if self.anchors != pending_state.anchor {
            self.anchors = pending_state.anchor;
        }
        if self.margins != pending_state.margin {
            self.margins = pending_state.margin;
        }
        if self.exclusive_zone != pending_state.exclusive_zone {
            self.exclusive_zone = pending_state.exclusive_zone;
            self.exclusive_zone_changed = true;
        }
        if self.desired_size != pending_state.size {
            self.desired_size = pending_state.size;
        }
        // z_index needs a custom protocol extension or similar to be set by client for zwlr_layer_shell_v1
        // For now, it remains unchanged unless we add such support.

        // Note: Actual geometry update is done by position_and_size_for_output
    }

    /// Computes and sets the final position and size of the LayerSurface
    /// based on output geometry, anchors, margins, and desired size.
    pub fn position_and_size_for_output(
        &mut self,
        output_geometry: &Rectangle<i32, Physical>,
        output_usable_area: &Rectangle<i32, Physical>, // Area after considering exclusive zones from other layers
    ) {
        let mut location = Point::from((0, 0));
        let mut size = self.desired_size;

        // If desired_size width or height is 0, it means "fill available" for that dimension.
        // We use output_usable_area for this calculation.
        if size.w == 0 {
            size.w = output_usable_area.size.w - (self.margins.loc.x + self.margins.size.w); // left + right margin
        }
        if size.h == 0 {
            size.h = output_usable_area.size.h - (self.margins.loc.y + self.margins.size.h); // top + bottom margin
        }
        size.w = size.w.max(0); // Ensure non-negative size
        size.h = size.h.max(0); // Ensure non-negative size


        // Horizontal positioning
        if self.anchors.contains(Anchor::LEFT) && self.anchors.contains(Anchor::RIGHT) {
            location.x = output_usable_area.loc.x + self.margins.loc.x;
            size.w = output_usable_area.size.w - (self.margins.loc.x + self.margins.size.w); // left + right margin
        } else if self.anchors.contains(Anchor::LEFT) {
            location.x = output_usable_area.loc.x + self.margins.loc.x;
        } else if self.anchors.contains(Anchor::RIGHT) {
            location.x = output_usable_area.loc.x + output_usable_area.size.w - size.w - self.margins.size.w; // margin.size.w is right margin
        } else { // Centered horizontally
            location.x = output_usable_area.loc.x + (output_usable_area.size.w - size.w) / 2;
        }

        // Vertical positioning
        if self.anchors.contains(Anchor::TOP) && self.anchors.contains(Anchor::BOTTOM) {
            location.y = output_usable_area.loc.y + self.margins.loc.y;
            size.h = output_usable_area.size.h - (self.margins.loc.y + self.margins.size.h); // top + bottom margin
        } else if self.anchors.contains(Anchor::TOP) {
            location.y = output_usable_area.loc.y + self.margins.loc.y;
        } else if self.anchors.contains(Anchor::BOTTOM) {
            location.y = output_usable_area.loc.y + output_usable_area.size.h - size.h - self.margins.size.h; // margin.size.h is bottom margin
        } else { // Centered vertically
            location.y = output_usable_area.loc.y + (output_usable_area.size.h - size.h) / 2;
        }

        // Unconstrained output: If the layer is not anchored to any edge, it can be placed outside the output bounds.
        // Smithay's LayerSurface doesn't have an explicit "unconstrained_output" flag in its state.
        // This behavior is implicitly handled by the client setting size and margins.
        // The positioning logic above respects this. If a client wants to be e.g. 20px off the left
        // of the screen, it would set a left margin of -20 and anchor to the left.

        self.geometry = Rectangle::from_loc_and_size(location, size.w.max(0), size.h.max(0));
        tracing::debug!(surface = ?self.layer_surface.wl_surface(), new_geometry = ?self.geometry, desired_size = ?self.desired_size, anchors = ?self.anchors, margins = ?self.margins, "Calculated geometry for layer surface");

        // After calculating geometry, configure the surface
        // The actual `configure` call to the client is done by LayerSurface::send_configure
        // which is typically called by the compositor after this logic.
        // Example: self.layer_surface.send_configure(size);
        // This also implies the compositor needs to call .ack_configure() on the surface
        // when the client acknowledges the configure event.
    }

    pub fn wl_surface(&self) -> &WlSurface {
        self.layer_surface.wl_surface()
    }

    pub fn namespace(&self) -> String {
        self.layer_surface.namespace().to_string()
    }

    // Helper to check if this layer surface affects the usable area of an output
    pub fn has_exclusive_zone(&self) -> bool {
        self.exclusive_zone > 0
    }

    // Helper to get the layer type for sorting
    pub fn layer(&self) -> Layer {
        self.current_layer
    }

    // Placeholder for keyboard interactivity. Smithay's LayerSurface has `can_receive_keyboard_focus`
    // and `keyboard_interactivity` in its state.
    pub fn keyboard_interactivity(&self) -> KeyboardInteractivity {
        self.layer_surface.current_state().keyboard_interactivity
    }
}

// Example of how DesktopState might store these (to be defined elsewhere)
/*
use std::collections::HashMap;
use wayland_server::protocol::wl_output::WlOutput;

pub struct OutputState {
    pub wl_output: WlOutput,
    pub geometry: Rectangle<i32, Physical>,
    pub usable_area: Rectangle<i32, Physical>, // Area available for normal windows
    // ... other output specific state
}

impl OutputState {
    pub fn recalculate_usable_area(&mut self, layer_surfaces: &[LayerSurfaceData]) {
        let mut exclusive_top = 0;
        let mut exclusive_bottom = 0;
        let mut exclusive_left = 0;
        let mut exclusive_right = 0;

        for ls_data in layer_surfaces {
            // Only consider layer surfaces on this output.
            // This requires associating layer surfaces with outputs, which LayerSurface::output() does.
            // if ls_data.layer_surface.output().as_ref() != Some(&self.wl_output) {
            //     continue;
            // }

            if ls_data.has_exclusive_zone() {
                // Simplified: assumes exclusive zone is from the edge the layer is anchored to.
                // A more robust implementation would check actual geometry overlap.
                if ls_data.anchors.contains(Anchor::TOP) && !ls_data.anchors.contains(Anchor::BOTTOM) {
                    exclusive_top = exclusive_top.max(ls_data.exclusive_zone);
                }
                if ls_data.anchors.contains(Anchor::BOTTOM) && !ls_data.anchors.contains(Anchor::TOP) {
                    exclusive_bottom = exclusive_bottom.max(ls_data.exclusive_zone);
                }
                if ls_data.anchors.contains(Anchor::LEFT) && !ls_data.anchors.contains(Anchor::RIGHT) {
                    exclusive_left = exclusive_left.max(ls_data.exclusive_zone);
                }
                if ls_data.anchors.contains(Anchor::RIGHT) && !ls_data.anchors.contains(Anchor::LEFT) {
                    exclusive_right = exclusive_right.max(ls_data.exclusive_zone);
                }
            }
        }

        self.usable_area = Rectangle {
            loc: Point::from((
                self.geometry.loc.x + exclusive_left,
                self.geometry.loc.y + exclusive_top,
            )),
            size: Size::from((
                (self.geometry.size.w - (exclusive_left + exclusive_right)).max(0),
                (self.geometry.size.h - (exclusive_top + exclusive_bottom)).max(0),
            )),
        };
    }
}
*/
