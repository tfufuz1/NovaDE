//! Display management module for the NovaDE system layer.
//!
//! This module provides display management functionality for the NovaDE desktop environment,
//! with implementations for both X11 and Wayland display servers.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
// use novade_core::types::geometry::{Point, Size, Rect}; // Point, Size, Rect are not directly used now, Display has position_x, position_y
use crate::error::{SystemError, SystemResult, to_system_error, SystemErrorKind};
use novade_core::types::display::{
    Display, DisplayConfiguration, DisplayConnector, DisplayMode, PhysicalProperties, DisplayStatus, DisplayLayout,
};

// Local Display and DisplayConfig structs are removed. We now use types from novade_core.

/// Display manager interface.
#[async_trait]
pub trait DisplayManager: Send + Sync {
    /// Gets all displays.
    ///
    /// # Returns
    ///
    /// A vector of all displays.
    async fn get_displays(&self) -> SystemResult<Vec<Display>>;

    /// Gets a display by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The display ID
    ///
    /// # Returns
    ///
    /// The display, or an error if it doesn't exist.
    async fn get_display(&self, id: &str) -> SystemResult<Display>;

    /// Gets the primary display.
    ///
    /// # Returns
    ///
    /// The primary display, or an error if there is no primary display.
    // Note: The concept of a "primary display" might need to be handled at a higher level
    // or by inspecting Display properties (e.g., a specific flag or position).
    // For now, we'll keep the method signature but acknowledge this might change.
    async fn get_primary_display(&self) -> SystemResult<Display>;

    /// Configures a display.
    /// This method might need to change to accept a single Display struct or a list of them,
    /// representing the desired state, rather than a DisplayConfiguration for a single display.
    /// Or, DisplayConfiguration itself needs to be re-evaluated.
    /// For now, we adapt to novade_core::types::display::Display for individual configuration.
    /// A more holistic approach might use DisplayConfiguration which contains Vec<Display>.
    /// Let's assume for now `configure_display` applies settings for one display.
    /// The core `DisplayConfiguration` struct holds a Vec<Display> and a Layout.
    /// This function might evolve to take `DisplayConfiguration` or a subset of its fields.
    /// For now, let's assume we are configuring a single display's properties like mode, position, enabled.
    /// The provided `DisplayConfig` (now removed) had fields like position, scale_factor, rotation, refresh_rate, enabled.
    /// The core `Display` struct has `current_mode`, `position_x`, `position_y`, `enabled`.
    /// Rotation is not in core `Display` yet. Scale factor is also not present.
    /// We will need a new struct or enum to pass parameters for configuration if `Display` itself isn't sufficient.
    /// Let's assume for now we pass the ID and a limited set of changes.
    /// Or, more simply, the `DisplayManager` applies a full `DisplayConfiguration`.
    /// The original `configure_display` took a `DisplayConfig` (local struct).
    /// Let's change it to take a `Display` object from core, implying the desired state for that display.
    async fn configure_display(&self, display_state: Display) -> SystemResult<()>;

    /// Sets the primary display.
    ///
    /// # Arguments
    ///
    /// * `id` - The display ID
    ///
    /// # Returns
    ///
    /// `Ok(())` if the primary display was set, or an error if it failed.
    async fn set_primary_display(&self, id: &str) -> SystemResult<()>;
    
    /// Applies the current display configuration.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration was applied, or an error if it failed.
    async fn apply_configuration(&self) -> SystemResult<()>;
}

/// X11 display manager implementation.
pub struct X11DisplayManager {
    /// The X11 connection.
    connection: Arc<Mutex<X11DisplayConnection>>,
    /// The display cache.
    display_cache: Arc<Mutex<HashMap<String, Display>>>, // Display is now from novade_core
}

