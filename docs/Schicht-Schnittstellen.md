**Grundlegende Kommunikationsprinzipien**

Die Architektur von NovaDE basiert auf fundamentalen Kommunikations- und Strukturprinzipien, die ein robustes, modulares und wartbares System gewährleisten sollen.

1. **API-basierte Interaktion:** Die Kommunikation zwischen den Schichten – Kern (Core), Domäne (Domain), System und Benutzeroberfläche (UI) – erfolgt primär über klar definierte öffentliche Programmierschnittstellen (APIs). Diese APIs werden in der Regel als Rust-Traits spezifiziert und von Service-Strukturen innerhalb der jeweiligen Schicht implementiert. Dies fördert ein vertragsbasiertes Interaktionsmodell.
2. **Ereignisgesteuertes System:** Ein robustes, ereignisgesteuertes System dient der Benachrichtigung über Zustandsänderungen und ermöglicht entkoppelte Interaktionen. Komponenten können auf relevante Ereignisse reagieren, ohne direkte Abhängigkeiten zu den Ereigniserzeugern aufzubauen. Dies ist besonders für die UI-Schicht wichtig, um reaktive Updates zu ermöglichen. Für die Event-Kommunikation wird `tokio::sync::broadcast` als bevorzugter Mechanismus für die Verteilung von Events spezifiziert.
3. **Strikte Schichtenhierarchie:** Höhere Schichten dürfen typischerweise nur von unmittelbar tieferliegenden Schichten abhängen, um zirkuläre Abhängigkeiten und unkontrollierte Interaktionen zu verhindern.
4. **Fehlerbehandlung:** Eine konsistente Fehlerbehandlungsstrategie wird über alle Schichten hinweg angewendet. Die `thiserror`-Bibliothek wird für die Definition von spezifischen, benutzerdefinierten Fehlertypen genutzt. `core::errors::CoreError` dient als fundamentaler Basistyp. Fehlerketten (`source()`) müssen bei der Weitergabe oder beim Wrappen von Fehlern erhalten bleiben. Panics sind strikt zu vermeiden.

## 1. Schnittstelle: Kernschicht (Core Layer) zu höheren Schichten (Domäne, System, UI)

Die Kernschicht stellt fundamentale Bausteine und Dienste bereit, die von allen darüberliegenden Schichten genutzt werden.

### 1.1. Bereitgestellte Funktionalität durch die Kernschicht

Die Kernschicht exportiert die folgenden Module und deren öffentliche APIs:

#### 1.1.1. `core::types`

- **Schnittstelle**: Direkte Verwendung von Typen durch die höheren Schichten.
- **Definierte Typen (Ultra-Fein)**:
    - `Point<T>`: Repräsentiert einen Punkt im 2D-Raum.
        - Felder: `pub x: T`, `pub y: T`.
        - Methoden: `new(x: T, y: T) -> Self`, `distance_squared(...)`, `distance(...)` (für Float-Typen), `manhattan_distance(...)`.
        - Assoziierte Konstanten: `ZERO_I32`, `ZERO_F32`, etc..
        - Generische Parameter: `T` (Basis-Constraints: `Copy + Debug + PartialEq + Default + Send + Sync + 'static`).
    - `Size<T>`: Repräsentiert eine 2D-Dimension.
        - Felder: `pub width: T`, `pub height: T`.
        - Methoden: `new(width: T, height: T) -> Self`, `area()`, `is_empty()`, `is_valid()`.
        - Assoziierte Konstanten: `ZERO_I32`, `ZERO_F32`, etc..
    - `Rect<T>`: Repräsentiert ein 2D-Rechteck.
        - Felder: `pub origin: Point<T>`, `pub size: Size<T>`.
        - Methoden: `new(...)`, `from_coords(...)`, Getter (`x`, `y`, `width`, `height`, `top`, `left`, `bottom`, `right`), `center()`, `contains_point(...)`, `intersects(...)`, `intersection(...)`, `union(...)`, `translated(...)`, `scaled(...)`, `is_valid()`.
    - `RectInt`: Typalias für `Rect<i32>` oder spezifische Struktur mit `x: i32, y: i32, width: u32, height: u32`. Bietet Methoden wie `inflate`, `translate`. Verwendet `saturating_add/sub` für Robustheit.
    - `Color`: Repräsentiert eine RGBA-Farbe.
        - Felder: `pub r: f32`, `pub g: f32`, `pub b: f32`, `pub a: f32` (Wertebereich `[0.0, 1.0]`).
        - Methoden: `new(r,g,b,a)` (klemmt Werte), `from_rgba8(...)`, `to_rgba8()`, `with_alpha(...)`, `blend(...)`, `lighten(...)`, `darken(...)`, `interpolate(...)`, `from_hex(...) -> Result<Self, ColorParseError>`, `to_hex_string(...)`.
        - Implementiert `Default` (z.B. `Color::TRANSPARENT`) und `Serialize`/`Deserialize` als Hex-String.
    - `Orientation`: Enum (`Horizontal`, `Vertical`) mit `toggle()` Methode und `Default` (Horizontal).
    - `uuid::Uuid`: Re-exportiert für eindeutige Identifikatoren.
    - `chrono::DateTime<Utc>`: Re-exportiert für Zeitstempel.
- **Nutzung**: Diese Typen werden direkt in den öffentlichen APIs und internen Logiken der höheren Schichten verwendet.

#### 1.1.2. `core::errors`

- **Schnittstelle**: Höhere Schichten wrappen Fehler aus der Kernschicht (z.B. `CoreError` oder spezifischere Modul-Fehler wie `ColorParseError`) mittels `#[from]` oder `#[source]` in ihre eigenen Fehlertypen. Die Fehlerkette (`source()`) wird beibehalten.
- **Definierte Typen (Ultra-Fein)**:
    - `CoreError`: Basis-Fehlertyp der Kernschicht.
        - Varianten: `InitializationFailed { component: String, source: Option<Box<dyn std::error::Error>> }`, `Io { path: PathBuf, source: std::io::Error }`, `Serialization { description: String, source: Option<Box<dyn std::error::Error>> }`, `Deserialization { ... }`, `InvalidId { invalid_id: String }`, `NotFound { resource_description: String }`, `CoreConfigError { message: String, source: Option<Box<dyn std::error::Error>> }`, `InternalError(String)`.
    - `ColorParseError`: Fehler beim Parsen von Hex-Farbcodes.
        - Varianten: `InvalidHexFormat(String)`, `InvalidHexDigit(String, source: std::num::ParseIntError)`, `InvalidHexLength(String)`.
- **Nutzung**: Jedes Modul, auch in höheren Schichten, definiert sein eigenes Fehler-Enum mit `thiserror` und integriert Fehler aus tieferen Schichten oder Bibliotheken.

#### 1.1.3. `core::logging`

- **Schnittstelle**:
    - `pub fn initialize_logging(level_filter: tracing::LevelFilter, format: LogFormat) -> Result<(), LoggingError>`: Einmaliger Aufruf beim Anwendungsstart.
        - `LogFormat`: Enum (`PlainTextDevelopment`, `JsonProduction`).
        - `LoggingError`: Enum (`SetGlobalDefaultError(String)`, `InitializationError(String)`).
    - Verwendung der `tracing`-Makros (`trace!`, `info!`, `warn!`, `error!`, `#[tracing::instrument]`) durch alle höheren Schichten.
- **Nutzung**: Alle Schichten nutzen `tracing`-Makros. Sensible Daten dürfen nicht geloggt werden.

#### 1.1.4. `core::config`

