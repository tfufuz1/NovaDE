use calloop::{EventLoop, LoopSignal};
use smithay::{
    delegate_compositor, delegate_output, delegate_seat, delegate_shm, delegate_xdg_shell,
    desktop::{Space, Window},
    reexports::wayland_server::{
        protocol::{wl_shm, wl_surface::WlSurface, wl_seat}, 
        Client, Display,
    },
    wayland::{
        compositor::{
            client_compositor_state, CompositorClientState, CompositorHandler, CompositorState, SurfaceData,
            on_commit_buffer_handler, 
        },
        output::{OutputHandler, OutputManagerState},
        seat::{SeatClientData, SeatHandler, SeatState, XkbConfig},
        shell::xdg::{XdgClientData, XdgShellHandler, XdgShellState, ToplevelSurface}, 
        shm::{buffer_attributes, is_shm_buffer, ShmClientData, ShmHandler, ShmState}, 
        Serial,
    },
    utils::{Physical, Rectangle},
    backend::{
        drm::{DrmDevice, DrmDisplay, DrmNode, DrmSurface}, // Added DrmDevice, DrmDisplay, DrmSurface
        egl::{EGLContext, EGLDisplay},
        renderer::gles::GlesRenderer,
        session::{direct::DirectSession, Session as SessionTrait},
        udev::UdevBackend,
    },
    reexports::drm::control::crtc, // Added for crtc::Handle
};
use tracing::{error, info, warn};
use std::{cell::RefCell, collections::HashMap}; // Added HashMap

use crate::compositor::state::{NovadeCompositorState, SurfaceDataExt};

// Define the NovadeCompositorState struct to also implement the necessary Smithay handler traits.
// Smithay's delegate macros require the state struct to implement these.

impl CompositorHandler for NovadeCompositorState {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }
    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<CompositorClientState>().unwrap()
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);

        // Ensure SurfaceDataExt is initialized. This should ideally be done when the surface gets a role.
        // If it's not present, we can't store the texture.
        if !surface.data_map().contains::<RefCell<SurfaceDataExt>>() {
            warn!("Committed surface {:?} has no SurfaceDataExt. Initializing now. This should ideally be done at role assignment.", surface.id());
            surface.data_map().insert_if_missing(|| RefCell::new(SurfaceDataExt::default()));
        }
        
        if let Some(surface_data_refcell) = surface.get_data::<RefCell<SurfaceDataExt>>() {
            let mut surface_data = surface_data_refcell.borrow_mut();
            
            if let Some(buffer) = self.compositor_state().get_buffer(surface) {
                if is_shm_buffer(&buffer) {
                    match buffer_attributes(&buffer) {
                        Ok(attributes) => {
                            // Full damage for now, as per subtask instructions for simplicity.
                            let damage = vec![Rectangle::from_loc_and_size(
                                (0, 0),
                                (attributes.width, attributes.height),
                            )];

                            match self.gles_renderer.import_shm_buffer(&buffer, Some(&attributes), &damage) {
                                Ok(texture) => {
                                    surface_data.texture = Some(texture);
                                    surface_data.damage_buffer.clear(); // Clear old screen damage on new texture
                                    info!("Successfully imported SHM buffer for surface {:?} into GlesTexture.", surface.id());
                                }
                                Err(err) => {
                                    error!("Failed to import SHM buffer for surface {:?}: {}", surface.id(), err);
                                    surface_data.texture = None; // Clear texture on import failure
                                }
                            }
                        }
                        Err(err) => {
                            error!("Failed to get SHM buffer attributes for surface {:?}: {}", surface.id(), err);
                            surface_data.texture = None;
                        }
                    }
                } else {
                    // Buffer is not SHM. Clear our SHM-specific texture.
                    if surface_data.texture.is_some() {
                        warn!("Surface {:?} committed with a non-SHM buffer, clearing existing SHM texture.", surface.id());
                        surface_data.texture = None;
                    }
                }
            } else {
                // No buffer attached (e.g., client attached a null buffer).
                if surface_data.texture.is_some() {
                    info!("Surface {:?} committed with a null buffer, clearing texture.", surface.id());
                    surface_data.texture = None;
                }
            }
        } else {
            // This case should ideally not be reached if we ensure SurfaceDataExt is initialized above or at role assignment.
            error!("Committed surface {:?} unexpectedly missing SurfaceDataExt after check/init.", surface.id());
        }

        // Damage tracking for rendering (simplified)
        // This should be more sophisticated, accumulating damage between frame renders.
        // For now, if a buffer was committed, we can add its full rectangle to our damage_buffer
        // in SurfaceDataExt if we were to use it for rendering.
        // The damage passed to import_shm_buffer is for uploading parts of the client buffer.
        // The damage_buffer in SurfaceDataExt would be for screen damage.
        
        // Example: if a surface is part of the space, mark its region as damaged for the renderer.
        // self.space.damage_element(window_representing_surface, ...);
    }
}

