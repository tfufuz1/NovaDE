//! Configuration file watching and reloading logic.

use crate::config::loader::ConfigLoader;
use crate::error::CoreError;
use crate::utils::paths::{get_system_config_path_with_override, get_app_config_dir};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, event::EventKind};
use std::{
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use tracing::{debug, error, info, warn};

/// Holds the state for the configuration watcher.
struct WatcherState {
    system_config_path: PathBuf,
    user_config_path: PathBuf,
    last_processed_event_time: Option<std::time::Instant>,
}

impl WatcherState {
    const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

    fn new(system_config_path: PathBuf, user_config_path: PathBuf) -> Self {
        WatcherState {
            system_config_path,
            user_config_path,
            last_processed_event_time: None,
        }
    }

    /// Checks if an event for a given path should be processed based on debounce logic.
    fn should_process(&mut self, event_path: &PathBuf) -> bool {
        if event_path != &self.system_config_path && event_path != &self.user_config_path {
            // Not one of the files we are directly watching.
            // This could be a directory event or an event for a temporary file (e.g. by an editor).
            // We are interested if this event is for the PARENT directory of our config files,
            // as renaming/atomic replacement often involves parent dir notifications.
            if !(event_path == self.system_config_path.parent().unwrap_or(&self.system_config_path) ||
                 event_path == self.user_config_path.parent().unwrap_or(&self.user_config_path)) {
                debug!("Ignoring event for path not directly watched or parent of watched files: {:?}", event_path);
                return false;
            }
        }

        let now = std::time::Instant::now();
        if let Some(last_time) = self.last_processed_event_time {
            if now.duration_since(last_time) < Self::DEBOUNCE_DURATION {
                debug!("Debouncing event for path: {:?}", event_path);
                return false; // Debounce
            }
        }
        self.last_processed_event_time = Some(now);
        true
    }
}


/// Attempts to reload the configuration and logs the outcome.
///
/// This function is called when a change in the configuration files is detected.
/// It loads the new configuration using `ConfigLoader::load()`. If successful,
/// it updates the global `CORE_CONFIG` and logs success.
/// If loading or updating the global config fails, it logs the error.
pub fn reload_and_update_global_config() {
    info!("Configuration file change detected. Attempting to reload and update global config...");
    match ConfigLoader::load() {
        Ok(new_config) => {
            match crate::config::update_global_config(new_config.clone()) {
                Ok(_) => {
                    info!("Global configuration updated successfully.");
                    debug!("New global config: {:?}", new_config);
                    // Attempt to re-initialize logging with the new config
                    if let Err(e) = crate::logging::init_logging(&new_config.logging, true) {
                        error!("Failed to re-initialize logging with new config: {}", e);
                    } else {
                        info!("Logging re-initialized with new configuration.");
                    }
                }
                Err(e) => {
                    error!("Failed to acquire lock or update global config: {}", e);
                }
            }
        }
        Err(e) => {
            error!("Failed to reload configuration from files: {}", e);
        }
    }
}

/// Starts the configuration file watcher in a new thread.
///
/// Monitors the system and user configuration files for changes. When a change
/// is detected (write, remove, rename), it calls `reload_and_update_global_config`.
///
/// # Returns
///
/// * `Ok(RecommendedWatcher)` if the watcher started successfully. The caller should keep this watcher instance alive.
/// * `Err(CoreError)` if there was an error determining configuration paths or starting the watcher.
pub fn start_config_watcher() -> Result<RecommendedWatcher, CoreError> {
    let system_config_path = get_system_config_path_with_override().map_err(CoreError::Config)?;
    let user_config_dir = get_app_config_dir()?;
    let user_config_path = user_config_dir.join("config.toml");

    info!(
        "Starting configuration watcher for: {:?} and {:?}",
        system_config_path, user_config_path
    );

    let watcher_state = Arc::new(Mutex::new(WatcherState::new(
        system_config_path.clone(),
        user_config_path.clone(),
    )));

    // Create a channel to receive events from notify
    // Using mpsc channel as notify example, though direct callback could also work.
    let (tx, rx): (std::sync::mpsc::Sender<Result<notify::Event, notify::Error>>, Receiver<Result<notify::Event, notify::Error>>) = channel();


    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            tx.send(res).expect("Failed to send event to channel");
        },
        notify::Config::default(),
    )
    .map_err(|e| CoreError::Internal(format!("Failed to create file watcher: {}", e)))?;

    // Watch system config file's parent directory to catch renames/atomic writes.
    if let Some(parent) = system_config_path.parent() {
        if parent.exists() {
            watcher.watch(parent, RecursiveMode::NonRecursive)
                .map_err(|e| CoreError::Internal(format!("Failed to watch system config parent directory {:?}: {}", parent, e)))?;
            debug!("Watching system config parent directory: {:?}", parent);
        } else {
            warn!("System config parent directory {:?} does not exist. Direct file watching might be less reliable for atomic replacements.", parent);
            // Fallback to watching the file directly if parent doesn't exist, though this is unusual for /etc/
             if system_config_path.exists() { // only watch if file itself exists
                watcher.watch(&system_config_path, RecursiveMode::NonRecursive)
                    .map_err(|e| CoreError::Internal(format!("Failed to watch system config file {:?}: {}", system_config_path, e)))?;
                debug!("Watching system config file directly: {:?}", system_config_path);
            }
        }
    } else { // Should not happen for /etc/novade/config.toml
        warn!("System config path {:?} has no parent. Watching file directly.", system_config_path);
        if system_config_path.exists() {
            watcher.watch(&system_config_path, RecursiveMode::NonRecursive)
                .map_err(|e| CoreError::Internal(format!("Failed to watch system config file {:?}: {}", system_config_path, e)))?;
             debug!("Watching system config file directly: {:?}", system_config_path);
        }
    }


    // Watch user config file's parent directory.
    // User config dir should always exist or be creatable by get_app_config_dir.
    // If user_config_dir itself doesn't exist, watching it will fail.
    // Ensure it exists for watcher stability, though ConfigLoader::load handles its absence for loading.
    if !user_config_dir.exists() {
        crate::utils::fs::ensure_dir_exists(&user_config_dir)?;
        info!("Created user config directory for watcher: {:?}", user_config_dir);
    }
    watcher.watch(&user_config_dir, RecursiveMode::NonRecursive) // Watch the directory
        .map_err(|e| CoreError::Internal(format!("Failed to watch user config directory {:?}: {}", user_config_dir, e)))?;
    debug!("Watching user config directory: {:?}", user_config_dir);


    // Spawn a thread to handle events from the receiver
    thread::spawn(move || {
        loop {
            match rx.recv() {
                Ok(event_result) => match event_result {
                    Ok(event) => {
                        debug!("Received file event: {:?}", event);
                        let mut state = watcher_state.lock().unwrap();

                        // We are interested in writes, removes, or renames.
                        // For creates, it's often part of a write (e.g. atomic replace via temp file then rename).
                        // We primarily care about the *paths* being affected.
                        let relevant_event = match event.kind {
                            EventKind::Create(_) |
                            EventKind::Modify(_) |
                            EventKind::Remove(_) => true,
                            _ => false, // Other events like Access or Any are ignored for reload
                        };

                        if relevant_event {
                            // Check all paths in the event. If any match our config files or their parents (with debounce), trigger reload.
                            // Events can have multiple paths.
                            for path in event.paths {
                                if state.should_process(&path) {
                                    info!("Relevant change detected for path: {:?}, event kind: {:?}", path, event.kind);
                                    reload_and_update_global_config(); // Call the renamed function
                                    break; // Processed for this event, break from paths loop
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error receiving file event: {}", e);
                    }
                },
                Err(e) => {
                    error!("File watcher channel disconnected: {}. Stopping watch thread.", e);
                    break; // Exit thread if channel is broken
                }
            }
        }
    });

    info!("Configuration watcher thread started.");
    Ok(watcher)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CoreConfig, get_cloned_core_config, initialize_core_config}; // Import global config accessors
    use crate::utils::fs as nova_fs;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;
    use std::sync::mpsc;
    use std::time::Duration;

    // Helper to set up environment for watcher tests
    struct WatcherTestEnv {
        _system_dir_owner: TempDir,
        _user_dir_owner: TempDir,
        system_config_path: PathBuf,
        user_config_path: PathBuf,
        // To ensure env vars are cleaned up
        _env_guard_system: TestEnvVarGuard,
        _env_guard_user_config: TestEnvVarGuard,
        _env_guard_user_state: TestEnvVarGuard,
    }

    struct TestEnvVarGuard {
        name: String,
        original_value: Option<String>,
    }

    impl TestEnvVarGuard {
        fn new(name: &str, value: &str) -> Self {
            let original_value = std::env::var(name).ok();
            std::env::set_var(name, value);
            TestEnvVarGuard { name: name.to_string(), original_value }
        }
    }
    impl Drop for TestEnvVarGuard {
        fn drop(&mut self) {
            if let Some(val) = &self.original_value {
                std::env::set_var(&self.name, val);
            } else {
                std::env::remove_var(&self.name);
            }
        }
    }


    impl WatcherTestEnv {
        fn new() -> Self {
            let system_dir = TempDir::new().unwrap();
            let user_dir = TempDir::new().unwrap();
            let user_state_dir = TempDir::new().unwrap(); // For logs, if any default path is used by validation

            let system_config_path = system_dir.path().join("config.toml");
            // Simulate app specific path for user config. get_app_config_dir will construct NovaDE/NovaDE under this.
            let user_xdg_config_home = user_dir.path().join("user_xdg_config");
            nova_fs::ensure_dir_exists(&user_xdg_config_home).unwrap();
            let user_app_specific_config_dir = user_xdg_config_home.join("NovaDE/NovaDE");
            nova_fs::ensure_dir_exists(&user_app_specific_config_dir).unwrap();
            let user_config_path = user_app_specific_config_dir.join("config.toml");

            let guard_sys = TestEnvVarGuard::new("NOVADE_TEST_SYSTEM_CONFIG_PATH", system_config_path.to_str().unwrap());
            let guard_user_cfg = TestEnvVarGuard::new("XDG_CONFIG_HOME", user_xdg_config_home.to_str().unwrap());
            let guard_user_state = TestEnvVarGuard::new("XDG_STATE_HOME", user_state_dir.path().to_str().unwrap());

            // Create initial empty files so watcher can attach if needed by OS/backend
            // and so ConfigLoader::load doesn't use pure defaults initially if we want to test updates from empty.
            File::create(&system_config_path).unwrap_or_else(|e| panic!("Failed to create system_config_path {:?}: {}", system_config_path, e) );
            File::create(&user_config_path).unwrap_or_else(|e| panic!("Failed to create user_config_path {:?}: {}", user_config_path, e) );

            // Initialize global config to a known default state for tests
            initialize_core_config(CoreConfig::default()).unwrap();


            WatcherTestEnv {
                _system_dir_owner: system_dir,
                _user_dir_owner: user_dir,
                system_config_path,
                user_config_path,
                _env_guard_system: guard_sys,
                _env_guard_user_config: guard_user_cfg,
                _env_guard_user_state: guard_user_state,
            }
        }

        fn write_to_system_config(&self, content: &str) {
            let mut file = File::create(&self.system_config_path).unwrap();
            writeln!(file, "{}", content).unwrap();
            file.sync_all().unwrap();
            std::thread::sleep(Duration::from_millis(200)); // Give watcher time to pick up event
        }

        fn write_to_user_config(&self, content: &str) {
            let mut file = File::create(&self.user_config_path).unwrap();
            writeln!(file, "{}", content).unwrap();
            file.sync_all().unwrap();
            std::thread::sleep(Duration::from_millis(200)); // Give watcher time to pick up event
        }
    }


    #[test]
    fn test_watcher_setup_runs() {
        let _test_env = WatcherTestEnv::new();
        match crate::config::watcher::start_config_watcher() {
            Ok(_watcher) => {
                info!("Watcher setup test successful, watcher started.");
            }
            Err(e) => {
                panic!("start_config_watcher failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_watcher_state_debounce() {
        let mut state = WatcherState::new(PathBuf::from("/sys/config.toml"), PathBuf::from("/usr/config.toml"));
        let event_path = PathBuf::from("/usr/config.toml");

        assert!(state.should_process(&event_path), "First event should be processed");
        assert!(!state.should_process(&event_path), "Second immediate event should be debounced");

        state.last_processed_event_time = Some(std::time::Instant::now() - WatcherState::DEBOUNCE_DURATION - Duration::from_millis(100));
        assert!(state.should_process(&event_path), "Event after debounce period should be processed");
    }

    #[test]
    fn test_watcher_state_ignores_irrelevant_paths() {
        let mut state = WatcherState::new(PathBuf::from("/system/config.toml"), PathBuf::from("/user/app/config.toml"));
        let irrelevant_path = PathBuf::from("/other/file.txt");
        assert!(!state.should_process(&irrelevant_path), "Should ignore event for completely unrelated path");

        let relevant_parent_path = PathBuf::from("/user/app"); // Parent of /user/app/config.toml
         assert!(state.should_process(&relevant_parent_path), "Should process event for parent directory of a watched file");
    }

    // More robust integration test for file modification triggering reload
    #[test]
    fn test_file_modification_updates_global_config() {
        let test_env = WatcherTestEnv::new();

        // Initial config state (default)
        let initial_config = get_cloned_core_config();
        assert_eq!(initial_config.logging.level, "info"); // Default log level

        // Start watcher (keep the watcher instance alive for the test duration)
        let _watcher = start_config_watcher().expect("Failed to start watcher for integration test");

        // Modify user config file
        let new_log_level = "debug_via_watch";
        let user_config_content = format!("[logging]\nlevel = \"{}\"", new_log_level);
        test_env.write_to_user_config(&user_config_content);

        // Give watcher and reload mechanism time to operate.
        // This is the flaky part. Debounce is 1s, file system events can take time.
        // We wait a bit longer than debounce.
        std::thread::sleep(WatcherState::DEBOUNCE_DURATION + Duration::from_secs(1));

        let reloaded_config = get_cloned_core_config();
        assert_eq!(reloaded_config.logging.level, new_log_level,
            "Global config was not updated after user file change. Initial: {:?}, Reloaded: {:?}",
            initial_config, reloaded_config);

        // Test modifying system config
        let new_theme_name = "system_theme_via_watch";
        let system_config_content = format!("[compositor.visual]\ntheme_name = \"{}\"", new_theme_name);
        test_env.write_to_system_config(&system_config_content);
        std::thread::sleep(WatcherState::DEBOUNCE_DURATION + Duration::from_secs(1));

        let reloaded_config_after_system_change = get_cloned_core_config();
        assert_eq!(reloaded_config_after_system_change.compositor.visual.theme_name, new_theme_name.to_lowercase(),
            "Global config was not updated after system file change. Previous: {:?}, Reloaded: {:?}",
            reloaded_config, reloaded_config_after_system_change);
        // Check that previous user change is still there (or merged appropriately if system also had logging)
        assert_eq!(reloaded_config_after_system_change.logging.level, new_log_level,
            "User config change (log level) was lost after system config update.");

    }
}
