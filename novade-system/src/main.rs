use calloop::EventLoop;
use smithay::{
    reexports::wayland_server::{Display, DisplayHandle},
    utils::Size,
};
use std::rc::Rc;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// --- MCP Related Imports START ---
use novade_domain::ai_interaction_service::{
    MCPServerConfig,
    ClientCapabilities as DomainClientCapabilities,
    MCPClientInstance,
    MCPConnectionService,
    transport::ActualStdioTransport,
    types::JsonRpcRequest as DomainJsonRpcRequest,
};
use crate::mcp_client_service::{DefaultMCPClientService, IMCPClientService};
use std::sync::Arc;
use tokio::sync::{Mutex as TokioMutex, mpsc as tokio_mpsc};
use anyhow::Context as AnyhowContext;
// --- MCP Related Imports END ---

// --- Domain Service Imports ---
use novade_domain::cpu_usage_service::{DefaultCpuUsageService, ICpuUsageService as DomainICpuUsageService};
// --- Domain Service Imports END ---

mod compositor;
use compositor::core::state::DesktopState;

// For placeholder EGL/Renderer setup
use khronos_egl as egl;
use compositor::renderers::gles2::renderer::Gles2Renderer;

// For global creation
use compositor::core::globals::create_all_wayland_globals;


fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("novade_system=info".parse().unwrap()))
        .init();

    tracing::info!("NovaDE System starting up...");

    // --- MCP Service Initialization START ---
    let initialized_mcp_connection_service: Arc<TokioMutex<MCPConnectionService>>;
    let initialized_cpu_usage_service: Arc<dyn DomainICpuUsageService>;
    // let initialized_mcp_client_spawner: Arc<dyn IMCPClientService>; // If storing spawner

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime for MCP setup");

    (
        initialized_mcp_connection_service,
        initialized_cpu_usage_service,
        // initialized_mcp_client_spawner,
    ) = rt.block_on(async {
        tracing::info!("Initializing MCP services (async block)...");
        let mcp_client_spawner: Arc<dyn IMCPClientService> = Arc::new(DefaultMCPClientService::new());
        // let mcp_client_spawner_for_state_binding = mcp_client_spawner.clone();

        let default_client_caps = DomainClientCapabilities {
            supports_streaming: true,
        };
        let mut mcp_connection_service_instance = MCPConnectionService::new(default_client_caps);

        let server_configs = vec![
            MCPServerConfig {
                host: "cpu_usage_server".to_string(),
                command: "cpu_mcp_server".to_string(),
                args: vec![],
                port: 0,
            },
            // Add other server configs here
        ];

        for config in server_configs {
            tracing::info!("Setting up MCP server: {}", config.host);
            match mcp_client_spawner.spawn_stdio_server(config.command.clone(), config.args.clone()).await {
                Ok(stdio_process) => {
                    let (notification_tx, notification_rx) = tokio_mpsc::unbounded_channel::<DomainJsonRpcRequest>();
                    
                    let transport = Arc::new(TokioMutex::new(
                        ActualStdioTransport::new(
                            stdio_process.stdin,
                            stdio_process.stdout,
                            stdio_process.pid,
                            notification_tx,
                        ),
                    ));

                    let mut client_instance = MCPClientInstance::new(
                        config.clone(),
                        mcp_connection_service_instance.get_default_client_capabilities().clone(),
                        transport.clone(),
                        notification_rx,
                    );

                    match client_instance.connect_and_initialize().await {
                        Ok(_) => {
                            tracing::info!("Successfully connected and initialized MCP client for {}", config.host);
                            if let Err(e) = mcp_connection_service_instance.add_managed_client(Arc::new(TokioMutex::new(client_instance))).await {
                                tracing::error!("Failed to add managed MCP client for {}: {:?}", config.host, e);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to connect and initialize MCP client for {}: {:?}", config.host, e);
                            if let Err(term_err) = mcp_client_spawner.terminate_stdio_server(stdio_process.pid).await {
                                tracing::error!("Failed to terminate process {} for server {}: {:?}", stdio_process.pid, config.host, term_err);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to spawn stdio server for command {}: {:?}", config.command, e);
                }
            }
        }
        
        let mcp_service_arc = Arc::new(TokioMutex::new(mcp_connection_service_instance));
        
        let cpu_server_id_for_service = "cpu_usage_server".to_string(); // Must match config.host
        let cpu_usage_service_instance = Arc::new(DefaultCpuUsageService::new(
            Arc::clone(&mcp_service_arc),
            Some(cpu_server_id_for_service) 
        ));
        
        tracing::info!("MCP services (async block) setup complete.");
        (
            mcp_service_arc, 
            cpu_usage_service_instance,
            // mcp_client_spawner_for_state_binding,
        )
    });
    // --- MCP Service Initialization END ---

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
        None, 
    ).expect("Failed to create placeholder Gles2Renderer");
    tracing::info!("Placeholder Gles2Renderer created.");
    // --- Placeholder EGL/Renderer Setup END ---

    let display_handle: DisplayHandle = display.handle();
    let loop_handle = event_loop.handle();

    let mut desktop_state = DesktopState::new( // DesktopState::new() now initializes service fields to None
        loop_handle, 
        display_handle.clone(),
    );
    tracing::info!("DesktopState created.");

    // --- Store initialized services in DesktopState ---
    desktop_state.mcp_connection_service = Some(initialized_mcp_connection_service);
    desktop_state.cpu_usage_service = Some(initialized_cpu_usage_service);
    // if let Some(spawner) = initialized_mcp_client_spawner {
    //     desktop_state.mcp_client_spawner = Some(spawner);
    // }
    tracing::info!("MCPConnectionService and CpuUsageService stored in DesktopState.");
    // --- End Storing services ---

    desktop_state.renderer = Some(renderer_placeholder);
    tracing::info!("Placeholder renderer assigned to DesktopState.");

    create_all_wayland_globals(&mut desktop_state, &display_handle) 
        .expect("Failed to ensure Wayland globals were created/logged.");
    tracing::info!("Wayland globals initialized/logged.");

    // --- Placeholder Output Setup START ---
    let physical_properties = smithay::output::PhysicalProperties {
        size: (527, 296).into(),
        subpixel: smithay::output::Subpixel::Unknown,
        make: "NovaDE Placeholder Inc.".into(),
        model: "Virtual Display 1".into(),
    };
    let initial_mode = smithay::output::Mode {
        size: (1920, 1080).into(),
        refresh: 60_000,
    };
    let output_name = "placeholder-1".to_string();
    let placeholder_output = smithay::output::Output::new(
        output_name.clone(),
        physical_properties,
        Some(tracing::info_span!("placeholder_output", name = %output_name))
    );
    placeholder_output.change_current_state(
        Some(initial_mode),
        Some(smithay::utils::Transform::Normal),
        Some(smithay::output::Scale::Fractional(1.0.into())),
        Some((0, 0).into())
    );
    placeholder_output.set_preferred(initial_mode);
    let _placeholder_output_global = placeholder_output.create_global::<DesktopState>(
        &desktop_state.display_handle,
    );
    desktop_state.outputs.push(placeholder_output.clone());
    desktop_state.space.map_output(&placeholder_output, (0,0));
    desktop_state.space.damage_all_outputs();
    tracing::info!("Created and registered placeholder output: {}", output_name);
    // --- Placeholder Output Setup END ---

    if let Err(e) = desktop_state.seat.add_keyboard(Default::default(), 200, 25) {
        tracing::warn!("Failed to add keyboard capability to seat: {}", e);
    } else {
        tracing::info!("Added keyboard capability to seat '{}'.", desktop_state.seat.name());
    }
    if let Err(e) = desktop_state.seat.add_pointer() {
        tracing::warn!("Failed to add pointer capability to seat: {}", e);
    } else {
        tracing::info!("Added pointer capability to seat '{}'.", desktop_state.seat.name());
    }

    event_loop.handle().insert_source(
        display, 
        |client_stream, _, state: &mut DesktopState| {
            match client_stream.dispatch(state) {
                Ok(_) => {}
                Err(e) => tracing::error!("Error dispatching Wayland client: {}", e),
            }
        },
    ).expect("Failed to insert Wayland display source into event loop.");
    tracing::info!("Wayland display event source registered with Calloop.");

    tracing::info!("NovaDE System event loop starting...");
    loop {
        match event_loop.dispatch(Some(std::time::Duration::from_millis(16)), &mut desktop_state) {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("Error during event loop dispatch: {}", e);
                break; 
            }
        }
        
        let renderer_mut_opt = desktop_state.renderer.as_mut();
        let mut renderer = if let Some(r) = renderer_mut_opt { r } else {
            tracing::error!("Renderer not available, skipping frame rendering.");
            std::thread::sleep(std::time::Duration::from_millis(16));
            if let Err(e) = desktop_state.display_handle.flush_clients() {
                 tracing::warn!("Failed to flush clients (no renderer): {}", e);
            }
            continue;
        };

        if let Some(output) = desktop_state.outputs.first() {
            let output_geometry = output.current_mode().map_or_else(
                || smithay::utils::Rectangle::from_loc_and_size((0,0), renderer.screen_size()),
                |mode| smithay::utils::Rectangle::from_loc_and_size((0,0), mode.size)
            );
            let output_scale = output.current_scale().fractional_scale();
            let mut render_elements: Vec<crate::compositor::renderer_interface::abstraction::RenderElement> = Vec::new();

            for window_arc in desktop_state.space.elements() {
                if !window_arc.is_mapped() { continue; }
                let window_geometry = window_arc.geometry();
                let window_wl_surface = match window_arc.wl_surface() { Some(s) => s, None => continue };
                let surface_data_arc = match window_wl_surface.data_map().get::<std::sync::Arc<std::sync::Mutex<crate::compositor::surface_management::SurfaceData>>>() {
                    Some(data) => data.clone(), None => continue
                };
                render_elements.push(crate::compositor::renderer_interface::abstraction::RenderElement::WaylandSurface {
                    surface_wl: &window_wl_surface,
                    surface_data_arc,
                    geometry: window_geometry,
                    damage_surface_local: vec![], 
                });
            }

            if let Err(err) = renderer.render_frame(render_elements.iter(), output_geometry, output_scale) {
                tracing::error!("Error rendering frame: {:?}", err);
            }
            if let Err(err) = renderer.present_frame() {
                tracing::error!("Error presenting frame: {:?}", err);
            }
        }
        desktop_state.last_render_time = std::time::Instant::now();
        if let Err(e) = desktop_state.display_handle.flush_clients() {
            tracing::warn!("Failed to flush clients post-render: {}", e);
        }

        // Dispatch frame callbacks
        let now_ns = desktop_state.clock.now();
        let time_for_send_frames = std::time::Duration::from_nanos(now_ns);
        desktop_state.space.send_frames(time_for_send_frames);
        tracing::trace!("Dispatched frame callbacks via space.send_frames() at time (ns): {}", now_ns);
    }
    tracing::info!("NovaDE System shutting down.");
}
