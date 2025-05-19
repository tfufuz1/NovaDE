//! Default power management service implementation for the NovaDE domain layer.
//!
//! This module provides a thread-safe default implementation of the power management service
//! for the NovaDE desktop environment.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use async_trait::async_trait;
use uuid::Uuid;
use crate::error::{DomainError, PowerManagementError};
use super::{PowerManagementService, PowerState, BatteryState, BatteryInfo};

/// Thread-safe default implementation of the power management service.
pub struct DefaultPowerManagementService {
    /// Current power state, protected by RwLock for concurrent read access
    power_state: RwLock<PowerState>,
    /// Battery information, protected by RwLock for concurrent read access
    batteries: RwLock<HashMap<String, BatteryInfo>>,
    /// Idle timeout in seconds, protected by RwLock for concurrent read access
    idle_timeout: RwLock<u64>,
    /// Current idle time in seconds, protected by RwLock for concurrent read access
    idle_time: RwLock<u64>,
    /// Sleep inhibitors, protected by Mutex for exclusive access
    sleep_inhibitors: Mutex<HashMap<String, String>>,
    /// Can suspend flag, protected by RwLock for concurrent read access
    can_suspend: RwLock<bool>,
    /// Can hibernate flag, protected by RwLock for concurrent read access
    can_hibernate: RwLock<bool>,
    /// Can shutdown flag, protected by RwLock for concurrent read access
    can_shutdown: RwLock<bool>,
    /// Can restart flag, protected by RwLock for concurrent read access
    can_restart: RwLock<bool>,
}

impl DefaultPowerManagementService {
    /// Creates a new default power management service.
    pub fn new() -> Self {
        // Create a mock battery for testing
        let mut batteries = HashMap::new();
        let mock_battery = BatteryInfo {
            id: "BAT0".to_string(),
            state: BatteryState::Discharging,
            percentage: 75.5,
            time_to_empty: Some(7200),
            time_to_full: None,
            is_present: true,
            vendor: Some("Mock Vendor".to_string()),
            model: Some("Mock Model".to_string()),
            serial: Some("12345".to_string()),
            technology: Some("Li-ion".to_string()),
            capacity: 95.0,
            energy: 45.6,
            energy_full: 60.0,
            energy_full_design: 65.0,
            voltage: 12.1,
            properties: HashMap::new(),
        };
        batteries.insert(mock_battery.id.clone(), mock_battery);
        
        Self {
            power_state: RwLock::new(PowerState::Running),
            batteries: RwLock::new(batteries),
            idle_timeout: RwLock::new(300), // 5 minutes default
            idle_time: RwLock::new(0),
            sleep_inhibitors: Mutex::new(HashMap::new()),
            can_suspend: RwLock::new(true),
            can_hibernate: RwLock::new(true),
            can_shutdown: RwLock::new(true),
            can_restart: RwLock::new(true),
        }
    }
    
    /// Creates a new default power management service with custom capabilities.
    pub fn with_capabilities(
        can_suspend: bool,
        can_hibernate: bool,
        can_shutdown: bool,
        can_restart: bool,
    ) -> Self {
        let mut service = Self::new();
        *service.can_suspend.write().unwrap() = can_suspend;
        *service.can_hibernate.write().unwrap() = can_hibernate;
        *service.can_shutdown.write().unwrap() = can_shutdown;
        *service.can_restart.write().unwrap() = can_restart;
        service
    }
    
    /// Updates the idle time.
    ///
    /// This method is intended to be called periodically by the system.
    ///
    /// # Arguments
    ///
    /// * `idle_time` - The new idle time in seconds
    pub fn update_idle_time(&self, idle_time: u64) {
        *self.idle_time.write().unwrap() = idle_time;
    }
    
    /// Updates a battery's information.
    ///
    /// This method is intended to be called when battery information changes.
    ///
    /// # Arguments
    ///
    /// * `battery_info` - The updated battery information
    pub fn update_battery(&self, battery_info: BatteryInfo) {
        let mut batteries = self.batteries.write().unwrap();
        batteries.insert(battery_info.id.clone(), battery_info);
    }
}

// Implement Send and Sync explicitly to guarantee thread safety
unsafe impl Send for DefaultPowerManagementService {}
unsafe impl Sync for DefaultPowerManagementService {}

#[async_trait]
impl PowerManagementService for DefaultPowerManagementService {
    async fn get_power_state(&self) -> Result<PowerState, DomainError> {
        // Use read lock for concurrent access
        let power_state = *self.power_state.read().unwrap();
        Ok(power_state)
    }
    
    async fn get_batteries(&self) -> Result<Vec<BatteryInfo>, DomainError> {
        // Use read lock for concurrent access
        let batteries = self.batteries.read().unwrap();
        let battery_list = batteries.values().cloned().collect();
        Ok(battery_list)
    }
    
