// novade-domain/src/window_management_policy/service.rs
use async_trait::async_trait;
use novade_core::types::geometry::{Rect, Point, Size};
use super::types::WindowPlacementInfo;
use std::sync::Arc; // For Arc<Self> in constructor

#[async_trait]
pub trait WindowManagementPolicyService: Send + Sync {
    async fn get_initial_window_geometry(
        &self,
        existing_windows: &[WindowPlacementInfo],
        client_requested_geometry: Option<Rect>,
        output_area: Rect,
    ) -> Rect;
}

pub struct DefaultWindowManagementPolicyService;

impl DefaultWindowManagementPolicyService {
    pub fn new() -> Arc<Self> { // Return Arc<Self> for easy sharing
        Arc::new(Self)
    }
}

#[async_trait]
impl WindowManagementPolicyService for DefaultWindowManagementPolicyService {
    async fn get_initial_window_geometry(
        &self,
        existing_windows: &[WindowPlacementInfo],
        client_requested_geometry: Option<Rect>,
        output_area: Rect,
    ) -> Rect {
        tracing::info!(
            "Calculating initial window geometry. Existing windows: {}, Requested: {:?}, Output area: {:?}",
            existing_windows.len(), client_requested_geometry, output_area
        );

        let default_size = Size::new(800.0, 600.0); // Assuming Rect/Size use f64 or similar float
        // Use client's requested size if valid, otherwise default.
        // Ensure requested size is not zero or negative.
        let mut requested_size = client_requested_geometry.map_or(default_size, |g| {
            if g.size.w > 0.0 && g.size.h > 0.0 { g.size } else { default_size }
        });
        
        // Ensure size is within output area bounds (simplified)
        requested_size.w = requested_size.w.min(output_area.size.w - 20.0).max(1.0); // Ensure min width 1
        requested_size.h = requested_size.h.min(output_area.size.h - 20.0).max(1.0); // Ensure min height 1
        
        let final_size = requested_size;

        if existing_windows.is_empty() {
            // Try to center the first window, or use client's requested position if valid
            let initial_pos = client_requested_geometry.map_or_else(
                || Point::new( // Default centered position
                    output_area.pos.x + (output_area.size.w - final_size.w) / 2.0,
                    output_area.pos.y + (output_area.size.h - final_size.h) / 2.0
                ),
                |g| { // Use client's position if it's reasonable within the output area
                    let client_pos = g.pos;
                    if client_pos.x >= output_area.pos.x &&
                       client_pos.x + final_size.w <= output_area.pos.x + output_area.size.w &&
                       client_pos.y >= output_area.pos.y &&
                       client_pos.y + final_size.h <= output_area.pos.y + output_area.size.h {
                        client_pos
                    } else { // Fallback to centered if client pos is out of bounds
                        Point::new(
                            output_area.pos.x + (output_area.size.w - final_size.w) / 2.0,
                            output_area.pos.y + (output_area.size.h - final_size.h) / 2.0
                        )
                    }
                }
            );
            tracing::debug!("Placing first window at {:?} with size {:?}", initial_pos, final_size);
            return Rect::new(initial_pos, final_size);
        }

        // Simple cascade for subsequent windows
        if let Some(last_window) = existing_windows.last() {
            let mut new_pos = Point::new(
                last_window.geometry.pos.x + 30.0,
                last_window.geometry.pos.y + 30.0,
            );

            // Basic boundary check: if cascade goes off screen, reset near origin (with offset)
            if new_pos.x + final_size.w > output_area.pos.x + output_area.size.w ||
               new_pos.y + final_size.h > output_area.pos.y + output_area.size.h {
                new_pos = Point::new(
                    output_area.pos.x + 70.0, // Offset from corner
                    output_area.pos.y + 70.0  // Offset from corner
                );
            }
            // Further check: ensure the reset position is also on screen
            if new_pos.x + final_size.w > output_area.pos.x + output_area.size.w {
                new_pos.x = output_area.pos.x + (output_area.size.w - final_size.w).max(0.0) / 2.0; // Center if still too wide
            }
            if new_pos.y + final_size.h > output_area.pos.y + output_area.size.h {
                new_pos.y = output_area.pos.y + (output_area.size.h - final_size.h).max(0.0) / 2.0; // Center if still too tall
            }

            tracing::debug!("Cascading window to {:?} with size {:?}", new_pos, final_size);
            return Rect::new(new_pos, final_size);
        }

        // Fallback (should ideally not be reached if existing_windows is not empty and .last() is Some)
        tracing::warn!("Fallback placement used (should not be reached if existing_windows is not empty).");
        Rect::new(
            Point::new(
                output_area.pos.x + (output_area.size.w - final_size.w) / 2.0,
                output_area.pos.y + (output_area.size.h - final_size.h) / 2.0
            ),
            final_size
        )
    }
}

// Example of how to make the service clonable if needed for multiple Arc holders
// impl Clone for DefaultWindowManagementPolicyService {
//     fn clone(&self) -> Self { Self } // If it's a ZST
// }
