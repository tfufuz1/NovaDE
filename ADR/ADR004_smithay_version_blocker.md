# ADR004: Smithay Dependency Blocker due to Rust Version and Cargo Index Issues

**Date**: 2025-05-31
**Status**: Accepted

## Context and Problem Statement

The NovaDE project relies on Smithay for its Wayland compositor implementation. During initial project setup, we encountered a critical issue: the development environment's Rust compiler (`rustc 1.75.0`) is incompatible with the Minimum Supported Rust Version (MSRV) of all Smithay versions accessible through the environment's current Cargo package index.

Attempts to use Smithay versions `0.5.0`, `0.5.1`, and `0.6.0` (which were the only versions discoverable by Cargo in the environment) resulted in compilation errors indicating they require `rustc 1.80.1` or newer. Attempts to find newer Smithay versions (e.g., `0.9.0`, `0.10.0`, `0.11.0`) failed, suggesting the local Cargo index is outdated or restricted. The command `cargo update` did not resolve this.

This effectively blocks all development work on the `nova_compositor` crate that directly involves Smithay.

## Decision Drivers

*   **Core Requirement**: Smithay is a foundational technology for NovaDE's compositor.
*   **Environment Constraint**: `rustc 1.75.0` is the fixed Rust version in the current environment.
*   **Cargo Index Limitation**: The Cargo package index available to the environment does not seem to provide access to a wider range of Smithay versions, including potentially older ones that might be compatible with Rust 1.75.0, or newer ones that might have different MSRV characteristics if the index were up-to-date.

## Considered Options

1.  **Attempt to find an older, compatible Smithay version**: Without a reliable way to browse or search crates.io for specific MSRVs across all historical Smithay versions, this is difficult and time-consuming with current tooling.
2.  **Upgrade Rust in the environment**: This is outside the autonomous capabilities of the development agent and would require manual intervention by environment administrators.
3.  **Fix/Update Cargo Index in the environment**: Similar to upgrading Rust, this requires external intervention.
4.  **Pause Smithay-dependent compositor development**: Proceed with other aspects of NovaDE that do not directly depend on Smithay being compilable, while formally documenting this blocker.

## Decision Outcome

**Chosen Option**: Option 4. Pause Smithay-dependent compositor development.

This decision is made due to the current inability to resolve the Smithay dependency issue within the existing environment constraints and agent capabilities.

**Immediate Consequences**:
*   The `nova_compositor` crate will be maintained as a placeholder library without Smithay integration for now.
*   The "Compositor-first" development approach is temporarily suspended.
*   Focus will shift to other areas of the project that can proceed, such as setting up the `nova_shell` crate (with GTK4), developing utility libraries, or refining architectural designs that are not Smithay-implementation-specific.

**Future Re-evaluation**: This decision should be re-evaluated immediately if:
*   The Rust version in the development environment is upgraded.
*   The Cargo package index is updated or fixed, providing access to a compatible Smithay version.

## Pros and Cons of the Decision

*   **Pros**:
    *   Allows some project progress in other areas despite a major blocker.
    *   Formally documents the issue for external stakeholders or environment maintainers.
*   **Cons**:
    *   Delays core compositor development, which is central to NovaDE.
    *   May lead to integration challenges later when Smithay can finally be incorporated.

## ADR Creation

*   Create a directory `ADR` in the project root if it doesn't exist.
*   Place the new ADR file within this directory.
