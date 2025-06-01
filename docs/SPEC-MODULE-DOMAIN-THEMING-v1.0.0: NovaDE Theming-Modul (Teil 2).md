# SPEC-MODULE-DOMAIN-THEMING-v1.0.0: NovaDE Theming-Modul (Teil 2)

## 6. Datenmodell (Fortsetzung)

### 6.9 Font

```
ENTITÄT: Font
BESCHREIBUNG: Schriftart
ATTRIBUTE:
  - NAME: family
    TYP: String
    BESCHREIBUNG: Schriftfamilie
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "Sans"
  - NAME: style
    TYP: FontStyle
    BESCHREIBUNG: Schriftstil
    WERTEBEREICH: Gültiger FontStyle
    STANDARDWERT: FontStyle::Normal
  - NAME: weight
    TYP: FontWeight
    BESCHREIBUNG: Schriftstärke
    WERTEBEREICH: Gültiger FontWeight
    STANDARDWERT: FontWeight::Regular
  - NAME: size
    TYP: Dimension
    BESCHREIBUNG: Schriftgröße
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 12.0, unit: DimensionUnit::Pt }
INVARIANTEN:
  - family darf nicht leer sein
  - size.value muss größer als 0 sein
```

### 6.10 FontStyle

```
ENTITÄT: FontStyle
BESCHREIBUNG: Schriftstil
ATTRIBUTE:
  - NAME: style
    TYP: Enum
    BESCHREIBUNG: Stil
    WERTEBEREICH: {
      Normal,
      Italic,
      Oblique
    }
    STANDARDWERT: Normal
INVARIANTEN:
  - Keine
```

### 6.11 FontWeight

```
ENTITÄT: FontWeight
BESCHREIBUNG: Schriftstärke
ATTRIBUTE:
  - NAME: weight
    TYP: Enum
    BESCHREIBUNG: Stärke
    WERTEBEREICH: {
      Thin,
      ExtraLight,
      Light,
      Regular,
      Medium,
      SemiBold,
      Bold,
      ExtraBold,
      Black
    }
    STANDARDWERT: Regular
INVARIANTEN:
  - Keine
```

### 6.12 Spacing

```
ENTITÄT: Spacing
BESCHREIBUNG: Abstandswert
ATTRIBUTE:
  - NAME: top
    TYP: Dimension
    BESCHREIBUNG: Oberer Abstand
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 0.0, unit: DimensionUnit::Px }
  - NAME: right
    TYP: Dimension
    BESCHREIBUNG: Rechter Abstand
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 0.0, unit: DimensionUnit::Px }
  - NAME: bottom
    TYP: Dimension
    BESCHREIBUNG: Unterer Abstand
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 0.0, unit: DimensionUnit::Px }
  - NAME: left
    TYP: Dimension
    BESCHREIBUNG: Linker Abstand
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 0.0, unit: DimensionUnit::Px }
INVARIANTEN:
  - Keine
```

### 6.13 Border

```
ENTITÄT: Border
BESCHREIBUNG: Rahmenwert
ATTRIBUTE:
  - NAME: width
    TYP: Dimension
    BESCHREIBUNG: Rahmenbreite
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 1.0, unit: DimensionUnit::Px }
  - NAME: style
    TYP: BorderStyle
    BESCHREIBUNG: Rahmenstil
    WERTEBEREICH: Gültiger BorderStyle
    STANDARDWERT: BorderStyle::Solid
  - NAME: color
    TYP: Color
    BESCHREIBUNG: Rahmenfarbe
    WERTEBEREICH: Gültige Color
    STANDARDWERT: Color { r: 0, g: 0, b: 0, a: 255 }
  - NAME: radius
    TYP: Dimension
    BESCHREIBUNG: Rahmenradius
    WERTEBEREICH: Gültige Dimension
    STANDARDWERT: Dimension { value: 0.0, unit: DimensionUnit::Px }
INVARIANTEN:
  - width.value muss größer oder gleich 0 sein
  - radius.value muss größer oder gleich 0 sein
```

### 6.14 BorderStyle

```
ENTITÄT: BorderStyle
BESCHREIBUNG: Rahmenstil
ATTRIBUTE:
  - NAME: style
    TYP: Enum
    BESCHREIBUNG: Stil
    WERTEBEREICH: {
      None,
      Solid,
      Dashed,
      Dotted,
      Double,
      Groove,
      Ridge,
      Inset,
      Outset
    }
    STANDARDWERT: Solid
INVARIANTEN:
  - Keine
```

### 6.15 Duration

```
ENTITÄT: Duration
BESCHREIBUNG: Zeitdauer
ATTRIBUTE:
  - NAME: value
    TYP: u32
    BESCHREIBUNG: Wert
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
  - NAME: unit
    TYP: DurationUnit
    BESCHREIBUNG: Einheit
    WERTEBEREICH: {
      Ms,
      S
    }
    STANDARDWERT: DurationUnit::Ms
INVARIANTEN:
  - Keine
```

