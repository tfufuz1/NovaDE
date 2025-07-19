use gtk4 as gtk;
use gtk::{prelude::*, Box as GtkBox, Orientation, Label, Notebook};
use std::sync::Arc;
use novade_domain::system_health_service::service::SystemHealthServiceTrait;
use glib::{subclass::prelude::*, clone, Properties, Variant, WeakRef}; // Added WeakRef
use libadwaita as adw; // Added adw

use super::{view_model::SystemHealthViewModel, metrics_panel::MetricsPanel};
use super::log_viewer_panel::LogViewerPanel;
use super::diagnostics_panel::DiagnosticsPanel;
use super::alerts_panel::AlertsPanel;

use novade_core::types::system_health::{CpuMetrics, MemoryMetrics}; // Already here for existing signals
// No need to import DiskActivityMetrics etc. here, as they are wrapped in Variant by the ViewModel
use log::debug;

mod imp {
    use super::*;
    use std::cell::RefCell;

    use crate::window_manager::WindowManager;
    use std::sync::Arc;
    #[derive(Debug, Properties, Default)]
    #[properties(wrapper_type = super::SystemHealthDashboardView)]
    pub struct SystemHealthDashboardViewPriv {
        #[property(get, set)]
        pub view_model: RefCell<Option<SystemHealthViewModel>>,
        pub metrics_panel: RefCell<Option<MetricsPanel>>,
        pub log_viewer_panel: RefCell<Option<LogViewerPanel>>,
        pub diagnostics_panel: RefCell<Option<DiagnosticsPanel>>,
        pub alerts_panel: RefCell<Option<AlertsPanel>>,
        pub notebook: RefCell<Option<Notebook>>,
        pub window_manager: RefCell<Option<Arc<WindowManager>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SystemHealthDashboardViewPriv {
        const NAME: &'static str = "NovaSystemHealthDashboardView";
        type Type = super::SystemHealthDashboardView;
        type ParentType = GtkBox;

        fn new() -> Self {
            Self {
                view_model: RefCell::new(None),
                metrics_panel: RefCell::new(None),
                log_viewer_panel: RefCell::new(None),
                diagnostics_panel: RefCell::new(None),
                alerts_panel: RefCell::new(None),
                notebook: RefCell::new(None),
            }
        }
    }

    impl ObjectImpl for SystemHealthDashboardViewPriv {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_orientation(Orientation::Vertical);
            obj.set_spacing(6);
            obj.set_margin_top(12);
            obj.set_margin_bottom(12);
            obj.set_margin_start(12);
            obj.set_margin_end(12);

            let notebook = Notebook::new();
            obj.append(&notebook);
            self.notebook.replace(Some(notebook));
        }

        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec);
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }
    }

    impl WidgetImpl for SystemHealthDashboardViewPriv {}
    impl BoxImpl for SystemHealthDashboardViewPriv {}
}

