# Spezifikation: System Health Monitoring und Diagnose-Schnittstellen (v1.0.0)

<!-- ANCHOR [NovaDE Developers <dev@novade.org>] Version 1.0.0 -->

**Status:** Entwurf

**Datum:** YYYY-MM-DD (Wird beim Merge aktualisiert)

**Verantwortlich:** NovaDE System Team

**Kurzbeschreibung:** Dieses Dokument beschreibt die Backend-Systeme für die Überwachung des Systemzustands, die Fehlerverfolgung, den Metrik-Export und die Debug-Schnittstellen von NovaDE. Es ergänzt `SPEC-FEATURE-SYSTEM-HEALTH-DASHBOARD-v0.1.0.md`, das sich auf die UI-Darstellung konzentriert.

## 1. Einleitung und Ziele

Eine robuste Überwachung und Diagnosefähigkeit ist entscheidend für die Stabilität, Leistung und Wartbarkeit von NovaDE. Dieses Dokument spezifiziert die Kernkomponenten, die diese Fähigkeiten auf Systemebene bereitstellen.

**Ziele:**

*   Implementierung umfassender Performance-Datensammler für CPU, Speicher, Frametimes und GPU.
*   Export der gesammelten Metriken im Prometheus-Format für externe Überwachungssysteme.
*   Integration eines Error-Tracking-Systems (Sentry) zur automatischen Erfassung und Meldung von Fehlern und Panics.
*   Bereitstellung einer Debug-Schnittstelle für Entwickler zur Laufzeitinspektion und -diagnose.
*   Sicherstellung, dass diese Systeme über `novade-core::config` konfigurierbar sind.

## 2. Systemkomponenten

### 2.1. Performance-Kollektoren (`novade-system`)

Die folgenden Kollektoren sind in `novade-system::system_health_collectors` implementiert:

*   **`CpuCollector`**:
    *   Sammelt die gesamte CPU-Auslastung sowie die Auslastung pro Kern.
    *   Basiert auf `psutil` (oder äquivalenten Systemaufrufen).
*   **`MemoryCollector`**:
    *   Sammelt die systemweite Speichernutzung (Gesamt, Genutzt, Frei, Verfügbar, Swap) mittels `psutil`.
    *   Ermöglicht die manuelle Erfassung der Speichernutzung einzelner Subsysteme des Compositors (z.B. Renderer, Szenengraph) über `record_subsystem_memory()`.
*   **`FrameTimeCollector`**:
    *   Erfasst die Dauer einzelner Frames (`record_frame_time()`).
    *   Berechnet Statistiken über ein rollierendes Fenster von Samples (Min, Max, Durchschnitt, 95. Perzentil).
    *   Speichert Samples in einer `VecDeque`.
*   **`GpuCollector`**:
    *   Sammelt GPU-Metriken wie Auslastung, Speichernutzung und Temperatur.
    *   **NVIDIA:** Nutzt `nvml-wrapper` (optional, über Feature-Flag `nvml`).
    *   **AMD:** <!-- ANCHOR [AMD GPU Monitoring] --> TODO: Implementierung ausstehend (potenziell via sysfs oder `amdgpu-sysfs`).
    *   **Intel:** <!-- ANCHOR [Intel GPU Monitoring] --> TODO: Implementierung ausstehend (potenziell via sysfs oder spezifische Intel-Bibliotheken).

### 2.2. Metrics Exporter (Prometheus) (`novade-system`)

*   **Modul:** `novade_system::system_health_collectors::metrics_exporter`
*   **Funktionalität:** Startet einen HTTP-Server (via `warp`), der auf dem Endpunkt `/metrics` Metriken im Prometheus-Textformat bereitstellt.
*   **Datensammlung:** Der Exporter registriert die oben genannten Performance-Kollektoren und sammelt deren Daten, um sie in Prometheus-Metriktypen (Gauge, Histogram) zu überführen.
*   **Konfiguration:**
    *   `metrics_exporter_enabled: bool` (aktiviert/deaktiviert den Exporter)
    *   `metrics_exporter_address: String` (z.B. "0.0.0.0:9090")
    *   Details siehe `docs/CONFIGURATION.md`.
*   **TODOs:**
    *   `//TODO [Custom Metrics]` Platzhalter für die Definition und Registrierung anwendungsspezifischer Metriken.
    *   `//TODO [Metrics Aggregation]` Hinweis, dass Aggregation primär serverseitig durch Prometheus erfolgt.
    *   `//TODO [Metrics Retention]` Hinweis auf serverseitige Konfiguration in Prometheus.
    *   `//TODO [Metrics Alerting]` Hinweis auf serverseitige Konfiguration in Prometheus.

### 2.3. Error Tracking (Sentry) (`novade-core`)

*   **Modul:** `novade_core::error_tracking`
*   **Funktionalität:** Initialisiert das Sentry SDK zur Erfassung von Fehlern und Panics. Integriert sich mit dem `tracing`-System über eine `SentryLayer`.
*   **API:**
    *   `init_error_tracking(config: &ErrorTrackingConfig)`: Initialisiert Sentry.
    *   `capture_error(error: &dyn std::error::Error, context: Option<serde_json::Value>)`: Sendet einen Fehler an Sentry.
    *   `add_breadcrumb(category: &str, message: &str, level: sentry::Level)`: Fügt einen Breadcrumb hinzu.
