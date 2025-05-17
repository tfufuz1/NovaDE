# Infrastruktur
## 1. Kernschicht
Diese Spezifikation beschreibt die fundamentalen Komponenten und Richtlinien für die Entwicklung der Kernschicht der Desktop-Umgebung. Die Kernschicht bildet das Fundament für alle darüberliegenden Schichten und umfasst Module für grundlegende Datentypen (`core::types`), Fehlerbehandlung (`core::errors`), Logging (`core::logging`), Konfigurationsmanagement (`core::config`) und allgemeine Hilfsfunktionen (`core::utils`).

### 1\. Modul: `core::types` (Fundamentale Datentypen)

**1.1. Zweck und Verantwortlichkeit**
Das Modul `core::types` definiert grundlegende, universell einsetzbare Datentypen, die von allen anderen Schichten und Modulen benötigt werden[cite: 3]. Dazu gehören geometrische Primitive, Farbdarstellungen und allgemeine Enumerationen[cite: 4]. Diese Typen sind reine Datenstrukturen ohne komplexe Geschäftslogik oder Abhängigkeiten zu höheren Schichten[cite: 5].

**1.2. Designphilosophie**
Das Design folgt den Prinzipien der Modularität, Wiederverwendbarkeit und minimalen Kopplung[cite: 7]. Typen sind generisch gehalten, wo sinnvoll (z.B. `Point<T>`, `Size<T>`, `Rect<T>`), um Flexibilität für verschiedene numerische Darstellungen zu ermöglichen (z.B. `i32` für Koordinaten, `f32` für Skalierungsfaktoren)[cite: 8]. Es besteht eine klare Trennung von Datenrepräsentation und Fehlerbehandlung[cite: 9].

**1.3. Ziel-Dateistruktur** [cite: 29, 30]

```
core/
└── src/
    ├── lib.rs         # Deklariert Kernmodule: pub mod types; pub mod errors; ...
    └── types/
        ├── mod.rs     # Deklariert und re-exportiert Typen aus geometry.rs, color.rs, etc.
        ├── geometry.rs # Enthält Point<T>, Size<T>, Rect<T>
        ├── color.rs    # Enthält Color
        └── enums.rs    # Enthält Orientation, etc.
```

**1.4. Spezifikation: Geometrische Primitive (`geometry.rs`)**

  * **`Point<T>`**: Repräsentiert einen Punkt im 2D-Raum mit `x: T` und `y: T`[cite: 31, 32].

      * Konstanten wie `ZERO_I32`, `ZERO_F32` etc.[cite: 33, 34, 35, 36].
      * Methoden: `new(x: T, y: T)`, `distance_squared(...)`, `distance(...)` (für Float-Typen), `manhattan_distance(...)`[cite: 37, 38, 40, 41].
      * Basis-Constraints für `T`: `Copy + Debug + PartialEq + Default + Send + Sync + 'static`[cite: 44].

  * **`Size<T>`**: Repräsentiert eine 2D-Dimension mit `width: T` und `height: T`[cite: 45].

      * Konstanten wie `ZERO_I32`, `ZERO_F32` etc.[cite: 46, 47, 48, 49].
      * Methoden: `new(width: T, height: T)`, `area()`, `is_empty()`, `is_valid()` (für nicht-negative Dimensionen)[cite: 50, 51, 52, 53].
      * Basis-Constraints für `T`: `Copy + Debug + PartialEq + Default + Send + Sync + 'static`[cite: 55]. Die Invariante nicht-negativer Dimensionen wird durch `is_valid()` prüfbar gemacht, aber nicht durch den Typ erzwungen[cite: 55].

  * **`Rect<T>`**: Repräsentiert ein 2D-Rechteck, definiert durch `origin: Point<T>` und `size: Size<T>`[cite: 56, 57].

      * Konstanten wie `ZERO_I32`, `ZERO_F32` etc.[cite: 58, 59, 60, 61].
      * Methoden: `new(origin, size)`, `from_coords(x,y,width,height)`, Zugriffsmethoden (`x()`, `y()`, `width()`, `height()`, `top()`, `left()`, `bottom()`, `right()`), `center()`, `contains_point(...)`, `intersects(...)`, `intersection(...)`, `union(...)`, `translated(...)`, `scaled(...)`, `is_valid()`[cite: 62, 63, 64, 65, 66, 67, 68, 69, 70, 71].
      * Basis-Constraints für `T`: `Copy + Debug + PartialEq + Default + Send + Sync + 'static`[cite: 73].
      * **Invariante**: Logisch sollten `width` und `height` nicht-negativ sein[cite: 73]. Die Methode `is_valid()` wird bereitgestellt; Nutzer (besonders mit `T=i32`) sollten diese aufrufen[cite: 80, 81]. Die Verantwortung für das Melden eines Fehlers bei Verwendung eines ungültigen `Rect` liegt beim Aufrufer[cite: 83].

  * **`RectInt`**: (aus einer anderen Quelldatei, aber thematisch passend) Repräsentiert ein achsenparalleles Rechteck mit ganzzahligen Koordinaten (`x: i32`, `y: i32`) und Dimensionen (`width: u32`, `height: u32`)[cite: 566, 567, 572].

      * Methoden u.a. `new(...)`, `from_points(...)`, `top_left()`, `size()`, `right()`, `bottom()`, `contains_point(...)`, `intersects(...)`, `intersection(...)`, `union(...)`, `translate(...)`, `inflate(...)`, `is_empty()`[cite: 572, 573, 576, 577, 578, 580, 581, 583, 585, 588, 591, 593, 596].
      * Verwendet `saturating_add` / `saturating_sub` um Überläufe zu vermeiden[cite: 578, 580, 592, 594].
      * Traits: `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `Default`[cite: 599].

**1.5. Spezifikation: Farbdarstellung (`color.rs`)**

  * **`Color` (RGBA)**: Repräsentiert eine Farbe mit `r: f32`, `g: f32`, `b: f32`, `a: f32` Komponenten im Bereich `[0.0, 1.0]`[cite: 85, 86, 87].
      * Konstanten: `TRANSPARENT`, `BLACK`, `WHITE`, `RED`, `GREEN`, `BLUE` etc.[cite: 88, 89, 90, 91, 92, 93].
      * Methoden: `new(r,g,b,a)` (klemmt Werte nicht automatisch, Aufruferverantwortung)[cite: 95], `from_rgba8(r,g,b,a)`, `to_rgba8()`, `with_alpha(alpha)` (klemmt Alpha), `blend(background)`, `lighten(amount)`, `darken(amount)`[cite: 96, 97, 99, 100, 101, 102, 103].
      * `Default` wird manuell implementiert, um `Color::TRANSPARENT` zurückzugeben[cite: 106, 113].
      * Soll `Serialize` und `Deserialize` implementieren, um als Hex-String (z.B. "\#RRGGBBAA") in Konfigurationsdateien dargestellt zu werden[cite: 603, 610, 622, 623]. Dies erfordert eine `ColorParseError`-Behandlung[cite: 603, 610, 628].

**1.6. Spezifikation: Allgemeine Enumerationen (`enums.rs`)**

  * **`Orientation`**: Repräsentiert eine horizontale oder vertikale Ausrichtung[cite: 107].
      * Varianten: `Horizontal`, `Vertical`[cite: 108].
      * Methoden: `toggle()`[cite: 108].
      * `Default` ist `Orientation::Horizontal`[cite: 109, 113].

**1.7. Standard Trait Implementierungen**
Alle Typen sollen grundlegende Traits wie `Debug`, `Clone`, `Copy` (wo anwendbar und `T` es unterstützt), `PartialEq`, `Default` (sinnvoll definiert), `Send` und `Sync` implementieren[cite: 14, 42, 44, 54, 55, 72, 73, 105, 106, 109, 110, 111, 112]. `Eq` und `Hash` sind für Fließkommazahlen generell nicht geeignet[cite: 113].

**1.8. Modulabhängigkeiten**
Minimale externe Abhängigkeiten: `std`[cite: 25, 26]. Optional `num-traits` (für erweiterte numerische Operationen) und `serde` (mit `derive`-Feature, falls Serialisierung direkt hier benötigt wird, aktuell aber eher in höheren Schichten vorgesehen)[cite: 26, 27, 28].

### 2\. Modul: `core::errors` (Fehlerbehandlung)

**2.1. Zweck und Geltungsbereich**
Spezifiziert die verbindliche Strategie und Implementierung der Fehlerbehandlung innerhalb der Kernschicht[cite: 146]. Ziel ist eine lückenlose, präzise Spezifikation für Entwickler[cite: 149].

**2.2. Grundlagen und Prinzipien**

  * **Verwendung von `thiserror`**: Obligatorisch für die Definition von benutzerdefinierten Fehlertypen[cite: 150, 156, 397]. `thiserror` reduziert Boilerplate-Code für `std::error::Error` und `std::fmt::Display`[cite: 151, 157, 399]. Alle benutzerdefinierten Fehler-Enums in der Kernschicht müssen `thiserror::Error` ableiten[cite: 157].
  * **`Result<T, E>` vs. `panic!`**: Strikte Trennung[cite: 158].
      * `Result<T, E>`: Standard für erwartete, potenziell behebbare Fehlerzustände (z.B. I/O-Fehler, ungültige Eingaben)[cite: 158, 159]. Funktionen müssen `Result<T, E>` zurückgeben, wobei `E` typischerweise `CoreError` oder ein spezifischerer Modul-Fehler ist[cite: 160].
      * `panic!`: Ausschließlich für nicht behebbare Programmierfehler (Bugs), Verletzung von Vorbedingungen oder logisch unmögliche interne Zustände[cite: 161, 361].
  * **Umgang mit `.unwrap()` und `.expect()`**: In produktivem Code der Kernschicht strengstens zu vermeiden, da sie die strukturierte Fehlerbehandlung umgehen[cite: 164, 362, 364].
      * Ausnahme für `expect()`: Nur wenn ein `Err`- oder `None`-Zustand nachweislich einen Bug darstellt (interne Invariante verletzt)[cite: 164, 368]. Die Nachricht muss dem "expect as precondition"-Stil folgen und erklären, *warum* ein `Ok`- oder `Some`-Wert erwartet wurde[cite: 165, 371, 373, 450].
  * **Anforderungen an Fehlermeldungen (`#[error("...")]`)**:
      * Klarheit und Präzision, eindeutige Problembeschreibung[cite: 169, 299, 413].
      * Kontextinformationen durch eingebettete Feldwerte (`{field_name}`)[cite: 170, 300, 414].
      * Zielgruppe: Entwickler (für Logging/Debugging)[cite: 170, 415].
      * Format: Knappe, klein geschriebene Sätze ohne abschließende Satzzeichen (Rust API Guidelines)[cite: 172, 298, 415].
  * **Keine sensiblen Daten in Fehlermeldungen**: Niemals Passwörter, API-Schlüssel, private Benutzerdaten etc. in Fehlermeldungen oder Kontextfeldern[cite: 222, 223, 441]. Daten müssen maskiert, entfernt oder durch Platzhalter ersetzt werden[cite: 224].

