// This is novade-system/src/compositor/core.rs
// Main Wayland compositor logic, event loop, and global state management.

use std::{
    process::Command,
    sync::{Arc, Mutex as StdMutex},
    time::Duration,
    env,
};
use tracing::{info, warn, error, debug};

use smithay::{
    backend::{
        input::InputEvent, // Generic input event
        renderer::gles2::Gles2Renderer, // Example, will be part of MainRenderer
        // TODO: Add Vulkan imports when Vulkan renderer is integrated
    },
    desktop::{Space, Window, PopupManager, LayerSurface},
    reexports::{
        calloop::{EventLoop, LoopHandle, Dispatcher, PostAction, generic::Generic},
        wayland_server::{Display, DisplayHandle, Client, Backend, protocol::{wl_surface, wl_seat, wl_output}},
        wayland_protocols::{ // For creating globals
            xdg::{
                shell::server::xdg_wm_base,
                decoration::zv1::server::zxdg_decoration_manager_v1,
                activation::v1::server::xdg_activation_v1,
                output::zv1::server::zxdg_output_manager_v1,
            },
            wlr::layer_shell::v1::server::zwlr_layer_shell_v1,
            wp::{
                presentation_time::server::wp_presentation_time,
                viewporter::server::wp_viewport,
                fractional_scale::v1::server::wp_fractional_scale_manager_v1,
                single_pixel_buffer::v1::server::wp_single_pixel_buffer_manager_v1,
                relative_pointer::zv1::server::zwp_relative_pointer_manager_v1,
                pointer_constraints::zv1::server::zwp_pointer_constraints_v1,
            },
            unstable::{
                input_method::v2::server::zwp_input_method_manager_v2,
                text_input::v3::server::zwp_text_input_manager_v3,
                foreign_toplevel_management::v1::server::zwlr_foreign_toplevel_manager_v1,
                idle_notify::v1::server::zwp_idle_notifier_v1,
            },
            linux_dmabuf::zv1::server::zwp_linux_dmabuf_v1,
        }
    },
    utils::{Clock, Logical, Point, Rectangle, SERIAL_COUNTER, Serial, Transform, Size},
    wayland::{
        compositor::CompositorState, // Used in DesktopState
        data_device::DataDeviceState, // Used in DesktopState
        dmabuf::{DmabufGlobalData, DmabufClientData}, // For DMABUF global
        output::OutputManagerState, // Used in DesktopState
        seat::Seat, // For input
        shell::{
            xdg::XdgShellState, // Used in DesktopState
            wlr_layer::WlrLayerShellState, // Used in DesktopState
        },
        shm::ShmState, // Used in DesktopState
        socket::ListeningSocketSource,
        xdg_activation::XdgActivationState,
        presentation::PresentationState,
        fractional_scale::FractionalScaleManagerState,
        viewporter::ViewporterState,
        single_pixel_buffer::SinglePixelBufferState,
        relative_pointer::RelativePointerManagerState,
        pointer_constraints::PointerConstraintsState,
        input_method::InputMethodManagerState,
        text_input::TextInputManagerState,
        idle_notify::IdleNotifierState,
    },
    xwayland::{XWayland, XWaylandEvent}, // For XWayland
};

use crate::compositor::{
    state::DesktopState,
    // render::MainRenderer, // Will be used for initializing renderer
    // input::initialize_input_system, // Will be used for input setup
    // xwayland::initialize_xwayland, // Will be used for XWayland setup
    errors::CompositorError,
};

const SOCKET_NAME: &str = "novade-wayland-0";