- **Schnittstelle**:
    - `pub fn load_core_config(config_paths: &[PathBuf]) -> Result<CoreConfig, ConfigError>`: Lädt Konfiguration aus TOML-Dateien in der angegebenen Reihenfolge.
    - `pub fn initialize_global_core_config(config: CoreConfig) -> Result<(), ConfigError>`: Initialisiert die globale Konfiguration einmalig.
    - `pub fn get_global_core_config() -> &'static CoreConfig`: Globaler, schreibgeschützter Zugriff auf die geladene `CoreConfig` (via `once_cell`).
- **Definierte Typen (Ultra-Fein)**:
    - `CoreConfig`: Struktur, die alle Kernschicht-spezifischen Einstellungen hält (z.B. `log_level: LogLevelConfig`, `feature_flags: FeatureFlags`). Implementiert `Deserialize`, `Default`. Verwendet `#[serde(default = "path")]` und `#[serde(deny_unknown_fields)]`.
    - `ConfigError`: Fehler beim Laden/Parsen der Konfiguration.
        - Varianten: `FileReadError { path: PathBuf, source: std::io::Error }`, `DeserializationError { path: PathBuf, source: toml::de::Error }`, `NoConfigurationFileFound { checked_paths: Vec<PathBuf> }`, `AlreadyInitializedError`, `NotInitializedError`.
- **Nutzung**: Domänenschicht (z.B. `domain::settings_persistence_iface`) kann Kernschicht-Dienste zum Lesen/Schreiben von Konfigurationsdateien nutzen, aber die `CoreConfig` selbst wird nach Initialisierung als unveränderlich betrachtet.

#### 1.1.5. `core::utils`

- **Schnittstelle**: Direkte Nutzung der öffentlichen, zustandslosen Hilfsfunktionen durch alle höheren Schichten.
- **Beispielhafte Utilities**:
    - `path_utils::normalize_path_robustly(...)`
    - `string_utils::sanitize_filename(...)`, `truncate_string_with_ellipsis(...)`
    - Fehlerbehandlung: Jede fehleranfällige Utility-Funktion gibt `Result<T, YourUtilError>` zurück.

## 2. Schnittstelle: Domänenschicht (Domain Layer) zu Systemschicht und Benutzeroberflächenschicht

Die Domänenschicht kapselt die Geschäftslogik und den Kernzustand von NovaDE. Sie interagiert nicht direkt mit Hardware oder OS-Details.

### 2.1. Bereitgestellte Funktionalität durch die Domänenschicht

Die Domänenschicht stellt ihre Logik und Zustandsinformationen über öffentliche APIs ihrer Service-Komponenten (Rust-Traits) und durch domänenspezifische Events bereit.

#### 2.1.1. Traits für Service-APIs (asynchron via `async_trait`)

- **`domain::theming::ThemingEngine` Trait**
    - Methoden:
        - `async fn get_current_theme_state(&self) -> Result<AppliedThemeState, ThemingError>`
        - `async fn get_available_themes(&self) -> Result<Vec<ThemeDefinition>, ThemingError>`
        - `async fn get_current_configuration(&self) -> Result<ThemingConfiguration, ThemingError>`
        - `async fn update_configuration(&self, new_config: ThemingConfiguration) -> Result<(), ThemingError>`
        - `async fn reload_themes_and_tokens(&self) -> Result<(), ThemingError>`
        - `fn subscribe_to_theme_changes(&self) -> tokio::sync::broadcast::Receiver<ThemeChangedEvent>`
- **`domain::workspaces::WorkspaceManagerService` Trait**
    - Methoden:
        - `async fn create_workspace(&self, name: Option<String>, persistent_id: Option<String>, icon_name: Option<String>, accent_color_hex: Option<String>) -> Result<WorkspaceId, WorkspaceManagerError>`
        - `async fn delete_workspace(&self, id: WorkspaceId, fallback_id_for_windows: Option<WorkspaceId>) -> Result<(), WorkspaceManagerError>`
        - `fn get_workspace(&self, id: WorkspaceId) -> Option<Workspace>` (gibt Klon)
        - `fn all_workspaces_ordered(&self) -> Vec<Workspace>` (gibt Klone)
        - `fn active_workspace_id(&self) -> Option<WorkspaceId>`
        - `async fn set_active_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceManagerError>`
        - `async fn assign_window_to_active_workspace(&self, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>`
        - `async fn assign_window_to_specific_workspace(&self, workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>`
        - `async fn remove_window_from_its_workspace(&self, window_id: &WindowIdentifier) -> Result<Option<WorkspaceId>, WorkspaceManagerError>`
        - `async fn move_window_to_specific_workspace(&self, target_workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>`
        - `async fn rename_workspace(&self, id: WorkspaceId, new_name: String) -> Result<(), WorkspaceManagerError>`
        - `async fn set_workspace_layout(&self, id: WorkspaceId, layout_type: WorkspaceLayoutType) -> Result<(), WorkspaceManagerError>`
        - `async fn set_workspace_icon(&self, id: WorkspaceId, icon_name: Option<String>) -> Result<(), WorkspaceManagerError>`
        - `async fn set_workspace_accent_color(&self, id: WorkspaceId, color_hex: Option<String>) -> Result<(), WorkspaceManagerError>`
        - `async fn save_configuration(&self) -> Result<(), WorkspaceManagerError>`
        - `fn subscribe_to_workspace_events(&self) -> tokio::sync::broadcast::Receiver<WorkspaceEvent>`
        - `async fn reorder_workspace(&self, workspace_id: WorkspaceId, new_index: usize) -> Result<(), WorkspaceManagerError>`
- **`domain::user_centric_services::ai_interaction::AIInteractionLogicService` Trait**
    - Methoden:
        - `async fn initiate_interaction(&mut self, relevant_categories: Vec<AIDataCategory>, initial_attachments: Option<Vec<AttachmentData>>) -> Result<Uuid, AIInteractionError>`
        - `async fn get_interaction_context(&self, context_id: Uuid) -> Result<AIInteractionContext, AIInteractionError>`
        - `async fn provide_consent(&mut self, context_id: Uuid, model_id: String, granted_categories: Vec<AIDataCategory>, consent_decision: bool) -> Result<(), AIInteractionError>`
        - `async fn get_consent_status_for_interaction(&self, context_id: Uuid, model_id: &str, required_categories: &[AIDataCategory]) -> Result<AIConsentStatus, AIInteractionError>`
        - `async fn add_attachment_to_context(&mut self, context_id: Uuid, attachment: AttachmentData) -> Result<(), AIInteractionError>`
        - `async fn list_available_models(&self) -> Result<Vec<AIModelProfile>, AIInteractionError>`
        - `async fn get_default_model(&self) -> Result<Option<AIModelProfile>, AIInteractionError>`
        - `async fn update_interaction_history(&mut self, context_id: Uuid, entry: InteractionHistoryEntry) -> Result<(), AIInteractionError>`
        - (Persistenz-bezogene Methoden wie `store_consent`, `get_all_user_consents`, `load_model_profiles` werden über separate Provider-Traits abgewickelt: `AIConsentProvider`, `AIModelProfileProvider`)
- **`domain::user_centric_services::notifications_core::NotificationService` Trait**
    - Methoden:
        - `async fn post_notification(&mut self, notification_data: NotificationInput) -> Result<Uuid, NotificationError>`
        - `async fn get_active_notification(&self, notification_id: Uuid) -> Result<Option<Notification>, NotificationError>`
        - `async fn mark_as_read(&mut self, notification_id: Uuid) -> Result<(), NotificationError>`
        - `async fn dismiss_notification(&mut self, notification_id: Uuid, reason: DismissReason) -> Result<(), NotificationError>`
        - `async fn get_active_notifications(&self, filter: Option<NotificationFilterCriteria>, sort_order: Option<NotificationSortOrder>) -> Result<Vec<Notification>, NotificationError>`
        - `async fn get_notification_history(&self, limit: Option<usize>, filter: Option<NotificationFilterCriteria>, sort_order: Option<NotificationSortOrder>) -> Result<Vec<Notification>, NotificationError>`
        - `async fn clear_history(&mut self) -> Result<(), NotificationError>`
        - `async fn clear_all_for_app(&mut self, app_id: &ApplicationId) -> Result<usize, NotificationError>`
        - `async fn set_do_not_disturb(&mut self, enabled: bool) -> Result<(), NotificationError>`
        - `async fn is_do_not_disturb_enabled(&self) -> Result<bool, NotificationError>`
        - `async fn invoke_action(&mut self, notification_id: Uuid, action_key: &str) -> Result<(), NotificationError>`
        - `async fn get_stats(&self) -> Result<NotificationStats, NotificationError>`
