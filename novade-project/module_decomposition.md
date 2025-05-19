# Project Module Decomposition

This document provides a detailed decomposition of the project into specific modules and components, based on the requirements analysis. Each module is broken down into its constituent components with clear responsibilities and dependencies.

## 1. Core Layer (`novade-core`)

### 1.1 Error Module (`error.rs`)
- **Purpose**: Provide standardized error handling across the entire project
- **Components**:
  - `CoreError` enum: Base error type for the entire project
  - `ConfigError` enum: Configuration-specific errors
  - `LoggingError` enum: Logging-specific errors
  - Error conversion traits: For converting between error types
- **Dependencies**: `thiserror` crate

### 1.2 Types Module (`types/`)
- **Purpose**: Define fundamental data types used throughout the project
- **Components**:
  - `geometry.rs`: 
    - `Point<T>` struct: Represents a 2D point with generic coordinate type
    - `Size<T>` struct: Represents dimensions with generic coordinate type
    - `Rect<T>` struct: Represents a rectangle with generic coordinate type
    - `RectInt` type alias: Integer-based rectangle for pixel operations
  - `color.rs`:
    - `Color` struct: RGBA color representation
    - `ColorFormat` enum: Different color formats (RGB, RGBA, HSL, etc.)
    - Color conversion functions
  - `orientation.rs`:
    - `Orientation` enum: Horizontal/Vertical orientation
    - `Direction` enum: North/South/East/West directions
  - `mod.rs`: Re-exports all type definitions
- **Dependencies**: Rust standard library

### 1.3 Configuration Module (`config/`)
- **Purpose**: Handle loading, parsing, and accessing configuration
- **Components**:
  - `mod.rs`:
    - `CoreConfig` struct: Root configuration structure
    - `ConfigLoader` trait: Interface for loading configuration
    - `ConfigProvider` trait: Interface for accessing configuration
  - `defaults.rs`:
    - Default configuration values
    - Functions for creating default configurations
  - `file_loader.rs`:
    - `FileConfigLoader` struct: Loads configuration from files
    - TOML parsing and validation logic
- **Dependencies**: `serde`, `toml`, `once_cell` crates

### 1.4 Logging Module (`logging.rs`)
- **Purpose**: Initialize and configure the logging framework
- **Components**:
  - `LoggingConfig` struct: Configuration for logging
  - `initialize_logging()` function: Set up the tracing framework
  - Log level filters and formatters
  - Context propagation utilities
- **Dependencies**: `tracing`, `tracing-subscriber` crates

### 1.5 Utilities Module (`utils/`)
- **Purpose**: Provide common utility functions
- **Components**:
  - `async_utils.rs`:
    - Async helper functions
    - Task management utilities
  - `file_utils.rs`:
    - File system operations
    - Path manipulation functions
  - `string_utils.rs`:
    - String manipulation utilities
    - Text processing functions
  - `mod.rs`: Re-exports utility functions
- **Dependencies**: Rust standard library, `tokio` for async utilities

## 2. Domain Layer (`novade-domain`)

### 2.1 Theming Module (`theming/`)
- **Purpose**: Manage the appearance and styling of the desktop environment
- **Components**:
  - `mod.rs`: Re-exports public API
  - `service.rs`:
    - `ThemingEngine` trait: Interface for theming services
    - `DefaultThemingEngine` struct: Default implementation
  - `types.rs`:
    - `TokenIdentifier` struct: Identifier for design tokens
    - `TokenValue` enum: Different types of token values
    - `RawToken` struct: Raw token definition
    - `TokenSet` type: Collection of tokens
    - `ThemeIdentifier` struct: Identifier for themes
    - `ColorSchemeType` enum: Light/Dark variants
    - `AccentColor` struct: Accent color definition
    - `ThemeVariantDefinition` struct: Theme variant definition
    - `ThemeDefinition` struct: Complete theme definition
    - `AppliedThemeState` struct: Resolved theme state
    - `ThemingConfiguration` struct: User theming preferences
  - `logic.rs`:
    - Token loading and parsing functions
    - Token validation functions
    - Token resolution pipeline
    - Theme application logic
  - `errors.rs`:
    - `ThemingError` enum: Theming-specific errors
  - `events.rs`:
    - `ThemeChangedEvent` struct: Event for theme changes
  - `default_themes/`: Default theme definitions in JSON format
- **Dependencies**: `novade-core`, `serde`, `serde_json`

