// This is novade-system/src/compositor/state.rs
// Defines CompositorStateGlobal (as DesktopState) and other state-related structs.

use std::{
    ffi::OsString,
    sync::{Arc, Mutex as StdMutex, RwLock}, // Using std::sync::Mutex as StdMutex
    time::Duration,
};
use tracing::{debug, error, info, info_span, warn};

use smithay::{
    backend::renderer::{
        gles2::Gles2Renderer, // For GLES2 rendering state if needed directly
        // Import Vulkan related types if DesktopState directly manages parts of Vulkan renderer
    },
    desktop::{Space, Window, PopupManager, LayerSurface, WindowSurfaceType},
    input::{
        Seat, SeatState, SeatHandler, // SeatState is Smithay's manager for seats
        pointer::{CursorImageStatus, GrabStartData as PointerGrabStartData, PointerAxisEventData, PointerButtonEventData, PointerMotionEventData, RelativeMotionEvent},
        keyboard::{KeyboardHandle, ModifiersState, XkbConfig, Keysym, FilterResult as XkbFilterResult},
        touch::TouchEventData,
        SeatGrab, // For grab_initiated/ended
        SeatFocus, // For focus_changed on SeatHandler
    },
    output::{Output as SmithayOutput, OutputHandler, OutputState as SmithayOutputState}, // Renamed Smithay's Output
    reexports::{
        calloop::{EventLoop, LoopHandle, generic::Generic, Interest, Mode, PostAction},
        wayland_server::{
            backend::{ClientId, DisconnectReason, GlobalId},
            protocol::{
                wl_output, wl_surface::WlSurface, wl_seat, wl_buffer::WlBuffer,
                wl_data_device_manager, wl_compositor, wl_subcompositor, wl_shm,
            },
            Display, DisplayHandle, Client, ListeningSocket, GlobalDispatch, Dispatch, New,
        },
        wayland_protocols::{
            xdg::{
                shell::server::{xdg_wm_base, xdg_surface, xdg_toplevel, xdg_popup},
                decoration::zv1::server::zxdg_decoration_manager_v1,
                activation::v1::server::xdg_activation_v1,
                output::zv1::server::zxdg_output_manager_v1,
            },
            wlr::layer_shell::v1::server::{zwlr_layer_shell_v1, zwlr_layer_surface_v1},
            wp::{
                presentation_time::server::wp_presentation_time,
                viewporter::server::wp_viewport,
                fractional_scale::v1::server::wp_fractional_scale_manager_v1,
                single_pixel_buffer::v1::server::wp_single_pixel_buffer_manager_v1,
                relative_pointer::zv1::server::zwp_relative_pointer_manager_v1,
                pointer_constraints::zv1::server::zwp_pointer_constraints_v1, // Also for locked pointer
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
    utils::{Clock, Logical, Point, Rectangle, SERIAL_COUNTER, Serial, Transform, Physical, Size},
    wayland::{
        compositor::{CompositorState, CompositorHandler, CompositorClientState, SurfaceAttributes as WlSurfaceAttributes, SubsurfaceCachedState, TraversalAction},
        data_device::{DataDeviceState, DataDeviceHandler, ServerDndGrabHandler, ClientDndGrabHandler, DataDeviceUserData},
        dmabuf::{DmabufState, DmabufHandler, DmabufGlobalData, DmabufClientData, DmabufData},
        output::{OutputManagerState, OutputData, XdgOutputUserData}, // Smithay's output state
        subcompositor::{SubcompositorState, SubcompositorHandler},
        shell::{
            xdg::{XdgShellState, XdgShellHandler, XdgSurfaceUserData, XdgPopupUserData, XdgToplevelUserData, XdgPositionerUserData},
            wlr_layer::{WlrLayerShellState, WlrLayerShellHandler, LayerSurfaceData}, // Smithay 0.30 uses this
        },
        shm::{ShmState, ShmHandler, ShmClientData},
        seat::SeatState as SmithaySeatState, // The manager for seats
        socket::ListeningSocketSource,
        selection::SelectionHandler, // For wl_data_device clipboard/dnd
        xdg_activation::XdgActivationState, // For xdg_activation_v1
        presentation::PresentationState,    // For wp_presentation_time
        fractional_scale::FractionalScaleManagerState, // For wp_fractional_scale_v1
        viewporter::ViewporterState,        // For wp_viewport
        relative_pointer::RelativePointerManagerState,
        pointer_constraints::{PointerConstraintsState, PointerConstraint, LockedPointerData, ConfinedPointerData},
        input_method::InputMethodManagerState,
        text_input::TextInputManagerState,
        idle_notify::IdleNotifierState,
        // Foreign toplevel state might be custom or from a helper crate if Smithay doesn't have one directly
    },
    xwayland::{XWayland, XWaylandEvent, XWaylandClientData, XWaylandSurface, Xwm, XWaylandConnection},
    signaling::SignalToken, // For Calloop signals
};

// NovaDE specific imports (assuming these exist or will be created)
// use crate::compositor::render::MainRenderer; // To select active renderer
// use crate::compositor::config::CompositorConfig;
// use novade_core::settings::SettingsManager; // Example if settings are used

// For zxdg_decoration_manager_v1
use smithay::wayland::shell::xdg::decoration::XdgDecorationState; // Smithay 0.30.0

// For wp_single_pixel_buffer_v1
use smithay::wayland::single_pixel_buffer::SinglePixelBufferState;

// Define a wrapper for the main Smithay SeatState to manage multiple seats if ever needed.
// For now, NovaDE might only use one seat.
pub struct NovaSeatState {
    pub inner: SmithaySeatState<DesktopState>, // Smithay's SeatState, generic over DesktopState
}

impl NovaSeatState {
    pub fn new() -> Self {
        Self { inner: SmithaySeatState::new() }
    }
}

/// The main state object for the NovaDE Wayland compositor.
/// This struct will be the `Data` type for the Calloop event loop.
pub struct DesktopState {
    pub display_handle: DisplayHandle, // Handle to the Wayland display
    pub event_loop_handle: LoopHandle<'static, Self>, // Handle to the Calloop event loop
    pub running: Arc<RwLock<bool>>, // To signal the event loop to stop

    // Core Wayland protocol states
    pub compositor_state: CompositorState,
    pub subcompositor_state: SubcompositorState,
    pub shm_state: ShmState,
    pub data_device_state: DataDeviceState,
    pub dmabuf_state: DmabufState, // For zwp_linux_dmabuf_v1

    // Shell protocol states
    pub xdg_shell_state: XdgShellState,
    pub layer_shell_state: WlrLayerShellState, // Smithay 0.30.0 state

    // Output and rendering
    pub output_manager_state: OutputManagerState,
    // pub main_renderer: Option<MainRenderer>, // To hold the active renderer (GLES or Vulkan)
    pub space: Arc<StdMutex<Space<Window>>>, // Manages layout of windows and outputs
    pub popups: Arc<StdMutex<PopupManager>>,  // Manages popup surfaces

    // Input
    pub seat_state: NovaSeatState, // Wrapper for Smithay's SeatState
    pub primary_seat: Seat<Self>, // The main/default seat
    pub pointer_location: Point<f64, Logical>, // Last known pointer location
    pub cursor_status: Arc<StdMutex<CursorImageStatus>>, // Current cursor image status
    // pub input_config: InputConfig, // NovaDE specific input configuration

    // XWayland
    pub xwayland_connection: Option<Arc<XWaylandConnection>>, // Connection to XWayland server
    // pub xwayland_guard: Option<XWaylandGuard>, // To manage XWayland lifecycle (if using older pattern)

    // Other Wayland protocol states
    pub xdg_decoration_state: XdgDecorationState, // Smithay 0.30.0 state
    pub xdg_activation_state: XdgActivationState,
    pub presentation_state: PresentationState,
    pub fractional_scale_manager_state: FractionalScaleManagerState,
    pub viewporter_state: ViewporterState,
    pub xdg_output_manager_state: OutputManagerState, // zxdg_output_manager_v1 uses OutputManagerState too
    pub single_pixel_buffer_state: SinglePixelBufferState,
    pub relative_pointer_manager_state: RelativePointerManagerState,
    pub pointer_constraints_state: PointerConstraintsState,
    // pub foreign_toplevel_manager_state: ForeignToplevelManagerState, // Needs a struct
    pub idle_notifier_state: IdleNotifierState,
    pub input_method_manager_state: InputMethodManagerState,
    pub text_input_manager_state: TextInputManagerState,


    // Timing and damage tracking
    pub clock: Clock, // For timing events and animations
    // pub output_damage_tracker: HashMap<OutputId, DamageTracker>, // Track damage per output

    // NovaDE specific state
    // pub config: Arc<SettingsManager<CompositorConfig>>,
    // pub nova_workspace_manager: Arc<StdMutex<NovaWorkspaceManager>>, // NovaDE's workspace logic

    // Winit backend specific data (if Winit backend is active)
    // These fields would be populated by init_winit_backend()
    // For now, they are not generic over WinitWindow type to avoid making DesktopState generic yet.
    // Consider a BackendState enum later if supporting multiple backends dynamically.
    #[cfg(feature = "backend_winit")]
    pub winit_event_loop_proxy: Option<smithay::reexports::winit::event_loop::EventLoopProxy<()>>, // To request redraws etc.
    // pub winit_window: Option<Arc<smithay::reexports::winit::window::Window>>, // Window needs to be Arc for some WinitGraphicsBackend
    // pub winit_graphics_backend: Option<Box<dyn smithay::backend::winit::WinitGraphicsBackend<Renderer = smithay::backend::renderer::gles2::Gles2Renderer>>>,
    // pub winit_renderer_node: Option<smithay::backend::renderer::RendererNode>,

    // XWayland related state
    pub xwayland_guard: Option<XWayland<DesktopState>>, // Guard to keep XWayland alive


    // Globals that have been created
    pub compositor_global: Option<GlobalId>,
    pub subcompositor_global: Option<GlobalId>,
    pub shm_global: Option<GlobalId>,
    pub data_device_global: Option<GlobalId>,
    pub dmabuf_global: Option<GlobalId>,
    pub xdg_shell_global: Option<GlobalId>,
    pub layer_shell_global: Option<GlobalId>,
    pub xdg_decoration_global: Option<GlobalId>,
    pub xdg_activation_global: Option<GlobalId>,
    pub presentation_global: Option<GlobalId>,
    pub fractional_scale_manager_global: Option<GlobalId>,
    pub viewporter_global: Option<GlobalId>,
    pub xdg_output_manager_global: Option<GlobalId>,
    pub single_pixel_buffer_global: Option<GlobalId>,
    pub relative_pointer_manager_global: Option<GlobalId>,
    pub pointer_constraints_global: Option<GlobalId>,
    // pub foreign_toplevel_manager_global: Option<GlobalId>,
    pub idle_notifier_global: Option<GlobalId>,
    pub input_method_manager_global: Option<GlobalId>,
    pub text_input_manager_global: Option<GlobalId>,
    // ... other global IDs ...
}

impl DesktopState {
    pub fn new(
        event_loop: &mut EventLoop<'static, Self>,
        display: &mut Display<Self>,
        // config: Arc<SettingsManager<CompositorConfig>>, // NovaDE config
    ) -> Self {
        let display_handle = display.handle();
        let event_loop_handle = event_loop.handle();

        info!("Initializing NovaDE DesktopState...");

        let clock = Clock::new();

        // Core Wayland states
        let compositor_state = CompositorState::new::<Self>(&display_handle, clock.id());
        let subcompositor_state = SubcompositorState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![], clock.id()); // SHM formats can be added later or taken from config
        let data_device_state = DataDeviceState::new::<Self>(&display_handle, clock.id());
        let dmabuf_state = DmabufState::new(); // DmabufState is simple, global data is separate

        // Shell states
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle, clock.id());
        let layer_shell_state = WlrLayerShellState::new::<Self>(&display_handle, clock.id());

        // Output and rendering related states
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let space = Arc::new(StdMutex::new(Space::new(clock.id())));
        let popups = Arc::new(StdMutex::new(PopupManager::new(clock.id())));

        // Input states
        let mut seat_state_manager = NovaSeatState::new();
        let primary_seat = seat_state_manager.inner.new_wl_seat(&display_handle, "seat0".to_string(), clock.id());
        let cursor_status = Arc::new(StdMutex::new(CursorImageStatus::Default));

        // Other protocol states
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle, clock.id());
        let xdg_activation_state = XdgActivationState::new::<Self>(&display_handle, clock.id());
        let presentation_state = PresentationState::new::<Self>(&display_handle, clock.id());
        let fractional_scale_manager_state = FractionalScaleManagerState::new::<Self>(&display_handle, clock.id());
        let viewporter_state = ViewporterState::new::<Self>(&display_handle, clock.id());
        // xdg_output_manager_state is the same as output_manager_state for XDG Output integration.
        let single_pixel_buffer_state = SinglePixelBufferState::new::<Self>(&display_handle, clock.id());
        let relative_pointer_manager_state = RelativePointerManagerState::new::<Self>(&display_handle, clock.id());
        let pointer_constraints_state = PointerConstraintsState::new::<Self>(&display_handle, clock.id());
        // let foreign_toplevel_manager_state = ForeignToplevelManagerState::new(); // Needs custom struct
        let idle_notifier_state = IdleNotifierState::new::<Self>(&display_handle, clock.id());
        let input_method_manager_state = InputMethodManagerState::new::<Self>(&display_handle, clock.id());
        let text_input_manager_state = TextInputManagerState::new::<Self>(&display_handle, clock.id());


        Self {
            display_handle,
            event_loop_handle,
            running: Arc::new(RwLock::new(true)),
            compositor_state,
            subcompositor_state,
            shm_state,
            data_device_state,
            dmabuf_state,
            xdg_shell_state,
            layer_shell_state,
            output_manager_state: output_manager_state.clone(), // Clone for xdg_output_manager_state
            space,
            popups,
            seat_state: seat_state_manager,
            primary_seat,
            pointer_location: (0.0, 0.0).into(),
            cursor_status,
            xwayland_connection: None,
            xdg_decoration_state,
            xdg_activation_state,
            presentation_state,
            fractional_scale_manager_state,
            viewporter_state,
            xdg_output_manager_state: output_manager_state, // Use the same state
            single_pixel_buffer_state,
            relative_pointer_manager_state,
            pointer_constraints_state,
            idle_notifier_state,
            input_method_manager_state,
            text_input_manager_state,
            clock,
            // config,
            compositor_global: None, // Globals will be initialized later
            subcompositor_global: None,
            shm_global: None,
            data_device_global: None,
            dmabuf_global: None,
            xdg_shell_global: None,
            layer_shell_global: None,
            xdg_decoration_global: None,
            xdg_activation_global: None,
            presentation_global: None,
            fractional_scale_manager_global: None,
            viewporter_global: None,
            xdg_output_manager_global: None,
            single_pixel_buffer_global: None,
            relative_pointer_manager_global: None,
            pointer_constraints_global: None,
            idle_notifier_global: None,
            input_method_manager_global: None,
            text_input_manager_global: None,
        }
    }

    // TODO: Add methods for managing workspaces, focus, input event routing, etc.
    // These will interact with the Smithay states and NovaDE's domain logic.
}


