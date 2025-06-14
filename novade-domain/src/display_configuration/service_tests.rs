// novade-domain/src/display_configuration/service_tests.rs
#![cfg(test)]
use super::service::{DefaultDisplayConfigService, DisplayConfigService};
use super::persistence::DisplayPersistence;
use super::errors::{Result as DisplayResult, DisplayConfigurationError};
use novade_core::types::display::{Display, DisplayConfiguration, DisplayLayout, DisplayMode, DisplayConnector, DisplayStatus, PhysicalProperties};
use std::sync::{Arc, Mutex as StdMutex}; // Renamed to avoid clash if tokio::sync::Mutex is used elsewhere

// --- Mock DisplayPersistence ---
#[derive(Clone, Default)]
struct MockPersistence {
    config: Arc<StdMutex<Option<DisplayConfiguration>>>,
    should_load_fail: bool,
    should_save_fail: bool,
}

impl MockPersistence {
    fn new(initial_config: Option<DisplayConfiguration>) -> Self {
        MockPersistence {
            config: Arc::new(StdMutex::new(initial_config)),
            should_load_fail: false,
            should_save_fail: false,
        }
    }

    #[allow(dead_code)] // May be used in future tests
    fn set_should_load_fail(&mut self, fail: bool) {
        self.should_load_fail = fail;
    }

    #[allow(dead_code)] // May be used in future tests
    fn set_should_save_fail(&mut self, fail: bool) {
        self.should_save_fail = fail;
    }
}

#[async_trait::async_trait]
impl DisplayPersistence for MockPersistence {
    async fn save_config(&self, config: &DisplayConfiguration) -> DisplayResult<()> {
        if self.should_save_fail {
            return Err(DisplayConfigurationError::Persistence("Mock save error".to_string()));
        }
        let mut config_guard = self.config.lock().unwrap();
        *config_guard = Some(config.clone());
        Ok(())
    }

    async fn load_config(&self) -> DisplayResult<DisplayConfiguration> {
        if self.should_load_fail {
            return Err(DisplayConfigurationError::Persistence("Mock load error".to_string()));
        }
        let config_guard = self.config.lock().unwrap();
        config_guard.clone()
            .ok_or_else(|| DisplayConfigurationError::Persistence("No config in mock".to_string()))
    }
}

// --- Helper function to create sample data ---
fn create_sample_display_mode(width: u32, height: u32, refresh: u32) -> DisplayMode {
    DisplayMode { width, height, refresh_rate: refresh }
}

fn create_sample_display(id: &str, name: &str) -> Display {
    Display {
        id: id.to_string(),
        name: name.to_string(),
        connector: DisplayConnector::HDMI,
        status: DisplayStatus::Connected,
        modes: vec![create_sample_display_mode(1920, 1080, 60000)],
        current_mode: Some(create_sample_display_mode(1920, 1080, 60000)),
        physical_properties: Some(PhysicalProperties { width_mm: 597, height_mm: 336 }),
        position_x: 0,
        position_y: 0,
        enabled: true,
    }
}

fn create_sample_display_config(displays: Vec<Display>, layout: DisplayLayout) -> DisplayConfiguration {
    DisplayConfiguration { displays, layout }
}

// --- Tests for DefaultDisplayConfigService ---
#[tokio::test]
async fn test_new_service_loads_config() {
    let initial_config = create_sample_display_config(vec![create_sample_display("DP-1", "Primary")], DisplayLayout::Single);
    let mock_persistence = Arc::new(MockPersistence::new(Some(initial_config.clone())));

    let service = DefaultDisplayConfigService::new(mock_persistence).await.unwrap();
    let current_service_config = service.get_display_configuration().await.unwrap();

    assert_eq!(current_service_config, initial_config);
}

#[tokio::test]
async fn test_new_service_default_config_on_load_fail() {
    let mut mock_persistence = MockPersistence::new(None);
    mock_persistence.set_should_load_fail(true);
    let arc_mock_persistence = Arc::new(mock_persistence);

    let service = DefaultDisplayConfigService::new(arc_mock_persistence).await.unwrap();
    let current_service_config = service.get_display_configuration().await.unwrap();

    // Expect default configuration
    assert_eq!(current_service_config.displays.len(), 0);
    assert_eq!(current_service_config.layout, DisplayLayout::Single);
}

#[tokio::test]
async fn test_get_display_configuration() {
    let initial_config = create_sample_display_config(vec![create_sample_display("DP-1", "Main")], DisplayLayout::Extended);
    let mock_persistence = Arc::new(MockPersistence::new(Some(initial_config.clone())));
    let service = DefaultDisplayConfigService::new(mock_persistence).await.unwrap();

    let retrieved_config = service.get_display_configuration().await.unwrap();
    assert_eq!(retrieved_config, initial_config);
}

#[tokio::test]
async fn test_apply_display_configuration_updates_state() {
    let mock_persistence = Arc::new(MockPersistence::new(None));
    let service = DefaultDisplayConfigService::new(mock_persistence).await.unwrap();

    let new_config = create_sample_display_config(vec![create_sample_display("HDMI-1", "Secondary")], DisplayLayout::Mirrored);
    service.apply_display_configuration(&new_config).await.unwrap();

    let current_service_config = service.get_display_configuration().await.unwrap();
    assert_eq!(current_service_config, new_config);
}

