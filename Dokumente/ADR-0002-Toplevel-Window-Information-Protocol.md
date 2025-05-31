# ADR-0002: Toplevel Window Information Protocol

## Status
Accepted

## Context
The NovaDE desktop shell (`novade-ui`) requires information about application windows (toplevels) managed by the Wayland compositor (`novade-system`). This information includes window titles, application IDs, states (active, maximized, etc.), and lifecycle events (creation, closure). This is essential for features like taskbars, window switchers, and general shell awareness of running applications.

## Decision
NovaDE will use the standard Wayland protocol extension `zwlr_foreign_toplevel_manager_v1` for communication of toplevel window information from the compositor (`novade-system`) to the shell (`novade-ui`).

## Rationale
*   **Standardization:** `zwlr_foreign_toplevel_manager_v1` is a well-established (though still part of "wlr-unstable-protocols") protocol designed specifically for this use case. Using a standard protocol promotes interoperability and leverages existing knowledge and tooling.
*   **Smithay Support:** The Smithay library, used for developing `novade-system`, provides built-in support for implementing the server-side of this protocol, reducing development effort.
*   **Client-Side Availability:** Wayland client libraries often include support or examples for interacting with this protocol.
*   **Sufficiency:** The protocol provides the necessary information (title, app_id, states, open/close events) for core shell features.
*   **Avoids Custom Complexity:** Designing a custom Wayland protocol or a D-Bus service for this specific, real-time window information would add unnecessary complexity and maintenance burden for a problem already solved by an existing standard.

## Alternatives Considered
1.  **Custom Wayland Protocol:**
    *   Design a new Wayland protocol specific to NovaDE for window information.
    *   *Pros:* Complete control over the protocol details.
    *   *Cons:* Significant development and maintenance effort, non-standard (harder for other tools/shells to integrate if ever needed), potential for design errors.
2.  **D-Bus Service:**
    *   Expose window information via a custom D-Bus service provided by `novade-system`.
    *   *Pros:* D-Bus is well-suited for service-oriented architecture and commands.
    *   *Cons:* Potentially less efficient for the rapid, real-time updates often needed for window states and lists compared to direct Wayland events. Wayland protocols are generally preferred for core display server and window management interactions. D-Bus could still be used for *requesting actions* on windows if needed, complementing the Wayland protocol.

## Consequences
*   **Implementation Effort:** Both `novade-system` (server-side) and `novade-ui` (client-side) must correctly implement their respective parts of the `zwlr_foreign_toplevel_manager_v1` protocol.
*   **Protocol Versioning:** The protocol is "unstable," so future versions might introduce changes, though it's relatively mature.
*   **Dependency:** Relies on the features being enabled in the Smithay and `wayland-protocols` crates.
*   **Shell Responsibility:** `novade-ui` becomes responsible for managing the state of toplevels it learns about via this protocol.
