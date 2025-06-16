# Spezifikation: NovaDE Kernschicht Logging-System (v1.1.0)

<!-- ANCHOR [NovaDE Developers <dev@novade.org>] Version 1.1.0 -->

**Status:** Entwurf (Aktualisiert)

**Datum:** YYYY-MM-DD (Wird beim Merge aktualisiert)

**Verantwortlich:** NovaDE Core Team

**Kurzbeschreibung:** Dieses Dokument beschreibt das Logging-System der NovaDE Kernschicht (`novade-core`), das für die Erfassung, Formatierung und Ausgabe von Log-Meldungen zuständig ist. Diese Version dokumentiert die Implementierung basierend auf dem `tracing` Framework.

## 1. Einleitung und Ziele

Das Logging-System ist eine fundamentale Komponente zur Diagnose, Überwachung und Fehlerbehebung von NovaDE. Es soll flexible Konfigurationsoptionen bieten und strukturierte Logs für eine einfache maschinelle Verarbeitung ermöglichen.

**Ziele:**

*   Bereitstellung eines zentralisierten Logging-Frameworks für alle Kernkomponenten.
*   Unterstützung verschiedener Log-Level (Trace, Debug, Info, Warn, Error).
*   Ermöglichung von strukturierten Logs im JSON-Format.
*   Konfiguration der Log-Ausgabe (Stdout, Datei).
*   Implementierung von Log-Rotation für Dateiausgaben.
*   Laufzeitkonfiguration von Log-Levels über Umgebungsvariablen.

## 2. Systemarchitektur und Implementierung

Das Logging-System basiert auf den folgenden Rust-Crates:

*   **`tracing`**: Ein Framework für instrumentierten Code, das als Fassade für Logging und Tracing dient.
*   **`tracing-subscriber`**: Stellt Implementierungen für `Collector` (Subscriber) bereit, die bestimmen, wie Tracing-Daten verarbeitet und ausgegeben werden. Wird für die Formatierung (JSON, Text) und Filterung (EnvFilter) genutzt.
*   **`tracing-appender`**: Stellt Rolling File Appender für die Log-Rotation bereit.

### 2.1. Initialisierung

Die Logging-Initialisierung erfolgt durch die Funktion `novade_core::logging::init_logging()`. Diese Funktion nimmt eine `LoggingConfig`-Struktur entgegen, die das Verhalten des Loggings steuert.

### 2.2. Strukturierte Logs (JSON)

Wenn als Log-Format `Json` konfiguriert ist, werden Log-Einträge als JSON-Objekte ausgegeben. Jedes Objekt enthält typischerweise:

*   `timestamp`: Zeitstempel der Meldung.
*   `level`: Log-Level (z.B. "INFO", "DEBUG").
*   `message`: Die eigentliche Log-Nachricht.
*   `fields`: Ein Objekt mit zusätzlichen strukturierten Daten, die dem Log-Event beigefügt wurden (z.B. `target`, Modulpfad, spezifische Event-Attribute).
*   `target`: Der Ursprung der Log-Meldung (z.B. `novade_core::config`).

**Beispiel JSON-Log:**
```json
{
  "timestamp": "2023-10-27T10:30:00.123Z",
  "level": "INFO",
  "fields": {
    "message": "System initialisiert.",
    "component": "core-startup"
  },
  "target": "novade_core::initialization"
}
```

### 2.3. Text-Format

Wenn als Log-Format `Text` konfiguriert ist, werden Logs als menschenlesbare Zeilen ausgegeben, typischerweise mit Zeitstempel, Level, Target und Nachricht.

**Beispiel Text-Log:**
```
Oct 27 10:30:00.123 INFO novade_core::initialization: System initialisiert. component=core-startup
```

### 2.4. Log-Level Konfiguration

Die Log-Levels werden primär durch die `RUST_LOG` Umgebungsvariable gesteuert, dank der Integration von `tracing_subscriber::EnvFilter`. Dies ermöglicht eine flexible Anpassung der Log-Ausführlichkeit zur Laufzeit, ohne dass die Anwendung neu kompiliert werden muss.

