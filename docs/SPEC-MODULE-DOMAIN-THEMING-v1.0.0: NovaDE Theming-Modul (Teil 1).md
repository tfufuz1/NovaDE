# SPEC-MODULE-DOMAIN-THEMING-v1.0.0: NovaDE Theming-Modul (Teil 1)

```
SPEZIFIKATION: SPEC-MODULE-DOMAIN-THEMING-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0, SPEC-LAYER-CORE-v1.0.0, SPEC-LAYER-DOMAIN-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Diese Spezifikation definiert das Theming-Modul (`domain::theming`) der NovaDE-Domänenschicht. Das Modul stellt die grundlegende Infrastruktur für die Verwaltung des Erscheinungsbilds und Stylings der Desktop-Umgebung bereit und definiert die Mechanismen zum Laden, Parsen, Validieren und Anwenden von Themes und Design-Tokens. Der Geltungsbereich umfasst alle Komponenten und Schnittstellen des Theming-Moduls sowie deren Interaktionen mit anderen Modulen.

## 2. Definitionen

### 2.1 Allgemeine Begriffe

- **Theme**: Sammlung von Design-Tokens, die das Erscheinungsbild definieren
- **Design-Token**: Atomare Designeinheit mit definiertem Wert (z.B. Farbe, Abstand)
- **Tokenreferenz**: Verweis auf ein anderes Token
- **Tokenauflösung**: Prozess der Auflösung von Tokenreferenzen zu konkreten Werten
- **Tokenvererbung**: Mechanismus, bei dem ein Token Eigenschaften eines anderen erbt
- **Tokenüberschreibung**: Mechanismus, bei dem ein Token ein anderes überschreibt

### 2.2 Modulspezifische Begriffe

- **ThemeManager**: Zentrale Komponente für die Verwaltung von Themes
- **TokenRegistry**: Komponente zur Verwaltung von Design-Tokens
- **ThemeLoader**: Komponente zum Laden von Themes
- **TokenResolver**: Komponente zur Auflösung von Tokenreferenzen
- **ThemeApplicator**: Komponente zur Anwendung von Themes
- **TokenIdentifier**: Eindeutiger Bezeichner für ein Design-Token
- **TokenValue**: Wert eines Design-Tokens
- **TokenCategory**: Kategorie eines Design-Tokens
- **ThemeVariant**: Variante eines Themes (z.B. hell, dunkel)
- **ColorScheme**: Farbschema eines Themes

## 3. Anforderungen

### 3.1 Funktionale Anforderungen

1. Das Modul MUSS Mechanismen zum Laden von Themes bereitstellen.
2. Das Modul MUSS Mechanismen zum Parsen von Theme-Definitionen bereitstellen.
3. Das Modul MUSS Mechanismen zum Validieren von Themes bereitstellen.
4. Das Modul MUSS Mechanismen zum Anwenden von Themes bereitstellen.
5. Das Modul MUSS Mechanismen zur Verwaltung von Design-Tokens bereitstellen.
6. Das Modul MUSS Mechanismen zur Auflösung von Tokenreferenzen bereitstellen.
7. Das Modul MUSS Mechanismen zur Tokenvererbung bereitstellen.
8. Das Modul MUSS Mechanismen zur Tokenüberschreibung bereitstellen.
9. Das Modul MUSS Mechanismen zum Wechseln zwischen Themes bereitstellen.
10. Das Modul MUSS Mechanismen zum Wechseln zwischen Theme-Varianten bereitstellen.
11. Das Modul MUSS Mechanismen zur Benachrichtigung über Theme-Änderungen bereitstellen.
12. Das Modul MUSS Mechanismen zur Persistenz von Theme-Einstellungen bereitstellen.

### 3.2 Nicht-funktionale Anforderungen

1. Das Modul MUSS effizient mit Ressourcen umgehen.
2. Das Modul MUSS thread-safe sein.
3. Das Modul MUSS eine klare und konsistente API bereitstellen.
4. Das Modul MUSS gut dokumentiert sein.
5. Das Modul MUSS leicht erweiterbar sein.
6. Das Modul MUSS robust gegen Fehleingaben sein.
7. Das Modul MUSS minimale externe Abhängigkeiten haben.
8. Das Modul MUSS eine hohe Performance bieten.
9. Das Modul MUSS eine geringe Latenz bei Theme-Wechseln bieten.
10. Das Modul MUSS eine hohe Zuverlässigkeit bieten.

## 4. Architektur

### 4.1 Komponentenstruktur

Das Theming-Modul besteht aus den folgenden Komponenten:

1. **ThemeManager** (`theme_manager.rs`): Zentrale Komponente für die Verwaltung von Themes
2. **TokenRegistry** (`token_registry.rs`): Komponente zur Verwaltung von Design-Tokens
3. **ThemeLoader** (`theme_loader.rs`): Komponente zum Laden von Themes
4. **TokenResolver** (`token_resolver.rs`): Komponente zur Auflösung von Tokenreferenzen
5. **ThemeApplicator** (`theme_applicator.rs`): Komponente zur Anwendung von Themes
6. **ThemeStorage** (`theme_storage.rs`): Komponente zur Persistenz von Theme-Einstellungen
7. **ThemeEvents** (`theme_events.rs`): Komponente zur Benachrichtigung über Theme-Änderungen
8. **TokenIdentifier** (`token_identifier.rs`): Datenstruktur für Token-Bezeichner
9. **TokenValue** (`token_value.rs`): Datenstruktur für Token-Werte
10. **ThemeDefinition** (`theme_definition.rs`): Datenstruktur für Theme-Definitionen
11. **ColorScheme** (`color_scheme.rs`): Datenstruktur für Farbschemata
12. **ThemeVariant** (`theme_variant.rs`): Datenstruktur für Theme-Varianten

### 4.2 Abhängigkeiten

Das Theming-Modul hat folgende Abhängigkeiten:

1. **Interne Abhängigkeiten**:
   - `core::errors`: Für die Fehlerbehandlung
   - `core::config`: Für die Konfiguration
   - `core::logging`: Für das Logging

2. **Externe Abhängigkeiten**:
   - `serde`: Für die Serialisierung und Deserialisierung von Theme-Definitionen
   - `serde_json`: Für das Parsen von JSON-Theme-Definitionen
   - `csscolorparser`: Für das Parsen von Farbwerten
   - `notify`: Für die Überwachung von Theme-Dateien

## 5. Schnittstellen

### 5.1 ThemeManager

```
SCHNITTSTELLE: domain::theming::ThemeManager
BESCHREIBUNG: Zentrale Komponente für die Verwaltung von Themes
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue ThemeManager-Instanz
    PARAMETER:
      - NAME: config
        TYP: ThemeConfig
        BESCHREIBUNG: Konfiguration für den ThemeManager
        EINSCHRÄNKUNGEN: Muss eine gültige ThemeConfig sein
    RÜCKGABETYP: Result<ThemeManager, ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des ThemeManagers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue ThemeManager-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des ThemeManagers auftritt
  
  - NAME: initialize
    BESCHREIBUNG: Initialisiert den ThemeManager
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn ein Fehler bei der Initialisierung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der ThemeManager wird initialisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Initialisierung auftritt
  
  - NAME: load_themes
    BESCHREIBUNG: Lädt Themes aus einem Verzeichnis
    PARAMETER:
      - NAME: directory
        TYP: &Path
        BESCHREIBUNG: Verzeichnis mit Theme-Definitionen
        EINSCHRÄNKUNGEN: Muss ein gültiger Verzeichnispfad sein
    RÜCKGABETYP: Result<(), ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn ein Fehler beim Laden der Themes auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Themes werden geladen
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Laden der Themes auftritt
  
  - NAME: load_theme
    BESCHREIBUNG: Lädt ein Theme aus einer Datei
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Theme-Definitionsdatei
        EINSCHRÄNKUNGEN: Muss ein gültiger Dateipfad sein
    RÜCKGABETYP: Result<ThemeId, ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn ein Fehler beim Laden des Themes auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Theme wird geladen und eine ThemeId wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Laden des Themes auftritt
  
  - NAME: get_theme
    BESCHREIBUNG: Gibt ein Theme zurück
    PARAMETER:
      - NAME: id
        TYP: &ThemeId
        BESCHREIBUNG: ID des Themes
        EINSCHRÄNKUNGEN: Muss eine gültige ThemeId sein
    RÜCKGABETYP: Result<&Theme, ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn das Theme nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Theme wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Theme nicht gefunden wird
  
  - NAME: get_current_theme
    BESCHREIBUNG: Gibt das aktuelle Theme zurück
    PARAMETER: Keine
    RÜCKGABETYP: &Theme
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das aktuelle Theme wird zurückgegeben
  
  - NAME: set_current_theme
    BESCHREIBUNG: Setzt das aktuelle Theme
    PARAMETER:
      - NAME: id
        TYP: &ThemeId
        BESCHREIBUNG: ID des Themes
        EINSCHRÄNKUNGEN: Muss eine gültige ThemeId sein
    RÜCKGABETYP: Result<(), ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn das Theme nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das aktuelle Theme wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn das Theme nicht gefunden wird
  
  - NAME: get_current_variant
    BESCHREIBUNG: Gibt die aktuelle Theme-Variante zurück
    PARAMETER: Keine
    RÜCKGABETYP: ThemeVariant
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die aktuelle Theme-Variante wird zurückgegeben
  
  - NAME: set_current_variant
    BESCHREIBUNG: Setzt die aktuelle Theme-Variante
    PARAMETER:
      - NAME: variant
        TYP: ThemeVariant
        BESCHREIBUNG: Theme-Variante
        EINSCHRÄNKUNGEN: Muss eine gültige ThemeVariant sein
    RÜCKGABETYP: Result<(), ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn die Theme-Variante nicht unterstützt wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die aktuelle Theme-Variante wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn die Theme-Variante nicht unterstützt wird
  
  - NAME: get_token_value
    BESCHREIBUNG: Gibt den Wert eines Tokens zurück
    PARAMETER:
      - NAME: id
        TYP: &TokenIdentifier
        BESCHREIBUNG: ID des Tokens
        EINSCHRÄNKUNGEN: Muss eine gültige TokenIdentifier sein
    RÜCKGABETYP: Result<&TokenValue, ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn das Token nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Wert des Tokens wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Token nicht gefunden wird
  
  - NAME: register_theme_change_listener
    BESCHREIBUNG: Registriert einen Listener für Theme-Änderungen
    PARAMETER:
      - NAME: listener
        TYP: Box<dyn Fn(&Theme, ThemeVariant) + Send + Sync + 'static>
        BESCHREIBUNG: Listener-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: ListenerId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird registriert und eine ListenerId wird zurückgegeben
  
  - NAME: unregister_theme_change_listener
    BESCHREIBUNG: Entfernt einen Listener für Theme-Änderungen
    PARAMETER:
      - NAME: id
        TYP: ListenerId
        BESCHREIBUNG: ID des Listeners
        EINSCHRÄNKUNGEN: Muss eine gültige ListenerId sein
    RÜCKGABETYP: Result<(), ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn der Listener nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Listener nicht gefunden wird
  
  - NAME: save_theme_settings
    BESCHREIBUNG: Speichert die Theme-Einstellungen
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn ein Fehler beim Speichern der Theme-Einstellungen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Theme-Einstellungen werden gespeichert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Speichern der Theme-Einstellungen auftritt
```

### 5.2 TokenRegistry

```
SCHNITTSTELLE: domain::theming::TokenRegistry
BESCHREIBUNG: Komponente zur Verwaltung von Design-Tokens
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue TokenRegistry-Instanz
    PARAMETER: Keine
    RÜCKGABETYP: TokenRegistry
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue TokenRegistry-Instanz wird erstellt
  
  - NAME: register_token
    BESCHREIBUNG: Registriert ein Token
    PARAMETER:
      - NAME: id
        TYP: TokenIdentifier
        BESCHREIBUNG: ID des Tokens
        EINSCHRÄNKUNGEN: Muss eine gültige TokenIdentifier sein
      - NAME: value
        TYP: TokenValue
        BESCHREIBUNG: Wert des Tokens
        EINSCHRÄNKUNGEN: Muss ein gültiger TokenValue sein
    RÜCKGABETYP: Result<(), ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn ein Fehler bei der Registrierung des Tokens auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Token wird registriert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Registrierung des Tokens auftritt
  
  - NAME: get_token
    BESCHREIBUNG: Gibt ein Token zurück
    PARAMETER:
      - NAME: id
        TYP: &TokenIdentifier
        BESCHREIBUNG: ID des Tokens
        EINSCHRÄNKUNGEN: Muss eine gültige TokenIdentifier sein
    RÜCKGABETYP: Result<&TokenValue, ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn das Token nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Wert des Tokens wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Token nicht gefunden wird
  
  - NAME: has_token
    BESCHREIBUNG: Prüft, ob ein Token existiert
    PARAMETER:
      - NAME: id
        TYP: &TokenIdentifier
        BESCHREIBUNG: ID des Tokens
        EINSCHRÄNKUNGEN: Muss eine gültige TokenIdentifier sein
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn das Token existiert
      - false wird zurückgegeben, wenn das Token nicht existiert
  
  - NAME: remove_token
    BESCHREIBUNG: Entfernt ein Token
    PARAMETER:
      - NAME: id
        TYP: &TokenIdentifier
        BESCHREIBUNG: ID des Tokens
        EINSCHRÄNKUNGEN: Muss eine gültige TokenIdentifier sein
    RÜCKGABETYP: Result<(), ThemingError>
    FEHLER:
      - TYP: ThemingError
        BEDINGUNG: Wenn das Token nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Token wird entfernt
      - Ein Fehler wird zurückgegeben, wenn das Token nicht gefunden wird
  
  - NAME: clear
    BESCHREIBUNG: Entfernt alle Tokens
    PARAMETER: Keine
    RÜCKGABETYP: ()
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Alle Tokens werden entfernt
  
  - NAME: get_tokens_by_category
    BESCHREIBUNG: Gibt alle Tokens einer Kategorie zurück
    PARAMETER:
      - NAME: category
        TYP: TokenCategory
        BESCHREIBUNG: Kategorie
        EINSCHRÄNKUNGEN: Muss eine gültige TokenCategory sein
    RÜCKGABETYP: Vec<(TokenIdentifier, &TokenValue)>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Alle Tokens der Kategorie werden zurückgegeben
  
  - NAME: get_all_tokens
    BESCHREIBUNG: Gibt alle Tokens zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<(TokenIdentifier, &TokenValue)>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Alle Tokens werden zurückgegeben