impl XdgShellHandler for NovadeCompositorState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }
    // Implement required methods for XdgShellHandler
    // For example, new_toplevel, new_popup, grab, etc.
    // These will involve interacting with the Space and focus management.
    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new(surface); // Create a Smithay Window from the ToplevelSurface
        self.space.map_element(window.clone(), (0, 0), true); // Map it into the space, activate
        info!("New toplevel surface {:?} mapped into space at (0,0) as Window {:?}", window.toplevel().wl_surface().id(), window);

        // Initialize SurfaceDataExt for the underlying WlSurface of this toplevel.
        window.toplevel().wl_surface().data_map().insert_if_missing(|| RefCell::new(SurfaceDataExt::default()));
        info!("Initialized SurfaceDataExt for new toplevel surface {:?}", window.toplevel().wl_surface().id());
    }
    fn new_popup(&mut self, popup: smithay::wayland::shell::xdg::PopupSurface, _parent: smithay::wayland::shell::xdg::PositionerState) {
        // TODO: Handle popup creation, mapping relative to parent, and SurfaceDataExt initialization.
        // For now, just initialize SurfaceDataExt.
        popup.wl_surface().data_map().insert_if_missing(|| RefCell::new(SurfaceDataExt::default()));
        info!("New popup surface created: {:?}, SurfaceDataExt initialized.", popup.wl_surface().id());
        // A full implementation would involve self.space.map_popup(...) or similar logic.
    }
    fn grab(&mut self, _surface: ToplevelSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        // Handle grab requests
    }
    // ... other XdgShellHandler methods
}

impl ShmHandler for NovadeCompositorState {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl SeatHandler for NovadeCompositorState {
    type KeyboardFocus = WlSurface; 
    type PointerFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &smithay::wayland::seat::Seat<Self>, _focused: Option<&Self::KeyboardFocus>) {
        // Handle keyboard focus changes
        // TODO: Implement focus tracking and event dispatching.
    }

