**Technische Gesamtspezifikation & Richtlinien: Linux Desktop-Umgebung "NovaDE" (Kompakte Gesamtdefinition inkl. Features)**

**I. Vision und Kernziele**

- **Vision:** NovaDE (Nova Desktop Environment) ist eine innovative Linux-Desktop-Umgebung, die eine moderne, schnelle, intuitive und KI-gestützte Benutzererfahrung schafft. Sie ist optimiert für Entwickler, Kreative und alltägliche Nutzer und zielt darauf ab, Produktivität und Freude an der Interaktion mit dem System zu maximieren.
- **Kernziele:** Performance, Intuition, Modernität, Modularität & Wartbarkeit, Anpassbarkeit, sichere KI-Integration, Stabilität & Sicherheit.

**II. Architektonischer Überblick: Vier-Schichten-Architektur**

NovaDE basiert auf einer strengen, vier-schichtigen Architektur (Kern, Domäne, System, Benutzeroberfläche) für Modularität, lose Kopplung und hohe Kohäsion. Kommunikation erfolgt über wohldefinierte Schnittstellen.

1. **Kernschicht (Core Layer):**
    
    - **Verantwortlichkeiten:** Fundamentale Datentypen (z.B. `Point<T>`, `Color`), Dienstprogramme, Konfigurationsprimitive (TOML, Serde), Logging (`tracing`), Basis-Fehler (`thiserror`).
    - **Featurespiegelung:** Stellt die atomaren Bausteine für alle visuellen und logischen Elemente bereit.
2. **Domänenschicht (Domain Layer):**
    
    - **Verantwortlichkeiten:** UI-unabhängige Geschäftslogik.
        - `domain::theming`: Logik für das Erscheinungsbild, Design-Token-Verwaltung, dynamische Theme-Wechsel (Hell/Dunkel, Akzentfarben).
        - `domain::workspaces`: Verwaltung von Arbeitsbereichen ("Spaces"), Fensterzuweisung, Workspace-Orchestrierung und -Persistenz.
        - `domain::user_centric_services`: Logik für KI-Interaktionen (inkl. Einwilligungsmanagement für Datenkategorien wie `FileSystemRead`, `ClipboardAccess`), Benachrichtigungsverwaltung.
        - `domain::notifications_rules`: Regelbasierte, dynamische Verarbeitung von Benachrichtigungen.
        - `domain::global_settings_and_state_management`: Verwaltung globaler Desktop-Einstellungen.
        - `domain::window_management_policy`: Richtlinien für Fensterplatzierung, automatisches Tiling (Layouts: Spalten, Spiralen), Snapping, Fenstergruppierung, Gap-Management.
    - **Featurespiegelung:** Definiert _was_ personalisierbar ist (Themes, Akzente), _wie_ Arbeitsbereiche funktionieren (Spaces mit Icons, gepinnten Apps), _wie_ KI sicher und mit Zustimmung agiert und _welche_ Regeln für Fenster gelten.
3. **Systemschicht (System Layer):**
    
    - **Verantwortlichkeiten:** OS-Interaktion, technische Umsetzung der Domänenrichtlinien.
        - `system::compositor`: Smithay-basierter Wayland-Compositor (Implementierung von `xdg-shell`, `wlr-layer-shell-unstable-v1`, etc.), XWayland.
        - `system::input`: `libinput`-basierte Eingabeverarbeitung, Gestenerkennung, Seat-Management (`xkbcommon`).
        - `system::dbus`: `zbus`-Schnittstellen zu Systemdiensten (NetworkManager, UPower, logind, org.freedesktop.Notifications, org.freedesktop.secrets, PolicyKit).
        - `system::outputs`: Monitorkonfiguration (Auflösung, Skalierung, DPMS über `wlr-output-management`).
        - `system::audio`: PipeWire-Client (`pipewire-rs`) für Audio-Management.
        - `system::mcp`: MCP-Client (`mcp_client_rs`) für KI-Modell-Kommunikation.
        - `system::portals`: Backend für XDG Desktop Portals (FileChooser, Screenshot).
        - `system::window_mechanics`: Technische Umsetzung des Fenstermanagements (Positionierung, Anwendung von Tiling-Layouts, Fokus, Fensterdekorationen). Technische Basis für die "Intelligente Tab-Leiste".
    - **Featurespiegelung:** Ermöglicht flüssige Darstellung (Wayland), präzise Eingabe (`libinput`, Gesten), Integration mit Systemdiensten für Energie, Netzwerk, Sound (PipeWire) und sichere KI-Kommunikation (MCP). Setzt Fensterregeln (Tiling, Snapping) technisch um.
