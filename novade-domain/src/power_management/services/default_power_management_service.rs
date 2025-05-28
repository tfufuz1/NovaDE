//! Default power management service implementation for the NovaDE domain layer.
//!
//! This module provides a thread-safe default implementation of the power management service
//! for the NovaDE desktop environment.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock}; // Keep Mutex for inhibitors if simple map is used
use async_trait::async_trait;
use uuid::Uuid; // Keep for mock inhibitor IDs if logind Inhibit is not used initially
use crate::error::{DomainError, PowerManagementError};
use super::{PowerManagementService, PowerState, BatteryState, BatteryInfo};
use crate::power_management::dbus_proxies::{
    UPowerProxy, UPowerDeviceProxy, LogindManagerProxy,
};
use zbus::{Connection, zvariant::OwnedObjectPath};
use futures_util::future::try_join_all; // For joining futures in get_batteries
use std::os::unix::io::{OwnedFd, AsRawFd}; // For logind inhibitors
use log::{error, info, warn}; // For logging

const UPOWER_STATE_CHARGING: u32 = 1;
const UPOWER_STATE_DISCHARGING: u32 = 2;
const UPOWER_STATE_EMPTY: u32 = 3;
const UPOWER_STATE_FULLY_CHARGED: u32 = 4;
const UPOWER_STATE_PENDING_CHARGE: u32 = 5;
const UPOWER_STATE_PENDING_DISCHARGE: u32 = 6;
const UPOWER_STATE_UNKNOWN: u32 = 0; // Or another value based on UPower docs

const UPOWER_TECHNOLOGY_UNKNOWN: u32 = 0;
const UPOWER_TECHNOLOGY_LITHIUM_ION: u32 = 1;
const UPOWER_TECHNOLOGY_LITHIUM_POLYMER: u32 = 2;
const UPOWER_TECHNOLOGY_LITHIUM_IRON_PHOSPHATE: u32 = 3;
const UPOWER_TECHNOLOGY_LEAD_ACID: u32 = 4;
const UPOWER_TECHNOLOGY_NICKEL_CADMIUM: u32 = 5;
const UPOWER_TECHNOLOGY_NICKEL_METAL_HYDRIDE: u32 = 6;


/// Thread-safe default implementation of the power management service using D-Bus.
pub struct DefaultPowerManagementService {
    u_power_proxy: UPowerProxy<'static>,
    logind_manager_proxy: LogindManagerProxy<'static>,
    system_bus: Connection,
    // For mock idle timeout as it's not directly on UPower/Logind in a simple way
    idle_timeout: RwLock<u64>, 
    // For mock inhibitors if logind's Inhibit is too complex for the initial step
    // If using logind inhibitors, this would be a Map of inhibitor IDs to OwnedFd
    sleep_inhibitors: Mutex<HashMap<String, OwnedFd>>, 
}

