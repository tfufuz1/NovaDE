# SPEC-MODULE-SYSTEM-NOTIFICATION-v1.0.0: NovaDE Benachrichtigungsmanager-Modul (Teil 2)

## 6. Datenmodell (Fortsetzung)

### 6.11 NotificationEventType

```
ENTITÄT: NotificationEventType
BESCHREIBUNG: Typ eines Benachrichtigungsereignisses
ATTRIBUTE:
  - NAME: event_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      Created,
      Updated,
      Closed,
      ActionInvoked,
      DoNotDisturbChanged,
      PolicyChanged,
      CenterShown,
      CenterHidden,
      HistoryCleared
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Keine
```

### 6.12 NotificationCenterConfig

```
ENTITÄT: NotificationCenterConfig
BESCHREIBUNG: Konfiguration für das Benachrichtigungszentrum
ATTRIBUTE:
  - NAME: position
    TYP: NotificationPosition
    BESCHREIBUNG: Position des Benachrichtigungszentrums
    WERTEBEREICH: Gültige NotificationPosition
    STANDARDWERT: NotificationPosition::TopRight
  - NAME: width
    TYP: u32
    BESCHREIBUNG: Breite des Benachrichtigungszentrums in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 400
  - NAME: max_height
    TYP: u32
    BESCHREIBUNG: Maximale Höhe des Benachrichtigungszentrums in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 600
  - NAME: spacing
    TYP: u32
    BESCHREIBUNG: Abstand zwischen Benachrichtigungen in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: padding
    TYP: u32
    BESCHREIBUNG: Innenabstand in Pixeln
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10
  - NAME: animation_duration
    TYP: u32
    BESCHREIBUNG: Dauer der Animationen in Millisekunden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 250
  - NAME: show_animation
    TYP: bool
    BESCHREIBUNG: Ob Animationen beim Einblenden angezeigt werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: hide_animation
    TYP: bool
    BESCHREIBUNG: Ob Animationen beim Ausblenden angezeigt werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: background_color
    TYP: Option<Color>
    BESCHREIBUNG: Hintergrundfarbe des Benachrichtigungszentrums
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
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
  - NAME: border
    TYP: bool
    BESCHREIBUNG: Ob ein Rahmen angezeigt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: border_color
    TYP: Option<Color>
    BESCHREIBUNG: Farbe des Rahmens
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
  - NAME: border_width
    TYP: u32
    BESCHREIBUNG: Breite des Rahmens in Pixeln
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
    BESCHREIBUNG: Farbe des Schattens
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
  - NAME: group_by_app
    TYP: bool
    BESCHREIBUNG: Ob Benachrichtigungen nach Anwendung gruppiert werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: group_by_category
    TYP: bool
    BESCHREIBUNG: Ob Benachrichtigungen nach Kategorie gruppiert werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: sort_order
    TYP: NotificationSortOrder
    BESCHREIBUNG: Sortierreihenfolge der Benachrichtigungen
    WERTEBEREICH: Gültige NotificationSortOrder
    STANDARDWERT: NotificationSortOrder::NewestFirst
  - NAME: max_notifications
    TYP: u32
    BESCHREIBUNG: Maximale Anzahl von Benachrichtigungen im Benachrichtigungszentrum
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 100
INVARIANTEN:
  - background_opacity muss im Bereich [0.0, 1.0] liegen
```

### 6.13 NotificationPosition

```
ENTITÄT: NotificationPosition
BESCHREIBUNG: Position von Benachrichtigungen
ATTRIBUTE:
  - NAME: position
    TYP: Enum
    BESCHREIBUNG: Position
    WERTEBEREICH: {
      TopLeft,
      TopCenter,
      TopRight,
      MiddleLeft,
      MiddleCenter,
      MiddleRight,
      BottomLeft,
      BottomCenter,
      BottomRight,
      Custom { x: i32, y: i32 }
    }
    STANDARDWERT: TopRight
INVARIANTEN:
  - Keine
```

### 6.14 NotificationSortOrder

