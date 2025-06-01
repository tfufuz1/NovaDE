# SPEC-MODULE-DOMAIN-APPLICATION-v1.0.0: NovaDE Anwendungsmanager-Modul (Teil 1)

```
SPEZIFIKATION: SPEC-MODULE-DOMAIN-APPLICATION-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0, SPEC-LAYER-CORE-v1.0.0, SPEC-LAYER-DOMAIN-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Diese Spezifikation definiert das Anwendungsmanager-Modul (`domain::application`) der NovaDE-Domänenschicht. Das Modul stellt die grundlegende Infrastruktur für die Verwaltung von Anwendungen bereit und definiert die Mechanismen zum Starten, Beenden, Überwachen und Verwalten von Anwendungen. Der Geltungsbereich umfasst alle Komponenten und Schnittstellen des Anwendungsmanager-Moduls sowie deren Interaktionen mit anderen Modulen.

## 2. Definitionen

### 2.1 Allgemeine Begriffe

- **Anwendung**: Ausführbares Programm, das vom Benutzer gestartet werden kann
- **Anwendungspaket**: Sammlung von Dateien, die eine Anwendung bilden
- **Anwendungsstarter**: Mechanismus zum Starten einer Anwendung
- **Anwendungskategorie**: Thematische Gruppierung von Anwendungen
- **Anwendungszustand**: Aktueller Status einer Anwendung (z.B. gestartet, beendet)
- **Anwendungsfenster**: Fenster, das zu einer Anwendung gehört
- **Anwendungssitzung**: Laufende Instanz einer Anwendung
- **Anwendungspräferenz**: Benutzerspezifische Einstellung für eine Anwendung
- **Standardanwendung**: Anwendung, die standardmäßig für bestimmte Dateitypen oder Aktionen verwendet wird
- **Anwendungsmenü**: Menü, das verfügbare Anwendungen anzeigt

### 2.2 Modulspezifische Begriffe

- **ApplicationManager**: Zentrale Komponente für die Verwaltung von Anwendungen
- **ApplicationInfo**: Informationen über eine Anwendung
- **ApplicationLauncher**: Komponente zum Starten von Anwendungen
- **ApplicationMonitor**: Komponente zur Überwachung von Anwendungen
- **ApplicationCategory**: Kategorie für Anwendungen
- **ApplicationState**: Zustand einer Anwendung
- **ApplicationSession**: Sitzung einer Anwendung
- **ApplicationPreference**: Präferenz für eine Anwendung
- **DefaultApplication**: Standardanwendung für bestimmte Dateitypen oder Aktionen
- **ApplicationMenu**: Menü für Anwendungen

## 3. Anforderungen

### 3.1 Funktionale Anforderungen

1. Das Modul MUSS Mechanismen zum Starten von Anwendungen bereitstellen.
2. Das Modul MUSS Mechanismen zum Beenden von Anwendungen bereitstellen.
3. Das Modul MUSS Mechanismen zur Überwachung von Anwendungszuständen bereitstellen.
4. Das Modul MUSS Mechanismen zur Verwaltung von Anwendungsinformationen bereitstellen.
5. Das Modul MUSS Mechanismen zur Kategorisierung von Anwendungen bereitstellen.
6. Das Modul MUSS Mechanismen zur Verwaltung von Anwendungspräferenzen bereitstellen.
7. Das Modul MUSS Mechanismen zur Verwaltung von Standardanwendungen bereitstellen.
8. Das Modul MUSS Mechanismen zur Generierung von Anwendungsmenüs bereitstellen.
9. Das Modul MUSS Mechanismen zur Verwaltung von Anwendungssitzungen bereitstellen.
10. Das Modul MUSS Mechanismen zur Integration mit dem Fenstermanager bereitstellen.
11. Das Modul MUSS Mechanismen zur Integration mit dem Benachrichtigungssystem bereitstellen.
12. Das Modul MUSS Mechanismen zur Integration mit dem Dateisystem bereitstellen.
13. Das Modul MUSS Mechanismen zur Verwaltung von Anwendungsberechtigungen bereitstellen.
14. Das Modul MUSS Mechanismen zur Verwaltung von Anwendungsressourcen bereitstellen.

### 3.2 Nicht-funktionale Anforderungen

1. Das Modul MUSS effizient mit Ressourcen umgehen.
2. Das Modul MUSS thread-safe sein.
3. Das Modul MUSS eine klare und konsistente API bereitstellen.
4. Das Modul MUSS gut dokumentiert sein.
5. Das Modul MUSS leicht erweiterbar sein.
6. Das Modul MUSS robust gegen Fehleingaben sein.
7. Das Modul MUSS minimale externe Abhängigkeiten haben.
8. Das Modul MUSS eine hohe Performance bieten.
9. Das Modul MUSS eine geringe Latenz beim Starten von Anwendungen bieten.
10. Das Modul MUSS eine hohe Zuverlässigkeit bieten.

## 4. Architektur

### 4.1 Komponentenstruktur

Das Anwendungsmanager-Modul besteht aus den folgenden Komponenten:

1. **ApplicationManager** (`application_manager.rs`): Zentrale Komponente für die Verwaltung von Anwendungen
2. **ApplicationInfo** (`application_info.rs`): Komponente für die Verwaltung von Anwendungsinformationen
3. **ApplicationLauncher** (`application_launcher.rs`): Komponente zum Starten von Anwendungen
4. **ApplicationMonitor** (`application_monitor.rs`): Komponente zur Überwachung von Anwendungen
5. **ApplicationCategory** (`application_category.rs`): Komponente für die Verwaltung von Anwendungskategorien
6. **ApplicationState** (`application_state.rs`): Komponente für die Verwaltung von Anwendungszuständen
7. **ApplicationSession** (`application_session.rs`): Komponente für die Verwaltung von Anwendungssitzungen
8. **ApplicationPreference** (`application_preference.rs`): Komponente für die Verwaltung von Anwendungspräferenzen
9. **DefaultApplication** (`default_application.rs`): Komponente für die Verwaltung von Standardanwendungen
10. **ApplicationMenu** (`application_menu.rs`): Komponente für die Generierung von Anwendungsmenüs
11. **ApplicationPermission** (`application_permission.rs`): Komponente für die Verwaltung von Anwendungsberechtigungen
12. **ApplicationResource** (`application_resource.rs`): Komponente für die Verwaltung von Anwendungsressourcen
13. **ApplicationConfig** (`application_config.rs`): Komponente für die Konfiguration des Anwendungsmanagers
14. **ApplicationEvent** (`application_event.rs`): Komponente für die Verwaltung von Anwendungsereignissen

### 4.2 Abhängigkeiten

Das Anwendungsmanager-Modul hat folgende Abhängigkeiten:

1. **Interne Abhängigkeiten**:
   - `core::errors`: Für die Fehlerbehandlung
   - `core::config`: Für die Konfiguration
   - `core::logging`: Für das Logging
   - `domain::settings`: Für die Einstellungsverwaltung
   - `system::process`: Für die Prozessverwaltung
   - `system::filesystem`: Für den Dateisystemzugriff
   - `system::windowmanager`: Für die Fensterverwaltung
   - `system::notification`: Für die Benachrichtigungsverwaltung

2. **Externe Abhängigkeiten**:
   - `freedesktop_entry_parser`: Für das Parsen von Desktop-Einträgen
   - `xdg`: Für den Zugriff auf XDG-Verzeichnisse
   - `gio`: Für die Integration mit GIO
   - `dbus`: Für die D-Bus-Integration
   - `serde`: Für die Serialisierung und Deserialisierung
   - `json`: Für die JSON-Verarbeitung

## 5. Schnittstellen

### 5.1 ApplicationManager

```
SCHNITTSTELLE: domain::application::ApplicationManager
BESCHREIBUNG: Zentrale Komponente für die Verwaltung von Anwendungen
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue ApplicationManager-Instanz
    PARAMETER:
      - NAME: config
        TYP: ApplicationConfig
        BESCHREIBUNG: Konfiguration für den ApplicationManager
        EINSCHRÄNKUNGEN: Muss eine gültige ApplicationConfig sein
    RÜCKGABETYP: Result<ApplicationManager, ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des ApplicationManagers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue ApplicationManager-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des ApplicationManagers auftritt
  
  - NAME: initialize
    BESCHREIBUNG: Initialisiert den ApplicationManager
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn ein Fehler bei der Initialisierung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der ApplicationManager wird initialisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Initialisierung auftritt
  
  - NAME: shutdown
    BESCHREIBUNG: Fährt den ApplicationManager herunter
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn ein Fehler beim Herunterfahren auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der ApplicationManager wird heruntergefahren
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Herunterfahren auftritt
  
  - NAME: get_applications
    BESCHREIBUNG: Gibt alle verfügbaren Anwendungen zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<ApplicationInfo>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste aller verfügbaren Anwendungen wird zurückgegeben
  
  - NAME: get_application_by_id
    BESCHREIBUNG: Gibt eine Anwendung anhand ihrer ID zurück
    PARAMETER:
      - NAME: id
        TYP: &str
        BESCHREIBUNG: ID der Anwendung
        EINSCHRÄNKUNGEN: Muss eine gültige Anwendungs-ID sein
    RÜCKGABETYP: Option<ApplicationInfo>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird zurückgegeben, wenn sie gefunden wird
      - None wird zurückgegeben, wenn die Anwendung nicht gefunden wird
  
  - NAME: get_applications_by_category
    BESCHREIBUNG: Gibt Anwendungen anhand ihrer Kategorie zurück
    PARAMETER:
      - NAME: category
        TYP: &ApplicationCategory
        BESCHREIBUNG: Kategorie
        EINSCHRÄNKUNGEN: Muss eine gültige ApplicationCategory sein
    RÜCKGABETYP: Vec<ApplicationInfo>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste von Anwendungen in der angegebenen Kategorie wird zurückgegeben
  
  - NAME: get_applications_by_name
    BESCHREIBUNG: Gibt Anwendungen anhand ihres Namens zurück
    PARAMETER:
      - NAME: name
        TYP: &str
        BESCHREIBUNG: Name oder Teil des Namens
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Vec<ApplicationInfo>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste von Anwendungen, deren Name den angegebenen Namen enthält, wird zurückgegeben
  
  - NAME: get_categories
    BESCHREIBUNG: Gibt alle verfügbaren Anwendungskategorien zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<ApplicationCategory>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste aller verfügbaren Anwendungskategorien wird zurückgegeben
  
  - NAME: launch_application
    BESCHREIBUNG: Startet eine Anwendung
    PARAMETER:
      - NAME: id
        TYP: &str
        BESCHREIBUNG: ID der Anwendung
        EINSCHRÄNKUNGEN: Muss eine gültige Anwendungs-ID sein
      - NAME: args
        TYP: Option<Vec<String>>
        BESCHREIBUNG: Argumente für die Anwendung
        EINSCHRÄNKUNGEN: Keine
      - NAME: env
        TYP: Option<HashMap<String, String>>
        BESCHREIBUNG: Umgebungsvariablen für die Anwendung
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<ApplicationSession, ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn die Anwendung nicht gefunden wird oder ein Fehler beim Starten auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird gestartet
      - Eine ApplicationSession wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn die Anwendung nicht gefunden wird oder ein Fehler beim Starten auftritt
  
  - NAME: launch_application_with_files
    BESCHREIBUNG: Startet eine Anwendung mit Dateien
    PARAMETER:
      - NAME: id
        TYP: &str
        BESCHREIBUNG: ID der Anwendung
        EINSCHRÄNKUNGEN: Muss eine gültige Anwendungs-ID sein
      - NAME: files
        TYP: Vec<&Path>
        BESCHREIBUNG: Dateien, die geöffnet werden sollen
        EINSCHRÄNKUNGEN: Müssen gültige Pfade sein
      - NAME: args
        TYP: Option<Vec<String>>
        BESCHREIBUNG: Argumente für die Anwendung
        EINSCHRÄNKUNGEN: Keine
      - NAME: env
        TYP: Option<HashMap<String, String>>
        BESCHREIBUNG: Umgebungsvariablen für die Anwendung
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<ApplicationSession, ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn die Anwendung nicht gefunden wird oder ein Fehler beim Starten auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird mit den angegebenen Dateien gestartet
      - Eine ApplicationSession wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn die Anwendung nicht gefunden wird oder ein Fehler beim Starten auftritt
  
  - NAME: terminate_application
    BESCHREIBUNG: Beendet eine Anwendung
    PARAMETER:
      - NAME: session
        TYP: &ApplicationSession
        BESCHREIBUNG: Sitzung der Anwendung
        EINSCHRÄNKUNGEN: Muss eine gültige ApplicationSession sein
      - NAME: force
        TYP: bool
        BESCHREIBUNG: Ob die Anwendung zwangsweise beendet werden soll
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn die Anwendung nicht beendet werden kann oder ein Fehler beim Beenden auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird beendet
      - Ein Fehler wird zurückgegeben, wenn die Anwendung nicht beendet werden kann oder ein Fehler beim Beenden auftritt
  
  - NAME: get_application_state
    BESCHREIBUNG: Gibt den Zustand einer Anwendung zurück
    PARAMETER:
      - NAME: session
        TYP: &ApplicationSession
        BESCHREIBUNG: Sitzung der Anwendung
        EINSCHRÄNKUNGEN: Muss eine gültige ApplicationSession sein
    RÜCKGABETYP: Result<ApplicationState, ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn die Anwendung nicht gefunden wird oder ein Fehler beim Abrufen des Zustands auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Zustand der Anwendung wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn die Anwendung nicht gefunden wird oder ein Fehler beim Abrufen des Zustands auftritt
  
  - NAME: get_running_applications
    BESCHREIBUNG: Gibt alle laufenden Anwendungen zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<(ApplicationInfo, ApplicationSession)>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste aller laufenden Anwendungen wird zurückgegeben
  
  - NAME: get_default_application_for_mime_type
    BESCHREIBUNG: Gibt die Standardanwendung für einen MIME-Typ zurück
    PARAMETER:
      - NAME: mime_type
        TYP: &str
        BESCHREIBUNG: MIME-Typ
        EINSCHRÄNKUNGEN: Muss ein gültiger MIME-Typ sein
    RÜCKGABETYP: Option<ApplicationInfo>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Standardanwendung für den angegebenen MIME-Typ wird zurückgegeben, wenn sie gefunden wird
      - None wird zurückgegeben, wenn keine Standardanwendung gefunden wird
  
  - NAME: set_default_application_for_mime_type
    BESCHREIBUNG: Setzt die Standardanwendung für einen MIME-Typ
    PARAMETER:
      - NAME: mime_type
        TYP: &str
        BESCHREIBUNG: MIME-Typ
        EINSCHRÄNKUNGEN: Muss ein gültiger MIME-Typ sein
      - NAME: application_id
        TYP: &str
        BESCHREIBUNG: ID der Anwendung
        EINSCHRÄNKUNGEN: Muss eine gültige Anwendungs-ID sein
    RÜCKGABETYP: Result<(), ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn die Anwendung nicht gefunden wird oder ein Fehler beim Setzen der Standardanwendung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Standardanwendung für den angegebenen MIME-Typ wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn die Anwendung nicht gefunden wird oder ein Fehler beim Setzen der Standardanwendung auftritt
  
  - NAME: get_application_preferences
    BESCHREIBUNG: Gibt die Präferenzen für eine Anwendung zurück
    PARAMETER:
      - NAME: id
        TYP: &str
        BESCHREIBUNG: ID der Anwendung
        EINSCHRÄNKUNGEN: Muss eine gültige Anwendungs-ID sein
    RÜCKGABETYP: Result<ApplicationPreference, ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn die Anwendung nicht gefunden wird oder ein Fehler beim Abrufen der Präferenzen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Präferenzen für die angegebene Anwendung werden zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn die Anwendung nicht gefunden wird oder ein Fehler beim Abrufen der Präferenzen auftritt
  
  - NAME: set_application_preferences
    BESCHREIBUNG: Setzt die Präferenzen für eine Anwendung
    PARAMETER:
      - NAME: id
        TYP: &str
        BESCHREIBUNG: ID der Anwendung
        EINSCHRÄNKUNGEN: Muss eine gültige Anwendungs-ID sein
      - NAME: preferences
        TYP: ApplicationPreference
        BESCHREIBUNG: Präferenzen
        EINSCHRÄNKUNGEN: Muss gültige ApplicationPreference sein
    RÜCKGABETYP: Result<(), ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn die Anwendung nicht gefunden wird oder ein Fehler beim Setzen der Präferenzen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Präferenzen für die angegebene Anwendung werden gesetzt
      - Ein Fehler wird zurückgegeben, wenn die Anwendung nicht gefunden wird oder ein Fehler beim Setzen der Präferenzen auftritt
  
  - NAME: generate_application_menu
    BESCHREIBUNG: Generiert ein Anwendungsmenü
    PARAMETER:
      - NAME: options
        TYP: ApplicationMenuOptions
        BESCHREIBUNG: Optionen für das Menü
        EINSCHRÄNKUNGEN: Muss gültige ApplicationMenuOptions sein
    RÜCKGABETYP: Result<ApplicationMenu, ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn ein Fehler bei der Generierung des Menüs auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein Anwendungsmenü wird generiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Generierung des Menüs auftritt
  
  - NAME: register_application_event_listener
    BESCHREIBUNG: Registriert einen Listener für Anwendungsereignisse
    PARAMETER:
      - NAME: listener
        TYP: Box<dyn Fn(&ApplicationEvent) -> bool + Send + Sync + 'static>
        BESCHREIBUNG: Listener-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: ListenerId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird registriert und eine ListenerId wird zurückgegeben
  
  - NAME: unregister_application_event_listener
    BESCHREIBUNG: Entfernt einen Listener für Anwendungsereignisse
    PARAMETER:
      - NAME: id
        TYP: ListenerId
        BESCHREIBUNG: ID des Listeners
        EINSCHRÄNKUNGEN: Muss eine gültige ListenerId sein
    RÜCKGABETYP: Result<(), ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn der Listener nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Listener nicht gefunden wird
