# SPEC-MODULE-SYSTEM-WINDOWMANAGER-v1.0.0: NovaDE Fenstermanager-Modul (Teil 1)

```
SPEZIFIKATION: SPEC-MODULE-SYSTEM-WINDOWMANAGER-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0, SPEC-LAYER-CORE-v1.0.0, SPEC-LAYER-SYSTEM-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Diese Spezifikation definiert das Fenstermanager-Modul (`system::windowmanager`) der NovaDE-Systemschicht. Das Modul stellt die grundlegende Infrastruktur für die Verwaltung von Fenstern und deren Anordnung auf dem Bildschirm bereit und definiert die Mechanismen zur Fensterplatzierung, -größenänderung, -fokussierung und -interaktion. Der Geltungsbereich umfasst alle Komponenten und Schnittstellen des Fenstermanager-Moduls sowie deren Interaktionen mit anderen Modulen.

## 2. Definitionen

### 2.1 Allgemeine Begriffe

- **Fenster**: Rechteckiger Bereich auf dem Bildschirm, der eine Anwendung oder einen Teil davon darstellt
- **Fenstermanager**: Komponente, die für die Verwaltung von Fenstern zuständig ist
- **Fensterdekoration**: Rahmen, Titelleiste und Steuerelemente eines Fensters
- **Fenstergeometrie**: Position und Größe eines Fensters
- **Fensterzustand**: Zustand eines Fensters (z.B. normal, maximiert, minimiert)
- **Fensterfokus**: Fenster, das aktuell Eingaben erhält
- **Fensterstapel**: Anordnung von Fenstern in der Z-Achse
- **Arbeitsbereich**: Bereich des Bildschirms, der für Fenster verfügbar ist

### 2.2 Modulspezifische Begriffe

- **WindowManager**: Zentrale Komponente für die Fensterverwaltung
- **Window**: Repräsentation eines Fensters
- **WindowDecorator**: Komponente für die Fensterdekoration
- **WindowLayout**: Komponente für die Fensteranordnung
- **WindowFocus**: Komponente für die Fensterfokussierung
- **WindowStack**: Komponente für die Verwaltung des Fensterstapels
- **WindowGeometry**: Datenstruktur für die Fenstergeometrie
- **WindowState**: Datenstruktur für den Fensterzustand
- **WindowType**: Datenstruktur für den Fenstertyp
- **WindowId**: Eindeutiger Bezeichner für ein Fenster
- **Workspace**: Repräsentation eines Arbeitsbereichs

## 3. Anforderungen

### 3.1 Funktionale Anforderungen

1. Das Modul MUSS Mechanismen zur Erstellung und Zerstörung von Fenstern bereitstellen.
2. Das Modul MUSS Mechanismen zur Änderung der Fenstergeometrie bereitstellen.
3. Das Modul MUSS Mechanismen zur Änderung des Fensterzustands bereitstellen.
4. Das Modul MUSS Mechanismen zur Fensterfokussierung bereitstellen.
5. Das Modul MUSS Mechanismen zur Verwaltung des Fensterstapels bereitstellen.
6. Das Modul MUSS Mechanismen zur Fensterdekoration bereitstellen.
7. Das Modul MUSS Mechanismen zur Fensteranordnung bereitstellen.
8. Das Modul MUSS Mechanismen zur Verwaltung von Arbeitsbereichen bereitstellen.
9. Das Modul MUSS Mechanismen zur Fensterinteraktion bereitstellen.
10. Das Modul MUSS Mechanismen zur Benachrichtigung über Fensteränderungen bereitstellen.
11. Das Modul MUSS Mechanismen zur Fensteranimation bereitstellen.
12. Das Modul MUSS Mechanismen zur Unterstützung von Multi-Monitor-Setups bereitstellen.

### 3.2 Nicht-funktionale Anforderungen

1. Das Modul MUSS effizient mit Ressourcen umgehen.
2. Das Modul MUSS thread-safe sein.
3. Das Modul MUSS eine klare und konsistente API bereitstellen.
4. Das Modul MUSS gut dokumentiert sein.
5. Das Modul MUSS leicht erweiterbar sein.
6. Das Modul MUSS robust gegen Fehleingaben sein.
7. Das Modul MUSS minimale externe Abhängigkeiten haben.
8. Das Modul MUSS eine hohe Performance bieten.
9. Das Modul MUSS eine geringe Latenz bei Fensteroperationen bieten.
10. Das Modul MUSS eine hohe Zuverlässigkeit bieten.

## 4. Architektur

### 4.1 Komponentenstruktur

Das Fenstermanager-Modul besteht aus den folgenden Komponenten:

1. **WindowManager** (`window_manager.rs`): Zentrale Komponente für die Fensterverwaltung
2. **Window** (`window.rs`): Repräsentation eines Fensters
3. **WindowDecorator** (`window_decorator.rs`): Komponente für die Fensterdekoration
4. **WindowLayout** (`window_layout.rs`): Komponente für die Fensteranordnung
5. **WindowFocus** (`window_focus.rs`): Komponente für die Fensterfokussierung
6. **WindowStack** (`window_stack.rs`): Komponente für die Verwaltung des Fensterstapels
7. **Workspace** (`workspace.rs`): Repräsentation eines Arbeitsbereichs
8. **WorkspaceManager** (`workspace_manager.rs`): Komponente für die Verwaltung von Arbeitsbereichen
9. **WindowAnimator** (`window_animator.rs`): Komponente für Fensteranimationen
10. **WindowEvents** (`window_events.rs`): Komponente für Fensterbenachrichtigungen
11. **WindowGeometry** (`window_geometry.rs`): Datenstruktur für die Fenstergeometrie
12. **WindowState** (`window_state.rs`): Datenstruktur für den Fensterzustand
13. **WindowType** (`window_type.rs`): Datenstruktur für den Fenstertyp
14. **WindowId** (`window_id.rs`): Datenstruktur für Fensterbezeichner
15. **MonitorManager** (`monitor_manager.rs`): Komponente für die Verwaltung von Monitoren

### 4.2 Abhängigkeiten

Das Fenstermanager-Modul hat folgende Abhängigkeiten:

1. **Interne Abhängigkeiten**:
   - `core::errors`: Für die Fehlerbehandlung
   - `core::config`: Für die Konfiguration
   - `core::logging`: Für das Logging
   - `system::display`: Für die Anzeige von Fenstern
   - `system::input`: Für die Eingabebehandlung

2. **Externe Abhängigkeiten**:
   - `x11-dl`: Für die X11-Integration
   - `wayland-client`: Für die Wayland-Integration
   - `wlroots`: Für die wlroots-Integration
   - `cairo`: Für die Grafikausgabe
   - `pango`: Für die Textdarstellung

## 5. Schnittstellen

### 5.1 WindowManager

```
SCHNITTSTELLE: system::windowmanager::WindowManager
BESCHREIBUNG: Zentrale Komponente für die Fensterverwaltung
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue WindowManager-Instanz
    PARAMETER:
      - NAME: config
        TYP: WindowManagerConfig
        BESCHREIBUNG: Konfiguration für den WindowManager
        EINSCHRÄNKUNGEN: Muss eine gültige WindowManagerConfig sein
    RÜCKGABETYP: Result<WindowManager, WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des WindowManagers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue WindowManager-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des WindowManagers auftritt
  
  - NAME: initialize
    BESCHREIBUNG: Initialisiert den WindowManager
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler bei der Initialisierung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der WindowManager wird initialisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Initialisierung auftritt
  
  - NAME: shutdown
    BESCHREIBUNG: Fährt den WindowManager herunter
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler beim Herunterfahren auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der WindowManager wird heruntergefahren
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Herunterfahren auftritt
  
  - NAME: create_window
    BESCHREIBUNG: Erstellt ein neues Fenster
    PARAMETER:
      - NAME: params
        TYP: WindowCreateParams
        BESCHREIBUNG: Parameter für die Fenstererstellung
        EINSCHRÄNKUNGEN: Muss gültige WindowCreateParams sein
    RÜCKGABETYP: Result<WindowId, WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler bei der Fenstererstellung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein neues Fenster wird erstellt und eine WindowId wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Fenstererstellung auftritt
  
  - NAME: destroy_window
    BESCHREIBUNG: Zerstört ein Fenster
    PARAMETER:
      - NAME: id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler bei der Fensterzerstörung auftritt oder das Fenster nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Fenster wird zerstört
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Fensterzerstörung auftritt oder das Fenster nicht gefunden wird
  
  - NAME: get_window
    BESCHREIBUNG: Gibt ein Fenster zurück
    PARAMETER:
      - NAME: id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
    RÜCKGABETYP: Result<&Window, WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn das Fenster nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Fenster wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Fenster nicht gefunden wird
  
  - NAME: get_window_mut
    BESCHREIBUNG: Gibt ein veränderbares Fenster zurück
    PARAMETER:
      - NAME: id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
    RÜCKGABETYP: Result<&mut Window, WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn das Fenster nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Fenster wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Fenster nicht gefunden wird
  
  - NAME: get_all_windows
    BESCHREIBUNG: Gibt alle Fenster zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<&Window>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Alle Fenster werden zurückgegeben
  
  - NAME: get_focused_window
    BESCHREIBUNG: Gibt das fokussierte Fenster zurück
    PARAMETER: Keine
    RÜCKGABETYP: Option<&Window>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das fokussierte Fenster wird zurückgegeben, wenn eines existiert
      - None wird zurückgegeben, wenn kein Fenster fokussiert ist
  
  - NAME: focus_window
    BESCHREIBUNG: Fokussiert ein Fenster
    PARAMETER:
      - NAME: id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler bei der Fensterfokussierung auftritt oder das Fenster nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Fenster wird fokussiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Fensterfokussierung auftritt oder das Fenster nicht gefunden wird
  
  - NAME: set_window_geometry
    BESCHREIBUNG: Setzt die Geometrie eines Fensters
    PARAMETER:
      - NAME: id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
      - NAME: geometry
        TYP: WindowGeometry
        BESCHREIBUNG: Neue Geometrie
        EINSCHRÄNKUNGEN: Muss eine gültige WindowGeometry sein
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler beim Setzen der Geometrie auftritt oder das Fenster nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Geometrie des Fensters wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen der Geometrie auftritt oder das Fenster nicht gefunden wird
  
  - NAME: set_window_state
    BESCHREIBUNG: Setzt den Zustand eines Fensters
    PARAMETER:
      - NAME: id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
      - NAME: state
        TYP: WindowState
        BESCHREIBUNG: Neuer Zustand
        EINSCHRÄNKUNGEN: Muss ein gültiger WindowState sein
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler beim Setzen des Zustands auftritt oder das Fenster nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Zustand des Fensters wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen des Zustands auftritt oder das Fenster nicht gefunden wird
  
  - NAME: raise_window
    BESCHREIBUNG: Hebt ein Fenster im Fensterstapel an
    PARAMETER:
      - NAME: id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler beim Anheben des Fensters auftritt oder das Fenster nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Fenster wird im Fensterstapel angehoben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Anheben des Fensters auftritt oder das Fenster nicht gefunden wird
  
  - NAME: lower_window
    BESCHREIBUNG: Senkt ein Fenster im Fensterstapel ab
    PARAMETER:
      - NAME: id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler beim Absenken des Fensters auftritt oder das Fenster nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Fenster wird im Fensterstapel abgesenkt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Absenken des Fensters auftritt oder das Fenster nicht gefunden wird
  
  - NAME: get_current_workspace
    BESCHREIBUNG: Gibt den aktuellen Arbeitsbereich zurück
    PARAMETER: Keine
    RÜCKGABETYP: &Workspace
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der aktuelle Arbeitsbereich wird zurückgegeben
  
  - NAME: set_current_workspace
    BESCHREIBUNG: Setzt den aktuellen Arbeitsbereich
    PARAMETER:
      - NAME: id
        TYP: WorkspaceId
        BESCHREIBUNG: ID des Arbeitsbereichs
        EINSCHRÄNKUNGEN: Muss eine gültige WorkspaceId sein
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler beim Setzen des Arbeitsbereichs auftritt oder der Arbeitsbereich nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der aktuelle Arbeitsbereich wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen des Arbeitsbereichs auftritt oder der Arbeitsbereich nicht gefunden wird
  
  - NAME: create_workspace
    BESCHREIBUNG: Erstellt einen neuen Arbeitsbereich
    PARAMETER:
      - NAME: name
        TYP: String
        BESCHREIBUNG: Name des Arbeitsbereichs
        EINSCHRÄNKUNGEN: Darf nicht leer sein
    RÜCKGABETYP: Result<WorkspaceId, WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des Arbeitsbereichs auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein neuer Arbeitsbereich wird erstellt und eine WorkspaceId wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des Arbeitsbereichs auftritt
  
  - NAME: destroy_workspace
    BESCHREIBUNG: Zerstört einen Arbeitsbereich
    PARAMETER:
      - NAME: id
        TYP: WorkspaceId
        BESCHREIBUNG: ID des Arbeitsbereichs
        EINSCHRÄNKUNGEN: Muss eine gültige WorkspaceId sein
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler bei der Zerstörung des Arbeitsbereichs auftritt oder der Arbeitsbereich nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Arbeitsbereich wird zerstört
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Zerstörung des Arbeitsbereichs auftritt oder der Arbeitsbereich nicht gefunden wird
  
  - NAME: get_all_workspaces
    BESCHREIBUNG: Gibt alle Arbeitsbereiche zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<&Workspace>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Alle Arbeitsbereiche werden zurückgegeben
  
  - NAME: move_window_to_workspace
    BESCHREIBUNG: Verschiebt ein Fenster in einen Arbeitsbereich
    PARAMETER:
      - NAME: window_id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
      - NAME: workspace_id
        TYP: WorkspaceId
        BESCHREIBUNG: ID des Arbeitsbereichs
        EINSCHRÄNKUNGEN: Muss eine gültige WorkspaceId sein
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn ein Fehler beim Verschieben des Fensters auftritt oder das Fenster oder der Arbeitsbereich nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Fenster wird in den Arbeitsbereich verschoben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Verschieben des Fensters auftritt oder das Fenster oder der Arbeitsbereich nicht gefunden wird
  
  - NAME: register_window_event_listener
    BESCHREIBUNG: Registriert einen Listener für Fensterereignisse
    PARAMETER:
      - NAME: listener
        TYP: Box<dyn Fn(WindowEvent) + Send + Sync + 'static>
        BESCHREIBUNG: Listener-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: ListenerId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird registriert und eine ListenerId wird zurückgegeben
  
  - NAME: unregister_window_event_listener
    BESCHREIBUNG: Entfernt einen Listener für Fensterereignisse
    PARAMETER:
      - NAME: id
        TYP: ListenerId
        BESCHREIBUNG: ID des Listeners
        EINSCHRÄNKUNGEN: Muss eine gültige ListenerId sein
    RÜCKGABETYP: Result<(), WindowManagerError>
    FEHLER:
      - TYP: WindowManagerError
        BEDINGUNG: Wenn der Listener nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Listener nicht gefunden wird