```
ENTITÄT: NotificationSortOrder
BESCHREIBUNG: Sortierreihenfolge von Benachrichtigungen
ATTRIBUTE:
  - NAME: order
    TYP: Enum
    BESCHREIBUNG: Reihenfolge
    WERTEBEREICH: {
      NewestFirst,
      OldestFirst,
      ByPriority,
      ByApplication,
      ByCategory
    }
    STANDARDWERT: NewestFirst
INVARIANTEN:
  - Keine
```

### 6.15 NotificationConfig

```
ENTITÄT: NotificationConfig
BESCHREIBUNG: Konfiguration für den NotificationManager
ATTRIBUTE:
  - NAME: center_config
    TYP: NotificationCenterConfig
    BESCHREIBUNG: Konfiguration für das Benachrichtigungszentrum
    WERTEBEREICH: Gültige NotificationCenterConfig
    STANDARDWERT: NotificationCenterConfig::default()
  - NAME: default_timeout
    TYP: u32
    BESCHREIBUNG: Standardtimeout für Benachrichtigungen in Millisekunden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 5000
  - NAME: default_urgency
    TYP: NotificationUrgency
    BESCHREIBUNG: Standarddringlichkeit für Benachrichtigungen
    WERTEBEREICH: Gültige NotificationUrgency
    STANDARDWERT: NotificationUrgency::Normal
  - NAME: max_actions
    TYP: u32
    BESCHREIBUNG: Maximale Anzahl von Aktionen pro Benachrichtigung
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 5
  - NAME: max_history
    TYP: u32
    BESCHREIBUNG: Maximale Anzahl von Benachrichtigungen im Verlauf
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 1000
  - NAME: history_retention_days
    TYP: u32
    BESCHREIBUNG: Anzahl der Tage, für die Benachrichtigungen im Verlauf gespeichert werden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 30
  - NAME: enable_sounds
    TYP: bool
    BESCHREIBUNG: Ob Töne für Benachrichtigungen aktiviert sind
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: enable_do_not_disturb_schedule
    TYP: bool
    BESCHREIBUNG: Ob ein Zeitplan für den Nicht stören-Modus aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: do_not_disturb_start_time
    TYP: Option<Time>
    BESCHREIBUNG: Startzeit für den Nicht stören-Modus
    WERTEBEREICH: Gültige Time oder None
    STANDARDWERT: None
  - NAME: do_not_disturb_end_time
    TYP: Option<Time>
    BESCHREIBUNG: Endzeit für den Nicht stören-Modus
    WERTEBEREICH: Gültige Time oder None
    STANDARDWERT: None
  - NAME: do_not_disturb_days
    TYP: Vec<DayOfWeek>
    BESCHREIBUNG: Tage, an denen der Nicht stören-Modus aktiv ist
    WERTEBEREICH: Gültige DayOfWeek-Werte
    STANDARDWERT: Leerer Vec
  - NAME: critical_bypass_do_not_disturb
    TYP: bool
    BESCHREIBUNG: Ob kritische Benachrichtigungen den Nicht stören-Modus umgehen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: config_dir
    TYP: PathBuf
    BESCHREIBUNG: Verzeichnis für Konfigurationsdateien
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: PathBuf::from("/etc/nova/notifications")
  - NAME: user_config_dir
    TYP: PathBuf
    BESCHREIBUNG: Verzeichnis für benutzerspezifische Konfigurationsdateien
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: PathBuf::from("~/.config/nova/notifications")
  - NAME: default_policies
    TYP: Vec<NotificationPolicy>
    BESCHREIBUNG: Standardrichtlinien für Benachrichtigungen
    WERTEBEREICH: Gültige NotificationPolicy-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - Keine
```

### 6.16 NotificationPolicy

