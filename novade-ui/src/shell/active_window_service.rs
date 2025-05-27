use std::cell::Cell;
use std::sync::Arc; // For Arc<dyn SystemWindowInfoProvider>
use gtk::gio;
use gtk::glib; // Not strictly needed for VariantTy here, but good for context
use novade_system::window_info_provider::SystemWindowInfoProvider; // Import the trait

#[derive(Clone, Debug)]
pub struct ActiveWindowData {
    pub app_id: Option<String>,
    pub window_title: Option<String>, // This will be primarily from SystemWindowInfoProvider
    pub icon_name: Option<String>,
    pub menu_model: Option<gio::MenuModel>,
}

// Helper functions to create stub menus (create_stub_menu1, create_stub_menu2)
// remain unchanged from your previous correct implementation.
// ... (keep create_stub_menu1 and create_stub_menu2 as they are)
// Helper function to create a stub menu
fn create_stub_menu1() -> gio::MenuModel {
    let menu = gio::Menu::new();

    // File Section
    let file_section = gio::Menu::new();
    file_section.append(Some("Open File"), Some("app.open"));
    file_section.append(Some("Save As..."), Some("app.save_as")); 
    file_section.append(Some("Quit Application"), Some("app.quit"));
    menu.append_section(Some("File"), &file_section); 

    // Edit Section
    let edit_section = gio::Menu::new();
    edit_section.append(Some("Copy Selection"), Some("app.copy"));
    edit_section.append(Some("Paste from Clipboard"), Some("app.paste"));
    menu.append_section(Some("Edit"), &edit_section);

    // Help Section with a single item
    let help_menu = gio::Menu::new();
    help_menu.append_item(&gio::MenuItem::new(Some("About App"), Some("app.about")));
    menu.append_submenu(Some("Help"), &help_menu); 

    menu.upcast::<gio::MenuModel>()
}

fn create_stub_menu2() -> gio::MenuModel {
    let menu = gio::Menu::new();
    menu.append(Some("Preferences"), Some("app.preferences"));
    menu.append(Some("Show Details"), Some("app.details"));
    menu.upcast::<gio::MenuModel>()
}


pub struct ActiveWindowService {
    system_info_provider: Arc<dyn SystemWindowInfoProvider>,
    counter: Cell<usize>, // Internal counter to cycle through predefined app_id, icon, menu
    // This data now primarily provides app_id, icon_name, and menu_model.
    // window_title in this vec can serve as a fallback if system_info_provider returns None.
    predefined_app_data: Vec<ActiveWindowData>,
}

impl ActiveWindowService {
    pub fn new(system_info_provider: Arc<dyn SystemWindowInfoProvider>) -> Self {
        let predefined_app_data = vec![
            // Title here is a fallback or for context, actual title comes from system_info_provider
            ActiveWindowData {
                app_id: Some("org.gnome.TextEditor".to_string()),
                window_title: Some("Fallback Text Editor Title".to_string()), 
                icon_name: Some("accessories-text-editor-symbolic".to_string()),
                menu_model: Some(create_stub_menu1()),
            },
            ActiveWindowData {
                app_id: Some("org.mozilla.firefox".to_string()),
                window_title: Some("Fallback Firefox Title".to_string()),
                icon_name: Some("web-browser-symbolic".to_string()),
                menu_model: None,
            },
            ActiveWindowData {
                app_id: None, 
                window_title: Some("Desktop".to_string()), // Fallback for desktop
                icon_name: Some("user-desktop-symbolic".to_string()),
                menu_model: None,
            },
            ActiveWindowData { 
                app_id: Some("org.gnome.Terminal".to_string()),
                window_title: Some("Fallback Terminal Title".to_string()),
                icon_name: Some("utilities-terminal-symbolic".to_string()),
                menu_model: Some(create_stub_menu2()),
            },
        ];
        Self {
            system_info_provider,
            counter: Cell::new(0),
            predefined_app_data,
        }
    }

    pub fn get_current_active_window(&self) -> ActiveWindowData {
        let current_stub_index = self.counter.get();
        self.counter.set((current_stub_index + 1) % self.predefined_app_data.len());
        let base_data = &self.predefined_app_data[current_stub_index];

        let focused_title = self.system_info_provider.get_focused_window_title();

        ActiveWindowData {
            app_id: base_data.app_id.clone(),
            // Use system title primarily, fallback to predefined if system returns None
            window_title: focused_title.or_else(|| base_data.window_title.clone()), 
            icon_name: base_data.icon_name.clone(),
            menu_model: base_data.menu_model.clone(), // Clone the Option<MenuModel>
        }
    }
}

// Default cannot be easily implemented without a default SystemWindowInfoProvider.
// If needed, it would require a concrete default provider or making system_info_provider an Option.
// For this exercise, we assume ActiveWindowService is always constructed with a provider.
// impl Default for ActiveWindowService {
//     fn default() -> Self {
//         // This would require a default SystemWindowInfoProvider instance
//         // For example:
//         // let default_provider = Arc::new(novade_system::StubSystemWindowInfoProvider::new());
//         // Self::new(default_provider)
//         panic!("ActiveWindowService cannot be defaulted without a SystemWindowInfoProvider");
//     }
// }
