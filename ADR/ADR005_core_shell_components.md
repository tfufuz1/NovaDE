# ADR005: Initial Core Shell Components for NovaDE

**Date**: 2025-05-31
**Status**: Proposed

## Context and Problem Statement

With the `nova_compositor` development temporarily blocked (see ADR004), focus is shifting to building out the `nova_shell`. To guide this development, we need to define an initial set of core user-facing components that constitute a minimal desktop shell experience. This ADR outlines these components.

## Decision Drivers

*   Provide a structured approach to `nova_shell` development.
*   Identify key UI areas that will require interaction with the (eventual) compositor.
*   Enable parallel work on different shell aspects.
*   Align with common expectations for a modern desktop environment.

## Proposed Core Shell Components

The initial set of core shell components for NovaDE will include:

1.  **Panel (`nova_shell::panel`):**
    *   **Description**: A persistent bar, typically at the top or bottom of the screen.
    *   **Responsibilities**: Displaying system status (clock, battery, network), application launchers, workspace indicators, and a system tray.
    *   **Sub-elements (examples)**: Clock, AppMenu button (if applicable), Workspace Switcher, System Tray.

2.  **Launcher (`nova_shell::launcher`):**
    *   **Description**: A system for users to find and launch applications.
    *   **Responsibilities**: Displaying installed applications, allowing search/filtering, and triggering application startup (which will require compositor interaction).
    *   **Possible Forms**: Full-screen app grid, pop-up menu, searchable command bar. Initial implementation will likely be simple.

3.  **Notification System (`nova_shell::notifications`):**
    *   **Description**: Displays transient messages from applications and the system.
    *   **Responsibilities**: Receiving notification requests, displaying them in a consistent manner (e.g., pop-ups, list), managing notification history/queue.

4.  **Workspace Manager (`nova_shell::workspaces`):**
    *   **Description**: Manages virtual desktops or workspaces.
    *   **Responsibilities**: Tracking available workspaces, the active workspace, and windows within each workspace (largely a compositor function, but shell provides UI). Displaying workspace overview/switcher.

5.  **Desktop Background Manager (`nova_shell::background`):**
    *   **Description**: Manages the desktop background wallpaper.
    *   **Responsibilities**: Setting and displaying wallpaper. May involve interaction with the compositor for rendering.

## Decision Outcome

These components will form the initial focus for `nova_shell` development. Each will be developed in its own submodule within the `nova_shell` crate. Initial implementations will be placeholders, followed by basic GTK4 UI structure, and then functionality.

Interaction points with the compositor will be identified and initially mocked or handled via placeholder APIs.

## Pros and Cons

*   **Pros**:
    *   Clear scope for initial shell development.
    *   Modular design.
    *   Addresses common DE functionalities.
*   **Cons**:
    *   Ambitious list; full implementation of all will take time.
    *   Some components heavily rely on future compositor features.

## Next Steps
*   Create corresponding submodules in `nova_shell`.
*   Begin placeholder struct/enum implementation for each.
