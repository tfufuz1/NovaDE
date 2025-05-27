use gtk::prelude::*;
use gtk::{Application, ApplicationWindow}; // ApplicationWindow might not be directly used for PanelWidget
use novade_ui::shell::panel_widget::{PanelWidget, PanelPosition, ModulePosition};
use novade_ui::shell::panel_widget::app_menu_button::AppMenuButton;
use novade_ui::shell::panel_widget::workspace_indicator_widget::WorkspaceIndicatorWidget;
use novade_ui::shell::panel_widget::clock_datetime_widget::ClockDateTimeWidget;

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
    let app_menu_button = AppMenuButton::new();
    let workspace_indicator = WorkspaceIndicatorWidget::new();
    let clock_widget = ClockDateTimeWidget::new();

    // Add modules to the panel
    panel.add_module(&app_menu_button, ModulePosition::Start, 0);
    panel.add_module(&workspace_indicator, ModulePosition::Center, 0);
    panel.add_module(&clock_widget, ModulePosition::End, 0);
    
    // Test properties
    // panel.set_position(PanelPosition::Bottom);
    // panel.set_panel_height(60);
    // panel.set_transparency_enabled(true); // Depends on compositor and theme

    panel.present();
}
