# SPEC-LAYER-DOMAIN-v1.0.0: NovaDE Domänenschicht-Spezifikation (Teil 2)

## 6. Datenmodell (Fortsetzung)

### 6.3 Benachrichtigungstypen

```
ENTITÄT: Notification
BESCHREIBUNG: Kernentität für Benachrichtigungen
ATTRIBUTE:
  - NAME: id
    TYP: Uuid
    BESCHREIBUNG: Eindeutige ID der Benachrichtigung
    WERTEBEREICH: Gültige UUID
    STANDARDWERT: Keiner
  - NAME: app_id
    TYP: ApplicationId
    BESCHREIBUNG: ID der sendenden Anwendung
    WERTEBEREICH: Gültige ApplicationId
    STANDARDWERT: Keiner
  - NAME: title
    TYP: String
    BESCHREIBUNG: Titel der Benachrichtigung
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: body
    TYP: String
    BESCHREIBUNG: Haupttext der Benachrichtigung
    WERTEBEREICH: Zeichenkette
    STANDARDWERT: ""
  - NAME: icon
    TYP: Option<NotificationIcon>
    BESCHREIBUNG: Symbol der Benachrichtigung
    WERTEBEREICH: Gültige NotificationIcon oder None
    STANDARDWERT: None
  - NAME: actions
    TYP: Vec<NotificationAction>
    BESCHREIBUNG: Verfügbare Aktionen für die Benachrichtigung
    WERTEBEREICH: Gültige NotificationAction-Werte
    STANDARDWERT: Leerer Vec
  - NAME: hints
    TYP: HashMap<String, NotificationHint>
    BESCHREIBUNG: Zusätzliche Hinweise für die Benachrichtigung
    WERTEBEREICH: Gültige String-NotificationHint-Paare
    STANDARDWERT: Leere HashMap
  - NAME: urgency
    TYP: NotificationUrgency
    BESCHREIBUNG: Dringlichkeit der Benachrichtigung
    WERTEBEREICH: {Low, Normal, Critical}
    STANDARDWERT: NotificationUrgency::Normal
  - NAME: creation_time
    TYP: DateTime<Utc>
    BESCHREIBUNG: Erstellungszeit der Benachrichtigung
    WERTEBEREICH: Gültige DateTime<Utc>
    STANDARDWERT: Keiner
  - NAME: expiration_time
    TYP: Option<DateTime<Utc>>
    BESCHREIBUNG: Ablaufzeit der Benachrichtigung
    WERTEBEREICH: Gültige DateTime<Utc> oder None
    STANDARDWERT: None
  - NAME: read_status
    TYP: NotificationReadStatus
    BESCHREIBUNG: Lesestatus der Benachrichtigung
    WERTEBEREICH: {Unread, Read}
    STANDARDWERT: NotificationReadStatus::Unread
  - NAME: close_reason
    TYP: Option<NotificationCloseReason>
    BESCHREIBUNG: Grund für das Schließen der Benachrichtigung
    WERTEBEREICH: Gültige NotificationCloseReason oder None
    STANDARDWERT: None
INVARIANTEN:
  - id muss gültig sein
  - app_id muss gültig sein
  - title darf nicht leer sein
  - creation_time muss gültig sein
  - Wenn expiration_time vorhanden ist, muss es nach creation_time liegen
```

```
ENTITÄT: NotificationAction
BESCHREIBUNG: Aktion innerhalb einer Benachrichtigung
ATTRIBUTE:
  - NAME: key
    TYP: String
    BESCHREIBUNG: Eindeutiger Schlüssel der Aktion
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: label
    TYP: String
    BESCHREIBUNG: Anzeigetext der Aktion
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: icon
    TYP: Option<String>
    BESCHREIBUNG: Optionales Symbol für die Aktion
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
INVARIANTEN:
  - key darf nicht leer sein
  - label darf nicht leer sein
```

```
ENTITÄT: NotificationUrgency
BESCHREIBUNG: Dringlichkeitsstufen für Benachrichtigungen
ATTRIBUTE:
  - NAME: value
    TYP: Enum
    BESCHREIBUNG: Dringlichkeitswert
    WERTEBEREICH: {Low, Normal, Critical}
    STANDARDWERT: Normal
```

```
ENTITÄT: NotificationCloseReason
BESCHREIBUNG: Gründe für das Schließen einer Benachrichtigung
ATTRIBUTE:
  - NAME: value
    TYP: Enum
    BESCHREIBUNG: Schließungsgrund
    WERTEBEREICH: {
      Expired,
      Dismissed,
      ActionInvoked,
      AppClosed,
      UserSessionEnded,
      RuleApplied
    }
    STANDARDWERT: Keiner
```