pub fn run_compositor() -> Result<(), CompositorError> {
    info!("Starting NovaDE Wayland Compositor core...");

    let mut event_loop: EventLoop<'static, DesktopState> = EventLoop::try_new()
        .map_err(|e| CompositorError::EventLoopError(e.into()))?;
    let mut display: Display<DesktopState> = Display::new()
        .map_err(|e| CompositorError::DisplayError(e.to_string()))?;

    // Initialize DesktopState
    // This is a simplified initialization. Actual DesktopState::new will take more args.
    let mut desktop_state = DesktopState::new(&mut event_loop, &mut display);

    // Initialize Wayland globals
    initialize_globals(&mut display.handle(), &mut desktop_state)?;

    // Setup listening socket for Wayland clients
    let listening_socket = ListeningSocketSource::new_auto(desktop_state.clock.id())
        .map_err(|e| CompositorError::IoError(e.into()))?;
    let socket_name = listening_socket.socket_name().to_os_string();

    event_loop.handle().insert_source(listening_socket, move |client_stream, _, state: &mut DesktopState| {
        info!("New client connected");
        if let Err(e) = state.display_handle.insert_client(client_stream, Arc::new(ClientState)) {
            warn!("Error adding wayland client: {}", e);
        }
    }).map_err(|e| CompositorError::EventLoopError(e.into()))?;

    info!("Listening on Wayland socket: {:?}", socket_name);
    env::set_var("WAYLAND_DISPLAY", socket_name.to_string_lossy().as_ref());


    // --- Backend Initialization (Winit for now) ---
    info!("Attempting Winit backend initialization...");
    let (winit_event_loop, mut winit_data, winit_gles_renderer) =
        crate::compositor::backend::winit_backend::init_winit_backend(display.handle(), desktop_state.clock.id())?;

    // Store the GLES renderer from Winit into DesktopState's MainNovaRenderer
    let gles_nova_renderer = GlesNovaRenderer::new(winit_gles_renderer);
    desktop_state.main_renderer = Some(MainNovaRenderer::Gles(Box::new(gles_nova_renderer)));
    info!("GLES Renderer (from Winit) stored in DesktopState.");

    // Add the Winit output to DesktopState's output management
    desktop_state.output_manager_state.add_output(&winit_data.smithay_output);
    desktop_state.space.lock().unwrap().map_output(&winit_data.smithay_output, (0,0).into(), winit_data.smithay_output.current_mode().unwrap());
    info!("Winit output '{}' added to compositor state and space.", winit_data.smithay_output.name());

    // Store Winit event loop proxy for requesting redraws from other parts of the system
    desktop_state.winit_event_loop_proxy = Some(winit_data.event_loop_proxy.clone());


    // We need to move winit_graphics_backend into the Calloop closure,
    // or store it in DesktopState if it can be made 'static or if DesktopState is generic.
    // For now, Winit's event loop will be run directly, and Calloop dispatching will be manual within it.
    // This simplifies ownership for now but is not the ideal Smithay Calloop integration pattern.
    // The ideal pattern uses WinitEventLoop as a Calloop source.

    // TODO: Refactor to use WinitEventLoop as a Calloop source for cleaner integration.
    // This would require WinitGraphicsBackend to be Send or managed differently.
    // For this iteration, we run Winit's loop and manually pump Calloop.

    let mut winit_graphics_backend = match smithay_winit::init_renderer_window_from_raw_display_handle(
        winit_data.window.clone(), // Arc<WinitWindow>
        winit_data.window.raw_display_handle(), // Added this line
        winit_data.window.raw_window_handle()  // Added this line
    ) {
        Ok((backend, _renderer)) => backend, // We already have renderer, just need backend
        Err(e) => return Err(CompositorError::BackendCreation(format!("Failed to re-init Winit GL backend for event loop: {}", e))),
    };


    // TODO: Initialize input backend (e.g., libinput via udev) - Winit provides input for now.

    // Initialize XWayland if enabled
    // The actual check for whether it's enabled would ideally come from a config.
    // For now, spawn_xwayland_if_enabled has a hardcoded true for testing.
    if let Err(e) = crate::compositor::xwayland::spawn_xwayland_if_enabled(&mut desktop_state, &event_loop.handle(), &display.handle()) {
        error!("Failed to spawn XWayland: {}", e);
        // Depending on policy, compositor might continue or exit. For now, continue.
    }

    // Tokio runtime for async tasks (if any are dispatched to it by Calloop)
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| CompositorError::Internal(format!("Failed to create Tokio runtime: {}", e)))?;

    info!("NovaDE Compositor starting Winit event loop...");

    winit_event_loop.run(move |event, _, control_flow| {
        // Dispatch Calloop events first, non-blockingly
        let mut calloop_dispatcher = Dispatcher::new(&mut desktop_state, |_, _, _| PostAction::Continue);
        if let Err(e) = event_loop.dispatch(Some(Duration::ZERO), &mut calloop_dispatcher) {
            error!("Error during Calloop event loop dispatch: {}", e);
            *desktop_state.running.write().unwrap() = false;
        }

        if !*desktop_state.running.read().unwrap() {
            *control_flow = ControlFlow::Exit;
            return;
        }

        crate::compositor::backend::winit_backend::handle_winit_event(
            event,
            &mut desktop_state,
            &winit_data,
            &mut winit_graphics_backend,
            control_flow,
        );

        // After handling Winit event, dispatch Wayland clients and flush
        if let Err(e) = display.dispatch_clients(&mut desktop_state) {
            warn!("Error dispatching Wayland client events: {}", e);
        }
        if let Err(e) = display.flush_clients() {
            warn!("Error flushing Wayland clients: {}", e);
        }

        // Perform rendering if needed (e.g., if damage occurred or redraw requested)
        // This is a simplified render call. A real compositor would track damage.
        if control_flow != &ControlFlow::Exit && desktop_state.running.read().unwrap().clone() { // Check running again
            // Check if a redraw was requested by winit_backend::handle_winit_event via request_redraw
            // For now, assume redraw is needed if not exiting.
            // This is a placeholder for proper damage tracking and redraw scheduling.

            // Actual rendering logic:
            // 1. Access MainNovaRenderer from desktop_state.
            // 2. Get output details (size, scale, transform) from winit_data.smithay_output.
            // 3. Get elements from desktop_state.space for this output.
            // 4. Call renderer.render_output_frame(...).
            // 5. Call winit_graphics_backend.submit(None) to present.
            // This is complex and will be built out in the rendering step.
            // For now, just a log message.
            // debug!("Winit loop: Placeholder for rendering call.");

            // Actual rendering logic for Winit backend
            if let smithay::reexports::winit::event::Event::RedrawRequested(_) = event {
                if let Some(main_renderer) = desktop_state.main_renderer.as_mut() {
                    if let MainNovaRenderer::Gles(gles_renderer_wrapper) = main_renderer {
                        let output = &winit_data.smithay_output;
                        let renderer_node = &winit_data.renderer_node; // This is the Winit window's node

                        let mut space_lock = desktop_state.space.lock().unwrap();
                        let mut damage_tracker = smithay::backend::renderer::damage::OutputDamageTracker::new_for_output(output); // Recreate for now, should be stored

                        // Gather render elements
                        let mut render_elements: Vec<smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement<Gles2Renderer>> = Vec::new();
                        let mut surfaces_for_callback: Vec<wl_surface::WlSurface> = Vec::new();

                        // Iterate over windows in space, filter for current output
                        for window_element in space_lock.elements_for_output(output).unwrap_or_default() {
                            if !window_element.is_mapped() { continue; } // Skip unmapped

                            if let Some(surface) = window_element.wl_surface() {
                                if let Some(surface_attributes) = surface.data::<smithay::wayland::compositor::SurfaceAttributes>() {
                                    if let Some(buffer) = surface_attributes.buffer.as_ref() {
                                        // Texture caching logic would go here. For now, always import.
                                        match gles_renderer_wrapper.import_shm_buffer(buffer, Some(surface_attributes), &[]) {
                                            Ok(texture) => {
                                                // Attempt to create the render element
                                                match smithay::backend::renderer::element::surface::WaylandSurfaceRenderElement::from_space_view(
                                                    &mut gles_renderer_wrapper.inner,
                                                    window_element,
                                                    surface_attributes,
                                                    &texture,
                                                    output,
                                                    &space_lock,
                                                ) {
                                                    Ok(element) => {
                                                        render_elements.push(element);
                                                        surfaces_for_callback.push(surface.clone()); // Clone WlSurface for callback
                                                    },
                                                    Err(e) => {
                                                        warn!("Failed to create render element for surface {:?}: {}", surface.id(), e);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                warn!("Failed to import SHM buffer for surface {:?}: {}", surface.id(), e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        drop(space_lock); // Release lock before rendering

                        // Bind the graphics backend for rendering
                        if let Err(e) = winit_graphics_backend.bind() {
                            error!("Failed to bind Winit graphics backend: {}", e);
                            *control_flow = ControlFlow::Exit;
                            return;
                        }

                        let render_result = damage_tracker.render_output(
                            &mut gles_renderer_wrapper.inner,
                            renderer_node,
                            0, // age
                            output.current_mode().unwrap().size,
                            output.current_scale(),
                            output.current_transform(),
                            &render_elements[..], // Pass as slice
                            [0.1, 0.1, 0.3, 1.0], // Clear color: dark blue
                        );

                        match render_result {
                            Ok(render_damage) => {
                                if let Err(e) = winit_graphics_backend.submit(render_damage.as_ref().map(|v| &v[..])) {
                                    error!("Winit graphics backend submit failed: {}", e);
                                } else {
                                    debug!("Winit frame submitted with damage: {:?}", render_damage);
                                    // Send frame callbacks
                                    let time = desktop_state.clock.now();
                                    for elem in render_elements {
                                        if let Some(surface) = elem.wl_surface() {
                                            surface.send_frame_done(time);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("GLES rendering failed: {}", e);
                            }
                        }
                    }
                }
            }
        }


        if !*desktop_state.running.read().unwrap() {
            *control_flow = ControlFlow::Exit;
        }
    }); // winit_event_loop.run consumes the loop and blocks until exit.

    info!("NovaDE Compositor Winit event loop finished.");
    // TODO: Cleanup resources (XWayland, backend resources, etc.)
    Ok(())
}

/// Initializes all required Wayland globals.
fn initialize_globals(display_handle: &mut DisplayHandle, state: &mut DesktopState) -> Result<(), CompositorError> {
    info!("Initializing Wayland globals...");

    state.compositor_global = Some(display_handle.create_global::<DesktopState, wl_compositor::WlCompositor, _>(5, state.compositor_state.clone()));
    state.subcompositor_global = Some(display_handle.create_global::<DesktopState, wl_subcompositor::WlSubcompositor, _>(1, state.subcompositor_state.clone()));
    state.shm_global = Some(display_handle.create_global::<DesktopState, wl_shm::WlShm, ShmClientData>(1, state.shm_state.clone()));
    state.data_device_global = Some(display_handle.create_global::<DesktopState, wl_data_device_manager::WlDataDeviceManager, DataDeviceUserData>(3, state.data_device_state.clone()));

    // DMABUF global (requires DmabufGlobalData)
    // let dmabuf_formats = ...; // Get supported DMABUF formats from renderer/DRM
    // state.dmabuf_global = Some(display_handle.create_global::<DesktopState, zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1, DmabufGlobalData>(
    //     4, DmabufGlobalData { formats: dmabuf_formats, state: state.dmabuf_state.clone() }
    // ));
    warn!("DMABUF global initialization placeholder - requires format list.");


    state.xdg_shell_global = Some(display_handle.create_global::<DesktopState, xdg_wm_base::XdgWmBase, _>(3, state.xdg_shell_state.clone()));
    state.layer_shell_global = Some(display_handle.create_global::<DesktopState, zwlr_layer_shell_v1::ZwlrLayerShellV1, _>(4, state.layer_shell_state.clone()));
    state.xdg_decoration_global = Some(display_handle.create_global::<DesktopState, zxdg_decoration_manager_v1::ZxdgDecorationManagerV1, _>(1, state.xdg_decoration_state.clone()));
    state.xdg_activation_global = Some(display_handle.create_global::<DesktopState, xdg_activation_v1::XdgActivationV1, _>(1, state.xdg_activation_state.clone()));
    state.presentation_global = Some(display_handle.create_global::<DesktopState, wp_presentation_time::WpPresentationTime, _>(1, state.presentation_state.clone()));
    state.fractional_scale_manager_global = Some(display_handle.create_global::<DesktopState, wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1, _>(1, state.fractional_scale_manager_state.clone()));
    state.viewporter_global = Some(display_handle.create_global::<DesktopState, wp_viewport::WpViewport, _>(1, state.viewporter_state.clone()));

    // zxdg_output_manager_v1 uses the existing OutputManagerState
    state.xdg_output_manager_global = Some(display_handle.create_global::<DesktopState, zxdg_output_manager_v1::ZxdgOutputManagerV1, _>(3, state.xdg_output_manager_state.clone()));

    state.single_pixel_buffer_global = Some(display_handle.create_global::<DesktopState, wp_single_pixel_buffer_manager_v1::WpSinglePixelBufferManagerV1, _>(1, state.single_pixel_buffer_state.clone()));
    state.relative_pointer_manager_global = Some(display_handle.create_global::<DesktopState, zwp_relative_pointer_manager_v1::ZwpRelativePointerManagerV1, _>(1, state.relative_pointer_manager_state.clone()));
    state.pointer_constraints_global = Some(display_handle.create_global::<DesktopState, zwp_pointer_constraints_v1::ZwpPointerConstraintsV1, _>(1, state.pointer_constraints_state.clone()));

    // state.foreign_toplevel_manager_global = Some(display_handle.create_global::<DesktopState, zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, _>(1, state.foreign_toplevel_manager_state.clone())); // Requires state struct
    warn!("Foreign Toplevel Manager global initialization placeholder - requires state struct.");

    state.idle_notifier_global = Some(display_handle.create_global::<DesktopState, zwp_idle_notifier_v1::ZwpIdleNotifierV1, _>(1, state.idle_notifier_state.clone()));
    state.input_method_manager_global = Some(display_handle.create_global::<DesktopState, zwp_input_method_manager_v2::ZwpInputMethodManagerV2, _>(1, state.input_method_manager_state.clone()));
    state.text_input_manager_global = Some(display_handle.create_global::<DesktopState, zwp_text_input_manager_v3::ZwpTextInputManagerV3, _>(1, state.text_input_manager_state.clone()));

    info!("Wayland globals initialized.");
    Ok(())
}


// Client state, can be expanded if per-client data is needed beyond what Smithay provides.
pub struct ClientState;
impl wayland_server::backend::ClientData for ClientState {
    fn initialized(&self, _client_id: wayland_server::backend::ClientId) {
        debug!("Client initialized: {:?}", _client_id);
    }
    fn disconnected(&self, client_id: wayland_server::backend::ClientId, reason: DisconnectReason) {
        info!("Client disconnected: {:?}, reason: {:?}", client_id, reason);
        // TODO: Cleanup client resources (windows, etc.) from DesktopState.
        // This needs access to DesktopState, which is tricky here.
        // Usually done by DesktopState implementing a handler trait called by Smithay.
        // Or, the main loop iterates clients and checks for disconnections.
        // Smithay's Display::dispatch_clients should handle invoking the relevant disconnect handlers on DesktopState.
    }
}
