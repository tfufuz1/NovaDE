use smithay::{
    delegate_compositor, delegate_damage_tracker, delegate_dmabuf, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell, // Added delegate_dmabuf
    reexports::{
        calloop::{EventLoop, LoopHandle},
        wayland_server::{
            backend::ClientData,
            protocol::{
                wl_buffer::WlBuffer, wl_compositor::WlCompositor, wl_shm::WlShm,
                wl_subcompositor::WlSubcompositor, wl_surface::WlSurface,
            },
            Client, DataInit, Display, DisplayHandle, GlobalDispatch, New, Resource,
        },
    },
    utils::{Clock, Logical, Point, Rectangle, Buffer as SmithayBuffer},
    wayland::{
        compositor::{
            self, add_destruction_hook, CompositorClientState, CompositorHandler, CompositorState,
            SurfaceAttributes as WlSurfaceAttributes, SubsurfaceRole,
        },
        output::OutputManagerState,
        shm::{BufferHandler, ShmHandler, ShmState},
        shell::xdg::XdgShellState,
        dmabuf::DmabufState, // Added DmabufState
    },
    backend::renderer::utils::buffer_dimensions,
    desktop::{Space, DamageTrackerState},
    output::Output,
    input::{Seat, SeatState, pointer::CursorImageStatus},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};
use crate::compositor::surface_management::{AttachedBufferInfo, SurfaceData}; 
use crate::compositor::core::ClientCompositorData;
use crate::compositor::xdg_shell::types::{DomainWindowIdentifier, ManagedWindow};


// Main desktop state
pub struct DesktopState {
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, Self>,
    pub clock: Clock<u64>,
    pub compositor_state: CompositorState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub renderer: Option<crate::compositor::renderers::gles2::renderer::Gles2Renderer>,
    pub xdg_shell_state: XdgShellState,
    pub space: Space<ManagedWindow>,
    pub windows: HashMap<DomainWindowIdentifier, Arc<ManagedWindow>>,
    pub outputs: Vec<Output>,
    pub last_render_time: Instant,
    pub damage_tracker_state: DamageTrackerState,
    pub seat_state: SeatState<Self>,
    pub seat_name: String,
    pub seat: Seat<Self>,
    pub pointer_location: Point<f64, Logical>,
    pub current_cursor_status: Arc<Mutex<CursorImageStatus>>,
    pub dmabuf_state: DmabufState, // Added dmabuf_state
}

impl DesktopState {
    pub fn new(event_loop: &mut EventLoop<'static, Self>, display_handle: DisplayHandle) -> Self {
        let loop_handle = event_loop.handle();
        let clock = Clock::new(None).expect("Failed to create clock");

        let compositor_state = CompositorState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle);
        let space = Space::new(tracing::info_span!("novade_space"));
        let damage_tracker_state = DamageTrackerState::new();
        let mut seat_state = SeatState::new();
        let seat_name = "seat0".to_string();
        let seat = seat_state.new_wl_seat(&display_handle, seat_name.clone(), Some(tracing::Span::current()));
        let dmabuf_state = DmabufState::new(); // Initialize DmabufState

        Self {
            display_handle,
            loop_handle,
            clock,
            compositor_state,
            shm_state,
            output_manager_state,
            renderer: None,
            xdg_shell_state,
            space,
            windows: HashMap::new(),
            outputs: Vec::new(),
            last_render_time: Instant::now(),
            damage_tracker_state,
            seat_state,
            seat_name,
            seat,
            pointer_location: (0.0, 0.0).into(),
            current_cursor_status: Arc::new(Mutex::new(CursorImageStatus::Default)),
            dmabuf_state, // Store initialized DmabufState
        }
    }
}

