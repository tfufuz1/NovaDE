**Grundprinzipien der UI-Schicht-Kommunikation:**

1. **Technologie:** Basiert vollständig auf GTK4 und den `gtk4-rs` Rust-Bindings [Gesamtspezifikationen.md II.4; D1 UI-Schicht.md 1.4].
2. **Reaktivität:** Die UI muss hochgradig reaktiv auf Benutzerinteraktionen und Änderungen in den unteren Schichten sein. Dies wird erreicht durch:
    - **GTK-Signale:** Standardmechanismus für Widget-Interaktionen (z.B. `button.connect_clicked(...)`).
    - **Domänen- und System-Events:** UI-Komponenten abonnieren Events von Services der Domänen- und Systemschicht (über `tokio::sync::broadcast::Receiver`) und aktualisieren ihren Zustand und ihre Darstellung im GTK-Hauptthread (`glib::MainContext::spawn_local`).
    - **GObject-Properties und Bindings:** UI-spezifischer Zustand wird oft in GObject-Properties von benutzerdefinierten Widgets gehalten. GTK-Properties-Bindings werden genutzt, um UI-Elemente direkt an diese Zustände oder an Datenmodelle zu binden [D1 UI-Schicht.md 3.1].
3. **Aktionsauslösung:** Benutzeraktionen in der UI (z.B. Klick auf einen Button zum Speichern von Einstellungen, Fenster verschieben) führen zu Aufrufen von Methoden auf Services der Domänen- oder Systemschicht.
4. **Fehlerdarstellung:** Fehler, die von unteren Schichten empfangen werden, müssen von der UI-Schicht benutzerfreundlich dargestellt werden (z.B. über `gtk::AlertDialog` oder Inline-Fehlermeldungen). Die UI-Schicht definiert eigene `thiserror`-basierte Fehler (z.B. `NovaUiError`), die Fehler aus tieferen Schichten wrappen können [D1 UI-Schicht.md 3.2].
5. **Datenfluss (UI zu Domäne/System):**
    - Benutzereingabe -> GTK-Signal-Handler im UI-Widget.
    - Signal-Handler validiert ggf. Eingabe und ruft Methode eines entsprechenden Domänen- oder System-Service auf.
    - Parameter für Service-Aufrufe sind oft primitive Typen oder domänenspezifische Datenstrukturen.
6. **Datenfluss (Domäne/System zu UI):**
    - Zustandsänderung in Domäne/System -> Event wird ausgelöst.
    - UI-Komponente (Subscriber) empfängt Event im GTK-Hauptthread.
    - UI-Komponente aktualisiert ihre GObject-Properties, Widget-Attribute (z.B. Label-Text, Sichtbarkeit) oder Listenmodelle (z.B. `GtkListStore`, `GioListStore`).
    - GTK rendert die Änderungen.

---

**Schnittstellen im Detail – Fokus UI-Schicht:**

Die vorherige Definition wird hier um die spezifischen Interaktionen und Datenflüsse der UI-Schicht erweitert und verfeinert.

**A. Benutzeroberflächenschicht (UI Layer) ZU Domänenschicht (Domain Layer)**

Die UI-Schicht ist ein primärer Konsument der von der Domänenschicht bereitgestellten Logik und Zustände.

- **Nutzung der Domänen-Service-APIs:**
    - **Abhängigkeiten:** UI-Module erhalten typischerweise Referenzen (`Arc<dyn ServiceTrait>`) auf Domänen-Services über Dependency Injection beim Start der Anwendung oder des jeweiligen UI-Teils.
    - **Methodenaufrufe:** UI-Komponenten rufen `async` Methoden der Domänen-Service-Traits auf. Da GTK-Signal-Handler synchron sind, müssen diese Aufrufe in einem `glib::MainContext::spawn_local` Block erfolgen, um den UI-Thread nicht zu blockieren. Das Ergebnis des `async`-Aufrufs (`Result<T, DomänenError>`) wird dann im Callback des `spawn_local` verarbeitet.
        - **Beispiel (`ui::control_center::AppearanceSettingsWidget` zu `domain::theming::ThemingEngine`):**
            - Ein `GtkDropDown` für die Theme-Auswahl hat einen `connect_changed` Signal-Handler.
            - Im Handler: `let theming_engine_clone = self.theming_engine.clone(); let new_theme_id_str = selected_item.string(); let current_config_clone = self.current_theming_config.clone();`
            - `glib::MainContext::default().spawn_local(async move { ... })`:
                - `new_config = current_config_clone.with_selected_theme(ThemeIdentifier::new(new_theme_id_str));`
                - `match theming_engine_clone.update_configuration(new_config).await { Ok(_) => tracing::info!("Theme-Konfiguration aktualisiert."), Err(e) => display_error_dialog(e); }`
        - **Beispiel (`ui::shell::WorkspaceSwitcherWidget` zu `domain::workspaces::WorkspaceManagerService`):**
            - Klick auf ein `SpaceIconWidget`:
            - `glib::MainContext::default().spawn_local(async move { workspace_manager.set_active_workspace(target_workspace_id).await; });`
    - **Parameterübergabe:** UI-Komponenten übergeben Daten (oft aus Benutzereingaben oder UI-Zustand) als Parameter an die Domänen-Service-Methoden. Diese Daten müssen ggf. von GTK-Typen (z.B. `GString`) in Domänen-Typen (z.B. `String`, `SettingValue`) konvertiert werden.
    - **Fehlerbehandlung:** `Result<_, DomänenError>` wird von Domänen-Services zurückgegeben. UI-Komponenten müssen diese Fehler behandeln:
        - Logging des detaillierten Fehlers (`tracing::error!(error = ?err, "Domänenoperation fehlgeschlagen");`).
        - Anzeige einer benutzerfreundlichen Fehlermeldung (z.B. über `gtk::AlertDialog` oder ein Inline-Benachrichtigungs-Widget). Die Fehlermeldung sollte i18n-fähig sein und den technischen Fehler nicht direkt exponieren.
        - Ggf. UI-Zustand zurücksetzen oder alternative Aktionen anbieten.
- **Konsumieren von Domänen-Events:**
    - **Abonnement:** UI-Komponenten abonnieren relevante Domänen-Events über die `subscribe_to_XYZ_events()` Methoden der Domänen-Services. Dies gibt einen `tokio::sync::broadcast::Receiver<EventTyp>` zurück.
        
    - **Event-Verarbeitung im UI-Thread:** Ein `tokio::task` (gestartet mit `glib::MainContext::spawn_local` oder über eine Brücke, wenn der Receiver in einem anderen Executor läuft) lauscht auf dem `Receiver`.
        
        - **Beispiel (`ui::theming_gtk` abonniert `ThemeChangedEvent`):**
            
            Rust
            
            Rust
            
            ```
            // In der Initialisierung von ui::theming_gtk
            let theming_engine = /* ... */;
            let mut theme_change_receiver = theming_engine.subscribe_to_theme_changes();
            let css_provider = self.gtk_css_provider.clone(); // GtkCssProvider
            
            glib::MainContext::default().spawn_local(async move {
                while let Ok(event) = theme_change_receiver.recv().await {
                    tracing::info!("ThemeChangedEvent empfangen in ui::theming_gtk");
                    let new_css_data = generate_css_from_resolved_tokens(&event.new_state.resolved_tokens);
                    css_provider.load_from_data(new_css_data.as_bytes());
                    // Ggf. gtk::StyleContext::reset_widgets(&display) aufrufen, obwohl GTK4 dies oft automatisch handhabt.
                }
            });
            ```
            
        - **Beispiel (`ui::shell::WorkspaceIndicatorWidget` abonniert `WorkspaceEvent`):**
            
            Rust
            
            Rust
            
            ```
            // In Initialisierung des WorkspaceIndicatorWidget
            let workspace_manager = /* ... */;
            let mut workspace_event_receiver = workspace_manager.subscribe_to_workspace_events();
            let self_weak = self.downgrade(); // Um Zyklen zu vermeiden
            
            glib::MainContext::default().spawn_local(async move {
                while let Ok(event) = workspace_event_receiver.recv().await {
                    if let Some(widget_instance) = self_weak.upgrade() {
                        match event {
                            WorkspaceEvent::ActiveWorkspaceChanged { new_id, .. } => {
                                widget_instance.imp().update_active_indicator(new_id);
                            }
                            WorkspaceEvent::WorkspaceCreated { id, name, position, .. } => {
                                widget_instance.imp().add_workspace_representation(id, name, position);
                            }
                            // ... andere Event-Arme ...
                        }
                    } else { break; /* Widget wurde zerstört */ }
                }
            });
            ```
            
    - **Datenextraktion:** Die UI-Komponente extrahiert die relevanten Daten aus der Event-Payload.
        
    - **UI-Aktualisierung:** Basierend auf den Event-Daten werden GTK-Widget-Properties gesetzt, Listenmodelle (`GtkListStore`, `GioListStore`) aktualisiert oder Widgets neu gezeichnet (`queue_draw()`). Alle UI-Manipulationen _müssen_ im GTK-Hauptthread erfolgen.
        
        - **Beispiel:** `self.label_widget.set_text(&event.new_data_string);`
        - **Beispiel:** `self.list_store_model.append(&new_row_data);`

**B. Benutzeroberflächenschicht (UI Layer) ZU Systemschicht (System Layer)**

Die UI-Schicht interagiert mit der Systemschicht, um systemnahe Informationen darzustellen und systemnahe Aktionen auszulösen, die nicht direkt durch Domänenlogik abgedeckt sind oder eine direkte technische Umsetzung erfordern.

- **Empfang von systemnahen Diensten und Ereignissen:**
    
    - **Fensterinformationen & Management-Befehle:**
        - **UI-Module betroffen:** `ui::shell` (Panel, Taskleiste), `ui::window_manager_frontend` (Übersicht, Alt+Tab, Fensterdekorationen falls CSD), `ui::shell::SmartTabBarWidget`.
        - **Schnittstelle von Systemschicht (konzeptionell, oft über `DesktopState` oder einen Event-Bus wie `SystemEventBridge`):**
            - **Event-Strom:**
                - `SystemLayerEvent::WindowCreated { window_id: DomainWindowIdentifier, initial_geometry: RectInt, title: String, app_id: ApplicationId, is_toplevel: bool, is_popup: bool, parent_id: Option<DomainWindowIdentifier> }`
                - `SystemLayerEvent::WindowClosed { window_id: DomainWindowIdentifier }`
                - `SystemLayerEvent::WindowGeometryChanged { window_id: DomainWindowIdentifier, new_geometry: RectInt }`
                - `SystemLayerEvent::WindowTitleChanged { window_id: DomainWindowIdentifier, new_title: String }`
                - `SystemLayerEvent::WindowAppIdChanged { window_id: DomainWindowIdentifier, new_app_id: ApplicationId }`
                - `SystemLayerEvent::WindowFocusChanged { old_focus_id: Option<DomainWindowIdentifier>, new_focus_id: Option<DomainWindowIdentifier> }`
                - `SystemLayerEvent::WindowMinimized { window_id: DomainWindowIdentifier }`
                - `SystemLayerEvent::WindowMaximized { window_id: DomainWindowIdentifier }`
                - `SystemLayerEvent::WindowFullscreened { window_id: DomainWindowIdentifier }`
                - `SystemLayerEvent::WindowRestored { window_id: DomainWindowIdentifier, previous_state: ... }`
            - **Abfrage-API (z.B. über einen `WindowManagerSystemFacade` Trait, implementiert von `DesktopState` oder einer dedizierten System-Service-Struktur):**
                - `async fn get_all_mapped_windows_for_workspace(&self, workspace_id: WorkspaceId) -> Result<Vec<WindowPresentationInfo>, SystemError>;` (wobei `WindowPresentationInfo` ID, Titel, AppID, Icon-Name, Geometrie, Zustand enthält)
                - `async fn get_window_thumbnail(&self, window_id: DomainWindowIdentifier, width: u32, height: u32) -> Result<Option<ImageData>, SystemError>;` (`ImageData` enthält Pixeldaten oder einen Pfad zu einem Bild). Dies erfordert Interaktion mit `system::compositor::screencopy`.
        - **Nutzung durch UI:**
            - UI-Komponenten abonnieren die `Window*` Events vom `SystemEventBridge`.
            - Bei `WindowCreated`: Neues Widget für das Fenster in Taskleiste/Übersicht erstellen.
            - Bei `WindowClosed`: Entsprechendes Widget entfernen.
            - Bei `Geometry/Title/FocusChanged`: Darstellung des Widgets aktualisieren.
            - `ui::window_manager_frontend::OverviewModeWidget` ruft `get_all_mapped_windows_for_workspace` ab, um Fensterkacheln zu rendern.
    - **Eingabeereignisse (Globale Hotkeys, Shell-Gesten):**
        - **UI-Module betroffen:** `ui::shell` (für globale Aktionen), `ui::command_palette`.
        - **Schnittstelle von Systemschicht (`system::input` via `SystemEventBridge` oder dedizierter Service):**
            - Event `SystemLayerEvent::GlobalShortcutActivated { shortcut_id: String }` (definiert in `system::input` und via Bridge gesendet).
            - Event `SystemLayerEvent::ShellGestureTriggered { gesture_type: ShellGestureType (z.B. ThreeFingerSwipeLeft), details: ... }`.
        - **Nutzung durch UI:**
            - `ui::shell` abonniert `GlobalShortcutActivated`. Löst Aktionen aus (z.B. `ui::command_palette` anzeigen bei `Super+Space`).
            - `ui::shell` abonniert `ShellGestureTriggered`. Löst Aktionen aus (z.B. Workspace-Wechsel).
            - _Normale Widget-Eingaben (Klicks, Text) werden direkt von GTK über Wayland-Events (die vom Compositor kommen) verarbeitet und benötigen keine spezielle Schnittstelle hier, außer dass der Compositor die Events korrekt an das fokussierte GTK-Fenster weiterleitet._
    - **Statusänderungen von Systemdiensten (Netzwerk, Energie, Audio etc.):**
        - **UI-Module betroffen:** `ui::shell::PanelWidget` (Indikatoren für Netzwerk, Batterie, Audio, Mikrofon, Bluetooth), `ui::shell::QuickSettingsPanelWidget`.
        - **Schnittstelle von Systemschicht (via `system::dbus_interfaces::*` und `system::audio_management`, publiziert über `SystemEventBridge`):**
            - `SystemLayerEvent::UPower(UPowerEvent)` (z.B. `OnBatteryChanged { is_on_battery: bool }`, `DeviceChanged { path, new_percentage, new_state }`).
            - `SystemLayerEvent::Logind(LogindEvent)` (z.B. `SessionLocked`, `PrepareForSleep`).
            - `SystemLayerEvent::NetworkManager(NetworkManagerEvent)` (z.B. `ConnectivityChanged { new_state: NMConnectivityState }`, `DeviceStateChanged { device_path, new_nm_device_state }`, `ActiveConnectionChanged { details: ActiveConnectionDetails }`).
            - `SystemLayerEvent::Audio(AudioEvent)` (z.B. `DeviceVolumeChanged { device_pw_id, new_volume_percent, is_muted }`, `DefaultDeviceChanged { device_type, new_default_device }`).
        - **Nutzung durch UI:**
            - Indikator-Widgets (z.B. `NetworkIndicatorWidget`) abonnieren die relevanten Events vom `SystemEventBridge`.
            - Bei Event-Empfang: `glib::MainContext::spawn_local` verwenden, um das Widget (z.B. Icon, Label, Tooltip) zu aktualisieren.
                - Beispiel: `BatteryIndicatorWidget` reagiert auf `UPowerEvent::DeviceChanged` für sein primäres Batteriegerät und aktualisiert die Prozentanzeige und das Icon.
    - **Monitor-/Output-Änderungen:**
        - **UI-Module betroffen:** `ui::control_center::DisplaySettingsWidget`, `ui::shell` (für korrekte Panel-Platzierung und -Größe auf jedem Monitor).
        - **Schnittstelle von Systemschicht (`system::outputs` via `SystemEventBridge` oder direkter API):**
            - Event `SystemLayerEvent::OutputConfigurationChanged { outputs: Vec<OutputInfoStruct> }` (wobei `OutputInfoStruct` Name, ID, aktuelle Auflösung, Position, Skala, Transformation, verfügbare Modi, ist_aktiv, ist_primär enthält).
            - API (z.B. auf einem `OutputSystemFacade` Trait): `async fn get_current_output_configuration() -> Result<Vec<OutputInfoStruct>, SystemError>;` `async fn apply_output_configuration(config_requests: Vec<OutputConfigRequest>) -> Result<(), SystemError>;` (wobei `OutputConfigRequest` Änderungen für einen bestimmten Output spezifiziert).
        - **Nutzung durch UI:**
            - `ui::control_center::DisplaySettingsWidget` ruft `get_current_output_configuration` für die Anzeige auf. Sendet `apply_output_configuration` bei Benutzeränderungen. Abonniert `OutputConfigurationChanged` für dynamische Updates.
            - `ui::shell::PanelWidget` abonniert `OutputConfigurationChanged`, um sich ggf. neu zu positionieren oder Instanzen pro Monitor zu erstellen/entfernen.
- **Technische Umsetzung von UI-Befehlen durch die Systemschicht:**
    
    - **Fenstermanipulationen (z.B. Klick auf "Schließen"-Button in CSD, Aktion aus Fenstermenü):**
        - **Ablauf:** UI-Widget -> `system::window_mechanics` (oder eine Fassade) -> `system::compositor` (`XdgToplevelSurface::send_close()` oder andere XDG-Requests).
        - **Schnittstelle Systemschicht (Beispiel):** `async fn request_window_close(&self, window_id: DomainWindowIdentifier) -> Result<(), WindowMechanicsError>;` `async fn request_window_maximize(&self, window_id: DomainWindowIdentifier) -> Result<(), WindowMechanicsError>;`
    - **Workspace-Wechsel (ausgelöst durch UI-Geste oder Klick im Switcher):**
        - **Ablauf:** UI-Widget -> `domain::workspaces::WorkspaceManagerService::set_active_workspace(...)`. Die Systemschicht (Compositor) reagiert dann auf das von der Domäne gesendete `ActiveWorkspaceChanged`-Event.
    - **Anforderung von Systeminformationen/Aktionen für UI-Elemente:**
        - **Beispiel: WLAN-Auswahl im `QuickSettingsPanelWidget`:**
            1. Widget ruft `system::dbus_interfaces::NetworkManagerClientService::scan_wifi_access_points()` auf (oder es abonniert ein Event, das die AP-Liste liefert).
            2. Benutzer wählt AP aus.
            3. Widget ruft `system::dbus_interfaces::NetworkManagerClientService::activate_connection(ap_connection_path, wifi_device_path)` auf.
            4. Systemschicht führt D-Bus-Aufruf aus.
            5. NetworkManager-Events werden über den `SystemEventBridge` an die UI zurückgesendet, um den Verbindungsstatus zu aktualisieren.
    - **Dialoge über XDG Desktop Portals (`system::portals`):**
        - **UI-Module betroffen:** Jedes UI-Element, das einen Datei-Dialog, Screenshot, Farbauswahl etc. benötigt. Oft gekapselt in `ui::components` (z.B. `AsyncFileChooserButton`).
        - **Schnittstelle Systemschicht:** Die Portal-Backends in `system::portals` implementieren die D-Bus-Schnittstellen (`org.freedesktop.portal.FileChooser` etc.). Die UI-Anwendung agiert als _Client_ dieser Portale.
        - **Nutzung durch UI:**
            - Verwendung von `ashpd` (Rust-Bindings für XDG Portals - Client-Seite) oder direkten `zbus`-Aufrufen an die Portal-D-Bus-Schnittstellen.
            - Beispiel `FileChooser`: `let files = ashpd::desktop::file_chooser::OpenFileRequest::new().title("Öffne eine Datei").send().await?.response()?;`
            - `system::portals` (als D-Bus-Server) empfängt diese Anfrage, interagiert mit der UI-Schicht (z.B. dem `ui::shell` oder einem dedizierten `ui::portal_dialog_handler`), um einen nativen GTK-Dateiauswahldialog anzuzeigen (`gtk::FileChooserNative`).
            - Das Ergebnis des GTK-Dialogs wird von `system::portals` an den anfragenden Client (die UI-Komponente, die `ashpd` nutzte) zurückgesendet.
- **Renderer-Schnittstelle (`system::compositor::renderer_interface`):**
    
    - **Schnittstelle:** Die Traits `FrameRenderer` und `RenderableTexture`.
    - **Nutzung durch UI:** _Nicht direkt._ Der Compositor (`system::compositor::core`) verwendet eine konkrete Implementierung dieser Schnittstelle (z.B. `DrmGbmRenderer`), um die `WlSurface`s (die GTK-Fenster enthalten) und andere Shell-Elemente (Panels, Cursor, die auch Wayland-Oberflächen sein können) zu zeichnen. GTK4 zeichnet seinen eigenen Inhalt in seine `WlSurface`-Puffer.
- **Fehlerbehandlung (von Systemschicht an UI):**
    
    - Systemschicht-Methoden geben `Result<_, SystemModulError>` zurück.
    - UI-Code, der diese Methoden aufruft, muss die Fehler behandeln:
        - Logging des `SystemModulError` (inkl. `source()`).
        - Anzeige einer generischen, benutzerfreundlichen Fehlermeldung: "Aktion konnte nicht ausgeführt werden." oder "Systemfehler aufgetreten."
        - In spezifischen Fällen (z.B. `DBusInterfaceError::ServiceUnavailable` für NetworkManager) kann die UI eine spezifischere Meldung anzeigen ("Netzwerkdienst nicht verfügbar.").
        - Verwendung von `gtk::AlertDialog` oder ähnlichen Mechanismen.

**IV. UI-Schicht Interne Schnittstellen (Zwischen UI-Modulen)**

Die Kommunikation und Abhängigkeiten innerhalb der UI-Schicht selbst (zwischen z.B. `ui::shell`, `ui::control_center`, `ui::widgets`) sind ebenfalls wichtig.

- **GObject-Properties und Signale:**
    - Benutzerdefinierte GTK-Widgets (z.B. `PanelWidget`, `SmartTabBarWidget`) definieren GObject-Properties für ihren Zustand und ihr Aussehen.
    - Sie definieren benutzerdefinierte GObject-Signale für Aktionen oder Zustandsänderungen, die für andere UI-Teile relevant sind.
        - **Beispiel:** `PanelWidget` könnte ein Signal `module-layout-changed` emittieren [D1 UI-Schicht.md 2.1.6].
        - **Beispiel:** `SmartTabBarWidget` könnte ein Signal `tab-selected(tab_id)` emittieren.
- **Datenmodelle (z.B. für Listen und Bäume):**
    - Verwendung von `Gio::ListStore` (oft gewrappt in `Gtk::FilterListModel` oder `Gtk::SortListModel`) für dynamische Listen in Widgets (z.B. Benachrichtigungsliste im `NotificationCenterPanelWidget`, Fensterliste im `OverviewModeWidget`).
    - Diese Modelle werden basierend auf Events aus der Domänen- oder Systemschicht aktualisiert.
- **Direkte Methodenaufrufe zwischen UI-Komponenten:**
    - Wenn eine klare Eltern-Kind-Beziehung oder eine enge Kopplung besteht (z.B. `PanelWidget` verwaltet seine `AppMenuButtonWidget`-Instanz).
    - Sollte minimiert werden zugunsten von Signalen/Events oder Datenbindung für lose Kopplung.
- **Gemeinsam genutzter UI-Zustand (`Rc<RefCell<UiState>>`):**
    - Für UI-Zustand, der von mehreren, nicht direkt hierarchisch verbundenen Widgets gemeinsam genutzt wird (z.B. Zustand der `QuickSettingsPanelWidget`-Sichtbarkeit, der von einem Button im Panel und einem globalen Shortcut beeinflusst wird). [D1 UI-Schicht.md 3.1]
    - Der `UiState` würde GObject-Properties oder Rust-Felder enthalten und Änderungen über `notify` oder Rust-Callbacks kommunizieren.
- **UI-spezifische Fehlerbehandlung (`NovaUiError`):**
    - Das `ui::errors`-Modul definiert `NovaUiError`, das Fehler aus unteren Schichten wrappen oder UI-spezifische Fehler (z.B. "Widget konnte nicht geladen werden", "Ungültige UI-Konfiguration") darstellen kann. [D1 UI-Schicht.md 3.2]
- **Interaktion `ui::shell` mit `ui::control_center`:**
    - `ui::shell` (z.B. ein Button in `QuickSettingsPanelWidget`) kann einen Befehl zum Öffnen des `ui::control_center` senden (z.B. über eine globale Aktions-API oder D-Bus-Aktivierung, falls `ControlCenter` eine separate Anwendung ist).
    - Beide lesen und schreiben Einstellungen über den `GlobalSettingsService` der Domänenschicht.
- **Interaktion `ui::shell` mit `ui::widgets` (`RightSidebarWidget`, `WidgetManagerService`):**
    - `RightSidebarWidget` enthält eine Liste von `PlacedWidgetWidget`.
    - `WidgetManagerService` (eine logische UI-Komponente, kein Domänen-Service) verwaltet die verfügbaren Widget-Typen und deren Konfiguration.
    - `WidgetPickerPopover` (aus `ui::widgets`) wird vom `RightSidebarWidget` verwendet, um Widgets hinzuzufügen. Interagiert mit `WidgetManagerService`, um verfügbare Widgets aufzulisten.
    - Drag & Drop von Widgets in der Sidebar wird von GTK-Drag-and-Drop-Signalen gehandhabt und vom `RightSidebarWidget` verarbeitet.

**Zusammenfassende Prinzipien der UI-Schnittstellen:**

1. **GTK-Idiome nutzen:** GObject-Properties, Signale, Datenmodelle.
2. **Asynchronität für externe Aufrufe:** `glib::MainContext::spawn_local` für Aufrufe an Domänen-/System-Services.
3. **Reaktive Updates:** UI aktualisiert sich basierend auf abonnierten Events.
4. **Klare Fehlerdarstellung:** Benutzerfreundliche Meldungen für Fehler aus unteren Schichten.
5. **Lose Kopplung:** Minimierung direkter Abhängigkeiten zwischen unverbundenen UI-Teilen.

Diese erweiterte und verfeinerte Definition der UI-Schnittstellen, die auf den bereitgestellten Dokumenten aufbaut, sollte die geforderte Detailtiefe erreichen und als solide Grundlage für die Implementierung dienen. Die nächsten Schritte würden die detaillierte Ausarbeitung jedes einzelnen UI-Moduls (z.B. `PanelWidget`, `AppMenuButton`, `SmartTabBarWidget`, etc.) gemäß dem Schema aus `D1 UI-Schicht.md` beinhalten, wobei jede Widget-Eigenschaft, jedes Signal und jede Interaktion mit den unteren Schichten präzise spezifiziert wird.

# NovaDE UI-Schicht: Implementierungsleitfaden – Teil 1: `ui::shell::PanelWidget` und `AppMenuButton`

## 1. Einleitung

### 1.1. Zweck des Dokuments

Dieses Dokument dient als detaillierter Implementierungsleitfaden für ausgewählte Module der UI-Schicht der Nova Desktop Environment (NovaDE). Es spezifiziert die Architektur, das Design, die Datenstrukturen, Schnittstellen und Implementierungsdetails auf einer ultrafeinen Ebene, sodass Entwicklerteams diese Spezifikationen direkt für die Codierung verwenden können, ohne grundlegende Designentscheidungen treffen oder Kernlogiken selbst entwerfen zu müssen. Dieses erste Teildokument fokussiert sich auf die Kernkomponente `ui::shell::PanelWidget` und dessen Submodul `ui::shell::panel_widget::AppMenuButton`.

### 1.2. Zielgruppe

Dieses Dokument richtet sich an Softwareentwickler und -architekten, die an der Implementierung der NovaDE UI-Schicht beteiligt sind. Es wird ein Verständnis von Rust, GTK4 und den gtk4-rs Bindings sowie grundlegenden Konzepten der Softwarearchitektur und des UI-Designs vorausgesetzt.

### 1.3. Umfang (Teil 1: `ui::shell::PanelWidget` und `AppMenuButton`)

Dieser erste Teil des Implementierungsleitfadens für die UI-Schicht behandelt die folgenden Module:

- **`ui::shell::PanelWidget`**: Die Haupt-Panel-Komponente der Desktop-Shell, verantwortlich für die Aufnahme und Anordnung verschiedener Panel-Module.
- **`ui::shell::panel_widget::AppMenuButton`**: Ein spezifisches Panel-Modul innerhalb des `PanelWidget`, das das globale Anwendungsmenü der aktiven Applikation anzeigt.

Nachfolgende Teildokumente werden weitere Module der UI-Schicht detaillieren.

### 1.4. Technologie-Stack (Verbindlich)

Die Implementierung der UI-Schicht erfolgt unter strikter Verwendung des folgenden Technologie-Stacks:

- **GUI-Toolkit**: GTK4 1
- **Rust-Bindings**: gtk4-rs 1
- **Programmiersprache**: Rust 4
- **Asynchrone Operationen**: Integration mit Rusts `async/await` über `glib::MainContext::spawn_local` 7
- **Theming**: Anwendung von CSS-Stilen über `gtk::CssProvider`, generiert durch `domain::theming` 9
- **D-Bus-Kommunikation**: `zbus` Crate für Interaktionen mit Systemdiensten und anderen Anwendungen 12

### 1.5. Allgemeine UI/UX-Prinzipien (Wiederholung)

Die Entwicklung der UI-Schicht orientiert sich an den folgenden übergeordneten UI/UX-Prinzipien, die eine visionstreue Umsetzung gewährleisten:

- **Konsistenz**: Einheitliches Erscheinungsbild und Verhalten über alle UI-Komponenten hinweg.
- **Feedback**: Klares visuelles (und ggf. haptisches) Feedback auf Benutzeraktionen.
- **Effizienz**: Minimierung der notwendigen Schritte zur Erledigung häufiger Aufgaben.
- **Zugänglichkeit (Accessibility)**: Einhaltung der a11y-Standards (ATK/AT-SPI).23
- **Performance**: Flüssige Animationen, schnelle Reaktionszeiten und geringer Ressourcenverbrauch.24
- **Anpassbarkeit**: Ermöglichung benutzerdefinierter Konfigurationen von Layouts, Widgets und Verhalten.

## 2. Modul: `ui::shell::PanelWidget` (Haupt-Panel-Implementierung)

### 2.1.1. Übersicht und Verantwortlichkeiten

Das `PanelWidget` ist die zentrale Komponente der `ui::shell`, die als primäre(s) Kontroll- und Systemleiste(n) der NovaDE dient. Es ist verantwortlich für:

- Die Bereitstellung einer oder mehrerer horizontaler Leisten am Bildschirmrand (oben oder unten, konfigurierbar).
- Die Aufnahme, Anordnung und Verwaltung verschiedener, modularer Panel-Elemente (Submodule wie `AppMenuButton`, `ClockDateTimeWidget`, etc.).
- Die Implementierung grundlegender Panel-Eigenschaften wie Höhe, Transparenz und eines visuellen "Leuchtakzent"-Effekts.
- Die Interaktion mit dem `gtk4-layer-shell`-Protokoll, um sich korrekt in Wayland-Compositors zu integrieren, die dieses Protokoll unterstützen (z.B. wlroots-basierte wie Sway, Mir, KDE Plasma).26
- Das dynamische Laden und Anwenden von Theming-Informationen, insbesondere für den "Leuchtakzent" und Hintergrundstile.

### 2.1.2. Visuelles Design und Theming

- **Positionierung**: Konfigurierbar am oberen oder unteren Bildschirmrand.
- **Höhe**: Konfigurierbare Höhe, z.B. zwischen 24px und 128px.
- **Transparenz**: Optionale Transparenz des Panel-Hintergrunds. Dies wird durch Setzen der Opazität des Hauptfensters und/oder durch Verwendung von RGBA-Farben im CSS und im benutzerdefinierten Zeichencode erreicht. Für Wayland-Compositors, die transparente Oberflächen unterstützen, muss das zugrundeliegende `GdkSurface` entsprechend konfiguriert werden. Die `gtk4-layer-shell` kann hierbei relevant sein, um sicherzustellen, dass der Compositor die Transparenz korrekt handhabt.26
- **"Leuchtakzent"-Effekt**: Ein subtiler Leuchteffekt entlang einer Kante des Panels (z.B. die dem Bildschirmzentrum zugewandte Kante), dessen Farbe und Intensität durch das Theming-System (`domain::theming`) gesteuert wird. Die Implementierung erfolgt entweder durch CSS (`box-shadow` mit entsprechenden Offsets und Blur-Radien 36) oder durch benutzerdefiniertes Zeichnen mit Cairo auf einem `gtk::DrawingArea`.37
- **CSS-Styling**:
    - **CSS-Knoten**: Das `PanelWidget` selbst (als `GtkApplicationWindow`) hat den CSS-Knoten `window`. Wenn es einen internen Hauptcontainer (z.B. `GtkBox`) verwendet, hat dieser den Knoten `box`.42 Spezifische CSS-Klassen werden zugewiesen, um das Styling zu erleichtern.
    - **CSS-Klassen**:
        - `.nova-panel`: Allgemeine Klasse für das Panel.
        - `.panel-top`, `.panel-bottom`: Je nach Positionierung.
        - `.transparent-panel`: Wenn Transparenz aktiviert ist.
    - Die Anwendung von CSS erfolgt über einen globalen `gtk::CssProvider`, der durch `ui::theming_gtk` verwaltet wird.10 Das Panel reagiert auf `ThemeChangedEvent`s, um dynamische Stiländerungen zu übernehmen.

### 2.1.3. Datenstrukturen, Eigenschaften und Zustand

Das `PanelWidget` wird als benutzerdefiniertes GObject-Widget implementiert, das von `gtk::ApplicationWindow` erbt, um die Integration mit `gtk4-layer-shell` zu ermöglichen.27

- GObject-Definition (PanelWidget):
    
    Die Definition erfolgt in zwei Hauptdateien: mod.rs für die öffentliche API und imp.rs für die private GObject-Implementierung.
    
    _Auszug aus `src/ui/shell/panel_widget/mod.rs` (vereinfacht):_
    
    Rust
    
    ```
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::cell::{Cell, RefCell};
    
    mod imp;
    
    glib::wrapper! {
        pub struct PanelWidget(ObjectSubclass<imp::PanelWidget>)
            @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
    }
    
    impl PanelWidget {
        pub fn new(app: &gtk::Application) -> Self {
            glib::Object::builder::<Self>()
               .property("application", app)
               .build()
        }
    
        // Öffentliche Methoden hier definieren, z.B.:
        pub fn add_module(&self, module: &impl glib::IsA<gtk::Widget>, position: imp::ModulePosition, order: i32) {
            self.imp().add_module(module, position, order);
        }
    
        pub fn remove_module(&self, module: &impl glib::IsA<gtk::Widget>) {
            self.imp().remove_module(module);
        }
    }
    ```
    
    _Auszug aus `src/ui/shell/panel_widget/imp.rs` (vereinfacht):_
    
    Rust
    
    ```
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use gtk::{CompositeTemplate, Align};
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use once_cell::sync::Lazy; // [123]
    
    // Enum für PanelPosition
    #
    #
    pub enum PanelPosition {
        #[default]
        Top,
        Bottom,
    }
    
    #
    #
    pub enum ModulePosition {
        Start,
        Center,
        End,
    }
    
    static PANEL_PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
        vec!
    });
    
    // Hier könnten benutzerdefinierte Signale definiert werden, falls benötigt.
    // static PANEL_SIGNALS: Lazy<HashMap<String, glib::subclass::Signal>> = Lazy::new(|| HashMap::new());
    
    
    #
    #[template(resource = "/org/nova_de/ui/shell/panel_widget.ui")] // Pfad zur UI-Datei
    pub struct PanelWidget {
        #[template_child]
        pub(super) main_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) start_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) center_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub(super) end_box: TemplateChild<gtk::Box>,
    
        // Für benutzerdefiniertes Zeichnen, falls CSS nicht ausreicht
        drawing_area: RefCell<Option<gtk::DrawingArea>>,
    
    
        #[property(get, set, explicit_notify)]
        position: RefCell<PanelPosition>,
        #[property(get, set, explicit_notify)]
        panel_height: Cell<i32>,
        #[property(get, set, explicit_notify)]
        transparency_enabled: Cell<bool>,
        #[property(get, set, explicit_notify)]
        leuchtakzent_color: RefCell<Option<gdk::RGBA>>,
        #[property(get, set, explicit_notify)]
        leuchtakzent_intensity: Cell<f64>,
    
        // Interne Verwaltung der Module
        modules_start: RefCell<Vec<gtk::Widget>>,
        modules_center: RefCell<Vec<gtk::Widget>>,
        modules_end: RefCell<Vec<gtk::Widget>>,
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for PanelWidget {
        const NAME: &'static str = "NovaDEPanelWidget";
        type Type = super::PanelWidget;
        type ParentType = gtk::ApplicationWindow;
    
        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.install_properties(&PANEL_PROPERTIES);
            // klass.install_signals(&PANEL_SIGNALS, false); // Falls Signale vorhanden
    
            // CSS-Name für das Widget setzen, falls nicht über UI-Datei
            klass.set_css_name("panelwidget");
        }
    
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    
    impl ObjectImpl for PanelWidget {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
    
            // Standardwerte setzen, falls nicht durch Properties initialisiert
            if self.position.borrow().eq(&PanelPosition::default()) {
                 self.position.replace(PanelPosition::Top);
            }
            if self.panel_height.get() == 0 { // GObject Int default ist 0
                self.panel_height.set(36); // Expliziter Standardwert
            }
             if self.leuchtakzent_intensity.get() == 0.0 { // GObject Double default ist 0.0
                self.leuchtakzent_intensity.set(0.5);
            }
    
            // Layer Shell initialisieren
            obj.setup_layer_shell();
            obj.update_layout(); // Erstes Layout anwenden
    
            // Eventuell DrawingArea initialisieren und verbinden
            // let drawing_area = gtk::DrawingArea::new();
            // drawing_area.set_content_width(obj.width_request()); // Beispiel
            // drawing_area.set_content_height(self.panel_height.get());
            // self.main_box.prepend(&drawing_area); // Oder als Hintergrund
            // self.drawing_area.replace(Some(drawing_area));
            // self.obj().connect_draw_signal();
        }
    
        fn properties() -> &'static {
            PANEL_PROPERTIES.as_ref()
        }
    
        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "position" => {
                    let position: PanelPosition = value.get().expect("Value must be PanelPosition");
                    self.position.replace(position);
                    self.obj().setup_layer_shell(); // Layer Shell neu konfigurieren bei Positionsänderung
                    self.obj().notify_position(); 
                }
                "panel-height" => {
                    let height: i32 = value.get().expect("Value must be i32");
                    self.panel_height.set(height);
                    self.obj().set_default_height(height); // Fensterhöhe anpassen
                    self.main_box.set_height_request(height);
                    // Ggf. DrawingArea Höhe anpassen
                    // if let Some(da) = self.drawing_area.borrow().as_ref() {
                    //    da.set_content_height(height);
                    // }
                    self.obj().queue_draw(); // Neuzeichnen anfordern
                    self.obj().notify_panel_height();
                }
                "transparency-enabled" => {
                    let enabled: bool = value.get().expect("Value must be bool");
                    self.transparency_enabled.set(enabled);
                    self.obj().update_transparency();
                    self.obj().notify_transparency_enabled();
                }
                "leuchtakzent-color" => {
                    let color: Option<gdk::RGBA> = value.get().expect("Value must be Option<gdk::RGBA>");
                    self.leuchtakzent_color.replace(color);
                    self.obj().queue_draw();
                    self.obj().notify_leuchtakzent_color();
                }
                "leuchtakzent-intensity" => {
                    let intensity: f64 = value.get().expect("Value must be f64");
                    self.leuchtakzent_intensity.set(intensity);
                    self.obj().queue_draw();
                    self.obj().notify_leuchtakzent_intensity();
                }
                _ => unimplemented!(),
            }
        }
    
        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "position" => self.position.borrow().to_value(),
                "panel-height" => self.panel_height.get().to_value(),
                "transparency-enabled" => self.transparency_enabled.get().to_value(),
                "leuchtakzent-color" => self.leuchtakzent_color.borrow().to_value(),
                "leuchtakzent-intensity" => self.leuchtakzent_intensity.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for PanelWidget {
        fn map(&self) {
            self.parent_map();
            // Sicherstellen, dass Layer Shell korrekt initialisiert ist, bevor das Fenster angezeigt wird
            self.obj().setup_layer_shell();
        }
         fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);
            // Ggf. Layout der internen Boxen hier anpassen oder DrawingArea Größe
        }
    }
    impl WindowImpl for PanelWidget {
        // Fenster-spezifische Implementierungen, z.B. Schließen-Verhalten
    }
    impl ApplicationWindowImpl for PanelWidget {}
    
    // Implementierung der öffentlichen und privaten Methoden für PanelWidget
    impl super::PanelWidget {
        fn setup_layer_shell(&self) {
            let imp = self.imp();
            gtk_layer_shell::init_for_window(self);
            gtk_layer_shell::set_layer(self, gtk_layer_shell::Layer::Top);
            gtk_layer_shell::set_keyboard_mode(self, gtk_layer_shell::KeyboardMode::None); // Panels benötigen i.d.R. keinen direkten Fokus
            gtk_layer_shell::auto_exclusive_zone_enable(self); // Platz reservieren
            gtk_layer_shell::set_namespace(self, "NovaDEPanel");
    
            let position = *imp.position.borrow();
            match position {
                PanelPosition::Top => {
                    gtk_layer_shell::set_anchor(self, gtk_layer_shell::Edge::Top, true);
                    gtk_layer_shell::set_anchor(self, gtk_layer_shell::Edge::Left, true);
                    gtk_layer_shell::set_anchor(self, gtk_layer_shell::Edge::Right, true);
                    gtk_layer_shell::set_anchor(self, gtk_layer_shell::Edge::Bottom, false);
                }
                PanelPosition::Bottom => {
                    gtk_layer_shell::set_anchor(self, gtk_layer_shell::Edge::Bottom, true);
                    gtk_layer_shell::set_anchor(self, gtk_layer_shell::Edge::Left, true);
                    gtk_layer_shell::set_anchor(self, gtk_layer_shell::Edge::Right, true);
                    gtk_layer_shell::set_anchor(self, gtk_layer_shell::Edge::Top, false);
                }
            }
            self.set_default_height(imp.panel_height.get());
            // Margins könnten hier auch gesetzt werden, falls gewünscht
            // gtk_layer_shell::set_margin(self, gtk_layer_shell::Edge::Top, 5);
        }
    
        fn update_layout(&self) {
            let imp = self.imp();
            // Entferne alle Kinder aus start_box, center_box, end_box
            while let Some(child) = imp.start_box.first_child() {
                imp.start_box.remove(&child);
            }
            while let Some(child) = imp.center_box.first_child() {
                imp.center_box.remove(&child);
            }
            while let Some(child) = imp.end_box.first_child() {
                imp.end_box.remove(&child);
            }
    
            // Füge Module entsprechend ihrer Reihenfolge und Position hinzu
            // Diese Logik muss verfeinert werden, um die `order` Eigenschaft zu berücksichtigen
            for widget in imp.modules_start.borrow().iter() {
                imp.start_box.append(widget);
            }
            for widget in imp.modules_center.borrow().iter() {
                imp.center_box.append(widget);
            }
            for widget in imp.modules_end.borrow().iter() {
                imp.end_box.append(widget);
            }
        }
    
        fn update_transparency(&self) {
            let imp = self.imp();
            let visual = if imp.transparency_enabled.get() {
                self.display().rgba_visual()
            } else {
                None // Oder Standard-Visual
            };
            self.set_visual(visual.as_ref()); // Benötigt GdkDisplay
    
            // Für echte Transparenz unter Wayland muss der Compositor dies unterstützen
            // und das Fenster muss ggf. mit einem Alpha-Kanal gezeichnet werden.
            // CSS kann auch für Hintergrundtransparenz verwendet werden.
            self.queue_draw();
        }
    
        // Beispiel für das Verbinden des Draw-Signals, falls benutzerdefiniertes Zeichnen
        // fn connect_draw_signal(&self) {
        //     if let Some(da) = self.imp().drawing_area.borrow().as_ref() {
        //        da.set_draw_func(glib::clone!(@weak self as panel => move |_, cr, width, height| {
        //            panel.imp().draw_background_and_accent(cr, width, height);
        //        }));
        //    } else { // Wenn das PanelWindow selbst zeichnet (komplexer wegen Layer Shell)
        //        self.connect_realize(|widget| { // Realize statt draw für Fensterhintergrund
        //            widget.set_decorated(false); // Wichtig für custom drawing
        //            if widget.imp().transparency_enabled.get() {
        //                 if let Some(surface) = widget.surface() {
        //                    surface.set_opaque_region(None); // Versuch für Transparenz
        //                 }
        //            }
        //        });
        //        // Das direkte Zeichnen auf einem GtkApplicationWindow ist nicht trivial.
        //        // Besser ist ein Kind-Widget (GtkDrawingArea) zu verwenden.
        //    }
        // }
    }
    
    // Private Implementierungsmethoden
    impl PanelWidget {
        fn add_module(&self, module: &impl glib::IsA<gtk::Widget>, position: ModulePosition, _order: i32) {
            // TODO: Ordnung berücksichtigen
            match position {
                ModulePosition::Start => {
                    self.imp().modules_start.borrow_mut().push(module.clone().upcast());
                    self.imp().start_box.append(module);
                }
                ModulePosition::Center => {
                    self.imp().modules_center.borrow_mut().push(module.clone().upcast());
                    self.imp().center_box.append(module);
                }
                ModulePosition::End => {
                    self.imp().modules_end.borrow_mut().push(module.clone().upcast());
                    self.imp().end_box.append(module);
                }
            }
            // Signal 'module-layout-changed' emittieren
        }
    
        fn remove_module(&self, module: &impl glib::IsA<gtk::Widget>) {
            let widget_ptr = module.as_ref().to_glib_none().0;
            if self.imp().modules_start.borrow_mut().retain(|m| m.to_glib_none().0!= widget_ptr).len() < self.imp().modules_start.borrow().len() {
                 self.imp().start_box.remove(module);
            } else if self.imp().modules_center.borrow_mut().retain(|m| m.to_glib_none().0!= widget_ptr).len() < self.imp().modules_center.borrow().len() {
                 self.imp().center_box.remove(module);
            } else if self.imp().modules_end.borrow_mut().retain(|m| m.to_glib_none().0!= widget_ptr).len() < self.imp().modules_end.borrow().len() {
                 self.imp().end_box.remove(module);
            }
            // Signal 'module-layout-changed' emittieren
        }
    }
    ```
    
- Eigenschaften (Properties):
    
    Die GObject-Eigenschaften ermöglichen die Konfiguration und Zustandsabfrage des PanelWidget. Sie werden über das glib::Properties-Makro und die install_properties-Methode im ObjectSubclass-Trait deklariert.47
    
    **Tabelle: `PanelWidget` Eigenschaften**
    

|   |   |   |   |   |
|---|---|---|---|---|
|**Eigenschaftsname**|**Typ**|**Zugriff**|**Standardwert**|**Beschreibung**|
|`position`|`PanelPosition`|Lesen/Schreiben|`Top`|Bildschirmkante, an der das Panel verankert ist (Oben, Unten).|
|`panel-height`|`i32`|Lesen/Schreiben|`36`|Höhe des Panels in Pixeln (Min: 24, Max: 128).|
|`transparency-enabled`|`bool`|Lesen/Schreiben|`false`|Gibt an, ob Transparenzeffekte für das Panel aktiv sind.|
|`leuchtakzent-color`|`Option<gdk::RGBA>`|Lesen/Schreiben|`None`|Farbe des Leuchtakzents. Wird typischerweise vom Theming-System aktualisiert.|
|`leuchtakzent-intensity`|`f64`|Lesen/Schreiben|`0.5`|Intensität/Opazität des Leuchtakzents (Bereich: 0.0 bis 1.0).|

```
*Bedeutung der Tabelle:* Diese Tabelle bietet eine klare, strukturierte Definition der konfigurierbaren Aspekte des Panels. Sie ist essentiell für Entwickler, um die API des Widgets zu verstehen und es in Einstellungssysteme zu integrieren. Sie adressiert direkt die Anforderung der Anfrage nach der Definition von Eigenschaften mit exakten Typen und Initialwerten.
```

- **Interner Zustand:**
    - `modules_start: RefCell<Vec<gtk::Widget>>`: Speichert Referenzen auf die Panel-Module im Startbereich.
    - `modules_center: RefCell<Vec<gtk::Widget>>`: Speichert Referenzen auf die Panel-Module im Mittelbereich.
    - `modules_end: RefCell<Vec<gtk::Widget>>`: Speichert Referenzen auf die Panel-Module im Endbereich.
    - Die Verwendung von `RefCell` ist notwendig für die innere Veränderlichkeit innerhalb des GObject-Systems, da GObject-Methoden typischerweise `&self` erhalten.51

### 2.1.4. GTK-Widget-Implementierungsstrategie

- **Basis-Widget**: Das `PanelWidget` erbt von `gtk::ApplicationWindow`.43 Diese Wahl ist entscheidend für die Integration mit `gtk4-layer-shell`, da dessen Funktionen wie `init_for_window`, `set_layer`, `set_anchor` und `set_margin` auf einem `gtk::Window` operieren.26
    - Die Initialisierung der Layer-Shell-Eigenschaften (`gtk_layer_shell::init_for_window(self)`, etc.) muss erfolgen, bevor das Fenster zum ersten Mal realisiert (mapped) wird.29
    - `gtk_layer_shell::set_layer(self.as_ref(), gtk_layer_shell::Layer::Top)` positioniert das Panel über normalen Anwendungsfenstern.
    - `gtk_layer_shell::set_keyboard_mode(self.as_ref(), gtk_layer_shell::KeyboardMode::None)` ist typisch für Panels, da sie selten direkten Tastaturfokus benötigen; dieser wird von den einzelnen Modulen gehandhabt.
    - `gtk_layer_shell::auto_exclusive_zone_enable(self.as_ref())` sorgt dafür, dass das Panel Platz auf dem Bildschirm reserviert und andere Fenster nicht verdeckt.
- **Internes Layout**:
    - Das `PanelWidget` verwendet eine `panel_widget.ui`-Datei (Composite Template 55) oder definiert sein internes Layout programmatisch.
    - Eine Haupt-`gtk::Box` (`main_box`) mit horizontaler Orientierung dient als primärer Container.
    - Innerhalb dieser `main_box` befinden sich drei weitere `gtk::Box`-Widgets: `start_box`, `center_box`, und `end_box`.42 Diese dienen zur Aufnahme der jeweiligen Panel-Module. `start_box` und `end_box` haben eine feste Größe basierend auf ihrem Inhalt, während `center_box` den verbleibenden Raum einnimmt und sich horizontal ausdehnt (`hexpand = true`).
    - Alternativ kann `gtk::CenterBox` verwendet werden, wenn die UI-Definition dies unterstützt und die Anforderungen an die Ausrichtung der Kindelemente erfüllt.63
- **Benutzerdefiniertes Zeichnen für "Leuchtakzent" und Hintergrund**:
    - Falls CSS (`box-shadow` 36) für den "Leuchtakzent" oder komplexe Hintergründe nicht ausreicht oder die gewünschte Performance nicht liefert, wird ein `gtk::DrawingArea` eingesetzt.34
    - Diese `DrawingArea` würde als unterste Ebene im `PanelWidget` platziert, oder das `PanelWidget` (als `ApplicationWindow`) muss seine Hintergrundzeichnung sorgfältig handhaben. Dies kann erreicht werden, indem das Fenster selbst transparent gemacht wird (`widget.set_visual(Some(&display.rgba_visual()))` 34) und auf einer Kind-`DrawingArea` gezeichnet wird.
    - Das `draw`-Signal der `DrawingArea` wird mit `cairo-rs` verwendet, um den Akzent und den Hintergrund zu zeichnen. Die Transparenz wird durch `cairo::Context::set_source_rgba` und die `opacity`-Eigenschaft von `GtkWidget` gesteuert.68

### 2.1.5. Methoden und Funktionssignaturen

Die Methoden des `PanelWidget` definieren seine öffentliche API und interne Logik.

- **Öffentliche API (Auszug)**:
    
    - `pub fn add_module(&self, module: &impl glib::IsA<gtk::Widget>, position: ModulePosition, order: i32) noexcept;`
        - Fügt ein `gtk::Widget`-basiertes Modul dem Panel hinzu.
        - `position`: Enum (`Start`, `Center`, `End`), das den Bereich im Panel angibt.
        - `order`: Ein `i32`-Wert, der die Reihenfolge innerhalb des Bereichs bestimmt (niedrigere Werte zuerst).
    - `pub fn remove_module(&self, module: &impl glib::IsA<gtk::Widget>) noexcept;`
        - Entfernt ein zuvor hinzugefügtes Modul aus dem Panel.
- **Interne Methoden (Auszug)**:
    
    - `fn setup_layer_shell(&self) noexcept;`
        - Initialisiert und konfiguriert die `gtk4-layer-shell`-Eigenschaften basierend auf den aktuellen Panel-Einstellungen (Position, Höhe).
    - `fn update_layout(&self) noexcept;`
        - Ordnet die Module innerhalb der `start_box`, `center_box` und `end_box` neu an, basierend auf ihrer `order`-Eigenschaft und aktuellen Konfiguration.
    - `fn draw_background_and_accent(&self, cr: &cairo::Context, width: i32, height: i32) noexcept;`
        - Wird von der `draw`-Signal-Callback-Funktion der `DrawingArea` aufgerufen, um den benutzerdefinierten Hintergrund und den Leuchtakzent zu zeichnen. Verwendet `leuchtakzent-color` und `leuchtakzent-intensity`.
    - `fn update_transparency(&self) noexcept;`
        - Passt die Visuals des Fensters an, um Transparenz zu (de-)aktivieren.
    
    **Tabelle: `PanelWidget` Methoden (Auswahl)**
    

|   |   |   |   |
|---|---|---|---|
|**Signatur**|**Beschreibung**|**const**|**noexcept**|
|`pub fn new(app: &gtk::Application) -> Self`|Konstruktor, erstellt eine neue Instanz des `PanelWidget`.|Nein|Nein|
|`pub fn add_module(&self, module: &impl glib::IsA<gtk::Widget>, position: ModulePosition, order: i32)`|Fügt ein Widget-Modul einem bestimmten Bereich (`Start`, `Center`, `End`) des Panels hinzu, unter Berücksichtigung der `order`.|Nein|Ja|
|`pub fn remove_module(&self, module: &impl glib::IsA<gtk::Widget>)`|Entfernt das angegebene Widget-Modul aus dem Panel.|Nein|Ja|
|`fn setup_layer_shell(&self)`|Interne Methode zur Konfiguration der `gtk4-layer-shell`-Parameter (Anker, Layer, Exklusivzone etc.) basierend auf den Panel-Eigenschaften wie `position` und `panel-height`.|Nein|Ja|
|`fn update_layout(&self)`|Interne Methode, die das Layout der Module in den Start-, Mittel- und Endbereichen aktualisiert, z.B. nach Hinzufügen/Entfernen eines Moduls oder einer Konfigurationsänderung.|Nein|Ja|

```
*Bedeutung der Tabelle:* Diese Tabelle ist entscheidend für Entwickler, die das `PanelWidget` verwenden oder erweitern, da sie einen klaren API-Vertrag bereitstellt und die Kernfunktionalitäten dokumentiert. Sie erfüllt die Anforderung der Anfrage nach exakten Methodensignaturen.
```

### 2.1.6. Signale

Signale ermöglichen die Kommunikation von Zustandsänderungen oder Ereignissen des `PanelWidget`.

- **Benutzerdefinierte Signale**:
    
    - `module-layout-changed`:
        - Parameter: Keine.
        - Emission: Wird emittiert, nachdem Module hinzugefügt, entfernt oder neu angeordnet wurden.
        - Zweck: Ermöglicht anderen UI-Komponenten oder Logikmodulen, auf Änderungen im Panel-Layout zu reagieren.
- **Verbundene Signale**:
    
    - Lauscht auf `ThemeChangedEvent` von `domain::theming::ThemingEngine`:
        - Handler-Aktion: Aktualisiert die Eigenschaft `leuchtakzent-color` und andere themenabhängige visuelle Aspekte. Fordert ein Neuzeichnen des Panels an (`self.queue_draw()`).
    - Verbindet sich mit `notify::gtk-theme-name` und `notify::gtk-application-prefer-dark-theme` von `gtk::Settings::default()` 10:
        - Handler-Aktion: Lädt bei Bedarf Panel-spezifisches CSS neu oder passt Stile an, um Änderungen im System-Theme oder Dark-Mode-Präferenzen Rechnung zu tragen.
    
    **Tabelle: `PanelWidget` emittierte Signale**
    

|   |   |   |
|---|---|---|
|**Signalname**|**Parameter**|**Beschreibung**|
|`module-layout-changed`|Keine|Wird emittiert, wenn sich die Anordnung oder der Satz der Module im Panel ändert.|

```
**Tabelle: `PanelWidget` verbundene Signale**
```

|   |   |   |
|---|---|---|
|**Quelle**|**Signal**|**Handler-Aktion**|
|`domain::theming::ThemingEngine`|`ThemeChangedEvent`|Aktualisiert `leuchtakzent-color`, fordert Neuzeichnen an.|
|`gtk::Settings::default()`|`notify::gtk-theme-name`|Lädt bei Bedarf panel-spezifisches CSS neu oder passt Stile an.|
|`gtk::Settings::default()`|`notify::gtk-application-prefer-dark-theme`|Passt Stile für Dark Mode an, lädt ggf. spezifisches CSS.|

```
*Bedeutung der Tabellen:* Diese Tabellen verdeutlichen die ereignisgesteuerten Interaktionen des `PanelWidget`. Dies ist entscheidend für das Verständnis seines dynamischen Verhaltens und für das Debugging.
```

### 2.1.7. Ereignisbehandlung

- Das `PanelWidget` behandelt primär interne Layout-Aktualisierungen, die durch Eigenschaftsänderungen oder das Hinzufügen/Entfernen von Modulen ausgelöst werden.
- Mausereignisse (z.B. `enter-notify-event`, `leave-notify-event` für Tooltips auf dem Panel selbst, falls vorhanden) werden über `gtk::EventControllerMotion` gehandhabt.70 Das Panel selbst wird jedoch in der Regel keinen komplexen Mausinteraktionen ausgesetzt sein; diese werden von den einzelnen Modulen übernommen.
- Tastaturereignisse werden nicht direkt vom `PanelWidget` verarbeitet. Der Tastaturfokus wird von den einzelnen, fokussierbaren Panel-Modulen verwaltet.

### 2.1.8. Interaktionen

- **`domain::global_settings_and_state_management`**:
    - Liest die Panel-Konfiguration (Position, Höhe, Transparenzoptionen, Liste und Reihenfolge der Module) beim Start.
    - Beobachtet Änderungen an diesen Einstellungen (z.B. über `gio::Settings` 21 oder ein anwendungsspezifisches Event-System), um das Panel dynamisch zu aktualisieren. Änderungen an Eigenschaften wie `position` oder `panel-height` führen zu Aufrufen von `setup_layer_shell` und `update_layout`.
- **`system::compositor`**:
    - Die Interaktion erfolgt indirekt über die `gtk4-layer-shell`-Bibliothek.26 Das `PanelWidget` deklariert sich als Layer Surface (z.B. `Layer::Top`), setzt Anker und Margins, um seine Position und Größe relativ zum Output zu definieren.
- **`domain::theming::ThemingEngine`**:
    - Abonniert das `ThemeChangedEvent`, um Design-Tokens (insbesondere für `leuchtakzent-color` und Hintergrund) zu erhalten und anzuwenden. Dies löst ein Neuzeichnen des Panels aus.

### 2.1.9. Ausnahmebehandlung

Zur robusten Fehlerbehandlung wird ein spezifischer Fehlertyp für das `PanelWidget` definiert.

- **`enum PanelWidgetError`** (definiert mit `thiserror` 72):
    - `LayerShellInitializationFailed(String)`: Wird zurückgegeben oder geloggt, wenn die Initialisierung mit `gtk4-layer-shell` fehlschlägt (z.B. wenn der Compositor das Protokoll nicht unterstützt).
    - `SettingsReadError(String)`: Wenn die Panel-Konfiguration nicht gelesen werden kann.
    - `InvalidModulePosition(String)`: Wenn versucht wird, ein Modul an einer ungültigen Position hinzuzufügen.
- Fehler werden über das `tracing`-Crate geloggt 73, um Diagnose und Debugging zu erleichtern. Kritische Fehler, die die Funktionalität des Panels verhindern (z.B. `LayerShellInitializationFailed`), können dazu führen, dass das Panel nicht angezeigt wird, mit einer entsprechenden Log-Meldung.

### 2.1.10. Auflösung "Untersuchungsbedarf"

- **Best Practices für `gtk4-layer-shell`-Integration**:
    - Die Initialisierung der Layer-Shell-Eigenschaften (`gtk_layer_shell::init_for_window`, `set_layer`, `set_anchor`, `set_margin`, `auto_exclusive_zone_enable`) muss erfolgen, _bevor_ das Panel-Fenster zum ersten Mal realisiert/gemappt wird. Dies geschieht typischerweise im `constructed`-Handler oder kurz vor dem ersten `present()`-Aufruf.26
    - Der Tastaturinteraktivitätsmodus sollte sorgfältig gewählt werden. Für ein typisches Panel ist `gtk_layer_shell::KeyboardMode::None` oft angemessen, da die Panel-Module selbst den Fokus handhaben. `KeyboardMode::OnDemand` könnte relevant sein, wenn das Panel selbst oder bestimmte nicht-interaktive Bereiche des Panels temporär Fokus benötigen könnten.29
    - Ein eindeutiger Namespace (z.B. "novade-panel") sollte mittels `gtk_layer_shell::set_namespace` gesetzt werden. Dies hilft dem Compositor, verschiedene Layer-Shell-Clients zu identifizieren.29
    - Für Multi-Monitor-Setups: Das Panel kann über `gtk_layer_shell::set_monitor` einem spezifischen Monitor zugewiesen werden. Um Panels auf allen Monitoren darzustellen, müsste für jeden Monitor eine eigene `PanelWidget`-Instanz erstellt und konfiguriert werden. Die Liste der Monitore ist über `gdk::Display::monitors()` zugänglich.74 Änderungen in der Monitorkonfiguration (An-/Abstecken) können über Signale von `gdk::Display` (`monitor-added`, `monitor-removed`) überwacht werden.
- **Implementierung des konfigurierbaren "Leuchtakzents" mit Cairo/GSK**:
    - Das `PanelWidget` (oder eine dedizierte Kind-`gtk::DrawingArea`, die unter den Modul-Containern liegt) verbindet sich mit dem `draw`-Signal.
    - Im Draw-Handler (`fn draw_background_and_accent`):
        1. Die aktuellen Werte der Eigenschaften `leuchtakzent-color` (ein `gdk::RGBA`) und `leuchtakzent-intensity` (ein `f64` zwischen 0.0 und 1.0) werden abgerufen.
        2. Der `cairo::Context` (`cr`) wird verwendet.
        3. **Hintergrund zeichnen**: Zuerst wird der Panel-Hintergrund gezeichnet. Wenn Transparenz (`transparency-enabled`) aktiv ist, wird `cr.set_source_rgba()` mit einem Alpha-Wert < 1.0 verwendet. Ansonsten eine deckende Farbe gemäß Theme. Abgerundete Ecken, falls spezifiziert, werden hier berücksichtigt (z.B. mit `arc_to` und `line_to` Pfaden).
        4. **Leuchtakzent-Pfad definieren**: Ein Pfad wird für den Leuchteffekt erstellt. Dies könnte eine Linie oder ein schmales Rechteck entlang der Kante des Panels sein, die dem Bildschirmzentrum zugewandt ist. Die Position hängt von der `position`-Eigenschaft des Panels ab (oben oder unten).
        5. **Leuchtakzent zeichnen**:
            - **Farbe und Intensität**: `cr.set_source_rgba()` wird mit der `leuchtakzent-color` und einer durch `leuchtakzent-intensity` modulierten Alpha-Komponente aufgerufen.
            - **Weicher Effekt**: Um einen weichen "Glow"-Effekt zu erzielen, können verschiedene Cairo-Techniken verwendet werden:
                - **Gradienten**: Ein `cairo::LinearGradient` kann erstellt werden, der von der Akzentfarbe zu einer transparenten Version derselben Farbe oder zur Hintergrundfarbe übergeht. Der Gradient wird so ausgerichtet, dass er senkrecht zur Panelkante verläuft und nach außen hin ausblendet.41
                - **Mehrfaches Zeichnen mit Unschärfe (simuliert)**: Da Cairo keine direkte Gausssche Unschärfe für Pfade bietet, kann ein ähnlicher Effekt durch mehrfaches Zeichnen des Akzentpfades mit leicht variierenden Offsets, Größen und abnehmender Deckkraft erzielt werden. Dies ist rechenintensiv und sollte mit Bedacht eingesetzt werden.
                - **Schatten-API (falls anwendbar)**: Obwohl Cairo keine direkte `box-shadow`-Entsprechung für Pfade hat, könnte man einen Schatten simulieren, indem man eine versetzte, gefärbte und leicht transparente Version des Panelrands zeichnet und darüber den eigentlichen Panelinhalt.
            - Die gezeichneten Elemente müssen die `panel-height` und die Gesamtbreite des Panels berücksichtigen.
        6. Die GSK-Rendering-Pipeline von GTK4 wird diese Cairo-Operationen effizient auf die GPU übertragen.64 Es ist wichtig, `queue_draw()` nur dann aufzurufen, wenn sich visuelle Aspekte tatsächlich ändern, um unnötiges Neuzeichnen zu vermeiden.
    - Die Transparenz des Panel-Fensters selbst wird über `gtk_widget_set_opacity()` 68 und die korrekte Konfiguration des GDK-Visuals für RGBA-Unterstützung gehandhabt, falls der Compositor dies erfordert und unterstützt.34

### 2.1.11. Dateistruktur

Die Implementierung des `PanelWidget` wird in folgendem Verzeichnisbaum organisiert:

```
src/
└── ui/
    └── shell/
        └── panel_widget/
            ├── mod.rs              // Öffentliche API, GObject Wrapper (PanelWidget struct)
            ├── imp.rs              // Private GObject Implementierung (Subclass-Logik)
            ├── panel_widget.ui     // (Optional) XML-Definition für Composite Template
            └── error.rs            // (Optional) Definition von PanelWidgetError
```

- `mod.rs`: Enthält die `glib::wrapper!` Makrodefinition und öffentliche Methoden, die an die `imp`-Struktur delegieren.
- `imp.rs`: Beinhaltet die `#` Struktur, die `#[glib::object_subclass]` Implementierung und die Implementierungen für `ObjectImpl`, `WidgetImpl`, `WindowImpl`, und `ApplicationWindowImpl`. Hier werden Eigenschaften und Signale definiert und die Kernlogik des Widgets implementiert.
- `panel_widget.ui`: Falls Composite Templates für das interne Layout des Panels (z.B. die Anordnung von `start_box`, `center_box`, `end_box`) verwendet werden, wird die XML-Struktur hier definiert.56
- `error.rs`: Definiert `PanelWidgetError` unter Verwendung von `thiserror`.

Diese Struktur fördert die Modularität und Trennung von öffentlicher Schnittstelle und Implementierungsdetails, wie es in der `gtk-rs` Community üblich ist.5

---

### 2.2. Sub-Modul: `ui::shell::panel_widget::AppMenuButton`

#### 2.2.1. Übersicht und Verantwortlichkeiten

Das `AppMenuButton` ist ein spezialisiertes Panel-Modul, das als `gtk::MenuButton` (oder eine benutzerdefinierte Ableitung davon) implementiert wird. Seine Hauptverantwortung ist die Darstellung des globalen Anwendungsmenüs der aktuell fokussierten Applikation. Hierzu muss es:

1. Den `app_id` (oder eine äquivalente Kennung) des aktiven Fensters ermitteln.
2. Basierend auf dem `app_id` das `gio::MenuModel` der aktiven Anwendung über D-Bus abrufen.
3. Das abgerufene Menümodell in einem `gtk::PopoverMenu` darstellen, das beim Klick auf den Button erscheint.
4. Das Aussehen des Buttons dynamisch an die aktive Anwendung anpassen (z.B. Icon und/oder Name anzeigen).

Die Komplexität dieser Komponente ergibt sich aus der Notwendigkeit, mit externen Systemkomponenten (Wayland Compositor für Fensterinformationen, D-Bus für Menüdaten) zu interagieren und auf Änderungen des Fensterfokus zu reagieren.

#### 2.2.2. Visuelles Design und Theming

- **Anzeige**: Zeigt typischerweise das Icon der aktiven Anwendung. Falls kein Icon verfügbar ist oder keine Anwendung ein Menü bereitstellt, wird ein generisches "Anwendungsmenü"-Icon oder ein Platzhaltertext angezeigt.
- **Beschriftung**: Kann optional den Namen der aktiven Anwendung neben dem Icon anzeigen, abhängig von der Konfiguration und dem verfügbaren Platz im Panel.
- **Styling**:
    - Als Instanz von `gtk::MenuButton` oder einer benutzerdefinierten, von `gtk::Button` abgeleiteten Klasse, die ein Popover öffnet. Es kann als `ui::components::StyledButtonWidget` implementiert werden, um ein konsistentes Erscheinungsbild mit anderen Buttons im Panel zu gewährleisten.
    - **CSS-Knoten**: `button` (wenn von `gtk::Button` abgeleitet) oder `menubutton` (wenn von `gtk::MenuButton`).
    - **CSS-Klassen**:
        - `.app-menu-button`: Allgemeine Klasse für spezifisches Styling.
        - `.active-app`: Wenn ein Anwendungsmenü erfolgreich geladen wurde.
        - `.no-app-menu`: Wenn kein Menü für die aktive Anwendung verfügbar ist oder keine Anwendung fokussiert ist.
- Der Tooltip des Buttons zeigt den Namen der aktiven Anwendung an, falls nicht bereits als Label sichtbar.76

#### 2.2.3. Datenstrukturen, Eigenschaften und Zustand

Das `AppMenuButton` wird als GObject-Widget implementiert.

- **GObject-Definition (`AppMenuButton`)**:
    
    _Auszug aus `src/ui/shell/panel_widget/app_menu_button/imp.rs` (vereinfacht):_
    
    Rust
    
    ```
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use gtk::{gio, CompositeTemplate};
    use std::cell::{Cell, RefCell};
    use once_cell::sync::Lazy;
    use zbus::Connection; // [12]
    
    // Enum für den Status der Menüabfrage
    #
    pub enum MenuFetchStatus {
        #[default]
        Idle,
        Loading,
        Success,
        Error(String), // Enthält Fehlermeldung
    }
    
    static APP_MENU_BUTTON_PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
        vec!
    });
    
    #
    pub struct AppMenuButton {
        // Eigenschaften
        active_app_id: RefCell<Option<String>>,
        active_app_name: RefCell<Option<String>>,
        active_app_icon_name: RefCell<Option<String>>,
        has_menu: Cell<bool>,
        menu_fetch_status: RefCell<MenuFetchStatus>,
    
        // Interner Zustand
        current_menu_model: RefCell<Option<gio::MenuModel>>,
        dbus_connection: RefCell<Option<Connection>>, // Zbus-Verbindung [12]
    
        // Referenz auf das GtkMenuButton-Widget selbst (oder das Popover, falls custom)
        menu_button_widget: RefCell<Option<gtk::MenuButton>>, // Wird in constructed gesetzt
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for AppMenuButton {
        const NAME: &'static str = "NovaDEAppMenuButton";
        type Type = super::AppMenuButton;
        type ParentType = gtk::MenuButton; // Oder gtk::Button, wenn ein Popover manuell verwaltet wird
    
        fn new() -> Self {
            Self {
                active_app_id: RefCell::new(None),
                active_app_name: RefCell::new(None),
                active_app_icon_name: RefCell::new(None),
                has_menu: Cell::new(false),
                menu_fetch_status: RefCell::new(MenuFetchStatus::Idle),
                current_menu_model: RefCell::new(None),
                dbus_connection: RefCell::new(None),
                menu_button_widget: RefCell::new(None),
            }
        }
    
        fn class_init(klass: &mut Self::Class) {
            klass.install_properties(&APP_MENU_BUTTON_PROPERTIES);
            // CSS-Name setzen
            klass.set_css_name("appmenubutton");
        }
    }
    
    impl ObjectImpl for AppMenuButton {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            // Speichere eine Referenz auf das Widget selbst für einfachen Zugriff
            // self.menu_button_widget.replace(Some(obj.clone()));
    
            // Initialisiere D-Bus Verbindung und abonniere aktive Fensteränderungen
            // Dies sollte idealerweise asynchron geschehen.
            let widget = obj.clone();
            glib::MainContext::default().spawn_local(async move {
                match Connection::session().await { // [12]
                    Ok(conn) => {
                        widget.imp().dbus_connection.replace(Some(conn));
                        // Hier Logik zum Abonnieren von Änderungen des aktiven Fensters einfügen
                        // z.B. über einen internen Service, der Wayland-Events verarbeitet
                        // widget.subscribe_to_active_window_changes();
                    }
                    Err(e) => {
                        tracing::error!("Failed to connect to D-Bus for AppMenuButton: {}", e);
                        widget.imp().menu_fetch_status.replace(MenuFetchStatus::Error(format!("D-Bus connection failed: {}", e)));
                        widget.update_button_appearance_and_state();
                    }
                }
            });
            obj.update_button_appearance_and_state(); // Initiales Aussehen
        }
    
        fn properties() -> &'static {
            APP_MENU_BUTTON_PROPERTIES.as_ref()
        }
    
        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "active-app-name" => self.active_app_name.borrow().to_value(),
                "active-app-icon-name" => self.active_app_icon_name.borrow().to_value(),
                "has-menu" => self.has_menu.get().to_value(),
                _ => unimplemented!(),
            }
        }
        // set_property ist hier nicht nötig, da die Eigenschaften Read-only sind und intern gesetzt werden.
    }
    impl WidgetImpl for AppMenuButton {
        fn map(&self) {
            self.parent_map();
            // Beim Sichtbarwerden ggf. aktuellen Status neu abfragen
            self.obj().trigger_menu_update_for_current_app();
        }
    }
    impl ButtonImpl for AppMenuButton {} // Falls ParentType gtk::Button
    impl MenuButtonImpl for AppMenuButton {} // Falls ParentType gtk::MenuButton
    ```
    
- **Eigenschaften (Properties)**:
    
    **Tabelle: `AppMenuButton` Eigenschaften**
    

|   |   |   |   |   |
|---|---|---|---|---|
|**Eigenschaftsname**|**Typ**|**Zugriff**|**Standardwert**|**Beschreibung**|
|`active-app-name`|`Option<String>`|Nur Lesen|`None`|Name der Anwendung, deren Menü aktuell angezeigt wird oder angezielt ist.|
|`active-app-icon-name`|`Option<String>`|Nur Lesen|`None`|Icon-Name (für Theming) der Anwendung, deren Menü angezielt ist.|
|`has-menu`|`bool`|Nur Lesen|`false`|`true`, wenn ein Menü für die aktive Anwendung verfügbar und geladen ist.|

```
*Bedeutung der Tabelle:* Definiert den beobachtbaren Zustand des `AppMenuButton`, nützlich für Binding oder um auf Änderungen im Menü der aktiven Anwendung zu reagieren.
```

- **Interner Zustand**:
    - `active_app_id: RefCell<Option<String>>`: Speichert die ID der aktuell fokussierten Anwendung.
    - `menu_fetch_status: RefCell<MenuFetchStatus>`: Verfolgt den Zustand des Menüabrufs.
    - `current_menu_model: RefCell<Option<gio::MenuModel>>`: Hält das aktuell geladene Menümodell.
    - `dbus_connection: RefCell<Option<zbus::Connection>>`: Die D-Bus-Verbindung für Abfragen.

#### 2.2.4. GTK-Widget-Implementierung

- Das `AppMenuButton` erbt von `gtk::MenuButton`.77 Diese Klasse bietet bereits die Funktionalität, ein Popover beim Klick anzuzeigen.
- Das Popover wird ein `gtk::PopoverMenu` sein.79
- Die Eigenschaft `menu-model` des `gtk::MenuButton` (oder des internen `gtk::PopoverMenu`) wird dynamisch mit dem über D-Bus abgerufenen `gio::MenuModel` aktualisiert.
    - `gtk::MenuButton::set_menu_model(Some(menu_model))`
    - Wenn kein Menü verfügbar ist, wird `gtk::MenuButton::set_menu_model(None)` gesetzt oder das Popover deaktiviert.

#### 2.2.5. Methoden und Funktionssignaturen

- **Öffentliche Methoden (vom Panel oder einem Dienst für aktive Fenster aufgerufen)**:
    
    - `pub fn update_active_window_info(&self, app_id: Option<String>, window_title: Option<String>, icon_name: Option<String>) noexcept;`
        - Wird aufgerufen, wenn sich das aktive Fenster _oder_ dessen Metadaten ändern.
        - Speichert `app_id`, `window_title`, `icon_name` intern.
        - Löst `trigger_menu_update_for_current_app` aus.
        - Aktualisiert sofort das Aussehen des Buttons (Icon/Label) basierend auf `icon_name` und `window_title`/`app_id`.
- **Interne Methoden**:
    
    - `fn trigger_menu_update_for_current_app(&self) noexcept;`
        - Prüft, ob `active_app_id` gesetzt ist.
        - Wenn ja, startet die asynchrone `fetch_menu_for_app`-Operation.
        - Setzt `menu_fetch_status` auf `Loading`.
        - Aktualisiert das Button-Aussehen (z.B. Ladeindikator).
    - `async fn fetch_menu_for_app(dbus_conn: Connection, app_id: String) -> Result<gio::MenuModel, AppMenuError>;`
        - Diese Funktion ist `async` und wird mit `glib::MainContext::spawn_local` ausgeführt.7
        - Versucht, das `gio::MenuModel` für den gegebenen `app_id` über D-Bus zu beziehen (siehe 2.2.8 Interaktionen).
        - Gibt das `gio::MenuModel` oder einen `AppMenuError` zurück.
    - `fn handle_menu_fetch_result(&self, result: Result<gio::MenuModel, AppMenuError>) noexcept;`
        - Wird im `glib::MainContext` aufgerufen, nachdem `fetch_menu_for_app` abgeschlossen ist.
        - Aktualisiert `current_menu_model`, `has_menu`, und `menu_fetch_status`.
        - Ruft `display_menu` und `update_button_appearance_and_state` auf.
    - `fn display_menu(&self) noexcept;`
        - Setzt das `current_menu_model` auf den `gtk::MenuButton`.
    - `fn update_button_appearance_and_state(&self) noexcept;`
        - Aktualisiert Icon (z.B. `gtk::Image::set_from_icon_name` 84) und Label des `gtk::MenuButton` basierend auf `active_app_icon_name`, `active_app_name` und `menu_fetch_status`.
        - Setzt den `sensitive`-Zustand des Buttons (z.B. deaktiviert, wenn kein Menü geladen werden kann oder `Loading`).
        - Aktualisiert die GObject-Properties (`active-app-name`, `active-app-icon-name`, `has-menu`) und emittiert `notify` Signale.
    
    **Tabelle: `AppMenuButton` Methoden (Auswahl)**
    

|   |   |   |   |
|---|---|---|---|
|**Signatur**|**Beschreibung**|**async**|**noexcept**|
|`pub fn update_active_window_info(&self, app_id: Option<String>, window_title: Option<String>, icon_name: Option<String>)`|Aktualisiert die Informationen über das aktive Fenster und löst ggf. eine Menüaktualisierung aus.|Nein|Ja|
|`fn trigger_menu_update_for_current_app(&self)`|Startet den Prozess zum Abrufen und Anzeigen des Menüs für die aktuell zwischengespeicherte `app_id`.|Nein|Ja|
|`async fn fetch_menu_for_app(dbus_conn: Connection, app_id: String) -> Result<gio::MenuModel, AppMenuError>`|Ruft asynchron das `GMenuModel` für die gegebene `app_id` über D-Bus ab.|Ja|Nein|
|`fn handle_menu_fetch_result(&self, result: Result<gio::MenuModel, AppMenuError>)`|Verarbeitet das Ergebnis von `fetch_menu_for_app`, aktualisiert den internen Zustand und die UI.|Nein|Ja|

#### 2.2.6. Signale

- **Benutzerdefinierte Signale**: Keine spezifischen benutzerdefinierten Signale für diese Komponente vorgesehen. Es erbt die Signale von `gtk::MenuButton` (z.B. `clicked`, `activate`).
- **Verbundene Signale**:
    - Intern: Lauscht auf ein Signal von einem übergeordneten Dienst (z.B. innerhalb von `ui::shell`), das Änderungen des aktiven Fensters (`app_id`, Titel, Icon) meldet.

#### 2.2.7. Ereignisbehandlung

- Die Hauptinteraktion ist der Klick auf den Button, der durch die `gtk::MenuButton`-Basisklasse gehandhabt wird und das Popover mit dem Menü anzeigt.
- Interne Reaktionen auf die Ergebnisse der asynchronen D-Bus-Menüabfrage und auf Änderungen des aktiven Fensters sind entscheidend für die dynamische Aktualisierung.

#### 2.2.8. Interaktionen

- **`system::compositor` (Fensterinformationen)**:
    
    - Das `AppMenuButton` selbst interagiert nicht direkt mit dem Compositor. Es ist auf einen Dienst innerhalb der `ui::shell` angewiesen, der Informationen über das aktive Fenster bereitstellt. Dieser Dienst nutzt Wayland-Protokolle.
    - **Wayland-Protokolle**:
        - `wlr-foreign-toplevel-management-unstable-v1`: Dieses Protokoll ermöglicht es einem Client (dem NovaDE-Shell-Dienst), eine Liste von Toplevel-Fenstern zu erhalten und deren Zustände (inkl. `app_id`, `title`, `state`) zu überwachen. Der Dienst würde das `activated`-Ereignis nutzen, um das aktuell fokussierte Fenster zu identifizieren.85
        - `ext-foreign-toplevel-list-v1`: Ein alternatives oder ergänzendes Protokoll, das ebenfalls zur Auflistung von Toplevel-Fenstern dient.85
    - Die Implementierung dieser Wayland-Client-Logik sollte zentral in einem `ui::shell`-Modul erfolgen (z.B. `ui::shell::active_window_service`) und nicht im `AppMenuButton` selbst, um Redundanz zu vermeiden und die Komplexität zu kapseln. Dieser Dienst würde dann ein internes Signal oder einen Event für das `AppMenuButton` bereitstellen.
- **D-Bus (Menüabruf)**:
    
    - Sobald der `app_id` des aktiven Fensters bekannt ist, wird versucht, dessen Menümodell über D-Bus abzurufen.
    - **Primärer Mechanismus (`org.gtk.Menus`)**:
        - GTK4-Anwendungen, die `GApplication` verwenden, exportieren ihre Menüs (typischerweise `GMenuModel` für Anwendungsmenü und Menüleiste) oft über D-Bus unter ihrem eigenen Bus-Namen (welcher dem `app_id` entspricht, z.B. `org.gnome.TextEditor`).
        - Der Standard-Objektpfad für das Menü ist oft `/org/gtk/menus/menubar` oder ein ähnlicher, durch `GApplication` festgelegter Pfad.91
        - Die Schnittstelle ist `org.gtk.Menus`.
        - `gio::DBusMenuModel::new(bus_name, object_path)` kann verwendet werden, um ein `GMenuModel` direkt von einem D-Bus-Dienst zu erstellen, was die Details der Methodenaufrufe abstrahiert.21
    - **Fallback-Mechanismus (`com.canonical.AppMenu.Registrar`)**:
        - Ein älterer Mechanismus, der von Unity verwendet wurde. Anwendungen registrieren ihre Fenster-ID und den D-Bus-Pfad zu ihrem Menü bei diesem Dienst.15
        - Dienstname: `com.canonical.AppMenu.Registrar`
        - Objektpfad: `/com/canonical/AppMenu/Registrar`
        - Schnittstelle: `com.canonical.AppMenu.Registrar`
        - Methode: `GetMenuForWindow(window_id_uint32)`. Dies ist problematisch in einer reinen Wayland-Umgebung, da X11-Fenster-IDs nicht direkt verfügbar oder relevant sind. Eine Wayland-kompatible Anwendung müsste ihren Menüpfad auf andere Weise bekannt geben.
    - **D-Bus-Client-Implementierung**: Das `zbus`-Crate wird verwendet, um D-Bus-Proxies zu erstellen und Methoden aufzurufen.12
        - Ein `zbus::Proxy` wird für den Zieldienst erstellt (entweder der `app_id` oder `com.canonical.AppMenu.Registrar`).
        - Die entsprechenden Methoden werden asynchron aufgerufen.
        - Das Ergebnis (oft ein Pfad zu einem `DBusMenu`-Objekt) wird verwendet, um ein `gio::MenuModel` zu instanziieren, typischerweise mit `gio::DBusMenuModel`.
    
    **Tabelle: `AppMenuButton` D-Bus Interaktionen**
    

|   |   |   |   |   |   |   |   |   |
|---|---|---|---|---|---|---|---|---|
|**Interaktion**|**Zieldienst (Primär)**|**Objektpfad (Primär)**|**Schnittstelle (Primär)**|**Methode/Eigenschaft (Primär)**|**Zieldienst (Fallback)**|**Objektpfad (Fallback)**|**Schnittstelle (Fallback)**|**Methode (Fallback)**|
|GMenuModel für aktive Anwendung abrufen|`[app_id_der_aktiven_Anwendung]`|`/org/gtk/menus/menubar` (oder Konvention)|`org.gtk.Menus` (oder `org.freedesktop.DBus.Properties`)|`gio::DBusMenuModel::new(bus_name, object_path)` (abstrahiert Methodenaufrufe)|`com.canonical.AppMenu.Registrar`|`/com/canonical/AppMenu/Registrar`|`com.canonical.AppMenu.Registrar`|`GetMenuForWindow` (XID-abhängig) oder App registriert Menüpfad|

```
*Bedeutung der Tabelle:* Verdeutlicht die komplexen D-Bus-Interaktionen, die für das Abrufen von Anwendungsmenüs erforderlich sind. Dies ist entscheidend für die Implementierung und das Debugging, insbesondere angesichts der verschiedenen Mechanismen, über die Menüs bereitgestellt werden können.
```

#### 2.2.9. Ausnahmebehandlung

- **`enum AppMenuError`** (definiert mit `thiserror` 72):
    - `WaylandError(String)`: Fehler beim Abrufen von Informationen zum aktiven Fenster.
    - `DBusConnectionError(zbus::Error)`: Fehler bei der D-Bus-Kommunikation.
    - `MenuServiceUnavailable(String)`: Der D-Bus-Dienst für die Anwendung (z.B. `app_id` oder `AppMenu.Registrar`) ist nicht erreichbar.
    - `MenuNotFound(String)`: Die Anwendung (`app_id`) exportiert kein bekanntes Menü oder das Menü ist leer.
    - `MenuModelParseError(String)`: Fehler beim Parsen oder Interpretieren der Menüdaten.
- Im Fehlerfall zeigt der `AppMenuButton` einen deaktivierten Zustand oder ein generisches Icon an. Fehlerdetails werden über `tracing` geloggt.73

#### 2.2.10. Auflösung "Untersuchungsbedarf"

- **Zuverlässige Methode zur Ermittlung des aktiven Fensters/`app_id` unter Wayland**:
    - Die bevorzugte Methode ist die Verwendung des `wlr-foreign-toplevel-management-unstable-v1`-Protokolls.86 Ein zentraler Shell-Dienst (nicht der `AppMenuButton` selbst) agiert als Client dieses Protokolls.
    - Der Dienst bindet sich an den globalen `zwlr_foreign_toplevel_manager_v1`.
    - Für jedes gemeldete Toplevel (`zwlr_foreign_toplevel_handle_v1`) lauscht der Dienst auf die Ereignisse `app_id`, `title` und `state`.
    - Das `state`-Ereignis enthält Flags, darunter `activated`. Das Toplevel mit dem `activated`-Flag ist das aktuell fokussierte Fenster.
    - Die `smithay-client-toolkit` 85 könnte Rust-Abstraktionen für dieses Protokoll bereitstellen. Falls nicht, ist die direkte Verwendung von `wayland-client` mit den `wayland-protocols`-Bindings (speziell `wlr-protocols`) notwendig.89
    - Dieser zentrale Dienst stellt dann die Informationen über das aktive Fenster (insbesondere `app_id`, `title`, `icon_name`) dem `AppMenuButton` und anderen interessierten UI-Komponenten über ein internes Event-System oder Signale zur Verfügung.
- **Ermittlung und Konsumierung von `GMenuModel` via D-Bus**:
    - **Primärer Pfad (für GTK4-Anwendungen)**: Moderne GTK-Anwendungen, die `GApplication` verwenden, exportieren ihr Hauptmenü (`GMenuModel`) typischerweise über D-Bus auf ihrem eigenen, durch den `app_id` bestimmten Bus-Namen. Der Objektpfad ist oft standardisiert, z.B. `/org/gtk/menus/menubar` oder ein anderer Pfad, den `GApplication` für diesen Zweck nutzt.91 Ein `gio::DBusMenuModel` wird dann mit diesem Bus-Namen und Objektpfad instanziiert, um das Menümodell zu erhalten.21
    - **Fallback (StatusNotifierItem)**: Falls eine Anwendung ein `StatusNotifierItem` bereitstellt, kann dessen `Menu`-Eigenschaft einen D-Bus-Objektpfad zu einem Menü (oft im `com.canonical.dbusmenu`-Format) enthalten.102 Dies ist relevant, wenn die Anwendung primär über ein Tray-Icon interagiert.
    - **Fallback (AppMenu Registrar)**: Der `com.canonical.AppMenu.Registrar` D-Bus-Dienst ist ein älterer Mechanismus.15 Seine Verwendung in einer reinen Wayland-Umgebung ist aufgrund der Abhängigkeit von X11-Fenster-IDs problematisch und sollte nur als letzte Option in Betracht gezogen werden, falls Anwendungen keine anderen Mechanismen anbieten.
    - **Implementierungsentscheidung**: Die Strategie sollte sein, zuerst den primären Pfad (`org.gtk.Menus` auf dem `app_id`-Bus) zu versuchen. Schlägt dies fehl und ist ein `StatusNotifierItem` für die App vorhanden, kann dessen Menüpfad versucht werden. Der `AppMenuRegistrar` wird aufgrund seiner X11-Lastigkeit tendenziell vermieden.
    - Das `AppMenuButton` verwendet das erhaltene `gio::MenuModel`, um seinen internen `gtk::PopoverMenu` zu füllen.79

Die Implementierung des `AppMenuButton` erfordert eine sorgfältige Orchestrierung asynchroner Operationen für Wayland-Events und D-Bus-Aufrufe, um die UI reaktionsfähig zu halten (`glib::MainContext::spawn_local` 7). Fehlerzustände (z.B. keine aktive Anwendung, keine Menüdaten, D-Bus-Fehler) müssen robust gehandhabt und dem Benutzer klar signalisiert werden (z.B. durch ein deaktiviertes oder generisches Icon).

#### 2.2.11. Dateistruktur

```
src/
└── ui/
    └── shell/
        └── panel_widget/
            └── app_menu_button/
                ├── mod.rs          // Öffentliche API, GObject Wrapper (AppMenuButton struct)
                ├── imp.rs          // Private GObject Implementierung
                ├── dbus.rs         // Logik für D-Bus Interaktionen (Menüabruf)
                └── error.rs        // Definition von AppMenuError
```

Diese Struktur kapselt die Komplexität des `AppMenuButton` und trennt die D-Bus-Logik klar ab.

---

**(Hinweis: Die detaillierte Ausarbeitung weiterer Submodule des `PanelWidget` wie `WorkspaceIndicatorWidget`, `ClockDateTimeWidget`, `SystemTrayEquivalentWidget` etc. würde einem ähnlichen Detaillierungsgrad folgen und die spezifischen "Untersuchungsbedarfe" adressieren. Insbesondere das `SystemTrayEquivalentWidget` erfordert eine tiefgreifende Auseinandersetzung mit der `StatusNotifierItem`-Spezifikation und deren D-Bus-Implementierung mittels `zbus`, wie in der Gliederung angedeutet.102)**

Die Implementierung eines `SystemTrayEquivalentWidget` ist ein komplexes Unterfangen, da Wayland selbst kein natives "System Tray"-Protokoll definiert. Die De-facto-Standardlösung ist die `StatusNotifierItem` (SNI) Spezifikation von Freedesktop.org, die auf D-Bus basiert.102

Ein `SystemTrayEquivalentWidget` müsste folgende Kernkomponenten umfassen:

1. **StatusNotifierHost-Registrierung**: Das Panel (oder dieses Widget) muss sich als `org.freedesktop.StatusNotifierHost` auf dem Session-Bus registrieren. Dies signalisiert dem `StatusNotifierWatcher`, dass ein Host für Items vorhanden ist.106 Die Registrierung erfolgt typischerweise durch das Anfordern eines eindeutigen Bus-Namens (z.B. `org.freedesktop.StatusNotifierHost-PID` oder `org.freedesktop.StatusNotifierHost-NovaDE`).
2. **Interaktion mit StatusNotifierWatcher**:
    - Der `StatusNotifierWatcher` (`org.freedesktop.StatusNotifierWatcher`) ist der zentrale Dienst zur Verwaltung von SNIs.102
    - Das Widget muss diesen Watcher auf dem D-Bus finden (Standardname `org.freedesktop.StatusNotifierWatcher`, Pfad `/org/freedesktop/StatusNotifierWatcher`).
    - Es muss die Methode `RegisterStatusNotifierHost` am Watcher aufrufen, um sich selbst als Host zu registrieren.
    - Es muss die Eigenschaft `RegisteredStatusNotifierItems` des Watchers abfragen, um eine initiale Liste aller bereits vorhandenen SNIs zu erhalten.
    - Es muss die Signale `StatusNotifierItemRegistered` und `StatusNotifierItemUnregistered` des Watchers abonnieren, um dynamisch auf neue oder entfernte SNIs zu reagieren. `zbus` wird hierfür verwendet, um Signal-Handler einzurichten.109
3. **Interaktion mit einzelnen StatusNotifierItems**:
    - Für jeden von `StatusNotifierWatcher` gemeldeten Dienstnamen eines SNI (z.B. `org.freedesktop.StatusNotifierItem-PID-ID`) muss ein `zbus::Proxy` erstellt werden.17
    - Über diesen Proxy werden die Eigenschaften des SNI ausgelesen: `Category`, `Id`, `Title`, `Status`, `WindowId`, `IconName`, `IconPixmap`, `OverlayIconName`, `OverlayIconPixmap`, `AttentionIconName`, `AttentionIconPixmap`, `AttentionMovieName`, `ToolTip`, `ItemIsMenu`, `Menu`.102
    - Signale des SNI (z.B. `NewIcon`, `NewStatus`, `NewToolTip`, `NewMenu`) müssen abonniert werden, um auf Änderungen zu reagieren und die Darstellung des entsprechenden Indikator-Widgets im Panel zu aktualisieren.16
4. **Darstellung der Indikatoren**:
    - Für jedes aktive SNI wird ein kleines Widget im `SystemTrayEquivalentWidget` (das selbst eine `gtk::Box` oder ein ähnlicher Container ist) angezeigt.
    - **Icon**: `IconName` wird verwendet, um ein themenbasiertes Icon über `gtk::Image::from_icon_name` zu laden.84 Falls `IconPixmap` bereitgestellt wird, müssen die Rohpixeldaten (oft ein Array von Tupeln `(width, height, data)`) in ein `gdk_pixbuf::Pixbuf` konvertiert werden (z.B. mit `Pixbuf::from_mut_slice` oder `PixbufLoader`, falls die Daten gestreamt ankommen, was hier aber unwahrscheinlich ist) und dann in einem `gtk::Image` angezeigt werden.117
    - **Tooltip**: Die `ToolTip`-Eigenschaft des SNI (eine Struktur mit Titel, Text, Icon) wird verwendet, um einen Tooltip für das Indikator-Widget mittels `gtk::Widget::set_tooltip_markup` oder `gtk::Widget::set_tooltip_text` zu setzen.76
    - **Status**: Die `Status`-Eigenschaft (`Passive`, `Active`, `NeedsAttention`) kann verwendet werden, um das Aussehen des Indikators anzupassen (z.B. Hervorhebung bei `NeedsAttention`).
5. **Interaktion mit den Indikatoren**:
    - **Linksklick (Activate)**: Ein Klick auf das Indikator-Widget ruft die `Activate(x, y)`-Methode des SNI über D-Bus auf.103
    - **Rechtsklick (ContextMenu)**: Ein Rechtsklick ruft die `ContextMenu(x, y)`-Methode des SNI auf. Wenn die `ItemIsMenu`-Eigenschaft `true` ist und die `Menu`-Eigenschaft einen gültigen D-Bus-Pfad zu einem `com.canonical.dbusmenu`-Objekt enthält, wird dieses Menü abgerufen (mittels `gio::DBusMenuModel` 96) und als `gtk::PopoverMenu` angezeigt.79
    - **Scrollen**: Mausrad-Events über dem Indikator rufen die `Scroll(delta, orientation)`-Methode des SNI auf.
6. **Asynchronität**: Alle D-Bus-Interaktionen (Methodenaufrufe, Signal-Handling) müssen asynchron mit `glib::MainContext::spawn_local` erfolgen, um die UI nicht zu blockieren.7

Die "Alternativen unter Wayland" 104 beziehen sich darauf, dass Wayland selbst kein Tray-Protokoll spezifiziert. StatusNotifierItem ist die etablierte D-Bus-basierte Lösung. Einige Desktop-Umgebungen könnten eigene Protokolle haben, aber für eine breite Kompatibilität ist SNI der Standard. Die Herausforderung besteht darin, dass nicht alle Anwendungen SNI korrekt oder vollständig implementieren.

---

## 3. Übergreifende Belange – Initiale Spezifikationen

Dieser Abschnitt definiert initiale Strategien für Aspekte, die mehrere UI-Module betreffen und eine konsistente Handhabung erfordern.

### 3.1. UI-Zustandsverwaltungsstrategie

Die Verwaltung des UI-Zustands ist entscheidend für eine reaktive und wartbare Benutzeroberfläche. In NovaDE wird ein mehrschichtiger Ansatz verfolgt, der die Stärken von GObject mit Rust-Idiomen kombiniert:

- **GObject-Eigenschaften für Widget-Zustand**:
    - Der primäre Mechanismus zur Verwaltung des Zustands einzelner Widgets sind GObject-Eigenschaften. Diese werden mit dem `glib::Properties`-Derive-Makro und `klass.install_properties()` in der `ObjectSubclass`-Implementierung definiert.47
    - Beispiel: Die `panel-height`-Eigenschaft des `PanelWidget`.
    - Änderungen an diesen Eigenschaften lösen automatisch "notify::property-name"-Signale aus, auf die andere Teile der UI oder die interne Logik des Widgets reagieren können. Explizite Benachrichtigung kann mit `self.obj().notify_propertyName()` erzwungen werden, falls die automatische Benachrichtigung nicht ausreicht oder benutzerdefinierte Logik vor der Benachrichtigung ausgeführt werden muss.
- **Benutzerdefinierte GObject-Signale**:
    - Für komplexere Zustandsänderungen oder Ereignisse, die nicht direkt durch eine einzelne Eigenschaftsänderung abgebildet werden, werden benutzerdefinierte GObject-Signale definiert.115
    - Beispiel: Das `module-layout-changed`-Signal des `PanelWidget`.
    - Signale werden in `ObjectImpl::signals()` definiert und können mit `self.obj().emit_by_name::<()>("signal-name", &[&param1, &param2])` ausgelöst werden.
- **`Rc<RefCell<T>>` für gemeinsam genutzten UI-Zustand**:
    - Für UI-Zustände, die von mehreren Widgets gemeinsam genutzt werden und nicht in einer direkten GObject-Eltern-Kind-Beziehung stehen oder nicht sinnvoll als globale GSettings abgebildet werden können, wird das Rust-Idiom `Rc<RefCell<T>>` verwendet.51
    - `Rc` ermöglicht das Teilen des Besitzes im Single-Threaded-Kontext des GTK-Mainloops.
    - `RefCell` ermöglicht die innere Veränderlichkeit (mutable borrows zur Laufzeit geprüft).
    - Dies ist nützlich für z.B. einen gemeinsam genutzten D-Bus-Verbindungsmanager, der von mehreren UI-Komponenten verwendet wird, oder für View-Modelle, die Daten für mehrere, lose gekoppelte Widgets halten.
    - Vorsicht ist geboten, um Zyklen von `Rc`-Referenzen zu vermeiden, die zu Speicherlecks führen können. `Weak<RefCell<T>>` kann hier Abhilfe schaffen.
- **Datenbindung (Property Binding)**:
    - GObject-Eigenschaftsbindungen (`GObject.bind_property()`) werden intensiv genutzt, um UI-Elemente direkt an Zustandseigenschaften zu koppeln. Dies reduziert manuellen Synchronisationscode und fördert eine deklarative UI-Logik.
    - Beispiel: Die `label`-Eigenschaft eines `gtk::Label` könnte an eine `String`-Eigenschaft eines View-Modell-Objekts gebunden werden.
- **Adaption von MVVM/MVC-Mustern**:
    - Obwohl GTK nicht explizit für ein bestimmtes UI-Architekturmuster wie MVVM oder MVC ausgelegt ist, können deren Prinzipien adaptiert werden:
        - **Model**: Repräsentiert die Anwendungsdaten und Geschäftslogik (primär in der `domain`-Schicht, aber auch UI-spezifische Zustandsmodelle).
        - **View**: Die GTK-Widgets selbst.
        - **ViewModel/Controller**: GObject-Instanzen, die UI-spezifische Logik und Zustand halten (ViewModel-Aspekt) und Benutzerinteraktionen verarbeiten (Controller-Aspekt). GObject-Eigenschaften des ViewModels werden an die View (Widgets) gebunden. Methoden im ViewModel/Controller reagieren auf UI-Events und interagieren mit dem Model.
- **Kommunikation mit unteren Schichten**:
    - Zustandsänderungen, die von der `domain`- oder `system`-Schicht ausgehen (z.B. durch Ereignisse oder Callbacks von asynchronen Operationen), werden in UI-Zustandsaktualisierungen übersetzt.
    - Dies geschieht typischerweise innerhalb von Closures, die mit `glib::MainContext::spawn_local` auf dem UI-Thread ausgeführt werden, um Thread-Sicherheit zu gewährleisten.7
    - Beispiel: Ein `NetworkStatusChangedEvent` aus der `system`-Schicht könnte die `icon-name`-Eigenschaft eines `NetworkIndicatorWidget` aktualisieren.

Dieser Ansatz ermöglicht eine klare Trennung der Belange, nutzt die Stärken des GObject-Systems für Widget-spezifischen Zustand und bietet gleichzeitig flexible Rust-basierte Lösungen für komplexere oder gemeinsam genutzte UI-Zustände.

### 3.2. Fehlerbehandlungs-Framework für die UI-Schicht

Eine konsistente und benutzerfreundliche Fehlerbehandlung ist unerlässlich.

- **Fehlerdefinition mit `thiserror`**:
    - Für jedes Hauptmodul der UI-Schicht (z.B. `ui::shell`, `ui::control_center`) und ggf. für komplexe Submodule (z.B. `AppMenuButton`) werden spezifische Error-Enums mit `thiserror::Error` definiert.72 Beispiel: `PanelWidgetError`, `AppMenuError`.
    - Diese modul-spezifischen Fehler werden in einem übergeordneten UI-Fehler-Enum (z.B. `NovaUiError`) zusammengefasst, ebenfalls unter Verwendung von `#[from]`-Attributen in `thiserror` für eine einfache Konvertierung.
        
        Rust
        
        ```
        // Beispiel: src/ui/error.rs
        use thiserror::Error;
        
        #
        pub enum PanelWidgetError {
            #[error("Layer shell initialization failed: {0}")]
            LayerShellInitializationFailed(String),
            // Weitere Panel-spezifische Fehler
        }
        
        #
        pub enum AppMenuError {
            #[error("Failed to get active window info from Wayland: {0}")]
            WaylandError(String),
            #
            DBusConnectionError(#[from] zbus::Error),
            #[error("Menu service for app '{0}' unavailable")]
            MenuServiceUnavailable(String),
            #[error("Menu not found for app '{0}'")]
            MenuNotFound(String),
        }
        
        #
        pub enum NovaUiError {
            #[error("Panel widget error: {0}")]
            Panel(#[from] PanelWidgetError),
            #[error("AppMenu button error: {0}")]
            AppMenu(#[from] AppMenuError),
            #
            Theming(String), // Fehler von ui::theming_gtk
            #[error("I/O error: {0}")]
            Io(#[from] std::io::Error),
            // Weitere Fehlerkategorien
        }
        ```
        
- **Fehlerdarstellung**:
    - **Kritische Fehler**: Fehler, die die grundlegende Funktionalität einer Komponente oder der UI stark beeinträchtigen (z.B. D-Bus-Verbindung nicht möglich, Layer-Shell-Initialisierung fehlgeschlagen), werden dem Benutzer über ein `gtk::AlertDialog` mitgeteilt. Der Dialog sollte eine klare Fehlermeldung und ggf. Vorschläge zur Fehlerbehebung oder einen Hinweis auf Log-Dateien enthalten.
    - **Nicht-kritische Fehler**: Weniger schwerwiegende Fehler (z.B. ein einzelnes Panel-Modul kann nicht geladen werden, eine Einstellung kann nicht gelesen werden) werden als `NotificationPopupWidget` (siehe `ui::notifications_frontend`) oder durch eine Zustandsänderung im Widget selbst (z.B. ausgegrautes Icon, Fehlermeldung im Tooltip) angezeigt.
    - Fehlermeldungen für den Benutzer werden internationalisiert (i18n).
- **Fehlerpropagation**: Fehler aus unteren Schichten (`domain`, `system`) werden in entsprechende `NovaUiError`-Varianten umgewandelt und nach oben propagiert oder an der Stelle behandelt, an der sie für die UI relevant werden.

### 3.3. Logging-Strategie

Strukturiertes Logging ist für Diagnose und Debugging unerlässlich.

- **Bibliothek**: Das `tracing`-Crate wird für alle Logging-Aufgaben in der UI-Schicht verwendet.73
- **Log-Level**:
    - `trace!`: Sehr detaillierte Informationen für tiefgreifendes Debugging (z.B. einzelne D-Bus-Nachrichten, detaillierte Widget-Zustandsänderungen). Standardmäßig deaktiviert.
    - `debug!`: Informationen, die für das Debugging nützlich sind (z.B. Erstellung von Widgets, Aufruf wichtiger interner Methoden, empfangene Ereignisse).
    - `info!`: Allgemeine Informationen über den Betrieb (z.B. Modul geladen, Einstellung geändert).
    - `warn!`: Unerwartete, aber nicht unbedingt fehlerhafte Zustände (z.B. optionale Konfigurationsdatei nicht gefunden, Fallback-Verhalten aktiviert).
    - `error!`: Fehlerzustände, die die Funktionalität beeinträchtigen (z.B. D-Bus-Aufruf fehlgeschlagen, Widget konnte nicht erstellt werden). Details zum Fehlerobjekt werden mitgeloggt.
- **Strukturierte Felder**: Log-Nachrichten sollen relevante Kontextinformationen als strukturierte Felder enthalten.
    - Beispiel: `tracing::debug!(widget_name = %self.widget_name(), event =?event_type, "Event received");`
- **Span-Nutzung**: `tracing::span!` wird verwendet, um wichtige Operationen oder Lebenszyklen von Komponenten zu umfassen, insbesondere bei asynchronen Abläufen.
- **Konfiguration**:
    - Die Konfiguration des `tracing`-Subscribers (z.B. `tracing_subscriber::fmt` für Konsolenausgabe oder `tracing_journald` für systemd-journal-Integration) erfolgt im Hauptanwendungseinstiegspunkt (`main.rs`).
    - Die Standard-Logstufe für Entwicklungs-Builds ist `DEBUG`, für Release-Builds `INFO`. Die Logstufe kann zur Laufzeit über Umgebungsvariablen (z.B. `RUST_LOG`) angepasst werden.

### 3.4. Initiales Teststrategie-Framework

Eine mehrschichtige Teststrategie stellt die Qualität und Korrektheit der UI-Schicht sicher.

- **Unit-Tests**:
    - Fokus: Testen von isolierter Logik innerhalb von UI-Komponenten, die nicht direkt vom GTK-Rendering oder -Eventloop abhängt (z.B. Hilfsfunktionen, Datenkonvertierungslogik, Zustandsmanagement-Helfer).
    - Werkzeuge: Standard Rust `#[test]`, Mocking-Bibliotheken (z.B. `mockall`) für Abhängigkeiten zu unteren Schichten oder externen Diensten.
- **Widget-Tests**:
    - Fokus: Testen des Verhaltens und Zustands einzelner GTK-Widgets und benutzerdefinierter GObject-Komponenten.
    - Werkzeuge:
        - `gtk::test` Namespace: Bietet Funktionen zum Initialisieren von GTK in Testumgebungen.
        - Programmatische Interaktion: Simulieren von Signalen (z.B. `widget.emit_by_name::<()>("clicked", &)`), Setzen und Abfragen von GObject-Eigenschaften.
        - Inspektion: Überprüfung von Widget-Zuständen (z.B. `label.text()`, `button.is_sensitive()`).
        - GTK-Inspektionswerkzeuge und Accessibility-APIs (ATK) können programmatisch genutzt werden, um Widget-Zustände und -Eigenschaften zu überprüfen.122 Die Evaluierung von Frameworks wie `gtk4-rs-test-utils` (falls existent und passend) oder ähnlichen Ansätzen ist Teil des Untersuchungsbedarfs.
- **Accessibility-Tests**:
    - Fokus: Sicherstellen, dass UI-Komponenten für assistive Technologien zugänglich sind.
    - Werkzeuge: Überprüfung von ATK-Eigenschaften (Rolle, Name, Beschreibung, Zustand) der Widgets. Manuelle Tests mit Screenreadern (z.B. Orca) sind ebenfalls notwendig. `gtk::Accessible`.23
- **Visuelle Regressionstests**: (Zur Evaluierung)
    - Fokus: Erkennen von unbeabsichtigten visuellen Änderungen in der UI.
    - Werkzeuge: Evaluierung von Werkzeugen für den visuellen Vergleich von UI-Zuständen (Screenshots). Dies ist oft aufwendig und wird initial möglicherweise zurückgestellt.
- **Integrations-/End-to-End-Tests**: (Herausfordernd, für kritische Pfade)
    - Fokus: Testen des Zusammenspiels mehrerer UI-Komponenten und deren Interaktion mit unteren Schichten.
    - Werkzeuge: Simulation von Benutzerinteraktionen auf Wayland-Ebene (z.B. mit Tools wie `ydotool` oder spezialisierten Test-Frameworks, falls verfügbar und integrierbar). Überprüfung des Systemverhaltens. Dies ist sehr komplex und wird nur für kritische User Journeys in Betracht gezogen.

### 3.5. Richtlinien für Performance-Optimierung und Profiling

Die Sicherstellung einer performanten UI ist ein Kernziel.

- **Profiling-Werkzeuge**:
    - **Rust-spezifisch**: `perf` unter Linux, `cargo flamegraph`, `tracing` mit `tracing-flame` für CPU-Profiling. Speicher-Profiler wie `heaptrack` oder Valgrind (mit Massif) können zur Analyse des Speicherverbrauchs herangezogen werden.
    - **GTK4-spezifisch**: Der GTK Inspector enthält einen Profiler, der Rendering-Zeiten und Widget-Updates visualisiert. GSK-spezifische Debug-Flags (`GSK_DEBUG`) können Aufschluss über Rendering-Pfade geben.
- **Optimierungsbereiche**:
    - **Widget-Zeichnung**: Bei benutzerdefinierten Zeichnungen mit Cairo (`GtkDrawingArea`) darauf achten, nur die notwendigen Bereiche neu zu zeichnen (`gtk_widget_queue_draw_area`). Komplexität der Zeichenoperationen minimieren.
    - **CSS-Anwendung**: CSS-Selektoren einfach halten. Komplexe Selektoren und Regeln können die Performance beeinträchtigen. Effiziente Aktualisierung von CSS bei Theme-Wechseln.10
    - **Datenbindung**: Übermäßige Nutzung von GObject-Property-Bindings oder zu häufige Benachrichtigungen bei kleinen Änderungen können zu Performance-Engpässen führen. Änderungen ggf. bündeln.
    - **Layout-Performance**: Vermeidung unnötig tiefer Widget-Hierarchien. Effiziente Nutzung von Layout-Managern wie `GtkBox` und `GtkGrid`.
    - **Asynchrone Operationen**: Konsequente Nutzung von `glib::MainContext::spawn_local` für alle potenziell blockierenden Operationen (Netzwerk, Datei-I/O, aufwändige Berechnungen in der Domänenschicht), um UI-Blockaden zu verhindern.7 Visuelles Feedback (Spinner, Fortschrittsbalken) für laufende Operationen bereitstellen.
- **Allgemeine Rust-Optimierungen**: Zero-Cost-Abstraktionen nutzen, unnötige Allokationen vermeiden, effiziente Datenstrukturen wählen.24

Performance-Messungen und -Optimierungen sollten ein integraler Bestandteil des Entwicklungsprozesses sein, nicht eine nachträgliche Maßnahme.

## 4. Plan für nachfolgende UI-Layer-Module

### 4.1. Priorisierung für nächste Module

Nach der initialen Implementierung und Stabilisierung des `PanelWidget` und des `AppMenuButton` sowie der grundlegenden übergreifenden Frameworks (Theming, State Management, Error Handling, Logging) werden die UI-Module in folgender logischer Reihenfolge priorisiert:

1. **Weitere Kern-Panel-Module (`ui::shell`)**:
    - `WorkspaceIndicatorWidget`: Essentiell für die Workspace-Navigation.
    - `ClockDateTimeWidget`: Grundlegende Benutzerinformation.
    - `SystemTrayEquivalentWidget`: Kritisch für die Integration von Drittanbieter-Anwendungen. Aufgrund seiner Komplexität (siehe oben) wird hierfür frühzeitig mit der Detailplanung und Prototyping begonnen.
    - `QuickSettingsButtonWidget` und `NotificationCenterButtonWidget`: Wichtige Zugriffspunkte für Systemfunktionen.
    - Weitere Indikatoren (`NetworkIndicatorWidget`, `PowerIndicatorWidget`, `AudioIndicatorWidget`).
2. **Ausklappbare Panel-Inhalte (`ui::shell`)**:
    - `QuickSettingsPanelWidget`: Wird vom `QuickSettingsButtonWidget` geöffnet.
    - `NotificationCenterPanelWidget`: Wird vom `NotificationCenterButtonWidget` geöffnet und interagiert mit `ui::notifications_frontend`.
3. **Weitere Shell-Komponenten (`ui::shell`)**:
    - `SmartTabBarWidget`
    - `WorkspaceSwitcherWidget`
    - `QuickActionDockWidget`
4. **Systemeinstellungsanwendung (`ui::control_center`)**:
    - Dies ist eine größere, eigenständige Anwendung und wird parallel zu weniger kritischen Shell-Komponenten entwickelt, sobald die Kern-Shell-Interaktionen stabil sind.
5. **Spezifische UI-Frontends und Widgets**:
    - `ui::notifications_frontend` (Popups)
    - `ui::widgets` (Sidebar-Widgets)
    - `ui::window_manager_frontend`
    - `ui::speed_dial`
    - `ui::command_palette`

Diese Priorisierung zielt darauf ab, schnell einen funktionalen Kern der Desktop-Shell zu etablieren und dann schrittweise weitere Funktionen und Anwendungen hinzuzufügen.

### 4.2. Identifizierte Abhängigkeiten und Parallelisierungsmöglichkeiten

- **Abhängigkeiten**:
    - Alle Panel-Module hängen von einem stabilen `PanelWidget` und dessen API ab.
    - `AppMenuButton` und `SystemTrayEquivalentWidget` haben starke Abhängigkeiten von D-Bus-Interaktionen und Wayland-Protokollen (bzw. den Abstraktionsdiensten dafür).
    - `NetworkIndicatorWidget`, `PowerIndicatorWidget`, `AudioIndicatorWidget` hängen von den entsprechenden D-Bus-Schnittstellen der `system`-Schicht ab.
    - `NotificationCenterPanelWidget` hängt von `domain::user_centric_services::NotificationService` und `ui::notifications_frontend::NotificationPopupWidget` ab.
    - `ui::control_center` Module hängen stark von `domain::global_settings_and_state_management::GlobalSettingsService` ab.
- **Parallelisierung**:
    - Sobald die API des `PanelWidget` definiert ist, können viele der darin enthaltenen Module (`ClockDateTimeWidget`, `WorkspaceIndicatorWidget`, einzelne Indikatoren) parallel entwickelt werden.
    - Die Entwicklung des `ui::control_center` kann weitgehend parallel zur Verfeinerung der `ui::shell` erfolgen, sobald die `GlobalSettingsService`-Schnittstelle stabil ist.
    - Wiederverwendbare Komponenten in `ui::components` können frühzeitig parallel entwickelt und in anderen Modulen eingesetzt werden.
    - Die Implementierung der D-Bus-Clients für verschiedene Systemdienste (`NetworkManager`, `UPower`, etc.) kann parallelisiert werden.

Eine enge Abstimmung zwischen den Teams, die an abhängigen Modulen arbeiten, ist entscheidend. Die Definition klarer Schnittstellen (GObject-Properties und -Signale, Rust-Traits) für Module und Dienste erleichtert die parallele Entwicklung und spätere Integration.

# NovaDE UI-Schicht: Implementierungsleitfaden – Teil 2: `WorkspaceIndicatorWidget` und `ClockDateTimeWidget`

## 1. Einleitung

### 1.1. Zweck des Dokuments

Dieses Dokument ist der zweite Teil des detaillierten Implementierungsleitfadens für die UI-Schicht der Nova Desktop Environment (NovaDE). Es baut auf den im ersten Teil etablierten Grundlagen und Spezifikationen auf und fokussiert sich auf zwei weitere wichtige Module innerhalb des `PanelWidget`: das `WorkspaceIndicatorWidget` und das `ClockDateTimeWidget`. Ziel ist es weiterhin, eine ultrafeine Spezifikation bereitzustellen, die Entwicklern eine direkte Implementierung ohne eigene grundlegende Designentscheidungen ermöglicht.

### 1.2. Zielgruppe

Dieses Dokument richtet sich an Softwareentwickler und -architekten, die an der Implementierung der NovaDE UI-Schicht beteiligt sind. Es wird ein Verständnis von Rust, GTK4 und den gtk4-rs Bindings sowie der im ersten Teil dieses Leitfadens beschriebenen Konzepte vorausgesetzt.

### 1.3. Umfang (Teil 2: `WorkspaceIndicatorWidget` und `ClockDateTimeWidget`)

Dieser Teil des Implementierungsleitfadens behandelt die folgenden Module:

- **`ui::shell::panel_widget::WorkspaceIndicatorWidget`**: Ein Panel-Modul zur visuellen Darstellung der verfügbaren Arbeitsbereiche ("Spaces") und des aktuell aktiven Space.
- **`ui::shell::panel_widget::ClockDateTimeWidget`**: Ein Panel-Modul zur Anzeige der aktuellen Uhrzeit und des Datums, mit optionaler Kalenderfunktionalität.

### 1.4. Referenzen auf vorherige Dokumente

Dieser Leitfaden bezieht sich auf die allgemeinen UI-Schicht-Prinzipien, Technologie-Stack-Definitionen und die Spezifikationen für `PanelWidget` und `AppMenuButton` aus "NovaDE UI-Schicht: Implementierungsleitfaden – Teil 1: `ui::shell::PanelWidget` und `AppMenuButton`". Die dort definierten Strategien für Zustandsmanagement, Fehlerbehandlung, Logging und Theming gelten auch hier.

## 2. Modul: `ui::shell::panel_widget::WorkspaceIndicatorWidget`

### 2.1. Übersicht und Verantwortlichkeiten

Das `WorkspaceIndicatorWidget` ist ein Panel-Modul, das dem Benutzer eine klare visuelle Rückmeldung über die verfügbaren und den aktuell aktiven Arbeitsbereich ("Space") gibt. Seine Hauptverantwortlichkeiten sind:

- Anzeige einer konfigurierbaren Anzahl von Indikatoren, die die einzelnen Workspaces repräsentieren.
- Visuelle Hervorhebung des aktuell aktiven Workspace.
- Ermöglichung des Wechsels zu einem anderen Workspace durch Klicken auf den entsprechenden Indikator.
- Dynamische Aktualisierung der Anzeige basierend auf Änderungen in `domain::workspaces::WorkspaceManager`.

### 2.2. Visuelles Design und Theming

- **Darstellung**: Das Widget besteht aus einer horizontalen Anordnung von einzelnen `WorkspaceItemWidget`-Instanzen. Jedes `WorkspaceItemWidget` repräsentiert einen Workspace.
    - **Optionen für `WorkspaceItemWidget`**:
        - **Nummeriert**: Zeigt die Nummer des Workspace an.
        - **Icons**: Zeigt ein Icon an, das dem Workspace zugeordnet ist (z.B. das Icon der primär gepinnten Anwendung oder ein benutzerdefiniertes Space-Icon aus `domain::workspaces`).
        - **Kombiniert**: Icon mit einer kleinen Nummer als Overlay.
    - **Aktiver Workspace**: Das `WorkspaceItemWidget` des aktiven Workspace wird deutlich hervorgehoben, z.B. durch eine andere Hintergrundfarbe (Akzentfarbe des Systems oder des Spaces), einen Rahmen oder eine hellere Text-/Iconfarbe.
- **Layout**: Die `WorkspaceItemWidget`s werden horizontal in einer `gtk::Box` angeordnet.
- **CSS-Styling**:
    - **CSS-Knoten**:
        - `WorkspaceIndicatorWidget` (als `gtk::Box`): `box`.1
        - `WorkspaceItemWidget` (als `gtk::Button` oder benutzerdefiniertes Widget): `button` oder ein benutzerdefinierter CSS-Name.2
    - **CSS-Klassen**:
        - `.workspace-indicator`: Für das Hauptcontainer-Widget.
        - `.workspace-item`: Für jedes einzelne `WorkspaceItemWidget`.
        - `.active-workspace`: Für das `WorkspaceItemWidget`, das den aktiven Workspace repräsentiert.
        - `.workspace-occupied`: (Optional) Für `WorkspaceItemWidget`s, die Fenster enthalten.

### 2.3. Datenstrukturen, Eigenschaften und Zustand

#### 2.3.1. `WorkspaceInfo` (UI-spezifische Datenstruktur)

Diese Struktur dient dem Datenaustausch zwischen der Domänenschicht und dem `WorkspaceIndicatorWidget`.

Rust

```
// In src/ui/shell/panel_widget/workspace_indicator_widget/types.rs (oder ähnlich)
#
pub struct WorkspaceInfo {
    pub id: String, // Eindeutige ID des Workspace (von domain::workspaces::WorkspaceId)
    pub name: String, // Name des Workspace
    pub icon_name: Option<String>, // Optionaler Icon-Name für den Workspace
    pub number: usize, // Ordnungsnummer des Workspace
    pub is_active: bool, // True, wenn dies der aktive Workspace ist
    pub is_occupied: bool, // True, wenn der Workspace Fenster enthält (optional für Styling)
}
```

#### 2.3.2. `WorkspaceItemWidget` (GObject-Subklasse)

Jeder Indikator im `WorkspaceIndicatorWidget` ist eine Instanz von `WorkspaceItemWidget`.

- GObject-Definition (WorkspaceItemWidget):
    
    Erbend von gtk::Button für einfache Klickbarkeit und Styling.
    
    _Auszug aus `src/ui/shell/panel_widget/workspace_indicator_widget/workspace_item_widget/imp.rs` (vereinfacht):_
    
    Rust
    
    ```
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;
    
    #
    pub struct WorkspaceItemWidget {
        workspace_id: RefCell<Option<String>>,
        workspace_number: RefCell<Option<usize>>,
        // Ggf. Referenz auf ein internes gtk::Label oder gtk::Image
        label_widget: TemplateChild<gtk::Label>, // Beispiel, wenn CompositeTemplate genutzt wird
        icon_widget: TemplateChild<gtk::Image>,  // Beispiel
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for WorkspaceItemWidget {
        const NAME: &'static str = "NovaDEWorkspaceItemWidget";
        type Type = super::WorkspaceItemWidget;
        type ParentType = gtk::Button; // Erbt von gtk::Button
    
        fn new() -> Self {
            Self {
                workspace_id: RefCell::new(None),
                workspace_number: RefCell::new(None),
                label_widget: TemplateChild::default(),
                icon_widget: TemplateChild::default(),
            }
        }
        fn class_init(klass: &mut Self::Class) {
            // klass.bind_template(); // Falls.ui Datei verwendet wird
            klass.set_css_name("workspaceitemwidget");
        }
        // fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        //     obj.init_template();
        // }
    }
    impl ObjectImpl for WorkspaceItemWidget {
        // Eigenschaften hier definieren, falls nötig (z.B. "workspace-id", "is-active-item")
        // Für dieses Widget reicht es oft, den Zustand über Methoden zu setzen und
        // das Aussehen direkt anzupassen.
    }
    impl WidgetImpl for WorkspaceItemWidget {}
    impl ButtonImpl for WorkspaceItemWidget {}
    ```
    
    _Auszug aus `src/ui/shell/panel_widget/workspace_indicator_widget/workspace_item_widget/mod.rs` (vereinfacht):_
    
    Rust
    
    ```
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use super::types::WorkspaceInfo; // Annahme, dass WorkspaceInfo hier definiert ist
    
    mod imp;
    
    glib::wrapper! {
        pub struct WorkspaceItemWidget(ObjectSubclass<imp::WorkspaceItemWidget>)
            @extends gtk::Button, gtk::Widget,
            @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
    }
    
    impl WorkspaceItemWidget {
        pub fn new(info: &WorkspaceInfo) -> Self {
            let item: Self = glib::Object::new();
            item.update_content(info);
            item.set_workspace_id(Some(info.id.clone()));
            item.set_workspace_number(Some(info.number));
            item
        }
    
        pub fn workspace_id(&self) -> Option<String> {
            self.imp().workspace_id.borrow().clone()
        }
    
        fn set_workspace_id(&self, id: Option<String>) {
            self.imp().workspace_id.replace(id);
        }
    
        pub fn workspace_number(&self) -> Option<usize> {
            *self.imp().workspace_number.borrow()
        }
    
        fn set_workspace_number(&self, num: Option<usize>) {
            self.imp().workspace_number.replace(num);
        }
    
        pub fn update_content(&self, info: &WorkspaceInfo) {
            // Logik zum Setzen des Labels (Nummer oder Name) oder Icons
            // Beispiel: self.set_label(&info.number.to_string());
            // Oder: self.imp().label_widget.set_text(&info.number.to_string());
            // Oder: if let Some(icon_name) = &info.icon_name {
            //           self.imp().icon_widget.set_from_icon_name(Some(icon_name));
            //           self.set_child(Some(&self.imp().icon_widget));
            //       } else {
            //           self.imp().label_widget.set_text(&info.number.to_string());
            //           self.set_child(Some(&self.imp().label_widget));
            //       }
    
            if info.is_active {
                self.add_css_class("active-workspace");
            } else {
                self.remove_css_class("active-workspace");
            }
            // Ggf. Tooltip setzen: self.set_tooltip_text(Some(&info.name));
        }
    }
    ```
    

#### 2.3.3. `WorkspaceIndicatorWidget` (GObject-Subklasse)

- GObject-Definition (WorkspaceIndicatorWidget):
    
    Erbend von gtk::Box.
    
    _Auszug aus `src/ui/shell/panel_widget/workspace_indicator_widget/imp.rs` (vereinfacht):_
    
    Rust
    
    ```
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use super::workspace_item_widget::WorkspaceItemWidget;
    use super::types::WorkspaceInfo;
    
    #
    pub struct WorkspaceIndicatorWidget {
        // GObject Properties
        max_visible_items: RefCell<u32>, // 0 für unbegrenzt
        show_workspace_names: RefCell<bool>, // Beeinflusst Darstellung in WorkspaceItemWidget
    
        // Interner Zustand
        workspace_items_container: TemplateChild<gtk::Box>, // Die Box, die WorkspaceItemWidgets hält
        workspace_item_widgets: RefCell<HashMap<String, WorkspaceItemWidget>>, // Map von Workspace-ID zu Widget
        // Referenz auf den WorkspaceManager-Proxy oder Event-Listener
        // workspace_event_listener_id: RefCell<Option<glib::SignalHandlerId>>, // Beispiel
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for WorkspaceIndicatorWidget {
        const NAME: &'static str = "NovaDEWorkspaceIndicatorWidget";
        type Type = super::WorkspaceIndicatorWidget;
        type ParentType = gtk::Box;
    
        fn new() -> Self {
            Self {
                max_visible_items: RefCell::new(10), // Standard: max 10 Items
                show_workspace_names: RefCell::new(false), // Standard: Nummern/Icons
                workspace_items_container: TemplateChild::default(),
                workspace_item_widgets: RefCell::new(HashMap::new()),
            }
        }
        fn class_init(klass: &mut Self::Class) {
            // klass.bind_template(); // Falls.ui Datei verwendet wird
            // Properties hier installieren
            klass.set_css_name("workspaceindicator");
        }
        // fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        //     obj.init_template();
        // }
    }
    impl ObjectImpl for WorkspaceIndicatorWidget {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.set_orientation(gtk::Orientation::Horizontal);
            obj.set_spacing(4); // Beispiel für Abstand zwischen Items
            // Initialisiere workspace_items_container, falls nicht über Template
            // self.workspace_items_container.set_orientation(gtk::Orientation::Horizontal);
            // self.obj().append(&self.workspace_items_container);
    
            // Hier Logik zum Abonnieren von Workspace-Events von domain::workspaces
            // obj.subscribe_to_workspace_events();
        }
        // fn dispose() und fn finalize() für Aufräumarbeiten (z.B. Signalhandler trennen)
    }
    impl WidgetImpl for WorkspaceIndicatorWidget {}
    impl BoxImpl for WorkspaceIndicatorWidget {}
    ```
    
- **Eigenschaften (Properties) für `WorkspaceIndicatorWidget`**:
    
    **Tabelle 2.1: `WorkspaceIndicatorWidget` Eigenschaften**
    

|   |   |   |   |   |
|---|---|---|---|---|
|**Eigenschaftsname**|**Typ**|**Zugriff**|**Standardwert**|**Beschreibung**|
|`max-visible-items`|`u32`|Lesen/Schreiben|`10`|Maximale Anzahl von Workspace-Indikatoren, die angezeigt werden. `0` bedeutet unbegrenzt. Bei Überschreitung könnten z.B. Pfeile oder ein Überlaufmenü erscheinen (zukünftige Erweiterung).|
|`show-workspace-names`|`bool`|Lesen/Schreiben|`false`|Wenn `true`, versuchen `WorkspaceItemWidget`s, den Namen des Workspace anzuzeigen, sonst eher Nummer oder Icon.|

### 2.4. API und Methoden

#### 2.4.1. `WorkspaceIndicatorWidget`

- `pub fn new() -> Self`
    - Signatur: `pub fn new() -> Self`
    - Verantwortlichkeit: Konstruktor. Initialisiert die interne `gtk::Box` für die `WorkspaceItemWidget`s und abonniert sich bei `domain::workspaces::WorkspaceManager` für `WorkspaceEvent`s.
- `pub fn update_workspaces(&self, workspaces_info: Vec<WorkspaceInfo>) noexcept;`
    - Signatur: `pub fn update_workspaces(&self, workspaces_info: Vec<super::types::WorkspaceInfo>) noexcept`
    - Verantwortlichkeit: Hauptmethode zur Aktualisierung der Anzeige.
        1. Entfernt alle aktuellen `WorkspaceItemWidget`s aus dem internen Container.
        2. Löscht den `workspace_item_widgets` HashMap.
        3. Iteriert über `workspaces_info`:
            - Erstellt für jeden Eintrag ein neues `WorkspaceItemWidget` mit den Daten aus `WorkspaceInfo`.
            - Verbindet das `clicked`-Signal des `WorkspaceItemWidget` mit der internen Methode `on_workspace_item_clicked`.
            - Fügt das `WorkspaceItemWidget` dem internen Container (`workspace_items_container`) hinzu.
            - Speichert das `WorkspaceItemWidget` im `workspace_item_widgets` HashMap (ID als Schlüssel).
        4. Berücksichtigt `max-visible-items` (initial einfach abschneiden, später ggf. komplexere Logik).
- `fn on_workspace_item_clicked(&self, item_widget: &WorkspaceItemWidget) noexcept;`
    - Signatur: `fn on_workspace_item_clicked(&self, item_widget: &super::workspace_item_widget::WorkspaceItemWidget) noexcept`
    - Verantwortlichkeit: Wird aufgerufen, wenn ein `WorkspaceItemWidget` geklickt wird.
        1. Ermittelt die `workspace_id` des geklickten Items.
        2. Sendet einen Befehl/Aktion an `domain::workspaces::WorkspaceManager`, um den aktiven Workspace zu wechseln (z.B. über eine UI-Aktionsschicht oder einen direkten Aufruf, falls die Architektur dies erlaubt und Thread-sicher ist).
- `fn handle_workspace_event(&self, event: &domain::workspaces::WorkspaceEvent) noexcept;` (Beispielhafter Name)
    - Signatur: `fn handle_workspace_event(&self, event: &<Pfad_zu_domain>::workspaces::WorkspaceEvent) noexcept`
    - Verantwortlichkeit: Callback-Funktion, die auf `WorkspaceEvent`s von `domain::workspaces::WorkspaceManager` reagiert.
        1. Fordert die aktuelle Liste aller Workspaces und den aktiven Workspace vom `WorkspaceManager` an.
        2. Konvertiert die Domain-Daten in eine `Vec<WorkspaceInfo>`.
        3. Ruft `self.update_workspaces()` mit den neuen Daten auf. _Diese Funktion wird typischerweise über `glib::MainContext::spawn_local` aufgerufen, wenn das Domain-Event aus einem anderen Thread kommt._ 4

#### 2.4.2. `WorkspaceItemWidget`

- `pub fn new(info: &WorkspaceInfo) -> Self` (siehe oben)
- `pub fn update_content(&self, info: &WorkspaceInfo) noexcept;` (siehe oben)
    - Aktualisiert das Label (Nummer/Name) und/oder das Icon basierend auf `info.icon_name`, `info.name`, `info.number` und der `show-workspace-names`-Eigenschaft des Parent-`WorkspaceIndicatorWidget`.
    - Setzt die CSS-Klasse `.active-workspace` basierend auf `info.is_active`.
    - Setzt optional einen Tooltip mit `info.name`.

### 2.5. Signale und Ereignisse

#### 2.5.1. `WorkspaceIndicatorWidget`

- **Emittierte Signale**: Keine spezifischen benutzerdefinierten Signale für externe Nutzung vorgesehen. Interne Logik wird durch Methodenaufrufe und Property-Änderungen gesteuert.
- **Verbundene Signale/Events**:
    - Abonniert `WorkspaceEvent`s von `domain::workspaces::WorkspaceManager`.
        - Publisher: `domain::workspaces::WorkspaceManager`.
        - Subscriber: `WorkspaceIndicatorWidget::handle_workspace_event`.
        - Zweck: Aktualisierung der Workspace-Anzeige bei Änderungen in der Domänenschicht.

#### 2.5.2. `WorkspaceItemWidget`

- **Emittierte Signale**: Erbt das `clicked`-Signal von `gtk::Button`.
- **Verbundene Signale**: Keine spezifischen.

### 2.6. Fehlerbehandlung

- **`enum WorkspaceIndicatorError`** (definiert mit `thiserror`):
    - `DomainCommunicationError(String)`: Fehler bei der Kommunikation mit `domain::workspaces::WorkspaceManager`.
    - `ItemCreationFailed(String)`: Wenn ein `WorkspaceItemWidget` nicht erstellt werden kann.
- Fehler werden primär geloggt (`tracing`). Eine fehlerhafte Aktualisierung könnte dazu führen, dass der Indikator veraltete Informationen anzeigt oder leer ist. Kritische Fehler bei der Initialisierung (z.B. Verbindung zum Domain-Service nicht möglich) könnten das Hinzufügen des Widgets zum Panel verhindern.

### 2.7. Implementierungsplan und Schritte

1. Definiere `WorkspaceInfo` Struktur.
2. Implementiere `WorkspaceItemWidget` (`workspace_item_widget/mod.rs` und `imp.rs`):
    - Subklasse von `gtk::Button`.
    - Implementiere `new()` und `update_content()`.
    - Stelle sicher, dass Klick-Events korrekt weitergeleitet werden können (durch das `clicked`-Signal der Basisklasse).
3. Implementiere `WorkspaceIndicatorWidget` (`workspace_indicator_widget/mod.rs` und `imp.rs`):
    - Subklasse von `gtk::Box`.
    - Definiere GObject-Properties (`max-visible-items`, `show-workspace-names`).
    - Implementiere `new()`: Initialisiere internen Container, abonniere Workspace-Events.
    - Implementiere `update_workspaces()`: Logik zum Leeren und neu Befüllen des Containers mit `WorkspaceItemWidget`s.
    - Implementiere `on_workspace_item_clicked()`: Logik zum Auslösen des Workspace-Wechsels.
    - Implementiere `handle_workspace_event()`: Anforderung neuer Daten und Aufruf von `update_workspaces`.
4. Integration: Das `PanelWidget` instanziiert das `WorkspaceIndicatorWidget` und fügt es einem seiner Bereiche hinzu.

### 2.8. Dateistruktur

```
src/
└── ui/
    └── shell/
        └── panel_widget/
            └── workspace_indicator_widget/
                ├── mod.rs                      // Öffentliche API für WorkspaceIndicatorWidget
                ├── imp.rs                      // GObject-Implementierung für WorkspaceIndicatorWidget
                ├── types.rs                    // Definition von WorkspaceInfo
                ├── workspace_item_widget.rs    // Enthält mod.rs und imp.rs für WorkspaceItemWidget
                │   ├── mod.rs                  // Öffentliche API für WorkspaceItemWidget
                │   └── imp.rs                  // GObject-Implementierung für WorkspaceItemWidget
                └── error.rs                    // (Optional) Definition von WorkspaceIndicatorError
```

## 3. Modul: `ui::shell::panel_widget::ClockDateTimeWidget`

### 3.1. Übersicht und Verantwortlichkeiten

Das `ClockDateTimeWidget` ist ein Panel-Modul, das die aktuelle Systemzeit und das Datum anzeigt. Optional kann es bei einem Klick ein Popover mit einem Kalender anzeigen.

- Anzeige der formatierten aktuellen Uhrzeit und des Datums.
- Regelmäßige Aktualisierung der Anzeige.
- Optionale Anzeige eines Kalender-Popovers bei Klick.
- Konfigurierbarkeit des Zeit-/Datumsformats.

### 3.2. Visuelles Design und Theming

- **Anzeige**: Textbasierte Anzeige der Zeit und des Datums.
    - Schriftart, -größe und -farbe werden durch das globale GTK-Theme und spezifische CSS-Regeln gesteuert.
- **Kalender-Popover**: Ein Standard `gtk::Popover`, das ein `gtk::Calendar`-Widget enthält.6
- **CSS-Styling**:
    - **CSS-Knoten**:
        - `ClockDateTimeWidget` (als `gtk::Button` oder `gtk::Label`): `button` oder `label`.
        - Kalender-Popover: `popover`.
    - **CSS-Klassen**:
        - `.clock-datetime-widget`: Für das Haupt-Widget.
        - `.calendar-popover-content`: Für den Inhalt des Kalender-Popovers (falls benutzerdefinierte Anpassungen nötig sind).

### 3.3. Datenstrukturen, Eigenschaften und Zustand

- GObject-Definition (ClockDateTimeWidget):
    
    Erbend von gtk::Button (um Klickbarkeit für das Popover zu ermöglichen) oder gtk::Label (wenn kein Popover). Für die Popover-Funktionalität ist gtk::Button oder ein gtk::Box mit einem gtk::GestureClick geeigneter.
    
    _Auszug aus `src/ui/shell/panel_widget/clock_datetime_widget/imp.rs` (vereinfacht):_
    
    Rust
    
    ```
    use gtk::glib;
    use gtk::subclass::prelude::*;
    use std::cell::{Cell, RefCell};
    use chrono::{DateTime, Local, SecondsFormat}; // [8]
    
    #
    pub struct ClockDateTimeWidget {
        // GObject Properties
        format_string: RefCell<String>,
        show_calendar_on_click: Cell<bool>,
    
        // Interner Zustand
        timer_id: RefCell<Option<glib::SourceId>>, // Für den Aktualisierungs-Timer [9, 10]
        calendar_popover: RefCell<Option<gtk::Popover>>,
        // Das Label-Widget, das die Zeit anzeigt (kann Kind des Buttons sein)
        time_label: TemplateChild<gtk::Label>,
    }
    
    #[glib::object_subclass]
    impl ObjectSubclass for ClockDateTimeWidget {
        const NAME: &'static str = "NovaDEClockDateTimeWidget";
        type Type = super::ClockDateTimeWidget;
        // Erbt von gtk::Button, um Klickbarkeit und Popover-Ankerung zu erleichtern
        type ParentType = gtk::Button;
    
        fn new() -> Self {
            Self {
                format_string: RefCell::new("%H:%M".to_string()), // Standardformat [11]
                show_calendar_on_click: Cell::new(true),
                timer_id: RefCell::new(None),
                calendar_popover: RefCell::new(None),
                time_label: TemplateChild::default(),
            }
        }
        fn class_init(klass: &mut Self::Class) {
            // klass.bind_template(); // Falls.ui Datei verwendet wird
            // Properties hier installieren
            klass.set_css_name("clockdatetimewidget");
        }
        // fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
        //     obj.init_template();
        // }
    }
    
    impl ObjectImpl for ClockDateTimeWidget {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
    
            // Initialisiere time_label, falls nicht über Template
            // self.time_label.set_use_markup(true); // Für Pango-Markup, falls benötigt
            // obj.set_child(Some(&self.time_label));
    
            obj.update_display(); // Erste Anzeige
            obj.setup_timer();    // Timer starten
    
            // Klick-Handler für Popover verbinden, falls show_calendar_on_click
            if self.show_calendar_on_click.get() {
                obj.connect_clicked(glib::clone!(@weak obj => move |_| {
                    obj.show_calendar_popover();
                }));
            }
        }
        fn dispose(&self) {
            if let Some(source_id) = self.timer_id.borrow_mut().take() {
                source_id.remove();
            }
            if let Some(popover) = self.calendar_popover.borrow_mut().take() {
                popover.unparent(); // Sicherstellen, dass das Popover entfernt wird
            }
        }
        // Properties hier implementieren (set_property, property)
    }
    impl WidgetImpl for ClockDateTimeWidget {}
    impl ButtonImpl for ClockDateTimeWidget {}
    ```
    
- **Eigenschaften (Properties) für `ClockDateTimeWidget`**:
    
    **Tabelle 3.1: `ClockDateTimeWidget` Eigenschaften**
    

|   |   |   |   |   |
|---|---|---|---|---|
|**Eigenschaftsname**|**Typ**|**Zugriff**|**Standardwert**|**Beschreibung**|
|`format-string`|`String`|Lesen/Schreiben|`"%H:%M"`|Der `strftime`-kompatible Format-String für die Zeit-/Datumsanzeige.11|
|`show-calendar-on-click`|`bool`|Lesen/Schreiben|`true`|Wenn `true`, wird bei Klick auf das Widget ein Kalender-Popover angezeigt.|

### 3.4. API und Methoden

- `pub fn new() -> Self`
    - Signatur: `pub fn new() -> Self`
    - Verantwortlichkeit: Konstruktor. Initialisiert das Label, startet den Timer und verbindet ggf. den Klick-Handler für das Kalender-Popover.
- `fn update_display(&self) noexcept;`
    - Signatur: `fn update_display(&self) noexcept`
    - Verantwortlichkeit: Ruft die aktuelle Zeit ab (`chrono::Local::now()` 8), formatiert sie gemäß `format-string` und aktualisiert den Text des internen `gtk::Label` (oder des Button-Labels).
    - Beispiel Formatierung: `Local::now().format(self.imp().format_string.borrow().as_str()).to_string()`.11
- `fn setup_timer(&self) noexcept;`
    - Signatur: `fn setup_timer(&self) noexcept`
    - Verantwortlichkeit: Startet einen Timer, der periodisch (z.B. jede Sekunde oder Minute, abhängig von der Granularität des `format-string`) `update_display` aufruft.
    - Verwendet `glib::timeout_add_seconds_local` oder `glib::timeout_add_local` für feinere Intervalle.9
    - Speichert die `glib::SourceId` im `timer_id`-Feld, um den Timer später entfernen zu können (im `dispose`-Handler).
- `fn show_calendar_popover(&self) noexcept;`
    - Signatur: `fn show_calendar_popover(&self) noexcept`
    - Verantwortlichkeit: Erstellt (falls noch nicht geschehen) und zeigt ein `gtk::Popover` an, das ein `gtk::Calendar`-Widget enthält.6
        1. Wenn `self.imp().calendar_popover` `None` ist:
            - Erstelle ein `gtk::Calendar::new()`.
            - Erstelle ein `gtk::Popover::builder().child(&calendar_widget).build()`.
            - Setze das Popover als Kind des `ClockDateTimeWidget` (oder eines spezifischen Ankerpunkts).
            - Speichere das Popover in `self.imp().calendar_popover`.
        2. Setze das Popover relativ zum `ClockDateTimeWidget` (dem Button): `popover.set_parent(self.obj().as_ref())`.
        3. Zeige das Popover: `popover.popup()`.

### 3.5. Signale und Ereignisse

- **Emittierte Signale**: Keine spezifischen benutzerdefinierten Signale. Erbt Signale von `gtk::Button` (z.B. `clicked`).
- **Verbundene Signale/Events**:
    - Interner Timer-Event von `glib::timeout_add_seconds_local`.
    - `clicked`-Signal des Buttons (falls `show-calendar-on-click` true ist), um `show_calendar_popover` aufzurufen.
    - Lauscht auf Änderungen der relevanten Einstellungen von `domain::global_settings_and_state_management` (z.B. für `format-string`), um die GObject-Properties zu aktualisieren.

### 3.6. Fehlerbehandlung

- **`enum ClockDateTimeError`** (definiert mit `thiserror`):
    - `InvalidFormatString(String)`: Wenn der `format-string` ungültig ist und `chrono::format` fehlschlägt.
    - `TimerSetupFailed(String)`: Wenn der `glib`-Timer nicht eingerichtet werden kann.
- Fehler werden geloggt. Ein ungültiger Format-String könnte zu einer Standardanzeige oder einer Fehlermeldung im Widget führen.

### 3.7. Implementierungsplan und Schritte

1. Definiere `ClockDateTimeWidget` (`clock_datetime_widget/mod.rs` und `imp.rs`):
    - Subklasse von `gtk::Button`.
    - Definiere GObject-Properties (`format-string`, `show-calendar-on-click`).
    - Implementiere `new()`: Initialisiere das interne Label (als Kind des Buttons), rufe `update_display()` und `setup_timer()` auf. Verbinde `clicked`-Signal, falls `show-calendar-on-click`.
2. Implementiere `update_display()`: Hole aktuelle Zeit, formatiere sie, setze Label-Text.
3. Implementiere `setup_timer()`: Nutze `glib::timeout_add_seconds_local` (z.B. für sekündliche Updates), speichere `SourceId`. Der Callback ruft `update_display()` auf und gibt `glib::ControlFlow::Continue` zurück.
4. Implementiere `show_calendar_popover()`: Erstelle und zeige `gtk::Popover` mit `gtk::Calendar`.
5. Implementiere den `dispose`-Handler, um den Timer zu entfernen.
6. Integration: Das `PanelWidget` instanziiert das `ClockDateTimeWidget` und fügt es einem seiner Bereiche hinzu.
7. Verbindung mit `domain::global_settings_and_state_management`: Lausche auf Änderungen der Format-String-Einstellung und aktualisiere die `format-string`-Property des Widgets.

### 3.8. Dateistruktur

```
src/
└── ui/
    └── shell/
        └── panel_widget/
            └── clock_datetime_widget/
                ├── mod.rs          // Öffentliche API für ClockDateTimeWidget
                ├── imp.rs          // GObject-Implementierung
                └── error.rs        // (Optional) Definition von ClockDateTimeError
```

## 4. Schlussfolgerungen (für Teil 2)

Die Spezifikationen für `WorkspaceIndicatorWidget` und `ClockDateTimeWidget` erweitern die Funktionalität des `PanelWidget` um zwei wichtige, benutzerorientierte Module. Die Implementierung des `WorkspaceIndicatorWidget` erfordert eine enge Kopplung mit `domain::workspaces::WorkspaceManager`, um stets aktuelle Informationen über die Arbeitsbereiche anzuzeigen und Interaktionen zu ermöglichen. Das `ClockDateTimeWidget` bietet eine grundlegende, aber essentielle Funktion mit optionaler Erweiterung durch ein Kalender-Popover.

Die konsequente Anwendung von GObject-Subclassing, die Nutzung von GTK-Standardwidgets und -Containern sowie die klare Definition von Schnittstellen und Verantwortlichkeiten sind weiterhin zentrale Aspekte. Die Fehlerbehandlung und das Logging folgen den im ersten Teil etablierten Mustern.

Mit diesen Modulen nimmt die Kernfunktionalität des Panels weiter Gestalt an. Die nächsten Schritte werden sich auf weitere Indikatoren und komplexere Panel-Module wie das `SystemTrayEquivalentWidget` konzentrieren.

NovaDE UI-Schicht: Implementierungsleitfaden – Teil 2: WorkspaceIndicatorWidget und ClockDateTimeWidget1. Einleitung1.1. Zweck des DokumentsDieses Dokument ist der zweite Teil des detaillierten Implementierungsleitfadens für die UI-Schicht der Nova Desktop Environment (NovaDE). Es baut auf den im ersten Teil etablierten Grundlagen und Spezifikationen auf und fokussiert sich auf zwei weitere wichtige Module innerhalb des PanelWidget: das WorkspaceIndicatorWidget und das ClockDateTimeWidget. Ziel ist es weiterhin, eine ultrafeine Spezifikation bereitzustellen, die Entwicklern eine direkte Implementierung ohne eigene grundlegende Designentscheidungen ermöglicht.1.2. ZielgruppeDieses Dokument richtet sich an Softwareentwickler und -architekten, die an der Implementierung der NovaDE UI-Schicht beteiligt sind. Es wird ein Verständnis von Rust, GTK4 und den gtk4-rs Bindings sowie der im ersten Teil dieses Leitfadens beschriebenen Konzepte vorausgesetzt.1.3. Umfang (Teil 2: WorkspaceIndicatorWidget und ClockDateTimeWidget)Dieser Teil des Implementierungsleitfadens behandelt die folgenden Module:
ui::shell::panel_widget::WorkspaceIndicatorWidget: Ein Panel-Modul zur visuellen Darstellung der verfügbaren Arbeitsbereiche ("Spaces") und des aktuell aktiven Space.
ui::shell::panel_widget::ClockDateTimeWidget: Ein Panel-Modul zur Anzeige der aktuellen Uhrzeit und des Datums, mit optionaler Kalenderfunktionalität.
1.4. Referenzen auf vorherige DokumenteDieser Leitfaden bezieht sich auf die allgemeinen UI-Schicht-Prinzipien, Technologie-Stack-Definitionen und die Spezifikationen für PanelWidget und AppMenuButton aus "NovaDE UI-Schicht: Implementierungsleitfaden – Teil 1: ui::shell::PanelWidget und AppMenuButton". Die dort definierten Strategien für Zustandsmanagement, Fehlerbehandlung, Logging und Theming gelten auch hier.2. Modul: ui::shell::panel_widget::WorkspaceIndicatorWidget2.1. Übersicht und VerantwortlichkeitenDas WorkspaceIndicatorWidget ist ein Panel-Modul, das dem Benutzer eine klare visuelle Rückmeldung über die verfügbaren und den aktuell aktiven Arbeitsbereich ("Space") gibt. Seine Hauptverantwortlichkeiten sind:
Anzeige einer konfigurierbaren Anzahl von Indikatoren, die die einzelnen Workspaces repräsentieren.
Visuelle Hervorhebung des aktuell aktiven Workspace.
Ermöglichung des Wechsels zu einem anderen Workspace durch Klicken auf den entsprechenden Indikator.
Dynamische Aktualisierung der Anzeige basierend auf Änderungen in domain::workspaces::WorkspaceManager.
2.2. Visuelles Design und Theming
Darstellung: Das Widget besteht aus einer horizontalen Anordnung von einzelnen WorkspaceItemWidget-Instanzen. Jedes WorkspaceItemWidget repräsentiert einen Workspace.

Optionen für WorkspaceItemWidget:

Nummeriert: Zeigt die Nummer des Workspace an.
Icons: Zeigt ein Icon an, das dem Workspace zugeordnet ist (z.B. das Icon der primär gepinnten Anwendung oder ein benutzerdefiniertes Space-Icon aus domain::workspaces).
Kombiniert: Icon mit einer kleinen Nummer als Overlay.


Aktiver Workspace: Das WorkspaceItemWidget des aktiven Workspace wird deutlich hervorgehoben, z.B. durch eine andere Hintergrundfarbe (Akzentfarbe des Systems oder des Spaces), einen Rahmen oder eine hellere Text-/Iconfarbe.


Layout: Die WorkspaceItemWidgets werden horizontal in einer gtk::Box angeordnet.
CSS-Styling:

CSS-Knoten:

WorkspaceIndicatorWidget (als gtk::Box): box.1
WorkspaceItemWidget (als gtk::Button oder benutzerdefiniertes Widget): button oder ein benutzerdefinierter CSS-Name.2


CSS-Klassen:

.workspace-indicator: Für das Hauptcontainer-Widget.
.workspace-item: Für jedes einzelne WorkspaceItemWidget.
.active-workspace: Für das WorkspaceItemWidget, das den aktiven Workspace repräsentiert.
.workspace-occupied: (Optional) Für WorkspaceItemWidgets, die Fenster enthalten.




2.3. Datenstrukturen, Eigenschaften und Zustand2.3.1. WorkspaceInfo (UI-spezifische Datenstruktur)Diese Struktur dient dem Datenaustausch zwischen der Domänenschicht und dem WorkspaceIndicatorWidget.Rust// In src/ui/shell/panel_widget/workspace_indicator_widget/types.rs (oder ähnlich)
#
pub struct WorkspaceInfo {
    pub id: String, // Eindeutige ID des Workspace (von domain::workspaces::WorkspaceId)
    pub name: String, // Name des Workspace
    pub icon_name: Option<String>, // Optionaler Icon-Name für den Workspace
    pub number: usize, // Ordnungsnummer des Workspace
    pub is_active: bool, // True, wenn dies der aktive Workspace ist
    pub is_occupied: bool, // True, wenn der Workspace Fenster enthält (optional für Styling)
}
2.3.2. WorkspaceItemWidget (GObject-Subklasse)Jeder Indikator im WorkspaceIndicatorWidget ist eine Instanz von WorkspaceItemWidget.

GObject-Definition (WorkspaceItemWidget):Erbend von gtk::Button für einfache Klickbarkeit und Styling.
Auszug aus src/ui/shell/panel_widget/workspace_indicator_widget/workspace_item_widget/imp.rs (vereinfacht):
Rustuse gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

#
pub struct WorkspaceItemWidget {
    workspace_id: RefCell<Option<String>>,
    workspace_number: RefCell<Option<usize>>,
    // Ggf. Referenz auf ein internes gtk::Label oder gtk::Image
    label_widget: TemplateChild<gtk::Label>, // Beispiel, wenn CompositeTemplate genutzt wird
    icon_widget: TemplateChild<gtk::Image>,  // Beispiel
}

#[glib::object_subclass]
impl ObjectSubclass for WorkspaceItemWidget {
    const NAME: &'static str = "NovaDEWorkspaceItemWidget";
    type Type = super::WorkspaceItemWidget;
    type ParentType = gtk::Button; // Erbt von gtk::Button

    fn new() -> Self {
        Self {
            workspace_id: RefCell::new(None),
            workspace_number: RefCell::new(None),
            label_widget: TemplateChild::default(),
            icon_widget: TemplateChild::default(),
        }
    }
    fn class_init(klass: &mut Self::Class) {
        // klass.bind_template(); // Falls.ui Datei verwendet wird
        klass.set_css_name("workspaceitemwidget");
    }
    // fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
    //     obj.init_template();
    // }
}
impl ObjectImpl for WorkspaceItemWidget {
    // Eigenschaften hier definieren, falls nötig (z.B. "workspace-id", "is-active-item")
    // Für dieses Widget reicht es oft, den Zustand über Methoden zu setzen und
    // das Aussehen direkt anzupassen.
}
impl WidgetImpl for WorkspaceItemWidget {}
impl ButtonImpl for WorkspaceItemWidget {}

Auszug aus src/ui/shell/panel_widget/workspace_indicator_widget/workspace_item_widget/mod.rs (vereinfacht):
Rustuse gtk::glib;
use gtk::subclass::prelude::*;
use super::types::WorkspaceInfo; // Annahme, dass WorkspaceInfo hier definiert ist

mod imp;

glib::wrapper! {
    pub struct WorkspaceItemWidget(ObjectSubclass<imp::WorkspaceItemWidget>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl WorkspaceItemWidget {
    pub fn new(info: &WorkspaceInfo) -> Self {
        let item: Self = glib::Object::new();
        item.update_content(info);
        item.set_workspace_id(Some(info.id.clone()));
        item.set_workspace_number(Some(info.number));
        item
    }

    pub fn workspace_id(&self) -> Option<String> {
        self.imp().workspace_id.borrow().clone()
    }

    fn set_workspace_id(&self, id: Option<String>) {
        self.imp().workspace_id.replace(id);
    }

    pub fn workspace_number(&self) -> Option<usize> {
        *self.imp().workspace_number.borrow()
    }

    fn set_workspace_number(&self, num: Option<usize>) {
        self.imp().workspace_number.replace(num);
    }

    pub fn update_content(&self, info: &WorkspaceInfo) {
        // Logik zum Setzen des Labels (Nummer oder Name) oder Icons
        // Beispiel: self.set_label(&info.number.to_string());
        // Oder: self.imp().label_widget.set_text(&info.number.to_string());
        // Oder: if let Some(icon_name) = &info.icon_name {
        //           self.imp().icon_widget.set_from_icon_name(Some(icon_name));
        //           self.set_child(Some(&self.imp().icon_widget));
        //       } else {
        //           self.imp().label_widget.set_text(&info.number.to_string());
        //           self.set_child(Some(&self.imp().label_widget));
        //       }

        if info.is_active {
            self.add_css_class("active-workspace");
        } else {
            self.remove_css_class("active-workspace");
        }
        // Ggf. Tooltip setzen: self.set_tooltip_text(Some(&info.name));
    }
}


2.3.3. WorkspaceIndicatorWidget (GObject-Subklasse)

GObject-Definition (WorkspaceIndicatorWidget):Erbend von gtk::Box.
Auszug aus src/ui/shell/panel_widget/workspace_indicator_widget/imp.rs (vereinfacht):
Rustuse gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use super::workspace_item_widget::WorkspaceItemWidget;
use super::types::WorkspaceInfo;

#
pub struct WorkspaceIndicatorWidget {
    // GObject Properties
    max_visible_items: RefCell<u32>, // 0 für unbegrenzt
    show_workspace_names: RefCell<bool>, // Beeinflusst Darstellung in WorkspaceItemWidget

    // Interner Zustand
    workspace_items_container: TemplateChild<gtk::Box>, // Die Box, die WorkspaceItemWidgets hält
    workspace_item_widgets: RefCell<HashMap<String, WorkspaceItemWidget>>, // Map von Workspace-ID zu Widget
    // Referenz auf den WorkspaceManager-Proxy oder Event-Listener
    // workspace_event_listener_id: RefCell<Option<glib::SignalHandlerId>>, // Beispiel
}

#[glib::object_subclass]
impl ObjectSubclass for WorkspaceIndicatorWidget {
    const NAME: &'static str = "NovaDEWorkspaceIndicatorWidget";
    type Type = super::WorkspaceIndicatorWidget;
    type ParentType = gtk::Box;

    fn new() -> Self {
        Self {
            max_visible_items: RefCell::new(10), // Standard: max 10 Items
            show_workspace_names: RefCell::new(false), // Standard: Nummern/Icons
            workspace_items_container: TemplateChild::default(),
            workspace_item_widgets: RefCell::new(HashMap::new()),
        }
    }
    fn class_init(klass: &mut Self::Class) {
        // klass.bind_template(); // Falls.ui Datei verwendet wird
        // Properties hier installieren
        klass.set_css_name("workspaceindicator");
    }
    // fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
    //     obj.init_template();
    // }
}
impl ObjectImpl for WorkspaceIndicatorWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();
        obj.set_orientation(gtk::Orientation::Horizontal);
        obj.set_spacing(4); // Beispiel für Abstand zwischen Items
        // Initialisiere workspace_items_container, falls nicht über Template
        // self.workspace_items_container.set_orientation(gtk::Orientation::Horizontal);
        // self.obj().append(&self.workspace_items_container);

        // Hier Logik zum Abonnieren von Workspace-Events von domain::workspaces
        // obj.subscribe_to_workspace_events();
    }
    // fn dispose() und fn finalize() für Aufräumarbeiten (z.B. Signalhandler trennen)
}
impl WidgetImpl for WorkspaceIndicatorWidget {}
impl BoxImpl for WorkspaceIndicatorWidget {}



Eigenschaften (Properties) für WorkspaceIndicatorWidget:
Tabelle 2.1: WorkspaceIndicatorWidget Eigenschaften

EigenschaftsnameTypZugriffStandardwertBeschreibungmax-visible-itemsu32Lesen/Schreiben10Maximale Anzahl von Workspace-Indikatoren, die angezeigt werden. 0 bedeutet unbegrenzt. Bei Überschreitung könnten z.B. Pfeile oder ein Überlaufmenü erscheinen (zukünftige Erweiterung).show-workspace-namesboolLesen/SchreibenfalseWenn true, versuchen WorkspaceItemWidgets, den Namen des Workspace anzuzeigen, sonst eher Nummer oder Icon.2.4. API und Methoden2.4.1. WorkspaceIndicatorWidget
pub fn new() -> Self

Signatur: pub fn new() -> Self
Verantwortlichkeit: Konstruktor. Initialisiert die interne gtk::Box für die WorkspaceItemWidgets und abonniert sich bei domain::workspaces::WorkspaceManager für WorkspaceEvents.


pub fn update_workspaces(&self, workspaces_info: Vec<WorkspaceInfo>) noexcept;

Signatur: pub fn update_workspaces(&self, workspaces_info: Vec<super::types::WorkspaceInfo>) noexcept
Verantwortlichkeit: Hauptmethode zur Aktualisierung der Anzeige.

Entfernt alle aktuellen WorkspaceItemWidgets aus dem internen Container.
Löscht den workspace_item_widgets HashMap.
Iteriert über workspaces_info:

Erstellt für jeden Eintrag ein neues WorkspaceItemWidget mit den Daten aus WorkspaceInfo.
Verbindet das clicked-Signal des WorkspaceItemWidget mit der internen Methode on_workspace_item_clicked.
Fügt das WorkspaceItemWidget dem internen Container (workspace_items_container) hinzu.
Speichert das WorkspaceItemWidget im workspace_item_widgets HashMap (ID als Schlüssel).


Berücksichtigt max-visible-items (initial einfach abschneiden, später ggf. komplexere Logik).




fn on_workspace_item_clicked(&self, item_widget: &WorkspaceItemWidget) noexcept;

Signatur: fn on_workspace_item_clicked(&self, item_widget: &super::workspace_item_widget::WorkspaceItemWidget) noexcept
Verantwortlichkeit: Wird aufgerufen, wenn ein WorkspaceItemWidget geklickt wird.

Ermittelt die workspace_id des geklickten Items.
Sendet einen Befehl/Aktion an domain::workspaces::WorkspaceManager, um den aktiven Workspace zu wechseln (z.B. über eine UI-Aktionsschicht oder einen direkten Aufruf, falls die Architektur dies erlaubt und Thread-sicher ist).




fn handle_workspace_event(&self, event: &domain::workspaces::WorkspaceEvent) noexcept; (Beispielhafter Name)

Signatur: fn handle_workspace_event(&self, event: &<Pfad_zu_domain>::workspaces::WorkspaceEvent) noexcept
Verantwortlichkeit: Callback-Funktion, die auf WorkspaceEvents von domain::workspaces::WorkspaceManager reagiert.

Fordert die aktuelle Liste aller Workspaces und den aktiven Workspace vom WorkspaceManager an.
Konvertiert die Domain-Daten in eine Vec<WorkspaceInfo>.
Ruft self.update_workspaces() mit den neuen Daten auf.
Diese Funktion wird typischerweise über glib::MainContext::spawn_local aufgerufen, wenn das Domain-Event aus einem anderen Thread kommt. 4




2.4.2. WorkspaceItemWidget
pub fn new(info: &WorkspaceInfo) -> Self (siehe oben)
pub fn update_content(&self, info: &WorkspaceInfo) noexcept; (siehe oben)

Aktualisiert das Label (Nummer/Name) und/oder das Icon basierend auf info.icon_name, info.name, info.number und der show-workspace-names-Eigenschaft des Parent-WorkspaceIndicatorWidget.
Setzt die CSS-Klasse .active-workspace basierend auf info.is_active.
Setzt optional einen Tooltip mit info.name.


2.5. Signale und Ereignisse2.5.1. WorkspaceIndicatorWidget
Emittierte Signale: Keine spezifischen benutzerdefinierten Signale für externe Nutzung vorgesehen. Interne Logik wird durch Methodenaufrufe und Property-Änderungen gesteuert.
Verbundene Signale/Events:

Abonniert WorkspaceEvents von domain::workspaces::WorkspaceManager.

Publisher: domain::workspaces::WorkspaceManager.
Subscriber: WorkspaceIndicatorWidget::handle_workspace_event.
Zweck: Aktualisierung der Workspace-Anzeige bei Änderungen in der Domänenschicht.




2.5.2. WorkspaceItemWidget
Emittierte Signale: Erbt das clicked-Signal von gtk::Button.
Verbundene Signale: Keine spezifischen.
2.6. Fehlerbehandlung
enum WorkspaceIndicatorError (definiert mit thiserror):

DomainCommunicationError(String): Fehler bei der Kommunikation mit domain::workspaces::WorkspaceManager.
ItemCreationFailed(String): Wenn ein WorkspaceItemWidget nicht erstellt werden kann.


Fehler werden primär geloggt (tracing). Eine fehlerhafte Aktualisierung könnte dazu führen, dass der Indikator veraltete Informationen anzeigt oder leer ist. Kritische Fehler bei der Initialisierung (z.B. Verbindung zum Domain-Service nicht möglich) könnten das Hinzufügen des Widgets zum Panel verhindern.
2.7. Implementierungsplan und Schritte
Definiere WorkspaceInfo Struktur.
Implementiere WorkspaceItemWidget (workspace_item_widget/mod.rs und imp.rs):

Subklasse von gtk::Button.
Implementiere new() und update_content().
Stelle sicher, dass Klick-Events korrekt weitergeleitet werden können (durch das clicked-Signal der Basisklasse).


Implementiere WorkspaceIndicatorWidget (workspace_indicator_widget/mod.rs und imp.rs):

Subklasse von gtk::Box.
Definiere GObject-Properties (max-visible-items, show-workspace-names).
Implementiere new(): Initialisiere internen Container, abonniere Workspace-Events.
Implementiere update_workspaces(): Logik zum Leeren und neu Befüllen des Containers mit WorkspaceItemWidgets.
Implementiere on_workspace_item_clicked(): Logik zum Auslösen des Workspace-Wechsels.
Implementiere handle_workspace_event(): Anforderung neuer Daten und Aufruf von update_workspaces.


Integration: Das PanelWidget instanziiert das WorkspaceIndicatorWidget und fügt es einem seiner Bereiche hinzu.
2.8. Dateistruktursrc/
└── ui/
    └── shell/
        └── panel_widget/
            └── workspace_indicator_widget/
                ├── mod.rs                      // Öffentliche API für WorkspaceIndicatorWidget
                ├── imp.rs                      // GObject-Implementierung für WorkspaceIndicatorWidget
                ├── types.rs                    // Definition von WorkspaceInfo
                ├── workspace_item_widget.rs    // Enthält mod.rs und imp.rs für WorkspaceItemWidget
                │   ├── mod.rs                  // Öffentliche API für WorkspaceItemWidget
                │   └── imp.rs                  // GObject-Implementierung für WorkspaceItemWidget
                └── error.rs                    // (Optional) Definition von WorkspaceIndicatorError
3. Modul: ui::shell::panel_widget::ClockDateTimeWidget3.1. Übersicht und VerantwortlichkeitenDas ClockDateTimeWidget ist ein Panel-Modul, das die aktuelle Systemzeit und das Datum anzeigt. Optional kann es bei einem Klick ein Popover mit einem Kalender anzeigen.
Anzeige der formatierten aktuellen Uhrzeit und des Datums.
Regelmäßige Aktualisierung der Anzeige.
Optionale Anzeige eines Kalender-Popovers bei Klick.
Konfigurierbarkeit des Zeit-/Datumsformats.
3.2. Visuelles Design und Theming
Anzeige: Textbasierte Anzeige der Zeit und des Datums.

Schriftart, -größe und -farbe werden durch das globale GTK-Theme und spezifische CSS-Regeln gesteuert.


Kalender-Popover: Ein Standard gtk::Popover, das ein gtk::Calendar-Widget enthält.6
CSS-Styling:

CSS-Knoten:

ClockDateTimeWidget (als gtk::Button oder gtk::Label): button oder label.
Kalender-Popover: popover.


CSS-Klassen:

.clock-datetime-widget: Für das Haupt-Widget.
.calendar-popover-content: Für den Inhalt des Kalender-Popovers (falls benutzerdefinierte Anpassungen nötig sind).




3.3. Datenstrukturen, Eigenschaften und Zustand

GObject-Definition (ClockDateTimeWidget):Erbend von gtk::Button (um Klickbarkeit für das Popover zu ermöglichen) oder gtk::Label (wenn kein Popover). Für die Popover-Funktionalität ist gtk::Button oder ein gtk::Box mit einem gtk::GestureClick geeigneter.
Auszug aus src/ui/shell/panel_widget/clock_datetime_widget/imp.rs (vereinfacht):
Rustuse gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::{Cell, RefCell};
use chrono::{DateTime, Local, SecondsFormat}; // [8]

#
pub struct ClockDateTimeWidget {
    // GObject Properties
    format_string: RefCell<String>,
    show_calendar_on_click: Cell<bool>,

    // Interner Zustand
    timer_id: RefCell<Option<glib::SourceId>>, // Für den Aktualisierungs-Timer [9, 10]
    calendar_popover: RefCell<Option<gtk::Popover>>,
    // Das Label-Widget, das die Zeit anzeigt (kann Kind des Buttons sein)
    time_label: TemplateChild<gtk::Label>,
}

#[glib::object_subclass]
impl ObjectSubclass for ClockDateTimeWidget {
    const NAME: &'static str = "NovaDEClockDateTimeWidget";
    type Type = super::ClockDateTimeWidget;
    // Erbt von gtk::Button, um Klickbarkeit und Popover-Ankerung zu erleichtern
    type ParentType = gtk::Button;

    fn new() -> Self {
        Self {
            format_string: RefCell::new("%H:%M".to_string()), // Standardformat [11]
            show_calendar_on_click: Cell::new(true),
            timer_id: RefCell::new(None),
            calendar_popover: RefCell::new(None),
            time_label: TemplateChild::default(),
        }
    }
    fn class_init(klass: &mut Self::Class) {
        // klass.bind_template(); // Falls.ui Datei verwendet wird
        // Properties hier installieren
        klass.set_css_name("clockdatetimewidget");
    }
    // fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
    //     obj.init_template();
    // }
}

impl ObjectImpl for ClockDateTimeWidget {
    fn constructed(&self) {
        self.parent_constructed();
        let obj = self.obj();

        // Initialisiere time_label, falls nicht über Template
        // self.time_label.set_use_markup(true); // Für Pango-Markup, falls benötigt
        // obj.set_child(Some(&self.time_label));

        obj.update_display(); // Erste Anzeige
        obj.setup_timer();    // Timer starten

        // Klick-Handler für Popover verbinden, falls show_calendar_on_click
        if self.show_calendar_on_click.get() {
            obj.connect_clicked(glib::clone!(@weak obj => move |_| {
                obj.show_calendar_popover();
            }));
        }
    }
    fn dispose(&self) {
        if let Some(source_id) = self.timer_id.borrow_mut().take() {
            source_id.remove();
        }
        if let Some(popover) = self.calendar_popover.borrow_mut().take() {
            popover.unparent(); // Sicherstellen, dass das Popover entfernt wird
        }
    }
    // Properties hier implementieren (set_property, property)
}
impl WidgetImpl for ClockDateTimeWidget {}
impl ButtonImpl for ClockDateTimeWidget {}



Eigenschaften (Properties) für ClockDateTimeWidget:
Tabelle 3.1: ClockDateTimeWidget Eigenschaften


EigenschaftsnameTypZugriffStandardwertBeschreibungformat-stringStringLesen/Schreiben"%H:%M"Der strftime-kompatible Format-String für die Zeit-/Datumsanzeige.11show-calendar-on-clickboolLesen/SchreibentrueWenn true, wird bei Klick auf das Widget ein Kalender-Popover angezeigt.
3.4. API und Methoden
pub fn new() -> Self

Signatur: pub fn new() -> Self
Verantwortlichkeit: Konstruktor. Initialisiert das Label, startet den Timer und verbindet ggf. den Klick-Handler für das Kalender-Popover.


fn update_display(&self) noexcept;

Signatur: fn update_display(&self) noexcept
Verantwortlichkeit: Ruft die aktuelle Zeit ab (chrono::Local::now() 8), formatiert sie gemäß format-string und aktualisiert den Text des internen gtk::Label (oder des Button-Labels).
Beispiel Formatierung: Local::now().format(self.imp().format_string.borrow().as_str()).to_string().11


fn setup_timer(&self) noexcept;

Signatur: fn setup_timer(&self) noexcept
Verantwortlichkeit: Startet einen Timer, der periodisch (z.B. jede Sekunde oder Minute, abhängig von der Granularität des format-string) update_display aufruft.
Verwendet glib::timeout_add_seconds_local oder glib::timeout_add_local für feinere Intervalle.9
Speichert die glib::SourceId im timer_id-Feld, um den Timer später entfernen zu können (im dispose-Handler).


fn show_calendar_popover(&self) noexcept;

Signatur: fn show_calendar_popover(&self) noexcept
Verantwortlichkeit: Erstellt (falls noch nicht geschehen) und zeigt ein gtk::Popover an, das ein gtk::Calendar-Widget enthält.6

Wenn self.imp().calendar_popover None ist:

Erstelle ein gtk::Calendar::new().
Erstelle ein gtk::Popover::builder().child(&calendar_widget).build().
Setze das Popover als Kind des ClockDateTimeWidget (oder eines spezifischen Ankerpunkts).
Speichere das Popover in self.imp().calendar_popover.


Setze das Popover relativ zum ClockDateTimeWidget (dem Button): popover.set_parent(self.obj().as_ref()).
Zeige das Popover: popover.popup().




3.5. Signale und Ereignisse
Emittierte Signale: Keine spezifischen benutzerdefinierten Signale. Erbt Signale von gtk::Button (z.B. clicked).
Verbundene Signale/Events:

Interner Timer-Event von glib::timeout_add_seconds_local.
clicked-Signal des Buttons (falls show-calendar-on-click true ist), um show_calendar_popover aufzurufen.
Lauscht auf Änderungen der relevanten Einstellungen von domain::global_settings_and_state_management (z.B. für format-string), um die GObject-Properties zu aktualisieren.


3.6. Fehlerbehandlung
enum ClockDateTimeError (definiert mit thiserror):

InvalidFormatString(String): Wenn der format-string ungültig ist und chrono::format fehlschlägt.
TimerSetupFailed(String): Wenn der glib-Timer nicht eingerichtet werden kann.


Fehler werden geloggt. Ein ungültiger Format-String könnte zu einer Standardanzeige oder einer Fehlermeldung im Widget führen.
3.7. Implementierungsplan und Schritte
Definiere ClockDateTimeWidget (clock_datetime_widget/mod.rs und imp.rs):

Subklasse von gtk::Button.
Definiere GObject-Properties (format-string, show-calendar-on-click).
Implementiere new(): Initialisiere das interne Label (als Kind des Buttons), rufe update_display() und setup_timer() auf. Verbinde clicked-Signal, falls show-calendar-on-click.


Implementiere update_display(): Hole aktuelle Zeit, formatiere sie, setze Label-Text.
Implementiere setup_timer(): Nutze glib::timeout_add_seconds_local (z.B. für sekündliche Updates), speichere SourceId. Der Callback ruft update_display() auf und gibt glib::ControlFlow::Continue zurück.
Implementiere show_calendar_popover(): Erstelle und zeige gtk::Popover mit gtk::Calendar.
Implementiere den dispose-Handler, um den Timer zu entfernen.
Integration: Das PanelWidget instanziiert das ClockDateTimeWidget und fügt es einem seiner Bereiche hinzu.
Verbindung mit domain::global_settings_and_state_management: Lausche auf Änderungen der Format-String-Einstellung und aktualisiere die format-string-Property des Widgets.
3.8. Dateistruktursrc/
└── ui/
    └── shell/
        └── panel_widget/
            └── clock_datetime_widget/
                ├── mod.rs          // Öffentliche API für ClockDateTimeWidget
                ├── imp.rs          // GObject-Implementierung
                └── error.rs        // (Optional) Definition von ClockDateTimeError
4. Schlussfolgerungen (für Teil 2)Die Spezifikationen für WorkspaceIndicatorWidget und ClockDateTimeWidget erweitern die Funktionalität des PanelWidget um zwei wichtige, benutzerorientierte Module. Die Implementierung des WorkspaceIndicatorWidget erfordert eine enge Kopplung mit domain::workspaces::WorkspaceManager, um stets aktuelle Informationen über die Arbeitsbereiche anzuzeigen und Interaktionen zu ermöglichen. Das ClockDateTimeWidget bietet eine grundlegende, aber essentielle Funktion mit optionaler Erweiterung durch ein Kalender-Popover.Die konsequente Anwendung von GObject-Subclassing, die Nutzung von GTK-Standardwidgets und -Containern sowie die klare Definition von Schnittstellen und Verantwortlichkeiten sind weiterhin zentrale Aspekte. Die Fehlerbehandlung und das Logging folgen den im ersten Teil etablierten Mustern.Mit diesen Modulen nimmt die Kernfunktionalität des Panels weiter Gestalt an. Die nächsten Schritte werden sich auf weitere Indikatoren und komplexere Panel-Module wie das SystemTrayEquivalentWidget konzentrieren.

# NovaDE: Ultra-Granulare Definition Interner Schnittstellen

## Präambel: Grundlegende Prinzipien

Die Architektur von NovaDE basiert auf einer Reihe fundamentaler Kommunikations- und Strukturprinzipien, die darauf abzielen, ein robustes, modulares und wartbares System zu schaffen. Diese Prinzipien sind entscheidend für das Erreichen der übergeordneten Projektziele wie Performance, intuitive Bedienung, Modernität, Modularität, Anpassbarkeit, sichere KI-Integration und Stabilität.

- **Kernprinzipien der Kommunikation:**
    - Die Interaktion zwischen den verschiedenen Schichten des Systems – Kern (Core), Domäne (Domain), System und Benutzeroberfläche (UI) – erfolgt primär über klar definierte öffentliche Programmierschnittstellen (APIs). Diese APIs werden in der Regel als Rust-Traits spezifiziert und von Service-Strukturen innerhalb der jeweiligen Schicht implementiert. Dieser Ansatz fördert ein vertragsbasiertes Interaktionsmodell, das die Kopplung zwischen den Schichten minimiert und die Austauschbarkeit von Implementierungen erleichtert.
    - Ein integraler Bestandteil der Architektur ist ein robustes, ereignisgesteuertes System. Dieses System dient der Benachrichtigung über Zustandsänderungen und ermöglicht entkoppelte Interaktionen zwischen den Schichten. Komponenten können auf relevante Ereignisse reagieren, ohne direkte Abhängigkeiten zu den Ereigniserzeugern aufbauen zu müssen. Dies ist besonders für die UI-Schicht von Bedeutung, da es reaktive Updates auf Änderungen im Domänenzustand oder Systemstatus ohne Polling-Mechanismen ermöglicht.
    - Eine strikte Schichtenarchitektur wird durchgesetzt: Höhere Schichten dürfen typischerweise nur von unmittelbar tieferliegenden Schichten abhängen. Dieses hierarchische Abhängigkeitsmodell verbessert die Modularität, Testbarkeit und Wartbarkeit, indem es zirkuläre Abhängigkeiten und unkontrollierte Interaktionen verhindert.
    - **Fehlerbehandlung:** Eine konsistente Fehlerbehandlungsstrategie wird über alle Schichten hinweg angewendet. Die `thiserror`-Bibliothek wird für die Definition von spezifischen, benutzerdefinierten Fehlertypen genutzt. `core::errors::CoreError` dient als fundamentaler Basistyp für Fehler. Es ist zwingend erforderlich, dass Fehlerketten (abrufbar über die `source()`-Methode) bei der Weitergabe oder beim Wrappen von Fehlern erhalten bleiben, um eine umfassende Fehlerdiagnose zu gewährleisten.

Die Kombination aus strikter Schichtung, einem API-First-Design (mittels Rust-Traits) und einem ereignisgesteuerten Ansatz bildet ein leistungsfähiges Fundament für NovaDE. Das Ereignissystem ist insbesondere für die UI-Schicht von entscheidender Bedeutung, da es ihr ermöglicht, reaktiv auf zugrundeliegende Änderungen im Domänenzustand oder Systemstatus zu reagieren, ohne auf ineffiziente Polling-Mechanismen zurückgreifen zu müssen. Beispielsweise kann die UI dynamisch aktualisiert werden, wenn ein `ThemeChangedEvent` vom `ThemingEngine` ausgelöst wird, anstatt periodisch den aktuellen Theme-Status abfragen zu müssen. Diese Entkopplung reduziert die Systemlast und verbessert die Reaktionsfähigkeit der Benutzeroberfläche. Die nachfolgende ultra-granulare Definition wird daher nicht nur die synchronen API-Methoden detailliert beschreiben, sondern auch die präzise Struktur, den Payload und die Semantik jedes Ereignisses, das zwischen den Schichten ausgetauscht wird. Dies stellt sicher, dass sowohl API-Konsumenten als auch Ereignis-Abonnenten ein klares und eindeutiges Verständnis der Daten haben, die sie senden oder empfangen werden.

---

**Teil 1: Ultra-Granulare Schnittstellendefinition der UI-Schicht**

Dieser Teil adressiert die primäre Anforderung einer detaillierten Schnittstellendefinition für die UI-Schicht und konzentriert sich darauf, wie diese mit der Domänen- und der Systemschicht interagiert. Die UI-Schicht ist für die Präsentation von Informationen für den Benutzer und die Übersetzung von Benutzereingaben in Aktionen auf den unteren Schichten verantwortlich.

- **1.1. Interaktionsprinzipien der UI-Schicht mit unteren Schichten**
    
    - **UI als Konsument von Domänenlogik und Systemdiensten:**
        - Die UI-Schicht agiert als primärer Interaktionspunkt mit dem Benutzer. Sie übersetzt Benutzereingaben (z.B. Klicks, Tastatureingaben, Gesten) in spezifische, wohldefinierte Aufrufe von Methoden, die von Diensten der Domänenschicht (für Geschäftslogik und Anwendungszustandsmanipulation) oder der Systemschicht (für Betriebssystem- oder Hardwareinteraktionen) bereitgestellt werden.
        - Ein entscheidender Aspekt ist, dass die UI-Schicht Ereignisse abonniert, die sowohl von der Domänen- als auch von der Systemschicht stammen. Diese Ereignisse signalisieren Zustandsänderungen oder das Auftreten bemerkenswerter Systemaktivitäten. Dies ermöglicht es der UI, ihre Darstellung dynamisch und reaktiv zu aktualisieren und so den aktuellen Zustand der Anwendung und des zugrundeliegenden Systems widerzuspiegeln.
        - Obwohl die interne Implementierung von UI-Komponenten stark auf einem Toolkit wie GTK4 basieren wird, werden die in diesem Dokument definierten Schnittstellen (zwischen UI und Domäne/System) in Form von reinen Rust-Typen, Traits und Ereignis-Strukturen spezifiziert. Diese Abstraktion stellt sicher, dass die Kernlogik von NovaDE nicht intrinsisch an ein spezifisches UI-Toolkit gebunden ist. Beispielsweise wird ein `core::types::Color` in API-Signaturen verwendet, nicht ein Toolkit-spezifischer Typ wie `gdk4::RGBA`.
    - **Zustandsverwaltung in der UI:**
        - Die UI-Schicht ist für die Verwaltung ihres eigenen Präsentationszustands verantwortlich (oft als ViewModels oder UI-spezifischer Zustand bezeichnet). Dieser Zustand wird von dem Zustand abgeleitet, der von der Domänen- und Systemschicht verwaltet wird, und muss mit diesem synchron gehalten werden.
        - Eine robuste und effiziente UI-Zustandsverwaltung ist für eine reaktionsschnelle und konsistente Benutzererfahrung von größter Bedeutung. Während die UI-Schicht ihre visuelle Darstellung und temporäre Interaktionszustände verwaltet, befindet sich die autoritative "Wahrheitsquelle" (Source of Truth) für geschäftskritische Daten innerhalb der Domänenschicht und für den Systemstatus innerhalb der Systemschicht. Die UI muss so aufgebaut sein, dass sie effizient auf Änderungen reagiert, die von diesen unteren Schichten signalisiert werden.
        - Die Benutzeranfrage liefert ein Beispiel: `ui::theming_gtk` konsumiert `AppliedThemeState` von `ThemingEngine`. Dieses Interaktionsmuster impliziert einen reaktiven Aktualisierungsmechanismus. Verschiedene Ansätze zur Zustandsverwaltung in `gtk-rs`-Anwendungen, einschließlich der Verwendung von `Rc<RefCell<T>>` für Single-Threaded Shared Mutability oder Mustern wie TEA (The Elm Architecture) oder Relm4, werden in der Forschung diskutiert.1 Für die geschichtete Architektur von NovaDE ist der am besten geeignete Ansatz, dass die UI-Schicht Domänen- und Systemereignisse abonniert. Nach dem Empfang eines Ereignisses aktualisiert die UI ihren lokalen Zustand (der GObject-Eigenschaften, Relm4-Modellfelder oder andere UI-Toolkit-spezifische Zustandscontainer sein können). Dieser ereignisgesteuerte Ansatz vermeidet eine enge Kopplung, bei der die Domänen- oder Systemschicht direkte Kenntnis von GTK-GObjects oder anderen UI-internen Details haben müsste.
        - Ereignisbehandlungsmechanismen innerhalb von GTK4 und verschiedene Zustandsverwaltungsmuster 1 sind für die interne Implementierung der UI-Schicht relevant. Dieses Schnittstellendefinitionsdokument konzentriert sich jedoch auf die Ereignisse, die die UI von unteren Schichten konsumiert, und die Daten-Payloads, die diese Ereignisse transportieren.
    - **Strategie zur Fehlerpräsentation:**
        - Die UI-Schicht trägt die Verantwortung für die Übersetzung von Fehlern, die von der Domänen- und Systemschicht empfangen werden, in benutzerfreundliche und kontextuell angemessene Benachrichtigungen, Dialoge oder visuelle Hinweise. Rohe technische Fehlerdetails sollten dem Endbenutzer nicht direkt präsentiert werden.
        - Intern könnte die UI `gtk::AlertDialog` 6 oder benutzerdefinierte Benachrichtigungs-Widgets (potenziell unter Verwendung von `gtk4-layer-shell` 9 für nicht-modale Popups) zur Darstellung dieser übersetzten Fehler verwenden. Diese Schnittstellendefinition wird spezifizieren, wie Fehlertypen (z.B. `ThemingError`, `CompositorError`) von unteren Schichten erwartet werden, von der UI behandelt und potenziell auf standardisierte benutzerorientierte Fehlerkategorien oder -nachrichten abgebildet zu werden.
        - Eine konsistente und verständliche Fehlerpräsentationsstrategie ist grundlegend für eine gute Benutzererfahrung. Die UI muss als Filter und Übersetzer fungieren und Benutzer vor komplexen oder kryptischen internen Fehlerdetails schützen. Die Benutzeranfrage spezifiziert, dass alle Schichten `thiserror`-basierte Fehler-Enums verwenden. Die UI-Schicht wird diese spezifischen Fehlertypen empfangen. Wenn beispielsweise der `WorkspaceManager` einen `WorkspaceManagerError::WorkspaceNotFound` zurückgibt, sollte die UI-Schicht diese rohe Enum-Variante nicht anzeigen. Stattdessen sollte sie dies in eine für Menschen lesbare Nachricht wie "Der ausgewählte Arbeitsbereich konnte nicht gefunden werden." übersetzen oder gegebenenfalls einen Dialog mit Wiederherstellungsoptionen anbieten. Diese Übersetzungs- und Präsentationslogik ist eine Kernaufgabe der UI-Schicht. Die Schnittstellendefinition sollte dies leiten, indem sie Fehlerkategorien vorschlägt, die die UI von jedem Dienst zu behandeln bereit sein sollte.
- 1.2. Schnittstelle der UI-Schicht mit der Domänenschicht
    
    Dieser Abschnitt definiert ultra-granular die Interaktionen der UI-Schicht mit jedem von der Domänenschicht bereitgestellten Dienst.
    
    - **1.2.1. `ThemingEngine`-API-Nutzung durch die UI:**
        
        - **Zweck:** Ermöglicht der UI, das aktuelle visuelle Thema der NovaDE-Umgebung abzufragen und zu ändern.
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `update_configuration`: `domain::theming::ThemeConfigurationUpdate { theme_name: Option<String>, variant_name: Option<String>, custom_color_palette: Option<HashMap<String, core::types::Color>> }`
            - Ausgabe von `get_current_theme_state`: `domain::theming::AppliedThemeState { theme_name: String, variant_name: String, primary_accent_color: core::types::Color, secondary_accent_color: core::types::Color, default_background_color: core::types::Color, default_text_color: core::types::Color, default_font_family: String, default_font_size: u16, border_radius: u8, shadow_intensity: f32 }`
            - Ausgabe von `get_available_themes`: `Vec<domain::theming::ThemeIdentifier { theme_id: String, display_name: String, available_variants: Vec<String>, preview_icon_path: Option<PathBuf>, author: Option<String>, description: Option<String> }>`
            - Ausgabe von `get_current_configuration`: `domain::theming::CurrentThemeConfiguration { theme_name: String, variant_name: String, custom_palette: HashMap<String, core::types::Color> }`
        - **Von der UI aufgerufene API-Methoden (aus `domain::ThemingEngine`-Trait):**
            - `fn get_current_theme_state(&self) -> Result<domain::theming::AppliedThemeState, domain::ThemingError>;`
            - `fn get_available_themes(&self) -> Result<Vec<domain::theming::ThemeIdentifier>, domain::ThemingError>;`
            - `fn get_current_configuration(&self) -> Result<domain::theming::CurrentThemeConfiguration, domain::ThemingError>;`
            - `fn update_configuration(&self, config_update: domain::theming::ThemeConfigurationUpdate) -> Result<(), domain::ThemingError>;`
            - `fn reload_themes_and_tokens(&self) -> Result<(), domain::ThemingError>;`
            - `// Die Ereignisabonnierung erfolgt über einen globalen EventBus oder eine dedizierte Abonnementmethode, falls eine feingranulare Steuerung erforderlich ist.`
        - **Von der UI konsumierte Event-Payloads:**
            - `domain::theming::ThemeChangedEvent { new_state: domain::theming::AppliedThemeState }`
        - **Fehlerbehandlung durch die UI:** Die UI muss `domain::ThemingError`-Varianten (z.B. `ThemeNotFound`, `InvalidConfiguration`, `AssetLoadFailure`) auf angemessene benutzerorientierte Nachrichten oder Wiederherstellungsaktionen abbilden.
    - **1.2.2. `WorkspaceManager`-API-Nutzung durch die UI:**
        
        - **Zweck:** Ermöglicht der UI die Verwaltung virtueller Arbeitsbereiche, einschließlich deren Erstellung, Löschung, Aktivierung und Zuweisung von Fenstern.
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `create_workspace`: `domain::workspaces::CreateWorkspaceParams { name: Option<String>, layout_hint: Option<domain::workspaces::WorkspaceLayoutType>, icon_name: Option<String> }`
            - Eingabe für `assign_window_to_workspace`: `window_id: core::types::WindowId`, `workspace_id: core::types::WorkspaceId`
            - Eingabe für `rename_workspace`: `workspace_id: core::types::WorkspaceId`, `new_name: String`
            - Ausgabe von `get_active_workspace`, `get_workspace_by_id`, `list_workspaces`: `domain::workspaces::WorkspaceDescriptor { id: core::types::WorkspaceId, name: String, display_name: String, icon_name: Option<String>, layout: domain::workspaces::WorkspaceLayoutType, window_ids: Vec<core::types::WindowId>, is_active: bool, creation_timestamp: core::types::DateTime<Utc> }`
            - Enum `domain::workspaces::WorkspaceLayoutType { TilingHorizontal, TilingVertical, Floating, Maximized }`
        - **Von der UI aufgerufene API-Methoden (aus `domain::WorkspaceManager`-Trait):**
            - `fn create_workspace(&self, params: domain::workspaces::CreateWorkspaceParams) -> Result<core::types::WorkspaceId, domain::WorkspaceManagerError>;`
            - `fn delete_workspace(&self, id: core::types::WorkspaceId) -> Result<(), domain::WorkspaceManagerError>;`
            - `fn set_active_workspace(&self, id: core::types::WorkspaceId) -> Result<(), domain::WorkspaceManagerError>;`
            - `fn get_active_workspace(&self) -> Result<Option<domain::workspaces::WorkspaceDescriptor>, domain::WorkspaceManagerError>;`
            - `fn get_workspace_by_id(&self, id: core::types::WorkspaceId) -> Result<Option<domain::workspaces::WorkspaceDescriptor>, domain::WorkspaceManagerError>;`
            - `fn list_workspaces(&self) -> Result<Vec<domain::workspaces::WorkspaceDescriptor>, domain::WorkspaceManagerError>;`
            - `fn assign_window_to_workspace(&self, window_id: core::types::WindowId, workspace_id: core::types::WorkspaceId) -> Result<(), domain::WorkspaceManagerError>;`
            - `fn remove_window_from_workspace(&self, window_id: core::types::WindowId, workspace_id: core::types::WorkspaceId) -> Result<(), domain::WorkspaceManagerError>;`
            - `fn get_window_assignment(&self, window_id: core::types::WindowId) -> Result<Option<core::types::WorkspaceId>, domain::WorkspaceManagerError>;`
            - `fn save_workspace_configuration(&self) -> Result<(), domain::WorkspaceManagerError>;`
            - `fn rename_workspace(&self, workspace_id: core::types::WorkspaceId, new_name: String) -> Result<(), domain::WorkspaceManagerError>;`
            - `fn set_workspace_layout(&self, workspace_id: core::types::WorkspaceId, layout: domain::workspaces::WorkspaceLayoutType) -> Result<(), domain::WorkspaceManagerError>;`
        - **Von der UI konsumierte Event-Payloads:**
            - `domain::workspaces::WorkspaceCreatedEvent { descriptor: domain::workspaces::WorkspaceDescriptor }`
            - `domain::workspaces::WorkspaceDeletedEvent { workspace_id: core::types::WorkspaceId, previous_name: String }`
            - `domain::workspaces::ActiveWorkspaceChangedEvent { old_workspace_id: Option<core::types::WorkspaceId>, new_workspace_id: core::types::WorkspaceId }`
            - `domain::workspaces::WorkspaceLayoutChangedEvent { workspace_id: core::types::WorkspaceId, new_layout: domain::workspaces::WorkspaceLayoutType }`
            - `domain::workspaces::WindowAssignedToWorkspaceEvent { window_id: core::types::WindowId, new_workspace_id: core::types::WorkspaceId, old_workspace_id: Option<core::types::WorkspaceId> }`
            - `domain::workspaces::WindowRemovedFromWorkspaceEvent { window_id: core::types::WindowId, workspace_id: core::types::WorkspaceId }`
            - `domain::workspaces::WorkspaceRenamedEvent { workspace_id: core::types::WorkspaceId, new_name: String, old_name: String }`
            - `domain::workspaces::WorkspaceOrderChangedEvent { ordered_workspace_ids: Vec<core::types::WorkspaceId> }`
        - **Fehlerbehandlung durch die UI:** Abbildung von `domain::WorkspaceManagerError`-Varianten (z.B. `WorkspaceNotFound`, `WindowNotManaged`, `NameConflict`) auf Benutzerfeedback.
    - **1.2.3. `AIInteractionLogicService`-API-Nutzung durch die UI:**
        
        - **Zweck:** Ermöglicht der UI, KI-gestützte Interaktionen zu initiieren, Kontextinformationen abzurufen und Benutzereinwilligungen zu verwalten.
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `initiate_interaction`: `domain::ai::InteractionRequest { prompt_text: String, context_type: domain::ai::AIContextType, attachments: Option<Vec<domain::ai::AIAttachment>>, preferred_model_id: Option<String> }`
            - Enum `domain::ai::AIContextType { GeneralText, CodeGeneration, ImageAnalysis, /*... */ }`
            - Struct `domain::ai::AIAttachment { mime_type: String, data_uri: String /* or PathBuf if local */, name: Option<String> }`
            - Ausgabe von `initiate_interaction`: `domain::ai::InteractionResponseHandle { interaction_id: core::types::Uuid }` (Weitere Ergebnisse/Streams könnten über Events oder eine Streaming-API kommen)
            - Ausgabe von `get_interaction_context`: `domain::ai::InteractionContext { interaction_id: core::types::Uuid, status: domain::ai::AIInteractionStatus, history: Vec<domain::ai::AIMessage>, available_actions: Vec<domain::ai::AIActionSuggestion> }`
            - Enum `domain::ai::AIInteractionStatus { PendingInput, Processing, AwaitingConsent, Completed, Error }`
            - Struct `domain::ai::AIMessage { role: domain::ai::AIRole, content: String, timestamp: core::types::DateTime<Utc> }`
            - Enum `domain::ai::AIRole { User, Assistant, System }`
            - Struct `domain::ai::AIActionSuggestion { action_id: String, display_text: String, action_type: domain::ai::AIActionType }`
            - Enum `domain::ai::AIActionType { ExecuteCommand, InsertText, OpenLink }`
            - Eingabe für `provide_consent`: `interaction_id: core::types::Uuid, model_id: String, consent_granted: bool`
            - Ausgabe von `get_consent_for_model`: `domain::ai::AIConsentStatus { model_id: String, granted: bool, last_updated: core::types::DateTime<Utc> }`
            - Struct `domain::ai::AIModelProfile { model_id: String, display_name: String, provider: String, capabilities: Vec<domain::ai::AIContextType>, requires_consent: bool }`
        - **Von der UI aufgerufene API-Methoden (aus `domain::AIInteractionLogicService`-Trait):**
            - `async fn initiate_interaction(&self, request: domain::ai::InteractionRequest) -> Result<domain::ai::InteractionResponseHandle, domain::AIInteractionError>;`
            - `async fn get_interaction_context(&self, interaction_id: core::types::Uuid) -> Result<domain::ai::InteractionContext, domain::AIInteractionError>;`
            - `async fn provide_consent(&self, interaction_id: core::types::Uuid, model_id: String, consent_granted: bool) -> Result<(), domain::AIInteractionError>;`
            - `async fn get_consent_for_model(&self, model_id: String) -> Result<domain::ai::AIConsentStatus, domain::AIInteractionError>;`
            - `async fn store_consent(&self, model_id: String, granted: bool) -> Result<(), domain::AIInteractionError>;`
            - `async fn load_model_profiles(&self) -> Result<Vec<domain::ai::AIModelProfile>, domain::AIInteractionError>;`
            - `async fn send_message_to_interaction(&self, interaction_id: core::types::Uuid, message: domain::ai::AIMessage) -> Result<(), domain::AIInteractionError>;`
        - **Von der UI konsumierte Event-Payloads:**
            - `domain::ai::AIInteractionInitiatedEvent { interaction_id: core::types::Uuid, initial_prompt: String }`
            - `domain::ai::AIConsentUpdatedEvent { model_id: String, new_status: domain::ai::AIConsentStatus }`
            - `domain::ai::AIInteractionStatusChangedEvent { interaction_id: core::types::Uuid, new_status: domain::ai::AIInteractionStatus, details: Option<String> }`
            - `domain::ai::NewAIMessageEvent { interaction_id: core::types::Uuid, message: domain::ai::AIMessage }`
            - `domain::ai::AIInteractionErrorEvent { interaction_id: Option<core::types::Uuid>, error_message: String, is_fatal: bool }`
        - **Fehlerbehandlung durch die UI:** Abbildung von `domain::AIInteractionError`-Varianten (z.B. `ModelUnavailable`, `ConsentRequired`, `NetworkError`) auf Benutzerfeedback.
    - **1.2.4. `NotificationService`-API-Nutzung durch die UI (primär für das Posten von UI-generierten Benachrichtigungen):**
        
        - **Zweck:** Ermöglicht der UI, anwendungsspezifische Benachrichtigungen zu erstellen und anzuzeigen, sowie bestehende Benachrichtigungen zu verwalten.
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `post_notification`: `domain::notifications::NotificationData { app_name: String, app_icon: Option<String>, summary: String, body: Option<String>, actions: Vec<domain::notifications::NotificationAction>, urgency: domain::notifications::NotificationUrgency, timeout_ms: Option<u32>, category: Option<String>, transient: bool }`
            - Struct `domain::notifications::NotificationAction { id: String, display_text: String }`
            - Enum `domain::notifications::NotificationUrgency { Low, Normal, Critical }`
            - Ausgabe von `post_notification`: `notification_id: core::types::Uuid`
            - Ausgabe von `get_notification`, `list_notifications`: `domain::notifications::Notification { id: core::types::Uuid, data: domain::notifications::NotificationData, timestamp: core::types::DateTime<Utc>, is_read: bool, is_dismissed: bool }`
        - **Von der UI aufgerufene API-Methoden (aus `domain::NotificationService`-Trait):**
            - `fn post_notification(&self, notification_data: domain::notifications::NotificationData) -> Result<core::types::Uuid, domain::NotificationError>;`
            - `fn get_notification(&self, id: core::types::Uuid) -> Result<Option<domain::notifications::Notification>, domain::NotificationError>;`
            - `fn list_notifications(&self, filter: Option<domain::notifications::NotificationFilter>) -> Result<Vec<domain::notifications::Notification>, domain::NotificationError>;` // Filter könnte `unread_only`, `app_name` etc. sein
            - `fn mark_as_read(&self, id: core::types::Uuid) -> Result<(), domain::NotificationError>;`
            - `fn dismiss_notification(&self, id: core::types::Uuid) -> Result<(), domain::NotificationError>;`
            - `fn invoke_action_on_notification(&self, notification_id: core::types::Uuid, action_id: String) -> Result<(), domain::NotificationError>;`
            - `fn set_do_not_disturb(&self, enabled: bool) -> Result<(), domain::NotificationError>;`
            - `fn get_do_not_disturb_status(&self) -> Result<bool, domain::NotificationError>;`
        - **Von der UI konsumierte Event-Payloads:**
            - `domain::notifications::NotificationPostedEvent { notification: domain::notifications::Notification }`
            - `domain::notifications::NotificationDismissedEvent { notification_id: core::types::Uuid, reason: domain::notifications::DismissReason }`
            - `domain::notifications::NotificationActionInvokedEvent { notification_id: core::types::Uuid, action_id: String }`
            - `domain::notifications::DoNotDisturbChangedEvent { is_enabled: bool }`
            - Enum `domain::notifications::DismissReason { User, Timeout, Programmatic }`
        - **Fehlerbehandlung durch die UI:** Abbildung von `domain::NotificationError`-Varianten (z.B. `NotificationNotFound`, `ActionNotFound`) auf Benutzerfeedback.
    - **1.2.5. `GlobalSettingsService`-API-Nutzung durch die UI:**
        
        - **Zweck:** Ermöglicht der UI, globale Anwendungseinstellungen zu lesen, zu ändern und auf Standardwerte zurückzusetzen.
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `update_setting`: `key: String`, `value: glib::Variant`
            - Ausgabe von `get_setting`: `Result<Option<glib::Variant>, domain::GlobalSettingsError>`
            - Ausgabe von `get_current_settings`: `Result<HashMap<String, glib::Variant>, domain::GlobalSettingsError>`
            - Struct `domain::settings::SettingDescriptor { key: String, display_name: String, description: Option<String>, value_type: glib::VariantType, current_value: glib::Variant, default_value: glib::Variant, possible_values: Option<Vec<glib::Variant>>, range: Option<(glib::Variant, glib::Variant)>, category: String }`
        - **Von der UI aufgerufene API-Methoden (aus `domain::GlobalSettingsService`-Trait):**
            - `fn load_settings(&self) -> Result<(), domain::GlobalSettingsError>;`
            - `fn save_settings(&self) -> Result<(), domain::GlobalSettingsError>;`
            - `fn get_current_settings(&self) -> Result<HashMap<String, glib::Variant>, domain::GlobalSettingsError>;`
            - `fn get_setting_descriptors(&self) -> Result<Vec<domain::settings::SettingDescriptor>, domain::GlobalSettingsError>;`
            - `fn update_setting(&self, key: String, value: glib::Variant) -> Result<(), domain::GlobalSettingsError>;`
            - `fn get_setting(&self, key: String) -> Result<Option<glib::Variant>, domain::GlobalSettingsError>;`
            - `fn reset_setting_to_default(&self, key: String) -> Result<(), domain::GlobalSettingsError>;`
            - `fn reset_all_settings_to_defaults(&self) -> Result<(), domain::GlobalSettingsError>;`
        - **Von der UI konsumierte Event-Payloads:**
            - `domain::settings::SettingChangedEvent { key: String, new_value: glib::Variant, old_value: glib::Variant }`
            - `domain::settings::SettingsLoadedEvent { settings: HashMap<String, glib::Variant> }`
            - `domain::settings::SettingsSavedEvent {}`
        - **Fehlerbehandlung durch die UI:** Abbildung von `domain::GlobalSettingsError`-Varianten (z.B. `SettingNotFound`, `InvalidValueType`, `PersistenceError`) auf Benutzerfeedback. Die Verwendung von `glib::Variant` für Einstellwerte 12 ermöglicht eine flexible Handhabung verschiedener Datentypen für Einstellungen.
    - **Schlüsseltabellen für Teil 1.2:**
        
        Für jede Methode der Domänendienst-API:
        
        1. **Methodensignatur-Tabelle:**

|   |   |   |   |
|---|---|---|---|
|**Methode**|**Parameter**|**Rückgabetyp**|**Kurzbeschreibung**|
|`ThemingEngine::get_current_theme_state`|`&self`|`Result<domain::theming::AppliedThemeState, domain::ThemingError>`|Ruft den aktuell angewendeten Theme-Zustand ab.|
|`WorkspaceManager::create_workspace`|`&self, params: domain::workspaces::CreateWorkspaceParams`|`Result<core::types::WorkspaceId, domain::WorkspaceManagerError>`|Erstellt einen neuen Arbeitsbereich.|
|...|...|...|...|

```
    2.  **Parameterdetail-Tabelle (Beispiel für `ThemingEngine::update_configuration`):**
```

|   |   |   |   |
|---|---|---|---|
|**Parametername**|**Typ**|**Erforderlich**|**Beschreibung**|
|`config_update`|`domain::theming::ThemeConfigurationUpdate`|Ja|Ein Objekt, das die gewünschten Änderungen an der Theme-Konfiguration enthält.|

```
    3.  **Rückgabewertdetail-Tabelle (Beispiel für `ThemingEngine::get_current_theme_state`):**
```

|   |   |   |
|---|---|---|
|**Erfolgsfall (Ok)**|**Fehlerfall (Err)**|**Beschreibung**|
|`domain::theming::AppliedThemeState`|`domain::ThemingError`|Enthält Details zum aktuell angewendeten Theme oder einen Fehler bei der Abfrage.|

```
    Für jedes von der UI konsumierte Domänenereignis:
    4.  **Ereignis-Payload-Detail-Tabelle (Beispiel für `ThemeChangedEvent`):**
```

|   |   |   |
|---|---|---|
|**Feldname**|**Typ**|**Beschreibung**|
|`new_state`|`domain::theming::AppliedThemeState`|Der neue Zustand des Themes nach der Änderung.|

```
    Diese Tabellen sind unerlässlich, um Entwicklern eine eindeutige und leicht zugängliche Referenz zu bieten. Sie erzwingen Konsistenz bei der Typverwendung und Methodenaufrufen und reduzieren so Integrationsfehler zwischen der UI- und der Domänenschicht erheblich. Beispielsweise beschreibt eine detaillierte Tabelle für `WorkspaceDescriptor` klar alle Informationen, die die UI über einen Arbeitsbereich erwarten kann, was die Entwicklung von UI-Komponenten wie Arbeitsbereichswechslern erleichtert.
```

- 1.3. Schnittstelle der UI-Schicht mit der Systemschicht
    
    Dieser Abschnitt definiert ultra-granular die Interaktionen der UI-Schicht mit Diensten, die von der Systemschicht bereitgestellt werden. Die Systemschicht abstrahiert Details des zugrundeliegenden Betriebssystems, der Hardware und des Fenstersystems (Wayland/X11).
    
    - **1.3.1. Compositor-Interaktion (`system::compositor_iface`) durch die UI:**
        
        - **Zweck:** Ermöglicht der UI, Fensterinformationen abzufragen, Fensterzustände (Fokus, Geometrie) zu verwalten und Aktualisierungen über den Lebenszyklus und die Eigenschaften von Fenstern zu erhalten. Diese Schnittstelle abstrahiert die zugrundeliegenden Wayland-Compositor-Interaktionen. Die UI interagiert nicht direkt mit Wayland-Protokollen wie `wlr-foreign-toplevel-management` 17 oder `ext-foreign-toplevel-list-v1` 17; stattdessen konsumiert sie die von der Systemschicht bereitgestellten Abstraktionen.
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `request_focus`: `window_id: core::types::WindowId`
            - Eingabe für `configure_window`: `window_id: core::types::WindowId`, `config: system::compositor::WindowConfigurationRequest { new_position: Option<core::types::PointInt>, new_size: Option<core::types::SizeInt>, new_state: Option<system::compositor::CompositorWindowState>, new_workspace_id: Option<core::types::WorkspaceId> }`
            - Enum `system::compositor::CompositorWindowState { Maximized, Minimized, Fullscreen, Normal, TiledLeft, TiledRight }`
            - Ausgabe von `get_window_info`, `list_managed_windows`: `system::compositor::WindowInfo { id: core::types::WindowId, title: String, app_id: core::types::AppId, class: Option<String>, geometry: core::types::RectInt, is_focused: bool, is_maximized: bool, is_minimized: bool, is_fullscreen: bool, workspace_id: Option<core::types::WorkspaceId>, can_resize: bool, can_maximize: bool, can_minimize: bool, can_close: bool, parent_window_id: Option<core::types::WindowId>, transient_for: Option<core::types::WindowId> }`
            - Ausgabe von `get_output_info`, `list_outputs`: `system::compositor::OutputInfo { id: String, name: String, description: String, geometry: core::types::RectInt, scale_factor: f32, refresh_rate_mhz: u32, current_resolution: core::types::SizeInt, available_resolutions: Vec<core::types::SizeInt>, is_primary: bool, is_enabled: bool }`
        - **Von der UI aufgerufene API-Methoden (aus `system::CompositorInterface`-Trait):**
            - `fn request_focus(&self, window_id: core::types::WindowId) -> Result<(), system::CompositorError>;`
            - `fn configure_window(&self, window_id: core::types::WindowId, config: system::compositor::WindowConfigurationRequest) -> Result<(), system::CompositorError>;`
            - `fn get_window_info(&self, window_id: core::types::WindowId) -> Result<Option<system::compositor::WindowInfo>, system::CompositorError>;`
            - `fn list_managed_windows(&self) -> Result<Vec<system::compositor::WindowInfo>, system::CompositorError>;`
            - `fn close_window(&self, window_id: core::types::WindowId) -> Result<(), system::CompositorError>;`
            - `fn list_outputs(&self) -> Result<Vec<system::compositor::OutputInfo>, system::CompositorError>;`
            - `fn get_output_info(&self, output_id: String) -> Result<Option<system::compositor::OutputInfo>, system::CompositorError>;`
            - `fn get_cursor_position(&self) -> Result<core::types::PointInt, system::CompositorError>;`
        - **Von der UI konsumierte Event-Payloads:**
            - `system::compositor::WindowCreatedEvent { info: system::compositor::WindowInfo }`
            - `system::compositor::WindowClosedEvent { window_id: core::types::WindowId }`
            - `system::compositor::WindowFocusedEvent { window_id: core::types::WindowId }`
            - `system::compositor::WindowUnfocusedEvent { window_id: core::types::WindowId }`
            - `system::compositor::WindowPropertiesChangedEvent { window_id: core::types::WindowId, changed_title: Option<String>, changed_app_id: Option<core::types::AppId>, changed_geometry: Option<core::types::RectInt>, changed_state: Option<system::compositor::CompositorWindowState> }`
            - `system::compositor::OutputAddedEvent { info: system::compositor::OutputInfo }`
            - `system::compositor::OutputRemovedEvent { output_id: String }`
            - `system::compositor::OutputConfigurationChangedEvent { info: system::compositor::OutputInfo }`
            - `system::compositor::CursorPositionChangedEvent { position: core::types::PointInt }`
        - **Fehlerbehandlung durch die UI:** Abbildung von `system::CompositorError`-Varianten (z.B. `WindowNotFound`, `WaylandProtocolError`, `InvalidConfiguration`) auf Benutzerfeedback.
    - **1.3.2. Eingabeverarbeitung (`system::input_iface`) durch die UI:**
        
        - **Zweck:** Liefert der UI verarbeitete Eingabeereignisse (Tastatur, Maus, Touch, Gesten) und ermöglicht es der UI, den Fokus zu steuern.
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `request_keyboard_focus`: `window_id: core::types::WindowId` (kann auch über `compositor_iface` laufen, hier zur Verdeutlichung der Input-Intention)
            - Event-Payloads für `KeyboardEvent`, `PointerEvent`, `TouchEvent`, `GestureEvent` (detaillierte Definitionen erforderlich, z.B. `KeyEvent { key_code: u32, modifiers: KeyModifiers, state: KeyState, utf8: Option<String> }`).
        - **Von der UI aufgerufene API-Methoden (aus `system::InputInterface`-Trait):**
            - `fn request_keyboard_focus(&self, window_id: core::types::WindowId) -> Result<(), system::InputError>;` (kann Duplikat zu Compositor-API sein, aber semantisch hier relevant)
            - `fn set_cursor_theme(&self, theme_name: String, size: u32) -> Result<(), system::InputError>;`
            - `fn get_current_keyboard_layout(&self) -> Result<String, system::InputError>;`
        - **Von der UI konsumierte Event-Payloads:**
            - `system::input::KeyboardEvent { window_id: Option<core::types::WindowId>, /*... event details... */ }`
            - `system::input::PointerButtonEvent { window_id: Option<core::types::WindowId>, /*... event details... */ }`
            - `system::input::PointerMotionEvent { window_id: Option<core::types::WindowId>, /*... event details... */ }`
            - `system::input::PointerAxisEvent { window_id: Option<core::types::WindowId>, /*... event details... */ }`
            - `system::input::TouchEvent { window_id: Option<core::types::WindowId>, /*... event details... */ }`
            - `system::input::GestureEvent { window_id: Option<core::types::WindowId>, gesture_type: system::input::GestureType, /*... event details... */ }`
            - `system::input::FocusChangedEvent { new_focus_window_id: Option<core::types::WindowId>, old_focus_window_id: Option<core::types::WindowId> }` (kann auch vom Compositor kommen)
            - `system::input::KeyboardLayoutChangedEvent { new_layout_name: String }`
        - **Fehlerbehandlung durch die UI:** Abbildung von `system::InputError`-Varianten.
    - **1.3.3. D-Bus Client-Schnittstellen (`system::dbus::*`) Nutzung durch die UI:**
        
        - **Allgemein:** Die UI interagiert mit D-Bus-Diensten über typsichere Rust-Wrapper, die von der Systemschicht bereitgestellt werden. Die UI konsumiert Ereignisse und ruft Methoden dieser Wrapper auf. Die Komplexität der D-Bus-Kommunikation (z.B. mit `zbus` 19 oder `gio::DBusConnection` 22) wird von der Systemschicht gekapselt.
        - **`system::dbus::upower_client_iface`:**
            - Events: `system::dbus::UPowerDeviceChangedEvent { device_id: String, percentage: f64, state: UPowerDeviceState, time_to_empty_s: Option<u64>, time_to_full_s: Option<u64> }`
        - **`system::dbus::logind_client_iface`:**
            - Methoden: `fn lock_session(&self) -> Result<(), system::DBusError>;`
            - Events: `system::dbus::LogindSessionEvent { event_type: LogindEventType /* Lock, Unlock, Sleep, Wakeup */ }`
        - **`system::dbus::networkmanager_client_iface`:**
            - Methoden: `fn list_access_points(&self, device_path: String) -> Result<Vec<system::dbus::AccessPointInfo>, system::DBusError>;`, `fn connect_to_access_point(&self, device_path: String, ap_bssid: String, password: Option<String>) -> Result<(), system::DBusError>;`
            - Datenstrukturen: `AccessPointInfo { bssid: String, ssid: String, strength: u8, security_flags: u32, is_active: bool }` (basierend auf NetworkManager D-Bus API 23)
            - Events: `system::dbus::NetworkStateChangedEvent { connectivity: NetworkConnectivityState, primary_connection_type: Option<String> }`, `system::dbus::AccessPointsChangedEvent { device_path: String }`
        - **`system::dbus::secrets_client_iface`:**
            - Methoden: `async fn prompt_for_secret(&self, prompt_message: String) -> Result<Option<String>, system::DBusError>;` (interagiert mit UI für Prompts)
    - **1.3.4. Output-Management (`system::outputs_iface`) durch die UI:**
        
        - **Zweck:** Ermöglicht der UI, Informationen über angeschlossene Displays abzurufen und deren Konfiguration (Auflösung, Skalierung, Position) zu ändern.
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `configure_output`: `output_id: String`, `config: system::outputs::OutputConfigurationRequest { resolution: Option<core::types::SizeInt>, scale: Option<f32>, position: Option<core::types::PointInt>, enabled: Option<bool>, refresh_rate_mhz: Option<u32> }`
            - Ausgabe von `list_outputs`, `get_output_details`: `system::outputs::OutputDetails` (ähnlich `system::compositor::OutputInfo`, aber ggf. mit mehr Konfigurationsdetails wie verfügbare Modi, aktuelle Skalierung, etc.)
        - **Von der UI aufgerufene API-Methoden (aus `system::OutputInterface`-Trait):**
            - `fn list_outputs(&self) -> Result<Vec<system::outputs::OutputDetails>, system::OutputError>;`
            - `fn get_output_details(&self, output_id: String) -> Result<Option<system::outputs::OutputDetails>, system::OutputError>;`
            - `fn configure_output(&self, output_id: String, config: system::outputs::OutputConfigurationRequest) -> Result<(), system::OutputError>;`
            - `fn set_primary_output(&self, output_id: String) -> Result<(), system::OutputError>;`
        - **Von der UI konsumierte Event-Payloads:**
            - `system::outputs::OutputConfigurationChangedEvent { output_id: String, new_details: system::outputs::OutputDetails }` (kann auch vom Compositor kommen, hier ggf. spezifischer für Output-Management-Aktionen)
            - `system::outputs::OutputConnectedEvent { details: system::outputs::OutputDetails }`
            - `system::outputs::OutputDisconnectedEvent { output_id: String }`
        - **Fehlerbehandlung durch die UI:** Abbildung von `system::OutputError`-Varianten.
    - **1.3.5. Audio-Management (`system::audio_iface`) durch die UI:**
        
        - **Zweck:** Ermöglicht der UI, Audio-Geräte und -Streams zu verwalten, Lautstärken zu regeln und Audio-Ereignisse zu empfangen. Abstrahiert PipeWire-Interaktionen.26
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `set_volume`: `device_id: String`, `stream_id: Option<String>`, `volume_percent: f32`, `is_muted: Option<bool>`
            - Ausgabe von `list_audio_devices`, `get_audio_device_details`: `system::audio::AudioDeviceDetails { id: String, name: String, device_type: AudioDeviceType /* Input, Output */, current_volume_percent: f32, is_muted: bool, available_ports: Vec<String>, active_port: Option<String> }`
            - Ausgabe von `list_audio_streams`: `system::audio::AudioStreamDetails { id: String, application_name: String, media_role: String, current_volume_percent: f32, is_muted: bool }`
        - **Von der UI aufgerufene API-Methoden (aus `system::AudioInterface`-Trait):**
            - `fn list_audio_devices(&self, device_type_filter: Option<system::audio::AudioDeviceType>) -> Result<Vec<system::audio::AudioDeviceDetails>, system::AudioError>;`
            - `fn get_audio_device_details(&self, device_id: String) -> Result<Option<system::audio::AudioDeviceDetails>, system::AudioError>;`
            - `fn set_device_volume(&self, device_id: String, volume_percent: f32) -> Result<(), system::AudioError>;`
            - `fn set_device_mute(&self, device_id: String, muted: bool) -> Result<(), system::AudioError>;`
            - `fn set_default_output_device(&self, device_id: String) -> Result<(), system::AudioError>;`
            - `fn set_default_input_device(&self, device_id: String) -> Result<(), system::AudioError>;`
            - `fn list_audio_streams(&self) -> Result<Vec<system::audio::AudioStreamDetails>, system::AudioError>;`
            - `fn set_stream_volume(&self, stream_id: String, volume_percent: f32) -> Result<(), system::AudioError>;`
            - `fn set_stream_mute(&self, stream_id: String, muted: bool) -> Result<(), system::AudioError>;`
        - **Von der UI konsumierte Event-Payloads:**
            - `system::audio::AudioDeviceChangedEvent { device_details: system::audio::AudioDeviceDetails }`
            - `system::audio::AudioDeviceAddedEvent { device_details: system::audio::AudioDeviceDetails }`
            - `system::audio::AudioDeviceRemovedEvent { device_id: String }`
            - `system::audio::AudioStreamAddedEvent { stream_details: system::audio::AudioStreamDetails }`
            - `system::audio::AudioStreamRemovedEvent { stream_id: String }`
            - `system::audio::AudioStreamVolumeChangedEvent { stream_id: String, new_volume_percent: f32, is_muted: bool }`
            - `system::audio::DefaultDeviceChangedEvent { device_type: system::audio::AudioDeviceType, new_default_device_id: String }`
        - **Fehlerbehandlung durch die UI:** Abbildung von `system::AudioError`-Varianten.
    - **1.3.6. MCP-Client (`system::mcp_client_iface`) Nutzung durch die UI:**
        
        - **Zweck:** Stellt der UI KI-Funktionen zur Verfügung, z.B. für die Befehlspalette oder andere intelligente UI-Elemente.
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `request_ai_completion`: `prompt: String`, `context_data: HashMap<String, glib::Variant>`, `max_tokens: Option<u32>`
            - Ausgabe von `request_ai_completion`: `system::mcp::AICompletionResponse { completion_id: core::types::Uuid, text_result: Option<String>, structured_result: Option<glib::Variant>, error_message: Option<String> }`
        - **Von der UI aufgerufene API-Methoden (aus `system::MCPClientInterface`-Trait):**
            - `async fn request_ai_completion(&self, prompt: String, context_data: HashMap<String, glib::Variant>, options: system::mcp::AICompletionOptions) -> Result<system::mcp::AICompletionResponse, system::MCPError>;`
            - `async fn get_ai_capabilities(&self) -> Result<Vec<system::mcp::AICapabilityDescriptor>, system::MCPError>;`
        - **Von der UI konsumierte Event-Payloads:**
            - `system::mcp::AIModelStatusChangedEvent { model_id: String, is_available: bool }`
        - **Fehlerbehandlung durch die UI:** Abbildung von `system::MCPError`-Varianten.
    - **1.3.7. XDG Desktop Portals (`system::portals_iface`) Nutzung durch die UI:**
        
        - **Zweck:** Ermöglicht der UI die Interaktion mit XDG Desktop Portals für Aktionen wie Dateiauswahl, Screenshots, etc., auf eine sandkastenfreundliche Weise. Die Systemschicht abstrahiert die direkte D-Bus-Kommunikation mit den Portal-Diensten. Die `ashpd`-Bibliothek 28 ist ein gutes Beispiel für eine solche Abstraktionsebene, die die Systemschicht intern nutzen könnte.
        - **Ausgetauschte Datenstrukturen (Typen):**
            - Eingabe für `open_file_dialog`: `parent_window_identifier: system::portals::WindowIdentifier`, `title: String`, `options: system::portals::OpenFileOptions { multiple: bool, directory: bool, filters: Vec<system::portals::FileFilter> }`
            - Struct `system::portals::FileFilter { name: String, patterns: Vec<String> /* z.B. "*.txt" */ }`
            - Ausgabe von `open_file_dialog`: `Vec<PathBuf>` (URIs werden von der Systemschicht in Pfade konvertiert)
            - Eingabe für `save_file_dialog`: `parent_window_identifier: system::portals::WindowIdentifier`, `title: String`, `options: system::portals::SaveFileOptions { current_name: Option<String>, current_folder: Option<PathBuf>, filters: Vec<system::portals::FileFilter> }`
            - Ausgabe von `save_file_dialog`: `Option<PathBuf>`
            - Typ `system::portals::WindowIdentifier`: Ein plattformagnostischer Handle für ein Fenster, der intern zu Wayland- oder X11-spezifischen Handles aufgelöst wird, wie von `ashpd` benötigt.
        - **Von der UI aufgerufene API-Methoden (aus `system::PortalsInterface`-Trait):**
            - `async fn open_file_dialog(&self, parent_window_identifier: system::portals::WindowIdentifier, title: String, options: system::portals::OpenFileOptions) -> Result<Vec<PathBuf>, system::PortalError>;`
            - `async fn save_file_dialog(&self, parent_window_identifier: system::portals::WindowIdentifier, title: String, options: system::portals::SaveFileOptions) -> Result<Option<PathBuf>, system::PortalError>;`
            - `async fn take_screenshot(&self, parent_window_identifier: system::portals::WindowIdentifier, interactive: bool, include_cursor: bool) -> Result<Option<PathBuf>, system::PortalError>;`
            - `async fn pick_color(&self, parent_window_identifier: system::portals::WindowIdentifier) -> Result<Option<core::types::Color>, system::PortalError>;`
        - **Von der UI konsumierte Event-Payloads:** (Portals sind typischerweise Request-Response, weniger eventbasiert für die UI direkt, außer Status-Events des Portal-Dienstes selbst, die hier aber nicht im Fokus stehen).
        - **Fehlerbehandlung durch die UI:** Abbildung von `system::PortalError`-Varianten (z.B. `UserCancelled`, `PortalNotAvailable`, `BackendError`).
    - **Schlüsseltabellen für Teil 1.3:** Ähnlich wie in Teil 1.2 werden Tabellen für Methodensignaturen, Parameterdetails, Rückgabewertdetails und Ereignis-Payload-Details für jede definierte Systemschicht-Schnittstelle erstellt.
        
        - Die Interaktionen der Systemschicht sind oft komplex und beinhalten viele Low-Level-Details (z.B. D-Bus-Varianten, Wayland-Protokollspezifika). Tabellen machen die abstrahierte Rust-API klar und reduzieren die Mehrdeutigkeit für UI-Entwickler.

---

**Teil 2: Ultra-Granulare Schnittstellendefinition der Kernschicht (Erweiterung des Benutzerdokuments)**

- 2.1. Kernschicht zu allen höheren Schichten
    
    Die Kernschicht (core) stellt fundamentale Bausteine und Dienste bereit, die von allen höheren Schichten (Domäne, System, UI) genutzt werden. Ihre Schnittstellen müssen daher besonders stabil und klar definiert sein.
    
    - **`core::types`:**
        
        - **Datentypen:** Stellt eine Sammlung grundlegender, wiederverwendbarer Datentypen bereit.
            - `Point<T> { x: T, y: T }` (generisch für numerische Typen wie `f32`, `i32`)
            - `Size<T> { width: T, height: T }` (generisch)
            - `Rect<T> { origin: Point<T>, size: Size<T> }` (generisch)
            - `RectInt` ist ein Typalias für `Rect<i32>`.
            - `Color { r: u8, g: u8, b: u8, a: u8 }` (RGBA-Format)
            - `Orientation { Horizontal, Vertical }` (Enum)
            - `Uuid` (Wrapper um die `uuid::Uuid`-Bibliothek, um eine konsistente Verwendung sicherzustellen)
            - `DateTime<Utc>` (Wrapper um `chrono::DateTime<chrono::Utc>`, um Zeitzonenprobleme zu standardisieren)
            - `WindowId` (Typalias, z.B. `pub type WindowId = u64;` oder eine spezifischere Struktur, falls Metadaten benötigt werden)
            - `WorkspaceId` (Typalias, z.B. `pub type WorkspaceId = core::types::Uuid;`)
            - `AppId` (Typalias, z.B. `pub type AppId = String;`, oft im Reverse-DNS-Format wie `org.novade.FileExplorer`)
        - **Nutzung:** Diese Typen werden direkt in den öffentlichen APIs und internen Logiken der höheren Schichten für Geometrieberechnungen, Farbangaben, eindeutige Identifikatoren, Zeitstempel und spezifische Entitäts-IDs verwendet. Die Standardisierung dieser Typen in der Kernschicht verhindert Inkonsistenzen und erleichtert die Interoperabilität zwischen den Modulen.
    - **`core::errors`:**
        
        - **Basis-Fehlertyp:**
            
            Rust
            
            ```
            #
            pub enum CoreError {
                #[error("I/O error: {source}")]
                Io { #[from] source: std::io::Error },
                #[error("Configuration parsing error: {0}")]
                ConfigParseError(String),
                #[error("Invalid state: {0}")]
                InvalidState(String),
                #[error("Functionality not implemented")]
                NotImplemented,
                #[error("Custom error: {0}")]
                Custom(String),
                #
                SerializationJson { #[from] source: serde_json::Error },
                // Weitere generische Fehlerkategorien hier hinzufügen
            }
            ```
            
        - **Modul-spezifische Fehlerstrategie:** Höhere Schichten und Module innerhalb der Kernschicht definieren ihre eigenen spezifischen Fehler-Enums mit `thiserror`. Diese spezifischen Fehler sollten `CoreError` oder andere relevante Fehler wrappen, um die Fehlerkette zu erhalten.
            - Beispiel: `core::config::ConfigError` könnte eine Variante haben wie:
                
                Rust
                
                ```
                #
                pub enum ConfigError {
                    #[error("Failed to open configuration file '{path}': {source}")]
                    FileOpenFailed { path: PathBuf, #[source] source: CoreError },
                    #[error("Failed to parse configuration: {message}")]
                    ParseFailed { message: String, #[source] source: Option<serde_json::Error> },
                    //...
                }
                ```
                
        - **Fehler-Wrapping-Illustration:**
            
            Rust
            
            ```
            // In einem hypothetischen Modul
            #
            pub enum MyModuleError {
                #[error("Failed to read file for stuff: {source}")]
                FileReadFailed(#[source] CoreError),
                //...
            }
            
            fn read_file_content(path: &PathBuf) -> Result<String, CoreError> {
                std::fs::read_to_string(path).map_err(CoreError::from)
            }
            
            fn load_stuff(file_path: &PathBuf) -> Result<(), MyModuleError> {
                let _content = read_file_content(file_path).map_err(MyModuleError::FileReadFailed)?;
                //... weitere Logik...
                Ok(())
            }
            ```
            
            Diese Strategie stellt sicher, dass der ursprüngliche Kontext des Fehlers (`source()`) verfügbar bleibt, was die Fehlersuche und -behebung erheblich erleichtert.
    - **`core::logging`:**
        
        - **Initialisierungsroutine:**
            
            Rust
            
            ```
            pub enum LogOutput {
                Stdout,
                Stderr,
                File(PathBuf),
            }
            
            pub fn initialize_logging(level: tracing::Level, output: LogOutput, format: LogFormat) -> Result<(), CoreError>;
            pub enum LogFormat { Json, Pretty, Compact }
            ```
            
            Diese Funktion konfiguriert das globale Logging-System (basierend auf `tracing`).
        - **Konvention für `tracing`-Makros:**
            - Alle Schichten verwenden die Standard-`tracing`-Makros (`trace!`, `debug!`, `info!`, `warn!`, `error!`).
            - Strukturierte Log-Nachrichten werden bevorzugt, um kontextbezogene Daten effizient zu erfassen: `trace!(target: "core::module_name", "Processing item {}", item_id, item_type = %item.kind, user_id = %current_user.id);`
            - Das `target`-Feld sollte das Modul oder die Komponente angeben, aus der die Log-Nachricht stammt (z.B. `core::config`, `domain::theming`).
            - Spezifische Felder (z.B., `window.id = %window_id`, `event.name = "ThemeChanged"`) sollten verwendet werden, um das Filtern und Analysieren von Logs zu erleichtern.
    - **`core::config`:**
        
        - **`CoreConfig`-Struktur:** Definiert die globalen Kernkonfigurationen. Diese Struktur wird beim Start der Anwendung geladen und als unveränderlich betrachtet.
            
            Rust
            
            ```
            #
            pub struct LoggingConfig {
                pub level: String, // z.B. "info", "debug"
                pub output: String, // z.B. "stdout", "/var/log/novade.log"
                pub format: String, // z.B. "json", "pretty"
            }
            
            #
            pub struct ThemeConfig {
                pub default_theme_name: String,
                pub default_variant: String,
                pub icon_theme: String,
            }
            
            #
            pub struct AIServiceConfig {
                pub service_url: Option<String>,
                pub default_model_id: Option<String>,
                pub consent_required_by_default: bool,
            }
            
            #
            pub struct CoreConfig {
                pub logging: LoggingConfig,
                pub theming: ThemeConfig,
                pub ai: AIServiceConfig,
                pub default_workspace_layout: String, // z.B. "Tiling", "Floating"
                // Weitere Kernkonfigurationen
            }
            ```
            
        - **Konfigurations-API:**
            - `pub fn load_core_config(path: &PathBuf) -> Result<CoreConfig, ConfigError>;` Lädt die Konfiguration aus einer Datei (z.B. TOML oder JSON).
            - `pub fn get_core_config() -> Result<&'static CoreConfig, CoreError>;` Bietet globalen, schreibgeschützten Zugriff auf die geladene Kernkonfiguration. Dies setzt voraus, dass die Konfiguration nach der Initialisierung in einem globalen statischen Speicher (z.B. `once_cell::sync::OnceCell` oder `std::sync::OnceLock`) gehalten wird.
        - **`ConfigError`-Enum:**
            
            Rust
            
            ```
            #
            pub enum ConfigError {
                #[error("Configuration file not found at path: {0}")]
                FileNotFound(PathBuf),
                #[error("Failed to open configuration file '{path}': {source}")]
                FileOpenFailed { path: PathBuf, #[source] source: CoreError },
                #[error("Failed to read configuration file '{path}': {source}")]
                FileReadFailed { path: PathBuf, #[source] source: CoreError },
                #[error("Failed to parse configuration from '{path}': {message}")]
                ParseFailed { path: PathBuf, message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync>> }, // Box für Flexibilität bei Parser-Fehlern
                #[error("Configuration validation failed: {0}")]
                ValidationError(String),
            }
            ```
            
            (Hinweis: `Box<dyn std::error::Error + Send + Sync>` für `source` in `ParseFailed` ermöglicht das Wrappen verschiedener Parser-Fehler, z.B. `serde_json::Error`, `toml::de::Error`).
    - **`core::utils`:**
        
        - **Hilfsfunktionen:** Stellt allgemeine, zustandslose Hilfsfunktionen bereit, die von mehreren Schichten genutzt werden können. Diese sollten in thematisch gruppierten Untermodulen organisiert sein.
            - `core::utils::string_utils`: Funktionen zur String-Manipulation (z.B. `truncate_with_ellipsis(text: &str, max_len: usize) -> String;`).
            - `core::utils::geometry_utils`: Einfache geometrische Berechnungen, die nicht in `core::types` selbst implementiert sind (z.B. `fn calculate_rect_area(rect: &core::types::RectInt) -> i32;`).
            - `core::utils::time_utils`: Formatierungs- oder Vergleichsfunktionen für `DateTime<Utc>` (z.B. `fn format_datetime_human_readable(dt: &core::types::DateTime<Utc>) -> String;` unter Verwendung von `chrono::format::strftime`).
            - `core::utils::uuid_utils`: Funktionen zur Erzeugung oder Validierung von UUIDs (z.B. `fn generate_v4_uuid() -> core::types::Uuid;`).
        - **Nutzung:** Direkte Verwendung durch alle höheren Schichten nach Bedarf. Da diese Funktionen zustandslos sind, führen sie keine Seiteneffekte ein und sind einfach zu testen. Die Signaturen dieser Hilfsfunktionen sind Teil der internen API der Kernschicht und sollten bei Stabilitätserwägungen berücksichtigt werden.
    - **Schlüsseltabellen für Teil 2.1:**
        
        1. **`core::types` Detail-Tabelle:**

|   |   |   |   |
|---|---|---|---|
|**Typname**|**Struktur / Enum-Definition**|**Generische Parameter**|**Beschreibung**|
|`Point<T>`|`struct Point<T> { x: T, y: T }`|`T: Copy + Num`|Repräsentiert einen Punkt im 2D-Raum.|
|`Color`|`struct Color { r: u8, g: u8, b: u8, a: u8 }`|-|Repräsentiert eine RGBA-Farbe.|
|`Uuid`|`struct Uuid(uuid::Uuid);`|-|Wrapper für einen universell eindeutigen Identifikator.|
|`DateTime<Utc>`|`struct DateTime<Utc>(chrono::DateTime<chrono::Utc>);`|-|Repräsentiert einen Zeitpunkt in UTC.|
|...|...|...|...|

```
    2.  **`CoreError` Varianten-Tabelle:**
```

|   |   |   |
|---|---|---|
|**Variante**|**Assoziierte Daten (#[from], #[source], Felder)**|**#[error("...")] Format-String**|
|`Io`|`#[from] source: std::io::Error`|`"I/O error: {source}"`|
|`ConfigParseError`|`String` (Meldung)|`"Configuration parsing error: {0}"`|
|`SerializationJson`|`#[from] source: serde_json::Error`|`"Serialization error (serde_json): {source}"`|
|...|...|...|

```
    3.  **`CoreConfig` Felder-Tabelle:**
```

|   |   |   |   |
|---|---|---|---|
|**Feldname**|**Typ**|**Beschreibung**|**Standardwert (falls zutreffend)**|
|`logging`|`LoggingConfig`|Konfiguration für das Logging-System.|Siehe `LoggingConfig`|
|`theming`|`ThemeConfig`|Standard-Theme-Einstellungen.|Siehe `ThemeConfig`|
|`default_workspace_layout`|`String`|Standard-Layout für neue Arbeitsbereiche (z.B. "Tiling").|`"Floating"`|
|...|...|...|...|

```
    Diese Tabellen bieten eine klare, strukturierte Übersicht über die von der Kernschicht bereitgestellten fundamentalen Elemente und sind für Entwickler aller höheren Schichten von entscheidender Bedeutung.
```

---

**Teil 3: Ultra-Granulare Schnittstellendefinition der Domänenschicht (Erweiterung des Benutzerdokuments)**

Die Domänenschicht enthält die Kernlogik und den Zustand der Anwendung. Ihre Schnittstellen definieren, wie höhere Schichten (System und UI) diese Logik nutzen und auf Zustandsänderungen reagieren können.

- **3.1. Domänenschicht zu System- & UI-Schichten (Allgemeine Prinzipien)**
    
    - **Logik und Zustand:** Jede Domänen-Service-Komponente kapselt einen spezifischen Aspekt der Geschäftslogik und des zugehörigen Zustands. Diese werden über öffentliche Methoden der Service-Traits und durch das Aussenden von domänenspezifischen Ereignissen bereitgestellt.
        - Für jeden Domänendienst wird explizit der von ihm verwaltete Zustand (z.B. für `ThemingEngine`: aktuelles Theme, verfügbare Themes, benutzerdefinierte Farbpaletten) und die wichtigsten Geschäftsregeln, die er durchsetzt (z.B. für `WorkspaceManager`: keine zwei Arbeitsbereiche mit demselben Namen, ein Fenster kann nur einem Arbeitsbereich zugewiesen sein), dokumentiert.
    - **Fehlertypen:** Jedes Modul innerhalb der Domänenschicht definiert seine eigenen, auf `thiserror` basierenden Fehler-Enums. Diese Fehler sind spezifisch für die jeweilige Domänenlogik und bieten detaillierte Informationen über aufgetretene Probleme.
        - Beispiel für `ThemingError`:
            
            Rust
            
            ```
            #
            pub enum ThemingError {
                #
                ThemeNotFound { name: String, path: Option<PathBuf> },
                #
                VariantNotFound { theme_name: String, variant_name: String },
                #[error("Failed to load theme assets for '{theme_name}': {source}")]
                AssetLoadError { theme_name: String, #[source] source: core::errors::CoreError },
                #[error("Invalid theme configuration: {message}")]
                InvalidConfiguration { message: String },
                #
                TokenNotFound { theme_name: String, token_name: String },
                #[error("Failed to parse color value '{value}' for token '{token_name}'")]
                ColorParseError { token_name: String, value: String },
            }
            ```
            
            (Diese detaillierte Definition wird für alle im Benutzerdokument genannten Fehler wie `WorkspaceCoreError`, `WindowAssignmentError`, etc. wiederholt.)
    - **Events:** Domänenspezifische Ereignisse werden ausgelöst, um andere Schichten über signifikante Zustandsänderungen oder abgeschlossene Aktionen zu informieren. Die Payload jedes Ereignisses ist so gestaltet, dass sie alle relevanten Informationen enthält, die ein Konsument benötigt, um ohne weitere Abfragen reagieren zu können.
        - Beispiel für `ThemeChangedEvent`:
            
            Rust
            
            ```
            # // Serialize/Deserialize falls Events über IPC gehen könnten
            pub struct PaletteDetails {
                pub primary_accent: core::types::Color,
                pub secondary_accent: core::types::Color,
                pub foreground: core::types::Color,
                pub background: core::types::Color,
                //... weitere relevante Farbdefinitionen
            }
            
            #
            pub struct ThemeChangedEvent {
                pub new_theme_name: String,
                pub new_variant_name: String,
                pub applied_palette: PaletteDetails, // Statt nur IDs, die volle Information
                pub font_details: FontDescription, // Annahme einer Struktur für Font-Details
            }
            
            #
            pub struct FontDescription {
                pub family: String,
                pub size_pt: f32,
                pub style: String, // z.B. "Regular", "Bold"
            }
            ```
            
            (Diese detaillierte Definition wird für alle im Benutzerdokument genannten Ereignisse wie `WorkspaceEvent`, `AIInteractionInitiatedEvent`, etc. wiederholt.)
- **3.2. Spezifische Domänen-Service-Schnittstellen (Ultra-Granular):**
    
    - **`ThemingEngine` API:**
        
        - **Trait Definition:**
            
            Rust
            
            ```
            pub trait ThemingEngine: Send + Sync {
                /// Ruft den aktuell angewendeten und aufgelösten Theme-Zustand ab.
                fn get_current_theme_state(&self) -> Result<AppliedThemeState, ThemingError>;
            
                /// Listet alle verfügbaren Themes und deren Varianten auf.
                fn get_available_themes(&self) -> Result<Vec<ThemeIdentifier>, ThemingError>;
            
                /// Ruft die aktuelle, persistierte Theme-Konfiguration ab (z.B. ausgewählter Theme-Name, Variante, Overrides).
                fn get_current_configuration(&self) -> Result<ThemeConfiguration, ThemingError>;
            
                /// Aktualisiert die Theme-Konfiguration. Änderungen werden persistiert und lösen ggf. ein `ThemeChangedEvent` aus.
                fn update_configuration(&self, config_update: ThemeConfigurationUpdate) -> Result<(), ThemingError>;
            
                /// Lädt alle Theme-Definitionen und Farb-Token neu vom Dateisystem oder der konfigurierten Quelle.
                fn reload_themes_and_tokens(&self) -> Result<(), ThemingError>;
            
                /// Abonniert Änderungen am Theme-Zustand.
                /// Gibt einen Receiver-Endpunkt eines Broadcast-Kanals zurück, über den `ThemeChangedEvent`s empfangen werden können.
                /// Alternativ könnte dies über einen zentralen EventBus erfolgen.
                fn subscribe_to_theme_changes(&self) -> Result<tokio::sync::broadcast::Receiver<ThemeChangedEvent>, ThemingError>;
            }
            ```
            
        - **Datenstrukturen:**
            - `AppliedThemeState { name: String, variant: String, primary_color: core::types::Color, text_color: core::types::Color, background_color: core::types::Color, accent_colors: HashMap<String, core::types::Color>, font_family: String, font_size_pt: f32, raw_tokens: HashMap<String, String> /* Für Debugging oder fortgeschrittene Nutzung */ }`
            - `ThemeIdentifier { id: String, name: String, display_name: String, description: Option<String>, author: Option<String>, variants: Vec<ThemeVariantIdentifier> }`
            - `ThemeVariantIdentifier { id: String, name: String, display_name: String, preview_image_path: Option<PathBuf> }`
            - `ThemeConfiguration { current_theme_id: String, current_variant_id: String, custom_palette_overrides: HashMap<String, core::types::Color>, font_scale_factor: Option<f32> }`
            - `ThemeConfigurationUpdate { theme_id: Option<String>, variant_id: Option<String>, set_custom_palette_overrides: Option<HashMap<String, core::types::Color>>, remove_custom_palette_overrides: Option<Vec<String>>, font_scale_factor: Option<f32> }`
        - **Events:**
            - `ThemeChangedEvent { new_state: AppliedThemeState }` (wie oben definiert, Payload enthält alle relevanten Infos)
        - **Fehler:** `ThemingError` (Varianten wie oben detailliert definiert: `ThemeNotFound`, `VariantNotFound`, `AssetLoadError`, `InvalidConfiguration`, `TokenNotFound`, `ColorParseError`).
    - **`WorkspaceManager` API:**
        
        - **Trait Definition:**
            
            Rust
            
            ```
            pub trait WorkspaceManager: Send + Sync {
                fn create_workspace(&self, params: CreateWorkspaceParams) -> Result<WorkspaceDescriptor, WorkspaceManagerError>;
                fn delete_workspace(&self, workspace_id: core::types::WorkspaceId) -> Result<(), WorkspaceManagerError>;
                fn set_active_workspace(&self, workspace_id: core::types::WorkspaceId) -> Result<(), WorkspaceManagerError>;
                fn get_active_workspace(&self) -> Result<Option<WorkspaceDescriptor>, WorkspaceManagerError>;
                fn get_workspace_by_id(&self, workspace_id: core::types::WorkspaceId) -> Result<Option<WorkspaceDescriptor>, WorkspaceManagerError>;
                fn list_workspaces(&self) -> Result<Vec<WorkspaceDescriptor>, WorkspaceManagerError>;
                fn assign_window_to_workspace(&self, window_id: core::types::WindowId, workspace_id: core::types::WorkspaceId) -> Result<(), WorkspaceManagerError>;
                fn assign_window_to_active_workspace(&self, window_id: core::types::WindowId) -> Result<(), WorkspaceManagerError>;
                fn remove_window_from_workspace(&self, window_id: core::types::WindowId, workspace_id: core::types::WorkspaceId) -> Result<(), WorkspaceManagerError>;
                fn get_window_assignment(&self, window_id: core::types::WindowId) -> Result<Option<core::types::WorkspaceId>, WorkspaceManagerError>;
                fn save_configuration(&self) -> Result<(), WorkspaceManagerError>; // Persistiert die aktuelle Arbeitsbereichsanordnung und -konfiguration
                fn load_configuration(&self) -> Result<(), WorkspaceManagerError>; // Lädt die Konfiguration beim Start
                fn rename_workspace(&self, workspace_id: core::types::WorkspaceId, new_name: String) -> Result<(), WorkspaceManagerError>;
                fn set_workspace_layout(&self, workspace_id: core::types::WorkspaceId, layout: WorkspaceLayoutType) -> Result<(), WorkspaceManagerError>;
                fn reorder_workspaces(&self, ordered_workspace_ids: Vec<core::types::WorkspaceId>) -> Result<(), WorkspaceManagerError>;
                fn subscribe_to_workspace_events(&self) -> Result<tokio::sync::broadcast::Receiver<WorkspaceEvent>, WorkspaceManagerError>;
            }
            ```
            
        - **Datenstrukturen:**
            - `CreateWorkspaceParams { name: Option<String>, layout_hint: Option<WorkspaceLayoutType>, icon_name: Option<String>, activate_when_created: bool }`
            - `WorkspaceDescriptor { id: core::types::WorkspaceId, name: String, display_name: String, icon_name: Option<String>, layout: WorkspaceLayoutType, window_ids: Vec<core::types::WindowId>, is_active: bool, creation_timestamp: core::types::DateTime<Utc>, last_accessed_timestamp: core::types::DateTime<Utc> }`
            - `WorkspaceLayoutType { TilingHorizontal, TilingVertical, Floating, MaximizedSingleWindow }`
        - **Events (als Enum `WorkspaceEvent` zusammengefasst für einfacheres Abonnement):**
            
            Rust
            
            ```
            #
            pub enum WorkspaceEvent {
                WorkspaceCreated { descriptor: WorkspaceDescriptor },
                WorkspaceDeleted { workspace_id: core::types::WorkspaceId, previous_name: String },
                ActiveWorkspaceChanged { old_workspace_id: Option<core::types::WorkspaceId>, new_workspace_id: core::types::WorkspaceId, new_descriptor: WorkspaceDescriptor },
                WorkspaceLayoutChanged { workspace_id: core::types::WorkspaceId, new_layout: WorkspaceLayoutType },
                WindowAssignedToWorkspace { window_id: core::types::WindowId, new_workspace_id: core::types::WorkspaceId, old_workspace_id: Option<core::types::WorkspaceId> },
                WindowRemovedFromWorkspace { window_id: core::types::WindowId, workspace_id: core::types::WorkspaceId },
                WorkspaceRenamed { workspace_id: core::types::WorkspaceId, new_name: String, old_name: String },
                WorkspaceOrderChanged { ordered_workspace_ids: Vec<core::types::WorkspaceId> },
                WorkspaceConfigurationSaved,
                WorkspaceConfigurationLoaded,
            }
            ```
            
        - **Fehler:** `WorkspaceManagerError`, `WorkspaceConfigError`, `WindowAssignmentError` (jeweils mit detaillierten Varianten wie `WorkspaceNotFound`, `NameAlreadyExists`, `WindowNotKnownToCompositor`, `ConfigSaveFailed`, `ConfigLoadFailed`).
    - **(Diese Detailtiefe wird für `AIInteractionLogicService`, `NotificationService` und `GlobalSettingsService` wiederholt, basierend auf den im Benutzerdokument und in Teil 1.2 dieses Dokuments skizzierten Schnittstellen. Für `GlobalSettingsService` ist die Verwendung von `glib::Variant` für Einstellwerte hervorzuheben, um Flexibilität bei den Datentypen zu gewährleisten.12)**
        
    - Schlüsseltabellen für Teil 3.2:
        
        Ähnlich wie in Teil 1.2 und Teil 2.1 werden für jeden Domänendienst detaillierte Tabellen für Methodensignaturen, Parameter, Rückgabewerte und Ereignis-Payloads erstellt.
        
        - **Beispiel: `AIInteractionLogicService` Methodenübersicht**

|   |   |   |   |
|---|---|---|---|
|**Methode**|**Parameter**|**Rückgabetyp (async Result<_, _>)**|**Kurzbeschreibung**|
|`initiate_interaction`|`&self, request: domain::ai::InteractionRequest`|`Result<domain::ai::InteractionResponseHandle, domain::AIInteractionError>`|Startet eine neue KI-Interaktion.|
|`get_interaction_context`|`&self, interaction_id: core::types::Uuid`|`Result<domain::ai::InteractionContext, domain::AIInteractionError>`|Ruft den aktuellen Kontext und Verlauf einer Interaktion ab.|
|`provide_consent`|`&self, interaction_id: core::types::Uuid, model_id: String, granted: bool`|`Result<(), domain::AIInteractionError>`|Gibt die Zustimmung für ein bestimmtes Modell im Kontext einer Interaktion.|
|...|...|...|...|

```
    *   **Beispiel: `AIInteractionStatusChangedEvent` Payload**
```

|   |   |   |
|---|---|---|
|**Feldname**|**Typ**|**Beschreibung**|
|`interaction_id`|`core::types::Uuid`|ID der Interaktion, deren Status sich geändert hat.|
|`new_status`|`domain::ai::AIInteractionStatus`|Der neue Status der Interaktion.|
|`details`|`Option<String>`|Zusätzliche Details oder Fehlermeldung zum Statuswechsel.|

```
    Die präzise Definition dieser Domänenschnittstellen ist fundamental, da sie die Kernfunktionalität von NovaDE kapseln und die Verträge für die Interaktion mit der System- und UI-Schicht festlegen.
```

---

**Teil 4: Ultra-Granulare Schnittstellendefinition der Systemschicht (Erweiterung des Benutzerdokuments)**

Die Systemschicht ist verantwortlich für die Interaktion mit dem zugrundeliegenden Betriebssystem, der Hardware und Low-Level-Diensten wie dem Compositor, Eingabegeräten und D-Bus-Diensten. Sie stellt der UI-Schicht abstrahierte Schnittstellen zu diesen systemnahen Funktionen bereit.

- **4.1. Systemschicht zu UI-Schicht (Allgemeine Prinzipien)**
    
    - **Systemnahe Dienste und Ereignisse:** Die Systemschicht stellt der UI-Schicht Informationen und Ereignisse bereit, die direkt vom Betriebssystem, der Hardware oder systemnahen Daemons stammen. Für jede Kategorie (z.B. Fenstergeometrie, Eingabeereignisse, Systemstatusänderungen) werden spezifische Ereignisse mit detaillierten Payloads definiert.
    - **Technische Umsetzung von UI-Befehlen:** Die Systemschicht empfängt Befehle von der UI-Schicht (z.B. Fenster verschieben, Arbeitsbereich wechseln, Fokus anfordern) und setzt diese technisch um, indem sie mit dem Compositor, dem Fenstermanager oder anderen Systemkomponenten interagiert. Für jeden solchen Befehl wird eine exakte Methodensignatur in der API der Systemschicht definiert, die von der UI aufgerufen wird.
    - **Renderer-Schnittstelle (`system::compositor::renderer_interface`):** Obwohl nicht direkt von der UI-Schicht konsumiert, stellt diese Schnittstelle eine Abstraktion für das Rendering bereit. Wenn diese Schnittstelle Implikationen dafür hat, wie UI-Elemente letztendlich dargestellt werden (z.B. unterstützte Oberflächenformate, Anforderungen an Textur-Sharing), werden diese kurz vermerkt, da sie indirekt die UI-Implementierung beeinflussen können.
    - **Fehlertypen:** Jedes Modul der Systemschicht (z.B. `system::compositor`, `system::input`) definiert eigene, auf `thiserror` basierende Fehler-Enums (z.B. `CompositorError`, `InputError`). Diese Fehler-Enums listen alle spezifischen Fehlerzustände auf, die bei der Interaktion mit den jeweiligen Systemkomponenten auftreten können.
- **4.2. Spezifische Systemschicht-Schnittstellen (Ultra-Granular, für UI-Interaktion):**
    
    - **Compositor-Schnittstelle (`system::compositor_iface`):**
        
        - Diese Schnittstelle wird als Trait `CompositorInterface` definiert, um eine klare API für die UI-Schicht bereitzustellen.
        - **Methoden, die von der UI aufgerufen werden:**
            
            Rust
            
            ```
            pub trait CompositorInterface: Send + Sync {
                /// Fordert den Fokus für ein bestimmtes Fenster an.
                fn request_focus(&self, window_id: core::types::WindowId) -> Result<(), CompositorError>;
            
                /// Verschiebt ein Fenster an eine neue Position.
                fn move_window(&self, window_id: core::types::WindowId, new_pos: core::types::PointInt) -> Result<(), CompositorError>;
            
                /// Ändert die Größe eines Fensters.
                fn resize_window(&self, window_id: core::types::WindowId, new_size: core::types::SizeInt) -> Result<(), CompositorError>;
            
                /// Fordert einen neuen Zustand für ein Fenster an (z.B. Maximieren, Minimieren).
                fn set_window_state(&self, window_id: core::types::WindowId, state: CompositorWindowState) -> Result<(), CompositorError>;
            
                /// Schließt ein Fenster.
                fn close_window(&self, window_id: core::types::WindowId) -> Result<(), CompositorError>;
            
                /// Ruft detaillierte Informationen zu einem bestimmten Fenster ab.
                fn get_window_info(&self, window_id: core::types::WindowId) -> Result<Option<WindowInfo>, CompositorError>;
            
                /// Listet alle aktuell vom Compositor verwalteten Fenster auf.
                fn list_windows(&self) -> Result<Vec<WindowInfo>, CompositorError>;
            
                /// Ruft Informationen zu allen verfügbaren Outputs (Monitoren) ab.
                fn list_outputs(&self) -> Result<Vec<OutputInfo>, CompositorError>;
            
                /// Konfiguriert einen Output (Auflösung, Skalierung, etc.).
                fn configure_output(&self, output_id: String, config: OutputConfigurationRequest) -> Result<(), CompositorError>;
            
                /// Abonniert Compositor-Ereignisse.
                fn subscribe_to_compositor_events(&self) -> Result<tokio::sync::broadcast::Receiver<CompositorEvent>, CompositorError>;
            }
            ```
            
        - **Datenstrukturen:**
            - `WindowInfo { id: core::types::WindowId, title: String, app_id: core::types::AppId, class: Option<String>, geometry: core::types::RectInt, is_focused: bool, is_fullscreen: bool, is_maximized: bool, is_minimized: bool, workspace_id: Option<core::types::WorkspaceId>, parent_id: Option<core::types::WindowId>, transient_for_id: Option<core::types::WindowId>, can_resize: bool, can_maximize: bool, can_minimize: bool, can_close: bool, pid: Option<u32> }`
            - `CompositorWindowState { Maximized, Minimized, Fullscreen, Normal, TiledLeft, TiledRight, Floating }`
            - `OutputInfo { id: String, name: String, description: String, manufacturer: Option<String>, model: Option<String>, serial_number: Option<String>, physical_size_mm: Option<core::types::Size<u32>>, geometry_pixels: core::types::RectInt, scale_factor: f32, current_refresh_rate_mhz: u32, current_mode: OutputMode, available_modes: Vec<OutputMode>, is_primary: bool, is_enabled: bool, transform: core::types::Orientation }`
            - `OutputMode { width_px: u32, height_px: u32, refresh_rate_mhz: u32, is_preferred: bool }`
            - `OutputConfigurationRequest { resolution: Option<core::types::SizeInt>, scale: Option<f32>, position: Option<core::types::PointInt>, enabled: Option<bool>, refresh_rate_mhz: Option<u32>, transform: Option<core::types::Orientation>, is_primary: Option<bool> }`
        - **Events (als Enum `CompositorEvent` für das Abonnement):**
            
            Rust
            
            ```
            #
            pub enum CompositorEvent {
                WindowCreated { info: WindowInfo },
                WindowClosed { window_id: core::types::WindowId, app_id: core::types::AppId },
                WindowFocused { window_id: core::types::WindowId },
                WindowUnfocused { window_id: core::types::WindowId },
                WindowPropertiesChanged { window_id: core::types::WindowId, changed_properties: WindowPropertiesDelta },
                OutputAdded { info: OutputInfo },
                OutputRemoved { output_id: String },
                OutputConfigurationChanged { info: OutputInfo }, // Enthält den neuen Zustand des Outputs
                // Weitere Ereignisse wie Workspace-Wechsel, wenn vom Compositor direkt gemeldet
            }
            
            #
            pub struct WindowPropertiesDelta {
                pub title: Option<String>,
                pub app_id: Option<core::types::AppId>,
                pub class: Option<Option<String>>, // Option<Option<T>> um 'wurde auf None gesetzt' von 'wurde nicht geändert' zu unterscheiden
                pub geometry: Option<core::types::RectInt>,
                pub is_fullscreen: Option<bool>,
                pub is_maximized: Option<bool>,
                pub is_minimized: Option<bool>,
                pub workspace_id: Option<Option<core::types::WorkspaceId>>,
            }
            ```
            
        - **Fehler:** `CompositorError` mit Varianten wie `WindowNotFound`, `OutputNotFound`, `InvalidOperationForWindowState`, `WaylandCommunicationError(String)`, `X11CommunicationError(String)`.
        - Die Systemschicht abstrahiert hier die Komplexität von Wayland-Protokollen wie `wlr-foreign-toplevel-management` 17 oder `ext-foreign-toplevel-list-v1` 17, die für Taskbars und Docks zur Abfrage von Fensterlisten fundamental sind. Die UI konsumiert die `WindowInfo`-Struktur, die von dieser Abstraktion bereitgestellt wird. Für die Implementierung von UI-Elementen wie Panels oder Docks, die eine feste Position auf dem Bildschirm einnehmen, wird die Systemschicht intern `gtk4-layer-shell` 9 verwenden und eine abstrahierte API dafür anbieten.
    - **(Diese Detailtiefe wird für `system::input_iface`, `system::dbus::upower_client_iface`, `system::dbus::logind_client_iface`, `system::dbus::networkmanager_client_iface` (unter Berücksichtigung von D-Bus-Spezifikationen für NetworkManager 23 und BlueZ 20), `system::outputs_iface` (bereits teilweise in Compositor-Schnittstelle enthalten, hier ggf. spezifischere Konfigurationsmethoden), `system::audio_iface` (Abstraktion von PipeWire 26), `system::mcp_client_iface` und `system::portals_iface` (Abstraktion von XDG Desktop Portals, potenziell unter Verwendung von Bibliotheken wie `ashpd` 28) wiederholt, wie in Teil 1, Abschnitt 1.3 skizziert.)**
        
        - Für `system::portals_iface` ist die `WindowIdentifier`-Struktur, die von `ashpd` verwendet wird, ein wichtiger Aspekt, um Portal-Dialoge korrekt dem aufrufenden Anwendungsfenster zuzuordnen. Die Systemschicht muss einen Mechanismus bereitstellen, um diese Kennung von der UI zu erhalten oder abzuleiten.
    - Schlüsseltabellen für Teil 4.2:
        
        Ähnlich wie in den vorherigen Teilen werden für jede Systemschicht-Schnittstelle detaillierte Tabellen für Methodensignaturen, Parameter, Rückgabewerte und Ereignis-Payloads erstellt.
        
        - **Beispiel: `InputInterface` Methodenübersicht**

|   |   |   |   |
|---|---|---|---|
|**Methode**|**Parameter**|**Rückgabetyp (Result<_, _>)**|**Kurzbeschreibung**|
|`set_cursor_theme`|`&self, theme_name: String, size: u32`|`Result<(), system::InputError>`|Setzt das Cursor-Theme und die Größe.|
|`get_current_keyboard_layout`|`&self`|`Result<String, system::InputError>`|Ruft das aktuell aktive Tastaturlayout ab.|
|`subscribe_to_input_events`|`&self`|`Result<tokio::sync::broadcast::Receiver<system::input::InputEventEnum>, system::InputError>`|Abonniert verarbeitete Eingabeereignisse.|

```
    *   **Beispiel: `system::input::InputEventEnum` Payload (Auszug)**
        ```rust
        #
        pub enum InputEventEnum {
            Keyboard(KeyboardEvent),
            PointerButton(PointerButtonEvent),
            PointerMotion(PointerMotionEvent),
            //... weitere Event-Typen
        }

        #
        pub struct KeyboardEvent {
            pub window_id: Option<core::types::WindowId>, // Fenster, das den Fokus hatte, falls zutreffend
            pub key_code: u32,          // System-agnostischer Keycode
            pub raw_key_code: u32,      // Hardware-Keycode
            pub modifiers: KeyModifiers, // Bitmaske für Shift, Ctrl, Alt, Super
            pub state: KeyState,        // Pressed, Released
            pub utf8: Option<String>,   // Decodiertes Zeichen, falls vorhanden
            pub timestamp_us: u64,      // Zeitstempel in Mikrosekunden
        }
        // Definitionen für KeyModifiers, KeyState, PointerButtonEvent etc. folgen
        ```

    Die klare Definition dieser Systemschicht-APIs ist entscheidend, da sie die Brücke zwischen der plattformunabhängigen UI-Logik und den plattformspezifischen Implementierungen schlägt.
```

---

**Teil 5: Verfeinerte Schichtübergreifende Schnittstellenprinzipien und Fortgeschrittene Themen**

Dieser Abschnitt behandelt übergreifende Prinzipien und fortgeschrittene Aspekte, die für das Design und die Implementierung der internen Schnittstellen von NovaDE von Bedeutung sind.

- 5.1. Asynchrone Operationen und UI-Synchronisation
    
    Die Reaktionsfähigkeit der Benutzeroberfläche ist ein primäres Ziel. Viele Operationen, insbesondere solche, die Netzwerkzugriffe, Dateisystemoperationen oder komplexe Berechnungen beinhalten (z.B. Aufrufe an AIInteractionLogicService oder MCPClientInterface), sind inhärent asynchron. Es ist unerlässlich, dass die UI nicht blockiert, während auf den Abschluss dieser Operationen gewartet wird.
    
    - **Muster für von der UI initiierte asynchrone Aufrufe:**
        
        - Die UI-Schicht ruft eine `async fn` Methode eines Domänen- oder Systemdienstes auf (z.B. `async fn perform_long_task() -> Result<Data, ServiceError>;`).
        - Um ein Blockieren des UI-Threads zu verhindern, sollte die UI-Komponente, die die Operation initiiert (z.B. bei einem Button-Klick), die asynchrone Funktion in einer separaten Task ausführen. In einer `gtk4-rs` Umgebung wird hierfür typischerweise `tokio::spawn` (oder ein äquivalenter Executor) verwendet, falls die Service-Methode selbst `Send` ist, oder `glib::MainContext::spawn_local` für nicht-`Send` Futures, die auf dem Hauptthread laufen müssen, aber dennoch asynchron sind.
    - **Übermittlung von Ergebnissen/Fehlern an den UI-Thread:**
        
        - **`glib::MainContext::spawn_local`:** Dies ist der bevorzugte Mechanismus in `gtk4-rs`, um UI-Aktualisierungen sicher auf dem Haupt-GTK-Thread zu planen, sobald eine asynchrone Operation abgeschlossen ist.3
            - **Ablaufbeispiel:**
                1. Eine UI-Komponente (z.B. ein Button-Klick-Handler) startet eine Tokio-Task (`tokio::spawn`).
                2. Diese Tokio-Task führt `await` auf den Aufruf des Domänen- oder Systemdienstes aus.
                3. Nach Abschluss der Operation (ob `Ok` oder `Err`) verwendet die Tokio-Task `main_context.spawn_local(...)`, um eine Closure zu planen.
                4. Diese Closure, die nun garantiert auf dem Hauptthread ausgeführt wird, aktualisiert die GTK-Widgets mit dem Ergebnis oder dem Fehler.
        - **Kanäle (z.B. `tokio::sync::mpsc`, `async_channel`):**
            - Der UI-Thread hält das `Receiver`-Ende des Kanals. Die asynchrone Task (z.B. in einem Tokio-Thread) hält das `Sender`-Ende.
            - Nach Abschluss der asynchronen Operation wird das Ergebnis oder der Fehler über den Kanal gesendet.
            - Der UI-Thread, typischerweise in einer `glib::idle_add_local`-Schleife oder einem `glib::MainContext::spawn_local`-Future, das den Receiver abfragt, verarbeitet eingehende Nachrichten und aktualisiert die Widgets.
            - Es ist wichtig zu beachten, dass `glib::MainContext::channel()` in neueren GLib-Versionen entfernt wurde.39 Der empfohlene Ansatz ist die Verwendung von Standard-Async-Kanälen (wie `async_channel` oder `tokio::sync::mpsc`) und dann `glib::MainContext::default().spawn_local()`, um die Ergebnisse auf den Hauptthread für UI-Aktualisierungen zu bringen. Diese Änderung vereinfacht die Codebasis, da sie auf etablierte asynchrone Rust-Patterns setzt, anstatt eine GLib-spezifische Kanallösung zu verwenden. Die Schnittstellendefinition selbst schreibt diese interne UI-Implementierung nicht vor, aber die Dokumentation für UI-Entwickler, die diese asynchronen APIs konsumieren, sollte dieses Muster empfehlen.
        - **Fehlerbehandlung in asynchroner UI:** Fehler von asynchronen Operationen müssen ebenfalls auf den Hauptthread gemarshallt und angemessen dargestellt werden. Dies kann beispielsweise durch die Anzeige eines `gtk::AlertDialog` 6 oder einer nicht-modalen Benachrichtigung geschehen.
    - **Tabelle: Asynchrones Interaktionsmuster**
        

|   |   |   |   |   |
|---|---|---|---|---|
|**Schritt**|**Aktion**|**Thread**|**Mechanismus**|**Anmerkungen**|
|1|Benutzerinteraktion|UI-Thread|GTK-Signal|z.B. Button-Klick|
|2|Asynchronen Aufruf initiieren|UI-Thread|`tokio::spawn` (für `Send` Futures)|Startet eine neue Task für den Service-Aufruf, um den UI-Thread nicht zu blockieren.|
|3|Service-Methode ausführen|Worker-Thread (Tokio)|`await service.method()`|Ruft die asynchrone Methode des Domänen-/Systemdienstes auf.|
|4a|Ergebnis/Fehler senden (Kanal-Ansatz)|Worker-Thread (Tokio)|`sender.send(result).await`|Sendet das Ergebnis an einen Kanal, der vom UI-Thread überwacht wird.|
|4b|UI-Update planen (Direkter Spawn-Ansatz)|Worker-Thread (Tokio)|`main_context.spawn_local(async move {... })`|Plant direkt eine Closure zur UI-Aktualisierung auf dem Hauptthread.|
|5a|Empfangen & Verarbeiten (Kanal-Ansatz)|UI-Thread|`receiver.recv().await` (in `spawn_local`)|Die Closure in `spawn_local` empfängt die Nachricht und aktualisiert GTK-Widgets.|
|5b|UI-Update ausführen (Direkter Spawn-Ansatz)|UI-Thread|Ausführung der Closure|Die geplante Closure wird ausgeführt und aktualisiert GTK-Widgets.|

```
    Diese Tabelle verdeutlicht das Threading-Modell und den Kommunikationsfluss für asynchrone Operationen, die von der UI initiiert werden. Dies ist ein häufiger Bereich für Komplexität und Fehlerquellen, daher ist ein klares Verständnis dieses Musters entscheidend.
```

- 5.2. Ereignissystem: Granulare Semantik
    
    Das Ereignissystem ist ein Rückgrat für die reaktive Natur von NovaDE. Eine präzise Definition der Ereignissemantik ist unerlässlich.
    
    - **Ereignisdefinition:** Jede Ereignisstruktur (`struct`) muss vollständig definiert sein, einschließlich aller Felder und deren exakter Rust-Typen. Die Nutzdaten (Payload) eines Ereignisses sollten umfassend sein, um es den Konsumenten zu ermöglichen, ohne zusätzliche Abfragen an den Ereigniserzeuger zu reagieren. Beispielsweise sollte ein `ThemeChangedEvent` nicht nur eine ID des neuen Themes enthalten, sondern direkt die `AppliedThemeState`-Struktur, die alle relevanten Details des neuen Themes beinhaltet. Dies minimiert den Kommunikationsaufwand und verbessert die Performance.
    - **Payload-Serialisierung:** Für die interne Kommunikation zwischen den Schichten innerhalb desselben Prozesses ist eine direkte Übergabe von Rust-Strukturen ausreichend und performant. Sollten Ereignisse jedoch zukünftig über Prozessgrenzen hinweg kommuniziert werden (z.B. über D-Bus an externe Komponenten), muss ein Serialisierungsformat (z.B. Serde mit JSON/CBOR oder GVariant für D-Bus) definiert werden. Die aktuellen Ereignisstrukturen sollten mit `#` annotiert werden, um diese zukünftige Erweiterbarkeit vorzubereiten.
    - **Ereignisgarantien:**
        - **Reihenfolge:** Innerhalb eines bestimmten Geltungsbereichs (z.B. Ereignisse von einem einzelnen Service) ist typischerweise davon auszugehen, dass Ereignisse in der Reihenfolge empfangen werden, in der sie ausgelöst wurden, vorausgesetzt, der Emitter ist single-threaded oder verwendet geeignete Synchronisationsmechanismen. Für einen systemweiten Event-Bus kann die globale Reihenfolge komplexer sein und hängt von der Implementierung des Busses ab.
        - **Zustellung:** Für einen In-Prozess-Event-Bus wird typischerweise eine "At-least-once"-Zustellung angestrebt, solange die Abonnenten aktiv sind und keine Fehler im Bus selbst auftreten. Garantien für "Exactly-once" sind komplexer zu implementieren.
    - **Abonnement-Mechanismus:**
        - Es muss eine klare API für Komponenten geben, um spezifische Ereignistypen zu abonnieren. Dies könnte über einen zentralen `EventBus` erfolgen: `event_bus.subscribe::<ThemeChangedEvent>(|event: &ThemeChangedEvent| { /* UI-Logik hier */ });`.
        - Der Event-Bus sollte Filterfunktionen unterstützen, z.B. das Abonnieren von `WorkspaceEvent` nur für eine bestimmte `workspace_id` oder das Filtern von `SettingChangedEvent` nach einem bestimmten `setting_key`.
        - Die zurückgegebenen `tokio::sync::broadcast::Receiver` von den `subscribe_to_..._events` Methoden der Service-Traits sind ein Beispiel für einen solchen Mechanismus.
    - **Broadcasting vs. Gezielte Ereignisse:** Die meisten Domänen- und Systemereignisse werden als Broadcast an alle interessierten Abonnenten gesendet. Gezielte Ereignisse (nur an einen bestimmten Listener) sind seltener und erfordern einen komplexeren Registrierungs- und Routing-Mechanismus. Für die interne Schichtkommunikation ist Broadcasting meist ausreichend.
    - Die Reaktivität der Benutzeroberfläche hängt stark von einem wohldefinierten und zuverlässigen Ereignissystem ab. Granulare Ereignis-Payloads sind der Schlüssel zur Minimierung von Daten-Refetching und zur Gewährleistung einer performanten UI. GTK4-Signale und Ereignisbehandlungsmechanismen werden intern von der UI-Schicht verwendet, um auf diese anwendungsweiten Ereignisse zu reagieren, die von den unteren Schichten über den Event-Bus oder direkte Abonnements empfangen werden.
- **5.3. API-Versionierung und Evolution (Kurzer Hinweis)**
    
    - APIs unterliegen im Laufe der Zeit Änderungen. Um die Stabilität und Wartbarkeit des Systems langfristig zu gewährleisten, sollten Strategien für die API-Evolution berücksichtigt werden:
        - **Hinzufügen neuer Methoden zu Traits:** Dies ist für bestehende Implementierer nicht-brechend, wenn die neuen Methoden Standardimplementierungen haben (obwohl dies für Kern-APIs seltener der Fall ist).
        - **Hinzufügen neuer optionaler Felder zu Strukturen/Event-Payloads:** Erfordert eine sorgfältige Behandlung durch die Konsumenten (z.B. Verwendung von `Option<T>` und `#[serde(default)]`).
        - **Einführung neuer Versionen von Traits (z.B. `ThemingEngineV2`):** Für signifikante, brechende Änderungen ist dies der sauberste Ansatz.
    - Diese Überlegungen sind zunächst auf hoher Ebene, aber wichtig für die langfristige Gesundheit des Projekts. Eine klare Dokumentation von Änderungen und Deprecation-Policys ist ebenfalls entscheidend.

---

**Teil 6: Schnittstellendetails UI-Komponenten-spezifisch**

Dieser Abschnitt widmet sich der ultra-granularen Definition der Interaktionen spezifischer UI-Komponenten mit den zuvor definierten Domänen- und System-APIs. Er beleuchtet, wie Schlüsselkomponenten der Benutzeroberfläche diese Schnittstellen nutzen, um ihre Funktionalität zu realisieren.

- **6.1. Window Manager Frontend (`ui::window_manager_frontend`)**
    - **Verantwortlichkeiten:** Anzeige von Fensterrepräsentationen (z.B. in einer Taskleiste, Alt-Tab-Umschalter), Bearbeitung von Fokusanfragen für Fenster von UI-Elementen.
    - **Interaktion mit `system::compositor_iface`:**
        - Konsumiert `system::compositor::WindowCreatedEvent`, `WindowClosedEvent`, `WindowFocusedEvent`, `WindowUnfocusedEvent`, `WindowPropertiesChangedEvent`, um seine interne Liste der Fenster und deren Zustände zu aktualisieren.
        - Ruft `compositor_iface.request_focus(window_id)` auf, wenn der Benutzer ein Fenster auswählt (z.B. durch Klick in der Taskleiste).
        - Ruft `compositor_iface.set_window_state(window_id, CompositorWindowState::Minimized)` oder `close_window(window_id)` auf, basierend auf Benutzeraktionen.
    - **Interne UI-Modelldatenstrukturen:**
        - `UIWindowRepresentation { id: core::types::WindowId, title: String, app_id: core::types::AppId, icon_name: Option<String>, // Name für Themed Icon gtk_icon: Option<gtk::gdk_pixbuf::Pixbuf>, // Geladenes Icon is_focused: bool, is_minimized: bool, workspace_tag: Option<String> /* z.B. "WS1" */ }`
    - **GTK-Implementierungshinweise:**
        - Könnte `gtk::ListView` 40 oder `gtk::FlowBox` 45 mit benutzerdefinierten Widgets für jedes Fenster verwenden. Jedes benutzerdefinierte Widget würde an eine `UIWindowRepresentation` binden.
        - Für die Fenster-Thumbnails oder Live-Vorschauen (wie in einem Alt-Tab-Switcher) sind Wayland-spezifische Protokolle oder Compositor-Features notwendig. Die Systemschicht müsste eine API bereitstellen, um solche Texturen oder Oberflächen-Handles zu liefern, die dann in einem GTK-Widget (ggf. ein spezielles Wayland-Oberflächen-Widget oder ein `gtk::Picture` mit aktualisierten Snapshots) dargestellt werden könnten. Wakefield 47 ist ein Beispiel für einen GTK-basierten Compositor, der Client-Oberflächen in GTK-Widgets einbetten kann; ähnliche Konzepte könnten für Vorschauen relevant sein, auch wenn NovaDE kein verschachtelter Compositor ist. Die `ext-foreign-toplevel-list-v1` und `wlr-foreign-toplevel-management-unstable-v1` Protokolle 17 sind fundamental für Taskleisten, um Fensterlisten und Metadaten zu erhalten. Die Systemschicht abstrahiert diese Protokolle, und die UI konsumiert die `WindowInfo`-Strukturen.
    - **Tabellen:**
        - **Interaktion `ui::window_manager_frontend` mit `system::compositor_iface`**

|   |   |   |   |
|---|---|---|---|
|**UI-Aktion**|**Aufgerufene compositor_iface-Methode**|**Konsumierte CompositorEvents**|**Aktualisierte UI-Daten**|
|Fenster in Taskleiste anklicken|`request_focus(window_id)`|`WindowFocusedEvent`, `WindowUnfocusedEvent`|`UIWindowRepresentation::is_focused`|
|Fenster minimieren|`set_window_state(id, Minimized)`|`WindowPropertiesChangedEvent` (mit `is_minimized = true`)|`UIWindowRepresentation::is_minimized`|
|Neues Fenster erscheint|-|`WindowCreatedEvent`|Neue `UIWindowRepresentation`|
|Fenstertitel ändert sich|-|`WindowPropertiesChangedEvent` (mit `changed_title`)|`UIWindowRepresentation::title`|

- **6.2. Desktop Shell (`ui::shell`) - Panels, Docks**
    
    - **Verantwortlichkeiten:** Anzeige von Panels (oben, unten, seitlich), Docks, Arbeitsbereichswechslern, Uhr, Systemindikatoren (Netzwerk, Lautstärke, Akku etc.).
    - **Interaktion mit `gtk4-layer-shell` (über Systemschicht-Abstraktion):**
        - Die Systemschicht stellt eine API bereit, z.B. `system::desktop_shell_iface`, um Layer-Oberflächen zu erstellen und zu verwalten. Diese API kapselt die direkte Verwendung von `gtk4-layer-shell`.9
        - `fn create_layer_surface(&self, params: LayerSurfaceParams) -> Result<LayerSurfaceHandle, ShellError>;`
            - `LayerSurfaceParams { window_handle: gtk::WindowHandle, // Das GTK-Fenster, das als Layer-Oberfläche dient anchor: LayerShellAnchor, // Bitflags: Top, Bottom, Left, Right exclusivity: LayerShellExclusivity, // Exclusive, OnDemand keyboard_interactivity: LayerShellKeyboardInteractivity // None, Exclusive, OnDemand margins: (i32, i32, i32, i32) // top, right, bottom, left }`
        - Die UI verwendet dieses `LayerSurfaceHandle`, um ihre GTK-Panel- oder Dock-Widgets in der Layer-Oberfläche zu positionieren und zu verankern.
        - Die Verwendung von `gtk4-layer-shell` ist entscheidend für die korrekte Integration von Panels und Docks in Wayland-Umgebungen, da es Apps erlaubt, sich über oder unter normalen Fenstern zu positionieren und Platz auf dem Bildschirm zu reservieren.9
    - **Arbeitsbereichswechsler:**
        - Konsumiert `domain::workspaces::WorkspaceEvent`-Ereignisse vom `WorkspaceManager`.
        - Zeigt `domain::workspaces::WorkspaceDescriptor`-Daten an (Name, Icon, aktive Fenster).
        - Ruft `WorkspaceManager::set_active_workspace(workspace_id)` bei Benutzerinteraktion auf.
        - GTK-Implementierung: Könnte eine `gtk::Box` mit `gtk::ToggleButton`s oder benutzerdefinierten gezeichneten Elementen für jeden Arbeitsbereich sein. `chunks-rs` 50 und allgemeine Pager-Beispiele 51 zeigen Konzepte für die Implementierung von Arbeitsbereichswechslern in GTK.
    - **Uhr / Kalenderanzeige:**
        - Verwendet `core::utils::time_utils` (basierend auf `chrono`) zur Formatierung von Datum und Uhrzeit.
        - Kann ein `gtk::Calendar` in einem `gtk::Popover` anzeigen, wenn auf die Uhr geklickt wird. Das Popover wird relativ zum Uhr-Widget positioniert.
    - **Systemindikatoren (Lautstärke, Netzwerk, Akku, Bluetooth etc.):**
        - **Lautstärke:**
            - Konsumiert `system::audio::AudioDeviceChangedEvent`, `AudioStreamVolumeChangedEvent` etc. vom `system::audio_iface`.
            - Zeigt aktuelle Lautstärke mit `gtk::Scale` (als Slider) oder `gtk::VolumeButton` 52 an.
            - Sendet `system::audio_iface::set_device_volume()` bei Benutzerinteraktion.
        - **Netzwerk:**
            - Konsumiert `system::dbus::NetworkStateChangedEvent`, `AccessPointsChangedEvent` vom `system::dbus::networkmanager_client_iface`.
            - Zeigt Verbindungsstatus (Icon), SSID, verfügbare Netzwerke in einem `gtk::Popover` mit einer `gtk::ListBox` an.
            - Ruft `system::dbus::networkmanager_client_iface::connect_to_access_point()` auf.
        - **Akku:**
            - Konsumiert `system::dbus::UPowerDeviceChangedEvent` vom `system::dbus::upower_client_iface`.
            - Zeigt Akkustand (Icon, Prozent) und geschätzte Restlaufzeit an.
        - **Bluetooth:**
            - Konsumiert Ereignisse von einem `system::dbus::bluez_adapter_iface` (analog zu NetworkManager).
            - Zeigt Bluetooth-Status, gekoppelte und verfügbare Geräte an.
            - Ruft Methoden wie `pair_device()`, `connect_device()` auf der BlueZ-Schnittstelle auf.20
        - **StatusNotifierItem/Ayatana Indicators (System Tray):**
            - Die Implementierung eines System Trays unter Wayland ist komplex. Die Spezifikation `StatusNotifierItem` (SNI) 48 ist der De-facto-Standard, der über D-Bus funktioniert.
            - Die `ui::shell` müsste als `StatusNotifierHost` agieren. Dies beinhaltet das Lauschen auf den D-Bus nach Diensten, die `StatusNotifierWatcher.RegisterStatusNotifierItem` aufrufen, und dann die Interaktion mit jedem `StatusNotifierItem` über dessen D-Bus-Schnittstelle, um Icon, Tooltip, Menü etc. abzurufen und darzustellen.
            - Die UI-Elemente für jedes Tray-Icon (typischerweise ein `gtk::Button` mit einem `gtk::Image` und einem `gtk::MenuButton` oder `gtk::Popover` für das Kontextmenü) würden dynamisch in einem Bereich des Panels erstellt.
            - Alternative Ansätze oder Herausforderungen unter Wayland werden diskutiert.58 NovaDE wird sich auf die `StatusNotifierItem`-Spezifikation konzentrieren, da sie am weitesten verbreitet ist. Die `libayatana-indicator`-Bibliothek 63 bietet eine Implementierung, die als Referenz dienen kann, obwohl NovaDE seine eigene D-Bus-Interaktion wahrscheinlich direkt mit `zbus` oder `gio::DBusConnection` implementieren würde, abstrahiert durch die Systemschicht.
            - Die Systemschicht würde eine `system::status_notifier_host_iface` bereitstellen, die Ereignisse wie `StatusNotifierItemRegisteredEvent { service_name: String, object_path: String }` und `StatusNotifierItemUnregisteredEvent` aussendet. Die UI würde diese abonnieren und dann die `system::status_notifier_item_proxy_iface` verwenden, um mit den einzelnen Items zu interagieren (z.B. `get_icon_pixmap()`, `get_tooltip()`, `activate()`, `context_menu()`).
- **6.3. Benachrichtigungs-Popups (`ui::notification_popups`)**
    
    - **Verantwortlichkeiten:** Anzeige von transienten Benachrichtigungs-Popups.
    - **Interaktion mit `domain::NotificationService`:**
        - Konsumiert `domain::notifications::NotificationPostedEvent`.
        - Zeigt die `domain::notifications::Notification`-Daten (Zusammenfassung, Textkörper, Icon, Aktionen) an.
    - **Interaktion mit `gtk4-layer-shell` (über Systemschicht-Abstraktion):**
        - Verwendet das Layer-Shell-Protokoll, um Benachrichtigungs-Popups an einer bestimmten Bildschirmkante (z.B. oben rechts) ohne Interferenz mit anderen Fenstern zu positionieren. Dies ist entscheidend für nicht-invasive Benachrichtigungen.9
        - Die Systemschicht stellt eine Methode bereit, z.B. `system::desktop_shell_iface::create_notification_layer_surface(params: NotificationSurfaceParams) -> Result<LayerSurfaceHandle, ShellError>;`, wobei `NotificationSurfaceParams` Details wie Anker, Dauer und Größe enthalten kann.
    - **Benutzerinteraktion:**
        - Klick auf eine Benachrichtigung könnte `NotificationService::dismiss_notification(id)` aufrufen oder eine Standardaktion auslösen (falls definiert).
        - Klick auf einen Aktionsbutton in einer Benachrichtigung (z.B. "Antworten", "Archivieren") resultiert in einem Aufruf an `NotificationService::invoke_action_on_notification(notification_id, action_id)`, was wiederum ein `domain::notifications::NotificationActionInvokedEvent` auslösen kann, auf das andere Teile der Anwendung reagieren können.
    - **GTK-Implementierungshinweise:**
        - Jedes Popup ist ein eigenes `gtk::Window`, das als Layer-Oberfläche konfiguriert ist. Es enthält typischerweise `gtk::Image` für das Icon, `gtk::Label` für Text und `gtk::Button` für Aktionen.
        - Mehrere Popups müssen verwaltet werden (z.B. in einer Warteschlange oder gestapelt auf dem Bildschirm).65
    - NovaDE wird seine eigenen Benachrichtigungen zeichnen, um volle Kontrolle über das Erscheinungsbild und die Integration mit dem Theming-System zu haben, anstatt sich auf einen externen Freedesktop-Benachrichtigungsdaemon zu verlassen.67 Der `domain::NotificationService` dient als zentrale Anlaufstelle und leitet die `NotificationPostedEvent` an die `ui::notification_popups`-Komponente weiter.
- **6.4. Anwendungsstarter (z.B. Dock, Anwendungsraster)**
    
    - **Verantwortlichkeiten:** Anzeige verfügbarer Anwendungen, Starten von Anwendungen.
    - **Interaktion mit `system::app_launcher_iface`:**
        - Ruft `system::app_launcher_iface::list_applications()` auf, um eine `Vec<system::applications::ApplicationEntry>` zu erhalten.
        - Zeigt diese Einträge an (Icon, Name). Die Icons werden basierend auf dem Icon-Namen und dem aktuellen Theme geladen.
        - Ruft `system::app_launcher_iface::launch_application(app_id, files_to_open: Option<Vec<PathBuf>>)` bei Benutzeraktivierung (Klick) oder per Drag-and-Drop auf.
    - **Interne UI-Modelldatenstrukturen:**
        - `UIAppEntry { id: String, // Desktop-Datei-ID oder AppStream-ID name: String, generic_name: Option<String>, comment: Option<String>, icon_name: Option<String>, gtk_icon: Option<gtk::gdk_pixbuf::Pixbuf>, categories: Vec<String>, keywords: Vec<String> }`
        - Die `system::app_launcher_iface` würde intern `freedesktop_entry_parser` 71 oder ähnliche Mechanismen verwenden, um `.desktop`-Dateien zu parsen und die `ApplicationEntry`-Daten bereitzustellen. Die UI konsumiert diese abstrahierten Daten.
    - **GTK-Implementierungshinweise:**
        - `gtk::FlowBox` 45 oder `gtk::GridView` (mit `gtk::SignalListItemFactory` 42) eignen sich gut zur Anzeige von App-Icons.
        - Jedes Icon könnte ein benutzerdefiniertes `gtk::Button` sein, das ein `gtk::Image` und optional ein `gtk::Label` enthält.
    - **Drag-and-Drop zum Starten:**
        - App-Icons fungieren als `GtkDropTarget`.74
        - Bei einem Drop werden Datei-URIs/Pfade extrahiert und an `system::app_launcher_iface::launch_application(app_id, Some(dropped_files))` übergeben.
        - Die `system::app_launcher_iface` verwendet intern `gio::AppInfo` und verwandte APIs, um Anwendungen zu starten [76,
    
    
# Technische Spezifikation: Benutzeroberflächenschicht (UI Layer)**

**I. Einleitung und Zweck**

Dieses Dokument definiert die detaillierten technischen Spezifikationen für die Benutzeroberflächenschicht (UI Layer) der zu entwickelnden Linux-Desktop-Umgebung. Es dient als direkte und verbindliche Implementierungsgrundlage für das Entwicklungsteam. Die UI-Schicht ist die dem Benutzer zugewandte Ebene und verantwortlich für die gesamte visuelle Darstellung sowie die Entgegennahme und Verarbeitung direkter Benutzerinteraktionen. Sie baut auf dem GTK4-Toolkit auf und arbeitet eng mit den darunterliegenden System- und Domänenschichten zusammen, um eine kohärente, intuitive und leistungsstarke Benutzererfahrung zu schaffen, die sich durch eine moderne, dunkle Ästhetik mit benutzerdefinierbaren Akzentfarben auszeichnet. Alle hier getroffenen Festlegungen sind das Ergebnis vorheriger Analysen und Designentscheidungen des Gesamtprojekts.

**II. Architektonische Einordnung und Verantwortlichkeiten**

Die UI-Schicht bildet die oberste Ebene der etablierten vier-schichtigen Architektur:

1. **Kernschicht (Core Layer):** Stellt fundamentale Datentypen, Dienstprogramme, Basiskonfigurationen, Logging-Mechanismen und grundlegende Fehlerdefinitionen bereit.
2. **Domänenschicht (Domain Layer):** Beinhaltet die Kernlogik und Geschäftsregeln der Desktop-Umgebung, wie Workspace-Verwaltung ("Spaces"), das Theming-System, Logik für KI-Interaktionen inklusive Einwilligungsmanagement, Verwaltung von Benachrichtigungen und die Definition von Richtlinien für das Fenstermanagement.
3. **Systemschicht (System Layer):** Implementiert die Interaktion mit dem zugrundeliegenden Betriebssystem und externen Diensten. Dies schließt den Wayland-Compositor, die Eingabeverarbeitung, die Kommunikation über D-Bus mit Systemdiensten, die Implementierung von Wayland-Protokollen, die XWayland-Integration, die MCP-Client-Implementierung und die Interaktion mit XDG Desktop Portals ein.
4. **Benutzeroberflächenschicht (User Interface Layer):**
    - **Primäre Verantwortlichkeit:** Darstellung aller visuellen Elemente der Desktop-Umgebung und Handhabung der direkten Interaktion mit dem Benutzer.
    - **Kernkomponenten:** Umfasst die Shell-UI (bestehend aus Panels/Leisten, dem Schnellaktionsdock, der intelligenten Tab-Leiste pro "Space", dem Workspace-Switcher und dem Quick-Settings-Panel), das Control Center für Systemeinstellungen, das Widget-System inklusive der adaptiven Seitenleisten, den Übersichtsmodus für Fenster- und Workspace-Management, die kontextuelle Befehlspalette und die Speed-Dial-Startansicht.
    - **Technologische Basis:** Verwendet das GTK4-Toolkit und die Rust-Programmiersprache.
    - **Interaktionen:**
        - **Mit der Systemschicht:** Zur Steuerung und Abfrage von Fensterzuständen (Positionierung, Größe, Sichtbarkeit), zum Empfang von Eingabeereignissen (Tastatur, Maus, Touch), zur Interaktion mit Systemdiensten (z.B. Netzwerkstatus, Energieverwaltung, Audio-Kontrolle) und zur Nutzung von Wayland-Protokollen für spezielle UI-Elemente (z.B. Panels via `wlr-layer-shell-unstable-v1`).
        - **Mit der Domänenschicht:** Zur Visualisierung von Zuständen (z.B. aktuelles Theme, Struktur und Inhalt von "Spaces", Liste der Benachrichtigungen, globale Einstellungen) und zum Auslösen von Geschäftslogik (z.B. Änderung einer Einstellung, Auswahl eines Themes, Erstellung eines neuen "Space").
        - **Mit dem Benutzer:** Entgegennahme von Eingaben über grafische Elemente und Weiterleitung entsprechender Befehle an die System- oder Domänenschicht. Präsentation von Informationen und Systemfeedback.

**III. Technologie-Stack (UI-Schicht-Spezifika)**

|Bereich|Technologie/Standard|Kernaspekte für die UI-Schicht|
|:--|:--|:--|
|GUI-Toolkit|GTK4|Moderne Widget-Bibliothek; Wayland-First-Ansatz; offizielle und gepflegte Rust-Bindings (`gtk4-rs`); CSS-basiertes Theming zur Umsetzung des Token-basierten Designsystems; Fähigkeit zum dynamischen Wechsel von Themes und Styles zur Laufzeit; Unterstützung für Composite Templates (`*.ui`-Dateien) zur Trennung von Layout und Code.|
|Programmiersprache|Rust|Gewährleistet Speichersicherheit und Performance, auch für komplexe UI-Logik; gute Integration mit GTK4 über `gtk4-rs`.|
|Wayland-Protokolle (Client)|`wlr-layer-shell-unstable-v1`, `wlr-foreign-toplevel-management-unstable-v1`|`wlr-layer-shell` wird für die korrekte Positionierung und das Verhalten von Shell-UI-Elementen wie Panels und Docks benötigt, die über oder unter normalen Anwendungsfenstern liegen. `wlr-foreign-toplevel-management` ist essentiell für Funktionen wie den Übersichtsmodus oder Taskleisten-Äquivalente, um Informationen über Fenster anderer Anwendungen zu erhalten und diese zu steuern.|
|Inter-Prozess-Kommunikation|D-Bus|Ermöglicht die Kommunikation mit Systemdiensten (Netzwerk, Energie, Audio etc. via Systemschicht) und potenziell mit internen Diensten der Desktop-Umgebung für entkoppelte Komponenteninteraktion.|
|Theming-Implementierung|Token-basiertes System via GTK4 CSS Custom Properties (`var()`)|Ermöglicht die dynamische Anwendung von Design-Tokens (Farben, Schriftgrößen, Abstände etc.), die von der Domänenschicht (`domain::theming`) verwaltet werden. Änderungen an den Tokens führen zu Laufzeitaktualisierungen des UI-Erscheinungsbildes.|
|Grafische Darstellung|Über den Wayland Compositor der Systemschicht|Die UI-Schicht ist primär für die Beschreibung der Szene und die Logik der UI-Elemente zuständig. Das eigentliche Rendern der Fenster und UI-Elemente auf dem Bildschirm wird vom Compositor (basierend auf Smithay) in der Systemschicht übernommen.|
|Eingabeverarbeitung|Empfang von Wayland-Events von der Systemschicht|Die UI-Schicht empfängt bereits prozessierte Eingabeereignisse (Tastatur, Maus, Touch, Gesten) als Wayland-Events vom Compositor, der diese mittels `libinput` aufbereitet hat. Die UI-Schicht ist dann für die semantische Interpretation dieser Events im Kontext der fokussierten UI-Elemente zuständig.|

In Google Sheets exportieren

**IV. Entwicklungsrichtlinien (UI-Schicht-Ergänzungen)**

Über die allgemeinen Projektrichtlinien hinaus gelten für die UI-Schicht folgende spezifische Vorgaben:

- **GTK4 Best Practices:**
    - Konsequente Nutzung von GTK4-Idiomen und -Mustern.
    - Verwendung von Composite Templates (`*.ui`-Dateien mit `blueprint` oder XML) zur Definition des UI-Layouts, um eine klare Trennung von der Rust-Logik zu gewährleisten. Widgets werden in Rust-Code über `#[template_child]` referenziert.
    - Nutzung der `Gtk::Application`-Klasse als Einstiegspunkt und für die Verwaltung des Anwendungslebenszyklus der Desktop-Shell. Hauptfenster sind Instanzen von `Gtk::ApplicationWindow` oder spezialisierten Ableitungen.
- **Asynchrone Programmierung:**
    - Alle potenziell blockierenden Operationen (z.B. Netzwerkzugriffe für Widgets, komplexe Berechnungen, die nicht sofort abgeschlossen werden können) müssen asynchron implementiert werden, um die Reaktionsfähigkeit der UI sicherzustellen.
    - Bevorzugte Nutzung von `async/await` in Verbindung mit `glib::MainContext::spawn_local` für GTK-kompatible asynchrone Aufgaben.
- **Zustandsverwaltung:**
    - Strikte Trennung von Darstellungslogik und Anwendungszustand. Die UI-Komponenten sollen primär als Visualisierung des Zustands dienen, der von der Domänen- oder Systemschicht gehalten und bereitgestellt wird.
    - Änderungen des Zustands werden durch Benutzerinteraktionen ausgelöst und an die zuständigen unteren Schichten delegiert. Die UI aktualisiert sich dann reaktiv auf Events oder geänderte Daten von diesen Schichten.
    - Lokaler UI-Zustand (z.B. Zustand einer Animation, geöffnete Popovers) ist auf das absolute Minimum zu beschränken.
- **CSS-Styling und Theming:**
    - Verwendung eines konsistenten und modularen CSS-Namensschemas (z.B. BEM-inspiriert) für alle Widgets und UI-Elemente, um die Lesbarkeit und Wartbarkeit der Stylesheets zu verbessern.
    - Stylesheets werden dynamisch über `Gtk::CssProvider` geladen und aktualisiert, basierend auf den Vorgaben der `ui::theming_gtk`-Komponente, die wiederum von der `domain::theming`-Logik gesteuert wird.
    - Die UI-Elemente müssen so gestaltet sein, dass sie korrekt auf Änderungen der globalen Design-Tokens (Akzentfarben, Basis-Theme-Farben, Schriftarten) reagieren.
- **Barrierefreiheit (Accessibility - A11y):**
    - Barrierefreiheit ist von Beginn an ein integraler Bestandteil des UI-Designs und der Implementierung.
    - Korrekte Implementierung von Accessibility-Informationen für alle Widgets unter Verwendung der von GTK4 und ATK bereitgestellten Mechanismen (z.B. Setzen von Accessible Roles, States und Properties).
    - Sicherstellung der Tastaturnavigation für alle interaktiven Elemente.
- **Performance und Optimierung:**
    - Die UI muss auf schnelle Startzeiten und flüssige Interaktionen sowie Animationen optimiert werden.
    - Vermeidung unnötiger Neuzeichnungen (`Gtk::Widget::queue_draw` nur bei tatsächlichen Änderungen).
    - Effiziente Nutzung von GTK-Layout-Managern.
    - Regelmäßiges Profiling der UI-Performance mit geeigneten Werkzeugen (z.B. Sysprof, GTK Inspector).
- **Responsive Design-Elemente:**
    - Obwohl der primäre Fokus auf Desktop-Systemen liegt, sollen UI-Elemente und Layouts so gestaltet werden, dass sie sich in einem gewissen Rahmen an unterschiedliche Fenstergrößen und -proportionen anpassen können, ohne dass das Layout bricht.
- **UI-Teststrategie:**
    - Entwicklung von Unit-Tests für einzelne UI-Komponenten und deren Logik.
    - Evaluierung und Einsatz von Werkzeugen für Integrationstests der UI, z.B. durch Simulation von Benutzerinteraktionen auf Widget-Ebene oder Wayland-Ebene.
    - Prüfung der Möglichkeit von visuellen Regressionstests, um unbeabsichtigte Änderungen am Erscheinungsbild frühzeitig zu erkennen.
    - Accessibility-Tests zur Überprüfung der korrekten Implementierung von A11y-Features.

**V. Detaillierte Spezifikation der UI-Schicht-Komponenten**

Diese Spezifikation beschreibt die Kernkomponenten der UI-Schicht, ihre Verantwortlichkeiten, Struktur und Interaktionen, basierend auf den etablierten Designzielen und Anwendungsfällen des Projekts.

**1. Modul: `ui::shell` (Implementierung der Haupt-Shell-UI)**

- **Verantwortlichkeit:** Bereitstellung der grundlegenden Rahmenstruktur und der primären Interaktionselemente des Desktops. Nutzt GTK4 und das `wlr-layer-shell-unstable-v1`-Protokoll für die korrekte Positionierung von Elementen wie Panels auf dem Bildschirm.
    
- **Unterkomponenten:**
    
    - **1.1. `ui::shell::panel` (Kontroll- und Systemleiste(n))**
        
        - **Definition:** Eine oder mehrere Leisten, deren Position (oben/unten) durch den Benutzer konfigurierbar ist. Dient als primärer Ankerpunkt für Systeminformationen und Schnellzugriffe.
        - **Struktur:** Ein `Gtk::Box`-Container, der dynamisch Abschnitte für Module (links, zentriert, rechts) bereitstellt. Das Erscheinungsbild ist durch das globale Theme bestimmt (dunkle Basis, Akzentfarben), mit optionaler Transluzenz und einem subtilen Leuchtakzent am Rand, realisiert über CSS.
        - **Kernmodule (als eigenständige GTK4-Widgets):**
            - `AppMenuButton`: Zeigt den Namen der fokussierten Anwendung und deren Anwendungsmenü. Interagiert clientseitig mit `wlr-foreign-toplevel-management` für Informationen über die aktive Anwendung und ggf. mit D-Bus (AppMenu-Spezifikation) zur Abfrage des Menüs.
            - `WorkspaceIndicatorWidget`: Integriert den `ui::shell::workspace_switcher` (siehe 1.3) oder eine kompaktere Variante davon direkt im Panel.
            - `ClockDateTimeWidget`: Zeigt Systemdatum und -uhrzeit. Ein Klick kann ein Popover mit einem Kalender-Widget (aus `ui::widgets`) und/oder einen Schnellzugriff zum `ui::shell::notification_center` öffnen.
            - `SystemTrayEquivalentWidget`: Stellt eine moderne Alternative zu traditionellen System-Tray-Icons dar, möglicherweise durch Integration mit der `org.freedesktop.StatusNotifierWatcher`-D-Bus-Schnittstelle.
            - `QuickSettingsButtonWidget`: Ein `Gtk::Button`, der bei Klick das `ui::shell::quick_settings`-Panel öffnet.
            - `NotificationCenterButtonWidget`: Ein `Gtk::Button`, der das `ui::shell::notification_center`-Panel öffnet/schließt.
            - `NetworkIndicatorWidget`: Zeigt den aktuellen Netzwerkstatus (WLAN-Signalstärke, Kabelverbindungssymbol). Bezieht Daten vom `NetworkManagerClient` der Systemschicht.
            - `PowerIndicatorWidget`: Zeigt den Akkustand (als Icon und/oder Prozentwert) und den Ladestatus. Bezieht Daten vom `UPowerClient` der Systemschicht.
            - `AudioIndicatorWidget`: Zeigt die aktuelle Systemlautstärke und den Stummschaltungsstatus. Ein Klick öffnet Lautstärkeregler und ggf. Gerätauswahl im `QuickSettingsPanel`. Bezieht Daten vom `PipeWireClient` der Systemschicht.
        - **Interaktionen:**
            - Kommuniziert mit dem Compositor (Systemschicht) über `wlr-layer-shell-unstable-v1` zur exklusiven Reservierung des Bildschirmbereichs und Positionierung.
            - Bezieht Zustandsinformationen (Netzwerk, Energie, Audio, aktuelles Theme) von den entsprechenden Diensten der Domänen- und Systemschicht.
            - Löst Aktionen aus (z.B. Öffnen des Quick-Settings-Panels) durch Weiterleitung an zuständige Handler.
        - **Styling:** Festgelegt durch das aktive, Token-basierte Theme. CSS-Klassen ermöglichen die gezielte Anpassung der Module und ihrer Zustände (z.B. :hover, :active).
    - **1.2. `ui::shell::smart_tab_bar` (Intelligente Tab-Leiste pro "Space")**
        
        - **Definition:** Eine horizontale `Gtk::Box`-basierte Leiste, die direkt im Hauptbereich jedes "Space" angezeigt wird. Sie visualisiert die dem aktuellen "Space" zugeordneten Anwendungen, insbesondere die "angepinnten" Anwendungen, als Tabs.
        - **Struktur und Verhalten:**
            - Enthält eine dynamische Liste von `ApplicationTabWidget`-Instanzen.
            - Der aktive (fokussierte) Tab wird durch die systemweite Akzentfarbe hervorgehoben.
            - Tabs sind modern gestaltet mit abgerundeten oberen Ecken.
            - Implementiert eine Überlauf-Logik (z.B. Scrollbuttons oder Dropdown-Liste), falls mehr Tabs vorhanden sind, als angezeigt werden können.
        - **`ApplicationTabWidget` (abgeleitet von `Gtk::ToggleButton` oder `Gtk::Box` mit Event-Handling):**
            - Zeigt das Icon und den Titel der Anwendung.
            - Kann spezielle Indikatoren für Split-View-Konfigurationen anzeigen (wenn mehrere Anwendungen als ein "gepinnter" Tab-Eintrag dargestellt werden).
            - Bietet ein Kontextmenü (Rechtsklick) mit Optionen wie "Fenster schließen", "An Space anpinnen/lösen", "Fenster in neuen Space verschieben".
        - **Interaktionen:**
            - Bezieht Informationen über die Struktur der "Spaces" und die darin "angepinnten" Anwendungen vom `domain::workspaces::manager`.
            - Nutzt `wlr-foreign-toplevel-management` (clientseitig über die Systemschicht), um Informationen über laufende Fenster (Titel, App-ID, Zugehörigkeit zu "Spaces") zu erhalten und den Fokus auf eine Anwendung zu setzen, wenn deren Tab angeklickt wird.
            - Die Tabs repräsentieren primär die Fenster, die dem aktuellen "Space" zugeordnet sind. "Gepinnte" Anwendungen können als persistente Tabs dargestellt werden, die bei Klick die Anwendung starten, falls sie noch nicht läuft.
        - **Styling:** Konsistent mit der Ästhetik des Panels. Akzentfarben für den aktiven Tab und Hover-Effekte.
    - **1.3. `ui::shell::workspace_switcher` (Linke Seitenleiste für Navigation & "Spaces")**
        
        - **Definition:** Eine optional einblendbare, adaptive Seitenleiste am linken Bildschirmrand, die primär der Navigation zwischen "Spaces" dient.
        - **Struktur (`Gtk::Revealer` umschließt ein `Gtk::Box`):**
            - **Eingeklappter Zustand:** Zeigt eine vertikale Liste von `SpaceIconWidget`-Instanzen. Der aktuell aktive "Space" wird visuell hervorgehoben (z.B. durch einen Indikator in Akzentfarbe).
            - **Ausgeklappter Zustand (ausgelöst durch Mouse-Over oder eine konfigurierbare Geste):** Erweitert sich, um zusätzlich zu den Icons die Namen der "Spaces" anzuzeigen. Optional können hier auch Miniaturansichten der "Spaces" oder eine kompakte Liste der darin geöffneten Fenster dargestellt werden.
            - Beinhaltet am oberen oder unteren Ende fest positionierte Elemente wie einen globalen Such-Button (der die `ui::command_palette` öffnet), einen Button zum Öffnen des Anwendungsstarters (`ui::app_launcher`) und einen Schnellzugriff zum `ui::control_center`.
        - **`SpaceIconWidget` (abgeleitet von `Gtk::Button`):**
            - Zeigt ein Icon, das den "Space" repräsentiert. Dies kann das Icon der primär "angepinnten" Anwendung, ein vom Benutzer gewähltes Symbol oder ein Standard-Workspace-Icon sein.
            - Ein Tooltip zeigt den vollständigen Namen des "Space" an.
        - **Interaktionen:**
            - Bezieht die Liste der `Workspace`-Objekte und die ID des aktiven Workspace vom `domain::workspaces::manager`.
            - Sendet bei Klick auf ein `SpaceIconWidget` einen Befehl zum Wechseln des aktiven Workspace an den `domain::workspaces::manager`.
            - Abonniert `WorkspaceEvent`s (z.B. `WorkspaceCreated`, `WorkspaceDeleted`, `ActiveWorkspaceChanged`, `WorkspaceRenamed`) vom `domain::workspaces::manager`, um seine Darstellung dynamisch zu aktualisieren.
        - **Styling:** Dunkler, dezent transluzenter Hintergrund, um sich leicht vom Desktop-Hintergrund abzuheben. Flüssige Animationen für das Ein- und Ausklappen. Klare visuelle Hervorhebung des aktiven "Space".
    - **1.4. `ui::shell::quick_settings` (Quick-Settings-Panel)**
        
        - **Definition:** Ein aus der Systemleiste (Panel) herausklappbares `Gtk::Popover` oder ein ähnliches transientes Widget, das schnellen Zugriff auf häufig benötigte Systemeinstellungen bietet.
        - **Struktur:** Ein `Gtk::Box` mit vertikaler Anordnung von Einstellungsmodulen.
        - **Module (Beispiele, als eigenständige GTK4-Widgets):**
            - WLAN-Auswahl und -Status (`Gtk::DropDown`, `Gtk::Switch`).
            - Bluetooth-Steuerung (An/Aus, Geräte koppeln via `Gtk::Switch`, `Gtk::Button`).
            - Lautstärkeregler (`Gtk::Scale`, Stummschalt-Button).
            - Dark Mode-Umschalter (`Gtk::Switch`).
            - Bildschirmhelligkeitsregler (`Gtk::Scale`).
        - **Interaktionen:**
            - Liest den aktuellen Zustand der jeweiligen Einstellungen vom `domain::global_settings_and_state_management` oder direkt von den Systemdiensten (über D-Bus-Schnittstellen der Systemschicht, z.B. `NetworkManagerClient`, `PipeWireClient`, `system::outputs` für Helligkeit).
            - Sendet Einstellungsänderungen an die entsprechenden Dienste oder den `GlobalSettingsService`.
        - **Styling:** Konsistent mit dem Panel und dem globalen dunklen Theme. Klare, leicht bedienbare Steuerelemente.
    - **1.5. `ui::shell::quick_action_dock` (Schnellaktionsdock)**
        
        - **Definition:** Ein vom Benutzer konfigurierbarer Bereich (kann schwebend oder an einem Bildschirmrand angedockt sein), der Schnellzugriffe auf favorisierte Anwendungen, häufig benötigte Dateien oder oft ausgeführte Aktionen ermöglicht.
        - **Struktur:** Implementiert als `Gtk::FlowBox` (für flexible Anordnung) oder `Gtk::Box`. Enthält eine Liste von `DockItemWidget`-Instanzen.
        - **`DockItemWidget` (abgeleitet von `Gtk::Button`):**
            - Zeigt das Icon der Anwendung/Datei oder ein Aktionssymbol.
            - Ein Tooltip zeigt den Namen oder Pfad.
            - Bietet ein Kontextmenü (Rechtsklick) für Optionen wie "Entfernen", "Eigenschaften".
        - **Interaktionen:**
            - Die Konfiguration (Inhalt, Position, Größe) wird über den `domain::global_settings_and_state_management` Service gespeichert und geladen.
            - Drag & Drop von Anwendungen (z.B. aus dem Anwendungsstarter) oder Dateien (z.B. aus dem Dateimanager) auf das Dock, um sie hinzuzufügen. Items können innerhalb des Docks neu angeordnet werden.
            - Startet Anwendungen (via `Gtk::AppInfo::launch` oder `XDG Desktop Portals` für sandboxed Apps) oder öffnet Dateien (via `Gtk::FileLauncher` oder Desktop Portals) bei Klick.
            - Potenzial für intelligente Vorschläge: Könnte mit `domain::ai` interagieren, um kontextsensitive Aktionen oder Anwendungen vorzuschlagen, basierend auf der aktuellen Aktivität oder Tageszeit des Benutzers.
        - **Styling:** Konsistent mit der Haupt-Shell-UI. Kann optional dezente Schwebeeffekte oder einen eigenen, leicht transluzenten Hintergrund haben.
    - **1.6. `ui::shell::notification_center` (Benachrichtigungszentrum)**
        
        - **Definition:** Ein Panel oder Popover (oft aus der Systemleiste zugänglich), das eine chronologische Liste der empfangenen Benachrichtigungen sowie eine Historie anzeigt.
        - **Struktur:** Ein `Gtk::ScrolledWindow`, das eine `Gtk::ListBox` enthält. Jedes Listenelement ist ein `NotificationWidget`. Bietet globale Aktionen wie "Alle Benachrichtigungen löschen" und einen Schalter für den "Bitte nicht stören"-Modus.
        - **`NotificationWidget` (abgeleitet von `Gtk::Frame` oder `Gtk::Box`):**
            - Zeigt Icon und Namen der sendenden Anwendung.
            - Stellt die Zusammenfassung (Titel) und den detaillierten Text (Body) der Benachrichtigung dar.
            - Zeigt interaktive Aktions-Buttons, falls von der Benachrichtigung bereitgestellt.
            - Zeigt den Zeitstempel des Eingangs.
            - Bietet eine Schaltfläche zum Schließen der einzelnen Benachrichtigung.
        - **Interaktionen:**
            - Ruft Methoden des `domain::user_centric_services::NotificationService` auf, um die Liste der aktiven Benachrichtigungen (`get_active_notifications`) und die Historie (`get_notification_history`) abzurufen.
            - Sendet Befehle wie `dismiss_notification` oder `invoke_action` (wenn ein Aktions-Button geklickt wird) an den `NotificationService`.
            - Reagiert auf Events wie `NotificationPostedEvent` oder `NotificationDismissedEvent` vom `NotificationService`, um die angezeigte Liste dynamisch zu aktualisieren.
        - **Styling:** Dunkles Design, klare visuelle Trennung zwischen einzelnen Benachrichtigungen. Akzentfarben können für Aktions-Buttons oder zur Hervorhebung der Dringlichkeit verwendet werden.

**2. Modul: `ui::control_center` (Zentrale Einstellungsverwaltung)**

- **Verantwortlichkeit:** Bereitstellung einer zentralen, grafischen Oberfläche für alle Systemeinstellungen der Desktop-Umgebung. Die Implementierung erfolgt mit GTK4.
- **Struktur:** Ein `Gtk::ApplicationWindow`, das eine Hauptnavigationsstruktur (z.B. eine `Gtk::StackSidebar` oder eine `Gtk::ListBox` als Seitenleiste) und einen Hauptbereich (`Gtk::Stack`) zur Anzeige der Einstellungsmodule für die ausgewählte Kategorie (z.B. "Erscheinungsbild", "Netzwerk", "Audio", "Energieverwaltung") verwendet.
- **Einstellungsmodule (Beispiele, implementiert als eigenständige GTK4-Widgets, die `Gtk::StackPage`s füllen):**
    - **`AppearanceSettingsWidget`:**
        - Theme-Auswahl: `Gtk::DropDown` zur Auswahl des aktiven Themes. Die Liste der Themes wird von `domain::theming::ThemingEngine::get_available_themes()` bezogen.
        - Farbschema-Auswahl: `Gtk::Switch` oder `Gtk::SegmentedButton` für "Hell", "Dunkel", "Automatisch".
        - Akzentfarben-Auswahl: `Gtk::ColorButton` oder ein Grid mit vordefinierten Farbpaletten, die von `ThemeDefinition::supported_accent_colors` des aktiven Themes stammen.
        - Einstellungen für Schriftarten (Familie, Größe für Standard, Monospace, Dokument), Icon-Theme, Cursor-Theme.
        - Schalter für Desktop-Animationen (`Gtk::Switch`).
        - Regler für die Skalierung der Benutzeroberfläche (`Gtk::Scale`).
        - Viele dieser Einstellungen bieten eine Live-Vorschau ihrer Auswirkungen.
    - **`NetworkSettingsWidget`:** Listet verfügbare Netzwerkadapter (LAN, WLAN). Zeigt verfügbare WLAN-Netzwerke an und ermöglicht deren Konfiguration (Passworteingabe etc.). VPN-Verwaltung.
    - **`AudioSettingsWidget`:** Auswahl von Standard-Audioein- und -ausgabegeräten. Lautstärkeregler für verschiedene Kanäle. Testmöglichkeit für Lautsprecher. Mikrofoneinstellungen.
    - **`PowerSettingsWidget`:** Konfiguration für Bildschirm-Timeout (im Akku- und Netzbetrieb), Verhalten beim Schließen des Laptop-Deckels, automatische Suspend-Zeiten. Option zur Anzeige des Batterieprozentsatzes.
    - **`MouseTouchpadSettingsWidget`:** Einstellungen für Mausbeschleunigungsprofil, Zeigerempfindlichkeit, natürliche Scrollrichtung (Maus und Touchpad), Tap-to-Click für Touchpads, Touchpad-Zeigergeschwindigkeit. Einstellungen für Tastaturwiederholungsverzögerung und -rate.
- **Interaktionen:**
    - Liest den aktuellen Zustand aller Einstellungen vom `domain::global_settings_and_state_management` Service (entweder durch Abruf des gesamten `GlobalDesktopSettings`-Objekts oder durch gezielte Abfrage einzelner Einstellungen via `GlobalSettingsService::get_setting()`).
    - Sendet geänderte Einstellungswerte über `GlobalSettingsService::update_setting()` an die Domänenschicht, wo sie validiert und persistiert werden.
    - Abonniert `SettingChangedEvent` vom `GlobalSettingsService`, um die UI-Anzeige zu aktualisieren, falls Einstellungen durch andere Mechanismen (z.B. Befehlspalette, Systemänderungen) modifiziert werden.
    - Für bestimmte systemnahe Einstellungen (z.B. sofortige Anwendung einer Netzwerkverbindung, Testton für Audio) kann eine direkte Kommunikation mit den entsprechenden D-Bus-Schnittstellen der Systemschicht erfolgen, um unmittelbares Feedback oder Aktionen auszulösen, die nicht direkt über das globale Einstellungs-Backend laufen.
- **Styling:** Muss sich nahtlos in das globale Theme der Desktop-Umgebung einfügen. Klare, intuitive und leicht verständliche Anordnung der Einstellungsoptionen.

**3. Modul: `ui::widgets` (Widget-System und Adaptive Seitenleisten)**

- **Verantwortlichkeit:** Implementierung des Systems für Desktop-Widgets, einschließlich der rechten adaptiven Seitenleiste als primären Container für diese Widgets. Stellt die Logik zum Laden, Anzeigen, Konfigurieren und Verwalten von Widgets bereit.
- **Struktur:**
    - **`RightSidebarWidget` (abgeleitet von `Gtk::Revealer` oder `Gtk::Box` mit benutzerdefinierter Ein-/Ausblendlogik):**
        - Positioniert an der rechten Bildschirmkante. Kann durch Benutzeraktion (Geste, Button) ein- und ausgeklappt werden.
        - Dient als primärer Container für eine vertikal angeordnete Liste von `PlacedWidgetWidget`-Instanzen.
        - Hat einen leicht abgesetzten, dezent transluzenten Hintergrund, um sich vom Haupt-Desktop-Inhalt zu unterscheiden.
    - **`WidgetManagerService` (kein UI-Element, sondern eine logische Komponente innerhalb der UI-Schicht oder ggf. Domänenschicht):**
        - Verantwortlich für das Entdecken und Laden verfügbarer Widgets (ggf. als Plugins implementiert).
        - Verwaltet die Konfiguration der platzierten Widgets (welche Widgets sind aktiv, in welcher Reihenfolge, ihre spezifischen Einstellungen) und speichert diese über den `domain::global_settings_and_state_management` Service.
    - **`WidgetPickerPopover` (abgeleitet von `Gtk::Popover`):** Ein Popover, das dem Benutzer eine Liste aller verfügbaren Widgets anzeigt und das Hinzufügen zur Seitenleiste per Klick oder Drag & Drop ermöglicht.
    - **`PlacedWidgetWidget` (abgeleitet von `Gtk::Frame` oder `Gtk::Box`):** Ein Standard-Wrapper-Widget, das ein `ActualWidget` enthält. Bietet gemeinsame Funktionen wie einen Rahmen, einen Titel (falls vom Widget gewünscht), eine Schaltfläche zum Entfernen des Widgets und Drag & Drop-Handles zur Neuanordnung innerhalb der Seitenleiste.
    - **`ActualWidget` (Basis-Trait oder ein Enum, das von allen konkreten Widget-Implementierungen genutzt wird):**
        - Definiert die Schnittstelle, die jedes Widget implementieren muss (z.B. `fn get_gtk_widget() -> Gtk::Widget`, `fn on_settings_changed(settings: &WidgetSettings)`).
        - **Beispiele für Standard-Widgets (jeweils als eigenständiges GTK4-Widget implementiert):**
            - `ClockWidget`: Zeigt die aktuelle Uhrzeit und optional das Datum an.
            - `CalendarWidget`: Zeigt einen Monatskalender. Klicks auf Tage könnten eine detailliertere Tagesansicht oder Termine aus einer Kalenderanwendung (via D-Bus oder `XDG Desktop Portals`) anzeigen.
            - `WeatherWidget`: Zeigt aktuelle Wetterinformationen (Temperatur, Icon, Ort) und eine kurze Vorhersage. Benötigt Netzwerkzugriff, der entweder über die Systemschicht (z.B. ein HTTP-Client-Service) oder sicher über `system::mcp` (falls eine KI-Integration für Wetterdaten genutzt wird) erfolgt.
            - `SystemMonitorWidget`: Zeigt Diagramme oder Textwerte für CPU-Auslastung, RAM-Nutzung und Netzwerkverkehr. Bezieht Daten von Systemdiensten oder durch direkten Zugriff auf Systeminformationen (z.B. `/proc`, via Systemschicht).
            - `NotesWidget`: Ein einfaches Textfeld oder eine Liste für kurze Notizen. Speichert Inhalt lokal (ggf. via `domain::global_settings_and_state_management` für einfache Notizen oder eine dedizierte kleine Datenbank/Datei für umfangreichere).
            - `WebBookmarksWidget`: Zeigt eine konfigurierbare Liste von Lesezeichen an und öffnet diese im Standardbrowser.
            - `ImageFeedWidget`: Zeigt dynamisch Bilder von einer vom Benutzer konfigurierten Quelle (lokaler Ordner, Online-Feed).
            - `ColorPaletteTrendWidget`: Zeigt aktuelle oder trendige Farbpaletten an, möglicherweise basierend auf Vorschlägen des `domain::ai` Moduls (via `system::mcp`).
- **Interaktionen:**
    - Benutzer können Widgets per Drag & Drop aus dem `WidgetPickerPopover` in die `RightSidebarWidget` ziehen oder direkt hinzufügen.
    - Widgets innerhalb der Seitenleiste können per Drag & Drop neu angeordnet werden.
    - Der `WidgetManagerService` lädt Widget-Definitionen (ggf. dynamisch als Plugins zur Laufzeit) und ist für die Instanziierung der `ActualWidget`-Implementierungen zuständig.
    - Einzelne Widgets können mit verschiedenen Diensten der Domänen- oder Systemschicht interagieren, um Daten abzurufen oder Aktionen auszuführen (z.B. das `WeatherWidget` mit einem Netzwerkdienst, das `SystemMonitorWidget` mit System-APIs).
- **Styling:** Widgets sollten ein konsistentes Erscheinungsbild innerhalb der Seitenleiste haben (z.B. ähnliche Rahmen, Abstände). Der Inhalt jedes Widgets wird individuell gestaltet, passt sich aber dem globalen Theme an.

**(Fortsetzung für `ui::window_manager_frontend`, `ui::notifications_frontend`, `ui::theming_gtk`, `ui::components`, `ui::speed_dial`, `ui::command_palette` sowie `VI. Zusammenspiel` und `VII. Deployment-Überlegungen` folgt in der nächsten Antwort, um den Rahmen nicht zu sprengen.)**

# Technische Gesamtspezifikation: Ergänzung Benutzeroberflächenschicht

Dieses Dokument ergänzt die bestehende technische Gesamtspezifikation um den detaillierten Entwurf der Benutzeroberflächenschicht (User Interface Layer).

## 4. Benutzeroberflächenschicht (User Interface Layer)

Die Benutzeroberflächenschicht ist die dem Benutzer zugewandte Komponente der Desktop-Umgebung. Sie ist verantwortlich für die visuelle Darstellung, die Entgegennahme von Benutzerinteraktionen und die Präsentation von Informationen aus den unteren Schichten.

### 4.1. Zweck und Verantwortlichkeiten

* **Visuelle Darstellung:** Erzeugung und Verwaltung aller grafischen Elemente des Desktops, einschließlich Shell, Fenster, Widgets und Dialoge.
* **Benutzerinteraktion:** Entgegennahme und Verarbeitung von Eingaben über Maus, Tastatur, Touch-Gesten und andere Eingabegeräte.
* **Zustandsvisualisierung:** Darstellung von Systemzuständen (z.B. Netzwerkkonnektivität, Akkustatus), Anwendungszuständen (z.B. aktive Fenster, Benachrichtigungen) und Konfigurationen.
* **Interaktive Steuerung:** Bereitstellung von Steuerelementen zur Bedienung der Desktop-Umgebung und ihrer Funktionen.
* **Technologische Basis:** Primäre Nutzung von GTK4 und den dazugehörigen Rust-Bindings (`gtk4-rs`). Optionale, aber empfohlene Nutzung von `libadwaita` für ein konsistentes Erscheinungsbild gemäß den Adwaita-Designrichtlinien und für den Zugriff auf spezialisierte Widgets.

### 4.2. Allgemeine Prinzipien und Entwicklungsrichtlinien der UI-Schicht

* **Technologie-Stack:**
    * **GUI-Toolkit:** GTK4.
    * **Rust-Bindings:** `gtk4-rs`.
    * **Styling & Widgets:** `libadwaita` (empfohlen für konsistentes Design und zusätzliche Widgets wie `AdwApplicationWindow`, `AdwHeaderBar`, `AdwPreferencesPage`).
    * **Programmiersprache:** Rust.
* **Reaktivität und Performance:**
    * Die Benutzeroberfläche muss stets flüssig und reaktionsschnell sein.
    * Langlaufende oder potenziell blockierende Operationen (z.B. Laden von Daten, komplexe Berechnungen) müssen asynchron ausgeführt werden, um ein Einfrieren der UI zu verhindern (z.B. mittels `glib::MainContext::spawn_local` oder `tokio` Runtime in Verbindung mit `glib`).
* **Zustandsverwaltung:**
    * Klare Trennung zwischen dem Zustand der UI-Komponenten und der Anwendungslogik in den Domänen- und Systemschichten.
    * Nutzung von Events, Signalen und dem Beobachter-Muster zur Kommunikation und Synchronisation von Zuständen zwischen den Schichten.
    * Für komplexe UI-Zustände können leichtgewichtige State-Management-Ansätze in Betracht gezogen werden.
* **Komponentenbasierte Architektur:**
    * Die UI wird aus modularen, wiederverwendbaren und möglichst voneinander unabhängigen Komponenten (Widgets, Dialoge, Ansichten) aufgebaut.
* **Theming:**
    * Tiefe Integration mit dem `domain::theming`-Modul. Design-Tokens und Stildefinitionen aus der Domänenschicht werden über GTK-CSS auf UI-Elemente angewendet.
    * Dynamische Theme-Wechsel (Hell/Dunkel, Akzentfarben) zur Laufzeit müssen unterstützt werden.
* **Accessibility (A11y – Barrierefreiheit):**
    * Strikte Einhaltung der Barrierefreiheitsrichtlinien (z.B. WCAG, soweit anwendbar).
    * Umfassende Nutzung der von GTK4 bereitgestellten Accessibility-Schnittstellen (`Accessible`, `AccessibleRole`).
    * Alle UI-Elemente müssen per Tastatur bedienbar sein und korrekte Informationen für assistive Technologien (z.B. Screenreader wie Orca) bereitstellen.
* **Internationalisierung (i18n) und Lokalisierung (l10n):**
    * Alle für den Benutzer sichtbaren Zeichenketten müssen lokalisierbar sein.
    * Bevorzugte Technologie: `gettext` in Kombination mit Tools wie `cargo-i18n` oder `fluent-rs` für komplexere Lokalisierungsanforderungen.
* **Fehlerbehandlung:**
    * Benutzerfreundliche Darstellung von Fehlern aus unteren Schichten oder UI-spezifischen Problemen (z.B. über `GtkInfoBar`, Dialoge oder Toasts).
    * Detailliertes Logging von UI-spezifischen Fehlern über das `tracing`-Framework.
* **Testbarkeit:**
    * UI-Komponenten sollen so entworfen werden, dass ihre Logik möglichst isoliert von der reinen Darstellung testbar ist.
    * Prüfung der Machbarkeit von automatisierten UI-Tests (z.B. mit `gtk4-rs`-kompatiblen Test-Frameworks, falls verfügbar, oder über Accessibility-Schnittstellen).
* **Konsistenz:**
    * Einheitliches Design, Verhalten und Terminologie über alle Teile der Benutzeroberfläche hinweg, idealerweise unter Verwendung von `libadwaita` und den Human Interface Guidelines (HIG) von GNOME als Referenz.
* **API-Design (intern):**
    * Interne APIs zwischen UI-Modulen sollten klar definiert und gut dokumentiert sein.
    * Nutzung von Rusts Typsystem zur Sicherstellung der Korrektheit.

### 4.3. Kernkomponenten der UI-Schicht

Die Benutzeroberflächenschicht wird in mehrere spezialisierte Module unterteilt, die jeweils spezifische Aspekte der UI implementieren.

#### 4.3.1. Modul: `ui::shell` (Desktop Shell)

* **Verantwortlichkeiten:** Implementierung der primären Interaktionsflächen des Desktops. Dazu gehören Panels (oben, unten, seitlich), ein optionales Dock, die "Intelligente Tab-Leiste" (pro Space/Workspace), der Workspace-Switcher, Systemstatus-Indikatoren (Netzwerk, Akku, Lautstärke, Bluetooth etc.), Uhrzeit- und Kalenderanzeige, Anwendungsstarter/Menü und das Quick Settings Panel.
* **Technologien:** GTK4, `gtk4-rs`. Für die Positionierung von Panels und Docks wird das Wayland-Protokoll `wlr-layer-shell-unstable-v1` genutzt, dessen serverseitige Implementierung in `system::compositor` erwartet wird. Die Client-seitige Ansteuerung erfolgt über entsprechende Rust-Bindings.
* **Datenstrukturen und Zustand (Beispiele):**
    * `ShellConfig`: Konfiguration der Shell-Elemente (Position, Sichtbarkeit, Inhalt).
    * `Vec<WindowInfo>`: Liste der laufenden Anwendungen und deren Fenster (bezogen von `system::compositor`, z.B. über `wlr-foreign-toplevel-management-unstable-v1`).
    * `CurrentWorkspaceInfo`, `Vec<WorkspaceInfo>`: Informationen über den aktiven Workspace und alle verfügbaren Workspaces (bezogen von `domain::workspaces`, typischerweise über die Systemschicht).
    * `SystemStatus`: Aggregierter Zustand von Systemdiensten (Netzwerk, Akku etc.), aktualisiert durch Events von `system::dbus` Clients.
* **Kernlogik:**
    * Dynamische Erstellung, Verwaltung und Aktualisierung der GTK-Widgets für alle Shell-Elemente.
    * Korrekte Positionierung und Verhalten von Layer-Surfaces (Panels, Dock) gemäß `wlr-layer-shell`-Protokoll.
    * Aktualisierung der "Intelligenten Tab-Leiste" basierend auf den Fenstern des aktiven Workspaces und deren Metadaten.
    * Implementierung des visuellen Workspace-Switchers und der Interaktion damit.
    * Bereitstellung des Anwendungsstarters (Suche, Kategorien).
    * Implementierung des Quick Settings Panels für schnellen Zugriff auf häufig benötigte Einstellungen (z.B. Lautstärke, Helligkeit, Netzwerk, Dunkelmodus).
* **Interaktionen:**
    * `system::compositor`: Empfängt Fensterlisten, Workspace-Informationen und Fokusänderungen. Sendet Anfragen zur Fensteraktivierung, Workspace-Wechsel. Nutzt das Layer-Shell-Protokoll zur Platzierung von UI-Elementen.
    * `domain::workspaces` (via Systemschicht): Zeigt Workspace-Informationen an und initiiert Workspace-bezogene Operationen (Wechsel, Erstellung etc.).
    * `domain::theming` (via `ui::theming_gtk`): Wendet Theme-Änderungen auf alle Shell-Elemente an.
    * `system::dbus` (Clients): Empfängt kontinuierlich Status-Updates (Netzwerk, Akku, Audio, etc.) und zeigt diese an.
    * Benutzer: Direkte Interaktion über Klicks, Tastatureingaben für Starter, Menüs, Quick Settings und Fenster-Tabs.
* **Fehlerbehandlung:** Stabile Anzeige auch bei fehlenden oder fehlerhaften Daten von unteren Schichten (z.B. Anzeige von Platzhaltern oder Standardwerten). Logging von Fehlern bei der Kommunikation mit dem Compositor oder D-Bus-Diensten.
* **Theming:** Alle Shell-Elemente sind vollständig über GTK CSS und die Design-Tokens aus `domain::theming` anpassbar.

#### 4.3.2. Modul: `ui::control_center` (Einstellungszentrale)

* **Verantwortlichkeiten:** Bereitstellung einer zentralen grafischen Oberfläche zur Ansicht und Modifikation aller Desktop-Einstellungen. Dies umfasst typischerweise Bereiche wie Erscheinungsbild (Themes, Schriftarten, Hintergrund), Workspaces, Eingabegeräte (Maus, Tastatur, Touchpad), Netzwerkverbindungen, Energieoptionen, Standardanwendungen, Benutzerkonten, Datum/Uhrzeit, Barrierefreiheit etc.
* **Technologien:** GTK4, `gtk4-rs`. Dringend empfohlen: `libadwaita` für eine konsistente Struktur (`AdwApplicationWindow`, `AdwPreferencesPage`, `AdwPreferencesGroup`, `AdwActionRow`, `AdwEntryRow` etc.) und ein ansprechendes Design.
* **Datenstrukturen und Zustand (Beispiele):**
    * `SettingsViewModel`: Repräsentation der Einstellungsstruktur und der aktuellen Werte, synchronisiert mit `domain::global_settings_and_state_management`.
    * Zwischenspeicherung von Änderungen vor dem Anwenden.
* **Kernlogik:**
    * Dynamisches Erstellen von Einstellungsseiten und -gruppen basierend auf den in `domain::global_settings_and_state_management` definierten Einstellungsmodulen und -pfaden.
    * Zwei-Wege-Bindung von UI-Steuerelementen (z.B. Schalter, Slider, Dropdowns) an die entsprechenden Einstellungswerte.
    * Echtzeit-Validierung von Benutzereingaben gemäß den Metadaten der Einstellungen (z.B. Wertebereiche, Regex).
    * Anwenden und Speichern von geänderten Einstellungen über die API von `domain::global_settings_and_state_management`.
    * Reaktion auf externe Einstellungsänderungen zur Aktualisierung der UI.
* **Interaktionen:**
    * `domain::global_settings_and_state_management`: Liest aktuelle Einstellungswerte und Metadaten. Schreibt geänderte Einstellungswerte. Abonniert `SettingChangedEvent` zur dynamischen Aktualisierung der angezeigten Werte.
    * `domain::theming`: Stellt die UI zur Auswahl von Themes, Farbschemata und Akzentfarben bereit, interagiert mit `ThemingEngine`.
    * `system::outputs`: Stellt die UI zur Konfiguration von Monitoren (Auflösung, Skalierung, Anordnung) bereit, interagiert mit `OutputManager`.
    * `system::input`: Stellt die UI zur Konfiguration von Tastatur (Layouts, Wiederholrate), Maus (Zeigergeschwindigkeit, Tastenbelegung) und Touchpad (Gesten, Scrollrichtung) bereit.
    * `system::audio`: Stellt die UI zur Konfiguration von Audiogeräten (Ein-/Ausgabegeräte, Lautstärken) bereit.
    * Benutzer: Navigation durch die Einstellungsseiten, Modifikation von Werten über die bereitgestellten Steuerelemente.
* **Fehlerbehandlung:** Klare Anzeige von Validierungsfehlern direkt an den Eingabefeldern. Informative Meldungen bei Fehlern während des Speicherns oder Ladens von Einstellungen.
* **Theming:** Das Control Center selbst nutzt das globale Theme. Die Vorschau-Elemente für Theme-Einstellungen müssen das jeweils ausgewählte Theme korrekt widerspiegeln.

#### 4.3.3. Modul: `ui::widgets_system` (Widget-System und Seitenleiste/Dashboard)

* **Verantwortlichkeiten:** Verwaltung, Konfiguration und Anzeige von Desktop-Widgets. Dies kann in einer dedizierten Seitenleiste, einem Dashboard-Modus oder als frei platzierbare Elemente geschehen. Bereitstellung einer API für Drittanbieter-Widgets.
* **Technologien:** GTK4, `gtk4-rs`.
* **Datenstrukturen und Zustand (Beispiele):**
    * `AvailableWidgetsRegistry`: Verzeichnis aller installierten und verfügbaren Widgets.
    * `ActiveWidgetInstance`: Zustand und Konfiguration eines spezifischen, vom Benutzer hinzugefügten Widgets.
    * `WidgetLayoutConfig`: Speichert Anordnung und Sichtbarkeit der Widgets.
* **Kernlogik:**
    * Mechanismus zum dynamischen Laden von Widget-Definitionen (ggf. als separate Crates/Plugins, die ein definiertes Trait implementieren).
    * Bereitstellung einer UI zur Auswahl, Hinzufügung, Entfernung und Konfiguration von Widgets durch den Benutzer.
    * Anzeige der aktiven Widgets im vorgesehenen Container (z.B. Seitenleiste).
    * Unterstützung für Drag & Drop zur Anordnung von Widgets.
    * Kommunikation mit Datenquellen für Widgets (z.B. Abruf von Wetterdaten über eine API, Auslesen von Systeminformationen, Zugriff auf Kalenderdaten).
    * Implementierung von Standard-Widgets:
        * `ClockWidget`: Anzeige von Uhrzeit und Datum.
        * `CalendarWidget`: Monatsansicht, Termine.
        * `WeatherWidget`: Aktuelles Wetter und Vorhersage.
        * `SystemMonitorWidget`: CPU-, RAM-, Netzwerkauslastung.
        * `NotesWidget`: Einfache Notizen.
* **Interaktionen:**
    * `domain::global_settings_and_state_management`: Speichert die Konfiguration des Widget-Systems und der einzelnen Widgets.
    * Externe Dienste/APIs: Für datengetriebene Widgets (z.B. Wetter-API, Nachrichten-Feeds).
    * `system::*`: Für Widgets, die Systeminformationen anzeigen.
    * Benutzer: Hinzufügen, Entfernen, Anordnen und Konfigurieren von Widgets. Interaktion mit den Widgets selbst.
* **Fehlerbehandlung:** Fehlerhafte oder abstürzende Widgets sollten isoliert werden, ohne das gesamte Widget-System oder den Desktop zu beeinträchtigen. Klare Fehlermeldungen bei Problemen mit dem Laden von Widgets oder dem Abrufen von Widget-Daten.
* **Theming:** Widgets müssen sich nahtlos in das globale Desktop-Theme einfügen und idealerweise über GTK CSS anpassbar sein.

#### 4.3.4. Modul: `ui::window_manager_frontend` (Fenstermanagement-UI)

* **Verantwortlichkeiten:** Bereitstellung von UI-Elementen und Interaktionen, die direkt mit der Verwaltung von Anwendungsfenstern zusammenhängen und über die reine Darstellung in der Shell hinausgehen.
    * **Client-Seitige Dekorationen (CSD):** Zeichnen und Verwalten von Fensterdekorationen (Titelleiste, Minimieren-, Maximieren-, Schließen-Buttons) für Anwendungen, falls der Compositor dies via `xdg-decoration-unstable-v1` an den Client delegiert.
    * **Fensterwechsler-UI:** Implementierung der UI für den schnellen Wechsel zwischen offenen Fenstern (z.B. der klassische "Alt+Tab"-Dialog).
    * **Übersichtsmodus:** Implementierung einer "Exposé"-artigen Ansicht, die alle Fenster des aktuellen Workspaces (oder optional aller Workspaces) als skalierte Vorschauen anzeigt und eine Auswahl per Klick oder Tastatur ermöglicht.
    * **Snapping/Tiling-Hilfen:** Visuelle Hinweise und Vorschau-Bereiche beim Verschieben oder Skalieren von Fenstern, um das Andocken oder Kacheln zu erleichtern (in Zusammenarbeit mit der Logik aus `domain::window_management` und `system::compositor`).
* **Technologien:** GTK4, `gtk4-rs`. Enge Interaktion mit `system::compositor` für Fensterinformationen (Position, Größe, Metadaten, Vorschau-Puffer) und Steuerung. Nutzung von `xdg-decoration-unstable-v1` (Client-Seite) und `wlr-foreign-toplevel-management-unstable-v1` (Client-Seite).
* **Datenstrukturen und Zustand (Beispiele):**
    * `WindowListCache`: Zwischengespeicherte Liste aller bekannten Anwendungsfenster mit relevanten Metadaten (Titel, App-ID, Icon, ggf. eine kleine Vorschau-Textur vom Compositor).
    * `ActiveSwitcherState`: Zustand des aktuellen Fensterwechslers (z.B. ausgewähltes Fenster im Alt+Tab-Dialog).
    * `OverviewLayout`: Anordnung der Fenster im Übersichtsmodus.
* **Kernlogik:**
    * Für CSDs: Erstellung eines `GtkHeaderBar` oder äquivalenter Widgets, Integration der Fenstersteuerungsbuttons, Anzeige des Fenstertitels und -icons. Verarbeitung von Mausinteraktionen auf der Titelleiste (z.B. Fenster verschieben).
    * Für den Fensterwechsler: Anzeige eines modalen Dialogs oder Overlays mit einer Liste/einem Raster von Fenstervorschauen. Navigation per Tastatur (Tab, Pfeiltasten) und Maus.
    * Für den Übersichtsmodus: Animationen zum Ein- und Ausblenden. Dynamische Anordnung der Fenstervorschauen. Interaktion per Klick zur Auswahl eines Fensters oder per Drag & Drop zum Verschieben zwischen Workspaces (falls unterstützt).
* **Interaktionen:**
    * `system::compositor`: Empfängt kontinuierlich aktualisierte Fensterdaten (Liste, Geometrie, Zustand, Fokus). Sendet Befehle an den Compositor (Fokus setzen, Fenster aktivieren, minimieren, maximieren, schließen, verschieben, Größe ändern).
    * `domain::window_management` (via Systemschicht): Erhält Informationen über Tiling-Layouts oder Snapping-Zonen, um entsprechende UI-Hilfen anzuzeigen.
    * Benutzer: Tastenkombinationen (z.B. Alt+Tab, Super+S für Übersicht), Mausinteraktionen in der Übersicht oder auf CSDs.
* **Fehlerbehandlung:** Stabile Darstellung auch bei inkonsistenten oder fehlenden Fensterinformationen vom Compositor. Graceful Degradation von Funktionen, falls bestimmte Wayland-Protokolle nicht verfügbar sind.
* **Theming:** CSDs, Fensterwechsler und der Übersichtsmodus müssen vollständig thematisierbar sein.

#### 4.3.5. Modul: `ui::notifications_frontend` (Benachrichtigungsanzeige)

* **Verantwortlichkeiten:** Visuelle Darstellung von Desktop-Benachrichtigungen. Bereitstellung eines Benachrichtigungszentrums oder -verlaufs. Ermöglichung der Interaktion mit Aktionen, die in Benachrichtigungen enthalten sind.
* **Technologien:** GTK4, `gtk4-rs`. `libadwaita` kann für konsistente Toasts (`AdwToast`) und Listen im Benachrichtigungszentrum verwendet werden. Die Kommunikation erfolgt über D-Bus durch Lauschen auf Signale des `org.freedesktop.Notifications`-Dienstes (dessen serverseitige Logik in `domain::notifications_core` und die D-Bus-Exposition in `system::dbus` liegen könnte).
* **Datenstrukturen und Zustand (Beispiele):**
    * `ActiveNotificationsList`: Liste der aktuell angezeigten oder kürzlich empfangenen Benachrichtigungen.
    * `NotificationHistory`: Längerfristiger Verlauf von Benachrichtigungen (ggf. mit Filter- und Suchfunktion).
    * `DoNotDisturbState`: Aktueller "Nicht stören"-Status.
* **Kernlogik:**
    * Registrierung beim `org.freedesktop.Notifications`-Dienst, um neue Benachrichtigungen (`Notify` Signal) und Schließ-Events (`NotificationClosed` Signal) zu empfangen.
    * Anzeige von transienten Benachrichtigungen als Popups oder Toasts (z.B. am Bildschirmrand).
    * Implementierung eines Benachrichtigungszentrums (z.B. als Teil der Quick Settings oder als separate Seitenleiste), das eine Liste aller (oder relevanter) Benachrichtigungen anzeigt.
    * Darstellung von Aktionen (Buttons) innerhalb von Benachrichtigungen und Weiterleitung der Aktionsauslösung (`ActionInvoked` Signal) an den Benachrichtigungsdienst.
    * Verwaltung des "Gelesen"-Status von Benachrichtigungen.
* **Interaktionen:**
    * `system::dbus` (als Client des `org.freedesktop.Notifications` Dienstes): Empfängt Benachrichtigungsdaten. Sendet Signale zum Schließen von Benachrichtigungen oder zum Auslösen von Aktionen.
    * `domain::notifications_core` (indirekt über D-Bus): Liefert die Inhalte und Regeln für Benachrichtigungen.
    * Benutzer: Klick auf Benachrichtigungen, Interaktion mit Aktionsbuttons, Schließen von Benachrichtigungen, Öffnen und Verwalten des Benachrichtigungszentrums.
* **Fehlerbehandlung:** Korrekte und sichere Anzeige auch bei fehlerhaft formatierten oder potenziell schädlichen Daten in Benachrichtigungen (z.B. Sanitization von HTML, falls unterstützt). Stabile Funktion auch bei hoher Benachrichtigungsfrequenz.
* **Theming:** Das Aussehen von Benachrichtigungs-Popups und des Benachrichtigungszentrums ist über GTK CSS anpassbar.

#### 4.3.6. Modul: `ui::theming_gtk` (GTK Theming-Integration)

* **Verantwortlichkeiten:** Konkrete Anwendung der von `domain::theming` (ThemingEngine) aufgelösten Design-Tokens und Stildefinitionen auf alle GTK4-basierten UI-Komponenten der Desktop-Umgebung. Laden und dynamisches Wechseln von GTK-Themes (CSS-Dateien) zur Laufzeit.
* **Technologien:** GTK4 (`GtkCssProvider`, `GtkStyleContext`), `gtk4-rs`.
* **Kernlogik:**
    * Abonnieren des `ThemeChangedEvent` (oder eines äquivalenten Signals/Callbacks) von `domain::theming::ThemingEngine`.
    * Bei Empfang eines `ThemeChangedEvent`:
        * Abrufen des neuen `AppliedThemeState` (inklusive der aufgelösten Tokens als CSS-Variablen oder einer generierten CSS-Datei).
        * Erstellen eines neuen `GtkCssProvider`.
        * Laden der CSS-Daten (entweder direkt die generierten CSS-Variablen und -Regeln oder eine Pfadangabe zu einer CSS-Datei) in den `GtkCssProvider` mittels `load_from_data()` oder `load_from_path()`.
        * Entfernen des alten `GtkCssProvider` vom `GdkDisplay` und Hinzufügen des neuen Providers mittels `GtkStyleContext::add_provider_for_display()`. Dies löst eine Aktualisierung aller GTK-Widgets aus.
    * Falls `libadwaita` verwendet wird: Interaktion mit `AdwStyleManager` zum Umschalten zwischen Hell-, Dunkel- und Hochkontrast-Modi und zur Verwaltung der Akzentfarbe, falls diese nicht rein über CSS gesteuert wird.
* **Interaktionen:**
    * `domain::theming::ThemingEngine`: Empfängt Benachrichtigungen über Theme-Änderungen und die zugehörigen Styling-Daten.
    * GTK4-System: Interagiert mit `GdkDisplay` und `GtkStyleContext`, um das CSS global anzuwenden.
    * Alle anderen `ui::*` Module: Werden durch die Änderungen im globalen CSS automatisch neu gestylt.
* **Fehlerbehandlung:** Implementierung eines Fallback-Mechanismus auf ein Standard-GTK-Theme (z.B. Adwaita default), falls die vom `ThemingEngine` bereitgestellten CSS-Daten fehlerhaft sind oder nicht geladen werden können. Logging solcher Fehler.

#### 4.3.7. Modul: `ui::command_palette` (Befehlspalette)

* **Verantwortlichkeiten:** Bereitstellung einer schnell zugänglichen, textbasierten Befehlspalette (ähnlich wie in VS Code oder Spotlight unter macOS) für den schnellen Zugriff auf eine Vielzahl von Aktionen, Anwendungen, Einstellungen, Dateien und potenziell KI-gestützten Funktionen.
* **Technologien:** GTK4, `gtk4-rs`.
* **Datenstrukturen und Zustand (Beispiele):**
    * `CommandRegistry`: Dynamisch befülltes Verzeichnis aller verfügbaren Befehle, kategorisiert und mit Metadaten (Name, Beschreibung, Icon, auszuführende Aktion) versehen.
    * `SearchResultList`: Gefilterte und sortierte Liste der Befehle basierend auf der Benutzereingabe.
* **Kernlogik:**
    * Öffnen der Befehlspalette über eine globale Tastenkombination (definiert in `system::input` und weitergeleitet an die UI).
    * Dynamische Aggregation von Befehlen aus verschiedenen Quellen:
        * Installierte Anwendungen (aus `.desktop`-Dateien).
        * Aktionen der Shell (z.B. "Neues Fenster", "Workspace wechseln").
        * Direktzugriff auf spezifische Einstellungsseiten im `ui::control_center`.
        * Dateisuche (Integration mit einem Dateisuchindex wie Tracker oder einer einfacheren Implementierung).
        * KI-Aktionen, bereitgestellt über `system::mcp` (z.B. "Fasse Text zusammen", "Übersetze Satz").
    * Implementierung einer effizienten Fuzzy-Suche und Filterung der Befehle in Echtzeit während der Benutzereingabe.
    * Ausführung der vom Benutzer ausgewählten Aktion (kann das Starten einer Anwendung, das Aufrufen einer internen Funktion, das Senden eines D-Bus-Befehls oder eine MCP-Anfrage umfassen).
    * Kontextsensitivität: Die angebotenen Befehle können sich je nach aktivem Fenster oder Zustand des Desktops ändern.
* **Interaktionen:**
    * `ui::shell`: Registriert Shell-spezifische Aktionen.
    * `ui::control_center`: Registriert Direktzugriffe auf Einstellungsseiten.
    * `system::mcp`: Stellt KI-gestützte Aktionen und potenziell Suchergebnisse bereit.
    * Dateisystem/Suchindex: Für die Dateisuche.
    * Anwendungsstarter-Logik: Zum Starten von Anwendungen.
    * Benutzer: Eingabe von Suchbegriffen, Auswahl von Befehlen per Tastatur oder Maus.
* **Fehlerbehandlung:** Klare Rückmeldung, falls keine passenden Befehle gefunden werden. Fehlerbehandlung und -anzeige bei Problemen während der Ausführung eines ausgewählten Befehls.
* **Theming:** Die Befehlspalette ist über GTK CSS thematisierbar.

#### 4.3.8. Modul: `ui::speed_dial` (Schnellstart-Ansicht)

* **Verantwortlichkeiten:** Anzeige einer Schnellstart-Seite oder eines "Neuer Tab"-Äquivalents, typischerweise beim Start einer neuen Sitzung oder eines neuen Fensters (falls anwendbar). Diese Ansicht kann häufig genutzte Anwendungen, Lesezeichen, kürzlich geöffnete Projekte oder andere für den Benutzer relevante Informationen enthalten.
* **Technologien:** GTK4, `gtk4-rs`.
* **Datenstrukturen und Zustand (Beispiele):**
    * `SpeedDialItemList`: Konfigurierbare und/oder dynamisch (lernbasiert) erstellte Liste der anzuzeigenden Schnellstart-Elemente.
    * `LayoutConfiguration`: Einstellungen zur Darstellung (Raster, Liste, Icongrößen).
* **Kernlogik:**
    * Anzeige der Schnellstart-Elemente in einem ansprechenden Layout (z.B. Raster von Icons mit Beschriftungen).
    * Ermöglichen des Hinzufügens, Entfernens und Anordnens von Elementen durch den Benutzer (Drag & Drop).
    * Potenzielle Integration mit dem Browser-Verlauf/Lesezeichen, dem Dateisystem (kürzliche Dateien/Projekte) oder Projektmanagement-Tools.
    * Lernalgorithmus zur Priorisierung oder Vorschlag von Elementen basierend auf Nutzungshäufigkeit.
* **Interaktionen:**
    * Benutzer: Klick auf Elemente zum Starten einer Anwendung oder Öffnen einer Datei/URL. Konfiguration der angezeigten Elemente und des Layouts.
    * Anwendungsstarter-Logik, Dateisystem-APIs.
* **Theming:** Die Schnellstart-Ansicht ist an das globale Desktop-Theme anpassbar.

#### 4.3.9. Modul: `ui::common_components` (Wiederverwendbare UI-Komponenten)

* **Verantwortlichkeiten:** Entwicklung und Bereitstellung einer Sammlung von anwendungsspezifischen, aber über verschiedene UI-Module hinweg wiederverwendbaren GTK4-Widgets und UI-Bausteinen. Ziel ist die Reduktion von Code-Duplikation und die Sicherstellung eines konsistenten Erscheinungsbilds und Verhaltens.
* **Beispiele:**
    * `ValidatedEntry`: Ein `GtkEntry`-Wrapper mit integrierter Validierungslogik und -anzeige.
    * `IconLabelButton`: Ein Button mit Icon und Text, der spezifische Styling- oder Verhaltensanforderungen erfüllt.
    * `PannableZoomableImageWidget`: Ein Widget zur Anzeige von Bildern mit Pan- und Zoom-Funktionalität.
    * Spezialisierte Layout-Container oder Listenelemente.
* **Technologien:** GTK4, `gtk4-rs`. Erstellung als `GtkWidget` Subklassen oder Composite Widgets.
* **Kernlogik:** Jede Komponente kapselt ihre spezifische Logik und Darstellung. Sie sollten gut dokumentiert und mit Beispielen versehen sein.
* **Interaktionen:** Werden von anderen UI-Modulen (`ui::shell`, `ui::control_center` etc.) instanziiert und verwendet.
* **Theming:** Die Komponenten müssen so gestaltet sein, dass sie das globale Theming respektieren und über GTK CSS angepasst werden können.

#### 4.3.10. Modul: `ui::input_handling` (UI-spezifische Eingabelogik)

* **Verantwortlichkeiten:** Verarbeitung von UI-spezifischen Tastenkombinationen (Shortcuts), die nicht global vom Compositor (`system::input`), sondern von der gerade aktiven UI-Komponente oder der Desktop-Anwendung als Ganzes behandelt werden sollen. Implementierung von UI-internen Gesten (z.B. Swipes in Listenansichten, Pinch-to-Zoom in einem Bildbetrachter-Widget innerhalb einer Anwendung).
* **Technologien:** GTK4 Event-Controller (`GtkGestureClick`, `GtkGestureDrag`, `GtkEventControllerKey`, `GtkShortcutManager`, `GtkShortcutsWindow`), `gtk4-rs`.
* **Kernlogik:**
    * Definition und Registrierung von UI-Aktionen und den zugehörigen Shortcuts über `GtkApplication::set_accels_for_action()` oder `GtkShortcutManager`.
    * Implementierung von Gestenerkennung für spezifische Widgets mittels GTK4 Event-Controllern.
    * Weiterleitung von verarbeiteten Eingabeereignissen an die entsprechenden Handler-Funktionen innerhalb der UI-Komponenten, um UI-spezifische Aktionen auszulösen.
    * Bereitstellung einer Hilfsansicht (z.B. über `GtkShortcutsWindow`) zur Anzeige aller verfügbaren Tastenkombinationen.
* **Interaktionen:**
    * GTK4-Framework: Empfängt rohe Eingabeereignisse vom Compositor und leitet sie an die Event-Controller und das Shortcut-Management weiter.
    * Andere `ui::*` Module: Definieren Aktionen und reagieren auf deren Auslösung durch Shortcuts oder Gesten.

#### 4.3.11. Modul: `ui::accessibility` (Barrierefreiheit)

* **Verantwortlichkeiten:** Zentralisierte Bemühungen und Richtlinien zur Sicherstellung, dass alle UI-Komponenten den etablierten Barrierefreiheitsstandards (z.B. WCAG AA als Ziel) entsprechen. Bereitstellung aller notwendigen Informationen und Schnittstellen für assistive Technologien (AT).
* **Technologien:** GTK4 Accessibility APIs (Implementierung des `GtkAccessible` Interfaces, Nutzung von `AtkObject` bzw. dessen Nachfolger-APIs), `gtk4-rs`. Testwerkzeuge wie `accerciser` und Screenreader wie Orca.
* **Kernlogik:**
    * Für alle benutzerdefinierten Widgets: Korrekte Implementierung der `GtkAccessible` Methoden, insbesondere die Zuweisung der korrekten `AccessibleRole`.
    * Sicherstellung, dass alle interaktiven Elemente zugängliche Namen (accessible names) und ggf. Beschreibungen (accessible descriptions) haben, die ihren Zweck und Zustand klar kommunizieren.
    * Gewährleistung einer vollständigen und logischen Tastaturnavigation für alle interaktiven UI-Elemente.
    * Ausreichender Farbkontrast gemäß den Richtlinien.
    * Unterstützung für Hochkontrast-Themes.
    * Regelmäßige Tests mit Screenreadern und anderen assistiven Technologien.
* **Interaktionen:**
    * Assistive Technologien (z.B. Orca): Greifen auf die von den GTK-Widgets bereitgestellten Accessibility-Informationen zu, um die UI für Benutzer mit Einschränkungen interpretierbar und bedienbar zu machen.
    * Alle anderen `ui::*` Module: Müssen bei der Implementierung ihrer Widgets die Accessibility-Anforderungen berücksichtigen.

#### 4.3.12. Modul: `ui::state_management` (UI-Zustandsverwaltung)

* **Verantwortlichkeiten:** Definition und Implementierung konsistenter Muster und Mechanismen zur Verwaltung des UI-Zustands und dessen Synchronisation mit den Daten und der Logik der Domänen- und Systemschichten. Dies ist besonders wichtig für komplexe UIs, um Datenkonsistenz zu gewährleisten und die Codebasis wartbar zu halten.
* **Technologien und Muster:**
    * Rust-Idiome: `Arc<Mutex<T>>` oder `Arc<RwLock<T>>` für sicher geteilten, veränderlichen Zustand.
    * Asynchrone Kanäle: `tokio::sync::watch` für "single producer, multiple consumer" Zustandsverteilung oder `tokio::sync::broadcast` für allgemeine Event-Benachrichtigungen, die Zustandsänderungen signalisieren.
    * Beobachter-Muster (Observer Pattern): UI-Komponenten registrieren sich bei Datenquellen (aus der Domänen- oder Systemschicht) und werden bei Änderungen benachrichtigt.
    * Model-View-ViewModel (MVVM) Anleihen: Trennung von UI (View), UI-Logik und Zustandspräsentation (ViewModel) und den eigentlichen Daten (Model in der Domänenschicht).
    * Nutzung von `glib::Object` Signalen für die Kommunikation zwischen GTK-Widgets und der Rust-Logik.
* **Kernlogik:**
    * Bereitstellung von Mechanismen, damit UI-Komponenten reaktiv auf Änderungen in der Domänen- oder Systemschicht reagieren können (z.B. automatische Aktualisierung einer Liste in der UI, wenn sich die zugrundeliegenden Daten in der Domänenschicht ändern).
    * Verwaltung von reinem UI-Zustand (z.B. ob ein Dropdown-Menü gerade geöffnet ist, der Scroll-Zustand einer Liste), der nicht in den unteren Schichten persistiert werden muss.
    * Sicherstellung der Datenkonsistenz zwischen der UI und den Backend-Schichten, insbesondere bei nebenläufigen Änderungen.
    * Abstraktion der Komplexität der direkten Interaktion mit nebenläufigen Primitiven für einzelne UI-Komponenten.
* **Interaktionen:**
    * Alle `ui::*` Module: Nutzen die bereitgestellten Zustandsverwaltungsmechanismen, um ihren eigenen Zustand zu verwalten und auf Änderungen von außen zu reagieren.
    * Domänen- und Systemschicht: Dienen als primäre Quelle für den Zustand, der in der UI dargestellt und modifiziert wird. Änderungen werden über definierte Schnittstellen (Events, Callbacks, Kanäle) an die UI-Schicht kommuniziert.

### 4.4. Interaktion mit anderen Schichten

Die UI-Schicht ist naturgemäß stark mit den darunterliegenden Schichten verbunden:

* **Interaktion mit der Systemschicht (`system::*`):**
    * **Empfängt:**
        * Fensterinformationen (Liste, Titel, App-ID, Fokus, Geometrie) von `system::compositor` (z.B. über `wlr-foreign-toplevel-management`).
        * Eingabeereignisse, die vom `system::compositor` und `system::input` an die fokussierte Anwendung (also die UI-Shell selbst oder andere Desktop-Komponenten) weitergeleitet werden.
        * Systemstatus-Events (z.B. Output-Änderungen von `system::outputs`, D-Bus-Signale von `system::dbus` für Netzwerk, Energie, Audio etc.).
        * Informationen über verfügbare MCP-Ressourcen oder Antworten von `system::mcp`.
    * **Sendet:**
        * Befehle an `system::compositor` (z.B. Fensterfokus ändern, Fenster verschieben/skalieren, Workspace wechseln, Layer-Surface-Konfiguration für `ui::shell`).
        * Befehle an D-Bus-Dienste über `system::dbus` (z.B. Lautstärke ändern, Netzwerkverbindung herstellen, Einstellungen ändern, die Systemdienste betreffen).
        * Anfragen an `system::mcp` für KI-Funktionen.
        * Anfragen an `system::portals` zur Durchführung von Aktionen im Namen von Anwendungen (z.B. Datei öffnen Dialog).
* **Interaktion mit der Domänenschicht (`domain::*`):**
    * **Liest (oft indirekt über Systemschicht-APIs oder dedizierte Services):**
        * Aktuelle Theme-Definitionen und aufgelöste Tokens von `domain::theming`.
        * Workspace-Konfigurationen und -zustände von `domain::workspaces`.
        * Globale Einstellungen von `domain::global_settings_and_state_management`.
        * Benachrichtigungslisten und -verläufe von `domain::notifications_core`.
        * KI-Einwilligungsstatus von `domain::user_centric_services::ai_interaction_service`.
    * **Löst Aktionen aus / Schreibt Daten (oft indirekt):**
        * Anforderung eines Theme-Wechsels an `domain::theming`.
        * Speichern von geänderten Einstellungen an `domain::global_settings_and_state_management`.
        * Markieren von Benachrichtigungen als gelesen oder Auslösen von Aktionen an `domain::notifications_core`.
        * Änderung des Workspace-Layouts oder Umbenennung an `domain::workspaces`.
    * **Abonniert Events:**
        * `ThemeChangedEvent` von `domain::theming` zur Aktualisierung des UI-Stylings.
        * `WorkspaceEvent` (z.B. `WorkspaceCreated`, `ActiveWorkspaceChanged`) von `domain::workspaces`.
        * `SettingChangedEvent` von `domain::global_settings_and_state_management`.
        * `NotificationEvent` (z.B. `NotificationPostedEvent`) von `domain::notifications_core`.
        * `AIConsentUpdatedEvent` von `domain::user_centric_services::ai_interaction_service`.
* **Interaktion mit der Kernschicht (`core::*`):**
    * Nutzt grundlegende Datentypen (`core::types` wie `Color`, `Rect`, `Point`, `Size`), sofern diese für UI-Berechnungen oder die Kommunikation mit anderen Schichten relevant sind.
    * Verwendet die Fehlerbehandlungsmechanismen (`core::errors`), um Fehler aus unteren Schichten zu verarbeiten oder eigene UI-spezifische Fehler zu definieren, die von `CoreError` abgeleitet sein können.
    * Nutzt die Logging-Infrastruktur (`core::logging` bzw. das `tracing`-Framework) für detailliertes Logging von UI-Aktivitäten und Fehlern.

Diese detaillierte Spezifikation der Benutzeroberflächenschicht sollte eine solide Grundlage für die weitere Entwicklung und Implementierung bieten.


# **NovaDE UI-Schicht: Implementierungsleitfaden – : ui::shell::PanelWidget und AppMenuButton**

## **1\. Einleitung**

### **1.1. Zweck des Dokuments**

Dieses Dokument dient als detaillierter Implementierungsleitfaden für ausgewählte Module der UI-Schicht der Nova Desktop Environment (NovaDE). Es spezifiziert die Architektur, das Design, die Datenstrukturen, Schnittstellen und Implementierungsdetails auf einer ultrafeinen Ebene, sodass Entwicklerteams diese Spezifikationen direkt für die Codierung verwenden können, ohne grundlegende Designentscheidungen treffen oder Kernlogiken selbst entwerfen zu müssen. Dieses erste Teildokument fokussiert sich auf die Kernkomponente ui::shell::PanelWidget und dessen Submodul ui::shell::panel\_widget::AppMenuButton.

### **1.2. Zielgruppe**

Dieses Dokument richtet sich an Softwareentwickler und \-architekten, die an der Implementierung der NovaDE UI-Schicht beteiligt sind. Es wird ein Verständnis von Rust, GTK4 und den gtk4-rs Bindings sowie grundlegenden Konzepten der Softwarearchitektur und des UI-Designs vorausgesetzt.

### **1.3. Umfang (Teil 1: ui::shell::PanelWidget und AppMenuButton)**

Dieser erste Teil des Implementierungsleitfadens für die UI-Schicht behandelt die folgenden Module:

* **ui::shell::PanelWidget**: Die Haupt-Panel-Komponente der Desktop-Shell, verantwortlich für die Aufnahme und Anordnung verschiedener Panel-Module.  
* **ui::shell::panel\_widget::AppMenuButton**: Ein spezifisches Panel-Modul innerhalb des PanelWidget, das das globale Anwendungsmenü der aktiven Applikation anzeigt.

Nachfolgende Teildokumente werden weitere Module der UI-Schicht detaillieren.

### **1.4. Technologie-Stack (Verbindlich)**

Die Implementierung der UI-Schicht erfolgt unter strikter Verwendung des folgenden Technologie-Stacks:

* **GUI-Toolkit**: GTK4 1  
* **Rust-Bindings**: gtk4-rs 1  
* **Programmiersprache**: Rust 4  
* **Asynchrone Operationen**: Integration mit Rusts async/await über glib::MainContext::spawn\_local 7  
* **Theming**: Anwendung von CSS-Stilen über gtk::CssProvider, generiert durch domain::theming 9  
* **D-Bus-Kommunikation**: zbus Crate für Interaktionen mit Systemdiensten und anderen Anwendungen 12

### **1.5. Allgemeine UI/UX-Prinzipien (Wiederholung)**

Die Entwicklung der UI-Schicht orientiert sich an den folgenden übergeordneten UI/UX-Prinzipien, die eine visionstreue Umsetzung gewährleisten:

* **Konsistenz**: Einheitliches Erscheinungsbild und Verhalten über alle UI-Komponenten hinweg.  
* **Feedback**: Klares visuelles (und ggf. haptisches) Feedback auf Benutzeraktionen.  
* **Effizienz**: Minimierung der notwendigen Schritte zur Erledigung häufiger Aufgaben.  
* **Zugänglichkeit (Accessibility)**: Einhaltung der a11y-Standards (ATK/AT-SPI).23  
* **Performance**: Flüssige Animationen, schnelle Reaktionszeiten und geringer Ressourcenverbrauch.24  
* **Anpassbarkeit**: Ermöglichung benutzerdefinierter Konfigurationen von Layouts, Widgets und Verhalten.

## **2\. Modul: ui::shell::PanelWidget (Haupt-Panel-Implementierung)**

### **2.1.1. Übersicht und Verantwortlichkeiten**

Das PanelWidget ist die zentrale Komponente der ui::shell, die als primäre(s) Kontroll- und Systemleiste(n) der NovaDE dient. Es ist verantwortlich für:

* Die Bereitstellung einer oder mehrerer horizontaler Leisten am Bildschirmrand (oben oder unten, konfigurierbar).  
* Die Aufnahme, Anordnung und Verwaltung verschiedener, modularer Panel-Elemente (Submodule wie AppMenuButton, ClockDateTimeWidget, etc.).  
* Die Implementierung grundlegender Panel-Eigenschaften wie Höhe, Transparenz und eines visuellen "Leuchtakzent"-Effekts.  
* Die Interaktion mit dem gtk4-layer-shell-Protokoll, um sich korrekt in Wayland-Compositors zu integrieren, die dieses Protokoll unterstützen (z.B. wlroots-basierte wie Sway, Mir, KDE Plasma).26  
* Das dynamische Laden und Anwenden von Theming-Informationen, insbesondere für den "Leuchtakzent" und Hintergrundstile.

### **2.1.2. Visuelles Design und Theming**

* **Positionierung**: Konfigurierbar am oberen oder unteren Bildschirmrand.  
* **Höhe**: Konfigurierbare Höhe, z.B. zwischen 24px und 128px.  
* **Transparenz**: Optionale Transparenz des Panel-Hintergrunds. Dies wird durch Setzen der Opazität des Hauptfensters und/oder durch Verwendung von RGBA-Farben im CSS und im benutzerdefinierten Zeichencode erreicht. Für Wayland-Compositors, die transparente Oberflächen unterstützen, muss das zugrundeliegende GdkSurface entsprechend konfiguriert werden. Die gtk4-layer-shell kann hierbei relevant sein, um sicherzustellen, dass der Compositor die Transparenz korrekt handhabt.26  
* **"Leuchtakzent"-Effekt**: Ein subtiler Leuchteffekt entlang einer Kante des Panels (z.B. die dem Bildschirmzentrum zugewandte Kante), dessen Farbe und Intensität durch das Theming-System (domain::theming) gesteuert wird. Die Implementierung erfolgt entweder durch CSS (box-shadow mit entsprechenden Offsets und Blur-Radien 36) oder durch benutzerdefiniertes Zeichnen mit Cairo auf einem gtk::DrawingArea.37  
* **CSS-Styling**:  
  * **CSS-Knoten**: Das PanelWidget selbst (als GtkApplicationWindow) hat den CSS-Knoten window. Wenn es einen internen Hauptcontainer (z.B. GtkBox) verwendet, hat dieser den Knoten box.42 Spezifische CSS-Klassen werden zugewiesen, um das Styling zu erleichtern.  
  * **CSS-Klassen**:  
    * .nova-panel: Allgemeine Klasse für das Panel.  
    * .panel-top, .panel-bottom: Je nach Positionierung.  
    * .transparent-panel: Wenn Transparenz aktiviert ist.  
  * Die Anwendung von CSS erfolgt über einen globalen gtk::CssProvider, der durch ui::theming\_gtk verwaltet wird.10 Das Panel reagiert auf ThemeChangedEvents, um dynamische Stiländerungen zu übernehmen.

### **2.1.3. Datenstrukturen, Eigenschaften und Zustand**

Das PanelWidget wird als benutzerdefiniertes GObject-Widget implementiert, das von gtk::ApplicationWindow erbt, um die Integration mit gtk4-layer-shell zu ermöglichen.27

* GObject-Definition (PanelWidget):  
  Die Definition erfolgt in zwei Hauptdateien: mod.rs für die öffentliche API und imp.rs für die private GObject-Implementierung.  
  *Auszug aus src/ui/shell/panel\_widget/mod.rs (vereinfacht):*  
  Rust  
  use gtk::glib;  
  use gtk::subclass::prelude::\*;  
  use std::cell::{Cell, RefCell};

  mod imp;

  glib::wrapper\! {  
      pub struct PanelWidget(ObjectSubclass\<imp::PanelWidget\>)  
          @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,  
          @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;  
  }

  impl PanelWidget {  
      pub fn new(app: \&gtk::Application) \-\> Self {  
          glib::Object::builder::\<Self\>()  
             .property("application", app)  
             .build()  
      }

      // Öffentliche Methoden hier definieren, z.B.:  
      pub fn add\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>, position: imp::ModulePosition, order: i32) {  
          self.imp().add\_module(module, position, order);  
      }

      pub fn remove\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>) {  
          self.imp().remove\_module(module);  
      }  
  }

  *Auszug aus src/ui/shell/panel\_widget/imp.rs (vereinfacht):*  
  Rust  
  use gtk::glib;  
  use gtk::subclass::prelude::\*;  
  use gtk::{CompositeTemplate, Align};  
  use std::cell::{Cell, RefCell};  
  use std::collections::HashMap;  
  use once\_cell::sync::Lazy; // \[123\]

  // Enum für PanelPosition  
  \#  
  \#  
  pub enum PanelPosition {  
      \#\[default\]  
      Top,  
      Bottom,  
  }

  \#  
  \#  
  pub enum ModulePosition {  
      Start,  
      Center,  
      End,  
  }

  static PANEL\_PROPERTIES: Lazy\<Vec\<glib::ParamSpec\>\> \= Lazy::new(|| {  
      vec\!  
  });

  // Hier könnten benutzerdefinierte Signale definiert werden, falls benötigt.  
  // static PANEL\_SIGNALS: Lazy\<HashMap\<String, glib::subclass::Signal\>\> \= Lazy::new(|| HashMap::new());

  \#  
  \#\[template(resource \= "/org/nova\_de/ui/shell/panel\_widget.ui")\] // Pfad zur UI-Datei  
  pub struct PanelWidget {  
      \#\[template\_child\]  
      pub(super) main\_box: TemplateChild\<gtk::Box\>,  
      \#\[template\_child\]  
      pub(super) start\_box: TemplateChild\<gtk::Box\>,  
      \#\[template\_child\]  
      pub(super) center\_box: TemplateChild\<gtk::Box\>,  
      \#\[template\_child\]  
      pub(super) end\_box: TemplateChild\<gtk::Box\>,

      // Für benutzerdefiniertes Zeichnen, falls CSS nicht ausreicht  
      drawing\_area: RefCell\<Option\<gtk::DrawingArea\>\>,

      \#\[property(get, set, explicit\_notify)\]  
      position: RefCell\<PanelPosition\>,  
      \#\[property(get, set, explicit\_notify)\]  
      panel\_height: Cell\<i32\>,  
      \#\[property(get, set, explicit\_notify)\]  
      transparency\_enabled: Cell\<bool\>,  
      \#\[property(get, set, explicit\_notify)\]  
      leuchtakzent\_color: RefCell\<Option\<gdk::RGBA\>\>,  
      \#\[property(get, set, explicit\_notify)\]  
      leuchtakzent\_intensity: Cell\<f64\>,

      // Interne Verwaltung der Module  
      modules\_start: RefCell\<Vec\<gtk::Widget\>\>,  
      modules\_center: RefCell\<Vec\<gtk::Widget\>\>,  
      modules\_end: RefCell\<Vec\<gtk::Widget\>\>,  
  }

  \#\[glib::object\_subclass\]  
  impl ObjectSubclass for PanelWidget {  
      const NAME: &'static str \= "NovaDEPanelWidget";  
      type Type \= super::PanelWidget;  
      type ParentType \= gtk::ApplicationWindow;

      fn class\_init(klass: \&mut Self::Class) {  
          klass.bind\_template();  
          klass.install\_properties(\&PANEL\_PROPERTIES);  
          // klass.install\_signals(\&PANEL\_SIGNALS, false); // Falls Signale vorhanden

          // CSS-Name für das Widget setzen, falls nicht über UI-Datei  
          klass.set\_css\_name("panelwidget");  
      }

      fn instance\_init(obj: \&glib::subclass::InitializingObject\<Self\>) {  
          obj.init\_template();  
      }  
  }

  impl ObjectImpl for PanelWidget {  
      fn constructed(\&self) {  
          self.parent\_constructed();  
          let obj \= self.obj();

          // Standardwerte setzen, falls nicht durch Properties initialisiert  
          if self.position.borrow().eq(\&PanelPosition::default()) {  
               self.position.replace(PanelPosition::Top);  
          }  
          if self.panel\_height.get() \== 0 { // GObject Int default ist 0  
              self.panel\_height.set(36); // Expliziter Standardwert  
          }  
           if self.leuchtakzent\_intensity.get() \== 0.0 { // GObject Double default ist 0.0  
              self.leuchtakzent\_intensity.set(0.5);  
          }

          // Layer Shell initialisieren  
          obj.setup\_layer\_shell();  
          obj.update\_layout(); // Erstes Layout anwenden

          // Eventuell DrawingArea initialisieren und verbinden  
          // let drawing\_area \= gtk::DrawingArea::new();  
          // drawing\_area.set\_content\_width(obj.width\_request()); // Beispiel  
          // drawing\_area.set\_content\_height(self.panel\_height.get());  
          // self.main\_box.prepend(\&drawing\_area); // Oder als Hintergrund  
          // self.drawing\_area.replace(Some(drawing\_area));  
          // self.obj().connect\_draw\_signal();  
      }

      fn properties() \-\> &'static {  
          PANEL\_PROPERTIES.as\_ref()  
      }

      fn set\_property(\&self, \_id: usize, value: \&glib::Value, pspec: \&glib::ParamSpec) {  
          match pspec.name() {  
              "position" \=\> {  
                  let position: PanelPosition \= value.get().expect("Value must be PanelPosition");  
                  self.position.replace(position);  
                  self.obj().setup\_layer\_shell(); // Layer Shell neu konfigurieren bei Positionsänderung  
                  self.obj().notify\_position();   
              }  
              "panel-height" \=\> {  
                  let height: i32 \= value.get().expect("Value must be i32");  
                  self.panel\_height.set(height);  
                  self.obj().set\_default\_height(height); // Fensterhöhe anpassen  
                  self.main\_box.set\_height\_request(height);  
                  // Ggf. DrawingArea Höhe anpassen  
                  // if let Some(da) \= self.drawing\_area.borrow().as\_ref() {  
                  //    da.set\_content\_height(height);  
                  // }  
                  self.obj().queue\_draw(); // Neuzeichnen anfordern  
                  self.obj().notify\_panel\_height();  
              }  
              "transparency-enabled" \=\> {  
                  let enabled: bool \= value.get().expect("Value must be bool");  
                  self.transparency\_enabled.set(enabled);  
                  self.obj().update\_transparency();  
                  self.obj().notify\_transparency\_enabled();  
              }  
              "leuchtakzent-color" \=\> {  
                  let color: Option\<gdk::RGBA\> \= value.get().expect("Value must be Option\<gdk::RGBA\>");  
                  self.leuchtakzent\_color.replace(color);  
                  self.obj().queue\_draw();  
                  self.obj().notify\_leuchtakzent\_color();  
              }  
              "leuchtakzent-intensity" \=\> {  
                  let intensity: f64 \= value.get().expect("Value must be f64");  
                  self.leuchtakzent\_intensity.set(intensity);  
                  self.obj().queue\_draw();  
                  self.obj().notify\_leuchtakzent\_intensity();  
              }  
              \_ \=\> unimplemented\!(),  
          }  
      }

      fn property(\&self, \_id: usize, pspec: \&glib::ParamSpec) \-\> glib::Value {  
          match pspec.name() {  
              "position" \=\> self.position.borrow().to\_value(),  
              "panel-height" \=\> self.panel\_height.get().to\_value(),  
              "transparency-enabled" \=\> self.transparency\_enabled.get().to\_value(),  
              "leuchtakzent-color" \=\> self.leuchtakzent\_color.borrow().to\_value(),  
              "leuchtakzent-intensity" \=\> self.leuchtakzent\_intensity.get().to\_value(),  
              \_ \=\> unimplemented\!(),  
          }  
      }  
  }  
  impl WidgetImpl for PanelWidget {  
      fn map(\&self) {  
          self.parent\_map();  
          // Sicherstellen, dass Layer Shell korrekt initialisiert ist, bevor das Fenster angezeigt wird  
          self.obj().setup\_layer\_shell();  
      }  
       fn size\_allocate(\&self, width: i32, height: i32, baseline: i32) {  
          self.parent\_size\_allocate(width, height, baseline);  
          // Ggf. Layout der internen Boxen hier anpassen oder DrawingArea Größe  
      }  
  }  
  impl WindowImpl for PanelWidget {  
      // Fenster-spezifische Implementierungen, z.B. Schließen-Verhalten  
  }  
  impl ApplicationWindowImpl for PanelWidget {}

  // Implementierung der öffentlichen und privaten Methoden für PanelWidget  
  impl super::PanelWidget {  
      fn setup\_layer\_shell(\&self) {  
          let imp \= self.imp();  
          gtk\_layer\_shell::init\_for\_window(self);  
          gtk\_layer\_shell::set\_layer(self, gtk\_layer\_shell::Layer::Top);  
          gtk\_layer\_shell::set\_keyboard\_mode(self, gtk\_layer\_shell::KeyboardMode::None); // Panels benötigen i.d.R. keinen direkten Fokus  
          gtk\_layer\_shell::auto\_exclusive\_zone\_enable(self); // Platz reservieren  
          gtk\_layer\_shell::set\_namespace(self, "NovaDEPanel");

          let position \= \*imp.position.borrow();  
          match position {  
              PanelPosition::Top \=\> {  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Top, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Left, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Right, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Bottom, false);  
              }  
              PanelPosition::Bottom \=\> {  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Bottom, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Left, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Right, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Top, false);  
              }  
          }  
          self.set\_default\_height(imp.panel\_height.get());  
          // Margins könnten hier auch gesetzt werden, falls gewünscht  
          // gtk\_layer\_shell::set\_margin(self, gtk\_layer\_shell::Edge::Top, 5);  
      }

      fn update\_layout(\&self) {  
          let imp \= self.imp();  
          // Entferne alle Kinder aus start\_box, center\_box, end\_box  
          while let Some(child) \= imp.start\_box.first\_child() {  
              imp.start\_box.remove(\&child);  
          }  
          while let Some(child) \= imp.center\_box.first\_child() {  
              imp.center\_box.remove(\&child);  
          }  
          while let Some(child) \= imp.end\_box.first\_child() {  
              imp.end\_box.remove(\&child);  
          }

          // Füge Module entsprechend ihrer Reihenfolge und Position hinzu  
          // Diese Logik muss verfeinert werden, um die \`order\` Eigenschaft zu berücksichtigen  
          for widget in imp.modules\_start.borrow().iter() {  
              imp.start\_box.append(widget);  
          }  
          for widget in imp.modules\_center.borrow().iter() {  
              imp.center\_box.append(widget);  
          }  
          for widget in imp.modules\_end.borrow().iter() {  
              imp.end\_box.append(widget);  
          }  
      }

      fn update\_transparency(\&self) {  
          let imp \= self.imp();  
          let visual \= if imp.transparency\_enabled.get() {  
              self.display().rgba\_visual()  
          } else {  
              None // Oder Standard-Visual  
          };  
          self.set\_visual(visual.as\_ref()); // Benötigt GdkDisplay

          // Für echte Transparenz unter Wayland muss der Compositor dies unterstützen  
          // und das Fenster muss ggf. mit einem Alpha-Kanal gezeichnet werden.  
          // CSS kann auch für Hintergrundtransparenz verwendet werden.  
          self.queue\_draw();  
      }

      // Beispiel für das Verbinden des Draw-Signals, falls benutzerdefiniertes Zeichnen  
      // fn connect\_draw\_signal(\&self) {  
      //     if let Some(da) \= self.imp().drawing\_area.borrow().as\_ref() {  
      //        da.set\_draw\_func(glib::clone\!(@weak self as panel \=\> move |\_, cr, width, height| {  
      //            panel.imp().draw\_background\_and\_accent(cr, width, height);  
      //        }));  
      //    } else { // Wenn das PanelWindow selbst zeichnet (komplexer wegen Layer Shell)  
      //        self.connect\_realize(|widget| { // Realize statt draw für Fensterhintergrund  
      //            widget.set\_decorated(false); // Wichtig für custom drawing  
      //            if widget.imp().transparency\_enabled.get() {  
      //                 if let Some(surface) \= widget.surface() {  
      //                    surface.set\_opaque\_region(None); // Versuch für Transparenz  
      //                 }  
      //            }  
      //        });  
      //        // Das direkte Zeichnen auf einem GtkApplicationWindow ist nicht trivial.  
      //        // Besser ist ein Kind-Widget (GtkDrawingArea) zu verwenden.  
      //    }  
      // }  
  }

  // Private Implementierungsmethoden  
  impl PanelWidget {  
      fn add\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>, position: ModulePosition, \_order: i32) {  
          // TODO: Ordnung berücksichtigen  
          match position {  
              ModulePosition::Start \=\> {  
                  self.imp().modules\_start.borrow\_mut().push(module.clone().upcast());  
                  self.imp().start\_box.append(module);  
              }  
              ModulePosition::Center \=\> {  
                  self.imp().modules\_center.borrow\_mut().push(module.clone().upcast());  
                  self.imp().center\_box.append(module);  
              }  
              ModulePosition::End \=\> {  
                  self.imp().modules\_end.borrow\_mut().push(module.clone().upcast());  
                  self.imp().end\_box.append(module);  
              }  
          }  
          // Signal 'module-layout-changed' emittieren  
      }

      fn remove\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>) {  
          let widget\_ptr \= module.as\_ref().to\_glib\_none().0;  
          if self.imp().modules\_start.borrow\_mut().retain(|m| m.to\_glib\_none().0\!= widget\_ptr).len() \< self.imp().modules\_start.borrow().len() {  
               self.imp().start\_box.remove(module);  
          } else if self.imp().modules\_center.borrow\_mut().retain(|m| m.to\_glib\_none().0\!= widget\_ptr).len() \< self.imp().modules\_center.borrow().len() {  
               self.imp().center\_box.remove(module);  
          } else if self.imp().modules\_end.borrow\_mut().retain(|m| m.to\_glib\_none().0\!= widget\_ptr).len() \< self.imp().modules\_end.borrow().len() {  
               self.imp().end\_box.remove(module);  
          }  
          // Signal 'module-layout-changed' emittieren  
      }  
  }

* Eigenschaften (Properties):  
  Die GObject-Eigenschaften ermöglichen die Konfiguration und Zustandsabfrage des PanelWidget. Sie werden über das glib::Properties-Makro und die install\_properties-Methode im ObjectSubclass-Trait deklariert.47  
  **Tabelle: PanelWidget Eigenschaften**

| Eigenschaftsname | Typ | Zugriff | Standardwert | Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| position | PanelPosition | Lesen/Schreiben | Top | Bildschirmkante, an der das Panel verankert ist (Oben, Unten). |
| panel-height | i32 | Lesen/Schreiben | 36 | Höhe des Panels in Pixeln (Min: 24, Max: 128). |
| transparency-enabled | bool | Lesen/Schreiben | false | Gibt an, ob Transparenzeffekte für das Panel aktiv sind. |
| leuchtakzent-color | Option\<gdk::RGBA\> | Lesen/Schreiben | None | Farbe des Leuchtakzents. Wird typischerweise vom Theming-System aktualisiert. |
| leuchtakzent-intensity | f64 | Lesen/Schreiben | 0.5 | Intensität/Opazität des Leuchtakzents (Bereich: 0.0 bis 1.0). |

\*Bedeutung der Tabelle:\* Diese Tabelle bietet eine klare, strukturierte Definition der konfigurierbaren Aspekte des Panels. Sie ist essentiell für Entwickler, um die API des Widgets zu verstehen und es in Einstellungssysteme zu integrieren. Sie adressiert direkt die Anforderung der Anfrage nach der Definition von Eigenschaften mit exakten Typen und Initialwerten.

* **Interner Zustand:**  
  * modules\_start: RefCell\<Vec\<gtk::Widget\>\>: Speichert Referenzen auf die Panel-Module im Startbereich.  
  * modules\_center: RefCell\<Vec\<gtk::Widget\>\>: Speichert Referenzen auf die Panel-Module im Mittelbereich.  
  * modules\_end: RefCell\<Vec\<gtk::Widget\>\>: Speichert Referenzen auf die Panel-Module im Endbereich.  
  * Die Verwendung von RefCell ist notwendig für die innere Veränderlichkeit innerhalb des GObject-Systems, da GObject-Methoden typischerweise \&self erhalten.51

### **2.1.4. GTK-Widget-Implementierungsstrategie**

* **Basis-Widget**: Das PanelWidget erbt von gtk::ApplicationWindow.43 Diese Wahl ist entscheidend für die Integration mit gtk4-layer-shell, da dessen Funktionen wie init\_for\_window, set\_layer, set\_anchor und set\_margin auf einem gtk::Window operieren.26  
  * Die Initialisierung der Layer-Shell-Eigenschaften (gtk\_layer\_shell::init\_for\_window(self), etc.) muss erfolgen, bevor das Fenster zum ersten Mal realisiert (mapped) wird.29  
  * gtk\_layer\_shell::set\_layer(self.as\_ref(), gtk\_layer\_shell::Layer::Top) positioniert das Panel über normalen Anwendungsfenstern.  
  * gtk\_layer\_shell::set\_keyboard\_mode(self.as\_ref(), gtk\_layer\_shell::KeyboardMode::None) ist typisch für Panels, da sie selten direkten Tastaturfokus benötigen; dieser wird von den einzelnen Modulen gehandhabt.  
  * gtk\_layer\_shell::auto\_exclusive\_zone\_enable(self.as\_ref()) sorgt dafür, dass das Panel Platz auf dem Bildschirm reserviert und andere Fenster nicht verdeckt.  
* **Internes Layout**:  
  * Das PanelWidget verwendet eine panel\_widget.ui-Datei (Composite Template 55) oder definiert sein internes Layout programmatisch.  
  * Eine Haupt-gtk::Box (main\_box) mit horizontaler Orientierung dient als primärer Container.  
  * Innerhalb dieser main\_box befinden sich drei weitere gtk::Box-Widgets: start\_box, center\_box, und end\_box.42 Diese dienen zur Aufnahme der jeweiligen Panel-Module. start\_box und end\_box haben eine feste Größe basierend auf ihrem Inhalt, während center\_box den verbleibenden Raum einnimmt und sich horizontal ausdehnt (hexpand \= true).  
  * Alternativ kann gtk::CenterBox verwendet werden, wenn die UI-Definition dies unterstützt und die Anforderungen an die Ausrichtung der Kindelemente erfüllt.63  
* **Benutzerdefiniertes Zeichnen für "Leuchtakzent" und Hintergrund**:  
  * Falls CSS (box-shadow 36) für den "Leuchtakzent" oder komplexe Hintergründe nicht ausreicht oder die gewünschte Performance nicht liefert, wird ein gtk::DrawingArea eingesetzt.34  
  * Diese DrawingArea würde als unterste Ebene im PanelWidget platziert, oder das PanelWidget (als ApplicationWindow) muss seine Hintergrundzeichnung sorgfältig handhaben. Dies kann erreicht werden, indem das Fenster selbst transparent gemacht wird (widget.set\_visual(Some(\&display.rgba\_visual())) 34) und auf einer Kind-DrawingArea gezeichnet wird.  
  * Das draw-Signal der DrawingArea wird mit cairo-rs verwendet, um den Akzent und den Hintergrund zu zeichnen. Die Transparenz wird durch cairo::Context::set\_source\_rgba und die opacity-Eigenschaft von GtkWidget gesteuert.68

### **2.1.5. Methoden und Funktionssignaturen**

Die Methoden des PanelWidget definieren seine öffentliche API und interne Logik.

* **Öffentliche API (Auszug)**:  
  * pub fn add\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>, position: ModulePosition, order: i32) noexcept;  
    * Fügt ein gtk::Widget-basiertes Modul dem Panel hinzu.  
    * position: Enum (Start, Center, End), das den Bereich im Panel angibt.  
    * order: Ein i32-Wert, der die Reihenfolge innerhalb des Bereichs bestimmt (niedrigere Werte zuerst).  
  * pub fn remove\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>) noexcept;  
    * Entfernt ein zuvor hinzugefügtes Modul aus dem Panel.  
* **Interne Methoden (Auszug)**:  
  * fn setup\_layer\_shell(\&self) noexcept;  
    * Initialisiert und konfiguriert die gtk4-layer-shell-Eigenschaften basierend auf den aktuellen Panel-Einstellungen (Position, Höhe).  
  * fn update\_layout(\&self) noexcept;  
    * Ordnet die Module innerhalb der start\_box, center\_box und end\_box neu an, basierend auf ihrer order-Eigenschaft und aktuellen Konfiguration.  
  * fn draw\_background\_and\_accent(\&self, cr: \&cairo::Context, width: i32, height: i32) noexcept;  
    * Wird von der draw-Signal-Callback-Funktion der DrawingArea aufgerufen, um den benutzerdefinierten Hintergrund und den Leuchtakzent zu zeichnen. Verwendet leuchtakzent-color und leuchtakzent-intensity.  
  * fn update\_transparency(\&self) noexcept;  
    * Passt die Visuals des Fensters an, um Transparenz zu (de-)aktivieren.

**Tabelle: PanelWidget Methoden (Auswahl)**

| Signatur | Beschreibung | const | noexcept |
| :---- | :---- | :---- | :---- |
| pub fn new(app: \&gtk::Application) \-\> Self | Konstruktor, erstellt eine neue Instanz des PanelWidget. | Nein | Nein |
| pub fn add\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>, position: ModulePosition, order: i32) | Fügt ein Widget-Modul einem bestimmten Bereich (Start, Center, End) des Panels hinzu, unter Berücksichtigung der order. | Nein | Ja |
| pub fn remove\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>) | Entfernt das angegebene Widget-Modul aus dem Panel. | Nein | Ja |
| fn setup\_layer\_shell(\&self) | Interne Methode zur Konfiguration der gtk4-layer-shell-Parameter (Anker, Layer, Exklusivzone etc.) basierend auf den Panel-Eigenschaften wie position und panel-height. | Nein | Ja |
| fn update\_layout(\&self) | Interne Methode, die das Layout der Module in den Start-, Mittel- und Endbereichen aktualisiert, z.B. nach Hinzufügen/Entfernen eines Moduls oder einer Konfigurationsänderung. | Nein | Ja |

\*Bedeutung der Tabelle:\* Diese Tabelle ist entscheidend für Entwickler, die das \`PanelWidget\` verwenden oder erweitern, da sie einen klaren API-Vertrag bereitstellt und die Kernfunktionalitäten dokumentiert. Sie erfüllt die Anforderung der Anfrage nach exakten Methodensignaturen.

### **2.1.6. Signale**

Signale ermöglichen die Kommunikation von Zustandsänderungen oder Ereignissen des PanelWidget.

* **Benutzerdefinierte Signale**:  
  * module-layout-changed:  
    * Parameter: Keine.  
    * Emission: Wird emittiert, nachdem Module hinzugefügt, entfernt oder neu angeordnet wurden.  
    * Zweck: Ermöglicht anderen UI-Komponenten oder Logikmodulen, auf Änderungen im Panel-Layout zu reagieren.  
* **Verbundene Signale**:  
  * Lauscht auf ThemeChangedEvent von domain::theming::ThemingEngine:  
    * Handler-Aktion: Aktualisiert die Eigenschaft leuchtakzent-color und andere themenabhängige visuelle Aspekte. Fordert ein Neuzeichnen des Panels an (self.queue\_draw()).  
  * Verbindet sich mit notify::gtk-theme-name und notify::gtk-application-prefer-dark-theme von gtk::Settings::default() 10:  
    * Handler-Aktion: Lädt bei Bedarf Panel-spezifisches CSS neu oder passt Stile an, um Änderungen im System-Theme oder Dark-Mode-Präferenzen Rechnung zu tragen.

**Tabelle: PanelWidget emittierte Signale**

| Signalname | Parameter | Beschreibung |
| :---- | :---- | :---- |
| module-layout-changed | Keine | Wird emittiert, wenn sich die Anordnung oder der Satz der Module im Panel ändert. |

\*\*Tabelle: \`PanelWidget\` verbundene Signale\*\*

| Quelle | Signal | Handler-Aktion |
| :---- | :---- | :---- |
| domain::theming::ThemingEngine | ThemeChangedEvent | Aktualisiert leuchtakzent-color, fordert Neuzeichnen an. |
| gtk::Settings::default() | notify::gtk-theme-name | Lädt bei Bedarf panel-spezifisches CSS neu oder passt Stile an. |
| gtk::Settings::default() | notify::gtk-application-prefer-dark-theme | Passt Stile für Dark Mode an, lädt ggf. spezifisches CSS. |

\*Bedeutung der Tabellen:\* Diese Tabellen verdeutlichen die ereignisgesteuerten Interaktionen des \`PanelWidget\`. Dies ist entscheidend für das Verständnis seines dynamischen Verhaltens und für das Debugging.

### **2.1.7. Ereignisbehandlung**

* Das PanelWidget behandelt primär interne Layout-Aktualisierungen, die durch Eigenschaftsänderungen oder das Hinzufügen/Entfernen von Modulen ausgelöst werden.  
* Mausereignisse (z.B. enter-notify-event, leave-notify-event für Tooltips auf dem Panel selbst, falls vorhanden) werden über gtk::EventControllerMotion gehandhabt.70 Das Panel selbst wird jedoch in der Regel keinen komplexen Mausinteraktionen ausgesetzt sein; diese werden von den einzelnen Modulen übernommen.  
* Tastaturereignisse werden nicht direkt vom PanelWidget verarbeitet. Der Tastaturfokus wird von den einzelnen, fokussierbaren Panel-Modulen verwaltet.

### **2.1.8. Interaktionen**

* **domain::global\_settings\_and\_state\_management**:  
  * Liest die Panel-Konfiguration (Position, Höhe, Transparenzoptionen, Liste und Reihenfolge der Module) beim Start.  
  * Beobachtet Änderungen an diesen Einstellungen (z.B. über gio::Settings 21 oder ein anwendungsspezifisches Event-System), um das Panel dynamisch zu aktualisieren. Änderungen an Eigenschaften wie position oder panel-height führen zu Aufrufen von setup\_layer\_shell und update\_layout.  
* **system::compositor**:  
  * Die Interaktion erfolgt indirekt über die gtk4-layer-shell-Bibliothek.26 Das PanelWidget deklariert sich als Layer Surface (z.B. Layer::Top), setzt Anker und Margins, um seine Position und Größe relativ zum Output zu definieren.  
* **domain::theming::ThemingEngine**:  
  * Abonniert das ThemeChangedEvent, um Design-Tokens (insbesondere für leuchtakzent-color und Hintergrund) zu erhalten und anzuwenden. Dies löst ein Neuzeichnen des Panels aus.

### **2.1.9. Ausnahmebehandlung**

Zur robusten Fehlerbehandlung wird ein spezifischer Fehlertyp für das PanelWidget definiert.

* **enum PanelWidgetError** (definiert mit thiserror 72):  
  * LayerShellInitializationFailed(String): Wird zurückgegeben oder geloggt, wenn die Initialisierung mit gtk4-layer-shell fehlschlägt (z.B. wenn der Compositor das Protokoll nicht unterstützt).  
  * SettingsReadError(String): Wenn die Panel-Konfiguration nicht gelesen werden kann.  
  * InvalidModulePosition(String): Wenn versucht wird, ein Modul an einer ungültigen Position hinzuzufügen.  
* Fehler werden über das tracing-Crate geloggt 73, um Diagnose und Debugging zu erleichtern. Kritische Fehler, die die Funktionalität des Panels verhindern (z.B. LayerShellInitializationFailed), können dazu führen, dass das Panel nicht angezeigt wird, mit einer entsprechenden Log-Meldung.

### **2.1.10. Auflösung "Untersuchungsbedarf"**

* **Best Practices für gtk4-layer-shell-Integration**:  
  * Die Initialisierung der Layer-Shell-Eigenschaften (gtk\_layer\_shell::init\_for\_window, set\_layer, set\_anchor, set\_margin, auto\_exclusive\_zone\_enable) muss erfolgen, *bevor* das Panel-Fenster zum ersten Mal realisiert/gemappt wird. Dies geschieht typischerweise im constructed-Handler oder kurz vor dem ersten present()-Aufruf.26  
  * Der Tastaturinteraktivitätsmodus sollte sorgfältig gewählt werden. Für ein typisches Panel ist gtk\_layer\_shell::KeyboardMode::None oft angemessen, da die Panel-Module selbst den Fokus handhaben. KeyboardMode::OnDemand könnte relevant sein, wenn das Panel selbst oder bestimmte nicht-interaktive Bereiche des Panels temporär Fokus benötigen könnten.29  
  * Ein eindeutiger Namespace (z.B. "novade-panel") sollte mittels gtk\_layer\_shell::set\_namespace gesetzt werden. Dies hilft dem Compositor, verschiedene Layer-Shell-Clients zu identifizieren.29  
  * Für Multi-Monitor-Setups: Das Panel kann über gtk\_layer\_shell::set\_monitor einem spezifischen Monitor zugewiesen werden. Um Panels auf allen Monitoren darzustellen, müsste für jeden Monitor eine eigene PanelWidget-Instanz erstellt und konfiguriert werden. Die Liste der Monitore ist über gdk::Display::monitors() zugänglich.74 Änderungen in der Monitorkonfiguration (An-/Abstecken) können über Signale von gdk::Display (monitor-added, monitor-removed) überwacht werden.  
* **Implementierung des konfigurierbaren "Leuchtakzents" mit Cairo/GSK**:  
  * Das PanelWidget (oder eine dedizierte Kind-gtk::DrawingArea, die unter den Modul-Containern liegt) verbindet sich mit dem draw-Signal.  
  * Im Draw-Handler (fn draw\_background\_and\_accent):  
    1. Die aktuellen Werte der Eigenschaften leuchtakzent-color (ein gdk::RGBA) und leuchtakzent-intensity (ein f64 zwischen 0.0 und 1.0) werden abgerufen.  
    2. Der cairo::Context (cr) wird verwendet.  
    3. **Hintergrund zeichnen**: Zuerst wird der Panel-Hintergrund gezeichnet. Wenn Transparenz (transparency-enabled) aktiv ist, wird cr.set\_source\_rgba() mit einem Alpha-Wert \< 1.0 verwendet. Ansonsten eine deckende Farbe gemäß Theme. Abgerundete Ecken, falls spezifiziert, werden hier berücksichtigt (z.B. mit arc\_to und line\_to Pfaden).  
    4. **Leuchtakzent-Pfad definieren**: Ein Pfad wird für den Leuchteffekt erstellt. Dies könnte eine Linie oder ein schmales Rechteck entlang der Kante des Panels sein, die dem Bildschirmzentrum zugewandt ist. Die Position hängt von der position-Eigenschaft des Panels ab (oben oder unten).  
    5. **Leuchtakzent zeichnen**:  
       * **Farbe und Intensität**: cr.set\_source\_rgba() wird mit der leuchtakzent-color und einer durch leuchtakzent-intensity modulierten Alpha-Komponente aufgerufen.  
       * **Weicher Effekt**: Um einen weichen "Glow"-Effekt zu erzielen, können verschiedene Cairo-Techniken verwendet werden:  
         * **Gradienten**: Ein cairo::LinearGradient kann erstellt werden, der von der Akzentfarbe zu einer transparenten Version derselben Farbe oder zur Hintergrundfarbe übergeht. Der Gradient wird so ausgerichtet, dass er senkrecht zur Panelkante verläuft und nach außen hin ausblendet.41  
         * **Mehrfaches Zeichnen mit Unschärfe (simuliert)**: Da Cairo keine direkte Gausssche Unschärfe für Pfade bietet, kann ein ähnlicher Effekt durch mehrfaches Zeichnen des Akzentpfades mit leicht variierenden Offsets, Größen und abnehmender Deckkraft erzielt werden. Dies ist rechenintensiv und sollte mit Bedacht eingesetzt werden.  
         * **Schatten-API (falls anwendbar)**: Obwohl Cairo keine direkte box-shadow-Entsprechung für Pfade hat, könnte man einen Schatten simulieren, indem man eine versetzte, gefärbte und leicht transparente Version des Panelrands zeichnet und darüber den eigentlichen Panelinhalt.  
       * Die gezeichneten Elemente müssen die panel-height und die Gesamtbreite des Panels berücksichtigen.  
    6. Die GSK-Rendering-Pipeline von GTK4 wird diese Cairo-Operationen effizient auf die GPU übertragen.64 Es ist wichtig, queue\_draw() nur dann aufzurufen, wenn sich visuelle Aspekte tatsächlich ändern, um unnötiges Neuzeichnen zu vermeiden.  
  * Die Transparenz des Panel-Fensters selbst wird über gtk\_widget\_set\_opacity() 68 und die korrekte Konfiguration des GDK-Visuals für RGBA-Unterstützung gehandhabt, falls der Compositor dies erfordert und unterstützt.34

### **2.1.11. Dateistruktur**

Die Implementierung des PanelWidget wird in folgendem Verzeichnisbaum organisiert:

src/  
└── ui/  
    └── shell/  
        └── panel\_widget/  
            ├── mod.rs              // Öffentliche API, GObject Wrapper (PanelWidget struct)  
            ├── imp.rs              // Private GObject Implementierung (Subclass-Logik)  
            ├── panel\_widget.ui     // (Optional) XML-Definition für Composite Template  
            └── error.rs            // (Optional) Definition von PanelWidgetError

* mod.rs: Enthält die glib::wrapper\! Makrodefinition und öffentliche Methoden, die an die imp-Struktur delegieren.  
* imp.rs: Beinhaltet die \# Struktur, die \#\[glib::object\_subclass\] Implementierung und die Implementierungen für ObjectImpl, WidgetImpl, WindowImpl, und ApplicationWindowImpl. Hier werden Eigenschaften und Signale definiert und die Kernlogik des Widgets implementiert.  
* panel\_widget.ui: Falls Composite Templates für das interne Layout des Panels (z.B. die Anordnung von start\_box, center\_box, end\_box) verwendet werden, wird die XML-Struktur hier definiert.56  
* error.rs: Definiert PanelWidgetError unter Verwendung von thiserror.

Diese Struktur fördert die Modularität und Trennung von öffentlicher Schnittstelle und Implementierungsdetails, wie es in der gtk-rs Community üblich ist.5

### ---

**2.2. Sub-Modul: ui::shell::panel\_widget::AppMenuButton**

#### **2.2.1. Übersicht und Verantwortlichkeiten**

Das AppMenuButton ist ein spezialisiertes Panel-Modul, das als gtk::MenuButton (oder eine benutzerdefinierte Ableitung davon) implementiert wird. Seine Hauptverantwortung ist die Darstellung des globalen Anwendungsmenüs der aktuell fokussierten Applikation. Hierzu muss es:

1. Den app\_id (oder eine äquivalente Kennung) des aktiven Fensters ermitteln.  
2. Basierend auf dem app\_id das gio::MenuModel der aktiven Anwendung über D-Bus abrufen.  
3. Das abgerufene Menümodell in einem gtk::PopoverMenu darstellen, das beim Klick auf den Button erscheint.  
4. Das Aussehen des Buttons dynamisch an die aktive Anwendung anpassen (z.B. Icon und/oder Name anzeigen).

Die Komplexität dieser Komponente ergibt sich aus der Notwendigkeit, mit externen Systemkomponenten (Wayland Compositor für Fensterinformationen, D-Bus für Menüdaten) zu interagieren und auf Änderungen des Fensterfokus zu reagieren.

#### **2.2.2. Visuelles Design und Theming**

* **Anzeige**: Zeigt typischerweise das Icon der aktiven Anwendung. Falls kein Icon verfügbar ist oder keine Anwendung ein Menü bereitstellt, wird ein generisches "Anwendungsmenü"-Icon oder ein Platzhaltertext angezeigt.  
* **Beschriftung**: Kann optional den Namen der aktiven Anwendung neben dem Icon anzeigen, abhängig von der Konfiguration und dem verfügbaren Platz im Panel.  
* **Styling**:  
  * Als Instanz von gtk::MenuButton oder einer benutzerdefinierten, von gtk::Button abgeleiteten Klasse, die ein Popover öffnet. Es kann als ui::components::StyledButtonWidget implementiert werden, um ein konsistentes Erscheinungsbild mit anderen Buttons im Panel zu gewährleisten.  
  * **CSS-Knoten**: button (wenn von gtk::Button abgeleitet) oder menubutton (wenn von gtk::MenuButton).  
  * **CSS-Klassen**:  
    * .app-menu-button: Allgemeine Klasse für spezifisches Styling.  
    * .active-app: Wenn ein Anwendungsmenü erfolgreich geladen wurde.  
    * .no-app-menu: Wenn kein Menü für die aktive Anwendung verfügbar ist oder keine Anwendung fokussiert ist.  
* Der Tooltip des Buttons zeigt den Namen der aktiven Anwendung an, falls nicht bereits als Label sichtbar.76

#### **2.2.3. Datenstrukturen, Eigenschaften und Zustand**

Das AppMenuButton wird als GObject-Widget implementiert.

* **GObject-Definition (AppMenuButton)**:  
  *Auszug aus src/ui/shell/panel\_widget/app\_menu\_button/imp.rs (vereinfacht):*  
  Rust  
  use gtk::glib;  
  use gtk::subclass::prelude::\*;  
  use gtk::{gio, CompositeTemplate};  
  use std::cell::{Cell, RefCell};  
  use once\_cell::sync::Lazy;  
  use zbus::Connection; // \[12\]

  // Enum für den Status der Menüabfrage  
  \#  
  pub enum MenuFetchStatus {  
      \#\[default\]  
      Idle,  
      Loading,  
      Success,  
      Error(String), // Enthält Fehlermeldung  
  }

  static APP\_MENU\_BUTTON\_PROPERTIES: Lazy\<Vec\<glib::ParamSpec\>\> \= Lazy::new(|| {  
      vec\!  
  });

  \#  
  pub struct AppMenuButton {  
      // Eigenschaften  
      active\_app\_id: RefCell\<Option\<String\>\>,  
      active\_app\_name: RefCell\<Option\<String\>\>,  
      active\_app\_icon\_name: RefCell\<Option\<String\>\>,  
      has\_menu: Cell\<bool\>,  
      menu\_fetch\_status: RefCell\<MenuFetchStatus\>,

      // Interner Zustand  
      current\_menu\_model: RefCell\<Option\<gio::MenuModel\>\>,  
      dbus\_connection: RefCell\<Option\<Connection\>\>, // Zbus-Verbindung \[12\]

      // Referenz auf das GtkMenuButton-Widget selbst (oder das Popover, falls custom)  
      menu\_button\_widget: RefCell\<Option\<gtk::MenuButton\>\>, // Wird in constructed gesetzt  
  }

  \#\[glib::object\_subclass\]  
  impl ObjectSubclass for AppMenuButton {  
      const NAME: &'static str \= "NovaDEAppMenuButton";  
      type Type \= super::AppMenuButton;  
      type ParentType \= gtk::MenuButton; // Oder gtk::Button, wenn ein Popover manuell verwaltet wird

      fn new() \-\> Self {  
          Self {  
              active\_app\_id: RefCell::new(None),  
              active\_app\_name: RefCell::new(None),  
              active\_app\_icon\_name: RefCell::new(None),  
              has\_menu: Cell::new(false),  
              menu\_fetch\_status: RefCell::new(MenuFetchStatus::Idle),  
              current\_menu\_model: RefCell::new(None),  
              dbus\_connection: RefCell::new(None),  
              menu\_button\_widget: RefCell::new(None),  
          }  
      }

      fn class\_init(klass: \&mut Self::Class) {  
          klass.install\_properties(\&APP\_MENU\_BUTTON\_PROPERTIES);  
          // CSS-Name setzen  
          klass.set\_css\_name("appmenubutton");  
      }  
  }

  impl ObjectImpl for AppMenuButton {  
      fn constructed(\&self) {  
          self.parent\_constructed();  
          let obj \= self.obj();  
          // Speichere eine Referenz auf das Widget selbst für einfachen Zugriff  
          // self.menu\_button\_widget.replace(Some(obj.clone()));

          // Initialisiere D-Bus Verbindung und abonniere aktive Fensteränderungen  
          // Dies sollte idealerweise asynchron geschehen.  
          let widget \= obj.clone();  
          glib::MainContext::default().spawn\_local(async move {  
              match Connection::session().await { // \[12\]  
                  Ok(conn) \=\> {  
                      widget.imp().dbus\_connection.replace(Some(conn));  
                      // Hier Logik zum Abonnieren von Änderungen des aktiven Fensters einfügen  
                      // z.B. über einen internen Service, der Wayland-Events verarbeitet  
                      // widget.subscribe\_to\_active\_window\_changes();  
                  }  
                  Err(e) \=\> {  
                      tracing::error\!("Failed to connect to D-Bus for AppMenuButton: {}", e);  
                      widget.imp().menu\_fetch\_status.replace(MenuFetchStatus::Error(format\!("D-Bus connection failed: {}", e)));  
                      widget.update\_button\_appearance\_and\_state();  
                  }  
              }  
          });  
          obj.update\_button\_appearance\_and\_state(); // Initiales Aussehen  
      }

      fn properties() \-\> &'static {  
          APP\_MENU\_BUTTON\_PROPERTIES.as\_ref()  
      }

      fn property(\&self, \_id: usize, pspec: \&glib::ParamSpec) \-\> glib::Value {  
          match pspec.name() {  
              "active-app-name" \=\> self.active\_app\_name.borrow().to\_value(),  
              "active-app-icon-name" \=\> self.active\_app\_icon\_name.borrow().to\_value(),  
              "has-menu" \=\> self.has\_menu.get().to\_value(),  
              \_ \=\> unimplemented\!(),  
          }  
      }  
      // set\_property ist hier nicht nötig, da die Eigenschaften Read-only sind und intern gesetzt werden.  
  }  
  impl WidgetImpl for AppMenuButton {  
      fn map(\&self) {  
          self.parent\_map();  
          // Beim Sichtbarwerden ggf. aktuellen Status neu abfragen  
          self.obj().trigger\_menu\_update\_for\_current\_app();  
      }  
  }  
  impl ButtonImpl for AppMenuButton {} // Falls ParentType gtk::Button  
  impl MenuButtonImpl for AppMenuButton {} // Falls ParentType gtk::MenuButton

* **Eigenschaften (Properties)**:  
  **Tabelle: AppMenuButton Eigenschaften**

| Eigenschaftsname | Typ | Zugriff | Standardwert | Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| active-app-name | Option\<String\> | Nur Lesen | None | Name der Anwendung, deren Menü aktuell angezeigt wird oder angezielt ist. |
| active-app-icon-name | Option\<String\> | Nur Lesen | None | Icon-Name (für Theming) der Anwendung, deren Menü angezielt ist. |
| has-menu | bool | Nur Lesen | false | true, wenn ein Menü für die aktive Anwendung verfügbar und geladen ist. |

\*Bedeutung der Tabelle:\* Definiert den beobachtbaren Zustand des \`AppMenuButton\`, nützlich für Binding oder um auf Änderungen im Menü der aktiven Anwendung zu reagieren.

* **Interner Zustand**:  
  * active\_app\_id: RefCell\<Option\<String\>\>: Speichert die ID der aktuell fokussierten Anwendung.  
  * menu\_fetch\_status: RefCell\<MenuFetchStatus\>: Verfolgt den Zustand des Menüabrufs.  
  * current\_menu\_model: RefCell\<Option\<gio::MenuModel\>\>: Hält das aktuell geladene Menümodell.  
  * dbus\_connection: RefCell\<Option\<zbus::Connection\>\>: Die D-Bus-Verbindung für Abfragen.

#### **2.2.4. GTK-Widget-Implementierung**

* Das AppMenuButton erbt von gtk::MenuButton.77 Diese Klasse bietet bereits die Funktionalität, ein Popover beim Klick anzuzeigen.  
* Das Popover wird ein gtk::PopoverMenu sein.79  
* Die Eigenschaft menu-model des gtk::MenuButton (oder des internen gtk::PopoverMenu) wird dynamisch mit dem über D-Bus abgerufenen gio::MenuModel aktualisiert.  
  * gtk::MenuButton::set\_menu\_model(Some(menu\_model))  
  * Wenn kein Menü verfügbar ist, wird gtk::MenuButton::set\_menu\_model(None) gesetzt oder das Popover deaktiviert.

#### **2.2.5. Methoden und Funktionssignaturen**

* **Öffentliche Methoden (vom Panel oder einem Dienst für aktive Fenster aufgerufen)**:  
  * pub fn update\_active\_window\_info(\&self, app\_id: Option\<String\>, window\_title: Option\<String\>, icon\_name: Option\<String\>) noexcept;  
    * Wird aufgerufen, wenn sich das aktive Fenster *oder* dessen Metadaten ändern.  
    * Speichert app\_id, window\_title, icon\_name intern.  
    * Löst trigger\_menu\_update\_for\_current\_app aus.  
    * Aktualisiert sofort das Aussehen des Buttons (Icon/Label) basierend auf icon\_name und window\_title/app\_id.  
* **Interne Methoden**:  
  * fn trigger\_menu\_update\_for\_current\_app(\&self) noexcept;  
    * Prüft, ob active\_app\_id gesetzt ist.  
    * Wenn ja, startet die asynchrone fetch\_menu\_for\_app-Operation.  
    * Setzt menu\_fetch\_status auf Loading.  
    * Aktualisiert das Button-Aussehen (z.B. Ladeindikator).  
  * async fn fetch\_menu\_for\_app(dbus\_conn: Connection, app\_id: String) \-\> Result\<gio::MenuModel, AppMenuError\>;  
    * Diese Funktion ist async und wird mit glib::MainContext::spawn\_local ausgeführt.7  
    * Versucht, das gio::MenuModel für den gegebenen app\_id über D-Bus zu beziehen (siehe 2.2.8 Interaktionen).  
    * Gibt das gio::MenuModel oder einen AppMenuError zurück.  
  * fn handle\_menu\_fetch\_result(\&self, result: Result\<gio::MenuModel, AppMenuError\>) noexcept;  
    * Wird im glib::MainContext aufgerufen, nachdem fetch\_menu\_for\_app abgeschlossen ist.  
    * Aktualisiert current\_menu\_model, has\_menu, und menu\_fetch\_status.  
    * Ruft display\_menu und update\_button\_appearance\_and\_state auf.  
  * fn display\_menu(\&self) noexcept;  
    * Setzt das current\_menu\_model auf den gtk::MenuButton.  
  * fn update\_button\_appearance\_and\_state(\&self) noexcept;  
    * Aktualisiert Icon (z.B. gtk::Image::set\_from\_icon\_name 84) und Label des gtk::MenuButton basierend auf active\_app\_icon\_name, active\_app\_name und menu\_fetch\_status.  
    * Setzt den sensitive-Zustand des Buttons (z.B. deaktiviert, wenn kein Menü geladen werden kann oder Loading).  
    * Aktualisiert die GObject-Properties (active-app-name, active-app-icon-name, has-menu) und emittiert notify Signale.

**Tabelle: AppMenuButton Methoden (Auswahl)**

| Signatur | Beschreibung | async | noexcept |
| :---- | :---- | :---- | :---- |
| pub fn update\_active\_window\_info(\&self, app\_id: Option\<String\>, window\_title: Option\<String\>, icon\_name: Option\<String\>) | Aktualisiert die Informationen über das aktive Fenster und löst ggf. eine Menüaktualisierung aus. | Nein | Ja |
| fn trigger\_menu\_update\_for\_current\_app(\&self) | Startet den Prozess zum Abrufen und Anzeigen des Menüs für die aktuell zwischengespeicherte app\_id. | Nein | Ja |
| async fn fetch\_menu\_for\_app(dbus\_conn: Connection, app\_id: String) \-\> Result\<gio::MenuModel, AppMenuError\> | Ruft asynchron das GMenuModel für die gegebene app\_id über D-Bus ab. | Ja | Nein |
| fn handle\_menu\_fetch\_result(\&self, result: Result\<gio::MenuModel, AppMenuError\>) | Verarbeitet das Ergebnis von fetch\_menu\_for\_app, aktualisiert den internen Zustand und die UI. | Nein | Ja |

#### **2.2.6. Signale**

* **Benutzerdefinierte Signale**: Keine spezifischen benutzerdefinierten Signale für diese Komponente vorgesehen. Es erbt die Signale von gtk::MenuButton (z.B. clicked, activate).  
* **Verbundene Signale**:  
  * Intern: Lauscht auf ein Signal von einem übergeordneten Dienst (z.B. innerhalb von ui::shell), das Änderungen des aktiven Fensters (app\_id, Titel, Icon) meldet.

#### **2.2.7. Ereignisbehandlung**

* Die Hauptinteraktion ist der Klick auf den Button, der durch die gtk::MenuButton-Basisklasse gehandhabt wird und das Popover mit dem Menü anzeigt.  
* Interne Reaktionen auf die Ergebnisse der asynchronen D-Bus-Menüabfrage und auf Änderungen des aktiven Fensters sind entscheidend für die dynamische Aktualisierung.

#### **2.2.8. Interaktionen**

* **system::compositor (Fensterinformationen)**:  
  * Das AppMenuButton selbst interagiert nicht direkt mit dem Compositor. Es ist auf einen Dienst innerhalb der ui::shell angewiesen, der Informationen über das aktive Fenster bereitstellt. Dieser Dienst nutzt Wayland-Protokolle.  
  * **Wayland-Protokolle**:  
    * wlr-foreign-toplevel-management-unstable-v1: Dieses Protokoll ermöglicht es einem Client (dem NovaDE-Shell-Dienst), eine Liste von Toplevel-Fenstern zu erhalten und deren Zustände (inkl. app\_id, title, state) zu überwachen. Der Dienst würde das activated-Ereignis nutzen, um das aktuell fokussierte Fenster zu identifizieren.85  
    * ext-foreign-toplevel-list-v1: Ein alternatives oder ergänzendes Protokoll, das ebenfalls zur Auflistung von Toplevel-Fenstern dient.85  
  * Die Implementierung dieser Wayland-Client-Logik sollte zentral in einem ui::shell-Modul erfolgen (z.B. ui::shell::active\_window\_service) und nicht im AppMenuButton selbst, um Redundanz zu vermeiden und die Komplexität zu kapseln. Dieser Dienst würde dann ein internes Signal oder einen Event für das AppMenuButton bereitstellen.  
* **D-Bus (Menüabruf)**:  
  * Sobald der app\_id des aktiven Fensters bekannt ist, wird versucht, dessen Menümodell über D-Bus abzurufen.  
  * **Primärer Mechanismus (org.gtk.Menus)**:  
    * GTK4-Anwendungen, die GApplication verwenden, exportieren ihre Menüs (typischerweise GMenuModel für Anwendungsmenü und Menüleiste) oft über D-Bus unter ihrem eigenen Bus-Namen (welcher dem app\_id entspricht, z.B. org.gnome.TextEditor).  
    * Der Standard-Objektpfad für das Menü ist oft /org/gtk/menus/menubar oder ein ähnlicher, durch GApplication festgelegter Pfad.91  
    * Die Schnittstelle ist org.gtk.Menus.  
    * gio::DBusMenuModel::new(bus\_name, object\_path) kann verwendet werden, um ein GMenuModel direkt von einem D-Bus-Dienst zu erstellen, was die Details der Methodenaufrufe abstrahiert.21  
  * **Fallback-Mechanismus (com.canonical.AppMenu.Registrar)**:  
    * Ein älterer Mechanismus, der von Unity verwendet wurde. Anwendungen registrieren ihre Fenster-ID und den D-Bus-Pfad zu ihrem Menü bei diesem Dienst.15  
    * Dienstname: com.canonical.AppMenu.Registrar  
    * Objektpfad: /com/canonical/AppMenu/Registrar  
    * Schnittstelle: com.canonical.AppMenu.Registrar  
    * Methode: GetMenuForWindow(window\_id\_uint32). Dies ist problematisch in einer reinen Wayland-Umgebung, da X11-Fenster-IDs nicht direkt verfügbar oder relevant sind. Eine Wayland-kompatible Anwendung müsste ihren Menüpfad auf andere Weise bekannt geben.  
  * **D-Bus-Client-Implementierung**: Das zbus-Crate wird verwendet, um D-Bus-Proxies zu erstellen und Methoden aufzurufen.12  
    * Ein zbus::Proxy wird für den Zieldienst erstellt (entweder der app\_id oder com.canonical.AppMenu.Registrar).  
    * Die entsprechenden Methoden werden asynchron aufgerufen.  
    * Das Ergebnis (oft ein Pfad zu einem DBusMenu-Objekt) wird verwendet, um ein gio::MenuModel zu instanziieren, typischerweise mit gio::DBusMenuModel.

**Tabelle: AppMenuButton D-Bus Interaktionen**

| Interaktion | Zieldienst (Primär) | Objektpfad (Primär) | Schnittstelle (Primär) | Methode/Eigenschaft (Primär) | Zieldienst (Fallback) | Objektpfad (Fallback) | Schnittstelle (Fallback) | Methode (Fallback) |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| GMenuModel für aktive Anwendung abrufen | \[app\_id\_der\_aktiven\_Anwendung\] | /org/gtk/menus/menubar (oder Konvention) | org.gtk.Menus (oder org.freedesktop.DBus.Properties) | gio::DBusMenuModel::new(bus\_name, object\_path) (abstrahiert Methodenaufrufe) | com.canonical.AppMenu.Registrar | /com/canonical/AppMenu/Registrar | com.canonical.AppMenu.Registrar | GetMenuForWindow (XID-abhängig) oder App registriert Menüpfad |

\*Bedeutung der Tabelle:\* Verdeutlicht die komplexen D-Bus-Interaktionen, die für das Abrufen von Anwendungsmenüs erforderlich sind. Dies ist entscheidend für die Implementierung und das Debugging, insbesondere angesichts der verschiedenen Mechanismen, über die Menüs bereitgestellt werden können.

#### **2.2.9. Ausnahmebehandlung**

* **enum AppMenuError** (definiert mit thiserror 72):  
  * WaylandError(String): Fehler beim Abrufen von Informationen zum aktiven Fenster.  
  * DBusConnectionError(zbus::Error): Fehler bei der D-Bus-Kommunikation.  
  * MenuServiceUnavailable(String): Der D-Bus-Dienst für die Anwendung (z.B. app\_id oder AppMenu.Registrar) ist nicht erreichbar.  
  * MenuNotFound(String): Die Anwendung (app\_id) exportiert kein bekanntes Menü oder das Menü ist leer.  
  * MenuModelParseError(String): Fehler beim Parsen oder Interpretieren der Menüdaten.  
* Im Fehlerfall zeigt der AppMenuButton einen deaktivierten Zustand oder ein generisches Icon an. Fehlerdetails werden über tracing geloggt.73

#### **2.2.10. Auflösung "Untersuchungsbedarf"**

* **Zuverlässige Methode zur Ermittlung des aktiven Fensters/app\_id unter Wayland**:  
  * Die bevorzugte Methode ist die Verwendung des wlr-foreign-toplevel-management-unstable-v1-Protokolls.86 Ein zentraler Shell-Dienst (nicht der AppMenuButton selbst) agiert als Client dieses Protokolls.  
  * Der Dienst bindet sich an den globalen zwlr\_foreign\_toplevel\_manager\_v1.  
  * Für jedes gemeldete Toplevel (zwlr\_foreign\_toplevel\_handle\_v1) lauscht der Dienst auf die Ereignisse app\_id, title und state.  
  * Das state-Ereignis enthält Flags, darunter activated. Das Toplevel mit dem activated-Flag ist das aktuell fokussierte Fenster.  
  * Die smithay-client-toolkit 85 könnte Rust-Abstraktionen für dieses Protokoll bereitstellen. Falls nicht, ist die direkte Verwendung von wayland-client mit den wayland-protocols-Bindings (speziell wlr-protocols) notwendig.89  
  * Dieser zentrale Dienst stellt dann die Informationen über das aktive Fenster (insbesondere app\_id, title, icon\_name) dem AppMenuButton und anderen interessierten UI-Komponenten über ein internes Event-System oder Signale zur Verfügung.  
* **Ermittlung und Konsumierung von GMenuModel via D-Bus**:  
  * **Primärer Pfad (für GTK4-Anwendungen)**: Moderne GTK-Anwendungen, die GApplication verwenden, exportieren ihr Hauptmenü (GMenuModel) typischerweise über D-Bus auf ihrem eigenen, durch den app\_id bestimmten Bus-Namen. Der Objektpfad ist oft standardisiert, z.B. /org/gtk/menus/menubar oder ein anderer Pfad, den GApplication für diesen Zweck nutzt.91 Ein gio::DBusMenuModel wird dann mit diesem Bus-Namen und Objektpfad instanziiert, um das Menümodell zu erhalten.21  
  * **Fallback (StatusNotifierItem)**: Falls eine Anwendung ein StatusNotifierItem bereitstellt, kann dessen Menu-Eigenschaft einen D-Bus-Objektpfad zu einem Menü (oft im com.canonical.dbusmenu-Format) enthalten.102 Dies ist relevant, wenn die Anwendung primär über ein Tray-Icon interagiert.  
  * **Fallback (AppMenu Registrar)**: Der com.canonical.AppMenu.Registrar D-Bus-Dienst ist ein älterer Mechanismus.15 Seine Verwendung in einer reinen Wayland-Umgebung ist aufgrund der Abhängigkeit von X11-Fenster-IDs problematisch und sollte nur als letzte Option in Betracht gezogen werden, falls Anwendungen keine anderen Mechanismen anbieten.  
  * **Implementierungsentscheidung**: Die Strategie sollte sein, zuerst den primären Pfad (org.gtk.Menus auf dem app\_id-Bus) zu versuchen. Schlägt dies fehl und ist ein StatusNotifierItem für die App vorhanden, kann dessen Menüpfad versucht werden. Der AppMenuRegistrar wird aufgrund seiner X11-Lastigkeit tendenziell vermieden.  
  * Das AppMenuButton verwendet das erhaltene gio::MenuModel, um seinen internen gtk::PopoverMenu zu füllen.79

Die Implementierung des AppMenuButton erfordert eine sorgfältige Orchestrierung asynchroner Operationen für Wayland-Events und D-Bus-Aufrufe, um die UI reaktionsfähig zu halten (glib::MainContext::spawn\_local 7). Fehlerzustände (z.B. keine aktive Anwendung, keine Menüdaten, D-Bus-Fehler) müssen robust gehandhabt und dem Benutzer klar signalisiert werden (z.B. durch ein deaktiviertes oder generisches Icon).

#### **2.2.11. Dateistruktur**

src/  
└── ui/  
    └── shell/  
        └── panel\_widget/  
            └── app\_menu\_button/  
                ├── mod.rs          // Öffentliche API, GObject Wrapper (AppMenuButton struct)  
                ├── imp.rs          // Private GObject Implementierung  
                ├── dbus.rs         // Logik für D-Bus Interaktionen (Menüabruf)  
                └── error.rs        // Definition von AppMenuError

Diese Struktur kapselt die Komplexität des AppMenuButton und trennt die D-Bus-Logik klar ab.  
---

**(Hinweis: Die detaillierte Ausarbeitung weiterer Submodule des PanelWidget wie WorkspaceIndicatorWidget, ClockDateTimeWidget, SystemTrayEquivalentWidget etc. würde einem ähnlichen Detaillierungsgrad folgen und die spezifischen "Untersuchungsbedarfe" adressieren. Insbesondere das SystemTrayEquivalentWidget erfordert eine tiefgreifende Auseinandersetzung mit der StatusNotifierItem-Spezifikation und deren D-Bus-Implementierung mittels zbus, wie in der Gliederung angedeutet.102)**  
Die Implementierung eines SystemTrayEquivalentWidget ist ein komplexes Unterfangen, da Wayland selbst kein natives "System Tray"-Protokoll definiert. Die De-facto-Standardlösung ist die StatusNotifierItem (SNI) Spezifikation von Freedesktop.org, die auf D-Bus basiert.102  
Ein SystemTrayEquivalentWidget müsste folgende Kernkomponenten umfassen:

1. **StatusNotifierHost-Registrierung**: Das Panel (oder dieses Widget) muss sich als org.freedesktop.StatusNotifierHost auf dem Session-Bus registrieren. Dies signalisiert dem StatusNotifierWatcher, dass ein Host für Items vorhanden ist.106 Die Registrierung erfolgt typischerweise durch das Anfordern eines eindeutigen Bus-Namens (z.B. org.freedesktop.StatusNotifierHost-PID oder org.freedesktop.StatusNotifierHost-NovaDE).  
2. **Interaktion mit StatusNotifierWatcher**:  
   * Der StatusNotifierWatcher (org.freedesktop.StatusNotifierWatcher) ist der zentrale Dienst zur Verwaltung von SNIs.102  
   * Das Widget muss diesen Watcher auf dem D-Bus finden (Standardname org.freedesktop.StatusNotifierWatcher, Pfad /org/freedesktop/StatusNotifierWatcher).  
   * Es muss die Methode RegisterStatusNotifierHost am Watcher aufrufen, um sich selbst als Host zu registrieren.  
   * Es muss die Eigenschaft RegisteredStatusNotifierItems des Watchers abfragen, um eine initiale Liste aller bereits vorhandenen SNIs zu erhalten.  
   * Es muss die Signale StatusNotifierItemRegistered und StatusNotifierItemUnregistered des Watchers abonnieren, um dynamisch auf neue oder entfernte SNIs zu reagieren. zbus wird hierfür verwendet, um Signal-Handler einzurichten.109  
3. **Interaktion mit einzelnen StatusNotifierItems**:  
   * Für jeden von StatusNotifierWatcher gemeldeten Dienstnamen eines SNI (z.B. org.freedesktop.StatusNotifierItem-PID-ID) muss ein zbus::Proxy erstellt werden.17  
   * Über diesen Proxy werden die Eigenschaften des SNI ausgelesen: Category, Id, Title, Status, WindowId, IconName, IconPixmap, OverlayIconName, OverlayIconPixmap, AttentionIconName, AttentionIconPixmap, AttentionMovieName, ToolTip, ItemIsMenu, Menu.102  
   * Signale des SNI (z.B. NewIcon, NewStatus, NewToolTip, NewMenu) müssen abonniert werden, um auf Änderungen zu reagieren und die Darstellung des entsprechenden Indikator-Widgets im Panel zu aktualisieren.16  
4. **Darstellung der Indikatoren**:  
   * Für jedes aktive SNI wird ein kleines Widget im SystemTrayEquivalentWidget (das selbst eine gtk::Box oder ein ähnlicher Container ist) angezeigt.  
   * **Icon**: IconName wird verwendet, um ein themenbasiertes Icon über gtk::Image::from\_icon\_name zu laden.84 Falls IconPixmap bereitgestellt wird, müssen die Rohpixeldaten (oft ein Array von Tupeln (width, height, data)) in ein gdk\_pixbuf::Pixbuf konvertiert werden (z.B. mit Pixbuf::from\_mut\_slice oder PixbufLoader, falls die Daten gestreamt ankommen, was hier aber unwahrscheinlich ist) und dann in einem gtk::Image angezeigt werden.117  
   * **Tooltip**: Die ToolTip-Eigenschaft des SNI (eine Struktur mit Titel, Text, Icon) wird verwendet, um einen Tooltip für das Indikator-Widget mittels gtk::Widget::set\_tooltip\_markup oder gtk::Widget::set\_tooltip\_text zu setzen.76  
   * **Status**: Die Status-Eigenschaft (Passive, Active, NeedsAttention) kann verwendet werden, um das Aussehen des Indikators anzupassen (z.B. Hervorhebung bei NeedsAttention).  
5. **Interaktion mit den Indikatoren**:  
   * **Linksklick (Activate)**: Ein Klick auf das Indikator-Widget ruft die Activate(x, y)-Methode des SNI über D-Bus auf.103  
   * **Rechtsklick (ContextMenu)**: Ein Rechtsklick ruft die ContextMenu(x, y)-Methode des SNI auf. Wenn die ItemIsMenu-Eigenschaft true ist und die Menu-Eigenschaft einen gültigen D-Bus-Pfad zu einem com.canonical.dbusmenu-Objekt enthält, wird dieses Menü abgerufen (mittels gio::DBusMenuModel 96) und als gtk::PopoverMenu angezeigt.79  
   * **Scrollen**: Mausrad-Events über dem Indikator rufen die Scroll(delta, orientation)-Methode des SNI auf.  
6. **Asynchronität**: Alle D-Bus-Interaktionen (Methodenaufrufe, Signal-Handling) müssen asynchron mit glib::MainContext::spawn\_local erfolgen, um die UI nicht zu blockieren.7

Die "Alternativen unter Wayland" 104 beziehen sich darauf, dass Wayland selbst kein Tray-Protokoll spezifiziert. StatusNotifierItem ist die etablierte D-Bus-basierte Lösung. Einige Desktop-Umgebungen könnten eigene Protokolle haben, aber für eine breite Kompatibilität ist SNI der Standard. Die Herausforderung besteht darin, dass nicht alle Anwendungen SNI korrekt oder vollständig implementieren.

## ---

**3\. Übergreifende Belange – Initiale Spezifikationen**

Dieser Abschnitt definiert initiale Strategien für Aspekte, die mehrere UI-Module betreffen und eine konsistente Handhabung erfordern.

### **3.1. UI-Zustandsverwaltungsstrategie**

Die Verwaltung des UI-Zustands ist entscheidend für eine reaktive und wartbare Benutzeroberfläche. In NovaDE wird ein mehrschichtiger Ansatz verfolgt, der die Stärken von GObject mit Rust-Idiomen kombiniert:

* **GObject-Eigenschaften für Widget-Zustand**:  
  * Der primäre Mechanismus zur Verwaltung des Zustands einzelner Widgets sind GObject-Eigenschaften. Diese werden mit dem glib::Properties-Derive-Makro und klass.install\_properties() in der ObjectSubclass-Implementierung definiert.47  
  * Beispiel: Die panel-height-Eigenschaft des PanelWidget.  
  * Änderungen an diesen Eigenschaften lösen automatisch "notify::property-name"-Signale aus, auf die andere Teile der UI oder die interne Logik des Widgets reagieren können. Explizite Benachrichtigung kann mit self.obj().notify\_propertyName() erzwungen werden, falls die automatische Benachrichtigung nicht ausreicht oder benutzerdefinierte Logik vor der Benachrichtigung ausgeführt werden muss.  
* **Benutzerdefinierte GObject-Signale**:  
  * Für komplexere Zustandsänderungen oder Ereignisse, die nicht direkt durch eine einzelne Eigenschaftsänderung abgebildet werden, werden benutzerdefinierte GObject-Signale definiert.115  
  * Beispiel: Das module-layout-changed-Signal des PanelWidget.  
  * Signale werden in ObjectImpl::signals() definiert und können mit self.obj().emit\_by\_name::\<()\>("signal-name", &\[\&param1, \&param2\]) ausgelöst werden.  
* **Rc\<RefCell\<T\>\> für gemeinsam genutzten UI-Zustand**:  
  * Für UI-Zustände, die von mehreren Widgets gemeinsam genutzt werden und nicht in einer direkten GObject-Eltern-Kind-Beziehung stehen oder nicht sinnvoll als globale GSettings abgebildet werden können, wird das Rust-Idiom Rc\<RefCell\<T\>\> verwendet.51  
  * Rc ermöglicht das Teilen des Besitzes im Single-Threaded-Kontext des GTK-Mainloops.  
  * RefCell ermöglicht die innere Veränderlichkeit (mutable borrows zur Laufzeit geprüft).  
  * Dies ist nützlich für z.B. einen gemeinsam genutzten D-Bus-Verbindungsmanager, der von mehreren UI-Komponenten verwendet wird, oder für View-Modelle, die Daten für mehrere, lose gekoppelte Widgets halten.  
  * Vorsicht ist geboten, um Zyklen von Rc-Referenzen zu vermeiden, die zu Speicherlecks führen können. Weak\<RefCell\<T\>\> kann hier Abhilfe schaffen.  
* **Datenbindung (Property Binding)**:  
  * GObject-Eigenschaftsbindungen (GObject.bind\_property()) werden intensiv genutzt, um UI-Elemente direkt an Zustandseigenschaften zu koppeln. Dies reduziert manuellen Synchronisationscode und fördert eine deklarative UI-Logik.  
  * Beispiel: Die label-Eigenschaft eines gtk::Label könnte an eine String-Eigenschaft eines View-Modell-Objekts gebunden werden.  
* **Adaption von MVVM/MVC-Mustern**:  
  * Obwohl GTK nicht explizit für ein bestimmtes UI-Architekturmuster wie MVVM oder MVC ausgelegt ist, können deren Prinzipien adaptiert werden:  
    * **Model**: Repräsentiert die Anwendungsdaten und Geschäftslogik (primär in der domain-Schicht, aber auch UI-spezifische Zustandsmodelle).  
    * **View**: Die GTK-Widgets selbst.  
    * **ViewModel/Controller**: GObject-Instanzen, die UI-spezifische Logik und Zustand halten (ViewModel-Aspekt) und Benutzerinteraktionen verarbeiten (Controller-Aspekt). GObject-Eigenschaften des ViewModels werden an die View (Widgets) gebunden. Methoden im ViewModel/Controller reagieren auf UI-Events und interagieren mit dem Model.  
* **Kommunikation mit unteren Schichten**:  
  * Zustandsänderungen, die von der domain- oder system-Schicht ausgehen (z.B. durch Ereignisse oder Callbacks von asynchronen Operationen), werden in UI-Zustandsaktualisierungen übersetzt.  
  * Dies geschieht typischerweise innerhalb von Closures, die mit glib::MainContext::spawn\_local auf dem UI-Thread ausgeführt werden, um Thread-Sicherheit zu gewährleisten.7  
  * Beispiel: Ein NetworkStatusChangedEvent aus der system-Schicht könnte die icon-name-Eigenschaft eines NetworkIndicatorWidget aktualisieren.

Dieser Ansatz ermöglicht eine klare Trennung der Belange, nutzt die Stärken des GObject-Systems für Widget-spezifischen Zustand und bietet gleichzeitig flexible Rust-basierte Lösungen für komplexere oder gemeinsam genutzte UI-Zustände.

### **3.2. Fehlerbehandlungs-Framework für die UI-Schicht**

Eine konsistente und benutzerfreundliche Fehlerbehandlung ist unerlässlich.

* **Fehlerdefinition mit thiserror**:  
  * Für jedes Hauptmodul der UI-Schicht (z.B. ui::shell, ui::control\_center) und ggf. für komplexe Submodule (z.B. AppMenuButton) werden spezifische Error-Enums mit thiserror::Error definiert.72 Beispiel: PanelWidgetError, AppMenuError.  
  * Diese modul-spezifischen Fehler werden in einem übergeordneten UI-Fehler-Enum (z.B. NovaUiError) zusammengefasst, ebenfalls unter Verwendung von \#\[from\]-Attributen in thiserror für eine einfache Konvertierung.  
    Rust  
    // Beispiel: src/ui/error.rs  
    use thiserror::Error;

    \#  
    pub enum PanelWidgetError {  
        \#\[error("Layer shell initialization failed: {0}")\]  
        LayerShellInitializationFailed(String),  
        // Weitere Panel-spezifische Fehler  
    }

    \#  
    pub enum AppMenuError {  
        \#\[error("Failed to get active window info from Wayland: {0}")\]  
        WaylandError(String),  
        \#  
        DBusConnectionError(\#\[from\] zbus::Error),  
        \#\[error("Menu service for app '{0}' unavailable")\]  
        MenuServiceUnavailable(String),  
        \#\[error("Menu not found for app '{0}'")\]  
        MenuNotFound(String),  
    }

    \#  
    pub enum NovaUiError {  
        \#\[error("Panel widget error: {0}")\]  
        Panel(\#\[from\] PanelWidgetError),  
        \#\[error("AppMenu button error: {0}")\]  
        AppMenu(\#\[from\] AppMenuError),  
        \#  
        Theming(String), // Fehler von ui::theming\_gtk  
        \#\[error("I/O error: {0}")\]  
        Io(\#\[from\] std::io::Error),  
        // Weitere Fehlerkategorien  
    }

* **Fehlerdarstellung**:  
  * **Kritische Fehler**: Fehler, die die grundlegende Funktionalität einer Komponente oder der UI stark beeinträchtigen (z.B. D-Bus-Verbindung nicht möglich, Layer-Shell-Initialisierung fehlgeschlagen), werden dem Benutzer über ein gtk::AlertDialog mitgeteilt. Der Dialog sollte eine klare Fehlermeldung und ggf. Vorschläge zur Fehlerbehebung oder einen Hinweis auf Log-Dateien enthalten.  
  * **Nicht-kritische Fehler**: Weniger schwerwiegende Fehler (z.B. ein einzelnes Panel-Modul kann nicht geladen werden, eine Einstellung kann nicht gelesen werden) werden als NotificationPopupWidget (siehe ui::notifications\_frontend) oder durch eine Zustandsänderung im Widget selbst (z.B. ausgegrautes Icon, Fehlermeldung im Tooltip) angezeigt.  
  * Fehlermeldungen für den Benutzer werden internationalisiert (i18n).  
* **Fehlerpropagation**: Fehler aus unteren Schichten (domain, system) werden in entsprechende NovaUiError-Varianten umgewandelt und nach oben propagiert oder an der Stelle behandelt, an der sie für die UI relevant werden.

### **3.3. Logging-Strategie**

Strukturiertes Logging ist für Diagnose und Debugging unerlässlich.

* **Bibliothek**: Das tracing-Crate wird für alle Logging-Aufgaben in der UI-Schicht verwendet.73  
* **Log-Level**:  
  * trace\!: Sehr detaillierte Informationen für tiefgreifendes Debugging (z.B. einzelne D-Bus-Nachrichten, detaillierte Widget-Zustandsänderungen). Standardmäßig deaktiviert.  
  * debug\!: Informationen, die für das Debugging nützlich sind (z.B. Erstellung von Widgets, Aufruf wichtiger interner Methoden, empfangene Ereignisse).  
  * info\!: Allgemeine Informationen über den Betrieb (z.B. Modul geladen, Einstellung geändert).  
  * warn\!: Unerwartete, aber nicht unbedingt fehlerhafte Zustände (z.B. optionale Konfigurationsdatei nicht gefunden, Fallback-Verhalten aktiviert).  
  * error\!: Fehlerzustände, die die Funktionalität beeinträchtigen (z.B. D-Bus-Aufruf fehlgeschlagen, Widget konnte nicht erstellt werden). Details zum Fehlerobjekt werden mitgeloggt.  
* **Strukturierte Felder**: Log-Nachrichten sollen relevante Kontextinformationen als strukturierte Felder enthalten.  
  * Beispiel: tracing::debug\!(widget\_name \= %self.widget\_name(), event \=?event\_type, "Event received");  
* **Span-Nutzung**: tracing::span\! wird verwendet, um wichtige Operationen oder Lebenszyklen von Komponenten zu umfassen, insbesondere bei asynchronen Abläufen.  
* **Konfiguration**:  
  * Die Konfiguration des tracing-Subscribers (z.B. tracing\_subscriber::fmt für Konsolenausgabe oder tracing\_journald für systemd-journal-Integration) erfolgt im Hauptanwendungseinstiegspunkt (main.rs).  
  * Die Standard-Logstufe für Entwicklungs-Builds ist DEBUG, für Release-Builds INFO. Die Logstufe kann zur Laufzeit über Umgebungsvariablen (z.B. RUST\_LOG) angepasst werden.

### **3.4. Initiales Teststrategie-Framework**

Eine mehrschichtige Teststrategie stellt die Qualität und Korrektheit der UI-Schicht sicher.

* **Unit-Tests**:  
  * Fokus: Testen von isolierter Logik innerhalb von UI-Komponenten, die nicht direkt vom GTK-Rendering oder \-Eventloop abhängt (z.B. Hilfsfunktionen, Datenkonvertierungslogik, Zustandsmanagement-Helfer).  
  * Werkzeuge: Standard Rust \#\[test\], Mocking-Bibliotheken (z.B. mockall) für Abhängigkeiten zu unteren Schichten oder externen Diensten.  
* **Widget-Tests**:  
  * Fokus: Testen des Verhaltens und Zustands einzelner GTK-Widgets und benutzerdefinierter GObject-Komponenten.  
  * Werkzeuge:  
    * gtk::test Namespace: Bietet Funktionen zum Initialisieren von GTK in Testumgebungen.  
    * Programmatische Interaktion: Simulieren von Signalen (z.B. widget.emit\_by\_name::\<()\>("clicked", &)), Setzen und Abfragen von GObject-Eigenschaften.  
    * Inspektion: Überprüfung von Widget-Zuständen (z.B. label.text(), button.is\_sensitive()).  
    * GTK-Inspektionswerkzeuge und Accessibility-APIs (ATK) können programmatisch genutzt werden, um Widget-Zustände und \-Eigenschaften zu überprüfen.122 Die Evaluierung von Frameworks wie gtk4-rs-test-utils (falls existent und passend) oder ähnlichen Ansätzen ist Teil des Untersuchungsbedarfs.  
* **Accessibility-Tests**:  
  * Fokus: Sicherstellen, dass UI-Komponenten für assistive Technologien zugänglich sind.  
  * Werkzeuge: Überprüfung von ATK-Eigenschaften (Rolle, Name, Beschreibung, Zustand) der Widgets. Manuelle Tests mit Screenreadern (z.B. Orca) sind ebenfalls notwendig. gtk::Accessible.23  
* **Visuelle Regressionstests**: (Zur Evaluierung)  
  * Fokus: Erkennen von unbeabsichtigten visuellen Änderungen in der UI.  
  * Werkzeuge: Evaluierung von Werkzeugen für den visuellen Vergleich von UI-Zuständen (Screenshots). Dies ist oft aufwendig und wird initial möglicherweise zurückgestellt.  
* **Integrations-/End-to-End-Tests**: (Herausfordernd, für kritische Pfade)  
  * Fokus: Testen des Zusammenspiels mehrerer UI-Komponenten und deren Interaktion mit unteren Schichten.  
  * Werkzeuge: Simulation von Benutzerinteraktionen auf Wayland-Ebene (z.B. mit Tools wie ydotool oder spezialisierten Test-Frameworks, falls verfügbar und integrierbar). Überprüfung des Systemverhaltens. Dies ist sehr komplex und wird nur für kritische User Journeys in Betracht gezogen.

### **3.5. Richtlinien für Performance-Optimierung und Profiling**

Die Sicherstellung einer performanten UI ist ein Kernziel.

* **Profiling-Werkzeuge**:  
  * **Rust-spezifisch**: perf unter Linux, cargo flamegraph, tracing mit tracing-flame für CPU-Profiling. Speicher-Profiler wie heaptrack oder Valgrind (mit Massif) können zur Analyse des Speicherverbrauchs herangezogen werden.  
  * **GTK4-spezifisch**: Der GTK Inspector enthält einen Profiler, der Rendering-Zeiten und Widget-Updates visualisiert. GSK-spezifische Debug-Flags (GSK\_DEBUG) können Aufschluss über Rendering-Pfade geben.  
* **Optimierungsbereiche**:  
  * **Widget-Zeichnung**: Bei benutzerdefinierten Zeichnungen mit Cairo (GtkDrawingArea) darauf achten, nur die notwendigen Bereiche neu zu zeichnen (gtk\_widget\_queue\_draw\_area). Komplexität der Zeichenoperationen minimieren.  
  * **CSS-Anwendung**: CSS-Selektoren einfach halten. Komplexe Selektoren und Regeln können die Performance beeinträchtigen. Effiziente Aktualisierung von CSS bei Theme-Wechseln.10  
  * **Datenbindung**: Übermäßige Nutzung von GObject-Property-Bindings oder zu häufige Benachrichtigungen bei kleinen Änderungen können zu Performance-Engpässen führen. Änderungen ggf. bündeln.  
  * **Layout-Performance**: Vermeidung unnötig tiefer Widget-Hierarchien. Effiziente Nutzung von Layout-Managern wie GtkBox und GtkGrid.  
  * **Asynchrone Operationen**: Konsequente Nutzung von glib::MainContext::spawn\_local für alle potenziell blockierenden Operationen (Netzwerk, Datei-I/O, aufwändige Berechnungen in der Domänenschicht), um UI-Blockaden zu verhindern.7 Visuelles Feedback (Spinner, Fortschrittsbalken) für laufende Operationen bereitstellen.  
* **Allgemeine Rust-Optimierungen**: Zero-Cost-Abstraktionen nutzen, unnötige Allokationen vermeiden, effiziente Datenstrukturen wählen.24

Performance-Messungen und \-Optimierungen sollten ein integraler Bestandteil des Entwicklungsprozesses sein, nicht eine nachträgliche Maßnahme.

## **4\. Plan für nachfolgende UI-Layer-Module**

### **4.1. Priorisierung für nächste Module**

Nach der initialen Implementierung und Stabilisierung des PanelWidget und des AppMenuButton sowie der grundlegenden übergreifenden Frameworks (Theming, State Management, Error Handling, Logging) werden die UI-Module in folgender logischer Reihenfolge priorisiert:

1. **Weitere Kern-Panel-Module (ui::shell)**:  
   * WorkspaceIndicatorWidget: Essentiell für die Workspace-Navigation.  
   * ClockDateTimeWidget: Grundlegende Benutzerinformation.  
   * SystemTrayEquivalentWidget: Kritisch für die Integration von Drittanbieter-Anwendungen. Aufgrund seiner Komplexität (siehe oben) wird hierfür frühzeitig mit der Detailplanung und Prototyping begonnen.  
   * QuickSettingsButtonWidget und NotificationCenterButtonWidget: Wichtige Zugriffspunkte für Systemfunktionen.  
   * Weitere Indikatoren (NetworkIndicatorWidget, PowerIndicatorWidget, AudioIndicatorWidget).  
2. **Ausklappbare Panel-Inhalte (ui::shell)**:  
   * QuickSettingsPanelWidget: Wird vom QuickSettingsButtonWidget geöffnet.  
   * NotificationCenterPanelWidget: Wird vom NotificationCenterButtonWidget geöffnet und interagiert mit ui::notifications\_frontend.  
3. **Weitere Shell-Komponenten (ui::shell)**:  
   * SmartTabBarWidget  
   * WorkspaceSwitcherWidget  
   * QuickActionDockWidget  
4. **Systemeinstellungsanwendung (ui::control\_center)**:  
   * Dies ist eine größere, eigenständige Anwendung und wird parallel zu weniger kritischen Shell-Komponenten entwickelt, sobald die Kern-Shell-Interaktionen stabil sind.  
5. **Spezifische UI-Frontends und Widgets**:  
   * ui::notifications\_frontend (Popups)  
   * ui::widgets (Sidebar-Widgets)  
   * ui::window\_manager\_frontend  
   * ui::speed\_dial  
   * ui::command\_palette

Diese Priorisierung zielt darauf ab, schnell einen funktionalen Kern der Desktop-Shell zu etablieren und dann schrittweise weitere Funktionen und Anwendungen hinzuzufügen.

### **4.2. Identifizierte Abhängigkeiten und Parallelisierungsmöglichkeiten**

* **Abhängigkeiten**:  
  * Alle Panel-Module hängen von einem stabilen PanelWidget und dessen API ab.  
  * AppMenuButton und SystemTrayEquivalentWidget haben starke Abhängigkeiten von D-Bus-Interaktionen und Wayland-Protokollen (bzw. den Abstraktionsdiensten dafür).  
  * NetworkIndicatorWidget, PowerIndicatorWidget, AudioIndicatorWidget hängen von den entsprechenden D-Bus-Schnittstellen der system-Schicht ab.  
  * NotificationCenterPanelWidget hängt von domain::user\_centric\_services::NotificationService und ui::notifications\_frontend::NotificationPopupWidget ab.  
  * ui::control\_center Module hängen stark von domain::global\_settings\_and\_state\_management::GlobalSettingsService ab.  
* **Parallelisierung**:  
  * Sobald die API des PanelWidget definiert ist, können viele der darin enthaltenen Module (ClockDateTimeWidget, WorkspaceIndicatorWidget, einzelne Indikatoren) parallel entwickelt werden.  
  * Die Entwicklung des ui::control\_center kann weitgehend parallel zur Verfeinerung der ui::shell erfolgen, sobald die GlobalSettingsService-Schnittstelle stabil ist.  
  * Wiederverwendbare Komponenten in ui::components können frühzeitig parallel entwickelt und in anderen Modulen eingesetzt werden.  
  * Die Implementierung der D-Bus-Clients für verschiedene Systemdienste (NetworkManager, UPower, etc.) kann parallelisiert werden.

Eine enge Abstimmung zwischen den Teams, die an abhängigen Modulen arbeiten, ist entscheidend. Die Definition klarer Schnittstellen (GObject-Properties und \-Signale, Rust-Traits) für Module und Dienste erleichtert die parallele Entwicklung und spätere Integration.

#### **Referenzen**

1. gtk4-rs/README.md at main \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/gtk-rs/gtk4-rs/blob/main/README.md](https://github.com/gtk-rs/gtk4-rs/blob/main/README.md)  
2. GTK and Rust \- The GTK Project \- A free and open-source cross-platform widget toolkit, Zugriff am Mai 14, 2025, [https://www.gtk.org/docs/language-bindings/rust](https://www.gtk.org/docs/language-bindings/rust)  
3. gtk4 \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/gtk4](https://docs.rs/gtk4)  
4. Patterns \- The Rust Reference, Zugriff am Mai 14, 2025, [https://doc.rust-lang.org/stable/reference/patterns.html?highlight=Patterns](https://doc.rust-lang.org/stable/reference/patterns.html?highlight=Patterns)  
5. Rust: Project structure example step by step \- DEV Community, Zugriff am Mai 14, 2025, [https://dev.to/ghost/rust-project-structure-example-step-by-step-3ee](https://dev.to/ghost/rust-project-structure-example-step-by-step-3ee)  
6. How to structure files in rust projects? \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/rust/comments/6qbi91/how\_to\_structure\_files\_in\_rust\_projects/](https://www.reddit.com/r/rust/comments/6qbi91/how_to_structure_files_in_rust_projects/)  
7. MainContext in glib \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/glib/latest/glib/struct.MainContext.html](https://docs.rs/glib/latest/glib/struct.MainContext.html)  
8. MainContext in glib \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/struct.MainContext.html](https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/struct.MainContext.html)  
9. How to force custom GTK4 theme to follow preferred theme? : r/gnome \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/gnome/comments/1kasp2c/how\_to\_force\_custom\_gtk4\_theme\_to\_follow/](https://www.reddit.com/r/gnome/comments/1kasp2c/how_to_force_custom_gtk4_theme_to_follow/)  
10. jbenner-radham/rust-gtk4-css-styling: How to style your ... \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/jbenner-radham/rust-gtk4-css-styling](https://github.com/jbenner-radham/rust-gtk4-css-styling)  
11. Gtk.CssProvider, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.CssProvider.html](https://docs.gtk.org/gtk4/class.CssProvider.html)  
12. zbus \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/](https://docs.rs/zbus/latest/zbus/)  
13. dbus2/zbus: Rust D-Bus crate. \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/dbus2/zbus](https://github.com/dbus2/zbus)  
14. zbus \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/zbus](https://crates.io/crates/zbus)  
15. zbus \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/zbus/3.15.2](https://crates.io/crates/zbus/3.15.2)  
16. Writing a service interface \- zbus: D-Bus for Rust made easy, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/service.html](https://dbus2.github.io/zbus/service.html)  
17. "dbus\_proxy" Search \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/?search=dbus\_proxy](https://docs.rs/zbus/latest/zbus/?search=dbus_proxy)  
18. proxy in zbus \- Rust \- openrr.github.io, Zugriff am Mai 14, 2025, [https://openrr.github.io/openrr/zbus/attr.proxy.html](https://openrr.github.io/openrr/zbus/attr.proxy.html)  
19. Introduction \- zbus: D-Bus for Rust made easy, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/](https://dbus2.github.io/zbus/)  
20. Fighting with zbus implementation \- help \- The Rust Programming Language Forum, Zugriff am Mai 14, 2025, [https://users.rust-lang.org/t/fighting-with-zbus-implementation/106696](https://users.rust-lang.org/t/fighting-with-zbus-implementation/106696)  
21. Gio – 2.0 \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gio/index.html](https://docs.gtk.org/gio/index.html)  
22. Zugriff am Januar 1, 1970, [https://github.com/dbus2/zbus/tree/main/zbus/examples](https://github.com/dbus2/zbus/tree/main/zbus/examples)  
23. Gtk – 4.0: Accessibility, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/section-accessibility.html](https://docs.gtk.org/gtk4/section-accessibility.html)  
24. Ultimate Rust Performance Optimization Guide 2024: Basics to Advanced \- Rapid Innovation, Zugriff am Mai 14, 2025, [https://www.rapidinnovation.io/post/performance-optimization-techniques-in-rust](https://www.rapidinnovation.io/post/performance-optimization-techniques-in-rust)  
25. Ultimate Rust FPS Optimization Guide ✔️ \- YouTube, Zugriff am Mai 14, 2025, [https://m.youtube.com/watch?v=lQxeBhTgPEQ](https://m.youtube.com/watch?v=lQxeBhTgPEQ)  
26. wmww/gtk4-layer-shell: A library to create panels and other ... \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/wmww/gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell)  
27. gtk4-layer-shell \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/gtk4-layer-shell](https://crates.io/crates/gtk4-layer-shell)  
28. gtk4-layer-shell/examples/simple-example.c at main \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/wmww/gtk4-layer-shell/blob/main/examples/simple-example.c](https://github.com/wmww/gtk4-layer-shell/blob/main/examples/simple-example.c)  
29. LayerShell in gtk\_layer\_shell \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/gtk-layer-shell/latest/gtk\_layer\_shell/trait.LayerShell.html](https://docs.rs/gtk-layer-shell/latest/gtk_layer_shell/trait.LayerShell.html)  
30. drkrssll/chunks-rs: Simplifies the process of making GTK4 widgets for wayland compositors. \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/drkrssll/chunks-rs/](https://github.com/drkrssll/chunks-rs/)  
31. Zugriff am Januar 1, 1970, [https://github.com/pentamassiv/gtk4-layer-shell-gir/tree/main/gtk4-layer-shell/examples](https://github.com/pentamassiv/gtk4-layer-shell-gir/tree/main/gtk4-layer-shell/examples)  
32. Zugriff am Januar 1, 1970, [https://github.com/pentamassiv/gtk4-layer-shell-gir/tree/main/examples](https://github.com/pentamassiv/gtk4-layer-shell-gir/tree/main/examples)  
33. Zugriff am Januar 1, 1970, [https://docs.rs/gtk4-layer-shell/latest/gtk\_layer\_shell/trait.LayerShell.html](https://docs.rs/gtk4-layer-shell/latest/gtk_layer_shell/trait.LayerShell.html)  
34. DrawingArea in gtk4 \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/0.8/docs/gtk4/struct.DrawingArea.html](https://gtk-rs.org/gtk4-rs/stable/0.8/docs/gtk4/struct.DrawingArea.html)  
35. gtk4/NEWS.pre-4.0 at layer-shell\_impish \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/pop-os/gtk4/blob/layer-shell\_impish/NEWS.pre-4.0](https://github.com/pop-os/gtk4/blob/layer-shell_impish/NEWS.pre-4.0)  
36. Gtk – 4.0: GTK CSS Properties, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/css-properties.html](https://docs.gtk.org/gtk4/css-properties.html)  
37. DrawingArea in gtk4 \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/0.5/docs/gtk4/struct.DrawingArea.html](https://gtk-rs.org/gtk4-rs/stable/0.5/docs/gtk4/struct.DrawingArea.html)  
38. DrawingArea in gtk4 \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.DrawingArea.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.DrawingArea.html)  
39. How do I create a glow effect like this? I tried gaussian blur, but it doesn't really look like this : r/krita \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/krita/comments/1j7innz/how\_do\_i\_create\_a\_glow\_effect\_like\_this\_i\_tried/](https://www.reddit.com/r/krita/comments/1j7innz/how_do_i_create_a_glow_effect_like_this_i_tried/)  
40. Creating a highlight effect with the cairo library \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/9955964/creating-a-highlight-effect-with-the-cairo-library](https://stackoverflow.com/questions/9955964/creating-a-highlight-effect-with-the-cairo-library)  
41. Cooking with Cairo \- Cairo graphics library, Zugriff am Mai 14, 2025, [https://www.cairographics.org/cookbook/](https://www.cairographics.org/cookbook/)  
42. GtkBox, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Box.html](https://docs.gtk.org/gtk4/class.Box.html)  
43. IsA for subclasses in GTK Rust \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/75623312/isa-for-subclasses-in-gtk-rust](https://stackoverflow.com/questions/75623312/isa-for-subclasses-in-gtk-rust)  
44. gtk4-rs/book/listings/actions/6/resources/window.ui at main \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/gtk-rs/gtk4-rs/blob/main/book/listings/actions/6/resources/window.ui](https://github.com/gtk-rs/gtk4-rs/blob/main/book/listings/actions/6/resources/window.ui)  
45. How to make gtk4-rs use native windows decorations? : r/rust \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/rust/comments/1jdm50v/how\_to\_make\_gtk4rs\_use\_native\_windows\_decorations/](https://www.reddit.com/r/rust/comments/1jdm50v/how_to_make_gtk4rs_use_native_windows_decorations/)  
46. Gtk.ApplicationWindow, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.ApplicationWindow.html](https://docs.gtk.org/gtk4/class.ApplicationWindow.html)  
47. Properties \- GUI development with Rust and GTK 4, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/g\_object\_properties.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/g_object_properties.html)  
48. Zugriff am Januar 1, 1970, [https://gtk-rs.org/gtk4-rs/stable/latest/book/gobject\_properties.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/gobject_properties.html)  
49. GObject – 2.0 \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gobject/index.html](https://docs.gtk.org/gobject/index.html)  
50. Zugriff am Januar 1, 1970, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/glib/index.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/glib/index.html)  
51. Memory Management \- GUI development with Rust and GTK 4, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/g\_object\_memory\_management.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/g_object_memory_management.html)  
52. glib \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/](https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/)  
53. Zugriff am Januar 1, 1970, [https://gtk-rs.org/gtk4-rs/stable/latest/book/gobject\_memory\_management.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/gobject_memory_management.html)  
54. GLib – 2.0 \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/glib/index.html](https://docs.gtk.org/glib/index.html)  
55. Counter App with GTK4 CompositeTemplate and Rust \- DEV Community, Zugriff am Mai 14, 2025, [https://dev.to/kashifsoofi/counter-app-with-gtk4-compositetemplate-and-rust-h0p](https://dev.to/kashifsoofi/counter-app-with-gtk4-compositetemplate-and-rust-h0p)  
56. Composite Templates \- GUI development with Rust and GTK 4, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/composite\_templates.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/composite_templates.html)  
57. Writing a Custom GTK widget with template UI files \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/77768792/writing-a-custom-gtk-widget-with-template-ui-files](https://stackoverflow.com/questions/77768792/writing-a-custom-gtk-widget-with-template-ui-files)  
58. Counter App with GTK4 CompositeTemplate and Rust \- Kashif Soofi, Zugriff am Mai 14, 2025, [https://kashifsoofi.github.io/gtk4/rust/gtk4-rust-counter-app-with-template/](https://kashifsoofi.github.io/gtk4/rust/gtk4-rust-counter-app-with-template/)  
59. Building a Simple To-Do App \- GUI development with Rust and GTK 4, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/todo\_1.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/todo_1.html)  
60. Introduction \- GUI development with Rust and GTK 4 \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/introduction.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/introduction.html)  
61. Gtk.Builder \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Builder.html](https://docs.gtk.org/gtk4/class.Builder.html)  
62. Box in gtk4 \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Box.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Box.html)  
63. Gtk.CenterBox, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.CenterBox.html](https://docs.gtk.org/gtk4/class.CenterBox.html)  
64. Zugriff am Januar 1, 1970, [https://docs.gtk.org/gtk4/section-drawing-model.html](https://docs.gtk.org/gtk4/section-drawing-model.html)  
65. Zugriff am Januar 1, 1970, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/cairo/index.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/cairo/index.html)  
66. Pango – 1.0 \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/Pango/index.html](https://docs.gtk.org/Pango/index.html)  
67. cairo-rs 0.20.7 \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/cairo-rs/latest/cairo/struct.ImageSurface.html](https://docs.rs/cairo-rs/latest/cairo/struct.ImageSurface.html)  
68. Gtk.Widget, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Widget.html](https://docs.gtk.org/gtk4/class.Widget.html)  
69. Gtk.Settings, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Settings.html](https://docs.gtk.org/gtk4/class.Settings.html)  
70. geting the coordinates of the user's mouse pointer in GTK4 in C \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/79016771/geting-the-coordinates-of-the-users-mouse-pointer-in-gtk4-in-c](https://stackoverflow.com/questions/79016771/geting-the-coordinates-of-the-users-mouse-pointer-in-gtk4-in-c)  
71. Gtk.EventControllerMotion, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.EventControllerMotion.html](https://docs.gtk.org/gtk4/class.EventControllerMotion.html)  
72. thiserror \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/thiserror/latest/thiserror/](https://docs.rs/thiserror/latest/thiserror/)  
73. tracing \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/tracing/latest/tracing/](https://docs.rs/tracing/latest/tracing/)  
74. Gdk.Display, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gdk4/class.Display.html](https://docs.gtk.org/gdk4/class.Display.html)  
75. Gdk.Monitor, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gdk4/class.Monitor.html](https://docs.gtk.org/gdk4/class.Monitor.html)  
76. Gtk.Widget:tooltip-text, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/property.Widget.tooltip-text.html](https://docs.gtk.org/gtk4/property.Widget.tooltip-text.html)  
77. Gtk.Button, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Button.html](https://docs.gtk.org/gtk4/class.Button.html)  
78. Gtk.MenuButton, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.MenuButton.html](https://docs.gtk.org/gtk4/class.MenuButton.html)  
79. Gtk.PopoverMenu, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.PopoverMenu.html](https://docs.gtk.org/gtk4/class.PopoverMenu.html)  
80. PopoverMenu in gtk4 \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.PopoverMenu.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.PopoverMenu.html)  
81. API changes in GTK4: removal of GtkMenu : r/GTK \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/GTK/comments/xdfgjr/api\_changes\_in\_gtk4\_removal\_of\_gtkmenu/](https://www.reddit.com/r/GTK/comments/xdfgjr/api_changes_in_gtk4_removal_of_gtkmenu/)  
82. Zugriff am Januar 1, 1970, [https://docs.gtk.org/gtk4/method.PopoverMenu.new\_from\_model.html](https://docs.gtk.org/gtk4/method.PopoverMenu.new_from_model.html)  
83. Gtk.Popover, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Popover.html](https://docs.gtk.org/gtk4/class.Popover.html)  
84. Gtk.Image \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Image.html](https://docs.gtk.org/gtk4/class.Image.html)  
85. I want to create a window switcher for Linux. Is a Wayland client the correct approach?, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/linux/comments/1juzcto/i\_want\_to\_create\_a\_window\_switcher\_for\_linux\_is\_a/](https://www.reddit.com/r/linux/comments/1juzcto/i_want_to_create_a_window_switcher_for_linux_is_a/)  
86. wlr foreign toplevel management protocol \- Wayland Explorer, Zugriff am Mai 14, 2025, [https://wayland.app/protocols/wlr-foreign-toplevel-management-unstable-v1](https://wayland.app/protocols/wlr-foreign-toplevel-management-unstable-v1)  
87. Zugriff am Januar 1, 1970, [https://wayland.freedesktop.org/protocols/wlr-foreign-toplevel-management-unstable-v1.html](https://wayland.freedesktop.org/protocols/wlr-foreign-toplevel-management-unstable-v1.html)  
88. Zugriff am Januar 1, 1970, [https://docs.rs/smithay-client-toolkit/latest/smithay\_client\_toolkit/shell/foreign\_toplevel/index.html](https://docs.rs/smithay-client-toolkit/latest/smithay_client_toolkit/shell/foreign_toplevel/index.html)  
89. Zugriff am Januar 1, 1970, [https://docs.rs/wayland-protocols/latest/wayland\_protocols/wlr/unstable/foreign\_toplevel\_management/v1/client/index.html](https://docs.rs/wayland-protocols/latest/wayland_protocols/wlr/unstable/foreign_toplevel_management/v1/client/index.html)  
90. wayland\_protocols::ext::foreign\_toplevel\_list::v1::client::ext\_foreign\_toplevel\_list\_v1 \- Rust, Zugriff am Mai 14, 2025, [https://pop-os.github.io/libcosmic/wayland\_protocols/ext/foreign\_toplevel\_list/v1/client/ext\_foreign\_toplevel\_list\_v1/index.html](https://pop-os.github.io/libcosmic/wayland_protocols/ext/foreign_toplevel_list/v1/client/ext_foreign_toplevel_list_v1/index.html)  
91. Projects/GLib/GApplication/DBusAPI – GNOME Wiki Archive, Zugriff am Mai 14, 2025, [https://wiki.gnome.org/Projects/GLib/GApplication/DBusAPI](https://wiki.gnome.org/Projects/GLib/GApplication/DBusAPI)  
92. gtk/gtk/gtkapplication-dbus.c at main · GNOME/gtk \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/GNOME/gtk/blob/master/gtk/gtkapplication-dbus.c](https://github.com/GNOME/gtk/blob/master/gtk/gtkapplication-dbus.c)  
93. D-Bus \- GNOME JavaScript, Zugriff am Mai 14, 2025, [https://gjs.guide/guides/gio/dbus.html](https://gjs.guide/guides/gio/dbus.html)  
94. Menus \- GNOME Developer Documentation, Zugriff am Mai 14, 2025, [https://developer.gnome.org/documentation/tutorials/menus.html](https://developer.gnome.org/documentation/tutorials/menus.html)  
95. Application menu / Actions \- GNOME Wiki, Zugriff am Mai 14, 2025, [https://wiki.gnome.org/ThreePointThree(2f)Features(2f)ApplicationMenu.html](https://wiki.gnome.org/ThreePointThree\(2f\)Features\(2f\)ApplicationMenu.html)  
96. Zugriff am Januar 1, 1970, [https://docs.gtk.org/gio/struct.DBusMenuModel.html](https://docs.gtk.org/gio/struct.DBusMenuModel.html)  
97. rust-zbus package : Ubuntu \- Launchpad, Zugriff am Mai 14, 2025, [https://launchpad.net/ubuntu/+source/rust-zbus](https://launchpad.net/ubuntu/+source/rust-zbus)  
98. Global Menu — helloSystem documentation, Zugriff am Mai 14, 2025, [https://hellosystem.github.io/docs/developer/menu.html](https://hellosystem.github.io/docs/developer/menu.html)  
99. Zugriff am Januar 1, 1970, [https://docs.rs/zbus/latest/zbus/struct.Proxy.html](https://docs.rs/zbus/latest/zbus/struct.Proxy.html)  
100. smithay\_client\_toolkit::foreign\_toplevel\_list \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/client-toolkit/smithay\_client\_toolkit/foreign\_toplevel\_list/index.html](https://smithay.github.io/client-toolkit/smithay_client_toolkit/foreign_toplevel_list/index.html)  
101. Zugriff am Januar 1, 1970, [https://github.com/smithay/smithay-client-toolkit/tree/master/examples](https://github.com/smithay/smithay-client-toolkit/tree/master/examples)  
102. StatusNotifierItem \- Freedesktop.org, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/)  
103. StatusNotifierItem \- Freedesktop.org, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierItem/](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierItem/)  
104. Where are status icons in Gnome 40/GTK 4? \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/gnome/comments/u9tq1k/where\_are\_status\_icons\_in\_gnome\_40gtk\_4/](https://www.reddit.com/r/gnome/comments/u9tq1k/where_are_status_icons_in_gnome_40gtk_4/)  
105. Introduce support for GTK 4 · Issue \#22 · AyatanaIndicators/libayatana-appindicator-glib, Zugriff am Mai 14, 2025, [https://github.com/AyatanaIndicators/libayatana-appindicator-glib/issues/22](https://github.com/AyatanaIndicators/libayatana-appindicator-glib/issues/22)  
106. StatusNotifierHost \- Freedesktop.org, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierHost/](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierHost/)  
107. StatusNotifierWatcher \- Freedesktop.org, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierWatcher/](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierWatcher/)  
108. Zugriff am Januar 1, 1970, [https://www.freedesktop.org/wiki/Specifications/StatusNotifierHost/](https://www.freedesktop.org/wiki/Specifications/StatusNotifierHost/)  
109. Writing a client proxy \- zbus: D-Bus for Rust made easy \- GitHub Pages, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/client.html](https://dbus2.github.io/zbus/client.html)  
110. NameOwnerChanged in zbus::fdo \- Rust \- openrr.github.io, Zugriff am Mai 14, 2025, [https://openrr.github.io/openrr/zbus/fdo/struct.NameOwnerChanged.html](https://openrr.github.io/openrr/zbus/fdo/struct.NameOwnerChanged.html)  
111. zbus::fdo \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/fdo/index.html](https://docs.rs/zbus/latest/zbus/fdo/index.html)  
112. Zbus: catching signal with a destination \- help \- The Rust Programming Language Forum, Zugriff am Mai 14, 2025, [https://users.rust-lang.org/t/zbus-catching-signal-with-a-destination/125709](https://users.rust-lang.org/t/zbus-catching-signal-with-a-destination/125709)  
113. Zugriff am Januar 1, 1970, [https://docs.rs/zbus/latest/zbus/struct.Connection.html](https://docs.rs/zbus/latest/zbus/struct.Connection.html)  
114. Zugriff am Januar 1, 1970, [https://docs.rs/zbus/latest/zbus/fdo/struct.NameOwnerChangedArgs.html](https://docs.rs/zbus/latest/zbus/fdo/struct.NameOwnerChangedArgs.html)  
115. Signals \- GUI development with Rust and GTK 4, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/g\_object\_signals.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/g_object_signals.html)  
116. icon-theme-spec \- Freedesktop.org, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/wiki/Specifications/icon-theme-spec/](https://www.freedesktop.org/wiki/Specifications/icon-theme-spec/)  
117. Zugriff am Januar 1, 1970, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk\_pixbuf/index.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk_pixbuf/index.html)  
118. Zugriff am Januar 1, 1970, [https://docs.rs/gdk-pixbuf/latest/gdk\_pixbuf/struct.PixbufLoader.html](https://docs.rs/gdk-pixbuf/latest/gdk_pixbuf/struct.PixbufLoader.html)  
119. dbusmenu\_glib \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/dbusmenu-glib](https://docs.rs/dbusmenu-glib)  
120. System Tray Icons in GTK4? \- Platform \- GNOME Discourse, Zugriff am Mai 14, 2025, [https://discourse.gnome.org/t/system-tray-icons-in-gtk4/22615](https://discourse.gnome.org/t/system-tray-icons-in-gtk4/22615)  
121. Correct way of state management for gtk-rs? : r/rust \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/rust/comments/1czodm3/correct\_way\_of\_state\_management\_for\_gtkrs/](https://www.reddit.com/r/rust/comments/1czodm3/correct_way_of_state_management_for_gtkrs/)  
122. Gtk4 \- how to deploy complex event handlers \- help \- Rust Users Forum, Zugriff am Mai 14, 2025, [https://users.rust-lang.org/t/gtk4-how-to-deploy-complex-event-handlers/88170](https://users.rust-lang.org/t/gtk4-how-to-deploy-complex-event-handlers/88170)  
123. once\_cell \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/once\_cell/latest/once\_cell/](https://docs.rs/once_cell/latest/once_cell/)
