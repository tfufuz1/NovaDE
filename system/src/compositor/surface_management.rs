use smithay::{
    backend::renderer::{
        element::surface::WaylandSurfaceRenderElement,
        gles::GlesTexture, // Assuming GLES renderer
        // Import other necessary renderer types
    },
    desktop::Window, // Or appropriate type for 'parent' and 'children' if different
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            protocol::{wl_buffer::WlBuffer, wl_surface::WlSurface},
            Client, // For client_id
        },
    },
    utils::{BufferCoords, Physical, Rectangle, Serial, Transform, Uuid}, // For Uuid, Rectangle, Serial, Transform
    wayland::{
        compositor::{self, SurfaceAttributes, SurfaceData as SmithaySurfaceData, CompositorHandler},
        seat::WaylandFocus, // For user_data_ext if it relates to focus
        viewporter::SurfaceViewporterState, // For surface_viewporter_state
        presentation::PresentationState, // For surface_presentation_state
    },
};
use std::sync::{Arc, Mutex};
use std::any::Any; // For user_data_ext

// As specified in "C1 System Implementierungsplan.md"
#[derive(Debug, Clone)]
pub struct AttachedBufferInfo {
    pub buffer: WlBuffer,
    pub damage: Vec<Rectangle<i32, BufferCoords>>, // Damage in buffer coordinates
    pub transform: Transform,
    pub scale: i32,
    // pub frame_callback: Option<FrameCallback>, // Frame callbacks are typically managed by Smithay's CompositorState
}

#[derive(Debug)]
pub struct SurfaceData {
    pub id: Uuid, // Unique identifier for the surface
    pub client_id: Option<Client>, // Identifier for the client owning the surface
    pub role: Option<String>, // E.g., "xdg_toplevel", "xdg_popup", "cursor"
    pub current_buffer_info: Option<AttachedBufferInfo>,
    pub pending_buffer_info: Option<AttachedBufferInfo>,
    pub last_commit_serial: Serial,
    pub damage_regions_buffer_coords: Vec<Rectangle<i32, BufferCoords>>, // From SurfaceDataExt
    pub opaque_region: Option<Vec<Rectangle<i32, BufferCoords>>>, // Opaque region in surface coordinates
    pub input_region: Option<Vec<Rectangle<i32, BufferCoords>>>,  // Input region in surface coordinates
    pub user_data_ext: Option<Arc<Mutex<dyn Any + Send + Sync>>>, // For extensions like window state
    pub parent: Option<WlSurface>, // Parent surface, if any
    pub children: Vec<WlSurface>, // Child surfaces
    // Hooks - these would be function pointers or closures, define their signatures as needed
    // pub commit_hook: Option<Box<dyn Fn(&mut Self, &WlSurface)>>,
    // pub pre_commit_hook: Option<Box<dyn Fn(&mut Self, &WlSurface)>>,
    // pub post_commit_hook: Option<Box<dyn Fn(&mut Self, &WlSurface)>>,
    pub surface_viewporter_state: SurfaceViewporterState,
    pub surface_presentation_state: PresentationState, // from Smithay
    pub surface_scale_factor: i32, // Scale factor applied to the surface

    // Fields from Novade's SurfaceDataExt
    pub texture: Option<GlesTexture>, // Assuming GLES; make generic if needed
    pub damage_buffer: Option<Rectangle<i32, Physical>>, // Damage in physical coordinates
    pub render_element: Option<WaylandSurfaceRenderElement<GlesTexture>>, // Example with GlesTexture
    pub window_map_state: Option<WindowMapState>, // If you keep this concept

    // Smithay's internal user_data, which we are replacing the direct use of
    // but need to ensure its functionality is covered.
    // Smithay uses its SurfaceData for things like roles, parent, children, etc.
    // We need to ensure our SurfaceData correctly integrates or replaces this.
    _smithay_surface_data: SmithaySurfaceData, // To hold Smithay's internal data initially
}

// Temporary struct, replace with actual definition if needed
#[derive(Debug, Clone)]
pub struct WindowMapState {
    pub a: i32, // Placeholder
}