- **`domain::global_settings_and_state_management::GlobalSettingsService` Trait**
    - Methoden:
        - `async fn load_settings(&mut self) -> Result<(), GlobalSettingsError>`
        - `async fn save_settings(&self) -> Result<(), GlobalSettingsError>`
        - `fn get_current_settings(&self) -> GlobalDesktopSettings`
        - `async fn update_setting(&mut self, path: SettingPath, value: serde_json::Value) -> Result<(), GlobalSettingsError>`
        - `fn get_setting(&self, path: &SettingPath) -> Result<serde_json::Value, GlobalSettingsError>`
        - `async fn reset_to_defaults(&mut self) -> Result<(), GlobalSettingsError>`
        - `fn subscribe_to_setting_changes(&self) -> tokio::sync::broadcast::Receiver<SettingChangedEvent>`
- **`domain::notifications_rules::NotificationRulesEngine` Trait**
    - Methoden:
        - `async fn reload_rules(&self) -> Result<(), NotificationRulesError>`
        - `async fn process_notification(&self, notification: Notification) -> Result<RuleProcessingResult, NotificationRulesError>`
        - `async fn get_rules(&self) -> Result<NotificationRuleSet, NotificationRulesError>`
        - `async fn update_rules(&self, new_rules: NotificationRuleSet) -> Result<(), NotificationRulesError>`
- **`domain::window_management_policy::WindowManagementPolicyService` Trait**
    - Methoden:
        - `async fn calculate_workspace_layout(&self, workspace_id: WorkspaceId, windows_to_layout: &[WindowLayoutInfo], available_area: RectInt, workspace_current_tiling_mode: TilingMode, focused_window_id: Option<&WindowIdentifier>, window_specific_overrides: &HashMap<WindowIdentifier, WindowPolicyOverrides>) -> Result<WorkspaceWindowLayout, WindowPolicyError>`
        - `async fn get_initial_window_geometry(&self, window_info: &WindowLayoutInfo, is_transient_for: Option<&WindowIdentifier>, parent_geometry: Option<RectInt>, workspace_id: WorkspaceId, active_layout_on_workspace: &WorkspaceWindowLayout, available_area: RectInt, window_specific_overrides: &Option<WindowPolicyOverrides>) -> Result<RectInt, WindowPolicyError>`
        - `async fn calculate_snap_target(&self, moving_window_id: &WindowIdentifier, current_geometry: RectInt, other_windows_on_workspace: &[(&WindowIdentifier, &RectInt)], workspace_area: RectInt, snapping_policy: &WindowSnappingPolicy, gap_settings: &GapSettings) -> Option<RectInt>`
        - `async fn get_effective_tiling_mode_for_workspace(&self, workspace_id: WorkspaceId) -> Result<TilingMode, WindowPolicyError>`
        - `async fn get_effective_gap_settings_for_workspace(&self, workspace_id: WorkspaceId) -> Result<GapSettings, WindowPolicyError>`
        - (Weitere Methoden zum Abruf von Teil-Policies)

#### 2.1.2. Datenstrukturen (Öffentlich exportierte Typen aus der Domänenschicht)

- **`domain::theming::types`**: `AppliedThemeState`, `ThemeDefinition`, `ThemingConfiguration`, `TokenIdentifier`, `ThemeIdentifier`, `ColorSchemeType`, `AccentColor` (mit `novade_core::types::Color`).
- **`domain::workspaces::core::types`**: `WorkspaceId`, `WindowIdentifier`, `WorkspaceLayoutType`.
- **`domain::workspaces::core`**: `Workspace` (mit `icon_name`, `accent_color_hex`).
- **`domain::user_centric_services::ai_interaction::types`**: `AIInteractionContext` (mit `InteractionHistoryEntry`), `AIConsent` (mit `AIConsentScope`), `AIModelProfile` (mit `AIModelCapability`), `AttachmentData`, `AIConsentStatus`, `AIDataCategory`.
- **`domain::user_centric_services::notifications_core::types`**: `Notification` (mit `category`, `hints`, `timeout_ms`), `NotificationAction`, `NotificationUrgency`, `NotificationActionType`, `NotificationInput`, `NotificationFilterCriteria`, `NotificationSortOrder`, `DismissReason`.
- **`domain::global_settings_and_state_management::types`**: `GlobalDesktopSettings` (und alle untergeordneten Einstellungs-Structs wie `AppearanceSettings`, `InputBehaviorSettings`), `SettingPath`-Enum-Hierarchie.
- **`domain::notifications_rules::types`**: `RuleConditionValue`, `RuleConditionOperator`, `RuleConditionField`, `SimpleRuleCondition`, `RuleCondition`, `RuleAction`, `NotificationRule`, `NotificationRuleSet`, `RuleProcessingResult`.
- **`domain::window_management_policy::types`**: `TilingMode`, `GapSettings`, `WindowSnappingPolicy`, `WindowGroupingPolicy`, `NewWindowPlacementStrategy`, `FocusStealingPreventionLevel`, `FocusPolicy`, `WindowPolicyOverrides`, `WorkspaceWindowLayout`, `WindowLayoutInfo`.
- **`domain::shared_types`**: `ApplicationId`, `UserSessionState`, `ResourceIdentifier`.

#### 2.1.3. Events (Domänenspezifisch, via `tokio::sync::broadcast`)

- **`domain::theming`**: `ThemeChangedEvent { new_state: AppliedThemeState }`.
- **`domain::workspaces::manager::events`**: `WorkspaceEvent` Enum (z.B. `WorkspaceCreated { descriptor }`, `ActiveWorkspaceChanged { old_id, new_id, new_descriptor }`, `WindowAssignedToWorkspace { ... }`, `WorkspaceIconChanged`, `WorkspaceAccentChanged`, `WorkspaceOrderChanged`).
- **`domain::user_centric_services::events`**: Wrapper-Enums `AIInteractionEventEnum` und `NotificationEventEnum`.
    - `AIInteractionEventEnum`: `AIInteractionInitiatedEvent { context }`, `AIConsentUpdatedEvent { ... }`, `AIContextUpdatedEvent { ... }`, `AIModelProfilesReloadedEvent { ... }`.
    - `NotificationEventEnum`: `NotificationPostedEvent { notification, suppressed_by_dnd }`, `NotificationDismissedEvent { notification_id, reason }`, `NotificationReadEvent { ... }`, `DoNotDisturbModeChangedEvent { ... }`, `NotificationActionInvokedEvent { ... }`, `NotificationHistoryClearedEvent`, `NotificationPopupExpiredEvent { ... }`.
- **`domain::global_settings_and_state_management`**: `SettingChangedEvent { path: SettingPath, new_value: serde_json::Value }`, `SettingsLoadedEvent { settings: GlobalDesktopSettings }`, `SettingsSavedEvent`.
- **`domain::common_events`**: `UserActivityDetectedEvent { timestamp, activity_type, ... }`, `SystemShutdownInitiatedEvent { reason, is_reboot, ... }`.

