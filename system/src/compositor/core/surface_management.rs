use crate::compositor::renderer_interface::{RenderableTexture, DummyRenderableTexture}; // Updated import
use smithay::{
    reexports::wayland_server::{
        protocol::{wl_buffer, wl_surface},
        Client, Resource,
    },
    utils::{BufferCoords, Physical, Point, Rectangle, Serial, Size, Logical}, // Added Logical
    wayland::compositor, // Import the whole module
};
use std::{
    any::Any,
    collections::VecDeque,
    sync::{Arc, Mutex},
};
use thiserror::Error;
use uuid::Uuid;

/// Error type for surface role conflicts.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SurfaceRoleError {
    #[error("Surface already has a role: {current_role}, but tried to assign {new_role}")]
    RoleAlreadyAssigned {
        current_role: &'static str,
        new_role: &'static str,
    },
    #[error("Surface does not have the expected role: {expected_role}")]
    IncorrectRole { expected_role: &'static str },
    #[error("Surface does not have a role assigned")]
    NoRoleAssigned,
}

/// Information about a buffer attached to a surface.
#[derive(Debug, Clone)]
pub struct AttachedBufferInfo {
    pub wl_buffer: wl_buffer::WlBuffer,
    pub offset: Point<i32, Logical>, // Offset of the buffer relative to the surface
    pub scale: i32,                  // Buffer scale
    pub transform: wl_surface::Transform, // Buffer transform
                                     // pub damage: Vec<Rectangle<i32, BufferCoords>>, // Damage in buffer coordinates
                                     // pub opaque_region: Option<Region<i32, BufferCoords>>, // Opaque region in buffer coordinates
                                     // pub input_region: Option<Region<i32, BufferCoords>>, // Input region in buffer coordinates
}

/// Data associated with a `wl_surface`.
#[derive(Debug)]
pub struct SurfaceData {
    pub id: Uuid,
    pub client_id: Option<Arc<dyn Any + Send + Sync>>, // To identify the client owning this surface
    pub role: Mutex<Option<&'static str>>, // e.g., "toplevel", "subsurface", "cursor"
    // pub role_data: Mutex<Option<Box<dyn Any + Send + Sync>>>, // Role-specific data

    // Buffer state
    pub current_buffer_info: Mutex<Option<AttachedBufferInfo>>,
    pub texture_handle: Mutex<Option<Arc<dyn RenderableTexture>>>, // Changed to Arc<dyn RenderableTexture>

    // Surface geometry and state
    pub damage_regions_buffer_coords: Mutex<VecDeque<Rectangle<i32, BufferCoords>>>,
    pub opaque_region: Mutex<Option<Rectangle<i32, Logical>>>, // Opaque region in surface coordinates
    pub input_region: Mutex<Option<Rectangle<i32, Logical>>>,  // Input region in surface coordinates

    // Hierarchy
    pub parent: Mutex<Option<wl_surface::WlSurface>>,
    pub children: Mutex<Vec<wl_surface::WlSurface>>,

    // Commit sequence and hooks
    // pub pre_commit_hooks: Mutex<Vec<Box<dyn Fn(&mut Self) + Send + Sync>>>,
    // pub post_commit_hooks: Mutex<Vec<Box<dyn Fn(&mut Self) + Send + Sync>>>,
    pub current_commit_serial: Mutex<Serial>,
    pub last_commit_serial: Mutex<Serial>,

    // Other attributes
    pub desired_size: Mutex<Option<Size<i32, Logical>>>, // For roles like toplevel
    pub preferred_scale: Mutex<i32>,
}

impl SurfaceData {
    pub fn new(client_id: Option<Arc<dyn Any + Send + Sync>>) -> Self {
        Self {
            id: Uuid::new_v4(),
            client_id,
            role: Mutex::new(None),
            // role_data: Mutex::new(None),
            current_buffer_info: Mutex::new(None),
            texture_handle: Mutex::new(None),
            damage_regions_buffer_coords: Mutex::new(VecDeque::new()),
            opaque_region: Mutex::new(None),
            input_region: Mutex::new(None),
            parent: Mutex::new(None),
            children: Mutex::new(Vec::new()),
            // pre_commit_hooks: Mutex::new(Vec::new()),
            // post_commit_hooks: Mutex::new(Vec::new()),
            current_commit_serial: Mutex::new(Serial::INITIAL),
            last_commit_serial: Mutex::new(Serial::INITIAL),
            desired_size: Mutex::new(None),
            preferred_scale: Mutex::new(1),
        }
    }

