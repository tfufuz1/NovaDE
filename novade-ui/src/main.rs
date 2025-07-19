use gtk4 as gtk;
use gtk::{prelude::*, Application, ApplicationWindow};
use libadwaita as adw;
use adw::prelude::*;
use std::sync::Arc;

use novade_core::config::CoreConfig;
use novade_domain::theming::{ThemingEngine, types::ThemingConfiguration};
use novade_core::errors::CoreError;
use novade_core::config::ConfigServiceAsync;
use crate::theming_gtk::GtkThemeManager;
use async_trait::async_trait;
use std::path::PathBuf;

const APP_ID: &str = "org.novade.Shell";

#[derive(Clone)]
struct SimpleFileConfigService;

#[async_trait]
impl ConfigServiceAsync for SimpleFileConfigService {
    async fn read_file_to_string(&self, path: &std::path::Path) -> Result<String, CoreError> {
        tokio::fs::read_to_string(path)
            .await
            .map_err(|e| CoreError::IoError(format!("Failed to read {:?}", path), Some(e)))
    }

    async fn list_files_in_dir(&self, _path: &std::path::Path, _extensions: &[String]) -> Result<Vec<PathBuf>, CoreError> {
        unimplemented!("SimpleFileConfigService::list_files_in_dir not implemented")
    }
    async fn file_exists(&self, _path: &std::path::Path) -> bool {
        unimplemented!("SimpleFileConfigService::file_exists not implemented")
    }
    async fn dir_exists(&self, _path: &std::path::Path) -> bool {
        unimplemented!("SimpleFileConfigService::dir_exists not implemented")
    }
     async fn ensure_dir_exists(&self, _path: &std::path::Path) -> Result<(), CoreError> {
        unimplemented!("SimpleFileConfigService::ensure_dir_exists not implemented")
    }
}

fn main() {
    adw::init().expect("Failed to initialize Adwaita.");
    tracing_subscriber::fmt::init();

    let app = Application::builder().application_id(APP_ID).build();

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::widgets::WindowDecoration;
use futures_util::stream::StreamExt;

    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("NovaDE")
            .default_width(1280)
            .default_height(720)
            .build();

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window.set_child(Some(&content));

        let decorations = Arc::new(Mutex::new(HashMap::<String, WindowDecoration>::new()));

        let content_clone = content.clone();
        let decorations_clone = decorations.clone();

        // Spawn a new thread to listen for D-Bus signals
        tokio::spawn(async move {
            if let Err(e) = async {
                let connection = zbus::Connection::session().await?;
                let proxy = novade_system::dbus_interfaces::window_management::WindowManagerProxy::new(&connection).await?;
                let mut decoration_mode_changed = proxy.receive_decoration_mode_changed().await?;

                while let Some(signal) = decoration_mode_changed.next().await {
                    let args = signal.args()?;
                    let window_id: String = args.get::<String>(0)?.clone();
                    let mode: String = args.get::<String>(1)?.clone();

                    let mut decorations = decorations_clone.lock().unwrap();
                    if mode == "ServerSide" {
                        if !decorations.contains_key(&window_id) {
                            let decoration = WindowDecoration::new();
                            content_clone.pack_start(&decoration, false, false, 0);
                            decorations.insert(window_id, decoration);
                        }
                    } else {
                        if let Some(decoration) = decorations.remove(&window_id) {
                            content_clone.remove(&decoration);
                        }
                    }
                }
                Ok::<(), anyhow::Error>(())
            }
            .await
            {
                tracing::error!("D-Bus error: {}", e);
            }
        });

        window.present();
    });

    std::process::exit(app.run_with_args::<&str>(&[]));
}
