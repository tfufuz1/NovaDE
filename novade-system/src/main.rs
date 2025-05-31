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

// --- D-Bus Notification Server Imports START ---
use novade_system::dbus_interfaces::NotificationsDBusService;
use novade_domain::notification_service::{DefaultNotificationManager, NotificationManager as DomainNotificationManagerTrait}; // Alias trait
use zbus::ConnectionBuilder;
// Arc and Mutex are already imported via MCP section (std::sync::Arc, tokio::sync::Mutex as TokioMutex)
// --- D-Bus Notification Server Imports END ---

// --- Domain Service Imports ---
use novade_domain::cpu_usage_service::{DefaultCpuUsageService, ICpuUsageService as DomainICpuUsageService};
// --- Domain Service Imports END ---

mod compositor;
use compositor::core::state::DesktopState;
use crate::compositor::backend::{CompositorBackend, BackendType, winit_backend::WinitBackend, drm_backend::DrmBackend};
use anyhow::Result;
use crate::system_services::SystemServices; // Added SystemServices import
use novade_domain::initialize_domain_layer;
use novade_core::config::DummyConfigService; // For initializing domain services
use std::path::PathBuf; // For domain service init


// For global creation
use compositor::core::globals::create_all_wayland_globals;

// --- Winit Backend Imports START ---
use smithay::backend::winit::{self, WinitEvent, WinitEventLoop, WinitGraphicsBackend};
use smithay::backend::renderer::glow::GlowRenderer;
use smithay::backend::renderer::gles::GlesError; // For error types, though Glow might have its own
use smithay::reexports::calloop::Error as CalloopError;
use smithay::reexports::calloop::timer::{Timer, TimeoutAction};
use std::time::Duration;
// --- Winit Backend Imports END ---

