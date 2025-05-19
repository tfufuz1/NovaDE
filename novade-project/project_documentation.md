# NovaDE Project Documentation

## Project Overview

NovaDE is a modern desktop environment designed with user-friendliness, modularity, and performance in mind. The project follows a layered architecture approach, separating concerns into distinct layers:

1. **Core Layer**: Fundamental building blocks and utilities
2. **Domain Layer**: Business logic and domain-specific functionality
3. **System Layer**: System integration and hardware interaction
4. **UI Layer**: User interface components and visual presentation

This documentation provides a comprehensive overview of the project structure, implementation details, and design decisions.

## Architecture

### Layered Architecture

The NovaDE desktop environment is built using a layered architecture pattern, where each layer has specific responsibilities and dependencies:

- **Core Layer**: The foundation layer with no external dependencies
- **Domain Layer**: Depends only on the Core Layer
- **System Layer**: Depends on Core and Domain Layers
- **UI Layer**: Depends on all other layers

This separation ensures that:
- Lower layers are not aware of higher layers
- Dependencies flow downward, not upward
- Each layer can be tested and developed independently
- Changes in one layer have minimal impact on other layers

### Module Organization

Each layer is further divided into modules, each with a specific responsibility:

1. **Core Layer Modules**:
   - Error handling
   - Logging
   - Configuration
   - Basic types
   - Utilities

2. **Domain Layer Modules**:
   - Workspace management
   - Theming
   - Global settings
   - Window policy
   - Notifications

3. **System Layer Modules**:
   - Compositor
   - Input handling
   - D-Bus interfaces
   - Event bridge

4. **UI Layer Modules**:
   - Application
   - Shell components
   - Widgets
   - Theming GTK

## Core Layer

The Core Layer provides fundamental building blocks and utilities used throughout the NovaDE desktop environment. It is designed to be independent of any specific desktop environment functionality and could potentially be reused in other projects.

### Error Module

The Error module (`error.rs`) provides a comprehensive error handling system using the `thiserror` crate. It defines a hierarchy of error types:

- `CoreError`: The root error type for the entire application
- `ConfigError`: Errors related to configuration loading and parsing
- `LoggingError`: Errors related to logging initialization and operation

Key features:
- Context can be added to errors for better diagnostics
- Error conversion traits for seamless integration
- Clear and actionable error messages

Example usage:
```rust
fn load_something() -> Result<(), CoreError> {
    let file = std::fs::File::open("config.toml")
        .map_err(|e| CoreError::from(e).with_context("Failed to open config file"))?;
    // ...
    Ok(())
}
```

### Types Module

The Types module provides fundamental data types used throughout the application:

#### Geometry (`types/geometry.rs`)

Provides geometric primitives:
- `Point<T>`: A point in 2D space with generic coordinate type
- `Size<T>`: Dimensions with width and height
- `Rect<T>`: A rectangle with position and size
- `RectInt`: A type alias for `Rect<i32>` for pixel-based operations

These types support common operations like addition, subtraction, multiplication, and division, as well as geometric operations like intersection and containment tests.

#### Color (`types/color.rs`)

Provides color handling:
- `Color`: An RGBA color representation
- `ColorFormat`: Supported color formats (RGB, RGBA, HSL, HSLA)

Features include:
- Conversion between different color formats
- Parsing from hex strings, rgb(), and hsl() formats
- Color blending and manipulation
- HSL adjustments (lightness, saturation)

#### Orientation (`types/orientation.rs`)

Provides orientation and direction types:
- `Orientation`: Horizontal or vertical orientation
- `Direction`: Cardinal directions (north, south, east, west)

These types are used for layout and UI components, with helper methods for common operations.

### Configuration Module

The Configuration module (`config/`) provides functionality for loading, parsing, and accessing configuration:

- `CoreConfig`: Root configuration structure
- `LoggingConfig`: Logging-specific configuration
- `ApplicationConfig`: Application-specific configuration
- `SystemConfig`: System-specific configuration