### 6.4 Benachrichtigungsregeltypen

```
ENTITÄT: RuleCondition
BESCHREIBUNG: Bedingungen für Benachrichtigungsregeln
ATTRIBUTE:
  - NAME: condition_type
    TYP: Enum
    BESCHREIBUNG: Typ der Bedingung
    WERTEBEREICH: {
      AppIdEquals(String),
      AppIdMatches(Regex),
      TitleContains(String),
      TitleMatches(Regex),
      BodyContains(String),
      BodyMatches(Regex),
      UrgencyIs(NotificationUrgency),
      TimeIsBetween(TimeRange),
      HasAction(String),
      HasHint(String),
      And(Vec<RuleCondition>),
      Or(Vec<RuleCondition>),
      Not(Box<RuleCondition>)
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Bei And und Or müssen mindestens zwei Bedingungen vorhanden sein
  - Bei Not muss genau eine Bedingung vorhanden sein
```

```
ENTITÄT: RuleAction
BESCHREIBUNG: Aktionen für Benachrichtigungsregeln
ATTRIBUTE:
  - NAME: action_type
    TYP: Enum
    BESCHREIBUNG: Typ der Aktion
    WERTEBEREICH: {
      Suppress,
      ChangeUrgency(NotificationUrgency),
      AddTag(String),
      RemoveTag(String),
      MoveToCategory(String),
      SetExpirationTime(Duration),
      PlaySound(String),
      CustomAction(String, HashMap<String, String>)
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Bei CustomAction muss der erste String nicht leer sein
```

```
ENTITÄT: NotificationRule
BESCHREIBUNG: Vollständige Regel für Benachrichtigungen
ATTRIBUTE:
  - NAME: id
    TYP: Uuid
    BESCHREIBUNG: Eindeutige ID der Regel
    WERTEBEREICH: Gültige UUID
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name der Regel
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: enabled
    TYP: bool
    BESCHREIBUNG: Ob die Regel aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: condition
    TYP: RuleCondition
    BESCHREIBUNG: Bedingung der Regel
    WERTEBEREICH: Gültige RuleCondition
    STANDARDWERT: Keiner
  - NAME: actions
    TYP: Vec<RuleAction>
    BESCHREIBUNG: Aktionen der Regel
    WERTEBEREICH: Gültige RuleAction-Werte
    STANDARDWERT: Keiner
  - NAME: priority
    TYP: i32
    BESCHREIBUNG: Priorität der Regel (höhere Werte haben Vorrang)
    WERTEBEREICH: Ganzzahlen
    STANDARDWERT: 0
  - NAME: stop_processing
    TYP: bool
    BESCHREIBUNG: Ob die Regelverarbeitung nach dieser Regel gestoppt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
INVARIANTEN:
  - id muss gültig sein
  - name darf nicht leer sein
  - condition muss gültig sein
  - actions darf nicht leer sein
```

### 6.5 Globale Einstellungstypen

```
ENTITÄT: GlobalDesktopSettings
BESCHREIBUNG: Wurzelstruktur für Desktop-weite Einstellungen
ATTRIBUTE:
  - NAME: appearance
    TYP: AppearanceSettings
    BESCHREIBUNG: Einstellungen für das Erscheinungsbild
    WERTEBEREICH: Gültige AppearanceSettings
    STANDARDWERT: AppearanceSettings::default()
  - NAME: input
    TYP: InputSettings
    BESCHREIBUNG: Einstellungen für Eingabegeräte
    WERTEBEREICH: Gültige InputSettings
    STANDARDWERT: InputSettings::default()
  - NAME: power
    TYP: PowerSettings
    BESCHREIBUNG: Einstellungen für die Energieverwaltung
    WERTEBEREICH: Gültige PowerSettings
    STANDARDWERT: PowerSettings::default()
  - NAME: privacy
    TYP: PrivacySettings
    BESCHREIBUNG: Einstellungen für den Datenschutz
    WERTEBEREICH: Gültige PrivacySettings
    STANDARDWERT: PrivacySettings::default()
  - NAME: notifications
    TYP: NotificationSettings
    BESCHREIBUNG: Einstellungen für Benachrichtigungen
    WERTEBEREICH: Gültige NotificationSettings
    STANDARDWERT: NotificationSettings::default()
  - NAME: window_management
    TYP: WindowManagementSettings
    BESCHREIBUNG: Einstellungen für die Fensterverwaltung
    WERTEBEREICH: Gültige WindowManagementSettings
    STANDARDWERT: WindowManagementSettings::default()
  - NAME: accessibility
    TYP: AccessibilitySettings
    BESCHREIBUNG: Einstellungen für die Barrierefreiheit
    WERTEBEREICH: Gültige AccessibilitySettings
    STANDARDWERT: AccessibilitySettings::default()
  - NAME: ai_features
    TYP: AIFeatureSettings
    BESCHREIBUNG: Einstellungen für KI-Funktionen
    WERTEBEREICH: Gültige AIFeatureSettings
    STANDARDWERT: AIFeatureSettings::default()
INVARIANTEN:
  - Alle Untereinstellungen müssen gültig sein
```

