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
    - `DefaultPowerManagementService` struct: Default implementation
  - `types.rs`:
    - `PowerState` enum: System power states
    - `BatteryInfo` struct: Battery information
    - `PowerProfile` enum: Power profiles
    - `PowerEvent` enum: Power-related events
  - `errors.rs`:
    - `PowerManagementError` enum: Power-specific errors
  - `upower_integration.rs`:
    - UPower integration for battery monitoring
  - `logind_integration.rs`:
    - logind integration for system power control
- **Dependencies**: `novade-core`, `novade-domain`, `dbus_interfaces`

### 3.8 Network Management Module (`network_management/`)
- **Purpose**: Handle network connections
- **Components**:
  - `mod.rs`: Re-exports public API
  - `service.rs`:
    - `NetworkManagementService` trait: Interface for network management
    - `DefaultNetworkManagementService` struct: Default implementation
  - `types.rs`:
    - `NetworkDevice` struct: Network device information
    - `ConnectionInfo` struct: Connection information
    - `NetworkEvent` enum: Network-related events
  - `errors.rs`:
    - `NetworkManagementError` enum: Network-specific errors
  - `nm_integration.rs`:
    - NetworkManager integration
- **Dependencies**: `novade-core`, `novade-domain`, `dbus_interfaces`

### 3.9 Display Management Module (`display_management/`)
- **Purpose**: Handle display configuration
- **Components**:
  - `mod.rs`: Re-exports public API
  - `service.rs`:
    - `DisplayManagementService` trait: Interface for display management
    - `DefaultDisplayManagementService` struct: Default implementation
  - `types.rs`:
    - `DisplayDevice` struct: Display device information
    - `DisplayMode` struct: Display mode information
    - `DisplayConfiguration` struct: Display configuration
    - `DisplayEvent` enum: Display-related events
  - `errors.rs`:
    - `DisplayManagementError` enum: Display-specific errors
  - `drm_integration.rs`:
    - DRM integration for display management
  - `xrandr_integration.rs`:
    - XRandR integration for X11 display management
- **Dependencies**: `novade-core`, `novade-domain`, `drm-rs`, `x11rb`

### 3.10 Bluetooth Management Module (`bluetooth_management/`)
- **Purpose**: Handle Bluetooth devices
- **Components**:
  - `mod.rs`: Re-exports public API
  - `service.rs`:
    - `BluetoothManagementService` trait: Interface for Bluetooth management
    - `DefaultBluetoothManagementService` struct: Default implementation
  - `types.rs`:
    - `BluetoothDevice` struct: Bluetooth device information
    - `BluetoothEvent` enum: Bluetooth-related events
  - `errors.rs`:
    - `BluetoothManagementError` enum: Bluetooth-specific errors
  - `bluez_integration.rs`:
    - BlueZ integration for Bluetooth management
- **Dependencies**: `novade-core`, `novade-domain`, `dbus_interfaces`

## 4. UI Layer (`novade-ui`)

### 4.1 Common UI Module (`common_ui/`)
- **Purpose**: Provide common UI components and utilities
- **Components**:
  - `mod.rs`: Re-exports public API
  - `widgets/`:
    - `button.rs`: Button widget
    - `entry.rs`: Text entry widget
    - `switch.rs`: Toggle switch widget
    - `slider.rs`: Slider widget
    - `dropdown.rs`: Dropdown widget
    - `list.rs`: List widget
    - `grid.rs`: Grid widget
    - `card.rs`: Card widget
    - `dialog.rs`: Dialog widget
    - `toast.rs`: Toast notification widget
    - `tooltip.rs`: Tooltip widget
    - `menu.rs`: Menu widget
    - `popover.rs`: Popover widget
    - `avatar.rs`: Avatar widget
    - `progress.rs`: Progress indicator widget
    - `spinner.rs`: Spinner widget
    - `icon.rs`: Icon widget
    - `label.rs`: Label widget
    - `separator.rs`: Separator widget
    - `scrollable.rs`: Scrollable container widget
  - `layout/`:
    - `box.rs`: Box layout
    - `grid.rs`: Grid layout
    - `stack.rs`: Stack layout
    - `flow.rs`: Flow layout
    - `responsive.rs`: Responsive layout
  - `animation/`:
    - `transition.rs`: Transition animations
    - `easing.rs`: Easing functions
    - `timeline.rs`: Animation timeline
  - `theming/`:
    - `style_provider.rs`: Style provider
    - `css_loader.rs`: CSS loader
    - `theme_manager.rs`: Theme manager
  - `accessibility/`:
    - `a11y_manager.rs`: Accessibility manager
    - `screen_reader.rs`: Screen reader integration
    - `keyboard_navigation.rs`: Keyboard navigation
  - `utils/`:
    - `geometry.rs`: Geometry utilities
    - `color.rs`: Color utilities
    - `text.rs`: Text utilities
    - `icon.rs`: Icon utilities
    - `clipboard.rs`: Clipboard utilities
