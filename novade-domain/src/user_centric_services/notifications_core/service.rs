use async_trait::async_trait;
use serde_json::Value as JsonValue;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;
use chrono::Utc;
use tracing::{debug, error, info, warn};

use super::types::{
    Notification, NotificationInput, NotificationAction, NotificationUrgency, NotificationStats,
    DismissReason, NotificationFilterCriteria, NotificationSortOrder,
};
use super::errors::NotificationError;
use crate::user_centric_services::events::NotificationEventEnum;
use crate::notifications_rules::{NotificationRulesEngine, RuleProcessingResult, errors::NotificationRulesError};
use crate::global_settings::{
    GlobalSettingsService, 
    paths::SettingPath, // Assuming SettingPath can represent notification settings paths
    // types::GlobalDesktopSettings, // Not directly used if paths are specific enough
};
use crate::shared_types::ApplicationId;

// Placeholder for where notification-specific settings might live within GlobalSettings
// For now, these are effectively constants if not read from GlobalSettingsService
const DEFAULT_MAX_ACTIVE_POPUPS: usize = 5;
const DEFAULT_MAX_HISTORY_ITEMS: usize = 100;

// Placeholder for setting paths if they were to be defined in global_settings::paths
// For example:
// pub enum NotificationsCoreSettingPath { MaxActivePopups, MaxHistoryItems, DefaultTimeout }
// Then SettingPath::NotificationsCore(NotificationsCoreSettingPath::MaxActivePopups)
// For this implementation, we'll assume settings_service.get_setting can take a string path
// or these values are managed differently (e.g. hardcoded or via a dedicated config struct).


// --- NotificationService Trait ---

#[async_trait]
pub trait NotificationService: Send + Sync {
    async fn post_notification(&self, notification_input: NotificationInput) -> Result<Uuid, NotificationError>;
    async fn get_notification(&self, notification_id: Uuid) -> Result<Option<Notification>, NotificationError>;
    async fn mark_as_read(&self, notification_id: Uuid) -> Result<(), NotificationError>;
    async fn dismiss_notification(&self, notification_id: Uuid, reason: DismissReason) -> Result<(), NotificationError>;
    async fn get_active_notifications(&self, filter: Option<&NotificationFilterCriteria>, sort_order: Option<NotificationSortOrder>) -> Result<Vec<Notification>, NotificationError>;
    async fn get_notification_history(&self, limit: Option<usize>, offset: Option<usize>, filter: Option<&NotificationFilterCriteria>, sort_order: Option<NotificationSortOrder>) -> Result<Vec<Notification>, NotificationError>;
    async fn clear_history(&self) -> Result<(), NotificationError>;
    async fn clear_all_for_app(&self, app_id: &ApplicationId, reason: DismissReason) -> Result<usize, NotificationError>;
    async fn set_do_not_disturb(&self, enabled: bool) -> Result<(), NotificationError>;
    async fn is_do_not_disturb_enabled(&self) -> Result<bool, NotificationError>;
    async fn invoke_action(&self, notification_id: Uuid, action_key: &str) -> Result<(), NotificationError>;
    async fn get_stats(&self) -> Result<NotificationStats, NotificationError>;
    fn subscribe_to_notification_events(&self) -> broadcast::Receiver<NotificationEventEnum>;
}

// --- DefaultNotificationService Implementation ---

pub struct DefaultNotificationService {
    active_notifications: Arc<RwLock<VecDeque<Notification>>>,
    history: Arc<RwLock<VecDeque<Notification>>>,
    dnd_enabled: Arc<RwLock<bool>>,
    rules_engine: Arc<dyn NotificationRulesEngine>,
    settings_service: Arc<dyn GlobalSettingsService>,
    event_publisher: broadcast::Sender<NotificationEventEnum>,
    max_active_popups_cache: Arc<RwLock<usize>>,
    max_history_items_cache: Arc<RwLock<usize>>,
}

