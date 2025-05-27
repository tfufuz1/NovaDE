use gtk::prelude::*;
use gtk::{Application, ApplicationWindow}; // ApplicationWindow might not be directly used for PanelWidget
use novade_ui::shell::panel_widget::{PanelWidget, PanelPosition, ModulePosition};
use novade_ui::shell::panel_widget::app_menu_button::AppMenuButton;
use novade_ui::shell::panel_widget::workspace_indicator_widget::WorkspaceIndicatorWidget;
use novade_ui::shell::panel_widget::clock_datetime_widget::ClockDateTimeWidget;
use novade_ui::shell::panel_widget::quick_settings_button::QuickSettingsButtonWidget; 
use novade_ui::shell::panel_widget::notification_center_button::NotificationCenterButtonWidget; 
// Domain and Connector imports
use novade_domain::workspaces::{StubWorkspaceManager, WorkspaceManager}; // Assuming these are re-exported at this level
use novade_ui::shell::domain_workspace_connector::DomainWorkspaceConnector;
use novade_ui::shell::panel_widget::workspace_indicator_widget::types::WorkspaceInfo as UiWorkspaceInfo; 

use std::sync::Arc; 
use gtk::gio::{self, SimpleAction}; // Added gio::SimpleAction
use gtk::glib::VariantTy; // Added VariantTy

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

    // --- ActiveWindowService for AppMenuButton (existing) ---
    let active_window_service = Rc::new(novade_ui::shell::active_window_service::ActiveWindowService::new());
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

    // Add modules to the panel
    panel.add_module(&app_menu_button, ModulePosition::Start, 0);
    panel.add_module(&workspace_indicator, ModulePosition::Center, 0);
    panel.add_module(&clock_widget, ModulePosition::End, 0); 
    panel.add_module(&quick_settings_button, ModulePosition::End, 1); 
    panel.add_module(&notification_center_button, ModulePosition::End, 2); 
    
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

    panel.present();
}
// Add Rc to imports
use std::rc::Rc;
