use gtk::prelude::*;
use gtk::{Application, ApplicationWindow}; // ApplicationWindow might not be directly used for PanelWidget
use novade_ui::shell::panel_widget::{PanelWidget, PanelPosition, ModulePosition};
use novade_ui::shell::panel_widget::app_menu_button::AppMenuButton;
use novade_ui::shell::panel_widget::workspace_indicator_widget::WorkspaceIndicatorWidget;
use novade_ui::shell::panel_widget::clock_datetime_widget::ClockDateTimeWidget;
use novade_ui::shell::panel_widget::quick_settings_button::QuickSettingsButtonWidget; // Import new widget
use novade_ui::shell::panel_widget::notification_center_button::NotificationCenterButtonWidget; // Import new widget

const APP_ID: &str = "org.novade.UIShellTest";

fn main() {
    // Initialize GDK and GTK layer shell
    gtk4_layer_shell::init_for_gtk_window();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        build_ui(app);
    });

    app.run();
}

fn build_ui(app: &Application) {
    // Create the PanelWidget
    let panel = PanelWidget::new(app);

    // Create module instances
    // Create ActiveWindowService
    let active_window_service = Rc::new(novade_ui::shell::active_window_service::ActiveWindowService::new());

    let app_menu_button = AppMenuButton::new();
    app_menu_button.set_active_window_service(active_window_service.clone()); // Set the service

    let workspace_indicator = WorkspaceIndicatorWidget::new();
    let clock_widget = ClockDateTimeWidget::new();
    let quick_settings_button = QuickSettingsButtonWidget::new(); // Instantiate new widget
    let notification_center_button = NotificationCenterButtonWidget::new(); // Instantiate new widget

    // Add modules to the panel
    panel.add_module(&app_menu_button, ModulePosition::Start, 0);
    panel.add_module(&workspace_indicator, ModulePosition::Center, 0);
    // Add new buttons to the end, Clock will be first in end_box, then quick_settings, then notification_center
    panel.add_module(&clock_widget, ModulePosition::End, 0); 
    panel.add_module(&quick_settings_button, ModulePosition::End, 1); // Order 1
    panel.add_module(&notification_center_button, ModulePosition::End, 2); // Order 2
    
    // Test properties
    // panel.set_position(PanelPosition::Bottom);
    // panel.set_panel_height(60);
    // panel.set_transparency_enabled(true); // Depends on compositor and theme


    // Periodically refresh AppMenuButton to simulate active window changes
    let app_menu_button_clone = app_menu_button.clone();
    glib::timeout_add_seconds_local(2, move || {
        println!("Refreshing AppMenuButton display via timeout...");
        app_menu_button_clone.refresh_display();
        glib::ControlFlow::Continue // Keep the timer running
    });
    
    // Initial refresh for AppMenuButton
    app_menu_button.refresh_display();

    // Create and set ShellWorkspaceService for WorkspaceIndicatorWidget
    let shell_workspace_service = Rc::new(novade_ui::shell::shell_workspace_service::ShellWorkspaceService::new());
    workspace_indicator.set_shell_workspace_service(shell_workspace_service.clone());
    
    // The WorkspaceIndicatorWidget is now driven by the service.
    // Initial state is loaded when set_shell_workspace_service is called (which calls refresh_workspaces).
    // Clicks on workspace items trigger service calls and then refresh_workspaces.

    panel.present();
}
// Add Rc to imports
use std::rc::Rc;