impl DefaultNotificationService {
    pub async fn new(
        rules_engine: Arc<dyn NotificationRulesEngine>,
        settings_service: Arc<dyn GlobalSettingsService>,
        broadcast_capacity: usize,
    ) -> Result<Self, NotificationError> {
        let (event_publisher, _) = broadcast::channel(broadcast_capacity);
        let service = Self {
            active_notifications: Arc::new(RwLock::new(VecDeque::new())),
            history: Arc::new(RwLock::new(VecDeque::new())),
            dnd_enabled: Arc::new(RwLock::new(false)),
            rules_engine,
            settings_service,
            event_publisher,
            max_active_popups_cache: Arc::new(RwLock::new(DEFAULT_MAX_ACTIVE_POPUPS)),
            max_history_items_cache: Arc::new(RwLock::new(DEFAULT_MAX_HISTORY_ITEMS)),
        };
        service.load_settings_cache().await?;
        Ok(service)
    }

    async fn load_settings_cache(&self) -> Result<(), NotificationError> {
        debug!("Loading notification settings cache...");
        // Using string paths for get_setting as SettingPath::NotificationsCore(...) isn't defined yet.
        // This assumes GlobalSettingsService can handle string paths or these are mapped.
        // If these settings are not found, defaults are used.
        
        // Path for MaxActivePopups (example string path)
        let max_popups_path_str = "notifications.max_active_popups"; 
        match self.settings_service.get_setting(&SettingPath::from_str_unsafe_for_testing(max_popups_path_str)).await {
            Ok(JsonValue::Number(num)) => {
                if let Some(val) = num.as_u64() { *self.max_active_popups_cache.write().await = val as usize; }
            }
            Ok(_) | Err(_) => { warn!("Could not read '{}' from global settings or invalid type, using default: {}", max_popups_path_str, DEFAULT_MAX_ACTIVE_POPUPS); }
        }

        // Path for MaxHistoryItems (example string path)
        let max_history_path_str = "notifications.max_history_items";
        match self.settings_service.get_setting(&SettingPath::from_str_unsafe_for_testing(max_history_path_str)).await {
            Ok(JsonValue::Number(num)) => {
                if let Some(val) = num.as_u64() { *self.max_history_items_cache.write().await = val as usize; }
            }
            Ok(_) | Err(_) => { warn!("Could not read '{}' from global settings or invalid type, using default: {}", max_history_path_str, DEFAULT_MAX_HISTORY_ITEMS); }
        }
        
        debug!("Notification settings cache loaded: max_popups={}, max_history={}", 
               *self.max_active_popups_cache.read().await, *self.max_history_items_cache.read().await);
        Ok(())
    }

    async fn add_to_history(&self, notification: Notification) {
        let mut history_guard = self.history.write().await;
        let max_history = *self.max_history_items_cache.read().await;
        if max_history > 0 && history_guard.len() >= max_history { // Only pop if max_history > 0
            history_guard.pop_front();
        }
        if max_history > 0 || notification.transient == false { // Add if history is enabled or not transient
             history_guard.push_back(notification);
        }
    }

    fn apply_filters_and_sort(
        notifications: Vec<Notification>,
        filter: Option<&NotificationFilterCriteria>,
        sort_order: Option<NotificationSortOrder>,
    ) -> Vec<Notification> {
        let mut filtered = if let Some(f) = filter {
            notifications.into_iter().filter(|n| Self::matches_filter(n, f)).collect()
        } else { notifications };

        if let Some(order) = sort_order {
            match order {
                NotificationSortOrder::TimestampDescending => filtered.sort_by_key(|n| std::cmp::Reverse(n.timestamp)),
                NotificationSortOrder::TimestampAscending => filtered.sort_by_key(|n| n.timestamp),
                NotificationSortOrder::UrgencyDescending => { filtered.sort_by_key(|n| std::cmp::Reverse(n.urgency)); }
                NotificationSortOrder::UrgencyAscending => { filtered.sort_by_key(|n| n.urgency); }
                // Simplified app name/summary sort
                NotificationSortOrder::ApplicationNameAscending => filtered.sort_by(|a,b| a.application_name.cmp(&b.application_name)),
                NotificationSortOrder::ApplicationNameDescending => filtered.sort_by(|a,b| b.application_name.cmp(&a.application_name)),
                NotificationSortOrder::SummaryAscending => filtered.sort_by(|a,b| a.summary.cmp(&b.summary)),
                NotificationSortOrder::SummaryDescending => filtered.sort_by(|a,b| b.summary.cmp(&a.summary)),
            }
        }
        filtered
    }
    
