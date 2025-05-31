# ADR-0001: NovaDE Orchestrator Design

## Status
Accepted

## Context
NovaDE consists of multiple primary components: the Wayland compositor (`novade-system`) and the desktop shell UI (`novade-ui`). A mechanism is required to launch and manage these components as a cohesive desktop environment.

## Decision
A dedicated `novade` crate will serve as the main orchestrator. Its primary responsibilities are:
1.  Launch the `novade-system` process (Wayland compositor and system services).
2.  After a brief initialization period for `novade-system`, launch the `novade-ui` process (desktop shell).
3.  Set necessary environment variables for `novade-ui`, critically `WAYLAND_DISPLAY`, to ensure it connects to the `novade-system` compositor.
4.  Monitor both child processes. If one terminates unexpectedly, the orchestrator will attempt to terminate the other to ensure a clean shutdown or allow for a restart of the environment.

## Rationale
*   **Process Separation:** Isolating the compositor and the shell into separate processes enhances stability. A crash in the shell UI will not necessarily bring down the entire display server and all running applications.
*   **Architectural Clarity:** This model aligns with common desktop environment architectures (e.g., Xorg server + window manager/desktop environment, or Wayland compositor + shell as separate entities).
*   **Simplicity for Initial Development:** Spawning processes is a straightforward approach for managing these distinct parts of the system compared to embedding them as libraries within a single, complex multi-threaded process, especially during early development stages.
*   **Independent Updatability (Future):** Allows for potentially updating the shell and compositor independently, though this is not a primary driver for the initial decision.

## Alternatives Considered
1.  **Single Process, Multi-Threaded/Async Tasks:**
    *   Run both `novade-system` (compositor logic) and `novade-ui` (GTK event loop) within the same process using Rust's async capabilities and/or threading.
    *   *Pros:* Potentially tighter integration, shared memory more easily.
    *   *Cons:* Increased complexity in managing event loops (Wayland/Calloop and GTK/GLib) in the same process, higher risk of one component's failure affecting the other directly, more complex build and state management.
2.  **Compositor Launches Shell:**
    *   Have `novade-system` (the compositor) be responsible for launching `novade-ui` after its own initialization.
    *   *Pros:* Ensures shell only starts after compositor is ready.
    *   *Cons:* Blurs the responsibility of `novade-system`, making it more than just a Wayland compositor and system service provider. The orchestrator pattern provides a clearer separation of concerns for overall session management.

## Consequences
*   **Inter-Process Communication (IPC):** Beyond standard Wayland protocols, any advanced or custom communication between the shell and the compositor/system services will require explicit IPC mechanisms (e.g., D-Bus, custom sockets).
*   **Startup Synchronization:** The current implementation uses a hardcoded delay in the `novade` orchestrator to wait for `novade-system` to initialize before launching `novade-ui`. This is not robust and will need to be replaced with a more reliable synchronization mechanism (e.g., `novade-system` signaling readiness via a temporary file, D-Bus signal, or specific stdout message).
*   **Environment Management:** The `novade` orchestrator is responsible for setting up the correct environment for child processes.
*   **Build System:** Executables for `novade-system` and `novade-ui` must be built and discoverable by the `novade` orchestrator (e.g., assumed to be in `target/debug/`).