    async fn get_battery(&self, battery_id: &str) -> Result<BatteryInfo, DomainError> {
        // Use read lock for concurrent access
        let batteries = self.batteries.read().unwrap();
        
        batteries.get(battery_id)
            .cloned()
            .ok_or_else(|| PowerManagementError::BatteryNotFound(battery_id.to_string()).into())
    }
    
    async fn suspend(&self) -> Result<(), DomainError> {
        // Check if we can suspend
        if !*self.can_suspend.read().unwrap() {
            return Err(PowerManagementError::OperationNotSupported("suspend".to_string()).into());
        }
        
        // Check for inhibitors
        let inhibitors = self.sleep_inhibitors.lock().unwrap();
        if !inhibitors.is_empty() {
            return Err(PowerManagementError::SleepInhibited(
                inhibitors.values().cloned().collect::<Vec<_>>().join(", ")
            ).into());
        }
        
        // Update power state
        *self.power_state.write().unwrap() = PowerState::Suspending;
        
        // In a real implementation, this would call the system's suspend function
        // For now, we just simulate it
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Reset power state
        *self.power_state.write().unwrap() = PowerState::Running;
        
        Ok(())
    }
    
    async fn hibernate(&self) -> Result<(), DomainError> {
        // Check if we can hibernate
        if !*self.can_hibernate.read().unwrap() {
            return Err(PowerManagementError::OperationNotSupported("hibernate".to_string()).into());
        }
        
        // Check for inhibitors
        let inhibitors = self.sleep_inhibitors.lock().unwrap();
        if !inhibitors.is_empty() {
            return Err(PowerManagementError::SleepInhibited(
                inhibitors.values().cloned().collect::<Vec<_>>().join(", ")
            ).into());
        }
        
        // Update power state
        *self.power_state.write().unwrap() = PowerState::Hibernating;
        
        // In a real implementation, this would call the system's hibernate function
        // For now, we just simulate it
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Reset power state
        *self.power_state.write().unwrap() = PowerState::Running;
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), DomainError> {
        // Check if we can shutdown
        if !*self.can_shutdown.read().unwrap() {
            return Err(PowerManagementError::OperationNotSupported("shutdown".to_string()).into());
        }
        
        // Update power state
        *self.power_state.write().unwrap() = PowerState::ShuttingDown;
        
        // In a real implementation, this would call the system's shutdown function
        // For now, we just simulate it
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        Ok(())
    }
    
    async fn restart(&self) -> Result<(), DomainError> {
        // Check if we can restart
        if !*self.can_restart.read().unwrap() {
            return Err(PowerManagementError::OperationNotSupported("restart".to_string()).into());
        }
        
        // Update power state
        *self.power_state.write().unwrap() = PowerState::Restarting;
        
        // In a real implementation, this would call the system's restart function
        // For now, we just simulate it
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        Ok(())
    }
    
    async fn can_suspend(&self) -> Result<bool, DomainError> {
        Ok(*self.can_suspend.read().unwrap())
    }
    
    async fn can_hibernate(&self) -> Result<bool, DomainError> {
        Ok(*self.can_hibernate.read().unwrap())
    }
    
    async fn can_shutdown(&self) -> Result<bool, DomainError> {
        Ok(*self.can_shutdown.read().unwrap())
    }
    
    async fn can_restart(&self) -> Result<bool, DomainError> {
        Ok(*self.can_restart.read().unwrap())
    }
    
    async fn get_idle_time(&self) -> Result<u64, DomainError> {
        Ok(*self.idle_time.read().unwrap())
    }
    
    async fn set_idle_timeout(&self, timeout: u64) -> Result<(), DomainError> {
        *self.idle_timeout.write().unwrap() = timeout;
        Ok(())
    }
    
    async fn get_idle_timeout(&self) -> Result<u64, DomainError> {
        Ok(*self.idle_timeout.read().unwrap())
    }
    
    async fn inhibit_sleep(&self, reason: &str) -> Result<String, DomainError> {
        let inhibitor_id = Uuid::new_v4().to_string();
        
        let mut inhibitors = self.sleep_inhibitors.lock().unwrap();
        inhibitors.insert(inhibitor_id.clone(), reason.to_string());
        
        Ok(inhibitor_id)
    }
    
    async fn uninhibit_sleep(&self, inhibitor_id: &str) -> Result<(), DomainError> {
        let mut inhibitors = self.sleep_inhibitors.lock().unwrap();
        
        if inhibitors.remove(inhibitor_id).is_none() {
            return Err(PowerManagementError::InhibitorNotFound(inhibitor_id.to_string()).into());
        }
        
        Ok(())
    }
    