    fn matches_filter(notification: &Notification, filter: &NotificationFilterCriteria) -> bool {
        match filter {
            NotificationFilterCriteria::Unread(unread) => notification.is_read != *unread,
            NotificationFilterCriteria::Application(app_id) => notification.application_name == app_id.as_str(),
            NotificationFilterCriteria::Urgency(urgency) => notification.urgency == *urgency,
            NotificationFilterCriteria::Category(cat) => notification.category.as_deref() == Some(cat.as_str()),
            NotificationFilterCriteria::HasActionWithKey(key) => notification.actions.iter().any(|a| a.key == *key),
            NotificationFilterCriteria::BodyContains(text) => notification.body.as_deref().unwrap_or("").contains(text),
            NotificationFilterCriteria::SummaryContains(text) => notification.summary.contains(text),
            NotificationFilterCriteria::IsTransient(transient) => notification.transient == *transient,
            NotificationFilterCriteria::TimeRange{start, end} => {
                let after_start = start.map_or(true, |s| notification.timestamp >= s);
                let before_end = end.map_or(true, |e| notification.timestamp <= e);
                after_start && before_end
            }
            NotificationFilterCriteria::And(criteria) => criteria.iter().all(|cf| Self::matches_filter(notification, cf)),
            NotificationFilterCriteria::Or(criteria) => criteria.iter().any(|cf| Self::matches_filter(notification, cf)),
            NotificationFilterCriteria::Not(criterion) => !Self::matches_filter(notification, criterion.as_ref()),
        }
    }

    fn publish_event(&self, event: NotificationEventEnum) {
        if self.event_publisher.send(event.clone()).is_err() { // Clone event for logging if send fails
            error!("Failed to send NotificationEventEnum: {:?}", event);
        }
    }
}

#[async_trait]
impl NotificationService for DefaultNotificationService {
    async fn post_notification(&self, notification_input: NotificationInput) -> Result<Uuid, NotificationError> {
        let original_id = notification_input.replaces_id.unwrap_or_else(Uuid::new_v4);
        let mut notification = Notification {
            id: original_id, application_name: notification_input.application_name.clone(),
            application_icon: notification_input.application_icon.clone(), summary: notification_input.summary.clone(),
            body: notification_input.body.clone(), actions: notification_input.actions.clone().unwrap_or_default(),
            urgency: notification_input.urgency.unwrap_or_default(), timestamp: Utc::now(),
            is_read: false, is_dismissed: false, transient: notification_input.transient.unwrap_or(false),
            category: notification_input.category.clone(), hints: notification_input.hints.clone().unwrap_or_default(),
            timeout_ms: notification_input.timeout_ms,
        };

        let rule_result = self.rules_engine.process_notification(&notification).await.map_err(NotificationError::RuleEngineError)?;
        
        match rule_result {
            RuleProcessingResult::Suppress { rule_id } => {
                debug!("Notification ID {} suppressed by rule ID {}", notification.id, rule_id);
                if !notification.transient { self.add_to_history(notification.clone()).await; }
                self.publish_event(NotificationEventEnum::NotificationSuppressedByRule { 
                    original_notification_id: notification.id, original_summary: notification.summary.clone(),
                    app_name: notification.application_name.clone(), rule_id,
                });
                return Ok(notification.id);
            }
            RuleProcessingResult::Modify(modified_notification) => { notification = modified_notification; }
            RuleProcessingResult::Allow => {}
        }

        let dnd_is_enabled = *self.dnd_enabled.read().await;
        let suppressed_by_dnd = dnd_is_enabled && notification.urgency != NotificationUrgency::Critical;

        if suppressed_by_dnd {
            debug!("Notification ID {} suppressed by DND mode", notification.id);
            if !notification.transient { self.add_to_history(notification.clone()).await; }
            self.publish_event(NotificationEventEnum::NotificationPosted { notification: notification.clone(), suppressed_by_dnd: true });
            return Ok(notification.id);
        }

        let mut active_guard = self.active_notifications.write().await;
        let max_popups = *self.max_active_popups_cache.read().await;
        if max_popups > 0 && active_guard.len() >= max_popups {
            if let Some(expired_notif) = active_guard.pop_front() {
                self.publish_event(NotificationEventEnum::NotificationPopupExpired { notification_id: expired_notif.id });
                if !expired_notif.transient { drop(active_guard); self.add_to_history(expired_notif).await; active_guard = self.active_notifications.write().await; }
            }
        }
        active_guard.push_back(notification.clone());
        drop(active_guard);

        if !notification.transient { self.add_to_history(notification.clone()).await; }
        
        self.publish_event(NotificationEventEnum::NotificationPosted { notification: notification.clone(), suppressed_by_dnd: false });
        info!("Notification ID {} posted. Summary: {}", notification.id, notification.summary);
        Ok(notification.id)
    }

