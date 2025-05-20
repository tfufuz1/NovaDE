# novade-system

System layer for the NovaDE desktop environment.

## Overview
The System Layer provides the interface between the domain logic and the underlying operating system. It implements the Wayland compositor, input handling, D-Bus interfaces, audio management, and other system-level functionality.

## Modules
- **Compositor**: Wayland compositor implementation
- **Input**: Input device handling
- **D-Bus Interfaces**: Communication with system services
- **Audio Management**: PipeWire integration
- **MCP Client**: Model Context Protocol client
- **Portals**: XDG Desktop Portals backend
- **Power Management**: System power management

## Dependencies
- novade-core
- novade-domain
- smithay
- libinput
- xkbcommon
- zbus
- pipewire-rs
- mcp_client_rs

## Thread Safety
All modules in the System Layer are designed to be thread-safe, using appropriate synchronization primitives like Arc, Mutex, and RwLock where necessary.
