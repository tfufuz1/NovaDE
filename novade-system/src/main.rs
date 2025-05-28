use calloop::EventLoop;
use smithay::{
    reexports::wayland_server::{Display, DisplayHandle},
    utils::Size,
};
use std::rc::Rc;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod compositor;
use compositor::core::state::DesktopState;

// For placeholder EGL/Renderer setup
use khronos_egl as egl;
use compositor::renderers::gles2::renderer::Gles2Renderer;

// For global creation - adjust path if globals.rs is elsewhere or content is directly in core
// Assuming create_all_wayland_globals is now in compositor::core::globals
use compositor::core::globals::create_all_wayland_globals;


fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("novade_system=info".parse().unwrap()))
        .init();

    tracing::info!("NovaDE System starting up...");

    let mut event_loop: EventLoop<'static, DesktopState> = EventLoop::try_new()
        .unwrap_or_else(|e| {
            tracing::error!("Failed to create event loop: {}", e);
            panic!("Failed to create event loop: {}", e);
        });

    let mut display: Display<DesktopState> = Display::new()
        .unwrap_or_else(|e| {
            tracing::error!("Failed to create Wayland display: {}", e);
            panic!("Failed to create Wayland display: {}", e);
        });

    // --- Placeholder EGL/Renderer Setup START ---
    tracing::warn!("Using placeholder EGL/Renderer setup. Visual output will not work.");
    
    let egl_instance = Rc::new(unsafe {
        egl::Instance::new(egl::Dynamic::from_name("libEGL.so.1").unwrap_or_else(|_| egl::Dynamic::from_name("libEGL.so").expect("Failed to load libEGL")))
    });

    let egl_display_placeholder: egl::Display = unsafe { 
        egl_instance.get_display(egl::DEFAULT_DISPLAY).unwrap_or(std::mem::zeroed())
    };
     if egl_display_placeholder == unsafe { std::mem::zeroed() } {
        tracing::warn!("EGL_DEFAULT_DISPLAY was zeroed, EGL display placeholder might be invalid.");
    }
    // Minimal EGL init for placeholder (might still fail if display is truly invalid)
    // if egl_display_placeholder != unsafe { std::mem::zeroed() } {
    //    unsafe { egl_instance.initialize(egl_display_placeholder).expect("Failed to initialize placeholder EGL display"); }
    // }


    let egl_context_placeholder: egl::Context = unsafe { std::mem::zeroed() }; 
    let initial_screen_size_placeholder = Size::from((800, 600));

    let glow_context_placeholder = unsafe {
        glow::Context::from_loader_function(|symbol| {
            let addr = egl_instance.get_proc_address(symbol);
            addr.map_or(std::ptr::null(), |p| p as *const _)
        })
    };

    let renderer_placeholder = Gles2Renderer::new(
        glow_context_placeholder,
        egl_display_placeholder,
        egl_context_placeholder,
        egl_instance.clone(), 
        initial_screen_size_placeholder,
        None, // No EGL surface
    ).expect("Failed to create placeholder Gles2Renderer");
    tracing::info!("Placeholder Gles2Renderer created.");
    // --- Placeholder EGL/Renderer Setup END ---

    let display_handle: DisplayHandle = display.handle();
    let loop_handle = event_loop.handle();

    let mut desktop_state = DesktopState::new(
        loop_handle, 
        display_handle.clone(),
    );
    tracing::info!("DesktopState created.");

    desktop_state.renderer = Some(renderer_placeholder);
    tracing::info!("Placeholder renderer assigned to DesktopState.");

    // Initialize Wayland Globals
    // This function might be more about logging or setting up future globals,
    // as compositor and shm are registered by their state initializations in DesktopState::new.
    create_all_wayland_globals(&mut desktop_state, &display_handle) 
        .expect("Failed to ensure Wayland globals were created/logged.");
    tracing::info!("Wayland globals initialized/logged.");

    // --- Placeholder Output Setup START ---
    let physical_properties = smithay::output::PhysicalProperties {
        size: (527, 296).into(), // Example: 23-inch 16:9 display in mm
        subpixel: smithay::output::Subpixel::Unknown,
        make: "NovaDE Placeholder Inc.".into(),
        model: "Virtual Display 1".into(),
    };
    let initial_mode = smithay::output::Mode {
        size: (1920, 1080).into(), // Common Full HD resolution
        refresh: 60_000,           // 60 Hz in mHz
    };
    let output_name = "placeholder-1".to_string(); // Name for the output (e.g., "DP-1", "HDMI-A-1")

    // Create a Smithay Output object
    let placeholder_output = smithay::output::Output::new(
        output_name.clone(),
        physical_properties,
        Some(tracing::info_span!("placeholder_output", name = %output_name)) // Attach a span for logging
    );

    // Set its initial state
    placeholder_output.change_current_state(
        Some(initial_mode),
        Some(smithay::utils::Transform::Normal),
        Some(smithay::output::Scale::Fractional(1.0.into())), // Smithay 0.10 uses FractionalScale
        Some((0, 0).into()) // Position in the global compositor space
    );
    placeholder_output.set_preferred(initial_mode);

    // Create the wl_output global for this output.
    // The zxdg_output_manager_v1 global is handled by OutputManagerState via new_with_xdg_output.
    // Output::create_global makes this specific output instance available to clients.
    let _placeholder_output_global = placeholder_output.create_global::<DesktopState>(
        &desktop_state.display_handle,
    );
    
    // Store the output in DesktopState and map it in the Space
    desktop_state.outputs.push(placeholder_output.clone()); // Clone for DesktopState's ownership
    desktop_state.space.map_output(&placeholder_output, (0,0)); // Map output to space at (0,0)
    // Damage all outputs in space to trigger redraw if needed (though no rendering happens yet)
    desktop_state.space.damage_all_outputs();

    tracing::info!("Created and registered placeholder output: {}", output_name);
    // --- Placeholder Output Setup END ---

    // Add keyboard and pointer capabilities to the seat
    if let Err(e) = desktop_state.seat.add_keyboard(
        Default::default(), // Default XKB config (empty rules, model, layout)
        200, // Repeat delay (ms)
        25   // Repeat rate (chars/sec)
    ) {
        tracing::warn!("Failed to add keyboard capability to seat: {}", e);
    } else {
        tracing::info!("Added keyboard capability to seat '{}'.", desktop_state.seat.name());
    }

    if let Err(e) = desktop_state.seat.add_pointer() {
        tracing::warn!("Failed to add pointer capability to seat: {}", e);
    } else {
        tracing::info!("Added pointer capability to seat '{}'.", desktop_state.seat.name());
    }
    // TODO: Add touch capability if needed: desktop_state.seat.add_touch();

    event_loop.handle().insert_source(
        display, 
        |client_stream, _, state: &mut DesktopState| {
            match client_stream.dispatch(state) {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Error dispatching Wayland client: {}", e);
                }
            }
        },
    ).expect("Failed to insert Wayland display source into event loop.");
    tracing::info!("Wayland display event source registered with Calloop.");

    tracing::info!("NovaDE System event loop starting...");
    loop {
        match event_loop.dispatch(Some(std::time::Duration::from_millis(16)), &mut desktop_state) {
            Ok(_) => {
                // Post-dispatch logic, e.g., space refresh if needed for other reasons.
                // For Smithay 0.10, explicit space.refresh() is less common for rendering,
                // as damage is usually handled via output-specific rendering logic.
            }
            Err(e) => {
                tracing::error!("Error during event loop dispatch: {}", e);
                break; 
            }
        }
        
        // --- Basic Rendering Integration START ---
        let renderer_mut_opt = desktop_state.renderer.as_mut();

        let mut renderer = if let Some(r) = renderer_mut_opt {
            r
        } else {
            tracing::error!("Renderer not available, skipping frame rendering.");
            std::thread::sleep(std::time::Duration::from_millis(16)); // Avoid busy loop
            if let Err(e) = desktop_state.display_handle.flush_clients() { // Still flush clients
                 tracing::warn!("Failed to flush clients (no renderer): {}", e);
            }
            continue;
        };

        // For each output (we have one placeholder output for now)
        if let Some(output) = desktop_state.outputs.first() {
            let output_geometry = output.current_mode().map_or_else(
                || smithay::utils::Rectangle::from_loc_and_size((0,0), renderer.screen_size()),
                |mode| smithay::utils::Rectangle::from_loc_and_size((0,0), mode.size)
            );
            let output_scale = output.current_scale().fractional_scale(); // f64

            let mut render_elements: Vec<crate::compositor::renderer_interface::abstraction::RenderElement> = Vec::new();

            // Iterate through windows in the space
            for window_arc in desktop_state.space.elements() {
                if !window_arc.is_mapped() {
                    continue;
                }

                let window_geometry = window_arc.geometry();
                let window_wl_surface = match window_arc.wl_surface() {
                    Some(s) => s,
                    None => {
                        tracing::warn!("Mapped window {:?} has no WlSurface.", window_arc.id());
                        continue;
                    }
                };

                let surface_data_arc = match window_wl_surface.data_map().get::<std::sync::Arc<std::sync::Mutex<crate::compositor::surface_management::SurfaceData>>>() {
                    Some(data) => data.clone(),
                    None => {
                        tracing::warn!("SurfaceData missing for a mapped window's WlSurface {:?}", window_wl_surface.id());
                        continue;
                    }
                };
                
                // For now, assume any mapped window with a buffer needs rendering.
                // Damage tracking will be refined.
                render_elements.push(crate::compositor::renderer_interface::abstraction::RenderElement::WaylandSurface {
                    surface_wl: &window_wl_surface,
                    surface_data_arc,
                    geometry: window_geometry,
                    damage_surface_local: vec![], // Placeholder: This should be surface-local damage
                });
            }

            // TODO: Add cursor rendering element (requires cursor state management)

            if let Err(err) = renderer.render_frame(
                render_elements.iter(), // Pass iterator
                output_geometry,
                output_scale,
            ) {
                tracing::error!("Error rendering frame: {:?}", err);
                // Handle rendering errors (e.g., context loss)
            }

            if let Err(err) = renderer.present_frame() {
                tracing::error!("Error presenting frame: {:?}", err);
                // Handle presentation errors
            }
        }
        // --- Basic Rendering Integration END ---

        desktop_state.last_render_time = std::time::Instant::now();
        
        // Flush clients after rendering and other operations
        if let Err(e) = desktop_state.display_handle.flush_clients() {
            tracing::warn!("Failed to flush clients post-render: {}", e);
        }
    }

    tracing::info!("NovaDE System shutting down.");
}