### 2.2 Workspace Management Module (`workspaces/`)
- **Purpose**: Manage virtual desktops and window organization
- **Components**:
  - `core/`:
    - `mod.rs`: 
      - `Workspace` struct: Core workspace entity
      - Methods for workspace manipulation
    - `types.rs`:
      - `WorkspaceId` struct: Unique workspace identifier
      - `WindowIdentifier` struct: Window identifier
      - `WorkspaceLayoutType` enum: Layout modes
    - `errors.rs`:
      - `WorkspaceCoreError` enum: Core workspace errors
    - `event_data.rs`:
      - Event payload structures
  - `assignment/`:
    - `mod.rs`:
      - Window assignment logic
      - Rules for automatic window placement
    - `errors.rs`:
      - `WindowAssignmentError` enum: Assignment-specific errors
  - `manager/`:
    - `mod.rs`:
      - `WorkspaceManagerService` trait: Interface for workspace management
      - `DefaultWorkspaceManager` struct: Default implementation
    - `errors.rs`:
      - `WorkspaceManagerError` enum: Manager-specific errors
    - `events.rs`:
      - `WorkspaceEvent` enum: Workspace-related events
  - `config/`:
    - `mod.rs`:
      - `WorkspaceSnapshot` struct: Serializable workspace state
      - `WorkspaceSetSnapshot` struct: Collection of workspace snapshots
      - `WorkspaceConfigProvider` trait: Interface for configuration
      - `FilesystemConfigProvider` struct: File-based implementation
    - `errors.rs`:
      - `WorkspaceConfigError` enum: Configuration-specific errors
- **Dependencies**: `novade-core`, `uuid`, `chrono`

### 2.3 AI Interaction Module (`user_centric_services/ai_interaction/`)
- **Purpose**: Manage AI-powered features and user consent
- **Components**:
  - `mod.rs`: Re-exports public API
  - `service.rs`:
    - `AIInteractionLogicService` trait: Interface for AI features
    - `DefaultAIInteractionService` struct: Default implementation
  - `types.rs`:
    - `UserConsent` struct: User consent for AI features
    - `AIRequestContext` struct: Context for AI requests
    - `AISuggestion<T>` struct: Generic AI suggestion
    - Specific payload types for different AI features
  - `errors.rs`:
    - `AIConsentError` enum: Consent-related errors
    - `AIFeatureError` enum: Feature-specific errors
  - `events.rs`:
    - `AIConsentEvent` enum: Consent-related events
    - `AIFeatureEvent` enum: Feature-related events
  - `persistence_iface.rs`:
    - `ConsentPersistencePort` trait: Interface for consent storage
    - `AIModelProfileProvider` trait: Interface for model profiles
- **Dependencies**: `novade-core`

### 2.4 Notification Management Module (`notifications_core/`)
- **Purpose**: Handle system and application notifications
- **Components**:
  - `mod.rs`: Re-exports public API
  - `service.rs`:
    - `NotificationService` trait: Interface for notification management
    - `DefaultNotificationService` struct: Default implementation
  - `types.rs`:
    - `Notification` struct: Core notification entity
    - `NotificationAction` struct: Action within a notification
    - `NotificationUrgency` enum: Urgency levels
    - `NotificationCloseReason` enum: Reasons for closing
  - `errors.rs`:
    - `NotificationError` enum: Notification-specific errors
  - `events.rs`:
    - `NotificationEvent` enum: Notification-related events
- **Dependencies**: `novade-core`, `chrono`

### 2.5 Notification Rules Module (`notifications_rules/`)
- **Purpose**: Apply rules to process and filter notifications
- **Components**:
  - `mod.rs`: Re-exports public API
  - `engine.rs`:
    - `NotificationRulesEngine` trait: Interface for rules engine
    - `DefaultNotificationRulesEngine` struct: Default implementation
  - `types.rs`:
    - `RuleCondition` enum: Conditions for rule matching
    - `RuleAction` enum: Actions to take on match
    - `NotificationRule` struct: Complete rule definition
  - `errors.rs`:
    - `NotificationRulesError` enum: Rules-specific errors
  - `persistence_iface.rs`:
    - `NotificationRulesProvider` trait: Interface for rule storage
- **Dependencies**: `novade-core`, `notifications_core`

### 2.6 Global Settings Module (`global_settings_and_state_management/`)
- **Purpose**: Manage desktop-wide settings and state
- **Components**:
  - `mod.rs`: Re-exports public API
  - `service.rs`:
    - `GlobalSettingsService` trait: Interface for settings management
    - `DefaultGlobalSettingsService` struct: Default implementation
  - `types.rs`:
    - `GlobalDesktopSettings` struct: Root settings structure
    - Various settings structs and enums
  - `paths.rs`:
    - `SettingPath` enum: Hierarchical setting paths
  - `errors.rs`:
    - `GlobalSettingsError` enum: Settings-specific errors
  - `events.rs`:
    - `SettingChangedEvent` struct: Event for setting changes
    - `SettingsLoadedEvent` struct: Event for settings loaded
  - `persistence_iface.rs`:
    - `SettingsPersistenceProvider` trait: Interface for settings storage
