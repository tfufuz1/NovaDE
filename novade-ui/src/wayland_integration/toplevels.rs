use anyhow::Context;
use std::sync::{Arc, Mutex};
use wayland_client::{
    globals::GlobalListContents,
    protocol::{wl_output, wl_registry, wl_seat, wl_shm, wl_compositor},
    Connection, Dispatch, EventQueue, QueueHandle, Proxy,
};
use wayland_protocols::wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1, zwlr_foreign_toplevel_manager_v1,
};
use gtk::glib; // For glib::Sender and glib::Receiver

#[derive(Debug, Clone)]
pub struct ToplevelInfo {
    pub wayland_id: u32,
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub raw_states: Vec<u32>,
    pub wl_handle: zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1,
}

#[derive(Debug, Clone)]
pub enum ToplevelUpdate {
    Snapshot(Vec<ToplevelInfo>),
    Added(ToplevelInfo),
    Removed { wayland_id: u32 },
    Updated(ToplevelInfo),
}

pub struct WaylandState {
    pub toplevel_manager: Option<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1>,
    pub toplevels_data: Arc<Mutex<Vec<ToplevelInfo>>>,
    _queue_handle: QueueHandle<WaylandState>,
    pub update_sender: glib::Sender<ToplevelUpdate>, // Added sender
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

                        let toplevels_data_for_manager = state.toplevels_data.clone();
                        let update_sender_for_manager = state.update_sender.clone();