```

### 5.2 ApplicationLauncher

```
SCHNITTSTELLE: domain::application::ApplicationLauncher
BESCHREIBUNG: Komponente zum Starten von Anwendungen
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue ApplicationLauncher-Instanz
    PARAMETER:
      - NAME: config
        TYP: ApplicationLauncherConfig
        BESCHREIBUNG: Konfiguration für den ApplicationLauncher
        EINSCHRÄNKUNGEN: Muss eine gültige ApplicationLauncherConfig sein
    RÜCKGABETYP: Result<ApplicationLauncher, ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des ApplicationLaunchers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue ApplicationLauncher-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des ApplicationLaunchers auftritt
  
  - NAME: launch
    BESCHREIBUNG: Startet eine Anwendung
    PARAMETER:
      - NAME: info
        TYP: &ApplicationInfo
        BESCHREIBUNG: Informationen über die Anwendung
        EINSCHRÄNKUNGEN: Muss gültige ApplicationInfo sein
      - NAME: args
        TYP: Option<Vec<String>>
        BESCHREIBUNG: Argumente für die Anwendung
        EINSCHRÄNKUNGEN: Keine
      - NAME: env
        TYP: Option<HashMap<String, String>>
        BESCHREIBUNG: Umgebungsvariablen für die Anwendung
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<ApplicationSession, ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn ein Fehler beim Starten der Anwendung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird gestartet
      - Eine ApplicationSession wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Starten der Anwendung auftritt
  
  - NAME: launch_with_files
    BESCHREIBUNG: Startet eine Anwendung mit Dateien
    PARAMETER:
      - NAME: info
        TYP: &ApplicationInfo
        BESCHREIBUNG: Informationen über die Anwendung
        EINSCHRÄNKUNGEN: Muss gültige ApplicationInfo sein
      - NAME: files
        TYP: Vec<&Path>
        BESCHREIBUNG: Dateien, die geöffnet werden sollen
        EINSCHRÄNKUNGEN: Müssen gültige Pfade sein
      - NAME: args
        TYP: Option<Vec<String>>
        BESCHREIBUNG: Argumente für die Anwendung
        EINSCHRÄNKUNGEN: Keine
      - NAME: env
        TYP: Option<HashMap<String, String>>
        BESCHREIBUNG: Umgebungsvariablen für die Anwendung
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<ApplicationSession, ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn ein Fehler beim Starten der Anwendung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird mit den angegebenen Dateien gestartet
      - Eine ApplicationSession wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Starten der Anwendung auftritt
  
  - NAME: launch_by_command
    BESCHREIBUNG: Startet eine Anwendung anhand eines Befehls
    PARAMETER:
      - NAME: command
        TYP: &str
        BESCHREIBUNG: Befehl zum Starten der Anwendung
        EINSCHRÄNKUNGEN: Muss ein gültiger Befehl sein
      - NAME: args
        TYP: Option<Vec<String>>
        BESCHREIBUNG: Argumente für die Anwendung
        EINSCHRÄNKUNGEN: Keine
      - NAME: env
        TYP: Option<HashMap<String, String>>
        BESCHREIBUNG: Umgebungsvariablen für die Anwendung
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<ApplicationSession, ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn ein Fehler beim Starten der Anwendung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird gestartet
      - Eine ApplicationSession wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Starten der Anwendung auftritt
  
  - NAME: terminate
    BESCHREIBUNG: Beendet eine Anwendung
    PARAMETER:
      - NAME: session
        TYP: &ApplicationSession
        BESCHREIBUNG: Sitzung der Anwendung
        EINSCHRÄNKUNGEN: Muss eine gültige ApplicationSession sein
      - NAME: force
        TYP: bool
        BESCHREIBUNG: Ob die Anwendung zwangsweise beendet werden soll
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), ApplicationError>
    FEHLER:
      - TYP: ApplicationError
        BEDINGUNG: Wenn ein Fehler beim Beenden der Anwendung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Anwendung wird beendet
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Beenden der Anwendung auftritt
```

## 6. Datenmodell (Teil 1)

### 6.1 ApplicationInfo

```
ENTITÄT: ApplicationInfo
BESCHREIBUNG: Informationen über eine Anwendung
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: description
    TYP: Option<String>
    BESCHREIBUNG: Beschreibung der Anwendung
    WERTEBEREICH: Zeichenkette oder None
    STANDARDWERT: None
  - NAME: command
    TYP: String
    BESCHREIBUNG: Befehl zum Starten der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: icon
    TYP: Option<String>
    BESCHREIBUNG: Icon der Anwendung
    WERTEBEREICH: Gültiger Icon-Name oder Pfad oder None
    STANDARDWERT: None
  - NAME: categories
    TYP: Vec<ApplicationCategory>
    BESCHREIBUNG: Kategorien der Anwendung
    WERTEBEREICH: Gültige ApplicationCategory-Werte
    STANDARDWERT: Leerer Vec
  - NAME: mime_types
    TYP: Vec<String>
    BESCHREIBUNG: MIME-Typen, die die Anwendung unterstützt
    WERTEBEREICH: Gültige MIME-Typ-Zeichenketten
    STANDARDWERT: Leerer Vec
  - NAME: keywords
    TYP: Vec<String>
    BESCHREIBUNG: Schlüsselwörter für die Anwendung
    WERTEBEREICH: Zeichenketten
    STANDARDWERT: Leerer Vec
  - NAME: exec_pattern
    TYP: String
    BESCHREIBUNG: Muster für die Ausführung der Anwendung
    WERTEBEREICH: Gültiges Ausführungsmuster
    STANDARDWERT: "%c"
  - NAME: terminal
    TYP: bool
    BESCHREIBUNG: Ob die Anwendung in einem Terminal ausgeführt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: startup_notify
    TYP: bool
    BESCHREIBUNG: Ob die Anwendung Startup-Benachrichtigungen unterstützt
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: no_display
    TYP: bool
    BESCHREIBUNG: Ob die Anwendung nicht im Menü angezeigt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: hidden
    TYP: bool
    BESCHREIBUNG: Ob die Anwendung versteckt ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: only_show_in
    TYP: Option<Vec<String>>
    BESCHREIBUNG: Desktop-Umgebungen, in denen die Anwendung angezeigt werden soll
    WERTEBEREICH: Gültige Desktop-Umgebungs-Namen oder None
    STANDARDWERT: None
  - NAME: not_show_in
    TYP: Option<Vec<String>>
    BESCHREIBUNG: Desktop-Umgebungen, in denen die Anwendung nicht angezeigt werden soll
    WERTEBEREICH: Gültige Desktop-Umgebungs-Namen oder None
    STANDARDWERT: None
  - NAME: try_exec
    TYP: Option<String>
    BESCHREIBUNG: Befehl zum Testen, ob die Anwendung ausgeführt werden kann
    WERTEBEREICH: Gültiger Befehl oder None
    STANDARDWERT: None
  - NAME: path
    TYP: Option<String>
    BESCHREIBUNG: Arbeitsverzeichnis für die Anwendung
    WERTEBEREICH: Gültiger Pfad oder None
    STANDARDWERT: None
  - NAME: startup_wm_class
    TYP: Option<String>
    BESCHREIBUNG: WM_CLASS für die Anwendung
    WERTEBEREICH: Gültige WM_CLASS oder None
    STANDARDWERT: None
  - NAME: url
    TYP: Option<String>
    BESCHREIBUNG: URL der Anwendung
    WERTEBEREICH: Gültige URL oder None
    STANDARDWERT: None
  - NAME: generic_name
    TYP: Option<String>
    BESCHREIBUNG: Generischer Name der Anwendung
    WERTEBEREICH: Zeichenkette oder None
    STANDARDWERT: None
  - NAME: actions
    TYP: Vec<ApplicationAction>
    BESCHREIBUNG: Aktionen, die die Anwendung unterstützt
    WERTEBEREICH: Gültige ApplicationAction-Werte
    STANDARDWERT: Leerer Vec
  - NAME: file_path
    TYP: Option<PathBuf>
    BESCHREIBUNG: Pfad zur Desktop-Datei
    WERTEBEREICH: Gültiger Pfad oder None
    STANDARDWERT: None
