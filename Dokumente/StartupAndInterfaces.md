# NovaDE Startup Sequence and Core Interfaces

This document outlines the startup process of the NovaDE desktop environment and the primary interfaces between its main components.

## Startup Sequence

1.  **`novade` Orchestrator Execution:**
    *   The user or system initiates `target/debug/novade`.
    *   The `novade` process starts, initializing logging.

2.  **`novade-system` (Compositor) Launch:**
    *   `novade` spawns `target/debug/novade-system` as a child process.
    *   `novade-system` begins initialization:
        *   Initializes `tracing` for logging.
        *   Sets up its Tokio runtime for asynchronous tasks (e.g., MCP services).
        *   Initializes Model Context Protocol (MCP) services (e.g., `MCPConnectionService`, `DefaultCpuUsageService`).
        *   Initializes `novade-domain` services (`DomainServices`).
        *   Initializes `novade-system` specific services (`SystemServices`).
        *   Creates the Calloop event loop and the Smithay `Display`.
        *   Instantiates `DesktopState`, which holds the central compositor state and references to the initialized services.
        *   Initializes all necessary Wayland globals (e.g., `wl_compositor`, `wl_shm`, `wl_seat`, `wl_output`, `xdg_shell`, `zwlr_foreign_toplevel_manager_v1`, `gtk_primary_selection_device_manager`, etc.) via `create_all_wayland_globals`.
        *   Sets up the input backend (e.g., `NovadeLibinputManager`) and integrates it with the Calloop event loop.
        *   Initializes the graphics backend. For development, this is typically the Winit backend using WGPU (`WinitBackend`). This involves creating a window and the WGPU renderer. An associated Smithay `Output` is created and globalized.
        *   The Wayland display socket listener is started (e.g., `wayland-0` or `wayland-1`).
        *   `novade-system` enters its main Calloop event loop, ready to accept client connections and process events.

3.  **Orchestrator Delay & `novade-ui` (Shell) Launch:**
    *   `novade` (orchestrator) currently employs a hardcoded delay (e.g., 5 seconds) to allow `novade-system` to initialize.
    *   After the delay, `novade` spawns `target/debug/novade-ui` as another child process.
    *   Crucially, `novade` sets the `WAYLAND_DISPLAY` environment variable for the `novade-ui` process to match the socket `novade-system` is listening on (e.g., `wayland-1`).

4.  **`novade-ui` (Shell) Initialization:**
    *   `novade-ui` process starts.
    *   Initializes `tracing` for logging.
    *   Sets up `gettextrs` for internationalization.
    *   Loads GResource bundle (compiled UI definitions, CSS, icons).
    *   Applies custom CSS.
    *   Initializes its Wayland client integration (`WaylandToplevelIntegration`):
        *   Connects to the Wayland display specified by `WAYLAND_DISPLAY`.
        *   Creates a Wayland event queue and spawns a separate thread to dispatch it.
        *   Retrieves the `wl_registry` and binds to necessary globals, particularly `zwlr_foreign_toplevel_manager_v1`.
        *   Sets up listeners for toplevel window events from the manager.
    *   Creates the `adw::Application` and `adw::ApplicationWindow`.
    *   The main window is configured as a layer surface using `gtk4-layer-shell` (e.g., bottom panel).
    *   Builds the UI components (e.g., `SimpleTaskbar` containing clock, app launcher button, workspace switcher placeholder, and eventually the toplevel list).
    *   Sets up a GLib channel to receive `ToplevelUpdate` messages from the Wayland thread, updating a `gio::ListStore` that holds `ToplevelListItemGObject`s.
    *   Enters the GTK main event loop (`app.run()`).

5.  **Monitoring:**
    *   The `novade` orchestrator monitors both `novade-system` and `novade-ui` processes. If one exits, it attempts to terminate the other.

## Core Interfaces

*   **`novade` -> `novade-system` / `novade-ui`:**
    *   **Mechanism:** Standard OS process creation and management.
    *   **Interface:** Command-line arguments (if any), environment variables (primarily `WAYLAND_DISPLAY` for `novade-ui`).
    *   **Direction:** Orchestrator to children.

*   **`novade-ui` (Client) <-> `novade-system` (Wayland Compositor/Server):**
    *   **Mechanism:** Wayland protocol over a Unix domain socket.
    *   **Key Protocols:**
        *   `wl_display`, `wl_registry`: Connection and global discovery.
        *   `wl_compositor`, `wl_subcompositor`: Surface creation and hierarchy.
        *   `wl_shm`: Shared memory buffer passing for client rendering.
        *   `wl_seat`: Input device handling (keyboard, pointer, touch).
        *   `xdg_shell` (`xdg_wm_base`): Management of toplevel windows and popups.
        *   `zwlr_foreign_toplevel_manager_v1`: Allows `novade-ui` (as a shell component) to get information about all toplevel windows, their titles, app_ids, and states.
        *   `gtk_primary_selection_device_manager` (and related): Clipboard integration.
        *   `xdg_activation_v1`: Protocol for activating windows (e.g., focusing or bringing to front).
        *   Layer Shell protocol (via `gtk4-layer-shell` library): Allows `novade-ui` components to be positioned as part of the desktop shell (panels, docks, wallpaper layer).
    *   **Direction:** Bidirectional according to protocol specifications.

*   **Internal to `novade-system`:**
    *   **`DesktopState`:** Central struct holding compositor state, including references to Wayland globals, Smithay state objects (`CompositorState`, `XdgShellState`, `SeatState`, `OutputManagerState`, `ForeignToplevelManagerState`, etc.), the `Space` (for window and output arrangement), and collections of managed entities like windows (`ManagedWindow`) and outputs (`Output`).
    *   **Handlers:** `DesktopState` implements various Smithay handler traits (e.g., `CompositorHandler`, `XdgShellHandler`, `SeatHandler`, `OutputHandler`, `ForeignToplevelHandler`) that are called by Smithay in response to client requests or backend events.
    *   **Service Integration:** `DesktopState` holds `Arc<DomainServices>` and `Arc<SystemServices>`, allowing handlers to access domain logic (e.g., window placement policies) and system-level operations.
    *   **Input Chain:** Backend (Winit/Libinput) -> `InputDispatcher` -> `Seat` methods -> Focused client.
    *   **Rendering Chain:** Client commits buffer -> `CompositorHandler::commit` -> `SurfaceData` updated (texture created/associated) -> Renderer (WGPU) uses `SurfaceData` and `Space` information to draw frame.

*   **Internal to `novade-ui`:**
    *   **`WaylandToplevelIntegration`:** Manages the Wayland client connection, event queue (on a separate thread), and communication of toplevel window information to the GTK main thread via a `glib::channel`.
    *   **`ListStore<ToplevelListItemGObject>`:** GTK data model on the main thread, kept synchronized with toplevel window state from the Wayland thread.
    *   **UI Components (`SimpleTaskbar`, etc.):** Standard GTK/Adwaita widgets that observe the `ListStore` or other UI state objects to render the shell.
