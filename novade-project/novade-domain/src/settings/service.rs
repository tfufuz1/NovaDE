//! Settings service module for the NovaDE domain layer.
//!
//! This module provides the settings service interface and implementation
//! for managing global settings in the NovaDE desktop environment.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::common_events::{DomainEvent, SettingsEvent};
use crate::error::{DomainResult, SettingsError};
use crate::settings::core::{Setting, SettingKey, SettingValue, SettingCategory};
use crate::settings::provider::SettingsProvider;

/// Interface for the settings service.
#[async_trait]
pub trait SettingsService: Send + Sync {
    /// Gets a setting by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    ///
    /// # Returns
    ///
    /// The setting, or an error if it doesn't exist.
    async fn get_setting(&self, key: &SettingKey) -> DomainResult<Setting>;

    /// Gets a setting value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    ///
    /// # Returns
    ///
    /// The setting value, or an error if the setting doesn't exist.
    async fn get_value(&self, key: &SettingKey) -> DomainResult<SettingValue>;

    /// Gets a setting value by key as a string.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    ///
    /// # Returns
    ///
    /// The setting value as a string, or an error if the setting doesn't exist or isn't a string.
    async fn get_string(&self, key: &SettingKey) -> DomainResult<String>;

    /// Gets a setting value by key as an integer.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    ///
    /// # Returns
    ///
    /// The setting value as an integer, or an error if the setting doesn't exist or isn't an integer.
    async fn get_integer(&self, key: &SettingKey) -> DomainResult<i64>;

    /// Gets a setting value by key as a floating-point number.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    ///
    /// # Returns
    ///
    /// The setting value as a floating-point number, or an error if the setting doesn't exist or isn't a floating-point number.
    async fn get_float(&self, key: &SettingKey) -> DomainResult<f64>;

    /// Gets a setting value by key as a boolean.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    ///
    /// # Returns
    ///
    /// The setting value as a boolean, or an error if the setting doesn't exist or isn't a boolean.
    async fn get_boolean(&self, key: &SettingKey) -> DomainResult<bool>;

    /// Gets all settings.
    ///
    /// # Returns
    ///
    /// A vector of all settings.
    async fn get_all_settings(&self) -> DomainResult<Vec<Setting>>;

    /// Gets settings by category.
    ///
    /// # Arguments
    ///
    /// * `category` - The category of the settings
    ///
    /// # Returns
    ///
    /// A vector of settings in the specified category.
    async fn get_settings_by_category(&self, category: SettingCategory) -> DomainResult<Vec<Setting>>;

    /// Sets a setting.
    ///
    /// # Arguments
    ///
    /// * `setting` - The setting to set
    ///
    /// # Returns
    ///
    /// The updated setting, or an error if the setting is read-only or invalid.
    async fn set_setting(&self, setting: Setting) -> DomainResult<Setting>;

    /// Sets a setting value by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    /// * `value` - The new value of the setting
    ///
    /// # Returns
    ///
    /// The updated setting, or an error if the setting is read-only or invalid.
    async fn set_value(&self, key: &SettingKey, value: impl Into<SettingValue>) -> DomainResult<Setting>;

    /// Resets a setting to its default value.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the setting
    ///
    /// # Returns
    ///
    /// The updated setting, or an error if the setting is read-only or doesn't exist.
    async fn reset_setting(&self, key: &SettingKey) -> DomainResult<Setting>;

    /// Resets all settings to their default values.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all settings were reset, or an error if any setting couldn't be reset.
    async fn reset_all_settings(&self) -> DomainResult<()>;

    /// Resets settings in a category to their default values.
    ///
    /// # Arguments
    ///
    /// * `category` - The category of the settings to reset
    ///
    /// # Returns
    ///
    /// `Ok(())` if all settings in the category were reset, or an error if any setting couldn't be reset.
    async fn reset_category(&self, category: SettingCategory) -> DomainResult<()>;

    /// Saves all settings.
    ///
    /// # Returns
    ///
    /// `Ok(())` if all settings were saved, or an error if saving failed.
    async fn save_settings(&self) -> DomainResult<()>;
}

/// Default implementation of the settings service.
pub struct DefaultSettingsService {
    /// The settings, keyed by key.
    settings: Arc<RwLock<HashMap<SettingKey, Setting>>>,
    /// The default settings, keyed by key.
    defaults: Arc<RwLock<HashMap<SettingKey, Setting>>>,
    /// The settings provider.
    provider: Arc<dyn SettingsProvider>,
    /// The event publisher function.
    event_publisher: Box<dyn Fn(DomainEvent<SettingsEvent>) + Send + Sync>,
}

