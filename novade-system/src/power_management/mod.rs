//! Power management module for the NovaDE system layer.
//!
//! This module provides power management functionality for the NovaDE desktop environment,
//! handling power-related features like sleep and shutdown via systemd-logind.

use async_trait::async_trait;
use std::sync::Arc; // Mutex removed from here, used by tokio::sync::Mutex if needed for LogindProxyHandler state
use std::time::Duration;
use crate::error::{SystemError, SystemResult, SystemErrorKind}; // to_system_error might not be needed
use zbus::{Proxy, Connection as ZbusConnection, SignalContext};
use zbus::zvariant::Value;
// use crate::dbus_integration::manager::DbusManagerError; // Using SystemError for now
use futures_util::stream::StreamExt;
use novade_domain::power_management::{PowerPolicyService, PowerEvent};
use novade_domain::DomainServices;
use tokio::sync::mpsc; // For internal channel if signals are complex

// Placeholder for actual domain event or callback
// This might be a channel sender or a direct callback to a domain service.
// For now, let's use a simple function callback placeholder.
type PrepareSignalCallback = Box<dyn Fn(bool) + Send + Sync>;

struct LogindProxyHandler {
    connection: Arc<ZbusConnection>,
    logind_proxy: Proxy<'static>,
    // For signal handling if SystemPowerManager owns the handler
    // prepare_for_shutdown_callback: Option<Arc<PrepareSignalCallback>>,
    // prepare_for_sleep_callback: Option<Arc<PrepareSignalCallback>>,
}

impl LogindProxyHandler {
    const LOGIND_DEST: &'static str = "org.freedesktop.login1";
    const LOGIND_PATH: &'static str = "/org/freedesktop/login1";
    const LOGIND_MANAGER_IFACE: &'static str = "org.freedesktop.login1.Manager";

    async fn new(connection: Arc<ZbusConnection>) -> SystemResult<Self> {
        tracing::info!("Initializing LogindProxyHandler...");
        let logind_proxy = Proxy::new(
            connection.as_ref(),
            Self::LOGIND_DEST,
            Self::LOGIND_PATH,
            Self::LOGIND_MANAGER_IFACE,
        )
        .await
        .map_err(|e| SystemError::new(SystemErrorKind::DBus, format!("Failed to create logind proxy: {}", e)))?;
        tracing::info!("Logind D-Bus proxy created successfully.");
        Ok(Self { connection, logind_proxy })
    }

    async fn can_verb(&self, verb: &str) -> SystemResult<bool> {
        let result: SystemResult<String> = self.logind_proxy
            .call_method(verb, &())
            .await
            .map_err(|e| SystemError::new(SystemErrorKind::DBus, format!("Logind {} call failed: {}", verb, e)))
            .and_then(|reply| {
                reply.body::<String>()
                     .map_err(|e| SystemError::new(SystemErrorKind::DBus, format!("Failed to parse {} reply: {}", verb, e)))
            });

        match result {
            Ok(s) => Ok(s == "yes" || s == "challenge"),
            Err(e) => {
                tracing::warn!("Failed to query logind capability '{}': {}. Assuming not supported.", verb, e);
                Ok(false)
            }
        }
    }

    async fn power_action_interactive(&self, action_method_name: &str, interactive: bool) -> SystemResult<()> {
        self.logind_proxy
            .call_method(action_method_name, &(interactive))
            .await
            .map_err(|e| SystemError::new(SystemErrorKind::DBus, format!("Logind {} call failed: {}", action_method_name, e)))?;
        Ok(())
    }

