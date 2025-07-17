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


    // TODO: Initialize input backend (e.g., libinput via udev)

    // Initialize XWayland if enabled
    // The actual check for whether it's enabled would ideally come from a config.
    if let Err(e) = crate::compositor::xwayland::spawn_xwayland_if_enabled(&mut desktop_state, &event_loop.handle(), &display.handle()) {
        error!("Failed to spawn XWayland: {}", e);
        // Depending on policy, compositor might continue or exit. For now, continue.
    }

    info!("NovaDE Compositor starting main event loop...");

    // This is a placeholder for the main DRM/libseat event loop.
    // A real implementation would now integrate sources for DRM, libinput, etc.
    // into the main `event_loop` and then call `event_loop.run(...)`.

    let mut damage_tracker = DamageTrackerState::new();

    loop {
        // Dispatch events
        let mut calloop_dispatcher = Dispatcher::new(&mut desktop_state, |_, _, _| PostAction::Continue);
        event_loop.dispatch(Some(Duration::from_millis(16)), &mut calloop_dispatcher)
            .map_err(|e| CompositorError::EventLoopError(e.into()))?;

        // Process signals and other events from the dispatcher's state
        // (This is a simplified representation of how state changes might be handled)

        // Render frame
        if let Some(renderer) = desktop_state.renderer.as_mut() {
            let mut space = desktop_state.space.lock().unwrap();
            for output in space.outputs() {
                let (elements, damage) = damage_tracker.render_elements_for_output(renderer, output, &space);
                let output_geometry = output.current_geometry().unwrap();
                let output_scale = output.current_scale().fractional_scale();

                renderer.render_frame(elements, output_geometry, output_scale)
                    .map_err(|e| CompositorError::RenderingError(e.to_string()))?;
            }
            renderer.submit_and_present_frame()
                .map_err(|e| CompositorError::RenderingError(e.to_string()))?;

            space.send_frames(desktop_state.clock.now());
        }

        // Flush clients
        if let Err(e) = display.flush_clients() {
            warn!("Error flushing Wayland clients: {}", e);
        }

        if !*desktop_state.running.read().unwrap() {
            break;
        }
    }

    info!("NovaDE Compositor event loop finished.");
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
