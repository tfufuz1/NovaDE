# SPEC-MODULE-UI-LAUNCHER-v1.0.0: NovaDE Anwendungsstarter-Modul (Teil 1)

```
SPEZIFIKATION: SPEC-MODULE-UI-LAUNCHER-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0, SPEC-LAYER-CORE-v1.0.0, SPEC-LAYER-UI-v1.0.0, SPEC-MODULE-DOMAIN-APPLICATION-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Diese Spezifikation definiert das Anwendungsstarter-Modul (`ui::launcher`) der NovaDE-UI-Schicht. Das Modul stellt die Benutzeroberfläche für den Zugriff auf und das Starten von Anwendungen bereit und definiert die Mechanismen zur Darstellung, Suche, Kategorisierung und Interaktion mit Anwendungen. Der Geltungsbereich umfasst alle Komponenten und Schnittstellen des Anwendungsstarter-Moduls sowie deren Interaktionen mit anderen Modulen.

## 2. Definitionen

### 2.1 Allgemeine Begriffe

- **Anwendungsstarter**: Benutzeroberfläche zum Starten von Anwendungen
- **Anwendungssymbol**: Grafische Darstellung einer Anwendung
- **Anwendungskategorie**: Thematische Gruppierung von Anwendungen
- **Anwendungssuche**: Mechanismus zum Suchen von Anwendungen
- **Favoriten**: Vom Benutzer bevorzugte Anwendungen
- **Kürzlich verwendet**: Kürzlich vom Benutzer verwendete Anwendungen
- **Häufig verwendet**: Häufig vom Benutzer verwendete Anwendungen
- **Anwendungsmenü**: Hierarchische Darstellung von Anwendungen
- **Schnellstarter**: Kompakte Darstellung häufig verwendeter Anwendungen
- **Anwendungsvorschau**: Vorschau einer Anwendung vor dem Start

### 2.2 Modulspezifische Begriffe

- **LauncherManager**: Zentrale Komponente für die Verwaltung des Anwendungsstarters
- **LauncherView**: Komponente für die Darstellung des Anwendungsstarters
- **LauncherModel**: Komponente für die Datenhaltung des Anwendungsstarters
- **LauncherController**: Komponente für die Steuerung des Anwendungsstarters
- **LauncherSearch**: Komponente für die Suche im Anwendungsstarter
- **LauncherCategory**: Komponente für die Kategorisierung im Anwendungsstarter
- **LauncherItem**: Komponente für die Darstellung einer Anwendung im Anwendungsstarter
- **LauncherLayout**: Komponente für das Layout des Anwendungsstarters
- **LauncherAnimation**: Komponente für Animationen im Anwendungsstarter
- **LauncherTheme**: Komponente für das Theming des Anwendungsstarters

## 3. Anforderungen

### 3.1 Funktionale Anforderungen

1. Das Modul MUSS Mechanismen zur Darstellung von Anwendungen bereitstellen.
2. Das Modul MUSS Mechanismen zum Starten von Anwendungen bereitstellen.
3. Das Modul MUSS Mechanismen zur Suche nach Anwendungen bereitstellen.
4. Das Modul MUSS Mechanismen zur Kategorisierung von Anwendungen bereitstellen.
5. Das Modul MUSS Mechanismen zur Verwaltung von Favoriten bereitstellen.
6. Das Modul MUSS Mechanismen zur Anzeige kürzlich verwendeter Anwendungen bereitstellen.
7. Das Modul MUSS Mechanismen zur Anzeige häufig verwendeter Anwendungen bereitstellen.
8. Das Modul MUSS Mechanismen zur Anpassung des Layouts bereitstellen.
9. Das Modul MUSS Mechanismen zur Anpassung des Erscheinungsbilds bereitstellen.
10. Das Modul MUSS Mechanismen zur Integration mit dem Anwendungsmanager bereitstellen.
11. Das Modul MUSS Mechanismen zur Integration mit dem Fenstermanager bereitstellen.
12. Das Modul MUSS Mechanismen zur Integration mit dem Benachrichtigungssystem bereitstellen.
13. Das Modul MUSS Mechanismen zur Integration mit dem Theming-System bereitstellen.
14. Das Modul MUSS Mechanismen zur Integration mit dem Einstellungssystem bereitstellen.

### 3.2 Nicht-funktionale Anforderungen

1. Das Modul MUSS effizient mit Ressourcen umgehen.
2. Das Modul MUSS thread-safe sein.
3. Das Modul MUSS eine klare und konsistente API bereitstellen.
4. Das Modul MUSS gut dokumentiert sein.
5. Das Modul MUSS leicht erweiterbar sein.
6. Das Modul MUSS robust gegen Fehleingaben sein.
7. Das Modul MUSS minimale externe Abhängigkeiten haben.
8. Das Modul MUSS eine hohe Performance bieten.
9. Das Modul MUSS eine geringe Latenz beim Öffnen und Navigieren bieten.
10. Das Modul MUSS eine hohe Zuverlässigkeit bieten.

## 4. Architektur

### 4.1 Komponentenstruktur

Das Anwendungsstarter-Modul besteht aus den folgenden Komponenten:

1. **LauncherManager** (`launcher_manager.rs`): Zentrale Komponente für die Verwaltung des Anwendungsstarters
2. **LauncherView** (`launcher_view.rs`): Komponente für die Darstellung des Anwendungsstarters
3. **LauncherModel** (`launcher_model.rs`): Komponente für die Datenhaltung des Anwendungsstarters
4. **LauncherController** (`launcher_controller.rs`): Komponente für die Steuerung des Anwendungsstarters
5. **LauncherSearch** (`launcher_search.rs`): Komponente für die Suche im Anwendungsstarter
6. **LauncherCategory** (`launcher_category.rs`): Komponente für die Kategorisierung im Anwendungsstarter
7. **LauncherItem** (`launcher_item.rs`): Komponente für die Darstellung einer Anwendung im Anwendungsstarter
8. **LauncherLayout** (`launcher_layout.rs`): Komponente für das Layout des Anwendungsstarters
9. **LauncherAnimation** (`launcher_animation.rs`): Komponente für Animationen im Anwendungsstarter
10. **LauncherTheme** (`launcher_theme.rs`): Komponente für das Theming des Anwendungsstarters
11. **LauncherConfig** (`launcher_config.rs`): Komponente für die Konfiguration des Anwendungsstarters
12. **LauncherEvent** (`launcher_event.rs`): Komponente für die Ereignisverwaltung des Anwendungsstarters
13. **LauncherDrag** (`launcher_drag.rs`): Komponente für Drag-and-Drop-Operationen im Anwendungsstarter
14. **LauncherShortcut** (`launcher_shortcut.rs`): Komponente für Tastaturkürzel im Anwendungsstarter

### 4.2 Abhängigkeiten

Das Anwendungsstarter-Modul hat folgende Abhängigkeiten:

1. **Interne Abhängigkeiten**:
   - `core::errors`: Für die Fehlerbehandlung
   - `core::config`: Für die Konfiguration
   - `core::logging`: Für das Logging
   - `domain::application`: Für die Anwendungsverwaltung
   - `domain::settings`: Für die Einstellungsverwaltung
   - `domain::theming`: Für das Theming
   - `system::windowmanager`: Für die Fensterverwaltung
   - `system::notification`: Für die Benachrichtigungsverwaltung
   - `ui::widget`: Für die Widget-Bibliothek

2. **Externe Abhängigkeiten**:
   - `gtk4`: Für die GUI-Komponenten
   - `cairo`: Für die Grafikausgabe
   - `pango`: Für die Textdarstellung
   - `gdk4`: Für die Ereignisverwaltung
   - `gio`: Für die Integration mit GIO
   - `serde`: Für die Serialisierung und Deserialisierung
   - `json`: Für die JSON-Verarbeitung

## 5. Schnittstellen

### 5.1 LauncherManager

```
SCHNITTSTELLE: ui::launcher::LauncherManager
BESCHREIBUNG: Zentrale Komponente für die Verwaltung des Anwendungsstarters
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue LauncherManager-Instanz
    PARAMETER:
      - NAME: config
        TYP: LauncherConfig
        BESCHREIBUNG: Konfiguration für den LauncherManager
        EINSCHRÄNKUNGEN: Muss eine gültige LauncherConfig sein
    RÜCKGABETYP: Result<LauncherManager, LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des LauncherManagers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue LauncherManager-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des LauncherManagers auftritt
  
  - NAME: initialize
    BESCHREIBUNG: Initialisiert den LauncherManager
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler bei der Initialisierung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der LauncherManager wird initialisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Initialisierung auftritt
  
  - NAME: shutdown
    BESCHREIBUNG: Fährt den LauncherManager herunter
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Herunterfahren auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der LauncherManager wird heruntergefahren
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Herunterfahren auftritt
  
  - NAME: show
    BESCHREIBUNG: Zeigt den Anwendungsstarter an
    PARAMETER:
      - NAME: mode
        TYP: LauncherMode
        BESCHREIBUNG: Modus des Anwendungsstarters
        EINSCHRÄNKUNGEN: Muss ein gültiger LauncherMode sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Anzeigen des Anwendungsstarters auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Anwendungsstarter wird angezeigt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Anzeigen des Anwendungsstarters auftritt
  
  - NAME: hide
    BESCHREIBUNG: Verbirgt den Anwendungsstarter
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Verbergen des Anwendungsstarters auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Anwendungsstarter wird verborgen
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Verbergen des Anwendungsstarters auftritt
  
  - NAME: is_visible
    BESCHREIBUNG: Prüft, ob der Anwendungsstarter sichtbar ist
    PARAMETER: Keine
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn der Anwendungsstarter sichtbar ist
      - false wird zurückgegeben, wenn der Anwendungsstarter nicht sichtbar ist
  
  - NAME: toggle
    BESCHREIBUNG: Wechselt die Sichtbarkeit des Anwendungsstarters
    PARAMETER:
      - NAME: mode
        TYP: Option<LauncherMode>
        BESCHREIBUNG: Modus des Anwendungsstarters
        EINSCHRÄNKUNGEN: Wenn vorhanden, muss ein gültiger LauncherMode sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Wechseln der Sichtbarkeit auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Wenn der Anwendungsstarter sichtbar war, wird er verborgen
      - Wenn der Anwendungsstarter nicht sichtbar war, wird er angezeigt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Wechseln der Sichtbarkeit auftritt
  
  - NAME: get_view
    BESCHREIBUNG: Gibt die Ansicht des Anwendungsstarters zurück
    PARAMETER: Keine
    RÜCKGABETYP: &LauncherView
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Ansicht des Anwendungsstarters wird zurückgegeben
  
  - NAME: get_model
    BESCHREIBUNG: Gibt das Modell des Anwendungsstarters zurück
    PARAMETER: Keine
    RÜCKGABETYP: &LauncherModel
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Modell des Anwendungsstarters wird zurückgegeben
  
  - NAME: get_controller
    BESCHREIBUNG: Gibt den Controller des Anwendungsstarters zurück
    PARAMETER: Keine
    RÜCKGABETYP: &LauncherController
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Controller des Anwendungsstarters wird zurückgegeben
  
  - NAME: get_config
    BESCHREIBUNG: Gibt die Konfiguration des Anwendungsstarters zurück
    PARAMETER: Keine
    RÜCKGABETYP: &LauncherConfig
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Konfiguration des Anwendungsstarters wird zurückgegeben
  
  - NAME: set_config
    BESCHREIBUNG: Setzt die Konfiguration des Anwendungsstarters
    PARAMETER:
      - NAME: config
        TYP: LauncherConfig
        BESCHREIBUNG: Konfiguration
        EINSCHRÄNKUNGEN: Muss eine gültige LauncherConfig sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Setzen der Konfiguration auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Konfiguration des Anwendungsstarters wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen der Konfiguration auftritt
  
  - NAME: refresh
    BESCHREIBUNG: Aktualisiert den Anwendungsstarter
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler bei der Aktualisierung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Anwendungsstarter wird aktualisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Aktualisierung auftritt
  
  - NAME: search
    BESCHREIBUNG: Sucht nach Anwendungen
    PARAMETER:
      - NAME: query
        TYP: &str
        BESCHREIBUNG: Suchanfrage
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<Vec<LauncherItem>, LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler bei der Suche auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste von Anwendungen, die der Suchanfrage entsprechen, wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Suche auftritt
  
  - NAME: launch_application
    BESCHREIBUNG: Startet eine Anwendung
    PARAMETER:
      - NAME: item
        TYP: &LauncherItem
        BESCHREIBUNG: Anwendungselement
        EINSCHRÄNKUNGEN: Muss ein gültiges LauncherItem sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Starten der Anwendung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird gestartet
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Starten der Anwendung auftritt
  
  - NAME: add_to_favorites
    BESCHREIBUNG: Fügt eine Anwendung zu den Favoriten hinzu
    PARAMETER:
      - NAME: item
        TYP: &LauncherItem
        BESCHREIBUNG: Anwendungselement
        EINSCHRÄNKUNGEN: Muss ein gültiges LauncherItem sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Hinzufügen zu den Favoriten auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird zu den Favoriten hinzugefügt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Hinzufügen zu den Favoriten auftritt
  
  - NAME: remove_from_favorites
    BESCHREIBUNG: Entfernt eine Anwendung aus den Favoriten
    PARAMETER:
      - NAME: item
        TYP: &LauncherItem
        BESCHREIBUNG: Anwendungselement
        EINSCHRÄNKUNGEN: Muss ein gültiges LauncherItem sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Entfernen aus den Favoriten auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird aus den Favoriten entfernt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Entfernen aus den Favoriten auftritt
  
  - NAME: is_favorite
    BESCHREIBUNG: Prüft, ob eine Anwendung zu den Favoriten gehört
    PARAMETER:
      - NAME: item
        TYP: &LauncherItem
        BESCHREIBUNG: Anwendungselement
        EINSCHRÄNKUNGEN: Muss ein gültiges LauncherItem sein
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn die Anwendung zu den Favoriten gehört
      - false wird zurückgegeben, wenn die Anwendung nicht zu den Favoriten gehört
  
  - NAME: get_favorites
    BESCHREIBUNG: Gibt die Favoriten zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<LauncherItem>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste der Favoriten wird zurückgegeben
  
  - NAME: get_recent_applications
    BESCHREIBUNG: Gibt kürzlich verwendete Anwendungen zurück
    PARAMETER:
      - NAME: count
        TYP: usize
        BESCHREIBUNG: Anzahl der zurückzugebenden Anwendungen
        EINSCHRÄNKUNGEN: Muss größer als 0 sein
    RÜCKGABETYP: Vec<LauncherItem>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste kürzlich verwendeter Anwendungen wird zurückgegeben
  
  - NAME: get_frequent_applications
    BESCHREIBUNG: Gibt häufig verwendete Anwendungen zurück
    PARAMETER:
      - NAME: count
        TYP: usize
        BESCHREIBUNG: Anzahl der zurückzugebenden Anwendungen
        EINSCHRÄNKUNGEN: Muss größer als 0 sein
    RÜCKGABETYP: Vec<LauncherItem>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste häufig verwendeter Anwendungen wird zurückgegeben
  
  - NAME: get_categories
    BESCHREIBUNG: Gibt die Kategorien zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<LauncherCategory>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste der Kategorien wird zurückgegeben
  
  - NAME: get_applications_by_category
    BESCHREIBUNG: Gibt Anwendungen nach Kategorie zurück
    PARAMETER:
      - NAME: category
        TYP: &LauncherCategory
        BESCHREIBUNG: Kategorie
        EINSCHRÄNKUNGEN: Muss eine gültige LauncherCategory sein
    RÜCKGABETYP: Vec<LauncherItem>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste von Anwendungen in der angegebenen Kategorie wird zurückgegeben
  
  - NAME: register_event_listener
    BESCHREIBUNG: Registriert einen Listener für Ereignisse
    PARAMETER:
      - NAME: listener
        TYP: Box<dyn Fn(&LauncherEvent) -> bool + Send + Sync + 'static>
        BESCHREIBUNG: Listener-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: ListenerId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird registriert und eine ListenerId wird zurückgegeben
  
  - NAME: unregister_event_listener
    BESCHREIBUNG: Entfernt einen Listener für Ereignisse
    PARAMETER:
      - NAME: id
        TYP: ListenerId
        BESCHREIBUNG: ID des Listeners
        EINSCHRÄNKUNGEN: Muss eine gültige ListenerId sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn der Listener nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Listener nicht gefunden wird