    async fn start_signal_listener(
        &self,
        domain_services: Arc<DomainServices>,
    ) -> SystemResult<()> {
        tracing::info!("LogindProxyHandler: Starting signal listeners for PrepareForShutdown and PrepareForSleep...");

        let mut pfs_stream = self.logind_proxy
            .receive_signal("PrepareForShutdown")
            .await
            .map_err(|e| SystemError::new(SystemErrorKind::DBus, format!("Failed to subscribe to PrepareForShutdown: {}",e)))?;

        let domain_services_clone_pfs = domain_services.clone();
        tokio::spawn(async move {
            tracing::debug!("Logind PrepareForShutdown signal listener started.");
            while let Some(signal) = pfs_stream.next().await {
                match signal.body::<(bool,)>() {
                    Ok((active,)) => {
                        tracing::info!("Received PrepareForShutdown signal (active: {})", active);
                        // Example: domain_services_clone_pfs.power_policy_service().handle_system_event(PowerEvent::PrepareShutdown(active)).await;
                        // Actual call to domain service method would go here.
                    }
                    Err(e) => {
                        tracing::error!("Error deserializing PrepareForShutdown signal: {}", e);
                    }
                }
            }
            tracing::warn!("Logind PrepareForShutdown signal stream ended.");
        });

        let mut pfsleep_stream = self.logind_proxy
            .receive_signal("PrepareForSleep")
            .await
            .map_err(|e| SystemError::new(SystemErrorKind::DBus, format!("Failed to subscribe to PrepareForSleep: {}",e)))?;

        // let domain_services_clone_pfsleep = domain_services.clone(); // domain_services already cloned or moved by previous spawn
        tokio::spawn(async move {
            tracing::debug!("Logind PrepareForSleep signal listener started.");
            while let Some(signal) = pfsleep_stream.next().await {
                 match signal.body::<(bool,)>() {
                    Ok((active,)) => {
                        tracing::info!("Received PrepareForSleep signal (active: {})", active);
                        // Example: domain_services.power_policy_service().handle_system_event(PowerEvent::PrepareSleep(active)).await;
                    }
                    Err(e) => {
                        tracing::error!("Error deserializing PrepareForSleep signal: {}", e);
                    }
                }
            }
            tracing::warn!("Logind PrepareForSleep signal stream ended.");
        });

        tracing::info!("LogindProxyHandler: Signal listeners started.");
        Ok(())
    }
}


/// Power state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    /// The system is running on AC power.
    AC,
    /// The system is running on battery power.
    Battery,
    /// The system is running on low battery power.
    LowBattery,
    /// The system is running on critical battery power.
    CriticalBattery,
}

/// Power action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    /// Suspend the system.
    Suspend,
    /// Hibernate the system.
    Hibernate,
    /// Shutdown the system.
    Shutdown,
    /// Reboot the system.
    Reboot,
    /// Log out the current user.
    Logout,
}

/// Battery information.
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    /// Whether a battery is present.
    pub present: bool,
    /// The battery percentage (0-100).
    pub percentage: f64,
    /// The battery state.
    pub state: BatteryState,
    /// The estimated time remaining (in seconds).
    pub time_remaining: Option<Duration>,
}

/// Battery state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryState {
    /// The battery is charging.
    Charging,
    /// The battery is discharging.
    Discharging,
    /// The battery is fully charged.
    Full,
    /// The battery state is unknown.
    Unknown,
}

/// Power manager interface.
#[async_trait]
pub trait PowerManager: Send + Sync {
    /// Gets the current power state.
    ///
    /// # Returns
    ///
    /// The current power state.
    async fn get_power_state(&self) -> SystemResult<PowerState>;

    /// Gets battery information.
    ///
    /// # Returns
    ///
    /// Battery information, or an error if it failed.
    async fn get_battery_info(&self) -> SystemResult<BatteryInfo>;

    /// Performs a power action.
    ///
    /// # Arguments
    ///
    /// * `action` - The power action to perform
    ///
    /// # Returns
    ///
    /// `Ok(())` if the action was performed, or an error if it failed.
    async fn perform_action(&self, action: PowerAction) -> SystemResult<()>;

    /// Checks if a power action is supported.
    ///
    /// # Arguments
    ///
    /// * `action` - The power action to check
    ///
    /// # Returns
    ///
    /// `true` if the action is supported, `false` otherwise.
    async fn is_action_supported(&self, action: PowerAction) -> SystemResult<bool>;

    /// Sets the screen brightness.
    ///
    /// # Arguments
    ///
    /// * `brightness` - The brightness level (0.0-1.0)
    ///
    /// # Returns
    ///
    /// `Ok(())` if the brightness was set, or an error if it failed.
    async fn set_screen_brightness(&self, brightness: f64) -> SystemResult<()>;

    /// Gets the current screen brightness.
    ///
    /// # Returns
    ///
    /// The current brightness level (0.0-1.0).
    async fn get_screen_brightness(&self) -> SystemResult<f64>;

    // TODO: Assistant Integration: The Smart Assistant might need to trigger actions like
    // "shutdown", "reboot", "suspend". The existing `perform_action` method with the
    // `PowerAction` enum (Shutdown, Reboot, Suspend, Hibernate) seems to cover these commands.
    // Ensure this service is accessible, e.g., via D-Bus or an internal API from a
    // privileged domain service that the assistant can call.
    //
    // Additional queries the assistant might make (some might be better via SystemSettingsService):
    // - "What is the current power profile?"
    // - "Is the screen set to turn off automatically?" (related to idle timeout, often a global setting)
}