    fn cursor_image(&mut self, _seat: &smithay::wayland::seat::Seat<Self>, image: smithay::wayland::seat::CursorImageStatus) {
        info!("Cursor image updated: {:?}", image);
        match image {
            CursorImageStatus::Hidden => {
                self.cursor_image_status = None;
                self.cursor_texture = None;
                // Hardware cursor hiding will be attempted in render_frame
            }
            CursorImageStatus::Surface(surface) => {
                self.cursor_image_status = Some(CursorImageStatus::Surface(surface.clone()));
                // Attempt to import the buffer as a texture for software rendering fallback
                // and to get attributes like hotspot.
                if let Some(buffer) = self.compositor_state.get_buffer(&surface) {
                    if is_shm_buffer(&buffer) {
                        if let Ok(attributes) = buffer_attributes(&buffer) {
                            let damage = vec![Rectangle::from_loc_and_size(
                                (0,0),
                                (attributes.width, attributes.height)
                            )];
                            match self.gles_renderer.import_shm_buffer(&buffer, Some(&attributes), &damage) {
                                Ok(texture) => {
                                    self.cursor_texture = Some(texture);
                                    // Hotspot from CursorImageAttributes if available, else default
                                    let data_map = surface.data_map();
                                    if let Some(attrs) = data_map.get::<smithay::wayland::seat::CursorImageAttributes>() {
                                        self.cursor_hotspot = attrs.hotspot;
                                    } else {
                                        self.cursor_hotspot = (0,0); // Default hotspot
                                    }
                                    info!("Cursor texture imported from SHM buffer, hotspot: {:?}", self.cursor_hotspot);
                                }
                                Err(err) => {
                                    error!("Failed to import SHM buffer for cursor: {}", err);
                                    self.cursor_texture = None;
                                    self.cursor_image_status = None;
                                }
                            }
                        } else {
                            error!("Failed to get SHM buffer attributes for cursor surface.");
                            self.cursor_texture = None;
                            self.cursor_image_status = None;
                        }
                    } else if smithay::wayland::dmabuf::get_dmabuf(&buffer).is_ok() {
                        // DMABuf cursor - store buffer and hotspot for hardware plane attempt.
                        // Software fallback texture import could also happen here if desired.
                        // For now, if it's a DMABuf, we primarily rely on hardware plane attempt in render_frame.
                        // We might still want to import it to GlesTexture for consistent hotspot handling or if hardware fails.
                        // Let's try to import for software fallback / hotspot for now.
                        match self.gles_renderer.import_dma(&smithay::wayland::dmabuf::get_dmabuf(&buffer).unwrap(), None) {
                            Ok(texture) => {
                                self.cursor_texture = Some(texture);
                                let data_map = surface.data_map();
                                if let Some(attrs) = data_map.get::<smithay::wayland::seat::CursorImageAttributes>() {
                                    self.cursor_hotspot = attrs.hotspot;
                                } else {
                                    self.cursor_hotspot = (0,0);
                                }
                                info!("Cursor texture imported from DMABuf, hotspot: {:?}", self.cursor_hotspot);
                            }
                            Err(err) => {
                                error!("Failed to import DMABuf for cursor software fallback: {}", err);
                                self.cursor_texture = None;
                                // Keep cursor_image_status as Surface so render_frame can still try hardware plane
                            }
                        }
                    } else {
                        warn!("Cursor surface has an unsupported buffer type.");
                        self.cursor_texture = None;
                        self.cursor_image_status = None;
                    }
                } else {
                    warn!("No buffer attached to cursor surface.");
                    self.cursor_texture = None;
                    self.cursor_image_status = None;
                }
            }
            CursorImageStatus::Named(name) => {
                warn!("Named cursors ('{}') are not yet supported. Hiding cursor.", name);
                self.cursor_image_status = None;
                self.cursor_texture = None;
            }
        }
    }
}

impl OutputHandler for NovadeCompositorState {
    // No specific methods required by the trait itself, but OutputManagerState needs it.
}


// Delegate macros - these connect the global states to NovadeCompositorState's implementations
delegate_compositor!(NovadeCompositorState);
delegate_xdg_shell!(NovadeCompositorState);
delegate_shm!(NovadeCompositorState);
delegate_seat!(NovadeCompositorState);
delegate_output!(NovadeCompositorState);


