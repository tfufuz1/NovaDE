# ADR-0002: Core Compositor Architecture

**Status:** Accepted

**Date:** 2023-10-28

**Context:**

NovaDE aims to be a modern, performant, and user-friendly desktop environment. At its heart lies the Wayland compositor, responsible for managing displays, windows, input, and rendering. The stability and efficiency of the compositor are critical to the overall user experience.

We have chosen the [Smithay](https://smithay.github.io/) library as the foundational framework for building our compositor due to its robust feature set, active development, and focus on correctness and safety, aligning well with Rust's principles.

A "Compositor-first Design Approach" is being adopted. This means establishing a clear and well-defined compositor architecture *before* significant investment in higher-level UI components (the shell). This ADR outlines this core architecture.

**Decision: Modular Compositor Design**

We will implement a modular compositor design within the `novade::compositor` crate. This design emphasizes separation of concerns, testability, and extensibility. The key modules are:

1.  **Core (`novade::compositor::core`):**
    *   Manages the global compositor state. Smithay's `CompositorState` will be central, initialized using `DataInit` for all necessary Wayland globals.
    *   Handles the creation, registration, and advertisement of Wayland globals (e.g., `wl_compositor`, `wl_shm`, `xdg_wm_base`, `wl_seat`).
    *   Orchestrates the main event loop and dispatches incoming Wayland events to appropriate handlers via Smithay's dispatching mechanisms.
    *   Manages client connections and their associated resources.

2.  **State Management (`novade::compositor::state`):**
    *   Defines clear, type-safe Rust data structures for representing all compositor-managed entities: clients, outputs (displays), surfaces (windows), seats (input device collections), popups, etc.
    *   Emphasizes strict adherence to Rust's ownership and borrowing rules to prevent data races, ensure thread safety (where applicable), and maintain overall system stability.
    *   Leverages Smithay's `DelegateDispatch<D, Global, State>` mechanism extensively. Each Wayland global interface (e.g., `WlCompositor`, `XdgShell`) will have its state managed by a dedicated struct that implements the relevant Smithay `Delegate<D>` traits. This promotes modularity in handling protocol-specific logic.

3.  **Backend Abstraction (`novade::compositor::backends`):**
    *   A trait-based abstraction layer will be designed to support multiple backend types, allowing NovaDE to run in various environments.
    *   **DRM/KMS Backend:** Prioritized for direct hardware control on Linux systems. This backend will manage display modes, atomic modesetting, and direct rendering buffer submission. It is essential for performance and stability on target production systems.
    *   **Winit Backend:** Crucial for development and testing on systems without direct DRM access (e.g., running as a window under X11 or another Wayland compositor). It will use the `winit` crate for windowing and input.
    *   **X11-embedded Backend (Optional Future):** For running NovaDE within an X11 window, potentially useful for development or nested scenarios.
    *   **Headless Backend (Testing):** For automated testing scenarios without actual display or input hardware.
    *   Each backend will be responsible for sourcing input events (keyboard, mouse, touch) and managing output display configurations and rendering contexts.

4.  **Handlers (`novade::compositor::handlers`):**
    *   This module will contain implementations of Smithay's `Delegate<D>` traits for each Wayland protocol extension NovaDE supports (e.g., `WlCompositorHandler`, `XdgShellHandler`, `WlSeatHandler`, `PresentationTimeHandler`, `FractionalScaleManagerHandler`, etc.).
    *   Handlers encapsulate the logic for processing client requests and sending events related to their specific Wayland protocols.
    *   They will interact with the `novade::compositor::state` module to modify and query compositor state based on client actions.
    *   Handlers should remain focused on protocol logic and delegate more complex operations (e.g., window management decisions, rendering tasks) to other specialized modules.

5.  **Rendering Pipeline (`novade::compositor::render`):**
    *   Abstracts the rendering process to be adaptable to different strategies and hardware capabilities.
    *   **Initial Support:** May start with Pixman for software rendering to ensure broad compatibility and ease of initial development.
    *   **Hardware Acceleration:** Target OpenGL ES (via Smithay's `glow` integration) for hardware-accelerated rendering, leveraging GPU capabilities for better performance.
    *   Manages efficient buffer handling, including support for client-provided SHM buffers and DMA-BUFs for zero-copy buffer sharing.
    *   Implements damage tracking (tracking regions of surfaces that have changed) to minimize rendering overhead by only redrawing necessary parts of the screen.
    *   Integrates closely with the `Backend Abstraction` for output-specific rendering tasks (e.g., swapping buffers).

6.  **Input Pipeline (`novade::compositor::input`):**
    *   Processes raw input events (e.g., key presses, pointer motion, touch events) received from the active backend.
    *   Manages the state of input devices via `wl_seat`, including keyboard focus, pointer location, cursor appearance, and device capabilities (keyboard, pointer, touch).
    *   Implements logic for determining which client surface should receive an input event based on factors like pointer position, keyboard focus, and grab semantics.
    *   Dispatches processed input events to the appropriate client surfaces according to Wayland protocol rules.

**Consequences:**

*   **Pros:**
    *   **Modularity:** Simplifies development by allowing teams or individuals to focus on specific components. Makes the codebase easier to understand, test, and maintain.
    *   **Clear Separation of Concerns:** Reduces coupling between different parts of the compositor, leading to a more robust design.
    *   **Extensibility:** Easier to add support for new Wayland protocols, rendering techniques, or backend types without major refactoring of existing code.
    *   **Testability:** Individual modules (e.g., input processing, specific protocol handlers) can be unit-tested more effectively. The headless backend will facilitate integration testing.
    *   **Backend Flexibility:** The ability to switch or add backends is crucial for development, testing, and deployment on diverse hardware.
*   **Cons:**
    *   **Initial Setup Complexity:** Defining clear interfaces and abstractions for all these modules requires significant upfront design effort compared to a more monolithic approach.
    *   **Interface Management:** Careful design and versioning of interfaces between modules are necessary to avoid integration issues.
    *   **Potential for Over-Abstraction:** If not managed carefully, some abstractions might add unnecessary boilerplate for simple tasks.
*   **Mitigation:**
    *   **Strong Documentation:** This ADR and subsequent detailed design documents for each module will serve as a reference. Inline code documentation will be emphasized.
    *   **Iterative Implementation:** Start with the `core`, a basic `state` implementation, the `Winit` backend, and essential handlers (`wl_compositor`, `xdg_shell`). Other modules and features will be added incrementally.
    *   **Regular Code Reviews:** Ensure adherence to the architectural principles and maintain interface consistency.
    *   **Prototyping:** Key interactions between modules might be prototyped to validate design choices early.

**Alternatives Considered:**

*   **Monolithic Compositor:**
    *   **Description:** All compositor logic (Wayland handling, rendering, input, backend interaction) is tightly coupled within a single, large codebase or a few large modules.
    *   **Pros:** Potentially faster initial development for a very basic compositor.
    *   **Cons:** Becomes very difficult to scale, maintain, and debug as features are added. Poor separation of concerns makes it brittle. Less flexible for supporting multiple backends or rendering strategies.
*   **Microkernel-style Compositor:**
    *   **Description:** The core compositor (microkernel) is minimal, with major functionalities like window management, input handling, and even protocol implementations running as separate processes communicating via IPC.
    *   **Pros:** Extremely high degree of decoupling and fault isolation.
    *   **Cons:** Significant IPC overhead, which can impact performance. Much higher complexity in terms_of process management and IPC mechanisms. This approach is generally more suited for systems requiring extreme security or supporting third-party, untrusted components directly within the compositor architecture, which is beyond the initial scope of NovaDE.

This modular approach, leveraging Smithay's strengths, is deemed the most suitable for NovaDE's long-term goals of creating a stable, performant, and maintainable desktop environment.