INVARIANTEN:
  - id darf nicht leer sein
  - name darf nicht leer sein
  - command darf nicht leer sein
```

### 6.2 ApplicationCategory

```
ENTITÄT: ApplicationCategory
BESCHREIBUNG: Kategorie für Anwendungen
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
INVARIANTEN:
  - id darf nicht leer sein
  - name darf nicht leer sein
```

### 6.3 ApplicationState

```
ENTITÄT: ApplicationState
BESCHREIBUNG: Zustand einer Anwendung
ATTRIBUTE:
  - NAME: state
    TYP: Enum
    BESCHREIBUNG: Zustand
    WERTEBEREICH: {
      Starting,
      Running,
      Paused,
      Stopping,
      Stopped,
      Failed,
      Unknown
    }
    STANDARDWERT: Unknown
  - NAME: pid
    TYP: Option<u32>
    BESCHREIBUNG: Prozess-ID
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: exit_code
    TYP: Option<i32>
    BESCHREIBUNG: Exit-Code
    WERTEBEREICH: Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: start_time
    TYP: Option<SystemTime>
    BESCHREIBUNG: Startzeitpunkt
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
  - NAME: end_time
    TYP: Option<SystemTime>
    BESCHREIBUNG: Endzeitpunkt
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
  - NAME: cpu_usage
    TYP: Option<f32>
    BESCHREIBUNG: CPU-Auslastung in Prozent
    WERTEBEREICH: [0.0, 100.0] oder None
    STANDARDWERT: None
  - NAME: memory_usage
    TYP: Option<u64>
    BESCHREIBUNG: Speicherverbrauch in Bytes
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: error_message
    TYP: Option<String>
    BESCHREIBUNG: Fehlermeldung
    WERTEBEREICH: Zeichenkette oder None
    STANDARDWERT: None
