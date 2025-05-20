# core
```
src/core/                   # Kernschicht (Core Layer)
├── mod.rs              # Modul-Definition für die Kernschicht
├── types/              # Modul für grundlegende Datentypen
│   ├── mod.rs
│   ├── point.rs        # Point<T, U> Struktur
│   ├── size.rs         # Size<T, U> Struktur
│   ├── rectangle.rs    # Rectangle<T, U> Struktur
│   ├── color.rs        # Color Struktur
│   ├── uuid.rs         # UUID Typalias und Utilities
│   ├── timestamp.rs    # Timestamp Typalias und Utilities
│   └── units.rs        # Typen für Maßeinheiten (z.B. Pixels, Dp)
│
├── event_system/       # Modul für das globale Event-System
│   ├── mod.rs
│   ├── event_bus.rs    # EventBus Implementierung (z.B. basierend auf tokio::sync::broadcast)
│   ├── events.rs       # Definition aller System-Events (Enum mit Event-Daten)
│   └── listener.rs     # Listener Registrierung und Handling (Trait und Implementierung)
│
├── config/             # Modul für Konfigurationsverwaltung
│   ├── mod.rs
│   ├── config_loader.rs # Logik zum Laden, Parsen und Speichern von Konfigurationsdateien (z.B. TOML, JSON)
│   ├── config_types.rs  # Definition von Konfigurationsstrukturen (serde-kompatibel)
│   └── validation.rs    # Konfigurationsvalidierungslogik
│
├── logging/            # Modul für das Logging
│   ├── mod.rs
│   └── setup.rs        # Logging Initialisierung und Utilities (z.B. basierend auf `env_logger` oder `tracing`)
│
├── error_handling/     # Modul für zentrale Fehlerbehandlung
│   ├── mod.rs
│   ├── error_types.rs  # Definition von benutzerdefinierten, spezifischen Fehlertypen (Enum)
│   └── result_types.rs # Typalias für Result mit Standard-Fehlertyp (z.B. `pub type Result<T> = std::result::Result<T, NovaError>;`)
│
└── utils/              # Modul für allgemeine Utilities
    ├── mod.rs
    ├── futures.rs      # Hilfsfunktionen für asynchrone Operationen
    ├── paths.rs        # Funktionen für Standard-Dateipfade (Konfiguration, Cache etc.)
    └── validators.rs   # Allgemeine Validierungsfunktionen
```