```

## 6. Datenmodell (Teil 1)

### 6.1 TokenIdentifier

```
ENTITÄT: TokenIdentifier
BESCHREIBUNG: Eindeutiger Bezeichner für ein Design-Token
ATTRIBUTE:
  - NAME: category
    TYP: TokenCategory
    BESCHREIBUNG: Kategorie des Tokens
    WERTEBEREICH: Gültige TokenCategory
    STANDARDWERT: TokenCategory::Misc
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name des Tokens
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: variant
    TYP: Option<ThemeVariant>
    BESCHREIBUNG: Optionale Variante des Tokens
    WERTEBEREICH: Gültige ThemeVariant oder None
    STANDARDWERT: None
INVARIANTEN:
  - name darf nicht leer sein
```

### 6.2 TokenValue

```
ENTITÄT: TokenValue
BESCHREIBUNG: Wert eines Design-Tokens
ATTRIBUTE:
  - NAME: value_type
    TYP: Enum
    BESCHREIBUNG: Typ des Werts
    WERTEBEREICH: {
      Color(Color),
      Dimension(Dimension),
      String(String),
      Number(f64),
      Boolean(bool),
      Reference(TokenIdentifier),
      Gradient(Vec<GradientStop>),
      Shadow(Shadow),
      Font(Font),
      Spacing(Spacing),
      Border(Border),
      Duration(Duration),
      Easing(Easing)
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Bei Color muss der Farbwert gültig sein
  - Bei Dimension muss der Wert gültig sein
  - Bei String darf die Zeichenkette nicht leer sein
  - Bei Reference muss die TokenIdentifier gültig sein
  - Bei Gradient muss mindestens ein GradientStop vorhanden sein
  - Bei Shadow müssen alle Werte gültig sein
  - Bei Font müssen alle Werte gültig sein
  - Bei Spacing müssen alle Werte gültig sein
  - Bei Border müssen alle Werte gültig sein
  - Bei Duration muss der Wert gültig sein
  - Bei Easing muss der Wert gültig sein
```

### 6.3 TokenCategory

```
ENTITÄT: TokenCategory
BESCHREIBUNG: Kategorie eines Design-Tokens
ATTRIBUTE:
  - NAME: category
    TYP: Enum
    BESCHREIBUNG: Kategorie
    WERTEBEREICH: {
      Color,
      Typography,
      Spacing,
      Border,
      Shadow,
      Animation,
      Layout,
      Misc
    }
    STANDARDWERT: Misc
INVARIANTEN:
  - Keine
```

### 6.4 ThemeVariant

```
ENTITÄT: ThemeVariant
BESCHREIBUNG: Variante eines Themes
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Variante
    WERTEBEREICH: {
      Light,
      Dark,
      HighContrast,
      Custom(String)
    }
    STANDARDWERT: Light
INVARIANTEN:
  - Bei Custom darf die Zeichenkette nicht leer sein
```

### 6.5 Color

```
ENTITÄT: Color
BESCHREIBUNG: Farbwert
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

### 6.6 Dimension

```
ENTITÄT: Dimension
BESCHREIBUNG: Dimensionswert
ATTRIBUTE:
  - NAME: value
    TYP: f64
    BESCHREIBUNG: Wert
    WERTEBEREICH: Reelle Zahlen
    STANDARDWERT: 0.0
  - NAME: unit
    TYP: DimensionUnit
    BESCHREIBUNG: Einheit
    WERTEBEREICH: {
      Px,
      Em,
      Rem,
      Percent,
      Pt,
      Dp,
      Sp
    }
    STANDARDWERT: DimensionUnit::Px
INVARIANTEN:
  - Keine
```

### 6.7 GradientStop

```
ENTITÄT: GradientStop
BESCHREIBUNG: Haltepunkt in einem Farbverlauf
ATTRIBUTE:
  - NAME: color
    TYP: Color
    BESCHREIBUNG: Farbe
    WERTEBEREICH: Gültige Color
    STANDARDWERT: Color { r: 0, g: 0, b: 0, a: 255 }
  - NAME: position
    TYP: f64
    BESCHREIBUNG: Position im Farbverlauf
    WERTEBEREICH: [0.0, 1.0]
    STANDARDWERT: 0.0
INVARIANTEN:
  - position muss im Bereich [0.0, 1.0] liegen
```

### 6.8 Shadow

```
ENTITÄT: Shadow
BESCHREIBUNG: Schattenwert
ATTRIBUTE:
  - NAME: x_offset
    TYP: Dimension
    BESCHREIBUNG: X-Offset
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 0.0, unit: DimensionUnit::Px }
  - NAME: y_offset
    TYP: Dimension
    BESCHREIBUNG: Y-Offset
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 0.0, unit: DimensionUnit::Px }
  - NAME: blur_radius
    TYP: Dimension
    BESCHREIBUNG: Unschärferadius
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 0.0, unit: DimensionUnit::Px }
  - NAME: spread_radius
    TYP: Dimension
    BESCHREIBUNG: Ausbreitungsradius
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 0.0, unit: DimensionUnit::Px }
  - NAME: color
    TYP: Color
    BESCHREIBUNG: Farbe
    WERTEBEREICH: Gültige Color
    STANDARDWERT: Color { r: 0, g: 0, b: 0, a: 255 }
  - NAME: inset
    TYP: bool
    BESCHREIBUNG: Ob der Schatten nach innen gerichtet ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
INVARIANTEN:
  - Keine
```
