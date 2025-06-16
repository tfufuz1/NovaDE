# NovaDE Konfiguration

<!-- ANCHOR [NovaDE Developers <dev@novade.org>] Konfigurationsdokument v1.0 -->

Dieses Dokument beschreibt die Konfigurationsoptionen für das NovaDE-System, die typischerweise in einer zentralen `config.toml`-Datei im [XDG Config Dir](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html) (z.B. `~/.config/novade/config.toml`) verwaltet werden. Die Konfiguration wird von der `novade-core` Kiste geladen und verarbeitet.

## Hauptstruktur (`CoreConfig`)

Die Hauptkonfiguration ist in Sektionen unterteilt, die verschiedenen Subsystemen von NovaDE entsprechen.

```toml
# Beispiel config.toml Struktur
[logging]
# ... Logging-Einstellungen ...

[error_tracking]
# ... Error Tracking (Sentry) Einstellungen ...

[metrics_exporter]
# ... Metrics Exporter (Prometheus) Einstellungen ...

[debug_interface]
# ... Debug Interface Einstellungen ...

[feature_flags]
# ... Feature Flag Einstellungen ...

[system_health]
# ... System Health Dashboard Einstellungen ...
```

---

### 1. Logging (`logging`) <!-- ANCHOR [Config Logging] -->

Konfiguriert das Verhalten des Logging-Systems.

*   **`log_level`**: (String)
    *   Beschreibung: Legt den minimalen Loglevel fest, der ausgegeben wird.
    *   Mögliche Werte: `"trace"`, `"debug"`, `"info"`, `"warn"`, `"error"`. Die tatsächliche Filterung kann auch durch die `RUST_LOG` Umgebungsvariable beeinflusst werden, die Vorrang hat.
    *   Standard: `"info"`
    *   Beispiel: `log_level = "debug"`
    *   *TODO [Config Validation]: Sicherstellen, dass nur valide Log-Level akzeptiert werden.*

*   **`log_output`**: (Tabelle oder Inline-Tabelle)
    *   Beschreibung: Definiert das Ziel für Log-Ausgaben.
    *   Mögliche Typen:
        *   **`Stdout`**: Loggt auf die Standardausgabe.
            *   Beispiel: `log_output = { type = "stdout" }` (angenommene TOML-Struktur für Enums mit Varianten)
                          Oder in TOML könnte es einfacher sein, wenn `type` das Schlüsselwort ist: `log_output_type = "stdout"` und dann spezifische Felder. Die Rust-Struktur `enum LogOutput { Stdout, File { ... } }` wird von `serde` entsprechend (de)serialisiert, oft als `type = "Stdout"` oder `File = { path = "...", ... }`. Für dieses Beispiel nehmen wir eine gängige `serde`-Darstellung an.
        *   **`File`**: Loggt in eine Datei. Benötigt zusätzliche Parameter:
            *   `path`: (String) Pfad zur Log-Datei.
                *   Standard (effektiv, wenn `File` gewählt, aber kein Pfad): `"novade-core.log"` (implementierungsabhängig im Logger, sollte aber konfigurierbar sein).
                *   Beispiel: `path = "/var/log/novade/core.log"` oder `path = "novade.log"` (relativ zum Arbeitsverzeichnis oder einem XDG-Datenverzeichnis).
            *   `rotation`: (Tabelle) Definiert die Rotationspolitik. Siehe `LogRotation` unten.
                *   Standard: `{ type = "daily" }`
    *   Standard: `{ type = "stdout" }` (als `LogOutput::Stdout`)
    *   Beispiel `File`:
        ```toml
        [logging.log_output]
        type = "file" # Oder direkt [logging.log_output.file] in manchen TOML Mappings
        path = "/var/log/novade.log"
        rotation = { type = "daily" }
        ```
        Oder mit `serde` und `adjacently tagged` oder `internally tagged` für Enums:
        ```toml
        [logging.log_output]
        type = "File" # Name der Enum-Variante
        path = "/var/log/novade.log"
        rotation = { type = "Daily" } # Name der Enum-Variante
        ```
        Die genaue TOML-Darstellung hängt von der `serde`-Konfiguration der Rust-Enums ab. Dieses Dokument nimmt eine verständliche Form an; die `serde` Implementierung muss dies widerspiegeln. Für `LogRotation`:
        ```toml
        [logging.log_output.rotation] # Wenn `log_output` als Tabelle File ist
        type = "SizeMB"
        value = 100 # Entspricht SizeMB(100)
        ```

*   **`log_rotation`** (innerhalb von `log_output.File`): (Enum / Tabelle) <!-- ANCHOR [Config LogRotation] -->
    *   Beschreibung: Definiert die Rotationspolitik für die Dateiausgabe.
    *   Mögliche Typen / Werte:
        *   **`Daily`**: Tägliche Rotation.
            *   Beispiel: `rotation = { type = "daily" }`
        *   **`SizeMB(usize)`**: Rotation bei Überschreitung der Größe in MB.
            *   Beispiel: `rotation = { type = "sizemb", value = 100 }` (für 100MB)
            *   *TODO [Log Rotation Policy]: Die Implementierung mit `tracing-appender` hat derzeit Einschränkungen bei der kombinierten oder einfachen größenbasierten Rotation. Diese Option ist ein Platzhalter für eine robustere zukünftige Implementierung.*
        *   **`None`**: Keine Rotation.
            *   Beispiel: `rotation = { type = "none" }`
    *   Standard (wenn `log_output.type = "file"`): `Daily`.

