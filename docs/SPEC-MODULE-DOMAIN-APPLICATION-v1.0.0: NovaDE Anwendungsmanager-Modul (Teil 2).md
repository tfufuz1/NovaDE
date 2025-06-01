# SPEC-MODULE-DOMAIN-APPLICATION-v1.0.0: NovaDE Anwendungsmanager-Modul (Teil 2)

## 6. Datenmodell (Fortsetzung)

### 6.11 ApplicationConfig

```
ENTITÄT: ApplicationConfig
BESCHREIBUNG: Konfiguration für den ApplicationManager
ATTRIBUTE:
  - NAME: application_dirs
    TYP: Vec<PathBuf>
    BESCHREIBUNG: Verzeichnisse für Anwendungen
    WERTEBEREICH: Gültige Pfade
    STANDARDWERT: vec![PathBuf::from("/usr/share/applications"), PathBuf::from("~/.local/share/applications")]
  - NAME: autostart_dir
    TYP: PathBuf
    BESCHREIBUNG: Verzeichnis für Autostart-Anwendungen
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: PathBuf::from("~/.config/autostart")
  - NAME: cache_enabled
    TYP: bool
    BESCHREIBUNG: Ob das Caching aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: cache_ttl
    TYP: Duration
    BESCHREIBUNG: Time-to-Live für Cache-Einträge
    WERTEBEREICH: Positive Zeitdauer
    STANDARDWERT: Duration::from_secs(300)
  - NAME: watch_application_dirs
    TYP: bool
    BESCHREIBUNG: Ob Anwendungsverzeichnisse überwacht werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: default_terminal
    TYP: String
    BESCHREIBUNG: Standardterminal für Anwendungen, die ein Terminal benötigen
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "xterm -e"
  - NAME: default_file_manager
    TYP: String
    BESCHREIBUNG: Standard-Dateimanager
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "xdg-open"
  - NAME: default_browser
    TYP: String
    BESCHREIBUNG: Standardbrowser
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "xdg-open"
  - NAME: default_email_client
    TYP: String
    BESCHREIBUNG: Standard-E-Mail-Client
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "xdg-email"
  - NAME: mime_cache_file
    TYP: PathBuf
    BESCHREIBUNG: Pfad zur MIME-Cache-Datei
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: PathBuf::from("~/.cache/nova/mime.cache")
  - NAME: recent_applications_max
    TYP: usize
    BESCHREIBUNG: Maximale Anzahl von kürzlich verwendeten Anwendungen
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: favorite_applications_file
    TYP: PathBuf
    BESCHREIBUNG: Pfad zur Datei mit favorisierten Anwendungen
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: PathBuf::from("~/.config/nova/favorite_applications.json")
  - NAME: application_history_file
    TYP: PathBuf
    BESCHREIBUNG: Pfad zur Datei mit dem Anwendungsverlauf
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: PathBuf::from("~/.local/share/nova/application_history.json")
  - NAME: application_preferences_dir
    TYP: PathBuf
    BESCHREIBUNG: Verzeichnis für Anwendungspräferenzen
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: PathBuf::from("~/.config/nova/application_preferences")
  - NAME: default_applications_file
    TYP: PathBuf
    BESCHREIBUNG: Pfad zur Datei mit Standardanwendungen
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: PathBuf::from("~/.config/nova/default_applications.json")
  - NAME: process_check_interval
    TYP: Duration
    BESCHREIBUNG: Intervall für die Prozessprüfung
    WERTEBEREICH: Positive Zeitdauer
    STANDARDWERT: Duration::from_secs(1)
INVARIANTEN:
  - application_dirs darf nicht leer sein
  - cache_ttl muss größer als Duration::from_secs(0) sein
  - recent_applications_max muss größer als 0 sein
  - process_check_interval muss größer als Duration::from_millis(100) sein
```

### 6.12 ApplicationLauncherConfig

