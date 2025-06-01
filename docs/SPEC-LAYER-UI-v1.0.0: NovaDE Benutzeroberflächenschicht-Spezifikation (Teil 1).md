# SPEC-LAYER-UI-v1.0.0: NovaDE Benutzeroberflächenschicht-Spezifikation (Teil 1)

```
SPEZIFIKATION: SPEC-LAYER-UI-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0, SPEC-LAYER-CORE-v1.0.0, SPEC-LAYER-DOMAIN-v1.0.0, SPEC-LAYER-SYSTEM-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Diese Spezifikation definiert die Benutzeroberflächenschicht (UI Layer) des NovaDE-Projekts. Die UI-Schicht ist für die grafische Darstellung und direkte Benutzerinteraktion verantwortlich und baut auf den tieferen Schichten (Kern, Domäne, System) auf. Der Geltungsbereich umfasst alle Module, Komponenten und Schnittstellen der UI-Schicht sowie deren Interaktionen mit tieferen Schichten.

## 2. Definitionen

### 2.1 Allgemeine Begriffe

- **UI-Schicht**: Oberste Schicht der NovaDE-Architektur, die für die grafische Darstellung und Benutzerinteraktion verantwortlich ist
- **Modul**: Funktionale Einheit innerhalb der UI-Schicht
- **Komponente**: Funktionale Einheit innerhalb eines Moduls
- **Widget**: Wiederverwendbare UI-Komponente
- **View**: Darstellungskomponente für Daten
- **Controller**: Komponente zur Steuerung der Benutzerinteraktion

### 2.2 UI-Schicht-spezifische Begriffe

- **Shell**: Hauptkomponenten der Desktop-Umgebung (Panel, Dock, etc.)
- **Control Center**: Anwendung zur Konfiguration der Desktop-Umgebung
- **Panel**: Leiste am Bildschirmrand mit Statusinformationen und Schnellzugriff
- **Dock**: Bereich für Anwendungsstarter und laufende Anwendungen
- **Workspace Switcher**: Komponente zum Wechseln zwischen virtuellen Desktops
- **Notification Center**: Komponente zur Anzeige und Verwaltung von Benachrichtigungen

## 3. Anforderungen

### 3.1 Funktionale Anforderungen

1. Die UI-Schicht MUSS eine Shell mit Panel, Dock und Workspace Switcher bereitstellen.
2. Die UI-Schicht MUSS ein Control Center zur Konfiguration der Desktop-Umgebung bereitstellen.
3. Die UI-Schicht MUSS ein Notification Center zur Anzeige und Verwaltung von Benachrichtigungen bereitstellen.
4. Die UI-Schicht MUSS wiederverwendbare Widgets für die Verwendung in verschiedenen Komponenten bereitstellen.
5. Die UI-Schicht MUSS eine konsistente Anwendung des Theming-Systems über alle Komponenten hinweg gewährleisten.
6. Die UI-Schicht MUSS Barrierefreiheitsfunktionen implementieren.
7. Die UI-Schicht MUSS responsive Layouts für verschiedene Bildschirmgrößen und -auflösungen unterstützen.

### 3.2 Nicht-funktionale Anforderungen

1. Die UI-Schicht MUSS eine hohe Performance und geringe Latenz bieten.
2. Die UI-Schicht MUSS eine konsistente und intuitive Benutzeroberfläche bieten.
3. Die UI-Schicht MUSS für verschiedene Eingabemethoden (Tastatur, Maus, Touch) optimiert sein.
4. Die UI-Schicht MUSS internationalisiert und lokalisierbar sein.
5. Die UI-Schicht MUSS thread-safe implementiert sein.
6. Die UI-Schicht MUSS umfassend dokumentiert sein.
7. Die UI-Schicht MUSS umfassend getestet sein.

## 4. Architektur

### 4.1 Modulstruktur

Die UI-Schicht besteht aus den folgenden Modulen:

1. **Shell Components** (`shell/`): Hauptkomponenten der Benutzeroberfläche
2. **Control Center** (`control_center/`): Einstellungsanwendung
3. **Widgets** (`widgets/`): Wiederverwendbare UI-Komponenten
4. **Window Manager Frontend** (`window_manager_frontend/`): UI-Aspekte der Fensterverwaltung
5. **Notifications Frontend** (`notifications_frontend/`): Benachrichtigungs-UI
6. **Theming GTK** (`theming_gtk/`): GTK-Theming
7. **Portals Client** (`portals/`): Client-seitige Portal-Interaktion

### 4.2 Abhängigkeiten

Die UI-Schicht hat folgende Abhängigkeiten:

1. **Kernschicht**: Für grundlegende Typen, Fehlerbehandlung, Logging und Konfiguration
2. **Domänenschicht**: Für Geschäftslogik und Richtlinien
3. **Systemschicht**: Für Interaktion mit dem Betriebssystem und der Hardware
4. **Externe Abhängigkeiten**:
   - `gtk4`: Für die Benutzeroberfläche
   - `libadwaita`: Für moderne GNOME-Widgets
   - `glib`: Für grundlegende Funktionalität
   - `cairo`: Für benutzerdefiniertes Zeichnen
   - `pango`: Für Textlayout und -rendering

## 5. Schnittstellen

### 5.1 Shell Components Module

```
SCHNITTSTELLE: ui::shell::ShellManager
BESCHREIBUNG: Verwaltet die Shell-Komponenten
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue ShellManager-Instanz
    PARAMETER:
      - NAME: config
        TYP: ShellConfig
        BESCHREIBUNG: Konfiguration für die Shell
        EINSCHRÄNKUNGEN: Muss eine gültige ShellConfig sein
    RÜCKGABETYP: Result<ShellManager, ShellError>
    FEHLER:
      - TYP: ShellError
        BEDINGUNG: Wenn der ShellManager nicht erstellt werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue ShellManager-Instanz wird erstellt oder ein Fehler wird zurückgegeben
  
  - NAME: initialize
    BESCHREIBUNG: Initialisiert die Shell-Komponenten
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), ShellError>
    FEHLER:
      - TYP: ShellError
        BEDINGUNG: Wenn die Shell-Komponenten nicht initialisiert werden können
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Shell-Komponenten werden initialisiert oder ein Fehler wird zurückgegeben
  
  - NAME: get_panel
    BESCHREIBUNG: Gibt die Panel-Komponente zurück
    PARAMETER: Keine
    RÜCKGABETYP: Option<&Panel>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Panel-Komponente wird zurückgegeben, wenn sie existiert, sonst None
  
  - NAME: get_dock
    BESCHREIBUNG: Gibt die Dock-Komponente zurück
    PARAMETER: Keine
    RÜCKGABETYP: Option<&Dock>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Dock-Komponente wird zurückgegeben, wenn sie existiert, sonst None
  
  - NAME: get_workspace_switcher
    BESCHREIBUNG: Gibt die Workspace-Switcher-Komponente zurück
    PARAMETER: Keine
    RÜCKGABETYP: Option<&WorkspaceSwitcher>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Workspace-Switcher-Komponente wird zurückgegeben, wenn sie existiert, sonst None
  
  - NAME: get_notification_center
    BESCHREIBUNG: Gibt die Notification-Center-Komponente zurück
    PARAMETER: Keine
    RÜCKGABETYP: Option<&NotificationCenter>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Notification-Center-Komponente wird zurückgegeben, wenn sie existiert, sonst None