4. **Benutzeroberflächenschicht (User Interface Layer):**
    
    - **Verantwortlichkeiten:** Grafische Darstellung, Benutzerinteraktion (GTK4, `gtk4-rs`).
        - `ui::shell`:
            - **Kontroll-/Systemleiste(n) (PanelWidget):** Module für AppMenu, Workspace-Indikator, Uhr, System-Tray, Schnelleinstellungen, Benachrichtigungszentrum, Netzwerk-, Energie-, Audio-Indikatoren. _Elegante Leiste mit optionalem Leuchtakzent._
            - **Intelligente Tab-Leiste (SmartTabBarWidget):** Pro "Space", mit ApplicationTabWidgets für "angepinnte" Apps/Split-Views, aktive Tabs mit Akzentfarbe. _Moderne Tabs mit abgerundeten oberen Ecken._
            - **Schnelleinstellungs-Panel (QuickSettingsPanelWidget):** Ausklappbar für WLAN, Bluetooth, Lautstärke, Dark Mode.
            - **Workspace-Switcher (WorkspaceSwitcherWidget):** Adaptive linke Seitenleiste mit SpaceIconWidgets (Icons der gepinnten App oder benutzerdefiniert) für schnelle Navigation zwischen "Spaces", mit Hervorhebung des aktiven Space. _Bei Mouse-Over/Geste aufklappbar mit Namen/Vorschau._
            - **Schnellaktionsdock (QuickActionDockWidget):** Konfigurierbares Dock (schwebend/angedockt) für Apps, Dateien, Aktionen; intelligente Vorschläge, Tastaturbedienung.
            - **Benachrichtigungszentrum (NotificationCenterPanelWidget):** Anzeige von Benachrichtigungsliste und -historie.
        - `ui::control_center`: Modulare GTK4-Anwendung für alle Systemeinstellungen (Erscheinungsbild, Netzwerk, etc.) mit Live-Vorschau.
        - `ui::widgets`:
            - **Adaptive rechte Seitenleiste (RightSidebarWidget):** Optional, mit dezent transluzentem Hintergrund für informative Widgets (Uhr, Kalender, Wetter, Systemmonitor), per Drag & Drop anpassbar.
            - WidgetManagerService, WidgetPickerPopover.
        - `ui::window_manager_frontend`:
            - **Client-Side Decorations (CSD):** Logik (z.B. via `Gtk::HeaderBar`).
            - **Übersichtsmodus (OverviewModeWidget):** Fenster- und Workspace-Übersicht als interaktive Kacheln mit Live-Vorschau, Drag & Drop von Fenstern zwischen Spaces. _Hintergrund abgedunkelt/unscharf._
            - AltTabSwitcherWidget.
        - `ui::notifications_frontend`: **Pop-up-Benachrichtigungen (NotificationPopupWidget):** Dezent, im Dark Mode Stil mit Akzentfarbe für Dringlichkeit.
        - `ui::theming_gtk`: Anwendung von CSS-Stilen aus `domain::theming` via `GtkCssProvider`.
        - `ui::speed_dial`: GTK4-Implementierung der Startansicht für leere Workspaces mit Favoriten und intelligenten Vorschlägen.
        - `ui::command_palette`: GTK4-Implementierung der kontextuellen Befehlspalette (`Super+Space`).
    - **Featurespiegelung:** Setzt die gesamte beschriebene Nutzererfahrung um: dunkle Ästhetik mit Akzentfarben, Panel(s), intelligente Tab-Leiste, adaptive Seitenleisten mit Widgets, Workspace-Switcher, Schnellaktionsdock, Control Center, Speed-Dial, Übersichtsmodus und die kontextuelle Befehlspalette. Ermöglicht die Personalisierung und direkte Manipulation.

**III. Technologie-Stack (Verbindliche Auswahl)**

Rust, Meson, GTK4 (`gtk4-rs`), Smithay Toolkit, Wayland (xdg-shell, wlr-Protokolle), D-Bus (`zbus`), Model Context Protocol (MCP), `libinput`, PipeWire (`pipewire-rs`), Freedesktop Secret Service API, PolicyKit, Token-basiertes CSS-Theming, XDG Desktop Portals.

**IV. Entwicklungsrichtlinien (Verbindlich)**

- **Rust:** `rustfmt`, Rust API Guidelines, `thiserror` pro Modul, `Result<T,E>`, `tracing` für Logging, `async/await` (Tokio, GLib).
- **Allgemein:** Git (GitHub Flow), Conventional Commits, umfassende Tests (Unit, Integration, Compositor, UI), CI-Pipeline, detaillierte Dokumentation (rustdoc, Architektur, READMEs).

**V. Deployment-Überlegungen**

Native Pakete (.deb, .rpm), Flatpak (evaluieren), Integration mit Display Managern, `systemd` User Sessions, PAM, XDG Base Directory Specification. SemVer.