impl CompositorHandler for DesktopState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        client
            .get_data::<ClientCompositorData>()
            .expect("ClientCompositorData not initialized for this client.") // As per plan
            .compositor_state() // Assuming ClientCompositorData has a method to get its internal state
    }

    fn new_surface(&mut self, surface: &WlSurface) {
        let client_id = surface.client().expect("Surface must have a client.").id(); // As per plan
        tracing::info!(surface_id = ?surface.id(), ?client_id, "New WlSurface created");

        // SurfaceData initialization from `crate::compositor::surface_management`
        let surface_data = Arc::new(Mutex::new(
            crate::compositor::surface_management::SurfaceData::new(client_id),
        ));
        
        surface.data_map().insert_if_missing_threadsafe(move || surface_data);

        add_destruction_hook(surface, |data_map_of_destroyed_surface| {
            let surface_data_arc = data_map_of_destroyed_surface
                .get::<Arc<Mutex<crate::compositor::surface_management::SurfaceData>>>()
                .expect("SurfaceData missing in destruction hook")
                .clone();
            
            let surface_id_for_log = { 
                let sd = surface_data_arc.lock().unwrap(); 
                sd.id // Assuming SurfaceData has a UUID id field
            };
            tracing::info!(
                "WlSurface with internal ID {:?} destroyed, SurfaceData cleaned up from UserDataMap.",
                surface_id_for_log
            );
            // Further cleanup (layout, renderer) would be triggered from here or by this.
        });
    }

    fn commit(&mut self, surface: &WlSurface) {
        tracing::debug!(surface_id = ?surface.id(), "Commit received for WlSurface");

        smithay::wayland::compositor::with_states(surface, |states| {
            let surface_data_arc = states
                .data_map
                .get::<Arc<Mutex<crate::compositor::surface_management::SurfaceData>>>()
                .expect("SurfaceData missing on commit")
                .clone();
            
            let mut surface_data = surface_data_arc.lock().unwrap();
            let current_surface_attributes = states.cached_state.current::<WlSurfaceAttributes>();

            // Buffer Handling & Damage
            if current_surface_attributes.buffer.is_some() {
                let buffer_object = current_surface_attributes.buffer.as_ref().unwrap();
                // For Smithay 0.10, buffer_dimensions might come from wl_buffer directly or renderer utils.
                // Assuming smithay::backend::renderer::utils::buffer_dimensions is available and works.
                let dimensions = buffer_dimensions(buffer_object); 

                let new_buffer_info = crate::compositor::surface_management::AttachedBufferInfo {
                    buffer: buffer_object.clone(),
                    scale: current_surface_attributes.buffer_scale,
                    transform: current_surface_attributes.buffer_transform,
                    dimensions: dimensions.map_or_else(Default::default, |d| d.size), // Handle Option from buffer_dimensions
                };
                surface_data.current_buffer_info = Some(new_buffer_info);
                tracing::debug!(
                    surface_id = ?surface.id(),
                    "Attached new buffer. Dimensions: {:?}, Scale: {}, Transform: {:?}",
                    dimensions, current_surface_attributes.buffer_scale, current_surface_attributes.buffer_transform
                );
                // TODO: Mark for texture recreation/update by the renderer
            } else if current_surface_attributes.buffer.is_none() { // Explicitly handle buffer detachment
                surface_data.current_buffer_info = None;
                tracing::debug!(surface_id = ?surface.id(), "Buffer detached.");
            }
            
            // Smithay 0.10 `SurfaceAttributes.damage` is `Vec<Rectangle<i32, Buffer>>`
            // which is `damage_buffer_coords`
            let previous_buffer_id = surface_data.current_buffer_info.as_ref().map(|info| info.buffer.id());
            let new_buffer_wl = current_surface_attributes.buffer.as_ref();
            let new_buffer_id = new_buffer_wl.map(|b| b.id());

            let new_buffer_attached = new_buffer_wl.is_some() && new_buffer_id != previous_buffer_id;
            let buffer_detached = new_buffer_wl.is_none() && previous_buffer_id.is_some();

            if new_buffer_attached {
                let buffer_to_texture = new_buffer_wl.unwrap();
                if let Some(renderer) = self.renderer.as_mut() {
                    match renderer.create_texture_from_shm(buffer_to_texture) {
                        Ok(new_texture) => {
                            surface_data.texture_handle = Some(new_texture);
                            tracing::info!("Created new texture for surface {:?}", surface.id());
                            
                            // Update current_buffer_info in SurfaceData
                            let dimensions = buffer_dimensions(buffer_to_texture).map_or_else(Default::default, |d| d.size);
                            surface_data.current_buffer_info = Some(crate::compositor::surface_management::AttachedBufferInfo {
                                buffer: buffer_to_texture.clone(),
                                scale: current_surface_attributes.buffer_scale,
                                transform: current_surface_attributes.buffer_transform,
                                dimensions,
                            });
                        }
                        Err(e) => {
                            tracing::error!("Failed to create texture for surface {:?}: {:?}", surface.id(), e);
                            surface_data.texture_handle = None;
                            surface_data.current_buffer_info = None; // Clear buffer info if texture creation failed
                        }
                    }
                } else {
                    tracing::warn!("No renderer to create texture for surface {:?}", surface.id());
                    // Buffer is attached but no renderer, so clear existing texture and buffer info if any,
                    // as they are now out of sync with the client's idea of the buffer.
                    surface_data.texture_handle = None;
                    surface_data.current_buffer_info = None;
                }
            } else if buffer_detached {
                tracing::info!("Buffer detached from surface {:?}, clearing texture and buffer info.", surface.id());
                surface_data.texture_handle = None; // Old texture is dropped
                surface_data.current_buffer_info = None;
            } else if new_buffer_wl.is_some() && new_buffer_id == previous_buffer_id {
                // Buffer is the same, but other attributes like scale or transform might have changed.
                // Update current_buffer_info if needed.
                // The texture itself doesn't need to be recreated if the buffer ID is the same.
                if let Some(info) = surface_data.current_buffer_info.as_mut() {
                    info.scale = current_surface_attributes.buffer_scale;
                    info.transform = current_surface_attributes.buffer_transform;
                }
            }


            surface_data.damage_buffer_coords.clear(); 
            surface_data.damage_buffer_coords.extend_from_slice(&current_surface_attributes.damage_buffer);
            tracing::trace!(
                surface_id = ?surface.id(),
                "Damage received (buffer_coords): {:?}",
                current_surface_attributes.damage_buffer
            );

            surface_data.opaque_region_surface_local = current_surface_attributes.opaque_region.clone();
            surface_data.input_region_surface_local = current_surface_attributes.input_region.clone();

            // TODO: Mark window for redraw (will involve DesktopState.space and window mapping)
        });
    }

    fn new_subsurface(&mut self, surface: &WlSurface, parent: &WlSurface) {
        // Ensure SurfaceData is initialized for the new subsurface
        // self.new_surface(surface); // This should be called by smithay internally when the wl_surface for subsurface is created.
                                 // If not, it means the surface object passed here is not yet fully init by our new_surface.
                                 // Smithay's design implies new_surface is called for *all* wl_surfaces.
        
        tracing::info!(surface_id = ?surface.id(), parent_id = ?parent.id(), "New WlSubsurface created");

        // Retrieve SurfaceData for both. It should exist due to prior new_surface calls.
        let surface_data_arc = surface.data_map()
            .get::<Arc<Mutex<crate::compositor::surface_management::SurfaceData>>>()
            .expect("SurfaceData not found for subsurface. new_surface should have run.")
            .clone();
        let parent_surface_data_arc = parent.data_map()
            .get::<Arc<Mutex<crate::compositor::surface_management::SurfaceData>>>()
            .expect("SurfaceData not found for parent surface.")
            .clone();

        surface_data_arc.lock().unwrap().parent = Some(parent.downgrade());
        parent_surface_data_arc.lock().unwrap().children.push(surface.downgrade());
        
        // Smithay handles assigning the SubsurfaceRole. We don't need to do it manually here
        // unless we have custom subsurface role data to attach.
    }

    fn destroyed(&mut self, _surface: &WlSurface) {
        // Most cleanup is handled by the destruction hook in `new_surface`.
        // This handler can be used for any `DesktopState`-level cleanup related to the surface
        // that isn't managed by `SurfaceData`'s `Drop` or its destruction hook.
        tracing::trace!("CompositorHandler::destroyed called for surface {:?}", _surface.id());
        // Example: if this surface was a cursor, update cursor state in DesktopState.
        // Or if it was a drag-and-drop icon, etc.
    }
}

