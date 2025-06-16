# NovaDE Developer Guide

<!-- ANCHOR [NovaDE Developers <dev@novade.org>] Developer Guide v0.1 -->

**Status:** Entwurf

**Datum:** YYYY-MM-DD (Wird beim Merge aktualisiert)

## 1. Einleitung

Willkommen beim NovaDE Developer Guide! Dieses Dokument soll Entwicklern den Einstieg in die Entwicklung von und für NovaDE erleichtern. Es enthält Informationen zu Coding-Konventionen, wichtigen Systemkomponenten und Best Practices.

<!-- TODO [NovaDE Developers <dev@novade.org>] Erweitern mit Setup-Anweisungen, Build-Prozess, Debugging-Tipps etc. -->

## 2. Kernkonzepte

<!-- TODO [NovaDE Developers <dev@novade.org>] Beschreiben der Schichtenarchitektur (Core, Domain, System, UI) und wichtiger Design-Prinzipien. -->

## 3. Logging-System verwenden <!-- ANCHOR [DevGuide Logging] -->

NovaDE verwendet das `tracing`-Crate für strukturiertes Logging. Es ist in `novade-core::logging` initialisiert und konfiguriert.

### 3.1. Log-Makros

Verwende die Standard-`tracing`-Makros, um Log-Meldungen auszugeben:

*   `trace!(field1 = "value", field2 = %object_display; "Eine sehr detaillierte Meldung")`
*   `debug!("Eine Debug-Meldung für Entwickler: {:?}", some_data)`
*   `info!("Allgemeine Information: System gestartet.")`
*   `warn!("Warnung: Konfiguration nicht optimal: {}", warning_details)`
*   `error!(error.message = %err, error.details = ?err; "Ein Fehler ist aufgetreten")` (Beispiel für das Loggen von Fehlerdetails mit `tracing::field::display` und `tracing::field::debug`)

**Best Practices:**

*   **Strukturierte Felder:** Füge relevante Kontextinformationen als strukturierte Felder hinzu (z.B. `request_id`, `component_name`). Dies erleichtert die Analyse von JSON-Logs.
    ```rust
    use tracing::info;
    // ...
    let user_id = 123;
    let item_id = "product_abc";
    info!(user.id = user_id, item.id = item_id; "Benutzer hat Artikel zum Warenkorb hinzugefügt");
    ```
*   **Fehler loggen:** Wenn Fehler behandelt werden (insbesondere bei `Result::Err`), logge den Fehler mit `error!` oder `warn!`, bevor er weiterpropagiert oder in einen anderen Fehlertyp umgewandelt wird.
    ```rust
    use tracing::error;
    // ...
    // match some_operation() {
    //     Ok(data) => { /* ... */ },
    //     Err(e) => {
    //         error!(error.message = %e, error.cause = ?e.source(); "Fehler bei some_operation");
    //         // return Err(MyAppError::from(e)); // Beispiel
    //     }
    // }
    ```
*   **Span-basiertes Tracing:** Für komplexere Abläufe oder Requests, nutze `tracing::Span`s, um Kontext über mehrere Log-Einträge hinweg zu verfolgen.
    ```rust
    use tracing::{span, Level, info, instrument};

    #[instrument(level = "debug", fields(request_id = %uuid::Uuid::new_v4()))]
    fn handle_request(data: &str) {
        info!("Anfrage wird verarbeitet.");
        // ... Logik ...
        info!("Anfrageverarbeitung abgeschlossen.");
    }
    ```

### 3.2. Konfiguration

Die Logging-Konfiguration (Level, Ausgabe, Format) wird über `novade-core::config::LoggingConfig` gesteuert und in der zentralen `config.toml` festgelegt. Siehe `docs/CONFIGURATION.md` für Details. Zur Laufzeit kann der `RUST_LOG` Umgebungsvariable die Loglevels weiter beeinflussen.

## 4. Fehlerbehandlung und Error Tracking <!-- ANCHOR [DevGuide ErrorTracking] -->

NovaDE verwendet ein zentralisiertes Error-Tracking-System, das in `novade-core::error_tracking` implementiert ist und Sentry für die Meldung von Fehlern und Panics nutzt.

### 4.1. Fehler definieren