```

### 5.2 LauncherView

```
SCHNITTSTELLE: ui::launcher::LauncherView
BESCHREIBUNG: Komponente für die Darstellung des Anwendungsstarters
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue LauncherView-Instanz
    PARAMETER:
      - NAME: model
        TYP: &LauncherModel
        BESCHREIBUNG: Modell für die Ansicht
        EINSCHRÄNKUNGEN: Muss ein gültiges LauncherModel sein
      - NAME: config
        TYP: &LauncherConfig
        BESCHREIBUNG: Konfiguration für die Ansicht
        EINSCHRÄNKUNGEN: Muss eine gültige LauncherConfig sein
    RÜCKGABETYP: Result<LauncherView, LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung der LauncherView auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue LauncherView-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung der LauncherView auftritt
  
  - NAME: initialize
    BESCHREIBUNG: Initialisiert die LauncherView
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler bei der Initialisierung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die LauncherView wird initialisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Initialisierung auftritt
  
  - NAME: show
    BESCHREIBUNG: Zeigt die Ansicht an
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Anzeigen der Ansicht auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Ansicht wird angezeigt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Anzeigen der Ansicht auftritt
  
  - NAME: hide
    BESCHREIBUNG: Verbirgt die Ansicht
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Verbergen der Ansicht auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Ansicht wird verborgen
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Verbergen der Ansicht auftritt
  
  - NAME: is_visible
    BESCHREIBUNG: Prüft, ob die Ansicht sichtbar ist
    PARAMETER: Keine
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn die Ansicht sichtbar ist
      - false wird zurückgegeben, wenn die Ansicht nicht sichtbar ist
  
  - NAME: refresh
    BESCHREIBUNG: Aktualisiert die Ansicht
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler bei der Aktualisierung der Ansicht auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Ansicht wird aktualisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Aktualisierung der Ansicht auftritt
  
  - NAME: set_mode
    BESCHREIBUNG: Setzt den Modus der Ansicht
    PARAMETER:
      - NAME: mode
        TYP: LauncherMode
        BESCHREIBUNG: Modus
        EINSCHRÄNKUNGEN: Muss ein gültiger LauncherMode sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Setzen des Modus auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Modus der Ansicht wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen des Modus auftritt
  
  - NAME: get_mode
    BESCHREIBUNG: Gibt den Modus der Ansicht zurück
    PARAMETER: Keine
    RÜCKGABETYP: LauncherMode
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Modus der Ansicht wird zurückgegeben
  
  - NAME: set_layout
    BESCHREIBUNG: Setzt das Layout der Ansicht
    PARAMETER:
      - NAME: layout
        TYP: LauncherLayout
        BESCHREIBUNG: Layout
        EINSCHRÄNKUNGEN: Muss ein gültiges LauncherLayout sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Setzen des Layouts auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Layout der Ansicht wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen des Layouts auftritt
  
  - NAME: get_layout
    BESCHREIBUNG: Gibt das Layout der Ansicht zurück
    PARAMETER: Keine
    RÜCKGABETYP: &LauncherLayout
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Layout der Ansicht wird zurückgegeben
  
  - NAME: set_theme
    BESCHREIBUNG: Setzt das Theme der Ansicht
    PARAMETER:
      - NAME: theme
        TYP: LauncherTheme
        BESCHREIBUNG: Theme
        EINSCHRÄNKUNGEN: Muss ein gültiges LauncherTheme sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Setzen des Themes auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Theme der Ansicht wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen des Themes auftritt
  
  - NAME: get_theme
    BESCHREIBUNG: Gibt das Theme der Ansicht zurück
    PARAMETER: Keine
    RÜCKGABETYP: &LauncherTheme
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Theme der Ansicht wird zurückgegeben
  
  - NAME: get_widget
    BESCHREIBUNG: Gibt das Widget der Ansicht zurück
    PARAMETER: Keine
    RÜCKGABETYP: &gtk4::Widget
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Widget der Ansicht wird zurückgegeben
  
  - NAME: set_search_query
    BESCHREIBUNG: Setzt die Suchanfrage
    PARAMETER:
      - NAME: query
        TYP: &str
        BESCHREIBUNG: Suchanfrage
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler beim Setzen der Suchanfrage auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Suchanfrage wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen der Suchanfrage auftritt
  
  - NAME: get_search_query
    BESCHREIBUNG: Gibt die Suchanfrage zurück
    PARAMETER: Keine
    RÜCKGABETYP: String
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Suchanfrage wird zurückgegeben
  
  - NAME: select_category
    BESCHREIBUNG: Wählt eine Kategorie aus
    PARAMETER:
      - NAME: category
        TYP: &LauncherCategory
        BESCHREIBUNG: Kategorie
        EINSCHRÄNKUNGEN: Muss eine gültige LauncherCategory sein
    RÜCKGABETYP: Result<(), LauncherError>
    FEHLER:
      - TYP: LauncherError
        BEDINGUNG: Wenn ein Fehler bei der Auswahl der Kategorie auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Kategorie wird ausgewählt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Auswahl der Kategorie auftritt
  
  - NAME: get_selected_category
    BESCHREIBUNG: Gibt die ausgewählte Kategorie zurück
    PARAMETER: Keine
    RÜCKGABETYP: Option<LauncherCategory>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die ausgewählte Kategorie wird zurückgegeben, wenn eine Kategorie ausgewählt ist
      - None wird zurückgegeben, wenn keine Kategorie ausgewählt ist