INVARIANTEN:
  - Wenn state == Stopped oder state == Failed, muss end_time vorhanden sein
  - Wenn state == Running, muss pid vorhanden sein
  - Wenn state == Failed, sollte error_message vorhanden sein
  - Wenn exit_code vorhanden ist, muss state == Stopped oder state == Failed sein
  - cpu_usage muss im Bereich [0.0, 100.0] liegen, wenn vorhanden
```

### 6.4 ApplicationSession

```
ENTITÄT: ApplicationSession
BESCHREIBUNG: Sitzung einer Anwendung
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID der Sitzung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: application_id
    TYP: String
    BESCHREIBUNG: ID der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: pid
    TYP: Option<u32>
    BESCHREIBUNG: Prozess-ID
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: command
    TYP: String
    BESCHREIBUNG: Ausgeführter Befehl
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: args
    TYP: Vec<String>
    BESCHREIBUNG: Argumente
    WERTEBEREICH: Zeichenketten
    STANDARDWERT: Leerer Vec
  - NAME: env
    TYP: HashMap<String, String>
    BESCHREIBUNG: Umgebungsvariablen
    WERTEBEREICH: Gültige Umgebungsvariablen
    STANDARDWERT: Leere HashMap
  - NAME: start_time
    TYP: SystemTime
    BESCHREIBUNG: Startzeitpunkt
    WERTEBEREICH: Gültiger Zeitpunkt
    STANDARDWERT: SystemTime::now()
  - NAME: end_time
    TYP: Option<SystemTime>
    BESCHREIBUNG: Endzeitpunkt
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
  - NAME: state
    TYP: ApplicationState
    BESCHREIBUNG: Zustand
    WERTEBEREICH: Gültiger ApplicationState
    STANDARDWERT: ApplicationState { state: ApplicationStateEnum::Starting, pid: None, exit_code: None, start_time: Some(SystemTime::now()), end_time: None, cpu_usage: None, memory_usage: None, error_message: None }
  - NAME: windows
    TYP: Vec<WindowId>
    BESCHREIBUNG: Fenster-IDs
    WERTEBEREICH: Gültige WindowId-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - id darf nicht leer sein
  - application_id darf nicht leer sein
  - command darf nicht leer sein
  - Wenn state.state == ApplicationStateEnum::Running, muss pid vorhanden sein
  - Wenn state.state == ApplicationStateEnum::Stopped oder state.state == ApplicationStateEnum::Failed, muss end_time vorhanden sein
