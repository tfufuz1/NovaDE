// novade-system/tests/echo_service_integration_test.rs

use anyhow::Result;
use novade_system::dbus_integration::{DbusServiceManager, EchoService, ObjectManager};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use zbus::{dbus_proxy, fdo::PropertiesProxy, Connection, Proxy};
use zbus::zvariant::{ObjectPath, OwnedObjectPath, Value, Dict};
use tracing_subscriber::{EnvFilter, fmt};

// ANCHOR: TestConstantsAndProxies
const TEST_SERVICE_NAME: &str = "org.novade.IntegrationTest.EchoService";
const SERVICE_ROOT_PATH_STR: &str = "/org/novade/integrationtest/echoservice";
const ECHO_MAIN_PATH_STR: &str = "/org/novade/integrationtest/echoservice/Main";

#[dbus_proxy(
    interface = "org.novade.ExampleEchoService.Echo",
    default_service = "org.novade.IntegrationTest.EchoService",
    default_path = "/org/novade/integrationtest/echoservice/Main"
)]
trait EchoServiceTestProxy {
    async fn echo_string(&self, input_string: String) -> zbus::Result<String>;
    // If EchoService had properties directly on its main interface (not via Properties interface)
    // #[dbus_proxy(property)]
    // async fn echo_count(&self) -> zbus::Result<i64>;
}

// Helper function to initialize tracing only once.
fn init_tracing() {
    let _ = fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive("novade_system=info".parse().unwrap()))
        .with_test_writer() // Write to test output
        .try_init(); // Use try_init to avoid panic if already initialized
}

// ANCHOR: SetupServiceTaskHelper
async fn setup_service_task() -> Result<tokio::task::JoinHandle<Result<(), anyhow::Error>>, anyhow::Error> {
    let service_task = tokio::spawn(async move {
        let dbus_manager = DbusServiceManager::new_session().await?;
        dbus_manager.request_name(TEST_SERVICE_NAME).await?;

        let root_path = ObjectPath::try_from(SERVICE_ROOT_PATH_STR).unwrap();
        let main_path = ObjectPath::try_from(ECHO_MAIN_PATH_STR).unwrap();

        let object_manager = Arc::new(ObjectManager::new(dbus_manager.connection(), root_path.clone()));
        let echo_service = Arc::new(EchoService::new(
            dbus_manager.connection(),
            main_path.clone(),
            object_manager.clone(),
        ));

        dbus_manager.serve_at(object_manager.clone(), &root_path).await?;
        dbus_manager.serve_at(echo_service.clone(), &main_path).await?;
        dbus_manager.serve_at(echo_service.properties_handler(), &main_path).await?;

        echo_service.register_with_object_manager().await?;
        echo_service.add_simple_sub_object("initial_sub", "Initial Label".to_string()).await?;

        // Keep the service alive indefinitely for the test
        std::future::pending::<()>().await;
        Ok(())
    });
    // Give the service a moment to start up and register its name
    tokio::time::sleep(Duration::from_millis(500)).await;
    Ok(service_task)
}


// ANCHOR: TestCallEchoStringAndProperties
#[tokio::test]
async fn test_call_echo_string_and_properties() -> Result<()> {
    init_tracing();
    let service_task = setup_service_task().await?;

    let client_conn = Connection::session().await?;

    let echo_proxy = EchoServiceTestProxy::builder(&client_conn).build().await?;

    // Call EchoString
    let response = echo_proxy.echo_string("World".to_string()).await?;
    assert_eq!(response, "Echo: World");

    // Check EchoCount property via Properties interface
    let props_proxy = PropertiesProxy::builder(&client_conn)
        .destination(TEST_SERVICE_NAME)?
        .path(ECHO_MAIN_PATH_STR)?
        .build()
        .await?;

    let echo_count_val = props_proxy.get("org.novade.ExampleEchoService.Echo", "EchoCount").await?;
    assert_eq!(echo_count_val, Value::from(1i64));

    // Call EchoString again
    let response2 = echo_proxy.echo_string("Again".to_string()).await?;
    assert_eq!(response2, "Echo: Again");

    // Check EchoCount property again
    let echo_count_val2 = props_proxy.get("org.novade.ExampleEchoService.Echo", "EchoCount").await?;
    assert_eq!(echo_count_val2, Value::from(2i64));

    // Check Prefix property
    let prefix_val = props_proxy.get("org.novade.ExampleEchoService.Echo", "Prefix").await?;
    assert_eq!(prefix_val, Value::from("Echo: "));

    service_task.abort();
    let _ = service_task.await; // Ensure task is fully cleaned up
    Ok(())
}

