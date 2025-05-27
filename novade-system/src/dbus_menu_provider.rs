use gtk4_gio::gio; // Use gtk4_gio for gio::MenuModel
use async_trait::async_trait;
use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum DBusMenuError {
    #[error("D-Bus call failed: {0}")]
    CallFailed(String),
    #[error("App not found or no menu: {0}")]
    NotFound(String), // This might be better represented by Ok(None) in the Result
}

#[async_trait]
pub trait DBusMenuProvider: Send + Sync {
    async fn fetch_menu_model(&self, app_id: &str) -> Result<Option<gio::MenuModel>, DBusMenuError>;
}

// Helper functions to create stub menus
fn create_simple_file_menu() -> gio::MenuModel {
    let menu = gio::Menu::new();
    menu.append(Some("New"), Some("app.new")); // General app actions
    menu.append(Some("Open..."), Some("app.open"));
    menu.append(Some("Save"), Some("app.save"));
    menu.append_section(None, &{
        let section = gio::Menu::new();
        section.append(Some("Quit"), Some("app.quit"));
        section
    });
    menu.upcast()
}

fn create_editor_menu() -> gio::MenuModel {
    let menu = gio::Menu::new();
    
    let file_menu = gio::Menu::new();
    file_menu.append(Some("New Tab"), Some("app.new_tab")); // Specific to an editor context
    file_menu.append(Some("Open File..."), Some("app.open_file")); // More specific than general app.open
    menu.append_submenu(Some("File"), &file_menu);

    let edit_menu = gio::Menu::new();
    edit_menu.append(Some("Copy"), Some("app.copy"));
    edit_menu.append(Some("Paste"), Some("app.paste"));
    menu.append_submenu(Some("Edit"), &edit_menu);
    
    menu.upcast()
}

pub struct StubDBusMenuProvider {
    menus: HashMap<String, gio::MenuModel>,
}

impl StubDBusMenuProvider {
    pub fn new() -> Self {
        let mut menus = HashMap::new();
        // Using app_ids consistent with StubSystemWindowInfoProvider for testing
        menus.insert("org.gnome.TextEditor".to_string(), create_editor_menu());
        menus.insert("org.example.SimpleApp".to_string(), create_simple_file_menu()); // A new example
        // "org.mozilla.firefox" and "org.gnome.Console" will implicitly have no menu from this provider
        Self { menus }
    }
}

impl Default for StubDBusMenuProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DBusMenuProvider for StubDBusMenuProvider {
    async fn fetch_menu_model(&self, app_id: &str) -> Result<Option<gio::MenuModel>, DBusMenuError> {
        // Simulate an error case for a specific app_id if needed for testing error paths
        if app_id == "app.that.errors" {
            return Err(DBusMenuError::CallFailed("Simulated D-Bus error for app.that.errors".to_string()));
        }
        
        // Simulate delay
        // tokio::time::sleep(std::time::Duration::from_millis(50)).await; // Example delay

        Ok(self.menus.get(app_id).cloned()) // Cloned to return owned MenuModel
    }
}
