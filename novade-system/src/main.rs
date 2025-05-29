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
use smithay::backend::input::{InputEvent, LibinputInputBackend, SeatEvent, Axis, KeyState};
use smithay::input::Seat; // Already imported via compositor::core::state but good to have explicitly if used here
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
        
        tracing::info!("MCP services (async block) setup complete.");
        (
            mcp_service_arc, 
            cpu_usage_service_instance,
            // mcp_client_spawner_for_state_binding,
        )
    });
    // --- MCP Service Initialization END ---

    let mut event_loop: EventLoop<'static, DesktopState /* NovadeCompositorState */> = EventLoop::try_new()
        .expect("Failed to create event loop");
    let mut display: Display<DesktopState /* NovadeCompositorState */> = Display::new()
        .expect("Failed to create Wayland display");
    
    let (mut backend, mut winit_event_loop) = winit::init_from_builder(
        winit::WinitEventLoopBuilder::new().with_title("NovaDE Compositor (Winit)"),
        None // No specific calloop handle needed here for init
    ).expect("Failed to initialize Winit backend");

    // Create the GlowRenderer using WinitGraphicsBackend
    let mut renderer = unsafe { GlowRenderer::new(backend.renderer()) }
        .expect("Failed to initialize GlowRenderer");
    
    let display_handle = display.handle();
    let loop_handle = event_loop.handle(); // Get loop_handle before moving event_loop into run

    // NovadeCompositorState initialization
    // Passing None for Gles2Renderer and Vulkan components as we are using GlowRenderer with Winit.
    // This implies NovadeCompositorState needs to be adapted or a new field for GlowRenderer added.
    // For this subtask, we focus on main.rs changes.
    let mut desktop_state = DesktopState::new( 
        loop_handle, // Pass the loop_handle here
        display_handle.clone(),
        // None, // old gles_renderer
        // None, None, None, None, None, // Vulkan parts
        // smithay::compositor::ActiveRendererType::Gles, // This would be Glow/Winit specific if adapted
    );
    // If DesktopState is adapted to hold GlowRenderer:
    // desktop_state.glow_renderer = Some(renderer); 
    // Or, renderer is passed around/accessed via Winit backend.
    // For now, `renderer` variable will be moved into the winit timer closure.

    tracing::info!("DesktopState created for Winit backend.");

    // Store initialized services in DesktopState (MCP, CPU Usage) - this part remains
    desktop_state.mcp_connection_service = Some(initialized_mcp_connection_service);
    desktop_state.cpu_usage_service = Some(initialized_cpu_usage_service);
    tracing::info!("MCPConnectionService and CpuUsageService stored in DesktopState.");

    create_all_wayland_globals(&mut desktop_state, &display_handle)
        .expect("Failed to ensure Wayland globals");
    tracing::info!("Wayland globals initialized.");

    // Initialize input capabilities on the seat
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
    if let Err(e) = desktop_state.seat.add_touch() { // Add touch capability
        tracing::warn!("Failed to add touch capability to seat: {}", e);
    } else {
        tracing::info!("Added touch capability to seat '{}'.", desktop_state.seat.name());
    }


    // Insert the Wayland display first
    event_loop.handle().insert_source(
        display,
        |client_stream, _, state: &mut DesktopState /* NovadeCompositorState */| {
            if let Err(err) = client_stream.dispatch(state) {
                tracing::error!("Error dispatching Wayland client: {}", err);
            }
        },
    ).expect("Failed to insert Wayland display source into event loop.");

    // Insert the libinput backend
    let mut libinput_backend = LibinputInputBackend::new(None::<fn(_)>);
    if libinput_backend.link_seat(&desktop_state.seat_name).is_ok() {
        event_loop.handle().insert_source(libinput_backend, move |event, _, state: &mut DesktopState| {
            // This is the existing libinput processing logic.
            // It should be adapted if process_input_event is implemented on DesktopState.
            match event {
                InputEvent::Keyboard { event, .. } => {
                    if let Some(keyboard) = state.seat.get_keyboard() {
                        let serial = smithay::utils::Serial::next();
                        keyboard.input(state, event.key_code(), event.state(), serial, event.time_msec(), |_, _, _| true);
                    }
                }
                InputEvent::PointerMotion { event, .. } => {
                    if let Some(pointer) = state.seat.get_pointer() {
                        let pos = pointer.current_position() + event.delta();
                        state.pointer_location = pos; 
                        pointer.motion(state, state.pointer_location, event.time_msec());
                    }
                }
                InputEvent::PointerButton { event, .. } => {
                    if let Some(pointer) = state.seat.get_pointer() {
                        pointer.button(state, event.button_code(), event.state(), event.time_msec());
                    }
                }
                InputEvent::PointerAxis { event, .. } => {
                    if let Some(pointer) = state.seat.get_pointer() {
                        let h = event.amount_discrete(Axis::Horizontal).unwrap_or(0.0);
                        let v = event.amount_discrete(Axis::Vertical).unwrap_or(0.0);
                        let h_c = event.amount(Axis::Horizontal).unwrap_or(0.0);
                        let v_c = event.amount(Axis::Vertical).unwrap_or(0.0);
                        let source = match event.source() {
                            smithay::backend::input::AxisSource::Wheel => smithay::input::pointer::AxisSource::Wheel,
                            smithay::backend::input::AxisSource::Finger => smithay::input::pointer::AxisSource::Finger,
                            smithay::backend::input::AxisSource::Continuous => smithay::input::pointer::AxisSource::Continuous,
                            smithay::backend::input::AxisSource::WheelTilt => smithay::input::pointer::AxisSource::WheelTilt,
                        };
                        if h != 0.0 || v != 0.0 || h_c != 0.0 || v_c != 0.0 {
                            pointer.axis(state, smithay::input::pointer::AxisFrame::new(event.time_msec())
                                .discrete(Axis::Horizontal, h as i32)
                                .discrete(Axis::Vertical, v as i32)
                                .value_continuous(Axis::Horizontal, h_c)
                                .value_continuous(Axis::Vertical, v_c)
                                .source(source).build());
                        }
                    }
                }
                InputEvent::TouchDown { event, .. } => {
                    if let Some(touch) = state.seat.get_touch() {
                        let serial = smithay::utils::Serial::next();
                        touch.down(state, serial, event.time_msec(), event.slot(), event.position(state.pointer_location.to_i32_round()));
                    }
                }
                InputEvent::TouchUp { event, .. } => {
                    if let Some(touch) = state.seat.get_touch() {
                        let serial = smithay::utils::Serial::next();
                        touch.up(state, serial, event.time_msec(), event.slot());
                    }
                }
                InputEvent::TouchMotion { event, .. } => {
                     if let Some(touch) = state.seat.get_touch() {
                        let serial = smithay::utils::Serial::next();
                        touch.motion(state, serial, event.time_msec(), event.slot(), event.position(state.pointer_location.to_i32_round()));
                    }
                }
                InputEvent::TouchFrame { .. } => { if let Some(touch) = state.seat.get_touch() { touch.frame(state); } }
                InputEvent::TouchCancel { .. } => { if let Some(touch) = state.seat.get_touch() { touch.cancel(state); } }
                _ => { tracing::trace!("Unhandled libinput event: {:?}", event); }
            }
        }).expect("Failed to insert libinput event source");
    } else {
        tracing::warn!("Failed to link libinput backend to seat, input will not work.");
    }
    
    // Winit event processing timer
    let winit_timer = Timer::immediate();
    let mut winit_renderer = renderer; // Move renderer here to be captured by the closure

    event_loop.handle().insert_source(winit_timer, move |_, _, state: &mut DesktopState /* NovadeCompositorState */| {
        let mut calloop_timeout_action = TimeoutAction::ToDuration(Duration::from_millis(16)); // Default reschedule
        
        if let Err(e) = winit_event_loop.dispatch_new_events(|event| {
            match event {
                WinitEvent::Resized { size, .. } => {
                    // Resize the renderer. WinitGraphicsBackend's resize is usually called by winit::init or through OutputHandler.
                    // For GlowRenderer, it might need explicit resize.
                    // state.glow_renderer.as_mut().unwrap().resize(size.width, size.height); // If state stores GlowRenderer
                    // For now, we assume the Winit backend handles output size changes which trigger OutputHandler.
                    tracing::info!("Winit window resized to: {:?}", size);
                }
                WinitEvent::CloseRequested { .. } => {
                    tracing::info!("Winit window close requested, initiating shutdown.");
                    calloop_timeout_action = TimeoutAction::Break; // Set action to break from event_loop.run
                }
                WinitEvent::Input(input_event) => {
                    // If not using LibinputInputBackend source, or for winit-specific inputs:
                    // state.process_winit_input_event(input_event);
                    tracing::trace!("Winit input event: {:?}", input_event);
                }
                WinitEvent::OutputCreated { output, .. } => {
                    tracing::info!("Winit backend created an output: {}", output.name());
                    // OutputHandler::new_output will be called for this.
                }
                WinitEvent::OutputResized { output, ..} => {
                    tracing::info!("Winit backend resized an output: {}", output.name());
                    // OutputHandler::output_mode_updated will be called.
                }
                WinitEvent::OutputDestroyed { output, .. } => {
                    tracing::info!("Winit backend destroyed an output: {}", output.name());
                    // OutputHandler::output_destroyed will be called.
                }
                // WinitEvent::Redraw => { /* Handled by rendering logic below */ }
                _ => {
                    // tracing::trace!("Other Winit event: {:?}", event);
                }
            }
        }) {
            tracing::error!("Error dispatching winit events: {}", e);
            calloop_timeout_action = TimeoutAction::Break; // Exit loop on error
        }

        if calloop_timeout_action == TimeoutAction::Break {
            return TimeoutAction::Break; // Propagate break request
        }

        // Perform rendering using Winit backend and GlowRenderer
        // This replaces the old manual rendering loop.
        let damage = state.space.damage_for_outputs(&state.outputs); // Get damage for all outputs known to space

        if let Err(e) = backend.bind() {
            tracing::error!("Failed to bind winit backend for rendering: {}", e);
            return calloop_timeout_action; // Reschedule or break
        }

        // Iterate over outputs known to the compositor state (which Winit should have created)
        for output in &state.outputs {
            let output_geometry = state.space.output_geometry(output).unwrap_or_else(|| {
                let fallback_size = backend.window_size().physical_size; // Use winit window size as fallback
                tracing::warn!("Output {} not found in space, using winit window size for geometry.", output.name());
                smithay::utils::Rectangle::from_loc_and_size((0,0), fallback_size)
            });
            let output_scale = output.current_scale().fractional_scale();
            
            // Collect render elements for this output
            let mut render_elements: Vec<crate::compositor::renderer_interface::abstraction::RenderElement> = Vec::new();
            for window_arc in state.space.elements_for_output(output) {
                if !window_arc.is_mapped() { continue; }
                let window_geometry = window_arc.geometry();
                let window_wl_surface = match window_arc.wl_surface() { Some(s) => s, None => continue };
                
                // Get SurfaceData - Assuming it's stored in WlSurface's user_data
                let surface_data_arc = match window_wl_surface.data_map().get::<std::sync::Arc<std::sync::Mutex<crate::compositor::surface_management::SurfaceData>>>() {
                    Some(data) => data.clone(), 
                    None => {
                        tracing::warn!("SurfaceData not found for WlSurface {:?} during Winit rendering.", window_wl_surface.id());
                        continue;
                    }
                };

                render_elements.push(crate::compositor::renderer_interface::abstraction::RenderElement::WaylandSurface {
                    surface_wl: &window_wl_surface, // This is a short-lived reference. Ensure it's valid.
                    surface_data_arc, // This Arc keeps SurfaceData alive.
                    geometry: window_geometry, // Geometry in space coordinates.
                    damage_surface_local: vec![], // TODO: Pass actual surface damage.
                });
            }
            
            // Actual rendering call
            // The GlowRenderer::render_frame needs to be adapted to take elements by reference or similar.
            // Or, elements need to be structured to be clonable or passed differently.
            // For now, assuming render_frame can work with this structure or will be adapted.
            // The render_frame method from the old loop might need to be a method on GlowRenderer or a helper.
            // This part is a placeholder for the actual rendering invocation.
            match unsafe { winit_renderer.render_frame_legacy_wrapper(&render_elements, output_geometry, output_scale) } {
                Ok(_) => {
                    tracing::trace!("Rendered frame for output {}", output.name());
                }
                Err(e) => {
                    tracing::error!("Error rendering frame for output {}: {:?}", output.name(), e);
                }
            }
        }
        
        if let Err(e) = backend.submit(None) { // Present to all windows/outputs managed by Winit backend
            tracing::error!("Failed to submit frame via winit backend: {}", e);
        }

        state.space.damage_all_outputs(); // Request redraw for next frame unconditionally for now
        
        // Dispatch frame callbacks to clients
        let now_ns = state.clock.now();
        let time_for_send_frames = std::time::Duration::from_nanos(now_ns);
        state.space.send_frames(time_for_send_frames);
        tracing::trace!("Dispatched frame callbacks via space.send_frames() at time (ns): {}", now_ns);

        if let Err(e) = state.display_handle.flush_clients() {
            tracing::warn!("Failed to flush clients post-winit-render: {}", e);
        }

        calloop_timeout_action // Reschedule or break
    }).expect("Failed to insert Winit event timer");

    tracing::info!("NovaDE System with Winit backend event loop starting...");
    event_loop.run(None, &mut desktop_state, |data| {
        // This closure is called after each event loop dispatch cycle.
        // Can be used for cleanup or periodic tasks not fitting other handlers.
        // For example, if we needed to manually clear damage: data.space.clear_damage();
    }).expect("Event loop failed");

    tracing::info!("NovaDE System shutting down.");
}