- **Dependencies**: `novade-core`, `serde`

### 2.7 Window Management Policy Module (`window_policy_engine/`)
- **Purpose**: Define high-level window management policies
- **Components**:
  - `mod.rs`: Re-exports public API
  - `service.rs`:
    - `WindowManagementPolicyService` trait: Interface for policy service
    - `DefaultWindowPolicyService` struct: Default implementation
  - `types.rs`:
    - `TilingMode` enum: Different tiling strategies
    - `GapSettings` struct: Settings for gaps between windows
    - `WorkspaceWindowLayout` struct: Layout configuration
  - `errors.rs`:
    - `WindowPolicyError` enum: Policy-specific errors
- **Dependencies**: `novade-core`, `workspaces`

### 2.8 Common Events Module (`common_events/`)
- **Purpose**: Define events shared across domain modules
- **Components**:
  - `mod.rs`:
    - `UserActivityDetectedEvent` struct: User activity event
    - Other common event definitions
- **Dependencies**: `novade-core`

### 2.9 Shared Types Module (`shared_types/`)
- **Purpose**: Define types used across domain modules
- **Components**:
  - `mod.rs`:
    - `ApplicationId` struct: Application identifier
    - `UserSessionState` enum: Session state
    - Other shared type definitions
- **Dependencies**: `novade-core`

## 3. System Layer (`novade-system`)

### 3.1 Compositor Module (`compositor/`)
- **Purpose**: Implement the Wayland compositor
- **Components**:
  - `mod.rs`: Re-exports public API
  - `core/`:
    - `state.rs`:
      - `DesktopState` struct: Core compositor state
      - `ClientCompositorData` struct: Per-client data
    - `errors.rs`:
      - `CompositorCoreError` enum: Core compositor errors
  - `surface_management.rs`:
    - `SurfaceData` struct: Surface metadata
    - `AttachedBufferInfo` struct: Buffer information
  - `xdg_shell/`:
    - `mod.rs`: Re-exports XDG shell components
    - `types.rs`:
      - `ManagedWindow` struct: Window implementation
    - `handlers.rs`:
      - XDG shell protocol handlers
    - `errors.rs`:
      - `XdgShellError` enum: XDG shell errors
  - `layer_shell/`: Layer shell implementation
  - `decoration/`: Window decoration implementation
  - `output_management/`: Output management implementation
  - `renderer_interface.rs`:
    - `FrameRenderer` trait: Interface for renderers
    - `RenderableTexture` trait: Interface for textures
  - `renderers/`:
    - `drm_gbm_renderer.rs`: DRM/GBM renderer implementation
    - `winit_renderer.rs`: Winit renderer for development
  - `init.rs`:
    - `initialize_compositor()` function: Setup function
    - Wayland global creation logic
- **Dependencies**: `novade-core`, `novade-domain`, `smithay`

### 3.2 Input Module (`input/`)
- **Purpose**: Handle input devices and events
- **Components**:
  - `mod.rs`: Re-exports public API
  - `errors.rs`:
    - `InputError` enum: Input-specific errors
  - `types.rs`:
    - `XkbKeyboardData` struct: Keyboard state
  - `seat_manager.rs`:
    - `SeatManager` struct: Manages input seats
  - `libinput_handler/`:
    - `mod.rs`: libinput integration
    - `session_interface.rs`: Session interface implementation
  - `keyboard/`:
    - `mod.rs`: Keyboard handling
    - `key_event_translator.rs`: Key event translation
    - `focus.rs`: Keyboard focus management
    - `xkb_config.rs`: XKB configuration
  - `pointer/`: Pointer input handling
  - `touch/`: Touch input handling
  - `gestures/`: Gesture recognition
- **Dependencies**: `novade-core`, `novade-domain`, `smithay`, `libinput`, `xkbcommon`

### 3.3 D-Bus Interfaces Module (`dbus_interfaces/`)
- **Purpose**: Interact with system services via D-Bus
- **Components**:
  - `mod.rs`: Re-exports public API
  - `connection_manager.rs`:
    - `DBusConnectionManager` struct: Manages D-Bus connections
  - `error.rs`:
    - `DBusInterfaceError` enum: D-Bus-specific errors
  - `upower_client/`: UPower client implementation
  - `logind_client/`: logind client implementation
  - `network_manager_client/`: NetworkManager client implementation
  - `notifications_server/`: Notifications server implementation
  - `secrets_service_client/`: Secrets service client implementation
  - `policykit_client/`: PolicyKit client implementation