- **Dependencies**: `novade-core`, `novade-domain`, `gtk4`, `libadwaita`

### 4.2 Desktop UI Module (`desktop_ui/`)
- **Purpose**: Implement the desktop shell UI
- **Components**:
  - `mod.rs`: Re-exports public API
  - `desktop.rs`:
    - `DesktopShell` struct: Main desktop shell
    - `DesktopBackground` struct: Desktop background
    - `DesktopIcons` struct: Desktop icons
  - `panel.rs`:
    - `Panel` struct: Main panel
    - `PanelApplet` trait: Interface for panel applets
    - `AppMenu` struct: Application menu
    - `Taskbar` struct: Window taskbar
    - `SystemTray` struct: System tray
    - `Clock` struct: Clock applet
    - `Workspace` struct: Workspace switcher applet
  - `dock.rs`:
    - `Dock` struct: Application dock
    - `DockItem` struct: Dock item
  - `launcher.rs`:
    - `ApplicationLauncher` struct: Application launcher
    - `SearchProvider` trait: Interface for search providers
    - `ApplicationSearchProvider` struct: Application search provider
    - `FileSearchProvider` struct: File search provider
    - `WebSearchProvider` struct: Web search provider
  - `notifications.rs`:
    - `NotificationCenter` struct: Notification center
    - `NotificationPopup` struct: Notification popup
  - `quick_settings.rs`:
    - `QuickSettings` struct: Quick settings panel
    - `QuickSettingsTile` struct: Quick settings tile
  - `workspace_view.rs`:
    - `WorkspaceView` struct: Workspace view
    - `WorkspaceSwitcher` struct: Workspace switcher
  - `window_switcher.rs`:
    - `WindowSwitcher` struct: Window switcher
    - `WindowPreview` struct: Window preview
  - `calendar.rs`:
    - `Calendar` struct: Calendar widget
    - `CalendarEvent` struct: Calendar event
  - `weather.rs`:
    - `Weather` struct: Weather widget
    - `WeatherProvider` trait: Interface for weather providers
  - `media_controls.rs`:
    - `MediaControls` struct: Media controls widget
    - `MediaPlayer` struct: Media player interface
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `common_ui`

### 4.3 Settings UI Module (`settings_ui/`)
- **Purpose**: Implement the settings application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `settings_window.rs`:
    - `SettingsWindow` struct: Main settings window
    - `SettingsPage` trait: Interface for settings pages
  - `appearance.rs`:
    - `AppearancePage` struct: Appearance settings page
    - `ThemeSelector` struct: Theme selector
    - `FontSelector` struct: Font selector
    - `ColorSelector` struct: Color selector
  - `desktop.rs`:
    - `DesktopPage` struct: Desktop settings page
    - `BackgroundSelector` struct: Background selector
    - `LayoutSelector` struct: Layout selector
  - `display.rs`:
    - `DisplayPage` struct: Display settings page
    - `DisplayArrangement` struct: Display arrangement widget
    - `DisplaySettings` struct: Display settings widget
  - `network.rs`:
    - `NetworkPage` struct: Network settings page
    - `WifiSettings` struct: Wi-Fi settings
    - `EthernetSettings` struct: Ethernet settings
    - `VpnSettings` struct: VPN settings
  - `bluetooth.rs`:
    - `BluetoothPage` struct: Bluetooth settings page
    - `DeviceList` struct: Device list
    - `PairingDialog` struct: Pairing dialog
  - `sound.rs`:
    - `SoundPage` struct: Sound settings page
    - `OutputSettings` struct: Output settings
    - `InputSettings` struct: Input settings
    - `SoundEffectsSettings` struct: Sound effects settings
  - `power.rs`:
    - `PowerPage` struct: Power settings page
    - `PowerModeSelector` struct: Power mode selector
    - `BatteryInfo` struct: Battery information widget
  - `users.rs`:
    - `UsersPage` struct: Users settings page
    - `UserEditor` struct: User editor
    - `PasswordDialog` struct: Password dialog
  - `privacy.rs`:
    - `PrivacyPage` struct: Privacy settings page
    - `LocationSettings` struct: Location settings
    - `UsageSettings` struct: Usage settings
    - `ConsentManager` struct: Consent manager
  - `about.rs`:
    - `AboutPage` struct: About settings page
    - `SystemInfo` struct: System information
    - `SoftwareUpdates` struct: Software updates
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `common_ui`