```

### 6.5 ApplicationAction

```
ENTITÄT: ApplicationAction
BESCHREIBUNG: Aktion einer Anwendung
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID der Aktion
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name der Aktion
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: command
    TYP: String
    BESCHREIBUNG: Befehl für die Aktion
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: icon
    TYP: Option<String>
    BESCHREIBUNG: Icon der Aktion
    WERTEBEREICH: Gültiger Icon-Name oder Pfad oder None
    STANDARDWERT: None
INVARIANTEN:
  - id darf nicht leer sein
  - name darf nicht leer sein
  - command darf nicht leer sein
```

### 6.6 ApplicationPreference

```
ENTITÄT: ApplicationPreference
BESCHREIBUNG: Präferenz für eine Anwendung
ATTRIBUTE:
  - NAME: application_id
    TYP: String
    BESCHREIBUNG: ID der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: autostart
    TYP: bool
    BESCHREIBUNG: Ob die Anwendung automatisch gestartet werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: autostart_delay
    TYP: Option<u32>
    BESCHREIBUNG: Verzögerung für den Autostart in Sekunden
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: single_instance
    TYP: bool
    BESCHREIBUNG: Ob nur eine Instanz der Anwendung ausgeführt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: startup_notification
    TYP: bool
    BESCHREIBUNG: Ob Startup-Benachrichtigungen angezeigt werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: custom_command
    TYP: Option<String>
    BESCHREIBUNG: Benutzerdefinierter Befehl zum Starten der Anwendung
    WERTEBEREICH: Gültiger Befehl oder None
    STANDARDWERT: None
  - NAME: custom_args
    TYP: Option<Vec<String>>
    BESCHREIBUNG: Benutzerdefinierte Argumente für die Anwendung
    WERTEBEREICH: Zeichenketten oder None
    STANDARDWERT: None
  - NAME: custom_env
    TYP: Option<HashMap<String, String>>
    BESCHREIBUNG: Benutzerdefinierte Umgebungsvariablen für die Anwendung
    WERTEBEREICH: Gültige Umgebungsvariablen oder None
    STANDARDWERT: None
  - NAME: custom_icon
    TYP: Option<String>
    BESCHREIBUNG: Benutzerdefiniertes Icon für die Anwendung
    WERTEBEREICH: Gültiger Icon-Name oder Pfad oder None
    STANDARDWERT: None
  - NAME: custom_name
    TYP: Option<String>
    BESCHREIBUNG: Benutzerdefinierter Name für die Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: custom_categories
    TYP: Option<Vec<ApplicationCategory>>
    BESCHREIBUNG: Benutzerdefinierte Kategorien für die Anwendung
    WERTEBEREICH: Gültige ApplicationCategory-Werte oder None
    STANDARDWERT: None