```
ENTITÄT: ApplicationLauncherConfig
BESCHREIBUNG: Konfiguration für den ApplicationLauncher
ATTRIBUTE:
  - NAME: default_terminal
    TYP: String
    BESCHREIBUNG: Standardterminal für Anwendungen, die ein Terminal benötigen
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "xterm -e"
  - NAME: default_env
    TYP: HashMap<String, String>
    BESCHREIBUNG: Standardumgebungsvariablen für Anwendungen
    WERTEBEREICH: Gültige Umgebungsvariablen
    STANDARDWERT: HashMap::new()
  - NAME: startup_notification_timeout
    TYP: Duration
    BESCHREIBUNG: Timeout für Startup-Benachrichtigungen
    WERTEBEREICH: Positive Zeitdauer
    STANDARDWERT: Duration::from_secs(30)
  - NAME: launch_timeout
    TYP: Duration
    BESCHREIBUNG: Timeout für das Starten von Anwendungen
    WERTEBEREICH: Positive Zeitdauer
    STANDARDWERT: Duration::from_secs(60)
  - NAME: max_concurrent_launches
    TYP: usize
    BESCHREIBUNG: Maximale Anzahl von gleichzeitigen Starts
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 5
  - NAME: use_shell
    TYP: bool
    BESCHREIBUNG: Ob eine Shell für das Starten von Anwendungen verwendet werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: shell_path
    TYP: String
    BESCHREIBUNG: Pfad zur Shell
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "/bin/sh"
  - NAME: shell_args
    TYP: Vec<String>
    BESCHREIBUNG: Argumente für die Shell
    WERTEBEREICH: Zeichenketten
    STANDARDWERT: vec!["-c".to_string()]
INVARIANTEN:
  - default_terminal darf nicht leer sein
  - startup_notification_timeout muss größer als Duration::from_secs(0) sein
  - launch_timeout muss größer als Duration::from_secs(0) sein
  - max_concurrent_launches muss größer als 0 sein
  - shell_path darf nicht leer sein
```

### 6.13 ApplicationEvent

```
ENTITÄT: ApplicationEvent
BESCHREIBUNG: Ereignis einer Anwendung
ATTRIBUTE:
  - NAME: event_type
    TYP: ApplicationEventType
    BESCHREIBUNG: Typ des Ereignisses
    WERTEBEREICH: Gültige ApplicationEventType
    STANDARDWERT: Keiner
  - NAME: application_id
    TYP: String
    BESCHREIBUNG: ID der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: session_id
    TYP: Option<String>
    BESCHREIBUNG: ID der Sitzung
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: timestamp
    TYP: SystemTime
    BESCHREIBUNG: Zeitpunkt des Ereignisses
    WERTEBEREICH: Gültiger Zeitpunkt
    STANDARDWERT: SystemTime::now()
  - NAME: details
    TYP: HashMap<String, String>
    BESCHREIBUNG: Details zum Ereignis
    WERTEBEREICH: Gültige Schlüssel-Wert-Paare
    STANDARDWERT: HashMap::new()
INVARIANTEN:
  - application_id darf nicht leer sein
  - Wenn session_id vorhanden ist, darf es nicht leer sein
```

### 6.14 ApplicationEventType

```
ENTITÄT: ApplicationEventType
BESCHREIBUNG: Typ eines Anwendungsereignisses
ATTRIBUTE:
  - NAME: event_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      Added,
      Removed,
      Updated,
      Launched,
      Terminated,
      StateChanged,
      WindowOpened,
      WindowClosed,
      FocusGained,
      FocusLost,
      Crashed,
      Error
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Keine
```

### 6.15 ApplicationPermission

```
ENTITÄT: ApplicationPermission
BESCHREIBUNG: Berechtigung für eine Anwendung
ATTRIBUTE:
  - NAME: application_id
    TYP: String
    BESCHREIBUNG: ID der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: permission_type
    TYP: ApplicationPermissionType
    BESCHREIBUNG: Typ der Berechtigung
    WERTEBEREICH: Gültige ApplicationPermissionType
    STANDARDWERT: Keiner
  - NAME: granted
    TYP: bool
    BESCHREIBUNG: Ob die Berechtigung gewährt wurde
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: timestamp
    TYP: SystemTime
    BESCHREIBUNG: Zeitpunkt der Berechtigungserteilung
    WERTEBEREICH: Gültiger Zeitpunkt
    STANDARDWERT: SystemTime::now()
  - NAME: expiration
    TYP: Option<SystemTime>
    BESCHREIBUNG: Ablaufzeitpunkt der Berechtigung
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
  - NAME: scope
    TYP: Option<String>
    BESCHREIBUNG: Geltungsbereich der Berechtigung
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
INVARIANTEN:
  - application_id darf nicht leer sein
  - Wenn expiration vorhanden ist, muss es größer als timestamp sein
  - Wenn scope vorhanden ist, darf es nicht leer sein
```