```
ENTITÄT: SettingPath
BESCHREIBUNG: Hierarchischer Pfad zu einer Einstellung
ATTRIBUTE:
  - NAME: path_segments
    TYP: Vec<String>
    BESCHREIBUNG: Pfadsegmente
    WERTEBEREICH: Nicht-leere Zeichenketten
    STANDARDWERT: Keiner
INVARIANTEN:
  - path_segments darf nicht leer sein
  - Kein Segment darf leer sein
```

## 7. Verhaltensmodell

### 7.1 Theming-Anwendung

```
ZUSTANDSAUTOMAT: ThemeApplication
BESCHREIBUNG: Prozess der Themenanwendung
ZUSTÄNDE:
  - NAME: Uninitialized
    BESCHREIBUNG: Theming-Engine ist nicht initialisiert
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: Loading
    BESCHREIBUNG: Themes und Tokens werden geladen
    EINTRITTSAKTIONEN: Dateisystem nach Themes und Tokens durchsuchen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Resolving
    BESCHREIBUNG: Token-Referenzen werden aufgelöst
    EINTRITTSAKTIONEN: Token-Auflösungspipeline starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: Ready
    BESCHREIBUNG: Theme ist angewendet und bereit
    EINTRITTSAKTIONEN: ThemeChangedEvent auslösen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler bei der Themenanwendung
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Uninitialized
    NACH: Loading
    EREIGNIS: Initialisierung der Theming-Engine
    BEDINGUNG: Keine
    AKTIONEN: Konfiguration laden
  - VON: Loading
    NACH: Resolving
    EREIGNIS: Themes und Tokens erfolgreich geladen
    BEDINGUNG: Keine
    AKTIONEN: Token-Referenzen sammeln
  - VON: Loading
    NACH: Error
    EREIGNIS: Fehler beim Laden
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
  - VON: Resolving
    NACH: Ready
    EREIGNIS: Token-Referenzen erfolgreich aufgelöst
    BEDINGUNG: Keine
    AKTIONEN: AppliedThemeState erstellen
  - VON: Resolving
    NACH: Error
    EREIGNIS: Fehler bei der Auflösung
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
  - VON: Ready
    NACH: Loading
    EREIGNIS: reload_themes_and_tokens aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Dateisystem nach Themes und Tokens durchsuchen
  - VON: Ready
    NACH: Resolving
    EREIGNIS: update_configuration aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Token-Referenzen neu sammeln
  - VON: Error
    NACH: Loading
    EREIGNIS: reload_themes_and_tokens aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Dateisystem nach Themes und Tokens durchsuchen
INITIALZUSTAND: Uninitialized
ENDZUSTÄNDE: [Ready, Error]
```

### 7.2 Workspace-Verwaltung