INVARIANTEN:
  - application_id darf nicht leer sein
  - Wenn autostart_delay vorhanden ist, muss autostart true sein
  - Wenn custom_name vorhanden ist, darf es nicht leer sein
```

### 6.7 ApplicationMenuOptions

```
ENTITÄT: ApplicationMenuOptions
BESCHREIBUNG: Optionen für ein Anwendungsmenü
ATTRIBUTE:
  - NAME: categories
    TYP: Option<Vec<ApplicationCategory>>
    BESCHREIBUNG: Kategorien, die im Menü angezeigt werden sollen
    WERTEBEREICH: Gültige ApplicationCategory-Werte oder None
    STANDARDWERT: None
  - NAME: include_hidden
    TYP: bool
    BESCHREIBUNG: Ob versteckte Anwendungen eingeschlossen werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: sort_by
    TYP: ApplicationSortCriteria
    BESCHREIBUNG: Kriterium für die Sortierung
    WERTEBEREICH: Gültige ApplicationSortCriteria
    STANDARDWERT: ApplicationSortCriteria::Name
  - NAME: sort_order
    TYP: SortOrder
    BESCHREIBUNG: Sortierreihenfolge
    WERTEBEREICH: Gültige SortOrder
    STANDARDWERT: SortOrder::Ascending
  - NAME: max_items
    TYP: Option<usize>
    BESCHREIBUNG: Maximale Anzahl von Einträgen
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: filter
    TYP: Option<Box<dyn Fn(&ApplicationInfo) -> bool + Send + Sync + 'static>>
    BESCHREIBUNG: Filterfunktion
    WERTEBEREICH: Gültige Funktion oder None
    STANDARDWERT: None
