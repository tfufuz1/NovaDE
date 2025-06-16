// novade-domain/src/system_health_service/service_tests.rs
#[cfg(test)]
mod tests {
    use super::super::service::*; // Access items from service.rs
    use novade_core::config::CoreConfig;
    use novade_core::types::system_health::*;
    use crate::error::SystemHealthError;
    use novade_system::system_health_collectors::{
        MockCpuMetricsCollector, MockMemoryMetricsCollector, MockDiskMetricsCollector,
        MockNetworkMetricsCollector, MockTemperatureMetricsCollector, MockLogHarvester,
        MockDiagnosticRunner, // Assuming these mocks exist or will be created
    };
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use std::collections::{HashMap, VecDeque};
    use chrono::{Utc, Duration as ChronoDuration};
    use uuid::Uuid;

    // Helper to create a default CoreConfig for tests, can be customized
    fn test_core_config() -> Arc<CoreConfig> {
        Arc::new(CoreConfig::default())
    }

    // Helper to create mock collectors (basic versions)
    // In a real scenario, these would be more sophisticated or come from a testing crate.
    #[derive(Default)]
    struct TestCpuCollector;
    #[async_trait::async_trait]
    impl novade_system::system_health_collectors::CpuMetricsCollector for TestCpuCollector {
        async fn collect_cpu_metrics(&self) -> Result<CpuMetrics, novade_system::error::SystemError> {
            Ok(CpuMetrics { total_usage_percent: 0.0, per_core_usage_percent: vec![] })
        }
    }

    #[derive(Default)]
    struct TestMemoryCollector;
    #[async_trait::async_trait]
    impl novade_system::system_health_collectors::MemoryMetricsCollector for TestMemoryCollector {
        async fn collect_memory_metrics(&self) -> Result<MemoryMetrics, novade_system::error::SystemError> {
            Ok(MemoryMetrics { total_bytes: 1000, used_bytes: 500, free_bytes: 0, available_bytes: 500, swap_total_bytes: 0, swap_used_bytes: 0 })
        }
    }

    #[derive(Default)]
    struct TestDiskCollector;
    #[async_trait::async_trait]
    impl novade_system::system_health_collectors::DiskMetricsCollector for TestDiskCollector {
        async fn collect_disk_activity_metrics(&self) -> Result<Vec<DiskActivityMetrics>, novade_system::error::SystemError> { Ok(vec![]) }
        async fn collect_disk_space_metrics(&self) -> Result<Vec<DiskSpaceMetrics>, novade_system::error::SystemError> { Ok(vec![]) }
    }

    // Create other dummy collectors as needed...
    #[derive(Default)] struct TestNetworkCollector;
    #[async_trait::async_trait]
    impl novade_system::system_health_collectors::NetworkMetricsCollector for TestNetworkCollector {
        async fn collect_network_activity_metrics(&self) -> Result<Vec<NetworkActivityMetrics>, novade_system::error::SystemError> { Ok(vec![]) }
    }
    #[derive(Default)] struct TestTempCollector;
    #[async_trait::async_trait]
    impl novade_system::system_health_collectors::TemperatureMetricsCollector for TestTempCollector {
        async fn collect_temperature_metrics(&self) -> Result<Vec<TemperatureMetric>, novade_system::error::SystemError> { Ok(vec![]) }
    }
    #[derive(Default)] struct TestLogHarvester;
    #[async_trait::async_trait]
    impl novade_system::system_health_collectors::LogHarvester for TestLogHarvester {
        async fn stream_logs(&self, _filter: Option<LogFilter>) -> Result<Box<dyn futures_core::Stream<Item = Result<LogEntry, novade_system::error::SystemError>> + Send + Unpin>, novade_system::error::SystemError> { todo!() }
        async fn query_logs(&self, _filter: Option<LogFilter>, _time_range: Option<TimeRange>, _limit: Option<usize>) -> Result<Vec<LogEntry>, novade_system::error::SystemError> { Ok(vec![]) }
    }
    #[derive(Default)] struct TestDiagnosticRunner;
    #[async_trait::async_trait]
    impl novade_system::system_health_collectors::DiagnosticRunner for TestDiagnosticRunner {
        fn list_available_tests(&self) -> Result<Vec<DiagnosticTestInfo>, novade_system::error::SystemError> { Ok(vec![]) }
        async fn run_test(&self, _test_id: &DiagnosticTestId) -> Result<DiagnosticTestResult, novade_system::error::SystemError> { todo!() }
    }


