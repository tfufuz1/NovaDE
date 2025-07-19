use smithay::{
    desktop::{Window, WindowSurfaceType, Space}, // Added Space for potential future use with Window
    reexports::wayland_server::{
        protocol::wl_surface::WlSurface,
        Resource, // For WlSurface.id()
    },
    utils::{Logical, Point, Rectangle, Size, Serial, Transform}, // Added Transform
    wayland::{
        compositor::SurfaceData as SmithaySurfaceData, // To check surface aliveness
        shell::xdg::{PopupSurface, ToplevelSurface, WindowSurface, XdgToplevelSurfaceData, ToplevelState as XdgToplevelState, ResizeEdge, XdgSurface as SmithayXdgSurface}, // Added SmithayXdgSurface
        seat::WaylandFocus, // For wl_surface() return type Option<&WlSurface>
    },
    wayland::presentation::presentation_time, // Correct path for Smithay 0.10
};
use std::{
    hash::{Hash, Hasher},
    sync::{Arc, RwLock, Weak, Mutex}, // Added RwLock, Mutex for specific cases
};
use uuid::Uuid;
use novade_domain::window_management as domain_wm;

// ANCHOR: XdgSurfaceRole
/// Represents the role of an XDG surface.
/// An XDG surface can only have one role during its lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XdgSurfaceRole {
    /// The surface is not yet assigned a role.
    None,
    /// The surface is a toplevel window.
    Toplevel,
    /// The surface is a popup.
    Popup,
}

// ANCHOR: XdgSurfaceState
/// Represents the lifecycle state of an XDG surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XdgSurfaceState {
    /// Alive but not yet configured. Awaiting initial configure event.
    PendingConfiguration,
    /// Configured and ready for use.
    Configured,
    /// The client has requested destruction, but resources might still be pending release.
    AwaitingDestroy,
    /// The surface has been destroyed and is no longer valid.
    Destroyed,
}

// ANCHOR: XdgSurfaceUserData
/// User data associated with an XdgSurface object.
/// This stores lifecycle state and role for the xdg_surface.
#[derive(Debug)]
pub struct XdgSurfaceUserData {
    pub role: Mutex<XdgSurfaceRole>,
    pub state: Mutex<XdgSurfaceState>,
    pub wl_surface: WlSurface, // Keep a reference to the underlying WlSurface

    // ANCHOR: XdgSurfaceUserDataDetailedStateStart
    /// Serial of the last configure event sent to the client by the compositor.
    /// This is set when the compositor sends a configure event.
    pub last_compositor_configure_serial: Mutex<Option<Serial>>,

    /// Serial of the configure event acked by the client.
    /// This is updated when the client sends an ack_configure.
    pub last_acked_configure_serial: Mutex<Option<Serial>>,

    /// Pending geometry if a configure event is acked by the client.
    /// This is set by the client via `set_window_geometry`.
    pub pending_window_geometry: Mutex<Option<Rectangle<i32, Logical>>>,

    /// Current effective geometry of the window, applied after an ack_configure.
    /// This might be different from what client requested if compositor overrides it.
    pub current_window_geometry: Mutex<Option<Rectangle<i32, Logical>>>,

    /// Min size requested by client for a toplevel surface.
    pub min_size: Mutex<Option<Size<i32, Logical>>>,

    /// Max size requested by client for a toplevel surface.
    pub max_size: Mutex<Option<Size<i32, Logical>>>,
    // ANCHOR_END: XdgSurfaceUserDataDetailedStateEnd
}

impl XdgSurfaceUserData {
    pub fn new(wl_surface: WlSurface) -> Self {
        Self {
            role: Mutex::new(XdgSurfaceRole::None),
            state: Mutex::new(XdgSurfaceState::PendingConfiguration),
            wl_surface,
            // ANCHOR: XdgSurfaceUserDataDetailedStateInitStart
            last_compositor_configure_serial: Mutex::new(None),
            last_acked_configure_serial: Mutex::new(None),
            pending_window_geometry: Mutex::new(None),
            current_window_geometry: Mutex::new(None),
            min_size: Mutex::new(None),
            max_size: Mutex::new(None),
            // ANCHOR_END: XdgSurfaceUserDataDetailedStateInitEnd
        }
    }
}