pub fn run_compositor() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Novade Compositor...");

    // Create the event loop
    let mut event_loop: EventLoop<NovadeCompositorState> =
        EventLoop::try_new().expect("Failed to initialize event loop");
    
    let _loop_signal: LoopSignal = event_loop.get_signal(); // Get the loop signal, store if needed later

    // Create the Wayland display
    let display: Display<NovadeCompositorState> = // display can be immutable after init
        Display::new().expect("Failed to initialize Wayland display");

    // Initialize NovadeCompositorState
    // The state needs a LoopHandle, which can be obtained from the EventLoop
    // However, LoopHandle is usually for dispatching from other threads or specific event sources.
    // The state itself is passed to event_loop.run().
    // For state initialization that needs the display handle, we pass display.handle().

    // DRM Node Discovery
    info!("Attempting to find a primary DRM node...");
    // Note: UdevBackend::new() can return an error, ensure it's handled.
    let udev_backend = UdevBackend::new().map_err(|e| {
        error!("Failed to initialize udev backend: {}", e);
        format!("Failed to initialize udev backend: {}", e)
    })?;

    let drm_node_path = udev_backend.device_list()
        .find(|event| 
            event.devtype() == "drm" && 
            event.get("ID_SEAT") == Some("seat0") && 
            event.has_tag("master-of-seat")
        )
        .or_else(|| udev_backend.device_list().find(|event| 
            event.devtype() == "drm" && 
            event.has_tag("master-of-seat"))
        )
        .or_else(|| udev_backend.device_list().find(|event| event.devtype() == "drm"))
        .and_then(|event| event.get("DEVNAME").map(std::path::PathBuf::from))
        .ok_or_else(|| {
            error!("No suitable DRM node found. Scanned devices: {:?}", udev_backend.device_list().collect::<Vec<_>>());
            "No suitable DRM node found".to_string()
        })?;
    
    let primary_drm_node = DrmNode::from_path(&drm_node_path).map_err(|e| {
        error!("Failed to create DrmNode from path {:?}: {}", drm_node_path, e);
        format!("Failed to create DrmNode from path {:?}: {}", drm_node_path, e)
    })?;
    info!("Using DRM node: {:?}", primary_drm_node.path());

    // Session Setup
    // The session needs to be kept alive, NovadeCompositorState will own it.
    let session = DirectSession::from_drm_fd(primary_drm_node.fd()).map_err(|e| {
        error!("Failed to create direct session from DRM FD for {:?}: {}. Ensure you have permissions or are using a seat manager like seatd/logind, or are running as root.", primary_drm_node.path(), e);
        format!("Failed to create direct session from DRM FD for {:?}: {}", primary_drm_node.path(), e)
    })?;
    info!("Direct session created for DRM node {:?}.", primary_drm_node.path());
    
    // EGLDisplay Creation
    // The EGLDisplay is created from the DRM node's file descriptor.
    // It's important that the DRM node is kept open for the lifetime of the EGLDisplay.
    // GlesRenderer will manage the EGLContext and EGLDisplay internally after its creation.
    let egl_display = match unsafe { EGLDisplay::new(primary_drm_node.fd()) } {
        Ok(display) => {
            info!("Successfully created EGLDisplay from DRM node FD.");
            display
        }
        Err(e) => {
            let egl_error_str = smithay::backend::egl::get_error_string();
            error!("Failed to create EGLDisplay: {}, EGL error string: {}", e, egl_error_str);
            return Err(format!("EGLDisplay creation failed: {} (EGL: {})", e, egl_error_str).into());
        }
    };

    // EGLContext Creation
    let context_attributes = smithay::backend::egl::ContextAttributes {
        version: (2, 0), // Request OpenGL ES 2.0
        profile: None,
        debug: cfg!(debug_assertions), // Enable EGL_CONTEXT_FLAGS_KHR, EGL_CONTEXT_OPENGL_DEBUG_BIT_KHR if supported
        robustness: None,
    };

    // Choose an EGLConfig. Smithay's EGLDisplay often finds a suitable one automatically,
    // but explicit attribute matching can be done if needed.
    // For GlesRenderer, we need a config that supports creating on-screen (window) surfaces.
    let egl_context = match egl_display.create_context_with_config_attribs(
        context_attributes,
        &[ 
            smithay::backend::egl::ffi::EGL_RENDERABLE_TYPE, smithay::backend::egl::ffi::EGL_OPENGL_ES2_BIT,
            smithay::backend::egl::ffi::EGL_SURFACE_TYPE, smithay::backend::egl::ffi::EGL_WINDOW_BIT,
            smithay::backend::egl::ffi::EGL_RED_SIZE, 8,
            smithay::backend::egl::ffi::EGL_GREEN_SIZE, 8,
            smithay::backend::egl::ffi::EGL_BLUE_SIZE, 8,
            smithay::backend::egl::ffi::EGL_ALPHA_SIZE, 8, 
            smithay::backend::egl::ffi::EGL_DEPTH_SIZE, 24, // Common depth buffer size
            smithay::backend::egl::ffi::EGL_NONE, // Terminator for the attribute list
        ],
    ) {
        Ok(context) => {
            info!("Successfully created EGLContext (OpenGL ES 2.0).");
            context
        }
        Err(e) => {
            let egl_error_str = smithay::backend::egl::get_error_string();
            error!("Failed to create EGLContext: {}, EGL error string: {}", e, egl_error_str);
            return Err(format!("EGLContext creation failed: {} (EGL: {})", e, egl_error_str).into());
        }
    };

    // GlesRenderer Instantiation
    // The EGLContext is moved into the GlesRenderer.
    let gles_renderer = match unsafe { GlesRenderer::new(egl_context) } {
        Ok(renderer) => {
            info!("Successfully instantiated GlesRenderer.");
            renderer
        }
        Err(e) => {
            // GlesRenderer::new logs glGetError internally.
            error!("Failed to create GlesRenderer: {}", e);
            return Err(format!("GlesRenderer creation failed: {}", e).into());
        }
    };

    // Initialize NovadeCompositorState with new graphics components

    // DrmDevice and DrmDisplay Initialization
    let drm_device = match DrmDevice::new(
        gles_renderer.clone(), // GlesRenderer must impl GbmAllocator + Clone. GlesRenderer is Rc-based.
        primary_drm_node.fd(), // Use the FD from the DrmNode (owned by NovadeCompositorState later)
        session.active(),      // Pass the session active state
        tracing::Span::current(), // For logging within DrmDevice
    ) {
        Ok(device) => {
            info!("Successfully created DrmDevice.");
            device
        }
        Err(e) => {
            error!("Failed to create DrmDevice: {}", e);
            return Err(format!("DrmDevice creation failed: {}", e).into());
        }
    };

    let drm_display = match DrmDisplay::new(
        &drm_device, // Pass reference to DrmDevice
        None,        // event_dispatcher, if not integrating with calloop here yet
        tracing::Span::current(),
    ) {
        Ok(display_state) => {
            info!("Successfully created DrmDisplay.");
            display_state
        }
        Err(e) => {
            error!("Failed to create DrmDisplay: {}", e);
            return Err(format!("DrmDisplay creation failed: {}", e).into());
        }
    };

    // Output Discovery and DrmSurface Creation
    let mut surfaces = HashMap::new();
    for (conn_handle, conn_info) in drm_display.connectors().iter() {
        if conn_info.is_connected() {
            info!("Connector {:?} is connected.", conn_handle);
            if let Some(mode) = conn_info.modes().get(0) { // Use the first preferred mode
                info!("Using mode: {:?}", mode.name());
                // Find a CRTC for this connector
                if let Some(crtc_handle) = conn_info.crtc() { // If a CRTC is already assigned by BIOS/firmware
                     info!("Connector {:?} already has CRTC {:?}, attempting to use it.", conn_handle, crtc_handle);
                     match drm_device.create_surface(crtc_handle, *mode, std::iter::once(*conn_handle)) {
                        Ok(surface) => {
                            info!("Created DrmSurface for CRTC {:?} with existing assignment.", crtc_handle);
                            surfaces.insert(crtc_handle, surface);
                            break; // Use the first successful surface
                        }
                        Err(err) => {
                            warn!("Failed to create DrmSurface for CRTC {:?} with existing assignment: {}. Trying to find another CRTC.", crtc_handle, err);
                        }
                    }
                } else {
                    // Try to find a suitable CRTC if none is assigned
                    let available_crtcs: Vec<crtc::Handle> = drm_device
                        .crtcs()
                        .iter()
                        .filter(|crtc_h| !surfaces.contains_key(crtc_h) && drm_device.is_crtc_connector_compatible(conn_info, **crtc_h))
                        .cloned()
                        .collect();

                    if let Some(&crtc_to_use) = available_crtcs.first() {
                        info!("Attempting to use CRTC {:?} for connector {:?}", crtc_to_use, conn_handle);
                        match drm_device.create_surface(crtc_to_use, *mode, std::iter::once(*conn_handle)) {
                            Ok(surface) => {
                                info!("Created DrmSurface for CRTC {:?} and connector {:?}", crtc_to_use, conn_handle);
                                surfaces.insert(crtc_to_use, surface);
                                break; // Use the first successful surface
                            }
                            Err(err) => {
                                warn!("Failed to create DrmSurface for CRTC {:?} and connector {:?}: {}", crtc_to_use, conn_handle, err);
                            }
                        }
                    } else {
                        warn!("No suitable CRTC found for connector {:?}", conn_handle);
                    }
                }
            } else {
                warn!("Connector {:?} has no modes.", conn_handle);
            }
        }
    }


    if surfaces.is_empty() {
        error!("No active DRM outputs found to create a surface for.");
        return Err("No usable DRM output found.".into());
    }
    
    info!("Successfully created {} DrmSurface(s).", surfaces.len());


    let mut state = NovadeCompositorState::new(
        display.handle(), 
        event_loop.handle(),
        gles_renderer,    // Pass the created GlesRenderer
        session,          // Pass the created Session
        primary_drm_node, // Pass the DrmNode (original node, DrmDevice has its FD)
        drm_device,       // Pass the DrmDevice
        drm_display,      // Pass the DrmDisplay
        surfaces          // Pass the HashMap of DrmSurfaces
    );


    // Register Wayland globals
    // The states within NovadeCompositorState (compositor_state, xdg_shell_state, etc.)
    // are already created in NovadeCompositorState::new.
    // Now we need to create the globals on the display.
    // Smithay 0.10.0 global creation:
    display.handle().create_global::<Self, _>(5, ()); // CompositorState, version 5 for wl_surface.offset
    display.handle().create_global::<Self, _>(1, ()); // XdgShellState (xdg_wm_base version 1 common)
    display.handle().create_global::<Self, _>(1, ()); // ShmState, version 1
    display.handle().create_global::<Self, _>(4, ()); // OutputManagerState, version 4 for xdg-output name/description
    
    // Seat global - version depends on features, e.g., 7 for pointer gestures. Start with a reasonable version.
    // The seat itself is already created in NovadeCompositorState::new and stored in state.seat
    // We just need to ensure it's advertised. The SeatState handles this.
    // We need to configure the seat (e.g., keyboard layout)
    state.seat.add_keyboard(XkbConfig::default(), 200, 25).expect("Failed to add keyboard to seat");
    state.seat.add_pointer();
    info!("Seat '{}' initialized with keyboard and pointer.", state.seat_name);


    // Add a Wayland socket for clients to connect to.
    // The socket name can be default ("wayland-0", "wayland-1", etc.) or specific.
    let listening_socket = display.add_socket_auto()?;
    info!(
        "Wayland compositor listening on socket: {:?}",
        listening_socket.get_socket_name().unwrap_or_default()
    );
    // TODO: Set WAYLAND_DISPLAY env variable if running nested or for clients.

    // Placeholder for backend initialization (DRM, libinput, etc.)
    // This will involve adding event sources to the calloop event loop.
    // For now, we'll just run a minimal loop.
    
    info!("Novade Compositor core components initialized.");

    // This will involve adding event sources to the calloop event loop.
    // For now, we'll just run a minimal loop.
    
    info!("Novade Compositor core components initialized.");

    // Setup a timer to drive the rendering and event processing loop.
    // This is a simplified approach; a more robust compositor would likely integrate
    // DRM page flip events directly as a calloop event source.
    use calloop::timer::{Timer, TimeoutAction};
    use std::time::Duration;

    let timer = Timer::immediate(); // Fire once immediately to render the first frame.
    event_loop.handle().insert_source(timer, move |_event, _metadata, shared_data| {
        // Process DRM device events. This handles page flip completions.
        if let Err(e) = shared_data.drm_device.process_events() {
            error!("Error processing DRM events: {}", e);
            // In a real scenario, might need to stop the loop or handle specific errors.
            // For now, we'll log and attempt to continue.
            // If it's a critical error (e.g., device lost), the session might become inactive.
        }

        // Perform rendering for the next frame.
        // The render_frame method clears, renders space, and queues a page flip.
        match shared_data.render_frame([0.1, 0.1, 0.1, 1.0]) { // Dark grey background
            Ok(_) => {
                // Frame submitted successfully.
            }
            Err(e) => {
                error!("Error rendering frame: {}", e);
                // If rendering fails consistently, might need to stop or re-initialize.
            }
        }
        
        // Re-schedule the timer for the next frame (e.g., aiming for ~60 FPS).
        // This creates a continuous rendering loop.
        TimeoutAction::ToDuration(Duration::from_millis(16)) // Roughly 60 FPS
    }).map_err(|e| format!("Failed to insert render timer: {}", e.to_string()))?;
    
    info!("Running event loop with periodic rendering timer...");
    event_loop.run(None, &mut state, |running_state| {
        // This closure is called after each event source dispatch (e.g., after our timer callback).
        // We can perform post-dispatch processing here.
        
        // Dispatch Wayland client events. This processes client requests and updates their states.
        if let Err(e) = running_state.display_handle.dispatch_clients(running_state) {
            error!("Error dispatching Wayland clients: {}", e);
            // Consider stopping the loop or handling the error.
        }

        // Flush client events.
        if let Err(e) = running_state.display_handle.flush_clients(running_state) {
            warn!("Error flushing Wayland clients: {}", e);
        }

        // The rendering and DRM event processing are now handled by the timer source.
        // Additional event sources (e.g., libinput for input events) would be added here as well.
    })?;

    info!("Novade Compositor shut down.");
    Ok(())
}