```

### 5.2 Window

```
SCHNITTSTELLE: system::windowmanager::Window
BESCHREIBUNG: Repräsentation eines Fensters
VERSION: 1.0.0
OPERATIONEN:
  - NAME: get_id
    BESCHREIBUNG: Gibt die ID des Fensters zurück
    PARAMETER: Keine
    RÜCKGABETYP: WindowId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die ID des Fensters wird zurückgegeben
  
  - NAME: get_title
    BESCHREIBUNG: Gibt den Titel des Fensters zurück
    PARAMETER: Keine
    RÜCKGABETYP: &str
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Titel des Fensters wird zurückgegeben
  
  - NAME: set_title
    BESCHREIBUNG: Setzt den Titel des Fensters
    PARAMETER:
      - NAME: title
        TYP: String
        BESCHREIBUNG: Neuer Titel
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), WindowError>
    FEHLER:
      - TYP: WindowError
        BEDINGUNG: Wenn ein Fehler beim Setzen des Titels auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Titel des Fensters wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen des Titels auftritt
  
  - NAME: get_geometry
    BESCHREIBUNG: Gibt die Geometrie des Fensters zurück
    PARAMETER: Keine
    RÜCKGABETYP: WindowGeometry
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Geometrie des Fensters wird zurückgegeben
  
  - NAME: set_geometry
    BESCHREIBUNG: Setzt die Geometrie des Fensters
    PARAMETER:
      - NAME: geometry
        TYP: WindowGeometry
        BESCHREIBUNG: Neue Geometrie
        EINSCHRÄNKUNGEN: Muss eine gültige WindowGeometry sein
    RÜCKGABETYP: Result<(), WindowError>
    FEHLER:
      - TYP: WindowError
        BEDINGUNG: Wenn ein Fehler beim Setzen der Geometrie auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Geometrie des Fensters wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen der Geometrie auftritt
  
  - NAME: get_state
    BESCHREIBUNG: Gibt den Zustand des Fensters zurück
    PARAMETER: Keine
    RÜCKGABETYP: WindowState
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Zustand des Fensters wird zurückgegeben
  
  - NAME: set_state
    BESCHREIBUNG: Setzt den Zustand des Fensters
    PARAMETER:
      - NAME: state
        TYP: WindowState
        BESCHREIBUNG: Neuer Zustand
        EINSCHRÄNKUNGEN: Muss ein gültiger WindowState sein
    RÜCKGABETYP: Result<(), WindowError>
    FEHLER:
      - TYP: WindowError
        BEDINGUNG: Wenn ein Fehler beim Setzen des Zustands auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Zustand des Fensters wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen des Zustands auftritt
  
  - NAME: get_type
    BESCHREIBUNG: Gibt den Typ des Fensters zurück
    PARAMETER: Keine
    RÜCKGABETYP: WindowType
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Typ des Fensters wird zurückgegeben
  
  - NAME: is_decorated
    BESCHREIBUNG: Prüft, ob das Fenster dekoriert ist
    PARAMETER: Keine
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn das Fenster dekoriert ist
      - false wird zurückgegeben, wenn das Fenster nicht dekoriert ist
  
  - NAME: set_decorated
    BESCHREIBUNG: Setzt, ob das Fenster dekoriert ist
    PARAMETER:
      - NAME: decorated
        TYP: bool
        BESCHREIBUNG: Ob das Fenster dekoriert sein soll
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), WindowError>
    FEHLER:
      - TYP: WindowError
        BEDINGUNG: Wenn ein Fehler beim Setzen der Dekoration auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Dekoration des Fensters wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen der Dekoration auftritt
  
  - NAME: is_resizable
    BESCHREIBUNG: Prüft, ob das Fenster größenveränderbar ist
    PARAMETER: Keine
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn das Fenster größenveränderbar ist
      - false wird zurückgegeben, wenn das Fenster nicht größenveränderbar ist
  
  - NAME: set_resizable
    BESCHREIBUNG: Setzt, ob das Fenster größenveränderbar ist
    PARAMETER:
      - NAME: resizable
        TYP: bool
        BESCHREIBUNG: Ob das Fenster größenveränderbar sein soll
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), WindowError>
    FEHLER:
      - TYP: WindowError
        BEDINGUNG: Wenn ein Fehler beim Setzen der Größenveränderbarkeit auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Größenveränderbarkeit des Fensters wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen der Größenveränderbarkeit auftritt
  
  - NAME: is_minimizable
    BESCHREIBUNG: Prüft, ob das Fenster minimierbar ist
    PARAMETER: Keine
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn das Fenster minimierbar ist
      - false wird zurückgegeben, wenn das Fenster nicht minimierbar ist
  
  - NAME: set_minimizable
    BESCHREIBUNG: Setzt, ob das Fenster minimierbar ist
    PARAMETER:
      - NAME: minimizable
        TYP: bool
        BESCHREIBUNG: Ob das Fenster minimierbar sein soll
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), WindowError>
    FEHLER:
      - TYP: WindowError
        BEDINGUNG: Wenn ein Fehler beim Setzen der Minimierbarkeit auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Minimierbarkeit des Fensters wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen der Minimierbarkeit auftritt
  
  - NAME: is_maximizable
    BESCHREIBUNG: Prüft, ob das Fenster maximierbar ist
    PARAMETER: Keine
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn das Fenster maximierbar ist
      - false wird zurückgegeben, wenn das Fenster nicht maximierbar ist
  
  - NAME: set_maximizable
    BESCHREIBUNG: Setzt, ob das Fenster maximierbar ist
    PARAMETER:
      - NAME: maximizable
        TYP: bool
        BESCHREIBUNG: Ob das Fenster maximierbar sein soll
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), WindowError>
    FEHLER:
      - TYP: WindowError
        BEDINGUNG: Wenn ein Fehler beim Setzen der Maximierbarkeit auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Maximierbarkeit des Fensters wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen der Maximierbarkeit auftritt
