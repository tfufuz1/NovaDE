// novade-system/examples/run_echo_service.rs

use std::sync::Arc;
use anyhow::Result;
use zbus::zvariant::ObjectPath;
use tracing_subscriber::{EnvFilter, fmt};

// Import ObjectManager as well
use novade_system::dbus_integration::{DbusServiceManager, EchoService, ObjectManager};

const SERVICE_NAME: &str = "org.novade.ExampleEchoService";
// Define the root path for the service where ObjectManager will reside
const SERVICE_ROOT_PATH_STR: &str = "/org/novade/ExampleEchoService";
// Define the path for the main Echo interface
const ECHO_MAIN_INTERFACE_PATH_STR: &str = "/org/novade/ExampleEchoService/Main";


// ANCHOR: MainFunctionForExample
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive("novade_system=info".parse()?))
        .with_writer(std::io.stderr)
        .init();

    tracing::info!("Starting ExampleEchoService D-Bus service with ObjectManager...");

    // 1. Instantiate DbusServiceManager
    let dbus_manager = DbusServiceManager::new().await?;
    tracing::info!("DbusServiceManager initialized.");

    // 2. Request a D-Bus Name
    dbus_manager.request_name(SERVICE_NAME).await?;
    tracing::info!("Successfully requested D-Bus name: {}", SERVICE_NAME);

    // 3. Create and Serve ObjectManager
    let service_root_path = ObjectPath::try_from(SERVICE_ROOT_PATH_STR)
        .expect("Failed to create valid ObjectPath for service root.");
    let object_manager_instance = Arc::new(ObjectManager::new(
        dbus_manager.system_bus(),
        service_root_path.clone(),
    ));
    dbus_manager.serve_at(object_manager_instance.clone(), &service_root_path).await?;
    tracing::info!("ObjectManager served at {}", service_root_path);

    // 4. Instantiate EchoService, providing it with the ObjectManager
    let echo_main_path = ObjectPath::try_from(ECHO_MAIN_INTERFACE_PATH_STR)
        .expect("Failed to create valid ObjectPath for EchoService main interface.");

    let echo_service_instance = Arc::new(EchoService::new(
        dbus_manager.system_bus(),
        echo_main_path.clone(),
        object_manager_instance.clone(), // Pass the ObjectManager Arc
    ));
    tracing::info!("EchoService instance created for path: {}", echo_main_path);

    // 5. Serve the EchoService Custom Interface
    dbus_manager.serve_at(echo_service_instance.clone(), &echo_main_path).await?;
    tracing::info!("EchoService custom interface served at {}", echo_main_path);

    // 6. Serve the Properties Interface for EchoService
    let properties_handler = echo_service_instance.properties_handler();
    dbus_manager.serve_at(properties_handler, &echo_main_path).await?;
    tracing::info!("Properties interface served at {} for EchoService", echo_main_path);

    // 7. Register EchoService's main interface with ObjectManager
    echo_service_instance.register_with_object_manager().await?;
    tracing::info!("EchoService main interface registered with ObjectManager.");

    // 8. Demonstrate adding and removing sub-objects via ObjectManager
    tracing::info!("Demonstrating ObjectManager by adding sub-objects...");
    let sub_obj1_path = echo_service_instance.add_simple_sub_object("obj1", "First Sub-Object".to_string()).await?;
    tracing::info!("Added sub-object: {}", sub_obj1_path);
    let sub_obj2_path = echo_service_instance.add_simple_sub_object("obj2", "Second Sub-Object".to_string()).await?;
    tracing::info!("Added sub-object: {}", sub_obj2_path);

    // Example: remove one sub-object
    // tokio::time::sleep(tokio::time::Duration::from_secs(5)).await; // Optional delay
    echo_service_instance.remove_simple_sub_object("obj1").await?;
    tracing::info!("Removed sub-object for 'obj1'.");


    // 9. Keep the Service Running
    tracing::info!("ExampleEchoService is now running. Press Ctrl+C to exit.");
    tracing::info!("You can now interact with the service using D-Bus tools like busctl.");
    tracing::info!("---");
    tracing::info!("Example busctl commands (assuming system bus):");
    tracing::info!("# To introspect the main Echo interface:");
    tracing::info!("# busctl --system introspect {} {}", SERVICE_NAME, ECHO_MAIN_INTERFACE_PATH_STR);
    tracing::info!("# To call EchoString:");
    tracing::info!("# busctl --system call {} {} {}.Echo EchoString s \"Hello from busctl\"", SERVICE_NAME, ECHO_MAIN_INTERFACE_PATH_STR, SERVICE_NAME);
    tracing::info!("# To get a property (e.g., Prefix) from Echo interface:");
    tracing::info!("# busctl --system get-property {} {} {}.Echo Prefix", SERVICE_NAME, ECHO_MAIN_INTERFACE_PATH_STR, SERVICE_NAME);
    tracing::info!("# To get all properties for the Echo interface:");
    tracing::info!("# busctl --system call {} {} org.freedesktop.DBus.Properties GetAll s \"{}.Echo\"", SERVICE_NAME, ECHO_MAIN_INTERFACE_PATH_STR, SERVICE_NAME);
    tracing::info!("# To get managed objects (from ObjectManager):");
    tracing::info!("# busctl --system call {} {} org.freedesktop.DBus.ObjectManager GetManagedObjects", SERVICE_NAME, SERVICE_ROOT_PATH_STR);
    tracing::info!("# To monitor signals (run in another terminal to see InterfacesAdded/Removed):");
    tracing::info!("# busctl --system monitor {}", SERVICE_NAME);
    tracing::info!("---");

    // Use tokio's signal handling to wait for Ctrl+C
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            tracing::info!("Ctrl+C received, shutting down service.");
        }
        Err(err) => {
            tracing::error!("Failed to listen for Ctrl+C signal: {}", err);
        }
    }

    // Note: ObjectServerGuards in DbusServiceManager will be dropped when dbus_manager goes out of scope,
    // automatically unregistering the objects.

    Ok(())
}
