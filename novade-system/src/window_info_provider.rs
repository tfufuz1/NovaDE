use async_trait::async_trait;
use std::cell::Cell;
use std::sync::{Arc, Mutex}; // Mutex for shared data if Wayland thread worked

// Wayland specific imports
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_output, wl_registry, wl_shm, wl_compositor},
};
// Attempt to import the foreign toplevel manager protocol
// This might require a specific crate or feature if not in wayland-protocols directly under this name
use wayland_protocols::wlr::unstable::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::{self, ZwlrForeignToplevelHandleV1},
    zwlr_foreign_toplevel_manager_v1::{self, ZwlrForeignToplevelManagerV1},
};
use tracing;

#[derive(Clone, Debug, Default)]
pub struct FocusedWindowDetails {
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub icon_name: Option<String>,
}

#[async_trait]
pub trait SystemWindowInfoProvider: Send + Sync {
    fn get_focused_window_details(&self) -> FocusedWindowDetails;
    async fn get_windows(&self) -> Vec<FocusedWindowDetails>;
}

// --- StubSystemWindowInfoProvider (remains unchanged from previous task) ---
pub struct StubSystemWindowInfoProvider {
    counter: Cell<usize>,
    details: Vec<FocusedWindowDetails>,
}

impl StubSystemWindowInfoProvider {
    pub fn new() -> Self {
        Self {
            counter: Cell::new(0),
            details: vec![
                FocusedWindowDetails {
                    title: Some("Text Editor - Document1 (Stub)".to_string()),
                    app_id: Some("org.gnome.TextEditor".to_string()),
                    icon_name: Some("accessories-text-editor-symbolic".to_string()),
                },
                FocusedWindowDetails {
                    title: Some("Firefox - NovaDE Docs (Stub)".to_string()),
                    app_id: Some("org.mozilla.firefox".to_string()),
                    icon_name: Some("web-browser-symbolic".to_string()),
                },
                FocusedWindowDetails {
                    title: None, 
                    app_id: None,
                    icon_name: Some("user-desktop-symbolic".to_string()),
                },
                FocusedWindowDetails {
                    title: Some("Terminal (Stub)".to_string()),
                    app_id: Some("org.gnome.Console".to_string()),
                    icon_name: Some("utilities-terminal-symbolic".to_string()),
                },
            ],
        }
    }
}

impl Default for StubSystemWindowInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemWindowInfoProvider for StubSystemWindowInfoProvider {
    fn get_focused_window_details(&self) -> FocusedWindowDetails {
        let index = self.counter.get();
        let details_to_return = self.details[index].clone();
        self.counter.set((index + 1) % self.details.len());
        details_to_return
    }

    async fn get_windows(&self) -> Vec<FocusedWindowDetails> {
        self.details.clone()
    }
}

// --- WaylandWindowInfoProvider Implementation ---

struct WaylandState {
    // We'd store Wayland objects like the toplevel manager here if event loop was running
    toplevel_manager: Option<ZwlrForeignToplevelManagerV1>,
    // For now, just a flag if manager was bound
    toplevel_manager_bound: bool,
    // If we were fully implementing, we'd have a list of toplevels and the current focused one.
    // current_focused_title: Arc<Mutex<Option<String>>>,
    // current_focused_app_id: Arc<Mutex<Option<String>>>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                "zwlr_foreign_toplevel_manager_v1" => {
                    tracing::info!("Wayland: Found zwlr_foreign_toplevel_manager_v1 (version {})", version);
                    let manager = registry.bind::<ZwlrForeignToplevelManagerV1, _, _>(name, version, qh, ());
                    state.toplevel_manager = Some(manager.clone()); // Clone if needed elsewhere or just use it
                    state.toplevel_manager_bound = true;
                    
                    // Setup listener for toplevel events from the manager
                    manager.quick_assign(move |manager_obj, event, _dispatch_data| {
                        match event {
                            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                                tracing::info!("Wayland: New toplevel: {:?}", toplevel);
                                // Here we would assign a handler to the `toplevel` handle
                                // to listen for its title, app_id, state changes.
                                // For this fallback, we don't fully implement this part.
                                // Example: toplevel.quick_assign(handle_toplevel_event);
                            }
                            zwlr_foreign_toplevel_manager_v1::Event::Finished => {
                                tracing::info!("Wayland: Toplevel manager finished.");
                                // state.toplevel_manager = None; // Or handle re-binding
                            }
                            _ => {}
                        }
                    });

                }
                "wl_compositor" => {
                    registry.bind::<wl_compositor::WlCompositor, _, _>(name, version, qh, ());
                    tracing::debug!("Wayland: Bound wl_compositor (version {})", version);
                }
                "wl_shm" => {
                    registry.bind::<wl_shm::WlShm, _, _>(name, version, qh, ());
                    tracing::debug!("Wayland: Bound wl_shm (version {})", version);
                }
                _ => {}
            }
        }
    }
}

// Dummy dispatch for other objects if needed for completeness in a real app
impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for WaylandState {
    fn event(_: &mut Self, _: &ZwlrForeignToplevelManagerV1, _: zwlr_foreign_toplevel_manager_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        // Handled by quick_assign above
    }
}
// Need dummy dispatch for wl_compositor and wl_shm too if not using quick_assign for them
impl Dispatch<wl_compositor::WlCompositor, ()> for WaylandState {fn event(_: &mut Self,_: &wl_compositor::WlCompositor, _: wl_compositor::Event,_: &(),_: &Connection,_: &QueueHandle<Self>,) {}}
impl Dispatch<wl_shm::WlShm, ()> for WaylandState {fn event(_: &mut Self,_: &wl_shm::WlShm, _: wl_shm::Event,_: &(),_: &Connection,_: &QueueHandle<Self>,) {}}


