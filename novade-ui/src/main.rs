use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use novade_ui::shell::panel_widget::{PanelWidget, PanelPosition, ModulePosition};
use novade_ui::shell::panel_widget::app_menu_button::AppMenuButton;
use novade_ui::shell::panel_widget::workspace_indicator_widget::WorkspaceIndicatorWidget;
use novade_ui::shell::panel_widget::clock_datetime_widget::ClockDateTimeWidget;
use novade_ui::shell::panel_widget::quick_settings_button::QuickSettingsButtonWidget; 
use novade_ui::shell::panel_widget::notification_center_button::NotificationCenterButtonWidget; 
use novade_ui::shell::panel_widget::network_management_widget::NetworkManagementWidget;
// CpuUsageWidget import
use novade_ui::shell::panel_widget::cpu_usage_widget::CpuUsageWidget;

// Domain and Connector imports
use novade_domain::workspaces::{StubWorkspaceManager, WorkspaceManager}; 
use novade_ui::shell::domain_workspace_connector::DomainWorkspaceConnector;
use novade_ui::shell::panel_widget::workspace_indicator_widget::types::WorkspaceInfo as UiWorkspaceInfo; 
// System Info Provider import
use novade_system::window_info_provider::{StubSystemWindowInfoProvider, WaylandWindowInfoProvider, SystemWindowInfoProvider};
// DBusMenuProvider import
use novade_system::dbus_menu_provider::StubDBusMenuProvider; 
// AppMenuService import
use novade_ui::shell::app_menu_service::AppMenuService; 

// ICpuUsageService and related imports for the stub
use novade_domain::cpu_usage_service::{ICpuUsageService, SubscriptionId};
use novade_domain::error::DomainError;
use tokio::sync::mpsc as tokio_mpsc; // For the sender in ICpuUsageService
use async_trait::async_trait; // For the stub impl

use std::sync::Arc; 
use std::rc::Rc; // For Rc used with services
use gtk::gio::{self, SimpleAction}; 
use gtk::glib::VariantTy; 
use tracing;

const APP_ID: &str = "org.novade.UIShellTest";

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    gtk4_layer_shell::init_for_gtk_window();
    let app = Application::builder().application_id(APP_ID).build();
    let tokio_runtime_handle = tokio::runtime::Handle::current();

    app.connect_activate(move |app| { 
        let actions = [
            ("open", None, "Open File"), ("save_as", None, "Save As..."), ("quit", None, "Quit App"),
            ("copy", None, "Copy Text"), ("paste", None, "Paste Text"), ("about", None, "About App"),
            ("preferences", None, "Preferences"), ("details", None, "Show Details"),
        ];
        for (name, parameter_type_str, _label) in actions {
            let action = SimpleAction::new(name, parameter_type_str.map(|s| VariantTy::new(s).expect("Invalid VariantTy string")));
            let captured_name = name.to_string();
            action.connect_activate(move |_act, var| {
                tracing::info!("Action '{}' activated with parameter {:?}", captured_name, var);
            });
            app.add_action(&action);
        }
        build_ui(app, tokio_runtime_handle.clone());
    });
    app.run();
}