```

## 6. Datenmodell (Teil 1)

### 6.1 LauncherConfig

```
ENTITÄT: LauncherConfig
BESCHREIBUNG: Konfiguration für den Anwendungsstarter
ATTRIBUTE:
  - NAME: default_mode
    TYP: LauncherMode
    BESCHREIBUNG: Standardmodus des Anwendungsstarters
    WERTEBEREICH: Gültige LauncherMode
    STANDARDWERT: LauncherMode::FullScreen
  - NAME: default_layout
    TYP: LauncherLayoutType
    BESCHREIBUNG: Standardlayout des Anwendungsstarters
    WERTEBEREICH: Gültige LauncherLayoutType
    STANDARDWERT: LauncherLayoutType::Grid
  - NAME: icon_size
    TYP: u32
    BESCHREIBUNG: Größe der Icons in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 64
  - NAME: show_names
    TYP: bool
    BESCHREIBUNG: Ob Anwendungsnamen angezeigt werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: show_categories
    TYP: bool
    BESCHREIBUNG: Ob Kategorien angezeigt werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: show_search
    TYP: bool
    BESCHREIBUNG: Ob die Suche angezeigt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: show_favorites
    TYP: bool
    BESCHREIBUNG: Ob Favoriten angezeigt werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: show_recent
    TYP: bool
    BESCHREIBUNG: Ob kürzlich verwendete Anwendungen angezeigt werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: show_frequent
    TYP: bool
    BESCHREIBUNG: Ob häufig verwendete Anwendungen angezeigt werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: recent_count
    TYP: usize
    BESCHREIBUNG: Anzahl der anzuzeigenden kürzlich verwendeten Anwendungen
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: frequent_count
    TYP: usize
    BESCHREIBUNG: Anzahl der anzuzeigenden häufig verwendeten Anwendungen
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: animation_enabled
    TYP: bool
    BESCHREIBUNG: Ob Animationen aktiviert sind
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: animation_duration
    TYP: u32
    BESCHREIBUNG: Dauer der Animationen in Millisekunden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 250
  - NAME: search_as_you_type
    TYP: bool
    BESCHREIBUNG: Ob die Suche während der Eingabe aktualisiert werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: search_delay
    TYP: u32
    BESCHREIBUNG: Verzögerung für die Suche in Millisekunden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 100
  - NAME: keyboard_navigation
    TYP: bool
    BESCHREIBUNG: Ob die Tastaturnavigation aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: mouse_navigation
    TYP: bool
    BESCHREIBUNG: Ob die Mausnavigation aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: touch_navigation
    TYP: bool
    BESCHREIBUNG: Ob die Touch-Navigation aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: position
    TYP: LauncherPosition
    BESCHREIBUNG: Position des Anwendungsstarters
    WERTEBEREICH: Gültige LauncherPosition
    STANDARDWERT: LauncherPosition::Center
  - NAME: background_opacity
    TYP: f32
    BESCHREIBUNG: Deckkraft des Hintergrunds
    WERTEBEREICH: [0.0, 1.0]
    STANDARDWERT: 0.9
  - NAME: background_blur
    TYP: bool
    BESCHREIBUNG: Ob der Hintergrund unscharf dargestellt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: background_blur_radius
    TYP: u32
    BESCHREIBUNG: Radius der Unschärfe
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: border_radius
    TYP: u32
    BESCHREIBUNG: Radius der abgerundeten Ecken in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: spacing
    TYP: u32
    BESCHREIBUNG: Abstand zwischen Elementen in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: padding
    TYP: u32
    BESCHREIBUNG: Innenabstand in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 20
  - NAME: custom_css
    TYP: Option<String>
    BESCHREIBUNG: Benutzerdefiniertes CSS
    WERTEBEREICH: Gültiges CSS oder None
    STANDARDWERT: None
