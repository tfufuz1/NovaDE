// novade-system/src/compositor/backend/winit_backend.rs

use anyhow::{Result, Context as AnyhowContext};
use calloop::{LoopHandle, EventLoop, timer::{Timer, TimeoutAction}};
use smithay::{
    backend::{
        renderer::{glow::GlowRenderer, gles::GlesError},
        winit::{self, WinitEvent, WinitEventLoop, WinitGraphicsBackend, WinitInputEvent}, // Added WinitInputEvent
    },
    reexports::wayland_server::DisplayHandle,
    utils::{Size, SERIAL_COUNTER}, // Added SERIAL_COUNTER
};
use std::time::Duration;
use smithay::backend::input::Axis; // For Axis enum

use crate::compositor::{
    core::state::DesktopState,
    // Import items that might be needed from main.rs, e.g., rendering logic
    // For now, we'll keep it minimal and refer to main.rs for complex parts.
};
use super::CompositorBackend; // Super refers to novade-system/src/compositor/backend/mod.rs

pub struct WinitBackend {
    event_loop_handle: LoopHandle<'static, DesktopState>,
    winit_event_loop: WinitEventLoop,
    // Smithay's WinitGraphicsBackend is generic over the Renderer.
    // We'll use GlowRenderer as seen in main.rs.
    // The backend itself (like WinitGraphicsBackend) might not be stored directly in the struct
    // if it's consumed or its parts are stored (like the renderer).
    // For now, let's assume we'll store what's necessary to run the event loop.
    // The renderer is a bit tricky as `backend.renderer()` in smithay example borrows it.
    // We might need to own the WinitGraphicsBackend or just the GlowRenderer.
    // Let's try owning WinitGraphicsBackend and GlowRenderer separately for now.
    backend: WinitGraphicsBackend<GlowRenderer>, // This holds the winit window and surface
    renderer: GlowRenderer, // The actual renderer
    display_handle: DisplayHandle,
}

impl CompositorBackend for WinitBackend {
    fn init(
        event_loop_handle: LoopHandle<'static, DesktopState>,
        display_handle: DisplayHandle,
        desktop_state: &mut DesktopState, // desktop_state is used for setup, not stored directly here
    ) -> Result<Self>
    where
        Self: Sized,
    {
        tracing::info!("Initializing Winit backend...");

        let (mut backend, winit_event_loop) = winit::init_from_builder(
            winit::WinitEventLoopBuilder::new().with_title("NovaDE Compositor (Winit)"),
            Some(event_loop_handle.clone()) // Pass the calloop handle for winit to integrate
        )
        .context("Failed to initialize Winit backend (smithay::backend::winit::init_from_builder)")?;

        let renderer = unsafe { GlowRenderer::new(backend.renderer()) }
            .context("Failed to initialize GlowRenderer for Winit backend")?;

        // DesktopState setup specific to Winit backend, if any.
        // For example, registering winit output, etc.
        // The main output registration usually happens via WinitEvent::OutputCreated.

        tracing::info!("Winit backend initialized successfully.");
        Ok(WinitBackend {
            event_loop_handle,
            winit_event_loop,
            backend,
            renderer,
            display_handle,
        })
    }

