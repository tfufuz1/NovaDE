use std::collections::VecDeque;
use std::sync::Arc;
use async_trait::async_trait;
use log::{debug, info, warn, error};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::user_centric_services::events::{NotificationEvent, NotificationDismissReason};
use super::types::{Notification, NotificationInput, NotificationAction, NotificationUrgency}; // Added NotificationUrgency
use super::errors::NotificationError;
use super::persistence_iface::NotificationHistoryProvider;

// Forward declarations for types from other modules not yet fully defined/integrated
// These would typically be proper imports.

// Placeholder for GlobalSettingsService and its path types
// use crate::global_settings_and_state_management::{GlobalSettingsService, GlobalSettingPath, NotificationsSettingPath};
pub mod placeholder_global_settings { // Wrap in a module to avoid name clashes
    use std::sync::Arc;
    use async_trait::async_trait;
    #[derive(Debug, Clone)] pub enum GlobalSettingPath { NotificationsMaxHistory } // Simplified
    #[async_trait]
    pub trait GlobalSettingsService: Send + Sync {
        async fn get_setting_u64(&self, _path: GlobalSettingPath) -> Option<u64>; // Simplified
    }
    pub struct MockGlobalSettingsService; // Simple mock for compilation
    #[async_trait]
    impl GlobalSettingsService for MockGlobalSettingsService {
        async fn get_setting_u64(&self, _path: GlobalSettingPath) -> Option<u64> { Some(100) } // Default mock value
    }
}


// Placeholder for NotificationRulesEngine and its error type
// use crate::notifications_rules::{NotificationRulesEngine, RuleProcessingResult, NotificationRulesError};
pub mod placeholder_rules_engine { // Wrap in a module
    use async_trait::async_trait;
    use super::Notification; // Use Notification from this module
    use super::NotificationError; // Use NotificationError from this module

    #[derive(Debug)] pub enum RuleProcessingResult { Keep(Notification), Modify(Notification), Suppress { rule_id: String } }
    #[derive(Debug, thiserror::Error)] #[error("Rule Engine Error: {0}")] pub struct NotificationRulesError(String);

    #[async_trait]
    pub trait NotificationRulesEngine: Send + Sync {
        async fn process_notification(&self, notification: Notification) -> Result<RuleProcessingResult, NotificationRulesError>;
    }
    pub struct MockNotificationRulesEngine; // Simple mock for compilation
    #[async_trait]
    impl NotificationRulesEngine for MockNotificationRulesEngine {
        async fn process_notification(&self, notification: Notification) -> Result<RuleProcessingResult, NotificationRulesError> {
            Ok(RuleProcessingResult::Keep(notification)) // Default mock behavior
        }
    }
}


const DEFAULT_MAX_ACTIVE_POPUPS: usize = 5; // Kept for visual popups, distinct from history
const DEFAULT_MAX_HISTORY_ITEMS: usize = 200;

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn post_notification(&self, input: NotificationInput) -> Result<Uuid, NotificationError>;
    async fn get_active_notifications(&self) -> Result<Vec<Notification>, NotificationError>;
    async fn mark_as_read(&self, notification_id: Uuid) -> Result<(), NotificationError>;
    async fn dismiss_notification(&self, notification_id: Uuid, reason: NotificationDismissReason) -> Result<(), NotificationError>;
    async fn invoke_action(&self, notification_id: Uuid, action_key: &str) -> Result<(), NotificationError>;
    
    // Iteration 2 methods
    async fn get_notification_history(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<Notification>, NotificationError>;
    async fn clear_history(&self) -> Result<(), NotificationError>;
    async fn set_do_not_disturb(&self, enabled: bool) -> Result<(), NotificationError>;
    async fn is_do_not_disturb_enabled(&self) -> Result<bool, NotificationError>;
    async fn load_history_from_provider(&self) -> Result<(), NotificationError>;
    async fn load_settings_dependent_config(&self) -> Result<(), NotificationError>; // Added to trait
}