### 6.16 ApplicationPermissionType

```
ENTITÄT: ApplicationPermissionType
BESCHREIBUNG: Typ einer Anwendungsberechtigung
ATTRIBUTE:
  - NAME: permission_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      FileAccess,
      NetworkAccess,
      NotificationAccess,
      CameraAccess,
      MicrophoneAccess,
      LocationAccess,
      ContactsAccess,
      CalendarAccess,
      BluetoothAccess,
      UsbAccess,
      PrinterAccess,
      ScreenshotAccess,
      AutostartAccess,
      BackgroundAccess,
      SystemSettingsAccess,
      Custom(String)
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Bei Custom darf die Zeichenkette nicht leer sein
```

### 6.17 ApplicationResource

```
ENTITÄT: ApplicationResource
BESCHREIBUNG: Ressource einer Anwendung
ATTRIBUTE:
  - NAME: application_id
    TYP: String
    BESCHREIBUNG: ID der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: session_id
    TYP: Option<String>
    BESCHREIBUNG: ID der Sitzung
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: resource_type
    TYP: ApplicationResourceType
    BESCHREIBUNG: Typ der Ressource
    WERTEBEREICH: Gültige ApplicationResourceType
    STANDARDWERT: Keiner
  - NAME: current_usage
    TYP: u64
    BESCHREIBUNG: Aktuelle Nutzung
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
  - NAME: limit
    TYP: Option<u64>
    BESCHREIBUNG: Limit
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: timestamp
    TYP: SystemTime
    BESCHREIBUNG: Zeitpunkt der Messung
    WERTEBEREICH: Gültiger Zeitpunkt
    STANDARDWERT: SystemTime::now()
INVARIANTEN:
  - application_id darf nicht leer sein
  - Wenn session_id vorhanden ist, darf es nicht leer sein
  - Wenn limit vorhanden ist, muss es größer als 0 sein
```

### 6.18 ApplicationResourceType

```
ENTITÄT: ApplicationResourceType
BESCHREIBUNG: Typ einer Anwendungsressource
ATTRIBUTE:
  - NAME: resource_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      CpuUsage,
      MemoryUsage,
      DiskUsage,
      NetworkUsage,
      GpuUsage,
      BatteryUsage,
      FileHandles,
      Threads,
      Processes,
      Custom(String)
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Bei Custom darf die Zeichenkette nicht leer sein
```

### 6.19 ApplicationMonitorConfig