```

## 6. Datenmodell (Teil 1)

### 6.1 WindowId

```
ENTITÄT: WindowId
BESCHREIBUNG: Eindeutiger Bezeichner für ein Fenster
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.2 WindowGeometry

```
ENTITÄT: WindowGeometry
BESCHREIBUNG: Geometrie eines Fensters
ATTRIBUTE:
  - NAME: x
    TYP: i32
    BESCHREIBUNG: X-Koordinate
    WERTEBEREICH: Ganzzahlen
    STANDARDWERT: 0
  - NAME: y
    TYP: i32
    BESCHREIBUNG: Y-Koordinate
    WERTEBEREICH: Ganzzahlen
    STANDARDWERT: 0
  - NAME: width
    TYP: u32
    BESCHREIBUNG: Breite
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 800
  - NAME: height
    TYP: u32
    BESCHREIBUNG: Höhe
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 600
INVARIANTEN:
  - width muss größer als 0 sein
  - height muss größer als 0 sein
```

### 6.3 WindowState

```
ENTITÄT: WindowState
BESCHREIBUNG: Zustand eines Fensters
ATTRIBUTE:
  - NAME: state
    TYP: Enum
    BESCHREIBUNG: Zustand
    WERTEBEREICH: {
      Normal,
      Minimized,
      Maximized,
      Fullscreen,
      Hidden
    }
    STANDARDWERT: Normal
INVARIANTEN:
  - Keine
```