**Beispiel `RUST_LOG` Syntax:**
`RUST_LOG="info,novade_core=debug,hyper=warn"`

Diese Einstellung würde standardmäßig Logs ab dem Level `INFO` ausgeben, für alle Module innerhalb von `novade_core` jedoch `DEBUG`-Logs und für das `hyper`-Crate nur `WARN`-Logs. Die in der `LoggingConfig` spezifizierte `log_level` dient als Fallback, falls `RUST_LOG` nicht gesetzt ist.

### 2.5. Log-Rotation

Für die Dateiausgabe wird `tracing-appender` verwendet, um Log-Rotation zu implementieren. Die Konfiguration erfolgt über `LoggingConfig.log_output.File.rotation`.

*   **`Daily`**: Log-Dateien werden täglich rotiert. Die rotierten Dateien erhalten typischerweise ein Datum im Dateinamen (z.B. `novade.log.2023-10-27`).
*   **`SizeMB(usize)`**: <!-- ANCHOR [Log Rotation Policy] -->Log-Dateien sollen rotiert werden, wenn sie eine bestimmte Größe in Megabyte überschreiten. **TODO:** Die aktuelle Implementierung mit `tracing_appender::rolling::RollingFileAppender` und `Rotation::NEVER` für diesen Fall ist ein Platzhalter. Eine echte größenbasierte Rotation erfordert eine benutzerdefinierte Logik oder ein anderes Crate, da `tracing-appender` nicht direkt eine kombinierte tägliche UND größenbasierte Rotation oder eine einfache größenbasierte Rotation mit mehreren Backup-Dateien bietet. Derzeit wird bei `SizeMB` eine Warnung ausgegeben und die Datei rotiert nicht.
*   **`None`**: Es findet keine Rotation statt. Die Log-Datei wächst unbegrenzt.

### 2.6. Konfigurationsoptionen

Siehe `docs/CONFIGURATION.md` für detaillierte Informationen zu den Logging-Konfigurationsoptionen (`log_level`, `log_output`, `log_format`, `LogRotation`).

## 3. API Referenz (Auszug)

*   `novade_core::logging::init_logging(config: &LoggingConfig, is_reload: bool)`: Initialisiert das Logging-System.
*   `novade_core::config::LoggingConfig`: Struktur zur Definition der Logging-Konfiguration.
*   Verwendung von `tracing`-Makros (`trace!`, `debug!`, `info!`, `warn!`, `error!`) im gesamten Code.

## 4. Zukünftige Überlegungen und TODOs

*   **TODO [Multi-Process Aggregation]:** Untersuchung und Implementierung von Strategien zur Aggregation von Logs aus mehreren Prozessen (z.B. Compositor, Anwendungsdienste) an einem zentralen Ort oder Dienst (z.B. systemd-journal, ELK-Stack, OpenTelemetry Collector).
*   **TODO [Advanced Filtering]:** Evaluierung komplexerer Filtermechanismen jenseits von `EnvFilter`, falls erforderlich (z.B. dynamische Filterung zur Laufzeit über eine Debug-Schnittstelle, Filterung basierend auf spezifischen Log-Feldern).
*   **TODO [Log Analysis Tools]:** Dokumentation und Empfehlungen für Werkzeuge zur Analyse der strukturierten JSON-Logs.
*   **TODO [Performance Overhead]:** Kontinuierliche Überwachung des Performance-Overheads des Logging-Systems, insbesondere bei sehr detaillierten Log-Levels.
*   **TODO [Log Rotation Policy]:** Vollständige Implementierung der größenbasierten Log-Rotation.

## 5. Anmerkungen zur alten Spezifikation (falls zutreffend)

Diese Version ersetzt frühere Logging-Spezifikationen, die möglicherweise auf anderen Mechanismen basierten. Die Umstellung auf `tracing` bietet verbesserte Flexibilität und Integration mit dem Rust-Ökosystem.

---
*Dieses Dokument ist Teil der NovaDE-Gesamtspezifikation.*
<!-- ANCHOR_END -->
