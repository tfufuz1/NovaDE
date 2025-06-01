# SPEC-MODULE-UI-PANEL-v1.0.0: NovaDE Panel-Modul (Teil 1)

```
SPEZIFIKATION: SPEC-MODULE-UI-PANEL-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0, SPEC-LAYER-CORE-v1.0.0, SPEC-LAYER-UI-v1.0.0, SPEC-MODULE-DOMAIN-THEMING-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Diese Spezifikation definiert das Panel-Modul (`ui::panel`) der NovaDE-UI-Schicht. Das Modul stellt die grundlegende Infrastruktur für die Erstellung und Verwaltung von Panels (Leisten) in der Desktop-Umgebung bereit und definiert die Mechanismen zur Anzeige von Widgets, Applets und Indikatoren. Der Geltungsbereich umfasst alle Komponenten und Schnittstellen des Panel-Moduls sowie deren Interaktionen mit anderen Modulen.

## 2. Definitionen

### 2.1 Allgemeine Begriffe

- **Panel**: Leiste am Bildschirmrand zur Anzeige von Widgets, Applets und Indikatoren
- **Widget**: Grafisches Element zur Anzeige von Informationen oder zur Interaktion
- **Applet**: Kleines Programm, das in einem Panel ausgeführt wird
- **Indikator**: Element zur Anzeige des Status eines Systems oder einer Anwendung
- **Launcher**: Element zum Starten von Anwendungen
- **Taskleiste**: Element zur Anzeige und Verwaltung laufender Anwendungen
- **Systemablage**: Bereich für Statussymbole und Benachrichtigungen
- **Dock**: Spezielles Panel zur Anzeige von Anwendungssymbolen

### 2.2 Modulspezifische Begriffe

- **PanelManager**: Zentrale Komponente für die Verwaltung von Panels
- **PanelContainer**: Container für ein Panel
- **PanelLayout**: Layout eines Panels
- **PanelWidget**: Widget in einem Panel
- **PanelApplet**: Applet in einem Panel
- **PanelIndicator**: Indikator in einem Panel
- **PanelLauncher**: Launcher in einem Panel
- **PanelTaskbar**: Taskleiste in einem Panel
- **PanelSystemTray**: Systemablage in einem Panel
- **PanelDock**: Dock in einem Panel

## 3. Anforderungen

### 3.1 Funktionale Anforderungen

1. Das Modul MUSS Mechanismen zur Erstellung und Verwaltung von Panels bereitstellen.
2. Das Modul MUSS Mechanismen zur Positionierung von Panels bereitstellen.
3. Das Modul MUSS Mechanismen zur Größenanpassung von Panels bereitstellen.
4. Das Modul MUSS Mechanismen zur Anzeige von Widgets in Panels bereitstellen.
5. Das Modul MUSS Mechanismen zur Anzeige von Applets in Panels bereitstellen.
6. Das Modul MUSS Mechanismen zur Anzeige von Indikatoren in Panels bereitstellen.
7. Das Modul MUSS Mechanismen zur Anzeige von Launchern in Panels bereitstellen.
8. Das Modul MUSS Mechanismen zur Anzeige einer Taskleiste in Panels bereitstellen.
9. Das Modul MUSS Mechanismen zur Anzeige einer Systemablage in Panels bereitstellen.
10. Das Modul MUSS Mechanismen zur Anzeige eines Docks in Panels bereitstellen.
11. Das Modul MUSS Mechanismen zur Konfiguration von Panels bereitstellen.
12. Das Modul MUSS Mechanismen zur Persistenz von Panel-Konfigurationen bereitstellen.
13. Das Modul MUSS Mechanismen zur Integration mit dem Theming-System bereitstellen.
14. Das Modul MUSS Mechanismen zur Integration mit dem Fenstermanager bereitstellen.

### 3.2 Nicht-funktionale Anforderungen

1. Das Modul MUSS effizient mit Ressourcen umgehen.
2. Das Modul MUSS thread-safe sein.
3. Das Modul MUSS eine klare und konsistente API bereitstellen.
4. Das Modul MUSS gut dokumentiert sein.
5. Das Modul MUSS leicht erweiterbar sein.
6. Das Modul MUSS robust gegen Fehleingaben sein.
7. Das Modul MUSS minimale externe Abhängigkeiten haben.
8. Das Modul MUSS eine hohe Performance bieten.
9. Das Modul MUSS eine geringe Latenz bei der Anzeige bieten.
10. Das Modul MUSS eine hohe Zuverlässigkeit bieten.

## 4. Architektur

### 4.1 Komponentenstruktur

Das Panel-Modul besteht aus den folgenden Komponenten:

1. **PanelManager** (`panel_manager.rs`): Zentrale Komponente für die Verwaltung von Panels
2. **PanelContainer** (`panel_container.rs`): Container für ein Panel
3. **PanelLayout** (`panel_layout.rs`): Layout eines Panels
4. **PanelWidget** (`panel_widget.rs`): Widget in einem Panel
5. **PanelApplet** (`panel_applet.rs`): Applet in einem Panel
6. **PanelIndicator** (`panel_indicator.rs`): Indikator in einem Panel
7. **PanelLauncher** (`panel_launcher.rs`): Launcher in einem Panel
8. **PanelTaskbar** (`panel_taskbar.rs`): Taskleiste in einem Panel
9. **PanelSystemTray** (`panel_system_tray.rs`): Systemablage in einem Panel
10. **PanelDock** (`panel_dock.rs`): Dock in einem Panel
11. **PanelConfig** (`panel_config.rs`): Konfiguration eines Panels
12. **PanelPersistence** (`panel_persistence.rs`): Persistenz von Panel-Konfigurationen
13. **PanelTheme** (`panel_theme.rs`): Theming eines Panels
14. **PanelWindowManager** (`panel_window_manager.rs`): Integration mit dem Fenstermanager

### 4.2 Abhängigkeiten

Das Panel-Modul hat folgende Abhängigkeiten:

1. **Interne Abhängigkeiten**:
   - `core::errors`: Für die Fehlerbehandlung
   - `core::config`: Für die Konfiguration
   - `core::logging`: Für das Logging
   - `domain::theming`: Für das Theming
   - `system::windowmanager`: Für die Fensterverwaltung
   - `ui::widget`: Für die Widget-Unterstützung

2. **Externe Abhängigkeiten**:
   - `gtk4`: Für die GUI-Komponenten
   - `cairo`: Für die Grafikausgabe
   - `pango`: Für die Textdarstellung
   - `json`: Für die Konfigurationsdateien
   - `dbus`: Für die Kommunikation mit anderen Anwendungen

## 5. Schnittstellen

### 5.1 PanelManager

```
SCHNITTSTELLE: ui::panel::PanelManager
BESCHREIBUNG: Zentrale Komponente für die Verwaltung von Panels
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue PanelManager-Instanz
    PARAMETER:
      - NAME: config
        TYP: PanelManagerConfig
        BESCHREIBUNG: Konfiguration für den PanelManager
        EINSCHRÄNKUNGEN: Muss eine gültige PanelManagerConfig sein
    RÜCKGABETYP: Result<PanelManager, PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des PanelManagers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue PanelManager-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des PanelManagers auftritt
  
  - NAME: initialize
    BESCHREIBUNG: Initialisiert den PanelManager
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler bei der Initialisierung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der PanelManager wird initialisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Initialisierung auftritt
  
  - NAME: shutdown
    BESCHREIBUNG: Fährt den PanelManager herunter
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler beim Herunterfahren auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der PanelManager wird heruntergefahren
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Herunterfahren auftritt
  
  - NAME: create_panel
    BESCHREIBUNG: Erstellt ein neues Panel
    PARAMETER:
      - NAME: config
        TYP: PanelConfig
        BESCHREIBUNG: Konfiguration für das Panel
        EINSCHRÄNKUNGEN: Muss eine gültige PanelConfig sein
    RÜCKGABETYP: Result<PanelId, PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des Panels auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein neues Panel wird erstellt
      - Eine PanelId wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des Panels auftritt
  
  - NAME: remove_panel
    BESCHREIBUNG: Entfernt ein Panel
    PARAMETER:
      - NAME: id
        TYP: PanelId
        BESCHREIBUNG: ID des Panels
        EINSCHRÄNKUNGEN: Muss eine gültige PanelId sein
    RÜCKGABETYP: Result<(), PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler beim Entfernen des Panels auftritt oder das Panel nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Panel wird entfernt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Entfernen des Panels auftritt oder das Panel nicht gefunden wird
  
  - NAME: get_panel
    BESCHREIBUNG: Gibt ein Panel zurück
    PARAMETER:
      - NAME: id
        TYP: PanelId
        BESCHREIBUNG: ID des Panels
        EINSCHRÄNKUNGEN: Muss eine gültige PanelId sein
    RÜCKGABETYP: Result<&PanelContainer, PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn das Panel nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Panel wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Panel nicht gefunden wird
  
  - NAME: get_panel_mut
    BESCHREIBUNG: Gibt ein veränderbares Panel zurück
    PARAMETER:
      - NAME: id
        TYP: PanelId
        BESCHREIBUNG: ID des Panels
        EINSCHRÄNKUNGEN: Muss eine gültige PanelId sein
    RÜCKGABETYP: Result<&mut PanelContainer, PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn das Panel nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Panel wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Panel nicht gefunden wird
  
  - NAME: get_all_panels
    BESCHREIBUNG: Gibt alle Panels zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<&PanelContainer>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Alle Panels werden zurückgegeben
  
  - NAME: save_panel_config
    BESCHREIBUNG: Speichert die Konfiguration eines Panels
    PARAMETER:
      - NAME: id
        TYP: PanelId
        BESCHREIBUNG: ID des Panels
        EINSCHRÄNKUNGEN: Muss eine gültige PanelId sein
    RÜCKGABETYP: Result<(), PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler beim Speichern der Konfiguration auftritt oder das Panel nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Konfiguration des Panels wird gespeichert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Speichern der Konfiguration auftritt oder das Panel nicht gefunden wird
  
  - NAME: load_panel_config
    BESCHREIBUNG: Lädt die Konfiguration eines Panels
    PARAMETER:
      - NAME: id
        TYP: PanelId
        BESCHREIBUNG: ID des Panels
        EINSCHRÄNKUNGEN: Muss eine gültige PanelId sein
    RÜCKGABETYP: Result<(), PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler beim Laden der Konfiguration auftritt oder das Panel nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Konfiguration des Panels wird geladen
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Laden der Konfiguration auftritt oder das Panel nicht gefunden wird
  
  - NAME: register_panel_event_listener
    BESCHREIBUNG: Registriert einen Listener für Panel-Ereignisse
    PARAMETER:
      - NAME: listener
        TYP: Box<dyn Fn(&PanelEvent) -> bool + Send + Sync + 'static>
        BESCHREIBUNG: Listener-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: ListenerId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird registriert und eine ListenerId wird zurückgegeben
  
  - NAME: unregister_panel_event_listener
    BESCHREIBUNG: Entfernt einen Listener für Panel-Ereignisse
    PARAMETER:
      - NAME: id
        TYP: ListenerId
        BESCHREIBUNG: ID des Listeners
        EINSCHRÄNKUNGEN: Muss eine gültige ListenerId sein
    RÜCKGABETYP: Result<(), PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn der Listener nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Listener nicht gefunden wird