```
ZUSTANDSAUTOMAT: WorkspaceManagement
BESCHREIBUNG: Prozess der Workspace-Verwaltung
ZUSTÄNDE:
  - NAME: Uninitialized
    BESCHREIBUNG: Workspace-Manager ist nicht initialisiert
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: Loading
    BESCHREIBUNG: Workspace-Konfiguration wird geladen
    EINTRITTSAKTIONEN: Konfigurationsdateien öffnen
    AUSTRITTSAKTIONEN: Dateien schließen
  - NAME: Ready
    BESCHREIBUNG: Workspace-Manager ist bereit
    EINTRITTSAKTIONEN: WorkspaceLoadedEvent auslösen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Modifying
    BESCHREIBUNG: Workspaces werden modifiziert
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: Saving
    BESCHREIBUNG: Workspace-Konfiguration wird gespeichert
    EINTRITTSAKTIONEN: Konfigurationsdateien öffnen
    AUSTRITTSAKTIONEN: Dateien schließen
  - NAME: Error
    BESCHREIBUNG: Fehler bei der Workspace-Verwaltung
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Uninitialized
    NACH: Loading
    EREIGNIS: Initialisierung des Workspace-Managers
    BEDINGUNG: Keine
    AKTIONEN: Konfigurationspfade prüfen
  - VON: Loading
    NACH: Ready
    EREIGNIS: Konfiguration erfolgreich geladen
    BEDINGUNG: Keine
    AKTIONEN: Workspaces erstellen
  - VON: Loading
    NACH: Error
    EREIGNIS: Fehler beim Laden
    BEDINGUNG: Keine
    AKTIONEN: WorkspaceManagerError erstellen
  - VON: Ready
    NACH: Modifying
    EREIGNIS: Workspace-Modifikation angefordert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: Modifying
    NACH: Ready
    EREIGNIS: Modifikation abgeschlossen
    BEDINGUNG: Keine
    AKTIONEN: Entsprechendes WorkspaceEvent auslösen
  - VON: Modifying
    NACH: Error
    EREIGNIS: Fehler bei der Modifikation
    BEDINGUNG: Keine
    AKTIONEN: WorkspaceManagerError erstellen
  - VON: Ready
    NACH: Saving
    EREIGNIS: save_configuration aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Konfiguration serialisieren
  - VON: Saving
    NACH: Ready
    EREIGNIS: Konfiguration erfolgreich gespeichert
    BEDINGUNG: Keine
    AKTIONEN: WorkspaceConfigSavedEvent auslösen
  - VON: Saving
    NACH: Error
    EREIGNIS: Fehler beim Speichern
    BEDINGUNG: Keine
    AKTIONEN: WorkspaceManagerError erstellen
  - VON: Error
    NACH: Loading
    EREIGNIS: Wiederherstellungsversuch
    BEDINGUNG: Keine
    AKTIONEN: Konfigurationspfade prüfen
INITIALZUSTAND: Uninitialized
ENDZUSTÄNDE: [Ready, Error]
```

## 8. Fehlerbehandlung

### 8.1 Fehlerbehandlungsstrategie

1. Alle Fehler MÜSSEN über spezifische Fehlertypen zurückgegeben werden.
2. Fehlertypen MÜSSEN mit `thiserror` definiert werden.
3. Fehler MÜSSEN kontextuelle Informationen enthalten.
4. Fehlerketten MÜSSEN bei der Weitergabe oder beim Wrappen von Fehlern erhalten bleiben.
5. Panics sind VERBOTEN, außer in Fällen, die explizit dokumentiert sind.

### 8.2 Domänenspezifische Fehlertypen

```
ENTITÄT: ThemingError
BESCHREIBUNG: Fehler im Theming-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      TokenLoadError { path: PathBuf, source: std::io::Error },
      TokenParseError { path: PathBuf, source: serde_json::Error },
      ThemeLoadError { path: PathBuf, source: std::io::Error },
      ThemeParseError { path: PathBuf, source: serde_json::Error },
      CircularTokenReference { token_path: Vec<TokenIdentifier> },
      UnresolvedTokenReference { reference: TokenIdentifier },
      InvalidTokenValue { token: TokenIdentifier, expected_type: String, actual_type: String },
      ThemeNotFound { theme_id: String },
      ConfigurationError { message: String, source: Option<Box<dyn std::error::Error>> },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

```
ENTITÄT: WorkspaceManagerError
BESCHREIBUNG: Fehler im Workspace-Management-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      WorkspaceNotFound { id: WorkspaceId },
      WindowNotFound { id: WindowIdentifier },
      ConfigLoadError { path: PathBuf, source: std::io::Error },
      ConfigParseError { path: PathBuf, source: serde_json::Error },
      ConfigSaveError { path: PathBuf, source: std::io::Error },
      InvalidWorkspaceOperation { operation: String, reason: String },
      CannotDeleteLastWorkspace,
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

```
ENTITÄT: NotificationError
BESCHREIBUNG: Fehler im Benachrichtigungsmodul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      NotificationNotFound { id: Uuid },
      InvalidNotificationData { reason: String },
      StorageError { operation: String, source: Option<Box<dyn std::error::Error>> },
      ActionNotFound { notification_id: Uuid, action_key: String },
      PermissionDenied { reason: String },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

```
ENTITÄT: NotificationRulesError
BESCHREIBUNG: Fehler im Benachrichtigungsregelmodul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      RuleNotFound { id: Uuid },
      InvalidRuleCondition { reason: String },
      InvalidRuleAction { reason: String },
      RuleStorageError { operation: String, source: Option<Box<dyn std::error::Error>> },
      RegexCompilationError { pattern: String, source: regex::Error },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