    /// Assigns a role to the surface. Returns `SurfaceRoleError` if a role is already assigned.
    pub fn set_role(&self, role_name: &'static str) -> Result<(), SurfaceRoleError> {
        let mut role_guard = self.role.lock().unwrap();
        if let Some(current_role) = *role_guard {
            if current_role != role_name {
                return Err(SurfaceRoleError::RoleAlreadyAssigned {
                    current_role,
                    new_role: role_name,
                });
            }
        }
        *role_guard = Some(role_name);
        Ok(())
    }

    /// Checks if the surface has the specified role.
    pub fn has_role(&self, role_name: &'static str) -> bool {
        *self.role.lock().unwrap() == Some(role_name)
    }

    /// Gets the role of the surface.
    pub fn get_role(&self) -> Option<&'static str> {
        *self.role.lock().unwrap()
    }

    // Add more methods as needed, e.g., for managing hierarchy, damage, etc.
}

impl Drop for SurfaceData {
    fn drop(&mut self) {
        tracing::debug!("SurfaceData dropped for surface ID: {}", self.id);
        // Any explicit cleanup can go here, though RAII handles most things.
        // For example, if texture_handle needed explicit release calls to a renderer.
    }
}

/// Helper function to get a reference to `SurfaceData` from a `wl_surface`.
/// Panics if the `SurfaceData` is not found (which should not happen in a well-behaved compositor).
pub fn get_surface_data(surface: &wl_surface::WlSurface) -> &SurfaceData {
    surface.get_data::<SurfaceData>().unwrap()
}

/// Helper function to get a mutable reference to `SurfaceData` from a `wl_surface`
/// using Smithay's recommended `compositor::with_states` approach for pending state.
///
/// This is a simplified version. A more complete version would interact with pending states
/// and potentially double buffering of surface state.
pub fn with_surface_data_mut<F, R>(surface: &wl_surface::WlSurface, f: F) -> R
where
    F: FnOnce(&mut compositor::SurfaceAttributes) -> R, // SurfaceAttributes is Smithay's pending state
{
    // This is the correct way to access pending state for a surface in Smithay.
    // The SurfaceData defined above is for our persistent state, which complements Smithay's.
    // Logic that needs to modify pending state (like what happens on commit) uses this.
    // Our SurfaceData can be accessed via surface.get_data() for its persistent aspects.
    compositor::with_states(surface, |states| {
        // `states.cached_state.current()` gives access to `SurfaceAttributes`
        // which is what Smithay uses to track pending state for wl_surface.
        // If our `SurfaceData` needs to be involved in commit logic *before* Smithay processes it,
        // this is where we'd interact.
        // However, often our `SurfaceData` is more about tracking *our* extended state
        // that lives alongside Smithay's.
        let surface_attributes = states.cached_state.current::<compositor::SurfaceAttributes>();
        f(surface_attributes)
    })
}

/// Initializes `SurfaceData` for a new `wl_surface`.
/// This should be called when a `wl_surface` is created.
pub fn init_surface_data(surface: &wl_surface::WlSurface, client: Option<&Client>) {
    let client_id = client.map(|c| c.id().into_any_arc()); // Smithay 0.3 provides `id()` on Client.
                                                        // For older versions, you might need a different way to get a client identifier.
    surface.set_data_destructor(destroy_surface_data);
    surface.insert_user_data(|| SurfaceData::new(client_id));
}