impl DefaultSettingsService {
    /// Creates a new default settings service.
    ///
    /// # Arguments
    ///
    /// * `provider` - The settings provider
    /// * `event_publisher` - A function to publish settings events
    ///
    /// # Returns
    ///
    /// A new `DefaultSettingsService`.
    pub fn new<F>(
        provider: Arc<dyn SettingsProvider>,
        event_publisher: F,
    ) -> Self
    where
        F: Fn(DomainEvent<SettingsEvent>) + Send + Sync + 'static,
    {
        DefaultSettingsService {
            settings: Arc::new(RwLock::new(HashMap::new())),
            defaults: Arc::new(RwLock::new(HashMap::new())),
            provider,
            event_publisher: Box::new(event_publisher),
        }
    }

    /// Initializes the settings service with default settings.
    ///
    /// # Arguments
    ///
    /// * `defaults` - The default settings
    ///
    /// # Returns
    ///
    /// `Ok(())` if initialization succeeded, or an error if it failed.
    pub async fn initialize(&self, defaults: Vec<Setting>) -> DomainResult<()> {
        // Store the defaults
        {
            let mut defaults_map = self.defaults.write().unwrap();
            for setting in &defaults {
                defaults_map.insert(setting.key().clone(), setting.clone());
            }
        }

        // Load settings from the provider
        let loaded_settings = self.provider.load_settings().await?;
        let mut settings_map = HashMap::new();

        // Merge loaded settings with defaults
        for default in defaults {
            let key = default.key().clone();
            let setting = loaded_settings
                .iter()
                .find(|s| s.key() == &key)
                .cloned()
                .unwrap_or(default);

            settings_map.insert(key, setting);
        }

        // Store the merged settings
        {
            let mut settings = self.settings.write().unwrap();
            *settings = settings_map;
        }

        self.publish_event(SettingsEvent::SettingsInitialized);

        Ok(())
    }

    /// Publishes a settings event.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to publish
    fn publish_event(&self, event: SettingsEvent) {
        let domain_event = DomainEvent::new(event, "SettingsService");
        (self.event_publisher)(domain_event);
    }
}

#[async_trait]
impl SettingsService for DefaultSettingsService {
    async fn get_setting(&self, key: &SettingKey) -> DomainResult<Setting> {
        let settings = self.settings.read().unwrap();

        settings
            .get(key)
            .cloned()
            .ok_or_else(|| SettingsError::NotFound(key.to_string()).into())
    }

    async fn get_value(&self, key: &SettingKey) -> DomainResult<SettingValue> {
        let setting = self.get_setting(key).await?;
        Ok(setting.value().clone())
    }

    async fn get_string(&self, key: &SettingKey) -> DomainResult<String> {
        let value = self.get_value(key).await?;

        value
            .as_string()
            .map(|s| s.to_string())
            .ok_or_else(|| SettingsError::TypeMismatch(key.to_string(), "string".to_string()).into())
    }

    async fn get_integer(&self, key: &SettingKey) -> DomainResult<i64> {
        let value = self.get_value(key).await?;

        value
            .as_integer()
            .ok_or_else(|| SettingsError::TypeMismatch(key.to_string(), "integer".to_string()).into())
    }

    async fn get_float(&self, key: &SettingKey) -> DomainResult<f64> {
        let value = self.get_value(key).await?;

        value
            .as_float()
            .ok_or_else(|| SettingsError::TypeMismatch(key.to_string(), "float".to_string()).into())
    }

    async fn get_boolean(&self, key: &SettingKey) -> DomainResult<bool> {
        let value = self.get_value(key).await?;

        value
            .as_boolean()
            .ok_or_else(|| SettingsError::TypeMismatch(key.to_string(), "boolean".to_string()).into())
    }

    async fn get_all_settings(&self) -> DomainResult<Vec<Setting>> {
        let settings = self.settings.read().unwrap();
        let result: Vec<Setting> = settings.values().cloned().collect();
        Ok(result)
    }

    async fn get_settings_by_category(&self, category: SettingCategory) -> DomainResult<Vec<Setting>> {
        let settings = self.settings.read().unwrap();

        let result: Vec<Setting> = settings
            .values()
            .filter(|s| s.key().category == category)
            .cloned()
            .collect();

        Ok(result)
    }

