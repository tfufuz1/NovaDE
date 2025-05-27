use std::cell::Cell; // For the counter

use gtk::gio;
use gtk::glib; // For Variant, though not directly used in struct, good for context

#[derive(Clone, Debug)]
pub struct ActiveWindowData {
    pub app_id: Option<String>,
    pub window_title: Option<String>,
    pub icon_name: Option<String>,
    pub menu_model: Option<gio::MenuModel>, // Added menu_model
}

// Helper function to create a stub menu
fn create_stub_menu1() -> gio::MenuModel {
    let menu = gio::Menu::new();

    // File Section
    let file_section = gio::Menu::new();
    file_section.append(Some("Open File"), Some("app.open"));
    file_section.append(Some("Save As..."), Some("app.save_as")); // Example new item
    file_section.append(Some("Quit Application"), Some("app.quit"));
    menu.append_section(Some("File"), &file_section); // Added section label

    // Edit Section
    let edit_section = gio::Menu::new();
    edit_section.append(Some("Copy Selection"), Some("app.copy"));
    edit_section.append(Some("Paste from Clipboard"), Some("app.paste"));
    menu.append_section(Some("Edit"), &edit_section);

    // Help Section with a single item
    let help_menu = gio::Menu::new();
    help_menu.append_item(&gio::MenuItem::new(Some("About App"), Some("app.about")));
    menu.append_submenu(Some("Help"), &help_menu); // Added as submenu for structure

    menu.upcast::<gio::MenuModel>()
}

fn create_stub_menu2() -> gio::MenuModel {
    let menu = gio::Menu::new();
    menu.append(Some("Preferences"), Some("app.preferences"));
    menu.append(Some("Show Details"), Some("app.details"));
    menu.upcast::<gio::MenuModel>()
}


pub struct ActiveWindowService {
    counter: Cell<usize>, // Internal counter to cycle through data
    predefined_windows: Vec<ActiveWindowData>,
}

impl ActiveWindowService {
    pub fn new() -> Self {
        let predefined_windows = vec![
            ActiveWindowData {
                app_id: Some("org.gnome.TextEditor".to_string()),
                window_title: Some("Document 1 - TextEditor".to_string()), // Slightly changed title
                icon_name: Some("accessories-text-editor-symbolic".to_string()),
                menu_model: Some(create_stub_menu1()), // Added menu model
            },
            ActiveWindowData {
                app_id: Some("org.mozilla.firefox".to_string()),
                window_title: Some("Mozilla Firefox".to_string()),
                icon_name: Some("web-browser-symbolic".to_string()),
                menu_model: None, // No menu for Firefox in this stub
            },
            ActiveWindowData {
                app_id: None, 
                window_title: Some("Desktop Focus".to_string()), // Changed title
                icon_name: Some("user-desktop-symbolic".to_string()),
                menu_model: None, // No menu for desktop
            },
            ActiveWindowData { 
                app_id: Some("org.gnome.Terminal".to_string()),
                window_title: Some("Terminal".to_string()),
                icon_name: Some("utilities-terminal-symbolic".to_string()),
                menu_model: Some(create_stub_menu2()), // Added another menu
            },
        ];
        Self {
            counter: Cell::new(0),
            predefined_windows,
        }
    }

    pub fn get_current_active_window(&self) -> ActiveWindowData {
        let current_index = self.counter.get();
        let data = self.predefined_windows[current_index].clone();
        self.counter.set((current_index + 1) % self.predefined_windows.len());
        data
    }
}

impl Default for ActiveWindowService {
    fn default() -> Self {
        Self::new()
    }
}