```

### 5.2 Control Center Module

```
SCHNITTSTELLE: ui::control_center::ControlCenterApplication
BESCHREIBUNG: Hauptanwendung für das Control Center
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue ControlCenterApplication-Instanz
    PARAMETER:
      - NAME: config
        TYP: ControlCenterConfig
        BESCHREIBUNG: Konfiguration für das Control Center
        EINSCHRÄNKUNGEN: Muss eine gültige ControlCenterConfig sein
    RÜCKGABETYP: Result<ControlCenterApplication, ControlCenterError>
    FEHLER:
      - TYP: ControlCenterError
        BEDINGUNG: Wenn die ControlCenterApplication nicht erstellt werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue ControlCenterApplication-Instanz wird erstellt oder ein Fehler wird zurückgegeben
  
  - NAME: run
    BESCHREIBUNG: Startet die Control-Center-Anwendung
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), ControlCenterError>
    FEHLER:
      - TYP: ControlCenterError
        BEDINGUNG: Wenn die Anwendung nicht gestartet werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird gestartet oder ein Fehler wird zurückgegeben
  
  - NAME: get_active_panel
    BESCHREIBUNG: Gibt das aktive Panel zurück
    PARAMETER: Keine
    RÜCKGABETYP: Option<&SettingsPanel>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das aktive Panel wird zurückgegeben, wenn es existiert, sonst None
  
  - NAME: activate_panel
    BESCHREIBUNG: Aktiviert ein bestimmtes Panel
    PARAMETER:
      - NAME: panel_id
        TYP: String
        BESCHREIBUNG: ID des zu aktivierenden Panels
        EINSCHRÄNKUNGEN: Muss eine gültige Panel-ID sein
    RÜCKGABETYP: Result<(), ControlCenterError>
    FEHLER:
      - TYP: ControlCenterError
        BEDINGUNG: Wenn das Panel nicht aktiviert werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN:
      - Ein Panel mit der angegebenen ID muss existieren
    NACHBEDINGUNGEN:
      - Das Panel wird aktiviert oder ein Fehler wird zurückgegeben