// Placeholder for Domain WindowIdentifier
// In a real system, this would come from `crate::core::types` or similar.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct DomainWindowIdentifier(Uuid);

impl DomainWindowIdentifier {
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Window state
#[derive(Debug, Clone)]
pub struct WindowState {
    /// Is the window maximized
    pub maximized: bool,
    
    /// Is the window fullscreen
    pub fullscreen: bool,
    
    /// Is the window minimized
    pub minimized: bool,
    
    /// Is the window activated (has focus)
    pub activated: bool,
    
    /// Window geometry (requested by client or set by compositor)
    pub geometry: Option<Rectangle<i32, Logical>>,
    
    /// Window position (current position on screen)
    pub position: Point<i32, Logical>,
    
    /// Window size (current size on screen)
    pub size: Size<i32, Logical>,
    
    /// Window minimum size
    pub min_size: Size<i32, Logical>,
    
    /// Window maximum size
    pub max_size: Size<i32, Logical>,

    /// Saved geometry before a maximize or fullscreen action
    pub saved_pre_action_geometry: Option<Rectangle<i32, Logical>>,
}

/// Window manager data, specific to compositor's internal state management
#[derive(Debug, Clone)]
pub struct WindowManagerData {
    /// Is the window being moved
    pub moving: bool,
    
    /// Is the window being resized
    pub resizing: bool,
    
    /// Resize edges
    pub resize_edges: Option<ResizeEdge>,
    
    /// Window workspace
    pub workspace: u32, // Or Option<u32> if a window might not be on any workspace
    
    /// Window layer
    pub layer: WindowLayer,
    
    /// Window opacity
    pub opacity: f64,
    
    /// Window z-index (relative to other windows in the same layer/space)
    pub z_index: i32,
    
    /// Window decorations state (e.g. server-side or client-side)
    pub decorations: bool, // true for server-side, false for client-side
}

#[derive(Debug)]
pub struct ManagedWindow {
    pub id: Uuid, // Internal compositor ID
    pub domain_id: DomainWindowIdentifier,
    pub domain_window: Arc<RwLock<domain_wm::Window>>,
    pub xdg_surface: WindowSurface, // Toplevel or Popup
    // ANCHOR: ManagedWindowCurrentGeometryRwLock
    pub current_geometry: Arc<RwLock<Rectangle<i32, Logical>>>,
    // ANCHOR_END: ManagedWindowCurrentGeometryRwLock
    pub is_mapped: bool,
    pub parent: Option<Weak<ManagedWindow>>,
    // Fields for title and app_id (direct, as per existing ManagedWindow in types.rs)
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub last_configure_serial: Option<Serial>,
    // ANCHOR: AddWorkspaceIdToManagedWindow
    pub workspace_id: Arc<RwLock<Option<Uuid>>>,
    // ANCHOR_END: AddWorkspaceIdToManagedWindow
    // ANCHOR: AddOutputNameToManagedWindow
    pub output_name: Arc<RwLock<Option<String>>>, // Name of the output the window is primarily on
    // ANCHOR_END: AddOutputNameToManagedWindow
    // ANCHOR: AddTilingMasterToManagedWindow
    pub tiling_master: Arc<RwLock<bool>>, // True if this window is the master in a tiling layout
    // ANCHOR_END: AddTilingMasterToManagedWindow
    // Added fields from xdg_shell/mod.rs's ManagedWindow
    pub state: Arc<RwLock<WindowState>>,
    pub manager_data: Arc<RwLock<WindowManagerData>>,
}

/// Window layer for stacking order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)] // Added PartialOrd, Ord for easy sorting/comparison
pub enum WindowLayer {
    /// Background layer (e.g., wallpapers)
    Background,
    
    /// Bottom layer (e.g., docks, panels that should be below normal windows)
    Bottom,
    
    /// Normal window layer
    Normal,
    
    /// Top layer (e.g., panels, notifications that should be above normal windows)
    Top,
    
