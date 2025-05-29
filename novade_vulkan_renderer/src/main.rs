// main.rs
use novade_vulkan_renderer::VulkanCoreContext; // Use the new central context
use std::error::Error; // For source()

fn main() {
    // Initialize logger, e.g., env_logger. Set default log level to info if RUST_LOG is not set.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("Attempting to initialize NovaDE Vulkan Core Context...");
    log::info!("Main: Attempting to initialize NovaDE Vulkan Core Context...");

    match VulkanCoreContext::new() {
        Ok(core_context) => {
            println!("\nNovaDE Vulkan Core Context initialized successfully!");
            log::info!("Main: NovaDE Vulkan Core Context initialized successfully!");

            println!("  Instance API Version: {}", core_context.instance.api_version());
            log::debug!("  Instance API Version: {}", core_context.instance.api_version());

            println!("  Selected Physical Device: {}", core_context.physical_device.properties().device_name);
            log::debug!("  Selected Physical Device: {} (Type: {:?})", 
                core_context.physical_device.properties().device_name,
                core_context.physical_device.properties().device_type);

            println!("  Graphics Queue Family Index: {:?}", core_context.queue_family_indices.graphics_family);
            log::debug!("  Graphics Queue Family Index: {:?}", core_context.queue_family_indices.graphics_family);
            
            println!("  Present Queue Family Index: {:?}", core_context.queue_family_indices.present_family);
            log::debug!("  Present Queue Family Index: {:?}", core_context.queue_family_indices.present_family);

            println!("  Graphics Queue: Family Index {}, ID within family {}", 
                core_context.graphics_queue.queue_family_index(), 
                core_context.graphics_queue.id_within_family()
            );
            log::debug!("  Graphics Queue: Family Index {}, ID within family {}, Vulkano ID: {:?}", 
                core_context.graphics_queue.queue_family_index(), 
                core_context.graphics_queue.id_within_family(),
                core_context.graphics_queue.id()
            );

            println!("  Present Queue: Family Index {}, ID within family {}", 
                core_context.present_queue.queue_family_index(), 
                core_context.present_queue.id_within_family()
            );
            log::debug!("  Present Queue: Family Index {}, ID within family {}, Vulkano ID: {:?}", 
                core_context.present_queue.queue_family_index(), 
                core_context.present_queue.id_within_family(),
                core_context.present_queue.id()
            );
            
            // Further application logic would go here, using core_context
            println!("\nApplication would continue using the VulkanCoreContext.");

        }
        Err(e) => {
            println!("\nError initializing Vulkan Core Context: {}", e);
            log::error!("Main: Error initializing Vulkan Core Context: {}", e);
            let mut current_err: Option<&dyn Error> = Some(&e);
            let mut indent_level = 1;
            while let Some(source_err) = current_err.and_then(Error::source) {
                let indent = "  ".repeat(indent_level);
                println!("{}Caused by: {}", indent, source_err);
                log::error!("{}Caused by: {}", indent, source_err);
                current_err = Some(source_err);
                indent_level += 1;
            }
        }
    }
}
