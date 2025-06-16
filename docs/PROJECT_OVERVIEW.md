## Projektübersicht NovaDE

### 1. Einleitung
   - NovaDE ist ein Projekt zur Entwicklung einer modernen Desktop-Umgebung. Es zielt darauf ab, eine benutzerfreundliche, performante und anpassbare Erfahrung zu bieten.
   - Dieses Dokument dient als aktueller Projektstand und richtet sich primär an Entwickler, um einen umfassenden Überblick über die Architektur, die einzelnen Komponenten und den Entwicklungsfortschritt zu geben.

### 2. Gesamtarchitektur
   - Das Projekt folgt einer klaren Schichtenarchitektur, die eine saubere Trennung der Verantwortlichkeiten gewährleistet:
     - **Core:** Enthält die grundlegendsten Bausteine, Utilities, Konfigurationsmanagement, Logging, Fehlerbehandlung und Basis-Datentypen, die von allen anderen Schichten genutzt werden.
     - **Domain:** Implementiert die Geschäftslogik und Domain-spezifische Funktionalitäten gemäß den Prinzipien des Domain-Driven Design (DDD). Hier finden sich Dienste für KI-Assistenz, globale Einstellungen, Benachrichtigungen, Theming, Fenster-Management-Richtlinien und Workspace-Verwaltung.
     - **System:** Dient als Schnittstelle zum Betriebssystem und handhabt systemnahe Aufgaben wie den Wayland-Compositor, Input-Verarbeitung (libinput, udev), D-Bus-Kommunikation und Systemdienste. Diese Schicht ist auch dafür verantwortlich, Executables zu bauen, insbesondere den Compositor.
     - **UI:** Verantwortlich für die grafische Benutzeroberfläche. NovaDE verfolgt hier einen Dual-Toolkit-Ansatz, bei dem Iced für die Haupt-Desktop-Shell und GTK4/LibAdwaita für spezifische Anwendungen wie das System Health Dashboard und andere UI-Komponenten zum Einsatz kommen.
   - Die Interaktion zwischen den Schichten erfolgt typischerweise von oben nach unten, wobei höhere Schichten auf Dienste und Funktionalitäten tieferliegender Schichten zugreifen.
   - Das Projekt nutzt Cargo Workspaces zur Strukturierung der Codebasis in die genannten Haupt-Crates (`novade-core`, `novade-domain`, `novade-system`, `novade-ui`). Rust ist die primäre Programmiersprache. Das Workspace-Konzept ermöglicht eine modulare Entwicklung und Verwaltung der Abhängigkeiten.

### 3. Crate-Analyse

#### 3.1. `novade-core`
   - **Zweck:** Stellt die fundamentalen Bausteine und Kernfunktionalitäten für das gesamte NovaDE-Projekt bereit. Dazu gehören Konfigurationsmanagement, Logging, Fehlerbehandlung und grundlegende Datentypen.
   - **Wichtige Module und deren Inhalt:**
     - `src/config/`: Verantwortlich für das statische Laden der `CoreConfig` aus einer TOML-Datei. Dies stellt die globale Konfiguration für die Core-Schicht bereit. Es wurde eine Abweichung von der ursprünglichen Spezifikation festgestellt, die eine dynamischere Konfiguration vorsah.
     - `src/error.rs`: Definiert die zentralen Fehlertypen `CoreError`, `ConfigError`, `LoggingError` und `ColorParseError` unter Verwendung der `thiserror`-Crate für eine strukturierte Fehlerbehandlung.
     - `src/logging.rs`: Implementiert ein `tracing`-basiertes Logging-System, das Ausgaben in die Konsole, in Dateien sowie im JSON- oder Textformat unterstützt. Eine temporäre Lösung mit `std::mem::forget` für `WorkerGuard` wurde identifiziert, die möglicherweise überarbeitet werden muss.
     - `src/types/`: Enthält Definitionen für grundlegende Datentypen, die projektweit verwendet werden, wie z.B. für Geometrie (Punkte, Größen), Farben, Anwendungs-Identifikatoren (`AppIdentifier`) und Status-Enums.
     - `src/utils/`: Bietet Utility-Funktionen für Dateisystemoperationen und Pfadmanipulationen, insbesondere im Kontext der XDG Base Directory Specification.
   - **Verzeichnisstruktur:** Die Struktur ist logisch nach Funktionalität gegliedert, mit dedizierten Verzeichnissen für Konfiguration, Fehler, Logging, Typen und Utilities.

