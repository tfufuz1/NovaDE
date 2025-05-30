use zbus::{Connection, Proxy, zvariant::Value};
use std::collections::HashMap;
use tracing; // For logging

pub async fn send_desktop_notification(
    app_name: &str,
    summary: &str,
    body: &str,
    icon: &str,
    timeout_ms: i32,
) -> Result<u32, zbus::Error> {
    tracing::info!(
        "Attempting to send notification: app='{}', summary='{}', body='{}', icon='{}', timeout={}ms",
        app_name, summary, body, icon, timeout_ms
    );

    let connection = Connection::session().await?;
    tracing::debug!("Successfully connected to D-Bus session bus.");

    let proxy = Proxy::new(
        &connection,
        "org.freedesktop.Notifications",      // service_name
        "/org/freedesktop/Notifications",      // object_path
        "org.freedesktop.Notifications",      // interface_name
    )
    .await?;
    tracing::debug!("Notification proxy created successfully.");

    // Prepare arguments for the Notify method
    // Signature: (String app_name, u32 replaces_id, String app_icon, String summary, String body, Vec<String> actions, HashMap<String, Variant> hints, i32 expire_timeout) -> u32 notification_id
    let replaces_id: u32 = 0; // 0 means it's a new notification
    let actions: Vec<String> = Vec::new(); // No actions for this simple notification
    let mut hints: HashMap<String, Value> = HashMap::new();
    // Example hint: urgency level (0=low, 1=normal, 2=critical)
    // hints.insert("urgency".to_string(), Value::new(1u8)); // u8 for byte
    // Example hint: transience
    // hints.insert("transient".to_string(), Value::new(true));


    tracing::debug!("Calling Notify method on proxy...");
    let result: (u32,) = proxy.call(
        "Notify",
        &(app_name, replaces_id, icon, summary, body, actions, hints, timeout_ms)
    ).await?;
    
    let notification_id = result.0;
    tracing::info!("Successfully sent notification with ID: {}", notification_id);

    Ok(notification_id)
}

// --- XDG Desktop Portal Functions ---

use ashpd::desktop::file_chooser::{OpenFileRequest, SelectedFiles};
use ashpd::WindowIdentifier; // To potentially pass a parent window

/// Opens a file chooser dialog using the XDG Desktop Portal.
/// Returns Ok(Some(paths)) if files were selected, Ok(None) if cancelled, Err on error.
pub async fn open_file_chooser(parent_window: Option<&impl gtk::prelude::IsA<gtk::Window>>) -> Result<Option<Vec<std::path::PathBuf>>, ashpd::Error> {
    tracing::info!("Attempting to open file chooser dialog via XDG portal.");

    let mut request_builder = OpenFileRequest::default();
    
    // Set parent window if provided, for better dialog behavior
    if let Some(window) = parent_window {
        let identifier = WindowIdentifier::from_native(window).await;
        request_builder = request_builder.parent_window(&identifier);
        tracing::debug!("Parent window identifier set for file chooser: {:?}", identifier);
    } else {
        tracing::debug!("No parent window provided for file chooser.");
    }

    // Example: Set a title for the dialog
    request_builder = request_builder.title("Select a File (NovaDE UI)");
    // Example: Allow multiple file selection
    // request_builder = request_builder.multiple(true);
    // Example: Add a filter
    // request_builder = request_builder.add_filter(ashpd::desktop::file_chooser::FileFilter::new("Images").add_mime_type("image/*"));

    let selected_files_response = request_builder.build().await;

    match selected_files_response {
        Ok(response) => {
            // The response contains a `SelectedFiles` enum or similar structure.
            // Let's assume `response.uris()` gives Vec<url::Url> or similar.
            // For ashpd 0.4, response is directly SelectedFiles.
            // If it's `SelectedFiles::Uris(uris)`
            let uris = response.uris(); // This gives Vec<zbus::zvariant::OwnedObjectPath> in ashpd 0.2
                                   // In ashpd 0.4, it's likely `Vec<url::Url>` if using `OpenFileRequest::send().await.uris()`
                                   // Or directly Vec<PathBuf> via `response.paths()` if available in SelectedFiles
                                   // Let's check ashpd docs for the exact type from `OpenFileRequest.build().await?`
                                   // For ashpd 0.4 OpenFileRequest::build().await returns a SelectedFiles directly.
                                   // SelectedFiles has a .paths() method.

            if uris.is_empty() {
                tracing::info!("File chooser dialog cancelled by user (no files selected).");
                Ok(None) // User cancelled or selected nothing
            } else {
                // Convert zbus::zvariant::ObjectPath (URI-like) to PathBuf
                // For ashpd 0.4, response.paths() should give Vec<PathBuf>
                // If response.uris() gives url::Url, convert them:
                // let paths: Vec<std::path::PathBuf> = uris.iter().filter_map(|uri| uri.to_file_path().ok()).collect();
                
                // Assuming response.uris() gives something convertible or `response.paths()` exists.
                // With ashpd 0.4, `response.paths()` directly gives `Vec<PathBuf>`.
                let paths = response.paths()?; // This will be Vec<PathBuf>
                
                if paths.is_empty() { // Should not happen if uris was not empty, but good check
                    tracing::info!("File chooser dialog cancelled by user (paths list empty).");
                    Ok(None)
                } else {
                    tracing::info!("Files selected: {:?}", paths);
                    Ok(Some(paths))
                }
            }
        }
        Err(ashpd::Error::Cancelled) => {
            tracing::info!("File chooser dialog cancelled by user.");
            Ok(None)
        }
        Err(e) => {
            tracing::error!("Error opening file chooser dialog: {}", e);
            Err(e)
        }
    }
}