// Client data structs required by Smithay 0.10 delegate macros
// These are typically empty or hold client-specific state if needed.
// They need to be accessible where NovadeCompositorState is defined or used with clients.
// It's common to define them near the state or handler implementations.
// For now, defining them here.
// Note: These are not directly used by the delegate_X macros themselves, but by the
// handler trait implementations which might be called by client interactions.
// Smithay 0.10.0 examples place these more globally or ensure they are accessible.
// The `Client::get_data()` calls in the handlers need these.
// For example, `CompositorClientState` is used in `CompositorHandler::client_compositor_state`.
// Let's ensure they are public or appropriately scoped if state.rs needs them.
// For now, this file is self-contained for main logic.

// These are needed if you store per-client data using client.insert_user_data()
// and retrieve it via client.get_user_data().
// The CompositorClientState, ShmClientData, XdgClientData are specific types
// that Smithay uses with get_data<T>() for its internal global management.
// They are typically initialized for each client when the respective global is bound.

// Smithay 0.10.0 provides `client_compositor_state`, `client_shm_data`, `client_xdg_shell_data`
// as helper functions to initialize these for clients if you don't have custom client data.
// They are used in the default global handlers.

// Example: `client.insert_user_data(|| CompositorClientState::default());`
// This is often done when a client binds to a global.
// The handler traits then expect to retrieve this.
// Smithay's default global handlers (used via `display.handle().create_global`)
// will often initialize these default client states.
// So, we might not need to manually insert them unless we have custom logic.
// The `client.get_data::<CompositorClientState>().unwrap()` calls in the handlers
// assume these have been initialized.