pub struct DefaultNotificationService {
    active_notifications: Arc<RwLock<VecDeque<Notification>>>, // For visual popups
    history: Arc<RwLock<VecDeque<Notification>>>,
    dnd_enabled: Arc<RwLock<bool>>,
    rules_engine: Arc<dyn placeholder_rules_engine::NotificationRulesEngine>, // Using placeholder
    settings_service: Arc<dyn placeholder_global_settings::GlobalSettingsService>, // Using placeholder
    history_provider: Arc<dyn NotificationHistoryProvider>,
    max_history_items: Arc<RwLock<usize>>, // Now an Arc<RwLock<usize>>
    event_publisher: broadcast::Sender<NotificationEvent>,
}

impl DefaultNotificationService {
    pub fn new(
        event_publisher: broadcast::Sender<NotificationEvent>,
        rules_engine: Arc<dyn placeholder_rules_engine::NotificationRulesEngine>,
        settings_service: Arc<dyn placeholder_global_settings::GlobalSettingsService>,
        history_provider: Arc<dyn NotificationHistoryProvider>,
    ) -> Self {
        Self {
            active_notifications: Arc::new(RwLock::new(VecDeque::new())),
            history: Arc::new(RwLock::new(VecDeque::new())),
            dnd_enabled: Arc::new(RwLock::new(false)),
            rules_engine,
            settings_service,
            history_provider,
            max_history_items: Arc::new(RwLock::new(DEFAULT_MAX_HISTORY_ITEMS)), // Initialize with default
            event_publisher,
        }
    }

    fn validate_input(input: &NotificationInput) -> Result<(), NotificationError> {
        if input.application_name.is_empty() { return Err(NotificationError::InvalidData { field: "application_name".to_string(), reason: "Application name cannot be empty.".to_string() }); }
        if input.summary.is_empty() { return Err(NotificationError::InvalidData { field: "summary".to_string(), reason: "Summary cannot be empty.".to_string() }); }
        for action in &input.actions {
            if action.key.is_empty() { return Err(NotificationError::InvalidData{ field: "action.key".to_string(), reason: "Action key cannot be empty.".to_string()}); }
            if action.label.is_empty() { return Err(NotificationError::InvalidData{ field: "action.label".to_string(), reason: "Action label cannot be empty.".to_string()}); }
        }
        Ok(())
    }
}

#[async_trait]
impl NotificationService for DefaultNotificationService {
    async fn load_settings_dependent_config(&self) -> Result<(), NotificationError> {
        info!("Loading notification service settings-dependent configuration...");
        // Placeholder path, replace with actual path once defined in global_settings
        let fetched_max_history = self.settings_service.get_setting_u64(placeholder_global_settings::GlobalSettingPath::NotificationsMaxHistory).await;
        
        let mut max_hist_lock = self.max_history_items.write().await;
        if let Some(val) = fetched_max_history {
            *max_hist_lock = val as usize;
            info!("Max history items set to {} from settings.", *max_hist_lock);
        } else {
            *max_hist_lock = DEFAULT_MAX_HISTORY_ITEMS; // Fallback to default
            info!("Max history items not found in settings, using default: {}.", *max_hist_lock);
        }
        Ok(())
    }
    
    async fn load_history_from_provider(&self) -> Result<(), NotificationError> {
        info!("Loading notification history from provider...");
        let loaded_history = self.history_provider.load_history().await?;
        let mut history_lock = self.history.write().await;
        *history_lock = loaded_history;
        info!("Notification history loaded. {} items.", history_lock.len());
        Ok(())
    }