/// Destructor for `SurfaceData` when a `wl_surface` is destroyed.
fn destroy_surface_data(data: &SurfaceData) {
    tracing::info!(
        "Destroying SurfaceData for surface ID: {}, client ID: {:?}",
        data.id,
        data.client_id
            .as_ref()
            .map(|id_arc| id_arc.type_id()) // Just logging TypeId as example
    );
    // Perform any cleanup needed when SurfaceData is about to be dropped.
    // This is called by Wayland when the wl_surface resource is destroyed.
    // The actual drop of SurfaceData happens when its Arc count goes to zero.
}

// Example of how you might associate SurfaceData with a WlSurface
// when it's created by a client in your CompositorHandler implementation:
//
// fn created(&mut self, surface: WlSurface) {
//     init_surface_data(&surface, self.client_info(surface.client().unwrap()).unwrap());
//     // ... other surface creation logic
// }
//
// (Assuming ClientInfo is a struct holding client-specific data accessible from DesktopState)

// To integrate with Smithay's commit logic, you'd typically use `CompositorHandler::commit`.
// Inside `commit`, you can access your `SurfaceData` using `get_surface_data(surface)`.
// And if you need to interact with Smithay's view of the surface state (pending buffer, damage, etc.),
// you'd use `smithay::wayland::compositor::with_states`.
//
// Example within CompositorHandler::commit:
//
// fn commit(&mut self, surface: &wl_surface::WlSurface) {
//     let our_data = get_surface_data(surface);
//     let mut our_buffer_info = our_data.current_buffer_info.lock().unwrap();
//
//     smithay::wayland::compositor::with_states(surface, |states| {
//         let attrs = states.cached_state.current::<smithay::wayland::compositor::SurfaceAttributes>();
//         if let Some(new_buffer) = attrs.buffer.as_ref() {
//             if our_buffer_info.as_ref().map(|b| &b.wl_buffer) != Some(new_buffer) {
//                 // Buffer has changed, update our_buffer_info and potentially the texture_handle
//                 *our_buffer_info = Some(AttachedBufferInfo {
//                     wl_buffer: new_buffer.clone(),
//                     offset: attrs.buffer_offset.unwrap_or_default(),
//                     scale: attrs.buffer_scale,
//                     transform: attrs.buffer_transform,
//                 });
//                 // Release old texture, create new one (renderer interaction)
//                 // *our_data.texture_handle.lock().unwrap() = self.renderer.import_buffer(new_buffer, Some(attrs));
//                 tracing::info!("New buffer attached to surface {}", our_data.id);
//             }
//         } else { // Buffer detached
//             if our_buffer_info.is_some() {
//                 *our_buffer_info = None;
//                 // Release texture
//                 // *our_data.texture_handle.lock().unwrap() = None;
//                 tracing::info!("Buffer detached from surface {}", our_data.id);
//             }
//         }
//         // Update damage regions, etc.
//         // *our_data.damage_regions_buffer_coords.lock().unwrap() = attrs.damage.clone();
//     });
//
//     // Trigger rendering for this surface or its outputs
// }

