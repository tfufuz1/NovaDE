# novade-ui

UI layer for the NovaDE desktop environment.

## Overview

The UI Layer provides the graphical user interface for the NovaDE desktop environment. It builds upon the System Layer to create a cohesive and user-friendly desktop experience.

## Modules

- **Window Manager UI**: User interface for window management
- **Desktop UI**: Main desktop interface and wallpaper management
- **Panel UI**: Top/bottom panel with system indicators
- **Application Launcher**: Application menu and search
- **Settings UI**: User interface for system settings
- **Notification UI**: User interface for system notifications
- **Theme UI**: User interface for theme management
- **Workspace UI**: User interface for workspace management
- **System Tray**: System tray implementation

## Dependencies

- novade-core: Core utilities and types
- novade-domain: Domain interfaces and types
- novade-system: System implementations
- iced: GUI toolkit for Rust
- Various libraries for image processing, file dialogs, etc.

## Usage

This crate is intended to be used as part of the NovaDE desktop environment and is not meant to be used standalone.