### 4.4 Session UI Module (`session_ui/`)
- **Purpose**: Implement session management UI
- **Components**:
  - `mod.rs`: Re-exports public API
  - `login.rs`:
    - `LoginScreen` struct: Login screen
    - `UserSelector` struct: User selector
    - `PasswordEntry` struct: Password entry
    - `SessionSelector` struct: Session selector
  - `lock.rs`:
    - `LockScreen` struct: Lock screen
    - `UnlockDialog` struct: Unlock dialog
  - `session_dialog.rs`:
    - `SessionDialog` struct: Session dialog
    - `PowerOffDialog` struct: Power off dialog
    - `RestartDialog` struct: Restart dialog
    - `LogoutDialog` struct: Logout dialog
    - `SuspendDialog` struct: Suspend dialog
  - `user_switching.rs`:
    - `UserSwitchingDialog` struct: User switching dialog
    - `UserList` struct: User list
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `common_ui`

### 4.5 Window Manager UI Module (`window_manager_ui/`)
- **Purpose**: Implement window manager UI
- **Components**:
  - `mod.rs`: Re-exports public API
  - `window_decorations.rs`:
    - `WindowDecorations` struct: Window decorations
    - `TitleBar` struct: Title bar
    - `WindowControls` struct: Window controls
  - `window_menu.rs`:
    - `WindowMenu` struct: Window menu
    - `WindowMenuItem` struct: Window menu item
  - `window_tiling.rs`:
    - `TilingPreview` struct: Tiling preview
    - `TilingOverlay` struct: Tiling overlay
  - `window_overview.rs`:
    - `WindowOverview` struct: Window overview
    - `WindowThumbnail` struct: Window thumbnail
  - `window_animations.rs`:
    - `WindowAnimations` struct: Window animations
    - `MapAnimation` struct: Map animation
    - `UnmapAnimation` struct: Unmap animation
    - `MinimizeAnimation` struct: Minimize animation
    - `MaximizeAnimation` struct: Maximize animation
  - `window_effects.rs`:
    - `WindowEffects` struct: Window effects
    - `ShadowEffect` struct: Shadow effect
    - `BlurEffect` struct: Blur effect
    - `TransparencyEffect` struct: Transparency effect
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `common_ui`

### 4.6 Accessibility UI Module (`accessibility_ui/`)
- **Purpose**: Implement accessibility UI
- **Components**:
  - `mod.rs`: Re-exports public API
  - `screen_reader.rs`:
    - `ScreenReaderUI` struct: Screen reader UI
    - `ScreenReaderSettings` struct: Screen reader settings
  - `magnifier.rs`:
    - `Magnifier` struct: Screen magnifier
    - `MagnifierSettings` struct: Magnifier settings
  - `on_screen_keyboard.rs`:
    - `OnScreenKeyboard` struct: On-screen keyboard
    - `KeyboardLayout` struct: Keyboard layout
  - `high_contrast.rs`:
    - `HighContrastSettings` struct: High contrast settings
  - `color_filters.rs`:
    - `ColorFilters` struct: Color filters
    - `ColorFilterSettings` struct: Color filter settings
  - `accessibility_menu.rs`:
    - `AccessibilityMenu` struct: Accessibility menu
    - `AccessibilityMenuItem` struct: Accessibility menu item
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `common_ui`