#### 2.1.4. Fehlertypen (Modulspezifisch via `thiserror`)

- `domain::theming::errors::ThemingError`.
- `domain::workspaces::core::errors::WorkspaceCoreError`, `assignment::errors::WindowAssignmentError`, `manager::errors::WorkspaceManagerError`, `config::errors::WorkspaceConfigError`.
- `domain::user_centric_services::ai_interaction::errors::AIInteractionError`, `notifications_core::errors::NotificationError`.
- `domain::global_settings_and_state_management::errors::GlobalSettingsError`.
- `domain::notifications_rules::errors::NotificationRulesError`.
- `domain::window_management_policy::errors::WindowPolicyError`.

### 2.2. Nutzung durch die Systemschicht

- **Anwendung von Domänenregeln**: Die Systemschicht setzt Domänenrichtlinien technisch um.
    - `system::window_mechanics` nutzt `WindowManagementPolicyService::calculate_workspace_layout()` und andere Methoden, um Fenstergeometrien und -verhalten zu bestimmen und anzuwenden.
    - Der `system::compositor` und seine Submodule (z.B. für XDG-Shell, Layer-Shell) interagieren mit `WindowManagementPolicyService` und `WorkspaceManagerService` für Platzierung, Stapelreihenfolge und Sichtbarkeit von Fenstern/Surfaces.
- **Abfrage von Zuständen und Konfigurationen**:
    - `system::mcp_client` interagiert mit `AIInteractionLogicService` für Einwilligungsprüfungen, Modellprofile und Kontextmanagement.
    - `system::dbus::notifications_server` (D-Bus Implementierung von `org.freedesktop.Notifications`) leitet Anfragen an den `NotificationService` weiter und reagiert auf dessen Events, um D-Bus Signale zu senden.
    - Systemkomponenten wie `system::power_management` oder `system::audio_management` können relevante Einstellungen vom `GlobalSettingsService` beziehen.
- **Event-Konsum und -Weiterleitung**: Die Systemschicht kann auf Domänen-Events reagieren (z.B. `ActiveWorkspaceChanged`, `SettingChangedEvent` für themenrelevante Einstellungen) und diese ggf. in systemnahe Aktionen übersetzen oder an die UI-Schicht weiterleiten, wenn sie systemweite Auswirkungen haben (z.B. über den `SystemEventBridge`).
- **Fehlerbehandlung**: Fehler aus der Domänenschicht werden von der Systemschicht gefangen, ggf. in systemspezifische Fehler gewrappt und an die UI-Schicht oder interne Logging-Mechanismen weitergeleitet.

### 2.3. Nutzung durch die Benutzeroberflächenschicht (UI Layer)

- **Zustandsdarstellung**: Die UI-Schicht visualisiert Zustände und Daten aus der Domänenschicht.
    - `ui::theming_gtk` verwendet `ThemingEngine::get_current_theme_state()` und abonniert `ThemeChangedEvent`, um CSS-Stile anzuwenden.
    - `ui::shell` (z.B. Workspace-Switcher, Panel-Elemente) und `ui::control_center` nutzen `WorkspaceManagerService` und `GlobalSettingsService` intensiv zur Anzeige und Modifikation.
    - UI-Elemente für KI-Interaktionen (Befehlspalette, Assistenz-Widgets) nutzen `AIInteractionLogicService`.
    - Das `ui::notifications_frontend` (Popups, Notification-Center) nutzt `NotificationService` und reagiert auf dessen Events.
- **Auslösen von Geschäftslogik**: Benutzerinteraktionen in der UI werden in Aufrufe an Domänen-Service-Methoden übersetzt.
- **Event-Konsum**: Die UI-Schicht ist ein primärer Konsument von Domänen-Events, um sich dynamisch zu aktualisieren.
- **Fehlerbehandlung**: Fehler von Domänendiensten werden von der UI gefangen und in benutzerfreundliche Nachrichten oder Aktionen übersetzt.

## 3. Schnittstelle: Systemschicht (System Layer) zu Benutzeroberflächenschicht (UI Layer)

Die Systemschicht stellt der UI-Schicht systemnahe Dienste und Ereignisse zur Verfügung und setzt deren UI-initiierte Befehle technisch um.

### 3.1. Bereitgestellte Funktionalität durch die Systemschicht

#### 3.1.1. Traits für Service-APIs (asynchron via `async_trait`)

- **`system::compositor::CompositorInterface` (Beispielhafter Trait-Name)**
    - Methoden (Beispiele):
        - `fn request_focus(&self, window_id: novade_core::types::WindowId) -> Result<(), CompositorError>`
        - `fn configure_window(&self, window_id: novade_core::types::WindowId, config: WindowConfigurationRequest) -> Result<(), CompositorError>`
        - `fn get_window_info(&self, window_id: novade_core::types::WindowId) -> Result<Option<WindowInfo>, CompositorError>`
        - `fn list_managed_windows(&self) -> Result<Vec<WindowInfo>, CompositorError>`
        - `fn list_outputs(&self) -> Result<Vec<OutputInfo>, CompositorError>`
        - `fn subscribe_to_compositor_events(&self) -> tokio::sync::broadcast::Receiver<CompositorEvent>`
- **`system::input::InputInterface` (Beispielhafter Trait-Name)**
    - Methoden (Beispiele):
        - `fn set_cursor_theme(&self, theme_name: String, size: u32) -> Result<(), InputError>`
        - `fn get_current_keyboard_layout(&self) -> Result<String, InputError>`
        - `fn subscribe_to_input_events(&self) -> tokio::sync::broadcast::Receiver<InputEventEnum>`
- **`system::dbus` (Wrapper für spezifische D-Bus Clients wie UPower, Logind, NetworkManager)**: Exponiert vereinfachte Methoden und Events.
    - Beispiel `UPowerClientInterface`: `async fn get_display_device_details() -> Result<PowerDeviceDetails, DBusInterfaceError>`, `fn subscribe() -> Receiver<UPowerEvent>`.
- **`system::audio_management::AudioServiceInterface` (Beispielhafter Trait-Name)**
    - Methoden: `async fn list_audio_devices(...)`, `async fn set_device_volume(...)`, `fn subscribe() -> Receiver<AudioEvent>`.
- **`system::mcp_client::SystemMcpService` Trait**
    - Methoden: `async fn configure_servers(...)`, `async fn initialize_server(...)`, `async fn list_resources(...)`, `async fn call_tool(...)`, `fn subscribe_to_mcp_events() -> Receiver<McpClientSystemEvent>`.
- **`system::portals::PortalsInterface` (Beispielhafter Trait-Name für XDG Portal Interaktionen)**
    - Methoden: `async fn open_file_dialog(...) -> Result<Vec<PathBuf>, PortalError>`, `async fn save_file_dialog(...)`, `async fn take_screenshot(...)`.

#### 3.1.2. Datenstrukturen (Öffentlich exportierte Typen aus der Systemschicht für die UI)

- **`system::compositor`**: `WindowInfo` (mit `id`, `title`, `app_id`, `geometry`, Fokus-Status, Workspace-Zuordnung etc.), `OutputInfo` (mit ID, Name, Geometrie, Auflösungen, Skalierung), `CompositorWindowState` (Enum).
- **`system::input`**: `KeyboardEvent`, `PointerEvent` (Button, Motion, Axis), `TouchEvent`, `GestureEvent`, `InputEventEnum` (Wrapper).
- **`system::dbus` Clients**: Spezifische Structs für Gerätedetails, Verbindungsstatus etc. (z.B. `UPowerDeviceDetails` [aus `upower_client::types` aber ggf. hier vereinfacht für UI], `NetworkDeviceDetails`, `AccessPointInfo` [aus `network_manager_client::types`]).
- **`system::audio_management::types`**: `AudioDevice` (vereinfacht für UI), `AudioStream` (vereinfacht für UI).
- **`system::mcp_client::types`**: `McpClientSystemEvent` (für Benachrichtigungen an UI), `AICompletionResponse` (oder vereinfachte Version).
- **`system::portals`**: `OpenFileOptions`, `SaveFileOptions`, `FileFilter` für Dialoge. `WindowIdentifier` für Portal-Fenster-Handles.