```

### 5.3 Widgets Module

```
SCHNITTSTELLE: ui::widgets::WidgetFactory
BESCHREIBUNG: Factory für wiederverwendbare Widgets
VERSION: 1.0.0
OPERATIONEN:
  - NAME: create_button
    BESCHREIBUNG: Erstellt einen Button
    PARAMETER:
      - NAME: label
        TYP: String
        BESCHREIBUNG: Text des Buttons
        EINSCHRÄNKUNGEN: Keine
      - NAME: icon_name
        TYP: Option<String>
        BESCHREIBUNG: Optionaler Icon-Name
        EINSCHRÄNKUNGEN: Muss ein gültiger Icon-Name sein, wenn angegeben
      - NAME: style
        TYP: ButtonStyle
        BESCHREIBUNG: Stil des Buttons
        EINSCHRÄNKUNGEN: Muss ein gültiger ButtonStyle sein
    RÜCKGABETYP: Result<Button, WidgetError>
    FEHLER:
      - TYP: WidgetError
        BEDINGUNG: Wenn der Button nicht erstellt werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein neuer Button wird erstellt oder ein Fehler wird zurückgegeben
  
  - NAME: create_toggle
    BESCHREIBUNG: Erstellt einen Toggle-Switch
    PARAMETER:
      - NAME: label
        TYP: String
        BESCHREIBUNG: Text des Toggle-Switch
        EINSCHRÄNKUNGEN: Keine
      - NAME: initial_state
        TYP: bool
        BESCHREIBUNG: Anfangszustand des Toggle-Switch
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<Toggle, WidgetError>
    FEHLER:
      - TYP: WidgetError
        BEDINGUNG: Wenn der Toggle-Switch nicht erstellt werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein neuer Toggle-Switch wird erstellt oder ein Fehler wird zurückgegeben
  
  - NAME: create_slider
    BESCHREIBUNG: Erstellt einen Slider
    PARAMETER:
      - NAME: label
        TYP: String
        BESCHREIBUNG: Text des Sliders
        EINSCHRÄNKUNGEN: Keine
      - NAME: min_value
        TYP: f64
        BESCHREIBUNG: Minimaler Wert des Sliders
        EINSCHRÄNKUNGEN: Keine
      - NAME: max_value
        TYP: f64
        BESCHREIBUNG: Maximaler Wert des Sliders
        EINSCHRÄNKUNGEN: Muss größer als min_value sein
      - NAME: initial_value
        TYP: f64
        BESCHREIBUNG: Anfangswert des Sliders
        EINSCHRÄNKUNGEN: Muss zwischen min_value und max_value liegen
    RÜCKGABETYP: Result<Slider, WidgetError>
    FEHLER:
      - TYP: WidgetError
        BEDINGUNG: Wenn der Slider nicht erstellt werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein neuer Slider wird erstellt oder ein Fehler wird zurückgegeben