    async fn get_sleep_inhibitors(&self) -> Result<HashMap<String, String>, DomainError> {
        let inhibitors = self.sleep_inhibitors.lock().unwrap().clone();
        Ok(inhibitors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_power_state() {
        let service = DefaultPowerManagementService::new();
        
        let power_state = service.get_power_state().await.unwrap();
        assert_eq!(power_state, PowerState::Running);
    }
    
    #[tokio::test]
    async fn test_get_batteries() {
        let service = DefaultPowerManagementService::new();
        
        let batteries = service.get_batteries().await.unwrap();
        assert_eq!(batteries.len(), 1);
        assert_eq!(batteries[0].id, "BAT0");
    }
    
    #[tokio::test]
    async fn test_get_battery() {
        let service = DefaultPowerManagementService::new();
        
        let battery = service.get_battery("BAT0").await.unwrap();
        assert_eq!(battery.id, "BAT0");
        assert_eq!(battery.state, BatteryState::Discharging);
        assert_eq!(battery.percentage, 75.5);
    }
    
    #[tokio::test]
    async fn test_get_battery_not_found() {
        let service = DefaultPowerManagementService::new();
        
        let result = service.get_battery("NONEXISTENT").await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_update_battery() {
        let service = DefaultPowerManagementService::new();
        
        let mut updated_battery = service.get_battery("BAT0").await.unwrap();
        updated_battery.percentage = 50.0;
        updated_battery.state = BatteryState::Charging;
        
        service.update_battery(updated_battery);
        
        let battery = service.get_battery("BAT0").await.unwrap();
        assert_eq!(battery.percentage, 50.0);
        assert_eq!(battery.state, BatteryState::Charging);
    }
    
    #[tokio::test]
    async fn test_idle_time() {
        let service = DefaultPowerManagementService::new();
        
        // Set idle timeout
        service.set_idle_timeout(600).await.unwrap();
        
        // Check idle timeout
        let idle_timeout = service.get_idle_timeout().await.unwrap();
        assert_eq!(idle_timeout, 600);
        
        // Update idle time
        service.update_idle_time(300);
        
        // Check idle time
        let idle_time = service.get_idle_time().await.unwrap();
        assert_eq!(idle_time, 300);
    }
    
    #[tokio::test]
    async fn test_sleep_inhibitors() {
        let service = DefaultPowerManagementService::new();
        
        // Inhibit sleep
        let inhibitor_id = service.inhibit_sleep("Testing").await.unwrap();
        
        // Check inhibitors
        let inhibitors = service.get_sleep_inhibitors().await.unwrap();
        assert_eq!(inhibitors.len(), 1);
        assert_eq!(inhibitors.get(&inhibitor_id), Some(&"Testing".to_string()));
        
        // Try to suspend (should fail)
        let result = service.suspend().await;
        assert!(result.is_err());
        
        // Uninhibit sleep
        service.uninhibit_sleep(&inhibitor_id).await.unwrap();
        
        // Check inhibitors again
        let inhibitors = service.get_sleep_inhibitors().await.unwrap();
        assert_eq!(inhibitors.len(), 0);
        
        // Try to suspend again (should succeed)
        let result = service.suspend().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_capabilities() {
        let service = DefaultPowerManagementService::with_capabilities(
            true, false, true, false
        );
        
        assert!(service.can_suspend().await.unwrap());
        assert!(!service.can_hibernate().await.unwrap());
        assert!(service.can_shutdown().await.unwrap());
        assert!(!service.can_restart().await.unwrap());
        
        // Try operations
        assert!(service.suspend().await.is_ok());
        assert!(service.hibernate().await.is_err());
        assert!(service.shutdown().await.is_ok());
        assert!(service.restart().await.is_err());
    }
    
    #[tokio::test]
    async fn test_thread_safety() {
        use std::thread;
        use std::sync::Arc;
        
        let service = Arc::new(DefaultPowerManagementService::new());
        
        let service_clone1 = Arc::clone(&service);
        let service_clone2 = Arc::clone(&service);
        
        // Spawn two threads that update the service concurrently
        let handle1 = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Thread 1: Update battery
                let mut battery = service_clone1.get_battery("BAT0").await.unwrap();
                battery.percentage = 60.0;
                service_clone1.update_battery(battery);
                
                // Thread 1: Set idle timeout
                service_clone1.set_idle_timeout(900).await.unwrap();
            });
        });
        
        let handle2 = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Thread 2: Inhibit sleep
                service_clone2.inhibit_sleep("Thread 2").await.unwrap();
                
                // Thread 2: Update idle time
                service_clone2.update_idle_time(150);
            });
        });
        
        // Wait for both threads to complete
        handle1.join().unwrap();
        handle2.join().unwrap();
        
        // Check that all updates were applied correctly
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let battery = service.get_battery("BAT0").await.unwrap();
            assert_eq!(battery.percentage, 60.0);
            
            let idle_timeout = service.get_idle_timeout().await.unwrap();
            assert_eq!(idle_timeout, 900);
            
            let idle_time = service.get_idle_time().await.unwrap();
            assert_eq!(idle_time, 150);
            
            let inhibitors = service.get_sleep_inhibitors().await.unwrap();
            assert_eq!(inhibitors.len(), 1);
            assert!(inhibitors.values().next().unwrap() == "Thread 2");
        });
    }
}
