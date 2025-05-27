use smithay::reexports::wayland_server::UserDataMap;
use uuid::Uuid;
use crate::compositor::core::state::ClientCompositorData; // For storing in UserDataMap
use smithay::wayland::shell::xdg::XdgWmBaseClientData; // For storing in UserDataMap

/// Data associated with each connected Wayland client.
///
/// This struct is intended to be stored in the `UserDataMap` of each
/// `wayland_server::Client` object. It can hold both our custom per-client
/// identifiers/data and also serve as a container for Smithay's per-client
/// protocol states via its `user_data` field (which is itself a `UserDataMap`).
#[derive(Debug, Clone)]
pub struct ClientData {
    /// A unique internal identifier for this client.
    pub id: Uuid,
    /// A `UserDataMap` to store various client-specific data,
    /// including Smithay's protocol states like `XdgWmBaseClientData`
    /// or our own `ClientCompositorData`.
    pub user_data_map: UserDataMap,
}

impl ClientData {
    /// Creates new `ClientData` with a fresh UUID and an empty `UserDataMap`.
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            user_data_map: UserDataMap::new(),
        }
    }

    // Example of how to store ClientCompositorData (if not already handled by Smithay's CompositorState)
    // pub fn compositor_data(&self) -> &ClientCompositorData {
    //     self.user_data_map.get::<ClientCompositorData>().expect("ClientCompositorData not initialized")
    // }

    // Example of how to store XdgWmBaseClientData
    // pub fn xdg_wm_base_data(&self) -> &XdgWmBaseClientData {
    //     self.user_data_map.get::<XdgWmBaseClientData>().expect("XdgWmBaseClientData not initialized")
    // }
}

impl Default for ClientData {
    fn default() -> Self {
        Self::new()
    }
}

// Note: Smithay's `Client` object (from `wayland_server::Client`) itself has a `UserDataMap`
// accessible via `client.user_data()`. We can either:
// 1. Store `ClientData` (this struct) directly into the `Client::user_data()`.
// 2. Store individual components like `Arc<Uuid>` and other states directly into `Client::user_data()`.
//
// The current plan seems to imply that `ClientData` itself is stored.
// If `ClientData` *is* the main struct in `Client::user_data()`, then its own `user_data_map` field
// would be a nested `UserDataMap`. This is possible but perhaps redundant if the top-level
// `Client::user_data()` can serve the purpose of holding all per-client states directly.
//
// Let's assume `ClientData` is stored in `Client::user_data()`. Then, things like
// `XdgWmBaseClientData` (which Smithay's `XdgShellState::new_client` provides)
// would be put into `ClientData::user_data_map`.
//
// Re-reading the plan: "Associate this ClientData with the new wayland_server::Client object using its data_map()".
// This suggests `ClientData` is *the* object inserted.
// "Smithay's XdgShellState::new_client ... should also be called here ... to initialize client-specific protocol states.
// These states are often stored within the ClientData::user_data map."
// This confirms the nested structure.

// Example initialization during client connection (conceptual):
// fn handle_new_client(client: &wayland_server::Client) {
//     let client_data = Arc::new(ClientData::new());
//     client.user_data().insert_if_missing_threadsafe(|| client_data.clone());
//
//     // Initialize XDG shell state for this client
//     let xdg_client_data = XdgShellState::new_client(...); // From DesktopState.xdg_shell_state
//     client_data.user_data_map.insert_if_missing_threadsafe(|| xdg_client_data);
//
//     // Initialize Compositor state for this client
//     let compositor_client_data = CompositorState::new_client(...); // From DesktopState.compositor_state
//     client_data.user_data_map.insert_if_missing_threadsafe(|| compositor_client_data);
// }
// Smithay's CompositorState and XdgShellState often manage their own per-client data
// directly in the Client's UserDataMap when their respective `new_client` methods are called,
// or when globals are bound.
//
// If `XdgShellState::new_client` and `CompositorState::new_client` (if it exists/is used this way)
// already populate the main `Client::user_data()`, then `ClientData` might just need its `id` field,
// and the `user_data_map` field within `ClientData` would be redundant.
//
// Let's simplify: `ClientData` holds our `id`. Smithay's states will populate `Client::user_data()` directly.
// We can access them from `Client::user_data()` when needed.
// So, `ClientData` will not have its own `user_data_map`.

// --- Revised ClientData based on simpler approach ---
// This ClientData will be one of the items in Client::user_data(), not the sole item.
#[derive(Debug, Clone)]
pub struct MinimalClientData {
    /// A unique internal identifier for this client.
    pub id: Uuid,
}

impl MinimalClientData {
    pub fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

impl Default for MinimalClientData {
    fn default() -> Self {
        Self::new()
    }
}
// In DesktopState::client_created:
// client.user_data().insert_if_missing_threadsafe(MinimalClientData::new);
// desktop_state.xdg_shell_state.new_client(client); // This would put XdgWmBaseClientData into client.user_data()
// desktop_state.compositor_state.new_client(client); // This would put CompositorClientState into client.user_data()

// The plan mentions: "ClientData::user_data: UserDataMap". This implies the first, nested approach.
// Let's stick to the original plan for ClientData with its own UserDataMap,
// as it allows namespacing our specific client data separately if needed,
// even if Smithay's states also use the main Client::user_data map.
// If Smithay's `new_client` methods return data that *we* must store, then ClientData::user_data_map
// is the place. If they store it themselves in Client::user_data(), then ClientData can be simpler.

// Smithay's `XdgShellState::new_client` *returns* an `XdgWmBaseClientData`. This is what we need to store.
// So, `ClientData { id: Uuid, user_data_map: UserDataMap }` is the correct interpretation.
// `XdgWmBaseClientData` will go into `ClientData.user_data_map`.
// `ClientCompositorState` (from `CompositorState`) will also go into `ClientData.user_data_map`.
// And then `Arc<ClientData>` itself will be put into `Client::user_data()`.
// This matches the detailed plan.Okay, I've created `system/src/compositor/display_loop/client_data.rs` and defined `ClientData` with a `Uuid` and its own `UserDataMap`. This aligns with the plan to store Smithay's per-client protocol data (like `XdgWmBaseClientData` and `CompositorClientState`) within this nested map.

**Step 3: Implement Display Loop Logic and Client Handling**

Next, I'll create `system/src/compositor/display_loop/mod.rs`. This file will contain:
1.  The function `register_wayland_event_source` (or similar logic integrated elsewhere).
2.  The callback logic for the Wayland event source.
3.  Logic for `ClientHandler::client_created` (which will be part of `DesktopState`'s implementation).

First, I need to ensure `DesktopState` in `system/src/compositor/core/state.rs` is prepared for `ClientHandler`. This involves:
- Adding `use smithay::reexports::wayland_server::{ClientData as WaylandClientData, ClientHandler, ClientId, Display};` (adjust imports as needed).
- Adding `delegate_client_handler!(DesktopState);`
- Implementing `ClientHandler` for `DesktopState`.

I will modify `system/src/compositor/core/state.rs` first.