```

## 6. Datenmodell (Teil 1)

### 6.1 Shell-Typen

```
ENTITÄT: ShellConfig
BESCHREIBUNG: Konfiguration für die Shell
ATTRIBUTE:
  - NAME: panel_position
    TYP: PanelPosition
    BESCHREIBUNG: Position des Panels
    WERTEBEREICH: {Top, Bottom, Left, Right}
    STANDARDWERT: PanelPosition::Top
  - NAME: dock_position
    TYP: DockPosition
    BESCHREIBUNG: Position des Docks
    WERTEBEREICH: {Bottom, Left, Right}
    STANDARDWERT: DockPosition::Bottom
  - NAME: panel_size
    TYP: u32
    BESCHREIBUNG: Größe des Panels in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 32
  - NAME: dock_size
    TYP: u32
    BESCHREIBUNG: Größe des Docks in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 64
  - NAME: panel_auto_hide
    TYP: bool
    BESCHREIBUNG: Ob das Panel automatisch ausgeblendet werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: dock_auto_hide
    TYP: bool
    BESCHREIBUNG: Ob das Dock automatisch ausgeblendet werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: workspace_switcher_enabled
    TYP: bool
    BESCHREIBUNG: Ob der Workspace Switcher aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: notification_center_enabled
    TYP: bool
    BESCHREIBUNG: Ob das Notification Center aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
INVARIANTEN:
  - panel_size muss größer als 0 sein
  - dock_size muss größer als 0 sein
```

```
ENTITÄT: Panel
BESCHREIBUNG: Panel-Komponente der Shell
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Panels
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "main_panel"
  - NAME: position
    TYP: PanelPosition
    BESCHREIBUNG: Position des Panels
    WERTEBEREICH: {Top, Bottom, Left, Right}
    STANDARDWERT: PanelPosition::Top
  - NAME: size
    TYP: u32
    BESCHREIBUNG: Größe des Panels in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 32
  - NAME: auto_hide
    TYP: bool
    BESCHREIBUNG: Ob das Panel automatisch ausgeblendet werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: widgets
    TYP: Vec<PanelWidget>
    BESCHREIBUNG: Widgets im Panel
    WERTEBEREICH: Gültige PanelWidget-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - id darf nicht leer sein
  - size muss größer als 0 sein
```

```
ENTITÄT: Dock
BESCHREIBUNG: Dock-Komponente der Shell
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Docks
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "main_dock"
  - NAME: position
    TYP: DockPosition
    BESCHREIBUNG: Position des Docks
    WERTEBEREICH: {Bottom, Left, Right}
    STANDARDWERT: DockPosition::Bottom
  - NAME: size
    TYP: u32
    BESCHREIBUNG: Größe des Docks in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 64
  - NAME: auto_hide
    TYP: bool
    BESCHREIBUNG: Ob das Dock automatisch ausgeblendet werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: launchers
    TYP: Vec<DockLauncher>
    BESCHREIBUNG: Launcher im Dock
    WERTEBEREICH: Gültige DockLauncher-Werte
    STANDARDWERT: Leerer Vec
  - NAME: running_apps
    TYP: Vec<DockAppEntry>
    BESCHREIBUNG: Laufende Anwendungen im Dock
    WERTEBEREICH: Gültige DockAppEntry-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - id darf nicht leer sein
  - size muss größer als 0 sein
```

```
ENTITÄT: WorkspaceSwitcher
BESCHREIBUNG: Workspace-Switcher-Komponente der Shell
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Workspace Switchers
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "main_workspace_switcher"
  - NAME: display_mode
    TYP: WorkspaceSwitcherDisplayMode
    BESCHREIBUNG: Anzeigemodus des Workspace Switchers
    WERTEBEREICH: {Icons, Thumbnails, Names, IconsAndNames}
    STANDARDWERT: WorkspaceSwitcherDisplayMode::Icons
  - NAME: orientation
    TYP: Orientation
    BESCHREIBUNG: Ausrichtung des Workspace Switchers
    WERTEBEREICH: {Horizontal, Vertical}
    STANDARDWERT: Orientation::Horizontal
  - NAME: workspaces
    TYP: Vec<WorkspaceSwitcherEntry>
    BESCHREIBUNG: Workspace-Einträge im Workspace Switcher
    WERTEBEREICH: Gültige WorkspaceSwitcherEntry-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - id darf nicht leer sein