fn build_ui(app: &Application, tokio_handle: tokio::runtime::Handle) { 
    let panel = PanelWidget::new(app);

    // --- Placeholder/Stub ICpuUsageService ---
    #[derive(Debug)]
    struct StubCpuUsageService;
    #[async_trait]
    impl ICpuUsageService for StubCpuUsageService {
        async fn get_current_cpu_percentage(&self) -> Result<f64, DomainError> {
            Ok(33.3) // Dummy value
        }
        async fn subscribe_to_cpu_updates(
            &self,
            sender: tokio_mpsc::UnboundedSender<Result<f64, DomainError>>,
        ) -> Result<SubscriptionId, DomainError> {
            let sub_id = SubscriptionId::new_v4();
            let _th = tokio_handle.clone(); // Clone handle for the task
            tokio_handle.spawn(async move {
                let mut i = 0;
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    let val = 30.0 + (i % 7) as f64; // Cycle through 30-36
                    if sender.send(Ok(val)).is_err() {
                        tracing::info!("[StubCpuUsageService] Subscriber channel closed for ID: {}", sub_id);
                        break;
                    }
                    i += 1;
                }
            });
            tracing::info!("[StubCpuUsageService] Subscribed with ID: {}, sending dummy values.", sub_id);
            Ok(sub_id)
        }
        async fn unsubscribe_from_cpu_updates(&self, id: SubscriptionId) -> Result<(), DomainError> {
            tracing::info!("[StubCpuUsageService] Unsubscribed ID: {}", id);
            Ok(())
        }
    }
    let cpu_usage_service_stub: Arc<dyn ICpuUsageService> = Arc::new(StubCpuUsageService);
    // --- End Placeholder ICpuUsageService ---

    let system_window_provider: Arc<dyn SystemWindowInfoProvider> =
        match WaylandWindowInfoProvider::new() {
            Ok(provider) => Arc::new(provider),
            Err(_) => Arc::new(StubSystemWindowInfoProvider::new()),
        };

    let active_window_service = Rc::new(
        novade_ui::shell::active_window_service::ActiveWindowService::new(system_window_provider.clone())
    );
    let dbus_menu_provider = Arc::new(StubDBusMenuProvider::new());
    let app_menu_service = Rc::new(AppMenuService::new(dbus_menu_provider.clone()));
    let app_menu_button = AppMenuButton::new();
    app_menu_button.set_active_window_service(active_window_service.clone());
    app_menu_button.set_app_menu_service(app_menu_service.clone());

    let domain_workspace_manager: Arc<dyn WorkspaceManager> = Arc::new(StubWorkspaceManager::new());
    let (ui_workspace_sender, ui_workspace_receiver) = 
        glib::MainContext::channel::<Vec<UiWorkspaceInfo>>(glib::Priority::DEFAULT);
    let _domain_connector = Rc::new(DomainWorkspaceConnector::new(
        domain_workspace_manager.clone(),
        ui_workspace_sender,
        tokio_handle.clone(),
    ));
    let workspace_indicator = WorkspaceIndicatorWidget::new();
    workspace_indicator.set_domain_workspace_connector(_domain_connector.clone(), ui_workspace_receiver);

    let clock_widget = ClockDateTimeWidget::new();
    let quick_settings_button = QuickSettingsButtonWidget::new(); 
    let notification_center_button = NotificationCenterButtonWidget::new(); 
    let network_widget = NetworkManagementWidget::new();

    // --- CpuUsageWidget Setup ---
    let cpu_widget = CpuUsageWidget::new();
    cpu_widget.set_cpu_usage_service(cpu_usage_service_stub.clone());

    // Add modules to the panel
    panel.add_module(&app_menu_button, ModulePosition::Start, 0);
    panel.add_module(&workspace_indicator, ModulePosition::Center, 0);
    panel.add_module(&cpu_widget, ModulePosition::Center, 1); // Added CPU widget
    panel.add_module(&network_widget, ModulePosition::End, 0);
    panel.add_module(&clock_widget, ModulePosition::End, 1); 
    panel.add_module(&quick_settings_button, ModulePosition::End, 2); 
    panel.add_module(&notification_center_button, ModulePosition::End, 3); 
    
    let app_menu_button_clone = app_menu_button.clone();
    glib::timeout_add_seconds_local(2, move || {
        app_menu_button_clone.refresh_display();
        glib::ControlFlow::Continue 
    });
    app_menu_button.refresh_display();

    // Test Dynamic Workspace Changes (omitted for brevity in this diff, assume it's still there)

    panel.present();
    
    // Start CPU widget subscription after panel is presented and part of the UI hierarchy.
    cpu_widget.start_subscription();
}
