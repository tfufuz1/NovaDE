// novade-ui/src/system_health_dashboard/view_model.rs
use std::sync::Arc;
use std::cell::RefCell;
use std::collections::HashMap;

use glib::{prelude::*, subclass::prelude::*, clone, Signal, ToVariant, Variant, MainContext, WeakRef};
use novade_domain::system_health_service::service::SystemHealthServiceTrait;
use novade_core::types::system_health::{
    CpuMetrics, MemoryMetrics, DiskActivityMetrics, DiskSpaceMetrics, NetworkActivityMetrics, TemperatureMetric,
    DiagnosticTestId, DiagnosticTestInfo, DiagnosticTestResult, DiagnosticStatus,
};
use log::{error, debug, info, warn, trace}; // Added trace
use tokio::sync::broadcast;

// Adwaita and UiFeedbackService imports
use libadwaita as adw;
use crate::ui_feedback;


// Helper to convert Vec<T> to Variant where T is Boxed.
fn vec_to_variant<T: glib::BoxedType + Clone + Send + Sync + 'static>(data: &Vec<T>) -> Variant {
    data.to_variant()
}

mod imp {
    use super::*;
    use glib::Properties;

    #[derive(Properties)]
    #[properties(wrapper_type = super::SystemHealthViewModel)]
    pub struct ViewModelPriv {
        #[property(get, set)]
        pub system_health_service: RefCell<Option<Arc<dyn SystemHealthServiceTrait>>>,
        // Metrics
        pub current_cpu_metrics: RefCell<Option<CpuMetrics>>,
        pub latest_memory_metrics: RefCell<Option<MemoryMetrics>>,
        pub current_disk_activity: RefCell<Option<Vec<DiskActivityMetrics>>>,
        pub current_disk_space: RefCell<Option<Vec<DiskSpaceMetrics>>>,
        pub current_network_activity: RefCell<Option<Vec<NetworkActivityMetrics>>>,
        pub current_temperatures: RefCell<Option<Vec<TemperatureMetric>>>,
        // Diagnostics
        pub available_diagnostic_tests: RefCell<Option<Vec<DiagnosticTestInfo>>>,
        pub diagnostic_test_results: RefCell<HashMap<DiagnosticTestId, DiagnosticTestResult>>,
        pub running_test_id: RefCell<Option<DiagnosticTestId>>,
        // Adwaita App Window reference
        #[property(get, set)] // Expose as property if needed for direct binding, though not typical for WeakRef
        pub app_window: RefCell<Option<WeakRef<adw::ApplicationWindow>>>,
    }

    // Default implementation for ViewModelPriv
    impl Default for ViewModelPriv {
        fn default() -> Self {
            Self {
                system_health_service: RefCell::new(None),
                current_cpu_metrics: RefCell::new(None),
                latest_memory_metrics: RefCell::new(None),
                current_disk_activity: RefCell::new(None),
                current_disk_space: RefCell::new(None),
                current_network_activity: RefCell::new(None),
                current_temperatures: RefCell::new(None),
                available_diagnostic_tests: RefCell::new(None),
                diagnostic_test_results: RefCell::new(HashMap::new()),
                running_test_id: RefCell::new(None),
                app_window: RefCell::new(None), // Initialize app_window
            }
        }
    }