### 6.16 Easing

```
ENTITÄT: Easing
BESCHREIBUNG: Beschleunigungsfunktion
ATTRIBUTE:
  - NAME: function
    TYP: Enum
    BESCHREIBUNG: Funktion
    WERTEBEREICH: {
      Linear,
      EaseIn,
      EaseOut,
      EaseInOut,
      EaseInQuad,
      EaseOutQuad,
      EaseInOutQuad,
      EaseInCubic,
      EaseOutCubic,
      EaseInOutCubic,
      EaseInQuart,
      EaseOutQuart,
      EaseInOutQuart,
      EaseInQuint,
      EaseOutQuint,
      EaseInOutQuint,
      EaseInSine,
      EaseOutSine,
      EaseInOutSine,
      EaseInExpo,
      EaseOutExpo,
      EaseInOutExpo,
      EaseInCirc,
      EaseOutCirc,
      EaseInOutCirc,
      EaseInBack,
      EaseOutBack,
      EaseInOutBack,
      EaseInElastic,
      EaseOutElastic,
      EaseInOutElastic,
      EaseInBounce,
      EaseOutBounce,
      EaseInOutBounce
    }
    STANDARDWERT: Linear
INVARIANTEN:
  - Keine
```

### 6.17 ThemeId

```
ENTITÄT: ThemeId
BESCHREIBUNG: Eindeutiger Bezeichner für ein Theme
ATTRIBUTE:
  - NAME: id
    TYP: String
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
INVARIANTEN:
  - id darf nicht leer sein
  - id muss eindeutig sein
```

### 6.18 Theme

```
ENTITÄT: Theme
BESCHREIBUNG: Theme für die Desktop-Umgebung
ATTRIBUTE:
  - NAME: id
    TYP: ThemeId
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Gültige ThemeId
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name des Themes
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "Unbenannt"
  - NAME: description
    TYP: String
    BESCHREIBUNG: Beschreibung des Themes
    WERTEBEREICH: Zeichenkette
    STANDARDWERT: ""
  - NAME: author
    TYP: String
    BESCHREIBUNG: Autor des Themes
    WERTEBEREICH: Zeichenkette
    STANDARDWERT: ""
  - NAME: version
    TYP: String
    BESCHREIBUNG: Version des Themes
    WERTEBEREICH: Zeichenkette
    STANDARDWERT: "1.0.0"
  - NAME: variants
    TYP: Vec<ThemeVariant>
    BESCHREIBUNG: Unterstützte Varianten
    WERTEBEREICH: Gültige ThemeVariant-Werte
    STANDARDWERT: [ThemeVariant::Light, ThemeVariant::Dark]
  - NAME: tokens
    TYP: HashMap<TokenIdentifier, TokenValue>
    BESCHREIBUNG: Design-Tokens
    WERTEBEREICH: Gültige TokenIdentifier-TokenValue-Paare
    STANDARDWERT: Leere HashMap
  - NAME: parent
    TYP: Option<ThemeId>
    BESCHREIBUNG: Eltern-Theme
    WERTEBEREICH: Gültige ThemeId oder None
    STANDARDWERT: None
INVARIANTEN:
  - name darf nicht leer sein
  - variants darf nicht leer sein
```

### 6.19 ThemeConfig

```
ENTITÄT: ThemeConfig
BESCHREIBUNG: Konfiguration für den ThemeManager
ATTRIBUTE:
  - NAME: theme_directories
    TYP: Vec<PathBuf>
    BESCHREIBUNG: Verzeichnisse mit Theme-Definitionen
    WERTEBEREICH: Gültige Verzeichnispfade
    STANDARDWERT: Leerer Vec
  - NAME: default_theme
    TYP: Option<ThemeId>
    BESCHREIBUNG: Standard-Theme
    WERTEBEREICH: Gültige ThemeId oder None
    STANDARDWERT: None
  - NAME: default_variant
    TYP: ThemeVariant
    BESCHREIBUNG: Standard-Variante
    WERTEBEREICH: Gültige ThemeVariant
    STANDARDWERT: ThemeVariant::Light
  - NAME: watch_theme_changes
    TYP: bool
    BESCHREIBUNG: Ob Änderungen an Theme-Dateien überwacht werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: settings_file
    TYP: Option<PathBuf>
    BESCHREIBUNG: Datei für Theme-Einstellungen
    WERTEBEREICH: Gültiger Dateipfad oder None
    STANDARDWERT: None
INVARIANTEN:
  - Keine
```

### 6.20 ListenerId