#### 3.2. `novade-domain`
   - **Zweck:** Beinhaltet die Geschäftslogik und Domain-spezifische Funktionalitäten des NovaDE-Projekts, orientiert an den Prinzipien des Domain-Driven Design (DDD).
   - **Wichtige Module und deren Inhalt:**
     - `src/ai/`, `src/ai_interaction_service/`: Module für die KI-Assistenzfunktionen, einschließlich Interaktionslogik, Consent-Management und Verarbeitung natürlicher Sprache (NLP).
     - `src/global_settings/`: Implementiert den `GlobalSettingsService` für die Verwaltung und Persistenz globaler Konfigurationen, die über die `CoreConfig` hinausgehen.
     - `src/notifications_rules/`, `src/user_centric_services/notifications_core/`: Verantwortlich für die Benachrichtigungs-Engine und den dazugehörigen Service, der Regeln für Benachrichtigungen verarbeitet und diese dem Benutzer präsentiert.
     - `src/theming/`: Enthält die Theming-Engine, Definitionen für Design-Tokens und Standard-Themes für das Erscheinungsbild der Desktop-Umgebung.
     - `src/window_management_policy/`: Definiert Richtlinien für das Fenster-Management, wie z.B. Tiling-Verhalten und Fokus-Regeln.
     - `src/workspaces/`: Implementiert das Workspace-Management, einschließlich CRUD-Operationen (Create, Read, Update, Delete) für Workspaces und die Zuweisung von Fenstern zu diesen.
     - `src/lib.rs`: Definiert `DomainServices` als zentralen Zugriffspunkt auf die verschiedenen Domain-Dienste und enthält die Funktion `initialize_domain_layer` für deren initiales Setup.
     - `src/error.rs`: Definiert `DomainError` als einen Wrapper-Fehlertyp, der spezifischere Fehler aus den einzelnen Domain-Modulen sowie `CoreError` kapseln kann.
   - **Verzeichnisstruktur:** Die Verzeichnisstruktur spiegelt die verschiedenen Subdomänen und Dienste wider, die in dieser Schicht implementiert sind.

#### 3.3. `novade-system`
   - **Zweck:** Dient als Schnittstelle zum Betriebssystem und kümmert sich um systemnahe Aufgaben. Dazu gehören der Wayland-Compositor, die Eingabeverarbeitung, D-Bus-Integration und die Sammlung von Systemmetriken. Diese Crate ist auch dafür vorgesehen, die primären Executables des Projekts zu bauen.
   - **Wichtige Module und deren Inhalt:**
     - `src/compositor/`: Enthält den umfangreichen und komplexen Wayland-Compositor. Dieser basiert auf einer älteren Version von Smithay (0.3.0), was eine zukünftige Migration auf eine neuere Version oder eine alternative Lösung nahelegt.
       - `backend/`: Implementierungen für verschiedene Backends, darunter DRM (Direct Rendering Manager) und Winit.
       - `core/`, `shell/`, `wayland_server/`: Kernkomponenten des Compositors und Implementierungen verschiedener Wayland-Protokolle und Shell-Erweiterungen.
       - `renderers/`: Beinhaltet GLES2- und Vulkan-Renderer. Der genaue Status und die Beziehung zum WGPU-Renderer sind noch unklar.
     - `src/nova_compositor_logic/`: Enthält weitere Logik für den Compositor. Das genaue Verhältnis und die Abgrenzung zu `src/compositor/` bedürfen weiterer Klärung. Es gibt Hinweise auf ein geplantes Refactoring, bei dem diese Logik in `src/compositor/` integriert werden soll.
     - `src/renderer/`: Hier findet sich `wgpu_renderer.rs`, was auf einen moderneren Rendering-Pfad mittels WGPU hindeutet. Dieser scheint der primär vorgesehene Renderer zu sein.
     - `src/input/`: Verantwortlich für die Verarbeitung von Eingaben über `libinput` und `udev`. Trotz auskommentierter Abhängigkeiten in `Cargo.toml` sind `libinput_handler.rs` und `udev_handler.rs` im Code vorhanden und aktiv.
     - `src/dbus_integration/`: Implementiert die Kommunikation mit anderen Anwendungen und Systemdiensten über D-Bus.
     - `src/system_health_collectors/`: Sammelt Metriken über den Systemzustand (CPU-Auslastung, Speicherverbrauch etc.).
     - `src/lib.rs`: Exponiert nur einen Teil der Funktionalität der Crate. Der Compositor wird voraussichtlich als separates Executable gebaut.
     - `src/error.rs`: Definiert den `SystemError`-Typ für Fehler, die in dieser Schicht auftreten.
   - **Verzeichnisstruktur:** Die Struktur ist komplex, was die Natur der systemnahen Aufgaben widerspiegelt, insbesondere im Bereich des Compositors.