    fn create_test_service(config: Arc<CoreConfig>) -> DefaultSystemHealthService {
        DefaultSystemHealthService::new(
            config,
            Arc::new(TestCpuCollector::default()),
            Arc::new(TestMemoryCollector::default()),
            Arc::new(TestDiskCollector::default()),
            Arc::new(TestNetworkCollector::default()),
            Arc::new(TestTempCollector::default()),
            Arc::new(TestLogHarvester::default()),
            Arc::new(TestDiagnosticRunner::default()),
        )
    }

    #[tokio::test]
    async fn test_acknowledge_alert_found() {
        let config = test_core_config();
        let service = create_test_service(config.clone());
        let alert_id_str = Uuid::new_v4().to_string();
        let alert_key = "test_alert_condition";

        let mut initial_alert = Alert {
            id: AlertId(alert_id_str.clone()),
            name: "Test CPU Alert".to_string(),
            message: "CPU is high".to_string(),
            severity: AlertSeverity::High,
            timestamp: Utc::now(),
            last_triggered_timestamp: Utc::now(),
            source_metric_or_log: "cpu".to_string(),
            acknowledged: false,
            last_triggered_count: 1,
            resolution_steps: None,
        };

        // Manually insert an alert
        {
            let mut active_alerts = service.active_alerts.lock().await;
            active_alerts.insert(alert_key.to_string(), initial_alert.clone());
        }

        let result = service.acknowledge_alert(alert_id_str.clone()).await;
        assert!(result.is_ok());

        let active_alerts = service.active_alerts.lock().await;
        let acked_alert = active_alerts.get(alert_key).unwrap();
        assert!(acked_alert.acknowledged);
        assert_eq!(acked_alert.id.0, alert_id_str);
    }

    #[tokio::test]
    async fn test_acknowledge_alert_not_found() {
        let config = test_core_config();
        let service = create_test_service(config);
        let non_existent_id = Uuid::new_v4().to_string();

        let result = service.acknowledge_alert(non_existent_id.clone()).await;
        assert!(matches!(result, Err(SystemHealthError::AlertNotFound { alert_id }) if alert_id == non_existent_id));
    }

    // Helper for CPU duration tests
    async fn setup_cpu_alert_test(
        cpu_threshold: f32,
        duration_secs: u64,
        history_size: usize,
    ) -> (Arc<CoreConfig>, Arc<Mutex<ActiveAlertsMap>>, Arc<Mutex<VecDeque<TimestampedCpuMetrics>>>) {
        let mut config = CoreConfig::default();
        config.system_health.alert_thresholds.high_cpu_usage_percent = Some(CpuAlertConfig {
            threshold_percent: cpu_threshold,
            duration_seconds: duration_secs as u32, // Note: CpuAlertConfig uses u32
        });
        config.system_health.cpu_alert_duration_secs = Some(duration_secs);
        config.system_health.cpu_alert_history_size = history_size;

        let arc_config = Arc::new(config);
        let active_alerts_map_arc = Arc::new(Mutex::new(HashMap::new()));
        let cpu_usage_history_arc = Arc::new(Mutex::new(VecDeque::with_capacity(history_size)));

        (arc_config, active_alerts_map_arc, cpu_usage_history_arc)
    }

    fn mock_cpu_metrics(usage: f32) -> CpuMetrics {
        CpuMetrics { total_usage_percent: usage, per_core_usage_percent: vec![usage] }
    }

    async fn run_eval_cycle_for_test(
        config: Arc<CoreConfig>,
        alerts: Arc<Mutex<ActiveAlertsMap>>,
        history: Arc<Mutex<VecDeque<TimestampedCpuMetrics>>>
    ) -> bool {
         // For these tests, we don't need actual collectors, as run_alert_evaluation_cycle
         // takes them as args, but the current duration logic primarily uses the history.
         // The other alert types (memory, disk) in run_alert_evaluation_cycle DO call collectors.
         // So we pass dummy ones here.
        DefaultSystemHealthService::run_alert_evaluation_cycle(
            config,
            alerts,
            history,
            Arc::new(TestCpuCollector::default()),
            Arc::new(TestMemoryCollector::default()),
            Arc::new(TestDiskCollector::default()),
        ).await.unwrap_or(false) // unwrap result for test simplicity
    }


