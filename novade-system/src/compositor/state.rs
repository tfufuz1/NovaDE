// This is novade-system/src/compositor/state.rs
// Defines CompositorStateGlobal (as DesktopState) and other state-related structs.

//! Holds the main state of the NovaDE Wayland compositor, `DesktopState`.
//!
//! `DesktopState` aggregates all other state objects (like those for Wayland protocols,
//! input, output, rendering, and window management) and serves as the central data
//! structure for the Calloop event loop.

use std::{
    ffi::OsString,
    sync::{Arc, Mutex as StdMutex, RwLock},
    time::{Duration, Instant},
};
use tracing::{debug, error, info, info_span, warn};

use smithay::{
    backend::renderer::{
        gles2::Gles2Renderer, // Example, not actively used if WGPU is primary
    },
    desktop::{Space, Window, PopupManager, LayerSurface, WindowSurfaceType},
    input::{
        Seat, SeatState as SmithaySeatState, SeatHandler, // Renamed Smithay's SeatState
        pointer::{CursorImageStatus, GrabStartData as PointerGrabStartData, PointerAxisEventData, PointerButtonEventData, PointerMotionEventData, RelativeMotionEvent},
        keyboard::{KeyboardHandle, ModifiersState, XkbConfig, Keysym, FilterResult as XkbFilterResult, KeysymHandle as SmithayKeysymHandle},
        touch::TouchEventData,
        SeatGrab,
        SeatFocus,
    },
    output::{Output as SmithayOutput, OutputHandler, OutputState as SmithayOutputState},
    reexports::{
        calloop::{EventLoop, LoopHandle, generic::Generic, Interest, Mode, PostAction, TimerHandle},
        wayland_server::{
            backend::{ClientId, DisconnectReason, GlobalId},
            protocol::{
                wl_output, wl_surface::WlSurface, wl_seat, wl_buffer::WlBuffer,
                wl_data_device_manager, wl_compositor, wl_subcompositor, wl_shm,
            },
            Display, DisplayHandle, Client, ListeningSocket, GlobalDispatch, Dispatch, New, DataInit,
        },
        wayland_protocols::{
            xdg::{
                shell::server::{xdg_wm_base::{self, XdgWmBase}, xdg_surface, xdg_toplevel::{self, XdgToplevel}, xdg_popup},
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
                pointer_constraints::zv1::server::zwp_pointer_constraints_v1,
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
        compositor::{CompositorState, CompositorHandler, CompositorClientState, SurfaceAttributes as WlSurfaceAttributes, SubsurfaceCachedState, TraversalAction, SurfaceData},
        data_device::{DataDeviceState, DataDeviceHandler, ServerDndGrabHandler, ClientDndGrabHandler, DataDeviceUserData, SelectionSource, SelectionTarget},
        dmabuf::{DmabufState, DmabufHandler, DmabufGlobalData, DmabufClientData, DmabufData},
        output::{OutputManagerState, OutputData, XdgOutputUserData},
        subcompositor::{SubcompositorState, SubcompositorHandler},
        shell::{
            xdg::{XdgShellState, XdgShellHandler, XdgSurfaceUserData, XdgPopupUserData, XdgToplevelUserData, XdgPositionerUserData, Configure, XdgWmBaseClientData},
            wlr_layer::{WlrLayerShellState, WlrLayerShellHandler, LayerSurfaceData, Layer},
        },
        shm::{ShmState, ShmHandler, ShmClientData},
        // seat::SeatState has been wrapped in NovaSeatState
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
    },
    xwayland::{XWayland, XWaylandEvent, XWaylandClientData, XWaylandSurface, Xwm, XWaylandConnection},
    signaling::SignalToken,
};
use smithay::wayland::shell::xdg::decoration::{XdgDecorationState, XdgDecorationHandler, Mode as XdgToplevelDecorationMode, ZxdgToplevelDecorationV1};
use smithay::desktop::space::SpaceElement;

use crate::compositor::foreign_toplevel::ForeignToplevelManagerState;


/// Client data attached to each Wayland client. Empty for now.
pub struct ClientState as NovaClientState;

/// UserData for storing active pointer constraint on a seat or surface.
/// This is a simplified example; a real implementation might need more robust tracking.
#[derive(Default, Debug, Clone)]
pub struct ActivePointerConstraint {
    pub constraint: Option<PointerConstraintData>,
}

/// Wrapper around Smithay's `SeatState` to potentially add NovaDE-specific seat logic in the future.
pub struct NovaSeatState {
    pub inner: SmithaySeatState<DesktopState>,
}

impl NovaSeatState {
    pub fn new() -> Self {
        Self { inner: SmithaySeatState::new() }
    }
}

/// The main state object for the NovaDE Wayland compositor.
///
/// This struct aggregates all protocol-specific states from Smithay,
/// manages core compositor resources like the display, event loop, input seats,
/// window layout (`Space`), and timing. It serves as the central `Data` argument
/// for the Calloop event loop, making it accessible to all event handlers.
pub struct DesktopState {
    // Core Handles & Control
    pub display_handle: DisplayHandle,
    pub event_loop_handle: LoopHandle<'static, Self>,
    pub running: Arc<RwLock<bool>>,

    // Smithay Wayland Protocol States
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
    pub idle_notifier_state: IdleNotifierState,
    pub input_method_manager_state: InputMethodManagerState,
    pub text_input_manager_state: TextInputManagerState,
    pub foreign_toplevel_manager_state: Arc<StdMutex<ForeignToplevelManagerState>>,

    // Output & Window Management
    pub output_manager_state: OutputManagerState, // Manages wl_output and zxdg_output_manager_v1
    pub xdg_output_manager_state: OutputManagerState, // Re-uses output_manager_state for zxdg_output_v1 logic
    pub space: Arc<StdMutex<Space<Window>>>,
    pub popups: Arc<StdMutex<PopupManager>>,

    // Input Management
    pub seat_state: NovaSeatState,
    pub primary_seat: Seat<Self>,
    pub pointer_location: Point<f64, Logical>,
    pub cursor_status: Arc<StdMutex<CursorImageStatus>>,

    // XWayland (optional feature)
    pub xwayland_connection: Option<Arc<XWaylandConnection>>,
    pub xwayland_guard: Option<XWayland<DesktopState>>,

    // Idle Notification State
    pub last_activity_time: Arc<StdMutex<Option<std::time::Instant>>>,
    pub is_user_idle: Arc<StdMutex<bool>>,
    pub idle_timeout: Duration,
    pub idle_timer_handle: Option<TimerHandle>,

    // Utilities
    pub clock: Clock,

    // Registered Global IDs (for tracking/debugging)
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
    pub xdg_output_manager_global: Option<GlobalId>, // This is the zxdg_output_manager_v1 global
    pub single_pixel_buffer_global: Option<GlobalId>,
    pub relative_pointer_manager_global: Option<GlobalId>,
    pub pointer_constraints_global: Option<GlobalId>,
    pub foreign_toplevel_manager_global: Option<GlobalId>,
    pub idle_notifier_global: Option<GlobalId>,
    pub input_method_manager_global: Option<GlobalId>,
    pub text_input_manager_global: Option<GlobalId>,
}

impl DesktopState {
    /// Creates a new `DesktopState`.
    ///
    /// Initializes all necessary Wayland protocol states, input/output managers,
    /// window management structures, and utility services like clocks and timers.
    /// It also registers fundamental Wayland globals with the provided display.
    pub fn new(
        _event_loop: &mut EventLoop<'static, Self>,
        display: &mut Display<Self>,
    ) -> Self {
        // Temporary block to hold mutable `state_instance` during construction.
        let mut state_instance = {
            let display_handle = display.handle();
            let event_loop_handle = _event_loop.handle();

            info!("Initializing NovaDE DesktopState...");

            let clock = Clock::new();

            // Initialize Smithay's state objects for various Wayland protocols
            let compositor_state = CompositorState::new::<Self>(&display_handle, clock.id());
            let subcompositor_state = SubcompositorState::new::<Self>(&display_handle);
            // TODO: SHM formats should be configurable or queried from a renderer
            let shm_state = ShmState::new::<Self>(&display_handle, vec![], clock.id());
            let data_device_state = DataDeviceState::new::<Self>(&display_handle, clock.id());
            let dmabuf_state = DmabufState::new();

            let xdg_shell_state = XdgShellState::new::<Self>(&display_handle, clock.id());
            let layer_shell_state = WlrLayerShellState::new::<Self>(&display_handle, clock.id());

            let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
            let space = Arc::new(StdMutex::new(Space::new(clock.id())));
            let popups = Arc::new(StdMutex::new(PopupManager::new(clock.id())));

            let mut seat_state_manager = NovaSeatState::new();
            let primary_seat = seat_state_manager.inner.new_wl_seat(&display_handle, "seat0".to_string(), clock.id());
            let cursor_status = Arc::new(StdMutex::new(CursorImageStatus::Default));

            let xdg_decoration_state = XdgDecorationState::new::<Self>(&display_handle, clock.id());
            let xdg_activation_state = XdgActivationState::new::<Self>(&display_handle, clock.id());
            let presentation_state = PresentationState::new::<Self>(&display_handle, clock.id());
            let fractional_scale_manager_state = FractionalScaleManagerState::new::<Self>(&display_handle, clock.id());
            let viewporter_state = ViewporterState::new::<Self>(&display_handle, clock.id());
            let single_pixel_buffer_state = SinglePixelBufferState::new::<Self>(&display_handle, clock.id());
            let relative_pointer_manager_state = RelativePointerManagerState::new::<Self>(&display_handle, clock.id());
            let pointer_constraints_state = PointerConstraintsState::new::<Self>(&display_handle, clock.id());
            let foreign_toplevel_manager_state = Arc::new(StdMutex::new(ForeignToplevelManagerState::new()));
            let idle_notifier_state = IdleNotifierState::new::<Self>(&display_handle);
            let input_method_manager_state = InputMethodManagerState::new::<Self>(&display_handle);
            let text_input_manager_state = TextInputManagerState::new::<Self>(&display_handle);

            // Initialize idle tracking state
            let last_activity_time = Arc::new(StdMutex::new(Some(std::time::Instant::now())));
            let is_user_idle = Arc::new(StdMutex::new(false));
            // TODO: Make idle_timeout configurable
            let idle_timeout = Duration::from_secs(300); // Default 5 minutes


            let mut desktop_state_fields = Self { // Changed name to avoid conflict
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
                output_manager_state: output_manager_state.clone(),
                space,
                popups,
                seat_state: seat_state_manager,
                primary_seat,
                pointer_location: (0.0, 0.0).into(),
                cursor_status,
                xwayland_connection: None,
                xwayland_guard: None,
                xdg_decoration_state,
                xdg_activation_state,
                presentation_state,
                fractional_scale_manager_state,
                viewporter_state,
                xdg_output_manager_state: output_manager_state,
                single_pixel_buffer_state,
                relative_pointer_manager_state,
                pointer_constraints_state,
                foreign_toplevel_manager_state,
                idle_notifier_state,
                input_method_manager_state,
                text_input_manager_state,
                last_activity_time,
                is_user_idle,
                idle_timeout,
                idle_timer_handle: None,
                clock,
                // Initialize all global ID fields to None
                compositor_global: None, subcompositor_global: None, shm_global: None,
                data_device_global: None, dmabuf_global: None, xdg_shell_global: None,
                layer_shell_global: None, xdg_decoration_global: None, xdg_activation_global: None,
                presentation_global: None, fractional_scale_manager_global: None,
                viewporter_global: None, xdg_output_manager_global: None,
                single_pixel_buffer_global: None, relative_pointer_manager_global: None,
                pointer_constraints_global: None, foreign_toplevel_manager_global: None,
                idle_notifier_global: None, input_method_manager_global: None,
                text_input_manager_global: None,
            };

            // Create and store DMABUF global ID
            use smithay::backend::allocator::Format;
            use smithay::reexports::drm_fourcc::{DrmFourcc, DrmFormatModifier};
            // TODO: Query actual supported DMABUF formats from renderer/DRM backend
            let preferred_dmabuf_formats = [
                Format { code: DrmFourcc::Argb8888, modifier: DrmFormatModifier::Linear },
                Format { code: DrmFourcc::Xrgb8888, modifier: DrmFormatModifier::Linear },
            ];
            let dmabuf_global_id = desktop_state_fields.dmabuf_state.create_global_with_default_feedback::<DesktopState>(
                &desktop_state_fields.display_handle,
                &preferred_dmabuf_formats,
                Some(tracing::Span::current().into()),
            );
            desktop_state_fields.dmabuf_global = Some(dmabuf_global_id);
            info!(
                "DMABUF global (zwp_linux_dmabuf_v1) registered, advertising preferred formats: {:?}",
                preferred_dmabuf_formats
            );

            // Create and store Foreign Toplevel Manager global ID
            let ft_manager_global = desktop_state_fields.display_handle.create_global::<DesktopState, ZwlrForeignToplevelManagerV1, _>(1, ());
            desktop_state_fields.foreign_toplevel_manager_global = Some(ft_manager_global);
            info!("Foreign Toplevel Manager global (zwlr_foreign_toplevel_manager_v1) registered.");

            // Store other global IDs if their states don't do it or if needed for direct access
            // Most Smithay states register globals upon their creation with ::new::<Self>(...)
            desktop_state_fields.compositor_global = Some(desktop_state_fields.compositor_state.global());
            desktop_state_fields.subcompositor_global = Some(desktop_state_fields.subcompositor_state.global());
            desktop_state_fields.shm_global = Some(desktop_state_fields.shm_state.global());
            desktop_state_fields.data_device_global = Some(desktop_state_fields.data_device_state.global());
            desktop_state_fields.xdg_shell_global = Some(desktop_state_fields.xdg_shell_state.global());
            desktop_state_fields.layer_shell_global = Some(desktop_state_fields.layer_shell_state.global());
            desktop_state_fields.xdg_decoration_global = Some(desktop_state_fields.xdg_decoration_state.global());
            desktop_state_fields.xdg_activation_global = Some(desktop_state_fields.xdg_activation_state.global());
            desktop_state_fields.presentation_global = Some(desktop_state_fields.presentation_state.global());
            desktop_state_fields.fractional_scale_manager_global = Some(desktop_state_fields.fractional_scale_manager_state.global());
            desktop_state_fields.viewporter_global = Some(desktop_state_fields.viewporter_state.global());
            desktop_state_fields.single_pixel_buffer_global = Some(desktop_state_fields.single_pixel_buffer_state.global());
            desktop_state_fields.relative_pointer_manager_global = Some(desktop_state_fields.relative_pointer_manager_state.global());
            desktop_state_fields.pointer_constraints_global = Some(desktop_state_fields.pointer_constraints_state.global());
            desktop_state_fields.idle_notifier_global = Some(desktop_state_fields.idle_notifier_state.global());
            desktop_state_fields.input_method_manager_global = Some(desktop_state_fields.input_method_manager_state.global());
            desktop_state_fields.text_input_manager_global = Some(desktop_state_fields.text_input_manager_state.global());
            // xdg_output_manager_global is implicitly part of output_manager_state when created with new_with_xdg_output

            desktop_state_fields
        };
        new_state_build
    }

    /// Records user activity, resetting the idle timer and notifying clients if resuming from idle.
    /// `is_significant_activity` can be used to differentiate between minor (e.g. small mouse move)
    /// and major (e.g. key press, button click) activity, though current logic resets timer fully on any activity.
    pub fn record_user_activity(&mut self, _is_significant_activity: bool) { // _is_significant_activity currently unused but kept for API
        let mut last_activity_guard = self.last_activity_time.lock().unwrap();
        let mut is_idle_guard = self.is_user_idle.lock().unwrap();
        let now = std::time::Instant::now();

        let previously_idle = *is_idle_guard;
        *last_activity_guard = Some(now);

        if previously_idle {
            *is_idle_guard = false;
            info!("User activity resumed, notifying idle_notifier_state.");
            self.idle_notifier_state.notify_activity(&self.display_handle);
        }

        if let Some(timer_handle) = &self.idle_timer_handle {
             debug!("Resetting idle timer to {:?} due to user activity.", self.idle_timeout);
             timer_handle.add_timeout(self.idle_timeout, ());
        } else {
            warn!("Idle timer handle not available to reset on user activity.");
        }
    }

    /// Checks if the user has become idle based on `last_activity_time` and `idle_timeout`.
    /// Notifies clients via `IdleNotifierState` if transitioning to idle.
    /// This method also reschedules the idle timer for the next check.
    pub fn check_for_idle_state(&mut self) {
        let mut reschedule_delay = self.idle_timeout; // Default to full timeout for next check
        {
            let mut is_idle_guard = self.is_user_idle.lock().unwrap();
            // This unwrap_or_else is a safeguard; last_activity_time should always be Some after init.
            let last_activity = self.last_activity_time.lock().unwrap().unwrap_or_else(Instant::now);

            if *is_idle_guard {
                // Already idle. Timer will be rescheduled for full duration.
                // No state change unless an inhibitor was just removed and we need to re-check sooner,
                // but inhibitor removal calls record_user_activity which resets the timer.
            } else if Instant::now().duration_since(last_activity) >= self.idle_timeout {
                // Idle timeout reached. Check inhibitors.
                // `is_inhibited()` on IdleNotifierState checks all active inhibitors.
                if !self.idle_notifier_state.is_inhibited() {
                    if !*is_idle_guard { // Check again due to lock patterns
                        *is_idle_guard = true;
                        info!("User has become idle, notifying idle_notifier_state.");
                        self.idle_notifier_state.notify_idle(&self.display_handle);
                    }
                } else {
                    info!("Idle timeout reached, but activity is inhibited. Checking again in a shorter interval.");
                    // If inhibited, check more frequently in case inhibitor is removed.
                    reschedule_delay = std::cmp::min(self.idle_timeout, Duration::from_secs(30));
                }
            } else {
                // Not yet idle, calculate remaining time for next check.
                let time_since_last_activity = Instant::now().duration_since(last_activity);
                if self.idle_timeout > time_since_last_activity {
                    reschedule_delay = self.idle_timeout - time_since_last_activity;
                } else {
                    // This case should ideally not be reached if logic is correct, means timeout just passed.
                    reschedule_delay = Duration::from_millis(100);
                }
            }
        }

        if let Some(timer_handle) = &self.idle_timer_handle {
            timer_handle.add_timeout(reschedule_delay, ());
            debug!("Idle check timer rescheduled for {:?}", reschedule_delay);
        }
    }

    /// Called by the rendering backend after a frame has been successfully presented.
    /// This method is responsible for triggering `wp_presentation_feedback` events.
    pub fn on_frame_presented(
        &mut self,
        surface: &WlSurface,
        _output: &SmithayOutput, // Not directly used by on_present_done, but good for context
        times: smithay::wayland::presentation::PresentationTimes,
    ) {
        if !surface.is_alive() {
            return;
        }
        // info!(surface_id = ?surface.id(), output = %_output.name(), "Frame presented, processing presentation feedback.");
        let clock_id = self.clock.id();
        self.presentation_state.on_present_done(
            surface,
            clock_id,
            times.presentation_time_monotonic,
            times.refresh_interval_ns,
            times.presentation_flags,
            &self.display_handle
        );
    }
}