INVARIANTEN:
  - icon_size muss größer als 0 sein
  - recent_count muss größer als 0 sein
  - frequent_count muss größer als 0 sein
  - animation_duration muss größer als 0 sein
  - search_delay muss größer als 0 sein
  - background_opacity muss im Bereich [0.0, 1.0] liegen
  - background_blur_radius muss größer als 0 sein
  - border_radius muss größer als 0 sein
  - spacing muss größer als 0 sein
  - padding muss größer als 0 sein
```

### 6.2 LauncherMode

```
ENTITÄT: LauncherMode
BESCHREIBUNG: Modus des Anwendungsstarters
ATTRIBUTE:
  - NAME: mode
    TYP: Enum
    BESCHREIBUNG: Modus
    WERTEBEREICH: {
      FullScreen,
      Windowed,
      Compact,
      Dock,
      Custom
    }
    STANDARDWERT: FullScreen
INVARIANTEN:
  - Keine
```

### 6.3 LauncherLayoutType

```
ENTITÄT: LauncherLayoutType
BESCHREIBUNG: Typ des Layouts des Anwendungsstarters
ATTRIBUTE:
  - NAME: layout_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      Grid,
      List,
      Flow,
      Carousel,
      Custom
    }
    STANDARDWERT: Grid
INVARIANTEN:
  - Keine