```
ENTITÄT: ListenerId
BESCHREIBUNG: Eindeutiger Bezeichner für einen Listener
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

## 7. Verhaltensmodell

### 7.1 Theme-Initialisierung

```
ZUSTANDSAUTOMAT: ThemeInitialization
BESCHREIBUNG: Prozess der Initialisierung des Theming-Systems
ZUSTÄNDE:
  - NAME: Uninitialized
    BESCHREIBUNG: Theming-System ist nicht initialisiert
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: Initializing
    BESCHREIBUNG: Theming-System wird initialisiert
    EINTRITTSAKTIONEN: Konfiguration laden
    AUSTRITTSAKTIONEN: Keine
  - NAME: LoadingThemes
    BESCHREIBUNG: Themes werden geladen
    EINTRITTSAKTIONEN: Theme-Verzeichnisse durchsuchen
    AUSTRITTSAKTIONEN: Keine
  - NAME: ResolvingTokens
    BESCHREIBUNG: Token-Referenzen werden aufgelöst
    EINTRITTSAKTIONEN: TokenResolver initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: ApplyingTheme
    BESCHREIBUNG: Theme wird angewendet
    EINTRITTSAKTIONEN: ThemeApplicator initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Initialized
    BESCHREIBUNG: Theming-System ist initialisiert
    EINTRITTSAKTIONEN: Listener benachrichtigen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler bei der Initialisierung
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Uninitialized
    NACH: Initializing
    EREIGNIS: initialize aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Konfiguration validieren
  - VON: Initializing
    NACH: LoadingThemes
    EREIGNIS: Konfiguration erfolgreich geladen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: Initializing
    NACH: Error
    EREIGNIS: Fehler beim Laden der Konfiguration
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
  - VON: LoadingThemes
    NACH: ResolvingTokens
    EREIGNIS: Themes erfolgreich geladen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: LoadingThemes
    NACH: Error
    EREIGNIS: Fehler beim Laden der Themes
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
  - VON: ResolvingTokens
    NACH: ApplyingTheme
    EREIGNIS: Token-Referenzen erfolgreich aufgelöst
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ResolvingTokens
    NACH: Error
    EREIGNIS: Fehler beim Auflösen der Token-Referenzen
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
  - VON: ApplyingTheme
    NACH: Initialized
    EREIGNIS: Theme erfolgreich angewendet
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ApplyingTheme
    NACH: Error
    EREIGNIS: Fehler beim Anwenden des Themes
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
INITIALZUSTAND: Uninitialized
ENDZUSTÄNDE: [Initialized, Error]
```

### 7.2 Theme-Wechsel

```
ZUSTANDSAUTOMAT: ThemeChange
BESCHREIBUNG: Prozess des Wechsels eines Themes
ZUSTÄNDE:
  - NAME: Initial
    BESCHREIBUNG: Initialer Zustand
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: ValidatingTheme
    BESCHREIBUNG: Theme wird validiert
    EINTRITTSAKTIONEN: Theme-ID prüfen
    AUSTRITTSAKTIONEN: Keine
  - NAME: ResolvingTokens
    BESCHREIBUNG: Token-Referenzen werden aufgelöst
    EINTRITTSAKTIONEN: TokenResolver initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: ApplyingTheme
    BESCHREIBUNG: Theme wird angewendet
    EINTRITTSAKTIONEN: ThemeApplicator initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: NotifyingListeners
    BESCHREIBUNG: Listener werden benachrichtigt
    EINTRITTSAKTIONEN: Listener-Liste durchlaufen
    AUSTRITTSAKTIONEN: Keine
  - NAME: SavingSettings
    BESCHREIBUNG: Einstellungen werden gespeichert
    EINTRITTSAKTIONEN: Einstellungsdatei öffnen
    AUSTRITTSAKTIONEN: Einstellungsdatei schließen
  - NAME: Completed
    BESCHREIBUNG: Theme-Wechsel abgeschlossen
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler beim Theme-Wechsel
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Initial
    NACH: ValidatingTheme
    EREIGNIS: set_current_theme aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ValidatingTheme
    NACH: ResolvingTokens
    EREIGNIS: Theme erfolgreich validiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ValidatingTheme
    NACH: Error
    EREIGNIS: Theme nicht gefunden
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
  - VON: ResolvingTokens
    NACH: ApplyingTheme
    EREIGNIS: Token-Referenzen erfolgreich aufgelöst
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ResolvingTokens
    NACH: Error
    EREIGNIS: Fehler beim Auflösen der Token-Referenzen
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
  - VON: ApplyingTheme
    NACH: NotifyingListeners
    EREIGNIS: Theme erfolgreich angewendet
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ApplyingTheme
    NACH: Error
    EREIGNIS: Fehler beim Anwenden des Themes
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
  - VON: NotifyingListeners
    NACH: SavingSettings
    EREIGNIS: Listener erfolgreich benachrichtigt
    BEDINGUNG: config.settings_file.is_some()
    AKTIONEN: Keine
  - VON: NotifyingListeners
    NACH: Completed
    EREIGNIS: Listener erfolgreich benachrichtigt
    BEDINGUNG: config.settings_file.is_none()
    AKTIONEN: Keine
  - VON: NotifyingListeners
    NACH: Error
    EREIGNIS: Fehler bei der Benachrichtigung der Listener
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
  - VON: SavingSettings
    NACH: Completed
    EREIGNIS: Einstellungen erfolgreich gespeichert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: SavingSettings
    NACH: Error
    EREIGNIS: Fehler beim Speichern der Einstellungen
    BEDINGUNG: Keine
    AKTIONEN: ThemingError erstellen
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
ENTITÄT: ThemingError
BESCHREIBUNG: Fehler im Theming-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      IoError { path: Option<PathBuf>, source: std::io::Error },
      ParseError { path: Option<PathBuf>, source: Box<dyn std::error::Error + Send + Sync + 'static> },
      ValidationError { theme_id: Option<ThemeId>, message: String },
      TokenError { token_id: Option<TokenIdentifier>, message: String },
      ThemeNotFoundError { theme_id: ThemeId },
      TokenNotFoundError { token_id: TokenIdentifier },
      CircularReferenceError { token_id: TokenIdentifier },
      IncompatibleVariantError { theme_id: ThemeId, variant: ThemeVariant },
      ListenerError { listener_id: Option<ListenerId>, message: String },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

## 9. Leistungsanforderungen

### 9.1 Allgemeine Leistungsanforderungen

1. Das Theming MUSS effizient mit Ressourcen umgehen.
2. Das Theming MUSS eine geringe Latenz haben.
3. Das Theming MUSS skalierbar sein.

### 9.2 Spezifische Leistungsanforderungen

1. Das Laden eines Themes MUSS in unter 100ms abgeschlossen sein.
2. Das Wechseln eines Themes MUSS in unter 50ms abgeschlossen sein.
3. Der Zugriff auf ein Token MUSS in unter 1ms abgeschlossen sein.
4. Die Auflösung von Token-Referenzen MUSS in unter 10ms abgeschlossen sein.
5. Das Theming DARF nicht mehr als 10MB Speicher verbrauchen.

## 10. Sicherheitsanforderungen

### 10.1 Allgemeine Sicherheitsanforderungen

1. Das Theming MUSS memory-safe sein.
2. Das Theming MUSS thread-safe sein.
3. Das Theming MUSS robust gegen Fehleingaben sein.

### 10.2 Spezifische Sicherheitsanforderungen

1. Das Theming MUSS Eingaben validieren, um Injection-Angriffe zu verhindern.
2. Das Theming MUSS Zugriffskontrollen für Theme-Dateien implementieren.
3. Das Theming MUSS sichere Standardwerte verwenden.
4. Das Theming MUSS Ressourcenlimits implementieren, um Denial-of-Service-Angriffe zu verhindern.

## 11. Testkriterien

### 11.1 Allgemeine Testkriterien

1. Jede Komponente MUSS Einheitstests haben.
2. Jede öffentliche Funktion MUSS getestet sein.
3. Jeder Fehlerfall MUSS getestet sein.

### 11.2 Spezifische Testkriterien

1. Das Theming MUSS mit verschiedenen Theme-Formaten getestet sein.
2. Das Theming MUSS mit verschiedenen Theme-Strukturen getestet sein.
3. Das Theming MUSS mit verschiedenen Fehlerszenarien getestet sein.
4. Das Theming MUSS mit verschiedenen Token-Typen getestet sein.
5. Das Theming MUSS mit verschiedenen Token-Referenzen getestet sein.
6. Das Theming MUSS mit verschiedenen Theme-Varianten getestet sein.
7. Das Theming MUSS mit verschiedenen Theme-Wechseln getestet sein.
8. Das Theming MUSS mit verschiedenen Listener-Szenarien getestet sein.

## 12. Anhänge

### 12.1 Referenzierte Dokumente

1. SPEC-ROOT-v1.0.0: NovaDE Spezifikationswurzel
2. SPEC-LAYER-CORE-v1.0.0: Spezifikation der Kernschicht
3. SPEC-LAYER-DOMAIN-v1.0.0: Spezifikation der Domänenschicht

### 12.2 Externe Abhängigkeiten

1. `serde`: Für die Serialisierung und Deserialisierung von Theme-Definitionen
2. `serde_json`: Für das Parsen von JSON-Theme-Definitionen
3. `csscolorparser`: Für das Parsen von Farbwerten
4. `notify`: Für die Überwachung von Theme-Dateien