```
ENTITÄT: ApplicationMonitorConfig
BESCHREIBUNG: Konfiguration für den ApplicationMonitor
ATTRIBUTE:
  - NAME: check_interval
    TYP: Duration
    BESCHREIBUNG: Intervall für die Überprüfung
    WERTEBEREICH: Positive Zeitdauer
    STANDARDWERT: Duration::from_secs(1)
  - NAME: resource_check_interval
    TYP: Duration
    BESCHREIBUNG: Intervall für die Ressourcenüberprüfung
    WERTEBEREICH: Positive Zeitdauer
    STANDARDWERT: Duration::from_secs(5)
  - NAME: cpu_limit
    TYP: Option<f32>
    BESCHREIBUNG: CPU-Limit in Prozent
    WERTEBEREICH: [0.0, 100.0] oder None
    STANDARDWERT: None
  - NAME: memory_limit
    TYP: Option<u64>
    BESCHREIBUNG: Speicherlimit in Bytes
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: disk_limit
    TYP: Option<u64>
    BESCHREIBUNG: Festplattenlimit in Bytes
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: network_limit
    TYP: Option<u64>
    BESCHREIBUNG: Netzwerklimit in Bytes pro Sekunde
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: notify_on_high_usage
    TYP: bool
    BESCHREIBUNG: Ob bei hoher Ressourcennutzung benachrichtigt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: high_cpu_threshold
    TYP: f32
    BESCHREIBUNG: Schwellenwert für hohe CPU-Nutzung in Prozent
    WERTEBEREICH: [0.0, 100.0]
    STANDARDWERT: 80.0
  - NAME: high_memory_threshold
    TYP: f32
    BESCHREIBUNG: Schwellenwert für hohe Speichernutzung in Prozent
    WERTEBEREICH: [0.0, 100.0]
    STANDARDWERT: 80.0
  - NAME: crash_detection_enabled
    TYP: bool
    BESCHREIBUNG: Ob die Absturzerkennung aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: restart_on_crash
    TYP: bool
    BESCHREIBUNG: Ob Anwendungen bei Absturz neu gestartet werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: max_restart_attempts
    TYP: u32
    BESCHREIBUNG: Maximale Anzahl von Neustartversuchen
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 3
  - NAME: restart_cooldown
    TYP: Duration
    BESCHREIBUNG: Abkühlzeit zwischen Neustartversuchen
    WERTEBEREICH: Positive Zeitdauer
    STANDARDWERT: Duration::from_secs(10)
INVARIANTEN:
  - check_interval muss größer als Duration::from_millis(100) sein
  - resource_check_interval muss größer als Duration::from_millis(100) sein
  - high_cpu_threshold muss im Bereich [0.0, 100.0] liegen
  - high_memory_threshold muss im Bereich [0.0, 100.0] liegen
  - max_restart_attempts muss größer als 0 sein
  - restart_cooldown muss größer als Duration::from_secs(0) sein
```

### 6.20 ApplicationHistory

```
ENTITÄT: ApplicationHistory
BESCHREIBUNG: Verlauf einer Anwendung
ATTRIBUTE:
  - NAME: application_id
    TYP: String
    BESCHREIBUNG: ID der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: launch_count
    TYP: u32
    BESCHREIBUNG: Anzahl der Starts
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
  - NAME: last_launched
    TYP: Option<SystemTime>
    BESCHREIBUNG: Zeitpunkt des letzten Starts
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
  - NAME: total_runtime
    TYP: Duration
    BESCHREIBUNG: Gesamtlaufzeit
    WERTEBEREICH: Positive Zeitdauer
    STANDARDWERT: Duration::from_secs(0)
  - NAME: average_runtime
    TYP: Option<Duration>
    BESCHREIBUNG: Durchschnittliche Laufzeit
    WERTEBEREICH: Positive Zeitdauer oder None
    STANDARDWERT: None
  - NAME: crash_count
    TYP: u32
    BESCHREIBUNG: Anzahl der Abstürze
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
  - NAME: last_crash
    TYP: Option<SystemTime>
    BESCHREIBUNG: Zeitpunkt des letzten Absturzes
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
  - NAME: last_exit_code
    TYP: Option<i32>
    BESCHREIBUNG: Letzter Exit-Code
    WERTEBEREICH: Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: recent_sessions
    TYP: Vec<ApplicationSession>
    BESCHREIBUNG: Kürzlich verwendete Sitzungen
    WERTEBEREICH: Gültige ApplicationSession-Werte
    STANDARDWERT: Leerer Vec
  - NAME: is_favorite
    TYP: bool
    BESCHREIBUNG: Ob die Anwendung favorisiert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
INVARIANTEN:
  - application_id darf nicht leer sein
  - Wenn launch_count > 0, muss last_launched vorhanden sein
  - Wenn crash_count > 0, muss last_crash vorhanden sein
```

## 7. Verhaltensmodell

### 7.1 Anwendungsstart

