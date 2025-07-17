// This is novade-system/src/compositor/state.rs
// Defines the canonical DesktopState and other state-related structs.

//! Holds the main state of the NovaDE Wayland compositor, `DesktopState`.
//!
//! `DesktopState` aggregates all other state objects (like those for Wayland protocols,
//! input, output, rendering, and window management) and serves as the central data
//! structure for the Calloop event loop.

use std::{
    collections::HashMap,
    ffi::OsString,
    sync::{Arc, Mutex as StdMutex, RwLock},
    time::{Duration, Instant},
};
use tracing::{debug, error, info, info_span, warn};
use uuid::Uuid;

use smithay::{
    backend::renderer::{
        gles2::Gles2Renderer,
        // Import renderer abstractions
        // renderer::{Frame, Renderer, Texture, TextureFilter},
    },
    desktop::{Space, Window, PopupManager, LayerSurface, WindowSurfaceType, DamageTrackerState},
    input::{
        Seat, SeatState as SmithaySeatState, SeatHandler,
        pointer::{CursorImageStatus, GrabStartData as PointerGrabStartData, PointerAxisEventData, PointerButtonEventData, PointerMotionEventData, RelativeMotionEvent},
        keyboard::{KeyboardHandle, ModifiersState, XkbConfig, Keysym, FilterResult as XkbFilterResult, KeysymHandle as SmithayKeysymHandle},
        touch::{TouchEventData, TouchSlotId},
        SeatGrab,
        SeatFocus,
    },
    output::{Output as SmithayOutput, OutputHandler, OutputState as SmithayOutputState},
    reexports::{
        calloop::{EventLoop, LoopHandle, generic::Generic, Interest, Mode, PostAction, TimerHandle},
        wayland_server::{
            backend::{ClientId, DisconnectReason, GlobalId},
            protocol::{
                wl_output, wl_surface::{self, WlSurface, Weak as WlWeakSurface}, wl_seat, wl_buffer::WlBuffer,
                wl_data_device_manager, wl_compositor, wl_subcompositor, wl_shm,
            },
            Display, DisplayHandle, Client, ListeningSocket, GlobalDispatch, Dispatch, New, DataInit,
        },
        wayland_protocols::{
            xdg::{
                shell::server::{
                    xdg_wm_base::{self, XdgWmBase},
                    xdg_surface,
                    xdg_toplevel::{self, XdgToplevel, State as XdgToplevelStateRead, ResizeEdge},
                    xdg_popup,
                },
                decoration::zv1::server::zxdg_toplevel_decoration_v1,
                activation::v1::server::xdg_activation_v1,
                output::zv1::server::zxdg_output_manager_v1,
            },
            wlr::layer_shell::v1::server::{zwlr_layer_shell_v1, zwlr_layer_surface_v1::{self, Layer}},
            wp::{
                presentation_time::server::wp_presentation_time,
                viewporter::server::wp_viewport,
                fractional_scale::v1::server::wp_fractional_scale_manager_v1,
                single_pixel_buffer::v1::server::wp_single_pixel_buffer_manager_v1,
                relative_pointer::zv1::server::zwp_relative_pointer_manager_v1,
                pointer_constraints::zv1::server::zwp_pointer_constraints_v1,
                screencopy::v1::server::zwp_screencopy_manager_v1,
            },
            unstable::{
                input_method::v2::server::zwp_input_method_manager_v2,
                text_input::v3::server::zwp_text_input_manager_v3,
                foreign_toplevel_management::v1::server::zwlr_foreign_toplevel_manager_v1::{ZwlrForeignToplevelManagerV1},
                idle_notify::v1::server::zwp_idle_notifier_v1,
            },
            linux_dmabuf::zv1::server::zwp_linux_dmabuf_v1,
        }
    },
    utils::{Clock, Logical, Point, Rectangle, SERIAL_COUNTER, Serial, Transform, Physical, Size, Buffer},
    wayland::{
        compositor::{CompositorState, CompositorHandler, CompositorClientState, SurfaceAttributes as WlSurfaceAttributes, SubsurfaceCachedState, TraversalAction, SurfaceData as SmithaySurfaceData, add_destruction_hook},
        data_device::{DataDeviceState, DataDeviceHandler, ServerDndGrabHandler, ClientDndGrabHandler, DataDeviceUserData, SelectionSource, SelectionTarget},
        dmabuf::{DmabufState, DmabufHandler, DmabufGlobalData, DmabufClientData, DmabufData, DmabufGlobal, ImportNotifier},
        output::{OutputManagerState, OutputData, XdgOutputUserData},
        subcompositor::{SubcompositorState, SubcompositorHandler},
        shell::{
            xdg::{XdgShellState, XdgShellHandler, XdgSurfaceUserData, XdgPopupUserData, XdgToplevelUserData, XdgPositionerUserData, Configure, XdgWmBaseClientData},
            wlr_layer::{WlrLayerShellState, WlrLayerShellHandler, LayerSurfaceData},
            xdg::decoration::{XdgDecorationState, XdgDecorationHandler, Mode as XdgToplevelDecorationMode, ZxdgToplevelDecorationV1},
        },
        shm::{ShmState, ShmHandler, ShmClientData, BufferHandler},
        socket::ListeningSocketSource,
        selection::SelectionHandler,
        xdg_activation::{XdgActivationState, XdgActivationHandler, XdgActivationTokenData, XdgActivationTokenSurfaceData, XdgActivationTokenV1},
        presentation::{PresentationState, PresentationHandler, PresentationFeedbackData, PresentationTimes, PresentationFeedbackFlags},
        fractional_scale::{FractionalScaleManagerState, FractionalScaleHandler, FractionalScaleManagerUserData, PreferredScale, Scale},
        viewporter::{ViewporterState, ViewporterHandler},
        single_pixel_buffer::{SinglePixelBufferState, SinglePixelBufferHandler},
        relative_pointer::{RelativePointerManagerState, RelativePointerManagerHandler},
        pointer_constraints::{PointerConstraintsState, PointerConstraintsHandler, PointerConstraintData, PointerConstraint, LockedPointerData, ConfinedPointerData},
        input_method::{InputMethodManagerState, InputMethodHandler, InputMethodKeyboardGrabCreator, InputMethodPopupSurfaceCreator, InputMethodSeatUserData, ZwpInputMethodV2, ZwpInputMethodKeyboardGrabV2, ZwpInputMethodPopupSurfaceV2},
        text_input::{TextInputManagerState, TextInputHandler, TextInputSeatUserData, ZwpTextInputV3, ContentHint, ContentPurpose},
        idle_notify::{IdleNotifierState, IdleNotifierHandler, IdleNotifySeatUserData, ZwpIdleInhibitorV1},
        primary_selection::{PrimarySelectionHandler, PrimarySelectionTarget, PrimarySelectionSource},
        screencopy::ScreencopyState,
    },
    xwayland::{XWayland, XWaylandEvent, XWaylandClientData, XWaylandSurface, Xwm, XWaylandConnection},
    signaling::SignalToken,
};

