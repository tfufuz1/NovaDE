# NovaDE: The next-generation Linux Desktop Environment built with Rust, Smithay, and GTK4-rs.

## Current Status

The NovaDE workspace has been successfully set up. This includes:
*   The `nova_compositor` crate (currently a placeholder).
*   The `nova_shell` crate (configured with GTK4 and GLib).

Workspace-level builds (`cargo build --workspace`) and tests (`cargo test --workspace`) are passing.

**Critical Blocker**: Development of the `nova_compositor` core using Smithay is currently blocked due to an incompatibility between the environment's Rust version (`1.75.0`) and the Minimum Supported Rust Version (MSRV) of available Smithay versions in the Cargo index. See `ADR/ADR004_smithay_version_blocker.md` for details.

Immediate development focus will be on `nova_shell` and other components that can be developed independently of the live compositor.

## Setup Instructions

1.  Clone the repository.
2.  Ensure you have necessary system dependencies for GTK4 and GLib installed (e.g., `libgtk-4-dev`, `libglib2.0-dev`, `pkg-config` on Debian/Ubuntu).
3.  Run `cargo build --workspace` from the project root.
4.  To run tests for the entire workspace, use `cargo test --workspace`.

## Contributing
(To be added - details on contribution guidelines, code style, etc.)

## License
(To be added - specify project license, likely MIT or Apache 2.0)