    #[glib::object_subclass]
    impl ObjectSubclass for ViewModelPriv {
        const NAME: &'static str = "NovaSystemHealthViewModel";
        type Type = super::SystemHealthViewModel;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ViewModelPriv {
        fn signals() -> &'static [Signal] {
            static SIGNALS: std::sync::OnceLock<Vec<Signal>> = std::sync::OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("cpu-metrics-updated").param_types([CpuMetrics::static_type()]).build(),
                    Signal::builder("memory-metrics-updated").param_types([MemoryMetrics::static_type()]).build(),
                    Signal::builder("disk-activity-updated").param_types([Variant::static_type()]).build(),
                    Signal::builder("disk-space-updated").param_types([Variant::static_type()]).build(),
                    Signal::builder("network-activity-updated").param_types([Variant::static_type()]).build(),
                    Signal::builder("temperature-metrics-updated").param_types([Variant::static_type()]).build(),
                    Signal::builder("available-tests-changed").param_types([Variant::static_type()]).build(),
                    Signal::builder("test-result-updated").param_types([DiagnosticTestResult::static_type()]).build(),
                    Signal::builder("test-status-changed")
                        .param_types([DiagnosticTestId::static_type(), DiagnosticStatus::static_type()])
                        .build(),
                ]
            })
        }
        fn constructed(&self) { self.parent_constructed(); }
        fn properties() -> &'static [glib::ParamSpec] { Self::derived_properties() }
        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) { self.derived_set_property(id, value, pspec); }
        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value { self.derived_property(id, pspec) }
    }
} // end mod imp

glib::wrapper! {
    pub struct SystemHealthViewModel(ObjectSubclass<imp::ViewModelPriv>);
}

impl SystemHealthViewModel {
    // Updated constructor signature
    pub fn new(service: Arc<dyn SystemHealthServiceTrait>, main_app_window: WeakRef<adw::ApplicationWindow>) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp().system_health_service.replace(Some(service.clone()));
        obj.imp().app_window.replace(Some(main_app_window)); // Store the window weak ref

        obj.imp().start_cpu_metrics_subscription(service.clone());
        obj.imp().start_memory_metrics_subscription(service.clone());
        obj.imp().start_disk_activity_subscription(service.clone());
        obj.imp().start_disk_space_subscription(service.clone());
        obj.imp().start_network_activity_subscription(service.clone());
        obj.imp().start_temperature_metrics_subscription(service);

