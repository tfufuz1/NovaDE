use gtk::prelude::*;
use gtk::{Application, ApplicationWindow}; // ApplicationWindow might not be directly used for PanelWidget
use novade_ui::shell::panel_widget::{PanelWidget, PanelPosition, ModulePosition};
use novade_ui::shell::panel_widget::app_menu_button::AppMenuButton;
use novade_ui::shell::panel_widget::workspace_indicator_widget::WorkspaceIndicatorWidget;
use novade_ui::shell::panel_widget::clock_datetime_widget::ClockDateTimeWidget;
use novade_ui::shell::panel_widget::quick_settings_button::QuickSettingsButtonWidget; 
use novade_ui::shell::panel_widget::notification_center_button::NotificationCenterButtonWidget; 
use novade_ui::shell::panel_widget::network_management_widget::NetworkManagementWidget;
// Domain and Connector imports
use novade_domain::workspaces::{StubWorkspaceManager, WorkspaceManager}; 
use novade_ui::shell::domain_workspace_connector::DomainWorkspaceConnector;
use novade_ui::shell::panel_widget::workspace_indicator_widget::types::WorkspaceInfo as UiWorkspaceInfo; 
// System Info Provider import
use novade_system::window_info_provider::StubSystemWindowInfoProvider; // Import the stub

use std::sync::Arc; 
use gtk::gio::{self, SimpleAction}; 
use gtk::glib::VariantTy; 

const APP_ID: &str = "org.novade.UIShellTest";