// ClientData definition (moved from previous attempt to be self-contained in this file for now)
// This should ideally be in its own module or alongside DesktopState if it's core.
#[derive(Debug)]
pub struct ClientCompositorData {
    // Smithay 0.10's CompositorClientState is usually managed internally by CompositorState.
    // We might not need to store it explicitly in ClientCompositorData unless we're customizing behavior
    // that CompositorState itself doesn't handle per client.
    // For now, let's assume it's a placeholder for any *additional* client-specific data
    // related to the compositor functionality, beyond what Smithay's CompositorClientState provides.
    // If CompositorHandler::client_compositor_state needs to return a ref to smithay's internal one,
    // DesktopState::compositor_state will be the source, and this struct might be simpler.
    _placeholder: (), // Replace with actual client data if needed.
    // For Smithay 0.10, `CompositorClientState` is often not directly stored by the user's `ClientData`.
    // The `CompositorHandler::client_compositor_state` method is usually implemented by
    // delegating to `CompositorState::client_state_for(client_id)`.
    // However, the plan implies `ClientCompositorData` holds a `CompositorClientState`.
    // Let's reconcile this: Smithay 0.10.0 `CompositorState` does not have `client_state_for`.
    // `CompositorClientState` is indeed user-managed per client.
    pub client_specific_state: CompositorClientState,
}