#### 3.4. `novade-ui`
   - **Zweck:** Verantwortlich für die Implementierung der grafischen Benutzeroberflächen von NovaDE.
   - **Wichtige Module und deren Inhalt:**
     - **Dual-Toolkit-Ansatz:** Eine zentrale Beobachtung ist die Verwendung von zwei verschiedenen UI-Toolkits:
       - **Iced (Haupt-Shell):** Die `src/lib.rs` definiert eine Iced-basierte Applikation (`NovaDE`), die als Haupt-Desktop-Shell fungiert. Module wie `desktop_ui` und `panel_ui` sind Teil dieser Iced-Anwendung.
       - **GTK4/LibAdwaita (System Health Dashboard & Komponenten):** Die `src/main.rs` baut eine separate GTK4-Anwendung (`org.novade.SystemHealthDashboard`). Zahlreiche GTK-spezifische Dateien (`.ui`-Dateien, `gresources.xml`, `style.css`, `theming_gtk.rs`) sowie das `src/shell/`-Verzeichnis (welches GTK-basierte Panel-Widgets enthält) deuten darauf hin, dass GTK4 für wichtige UI-Teile, insbesondere das System Health Dashboard und eventuell weitere Systemkomponenten, verwendet wird.
     - `src/system_health_dashboard/`: Enthält die UI-Implementierung für das GTK-basierte System Health Dashboard, das Systemmetriken anzeigt.
     - `src/shell/`: Beinhaltet GTK-Komponenten, die vermutlich für die Desktop-Shell verwendet werden, wie z.B. Panel-Widgets. Dies steht im Kontrast zur Iced-basierten Haupt-Shell und deutet auf eine hybride UI-Strategie hin.
     - `src/error.rs`: Definiert den `UiError`-Typ für Fehler im UI-Bereich.
   - **Verzeichnisstruktur:** Die Struktur ist durch den Dual-Toolkit-Ansatz geprägt, mit separaten Bereichen für Iced- und GTK-Komponenten.