glib::wrapper! {
    pub struct SystemHealthDashboardView(ObjectSubclass<imp::SystemHealthDashboardViewPriv>)
        @extends GtkBox, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

use crate::window_manager::WindowManager;
use crate::context_menu::ContextMenuManager;
use crate::styles::StyleManager;
use gtk::gdk;

impl SystemHealthDashboardView {
    // Updated constructor signature
    pub fn new(
        service: Arc<dyn SystemHealthServiceTrait>,
        main_app_window: WeakRef<adw::ApplicationWindow>,
        window_manager: Arc<WindowManager>,
        context_menu_manager: Arc<ContextMenuManager>,
    ) -> Self {
        let obj: Self = glib::Object::builder().build();
        let imp = obj.imp();
        imp.window_manager.replace(Some(window_manager.clone()));

        let gesture = gtk::GestureClick::new();
        gesture.set_button(gdk::BUTTON_SECONDARY);
        gesture.connect_pressed(move |gesture, _, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let position = (x as i32, y as i32);
            if let Ok(menu) = context_menu_manager.create_window_context_menu(position, window_manager.clone()) {
                if let Err(e) = context_menu_manager.show_menu(&menu.id, Some(position)) {
                    eprintln!("Failed to show context menu: {}", e);
                }
            }
        });
        obj.add_controller(gesture);

        // Pass main_app_window to SystemHealthViewModel constructor
        let vm = SystemHealthViewModel::new(service.clone(), main_app_window);
        imp.view_model.replace(Some(vm.clone()));

        let mp = MetricsPanel::new(vm.clone());
        imp.metrics_panel.replace(Some(mp));

        if let (Some(notebook), Some(metrics_panel_ref)) = (imp.notebook.borrow().as_ref(), imp.metrics_panel.borrow().as_ref()) {
            notebook.append_page(metrics_panel_ref.get_widget(), Some(&Label::new(Some("Metrics"))));
        }

        let logs_panel = LogViewerPanel::new(service.clone());
        imp.log_viewer_panel.replace(Some(logs_panel));
        if let (Some(notebook), Some(panel_ref)) = (imp.notebook.borrow().as_ref(), imp.log_viewer_panel.borrow().as_ref()) {
            notebook.append_page(panel_ref.get_widget(), Some(&Label::new(Some("Logs"))));
        }

        // Pass ViewModel to DiagnosticsPanel
        let diagnostics_panel = DiagnosticsPanel::new(vm.clone()); // Use vm (ViewModel)
        imp.diagnostics_panel.replace(Some(diagnostics_panel));
        if let (Some(notebook), Some(panel_ref)) = (imp.notebook.borrow().as_ref(), imp.diagnostics_panel.borrow().as_ref()) {
            notebook.append_page(panel_ref.get_widget(), Some(&Label::new(Some("Diagnostics"))));
        }
        // Trigger initial load of diagnostic tests
        vm.load_available_tests();

        let alerts_panel = AlertsPanel::new(service.clone());
        imp.alerts_panel.replace(Some(alerts_panel));
        if let (Some(notebook), Some(panel_ref)) = (imp.notebook.borrow().as_ref(), imp.alerts_panel.borrow().as_ref()) {
            notebook.append_page(panel_ref.get_widget(), Some(&Label::new(Some("Alerts"))));
        }

        // --- Connect ViewModel Signals to MetricsPanel update methods ---
        let view_model_instance = vm; // Use the vm created above, it's already the one stored in imp

        // Existing CPU and Memory connections
        view_model_instance.connect_cpu_metrics_updated(clone!(@weak obj => move |_vm, metrics: &CpuMetrics| {
            if let Some(panel) = obj.imp().metrics_panel.borrow().as_ref() {
                panel.update_cpu_metrics_display(metrics);
            }
        }));

        view_model_instance.connect_memory_metrics_updated(clone!(@weak obj => move |_vm, metrics: &MemoryMetrics| {
            if let Some(panel) = obj.imp().metrics_panel.borrow().as_ref() {
                panel.update_memory_metrics_display(metrics);
            }
        }));

        // New connections for Disk, Network, Temperature
        view_model_instance.connect_disk_activity_updated(clone!(@weak obj => move |_vm, metrics_variant: &Variant| {
            debug!("MainView: Received disk-activity-updated signal");
            if let Some(panel) = obj.imp().metrics_panel.borrow().as_ref() {
                panel.update_disk_activity_display(metrics_variant);
            }
        }));

        view_model_instance.connect_disk_space_updated(clone!(@weak obj => move |_vm, metrics_variant: &Variant| {
            debug!("MainView: Received disk-space-updated signal");
            if let Some(panel) = obj.imp().metrics_panel.borrow().as_ref() {
                panel.update_disk_space_display(metrics_variant);
            }
        }));

        view_model_instance.connect_network_activity_updated(clone!(@weak obj => move |_vm, metrics_variant: &Variant| {
            debug!("MainView: Received network-activity-updated signal");
            if let Some(panel) = obj.imp().metrics_panel.borrow().as_ref() {
                panel.update_network_activity_display(metrics_variant);
            }
        }));

        view_model_instance.connect_temperature_metrics_updated(clone!(@weak obj => move |_vm, metrics_variant: &Variant| {
            debug!("MainView: Received temperature-metrics-updated signal");
            if let Some(panel) = obj.imp().metrics_panel.borrow().as_ref() {
                panel.update_temperature_metrics_display(metrics_variant);
            }
        }));

        debug!("SystemHealthDashboardView (GObject) created, ViewModel and MetricsPanel initialized and ALL signals connected.");
        obj
    }
}