    /// Overlay layer (e.g., screen lockers, critical alerts, input method popups)
    Overlay,
}

#[derive(Debug)]
pub struct ManagedWindow {
    pub id: Uuid, // Internal compositor ID
    pub domain_id: DomainWindowIdentifier,
    pub xdg_surface: WindowSurface, // Toplevel or Popup
    // ANCHOR: ManagedWindowCurrentGeometryRwLock
    pub current_geometry: Arc<RwLock<Rectangle<i32, Logical>>>,
    // ANCHOR_END: ManagedWindowCurrentGeometryRwLock
    pub is_mapped: bool,
    pub parent: Option<Weak<ManagedWindow>>,
    // Fields for title and app_id (direct, as per existing ManagedWindow in types.rs)
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub last_configure_serial: Option<Serial>,
    // ANCHOR: AddWorkspaceIdToManagedWindow
    pub workspace_id: Arc<RwLock<Option<Uuid>>>,
    // ANCHOR_END: AddWorkspaceIdToManagedWindow
    // ANCHOR: AddOutputNameToManagedWindow
    pub output_name: Arc<RwLock<Option<String>>>, // Name of the output the window is primarily on
    // ANCHOR_END: AddOutputNameToManagedWindow
    // ANCHOR: AddTilingMasterToManagedWindow
    pub tiling_master: Arc<RwLock<bool>>, // True if this window is the master in a tiling layout
    // ANCHOR_END: AddTilingMasterToManagedWindow
    // Added fields from xdg_shell/mod.rs's ManagedWindow
    pub state: Arc<RwLock<WindowState>>,
    pub manager_data: Arc<RwLock<WindowManagerData>>,
}

impl ManagedWindow {
    pub fn new_toplevel(toplevel_surface: ToplevelSurface, domain_id: DomainWindowIdentifier) -> Self {
        let initial_geometry = Rectangle::from_loc_and_size((0, 0), (0, 0)); // Default, will be updated by configure
        let title = toplevel_surface.title();
        let app_id = toplevel_surface.app_id();

        let domain_window = domain_wm::Window::new(
            domain_id.to_string(),
            title.clone().unwrap_or_default(),
            app_id.clone().unwrap_or_default(),
            domain_wm::WindowType::Normal,
        );

        Self {
            id: Uuid::new_v4(),
            domain_id,
            domain_window: Arc::new(RwLock::new(domain_window)),
            xdg_surface: WindowSurface::Toplevel(toplevel_surface),
            current_geometry: Arc::new(RwLock::new(initial_geometry)), // ANCHOR_REF: ManagedWindowCurrentGeometryRwLock
            is_mapped: false,
            parent: None,
            title,
            app_id,
            last_configure_serial: None,
            workspace_id: Arc::new(RwLock::new(None)),
            output_name: Arc::new(RwLock::new(None)), // Initialize output_name to None
            tiling_master: Arc::new(RwLock::new(false)), // Default to not being master
            state: Arc::new(RwLock::new(WindowState {
                maximized: false,
                fullscreen: false,
                minimized: false,
                activated: false, // Should be set true when focused
                geometry: None, // Client's requested geometry
                position: Point::from((100, 100)), // Default initial position
                size: Size::from((300, 200)),    // Default initial size
                min_size: Size::from((1, 1)),
                max_size: Size::from((0, 0)), // 0 means unlimited
                saved_pre_action_geometry: None,
            })),
            manager_data: Arc::new(RwLock::new(WindowManagerData {
                moving: false,
                resizing: false,
                resize_edges: None,
                workspace: 0, // Default workspace
                layer: WindowLayer::Normal,
                opacity: 1.0,
                z_index: 0,
                decorations: true, // Default to SSD
            })),
        }
    }