use crate::compositor::foreign_toplevel::ForeignToplevelManagerState;
use crate::compositor::renderer_interface::abstraction::{FrameRenderer, RenderableTexture};
use crate::compositor::shell::xdg_shell::types::{DomainWindowIdentifier, ManagedWindow};
use crate::compositor::workspaces::{CompositorWorkspace, TilingLayout};
use crate::compositor::outputs::OutputConfig;
use crate::input::keyboard::xkb_config::XkbKeyboardData;
use crate::input::input_dispatcher::InputDispatcher;
use crate::input::keyboard_layout::KeyboardLayoutManager;
use crate::error::SystemResult;
use crate::compositor::renderers::{
    gles2::Gles2Renderer,
    vulkan::frame_renderer::FrameRenderer as VulkanFrameRenderer,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererType {
    Gles2,
    Vulkan,
}

// --- Client Data Structs ---

/// Client data attached to each Wayland client.
#[derive(Debug, Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
    // Add other client-specific states here if needed
}

/// Wrapper around Smithay's `SeatState` to add NovaDE-specific seat logic.
pub struct NovaSeatState {
    pub inner: SmithaySeatState<DesktopState>,
}

impl NovaSeatState {
    pub fn new() -> Self {
        Self { inner: SmithaySeatState::new() }
    }
}

// --- Grab and Interaction State Structs ---

#[derive(Debug, Clone)]
pub struct MoveGrabState {
    pub window: Arc<ManagedWindow>,
    pub initial_pointer_pos_logical: Point<f64, Logical>,
    pub initial_window_pos_logical: Point<i32, Logical>,
}

#[derive(Debug, Clone)]
pub struct ActiveResizeGrabState {
    pub window: Arc<ManagedWindow>,
    pub edge: ResizeEdge,
    pub initial_pointer_pos_logical: Point<f64, Logical>,
    pub initial_window_geometry: Rectangle<i32, Logical>,
}

// --- Main DesktopState Struct ---

/// The canonical state object for the NovaDE Wayland compositor.
pub struct DesktopState {
    // --- Core Handles & Control ---
    pub display_handle: DisplayHandle,
    pub event_loop_handle: LoopHandle<'static, Self>,
    pub running: Arc<RwLock<bool>>,
    pub clock: Clock,