impl DefaultPowerManagementService {
    /// Creates a new default power management service.
    /// Connects to D-Bus and initializes proxies.
    pub async fn new() -> Result<Self, DomainError> {
        info!("Initializing DefaultPowerManagementService with D-Bus backend.");
        let system_bus = Connection::system()
            .await
            .map_err(|e| PowerManagementError::DbusCommunicationError(e.to_string()))?;
        
        let u_power_proxy = UPowerProxy::new(&system_bus)
            .await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("UPowerProxy init failed: {}", e)))?;
            
        let logind_manager_proxy = LogindManagerProxy::new(&system_bus)
            .await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("LogindManagerProxy init failed: {}", e)))?;

        Ok(Self {
            u_power_proxy,
            logind_manager_proxy,
            system_bus,
            idle_timeout: RwLock::new(300), // Default 5 minutes, mock
            sleep_inhibitors: Mutex::new(HashMap::new()), // Mock inhibitors
        })
    }
        
    // The with_capabilities method might be removed or changed, as capabilities
    // will now be determined by D-Bus calls.
    // For now, let's remove it. If specific overrides are needed later,
    // they can be added.
    
    /// Updates the idle time (mock implementation for now).
    pub fn update_idle_time(&self, _idle_time: u64) {
        // TODO: This should ideally use logind's IdleHint if available
        // For now, this method might not be directly usable as expected
        warn!("update_idle_time is currently a no-op with D-Bus backend pending IdleHint integration.");
    }
    
    /// Updates a battery's information (mock implementation for now).
    /// With D-Bus, data is fetched live, so this might become a no-op
    /// or used for testing with a mock D-Bus environment.
    pub fn update_battery(&self, _battery_info: BatteryInfo) {
        warn!("update_battery is currently a no-op with D-Bus backend as data is fetched live.");
    }

    // Helper to convert UPower state to domain BatteryState
    fn upower_state_to_domain(&self, state: u32, percentage: f64) -> BatteryState {
        match state {
            UPOWER_STATE_CHARGING => BatteryState::Charging,
            UPOWER_STATE_DISCHARGING => BatteryState::Discharging,
            UPOWER_STATE_EMPTY => BatteryState::Empty,
            UPOWER_STATE_FULLY_CHARGED => BatteryState::FullyCharged,
            UPOWER_STATE_PENDING_CHARGE | UPOWER_STATE_PENDING_DISCHARGE => BatteryState::Pending,
            _ => {
                // UPower's "Unknown" state might mean it's still determining.
                // If percentage is 100, it's likely fully charged.
                // This is a heuristic.
                if percentage > 99.0 { BatteryState::FullyCharged }
                else { BatteryState::Unknown }
            }
        }
    }

    // Helper to convert UPower technology to string
    fn upower_technology_to_string(&self, tech: u32) -> String {
        match tech {
            UPOWER_TECHNOLOGY_LITHIUM_ION => "Lithium Ion".to_string(),
            UPOWER_TECHNOLOGY_LITHIUM_POLYMER => "Lithium Polymer".to_string(),
            UPOWER_TECHNOLOGY_LITHIUM_IRON_PHOSPHATE => "Lithium Iron Phosphate".to_string(),
            UPOWER_TECHNOLOGY_LEAD_ACID => "Lead Acid".to_string(),
            UPOWER_TECHNOLOGY_NICKEL_CADMIUM => "Nickel Cadmium".to_string(),
            UPOWER_TECHNOLOGY_NICKEL_METAL_HYDRIDE => "Nickel Metal Hydride".to_string(),
            _ => "Unknown".to_string(),
        }
    }
}

// Implement Send and Sync explicitly to guarantee thread safety
// The proxies and connection from zbus are Send + Sync when the connection is 'static.
unsafe impl Send for DefaultPowerManagementService {}
unsafe impl Sync for DefaultPowerManagementService {}