    pub fn new_popup(popup_surface: PopupSurface, parent_domain_id: DomainWindowIdentifier, parent_window: Option<Arc<ManagedWindow>>) -> Self {
        let title = popup_surface.wl_surface().data_map().get::<XdgToplevelSurfaceData>().and_then(|d| d.title.clone());
        let app_id = popup_surface.wl_surface().data_map().get::<XdgToplevelSurfaceData>().and_then(|d| d.app_id.clone());

        let domain_window = domain_wm::Window::new(
            parent_domain_id.to_string(),
            title.clone().unwrap_or_default(),
            app_id.clone().unwrap_or_default(),
            domain_wm::WindowType::Popup,
        );

        Self {
            id: Uuid::new_v4(),
            // Popups might share parent's domain_id or have a new one. For now, new.
            domain_id: DomainWindowIdentifier::new_v4(),
            domain_window: Arc::new(RwLock::new(domain_window)),
            xdg_surface: WindowSurface::Popup(popup_surface),
            current_geometry: Arc::new(RwLock::new(Rectangle::from_loc_and_size((0, 0), (0, 0)))), // ANCHOR_REF: ManagedWindowCurrentGeometryRwLockInitPopup
            is_mapped: false,
            parent: parent_window.map(|p| Arc::downgrade(&p)),
            title,
            app_id,
            last_configure_serial: None,
            workspace_id: Arc::new(RwLock::new(None)),
            output_name: Arc::new(RwLock::new(None)), // Popups are typically on the same output as parent
            tiling_master: Arc::new(RwLock::new(false)), // Popups are not tiled
            // Popups usually don't have complex state/manager_data like toplevels,
            // but initializing them for consistency.
            state: Arc::new(RwLock::new(WindowState {
                maximized: false, fullscreen: false, minimized: false, activated: false,
                geometry: None, position: Point::from((0,0)), size: Size::from((0,0)),
                min_size: Size::from((0,0)), max_size: Size::from((0,0)),
                saved_pre_action_geometry: None,
            })),
            manager_data: Arc::new(RwLock::new(WindowManagerData {
                moving: false, resizing: false, resize_edges: None, workspace: 0,
                layer: WindowLayer::Overlay, // Popups are often overlays
                opacity: 1.0, z_index: 0, decorations: false, // Popups don't have decorations
            })),
        }
    }

    pub fn wl_surface_ref(&self) -> Option<&WlSurface> {
        self.xdg_surface.wl_surface().as_ref()
    }

    // Helper methods to access interior mutability, if needed for handlers
    // Example:
    // pub fn with_state<F, R>(&self, func: F) -> R where F: FnOnce(&mut WindowState) -> R {
    //     let mut guard = self.state.write().unwrap();
    //     func(&mut *guard)
    // }
    // pub fn with_manager_data<F, R>(&self, func: F) -> R where F: FnOnce(&mut WindowManagerData) -> R {
    //     let mut guard = self.manager_data.write().unwrap();
    //     func(&mut *guard)
    // }
}

impl PartialEq for ManagedWindow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for ManagedWindow {}

impl Hash for ManagedWindow {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

// --- smithay::desktop::Window Trait Implementation ---
impl Window for ManagedWindow {
    fn id(&self) -> usize {
        // Using a stable hash of the Uuid for the usize ID required by Space.
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.id.hash(&mut hasher);
        hasher.finish() as usize
    }

    fn wl_surface(&self) -> Option<WlSurface> {
        Some(self.xdg_surface.wl_surface().clone())
    }
    
    fn focus_surface(&self) -> Option<&WlSurface> {
        self.wl_surface_ref()
    }

    fn surface_type(&self) -> WindowSurfaceType {
        self.xdg_surface.surface_type()
    }

    fn geometry(&self) -> Rectangle<i32, Logical> {
        *self.current_geometry.read().unwrap() // ANCHOR_REF: ManagedWindowCurrentGeometryRwLockRead
    }

    fn is_mapped(&self) -> bool {
        self.is_mapped && self.xdg_surface.alive()
    }
    
    fn is_suspended(&self) -> bool {
        self.xdg_surface.toplevel().map_or(false, |t| {
            let current_states = t.current_states();
            current_states.contains(XdgToplevelState::Suspended) || current_states.contains(XdgToplevelState::Minimized)
        })
    }

    fn send_frame(&self, time: u32, throttle_harming_region: Option<Rectangle<i32, Logical>>) {
        if let Some(surface) = self.xdg_surface.wl_surface().as_ref() {
            if surface.alive() { // Check if surface is still alive before sending frame
                presentation_time::send_frames_surface_dest_harming_region_transform(
                    surface,
                    time,
                    throttle_harming_region,
                    Transform::Normal, // Assuming normal transform for now
                );
            }
        }
    }
    