    async fn set_setting(&self, setting: Setting) -> DomainResult<Setting> {
        setting.validate()?;

        let key = setting.key().clone();
        let key_str = key.to_string();

        {
            let mut settings = self.settings.write().unwrap();

            if let Some(existing) = settings.get(&key) {
                if existing.is_read_only() {
                    return Err(SettingsError::ReadOnly(key_str).into());
                }
            } else {
                return Err(SettingsError::NotFound(key_str).into());
            }

            settings.insert(key.clone(), setting.clone());
        }

        self.publish_event(SettingsEvent::SettingChanged {
            key: key_str.clone(),
        });

        Ok(setting)
    }

    async fn set_value(&self, key: &SettingKey, value: impl Into<SettingValue>) -> DomainResult<Setting> {
        let mut setting = self.get_setting(key).await?;
        setting.set_value(value)?;
        self.set_setting(setting).await
    }

    async fn reset_setting(&self, key: &SettingKey) -> DomainResult<Setting> {
        let defaults = self.defaults.read().unwrap();

        let default = defaults
            .get(key)
            .cloned()
            .ok_or_else(|| SettingsError::NotFound(key.to_string()).into())?;

        self.set_setting(default).await
    }

    async fn reset_all_settings(&self) -> DomainResult<()> {
        let keys = {
            let settings = self.settings.read().unwrap();
            settings.keys().cloned().collect::<Vec<_>>()
        };

        for key in keys {
            let _ = self.reset_setting(&key).await;
        }

        self.publish_event(SettingsEvent::AllSettingsReset);

        Ok(())
    }

    async fn reset_category(&self, category: SettingCategory) -> DomainResult<()> {
        let keys = {
            let settings = self.settings.read().unwrap();
            settings
                .keys()
                .filter(|k| k.category == category)
                .cloned()
                .collect::<Vec<_>>()
        };

        for key in keys {
            let _ = self.reset_setting(&key).await;
        }

        self.publish_event(SettingsEvent::CategorySettingsReset {
            category: category.to_string(),
        });

        Ok(())
    }

    async fn save_settings(&self) -> DomainResult<()> {
        let settings = {
            let settings = self.settings.read().unwrap();
            settings.values().cloned().collect::<Vec<_>>()
        };

        self.provider.save_settings(&settings).await?;

        self.publish_event(SettingsEvent::SettingsSaved);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        SettingsProvider {}

        #[async_trait]
        impl SettingsProvider for SettingsProvider {
            async fn load_settings(&self) -> DomainResult<Vec<Setting>>;
            async fn save_settings(&self, settings: &[Setting]) -> DomainResult<()>;
        }
    }

    struct TestContext {
        service: DefaultSettingsService,
        provider: Arc<MockSettingsProvider>,
        events: Arc<Mutex<Vec<SettingsEvent>>>,
    }

    impl TestContext {
        fn new() -> Self {
            let events = Arc::new(Mutex::new(Vec::new()));
            let events_clone = events.clone();

            let provider = Arc::new(MockSettingsProvider::new());

            let service = DefaultSettingsService::new(
                provider.clone(),
                move |event| {
                    let mut events = events_clone.lock().unwrap();
                    events.push(event.payload);
                },
            );

            TestContext {
                service,
                provider,
                events,
            }
        }

        fn get_events(&self) -> Vec<SettingsEvent> {
            let events = self.events.lock().unwrap();
            events.clone()
        }

        fn clear_events(&self) {
            let mut events = self.events.lock().unwrap();
            events.clear();
        }
    }

    #[tokio::test]
    async fn test_initialize() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_load_settings()
            .returning(|| Ok(Vec::new()));

        let defaults = vec![
            Setting::new(
                SettingKey::new(SettingCategory::General, "language"),
                "en-US",
            ),
            Setting::new(
                SettingKey::new(SettingCategory::Appearance, "theme"),
                "light",
            ),
        ];