    fn run(mut self, desktop_state: &mut DesktopState) -> Result<()> {
        tracing::info!("Running Winit backend event loop...");

        // The main.rs has a complex timer-based loop for winit.
        // We need to adapt that here.
        // We are moving winit_event_loop, backend, renderer into this scope.

        let winit_timer = Timer::immediate();
        self.event_loop_handle.insert_source(winit_timer, move |_, _, state: &mut DesktopState| {
            // This closure captures `self.winit_event_loop`, `self.backend`, `self.renderer`
            // `desktop_state` is passed as `state` by calloop.
            // `self.display_handle` is also captured.

            let mut calloop_timeout_action = TimeoutAction::ToDuration(Duration::from_millis(16));

            if let Err(e) = self.winit_event_loop.dispatch_new_events(|event| {
                // Simplified event handling for now.
                // Full handling (Resized, CloseRequested, Input, Output*) will be needed.
                match event {
                    WinitEvent::CloseRequested { .. } => {
                        tracing::info!("Winit window close requested from winit_backend, initiating shutdown.");
                        calloop_timeout_action = TimeoutAction::Break;
                    }
                    WinitEvent::OutputCreated { output, .. } => {
                        tracing::info!("Winit backend created an output: {}", output.name());
                        // In Smithay, OutputHandler::new_output is typically invoked here by the winit backend itself.
                        // We need to ensure DesktopState's OutputHandler is correctly registered with Winit.
                        // This often happens if DesktopState implements OutputHandler and is passed to winit init,
                        // or if the output is manually added to DesktopState's space.
                        // For now, assuming DesktopState is correctly set up to handle this via its OutputHandler impl.
                        // Example: state.on_new_output(&output);
                    }
                    WinitEvent::Input(winit_event_data) => {
                        match winit_event_data {
                            WinitInputEvent::Keyboard { event } => {
                                if let Some(keyboard) = state.seat.get_keyboard() {
                                    let serial = SERIAL_COUNTER.next_serial();
                                    keyboard.input(
                                        state,
                                        event.key_code(),
                                        event.state(),
                                        serial,
                                        event.time_msec(),
                                        |_, modifiers, handle| {
                                            tracing::debug!(
                                                "Winit Keyboard event: keycode {}, state {:?}, keysym {:?}, modifiers {:?}",
                                                event.key_code(), event.state(), handle.modified_sym(), modifiers
                                            );
                                            smithay::input::keyboard::FilterResult::Forward
                                        }
                                    );
                                }
                            }
                            WinitInputEvent::PointerMotion { delta, time, .. } => {
                                if let Some(pointer) = state.seat.get_pointer() {
                                    state.pointer_location = pointer.current_position() + delta;
                                    pointer.motion(
                                        state,
                                        time,
                                    );
                                }
                            }
                            WinitInputEvent::PointerButton { button, state: button_state, time, .. } => {
                                if let Some(pointer) = state.seat.get_pointer() {
                                    let serial = SERIAL_COUNTER.next_serial();
                                    pointer.button(
                                        state,
                                        button,
                                        button_state,
                                        serial,
                                        time,
                                    );
                                }
                            }
                            WinitInputEvent::PointerAxis { source, horizontal, vertical, time, .. } => {
                                if let Some(pointer) = state.seat.get_pointer() {
                                    let mut axis_frame = smithay::input::pointer::AxisFrame::new(time)
                                        .source(source);
                                    if let Some((discrete_x, discrete_y)) = vertical.discrete_pixels().or_else(|| horizontal.discrete_pixels()) {
                                        if horizontal.discrete_pixels().is_some() {
                                            axis_frame = axis_frame.discrete(Axis::Horizontal, discrete_x as i32);
                                        }
                                        if vertical.discrete_pixels().is_some() {
                                             axis_frame = axis_frame.discrete(Axis::Vertical, discrete_y as i32);
                                        }
                                    }
                                    if let Some((continuous_x, continuous_y)) = vertical.pixels().or_else(|| horizontal.pixels()) {
                                         if horizontal.pixels().is_some() {
                                            axis_frame = axis_frame.value(Axis::Horizontal, continuous_x);
                                         }
                                         if vertical.pixels().is_some() {
                                            axis_frame = axis_frame.value(Axis::Vertical, continuous_y);
                                         }
                                    }
                                    pointer.axis(state, axis_frame);
                                }
                            }
                            _ => {
                                 tracing::trace!("Unhandled WinitInputEvent: {:?}", winit_event_data);
                            }
                        }
                    }
                    _ => {
                        // tracing::trace!("Other Winit event: {:?}", event);
                    }
                }
            }) {
                tracing::error!("Error dispatching winit events in winit_backend: {}", e);
                calloop_timeout_action = TimeoutAction::Break; // Exit loop on error
            }

            if calloop_timeout_action == TimeoutAction::Break {
                return TimeoutAction::Break; // Propagate break request
            }

            // --- Rendering Logic (Simplified Placeholder) ---
            // The detailed rendering logic from main.rs needs to be integrated here.
            // This involves:
            // 1. Binding the backend (self.backend.bind())
            // 2. Getting damage (state.space.damage_for_outputs())
            // 3. Iterating outputs, collecting render elements
            // 4. Calling self.renderer.render_frame_legacy_wrapper(...)
            // 5. Submitting the frame (self.backend.submit(None))
            // 6. Sending frame callbacks (state.space.send_frames(...))
            // 7. Flushing clients (state.display_handle.flush_clients())

            if let Err(e) = self.backend.bind() {
                tracing::error!("Failed to bind winit backend for rendering: {}", e);
                return calloop_timeout_action;
            }

            // Minimal rendering placeholder: clear screen to blue
            // let (w, h) = self.backend.window_size().physical_size;
            // self.renderer.clear([0.1, 0.2, 0.8, 1.0], &[smithay::utils::Rectangle::from_loc_and_size((0,0), (w,h))]);

            // This is a very simplified version of the rendering logic from main.rs
            // The actual logic involves iterating outputs, getting damage, and rendering elements.
            // For now, let's just ensure the structure is here.
            // TODO: Integrate full rendering logic from main.rs

            let damage = state.space.damage_for_outputs(&state.outputs); // Get damage
            // For simplicity, assume one main output for now or adapt main.rs logic.
            if let Some(output) = state.outputs.get(0) { // Highly simplified
                let output_geometry = state.space.output_geometry(output).unwrap_or_else(|| {
                    let fallback_size = self.backend.window_size().physical_size;
                    smithay::utils::Rectangle::from_loc_and_size((0,0), fallback_size)
                });
                let output_scale = output.current_scale().fractional_scale();

                // Collect render elements (simplified)
                let render_elements: Vec<crate::compositor::renderer_interface::abstraction::RenderElement> = Vec::new();
                // TODO: Populate render_elements like in main.rs

                match unsafe { self.renderer.render_frame_legacy_wrapper(&render_elements, output_geometry, output_scale) } {
                    Ok(_) => {} // tracing::trace!("Rendered frame (simplified)"),
                    Err(e) => tracing::error!("Error rendering frame in winit_backend: {:?}", e),
                }
            }


            if let Err(e) = self.backend.submit(None) {
                tracing::error!("Failed to submit frame via winit backend: {}", e);
            }

            state.space.damage_all_outputs(); // Request redraw for next frame

            let now_ns = state.clock.now();
            let time_for_send_frames = std::time::Duration::from_nanos(now_ns);
            state.space.send_frames(time_for_send_frames);

            if let Err(e) = self.display_handle.flush_clients() {
                tracing::warn!("Failed to flush clients in winit_backend: {}", e);
            }
            // --- End Rendering Logic Placeholder ---

            calloop_timeout_action
        }).context("Failed to insert Winit event timer into event loop")?;

        // The event_loop.run() call is made by the top-level application (e.g. in main.rs)
        // after the backend is initialized. This `run` method here sets up the winit event source.
        // The actual blocking run will be on the event_loop instance itself.
        // So, this function doesn't block; it just prepares the winit source.
        // The trait name `run` might be slightly misleading if it doesn't block.
        // However, many backend `run` methods in Smithay examples *do* block.
        // Let's clarify: this `run` method completes the setup for Winit events.
        // The main `event_loop.run()` in `main.rs` will drive this.
        Ok(())
    }