                        manager.quick_assign(move |_mgr_proxy, event, _dispatch_data| {
                            match event {
                                zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                                    let handle_protocol_id = toplevel.id().protocol_id();
                                    tracing::info!("New toplevel advertised by manager: id={:?} (protocol_id: {})", toplevel.id(), handle_protocol_id);

                                    let new_toplevel_info = ToplevelInfo {
                                        wayland_id: handle_protocol_id,
                                        title: None,
                                        app_id: None,
                                        raw_states: Vec::new(),
                                        wl_handle: toplevel.clone(),
                                    };

                                    toplevels_data_for_manager.lock().unwrap().push(new_toplevel_info.clone());
                                    if let Err(e) = update_sender_for_manager.send(ToplevelUpdate::Added(new_toplevel_info)) {
                                        tracing::error!("Failed to send ToplevelUpdate::Added: {}", e);
                                    }

                                    let toplevels_data_for_handle = toplevels_data_for_manager.clone();
                                    let update_sender_for_handle = update_sender_for_manager.clone();
                                    toplevel.quick_assign(move |top_handle_proxy, handle_event, _handle_dispatch_data| {
                                        let mut toplevels_guard = toplevels_data_for_handle.lock().unwrap();
                                        if let Some(info) = toplevels_guard.iter_mut().find(|ti| ti.wayland_id == handle_protocol_id) {
                                            let mut send_update = true;
                                            match handle_event {
                                                zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                                                    tracing::info!("Toplevel (id: {}) Title: {}", handle_protocol_id, title);
                                                    info.title = Some(title);
                                                }
                                                zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                                                    tracing::info!("Toplevel (id: {}) AppId: {}", handle_protocol_id, app_id);
                                                    info.app_id = Some(app_id);
                                                }
                                                zwlr_foreign_toplevel_handle_v1::Event::State { state } => {
                                                    let states_u32: Vec<u32> = state
                                                        .chunks_exact(4)
                                                        .map(|chunk| u32::from_ne_bytes(chunk.try_into().unwrap()))
                                                        .collect();
                                                    tracing::info!("Toplevel (id: {}) State: {:?}", handle_protocol_id, states_u32);
                                                    info.raw_states = states_u32;
                                                }
                                                zwlr_foreign_toplevel_handle_v1::Event::Done => {
                                                    tracing::debug!("Toplevel (id: {}) Done (batch of updates complete)", handle_protocol_id);
                                                    // Done event usually follows other events, so the update will be sent.
                                                    // No separate ToplevelUpdate for Done.
                                                }
                                                zwlr_foreign_toplevel_handle_v1::Event::Closed => {
                                                    tracing::info!("Toplevel (id: {}) Closed", handle_protocol_id);
                                                    if let Err(e) = update_sender_for_handle.send(ToplevelUpdate::Removed { wayland_id: handle_protocol_id }) {
                                                       tracing::error!("Failed to send ToplevelUpdate::Removed: {}", e);
                                                    }
                                                    send_update = false; // Don't send Updated after Removed
                                                    // Release lock before retain and destroy
                                                    drop(toplevels_guard);
                                                    toplevels_data_for_handle.lock().unwrap().retain(|ti| ti.wayland_id != handle_protocol_id);
                                                    top_handle_proxy.destroy();
                                                    return;
                                                }
                                                zwlr_foreign_toplevel_handle_v1::Event::Parent { parent } => {
                                                    tracing::info!("Toplevel (id: {}) Parent: {:?}", handle_protocol_id, parent.map(|p| p.id()));
                                                    send_update = false; // Or handle as a specific update type if needed
                                                }
                                                zwlr_foreign_toplevel_handle_v1::Event::OutputEnter { output } => {
                                                    tracing::debug!("Toplevel (id: {}) OutputEnter: {:?}", handle_protocol_id, output.id());
                                                    send_update = false; // Or handle as a specific update type
                                                }
                                                zwlr_foreign_toplevel_handle_v1::Event::OutputLeave { output } => {
                                                    tracing::debug!("Toplevel (id: {}) OutputLeave: {:?}", handle_protocol_id, output.id());
                                                    send_update = false; // Or handle as a specific update type
                                                }
                                                _ => {
                                                     tracing::warn!("Toplevel (id: {}) Unhandled event: {:?}", handle_protocol_id, handle_event);
                                                     send_update = false;
                                                }
                                            }
                                            if send_update {
                                                if let Err(e) = update_sender_for_handle.send(ToplevelUpdate::Updated(info.clone())) {
                                                    tracing::error!("Failed to send ToplevelUpdate::Updated: {}", e);
                                                }
                                            }
                                        } else {
                                            tracing::warn!("Received event for toplevel handle (id: {}) but no ToplevelInfo found (possibly already closed).", handle_protocol_id);
                                        }
                                    });
                                }
                                zwlr_foreign_toplevel_manager_v1::Event::Finished => {
                                    tracing::info!("Foreign toplevel manager finished sending initial list of toplevels.");
                                    let current_list_clone = toplevels_data_for_manager.lock().unwrap().clone();
                                    if let Err(e) = update_sender_for_manager.send(ToplevelUpdate::Snapshot(current_list_clone)) {
                                        tracing::error!("Failed to send ToplevelUpdate::Snapshot: {}", e);
                                    }
                                }
                                _ => {}
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

impl Dispatch<wl_compositor::WlCompositor, ()> for WaylandState {fn event(_s: &mut Self, _i: &wl_compositor::WlCompositor, _e: wl_compositor::Event, _d: &(), _c: &Connection, _q: &QueueHandle<Self>) {}}
impl Dispatch<wl_shm::WlShm, ()> for WaylandState {fn event(_s: &mut Self, _i: &wl_shm::WlShm, _e: wl_shm::Event, _d: &(), _c: &Connection, _q: &QueueHandle<Self>) {}}
impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {fn event(_s: &mut Self, _i: &wl_seat::WlSeat, _e: wl_seat::Event, _d: &(), _c: &Connection, _q: &QueueHandle<Self>) {}}
impl Dispatch<wl_output::WlOutput, ()> for WaylandState {fn event(_s: &mut Self, _i: &wl_output::WlOutput, _e: wl_output::Event, _d: &(), _c: &Connection, _q: &QueueHandle<Self>) {}}
impl Dispatch<zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, ()> for WaylandState { fn event( _state: &mut Self, _manager: &zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1, _event: zwlr_foreign_toplevel_manager_v1::Event, _data: &(), _conn: &Connection, _qh: &QueueHandle<Self>) {}}
impl Dispatch<zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, ()> for WaylandState { fn event( _state: &mut Self, _handle: &zwlr_foreign_toplevel_handle_v1::ZwlrForeignToplevelHandleV1, _event: zwlr_foreign_toplevel_handle_v1::Event, _data: &(), _conn: &Connection, _qh: &QueueHandle<Self>) {}}

pub struct WaylandToplevelIntegration {
    pub toplevels_data_access: Arc<Mutex<Vec<ToplevelInfo>>>,
    _wayland_thread_join_handle: Option<std::thread::JoinHandle<()>>,
    pub update_sender: glib::Sender<ToplevelUpdate>, // Made public for access if needed, though typically not directly
}

impl WaylandToplevelIntegration {
    // Changed return type to include glib::Receiver
    pub fn new_and_start_thread() -> Result<(Self, glib::Receiver<ToplevelUpdate>), anyhow::Error> {
        tracing::info!("Initializing WaylandToplevelIntegration...");

        let (tx, rx) = glib::MainContext::channel(glib::Priority::DEFAULT);

        let conn = Connection::connect_to_env()
            .context("Failed to connect to Wayland display. Is WAYLAND_DISPLAY set for novade-ui?")?;

        let (_globals, mut event_queue) = wayland_client::globals::registry_queue_init::<WaylandState>(&conn)
            .context("Failed to initialize globals and event queue using registry_queue_init")?;
        let qh = event_queue.handle();

        let toplevels_list_arc = Arc::new(Mutex::new(Vec::new()));

        let mut wayland_state_instance = WaylandState {
            toplevel_manager: None,
            toplevels_data: toplevels_list_arc.clone(),
            _queue_handle: qh.clone(),
            update_sender: tx.clone(), // Store sender in WaylandState
        };

        event_queue.roundtrip(&mut wayland_state_instance)
            .context("Wayland roundtrip failed (initial global list processing)")?;

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
                match event_queue.blocking_dispatch(&mut wayland_state_instance) {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Error in Wayland event dispatch loop: {}", e);
                        if matches!(e.kind(), wayland_client::backend::WaylandError::Io(_)) {
                            tracing::info!("Wayland connection likely closed, exiting dispatch thread.");
                            break;
                        }
                    }
                }
            }
            tracing::info!("Wayland event dispatch thread finished.");
        });

        Ok((
            Self {
                toplevels_data_access: toplevels_list_arc,
                _wayland_thread_join_handle: Some(wayland_thread_handle),
                update_sender: tx, // Store the sender in the struct as well
            },
            rx // Return the receiver
        ))
    }
}
