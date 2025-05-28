use crate::compositor::{
    core::state::{DesktopState, GraphicsBackendHandles}, // Import GraphicsBackendHandles
    renderers::gles2_renderer::Gles2NovaRenderer, 
    // renderer_interface::abstraction::FrameRenderer,
};
use smithay::{
    backend::{
        drm::{DrmDevice, DrmDisplay, DrmNode, DrmSurface},
        egl::{EGLContext, EGLDisplay}, // Keep EGLDisplay for Gles2NovaRenderer creation
        session::{direct::DirectSession, Session as SessionTrait},
        udev::UdevBackend,
    },
    reexports::{
        calloop::{timer::{Timer, TimeoutAction}, EventLoop}, // LoopSignal removed as it's obtained from event_loop.get_signal()
        wayland_server::Display, // DisplayHandle removed as it's obtained from display.handle()
    },
    wayland::{
        seat::XkbConfig,
        output::OutputManagerState, // For creating output globals
        compositor::CompositorState, // For creating compositor globals
        shell::xdg::XdgShellState,   // For creating xdg_shell globals
        shm::ShmState,               // For creating shm globals
        seat::SeatState,             // For creating seat globals
    }
};
use std::{collections::HashMap, error::Error, time::Duration, sync::{Arc, Mutex}}; // Added Arc, Mutex
use tracing::{error, info, warn};

// Placeholder implementations for DesktopState methods used in the loop.
// These would eventually call actual methods on DesktopState or its fields.
impl DesktopState {
    pub fn render_frame_placeholder(&mut self, _clear_color: [f32; 4]) -> Result<(), Box<dyn Error>> {
        // This should use self.renderer and self.drm_device / self.drm_surfaces
        info!("Placeholder: DesktopState::render_frame_placeholder called. Will use self.renderer and DRM handles.");
        // Example: Iterate over DRM surfaces and render on each.
        // for (_crtc, drm_surface) in self.drm_surfaces.lock().unwrap().iter_mut() {
        //     // self.renderer.render_frame_on_surface(drm_surface, ...)?;
        // }
        // self.renderer.present_frame(None)?; // Or per surface
        Ok(())
    }

    pub fn dispatch_clients_placeholder(&mut self) -> Result<(), Box<dyn Error>> {
        self.display_handle.dispatch_clients(self).map_err(Into::into)
    }

    pub fn flush_clients_placeholder(&mut self) -> Result<(), Box<dyn Error>> {
        self.display_handle.flush_clients(self).map_err(Into::into)
    }

    // Method to access DrmDevice for event processing
    // This assumes DrmDevice is stored in DesktopState as self.drm_device (Arc<DrmDevice>)
    pub fn process_drm_events_placeholder(&mut self) -> Result<(), smithay::backend::drm::DrmError> {
        self.drm_device.process_events()
    }
}