Definiere spezifische Fehlertypen für deine Module oder Komponenten mit `thiserror::Error`.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyModuleError {
    #[error("Ein I/O Fehler ist aufgetreten: {0}")]
    Io(#[from] std::io::Error),
    #[error("Ungültiger Zustand: {details}")]
    InvalidState { details: String },
    // ... andere Fehlervarianten
}
```

### 4.2. Fehler an Sentry melden

Verwende die Funktion `novade_core::error_tracking::capture_error()`, um behandelte Fehler explizit an Sentry zu senden.

```rust
use novade_core::error_tracking::capture_error;
use serde_json::json;
// ...
// if let Err(e) = result_of_some_operation() {
//     let context = json!({ "operation_details": "Wichtige Operation fehlgeschlagen" });
//     capture_error(&e, Some(context));
//     // Gegebenenfalls den Fehler weiter behandeln oder zurückgeben
// }
```
Panics werden automatisch von Sentry erfasst, sofern Sentry initialisiert wurde.

### 4.3. Breadcrumbs hinzufügen

Um den Kontext vor einem Fehler oder einem wichtigen Ereignis nachzuvollziehen, füge Breadcrumbs hinzu:

```rust
use novade_core::error_tracking::add_breadcrumb;
use sentry::Level;
// ...
add_breadcrumb("Auth", "Benutzer-Login-Versuch gestartet.", Level::Info);
// ...
// match login_user(username, password) {
//    Ok(_) => add_breadcrumb("Auth", "Benutzer-Login erfolgreich.", Level::Info),
//    Err(_) => add_breadcrumb("Auth", "Benutzer-Login fehlgeschlagen.", Level::Warning),
// }
```

### 4.4. Konfiguration

Die Sentry-Integration (DSN, Environment, Release) wird über `novade-core::config::ErrorTrackingConfig` gesteuert. Siehe `docs/CONFIGURATION.md`.

## 5. Metriken zum Prometheus Exporter hinzufügen <!-- ANCHOR [DevGuide Metrics] -->

Das System (`novade-system`) enthält einen Prometheus Exporter, um Metriken zu veröffentlichen. So fügst du neue Metriken hinzu:

1.  **Definiere die Metrik statisch in `metrics_exporter.rs`:**
    Verwende `prometheus`-Makros wie `register_gauge!`, `register_counter!`, `register_histogram!`.
    ```rust
    // In novade_system::system_health_collectors::metrics_exporter.rs
    // use prometheus::{register_gauge, Opts, Gauge};
    // use once_cell::sync::Lazy;
    //
    // static MY_CUSTOM_METRIC: Lazy<Gauge> = Lazy::new(|| {
    //     register_gauge!(Opts::new("novade_my_custom_metric", "Beschreibung meiner Metrik.")
    //                     .namespace("novade_custom_module"))
    //                     .expect("Failed to register MY_CUSTOM_METRIC")
    // });
    ```

2.  **Aktualisiere `update_metrics_from_collectors()` in `metrics_exporter.rs`:**
    Füge Logik hinzu, um den Wert deiner neuen Metrik zu setzen. Dies kann das Abrufen von Daten von einem neuen oder existierenden Collector beinhalten.
    ```rust
    // In MetricsExporter::update_metrics_from_collectors()
    // async fn update_metrics_from_collectors(&self) {
    //     // ... andere Metriken ...
    //     let current_value = self.my_custom_data_source.get_value().await;
    //     MY_CUSTOM_METRIC.set(current_value as f64);
    // }
    ```
    Wenn deine Metrik von einem neuen Collector stammt, stelle sicher, dass dieser Collector in `MetricsExporter::new()` initialisiert und der `MetricsExporter`-Struktur hinzugefügt wird.

3.  **Stelle sicher, dass dein Collector die Daten bereitstellt:**
    Der `my_custom_data_source` im Beispiel oben muss die entsprechenden Daten liefern können.

Die Metriken sind dann über den `/metrics`-Endpunkt verfügbar, wenn der Exporter aktiviert ist.

## 6. Debug Interface verwenden (Zukünftig) <!-- ANCHOR [DevGuide DebugIF] -->

Eine Debug-Schnittstelle ist in `novade-system::debug_interface` geplant. Sie soll Laufzeitinspektion und Diagnose ermöglichen.

**Mögliche zukünftige Funktionen:**

*   Abruf von Systemzustands-Snapshots.
*   Generierung von State Dumps.
*   Auslösen von Profiling-Sitzungen.
*   Anstoßen von Speicherberichten.

Die Interaktion erfolgt voraussichtlich über einen Unix-Socket oder D-Bus. Die genauen Befehle und das Protokoll werden noch definiert. Die Konfiguration (Aktivierung, Adresse) erfolgt über `novade-core::config::DebugInterfaceConfig`.

<!-- TODO [NovaDE Developers <dev@novade.org>] Erweitern mit Details, sobald das Debug Interface implementiert ist. -->

---
*Dieses Dokument ist Teil der NovaDE-Gesamtspezifikation.*
<!-- ANCHOR_END -->