### 4.7 Notification UI Module (`notification_ui/`)
- **Purpose**: Implement notification UI
- **Components**:
  - `mod.rs`: Re-exports public API
  - `notification_popup.rs`:
    - `NotificationPopup` struct: Notification popup
    - `NotificationActions` struct: Notification actions
  - `notification_center.rs`:
    - `NotificationCenter` struct: Notification center
    - `NotificationList` struct: Notification list
    - `NotificationGroup` struct: Notification group
  - `do_not_disturb.rs`:
    - `DoNotDisturbToggle` struct: Do not disturb toggle
    - `DoNotDisturbSettings` struct: Do not disturb settings
  - `notification_settings.rs`:
    - `NotificationSettings` struct: Notification settings
    - `ApplicationNotificationSettings` struct: Application notification settings
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `common_ui`

### 4.8 System Integration UI Module (`system_integration_ui/`)
- **Purpose**: Implement system integration UI
- **Components**:
  - `mod.rs`: Re-exports public API
  - `volume_control.rs`:
    - `VolumeControl` struct: Volume control
    - `VolumeSlider` struct: Volume slider
    - `OutputSelector` struct: Output selector
  - `brightness_control.rs`:
    - `BrightnessControl` struct: Brightness control
    - `BrightnessSlider` struct: Brightness slider
  - `network_indicator.rs`:
    - `NetworkIndicator` struct: Network indicator
    - `WifiList` struct: Wi-Fi list
    - `ConnectionInfo` struct: Connection information
  - `bluetooth_indicator.rs`:
    - `BluetoothIndicator` struct: Bluetooth indicator
    - `DeviceList` struct: Device list
  - `battery_indicator.rs`:
    - `BatteryIndicator` struct: Battery indicator
    - `PowerInfo` struct: Power information
  - `system_monitor.rs`:
    - `SystemMonitor` struct: System monitor
    - `CpuGraph` struct: CPU graph
    - `MemoryGraph` struct: Memory graph
    - `DiskGraph` struct: Disk graph
    - `NetworkGraph` struct: Network graph
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `common_ui`

## 5. Application Layer (`novade-applications`)

### 5.1 File Manager (`file_manager/`)
- **Purpose**: Implement a file manager application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `application.rs`:
    - `FileManagerApplication` struct: Main application
    - `FileManagerWindow` struct: Main window
  - `views/`:
    - `icon_view.rs`: Icon view
    - `list_view.rs`: List view
    - `column_view.rs`: Column view
    - `tree_view.rs`: Tree view
  - `operations/`:
    - `file_operations.rs`: File operations
    - `search.rs`: File search
    - `trash.rs`: Trash management
  - `sidebar.rs`:
    - `Sidebar` struct: Sidebar
    - `PlacesList` struct: Places list
  - `properties.rs`:
    - `PropertiesDialog` struct: Properties dialog
  - `preferences.rs`:
    - `PreferencesDialog` struct: Preferences dialog
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `novade-ui`

### 5.2 Text Editor (`text_editor/`)
- **Purpose**: Implement a text editor application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `application.rs`:
    - `TextEditorApplication` struct: Main application
    - `TextEditorWindow` struct: Main window
  - `editor.rs`:
    - `Editor` struct: Editor widget
    - `EditorBuffer` struct: Editor buffer
  - `syntax_highlighting.rs`:
    - `SyntaxHighlighter` struct: Syntax highlighter
    - `SyntaxTheme` struct: Syntax theme
  - `search_replace.rs`:
    - `SearchReplace` struct: Search and replace
  - `preferences.rs`:
    - `PreferencesDialog` struct: Preferences dialog
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `novade-ui`

### 5.3 Terminal Emulator (`terminal/`)
- **Purpose**: Implement a terminal emulator application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `application.rs`:
    - `TerminalApplication` struct: Main application
    - `TerminalWindow` struct: Main window
  - `terminal.rs`:
    - `Terminal` struct: Terminal widget
    - `TerminalBuffer` struct: Terminal buffer
  - `vte.rs`:
    - VTE integration
  - `preferences.rs`:
    - `PreferencesDialog` struct: Preferences dialog
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `novade-ui`, `vte-rs`

### 5.4 System Monitor (`system_monitor/`)
- **Purpose**: Implement a system monitor application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `application.rs`:
    - `SystemMonitorApplication` struct: Main application
    - `SystemMonitorWindow` struct: Main window
  - `process_view.rs`:
    - `ProcessView` struct: Process view
    - `ProcessList` struct: Process list
  - `resource_view.rs`:
    - `ResourceView` struct: Resource view
    - `CpuGraph` struct: CPU graph
    - `MemoryGraph` struct: Memory graph
    - `DiskGraph` struct: Disk graph
    - `NetworkGraph` struct: Network graph
  - `file_systems.rs`:
    - `FileSystemsView` struct: File systems view
  - `preferences.rs`:
    - `PreferencesDialog` struct: Preferences dialog
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `novade-ui`

