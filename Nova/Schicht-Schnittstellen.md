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