```
ENTITÄT: NotificationPolicy
BESCHREIBUNG: Richtlinie für die Behandlung von Benachrichtigungen
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: app_name
    TYP: Option<String>
    BESCHREIBUNG: Name der Anwendung
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: category
    TYP: Option<NotificationCategory>
    BESCHREIBUNG: Kategorie
    WERTEBEREICH: Gültige NotificationCategory oder None
    STANDARDWERT: None
  - NAME: enabled
    TYP: bool
    BESCHREIBUNG: Ob Benachrichtigungen aktiviert sind
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: show_in_center
    TYP: bool
    BESCHREIBUNG: Ob Benachrichtigungen im Benachrichtigungszentrum angezeigt werden
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: show_as_popup
    TYP: bool
    BESCHREIBUNG: Ob Benachrichtigungen als Popup angezeigt werden
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: play_sound
    TYP: bool
    BESCHREIBUNG: Ob ein Ton abgespielt wird
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: sound
    TYP: Option<String>
    BESCHREIBUNG: Ton, der abgespielt wird
    WERTEBEREICH: Gültiger Tonpfad oder None
    STANDARDWERT: None
  - NAME: override_do_not_disturb
    TYP: bool
    BESCHREIBUNG: Ob der Nicht stören-Modus umgangen wird
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: timeout
    TYP: Option<u32>
    BESCHREIBUNG: Timeout in Millisekunden
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: urgency
    TYP: Option<NotificationUrgency>
    BESCHREIBUNG: Dringlichkeit
    WERTEBEREICH: Gültige NotificationUrgency oder None
    STANDARDWERT: None
  - NAME: position
    TYP: Option<NotificationPosition>
    BESCHREIBUNG: Position
    WERTEBEREICH: Gültige NotificationPosition oder None
    STANDARDWERT: None
INVARIANTEN:
  - id darf nicht leer sein
  - app_name und category können nicht beide None sein
```

### 6.17 Color

```
ENTITÄT: Color
BESCHREIBUNG: Farbe
ATTRIBUTE:
  - NAME: r
    TYP: u8
    BESCHREIBUNG: Rotwert
    WERTEBEREICH: [0, 255]
    STANDARDWERT: 0
  - NAME: g
    TYP: u8
    BESCHREIBUNG: Grünwert
    WERTEBEREICH: [0, 255]
    STANDARDWERT: 0
  - NAME: b
    TYP: u8
    BESCHREIBUNG: Blauwert
    WERTEBEREICH: [0, 255]
    STANDARDWERT: 0
  - NAME: a
    TYP: u8
    BESCHREIBUNG: Alphawert
    WERTEBEREICH: [0, 255]
    STANDARDWERT: 255
INVARIANTEN:
  - Keine
```

### 6.18 Time

```
ENTITÄT: Time
BESCHREIBUNG: Uhrzeit
ATTRIBUTE:
  - NAME: hour
    TYP: u8
    BESCHREIBUNG: Stunde
    WERTEBEREICH: [0, 23]
    STANDARDWERT: 0
  - NAME: minute
    TYP: u8
    BESCHREIBUNG: Minute
    WERTEBEREICH: [0, 59]
    STANDARDWERT: 0
  - NAME: second
    TYP: u8
    BESCHREIBUNG: Sekunde
    WERTEBEREICH: [0, 59]
    STANDARDWERT: 0
INVARIANTEN:
  - hour muss im Bereich [0, 23] liegen
  - minute muss im Bereich [0, 59] liegen
  - second muss im Bereich [0, 59] liegen
```

### 6.19 DayOfWeek

```
ENTITÄT: DayOfWeek
BESCHREIBUNG: Wochentag
ATTRIBUTE:
  - NAME: day
    TYP: Enum
    BESCHREIBUNG: Tag
    WERTEBEREICH: {
      Monday,
      Tuesday,
      Wednesday,
      Thursday,
      Friday,
      Saturday,
      Sunday
    }
    STANDARDWERT: Monday
INVARIANTEN:
  - Keine
```

### 6.20 NotificationHistory

```
ENTITÄT: NotificationHistory
BESCHREIBUNG: Verlauf von Benachrichtigungen
ATTRIBUTE:
  - NAME: items
    TYP: Vec<NotificationItem>
    BESCHREIBUNG: Benachrichtigungen
    WERTEBEREICH: Gültige NotificationItem-Werte
    STANDARDWERT: Leerer Vec
  - NAME: max_items
    TYP: u32
    BESCHREIBUNG: Maximale Anzahl von Benachrichtigungen
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 1000
  - NAME: retention_days
    TYP: u32
    BESCHREIBUNG: Anzahl der Tage, für die Benachrichtigungen gespeichert werden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 30
INVARIANTEN:
  - Keine
```