    async fn get_notification(&self, id: Uuid) -> Result<Option<Notification>, NotificationError> {
        if let Some(n) = self.active_notifications.read().await.iter().find(|n| n.id == id) { return Ok(Some(n.clone())); }
        Ok(self.history.read().await.iter().find(|n| n.id == id).cloned())
    }

    async fn mark_as_read(&self, id: Uuid) -> Result<(), NotificationError> {
        if let Some(n) = self.active_notifications.write().await.iter_mut().find(|n| n.id == id) { if !n.is_read { n.mark_as_read(); self.publish_event(NotificationEventEnum::NotificationRead { notification_id: id }); } return Ok(()); }
        if let Some(n) = self.history.write().await.iter_mut().find(|n| n.id == id) { if !n.is_read { n.mark_as_read(); self.publish_event(NotificationEventEnum::NotificationRead { notification_id: id }); } return Ok(()); }
        Err(NotificationError::NotFound(id))
    }

    async fn dismiss_notification(&self, id: Uuid, reason: DismissReason) -> Result<(), NotificationError> {
        if let Some(idx) = self.active_notifications.read().await.iter().position(|n| n.id == id) {
            let mut notification = self.active_notifications.write().await.remove(idx).unwrap();
            notification.dismiss();
            if !notification.transient { self.add_to_history(notification).await; }
            self.publish_event(NotificationEventEnum::NotificationDismissed { notification_id: id, reason });
            return Ok(());
        }
        if let Some(n) = self.history.write().await.iter_mut().find(|n| n.id == id) {
            if !n.is_dismissed { n.dismiss(); self.publish_event(NotificationEventEnum::NotificationDismissed { notification_id: id, reason }); }
            return Ok(());
        }
        Err(NotificationError::NotFound(id))
    }

    async fn get_active_notifications(&self, filter: Option<&NotificationFilterCriteria>, sort: Option<NotificationSortOrder>) -> Result<Vec<Notification>, NotificationError> {
        Ok(Self::apply_filters_and_sort(self.active_notifications.read().await.iter().cloned().collect(), filter, sort))
    }

    async fn get_notification_history(&self, limit: Option<usize>, offset: Option<usize>, filter: Option<&NotificationFilterCriteria>, sort: Option<NotificationSortOrder>) -> Result<Vec<Notification>, NotificationError> {
        let processed = Self::apply_filters_and_sort(self.history.read().await.iter().cloned().collect(), filter, sort);
        let start = offset.unwrap_or(0); if start >= processed.len() { return Ok(Vec::new()); }
        let end = limit.map_or(processed.len(), |l| (start + l).min(processed.len()));
        Ok(processed.into_iter().skip(start).take(end - start).collect())
    }

