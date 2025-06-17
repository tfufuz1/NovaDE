// novade-system/examples/test_upower_client.rs

use std::sync::Arc;
use anyhow::Result;
use tracing_subscriber::{EnvFilter, fmt};

// Ensure correct imports from your crate structure
use novade_system::dbus_clients::upower_client::{UPowerClient, UPowerDeviceData, ClientError};
use novade_system::dbus_integration::DbusServiceManager;

// Helper function to map UPower device type enum to string
fn map_device_type_to_string(dev_type: Option<u32>) -> String {
    match dev_type {
        Some(0) => "Unknown".to_string(),
        Some(1) => "Line Power".to_string(),
        Some(2) => "Battery".to_string(),
        Some(3) => "UPS".to_string(),
        Some(4) => "Monitor".to_string(),
        Some(5) => "Mouse".to_string(),
        Some(6) => "Keyboard".to_string(),
        Some(7) => "PDA".to_string(),
        Some(8) => "Phone".to_string(),
        Some(9) => "Gaming Input".to_string(),
        Some(10) => "Bluetooth Generic".to_string(),
        Some(11) => "Tablet".to_string(),
        None => "N/A".to_string(),
        _ => "Other/Undefined".to_string(),
    }
}

// Helper function to map UPower device state enum to string
fn map_device_state_to_string(state: Option<u32>) -> String {
    match state {
        Some(0) => "Unknown".to_string(),
        Some(1) => "Charging".to_string(),
        Some(2) => "Discharging".to_string(),
        Some(3) => "Empty".to_string(),
        Some(4) => "Fully Charged".to_string(),
        Some(5) => "Pending Charge".to_string(),
        Some(6) => "Pending Discharge".to_string(),
        None => "N/A".to_string(),
        _ => "Other/Undefined".to_string(),
    }
}


// ANCHOR: MainFunctionForUPowerClientExample
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing subscriber for logging
    fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive("novade_system=info".parse()?))
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting UPowerClient example...");

    // 1. Instantiate DbusServiceManager (wrapped in Arc for UPowerClient)
    let dbus_manager = Arc::new(DbusServiceManager::new().await.map_err(|e| {
        tracing::error!("Failed to create DbusServiceManager: {}", e);
        anyhow::anyhow!("D-Bus manager creation failed: {}", e)
    })?);
    tracing::info!("DbusServiceManager initialized.");

    // 2. Instantiate UPowerClient
    let upower_client = UPowerClient::new(dbus_manager);
    tracing::info!("UPowerClient initialized.");

    // 3. List Devices
    match upower_client.list_devices().await {
        Ok(devices) => {
            if devices.is_empty() {
                tracing::info!("No UPower devices found.");
            } else {
                tracing::info!("Found {} UPower devices:", devices.len());
                for device_path in devices {
                    tracing::info!("--------------------------------------------------");
                    tracing::info!("Device Path: {}", device_path.as_str());

                    // 4. Get and Print Device Details (structured)
                    match upower_client.get_device_details(&device_path).await {
                        Ok(details) => {
                            println!("  Path: {}", details.path.as_str());
                            println!("  Type: {}", map_device_type_to_string(details.type_));
                            if let Some(vendor) = details.vendor {
                                if !vendor.is_empty() { println!("  Vendor: {}", vendor); }
                            }
                            if let Some(model) = details.model {
                                if !model.is_empty() { println!("  Model: {}", model); }
                            }
                            if let Some(icon) = details.icon_name {
                                if !icon.is_empty() {println!("  Icon Name: {}", icon); }
                            }
                            if let Some(percentage) = details.percentage {
                                println!("  Percentage: {:.2}%", percentage);
                            }
                            println!("  State: {}", map_device_state_to_string(details.state));
                            if let Some(rechargeable) = details.is_rechargeable {
                                println!("  Rechargeable: {}", rechargeable);
                            }
                            if let Some(online) = details.line_power_online {
                                println!("  Line Power Online: {}", online); // Relevant for AC adapters
                            }
                            if let Some(time_to_full) = details.time_to_full {
                                if details.state == Some(1) && time_to_full > 0 { // Charging
                                    println!("  Time to Full: {} seconds", time_to_full);
                                }
                            }
                            if let Some(time_to_empty) = details.time_to_empty {
                                 if details.state == Some(2) && time_to_empty > 0 { // Discharging
                                    println!("  Time to Empty: {} seconds", time_to_empty);
                                 }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to get details for device {}: {}", device_path.as_str(), e);
                        }
                    }

                    // 5. Get and Print All Raw Properties (for debugging/completeness)
                    // tracing::info!("  Raw Properties for {}:", device_path.as_str());
                    // match upower_client.get_device_properties_raw(&device_path).await {
                    //     Ok(props) => {
                    //         for (key, value) in props {
                    //             tracing::info!("    {}: {:?}", key, value);
                    //         }
                    //     }
                    //     Err(e) => {
                    //         tracing::error!("  Failed to get raw properties for device {}: {}", device_path.as_str(), e);
                    //     }
                    // }
                }
                tracing::info!("--------------------------------------------------");
            }
        }
        Err(e) => {
            tracing::error!("Failed to list UPower devices: {}", e);
            if matches!(e, ClientError::ZbusFailed(ref zbus_err) if zbus_err.name() == Some("org.freedesktop.DBus.Error.ServiceUnknown")) {
                tracing::warn!("Hint: Ensure that the UPower service (upowerd) is installed and running on your system.");
            }
        }
    }

    tracing::info!("UPowerClient example finished.");
    Ok(())
}