impl SurfaceData {
    pub fn new(surface: &WlSurface) -> Arc<Mutex<Self>> {
        let id = Uuid::new_v4();
        let client_id = surface.client();
        let smithay_data = compositor::SurfaceData::new(); // Smithay's default

        let data = Arc::new(Mutex::new(Self {
            id,
            client_id,
            role: None,
            current_buffer_info: None,
            pending_buffer_info: None,
            last_commit_serial: Serial::INITIAL,
            damage_regions_buffer_coords: Vec::new(),
            opaque_region: None,
            input_region: None,
            user_data_ext: None,
            parent: smithay_data.parent().cloned(), // Initialize from Smithay's data
            children: Vec::new(), // Smithay manages children internally via its data
            surface_viewporter_state: SurfaceViewporterState::default(),
            surface_presentation_state: PresentationState::default(),
            surface_scale_factor: 1,
            texture: None,
            damage_buffer: None,
            render_element: None,
            window_map_state: None,
            _smithay_surface_data: smithay_data,
        }));

        // Store our custom SurfaceData in Smithay's user_data map for now.
        // This is a common pattern to extend Smithay objects.
        surface.data::<Arc<Mutex<Self>>>().set(data.clone());
        data
    }
}

/// Retrieves a clone of the Arc<Mutex<SurfaceData>> for a given WlSurface.
/// Initializes it if it doesn't exist.
pub fn get_surface_data(surface: &WlSurface) -> Arc<Mutex<SurfaceData>> {
    let data_map = surface.data::<Arc<Mutex<SurfaceData>>>();
    data_map.get().cloned().unwrap_or_else(|| SurfaceData::new(surface))
}

/// Provides mutable access to the SurfaceData associated with a WlSurface.
/// Initializes it if it doesn't exist.
pub fn with_surface_data_mut<F, R>(surface: &WlSurface, f: F) -> R
where
    F: FnOnce(&mut SurfaceData) -> R,
{
    let data_arc = get_surface_data(surface);
    let mut guard = data_arc.lock().unwrap();
    f(&mut guard)
}

// Example of how you might integrate with Smithay's CompositorHandler
// This part would typically be in your main compositor state or similar.
// This is just for conceptual illustration here.
/*
struct YourCompositorState {
    // ... other fields
}

impl CompositorHandler for YourCompositorState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        // return &mut self.compositor_state;
        unimplemented!()
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        // return &self.client_compositor_state_map.get(client).unwrap();
        unimplemented!()
    }

    fn commit(&mut self, surface: &WlSurface) {
        // Default commit processing by Smithay
        smithay::wayland::compositor::handlers::commit_handler::<Self>(surface);

        // Custom logic after Smithay's processing
        with_surface_data_mut(surface, |surface_data| {
            // Update your custom SurfaceData based on the commit
            // For example, move pending_buffer_info to current_buffer_info
            if surface_data.pending_buffer_info.is_some() {
                surface_data.current_buffer_info = surface_data.pending_buffer_info.take();
                // Accumulate damage, etc.
            }
            // Update last_commit_serial, etc.
            let smithay_attrs = compositor::surface_attributes(surface);
            surface_data.last_commit_serial = smithay_attrs.last_commit_serial;


            // Call post_commit_hook if defined
            // if let Some(hook) = &surface_data.post_commit_hook {
            //     hook(surface_data, surface);
            // }
        });

        // Example: Manage XDG surface roles
        if let Some(xdg_surface) = get_xdg_surface(surface) {
            // process xdg_surface commit
        }
    }

    // Implement other required methods...
    fn new_surface(&mut self, surface: &WlSurface) {
        // Initialize your SurfaceData when a new surface is created
        SurfaceData::new(surface);
    }

    fn new_subsurface(&mut self, surface: &WlSurface, parent: &WlSurface) {
        SurfaceData::new(surface); // Initialize for the new subsurface
        with_surface_data_mut(surface, |data| {
            data.parent = Some(parent.clone());
            // Potentially update role if it's a subsurface
        });
        with_surface_data_mut(parent, |parent_data| {
            parent_data.children.push(surface.clone());
        });
    }
}

// Helper to get XDG surface, assuming you have XdgShellState
use smithay::wayland::shell::xdg::{XdgShellHandler, XdgSurfaceUserData};
fn get_xdg_surface(surface: &WlSurface) -> Option<xdg_toplevel::XdgToplevel> {
    surface.data::<XdgSurfaceUserData>()?.xdg_surface.clone().into()
    // This needs proper role checking, toplevel vs popup etc.
}
*/