// Note: The `Client` object itself is not Send+Sync. If you need to store client-specific
// information that needs to be shared across threads (though less common for `SurfaceData` itself),
// you'd typically store a client ID or a reference to data stored in `Client::data` (UserDataMap).
// Smithay's `Client::id()` (from version 0.3) returns a `ClientId` which is `Send + Sync + Clone + Eq + Hash`.
// For older versions, `client.object_id()` might be an option, or manually assigning IDs.
// Using `Arc<dyn Any + Send + Sync>` for `client_id` is a flexible way to store a type-erased client identifier.
// A common pattern is to have a `ClientData` struct stored in `Client::data` via `insert_user_data_if_missing`.
// Then `SurfaceData` could hold an `Arc<ClientData>` if it needs to access shared client state.
// For now, `client_id: Option<Arc<dyn Any + Send + Sync>>` is a placeholder.
// A more concrete type like `Option<ClientId>` (if using Smithay 0.3+) would be typical.
// Or if you have your own `ClientTracker` system, an ID from that.
//
// Smithay 0.3 example for getting ClientId:
// let client_id = client.map(|c| c.id());
// SurfaceData::new(client_id)
//
// And then SurfaceData.client_id would be Option<ClientId>
//
// For the current setup, let's assume client_id is just an opaque identifier for now.
// We'll refine client identification and association as we integrate more with DesktopState.
// The `init_surface_data` function shows how to get a `Client` object if the surface is created
// in a context where the client is known (e.g. in `GlobalDispatch::bind` for `wl_compositor`
// or in `CompositorHandler::created`).
//
// The `smithay::wayland::compositor::SurfaceAttributes` struct holds the pending state
// for a `wl_surface` (buffer, scale, transform, damage, opaque/input regions).
// `SurfaceData` is intended to hold *our* additional state that Smithay doesn't manage,
// or a "cached" version of Smithay's state if needed for specific rendering logic.
// The interaction between `SurfaceData` and `SurfaceAttributes` happens during `commit`.
//
// The `with_surface_data_mut` provided is a bit of a misnomer in the context of how Smithay
// handles pending state. It should really be named something like `with_surface_pending_state_mut`.
// Direct mutation of `SurfaceData` typically happens via `surface.get_data::<SurfaceData>().unwrap()`
// and then locking its `Mutex` fields. Smithay's `with_states` is for its own pending state.
// I will keep the function signature as requested but clarify its usage in comments.
// The primary role of `SurfaceData` here is to store persistent custom data for the surface.
//
// Re-evaluating `with_surface_data_mut`: The request might imply that `SurfaceData` itself
// should be the primary store for *all* surface attributes, including those Smithay manages.
// This would mean duplicating state from `SurfaceAttributes` into `SurfaceData` on commit.
// Smithay's model encourages using `SurfaceAttributes` as the source of truth for pending state
// and then applying that to your own representations (like `SurfaceData` or renderer textures) on commit.
//
// Let's stick to `SurfaceData` being our *additional* state and state cached from Smithay
// after a commit. The `with_surface_data_mut` helper should therefore operate on `SurfaceData`.
// For clarity, I will adjust `with_surface_data_mut` to operate on `SurfaceData` via `Mutex`.

// Re-defined `with_surface_data_mut` to work with our `SurfaceData` structure.
// This is more aligned if `SurfaceData` is meant to be the primary mutable store
// that other parts of the compositor interact with.
// However, remember that Smithay's commit model uses `SurfaceAttributes` for pending state.
// Changes made here might need to be synchronized with Smithay's view or vice-versa,
// typically during the `CompositorHandler::commit` processing.

/// Helper function to mutate `SurfaceData` associated with a `wl_surface`.
/// It locks the relevant mutexes within `SurfaceData` as needed.
/// This is a generic helper; specific helpers for common operations might be more ergonomic.
///
/// Example usage:
/// ```
/// // with_surface_data_mut_example(surface, |data| {
/// //     let mut role = data.role.lock().unwrap();
/// //     *role = Some("new_role");
/// // });
/// ```
/// Note: The user of this function is responsible for knowing which fields of `SurfaceData`
/// they intend to modify and handling the locks accordingly.
/// This function itself doesn't abstract away the `Mutex` fields.
/// A more robust approach for specific operations would be methods on `SurfaceData` itself.
/// The request for a generic `with_surface_data_mut` is interpreted as providing access
/// to the `SurfaceData` instance for arbitrary mutations.
pub fn with_surface_data_mut_direct<F, R>(surface: &wl_surface::WlSurface, f: F) -> R
where
    F: FnOnce(&SurfaceData) -> R,
{
    let data = surface.get_data::<SurfaceData>().unwrap();
    f(data)
}

// If the intention of `with_surface_data_mut` was to mirror Smithay's `with_states`
// but for our `SurfaceData`, it implies `SurfaceData` might have its own pending/current
// state logic, which adds complexity.
// For now, `SurfaceData` holds the committed state, and mutations are direct
// (e.g., `get_surface_data(surface).role.lock().unwrap() = ...`).
// The `CompositorHandler::commit` is where `SurfaceAttributes` (Smithay's pending state)
// would be used to update `SurfaceData` and any renderer resources.