#[async_trait]
impl PowerManagementService for DefaultPowerManagementService {
    async fn get_power_state(&self) -> Result<PowerState, DomainError> {
        // Simplification: Use UPower's OnBattery property from the display device.
        // This is not a full system power state but gives some indication.
        // A more robust solution would involve more complex logic or other D-Bus services.
        info!("Fetching power state via UPower's display device OnBattery property.");
        match self.u_power_proxy.get_display_device().await {
            Ok(display_device_path) => {
                match UPowerDeviceProxy::builder(&self.system_bus)
                    .path(display_device_path)
                    .build()
                    .await {
                        Ok(device_proxy) => {
                            match device_proxy.on_battery().await {
                                Ok(on_battery) => {
                                    if on_battery {
                                        Ok(PowerState::LowPower) // Assuming "on battery" means low power mode
                                    } else {
                                        Ok(PowerState::Running) // Plugged in
                                    }
                                }
                                Err(e) => {
                                    error!("Failed to get OnBattery property: {}", e);
                                    Err(PowerManagementError::DbusCommunicationError(format!("Failed to get OnBattery: {}", e)).into())
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to create UPowerDeviceProxy for display device: {}", e);
                            Err(PowerManagementError::DbusCommunicationError(format!("UPowerDeviceProxy for display device failed: {}", e)).into())
                        }
                    }
            }
            Err(e) => {
                warn!("Could not get display device from UPower, defaulting to PowerState::Running. Error: {}", e);
                // Fallback or error. For now, let's assume Running if we can't determine.
                Ok(PowerState::Running)
            }
        }
    }
    
    async fn get_batteries(&self) -> Result<Vec<BatteryInfo>, DomainError> {
        info!("Fetching battery list from UPower.");
        let device_paths = self.u_power_proxy.enumerate_devices().await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("EnumerateDevices failed: {}", e)))?;

        let mut battery_futures = Vec::new();
        for path in device_paths {
            // Filter for actual battery devices by path (UPower convention)
            if path.as_str().contains("/battery_") {
                let system_bus_clone = self.system_bus.clone(); // Clone for async block
                battery_futures.push(async move {
                    let device_proxy = UPowerDeviceProxy::builder(&system_bus_clone)
                        .path(path.clone())?
                        .build()
                        .await
                        .map_err(|e| PowerManagementError::DbusCommunicationError(format!("UPowerDeviceProxy for {} failed: {}", path.as_str(), e)))?;
                    
                    // Fetch all properties concurrently (minor optimization, zbus might do this)
                    let (
                        percentage, state, time_to_empty, time_to_full, 
                        vendor, model, technology, is_present, icon_name
                    ) = futures::try_join!(
                        device_proxy.percentage(),
                        device_proxy.state(),
                        device_proxy.time_to_empty(),
                        device_proxy.time_to_full(),
                        device_proxy.vendor(),
                        device_proxy.model(),
                        device_proxy.technology(),
                        device_proxy.is_present(),
                        device_proxy.icon_name()
                    ).map_err(|e| PowerManagementError::DbusCommunicationError(format!("Failed to get props for {}: {}", path.as_str(), e)))?;

                    // Ensure device is present before including it
                    if !is_present {
                        return Ok(None);
                    }
                    
                    let id = path.as_str().split('/').last().unwrap_or("unknown_bat").to_string();

                    Ok(Some(BatteryInfo {
                        id,
                        state: self.upower_state_to_domain(state, percentage),
                        percentage,
                        time_to_empty: if time_to_empty > 0 { Some(time_to_empty) } else { None },
                        time_to_full: if time_to_full > 0 { Some(time_to_full) } else { None },
                        is_present,
                        vendor: Some(vendor),
                        model: Some(model),
                        serial: None, // UPower typically doesn't provide serial easily
                        technology: Some(self.upower_technology_to_string(technology)),
                        capacity: None, // UPower provides energy, not direct capacity in Ah
                        energy: None, // TODO: UPower has 'Energy' property, map if needed
                        energy_full: None, // TODO: UPower has 'EnergyFull'
                        energy_full_design: None, // TODO: UPower has 'EnergyFullDesign'
                        voltage: None, // TODO: UPower has 'Voltage'
                        properties: HashMap::new(), // Can be extended if needed
                    }))
                });
            }
        }

        let results = try_join_all(battery_futures).await?;
        Ok(results.into_iter().filter_map(|x| x).collect())
    }
    