```
ZUSTANDSAUTOMAT: ApplicationLaunch
BESCHREIBUNG: Prozess des Startens einer Anwendung
ZUSTÄNDE:
  - NAME: Initial
    BESCHREIBUNG: Initialer Zustand
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: ValidatingApplication
    BESCHREIBUNG: Anwendung wird validiert
    EINTRITTSAKTIONEN: Anwendungsinformationen prüfen
    AUSTRITTSAKTIONEN: Keine
  - NAME: CheckingPermissions
    BESCHREIBUNG: Berechtigungen werden geprüft
    EINTRITTSAKTIONEN: Berechtigungen abrufen
    AUSTRITTSAKTIONEN: Keine
  - NAME: PreparingEnvironment
    BESCHREIBUNG: Umgebung wird vorbereitet
    EINTRITTSAKTIONEN: Umgebungsvariablen setzen
    AUSTRITTSAKTIONEN: Keine
  - NAME: BuildingCommand
    BESCHREIBUNG: Befehl wird erstellt
    EINTRITTSAKTIONEN: Befehlszeile zusammensetzen
    AUSTRITTSAKTIONEN: Keine
  - NAME: LaunchingProcess
    BESCHREIBUNG: Prozess wird gestartet
    EINTRITTSAKTIONEN: Prozess starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: WaitingForStartup
    BESCHREIBUNG: Auf Startup-Benachrichtigung wird gewartet
    EINTRITTSAKTIONEN: Timer starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: CreatingSession
    BESCHREIBUNG: Sitzung wird erstellt
    EINTRITTSAKTIONEN: Sitzungsdaten erstellen
    AUSTRITTSAKTIONEN: Keine
  - NAME: NotifyingListeners
    BESCHREIBUNG: Listener werden benachrichtigt
    EINTRITTSAKTIONEN: Listener-Liste durchlaufen
    AUSTRITTSAKTIONEN: Keine
  - NAME: UpdatingHistory
    BESCHREIBUNG: Verlauf wird aktualisiert
    EINTRITTSAKTIONEN: Verlaufsdaten aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Completed
    BESCHREIBUNG: Start abgeschlossen
    EINTRITTSAKTIONEN: Statistiken aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler beim Start
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Initial
    NACH: ValidatingApplication
    EREIGNIS: launch_application aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ValidatingApplication
    NACH: CheckingPermissions
    EREIGNIS: Anwendung erfolgreich validiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ValidatingApplication
    NACH: Error
    EREIGNIS: Anwendung nicht gefunden oder ungültig
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: CheckingPermissions
    NACH: PreparingEnvironment
    EREIGNIS: Berechtigungen erfolgreich geprüft
    BEDINGUNG: Alle erforderlichen Berechtigungen vorhanden
    AKTIONEN: Keine
  - VON: CheckingPermissions
    NACH: Error
    EREIGNIS: Fehlende Berechtigungen
    BEDINGUNG: Nicht alle erforderlichen Berechtigungen vorhanden
    AKTIONEN: ApplicationError erstellen
  - VON: PreparingEnvironment
    NACH: BuildingCommand
    EREIGNIS: Umgebung erfolgreich vorbereitet
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: PreparingEnvironment
    NACH: Error
    EREIGNIS: Fehler bei der Vorbereitung der Umgebung
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: BuildingCommand
    NACH: LaunchingProcess
    EREIGNIS: Befehl erfolgreich erstellt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: BuildingCommand
    NACH: Error
    EREIGNIS: Fehler bei der Erstellung des Befehls
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: LaunchingProcess
    NACH: WaitingForStartup
    EREIGNIS: Prozess erfolgreich gestartet
    BEDINGUNG: application_info.startup_notify
    AKTIONEN: Keine
  - VON: LaunchingProcess
    NACH: CreatingSession
    EREIGNIS: Prozess erfolgreich gestartet
    BEDINGUNG: !application_info.startup_notify
    AKTIONEN: Keine
  - VON: LaunchingProcess
    NACH: Error
    EREIGNIS: Fehler beim Starten des Prozesses
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: WaitingForStartup
    NACH: CreatingSession
    EREIGNIS: Startup-Benachrichtigung erhalten
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: WaitingForStartup
    NACH: CreatingSession
    EREIGNIS: Timeout für Startup-Benachrichtigung
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: CreatingSession
    NACH: NotifyingListeners
    EREIGNIS: Sitzung erfolgreich erstellt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: CreatingSession
    NACH: Error
    EREIGNIS: Fehler bei der Erstellung der Sitzung
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: NotifyingListeners
    NACH: UpdatingHistory
    EREIGNIS: Listener erfolgreich benachrichtigt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: NotifyingListeners
    NACH: Error
    EREIGNIS: Fehler bei der Benachrichtigung der Listener
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: UpdatingHistory
    NACH: Completed
    EREIGNIS: Verlauf erfolgreich aktualisiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: UpdatingHistory
    NACH: Error
    EREIGNIS: Fehler bei der Aktualisierung des Verlaufs
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
INITIALZUSTAND: Initial
ENDZUSTÄNDE: [Completed, Error]
```