impl X11DisplayManager {
    /// Creates a new X11 display manager.
    ///
    /// # Returns
    ///
    /// A new X11 display manager.
    pub fn new() -> SystemResult<Self> {
        let connection = X11DisplayConnection::new()?;
        
        Ok(X11DisplayManager {
            connection: Arc::new(Mutex::new(connection)),
            display_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Updates the display cache.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the cache was updated, or an error if it failed.
    async fn update_cache(&self) -> SystemResult<()> {
        let displays = {
            let connection = self.connection.lock().unwrap();
            connection.get_displays()?
        };
        
        let mut cache = self.display_cache.lock().unwrap();
        cache.clear();
        
        for display in displays {
            cache.insert(display.id.clone(), display); // Use .id field
        }
        
        Ok(())
    }
}

#[async_trait]
impl DisplayManager for X11DisplayManager {
    async fn get_displays(&self) -> SystemResult<Vec<Display>> { // Display is now from novade_core
        self.update_cache().await?;
        
        let cache = self.display_cache.lock().unwrap();
        let displays = cache.values().cloned().collect();
        
        Ok(displays)
    }
    
    async fn get_display(&self, id: &str) -> SystemResult<Display> { // Display is now from novade_core
        self.update_cache().await?;
        
        let cache = self.display_cache.lock().unwrap();
        
        cache.get(id)
            .cloned()
            .ok_or_else(|| to_system_error(format!("Display not found: {}", id), SystemErrorKind::DisplayManagement))
    }
    
    async fn get_primary_display(&self) -> SystemResult<Display> { // Display is now from novade_core
        self.update_cache().await?;
        
        let cache = self.display_cache.lock().unwrap();
        
        // The core Display struct doesn't have an `is_primary` field.
        // This needs to be determined by convention (e.g., position 0,0 or a specific flag in a future DisplayConfiguration.layout_properties)
        // For now, let's return the first enabled display if any, or error.
        cache.values()
            .find(|d| d.enabled)
            .cloned()
            .ok_or_else(|| to_system_error("No primary (enabled) display found", SystemErrorKind::DisplayManagement))
    }
    
    async fn configure_display(&self, display_state: Display) -> SystemResult<()> { // Takes core Display
        let connection = self.connection.lock().unwrap();
        connection.configure_display(display_state)
    }
    
    async fn set_primary_display(&self, id: &str) -> SystemResult<()> {
        let connection = self.connection.lock().unwrap();
        connection.set_primary_display(id)
    }
    
    async fn apply_configuration(&self) -> SystemResult<()> {
        let connection = self.connection.lock().unwrap();
        connection.apply_configuration()
    }
}

/// Wayland display manager implementation.
pub struct WaylandDisplayManager {
    /// The Wayland connection.
    connection: Arc<Mutex<WaylandDisplayConnection>>,
    /// The display cache.
    display_cache: Arc<Mutex<HashMap<String, Display>>>, // Display is now from novade_core
}

impl WaylandDisplayManager {
    /// Creates a new Wayland display manager.
    ///
    /// # Returns
    ///
    /// A new Wayland display manager.
    pub fn new() -> SystemResult<Self> {
        let connection = WaylandDisplayConnection::new()?;
        
        Ok(WaylandDisplayManager {
            connection: Arc::new(Mutex::new(connection)),
            display_cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// Updates the display cache.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the cache was updated, or an error if it failed.
    async fn update_cache(&self) -> SystemResult<()> {
        let displays = {
            let connection = self.connection.lock().unwrap();
            connection.get_displays()?
        };
        
        let mut cache = self.display_cache.lock().unwrap();
        cache.clear();
        
        for display in displays {
            cache.insert(display.id.clone(), display); // Use .id field
        }
        
        Ok(())
    }
}

#[async_trait]
impl DisplayManager for WaylandDisplayManager {
    async fn get_displays(&self) -> SystemResult<Vec<Display>> { // Display is now from novade_core
        // This will be replaced by DRM logic later for WaylandDisplayManager
        self.update_cache().await?;
        
        let cache = self.display_cache.lock().unwrap();
        let displays = cache.values().cloned().collect();
        
        Ok(displays)
    }
    
    async fn get_display(&self, id: &str) -> SystemResult<Display> { // Display is now from novade_core
        self.update_cache().await?;
        
        let cache = self.display_cache.lock().unwrap();
        
        cache.get(id)
            .cloned()
            .ok_or_else(|| to_system_error(format!("Display not found: {}", id), SystemErrorKind::DisplayManagement))
    }
    
    async fn get_primary_display(&self) -> SystemResult<Display> { // Display is now from novade_core
        self.update_cache().await?;
        
        let cache = self.display_cache.lock().unwrap();
        
        // The core Display struct doesn't have an `is_primary` field.
        // Similar to X11DisplayManager, find first enabled display.
        cache.values()
            .find(|d| d.enabled)
            .cloned()
            .ok_or_else(|| to_system_error("No primary (enabled) display found", SystemErrorKind::DisplayManagement))
    }
    
    async fn configure_display(&self, display_state: Display) -> SystemResult<()> { // Takes core Display
        let connection = self.connection.lock().unwrap();
        connection.configure_display(display_state)
    }
    
    async fn set_primary_display(&self, id: &str) -> SystemResult<()> {
        let connection = self.connection.lock().unwrap();
        connection.set_primary_display(id)
    }
    
    async fn apply_configuration(&self) -> SystemResult<()> {
        let connection = self.connection.lock().unwrap();
        connection.apply_configuration()
    }
}

/// X11 display connection.
struct X11DisplayConnection {
    // In a real implementation, this would contain the X11 connection
    // For now, we'll use a placeholder implementation
}

impl X11DisplayConnection {
    /// Creates a new X11 display connection.
    ///
    /// # Returns
    ///
    /// A new X11 display connection.
    fn new() -> SystemResult<Self> {
        // In a real implementation, this would connect to the X11 server
        Ok(X11DisplayConnection {})
    }
    
    /// Gets all displays.
    ///
    /// # Returns
    ///
    /// A vector of all displays.
    fn get_displays(&self) -> SystemResult<Vec<Display>> { // Display is now from novade_core
        // In a real implementation, this would query the X11 server for displays
        // For now, we'll return a placeholder display aligned with novade_core::Display
        let mode = DisplayMode {
            width: 1920,
            height: 1080,
            refresh_rate: 60000, // 60Hz in mHz
        };
        let display = Display {
            id: "HDMI-1".to_string(),
            name: "HDMI-1 (X11)".to_string(),
            connector: DisplayConnector::HDMI,
            status: DisplayStatus::Connected,
            modes: vec![mode.clone()],
            current_mode: Some(mode),
            physical_properties: Some(PhysicalProperties { width_mm: 597, height_mm: 336 }), // Example values
            position_x: 0,
            position_y: 0,
            enabled: true,
            // rotation and scale_factor are not part of core Display
        };
        
        Ok(vec![display])
    }
    
    /// Configures a display.
    ///
    /// # Arguments
    ///
    /// * `_display_state` - The desired state for the display (core Display struct)
    ///
    /// # Returns
    ///
    /// `Ok(())` if the display was configured, or an error if it failed.
    fn configure_display(&self, _display_state: Display) -> SystemResult<()> { // Takes core Display
        // In a real implementation, this would configure the display
        Ok(())
    }
    
    /// Sets the primary display.
    ///
    /// # Arguments
    ///
    /// * `id` - The display ID
    ///
    /// # Returns
    ///
    /// `Ok(())` if the primary display was set, or an error if it failed.
    fn set_primary_display(&self, _id: &str) -> SystemResult<()> {
        // In a real implementation, this would set the primary display
        Ok(())
    }
    
    /// Applies the current display configuration.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration was applied, or an error if it failed.
    fn apply_configuration(&self) -> SystemResult<()> {
        // In a real implementation, this would apply the configuration
        Ok(())
    }
}

/// Wayland display connection.
struct WaylandDisplayConnection {
    // In a real implementation, this would contain the Wayland connection
    // For now, we'll use a placeholder implementation
}

impl WaylandDisplayConnection {
    /// Creates a new Wayland display connection.
    ///
    /// # Returns
    ///
    /// A new Wayland display connection.
    fn new() -> SystemResult<Self> {
        // In a real implementation, this would connect to the Wayland server
        Ok(WaylandDisplayConnection {})
    }
    
    /// Gets all displays.
    ///
    /// # Returns
    ///
    /// A vector of all displays.
    fn get_displays(&self) -> SystemResult<Vec<Display>> { // Display is now from novade_core
        // DRM-based display detection
        use drm::control::{Device as DrmDevice, ResourceHandles, Connector as DrmConnectorInfo, Mode as DrmMode};
        use drm::Device as DrmControlDevice; // Trait for common device methods
        use std::fs::{File, OpenOptions};
        use std::os::unix::io::{AsRawFd, RawFd};
        // Assuming novade_core types are already in scope:
        // use novade_core::types::display::{Display, DisplayMode, DisplayConnector, DisplayStatus, PhysicalProperties};

        // Simplified path, real implementation might need to iterate or use udev
        const DRM_NODE_PATH: &str = "/dev/dri/card0";
        let drm_file = OpenOptions::new().read(true).write(true).open(DRM_NODE_PATH)
            .map_err(|e| to_system_error(format!("Failed to open DRM node {}: {}", DRM_NODE_PATH, e), SystemErrorKind::DisplayManagement))?;

        // The drm crate's `control::Device` is typically what we need for KMS ioctls.
        // It's usually created from a RawFd.
        // Let's ensure we have a struct that implements DrmControlDevice.
        // The `drm::control::Device` itself can be used.
        let drm_device = drm::control::Device::new(drm_file.as_raw_fd(), true) // true to disable client cap
             .map_err(|e| to_system_error(format!("Failed to create DRM control device: {}", e), SystemErrorKind::DisplayManagement))?;


        let res_handles = drm_device.resource_handles()
            .map_err(|e| to_system_error(format!("Failed to get DRM resource handles: {}", e), SystemErrorKind::DisplayManagement))?;

        let mut core_displays = Vec::new();

        for &conn_id in res_handles.connectors() {
            let conn_info = drm_device.get_connector(conn_id, false) // false for no_encoders
                .map_err(|e| to_system_error(format!("Failed to get connector info for ID {}: {:?}", conn_id, e), SystemErrorKind::DisplayManagement))?;

            let core_status = match conn_info.state() {
                drm::control::connector::State::Connected => DisplayStatus::Connected,
                drm::control::connector::State::Disconnected => DisplayStatus::Disconnected,
                _ => DisplayStatus::Unknown,
            };

            if core_status != DisplayStatus::Connected {
                continue;
            }

            let core_connector = match conn_info.interface() {
                drm::control::connector::Interface::HDMIA => DisplayConnector::HDMI,
                drm::control::connector::Interface::HDMIB => DisplayConnector::HDMI,
                drm::control::connector::Interface::DisplayPort => DisplayConnector::DisplayPort,
                drm::control::connector::Interface::DVID => DisplayConnector::DVI,
                drm::control::connector::Interface::DVII => DisplayConnector::DVI,
                drm::control::connector::Interface::VGA => DisplayConnector::VGA,
                drm::control::connector::Interface::EmbeddedDisplayPort => DisplayConnector::DisplayPort,
                drm::control::connector::Interface::LVDS => DisplayConnector::LVDS,
                _ => DisplayConnector::Unknown,
            };

            let core_modes: Vec<DisplayMode> = conn_info.modes().iter().map(|drm_mode| {
                DisplayMode {
                    width: drm_mode.size().0 as u32,
                    height: drm_mode.size().1 as u32,
                    refresh_rate: drm_mode.vrefresh() as u32 * 1000, // DRM vrefresh is in Hz, convert to mHz
                }
            }).collect();

            if core_modes.is_empty() {
                // If no modes, we might not want to add this display or mark it as problematic
                // For now, skip if no modes are found, as a display isn't very useful without them.
                continue;
            }

            let current_core_mode = conn_info.modes().iter()
                .find(|m| m.mode_type().contains(drm::control::ModeTypeFlags::PREFERRED))
                .map(|drm_mode| DisplayMode {
                    width: drm_mode.size().0 as u32,
                    height: drm_mode.size().1 as u32,
                    refresh_rate: drm_mode.vrefresh() as u32 * 1000,
                })
                .or_else(|| core_modes.get(0).cloned());


            let core_physical = if conn_info.size_mm().0 > 0 && conn_info.size_mm().1 > 0 {
                Some(PhysicalProperties {
                    width_mm: conn_info.size_mm().0 as u32,
                    height_mm: conn_info.size_mm().1 as u32,
                })
            } else {
                None // Physical size not available or zero
            };

            let display_id = format!("{:?}-{}", conn_info.interface(), conn_info.interface_id());
            // A more robust name might involve querying EDID if available, or using a model name.
            // For now, the ID is sufficient for a name.
            let display_name = format!("Display {} ({:?})", core_displays.len() + 1, conn_info.interface());


            core_displays.push(Display {
                id: display_id,
                name: display_name,
                connector: core_connector,
                status: core_status,
                modes: core_modes,
                current_mode: current_core_mode,
                physical_properties: core_physical,
                position_x: 0, // Default position, layouting is a separate step
                position_y: 0,
                enabled: true, // Assuming connected and has modes means enabled initially
            });
        }
        Ok(core_displays)
    }
    
    /// Configures a display.
    ///
    /// # Arguments
    ///
    /// * `_display_state` - The desired state for the display (core Display struct)
    ///
    /// # Returns
    ///
    /// `Ok(())` if the display was configured, or an error if it failed.
    fn configure_display(&self, _display_state: Display) -> SystemResult<()> { // Takes core Display
        // In a real implementation, this would configure the display
        Ok(())
    }
    
    /// Sets the primary display.
    ///
    /// # Arguments
    ///
    /// * `id` - The display ID
    ///
    /// # Returns
    ///
    /// `Ok(())` if the primary display was set, or an error if it failed.
    fn set_primary_display(&self, _id: &str) -> SystemResult<()> {
        // In a real implementation, this would set the primary display
        Ok(())
    }
    
    /// Applies the current display configuration.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration was applied, or an error if it failed.
    fn apply_configuration(&self) -> SystemResult<()> {
        // In a real implementation, this would apply the configuration
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // These tests are placeholders and would be more comprehensive in a real implementation
    
    #[tokio::test]
    async fn test_x11_display_manager() {
        let manager = X11DisplayManager::new().unwrap();
        
        let displays = manager.get_displays().await.unwrap();
        assert_eq!(displays.len(), 1);
        
        let display_from_get_displays = &displays[0];
        let id = display_from_get_displays.id.clone();
        
        let retrieved = manager.get_display(&id).await.unwrap();
        assert_eq!(retrieved.id, id);
        
        let primary = manager.get_primary_display().await.unwrap();
        assert_eq!(primary.id, id); // Assumes the first display is primary for now

        // Create a Display struct for configuration, matching the core definition
        // The old DisplayConfig is gone. We pass a Display object.
        // For testing, we can use the retrieved display and potentially modify it.
        let config_display_state = retrieved.clone();
        // Example: manager.configure_display(config_display_state.clone()).await.unwrap();
        // Since configure_display now takes Display, we pass a Display object.
        // The old DisplayConfig had fields like position, scale_factor, is_primary, rotation, refresh_rate, enabled.
        // These need to map to the new Display structure or a new configuration mechanism.
        // For now, we'll just pass a cloned Display object.

        manager.configure_display(config_display_state).await.unwrap();
        manager.set_primary_display(&id).await.unwrap();
        manager.apply_configuration().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_wayland_display_manager() {
        let manager = WaylandDisplayManager::new().unwrap();
        
        let displays = manager.get_displays().await.unwrap();
        assert_eq!(displays.len(), 1);
        
        let display_from_get_displays = &displays[0];
        let id = display_from_get_displays.id.clone();
        
        let retrieved = manager.get_display(&id).await.unwrap();
        assert_eq!(retrieved.id, id);
        
        let primary = manager.get_primary_display().await.unwrap();
        // Assuming the first display is considered primary for the test
        assert_eq!(primary.id, id);

        // Similar to the X11 test, create a Display struct for configuration.
        let config_display_state = retrieved.clone();

        manager.configure_display(config_display_state).await.unwrap();
        manager.set_primary_display(&id).await.unwrap();
        manager.apply_configuration().await.unwrap();
    }
}
