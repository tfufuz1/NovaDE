# SPEC-LAYER-DOMAIN-v1.0.0: NovaDE Domänenschicht-Spezifikation (Teil 1)

```
SPEZIFIKATION: SPEC-LAYER-DOMAIN-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0, SPEC-LAYER-CORE-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Diese Spezifikation definiert die Domänenschicht (Domain Layer) des NovaDE-Projekts. Die Domänenschicht kapselt die Geschäftslogik und den Kernzustand der Desktop-Umgebung und interagiert nicht direkt mit Hardware oder OS-Details. Der Geltungsbereich umfasst alle Module, Komponenten und Schnittstellen der Domänenschicht sowie deren Interaktionen mit höheren Schichten (System, UI).

## 2. Definitionen

### 2.1 Allgemeine Begriffe

- **Domänenschicht**: Mittlere Schicht der NovaDE-Architektur, die die Geschäftslogik und den Kernzustand kapselt
- **Modul**: Funktionale Einheit innerhalb der Domänenschicht
- **Komponente**: Funktionale Einheit innerhalb eines Moduls
- **Schnittstelle**: Definierte Interaktionspunkte zwischen Komponenten oder Modulen
- **Service**: Komponente, die eine bestimmte Funktionalität über eine definierte Schnittstelle bereitstellt

### 2.2 Domänenschicht-spezifische Begriffe

- **Theming**: Verwaltung des Erscheinungsbilds und Stylings der Desktop-Umgebung
- **Workspace**: Virtueller Desktop zur Organisation von Fenstern
- **Notification**: Benachrichtigung an den Benutzer
- **Global Settings**: Desktop-weite Einstellungen und Zustandsverwaltung
- **Window Policy**: Richtlinien für die Fensterverwaltung
- **AI Interaction**: KI-gestützte Funktionen und Benutzereinwilligung

## 3. Anforderungen

### 3.1 Funktionale Anforderungen

1. Die Domänenschicht MUSS ein umfassendes Theming-System bereitstellen.
2. Die Domänenschicht MUSS eine Workspace-Verwaltung für virtuelle Desktops bereitstellen.
3. Die Domänenschicht MUSS ein Benachrichtigungssystem mit Regeln und Filterung bereitstellen.
4. Die Domänenschicht MUSS eine globale Einstellungs- und Zustandsverwaltung bereitstellen.
5. Die Domänenschicht MUSS Richtlinien für die Fensterverwaltung definieren.
6. Die Domänenschicht MUSS KI-gestützte Funktionen mit Benutzereinwilligung bereitstellen.
7. Die Domänenschicht MUSS ein ereignisgesteuertes System für die Kommunikation zwischen Komponenten bereitstellen.

### 3.2 Nicht-funktionale Anforderungen

1. Die Domänenschicht DARF NICHT direkt mit Hardware oder OS-Details interagieren.
2. Die Domänenschicht MUSS thread-sicher implementiert sein.
3. Die Domänenschicht MUSS asynchrone Operationen unterstützen.
4. Die Domänenschicht MUSS eine klare Trennung zwischen Schnittstellen und Implementierungen aufweisen.
5. Die Domänenschicht MUSS umfassend dokumentiert sein.
6. Die Domänenschicht MUSS umfassend getestet sein.

## 4. Architektur

### 4.1 Modulstruktur

Die Domänenschicht besteht aus den folgenden Modulen:

1. **Theming Module** (`theming/`): Erscheinungsbild und Styling
2. **Workspace Management Module** (`workspaces/`): Virtuelle Desktops
3. **AI Interaction Module** (`user_centric_services/ai_interaction/`): KI-Funktionen
4. **Notification Management Module** (`notifications_core/`): Benachrichtigungen
5. **Notification Rules Module** (`notifications_rules/`): Benachrichtigungsregeln
6. **Global Settings Module** (`global_settings_and_state_management/`): Einstellungen
7. **Window Management Policy Module** (`window_policy_engine/`): Fensterverwaltungsrichtlinien
8. **Common Events Module** (`common_events/`): Gemeinsame Ereignisse
9. **Shared Types Module** (`shared_types/`): Gemeinsame Typen

### 4.2 Abhängigkeiten

Die Domänenschicht hat folgende Abhängigkeiten:

1. **Kernschicht**: Für grundlegende Typen, Fehlerbehandlung, Logging und Konfiguration
2. **Externe Abhängigkeiten**:
   - `async_trait`: Für asynchrone Trait-Definitionen
   - `tokio`: Für asynchrone Laufzeitunterstützung
   - `serde`: Für Serialisierung/Deserialisierung
   - `uuid`: Für eindeutige Identifikatoren
   - `chrono`: Für Zeitstempel und Zeitoperationen

## 5. Schnittstellen

### 5.1 Theming Module

```
SCHNITTSTELLE: domain::theming::ThemingEngine
BESCHREIBUNG: Schnittstelle für die Verwaltung des Erscheinungsbilds und Stylings
VERSION: 1.0.0
OPERATIONEN:
  - NAME: get_current_theme_state
    BESCHREIBUNG: Gibt den aktuellen Themenzustand zurück
    PARAMETER: Keine
    RÜCKGABETYP: Result<AppliedThemeState, ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn der Themenzustand nicht abgerufen werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der aktuelle Themenzustand wird zurückgegeben oder ein Fehler wird zurückgegeben
  
  - NAME: get_available_themes
    BESCHREIBUNG: Gibt eine Liste verfügbarer Themes zurück
    PARAMETER: Keine
    RÜCKGABETYP: Result<Vec<ThemeDefinition>, ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn die Themes nicht abgerufen werden können
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine Liste verfügbarer Themes wird zurückgegeben oder ein Fehler wird zurückgegeben
  
  - NAME: get_current_configuration
    BESCHREIBUNG: Gibt die aktuelle Theming-Konfiguration zurück
    PARAMETER: Keine
    RÜCKGABETYP: Result<ThemingConfiguration, ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn die Konfiguration nicht abgerufen werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die aktuelle Theming-Konfiguration wird zurückgegeben oder ein Fehler wird zurückgegeben
  
  - NAME: update_configuration
    BESCHREIBUNG: Aktualisiert die Theming-Konfiguration
    PARAMETER:
      - NAME: new_config
        TYP: ThemingConfiguration
        BESCHREIBUNG: Die neue Theming-Konfiguration
        EINSCHRÄNKUNGEN: Muss eine gültige ThemingConfiguration sein
    RÜCKGABETYP: Result<(), ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn die Konfiguration nicht aktualisiert werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Theming-Konfiguration wird aktualisiert oder ein Fehler wird zurückgegeben
      - Bei erfolgreicher Aktualisierung wird ein ThemeChangedEvent ausgelöst
  
  - NAME: reload_themes_and_tokens
    BESCHREIBUNG: Lädt Themes und Tokens neu
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn Themes und Tokens nicht neu geladen werden können
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Themes und Tokens werden neu geladen oder ein Fehler wird zurückgegeben
      - Bei erfolgreicher Neuladung wird ein ThemeChangedEvent ausgelöst
  
  - NAME: subscribe_to_theme_changes
    BESCHREIBUNG: Abonniert Themenänderungsereignisse
    PARAMETER: Keine
    RÜCKGABETYP: tokio::sync::broadcast::Receiver<ThemeChangedEvent>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein Receiver für ThemeChangedEvents wird zurückgegeben