### 7.2 Anwendungsbeendigung

```
ZUSTANDSAUTOMAT: ApplicationTermination
BESCHREIBUNG: Prozess des Beendens einer Anwendung
ZUSTÄNDE:
  - NAME: Initial
    BESCHREIBUNG: Initialer Zustand
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: ValidatingSession
    BESCHREIBUNG: Sitzung wird validiert
    EINTRITTSAKTIONEN: Sitzungsinformationen prüfen
    AUSTRITTSAKTIONEN: Keine
  - NAME: CheckingState
    BESCHREIBUNG: Zustand wird geprüft
    EINTRITTSAKTIONEN: Anwendungszustand abrufen
    AUSTRITTSAKTIONEN: Keine
  - NAME: SendingTerminateSignal
    BESCHREIBUNG: Beendigungssignal wird gesendet
    EINTRITTSAKTIONEN: Signal senden
    AUSTRITTSAKTIONEN: Keine
  - NAME: WaitingForTermination
    BESCHREIBUNG: Auf Beendigung wird gewartet
    EINTRITTSAKTIONEN: Timer starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: SendingKillSignal
    BESCHREIBUNG: Kill-Signal wird gesendet
    EINTRITTSAKTIONEN: Signal senden
    AUSTRITTSAKTIONEN: Keine
  - NAME: UpdatingSession
    BESCHREIBUNG: Sitzung wird aktualisiert
    EINTRITTSAKTIONEN: Sitzungsdaten aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: NotifyingListeners
    BESCHREIBUNG: Listener werden benachrichtigt
    EINTRITTSAKTIONEN: Listener-Liste durchlaufen
    AUSTRITTSAKTIONEN: Keine
  - NAME: UpdatingHistory
    BESCHREIBUNG: Verlauf wird aktualisiert
    EINTRITTSAKTIONEN: Verlaufsdaten aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Completed
    BESCHREIBUNG: Beendigung abgeschlossen
    EINTRITTSAKTIONEN: Statistiken aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler bei der Beendigung
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Initial
    NACH: ValidatingSession
    EREIGNIS: terminate_application aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ValidatingSession
    NACH: CheckingState
    EREIGNIS: Sitzung erfolgreich validiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ValidatingSession
    NACH: Error
    EREIGNIS: Sitzung nicht gefunden oder ungültig
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: CheckingState
    NACH: SendingTerminateSignal
    EREIGNIS: Anwendung läuft
    BEDINGUNG: session.state.state == ApplicationStateEnum::Running
    AKTIONEN: Keine
  - VON: CheckingState
    NACH: Completed
    EREIGNIS: Anwendung bereits beendet
    BEDINGUNG: session.state.state == ApplicationStateEnum::Stopped || session.state.state == ApplicationStateEnum::Failed
    AKTIONEN: Keine
  - VON: CheckingState
    NACH: Error
    EREIGNIS: Ungültiger Zustand
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: SendingTerminateSignal
    NACH: WaitingForTermination
    EREIGNIS: Signal erfolgreich gesendet
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: SendingTerminateSignal
    NACH: Error
    EREIGNIS: Fehler beim Senden des Signals
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: WaitingForTermination
    NACH: UpdatingSession
    EREIGNIS: Anwendung beendet
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: WaitingForTermination
    NACH: SendingKillSignal
    EREIGNIS: Timeout für Beendigung
    BEDINGUNG: force
    AKTIONEN: Keine
  - VON: WaitingForTermination
    NACH: Error
    EREIGNIS: Timeout für Beendigung
    BEDINGUNG: !force
    AKTIONEN: ApplicationError erstellen
  - VON: SendingKillSignal
    NACH: UpdatingSession
    EREIGNIS: Signal erfolgreich gesendet
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: SendingKillSignal
    NACH: Error
    EREIGNIS: Fehler beim Senden des Signals
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: UpdatingSession
    NACH: NotifyingListeners
    EREIGNIS: Sitzung erfolgreich aktualisiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: UpdatingSession
    NACH: Error
    EREIGNIS: Fehler bei der Aktualisierung der Sitzung
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: NotifyingListeners
    NACH: UpdatingHistory
    EREIGNIS: Listener erfolgreich benachrichtigt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: NotifyingListeners
    NACH: Error
    EREIGNIS: Fehler bei der Benachrichtigung der Listener
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
  - VON: UpdatingHistory
    NACH: Completed
    EREIGNIS: Verlauf erfolgreich aktualisiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: UpdatingHistory
    NACH: Error
    EREIGNIS: Fehler bei der Aktualisierung des Verlaufs
    BEDINGUNG: Keine
    AKTIONEN: ApplicationError erstellen
INITIALZUSTAND: Initial
ENDZUSTÄNDE: [Completed, Error]
```