*   **Konfiguration:**
    *   `sentry_dsn: Option<String>` (deaktiviert Sentry bei `None`)
    *   `sentry_environment: Option<String>`
    *   `sentry_release: Option<String>`
    *   Details siehe `docs/CONFIGURATION.md`.
*   **TODOs:**
    *   `//TODO [Error Deduplication]` Prüfung der Standard-Deduplizierung von Sentry.
    *   `//TODO [Configurable Thresholds]` Hinweis auf serverseitige Konfiguration bei Sentry.
    *   `//TODO [Error Recovery Tracking]` Überlegungen zur spezifischen Verfolgung von Fehlerbehebungen.

### 2.4. Debug Interface (`novade-system`)

*   **Modul:** `novade_system::debug_interface`
*   **Funktionalität:** Stellt eine (zukünftige) Schnittstelle für Entwickler zur Laufzeitinspektion und -diagnose bereit. Die aktuelle Implementierung enthält Platzhalterfunktionen.
*   **Placeholder-Funktionen:**
    *   `get_compositor_state_snapshot()`: Soll einen Schnappschuss des Compositor-Zustands liefern.
        *   `//TODO [Compositor State Access]` Details zur Implementierung des Zugriff auf den Compositor-Status.
    *   `generate_state_dump()`: Serialisiert Systemzustände in JSON.
        *   `//TODO [Sensitive Information]` Hinweis zur Notwendigkeit der Schwärzung sensibler Daten.
    *   `process_command()`: Platzhalter für die Verarbeitung von Debug-Befehlen.
    *   `start_profiling() / stop_profiling()`: Platzhalter für die Steuerung von Performance-Profilern.
        *   `//TODO [External Profiler Control]` Details zur Integration externer Profiler.
    *   `trigger_memory_report()`: Platzhalter für die Auslösung von Speicheranalyse-Tools.
        *   `//TODO [Memory Leak Tooling]` Details zur Integration von Speicheranalyse-Werkzeugen.
*   **Konfiguration:**
    *   `debug_interface_enabled: bool`
    *   `debug_interface_address: Option<String>` (z.B. für einen Unix-Socket)
    *   Details siehe `docs/CONFIGURATION.md`.
*   **TODOs:**
    *   `//TODO [Debug Command Transport]` Spezifikation des Transportmechanismus für Befehle (z.B. Unix-Socket, D-Bus).

## 3. Interaktion und Datenfluss

1.  **Datensammlung:** Die Performance-Kollektoren in `novade-system` sammeln kontinuierlich oder bei Bedarf Daten. `FrameTimeCollector` wird extern getriggert, andere pollen oder reagieren auf Anfragen.
2.  **Metrics Export:** Der `MetricsExporter` (falls aktiviert) ruft regelmäßig `update_metrics_from_collectors()` auf, um die Prometheus-Gauges/-Histograms mit den neuesten Daten der Kollektoren zu aktualisieren. Ein Prometheus-Server scrapt dann den `/metrics`-Endpunkt.
3.  **Error Tracking:**
    *   `init_error_tracking()` wird früh im Startprozess von `novade-core` (oder der Hauptanwendung) aufgerufen.
    *   Panics werden automatisch von Sentry erfasst.
    *   Fehler werden explizit mit `capture_error()` gemeldet.
    *   `tracing`-Events werden (abhängig von der Konfiguration der `SentryLayer` im Logging-Setup) als Breadcrumbs an Sentry gesendet.
4.  **Debug Interface:** (Zukünftig) Ein externes Debug-Tool oder ein CLI-Client verbindet sich mit dem Debug Interface (z.B. über einen Unix-Socket). Befehle werden an `DebugInterface::process_command()` gesendet, das dann die entsprechenden Aktionen ausführt (z.B. State Dumps, Profiler-Steuerung).

## 4. Konfiguration

Alle oben genannten Systeme sind über die zentrale Konfigurationsdatei (`config.toml`), die von `novade-core::config` geladen wird, konfigurierbar. Die spezifischen Konfigurationsabschnitte und -parameter sind in `docs/CONFIGURATION.md` detailliert beschrieben.

## 5. Performance-Ziele (Beispiele)

*   **Metrik-Sammlung:** Der Overhead durch die Sammlung aller aktivierten Systemmetriken sollte im Durchschnitt unter 2-5% CPU-Last auf einem Zielsystem bleiben.
*   **Error Tracking:** Das Senden von Fehlerberichten sollte die Anwendungsperformance nicht merklich beeinträchtigen, insbesondere bei hoher Fehlerfrequenz (asynchrone Verarbeitung ist hier wichtig).
*   **Metrics Exporter:** Der `/metrics`-Endpunkt sollte typischerweise in unter 50-100ms antworten.

Diese Ziele dienen als Richtwerte und müssen im Kontext der Zielhardware und des Systemzustands bewertet werden.

---
*Dieses Dokument ist Teil der NovaDE-Gesamtspezifikation.*
<!-- ANCHOR_END -->