#### 3.1.3. Events (System-Level Events, via `system::event_bridge` oder spezifische `broadcast::Sender`)

- **`system::compositor`**: `CompositorEvent` Enum (z.B. `WindowCreated { info }`, `WindowFocused { window_id }`, `OutputConfigurationChanged { info }`).
- **`system::input`**: `InputEventEnum` (Wrapper für spezifische Eingabe-Events).
- **`system::dbus` Clients**: Events wie `UPowerSystemEvent::DeviceChanged`, `LogindSystemEvent::SessionLocked`, `NetworkManagerSystemEvent::ConnectivityChanged`.
- **`system::audio_management`**: `AudioSystemEvent::DeviceVolumeChanged`, `AudioSystemEvent::DefaultDeviceChanged`.
- **`system::mcp_client`**: `McpClientSystemEvent::McpNotificationReceived`, `McpClientSystemEvent::McpToolCallSuccessful`.
- **`system::event_bridge::SystemLayerEvent`**: Ein übergreifendes Enum, das spezifische Events aus den Systemmodulen wrappen kann, um ein zentrales Abonnement zu ermöglichen.

#### 3.1.4. Fehlertypen (Modulspezifisch via `thiserror`)

- `system::compositor::errors::CompositorError` (z.B. `WindowNotFound`, `WaylandError`).
- `system::input::errors::InputError` (z.B. `SeatError`, `LibinputError`).
- `system::dbus_interfaces::common::errors::DBusInterfaceError` (z.B. `ConnectionFailed`, `MethodCallFailed`).
- `system::audio_management::errors::AudioManagementError` (z.B. `ConnectionFailed`, `ObjectNotFound`).
- `system::mcp_client::errors::McpSystemClientError` (z.B. `ServerConfigNotFound`, `RequestTimeout`).
- `system::portals` (z.B. `PortalError` mit `UserCancelled`, `PortalNotAvailable`).
- `system::window_mechanics::errors::WindowMechanicsError`.
- `system::power_management::errors::PowerManagementError`.

### 3.2. Nutzung durch die Benutzeroberflächenschicht (UI Layer)

- **Darstellung von Systeminformationen**: Die UI visualisiert die von der Systemschicht bereitgestellten Zustände (Fensterlisten, Fokus, Netzwerk-/Batterie-/Audio-Indikatoren etc.).
- **Reaktion auf Eingabeereignisse**: UI-Elemente (Buttons, Textfelder) reagieren auf verarbeitete Eingabeereignisse von `system::input`.
- **Initiierung von Systemaktionen**: Benutzerinteraktionen in der UI führen zu Befehlsaufrufen an die Systemschicht-APIs (Fenster schließen, Lautstärke ändern, Netzwerk verbinden, Datei öffnen via Portal).
- **Dialogmanagement**: UI-Komponenten (z.B. `ui::shell` oder spezifische Einstellungsdialoge) rufen Methoden von `system::portals::PortalsInterface` auf, um native Dialoge anzuzeigen. Die UI stellt dabei einen `WindowIdentifier` für das Elternfenster bereit.
- **Fehlerbehandlung**: Die UI fängt Fehler von der Systemschicht ab und präsentiert dem Benutzer verständliche Meldungen oder Wiederherstellungsoptionen.
- **Event-Konsum**: Die UI abonniert System-Events (z.B. `CompositorEvent::WindowCreated`, `InputEventEnum`, `UPowerSystemEvent`, etc.), um ihre Ansichten reaktiv zu aktualisieren.

## 4. Allgemeine Kommunikationsmuster (Wiederholung und Verfeinerung)

- **Synchrone vs. Asynchrone Aufrufe**:
    - Methoden, die schnell antworten und keine blockierenden Operationen ausführen (z.B. Abfrage eines bereits im Speicher befindlichen Zustands), können synchron sein.
    - Methoden, die I/O, IPC oder potenziell langlaufende Operationen beinhalten, **müssen** `async` sein (z.B. `save_settings()`, `call_tool()`, die meisten D-Bus-Aufrufe).
- **Event-Broadcasting**:
    - `tokio::sync::broadcast` wird als primärer Mechanismus für Events innerhalb einer Schicht und zwischen Schichten verwendet.
    - Services, die Events publizieren, halten einen `broadcast::Sender<EventType>`.
    - Services oder UI-Komponenten, die Events konsumieren, erhalten einen `broadcast::Receiver<EventType>` über eine `subscribe_to_..._events()` Methode.
- **Fehlerpropagation**:
    - Die konsequente Nutzung von `Result<T, E>` und dem `?`-Operator ist verbindlich.
    - Fehler werden mit `thiserror` definiert und sollten die `source()`-Kette erhalten, um die Ursachenanalyse zu ermöglichen.
    - Schichten wrappen Fehler aus tieferen Schichten in ihre eigenen, kontextspezifischeren Fehlertypen unter Beibehaltung der `source`.
- **Thread-Sicherheit für geteilten Zustand**:
    - Zustand, der von mehreren `async` Tasks oder über Thread-Grenzen hinweg geteilt wird, muss mit `Arc<tokio::sync::Mutex<...>>` oder `Arc<tokio::sync::RwLock<...>>` geschützt werden.
    - Der UI-Thread (GTK-Hauptschleife) erfordert spezielle Behandlung: Daten von Worker-Threads müssen über `glib::MainContext::spawn_local` oder geeignete Kanäle sicher an den UI-Thread übergeben werden, um UI-Aktualisierungen durchzuführen.

Diese detaillierte Schnittstellenspezifikation dient als Grundlage für die Implementierung der einzelnen Module und die Sicherstellung einer kohärenten und robusten Kommunikation zwischen den Architekturschichten von NovaDE.

**Grundprinzip der Kommunikation:**

Die Kommunikation zwischen den Schichten erfolgt primär über wohldefinierte öffentliche APIs (oft Rust-Traits, die von Service-Strukturen implementiert werden) und durch ein Event-System. Direkte Abhängigkeiten existieren typischerweise nur von einer höheren zu einer unmittelbar tieferen Schicht.

**Schnittstellen im Detail:**

**1. Kernschicht (Core Layer) zu allen höheren Schichten (Domäne, System, UI)**

