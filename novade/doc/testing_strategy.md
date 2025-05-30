# NovaDE Testing Strategy

This document outlines the strategy for testing the NovaDE Wayland compositor. It covers both unit testing and integration testing approaches.

## Unit Testing

Unit tests are written alongside the code (inline `#[cfg(test)]` modules) and focus on testing individual functions, methods, or small modules in isolation.

**Scope:**
-   **Compositor State (`NovaCompositorState`):** Verify initialization, default values, and state transitions where possible (e.g., focus tracking).
-   **Protocol Handlers:** Test specific logic within handlers that can be isolated. This is often challenging due to the handlers' reliance on live Wayland objects and client interactions. Conceptual tests are used where direct mocking is difficult, focusing on expected changes to `NovaCompositorState`.
-   **Utility Functions:** Any helper functions or distinct algorithms should be unit tested thoroughly.

**Challenges for Unit Testing Wayland Compositors:**
-   **Mocking Wayland Objects:** Core Wayland objects (like `wl_surface`, `wl_client`, or protocol-specific objects like `xdg_toplevel`) are difficult to mock effectively as their state and behavior are deeply tied to the Wayland C libraries and display server state.
-   **Handler Dependencies:** Protocol handlers often depend on the global compositor state, other handlers, and live client data, making isolated testing complex.
-   **Asynchronous Nature:** Some actions might trigger asynchronous responses or events, which are hard to capture in simple unit tests.

**Approach for Unit Tests:**
-   Focus on pure logic within functions/methods where possible.
-   For handler logic, test the parts that modify the compositor's own state (`NovaCompositorState`) assuming the Wayland object interactions are correct.
-   Use Smithay's testing utilities if/when available and suitable for isolated tests.
-   Acknowledge that comprehensive testing of handlers often requires integration tests.

## Integration Testing

Integration tests are crucial for verifying the interactions between different parts of the compositor and its behavior with real Wayland clients.

**Approach:**

1.  **Test Environment:**
    *   The NovaDE compositor will be launched in a test mode. This could be:
        *   **Headless Mode:** Using Smithay's headless backend. This is ideal for CI and automated tests as it requires no graphical environment.
        *   **Winit Mode:** Running the compositor within a Winit window on a host desktop. Useful for local testing and debugging visual aspects.
    *   The compositor runs as a separate thread or process from the test runner.

2.  **Test Client:**
    *   A dedicated Wayland test client will be used. This client can be:
        *   Built using a standard Wayland client library like `wayland-rs` or `wayland-client`.
        *   Leveraging examples from `smithay-client-toolkit` (SCTK) if they suit the testing needs.
        *   A custom-built minimal client specifically for sending test sequences.
    *   The client connects to the test instance of NovaDE compositor via its advertised Wayland socket.

3.  **Test Scenarios:**
    *   The test client will perform a sequence of Wayland requests (e.g., create surface, create SHM buffer, create xdg_toplevel, request move/resize).
    *   Assertions will be made based on:
        *   **Client-side state:** Did the client receive the expected configure events? Is its surface in the expected state?
        *   **Compositor-side effects (if observable):** This is harder. For visual tests, screenshot comparisons could be used (complex). For state, it's generally not directly queried in pure Wayland.
        *   **Protocol conformance:** Does the interaction adhere to Wayland protocol specifications? Smithay itself helps greatly here.

**Phased Rollout of Integration Tests:**

*   **Phase 1: Core Protocol Functionality**
    *   Client connection and disconnection.
    *   `wl_display` (global registration, sync/callback).
    *   `wl_registry` (listing and binding globals).
    *   `wl_compositor` (surface creation, attaching buffers, committing).
    *   `wl_shm` (shared memory pool creation, buffer creation from pool).

*   **Phase 2: Shell Integration (XDG Shell)**
    *   `xdg_wm_base` global and its events.
    *   Creation of `xdg_toplevel` and `xdg_popup` surfaces.
    *   Surface configuration cycle (compositor sends configure, client acks).
    *   Window states (maximized, minimized, fullscreen - if supported).
    *   Basic window movement and resizing requests.

*   **Phase 3: Input Handling (`wl_seat`)**
    *   Seat creation and capability reporting.
    *   Pointer events: motion, button clicks, enter/leave events on surfaces.
    *   Keyboard events: key presses, modifier states, enter/leave events.
    *   Focus changes (pointer and keyboard focus) and verification on the client side.
    *   Testing click-to-focus, keyboard focus setting.

*   **Phase 4: Advanced Features**
    *   Copy-paste (`wl_data_device_manager`).
    *   Drag-and-drop.
    *   Output management (`wl_output` - properties, geometry, scale).
    *   Screen capture/sharing protocols (if implemented).
    *   Custom NovaDE protocols.

**Tools and Libraries for Integration Testing:**
*   `wayland-rs` (client and server crates) for building test clients or tools.
*   `smithay-client-toolkit` (SCTK) for more feature-rich client interactions or as a reference.
*   Potentially test frameworks like `iai-wayland` (for Wayland protocol benching/testing) or custom scripting around client executables.

**Challenges for Integration Testing:**
*   **Environment Setup:** Ensuring the compositor can run reliably in a test environment (headless or Winit).
*   **Asynchronous Event Handling:** Tests need to correctly wait for and handle asynchronous events from the compositor. Synchronization primitives (e.g., fences, client-side sync callbacks) are essential.
*   **Debugging:** Debugging issues across the client-server boundary can be complex. Clear logging on both client and server side is vital.
*   **Test Flakiness:** Asynchronous interactions can sometimes lead to flaky tests if not carefully managed.
*   **Resource Cleanup:** Ensuring all client and server resources are properly cleaned up after each test.

This strategy will be refined as NovaDE development progresses and more specific testing needs arise.
```
