// novade-system/examples/vulkan_init_main.rs

// Updated use path for VulkanCoreContext
use novade_system::renderer::vulkan::VulkanCoreContext;
// If VulkanError variants were matched, this would be needed:
// use novade_system::renderer::vulkan::error::VulkanError; 
use std::error::Error; // For source()

fn main() {
    // Initialize logger, e.g., env_logger. Set default log level to info if RUST_LOG is not set.
    // Ensure RUST_LOG can be used, e.g., RUST_LOG=novade_system::renderer::vulkan=debug,info
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,novade_system::renderer::vulkan=info")
    ).init();

    println!("Attempting to initialize NovaDE Vulkan Core Context (from novade-system example)...");
    log::info!("Main (vulkan_init_main): Attempting to initialize NovaDE Vulkan Core Context...");

    match VulkanCoreContext::new() {
        Ok(core_context) => {
            println!("\nNovaDE Vulkan Core Context initialized successfully!");
            log::info!("Main (vulkan_init_main): NovaDE Vulkan Core Context initialized successfully!");

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
            log::info!("Main (vulkan_init_main): Example finished successfully.");

        }
        Err(e) => {
            // The error 'e' is of type novade_system::renderer::vulkan::error::VulkanError
            println!("\nError initializing Vulkan Core Context: {}", e);
            log::error!("Main (vulkan_init_main): Error initializing Vulkan Core Context: {}", e);
            let mut current_err: Option<&dyn Error> = Some(&e); // e is already a VulkanError
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