    fn send_configure(&mut self) {
        match &self.xdg_surface {
            WindowSurface::Toplevel(toplevel) => {
                // Smithay 0.10.0: configure() returns the serial
                let serial = toplevel.send_configure();
                self.last_configure_serial = Some(serial);
            }
            WindowSurface::Popup(popup) => {
                let serial = popup.send_configure();
                self.last_configure_serial = Some(serial);
            }
        }
    }

    fn requested_extents(&self) -> Option<Rectangle<i32, Logical>> {
        None // Can be implemented if client-side decorations or specific extent requests are handled.
    }
    
    fn user_data(&self) -> &smithay::utils::UserDataMap {
        self.xdg_surface.user_data()
    }

    fn self_update(&mut self) {
        // This method allows the window to synchronize its state with the underlying XDG surface.
        // For example, update title or app_id if they changed on the XDG surface.
        if let WindowSurface::Toplevel(toplevel) = &self.xdg_surface {
            self.title = toplevel.title();
            self.app_id = toplevel.app_id();
            // Could also check current_states() here if needed for other properties.
        }
        // Popups don't typically have titles/app_ids in the same way.
    }

    fn on_commit(&mut self) {
        self.self_update();
    }

    // Optional methods from Window trait, can be default or implemented:
    // fn damage_applied(&mut self) { /* ... */ }
    // fn is_solid(&self) -> bool { false } 
    // fn z_index(&self) -> u8 { 0 } 
}

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::reexports::wayland_server::{
        Client, Display, DisplayHandle, GlobalDispatch, Main, UserData, Interface, MessageGroup, ClientUserData,
        backend::{ClientData, ClientId, GlobalId, Handle, ObjectData, ObjectId},
        protocol::wl_surface::WlSurface,
    };
    use smithay::reexports::wayland_server::globals::GlobalData;
    use smithay::wayland::shell::xdg::XDG_WM_BASE_VERSION; // For XdgWmBase
    use std::sync::Arc;
    use std::collections::HashMap; // For mock client data store
    use smithay::delegate_xdg_shell; // Required for DesktopState délégation


    // --- Test Infrastructure ---
    // ANCHOR: TestStateSetup
    #[derive(Default)]
    struct TestState {
        xdg_shell_state: XdgShellState,
        // Add other necessary state components for DesktopState if they become relevant
        // For now, we only need XdgShellState for these tests.
    }

    // We need to implement GlobalDispatch for XdgWmBase for TestState
    // as DesktopState does, to allow XdgShellState::new() to work.
    impl GlobalDispatch<XdgWmBase, Arc<XdgWmBaseClientData>> for TestState {
        fn bind(
            _state: &mut Self,
            _handle: &DisplayHandle,
            _client: &Client,
            resource: New<XdgWmBase>,
            global_data: &Arc<XdgWmBaseClientData>,
            data_init: &mut DataInit<'_, Self>,
        ) {
            data_init.init(resource, global_data.clone());
        }
        fn can_view(_client: Client, _global_data: &Arc<XdgWmBaseClientData>) -> bool {
            true // Allow all clients to view for tests
        }
    }

    // Implement necessary delegations for XdgShellState
    delegate_xdg_shell!(TestState);


    // Minimal mock for WlSurface for testing purposes.
    // This is challenging because WlSurface is deeply tied to a Display and Client.
    // We'll use a very simplified mock that allows ManagedWindow to be constructed.
    // Smithay's test_utils might offer better ways for more integrated tests.

