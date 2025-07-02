// This is novade-system/src/compositor/backend/winit_backend.rs
// Implementation for running the compositor within a Winit window for testing and development.

use std::time::Duration;
use smithay::{
    backend::{
        input::{self as backend_input, Axis, AxisSource as BackendAxisSource, InputEvent as BackendInputEvent},
        renderer::{gles2::Gles2Renderer, RendererNode},
        winit::{self as smithay_winit, WinitEvent, WinitEventLoop, WinitGraphicsBackend},
    },
    desktop::Output,
    reexports::{
        calloop::LoopHandle as CalloopLoopHandle,
        winit::{
            event_loop::{ControlFlow, EventLoop as WinitAssociatedEventLoop, EventLoopBuilder as WinitEventLoopBuilder, EventLoopProxy},
            window::{WindowBuilder, Window as WinitWindow},
            dpi::LogicalSize,
        },
        wayland_server::DisplayHandle,
    },
    input::pointer::AxisFrame, // For PointerAxis event construction
    utils::{Rectangle, Point, Size, Transform, Physical, Scale, SERIAL_COUNTER},
};
use tracing::{info, error, warn, debug};
use raw_window_handle::HasRawWindowHandle;

use crate::compositor::{
    state::DesktopState,
    render::{MainNovaRenderer, GlesNovaRenderer},
    errors::CompositorError,
    input::process_input_event,
};

pub const DEFAULT_WINDOW_TITLE: &str = "NovaDE Compositor (Winit)";
pub const WINIT_OUTPUT_NAME: &str = "winit";

/// Data specific to the Winit backend that might be stored or managed.
/// For now, graphics_backend is passed mutably to event handler.
pub struct WinitData {
    pub window: Arc<WinitWindow>, // Make window Arc for potential sharing
    // pub graphics_backend: Arc<Mutex<Box<dyn WinitGraphicsBackend<Renderer = Gles2Renderer>>>>, // If shared
    pub smithay_output: Output,
    pub renderer_node: RendererNode,
    pub event_loop_proxy: EventLoopProxy<()>, // To request redraws from other parts of Calloop
}


/// Initializes the Winit backend.
/// Returns the Winit event loop (to be run by the caller, e.g. core.rs),
/// and the WinitData containing window, graphics backend, output, etc.
pub fn init_winit_backend(
    display_handle: DisplayHandle,
    clock_id: usize, // Pass clock_id for output creation
) -> Result<(WinitAssociatedEventLoop<()>, WinitData, Gles2Renderer), CompositorError> {
    info!("Initializing Winit backend for NovaDE Compositor...");

    let winit_event_loop = WinitEventLoopBuilder::new().with_user_event().build(); // with_user_event for proxy
    let window = Arc::new(WindowBuilder::new() // Arc the window
        .with_title(DEFAULT_WINDOW_TITLE)
        .with_inner_size(LogicalSize::new(1280, 800))
        .build(&winit_event_loop)
        .map_err(|e| CompositorError::BackendCreation(format!("Failed to create Winit window: {}", e)))?);

    let (graphics_backend, gles_renderer) = match smithay_winit::init_renderer_window(window.clone()) { // Pass Arc<Window>
        Ok(res) => res,
        Err(e) => {
            return Err(CompositorError::BackendCreation(format!("Failed to initialize Winit GL backend: {}", e)));
        }
    };
    info!("Winit GL backend and Gles2Renderer initialized.");

    let physical_properties = smithay::output::PhysicalProperties {
        size: window.inner_size().into(),
        subpixel: smithay::output::Subpixel::Unknown,
        make: "Winit".into(),
        model: "Window".into(),
    };
    let smithay_output = Output::new(WINIT_OUTPUT_NAME.to_string(), physical_properties, clock_id, display_handle);

    let window_size_physical: Size<i32, Physical> = window.inner_size().into();
    let mode = smithay::output::Mode {
        size: window_size_physical,
        refresh: 60_000,
    };
    smithay_output.change_current_state(Some(mode), Some(Transform::Normal), Some(Scale::Integer(1)), Some((0, 0).into()));
    smithay_output.set_preferred(mode);

    let renderer_node = unsafe { RendererNode::new_for_unknown_plane(window.raw_window_handle()) };

    let winit_data = WinitData {
        window: window.clone(),
        // graphics_backend will be returned separately to be owned by the main loop closure
        smithay_output,
        renderer_node,
        event_loop_proxy: winit_event_loop.create_proxy(),
    };

    info!("Winit backend components created. Event loop to be run by caller.");
    // We return the Gles2Renderer separately because WinitGraphicsBackend takes ownership of it in some init paths,
    // but we want our MainNovaRenderer to own it. smithay_winit::init_renderer_window returns it.
    Ok((winit_event_loop, winit_data, gles_renderer))
}

