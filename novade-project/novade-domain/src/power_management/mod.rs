//! Power management module for the NovaDE domain layer.
//!
//! This module provides power management functionality for the NovaDE desktop environment,
//! including battery monitoring, power state management, and sleep inhibition.

mod services;

pub use services::default_power_management_service::DefaultPowerManagementService;

/// Power state of the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PowerState {
    /// System is running normally.
    Running,
    /// System is in a low-power state but still running.
    LowPower,
    /// System is suspending to RAM.
    Suspending,
    /// System is hibernating to disk.
    Hibernating,
    /// System is shutting down.
    ShuttingDown,
    /// System is restarting.
    Restarting,
}

/// Battery state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BatteryState {
    /// Battery is charging.
    Charging,
    /// Battery is discharging.
    Discharging,
    /// Battery is fully charged.
    Full,
    /// Battery state is unknown.
    Unknown,
}

/// Battery information.
#[derive(Debug, Clone, PartialEq)]
pub struct BatteryInfo {
    /// Battery identifier.
    pub id: String,
    /// Battery state.
    pub state: BatteryState,
    /// Battery percentage (0-100).
    pub percentage: f64,
    /// Battery time to empty in seconds, if available.
    pub time_to_empty: Option<u64>,
    /// Battery time to full in seconds, if available.
    pub time_to_full: Option<u64>,
    /// Battery is present.
    pub is_present: bool,
    /// Battery vendor.
    pub vendor: Option<String>,
    /// Battery model.
    pub model: Option<String>,
    /// Battery serial number.
    pub serial: Option<String>,
    /// Battery technology.
    pub technology: Option<String>,
    /// Battery capacity in percentage (0-100).
    pub capacity: f64,
    /// Battery energy in Wh.
    pub energy: f64,
    /// Battery energy full in Wh.
    pub energy_full: f64,
    /// Battery energy full design in Wh.
    pub energy_full_design: f64,
    /// Battery voltage in V.
    pub voltage: f64,
    /// Battery additional properties.
    pub properties: std::collections::HashMap<String, String>,
}

/// Power management service interface.
#[async_trait::async_trait]
pub trait PowerManagementService: Send + Sync {
    /// Gets the current power state.
    ///
    /// # Returns
    ///
    /// A `Result` containing the current power state.
    async fn get_power_state(&self) -> Result<PowerState, crate::error::DomainError>;
    
    /// Gets information about all batteries.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of battery information.
    async fn get_batteries(&self) -> Result<Vec<BatteryInfo>, crate::error::DomainError>;
    
    /// Gets information about a specific battery.
    ///
    /// # Arguments
    ///
    /// * `battery_id` - The battery ID
    ///
    /// # Returns
    ///
    /// A `Result` containing the battery information.
    async fn get_battery(&self, battery_id: &str) -> Result<BatteryInfo, crate::error::DomainError>;
    
    /// Suspends the system.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn suspend(&self) -> Result<(), crate::error::DomainError>;
    
    /// Hibernates the system.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn hibernate(&self) -> Result<(), crate::error::DomainError>;
    
    /// Shuts down the system.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn shutdown(&self) -> Result<(), crate::error::DomainError>;
    
    /// Restarts the system.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn restart(&self) -> Result<(), crate::error::DomainError>;
    
    /// Checks if the system can suspend.
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating if the system can suspend.
    async fn can_suspend(&self) -> Result<bool, crate::error::DomainError>;
    
    /// Checks if the system can hibernate.
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating if the system can hibernate.
    async fn can_hibernate(&self) -> Result<bool, crate::error::DomainError>;
    
    /// Checks if the system can shutdown.
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating if the system can shutdown.
    async fn can_shutdown(&self) -> Result<bool, crate::error::DomainError>;
    
    /// Checks if the system can restart.
    ///
    /// # Returns
    ///
    /// A `Result` containing a boolean indicating if the system can restart.
    async fn can_restart(&self) -> Result<bool, crate::error::DomainError>;
    
    /// Gets the system idle time in seconds.
    ///
    /// # Returns
    ///
    /// A `Result` containing the system idle time in seconds.
    async fn get_idle_time(&self) -> Result<u64, crate::error::DomainError>;
    
    /// Sets the system idle timeout in seconds.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The idle timeout in seconds
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn set_idle_timeout(&self, timeout: u64) -> Result<(), crate::error::DomainError>;
    
    /// Gets the system idle timeout in seconds.
    ///
    /// # Returns
    ///
    /// A `Result` containing the system idle timeout in seconds.
    async fn get_idle_timeout(&self) -> Result<u64, crate::error::DomainError>;
    
    /// Inhibits the system from going to sleep.
    ///
    /// # Arguments
    ///
    /// * `reason` - The reason for inhibiting
    ///
    /// # Returns
    ///
    /// A `Result` containing an inhibitor ID.
    async fn inhibit_sleep(&self, reason: &str) -> Result<String, crate::error::DomainError>;
    
    /// Uninhibits the system from going to sleep.
    ///
    /// # Arguments
    ///
    /// * `inhibitor_id` - The inhibitor ID
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    async fn uninhibit_sleep(&self, inhibitor_id: &str) -> Result<(), crate::error::DomainError>;
    
    /// Gets all active sleep inhibitors.
    ///
    /// # Returns
    ///
    /// A `Result` containing a map of inhibitor IDs to reasons.
    async fn get_sleep_inhibitors(&self) -> Result<std::collections::HashMap<String, String>, crate::error::DomainError>;
}