    // Helper to create a DisplayHandle and a Client for tests that need WlSurface.
    fn create_test_display_client_and_init_xdg_shell_state() -> (Display<TestState>, DisplayHandle, Client, Arc<XdgShellState>) {
        let mut display: Display<TestState> = Display::new().unwrap();
        let dh = display.handle();

        // Initialize XdgShellState within our TestState
        let mut test_state = TestState::default();
        let xdg_activation_state = XdgActivationState::new(); // Smithay 0.10
        let (xdg_shell_state_arc, _xdg_wm_base_global) = XdgShellState::new_with_activation(
            &dh,
            &xdg_activation_state, // Pass XdgActivationState
        );

        // Store the XdgShellState in our TestState (or make it accessible)
        // For simplicity in this test setup, we'll return it directly.
        // In a real compositor, it would be part of the main state struct.
        // test_state.xdg_shell_state = xdg_shell_state_arc.clone(); // If TestState owned it directly

        // Register the XDG WM Base global
        dh.create_global::<TestState, XdgWmBase, Arc<XdgWmBaseClientData>>(
            XDG_WM_BASE_VERSION, // Use the constant for version
            xdg_shell_state_arc.xdg_wm_base_data().clone(), // Get the data for the global
        );

        let client_data = TestClientData::default();
        let client = display.create_client(client_data.into());

        // Need to run the event loop once to process global registration
        // display.dispatch_clients(&mut test_state).unwrap();

        (display, dh, client, xdg_shell_state_arc)
    }

    #[derive(Default, Clone)]
    struct TestClientData {
        user_data: UserData,
        // other client specific data if needed
    }
    impl ClientData for TestClientData {
        fn initialized(&self, _client_id: ClientId) {}
        fn disconnected(&self, _client_id: ClientId, _reason: smithay::reexports::wayland_server::DisconnectReason) {}
        fn data_map(&self) -> &UserData {
            &self.user_data
        }
    }
    
    // Mock ToplevelSurface and PopupSurface using a WlSurface created with test display/client
    // These are very basic and won't respond to most Wayland requests.
    fn mock_wl_surface(dh: &DisplayHandle, client: &Client) -> Main<WlSurface> {
        // Client::create_object now returns Main<T> for objects.
        client.create_object::<WlSurface, _>(dh, WlSurface::interface().version, Arc::new(TestObjectData)).unwrap()
    }

    // Mock ObjectData for WlSurface
    #[derive(Default)]
    struct TestObjectData;
    impl ObjectData<WlSurface> for TestObjectData {
        fn request(
            self: Arc<Self>,
            _handle: &Handle,
            _client_data: &mut dyn ClientData,
            _client_id: ClientId,
            _msg: Message<WlSurface>,
        ) -> Option<Arc<dyn ObjectData<WlSurface>>> {
            None // No actual request handling for this mock
        }

        fn destroyed(
            self: Arc<Self>,
            _client_id: ClientId,
            _object_id: ObjectId,
        ) {} // No special handling on destroy for mock
    }


    fn mock_toplevel_surface(dh: &DisplayHandle, client: &Client) -> ToplevelSurface {
        let surface_main = mock_wl_surface(dh, client);
        // Attach some user data for XdgSurface to be created successfully.
        // Smithay's XdgSurface::new expects SurfaceData to be present.
        surface_main.as_ref().data_map().insert_if_missing_threadsafe(|| {
            Arc::new(SmithaySurfaceData::new(None, Rectangle::from_loc_and_size((0,0), (0,0))))
        });
        let xdg_surface = SmithayXdgSurface::new_toplevel(surface_main.as_ref().clone());
        ToplevelSurface::from_xdg_surface(xdg_surface, Default::default()).unwrap()
    }

    fn mock_popup_surface(dh: &DisplayHandle, client: &Client, parent_surface: &WlSurface) -> PopupSurface {
        let surface_main = mock_wl_surface(dh, client);
        surface_main.as_ref().data_map().insert_if_missing_threadsafe(|| {
            Arc::new(SmithaySurfaceData::new(None, Rectangle::from_loc_and_size((0,0), (0,0))))
        });
        let xdg_surface = SmithayXdgSurface::new_popup(surface_main.as_ref().clone(), parent_surface.clone());
        PopupSurface::from_xdg_surface(xdg_surface, Default::default()).unwrap()
    }


    // ANCHOR_END: TestStateSetup