### 6.4 WindowType

```
ENTITÄT: WindowType
BESCHREIBUNG: Typ eines Fensters
ATTRIBUTE:
  - NAME: type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      Normal,
      Dialog,
      Popup,
      Menu,
      Toolbar,
      Utility,
      Splash,
      Desktop,
      Dock,
      Notification
    }
    STANDARDWERT: Normal
INVARIANTEN:
  - Keine
```

### 6.5 WorkspaceId

```
ENTITÄT: WorkspaceId
BESCHREIBUNG: Eindeutiger Bezeichner für einen Arbeitsbereich
ATTRIBUTE:
  - NAME: id
    TYP: u32
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.6 WindowCreateParams

```
ENTITÄT: WindowCreateParams
BESCHREIBUNG: Parameter für die Fenstererstellung
ATTRIBUTE:
  - NAME: title
    TYP: String
    BESCHREIBUNG: Titel des Fensters
    WERTEBEREICH: Zeichenkette
    STANDARDWERT: "Unbenannt"
  - NAME: geometry
    TYP: Option<WindowGeometry>
    BESCHREIBUNG: Geometrie des Fensters
    WERTEBEREICH: Gültige WindowGeometry oder None
    STANDARDWERT: None
  - NAME: type
    TYP: WindowType
    BESCHREIBUNG: Typ des Fensters
    WERTEBEREICH: Gültiger WindowType
    STANDARDWERT: WindowType::Normal
  - NAME: decorated
    TYP: bool
    BESCHREIBUNG: Ob das Fenster dekoriert sein soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: resizable
    TYP: bool
    BESCHREIBUNG: Ob das Fenster größenveränderbar sein soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: minimizable
    TYP: bool
    BESCHREIBUNG: Ob das Fenster minimierbar sein soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: maximizable
    TYP: bool
    BESCHREIBUNG: Ob das Fenster maximierbar sein soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: parent
    TYP: Option<WindowId>
    BESCHREIBUNG: Elternfenster
    WERTEBEREICH: Gültige WindowId oder None
    STANDARDWERT: None
  - NAME: workspace
    TYP: Option<WorkspaceId>
    BESCHREIBUNG: Arbeitsbereich
    WERTEBEREICH: Gültige WorkspaceId oder None
    STANDARDWERT: None
INVARIANTEN:
  - Keine
```

### 6.7 WindowEvent

```
ENTITÄT: WindowEvent
BESCHREIBUNG: Ereignis für ein Fenster
ATTRIBUTE:
  - NAME: event_type
    TYP: Enum
    BESCHREIBUNG: Typ des Ereignisses
    WERTEBEREICH: {
      Created { window_id: WindowId },
      Destroyed { window_id: WindowId },
      Moved { window_id: WindowId, x: i32, y: i32 },
      Resized { window_id: WindowId, width: u32, height: u32 },
      StateChanged { window_id: WindowId, state: WindowState },
      FocusGained { window_id: WindowId },
      FocusLost { window_id: WindowId },
      TitleChanged { window_id: WindowId, title: String },
      WorkspaceChanged { window_id: WindowId, workspace_id: WorkspaceId }
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Keine
```