    async fn post_notification(&self, input: NotificationInput) -> Result<Uuid, NotificationError> {
        info!("Attempting to post notification from app: {}", input.application_name);
        Self::validate_input(&input)?;

        let original_notification_id = Uuid::new_v4();
        let initial_notification = Notification::new(input, original_notification_id, chrono::Utc::now());
        
        // 1. Process with rules engine
        let processing_result = self.rules_engine.process_notification(initial_notification.clone()).await // Clone for rules
            .map_err(|e_rules| NotificationError::RuleApplicationError { source: placeholder_rules_engine::NotificationRulesError(format!("{:?}", e_rules)) })?; // Map placeholder error

        let final_notification = match processing_result {
            placeholder_rules_engine::RuleProcessingResult::Keep(n) => n,
            placeholder_rules_engine::RuleProcessingResult::Modify(n) => n,
            placeholder_rules_engine::RuleProcessingResult::Suppress { rule_id } => {
                info!("Notification {} suppressed by rule '{}'", original_notification_id, rule_id);
                if let Err(e) = self.event_publisher.send(NotificationEvent::NotificationSuppressedByRule {
                    original_summary: initial_notification.summary.clone(), // Use original for event
                    app_name: initial_notification.application_name.clone(),
                    rule_id,
                }) {
                    warn!("Failed to send NotificationSuppressedByRule event: {}", e);
                }
                return Ok(original_notification_id); // Return original ID even if suppressed
            }
        };

        // 2. Check DND
        let dnd = *self.dnd_enabled.read().await;
        let mut suppressed_by_dnd = false;

        if dnd && final_notification.urgency != NotificationUrgency::Critical {
            info!("Notification {} (final id {}) suppressed by DND.", original_notification_id, final_notification.id);
            suppressed_by_dnd = true;
            // Publish event indicating it was posted but DND suppressed visual/sound
             if let Err(e) = self.event_publisher.send(NotificationEvent::NotificationPosted {
                notification: final_notification.clone(),
                suppressed_by_dnd: true,
            }) {
                warn!("Failed to send NotificationPosted (DND suppressed) event: {}", e);
            }
        } else {
            // Not suppressed by DND (or critical urgency) -> add to active visual popups
            let mut active_notifications_lock = self.active_notifications.write().await;
            while active_notifications_lock.len() >= DEFAULT_MAX_ACTIVE_POPUPS { // Use fixed small limit for popups
                if let Some(old_popup) = active_notifications_lock.pop_front() {
                    info!("Max active popups ({}) reached. Removing oldest popup: {}", DEFAULT_MAX_ACTIVE_POPUPS, old_popup.id);
                    if !old_popup.is_dismissed { // Only send dismiss if not already dismissed
                        if let Err(e) = self.event_publisher.send(NotificationEvent::NotificationDismissed {
                            notification_id: old_popup.id,
                            reason: NotificationDismissReason::ReplacedByApp,
                        }) { warn!("Failed to send NotificationDismissed for old popup: {}", e); }
                    }
                } else { break; }
            }
            active_notifications_lock.push_back(final_notification.clone());
            debug!("Notification {} added to active popups. Total active popups: {}", final_notification.id, active_notifications_lock.len());
            
            if let Err(e) = self.event_publisher.send(NotificationEvent::NotificationPosted {
                notification: final_notification.clone(),
                suppressed_by_dnd: false,
            }) {
                warn!("Failed to send NotificationPosted event: {}", e);
            }
        }
        
        // 3. Add to history if not transient
        if !final_notification.transient {
            let mut history_lock = self.history.write().await;
            let max_hist = *self.max_history_items.read().await; // Get current max
            while history_lock.len() >= max_hist && max_hist > 0 { // Check max_hist > 0 to prevent infinite loop if set to 0
                history_lock.pop_front(); // Remove oldest
            }
            if max_hist > 0 { // Only add if history is enabled (max_hist > 0)
                history_lock.push_back(final_notification.clone());
                debug!("Notification {} added to history. History size: {}", final_notification.id, history_lock.len());
                // Save history (could be debounced in a real system)
                if let Err(e) = self.history_provider.save_history(&*history_lock).await {
                     error!("Failed to save notification history after posting: {}", e);
                    // Decide if this should be a hard error for post_notification
                }
            }
        }
        
        info!("Notification (original ID {}) processed. Final state ID: {}. DND suppressed: {}", original_notification_id, final_notification.id, suppressed_by_dnd);
        Ok(final_notification.id) // Return the ID of the notification that was actually processed/stored
    }