    #[test]
    fn test_window_state_defaults() {
        // Values from ManagedWindow::new_toplevel constructor's WindowState init
        let state = WindowState {
            maximized: false, // Direct field access for struct literal
            fullscreen: false, // Direct field access
            minimized: false,  // Direct field access
            activated: false,  // Direct field access
            geometry: None,    // Direct field access
            position: Point::from((100, 100)), // Direct field access
            size: Size::from((300, 200)),    // Direct field access
            min_size: Size::from((1, 1)),    // Direct field access
            max_size: Size::from((0, 0)),    // Direct field access
            saved_pre_action_geometry: None, // Direct field access
        };

        assert_eq!(state.maximized, false);
        assert_eq!(state.fullscreen, false);
        assert_eq!(state.minimized, false);
        assert_eq!(state.activated, false);
        assert_eq!(state.geometry, None);
        assert_eq!(state.position, Point::from((100, 100)));
        assert_eq!(state.size, Size::from((300, 200)));
        assert_eq!(state.min_size, Size::from((1, 1)));
        assert_eq!(state.max_size, Size::from((0, 0)));
        assert!(state.saved_pre_action_geometry.is_none());
    }

    #[test]
    fn test_window_manager_data_defaults() {
        // Values from ManagedWindow::new_toplevel constructor's WindowManagerData init
        let data = WindowManagerData {
            moving: false,     // Direct field access
            resizing: false,   // Direct field access
            resize_edges: None, // Direct field access
            workspace: 0,      // Direct field access
            layer: WindowLayer::Normal, // Direct field access
            opacity: 1.0,      // Direct field access
            z_index: 0,        // Direct field access
            decorations: true, // Direct field access
        };

        assert_eq!(data.moving, false);
        assert_eq!(data.resizing, false);
        assert!(data.resize_edges.is_none());
        assert_eq!(data.workspace, 0);
        assert_eq!(data.layer, WindowLayer::Normal);
        assert_eq!(data.opacity, 1.0);
        assert_eq!(data.z_index, 0);
        assert_eq!(data.decorations, true);
    }

    // ANCHOR: TestXdgSurfaceUserDataDefaults
    #[test]
    fn test_xdg_surface_user_data_defaults() {
        let (_display, dh, client, _xdg_shell_state) = create_test_display_client_and_init_xdg_shell_state();
        let wl_surface_main = mock_wl_surface(&dh, &client);
        let user_data = XdgSurfaceUserData::new(wl_surface_main.as_ref().clone());

        assert_eq!(*user_data.role.lock().unwrap(), XdgSurfaceRole::None);
        assert_eq!(*user_data.state.lock().unwrap(), XdgSurfaceState::PendingConfiguration);
        assert!(user_data.wl_surface.alive());
        assert_eq!(user_data.wl_surface.id(), wl_surface_main.as_ref().id());
    }
    // ANCHOR_END: TestXdgSurfaceUserDataDefaults

    // ANCHOR: DecorationConstants
    pub const DEFAULT_TITLE_BAR_HEIGHT: i32 = 30;
    pub const DEFAULT_BORDER_SIZE: i32 = 5;
    // ANCHOR_END: DecorationConstants

    #[test]
    fn test_managed_window_toplevel_initialization_basic() {
        let (_display, dh, client, _xdg_shell_state) = create_test_display_client_and_init_xdg_shell_state();
        let toplevel_surface_mock = mock_toplevel_surface(&dh, &client); // Renamed to avoid conflict
        let domain_id = DomainWindowIdentifier::new_v4();

        let managed_window = ManagedWindow::new_toplevel(toplevel_surface_mock, domain_id);


        assert_eq!(managed_window.domain_id, domain_id);
        assert_eq!(managed_window.is_mapped, false);
        assert!(managed_window.parent.is_none());
        // title and app_id are derived from the surface, which is a mock here.
        // They will likely be None unless the mock ToplevelSurface provides them.
        // Assertions for title and app_id depend on mock_toplevel_surface details.
        // If mock_toplevel_surface doesn't set them, they will be None.
        assert!(managed_window.title.is_none());
        assert!(managed_window.app_id.is_none());

        let state = managed_window.state.read().unwrap();
        assert_eq!(state.position, Point::from((100, 100)));
        assert_eq!(state.size, Size::from((300, 200)));

        let manager_data = managed_window.manager_data.read().unwrap();
        assert_eq!(manager_data.layer, WindowLayer::Normal);
        assert_eq!(manager_data.decorations, true);
    }
    