// ANCHOR: TestPropertiesChangedSignal
#[tokio::test]
async fn test_properties_changed_signal() -> Result<()> {
    init_tracing();
    let service_task = setup_service_task().await?;

    let client_conn = Connection::session().await?;

    let props_proxy = PropertiesProxy::builder(&client_conn)
        .destination(TEST_SERVICE_NAME)?
        .path(ECHO_MAIN_PATH_STR)?
        .build()
        .await?;

    let mut signal_stream = props_proxy.receive_properties_changed().await?;

    let echo_proxy = EchoServiceTestProxy::builder(&client_conn).build().await?;
    echo_proxy.echo_string("Signal Test".to_string()).await?; // This should trigger EchoCount change

    match timeout(Duration::from_secs(2), signal_stream.next()).await {
        Ok(Some(signal_args)) => {
            assert_eq!(signal_args.interface_name(), "org.novade.ExampleEchoService.Echo");
            let changed_props = signal_args.changed_properties();
            assert!(changed_props.contains_key("EchoCount"));
            assert_eq!(changed_props.get("EchoCount").unwrap(), &Value::from(1i64)); // EchoCount becomes 1 after first call by test setup + this call
        }
        Ok(None) => anyhow::bail!("Signal stream ended unexpectedly."),
        Err(_) => anyhow::bail!("Timeout waiting for PropertiesChanged signal."),
    }

    service_task.abort();
    let _ = service_task.await;
    Ok(())
}

// ANCHOR: TestObjectManagerGetManagedObjects
#[tokio::test]
async fn test_object_manager_get_managed_objects() -> Result<()> {
    init_tracing();
    let service_task = setup_service_task().await?; // setup_service_task already adds an "initial_sub"

    let client_conn = Connection::session().await?;

    let om_proxy = zbus::fdo::ObjectManagerProxy::builder(&client_conn)
        .destination(TEST_SERVICE_NAME)?
        .path(SERVICE_ROOT_PATH_STR)?
        .build()
        .await?;

    let managed_objects = om_proxy.get_managed_objects().await?;

    // Expected paths
    let main_echo_obj_path = ObjectPath::try_from(ECHO_MAIN_PATH_STR)?;
    let initial_sub_obj_path = ObjectPath::try_from(format!("{}/sub/initial_sub", SERVICE_ROOT_PATH_STR))?;

    // Check main echo service object
    assert!(managed_objects.contains_key(&main_echo_obj_path));
    let main_echo_interfaces = managed_objects.get(&main_echo_obj_path).unwrap();
    assert!(main_echo_interfaces.contains_key("org.novade.ExampleEchoService.Echo"));
    assert!(main_echo_interfaces.contains_key("org.freedesktop.DBus.Properties"));
    let echo_props = main_echo_interfaces.get("org.novade.ExampleEchoService.Echo").unwrap();
    assert!(echo_props.contains_key("Prefix"));
    assert_eq!(echo_props.get("Prefix").unwrap().downcast_ref::<String>().unwrap(), "Echo: ");


    // Check initial sub object
    assert!(managed_objects.contains_key(&initial_sub_obj_path));
    let sub_obj_interfaces = managed_objects.get(&initial_sub_obj_path).unwrap();
    assert!(sub_obj_interfaces.contains_key("org.novade.ExampleEchoService.SubObjectData"));
    let sub_obj_props = sub_obj_interfaces.get("org.novade.ExampleEchoService.SubObjectData").unwrap();
    assert!(sub_obj_props.contains_key("Label"));
    assert_eq!(sub_obj_props.get("Label").unwrap().downcast_ref::<String>().unwrap(), "Initial Label");

    // There should be at least these two objects.
    // The ObjectManager itself is not listed in its GetManagedObjects output.
    assert!(managed_objects.len() >= 2);


    service_task.abort();
    let _ = service_task.await;
    Ok(())
}
