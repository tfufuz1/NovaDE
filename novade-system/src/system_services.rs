// novade-system/src/system_services.rs

use crate::dbus_integration::manager::DbusServiceManager;
use crate::power_management::{SystemPowerManager, PowerManager};
use crate::network_manager::{NetworkManagerIntegration, NetworkManager}; // NetworkDBusConnection removed as it's internal to NMI::new_production
use novade_domain::DomainServices;
use std::sync::Arc;
use crate::error::{SystemResult, SystemError, SystemErrorKind};

// Placeholder traits and DbusConnection struct removed

/// `SystemServices` provides a centralized container for accessing various
/// system-level services and backends.
///
/// This structure will hold initialized instances or handles to services like
/// network management, power management, D-Bus connections, etc., which
/// are part of the `novade-system` layer and provide concrete implementations
/// for policies defined in the `novade-domain` layer.
#[derive(Clone)]
pub struct SystemServices {
    pub dbus_manager: Arc<DbusServiceManager>,
    pub power_manager: Arc<dyn PowerManager>,
    pub network_manager: Arc<dyn NetworkManager>,
    // pub domain_services: Arc<DomainServices>, // Optionally store if needed
}

impl SystemServices {
    /// Creates a new instance of `SystemServices`.
    ///
    /// Initialization of individual services happens here.
    pub async fn new(domain_services: Arc<DomainServices>) -> SystemResult<Self> {
        tracing::info!("Initializing SystemServices container with D-Bus integration...");

        // 1. Initialize DbusServiceManager
        let dbus_manager = Arc::new(DbusServiceManager::new().await.map_err(|e| {
            SystemError::new(SystemErrorKind::DBus, format!("Failed to create DbusServiceManager: {}", e))
        })?);
        tracing::info!("DbusServiceManager initialized.");

        // 2. Initialize SystemPowerManager
        let power_manager_dbus_conn = dbus_manager.system_bus();
        let system_power_manager = Arc::new(SystemPowerManager::new(power_manager_dbus_conn).await.map_err(|e| {
            SystemError::new(SystemErrorKind::PowerManagement, format!("Failed to create SystemPowerManager: {}", e))
        })?);
        tracing::info!("SystemPowerManager initialized.");

        if let Err(e) = system_power_manager.get_logind_handler().start_signal_listener(domain_services.clone()).await {
             tracing::error!("Failed to start logind signal listener: {}. Power events from logind might not be handled.", e);
        } else {
            tracing::info!("Logind signal listener started via SystemPowerManager.");
        }

        // 3. Initialize NetworkManagerIntegration
        let network_manager = Arc::new(NetworkManagerIntegration::new_production().map_err(|e| {
            SystemError::new(SystemErrorKind::NetworkManagement, format!("Failed to create NetworkManagerIntegration: {}", e))
        })?);
        tracing::info!("NetworkManagerIntegration initialized.");

        // 4. Serve NotificationsServer using DbusServiceManager
        if let Some(notification_service) = domain_services.notification_service.as_ref() {
            let notification_service_arc = Arc::clone(notification_service);
            if let Err(e) = dbus_manager.serve_notifications_server(notification_service_arc).await {
                tracing::error!("Failed to serve NotificationsServer: {}. Notifications D-Bus service will not be available.", e);
            } else {
                tracing::info!("NotificationsServer is being served by DbusServiceManager.");
            }
        } else {
            tracing::warn!("DomainNotificationService not available, cannot serve D-Bus NotificationsServer.");
        }

        Ok(Self {
            dbus_manager,
            power_manager: system_power_manager as Arc<dyn PowerManager>,
            network_manager: network_manager as Arc<dyn NetworkManager>,
            // domain_services, // Optionally store
        })
    }
}

// Default removed as new() is async and takes arguments now.
// impl Default for SystemServices {
//     fn default() -> Self {
//         // This would require a way to get DomainServices, or make it Option in new()
//         // For now, removing default as `new` is async and requires domain_services.
//         panic!("SystemServices cannot be created with Default::default() due to async new and required DomainServices.")
//     }
// }