```
src/domain/                 # Domänenschicht (Domain Layer)
├── mod.rs              # Modul-Definition für die Domänenschicht
├── workspace/          # Modul für Workspace (Space) Management
│   ├── mod.rs
│   ├── workspace.rs    # Definition der Workspace-Struktur (ID, Name, Icon, Akzentfarbe, gepinnte Apps/Layouts)
│   ├── manager.rs      # Service zur Verwaltung von Workspaces (Erstellen, Löschen, Wechseln, Konfigurieren)
│   └── events.rs       # Events im Zusammenhang mit Workspace-Änderungen (z.B. `WorkspaceCreated`, `WorkspaceChanged`, `WorkspaceClosed`)
│
├── window_management/  # Modul für Fensterverwaltung (Domänenlogik)
│   ├── mod.rs
│   ├── window.rs       # Definition der Fensterstruktur (ID, Titel, App-ID, Größe, Position, Zustand - minimiert/maximiert/Vollbild/gekachelt)
│   ├── manager.rs      # Service zur Verwaltung des Fensterzustands und der Logik (Zuweisung zu Workspaces, Zustandsänderungen, Interaktion mit Tiling)
│   ├── tiling.rs       # Implementierung der Tiling-Logik und definierter Layouts (Structs/Enums für Layout-Typen, Funktionen zur Fensteranordnung)
│   └── events.rs       # Events im Zusammenhang mit Fensteränderungen (z.B. `WindowOpened`, `WindowClosed`, `WindowStateChanged`, `WindowMoved`, `WindowResized`)
│
├── application/        # Modul für Anwendungsmanagement (Domänenlogik)
│   ├── mod.rs
│   ├── application.rs  # Definition der Anwendungsstruktur (App-ID, Name, Executable Path, Icon Path)
│   ├── manager.rs      # Service zur Verwaltung installierter und laufender Anwendungen, Starten/Beenden von Anwendungen (Interaktion mit Systemschicht für Prozessmanagement)
│   └── events.rs       # Events im Zusammenhang mit Anwendungslebenszyklus (z.B. `ApplicationLaunched`, `ApplicationQuit`)
│
├── settings/           # Modul für Benutzer-Einstellungen (Domänenlogik)
│   ├── mod.rs
│   ├── settings.rs     # Struktur, die alle Benutzer-Einstellungen repräsentiert (serialisierbar)
│   └── manager.rs      # Service zum Laden, Speichern und Abrufen von Benutzer-Einstellungen (Interaktion mit Core/config)
│
├── shortcuts/          # Modul für Tastaturkürzelverwaltung (Domänenlogik)
│   ├── mod.rs
│   ├── shortcut.rs     # Definition eines Tastaturkürzels (Key Combination, zugehörige Aktion/Kommando)
│   └── manager.rs      # Service zur Verwaltung registrierter Tastaturkürzel und Ausführung der zugehörigen Aktionen (Interaktion mit Systemschicht für Eingabe-Events)
│
├── theming/            # Modul für Theme-Verwaltung (Domänenlogik)
│   ├── mod.rs
│   ├── theme.rs        # Definition einer Theme-Konfiguration (Referenzen auf Token-Sets, etc.)
│   ├── manager.rs      # Service zur Verwaltung aktiver Themes, Wechsel von Themes zur Laufzeit
│   └── tokens.rs       # Definition von Design-Token-Sets (Farben, Abstände, Schriftgrößen als strukturierte Daten)
│
├── search/             # Modul für globale Suche (Domänenlogik)
│   ├── mod.rs
│   ├── indexer.rs      # Logik zur Indexierung durchsuchbarer Inhalte (Dateien, Anwendungen, Einstellungen)
│   └── query_engine.rs # Logik zur Verarbeitung von Suchanfragen und Rückgabe von Suchergebnissen
│
└── ki_integration/     # Modul für KI-Integration (Domänenlogik)
    ├── mod.rs
    ├── ki_context.rs   # Definition von KI-Kontexten (z.B. aktuelles Fenster, markierter Text)
    ├── consent_manager.rs # Service zur Verwaltung und Abfrage der expliziten Benutzerzustimmung für KI-Aktionen (Speicherung der Zustimmungen)
    ├── mcp_client.rs   # Client-Logik zur Kommunikation mit MCP-konformen KI-Modellen (Sendet Anfragen, verarbeitet Antworten)
    └── ki_actions.rs   # Definition von Aktionen, die von der KI ausgeführt werden können (z.B. Dateianalyse, Textgenerierung)

```

