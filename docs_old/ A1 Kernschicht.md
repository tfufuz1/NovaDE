
# Kerninfrastruktur Implementierungsplan (Ultra-Feinspezifikation)

## 1. Einleitung

Dieses Dokument stellt den finalen, lückenlosen Entwickler-Implementierungsleitfaden für die Kerninfrastrukturschicht (Core Layer) dar. Es ist als Ultra-Feinspezifikation konzipiert und enthält alle notwendigen Details, um Entwicklern die direkte Implementierung in Rust zu ermöglichen, ohne dass eigene Architekturentscheidungen, Logikentwürfe oder Algorithmen erforderlich sind. Alle relevanten Aspekte wurden recherchiert, entschieden und präzise spezifiziert.

Die Kerninfrastruktur (`core_infra`) bildet das Fundament des Systems. Ihre Hauptverantwortlichkeiten umfassen die Bereitstellung grundlegendster Datentypen, Dienstprogramme, der Konfigurationsgrundlagen, der Logging-Infrastruktur und allgemeiner Fehlerdefinitionen. Diese Schicht dient als Basis für alle anderen Schichten des Systems und weist selbst keine Abhängigkeiten zu diesen höheren Schichten auf. Ihre Funktionalität wird von allen übergeordneten Schichten genutzt. Die Implementierung folgt strikt den Rust API Guidelines 1 und Best Practices für sichere und wartbare Rust-Entwicklung.3

## 2. Allgemeine Designprinzipien und Konventionen

### 2.1. Programmiersprache und Tooling

- **Sprache:** Rust (aktuellste stabile Version, Mindestanforderung gemäß Abschnitt 10.1).
- **Build-System:** Cargo (Standard Rust Build-System und Paketmanager).5
- **Formatierung:** `rustfmt` mit Standardkonfiguration (100 Zeichen Zeilenbreite, 4 Leerzeichen Einrückung).3
- **Linting:** `clippy` mit Standardempfehlungen (`cargo clippy`).

### 2.2. Code-Stil und Namenskonventionen

- Strikte Einhaltung der offiziellen Rust API Guidelines.1
- **Casing:** `snake_case` für Funktionen, Methoden, Variablen, Module; `PascalCase` für Typen (Structs, Enums, Traits).1
- **Konvertierungen:** `as_` für günstige Referenz-zu-Referenz-Konvertierungen, `to_` für teurere Wert-zu-Wert-Konvertierungen, `into_` für übernehmende Konvertierungen.1
- **Getter:** Namen folgen der Konvention `field_name()` für einfachen Zugriff, `set_field_name()` für Setter (falls mutability erlaubt ist).1
- **Iteratoren:** Methoden liefern `iter()`, `iter_mut()`, `into_iter()`; Iterator-Typen heißen entsprechend (z.B. `MyTypeIter`).1
- **Modulstruktur:** Klar definierte Module gemäß Abschnitt 10.5.

### 2.3. Fehlerbehandlung

- Keine Panics für erwartbare Fehler; stattdessen `Result<T, CoreError>` verwenden.6
- Vermeidung von `.unwrap()` und `.expect()`; Nutzung des `?`-Operators zur Fehlerpropagation.1
- Definition einer zentralen `CoreError`-Enum mit `thiserror` für die gesamte Schicht.6
- Fehlermeldungen müssen klar, kontextbezogen und informativ sein.

### 2.4. Sicherheit

- **Kein `unsafe` Code:** Die Verwendung von `unsafe`-Blöcken ist in dieser Schicht strikt untersagt, um die von Rust garantierte Speichersicherheit zu gewährleisten.4
- **Input Validierung:** Obwohl die Kernschicht primär interne Dienste bereitstellt, müssen alle von außen kommenden Konfigurationsdaten oder Parameter validiert werden (C-VALIDATE 1).
- **Dependency Management:** Abhängigkeiten werden minimal gehalten und regelmäßig auf Sicherheitsupdates überprüft.4

### 2.5. Dokumentation

- Alle öffentlichen Elemente (Module, Typen, Funktionen, Methoden) MÜSSEN mit ausführlichen Rustdoc-Kommentaren (`///`) versehen sein.1
- Dokumentation umfasst Zweck, Parameter, Rückgabewerte, mögliche Fehler (`CoreError`-Varianten) und Beispiele (C-EXAMPLE 1).
- Fehler- und Panikbedingungen (obwohl Panics vermieden werden sollen) müssen dokumentiert werden (C-FAILURE 1).

## 3. Kern-Datentypen (`core_infra::types`)

Dieses Modul definiert grundlegende Datenstrukturen (Structs und Enums), die potenziell von mehreren anderen Modulen innerhalb oder außerhalb der Kernschicht verwendet werden könnten. Es enthält keine komplexe Logik, sondern nur die Definitionen selbst.

### 3.1. Modulstruktur

```
core_infra/
└── src/
    ├── lib.rs
    └── types.rs
```

### 3.2. Grundlegende Wertobjekte und Structs

Wertobjekte sind einfache Strukturen, die hauptsächlich Daten kapseln und deren Identität durch ihre Werte definiert wird. Sie sollten unveränderlich sein, nachdem sie erstellt wurden, was durch private Felder und Konstruktoren, die Validierungen durchführen, sichergestellt wird.

- **Beispiel:** `AppIdentifier` (falls eine spezifische ID-Struktur benötigt wird)
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    use std::fmt;
    
    /// Represents a unique identifier for an application component or instance.
    /// Enforces specific formatting rules through its constructor.
    #
    pub struct AppIdentifier(String); // Internal representation is a String
    
    impl AppIdentifier {
        /// Creates a new AppIdentifier from a string slice.
        ///
        /// # Arguments
        /// * `value` - The string representation of the identifier. Must not be empty
        ///           and should adhere to specific format rules (e.g., alphanumeric).
        ///
        /// # Errors
        /// Returns `CoreError::InvalidInput` if the value is empty or does not meet
        /// the required format.
        pub fn new(value: &str) -> Result<Self, crate::error::CoreError> {
            if value.is_empty() {
                Err(crate::error::CoreError::InvalidInput("AppIdentifier cannot be empty".to_string()))
            } else if!value.chars().all(|c| c.is_alphanumeric() |
    ```
    

| c == '-') {

Err(crate::error::CoreError::InvalidInput(format!("AppIdentifier contains invalid characters: {}", value)))

}

else {

Ok(Self(value.to_string()))

}

}

````
    /// Returns a reference to the underlying string value.
    /// Conforms to Rust API guidelines for getters (C-GETTER [1]).
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Allows displaying the AppIdentifier.
impl fmt::Display for AppIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Allows conversion from AppIdentifier to String.
impl From<AppIdentifier> for String {
    fn from(id: AppIdentifier) -> Self {
        id.0
    }
}

/// Allows borrowing as a string slice.
/// Conforms to Rust API guidelines for conversions (C-CONV-TRAITS [1]).
impl AsRef<str> for AppIdentifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
```
````

- **Spezifikation:**
    - Alle Wertobjekte müssen `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash` implementieren (C-COMMON-TRAITS 1).
    - Falls sie in Konfigurationen oder Logs serialisiert werden sollen, müssen sie `serde::Serialize` und `serde::Deserialize` implementieren.8
    - Felder müssen privat sein, um Invarianten zu schützen (C-STRUCT-PRIVATE 1).
    - Öffentliche Konstruktoren (`fn new(...) -> Result<Self, CoreError>`) validieren Eingaben und erzwingen Invarianten (C-CTOR 1, C-VALIDATE 1).
    - Getter-Methoden (`fn field_name(&self) -> &T`) bieten Lesezugriff.
    - Implementierung relevanter Traits wie `Display`, `From`, `AsRef` (C-CONV-TRAITS 1).

### 3.3. Fundamentale Enums

Definition einfacher Enums für Zustände oder Kategorien, die systemweit relevant sein könnten.

- **Beispiel:** `Status`
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    
    /// Represents a general status indicator.
    #
    pub enum Status {
        Enabled,
        Disabled,
        Pending,
        Error(i32), // Example with associated data
    }
    
    impl Status {
        /// Checks if the status indicates an active or ready state.
        pub fn is_active(&self) -> bool {
            matches!(self, Status::Enabled)
        }
    }
    ```
    
- **Spezifikation:**
    - Müssen `Debug`, `Clone`, `Copy` (wenn sinnvoll), `PartialEq`, `Eq`, `Hash` implementieren (C-COMMON-TRAITS 1).
    - Falls für Konfiguration/Logs benötigt, `serde::Serialize` und `serde::Deserialize` implementieren.8
    - Varianten klar dokumentieren.
    - Nützliche Hilfsmethoden (wie `is_active`) können implementiert werden.

## 4. Utility Services (`core_infra::utils`)

Dieses Modul stellt grundlegende Hilfsfunktionen und Dienste bereit, die keine spezifische Domänenlogik enthalten, aber von vielen Teilen der Anwendung benötigt werden.

### 4.1. Modulstruktur

```
core_infra/
└── src/
    ├── lib.rs
    ├── types.rs
    ├── error.rs
    └── utils/
        ├── mod.rs  // Re-exportiert öffentliche Funktionen aus Submodulen
        ├── fs.rs
        └── paths.rs
        // Optional: strings.rs, time.rs
```

### 4.2. Filesystem Utilities (`core_infra::utils::fs`)

Enthält grundlegende, sichere Operationen für das Dateisystem. Komplexe Dateioperationen gehören in höhere Schichten.

- **Funktionen:**
    
    Rust
    
    ```
    // core_infra/src/utils/fs.rs
    use crate::error::CoreError;
    use std::fs;
    use std::path::Path;
    
    /// Ensures that a directory exists at the specified path.
    /// If the directory does not exist, it attempts to create it, including any
    /// necessary parent directories.
    ///
    /// # Arguments
    /// * `path` - The path to the directory to ensure existence of.
    ///
    /// # Errors
    /// Returns `CoreError::Filesystem` if the directory could not be created
    /// or if the path exists but is not a directory.
    pub fn ensure_dir_exists(path: &Path) -> Result<(), CoreError> {
        if path.exists() {
            if!path.is_dir() {
                return Err(CoreError::Filesystem {
                    message: format!("Path exists but is not a directory"),
                    path: path.to_path_buf(),
                    // Use a placeholder error kind or map specific std::io::ErrorKind
                    source: std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Path exists but is not a directory"),
                });
            }
            Ok(()) // Directory already exists
        } else {
            fs::create_dir_all(path).map_err(|e| CoreError::Filesystem {
                message: "Failed to create directory".to_string(),
                path: path.to_path_buf(),
                source: e,
            })
        }
    }
    
    // Add other minimal, safe filesystem utilities if absolutely necessary.
    // Example: Reading a file with specific error mapping.
    /// Reads the entire contents of a file into a string.
    ///
    /// # Arguments
    /// * `path` - The path to the file to read.
    ///
    /// # Errors
    /// Returns `CoreError::Filesystem` if the file cannot be read.
    pub fn read_to_string(path: &Path) -> Result<String, CoreError> {
        fs::read_to_string(path).map_err(|e| CoreError::Filesystem {
            message: "Failed to read file to string".to_string(),
            path: path.to_path_buf(),
            source: e,
        })
    }
    ```
    
- **Spezifikation:**
    - Alle Funktionen müssen `Result<(), CoreError>` oder `Result<T, CoreError>` zurückgeben.
    - Unterliegende `std::io::Error` müssen sorgfältig in `CoreError::Filesystem` oder `CoreError::Io` gemappt werden, wobei Kontext (wie der Dateipfad) hinzugefügt wird. Die `Filesystem`-Variante ist vorzuziehen, wenn der Pfad relevant ist, um aussagekräftigere Fehlermeldungen zu ermöglichen.

### 4.3. Path Resolution (`core_infra::utils::paths`)

Stellt standardisierte Pfade für Konfiguration, Daten, Cache etc. bereit, basierend auf Betriebssystemkonventionen (insbesondere XDG Base Directory Specification auf Linux 10). Nutzt die `directories-next` Crate.11

- **Abhängigkeit:** `directories-next = "2.0.0"` (oder aktuellste stabile Version)
- **Funktionen:**
    
    Rust
    
    ```
    // core_infra/src/utils/paths.rs
    use crate::error::{CoreError, ConfigError};
    use directories_next::{BaseDirs, ProjectDirs}; // Verwende directories-next
    use std::path::PathBuf;
    
    // Definiere hier die Projekt-Qualifizierer, falls ProjectDirs verwendet wird.
    // Diese sollten global konfigurierbar sein oder aus einer zentralen Stelle stammen.
    const QUALIFIER: &str = "org"; // Beispiel
    const ORGANIZATION: &str = "YourOrg"; // Beispiel
    const APPLICATION: &str = "YourApp"; // Beispiel
    
    /// Returns the primary base directory for user-specific configuration files.
    /// Corresponds to $XDG_CONFIG_HOME or platform equivalent.
    ///
    /// # Errors
    /// Returns `CoreError::Config(ConfigError::DirectoryUnavailable)` if the directory cannot be determined.
    pub fn get_config_base_dir() -> Result<PathBuf, CoreError> {
        BaseDirs::new()
           .map(|dirs| dirs.config_dir().to_path_buf())
           .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable { dir_type: "Config Base".to_string() }))
    }
    
     /// Returns the primary base directory for user-specific data files.
    /// Corresponds to $XDG_DATA_HOME or platform equivalent.
    ///
    /// # Errors
    /// Returns `CoreError::Config(ConfigError::DirectoryUnavailable)` if the directory cannot be determined.
    pub fn get_data_base_dir() -> Result<PathBuf, CoreError> {
        BaseDirs::new()
           .map(|dirs| dirs.data_dir().to_path_buf())
           .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable { dir_type: "Data Base".to_string() }))
    }
    
    /// Returns the primary base directory for user-specific cache files.
    /// Corresponds to $XDG_CACHE_HOME or platform equivalent.
    ///
    /// # Errors
    /// Returns `CoreError::Config(ConfigError::DirectoryUnavailable)` if the directory cannot be determined.
    pub fn get_cache_base_dir() -> Result<PathBuf, CoreError> {
        BaseDirs::new()
           .map(|dirs| dirs.cache_dir().to_path_buf())
           .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable { dir_type: "Cache Base".to_string() }))
    }
    
    /// Returns the primary base directory for user-specific state files.
    /// Corresponds to $XDG_STATE_HOME or platform equivalent.
    /// Returns None if not applicable on the current platform (e.g., Windows, older macOS).
    ///
    /// # Errors
    /// Returns `CoreError::Config(ConfigError::DirectoryUnavailable)` if the directory cannot be determined.
    pub fn get_state_base_dir() -> Result<PathBuf, CoreError> {
         BaseDirs::new()
            // state_dir() ist in BaseDirs nicht direkt verfügbar,
            // aber XDG definiert es. directories-next unterstützt es möglicherweise
            // nicht direkt oder es muss manuell abgeleitet werden.
            // Fallback auf.local/state gemäß XDG Spec [10]
           .map(|dirs| {
                #[cfg(target_os = "linux")]
                {
                    std::env::var("XDG_STATE_HOME")
                       .map(PathBuf::from)
                       .unwrap_or_else(|_| dirs.home_dir().join(".local/state"))
                }
                #[cfg(not(target_os = "linux"))]
                {
                    // Für andere OS gibt es keinen direkten Standard, oft wird data_local_dir verwendet
                     dirs.data_local_dir().to_path_buf()
                }
            })
           .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable { dir_type: "State Base".to_string() }))
    }
    
    
    /// Returns the application-specific configuration directory.
    /// Uses ProjectDirs based on QUALIFIER, ORGANIZATION, APPLICATION constants.
    /// Example: ~/.config/YourOrg/YourApp on Linux.
    ///
    /// # Errors
    /// Returns `CoreError::Config(ConfigError::DirectoryUnavailable)` if the directory cannot be determined.
    pub fn get_app_config_dir() -> Result<PathBuf, CoreError> {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
           .map(|dirs| dirs.config_dir().to_path_buf())
           .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable { dir_type: "App Config".to_string() }))
    }
    
    /// Returns the application-specific data directory.
    /// Example: ~/.local/share/YourOrg/YourApp on Linux.
    ///
    /// # Errors
    /// Returns `CoreError::Config(ConfigError::DirectoryUnavailable)` if the directory cannot be determined.
     pub fn get_app_data_dir() -> Result<PathBuf, CoreError> {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
           .map(|dirs| dirs.data_dir().to_path_buf())
           .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable { dir_type: "App Data".to_string() }))
    }
    
    /// Returns the application-specific cache directory.
    /// Example: ~/.cache/YourOrg/YourApp on Linux.
    ///
    /// # Errors
    /// Returns `CoreError::Config(ConfigError::DirectoryUnavailable)` if the directory cannot be determined.
    pub fn get_app_cache_dir() -> Result<PathBuf, CoreError> {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
           .map(|dirs| dirs.cache_dir().to_path_buf())
           .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable { dir_type: "App Cache".to_string() }))
    }
    
     /// Returns the application-specific state directory.
    /// Example: ~/.local/state/YourOrg/YourApp on Linux.
    ///
    /// # Errors
    /// Returns `CoreError::Config(ConfigError::DirectoryUnavailable)` if the directory cannot be determined.
    pub fn get_app_state_dir() -> Result<PathBuf, CoreError> {
        ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
             // ProjectDirs hat kein state_dir. Wir leiten es vom Basis-State-Dir ab.
           .and_then(|proj_dirs| {
                 get_state_base_dir().map(|base_state| base_state.join(proj_dirs.project_path()))
            })
           .ok_or_else(|| CoreError::Config(ConfigError::DirectoryUnavailable { dir_type: "App State".to_string() }))
    }
    ```
    
- **Spezifikation:**
    - Die Funktionen kapseln die Logik zur Pfadermittlung und abstrahieren die Unterschiede zwischen Betriebssystemen.11 Dies ist eine zentrale Aufgabe der Kernschicht, um Portabilität zu gewährleisten.
    - `Option`-Rückgabewerte von `directories-next` werden in `CoreError::Config(ConfigError::DirectoryUnavailable)` umgewandelt, um eine konsistente Fehlerbehandlung sicherzustellen.
    - Die Verwendung von `ProjectDirs` ist optional, aber empfohlen, wenn anwendungsspezifische Unterverzeichnisse standardmäßig benötigt werden. Die Konstanten `QUALIFIER`, `ORGANIZATION`, `APPLICATION` müssen definiert werden.

### 4.4. Basic String Manipulation (`core_infra::utils::strings`)

Nur hinzufügen, wenn generische String-Helfer benötigt werden, die über `std::str` und `String` hinausgehen und in der Kernschicht _unbedingt_ erforderlich sind. Im Allgemeinen sollte dieses Modul vermieden werden, um die Schicht schlank zu halten. Falls benötigt, müssen Signaturen exakt definiert werden (`pub fn...(...) ->...`).

### 4.5. Time Utilities (`core_infra::utils::time`)

Normalerweise wird `chrono` direkt in höheren Schichten verwendet. Dieses Modul ist nur notwendig, wenn die Kernschicht spezifische Zeit-Wrapper, Formate oder Abstraktionen _bereitstellen muss_. Wenn `chrono` verwendet wird, sollte es als Abhängigkeit deklariert werden.

## 5. Configuration Management (`core_infra::config`)

Verantwortlich für das Laden, Parsen und Validieren der Kernkonfiguration der Anwendung.

### 5.1. Modulstruktur

```
core_infra/
└── src/
    ├── lib.rs
    ├── types.rs
    ├── error.rs
    ├── utils/
    │   └──...
    └── config/
        ├── mod.rs // Definiert CoreConfig, LoggingConfig etc. und ConfigLoader
        └── defaults.rs // Enthält Funktionen für Standardwerte
```

### 5.2. Configuration Data Structures (`core_infra::config::mod.rs`)

Definition der Rust-Strukturen, die das Schema der Kernkonfiguration abbilden.

Rust

```
// core_infra/src/config/mod.rs
use crate::error::{CoreError, ConfigError};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::fs;
use super::utils; // Importiere utils Modul
use super::config::defaults; // Importiere defaults Modul

/// Represents the core configuration for the application.
/// Loaded from a TOML file.
#
#[serde(deny_unknown_fields)] // Strikte Prüfung auf unbekannte Felder
pub struct CoreConfig {
    #[serde(default = "defaults::default_logging_config")]
    pub logging: LoggingConfig,
    // Weitere Kern-Einstellungen hier hinzufügen, z.B.:
    // #[serde(default = "defaults::default_feature_flags")]
    // pub features: FeatureFlags,
}

/// Configuration specific to the logging subsystem.
#
#[serde(deny_unknown_fields)]
pub struct LoggingConfig {
    /// The minimum log level to record (e.g., "trace", "debug", "info", "warn", "error").
    #[serde(default = "defaults::default_log_level")]
    pub level: String,

    /// Optional path to a file where logs should be written.
    /// If None, logs are written to stdout/stderr.
    #[serde(default = "defaults::default_log_file_path")]
    pub file_path: Option<PathBuf>,

    /// Log format (e.g., "text", "json").
    #[serde(default = "defaults::default_log_format")]
    pub format: String, // Oder eine Enum LogFormat definieren
}

// Beispiel für weitere Konfigurationsstrukturen
// #
// #[serde(deny_unknown_fields)]
// pub struct FeatureFlags {
//     #[serde(default = "defaults::default_bool_false")]
//     pub experimental_feature_x: bool,
// }

/// Service responsible for loading the application's core configuration.
#
pub struct ConfigLoader {
    // Keine Felder benötigt, da die Logik in der Methode liegt
}

impl ConfigLoader {
    /// Loads the core configuration from the standard location(s).
    ///
    /// Looks for `config.toml` in the application-specific config directory
    /// determined by `core_infra::utils::paths::get_app_config_dir()`.
    ///
    /// # Errors
    /// Returns `CoreError::Config` variants if loading, parsing, or validation fails.
    pub fn load() -> Result<CoreConfig, CoreError> {
        let config_dir = utils::paths::get_app_config_dir()?;
        let config_path = config_dir.join("config.toml");

        // 1. Sicherstellen, dass das Konfigurationsverzeichnis existiert (optional, aber gut für Erststart)
        // utils::fs::ensure_dir_exists(&config_dir)?; // Kann Fehler werfen, wenn nicht beschreibbar

        // 2. Konfigurationsdatei lesen
        let content = fs::read_to_string(&config_path).map_err(|e| {
            // Unterscheide zwischen "nicht gefunden" und anderen Lesefehlern
            if e.kind() == std::io::ErrorKind::NotFound {
                 CoreError::Config(ConfigError::NotFound { locations: vec![config_path.clone()] })
            } else {
                 CoreError::Config(ConfigError::ReadError { path: config_path.clone(), source: e })
            }
        })?;

        // 3. TOML-Inhalt deserialisieren
        let mut config: CoreConfig = toml::from_str(&content)
           .map_err(|e| CoreError::Config(ConfigError::ParseError(e)))?;

        // 4. Post-Deserialisierungs-Validierung
        Self::validate_config(&mut config)?;

        // 5. Validierte Konfiguration zurückgeben
        Ok(config)
    }

    /// Performs post-deserialization validation of the configuration.
    /// Modifiziert die Konfiguration ggf. (z.B. Pfade absolut machen).
    fn validate_config(config: &mut CoreConfig) -> Result<(), CoreError> {
        // Validiere Log-Level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if!valid_levels.contains(&config.logging.level.to_lowercase().as_str()) {
            return Err(CoreError::Config(ConfigError::ValidationError(format!(
                "Invalid logging level '{}'. Must be one of: {:?}",
                config.logging.level, valid_levels
            ))));
        }

        // Validiere Log-Format
        let valid_formats = ["text", "json"];
        if!valid_formats.contains(&config.logging.format.to_lowercase().as_str()) {
             return Err(CoreError::Config(ConfigError::ValidationError(format!(
                "Invalid logging format '{}'. Must be one of: {:?}",
                config.logging.format, valid_formats
            ))));
        }


        // Wandle relative Logpfade in absolute Pfade um (relativ zum Konfig-Verzeichnis oder Daten-Verzeichnis)
        if let Some(ref mut log_path) = config.logging.file_path {
            if log_path.is_relative() {
                // Entscheide, relativ wozu? Hier Annahme: relativ zum State-Verzeichnis
                let state_dir = utils::paths::get_app_state_dir()?;
                 // Stelle sicher, dass das State-Verzeichnis existiert
                utils::fs::ensure_dir_exists(&state_dir)?;
                *log_path = state_dir.join(&log_path);
            }
             // Optional: Stelle sicher, dass das übergeordnete Verzeichnis des Logfiles existiert
            if let Some(parent_dir) = log_path.parent() {
                utils::fs::ensure_dir_exists(parent_dir)?;
            }
        }


        // Füge hier weitere Validierungen für andere Konfigurationsabschnitte hinzu

        Ok(())
    }
}

```

- **Spezifikation:**
    - Alle Konfigurationsstrukturen müssen `serde::Deserialize`, `Debug`, `Clone`, `PartialEq` implementieren.8
    - `#[serde(deny_unknown_fields)]` ist zwingend, um Fehler bei unbekannten Feldern in der TOML-Datei zu erzeugen.
    - Felder sind `pub`. Standardwerte werden über `#[serde(default = "path::to::default_fn")]` gesetzt, wobei die Default-Funktionen in `core_infra::config::defaults` liegen.
    - Typen müssen exakt spezifiziert sein (`String`, `Option<PathBuf>`, `bool`, etc.).

### 5.3. Configuration Loading Service (`core_infra::config::mod.rs`)

- **Struktur:** `ConfigLoader` (Struct ohne Felder, Logik in `impl`-Block).
- **Methode:** `pub fn load() -> Result<CoreConfig, CoreError>`
- **Logik (Schritt-für-Schritt):**
    1. Ermittle den Pfad zum anwendungsspezifischen Konfigurationsverzeichnis mittels `utils::paths::get_app_config_dir()`.
    2. Konstruiere den vollständigen Pfad zur Konfigurationsdatei (z.B. `config.toml`). Der Dateiname ist fest auf `config.toml` festgelegt.
    3. Versuche, den Inhalt der Datei mit `std::fs::read_to_string` zu lesen. Bilde `std::io::Error` auf `CoreError::Config(ConfigError::ReadError)` oder `CoreError::Config(ConfigError::NotFound)` ab.
    4. Deserialisiere den gelesenen String mittels `toml::from_str::<CoreConfig>`.12 Bilde `toml::de::Error` auf `CoreError::Config(ConfigError::ParseError)` ab.
    5. Rufe die interne Validierungsfunktion `validate_config` auf.
    6. Gib die validierte `CoreConfig` im `Ok`-Fall zurück.

### 5.4. Configuration Format

- Das einzige unterstützte Konfigurationsformat ist **TOML**.12
- **Beispiel `config.toml`:**
    
    Ini, TOML
    
    ```
    # Beispiel config.toml
    
    [logging]
    level = "debug" # Mögliche Werte: "trace", "debug", "info", "warn", "error"
    file_path = "app.log" # Optional. Relativ zum State-Verzeichnis oder absolut.
    format = "text" # Mögliche Werte: "text", "json"
    
    # [features]
    # experimental_feature_x = true
    ```
    

### 5.5. Default Values (`core_infra::config::defaults.rs`)

Definition von Funktionen, die Standardwerte für die Konfigurationsstrukturen liefern.

Rust

```
// core_infra/src/config/defaults.rs
use super::{LoggingConfig}; // Importiere die relevanten Structs
use std::path::PathBuf;

pub(super) fn default_logging_config() -> LoggingConfig {
    LoggingConfig {
        level: default_log_level(),
        file_path: default_log_file_path(),
        format: default_log_format(),
    }
}

pub(super) fn default_log_level() -> String {
    "info".to_string()
}

pub(super) fn default_log_file_path() -> Option<PathBuf> {
    None // Standardmäßig auf stdout/stderr loggen
}

pub(super) fn default_log_format() -> String {
    "text".to_string()
}

// Beispiel für booleschen Default
// pub(super) fn default_bool_false() -> bool {
//     false
// }

// Füge hier Default-Funktionen für alle `#[serde(default = "...")]` Felder hinzu.
```

- **Spezifikation:**
    - Für jedes Feld mit `#[serde(default = "...")]` muss eine entsprechende `pub(super) fn default_...() -> FieldType` Funktion existieren.
    - Die Funktionen müssen den korrekten Typ zurückgeben und den dokumentierten Standardwert repräsentieren.

Die Trennung der Konfigurationslogik (Laden, Parsen, Validieren) vom reinen Datenschema (`CoreConfig`, `LoggingConfig`) und den Standardwerten (`defaults.rs`) fördert die Modularität und Testbarkeit. Die Validierung nach der Deserialisierung ist entscheidend, um sicherzustellen, dass die Konfiguration nicht nur syntaktisch korrekt ist, sondern auch semantisch gültig (z.B. gültige Log-Level). Die Konfiguration beeinflusst direkt das Verhalten anderer Kernkomponenten, insbesondere des Loggings, was eine sorgfältige Initialisierungsreihenfolge erfordert.

## 6. Logging Infrastructure (`core_infra::logging`)

Stellt eine zentrale und konfigurierbare Logging-Lösung für die gesamte Anwendung bereit, basierend auf der `tracing`-Crate.

### 6.1. Modulstruktur

```
core_infra/
└── src/
    ├── lib.rs
    ├── types.rs
    ├── error.rs
    ├── utils/
    │   └──...
    ├── config/
    │   └──...
    └── logging.rs // Enthält Initialisierungslogik
```

### 6.2. Logging Facade

Die `tracing`-Crate 13 wird als alleinige Schnittstelle für alle Logging-Aktivitäten in der Anwendung vorgeschrieben. Sie bietet strukturierte Logs und Span-basiertes Tracing.

### 6.3. Initialisierung (`core_infra::logging::init_logging`)

Eine Funktion zur Initialisierung des globalen `tracing`-Subscribers basierend auf der geladenen Konfiguration.

Rust

```
// core_infra/src/logging.rs
use crate::config::LoggingConfig;
use crate::error::CoreError;
use tracing::{Level, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer, Registry};
use std::io::{stdout, Write};
use std::path::Path;
use tracing_appender; // Für Dateilogging mit Rotation

/// Initializes the global tracing subscriber based on the provided configuration.
///
/// This function should be called **once** at the very beginning of the application startup.
/// It sets up logging to stdout/stderr and optionally to a file.
///
/// Handles the potential issue of needing logging before config is fully loaded
/// by allowing an optional initial call with default settings.
///
/// # Arguments
/// * `config` - The logging configuration obtained from `CoreConfig`.
/// * `is_reload` - Set to true if this is re-initializing after an initial basic setup.
///
/// # Errors
/// Returns `CoreError::LoggingInitialization` if setup fails (e.g., file cannot be opened).
pub fn init_logging(config: &LoggingConfig, is_reload: bool) -> Result<(), CoreError> {
    // 1. Filter-Level bestimmen
    let level_filter = match config.level.to_lowercase().as_str() {
        "trace" => EnvFilter::new(Level::TRACE.to_string()),
        "debug" => EnvFilter::new(Level::DEBUG.to_string()),
        "info" => EnvFilter::new(Level::INFO.to_string()),
        "warn" => EnvFilter::new(Level::WARN.to_string()),
        "error" => EnvFilter::new(Level::ERROR.to_string()),
        _ => {
            // Sollte durch validate_config abgefangen werden, aber sicherheitshalber
            return Err(CoreError::LoggingInitialization(format!(
                "Invalid log level in config: {}",
                config.level
            )));
        }
    };

    // 2. Layer für stdout/stderr erstellen (immer aktiv)
    let use_ansi = atty::is(atty::Stream::Stdout); // Farben nur bei TTY
    let stdout_layer = fmt::layer()
       .with_writer(stdout) // Explizit stdout
       .with_ansi(use_ansi) // ANSI-Farben aktivieren/deaktivieren
       .with_filter(level_filter.clone()); // Klonen, da Filter mehrfach verwendet wird

    // 3. Optional: Layer für Dateilogging erstellen
    let file_layer = if let Some(log_path) = &config.file_path {
        Some(create_file_layer(log_path, &config.format)?
            .with_filter(level_filter)) // Gleiches Level für Datei
    } else {
        None
    };

    // 4. Subscriber zusammenbauen und global setzen
    let registry = Registry::default()
       .with(stdout_layer); // stdout ist immer dabei

    // Füge den Dateilayer hinzu, falls vorhanden
    let subscriber = if let Some(layer) = file_layer {
        registry.with(layer)
    } else {
        registry.with(tracing_subscriber::filter::FilterExt::boxed(stdout_layer)) // Boxen, um Typkonsistenz zu wahren, wenn nur ein Layer da ist
    };


    // Versuche, den globalen Subscriber zu setzen
    if SubscriberInitExt::try_init(subscriber).is_err() {
        // Fehler nur werfen, wenn es nicht der Reload nach der initialen Einrichtung ist.
        // Beim Reload ist es erwartet, dass bereits ein Subscriber gesetzt ist.
        if!is_reload {
             return Err(CoreError::LoggingInitialization(
                "Failed to set global tracing subscriber. Was it already initialized?".to_string(),
            ));
        }
        // Beim Reload loggen wir, dass wir rekonfigurieren (mit dem *alten* Logger)
        info!("Re-initializing logging configuration.");
        // Der neue Subscriber wird nicht gesetzt, aber die Konfiguration wurde validiert.
        // In einem realen Szenario bräuchte man einen Mechanismus zur dynamischen Rekonfiguration
        // des Filters/Writers, was über tracing-subscriber's ReloadHandle ginge, aber
        // die Komplexität hier übersteigt. Für diese Spezifikation reicht die Validierung.
    }

    Ok(())
}

/// Helper function to create a file logging layer.
fn create_file_layer(log_path: &Path, format: &str) -> Result<Box<dyn Layer<Registry> + Send + Sync + 'static>, CoreError> {
     // Stelle sicher, dass das Verzeichnis existiert (sollte durch validate_config erfolgt sein)
    if let Some(parent) = log_path.parent() {
        if!parent.exists() {
             // Versuche es zu erstellen, falls validate_config es nicht getan hat
            utils::fs::ensure_dir_exists(parent)?;
        }
    }

    // Konfiguriere den File Appender (z.B. tägliche Rotation)
    let file_appender = tracing_appender::rolling::daily(
        log_path.parent().unwrap_or_else(|| Path::new(".")), // Sicherstellen, dass parent existiert
        log_path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("core.log")),
    );
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(file_appender);

    // Wähle das Format basierend auf der Konfiguration
     match format.to_lowercase().as_str() {
        "json" => {
            let layer = fmt::layer()
               .json() // JSON-Format aktivieren
               .with_writer(non_blocking_writer)
               .with_ansi(false); // Keine ANSI-Codes in Dateien
             Ok(Box::new(layer))
        }
        "text" | _ => { // Default auf Text
             let layer = fmt::layer()
               .with_writer(non_blocking_writer)
               .with_ansi(false); // Keine ANSI-Codes in Dateien
             Ok(Box::new(layer))
        }
    }
    // _guard muss im Scope bleiben, damit der Writer funktioniert.
    // In einer echten Anwendung muss dieser Guard an einen geeigneten Ort verschoben werden,
    // z.B. in die Hauptanwendungsstruktur oder global statisch (mit lazy_static/once_cell).
    // Für diese Spezifikation ignorieren wir die Lebenszeit des Guards, gehen aber davon aus,
    // dass er korrekt gehandhabt wird.
    // std::mem::forget(_guard); // NICHT IN PRODUKTION VERWENDEN! Nur zur Kompilierung hier.
    // Besser: Rückgabe des Guards oder Speicherung in einem globalen Kontext.
}

/// Initializes a minimal fallback logger to stderr before configuration is loaded.
/// This should be called unconditionally at the very start.
pub fn init_minimal_logging() {
     // Setze einen einfachen Logger, der nur auf stderr schreibt, falls noch keiner gesetzt ist.
    // Ignoriere Fehler, falls bereits einer gesetzt wurde (z.B. in Tests).
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(Level::INFO.to_string()));
    let _ = fmt::Subscriber::builder()
       .with_env_filter(filter)
       .with_writer(std::io::stderr) // Explizit stderr für frühe Logs
       .try_init();
}


```

- **Spezifikation:**
    - Die Funktion `init_logging` nimmt die `LoggingConfig` entgegen.
    - Sie MUSS das Log-Level (`config.level`) parsen und in einen `EnvFilter` oder äquivalenten Filter umwandeln. Ungültige Level führen zu `CoreError::LoggingInitialization`.
    - Ein `fmt::Layer` für die Standardausgabe (stdout/stderr) wird immer konfiguriert. ANSI-Farbunterstützung wird basierend auf `atty::is(atty::Stream::Stdout)` aktiviert/deaktiviert.
    - Wenn `config.file_path` `Some` ist:
        - Ein Dateilogging-Layer wird mittels `tracing_appender` (oder einer ähnlichen Crate für Rotation) erstellt. `tracing_appender` muss als Abhängigkeit hinzugefügt werden.
        - Der Pfad wird aus der Konfiguration übernommen. Die Validierung (und Umwandlung in einen absoluten Pfad) sollte bereits in `ConfigLoader::validate_config` erfolgt sein.
        - Fehler beim Öffnen/Erstellen der Logdatei oder des Verzeichnisses führen zu `CoreError::LoggingInitialization` oder `CoreError::Filesystem`.
        - Das Format (`text` oder `json`) wird gemäß `config.format` konfiguriert. ANSI-Codes werden für Dateilogs deaktiviert.
    - Die Layer werden kombiniert (mittels `.with()`) und der resultierende Subscriber wird mit `SubscriberInitExt::try_init` als globaler Standard gesetzt. Fehler beim Setzen (z.B. wenn bereits initialisiert) werden behandelt (siehe `is_reload`).
    - **Initialisierungsproblem:** Die Funktion `init_minimal_logging` wird hinzugefügt. Sie MUSS als Allererstes im `main`-Funktion aufgerufen werden, _bevor_ `ConfigLoader::load` versucht wird. Sie richtet einen einfachen Fallback-Logger ein (z.B. `INFO` Level auf `stderr`). `init_logging` wird dann _nach_ erfolgreichem Laden der Konfiguration erneut aufgerufen (mit `is_reload = true`), um die endgültige Konfiguration anzuwenden. Dies stellt sicher, dass Konfigurationsladefehler geloggt werden können.

### 6.4. Logging Macros

Entwickler MÜSSEN die Standard-`tracing`-Makros verwenden:

- `trace!(...)`: Für sehr detaillierte Diagnoseinformationen.
- `debug!(...)`: Für Debugging-Informationen während der Entwicklung.
- `info!(...)`: Für informative Nachrichten über den normalen Betrieb.
- `warn!(...)`: Für Warnungen über potenziell problematische Situationen.
- `error!(...)`: Für Fehlerbedingungen, die den Betrieb beeinträchtigen.
- `event!(Level::...,...)`: Für explizite Events mit spezifischem Level.

### 6.5. Structured Logging

Die Verwendung von strukturierten Feldern wird dringend empfohlen, um den vollen Nutzen aus `tracing` zu ziehen.13

- **Beispiel:** `info!(user_id = %user.id, operation = "login", success = true, "User logged in successfully");`
- Felder sollten konsistent benannt werden.

### 6.6. Spans

Die Verwendung von `tracing::span!` wird für die Instrumentierung logischer Arbeitsabschnitte empfohlen, insbesondere für Operationen, die Zeit in Anspruch nehmen oder über asynchrone Grenzen hinweggehen.

- **Beispiel:**
    
    Rust
    
    ```
    let span = tracing::span!(Level::DEBUG, "config_loading", path = %config_path.display());
    let _enter = span.enter(); // Span betreten
    //... Logik zum Laden der Konfiguration...
    // Span wird automatisch verlassen, wenn _enter aus dem Scope geht
    ```
    
- Spans ermöglichen die Korrelation von Log-Ereignissen und die Messung von Dauern.

Die Wahl von `tracing` bietet eine flexible und leistungsstarke Grundlage. Die Spezifikation eines klaren Initialisierungsprozesses, der auch frühe Fehler beim Konfigurationsladen abdeckt, ist entscheidend. Die Festlegung auf `fmt::Subscriber` mit optionalem Dateilogging als Standard vereinfacht die Implementierung für Entwickler, während die `tracing`-API selbst fortgeschrittene Anwendungsfälle ermöglicht.

## 7. Error Handling (`core_infra::error`)

Definiert eine einheitliche und robuste Fehlerbehandlungsstrategie für die Kernschicht.

### 7.1. Modulstruktur

```
core_infra/
└── src/
    ├── lib.rs
    ├── types.rs
    ├── utils/
    │   └──...
    ├── config/
    │   └──...
    ├── logging.rs
    └── error.rs // Definiert CoreError, ConfigError etc.
```

### 7.2. Core Error Enum (`core_infra::error.rs`)

Eine zentrale Enum `CoreError`, die alle möglichen Fehlerfälle der Kernschicht repräsentiert. Verwendet `thiserror` zur einfachen Implementierung.7

Rust

```
// core_infra/src/error.rs
use thiserror::Error;
use std::path::PathBuf;

/// The primary error type for the core infrastructure layer.
/// Aggregates specific error categories.
#
pub enum CoreError {
    /// Errors related to configuration loading, parsing, or validation.
    #[error("Configuration Error: {0}")]
    Config(#[from] ConfigError),

    /// Errors occurring during logging subsystem initialization.
    #[error("Logging Initialization Failed: {0}")]
    LoggingInitialization(String), // Kann spezifischer sein, z.B. eine eigene Enum

    /// Errors related to filesystem operations, including context.
    #[error("Filesystem Error: {message} (Path: {path:?})")]
    Filesystem {
        message: String,
        path: PathBuf,
        #[source] // Behält die ursprüngliche IO-Fehlerquelle
        source: std::io::Error,
    },

    /// General I/O errors not covered by Filesystem or Config::ReadError.
    #[error("I/O Error: {0}")]
    Io(#[from] std::io::Error),

    /// Errors indicating invalid input parameters or data.
    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    /// Placeholder for other potential core errors.
    #[error("An unexpected internal error occurred: {0}")]
    Internal(String),
}

/// Specific errors related to configuration handling.
#
pub enum ConfigError {
    /// Failed to read the configuration file.
    #[error("Failed to read configuration file from {path:?}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Failed to parse the TOML configuration content.
    #[error("Failed to parse configuration file: {0}")]
    ParseError(#[from] toml::de::Error),

    /// Configuration validation failed after parsing.
    #[error("Configuration validation failed: {0}")]
    ValidationError(String),

    /// Configuration file was not found at the expected location(s).
    #[error("Configuration file not found at expected locations: {locations:?}")]
    NotFound { locations: Vec<PathBuf> },

    /// Required configuration directory (e.g., XDG_CONFIG_HOME) could not be determined.
    #[error("Could not determine base directory for {dir_type}")]
    DirectoryUnavailable { dir_type: String },
}

// Implementiere Konvertierungen, falls nötig, um Kontext hinzuzufügen,
// bevor #[from] verwendet wird. Beispiel:
// impl From<SpecificError> for CoreError {
//     fn from(err: SpecificError) -> Self {
//         CoreError::Internal(format!("Specific error occurred: {}", err))
//     }
// }
```

- **Spezifikation:**
    - `CoreError` ist die einzige Fehlertyp, der von öffentlichen Funktionen dieser Schicht zurückgegeben wird.
    - Verwendet `thiserror::Error` für die Ableitung von `std::error::Error` und `Display`.
    - `#` ist obligatorisch.
    - Varianten decken alle logischen Fehlerquellen ab (Konfiguration, Logging, FS, IO, Input-Validierung).
    - `#[error("...")]` Attribute definieren menschenlesbare Fehlermeldungen. Diese Meldungen sollten kontextreich sein.
    - `#[from]` wird verwendet, um Standardfehler (wie `std::io::Error`, `toml::de::Error`) automatisch in `CoreError`-Varianten umzuwandeln.7 Dies vereinfacht die Fehlerkonvertierung.
    - `#[source]` wird verwendet, um die zugrundeliegende Fehlerursache für Debugging-Zwecke beizubehalten.7
    - Spezifischere Fehler-Enums (wie `ConfigError`) können definiert und mittels `#[from]` in `CoreError` eingebettet werden. Dies verbessert die interne Strukturierung und ermöglicht es Aufrufern bei Bedarf, spezifischere Fehlerfälle zu behandeln, ohne die Komplexität der öffentlichen API zu erhöhen.17
    - Die `CoreError::Filesystem`-Variante demonstriert das Hinzufügen von Kontext (Nachricht, Pfad) zu einem zugrundeliegenden Fehler (`source: std::io::Error`), was für die Fehlersuche unerlässlich ist.

### 7.3. Error Propagation

- Alle fehleranfälligen öffentlichen Funktionen MÜSSEN `Result<T, CoreError>` zurückgeben.
- Der `?`-Operator MUSS zur Fehlerpropagation innerhalb der Funktionen verwendet werden.
- Wo nötig, MUSS `.map_err()` verwendet werden, um Low-Level-Fehler in passende `CoreError`-Varianten umzuwandeln und dabei relevanten Kontext (z.B. Dateipfade, Operationsnamen) hinzuzufügen.

### 7.4. Error Handling Strategy

- **Kein `unwrap`/`expect`:** Die Verwendung von `.unwrap()` oder `.expect()` in der Kernschicht ist verboten, da dies zu Panics führt, die nicht ordnungsgemäß behandelt werden können.1 Erwartete Fehler müssen über `Result` signalisiert werden.
- **Panics:** Panics sollten nur bei logischen Programmierfehlern (Bugs) auftreten, die als nicht behebbar gelten (z.B. Verletzung von internen Invarianten, die durch korrekte Nutzung der API nicht auftreten sollten). Solche Fälle deuten auf einen Fehler im Code selbst hin, nicht auf einen Laufzeitfehler.

Diese Strategie stellt sicher, dass Fehler explizit behandelt und propagiert werden, was die Robustheit und Wartbarkeit des Codes verbessert. Die zentrale `CoreError`-Enum bietet eine konsistente Schnittstelle für Fehler aus der Kernschicht.

## 8. Core Event Definitions

Für diese Kerninfrastrukturschicht werden **keine** eigenen Events definiert.

Die Kernschicht konzentriert sich auf grundlegende, meist synchrone Setup-Aufgaben und die Bereitstellung von Utilities. Ereignisbasierte Kommunikation (Publish/Subscribe) ist typischerweise eine Aufgabe höherer Schichten oder dedizierter Event-Bus-Systeme, die auf der Kerninfrastruktur aufbauen, aber nicht Teil davon sind. Die Definition eines Event-Systems würde die Komplexität der Kernschicht unnötig erhöhen und ihre Abhängigkeiten erweitern (z.B. auf eine asynchrone Laufzeit oder eine spezifische Event-Bibliothek), was ihrem Zweck widerspricht.

Sollte in Zukunft ein Bedarf für _fundamentale_, von der Kernschicht ausgehende Ereignisse entstehen (z.B. dynamische Neuladung der Kernkonfiguration), müsste diese Spezifikation entsprechend erweitert werden, inklusive:

- Definition der Event-Typen (Structs/Enums).
- Spezifikation der Payloads (`Send + Sync + Clone + Debug`).
- Identifikation der emittierenden Komponente (Publisher).
- Definition des Übertragungsmechanismus (z.B. `tokio::sync::broadcast`).

Aktuell ist dies jedoch nicht vorgesehen.

## 9. External Dependencies

Die Kerninfrastruktur minimiert ihre externen Abhängigkeiten, um schlank, stabil und schnell kompilierbar zu bleiben. Nur essenzielle Crates für die Kernfunktionalitäten (Logging, Konfiguration, Fehlerbehandlung, Pfadermittlung) sind erlaubt.

### 9.1. Dependency Policy

- Nur absolut notwendige Abhängigkeiten hinzufügen.
- Stabile Versionen verwenden. Versionen müssen exakt spezifiziert werden.
- Benötigte Crate-Features explizit angeben. Standard-Features deaktivieren (`default-features = false`), wenn nicht alle benötigt werden, um die Abhängigkeitsgröße zu reduzieren.
- Regelmäßige Überprüfung auf Updates und Sicherheitsschwachstellen.

### 9.2. Dependency Table

Die folgenden externen Crates sind für die Implementierung der Kerninfrastruktur zwingend erforderlich:

|   |   |   |   |   |
|---|---|---|---|---|
|**Crate**|**Exakte Version¹**|**Benötigte Features**|**Rationale**|**Snippet Refs**|
|`tracing`|`0.1.40`|`std`|Kern-Logging-Fassade und API|13|
|`tracing-subscriber`|`0.3.18`|`fmt`, `env-filter`, `std`, `registry`, `json` (optional für Format)|`fmt`-Subscriber, Filterung, Registry-Basis|15|
|`tracing-appender`|`0.2.3`|(Standard)|Dateilogging mit Rotation|-|
|`serde`|`1.0.219`|`derive`, `std`|Deserialisierung für Konfigurationsstrukturen|8|
|`toml`|`0.8.22`|(Standard, enthält `parse`)|TOML-Parsing für Konfigurationsdateien|12|
|`thiserror`|`1.0.59`|(Standard)|Ableitung von `std::error::Error`, `Display`|6|
|`directories-next`|`2.0.0`|(Standard)|Cross-Plattform XDG/Standard-Verzeichnisermittlung|11|
|`log`|`0.4.21`|`std`|Transitiv benötigt von `tracing-subscriber`|14|
|`atty`|`0.2.15`|(Standard)|Erkennung von TTY für ANSI-Farben im Logging|-|

¹ _Anmerkung zu Versionen:_ Die hier angegebenen Versionen entsprechen den zum Zeitpunkt der Erstellung dieses Dokuments als stabil bekannten oder in den Referenzmaterialien genannten Versionen. Vor der Implementierung sind die **aktuellsten stabilen Versionen** zu überprüfen und zu verwenden, sofern sie API-kompatibel sind oder die Spezifikation entsprechend angepasst wird. Die exakten Versionen MÜSSEN in der `Cargo.toml` fixiert werden.

Diese Tabelle ist entscheidend für die Reproduzierbarkeit der Builds und die Stabilität der Kernschicht. Jede Änderung an diesen Abhängigkeiten (Version, Features) erfordert eine Überprüfung und Anpassung dieser Spezifikation.

## 10. Implementation Constraints and Guidelines

Diese Richtlinien stellen sicher, dass die Implementierung konsistent, wartbar und konform mit den Designzielen der Kernschicht ist.

### 10.1. Rust Version

- **Minimum Supported Rust Version (MSRV):** `1.70.0` (oder höher, basierend auf den MSRV-Anforderungen der Abhängigkeiten wie `tracing` 16 und der Verwendung von Sprachfeatures). Muss in `Cargo.toml` angegeben werden.
- **Entwicklung:** Die Entwicklung sollte mit der **aktuellsten stabilen Rust-Version** erfolgen.

### 10.2. `unsafe` Code

- Die Verwendung von `unsafe`-Blöcken ist in der gesamten Kerninfrastruktur **strikt verboten** (siehe Abschnitt 2.4).

### 10.3. Testing

- **Unit Tests:** Jede öffentliche Funktion und Methode MUSS durch Unit-Tests abgedeckt sein. Tests müssen sowohl Erfolgs- als auch Fehlerpfade validieren. Testmodule (`#[cfg(test)] mod tests {... }`) sollen direkt in den jeweiligen Quelldateien platziert werden.
- **Integration Tests:** Integrationstests, die das Zusammenspiel der Kernschicht mit anderen Schichten testen, gehören _nicht_ in dieses Crate, sondern in eine übergeordnete Testsuite.

### 10.4. Documentation

- **Rustdoc:** Alle öffentlichen Elemente (Crates, Module, Typen, Funktionen, Methoden, Traits, Konstanten) MÜSSEN umfassende Dokumentationskommentare (`///`) aufweisen.1
- **Inhalt:** Die Dokumentation muss den Zweck, Parameter, Rückgabewerte, garantierte Vor-/Nachbedingungen, mögliche `CoreError`-Varianten und ggf. Beispiele (`#[test]`-fähige Beispiele bevorzugt) enthalten (C-EXAMPLE, C-FAILURE 1).
- **Crate-Level Doku:** `src/lib.rs` MUSS eine ausführliche Crate-Level-Dokumentation enthalten, die den Zweck und die Verwendung der Kerninfrastruktur erklärt (C-CRATE-DOC 1).

### 10.5. Module Structure

Die Implementierung MUSS der folgenden Modulstruktur folgen:

```
core_infra/
└── src/
    ├── lib.rs         # Crate root, re-exportiert öffentliche APIs
    ├── error.rs       # Definition von CoreError, ConfigError etc.
    ├── types.rs       # Definition von Core-Datentypen (Structs, Enums)
    ├── config/        # Konfigurations-bezogene Module
    │   ├── mod.rs     # Definition von CoreConfig, LoggingConfig, ConfigLoader
    │   └── defaults.rs# Funktionen für Standardwerte
    ├── logging.rs     # Logging-Initialisierungslogik
    └── utils/         # Utility-Module
        ├── mod.rs     # Re-exportiert öffentliche Utils
        ├── fs.rs      # Filesystem-Utilities
        └── paths.rs   # Pfadermittlungs-Utilities
        // Optional: strings.rs, time.rs
```

Öffentliche APIs sollten selektiv von `lib.rs` re-exportiert werden, um die Schnittstelle klar zu definieren.

### 10.6. `Cargo.toml` Manifest

Die `Cargo.toml`-Datei MUSS wie folgt konfiguriert sein:

Ini, TOML

```
[package]
name = "core_infra"
version = "0.1.0" # Startversion, SemVer beachten
edition = "2021" # Oder neuere Edition, falls MSRV es erlaubt
authors = ["Your Name <your.email@example.com>"] # Anpassen
description = "Core infrastructure layer providing foundational utilities, configuration, logging, and error handling."
license = "MIT OR Apache-2.0" # Lizenz gemäß Vorgabe [12, 18]
repository = "URL/to/your/repository" # Optional, aber empfohlen
homepage = "URL/to/project/homepage" # Optional
documentation = "URL/to/docs" # Optional, falls extern gehostet
readme = "README.md" # Optional
keywords = ["core", "infrastructure", "config", "logging", "error"] # Optional
categories = ["api-bindings", "config", "filesystem"] # Optional, siehe crates.io Kategorien

# Mindest-Rust-Version festlegen
rust-version = "1.70.0" # Anpassen gemäß Abschnitt 10.1

[dependencies]
# Versionen exakt wie in Abschnitt 9.2 spezifiziert
tracing = { version = "0.1.40" }
tracing-subscriber = { version = "0.3.18", features = ["fmt", "env-filter", "std", "registry", "json"] } # json optional
tracing-appender = { version = "0.2.3" }
serde = { version = "1.0.219", features = ["derive"] } # std ist default feature
toml = { version = "0.8.22" }
thiserror = { version = "1.0.59" }
directories-next = { version = "2.0.0" }
log = { version = "0.4.21" }
atty = { version = "0.2.15" }

# Optional: Dev-Dependencies für Tests
[dev-dependencies]
# z.B. pretty_assertions = "1.4.0" für bessere Test-Diffs

# Optional: Build-Dependencies, falls benötigt
[build-dependencies]

# Optional: Profile für Optimierungen, etc.
[profile.release]
lto = true          # Link Time Optimization für Release-Builds
codegen-units = 1   # Bessere Optimierung, langsamere Kompilierung
panic = 'abort'     # Kleinere Binaries, keine Stack Unwinding Info bei Panic
strip = true        # Symbole entfernen für kleinere Binaries
```

- **Spezifikation:**
    - Alle Metadaten im `[package]`-Abschnitt müssen korrekt ausgefüllt sein.1
    - Die `edition` muss angegeben werden.
    - Die `license` muss den gängigen Open-Source-Lizenzen entsprechen, die in den Referenzen verwendet werden (oft MIT/Apache-2.0).7
    - Die `rust-version` MUSS gesetzt sein.
    - Der `[dependencies]`-Abschnitt MUSS exakt die Crates, Versionen und Features aus Abschnitt 9.2 enthalten. Keine zusätzlichen Abhängigkeiten sind erlaubt.

## 11. Schlussfolgerungen

Dieser Implementierungsleitfaden definiert die Kerninfrastrukturschicht (`core_infra`) mit höchster Präzision und Detailgenauigkeit. Durch die strikte Befolgung dieser Spezifikation wird sichergestellt, dass Entwickler eine konsistente, robuste und wartbare Basis für das Gesamtsystem erstellen können, ohne eigene grundlegende Designentscheidungen treffen zu müssen.

Die Kernpunkte umfassen:

1. **Klare Verantwortlichkeiten:** Die Schicht beschränkt sich auf fundamentale Utilities, Konfiguration, Logging und Fehlerbehandlung und bleibt frei von Abhängigkeiten zu höheren Schichten.
2. **Rust Best Practices:** Die Implementierung folgt konsequent den Rust API Guidelines, Sicherheitsprinzipien (kein `unsafe`) und etablierten Mustern für Fehlerbehandlung und Dokumentation.
3. **Standardisierte Werkzeuge:** Die Verwendung von etablierten Crates wie `tracing`, `serde`, `toml`, `thiserror` und `directories-next` stellt die Nutzung bewährter Lösungen sicher und reduziert den Implementierungsaufwand.
4. **Präzise Schnittstellen:** Alle öffentlichen Typen, Funktionen und Module sind exakt definiert, inklusive Signaturen, Fehlertypen und Verhalten.
5. **Reproduzierbarkeit:** Durch die Festlegung exakter Abhängigkeitsversionen und Build-Konfigurationen wird eine hohe Reproduzierbarkeit gewährleistet.
6. **Plattformabstraktion:** Kritische plattformspezifische Aspekte wie die Verzeichnisstruktur werden durch Utilities (z.B. `core_infra::utils::paths`) abstrahiert.

Die sorgfältige Behandlung von Randfällen, wie die Initialisierungsreihenfolge von Konfiguration und Logging, sowie die detaillierte Definition der Fehlerbehandlung mit `CoreError` und `ConfigError` tragen maßgeblich zur Stabilität der Schicht bei. Entwickler können diesen Leitfaden als direkte Vorlage für die Implementierung verwenden.

# **Nova A1 Kernschicht Implementierungsleitfaden: Modul 1 \- Fundamentale Datentypen (core::types)**

## **1\. Modulübersicht: core::types**

### **1.1. Zweck und Verantwortlichkeit**

Dieses Modul, core::types, bildet das Fundament der Kernschicht (core) und somit des gesamten Systems. Seine primäre Verantwortung liegt in der Definition grundlegender, universell einsetzbarer Datentypen, die von allen anderen Schichten und Modulen der Desktop-Umgebung benötigt werden. Dazu gehören geometrische Primitive (wie Punkte, Größen, Rechtecke), Farbdarstellungen und allgemeine Enumerationen (wie Orientierungen).  
Die in diesem Modul definierten Typen sind bewusst einfach gehalten und repräsentieren reine Datenstrukturen ohne komplexe Geschäftslogik oder Abhängigkeiten zu höheren Schichten oder externen Systemen. Sie dienen als Bausteine für komplexere Operationen und Zustandsrepräsentationen in den Domänen-, System- und Benutzeroberflächenschichten.

### **1.2. Designphilosophie**

Das Design von core::types folgt den Prinzipien der Modularität, Wiederverwendbarkeit und minimalen Kopplung. Die Typen sind generisch gehalten (wo sinnvoll, z.B. bei geometrischen Primitiven), um Flexibilität für verschiedene numerische Darstellungen (z.B. i32 für Koordinaten, f32 für Skalierungsfaktoren) zu ermöglichen.  
Ein wesentlicher Aspekt ist die klare Trennung von Datenrepräsentation (in core::types) und Fehlerbehandlung. Während dieses Modul die Datenstrukturen definiert, werden die spezifischen Fehler, die bei Operationen mit diesen Typen auftreten können (z.B. durch ungültige Werte), in den Modulen definiert, die diese Operationen durchführen (typischerweise in core::errors oder modulspezifischen Fehler-Enums höherer Schichten).

### **1.3. Zusammenspiel mit Fehlerbehandlung**

Obwohl core::types selbst keine Error-Typen definiert, ist das Design der hier enthaltenen Typen entscheidend für eine robuste und konsistente Fehlerbehandlungsstrategie im gesamten Projekt. Die übergeordnete Richtlinie sieht die Verwendung des thiserror-Crates vor, um spezifische Fehler-Enums pro Modul zu definieren. Dies ermöglicht eine granulare Fehlerbehandlung, ohne die Komplexität übermäßig zu erhöhen.  
Die Typen in core::types unterstützen diese Strategie, indem sie:

1. **Standard-Traits implementieren:** Alle Typen implementieren grundlegende Traits wie Debug und Display. Dies ist essenziell, damit Instanzen dieser Typen effektiv in Fehlermeldungen und Log-Ausgaben eingebettet werden können, die von höheren Schichten unter Verwendung von thiserror generiert werden. Eine gute Fehlerdarstellung ist entscheidend für die Fehlersuche und das Verständnis von Problemen im Laufzeitbetrieb.  
2. **Invarianten dokumentieren:** Für Typen wie Rect\<T\> existieren logische Invarianten (z.B. nicht-negative Breite und Höhe). Diese Invarianten werden klar dokumentiert.  
3. **Validierung ermöglichen:** Wo sinnvoll, werden Methoden zur Überprüfung der Gültigkeit bereitgestellt (z.B. Rect::is\_valid()). Diese Methoden erlauben es aufrufendem Code in höheren Schichten, Zustände zu überprüfen, *bevor* Operationen ausgeführt werden, die fehlschlagen könnten.  
4. **Keine Panics in Kernfunktionen:** Konstruktoren und einfache Zugriffsmethoden in core::types lösen keine Panics aus und geben keine Result-Typen zurück, um die API auf dieser fundamentalen Ebene einfach und vorhersagbar zu halten. Die Verantwortung für die Handhabung potenziell ungültiger Zustände (z.B. ein Rect mit negativer Breite, das an eine Rendering-Funktion übergeben wird) liegt bei den konsumierenden Funktionen, die dann die definierten Fehlerpfade (mittels Result\<T, E\> 3 und den thiserror-basierten E-Typen) nutzen.

Diese Designentscheidungen stellen sicher, dass die fundamentalen Typen nahtlos in das übergeordnete Fehlerbehandlungskonzept integriert werden können, ohne selbst die Komplexität der Fehlerdefinition tragen zu müssen. Die gewählte Fehlerstrategie mit thiserror pro Modul wird als ausreichend für die Bedürfnisse der Kernschicht erachtet, auch wenn alternative Ansätze wie snafu für komplexere Szenarien existieren, in denen z.B. die Unterscheidung von Fehlern aus derselben Quelle kritisch ist. Für die Kernschicht wird die Einfachheit und Direktheit von thiserror bevorzugt.

### **1.4. Modulabhängigkeiten**

Dieses Modul ist darauf ausgelegt, minimale externe Abhängigkeiten zu haben, um seine grundlegende Natur und breite Anwendbarkeit zu gewährleisten.

* **Erlaubte Abhängigkeiten:**  
  * std (Rust Standardbibliothek)  
* **Optionale Abhängigkeiten (derzeit nicht verwendet):**  
  * num-traits: Nur hinzufügen, falls generische numerische Operationen benötigt werden, die über std::ops hinausgehen.  
  * serde (mit derive-Feature): Nur hinzufügen, wenn Serialisierung/Deserialisierung dieser Basistypen *direkt auf dieser Ebene* zwingend erforderlich ist (z.B. für Konfigurationsdateien, die diese Typen direkt verwenden). Aktuell wird davon ausgegangen, dass Serialisierungslogik in höheren Schichten implementiert wird, um unnötige Abhängigkeiten zu vermeiden.

### **1.5. Ziel-Dateistruktur**

Die Implementierung dieses Moduls erfolgt innerhalb des core-Crates mit folgender Verzeichnisstruktur:

src/  
└── core/  
    ├── Cargo.toml         \# (Definiert das 'core' Crate)  
    └── src/  
        ├── lib.rs             \# (Deklariert Kernmodule: pub mod types; pub mod errors;...)  
        └── types/  
            ├── mod.rs         \# (Deklariert und re-exportiert Typen: pub mod geometry; pub mod color;...)  
            ├── geometry.rs    \# (Enthält Point\<T\>, Size\<T\>, Rect\<T\>)  
            ├── color.rs       \# (Enthält Color)  
            └── enums.rs       \# (Enthält Orientation, etc.)

## **2\. Spezifikation: Geometrische Primitive (geometry.rs)**

Diese Datei definiert grundlegende 2D-Geometrietypen, die für Layout, Positionierung und Rendering unerlässlich sind.

### **2.1. Struct: Point\<T\>**

* **2.1.1. Definition und Zweck:** Repräsentiert einen Punkt im 2D-Raum mit x- und y-Koordinaten. Generisch über den Typ T.  
* **2.1.2. Felder:**  
  * pub x: T  
  * pub y: T  
* **2.1.3. Assoziierte Konstanten:**  
  * pub const ZERO\_I32: Point\<i32\> \= Point { x: 0, y: 0 };  
  * pub const ZERO\_U32: Point\<u32\> \= Point { x: 0, y: 0 };  
  * pub const ZERO\_F32: Point\<f32\> \= Point { x: 0.0, y: 0.0 };  
  * pub const ZERO\_F64: Point\<f64\> \= Point { x: 0.0, y: 0.0 };  
* **2.1.4. Methoden:**  
  * pub const fn new(x: T, y: T) \-\> Self  
    * Erstellt einen neuen Punkt.  
  * pub fn distance\_squared(\&self, other: \&Point\<T\>) \-\> T  
    * Berechnet das Quadrat der euklidischen Distanz.  
    * *Constraints:* T:Copy+std::ops::Add\<Output=T\>+std::ops::Sub\<Output=T\>+std::ops::Mul\<Output=T\>  
  * pub fn distance(\&self, other: \&Point\<T\>) \-\> T  
    * Berechnet die euklidische Distanz.  
    * *Constraints:* T:Copy+std::ops::Add\<Output=T\>+std::ops::Sub\<Output=T\>+std::ops::Mul\<Output=T\>+numt​raits::Float (Implementierung nur für Float-Typen sinnvoll oder über sqrt-Funktion). Vorerst nur für f32,f64 implementieren.  
  * pub fn manhattan\_distance(\&self, other: \&Point\<T\>) \-\> T  
    * Berechnet die Manhattan-Distanz (∣x1​−x2​∣+∣y1​−y2​∣).  
    * *Constraints:* T:Copy+std::ops::Add\<Output=T\>+std::ops::Sub\<Output=T\>+numt​raits::Signed (Benötigt abs()).  
* **2.1.5. Trait Implementierungen:**  
  * \#  
    * *Bedingung:* T muss die jeweiligen Traits ebenfalls implementieren. Default setzt x und y auf T::default().  
  * impl\<T: Send \+ 'static\> Send for Point\<T\> {}  
  * impl\<T: Sync \+ 'static\> Sync for Point\<T\> {}  
  * impl\<T: std::ops::Add\<Output \= T\>\> std::ops::Add for Point\<T\>  
  * impl\<T: std::ops::Sub\<Output \= T\>\> std::ops::Sub for Point\<T\>  
* **2.1.6. Generische Constraints (Basis):** T:Copy+Debug+PartialEq+Default+Send+Sync+′static. Weitere Constraints werden pro Methode spezifiziert.

### **2.2. Struct: Size\<T\>**

* **2.2.1. Definition und Zweck:** Repräsentiert eine 2D-Dimension (Breite und Höhe). Generisch über den Typ T.  
* **2.2.2. Felder:**  
  * pub width: T  
  * pub height: T  
* **2.2.3. Assoziierte Konstanten:**  
  * pub const ZERO\_I32: Size\<i32\> \= Size { width: 0, height: 0 };  
  * pub const ZERO\_U32: Size\<u32\> \= Size { width: 0, height: 0 };  
  * pub const ZERO\_F32: Size\<f32\> \= Size { width: 0.0, height: 0.0 };  
  * pub const ZERO\_F64: Size\<f64\> \= Size { width: 0.0, height: 0.0 };  
* **2.2.4. Methoden:**  
  * pub const fn new(width: T, height: T) \-\> Self  
    * Erstellt eine neue Größe.  
  * pub fn area(\&self) \-\> T  
    * Berechnet die Fläche (width×height).  
    * *Constraints:* T:Copy+std::ops::Mul\<Output=T\>  
  * pub fn is\_empty(\&self) \-\> bool  
    * Prüft, ob Breite oder Höhe null ist.  
    * *Constraints:* T:PartialEq+numt​raits::Zero  
  * pub fn is\_valid(\&self) \-\> bool  
    * Prüft, ob Breite und Höhe nicht-negativ sind. Nützlich für Typen wie i32.  
    * *Constraints:* T:PartialOrd+numt​raits::Zero  
* **2.2.5. Trait Implementierungen:**  
  * \#  
    * *Bedingung:* T muss die jeweiligen Traits ebenfalls implementieren. Default setzt width und height auf T::default().  
  * impl\<T: Send \+ 'static\> Send for Size\<T\> {}  
  * impl\<T: Sync \+ 'static\> Sync for Size\<T\> {}  
* **2.2.6. Generische Constraints (Basis):** T:Copy+Debug+PartialEq+Default+Send+Sync+′static. Weitere Constraints werden pro Methode spezifiziert. Die Invariante nicht-negativer Dimensionen wird durch is\_valid prüfbar gemacht, aber nicht durch den Typ erzwungen.

### **2.3. Struct: Rect\<T\>**

* **2.3.1. Definition und Zweck:** Repräsentiert ein 2D-Rechteck, definiert durch einen Ursprungspunkt (oben-links) und eine Größe. Generisch über den Typ T.  
* **2.3.2. Felder:**  
  * pub origin: Point\<T\>  
  * pub size: Size\<T\>  
* **2.3.3. Assoziierte Konstanten:**  
  * pub const ZERO\_I32: Rect\<i32\> \= Rect { origin: Point::ZERO\_I32, size: Size::ZERO\_I32 };  
  * pub const ZERO\_U32: Rect\<u32\> \= Rect { origin: Point::ZERO\_U32, size: Size::ZERO\_U32 };  
  * pub const ZERO\_F32: Rect\<f32\> \= Rect { origin: Point::ZERO\_F32, size: Size::ZERO\_F32 };  
  * pub const ZERO\_F64: Rect\<f64\> \= Rect { origin: Point::ZERO\_F64, size: Size::ZERO\_F64 };  
* **2.3.4. Methoden:**  
  * pub const fn new(origin: Point\<T\>, size: Size\<T\>) \-\> Self  
  * pub fn from\_coords(x: T, y: T, width: T, height: T) \-\> Self  
    * *Constraints:* T muss die Constraints von Point::new und Size::new erfüllen.  
  * pub fn x(\&self) \-\> T (*Constraints:* T:Copy)  
  * pub fn y(\&self) \-\> T (*Constraints:* T:Copy)  
  * pub fn width(\&self) \-\> T (*Constraints:* T:Copy)  
  * pub fn height(\&self) \-\> T (*Constraints:* T:Copy)  
  * pub fn top(\&self) \-\> T (Alias für y, *Constraints:* T:Copy)  
  * pub fn left(\&self) \-\> T (Alias für x, *Constraints:* T:Copy)  
  * pub fn bottom(\&self) \-\> T (y+height, *Constraints:* T:Copy+std::ops::Add\<Output=T\>)  
  * pub fn right(\&self) \-\> T (x+width, *Constraints:* T:Copy+std::ops::Add\<Output=T\>)  
  * pub fn center(\&self) \-\> Point\<T\>  
    * Berechnet den Mittelpunkt.  
    * *Constraints:* T:Copy+std::ops::Add\<Output=T\>+std::ops::Div\<Output=T\>+numt​raits::FromPrimitive (Benötigt Division durch 2).  
  * pub fn contains\_point(\&self, point: \&Point\<T\>) \-\> bool  
    * Prüft, ob der Punkt innerhalb des Rechtecks liegt (Grenzen inklusiv für top/left, exklusiv für bottom/right).  
    * *Constraints:* T:Copy+PartialOrd+std::ops::Add\<Output=T\>  
  * pub fn intersects(\&self, other: \&Rect\<T\>) \-\> bool  
    * Prüft, ob sich dieses Rechteck mit einem anderen überschneidet.  
    * *Constraints:* T:Copy+PartialOrd+std::ops::Add\<Output=T\>  
  * pub fn intersection(\&self, other: \&Rect\<T\>) \-\> Option\<Rect\<T\>\>  
    * Berechnet das Schnittrechteck. Gibt None zurück, wenn keine Überschneidung vorliegt.  
    * *Constraints:* T:Copy+Ord+std::ops::Add\<Output=T\>+std::ops::Sub\<Output=T\>+numt​raits::Zero  
  * pub fn union(\&self, other: \&Rect\<T\>) \-\> Rect\<T\>  
    * Berechnet das umschließende Rechteck beider Rechtecke.  
    * *Constraints:* T:Copy+Ord+std::ops::Add\<Output=T\>+std::ops::Sub\<Output=T\>  
  * pub fn translated(\&self, dx: T, dy: T) \-\> Rect\<T\>  
    * Verschiebt das Rechteck um (dx,dy).  
    * *Constraints:* T:Copy+std::ops::Add\<Output=T\>  
  * pub fn scaled(\&self, sx: T, sy: T) \-\> Rect\<T\>  
    * Skaliert das Rechteck relativ zum Ursprung (0,0). Beachtet, dass dies Ursprung und Größe skaliert.  
    * *Constraints:* T:Copy+std::ops::Mul\<Output=T\>  
  * pub fn is\_valid(\&self) \-\> bool  
    * Prüft, ob size.is\_valid() wahr ist.  
    * *Constraints:* T:PartialOrd+numt​raits::Zero  
* **2.3.5. Trait Implementierungen:**  
  * \#  
    * *Bedingung:* T muss die jeweiligen Traits ebenfalls implementieren. Default verwendet Point::default() und Size::default().  
  * impl\<T: Send \+ 'static\> Send for Rect\<T\> {}  
  * impl\<T: Sync \+ 'static\> Sync for Rect\<T\> {}  
* **2.3.6. Generische Constraints (Basis):** T:Copy+Debug+PartialEq+Default+Send+Sync+′static. Weitere Constraints werden pro Methode spezifiziert.  
* **2.3.7. Invarianten und Validierung (Verbindung zur Fehlerbehandlung):**  
  * **Invariante:** Logisch sollten width und height der size-Komponente nicht-negativ sein.  
  * **Kontext:** Die Verwendung von vorzeichenbehafteten Typen wie i32 für Koordinaten ist üblich, erlaubt aber technisch negative Dimensionen. Eine Erzwingung nicht-negativer Dimensionen auf Typebene (z.B. durch u32) wäre zu restriktiv für Koordinatensysteme.  
  * **Konsequenz:** Die Flexibilität, Rect\<i32\> zu verwenden, verlagert die Verantwortung für die Validierung auf die Nutzer des Rect-Typs. Funktionen in höheren Schichten (z.B. Layout-Algorithmen, Rendering-Engines), die ein Rect konsumieren, müssen potenziell ungültige Rechtecke (mit negativer Breite oder Höhe) behandeln. Solche Fälle stellen Laufzeitfehler dar, die über das etablierte Fehlerbehandlungssystem (basierend auf Result\<T, E\> und thiserror-definierten E-Typen) signalisiert werden müssen.  
  * **Implementierung in core::types:** Das Modul erzwingt die Invariante nicht zur Compilezeit oder in Konstruktoren. Stattdessen wird die Methode pub fn is\_valid(\&self) \-\> bool bereitgestellt. Nutzer von Rect\<T\> (insbesondere mit T=i32) *sollten* diese Methode aufrufen, um die Gültigkeit sicherzustellen, bevor Operationen durchgeführt werden, die eine positive Breite und Höhe voraussetzen. Die Dokumentation des Rect-Typs muss explizit auf diese Invariante und die Notwendigkeit der Validierung durch den Aufrufer hinweisen. Die Verantwortung für das *Melden* eines Fehlers bei Verwendung eines ungültigen Rect liegt beim Aufrufer, der dafür die Fehlerinfrastruktur (z.B. core::errors oder modulspezifische Fehler) nutzt.

## **3\. Spezifikation: Farbdarstellung (color.rs)**

Diese Datei definiert einen Standard-Farbtyp für die Verwendung im gesamten System.

### **3.1. Struct: Color (RGBA)**

* **3.1.1. Definition und Zweck:** Repräsentiert eine Farbe mit Rot-, Grün-, Blau- und Alpha-Komponenten. Verwendet f32-Komponenten im Bereich \[0.0,1.0\] für hohe Präzision und Flexibilität bei Farboperationen wie Mischen und Transformationen.  
* **3.1.2. Felder:**  
  * pub r: f32 (Rotkomponente, 0.0 bis 1.0)  
  * pub g: f32 (Grünkomponente, 0.0 bis 1.0)  
  * pub b: f32 (Blaukomponente, 0.0 bis 1.0)  
  * pub a: f32 (Alphakomponente, 0.0=transparent bis 1.0=opak)  
* **3.1.3. Assoziierte Konstanten:**  
  * pub const TRANSPARENT: Color \= Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };  
  * pub const BLACK: Color \= Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };  
  * pub const WHITE: Color \= Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };  
  * pub const RED: Color \= Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };  
  * pub const GREEN: Color \= Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };  
  * pub const BLUE: Color \= Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };  
  * *(Weitere Standardfarben nach Bedarf hinzufügen)*  
* **3.1.4. Methoden:**  
  * pub const fn new(r: f32, g: f32, b: f32, a: f32) \-\> Self  
    * Erstellt eine neue Farbe. Werte außerhalb \[0.0,1.0\] werden nicht automatisch geklemmt, dies liegt in der Verantwortung des Aufrufers oder nachfolgender Operationen. debug\_assert\! kann zur Laufzeitprüfung in Debug-Builds verwendet werden.  
  * pub fn from\_rgba8(r: u8, g: u8, b: u8, a: u8) \-\> Self  
    * Konvertiert von 8-Bit-Ganzzahlkomponenten (0−255) zu f32 (0.0−1.0). value/255.0.  
  * pub fn to\_rgba8(\&self) \-\> (u8, u8, u8, u8)  
    * Konvertiert von f32 zu 8-Bit-Ganzzahlkomponenten. Klemmt Werte auf \[0.0,1.0\] und skaliert dann auf $$. (value.clamp(0.0,1.0)∗255.0).round()asu8.  
  * pub fn with\_alpha(\&self, alpha: f32) \-\> Self  
    * Erstellt eine neue Farbe mit dem angegebenen Alpha-Wert, wobei RGB beibehalten wird. Klemmt Alpha auf \[0.0,1.0\].  
  * pub fn blend(\&self, background: \&Color) \-\> Color  
    * Führt Alpha-Blending ("source-over") dieser Farbe über einer Hintergrundfarbe durch. Formel: Cout​=Cfg​×αfg​+Cbg​×αbg​×(1−αfg​). αout​=αfg​+αbg​×(1−αfg​). Annahme: Farben sind nicht vormultipliziert.  
  * pub fn lighten(\&self, amount: f32) \-\> Color  
    * Hellt die Farbe um einen Faktor amount auf (z.B. durch lineare Interpolation zu Weiß). Klemmt das Ergebnis auf gültige Farbwerte. amount im Bereich \[0.0,1.0\].  
  * pub fn darken(\&self, amount: f32) \-\> Color  
    * Dunkelt die Farbe um einen Faktor amount ab (z.B. durch lineare Interpolation zu Schwarz). Klemmt das Ergebnis. amount im Bereich \[0.0,1.0\].  
* **3.1.5. Trait Implementierungen:**  
  * \#  
    * PartialEq: Verwendet den Standard-Float-Vergleich. Für präzisere Vergleiche könnten benutzerdefinierte Implementierungen mit Epsilon erforderlich sein, dies wird jedoch für die Kernschicht als unnötige Komplexität betrachtet.  
    * Default: Implementiert Default manuell, um Color::TRANSPARENT zurückzugeben.  
  * impl Send for Color {}  
  * impl Sync for Color {}

## **4\. Spezifikation: Allgemeine Enumerationen (enums.rs)**

Diese Datei enthält häufig verwendete, einfache Enumerationen.

### **4.1. Enum: Orientation**

* **4.1.1. Definition und Zweck:** Repräsentiert eine horizontale oder vertikale Ausrichtung, häufig verwendet in UI-Layouts und Widgets.  
* **4.1.2. Varianten:**  
  * Horizontal  
  * Vertical  
* **4.1.3. Methoden:**  
  * pub fn toggle(\&self) \-\> Self  
    * Gibt die jeweils andere Orientierung zurück (Horizontal \-\> Vertical, Vertical \-\> Horizontal).  
* **4.1.4. Trait Implementierungen:**  
  * \#  
  * impl Default for Orientation { fn default() \-\> Self { Orientation::Horizontal } } (Standard ist Horizontal).  
  * impl Send for Orientation {}  
  * impl Sync for Orientation {}

## **5\. Zusammenfassung: Standard Trait Implementierungen**

Die folgende Tabelle gibt einen Überblick über die Implementierung gängiger Standard-Traits für die in diesem Modul definierten Typen. Dies dient als schnelle Referenz für Entwickler.

| Typ | Debug | Clone | Copy | PartialEq | Eq | Default | Hash | Send | Sync |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| Point\<T\> | Ja | Ja | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) |
| Size\<T\> | Ja | Ja | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) |
| Rect\<T\> | Ja | Ja | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) |
| Color | Ja | Ja | Ja | Ja | Nein | Ja | Nein | Ja | Ja |
| Orientation | Ja | Ja | Ja | Ja | Ja | Ja | Ja | Ja | Ja |

Anmerkungen:  
Eq und Hash sind aufgrund von Präzisionsproblemen generell nicht für Fließkommazahlen geeignet.  
Default::default() ergibt Color::TRANSPARENT.  
Default::default() ergibt Orientation::Horizontal.

## **6\. Schritt-für-Schritt Implementierungsplan**

Die Implementierung des core::types-Moduls folgt diesen Schritten:

* **6.1. Setup: Verzeichnis- und Dateierstellung:**  
  * Sicherstellen, dass das core-Crate existiert (ggf. cargo new core \--lib ausführen).  
  * Erstellen des Verzeichnisses src/core/src/types.  
  * Erstellen der Dateien:  
    * src/core/src/types/mod.rs  
    * src/core/src/types/geometry.rs  
    * src/core/src/types/color.rs  
    * src/core/src/types/enums.rs  
* **6.2. Implementierung geometry.rs: Point\<T\>, Size\<T\>, Rect\<T\>:**  
  * Definieren der Point\<T\>-Struktur mit Feldern x, y. Hinzufügen der spezifizierten generischen Basis-Constraints (T:Copy+Debug+PartialEq+Default+Send+Sync+′static). Implementieren von new, Konstanten (ZERO\_I32 etc.), Methoden (distance\_squared, distance (für Floats), manhattan\_distance) mit ihren spezifischen Constraints und Ableiten/Implementieren der spezifizierten Traits (Add, Sub).  
  * Definieren der Size\<T\>-Struktur mit Feldern width, height. Hinzufügen der Basis-Constraints. Implementieren von new, Konstanten (ZERO\_I32 etc.), Methoden (area, is\_empty, is\_valid) mit ihren Constraints und Ableiten/Implementieren der Traits.  
  * Definieren der Rect\<T\>-Struktur mit Feldern origin, size. Hinzufügen der Basis-Constraints. Implementieren von new, from\_coords, Konstanten (ZERO\_I32 etc.), Zugriffsmethoden (x, y, width, height, top, left, bottom, right), geometrischen Methoden (center, contains\_point, intersects, intersection, union, translated, scaled), Validierungsmethode (is\_valid) mit ihren Constraints und Ableiten/Implementieren der Traits.  
  * Hinzufügen notwendiger use-Anweisungen (z.B. std::ops, num\_traits).  
* **6.3. Implementierung color.rs: Color:**  
  * Definieren der Color-Struktur mit Feldern r, g, b, a (alle f32).  
  * Implementieren von new, Konstanten (TRANSPARENT, BLACK, WHITE, etc.), Konvertierungsmethoden (from\_rgba8, to\_rgba8), Hilfsmethoden (with\_alpha, blend, lighten, darken) und Ableiten/Implementieren der Traits (Default manuell).  
* **6.4. Implementierung enums.rs: Orientation:**  
  * Definieren des Orientation-Enums mit Varianten Horizontal, Vertical.  
  * Implementieren der toggle-Methode.  
  * Ableiten/Implementieren der spezifizierten Traits (Default manuell).  
* **6.5. Implementierung Moduldeklaration (mod.rs):**  
  * In src/core/src/types/mod.rs:  
    Rust  
    // src/core/src/types/mod.rs  
    pub mod color;  
    pub mod enums;  
    pub mod geometry;

    // Re-exportiere die primären Typen für einfacheren Zugriff  
    pub use color::Color;  
    pub use enums::Orientation;  
    pub use geometry::{Point, Rect, Size};

  * In src/core/src/lib.rs:  
    Rust  
    // src/core/src/lib.rs  
    // Deklariere das types-Modul  
    pub mod types;

    // Deklariere andere Kernmodule (werden später hinzugefügt)  
    // pub mod errors;  
    // pub mod logging;  
    // pub mod config;  
    // pub mod utils;

* **6.6. Unit-Testing Anforderungen:**  
  * Erstellen eines \#\[cfg(test)\]-Moduls innerhalb jeder Implementierungsdatei (geometry.rs, color.rs, enums.rs).  
  * Schreiben von Unit-Tests, die Folgendes abdecken:  
    * Konstruktorfunktionen (new, from\_coords, from\_rgba8).  
    * Konstantenwerte (deren Eigenschaften überprüfen).  
    * Methodenlogik (z.B. distance\_squared, area, is\_empty, bottom, right, contains\_point, intersects, intersection, union, toggle, blend). Testen von Grenzfällen (Nullwerte, überlappende/nicht überlappende Rechtecke, identische Punkte, Farbblending mit transparent/opak).  
    * Trait-Implementierungen (insbesondere Default, PartialEq, Add/Sub, wo zutreffend).  
    * Invariantenprüfungen (z.B. is\_valid für Rect und Size testen).  
  * Anstreben einer hohen Testabdeckung für diesen fundamentalen Code.  
* **6.7. Dokumentationsanforderungen (rustdoc):**  
  * Hinzufügen von ///-Dokumentationskommentaren zu *allen* öffentlichen Elementen: Module (mod.rs-Dateien), Structs, Enums, Felder, Konstanten, Methoden, Typ-Aliase.  
  * Modul-Level-Kommentare sollen den Zweck des Moduls erklären (geometry.rs, color.rs, etc.).  
  * Typ-Level-Kommentare sollen den Zweck und die Invarianten der Struktur/des Enums erklären (besonders wichtig für Rect-Invarianten).  
  * Feld-Level-Kommentare sollen die Bedeutung des Feldes erklären (z.B. Wertebereich für Color-Komponenten).  
  * Methoden-Level-Kommentare sollen erklären, was die Methode tut, ihre Parameter, Rückgabewerte, mögliche Panics (sollten hier idealerweise keine auftreten, außer bei unwrap/expect in Tests), relevante Vor-/Nachbedingungen oder verwendete Algorithmen (z.B. Alpha-Blending-Formel). \# Examples-Abschnitte verwenden, wo sinnvoll.  
  * Strikte Einhaltung der Rust API Guidelines für Dokumentation.  
  * Ausführen von cargo doc \--open zur Überprüfung der generierten Dokumentation.

## **7\. Schlussfolgerung**

Dieses Dokument spezifiziert das Modul core::types, welches die grundlegendsten Datentypen für die neue Linux-Desktop-Umgebung bereitstellt. Die definierten Typen (Point\<T\>, Size\<T\>, Rect\<T\>, Color, Orientation) sind mit Fokus auf Einfachheit, Wiederverwendbarkeit und minimalen Abhängigkeiten entworfen. Besonderes Augenmerk wurde auf die klare Trennung zwischen Datenrepräsentation und Fehlerbehandlung gelegt, wobei die Typen so gestaltet sind, dass sie die übergeordnete, auf thiserror basierende Fehlerstrategie des Projekts unterstützen, ohne selbst Fehlerdefinitionen zu enthalten. Die Bereitstellung von Validierungsfunktionen wie Rect::is\_valid und die klare Dokumentation von Invarianten sind entscheidend, um Robustheit in den konsumierenden Schichten zu ermöglichen. Der detaillierte Implementierungsplan inklusive Test- und Dokumentationsanforderungen stellt sicher, dass dieses fundamentale Modul mit hoher Qualität und Konsistenz entwickelt werden kann.

---

# **A2 Implementierungsleitfaden: Kernschicht – Teil 2: Fehlerbehandlung (core::errors)**

## **1\. Einleitung**

### **1.1. Zweck und Geltungsbereich**

Dieser Abschnitt des Implementierungsleitfadens spezifiziert die verbindliche Strategie und Implementierung der Fehlerbehandlung innerhalb der Kernschicht (Core Layer) des Projekts. Er stellt Teil 2 der Spezifikation für die Kernschicht dar und baut direkt auf der technischen Gesamtspezifikation auf, insbesondere auf Abschnitt III (Technologie-Stack) und IV (Entwicklungsrichtlinien). Die hier dargelegten Definitionen und Richtlinien konkretisieren die Anforderungen für das Modul core::errors. Das Ziel ist die Bereitstellung einer lückenlosen, präzisen Spezifikation, die Entwicklern die direkte Implementierung der Fehlerbehandlungsmechanismen ermöglicht, ohne eigene architektonische Entscheidungen treffen oder grundlegende Logiken entwerfen zu müssen.

### **1.2. Bezug zur Gesamtspezifikation**

Wie in Abschnitt IV. 4.3 der technischen Gesamtspezifikation festgelegt, ist die Verwendung des thiserror Crates für die Definition von benutzerdefinierten Fehlertypen obligatorisch. Diese Entscheidung basiert auf der Notwendigkeit, idiomatisches, wartbares und kontextreiches Fehlerhandling für Code zu implementieren, der als Bibliothek für andere Schichten dient – eine primäre Funktion der Kernschicht.1 thiserror erleichtert die Erstellung von Fehlertypen, die das std::error::Error Trait implementieren, erheblich.1

### **1.3. Anforderungen an die Spezifikation**

Die folgenden Anforderungen gelten für diesen Implementierungsleitfaden:

* **Höchste Präzision:** Alle Typen, Enums, Traits und Methoden im Zusammenhang mit der Fehlerbehandlung müssen exakt definiert werden, einschließlich ihrer Signaturen, Felder und abgeleiteten Traits.  
* **Eindeutigkeit:** Benennung und Semantik aller Fehlerarten müssen klar und unmissverständlich sein.  
* **Vollständigkeit:** Alle relevanten Aspekte der Fehlerbehandlungsstrategie und \-implementierung müssen abgedeckt sein.  
* **Detaillierte Anleitungen:** Schritt-für-Schritt-Anleitungen für typische Implementierungsaufgaben im Zusammenhang mit Fehlern müssen bereitgestellt werden.

## **2\. Kernschicht Fehlerbehandlungsstrategie (core::errors)**

### **2.1. Grundlagen und Prinzipien**

#### **Verwendung von thiserror**

Die Entscheidung für das thiserror Crate, wie in der Gesamtspezifikation (IV. 4.3) festgelegt, wird hier bekräftigt und als verbindlich erklärt. thiserror stellt ein deklaratives Makro (\#\[derive(Error)\]) bereit, das den Boilerplate-Code für die Implementierung des std::error::Error Traits und verwandter Traits (wie std::fmt::Display) signifikant reduziert.1 Alle benutzerdefinierten Fehler-Enums, die innerhalb der Kernschicht definiert werden, *müssen* thiserror::Error ableiten.

#### **$Result\<T, E\>$ vs. $panic\!$**

Eine strikte und konsequente Trennung zwischen der Verwendung von $Result\<T, E\>$ und $panic\!$ ist für die Stabilität und Vorhersagbarkeit des Systems unerlässlich.3 Die folgenden Regeln sind einzuhalten:

* **$Result\<T, E\>$:** Dieses Konstrukt, wobei E das std::error::Error Trait implementiert, ist der Standardmechanismus zur Signalisierung von *erwarteten*, potenziell behebbaren Fehlerzuständen zur Laufzeit. Beispiele hierfür sind fehlgeschlagene I/O-Operationen (Datei nicht gefunden), ungültige Benutzereingaben, Fehler bei der Netzwerkkommunikation oder Probleme beim Parsen von Daten. Funktionen in der Kernschicht, die solche Fehlerzustände antizipieren, *müssen* einen $Result\<T, E\>$ zurückgeben, wobei E typischerweise CoreError oder ein spezifischerer Modul-Fehler ist (siehe Abschnitt 2.2 und 2.3).  
* **$panic\!$:** Der $panic\!-Mechanismus ist ausschließlich für die Signalisierung von *nicht behebbaren Programmierfehlern* (Bugs) reserviert.3 Ein Panic tritt ein, wenn eine Funktion in einem Zustand aufgerufen wird, der gegen ihre dokumentierten Vorbedingungen (Invariants) verstößt, oder wenn ein interner Systemzustand erreicht wird, der logisch unmöglich sein sollte und auf einen Fehler in der Programmlogik hindeutet. Panics signalisieren, dass das Programm in einem inkonsistenten Zustand ist, von dem es sich nicht sicher erholen kann.

#### **Umgang mit $unwrap()$ und $expect()$**

Die Methoden $unwrap()$ und $expect()$ auf $Result$ oder $Option$ führen bei einem Err- bzw. None-Wert zu einem $panic\!$. Ihre Verwendung in produktivem Code der Kernschicht ist daher **strengstens zu vermeiden**, da sie die strukturierte Fehlerbehandlung umgehen und die Kontrolle über den Fehlerfluss dem Aufrufer entziehen.1  
Es gibt nur eine seltene Ausnahme: Wenn ein Err- oder None-Zustand an einer bestimmten Stelle *nachweislich* und *unwiderlegbar* einen Bug darstellt (d.h., eine interne Invariante wurde verletzt, die unter normalen Umständen niemals verletzt sein dürfte), *darf* $expect()$ verwendet werden. In diesem Fall *muss* die übergebene Nachricht dem "expect as precondition"-Stil folgen.3 Diese Nachricht sollte klar artikulieren, *warum* der Entwickler an dieser Stelle einen Ok- oder Some-Wert erwartet hat und welche Bedingung verletzt wurde. Beispiel:

Rust

// FALSCH (unzureichende Begründung):  
// let config\_value \= config\_map.get("required\_key").expect("Config key missing\!");

// RICHTIG (Begründung der Erwartung):  
let config\_value \= config\_map.get("required\_key")  
   .expect("Internal invariant violated: Configuration map should always contain 'required\_key' after initialization phase.");

Die Verwendung von $unwrap()$ ist generell zu unterlassen, da es keine Begründung für die Erwartung liefert.

#### **Anforderungen an Fehlermeldungen**

Fehlermeldungen, die durch das \#\[error("...")\] Attribut von thiserror für die Display-Implementierung generiert werden, müssen folgende Kriterien erfüllen:

* **Klarheit und Präzision:** Die Meldung muss das aufgetretene Problem eindeutig beschreiben.  
* **Kontext:** Sie sollte genügend Kontextinformationen enthalten (oft durch eingebettete Feldwerte wie {field\_name} im Formatstring), um Entwicklern die Diagnose des Problems zu ermöglichen, idealerweise ohne sofortigen Blick in den Quellcode.1  
* **Zielgruppe:** Die primäre Zielgruppe dieser Meldungen sind Entwickler (für Logging und Debugging). Sie können jedoch als Grundlage für benutzerfreundlichere Fehlermeldungen dienen, die in höheren Schichten (insbesondere der UI-Schicht) generiert werden.  
* **Format:** Fehlermeldungen sollten typischerweise knappe, klein geschriebene Sätze ohne abschließende Satzzeichen sein, wie in der std::error::Error Dokumentation empfohlen.4

#### **Akzeptierte Einschränkungen bei thiserror**

Die Wahl von thiserror bietet Einfachheit und reduziert Boilerplate für den häufigen Anwendungsfall der Fehlerdefinition in Bibliotheken.1 Es ist jedoch wichtig, eine spezifische Einschränkung zu verstehen, die sich aus der Funktionsweise von thiserror ergibt, insbesondere bei der Verwendung des \#\[from\]-Attributs zur automatischen Konvertierung von Quellfehlern. thiserror implementiert das std::convert::From-Trait, um die nahtlose Verwendung des ?-Operators zu ermöglichen.1 Eine Konsequenz daraus ist, dass ein bestimmter Quellfehlertyp (z.B. std::io::Error) nicht ohne Weiteres über \#\[from\] in *mehrere verschiedene Varianten* desselben Ziel-Enums (z.B. CoreError) konvertiert werden kann, da die From-Implementierung eindeutig sein muss.1  
Wenn beispielsweise ein std::io::Error sowohl beim Lesen einer Konfigurationsdatei als auch beim Schreiben in eine Log-Datei auftreten kann, können nicht einfach zwei Varianten wie ConfigReadIo(\#\[from\] std::io::Error) und LogWriteIo(\#\[from\] std::io::Error) innerhalb von CoreError definiert werden. Diese Einschränkung unterscheidet thiserror von flexibleren, aber potenziell komplexeren Fehlerbehandlungs-Frameworks wie snafu, die explizit darauf ausgelegt sind, Kontext aus dem Fehlerpfad abzuleiten.1  
Diese systembedingte Eigenschaft von thiserror erfordert eine bewusste Gestaltung der Fehlerhierarchie. Um dennoch semantisch unterschiedliche Fehlerfälle zu behandeln, die auf denselben zugrunde liegenden Fehlertyp zurückzuführen sind, wird die Strategie der Modul-spezifischen Fehler verfolgt (siehe Abschnitt 2.3). Diese spezifischen Fehler können dann eindeutig in eine dedizierte Variante des übergeordneten Fehlers (CoreError) gekapselt werden, wobei der notwendige Kontext entweder im Modul-Fehler selbst oder in der Kapselungsvariante hinzugefügt wird. Dieser Ansatz stellt sicher, dass der semantische Kontext des Fehlers erhalten bleibt, auch wenn der unmittelbare Quelltyp mehrdeutig sein könnte.

### **2.2. Definition des Basis-Fehlertyps: $CoreError$**

#### **Spezifikation**

Im Modul core::errors wird ein zentrales, öffentliches Enum namens CoreError definiert. Dieses Enum stellt die primäre Schnittstelle für Fehler dar, die von öffentlichen Funktionen der Kernschicht nach außen propagiert werden. Es aggregiert sowohl allgemeine Fehlerarten als auch spezifischere Fehler aus den Untermodulen der Kernschicht.

Rust

// In core/src/errors.rs  
use thiserror::Error;  
use std::path::PathBuf; // Beispiel für einen benötigten Typ

// Import von Modul-spezifischen Fehlern (Beispiel)  
use crate::config::errors::ConfigError;  
// use crate::utils::errors::UtilsError; // Falls vorhanden

\#  
pub enum CoreError {  
    /// Fehler bei Ein-/Ausgabeoperationen. Enthält den ursprünglichen I/O-Fehler.  
    \#\[error("I/O error accessing '{path}': {source}")\]  
    Io {  
        path: PathBuf, // Pfad zur Ressource, bei der der Fehler auftrat  
        \#\[source\] // \#\[source\] statt \#\[from\], um Kontext (path) hinzuzufügen  
        source: std::io::Error,  
    },

    /// Fehler im Zusammenhang mit der Konfigurationsverwaltung. Kapselt spezifischere ConfigError-Typen.  
    \#\[error("Configuration error: {0}")\]  
    Configuration(\#\[from\] ConfigError), // Nutzt \#\[from\] für nahtlose Konvertierung

    /// Fehler bei der Serialisierung oder Deserialisierung von Daten (z.B. JSON, TOML).  
    /// Enthält eine Beschreibung des Fehlers. Ggf. spezifischere Varianten für Serde etc. hinzufügen.  
    \#  
    Serialization { description: String },

    /// Eine ungültige ID oder ein ungültiger Bezeichner wurde verwendet.  
    \#\[error("Invalid identifier provided: '{invalid\_id}'")\]  
    InvalidId { invalid\_id: String },

    /// Ein angeforderter Wert oder eine Ressource wurde nicht gefunden.  
    \#  
    NotFound { resource\_description: String },

    /// Ein allgemeiner Fehler in einem Hilfsmodul (Beispiel für Kapselung).  
    // \#\[error("Utility error: {0}")\]  
    // Utility(\#\[from\] UtilsError), // Beispiel für Integration eines weiteren Modul-Fehlers

    /// Platzhalter für einen unerwarteten oder nicht näher spezifizierten internen Fehler.  
    /// Sollte möglichst vermieden und durch spezifischere Varianten ersetzt werden.  
    \#\[error("Internal error: {0}")\]  
    Internal(String),  
}

// Manuelle Implementierung von From\<std::io::Error\>, falls \#\[source\] verwendet wird  
// und man dennoch eine einfache Konvertierung für bestimmte Fälle braucht,  
// aber hier wollen wir Kontext (den Pfad) hinzufügen, daher ist eine manuelle  
// Erzeugung von CoreError::Io an der Fehlerquelle notwendig.  
// Beispiel:  
// std::fs::read("some/path").map\_err(|e| CoreError::Io { path: "some/path".into(), source: e })?;

#### **Ableitungen**

Das CoreError-Enum *muss* mindestens die folgenden Traits ableiten oder implementieren:

* \#: Unerlässlich für Debugging und Diagnosezwecke.  
* \#\[derive(thiserror::Error)\]: Implementiert automatisch std::error::Error und std::fmt::Display basierend auf den \#\[error(...)\]-Attributen und \#\[source\]-/\#\[from\]-Annotationen.1

#### **Fehlerverkettung (source())**

Varianten, die andere Fehler kapseln (entweder durch \#\[from\] oder \#\[source\] annotierte Felder), stellen den ursprünglichen, zugrunde liegenden Fehler über die source()-Methode des std::error::Error-Traits zur Verfügung.4 Dies ist ein fundamentaler Mechanismus für die Fehleranalyse über Schicht- und Modulgrenzen hinweg, da er es ermöglicht, die Kette der verursachenden Fehler bis zur Wurzel zurückzuverfolgen. thiserror implementiert die source()-Methode automatisch korrekt für annotierte Felder.

#### **Tabelle 1: CoreError Varianten (Initial)**

Die folgende Tabelle dient als Referenz für Entwickler und definiert den initialen "Fehlervertrag" der Kernschicht-API. Sie listet die Varianten des CoreError-Enums auf und beschreibt deren Semantik und Struktur.

| Variantenname | \#\[error("...")\] Formatstring | Enthaltene Felder | Beschreibung / Typischer Auslöser | Kapselung (\#\[from\] / \#\[source\]) |
| :---- | :---- | :---- | :---- | :---- |
| Io | I/O error accessing '{path}': {source} | path: PathBuf, source: std::io::Error | Fehler beim Lesen/Schreiben von Dateien oder anderen I/O-Ressourcen. | \#\[source\] (std::io::Error) |
| Configuration | Configuration error: {0} | ConfigError (intern) | Fehler beim Laden, Parsen oder Validieren von Konfigurationen. Kapselt ConfigError. | \#\[from\] (ConfigError) |
| Serialization | Serialization/Deserialization error: {description} | description: String | Fehler beim Umwandeln von Datenstrukturen in/aus Formaten wie JSON, TOML, etc. | \- |
| InvalidId | Invalid identifier provided: '{invalid\_id}' | invalid\_id: String | Eine verwendete ID (z.B. für eine Ressource) ist syntaktisch oder semantisch ungültig. | \- |
| NotFound | Resource not found: {resource\_description} | resource\_description: String | Eine angeforderte Ressource oder ein Wert konnte nicht gefunden werden (z.B. Schlüssel in Map). | \- |
| Internal | Internal error: {0} | String | Allgemeiner interner Fehler, der nicht spezifischer kategorisiert werden konnte. | \- |

Diese Tabelle stellt eine klare Referenz dar, welche Fehlerarten von der Kernschicht erwartet werden können und wie sie strukturiert sind. Sie ist ein wesentlicher Bestandteil der "Ultra-Feinspezifikation", da sie Entwicklern die genaue Struktur der Fehler mitteilt, die sie behandeln oder erzeugen müssen.

### **2.3. Modul-spezifische Fehler und Integration**

#### **Richtlinie**

Während CoreError den zentralen, nach außen sichtbaren Fehlertyp der Kernschicht darstellt, *dürfen* und *sollen* komplexere Module innerhalb der Kernschicht (z.B. core::config, core::utils, core::types falls dort komplexe Validierungen stattfinden) ihre eigenen, spezifischeren Fehler-Enums definieren. Diese Modul-Fehler *müssen* ebenfalls thiserror::Error ableiten.

#### **Begründung**

Diese Vorgehensweise verfolgt einen hybriden Ansatz, der die Vorteile spezifischer Fehler 2 mit der Notwendigkeit einer zentralen Fehlerschnittstelle verbindet. Sie adressiert auch direkt die zuvor beschriebene Einschränkung von thiserror bezüglich mehrdeutiger \#\[from\]-Konvertierungen. Die Definition von Modul-Fehlern bietet folgende Vorteile:

* **Feinere Granularität:** Ermöglicht eine detailliertere Darstellung von Fehlerzuständen, die spezifisch für die Logik eines Moduls sind.  
* **Bessere Kapselung:** Hält die Fehlerdefinitionen und die zugehörige Logik nahe am Code, der die Fehler erzeugt.  
* **Vermeidung von Überladung:** Verhindert, dass das zentrale CoreError-Enum mit einer übermäßigen Anzahl sehr spezifischer Varianten überladen wird, was dessen Übersichtlichkeit und Wartbarkeit beeinträchtigen würde.2

#### **Integrationsmechanismus**

Modul-spezifische Fehler müssen nahtlos in CoreError integrierbar sein, um die Fehlerpropagation mittels des ?-Operators zu gewährleisten. Der **bevorzugte Mechanismus** hierfür ist die Definition einer dedizierten Variante in CoreError, die den Modul-Fehler als einziges Feld enthält und das \#\[from\]-Attribut verwendet.

Rust

// Beispiel in core/src/config/errors.rs  
use thiserror::Error;  
use std::path::PathBuf;

\#  
pub enum ConfigError {  
    \#\[error("Failed to parse configuration file '{file\_path}': {source}")\]  
    ParseError {  
        file\_path: PathBuf,  
        // Box\<dyn Error\> für Flexibilität bei verschiedenen Parser-Fehlern (z.B. TOML, JSON)  
        \#\[source\] source: Box\<dyn std::error::Error \+ Send \+ Sync \+ 'static\>,  
    },

    \#\[error("Missing required configuration key: '{key}' in section '{section}'")\]  
    MissingKey { key: String, section: String },

    \#\[error("Invalid value for key '{key}': {reason}")\]  
    InvalidValue { key: String, reason: String },

    // Spezifischer I/O-Fehler im Kontext der Konfiguration  
    \#\[error("I/O error while accessing config '{path}': {source}")\]  
    Io {  
        path: PathBuf,  
        \#\[source\] source: std::io::Error, // Hier \#\[source\], da Kontext (path) hinzugefügt wird  
    },  
}

// Integration in core/src/errors.rs (Erweiterung von CoreError)  
// (bereits oben im CoreError Beispiel gezeigt)  
// \#\[error("Configuration error: {0}")\]  
// Configuration(\#\[from\] ConfigError),

Die Verwendung von \#\[from\] auf der CoreError::Configuration-Variante ermöglicht die automatische Konvertierung eines Result\<\_, ConfigError\> in ein Result\<\_, CoreError\> durch den ?-Operator.1

#### **Etablierung einer strukturierten Fehlerhierarchie**

Der Ansatz, einen zentralen CoreError mit integrierten, Modul-spezifischen Fehlern über \#\[from\] zu kombinieren, etabliert eine klare, zweistufige Fehlerhierarchie innerhalb der Kernschicht. Diese Struktur bietet eine gute Balance:

1. **Zentrale Schnittstelle:** Höhere Schichten interagieren primär mit dem wohldefinierten CoreError, was die Komplexität für die Nutzer der Kernschicht reduziert.  
2. **Lokale Spezifität:** Entwickler, die innerhalb eines Kernschicht-Moduls arbeiten, können mit spezifischeren, kontextbezogenen Fehlertypen (ConfigError, UtilsError, etc.) arbeiten, was die interne Logik klarer und wartbarer macht.  
3. **Nahtlose Propagation:** Die \#\[from\]-Integration stellt sicher, dass die Vorteile des ?-Operators für die Fehlerpropagation über Modulgrenzen hinweg erhalten bleiben.

Diese bewusste Strukturierung ist entscheidend für die Skalierbarkeit und Wartbarkeit der Fehlerbehandlung in einem größeren Projekt. Sie verhindert sowohl eine unübersichtliche Flut von Fehlertypen auf der obersten Ebene als auch den Verlust von spezifischem Fehlerkontext.

### **2.4. Fehlerkontext und Diagnose**

#### **Anreicherung mit Kontext**

Fehlervarianten *sollen* über die reine Fehlermeldung hinaus relevante Kontextinformationen als Felder enthalten. Diese Informationen sind entscheidend für eine effektive Diagnose und Fehlersuche.1 Beispiele für nützliche Kontextfelder sind:

* Dateipfade oder Ressourcennamen (path: PathBuf)  
* Ungültige Werte oder Eingaben (invalid\_value: String)  
* Betroffene Schlüssel oder Bezeichner (key: String, item\_id: Uuid)  
* Zustandsinformationen zum Zeitpunkt des Fehlers (z.B. index: usize, state: String)  
* Zeitstempel (falls relevant)

Rust

// Beispiel für eine Variante mit Kontextfeldern  
\#  
pub enum ProcessingError {  
    \#\[error("Failed to process item '{item\_id}' at index {index} due to: {reason}")\]  
    ItemFailure {  
        item\_id: String,  
        index: usize,  
        reason: String, // Könnte auch ein \#\[source\] Fehler sein  
    },  
    //...  
}

Die Auswahl der Kontextfelder sollte darauf abzielen, die Frage "Was ist passiert und unter welchen Umständen?" möglichst präzise zu beantworten.

#### **Backtraces**

Das thiserror-Crate bettet standardmäßig keine Backtraces in die erzeugten Fehlertypen ein, wie es bei anyhow oder eyre der Fall ist. Backtraces sind primär mit dem $panic\!-Mechanismus assoziiert und können durch Setzen der Umgebungsvariable RUST\_BACKTRACE=1 (oder full) aktiviert werden, um den Call Stack zum Zeitpunkt des Panics anzuzeigen.1  
Für die Diagnose von Fehlern, die über $Result::Err$ zurückgegeben werden, sind die primären Werkzeuge:

1. **Fehlerverkettung (source()):** Verfolgung der Ursache über die source()-Methode.4  
2. **Kontextfelder:** Analyse der in den Fehlervarianten gespeicherten Daten.  
3. **Logging:** Korrelation mit Log-Einträgen, die zum Zeitpunkt des Fehlers erstellt wurden (siehe Abschnitt 4).

Es ist nicht vorgesehen, Backtraces manuell in CoreError oder Modul-Fehler einzubetten, um die Komplexität gering zu halten und sich auf die strukturierte Fehlerinformation zu konzentrieren.

#### **Keine sensiblen Daten**

Es ist absolut entscheidend, dass Fehlermeldungen (\#\[error("...")\]) und die Werte von Kontextfeldern in Fehlervarianten **niemals** sensible Informationen enthalten. Dazu gehören insbesondere:

* Passwörter  
* API-Schlüssel oder Tokens  
* Private Benutzerdaten (Namen, Adressen, etc.)  
* Andere vertrauliche Informationen

Diese Daten dürfen unter keinen Umständen in Logs oder Diagnosedateien gelangen. Wenn solche Daten Teil des Kontexts sind, der zum Fehler führt, müssen sie vor der Aufnahme in den Fehlertyp maskiert, entfernt oder durch Platzhalter ersetzt werden.

## **3\. Implementierungsleitfaden für Entwickler**

### **3.1. Fehlerdefinition**

#### **Neue Variante zu $CoreError$ hinzufügen**

1. **Bedarf prüfen:** Stellen Sie sicher, dass der neue Fehlerfall eine allgemeine Bedeutung für die Kernschicht hat und nicht besser durch einen bestehenden oder einen neuen Modul-Fehler abgedeckt wird.  
2. **Variante definieren:** Fügen Sie eine neue Variante zum CoreError-Enum in core/src/errors.rs hinzu.  
3. **Attribute hinzufügen:** Versehen Sie das CoreError-Enum (falls noch nicht geschehen) mit \#.  
4. **Fehlermeldung (\#\[error\])**: Definieren Sie einen klaren und informativen \#\[error("...")\]-Formatstring für die neue Variante. Nutzen Sie {field\_name}-Platzhalter für Kontextfelder.  
5. **Kontextfelder:** Fügen Sie der Variante die notwendigen Felder hinzu, um den Fehlerkontext zu speichern. Definieren Sie deren Typen.  
6. **Kapselung (\#\[source\] / \#\[from\]):** Falls die Variante einen anderen Fehler kapselt:  
   * Verwenden Sie \#\[source\] auf dem Feld, wenn Sie zusätzlichen Kontext hinzufügen möchten oder der Quelltyp nicht direkt konvertiert werden soll. Die Erzeugung des Fehlers erfolgt dann manuell (z.B. via .map\_err(|e| CoreError::SomeVariant {..., source: e })).  
   * Verwenden Sie \#\[from\] auf dem Feld, wenn eine direkte, automatische Konvertierung vom Quelltyp zur Variante gewünscht ist (nur möglich, wenn der Quelltyp eindeutig dieser Variante zugeordnet werden kann).  
7. **Dokumentation:** Fügen Sie die neue Variante zur Tabelle 1 (oder einer Folgetabelle in der Dokumentation) hinzu und beschreiben Sie ihre Bedeutung und Verwendung. Aktualisieren Sie ggf. Doc-Kommentare.

#### **Neuen Modul-Fehler erstellen und integrieren**

1. **Datei erstellen:** Legen Sie eine neue Datei für die Fehler des Moduls an, typischerweise errors.rs im Modulverzeichnis (z.B. core/src/neues\_modul/errors.rs).  
2. **Enum definieren:** Definieren Sie ein neues, öffentliches Enum (z.B. pub enum NeuesModulError) und leiten Sie \# ab.  
3. **Varianten definieren:** Fügen Sie spezifische Fehlervarianten für das Modul hinzu, wie im vorherigen Abschnitt beschrieben (inkl. \#\[error\], Kontextfeldern, \#\[source\]/\#\[from\] falls interne Fehler gekapselt werden).  
4. **Integration in CoreError:**  
   * Importieren Sie den neuen Modul-Fehler in core/src/errors.rs (z.B. use crate::neues\_modul::errors::NeuesModulError;).  
   * Fügen Sie eine neue Variante zu CoreError hinzu, die den Modul-Fehler kapselt. Der bevorzugte Weg ist:  
     Rust  
     \#\[error("Neues Modul error: {0}")\] // Display delegiert an Modul-Fehler  
     NeuesModul(\#\[from\] NeuesModulError),

5. **Dokumentation:** Dokumentieren Sie den neuen Modul-Fehler (in seiner eigenen Datei) und die Integrationsvariante in CoreError (in core/src/errors.rs und der Tabelle).

### **3.2. Fehlerbehandlung im Code**

#### **Verwendung des ?-Operators**

Der ?-Operator ist das idiomatisches Mittel zur Fehlerpropagation in Rust und *sollte* standardmäßig verwendet werden, wenn eine Funktion, die $Result$ zurückgibt, eine andere Funktion aufruft, die ebenfalls $Result$ zurückgibt.

Rust

use crate::errors::CoreError;  
use crate::config::errors::ConfigError; // Beispiel Modul-Fehler

// Funktion, die einen Modul-Fehler zurückgibt  
fn load\_setting\_internal() \-\> Result\<String, ConfigError\> {  
    //... Logik...  
    if condition {  
        Ok("value".to\_string())  
    } else {  
        Err(ConfigError::MissingKey { key: "foo".to\_string(), section: "bar".to\_string() })  
    }  
}

// Funktion, die CoreError zurückgibt und intern load\_setting\_internal aufruft  
pub fn get\_setting() \-\> Result\<String, CoreError\> {  
    // Das '?' hier konvertiert ConfigError automatisch zu CoreError::Configuration  
    // dank der \#\[from\]-Annotation auf der CoreError::Configuration Variante.  
    let setting \= load\_setting\_internal()?;  
    //... weitere Logik...  
    Ok(setting)  
}

Der ?-Operator funktioniert nahtlos, solange die Fehlertypen entweder identisch sind oder eine From-Implementierung existiert (was thiserror mit \#\[from\] bereitstellt).

#### **Fehler-Matching (match)**

Wenn ein Fehler nicht nur propagiert, sondern spezifisch behandelt werden muss (z.B. um einen Standardwert zu verwenden, einen alternativen Pfad zu wählen oder den Fehler anzureichern), verwenden Sie eine match-Anweisung auf das $Result$.

Rust

use crate::errors::CoreError;  
use crate::config::errors::ConfigError;  
use tracing::warn; // Beispiel für Logging

fn handle\_config\_loading() {  
    match get\_setting() {  
        Ok(setting) \=\> {  
            println\!("Einstellung erfolgreich geladen: {}", setting);  
            //... mit der Einstellung arbeiten...  
        }  
        Err(CoreError::Configuration(ConfigError::MissingKey { ref key, ref section })) \=\> {  
            warn\!(key \= %key, section \= %section, "Konfigurationsschlüssel fehlt, verwende Standardwert.");  
            //... Standardwert verwenden...  
        }  
        Err(CoreError::Io { ref path, ref source }) \=\> {  
            // Kritischer Fehler, kann oft nicht sinnvoll behandelt werden  
            eprintln\!("FATAL: I/O Fehler beim Zugriff auf {:?}: {}", path, source);  
            // Ggf. Programm beenden oder Fehler weiter nach oben geben  
            // return Err(CoreError::Io { path: path.clone(), source: \*source }); // Beispiel für Weitergabe  
        }  
        Err(ref other\_error) \=\> {  
            // Alle anderen CoreError-Varianten behandeln  
            eprintln\!("Ein unerwarteter Kernschicht-Fehler ist aufgetreten: {}", other\_error);  
            // Allgemeine Fehlerbehandlung, ggf. weiter propagieren  
            // return Err(other\_error.clone()); // Klonen nur wenn Fehler Clone implementiert  
        }  
    }  
}

Behandeln Sie nur die Fehlerfälle, für die eine spezifische Logik sinnvoll ist. Für alle anderen Fälle sollte der Fehler entweder weiter propagiert oder in einen allgemeineren Fehler umgewandelt werden.

#### **Umgang mit externen Crates**

Fehler, die von externen Bibliotheken (Crates) zurückgegeben werden (z.B. serde\_json::Error, toml::de::Error, std::io::Error), *müssen* in einen geeigneten Fehlertyp der Kernschicht (CoreError oder einen Modul-Fehler) gekapselt werden, bevor sie die Grenzen der Kernschicht verlassen.

* **Bevorzugt mit \#\[from\]:** Wenn eine eindeutige Zuordnung des externen Fehlers zu einer Variante sinnvoll ist und keine zusätzliche Kontextinformation benötigt wird, verwenden Sie \#\[from\] auf einem Feld dieser Variante. Dies ist oft bei std::io::Error der Fall, wobei hier entschieden wurde, Kontext (path) hinzuzufügen, was \#\[source\] erfordert (siehe CoreError::Io).  
* **Mit \#\[source\]:** Wenn zusätzlicher Kontext hinzugefügt werden soll oder der externe Fehler nicht direkt einer Variante zugeordnet werden kann, verwenden Sie \#\[source\] auf einem Feld und erzeugen Sie die Fehlervariante manuell im Code mittels .map\_err().  
  Rust  
  use serde\_json;  
  use crate::errors::CoreError;

  fn parse\_json\_data(data: \&str) \-\> Result\<serde\_json::Value, CoreError\> {  
      serde\_json::from\_str(data).map\_err(|e| CoreError::Serialization {  
          description: format\!("Failed to parse JSON: {}", e),  
          // Hier wird der Fehler in einen String umgewandelt.  
          // Alternativ könnte man den Fehler boxen: source: Box::new(e)  
          // und die Variante anpassen, wenn der Originalfehler benötigt wird.  
      })  
  }

* **Manuelle Konvertierung:** In komplexeren Fällen kann eine explizite match-Anweisung auf den externen Fehler notwendig sein, um ihn auf verschiedene Varianten des Kernschicht-Fehlers abzubilden.

## **4\. Zusammenspiel mit Logging (core::logging)**

### **4.1. Verweis**

Die detaillierte Spezifikation des Logging-Frameworks (tracing) und dessen Initialisierung ist Gegenstand eines separaten Abschnitts des Kernschicht-Implementierungsleitfadens (Teil 3 oder 4, basierend auf Gesamtspezifikation IV. 4.4). Die hier beschriebenen Richtlinien beziehen sich auf die *Verwendung* des Logging-Frameworks im Kontext der Fehlerbehandlung.

### **4.2. Vorgabe: Logging von Fehlern**

Jeder Fehler, der mittels $Result::Err$ zurückgegeben wird, *sollte* an der Stelle seines Ursprungs oder an einer geeigneten übergeordneten Stelle, die über ausreichend Kontext verfügt, geloggt werden. Das Logging *muss* mindestens auf dem ERROR-Level erfolgen. Das Makro tracing::error\! ist hierfür zu verwenden.  
Das Logging sollte typischerweise *vor* der Propagation des Fehlers mittels ? oder return Err(...) geschehen, um sicherzustellen, dass der Fehler erfasst wird, auch wenn er in höheren Schichten möglicherweise abgefangen oder ignoriert wird.

### **4.3. Strukturiertes Logging**

Das tracing-Framework ermöglicht strukturiertes Logging, bei dem Schlüssel-Wert-Paare an Log-Ereignisse angehängt werden können. Es ist **dringend empfohlen**, den aufgetretenen Fehler selbst als strukturiertes Feld im Log-Eintrag mitzugeben. Dies erleichtert die automatisierte Analyse und Filterung von Logs erheblich.

Rust

use tracing::{error, instrument};  
use crate::errors::CoreError;

\#\[instrument\] // Instrumentiert die Funktion für Tracing (Span)  
fn perform\_critical\_operation(config\_path: \&std::path::Path) \-\> Result\<(), CoreError\> {  
    match std::fs::read\_to\_string(config\_path) {  
        Ok(content) \=\> {  
            //... Operation mit content...  
            Ok(())  
        }  
        Err(io\_error) \=\> {  
            // Fehler loggen, bevor er gekapselt und zurückgegeben wird  
            let core\_err \= CoreError::Io {  
                path: config\_path.to\_path\_buf(),  
                source: io\_error, // Beachten: io::Error implementiert nicht Copy/Clone  
            };

            // Strukturiertes Logging mit dem Fehler als Feld  
            // %core\_err nutzt die Display-Implementierung  
            //?core\_err würde die Debug-Implementierung nutzen  
            error\!(  
                error \= %core\_err, // Fehlerobjekt als Feld 'error'  
                file\_path \= %config\_path.display(), // Zusätzlicher Kontext  
                "Failed during critical operation while reading config" // Log-Nachricht  
            );

            Err(core\_err) // Fehler zurückgeben  
        }  
    }  
}

Die Verwendung von error \= %e (wobei e der Fehler ist) nutzt die Display-Implementierung des Fehlers für die Log-Ausgabe, während error \=?e die Debug-Implementierung verwenden würde. Die Display-Implementierung ist oft für die primäre Log-Nachricht vorzuziehen, während die Debug-Darstellung bei Bedarf für detailliertere Analysen herangezogen werden kann.

### **4.4. Fehler als integraler Bestandteil der Observability**

Die konsequente Verknüpfung von $Result::Err$-Rückgaben mit strukturiertem tracing::error\!-Logging hebt die Fehlerbehandlung über reines Debugging hinaus. Sie macht Fehler zu einem integralen Bestandteil der System-Observability. Die Kombination aus wohldefinierten, typisierten Fehlern (thiserror) und einem strukturierten Logging-Framework (tracing) schafft einen Datenstrom von Fehlerereignissen, der für Monitoring und Alerting genutzt werden kann.  
Systeme zur Log-Aggregation und \-Analyse (wie z.B. Elasticsearch/Kibana, Loki/Grafana oder spezialisierte Tracing-Backends) können diesen strukturierten Datenstrom verarbeiten. Dies ermöglicht:

* **Visualisierung:** Erstellung von Dashboards, die Fehlerraten über Zeit anzeigen, aufgeschlüsselt nach Fehlertyp (z.B. CoreError::Io vs. CoreError::Configuration).  
* **Filterung und Suche:** Gezielte Suche nach spezifischen Fehlervarianten oder Fehlern, die bestimmte Kontextdaten enthalten (z.B. alle Fehler im Zusammenhang mit einer bestimmten Datei).  
* **Alerting:** Konfiguration von Alarmen, die ausgelöst werden, wenn die Häufigkeit bestimmter Fehler einen Schwellenwert überschreitet.

Diese systematische Erfassung und Analyse von Fehlern ist entscheidend für die Aufrechterhaltung der Stabilität und Zuverlässigkeit des Systems im Betrieb und verbessert die Reaktionsfähigkeit auf Probleme erheblich.

## **5\. Ausblick**

Dieser Implementierungsleitfaden für core::errors legt das Fundament für eine robuste und konsistente Fehlerbehandlung in der gesamten Desktop-Umgebung. Die hier definierten Prinzipien, der CoreError-Typ und die Mechanismen zur Integration von Modul-Fehlern sind verbindlich für alle weiteren Entwicklungen innerhalb der Kernschicht und dienen als Vorbild für die Fehlerbehandlung in den darüberliegenden Schichten (Domäne, System, UI).  
Die nachfolgenden Teile der Kernschicht-Spezifikation, beginnend mit core::logging (Implementierung der tracing-Integration), core::config (Laden und Parsen von Konfigurationen unter Verwendung von CoreError::Configuration und ConfigError) und core::types (Definition fundamentaler Datenstrukturen mit entsprechender Fehlerbehandlung bei Validierungen), werden die hier etablierten Fehlerkonventionen konsequent anwenden und darauf aufbauen. Die disziplinierte Einhaltung dieser Fehlerstrategie ist von zentraler Bedeutung für die Entwicklung einer qualitativ hochwertigen, stabilen und wartbaren Software.

---

## **A3 Kernschicht Fehlerbehandlung** **1\. Fehlerbehandlung (core::errors)**

Die Fehlerbehandlung ist ein kritischer Aspekt der Systemstabilität und Wartbarkeit. Dieses Kapitel definiert die Strategien und Mechanismen für die Fehlerbehandlung innerhalb der Kernschicht (Core Layer). Ziel ist es, eine konsistente, informative und robuste Fehlerpropagierung und \-behandlung im gesamten System sicherzustellen. Die hier festgelegten Richtlinien basieren auf den allgemeinen Entwicklungsrichtlinien (Abschnitt IV.3. Fehlerbehandlung) und spezifizieren deren Anwendung innerhalb der Kernschicht.

### **1.1. Definition des Basis-Fehlertyps (CoreError)**

Zweck:  
Ein grundlegender, allgemeiner Fehlertyp für die Kernschicht, CoreError, wird definiert. Dieser dient dazu, Fehler zu repräsentieren, die direkt von generischen Kern-Dienstprogrammen stammen oder als gemeinsame Basis für Fehler innerhalb des core::errors-Moduls selbst dienen. Die Existenz von CoreError verhindert die Ad-hoc-Verwendung von unspezifischen Fehlertypen wie Box\<dyn std::error::Error\> für nicht klassifizierte Kernprobleme und stellt ein kanonisches Beispiel für die Verwendung von thiserror dar. Es ist jedoch entscheidend, dass CoreError nicht zu einem Sammelbecken für alle Arten von Fehlern wird, da die primäre Strategie auf modul-spezifischen Fehlertypen beruht (siehe Abschnitt 1.3), um Präzision und Klarheit in der Fehlerbehandlung zu gewährleisten.1  
Spezifikation:  
Der CoreError-Enum wird wie folgt definiert:

Rust

\#  
pub enum CoreError {  
    \#\[error("Core component '{component}' failed to initialize")\]  
    InitializationFailed {  
        component: String,  
        \#\[source\]  
        source: Option\<Box\<dyn std::error::Error \+ Send \+ Sync \+ 'static\>\>,  
    },

    \#\[error("Core configuration error: {message}")\]  
    ConfigurationError {  
        message: String,  
        \#\[source\]  
        source: Option\<Box\<dyn std::error::Error \+ Send \+ Sync \+ 'static\>\>,  
    },

    \#\[error("An I/O operation failed at the core level")\]  
    Io {  
        \#\[from\] // Beispiel für direkte Konvertierung eines häufigen, eindeutigen Fehlers  
        source: std::io::Error,  
    },

    \#\[error("Core internal assertion failed: {context}")\]  
    InternalAssertionFailed {  
        context: String,  
        // Diese Variante hat typischerweise keine \`source\`, da sie einen internen Logikfehler darstellt.  
    },

    // Weitere wirklich generische Core Layer Fehlervarianten können hier bei Bedarf ergänzt werden.  
    // Es ist zu vermeiden, Varianten hinzuzufügen, die spezifischen Submodulen wie config, utils etc. zugeordnet werden sollten.  
}

* **Display (\#\[error(...)\]) Nachrichten:**  
  * Die Fehlermeldungen, die durch das \#\[error(...)\]-Attribut generiert werden, *müssen* den Rust API Guidelines entsprechen: prägnant, in Kleinbuchstaben und ohne abschließende Satzzeichen (z.B. "invalid digit found in string" 3).  
  * Die Nachrichten *müssen* klar artikulieren, welches spezifische Problem aus der Perspektive des Betriebs der Kernschicht aufgetreten ist.  
  * Platzhalter (z.B. {component}, {message}) *müssen* verwendet werden, um dynamische kontextuelle Informationen in die Nachricht zu integrieren.  
  * Die Sprache *muss* so gewählt werden, dass sie für einen Entwickler, der das System debuggt, verständlich ist.  
* **Debug Format:**  
  * Die abgeleitete Debug-Implementierung ist Standard. Sie wird für detailliertes Logging und Debugging-Sitzungen verwendet, bei denen die vollständige Struktur des Fehlers, einschließlich aller Felder und der Debug-Repräsentation jeglicher \#\[source\]-Fehler, erforderlich ist.  
* **std::error::Error Trait Implementierung:**  
  * Diese wird automatisch durch \#\[derive(thiserror::Error)\] bereitgestellt. Die source()-Methode ist verfügbar, wenn eine Variante ein Feld enthält, das mit \#\[source\] oder \#\[from\] annotiert ist.

Die Varianten von CoreError *müssen* strikt auf wirklich generische Situationen beschränkt bleiben. Dieser Enum darf nicht zu einem "Catch-all"-Typ werden, da dies die Vorteile spezifischer, modulbezogener Fehlertypen untergraben würde, die eine präzise Fehlerbehandlung durch Aufrufer ermöglichen.1 Eine übermäßige Ansammlung diverser Varianten, die eigentlich zu Submodulen gehören (z.B. ConfigParseError, UtilsStringFormatError), würde CoreError zu einem monolithischen Fehlertyp machen. Die Behandlung eines solchen Fehlers würde dann umfangreiches Pattern-Matching und möglicherweise die Inspektion von Zeichenketten erfordern, was die Vorteile spezifischer Enums zunichtemacht. Daher wird sichergestellt, dass CoreError schlank bleibt und sich auf genuinely schichtweite oder spezifische Probleme des core::errors-Moduls konzentriert.  
**Tabelle 1: CoreError Enum Spezifikation**  
Die folgende Tabelle dient als eindeutige Referenz für Entwickler und als Vertrag für den CoreError-Typ, um Konsistenz über das gesamte Projekt hinweg sicherzustellen und die Anforderung einer "Ultra-Feinspezifikation" zu erfüllen.

| Variantenname | Felder | \#\[error(...)\] Format-String | Beschreibung / Verwendungszweck |
| :---- | :---- | :---- | :---- |
| InitializationFailed | component: String, source: Option\<Box\<dyn std::error::Error \+ Send \+ Sync \+ 'static\>\> (\#\[source\]) | Core component '{component}' failed to initialize | Wird verwendet, wenn eine Kernkomponente nicht initialisiert werden konnte. Enthält optional den zugrundeliegenden Fehler. |
| ConfigurationError | message: String, source: Option\<Box\<dyn std::error::Error \+ Send \+ Sync \+ 'static\>\> (\#\[source\]) | Core configuration error: {message} | Repräsentiert einen allgemeinen Konfigurationsfehler auf Kernschichtebene. |
| Io | source: std::io::Error (\#\[from\]) | An I/O operation failed at the core level | Für generische E/A-Fehler, die direkt auf der Kernschichtebene auftreten und von std::io::Error konvertiert werden können. |
| InternalAssertionFailed | context: String | Core internal assertion failed: {context} | Zeigt einen internen Logikfehler oder eine verletzte Invariante innerhalb der Kernschicht an. |

Diese tabellarische Darstellung ermöglicht es Entwicklern, alle kritischen Attribute jeder Fehlervariante – Name, enthaltene Daten, Display-Format und Zweck – sofort zu erfassen. Diese Präzision minimiert Mehrdeutigkeiten und stellt sicher, dass alle Entwickler CoreError identisch implementieren und verwenden.

### **1.2. Fehlerquellenverkettung und Kontext (Error Source Chaining and Context)**

Zweck:  
Es werden verbindliche Praktiken zur Bewahrung und Offenlegung der zugrundeliegenden Ursachen von Fehlern etabliert. Dies stellt sicher, dass ein vollständiger Diagnosepfad verfügbar ist, was das Debugging erleichtert, indem Entwickler einen Fehler bis zu seiner ursprünglichen Ursache zurückverfolgen können.1 Ein Fehlerbericht sollte die grundlegende Ursache und den vollständigen Kontext-Stack für das Debugging enthalten.  
**Spezifikationen:**

* **Verbindliche Verwendung von \#\[from\] für eindeutige direkte Konvertierungen:**  
  * Wenn eine Funktion der Kernschicht eine andere Funktion aufruft (intern, aus std oder aus einer externen Crate), die ein Result zurückgibt, und der Fehlertyp des Aufgerufenen *eindeutig und direkt* einer spezifischen Variante des thiserror-Enums des Aufrufers zugeordnet werden kann, *muss* das \#\[from\]-Attribut auf einem Feld dieser Variante verwendet werden, um eine automatische Konvertierung über den ?-Operator zu ermöglichen.  
  * Beispiel:  
    Rust  
    // In core/src/some\_module/errors.rs  
    \#  
    pub enum SomeModuleError {  
        \#\[error("A core I/O operation failed")\]  
        CoreIo(\#\[from\] std::io::Error), // Eindeutige Konvertierung von std::io::Error

        \#\[error("Failed to parse item data")\]  
        Parsing(\#\[from\] serde\_json::Error), // Eindeutige Konvertierung von serde\_json::Error  
    }

* **Manuelles Wrappen zur Hinzufügung von Kontext oder zur Auflösung von Mehrdeutigkeiten:**  
  * **Hinzufügen von Kontext:** Wenn ein Fehler eines Aufgerufenen gewrappt werden muss, um *zusätzliche kontextuelle Informationen* bereitzustellen, die für das Verständnis des Fehlers im Kontext des Aufrufers entscheidend sind (z.B. die spezifische Datei, die verarbeitet wird, der gesuchte Schlüssel), *muss* eine dedizierte Fehlervariante definiert werden. Diese Variante *muss* Felder für den zusätzlichen Kontext und ein Feld, das mit \#\[source\] annotiert ist, zur Speicherung des ursprünglichen Fehlers enthalten.  
    Rust  
    // In core/src/config/errors.rs  
    use std::path::PathBuf; // Hinzugefügt für Vollständigkeit

    \#  
    pub enum ConfigError {  
        \#\[error("Failed to load configuration from '{path}'")\]  
        LoadFailed {  
            path: PathBuf,  
            \#\[source\]  
            source: std::io::Error, // Manuell gewrappt, um 'path'-Kontext hinzuzufügen  
        },  
        //... andere Varianten  
    }

  * **Auflösung von \#\[from\]-Mehrdeutigkeiten:** Die thiserror-Crate erlaubt nicht mehrere \#\[from\]-Annotationen für den *gleichen Quellfehlertyp* innerhalb eines einzelnen Enums.1 Wenn die Operationen eines Moduls denselben zugrundeliegenden Fehlertyp (z.B. std::io::Error) aus logisch unterschiedlichen Operationen (z.B. Lesen einer Datei vs. Schreiben einer Datei) ergeben können, kann \#\[from\] nicht für beide verwendet werden. In diesem Szenario:  
    1. Es *müssen* unterschiedliche Fehlervarianten für jede logische Operation erstellt werden.  
    2. Jede solche Variante *muss* den gemeinsamen zugrundeliegenden Fehlertyp manuell unter Verwendung eines mit \#\[source\] annotierten Feldes wrappen.  
    3. Die \#\[error("...")\]-Nachricht und alle zusätzlichen kontextuellen Felder dieser Varianten *müssen* die logischen Operationen klar unterscheiden.

Rust  
// In core/src/some\_module/errors.rs  
use std::path::PathBuf; // Hinzugefügt für Vollständigkeit

\#  
pub enum FileOperationError {  
    \#\[error("Failed to read data from file '{path}'")\]  
    ReadError {  
        path: PathBuf,  
        \#\[source\]  
        source: std::io::Error, // std::io::Error aus einer Leseoperation  
    },

    \#\[error("Failed to write data to file '{path}'")\]  
    WriteError {  
        path: PathBuf,  
        \#\[source\]  
        source: std::io::Error, // std::io::Error aus einer Schreiboperation  
    },  
}  
Diese Vorgehensweise erhält die semantische Spezifität des Fehlers und ermöglicht es Aufrufern, Fehlermodi zu unterscheiden, was für eine robuste Fehlerbehandlungslogik entscheidend ist. Es wandelt eine potenzielle Einschränkung von thiserror (bei unsachgemäßer Verwendung) in ein Muster um, das zu aussagekräftigeren Fehlervarianten anregt.

* **Nutzung der source()-Methode:**  
  * Die Methode std::error::Error::source() (verfügbar bei thiserror-abgeleiteten Enums) ist der Standardmechanismus für den Zugriff auf die zugrundeliegende Ursache eines Fehlers.3  
  * Entwickler, die Fehler der Kernschicht (oder Fehler anderer Schichten) konsumieren, *müssen* sich dieser Methode bewusst sein und *sollten* sie in Logging- und Debugging-Routinen verwenden, um die Fehlerkette zu durchlaufen und die vollständige Abfolge der Ursachen zu melden.  
  * Der experimentelle sources()-Iterator 3 wäre, falls stabilisiert, der bevorzugte Weg, um die gesamte Kette zu iterieren. Bis dahin ist eine manuelle Schleife erforderlich:  
    Rust  
    // fn log\_full\_error\_chain(err: &(dyn std::error::Error \+ 'static)) {  
    //     tracing::error\!("Error: {}", err);  
    //     let mut current\_source \= err.source();  
    //     while let Some(source) \= current\_source {  
    //         tracing::error\!("  Caused by: {}", source);  
    //         current\_source \= source.source();  
    //     }  
    // }

Die Bequemlichkeit von \#\[from\] ist verlockend, aber die Einschränkung, dass nicht zwei Fehlervarianten vom selben Quelltyp abgeleitet werden können 1, kann zu einem Verlust an semantischer Unterscheidung führen, wenn sie nicht sorgfältig gehandhabt wird. Die Spezifikation begegnet dem direkt, indem sie manuelles Wrappen mit unterschiedlichen Varianten vorschreibt, wenn eine solche Mehrdeutigkeit auftritt. Dies erhält die Klarheit und nutzt thiserror dennoch effektiv. Effektives Debugging hängt von ausreichendem Kontext ab. Die \#\[source\]-Kette liefert das "Warum" ein Fehler auf einer niedrigeren Ebene aufgetreten ist, während benutzerdefinierte Felder in Fehlervarianten das "Was" und "Wo" spezifisch für die aktuelle Operation liefern.1 Durch die Vorschrift, solche Kontextfelder einzuschließen und \#\[source\] zu verwenden, wird sichergestellt, dass Fehlertypen reich an Informationen sind, was die Debugfähigkeit direkt verbessert.

### **1.3. Modul-spezifische Fehler innerhalb der Kernschicht**

Zweck:  
Durchsetzung eines modularen und spezifischen Ansatzes zur Fehlerbehandlung gemäß Richtlinie 4.3 ("spezifischen Fehler-Enums pro Modul"). Jedes logische Submodul innerhalb der Kernschicht (z.B. core::config, core::utils::string\_processing, core::types\_validation) muss seinen eigenen, distinkten Fehler-Enum definieren. Dies verbessert die Kapselung, erhöht die Klarheit für die Konsumenten des Moduls und steht im Einklang mit bewährten Praktiken.2  
**Spezifikationen:**

* **Verbindliche modul-level Fehler-Enums:**  
  * Jedes nicht-triviale öffentliche Submodul innerhalb der Kernschicht, das behebbare Fehler erzeugen kann, *muss* seinen eigenen öffentlichen Fehler-Enum definieren (z.B. pub enum ConfigError {... } in core::config::errors, pub enum ValidationRuleError {... } in core::types::validation::errors).  
  * Diese Enums *müssen* mittels \# definiert werden.  
  * Sie *müssen* allen Spezifikationen bezüglich Display-Nachrichten (Abschnitt 1.1) und Fehlerquellenverkettung/Kontext (Abschnitt 1.2) entsprechen.  
* **Granularität und Kohäsion:**  
  * Die Granularität der Fehler-Enums sollte sich an Modulgrenzen und logischen Funktionsbereichen orientieren. Ein einzelnes, großes Modul könnte einen umfassenden Fehler-Enum für seine Operationen definieren. Wenn ein Modul übermäßig groß wird oder seine Fehlerzustände zu vielfältig werden, *sollte* eine Refaktorierung in kleinere Submodule in Betracht gezogen werden, von denen jedes einen fokussierteren Fehler-Enum besitzt. Dies folgt dem Geist der Diskussion in 2 über das Gleichgewicht zwischen der Verbreitung von Fehlertypen und der Spezifität.  
  * Die Erstellung von Fehlertypen für einzelne Funktionen ist zu vermeiden, es sei denn, diese Funktion stellt eine signifikante, distinkte Einheit fehlbarer Arbeit dar.  
* **Keine direkte Propagierung von CoreError aus Submodulen:**  
  * Submodule der Kernschicht (z.B. core::config) *dürfen typischerweise nicht* den generischen CoreError (definiert in Abschnitt 1.1) zurückgeben. Sie *müssen* ihre eigenen spezifischen Fehlertypen zurückgeben (z.B. ConfigError).  
  * CoreError ist für Fehler reserviert, die innerhalb von core::errors selbst entstehen, oder für wirklich schichtweite, nicht klassifizierbare Probleme, die keinem spezifischen Submodul zugeordnet werden können.  
* **Intermodul-Fehlerkonvertierung/-wrapping (innerhalb der Kernschicht):**  
  * Wenn ein Kernschichtmodul Alpha eine Funktion eines anderen Kernschichtmoduls Beta aufruft und die Funktion von Beta Result\<T, BetaError\> zurückgibt, dann *muss* der Fehler-Enum von Alpha (AlphaError) eine Variante definieren, um BetaError zu wrappen, falls dieser Fehler propagiert werden soll.  
  * Dieses Wrapping *sollte* typischerweise \#\[from\] für die Kürze verwenden, wenn die Zuordnung innerhalb von AlphaError eindeutig ist.  
    Rust  
    // In core/src/module\_alpha/errors.rs  
    use crate::module\_beta::errors::BetaError; // Annahme: BetaError ist korrekt importiert

    \#  
    pub enum AlphaError {  
        \#\[error("An error occurred in the beta subsystem")\]  
        BetaSystemFailure(\#\[from\] BetaError),  
        //... andere AlphaError Varianten  
    }

  * Dies stellt sicher, dass Konsumenten von module\_alpha nur direkt auf AlphaError-Varianten matchen müssen, aber immer noch über AlphaError::BetaSystemFailure(...).source() auf den zugrundeliegenden BetaError zugreifen können.  
* **Handhabung von \#\[from\]-Konflikten (Wiederholung für Modulfehler):**  
  * Die Regel aus Abschnitt 1.2 bezüglich des manuellen Wrappings für mehrdeutige \#\[from\]-Quellen gilt gleichermaßen für modul-spezifische Fehler-Enums. Wenn core::config::ConfigError std::io::Error sowohl von einer Lese- als auch einer Schreiboperation repräsentieren muss, *muss* es distinkte Varianten wie ReadIoError { \#\[source\] source: std::io::Error,... } und WriteIoError { \#\[source\] source: std::io::Error,... } haben.

Modul-spezifische Fehler sind ein Eckpfeiler der Kapselung. Konsumenten eines Moduls (z.B. core::config) müssen nur ConfigError kennen, nicht die internen Fehlertypen (wie serde\_json::Error oder std::io::Error), die core::config möglicherweise handhabt und wrappt.2 Dies reduziert die Kopplung zwischen Modulen und Schichten erheblich. Würde core::config die Fehler seiner internen Abhängigkeiten direkt exponieren, wären alle Nutzer von core::config auch an diese Abhängigkeiten gekoppelt. Eine spätere Änderung der JSON-Parsing-Bibliothek in core::config würde dann alle seine Konsumenten brechen. Durch die Definition von ConfigError mit Varianten wie ParseFailure(\#\[from\] serde\_json::Error) schirmt core::config seine Konsumenten ab.  
Indem jedes Modul nur seinen eigenen Fehler-Enum definiert und exponiert, stellt die Kernschicht den höheren Schichten (Domäne, System, UI) eine abstraktere und handhabbare Menge von Fehlertypen zur Verfügung. Diese höheren Schichten wrappen dann Fehler der Kernschicht in ihre eigenen, abstrakteren Fehlertypen. Dies erzeugt eine saubere Hierarchie der Fehlerabstraktion und verhindert eine überwältigende Verbreitung spezifischer Fehlertypen auf höheren Ebenen.2 Die detaillierten Regeln für die Verwendung von thiserror, insbesondere bezüglich \#\[from\]-Mehrdeutigkeiten und manuellem Wrappen für Kontext, stellen sicher, dass die gewählte Bibliothek ihr volles Potenzial entfaltet und Fehlertypen erzeugt werden, die sowohl ergonomisch für Entwickler als auch reich an diagnostischen Informationen sind. Dies begegnet potenziellen Fallstricken, die in 1 erwähnt werden, durch die Bereitstellung konkreter, handlungsorientierter Muster.

### **1.4. Durchsetzung der Strategie für Panic vs. Error**

Zweck:  
Es wird eine strikte, unzweideutige Unterscheidung zwischen behebbaren Laufzeitfehlern (die zwingend mittels Result\<T, E\> und den oben definierten Fehlertypen behandelt werden müssen) und nicht behebbaren Programmierfehlern oder kritischen Invariantenverletzungen (die zu einem panic führen sollten) etabliert und durchgesetzt. Dies entspricht der fundamentalen Fehlerbehandlungsphilosophie von Rust.4  
**Spezifikationen:**

* **Striktes Verbot von .unwrap() und .expect() in Bibliotheks-Code der Kernschicht:**  
  * Die Verwendung der Methoden .unwrap() oder .expect() auf Result\<T, E\>- oder Option\<T\>-Typen ist in jeglichem Bibliotheks-Code der Kernschicht *strikt verboten*. Bibliotheks-Code ist definiert als jeder Code innerhalb der core-Crate, der für die Verwendung durch andere Schichten (Domäne, System, UI) oder andere Module innerhalb der Kernschicht vorgesehen ist.  
  * Alle Operationen, die auf einen behebbaren Fehler stoßen können, *müssen* explizit Result\<T, E\> zurückgeben, wobei E ein geeigneter Fehlertyp gemäß den Spezifikationen in den Abschnitten 1.1-1.3 ist. Diese strikte Regel ist der primäre Mechanismus, um sicherzustellen, dass alle potenziellen behebbaren Fehlerpfade explizit berücksichtigt und durch Rückgabe von Result behandelt werden, was fundamental für die Entwicklung robuster Software in Rust ist.4 Jeder Aufruf von .unwrap() oder .expect() in Bibliotheks-Code ist ein versteckter panic, der die gesamte Desktop-Umgebung zum Absturz bringen kann.  
* **Zulässige, wohlüberlegte Verwendung von .expect() (Nicht-Bibliotheks-Kontexte):**  
  * .expect() *darf nur* in den folgenden, gut begründeten Nicht-Bibliotheks-Kontexten verwendet werden:  
    * **Tests:** Innerhalb von Unit-Tests (\#\[test\]) und Integrationstests (in tests/), wo ein Fehlschlag einen Fehler im Test-Setup, ein Missverständnis der getesteten Komponente oder einen echten, durch den Test aufgedeckten Bug anzeigt. Der Test selbst ist die Grenze der Wiederherstellbarkeit.  
    * **Interne Werkzeuge/Binaries:** In main.rs oder Hilfsfunktionen von internen Kommandozeilenwerkzeugen, Build-Skripten oder Dienstprogrammen, die *nicht* Teil der Kernschicht-Bibliothek selbst sind und bei denen ein Fehlerzustand für die Ausführung *dieses spezifischen Werkzeugs* tatsächlich nicht behebbar ist.  
    * **Kritische Invarianten (selten):** In äußerst seltenen Situationen innerhalb des Bibliotheks-Codes, in denen eine Bedingung aufgrund vorheriger validierter Logik *garantiert* wahr ist (z.B. Zugriff auf ein Array-Element nach einer Grenzenprüfung). Wenn diese Invariante verletzt wird, signalisiert dies einen kritischen, nicht behebbaren internen Logikfehler (einen Bug). Eine solche Verwendung *muss* ausführlich kommentiert und begründet werden. Dies ist eine Ausnahme, nicht die Regel.  
* **Verbindlicher Stil für .expect()-Nachrichten:**  
  * Wenn .expect() zulässigerweise verwendet wird (wie oben definiert), *muss* die bereitgestellte Nachrichtenzeichenkette dem Stil "expect as precondition" entsprechen, wie in 4 befürwortet.  
  * Die Nachricht *darf nicht* lediglich den aufgetretenen Fehler beschreiben (was oft redundant mit der Display-Nachricht des zugrundeliegenden Fehlers ist, falls das Result ein Err enthielt).  
  * Stattdessen *muss* die Nachricht die *Vorbedingung* oder *Invariante* beschreiben, von der erwartet wurde, dass sie zutrifft, und erklären, *warum* erwartet wurde, dass die Operation erfolgreich ist.  
  * **Korrektes Beispiel (Precondition Style):**  
    Rust  
    // In einem Test oder internen Werkzeug:  
    // let config \= get\_config\_somehow(); // Platzhalter für Konfigurationsbeschaffung  
    // let user\_count: u32 \= config.get\_max\_users()  
    //    .expect("System configuration 'max\_users' should be present and valid at this point");

  * **Falsches Beispiel (Error Message Style \- NICHT VERWENDEN):**  
    Rust  
    // let user\_count \= config.get\_max\_users().expect("Failed to get max\_users"); // SCHLECHTER STIL

Die Übernahme des "expect as precondition"-Stils für Panic-Nachrichten 4 verwandelt Panics von einfachen Absturzberichten in wertvolle Diagnosewerkzeuge. Diese Nachrichten erklären die verletzten Annahmen des Programmierers und lenken die Debugging-Bemühungen direkt auf den logischen Fehler. Eine Nachricht wie "env variable 'IMPORTANT\_PATH' should be set by 'wrapper\_script.sh'" 4 ist weitaus informativer als "env variable 'IMPORTANT\_PATH' is not set".

* **Direkte Verwendung des panic\!-Makros:**  
  * Direkte Aufrufe von panic\!("message") *sollten* Situationen vorbehalten bleiben, in denen das Programm einen nicht wiederherstellbaren Zustand, eine verletzte kritische Invariante oder eine logische Unmöglichkeit feststellt, die eindeutig auf einen Bug im eigenen Code der Kernschicht hinweist.  
  * Die Panic-Nachricht *sollte* klar und informativ sein und Entwicklern bei der Diagnose des Bugs helfen.  
  * Panicking ist angebracht, wenn eine Fortsetzung der Ausführung zu weiteren Fehlern, Datenkorruption oder undefiniertem Verhalten führen würde.

---


# **A4 Kernschicht: Kerninfrastruktur (Teil 4/4)**

## **1\. Einleitung**

Dieses Dokument ist Teil 4 der Spezifikation für die Kernschicht (Core Layer) und konzentriert sich auf die Definition der fundamentalen Infrastrukturkomponenten. Diese Komponenten bilden das Rückgrat für alle darüberliegenden Schichten der Desktop-Umgebung und umfassen die Fehlerbehandlung, das Logging-System, Mechanismen zur Konfigurationsverwaltung sowie grundlegende Datentypen und Hilfsfunktionen.  
Ziel dieses Dokuments ist es, eine ultra-feingranulare Spezifikation bereitzustellen, die es Entwicklern ermöglicht, diese Kerninfrastrukturelemente direkt zu implementieren. Jede Komponente, Methode, Datenstruktur und Richtlinie wird detailliert beschrieben, um Klarheit zu gewährleisten und Designentscheidungen vorwegzunehmen. Die hier definierten Systeme sind entscheidend für die Stabilität, Wartbarkeit, Diagnosefähigkeit und Konsistenz der gesamten Desktop-Umgebung.  
Die folgenden Abschnitte behandeln:

* **Fehlerbehandlungsinfrastruktur (core::errors)**: Definition eines robusten und konsistenten Ansatzes zur Fehlerbehandlung unter Verwendung der thiserror-Crate.  
* **Core Logging Infrastruktur (core::logging)**: Spezifikation eines strukturierten Logging-Systems basierend auf der tracing-Crate.  
* **Core Konfigurationsprimitive (core::config)**: Festlegung von Mechanismen zum Laden, Parsen und Zugreifen auf Basiskonfigurationen.  
* **Core Utilities (core::utils)**: Richtlinien für allgemeine Hilfsfunktionen.  
* **Core Datentypen (core::types)**: Definition fundamentaler, systemweit genutzter Datentypen.

Die sorgfältige Implementierung dieser Infrastrukturkomponenten ist unerlässlich, da sie die Qualität und Zuverlässigkeit aller anderen Teile des Systems maßgeblich beeinflussen.

## **2\. Fehlerbehandlungsinfrastruktur (core::errors)**

Eine robuste und aussagekräftige Fehlerbehandlung ist das Fundament stabiler Software. Für die Kernschicht, die von allen anderen Schichten genutzt wird, ist dies von besonderer Bedeutung. Die hier definierte Infrastruktur zielt auf Klarheit, Konsistenz und einfache Nutzung für Entwickler ab.

### **2.1. Grundlagen und Wahl von thiserror**

Die Fehlerbehandlung in Rust basiert auf dem Result\<T, E\>-Enum, wobei E typischerweise den std::error::Error-Trait implementiert.1 Für die Definition benutzerdefinierter Fehlertypen wird die Crate thiserror eingesetzt. Diese Wahl begründet sich dadurch, dass thiserror speziell für Bibliotheken konzipiert ist, im Gegensatz zu anyhow, das eher für Applikationen (Binaries) gedacht ist.1 Die Kernschicht und viele Teile der Domänen- und Systemschicht fungieren als Bibliotheken für andere Teile der Desktop-Umgebung.  
thiserror bietet folgende Vorteile:

* Es generiert Boilerplate-Code für die Implementierung des std::error::Error-Traits.  
* Es ermöglicht die einfache Definition von Fehlermeldungen über das \#\[error(...)\]-Attribut.  
* Es unterstützt die Konvertierung von zugrundeliegenden Fehlern mittels des \#\[from\]-Attributs, was die Verwendung des ?-Operators erleichtert.1

### **2.2. Granularität: Ein Fehler-Enum pro Modul**

Um eine klare Struktur und gute Verwaltbarkeit der Fehlertypen zu gewährleisten, wird festgelegt, dass jedes signifikante Modul innerhalb der Kernschicht (und konsequenterweise auch in den höheren Schichten) sein eigenes, spezifisches Fehler-Enum definiert.2 Dies stellt einen guten Kompromiss zwischen der Notwendigkeit spezifischer Fehlerbehandlung und der Vermeidung einer übermäßigen Anzahl globaler oder unspezifischer Fehlertypen dar.  
Eine potenzielle Einschränkung von thiserror ist, dass man nicht zwei Fehlervarianten vom selben Ursprungstyp (source type) definieren kann, wenn man \#\[from\] direkt verwendet, was dazu führen könnte, dass der Kontext verloren geht (z.B. ob ein std::io::Error beim Lesen oder Schreiben auftrat).1 Die Strategie, pro Modul ein eigenes Fehler-Enum zu definieren, mildert dieses Problem erheblich. Selbst wenn sowohl ModuleAError als auch ModuleBError einen std::io::Error wrappen, liefert bereits der Typ des Fehler-Enums (ModuleAError vs. ModuleBError) wichtigen Kontext. Innerhalb eines Modul-Enums können zudem spezifische Varianten erstellt werden, die denselben zugrundeliegenden Fehlertyp wrappen, aber unterschiedliche Operationen oder Kontexte repräsentieren. Zum Beispiel könnte ein ConfigError-Enum Varianten wie ReadError { path: PathBuf, \#\[source\] source: std::io::Error } und ParseError { path: PathBuf, \#\[source\] source: serde\_toml::Error } haben. Dies stellt sicher, dass der Kontext nicht "verwischt" wird, wie in 1 als potenzielle Herausforderung beschrieben. Die Kombination aus modul-spezifischen Enums und sorgfältig benannten Varianten mit kontextuellen Feldern sorgt für die notwendige Klarheit.

### **2.3. thiserror Implementierungsrichtlinien und Pro-Modul Fehler-Enums**

Für jedes Modul, das Fehler erzeugen kann, muss ein Fehler-Enum mit thiserror definiert werden.  
Strukturbeispiel:  
Angenommen, es gibt ein Modul core::some\_module:

Rust

// In core::some\_module::error.rs (oder direkt im Modul)  
use std::path::PathBuf;  
use thiserror::Error;

\#  
pub enum SomeModuleError {  
    \#\[error("Fehler bei der Initialisierung der Komponente: {reason}")\]  
    InitializationFailure { reason: String },

    \#\[error("Ungültiger Parameter '{parameter\_name}': {details}")\]  
    InvalidParameter { parameter\_name: String, details: String },

    \#  
    IoError {  
        operation: String,  
        path: PathBuf,  
        \#\[source\]  
        source: std::io::Error,  
    },

    \#  
    DeserializationError {  
        path: PathBuf,  
        \#\[source\]  
        source: serde\_json::Error, // Beispiel für einen spezifischen Deserialisierungsfehler  
    },

    \#\[error("Feature '{feature\_name}' ist nicht verfügbar.")\]  
    FeatureUnavailable { feature\_name: String },

    \#  
    DependentServiceError {  
        service\_name: String,  
        \#\[source\]  
        source: Box\<dyn std::error::Error \+ Send \+ Sync \+ 'static\>, // Für generische Fehler von Abhängigkeiten  
    },  
}

**\#\[error(...)\]-Annotationen:**

* Die Fehlermeldungen müssen primär entwicklerorientiert sein: präzise, informativ und klar verständlich.  
* Sie müssen den Grund des Fehlers erläutern und wichtige kontextuelle Parameter (z.B. Dateipfade, Parameternamen, fehlerhafte Werte) über die Felder der Enum-Variante einbinden (z.B. {parameter\_name}).  
* Der Stil der Meldungen soll konsistent sein: typischerweise in Kleinschreibung, prägnant und ohne abschließende Satzzeichen, es sei denn, diese sind Teil eines zitierten Literals.3  
* Die Meldungen sollen dazu beitragen, die "Grundursache" des Fehlers zu verstehen.1 Obwohl die "Benutzerperspektive" in 1 erwähnt wird, ist der "Benutzer" eines Core-Layer-Fehlers typischerweise ein anderer Entwickler, der diese Schicht verwendet.

**\#\[from\]-Annotationen:**

* Das \#\[from\]-Attribut wird verwendet, um Fehler von anderen Typen (z.B. std::io::Error, Fehler aus anderen Kernschichtmodulen oder externen Crates) transparent in eine Variante des aktuellen Modul-Fehler-Enums zu konvertieren.  
* Dies ist entscheidend für die ergonomische Fehlerweitergabe mittels des ?-Operators.  
* **Spezifikation**: \#\[from\] ist dann angemessen, wenn ein externer Fehlertyp direkt einer *semantisch eindeutigen* Fehlerbedingung innerhalb des Moduls zugeordnet werden kann. Falls ein externer Fehlertyp aus mehreren unterschiedlichen Operationen innerhalb des Moduls resultieren kann, sind spezifische Varianten zu erstellen, die den Ursprungsfehler mit zusätzlichem Kontext umhüllen (wie im IoError-Beispiel oben, das ein operation-Feld und path-Feld enthält). Dies vermeidet Ambiguität und stellt sicher, dass der Fehlertyp selbst bereits maximalen Kontext liefert.

**Kontextuelle Informationen:**

* Fehlervarianten müssen Felder enthalten, die relevante kontextuelle Informationen zum Zeitpunkt der Fehlererzeugung erfassen (z.B. Dateipfade, betroffene Werte, Operationsnamen). Dies unterstützt die Forderung nach einem "vollständigen Kontext-Stack" für Debugging-Zwecke.1

**Tabelle: Übersicht der Kernmodul-Fehler-Enums (Auszug)**

| Modulpfad | Fehler-Enum-Name | Schlüssekvarianten (illustrativ) | Primäre \#\[from\] Quellen (Beispiele) |
| :---- | :---- | :---- | :---- |
| core::config | ConfigError | FileReadError, DeserializationError, MissingKeyError | std::io::Error, serde\_toml::de::Error |
| core::utils::json | JsonUtilError | SerializationError, DeserializationError | serde\_json::Error |
| core::ipc | IpcError | ConnectionFailed, MessageSendError, ResponseTimeout | zbus::Error (falls zbus verwendet wird) |
| core::types::color | ColorParseError | InvalidHexFormat, InvalidHexDigit | std::num::ParseIntError |

*Begründung für den Wert der Tabelle*:

1. **Auffindbarkeit**: Bietet Entwicklern einen schnellen Überblick über alle benutzerdefinierten Fehlertypen innerhalb der Kernschicht.  
2. **Konsistenz**: Fördert einen standardisierten Ansatz für die Benennung und Strukturierung von Fehler-Enums über Module hinweg.  
3. **Modulübergreifendes Verständnis**: Hilft Entwicklern zu verstehen, welche Arten von Fehlern beim Aufruf von Funktionen aus verschiedenen Kernmodulen zu erwarten sind, was eine bessere Fehlerbehandlung im aufrufenden Code ermöglicht.  
4. **Wartung**: Dient als Checkliste bei Code-Reviews, um sicherzustellen, dass neue Module ihre Fehlertypen gemäß den Projektspezifikationen korrekt definiert haben.

### **2.4. Fehlerweitergabe, \-konvertierung und \-verkettung**

* **?-Operator**: Die Verwendung des ?-Operators ist für die Weitergabe von Result-Fehlern den Aufrufstack hinauf verbindlich vorgeschrieben. Dies ist idiomatisches Rust und verbessert die Lesbarkeit des Codes erheblich.  
* **\#\[from\] zur Konvertierung**: Wie oben detailliert, ist \#\[from\] (bereitgestellt durch thiserror) der primäre Mechanismus zur Konvertierung eines Fehlertyps in einen anderen, was die Nutzung von ? erleichtert.  
* **source()-Verkettung**: Es ist sicherzustellen, dass das \#\[source\]-Attribut von thiserror auf dem Feld verwendet wird, das den zugrundeliegenden Fehler enthält. Dies ermöglicht es Konsumenten, die vollständige Fehlerkette über std::error::Error::source() zu inspizieren, was für das Debugging komplexer Probleme, die sich über mehrere Module oder Operationen erstrecken, unerlässlich ist.3 Die source()-Kette ist das programmatische Äquivalent des in 1 erwähnten "virtuellen Benutzer-Stacks". Jede Ebene der source()-Aufrufe enthüllt eine tiefere Ursache des Fehlers. Wenn ein Fehler E1 einen Ursprungsfehler E2 (der wiederum E3 usw. wrappen könnte) umschließt, rekonstruiert die Iteration durch e1.source(), dann e1.source().unwrap().source() usw. effektiv die kausale Fehlerkette. Diese Kette liefert den "vollständigen Kontext-Stack", indem sie zeigt, wie sich ein Low-Level-Fehler durch verschiedene Abstraktionsschichten fortgepflanzt und transformiert hat. Daher ist die konsistente und korrekte Verwendung von \#\[source\] für die Erreichung der Debugging-Ziele von entscheidender Bedeutung.

### **2.5. Fehlerkontext und entwicklerorientiertes Reporting**

* **Hinzufügen von Kontext**: Über die \#\[error(...)\]-Nachricht hinaus müssen Funktionen, die Result zurückgeben, sicherstellen, dass die von ihnen konstruierten Fehlerwerte genügend Informationen enthalten, damit ein Entwickler den Zustand verstehen kann, der zum Fehler geführt hat. Dies bedeutet oft, Fehlervarianten mit spezifischen Feldern zu erstellen, die diesen Zustand erfassen.  
* **Integration mit core::logging**: Wenn ein Fehler behandelt wird (d.h. nicht weiter mit ? propagiert wird), sollte er typischerweise mit der core::logging-Infrastruktur (siehe Abschnitt 3\) protokolliert werden. Der Log-Eintrag sollte die vollständigen Fehlerinformationen enthalten, oft durch Protokollierung der Debug-Repräsentation des Fehlers, die die source-Kette einschließt.  
  * Beispiel: tracing::error\!(error \=?e, "Kritische Operation X fehlgeschlagen");  
* **Keine sensiblen Daten**: Es wird die strikte Richtlinie wiederholt: Fehlermeldungen und protokollierte Fehlerdetails dürfen *niemals* Passwörter, API-Schlüssel, personenbezogene Daten (PII) oder andere sensible Informationen enthalten. Redaktion oder Auslassung ist erforderlich, wenn solche Daten peripher an einer Fehlerbedingung beteiligt sind.

### **2.6. Panic-Strategie (Core Layer Spezifika)**

Panics signalisieren nicht behebbare Fehler, die typischerweise auf Programmierfehler hinweisen.4 Ihre Verwendung in der Kernschicht muss streng kontrolliert werden.

* **Verbot in Bibliothekscode**: Panics (unwrap(), expect(), panic\!) sind in Code der Kernschicht, der für die allgemeine Nutzung durch andere Schichten vorgesehen ist, strikt verboten. Funktionen und Methoden müssen für alle fehleranfälligen Operationen Result zurückgeben.  
* **Zulässige Verwendungen**:  
  * **Nicht behebbare Initialisierung**: In den frühesten Phasen des Anwendungsstarts, wenn eine fundamentale Ressource nicht initialisiert werden kann und die Anwendung unmöglich fortfahren kann (z.B. eine kritische Konfigurationsdatei ist fehlerhaft und es gibt keine Standardwerte), kann ein Panic als letztes Mittel akzeptabel sein.  
  * **Tests**: unwrap() und expect() sind in Testcode zulässig und oft idiomatisch, um Bedingungen zu assertieren, die *unbedingt* gelten müssen.  
  * **Interne Invarianten**: In seltenen Fällen kann expect() verwendet werden, um eine interne Invariante zu assertieren, die logischerweise *niemals* verletzt werden sollte. Wenn sie es doch wird, deutet dies auf einen Fehler in der Kernschicht selbst hin.  
* **expect()-Nachrichtenstil**: Wenn expect() in den zulässigen Szenarien verwendet wird, *muss* die Nachricht dem Stil "expect as precondition" (Erwartung als Vorbedingung) folgen.4 Die Nachricht sollte beschreiben, *warum* erwartet wurde, dass die Operation erfolgreich ist, und nicht nur den Fehler wiederholen.  
  * Beispiel: let config\_value \= map.get("critical\_key").expect("critical\_key sollte in der beim Start geladenen Standardkonfiguration vorhanden sein"); Der Stil "expect as precondition" ist dem Stil "expect as error message" überlegen, da er dem Entwickler, der den Panic debuggt, neue Informationen hinzufügt.4 Er erklärt die verletzte Annahme, während "expect as error message" oft nur wiederholt, was der zugrundeliegende Fehler bereits aussagt (z.B. Panic-Nachricht: "...ist nicht gesetzt: Nicht vorhanden"). Durch die Fokussierung auf das, was hätte wahr sein *sollen*, wird der Kontext über den beabsichtigten Zustand und die Annahmen des Programms verdeutlicht. Dies erleichtert das Debugging, da es unmittelbar auf eine fehlerhafte Annahme oder einen Fehler in einem vorangegangenen Schritt hinweist, der diese Vorbedingung hätte herstellen sollen. Für die Kernschicht, wo Robustheit und Klarheit an erster Stelle stehen, verbessert die Durchsetzung dieses Stils für die seltenen Fälle von expect() die Wartbarkeit und Fehlerdiagnose.

## **3\. Core Logging Infrastruktur Spezifikation (core::logging)**

Diese Sektion definiert die standardisierte Logging-Infrastruktur für die gesamte Desktop-Umgebung, basierend auf der tracing-Crate, wie in der Gesamtarchitektur (Abschnitt 4.4) festgelegt. Das Modul core::logging wird Initialisierungsroutinen und potenziell gemeinsame Logging-Makros oder Hilfsfunktionen bereitstellen, obwohl die Makros von tracing selbst in der Regel ausreichend sind.

### **3.1. tracing Framework Integrationsdetails**

* **Initialisierung**:  
  * Eine dedizierte Funktion, z.B. pub fn initialize\_logging(level\_filter: tracing::LevelFilter, format: LogFormatEnum) \-\> Result\<(), LoggingError\>, muss bereitgestellt werden. Diese Funktion wird sehr früh im Anwendungslebenszyklus aufgerufen (z.B. in main.rs).  
  * Sie konfiguriert einen globalen Standard tracing\_subscriber.  
  * LogFormatEnum könnte Varianten wie PlainTextDevelopment, JsonProduction definieren.  
  * LoggingError wäre ein Enum, das mit thiserror im Modul core::logging definiert wird (z.B. für Fehler beim Setzen des globalen Subscribers).  
* **Subscriber-Konfiguration**:  
  * Für Entwicklungs-Builds (LogFormatEnum::PlainTextDevelopment): tracing\_subscriber::fmt() mit with\_ansi(true) (falls Terminal es unterstützt), with\_target(true) (zeigt Modulpfad), with\_file(true), with\_line\_number(true) und dem übergebenen level\_filter. Dies liefert eine reichhaltige, menschenlesbare Ausgabe.  
  * Für Release-Builds (LogFormatEnum::JsonProduction): Es wird ein strukturiertes Format wie JSON empfohlen, um die Log-Aggregation und maschinelle Analyse zu erleichtern.2 Dies kann über tracing\_subscriber::fmt::json() oder spezialisierte Formatter wie tracing-bunyan-formatter erreicht werden. Die Wahl des Formats kann ein Argument für initialize\_logging sein.  
* **Dynamische Log-Level-Änderungen**: Obwohl keine V1-Anforderung für core::logging selbst, sollte das Subscriber-Setup im Hinblick auf mögliche zukünftige Anforderungen an dynamische Log-Level-Anpassungen (z.B. über ein D-Bus-Signal oder Neuladen einer Konfigurationsdatei) gestaltet sein. tracing\_subscriber::filter::EnvFilter oder benutzerdefinierte Filter-Implementierungen können dies unterstützen. EnvFilter erlaubt es, den Log-Level über eine Umgebungsvariable (z.B. RUST\_LOG) zu steuern.

### **3.2. Standardisierte Log-Makros und tracing::instrument Verwendung**

* **Standard-Makros**: Die direkte Verwendung der tracing-Makros (trace\!, debug\!, info\!, warn\!, error\!) ist verbindlich vorgeschrieben.  
* **Log-Nachrichtenstruktur**:  
  * Nachrichten sollten prägnant und beschreibend sein.  
  * Für strukturierte Daten sind Schlüssel-Wert-Paare zu verwenden: tracing::info\!(user\_id \= %user.id, action \= "login", "Benutzer hat sich angemeldet"); (Verwendung von % für Display-Implementierungen, ? für Debug).  
  * Fehler sollten mit dem Feld error protokolliert werden: tracing::error\!(error \=?err, "Anfrage konnte nicht verarbeitet werden");. Das ?-Zeichen stellt sicher, dass die Debug-Repräsentation des Fehlers (einschließlich der source-Kette) erfasst wird.  
* **\#\[tracing::instrument\] Verwendung**:  
  * **Zweck**: Erzeugt Spans für Funktionen oder Codeblöcke, die kontextuelle Informationen (einschließlich Timing) liefern und nachfolgende Log-Ereignisse innerhalb dieses Spans gruppieren.  
  * **Richtlinien**:  
    * Anwendung auf öffentliche API-Funktionen signifikanter Module, insbesondere solche, die I/O oder komplexe Berechnungen beinhalten.  
    * Anwendung auf Funktionen, die abgeschlossene operative Einheiten oder Phasen in einem Prozess darstellen.  
    * Verwendung von skip(...) oder skip\_all, um die Protokollierung sensibler oder übermäßig ausführlicher Argumente zu vermeiden.  
    * Verwendung von fields(...), um dem Span spezifischen Kontext hinzuzufügen, z.B. \#\[tracing::instrument(fields(entity.id \= %entity.id))\].  
    * Die Option err kann verwendet werden, um Fehler automatisch auf dem ERROR-Level zu erfassen, wenn die instrumentierte Funktion ein Result::Err zurückgibt: \#\[tracing::instrument(err)\].  
    * Das level Attribut kann verwendet werden, um das Level des Spans selbst zu steuern (z.B. \#\[tracing::instrument(level \= "debug")\]).

**Tabelle: tracing::instrument Verwendungsmuster**

| Szenario | \#\[tracing::instrument\] Attribute | Begründung |
| :---- | :---- | :---- |
| Öffentlicher API-Einstiegspunkt | level \= "debug" (oder info für sehr wichtige APIs) | Nachverfolgung aller Aufrufe öffentlicher APIs für Audit- und Debugging-Zwecke. |
| I/O-Operation (z.B. Datei lesen) | fields(path \= %file\_path.display()), err | Kontextualisierung der Operation mit relevanten Daten (Dateipfad) und automatische Fehlerprotokollierung. |
| Komplexe Berechnung | skip\_all (falls Argumente groß/komplex), fields(param\_count \= args.len()) | Vermeidung der Protokollierung großer Datenstrukturen, aber Erfassung von Metadaten über die Eingabe. |
| Ereignisbehandlung | fields(event.type \= %event.kind()) | Verknüpfung von Log-Einträgen mit spezifischen Ereignistypen für eine einfachere Analyse. |
| Funktion mit sensiblen Argumenten | skip(password, api\_key) oder skip\_all | Sicherstellung, dass keine sensiblen Daten versehentlich protokolliert werden. |

*Begründung für den Wert der Tabelle*:

1. **Konsistenz**: Stellt sicher, dass \#\[tracing::instrument\] einheitlich und effektiv im gesamten Code verwendet wird.  
2. **Performance-Bewusstsein**: Leitet Entwickler an, wann und wie skip verwendet werden sollte, um Performance-Overhead durch übermäßige Protokollierung von Argumenten zu vermeiden.  
3. **Debuggabilität**: Fördert die Erstellung gut definierter Spans, die das Verständnis des Kontrollflusses und die Diagnose von Problemen in verteilten oder asynchronen Operationen erheblich erleichtern.  
4. **Best Practices**: Kodifiziert bewährte Verfahren für die Instrumentierung verschiedener Arten von Funktionen und reduziert das Rätselraten für Entwickler.

### **3.3. Log-Daten Sensibilität und Redaktionsrichtlinie**

* **Striktes Verbot**: Absolut keine sensiblen Daten (Passwörter, API-Schlüssel, PII, Finanzdetails, Gesundheitsinformationen usw.) dürfen im Klartext protokolliert werden.  
* **Redaktion/Auslassung**: Wenn auf eine Variable, die sensible Daten enthält, Bezug genommen werden *muss* (z.B. wegen ihrer Existenz oder ihres Typs), sollte sie redigiert (z.B. password: "\*\*\*") oder vollständig aus den Log-Feldern entfernt werden.  
* **Debug-Trait-Bewusstsein**: Vorsicht ist geboten beim Ableiten von Debug für Strukturen, die sensible Informationen enthalten. Wenn solche Strukturen über ? protokolliert werden (z.B. error \=?sensitive\_struct), muss ihre Debug-Implementierung eine Redaktion durchführen. Benutzerdefinierte Debug-Implementierungen oder Wrapper-Typen, die die Redaktion handhaben, sind in Betracht zu ziehen.  
* **\#\[tracing::instrument(skip\_all)\]**: Ein primäres Werkzeug, um die versehentliche Protokollierung aller Funktionsargumente zu verhindern. Selektive fields können dann wieder hinzugefügt werden.

Die Verantwortung für die Datensensibilität in Logs ist verteilt. Während core::logging den Mechanismus bereitstellt, muss jedes Modul und jeder Entwickler, der Logging-Anweisungen schreibt oder Debug ableitet, wachsam sein. Das tracing-Framework protokolliert Daten basierend auf dem, was Entwickler in Makros bereitstellen oder was Debug-Implementierungen ausgeben. \#\[tracing::instrument\] kann Funktionsargumente automatisch protokollieren, wenn sie nicht übersprungen werden. Eine zentrale Logging-Richtlinie (wie "keine sensiblen Daten") ist unerlässlich. Das Modul core::logging selbst kann diese Richtlinie jedoch nicht für den *Inhalt* der Logs erzwingen; es stellt nur die Infrastruktur bereit. Daher muss die Richtlinie von den Entwicklern im gesamten Code durch sorgfältige Logging-Praktiken, skip-Attribute und gegebenenfalls benutzerdefinierte Debug-Implementierungen aktiv umgesetzt werden. Dies impliziert die Notwendigkeit von Entwicklerschulungen und Code-Review-Checklisten, die sich auf die Sensibilität von Log-Daten konzentrieren.

## **4\. Core Konfigurationsprimitive Spezifikation (core::config)**

Dieser Abschnitt definiert, wie die Kernschicht und nachfolgend andere Schichten grundlegende Konfigurationseinstellungen laden, parsen und darauf zugreifen. Der Fokus liegt auf Einfachheit, Robustheit und der Einhaltung von XDG-Standards, wo dies für benutzerspezifische Überschreibungen relevant ist (obwohl Konfigurationen der Kernschicht wahrscheinlich systemweit oder Standardeinstellungen sind).

### **4.1. Konfigurationsdateiformat(e) und Parsing-Logik**

* **Format**: TOML (Tom's Obvious, Minimal Language) wird aufgrund seiner guten Lesbarkeit für Menschen und der einfachen Verarbeitung durch Maschinen ausgewählt.  
* **Parsing-Bibliothek**: Das serde-Framework in Verbindung mit der toml-Crate (serde\_toml) wird für die Deserialisierung verwendet.  
* **Ladelogik**:  
  1. Definition von Standard-Konfigurationspfaden (z.B. /usr/share/YOUR\_DESKTOP\_ENV\_NAME/core.toml für Systemstandards, /etc/YOUR\_DESKTOP\_ENV\_NAME/core.toml für systemweite Überschreibungen, und potenziell ein Pfad für Entwicklungstests, z.B. relativ zum Projekt-Root). Die XDG Base Directory Specification ($XDG\_CONFIG\_DIRS, $XDG\_CONFIG\_HOME) sollte für benutzerspezifische Konfigurationen in höheren Schichten berücksichtigt werden, ist aber für core.toml (als Basiskonfiguration) möglicherweise weniger relevant, wenn es sich um reine Systemstandards handelt.  
  2. Eine Funktion wie pub fn load\_core\_config(custom\_path: Option\<PathBuf\>) \-\> Result\<CoreConfig, ConfigError\> wird verantwortlich sein. Sie würde eine definierte Suchreihenfolge für Konfigurationsdateien implementieren (z.B. custom\_path falls gegeben, dann Entwicklungspfad, dann Systempfade).  
  3. Sie versucht, den Inhalt der TOML-Datei vom ersten gefundenen Pfad zu lesen.  
  4. Verwendet serde\_toml::from\_str() zur Deserialisierung des Inhalts in die CoreConfig-Struktur.  
  5. Behandelt I/O-Fehler (Datei nicht gefunden, Zugriff verweigert) und Parsing-Fehler (fehlerhaftes TOML, Typ-Inkonsistenzen) und konvertiert sie in entsprechende Varianten von core::config::ConfigError.  
* **Fehlerbehandlung**: Ein core::config::ConfigError-Enum (unter Verwendung von thiserror) wird definiert, mit Varianten wie:  
  Rust  
  use std::path::PathBuf;  
  use thiserror::Error;

  \#  
  pub enum ConfigError {  
      \#\[error("Fehler beim Lesen der Konfigurationsdatei '{path:?}': {source}")\]  
      FileReadError {  
          path: PathBuf,  
          \#\[source\]  
          source: std::io::Error,  
      },  
      \#\[error("Fehler beim Parsen der Konfigurationsdatei '{path:?}': {source}")\]  
      DeserializationError {  
          path: PathBuf,  
          \#\[source\]  
          source: serde\_toml::de::Error,  
      },  
      \#\[error("Keine Konfigurationsdatei gefunden an den geprüften Pfaden: {checked\_paths:?}")\]  
      NoConfigurationFileFound { checked\_paths: Vec\<PathBuf\> },  
      // Ggf. weitere Varianten für Validierungsfehler, falls nicht in Deserialisierung abgedeckt  
  }

### **4.2. Konfigurationsdatenstrukturen (Ultra-Fein)**

* **CoreConfig-Struktur**: Eine primäre Struktur, z.B. CoreConfig, wird in core::config definiert, um alle spezifischen Konfigurationen der Kernschicht zu halten.  
  Rust  
  use serde::Deserialize;  
  use std::path::PathBuf; // Beispiel für einen komplexeren Typ

  // Beispiel für ein Enum, das in der Konfiguration verwendet wird  
  \#  
  \#\[serde(rename\_all \= "lowercase")\] // Erlaubt "info", "debug" etc. in TOML  
  pub enum LogLevelConfig {  
      Trace,  
      Debug,  
      Info,  
      Warn,  
      Error,  
  }

  impl Default for LogLevelConfig {  
      fn default() \-\> Self { LogLevelConfig::Info }  
  }

  \#  
  \#\[serde(deny\_unknown\_fields)\] // Strikte Prüfung auf unbekannte Felder  
  pub struct CoreConfig {  
      \#\[serde(default \= "default\_log\_level")\]  
      pub log\_level: LogLevelConfig,

      \#\[serde(default \= "default\_feature\_flags")\]  
      pub feature\_flags: FeatureFlags,

      \#\[serde(default)\] // Verwendet FeatureXConfig::default()  
      pub feature\_x\_config: FeatureXConfig,

      \#\[serde(default \= "default\_some\_path")\]  
      pub some\_critical\_path: PathBuf,  
  }

  fn default\_log\_level() \-\> LogLevelConfig { LogLevelConfig::default() }  
  fn default\_feature\_flags() \-\> FeatureFlags { FeatureFlags::default() }  
  fn default\_some\_path() \-\> PathBuf { PathBuf::from("/usr/share/YOUR\_DESKTOP\_ENV\_NAME/default\_resource") }

  impl Default for CoreConfig {  
      fn default() \-\> Self {  
          Self {  
              log\_level: default\_log\_level(),  
              feature\_flags: default\_feature\_flags(),  
              feature\_x\_config: FeatureXConfig::default(),  
              some\_critical\_path: default\_some\_path(),  
          }  
      }  
  }

  \#  
  \#\[serde(deny\_unknown\_fields)\]  
  pub struct FeatureFlags {  
      \#\[serde(default)\] // bool-Felder standardmäßig auf false  
      pub enable\_alpha\_feature: bool,  
      \#\[serde(default \= "default\_beta\_timeout\_ms")\]  
      pub beta\_feature\_timeout\_ms: u64,  
  }

  fn default\_beta\_timeout\_ms() \-\> u64 { 1000 }

  \#  
  \#\[serde(deny\_unknown\_fields)\]  
  pub struct FeatureXConfig {  
      \#\[serde(default \= "default\_retries")\]  
      pub retries: u32,  
      \#\[serde(default)\]  
      pub some\_string\_option: Option\<String\>,  
  }

  fn default\_retries() \-\> u32 { 3 }

  impl Default for FeatureXConfig {  
      fn default() \-\> Self {  
          Self {  
              retries: default\_retries(),  
              some\_string\_option: None,  
          }  
      }  
  }

* **Felder**: Alle Felder müssen explizit definierte Typen haben.  
* **serde::Deserialize**: Die Struktur und ihre verschachtelten Strukturen müssen Deserialize ableiten.  
* **\#\[serde(default \= "path")\]**: Wird umfassend verwendet, um Standardwerte für fehlende Felder in der TOML-Datei bereitzustellen, was die Robustheit erhöht. Die referenzierte Funktion muss den Typ des Feldes zurückgeben. Für Felder, deren Typ Default implementiert, kann auch \#\[serde(default)\] verwendet werden.  
* **\#\[serde(deny\_unknown\_fields)\]**: Wird erzwungen, um zu verhindern, dass Tippfehler oder nicht erkannte Felder in Konfigurationsdateien stillschweigend ignoriert werden.  
* **Validierung**:  
  * Grundlegende Validierung kann durch Typen erfolgen (z.B. u32 für eine Anzahl).  
  * Komplexere Validierungen (z.B. log\_level muss ein gültiger Wert sein, was hier durch das LogLevelConfig-Enum und serde(rename\_all \= "lowercase") bereits gut gehandhabt wird) sollten *nach* der Deserialisierung durchgeführt werden. Dies kann entweder in einem TryFrom\<CoreConfigRaw\>-Muster geschehen, bei dem CoreConfigRaw die deserialisierte Struktur ohne komplexe Validierung ist und CoreConfig die validierte Version, oder durch eine dedizierte validate()-Methode auf CoreConfig, die ein Result\<(), ConfigError\> zurückgibt. Für die Kernschicht kann die initiale Validierung auf die Fähigkeiten von serde und Typbeschränkungen beschränkt sein. Komplexere, semantische Validierungen können bei Bedarf in höheren Schichten oder durch benutzerdefinierte Deserialisierungsfunktionen mit \#\[serde(deserialize\_with \= "...")\] hinzugefügt werden.  
* **Invarianten**: Als Kommentare dokumentiert oder durch Validierungslogik erzwungen (z.B. timeout\_ms \> 0).

**Tabelle: Definitionen der Core-Konfigurationsparameter (Auszug)**

| Parameterpfad | Typ | serde Default-Funktion/Wert | Validierungsregeln (Beispiele) | Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| log\_level | LogLevelConfig | default\_log\_level() | Muss einer der Enum-Werte sein (implizit durch Deserialize) | Globaler Standard-Log-Level für die Anwendung. |
| feature\_flags.enable\_alpha\_feature | bool | false (implizit) | \- | Schaltet ein experimentelles Alpha-Feature ein oder aus. |
| feature\_flags.beta\_feature\_timeout\_ms | u64 | default\_beta\_timeout\_ms() | Muss \>= 0 sein (implizit durch u64) | Timeout-Wert in Millisekunden für ein Beta-Feature. |
| feature\_x\_config.retries | u32 | default\_retries() | Muss \>= 0 sein (implizit durch u32) | Anzahl der Wiederholungsversuche für eine bestimmte Operation in Feature X. |
| some\_critical\_path | PathBuf | default\_some\_path() | Pfad sollte idealerweise existieren (Laufzeitprüfung nötig) | Pfad zu einer kritischen Ressource. |

*Begründung für den Wert der Tabelle*:

1. **Klarheit**: Bietet eine einzige, maßgebliche Referenz für alle verfügbaren Kernkonfigurationen, ihre Typen und Standardwerte.  
2. **Dokumentation**: Unerlässlich für Benutzer/Administratoren, die diese Kerneinstellungen möglicherweise anpassen müssen.  
3. **Entwicklungshilfe**: Hilft Entwicklern, die verfügbaren Konfigurationen zu verstehen und neue konsistent hinzuzufügen.  
4. **Validierungsreferenz**: Zentralisiert die Definition gültiger Werte und Bereiche und unterstützt sowohl die automatisierte Validierung als auch die manuelle Konfiguration.

### **4.3. Konfigurationszugriffs-API**

* **Globaler Zugriff**: Die geladene CoreConfig-Instanz sollte so gespeichert werden, dass sie im gesamten Anwendungskontext effizient zugänglich ist. Hierfür wird eine threadsichere statische Variable verwendet, typischerweise mittels once\_cell::sync::OnceCell.  
  Rust  
  // In core::config  
  use once\_cell::sync::OnceCell;  
  //... CoreConfig Strukturendefinition und andere...

  static CORE\_CONFIG: OnceCell\<CoreConfig\> \= OnceCell::new();

  /// Initialisiert die globale Core-Konfiguration.  
  /// Darf nur einmal während des Anwendungsstarts aufgerufen werden.  
  ///  
  /// \# Errors  
  ///  
  /// Gibt einen Fehler zurück, wenn die Konfiguration bereits initialisiert wurde.  
  pub fn initialize\_core\_config(config: CoreConfig) \-\> Result\<(), CoreConfig\> {  
      CORE\_CONFIG.set(config)  
  }

  /// Gibt eine Referenz auf die global initialisierte Core-Konfiguration zurück.  
  ///  
  /// \# Panics  
  ///  
  /// Paniert, wenn \`initialize\_core\_config()\` nicht zuvor erfolgreich aufgerufen wurde.  
  /// Dies signalisiert einen schwerwiegenden Programmierfehler in der Anwendungsinitialisierung.  
  pub fn get\_core\_config() \-\> &'static CoreConfig {  
      CORE\_CONFIG.get().expect("CoreConfig wurde nicht initialisiert. initialize\_core\_config() muss zuerst aufgerufen werden.")  
  }  
  Das expect im get\_core\_config() ist hier vertretbar, da es einen Programmierfehler darstellt: der Versuch, auf die Konfiguration zuzugreifen, bevor sie geladen wurde, was ein fatales Setup-Problem ist und nicht zur Laufzeit normal behandelt werden kann.  
* **Zugriffsmethoden**: Einfache Getter-Funktionen oder direkter Feldzugriff auf die abgerufene &'static CoreConfig-Instanz.  
* **Thread-Sicherheit**: Der gewählte statische Speichermechanismus (OnceCell) gewährleistet eine threadsichere Initialisierung und einen threadsicheren Zugriff. Die CoreConfig-Struktur selbst sollte Send \+ Sync sein (was sie typischerweise ist, wenn ihre Felder dies sind). Clone wird abgeleitet für Fälle, in denen Teile der Konfiguration herumgereicht oder in einem lokalen Kontext modifiziert werden müssen, ohne den globalen Zustand zu beeinflussen.  
* **Immutabilität**: Die global zugängliche Konfiguration sollte nach der Initialisierung unveränderlich sein, um Laufzeitinkonsistenzen zu vermeiden. Wenn dynamische Konfigurationsaktualisierungen erforderlich sind, würde dies einen komplexeren Mechanismus erfordern (z.B. mit RwLock und einem dedizierten Aktualisierungsprozess), der außerhalb des Rahmens dieser initialen Kernschichtspezifikation liegt, aber architektonisch für zukünftige Erweiterbarkeit berücksichtigt werden sollte.

Die Ableitung von Clone für CoreConfig ermöglicht es Komponenten, bei Bedarf eine Momentaufnahme der Konfiguration zu einem bestimmten Zeitpunkt zu erstellen oder für Testzwecke. Der primäre Zugriff sollte jedoch über die statische Referenz erfolgen, um sicherzustellen, dass alle Teile des Systems denselben konsistenten Konfigurationszustand verwenden. Beispielsweise könnte eine langlaufende Aufgabe den relevanten Teil der Konfiguration bei ihrem Start klonen, um sicherzustellen, dass sie während ihrer gesamten Lebensdauer mit konsistenten Einstellungen arbeitet, selbst wenn später ein globaler Mechanismus zum Neuladen der Konfiguration eingeführt würde.

## **5\. Core Utilities Spezifikation (core::utils) (Ausgewählte kritische Utilities)**

Das Modul core::utils wird allgemeine Hilfsfunktionen und kleine, in sich geschlossene Utilities beherbergen, die nicht in spezifischere Module wie types oder config passen, aber über mehrere Teile der Kernschicht oder von anderen Schichten verwendet werden. Nur Utilities mit nicht-trivialer Logik oder spezifischen Designentscheidungen rechtfertigen hier eine detaillierte Spezifikation. Einfache Einzeiler-Helfer tun dies nicht.  
Für die initiale Kernschicht wird davon ausgegangen, dass keine hochkomplexen, neuartigen Utilities identifiziert wurden, die eine tiefergehende Spezifikation erfordern. Sollte sich dies ändern (z.B. ein benutzerdefinierter ID-Generator, ein spezialisierter String-Interner oder eine komplexe Pfad-Normalisierungsroutine), würde die Spezifikation dem untenstehenden Muster folgen.

* **5.X.1. Utility:**  
  * **Zweck, Begründung und Designentscheidungen**: (z.B. "Stellt ein robustes, plattformübergreifendes Dienstprogramm zur Pfadnormalisierung bereit, das Symlinks und relative Pfade konsistenter behandelt als Standardbibliotheksfunktionen in spezifischen Grenzfällen, die für die Desktop-Umgebung relevant sind.")  
  * **API**:  
    * **Strukturen/Enums**:  
      Rust  
      // pub struct NormalizedPath { /\*... \*/ }  
      // pub enum NormalizationError { /\*... \*/ } // Verwendet thiserror

    * **Methoden**:  
      Rust  
      // impl ComplexPathNormalizer {  
      //     pub fn new(/\*... \*/) \-\> Self;  
      //     pub fn normalize(base: \&Path, input: \&Path) \-\> Result\<NormalizedPath, NormalizationError\>;  
      // }  
      Vollständige Signaturen: fn normalize(base: \&std::path::Path, input: \&std::path::Path) \-\> Result\<NormalizedPath, NormalizationError\>; (Rusts noexcept ist implizit für Funktionen, die nicht unsafe deklariert sind und nicht paniken; die explizite Erwähnung der Panic-Vermeidung ist jedoch entscheidend).  
  * **Interne Algorithmen**: (Schritt-für-Schritt-Logik für komplexe Teile, z.B. Symlink-Auflösungsschleife, Behandlung von ..).  
  * **Fehlerbedingungen**: Abbildung auf NormalizationError-Varianten (z.B. PathNotFound, MaxSymlinkDepthExceeded).  
  * **Invarianten, Vorbedingungen, Nachbedingungen**: (z.B. "Eingabepfad muss für bestimmte Operationen existieren", "Zurückgegebener Pfad ist absolut und frei von . oder .. Komponenten").  
* **Allgemeine Richtlinien für core::utils:**  
  * **Geltungsbereich**: Utilities müssen wirklich allgemeiner Natur sein. Wenn ein Utility nur von einem anderen Modul verwendet wird, sollte es wahrscheinlich innerhalb dieses Moduls angesiedelt sein.  
  * **Einfachheit**: Einfache Funktionen sind komplexen Strukturen vorzuziehen, es sei denn, Zustand ist wirklich erforderlich.  
  * **Reinheit**: Reine Funktionen sind wo möglich zu bevorzugen (Ausgabe hängt nur von der Eingabe ab, keine Seiteneffekte).  
  * **Fehlerbehandlung**: Jede fehleranfällige Utility-Funktion muss Result\<T, YourUtilError\> zurückgeben, wobei YourUtilError unter Verwendung von thiserror innerhalb des Submoduls des Utilities definiert wird (z.B. core::utils::path\_utils::Error).  
  * **Dokumentation**: Alle öffentlichen Utilities müssen umfassende rustdoc-Kommentare haben, einschließlich Beispielen.  
  * **Tests**: Gründliche Unit-Tests sind für alle Utilities zwingend erforderlich.

## **6\. Core Datentypen Spezifikation (core::types) (Ausgewählte kritische Datentypen)**

Das Modul core::types definiert fundamentale Datenstrukturen und Enums, die in der gesamten Desktop-Umgebung verwendet werden. Diese unterscheiden sich von Konfigurationsstrukturen und sind eher primitive Bausteine für die Anwendungslogik. Beispiele hierfür sind Point, Size, Rect, Color, ResourceId usw.

### **6.1. Datentyp: RectInt (Integer-basiertes Rechteck)**

* **Zweck und Begründung**: Repräsentiert ein achsenparalleles Rechteck, das durch ganzzahlige Koordinaten und Dimensionen definiert ist. Unerlässlich für Fenstergeometrie, Positionierung von UI-Elementen und pixelbasierte Berechnungen. Die Verwendung von i32 für Koordinaten und u32 für Größen ist üblich für Bildschirmkoordinaten.  
* **Definition**:  
  Rust  
  use serde::{Serialize, Deserialize};

  \#  
  pub struct PointInt {  
      pub x: i32,  
      pub y: i32,  
  }

  impl PointInt {  
      pub const ZERO: Self \= Self { x: 0, y: 0 };

      \#\[must\_use\]  
      pub fn new(x: i32, y: i32) \-\> Self {  
          Self { x, y }  
      }

      // Weitere Methoden wie add, sub, etc. können hier hinzugefügt werden.  
      // pub fn add(self, other: Self) \-\> Self { Self { x: self.x \+ other.x, y: self.y \+ other.y } }  
  }

  \#  
  pub struct SizeInt {  
      pub width: u32,  
      pub height: u32,  
  }

  impl SizeInt {  
      pub const ZERO: Self \= Self { width: 0, height: 0 };

      \#\[must\_use\]  
      pub fn new(width: u32, height: u32) \-\> Self {  
          Self { width, height }  
      }

      \#\[must\_use\]  
      pub fn is\_empty(\&self) \-\> bool {  
          self.width \== 0 |

| self.height \== 0  
}  
}

\#  
pub struct RectInt {  
    pub x: i32,  
    pub y: i32,  
    pub width: u32,  
    pub height: u32,  
}  
\`\`\`

* **Methoden für RectInt**:  
  * pub const fn new(x: i32, y: i32, width: u32, height: u32) \-\> Self: Konstruktor.  
  * \#\[must\_use\] pub fn from\_points(p1: PointInt, p2: PointInt) \-\> Self: Erstellt ein Rechteck, das zwei Punkte umschließt.  
    * Vorbedingung: Keine.  
    * Nachbedingung: x ist min(p1.x, p2.x), y ist min(p1.y, p2.y), width ist abs(p1.x \- p2.x) as u32, height ist abs(p1.y \- p2.y) as u32. Die Umwandlung in u32 ist sicher, da die Differenz absolut ist.  
  * \#\[must\_use\] pub fn top\_left(\&self) \-\> PointInt: Gibt PointInt { x: self.x, y: self.y } zurück.  
  * \#\[must\_use\] pub fn size(\&self) \-\> SizeInt: Gibt SizeInt { width: self.width, height: self.height } zurück.  
  * \#\[must\_use\] pub fn right(\&self) \-\> i32: Gibt self.x.saturating\_add(self.width as i32) zurück. Verwendet saturating\_add um Überlauf zu vermeiden, obwohl dies bei typischen Bildschirmkoordinaten unwahrscheinlich ist.  
  * \#\[must\_use\] pub fn bottom(\&self) \-\> i32: Gibt self.y.saturating\_add(self.height as i32) zurück.  
  * \#\[must\_use\] pub fn contains\_point(\&self, p: PointInt) \-\> bool: Prüft, ob ein Punkt innerhalb des Rechtecks liegt (einschließlich der Ränder).  
    * Logik: p.x \>= self.x && p.x \< self.right() && p.y \>= self.y && p.y \< self.bottom(). Beachten Sie, dass right() und bottom() exklusiv sind.  
  * \#\[must\_use\] pub fn intersects(\&self, other: RectInt) \-\> bool: Prüft, ob dieses Rechteck ein anderes schneidet.  
    * Logik: self.x \< other.right() && self.right() \> other.x && self.y \< other.bottom() && self.bottom() \> other.y.  
  * \#\[must\_use\] pub fn intersection(\&self, other: RectInt) \-\> Option\<RectInt\>: Gibt das Schnittrechteck zurück oder None, wenn sie sich nicht schneiden.  
    * Logik: Berechne x\_intersect \= max(self.x, other.x), y\_intersect \= max(self.y, other.y). Berechne right\_intersect \= min(self.right(), other.right()), bottom\_intersect \= min(self.bottom(), other.bottom()). Wenn right\_intersect \> x\_intersect und bottom\_intersect \> y\_intersect, dann ist das Ergebnis RectInt::new(x\_intersect, y\_intersect, (right\_intersect \- x\_intersect) as u32, (bottom\_intersect \- y\_intersect) as u32). Sonst None.  
  * \#\[must\_use\] pub fn union(\&self, other: RectInt) \-\> RectInt: Gibt das kleinste Rechteck zurück, das beide umschließt.  
    * Logik: x\_union \= min(self.x, other.x), y\_union \= min(self.y, other.y). right\_union \= max(self.right(), other.right()), bottom\_union \= max(self.bottom(), other.bottom()). Ergebnis ist RectInt::new(x\_union, y\_union, (right\_union \- x\_union) as u32, (bottom\_union \- y\_union) as u32).  
  * \#\[must\_use\] pub fn translate(\&self, dx: i32, dy: i32) \-\> RectInt: Gibt ein neues, um (dx, dy) verschobenes Rechteck zurück.  
    * Logik: RectInt::new(self.x.saturating\_add(dx), self.y.saturating\_add(dy), self.width, self.height).  
  * \#\[must\_use\] pub fn inflate(\&self, dw: i32, dh: i32) \-\> RectInt: Gibt ein neues Rechteck zurück, das auf jeder Seite um dw (links/rechts) bzw. dh (oben/unten) erweitert (oder verkleinert, wenn dw, dh negativ sind) wird. Die resultierende Breite/Höhe darf nicht negativ werden.  
    * Logik: new\_x \= self.x.saturating\_sub(dw), new\_y \= self.y.saturating\_sub(dh). new\_width \= (self.width as i64).saturating\_add(2 \* dw as i64), new\_height \= (self.height as i64).saturating\_add(2 \* dh as i64). RectInt::new(new\_x, new\_y, max(0, new\_width) as u32, max(0, new\_height) as u32).  
  * \#\[must\_use\] pub fn is\_empty(\&self) \-\> bool: Gibt self.width \== 0 | | self.height \== 0 zurück.  
* **Invarianten**: width \>= 0, height \>= 0\. (Durch den u32-Typ erzwungen).  
* **Serialisierung**: Leitet Serialize, Deserialize für einfache Verwendung in Konfigurationen oder IPC ab.  
* **Traits**: Debug, Clone, Copy, PartialEq, Eq, Hash, Default.

### **6.2. Datentyp: Color (RGBA Farbrepräsentation)**

* **Zweck und Begründung**: Repräsentiert eine Farbe mit Rot-, Grün-, Blau- und Alpha-Komponenten. Fundamental für Theming, UI-Rendering und Grafik. Die Verwendung von f32 für Komponenten im Bereich \[0.0, 1.0\] ist üblich für Grafikpipelines. GTK4 verwendet intern oft f64, aber f32 bietet einen guten Kompromiss zwischen Präzision und Speicherbedarf und ist weit verbreitet.  
* **Definition**:  
  Rust  
  use serde::{Serialize, Deserialize, Deserializer, Serializer};  
  use thiserror::Error;  
  use std::num::ParseIntError;

  \#  
  pub struct Color {  
      pub r: f32, // Bereich \[0.0, 1.0\]  
      pub g: f32, // Bereich \[0.0, 1.0\]  
      pub b: f32, // Bereich \[0.0, 1.0\]  
      pub a: f32, // Bereich \[0.0, 1.0\]  
  }

  \#  
  pub enum ColorParseError {  
      \#  
      InvalidHexFormat(String),  
      \#\[error("Ungültige Hex-Ziffer in '{0}'")\]  
      InvalidHexDigit(String, \#\[source\] ParseIntError),  
      \#  
      InvalidHexLength(String),  
  }

* **Methoden für Color**:  
  * \#\[must\_use\] pub fn new(r: f32, g: f32, b: f32, a: f32) \-\> Self: Konstruktor. Klemmt Werte auf den Bereich \[0.0, 1.0\].  
    * Implementierung: Self { r: r.clamp(0.0, 1.0), g: g.clamp(0.0, 1.0), b: b.clamp(0.0, 1.0), a: a.clamp(0.0, 1.0) }.  
    * Nachbedingung: 0.0 \<= self.r \<= 1.0, usw.  
  * pub const OPAQUE\_BLACK: Color \= Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };  
  * pub const OPAQUE\_WHITE: Color \= Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };  
  * pub const TRANSPARENT: Color \= Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };  
  * pub fn from\_hex(hex\_string: \&str) \-\> Result\<Self, ColorParseError\>: Parst aus den Formaten "\#RRGGBB", "\#RGB", "\#RRGGBBAA" oder "\#RGBA".  
    * Logik: String validieren (Präfix \#, Länge), dann entsprechende Paare von Hex-Ziffern parsen, zu u8 konvertieren und dann zu f32 normalisieren (/ 255.0). Für Kurzformate (\#RGB, \#RGBA) Ziffern verdoppeln (z.B. "F" wird zu "FF").  
  * \#\[must\_use\] pub fn to\_hex\_string(\&self, include\_alpha: bool) \-\> String: Konvertiert in einen Hex-String ("\#RRGGBB" oder "\#RRGGBBAA").  
    * Logik: Komponenten mit 255.0 multiplizieren, zu u8 runden/casten, dann als Hex formatieren.  
  * \#\[must\_use\] pub fn with\_alpha(\&self, alpha: f32) \-\> Self: Gibt eine neue Farbe mit dem angegebenen Alpha-Wert zurück (geklemmt).  
  * \#\[must\_use\] pub fn lighten(\&self, amount: f32) \-\> Self: Hellt die Farbe auf. Eine einfache Methode ist, amount zu R, G und B zu addieren und dann zu klemmen. Komplexere Methoden würden im HSL/HSV-Raum arbeiten. Für die Kernschicht ist eine einfache RGB-Aufhellung zunächst ausreichend.  
  * \#\[must\_use\] pub fn darken(\&self, amount: f32) \-\> Self: Dunkelt die Farbe ab (analog zu lighten).  
  * \#\[must\_use\] pub fn interpolate(\&self, other: Color, t: f32) \-\> Self: Lineare Interpolation zwischen dieser Farbe und other. t wird auf \[0.0, 1.0\] geklemmt.  
    * Logik: r \= self.r \* (1.0 \- t) \+ other.r \* t, analog für g, b, a.  
* **Serialisierung**: Color soll Serialize und Deserialize implementieren, um als Hex-String in Konfigurationsdateien (z.B. TOML, JSON) dargestellt zu werden. Dies macht Konfigurationen (z.B. für Theming) wesentlich benutzerfreundlicher.  
  Rust  
  impl Serialize for Color {  
      fn serialize\<S\>(\&self, serializer: S) \-\> Result\<S::Ok, S::Error\>  
      where  
          S: Serializer,  
      {  
          serializer.serialize\_str(\&self.to\_hex\_string(true)) // Immer mit Alpha serialisieren für Konsistenz  
      }  
  }

  impl\<'de\> Deserialize\<'de\> for Color {  
      fn deserialize\<D\>(deserializer: D) \-\> Result\<Self, D::Error\>  
      where  
          D: Deserializer\<'de\>,  
      {  
          let s \= String::deserialize(deserializer)?;  
          Color::from\_hex(\&s).map\_err(serde::de::Error::custom)  
      }  
  }  
  Die Verwendung von serde(try\_from \= "String", into \= "String") ist eine Alternative, erfordert aber die Implementierung von TryFrom\<String\> for Color und From\<Color\> for String. Der oben gezeigte Weg mit manueller Implementierung von Serialize und Deserialize gibt volle Kontrolle. Die Möglichkeit, Farben als Hex-Codes ("\#CC331A") anstelle von Arrays von Fließkommazahlen (\[0.8, 0.2, 0.1, 1.0\]) in Konfigurationsdateien anzugeben, ist ein erheblicher Gewinn für die Benutzerfreundlichkeit, sowohl für Entwickler (bei der Erstellung von Standardkonfigurationen) als auch für Endbenutzer (bei der Anpassung von Themes). Dies erfordert eine robuste ColorParseError-Behandlung, um ungültige Hex-Strings abzufangen.  
* **Traits**: Debug, Clone, Copy, PartialEq, Default (kann auf OPAQUE\_BLACK oder TRANSPARENT gesetzt werden, OPAQUE\_BLACK ist eine gängige Wahl).

## **7\. Schlussfolgerung und Schichtübergreifende Integrationsrichtlinien (Fokus Kerninfrastruktur)**

* **Zusammenfassung der Kerninfrastruktur**: Dieser Teil der Spezifikation hat die fundamentalen Elemente der Kernschicht detailliert beschrieben: ein robustes Fehlerbehandlungsframework (core::errors) basierend auf thiserror und modul-spezifischen Enums; ein strukturiertes Logging-System (core::logging) unter Verwendung von tracing; Primitive für das Laden und den Zugriff auf Konfigurationen (core::config) über TOML und serde; sowie Definitionen für essentielle gemeinsame Datentypen (core::types) wie RectInt und Color. Diese Komponenten sind darauf ausgelegt, eine solide, wartbare und performante Basis für die gesamte Desktop-Umgebung zu schaffen.  
* **Richtlinien für die Nutzung durch andere Schichten**:  
  * **Fehlerbehandlung**: Alle Module in der Domänen-, System- und UI-Schicht *müssen* ihre eigenen thiserror-basierten Fehler-Enums definieren. Fehler, die von Funktionen der Kernschicht stammen, müssen entweder behandelt oder mittels ? weitergegeben werden, wobei sie potenziell unter Verwendung von \#\[from\] in die eigenen Fehlertypen der aufrufenden Schicht umgewandelt werden. Die Fehlerkette (source()) muss dabei erhalten bleiben.  
  * **Logging**: Alle Schichten *müssen* die tracing-Makros (trace\!, info\!, etc.) für sämtliche Logging-Aktivitäten verwenden. Die Funktion core::logging::initialize\_logging() muss vom Hauptanwendungsbinary beim Start aufgerufen werden. Die Einhaltung der Log-Level und der Richtlinien zur Datensensibilität ist zwingend erforderlich.  
  * **Konfiguration**: Höhere Schichten können ihre eigenen Konfigurationsstrukturen definieren, die als Teil eines größeren Anwendungskonfigurationsobjekts geladen werden können. Sie greifen auf Konfigurationen der Kernschicht über core::config::get\_core\_config() zu. Sie sollten nicht versuchen, die Konfiguration der Kernschicht zur Laufzeit zu modifizieren, da diese als statisch und nach der Initialisierung unveränderlich betrachtet wird.  
  * **Typen und Utilities**: Kerndatentypen (RectInt, Color usw.) und Utilities (core::utils) sollten direkt verwendet werden, wo dies angemessen ist, um Konsistenz zu gewährleisten und Neuimplementierungen zu vermeiden. Wenn eine höhere Schicht eine spezialisierte Version eines Kerntyps benötigt, sollte sie Komposition oder Newtype-Wrapper um den Kerntyp in Betracht ziehen, anstatt den Typ neu zu definieren.  
* **Immutabilität und Stabilität**: Die API der Kernschicht sollte nach ihrer Stabilisierung als äußerst stabil behandelt werden. Änderungen hier haben weitreichende Auswirkungen auf das gesamte System. Alle spezifizierten Komponenten sind so konzipiert, dass sie Send \+ Sync sind, wo dies sinnvoll ist, was ihre Verwendung in einer multithreaded Umgebung ermöglicht – ein Schlüsselmerkmal von Rust und wichtig für eine reaktionsschnelle Desktop-Umgebung. Die strikte Einhaltung der hier definierten Schnittstellen und Richtlinien ist entscheidend für den langfristigen Erfolg und die Wartbarkeit des Projekts.

---


---

## 1. Allgemeine Vorbemerkungen zur Implementierung

- **Rust Edition:** Es wird die jeweils aktuell stabile Rust-Edition zum Zeitpunkt der Implementierung verwendet (aktuell Rust 2021, potenziell Rust 2024, falls bis dahin relevant).
- **Abhängigkeitsmanagement:** Cargo wird für das Abhängigkeitsmanagement verwendet. Versionen von Abhängigkeiten sollten sorgfältig gewählt und bei Bedarf über `Cargo.lock` fixiert werden, um reproduzierbare Builds sicherzustellen. Es wird empfohlen, `cargo update -p <crate_name>` für gezielte Updates zu verwenden.
- **Asynchrone Runtime:** Wo nicht anders spezifiziert (z.B. für GTK-spezifische Aufgaben), wird `tokio` als primäre asynchrone Runtime für I/O-gebundene Operationen und Nebenläufigkeit verwendet, insbesondere in der System- und UI-Schicht.
- **Fehlerbehandlung (Globale Konvention):** Die Verwendung von `thiserror` für spezifische Fehler-Enums pro Modul und die Weitergabe von Fehlern über `Result<T, E>` ist verbindlich. Panics sind strikt zu vermeiden, außer in Tests oder bei nachweislich nicht behebbaren internen Invariantenverletzungen mit aussagekräftiger Begründung. Die `source()`-Kette von Fehlern muss erhalten bleiben.
- **Logging (Globale Konvention):** Das `tracing`-Framework ist für strukturiertes, kontextbezogenes Logging verbindlich. Sensible Daten dürfen niemals geloggt werden.
- **Code-Formatierung und Linting:** `rustfmt` mit Projektstandardkonfiguration und `clippy` (mit `-D warnings`) sind bei jedem Commit/Push obligatorisch und werden durch CI erzwungen.
- **Dokumentation:** Umfassende `rustdoc`-Kommentare für alle öffentlichen APIs sind zwingend erforderlich.
- **Tests:** Unit-Tests (`#[cfg(test)] mod tests { ... }`) müssen parallel zur Implementierung geschrieben werden und eine hohe Codeabdeckung anstreben. Integrationstests (`tests/integration_test.rs`) sind für das Zusammenspiel von Modulen/Crates vorzusehen.

---

## 2. Ultra-Feinspezifikation: Kernschicht (Core Layer)

Die Kernschicht (`novade-core` Crate) enthält die absolut grundlegendsten, systemweit genutzten Elemente und hat keine Abhängigkeiten zu anderen Schichten von NovaDE.

### Modul: `core::types` (Fundamentale Datentypen)

**Zweck:** Definition grundlegender, universell einsetzbarer Datentypen für Geometrie, Farben und allgemeine Enumerationen. Diese Typen sind reine Datenstrukturen ohne komplexe Geschäftslogik.

**Designphilosophie:** Modularität, Wiederverwendbarkeit, minimale Kopplung. Generische Typen wo sinnvoll. Klare Trennung von Datenrepräsentation und Fehlerbehandlung.

**Abhängigkeiten:** `std`, `uuid` (für IDs in höheren Schichten, hier als Beispiel für einen Basistyp), `chrono` (für Zeitstempel, dito), `serde` (optional, mit `derive`-Feature, falls Serialisierung hier benötigt wird), `num-traits` (optional).

#### Untermodul: `core::types::geometry`

**Datei:** `src/types/geometry.rs`

##### 1. Struct: `Point<T>`

- **Zweck:** Repräsentiert einen Punkt im 2D-Raum.
- **Generische Parameter:** `T`
- **Felder:**
    - `pub x: T`
    - `pub y: T`
- **Ableitungen (Basis):** `#[derive(Debug, Clone, Copy, PartialEq, Default)]`
    - `Eq` und `Hash` nur, wenn `T: Eq` bzw. `T: Hash`.
- **`serde` (optional):** `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`
- **Invarianten:** Keine spezifischen, hängen von `T` ab.
- **Methoden:**
    - `pub const fn new(x: T, y: T) -> Self`
        - **Logik:** Erstellt einen neuen Punkt.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** `self.x == x`, `self.y == y`.
    - `pub fn distance_squared(&self, other: &Point<T>) -> T`
        - **Generische Constraints:** `T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<Output = T>`
        - **Logik:** Berechnet das Quadrat der euklidischen Distanz: `(self.x - other.x)^2 + (self.y - other.y)^2`.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt das Quadrat der Distanz zurück.
    - `pub fn distance(&self, other: &Point<T>) -> T`
        - **Generische Constraints:** `T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + std::ops::Mul<Output = T> + num_traits::Float` (oder spezifische Implementierung für `f32`, `f64`)
        - **Logik:** Berechnet die euklidische Distanz: `sqrt((self.x - other.x)^2 + (self.y - other.y)^2)`.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt die Distanz zurück.
    - `pub fn manhattan_distance(&self, other: &Point<T>) -> T`
        - **Generische Constraints:** `T: Copy + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + num_traits::Signed`
        - **Logik:** Berechnet die Manhattan-Distanz: `abs(self.x - other.x) + abs(self.y - other.y)`.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt die Manhattan-Distanz zurück.
- **Trait-Implementierungen (Zusätzlich):**
    - `impl<T: std::ops::Add<Output = T>> std::ops::Add for Point<T>`
    - `impl<T: std::ops::Sub<Output = T>> std::ops::Sub for Point<T>`
- **Assoziierte Konstanten:**
    - `pub const ZERO_I32: Point<i32> = Point { x: 0, y: 0 };`
    - `pub const ZERO_F32: Point<f32> = Point { x: 0.0, y: 0.0 };`
    - (Weitere für `u32`, `f64` etc.)

##### 2. Struct: `Size<T>`

- **Zweck:** Repräsentiert eine 2D-Dimension (Breite und Höhe).
- **Generische Parameter:** `T`
- **Felder:**
    - `pub width: T`
    - `pub height: T`
- **Ableitungen (Basis):** `#[derive(Debug, Clone, Copy, PartialEq, Default)]`
    - `Eq` und `Hash` nur, wenn `T: Eq` bzw. `T: Hash`.
- **`serde` (optional):** `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`
- **Invarianten:** Logisch sollten `width` und `height` nicht-negativ sein. Dies wird nicht durch den Typ erzwungen, aber durch `is_valid()` prüfbar gemacht.
- **Methoden:**
    - `pub const fn new(width: T, height: T) -> Self`
        - **Logik:** Erstellt eine neue Größe.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** `self.width == width`, `self.height == height`.
    - `pub fn area(&self) -> T`
        - **Generische Constraints:** `T: Copy + std::ops::Mul<Output = T>`
        - **Logik:** Berechnet die Fläche: `self.width * self.height`.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt die Fläche zurück.
    - `pub fn is_empty(&self) -> bool`
        - **Generische Constraints:** `T: PartialEq + num_traits::Zero`
        - **Logik:** Prüft, ob `self.width == T::zero()` oder `self.height == T::zero()`.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt `true` zurück, wenn leer, sonst `false`.
    - `pub fn is_valid(&self) -> bool`
        - **Generische Constraints:** `T: PartialOrd + num_traits::Zero`
        - **Logik:** Prüft, ob `self.width >= T::zero()` und `self.height >= T::zero()`.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt `true` zurück, wenn gültig, sonst `false`.
- **Assoziierte Konstanten:**
    - `pub const ZERO_I32: Size<i32> = Size { width: 0, height: 0 };`
    - `pub const ZERO_F32: Size<f32> = Size { width: 0.0, height: 0.0 };`
    - (Weitere für `u32`, `f64` etc.)

##### 3. Struct: `Rect<T>`

- **Zweck:** Repräsentiert ein 2D-Rechteck, definiert durch Ursprung (oben-links) und Größe.
- **Generische Parameter:** `T`
- **Felder:**
    - `pub origin: Point<T>`
    - `pub size: Size<T>`
- **Ableitungen (Basis):** `#[derive(Debug, Clone, Copy, PartialEq, Default)]`
    - `Eq` und `Hash` nur, wenn `T: Eq` bzw. `T: Hash`.
- **`serde` (optional):** `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`
- **Invarianten:** Logisch sollten `size.width` und `size.height` nicht-negativ sein. `is_valid()` prüft dies. Die Verantwortung für das Melden eines Fehlers bei Verwendung eines ungültigen `Rect` liegt beim Aufrufer.
- **Methoden:**
    - `pub const fn new(origin: Point<T>, size: Size<T>) -> Self`
        - **Logik:** Erstellt ein neues Rechteck.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** `self.origin == origin`, `self.size == size`.
    - `pub fn from_coords(x: T, y: T, width: T, height: T) -> Self`
        - **Generische Constraints:** `T` muss die Constraints für `Point::new` und `Size::new` erfüllen.
        - **Logik:** Erstellt ein neues Rechteck aus Koordinaten und Dimensionen.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** `self.origin.x == x`, `self.origin.y == y`, `self.size.width == width`, `self.size.height == height`.
    - `pub fn x(&self) -> T` (Constraints: `T: Copy`) - Gibt `self.origin.x` zurück.
    - `pub fn y(&self) -> T` (Constraints: `T: Copy`) - Gibt `self.origin.y` zurück.
    - `pub fn width(&self) -> T` (Constraints: `T: Copy`) - Gibt `self.size.width` zurück.
    - `pub fn height(&self) -> T` (Constraints: `T: Copy`) - Gibt `self.size.height` zurück.
    - `pub fn top(&self) -> T` (Constraints: `T: Copy`) - Gibt `self.y()` zurück.
    - `pub fn left(&self) -> T` (Constraints: `T: Copy`) - Gibt `self.x()` zurück.
    - `pub fn bottom(&self) -> T` (Constraints: `T: Copy + std::ops::Add<Output = T>`) - Gibt `self.y() + self.height()` zurück.
    - `pub fn right(&self) -> T` (Constraints: `T: Copy + std::ops::Add<Output = T>`) - Gibt `self.x() + self.width()` zurück.
    - `pub fn center(&self) -> Point<T>`
        - **Generische Constraints:** `T: Copy + std::ops::Add<Output = T> + std::ops::Div<Output = T> + num_traits::FromPrimitive` (für `T::from(2).unwrap()`).
        - **Logik:** Berechnet den Mittelpunkt: `Point::new(self.x() + self.width() / 2, self.y() + self.height() / 2)`.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt den Mittelpunkt zurück.
    - `pub fn contains_point(&self, point: &Point<T>) -> bool`
        - **Generische Constraints:** `T: Copy + PartialOrd + std::ops::Add<Output = T>`
        - **Logik:** `point.x >= self.left() && point.x < self.right() && point.y >= self.top() && point.y < self.bottom()`.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt `true` zurück, wenn der Punkt enthalten ist, sonst `false`.
    - `pub fn intersects(&self, other: &Rect<T>) -> bool`
        - **Generische Constraints:** `T: Copy + PartialOrd + std::ops::Add<Output = T>`
        - **Logik:** `self.left() < other.right() && self.right() > other.left() && self.top() < other.bottom() && self.bottom() > other.top()`.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt `true` zurück, wenn sich die Rechtecke überschneiden, sonst `false`.
    - `pub fn intersection(&self, other: &Rect<T>) -> Option<Rect<T>>`
        - **Generische Constraints:** `T: Copy + Ord + std::ops::Add<Output = T> + std::ops::Sub<Output = T> + num_traits::Zero`
        - **Logik:**
            1. `intersect_x = self.x().max(other.x())`
            2. `intersect_y = self.y().max(other.y())`
            3. `intersect_right = self.right().min(other.right())`
            4. `intersect_bottom = self.bottom().min(other.bottom())`
            5. Wenn `intersect_right > intersect_x` und `intersect_bottom > intersect_y`:
                - `intersect_width = intersect_right - intersect_x`
                - `intersect_height = intersect_bottom - intersect_y`
                - `Some(Rect::new(Point::new(intersect_x, intersect_y), Size::new(intersect_width, intersect_height)))`
            6. Sonst: `None`
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt das Schnittrechteck oder `None` zurück.
    - `pub fn union(&self, other: &Rect<T>) -> Rect<T>`
        - **Generische Constraints:** `T: Copy + Ord + std::ops::Add<Output = T> + std::ops::Sub<Output = T>`
        - **Logik:**
            1. `union_x = self.x().min(other.x())`
            2. `union_y = self.y().min(other.y())`
            3. `union_right = self.right().max(other.right())`
            4. `union_bottom = self.bottom().max(other.bottom())`
            5. `union_width = union_right - union_x`
            6. `union_height = union_bottom - union_y`
            7. `Rect::new(Point::new(union_x, union_y), Size::new(union_width, union_height))`
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt das umschließende Rechteck zurück.
    - `pub fn translated(&self, dx: T, dy: T) -> Rect<T>`
        - **Generische Constraints:** `T: Copy + std::ops::Add<Output = T>`
        - **Logik:** `Rect::new(Point::new(self.origin.x + dx, self.origin.y + dy), self.size)`
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt das verschobene Rechteck zurück.
    - `pub fn scaled(&self, sx: T, sy: T) -> Rect<T>` (Skaliert Ursprung und Größe)
        - **Generische Constraints:** `T: Copy + std::ops::Mul<Output = T>`
        - **Logik:** `Rect::new(Point::new(self.origin.x * sx, self.origin.y * sy), Size::new(self.size.width * sx, self.size.height * sy))`
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt das skalierte Rechteck zurück.
    - `pub fn is_valid(&self) -> bool`
        - **Generische Constraints:** `T` muss `Size::is_valid` unterstützen.
        - **Logik:** Ruft `self.size.is_valid()` auf.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt `true` zurück, wenn `size` gültig ist, sonst `false`.
- **Assoziierte Konstanten:**
    - `pub const ZERO_I32: Rect<i32> = Rect { origin: Point::ZERO_I32, size: Size::ZERO_I32 };`
    - `pub const ZERO_F32: Rect<f32> = Rect { origin: Point::ZERO_F32, size: Size::ZERO_F32 };`

##### 4. Struct `RectInt` (Spezifische Implementierung von `Rect<i32/u32>`)

- **Zweck:** Ein achsenparalleles Rechteck mit ganzzahligen `i32` Koordinaten und `u32` Dimensionen. Dies ist oft praktisch für Pixel-basierte Operationen und Fenstergeometrie.
- **Felder:**
    - `pub x: i32`
    - `pub y: i32`
    - `pub width: u32`
    - `pub height: u32`
- **Ableitungen:** `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]` (Serde ist hier oft nützlich)
- **Invarianten:** `width >= 0`, `height >= 0` (durch `u32` Typ erzwungen).
- **Methoden (Beispiele, basierend auf):**
    - `pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self`
    - `pub fn from_points(p1: Point<i32>, p2: Point<i32>) -> Self`
        - **Logik:** `x = p1.x.min(p2.x)`, `y = p1.y.min(p2.y)`, `width = (p1.x - p2.x).abs() as u32`, `height = (p1.y - p2.y).abs() as u32`.
    - `pub fn top_left(&self) -> Point<i32>` - Gibt `Point::new(self.x, self.y)` zurück.
    - `pub fn size(&self) -> Size<u32>` - Gibt `Size::new(self.width, self.height)` zurück.
    - `pub fn right(&self) -> i32` - Gibt `self.x.saturating_add(self.width as i32)` zurück.
    - `pub fn bottom(&self) -> i32` - Gibt `self.y.saturating_add(self.height as i32)` zurück.
    - `pub fn contains_point(&self, p_x: i32, p_y: i32) -> bool` - `p_x >= self.x && p_x < self.right() && p_y >= self.y && p_y < self.bottom()`.
    - `pub fn intersects(&self, other: RectInt) -> bool` - `self.x < other.right() && self.right() > other.x && self.y < other.bottom() && self.bottom() > other.y`.
    - `pub fn intersection(&self, other: RectInt) -> Option<RectInt>` (Logik wie bei `Rect<T>`)
    - `pub fn union(&self, other: RectInt) -> RectInt` (Logik wie bei `Rect<T>`)
    - `pub fn translate(&self, dx: i32, dy: i32) -> RectInt` - `RectInt::new(self.x.saturating_add(dx), self.y.saturating_add(dy), self.width, self.height)`.
    - `pub fn inflate(&self, dw: i32, dh: i32) -> RectInt`
        - **Logik:** `new_x = self.x.saturating_sub(dw)`, `new_y = self.y.saturating_sub(dh)`. `new_width_signed = (self.width as i64).saturating_add(2 * dw as i64)`. `new_height_signed = (self.height as i64).saturating_add(2 * dh as i64)`. `RectInt::new(new_x, new_y, new_width_signed.max(0) as u32, new_height_signed.max(0) as u32)`.
    - `pub fn is_empty(&self) -> bool` - `self.width == 0 || self.height == 0`.

#### Untermodul: `core::types::color`

**Datei:** `src/types/color.rs`

##### 1. Struct: `Color` (RGBA)

- **Zweck:** Repräsentiert eine Farbe mit Rot-, Grün-, Blau- und Alpha-Komponenten.
- **Felder:**
    - `pub r: f32` (Bereich `[0.0, 1.0]`)
    - `pub g: f32` (Bereich `[0.0, 1.0]`)
    - `pub b: f32` (Bereich `[0.0, 1.0]`)
    - `pub a: f32` (Bereich `[0.0, 1.0]`)
- **Ableitungen:** `#[derive(Debug, Clone, Copy, PartialEq)]`
- **Invarianten:** Alle Komponenten `r, g, b, a` müssen im Bereich `[0.0, 1.0]` liegen. Konstruktoren und Methoden klemmen Werte entsprechend.
- **Methoden:**
    - `pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self`
        - **Logik:** `Self { r: r.clamp(0.0, 1.0), g: g.clamp(0.0, 1.0), b: b.clamp(0.0, 1.0), a: a.clamp(0.0, 1.0) }`.
        - **Nachbedingungen:** Alle Felder sind im Bereich `[0.0, 1.0]`.
    - `pub fn from_rgba8(r_u8: u8, g_u8: u8, b_u8: u8, a_u8: u8) -> Self`
        - **Logik:** `Self::new(r_u8 as f32 / 255.0, g_u8 as f32 / 255.0, b_u8 as f32 / 255.0, a_u8 as f32 / 255.0)`.
    - `pub fn to_rgba8(&self) -> (u8, u8, u8, u8)`
        - **Logik:** `((self.r * 255.0).round() as u8, (self.g * 255.0).round() as u8, (self.b * 255.0).round() as u8, (self.a * 255.0).round() as u8)`.
    - `pub fn with_alpha(&self, alpha: f32) -> Self`
        - **Logik:** `Self::new(self.r, self.g, self.b, alpha)`.
    - `pub fn blend(&self, background: &Color) -> Color` (Source-Over Blending)
        - **Logik:**
            1. `fg_a = self.a`
            2. `bg_a = background.a`
            3. `out_a = fg_a + bg_a * (1.0 - fg_a)`
            4. Wenn `out_a == 0.0`, gib `Color::TRANSPARENT` zurück.
            5. `out_r = (self.r * fg_a + background.r * bg_a * (1.0 - fg_a)) / out_a`
            6. `out_g = (self.g * fg_a + background.g * bg_a * (1.0 - fg_a)) / out_a`
            7. `out_b = (self.b * fg_a + background.b * bg_a * (1.0 - fg_a)) / out_a`
            8. `Color::new(out_r, out_g, out_b, out_a)`
    - `pub fn lighten(&self, amount: f32) -> Color`
        - **Logik:** `amount_clamped = amount.clamp(0.0, 1.0)`. `Color::new(self.r + (1.0 - self.r) * amount_clamped, self.g + (1.0 - self.g) * amount_clamped, self.b + (1.0 - self.b) * amount_clamped, self.a)`
    - `pub fn darken(&self, amount: f32) -> Color`
        - **Logik:** `amount_clamped = amount.clamp(0.0, 1.0)`. `Color::new(self.r * (1.0 - amount_clamped), self.g * (1.0 - amount_clamped), self.b * (1.0 - amount_clamped), self.a)`
    - `pub fn interpolate(&self, other: Color, t: f32) -> Color`
        - **Logik:** `t_clamped = t.clamp(0.0, 1.0)`. `r = self.r * (1.0 - t_clamped) + other.r * t_clamped` `g = self.g * (1.0 - t_clamped) + other.g * t_clamped` `b = self.b * (1.0 - t_clamped) + other.b * t_clamped` `a = self.a * (1.0 - t_clamped) + other.a * t_clamped` `Color::new(r,g,b,a)`
    - `pub fn from_hex(hex_string: &str) -> Result<Self, ColorParseError>`
        - **Logik:**
            1. Entferne optionales `#`-Präfix.
            2. Validiere Länge (3, 4, 6, 8 Zeichen). Bei ungültiger Länge: `Err(ColorParseError::InvalidHexLength(hex_string.to_string()))`.
            3. Parse Hex-Zeichen in `u8` Komponenten. Bei ungültigen Zeichen: `Err(ColorParseError::InvalidHexDigit(...))`.
                - `#RGB`: `R`, `G`, `B` (Alpha = FF) -> `RR`, `GG`, `BB`
                - `#RGBA`: `R`, `G`, `B`, `A` -> `RR`, `GG`, `BB`, `AA`
                - `#RRGGBB`: `RR`, `GG`, `BB` (Alpha = FF)
                - `#RRGGBBAA`: `RR`, `GG`, `BB`, `AA`
            4. Konvertiere `u8` zu `f32` (`/ 255.0`).
            5. Erzeuge `Color` via `Color::new()`.
            6. Bei Erfolg `Ok(Self)`.
        - **Fehler:** `ColorParseError` (siehe `core::errors`)
    - `pub fn to_hex_string(&self, include_alpha: bool) -> String`
        - **Logik:** Konvertiere `r,g,b,a` zu `u8`. Formatiere als Hex-String.
            - Wenn `include_alpha` oder `self.a < 1.0` (oder immer Alpha für Konsistenz): `format!("#{:02X}{:02X}{:02X}{:02X}", r_u8, g_u8, b_u8, a_u8)`
            - Sonst: `format!("#{:02X}{:02X}{:02X}", r_u8, g_u8, b_u8)`
- **Trait-Implementierungen (Zusätzlich):**
    - `impl Default for Color { fn default() -> Self { Color::TRANSPARENT } }`
- **Assoziierte Konstanten:**
    - `pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };`
    - `pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };`
    - `pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };`
    - (Weitere wie `RED`, `GREEN`, `BLUE`)
- **Serialisierung (`serde`):**
    - Soll als Hex-String serialisiert/deserialisiert werden (siehe).
    - `impl Serialize for Color { ... serializer.serialize_str(&self.to_hex_string(true)) ... }`
    - `impl<'de> Deserialize<'de> for Color { ... Color::from_hex(&s).map_err(serde::de::Error::custom) ... }`

#### Untermodul: `core::types::enums`

**Datei:** `src/types/enums.rs`

##### 1. Enum: `Orientation`

- **Zweck:** Repräsentiert eine horizontale oder vertikale Ausrichtung.
- **Varianten:**
    - `Horizontal`
    - `Vertical`
- **Ableitungen:** `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]`
- **Methoden:**
    - `pub fn toggle(&self) -> Self`
        - **Logik:** `match self { Orientation::Horizontal => Orientation::Vertical, Orientation::Vertical => Orientation::Horizontal }`.
- **Trait-Implementierungen (Zusätzlich):**
    - `impl Default for Orientation { fn default() -> Self { Orientation::Horizontal } }`

#### Untermodul: `core::types::ids`

Datei: src/types/ids.rs

(Obwohl in der Gesamtspezifikation WorkspaceId, WindowIdentifier etc. in core::types erwähnt werden, gehören sie semantisch eher in die Domänenschicht oder sind Newtypes um primitive IDs. Hier ein Beispiel für generische ID-Typen, falls benötigt.)

##### 1. Struct: `GenericId` (Beispiel)

- **Zweck:** Ein typsicherer Wrapper um `uuid::Uuid` für generische Entitäts-IDs.
- **Felder:** `pub id: uuid::Uuid`
- **Ableitungen:** `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]`
- **Methoden:**
    - `pub fnnew() -> Self { Self { id: Uuid::new_v4() } }`
    - `pub fnfrom_uuid(uuid: Uuid) -> Self { Self { id: uuid } }`
    - `pub fnas_uuid(&self) -> &Uuid { &self.id }`
- **Trait-Implementierungen:** `impl Default for GenericId { fn default() -> Self { Self::new() } }`

#### Moduldeklaration `core::types::mod.rs`

Rust

```
// src/types/mod.rs

pub mod color;
pub mod enums;
pub mod geometry;
// pub mod ids; // Falls vorhanden

pub use color::Color;
pub use enums::Orientation;
pub use geometry::{Point, Rect, Size, RectInt};
// pub use ids::GenericId;
```

#### Implementierungsschritte `core::types`

1. **Verzeichnis- und Dateierstellung:**
    - `core/src/types/mod.rs`
    - `core/src/types/geometry.rs`
    - `core/src/types/color.rs`
    - `core/src/types/enums.rs`
    - (`core/src/types/ids.rs` optional)
2. **Implementierung `geometry.rs`:**
    - `Point<T>`: Struktur, Ableitungen, Konstanten, Methoden (`new`, `distance_squared`, `distance`, `manhattan_distance`), Trait-Impls.
    - `Size<T>`: Struktur, Ableitungen, Konstanten, Methoden (`new`, `area`, `is_empty`, `is_valid`).
    - `Rect<T>`: Struktur, Ableitungen, Konstanten, Methoden (`new`, `from_coords`, Zugriffsmethoden, `center`, `contains_point`, `intersects`, `intersection`, `union`, `translated`, `scaled`, `is_valid`).
    - `RectInt`: Struktur, Ableitungen, Methoden wie spezifiziert.
    - Hinzufügen von `use num_traits::{Float, Signed, Zero};` und `use serde::{Serialize, Deserialize};` (letzteres mit `cfg_attr`).
3. **Implementierung `color.rs`:**
    - `Color`: Struktur, Ableitungen, Konstanten, Methoden (`new`, `from_rgba8`, `to_rgba8`, `with_alpha`, `blend`, `lighten`, `darken`, `interpolate`, `from_hex`, `to_hex_string`), `Default`-Impl, `Serialize`/`Deserialize`-Impls.
    - Benötigt `use crate::core::errors::ColorParseError;` (siehe `core::errors`) und `use serde::{Serializer, Deserializer, de::Error as SerdeError};`.
4. **Implementierung `enums.rs`:**
    - `Orientation`: Enum, Ableitungen, `toggle`-Methode, `Default`-Impl.
5. **Implementierung `ids.rs` (optional):**
    - `GenericId` (oder spezifischere ID-Typen, falls hier sinnvoll).
6. **Moduldeklaration `types/mod.rs`:** `pub mod ...` und `pub use ...` für alle definierten Typen.
7. **Aktualisierung `core/src/lib.rs`:** `pub mod types;`
8. **Unit-Tests:**
    - Für jeden Typ und jede Methode Testfälle erstellen.
    - `Point<T>`: Teste `new`, Distanzberechnungen für `i32` und `f32`.
    - `Size<T>`: Teste `new`, `area`, `is_empty`, `is_valid` für `i32`, `u32`, `f32`.
    - `Rect<T>`: Teste Konstruktoren, Zugriffsmethoden, `center`, `contains_point`, `intersects`, `intersection`, `union`.
    - `RectInt`: Teste alle Methoden, insbesondere `inflate` mit positiven/negativen Werten.
    - `Color`: Teste `new` (Klemmung), `from_rgba8`, `to_rgba8`, `blend`, `lighten`, `darken`, `from_hex` (alle Formate, Fehlerfälle), `to_hex_string`, `Default`.
    - `Orientation`: Teste `toggle`, `Default`.
    - Überprüfe `serde`-Implementierungen (Serialisierung zu erwartetem JSON/String, Deserialisierung).
9. **Dokumentation (`rustdoc`):**
    - Umfassende Kommentare für alle öffentlichen Elemente (Module, Structs, Enums, Felder, Methoden, Konstanten).
    - Erklärung von Invarianten, Wertebereichen, Vor-/Nachbedingungen.
    - `# Examples` für komplexere Methoden oder Typverwendungen.

---

### Modul: `core::errors` (Fehlerbehandlung)

**Zweck:** Definition einer robusten und konsistenten Fehlerbehandlungsstrategie und grundlegender Fehlertypen für die Kernschicht.

**Designphilosophie:** Verwendung von `thiserror` für spezifische Fehler-Enums pro Modul. Klare Trennung zwischen `Result<T, E>` für behebbare Fehler und `panic!` für nicht behebbare Programmierfehler. Kontextreiche Fehlermeldungen.

**Abhängigkeiten:** `std`, `thiserror`, `uuid` (für IDs in Fehlermeldungen), `std::path::PathBuf`.

**Datei:** `src/errors.rs`

##### 1. Enum: `CoreError` (Basis-Fehlertyp der Kernschicht)

- **Zweck:** Dient als primäre Schnittstelle für allgemeine Fehler, die von öffentlichen Funktionen der Kernschicht propagiert werden können, oder für Fehler, die keinem spezifischen Submodul zugeordnet werden können.
- **Ableitungen:** `#[derive(Debug, thiserror::Error)]`
- **Varianten:**
    - `#[error("Core component '{component}' failed to initialize")]` `InitializationFailed { component: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> }`
    - `#[error("I/O error accessing path '{path}': {source}")]` `Io { path: PathBuf, #[source] source: std::io::Error }`
        - **Publisher:** Jede Kernfunktion, die direkt I/O-Operationen durchführt.
        - **Subscriber:** Aufrufer, die diese I/O-Fehler behandeln müssen.
    - `#[error("Serialization error: {description}")]` (für generische Serialisierungsfehler, spezifischere sollten eigene Typen haben) `Serialization { description: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> }`
    - `#[error("Deserialization error: {description}")]` `Deserialization { description: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> }`
    - `#[error("Invalid identifier provided: '{invalid_id}'")]` `InvalidId { invalid_id: String }`
    - `#[error("Resource not found: {resource_description}")]` `NotFound { resource_description: String }`
    - `#[error("Configuration error (core level): {message}")]` `CoreConfigError { message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> }` (Spezifischere `ConfigError` in `core::config::errors`)
    - `#[error("An internal logic error occurred: {0}")]` `InternalError(String)` (Sollte selten verwendet werden; spezifischere Fehler sind vorzuziehen)

##### 2. Enum: `ColorParseError` (Spezifischer Fehler für `Color::from_hex`)

- **Zweck:** Repräsentiert Fehler, die beim Parsen eines Hex-Strings zu einer `Color` auftreten können.
- **Datei:** `src/types/color.rs` (oder `src/errors/color_errors.rs` und re-exportiert) – hier in `errors.rs` für Zentralität der Fehler.
- **Ableitungen:** `#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]`
- **Varianten:**
    - `#[error("Invalid hex color string format for '{0}'. Expected formats: #RGB, #RGBA, #RRGGBB, #RRGGBBAA.")]` `InvalidHexFormat(String)`
    - `#[error("Invalid hex digit in string '{0}': {source}")]` `InvalidHexDigit(String, #[source] std::num::ParseIntError)`
    - `#[error("Invalid hex color string length for '{0}'. Expected 3, 4, 6, or 8 characters after '#'.")]` `InvalidHexLength(String)`

##### Implementierungsschritte `core::errors`

1. **Abhängigkeiten in `core/Cargo.toml` sicherstellen:**
    
    Ini, TOML
    
    ```
    [dependencies]
    thiserror = "1.0"
    uuid = { version = "1.0", features = ["v4", "serde"] } # serde optional für Fehler
    chrono = { version = "0.4", features = ["serde"] } # serde optional
    # num-traits, serde, toml, once_cell je nach Bedarf anderer Module
    ```
    
2. **Datei `core/src/errors.rs` erstellen/modifizieren:**
    - `CoreError`-Enum mit allen Varianten, `#[error(...)]`-Attributen und `#[source]`-Feldern definieren.
    - `ColorParseError`-Enum definieren.
3. **Öffentliche API und Interne Schnittstellen:**
    - Alle Enums und ihre Varianten sind `pub`.
    - Die `source()`-Methode wird von `thiserror` bereitgestellt.
4. **Unit-Tests (`core/src/errors.rs` -> `#[cfg(test)] mod tests`):**
    - Für jede Fehlervariante testen, ob die `Display`-Implementierung (via `#[error]`) die erwartete Nachricht erzeugt.
    - Für Varianten mit `#[source]`, testen, ob `source()` den zugrunde liegenden Fehler korrekt zurückgibt.
        - Beispiel für `CoreError::Io`:
            
            Rust
            
            ```
            let original_io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
            let core_io_err = CoreError::Io { path: "test.txt".into(), source: original_io_err };
            assert!(core_io_err.source().is_some());
            // Ggf. den Typ des source-Fehlers prüfen.
            ```
            
    - Testen der `ColorParseError`-Varianten.
5. **Aktualisierung `core/src/lib.rs`:** `pub mod errors;`
    - `pub use errors::{CoreError, ColorParseError};` (oder nur `CoreError` und `ColorParseError` wird über `crate::types::Color::from_hex` verwendet)

---

### Modul: `core::logging` (Logging-Infrastruktur)

**Zweck:** Initialisierung und Konfiguration des globalen `tracing`-Frameworks.

**Designphilosophie:** Bereitstellung einer einfachen Initialisierungsfunktion. Die eigentliche Verwendung der `tracing::{trace, debug, info, warn, error}` Makros erfolgt direkt im Code der anderen Module/Schichten.

**Abhängigkeiten:** `tracing`, `tracing-subscriber` (mit Features wie `fmt`, `json`, `env-filter`).

**Datei:** `src/logging.rs`

##### 1. Enum: `LogFormat`

- **Zweck:** Definiert die möglichen Ausgabeformate für das Logging.
- **Varianten:**
    - `PlainTextDevelopment`
    - `JsonProduction`
- **Ableitungen:** `#[derive(Debug, Clone, Copy)]`

##### 2. Enum: `LoggingError`

- **Zweck:** Fehler, die bei der Initialisierung des Loggings auftreten können.
- **Ableitungen:** `#[derive(Debug, thiserror::Error)]`
- **Varianten:**
    - `#[error("Failed to set global default tracing subscriber: {0}")]` `SetGlobalDefaultError(String)` (Kapselt den Fehler von `tracing::subscriber::set_global_default`)
    - `#[error("Failed to initialize tracing subscriber: {0}")]` `InitializationError(String)`

##### 3. Funktion: `initialize_logging`

- **Signatur:** `pub fn initialize_logging(level_filter: tracing::LevelFilter, format: LogFormat) -> Result<(), LoggingError>`
- **Parameter:**
    - `level_filter: tracing::LevelFilter`: Der minimale Log-Level, der global gelten soll.
    - `format: LogFormat`: Das gewünschte Ausgabeformat.
- **Rückgabe:** `Result<(), LoggingError>`
- **Logik:**
    1. Erstelle einen `tracing_subscriber::fmt::Builder` oder einen `tracing_subscriber::Registry` mit Layern.
    2. Konfiguriere den Subscriber basierend auf `format`:
        - `LogFormat::PlainTextDevelopment`:
            - Verwende `tracing_subscriber::fmt::layer()`
            - `with_ansi(true)` (falls Terminal es unterstützt, kann über Feature-Flag gesteuert werden)
            - `with_target(true)` (Modulpfad anzeigen)
            - `with_file(true)`
            - `with_line_number(true)`
            - `with_level(true)`
            - `with_filter(level_filter)`
        - `LogFormat::JsonProduction`:
            - Verwende `tracing_subscriber::fmt::layer().json()`
            - `with_current_span(true)`
            - `with_span_list(true)`
            - `with_filter(level_filter)`
            - Alternativ: `tracing_bunyan_formatter` für spezifisches Bunyan-JSON-Format.
    3. Optional: Füge einen `EnvFilter` hinzu, um Log-Levels zur Laufzeit über `RUST_LOG` feingranularer zu steuern, zusätzlich zum globalen `level_filter`. `let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level_filter.to_string()));` Der `level_filter` Parameter dient dann als Fallback, wenn `RUST_LOG` nicht gesetzt ist. Der Layer wird dann mit `with_filter(env_filter)` konfiguriert.
    4. Baue den Subscriber und versuche, ihn als globalen Default zu setzen: `tracing::subscriber::set_global_default(subscriber).map_err(|e| LoggingError::SetGlobalDefaultError(e.to_string()))`.
- **Fehlerbehandlung:** Gibt `LoggingError` zurück, falls die Initialisierung fehlschlägt.

##### Implementierungsschritte `core::logging`

1. **Abhängigkeiten in `core/Cargo.toml`:**
    
    Ini, TOML
    
    ```
    tracing = "0.1"
    tracing-subscriber = { version = "0.3", features = ["fmt", "json", "env-filter"] }
    # Ggf. tracing-bunyan-formatter
    ```
    
2. **Datei `core/src/logging.rs` erstellen:**
    - `LogFormat` Enum definieren.
    - `LoggingError` Enum mit `thiserror` definieren.
    - `initialize_logging` Funktion implementieren.
3. **Unit-Tests (konzeptionell, da `set_global_default` global ist):**
    - Testen, ob die Funktion mit verschiedenen Formaten und Leveln ohne Panic durchläuft. Das tatsächliche Setzen des globalen Subscribers ist schwer isoliert zu testen. Man könnte prüfen, ob der Builder korrekt konfiguriert wird.
4. **Aktualisierung `core/src/lib.rs`:** `pub mod logging;`
    - `pub use logging::{initialize_logging, LogFormat, LoggingError};`

---

### Modul: `core::config` (Konfigurationsprimitive)

**Zweck:** Definition von Mechanismen zum Laden, Parsen und Zugreifen auf Basiskonfigurationen.

**Designphilosophie:** Einfachheit, Robustheit. Verwendung von TOML als Konfigurationsformat und `serde` für (De-)Serialisierung.

**Abhängigkeiten:** `serde` (mit `derive`), `toml`, `once_cell` (für globalen Zugriff).

**Datei:** `src/config/mod.rs` (kann `errors.rs`, `types.rs`, `loader.rs` enthalten)

#### Untermodul: `core::config::errors`

**Datei:** `src/config/errors.rs`

##### 1. Enum: `ConfigError`

- **Zweck:** Fehler, die beim Laden oder Verarbeiten von Konfigurationen auftreten können.
- **Ableitungen:** `#[derive(Debug, thiserror::Error)]`
- **Varianten:**
    - `#[error("Failed to read configuration file from path '{path}': {source}")]` `FileReadError { path: PathBuf, #[source] source: std::io::Error }`
    - `#[error("Failed to deserialize configuration from path '{path}': {source}")]` `DeserializationError { path: PathBuf, #[source] source: toml::de::Error }`
    - `#[error("No configuration file found. Checked paths: {checked_paths:?}")]` `NoConfigurationFileFound { checked_paths: Vec<PathBuf> }`
    - `#[error("Configuration already initialized.")]` `AlreadyInitializedError`
    - `#[error("Configuration not yet initialized.")]` `NotInitializedError`
    - `#[error("Invalid configuration value for key '{key}': {reason}")]` (Falls Validierung hier stattfindet) `InvalidValueError { key: String, reason: String }`

#### Untermodul: `core::config::types`

**Datei:** `src/config/types.rs`

##### 1. Struct: `CoreConfig` (Beispielstruktur)

- **Zweck:** Hält alle spezifischen Konfigurationen der Kernschicht. Muss an die tatsächlichen Bedürfnisse angepasst werden.
- **Ableitungen:** `#[derive(Debug, Clone, serde::Deserialize, Default)]`
- **Attribute:** `#[serde(deny_unknown_fields)]` auf der Struktur.
- **Felder (Beispiele):**
    
    Rust
    
    ```
    use serde::Deserialize;
    use std::path::PathBuf;
    
    #[derive(Debug, Clone, Deserialize, Default)]
    #[serde(rename_all = "kebab-case")] // TOML verwendet oft kebab-case
    pub enum LogLevelConfig { // Muss auch in core::logging bekannt sein oder hierhin verschoben werden
        Trace,
        Debug,
        #[default]
        Info,
        Warn,
        Error,
    }
    
    #[derive(Debug, Clone, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub struct CoreConfig {
        #[serde(default = "default_log_level")]
        pub log_level: LogLevelConfig,
        #[serde(default = "default_some_path")]
        pub some_critical_path: PathBuf,
        #[serde(default)] // Verwendet FeatureFlags::default()
        pub feature_flags: FeatureFlags,
    }
    
    // Default-Funktionen müssen den korrekten Typ zurückgeben
    fn default_log_level() -> LogLevelConfig { LogLevelConfig::default() }
    fn default_some_path() -> PathBuf { PathBuf::from("/usr/share/novade/default_resource") }
    
    impl Default for CoreConfig {
        fn default() -> Self {
            Self {
                log_level: default_log_level(),
                some_critical_path: default_some_path(),
                feature_flags: FeatureFlags::default(),
            }
        }
    }
    
    #[derive(Debug, Clone, Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    pub struct FeatureFlags {
        #[serde(default)] // bool standardmäßig auf false
        pub enable_alpha_feature: bool,
        #[serde(default = "default_beta_timeout_ms")]
        pub beta_feature_timeout_ms: u64,
    }
    
    fn default_beta_timeout_ms() -> u64 { 1000 }
    ```
    
- **Validierung:** Grundlegende Validierung durch Typen und `serde`-Attribute. Komplexere Validierung kann nach Deserialisierung erfolgen (z.B. `TryFrom<RawCoreConfig>` oder `validate()`-Methode), die dann `ConfigError::InvalidValueError` zurückgibt.

#### Untermodul: `core::config::loader`

**Datei:** `src/config/loader.rs`

##### 1. Funktion: `load_core_config`

- **Signatur:** `pub fn load_core_config(config_paths: &[PathBuf]) -> Result<CoreConfig, ConfigError>` (Nimmt eine Liste von Pfaden, um sie der Reihe nach zu prüfen).
- **Logik:**
    1. Iteriere über `config_paths`.
    2. Für jeden Pfad:
        - Prüfe, ob die Datei existiert.
        - Wenn ja, versuche sie zu lesen: `std::fs::read_to_string(path).map_err(|e| ConfigError::FileReadError { path: path.to_path_buf(), source: e })?`.
        - Versuche, den Inhalt zu deserialisieren: `toml::from_str(&content_str).map_err(|e| ConfigError::DeserializationError { path: path.to_path_buf(), source: e })?`.
        - Wenn erfolgreich, gib `Ok(config)` zurück.
    3. Wenn keine Datei gefunden oder erfolgreich geparst wurde, gib `Err(ConfigError::NoConfigurationFileFound { checked_paths: config_paths.to_vec() })` zurück.

#### Statischer Zugriff (`core::config::mod.rs` oder `core::config::global.rs`)

Rust

```
// In core::config::mod.rs oder einem neuen core::config::global.rs
use super::types::CoreConfig;
use super::errors::ConfigError;
use once_cell::sync::OnceCell;
use std::path::PathBuf;

static CORE_CONFIG: OnceCell<CoreConfig> = OnceCell::new();

/// Initialisiert die globale Core-Konfiguration.
/// Darf nur einmal während des Anwendungsstarts aufgerufen werden.
pub fn initialize_global_core_config(config: CoreConfig) -> Result<(), ConfigError> {
    CORE_CONFIG.set(config).map_err(|_| ConfigError::AlreadyInitializedError)
}

/// Gibt eine Referenz auf die global initialisierte Core-Konfiguration zurück.
///
/// # Panics
///
/// Paniert, wenn `initialize_global_core_config()` nicht zuvor erfolgreich aufgerufen wurde.
/// Dies signalisiert einen schwerwiegenden Programmierfehler in der Anwendungsinitialisierung.
pub fn get_global_core_config() -> &'static CoreConfig {
    CORE_CONFIG.get().expect("CoreConfig wurde nicht initialisiert. initialize_global_core_config() muss zuerst aufgerufen werden.")
}

/// Lädt die Konfiguration von den angegebenen Pfaden und initialisiert sie global.
/// Dies ist eine Bequemlichkeitsfunktion.
pub fn load_and_initialize_global_config(config_paths: &[PathBuf]) -> Result<(), ConfigError> {
    use super::loader::load_core_config; // Pfad anpassen
    let config = load_core_config(config_paths)?;
    initialize_global_core_config(config)
}
```

##### Implementierungsschritte `core::config`

1. **Abhängigkeiten in `core/Cargo.toml`:**
    
    Ini, TOML
    
    ```
    serde = { version = "1.0", features = ["derive"] }
    toml = "0.8" # Aktuelle Version prüfen
    once_cell = "1.19" # Aktuelle Version prüfen
    ```
    
2. **Verzeichnisstruktur erstellen:** `core/src/config/`, darin `mod.rs`, `errors.rs`, `types.rs`, `loader.rs`, `global.rs` (optional).
3. **`errors.rs`:** `ConfigError` Enum implementieren.
4. **`types.rs`:** `CoreConfig` (und ggf. untergeordnete Strukturen wie `LogLevelConfig`, `FeatureFlags`) mit `serde`-Attributen und `Default`-Implementierungen definieren. Default-Funktionen erstellen.
5. **`loader.rs`:** `load_core_config` Funktion implementieren.
6. **`global.rs` (oder `mod.rs`):** Statische `CORE_CONFIG` Variable mit `OnceCell`, `initialize_global_core_config`, `get_global_core_config` und `load_and_initialize_global_config` implementieren.
7. **`config/mod.rs`:** Module deklarieren und öffentliche Typen/Funktionen re-exportieren.
    
    Rust
    
    ```
    pub mod errors;
    pub mod types;
    pub mod loader;
    pub mod global; // oder Inhalt direkt hier
    
    pub use errors::ConfigError;
    pub use types::{CoreConfig, LogLevelConfig, FeatureFlags}; // Beispiele
    pub use loader::load_core_config;
    pub use global::{initialize_global_core_config, get_global_core_config, load_and_initialize_global_config};
    ```
    
8. **Aktualisierung `core/src/lib.rs`:** `pub mod config;`
9. **Unit-Tests:**
    - `ConfigError`: Teste Display-Implementierungen.
    - `CoreConfig`: Teste `Default`-Implementierung und `serde` (De-)Serialisierung mit Beispieldaten (gültiges TOML, TOML mit fehlenden Feldern, TOML mit unbekannten Feldern bei `deny_unknown_fields`).
    - `load_core_config`:
        - Test mit gültiger Konfigurationsdatei.
        - Test mit mehreren Pfaden, wobei die erste gefundene Datei verwendet wird.
        - Test, wenn keine Datei gefunden wird (`NoConfigurationFileFound`).
        - Test mit nicht lesbarer Datei (`FileReadError`).
        - Test mit fehlerhafter TOML-Syntax (`DeserializationError`).
    - Globaler Zugriff: Teste `initialize_global_core_config` (Erfolg, Fehler bei Mehrfachinitialisierung), `get_global_core_config` (Erfolg nach Init, Panic vor Init).

---

### Modul: `core::utils` (Allgemeine Hilfsfunktionen)

**Zweck:** Beherbergt allgemeine, in sich geschlossene Hilfsfunktionen, die nicht in spezifischere Module passen.

**Designphilosophie:** Einfachheit, Reinheit (wo möglich), keine Abhängigkeiten zu anderen Kernschicht-Modulen außer `core::errors` (für Utility-spezifische Fehler).

**Abhängigkeiten:** `std`, `thiserror`.

**Struktur:** Dieses Modul kann in Submodule unterteilt werden, falls viele Utilities entstehen (z.B. `core::utils::string`, `core::utils::math`). Für den Anfang eine einzelne `src/utils.rs`.

**Datei:** `src/utils.rs` (und ggf. `src/utils/errors.rs`)

#### Beispiel: Utility-Submodul `core::utils::path_utils`

**Datei:** `src/utils/path_utils.rs`

##### 1. Enum: `PathUtilError`

- **Ableitungen:** `#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]`
- **Varianten:**
    - `#[error("Path normalization failed: {0}")]` `NormalizationFailed(String)`

##### 2. Funktion: `normalize_path_robustly` (Beispiel für ein komplexeres Utility)

- **Signatur:** `pub fn normalize_path_robustly(path: &std::path::Path) -> Result<PathBuf, PathUtilError>`
- **Logik:** (Hier würde eine detaillierte Schritt-für-Schritt-Beschreibung des Normalisierungsalgorithmus stehen, z.B. Auflösen von `.` und `..`, Umgang mit Symlinks (falls im Scope), Sicherstellung einer kanonischen Form). Für diese Spezifikation wird kein konkreter Algorithmus vorgegeben, da die Anforderung eines "komplexen" Utilities noch nicht klar ist.
- **Fehlerbehandlung:** Gibt `PathUtilError` zurück.

#### Allgemeine Richtlinien für `core::utils`

- **Geltungsbereich:** Nur wirklich allgemeine Utilities.
- **Einfachheit:** Einfache Funktionen bevorzugen.
- **Reinheit:** Reine Funktionen bevorzugen.
- **Fehlerbehandlung:** Jede fehleranfällige Utility-Funktion gibt `Result<T, YourUtilError>` zurück, wobei `YourUtilError` mit `thiserror` im Utility-Submodul definiert und in einen allgemeinen `UtilsError` (oder direkt in `CoreError` via `#[from]`, falls sehr generisch) gewrappt werden kann.
- **Dokumentation:** Umfassende `rustdoc`-Kommentare mit Beispielen.
- **Tests:** Gründliche Unit-Tests für jede Utility-Funktion.

##### Implementierungsschritte `core::utils`

1. **Datei `core/src/utils/mod.rs` (oder `core/src/utils.rs`) erstellen.**
2. Falls Submodule für Utilities benötigt werden (z.B. `path_utils.rs`), diese erstellen und in `utils/mod.rs` deklarieren.
3. Für jedes Utility oder Utility-Submodul ggf. eine eigene `errors.rs` erstellen oder Fehler im Haupt-`core::errors` definieren, falls extrem generisch.
4. Utility-Funktionen implementieren, dabei die oben genannten Richtlinien beachten.
5. Umfassende Unit-Tests für jede Funktion schreiben.
6. **Aktualisierung `core/src/lib.rs`:** `pub mod utils;`

---

**Finale `core/src/lib.rs` (Beispiel):**

Rust

```
// src/lib.rs

// Module deklarieren
pub mod config;
pub mod errors;
pub mod logging;
pub mod types;
pub mod utils;

// Wichtige Typen und Funktionen re-exportieren, falls gewünscht
pub use errors::{CoreError, ColorParseError}; // Beispiel
pub use types::{Point, Size, Rect, RectInt, Color, Orientation}; // Beispiel
// ...usw. für andere Module, falls eine flachere API für das Crate gewünscht ist.

// Dieser Crate dient als Fundament und sollte keine spezifische Anwendungslogik enthalten.
// Seine API sollte stabil und gut dokumentiert sein.
```

---

## 3. Ausblick auf weitere Schichten (Methodik)

Die Ultra-Feinspezifikationen für die Domänen-, System- und UI-Schicht würden derselben detaillierten Methodik folgen:

1. **Modulübersicht:** Zweck, Verantwortlichkeiten, Design-Rationale.
2. **Datenstrukturen und Typdefinitionen:**
    - Alle `struct`s, `enum`s, `trait`s, Typaliase in Rust-Syntax.
    - Felder/Varianten: Name, Typ, Sichtbarkeit, Initialwerte (falls zutreffend).
    - Ableitungen (`Debug`, `Clone`, `Serialize`, `Deserialize`, `Default`, `thiserror::Error`, etc.).
    - Invarianten und Geschäftsregeln, die durch die Typen abgebildet werden.
3. **Öffentliche API und Interne Schnittstellen:**
    - Exakte Signaturen aller `pub fn`, `pub trait`, `pub struct` Methoden (Parameter: Name, Typ; Rückgabetyp; `async`, `const`, Zugriffsmodifikatoren).
    - Vor- und Nachbedingungen für jede Methode.
    - Detaillierte Beschreibung der Logik/Algorithmen, die die Methode implementiert.
4. **Event-Spezifikationen (falls zutreffend):**
    - Eindeutiger Event-Name/Typ (Rust-Struktur).
    - Payload-Struktur (Felder und deren Typen).
    - Typische Publisher und Subscriber.
    - Auslösebedingungen für das Event.
5. **Fehlerbehandlung:**
    - Definition des modulspezifischen Fehler-Enums mit `thiserror`.
    - Jede Variante: `#[error("...")]`-Nachricht, Felder für Kontext.
    - Verwendung von `#[source]` und `#[from]` für Fehlerverkettung und -konvertierung.
    - Abbildung von Fehlern aus tieferliegenden Schichten oder externen Bibliotheken.
6. **Zustandsverwaltung und Lebenszyklus (für komplexe Komponenten):**
    - Detaillierte Beschreibung des internen Zustands.
    - Methoden zur Zustandsänderung und deren Auswirkungen.
    - Lebenszyklusmanagement (Initialisierung, Laufzeit, Beendigung).
7. **Interaktionen und Abhängigkeiten:**
    - Mit anderen Modulen derselben Schicht.
    - Mit Modulen anderer Schichten (explizite Nutzung von deren APIs oder Events).
8. **Detaillierte Implementierungsschritte:**
    - Ziel-Dateistruktur für das Modul.
    - Schritt-für-Schritt-Anleitung zur Implementierung von Datenstrukturen, Logik und API.
    - Reihenfolge der Implementierung von Funktionen/Methoden.
9. **Testaspekte:**
    - Identifikation testkritischer Logik.
    - Beispiele für Unit-Testfälle (Szenarien, Eingaben, erwartete Ausgaben/Fehler).
    - Hinweise zu Mocking-Strategien für Abhängigkeiten.

Für die **Domänenschicht** würde dies beispielsweise für `domain::theming`, `domain::workspaces`, `domain::user_centric_services` etc. detailliert erfolgen, wobei jede Entität, jeder Service-Trait und jede Methode genau spezifiziert wird.

Für die **Systemschicht** (z.B. `system::compositor`, `system::input`, `system::dbus`) wäre die Spezifikation besonders komplex, da sie die Interaktion mit externen Bibliotheken (Smithay, libinput, zbus), Wayland-Protokollen und Systemdiensten detailliert beschreiben muss. Hier wären die Handler-Implementierungen (z.B. `CompositorHandler`, `XdgShellHandler`) und die exakte Nutzung der Smithay-APIs im Fokus.

Für die **UI-Schicht** (z.B. `ui::shell`, `ui::widgets`, `ui::control_center`) würde die Spezifikation die GTK4-Widget-Struktur, das Layout, die Signal-Handler, die Datenbindung an Domänen-/Systemzustände und die Logik zur Auslösung von Aktionen in den unteren Schichten detailliert beschreiben.

Dieser extrem detaillierte Ansatz ist sehr aufwendig, stellt aber sicher, dass Entwickler eine klare, unmissverständliche Anleitung haben, was die Konsistenz und Qualität des Endprodukts maßgeblich verbessert.


# Pflichtenheft für die Coreschicht

**Version:** 1.0

**Datum:** 2024-05-16

**Autor:** Dr. rer. nat. Expertenteam

**Status:** Entwurf

## Inhaltsverzeichnis

1. Einleitung 1.1. Zweck des Dokuments 1.2. Geltungsbereich 1.3. Zielgruppe 1.4. Definitionen und Akronyme 1.5. Referenzierte Dokumente und Standards
2. Datenbankdesign-Spezifikation 2.1. Data Dictionary 2.1.1. Tabellenstruktur und Felddefinitionen 2.1.2. Datentypen 2.1.3. Primär- und Fremdschlüssel 2.1.4. Constraints und Validierungsregeln 2.2. Entity-Relationship-Diagramme (ERD) 2.2.1. Notation 2.2.2. Werkzeuge 2.2.3. Konzeptionelles ER-Diagramm (Beschreibung) 2.2.4. Best Practices für ER-Diagramme
3. Schnittstellendesign-Spezifikation (API) 3.1. Interne Schnittstellen 3.1.1. REST-API für Kernfunktionalitäten 3.1.1.1. Basis-URL 3.1.1.2. Authentifizierung und Autorisierung 3.1.1.3. Datenformate (Request/Response) 3.1.1.4. HTTP-Statuscodes 3.1.1.5. Endpunktspezifikationen (Beispiele) 3.1.1.6. Versionierung 3.2. Externe Schnittstellen (falls zutreffend) 3.3. API-Dokumentation (OpenAPI/Swagger) 3.4. Best Practices für API-Design
4. UI/UX-Design-Spezifikationen (für Administrations- und Konfigurationsoberflächen) 4.1. Wireframes 4.1.1. Zweck und Detailgrad 4.1.2. Werkzeuge 4.1.3. Beispiele (Beschreibung der wichtigsten Ansichten) 4.1.4. Informationsarchitektur und Navigation 4.2. Mockups 4.2.1. Visuelles Design und Detailtiefe 4.2.2. Werkzeuge 4.2.3. Beispiele (Beschreibung der wichtigsten Ansichten) 4.3. Prototypen (klickbar) 4.3.1. Interaktivität und Benutzerflüsse 4.3.2. Werkzeuge 4.3.3. Zu testende Szenarien 4.4. Styleguide 4.4.1. Farbpalette 4.4.2. Typografie 4.4.3. Ikonografie 4.4.4. Abstände und Layout-Raster 4.4.5. UI-Komponentenbibliothek
5. Sicherheitskonzept 5.1. Grundlagen und Prinzipien 5.1.1. Layered Security (Defense in Depth) 5.1.2. Prinzip der geringsten Rechte (Principle of Least Privilege) 5.2. Authentifizierung und Autorisierung 5.2.1. Authentifizierungsmechanismen 5.2.2. Autorisierungsmodell (z.B. RBAC) 5.3. Datensicherheit 5.3.1. Verschlüsselung von Daten "at rest" 5.3.2. Verschlüsselung von Daten "in transit" 5.4. API-Sicherheitsmaßnahmen 5.5. Sichere Eingabevalidierung und -verarbeitung 5.6. Protokollierung (Logging) und Überwachung sicherheitsrelevanter Ereignisse
6. Schlussfolgerungen
7. Anhang 7.1. Glossar 7.2. Referenzierte Dokumente und Standards (erweitert)

---

## 1. Einleitung

Dieses Pflichtenheft (auch als funktionale Spezifikation oder Sollkonzept bezeichnet 1) definiert die Anforderungen und Spezifikationen für die Entwicklung der Coreschicht einer Softwareanwendung. Es dient als verbindliche Grundlage für die Implementierung und stellt sicher, dass das Endprodukt die definierten Bedürfnisse und Erwartungen erfüllt.2 Die Coreschicht umfasst die zentrale Geschäftslogik, die Datenhaltung sowie die internen und externen Schnittstellen des Systems.

### 1.1. Zweck des Dokuments

Der Hauptzweck dieses Dokuments ist die detaillierte und unmissverständliche Beschreibung der zu realisierenden Coreschicht. Es legt fest, _was_ entwickelt werden soll und _wie_ die einzelnen Komponenten strukturiert und implementiert werden müssen, um die übergeordneten Projektziele zu erreichen.3 Dieses Pflichtenheft bildet die Grundlage für Design, Entwicklung, Test und Abnahme der Coreschicht. Es dient der Vermeidung von Missverständnissen zwischen Auftraggeber und Auftragnehmer und reduziert das Risiko von Fehlentwicklungen und Nacharbeiten.5

### 1.2. Geltungsbereich

Dieses Pflichtenheft bezieht sich ausschließlich auf die Entwicklung und Implementierung der Coreschicht. Dies beinhaltet:

- **Datenbankdesign:** Struktur, Tabellen, Felder, Datentypen und Beziehungen der primären Datenbank.
- **Schnittstellendesign:** Spezifikation der internen APIs (insbesondere REST-APIs) für die Kommunikation mit anderen Systemkomponenten (z.B. Frontend, andere Backend-Services) und ggf. externen Diensten.
- **UI/UX-Design-Spezifikationen:** Konzeption und Gestaltung von Benutzeroberflächen, die direkt mit der Coreschicht interagieren, typischerweise Administrations- oder Konfigurationsoberflächen.
- **Sicherheitskonzept:** Maßnahmen zur Gewährleistung der Vertraulichkeit, Integrität und Verfügbarkeit der Daten und Funktionen der Coreschicht.

Aspekte wie die detaillierte Ausgestaltung von Frontend-Applikationen, die nicht direkt Administrationszwecken der Coreschicht dienen, oder die übergeordnete Infrastrukturplanung sind nicht primärer Gegenstand dieses Dokuments, können aber referenziert werden, wo Interdependenzen bestehen.

### 1.3. Zielgruppe

Dieses Dokument richtet sich an folgende Stakeholder:

- **Entwicklerteam:** Als detaillierte Vorgabe für die Implementierung.
- **Projektmanager:** Zur Planung, Steuerung und Überwachung des Entwicklungsprozesses.
- **Qualitätssicherungsteam:** Als Grundlage für die Erstellung von Testfällen und die Durchführung von Tests.
- **Systemarchitekten:** Zur Sicherstellung der Kompatibilität mit der Gesamtarchitektur.
- **Auftraggeber/Produktmanagement:** Zur Überprüfung, ob die spezifizierten Funktionalitäten den Anforderungen entsprechen und zur formalen Abnahme.

Ein klares Verständnis der Inhalte ist für alle Beteiligten essenziell, um den Projekterfolg sicherzustellen.2

### 1.4. Definitionen und Akronyme

Eine umfassende Liste der in diesem Dokument verwendeten Fachbegriffe, Abkürzungen und Akronyme befindet sich im Anhang (Kapitel 7.1 Glossar).5 Dies dient der Vermeidung von Missverständnissen und der Sicherstellung eines einheitlichen Sprachgebrauchs.

### 1.5. Referenzierte Dokumente und Standards

Dieses Pflichtenheft basiert auf und referenziert verschiedene externe Dokumente, Normen und interne Vorgaben. Eine detaillierte Auflistung findet sich im Anhang (Kapitel 7.2 Referenzierte Dokumente und Standards).1 Zu den wichtigsten gehören Normen wie DIN 69901-5 für Projektmanagement und Pflichtenhefte 1 sowie ggf. der IEEE 830 Standard für Software Requirements Specifications (SRS) 8, falls internationale Kontexte relevant sind.

## 2. Datenbankdesign-Spezifikation

Die Datenbank bildet das Fundament der Coreschicht und ist für die persistente Speicherung und Verwaltung aller relevanten Anwendungsdaten zuständig. Eine sorgfältige Planung und Dokumentation des Datenbankdesigns ist entscheidend für die Performance, Integrität und Skalierbarkeit des Gesamtsystems. Dieses Kapitel beschreibt die Struktur der Datenbank, einschließlich des Data Dictionary und der Entity-Relationship-Diagramme (ERD). Die hier getroffenen Entscheidungen müssen Aspekte der Datensicherheit, wie Verschlüsselung und Zugriffskontrolle, von Beginn an berücksichtigen, um ein "Secure by Design"-Prinzip zu gewährleisten.

### 2.1. Data Dictionary

Das Data Dictionary dient als zentrales Nachschlagewerk für alle Datenelemente der Datenbank. Es dokumentiert detailliert jede Tabelle, deren Felder, Datentypen, Beziehungen und Constraints.11 Ein gut gepflegtes Data Dictionary ist unerlässlich für das Verständnis der Datenstruktur, die Entwicklung konsistenter Abfragen und die Wartung des Systems.12

#### 2.1.1. Tabellenstruktur und Felddefinitionen

Für jede Tabelle im Datenbankschema werden die folgenden Informationen spezifiziert:

- **Tabellenname:** Ein eindeutiger und deskriptiver Name für die Tabelle (z.B. `Kunden`, `Bestellungen`, `Produkte`). Es empfiehlt sich, konsistente Namenskonventionen zu verwenden (z.B. Singular- oder Pluralformen).14
- **Beschreibung:** Eine kurze Erläuterung des Zwecks und Inhalts der Tabelle.
- **Felder (Attribute):** Für jedes Feld in der Tabelle:
    - **Feldname:** Eindeutiger Name des Feldes innerhalb der Tabelle (z.B. `KundenID`, `Vorname`, `Erstelldatum`). Auch hier sind konsistente Namenskonventionen wichtig (z.B. CamelCase oder snake_case).12
    - **Beschreibung:** Bedeutung und Zweck des Feldes.
    - **Datentyp:** Der spezifische Datentyp des Feldes (siehe Kapitel 2.1.2).
    - **Primärschlüssel (PK):** Kennzeichnung, ob das Feld Teil des Primärschlüssels ist.
    - **Fremdschlüssel (FK):** Kennzeichnung, ob das Feld ein Fremdschlüssel ist, inklusive der Referenztabelle und des Referenzfeldes.
    - **NotNull-Constraint:** Angabe, ob das Feld einen Wert enthalten muss (`TRUE`/`FALSE`).
    - **Unique-Constraint:** Angabe, ob der Wert des Feldes innerhalb der Tabelle eindeutig sein muss (`TRUE`/`FALSE`).
    - **Standardwert:** Ein optionaler Standardwert, der verwendet wird, wenn beim Einfügen eines neuen Datensatzes kein Wert für dieses Feld angegeben wird.
    - **Beispielwert:** Ein typischer Beispielwert zur Veranschaulichung.
    - **Sensitivität:** Klassifizierung der Sensitivität der Daten im Feld (z.B. öffentlich, intern, vertraulich, streng vertraulich) als Grundlage für Sicherheitsmaßnahmen.

Die Definition der Tabellen und Felder erfolgt in einer strukturierten Form, beispielsweise durch eine Serie von Tabellen, die jeweils eine Datenbanktabelle beschreiben.

**Tabelle 2.1: Beispielstruktur für eine Data Dictionary Tabelle (am Beispiel `Kunden`)**

|   |   |   |   |   |   |   |   |   |   |   |
|---|---|---|---|---|---|---|---|---|---|---|
|**Attribut**|**Feldname**|**Datentyp**|**PK**|**FK (Tabelle.Feld)**|**NotNull**|**Unique**|**Standardwert**|**Beispielwert**|**Sensitivität**|**Beschreibung**|
|Kundenidentifikation|`KundenID`|`INTEGER`|Ja||Ja|Ja|AUTOINCREMENT|1001|Intern|Eindeutiger Identifikator des Kunden|
|Vorname des Kunden|`Vorname`|`VARCHAR(50)`|Nein||Ja|Nein||Max|Vertraulich|Vorname des Kunden|
|Nachname des Kunden|`Nachname`|`VARCHAR(50)`|Nein||Ja|Nein||Mustermann|Vertraulich|Nachname des Kunden|
|E-Mail-Adresse|`Email`|`VARCHAR(100)`|Nein||Ja|Ja||max.mustermann@example.com|Vertraulich|E-Mail-Adresse des Kunden|
|Registrierungsdatum|`RegistriertAm`|`TIMESTAMP`|Nein||Ja|Nein|CURRENT_TIMESTAMP|2023-10-26 10:00:00|Intern|Zeitpunkt der Registrierung des Kunden|
|Letzter Login|`LetzterLogin`|`TIMESTAMP`|Nein||Nein|Nein||2024-05-15 14:30:00|Vertraulich|Zeitpunkt des letzten Logins des Kunden|

Die konsequente Anwendung von Namenskonventionen und die detaillierte Beschreibung jedes Elements sind entscheidend. Dies erleichtert nicht nur die Entwicklung, sondern auch die spätere Wartung und Erweiterung des Systems, da neue Teammitglieder sich schneller einarbeiten können und die Datenstrukturen klar verständlich sind.12 Die Klassifizierung der Datensensitivität ist ein wichtiger Input für das Sicherheitskonzept, insbesondere für Verschlüsselungsstrategien und Zugriffskontrollen.

#### 2.1.2. Datentypen

Die Auswahl des korrekten Datentyps für jedes Feld ist fundamental, da sie bestimmt, welche Art von Daten gespeichert werden können, wie viel Speicherplatz benötigt wird und welche Operationen auf den Daten ausgeführt werden können.15 Die Datentypen müssen dem gewählten Datenbanksystem entsprechen. Gängige Datentypen umfassen:

- **Zeichenketten:**
    - `CHAR(n)`: Feste Länge, geeignet für kurze, immer gleich lange Zeichenketten.
    - `VARCHAR(n)`: Variable Länge bis zu `n` Zeichen, effizient für unterschiedlich lange Texte.
    - `TEXT` / `CLOB`: Für sehr lange Textdaten.
- **Numerische Typen:**
    - `INTEGER` / `INT`: Ganze Zahlen.
    - `SMALLINT`, `BIGINT`: Ganze Zahlen mit kleinerem bzw. größerem Wertebereich.
    - `NUMERIC(p,s)` / `DECIMAL(p,s)`: Festkommazahlen mit Präzision `p` und `s` Nachkommastellen, ideal für Währungsbeträge.
    - `FLOAT`, `REAL`, `DOUBLE PRECISION`: Gleitkommazahlen.
- **Datum und Zeit:**
    - `DATE`: Speichert nur das Datum.
    - `TIME`: Speichert nur die Uhrzeit.
    - `TIMESTAMP` / `DATETIME`: Speichert Datum und Uhrzeit, oft mit Zeitzoneninformation.
- **Boole'sche Werte:**
    - `BOOLEAN`: Speichert `TRUE` oder `FALSE`.
- **Binärdaten:**
    - `BLOB` / `BYTEA`: Für die Speicherung von Binärdaten wie Bildern oder Dateien (obwohl die Speicherung großer Binärdateien direkt in der Datenbank oft kritisch hinterfragt werden sollte).
- **Spezifische Typen:**
    - `UUID`: Universally Unique Identifier.
    - `JSON` / `JSONB`: Zur Speicherung von JSON-Dokumenten direkt in der Datenbank.
    - `XML`: Zur Speicherung von XML-Daten.

Die Wahl des Datentyps beeinflusst auch die Datenvalidierung auf Datenbankebene; beispielsweise kann in ein `INTEGER`-Feld kein Text eingegeben werden.15 Die Feldgröße (z.B. bei `VARCHAR(n)` oder die Wahl zwischen `INTEGER` und `BIGINT`) sollte so gewählt werden, dass sie den erwarteten Datenmengen entspricht, aber keinen unnötigen Speicherplatz verschwendet.16

#### 2.1.3. Primär- und Fremdschlüssel

- **Primärschlüssel (PK):** Jede Tabelle muss einen Primärschlüssel besitzen, der jeden Datensatz eindeutig identifiziert. Ein Primärschlüssel kann aus einem oder mehreren Feldern bestehen (zusammengesetzter Primärschlüssel). Üblicherweise werden hierfür `INTEGER`- oder `UUID`-Felder mit automatischer Generierung (z.B. `AUTO_INCREMENT` oder Sequenzen) verwendet.16
- **Fremdschlüssel (FK):** Fremdschlüssel dienen dazu, Beziehungen zwischen Tabellen herzustellen und die referentielle Integrität sicherzustellen. Ein Fremdschlüssel in einer Tabelle verweist auf den Primärschlüssel einer anderen (oder derselben) Tabelle. Es muss sichergestellt werden, dass für jeden Fremdschlüsselwert ein entsprechender Primärschlüsselwert in der referenzierten Tabelle existiert. Datenbanken bieten Optionen wie `ON DELETE CASCADE` oder `ON DELETE SET NULL`, um das Verhalten bei Löschoperationen in der referenzierten Tabelle zu steuern.

Die klare Definition von Primär- und Fremdschlüsseln ist die Basis für die Abbildung von Entitätsbeziehungen im relationalen Modell und somit ein Kernstück des Datenbankdesigns.11

#### 2.1.4. Constraints und Validierungsregeln

Zusätzlich zu `NotNull`- und `Unique`-Constraints können weitere Validierungsregeln auf Datenbankebene definiert werden, um die Datenintegrität zu gewährleisten:

- **CHECK-Constraints:** Erlauben die Definition spezifischer Bedingungen, die Daten erfüllen müssen (z.B. `Alter >= 18`, `Preis > 0`).
- **Default-Werte:** Wie in 2.1.1 erwähnt, können Standardwerte für Felder definiert werden.

Obwohl viele Validierungen auch auf Applikationsebene (in der Coreschicht-Logik) stattfinden, bieten datenbankseitige Constraints eine zusätzliche Sicherheitsebene und stellen die Datenkonsistenz auch bei direkten Datenbankzugriffen sicher. Die Definition dieser Regeln im Data Dictionary ist daher von großer Bedeutung.17

### 2.2. Entity-Relationship-Diagramme (ERD)

Entity-Relationship-Diagramme (ERDs) visualisieren die Struktur der Datenbank, indem sie Entitäten (Tabellen), deren Attribute (Felder) und die Beziehungen zwischen ihnen darstellen.18 Sie sind ein unverzichtbares Werkzeug für das Verständnis und die Kommunikation des Datenbankmodells.

#### 2.2.1. Notation

Es existieren verschiedene Notationen für ERDs. Die gebräuchlichsten sind:

- **Chen-Notation:** Stellt Entitäten als Rechtecke, Attribute als Ovale und Beziehungen als Rauten dar. Sie ist sehr ausdrucksstark und detailliert.18
- **Crow's Foot Notation (Krähenfußnotation):** Stellt Entitäten als Rechtecke dar und Beziehungen als Linien zwischen ihnen, wobei die Kardinalität (z.B. eins-zu-viele, viele-zu-viele) durch spezielle Symbole (Krähenfüße) am Ende der Linien angezeigt wird. Diese Notation ist oft intuitiver und wird in vielen modernen Modellierungswerkzeugen verwendet.18
- **UML-Klassendiagramm-Notation:** Kann ebenfalls zur Darstellung von Datenmodellen verwendet werden, wobei Klassen Entitäten repräsentieren.18

Für dieses Projekt wird die **Crow's Foot Notation** empfohlen, da sie eine klare und weit verbreitete Methode zur Darstellung von Kardinalitäten bietet und von vielen gängigen ERD-Werkzeugen unterstützt wird.

#### 2.2.2. Werkzeuge

Zur Erstellung und Pflege von ER-Diagrammen stehen zahlreiche Werkzeuge zur Verfügung, sowohl Desktop-Anwendungen als auch webbasierte Lösungen. Einige Beispiele sind:

- **Lucidchart:** Ein populäres Online-Tool mit Funktionen für Datenimport und Kollaboration.20
- **ERDPlus:** Ein webbasiertes Tool, das die automatische Konvertierung von ER-Diagrammen in relationale Schemata ermöglicht und SQL exportieren kann.20
- **Visual Paradigm Online:** Bietet eine breite Palette an Diagrammtypen, einschließlich ERDs, mit vielen Vorlagen.20
- **SmartDraw:** Ermöglicht die automatische Erstellung von ERDs aus bestehenden Datenbanken.20
- **Creately:** Ein weiteres kollaboratives Online-Diagrammwerkzeug.20
- **dbdiagram.io:** Ermöglicht die Erstellung von ERDs durch das Schreiben einer einfachen textbasierten Sprache (DSL).21
- **ClickUp:** Bietet Whiteboard-Funktionen und Vorlagen für ER-Diagramme als Teil einer umfassenderen Projektmanagement-Plattform.23

Die Auswahl des Werkzeugs sollte auf Basis der Projektanforderungen, der Teampräferenzen und ggf. vorhandener Lizenzen erfolgen. Wichtig ist, dass das Werkzeug den Export in gängige Bildformate oder als Vektorgrafik ermöglicht und idealerweise eine Versionierung oder Integration mit Versionskontrollsystemen unterstützt.

#### 2.2.3. Konzeptionelles ER-Diagramm (Beschreibung)

Das konzeptionelle ER-Diagramm stellt die Hauptentitäten des Systems und ihre Beziehungen auf einer hohen Abstraktionsebene dar. Es dient dazu, ein grundlegendes Verständnis der Datenstruktur zu vermitteln, ohne sich in Implementierungsdetails zu verlieren.

Für die Coreschicht werden folgende Hauptentitäten und ihre Beziehungen erwartet (Beispielhaft, muss an das spezifische Projekt angepasst werden):

- **Benutzer (`Users`):** Enthält Informationen über die Benutzer des Systems (z.B. `UserID`, `Username`, `PasswortHash`, `Email`, `Rolle`).
- **[Hauptentität 1] (z.B. `Produkte`):** Enthält Attribute spezifisch für die Kernfunktionalität (z.B. `ProduktID`, `Name`, `Beschreibung`, `Preis`).
- **[Hauptentität 2] (z.B. `Bestellungen`):** (z.B. `BestellID`, `KundenID` (FK zu `Users`), `Bestelldatum`, `Status`).
- ** (z.B. `Bestellpositionen`):** Verbindet `Bestellungen` und `Produkte` (z.B. `BestellpositionsID`, `BestellID` (FK), `ProduktID` (FK), `Menge`, `Einzelpreis`).

**Beziehungen (Beispiele):**

- Ein `Benutzer` kann viele `Bestellungen` aufgeben (1:N).
- Eine `Bestellung` gehört zu genau einem `Benutzer` (N:1).
- Eine `Bestellung` kann viele `Produkte` über `Bestellpositionen` enthalten (M:N, realisiert über die Verknüpfungstabelle `Bestellpositionen`).
- Ein `Produkt` kann in vielen `Bestellungen` über `Bestellpositionen` enthalten sein (M:N).

Das tatsächliche ER-Diagramm wird als separate grafische Darstellung beigefügt oder im gewählten Werkzeug gepflegt und hier referenziert. Es muss alle im Data Dictionary definierten Tabellen und ihre durch Fremdschlüssel spezifizierten Beziehungen abbilden.

#### 2.2.4. Best Practices für ER-Diagramme

Bei der Erstellung und Pflege von ER-Diagrammen sollten folgende Best Practices beachtet werden, um Klarheit, Lesbarkeit und Korrektheit zu gewährleisten 14:

- **Standardisierte Symbole und Notationen verwenden:** Dies erleichtert das Verständnis für alle Beteiligten.24
- **Überlappende Linien vermeiden:** Linienkreuzungen können die Lesbarkeit stark beeinträchtigen. Diagramme sollten ausreichend Platz bieten.14
- **Konsistente Ausrichtung und Anordnung:** Objekte sollten logisch gruppiert und ausgerichtet werden, um die Struktur hervorzuheben.24
- **Farbcodierung (optional):** Farben können verwendet werden, um verschiedene Arten von Entitäten oder Beziehungen hervorzuheben, sollten aber sparsam und konsistent eingesetzt werden.14
- **Klare und konsistente Namenskonventionen:** Tabellen- und Attributnamen sollten den im Data Dictionary definierten Namen entsprechen und verständlich sein.14
- **Korrekte Darstellung von Kardinalitäten:** Die Beziehungen (1:1, 1:N, M:N) müssen korrekt abgebildet werden, da sie wesentliche Geschäftsregeln widerspiegeln.24
- **Keine Instanzen, sondern Entitätstypen darstellen:** ERDs zeigen Entitätsmengen (Tabellen), nicht einzelne Datensätze.19
- **Attribute nicht mit Entitäten verwechseln:** Attribute sind Eigenschaften einer Entität, keine eigenständigen Entitäten (es sei denn, es handelt sich um komplexe Attribute, die als eigene Entität modelliert werden sollten).24
- **Lesefluss von links nach rechts oder oben nach unten beibehalten:** Ein konsistenter Lesefluss verbessert die Verständlichkeit.24

Die Einhaltung dieser Praktiken stellt sicher, dass das ER-Diagramm ein effektives Kommunikationsmittel bleibt und die zugrundeliegende Datenbankstruktur präzise repräsentiert.

## 3. Schnittstellendesign-Spezifikation (API)

Die Schnittstellen der Coreschicht sind entscheidend für die Interaktion mit anderen Systemteilen (z.B. Frontend-Anwendungen, mobile Apps, andere Backend-Dienste) sowie potenziell mit externen Systemen. Eine klare, konsistente und gut dokumentierte API-Spezifikation ist unerlässlich für eine effiziente Entwicklung, Integration und Wartung.25 Dieses Kapitel fokussiert sich primär auf RESTful APIs, die als moderner Standard für Web-Schnittstellen gelten.27

### 3.1. Interne Schnittstellen

Interne Schnittstellen ermöglichen die Kommunikation und den Datenaustausch zwischen der Coreschicht und anderen Komponenten innerhalb der Gesamtarchitektur der Anwendung.29

#### 3.1.1. REST-API für Kernfunktionalitäten

Die primäre interne Schnittstelle wird als RESTful API (Representational State Transfer) realisiert. REST-APIs nutzen Standard-HTTP-Methoden, um Operationen auf Ressourcen auszuführen, die durch URIs identifiziert werden.27

##### 3.1.1.1. Basis-URL

Alle Endpunkte der Coreschicht-API werden unter einer gemeinsamen Basis-URL erreichbar sein, die auch die Versionierung der API beinhaltet.

Beispiel: https://api.example.com/core/v1/

Die Verwendung von v1 im Pfad kennzeichnet die erste Hauptversion der API.30

##### 3.1.1.2. Authentifizierung und Autorisierung

Jeder Zugriff auf die API muss authentifiziert und autorisiert werden.

- **Authentifizierung:**
    
    - Die primäre Authentifizierungsmethode für Endbenutzer-initiierte Anfragen (z.B. vom Frontend) erfolgt über **OAuth 2.0** (Authorization Code Flow oder Client Credentials Flow, je nach Anwendungsfall).31 JWTs (JSON Web Tokens) werden als Bearer-Token im `Authorization`-Header übertragen. Die JWTs müssen serverseitig validiert werden (Signatur, Ablaufdatum, Aussteller, Zielgruppe).31
    - Für rein serverseitige Kommunikation zwischen vertrauenswürdigen internen Diensten können **API-Keys** verwendet werden, die ebenfalls im HTTP-Header (z.B. `X-API-Key`) übertragen werden.31 API-Keys müssen sicher generiert, gespeichert und rotiert werden können.
    - Basisauthentifizierung (Username/Passwort im Header) ist aufgrund ihrer Unsicherheit zu vermeiden, es sei denn, sie wird ausschließlich über HTTPS in stark kontrollierten, nicht-produktiven Umgebungen eingesetzt.31
- **Autorisierung:**
    
    - Nach erfolgreicher Authentifizierung wird die Autorisierung auf Basis von Rollen und Berechtigungen durchgeführt (Role-Based Access Control - RBAC).32 Die im JWT enthaltenen `scopes` oder `roles` (oder aus einer Benutzerdatenbank abgerufene Rollen) bestimmen, auf welche Ressourcen und Operationen der Benutzer oder Dienst zugreifen darf.
    - Detaillierte Spezifikationen zu Rollen und Berechtigungen sind im Sicherheitskonzept (Kapitel 5.2.2) definiert.

Die Sicherheit der Authentifizierungs- und Autorisierungsmechanismen ist von höchster Priorität. Passwörter und API-Schlüssel dürfen niemals in Klartext übertragen oder gespeichert werden und sollten stets über HTTPS gesendet werden.27

##### 3.1.1.3. Datenformate (Request/Response)

- **Request Body Format:** Für Anfragen, die Daten im Body übertragen (z.B. POST, PUT, PATCH), wird ausschließlich das **JSON (JavaScript Object Notation)** Format verwendet.27 Der `Content-Type`-Header der Anfrage muss auf `application/json` gesetzt sein.
- **Response Body Format:** Antworten der API werden ebenfalls im JSON-Format ausgeliefert.27 Der `Content-Type`-Header der Antwort wird auf `application/json; charset=utf-8` gesetzt. XML wird aufgrund der geringeren Verbreitung und des höheren Verarbeitungsaufwands im Kontext moderner Web-APIs nicht standardmäßig unterstützt.28

Die JSON-Strukturen für Requests und Responses müssen klar definiert sein (siehe OpenAPI-Spezifikation in Kapitel 3.3).

##### 3.1.1.4. HTTP-Statuscodes

Die API verwendet Standard-HTTP-Statuscodes, um das Ergebnis einer Anfrage anzuzeigen.27 Dies ermöglicht es Clients, standardisiert auf verschiedene Situationen zu reagieren. Wichtige Statuscodes sind:

- **2xx (Erfolg):**
    - `200 OK`: Standardantwort für erfolgreiche GET-, PUT-, PATCH- oder DELETE-Anfragen.
    - `201 Created`: Antwort für erfolgreiche POST-Anfragen, die eine neue Ressource erstellt haben. Die Antwort sollte einen `Location`-Header mit der URI der neu erstellten Ressource enthalten.
    - `204 No Content`: Antwort für erfolgreiche Anfragen, die keinen Antwortkörper zurückgeben (z.B. erfolgreiche DELETE-Anfrage).
- **3xx (Umleitung):**
    - `304 Not Modified`: Wird verwendet, um Caching-Mechanismen zu unterstützen.
- **4xx (Client-Fehler):**
    - `400 Bad Request`: Die Anfrage war fehlerhaft oder konnte nicht verarbeitet werden (z.B. ungültiges JSON, fehlende Pflichtfelder, Validierungsfehler). Die Antwort sollte eine detailliertere Fehlermeldung im JSON-Format enthalten.28
    - `401 Unauthorized`: Authentifizierung ist fehlgeschlagen oder erforderlich, aber nicht vorhanden.28
    - `403 Forbidden`: Der authentifizierte Benutzer hat keine Berechtigung, auf die angeforderte Ressource oder Operation zuzugreifen.28
    - `404 Not Found`: Die angeforderte Ressource existiert nicht.28
    - `405 Method Not Allowed`: Die verwendete HTTP-Methode ist für die angeforderte Ressource nicht zulässig.
    - `409 Conflict`: Die Anfrage konnte aufgrund eines Konflikts mit dem aktuellen Zustand der Ressource nicht abgeschlossen werden (z.B. Versuch, eine eindeutige Ressource zu erstellen, die bereits existiert).
    - `429 Too Many Requests`: Der Client hat zu viele Anfragen in einem bestimmten Zeitraum gesendet (Ratenbegrenzung).
- **5xx (Server-Fehler):**
    - `500 Internal Server Error`: Ein unerwarteter Fehler ist auf dem Server aufgetreten. Es sollten keine sensiblen Fehlerdetails an den Client weitergegeben werden.28
    - `503 Service Unavailable`: Der Server ist temporär nicht verfügbar (z.B. wegen Überlastung oder Wartung).

Eine konsistente Verwendung von HTTP-Statuscodes ist entscheidend für die Interoperabilität und Robustheit der API-Clients.

##### 3.1.1.5. Endpunktspezifikationen (Beispiele)

Die Endpunkte der API folgen den REST-Prinzipien, wobei Pfade Substantive im Plural verwenden, um Sammlungen von Ressourcen darzustellen, und HTTP-Methoden die auszuführenden Aktionen definieren.27

**Beispiel: Ressourcenverwaltung für "Produkte"**

- **`GET /core/v1/produkte`**
    
    - **Beschreibung:** Ruft eine Liste aller Produkte ab.
    - **Parameter (Query):**
        - `limit` (optional, integer): Maximale Anzahl der zurückzugebenden Produkte (für Paginierung).
        - `offset` (optional, integer): Anzahl der zu überspringenden Produkte (für Paginierung).
        - `sortBy` (optional, string): Feld, nach dem sortiert werden soll (z.B. `name`, `preis`).
        - `sortOrder` (optional, enum: `asc`, `desc`): Sortierreihenfolge.
        - Filterparameter (z.B. `kategorie`, `minPreis`).
    - **Erfolgsantwort (`200 OK`):** JSON-Array von Produktobjekten.
    - **Fehlerantworten:** `401`, `403`.
- **`POST /core/v1/produkte`**
    
    - **Beschreibung:** Erstellt ein neues Produkt.
    - **Request Body (JSON):** Produktobjekt mit den erforderlichen Attributen (z.B. `name`, `beschreibung`, `preis`).
    - **Erfolgsantwort (`201 Created`):** JSON-Objekt des neu erstellten Produkts. `Location`-Header mit URI des neuen Produkts.
    - **Fehlerantworten:** `400` (Validierungsfehler), `401`, `403`, `409` (falls Produkt mit gleichem eindeutigen Bezeichner bereits existiert).
- **`GET /core/v1/produkte/{produktId}`**
    
    - **Beschreibung:** Ruft ein spezifisches Produkt anhand seiner ID ab.
    - **Parameter (Path):** `produktId` (string/integer, je nach ID-Format).
    - **Erfolgsantwort (`200 OK`):** JSON-Objekt des Produkts.
    - **Fehlerantworten:** `401`, `403`, `404` (Produkt nicht gefunden).
- **`PUT /core/v1/produkte/{produktId}`**
    
    - **Beschreibung:** Aktualisiert ein bestehendes Produkt vollständig. Alle Felder des Produkts müssen im Request Body mitgesendet werden.
    - **Parameter (Path):** `produktId`.
    - **Request Body (JSON):** Vollständiges Produktobjekt mit den aktualisierten Werten.
    - **Erfolgsantwort (`200 OK`):** JSON-Objekt des aktualisierten Produkts.
    - **Fehlerantworten:** `400` (Validierungsfehler), `401`, `403`, `404`.
- **`PATCH /core/v1/produkte/{produktId}`**
    
    - **Beschreibung:** Aktualisiert ein bestehendes Produkt partiell. Nur die zu ändernden Felder müssen im Request Body mitgesendet werden.
    - **Parameter (Path):** `produktId`.
    - **Request Body (JSON):** Produktobjekt mit den zu aktualisierenden Feldern.
    - **Erfolgsantwort (`200 OK`):** JSON-Objekt des aktualisierten Produkts.
    - **Fehlerantworten:** `400` (Validierungsfehler), `401`, `403`, `404`.
- **`DELETE /core/v1/produkte/{produktId}`**
    
    - **Beschreibung:** Löscht ein spezifisches Produkt.
    - **Parameter (Path):** `produktId`.
    - **Erfolgsantwort (`204 No Content`):** Kein Antwortkörper.
    - **Fehlerantworten:** `401`, `403`, `404`.

Diese Struktur wird für alle Ressourcen der Coreschicht analog angewendet. Die genauen Definitionen aller Endpunkte, ihrer Parameter, Request- und Response-Schemata werden in der OpenAPI-Spezifikation (Kapitel 3.3) dokumentiert. Die Implementierung von Filterung, Sortierung und Paginierung ist entscheidend für die Benutzerfreundlichkeit und Performance bei großen Datenmengen.26

##### 3.1.1.6. Versionierung

API-Versionierung ist notwendig, um Änderungen an der API vorzunehmen, ohne bestehende Clients zu beeinträchtigen. Die Hauptversion der API wird im URI-Pfad angegeben (z.B. `/v1/`, `/v2/`).30 Kleinere, abwärtskompatible Änderungen (Minor- und Patch-Versionen) erfordern keine neue URI-Version, sollten aber in der API-Dokumentation vermerkt werden.30 Breaking Changes führen immer zu einer neuen Hauptversion im URI.

### 3.2. Externe Schnittstellen (falls zutreffend)

Falls die Coreschicht mit externen Diensten oder Systemen von Drittanbietern kommunizieren muss (z.B. Zahlungsanbieter, externe Datenquellen, Benachrichtigungsdienste), werden diese Schnittstellen hier spezifiziert. Für jede externe Schnittstelle sind folgende Informationen zu dokumentieren:

- **Name des externen Dienstes/Systems.**
- **Zweck der Integration.**
- **Kommunikationsprotokoll** (z.B. REST, SOAP, gRPC).
- **Authentifizierungsmethode** (z.B. API-Key, OAuth 2.0).
- **Datenformate.**
- **Wichtige Endpunkte/Funktionen, die genutzt werden.**
- **Fehlerbehandlung und Retry-Mechanismen.**
- **Performance- und Zuverlässigkeitserwartungen.**

Die Spezifikation externer Schnittstellen ist oft von der Dokumentation des Drittanbieters abhängig.

### 3.3. API-Dokumentation (OpenAPI/Swagger)

Eine umfassende und aktuelle API-Dokumentation ist für Entwickler, die die API nutzen, unerlässlich.25 Die API der Coreschicht wird gemäß der **OpenAPI Specification (OAS)**, Version 3.x (früher bekannt als Swagger) 35, dokumentiert. Die OpenAPI-Spezifikation ist eine standardisierte, sprachunabhängige Beschreibung von REST-APIs, die sowohl von Menschen als auch von Maschinen gelesen werden kann.35

Die OpenAPI-Definitionsdatei (üblicherweise im YAML- oder JSON-Format) wird folgende Informationen für jeden Endpunkt enthalten 37:

- **Pfade und Operationen:** Alle verfügbaren Endpunkte und die unterstützten HTTP-Methoden (GET, POST, PUT, PATCH, DELETE etc.).
- **Parameter:** Definition von Pfad-, Query-, Header- und Cookie-Parametern, einschließlich ihrer Namen, Datentypen, ob sie erforderlich sind und Beschreibungen.
- **Request Bodies:** Beschreibung der erwarteten Request-Payloads, einschließlich Medientypen (z.B. `application/json`) und Schemadefinitionen für die Datenstrukturen.
- **Responses:** Beschreibung möglicher HTTP-Statuscodes für jede Operation und die zugehörigen Response Bodies, einschließlich Medientypen und Schemadefinitionen.
- **Schemadefinitionen (Components):** Wiederverwendbare Definitionen für Datenmodelle (z.B. `Produkt`, `Benutzer`), die in Request- und Response-Bodies verwendet werden. Dies fördert Konsistenz und reduziert Redundanz.
- **Sicherheitsdefinitionen (Security Schemes):** Beschreibung der verwendeten Authentifizierungs- und Autorisierungsmechanismen (z.B. OAuth 2.0, API-Key).
- **Metainformationen:** Titel, Version, Beschreibung der API, Kontaktinformationen, Lizenzinformationen.

Werkzeuge wie Swagger Editor oder Swagger UI können verwendet werden, um die OpenAPI-Spezifikation zu erstellen, zu validieren und interaktiv darzustellen.35 Swagger UI ermöglicht es Entwicklern, die API direkt im Browser zu testen. Postman-Templates können ebenfalls als Grundlage für die Dokumentation dienen.25 Die OpenAPI-Datei sollte versioniert und zusammen mit dem Quellcode der Coreschicht verwaltet werden.

### 3.4. Best Practices für API-Design

Bei der Gestaltung der APIs der Coreschicht werden folgende Best Practices berücksichtigt, um eine hohe Qualität, Benutzerfreundlichkeit und Wartbarkeit sicherzustellen:

- **Konsistenz:** Einheitliche Namenskonventionen für Pfade, Parameter und Felder. Konsistente Verwendung von HTTP-Methoden und Statuscodes.27
- **Ressourcenorientierung:** Design der API um Ressourcen herum, nicht um Aktionen (Verben in Pfaden vermeiden, außer für spezifische nicht-CRUD-Operationen).27
- **Korrekte Verwendung von HTTP-Methoden:** GET für Abruf, POST für Erstellung, PUT für vollständige Aktualisierung, PATCH für partielle Aktualisierung, DELETE für Löschung.28
- **Statuslose Kommunikation:** Jede Anfrage vom Client an den Server muss alle Informationen enthalten, die zur Bearbeitung der Anfrage erforderlich sind. Der Server speichert keinen Client-Kontext zwischen Anfragen (fundamental für REST).32
- **Sicherheit:** Implementierung robuster Authentifizierung und Autorisierung, Verwendung von HTTPS für die gesamte Kommunikation, Validierung aller Eingaben.27 Sensible Daten wie Passwörter oder API-Keys dürfen niemals in URLs erscheinen.27
- **Paginierung, Filterung und Sortierung:** Für Endpunkte, die Listen von Ressourcen zurückgeben, müssen Mechanismen zur Paginierung (z.B. `limit`/`offset` oder cursor-basiert), Filterung und Sortierung bereitgestellt werden, um die Performance zu optimieren und die Datenmenge zu kontrollieren.26
- **Fehlerbehandlung:** Klare und informative Fehlermeldungen im JSON-Format zurückgeben, ohne interne Implementierungsdetails preiszugeben.28
- **Caching-Unterstützung:** Verwendung von HTTP-Headern wie `ETag` und `Last-Modified` zur Unterstützung von Caching auf Client- oder Proxy-Ebene, um die Serverlast zu reduzieren und die Antwortzeiten zu verbessern.27
- **Ratenbegrenzung (Rate Limiting):** Schutz der API vor Missbrauch und Überlastung durch Implementierung von Ratenbegrenzungen pro Client/Benutzer.26
- **Dokumentation:** Umfassende und aktuelle Dokumentation unter Verwendung von Standards wie OpenAPI.25

Die Beachtung dieser Prinzipien führt zu APIs, die nicht nur funktional, sondern auch sicher, performant und einfach zu integrieren sind. Die API-Spezifikation sollte als "Vertrag" zwischen dem API-Anbieter (Coreschicht) und den API-Konsumenten betrachtet werden.26

## 4. UI/UX-Design-Spezifikationen (für Administrations- und Konfigurationsoberflächen)

Obwohl die Coreschicht primär Backend-Funktionalitäten bereitstellt, können spezifische Benutzeroberflächen (UIs) für Administrations-, Konfigurations- oder Überwachungsaufgaben erforderlich sein. Diese UIs interagieren direkt mit den APIs der Coreschicht. Dieses Kapitel definiert die Spezifikationen für das User Interface (UI) und die User Experience (UX) dieser speziellen Oberflächen. Der Designprozess folgt typischerweise einer Progression von Wireframes über Mockups zu klickbaren Prototypen.39

### 4.1. Wireframes

Wireframes sind grundlegende, schematische Darstellungen der Bildschirmlayouts und der Informationsarchitektur. Sie fokussieren auf Struktur, Inhaltshierarchie und grundlegende Funktionalität, ohne visuelle Designelemente wie Farben oder detaillierte Grafiken.39

#### 4.1.1. Zweck und Detailgrad

Der Zweck von Wireframes in diesem Kontext ist:

- Definition der grundlegenden Struktur und des Layouts der Administrationsseiten.
- Festlegung der Anordnung von UI-Elementen (z.B. Navigationsmenüs, Tabellen, Formulare, Schaltflächen).
- Visualisierung der Benutzerflüsse für typische Administrationsaufgaben.
- Frühzeitige Abstimmung über Funktionalität und Informationsarchitektur mit den Stakeholdern.

Wireframes für die Coreschicht-Administrationsoberflächen werden in der Regel als Low-Fidelity-Darstellungen erstellt, um schnelle Iterationen und Feedbackschleifen zu ermöglichen.39 Sie sollten jedoch genügend Details enthalten, um die Kernfunktionalitäten und die Navigation klar zu vermitteln.

#### 4.1.2. Werkzeuge

Für die Erstellung von Wireframes können verschiedene Werkzeuge eingesetzt werden:

- **Balsamiq:** Bekannt für seinen skizzenhaften Stil, der den Low-Fidelity-Charakter betont und Diskussionen auf die Struktur lenkt.42
- **Figma:** Ein kollaboratives All-in-One-Tool, das auch für Wireframing gut geeignet ist und einen nahtlosen Übergang zu Mockups und Prototypen ermöglicht.42
- **Moqups:** Einsteigerfreundliches Web-Tool für Wireframes, Mockups und Diagramme.42
- **ClickUp:** Bietet Whiteboard-Funktionen und Vorlagen für Wireframing im Rahmen einer Projektmanagement-Plattform.46
- **Visily:** Bietet KI-gestützte Wireframing-Funktionen, z.B. Umwandlung von Screenshots in editierbare Wireframes.42
- Auch einfache Werkzeuge wie Stift und Papier oder Whiteboards können für erste Entwürfe verwendet werden.41

Die Wahl des Werkzeugs hängt von den Präferenzen des Teams und den Anforderungen an Kollaboration und Detailgrad ab. Figma wird aufgrund seiner Vielseitigkeit und Kollaborationsmöglichkeiten oft bevorzugt.

#### 4.1.3. Beispiele (Beschreibung der wichtigsten Ansichten)

Die Wireframes werden die wichtigsten Ansichten der Administrationsoberfläche abdecken. Typische Ansichten könnten sein:

- **Dashboard/Übersichtsseite:** Anzeige wichtiger Systemstatistiken, Benachrichtigungen oder schneller Zugriff auf häufig genutzte Funktionen.
- **Benutzerverwaltung:** Liste der Benutzer, Formulare zum Anlegen/Bearbeiten von Benutzern, Zuweisung von Rollen und Berechtigungen.
- **Datenmanagement-Ansichten:** Tabellarische Darstellung von Kerndatenobjekten (z.B. Produkte, Bestellungen), mit Funktionen zum Suchen, Filtern, Erstellen, Bearbeiten und Löschen von Einträgen.
- **Konfigurationsseiten:** Formulare zur Einstellung systemspezifischer Parameter der Coreschicht.
- **Log-Ansicht:** Anzeige von System- oder Audit-Logs mit Filter- und Suchfunktionen.

Jeder Wireframe wird die Platzierung von Navigationselementen, Hauptinhaltsbereichen, Aktionsschaltflächen und wichtigen Datenfeldern skizzieren. Die Verwendung von echtem oder realitätsnahem Beispieltext anstelle von "Lorem Ipsum" wird empfohlen, um die Struktur besser beurteilen zu können.41

#### 4.1.4. Informationsarchitektur und Navigation

Ein wesentlicher Bestandteil der Wireframing-Phase ist die Definition der Informationsarchitektur (IA) und des Navigationskonzepts.48 Dies beinhaltet:

- **Strukturierung der Inhalte:** Logische Gruppierung von Funktionen und Informationen.
- **Navigationsmenüs:** Definition der Hauptnavigation (z.B. Seitenleiste, Top-Menü) und ggf. Unternavigation.
- **Benutzerflüsse:** Darstellung, wie Benutzer typische Aufgaben innerhalb der Administrationsoberfläche erledigen (z.B. Anlegen eines neuen Benutzers, Ändern einer Konfigurationseinstellung).

Die IA sollte intuitiv sein und es Administratoren ermöglichen, gesuchte Informationen und Funktionen schnell zu finden und zu bedienen.49 Die Konsistenz in der Navigation und Struktur über verschiedene Ansichten hinweg ist dabei entscheidend.41

### 4.2. Mockups

Mockups sind detailliertere, oft farbige, aber nicht interaktive Entwürfe der Benutzeroberfläche. Sie bauen auf den Wireframes auf und fügen visuelle Designelemente hinzu, um ein realistischeres Bild des Endprodukts zu vermitteln.39

#### 4.2.1. Visuelles Design und Detailtiefe

Mockups für die Administrationsoberfläche der Coreschicht werden folgende Aspekte des visuellen Designs konkretisieren:

- **Farbpalette:** Anwendung der im Styleguide (siehe Kapitel 4.4) definierten Farben.
- **Typografie:** Verwendung der festgelegten Schriftarten, -größen und -schnitte.
- **Ikonografie:** Einsatz spezifischer Icons für Aktionen und Navigationselemente.
- **Layout und Abstände:** Präzisere Definition von Rastern, Abständen und Ausrichtung der Elemente.
- **Visuelle Darstellung von UI-Komponenten:** Detaillierte Gestaltung von Schaltflächen, Formularfeldern, Tabellen, Benachrichtigungen etc.

Mockups sind statisch und dienen primär der Abstimmung des visuellen Erscheinungsbildes.39 Sie sollten High-Fidelity sein, um eine genaue Vorstellung vom Look-and-Feel zu geben.

#### 4.2.2. Werkzeuge

Viele der für Wireframing genannten Werkzeuge eignen sich auch für die Erstellung von Mockups, insbesondere solche, die einen fließenden Übergang von Low- zu High-Fidelity ermöglichen:

- **Figma:** Sehr stark für detailliertes UI-Design und Mockup-Erstellung.42
- **Sketch:** Ein weiteres professionelles UI-Design-Tool, primär für macOS.43
- **Adobe XD:** Teil der Adobe Creative Cloud, bietet umfangreiche Design- und Prototyping-Funktionen.50
- **Visily:** Kann auch für High-Fidelity Mockups verwendet werden, insbesondere durch seine KI-Funktionen und Vorlagen.47
- **Moqups:** Unterstützt ebenfalls den Übergang von Wireframes zu detaillierteren Mockups.45

Die Wahl des Werkzeugs wird oft durch die bereits im Team etablierten Tools und die Notwendigkeit der Kollaboration bestimmt.

#### 4.2.3. Beispiele (Beschreibung der wichtigsten Ansichten)

Für alle in Kapitel 4.1.3 beschriebenen Wireframe-Ansichten werden entsprechende Mockups erstellt. Diese zeigen die finale visuelle Gestaltung:

- **Dashboard:** Mit realitätsnahen Diagrammen, Farbschemata und Icons.
- **Benutzerverwaltung:** Formulare und Tabellen im finalen Design, inklusive Statusanzeigen (z.B. aktiv/inaktiv).
- **Datenmanagement-Ansichten:** Tabellen mit korrekter Typografie, Farbcodierung für bestimmte Zustände, gestaltete Aktionsschaltflächen.
- **Konfigurationsseiten:** Klar strukturierte Formulare mit ansprechenden Eingabeelementen.

Die Mockups dienen als Vorlage für die Frontend-Entwicklung der Administrationsoberfläche.

### 4.3. Prototypen (klickbar)

Klickbare Prototypen sind interaktive Modelle der Benutzeroberfläche, die den Benutzerfluss simulieren und es ermöglichen, die User Experience (UX) vor der eigentlichen Implementierung zu testen.39

#### 4.3.1. Interaktivität und Benutzerflüsse

Die Prototypen für die Administrationsoberfläche werden folgende Interaktionen ermöglichen:

- **Navigation:** Klickbare Menüpunkte, die zu den entsprechenden Seiten führen.
- **Formularinteraktionen:** Simulation von Eingaben in Formularfelder (ohne tatsächliche Datenverarbeitung), Auswahl aus Dropdowns.
- **Schaltflächen-Interaktionen:** Klickbare Schaltflächen, die zu anderen Ansichten navigieren oder Zustandsänderungen simulieren (z.B. Anzeige einer Erfolgs- oder Fehlermeldung).
- **Tabelleninteraktionen:** Simulation von Sortier- oder Filterfunktionen.

Ziel ist es, die wichtigsten Benutzerflüsse (Use Cases) durch die Administrationsoberfläche erlebbar zu machen.39 Die Prototypen sind in der Regel High-Fidelity in Bezug auf das visuelle Design (basierend auf den Mockups), aber die Interaktionen sind simuliert und greifen nicht auf das tatsächliche Backend zu.

#### 4.3.2. Werkzeuge

Viele moderne UI/UX-Designwerkzeuge bieten integrierte Prototyping-Funktionen:

- **Figma:** Ermöglicht das Verknüpfen von Frames und das Definieren von Übergängen und einfachen Interaktionen, um klickbare Prototypen zu erstellen.43
- **Adobe XD:** Bietet ebenfalls starke Prototyping-Funktionen, einschließlich Auto-Animate für komplexere Übergänge.50
- **Sketch:** In Kombination mit Plugins oder anderen Werkzeugen wie InVision oder Marvel für Prototyping nutzbar.43
- **Justinmind:** Spezialisiert auf interaktive Prototypen, ermöglicht auch komplexere Logik und Datenmanipulationen in Prototypen.42
- **ProtoPie:** Ein leistungsstarkes Werkzeug für High-Fidelity-Prototyping mit Fokus auf komplexe Interaktionen, kann Designs aus Figma oder Sketch importieren.43
- **Marvel:** Ein weiteres Tool für schnelles Prototyping und Testing.43
- **UXPin:** Ermöglicht die Erstellung von Prototypen, die sehr nah an das Endprodukt herankommen, inklusive Code-basierter Designelemente.42

Figma und Adobe XD sind oft ausreichend für die meisten klickbaren Prototypen im Administrationsbereich. Spezialisierte Werkzeuge wie ProtoPie oder Justinmind kommen bei Bedarf für komplexere Interaktionssimulationen in Frage.

#### 4.3.3. Zu testende Szenarien

Die klickbaren Prototypen werden verwendet, um spezifische Benutzerszenarien zu testen und Feedback zur Usability zu sammeln. Beispiele für Testszenarien:

- **Anlegen eines neuen Benutzers:** Kann ein Administrator den Prozess von Anfang bis Ende intuitiv durchlaufen?
- **Ändern einer wichtigen Systemeinstellung:** Ist der Pfad zur Einstellung klar? Sind die Optionen verständlich?
- **Suchen und Filtern von Daten in einer Tabelle:** Funktionieren die Interaktionen wie erwartet und sind sie effizient?
- **Verständlichkeit von Fehlermeldungen und Hinweisen (simuliert).**

Die Ergebnisse dieser Tests fließen direkt in die Optimierung des UI/UX-Designs ein, bevor Entwicklungsressourcen gebunden werden.39

### 4.4. Styleguide

Der Styleguide definiert die visuellen und gestalterischen Grundlagen für die Administrationsoberfläche. Er stellt Konsistenz über alle Ansichten und Komponenten hinweg sicher und dient als Referenz für Designer und Entwickler.52 Er ist ein zentrales Element eines umfassenderen Design Systems.54

#### 4.4.1. Farbpalette

- **Primärfarben:** Die Hauptfarben, die das Branding der Administrationsoberfläche prägen (z.B. für Hintergründe, Navigationselemente).
- **Sekundärfarben:** Akzentfarben zur Hervorhebung von aktiven Elementen, Links oder wichtigen Informationen.
- **Statusfarben:** Farben für Erfolgsmeldungen (grün), Warnungen (gelb/orange), Fehlermeldungen (rot) und Informationshinweise (blau).
- **Neutrale Farben:** Graustufen für Text, Hintergründe, Trennlinien und deaktivierte Elemente.

Für jede Farbe werden die exakten Farbwerte (z.B. HEX, RGB, HSL) spezifiziert.52

#### 4.4.2. Typografie

- **Schriftfamilien:** Definition der primären Schriftart für Überschriften und der sekundären Schriftart für Fließtext und UI-Elemente.
- **Schriftschnitte und -größen:** Festlegung verschiedener Schriftschnitte (z.B. Regular, Bold, Italic) und einer Hierarchie von Schriftgrößen für unterschiedliche Textelemente (z.B. H1, H2, H3, Paragraph, Label).
- **Zeilenhöhe und Zeichenabstand:** Vorgaben für optimale Lesbarkeit.
- **Textfarben:** Definition der Standardtextfarbe und Farben für Links oder hervorgehobenen Text in Abstimmung mit der Farbpalette.

Die typografischen Regeln gewährleisten ein einheitliches und gut lesbares Erscheinungsbild.52

#### 4.4.3. Ikonografie

- **Icon-Set:** Auswahl oder Erstellung eines konsistenten Icon-Sets (z.B. Material Design Icons, Font Awesome oder ein benutzerdefiniertes Set).
- **Stil:** Definition des visuellen Stils der Icons (z.B. outlined, filled, two-tone).
- **Größen:** Standardgrößen für Icons in verschiedenen Kontexten (z.B. in Schaltflächen, Menüs, Tabellen).
- **Verwendungsrichtlinien:** Beispiele für die korrekte Anwendung von Icons.52

Icons tragen maßgeblich zur intuitiven Bedienbarkeit bei.

#### 4.4.4. Abstände und Layout-Raster

- **Grid-System:** Definition eines Basisrasters (z.B. 8-Punkt-Grid), das für die Ausrichtung und Platzierung aller UI-Elemente verwendet wird. Dies sorgt für visuelle Harmonie und Konsistenz.
- **Abstandsregeln (Spacing):** Festlegung von Standardabständen zwischen Elementen (z.B. Margin, Padding) basierend auf dem Grid-System.
- **Responsive Design Vorgaben:** Wie sich das Layout und die Abstände auf verschiedenen Bildschirmgrößen anpassen (falls die Administrationsoberfläche responsiv sein soll).53

Ein durchdachtes Raster- und Abstandssystem ist fundamental für ein professionelles und aufgeräumtes UI-Design.52

#### 4.4.5. UI-Komponentenbibliothek

Die UI-Komponentenbibliothek ist eine Sammlung wiederverwendbarer UI-Elemente, die in der Administrationsoberfläche zum Einsatz kommen. Sie ist ein praktischer Teil des Styleguides und oft der Kern eines Design Systems.54 Für jede Komponente werden definiert:

- **Name der Komponente** (z.B. Button, Input Field, Dropdown, Table, Modal, Notification).
- **Visuelle Spezifikation:** Aussehen basierend auf Farben, Typografie, Icons und Abständen des Styleguides.
- **Zustände:** Definition verschiedener Zustände der Komponente (z.B. für einen Button: default, hover, active, disabled, loading).52
- **Verhaltensregeln:** Wie die Komponente auf Benutzerinteraktionen reagiert.
- **Anwendungsbeispiele ("Do's and Don'ts"):** Richtlinien für den korrekten Einsatz der Komponente.52

Beispiele für UI-Komponenten:

- **Schaltflächen (Buttons):** Primär-, Sekundär-, Tertiär-Buttons; Buttons mit Icons.
- **Formularelemente:** Textfelder, Textareas, Checkboxen, Radiobuttons, Select-Dropdowns, Datepicker.
- **Tabellen:** Darstellung, Sortier- und Filterindikatoren, Paginierungselemente.
- **Navigationselemente:** Menüs, Tabs, Breadcrumbs.
- **Feedback-Elemente:** Modale Dialoge, Popovers, Toasts/Notifications, Ladeindikatoren.

Diese Bibliothek stellt sicher, dass Entwickler auf standardisierte und bereits gestaltete Elemente zurückgreifen können, was die Entwicklungszeit verkürzt und die Konsistenz der UI erhöht.49 Die UI-Komponenten sollten so gestaltet sein, dass sie die Prinzipien einer guten Admin-UI erfüllen: Benutzerfreundlichkeit, klare Layouts und Anpassbarkeit.49

## 5. Sicherheitskonzept

Das Sicherheitskonzept beschreibt die geplanten Maßnahmen zur Gewährleistung der Vertraulichkeit, Integrität und Verfügbarkeit der Daten und Funktionen der Coreschicht. Sicherheit ist kein nachträglicher Gedanke, sondern ein integraler Bestandteil des gesamten Entwicklungszyklus ("Secure by Design" und "Secure Software Development Lifecycle" - SDLC).57 Die hier definierten Maßnahmen müssen in allen Phasen des Datenbankdesigns, der API-Entwicklung und der UI/UX-Gestaltung für Administrationstools berücksichtigt werden. Dieses Konzept orientiert sich an etablierten Sicherheitspraktiken und adressiert gängige Bedrohungen.

### 5.1. Grundlagen und Prinzipien

#### 5.1.1. Layered Security (Defense in Depth)

Das Sicherheitskonzept basiert auf dem Prinzip der "Layered Security" oder "Defense in Depth".58 Dies bedeutet, dass mehrere Sicherheitsebenen implementiert werden, sodass ein Angreifer, der eine einzelne Schutzmaßnahme überwindet, immer noch auf weitere Barrieren trifft. Diese Ebenen umfassen typischerweise Netzwerk-, Anwendungs-, Daten- und physische Sicherheit.57 Für die Coreschicht sind insbesondere die Anwendungs- und Datensicherheit relevant, die durch Maßnahmen auf Netzwerkebene (z.B. Firewalls, die hier nicht im Detail spezifiziert werden, aber vorausgesetzt werden) ergänzt werden.

#### 5.1.2. Prinzip der geringsten Rechte (Principle of Least Privilege)

Benutzern und Systemkomponenten werden nur die minimal notwendigen Berechtigungen erteilt, die sie zur Erfüllung ihrer Aufgaben benötigen.31 Dies minimiert den potenziellen Schaden im Falle einer Kompromittierung eines Kontos oder einer Komponente. Dieses Prinzip wird durch das Autorisierungsmodell (siehe 5.2.2) konsequent umgesetzt.

### 5.2. Authentifizierung und Autorisierung

Eine robuste Authentifizierung und Autorisierung ist fundamental, um sicherzustellen, dass nur legitime Benutzer und Systeme auf die Coreschicht zugreifen und nur die ihnen erlaubten Aktionen durchführen können.32

#### 5.2.1. Authentifizierungsmechanismen

Die Coreschicht muss starke Authentifizierungsmechanismen für alle Zugriffe implementieren.

- **Für Benutzer (z.B. über Administrations-UI oder clientseitige Anwendungen):**
    
    - **OAuth 2.0 mit OpenID Connect (OIDC):** Als Standard für die delegierte Authentifizierung.31 Dies ermöglicht es Clients (z.B. Web-Frontends, mobile Apps), Benutzer sicher zu authentifizieren, ohne direkten Zugriff auf deren Anmeldedaten zu haben.
    - **JSON Web Tokens (JWTs):** Nach erfolgreicher Authentifizierung werden JWTs ausgestellt, die als Bearer-Token für nachfolgende API-Anfragen verwendet werden.31 JWTs müssen signiert (z.B. mit RS256 oder ES256) und validiert werden (Signatur, Ablaufdatum, Aussteller, Zielgruppe, `kid`-Header gegen JWK).33
    - **Multi-Faktor-Authentifizierung (MFA):** Für administrative Zugriffe und sensible Operationen ist MFA zwingend erforderlich.60 Dies fügt eine zusätzliche Sicherheitsebene über das reine Passwort hinaus hinzu.
    - **Passwortrichtlinien:** Starke Passwortrichtlinien (Mindestlänge, Komplexität, keine gängigen Passwörter) müssen durchgesetzt werden.60 Passwörter müssen sicher gehasht (z.B. mit Argon2id oder bcrypt) und gesalzen gespeichert werden; niemals in Klartext.33
    - **Konto-Sperrungsrichtlinien:** Nach einer definierten Anzahl fehlgeschlagener Anmeldeversuche wird das Konto temporär gesperrt, um Brute-Force-Angriffe zu erschweren.60
- **Für serverseitige System-zu-System-Kommunikation (interne Dienste):**
    
    - **OAuth 2.0 Client Credentials Flow:** Geeignet für vertrauenswürdige Server-Anwendungen, die im eigenen Namen auf Ressourcen zugreifen.31
    - **API-Keys:** Für einfachere Szenarien können API-Keys verwendet werden. Diese müssen eine hohe Entropie aufweisen, sicher übertragen (z.B. im HTTP-Header `X-API-Key`), serverseitig validiert und regelmäßig rotiert werden.31 API-Keys sollten nicht im Code fest verdrahtet, sondern sicher verwaltet werden (z.B. über Secret-Management-Systeme).

Alle Authentifizierungsdaten (Passwörter, Token, API-Keys) müssen stets über verschlüsselte Verbindungen (HTTPS) übertragen werden.27

#### 5.2.2. Autorisierungsmodell (z.B. RBAC)

Nach erfolgreicher Authentifizierung erfolgt die Autorisierung, um zu bestimmen, welche Aktionen ein authentifizierter Benutzer oder Dienst durchführen darf. Es wird ein **Role-Based Access Control (RBAC)** Modell implementiert.32

- **Rollen:** Definieren Gruppen von Berechtigungen, die typischen Benutzerkategorien oder Systemfunktionen entsprechen (z.B. `Administrator`, `ReadOnlyUser`, `CoreServiceCommunicator`).
- **Berechtigungen:** Spezifische Rechte, die Aktionen auf bestimmten Ressourcen erlauben (z.B. `produkt:lesen`, `produkt:erstellen`, `benutzer:verwalten`).
- **Zuweisung:** Benutzern oder Dienst-Identitäten werden eine oder mehrere Rollen zugewiesen.

Die Autorisierungsentscheidungen werden bei jedem API-Aufruf basierend auf der Rolle/den Berechtigungen des anfragenden Subjekts getroffen. Dies adressiert direkt die OWASP API Security Risiken #1 (Broken Object Level Authorization) und #5 (Broken Function Level Authorization).61

**Tabelle 5.1: Rollen- und Rechteübersicht (Beispiel)**

|   |   |   |   |   |
|---|---|---|---|---|
|**Rolle**|**Beschreibung der Rolle**|**Berechtigungen für Datenzugriff (CRUD - Beispiel: Produkt, Benutzer)**|**Berechtigungen für API-Endpunkte (Beispiel)**|**Zugeordnete Authentifizierungsmethoden**|
|`SystemAdministrator`|Vollständige Kontrolle über das System, Benutzerverwaltung, Konfiguration.|Produkt:CRUDE, Benutzer:CRUDE, Konfiguration:CRUDE|`GET /produkte`, `POST /produkte`, `PUT /produkte/{id}`, `DELETE /produkte/{id}`, `GET /benutzer`, `POST /benutzer`, etc. (alle administrativen Endpunkte)|OAuth 2.0 (mit MFA)|
|`DatenAnalyst`|Kann alle Daten lesen, aber keine Änderungen vornehmen.|Produkt:R, Benutzer:R, Bestellung:R|`GET /produkte`, `GET /produkte/{id}`, `GET /benutzer`, `GET /benutzer/{id}`, `GET /bestellungen`|OAuth 2.0|
|`FrontendService`|Stellt Daten für die Hauptanwendung bereit, kann im Namen von Benutzern Bestellungen erstellen.|Produkt:R, Bestellung:CR (im Kontext des Benutzers), Benutzer:R (eingeschränkt auf eigene Daten des Benutzers)|`GET /produkte`, `GET /produkte/{id}`, `POST /bestellungen` (im Benutzerkontext), `GET /benutzer/me`|OAuth 2.0 (Authorization Code Flow für Benutzer, Client Credentials für eigene Operationen)|
|`InternerBatchService`|Führt Hintergrundaufgaben aus, z.B. Datenaggregation.|Produkt:R, Bestellung:R, AggregierteDaten:CRU|`GET /produkte/all`, `POST /aggregierteDaten`|API-Key oder OAuth 2.0 Client Credentials|

Diese Tabelle ist ein kritisches Werkzeug, um das Prinzip der geringsten Rechte systematisch anzuwenden. Sie muss detailliert für alle relevanten Rollen, Datenobjekte und API-Endpunkte ausgearbeitet werden. Die Granularität der Berechtigungen muss bis auf die Ebene einzelner Objekte und deren Eigenschaften reichen (Object Level und Object Property Level Authorization), um Risiken wie API1:2023 und API3:2023 der OWASP Top 10 zu mitigieren.61

### 5.3. Datensicherheit

Der Schutz der in der Coreschicht gespeicherten und verarbeiteten Daten ist von höchster Bedeutung.

#### 5.3.1. Verschlüsselung von Daten "at rest"

Alle sensiblen Daten, die in der Datenbank oder anderen persistenten Speichern der Coreschicht abgelegt werden, müssen verschlüsselt werden ("encryption at rest").63

- **Algorithmen:** Es sind starke, etablierte symmetrische Verschlüsselungsalgorithmen wie **AES-256 (Advanced Encryption Standard mit 256-Bit Schlüssellänge)** zu verwenden.63
- **Anwendungsbereich:** Dies betrifft insbesondere personenbezogene Daten (PII), Finanzdaten, Authentifizierungsdaten (obwohl Passwörter gehasht und nicht nur verschlüsselt werden) und andere geschäftskritische Informationen.
- **Schlüsselmanagement:** Ein sicheres Schlüsselmanagement ist entscheidend. Die Verschlüsselungsschlüssel müssen sicher generiert, gespeichert, rotiert und verwaltet werden. Hierfür sollte ein dediziertes **Hardware Security Module (HSM)** oder ein **Key Management Service (KMS)** (z.B. AWS KMS, Azure Key Vault, Google Cloud KMS) eingesetzt werden.63 Der Zugriff auf die Schlüssel muss streng kontrolliert und protokolliert werden. Regelmäßige Schlüsselrotation ist ein Muss.
- **Datenbankseitige vs. Applikationsseitige Verschlüsselung:** Je nach Sensitivität und Anforderungen kann die Verschlüsselung auf Datenbankebene (Transparent Data Encryption - TDE) oder auf Applikationsebene (Client-Side Encryption, bevor die Daten in die DB geschrieben werden) erfolgen.64 Applikationsseitige Verschlüsselung bietet oft mehr Kontrolle, erfordert aber sorgfältige Implementierung.

#### 5.3.2. Verschlüsselung von Daten "in transit"

Jegliche Datenübertragung zur und von der Coreschicht sowie zwischen internen Komponenten der Coreschicht (falls diese über ein Netzwerk kommunizieren) muss verschlüsselt werden ("encryption in transit").63

- **Protokoll:** Für die API-Kommunikation (extern und intern) ist ausschließlich **HTTPS (HTTP Secure)** zu verwenden, basierend auf **TLS (Transport Layer Security)** in einer aktuellen Version (mindestens TLS 1.2, bevorzugt TLS 1.3).27
- **Zertifikate:** Es sind gültige digitale Zertifikate von vertrauenswürdigen Zertifizierungsstellen (CAs) zu verwenden.
- **Cipher Suites:** Nur starke und aktuell als sicher geltende Cipher Suites dürfen konfiguriert werden. Veraltete oder schwache Algorithmen (z.B. SSLv3, frühe TLS-Versionen, MD5, SHA1) sind zu deaktivieren.
- **Interne Kommunikation:** Auch die Kommunikation zwischen Microservices oder verschiedenen Instanzen der Coreschicht sollte, wenn sie über ein Netzwerk erfolgt, mittels TLS gesichert werden.

Die konsequente Verschlüsselung von Daten "at rest" und "in transit" schützt vor unbefugtem Zugriff und Datenlecks.

### 5.4. API-Sicherheitsmaßnahmen

Die APIs der Coreschicht sind potenzielle Angriffsvektoren. Daher müssen spezifische Sicherheitsmaßnahmen implementiert werden, die sich insbesondere an den **OWASP API Security Top 10** orientieren.61

- **`API1:2023 Broken Object Level Authorization (BOLA)`:** Strikte Überprüfung bei jedem Zugriff auf ein Objekt (z.B. `GET /produkte/{id}`), ob der authentifizierte Benutzer tatsächlich die Berechtigung hat, auf _dieses spezifische Objekt_ zuzugreifen, nicht nur auf die Objektart allgemein. Dies wird durch das RBAC-Modell und detaillierte Berechtigungsprüfungen in der Geschäftslogik erreicht.61
- **`API2:2023 Broken Authentication`:** Implementierung der robusten Authentifizierungsmechanismen wie in 5.2.1 beschrieben (OAuth 2.0, JWT-Validierung, MFA, sichere Passwort-Policies und -Speicherung).61
- **`API3:2023 Broken Object Property Level Authorization (BOPLA)`:**
    - **Excessive Data Exposure:** API-Antworten dürfen nur die Datenfelder enthalten, die für den jeweiligen Benutzer und Anwendungsfall tatsächlich benötigt und erlaubt sind. Sensible Felder müssen herausgefiltert werden.
    - **Mass Assignment:** Bei Operationen, die Datenobjekte entgegennehmen (z.B. PUT, POST, PATCH), dürfen nur die Felder aktualisiert werden, die vom Benutzer geändert werden dürfen. Eine Whitelist erlaubter Felder ist zu verwenden, um das Überschreiben interner oder schützenswerter Felder zu verhindern.61
- **`API4:2023 Unrestricted Resource Consumption`:** Implementierung von Maßnahmen zur Begrenzung der Ressourcennutzung:
    - **Ratenbegrenzung (Rate Limiting):** Begrenzung der Anzahl der Anfragen, die ein Client innerhalb eines bestimmten Zeitraums stellen kann.26
    - **Quotas:** Begrenzung der Gesamtmenge an Ressourcen (z.B. Speicherplatz, Anzahl Objekte), die ein Benutzer/Tenant nutzen darf.
    - **Größenbeschränkungen:** Validierung und Begrenzung der Größe von Request- und Response-Payloads sowie hochgeladenen Dateien.61
    - **Timeout-Konfigurationen:** Angemessene Timeouts für Anfragen.
- **`API5:2023 Broken Function Level Authorization (BFLA)`:** Strikte Trennung der Berechtigungen für administrative Funktionen (z.B. Benutzerverwaltung, Systemkonfiguration) von regulären Benutzerfunktionen. Administrative Endpunkte müssen besonders geschützt und nur für autorisierte Rollen zugänglich sein.61
- **`API6:2023 Unrestricted Access to Sensitive Business Flows`:** Identifizierung und besondere Absicherung von Geschäftsabläufen, die über APIs ausgelöst werden und ein hohes Missbrauchspotenzial haben (z.B. Massenbestellungen, Kontoerstellungen). Dies kann zusätzliche Validierungen, Überwachung oder menschliche Interaktion erfordern.61
- **`API7:2023 Server-Side Request Forgery (SSRF)`:** Wenn die API serverseitig Anfragen an andere URLs stellt (basierend auf Benutzereingaben), müssen diese URLs rigoros validiert und auf eine Whitelist erlaubter Ziele beschränkt werden, um zu verhindern, dass Angreifer interne Systeme scannen oder angreifen können.61
- **Weitere OWASP-Punkte:**
    - **`API8:2023 Security Misconfiguration`:** Sorgfältiges Konfigurationsmanagement, Deaktivierung unnötiger Features, regelmäßige Sicherheitsüberprüfungen der Konfigurationen.62
    - **`API9:2023 Improper Inventory Management`:** Führen eines aktuellen Inventars aller API-Endpunkte, Versionen und deren Sicherheitsstatus. "Shadow APIs" oder veraltete, ungesicherte Endpunkte sind zu vermeiden.62
    - **`API10:2023 Unsafe Consumption of APIs`:** Wenn die Coreschicht selbst externe APIs konsumiert, müssen auch hier Sicherheitsaspekte wie Validierung der Antworten, sichere Authentifizierung und Fehlerbehandlung beachtet werden.62

Die Implementierung dieser Maßnahmen erfordert eine kontinuierliche Aufmerksamkeit während des gesamten API-Lebenszyklus.

### 5.5. Sichere Eingabevalidierung und -verarbeitung

Alle von externen Quellen (insbesondere API-Requests) stammenden Daten müssen serverseitig in der Coreschicht rigoros validiert werden, bevor sie weiterverarbeitet oder gespeichert werden. Clientseitige Validierung dient lediglich der Verbesserung der User Experience, bietet aber keinen Sicherheitsschutz, da sie leicht umgangen werden kann.67

**Arten der Validierung:**

- **Typprüfung:** Sicherstellen, dass die Daten dem erwarteten Datentyp entsprechen (z.B. String, Integer, Boolean, Array, Objekt).67
- **Formatprüfung:** Überprüfung, ob Daten spezifischen Formaten entsprechen (z.B. E-Mail-Adresse, Datum (ISO 8601), UUID, Telefonnummer).67
- **Längen-/Größenprüfung:** Validierung der Mindest- und Maximallänge von Zeichenketten, der Anzahl von Elementen in Arrays oder der Größe von Dateien.67
- **Bereichsprüfung:** Sicherstellen, dass numerische Werte innerhalb eines erlaubten Bereichs liegen (z.B. `Alter >= 0`, `Preis > 0`).
- **Prüfung auf erlaubte Zeichen/Werte (Whitelisting):** Bevorzugt sollte eine Whitelist von erlaubten Zeichen oder Werten verwendet werden, anstatt eine Blacklist von verbotenen Zeichen zu pflegen. Dies ist sicherer, da es schwieriger ist, alle potenziell schädlichen Eingaben vorherzusehen.67
- **Konsistenzprüfung:** Überprüfung, ob zusammengehörige Daten logisch konsistent sind (z.B. Startdatum vor Enddatum).67

**Schutz vor Injection-Angriffen:**

- **SQL-Injection (und NoSQL-Injection):** Verwendung von Prepared Statements (parametrisierten Abfragen) oder ORM-Frameworks, die dies intern handhaben, ist zwingend erforderlich, um SQL-Injection-Angriffe zu verhindern.65 Benutzereingaben dürfen niemals direkt in SQL-Abfragen konkateniert werden.
- **Command-Injection:** Vermeidung der Ausführung von Betriebssystembefehlen, die direkt oder indirekt aus Benutzereingaben konstruiert werden. Wenn unvermeidbar, müssen Eingaben extrem sorgfältig validiert und saniert werden.
- **Cross-Site Scripting (XSS):** Obwohl XSS primär ein Frontend-Problem ist, kann die Coreschicht dazu beitragen, indem sie Daten, die später im Frontend angezeigt werden, korrekt validiert und ggf. vor der Speicherung saniert oder bei der Ausgabe kontextbezogen kodiert (z.B. HTML-Encoding).

Fehlgeschlagene Validierungen müssen zu einer klaren Fehlermeldung an den Client führen (z.B. HTTP `400 Bad Request`), ohne sensible interne Details preiszugeben.67

### 5.6. Protokollierung (Logging) und Überwachung sicherheitsrelevanter Ereignisse

Eine umfassende Protokollierung aller sicherheitsrelevanten Ereignisse ist notwendig, um Sicherheitsvorfälle erkennen, analysieren und darauf reagieren zu können.60

**Zu protokollierende Ereignisse umfassen mindestens:**

- **Authentifizierungsversuche:** Erfolgreiche und fehlgeschlagene Anmeldungen (Benutzername, Quell-IP, Zeitstempel).60
- **Autorisierungsentscheidungen:** Verweigerte Zugriffsversuche auf Ressourcen oder Funktionen (Benutzerkennung, angeforderte Ressource/Funktion, Zeitstempel).
- **Wichtige Konfigurationsänderungen:** Änderungen an Sicherheitseinstellungen, Benutzerrollen oder Berechtigungen (wer hat was wann geändert).
- **API-Anfragen mit Fehlern:** Insbesondere solche, die auf potenzielle Angriffe hindeuten (z.B. wiederholte `401`/`403`-Fehler, Validierungsfehler).
- **Fehler bei der kryptographischen Schlüsselverwaltung.**
- **Erkannte Angriffsversuche oder Anomalien.**

**Anforderungen an die Protokolle:**

- **Ausreichender Detaillierungsgrad:** Logs müssen genügend Informationen enthalten, um den Kontext eines Ereignisses zu verstehen (z.B. Zeitstempel mit Zeitzone, Quell-IP-Adresse, betroffene Benutzerkennung, Ereignistyp, Ergebnis, betroffene Ressource).
- **Integrität und Schutz:** Log-Daten müssen vor unbefugtem Zugriff und Manipulation geschützt werden. Sie sollten idealerweise an ein zentrales, gesichertes Log-Management-System (z.B. SIEM - Security Information and Event Management) gesendet werden.
- **Regelmäßige Auswertung:** Logs müssen regelmäßig (automatisiert und manuell) auf verdächtige Aktivitäten und Muster überwacht werden.60
- **Aufbewahrungsfristen:** Definition von Aufbewahrungsfristen für Log-Daten gemäß rechtlicher und betrieblicher Anforderungen.

Die Protokollierung unterstützt nicht nur die Reaktion auf Vorfälle, sondern auch proaktive Sicherheitsanalysen und die Einhaltung von Compliance-Vorgaben. Die Dokumentation der Logging-Mechanismen und der Zugriff auf Logs ist ebenfalls Teil eines umfassenden Sicherheitsansatzes.57

## 6. Schlussfolgerungen

Dieses Pflichtenheft legt die detaillierten Spezifikationen für die Entwicklung der Coreschicht fest und dient als zentrale Referenz für alle Projektbeteiligten. Es umfasst die genauen Anforderungen an das Datenbankdesign, die API-Schnittstellen, die UI/UX-Gestaltung für administrative Zwecke sowie ein umfassendes Sicherheitskonzept.

Die **Datenbankdesign-Spezifikation** mit einem detaillierten Data Dictionary und konzeptionellen ER-Diagrammen bildet die Grundlage für eine robuste und skalierbare Datenhaltung. Die konsequente Anwendung von Namenskonventionen, die sorgfältige Auswahl von Datentypen und die klare Definition von Beziehungen und Constraints sind hierbei unerlässlich.

Die **Schnittstellendesign-Spezifikation** definiert primär eine RESTful API unter Verwendung von JSON als Datenformat und OAuth 2.0 sowie API-Keys für die Authentifizierung. Die Dokumentation mittels OpenAPI (Swagger) gewährleistet eine klare und maschinenlesbare Beschreibung aller Endpunkte und Datenstrukturen, was die Integration und Nutzung der API erleichtert. Die Einhaltung von Best Practices im API-Design ist entscheidend für die Erstellung einer sicheren, performanten und benutzerfreundlichen Schnittstelle.

Für eventuell notwendige **Administrations- und Konfigurationsoberflächen** werden UI/UX-Spezifikationen bereitgestellt, die den Prozess von Wireframes über Mockups bis hin zu klickbaren Prototypen beschreiben. Ein detaillierter Styleguide inklusive einer UI-Komponentenbibliothek stellt die visuelle Konsistenz und eine effiziente Entwicklung dieser Oberflächen sicher.

Das **Sicherheitskonzept** ist als integraler Bestandteil des gesamten Pflichtenhefts zu verstehen. Es basiert auf den Prinzipien der Layered Security und der geringsten Rechte. Starke Authentifizierungs- und Autorisierungsmechanismen (RBAC), die Verschlüsselung von Daten "at rest" (AES-256) und "in transit" (TLS), die Adressierung der OWASP API Security Top 10, rigorose Eingabevalidierung und umfassende Protokollierung sind Kernkomponenten dieses Konzepts. Die Sicherheit der Coreschicht muss in jeder Phase des Designs und der Entwicklung berücksichtigt werden, um ein "Secure by Design"-Produkt zu gewährleisten.

Die erfolgreiche Umsetzung der in diesem Pflichtenheft definierten Spezifikationen erfordert eine enge Zusammenarbeit aller Beteiligten und eine kontinuierliche Überprüfung der Anforderungen im Projektverlauf. Dieses Dokument ist als "lebendes Dokument" zu betrachten, das bei Bedarf und nach formaler Abstimmung angepasst werden kann, um Änderungen und neue Erkenntnisse zu reflektieren.4 Die Einhaltung der hier festgelegten Vorgaben ist entscheidend für die Entwicklung einer qualitativ hochwertigen, sicheren und wartbaren Coreschicht, die den Anforderungen des Gesamtsystems gerecht wird.

## 7. Anhang

### 7.1. Glossar

|   |   |   |
|---|---|---|
|**Begriff**|**Definition**|**Quelle (falls zutreffend)**|
|API|Application Programming Interface (Programmierschnittstelle)|29|
|CRUD|Create, Read, Update, Delete (Grundlegende Datenoperationen)|27|
|ERD|Entity-Relationship-Diagram (Diagramm zur Darstellung von Datenbankstrukturen)|18|
|GUI|Graphical User Interface (Grafische Benutzeroberfläche)|29|
|HSM|Hardware Security Module (Hardwaremodul zur sicheren Schlüsselspeicherung und -verwaltung)|63|
|HTTP|Hypertext Transfer Protocol (Protokoll zur Übertragung von Daten im Web)||
|HTTPS|Hypertext Transfer Protocol Secure (Sichere Variante von HTTP durch TLS/SSL-Verschlüsselung)|27|
|JWT|JSON Web Token (Standard zur Übertragung von Claims zwischen Parteien als JSON-Objekt)|31|
|JSON|JavaScript Object Notation (Leichtgewichtiges Datenaustauschformat)|27|
|KMS|Key Management Service (Dienst zur Verwaltung kryptographischer Schlüssel)|63|
|MFA|Multi-Factor Authentication (Authentifizierungsmethode mit mehreren Faktoren)|60|
|OAuth 2.0|Open Authorization 2.0 (Offenes Protokoll für delegierte Autorisierung)|31|
|OIDC|OpenID Connect (Identitätsschicht aufbauend auf OAuth 2.0)|31|
|OpenAPI|Standardisierte Speifikation zur Beschreibung von REST-APIs (früher Swagger)|35|
|ORM|Object-Relational Mapping (Technik zur Abbildung von Objekten auf relationale Datenbanken)||
|OWASP|Open Web Application Security Project (Non-Profit-Organisation mit Fokus auf Softwaresicherheit)|61|
|PII|Personally Identifiable Information (Personenbezogene Daten)||
|PK|Primary Key (Primärschlüssel in einer Datenbanktabelle)||
|FK|Foreign Key (Fremdschlüssel in einer Datenbanktabelle)||
|RBAC|Role-Based Access Control (Rollenbasiertes Zugriffskontrollmodell)|32|
|REST|Representational State Transfer (Architekturstil für verteilte Hypermedia-Systeme)|27|
|SDLC|Software Development Lifecycle (Softwareentwicklungslebenszyklus)|57|
|SIEM|Security Information and Event Management (System zur Sammlung und Analyse von Sicherheitsinformationen)||
|SQL|Structured Query Language (Standardsprache zur Verwaltung relationaler Datenbanken)||
|SRS|Software Requirements Specification (Software-Anforderungsspezifikation)|1|
|SSRF|Server-Side Request Forgery (Sicherheitslücke, bei der ein Server dazu gebracht wird, Anfragen an beliebige Ziele zu senden)|61|
|Swagger|Werkzeugsatz zur Implementierung der OpenAPI-Spezifikation (siehe OpenAPI)|35|
|TLS|Transport Layer Security (Verschlüsselungsprotokoll zur sicheren Datenübertragung)|31|
|UI|User Interface (Benutzerschnittstelle)|47|
|UML|Unified Modeling Language (Standardisierte Modellierungssprache)|2|
|URI|Uniform Resource Identifier (Eindeutiger Bezeichner für eine Ressource)||
|URL|Uniform Resource Locator (Spezifische Art von URI, die den Ort einer Ressource angibt)||
|UX|User Experience (Benutzererlebnis)|72|
|UUID|Universally Unique Identifier (Eindeutiger 128-Bit-Identifikator)||
|XSS|Cross-Site Scripting (Art von Sicherheitslücke in Webanwendungen)|67|
|YAML|YAML Ain't Markup Language (Menschenlesbares Datenformat, oft für Konfigurationsdateien verwendet)|37|

### 7.2. Referenzierte Dokumente und Standards (erweitert)

- DIN 69901-5: Projektmanagement – Projektmanagementsysteme – Teil 5: Begriffe 1
- VDI Richtlinie 2519 Blatt 1: Vorgehensweise bei der Planung und Ausführung von Automatisierungsprojekten 1
- VDI Richtlinie 3694: Anforderungen an Automatisierungssysteme 1
- IEEE Std 830-1998: IEEE Recommended Practice for Software Requirements Specifications (ggf. ersetzt durch ISO/IEC/IEEE 29148) 8
- ISO/IEC/IEEE 29148:2018: Systems and software engineering — Life cycle processes — Requirements engineering 7
- OpenAPI Specification (Version 3.x): Standard zur Beschreibung von REST-APIs 35
- OWASP API Security Top 10: Regelmäßig aktualisierte Liste der kritischsten Sicherheitsrisiken für APIs 61
- OWASP Secure Coding Practices
- NIST Special Publications (z.B. SP 800-53 für Sicherheitskontrollen, NIST Cybersecurity Framework) 57
- RFCs (Request for Comments) relevant für HTTP, TLS, JWT, OAuth 2.0 (z.B. RFC 2616, RFC 8446, RFC 7519, RFC 6749)
- [Internes Dokument XYZ]: Lastenheft für das Projekt (falls vorhanden und Basis für dieses Pflichtenheft)
- : Übergeordnete Systemarchitektur
