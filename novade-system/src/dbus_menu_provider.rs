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


// --- LiveDBusMenuProvider Implementation ---
use tracing; // Ensure tracing is imported

pub struct LiveDBusMenuProvider {
    // No specific fields needed if using gio::DBusMenuModel::new directly,
    // as it uses the default main context's D-Bus connection.
}

impl LiveDBusMenuProvider {
    pub fn new() -> Self {
        // Potentially initialize a shared D-Bus connection here if not relying on default.
        // For now, new() can be empty.
        Self {}
    }
}

#[async_trait]
impl DBusMenuProvider for LiveDBusMenuProvider {
    async fn fetch_menu_model(&self, app_id: &str) -> Result<Option<gio::MenuModel>, DBusMenuError> {
        // Standard object path for GApplication menus.
        let object_path = "/org/gtk/menus/menubar"; // Common path
        // Some applications might use /org/gtk/APPLICATION_ID/menus/menubar or similar if they have a unique path.
        // For example, for an app with ID "org.gnome.TextEditor", the path might be
        // "/org/gnome/TextEditor/menus/menubar" or simply "/org/gtk/menus/menubar"
        // if it's a simple GtkApplication.
        // Let's try the generic one first, then consider app_id specific if needed.
        // For a more robust solution, one might need to query or try multiple paths.

        tracing::info!("LiveDBusMenuProvider: Attempting to fetch D-Bus menu for app_id: '{}', object_path: '{}'", app_id, object_path);

        let app_id_owned = app_id.to_string();
        let object_path_owned = object_path.to_string();

        // gio::DBusMenuModel::new is synchronous and uses the thread-default main context.
        // To make this truly async and non-blocking if used from a tokio task,
        // it should be wrapped with `tokio::task::spawn_blocking`.
        match tokio::task::spawn_blocking(move || {
            // This synchronous call will run in a blocking thread
            gio::DBusMenuModel::new(&app_id_owned, &object_path_owned)
        }).await {
            Ok(menu_model) => {
                // Check if the menu model is actually valid by checking item count.
                // An empty menu (0 items) often indicates the service/path was not found
                // or the application doesn't export a menu there.
                if menu_model.n_items() > 0 {
                    tracing::info!("LiveDBusMenuProvider: Successfully fetched menu model for app_id: '{}' with {} items.", app_id, menu_model.n_items());
                    Ok(Some(menu_model))
                } else {
                    tracing::info!("LiveDBusMenuProvider: Fetched menu model for app_id: '{}' is empty (0 items). Assuming no menu or incorrect path.", app_id);
                    // Attempting a common alternative path structure for some apps.
                    // Example: /org/gnome/TextEditor/menubar (without /menus/)
                    // This is highly speculative and real applications might have different paths.
                    let alternative_object_path = format!("/org/{}/menubar", app_id.replace(".", "/"));
                    tracing::info!("LiveDBusMenuProvider: Trying alternative path: '{}'", alternative_object_path);
                    let app_id_clone_for_alt = app_id.to_string(); // Clone for the next spawn_blocking
                    match tokio::task::spawn_blocking(move || {
                         gio::DBusMenuModel::new(&app_id_clone_for_alt, &alternative_object_path)
                    }).await {
                        Ok(alt_menu_model) => {
                            if alt_menu_model.n_items() > 0 {
                                tracing::info!("LiveDBusMenuProvider: Successfully fetched menu model for app_id: '{}' with {} items using alternative path.", app_id, alt_menu_model.n_items());
                                Ok(Some(alt_menu_model))
                            } else {
                                tracing::info!("LiveDBusMenuProvider: Alternative path also yielded an empty menu for app_id: '{}'.", app_id);
                                Ok(None) // No menu found at either path
                            }
                        }
                        Err(e) => { // JoinError from spawn_blocking for alternative path
                            tracing::error!("LiveDBusMenuProvider: Spawn_blocking failed for alternative DBusMenuModel for app_id {}: {:?}", app_id, e);
                            Err(DBusMenuError::CallFailed(format!("Task join error for alternative path: {}", e)))
                        }
                    }
                }
            }
            Err(e) => { // JoinError from spawn_blocking for primary path
                tracing::error!("LiveDBusMenuProvider: Spawn_blocking failed for DBusMenuModel for app_id {}: {:?}", app_id, e);
                Err(DBusMenuError::CallFailed(format!("Task join error: {}", e)))
            }
        }
    }
}