```

### 6.4 LauncherPosition

```
ENTITÄT: LauncherPosition
BESCHREIBUNG: Position des Anwendungsstarters
ATTRIBUTE:
  - NAME: position
    TYP: Enum
    BESCHREIBUNG: Position
    WERTEBEREICH: {
      Center,
      TopLeft,
      TopCenter,
      TopRight,
      MiddleLeft,
      MiddleRight,
      BottomLeft,
      BottomCenter,
      BottomRight,
      Custom { x: i32, y: i32 }
    }
    STANDARDWERT: Center
INVARIANTEN:
  - Keine
```

### 6.5 LauncherItem

```
ENTITÄT: LauncherItem
BESCHREIBUNG: Element im Anwendungsstarter
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Elements
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name des Elements
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: description
    TYP: Option<String>
    BESCHREIBUNG: Beschreibung des Elements
    WERTEBEREICH: Zeichenkette oder None
    STANDARDWERT: None
  - NAME: icon
    TYP: Option<String>
    BESCHREIBUNG: Icon des Elements
    WERTEBEREICH: Gültiger Icon-Name oder Pfad oder None
    STANDARDWERT: None
  - NAME: categories
    TYP: Vec<LauncherCategory>
    BESCHREIBUNG: Kategorien des Elements
    WERTEBEREICH: Gültige LauncherCategory-Werte
    STANDARDWERT: Leerer Vec
  - NAME: keywords
    TYP: Vec<String>
    BESCHREIBUNG: Schlüsselwörter für das Element
    WERTEBEREICH: Zeichenketten
    STANDARDWERT: Leerer Vec
  - NAME: application_id
    TYP: String
    BESCHREIBUNG: ID der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: is_favorite
    TYP: bool
    BESCHREIBUNG: Ob das Element ein Favorit ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: last_used
    TYP: Option<SystemTime>
    BESCHREIBUNG: Zeitpunkt der letzten Verwendung
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
  - NAME: use_count
    TYP: u32
    BESCHREIBUNG: Anzahl der Verwendungen
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
  - NAME: custom_data
    TYP: HashMap<String, String>
    BESCHREIBUNG: Benutzerdefinierte Daten
    WERTEBEREICH: Gültige Schlüssel-Wert-Paare
    STANDARDWERT: Leere HashMap
