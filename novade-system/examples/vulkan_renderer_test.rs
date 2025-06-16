// novade-system/examples/vulkan_renderer_test.rs

// ANCHOR: Use Statements
use novade_system::renderers::vulkan::NovaVulkanRenderer;
use std::ffi::c_void;
use std::thread;
use std::time::Duration;

// ANCHOR: Main Function
fn main() {
    println!("Starting NovaVulkanRenderer test...");

    // ANCHOR_EXT: Renderer Initialization
    // Using null pointers for Wayland handles as this is a non-displaying test.
    // This will likely cause surface/swapchain creation to fail, but we want to test
    // how much of the renderer can initialize and if it handles such failures gracefully.
    let mut renderer = match NovaVulkanRenderer::new(std::ptr::null_mut() as *mut c_void, std::ptr::null_mut() as *mut c_void) {
        Ok(r) => {
            println!("NovaVulkanRenderer created successfully.");
            r
        }
        Err(e) => {
            eprintln!("Failed to create NovaVulkanRenderer: {}", e);
            // Depending on the error, some parts of VulkanContext might still be alive.
            // For this test, we'll exit if the core renderer can't be made.
            return;
        }
    };

    // ANCHOR_EXT: Swapchain Initialization (Attempt 1)
    let initial_width = 800;
    let initial_height = 600;
    println!("\nAttempting initial swapchain initialization ({}x{})...", initial_width, initial_height);
    match renderer.init_swapchain(initial_width, initial_height) {
        Ok(()) => {
            println!("Initial swapchain initialization successful.");

            // ANCHOR_EXT: Render Loop (Attempt 1)
            println!("\nStarting render loop (10 frames)...");
            for i in 0..10 {
                print!("Rendering frame {}... ", i + 1);
                match renderer.render_frame() {
                    Ok(()) => println!("Done."),
                    Err(e) => {
                        eprintln!("Error during render_frame: {}", e);
                        // If render_frame fails (e.g. swapchain out of date and recreation not fully implemented)
                        // we might want to break the loop or attempt recreation here.
                        // For this test, we'll log and continue to see if it recovers or consistently fails.
                        if e.contains("Swapchain out of date") {
                             println!("Swapchain out of date, attempting recreation for next frame simulation might be needed.");
                             // In a real app, signal for swapchain recreation.
                        }
                    }
                }
                // thread::sleep(Duration::from_millis(16)); // Simulate frame delay
            }
            println!("Render loop finished.");

        }
        Err(e) => {
            eprintln!("Failed to initialize swapchain (initial attempt): {}", e);
            println!("This is expected if Wayland handles are null and surface creation fails.");
            println!("Skipping render loop and resize test due to swapchain initialization failure.");
        }
    }


    // ANCHOR_EXT: Swapchain Recreation Test (Attempt 2, if initial succeeded)
    // This section will only be meaningful if the first init_swapchain was successful.
    // If surface creation is the blocker, this will also likely fail or be skipped.
    if renderer.swapchain.is_some() { // Check if swapchain was successfully created
        let new_width = 1024;
        let new_height = 768;
        println!("\nAttempting swapchain recreation ({}x{})...", new_width, new_height);
        match renderer.init_swapchain(new_width, new_height) {
            Ok(()) => {
                println!("Swapchain recreation successful.");

                println!("\nStarting render loop after recreation (5 frames)...");
                for i in 0..5 {
                    print!("Rendering frame {} (post-recreation)... ", i + 1);
                    match renderer.render_frame() {
                        Ok(()) => println!("Done."),
                        Err(e) => {
                            eprintln!("Error during render_frame (post-recreation): {}", e);
                             if e.contains("Swapchain out of date") {
                                println!("Swapchain out of date, attempting recreation for next frame simulation might be needed.");
                            }
                        }
                    }
                    // thread::sleep(Duration::from_millis(16));
                }
                println!("Render loop (post-recreation) finished.");
            }
            Err(e) => {
                eprintln!("Failed to recreate swapchain: {}", e);
            }
        }
    } else {
        println!("\nSkipping swapchain recreation test as initial swapchain was not created.");
    }


    // ANCHOR_EXT: Cleanup
    // `renderer` goes out of scope here, its `Drop` implementation will be called.
    println!("\nTest finished. Renderer cleanup will now occur via Drop.");
}