#[tokio::main] // Make main async
async fn main() { // Make main async
    tracing_subscriber::fmt::init(); // Initialize tracing
    
    // Initialize GDK and GTK layer shell
    gtk4_layer_shell::init_for_gtk_window();

    let app = Application::builder().application_id(APP_ID).build();

    // Get a handle to the current Tokio runtime for DomainWorkspaceConnector
    let tokio_runtime_handle = tokio::runtime::Handle::current();

    app.connect_activate(move |app| { 
        // Define actions here, as `app` is the GtkApplication instance
        let actions = [
            ("open", None, "Open File"),
            ("save_as", None, "Save As..."),
            ("quit", None, "Quit App"),
            ("copy", None, "Copy Text"),
            ("paste", None, "Paste Text"),
            ("about", None, "About App"),
            ("preferences", None, "Preferences"),
            ("details", None, "Show Details"),
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

    // GTK run is not async, it blocks the main thread.
    app.run();
}

fn build_ui(app: &Application, tokio_handle: tokio::runtime::Handle) { 
    // Create the PanelWidget
    let panel = PanelWidget::new(app);

    // --- SystemWindowInfoProvider Setup ---
    let system_window_provider = Arc::new(StubSystemWindowInfoProvider::new());

    // --- ActiveWindowService for AppMenuButton ---
    // Pass the system_window_provider to ActiveWindowService constructor
    let active_window_service = Rc::new(
        novade_ui::shell::active_window_service::ActiveWindowService::new(system_window_provider.clone())
    );
    let app_menu_button = AppMenuButton::new();
    app_menu_button.set_active_window_service(active_window_service.clone());

    // --- Domain Workspace Management Setup ---
    let domain_workspace_manager: Arc<dyn WorkspaceManager> = Arc::new(StubWorkspaceManager::new());
    
    // Create a glib channel for UI updates for WorkspaceIndicatorWidget
    let (ui_workspace_sender, ui_workspace_receiver) = 
        glib::MainContext::channel::<Vec<UiWorkspaceInfo>>(glib::Priority::DEFAULT);

    // Instantiate DomainWorkspaceConnector
    let _domain_connector = Rc::new(DomainWorkspaceConnector::new( // Store in Rc for potential future use if needed
        domain_workspace_manager.clone(), // Pass the domain manager
        ui_workspace_sender,          // Pass the sender part of the channel
        tokio_handle.clone(),         // Pass the Tokio runtime handle
    ));
    
    let workspace_indicator = WorkspaceIndicatorWidget::new();
    // Pass the connector and the receiver to the widget
    // Note: We pass domain_connector (the Rc) here, not domain_workspace_manager directly to the UI widget
    // The domain_connector Rc is not actually used by set_domain_workspace_connector in the widget's mod.rs,
    // only its type is needed for the method signature. The widget stores the Rc<DomainWorkspaceConnector>
    // in its imp.rs.
    // We pass the *same* Rc<DomainWorkspaceConnector> that was used to create the listener.
    workspace_indicator.set_domain_workspace_connector(_domain_connector.clone(), ui_workspace_receiver);


    // --- Other Widgets (existing) ---
    let clock_widget = ClockDateTimeWidget::new();
    let quick_settings_button = QuickSettingsButtonWidget::new(); 
    let notification_center_button = NotificationCenterButtonWidget::new(); 
    let network_widget = NetworkManagementWidget::new();

    // Add modules to the panel
    panel.add_module(&app_menu_button, ModulePosition::Start, 0);
    panel.add_module(&workspace_indicator, ModulePosition::Center, 0);
    panel.add_module(&network_widget, ModulePosition::End, 0);
    panel.add_module(&clock_widget, ModulePosition::End, 1); 
    panel.add_module(&quick_settings_button, ModulePosition::End, 2); 
    panel.add_module(&notification_center_button, ModulePosition::End, 3); 
    
    // Test properties
    // panel.set_position(PanelPosition::Bottom);
    // panel.set_panel_height(60);
    // panel.set_transparency_enabled(true); // Depends on compositor and theme

    // --- AppMenuButton periodic refresh (existing) ---
    let app_menu_button_clone = app_menu_button.clone();
    glib::timeout_add_seconds_local(2, move || {
        // println!("Refreshing AppMenuButton display via timeout..."); // Less verbose
        app_menu_button_clone.refresh_display();
        glib::ControlFlow::Continue 
    });
    app_menu_button.refresh_display(); // Initial refresh

    // WorkspaceIndicatorWidget is now driven by events from DomainWorkspaceConnector.
    // Initial data load is handled by set_domain_workspace_connector.
    // Clicks on workspace items trigger service calls, which then lead to events.

    // --- Test Dynamic Workspace Changes ---
    let domain_manager_clone_for_create = domain_workspace_manager.clone();
    let tokio_handle_clone_for_create = tokio_handle.clone();
    glib::timeout_add_seconds_local_once(5, move || {
        let dm_create = domain_manager_clone_for_create.clone();
        tokio_handle_clone_for_create.spawn(async move {
            match dm_create.create_workspace("New WS Alpha".to_string()).await {
                Ok(ws) => tracing::info!("Test: Created workspace: id='{}', name='{}'", ws.id, ws.name),
                Err(e) => tracing::error!("Test: Error creating workspace: {:?}", e),
            }
        });
    });

    let domain_manager_clone_for_delete = domain_workspace_manager.clone();
    let tokio_handle_clone_for_delete = tokio_handle.clone();
    // Assuming "ws2" is an ID from the initial stubs in StubWorkspaceManager
    let id_to_delete = "ws2".to_string(); 
    glib::timeout_add_seconds_local_once(10, move || {
        let dm_delete = domain_manager_clone_for_delete.clone();
        tokio_handle_clone_for_delete.spawn(async move {
            match dm_delete.delete_workspace(id_to_delete.clone()).await {
                Ok(()) => tracing::info!("Test: Deleted workspace {}", id_to_delete),
                Err(e) => tracing::error!("Test: Error deleting workspace {}: {:?}", id_to_delete, e),
            }
        });
    });
    
    // Test creating another workspace to see if active state is handled if ws2 was active
    let domain_manager_clone_for_create2 = domain_workspace_manager.clone();
    let tokio_handle_clone_for_create2 = tokio_handle.clone();
    glib::timeout_add_seconds_local_once(15, move || {
        let dm_create2 = domain_manager_clone_for_create2.clone();
        tokio_handle_clone_for_create2.spawn(async move {
            match dm_create2.create_workspace("Another New WS Beta".to_string()).await {
                Ok(ws) => tracing::info!("Test: Created workspace: id='{}', name='{}'", ws.id, ws.name),
                Err(e) => tracing::error!("Test: Error creating workspace: {:?}", e),
            }
        });
    });


    panel.present();
}
// Add Rc to imports
use std::rc::Rc;