    // --- Smithay Protocol States ---
    pub compositor_state: CompositorState,
    pub subcompositor_state: SubcompositorState,
    pub shm_state: ShmState,
    pub data_device_state: DataDeviceState,
    pub dmabuf_state: DmabufState,
    pub xdg_shell_state: XdgShellState,
    pub layer_shell_state: WlrLayerShellState,
    pub xdg_decoration_state: XdgDecorationState,
    pub xdg_activation_state: XdgActivationState,
    pub presentation_state: PresentationState,
    pub fractional_scale_manager_state: FractionalScaleManagerState,
    pub viewporter_state: ViewporterState,
    pub single_pixel_buffer_state: SinglePixelBufferState,
    pub relative_pointer_manager_state: RelativePointerManagerState,
    pub pointer_constraints_state: PointerConstraintsState,
    pub screencopy_state: ScreencopyState,
    pub idle_notifier_state: IdleNotifierState,
    pub input_method_manager_state: InputMethodManagerState,
    pub text_input_manager_state: TextInputManagerState,
    pub foreign_toplevel_manager_state: Arc<StdMutex<ForeignToplevelManagerState>>,

    // --- Output, Window, and Workspace Management ---
    pub output_manager_state: OutputManagerState,
    pub space: Arc<StdMutex<Space<ManagedWindow>>>,
    pub popups: Arc<StdMutex<PopupManager>>,
    pub windows: HashMap<DomainWindowIdentifier, Arc<ManagedWindow>>,
    pub output_workspaces: HashMap<String, Vec<Arc<RwLock<CompositorWorkspace>>>>,
    pub active_workspaces: Arc<RwLock<HashMap<String, Uuid>>>,
    pub primary_output_name: Arc<RwLock<Option<String>>>,

    // --- Input Management ---
    pub seat_state: NovaSeatState,
    pub primary_seat: Seat<Self>,
    pub pointer_location: Point<f64, Logical>,
    pub cursor_status: Arc<StdMutex<CursorImageStatus>>,
    pub keyboard_layout_manager: KeyboardLayoutManager,
    pub keyboard_data_map: HashMap<String, XkbKeyboardData>,
    pub touch_focus_per_slot: HashMap<TouchSlotId, WlWeakSurface>,
    pub active_move_grab: Option<MoveGrabState>,
    pub active_resize_grab: Option<ActiveResizeGrabState>,

    // --- Rendering ---
    pub renderer: Option<Box<dyn FrameRenderer>>,
    pub active_renderer_type: Option<RendererType>,
    pub damage_tracker_state: DamageTrackerState,
    pub last_render_time: Instant,
    pub cursor_texture: Option<Arc<dyn RenderableTexture>>,
    pub cursor_hotspot: Point<i32, Logical>,

    // --- XWayland ---
    pub xwayland_connection: Option<Arc<XWaylandConnection>>,
    pub xwayland_guard: Option<XWayland<Self>>,

    // --- Idle Notification ---
    pub last_activity_time: Arc<StdMutex<Option<Instant>>>,
    pub is_user_idle: Arc<StdMutex<bool>>,
    pub idle_timeout: Duration,
    pub idle_timer_handle: Option<TimerHandle>,
}

impl DesktopState {
    pub fn new(
        event_loop: &mut EventLoop<'static, Self>,
        display: &mut Display<Self>,
    ) -> SystemResult<Self> {
        let display_handle = display.handle();
        let event_loop_handle = event_loop.handle();
        let clock = Clock::new();
        info!("Initializing NovaDE DesktopState...");

        // --- Protocol State Initialization ---
        let compositor_state = CompositorState::new::<Self>(&display_handle, clock.id());
        let subcompositor_state = SubcompositorState::new::<Self>(&display_handle);
        let shm_state = ShmState::new::<Self>(&display_handle, vec![], clock.id());
        let data_device_state = DataDeviceState::new::<Self>(&display_handle, clock.id());
        let dmabuf_state = DmabufState::new();
        let xdg_shell_state = XdgShellState::new::<Self>(&display_handle, clock.id());
        let layer_shell_state = WlrLayerShellState::new::<Self>(&display_handle, clock.id());
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle, clock.id());
        let xdg_activation_state = XdgActivationState::new::<Self>(&display_handle, clock.id());
        let presentation_state = PresentationState::new::<Self>(&display_handle, clock.id());
        let fractional_scale_manager_state = FractionalScaleManagerState::new::<Self>(&display_handle, clock.id());
        let viewporter_state = ViewporterState::new::<Self>(&display_handle, clock.id());
        let single_pixel_buffer_state = SinglePixelBufferState::new::<Self>(&display_handle, clock.id());
        let relative_pointer_manager_state = RelativePointerManagerState::new::<Self>(&display_handle, clock.id());
        let pointer_constraints_state = PointerConstraintsState::new::<Self>(&display_handle, clock.id());
        let screencopy_state = ScreencopyState::new::<Self>(&display_handle);
        let idle_notifier_state = IdleNotifierState::new::<Self>(&display_handle);
        let input_method_manager_state = InputMethodManagerState::new::<Self>(&display_handle);
        let text_input_manager_state = TextInputManagerState::new::<Self>(&display_handle);
        let foreign_toplevel_manager_state = Arc::new(StdMutex::new(ForeignToplevelManagerState::new()));