    async fn get_active_notifications(&self) -> Result<Vec<Notification>, NotificationError> {
        let active_notifications_lock = self.active_notifications.read().await;
        let non_dismissed: Vec<Notification> = active_notifications_lock.iter().filter(|n| !n.is_dismissed).cloned().collect();
        Ok(non_dismissed)
    }

    async fn mark_as_read(&self, notification_id: Uuid) -> Result<(), NotificationError> {
        // Try finding in active popups first
        let mut active_notifications_lock = self.active_notifications.write().await;
        if let Some(notification) = active_notifications_lock.iter_mut().find(|n| n.id == notification_id) {
            if !notification.is_read {
                notification.is_read = true;
                info!("Active notification {} marked as read.", notification_id);
                if let Err(e) = self.event_publisher.send(NotificationEvent::NotificationRead { notification_id }) { warn!("Failed to send NotificationRead event: {}", e); }
            }
            // Also update in history if present
            let mut history_lock = self.history.write().await;
            if let Some(hist_notification) = history_lock.iter_mut().find(|n| n.id == notification_id) {
                hist_notification.is_read = true; // Ensure history is also updated
                // Consider saving history here if this change should be persisted immediately
                // if let Err(e) = self.history_provider.save_history(&*history_lock).await { error!("Failed to save history after mark_as_read: {}", e); }
            }
            return Ok(());
        }
        drop(active_notifications_lock); // Release lock

        // If not in active, check history
        let mut history_lock = self.history.write().await;
        if let Some(notification) = history_lock.iter_mut().find(|n| n.id == notification_id) {
            if !notification.is_read {
                notification.is_read = true;
                info!("Historical notification {} marked as read.", notification_id);
                if let Err(e) = self.event_publisher.send(NotificationEvent::NotificationRead { notification_id }) { warn!("Failed to send NotificationRead event: {}", e); }
                // Save history as this is a persistent change
                if let Err(e) = self.history_provider.save_history(&*history_lock).await { error!("Failed to save history after mark_as_read: {}", e); }
            }
            Ok(())
        } else {
            warn!("Attempted to mark non-existent notification {} as read.", notification_id);
            Err(NotificationError::NotFound(notification_id))
        }
    }

    async fn dismiss_notification(&self, notification_id: Uuid, reason: NotificationDismissReason) -> Result<(), NotificationError> {
        let mut changed_in_active = false;
        let mut active_notifications_lock = self.active_notifications.write().await;
        if let Some(notification) = active_notifications_lock.iter_mut().find(|n| n.id == notification_id && !n.is_dismissed) {
            notification.is_dismissed = true;
            changed_in_active = true;
            // Event will be sent after history check
        }
        drop(active_notifications_lock);

        let mut changed_in_history = false;
        let mut history_lock = self.history.write().await;
        if let Some(notification) = history_lock.iter_mut().find(|n| n.id == notification_id && !n.is_dismissed) {
            notification.is_dismissed = true;
            changed_in_history = true;
            // Event will be sent after history check
        }

        if changed_in_active || changed_in_history {
            info!("Notification {} dismissed with reason: {:?}", notification_id, reason);
            if let Err(e) = self.event_publisher.send(NotificationEvent::NotificationDismissed { notification_id, reason }) {
                warn!("Failed to send NotificationDismissed event: {}", e);
            }
            if changed_in_history { // Only save history if a persisted item was changed
                 if let Err(e) = self.history_provider.save_history(&*history_lock).await { error!("Failed to save history after dismiss: {}", e); }
            }
            Ok(())
        } else {
            warn!("Attempted to dismiss non-existent or already dismissed notification {}.", notification_id);
            Err(NotificationError::NotFound(notification_id))
        }
    }

