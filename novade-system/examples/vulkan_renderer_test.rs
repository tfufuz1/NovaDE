// novade-system/examples/vulkan_renderer_test.rs

use novade_system::compositor::state::{DesktopState, RendererType};
use novade_system::compositor::renderer_interface::abstraction::{RenderElement, FrameRenderer};
use smithay::reexports::calloop::EventLoop;
use smithay::reexports::wayland_server::Display;
use smithay::utils::{Rectangle, Point, Size};

fn main() {
    println!("Starting unified renderer test for Vulkan backend...");

    // Basic setup for DesktopState
    let mut event_loop: EventLoop<'static, DesktopState> = EventLoop::try_new().unwrap();
    let mut display: Display<DesktopState> = Display::new().unwrap();
    let mut desktop_state = DesktopState::new(&mut event_loop, &mut display);

    // Initialize the Vulkan renderer
    if let Err(e) = desktop_state.init_renderer_backend(RendererType::Vulkan) {
        eprintln!("Failed to initialize Vulkan renderer backend: {}", e);
        eprintln!("This test requires a functional Vulkan environment.");
        return;
    }
    println!("Vulkan renderer backend initialized (conceptually).");

    if let Some(renderer) = desktop_state.renderer.as_mut() {
        println!("Renderer ID: {}", renderer.id());

        let output_geometry = Rectangle::from_loc_and_size(Point::from((0, 0)), Size::from((800, 600)));
        let output_scale = 1.0;

        // Create a simple scene with one solid color element
        let elements = vec![RenderElement::SolidColor {
            color: [1.0, 0.0, 0.0, 1.0], // Red
            geometry: Rectangle::from_loc_and_size(Point::from((100, 100)), Size::from((200, 200))),
        }];

        println!("\nAttempting to render a single frame...");
        let render_result = renderer.render_frame(elements, output_geometry, output_scale);

        match render_result {
            Ok(_) => {
                println!("render_frame call succeeded.");
                let present_result = renderer.submit_and_present_frame();
                match present_result {
                    Ok(_) => println!("submit_and_present_frame call succeeded."),
                    Err(e) => {
                        eprintln!("submit_and_present_frame call failed: {}", e);
                        println!("This may be expected in a headless environment without a swapchain.");
                    }
                }
            }
            Err(e) => {
                eprintln!("render_frame call failed: {}", e);
            }
        }
    } else {
        eprintln!("Renderer was not initialized in DesktopState.");
    }

    println!("\nTest finished.");
}