```

```
ENTITÄT: NotificationCenter
BESCHREIBUNG: Notification-Center-Komponente der Shell
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Notification Centers
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "main_notification_center"
  - NAME: max_visible_notifications
    TYP: u32
    BESCHREIBUNG: Maximale Anzahl sichtbarer Benachrichtigungen
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 5
  - NAME: group_by_app
    TYP: bool
    BESCHREIBUNG: Ob Benachrichtigungen nach Anwendung gruppiert werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: show_actions
    TYP: bool
    BESCHREIBUNG: Ob Aktionen angezeigt werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: notifications
    TYP: Vec<NotificationEntry>
    BESCHREIBUNG: Benachrichtigungen im Notification Center
    WERTEBEREICH: Gültige NotificationEntry-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - id darf nicht leer sein
  - max_visible_notifications muss größer als 0 sein
```

### 6.2 Control-Center-Typen

```
ENTITÄT: ControlCenterConfig
BESCHREIBUNG: Konfiguration für das Control Center
ATTRIBUTE:
  - NAME: window_width
    TYP: u32
    BESCHREIBUNG: Breite des Control-Center-Fensters
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 800
  - NAME: window_height
    TYP: u32
    BESCHREIBUNG: Höhe des Control-Center-Fensters
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 600
  - NAME: sidebar_width
    TYP: u32
    BESCHREIBUNG: Breite der Seitenleiste
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 200
  - NAME: default_panel
    TYP: String
    BESCHREIBUNG: ID des Standardpanels
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "appearance"
  - NAME: search_enabled
    TYP: bool
    BESCHREIBUNG: Ob die Suche aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
INVARIANTEN:
  - window_width muss größer als 0 sein
  - window_height muss größer als 0 sein
  - sidebar_width muss größer als 0 sein
  - default_panel darf nicht leer sein
```

```
ENTITÄT: SettingsPanel
BESCHREIBUNG: Panel im Control Center
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Panels
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: title
    TYP: String
    BESCHREIBUNG: Titel des Panels
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: icon_name
    TYP: String
    BESCHREIBUNG: Name des Icons für das Panel
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: sections
    TYP: Vec<SettingsSection>
    BESCHREIBUNG: Abschnitte im Panel
    WERTEBEREICH: Gültige SettingsSection-Werte
    STANDARDWERT: Leerer Vec
  - NAME: order
    TYP: u32
    BESCHREIBUNG: Reihenfolge des Panels
    WERTEBEREICH: Ganzzahlen
    STANDARDWERT: 0
INVARIANTEN:
  - id darf nicht leer sein
  - title darf nicht leer sein
  - icon_name darf nicht leer sein
```

```
ENTITÄT: SettingsSection
BESCHREIBUNG: Abschnitt in einem Settings-Panel
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Abschnitts
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: title
    TYP: String
    BESCHREIBUNG: Titel des Abschnitts
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: controls
    TYP: Vec<SettingsControl>
    BESCHREIBUNG: Steuerelemente im Abschnitt
    WERTEBEREICH: Gültige SettingsControl-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - id darf nicht leer sein
  - title darf nicht leer sein
```

```
ENTITÄT: SettingsControl
BESCHREIBUNG: Steuerelement in einem Settings-Abschnitt
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Steuerelements
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: label
    TYP: String
    BESCHREIBUNG: Beschriftung des Steuerelements
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: description
    TYP: Option<String>
    BESCHREIBUNG: Optionale Beschreibung des Steuerelements
    WERTEBEREICH: Zeichenkette oder None
    STANDARDWERT: None
  - NAME: control_type
    TYP: SettingsControlType
    BESCHREIBUNG: Typ des Steuerelements
    WERTEBEREICH: {
      Toggle { setting_path: SettingPath },
      Slider { setting_path: SettingPath, min: f64, max: f64, step: f64 },
      ComboBox { setting_path: SettingPath, options: Vec<String> },
      ColorPicker { setting_path: SettingPath },
      FileChooser { setting_path: SettingPath, file_type: FileType },
      Button { action: String, params: HashMap<String, String> }
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - id darf nicht leer sein
  - label darf nicht leer sein
  - Bei control_type muss setting_path gültig sein, außer bei Button
  - Bei Slider müssen min < max und step > 0 sein
  - Bei ComboBox darf options nicht leer sein
  - Bei Button darf action nicht leer sein
```