- **Dependencies**: `novade-core`, `novade-domain`, `zbus`

### 3.4 Audio Management Module (`audio_management/`)
- **Purpose**: Handle audio devices and streams
- **Components**:
  - `mod.rs`: Re-exports public API
  - `client.rs`:
    - `PipeWireClient` struct: PipeWire client implementation
    - `PipeWireLoopData` struct: Loop data for PipeWire
    - `InternalAudioEvent` enum: Internal events
  - `manager.rs`:
    - Logic for processing registry events
    - Device and stream management
  - `control.rs`:
    - Volume and mute control
    - Default device selection
  - `types.rs`:
    - `AudioDevice` struct: Audio device representation
    - `StreamInfo` struct: Audio stream information
    - `AudioCommand` enum: Commands for audio control
    - `AudioEvent` enum: Audio-related events
    - `VolumeCurve` struct: Volume curve definition
  - `spa_pod_utils.rs`:
    - Utilities for SPA pod handling
  - `error.rs`:
    - `AudioError` enum: Audio-specific errors
- **Dependencies**: `novade-core`, `novade-domain`, `pipewire-rs`

### 3.5 MCP Client Module (`mcp_client/`)
- **Purpose**: Implement Model Context Protocol client
- **Components**:
  - `mod.rs`: Re-exports public API
  - `service.rs`:
    - `SystemMcpService` trait: Interface for MCP service
    - `DefaultSystemMcpService` struct: Default implementation
  - `connection_manager.rs`:
    - `McpConnection` struct: Single MCP connection
    - `McpConnectionManager` struct: Manages connections
  - `types.rs`:
    - `McpServerConfig` struct: Server configuration
    - `McpClientSystemEvent` enum: System events for MCP
    - Re-exports from `mcp_client_rs::protocol`
  - `error.rs`:
    - `McpSystemClientError` enum: MCP-specific errors
- **Dependencies**: `novade-core`, `novade-domain`, `mcp_client_rs`

### 3.6 Portals Module (`portals/`)
- **Purpose**: Implement XDG Desktop Portals backend
- **Components**:
  - `mod.rs`: Re-exports public API
  - `file_chooser.rs`: File chooser portal implementation
  - `screenshot.rs`: Screenshot portal implementation
  - `common.rs`:
    - `DesktopPortal` struct: Combined portal structure
    - `run_portal_service()` function: Service startup
  - `error.rs`:
    - `PortalsError` enum: Portal-specific errors
- **Dependencies**: `novade-core`, `novade-domain`, `zbus`

### 3.7 Power Management Module (`power_management/`)
- **Purpose**: Handle power-related features
- **Components**:
  - `mod.rs`: Re-exports public API
  - `service.rs`:
    - `PowerManagementService` trait: Interface for power management
    - `PowerManagementControl` trait: Control interface
    - `DefaultPowerManagementService` struct: Default implementation
  - `types.rs`:
    - `DpmsState` enum: Display power management states
    - `PowerManagementSystemEvent` enum: Power-related events
    - `IdleTimerState` struct: Idle timer state
  - `error.rs`:
    - `PowerManagementError` enum: Power-specific errors
- **Dependencies**: `novade-core`, `novade-domain`

### 3.8 Window Mechanics Module (`window_mechanics/`)
- **Purpose**: Implement technical aspects of window management
- **Components**:
  - `mod.rs`: Re-exports public API
  - `types.rs`:
    - `InteractiveOpState` enum: Interactive operation states
    - `WindowMechanicsEvent` enum: Window mechanics events
  - `errors.rs`:
    - `WindowMechanicsError` enum: Window mechanics errors
  - `layout_applier.rs`:
    - Layout application logic
  - `interactive_ops.rs`:
    - Interactive window operations
  - `focus_manager.rs`:
    - Window focus management
- **Dependencies**: `novade-core`, `novade-domain`, `smithay`

### 3.9 Event Bridge Module (`event_bridge.rs`)
- **Purpose**: Bridge events between system and domain layers
- **Components**:
  - `SystemEventBridge` struct: Event bridge implementation
  - `SystemLayerEvent` enum: System layer events
- **Dependencies**: `novade-core`, `novade-domain`

## 4. UI Layer (`novade-ui`)

### 4.1 Application Module (`application.rs`)
- **Purpose**: Implement the main application
- **Components**:
  - `NovaApplication` struct: GtkApplication subclass
  - Application initialization and setup
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `gtk4`