```

### 5.2 Workspace Management Module

```
SCHNITTSTELLE: domain::workspaces::WorkspaceManagerService
BESCHREIBUNG: Schnittstelle für die Verwaltung virtueller Desktops
VERSION: 1.0.0
OPERATIONEN:
  - NAME: create_workspace
    BESCHREIBUNG: Erstellt einen neuen Workspace
    PARAMETER:
      - NAME: name
        TYP: Option<String>
        BESCHREIBUNG: Optionaler Name des Workspace
        EINSCHRÄNKUNGEN: Keine
      - NAME: persistent_id
        TYP: Option<String>
        BESCHREIBUNG: Optionale persistente ID des Workspace
        EINSCHRÄNKUNGEN: Keine
      - NAME: icon_name
        TYP: Option<String>
        BESCHREIBUNG: Optionaler Icon-Name des Workspace
        EINSCHRÄNKUNGEN: Keine
      - NAME: accent_color_hex
        TYP: Option<String>
        BESCHREIBUNG: Optionale Akzentfarbe des Workspace als Hex-String
        EINSCHRÄNKUNGEN: Muss ein gültiger Hex-Farbcode sein, wenn angegeben
    RÜCKGABETYP: Result<WorkspaceId, WorkspaceManagerError>
    FEHLER:
      - TYP: WorkspaceManagerError
        BEDINGUNG: Wenn der Workspace nicht erstellt werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein neuer Workspace wird erstellt und seine ID wird zurückgegeben oder ein Fehler wird zurückgegeben
      - Bei erfolgreicher Erstellung wird ein WorkspaceCreatedEvent ausgelöst
  
  - NAME: delete_workspace
    BESCHREIBUNG: Löscht einen Workspace
    PARAMETER:
      - NAME: id
        TYP: WorkspaceId
        BESCHREIBUNG: ID des zu löschenden Workspace
        EINSCHRÄNKUNGEN: Muss eine gültige WorkspaceId sein
      - NAME: fallback_id_for_windows
        TYP: Option<WorkspaceId>
        BESCHREIBUNG: Optionale ID eines Fallback-Workspace für Fenster
        EINSCHRÄNKUNGEN: Muss eine gültige WorkspaceId sein, wenn angegeben
    RÜCKGABETYP: Result<(), WorkspaceManagerError>
    FEHLER:
      - TYP: WorkspaceManagerError
        BEDINGUNG: Wenn der Workspace nicht gelöscht werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN:
      - Der Workspace mit der angegebenen ID muss existieren
    NACHBEDINGUNGEN:
      - Der Workspace wird gelöscht oder ein Fehler wird zurückgegeben
      - Bei erfolgreicher Löschung wird ein WorkspaceDeletedEvent ausgelöst
      - Fenster werden zum Fallback-Workspace verschoben, wenn angegeben
  
  - NAME: get_workspace
    BESCHREIBUNG: Gibt einen Workspace zurück
    PARAMETER:
      - NAME: id
        TYP: WorkspaceId
        BESCHREIBUNG: ID des abzurufenden Workspace
        EINSCHRÄNKUNGEN: Muss eine gültige WorkspaceId sein
    RÜCKGABETYP: Option<Workspace>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Workspace wird zurückgegeben, wenn er existiert, sonst None
  
  - NAME: all_workspaces_ordered
    BESCHREIBUNG: Gibt alle Workspaces in geordneter Reihenfolge zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<Workspace>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Alle Workspaces werden in geordneter Reihenfolge zurückgegeben
  
  - NAME: active_workspace_id
    BESCHREIBUNG: Gibt die ID des aktiven Workspace zurück
    PARAMETER: Keine
    RÜCKGABETYP: Option<WorkspaceId>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die ID des aktiven Workspace wird zurückgegeben, wenn ein aktiver Workspace existiert, sonst None
  
  - NAME: set_active_workspace
    BESCHREIBUNG: Setzt den aktiven Workspace
    PARAMETER:
      - NAME: id
        TYP: WorkspaceId
        BESCHREIBUNG: ID des zu aktivierenden Workspace
        EINSCHRÄNKUNGEN: Muss eine gültige WorkspaceId sein
    RÜCKGABETYP: Result<(), WorkspaceManagerError>
    FEHLER:
      - TYP: WorkspaceManagerError
        BEDINGUNG: Wenn der Workspace nicht aktiviert werden kann
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN:
      - Der Workspace mit der angegebenen ID muss existieren
    NACHBEDINGUNGEN:
      - Der Workspace wird aktiviert oder ein Fehler wird zurückgegeben
      - Bei erfolgreicher Aktivierung wird ein WorkspaceActivatedEvent ausgelöst
