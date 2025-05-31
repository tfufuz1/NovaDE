# NovaDE Architecture

## Vision

To create the most stable, performant, secure, and innovative Linux Desktop Environment.

## Core Components

*   **`nova_compositor`**: Smithay-based Wayland compositor. Handles window management, rendering, input, and Wayland protocol implementations.
    *   **Note**: As of 2025-05-31, direct implementation of the Smithay-based compositor is on hold due to issues detailed in `ADR/ADR004_smithay_version_blocker.md`. The `nova_compositor` crate exists as a placeholder. Architectural design will continue, but implementation will resume once the blocker is resolved.
*   **`nova_shell`**: GTK4-rs based desktop shell. Provides the user interface elements (panels, launchers, notifications, etc.). Initial setup of this crate with GTK4 dependencies is complete.

## Compositor-First Approach

The compositor is the heart of NovaDE. The shell UI communicates with and is driven by the compositor. While direct compositor implementation is paused, design and theoretical work on shell-compositor interaction protocols will continue.

## Current Development Focus

With the `nova_compositor` implementation temporarily blocked, immediate development efforts will concentrate on:
*   Building out `nova_shell` components and UI elements.
*   Developing shared libraries and utilities.
*   Further refining architectural plans and preparing for eventual compositor integration.

## Interaction Model

`nova_shell` will communicate with `nova_compositor` primarily through custom Wayland protocols. Further details and potential use of other IPC mechanisms (like D-Bus for specific services) will be defined in Architecture Decision Records (ADRs).

## Modularity

NovaDE aims for a modular and extensible architecture. Components should be loosely coupled and replaceable where feasible. This principle remains key even with current implementation challenges.

## Standards and Best Practices

We adhere to:
*   RUST-SPEZIFISCHE EXCELLENCE-STANDARDS (Rust specific best practices and idiomatic code)
*   GTK4-RS BEST PRACTICES (Effective and modern usage of gtk4-rs)

This document reflects the project's long-term architectural goals. For current implementation status and blockers, please refer to the main `README.md` and relevant ADRs.