INVARIANTEN:
  - Keine
```

### 6.8 ApplicationSortCriteria

```
ENTITÄT: ApplicationSortCriteria
BESCHREIBUNG: Kriterium für die Sortierung von Anwendungen
ATTRIBUTE:
  - NAME: criteria
    TYP: Enum
    BESCHREIBUNG: Kriterium
    WERTEBEREICH: {
      Name,
      Category,
      Popularity,
      RecentlyUsed,
      FrequentlyUsed
    }
    STANDARDWERT: Name
INVARIANTEN:
  - Keine
```

### 6.9 ApplicationMenu

```
ENTITÄT: ApplicationMenu
BESCHREIBUNG: Menü für Anwendungen
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Menüs
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: categories
    TYP: Vec<ApplicationMenuCategory>
    BESCHREIBUNG: Kategorien im Menü
    WERTEBEREICH: Gültige ApplicationMenuCategory-Werte
    STANDARDWERT: Leerer Vec
  - NAME: all_applications
    TYP: Vec<ApplicationInfo>
    BESCHREIBUNG: Alle Anwendungen im Menü
    WERTEBEREICH: Gültige ApplicationInfo-Werte
    STANDARDWERT: Leerer Vec
  - NAME: recent_applications
    TYP: Vec<ApplicationInfo>
    BESCHREIBUNG: Kürzlich verwendete Anwendungen
    WERTEBEREICH: Gültige ApplicationInfo-Werte
    STANDARDWERT: Leerer Vec
  - NAME: favorite_applications
    TYP: Vec<ApplicationInfo>
    BESCHREIBUNG: Favorisierte Anwendungen
    WERTEBEREICH: Gültige ApplicationInfo-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - id darf nicht leer sein
```

### 6.10 ApplicationMenuCategory

```
ENTITÄT: ApplicationMenuCategory
BESCHREIBUNG: Kategorie in einem Anwendungsmenü
ATTRIBUTE:
  - NAME: category
    TYP: ApplicationCategory
    BESCHREIBUNG: Kategorie
    WERTEBEREICH: Gültige ApplicationCategory
    STANDARDWERT: Keiner
  - NAME: applications
    TYP: Vec<ApplicationInfo>
    BESCHREIBUNG: Anwendungen in der Kategorie
    WERTEBEREICH: Gültige ApplicationInfo-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - Keine
```