### 5.5 Image Viewer (`image_viewer/`)
- **Purpose**: Implement an image viewer application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `application.rs`:
    - `ImageViewerApplication` struct: Main application
    - `ImageViewerWindow` struct: Main window
  - `image_view.rs`:
    - `ImageView` struct: Image view widget
  - `slideshow.rs`:
    - `Slideshow` struct: Slideshow
  - `editing.rs`:
    - `ImageEditing` struct: Image editing
  - `preferences.rs`:
    - `PreferencesDialog` struct: Preferences dialog
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `novade-ui`

### 5.6 Calculator (`calculator/`)
- **Purpose**: Implement a calculator application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `application.rs`:
    - `CalculatorApplication` struct: Main application
    - `CalculatorWindow` struct: Main window
  - `calculator.rs`:
    - `Calculator` struct: Calculator widget
    - `CalculatorEngine` struct: Calculator engine
  - `history.rs`:
    - `History` struct: Calculation history
  - `preferences.rs`:
    - `PreferencesDialog` struct: Preferences dialog
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `novade-ui`

### 5.7 Calendar (`calendar/`)
- **Purpose**: Implement a calendar application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `application.rs`:
    - `CalendarApplication` struct: Main application
    - `CalendarWindow` struct: Main window
  - `calendar_view.rs`:
    - `CalendarView` struct: Calendar view widget
    - `MonthView` struct: Month view
    - `WeekView` struct: Week view
    - `DayView` struct: Day view
  - `events.rs`:
    - `EventEditor` struct: Event editor
    - `EventList` struct: Event list
  - `preferences.rs`:
    - `PreferencesDialog` struct: Preferences dialog
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `novade-ui`

### 5.8 Weather (`weather/`)
- **Purpose**: Implement a weather application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `application.rs`:
    - `WeatherApplication` struct: Main application
    - `WeatherWindow` struct: Main window
  - `weather_view.rs`:
    - `WeatherView` struct: Weather view widget
    - `CurrentWeather` struct: Current weather
    - `Forecast` struct: Weather forecast
  - `locations.rs`:
    - `LocationManager` struct: Location manager
    - `LocationSearch` struct: Location search
  - `providers.rs`:
    - `WeatherProvider` trait: Interface for weather providers
    - `DefaultWeatherProvider` struct: Default implementation
  - `preferences.rs`:
    - `PreferencesDialog` struct: Preferences dialog
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `novade-ui`

### 5.9 Notes (`notes/`)
- **Purpose**: Implement a notes application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `application.rs`:
    - `NotesApplication` struct: Main application
    - `NotesWindow` struct: Main window
  - `note_editor.rs`:
    - `NoteEditor` struct: Note editor widget
  - `note_list.rs`:
    - `NoteList` struct: Note list
  - `storage.rs`:
    - `NoteStorage` trait: Interface for note storage
    - `FileNoteStorage` struct: File-based implementation
  - `preferences.rs`:
    - `PreferencesDialog` struct: Preferences dialog
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `novade-ui`

### 5.10 Music Player (`music_player/`)
- **Purpose**: Implement a music player application
- **Components**:
  - `mod.rs`: Re-exports public API
  - `application.rs`:
    - `MusicPlayerApplication` struct: Main application
    - `MusicPlayerWindow` struct: Main window
  - `player.rs`:
    - `Player` struct: Player widget
    - `PlaybackControls` struct: Playback controls
  - `library.rs`:
    - `MusicLibrary` struct: Music library
    - `AlbumView` struct: Album view
    - `ArtistView` struct: Artist view
    - `SongView` struct: Song view
  - `playlists.rs`:
    - `PlaylistManager` struct: Playlist manager
    - `PlaylistEditor` struct: Playlist editor
  - `preferences.rs`:
    - `PreferencesDialog` struct: Preferences dialog
- **Dependencies**: `novade-core`, `novade-domain`, `novade-system`, `novade-ui`, `gstreamer-rs`
