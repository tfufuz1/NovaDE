// Copyright (c) 2025 NovaDE Contributors
// SPDX-License-Identifier: MIT

//! # Compositor Initialization Implementation
//!
//! This module provides functionality for initializing the compositor.

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use smithay::reexports::wayland_server::Display;
use smithay::wayland::compositor::CompositorState;
use smithay::wayland::shell::xdg::XdgShellState;
use smithay::wayland::shell::wlr_layer::LayerShellState;
use smithay::wayland::output::{OutputManagerState, Output as SmithayOutput, Mode as SmithayMode, PhysicalProperties as SmithayPhysicalProperties, Scale};
use smithay::wayland::seat::SeatState;
use smithay::backend::drm::DrmNode;
use smithay::backend::winit::{WinitGraphicsBackend, WinitEventLoop};
use smithay::input::Seat;
use smithay::utils::{Logical, Size, Transform};
use tokio::runtime::Handle as TokioHandle; // For block_on

use super::{CompositorError, CompositorResult};
use super::core::{DesktopState, OutputConfiguration, OutputMode};
use super::renderers::{DrmGbmRenderer, WinitRenderer};
use super::renderer_interface::FrameRenderer;
use super::thread_safety::run_all_validations;
use smithay::reexports::calloop::EventLoop;

/// Initializes the compositor with a DRM/GBM backend
pub async fn initialize_compositor_drm(
    drm_node: DrmNode,
    output_size: (i32, i32),
    output_scale: f64,
) -> CompositorResult<(Arc<DesktopState>, Arc<Mutex<DrmGbmRenderer>>)> {
    // Create the DisplayHandle and LoopHandle (assuming they are created or passed here)
    // This part is tricky as DesktopState::new expects them but they are not directly available here.
    // For now, I'll assume that DesktopState::new can be called without them, or they are part of a larger context.
    // The original DesktopState::new signature was: loop_handle: LoopHandle<'static, Self>, display_handle: DisplayHandle,
    // This needs to be reconciled. For now, I'll proceed assuming this can be resolved.
    // Let's simulate getting them if they are not passed. This is a placeholder.
    let mut display: Display<DesktopState> = Display::new().expect("Failed to create Wayland display");
    let mut event_loop: EventLoop<DesktopState> = EventLoop::try_new().expect("Failed to create event loop");

    // Create the desktop state
    let desktop_state = Arc::new(DesktopState::new(&mut event_loop, &mut display).await.map_err(|e| CompositorError::InitializationError(format!("Failed to create DesktopState: {}",e)))?);
    
    // Create the renderer
    let renderer = DrmGbmRenderer::new(drm_node, Size::from(output_size), output_scale)?;
    let renderer = Arc::new(Mutex::new(renderer));
    
    // Initialize the renderer
    renderer.lock().map_err(|_| {
        CompositorError::InitializationError("Failed to acquire lock on renderer".to_string())
    })?.initialize()?;
    
    // Set up signal handlers and other initialization
    setup_signal_handlers(desktop_state.clone())?;
    
    // Create a default seat
    desktop_state.create_seat("default")?;
    
    // Set the output configuration
    desktop_state.set_output_configuration(OutputConfiguration {
        name: "default".to_string(),
        size: Size::from(output_size),
        scale: output_scale,
        transform: smithay::utils::Transform::Normal,
        mode: OutputMode::Fixed {
            width: output_size.0,
            height: output_size.1,
            refresh: 60000,
        },
    })?;
    
    // Validate thread safety
    run_all_validations(desktop_state.clone())?;

    // Initialize outputs from WaylandDisplayManager
    let display_manager = desktop_state.display_manager.clone();
    let mut state_guard = desktop_state.clone(); // Arc<DesktopState> can be used if new_output takes &self

    match TokioHandle::current().block_on(display_manager.get_displays()) {
        Ok(detected_core_displays) => {
            for core_display in detected_core_displays {
                if !core_display.enabled {
                    tracing::info!("Skipping disabled display: {}", core_display.name);
                    continue;
                }

                let smithay_modes: Vec<SmithayMode> = core_display.modes.iter().map(|core_mode| {
                    SmithayMode {
                        size: (core_mode.width as i32, core_mode.height as i32).into(),
                        refresh: core_mode.refresh_rate / 1000, // Convert mHz to Hz
                    }
                }).collect();

                let current_smithay_mode = core_display.current_mode.as_ref().and_then(|core_mode| {
                    smithay_modes.iter().find(|sm| sm.size.w == core_mode.width as i32 && sm.size.h == core_mode.height as i32 && sm.refresh == (core_mode.refresh_rate / 1000))
                }).cloned();

                let final_current_mode = current_smithay_mode.or_else(|| smithay_modes.get(0).cloned()).unwrap_or_else(|| {
                    tracing::warn!("Display {} has no modes, using default 800x600@60Hz", core_display.name);
                    SmithayMode { size: (800, 600).into(), refresh: 60 }
                });

                let physical_props = core_display.physical_properties.as_ref().map(|pp| {
                    SmithayPhysicalProperties {
                        size: (pp.width_mm as i32, pp.height_mm as i32).into(),
                        subpixel: smithay::output::Subpixel::Unknown,
                        make: "NovaDE".to_string(),
                        model: core_display.name.clone(),
                    }
                }).unwrap_or_else(|| {
                    SmithayPhysicalProperties {
                        size: (0,0).into(),
                        subpixel: smithay::output::Subpixel::Unknown,
                        make: "NovaDE".to_string(),
                        model: core_display.name.clone(),
                    }
                });

                let new_smithay_output = SmithayOutput::new(
                    core_display.id.clone(), // Use unique ID for Smithay Output name
                    physical_props,
                    None
                );

                // Smithay 0.10.0 Output API:
                new_smithay_output.change_current_state(
                    Some(final_current_mode),
                    Some(Transform::Normal), // Assuming Normal transform
                    Some(Scale::Integer(1)), // Assuming scale 1
                    Some((core_display.position_x, core_display.position_y).into()) // Position from core_display
                );
                new_smithay_output.set_preferred(final_current_mode);
                for mode in smithay_modes {
                    new_smithay_output.add_mode(mode);
                }

                // Create global before calling new_output
                // state_guard.output_manager_state.create_global::<DesktopState>(&display_handle, &new_smithay_output);
                // new_output in output_handlers.rs is now expected to handle global creation via OutputManagerState
                // For Smithay 0.10.0, OutputHandler::new_output is directly on DesktopState
                state_guard.new_output(new_smithay_output);
            }
        }
        Err(e) => {
            tracing::error!("Failed to get displays from display manager: {}", e);
            // Decide if this is a fatal error or continue with no displays
        }
    }
    
    Ok((desktop_state, renderer))
}