```
src/system/                 # Systemschicht (System Layer)
├── mod.rs              # Modul-Definition für die Systemschicht
├── wayland_integration/ # Modul für Wayland-Integration (Compositor-Seite)
│   ├── mod.rs
│   ├── compositor.rs   # Implementierung des Smithay Compositors (Haupt-Event-Loop, State-Management)
│   ├── protocols.rs    # Implementierung spezifischer Wayland Protokolle (z.B. xdg-shell für Fensteroberflächen, zxdg_output_manager_v1 für Monitore)
│   ├── seat.rs         # Verwaltung des Wayland Seat (Tastaturfokus, Zeigerposition, Touch-Events)
│   ├── shm.rs          # Shared Memory Management für Buffer-Austausch
│   ├── output.rs       # Monitor (Output) Management und Konfiguration
│   └── data_device.rs  # Copy-Paste und Drag-and-Drop Funktionalität
│
├── dbus_integration/   # Modul für D-Bus Kommunikation (Client- und Service-Seite)
│   ├── mod.rs
│   ├── connection.rs   # Verwaltung der D-Bus Verbindungen (System Bus, Session Bus)
│   ├── services.rs     # Implementierung und Registrierung von D-Bus Services, die NovaDE anbietet (z.B. für Benachrichtigungen)
│   └── client.rs       # Clients für die Interaktion mit externen D-Bus Services (UPower, NetworkManager etc.)
│
├── policykit_integration/ # Modul für PolicyKit Interaktion
│   ├── mod.rs
│   └── client.rs       # PolicyKit Client zur Überprüfung und Anforderung von Privilegien (z.B. für Shutdown, Reboot)
│
├── libinput_integration/ # Modul für libinput Verarbeitung
│   ├── mod.rs
│   └── handler.rs      # Handler zur Verarbeitung von Roh-Eingabe-Events von libinput und Weiterleitung an den Wayland Seat
│
├── power_management/   # Modul für Energieverwaltung
│   ├── mod.rs
│   └── upower_client.rs # D-Bus Client zur Interaktion mit UPower (Batteriestatus, Energieprofile)
│
├── network_management/ # Modul für Netzwerkverwaltung
│   ├── mod.rs
│   └── networkmanager_client.rs # D-Bus Client zur Interaktion mit NetworkManager (WLAN, VPN, etc.)
│
├── audio_management/   # Modul für Audioverwaltung
│   ├── mod.rs
│   └── audio_client.rs # Client zur Steuerung des Audio-Systems (z.B. PipeWire oder Pulseaudio via D-Bus oder deren native APIs)
│
├── notification_management/ # Modul für Benachrichtigungssystem
│   ├── mod.rs
│   ├── daemon.rs       # Implementierung des Freedesktop Notification Specification D-Bus Service
│   └── client.rs       # Client zum Senden von Desktop-Benachrichtigungen von NovaDE-Komponenten
│
├── secret_service_integration/ # Modul für Secret Service Interaktion
│   ├── mod.rs
│   └── client.rs       # Client zur sicheren Speicherung und Abrufung von Geheimnissen (API-Schlüssel etc.)
│
└── xdg_portals/        # Modul für XDG Desktop Portals
    ├── mod.rs
    ├── implementation.rs # Implementierung von Portal-Schnittstellen, die NovaDE bereitstellt (z.B. für Screenshots, Dateiauswahl)
    └── client.rs       # Client zur Nutzung von Portal-Schnittstellen, die von anderen Anwendungen/Services bereitgestellt werden
```