INVARIANTEN:
  - id darf nicht leer sein
  - name darf nicht leer sein
  - application_id darf nicht leer sein
```

### 6.6 LauncherCategory

```
ENTITÄT: LauncherCategory
BESCHREIBUNG: Kategorie im Anwendungsstarter
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID der Kategorie
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name der Kategorie
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: description
    TYP: Option<String>
    BESCHREIBUNG: Beschreibung der Kategorie
    WERTEBEREICH: Zeichenkette oder None
    STANDARDWERT: None
  - NAME: icon
    TYP: Option<String>
    BESCHREIBUNG: Icon der Kategorie
    WERTEBEREICH: Gültiger Icon-Name oder Pfad oder None
    STANDARDWERT: None
  - NAME: parent
    TYP: Option<String>
    BESCHREIBUNG: ID der übergeordneten Kategorie
    WERTEBEREICH: Gültige Kategorie-ID oder None
    STANDARDWERT: None
  - NAME: order
    TYP: i32
    BESCHREIBUNG: Reihenfolge der Kategorie
    WERTEBEREICH: Ganzzahlen
    STANDARDWERT: 0
INVARIANTEN:
  - id darf nicht leer sein
  - name darf nicht leer sein
```

### 6.7 LauncherLayout

```
ENTITÄT: LauncherLayout
BESCHREIBUNG: Layout des Anwendungsstarters
ATTRIBUTE:
  - NAME: layout_type
    TYP: LauncherLayoutType
    BESCHREIBUNG: Typ des Layouts
    WERTEBEREICH: Gültige LauncherLayoutType
    STANDARDWERT: LauncherLayoutType::Grid
  - NAME: columns
    TYP: u32
    BESCHREIBUNG: Anzahl der Spalten
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 6
  - NAME: rows
    TYP: u32
    BESCHREIBUNG: Anzahl der Zeilen
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 4
  - NAME: spacing
    TYP: u32
    BESCHREIBUNG: Abstand zwischen Elementen in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: padding
    TYP: u32
    BESCHREIBUNG: Innenabstand in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 20
  - NAME: alignment
    TYP: LauncherAlignment
    BESCHREIBUNG: Ausrichtung der Elemente
    WERTEBEREICH: Gültige LauncherAlignment
    STANDARDWERT: LauncherAlignment::Center
  - NAME: orientation
    TYP: LauncherOrientation
    BESCHREIBUNG: Orientierung des Layouts
    WERTEBEREICH: Gültige LauncherOrientation
    STANDARDWERT: LauncherOrientation::Vertical
  - NAME: wrap
    TYP: bool
    BESCHREIBUNG: Ob Elemente umgebrochen werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: fixed_size
    TYP: bool
    BESCHREIBUNG: Ob Elemente eine feste Größe haben sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: item_width
    TYP: u32
    BESCHREIBUNG: Breite der Elemente in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 100
  - NAME: item_height
    TYP: u32
    BESCHREIBUNG: Höhe der Elemente in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 100