        ctx.service.initialize(defaults).await.unwrap();

        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SettingsEvent::SettingsInitialized));

        let all_settings = ctx.service.get_all_settings().await.unwrap();
        assert_eq!(all_settings.len(), 2);
    }

    #[tokio::test]
    async fn test_get_setting() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_load_settings()
            .returning(|| Ok(Vec::new()));

        let key = SettingKey::new(SettingCategory::General, "language");
        let setting = Setting::new(key.clone(), "en-US");

        ctx.service.initialize(vec![setting.clone()]).await.unwrap();
        ctx.clear_events();

        let retrieved = ctx.service.get_setting(&key).await.unwrap();
        assert_eq!(retrieved.key(), &key);
        assert_eq!(retrieved.value(), &SettingValue::String("en-US".to_string()));

        let not_found = ctx.service
            .get_setting(&SettingKey::new(SettingCategory::General, "nonexistent"))
            .await;
        assert!(not_found.is_err());
    }

    #[tokio::test]
    async fn test_get_value_methods() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_load_settings()
            .returning(|| Ok(Vec::new()));

        let defaults = vec![
            Setting::new(
                SettingKey::new(SettingCategory::General, "language"),
                "en-US",
            ),
            Setting::new(
                SettingKey::new(SettingCategory::General, "count"),
                42i64,
            ),
            Setting::new(
                SettingKey::new(SettingCategory::General, "scale"),
                3.14f64,
            ),
            Setting::new(
                SettingKey::new(SettingCategory::General, "enabled"),
                true,
            ),
        ];

        ctx.service.initialize(defaults).await.unwrap();
        ctx.clear_events();

        // Test get_value
        let value = ctx.service
            .get_value(&SettingKey::new(SettingCategory::General, "language"))
            .await
            .unwrap();
        assert_eq!(value, SettingValue::String("en-US".to_string()));

        // Test get_string
        let string = ctx.service
            .get_string(&SettingKey::new(SettingCategory::General, "language"))
            .await
            .unwrap();
        assert_eq!(string, "en-US");

        // Test get_integer
        let integer = ctx.service
            .get_integer(&SettingKey::new(SettingCategory::General, "count"))
            .await
            .unwrap();
        assert_eq!(integer, 42);

        // Test get_float
        let float = ctx.service
            .get_float(&SettingKey::new(SettingCategory::General, "scale"))
            .await
            .unwrap();
        assert_eq!(float, 3.14);

        // Test get_boolean
        let boolean = ctx.service
            .get_boolean(&SettingKey::new(SettingCategory::General, "enabled"))
            .await
            .unwrap();
        assert!(boolean);

        // Test type mismatch
        let result = ctx.service
            .get_string(&SettingKey::new(SettingCategory::General, "count"))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_settings_by_category() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_load_settings()
            .returning(|| Ok(Vec::new()));

        let defaults = vec![
            Setting::new(
                SettingKey::new(SettingCategory::General, "language"),
                "en-US",
            ),
            Setting::new(
                SettingKey::new(SettingCategory::General, "region"),
                "US",
            ),
            Setting::new(
                SettingKey::new(SettingCategory::Appearance, "theme"),
                "light",
            ),
        ];

        ctx.service.initialize(defaults).await.unwrap();
        ctx.clear_events();

        let general_settings = ctx.service
            .get_settings_by_category(SettingCategory::General)
            .await
            .unwrap();
        assert_eq!(general_settings.len(), 2);

        let appearance_settings = ctx.service
            .get_settings_by_category(SettingCategory::Appearance)
            .await
            .unwrap();
        assert_eq!(appearance_settings.len(), 1);

        let behavior_settings = ctx.service
            .get_settings_by_category(SettingCategory::Behavior)
            .await
            .unwrap();
        assert_eq!(behavior_settings.len(), 0);
    }

    #[tokio::test]
    async fn test_set_setting() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_load_settings()
            .returning(|| Ok(Vec::new()));

        let key = SettingKey::new(SettingCategory::General, "language");
        let setting = Setting::new(key.clone(), "en-US");

        ctx.service.initialize(vec![setting.clone()]).await.unwrap();
        ctx.clear_events();

        let updated = Setting::new(key.clone(), "fr-FR");
        let result = ctx.service.set_setting(updated.clone()).await.unwrap();

        assert_eq!(result.value(), &SettingValue::String("fr-FR".to_string()));

        let retrieved = ctx.service.get_setting(&key).await.unwrap();
        assert_eq!(retrieved.value(), &SettingValue::String("fr-FR".to_string()));

        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SettingsEvent::SettingChanged { key: _ }));
    }

    #[tokio::test]
    async fn test_set_value() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_load_settings()
            .returning(|| Ok(Vec::new()));

        let key = SettingKey::new(SettingCategory::General, "language");
        let setting = Setting::new(key.clone(), "en-US");

        ctx.service.initialize(vec![setting.clone()]).await.unwrap();
        ctx.clear_events();

        let result = ctx.service.set_value(&key, "fr-FR").await.unwrap();

        assert_eq!(result.value(), &SettingValue::String("fr-FR".to_string()));

        let retrieved = ctx.service.get_setting(&key).await.unwrap();
        assert_eq!(retrieved.value(), &SettingValue::String("fr-FR".to_string()));

        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SettingsEvent::SettingChanged { key: _ }));
    }

    #[tokio::test]
    async fn test_reset_setting() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_load_settings()
            .returning(|| Ok(Vec::new()));

        let key = SettingKey::new(SettingCategory::General, "language");
        let setting = Setting::new(key.clone(), "en-US");

        ctx.service.initialize(vec![setting.clone()]).await.unwrap();
        ctx.clear_events();

        // Change the setting
        ctx.service.set_value(&key, "fr-FR").await.unwrap();
        ctx.clear_events();

        // Reset the setting
        let result = ctx.service.reset_setting(&key).await.unwrap();

        assert_eq!(result.value(), &SettingValue::String("en-US".to_string()));

        let retrieved = ctx.service.get_setting(&key).await.unwrap();
        assert_eq!(retrieved.value(), &SettingValue::String("en-US".to_string()));

        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SettingsEvent::SettingChanged { key: _ }));
    }

    #[tokio::test]
    async fn test_reset_all_settings() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_load_settings()
            .returning(|| Ok(Vec::new()));

        let defaults = vec![
            Setting::new(
                SettingKey::new(SettingCategory::General, "language"),
                "en-US",
            ),
            Setting::new(
                SettingKey::new(SettingCategory::Appearance, "theme"),
                "light",
            ),
        ];

        ctx.service.initialize(defaults).await.unwrap();
        ctx.clear_events();

        // Change the settings
        ctx.service
            .set_value(&SettingKey::new(SettingCategory::General, "language"), "fr-FR")
            .await
            .unwrap();
        ctx.service
            .set_value(&SettingKey::new(SettingCategory::Appearance, "theme"), "dark")
            .await
            .unwrap();
        ctx.clear_events();

        // Reset all settings
        ctx.service.reset_all_settings().await.unwrap();

        let language = ctx.service
            .get_string(&SettingKey::new(SettingCategory::General, "language"))
            .await
            .unwrap();
        assert_eq!(language, "en-US");

        let theme = ctx.service
            .get_string(&SettingKey::new(SettingCategory::Appearance, "theme"))
            .await
            .unwrap();
        assert_eq!(theme, "light");

        let events = ctx.get_events();
        assert!(events.iter().any(|e| matches!(e, SettingsEvent::AllSettingsReset)));
    }

    #[tokio::test]
    async fn test_reset_category() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_load_settings()
            .returning(|| Ok(Vec::new()));

        let defaults = vec![
            Setting::new(
                SettingKey::new(SettingCategory::General, "language"),
                "en-US",
            ),
            Setting::new(
                SettingKey::new(SettingCategory::General, "region"),
                "US",
            ),
            Setting::new(
                SettingKey::new(SettingCategory::Appearance, "theme"),
                "light",
            ),
        ];

        ctx.service.initialize(defaults).await.unwrap();
        ctx.clear_events();

        // Change the settings
        ctx.service
            .set_value(&SettingKey::new(SettingCategory::General, "language"), "fr-FR")
            .await
            .unwrap();
        ctx.service
            .set_value(&SettingKey::new(SettingCategory::General, "region"), "FR")
            .await
            .unwrap();
        ctx.service
            .set_value(&SettingKey::new(SettingCategory::Appearance, "theme"), "dark")
            .await
            .unwrap();
        ctx.clear_events();

        // Reset general settings
        ctx.service.reset_category(SettingCategory::General).await.unwrap();

        let language = ctx.service
            .get_string(&SettingKey::new(SettingCategory::General, "language"))
            .await
            .unwrap();
        assert_eq!(language, "en-US");

        let region = ctx.service
            .get_string(&SettingKey::new(SettingCategory::General, "region"))
            .await
            .unwrap();
        assert_eq!(region, "US");

        let theme = ctx.service
            .get_string(&SettingKey::new(SettingCategory::Appearance, "theme"))
            .await
            .unwrap();
        assert_eq!(theme, "dark"); // Not reset

        let events = ctx.get_events();
        assert!(events.iter().any(|e| matches!(e, SettingsEvent::CategorySettingsReset { category } if category == "general")));
    }

    #[tokio::test]
    async fn test_save_settings() {
        let ctx = TestContext::new();

        ctx.provider
            .expect_load_settings()
            .returning(|| Ok(Vec::new()));

        ctx.provider
            .expect_save_settings()
            .returning(|_| Ok(()));

        let defaults = vec![
            Setting::new(
                SettingKey::new(SettingCategory::General, "language"),
                "en-US",
            ),
            Setting::new(
                SettingKey::new(SettingCategory::Appearance, "theme"),
                "light",
            ),
        ];

        ctx.service.initialize(defaults).await.unwrap();
        ctx.clear_events();

        ctx.service.save_settings().await.unwrap();

        let events = ctx.get_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], SettingsEvent::SettingsSaved));
    }
}