```

## 6. Datenmodell (Teil 1)

### 6.1 Theming-Typen

```
ENTITÄT: TokenIdentifier
BESCHREIBUNG: Identifikator für Design-Tokens
ATTRIBUTE:
  - NAME: category
    TYP: String
    BESCHREIBUNG: Kategorie des Tokens (z.B. "color", "spacing")
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name des Tokens
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
INVARIANTEN:
  - category und name dürfen nicht leer sein
```

```
ENTITÄT: TokenValue
BESCHREIBUNG: Wert eines Design-Tokens
ATTRIBUTE:
  - NAME: value_type
    TYP: Enum
    BESCHREIBUNG: Typ des Token-Werts
    WERTEBEREICH: {
      ColorValue(Color),
      DimensionValue { value: f32, unit: DimensionUnit },
      StringValue(String),
      NumberValue(f64),
      BooleanValue(bool),
      ReferenceValue(TokenIdentifier)
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Bei ReferenceValue darf keine zirkuläre Referenz entstehen
```

```
ENTITÄT: RawToken
BESCHREIBUNG: Rohe Token-Definition
ATTRIBUTE:
  - NAME: identifier
    TYP: TokenIdentifier
    BESCHREIBUNG: Identifikator des Tokens
    WERTEBEREICH: Gültige TokenIdentifier
    STANDARDWERT: Keiner
  - NAME: value
    TYP: TokenValue
    BESCHREIBUNG: Wert des Tokens
    WERTEBEREICH: Gültige TokenValue
    STANDARDWERT: Keiner
  - NAME: metadata
    TYP: HashMap<String, String>
    BESCHREIBUNG: Zusätzliche Metadaten zum Token
    WERTEBEREICH: Beliebige Schlüssel-Wert-Paare
    STANDARDWERT: Leere HashMap
INVARIANTEN:
  - identifier muss gültig sein
  - value muss gültig sein
```

```
ENTITÄT: ThemeIdentifier
BESCHREIBUNG: Identifikator für Themes
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Themes
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: display_name
    TYP: String
    BESCHREIBUNG: Anzeigename des Themes
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
INVARIANTEN:
  - id und display_name dürfen nicht leer sein
```

```
ENTITÄT: ColorSchemeType
BESCHREIBUNG: Typ des Farbschemas
ATTRIBUTE:
  - NAME: value
    TYP: Enum
    BESCHREIBUNG: Wert des Farbschematyps
    WERTEBEREICH: {Light, Dark}
    STANDARDWERT: Light
```

```
ENTITÄT: AccentColor
BESCHREIBUNG: Akzentfarbe für Themes
ATTRIBUTE:
  - NAME: primary
    TYP: Color
    BESCHREIBUNG: Primäre Akzentfarbe
    WERTEBEREICH: Gültige Color
    STANDARDWERT: Keiner
  - NAME: secondary
    TYP: Option<Color>
    BESCHREIBUNG: Optionale sekundäre Akzentfarbe
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
  - NAME: tertiary
    TYP: Option<Color>
    BESCHREIBUNG: Optionale tertiäre Akzentfarbe
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
INVARIANTEN:
  - primary muss gültig sein
```

```
ENTITÄT: ThemeVariantDefinition
BESCHREIBUNG: Definition einer Themenvariante
ATTRIBUTE:
  - NAME: color_scheme_type
    TYP: ColorSchemeType
    BESCHREIBUNG: Typ des Farbschemas
    WERTEBEREICH: Gültige ColorSchemeType
    STANDARDWERT: ColorSchemeType::Light
  - NAME: token_overrides
    TYP: HashMap<TokenIdentifier, TokenValue>
    BESCHREIBUNG: Überschreibungen von Token-Werten
    WERTEBEREICH: Gültige TokenIdentifier-TokenValue-Paare
    STANDARDWERT: Leere HashMap
INVARIANTEN:
  - Alle TokenIdentifier und TokenValue müssen gültig sein
```

```
ENTITÄT: ThemeDefinition
BESCHREIBUNG: Vollständige Theme-Definition
ATTRIBUTE:
  - NAME: identifier
    TYP: ThemeIdentifier
    BESCHREIBUNG: Identifikator des Themes
    WERTEBEREICH: Gültige ThemeIdentifier
    STANDARDWERT: Keiner
  - NAME: base_tokens
    TYP: HashMap<TokenIdentifier, TokenValue>
    BESCHREIBUNG: Basis-Token-Werte
    WERTEBEREICH: Gültige TokenIdentifier-TokenValue-Paare
    STANDARDWERT: Leere HashMap
  - NAME: variants
    TYP: HashMap<ColorSchemeType, ThemeVariantDefinition>
    BESCHREIBUNG: Themenvarianten
    WERTEBEREICH: Gültige ColorSchemeType-ThemeVariantDefinition-Paare
    STANDARDWERT: Leere HashMap
  - NAME: accent_colors
    TYP: Vec<AccentColor>
    BESCHREIBUNG: Verfügbare Akzentfarben
    WERTEBEREICH: Gültige AccentColor-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - identifier muss gültig sein
  - Alle TokenIdentifier und TokenValue müssen gültig sein
  - Alle ThemeVariantDefinition müssen gültig sein
  - Alle AccentColor müssen gültig sein
```

```
ENTITÄT: AppliedThemeState
BESCHREIBUNG: Aufgelöster Themezustand
ATTRIBUTE:
  - NAME: theme_id
    TYP: ThemeIdentifier
    BESCHREIBUNG: Identifikator des angewendeten Themes
    WERTEBEREICH: Gültige ThemeIdentifier
    STANDARDWERT: Keiner
  - NAME: color_scheme_type
    TYP: ColorSchemeType
    BESCHREIBUNG: Typ des angewendeten Farbschemas
    WERTEBEREICH: Gültige ColorSchemeType
    STANDARDWERT: ColorSchemeType::Light
  - NAME: accent_color
    TYP: AccentColor
    BESCHREIBUNG: Angewendete Akzentfarbe
    WERTEBEREICH: Gültige AccentColor
    STANDARDWERT: Keiner
  - NAME: resolved_tokens
    TYP: HashMap<TokenIdentifier, TokenValue>
    BESCHREIBUNG: Aufgelöste Token-Werte
    WERTEBEREICH: Gültige TokenIdentifier-TokenValue-Paare
    STANDARDWERT: Leere HashMap
INVARIANTEN:
  - theme_id muss gültig sein
  - accent_color muss gültig sein
  - Alle TokenIdentifier und TokenValue müssen gültig sein
  - resolved_tokens darf keine ReferenceValue enthalten (alle Referenzen müssen aufgelöst sein)
```

```
ENTITÄT: ThemingConfiguration
BESCHREIBUNG: Benutzereinstellungen für Theming
ATTRIBUTE:
  - NAME: selected_theme_id
    TYP: String
    BESCHREIBUNG: ID des ausgewählten Themes
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "default"
  - NAME: preferred_color_scheme
    TYP: ColorSchemeType
    BESCHREIBUNG: Bevorzugter Farbschematyp
    WERTEBEREICH: Gültige ColorSchemeType
    STANDARDWERT: ColorSchemeType::Light
  - NAME: selected_accent_color_index
    TYP: usize
    BESCHREIBUNG: Index der ausgewählten Akzentfarbe
    WERTEBEREICH: Gültige Indizes für die Akzentfarben des ausgewählten Themes
    STANDARDWERT: 0
  - NAME: follow_system_color_scheme
    TYP: bool
    BESCHREIBUNG: Ob das Farbschema dem System folgen soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
INVARIANTEN:
  - selected_theme_id darf nicht leer sein
```

### 6.2 Workspace-Typen

```
ENTITÄT: WorkspaceId
BESCHREIBUNG: Eindeutiger Identifikator für Workspaces
ATTRIBUTE:
  - NAME: id
    TYP: Uuid
    BESCHREIBUNG: Eindeutige ID des Workspace
    WERTEBEREICH: Gültige UUID
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss gültig sein
```

```
ENTITÄT: WindowIdentifier
BESCHREIBUNG: Eindeutiger Identifikator für Fenster
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID des Fensters
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
INVARIANTEN:
  - id darf nicht leer sein
```

```
ENTITÄT: WorkspaceLayoutType
BESCHREIBUNG: Layoutmodus für Workspaces
ATTRIBUTE:
  - NAME: value
    TYP: Enum
    BESCHREIBUNG: Wert des Layouttyps
    WERTEBEREICH: {
      Floating,
      Tiling,
      Stacking,
      Tabbed
    }
    STANDARDWERT: Floating
```

```
ENTITÄT: Workspace
BESCHREIBUNG: Virtueller Desktop zur Organisation von Fenstern
ATTRIBUTE:
  - NAME: id
    TYP: WorkspaceId
    BESCHREIBUNG: Eindeutige ID des Workspace
    WERTEBEREICH: Gültige WorkspaceId
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name des Workspace
    WERTEBEREICH: Zeichenkette
    STANDARDWERT: ""
  - NAME: persistent_id
    TYP: Option<String>
    BESCHREIBUNG: Optionale persistente ID des Workspace
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: icon_name
    TYP: Option<String>
    BESCHREIBUNG: Optionaler Icon-Name des Workspace
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: accent_color_hex
    TYP: Option<String>
    BESCHREIBUNG: Optionale Akzentfarbe des Workspace als Hex-String
    WERTEBEREICH: Gültiger Hex-Farbcode oder None
    STANDARDWERT: None
  - NAME: layout_type
    TYP: WorkspaceLayoutType
    BESCHREIBUNG: Layouttyp des Workspace
    WERTEBEREICH: Gültige WorkspaceLayoutType
    STANDARDWERT: WorkspaceLayoutType::Floating
  - NAME: windows
    TYP: Vec<WindowIdentifier>
    BESCHREIBUNG: Fenster im Workspace
    WERTEBEREICH: Gültige WindowIdentifier-Werte
    STANDARDWERT: Leerer Vec
  - NAME: creation_time
    TYP: DateTime<Utc>
    BESCHREIBUNG: Erstellungszeit des Workspace
    WERTEBEREICH: Gültige DateTime<Utc>
    STANDARDWERT: Keiner
  - NAME: last_active_time
    TYP: DateTime<Utc>
    BESCHREIBUNG: Zeit der letzten Aktivierung des Workspace
    WERTEBEREICH: Gültige DateTime<Utc>
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss gültig sein
  - creation_time muss gültig sein
  - last_active_time muss gültig sein
  - Alle WindowIdentifier müssen gültig sein
```