- **Bereitgestellte Funktionalität durch die Kernschicht:**
    - **`core::types`**:
        - **Datentypen**: Stellt fundamentale Datentypen wie `Point<T>`, `Size<T>`, `Rect<T>`, `RectInt`, `Color`, `Orientation`, `Uuid` und `DateTime<Utc>` bereit.
        - **Nutzung**: Diese Typen werden direkt in den höheren Schichten für Geometrieberechnungen, Farbangaben, Identifikatoren und Zeitstempel verwendet.
    - **`core::errors`**:
        - **Fehlertypen**: Definiert den Basis-Fehlertyp `CoreError` und die Strategie für Modul-spezifische Fehler mit `thiserror`.
        - **Nutzung**: Höhere Schichten wrappen Fehler aus der Kernschicht oft in ihre eigenen spezifischeren Fehlertypen mittels `#[from]` oder `#[source]` auf `CoreError` oder spezifischen Kernschicht-Modulfehlern. Die Fehlerkette (`source()`) bleibt dabei erhalten.
    - **`core::logging`**:
        - **Logging-API**: Stellt Initialisierungsroutinen (`initialize_logging`) und die Konvention zur Verwendung von `tracing`-Makros (`trace!`, `debug!`, `info!`, `warn!`, `error!`) bereit.
        - **Nutzung**: Alle höheren Schichten verwenden die `tracing`-Makros für strukturiertes Logging. `initialize_logging` wird typischerweise vom Hauptanwendungsbinary (UI-Schicht oder Anwendungs-Root) aufgerufen.
    - **`core::config`**:
        - **Konfigurations-API**: Stellt Funktionen zum Laden (`load_core_config`) und globalen Zugriff (`get_core_config`) auf Kernkonfigurationen (`CoreConfig` ) bereit. Definiert `ConfigError`.
        - **Nutzung**:
            - **Domänenschicht**: Module wie `domain::settings_persistence_iface` (z.B. `FilesystemConfigProvider` ) und `domain::workspaces::config` nutzen Kernschicht-Dienste zum Lesen/Schreiben von Konfigurationsdateien.
            - **Andere Schichten**: Können `get_core_config()` für den Zugriff auf Kern-spezifische Einstellungen verwenden. Die Kernkonfiguration wird nach der Initialisierung als unveränderlich betrachtet.
    - **`core::utils`**:
        - **Hilfsfunktionen**: Stellt allgemeine, zustandslose Hilfsfunktionen bereit.
        - **Nutzung**: Direkte Verwendung durch alle höheren Schichten nach Bedarf.

**2. Domänenschicht (Domain Layer) zu Systemschicht (System Layer) und Benutzeroberflächenschicht (User Interface Layer)**

- **Bereitgestellte Funktionalität durch die Domänenschicht:**
    - **Logik und Zustand**: Die Domänenschicht stellt ihre Geschäftslogik und Zustandsinformationen über öffentliche APIs ihrer Service-Komponenten (oft als Traits definiert) und durch das Aussenden von domänenspezifischen Events bereit.
    - **Fehlertypen**: Jedes Domänenmodul definiert eigene `thiserror`-basierte Fehler-Enums (z.B. `ThemingError`, `WorkspaceCoreError`, `WindowAssignmentError`, `WorkspaceManagerError`, `WorkspaceConfigError`, `AIInteractionError`, `NotificationError`, `GlobalSettingsError` ).
    - **Events**: Domänenspezifische Events werden ausgelöst, um andere Schichten über Zustandsänderungen zu informieren (z.B. `ThemeChangedEvent`, `WorkspaceEvent`, `AIInteractionInitiatedEvent`, `AIConsentUpdatedEvent`, `NotificationPostedEvent`, `SettingChangedEvent` ).
- **Nutzung durch die Systemschicht:**
    - **Domänenregeln anwenden**: Die Systemschicht wendet die von der Domänenschicht definierten Richtlinien technisch an.
        - `system::compositor` interagiert mit `domain::window_management` für Platzierungsrichtlinien.
        - `system::window_mechanics` setzt die Policy aus `domain::window_management_policy` technisch um.
    - **Zustände abfragen**: Liest Zustände und Konfigurationen aus der Domänenschicht.
        - MCP-Client (`system::mcp`) interagiert mit `AIInteractionLogicService` für Einwilligungen und Kontext.
        - D-Bus Handler (`system::dbus`) für Benachrichtigungen nutzt `NotificationService`.
    - **Fehlerbehandlung**: Fehler aus der Domänenschicht werden von der Systemschicht behandelt oder weiterpropagiert (ggf. gewrappt).
    - **Event-Konsum**: Die Systemschicht kann auf Domänen-Events reagieren (z.B. Compositor passt Sichtbarkeit bei `ActiveWorkspaceChanged` an ).
- **Nutzung durch die Benutzeroberflächenschicht (UI Layer):**
    - **Zustandsdarstellung**: Visualisiert Zustände und Daten aus der Domänenschicht.
        - `ui::theming_gtk` konsumiert `AppliedThemeState` von `ThemingEngine`.
        - `ui::shell` und `ui::control_center` nutzen `GlobalSettingsService` und `WorkspaceManager`.
    - **Geschäftslogik auslösen**: Löst Aktionen und Zustandsänderungen in der Domänenschicht basierend auf Benutzerinteraktionen aus.
    - **Fehlerbehandlung**: Behandelt Fehler aus der Domänenschicht und stellt sie ggf. benutzerfreundlich dar.
    - **Event-Konsum**: Abonniert Domänen-Events, um sich dynamisch zu aktualisieren.

**Spezifische Domänen-Service-Schnittstellen (Beispiele):**

- **`ThemingEngine` API**:
    - Methoden: `new()`, `get_current_theme_state()`, `get_available_themes()`, `get_current_configuration()`, `update_configuration()`, `reload_themes_and_tokens()`, `subscribe_to_theme_changes()`.
    - Events: `ThemeChangedEvent`.
- **`WorkspaceManager` API**:
    - Methoden: `new()`, `create_workspace()`, `delete_workspace()`, `set_active_workspace()`, `assign_window_to_active_workspace()`, `save_configuration()`, etc.
    - Events: `WorkspaceCreated`, `WorkspaceDeleted`, `ActiveWorkspaceChanged`, etc.
- **`AIInteractionLogicService` Trait API**:
    - Methoden: `initiate_interaction()`, `get_interaction_context()`, `provide_consent()`, `get_consent_for_model()`, `store_consent()`, `load_model_profiles()`, etc.
    - Events: `AIInteractionInitiatedEvent`, `AIConsentUpdatedEvent`.
- **`NotificationService` Trait API**:
    - Methoden: `post_notification()`, `get_notification()`, `mark_as_read()`, `dismiss_notification()`, `set_do_not_disturb()`, etc.
    - Events: `NotificationPostedEvent`, `NotificationDismissedEvent`, etc.
- **`GlobalSettingsService` Trait API**:
    - Methoden: `load_settings()`, `save_settings()`, `get_current_settings()`, `update_setting()`, `get_setting()`, `reset_to_defaults()`, etc.
    - Events: `SettingChangedEvent`, `SettingsLoadedEvent`, `SettingsSavedEvent`.

**3. Systemschicht (System Layer) zu Benutzeroberflächenschicht (User Interface Layer)**

- **Bereitgestellte Funktionalität durch die Systemschicht:**
    - **Systemnahe Dienste und Ereignisse**: Stellt der UI-Schicht Informationen und Ereignisse bereit, die direkt vom Betriebssystem oder der Hardware stammen.
        - Fenstergeometrie, Fokusänderungen, neue Fenster
        - Eingabeereignisse (Tastatur, Maus, Touch, Gesten)
        - Statusänderungen von Systemdiensten (Netzwerk, Energie, Audio)
        - Monitor-/Output-Änderungen
    - **Technische Umsetzung von UI-Befehlen**: Empfängt Befehle von der UI-Schicht (z.B. Fenster verschieben, Space wechseln, Fokus anfordern) und setzt diese technisch um.
    - **Renderer-Schnittstelle**: Obwohl nicht direkt von der UI-Schicht konsumiert, stellt `system::compositor::renderer_interface` eine Abstraktion für das Rendering bereit, die vom Compositor genutzt wird, um die UI-Elemente darzustellen.
    - **Fehlertypen**: Jedes Systemschicht-Modul definiert eigene `thiserror`-basierte Fehler-Enums (z.B. `CompositorCoreError`, `ShmError`, `XdgShellError`, `InputError`, `RendererError` ).