### 4.2 Shell Module (`shell/`)
- **Purpose**: Implement the main shell UI
- **Components**:
  - `mod.rs`: Re-exports public API
  - `panel_widget/`:
    - `mod.rs`: Panel widget implementation
    - `imp.rs`: GObject implementation
    - `error.rs`: Panel-specific errors
    - `app_menu_button/`: App menu button implementation
    - `workspace_indicator_widget/`: Workspace indicator implementation
    - `clock_datetime_widget/`: Clock widget implementation
  - `smart_tab_bar_widget/`: Tab bar implementation
  - `quick_settings_panel_widget/`: Quick settings panel implementation
  - `workspace_switcher_widget/`: Workspace switcher implementation
  - `quick_action_dock_widget/`: Quick action dock implementation
  - `notification_center_panel_widget/`: Notification center implementation
  - `active_window_service.rs`: Active window tracking service
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `gtk4`

### 4.3 Control Center Module (`control_center/`)
- **Purpose**: Implement the settings application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `main_window.rs`: Main window implementation
  - `settings_panels/`: Various settings panels
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `gtk4`

### 4.4 Widgets Module (`widgets/`)
- **Purpose**: Provide reusable UI components
- **Components**:
  - `mod.rs`: Re-exports public API
  - Various widget implementations
- **Dependencies**: `novade-core`, `gtk4`

### 4.5 Window Manager Frontend Module (`window_manager_frontend/`)
- **Purpose**: Implement UI aspects of window management
- **Components**:
  - `mod.rs`: Re-exports public API
  - `window_decorations/`: Window decoration implementation
  - `overview/`: Window overview implementation
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `gtk4`

### 4.6 Notifications Frontend Module (`notifications_frontend/`)
- **Purpose**: Implement notification UI
- **Components**:
  - `mod.rs`: Re-exports public API
  - `popup/`: Notification popup implementation
  - `center/`: Notification center implementation
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `gtk4`

### 4.7 Theming GTK Module (`theming_gtk/`)
- **Purpose**: Apply theming to GTK
- **Components**:
  - `mod.rs`: Re-exports public API
  - `css_provider.rs`: CSS provider implementation
  - `theme_switcher.rs`: Theme switching implementation
- **Dependencies**: `novade-core`, `novade-domain`, `gtk4`

### 4.8 Portals Client Module (`portals/`)
- **Purpose**: Implement client-side portal interaction
- **Components**:
  - `mod.rs`: Re-exports public API
  - `file_chooser.rs`: File chooser portal client
  - `screenshot.rs`: Screenshot portal client
- **Dependencies**: `novade-core`, `novade-domain`, `ashpd` or `zbus`

### 4.9 Resources Module (`resources/`)
- **Purpose**: Provide UI resources
- **Components**:
  - `resources.xml`: GResource XML definition
  - `ui/`: UI definition files
    - `shell/`: Shell UI definitions
    - `control_center/`: Control center UI definitions
- **Dependencies**: None (resource files)

## 5. Implementation Dependencies

### 5.1 External Dependencies
- **Rust Standard Library**: Core Rust functionality
- **thiserror**: Error definition and handling
- **serde**, **serde_json**, **toml**: Serialization/deserialization
- **tracing**, **tracing-subscriber**: Logging framework
- **uuid**: Unique identifier generation
- **chrono**: Date and time handling
- **tokio**: Asynchronous runtime
- **smithay**: Wayland compositor framework
- **libinput**: Input device handling
- **xkbcommon**: Keyboard mapping
- **zbus**: D-Bus communication
- **pipewire-rs**: PipeWire audio integration
- **mcp_client_rs**: Model Context Protocol client
- **gtk4**: UI framework
- **ashpd**: XDG Desktop Portal client

### 5.2 Internal Dependencies
- **novade-core**: Used by all higher layers
- **novade-domain**: Used by system and UI layers
- **novade-system**: Used by UI layer

## 6. Cross-Cutting Implementation Concerns

### 6.1 Error Handling
- Use `thiserror` for error definitions
- Implement proper error propagation
- Ensure context preservation in errors

### 6.2 Logging
- Use `tracing` for structured logging
- Implement appropriate log levels
- Ensure context propagation

### 6.3 Testing
- Implement unit tests for all modules
- Create integration tests for component interactions
- Develop end-to-end tests for critical paths

### 6.4 Performance
- Optimize critical paths
- Use efficient data structures
- Implement caching where appropriate

### 6.5 Security
- Validate all inputs
- Implement proper authentication
- Follow secure coding practices