        info!("UI: SystemHealthViewModel (GObject) created, window ref stored, and subscriptions started.");
        obj
    }

    pub fn load_available_tests(&self) {
        debug!("VM: Requesting available diagnostic tests.");
        let service_opt = self.imp().system_health_service.borrow();
        if let Some(service) = service_opt.as_ref() {
            let service_clone = service.clone();
            let this = self.obj();
            let main_context = MainContext::default();

            main_context.spawn_local(clone!(@weak this => async move {
                match service_clone.list_available_diagnostic_tests().await {
                    Ok(tests_vec) => {
                        this.imp().available_diagnostic_tests.replace(Some(tests_vec.clone()));
                        this.emit_by_name::<()>("available-tests-changed", &[&vec_to_variant(&tests_vec)]);
                    }
                    Err(e) => {
                        error!("VM: Error loading available diagnostic tests: {:?}", e);
                        let user_message = format!("Failed to load diagnostic tests: {}", e);
                        if let Some(window_weak) = this.imp().app_window.borrow().as_ref() {
                            if let Some(window_strong) = window_weak.upgrade() {
                                ui_feedback::show_error_toast(&window_strong, &user_message);
                            }
                        }
                        this.imp().available_diagnostic_tests.replace(Some(vec![]));
                        this.emit_by_name::<()>("available-tests-changed", &[&vec_to_variant(&Vec::<DiagnosticTestInfo>::new())]);
                    }
                }
            }));
        } else {
            warn!("VM: SystemHealthService not available for load_available_tests.");
        }
    }

    pub fn run_diagnostic_test(&self, test_id: &DiagnosticTestId) {
        debug!("VM: Request to run diagnostic test: {:?}", test_id);
        let service_opt = self.imp().system_health_service.borrow();
        if let Some(service) = service_opt.as_ref() {
            self.imp().running_test_id.replace(Some(test_id.clone()));
            self.emit_by_name::<()>("test-status-changed", &[test_id, &DiagnosticStatus::Running]);

            let service_clone = service.clone();
            let test_id_clone = test_id.clone();
            let this = self.obj();
            let main_context = MainContext::default();

            main_context.spawn_local(clone!(@weak this => async move {
                trace!("VM: Future for test {:?} started.", test_id_clone);
                match service_clone.run_diagnostic_test(test_id_clone.clone(), None).await {
                    Ok(res_data) => {
                        trace!("VM: Test {:?} completed successfully in future.", test_id_clone);
                        this.imp().diagnostic_test_results.borrow_mut().insert(test_id_clone.clone(), res_data.clone());
                        this.imp().running_test_id.replace(None);
                        this.emit_by_name::<()>("test-result-updated", &[&res_data]);
                        this.emit_by_name::<()>("test-status-changed", &[&test_id_clone, &res_data.status]);
                    }
                    Err(e) => {
                        // Log the detailed technical error
                        error!("VM: Error running diagnostic test '{:?}': {}", test_id_clone, e);

                        // Show user-friendly toast
                        let user_message = format!("Failed to run test '{}': {}", test_id_clone.0, e);
                        if let Some(window_weak) = this.imp().app_window.borrow().as_ref() {
                            if let Some(window_strong) = window_weak.upgrade() {
                                ui_feedback::show_error_toast(&window_strong, &user_message);
                            } else {
                                error!("VM (Toast): App window weak ref failed to upgrade for test error.");
                            }
                        } else {
                            error!("VM (Toast): App window weak ref not set in ViewModel for test error.");
                        }

                        // Create and emit error result for UI update
                        this.imp().running_test_id.replace(None);
                        let error_result = DiagnosticTestResult {
                            id: test_id_clone.clone(),
                            status: DiagnosticStatus::Error,
                            summary: format!("Execution Error: {}", e),
                            details: None, start_time: None, end_time: None,
                        };
                        this.imp().diagnostic_test_results.borrow_mut().insert(test_id_clone.clone(), error_result.clone());
                        this.emit_by_name::<()>("test-result-updated", &[&error_result]);
                        this.emit_by_name::<()>("test-status-changed", &[&test_id_clone, &DiagnosticStatus::Error]);
                    }
                }
            }));
        } else {
            warn!("VM: SystemHealthService not available for run_diagnostic_test.");
             // Potentially show a toast here too if the service itself is missing.
            if let Some(window_weak) = self.imp().app_window.borrow().as_ref() {
                if let Some(window_strong) = window_weak.upgrade() {
                    ui_feedback::show_error_toast(&window_strong, "Cannot run test: System service is unavailable.");
                }
            }
        }
    }

    // Connect methods (condensed for brevity, assuming they are defined as before)
    pub fn connect_cpu_metrics_updated<F: Fn(&Self, &CpuMetrics) + 'static>(&self, f: F) -> SignalHandlerId { self.connect_closure("cpu-metrics-updated", false, move |args| f(args[0].get().unwrap(), args[1].get().unwrap())) }
    pub fn connect_memory_metrics_updated<F: Fn(&Self, &MemoryMetrics) + 'static>(&self, f: F) -> SignalHandlerId { self.connect_closure("memory-metrics-updated", false, move |args| f(args[0].get().unwrap(), args[1].get().unwrap())) }
    pub fn connect_disk_activity_updated<F: Fn(&Self, &Variant) + 'static>(&self, f: F) -> SignalHandlerId { self.connect_closure("disk-activity-updated", false, move |args| f(args[0].get().unwrap(), args[1].get().unwrap())) }
    pub fn connect_disk_space_updated<F: Fn(&Self, &Variant) + 'static>(&self, f: F) -> SignalHandlerId { self.connect_closure("disk-space-updated", false, move |args| f(args[0].get().unwrap(), args[1].get().unwrap())) }
    pub fn connect_network_activity_updated<F: Fn(&Self, &Variant) + 'static>(&self, f: F) -> SignalHandlerId { self.connect_closure("network-activity-updated", false, move |args| f(args[0].get().unwrap(), args[1].get().unwrap())) }
    pub fn connect_temperature_metrics_updated<F: Fn(&Self, &Variant) + 'static>(&self, f: F) -> SignalHandlerId { self.connect_closure("temperature-metrics-updated", false, move |args| f(args[0].get().unwrap(), args[1].get().unwrap())) }
    pub fn connect_available_tests_changed<F: Fn(&Self, &Variant) + 'static>(&self, f: F) -> SignalHandlerId { self.connect_closure("available-tests-changed", false, move |args| f(args[0].get().unwrap(), args[1].get().unwrap())) }
    pub fn connect_test_result_updated<F: Fn(&Self, &DiagnosticTestResult) + 'static>(&self, f: F) -> SignalHandlerId { self.connect_closure("test-result-updated", false, move |args| f(args[0].get().unwrap(), args[1].get().unwrap())) }
    pub fn connect_test_status_changed<F: Fn(&Self, &DiagnosticTestId, &DiagnosticStatus) + 'static>(&self, f: F) -> SignalHandlerId { self.connect_closure("test-status-changed", false, move |args| f(args[0].get().unwrap(), args[1].get().unwrap(), args[2].get().unwrap())) }
}