pub fn run_compositor_event_loop() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting Novade Compositor Event Loop...");

    let mut event_loop: EventLoop<DesktopState> = // DesktopState no longer generic over BackendData
        EventLoop::try_new().expect("Failed to initialize event loop");
    
    let loop_signal = event_loop.get_signal(); // Get LoopSignal

    let display: Display<DesktopState> = // DesktopState no longer generic
        Display::new().expect("Failed to initialize Wayland display");
    let display_handle = display.handle();

    info!("Attempting to find a primary DRM node...");
    let udev_backend = UdevBackend::new().map_err(|e| format!("Failed to initialize udev backend: {}", e))?;

    let drm_node_path = udev_backend
        .device_list()
        .find(|event| {
            event.devtype() == "drm"
                && event.get("ID_SEAT") == Some("seat0")
                && event.has_tag("master-of-seat")
        })
        .or_else(|| {
            udev_backend
                .device_list()
                .find(|event| event.devtype() == "drm" && event.has_tag("master-of-seat"))
        })
        .or_else(|| udev_backend.device_list().find(|event| event.devtype() == "drm"))
        .and_then(|event| event.get("DEVNAME").map(std::path::PathBuf::from))
        .ok_or_else(|| {
            error!(
                "No suitable DRM node found. Scanned devices: {:?}",
                udev_backend.device_list().collect::<Vec<_>>()
            );
            "No suitable DRM node found".to_string()
        })?;

    let primary_drm_node = DrmNode::from_path(&drm_node_path)
        .map_err(|e| format!("Failed to create DrmNode from path {:?}: {}", drm_node_path, e))?;
    info!("Using DRM node: {:?}", primary_drm_node.path());

    let session = Arc::new(DirectSession::from_drm_fd(primary_drm_node.fd()).map_err(|e| {
        error!(
            "Failed to create direct session from DRM FD for {:?}: {}. Ensure you have permissions or are using a seat manager like seatd/logind, or are running as root.",
            primary_drm_node.path(),
            e
        );
        format!(
            "Failed to create direct session from DRM FD for {:?}: {}",
            primary_drm_node.path(),
            e
        )
    })?);
    info!("Direct session created for DRM node {:?} and wrapped in Arc.", primary_drm_node.path());
    let egl_display = unsafe { EGLDisplay::new(primary_drm_node.fd()) }.map_err(|e| {
        let egl_error_str = smithay::backend::egl::get_error_string();
        error!("Failed to create EGLDisplay: {}, EGL error string: {}", e, egl_error_str);
        format!("EGLDisplay creation failed: {} (EGL: {})", e, egl_error_str)
    })?;
    info!("Successfully created EGLDisplay from DRM node FD.");

    // Renderer and DRM Device/Surface Initialization (Corrected Order)
    let egl_context = EGLContext::new(&egl_display, slog_scope::logger())
        .map_err(|e| format!("Failed to create EGL context: {}", e))?;
    
    // This is Smithay's Gles2Renderer, which Gles2NovaRenderer will wrap/use.
    let smithay_gles_renderer = unsafe {
        smithay::backend::renderer::gles2::Gles2Renderer::new(egl_context.clone(), slog_scope::logger())
            .map_err(|e| format!("Failed to create Smithay Gles2Renderer: {}", e))?
    };

    let drm_device = Arc::new(DrmDevice::new(
        smithay_gles_renderer.clone(), // Pass Smithay's GlesRenderer as GbmAllocator
        primary_drm_node.fd(),
        session.active(), // session is Arc<DirectSession>
        slog_scope::logger(),
    ).map_err(|e| format!("DrmDevice creation failed: {}", e))?);
    info!("DrmDevice created and wrapped in Arc.");
    
    let drm_display = DrmDisplay::new(
        &drm_device, // Pass reference to Arc<DrmDevice>
        None,        // event_dispatcher
        slog_scope::logger(),
    ).map_err(|e| format!("DrmDisplay creation failed: {}", e))?;

    let mut drm_surfaces_map = HashMap::new();
    for (conn_handle, conn_info) in drm_display.connectors().iter() {
        if conn_info.is_connected() {
            info!("Connector {:?} is connected.", conn_handle);
            if let Some(mode) = conn_info.modes().get(0) {
                info!("Using mode: {:?}", mode.name());
                let crtc_option = conn_info.crtc().or_else(|| {
                    drm_device
                        .crtcs()
                        .iter()
                        .find(|crtc_h| !drm_surfaces_map.contains_key(crtc_h) && drm_device.is_crtc_connector_compatible(conn_info, **crtc_h))
                        .cloned()
                });

                if let Some(crtc_handle) = crtc_option {
                     match drm_device.create_surface(crtc_handle, *mode, std::iter::once(*conn_handle)) {
                        Ok(surface) => {
                            info!("Created DrmSurface for CRTC {:?}.", crtc_handle);
                            drm_surfaces_map.insert(crtc_handle, surface);
                        }
                        Err(err) => warn!("Failed to create DrmSurface for CRTC {:?}: {}", crtc_handle, err),
                    }
                } else {
                    warn!("No suitable CRTC found for connector {:?}", conn_handle);
                }
            } else {
                warn!("Connector {:?} has no modes.", conn_handle);
            }
        }
    }

    if drm_surfaces_map.is_empty() {
        error!("No active DRM outputs found to create a surface for.");
        return Err("No usable DRM output found.".into());
    }
    info!("Successfully created {} DrmSurface(s).", drm_surfaces_map.len());
    
    // Gles2NovaRenderer instantiation
    // Assuming Gles2NovaRenderer constructor is refactored to take the Smithay GlesRenderer,
    // DrmDevice, a specific DrmSurface (e.g. primary), DrmNode, and EGLDisplay.
    // For now, we pass one surface. Multi-monitor rendering logic will be inside Gles2NovaRenderer.
    let (any_crtc, any_surface) = drm_surfaces_map.iter().next().unwrap(); // Get a surface for init
    let gles_nova_renderer = Gles2NovaRenderer::new(
        (*drm_device).clone(), // DrmDevice is Arc, clone it
        any_surface.clone(),    // Pass a DrmSurface clone for initialization
        primary_drm_node.clone(),
        egl_display.clone(),
        slog_scope::logger(),
    ).map_err(|e| format!("Gles2NovaRenderer creation failed: {}", e.to_string()))?;


    // Prepare GraphicsBackendHandles
    let graphics_handles = GraphicsBackendHandles {
        session, // Already Arc<DirectSession>
        drm_device: drm_device.clone(), // Clone Arc<DrmDevice>
        drm_display, // DrmDisplay is not Arc here, owned by DesktopState
        drm_surfaces: Arc::new(Mutex::new(drm_surfaces_map)), // Wrap HashMap in Arc<Mutex<>>
        drm_node: primary_drm_node.clone(),
        egl_display: egl_display.clone(),
    };

    let mut desktop_state = DesktopState::new(
        display.handle(),
        event_loop.handle(),
        loop_signal, // Pass the loop_signal
        Box::new(gles_nova_renderer),
        graphics_handles,
    );

    // Create Wayland globals
    // Note: DesktopState is no longer generic, so <DesktopState> is sufficient.
    display_handle.create_global::<DesktopState, CompositorState>(5, ());
    display_handle.create_global::<DesktopState, XdgShellState>(1, ());
    display_handle.create_global::<DesktopState, ShmState>(1, ());
    display_handle.create_global::<DesktopState, OutputManagerState>(4, ());
    // Seat global: SeatState is managed within DesktopState.
    // The global is typically created when SeatState is initialized if it's version >= 7,
    // or you might need to create it explicitly for older versions or specific configurations.
    // DesktopState::new already creates SeatState. We need to ensure it's advertised.
    // Smithay 0.10+ SeatState creates the global automatically.
    // We do need to configure capabilities:
    desktop_state.seat_state.add_keyboard(XkbConfig::default(), 200, 25)
        .expect("Failed to add keyboard to seat");
    desktop_state.seat_state.add_pointer();
    info!("Seat '{}' initialized with keyboard and pointer.", desktop_state.seat_name);


    let listening_socket = display.add_socket_auto()?;
    info!(
        "Wayland compositor listening on socket: {:?}",
        listening_socket.get_socket_name().unwrap_or_default()
    );

    info!("Novade Compositor core components initialized. Running event loop...");

    let timer = Timer::immediate();
    event_loop.handle().insert_source(timer, move |_event, _metadata, shared_data| {
        // Process DRM device events
        if let Err(e) = shared_data.process_drm_events_placeholder() {
            error!("Error processing DRM events: {}", e);
            // Handle DRM errors, potentially stop loop or re-initialize
        }

        // Perform rendering
        match shared_data.render_frame_placeholder([0.1, 0.1, 0.1, 1.0]) {
            Ok(_) => {} // Frame submitted/rendered
            Err(e) => error!("Error rendering frame: {}", e),
        }
        
        TimeoutAction::ToDuration(Duration::from_millis(16)) // Aim for ~60 FPS
    }).map_err(|e| format!("Failed to insert render timer: {}", e.to_string()))?;
    
    event_loop.run(None, &mut desktop_state, |current_state| {
        // Dispatch Wayland client events
        if let Err(e) = current_state.dispatch_clients_placeholder() {
            error!("Error dispatching Wayland clients: {}", e);
        }

        // Flush client events
        if let Err(e) = current_state.flush_clients_placeholder() {
            warn!("Error flushing Wayland clients: {}", e);
        }
        // Additional post-dispatch processing can go here
    })?;

    info!("Novade Compositor shut down.");
    Ok(())
}