- **Nutzung durch die Benutzeroberflächenschicht (UI Layer):**
    - **Empfang von Eingabeereignissen**: Die UI-Schicht empfängt verarbeitete Eingabeereignisse von der Systemschicht, um darauf zu reagieren (z.B. Klicks auf Buttons, Tastatureingaben in Textfeldern).
    - **Visualisierung von Systemzuständen**: Stellt Informationen dar, die von der Systemschicht bereitgestellt werden (z.B. aktive Fenster, Netzwerkstatus, Batterieladung, Audio-Lautstärke).
        - `ui::shell` und `ui::window_manager_frontend` interagieren mit `system::compositor` und `system::input` für Fenster- und Fokusinformationen.
        - UI-Indikatoren reagieren auf Events von `system::dbus` (UPower, NetworkManager) und `system::audio` (PipeWire).
    - **Auslösen von Systemaktionen**: Sendet Befehle an die Systemschicht basierend auf Benutzerinteraktionen (z.B. Klick auf "Fenster schließen", Auswahl eines anderen Netzwerks).
    - **Fehlerbehandlung**: Behandelt Fehler von der Systemschicht oder leitet sie an den Benutzer weiter.
    - **Event-Konsum**: Abonniert System-Events, um die UI dynamisch zu aktualisieren (z.B. Fokusänderung, neues Fenster, Output-Änderung).

**Spezifische Systemschicht-Schnittstellen (Beispiele für Interaktion mit UI):**

- **Compositor (`system::compositor`)**:
    - Stellt `WlSurface`-Informationen und Fensterstruktur bereit. Meldet Fensterzustände (Titel, AppID, Geometrie) an `ui::shell` und `ui::window_manager_frontend`.
    - Empfängt Befehle zur Fokusänderung von der UI.
- **Eingabeverarbeitung (`system::input`)**:
    - Sendet Fokusänderungs-Events und Cursor-Informationen an `ui::shell`.
    - Empfängt Befehle zur Fokusänderung von der UI.
- **D-Bus Clients (`system::dbus`)**:
    - `upower_client` sendet `UPowerEvent` (Batteriestatus) an UI.
    - `logind_client` sendet `LogindEvent` (Suspend, Sitzungssperre) an UI, kann `LockSession` von UI empfangen.
    - `networkmanager_client` sendet Netzwerkstatus-Events an UI.
    - `secrets_client` interagiert mit UI für Prompts.
- **Output-Management (`system::outputs`)**:
    - Meldet Output-Änderungen an `ui::shell` und `ui::control_center`.
    - Empfängt Konfigurationsbefehle (Auflösung, Skalierung) von `ui::control_center`.
- **Audio-Management (`system::audio`)**:
    - Sendet `AudioEvent` (Geräte-/Stream-Änderungen, Lautstärke) an UI.
    - Empfängt `AudioCommand` (Lautstärke ändern) von UI.
- **MCP-Client (`system::mcp`)**:
    - Stellt KI-Funktionen für UI-Elemente wie die Befehlspalette bereit.
    - Empfängt Anfragen von der UI.
- **XDG Desktop Portals (`system::portals`)**:
    - Interagiert mit `ui::shell`/`ui::components` zur Anzeige von Dialogen (FileChooser, Screenshot).

**Zusammenfassende Prinzipien der Schnittstellen:**

1. **Klare Verantwortlichkeiten**: Jede Schicht hat definierte Aufgaben, und die Schnittstellen spiegeln diese wider.
2. **Lose Kopplung**: Kommunikation erfolgt über APIs und Events, um direkte Abhängigkeiten von Implementierungsdetails zu minimieren.
3. **Hohe Kohäsion**: Module innerhalb einer Schicht arbeiten eng zusammen, aber die Schicht als Ganzes hat eine klar definierte externe Schnittstelle.
4. **Einheitliche Fehlerbehandlung**: Die `thiserror`-basierte Strategie wird über alle Schichten hinweg verwendet, wobei Fehler weitergeleitet oder spezifisch behandelt werden. Die `source()`-Kette ist wichtig.
5. **Event-gesteuerte Updates**: Viele Zustandsänderungen werden über Events kommuniziert, was eine reaktive und entkoppelte Architektur fördert.

Diese Spezifikation der internen Schnittstellen ist entscheidend, um sicherzustellen, dass die verschiedenen Schichten der NovaDE kohärent zusammenarbeiten und die Gesamtziele des Projekts – Performance, Intuition, Modernität, Modularität, Anpassbarkeit, sichere KI-Integration und Stabilität – erreicht werden können.**Technische Spezifikation der internen Schnittstellen zwischen den Schichten der Linux Desktop-Umgebung "NovaDE"**

Diese Spezifikation beschreibt die Art und Weise, wie die vier Hauptarchitekturschichten von NovaDE – Kernschicht, Domänenschicht, Systemschicht und Benutzeroberflächenschicht – miteinander interagieren. Die Kommunikation erfolgt primär über wohldefinierte öffentliche APIs, die oft als Rust-Traits implementiert sind, sowie über ein Event-System zur Signalisierung von Zustandsänderungen.

**1. Schnittstelle: Kernschicht (Core Layer) zu höheren Schichten (Domäne, System, UI)**

Die Kernschicht stellt fundamentale Bausteine und Dienste bereit, die von allen darüberliegenden Schichten genutzt werden.

- **Bereitgestellte Funktionalität:**
    - **`core::types`**: Definiert grundlegende, universell einsetzbare Datentypen.
        - **Schnittstelle**: Direkte Verwendung von Typen wie `Point<T>`, `Size<T>`, `Rect<T>`, `RectInt`, `Color`, `Orientation` sowie `uuid::Uuid` und `chrono::DateTime<Utc>` durch die höheren Schichten.
        - **Beispielhafte Nutzung**: Die Domänenschicht verwendet `Color` für Theming-Definitionen, die Systemschicht `RectInt` für Fenstergeometrien, und die UI-Schicht `Point<T>` für die Positionierung von Elementen.
    - **`core::errors`**: Stellt eine Basis-Fehlerbehandlungsstrategie und den `CoreError`-Typ bereit.
        - **Schnittstelle**: Höhere Schichten können `CoreError` oder spezifischere Fehler aus Kernmodulen mittels `#[from]` oder `#[source]` in ihre eigenen Fehlertypen wrappen. Die Fehlerursachenkette (`source()`) wird dabei beibehalten.
        - **Beispielhafte Nutzung**: Ein `ConfigError` in `domain::workspaces::config` kann einen `CoreError::Io` wrappen, der beim Lesen einer Datei in `core::config` aufgetreten ist.
    - **`core::logging`**: Definiert die Logging-Infrastruktur basierend auf `tracing`.
        - **Schnittstelle**: Alle höheren Schichten verwenden die `tracing`-Makros (`trace!`, `info!`, etc.) für ihre Logging-Ausgaben. Die Funktion `core::logging::initialize_logging()` wird typischerweise einmalig von der Anwendung (z.B. UI-Schicht) beim Start aufgerufen.
    - **`core::config`**: Stellt Primitive zum Laden, Parsen und Zugreifen auf Kernkonfigurationen bereit.
        - **Schnittstelle**: Funktionen wie `load_core_config(custom_path: Option<PathBuf>) -> Result<CoreConfig, ConfigError>` und `get_core_config() -> &'static CoreConfig` für den globalen Zugriff. Die `CoreConfig`-Struktur selbst ist Teil der Schnittstelle.
        - **Beispielhafte Nutzung**: `domain::settings_persistence_iface` (oder eine konkrete Implementierung wie `FilesystemConfigProvider` ) nutzt diese API, um Basiskonfigurationen zu lesen, die dann von der Domänenschicht weiterverarbeitet werden.
    - **`core::utils`**: Bietet allgemeine Hilfsfunktionen.
        - **Schnittstelle**: Direkte Nutzung der öffentlichen Funktionen durch alle höheren Schichten.

**2. Schnittstelle: Domänenschicht (Domain Layer) zu Systemschicht und Benutzeroberflächenschicht**

Die Domänenschicht kapselt die Geschäftslogik und den Kernzustand der Desktop-Umgebung.

