// novade-domain/tests/notification_service_policies_grouping_tests.rs

use chrono::Utc;
use novade_domain::notification_service::policies::grouping::{
    DefaultGroupingPolicy, NotificationGroupingPolicy,
};
use novade_domain::user_centric_services::notifications_core::types::{
    Notification, NotificationUrgency,
};
use std::collections::HashMap;
use uuid::Uuid;

// Helper function to create a dummy notification
fn create_test_notification(
    id: Uuid,
    app_name: &str,
    summary: &str,
    urgency: NotificationUrgency,
) -> Notification {
    Notification {
        id,
        application_name: app_name.to_string(),
        application_icon: None,
        summary: summary.to_string(),
        body: None,
        actions: Vec::new(),
        urgency,
        timestamp: Utc::now(),
        is_read: false,
        is_dismissed: false,
        transient: false,
        category: None,
        hints: HashMap::new(),
        timeout_ms: None,
    }
}

#[test]
fn default_grouping_policy_empty_list() {
    let policy = DefaultGroupingPolicy::new();
    let notifications: Vec<Notification> = Vec::new();
    let result = policy.group_notifications(&notifications);

    assert!(result.is_ok());
    let grouped = result.unwrap();
    assert!(
        grouped.is_empty(),
        "Grouped map should be empty for empty input."
    );
}

#[test]
fn default_grouping_policy_single_app() {
    let policy = DefaultGroupingPolicy::new();
    let app_name = "TestApp";
    let notif1 = create_test_notification(
        Uuid::new_v4(),
        app_name,
        "Summary 1",
        NotificationUrgency::Normal,
    );
    let notif2 = create_test_notification(
        Uuid::new_v4(),
        app_name,
        "Summary 2",
        NotificationUrgency::Low,
    );
    let notifications = vec![notif1.clone(), notif2.clone()];

    let result = policy.group_notifications(&notifications);
    assert!(result.is_ok());
    let grouped = result.unwrap();

    assert_eq!(
        grouped.len(),
        1,
        "Should be one group for a single app name."
    );
    assert!(
        grouped.contains_key(app_name),
        "Group key should be the app name."
    );

    let app_group = grouped.get(app_name).unwrap();
    assert_eq!(
        app_group.len(),
        2,
        "Group should contain two notifications."
    );
    assert!(app_group.contains(&notif1));
    assert!(app_group.contains(&notif2));
}

#[test]
fn default_grouping_policy_multiple_apps() {
    let policy = DefaultGroupingPolicy::new();
    let app1_name = "AppOne";
    let app2_name = "AppTwo";

    let notif_app1_1 = create_test_notification(
        Uuid::new_v4(),
        app1_name,
        "App1 Summary1",
        NotificationUrgency::Normal,
    );
    let notif_app1_2 = create_test_notification(
        Uuid::new_v4(),
        app1_name,
        "App1 Summary2",
        NotificationUrgency::High,
    );
    let notif_app2_1 = create_test_notification(
        Uuid::new_v4(),
        app2_name,
        "App2 Summary1",
        NotificationUrgency::Normal,
    );

    let notifications = vec![
        notif_app1_1.clone(),
        notif_app2_1.clone(),
        notif_app1_2.clone(),
    ];

    let result = policy.group_notifications(&notifications);
    assert!(result.is_ok());
    let grouped = result.unwrap();

    assert_eq!(
        grouped.len(),
        2,
        "Should be two groups for two different app names."
    );

    // Check AppOne group
    assert!(
        grouped.contains_key(app1_name),
        "Group key for AppOne missing."
    );
    let app1_group = grouped.get(app1_name).unwrap();
    assert_eq!(
        app1_group.len(),
        2,
        "AppOne group should contain two notifications."
    );
    assert!(app1_group.contains(&notif_app1_1));
    assert!(app1_group.contains(&notif_app1_2));

    // Check AppTwo group
    assert!(
        grouped.contains_key(app2_name),
        "Group key for AppTwo missing."
    );
    let app2_group = grouped.get(app2_name).unwrap();
    assert_eq!(
        app2_group.len(),
        1,
        "AppTwo group should contain one notification."
    );
    assert!(app2_group.contains(&notif_app2_1));
}

#[test]
fn default_grouping_policy_notifications_are_cloned() {
    let policy = DefaultGroupingPolicy::new();
    let app_name = "CloneTestApp";
    let original_notif = create_test_notification(
        Uuid::new_v4(),
        app_name,
        "Original",
        NotificationUrgency::Normal,
    );
    let notifications = vec![original_notif.clone()];

    let result = policy.group_notifications(&notifications);
    assert!(result.is_ok());
    let grouped = result.unwrap();

    let app_group = grouped.get(app_name).unwrap();
    let grouped_notif = &app_group[0];

    // Check they are equal in content
    assert_eq!(
        &original_notif, grouped_notif,
        "Cloned notification should be equal to original."
    );

    // To truly check if it's a clone, you'd compare pointers or modify one and see
    // if the other changes. However, direct pointer comparison is tricky here.
    // The fact that `Notification` implements `Clone` and we call `.clone()`
    // in the policy is the guarantee.
    // A simple check: their IDs should be the same.
    assert_eq!(original_notif.id, grouped_notif.id);

    // If we could modify, e.g., `is_read` (and it wasn't part of the equality check)
    // let mut original_mut = original_notif;
    // original_mut.is_read = true; // if Notification had a public mutable field or method
    // assert_ne!(original_mut.is_read, grouped_notif.is_read); // This would fail if not cloned.
    // For now, relying on the `.clone()` call is sufficient for this test's intent.
}

#[test]
fn default_grouping_policy_app_name_case_sensitivity() {
    let policy = DefaultGroupingPolicy::new();
    let app_name_upper = "MyApp";
    let app_name_lower = "myapp";

    let notif1 = create_test_notification(
        Uuid::new_v4(),
        app_name_upper,
        "Summary Upper",
        NotificationUrgency::Normal,
    );
    let notif2 = create_test_notification(
        Uuid::new_v4(),
        app_name_lower,
        "Summary Lower",
        NotificationUrgency::Normal,
    );

    // If grouping is case-sensitive (which it is by default for String keys in HashMap)
    // these should end up in different groups.
    let notifications = vec![notif1.clone(), notif2.clone()];
    let result = policy.group_notifications(&notifications);
    assert!(result.is_ok());
    let grouped = result.unwrap();

    assert_eq!(
        grouped.len(),
        2,
        "Should be two groups due to case sensitivity in app name."
    );
    assert!(grouped.contains_key(app_name_upper));
    assert!(grouped.contains_key(app_name_lower));
    assert_eq!(grouped.get(app_name_upper).unwrap().len(), 1);
    assert_eq!(grouped.get(app_name_lower).unwrap().len(), 1);
}