pub struct WaylandWindowInfoProvider {
    // For this fallback, we won't run a separate event loop thread.
    // We'll try to connect and bind, then use stubbed cycling data.
    // If a real implementation was done, conn, queue, and shared state (Arc<Mutex<...>>) would be here.
    // conn: Option<Connection>,
    // queue_handle: Option<QueueHandle<WaylandState>>,
    connection_status: String, // To store status of connection attempt
    toplevel_manager_was_bound: bool,
    
    // Fallback cycling data for this provider
    fallback_counter: Cell<usize>,
    fallback_details: Vec<FocusedWindowDetails>,
}

impl WaylandWindowInfoProvider {
    pub fn new() -> Result<Self, String> {
        tracing::info!("Attempting to initialize WaylandWindowInfoProvider...");
        let conn = match Connection::connect_to_env() {
            Ok(c) => c,
            Err(e) => {
                let err_msg = format!("Failed to connect to Wayland display: {:?}", e);
                tracing::error!("{}", err_msg);
                return Err(err_msg);
            }
        };
        tracing::info!("Wayland connection established.");

        let mut event_queue = conn.new_event_queue();
        let qh = event_queue.handle();

        let display = conn.display();
        display.get_registry(&qh, ()); // Attach no data

        let mut wayland_state = WaylandState {
            toplevel_manager: None,
            toplevel_manager_bound: false,
            // current_focused_title: Arc::new(Mutex::new(None)), // Example if using shared state
            // current_focused_app_id: Arc::new(Mutex::new(None)),
        };

        // Perform a few rounds of dispatch to catch globals.
        // In a real app, this would be a loop in a dedicated thread.
        // For this stub, we just do a couple of roundtrips.
        if event_queue.roundtrip(&mut wayland_state).is_err() {
            let err_msg = "Wayland roundtrip failed during initial setup.".to_string();
            tracing::error!("{}", err_msg);
            // Proceeding to see if manager was bound, but this is likely a failure.
        }
         if event_queue.roundtrip(&mut wayland_state).is_err() { // Second roundtrip
            tracing::warn!("Second Wayland roundtrip failed during initial setup.");
        }


        let connection_status_msg: String;
        if wayland_state.toplevel_manager_bound {
            connection_status_msg = "Wayland connected, toplevel manager bound. Title extraction stubbed.".to_string();
            tracing::info!("{}", connection_status_msg);
        } else if wayland_state.toplevel_manager.is_some() { // Bound but quick_assign might not have run if roundtrip issue
             connection_status_msg = "Wayland connected, toplevel manager INTERNALLY bound but may not be fully active. Title extraction stubbed.".to_string();
            tracing::warn!("{}", connection_status_msg);
        }
        else {
            connection_status_msg = "Wayland connected, but zwlr_foreign_toplevel_manager_v1 NOT bound. Title extraction stubbed.".to_string();
            tracing::warn!("{}", connection_status_msg);
        }
        
        // Not spawning a thread for the event loop in this fallback version.
        // The state captured in `wayland_state` after roundtrips is what we have.

        Ok(Self {
            connection_status: connection_status_msg,
            toplevel_manager_was_bound: wayland_state.toplevel_manager_bound,
            fallback_counter: Cell::new(0),
            fallback_details: vec![
                FocusedWindowDetails {
                    title: Some("WaylandProvider: Firefox".to_string()),
                    app_id: Some("wayland.firefox".to_string()),
                    icon_name: Some("web-browser-symbolic".to_string()),
                },
                FocusedWindowDetails {
                    title: Some("WaylandProvider: Kitty Terminal".to_string()),
                    app_id: Some("wayland.kitty".to_string()),
                    icon_name: Some("utilities-terminal-symbolic".to_string()),
                },
                FocusedWindowDetails {
                    title: None, // Simulate no specific window
                    app_id: Some("wayland.desktop".to_string()),
                    icon_name: Some("user-desktop-symbolic".to_string()),
                },
            ],
        })
    }
}

#[async_trait]
impl SystemWindowInfoProvider for WaylandWindowInfoProvider {
    fn get_focused_window_details(&self) -> FocusedWindowDetails {
        if !self.toplevel_manager_was_bound {
            // If manager wasn't bound, return the connection status as title
            return FocusedWindowDetails {
                title: Some(format!("Wayland Init: {}", self.connection_status)),
                app_id: None,
                icon_name: None,
            };
        }
        
        // If manager was bound, but we are in fallback mode (no real title extraction from events)
        // log it and return cycling stub data.
        tracing::info!("WaylandWindowInfoProvider: get_focused_window_details - {}", self.connection_status);
        
        let index = self.fallback_counter.get();
        let details_to_return = self.fallback_details[index].clone();
        self.fallback_counter.set((index + 1) % self.fallback_details.len());
        details_to_return
    }

    async fn get_windows(&self) -> Vec<FocusedWindowDetails> {
        if !self.toplevel_manager_was_bound {
            return vec![FocusedWindowDetails {
                title: Some(format!("Wayland Init: {}", self.connection_status)),
                app_id: None,
                icon_name: None,
            }];
        }
        self.fallback_details.clone()
    }
}