## 7. Verhaltensmodell

### 7.1 Benachrichtigungserstellung

```
ZUSTANDSAUTOMAT: NotificationCreation
BESCHREIBUNG: Prozess der Erstellung einer Benachrichtigung
ZUSTÄNDE:
  - NAME: Initial
    BESCHREIBUNG: Initialer Zustand
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: ValidatingParams
    BESCHREIBUNG: Parameter werden validiert
    EINTRITTSAKTIONEN: Parameter prüfen
    AUSTRITTSAKTIONEN: Keine
  - NAME: CheckingPolicy
    BESCHREIBUNG: Richtlinie wird geprüft
    EINTRITTSAKTIONEN: Richtlinie abrufen
    AUSTRITTSAKTIONEN: Keine
  - NAME: CheckingDoNotDisturb
    BESCHREIBUNG: Nicht stören-Modus wird geprüft
    EINTRITTSAKTIONEN: Nicht stören-Status abrufen
    AUSTRITTSAKTIONEN: Keine
  - NAME: CreatingNotification
    BESCHREIBUNG: Benachrichtigung wird erstellt
    EINTRITTSAKTIONEN: ID generieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: StoringNotification
    BESCHREIBUNG: Benachrichtigung wird gespeichert
    EINTRITTSAKTIONEN: Benachrichtigung in Speicher ablegen
    AUSTRITTSAKTIONEN: Keine
  - NAME: NotifyingListeners
    BESCHREIBUNG: Listener werden benachrichtigt
    EINTRITTSAKTIONEN: Listener-Liste durchlaufen
    AUSTRITTSAKTIONEN: Keine
  - NAME: DisplayingNotification
    BESCHREIBUNG: Benachrichtigung wird angezeigt
    EINTRITTSAKTIONEN: Renderer aktivieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: PlayingSound
    BESCHREIBUNG: Ton wird abgespielt
    EINTRITTSAKTIONEN: Sound-Player aktivieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Completed
    BESCHREIBUNG: Erstellung abgeschlossen
    EINTRITTSAKTIONEN: Statistiken aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler bei der Erstellung
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Initial
    NACH: ValidatingParams
    EREIGNIS: create_notification aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ValidatingParams
    NACH: CheckingPolicy
    EREIGNIS: Parameter erfolgreich validiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ValidatingParams
    NACH: Error
    EREIGNIS: Ungültige Parameter
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
  - VON: CheckingPolicy
    NACH: CheckingDoNotDisturb
    EREIGNIS: Richtlinie erlaubt Benachrichtigung
    BEDINGUNG: policy.enabled == true
    AKTIONEN: Keine
  - VON: CheckingPolicy
    NACH: Completed
    EREIGNIS: Richtlinie blockiert Benachrichtigung
    BEDINGUNG: policy.enabled == false
    AKTIONEN: Keine
  - VON: CheckingDoNotDisturb
    NACH: CreatingNotification
    EREIGNIS: Nicht stören-Modus ist deaktiviert oder wird umgangen
    BEDINGUNG: !is_do_not_disturb_enabled() || policy.override_do_not_disturb || urgency == NotificationUrgency::Critical && config.critical_bypass_do_not_disturb
    AKTIONEN: Keine
  - VON: CheckingDoNotDisturb
    NACH: CreatingNotification
    EREIGNIS: Nicht stören-Modus ist aktiviert und wird nicht umgangen
    BEDINGUNG: is_do_not_disturb_enabled() && !policy.override_do_not_disturb && !(urgency == NotificationUrgency::Critical && config.critical_bypass_do_not_disturb)
    AKTIONEN: show_as_popup = false, play_sound = false
  - VON: CreatingNotification
    NACH: StoringNotification
    EREIGNIS: Benachrichtigung erfolgreich erstellt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: CreatingNotification
    NACH: Error
    EREIGNIS: Fehler bei der Erstellung der Benachrichtigung
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
  - VON: StoringNotification
    NACH: NotifyingListeners
    EREIGNIS: Benachrichtigung erfolgreich gespeichert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: StoringNotification
    NACH: Error
    EREIGNIS: Fehler beim Speichern der Benachrichtigung
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
  - VON: NotifyingListeners
    NACH: DisplayingNotification
    EREIGNIS: Listener erfolgreich benachrichtigt
    BEDINGUNG: policy.show_as_popup
    AKTIONEN: Keine
  - VON: NotifyingListeners
    NACH: PlayingSound
    EREIGNIS: Listener erfolgreich benachrichtigt
    BEDINGUNG: !policy.show_as_popup && policy.play_sound
    AKTIONEN: Keine
  - VON: NotifyingListeners
    NACH: Completed
    EREIGNIS: Listener erfolgreich benachrichtigt
    BEDINGUNG: !policy.show_as_popup && !policy.play_sound
    AKTIONEN: Keine
  - VON: DisplayingNotification
    NACH: PlayingSound
    EREIGNIS: Benachrichtigung erfolgreich angezeigt
    BEDINGUNG: policy.play_sound
    AKTIONEN: Keine
  - VON: DisplayingNotification
    NACH: Completed
    EREIGNIS: Benachrichtigung erfolgreich angezeigt
    BEDINGUNG: !policy.play_sound
    AKTIONEN: Keine
  - VON: DisplayingNotification
    NACH: Error
    EREIGNIS: Fehler beim Anzeigen der Benachrichtigung
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
  - VON: PlayingSound
    NACH: Completed
    EREIGNIS: Ton erfolgreich abgespielt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: PlayingSound
    NACH: Error
    EREIGNIS: Fehler beim Abspielen des Tons
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
INITIALZUSTAND: Initial
ENDZUSTÄNDE: [Completed, Error]
```