    #[tokio::test]
    async fn test_cpu_duration_alert_trigger() {
        let threshold = 70.0;
        let duration_secs = 5; // Test with a short duration
        let history_size = 10;
        let (config, alerts, history) = setup_cpu_alert_test(threshold, duration_secs, history_size).await;

        // Populate history: 6 samples over threshold, spanning > duration_secs
        let mut current_time = Utc::now();
        {
            let mut hist = history.lock().await;
            for i in 0..6 {
                hist.push_back(TimestampedCpuMetrics {
                    metrics: mock_cpu_metrics(threshold + 5.0), // Above threshold
                    timestamp: current_time - ChronoDuration::seconds(5 - i as i64), // Samples 1 sec apart, newest at end
                });
            }
        }

        run_eval_cycle_for_test(config.clone(), alerts.clone(), history.clone()).await;

        let active_alerts = alerts.lock().await;
        assert_eq!(active_alerts.len(), 1, "Expected 1 CPU duration alert");
        let alert = active_alerts.values().next().unwrap();
        assert_eq!(alert.name, "Sustained High CPU Usage");
        assert!(!alert.acknowledged);
    }

    #[tokio::test]
    async fn test_cpu_duration_alert_no_trigger_spiky() {
        let threshold = 70.0;
        let duration_secs = 5;
        let history_size = 10;
        let (config, alerts, history) = setup_cpu_alert_test(threshold, duration_secs, history_size).await;

        {
            let mut hist = history.lock().await;
            let now = Utc::now();
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold + 10.0), timestamp: now - ChronoDuration::seconds(6) });
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold - 10.0), timestamp: now - ChronoDuration::seconds(5) }); // Dip
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold + 10.0), timestamp: now - ChronoDuration::seconds(4) });
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold - 10.0), timestamp: now - ChronoDuration::seconds(3) }); // Dip
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold + 10.0), timestamp: now - ChronoDuration::seconds(2) });
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold + 10.0), timestamp: now - ChronoDuration::seconds(1) });
        }

        run_eval_cycle_for_test(config.clone(), alerts.clone(), history.clone()).await;
        assert!(alerts.lock().await.is_empty(), "Expected no alerts for spiky CPU usage");
    }

    #[tokio::test]
    async fn test_cpu_duration_alert_no_trigger_below_threshold() {
        let threshold = 70.0;
        let duration_secs = 5;
        let history_size = 10;
        let (config, alerts, history) = setup_cpu_alert_test(threshold, duration_secs, history_size).await;

        {
            let mut hist = history.lock().await;
            let now = Utc::now();
            for i in 0..6 {
                 hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold - 10.0), timestamp: now - ChronoDuration::seconds(5-i)});
            }
        }
        run_eval_cycle_for_test(config.clone(), alerts.clone(), history.clone()).await;
        assert!(alerts.lock().await.is_empty(), "Expected no alerts for CPU usage below threshold");
    }

    #[tokio::test]
    async fn test_cpu_duration_alert_retrigger_after_ack() {
        let threshold = 70.0;
        let duration_secs = 3; // shorter for test
        let history_size = 5;
        let (config, alerts, history) = setup_cpu_alert_test(threshold, duration_secs, history_size).await;
        let alert_key = "high_cpu_usage_duration";

        // 1. Trigger alert
        {
            let mut hist = history.lock().await;
            let now = Utc::now();
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold + 5.0), timestamp: now - ChronoDuration::seconds(3) });
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold + 5.0), timestamp: now - ChronoDuration::seconds(2) });
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold + 5.0), timestamp: now - ChronoDuration::seconds(1) });
        }
        run_eval_cycle_for_test(config.clone(), alerts.clone(), history.clone()).await;
        assert_eq!(alerts.lock().await.len(), 1, "Alert should be triggered");

        // 2. Acknowledge alert
        {
            let mut active_alerts = alerts.lock().await;
            let alert_to_ack = active_alerts.get_mut(alert_key).expect("Alert should exist");
            alert_to_ack.acknowledged = true;
            assert!(alert_to_ack.acknowledged);
        }

        // 3. Keep CPU high, run evaluation again
        {
            let mut hist = history.lock().await;
            let now = Utc::now(); // new 'now'
             // Ensure history still shows sustained high usage up to current 'now'
            hist.clear(); // Clear old history to simulate new readings after ack
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold + 5.0), timestamp: now - ChronoDuration::seconds(3) });
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold + 5.0), timestamp: now - ChronoDuration::seconds(2) });
            hist.push_back(TimestampedCpuMetrics { metrics: mock_cpu_metrics(threshold + 5.0), timestamp: now - ChronoDuration::seconds(1) });
        }
        run_eval_cycle_for_test(config.clone(), alerts.clone(), history.clone()).await;

        let active_alerts = alerts.lock().await;
        assert_eq!(active_alerts.len(), 1, "Alert should still exist or be re-triggered");
        let alert = active_alerts.get(alert_key).unwrap();
        assert!(!alert.acknowledged, "Alert should be unacknowledged again");
        assert!(alert.last_triggered_count > 0, "Trigger count should increment or alert be new");
    }
}