    async fn clear_history(&self) -> Result<(), NotificationError> { self.history.write().await.clear(); self.publish_event(NotificationEventEnum::NotificationHistoryCleared); Ok(()) }
    async fn clear_all_for_app(&self, app_id: &ApplicationId, reason: DismissReason) -> Result<usize, NotificationError> {
        let mut dismissed_count = 0;
        let mut active_guard = self.active_notifications.write().await;
        let mut i = 0;
        while i < active_guard.len() {
            if active_guard[i].application_name == app_id.as_str() {
                let mut notif = active_guard.remove(i).unwrap(); notif.dismiss(); let notif_id = notif.id;
                if !notif.transient { drop(active_guard); self.add_to_history(notif).await; active_guard = self.active_notifications.write().await; } // Re-acquire
                self.publish_event(NotificationEventEnum::NotificationDismissed { notification_id: notif_id, reason }); dismissed_count += 1;
            } else { i += 1; }
        }
        drop(active_guard);
        let mut history_guard = self.history.write().await;
        for notif in history_guard.iter_mut() {
            if notif.application_name == app_id.as_str() && !notif.is_dismissed { notif.dismiss(); dismissed_count += 1; /* No event for already historical items */ }
        }
        Ok(dismissed_count)
    }

    async fn set_do_not_disturb(&self, enabled: bool) -> Result<(), NotificationError> {
        let mut dnd = self.dnd_enabled.write().await; if *dnd != enabled { *dnd = enabled; self.publish_event(NotificationEventEnum::DoNotDisturbModeChanged { dnd_enabled: enabled }); } Ok(())
    }
    async fn is_do_not_disturb_enabled(&self) -> Result<bool, NotificationError> { Ok(*self.dnd_enabled.read().await) }

    async fn invoke_action(&self, id: Uuid, key: &str) -> Result<(), NotificationError> {
        let notif = self.get_notification(id).await?.ok_or(NotificationError::NotFound(id))?;
        if notif.actions.iter().any(|a| a.key == key) { self.publish_event(NotificationEventEnum::NotificationActionInvoked { notification_id: id, action_key: key.to_string() }); Ok(()) }
        else { Err(NotificationError::ActionNotFound { notification_id: id, action_key: key.to_string() }) }
    }