        // --- Output, Workspace, and Space Initialization ---
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
        let space = Arc::new(StdMutex::new(Space::new(clock.id())));
        let popups = Arc::new(StdMutex::new(PopupManager::new(clock.id())));

        // Workspace setup can be complex, for now we initialize empty maps
        let output_workspaces = HashMap::new();
        let active_workspaces = Arc::new(RwLock::new(HashMap::new()));
        let primary_output_name = Arc::new(RwLock::new(None));

        // --- Input Initialization ---
        let mut seat_state_manager = NovaSeatState::new();
        let primary_seat = seat_state_manager.inner.new_wl_seat(&display_handle, "seat0".to_string(), clock.id());
        let cursor_status = Arc::new(StdMutex::new(CursorImageStatus::Default));
        let keyboard_layout_manager = KeyboardLayoutManager::new()?;

        if let Err(e) = primary_seat.add_keyboard(keyboard_layout_manager.xkb_config_cloned(), 200, 25) {
            warn!("Failed to add keyboard to seat: {}", e);
        }
        primary_seat.add_pointer();
        primary_seat.add_touch();

        Ok(Self {
            display_handle,
            event_loop_handle,
            running: Arc::new(RwLock::new(true)),
            clock,
            compositor_state,
            subcompositor_state,
            shm_state,
            data_device_state,
            dmabuf_state,
            xdg_shell_state,
            layer_shell_state,
            xdg_decoration_state,
            xdg_activation_state,
            presentation_state,
            fractional_scale_manager_state,
            viewporter_state,
            single_pixel_buffer_state,
            relative_pointer_manager_state,
            pointer_constraints_state,
            screencopy_state,
            idle_notifier_state,
            input_method_manager_state,
            text_input_manager_state,
            foreign_toplevel_manager_state,
            output_manager_state,
            space,
            popups,
            windows: HashMap::new(),
            output_workspaces,
            active_workspaces,
            primary_output_name,
            seat_state: seat_state_manager,
            primary_seat,
            pointer_location: (0.0, 0.0).into(),
            cursor_status,
            keyboard_layout_manager,
            keyboard_data_map: HashMap::new(),
            touch_focus_per_slot: HashMap::new(),
            active_move_grab: None,
            active_resize_grab: None,
            renderer: None, // To be initialized by the backend
            damage_tracker_state: DamageTrackerState::new(),
            last_render_time: Instant::now(),
            cursor_texture: None,
            cursor_hotspot: (0, 0).into(),
            xwayland_connection: None,
            xwayland_guard: None,
            last_activity_time: Arc::new(StdMutex::new(Some(Instant::now()))),
            is_user_idle: Arc::new(StdMutex::new(false)),
            idle_timeout: Duration::from_secs(300),
            idle_timer_handle: None,
        })
    }

    // ... other methods from the original files will be merged here ...

    pub fn init_renderer_backend(&mut self, renderer_type: RendererType) -> SystemResult<()> {
        info!("Initializing renderer backend: {:?}", renderer_type);
        self.active_renderer_type = Some(renderer_type);

        let renderer: Box<dyn FrameRenderer> = match renderer_type {
            RendererType::Gles2 => {
                // Placeholder for GLES2 renderer initialization
                // This would typically involve setting up an EGL context.
                // For now, we'll assume a Gles2Renderer can be created.
                // let gles_renderer = Gles2Renderer::new(...)
                // Box::new(gles_renderer)
                unimplemented!("GLES2 renderer initialization not yet implemented in backend handler.");
            }
            RendererType::Vulkan => {
                // Placeholder for Vulkan renderer initialization
                // This involves creating instance, device, swapchain, etc.
                // let vulkan_renderer = VulkanFrameRenderer::new(...)
                // Box::new(vulkan_renderer)
                unimplemented!("Vulkan renderer initialization not yet implemented in backend handler.");
            }
        };

        self.renderer = Some(renderer);
        info!("Renderer backend initialized and stored in DesktopState.");
        Ok(())
    }
}