```

### 5.2 PanelContainer

```
SCHNITTSTELLE: ui::panel::PanelContainer
BESCHREIBUNG: Container für ein Panel
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue PanelContainer-Instanz
    PARAMETER:
      - NAME: id
        TYP: PanelId
        BESCHREIBUNG: ID des Panels
        EINSCHRÄNKUNGEN: Muss eine gültige PanelId sein
      - NAME: config
        TYP: PanelConfig
        BESCHREIBUNG: Konfiguration für das Panel
        EINSCHRÄNKUNGEN: Muss eine gültige PanelConfig sein
    RÜCKGABETYP: Result<PanelContainer, PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des PanelContainers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue PanelContainer-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des PanelContainers auftritt
  
  - NAME: get_id
    BESCHREIBUNG: Gibt die ID des Panels zurück
    PARAMETER: Keine
    RÜCKGABETYP: PanelId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die ID des Panels wird zurückgegeben
  
  - NAME: get_config
    BESCHREIBUNG: Gibt die Konfiguration des Panels zurück
    PARAMETER: Keine
    RÜCKGABETYP: &PanelConfig
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Konfiguration des Panels wird zurückgegeben
  
  - NAME: get_config_mut
    BESCHREIBUNG: Gibt die veränderbare Konfiguration des Panels zurück
    PARAMETER: Keine
    RÜCKGABETYP: &mut PanelConfig
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die veränderbare Konfiguration des Panels wird zurückgegeben
  
  - NAME: get_layout
    BESCHREIBUNG: Gibt das Layout des Panels zurück
    PARAMETER: Keine
    RÜCKGABETYP: &PanelLayout
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Layout des Panels wird zurückgegeben
  
  - NAME: get_layout_mut
    BESCHREIBUNG: Gibt das veränderbare Layout des Panels zurück
    PARAMETER: Keine
    RÜCKGABETYP: &mut PanelLayout
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das veränderbare Layout des Panels wird zurückgegeben
  
  - NAME: add_widget
    BESCHREIBUNG: Fügt ein Widget zum Panel hinzu
    PARAMETER:
      - NAME: widget
        TYP: Box<dyn PanelWidget>
        BESCHREIBUNG: Widget
        EINSCHRÄNKUNGEN: Muss ein gültiges PanelWidget sein
      - NAME: position
        TYP: Option<PanelPosition>
        BESCHREIBUNG: Position des Widgets
        EINSCHRÄNKUNGEN: Wenn vorhanden, muss eine gültige PanelPosition sein
    RÜCKGABETYP: Result<PanelWidgetId, PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler beim Hinzufügen des Widgets auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Widget wird zum Panel hinzugefügt
      - Eine PanelWidgetId wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Hinzufügen des Widgets auftritt
  
  - NAME: remove_widget
    BESCHREIBUNG: Entfernt ein Widget aus dem Panel
    PARAMETER:
      - NAME: id
        TYP: PanelWidgetId
        BESCHREIBUNG: ID des Widgets
        EINSCHRÄNKUNGEN: Muss eine gültige PanelWidgetId sein
    RÜCKGABETYP: Result<(), PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler beim Entfernen des Widgets auftritt oder das Widget nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Widget wird aus dem Panel entfernt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Entfernen des Widgets auftritt oder das Widget nicht gefunden wird
  
  - NAME: get_widget
    BESCHREIBUNG: Gibt ein Widget zurück
    PARAMETER:
      - NAME: id
        TYP: PanelWidgetId
        BESCHREIBUNG: ID des Widgets
        EINSCHRÄNKUNGEN: Muss eine gültige PanelWidgetId sein
    RÜCKGABETYP: Result<&Box<dyn PanelWidget>, PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn das Widget nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Widget wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Widget nicht gefunden wird
  
  - NAME: get_widget_mut
    BESCHREIBUNG: Gibt ein veränderbares Widget zurück
    PARAMETER:
      - NAME: id
        TYP: PanelWidgetId
        BESCHREIBUNG: ID des Widgets
        EINSCHRÄNKUNGEN: Muss eine gültige PanelWidgetId sein
    RÜCKGABETYP: Result<&mut Box<dyn PanelWidget>, PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn das Widget nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Widget wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Widget nicht gefunden wird
  
  - NAME: get_all_widgets
    BESCHREIBUNG: Gibt alle Widgets zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<&Box<dyn PanelWidget>>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Alle Widgets werden zurückgegeben
  
  - NAME: show
    BESCHREIBUNG: Zeigt das Panel an
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler beim Anzeigen des Panels auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Panel wird angezeigt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Anzeigen des Panels auftritt
  
  - NAME: hide
    BESCHREIBUNG: Versteckt das Panel
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), PanelError>
    FEHLER:
      - TYP: PanelError
        BEDINGUNG: Wenn ein Fehler beim Verstecken des Panels auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Panel wird versteckt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Verstecken des Panels auftritt
  
  - NAME: is_visible
    BESCHREIBUNG: Prüft, ob das Panel sichtbar ist
    PARAMETER: Keine
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn das Panel sichtbar ist
      - false wird zurückgegeben, wenn das Panel nicht sichtbar ist