Key interfaces:
- `ConfigLoader`: Interface for loading configuration
- `ConfigProvider`: Interface for accessing configuration

Implementations:
- `FileConfigLoader`: Loads configuration from TOML files
- `FileConfigProvider`: Provides access to configuration loaded from files

The module includes default values for all configuration settings, ensuring the system works well out of the box.

### Logging Module

The Logging module (`logging.rs`) provides structured logging using the `tracing` framework:

- `initialize_logging()`: Sets up the logging system
- `is_initialized()`: Checks if logging has been initialized

Features:
- Console and file logging
- Configurable log levels
- One-time initialization to prevent duplicate setup

### Utilities Module

The Utilities module (`utils/`) provides various utility functions:

#### Async Utilities (`utils/async_utils.rs`)

Asynchronous utilities built on Tokio:
- `spawn_task()`: Spawns a task on the Tokio runtime
- `timeout()`: Runs a future with a timeout
- `sleep()`: Sleeps for a specified duration
- `interval()`: Creates a repeating interval

#### File Utilities (`utils/file_utils.rs`)

File-related utilities:
- `ensure_directory_exists()`: Creates directories if they don't exist
- `read_file_to_string()`: Reads a file to a string
- `write_string_to_file()`: Writes a string to a file
- `copy_file()`: Copies a file from source to destination
- `get_all_files()`: Gets all files in a directory recursively
- `get_file_extension()`: Gets the file extension
- `get_file_stem()`: Gets the file name without extension

#### String Utilities (`utils/string_utils.rs`)

String-related utilities:
- `truncate_string()`: Truncates a string with ellipsis
- `format_bytes()`: Formats byte counts as human-readable strings
- `to_snake_case()`: Converts a string to snake_case
- `to_camel_case()`: Converts a string to camelCase
- `to_pascal_case()`: Converts a string to PascalCase
- `to_kebab_case()`: Converts a string to kebab-case

## Domain Layer

The Domain Layer builds upon the Core Layer and implements the business logic of the desktop environment. It defines the rules and behaviors that govern how the system operates.

### Workspace Management

The Workspace Management module (`workspaces/`) handles the organization of windows into workspaces:

- `Workspace`: Represents a single workspace
- `WorkspaceManagerService`: Interface for managing workspaces
- `DefaultWorkspaceManager`: Default implementation of the workspace manager

Features:
- Window assignment to workspaces
- Workspace switching
- Workspace configuration

### Theming System

The Theming System module (`theming/`) provides a flexible theming engine:

- `ThemingEngine`: Interface for the theming engine
- Token and theme types
- Token resolution pipeline
- Theme application logic

Features:
- Token-based theming
- Theme switching
- Default theme definitions

### Global Settings

The Global Settings module (`global_settings_and_state_management/`) manages application-wide settings:

- `GlobalSettingsService`: Interface for accessing and modifying settings
- Setting types and paths
- Persistence interface

Features:
- Settings change notifications
- Persistent storage
- Default values

### Window Policy Engine

The Window Policy Engine module (`window_policy_engine/`) defines rules for window management:

- `WindowManagementPolicyService`: Interface for window policies
- Policy types and rules
- Policy application logic

Features:
- Window placement rules
- Focus behavior rules
- Window state management

### Notification Management

The Notification Management module (`notifications_core/` and `notifications_rules/`) handles system notifications:

- `NotificationService`: Interface for notification management
- `NotificationRulesEngine`: Interface for notification rules
- Notification types and rules
- Persistence interface

Features:
- Notification display rules
- Notification grouping
- Do-not-disturb mode

## System Layer

The System Layer integrates with the underlying system and hardware, providing a bridge between the domain logic and the physical devices.

### Compositor

The Compositor module (`compositor/`) manages the rendering of windows and surfaces:

- `DesktopState`: Represents the current state of the desktop
- `ClientCompositorData`: Client-specific compositor data
- Surface management
- Renderer interfaces

Protocols:
- XDG Shell implementation
- Layer Shell implementation

Renderers:
- DRM/GBM renderer
- Winit renderer

### Input Handling