**2.3. Strategie: Ein Fehler-Enum pro Modul**
Jedes signifikante Modul innerhalb der Kernschicht (und höheren Schichten) definiert sein eigenes, spezifisches Fehler-Enum mit `thiserror`[cite: 200, 201, 337, 402]. Dies vermeidet Überladung des zentralen `CoreError` und adressiert `thiserror`-Einschränkungen bezüglich mehrdeutiger `#[from]`-Konvertierungen desselben Quelltyps[cite: 173, 175, 202, 205, 403].

**2.4. Definition des Basis-Fehlertyps: `CoreError`**
Ein zentrales, öffentliches Enum `CoreError` in `core::errors` dient als primäre Schnittstelle für Fehler, die von öffentlichen Funktionen der Kernschicht propagiert werden[cite: 178, 179, 290]. Es aggregiert allgemeine Fehlerarten und spezifischere Fehler aus Untermodulen (via `#[from]`)[cite: 180, 184].

  * **Spezifikation `CoreError`** (Beispielvarianten)[cite: 182, 183, 184, 187, 188, 191, 293, 294, 295, 296]:
      * `Io { path: PathBuf, #[source] source: std::io::Error }` [cite: 183, 197, 295]
      * `Configuration(#[from] ConfigError)` [cite: 184, 197]
      * `Serialization { description: String }` [cite: 186, 197]
      * `InvalidId { invalid_id: String }` [cite: 187, 197]
      * `NotFound { resource_description: String }` [cite: 188, 197]
      * `Internal(String)` (sollte vermieden und durch spezifischere Varianten ersetzt werden) [cite: 191, 197]
      * `InitializationFailed { component: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> }` [cite: 293, 309]
  * **Ableitungen**: Mindestens `Debug` und `thiserror::Error`[cite: 193, 194].
  * **Fehlerverkettung (`source()`)**: Wird von `thiserror` automatisch für `#[source]` und `#[from]` annotierte Felder implementiert, um die Ursache zurückzuverfolgen[cite: 194, 195, 303, 326, 431].

**2.5. Modul-spezifische Fehler und Integration**
Module definieren eigene Fehler-Enums (z.B. `ConfigError`, `UtilsError`) die `thiserror::Error` ableiten[cite: 200, 201, 338].

  * **Integrationsmechanismus**: Eine dedizierte Variante in `CoreError`, die den Modul-Fehler kapselt und `#[from]` verwendet, ist der bevorzugte Weg[cite: 206, 209, 239, 349]. Beispiel: `Configuration(#[from] ConfigError)` in `CoreError`.
  * Dies etabliert eine zweistufige Fehlerhierarchie[cite: 209, 210].

**2.6. Fehlerkontext und Diagnose**
Fehlervarianten sollen relevante Kontextinformationen als Felder enthalten (Dateipfade, ungültige Werte etc.)[cite: 215, 216, 421].

**2.7. Implementierungsleitfaden für Entwickler (Fehlerdefinition und -behandlung)**

  * **Neue Variante zu `CoreError` hinzufügen**: Prüfen, ob der Fehlerfall allgemeine Bedeutung hat oder besser in einem Modul-Fehler aufgehoben ist[cite: 225]. Variante, `#[error]`-Meldung und Kontextfelder definieren[cite: 226, 228, 229]. `#[source]` oder `#[from]` für Kapselung verwenden[cite: 230, 231, 232].
  * **Neuen Modul-Fehler erstellen**: `errors.rs` im Modulverzeichnis anlegen[cite: 234]. Enum definieren, `thiserror::Error` ableiten, Varianten und Meldungen spezifizieren[cite: 235, 236]. In `CoreError` über eine `#[from]`-Variante integrieren[cite: 237, 238, 239].
  * **Verwendung des `?`-Operators**: Standard für Fehlerpropagation[cite: 240, 243, 428]. Funktioniert nahtlos bei identischen Fehlertypen oder existierender `From`-Implementierung[cite: 243].
  * **Fehler-Matching (`match`)**: Für spezifische Behandlung (Standardwerte, alternative Pfade, Anreicherung)[cite: 244].
  * **Umgang mit externen Crates**: Fehler von externen Bibliotheken müssen in einen Kernschicht-Fehlertyp (`CoreError` oder Modul-Fehler) gekapselt werden[cite: 253]. Bevorzugt mit `#[from]` oder `#[source]` (manuelle Erzeugung via `.map_err()`)[cite: 254, 255, 256, 257].

### 3\. Modul: `core::logging` (Logging-Infrastruktur)

**3.1. Grundlagen und Wahl von `tracing`**
Die Desktop-Umgebung verwendet das `tracing`-Crate für strukturiertes Logging[cite: 390, 456]. `core::logging` stellt Initialisierungsroutinen bereit[cite: 457].

**3.2. `tracing` Framework Integrationsdetails**

  * **Initialisierung**: Eine Funktion `initialize_logging(level_filter: tracing::LevelFilter, format: LogFormatEnum) -> Result<(), LoggingError>` wird früh im Anwendungsstart aufgerufen[cite: 458, 459]. `LogFormatEnum` könnte `PlainTextDevelopment`, `JsonProduction` definieren[cite: 460]. `LoggingError` ist ein `thiserror`-Enum in `core::logging`[cite: 460].
  * **Subscriber-Konfiguration**:
      * Entwicklung: `tracing_subscriber::fmt()` mit menschenlesbarer Ausgabe (`with_ansi(true)`, `with_target(true)`, `with_file(true)`, `with_line_number(true)`)[cite: 461, 462].
      * Release: Strukturiertes JSON-Format für Log-Aggregation und maschinelle Analyse (`tracing_subscriber::fmt::json()` oder `tracing-bunyan-formatter`)[cite: 462].
  * **Dynamische Log-Level-Änderungen**: Für zukünftige Erweiterungen berücksichtigen (z.B. via `tracing_subscriber::filter::EnvFilter` oder `RUST_LOG`)[cite: 464, 465].

**3.3. Standardisierte Log-Makros und `tracing::instrument` Verwendung**

  * **Standard-Makros**: Direkte Verwendung von `trace!`, `debug!`, `info!`, `warn!`, `error!` ist verbindlich[cite: 466].
  * **Log-Nachrichtenstruktur**: Prägnant und beschreibend[cite: 467]. Schlüssel-Wert-Paare für strukturierte Daten: `tracing::info!(user_id = %user.id, " Nachricht")` (% für Display, ? für Debug)[cite: 467]. Fehler mit `error = ?err` loggen, um die Debug-Repräsentation (inkl. `source`-Kette) zu erfassen[cite: 468, 469].
  * **`#[tracing::instrument]` Verwendung**: Erzeugt Spans für Funktionen/Codeblöcke, gruppiert Log-Ereignisse[cite: 470].
      * Anwendung auf öffentliche API-Funktionen, I/O-Operationen, komplexe Berechnungen, abgeschlossene operative Einheiten[cite: 471, 472].
      * `skip(...)` / `skip_all` für sensible/ausführliche Argumente[cite: 473, 488].
      * `fields(...)` für spezifischen Kontext im Span[cite: 474].
      * `err` zur automatischen Fehlerprotokollierung bei `Result::Err`[cite: 475].
      * `level` zur Steuerung des Span-Levels[cite: 476].

**3.4. Logging von Fehlern**
Jeder Fehler (`Result::Err`) sollte an seiner Ursprungsstelle oder einer geeigneten übergeordneten Stelle mit ausreichend Kontext geloggt werden, mindestens auf `ERROR`-Level (`tracing::error!`)[cite: 261, 262]. Dies sollte typischerweise *vor* der Propagation geschehen[cite: 263, 264]. Den Fehler selbst als strukturiertes Feld mitgeben: `error!(error = %core_err, "Nachricht")`[cite: 266, 270, 271].

**3.5. Log-Daten Sensibilität**
Absolutes Verbot, sensible Daten (Passwörter, API-Schlüssel, PII etc.) im Klartext zu loggen[cite: 441, 483]. Daten redigieren oder auslassen[cite: 484]. Vorsicht bei `Debug`-Implementierungen für Strukturen mit sensiblen Daten; ggf. manuelle Redaktion in `Debug` oder `skip_all` in `#[tracing::instrument]` verwenden[cite: 485, 486, 487, 488].

### 4\. Modul: `core::config` (Konfigurationsprimitive)

**4.1. Zweck**
Definiert, wie grundlegende Konfigurationseinstellungen geladen, geparst und zugegriffen werden[cite: 391, 495]. Fokus auf Einfachheit, Robustheit[cite: 496].