```

## 6. Datenmodell (Teil 1)

### 6.1 PanelId

```
ENTITÄT: PanelId
BESCHREIBUNG: Eindeutiger Bezeichner für ein Panel
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.2 PanelWidgetId

```
ENTITÄT: PanelWidgetId
BESCHREIBUNG: Eindeutiger Bezeichner für ein Widget in einem Panel
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.3 PanelPosition

```
ENTITÄT: PanelPosition
BESCHREIBUNG: Position eines Panels oder Widgets
ATTRIBUTE:
  - NAME: position_type
    TYP: Enum
    BESCHREIBUNG: Typ der Position
    WERTEBEREICH: {
      Top,
      Bottom,
      Left,
      Right,
      TopLeft,
      TopRight,
      BottomLeft,
      BottomRight,
      Center,
      Custom { x: i32, y: i32 }
    }
    STANDARDWERT: Top
INVARIANTEN:
  - Keine
```

### 6.4 PanelSize

```
ENTITÄT: PanelSize
BESCHREIBUNG: Größe eines Panels
ATTRIBUTE:
  - NAME: width
    TYP: u32
    BESCHREIBUNG: Breite
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
  - NAME: height
    TYP: u32
    BESCHREIBUNG: Höhe
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
  - NAME: size_type
    TYP: PanelSizeType
    BESCHREIBUNG: Typ der Größe
    WERTEBEREICH: Gültiger PanelSizeType
    STANDARDWERT: PanelSizeType::Auto