// --- Re-inserting condensed subscription methods for metrics here ---
mod imp_metrics_subs {
    use super::*; use tokio::sync::broadcast; use log::{debug, error}; use glib::clone;
    impl imp::ViewModelPriv {
        pub fn start_cpu_metrics_subscription(&self, service: Arc<dyn SystemHealthServiceTrait>) { let mut cpu_rx = service.subscribe_to_cpu_metrics_updates(); let this = self.obj(); tokio::spawn(clone!(@weak this => async move { while let Ok(metrics) = cpu_rx.recv().await { this.imp().current_cpu_metrics.replace(Some(metrics.clone())); this.emit_by_name::<()>("cpu-metrics-updated", &[&metrics]); } debug!("VM Subscription: CPU metrics listener task ended."); })); }
        pub fn start_memory_metrics_subscription(&self, service: Arc<dyn SystemHealthServiceTrait>) { let mut memory_rx = service.subscribe_to_memory_metrics_updates(); let this = self.obj(); tokio::spawn(clone!(@weak this => async move { while let Ok(metrics) = memory_rx.recv().await { this.imp().latest_memory_metrics.replace(Some(metrics.clone())); this.emit_by_name::<()>("memory-metrics-updated", &[&metrics]); } debug!("VM Subscription: Memory metrics listener task ended."); })); }
        pub fn start_disk_activity_subscription(&self, service: Arc<dyn SystemHealthServiceTrait>) { let mut disk_rx = service.subscribe_to_disk_activity_metrics_updates(); let this = self.obj(); tokio::spawn(clone!(@weak this => async move { loop { match disk_rx.recv().await { Ok(metrics_vec) => { this.imp().current_disk_activity.replace(Some(metrics_vec.clone())); this.emit_by_name::<()>("disk-activity-updated", &[&vec_to_variant(&metrics_vec)]); } Err(broadcast::error::RecvError::Lagged(n)) => error!("VM Subscription: Disk activity lagged by {}", n), Err(e) => { error!("VM Subscription: Disk activity error: {:?}", e); break; } } } })); }
        pub fn start_disk_space_subscription(&self, service: Arc<dyn SystemHealthServiceTrait>) { let mut disk_space_rx = service.subscribe_to_disk_space_metrics_updates(); let this = self.obj(); tokio::spawn(clone!(@weak this => async move { loop { match disk_space_rx.recv().await { Ok(metrics_vec) => { this.imp().current_disk_space.replace(Some(metrics_vec.clone())); this.emit_by_name::<()>("disk-space-updated", &[&vec_to_variant(&metrics_vec)]); } Err(broadcast::error::RecvError::Lagged(n)) => error!("VM Subscription: Disk space lagged by {}", n), Err(e) => { error!("VM Subscription: Disk space error: {:?}", e); break; } } } })); }
        pub fn start_network_activity_subscription(&self, service: Arc<dyn SystemHealthServiceTrait>) { let mut network_rx = service.subscribe_to_network_activity_metrics_updates(); let this = self.obj(); tokio::spawn(clone!(@weak this => async move { loop { match network_rx.recv().await { Ok(metrics_vec) => { this.imp().current_network_activity.replace(Some(metrics_vec.clone())); this.emit_by_name::<()>("network-activity-updated", &[&vec_to_variant(&metrics_vec)]); } Err(broadcast::error::RecvError::Lagged(n)) => error!("VM Subscription: Network activity lagged {}", n), Err(e) => { error!("VM Subscription: Network activity error: {:?}", e); break; } } } })); }
        pub fn start_temperature_metrics_subscription(&self, service: Arc<dyn SystemHealthServiceTrait>) { let mut temp_rx = service.subscribe_to_temperature_metrics_updates(); let this = self.obj(); tokio::spawn(clone!(@weak this => async move { loop { match temp_rx.recv().await { Ok(metrics_vec) => { this.imp().current_temperatures.replace(Some(metrics_vec.clone())); this.emit_by_name::<()>("temperature-metrics-updated", &[&vec_to_variant(&metrics_vec)]); } Err(broadcast::error::RecvError::Lagged(n)) => error!("VM Subscription: Temperature lagged {}", n), Err(e) => { error!("VM Subscription: Temperature error: {:?}", e); break; } } } })); }
    }
}
// --- End of re-inserted metric subscriptions ---