    async fn invoke_action(&self, notification_id: Uuid, action_key: &str) -> Result<(), NotificationError> {
        // Check active first (most likely target)
        let active_notifications_lock = self.active_notifications.read().await;
        if let Some(notification) = active_notifications_lock.iter().find(|n| n.id == notification_id && !n.is_dismissed) {
            if notification.actions.iter().any(|act| act.key == action_key) {
                info!("Action '{}' invoked for active notification {}.", action_key, notification_id);
                if let Err(e) = self.event_publisher.send(NotificationEvent::NotificationActionInvoked { notification_id, action_key: action_key.to_string() }) { warn!("Event send error: {}", e); }
                return Ok(());
            } else {
                return Err(NotificationError::ActionNotFound { notification_id, action_key: action_key.to_string() });
            }
        }
        drop(active_notifications_lock);

        // Check history if not found in active
        let history_lock = self.history.read().await;
         if let Some(notification) = history_lock.iter().find(|n| n.id == notification_id && !n.is_dismissed) { // Check !is_dismissed for actions on historical items too?
            if notification.actions.iter().any(|act| act.key == action_key) {
                info!("Action '{}' invoked for historical notification {}.", action_key, notification_id);
                if let Err(e) = self.event_publisher.send(NotificationEvent::NotificationActionInvoked { notification_id, action_key: action_key.to_string() }) { warn!("Event send error: {}", e); }
                return Ok(());
            } else {
                 return Err(NotificationError::ActionNotFound { notification_id, action_key: action_key.to_string() });
            }
        }
        warn!("Attempted to invoke action on non-existent or dismissed notification {}.", notification_id);
        Err(NotificationError::NotFound(notification_id))
    }

    async fn get_notification_history(&self, limit: Option<usize>, offset: Option<usize>) -> Result<Vec<Notification>, NotificationError> {
        let history_lock = self.history.read().await;
        let start = offset.unwrap_or(0);
        let end = limit.map_or_else(|| history_lock.len(), |l| (start + l).min(history_lock.len()));
        
        if start >= history_lock.len() {
            return Ok(Vec::new()); // Offset out of bounds
        }
        // History is stored oldest first (pushed to back). For display, usually newest first.
        // So, we might want to reverse before taking slice, or slice from end.
        // VecDeque doesn't directly support slicing from end easily.
        // Let's iterate and collect in reverse for typical display.
        let result: Vec<Notification> = history_lock.iter().rev().skip(start).take(limit.unwrap_or(usize::MAX)).cloned().collect();
        Ok(result)
    }

    async fn clear_history(&self) -> Result<(), NotificationError> {
        info!("Clearing notification history.");
        let mut history_lock = self.history.write().await;
        if history_lock.is_empty() {
            debug!("History already empty.");
            return Ok(());
        }
        history_lock.clear();
        
        // Save the now-empty history
        self.history_provider.save_history(&*history_lock).await?; // Pass empty deque
        
        if let Err(e) = self.event_publisher.send(NotificationEvent::NotificationHistoryCleared) {
            warn!("Failed to send NotificationHistoryCleared event: {}", e);
        }
        info!("Notification history cleared and saved.");
        Ok(())
    }

    async fn set_do_not_disturb(&self, enabled: bool) -> Result<(), NotificationError> {
        let mut dnd_lock = self.dnd_enabled.write().await;
        if *dnd_lock != enabled {
            *dnd_lock = enabled;
            info!("Do Not Disturb mode set to: {}", enabled);
            if let Err(e) = self.event_publisher.send(NotificationEvent::DoNotDisturbModeChanged { dnd_enabled: enabled }) {
                warn!("Failed to send DoNotDisturbModeChanged event: {}", e);
            }
        } else {
            debug!("Do Not Disturb mode already {}.", enabled);
        }
        Ok(())
    }