## 8. Fehlerbehandlung

### 8.1 Fehlerbehandlungsstrategie

1. Alle Fehler MÜSSEN über spezifische Fehlertypen zurückgegeben werden.
2. Fehlertypen MÜSSEN mit `thiserror` definiert werden.
3. Fehler MÜSSEN kontextuelle Informationen enthalten.
4. Fehlerketten MÜSSEN bei der Weitergabe oder beim Wrappen von Fehlern erhalten bleiben.
5. Panics sind VERBOTEN, außer in Fällen, die explizit dokumentiert sind.

### 8.2 Modulspezifische Fehlertypen

```
ENTITÄT: ApplicationError
BESCHREIBUNG: Fehler im Anwendungsmanager-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      ApplicationNotFound { id: String },
      InvalidApplication { id: String, message: String },
      SessionNotFound { id: String },
      LaunchError { id: String, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      TerminationError { id: String, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      PermissionDenied { id: String, permission: ApplicationPermissionType, message: String },
      TimeoutError { id: String, operation: String, message: String },
      ResourceLimitExceeded { id: String, resource: ApplicationResourceType, limit: u64, current: u64 },
      ConfigError { message: String },
      IoError { message: String, source: std::io::Error },
      DBusError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

## 9. Leistungsanforderungen

### 9.1 Allgemeine Leistungsanforderungen

1. Das Anwendungsmanager-Modul MUSS effizient mit Ressourcen umgehen.
2. Das Anwendungsmanager-Modul MUSS eine geringe Latenz haben.
3. Das Anwendungsmanager-Modul MUSS skalierbar sein.

### 9.2 Spezifische Leistungsanforderungen

1. Das Starten einer Anwendung MUSS in unter 100ms abgeschlossen sein (ohne Berücksichtigung der Anwendungsstartzeit).
2. Das Beenden einer Anwendung MUSS in unter 50ms abgeschlossen sein (ohne Berücksichtigung der Anwendungsbeendigungszeit).
3. Das Abrufen von Anwendungsinformationen MUSS in unter 5ms abgeschlossen sein.
4. Das Generieren eines Anwendungsmenüs MUSS in unter 20ms abgeschlossen sein.
5. Das Anwendungsmanager-Modul MUSS mit mindestens 1000 installierten Anwendungen umgehen können.
6. Das Anwendungsmanager-Modul MUSS mit mindestens 100 gleichzeitig laufenden Anwendungen umgehen können.
7. Das Anwendungsmanager-Modul DARF nicht mehr als 1% CPU-Auslastung im Leerlauf verursachen.
8. Das Anwendungsmanager-Modul DARF nicht mehr als 50MB Speicher verbrauchen.

## 10. Sicherheitsanforderungen

### 10.1 Allgemeine Sicherheitsanforderungen

1. Das Anwendungsmanager-Modul MUSS memory-safe sein.
2. Das Anwendungsmanager-Modul MUSS thread-safe sein.
3. Das Anwendungsmanager-Modul MUSS robust gegen Fehleingaben sein.

### 10.2 Spezifische Sicherheitsanforderungen

1. Das Anwendungsmanager-Modul MUSS Eingaben validieren, um Command Injection-Angriffe zu verhindern.
2. Das Anwendungsmanager-Modul MUSS Zugriffskontrollen für Anwendungsoperationen implementieren.
3. Das Anwendungsmanager-Modul MUSS sichere Standardwerte verwenden.
4. Das Anwendungsmanager-Modul MUSS Ressourcenlimits implementieren, um Denial-of-Service-Angriffe zu verhindern.
5. Das Anwendungsmanager-Modul MUSS verhindern, dass nicht autorisierte Anwendungen auf geschützte Ressourcen zugreifen.
6. Das Anwendungsmanager-Modul MUSS Anwendungsstarts und -beendigungen protokollieren.
7. Das Anwendungsmanager-Modul MUSS Anwendungsberechtigungen sicher verwalten.
8. Das Anwendungsmanager-Modul MUSS Anwendungsressourcen überwachen und begrenzen.

## 11. Testkriterien

### 11.1 Allgemeine Testkriterien

1. Jede Komponente MUSS Einheitstests haben.
2. Jede öffentliche Funktion MUSS getestet sein.
3. Jeder Fehlerfall MUSS getestet sein.

### 11.2 Spezifische Testkriterien

1. Das Anwendungsmanager-Modul MUSS mit verschiedenen Anwendungstypen getestet sein.
2. Das Anwendungsmanager-Modul MUSS mit verschiedenen Anwendungskategorien getestet sein.
3. Das Anwendungsmanager-Modul MUSS mit verschiedenen Anwendungszuständen getestet sein.
4. Das Anwendungsmanager-Modul MUSS mit verschiedenen Anwendungsereignissen getestet sein.
5. Das Anwendungsmanager-Modul MUSS mit verschiedenen Anwendungsberechtigungen getestet sein.
6. Das Anwendungsmanager-Modul MUSS mit verschiedenen Fehlerszenarien getestet sein.
7. Das Anwendungsmanager-Modul MUSS mit verschiedenen Benutzerinteraktionen getestet sein.
8. Das Anwendungsmanager-Modul MUSS mit vielen gleichzeitig laufenden Anwendungen getestet sein.

## 12. Anhänge

### 12.1 Referenzierte Dokumente

1. SPEC-ROOT-v1.0.0: NovaDE Spezifikationswurzel
2. SPEC-LAYER-CORE-v1.0.0: Spezifikation der Kernschicht
3. SPEC-LAYER-DOMAIN-v1.0.0: Spezifikation der Domänenschicht
4. SPEC-MODULE-SYSTEM-PROCESS-v1.0.0: Spezifikation des Prozessmanager-Moduls
5. SPEC-MODULE-SYSTEM-FILESYSTEM-v1.0.0: Spezifikation des Dateisystemmanager-Moduls
6. SPEC-MODULE-SYSTEM-WINDOWMANAGER-v1.0.0: Spezifikation des Fenstermanager-Moduls
7. SPEC-MODULE-SYSTEM-NOTIFICATION-v1.0.0: Spezifikation des Benachrichtigungsmanager-Moduls

### 12.2 Externe Abhängigkeiten

1. `freedesktop_entry_parser`: Für das Parsen von Desktop-Einträgen
2. `xdg`: Für den Zugriff auf XDG-Verzeichnisse
3. `gio`: Für die Integration mit GIO
4. `dbus`: Für die D-Bus-Integration
5. `serde`: Für die Serialisierung und Deserialisierung
6. `json`: Für die JSON-Verarbeitung