#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::config::CoreConfig; // For DefaultSystemHealthService if used as a real fallback
    use novade_domain::error::DomainError; // For Mock errors
    use std::sync::Mutex;
    use tokio::sync::broadcast::{channel, Sender};
    use async_trait::async_trait;
    use novade_core::types::system_health::*; // Import all system_health types for convenience

    // Initialize GTK for tests
    fn init_gtk() {
        if !gtk::is_initialized() {
            gtk::test_init();
        }
    }

    // A dummy adw::ApplicationWindow for creating WeakRef
    fn create_dummy_window() -> adw::ApplicationWindow {
        if !gtk::is_initialized() { gtk::test_init(); }
        adw::ApplicationWindow::new(&gtk::Application::default().expect("Failed to get default app"))
    }


    // MockSystemHealthService Definition
    #[derive(Clone, Default)]
    struct MockSystemHealthService {
        run_diagnostic_test_handler: Arc<Mutex<Option<Box<dyn Fn(DiagnosticTestId) -> Result<DiagnosticTestResult, DomainError> + Send>>>>,
        list_available_diagnostic_tests_handler: Arc<Mutex<Option<Box<dyn Fn() -> Result<Vec<DiagnosticTestInfo>, DomainError> + Send>>>>,
        // Senders for metric subscriptions
        cpu_metrics_sender: Arc<Mutex<Option<Sender<Result<CpuMetrics, DomainError>>>>>,
        memory_metrics_sender: Arc<Mutex<Option<Sender<Result<MemoryMetrics, DomainError>>>>>,
        disk_activity_sender: Arc<Mutex<Option<Sender<Result<Vec<DiskActivityMetrics>, DomainError>>>>>,
        disk_space_sender: Arc<Mutex<Option<Sender<Result<Vec<DiskSpaceMetrics>, DomainError>>>>>,
        network_activity_sender: Arc<Mutex<Option<Sender<Result<Vec<NetworkActivityMetrics>, DomainError>>>>>,
        temperature_sender: Arc<Mutex<Option<Sender<Result<Vec<TemperatureMetric>, DomainError>>>>>,
    }

    impl MockSystemHealthService {
        fn new() -> Self { Self::default() }

        fn set_run_diagnostic_test_handler<F>(&self, handler: F)
        where F: Fn(DiagnosticTestId) -> Result<DiagnosticTestResult, DomainError> + Send + 'static,
        { *self.run_diagnostic_test_handler.lock().unwrap() = Some(Box::new(handler)); }

        #[allow(dead_code)] // May not be used in all tests
        fn set_list_available_diagnostic_tests_handler<F>(&self, handler: F)
        where F: Fn() -> Result<Vec<DiagnosticTestInfo>, DomainError> + Send + 'static,
        { *self.list_available_diagnostic_tests_handler.lock().unwrap() = Some(Box::new(handler)); }


        fn get_cpu_metrics_sender(&self) -> Sender<Result<CpuMetrics, DomainError>> {
            let mut guard = self.cpu_metrics_sender.lock().unwrap();
            if guard.is_none() { *guard = Some(channel(16).0); }
            guard.as_ref().unwrap().clone()
        }
        // Similar getters for other metric senders if needed for tests
    }

    #[async_trait]
    impl SystemHealthServiceTrait for MockSystemHealthService {
        fn get_config(&self) -> Arc<CoreConfig> { Arc::new(CoreConfig::default()) }

        async fn get_cpu_metrics(&self) -> Result<CpuMetrics, DomainError> { unimplemented!() }
        async fn get_memory_metrics(&self) -> Result<MemoryMetrics, DomainError> { unimplemented!() }
        async fn get_disk_activity_metrics(&self) -> Result<Vec<DiskActivityMetrics>, DomainError> { unimplemented!() }
        async fn get_disk_space_metrics(&self) -> Result<Vec<DiskSpaceMetrics>, DomainError> { unimplemented!() }
        async fn get_network_activity_metrics(&self) -> Result<Vec<NetworkActivityMetrics>, DomainError> { unimplemented!() }
        async fn get_temperature_metrics(&self) -> Result<Vec<TemperatureMetric>, DomainError> { unimplemented!() }
        async fn get_available_log_sources(&self) -> Result<Vec<LogSourceIdentifier>, DomainError> { unimplemented!() }
        async fn query_log_entries(&self, _filter: LogFilter) -> Result<Vec<LogEntry>, DomainError> { unimplemented!() }

        async fn list_available_diagnostic_tests(&self) -> Result<Vec<DiagnosticTestInfo>, DomainError> {
            if let Some(handler) = self.list_available_diagnostic_tests_handler.lock().unwrap().as_ref() {
                return handler();
            }
            Ok(vec![]) // Default empty list
        }

        async fn run_diagnostic_test(&self, id: DiagnosticTestId, _params: Option<serde_json::Value>) -> Result<DiagnosticTestResult, DomainError> {
            if let Some(handler) = self.run_diagnostic_test_handler.lock().unwrap().as_ref() {
                return handler(id);
            }
            Err(DomainError::NotImplemented("run_diagnostic_test mock not set".to_string()))
        }

        fn subscribe_to_cpu_metrics_updates(&self) -> broadcast::Receiver<Result<CpuMetrics, DomainError>> {
            self.get_cpu_metrics_sender().subscribe()
        }
        fn subscribe_to_memory_metrics_updates(&self) -> broadcast::Receiver<Result<MemoryMetrics, DomainError>> {
            self.memory_metrics_sender.lock().unwrap().get_or_insert_with(|| channel(16).0).clone().subscribe()
        }
        fn subscribe_to_disk_activity_metrics_updates(&self) -> broadcast::Receiver<Result<Vec<DiskActivityMetrics>, DomainError>> {
            self.disk_activity_sender.lock().unwrap().get_or_insert_with(|| channel(16).0).clone().subscribe()
        }
        fn subscribe_to_disk_space_metrics_updates(&self) -> broadcast::Receiver<Result<Vec<DiskSpaceMetrics>, DomainError>> {
            self.disk_space_sender.lock().unwrap().get_or_insert_with(|| channel(16).0).clone().subscribe()
        }
        fn subscribe_to_network_activity_metrics_updates(&self) -> broadcast::Receiver<Result<Vec<NetworkActivityMetrics>, DomainError>> {
            self.network_activity_sender.lock().unwrap().get_or_insert_with(|| channel(16).0).clone().subscribe()
        }
        fn subscribe_to_temperature_metrics_updates(&self) -> broadcast::Receiver<Result<Vec<TemperatureMetric>, DomainError>> {
            self.temperature_sender.lock().unwrap().get_or_insert_with(|| channel(16).0).clone().subscribe()
        }
        fn subscribe_to_log_updates(&self, _filter: LogFilter) -> broadcast::Receiver<Result<LogEntry, DomainError>> { unimplemented!() }
        fn subscribe_to_alerts(&self) -> broadcast::Receiver<Result<Alert, DomainError>> { unimplemented!() }
    }

    #[tokio::test] // Use tokio::test for async tests
    async fn test_vm_run_diagnostic_test_error_signals() {
        init_gtk();
        let mock_service = Arc::new(MockSystemHealthService::new());
        let test_id = DiagnosticTestId("test_error_case".to_string());
        let test_id_clone = test_id.clone();

        mock_service.set_run_diagnostic_test_handler(move |id| {
            assert_eq!(id, test_id_clone); // Ensure correct test_id is passed
            Err(DomainError::Unknown("Simulated test error".to_string()))
        });

        let dummy_window = create_dummy_window();
        let view_model = SystemHealthViewModel::new(mock_service.clone(), dummy_window.downgrade());

        let status_changed_data = Arc::new(Mutex::new(Vec::new()));
        let result_updated_data = Arc::new(Mutex::new(None));

        let status_data_clone = status_changed_data.clone();
        view_model.connect_test_status_changed(move |_vm, id, status| {
            status_data_clone.lock().unwrap().push((id.clone(), status.clone()));
        });

        let result_data_clone = result_updated_data.clone();
        view_model.connect_test_result_updated(move |_vm, result| {
            *result_data_clone.lock().unwrap() = Some(result.clone());
        });

        view_model.run_diagnostic_test(&test_id);

        // Allow GLib main context to process events from spawn_local
        // Loop a few times to ensure futures are polled and signals emitted.
        for _ in 0..5 {
            glib::MainContext::default().iteration(false); // Non-blocking
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await; // Brief yield
        }

        let statuses = status_changed_data.lock().unwrap();
        assert_eq!(statuses.len(), 2, "Should have received two status updates");
        assert_eq!(statuses[0].0, test_id, "First status update ID mismatch");
        assert_eq!(statuses[0].1, DiagnosticStatus::Running, "First status should be Running");
        assert_eq!(statuses[1].0, test_id, "Second status update ID mismatch");
        assert_eq!(statuses[1].1, DiagnosticStatus::Error, "Second status should be Error");

        let result_opt = result_updated_data.lock().unwrap();
        assert!(result_opt.is_some(), "Should have received a test result");
        if let Some(result) = result_opt.as_ref() {
            assert_eq!(result.id, test_id, "Result ID mismatch");
            assert_eq!(result.status, DiagnosticStatus::Error, "Result status should be Error");
            assert!(result.summary.contains("Simulated test error"), "Error message mismatch");
        }
    }

    #[tokio::test]
    async fn test_vm_cpu_metrics_signal_emission() {
        init_gtk();
        let mock_service = Arc::new(MockSystemHealthService::new());
        let cpu_sender = mock_service.get_cpu_metrics_sender();

        let dummy_window = create_dummy_window();
        let view_model = SystemHealthViewModel::new(mock_service.clone(), dummy_window.downgrade());

        let received_metrics = Arc::new(Mutex::new(None));
        let received_metrics_clone = received_metrics.clone();

        view_model.connect_cpu_metrics_updated(move |_vm, metrics| {
            *received_metrics_clone.lock().unwrap() = Some(metrics.clone());
        });

        let test_cpu_data = CpuMetrics { total_usage_percent: 55.5, per_core_usage_percent: vec![50.0, 60.0] };
        cpu_sender.send(Ok(test_cpu_data.clone())).unwrap();

        for _ in 0..5 {
            glib::MainContext::default().iteration(false);
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let received_opt = received_metrics.lock().unwrap();
        assert!(received_opt.is_some(), "Should have received CPU metrics");
        if let Some(metrics) = received_opt.as_ref() {
            assert_eq!(metrics.total_usage_percent, test_cpu_data.total_usage_percent);
            assert_eq!(metrics.per_core_usage_percent, test_cpu_data.per_core_usage_percent);
        }
    }
}