    async fn get_stats(&self) -> Result<NotificationStats, NotificationError> {
        let active = self.active_notifications.read().await;
        Ok(NotificationStats { num_active: active.len(), num_history: self.history.read().await.len(), num_unread_active: active.iter().filter(|n| !n.is_read).count() })
    }
    fn subscribe_to_notification_events(&self) -> broadcast::Receiver<NotificationEventEnum> { self.event_publisher.subscribe() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notifications_rules::{MockNotificationRulesEngine, RuleProcessingResult};
    use crate::global_settings::{MockGlobalSettingsService, SettingPathParseError}; // Assuming this mock exists
    use tokio::sync::broadcast::error::RecvError;
    use crate::user_centric_services::events::NotificationEventEnum as Event;

    // Helper for SettingPath in tests, as it's not fully defined for notifications yet.
    // This mirrors the unsafe helper in GlobalSettingsService tests if that was introduced.
    // If SettingPath has a proper variant for notifications, this won't be needed.
    impl SettingPath {
        fn from_str_unsafe_for_testing(s: &str) -> Self {
            // This is a simplified parser for test purposes only.
            // It does not represent the full complexity of SettingPath.
            // It assumes a simple dot-separated path.
            // A real SettingPath would have proper variants.
            // Example: "notifications.max_active_popups"
            // For now, we'll just return a placeholder that might not match real paths.
            // This is a known limitation due to SettingPath not having notification-specific variants.
            warn!("Using unsafe SettingPath::from_str_unsafe_for_testing. This is for testing only.");
            // In a real scenario, SettingPath would have a variant like:
            // SettingPath::NotificationsCore(NotificationsCorePath::MaxActivePopups)
            // For now, we'll use a root path as a placeholder.
            SettingPath::Root 
        }
    }


    fn create_test_notification_input(summary: &str) -> NotificationInput {
        NotificationInput { application_name: "TestApp".to_string(), summary: summary.to_string(), ..Default::default() }
    }
    async fn drain_events(rx: &mut broadcast::Receiver<NotificationEventEnum>) {
        loop { match tokio::time::timeout(std::time::Duration::from_millis(1), rx.recv()).await { Ok(Ok(_)) => continue, _ => break } }
    }

    #[tokio::test]
    async fn test_post_notification_simple_flow() {
        let rules_engine = Arc::new(MockNotificationRulesEngine::new());
        let settings_service = Arc::new(MockGlobalSettingsService::new());
        rules_engine.expect_process_notification().times(1).returning(|_| Ok(RuleProcessingResult::Allow));
        let service = DefaultNotificationService::new(rules_engine, settings_service, 5).await.unwrap();
        let mut rx = service.subscribe_to_notification_events();

        let result = service.post_notification(create_test_notification_input("Test Notif 1")).await;
        assert!(result.is_ok()); let notif_id = result.unwrap();
        assert_eq!(service.get_active_notifications(None, None).await.unwrap().len(), 1);
        match rx.try_recv() { Ok(Event::NotificationPosted { notification, .. }) => assert_eq!(notification.id, notif_id), e => panic!("{:?}", e) }
    }
    
    #[tokio::test]
    async fn test_post_notification_suppressed_by_rule() {
        let rules_engine = Arc::new(MockNotificationRulesEngine::new());
        let settings_service = Arc::new(MockGlobalSettingsService::new());
        rules_engine.expect_process_notification().times(1).returning(|_| Ok(RuleProcessingResult::Suppress { rule_id: "rule1".to_string() }));
        let service = DefaultNotificationService::new(rules_engine, settings_service, 5).await.unwrap();
        let mut rx = service.subscribe_to_notification_events();

        let result = service.post_notification(create_test_notification_input("Suppressed")).await;
        assert!(result.is_ok());
        assert!(service.get_active_notifications(None, None).await.unwrap().is_empty());
        assert_eq!(service.get_notification_history(None, None, None, None).await.unwrap().len(), 1);
        match rx.try_recv() { Ok(Event::NotificationSuppressedByRule { rule_id, .. }) => assert_eq!(rule_id, "rule1"), e => panic!("{:?}", e) }
    }

    #[tokio::test]
    async fn test_post_notification_dnd_suppression() {
        let rules_engine = Arc::new(MockNotificationRulesEngine::new());
        let settings_service = Arc::new(MockGlobalSettingsService::new());
        rules_engine.expect_process_notification().returning(|_| Ok(RuleProcessingResult::Allow));
        let service = DefaultNotificationService::new(rules_engine, settings_service, 5).await.unwrap();
        service.set_do_not_disturb(true).await.unwrap();
        let mut rx = service.subscribe_to_notification_events(); drain_events(&mut rx).await;

        service.post_notification(create_test_notification_input("DND Suppressed")).await.unwrap();
        assert!(service.get_active_notifications(None, None).await.unwrap().is_empty());
        match rx.try_recv() { Ok(Event::NotificationPosted { suppressed_by_dnd, .. }) => assert!(suppressed_by_dnd), e => panic!("{:?}", e) }
    }
    
    #[tokio::test]
    async fn test_dismiss_notification() {
        let rules_engine = Arc::new(MockNotificationRulesEngine::new());
        let settings_service = Arc::new(MockGlobalSettingsService::new());
        rules_engine.expect_process_notification().returning(|_| Ok(RuleProcessingResult::Allow));
        let service = DefaultNotificationService::new(rules_engine, settings_service, 5).await.unwrap();
        let mut rx = service.subscribe_to_notification_events();

        let notif_id = service.post_notification(create_test_notification_input("To Dismiss")).await.unwrap();
        drain_events(&mut rx).await;

        service.dismiss_notification(notif_id, DismissReason::ByUser).await.unwrap();
        assert!(service.get_active_notifications(None, None).await.unwrap().is_empty());
        match rx.try_recv() { Ok(Event::NotificationDismissed { notification_id, reason, .. }) => { assert_eq!(notification_id, notif_id); assert_eq!(reason, DismissReason::ByUser); }, e => panic!("{:?}", e) }
    }
}