INVARIANTEN:
  - Keine
```

### 6.5 PanelSizeType

```
ENTITÄT: PanelSizeType
BESCHREIBUNG: Typ der Größe eines Panels
ATTRIBUTE:
  - NAME: size_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      Auto,
      Fixed,
      Percentage
    }
    STANDARDWERT: Auto
INVARIANTEN:
  - Keine
```

### 6.6 PanelOrientation

```
ENTITÄT: PanelOrientation
BESCHREIBUNG: Ausrichtung eines Panels
ATTRIBUTE:
  - NAME: orientation
    TYP: Enum
    BESCHREIBUNG: Ausrichtung
    WERTEBEREICH: {
      Horizontal,
      Vertical
    }
    STANDARDWERT: Horizontal
INVARIANTEN:
  - Keine
```

### 6.7 PanelAlignment

```
ENTITÄT: PanelAlignment
BESCHREIBUNG: Ausrichtung von Elementen in einem Panel
ATTRIBUTE:
  - NAME: alignment
    TYP: Enum
    BESCHREIBUNG: Ausrichtung
    WERTEBEREICH: {
      Start,
      Center,
      End,
      Fill
    }
    STANDARDWERT: Start
INVARIANTEN:
  - Keine
```

### 6.8 PanelVisibility

```
ENTITÄT: PanelVisibility
BESCHREIBUNG: Sichtbarkeit eines Panels
ATTRIBUTE:
  - NAME: visibility
    TYP: Enum
    BESCHREIBUNG: Sichtbarkeit
    WERTEBEREICH: {
      Always,
      Auto,
      Never,
      Intelligent
    }
    STANDARDWERT: Always
INVARIANTEN:
  - Keine
```

### 6.9 PanelLayer

```
ENTITÄT: PanelLayer
BESCHREIBUNG: Ebene eines Panels
ATTRIBUTE:
  - NAME: layer
    TYP: Enum
    BESCHREIBUNG: Ebene
    WERTEBEREICH: {
      Background,
      Bottom,
      Normal,
      Top,
      Overlay
    }
    STANDARDWERT: Normal
INVARIANTEN:
  - Keine
```

### 6.10 PanelBehavior

```
ENTITÄT: PanelBehavior
BESCHREIBUNG: Verhalten eines Panels
ATTRIBUTE:
  - NAME: behavior
    TYP: Enum
    BESCHREIBUNG: Verhalten
    WERTEBEREICH: {
      Normal,
      AutoHide,
      DodgeWindows,
      DodgeActive,
      DodgeMaximized
    }
    STANDARDWERT: Normal
INVARIANTEN:
  - Keine
```