// The provided `NovadeCompositorState` constructor and main function setup should be
// largely compatible with Smithay 0.10's expectations for these default client data types.
// If compilation errors arise around client data, it's usually about ensuring these
// types are known and correctly initialized for clients.
// For now, the handlers assume they are present.
```

**Note on `ShmState::new` and `OutputManagerState::new_with_xdg_output`**:
The `ShmState::new` function in Smithay 0.10 takes `&DisplayHandle` and `Vec<wl_shm::Format>` (and `slog::Logger` which is now `tracing::Span` or similar).
The `OutputManagerState::new_with_xdg_output` takes `&DisplayHandle` (and `slog::Logger`).
My `NovadeCompositorState::new` already initializes these correctly. The registration in `run_compositor` with `display.handle().create_global::<Self, _>(1, ());` will use these initialized states.

**Note on `delegate_seat!` and `SeatHandler`**:
The `SeatHandler` in Smithay 0.10.0 requires `KeyboardFocus` and `PointerFocus` associated types. I've used `wl_surface::WlSurface` as a placeholder. This will need to be adjusted when actual focus logic is implemented (likely to be `Window` or a similar abstraction).

**Note on Global Creation**:
The `display.handle().create_global::<Self, _>(version, data)` method is the modern way in Smithay to register globals. The `data` argument is often `()` if the global's state is managed internally by the respective `*State` struct (e.g., `CompositorState`) and the handler methods on `NovadeCompositorState` provide access to it. The version numbers are important for Wayland protocols.

I will now ensure that `novade-system/src/compositor/mod.rs` declares `main.rs` and `state.rs` as modules, and that `novade-system/src/lib.rs` declares the `compositor` module.
