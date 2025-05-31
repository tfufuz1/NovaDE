use anyhow::Context;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};
use wayland_client::{
    globals::{BindError, GlobalList, GlobalListContents},
    protocol::{wl_output, wl_registry, wl_seat, wl_shm, wl_compositor}, // Added wl_compositor
    Connection, Dispatch, EventQueue, QueueHandle,
};
use wayland_protocols::wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1, zwlr_foreign_toplevel_manager_v1,
};

#[derive(Debug, Clone)]
pub struct ToplevelInfo {
    pub wayland_id: u32,
    pub title: Option<String>,
    pub app_id: Option<String>,
}

pub struct WaylandState {
    pub toplevel_manager: Option<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1>,
    pub toplevels_data: Arc<Mutex<Vec<ToplevelInfo>>>,
    _queue_handle: QueueHandle<WaylandState>, // Store if needed, mark as unused for now if not directly used by WaylandState methods
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _global_data: &GlobalListContents,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            if interface == zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1::interface().name {
                tracing::info!(
                    "Found zwlr_foreign_toplevel_manager_v1 global (name: {}, version: {})",
                    name, version
                );
                let manager_version = std::cmp::min(version, 3);
                match registry.bind::<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, _, _>(
                    name,
                    manager_version,
                    qh,
                    (),
                ) {
                    Ok(manager) => {
                        tracing::info!("Successfully bound to zwlr_foreign_toplevel_manager_v1 (version {}).", manager_version);
                        state.toplevel_manager = Some(manager.clone());
                        manager.quick_assign({
                            let _toplevels_data_clone = state.toplevels_data.clone(); // Not used in this minimal log
                            let _qh_clone = qh.clone();

                            move |_mgr_proxy, event, _dispatch_data| {
                                match event {
                                    zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                                        tracing::info!("New toplevel advertised by manager: id={:?}", toplevel.id());
                                        // In a later step, we'll create ToplevelInfo and assign a handler for 'toplevel'
                                    }
                                    zwlr_foreign_toplevel_manager_v1::Event::Finished => {
                                        tracing::info!("Foreign toplevel manager finished sending initial list of toplevels.");
                                    }
                                    _ => {}
                                }
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to bind zwlr_foreign_toplevel_manager_v1: {:?}", e);
                    }
                }
            }
        }
    }
}

// Minimal Dispatch implementations for core globals if bound by registry_queue_init.
// These are needed for registry_queue_init to not panic if these globals are present.
impl Dispatch<wl_compositor::WlCompositor, ()> for WaylandState {fn event(_s: &mut Self, _i: &wl_compositor::WlCompositor, _e: wl_compositor::Event, _d: &(), _c: &Connection, _q: &QueueHandle<Self>) {}}
impl Dispatch<wl_shm::WlShm, ()> for WaylandState {fn event(_s: &mut Self, _i: &wl_shm::WlShm, _e: wl_shm::Event, _d: &(), _c: &Connection, _q: &QueueHandle<Self>) {}}
impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {fn event(_s: &mut Self, _i: &wl_seat::WlSeat, _e: wl_seat::Event, _d: &(), _c: &Connection, _q: &QueueHandle<Self>) {}}
impl Dispatch<wl_output::WlOutput, ()> for WaylandState {fn event(_s: &mut Self, _i: &wl_output::WlOutput, _e: wl_output::Event, _d: &(), _c: &Connection, _q: &QueueHandle<Self>) {}}

// Dispatch for ZwlrForeignToplevelManagerV1 (not used if quick_assign is used for the manager)
impl Dispatch<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, ()> for WaylandState {
    fn event( _state: &mut Self, _manager: &zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, _event: zwlr_foreign_toplevel_manager_v1::Event, _data: &(), _conn: &Connection, _qh: &QueueHandle<Self>) { }
}

// Dispatch for ZwlrForeignToplevelHandleV1 (not used in this subtask, but needed for completeness if handles were assigned full dispatchers)
impl Dispatch<zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, u32> for WaylandState {
    fn event( _state: &mut Self, _handle: &zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, _event: zwlr_foreign_toplevel_handle_v1::Event, _toplevel_protocol_id: &u32, _conn: &Connection, _qh: &QueueHandle<Self>) { }
}

pub struct WaylandToplevelIntegration {
    pub toplevels_data_access: Arc<Mutex<Vec<ToplevelInfo>>>,
    // Store the join handle to allow the main application to wait for the thread if necessary,
    // or to manage its lifecycle. For now, it's not explicitly joined.
    _wayland_thread_join_handle: Option<std::thread::JoinHandle<()>>,
}

impl WaylandToplevelIntegration {
    pub fn new_and_start_thread() -> Result<Self, anyhow::Error> {
        tracing::info!("Initializing WaylandToplevelIntegration...");

        let conn = Connection::connect_to_env()
            .context("Failed to connect to Wayland display. Is WAYLAND_DISPLAY set for novade-ui?")?;

        // registry_queue_init will get wl_registry, wl_compositor, wl_shm, and wl_seat if available.
        // It requires WaylandState to implement Dispatch for these.
        let (globals, mut event_queue) = wayland_client::globals::registry_queue_init::<WaylandState>(&conn)
            .context("Failed to initialize globals and event queue using registry_queue_init")?;
        let qh = event_queue.handle();

        let toplevels_list_arc = Arc::new(Mutex::new(Vec::new()));

        let mut wayland_state_instance = WaylandState {
            toplevel_manager: None, // Will be populated by registry event
            toplevels_data: toplevels_list_arc.clone(),
            _queue_handle: qh.clone(),
        };

        // First roundtrip to get initial globals list processed by WaylandState's Dispatch for wl_registry
        event_queue.roundtrip(&mut wayland_state_instance)
            .context("Wayland roundtrip failed (initial global list processing)")?;

        // Second roundtrip to ensure any post-bind dispatches (like for the manager's quick_assign) might occur
        event_queue.roundtrip(&mut wayland_state_instance)
            .context("Wayland roundtrip failed (after manager binding attempt)")?;

        if wayland_state_instance.toplevel_manager.is_none() {
            tracing::warn!("zwlr_foreign_toplevel_manager_v1 global was not found or failed to bind after roundtrips. Toplevel info will not be available.");
        } else {
            tracing::info!("WaylandToplevelIntegration initialized, zwlr_foreign_toplevel_manager_v1 should be bound if available.");
        }

        let wayland_thread_handle = std::thread::spawn(move || {
            tracing::info!("Wayland event dispatch thread started.");
            loop {
                if let Err(e) = event_queue.blocking_dispatch(&mut wayland_state_instance) {
                    tracing::error!("Error in Wayland event dispatch loop: {}", e);
                    break;
                }
            }
            tracing::info!("Wayland event dispatch thread finished.");
        });

        Ok(Self {
            toplevels_data_access: toplevels_list_arc,
            _wayland_thread_join_handle: Some(wayland_thread_handle),
        })
    }
}