// --- Libinput Imports START ---
// Remove: use smithay::backend::input::{InputEvent, LibinputInputBackend, SeatEvent, Axis, KeyState};
// Keep: use smithay::input::Seat; // May still be used by other parts or can be removed if truly unused.
// For now, let's assume Seat might be used somewhere else or by DesktopState implicitly.
// If not, it can be cleaned up later.
use crate::input::libinput_handler::NovadeLibinputManager;
// UdevBackend and DirectSession are not used for the simplified approach
// use smithay::backend::session::direct::DirectSession;
// use smithay::backend::udev::UdevBackend;
// use std::path::PathBuf;
// --- Libinput Imports END ---


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

        // --- D-Bus Notification Server Initialization START ---
        tokio::spawn(async {
            tracing::info!("Initializing D-Bus Notification Server (background task)...");
            let domain_notification_manager = Arc::new(TokioMutex::new(DefaultNotificationManager::new()));
            let notifications_dbus_service = NotificationsDBusService::new(domain_notification_manager);

            match ConnectionBuilder::session()
                .name("org.freedesktop.Notifications")
                .expect("Failed to acquire D-Bus service name 'org.freedesktop.Notifications'")
                .serve_at("/org/freedesktop/Notifications", notifications_dbus_service)
                .expect("Failed to serve D-Bus interface at '/org/freedesktop/Notifications'")
                .build()
                .await
            {
                Ok(_conn) => {
                    // The connection object `_conn` must be kept alive for the service to run.
                    // Since it's created and awaited within this spawned task, if build() returns Ok,
                    // the service is running. If this task were to complete, _conn would be dropped.
                    // For a continuously running service, this task should ideally never complete normally.
                    // zbus::Error::NameTaken would be caught by expect.
                    // If build().await returns, it might mean the connection was lost or another issue.
                    tracing::info!("D-Bus Notification Server started and connection built. It will run as long as this task is alive.");
                    // To keep it alive indefinitely if build().await is not blocking forever (it should be for server):
                    // std::future::pending::<()>().await; // This would keep the task alive forever after setup
                    // However, zbus's build().await for a server connection should itself be a future that completes only on error/shutdown.
                }
                Err(e) => {
                    tracing::error!("D-Bus Notification Server failed to build or run: {}", e);
                }
            }
            tracing::warn!("D-Bus Notification Server task finished. This may indicate an issue if unexpected.");
        });
        tracing::info!("D-Bus Notification Server spawned as a background task.");
        // --- D-Bus Notification Server Initialization END ---
        
        tracing::info!("MCP services (async block) setup complete.");
        (
            mcp_service_arc, 
            cpu_usage_service_instance,
            // mcp_client_spawner_for_state_binding,
        )
    });
    // --- MCP Service Initialization END ---

    // --- Domain Services Initialization START ---
    let core_config_service = Arc::new(DummyConfigService::new()); // Placeholder
    let domain_services_arc = rt.block_on(async {
        tracing::info!("Initializing NovaDE Domain Layer (async block)...");
        match initialize_domain_layer(
            core_config_service,
            "current_user_id_placeholder".to_string(), // Replace with actual user ID logic if available
            None, // event_broadcast_capacity_override
            None, // theme_load_paths_override
            None, // token_load_paths_override
        ).await {
            Ok(services) => {
                tracing::info!("NovaDE Domain Layer Initialized Successfully.");
                Some(Arc::new(services))
            }
            Err(e) => {
                tracing::error!("Failed to initialize NovaDE Domain Layer: {:?}", e);
                None
            }
        }
    });
    // --- Domain Services Initialization END ---

    // --- System Services Initialization START ---
    let system_services_arc = if let Some(ds_arc) = domain_services_arc.as_ref() {
        rt.block_on(async { // Use the existing tokio runtime
            match SystemServices::new(ds_arc.clone()).await {
                Ok(services) => {
                    tracing::info!("SystemServices initialized successfully.");
                    Some(Arc::new(services))
                }
                Err(e) => {
                    tracing::error!("Failed to initialize SystemServices: {:?}", e);
                    None
                }
            }
        })
    } else {
        tracing::warn!("DomainServices not available, skipping SystemServices initialization.");
        None
    };
    // --- System Services Initialization END ---

    let mut event_loop: EventLoop<'static, DesktopState> = EventLoop::try_new()
        .expect("Failed to create event loop");
    let mut display: Display<DesktopState> = Display::new()
        .expect("Failed to create Wayland display");
    
    let display_handle = display.handle();
    let loop_handle = event_loop.handle();

    // DesktopState initialization (Renderer specific parts are now handled by backends)
    let mut desktop_state = DesktopState::new(
        loop_handle.clone(), // Pass the loop_handle
        display_handle.clone(),
        // Vulkan/GLES specific parts removed from here
    );
    tracing::info!("DesktopState created.");

    // Store initialized services in DesktopState
    desktop_state.mcp_connection_service = Some(initialized_mcp_connection_service);
    desktop_state.cpu_usage_service = Some(initialized_cpu_usage_service);
    desktop_state.domain_services = domain_services_arc;
    desktop_state.system_services = system_services_arc; // Add this line
    tracing::info!("Domain and System services stored in DesktopState.");

    create_all_wayland_globals(&mut desktop_state, &display_handle)
        .expect("Failed to ensure Wayland globals");
    tracing::info!("Wayland globals initialized.");

    // --- Libinput Backend Initialization START ---
    let mut novade_libinput_manager = NovadeLibinputManager::new(&desktop_state.seat_name)
        .expect("Failed to initialize NovadeLibinputManager");
    let libinput_event_source = novade_libinput_manager.into_event_source();

    event_loop.handle().insert_source(libinput_event_source, move |event, _, d_state: &mut DesktopState| {
        // Dispatch the event using the InputDispatcher from DesktopState
        d_state.input_dispatcher.dispatch_event(d_state, event);
        // No PostAction needed unless specific conditions arise
    }).expect("Failed to insert libinput event source into event loop.");
    tracing::info!("Libinput event source inserted into event loop.");
    // --- Libinput Backend Initialization END ---

    // --- Backend Selection and Initialization START ---
    let selected_backend_type = BackendType::Winit; // Hardcode Winit for now
    tracing::info!("Selected backend type: {:?}", selected_backend_type);

    // The backend variable needs to be mutable if its run method takes self by value
    // or if it's modified later. The trait defines `run(self, ...)`.
    // Let's use a Box<dyn CompositorBackend> to hold the selected backend.
    let mut active_backend: Box<dyn CompositorBackend> = match selected_backend_type {
        BackendType::Winit => {
            Box::new(WinitBackend::init(loop_handle.clone(), display_handle.clone(), &mut desktop_state)
                .expect("Failed to initialize Winit backend"))
        }
        BackendType::Drm => {
            // Box::new(DrmBackend::init(loop_handle.clone(), display_handle.clone(), &mut desktop_state)
            //    .expect("Failed to initialize DRM backend"))
            panic!("DRM backend selected but not fully implemented for run.");
        }
    };
    tracing::info!("Compositor backend initialized.");
    // --- Backend Selection and Initialization END ---

    // Insert the Wayland display first
    event_loop.handle().insert_source(
        display,
        |client_stream, _, state: &mut DesktopState| {
            if let Err(err) = client_stream.dispatch(state) {
                tracing::error!("Error dispatching Wayland client: {}", err);
            }
        },
    ).expect("Failed to insert Wayland display source into event loop.");

    // Call the backend's run method to set up its event sources within the main event loop
    if let Err(e) = active_backend.run(&mut desktop_state) {
       if selected_backend_type == BackendType::Winit {
            tracing::error!("WinitBackend setup via run() failed: {:?}", e);
            return;
       } else {
           tracing::error!("DRMBackend run failed: {:?}", e);
           return;
       }
    }

    tracing::info!("NovaDE System event loop starting with {} backend...",
        match selected_backend_type {
            BackendType::Winit => "Winit",
            BackendType::Drm => "DRM (Placeholder - expected to fail or not run)",
        }
    );

    // This is the main blocking call.
    event_loop.run(None, &mut desktop_state, |_desktop_state| {
        // This closure is called after each event loop dispatch cycle.
        // Can be used for cleanup or periodic tasks not fitting other handlers.
    }).expect("Event loop failed");

    tracing::info!("NovaDE System shutting down.");
}