```
src/ui/                     # Benutzeroberflächenschicht (UI Layer) - Primär basierend auf GTK4 und Rust Bindings (gtk4-rs)
├── mod.rs              # Modul-Definition für die UI-Schicht
├── shell/              # Modul für die Haupt-Applikations-Shell
│   ├── mod.rs
│   ├── app_initializer.rs # GTK::Application Initialisierung, Signal-Handling (activate, startup), Lebenszyklus des globalen ApplicationState Objekts (Non-Widget GObject)
│   ├── main_window.rs  # Hauptfenster-Widget (gtk::ApplicationWindow), enthält die primäre Layout-Struktur für die UI-Komponenten (z.B. Box mit Seitenleisten, Hauptbereich)
│   └── application_state.rs # Globaler Anwendungszustand (enthält Referenzen auf Domain-Services und UI-Manager)
│
├── system_bar/         # Modul für Kontroll- und Systemleiste (Top/Bottom)
│   ├── mod.rs
│   ├── system_bar_widget.rs # Das Haupt-Widget für die Systemleiste (gtk::Box oder ähnliches)
│   └── indicators/     # Untermodule für einzelne Indikatoren/Widgets in der Systemleiste (Uhrzeit, Datum, Netzwerk, Audio, Akku, Benachrichtigungs-Trigger)
│       ├── mod.rs
│       ├── clock_indicator.rs
│       ├── network_indicator.rs
│       ├── audio_indicator.rs
│       ├── battery_indicator.rs
│       └── notification_trigger.rs # Button oder Icon zum Öffnen des Benachrichtigungszentrums
│
├── tab_bar/            # Modul für Intelligente Tab-Leiste pro Space
│   ├── mod.rs
│   ├── tab_bar_widget.rs # Das Haupt-Widget für die Tab-Leiste (gtk::Box oder gtk::StackSwitcher/Stack)
│   ├── tab_button.rs   # Widget für einen einzelnen Tab (Button mit Icon, Titel, Close-Button)
│   └── tab_data.rs     # Struktur zur Repräsentation der Daten eines Tabs (referenziert Fenster/Anwendung)
│
├── quick_settings/     # Modul für Quick-Settings-Panel (Popover aus Systemleiste)
│   ├── mod.rs
│   ├── quick_settings_widget.rs # Das Popover- oder Panel-Widget
│   └── settings_toggles/ # Widgets für einzelne Schnelleinstellungen (WLAN, Bluetooth, Dark Mode, Lautstärke-Schieberegler)
│       ├── mod.rs
│       ├── wifi_toggle.rs
│       ├── bluetooth_toggle.rs
│       ├── dark_mode_toggle.rs
│       └── volume_slider.rs
│
├── control_center/     # Modul für Control Center (Alle Einstellungen)
│   ├── mod.rs
│   ├── control_center_window.rs # Das Hauptfenster für das Control Center (gtk::ApplicationWindow)
│   ├── control_center_sidebar.rs # Navigationsleiste im Control Center (gtk::ListView/StackSidebar)
│   └── modules/        # Untermodule für einzelne Einstellungsbereiche (gtk::StackPage oder separates Widget pro Bereich)
│       ├── mod.rs
│       ├── network_settings.rs   # Netzwerk-Einstellungen UI
│       ├── appearance_settings.rs # Erscheinungsbild-Einstellungen UI (Theming, Akzentfarbe, Hintergründe)
│       ├── display_settings.rs   # Anzeige-Einstellungen UI (Auflösung, Skalierung, Multi-Monitor)
│       ├── sound_settings.rs     # Sound-Einstellungen UI
│       ├── power_settings.rs     # Energie-Einstellungen UI
│       ├── shortcuts_settings.rs # Tastaturkürzel-Einstellungen UI
│       ├── users_settings.rs     # Benutzer-Einstellungen UI
│       ├── ki_settings.rs        # KI-Integrations-Einstellungen UI (MCP-Konfiguration, Zustimmungen)
│       # ... weitere Einstellungsmodule
│
├── dock/               # Modul für Schnellaktionsdock
│   ├── mod.rs
│   ├── dock_widget.rs  # Das Haupt-Widget für das Dock (gtk::Box, ggf. mit Positionierungslogik)
│   └── dock_item_widget.rs # Widget für ein einzelnes Dock-Element (Icon, ggf. Label, Badge für laufende Instanzen)
│
├── sidebar_left/       # Modul für Linke Seitenleiste (Navigation, Workspace Switcher)
│   ├── mod.rs
│   ├── sidebar_left_widget.rs # Das Haupt-Widget für die linke Seitenleiste (gtk::Box, ggf. mit Ein-/Ausklapp-Logik)
│   ├── workspace_switcher_widget.rs # Widget für den Workspace-Switcher (gtk::ListView oder custom Widget, zeigt Workspace Icons/Namen)
│   ├── global_search_widget.rs # Suchfeld für die globale Suche
│   └── launcher_shortcut_widget.rs # Button oder Icon zum Öffnen des Anwendungsstarters
│
├── sidebar_right/      # Modul für Rechte Seitenleiste (Widgets)
│   ├── mod.rs
│   ├── sidebar_right_widget.rs # Das Haupt-Widget für die rechte Seitenleiste (gtk::Box, ggf. mit Ein-/Ausklapp-Logik)
│   └── widget_host.rs  # Container-Widget, das einzelne Widgets aufnimmt und verwaltet (Drag & Drop Integration)
│
├── widgets/            # Modul für individuelle Widget-Implementierungen
│   ├── mod.rs
│   ├── clock_widget.rs       # Widget zur Anzeige von Uhrzeit und Datum
│   ├── weather_widget.rs     # Widget zur Anzeige des Wetters (Interaktion mit Domain/KI/System für Daten)
│   ├── system_monitor_widget.rs # Widget zur Anzeige von CPU/RAM/Netzwerk-Auslastung (Interaktion mit Systemschicht)
│   ├── notes_widget.rs       # Widget für schnelle Notizen
│   ├── todo_widget.rs        # Widget für To-Do-Listen
│   ├── media_controls_widget.rs # Widget zur Steuerung der Medienwiedergabe (Interaktion mit Systemschicht - MPRIS)
│   # ... weitere Widget-Implementierungen
│
├── speed_dial/         # Modul für Speed-Dial Startansicht (neuer Workspace oder leere Ansicht)
│   ├── mod.rs
│   ├── speed_dial_widget.rs # Das Haupt-Widget für die Speed-Dial Ansicht
│   ├── favorites_widget.rs  # Widget für Favoriten (Apps, Dateien, Ordner)
│   ├── recent_items_widget.rs # Widget für kürzlich genutzte Elemente
│   ├── suggested_actions_widget.rs # Widget für kontextbasierte Aktionsvorschläge (KI-Integration)
│   └── speed_dial_search.rs # Suchfeld für die integrierte Suche im Speed-Dial
│
├── overview_mode/      # Modul für Übersichtsmodus (Fenster-Cockpit)
│   ├── mod.rs
│   ├── overview_widget.rs # Das Haupt-Widget für den Übersichtsmodus (Gitter-Layout oder ähnliches)
│   ├── window_thumbnail_widget.rs # Widget zur Anzeige einer einzelnen Fenster-Miniaturansicht (Live-Vorschau)
│   ├── workspace_preview_widget.rs # Widget zur Anzeige einer Vorschau eines Workspaces (im Übersichtsmodus)
│   └── window_grouping.rs # Logik zur optionalen Gruppierung von Fenstern pro Anwendung
│
├── command_palette/    # Modul für Kontextuelle Befehlspalette
│   ├── mod.rs
│   ├── command_palette_widget.rs # Das Popover- oder Overlay-Widget für die Befehlspalette
│   └── command_list_widget.rs # Widget zur Anzeige der verfügbaren Befehle und Suchergebnisse (gtk::ListView)
│
├── notifications_ui/   # Modul für Benachrichtigungs-UI
│   ├── mod.rs
│   ├── notification_display_widget.rs # Widget zur Anzeige einzelner Benachrichtigungs-Popups (Toast-Nachrichten)
│   └── notification_center_widget.rs # Widget für das zentrale Benachrichtigungszentrum (Liste der Benachrichtigungen)
│
├── theming_ui/         # Modul für Theme-Anpassungs-UI
│   ├── mod.rs
│   └── theme_settings_widget.rs # UI-Komponenten und Logik zur Konfiguration von Themes, Akzentfarben, etc.
│
├── components/         # Modul für wiederverwendbare UI-Komponenten
│   ├── mod.rs
│   ├── button.rs
│   ├── label.rs
│   ├── slider.rs
│   ├── switch.rs
│   ├── icon.rs
│   └── list_item.rs    # Generisches Widget für Elemente in Listenansichten
│   # ... weitere UI-Komponenten
│
└── dialogs/            # Modul für Standard-Dialoge (Modale Popups)
    ├── mod.rs
    ├── consent_dialog.rs # Dialog für KI-Einwilligungsabfragen
    ├── error_dialog.rs   # Dialog zur Anzeige von Fehlermeldungen
    ├── confirmation_dialog.rs # Dialog für Bestätigungsabfragen (z.B. "Möchten Sie wirklich herunterfahren?")
    └── file_chooser_dialog.rs # Integration des XDG File Chooser Portals oder native GTK Implementierung
```