INVARIANTEN:
  - columns muss größer als 0 sein
  - rows muss größer als 0 sein
  - spacing muss größer als 0 sein
  - padding muss größer als 0 sein
  - item_width muss größer als 0 sein
  - item_height muss größer als 0 sein
```

### 6.8 LauncherAlignment

```
ENTITÄT: LauncherAlignment
BESCHREIBUNG: Ausrichtung der Elemente im Anwendungsstarter
ATTRIBUTE:
  - NAME: alignment
    TYP: Enum
    BESCHREIBUNG: Ausrichtung
    WERTEBEREICH: {
      Start,
      Center,
      End,
      SpaceBetween,
      SpaceAround,
      SpaceEvenly
    }
    STANDARDWERT: Center
INVARIANTEN:
  - Keine
```

### 6.9 LauncherOrientation

```
ENTITÄT: LauncherOrientation
BESCHREIBUNG: Orientierung des Layouts im Anwendungsstarter
ATTRIBUTE:
  - NAME: orientation
    TYP: Enum
    BESCHREIBUNG: Orientierung
    WERTEBEREICH: {
      Horizontal,
      Vertical
    }
    STANDARDWERT: Vertical
INVARIANTEN:
  - Keine
```

### 6.10 LauncherTheme

```
ENTITÄT: LauncherTheme
BESCHREIBUNG: Theme des Anwendungsstarters
ATTRIBUTE:
  - NAME: background_color
    TYP: Option<Color>
    BESCHREIBUNG: Hintergrundfarbe
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
  - NAME: text_color
    TYP: Option<Color>
    BESCHREIBUNG: Textfarbe
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
  - NAME: selected_color
    TYP: Option<Color>
    BESCHREIBUNG: Farbe für ausgewählte Elemente
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
  - NAME: hover_color
    TYP: Option<Color>
    BESCHREIBUNG: Farbe für Hover-Effekte
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
  - NAME: border_color
    TYP: Option<Color>
    BESCHREIBUNG: Rahmenfarbe
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
  - NAME: border_width
    TYP: u32
    BESCHREIBUNG: Rahmenbreite in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 1
  - NAME: border_radius
    TYP: u32
    BESCHREIBUNG: Radius der abgerundeten Ecken in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: shadow
    TYP: bool
    BESCHREIBUNG: Ob ein Schatten angezeigt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: shadow_color
    TYP: Option<Color>
    BESCHREIBUNG: Schattenfarbe
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
  - NAME: shadow_radius
    TYP: u32
    BESCHREIBUNG: Radius des Schattens in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: shadow_offset_x
    TYP: i32
    BESCHREIBUNG: X-Offset des Schattens in Pixeln
    WERTEBEREICH: Ganzzahlen
    STANDARDWERT: 0
  - NAME: shadow_offset_y
    TYP: i32
    BESCHREIBUNG: Y-Offset des Schattens in Pixeln
    WERTEBEREICH: Ganzzahlen
    STANDARDWERT: 5
  - NAME: font_family
    TYP: Option<String>
    BESCHREIBUNG: Schriftfamilie
    WERTEBEREICH: Gültige Schriftfamilie oder None
    STANDARDWERT: None
  - NAME: font_size
    TYP: u32
    BESCHREIBUNG: Schriftgröße in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 12
  - NAME: font_weight
    TYP: FontWeight
    BESCHREIBUNG: Schriftstärke
    WERTEBEREICH: Gültige FontWeight
    STANDARDWERT: FontWeight::Normal
  - NAME: icon_effect
    TYP: IconEffect
    BESCHREIBUNG: Effekt für Icons
    WERTEBEREICH: Gültige IconEffect
    STANDARDWERT: IconEffect::None
INVARIANTEN:
  - border_width muss größer als 0 sein
  - border_radius muss größer als 0 sein
  - shadow_radius muss größer als 0 sein
  - font_size muss größer als 0 sein
```