/// Handles a single Winit event.
/// This is intended to be called from the Winit event loop closure in `core.rs`.
pub fn handle_winit_event(
    event: smithay::reexports::winit::event::Event<'_, ()>,
    desktop_state: &mut DesktopState,
    winit_data: &WinitData, // Pass WinitData by reference
    graphics_backend: &mut dyn WinitGraphicsBackend<Renderer = Gles2Renderer>, // Pass backend mutably
    control_flow: &mut ControlFlow
) {
    match event {
        smithay::reexports::winit::event::Event::WindowEvent { event: winit_window_event, window_id } if window_id == winit_data.window.id() => {
            match winit_window_event {
                smithay::reexports::winit::event::WindowEvent::Resized(new_size) => {
                    info!("Winit window resized to {:?}, scale factor: {}", new_size, winit_data.window.scale_factor());
                    graphics_backend.resize(new_size.into(), None);

                    let space = desktop_state.space.lock().unwrap();
                    if let Some(output) = space.outputs().find(|o| o.name() == WINIT_OUTPUT_NAME) {
                        let new_mode = smithay::output::Mode {
                            size: new_size.into(),
                            refresh: output.current_mode().map_or(60_000, |m| m.refresh),
                        };
                        let new_scale = Scale::Float(winit_data.window.scale_factor());
                        output.change_current_state(Some(new_mode), None, Some(new_scale), None);
                        // TODO: Trigger layout recalculation & damage output.
                    }
                    winit_data.window.request_redraw();
                }
                smithay::reexports::winit::event::WindowEvent::Focused(focused) => {
                    info!("Winit window focus changed: {}", focused);
                    // TODO: Update internal focus state if needed, e.g., for seat.
                }
                smithay::reexports::winit::event::WindowEvent::CloseRequested => {
                    info!("Winit window close requested. Shutting down compositor.");
                    *desktop_state.running.write().unwrap() = false;
                }
                smithay::reexports::winit::event::WindowEvent::KeyboardInput { event: ki, .. } => {
                    if let Some(smithay_event) = smithay_winit::convert_keyboard_input(ki) {
                         process_input_event(desktop_state, BackendInputEvent::Keyboard{ event: smithay_event });
                    }
                }
                smithay::reexports::winit::event::WindowEvent::CursorMoved { position, .. } => {
                    if let Some(pointer) = desktop_state.primary_seat.get_pointer() {
                        let logical_pos: Point<f64, Logical> = position.to_logical(winit_data.window.scale_factor()).into();
                        desktop_state.pointer_location = logical_pos; // Update global pointer location
                        pointer.motion(desktop_state, logical_pos, SERIAL_COUNTER.next_serial(), desktop_state.clock.now().as_millis() as u32);
                    }
                }
                smithay::reexports::winit::event::WindowEvent::MouseInput { state: btn_state, button, .. } => {
                    let serial = SERIAL_COUNTER.next_serial();
                    let time = desktop_state.clock.now().as_millis() as u32;

                    if let Some(pointer) = desktop_state.primary_seat.get_pointer() {
                        if let Some(s_button) = smithay_winit::convert_mouse_button(button) {
                            let s_state = smithay_winit::convert_element_state(btn_state);
                            pointer.button(desktop_state, s_button.into(), s_state, serial, time);

                            // Click-to-focus logic
                            if s_state == backend_input::ButtonState::Pressed &&
                               (s_button == backend_input::MouseButton::Left || s_button == backend_input::MouseButton::Right) {
                                let space = desktop_state.space.lock().unwrap();
                                if let Some(window) = space.element_under(desktop_state.pointer_location).map(|(w, _loc)| w.clone()) {
                                    if let Some(keyboard) = desktop_state.primary_seat.get_keyboard() {
                                        let target_surface = match window.toplevel() {
                                            Some(WindowSurfaceType::Xdg(xdg)) => Some(xdg.wl_surface().clone()),
                                            Some(WindowSurfaceType::X11(x11)) => x11.wl_surface(), // X11 surface might have its own wl_surface for input
                                            _ => None,
                                        };
                                        if let Some(surface_to_focus) = target_surface {
                                            keyboard.set_focus(desktop_state, Some(&surface_to_focus), serial);
                                            info!("Set keyboard focus to surface {:?} on click.", surface_to_focus.id());
                                        } else {
                                            // If no specific surface, try to focus the window's primary surface if any
                                            if let Some(primary_wl_surface) = window.wl_surface() {
                                                 keyboard.set_focus(desktop_state, Some(&primary_wl_surface), serial);
                                                 info!("Set keyboard focus to primary_wl_surface {:?} of window on click.", primary_wl_surface.id());
                                            } else {
                                                warn!("Clicked window has no obvious surface to focus for keyboard input.");
                                            }
                                        }
                                    }
                                } else {
                                    // Clicked on empty space, clear focus
                                     if let Some(keyboard) = desktop_state.primary_seat.get_keyboard() {
                                        keyboard.set_focus(desktop_state, None, serial);
                                        info!("Cleared keyboard focus due to click on empty space.");
                                    }
                                }
                            }
                        }
                    }
                }
                smithay::reexports::winit::event::WindowEvent::MouseWheel { delta, phase, .. } => {
                     if let Some(pointer) = desktop_state.primary_seat.get_pointer() {
                        let (x, y) = smithay_winit::convert_mouse_scroll(delta, phase);
                        let mut frame = AxisFrame::new(desktop_state.clock.now().as_millis() as u32)
                            .source(smithay::input::pointer::AxisSource::Wheel);
                        if x.abs() > f64::EPSILON { // Use f64 for comparison
                            frame = frame.value(smithay::input::pointer::Axis::Horizontal, x * 10.0);
                        }
                        if y.abs() > f64::EPSILON {
                            frame = frame.value(smithay::input::pointer::Axis::Vertical, y * 10.0);
                        }
                        // Only send if there's actual scroll value
                        if x.abs() > f64::EPSILON || y.abs() > f64::EPSILON {
                           pointer.axis(desktop_state, frame);
                        }
                    }
                }
                // TODO: Handle Touch events from Winit
                _ => {}
            }
        }
        smithay::reexports::winit::event::Event::RedrawRequested(window_id) if window_id == winit_data.window.id() => {
            debug!("Winit redraw requested for output: {}", WINIT_OUTPUT_NAME);
            // The actual rendering is called from core.rs main loop based on this request or damage.
            // Here, we just acknowledge it. The main loop in core.rs should check for pending redraws.
            // For direct rendering test:
            // if let Some(MainNovaRenderer::Gles(gles_renderer)) = desktop_state.main_renderer.as_mut() {
            //     // Simplified render call, real one needs elements and damage_tracker
            //     if let Err(e) = graphics_backend.render_frame_srgb(|renderer, frame| {
            //         // renderer is &mut Gles2Renderer, frame is &mut Frame
            //         // frame.clear([0.1, 0.1, 0.1, 1.0], &[Rectangle::from_loc_and_size((0,0), winit_data.smithay_output.current_mode().unwrap().size)])?;
            //         Ok(())
            //     }) {
            //         error!("Winit rendering failed: {}", e);
            //     }
            // }
        }
        smithay::reexports::winit::event::Event::MainEventsCleared => {
             winit_data.window.request_redraw(); // Keep requesting redraw for continuous rendering/updates
        }
        smithay::reexports::winit::event::Event::LoopDestroyed => {
            info!("Winit event loop destroyed.");
            *desktop_state.running.write().unwrap() = false;
        }
        _ => {}
    }

    if !*desktop_state.running.read().unwrap() {
        *control_flow = ControlFlow::Exit;
    } else {
        *control_flow = ControlFlow::WaitUntil(std::time::Instant::now() + Duration::from_millis(16));
    }
}