impl ClientCompositorData {
    pub fn new() -> Self {
        Self {
            _placeholder: (),
            client_specific_state: CompositorClientState::default(),
        }
    }
    // Added as per plan's expectation for client_compositor_state
    pub fn compositor_state(&self) -> &CompositorClientState {
        &self.client_specific_state
    }
}

impl Default for ClientCompositorData {
    fn default() -> Self {
        Self::new()
    }
}


// Delegate Compositor
delegate_compositor!(DesktopState);

// Delegate SHM
// This macro needs to be present for ShmHandler and BufferHandler to be correctly wired up
// with the ShmState and CompositorState when dispatching client requests.
delegate_shm!(DesktopState);

impl ShmHandler for DesktopState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
    // Optional: send_formats - Smithay 0.10 ShmState handles this automatically if formats are provided at creation.
    // fn send_formats(&self, shm: &wl_shm::WlShm) {
    //     // If custom logic is needed to send formats, implement here.
    //     // Otherwise, ShmState::new constructor and its internal bind logic handle it.
    // }
}

impl BufferHandler for DesktopState {
    fn buffer_destroyed(&mut self, buffer: &WlBuffer) {
        tracing::debug!(buffer_id = ?buffer.id(), "WlBuffer (potentially SHM or other type) destroyed notification received in BufferHandler.");
        // This is a generic notification that a WlBuffer has been destroyed.
        // It could be an SHM buffer, a DMA-BUF, or any other type that results in a WlBuffer.
        
        // The detailed plan suggests iterating through all windows/surfaces.
        // This requires a central tracking mechanism for surfaces/windows,
        // which is not yet implemented (e.g., `DesktopState.space`).
        //
        // For now, this log confirms the handler is called.
        // Actual cleanup of renderer resources associated with this buffer
        // would typically involve:
        // 1. Identifying which SurfaceData instances were using this buffer.
        // 2. Clearing their `current_buffer_info` and `texture_handle`.
        // 3. Notifying the renderer to release any GPU resources tied to this buffer/texture.
        // 4. Damaging the affected windows/surfaces.
        //
        // Example conceptual logic (will not compile without `self.space`):
        /*
        let mut affected_windows = Vec::new();
        if let Some(space) = self.space.as_mut() { // Assuming space is Option<Space<YourWindowType>>
            for window_element in space.elements_mut() {
                // Assuming YourWindowType has a way to get its WlSurface
                // and then its SurfaceData. This is highly dependent on future window management code.
                // For example, if window_element.user_data() holds Arc<Mutex<SurfaceData>>:
                if let Some(surface_data_arc) = window_element.user_data().get::<Arc<Mutex<SurfaceData>>>() {
                    let mut surface_data = surface_data_arc.lock().unwrap();
                    if surface_data.current_buffer_info.as_ref().map_or(false, |info| &info.buffer == buffer) {
                        tracing::info!(
                            "Buffer {:?} for surface with internal ID {:?} (associated with window {:?}) destroyed. \
                             Clearing buffer info and texture handle from SurfaceData.",
                            buffer.id(),
                            surface_data.id,
                            // window_element.id() // Assuming YourWindowType has an id()
                        );
                        surface_data.current_buffer_info = None;
                        surface_data.texture_handle = None; // This signals the renderer to drop its texture.
                        
                        // Mark the window for redraw.
                        // affected_windows.push(window_element.clone()); // Or some ID
                    }
                }
            }
            // After iterating, damage all affected windows.
            // for window_id_or_ref in affected_windows {
            //     space.damage_window(&window_id_or_ref, None, None);
            // }
        }
        */
        // Since we don't have self.space or a similar list yet, this is a no-op beyond logging.
        // Individual surfaces will clear their buffer on the next commit if it's invalid,
        // and the renderer will need to handle cases where a texture's underlying buffer is gone.
    }
}


/*
GlobalDispatch implementations for WlCompositor and WlSubcompositor are in globals.rs
GlobalDispatch for WlShm will be added to globals.rs next.
*/