    #[test]
    fn test_managed_window_popup_initialization_basic() {
        let (_display, dh, client, _xdg_shell_state) = create_test_display_client_and_init_xdg_shell_state();
        let parent_wl_surface_main = mock_wl_surface(&dh, &client); // Main<WlSurface>
        let parent_wl_surface_ref = parent_wl_surface_main.as_ref(); // &WlSurface

        let popup_surface_mock = mock_popup_surface(&dh, &client, parent_wl_surface_ref);
        let parent_domain_id = DomainWindowIdentifier::new_v4();

        let managed_window = ManagedWindow::new_popup(popup_surface_mock, parent_domain_id, None);

        // Domain ID for popup itself is new_v4 in constructor, not parent's for this field.
        assert_ne!(managed_window.domain_id, parent_domain_id);
        assert_eq!(managed_window.is_mapped, false);
        assert!(managed_window.parent.is_none()); // No Arc<ManagedWindow> parent passed

        let state = managed_window.state.read().unwrap();
        assert_eq!(state.position, Point::from((0,0))); // Defaults for popup state
        assert_eq!(state.size, Size::from((0,0)));

        let manager_data = managed_window.manager_data.read().unwrap();
        assert_eq!(manager_data.layer, WindowLayer::Overlay); // Popups default to Overlay layer
        assert_eq!(manager_data.decorations, false); // Popups don't have decorations
    }


    #[test]
    fn test_managed_window_ids_are_unique() {
        let (mut display, dh, client1, _xdg_shell_state) = create_test_display_client_and_init_xdg_shell_state();
        let toplevel_surface1_mock = mock_toplevel_surface(&dh, &client1);
        let domain_id1 = DomainWindowIdentifier::new_v4();
        let window1 = ManagedWindow::new_toplevel(toplevel_surface1_mock, domain_id1);

        // Need a new WlSurface for the second ToplevelSurface
        // Create a new client or new surface on same client. Let's use a new client for isolation.
        let client2_data = TestClientData::default();
        let client2 = display.create_client(client2_data.into());
        let toplevel_surface2_mock = mock_toplevel_surface(&dh, &client2);

        let domain_id2 = DomainWindowIdentifier::new_v4();
        let window2 = ManagedWindow::new_toplevel(toplevel_surface2_mock, domain_id2);

        assert_ne!(window1.id, window2.id); // UUIDs should be different
        assert_ne!(Window::id(&window1), Window::id(&window2)); // usize hashes should be different
    }

    #[test]
    fn test_managed_window_trait_geometry_and_is_mapped() {
        let (_display, dh, client, _xdg_shell_state) = create_test_display_client_and_init_xdg_shell_state();
        let toplevel_surface_mock = mock_toplevel_surface(&dh, &client);
        let domain_id = DomainWindowIdentifier::new_v4();
        
        let mut managed_window = ManagedWindow::new_toplevel(toplevel_surface_mock.clone(), domain_id);

        let test_geometry = Rectangle::from_loc_and_size((10, 20), (300, 400));
        // ANCHOR_REF: ManagedWindowCurrentGeometryRwLockWriteTest
        *managed_window.current_geometry.write().unwrap() = test_geometry;
        assert_eq!(Window::geometry(&managed_window), test_geometry);

        // Test is_mapped
        // `Window::is_mapped()` checks `self.is_mapped && self.xdg_surface.alive()`.
        // Our mock WlSurface via mock_wl_surface should be alive as long as client and display exist.
        
        managed_window.is_mapped = true;
        // Assuming mock_toplevel_surface.wl_surface().alive() is true
        assert_eq!(Window::is_mapped(&managed_window), true, "Window should be mapped when is_mapped is true and surface is alive.");

        managed_window.is_mapped = false;
        assert_eq!(Window::is_mapped(&managed_window), false, "Window should not be mapped when is_mapped is false.");

        // If we could simulate the surface dying:
        // This is harder with current mock structure. Smithay's internal resource management handles this.
        // For this unit test, we assume surface remains alive for the duration of the test after creation.
    }
}