*   **`log_format`**: (String)
    *   Beschreibung: Definiert das Format der Log-Nachrichten.
    *   Mögliche Werte: `"Json"`, `"Text"`.
    *   Standard: `"Text"`
    *   Beispiel: `log_format = "Json"`

---

### 2. Error Tracking (`error_tracking`) <!-- ANCHOR [Config ErrorTracking] -->

Konfiguriert das Sentry Error-Tracking-System.

*   **`sentry_dsn`**: (String, optional)
    *   Beschreibung: Der Data Source Name (DSN) für die Sentry-Integration. Wenn nicht gesetzt oder leer, ist Sentry deaktiviert.
    *   Standard: `None` (deaktiviert)
    *   Beispiel: `sentry_dsn = "https://your_key@sentry.io/your_project_id"`

*   **`sentry_environment`**: (String, optional)
    *   Beschreibung: Name der Umgebung für Sentry (z.B. "development", "production").
    *   Standard: `None` (Sentry verwendet ggf. einen Standardwert oder keinen)
    *   Beispiel: `sentry_environment = "production"`

*   **`sentry_release`**: (String, optional)
    *   Beschreibung: Name/Version des Releases für Sentry. Sollte mit der Anwendungsversion übereinstimmen.
    *   Standard: `None` (Sentry versucht ggf., dies aus Cargo-Umgebungsvariablen beim Build zu lesen, falls nicht zur Laufzeit gesetzt)
    *   Beispiel: `sentry_release = "novade-1.2.0"`

---

### 3. Metrics Exporter (`metrics_exporter`) <!-- ANCHOR [Config MetricsExporter] -->

Konfiguriert den Prometheus Metrics Exporter.

*   **`metrics_exporter_enabled`**: (Boolean)
    *   Beschreibung: Aktiviert oder deaktiviert den Prometheus Metrics Exporter.
    *   Standard: `false`
    *   Beispiel: `metrics_exporter_enabled = true`

*   **`metrics_exporter_address`**: (String)
    *   Beschreibung: Die IP-Adresse und der Port, auf dem der Exporter lauschen soll.
    *   Standard: `"0.0.0.0:9090"`
    *   Beispiel: `metrics_exporter_address = "127.0.0.1:9898"`
    *   *TODO [Config Validation]: Sicherstellen, dass die Adresse als `SocketAddr` geparst werden kann.*

---

### 4. Debug Interface (`debug_interface`) <!-- ANCHOR [Config DebugInterface] -->

Konfiguriert die Debug-Schnittstelle für Entwickler.

*   **`debug_interface_enabled`**: (Boolean)
    *   Beschreibung: Aktiviert oder deaktiviert die Debug-Schnittstelle.
    *   Standard: `false`
    *   Beispiel: `debug_interface_enabled = true`

*   **`debug_interface_address`**: (String, optional)
    *   Beschreibung: Die Adresse, auf der die Debug-Schnittstelle lauschen soll (z.B. Pfad zu einem Unix-Socket oder eine IP:Port-Kombination, abhängig von der Implementierung).
    *   Standard: `None`
    *   Beispiel für Unix-Socket: `debug_interface_address = "/tmp/novade-debug.sock"`
    *   *TODO [Config Validation]: Validierung abhängig vom gewählten Transportmechanismus.*

---

### 5. Feature Flags (`feature_flags`) <!-- ANCHOR [Config FeatureFlags] -->

Ermöglicht das Aktivieren oder Deaktivieren von experimentellen oder optionalen Funktionen.

*   **`experimental_feature_x`**: (Boolean)
    *   Beschreibung: Ein Beispiel für einen Feature-Flag.
    *   Standard: `false`
    *   Beispiel: `experimental_feature_x = true`

---

### 6. System Health Dashboard (`system_health`) <!-- ANCHOR [Config SystemHealth] -->

Konfiguration für das System Health Dashboard (Details siehe `SPEC-FEATURE-SYSTEM-HEALTH-DASHBOARD-v0.1.0.md`).

*   **`metric_refresh_interval_ms`**: (Integer)
    *   Beschreibung: Aktualisierungsintervall für Metriken im Dashboard in Millisekunden.
    *   Standard: `1000`
    *   Beispiel: `metric_refresh_interval_ms = 500`

*   **`default_time_range_hours`**: (Integer)
    *   Beschreibung: Standard-Zeitbereich in Stunden für die Anzeige von Verlaufsdaten (z.B. in Graphen).
    *   Standard: `1`
    *   Beispiel: `default_time_range_hours = 24`

---
*Dieses Dokument ist Teil der NovaDE-Gesamtspezifikation.*
<!-- ANCHOR_END -->