### 4. Root-Verzeichnis und Dokumentation
   - **`docs/`**: Dieses Verzeichnis ist eine Goldgrube an Informationen. Es enthält umfangreiche und detaillierte Design- und Spezifikationsdokumente, meist in deutscher Sprache. Diese Dokumente dienen als primäre Quelle für das Verständnis der geplanten Architektur und der funktionalen Anforderungen.
   - **`Cargo.toml` (Root)**: Definiert den Cargo-Workspace, der die vier Haupt-Crates (`novade-core`, `novade-domain`, `novade-system`, `novade-ui`) zusammenfasst. Ein wichtiger Kommentar weist darauf hin, dass der Crate `nova_compositor` (vermutlich `src/nova_compositor_logic` in `novade-system`) in den Haupt-Compositor-Crate (`src/compositor/` in `novade-system`) integriert werden soll.
   - **`.github/workflows/rust.yml`**: Enthält ein Standard-CI-Setup für Rust-Projekte, das Build- und Testprozesse automatisiert.
   - **`assets/icons/`**: Beinhaltet Projektsymbole und andere grafische Assets.
   - **`docs_old/`**: Enthält veraltete Dokumente, die möglicherweise historischen Kontext bieten, aber nicht mehr den aktuellen Planungsstand widerspiegeln.
   - **`EXISTING_IMPLEMENTATIONS.md`**: Ein sehr nützliches Dokument, das eine Analyse des aktuellen Implementierungsstands im Vergleich zu den Spezifikationen bietet. Es hilft, den Fortschritt und die bereits umgesetzten Funktionalitäten zu verstehen.
   - **`MISSING_IMPLEMENTATIONS.md`**: Dient als Aufgabenliste für fehlende oder unvollständige Implementierungen. Dieses Dokument ist entscheidend für die Planung zukünftiger Entwicklungsarbeiten.
   - **`NovaDE Compositor Modul-Implementierungs-Prompts.md`**: Enthält detaillierte Prompts, die anscheinend für KI-Entwickler oder als detaillierte Implementierungsrichtlinien für Compositor-Module gedacht sind. Dieses Dokument enthüllt tiefgreifende Design-Absichten und technische Details.
   - **`README.md` (Root)**: Ein `README.md` im Root-Verzeichnis fehlt, was die Einstiegshürde für neue Entwickler erhöhen könnte.

### 5. Wichtige Erkenntnisse und Beobachtungen
   - **Starke Schichtung:** Die Codebasis ist klar in die vier Hauptschichten Core, Domain, System und UI unterteilt, was eine gute Modularität und Wartbarkeit fördert.
   - **Dokumentation als Grundlage:** Die umfangreichen Spezifikationen und Designdokumente im `docs/`-Verzeichnis sind der zentrale Leitfaden für die Entwicklung.
   - **Compositor-Technologie:** Der Compositor in `novade-system` basiert auf einer älteren Smithay-Version (0.3.0). Es gibt klare Anzeichen, dass WGPU als primärer Renderer angestrebt wird, was eine Modernisierung des Rendering-Pfads bedeutet.
   - **Dual-Toolkit-Strategie in der UI:** `novade-ui` verfolgt einen interessanten Ansatz mit Iced für die Haupt-Desktop-Shell und GTK4/LibAdwaita für spezifische Anwendungen wie das System Health Dashboard und potenziell weitere UI-Komponenten. Die genaue Abgrenzung und Integration dieser beiden Toolkits ist ein wichtiger Aspekt.
   - **Geplante Refactorings:** Es gibt Hinweise auf geplante Refactorings, wie z.B. die Integration von `nova_compositor_logic` in den Haupt-Compositor.
   - **Meta-Analyse-Dokumente:** `EXISTING_IMPLEMENTATIONS.md` und `MISSING_IMPLEMENTATIONS.md` sind extrem wertvoll, um einen schnellen Überblick über den Projektfortschritt und die noch offenen Aufgaben zu erhalten.

### 6. Fazit
   - Das NovaDE-Projekt befindet sich in einem fortgeschrittenen Planungs- und frühen Implementierungsstadium. Die Architektur ist gut durchdacht und die Trennung der Verantwortlichkeiten durch die Schichten ist klar ersichtlich.
   - Die Dokumentation in `docs/`, zusammen mit den Meta-Analyse-Dokumenten wie `EXISTING_IMPLEMENTATIONS.md` und `MISSING_IMPLEMENTATIONS.md`, ist von unschätzbarem Wert für das Verständnis des Projekts, seines aktuellen Stands und der zukünftigen Entwicklungsrichtung. Die Klärung der Compositor-Strategie (Smithay-Version, WGPU-Integration) und die Details der Dual-Toolkit-Nutzung in der UI-Schicht sind zentrale technische Aspekte, die weiter verfolgt werden müssen. Die vorhandenen Dokumente bieten jedoch eine solide Grundlage für alle Entwickler, die sich in das Projekt einarbeiten möchten.