/// System power manager implementation.
pub struct SystemPowerManager {
    logind_handler: Arc<LogindProxyHandler>,
}

impl SystemPowerManager {
    pub async fn new(
        dbus_connection: Arc<ZbusConnection>,
    ) -> SystemResult<Self> {
        let logind_handler = Arc::new(LogindProxyHandler::new(dbus_connection).await?);
        Ok(SystemPowerManager {
            logind_handler,
        })
    }

    pub fn get_logind_handler(&self) -> Arc<LogindProxyHandler> {
        self.logind_handler.clone()
    }
}

#[async_trait]
impl PowerManager for SystemPowerManager {
    async fn get_power_state(&self) -> SystemResult<PowerState> {
        Err(SystemError::new(SystemErrorKind::NotSupported, "get_power_state not supported by logind backend".to_string()))
    }

    async fn get_battery_info(&self) -> SystemResult<BatteryInfo> {
        Err(SystemError::new(SystemErrorKind::NotSupported, "get_battery_info not supported by logind backend".to_string()))
    }

    async fn perform_action(&self, action: PowerAction) -> SystemResult<()> {
        match action {
            PowerAction::Shutdown => self.logind_handler.power_action_interactive("PowerOff", false).await,
            PowerAction::Reboot => self.logind_handler.power_action_interactive("Reboot", false).await,
            PowerAction::Suspend => self.logind_handler.power_action_interactive("Suspend", false).await,
            PowerAction::Hibernate => self.logind_handler.power_action_interactive("Hibernate", false).await,
            PowerAction::Logout => Err(SystemError::new(SystemErrorKind::NotSupported, "Logout action not handled by logind directly".to_string())),
        }
    }

    async fn is_action_supported(&self, action: PowerAction) -> SystemResult<bool> {
        match action {
            PowerAction::Shutdown => self.logind_handler.can_verb("CanPowerOff").await,
            PowerAction::Reboot => self.logind_handler.can_verb("CanReboot").await,
            PowerAction::Suspend => self.logind_handler.can_verb("CanSuspend").await,
            PowerAction::Hibernate => self.logind_handler.can_verb("CanHibernate").await,
            PowerAction::Logout => Ok(false),
        }
    }

    async fn set_screen_brightness(&self, _brightness: f64) -> SystemResult<()> {
        Err(SystemError::new(SystemErrorKind::NotSupported, "set_screen_brightness not supported by logind backend".to_string()))
    }

    async fn get_screen_brightness(&self) -> SystemResult<f64> {
        Err(SystemError::new(SystemErrorKind::NotSupported, "get_screen_brightness not supported by logind backend".to_string()))
    }
}

// PowerDBusConnection struct and its impl removed as it's replaced by LogindProxyHandler

#[cfg(test)]
mod tests {
    use super::*;
    // Note: These tests would need a running D-Bus session with logind for full functionality.
    // Mocking zbus::Connection and Proxy would be necessary for true unit tests.

    #[tokio::test]
    async fn test_system_power_manager_init_and_check_support() {
        // This test requires a D-Bus connection. It might fail in CI or environments without D-Bus.
        let connection = ZbusConnection::system().await;
        if let Err(e) = connection {
            tracing::warn!("System D-Bus connection failed, skipping SystemPowerManager test: {}", e);
            return;
        }
        let manager = SystemPowerManager::new(Arc::new(connection.unwrap())).await;
        assert!(manager.is_ok(), "Failed to create SystemPowerManager: {:?}", manager.err());
        let manager = manager.unwrap();

        // Check a few actions. Whether these are true/false depends on the system's capabilities.
        let can_shutdown = manager.is_action_supported(PowerAction::Shutdown).await;
        tracing::info!("CanPowerOff reported: {:?}", can_shutdown);
        assert!(can_shutdown.is_ok()); // Should return Ok(true) or Ok(false)

        let can_suspend = manager.is_action_supported(PowerAction::Suspend).await;
        tracing::info!("CanSuspend reported: {:?}", can_suspend);
        assert!(can_suspend.is_ok());

        // Test an unsupported action by this backend
        let can_logout = manager.is_action_supported(PowerAction::Logout).await;
        assert_eq!(can_logout, Ok(false));

        let get_state_err = manager.get_power_state().await;
        assert!(get_state_err.is_err());
        if let Err(e) = get_state_err {
            assert_eq!(e.kind(), SystemErrorKind::NotSupported);
        }
    }
}