- **Bereitgestellte Funktionalität:**
    
    - **Service-APIs (Traits)**: Öffentliche Schnittstellen werden primär durch Rust-Traits definiert, die von Service-Strukturen innerhalb der Domänenmodule implementiert werden.
        - `domain::theming::ThemingEngine`: Methoden wie `get_current_theme_state()`, `update_configuration()`.
        - `domain::workspaces::WorkspaceManager`: Methoden wie `create_workspace()`, `set_active_workspace()`.
        - `domain::user_centric_services::AIInteractionLogicService`: Methoden wie `initiate_interaction()`, `provide_consent()`.
        - `domain::user_centric_services::NotificationService`: Methoden wie `post_notification()`, `get_active_notifications()`.
        - `domain::global_settings_and_state_management::GlobalSettingsService`: Methoden wie `load_settings()`, `update_setting()`.
    - **Datenstrukturen**: Öffentliche Datenstrukturen, die Zustände oder Konfigurationen repräsentieren (z.B. `AppliedThemeState`, `Workspace`, `Notification`, `GlobalDesktopSettings` ).
    - **Events**: Domänenspezifische Events, die Zustandsänderungen signalisieren.
        - Beispiele: `ThemeChangedEvent`, `WorkspaceEvent` (z.B. `ActiveWorkspaceChanged` ), `NotificationPostedEvent`, `SettingChangedEvent`.
    - **Fehlertypen**: Modulspezifische Fehler-Enums (z.B. `ThemingError`, `WorkspaceManagerError`, `AIInteractionError`, `GlobalSettingsError` ).
- **Nutzung durch die Systemschicht:**
    
    - Die Systemschicht konsumiert die Service-APIs der Domänenschicht, um Geschäftsregeln anzuwenden und Zustände abzufragen.
        - Der `system::compositor` nutzt `domain::window_management_policy` für Fensterplatzierungsrichtlinien.
        - Der `system::mcp` Client interagiert mit `AIInteractionLogicService` für Einwilligungsprüfungen und Kontextinformationen.
        - `system::dbus` (für Benachrichtigungen) interagiert mit `NotificationService`.
    - Die Systemschicht kann auf Domänen-Events reagieren, um ihr Verhalten anzupassen (z.B. Umschalten der sichtbaren Surfaces im Compositor bei `ActiveWorkspaceChanged` ).
    - Fehler aus der Domänenschicht werden in der Systemschicht behandelt oder weitergeleitet.
- **Nutzung durch die Benutzeroberflächenschicht:**
    
    - Die UI-Schicht nutzt die Service-APIs der Domänenschicht, um Daten für die Darstellung abzurufen und Benutzeraktionen in Domänenlogik umzusetzen.
        - `ui::control_center` verwendet `GlobalSettingsService` zum Anzeigen und Ändern von Einstellungen.
        - `ui::shell` interagiert mit `WorkspaceManager` für die Workspace-Darstellung und -Navigation.
        - `ui::theming_gtk` reagiert auf `ThemeChangedEvent` und wendet Stile an.
    - Die UI-Schicht abonniert Domänen-Events, um ihre Ansichten dynamisch zu aktualisieren.
    - Fehler aus der Domänenschicht werden von der UI-Schicht behandelt und dem Benutzer ggf. in verständlicher Form präsentiert.

**3. Schnittstelle: Systemschicht (System Layer) zu Benutzeroberflächenschicht (UI Layer)**

Die Systemschicht stellt der UI-Schicht systemnahe Dienste und Ereignisse zur Verfügung und setzt deren Befehle technisch um.

- **Bereitgestellte Funktionalität:**
    
    - **Systemereignisse und -zustände**:
        - **Fensterinformationen**: Geometrie, Titel, AppID, Fokusstatus von Fenstern (aus `system::compositor` und `system::xdg_shell`).
        - **Eingabeereignisse**: Verarbeitete Tastatur-, Maus-, Touch- und Gestenereignisse (aus `system::input`).
        - **Output-Informationen**: Verfügbare Monitore, Auflösungen, Skalierungsfaktoren (aus `system::outputs`).
        - **Status von Systemdiensten**: Netzwerkverbindungen (`system::dbus::networkmanager_client` ), Energiestatus (`system::dbus::upower_client` ), Audiostatus (`system::audio` ).
        - **Sitzungsereignisse**: Sperren, Abmelden (von `system::dbus::logind_client` ).
    - **Ausführung von UI-Befehlen**:
        - Fenstermanipulationen (Verschieben, Größe ändern, Fokus setzen), initiiert durch die UI, werden vom `system::compositor` und `system::window_mechanics` umgesetzt.
        - Workspace-Wechsel.
        - Anpassung von Systemeinstellungen (z.B. Bildschirmhelligkeit, Lautstärke), die von `system::outputs` bzw. `system::audio` ausgeführt werden.
    - **Fehlertypen**: Modulspezifische Fehler-Enums (z.B. `CompositorCoreError`, `InputError` ).
- **Nutzung durch die Benutzeroberflächenschicht:**
    
    - **Darstellung von Systeminformationen**: Die UI visualisiert die von der Systemschicht bereitgestellten Zustände.
        - Fensterlisten, Titelleisten, Fokus-Hervorhebungen basieren auf Daten von `system::compositor`.
        - Netzwerk-, Batterie-, Audio-Indikatoren in `ui::shell` zeigen Daten von `system::dbus` und `system::audio`.
    - **Reaktion auf Eingabeereignisse**: UI-Elemente reagieren auf verarbeitete Eingabeereignisse, um Aktionen auszulösen.
    - **Initiierung von Systemaktionen**: Benutzerinteraktionen in der UI führen zu Befehlsaufrufen an die Systemschicht.
        - Klick auf "Lauter"-Button in `ui::shell` ruft eine Funktion in `system::audio` auf.
        - Auswahl eines anderen Monitorsetups in `ui::control_center` sendet Befehl an `system::outputs`.
    - **Dialoge über XDG Portals**: `ui::shell` oder `ui::components` interagieren mit `system::portals` für Datei-Auswahl- oder Screenshot-Dialoge.
    - Die UI-Schicht behandelt Fehler von der Systemschicht und informiert ggf. den Benutzer.
    - Die UI-Schicht reagiert auf Systemereignisse (z.B. `ActiveWorkspaceChanged` indirekt über Änderungen der sichtbaren Fenster, `DeviceAdded` für Eingabegeräte), um ihre Darstellung anzupassen.

**4. Allgemeine Kommunikationsmuster**

- **Synchrone Aufrufe**: Direkte Methodenaufrufe an Services oder Funktionen der tieferen Schicht für Anfragen, die eine sofortige Antwort erfordern oder direkte Zustandsmanipulationen durchführen.
- **Asynchrone Operationen**: Wo sinnvoll (z.B. I/O-gebundene Operationen in der Systemschicht oder langlaufende Prozesse in der Domänenschicht), werden `async/await` und entsprechende Runtimes (Tokio, GLib-Kontext) verwendet.
- **Event-Broadcasting**: Für entkoppelte Benachrichtigungen über Zustandsänderungen. Oft mittels `tokio::sync::broadcast` oder ähnlichen Mechanismen.
- **Fehlerpropagation**: Konsequente Nutzung von `Result<T, E>` und dem `?`-Operator. Fehler werden entweder behandelt oder an die aufrufende Schicht weitergegeben, wobei die `source()`-Kette erhalten bleibt.

Diese detaillierte Spezifikation der internen Schnittstellen dient als Grundlage für eine modulare, wartbare und robuste Entwicklung der NovaDE. Die klare Trennung der Verantwortlichkeiten und die wohldefinierten Kommunikationswege sind entscheidend für den Erfolg des Projekts.