    fn loop_handle(&self) -> LoopHandle<'static, DesktopState> {
        self.event_loop_handle.clone()
    }
}

// Helper trait/extension for GlowRenderer if needed for render_frame_legacy_wrapper
// This is a temporary workaround for the GlowRenderer not having this method directly.
// Ideally, the rendering logic would be more robustly integrated.
pub trait GlowRendererExt {
    unsafe fn render_frame_legacy_wrapper(
        &mut self,
        elements: &[crate::compositor::renderer_interface::abstraction::RenderElement<'_,'_>], // Adjusted lifetime
        output_geometry: smithay::utils::Rectangle<i32, smithay::utils::Physical>,
        output_scale: f64,
    ) -> Result<(), smithay::backend::renderer::gles::GlesError>;
}

impl GlowRendererExt for GlowRenderer {
    unsafe fn render_frame_legacy_wrapper(
        &mut self,
        _elements: &[crate::compositor::renderer_interface::abstraction::RenderElement<'_,'_>],
        _output_geometry: smithay::utils::Rectangle<i32, smithay::utils::Physical>,
        _output_scale: f64,
    ) -> Result<(), smithay::backend::renderer::gles::GlesError> {
        // This is a stub. The actual rendering logic from novade-system/src/main.rs
        // (the part that calls render_elements, deals with textures, shaders etc.)
        // needs to be moved or adapted here.
        // For now, let's just clear the screen to indicate it's working.
        let screen_size = _output_geometry.size; // Assuming this is the size to clear
        self.clear([0.1, 0.2, 0.8, 1.0], &[smithay::utils::Rectangle::from_loc_and_size((0,0), screen_size)]);
        // tracing::info!("render_frame_legacy_wrapper called, cleared screen (STUBBED)");
        Ok(())
    }
}