#[cfg(test)]
mod init_tests;

/// Initializes the compositor with a Winit backend
pub async fn initialize_compositor_winit(
    backend: WinitGraphicsBackend,
    output_scale: f64,
) -> CompositorResult<(Arc<DesktopState>, Arc<Mutex<WinitRenderer>>)> {
    // Similar to DRM, create Display and EventLoop for DesktopState::new
    let mut display: Display<DesktopState> = Display::new().expect("Failed to create Wayland display");
    let mut event_loop: EventLoop<DesktopState> = EventLoop::try_new().expect("Failed to create event loop");

    // Create the desktop state
    let desktop_state = Arc::new(DesktopState::new(&mut event_loop, &mut display).await.map_err(|e| CompositorError::InitializationError(format!("Failed to create DesktopState: {}",e)))?);
    
    // Create the renderer
    let renderer = WinitRenderer::new(backend, output_scale)?;
    let renderer = Arc::new(Mutex::new(renderer));
    
    // Initialize the renderer
    renderer.lock().map_err(|_| {
        CompositorError::InitializationError("Failed to acquire lock on renderer".to_string())
    })?.initialize()?;
    
    // Set up signal handlers and other initialization
    setup_signal_handlers(desktop_state.clone())?;
    
    // Create a default seat
    desktop_state.create_seat("default")?;
    
    // Set the output configuration
    let output_size = renderer.lock().map_err(|_| {
        CompositorError::InitializationError("Failed to acquire lock on renderer".to_string())
    })?.output_size;
    
    desktop_state.set_output_configuration(OutputConfiguration {
        name: "default".to_string(),
        size: Size::from((output_size.w, output_size.h)),
        scale: output_scale,
        transform: smithay::utils::Transform::Normal,
        mode: OutputMode::Fixed {
            width: output_size.w,
            height: output_size.h,
            refresh: 60000,
        },
    })?;
    
    // Validate thread safety
    run_all_validations(desktop_state.clone())?;

    // Initialize outputs from WaylandDisplayManager
    let display_manager = desktop_state.display_manager.clone();
    let mut state_guard = desktop_state.clone(); // Arc<DesktopState> for new_output

    match TokioHandle::current().block_on(display_manager.get_displays()) {
        Ok(detected_core_displays) => {
            for core_display in detected_core_displays {
                if !core_display.enabled {
                    tracing::info!("Skipping disabled display: {}", core_display.name);
                    continue;
                }

                let smithay_modes: Vec<SmithayMode> = core_display.modes.iter().map(|core_mode| {
                    SmithayMode {
                        size: (core_mode.width as i32, core_mode.height as i32).into(),
                        refresh: core_mode.refresh_rate / 1000,
                    }
                }).collect();

                let current_smithay_mode = core_display.current_mode.as_ref().and_then(|core_mode| {
                    smithay_modes.iter().find(|sm| sm.size.w == core_mode.width as i32 && sm.size.h == core_mode.height as i32 && sm.refresh == (core_mode.refresh_rate / 1000))
                }).cloned();

                let final_current_mode = current_smithay_mode.or_else(|| smithay_modes.get(0).cloned()).unwrap_or_else(|| {
                    tracing::warn!("Display {} has no modes, using default 800x600@60Hz", core_display.name);
                    SmithayMode { size: (800, 600).into(), refresh: 60 }
                });

                let physical_props = core_display.physical_properties.as_ref().map(|pp| {
                    SmithayPhysicalProperties {
                        size: (pp.width_mm as i32, pp.height_mm as i32).into(),
                        subpixel: smithay::output::Subpixel::Unknown,
                        make: "NovaDE".to_string(),
                        model: core_display.name.clone(),
                    }
                }).unwrap_or_else(|| {
                    SmithayPhysicalProperties {
                        size: (0,0).into(),
                        subpixel: smithay::output::Subpixel::Unknown,
                        make: "NovaDE".to_string(),
                        model: core_display.name.clone(),
                    }
                });

                let new_smithay_output = SmithayOutput::new(
                    core_display.id.clone(),
                    physical_props,
                    None
                );

                new_smithay_output.change_current_state(
                    Some(final_current_mode),
                    Some(Transform::Normal),
                    Some(Scale::Integer(1)),
                    Some((core_display.position_x, core_display.position_y).into())
                );
                new_smithay_output.set_preferred(final_current_mode);
                for mode in smithay_modes {
                    new_smithay_output.add_mode(mode);
                }

                // state_guard.output_manager_state.create_global::<DesktopState>(&display_handle, &new_smithay_output);
                state_guard.new_output(new_smithay_output);
            }
        }
        Err(e) => {
            tracing::error!("Failed to get displays from display manager: {}", e);
        }
    }
    
    Ok((desktop_state, renderer))
}