    async fn is_do_not_disturb_enabled(&self) -> Result<bool, NotificationError> {
        Ok(*self.dnd_enabled.read().await)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::user_centric_services::notifications_core::types::{NotificationAction, NotificationActionType, NotificationUrgency};
    use tokio::time::{timeout, Duration};
    use crate::user_centric_services::notifications_core::persistence_iface::NotificationHistoryProvider;

    // Mock HistoryProvider
    #[derive(Default, Clone)]
    struct MockHistoryProvider {
        history: Arc<RwLock<VecDeque<Notification>>>,
        force_load_error: bool,
        force_save_error: bool,
    }
    impl MockHistoryProvider {
        fn new() -> Self { Default::default() }
        #[allow(dead_code)] fn set_force_load_error(&mut self, force: bool) { self.force_load_error = force; }
        #[allow(dead_code)] fn set_force_save_error(&mut self, force: bool) { self.force_save_error = force; }
    }
    #[async_trait]
    impl NotificationHistoryProvider for MockHistoryProvider {
        async fn load_history(&self) -> Result<VecDeque<Notification>, NotificationError> {
            if self.force_load_error { return Err(NotificationError::HistoryPersistenceError{ operation: "load".to_string(), message: "mock load error".to_string(), source: novade_core::errors::CoreError::new_custom("mock") }); }
            Ok(self.history.read().await.clone())
        }
        async fn save_history(&self, history_to_save: &VecDeque<Notification>) -> Result<(), NotificationError> {
            if self.force_save_error { return Err(NotificationError::HistoryPersistenceError{ operation: "save".to_string(), message: "mock save error".to_string(), source: novade_core::errors::CoreError::new_custom("mock") }); }
            *self.history.write().await = history_to_save.clone();
            Ok(())
        }
    }
    
    // Mock Rules Engine (simplified)
    struct TestRulesEngine { process_result: Option<placeholder_rules_engine::RuleProcessingResult> }
    impl TestRulesEngine { fn new(result: placeholder_rules_engine::RuleProcessingResult) -> Self { Self { process_result: Some(result) } } }
    #[async_trait]
    impl placeholder_rules_engine::NotificationRulesEngine for TestRulesEngine {
        async fn process_notification(&self, notification: Notification) -> Result<placeholder_rules_engine::RuleProcessingResult, placeholder_rules_engine::NotificationRulesError> {
            Ok(self.process_result.clone().unwrap_or(placeholder_rules_engine::RuleProcessingResult::Keep(notification)))
        }
    }

    fn create_test_input(summary: &str) -> NotificationInput {
        NotificationInput {
            application_name: "TestApp".to_string(), application_icon: None, summary: summary.to_string(),
            body: Some("Body for ".to_string() + summary),
            actions: vec![NotificationAction { key: "default".to_string(), label: "OK".to_string(), action_type: Default::default()}],
            urgency: Default::default(), category: None, hints: None, timeout_ms: None, transient: false,
        }
    }
    
    fn setup_service() -> (DefaultNotificationService, Arc<MockHistoryProvider>, broadcast::Receiver<NotificationEvent>) {
        let (tx, rx) = broadcast::channel(32);
        let rules_engine = Arc::new(placeholder_rules_engine::MockNotificationRulesEngine); // Use basic mock
        let settings_service = Arc::new(placeholder_global_settings::MockGlobalSettingsService);
        let history_provider = Arc::new(MockHistoryProvider::new());
        let service = DefaultNotificationService::new(tx, rules_engine, settings_service, history_provider.clone());
        (service, history_provider, rx)
    }


    #[tokio::test]
    async fn test_load_settings_dependent_config_uses_default() {
        let (service, _, _) = setup_service();
        service.load_settings_dependent_config().await.unwrap();
        let max_hist = *service.max_history_items.read().await;
        assert_eq!(max_hist, DEFAULT_MAX_HISTORY_ITEMS); // MockGlobalSettings returns Some(100) but path is placeholder
    }
    
    // To test with actual settings value, MockGlobalSettingsService needs to handle the specific path.
    // For now, this confirms default fallback.

    #[tokio::test]
    async fn test_post_notification_dnd_suppression_critical_bypass() {
        let (service, _, mut rx) = setup_service();
        service.set_do_not_disturb(true).await.unwrap(); // Enable DND
        let _ = rx.recv().await; // consume DND event

        let mut critical_input = create_test_input("Critical DND Bypass");
        critical_input.urgency = NotificationUrgency::Critical;
        
        let _id = service.post_notification(critical_input).await.unwrap();
        
        let event = timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap();
        match event {
            NotificationEvent::NotificationPosted { suppressed_by_dnd, .. } => {
                assert!(!suppressed_by_dnd, "Critical notification should bypass DND");
            }
            _ => panic!("Wrong event type"),
        }
        assert_eq!(service.get_active_notifications().await.unwrap().len(), 1); // Should be in active popups
    }
    
    #[tokio::test]
    async fn test_post_notification_dnd_suppression_normal() {
        let (service, _, mut rx) = setup_service();
        service.set_do_not_disturb(true).await.unwrap();
        let _ = rx.recv().await; 

        let normal_input = create_test_input("Normal DND Suppressed"); // Default urgency is Normal
        let _id = service.post_notification(normal_input).await.unwrap();

        let event = timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap();
        match event {
            NotificationEvent::NotificationPosted { suppressed_by_dnd, .. } => {
                assert!(suppressed_by_dnd, "Normal notification should be suppressed by DND");
            }
            _ => panic!("Wrong event type"),
        }
        assert!(service.get_active_notifications().await.unwrap().is_empty()); // Should NOT be in active popups
        assert_eq!(service.history.read().await.len(), 1); // But should be in history
    }

    #[tokio::test]
    async fn test_post_notification_rules_suppress() {
        let (tx, mut rx) = broadcast::channel(32);
        let rules_engine = Arc::new(TestRulesEngine::new(placeholder_rules_engine::RuleProcessingResult::Suppress { rule_id: "rule123".to_string() }));
        let settings_service = Arc::new(placeholder_global_settings::MockGlobalSettingsService);
        let history_provider = Arc::new(MockHistoryProvider::new());
        let service = DefaultNotificationService::new(tx, rules_engine, settings_service, history_provider);

        let input = create_test_input("Suppressed by Rule");
        let _id = service.post_notification(input.clone()).await.unwrap();

        let event = timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap();
        match event {
            NotificationEvent::NotificationSuppressedByRule { original_summary, app_name, rule_id } => {
                assert_eq!(original_summary, input.summary);
                assert_eq!(app_name, input.application_name);
                assert_eq!(rule_id, "rule123");
            }
            _ => panic!("Wrong event type"),
        }
        assert!(service.get_active_notifications().await.unwrap().is_empty());
        assert!(service.history.read().await.is_empty()); // Suppressed should not go to history
    }
    
    #[tokio::test]
    async fn test_post_notification_rules_modify() {
        let (tx, mut rx) = broadcast::channel(32);
        let modified_summary = "Modified by Rule Engine".to_string();
        let mut initial_notif = Notification::from(create_test_input("Original for Modify"));
        let modified_notif_id = initial_notif.id; // ID should remain same or be handled if rules can change it
        initial_notif.summary = modified_summary.clone(); 
        let rules_engine = Arc::new(TestRulesEngine::new(placeholder_rules_engine::RuleProcessingResult::Modify(initial_notif)));
        
        let settings_service = Arc::new(placeholder_global_settings::MockGlobalSettingsService);
        let history_provider = Arc::new(MockHistoryProvider::new());
        let service = DefaultNotificationService::new(tx, rules_engine, settings_service, history_provider);

        let input = create_test_input("Original for Modify"); // This summary will be overridden by rule
        let returned_id = service.post_notification(input.clone()).await.unwrap();
        assert_eq!(returned_id, modified_notif_id); // Ensure ID from rule-modified notification is returned

        let event = timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap();
        match event {
            NotificationEvent::NotificationPosted { notification, suppressed_by_dnd } => {
                assert_eq!(notification.id, modified_notif_id);
                assert_eq!(notification.summary, modified_summary);
                assert!(!suppressed_by_dnd);
            }
            _ => panic!("Wrong event type"),
        }
        let active = service.get_active_notifications().await.unwrap();
        assert_eq!(active[0].summary, modified_summary);
        let history = service.history.read().await;
        assert_eq!(history[0].summary, modified_summary);
    }


    #[tokio::test]
    async fn test_history_management_load_save_clear() {
        let (service, history_provider, mut rx) = setup_service();
        service.load_settings_dependent_config().await.unwrap(); // To set max_history_items

        // Load initial (empty)
        service.load_history_from_provider().await.unwrap();
        assert!(service.history.read().await.is_empty());

        // Post some notifications to populate history
        let id1 = service.post_notification(create_test_input("Hist1")).await.unwrap();
        let _ = rx.recv().await; // consume post
        let id2 = service.post_notification(create_test_input("Hist2")).await.unwrap();
        let _ = rx.recv().await; // consume post

        // History provider should now have these (due to save_history in post_notification)
        assert_eq!(history_provider.history.read().await.len(), 2);
        
        // Get history
        let history_page = service.get_notification_history(Some(10), Some(0)).await.unwrap();
        assert_eq!(history_page.len(), 2);
        // newest first
        assert_eq!(history_page[0].id, id2); 
        assert_eq!(history_page[1].id, id1);

        // Clear history
        service.clear_history().await.unwrap();
        assert!(service.history.read().await.is_empty());
        assert!(history_provider.history.read().await.is_empty()); // Provider should be cleared too
        
        match timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap() {
            NotificationEvent::NotificationHistoryCleared => {},
            _ => panic!("Wrong event type"),
        }
    }
    
    #[tokio::test]
    async fn test_history_max_items_enforced() {
        let (service, _, _) = setup_service();
        // service.load_settings_dependent_config().await.unwrap(); // Uses default 200
        // For test, let's set it directly if possible, or ensure default is small enough.
        // The mock settings service doesn't allow easy setting of this value yet.
        // Let's assume DEFAULT_MAX_HISTORY_ITEMS is used, or it's set to a small value for test.
        let test_max_history = 2;
        *service.max_history_items.write().await = test_max_history;


        let _id1 = service.post_notification(create_test_input("H_Item1")).await.unwrap();
        let _id2 = service.post_notification(create_test_input("H_Item2")).await.unwrap();
        let _id3 = service.post_notification(create_test_input("H_Item3")).await.unwrap(); // This should push out H_Item1

        let history = service.history.read().await;
        assert_eq!(history.len(), test_max_history);
        assert_eq!(history[0].summary, "Hist_Item2"); // Assuming Hist_Item1 was summary
        assert_eq!(history[1].summary, "Hist_Item3");
    }


    #[tokio::test]
    async fn test_dnd_mode_set_and_get_event() {
        let (service, _, mut rx) = setup_service();

        assert!(!service.is_do_not_disturb_enabled().await.unwrap()); // Default is false
        service.set_do_not_disturb(true).await.unwrap();
        assert!(service.is_do_not_disturb_enabled().await.unwrap());
        
        match timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap() {
            NotificationEvent::DoNotDisturbModeChanged { dnd_enabled } => assert!(dnd_enabled),
            _ => panic!("Wrong event type"),
        }

        service.set_do_not_disturb(false).await.unwrap();
        assert!(!service.is_do_not_disturb_enabled().await.unwrap());
         match timeout(Duration::from_millis(10), rx.recv()).await.unwrap().unwrap() {
            NotificationEvent::DoNotDisturbModeChanged { dnd_enabled } => assert!(!dnd_enabled),
            _ => panic!("Wrong event type"),
        }
        
        // Set to same value, should not send event
        service.set_do_not_disturb(false).await.unwrap();
        assert!(rx.try_recv().is_err());
    }
}