// --- Old NovadeCompositorState (to be removed or merged) ---
// This section appears to be from an older iteration and should be
// carefully reviewed, merged into DesktopState if relevant, or removed.
// For now, it's commented out to focus on the new DesktopState structure.
/*
use smithay::{
    backend::egl::Egl,
    wayland::{
        dmabuf::DmabufState as OldDmabufState, // Avoid conflict if names are same
    },
    backend::{
        drm::{DrmNode as DrmNodeSmithay},
        session::Session,
    },
};
use std::{cell::RefCell, collections::HashMap, time::SystemTime};
use smithay::wayland::compositor as smithay_compositor_old; // Alias to avoid conflict

use crate::compositor::render::gl::{init_gl_renderer, GlInitError}; // Assuming this is for GLES

pub trait RenderableTextureOld: std::fmt::Debug + Send + Sync {}
pub trait FrameRendererOld: Send + Sync {
    // ... methods ...
}
#[derive(Debug, thiserror::Error)]
pub enum RendererErrorOld { /* ... variants ... */ }
use uuid::Uuid;
pub trait RenderableTextureUuid: std::fmt::Debug + Send + Sync + std::any::Any { /* ... methods ... */ }
#[derive(Debug)]
pub enum RenderElementOld<'a> { /* ... variants ... */ }
#[derive(Default, Debug)]
pub struct SurfaceDataExtOld { /* ... fields ... */ }

pub struct NovadeCompositorState {
    pub display_handle: Display<Self>, // This Self refers to NovadeCompositorState, problematic
    pub loop_handle: LoopHandle<'static, Self>, // Same Self issue
    // ... many other fields similar to DesktopState but potentially with older patterns ...
    pub gl_renderer: Gles2Renderer, // Direct GLES renderer, DesktopState might use MainRenderer abstraction
    // ...
}

impl NovadeCompositorState {
    // pub fn new(...) -> Result<Self, GlInitError> { /* ... old constructor ... */ }
    // pub fn render_frame(...) -> Result<(), String> { /* ... old render logic ... */ }
}
*/