#[tokio::test]
async fn test_save_configuration_uses_persistence() {
    let mock_persistence = Arc::new(MockPersistence::new(None));
    let service = DefaultDisplayConfigService::new(mock_persistence.clone()).await.unwrap(); // Clone Arc for mock_persistence check

    let config_to_save = create_sample_display_config(vec![create_sample_display("LVDS-1", "Laptop Screen")], DisplayLayout::Single);
    service.apply_display_configuration(&config_to_save).await.unwrap(); // Apply to service state first
    service.save_configuration().await.unwrap();

    let persisted_config = mock_persistence.config.lock().unwrap().clone().unwrap();
    assert_eq!(persisted_config, config_to_save);
}

#[tokio::test]
async fn test_load_configuration_updates_state_and_returns() {
    let initial_persisted_config = create_sample_display_config(vec![create_sample_display("DP-2", "Aux")], DisplayLayout::Extended);
    let mock_persistence = Arc::new(MockPersistence::new(Some(initial_persisted_config.clone())));
    let service = DefaultDisplayConfigService::new(mock_persistence).await.unwrap(); // Initial load happens here

    // Simulate a different config being loaded from persistence later
    let new_persisted_config = create_sample_display_config(vec![create_sample_display("DVI-1", "Old Monitor")], DisplayLayout::Single);
    // Manually update the "persisted" config in the mock for the test
    let mut mock_persistence_guard = service.persistence.config.lock().unwrap();
    *mock_persistence_guard = Some(new_persisted_config.clone());
    drop(mock_persistence_guard);


    let loaded_config_result = service.load_configuration().await;
    assert!(loaded_config_result.is_ok());
    let loaded_config = loaded_config_result.unwrap();

    assert_eq!(loaded_config, new_persisted_config);
    let current_service_config = service.get_display_configuration().await.unwrap();
    assert_eq!(current_service_config, new_persisted_config);
}

#[tokio::test]
async fn test_update_single_display_config() {
    let display1 = create_sample_display("HDMI-1", "Monitor 1");
    let display2 = create_sample_display("DP-1", "Monitor 2");
    let initial_config = create_sample_display_config(vec![display1.clone(), display2.clone()], DisplayLayout::Extended);
    let mock_persistence = Arc::new(MockPersistence::new(Some(initial_config)));
    let service = DefaultDisplayConfigService::new(mock_persistence).await.unwrap();

    let mut updated_display1_config = display1.clone();
    updated_display1_config.enabled = false;
    updated_display1_config.position_x = 1920;

    service.update_single_display_config("HDMI-1", &updated_display1_config).await.unwrap();

    let current_full_config = service.get_display_configuration().await.unwrap();
    let changed_display_in_service = current_full_config.displays.iter().find(|d| d.id == "HDMI-1").unwrap();

    assert_eq!(changed_display_in_service.enabled, false);
    assert_eq!(changed_display_in_service.position_x, 1920);
    assert_eq!(current_full_config.displays.len(), 2); // Ensure other displays are still there
}

#[tokio::test]
async fn test_update_single_display_config_not_found() {
    let initial_config = create_sample_display_config(vec![create_sample_display("DP-1", "Primary")], DisplayLayout::Single);
    let mock_persistence = Arc::new(MockPersistence::new(Some(initial_config)));
    let service = DefaultDisplayConfigService::new(mock_persistence).await.unwrap();

    let non_existent_display_config = create_sample_display("HDMI-NONEXISTENT", "Ghost Monitor");
    let result = service.update_single_display_config("HDMI-NONEXISTENT", &non_existent_display_config).await;

    assert!(matches!(result, Err(DisplayConfigurationError::DisplayNotFound(_))));
}

#[tokio::test]
async fn test_set_layout() {
    let initial_config = create_sample_display_config(vec![create_sample_display("DP-1", "Primary")], DisplayLayout::Single);
    let mock_persistence = Arc::new(MockPersistence::new(Some(initial_config)));
    let service = DefaultDisplayConfigService::new(mock_persistence).await.unwrap();

    service.set_layout(DisplayLayout::Mirrored).await.unwrap();
    let current_config = service.get_display_configuration().await.unwrap();
    assert_eq!(current_config.layout, DisplayLayout::Mirrored);

    service.set_layout(DisplayLayout::Extended).await.unwrap();
    let current_config_2 = service.get_display_configuration().await.unwrap();
    assert_eq!(current_config_2.layout, DisplayLayout::Extended);
}

// Example of a validation test that could be added to apply_display_configuration
// This test is commented out because the validation logic itself is commented out in service.rs
/*
#[tokio::test]
async fn test_apply_display_configuration_validation_fail() {
    let mock_persistence = Arc::new(MockPersistence::new(None));
    let service = DefaultDisplayConfigService::new(mock_persistence).await.unwrap();

    // Invalid: No displays but layout is Extended
    let invalid_config = DisplayConfiguration {
        displays: vec![],
        layout: DisplayLayout::Extended,
    };

    let result = service.apply_display_configuration(&invalid_config).await;
    assert!(matches!(result, Err(DisplayConfigurationError::Validation(_))));
}
*/