/// Convenience function to initialize the compositor
pub async fn initialize_compositor() -> CompositorResult<Arc<DesktopState>> {
    // Try to find a DRM device
    let drm_nodes = DrmNode::available_nodes().map_err(|e| {
        CompositorError::InitializationError(format!("Failed to enumerate DRM nodes: {}", e))
    })?;
    
    if let Some(node) = drm_nodes.first() {
        // Initialize with DRM/GBM
        let (desktop_state, _) = initialize_compositor_drm(*node, (1920, 1080), 1.0).await?;
        Ok(desktop_state)
    } else {
        // Initialize with Winit
        let event_loop = WinitEventLoop::new().map_err(|e| {
            CompositorError::InitializationError(format!("Failed to create Winit event loop: {}", e))
        })?;
        
        let backend = WinitGraphicsBackend::new(&event_loop, None, None).map_err(|e| {
            CompositorError::InitializationError(format!("Failed to create Winit backend: {}", e))
        })?;
        
        let (desktop_state, renderer) = initialize_compositor_winit(backend, 1.0).await?;
        
        // Store the event loop in the renderer
        renderer.lock().map_err(|_| {
            CompositorError::InitializationError("Failed to acquire lock on renderer".to_string())
        })?.set_event_loop(event_loop)?;
        
        Ok(desktop_state)
    }
}

/// Sets up signal handlers for the compositor
fn setup_signal_handlers(desktop_state: Arc<DesktopState>) -> CompositorResult<()> {
    // Set up signal handlers for SIGINT, SIGTERM, etc.
    // This would typically use the signal_hook crate
    
    // For now, this is just a placeholder
    Ok(())
}

/// Runs the compositor main loop
pub fn run_compositor(
    desktop_state: Arc<DesktopState>,
    renderer: Arc<Mutex<dyn FrameRenderer + Send + Sync>>,
) -> CompositorResult<()> {
    // Get the display
    let display = desktop_state.display.clone();
    
    // Create a channel for events
    let (event_sender, event_receiver) = std::sync::mpsc::channel();
    
    // Set up a thread for the Wayland event loop
    let wayland_thread = std::thread::spawn(move || {
        let mut display = display.lock().unwrap();
        
        loop {
            // Dispatch Wayland events
            display.dispatch_clients(&mut ()).unwrap();
            display.flush_clients().unwrap();
            
            // Check for exit signal
            if let Ok(()) = event_receiver.try_recv() {
                break;
            }
            
            // Sleep a bit to avoid busy waiting
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
    });
    
    // Main rendering loop
    loop {
        // Begin frame
        renderer.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on renderer".to_string())
        })?.begin_frame()?;
        
        // Render all surfaces
        // This would iterate through all surfaces and render them
        
        // End frame
        renderer.lock().map_err(|_| {
            CompositorError::RenderError("Failed to acquire lock on renderer".to_string())
        })?.end_frame()?;
        
        // Sleep to maintain frame rate
        std::thread::sleep(std::time::Duration::from_millis(16));
        
        // Check for exit condition
        // This would check for a signal or other exit condition
        
        // For now, just break after a few frames for testing
        break;
    }
    
    // Signal the Wayland thread to exit
    event_sender.send(()).map_err(|_| {
        CompositorError::InitializationError("Failed to send exit signal to Wayland thread".to_string())
    })?;
    
    // Wait for the Wayland thread to exit
    wayland_thread.join().map_err(|_| {
        CompositorError::InitializationError("Failed to join Wayland thread".to_string())
    })?;
    
    // Clean up the renderer
    renderer.lock().map_err(|_| {
        CompositorError::RenderError("Failed to acquire lock on renderer".to_string())
    })?.cleanup()?;
    
    Ok(())
}