The Input Handling module (`input/`) manages input devices and events:

- `SeatManager`: Manages input seats
- Keyboard handling
- Pointer and touch handling
- Gesture recognition

Features:
- libinput integration
- XKB configuration
- Focus management

### D-Bus Interfaces

The D-Bus Interfaces module (`dbus_interfaces/`) provides D-Bus communication:

- `DBusConnectionManager`: Manages D-Bus connections
- Interface definitions
- Method handlers

### Event Bridge

The Event Bridge module (`event_bridge.rs`) bridges events between different parts of the system:

- `SystemEventBridge`: Bridges system events to the domain layer
- `SystemLayerEvent`: System-level events

## UI Layer

The UI Layer provides the visual presentation and user interaction components of the desktop environment.

### Application

The Application module (`application.rs`) defines the main application:

- `NovaApplication`: The main application class
- Application initialization
- Event handling

### Shell Components

The Shell Components modules implement the main UI elements:

- Panel widget
- App menu button
- Workspace indicator
- Clock widget
- Smart tab bar
- Quick settings panel
- Workspace switcher
- Quick action dock

### Widgets

The Widgets module (`widgets/`) provides reusable UI components:

- Custom buttons
- Sliders
- Toggles
- Cards
- Lists

### Theming GTK

The Theming GTK module (`theming_gtk/`) applies themes to GTK widgets:

- CSS provider
- Theme switching
- Widget styling

## Design Decisions

### Error Handling Strategy

The project uses the `thiserror` crate for defining error types, which provides:
- Compile-time checking of error handling
- Clear and structured error messages
- Easy conversion between error types

This approach was chosen over alternatives like `anyhow` because it provides more structure and type safety, which is important for a large project with many different error types.

### Asynchronous Programming

The project uses Tokio for asynchronous programming, which provides:
- A mature and well-tested runtime
- Comprehensive set of asynchronous primitives
- Good performance characteristics

Asynchronous programming is used primarily for I/O operations and event handling, allowing the system to remain responsive even when performing potentially blocking operations.

### Configuration Management

The configuration system is designed to be:
- Flexible: Supporting multiple sources (files, environment variables, etc.)
- Type-safe: Using strongly typed configuration structures
- Default-friendly: Providing sensible defaults for all settings

This approach ensures that the system can be easily configured while maintaining robustness and type safety.

### Modular Architecture

The modular architecture was chosen to:
- Improve maintainability by separating concerns
- Enable independent development and testing of components
- Allow for future extensibility
- Facilitate code reuse

Each module has a clear responsibility and well-defined interfaces, making the system easier to understand and modify.

## Implementation Notes

### Testing Strategy

The project uses a comprehensive testing strategy:
- Unit tests for individual functions and methods
- Integration tests for module interactions
- Property-based tests for complex behaviors
- Mock objects for external dependencies

All tests are written using the standard Rust testing framework, with additional support from crates like `mockall` for mocking and `proptest` for property-based testing.

### Documentation Standards

All code is documented following these standards:
- Module-level documentation explaining the purpose and structure
- Type and function documentation explaining behavior and usage
- Examples for complex or non-obvious functionality
- Cross-references to related types and functions

### Performance Considerations

Performance-critical parts of the system, such as the compositor and input handling, are designed with performance in mind:
- Minimizing allocations in hot paths
- Using efficient data structures
- Avoiding unnecessary copies
- Leveraging parallelism where appropriate

## Future Work

### Planned Features

- Multi-monitor support
- Accessibility improvements
- Plugin system for extensions
- Additional theme options
- Performance optimizations

### Known Limitations

- Limited hardware acceleration support
- No support for remote displays
- Limited internationalization

## Conclusion

The NovaDE desktop environment is designed to be a modern, user-friendly, and performant desktop environment. Its modular architecture and comprehensive documentation make it easy to understand, modify, and extend.

The implementation follows best practices for Rust development, with a focus on safety, performance, and maintainability. The comprehensive test suite ensures that the system behaves correctly and remains stable as it evolves.