```
ENTITÄT: GlobalSettingsError
BESCHREIBUNG: Fehler im globalen Einstellungsmodul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      SettingNotFound { path: SettingPath },
      InvalidSettingValue { path: SettingPath, expected_type: String, actual_type: String },
      SettingStorageError { operation: String, source: Option<Box<dyn std::error::Error>> },
      InvalidSettingPath { path: String, reason: String },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

## 9. Leistungsanforderungen

### 9.1 Allgemeine Leistungsanforderungen

1. Die Domänenschicht MUSS effizient mit Ressourcen umgehen.
2. Die Domänenschicht MUSS für interaktive Anwendungen geeignet sein.
3. Die Domänenschicht MUSS asynchrone Operationen unterstützen, um Blockierungen zu vermeiden.

### 9.2 Spezifische Leistungsanforderungen

1. Die Themenanwendung MUSS in unter 100ms abgeschlossen sein.
2. Die Workspace-Umschaltung MUSS in unter 50ms abgeschlossen sein.
3. Die Benachrichtigungsverarbeitung MUSS in unter 10ms abgeschlossen sein.
4. Die Einstellungsaktualisierung MUSS in unter 20ms abgeschlossen sein.
5. Die Ereignisverteilung MUSS in unter 5ms abgeschlossen sein.

## 10. Sicherheitsanforderungen

### 10.1 Allgemeine Sicherheitsanforderungen

1. Die Domänenschicht MUSS memory-safe sein.
2. Die Domänenschicht MUSS thread-safe sein.
3. Die Domänenschicht MUSS robust gegen Fehleingaben sein.

### 10.2 Spezifische Sicherheitsanforderungen

1. Die Benachrichtigungsverarbeitung MUSS Eingaben validieren, um Injection-Angriffe zu verhindern.
2. Die Einstellungsverwaltung MUSS Zugriffskontrollen implementieren.
3. Die KI-Interaktion MUSS explizite Benutzereinwilligung erfordern.
4. Die Persistenz MUSS Daten sicher speichern und vor unbefugtem Zugriff schützen.

## 11. Testkriterien

### 11.1 Allgemeine Testkriterien

1. Jedes Modul MUSS Einheitstests haben.
2. Jede öffentliche Funktion MUSS getestet sein.
3. Jeder Fehlerfall MUSS getestet sein.
4. Asynchrone Operationen MÜSSEN mit verschiedenen Timing-Szenarien getestet sein.

### 11.2 Spezifische Testkriterien

1. Die Themenanwendung MUSS mit verschiedenen Theme-Definitionen getestet sein.
2. Die Workspace-Verwaltung MUSS mit verschiedenen Fensterkonstellationen getestet sein.
3. Die Benachrichtigungsverarbeitung MUSS mit verschiedenen Benachrichtigungstypen getestet sein.
4. Die Regelanwendung MUSS mit komplexen Bedingungen getestet sein.
5. Die Einstellungsverwaltung MUSS mit verschiedenen Einstellungspfaden getestet sein.

## 12. Anhänge

### 12.1 Referenzierte Dokumente

1. SPEC-ROOT-v1.0.0: NovaDE Spezifikationswurzel
2. SPEC-LAYER-CORE-v1.0.0: Spezifikation der Kernschicht
3. SPEC-MODULE-DOMAIN-THEMING-v1.0.0: Spezifikation des Theming-Moduls
4. SPEC-MODULE-DOMAIN-WORKSPACES-v1.0.0: Spezifikation des Workspace-Moduls
5. SPEC-MODULE-DOMAIN-NOTIFICATIONS-v1.0.0: Spezifikation des Benachrichtigungsmoduls
6. SPEC-MODULE-DOMAIN-SETTINGS-v1.0.0: Spezifikation des Einstellungsmoduls
7. SPEC-MODULE-DOMAIN-WINDOW-POLICY-v1.0.0: Spezifikation des Fensterrichtlinienmoduls

### 12.2 Externe Abhängigkeiten

1. `async_trait`: Für asynchrone Trait-Definitionen
2. `tokio`: Für asynchrone Laufzeitunterstützung
3. `serde`: Für Serialisierung/Deserialisierung
4. `uuid`: Für eindeutige Identifikatoren
5. `chrono`: Für Zeitstempel und Zeitoperationen
6. `regex`: Für reguläre Ausdrücke in Benachrichtigungsregeln