// Let's assume `SurfaceData` is the primary, authoritative store for all aspects
// including those that overlap with Smithay's `SurfaceAttributes`.
// This means on `commit`, we'd read from `SurfaceAttributes` and update `SurfaceData`.
// And other parts of the compositor read from `SurfaceData`.

// Example of updating SurfaceData from SurfaceAttributes during commit:
pub fn update_surface_data_from_commit(
    surface: &wl_surface::WlSurface,
    // renderer: &mut YourRenderer, // If texture creation/update is needed
) {
    let data = get_surface_data(surface);

    smithay::wayland::compositor::with_states(surface, |states| {
        let attrs = states.cached_state.current::<compositor::SurfaceAttributes>();

        // Update buffer info
        let mut current_buffer_info_guard = data.current_buffer_info.lock().unwrap();
        if attrs.buffer.is_some() { // Buffer is attached
            let new_wl_buffer = attrs.buffer.as_ref().unwrap().clone();
            let new_offset = attrs.buffer_offset.unwrap_or_default();
            let new_scale = attrs.buffer_scale;
            let new_transform = attrs.buffer_transform;

            let changed = if let Some(ref existing_info) = *current_buffer_info_guard {
                existing_info.wl_buffer != new_wl_buffer ||
                existing_info.offset != new_offset ||
                existing_info.scale != new_scale ||
                existing_info.transform != new_transform
            } else {
                true // Was None, now Some
            };

            if changed {
                *current_buffer_info_guard = Some(AttachedBufferInfo {
                    wl_buffer: new_wl_buffer,
                    offset: new_offset,
                    scale: new_scale,
                    transform: new_transform,
                });
                // Invalidate or update texture
                let mut texture_handle_guard = data.texture_handle.lock().unwrap();
                // Example: Replace with a dummy texture if a real one cannot be created yet.
                // In a real scenario, this would involve calling renderer.create_texture_from_shm(...)
                // *texture_handle_guard = Some(Arc::new(DummyRenderableTexture::new(attrs.buffer_dimensions.map_or(0, |d| d.w as u32), attrs.buffer_dimensions.map_or(0, |d| d.h as u32), None)));
                *texture_handle_guard = None; // Keep as None for now, renderer will populate.
                tracing::info!("Surface {}: Buffer updated or attached. Texture marked for update.", data.id);
            }
        } else { // Buffer is detached
            if current_buffer_info_guard.is_some() {
                *current_buffer_info_guard = None;
                let mut texture_handle_guard = data.texture_handle.lock().unwrap();
                *texture_handle_guard = None; // Release texture by dropping Arc
                tracing::info!("Surface {}: Buffer detached. Texture released.", data.id);
            }
        }

        // Update damage regions
        // Smithay's damage is in surface coordinates, but SurfaceAttributes.damage is buffer damage.
        // For consistency, let's assume damage_regions_buffer_coords stores damage in buffer coordinates.
        let mut damage_guard = data.damage_regions_buffer_coords.lock().unwrap();
        damage_guard.clear();
        for rect in &attrs.damage { // attrs.damage is Vec<Rectangle<i32, BufferCoords>>
            damage_guard.push_back(*rect);
        }

        // Update opaque and input regions (convert from Smithay's global region to local if necessary)
        // Smithay's SurfaceAttributes stores these as Option<Region<i32, Logical>>.
        // Our SurfaceData uses Option<Rectangle<i32, Logical>> which is simpler.
        // This implies we might only store the bounding box of these regions.
        // For now, let's assume we just take them as is if they are simple enough.
        // A full Region object might be needed for more complex shapes.
        *data.opaque_region.lock().unwrap() = attrs.opaque_region.as_ref().and_then(|r| r.extents());
        *data.input_region.lock().unwrap() = attrs.input_region.as_ref().and_then(|r| r.extents());

        // Update commit serials
        *data.last_commit_serial.lock().unwrap() = *data.current_commit_serial.lock().unwrap();
        *data.current_commit_serial.lock().unwrap() = Serial::now(); // Or from event if available

        // Other attributes like preferred_scale might be updated from other Wayland protocols (e.g. xdg_surface)
    });
}