**4.2. Konfigurationsdateiformat und Parsing-Logik**

  * **Format**: TOML (Tom's Obvious, Minimal Language) wegen Lesbarkeit und einfacher Verarbeitung[cite: 497].
  * **Parsing-Bibliothek**: `serde` in Verbindung mit `toml`-Crate (`serde_toml`)[cite: 498].
  * **Ladelogik**:
      * Definition von Standard-Konfigurationspfaden (z.B. systemweit, Entwicklungstests)[cite: 499]. XDG-Pfade für benutzerspezifische Konfigurationen in höheren Schichten berücksichtigen[cite: 500].
      * Eine Funktion wie `load_core_config(custom_path: Option<PathBuf>) -> Result<CoreConfig, ConfigError>` implementiert eine Suchreihenfolge, liest und deserialisiert die TOML-Datei[cite: 501, 502, 503, 504].
      * Fehlerbehandlung mit `core::config::ConfigError` (definiert mit `thiserror`), Varianten wie `FileReadError`, `DeserializationError`, `NoConfigurationFileFound`[cite: 504, 505, 506, 507, 508].

**4.3. Konfigurationsdatenstrukturen (Ultra-Fein)**

  * **`CoreConfig`-Struktur**: Eine primäre Struktur (z.B. `CoreConfig`) hält alle spezifischen Konfigurationen der Kernschicht[cite: 509, 510, 511, 512, 513, 514].
      * Felder mit explizit definierten Typen[cite: 514].
      * Muss `serde::Deserialize` ableiten[cite: 515].
      * `#[serde(default = "path")]` oder `#[serde(default)]` umfassend verwenden für Standardwerte bei fehlenden Feldern[cite: 511, 512, 513, 516, 517].
      * `#[serde(deny_unknown_fields)]` erzwingen, um Tippfehler oder unbekannte Felder in Konfigurationsdateien zu verhindern[cite: 512, 513, 518].
  * **Validierung**: Grundlegende Validierung durch Typen[cite: 519]. Komplexere Validierungen nach Deserialisierung (z.B. via `TryFrom` Muster oder `validate()`-Methode)[cite: 520, 521]. Für Kernschicht kann initiale Validierung auf `serde`-Fähigkeiten beschränkt sein[cite: 522].

**4.4. Konfigurationszugriffs-API**

  * **Globaler Zugriff**: Geladene `CoreConfig`-Instanz threadsicher speichern, typischerweise mittels `once_cell::sync::OnceCell`[cite: 530, 531, 532].
      * `initialize_core_config(config: CoreConfig) -> Result<(), CoreConfig>` zum einmaligen Setzen[cite: 532, 533, 534].
      * `get_core_config() -> &'static CoreConfig` für den Zugriff; paniert, wenn nicht initialisiert (Programmierfehler)[cite: 535, 536].
  * **Immutabilität**: Global zugängliche Konfiguration sollte nach Initialisierung unveränderlich sein[cite: 541]. `CoreConfig` sollte `Clone` ableiten für Momentaufnahmen oder Tests[cite: 540, 543, 544].

### 5\. Modul: `core::utils` (Allgemeine Hilfsfunktionen)

**5.1. Zweck**
Beherbergt allgemeine Hilfsfunktionen und kleine, in sich geschlossene Utilities, die nicht in spezifischere Module passen, aber breit verwendet werden[cite: 392, 546].

**5.2. Allgemeine Richtlinien** [cite: 556, 557, 558, 559, 560, 561, 562]

  * **Geltungsbereich**: Nur wirklich allgemeine Utilities.
  * **Einfachheit**: Einfache Funktionen bevorzugen.
  * **Reinheit**: Reine Funktionen (Ausgabe hängt nur von Eingabe ab, keine Seiteneffekte) bevorzugen.
  * **Fehlerbehandlung**: Jede fehleranfällige Utility-Funktion gibt `Result<T, YourUtilError>` zurück, wobei `YourUtilError` mit `thiserror` im Utility-Submodul definiert wird.
  * **Dokumentation**: Umfassende `rustdoc`-Kommentare mit Beispielen.
  * **Tests**: Gründliche Unit-Tests.

### 6\. Allgemeine Entwicklungsrichtlinien (Kernschicht)

**6.1. Dokumentation (`rustdoc`)** [cite: 133, 134, 135, 136, 137, 138, 139]
Alle öffentlichen Elemente (Module, Structs, Enums, Felder, Konstanten, Methoden) müssen `///`-Dokumentationskommentare haben.

  * Modul-Level: Zweck des Moduls.
  * Typ-Level: Zweck und Invarianten.
  * Feld-Level: Bedeutung des Feldes.
  * Methoden-Level: Was die Methode tut, Parameter, Rückgabewerte, mögliche Panics (idealerweise keine außer in Tests), Vor-/Nachbedingungen, Algorithmen.
  * `# Examples`-Abschnitte verwenden.
  * Strikte Einhaltung der Rust API Guidelines.
  * `cargo doc --open` zur Überprüfung.

**6.2. Unit-Testing** [cite: 128, 129, 130, 131, 132]

  * Ein `#[cfg(test)]`-Modul innerhalb jeder Implementierungsdatei.
  * Tests für Konstruktoren, Konstanten, Methodenlogik, Grenzfälle, Trait-Implementierungen, Invariantenprüfungen.
  * Anstreben einer hohen Testabdeckung.

**6.3. Immutabilität und Stabilität**
Die API der Kernschicht sollte nach Stabilisierung als äußerst stabil behandelt werden[cite: 643]. Änderungen haben weitreichende Auswirkungen[cite: 644]. Komponenten sind so konzipiert, dass sie `Send + Sync` sind, wo sinnvoll, für Multithreading[cite: 644].

**6.4. Schichtübergreifende Integrationsrichtlinien** [cite: 633, 634, 635, 636, 637, 638, 639, 640, 641, 642]

  * **Fehlerbehandlung**: Höhere Schichten definieren eigene `thiserror`-Enums. Fehler aus der Kernschicht werden behandelt oder via `?` propagiert (ggf. mit `#[from]` in eigene Fehlertypen konvertiert), Fehlerkette (`source()`) muss erhalten bleiben.
  * **Logging**: Alle Schichten nutzen `tracing`-Makros. `core::logging::initialize_logging()` wird vom Hauptbinary aufgerufen. Einhaltung von Log-Leveln und Datensensibilität ist zwingend.
  * **Konfiguration**: Höhere Schichten können eigene Konfigs definieren. Zugriff auf Kern-Konfig via `core::config::get_core_config()`. Kern-Konfig nicht zur Laufzeit modifizieren.
  * **Typen und Utilities**: Kerndatentypen und -utilities direkt verwenden. Bei Spezialisierung Komposition oder Newtype-Wrapper um Kerntypen in Betracht ziehen.

Diese Spezifikation legt den Grundstein für eine robuste, wartbare und performante Kernschicht. Die disziplinierte Einhaltung dieser Richtlinien ist für den Erfolg des Projekts entscheidend[cite: 284, 632, 645].
## 2. Domänenschicht

### 1. Allgemeine Prinzipien und Entwicklungsrichtlinien der Domänenschicht

Die Domänenschicht ist das Herzstück der Anwendungslogik und repräsentiert die Geschäftsregeln und -konzepte der Desktop-Umgebung. Sie ist UI-unabhängig und entkoppelt von spezifischen Systemdetails oder Infrastrukturbelangen. [cite: 4, 332, 1105]

**Entwicklungsrichtlinien:**

* **Sprache und Tooling:** Rust wird als primäre Programmiersprache verwendet.
    * **Fehlerbehandlung:** `thiserror` wird für die Definition spezifischer, benutzerdefinierter Fehler-Enums pro Modul verwendet. [cite: 179, 379, 821, 1150] Dies ermöglicht eine klare Kommunikation von Fehlerzuständen. [cite: 180] Fehler werden über `Result<T, E>` zurückgegeben; `unwrap()` und `expect()` sind zu vermeiden, außer in absoluten Ausnahmefällen. [cite: 181, 199, 200, 686] Die `source()`-Kette von Fehlern soll durch korrekte Verwendung von `#[source]` und `#[from]` erhalten bleiben. [cite: 184, 204, 388, 518, 683, 829]
    * **Serialisierung/Deserialisierung:** `serde` (mit `serde_json` für JSON) wird für das Laden und Speichern von Konfigurationen und Datenstrukturen verwendet. [cite: 6, 50, 910] Attribute wie `#[serde(rename_all = "kebab-case")]`, `#[serde(default)]` und `#[serde(skip_serializing_if = "Option::is_none")]` sollen konsistent genutzt werden. [cite: 13, 35, 233, 914, 1034]
    * **Asynchronität:** Wo Operationen potenziell blockierend sind (z.B. I/O beim Laden von Konfigurationen, Kommunikation mit externen Diensten), werden `async/await` und `async_trait` verwendet. [cite: 757, 935, 1133, 1191] Für nebenläufigen Zugriff auf geteilte Zustände sind `tokio::sync` Mechanismen wie `RwLock` und `Mutex` einzusetzen. [cite: 117, 1137, 1180, 1252, 1286]
    * **Eindeutige IDs:** `uuid` (Version 4) wird zur Generierung eindeutiger Identifikatoren für Entitäten verwendet. [cite: 344, 732, 1108, 1227]
    * **Zeitstempel:** `chrono::DateTime<Utc>` wird für Zeitstempel verwendet, um Konsistenz zu gewährleisten. [cite: 352, 713, 1108, 1227]
    * **Event-Handling:** `tokio::sync::broadcast` wird für ein entkoppeltes, internes Event-System genutzt, um Änderungen an andere Systemteile zu kommunizieren. [cite: 119, 1135, 1255]
* **Modularität und Kohäsion:** Die Domänenschicht ist in klar abgegrenzte Module unterteilt, die jeweils spezifische Verantwortlichkeiten haben (z.B. `domain::theming`, `domain::workspaces`, `domain::user_centric_services`, `domain::global_settings_and_state_management`, `domain::notifications_core`, `domain::notifications_rules`). [cite: 1, 333, 697, 895, 1097] Jedes Modul sollte eine hohe Kohäsion aufweisen und lose mit anderen Modulen gekoppelt sein.
* **Typsicherheit:** Newtypes und spezifische Enums werden verwendet, um die Typsicherheit zu erhöhen und die Semantik von Daten klarer zu gestalten (z.B. `TokenIdentifier`[cite: 8, 9], `WorkspaceId`[cite: 364, 365], `SettingKey` [cite: 1108, 1109]).
* **Abstraktion und Schnittstellen:** Öffentliche APIs von Modulen werden oft durch Traits definiert, um Implementierungsdetails zu kapseln und Testbarkeit durch Mocking zu ermöglichen (z.B. `AIInteractionLogicService`[cite: 755, 757], `NotificationService`[cite: 755, 770], `GlobalSettingsService`[cite: 933, 937], `SettingsProvider` [cite: 1103, 1185]).
* **Zustandsverwaltung:** Veränderliche Zustände innerhalb von Services werden typischerweise mit `Arc<Mutex<...>>` oder `Arc<RwLock<...>>` gekapselt, um Thread-Sicherheit zu gewährleisten. [cite: 117, 1137]
* **Validierung:** Eingabedaten und Einstellungsänderungen werden aktiv validiert, um die Konsistenz und Integrität der Domänendaten sicherzustellen. [cite: 51, 53, 59, 346, 900, 1102]
* **Logging:** Das `tracing`-Framework soll für strukturiertes Logging und Debugging verwendet werden. [cite: 61, 257, 873, 1059, 1311]
* **Dokumentation:** Öffentliche Typen, Methoden und Felder müssen umfassend mit `rustdoc`-Kommentaren dokumentiert werden, inklusive Vor- und Nachbedingungen, Fehler und Beispiele. [cite: 415, 416, 555]
* **Testbarkeit:** Unit-Tests sind parallel zur Implementierung zu erstellen und sollen eine hohe Codeabdeckung anstreben. [cite: 230, 317, 868, 885, 1053] Mocking von Abhängigkeiten (insbesondere von Schnittstellen zur Kern- oder Systemschicht) ist entscheidend. [cite: 325, 557, 892, 1080]

### 2. Struktur und Kernkomponenten der Domänenschicht

Die Domänenschicht besteht aus mehreren Kernmodulen, die spezifische Aufgabenbereiche abdecken:

#### 2.1. Modul: `domain::theming`

* **Verantwortlichkeit:** Logik des Erscheinungsbilds (Theming), Verwaltung von Design-Tokens, Interpretation von Theme-Definitionen, dynamische Theme-Wechsel (Farbschema, Akzentfarben). [cite: 1, 2, 3]
* **Datenstrukturen:**
    * `TokenIdentifier` (String-Wrapper für hierarchische Token-IDs wie "color.background.primary"). [cite: 8, 9]
    * `TokenValue` (Enum für Token-Wertetypen: Color, Dimension, FontSize, FontFamily, FontWeight, LineHeight, LetterSpacing, Border, Shadow, Radius, Spacing, ZIndex, Opacity, Text, Reference zu anderem Token). [cite: 11, 12, 13, 14]
    * `RawToken` (Struct: id, value, optionale description, group). [cite: 15, 16, 39]
    * `TokenSet` (Typalias für `HashMap<TokenIdentifier, RawToken>`). [cite: 17]
    * `ThemeIdentifier` (String-Wrapper für Theme-IDs). [cite: 18, 19]
    * `ColorSchemeType` (Enum: Light, Dark). [cite: 20]
    * `AccentColor` (Struct: optionaler name, value als CSS-Farbwert). [cite: 21]
    * `ThemeVariantDefinition` (Struct: applies_to_scheme, tokens als TokenSet für Überschreibungen). [cite: 22, 23]
    * `ThemeDefinition` (Struct: id, name, description, author, version, base_tokens, variants, supported_accent_colors). [cite: 24, 25, 26, 41]
    * `AppliedThemeState` (Struct: theme_id, color_scheme, active_accent_color, resolved_tokens als `HashMap<TokenIdentifier, String>`). [cite: 27, 28, 29, 31, 43]
    * `ThemingConfiguration` (Struct: selected_theme_id, preferred_color_scheme, selected_accent_color, custom_user_token_overrides). [cite: 32, 33, 34]
* **Kernlogik (`ThemingEngine` Service):** [cite: 112, 114]
    * Laden, Parsen und Validieren von Token- (*.tokens.json) und Theme-Definitionen (*.theme.json) von standardisierten Pfaden (System- und Benutzer-spezifisch). [cite: 46, 47, 48, 56] Validierung beinhaltet Eindeutigkeit von Token-IDs und Erkennung zyklischer Referenzen. [cite: 51, 53, 54]
    * Token Resolution Pipeline: Auflösung von Token-Referenzen und Anwendung von Überschreibungen (Theme-Basis, Variante, Akzentfarbe, Benutzer-Overrides) in definierter Reihenfolge. [cite: 64, 65, 66, 67] Ergebnis ist der `AppliedThemeState`.
    * Dynamische Theme-Wechsel basierend auf Änderungen in `ThemingConfiguration`. [cite: 99, 100]
    * Caching von aufgelösten `AppliedThemeState`s. [cite: 96, 97]
* **Öffentliche API (`ThemingEngine`):**
    * `new(initial_config, theme_load_paths, token_load_paths)`: Konstruktor. [cite: 129]
    * `get_current_theme_state()`: Gibt aktuellen `AppliedThemeState` zurück. [cite: 125, 137]
    * `get_available_themes()`: Gibt `Vec<ThemeDefinition>` zurück. [cite: 126, 141]
    * `get_current_configuration()`: Gibt aktuelle `ThemingConfiguration` zurück. [cite: 127, 143]
    * `update_configuration(new_config)`: Aktualisiert Konfiguration und löst Neuberechnung aus. [cite: 144]
    * `reload_themes_and_tokens()`: Lädt alle Definitionen neu. [cite: 150]
    * `subscribe_to_theme_changes()`: Gibt einen `mpsc::Receiver<ThemeChangedEvent>` zurück. [cite: 157]
* **Events:** `ThemeChangedEvent { new_state: AppliedThemeState }`. [cite: 103, 166, 167, 168, 177]
* **Fehlerbehandlung:** `ThemingError` Enum (z.B. `TokenFileParseError`, `CyclicTokenReference`, `ThemeNotFound`, `MissingTokenReference`). [cite: 182, 186, 187, 188, 189, 190, 191, 209]
* **Dateistruktur:** `domain/theming/{mod.rs, types.rs, errors.rs, logic.rs, default_themes/}`. [cite: 211, 212, 213]

#### 2.2. Modul: `domain::workspaces`

Verantwortlich für die Logik und Verwaltung von Arbeitsbereichen ("Spaces" oder virtuelle Desktops). [cite: 330] Unterteilt in `core`, `assignment`, `manager`, und `config`. [cite: 333]

* **`workspaces::core`**: Fundamentale Workspace-Definition. [cite: 337]
    * **Datenstrukturen:**
        * `WorkspaceId` (Typalias für `uuid::Uuid`). [cite: 344, 365]
        * `WindowIdentifier` (Newtype für `String`, repräsentiert Fenster-IDs). [cite: 344, 350, 353, 354]
        * `WorkspaceLayoutType` (Enum: Floating, TilingHorizontal, TilingVertical, Maximized; Default: Floating). [cite: 344, 349, 361, 362]
        * `Workspace` (Struct: id, name, persistent_id, layout_type, window_ids: `HashSet<WindowIdentifier>`, created_at). [cite: 343, 344] Validierungen für `name` (nicht leer, Maximallänge) und `persistent_id`. [cite: 346, 348]
    * **API (`impl Workspace`):** `new()`, `id()`, `name()`, `rename()`, `layout_type()`, `set_layout_type()`, `add_window_id()` (crate-intern), `remove_window_id()` (crate-intern), `window_ids()`, `persistent_id()`, `set_persistent_id()`, `created_at()`. [cite: 367]
    * **Event-Payloads (Definiert in `core::event_data`):** `WorkspaceRenamedData`, `WorkspaceLayoutChangedData`, `WindowAddedToWorkspaceData`, `WindowRemovedFromWorkspaceData`, `WorkspacePersistentIdChangedData`. [cite: 375, 376, 377]
    * **Fehlerbehandlung:** `WorkspaceCoreError` (z.B. `InvalidName`, `NameCannotBeEmpty`, `NameTooLong`, `InvalidPersistentId`). [cite: 378, 380, 382, 383, 399]
* **`workspaces::assignment`**: Logik zur Fensterzuweisung. [cite: 334, 422]
    * **API (Freistehende Funktionen):**
        * `assign_window_to_workspace(workspaces: &mut HashMap<WorkspaceId, Workspace>, target_workspace_id, window_id, ensure_unique_assignment: bool)` [cite: 438, 439]
        * `remove_window_from_workspace(workspaces: &mut HashMap<WorkspaceId, Workspace>, source_workspace_id, window_id)` [cite: 439]
        * `move_window_to_workspace(workspaces: &mut HashMap<WorkspaceId, Workspace>, source_workspace_id, target_workspace_id, window_id)` [cite: 439]
        * `find_workspace_for_window(workspaces: &HashMap<WorkspaceId, Workspace>, window_id) -> Option<WorkspaceId>` [cite: 439]
    * **Fehlerbehandlung:** `WindowAssignmentError` (z.B. `WorkspaceNotFound`, `WindowAlreadyAssigned`, `WindowNotOnSourceWorkspace`, `CannotMoveToSameWorkspace`, `RuleViolation`). [cite: 446, 447, 448, 449, 457]
* **`workspaces::manager`**: Orchestrierung und übergeordnete Verwaltung. [cite: 335, 479]
    * **Zustand (`WorkspaceManager` Struct):** `workspaces: HashMap<WorkspaceId, Workspace>`, `active_workspace_id: Option<WorkspaceId>`, `ordered_workspace_ids: Vec<WorkspaceId>`, `next_workspace_number`, `config_provider: Arc<dyn WorkspaceConfigProvider>`, `event_publisher: Arc<dyn EventPublisher<WorkspaceEvent>>`, `ensure_unique_window_assignment: bool`. [cite: 497, 500]
    * **API (`impl WorkspaceManager`):** `new()`, `create_workspace()`, `delete_workspace()`, `get_workspace()`, `get_workspace_mut()`, `all_workspaces_ordered()`, `active_workspace_id()`, `set_active_workspace()`, `assign_window_to_active_workspace()`, `assign_window_to_specific_workspace()`, `remove_window_from_its_workspace()`, `move_window_to_specific_workspace()`, `rename_workspace()`, `set_workspace_layout()`, `save_configuration()`. [cite: 502]
    * **Events (`WorkspaceEvent` Enum):** `WorkspaceCreated`, `WorkspaceDeleted`, `ActiveWorkspaceChanged`, `WorkspaceRenamed`, `WorkspaceLayoutChanged`, `WindowAddedToWorkspace`, `WindowRemovedFromWorkspace`, `WorkspaceOrderChanged`, `WorkspacesReloaded`, `WorkspacePersistentIdChanged`. [cite: 504, 505, 506, 507, 508, 512]
    * **Fehlerbehandlung:** `WorkspaceManagerError` (z.B. `WorkspaceNotFound`, `CannotDeleteLastWorkspace`, `NoActiveWorkspace`, Wraps: `WorkspaceCoreError`, `WindowAssignmentError`, `WorkspaceConfigError`). [cite: 513, 514, 515, 516, 522]
* **`workspaces::config`**: Konfigurations- und Persistenzlogik. [cite: 335, 557]
    * **Datenstrukturen (Snapshots für Persistenz):**
        * `WorkspaceSnapshot` (Struct: persistent_id, name, layout_type). [cite: 564, 565, 566, 568, 569]
        * `WorkspaceSetSnapshot` (Struct: workspaces: `Vec<WorkspaceSnapshot>`, active_workspace_persistent_id). [cite: 570, 571, 572]
    * **Schnittstelle (`WorkspaceConfigProvider` Trait):**
        * `load_workspace_config() -> Result<WorkspaceSetSnapshot, WorkspaceConfigError>` [cite: 573, 575, 581]
        * `save_workspace_config(config_snapshot: &WorkspaceSetSnapshot) -> Result<(), WorkspaceConfigError>` [cite: 576, 581]
    * **Beispielimplementierung:** `FilesystemConfigProvider` (nutzt `core::config::ConfigService`). [cite: 577, 578]
    * **Fehlerbehandlung:** `WorkspaceConfigError` (z.B. `LoadError`, `SaveError`, `InvalidData`, `SerializationError`, `DeserializationError`, `PersistentIdNotFound`, `DuplicatePersistentId`). [cite: 587, 588, 589, 590, 591, 592, 602]

#### 2.3. Modul: `domain::user_centric_services`

Bündelt Logik für KI-Interaktionen (inkl. Einwilligungsmanagement) und ein umfassendes Benachrichtigungssystem. [cite: 697, 698, 700]

* **KI-Interaktionsmanagement:**
    * **Datenstrukturen:**
        * `AIInteractionContext` (Struct: id: Uuid, creation_timestamp, active_model_id, consent_status: `AIConsentStatus`, associated_data_categories: `Vec<AIDataCategory>`, interaction_history, attachments: `Vec<AttachmentData>`). [cite: 712, 713, 714, 715, 716, 717, 751]
        * `AIConsent` (Struct: id: Uuid, user_id, model_id, data_categories: `Vec<AIDataCategory>`, granted_timestamp, expiry_timestamp, is_revoked). [cite: 720, 721, 722, 723, 724, 725, 751]
        * `AIModelProfile` (Struct: model_id, display_name, description, provider, required_consent_categories: `Vec<AIDataCategory>`, capabilities). [cite: 727, 728, 729, 730, 751]
        * `AttachmentData` (Struct: id: Uuid, mime_type, source_uri, content, description). [cite: 717, 742, 743, 744, 745, 751]
        * `AIConsentStatus` (Enum: Granted, Denied, PendingUserAction, NotRequired). [cite: 714, 745]
        * `AIDataCategory` (Enum: UserProfile, ApplicationUsage, FileSystemRead, ClipboardAccess, LocationData, GenericText, GenericImage). [cite: 701, 715, 723, 729, 745, 746]
    * **API (`AIInteractionLogicService` Trait):** `initiate_interaction()`, `get_interaction_context()`, `provide_consent()`, `get_consent_for_model()`, `add_attachment_to_context()`, `list_available_models()`, `store_consent()`, `get_all_user_consents()`, `load_model_profiles()`. [cite: 748, 755, 757, 758, 759, 760, 761, 762, 763, 764, 765, 766, 767, 768, 769]
    * **Events:** `AIInteractionInitiatedEvent`, `AIConsentUpdatedEvent`. [cite: 786, 803, 805, 818]
    * **Fehlerbehandlung:** `AIInteractionError` (z.B. `ContextNotFound`, `ConsentRequired`, `ModelNotFound`, `ConsentStorageError`, `ModelProfileLoadError`). [cite: 822, 823, 824, 825, 839]
* **Benachrichtigungsmanagement:**
    * **Datenstrukturen:**
        * `Notification` (Struct: id: Uuid, application_name, application_icon, summary, body, actions: `Vec<NotificationAction>`, urgency: `NotificationUrgency`, timestamp, is_read, is_dismissed, transient). [cite: 703, 732, 733, 734, 735, 736, 737, 738, 751]
        * `NotificationAction` (Struct: key, label, action_type: `NotificationActionType`). [cite: 704, 735, 740, 741, 751]
        * `NotificationUrgency` (Enum: Low, Normal, Critical). [cite: 704, 735, 745, 746]
        * `NotificationActionType` (Enum: Callback, OpenLink). [cite: 742, 745, 746]
        * `NotificationFilterCriteria` (Enum: Unread, Application(String), Urgency(NotificationUrgency)). [cite: 747, 776]
        * `NotificationSortOrder` (Enum: TimestampAscending, TimestampDescending, Urgency). [cite: 747, 776]
    * **API (`NotificationService` Trait):** `post_notification()`, `get_notification()`, `mark_as_read()`, `dismiss_notification()`, `get_active_notifications()`, `get_notification_history()`, `clear_history()`, `set_do_not_disturb()`, `is_do_not_disturb_enabled()`, `invoke_action()`. [cite: 749, 770, 771, 772, 773, 774, 775, 776, 777, 778, 779, 780, 781, 782]
    * **Events:** `NotificationPostedEvent`, `NotificationDismissedEvent`, `NotificationReadEvent`, `DoNotDisturbModeChangedEvent`. [cite: 808, 809, 810, 811, 812, 813, 814, 815, 816, 818]
    * **Fehlerbehandlung:** `NotificationError` (z.B. `NotFound`, `InvalidData`, `HistoryFull`, `ActionNotFound`). [cite: 822, 826, 827, 839]
* **Dateistruktur:** `domain/user_centric_services/{mod.rs, ai_interaction_service.rs, notification_service.rs, types.rs, errors.rs}`. [cite: 842]

#### 2.4. Modul: `domain::global_settings_and_state_management` (auch `domain::settings_core` + `domain::settings_persistence_iface`)

Verantwortlich für die Repräsentation, Verwaltung und Konsistenz globaler Desktop-Einstellungen. [cite: 895, 896, 897, 1099, 1100]

* **`domain::settings_core`**: Kernlogik der Einstellungsverwaltung. [cite: 1099]
    * **Datenstrukturen:**
        * `SettingKey` (Newtype für `String`, für Einstellungsschlüssel wie "appearance.theme.name"). [cite: 1101, 1108, 1109, 1110]
        * `SettingValue` (Enum: Boolean, Integer, Float, String, Color, FilePath, List, Map). [cite: 1101, 1111, 1112, 1113]
        * `SettingMetadata` (Struct: description, default_value, value_type_hint, possible_values, validation_regex, min_value, max_value, is_sensitive, requires_restart). [cite: 1101, 1115, 1119]
        * `Setting` (Struct: id: Uuid, key, current_value, metadata, last_modified, is_dirty). [cite: 1101, 1120, 1121, 1122]
        * `GlobalDesktopSettings` (Hauptstruktur, die alle globalen Einstellungen kategorisiert, z.B. `AppearanceSettings`, `WorkspaceSettings`, `InputBehaviorSettings`, `PowerManagementPolicySettings`, `DefaultApplicationsSettings`). [cite: 899, 911, 913] Jede Unterstruktur enthält spezifische Einstellungsfelder.
        * `SettingPath` (Enum-Hierarchie zur typsicheren Adressierung von Einstellungen, z.B. `SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize))`). [cite: 923, 924, 925]
    * **API (`SettingsCoreManager` oder `GlobalSettingsService` Trait):**
        * `new(provider, initial_metadata, event_channel_capacity)` / `load_settings()` [cite: 1137, 1141, 937, 938, 939]
        * `save_settings()` [cite: 940]
        * `get_current_settings()` / `get_setting_value(key)` / `get_setting(path)` [cite: 941, 945, 946, 1141]
        * `set_setting_value(key, value)` / `update_setting(path, value: JsonValue)` [cite: 942, 943, 944, 1141]
        * `reset_setting_to_default(key)` / `reset_to_defaults()` [cite: 947, 1141]
        * `register_setting_metadata(key, metadata)` [cite: 1141]
        * `get_all_settings_with_metadata()` [cite: 1141]
        * `subscribe_to_changes()` / `subscribe_to_setting_changes()` [cite: 948, 949, 950, 951, 1141]
    * **Events:** `SettingChangedEvent { key/path, new_value }`[cite: 1104, 1136, 1142, 1147, 986], `SettingsLoadedEvent { settings }`[cite: 972, 993], `SettingsSavedEvent`[cite: 997].
    * **Fehlerbehandlung:** `SettingsCoreError` / `GlobalSettingsError` (z.B. `SettingNotFound`, `ValidationError`, `PersistenceError`, `PathNotFound`, `InvalidValueType`). [cite: 1126, 1149, 1150, 1151, 1152, 1155, 1005, 1010, 1011, 1012, 1013, 1014, 1027]
* **`domain::settings_persistence_iface`**: Persistenzabstraktion. [cite: 1097, 1184]
    * **Schnittstelle (`SettingsProvider` Trait):**
        * `load_setting(key) -> Result<Option<SettingValue>, SettingsPersistenceError>` [cite: 1192, 1196]
        * `save_setting(key, value) -> Result<(), SettingsPersistenceError>` [cite: 1193, 1196]
        * `load_all_settings() -> Result<Vec<(SettingKey, SettingValue)>, SettingsPersistenceError>` [cite: 1193, 1196]
        * `delete_setting(key) -> Result<(), SettingsPersistenceError>` [cite: 1194, 1196]
        * `setting_exists(key) -> Result<bool, SettingsPersistenceError>` [cite: 1194, 1196]
    * **Fehlerbehandlung:** `SettingsPersistenceError` (z.B. `BackendUnavailable`, `StorageAccessError`, `SerializationError`, `DeserializationError`, `IoError`). [cite: 1186, 1201, 1202, 1203, 1206]
* **Dateistruktur (Global Settings):** `domain/global_settings_management/{mod.rs, service.rs, types.rs, paths.rs, errors.rs}`. [cite: 1029, 1030]
* **Dateistruktur (Settings Core & Persistence Interface):** `domain/src/settings_core/{mod.rs, types.rs, error.rs}`, `domain/src/settings_persistence_iface/{mod.rs, error.rs}`. [cite: 1106, 1184, 1190, 1201]

#### 2.5. Modul: `domain::notifications_rules`

Implementiert die Logik zur dynamischen Verarbeitung von Benachrichtigungen basierend auf konfigurierbaren Regeln. [cite: 1097, 1288]

* **Verantwortlichkeit:** Definition von Benachrichtigungsregeln (`NotificationRule`), deren Bedingungen (`RuleCondition`) und Aktionen (`RuleAction`); Bereitstellung einer Engine (`NotificationRulesEngine`) zur Regelauswertung und -anwendung. [cite: 1288, 1289]
* **Datenstrukturen:**
    * `RuleCondition` (Enum: AppNameIs, AppNameMatches (Regex), SummaryContains, UrgencyIs, CategoryIs, HintExists, HintValueIs, SettingIsTrue, LogicalAnd, LogicalOr, LogicalNot etc.). [cite: 1288, 1300, 1301, 1302]
    * `RuleAction` (Enum: SuppressNotification, SetUrgency, AddAction, SetHint, PlaySound, MarkAsPersistent, SetExpiration, LogMessage etc.). [cite: 1288, 1303, 1304, 1305]
    * `NotificationRule` (Struct: id, description, conditions: `RuleCondition`, actions: `Vec<RuleAction>`, is_enabled, priority, stop_after). [cite: 1288, 1306, 1307, 1308]
* **Kernlogik (`NotificationRulesEngine` Service):**
    * Lädt und verwaltet Regeldefinitionen (sortiert nach Priorität). [cite: 1292, 1313, 1314]
    * `process_notification(notification)`: Wertet Regeln gegen eine eingehende Benachrichtigung aus. [cite: 1320]
        * Gibt `RuleProcessingResult` zurück: `Allow(modified_notification)` oder `Suppress(rule_id)`. [cite: 1312]
    * `evaluate_condition(condition, notification, rule)`: Rekursive Auswertung von Regelbedingungen. [cite: 1322, 1330] Interagiert mit `SettingsCoreManager` für `Setting*`-Bedingungen. [cite: 1333]
    * `apply_action(action, notification, rule)`: Anwendung von Regelaktionen auf eine Benachrichtigung. [cite: 1324, 1344]
    * Reagiert auf `SettingChangedEvent` (optional, zur Cache-Invalidierung oder Neubewertung). [cite: 1291, 1315, 1316, 1317, 1362, 1363, 1364]
* **Öffentliche API (`NotificationRulesEngine`):**
    * `new(settings_manager, initial_rules, settings_event_receiver)` [cite: 1312, 1313, 1369]
    * `load_rules(new_rules)` [cite: 1318, 1319, 1369]
    * `process_notification(notification) -> Result<RuleProcessingResult, NotificationRulesError>` [cite: 1320, 1369]
    * `handle_setting_changed(event)` (intern aufgerufen). [cite: 1362, 1369]
* **Fehlerbehandlung:** `NotificationRulesError` (z.B. `InvalidRuleDefinition`, `ConditionEvaluationError`, `ActionApplicationError`, `SettingsAccessError`). [cite: 1371, 1372, 1373, 1374, 1375]
* **Dateistruktur:** `domain/src/notifications_rules/{mod.rs, types.rs, error.rs}`.

### 3. Interaktionen und Abhängigkeiten

* **Domänenmodule untereinander:**
    * `NotificationCoreManager` nutzt `NotificationRulesEngine` zur Verarbeitung von Benachrichtigungen. [cite: 1250, 1252]
    * `NotificationRulesEngine` nutzt `SettingsCoreManager` (oder `GlobalSettingsService`), um regelbedingte Einstellungen abzufragen. [cite: 1291, 1310, 1312]
    * `ThemingEngine` reagiert auf `SettingChangedEvent` von `SettingsCoreManager` für themenrelevante Einstellungen. [cite: 990, 1068]
    * Services aus `domain::user_centric_services` und `domain::workspaces` können globale Einstellungen von `GlobalSettingsService` lesen. [cite: 881, 883, 1069, 1070]
* **Abhängigkeiten zur Kernschicht (`core::*`):**
    * `core::config`: Wird von `domain::settings_persistence_iface`-Implementierungen und `domain::workspaces::config` für das Lesen/Schreiben von Konfigurationsdateien genutzt. [cite: 561, 638, 871, 1056]
    * `core::errors`: Basisfehlertypen können in Domänenfehler gewrappt werden. [cite: 380, 639, 870, 1018, 1058]
    * `core::types`: Fundamentale Typen wie `Uuid`, `DateTime<Utc>`. [cite: 640, 869]
    * `core::logging` (`tracing`): Wird für Logging verwendet. [cite: 641, 873, 1059]
* **Schnittstellen zu höheren Schichten (System- und UI-Schicht):**
    * Die Domänenschicht stellt ihre Funktionalität über öffentliche APIs (oft Traits) ihrer Service-Komponenten bereit. [cite: 4, 332, 708, 874, 1105]
    * Die UI-Schicht (z.B. `ui::control_center`[cite: 905, 989, 1060], `ui::shell` [cite: 655]) konsumiert diese APIs und reagiert auf Events aus der Domänenschicht.
    * Die Systemschicht (z.B. MCP-Client[cite: 707, 875], D-Bus Handler[cite: 707, 876], Compositor [cite: 510, 650, 991]) interagiert ebenfalls mit den Domänendiensten und leitet Systemereignisse an diese weiter oder setzt deren Anweisungen um.

### 4. Zusammenfassende Betrachtungen

Die Domänenschicht ist als eine Sammlung modularer, voneinander entkoppelter Komponenten konzipiert, die jeweils klar definierte Verantwortlichkeiten besitzen. Durch die konsequente Anwendung von Prinzipien wie Typsicherheit, expliziter Fehlerbehandlung, Event-basierter Kommunikation und der Abstraktion von Persistenz- und UI-Belangen wird eine robuste, wartbare und erweiterbare Grundlage für die Desktop-Umgebung geschaffen. [cite: 1084, 1085, 1088, 1090, 1091, 1093, 1408] Die detaillierten Spezifikationen der einzelnen Module, ihrer Datenstrukturen, APIs und Fehlerfälle dienen als direkter Leitfaden für die Implementierung.
## 3. Systemschicht
**Technische Gesamtspezifikation und Entwicklungsrichtlinien (Systemschicht-Fokus)**

**I. Einleitung**

Dieses Dokument ist die umfassende technische Spezifikation und Richtliniensammlung für die Entwicklung einer neuartigen Linux-Desktop-Umgebung. Ziel ist eine moderne, schnelle, intuitive und KI-gestützte Benutzererfahrung, optimiert für Entwickler, Kreative und alltägliche Nutzer. Diese Spezifikation legt die technische Grundlage und klare Entwicklungsrichtlinien fest, definiert die Architektur, den Technologie-Stack, Kernkomponenten und Entwicklungsprinzipien. Sie dient als Basis für detaillierte Implementierungsleitfäden der Architekturschichten und ermöglicht Entwicklern die direkte Implementierung ohne grundlegende Technologie- oder Architekturentscheidungen. Die Desktop-Umgebung basiert auf einer geschichteten Architektur für Modularität, Wartbarkeit und Testbarkeit.

**II. Architektonischer Überblick (Schichtenarchitektur)**

Das System ist in vier logische Schichten unterteilt, jede mit spezifischen Verantwortlichkeiten und definierten Schnittstellen, was Kohäsion fördert und Kopplung reduziert.

* **Kernschicht (Core Layer):**
    * **Verantwortlichkeiten:** Grundlegendste Datentypen, Dienstprogramme, Konfigurationsgrundlagen, Logging-Infrastruktur, allgemeine Fehlerdefinitionen. Keine Abhängigkeiten zu anderen Schichten.
    * **Interaktionen:** Stellt Funktionalität für alle darüberliegenden Schichten bereit.
* **Domänenschicht (Domain Layer):**
    * **Verantwortlichkeiten:** Kernlogik und Geschäftsregeln: Workspace-Verwaltung ("Spaces"), Theming-System, Logik für KI-Interaktionen (inkl. Einwilligungsmanagement), Benachrichtigungsverwaltung, Richtlinien für Fenstermanagement (z.B. Tiling-Regeln). Unabhängig von UI-Implementierungen oder Systemdetails (D-Bus, Wayland).
    * **Interaktionen:** Nutzt Kernschicht. Stellt Logik und Zustand für System- und Benutzeroberflächenschicht bereit.
* **Systemschicht (System Layer):**
    * **Verantwortlichkeiten:** Interaktion mit Betriebssystem und externen Diensten: Wayland-Compositor, Eingabeverarbeitung (libinput), D-Bus-Kommunikation (Netzwerk, Energie, Audio, Secrets, PolicyKit), Implementierung von Wayland-Protokollen (inkl. `xdg-shell`, `wlr-layer-shell-unstable-v1`, `xdg-decoration-unstable-v1`, `wlr-foreign-toplevel-management-unstable-v1`, `wlr-output-management-unstable-v1`, `wlr-output-power-management-unstable-v1`), XWayland-Integration, MCP-Client-Implementierung, Interaktion mit XDG Desktop Portals. Implementiert die "Mechanik" des Fenstermanagements (Positionierung, Tiling-Anwendung, Fokus, Sichtbarkeit, Fensterdekorationen) und des Workspace-Managements (Erstellung, Zuweisung von Fenstern, Wechsel). Stellt die technische Basis für die "Intelligente Tab-Leiste" bereit, indem sie Anwendungsfenster und deren Metadaten verwaltet und Gruppierungslogik für "Spaces" ermöglicht. Handhabt Bildschirmfreigabe über XDG Desktop Portals. Verwaltet Monitorkonfigurationen (Auflösung, Skalierung, Anordnung) und DPMS. Stellt Schnittstellen für Audio-Management (PipeWire) und die Verwaltung von Anwendungs-Streams bereit.
    * **Interaktionen:** Nutzt Kern- und Domänenschicht (Anwendung von Regeln, Zustandsabfragen). Stellt systemnahe Dienste und Ereignisse für Benutzeroberflächenschicht bereit. Empfängt Befehle von der UI-Schicht (z.B. Fenster verschieben, Space wechseln) und setzt diese um. Sendet Systemereignisse (z.B. neues Fenster, Fokusänderung, Output-Änderung) an die UI-Schicht.
* **Benutzeroberflächenschicht (User Interface Layer):**
    * **Verantwortlichkeiten:** Darstellung der Benutzeroberfläche und Benutzerinteraktion: Shell-UI (Panels, Dock, "Intelligente Tab-Leiste" pro Space, Workspace-Switcher, Quick-Settings), Control Center, Widget-System (adaptive Seitenleisten), Übersichtsmodus, kontextuelle Befehlspalette, Speed-Dial. Nutzt GTK4.
    * **Interaktionen:** Nutzt alle darunterliegenden Schichten, insbesondere Systemschicht (Fensterverwaltung, Eingabeempfang, Systemdienstansprache) und Domänenschicht (Zustandsdarstellung, Geschäftslogikauslösung). Sendet Benutzerbefehle an die Systemschicht und reagiert auf deren Ereignisse.

Diese Schichtung minimiert Auswirkungen von Änderungen in einer Schicht auf andere, besonders auf Kern- und Domänenschicht.

**III. Technologie-Stack (Systemschicht-Fokus)**

Die Auswahl basiert auf Modernität, Leistung, Sicherheit, Wartbarkeit und Verfügbarkeit im Linux-Ökosystem.

| Bereich                      | Technologie/Standard                                                                                                   | Begründung                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| :--------------------------- | :--------------------------------------------------------------------------------------------------------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Programmiersprache           | Rust                                                                                                                   | Leistung, Speichersicherheit ohne GC, starkes Typsystem, Ownership-Modell, moderne Nebenläufigkeitskonzepte. Zukunftsicher für Systemsoftware.                                                                                                                                                                                                                                                                                                                              |
| Build-System                 | Meson                                                                                                                  | Modern, einfach, schnell, gute Rust/C++ Integration, Abhängigkeitsmanagement über WrapDB/Subprojekte.                                                                                                                                                                                                                                                                                                                                                               |
| GUI-Toolkit                  | GTK4                                                                                                                   | Modern, aktiv entwickelt, erstklassige Wayland-Unterstützung, Rust-Bindings (gtk4-rs), CSS-Theming, dynamischer Theme-Wechsel zur Laufzeit.                                                                                                                                                                                                                                                                                                                           |
| **Wayland Compositor & Bib.** | **Smithay Toolkit** | In Rust geschrieben, modular, selektive Nutzung von Komponenten, Unterstützung für Wayland-Protokolle, XWayland-Integration, Abstraktionen für Backends (DRM, libinput). Native Rust-Implementierung vereinfacht Integration.                                                                                                                                                                                                                                  |
| **Essentielle Wayland-Prot.** | `wayland.xml`, `xdg-shell`, `wlr-layer-shell-unstable-v1`, `xdg-decoration-unstable-v1`, `wlr-foreign-toplevel-management-unstable-v1`, `wlr-output-management-unstable-v1`, `wlr-output-power-management-unstable-v1`, `input-method-unstable-v1`, `text-input-unstable-v3`, `presentation-time`, `viewporter`, `linux-dmabuf-unstable-v1`, `idle-notify-unstable-v1`. | Basisprotokoll. Standard für Desktop-Fenster. Für Panels, Docks, Benachrichtigungen. Fensterdekorationen. Auflisten/Steuern von Fenstern. Monitorkonfiguration. Energiesparmodus von Monitoren. Eingabemethoden. Zwischenablage/DND (Wayland Core, ggf. `wlr-data-control-unstable-v1`). Screencasting/Screenshots (XDG Portals oder `wlr-screencopy-unstable-v1`). X11-Kompatibilität (XWayland). |
| **Inter-Prozess-Komm. (IPC)** | **D-Bus** | De-facto-Standard im Linux-Desktop. Integration mit Systemdiensten (NetworkManager, UPower, logind, PolicyKit, org.freedesktop.Notifications, org.freedesktop.secrets). Etablierte Rust-Bibliotheken (z.B. zbus).                                                                                                                                                                                                                                |
| **KI-Integration** | **Model Context Protocol (MCP)** | Offener Standard für sichere, standardisierte LLM-Anbindung. Client-Server-Architektur, definierte Nachrichtenformate. Ermöglicht lokale/Cloud-Modelle, Benutzerkontrolle über Datenzugriff.                                                                                                                                                                                                                                                                     |
| **Eingabeverarbeitung** | **libinput** (via Smithay)                                                                                             | Standardbibliothek für Eingabeereignisse (Tastatur, Maus, Touchpad). Integration in Wayland-Compositors. Robuste Gestenunterstützung (Pinch, Swipe). Konsistente, präzise Eingabebehandlung.                                                                                                                                                                                                                                                                           |
| **Audio-Management** | **PipeWire** | Moderner Standard für Audio/Video. Geringe Latenz, flexibles Routing, sandboxed. Kompatibilitätsschichten für PulseAudio/JACK/ALSA. Rust-Bibliotheken (pipewire-rs). Steuerung von Lautstärke, Geräteauswahl, Anwendungs-Streams.                                                                                                                                                                                                                                |
| **Geheimnisverwaltung** | **Freedesktop Secret Service API** | Standard zum sicheren Speichern sensibler Daten (Passwörter, API-Schlüssel). Implementierungen (GNOME Keyring, KWallet) via D-Bus. Schützt Daten vor Klartextspeicherung. Rust-Bibliotheken (secret-service-rs).                                                                                                                                                                                                                                                  |
| **Rechteverwaltung** | **PolicyKit (polkit)** | Standard für privilegierte Aktionen (Systemupdates, Energieeinstellungen). Interaktion über D-Bus. Stellt sicher, dass administrative Aufgaben nur mit Benutzerzustimmung erfolgen.                                                                                                                                                                                                                                                                                           |
| Theming-Implementierung      | Token-basiert via GTK4 CSS Custom Properties (`var()`) und `@define-color`.                                            | Abstraktion über konkreten Werten. Designentscheidungen als benannte Tokens. Laufzeitänderungen für dynamische Theme-Umschaltung ohne Neustart. Organisation in Schichten (Foundation -> Alias -> Component). Generierte CSS-Dateien via `GtkCssProvider`.                                                                                                                                                                                                    |
| **Sandboxing-Interaktion** | **XDG Desktop Portals** | Standardisierte D-Bus-Schnittstellen für sandboxed Anwendungen für sicheren Ressourcenzugriff (Dateidialoge, Kamera, Mikrofon, Screencasting). Empfohlener Weg unter Wayland. Rust-Bibliotheken (xdg-portal). Backend-Implementierungen ggf. durch Desktop-Umgebung.                                                                                                                                                                                      |

**IV. Entwicklungsrichtlinien (Auszug)**

* **Coding Style & Formatierung:** Standard `rustfmt` Konfiguration, Rust API Guidelines. CI-Prüfung.
* **API-Design:** Befolgung Rust API Guidelines Checklist (Traits, Fehler, Generics, Newtypes, Builder).
* **Fehlerbehandlung:** `thiserror` Crate für spezifische Fehler-Enums pro Modul. Panics vermeiden.
* **Logging & Tracing:** `tracing` Crate-Framework für strukturiertes, kontextbezogenes Logging mit Spans.
* **Versionskontrolle & Branching:** Git mit GitHub Flow. `main`-Branch stabil. Feature-Branches, PRs obligatorisch.
* **Teststrategie:** Umfassende Unit-Tests (Kern-, Domänenschicht). Integrationstests (Modulzusammenspiel, externe Schnittstellen). Compositor-Tests (Evaluierung von Headless Backends, Test-Clients). UI-Tests (Strategie TBD, Fokus auf untere Schichten). CI-Pipeline für alle Tests.
* **Dokumentation:** Umfassende `rustdoc`-Kommentare für öffentliche APIs. Architektur-Dokus. READMEs. Cargo.toml Metadaten.

**V. Initiale Schichtspezifikationen (Systemschicht-Komponenten – Detailliert)**

* **`system::compositor`**: Smithay-basierter Wayland-Compositor.
    * **`core`**: `DesktopState` (zentraler Zustand, implementiert Smithay Handler wie `CompositorHandler`, `XdgShellHandler`), `SurfaceData` (pro `WlSurface`, speichert Puffer, Rolle, Schaden, Geometrie, Hooks), Globalerstellung (`wl_compositor`, `wl_subcompositor`).
        * *Schnittstellen*: Nimmt Konfigurationsdaten (Fensterrollen, Tiling-Regeln) von der Domänenschicht entgegen. Stellt `WlSurface`-Informationen und Fensterstruktur für UI-Schicht bereit.
    * **`shm`**: `ShmState`, `ShmHandler` für `wl_shm` Puffer.
    * **`xdg_shell`**: `XdgShellState`, `XdgShellHandler`-Implementierungen für `DesktopState`. Verwaltung von `ManagedToplevel` (Titel, AppID, Zustand, Geometrie, Dekorationen) und `ManagedPopup`. Sendet `configure`-Events.
        * *Schnittstellen*: Interagiert mit `domain::window_management` für Platzierungsrichtlinien. Meldet Fensterzustände (Titel, AppID, Geometrie) an `ui::shell` und `ui::window_manager_frontend`.
    * **`display_loop`**: Integration der Wayland-Anzeige (`DisplayHandle`) in `calloop`-Ereignisschleife. `ClientData` für Wayland-Clients.
    * **`renderer_interface`**: Abstrakte Traits `FrameRenderer` und `RenderableTexture` zur Entkopplung von Rendering-Backends. `RenderElement` Enum (Surface, SolidColor, Cursor).
* **`system::input`**: Libinput-basierte Eingabeverarbeitung.
    * **`seat_manager`**: `SeatState`, `SeatHandler` (Fokusmanagement `KeyboardFocus`, `PointerFocus`, `TouchFocus`; `cursor_image`-Logik). `XkbKeyboardData` (Keymap, xkbcommon State, Tastenwiederholung).
        * *Schnittstellen*: Sendet Fokusänderungs-Events und Cursor-Informationen an `ui::shell`. Empfängt Befehle zur Fokusänderung von der UI.
    * **`libinput_handler`**: `LibinputInputBackend`-Initialisierung. `process_input_event` leitet libinput-Events (Keyboard, Pointer, Touch, Gesture) an Übersetzer weiter.
    * **`keyboard`**: `key_event_translator` (Keycode zu Keysym/UTF-8, Modifikatoren, Tastenwiederholung). `focus_handler_keyboard` (sendet `enter`/`leave` an Clients).
    * **`pointer`**: `pointer_event_translator` (Motion, Button, Axis). `focus_handler_pointer` (Enter/Leave, Fokus-Logik basierend auf globaler Cursorposition und Fenstern).
    * **`touch`**: `touch_event_translator` (Down, Up, Motion, Frame, Cancel). `focus_handler_touch`.
* **`system::dbus`**: Schnittstellen zu D-Bus-Diensten via `zbus`.
    * **`connection`**: `DBusConnectionManager` (Session/System-Bus).
    * **`upower_client`**: `UPowerClient`, `UPowerManagerProxy`, `UPowerDeviceProxy`. Typen: `PowerDeviceDetails`, `UPowerProperties`, `PowerDeviceType`, `PowerState`. Signale: `DeviceAdded`, `DeviceRemoved`, `PropertiesChanged`.
        * *Schnittstellen*: Sendet `UPowerEvent` (Batteriestatus, Deckelzustand) an Domänen- und UI-Schicht.
    * **`logind_client`**: `LogindClient`, `LogindManagerProxy`, `LogindSessionProxy`. Typen: `SessionInfo`. Signale: `SessionNew`, `SessionRemoved`, `PrepareForSleep`, `Lock`/`Unlock` auf Session.
        * *Schnittstellen*: Sendet `LogindEvent` (Suspend-Vorbereitung, Sitzungssperre) an Domänen- und UI-Schicht. Kann `LockSession` von UI empfangen.
    * **`networkmanager_client`**: `NetworkManagerClient`, `NetworkManagerProxy`, `NMDeviceProxy`, `NMActiveConnectionProxy`. Typen: `NetworkManagerState`, `NetworkDevice`, `ActiveConnection`. Signale: `StateChanged`, `DeviceAdded/Removed`.
        * *Schnittstellen*: Sendet Netzwerkstatus-Events an Domänen- und UI-Schicht.
    * **`secrets_client`**: `SecretsClient`, `SecretServiceProxy`, `SecretCollectionProxy`, `SecretItemProxy`, `SecretPromptProxy`. Typen: `Secret`, `SecretItemInfo`. Methoden: `StoreSecret`, `RetrieveSecret`, `SearchItems`. Handhabt Prompts.
        * *Schnittstellen*: Speichert/ruft sensible Daten (z.B. MCP API Keys) für `system::mcp` oder Domänenschicht ab. Interagiert mit UI für Prompts.
    * **`policykit_client`**: `PolicyKitClient`, `PolicyKitAuthorityProxy`. Typen: `PolicyKitCheckAuthFlags`, `PolicyKitSubject`, `PolicyKitAuthorizationResult`. Methode: `CheckAuthorization`.
        * *Schnittstellen*: Wird von anderen Systemschicht-Modulen oder der Domänenschicht für Rechteprüfungen genutzt.
* **`system::outputs`**: Verwaltung von Anzeigeausgängen.
    * **`output_device`**: `OutputDevice` (kapselt `smithay::output::Output`, Name, Globals, DPMS-Zustand). `OutputDevicePendingState` für wlr-output-management.
    * **`manager`**: `OutputManager` (Liste von `OutputDevice`), Hotplug-Event-Handling (`DeviceAdded`, `DeviceRemoved`). Erstellt/zerstört Wayland-Globals für Outputs.
        * *Schnittstellen*: Meldet Output-Änderungen an `ui::shell` und `ui::control_center`. Empfängt Konfigurationsbefehle.
    * **`wl_output_handler`**: Integration mit `smithay::wayland::output::OutputHandler`.
    * **`wlr_output_management_handler`**: Serverseitige Implementierung von `wlr-output-management-unstable-v1`. `WlrOutputManagementState`, `OutputConfigurationRequest`. Handhabt `create_configuration`, `apply`, `test`. Serial-Management für Konsistenz.
    * **`wlr_output_power_management_handler`**: Serverseitige Implementierung von `wlr-output-power-management-unstable-v1`. `WlrOutputPowerManagementState`. Handhabt `get_output_power`, `set_mode` (On/Off).
    * **`xdg_output_handler`**: Serverseitige Implementierung von `xdg-output-unstable-v1`. Stellt logische Geometrie bereit.
* **`system::xwayland`**: Logik zur Integration und Verwaltung des XWayland-Servers (via Smithay). Stellt Kompatibilität für X11-Anwendungen sicher.
    * *Schnittstellen*: Macht X11-Fenster für den Compositor und die UI-Schicht sichtbar und handhabbar.
* **`system::audio`**: PipeWire Client-Integration via `pipewire-rs`.
    * **`client`**: `PipeWireClient` (verwaltet Core, MainLoop-Thread, Befehls-/Ereigniskanäle). `PipeWireLoopData` (interner Zustand im Loop-Thread).
    * **`manager`**: Verarbeitet Registry-Events, verwaltet `AudioDevice`- und `StreamInfo`-Objekte. Handhabt Parameteränderungen.
    * **`control`**: Implementiert Lautstärke-/Stummschaltungsbefehle, Standardgeräteauswahl.
    * **`types`**: `AudioDevice` (ID, Name, Typ, Lautstärke, Mute, Default), `StreamInfo` (ID, Name, App, PID, Lautstärke, Mute), `AudioCommand`, `AudioEvent`.
    * **`spa_pod_utils`**: Hilfsfunktionen für SPA POD-Erstellung (Lautstärke, Mute für Props/Route).
        * *Schnittstellen*: Sendet `AudioEvent` (Geräte-/Stream-Änderungen, Lautstärke, Mute, Standardgerät) an Domänen- und UI-Schicht. Empfängt `AudioCommand` von UI (z.B. Lautstärke ändern).
* **`system::mcp`**: Model Context Protocol Client via `mcp_client_rs`.
    * **`client`**: `McpClient` (verwaltet Serverprozess, Client-Handle, Anfrage-Handling, Benachrichtigungsverteilung). `McpServerConfig`.
    * **`types`**: Wrapper/Re-Exporte von `mcp_client_rs::protocol` Typen (`InitializeParams`, `ListResourcesParams`, `CallToolParams`, `Resource`, `Tool`). `McpNotification`.
        * *Schnittstellen*: Stellt KI-Funktionen (Ressourcenauflistung, Tool-Aufrufe) für Domänen- und UI-Schicht bereit. Empfängt Anfragen von UI (z.B. Befehlspalette).
* **`system::portals`**: Backend-Implementierung für XDG Desktop Portals via `zbus`.
    * **`file_chooser`**: `FileChooserPortal` (implementiert `org.freedesktop.portal.FileChooser`: `OpenFile`, `SaveFile`, `SaveFiles`).
    * **`screenshot`**: `ScreenshotPortal` (implementiert `org.freedesktop.portal.Screenshot`: `Screenshot`, `PickColor`).
    * **`common`**: `DesktopPortal`-Struktur (aggregiert Portal-Implementierungen), `run_portal_service` (startet D-Bus-Dienst). Kommunikationsstrukturen (`UiPortalCommand`, `CompositorScreenshotCommand`).
        * *Schnittstellen*: Interagiert mit `ui::shell`/`ui::components` zur Anzeige von Dialogen. Kommuniziert mit `system::compositor` für Screenshot-/Farbpick-Aktionen. Dient Anfragen von sandboxed und nativen Anwendungen.

**VI. Deployment-Überlegungen**

* **Paketierung:** Zielformate (.deb/.rpm, Flatpak für Teile/SDK). Build-Prozess spezifizieren. Tiefere Systemintegration (Display-Manager, systemd User-Sessions, PAM) als reine Agenteninstallation.
* **Konfiguration:** Standardkonfigurationen, Benutzerüberschreibungen. Strikte Einhaltung XDG Base Directory Specification (`$XDG_CONFIG_HOME`, `$XDG_DATA_HOME`, `$XDG_STATE_HOME`).
* **Updates:** Strategie für Updates (Distro-Paketmanager, Flatpak). Versionierung, Konfigurationsänderungen bei Updates.

**VII. Schlussfolgerung**

Diese technische Gesamtspezifikation mit Systemschicht-Fokus legt das Fundament für die Entwicklung. Sie definiert eine klare Architektur, wählt einen modernen Technologie-Stack (Rust, Wayland, GTK4, Smithay, PipeWire, MCP) und etabliert Entwicklungsrichtlinien. Dies bildet die Basis für detaillierte Implementierungsleitfäden. Ziel ist eine hochwertige, moderne, sichere und anpassungsfähige Desktop-Umgebung.