    async fn get_battery(&self, battery_id: &str) -> Result<BatteryInfo, DomainError> {
        info!("Fetching battery info for ID: {}", battery_id);
        // UPower identifies devices by their object path.
        // The `battery_id` here should correspond to the last segment of that path.
        // e.g., "BAT0" -> "/org/freedesktop/UPower/devices/battery_BAT0"
        // Or, if the full path is passed, use that.
        
        let expected_suffix = format!("/battery_{}", battery_id); // Standard UPower naming
        let alternative_path = battery_id; // If full path is given

        let device_paths = self.u_power_proxy.enumerate_devices().await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("EnumerateDevices failed: {}", e)))?;

        for path in device_paths {
            if path.as_str().ends_with(&expected_suffix) || path.as_str() == alternative_path {
                let device_proxy = UPowerDeviceProxy::builder(&self.system_bus)
                    .path(path.clone())?
                    .build()
                    .await
                    .map_err(|e| PowerManagementError::DbusCommunicationError(format!("UPowerDeviceProxy for {} failed: {}", path.as_str(), e)))?;

                let (
                    percentage, state, time_to_empty, time_to_full, 
                    vendor, model, technology, is_present, icon_name
                ) = futures::try_join!(
                    device_proxy.percentage(),
                    device_proxy.state(),
                    device_proxy.time_to_empty(),
                    device_proxy.time_to_full(),
                    device_proxy.vendor(),
                    device_proxy.model(),
                    device_proxy.technology(),
                    device_proxy.is_present(),
                    device_proxy.icon_name()
                ).map_err(|e| PowerManagementError::DbusCommunicationError(format!("Failed to get props for {}: {}", path.as_str(), e)))?;

                if !is_present {
                    return Err(PowerManagementError::DeviceNotFound(battery_id.to_string()).into());
                }

                let id = path.as_str().split('/').last().unwrap_or("unknown_bat").to_string();

                return Ok(BatteryInfo {
                    id,
                    state: self.upower_state_to_domain(state, percentage),
                    percentage,
                    time_to_empty: if time_to_empty > 0 { Some(time_to_empty) } else { None },
                    time_to_full: if time_to_full > 0 { Some(time_to_full) } else { None },
                    is_present,
                    vendor: Some(vendor),
                    model: Some(model),
                    serial: None,
                    technology: Some(self.upower_technology_to_string(technology)),
                    capacity: None,
                    energy: None,
                    energy_full: None,
                    energy_full_design: None,
                    voltage: None,
                    properties: HashMap::new(),
                });
            }
        }
        Err(PowerManagementError::BatteryNotFound(battery_id.to_string()).into())
    }
    
    async fn suspend(&self) -> Result<(), DomainError> {
        info!("Attempting to suspend system via logind.");
        self.logind_manager_proxy.suspend(false).await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("Suspend failed: {}", e)).into())
    }
    
    async fn hibernate(&self) -> Result<(), DomainError> {
        info!("Attempting to hibernate system via logind.");
        self.logind_manager_proxy.hibernate(false).await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("Hibernate failed: {}", e)).into())
    }
    
    async fn shutdown(&self) -> Result<(), DomainError> {
        info!("Attempting to power off system via logind.");
        self.logind_manager_proxy.power_off(false).await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("PowerOff failed: {}", e)).into())
    }
    
    async fn restart(&self) -> Result<(), DomainError> {
        info!("Attempting to reboot system via logind.");
        self.logind_manager_proxy.reboot(false).await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("Reboot failed: {}", e)).into())
    }
    
    async fn can_suspend(&self) -> Result<bool, DomainError> {
        info!("Checking if suspend is possible via logind.");
        let res = self.logind_manager_proxy.can_suspend().await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("CanSuspend failed: {}", e)))?;
        Ok(res == "yes")
    }
    
    async fn can_hibernate(&self) -> Result<bool, DomainError> {
        info!("Checking if hibernate is possible via logind.");
        let res = self.logind_manager_proxy.can_hibernate().await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("CanHibernate failed: {}", e)))?;
        Ok(res == "yes")
    }
    
    async fn can_shutdown(&self) -> Result<bool, DomainError> {
        info!("Checking if power off is possible via logind.");
        let res = self.logind_manager_proxy.can_power_off().await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("CanPowerOff failed: {}", e)))?;
        Ok(res == "yes")
    }
    
    async fn can_restart(&self) -> Result<bool, DomainError> {
        info!("Checking if reboot is possible via logind.");
        let res = self.logind_manager_proxy.can_reboot().await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("CanReboot failed: {}", e)))?;
        Ok(res == "yes")
    }
    
    async fn get_idle_time(&self) -> Result<u64, DomainError> {
        // TODO: Implement using logind.Manager.IdleHint if available.
        // This requires checking if the property exists and getting its value.
        // For now, returning a mock value.
        warn!("get_idle_time is returning mock value (0), pending logind IdleHint integration.");
        Ok(0) // Mock value
    }
    
    async fn set_idle_timeout(&self, timeout: u64) -> Result<(), DomainError> {
        // This is not directly settable via UPower/logind in a simple way.
        // It usually involves session managers or power plugins of desktop environments.
        // Storing it locally for now.
        info!("Setting idle timeout (mock implementation) to: {} seconds", timeout);
        *self.idle_timeout.write().unwrap() = timeout;
        Ok(())
    }
    
    async fn get_idle_timeout(&self) -> Result<u64, DomainError> {
        info!("Getting idle timeout (mock implementation).");
        Ok(*self.idle_timeout.read().unwrap())
    }
    
    async fn inhibit_sleep(&self, reason: &str) -> Result<String, DomainError> {
        // Using logind's Inhibit mechanism.
        // "sleep" refers to suspend and hibernate.
        // "novade" is a placeholder for application name.
        // The returned FD must be kept open.
        info!("Attempting to inhibit sleep via logind for reason: {}", reason);
        // TODO: Use a proper application name
        let fd = self.logind_manager_proxy
            .inhibit("sleep", "NovaDE", reason, "block") 
            .await
            .map_err(|e| PowerManagementError::DbusCommunicationError(format!("Inhibit failed: {}", e)))?;
        
        let inhibitor_id = Uuid::new_v4().to_string();
        // Store the FD. When the FD is dropped (closed), the inhibit lock is released.
        self.sleep_inhibitors.lock().unwrap().insert(inhibitor_id.clone(), fd);
        info!("Sleep inhibited with ID: {}", inhibitor_id);
        Ok(inhibitor_id)
    }
    
    async fn uninhibit_sleep(&self, inhibitor_id: &str) -> Result<(), DomainError> {
        info!("Attempting to uninhibit sleep for ID: {}", inhibitor_id);
        let mut inhibitors = self.sleep_inhibitors.lock().unwrap();
        if let Some(fd) = inhibitors.remove(inhibitor_id) {
            // Dropping the fd will close it and release the inhibitor.
            drop(fd);
            info!("Sleep uninhibited for ID: {}", inhibitor_id);
            Ok(())
        } else {
            warn!("Inhibitor ID not found: {}", inhibitor_id);
            Err(PowerManagementError::InhibitorNotFound(inhibitor_id.to_string()).into())
        }
    }
    
    async fn get_sleep_inhibitors(&self) -> Result<HashMap<String, String>, DomainError> {
        // The current logind Inhibit mechanism doesn't easily allow listing reasons for active FDs
        // held by ourselves without more complex tracking.
        // This will return a list of our active inhibitor IDs, but the "reason"
        // would need to be stored separately if we want to return it here.
        // For now, returning IDs with a placeholder reason.
        info!("Getting sleep inhibitors (IDs only).");
        let inhibitors = self.sleep_inhibitors.lock().unwrap();
        let result = inhibitors
            .keys()
            .map(|id| (id.clone(), "Inhibited by NovaDE".to_string())) // Placeholder reason
            .collect();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::power_management::{BatteryState, PowerState}; // Ensure enums are in scope

    // --- Documentation for Running Integration Tests ---
    // The tests below marked with `#[ignore]` are D-Bus integration tests.
    // They require a live D-Bus environment with active UPower and systemd-logind services.
    //
    // To run these tests:
    // 1. Ensure UPower (org.freedesktop.UPower) and systemd-logind (org.freedesktop.login1)
    //    are running and accessible on the system D-Bus.
    // 2. Execute the tests using:
    //    `cargo test --package novade-domain --lib power_management::services::default_power_management_service::tests -- --ignored`
    //    Or, to run a specific test:
    //    `cargo test --package novade-domain --lib power_management::services::default_power_management_service::tests specific_test_name_dbus -- --ignored`
    //    (Replace `specific_test_name_dbus` with the actual test name, e.g., `test_get_batteries_dbus`)
    //
    // Expectations & Potential Issues:
    // - Some tests (like `test_get_batteries_dbus` or `test_get_specific_battery_dbus`)
    //   are more meaningful if the system has at least one battery. They will still pass
    //   if no batteries are present (returning empty Vec or DeviceNotFound error), but assertions
    //   about battery properties won't be deeply tested.
    // - Network connection or D-Bus misconfiguration can cause these tests to fail.
    // - `test_sleep_inhibitors_dbus` actively creates and removes a sleep inhibitor via logind.
    //   This is generally safe but modifies system state temporarily.
    // - Tests involving actual power state changes (suspend, hibernate, shutdown, restart)
    //   are NOT executed by default. `test_power_actions_dry_run_dbus` is a placeholder
    //   and does not perform these actions. Executing such actions should be done manually
    //   and with caution as they are disruptive. Tests for `can_...` methods are safe.
    //
    // If tests fail, check:
    // - D-Bus daemon status (`systemctl status dbus`)
    // - UPower service status (`systemctl status upower`)
    // - Logind service status (`systemctl status systemd-logind`)
    // - D-Bus permissions for the user running the tests.
    // - Logs from the test execution for specific D-Bus errors.

    // Helper function to create a service instance for D-Bus tests.
    // Panics if D-Bus connection fails, which is acceptable for these integration tests.
    async fn new_dbus_test_service() -> DefaultPowerManagementService {
        // Allow a bit more time for D-Bus to connect in CI/constrained environments
        match tokio::time::timeout(std::time::Duration::from_secs(15), DefaultPowerManagementService::new()).await {
            Ok(Ok(service)) => service,
            Ok(Err(e)) => panic!("Failed to connect to D-Bus for testing: {:?}. Ensure D-Bus, UPower, and Logind are running.", e),
            Err(_) => panic!("Timed out connecting to D-Bus for testing. Ensure D-Bus, UPower, and Logind are running."),
        }
    }

    // --- Unit Tests ---
    #[test]
    fn test_upower_state_to_domain() {
        let service = DefaultPowerManagementService { // Dummy instance for helper method
            u_power_proxy: unsafe { std::mem::zeroed() },
            logind_manager_proxy: unsafe { std::mem::zeroed() },
            system_bus: unsafe { std::mem::zeroed() },
            idle_timeout: RwLock::new(0),
            sleep_inhibitors: Mutex::new(HashMap::new()),
        };

        assert_eq!(service.upower_state_to_domain(UPOWER_STATE_CHARGING, 50.0), BatteryState::Charging);
        assert_eq!(service.upower_state_to_domain(UPOWER_STATE_DISCHARGING, 50.0), BatteryState::Discharging);
        assert_eq!(service.upower_state_to_domain(UPOWER_STATE_EMPTY, 0.0), BatteryState::Empty);
        assert_eq!(service.upower_state_to_domain(UPOWER_STATE_FULLY_CHARGED, 100.0), BatteryState::FullyCharged);
        assert_eq!(service.upower_state_to_domain(UPOWER_STATE_PENDING_CHARGE, 98.0), BatteryState::Pending);
        assert_eq!(service.upower_state_to_domain(UPOWER_STATE_PENDING_DISCHARGE, 10.0), BatteryState::Pending);
        assert_eq!(service.upower_state_to_domain(UPOWER_STATE_UNKNOWN, 50.0), BatteryState::Unknown);
        assert_eq!(service.upower_state_to_domain(UPOWER_STATE_UNKNOWN, 100.0), BatteryState::FullyCharged); // Heuristic
         assert_eq!(service.upower_state_to_domain(99, 50.0), BatteryState::Unknown); // Arbitrary other value
    }

    #[test]
    fn test_upower_technology_to_string() {
        let service = DefaultPowerManagementService { // Dummy instance for helper method
            u_power_proxy: unsafe { std::mem::zeroed() },
            logind_manager_proxy: unsafe { std::mem::zeroed() },
            system_bus: unsafe { std::mem::zeroed() },
            idle_timeout: RwLock::new(0),
            sleep_inhibitors: Mutex::new(HashMap::new()),
        };
        assert_eq!(service.upower_technology_to_string(UPOWER_TECHNOLOGY_LITHIUM_ION), "Lithium Ion");
        assert_eq!(service.upower_technology_to_string(UPOWER_TECHNOLOGY_LITHIUM_POLYMER), "Lithium Polymer");
        assert_eq!(service.upower_technology_to_string(UPOWER_TECHNOLOGY_LITHIUM_IRON_PHOSPHATE), "Lithium Iron Phosphate");
        assert_eq!(service.upower_technology_to_string(UPOWER_TECHNOLOGY_LEAD_ACID), "Lead Acid");
        assert_eq!(service.upower_technology_to_string(UPOWER_TECHNOLOGY_NICKEL_CADMIUM), "Nickel Cadmium");
        assert_eq!(service.upower_technology_to_string(UPOWER_TECHNOLOGY_NICKEL_METAL_HYDRIDE), "Nickel Metal Hydride");
        assert_eq!(service.upower_technology_to_string(UPOWER_TECHNOLOGY_UNKNOWN), "Unknown");
        assert_eq!(service.upower_technology_to_string(99), "Unknown"); // Arbitrary other value
    }
    
    #[tokio::test]
    async fn test_idle_timeout_mock_logic() { // Renamed to clarify it's testing mock logic
        let service = DefaultPowerManagementService {
            u_power_proxy: unsafe { std::mem::zeroed() }, 
            logind_manager_proxy: unsafe { std::mem::zeroed() }, 
            system_bus: unsafe {std::mem::zeroed() }, 
            idle_timeout: RwLock::new(100), // Initial mock value
            sleep_inhibitors: Mutex::new(HashMap::new()),
        };

        assert_eq!(service.get_idle_timeout().await.unwrap(), 100);
        service.set_idle_timeout(600).await.unwrap();
        assert_eq!(service.get_idle_timeout().await.unwrap(), 600);
        // get_idle_time is mocked to return 0, so no specific logic to test there without D-Bus.
        assert_eq!(service.get_idle_time().await.unwrap(), 0);
    }

    // --- D-Bus Integration Tests (Marked with #[ignore]) ---
    #[tokio::test]
    #[ignore] 
    async fn test_get_power_state_dbus() { // Renamed from _real to _dbus
        let service = new_dbus_test_service().await;
        match service.get_power_state().await {
            Ok(state) => {
                info!("D-Bus Power state: {:?}", state);
                // Assert it's one of the known states. Actual state depends on the test system.
                assert!(matches!(state, PowerState::Running | PowerState::LowPower | PowerState::Unknown), "Unexpected power state: {:?}", state);
            }
            Err(e) => {
                // UPower might not be fully available or display device not found in some CI.
                // Log warning and accept if it's a communication or device not found error.
                warn!("test_get_power_state_dbus: Could not get power state: {:?}. This might be okay in environments without full UPower.", e);
                assert!(matches!(e, DomainError::PowerManagement(PowerManagementError::DbusCommunicationError(_)) | DomainError::PowerManagement(PowerManagementError::DeviceNotFound(_))), "Unexpected error type: {:?}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_batteries_dbus() { // Renamed
        let service = new_dbus_test_service().await;
        match service.get_batteries().await {
            Ok(batteries) => {
                info!("D-Bus Found {} batteries.", batteries.len());
                for battery in &batteries {
                    info!("Battery ID: {}, Percentage: {}, State: {:?}", battery.id, battery.percentage, battery.state);
                    assert!(battery.percentage >= 0.0 && battery.percentage <= 100.0, "Battery percentage out of range: {}", battery.percentage);
                    // Basic check for ID
                    assert!(!battery.id.is_empty(), "Battery ID is empty");
                }
                // It's valid for a system to have 0 batteries.
                // No assertion on batteries.len() > 0.
            }
            Err(e) => {
                panic!("test_get_batteries_dbus failed: {:?}. Check UPower service.", e);
            }
        }
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_get_specific_battery_dbus() { // Renamed and clarified
        let service = new_dbus_test_service().await;
        let batteries = service.get_batteries().await.unwrap_or_else(|e| {
            warn!("Could not list batteries for test_get_specific_battery_dbus setup: {:?}. Test might be inconclusive.", e);
            vec![]
        });

        if let Some(first_battery) = batteries.first() {
            info!("Testing get_battery with actual ID: {}", first_battery.id);
            match service.get_battery(&first_battery.id).await {
                Ok(battery) => {
                    assert_eq!(battery.id, first_battery.id);
                    assert!(battery.percentage >= 0.0 && battery.percentage <= 100.0);
                    info!("Successfully fetched battery by ID: {:?}", battery);
                }
                Err(e) => {
                    panic!("test_get_specific_battery_dbus failed for existing ID {}: {:?}", first_battery.id, e);
                }
            }
        } else {
            info!("No batteries found on system, skipping specific battery fetch part of test_get_specific_battery_dbus.");
            // If no batteries, test that querying a fake ID gives DeviceNotFound
            let result = service.get_battery("NONEXISTENT_BATTERY_ID_12345_XYZ").await;
            info!("Result for non-existent battery: {:?}", result);
            assert!(matches!(result, Err(DomainError::PowerManagement(PowerManagementError::DeviceNotFound(_)))), "Expected DeviceNotFound for non-existent battery, got: {:?}", result);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_battery_not_found_dbus() { // Renamed and updated error
        let service = new_dbus_test_service().await;
        let result = service.get_battery("NONEXISTENT_BATTERY_ID_12345_ABCDEFG").await;
        info!("Result for non-existent battery in test_get_battery_not_found_dbus: {:?}", result);
        assert!(matches!(result, Err(DomainError::PowerManagement(PowerManagementError::DeviceNotFound(_)))), "Expected DeviceNotFound, got {:?}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_capabilities_dbus() { // Renamed
        let service = new_dbus_test_service().await;
        // These calls should succeed, returning true or false based on system config.
        // We are mostly testing that the D-Bus call itself doesn't fail.
        // Asserting the result is Ok is the primary goal.
        assert!(service.can_suspend().await.is_ok(), "can_suspend() call failed");
        assert!(service.can_hibernate().await.is_ok(), "can_hibernate() call failed");
        assert!(service.can_shutdown().await.is_ok(), "can_shutdown() call failed");
        assert!(service.can_restart().await.is_ok(), "can_restart() call failed");
        
        info!("CanSuspend: {:?}", service.can_suspend().await.unwrap());
        info!("CanHibernate: {:?}", service.can_hibernate().await.unwrap());
        info!("CanShutdown: {:?}", service.can_shutdown().await.unwrap());
        info!("CanReboot: {:?}", service.can_restart().await.unwrap());
    }

    #[tokio::test]
    #[ignore] 
    async fn test_power_actions_dry_run_dbus() { // Renamed
        // This test remains a "dry run" and does not execute actual power actions.
        // It primarily serves as a placeholder to indicate where such manual tests would go.
        let service = new_dbus_test_service().await;
        info!("--- Power Actions Dry Run (No actual operations performed) ---");
        if service.can_suspend().await.unwrap_or(false) {
            info!("System reports: CanSuspend = true. (Suspend not called)");
            // Test actual suspend manually with:
            // tokio::runtime::Runtime::new().unwrap().block_on(async {
            //     let service = new_dbus_test_service().await;
            //     service.suspend().await.expect("Manual suspend call failed");
            // });
        } else {
            info!("System reports: CanSuspend = false.");
        }
        // Similar checks for hibernate, shutdown, restart could be added.
        // Actual calls should remain commented out for automated tests.
        info!("--- End of Power Actions Dry Run ---");
    }
    
    #[tokio::test]
    #[ignore]
    async fn test_sleep_inhibitors_dbus() { // Renamed and updated error
        let service = new_dbus_test_service().await;
        let reason = "Automated Integration Test Inhibitor";
        
        // Ensure system supports suspend before trying to inhibit it for a more robust test
        if !service.can_suspend().await.unwrap_or(false) {
            info!("System cannot suspend. Skipping test_sleep_inhibitors_dbus as inhibit might behave unexpectedly.");
            return;
        }

        let inhibitor_id = service.inhibit_sleep(reason).await.expect("Failed to inhibit sleep via D-Bus");
        info!("D-Bus Sleep inhibitor ID: {}", inhibitor_id);
        assert!(!inhibitor_id.is_empty(), "Inhibitor ID should not be empty");
        
        let inhibitors = service.get_sleep_inhibitors().await.expect("Failed to get inhibitors via D-Bus");
        assert!(inhibitors.contains_key(&inhibitor_id), "Newly created inhibitor not found in list");
        assert_eq!(inhibitors.get(&inhibitor_id).unwrap(), "Inhibited by NovaDE", "Inhibitor reason mismatch");

        // We cannot programmatically verify that suspend is *actually* blocked without trying to suspend,
        // which is disruptive. The D-Bus call succeeding is the main check here.
        // Logind itself is responsible for honoring the inhibitor.

        service.uninhibit_sleep(&inhibitor_id).await.expect("Failed to uninhibit sleep via D-Bus");
        let inhibitors_after = service.get_sleep_inhibitors().await.expect("Failed to get inhibitors after uninhibit");
        assert!(!inhibitors_after.contains_key(&inhibitor_id), "Inhibitor still present after uninhibiting");
        
        // Test uninhibit non-existent
        let random_id = Uuid::new_v4().to_string();
        let uninhib_res = service.uninhibit_sleep(&random_id).await;
        info!("Result for uninhibit non-existent ID ({}): {:?}", random_id, uninhib_res);
        assert!(matches!(uninhib_res, Err(DomainError::PowerManagement(PowerManagementError::InhibitorNotFound(_)))), "Expected InhibitorNotFound, got: {:?}", uninhib_res);
    }
}