### 7.2 Benachrichtigungsschließung

```
ZUSTANDSAUTOMAT: NotificationClosure
BESCHREIBUNG: Prozess des Schließens einer Benachrichtigung
ZUSTÄNDE:
  - NAME: Initial
    BESCHREIBUNG: Initialer Zustand
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: ValidatingId
    BESCHREIBUNG: ID wird validiert
    EINTRITTSAKTIONEN: ID prüfen
    AUSTRITTSAKTIONEN: Keine
  - NAME: RetrievingNotification
    BESCHREIBUNG: Benachrichtigung wird abgerufen
    EINTRITTSAKTIONEN: Benachrichtigung suchen
    AUSTRITTSAKTIONEN: Keine
  - NAME: ClosingNotification
    BESCHREIBUNG: Benachrichtigung wird geschlossen
    EINTRITTSAKTIONEN: Schließzeitpunkt setzen
    AUSTRITTSAKTIONEN: Keine
  - NAME: HidingNotification
    BESCHREIBUNG: Benachrichtigung wird ausgeblendet
    EINTRITTSAKTIONEN: Renderer benachrichtigen
    AUSTRITTSAKTIONEN: Keine
  - NAME: CheckingResidency
    BESCHREIBUNG: Residenz wird geprüft
    EINTRITTSAKTIONEN: Residenzstatus prüfen
    AUSTRITTSAKTIONEN: Keine
  - NAME: RemovingFromCenter
    BESCHREIBUNG: Benachrichtigung wird aus dem Benachrichtigungszentrum entfernt
    EINTRITTSAKTIONEN: Aus Zentrum entfernen
    AUSTRITTSAKTIONEN: Keine
  - NAME: AddingToHistory
    BESCHREIBUNG: Benachrichtigung wird zum Verlauf hinzugefügt
    EINTRITTSAKTIONEN: Zum Verlauf hinzufügen
    AUSTRITTSAKTIONEN: Keine
  - NAME: NotifyingListeners
    BESCHREIBUNG: Listener werden benachrichtigt
    EINTRITTSAKTIONEN: Listener-Liste durchlaufen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Completed
    BESCHREIBUNG: Schließung abgeschlossen
    EINTRITTSAKTIONEN: Statistiken aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler bei der Schließung
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Initial
    NACH: ValidatingId
    EREIGNIS: close_notification aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ValidatingId
    NACH: RetrievingNotification
    EREIGNIS: ID erfolgreich validiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ValidatingId
    NACH: Error
    EREIGNIS: Ungültige ID
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
  - VON: RetrievingNotification
    NACH: ClosingNotification
    EREIGNIS: Benachrichtigung erfolgreich abgerufen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: RetrievingNotification
    NACH: Error
    EREIGNIS: Benachrichtigung nicht gefunden
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
  - VON: ClosingNotification
    NACH: HidingNotification
    EREIGNIS: Benachrichtigung erfolgreich geschlossen
    BEDINGUNG: notification.displayed
    AKTIONEN: Keine
  - VON: ClosingNotification
    NACH: CheckingResidency
    EREIGNIS: Benachrichtigung erfolgreich geschlossen
    BEDINGUNG: !notification.displayed
    AKTIONEN: Keine
  - VON: ClosingNotification
    NACH: Error
    EREIGNIS: Fehler beim Schließen der Benachrichtigung
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
  - VON: HidingNotification
    NACH: CheckingResidency
    EREIGNIS: Benachrichtigung erfolgreich ausgeblendet
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: HidingNotification
    NACH: Error
    EREIGNIS: Fehler beim Ausblenden der Benachrichtigung
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
  - VON: CheckingResidency
    NACH: RemovingFromCenter
    EREIGNIS: Benachrichtigung ist nicht resident
    BEDINGUNG: !notification.params.resident
    AKTIONEN: Keine
  - VON: CheckingResidency
    NACH: AddingToHistory
    EREIGNIS: Benachrichtigung ist resident
    BEDINGUNG: notification.params.resident
    AKTIONEN: Keine
  - VON: RemovingFromCenter
    NACH: AddingToHistory
    EREIGNIS: Benachrichtigung erfolgreich aus dem Benachrichtigungszentrum entfernt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: RemovingFromCenter
    NACH: Error
    EREIGNIS: Fehler beim Entfernen der Benachrichtigung aus dem Benachrichtigungszentrum
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
  - VON: AddingToHistory
    NACH: NotifyingListeners
    EREIGNIS: Benachrichtigung erfolgreich zum Verlauf hinzugefügt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: AddingToHistory
    NACH: Error
    EREIGNIS: Fehler beim Hinzufügen der Benachrichtigung zum Verlauf
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
  - VON: NotifyingListeners
    NACH: Completed
    EREIGNIS: Listener erfolgreich benachrichtigt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: NotifyingListeners
    NACH: Error
    EREIGNIS: Fehler bei der Benachrichtigung der Listener
    BEDINGUNG: Keine
    AKTIONEN: NotificationError erstellen
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
ENTITÄT: NotificationError
BESCHREIBUNG: Fehler im Benachrichtigungsmanager-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      InvalidParams { message: String },
      NotificationNotFound { id: NotificationId },
      PolicyError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      RenderError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      SoundError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      StorageError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      DBusError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      ListenerError { message: String },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

## 9. Leistungsanforderungen

### 9.1 Allgemeine Leistungsanforderungen

1. Das Benachrichtigungsmanager-Modul MUSS effizient mit Ressourcen umgehen.
2. Das Benachrichtigungsmanager-Modul MUSS eine geringe Latenz haben.
3. Das Benachrichtigungsmanager-Modul MUSS skalierbar sein.

### 9.2 Spezifische Leistungsanforderungen

1. Die Erstellung einer Benachrichtigung MUSS in unter 50ms abgeschlossen sein.
2. Die Anzeige einer Benachrichtigung MUSS in unter 100ms abgeschlossen sein.
3. Das Schließen einer Benachrichtigung MUSS in unter 50ms abgeschlossen sein.
4. Das Benachrichtigungsmanager-Modul MUSS mit mindestens 100 gleichzeitigen Benachrichtigungen umgehen können.
5. Das Benachrichtigungsmanager-Modul MUSS mit mindestens 1000 Benachrichtigungen im Verlauf umgehen können.
6. Das Benachrichtigungsmanager-Modul DARF nicht mehr als 2% CPU-Auslastung im Leerlauf verursachen.
7. Das Benachrichtigungsmanager-Modul DARF nicht mehr als 50MB Speicher verbrauchen.

## 10. Sicherheitsanforderungen

### 10.1 Allgemeine Sicherheitsanforderungen

1. Das Benachrichtigungsmanager-Modul MUSS memory-safe sein.
2. Das Benachrichtigungsmanager-Modul MUSS thread-safe sein.
3. Das Benachrichtigungsmanager-Modul MUSS robust gegen Fehleingaben sein.

### 10.2 Spezifische Sicherheitsanforderungen

1. Das Benachrichtigungsmanager-Modul MUSS Eingaben validieren, um Injection-Angriffe zu verhindern.
2. Das Benachrichtigungsmanager-Modul MUSS Zugriffskontrollen für Benachrichtigungsoperationen implementieren.
3. Das Benachrichtigungsmanager-Modul MUSS sichere Standardwerte verwenden.
4. Das Benachrichtigungsmanager-Modul MUSS Ressourcenlimits implementieren, um Denial-of-Service-Angriffe zu verhindern.
5. Das Benachrichtigungsmanager-Modul MUSS verhindern, dass Benachrichtigungen auf geschützte Bereiche des Bildschirms zugreifen.
6. Das Benachrichtigungsmanager-Modul MUSS verhindern, dass Benachrichtigungen Eingaben von anderen Anwendungen abfangen.
7. Das Benachrichtigungsmanager-Modul MUSS die Ausführung von Code in Benachrichtigungen einschränken.
8. Das Benachrichtigungsmanager-Modul MUSS die Kommunikation zwischen Benachrichtigungen einschränken.

## 11. Testkriterien

### 11.1 Allgemeine Testkriterien

1. Jede Komponente MUSS Einheitstests haben.
2. Jede öffentliche Funktion MUSS getestet sein.
3. Jeder Fehlerfall MUSS getestet sein.

### 11.2 Spezifische Testkriterien

1. Das Benachrichtigungsmanager-Modul MUSS mit verschiedenen Benachrichtigungsparametern getestet sein.
2. Das Benachrichtigungsmanager-Modul MUSS mit verschiedenen Benachrichtigungsrichtlinien getestet sein.
3. Das Benachrichtigungsmanager-Modul MUSS mit verschiedenen Nicht stören-Modi getestet sein.
4. Das Benachrichtigungsmanager-Modul MUSS mit verschiedenen Benachrichtigungskategorien getestet sein.
5. Das Benachrichtigungsmanager-Modul MUSS mit verschiedenen Benachrichtigungsaktionen getestet sein.
6. Das Benachrichtigungsmanager-Modul MUSS mit verschiedenen Benachrichtigungspositionen getestet sein.
7. Das Benachrichtigungsmanager-Modul MUSS mit verschiedenen Fehlerszenarien getestet sein.
8. Das Benachrichtigungsmanager-Modul MUSS mit verschiedenen Benutzerinteraktionen getestet sein.

## 12. Anhänge

### 12.1 Referenzierte Dokumente

1. SPEC-ROOT-v1.0.0: NovaDE Spezifikationswurzel
2. SPEC-LAYER-CORE-v1.0.0: Spezifikation der Kernschicht
3. SPEC-LAYER-SYSTEM-v1.0.0: Spezifikation der Systemschicht
4. SPEC-MODULE-DOMAIN-THEMING-v1.0.0: Spezifikation des Theming-Moduls

### 12.2 Externe Abhängigkeiten

1. `dbus`: Für die D-Bus-Integration
2. `gtk4`: Für die GUI-Komponenten
3. `cairo`: Für die Grafikausgabe
4. `pango`: Für die Textdarstellung
5. `json`: Für die Konfigurationsdateien
