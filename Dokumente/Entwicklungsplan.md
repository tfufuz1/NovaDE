
### GESAMTARCHITEKTUR UND SCHICHTENDESIGN

---

#### 1. Gesamtarchitektur und Architekturelle Patterns

NovaDE MUSS eine modulare, wartbare und erweiterbare Architektur implementieren, die auf etablierten und bewährten Architekturmustern basiert. Die primäre architektonische Grundlage ist die **Layered Architecture (Schichtenarchitektur)**. Diese Wahl fördert die Trennung der Belange (Separation of Concerns), reduziert die Kopplung zwischen Systemteilen und verbessert die Testbarkeit.

Zusätzlich zur Layered Architecture WERDEN folgende architektonische Patterns und Prinzipien systemweit bzw. schichtspezifisch angewendet:

1.  **Component-Based Architecture:**
    * **Anwendung:** Innerhalb jeder Schicht, insbesondere in der `novade-system` und `novade-ui` Schicht.
    * **Begründung:** Fördert die Wiederverwendbarkeit von Code, erleichtert die unabhängige Entwicklung und das Testen von Modulen. Module wie der Display-Manager, Netzwerk-Manager oder das Panel in der UI-Schicht SIND als eigenständige Komponenten zu konzipieren.
    * **Spezifikation:** Jede Komponente MUSS eine klar definierte Schnittstelle (API) aufweisen und ihre Abhängigkeiten explizit deklarieren.

2.  **Event-Driven Architecture (EDA):**
    * **Anwendung:** Primär in der `novade-system` Schicht für die Interaktion mit dem Betriebssystem (z.B. Wayland-Events, Hardware-Events, D-Bus-Signale) und in der `novade-ui` Schicht für Benutzerinteraktionen.
    * **Begründung:** Ermöglicht eine lose Kopplung und reaktive Systemkomponenten, die auf asynchrone Ereignisse reagieren. Dies ist essenziell für eine responsive Desktop-Umgebung.
    * **Spezifikation:** Ereignisse MÜSSEN klar definierte Datenstrukturen haben. Event-Handler MÜSSEN so implementiert werden, dass sie die Ereignisschleife nicht blockieren (z.B. durch Auslagerung langlaufender Aufgaben in separate Threads oder asynchrone Tasks). Smithay für den Wayland-Compositor und GTK4 für die UI setzen inhärent auf EDA.

3.  **Service-Oriented Architecture (SOA) / Microservices-Prinzipien (intern):**
    * **Anwendung:** Für klar abgrenzbare Systemdienste innerhalb der `novade-system` Schicht (z.B. `network_manager_service`, `power_management_service`, `display_manager_service`). Obwohl NovaDE kein verteiltes System im klassischen Microservice-Sinne ist, WERDEN die Prinzipien der losen Kopplung, unabhängigen Entwickelbarkeit und klaren Schnittstellendefinition für diese Dienste angewendet.
    * **Begründung:** Verbessert die Modularität und ermöglicht es, einzelne Dienste bei Bedarf neu zu starten oder zu aktualisieren, ohne das gesamte System zu beeinträchtigen (langfristiges Ziel).
    * **Spezifikation:** Jeder Dienst MUSS über eine wohldefinierte Schnittstelle (z.B. D-Bus oder interne Rust-API) zugänglich sein und seine Verantwortlichkeiten strikt einhalten.

4.  **Model-View-Controller (MVC) / Model-View-ViewModel (MVVM) - Varianten:**
    * **Anwendung:** Primär für die `novade-ui` Schicht (GTK4). Die Wahl zwischen MVC, MVP oder MVVM (oder einer GTK-spezifischen Variante wie die Verwendung von `gtk::ApplicationWindow` mit internem State-Management und UI-Definitionen via XML/Blueprints) MUSS pro UI-Komponente getroffen werden, um die Lesbarkeit und Wartbarkeit des UI-Codes zu maximieren.
    * **Begründung:** Trennt die Anwendungslogik von der Benutzeroberfläche und der Datenrepräsentation, was die UI-Entwicklung und das Testen erleichtert.
    * **Spezifikation:** Datenbindung (wo von GTK4 unterstützt), Nachrichtenweiterleitung und klare Verantwortlichkeiten für Controller/ViewModel MÜSSEN etabliert werden.

5.  **Repository Pattern & Service Layer Pattern:**
    * **Anwendung:** In der `novade-domain` und `novade-system` Schicht zur Abstraktion des Datenzugriffs (Repository) und zur Kapselung von Geschäftslogik bzw. Systemoperationen (Service Layer).
    * **Begründung:** Entkoppelt die Geschäftslogik von den Details der Datenpersistenz und den systemspezifischen Implementierungen.
    * **Spezifikation:** Repositories MÜSSEN Schnittstellen für CRUD-Operationen (Create, Read, Update, Delete) auf Domänenobjekten bereitstellen. Service-Layer-Komponenten MÜSSEN die Interaktion zwischen UI-Anfragen, Domänenlogik und Datenzugriff koordinieren.

6.  **Dependency Injection (DI):**
    * **Anwendung:** Systemweit.
    * **Begründung:** Ist KRITISCH für die Erstellung von lose gekoppelten Komponenten und die Vereinfachung von Unit-Tests durch Ermöglichung von Mocking.
    * **Spezifikation:** Abhängigkeiten MÜSSEN über Konstruktoren oder dedizierte Initialisierungsmethoden injiziert werden. Globale statische Zustände SIND ZU VERMEIDEN. Frameworks oder Bibliotheken für DI in Rust (z.B. `shaku` oder manuelle Implementierung) KÖNNEN evaluiert werden.

7.  **Command Query Responsibility Segregation (CQRS) - Lite:**
    * **Anwendung:** Potenziell für komplexe Zustandsverwaltungskomponenten, z.B. im Konfigurationsmanagement oder in Teilen des Window-Managers, wo Lese- und Schreibpfade stark voneinander abweichen und unterschiedliche Optimierungsanforderungen haben.
    * **Begründung:** Kann die Leistung und Skalierbarkeit verbessern, indem Lese- und Schreibvorgänge getrennt behandelt werden.
    * **Spezifikation:** Eine "Lite"-Version ohne getrennte Datenspeicher ist zunächst ausreichend. Klare Trennung von Befehlen (Commands), die den Zustand ändern, und Abfragen (Queries), die Daten lesen.

8.  **Asynchronous Programming (async/await in Rust):**
    * **Anwendung:** Systemweit, insbesondere für I/O-gebundene Operationen, IPC (D-Bus), Netzwerkkommunikation und gleichzeitige Aufgaben in der UI und den Systemdiensten. `tokio` IST das primäre asynchrone Runtime Environment.
    * **Begründung:** Gewährleistet eine responsive Anwendung und effiziente Ressourcennutzung ohne die Komplexität manuellen Thread-Managements für jede Operation.
    * **Spezifikation:** Alle potenziell blockierenden Operationen MÜSSEN asynchron implementiert werden. Die Korrektheit der Synchronisation (z.B. `Mutex`, `RwLock` aus `tokio::sync`) bei Zugriff auf geteilte Daten IST SICHERZUSTELLEN.

Diese Patterns bilden das Rückgrat der NovaDE-Architektur. Ihre korrekte und konsistente Anwendung IST entscheidend für den Erfolg des Projekts.

---

#### 2. Spezifikation der Architekturschichten und deren Verantwortlichkeiten

NovaDE IST in vier klar definierte logische Schichten unterteilt. Die Kommunikation zwischen den Schichten erfolgt ausschließlich in eine Richtung (UI -> System -> Domäne -> Kern), es sei denn, es handelt sich um Rückgabewerte oder explizit definierte Callback-/Event-Mechanismen. Direkte Aufrufe von einer unteren Schicht in eine höhere Schicht SIND STRIKT VERBOTEN.

##### 2.1. Kernschicht (`novade-core`)

* **Crate-Name:** `novade_core`
* **Verantwortlichkeiten:**
    1.  **Grundlegende Datentypen:** Definition von fundamentalen, projektweit genutzten Datentypen (z.B. spezifische ID-Typen, Fehler-Enums-Basistypen, Koordinatensysteme), die keine Abhängigkeiten zu anderen Schichten haben.
    2.  **Utility-Funktionen:** Bereitstellung von allgemeinen Hilfsfunktionen und Makros (z.B. für String-Manipulation, mathematische Berechnungen, Collection-Helfer), die keine spezifische Domänen- oder Systemlogik enthalten.
    3.  **Logging-Infrastruktur:** Konfiguration und Bereitstellung der zentralen Logging-Fassade (z.B. basierend auf `tracing` oder `log` Crate). Module in anderen Schichten MÜSSEN diese Fassade für alle Protokollierungsaktivitäten nutzen.
    4.  **Basiskonfiguration:** Definition der Mechanismen zum Laden und Parsen von Basiskonfigurationsaspekten, die für den Start des Systems notwendig sind (z.B. Pfaddefinitionen, Logging-Level aus Umgebungsvariablen).
    5.  **Allgemeine Fehlerdefinitionen:** Bereitstellung eines Basis-Error-Typs oder -Traits (z.B. `NovaCoreError`), von dem spezifischere Fehler in höheren Schichten abgeleitet werden können. Dies MUSS Mechanismen zur Fehlerverkettung und Kontextualisierung unterstützen.
    6.  **Konstanten und globale Attribute:** Definition von systemweiten Konstanten (z.B. Standard-Socket-Pfade, Anwendungsname).
* **Abhängigkeiten:** Minimale externe Abhängigkeiten (primär Rust Standard Library und Utility-Crates wie `serde` für Serialisierung, `log` oder `tracing`). KEINE Abhängigkeiten zu `novade-domain`, `novade-system`, oder `novade-ui`.
* **Schnittstellen:** Stellt Funktionen, Typen, Traits und Makros bereit, die von den höheren Schichten importiert werden können.

##### 2.2. Domänenschicht (`novade-domain`)

* **Crate-Name:** `novade_domain`
* **Verantwortlichkeiten:**
    1.  **Domänenmodelle (Entitäten & Werteobjekte):** Definition der Kernkonzepte und Datenstrukturen der Desktop-Umgebung (z.B. `Window`, `Workspace`, `Application`, `UserSettings`, `Notification`). Diese Modelle repräsentieren den reinen Zustand und die Logik, unabhängig von UI oder Systemimplementierung.
    2.  **Domänenlogik (Policies & Business Rules):** Implementierung der Regeln und Logik, die das Verhalten der Domänenmodelle steuern (z.B. Validierungsregeln für Einstellungen, Logik zur Fensterplatzierung, Policy für Benachrichtigungsanzeige).
    3.  **Domänenservices:** Kapselung von komplexerer Domänenlogik, die mehrere Entitäten koordiniert, aber keine direkten externen Abhängigkeiten (wie I/O oder Systemaufrufe) hat.
    4.  **Schnittstellen für Repositories (Abstraktionen):** Definition von Traits für Datenzugriffsoperationen (z.B. `SettingsRepository`, `ApplicationRepository`). Die konkreten Implementierungen dieser Repositories residieren in der Systemschicht oder Kernschicht (für einfache Fälle).
    5.  **Definition von Domänenereignissen:** Spezifikation von Ereignissen, die signifikante Zustandsänderungen innerhalb der Domäne darstellen (z.B. `WorkspaceChanged`, `SettingModified`).
* **Abhängigkeiten:** Abhängig von `novade_core`. KEINE Abhängigkeiten zu `novade-system` oder `novade-ui`. Darf externe Crates für Logikunterstützung (z.B. `uuid`, `chrono`) verwenden.
* **Schnittstellen:** Stellt Domänenmodelle, Domänenservices und Repository-Schnittstellen für die `novade-system` Schicht bereit.

##### 2.3. Systemschicht (`novade-system`)

* **Crate-Name:** `novade_system`
* **Verantwortlichkeiten:**
    1.  **Interaktion mit dem Betriebssystem und der Hardware:** Kapselung aller direkten Interaktionen mit dem Linux-Kernel, Systembibliotheken und Hardware-Abstraktionen. Dies umfasst:
        * **Display Management:** Kommunikation mit dem Wayland-Compositor (unter Verwendung von Smithay oder einer ähnlichen Bibliothek), Verwaltung von Ausgaben, Auflösungen, DPMS. (Siehe `Rendering Vulkan.md`, `Compositor Smithay Wayland.md` für Details).
        * **Input Management:** Verarbeitung von Eingabeereignissen (Maus, Tastatur, Touch) vom Wayland-Compositor.
        * **Network Management:** Überwachung und Verwaltung von Netzwerkverbindungen (z.B. via `NetworkManager` D-Bus Schnittstelle).
        * **Power Management:** Interaktion mit `logind` oder ähnlichen Systemen für Energieverwaltung (Suspend, Hibernate, Bildschirmhelligkeit).
        * **Storage Management:** Überwachung von Datenträgern, Mount-Points (z.B. via UDisks).
        * **Prozessmanagement:** Starten und Verwalten von Anwendungsprozessen.
    2.  **Implementierung von Repository-Schnittstellen:** Bereitstellung konkreter Implementierungen der in `novade-domain` definierten Repository-Traits (z.B. Speichern von Einstellungen in Konfigurationsdateien).
    3.  **Systemdienste:** Implementierung von langlebigen Diensten, die Kernfunktionalitäten der Desktop-Umgebung bereitstellen (z.B. `NotificationService`, `ClipboardManagerService`, `WindowManagerService` - als Teil des Compositors). Diese Dienste können über IPC (z.B. D-Bus) oder interne APIs angesprochen werden.
    4.  **Hardware-Abstraktion:** Bereitstellung einer abstrahierten Sicht auf Hardware-Komponenten für die oberen Schichten.
    5.  **Plattformspezifische Anpassungen:** Kapselung von Code, der spezifisch für bestimmte Linux-Distributionen oder Systemkonfigurationen sein könnte (obwohl dies minimiert werden MUSS).
    6.  **KI-Backend-Integration:** Hosting und Management der KI-Modelle und des Model-Context-Protocol (MCP) Backends, wie in `Model-Context-Protocol.md` beschrieben, falls dieses systemnahe Ressourcen benötigt oder als Systemdienst läuft.
* **Abhängigkeiten:** Abhängig von `novade_core` und `novade_domain`. Nutzt diverse externe Crates für Systeminteraktion (z.B. `dbus`, `smithay`, `tokio`, `input`, `libloading` etc.). KEINE Abhängigkeiten zu `novade-ui`.
* **Schnittstellen:** Stellt APIs und Dienste für die `novade-ui` Schicht bereit, um Systemzustände abzufragen, Aktionen auszulösen und auf Systemereignisse zu reagieren. Diese Schnittstellen MÜSSEN so gestaltet sein, dass sie die Komplexität der Systeminteraktion verbergen.

##### 2.4. UI-Schicht (`novade-ui`)

* **Crate-Name:** `novade_ui`
* **Verantwortlichkeiten:**
    1.  **Benutzeroberfläche (GUI):** Implementierung aller visuellen Komponenten der Desktop-Umgebung, einschließlich Desktop-Shell (Panel, Anwendungsstarter, System-Tray, Desktop-Hintergrund), Anwendungsfenster-Dekorationen (Client-Side Decorations falls vom Compositor nicht vollständig übernommen), Dialoge, Einstellungs-Interfaces und Widgets.
    2.  **Benutzerinteraktion:** Verarbeitung von Benutzereingaben (Mausklicks, Tastatureingaben, Touch-Gesten), die von der Systemschicht (Wayland-Events) weitergeleitet werden, und deren Übersetzung in Aktionen oder Zustandsänderungen.
    3.  **Visualisierung des Systemzustands:** Darstellung von Informationen und Zuständen, die von der System- und Domänenschicht bereitgestellt werden (z.B. Netzwerkstatus, Lautstärke, laufende Anwendungen, Benachrichtigungen).
    4.  **Kommunikation mit der Systemschicht:** Aufruf von Diensten und Funktionen der `novade-system` Schicht, um Aktionen auszuführen oder Daten abzurufen.
    5.  **Theming und Anpassung:** Implementierung von Mechanismen zur visuellen Anpassung der UI (Themes, Icons, Schriftarten).
    6.  **Accessibility:** Sicherstellung der Barrierefreiheit der Benutzeroberfläche gemäß etablierten Standards.
* **Primäre Technologie:** GTK4 und die `gtk4-rs` Bindings. UI-Definitionen KÖNNEN deklarativ mittels UI-Blueprints erfolgen.
* **Abhängigkeiten:** Abhängig von `novade_core`, `novade_domain` (für die Darstellung von Domänenobjekten) und `novade_system` (für die Interaktion mit dem System). Nutzt `gtk4-rs` und assoziierte Crates (z.B. `gdk`, `glib`, `cairo`).
* **Schnittstellen:** Stellt die grafische Benutzeroberfläche für den Endbenutzer dar. Empfängt Ereignisse von der Systemschicht und delegiert Aktionen an diese.

Die strikte Einhaltung dieser Schichtengrenzen und Verantwortlichkeiten IST FUNDAMENTAL für die Integrität und Wartbarkeit der NovaDE-Architektur.

---
#### 3. Detaillierte Schnittstellen zwischen den Architekturschichten

Die Interaktion zwischen den Schichten MUSS streng kontrolliert und explizit definiert werden. Jede Schicht exponiert eine wohldefinierte API für die nächsthöhere Schicht. Direkte Kommunikation, die Schichten überspringt, IST VERBOTEN.

##### 3.1. Schnittstelle: `novade-core` -> `novade-domain`, `novade-system`, `novade-ui`

- **Exponierte Elemente durch `novade-core`:**
    - **Typen:** Fundamentale Datentypen (z.B. `NovaUuid`, `NovaErrorBase`, `LogLevel`, `Point2D`, `Dimensions2D`). Diese Typen SIND `pub` und können direkt verwendet werden.
    - **Traits:** Allgemeine Traits (z.B. `Configurable`, `Serializable`, `Validatable`).
    - **Funktionen/Makros:** Utility-Funktionen (z.B. `novacore::utils::generate_uuid()`, `novacore::logging::info!(...)`).
    - **Konstanten:** Globale Konstanten (z.B. `novacore::constants::APPLICATION_NAME`).
- **Interaktionsmuster:** Direkte Funktionsaufrufe und Typverwendung.
- **Datenfluss:** `novade-core` liefert Bausteine, aber empfängt keine schichtspezifischen Daten zurück.
- **Beispiel (Konzeptuell):**
    
    Rust
    
    ```
    // In novade-domain/src/models/user_settings.rs
    use novade_core::types::{NovaUuid, NovaErrorBase};
    use novade_core::traits::Validatable;
    
    pub struct UserSetting {
        id: NovaUuid,
        key: String,
        value: String, // Vereinfacht, wird später detaillierter
    }
    
    impl Validatable for UserSetting {
        fn validate(&self) -> Result<(), NovaErrorBase> {
            // Validierungslogik
            Ok(())
        }
    }
    ```
    

##### 3.2. Schnittstelle: `novade-domain` -> `novade-system`

- **Exponierte Elemente durch `novade-domain`:**
    - **Domänenmodelle:** Vollständige Definitionen von Entitäten und Werteobjekten (z.B. `Window`, `Workspace`, `ApplicationProfile`, `NotificationSpec`). Diese SIND `pub` und repräsentieren den "Was"-Aspekt.
    - **Domänenservices (als Traits oder konkrete Typen):** Schnittstellen oder Implementierungen für Domänenlogik (z.B. `WindowManagementPolicyService`, `SettingsValidationService`).
    - **Repository-Traits:** Abstrakte Definitionen für Datenzugriffsoperationen (z.B. `trait SettingsRepository { fn load_settings(&self, user_id: &NovaUuid) -> Result<UserSettings, NovaErrorBase>; }`).
    - **Domänenereignisse:** Typdefinitionen für Ereignisse (z.B. `enum DomainEvent { WorkspaceCreated(Workspace), ... }`).
- **Interaktionsmuster:**
    - `novade-system` instanziiert oder verwendet Domänenmodelle.
    - `novade-system` ruft Methoden von Domänenservices auf, um Geschäftsregeln auszuführen oder komplexe Operationen zu validieren.
    - `novade-system` implementiert die von `novade-domain` definierten Repository-Traits.
- **Datenfluss:** `novade-system` übergibt Rohdaten oder Anfragen an `novade-domain`. `novade-domain` retourniert validierte Domänenobjekte, Ergebnisse von Logikoperationen oder Domänenereignisse.
- **Beispiel (Konzeptuell):**
    
    Rust
    
    ```
    // In novade_domain/src/services/settings_service.rs
    pub trait SettingsPolicyService {
        fn apply_brightness_policy(&self, current_brightness: u8, profile: &PowerProfile) -> u8;
    }
    
    // In novade_system/src/power_manager_service/manager.rs
    use novade_domain::models::PowerProfile;
    use novade_domain::services::SettingsPolicyService;
    // ...
    fn update_brightness(&self, settings_policy_service: &dyn SettingsPolicyService) {
        let current_brightness = self.backend.get_current_brightness(); // Hardware-Zugriff
        let active_profile = self.settings_repo.load_active_power_profile();
        let new_brightness = settings_policy_service.apply_brightness_policy(current_brightness, &active_profile);
        self.backend.set_brightness(new_brightness);
    }
    ```
    

##### 3.3. Schnittstelle: `novade-system` -> `novade-ui`

- **Exponierte Elemente durch `novade-system`:**
    - **System-API-Fassade:** Eine oder mehrere Fassaden (Facade Pattern) oder spezialisierte Service-Module, die eine vereinfachte und stabile Schnittstelle zu den Systemfunktionen bieten. Dies MUSS die Komplexität der Systeminteraktionen verbergen.
        - Beispiele: `DisplayManagerApi`, `NetworkManagerApi`, `NotificationSystemApi`, `UserSessionApi`.
    - **Datenübertragungsobjekte (DTOs):** Spezifische Structs, die für die Kommunikation von Daten an die UI-Schicht optimiert sind. Diese können von Domänenmodellen abgeleitet sein, aber für die UI-Darstellung zugeschnitten werden (ViewModel-ähnlich).
    - **Asynchrone Operationen:** Funktionen, die `Future`s zurückgeben (via `async fn`), für langlaufende Systemoperationen.
    - **Event-Streams / Callbacks:** Mechanismen für die `novade-ui`, um auf Systemereignisse zu reagieren (z.B. `tokio::sync::watch::Receiver` für Zustandsänderungen, `tokio::sync::broadcast::Receiver` für Ereignisse, oder Callback-Registrierungen).
- **Interaktionsmuster:**
    - `novade-ui` ruft Methoden der System-API-Fassaden auf, um Aktionen auszulösen (z.B. "Netzwerk verbinden", "Fenster minimieren", "Helligkeit ändern").
    - `novade-ui` abonniert Event-Streams oder registriert Callbacks, um über Systemänderungen informiert zu werden (z.B. "Netzwerkstatus geändert", "Neue Benachrichtigung", "Fensterfokus geändert").
- **Datenfluss:** `novade-ui` sendet Anfragen (ggf. mit Parametern) an `novade-system`. `novade-system` retourniert Ergebnisse (DTOs, Bestätigungen, Fehler) und sendet asynchrone Ereignisbenachrichtigungen.
- **Beispiel (Konzeptuell):**
    
    Rust
    
    ```
    // In novade_system/src/api/display_manager_api.rs
    pub struct DisplayInfoDTO { /* ... */ }
    pub struct MonitorDTO { pub id: String, pub resolution: (u32, u32), /* ... */ }
    
    #[async_trait::async_trait] // Falls erforderlich
    pub trait DisplayManagerApi {
        async fn list_monitors(&self) -> Result<Vec<MonitorDTO>, SystemApiError>;
        async fn set_monitor_resolution(&self, monitor_id: &str, width: u32, height: u32) -> Result<(), SystemApiError>;
        // Stream für Monitor-Änderungen
        fn monitor_change_stream(&self) -> tokio::sync::broadcast::Receiver<MonitorChangeEvent>;
    }
    
    // In novade_ui/src/widgets/display_settings_widget.rs
    // let display_api: Arc<dyn DisplayManagerApi> = get_system_api_handle(); // Via DI
    // let monitors = display_api.list_monitors().await?;
    // display_api.monitor_change_stream().subscribe()...
    ```
    

---

#### 4. Kommunikationsprotokolle und Datenformate

##### 4.1. Intra-Prozess-Kommunikation (Innerhalb von NovaDE)

- **Primäres Protokoll:** Direkte synchrone und asynchrone (Rust `async/await` mit `tokio`) Funktionsaufrufe zwischen Crates.
- **Datenformat:** Native Rust-Datenstrukturen (`struct`, `enum`).
    - Für Daten, die über Schichtgrenzen hinweg übergeben werden, insbesondere von `novade-system` an `novade-ui` oder als Ergebnisse von Repository-Implementierungen, MÜSSEN diese Strukturen `Send + Sync + 'static` sein, wenn sie in asynchronen Kontexten oder über Thread-Grenzen hinweg verwendet werden.
    - Serialisierung (z.B. mit `serde`) IST NICHT ERFORDERLICH für die interne Kommunikation, es sei denn, es handelt sich um Persistenz oder IPC.
- **Event-Übertragung:**
    - Für State-Propagation: `tokio::sync::watch` Kanäle.
    - Für diskrete Events: `tokio::sync::broadcast` Kanäle oder dedizierte Callback-Mechanismen.
    - Alternative für komplexe Event-Bus-Szenarien: Crates wie `event-listener` oder `async-broadcast`.

##### 4.2. Inter-Prozess-Kommunikation (IPC)

- **Anwendungsfälle:**
    1. Kommunikation zwischen NovaDE-Systemdiensten und externen Anwendungen (z.B. eine Anwendung fragt den `NotificationService` an).
    2. Kommunikation zwischen verschiedenen NovaDE-Prozessen, falls NovaDE zukünftig in mehrere separate Prozesse aufgeteilt wird (aktuell nicht der primäre Fokus, aber die Architektur SOLLTE es ermöglichen).
    3. Interaktion mit bestehenden Systemdiensten (z.B. `NetworkManager`, `UPower`, `logind` via deren D-Bus-Schnittstellen).
- **Primäres Protokoll:** **D-Bus**.
    - NovaDE-Systemdienste (z.B. `NotificationService`, `ClipboardService`, `GlobalShortcutService`) MÜSSEN standardisierte D-Bus-Schnittstellen exponieren.
    - Die `novade-system` Schicht MUSS Bibliotheken wie `zbus` oder `dbus-rs` für die D-Bus-Kommunikation verwenden. Die Wahl MUSS auf `async` Kompatibilität mit `tokio` basieren. `zbus` wird aufgrund seiner modernen API und `async` Integration bevorzugt.
- **Datenformat für D-Bus:** Standard D-Bus Datentypen. Komplexe Rust-Strukturen MÜSSEN in D-Bus-kompatible Typen (oft Dictionaries oder Structs aus Basistypen) konvertiert werden. `serde` KANN hierfür in Verbindung mit D-Bus Bibliotheken genutzt werden.
- **Alternative IPC (falls D-Bus ungeeignet):** Für sehr hochperformante, lokale IPC zwischen eng gekoppelten NovaDE-Prozessen KÖNNTEN Unix Domain Sockets mit einem binären Serialisierungsformat (z.B. `bincode` oder `protobuf`) in Betracht gezogen werden, aber D-Bus IST der Standard für externe Schnittstellen.

##### 4.3. Datenpersistenzformate

- **Konfigurationsdateien:**
    - Format: **TOML** (`.toml`) für von Menschen lesbare und einfach editierbare Konfigurationen.
    - Serialisierung/Deserialisierung: Die `serde` Crate in Verbindung mit `toml-rs`.
    - Struktur: Klar definierte `structs` in `novade-core` oder `novade-domain`, die die Konfigurationsstruktur abbilden.
- **Benutzerdaten / Anwendungsstatus (komplexer Natur):**
    - Format: **SQLite** für strukturierte Daten, die Abfragen oder Transaktionen erfordern (z.B. Verlaufsdaten, komplexere Anwendungszustände).
    - Zugriff: Über die `rusqlite` Crate, potenziell mit einer ORM-Lite-Schicht wie `sqlx` (falls asynchroner Zugriff und compile-time checks gewünscht sind). `sqlx` wird aufgrund der `async`-Natur von NovaDE bevorzugt.
- **Binäre Caches / Temporäre Daten:**
    - Format: `bincode` oder ein anderes effizientes binäres Serialisierungsformat.
    - Anwendung: Für das schnelle Speichern und Laden von internen Zuständen, die nicht menschenlesbar sein müssen (z.B. gecachte UI-Zustände, vorberechnete Daten).

---

#### 5. Fehlerbehandlungsstrategien

Eine robuste und konsistente Fehlerbehandlung IST KRITISCH.

- **Fehlertypen:**
    1. **Dedizierte Fehler-Enums pro Modul/Crate:** Jedes Modul oder jede Crate SOLLTE ein eigenes `Error` Enum definieren, das spezifische Fehlerfälle abbildet.
        
        Rust
        
        ```
        // Beispiel: novade_system/src/network_manager_service/error.rs
        #[derive(Debug, thiserror::Error)] // thiserror für einfache Implementierung von std::error::Error
        pub enum NetworkManagerError {
            #[error("D-Bus connection failed: {0}")]
            DBusConnection(String),
            #[error("Network interface '{0}' not found")]
            InterfaceNotFound(String),
            #[error("Operation timed out")]
            Timeout,
            #[error("Underlying I/O error: {0}")]
            Io(#[from] std::io::Error), // Fehlerkonvertierung
            // ... weitere Fehler
        }
        ```
        
    2. **Wrapper-Fehlertypen für Schichten:** Jede Schicht (insbesondere `novade-system` und `novade-ui` für ihre exponierten APIs) KANN einen übergreifenden Fehler-Wrapper definieren, der Fehler aus unteren Schichten oder Modulen aggregiert.
    3. **Basis-Fehler-Trait in `novade-core`:** Ein Trait `NovaError` oder ein Basis-Enum `NovaCoreError` KANN für allgemeine Fehlerkategorien (I/O, Serialisierung, Konfiguration) und zur Vereinfachung der Fehlerkonvertierung dienen.
- **Fehlerpropagation:**
    - Das `Result<T, E>`-Typ MUSS konsequent für alle Operationen verwendet werden, die fehlschlagen können.
    - Der `?`-Operator MUSS zur Propagation von Fehlern verwendet werden.
    - Explizite `match`-Anweisungen oder `map_err` SIND ZU VERWENDEN, wenn Fehler kontextualisiert oder in einen anderen Fehlertyp umgewandelt werden müssen.
- **Fehlerkontext und Quellen (Error Chaining):**
    - Die `thiserror` Crate MUSS verwendet werden, um `std::error::Error` leicht zu implementieren und Fehlerquellen (`#[from]`, `#[source]`) zu definieren.
    - Die `eyre` oder `anyhow` Crates KÖNNEN für Anwendungs-Top-Level-Fehlerbehandlung (z.B. in `main.rs` der UI-Anwendung) in Betracht gezogen werden, um einfache Fehlerberichte mit Backtraces zu erstellen. Für Bibliotheks-Crates (alle `novade-*` Crates) SIND spezifische Fehlertypen mit `thiserror` zu bevorzugen.
- **Logging von Fehlern:**
    - Fehler MÜSSEN an der Stelle geloggt werden, an der sie behandelt werden oder an der genügend Kontext für eine aussagekräftige Log-Nachricht vorhanden ist.
    - Unbehandelte Fehler, die bis zum Top-Level propagieren, MÜSSEN dort geloggt werden, bevor die Anwendung oder der Thread terminiert wird. Log-Level `ERROR` oder `CRITICAL` verwenden.
- **Fehler-Recovery:**
    - Wo immer möglich, SOLLTEN Strategien zur Fehlerbehebung implementiert werden (z.B. Wiederholungsversuche mit Backoff für Netzwerkoperationen, Fallback auf Standardwerte bei Konfigurationsfehlern).
    - Circuit Breaker Pattern KANN für Dienste in Betracht gezogen werden, die von externen, potenziell unzuverlässigen Ressourcen abhängen.
- **Keine Panics für erwartete Fehler:** `panic!` MUSS vermieden werden für Fehlerzustände, die erwartet werden können und von denen sich das System erholen kann oder die dem Benutzer gemeldet werden können. Panics SIND für irreparable Programmierfehler (z.B. verletzte Invarianten) reserviert.

---

#### 6. Caching- und Persistierungsstrategien

##### 6.1. Caching

- **Zweck:** Verbesserung der Performance und Reduzierung der Latenz durch Speicherung häufig abgerufener oder teuer zu berechnender Daten im Speicher.
- **Anwendungsbereiche:**
    - **`novade-system`:** Caching von Systeminformationen (z.B. Hardware-Details, häufig abgefragte D-Bus-Eigenschaften), Ergebnisse von teuren OS-Aufrufen.
    - **`novade-ui`:** Caching von gerenderten Elementen (falls nicht vom UI-Framework gehandhabt), Icons, Thumbnails, vorberechnete Layout-Informationen.
    - **`novade-domain`:** Caching von Ergebnissen von Domänenlogik-Berechnungen, wenn diese deterministisch und teuer sind.
- **Strategien:**
    1. **In-Memory Cache (pro Komponente/Service):**
        - Verwendung von `std::collections::HashMap` oder `LruCache` (z.B. aus der `lru` Crate) für einfache LRU-basierte Caches.
        - Asynchrone Caches: Crates wie `async-lock::RwLock` oder spezifische `async-cache` Implementierungen.
    2. **Cache-Invalidierung:**
        - **Zeitbasiert (TTL - Time To Live):** Einträge verfallen nach einer bestimmten Zeit.
        - **Eventbasiert:** Cache wird invalidiert, wenn bestimmte System- oder Domänenereignisse auftreten (z.B. Konfigurationsänderung invalidiert Konfigurationscache).
        - **Manuell:** Explizite Invalidierung durch Code.
    3. **Write-Through / Write-Back Caching:** Für Daten, die auch persistiert werden, um Konsistenz zu gewährleisten (eher relevant für Persistenz).
- **Zu cachende Daten MÜSSEN sorgfältig ausgewählt werden:**
    - Häufigkeit des Zugriffs.
    - Kosten der Neuberechnung/-beschaffung.
    - Größe der Daten.
    - Akzeptable Veralterung der Daten.
- **Spezifikation für Cache-Implementierungen:**
    - Maximale Größe/Anzahl der Einträge MUSS definierbar sein.
    - Invalidierungsstrategie MUSS klar definiert sein.
    - Thread-Sicherheit MUSS gewährleistet sein (`Arc<Mutex<Cache>>` oder `Arc<RwLock<Cache>>` bzw. äquivalente `tokio::sync` Primitive).

##### 6.2. Persistierung

- **Zweck:** Dauerhafte Speicherung von Daten über Anwendungsneustarts hinweg.
- **Anwendungsbereiche:**
    - **Benutzereinstellungen:** Alle Konfigurationen, die vom Benutzer vorgenommen werden (Theme, Tastenkürzel, Panel-Konfiguration, Anwendungseinstellungen). Ort: Gemäß XDG Base Directory Specification (`$XDG_CONFIG_HOME/novade/...`). Format: TOML.
    - **Anwendungszustände:** Spezifische Zustände von NovaDE-Komponenten, die einen Neustart überdauern sollen (z.B. zuletzt geöffnete Dateien pro Anwendung, Fensterpositionen – falls gewünscht). Ort: Gemäß XDG Base Directory Specification (`$XDG_STATE_HOME/novade/...` oder `$XDG_DATA_HOME/novade/...`). Format: SQLite oder Binärformat.
    - **Systemweite Konfiguration (falls von NovaDE verwaltet):** Konfigurationen, die das gesamte System betreffen (vorsichtig verwenden, typischerweise Aufgabe des OS). Ort: `/etc/novade/...` (erfordert Root-Rechte für Änderungen).
- **Strategien:**
    1. **Explizites Speichern bei Änderung:** Daten werden sofort persistiert, wenn sie sich ändern (einfach, aber potenziell I/O-intensiv).
    2. **Speichern bei Beendigung (Graceful Shutdown):** Daten werden beim Herunterfahren der Komponente oder Anwendung gespeichert (Risiko von Datenverlust bei Absturz).
    3. **Periodisches Speichern / Autosave:** Daten werden in regelmäßigen Abständen gespeichert (Kompromiss).
    4. **Transaktionales Speichern (für kritische Daten):** Verwendung von Datenbanktransaktionen (SQLite) oder atomaren Dateischreibvorgängen (Schreiben in temporäre Datei, dann atomares Verschieben/Umbenennen) um Datenkonsistenz sicherzustellen. DIES IST DIE BEVORZUGTE METHODE FÜR KONFIGURATIONSDATEIEN.
- **Repository Pattern:** Die Persistenzlogik MUSS hinter Repository-Schnittstellen in der `novade-system` Schicht gekapselt werden. `novade-domain` definiert nur die Traits.
- **Migration von Datenformaten:** Mechanismen für die Migration von alten Datenformaten zu neuen Versionen MÜSSEN bei der Gestaltung der Persistenz berücksichtigt werden, falls sich Datenstrukturen ändern.

Diese Strategien bilden die Grundlage für ein robustes und performantes Datenmanagement innerhalb von NovaDE.

---

## 7. Zerlegung der Schicht `novade-core` in kohärente Module

Die Schicht `novade-core` IST die Basis für alle anderen Schichten in NovaDE. Ihre Module MÜSSEN generisch, stabil und frei von spezifischer Domänen- oder Systemlogik sein. Die folgende modulare Struktur IST für `novade-core` (Crate: `novade_core`) verbindlich:

1.  **Modul `types`:**
    * **Pfad:** `novade_core/src/types/mod.rs` und Untermodule (z.B. `novade_core/src/types/id.rs`, `novade_core/src/types/geometry.rs`)
2.  **Modul `error`:**
    * **Pfad:** `novade_core/src/error.rs` (oder `novade_core/src/error/mod.rs`)
3.  **Modul `utils`:**
    * **Pfad:** `novade_core/src/utils/mod.rs` und Untermodule (z.B. `novade_core/src/utils/string_utils.rs`, `novade_core/src/utils/macros.rs`)
4.  **Modul `config_loader`:**
    * **Pfad:** `novade_core/src/config_loader/mod.rs`
5.  **Modul `logging`:**
    * **Pfad:** `novade_core/src/logging/mod.rs`
6.  **Modul `constants`:**
    * **Pfad:** `novade_core/src/constants.rs`

Diese Module SIND so konzipiert, dass sie minimale Interdependenzen untereinander aufweisen, mit Ausnahme der Nutzung von Basistypen aus `types` oder Fehlerdefinitionen aus `error`.

---

#### 8. Definition der Modulverantwortlichkeiten und -grenzen für `novade-core`

##### 8.1. Modul `types` (`novade_core::types`)

* **Verantwortlichkeiten:**
    1.  **Definition fundamentaler, atomarer Datentypen:** Bereitstellung von projektweit genutzten, generischen Typ-Aliassen und Structs.
        * `NovaUuid`: Ein Wrapper um eine UUID-Implementierung (z.B. `uuid::Uuid`), um eine konsistente ID-Verwendung sicherzustellen.
        * Geometrische Basistypen: `Point2D<T>`, `Size2D<T>`, `Rect<T>` für 2D-Koordinaten und Dimensionen, generisch über den numerischen Typ `T`.
        * Basis-Enums für Zustände oder Kategorien, die keine spezifische Domänenlogik haben (z.B. `SortOrder { Ascending, Descending }`).
    2.  **Definition von Traits für Basistypen:** Bereitstellung von allgemeinen Traits, die auf diesen Typen operieren oder von ihnen implementiert werden (z.B. `trait AsPoint { fn as_point(&self) -> Point2D<i32>; }`).
* **Grenzen:**
    * KEINE Logik, die spezifisch für Domänenobjekte, Systeminteraktionen oder UI-Darstellung ist.
    * KEINE externen Abhängigkeiten außer der Rust Standard Library und fundamentalen Utility-Crates (z.B. `uuid`, `serde` für `Serialize/Deserialize` derive, falls Typen persistierbar sein sollen auf Basis-Level).
* **Struktur (Beispiel für Untermodule):**
    * `id.rs`: Enthält `NovaUuid` und zugehörige Helfer.
    * `geometry.rs`: Enthält `Point2D`, `Size2D`, `Rect`.
    * `common.rs`: Enthält weitere generische Typen oder Enums.

##### 8.2. Modul `error` (`novade_core::error`)

* **Verantwortlichkeiten:**
    1.  **Definition eines Basis-Fehlertyps:** Bereitstellung eines `NovaCoreError` Enum oder Struct, das allgemeine Fehlerkategorien der Kernschicht abbildet (z.B. `IoError`, `SerializationError`, `ConfigParseError`, `InvalidParameter`).
    2.  **Implementierung von `std::error::Error`:** Verwendung von `thiserror::Error` zur einfachen Ableitung des `Error` Traits.
    3.  **Unterstützung für Fehlerkontext und -verkettung:** Ermöglichung der Aufnahme von Quellfehlern (`#[source]`) und Kontextinformationen.
    4.  **Bereitstellung eines generischen `NovaResult<T>` Alias:** `type NovaResult<T> = Result<T, NovaCoreError>;`
* **Grenzen:**
    * Definiert NUR Fehler, die ihren Ursprung direkt in `novade-core` Operationen haben oder allgemeine Fehlerkategorien darstellen, die von höheren Schichten wiederverwendet werden können.
    * KEINE domänen-, system- oder UI-spezifischen Fehler.
* **Struktur:** Typischerweise eine einzelne `error.rs` Datei, die das Haupt-Enum `NovaCoreError` und ggf. kleinere Hilfs-Enums/Structs enthält.

##### 8.3. Modul `utils` (`novade_core::utils`)

* **Verantwortlichkeiten:**
    1.  **Bereitstellung generischer Hilfsfunktionen und -makros:** Sammlungen von Funktionen, die keine eigene Zustandsverwaltung haben und reine Berechnungen oder Transformationen durchführen.
        * String-Manipulationen (z.B. `truncate_string`, `is_alphanumeric`).
        * Collection-Helfer (z.B. `group_by_key`, `find_duplicate`).
        * Dateisystem-Helfer für Pfadoperationen (ABER keine direkten Lese-/Schreibvorgänge, die `std::io::Error` spezifisch behandeln müssten – diese wären eher in `config_loader` oder spezifischen Systemmodulen).
        * Generische Makros (z.B. `unwrap_or_return_err!`, `cfg_debug!`).
    2.  **Zeit- und Datums-Utilities:** Wrapper oder Helfer für die Arbeit mit Zeit (z.B. `chrono` Crate), falls generische Funktionalität benötigt wird.
* **Grenzen:**
    * KEINE spezifische Geschäftslogik.
    * KEINE Abhängigkeiten von anderen `novade-core` Modulen außer potenziell `types` für Parameter- oder Rückgabetypen.
* **Struktur (Beispiel für Untermodule):**
    * `string_utils.rs`
    * `collection_utils.rs`
    * `time_utils.rs`
    * `macros.rs`

##### 8.4. Modul `config_loader` (`novade_core::config_loader`)

* **Verantwortlichkeiten:**
    1.  **Bereitstellung generischer Mechanismen zum Laden und Parsen von Konfigurationsdateien.**
    2.  **Abstraktion des Konfigurationsformats (primär TOML):** Verwendung von `serde` und `toml-rs` zur Deserialisierung von Konfigurationsdaten in typisierte Rust-Strukturen.
    3.  **Definition von Traits für konfigurierbare Komponenten:** (Optional) Ein Trait wie `trait LoadableConfig: DeserializeOwned { fn load_from_path(path: &Path) -> Result<Self, NovaCoreError>; }`.
    4.  **Pfadmanagement für Konfigurationsdateien:** Grundlegende Logik zur Bestimmung von Konfigurationsdateipfaden basierend auf XDG-Richtlinien oder übergebenen Pfaden (NUR die Logik zur Pfad*bestimmung*, nicht das Laden selbst, das ist aufgabenspezifisch).
* **Grenzen:**
    * Beschränkt sich auf das technische Laden und Parsen. Die Definition der *Struktur* der Konfigurationsdaten obliegt den Modulen/Schichten, die diese Konfiguration benötigen.
    * Fehlerbehandlung MUSS auf `NovaCoreError` (z.B. `ConfigParseError`, `IoError`) abgebildet werden.
* **Struktur:** Enthält Funktionen wie `load_toml_config<T: DeserializeOwned>(path: &Path) -> NovaResult<T>`.

##### 8.5. Modul `logging` (`novade_core::logging`)

* **Verantwortlichkeiten:**
    1.  **Initialisierung und Konfiguration der globalen Logging-Infrastruktur:** Verwendung einer Logging-Fassade wie `tracing` (bevorzugt) oder `log`.
    2.  **Bereitstellung von einfachen Initialisierungsfunktionen:** z.B. `init_logging(level: LogLevel, output: LogOutput)`.
    3.  **Definition von Log-Levels und Ausgabeoptionen (Konsole, Datei):** Als Typen oder Enums innerhalb dieses Moduls.
    4.  **Integration mit System-Logging (optional):** Zukünftige Möglichkeit zur Integration mit `journald` o.ä.
* **Grenzen:**
    * Stellt nur die Infrastruktur bereit. Die eigentlichen Log-Aufrufe (`info!`, `warn!`, `error!`) erfolgen direkt über die Makros der gewählten Logging-Crate in den jeweiligen Modulen des gesamten Projekts.
* **Struktur:** Enthält Initialisierungsfunktionen und ggf. Konfigurationsstrukturen für das Logging.

##### 8.6. Modul `constants` (`novade_core::constants`)

* **Verantwortlichkeiten:**
    1.  **Definition von globalen, unveränderlichen Konstanten:** Werte, die systemweit verwendet werden und keinen dynamischen Zustand haben.
        * Anwendungsname: `pub const APPLICATION_NAME: &str = "NovaDE";`
        * Standard-Timeout-Werte (falls generisch anwendbar).
        * Standard-Pfade oder Dateinamen (nur die Namen, nicht die volle Pfadlogik).
        * Versionsnummer der Core-Schicht (kann aus `Cargo.toml` via `env!("CARGO_PKG_VERSION")` bezogen werden).
* **Grenzen:**
    * NUR echte Konstanten. Keine konfigurierbaren Werte.
* **Struktur:** Eine einzelne `constants.rs` Datei mit `pub const` Definitionen.

---

#### 9. Spezifikation der Schnittstellen zwischen Modulen innerhalb von `novade-core`

Die Interaktion zwischen den Modulen von `novade-core` ist typischerweise unidirektional oder minimal:

* **`types`:** Wird von fast allen anderen `novade-core` Modulen sowie allen höheren Schichten verwendet. Exportiert `pub` Typen und Traits.
* **`error`:** Wird von `config_loader`, `logging` und potenziell `utils` verwendet, um Fehler zu definieren oder zurückzugeben. Wird auch von allen höheren Schichten für Basisfehlerbehandlung genutzt. Exportiert `pub NovaCoreError` und `NovaResult`.
* **`utils`:** Kann `types` für Parameter/Rückgabetypen verwenden. Wird von anderen `novade-core` Modulen und höheren Schichten für Hilfsfunktionen genutzt. Exportiert `pub` Funktionen und Makros.
* **`config_loader`:** Verwendet `types` (für `DeserializeOwned`), `error` (für `NovaCoreError`), und potenziell `utils` für Pfad-Helfer. Wird von höheren Schichten aufgerufen, um Konfigurationen zu laden. Exportiert `pub` Ladefunktionen und Traits.
* **`logging`:** Verwendet `types` (für `LogLevel` Enum), `error` (für Initialisierungsfehler). Wird typischerweise einmalig beim Start einer Anwendung aus einer höheren Schicht (z.B. `novade-ui/src/main.rs` oder einem Hauptservice in `novade-system`) initialisiert. Exportiert `pub` Initialisierungsfunktionen. Die Logging-Makros (`info!`, etc.) werden direkt von der zugrundeliegenden Logging-Crate bereitgestellt.
* **`constants`:** Wird von allen anderen Modulen und Schichten bei Bedarf verwendet. Exportiert `pub const` Werte.

**Formale API-Definitionen (Auszug):**

* `pub mod novade_core::types { pub struct Point2D<T> { pub x: T, pub y: T } ... }`
* `pub mod novade_core::error { pub enum NovaCoreError { ... } pub type NovaResult<T> = Result<T, NovaCoreError>; }`
* `pub mod novade_core::utils { pub fn some_utility_function(input: &str) -> String { ... } }`
* `pub mod novade_core::config_loader { use crate::error::NovaResult; use serde::de::DeserializeOwned; use std::path::Path; pub fn load_toml_config<T: DeserializeOwned>(path: &Path) -> NovaResult<T>; }`
* `pub mod novade_core::logging { use crate::error::NovaResult; pub enum LogLevel { Debug, Info, Warn, Error } pub fn init_global_logging(level: LogLevel) -> NovaResult<()>; }`
* `pub mod novade_core::constants { pub const CORE_VERSION: &str = "0.1.0"; }`

---

#### 10. Abhängigkeitsdiagramm für `novade-core` Module

Da die Beziehungen relativ einfach sind, kann dies textuell beschrieben werden. Ein komplexeres Diagramm ist hier nicht zwingend notwendig, aber für das Verständnis hilfreich.

```mermaid
graph LR
    subgraph novade_core
        types_mod[types]
        error_mod[error]
        utils_mod[utils]
        config_loader_mod[config_loader]
        logging_mod[logging]
        constants_mod[constants]

        error_mod --> types_mod  // error kann Basistypen für Kontext verwenden
        utils_mod --> types_mod
        config_loader_mod --> types_mod
        config_loader_mod --> error_mod
        logging_mod --> types_mod // für LogLevel Enum z.B.
        logging_mod --> error_mod // für Init-Fehler

        %% Alle anderen Module können constants und types potentiell nutzen
        utils_mod --> constants_mod
        error_mod --> constants_mod
        config_loader_mod --> constants_mod
        logging_mod --> constants_mod
    end

    %% Externe Schichten (symbolisch)
    higher_layer[Höhere Schichten (domain, system, ui)]
    higher_layer --> types_mod
    higher_layer --> error_mod
    higher_layer --> utils_mod
    higher_layer --> config_loader_mod
    higher_layer --> logging_mod %% Hauptsächlich für Initialisierung
    higher_layer --> constants_mod
```

**Interpretation des Diagramms:**

* Pfeile (`-->`) bedeuten "hängt ab von" oder "verwendet".
* `types` und `constants` sind die fundamentalsten Module ohne interne `novade_core` Abhängigkeiten (außer potenziell zu externen Basis-Crates).
* `error` kann `types` für detailliertere Fehlerinformationen verwenden.
* `utils`, `config_loader`, `logging` können `types` und `error` verwenden.
* Alle höheren Schichten können auf alle öffentlichen Module von `novade_core` zugreifen, insbesondere auf `types`, `error`, `constants` und `utils`. `config_loader` und `logging` werden typischerweise gezielt für spezifische Aufgaben (Laden, Initialisierung) von höheren Schichten genutzt.

---

#### 11. & 12. Modul-Roadmap, Arbeitspakete und Implementierungsreihenfolge für `novade-core`

Die Implementierung der `novade-core` Module SOLLTE in folgender Reihenfolge erfolgen, um Abhängigkeiten aufzulösen:

1.  **`constants.rs`:** (Priorität: 1 - Sehr hoch)
    * **Arbeitspaket:** Definition aller initial identifizierten globalen Konstanten.
    * **Begründung:** Keine Abhängigkeiten, wird von vielen anderen benötigt.
2.  **`types/mod.rs` (und Untermodule wie `id.rs`, `geometry.rs`):** (Priorität: 1 - Sehr hoch)
    * **Arbeitspaket:** Definition aller fundamentalen Datentypen (`NovaUuid`, `Point2D`, etc.).
    * **Begründung:** Basis für fast alles andere.
3.  **`error.rs`:** (Priorität: 1 - Sehr hoch)
    * **Arbeitspaket:** Definition von `NovaCoreError` und `NovaResult`.
    * **Begründung:** Essentiell für jegliche fehleranfällige Operation.
4.  **`logging/mod.rs`:** (Priorität: 2 - Hoch)
    * **Arbeitspaket:** Implementierung der Logging-Initialisierung (`tracing` Setup).
    * **Begründung:** Frühzeitige Verfügbarkeit von Logging erleichtert die Entwicklung aller nachfolgenden Module.
5.  **`utils/mod.rs` (und Untermodule):** (Priorität: 2 - Hoch)
    * **Arbeitspaket:** Implementierung der ersten Charge benötigter Utility-Funktionen und Makros. Kann iterativ erweitert werden.
    * **Begründung:** Nützliche Helfer, die oft benötigt werden.
6.  **`config_loader/mod.rs`:** (Priorität: 3 - Mittel)
    * **Arbeitspaket:** Implementierung der generischen TOML-Ladefunktion.
    * **Begründung:** Wird benötigt, sobald die ersten Konfigurationen in höheren Schichten definiert werden.

Jedes Arbeitspaket MUSS Unit-Tests beinhalten, die die korrekte Funktionalität des Moduls sicherstellen. Die Code-Coverage für `novade-core` MUSS hoch sein (>90%), da es die Grundlage für die Stabilität des Gesamtsystems bildet.

---

## 7. Zerlegung der Schicht `novade-domain` in kohärente Module

Die Schicht `novade-domain` (Crate: `novade_domain`) enthält die Kernlogik, die Entitäten und die Regeln, die das Verhalten von NovaDE definieren, unabhängig von spezifischen Systemimplementierungen oder UI-Darstellungen. Die folgende modulare Struktur IST für `novade_domain` verbindlich:

1.  **Modul `models`:**
    * **Pfad:** `novade_domain/src/models/mod.rs`
    * **Untermodule (Beispiele):**
        * `novade_domain/src/models/common.rs` (für gemeinsam genutzte Domänen-Werteobjekte)
        * `novade_domain/src/models/settings.rs` (z.B. `UserSettings`, `DisplayProfile`, `PowerProfile`)
        * `novade_domain/src/models/desktop.rs` (z.B. `Workspace`, `PanelConfig`)
        * `novade_domain/src/models/application.rs` (z.B. `ApplicationProfile`, `RunningApplication`)
        * `novade_domain/src/models/window.rs` (z.B. `Window`, `WindowState`, `WindowLayoutHint`)
        * `novade_domain/src/models/notification.rs` (z.B. `NotificationSpec`, `NotificationUrgency`)
        * `novade_domain/src/models/ai_knowledge.rs` (z.B. `KnowledgeItem`, `UserAiPreference` - basierend auf MCP-Anforderungen)
2.  **Modul `services`:**
    * **Pfad:** `novade_domain/src/services/mod.rs`
    * **Untermodule (Beispiele):**
        * `novade_domain/src/services/settings_policy_service.rs`
        * `novade_domain/src/services/window_management_policy_service.rs`
        * `novade_domain/src/services/notification_policy_service.rs`
        * `novade_domain/src/services/ai_context_policy_service.rs`
3.  **Modul `repositories`:**
    * **Pfad:** `novade_domain/src/repositories/mod.rs`
    * **Untermodule (Beispiele):**
        * `novade_domain/src/repositories/settings_repository.rs`
        * `novade_domain/src/repositories/application_repository.rs`
        * `novade_domain/src/repositories/ai_knowledge_repository.rs`
4.  **Modul `events`:**
    * **Pfad:** `novade_domain/src/events.rs` (oder `novade_domain/src/events/mod.rs`)
5.  **Modul `error`:**
    * **Pfad:** `novade_domain/src/error.rs` (oder `novade_domain/src/error/mod.rs`)

Diese Struktur ermöglicht eine klare Trennung zwischen Daten (Modelle), Verhalten (Services/Policies) und Datenzugriffsabstraktionen (Repositories).

---

#### 8. Definition der Modulverantwortlichkeiten und -grenzen für `novade-domain`

##### 8.1. Modul `models` (`novade_domain::models`)

* **Verantwortlichkeiten:**
    1.  **Definition von Domänenentitäten und Werteobjekten:** Repräsentation der Kernkonzepte von NovaDE. Jede Entität MUSS eine eindeutige Identität besitzen (oft eine `NovaUuid` aus `novade_core`). Werteobjekte SIND durch ihre Attribute definiert und unveränderlich.
        * **Beispiele:** `UserSettings` (Entität), `ColorPalette` (Werteobjekt), `WindowIdentifier` (Werteobjekt), `ApplicationShortcut` (Werteobjekt innerhalb von `ApplicationProfile`).
    2.  **Kapselung von datenzentrierter Logik:** Methoden, die direkt auf den Attributen eines Modells operieren und dessen Invarianten sicherstellen (z.B. Validierungsmethoden, Zustandsübergänge innerhalb eines Modells).
    3.  **Sicherstellung der Konsistenz und Gültigkeit der Modelldaten:** Implementierung von Validierungsregeln direkt in den Modellen oder durch assoziierte Validatoren.
* **Grenzen:**
    * KEINE Abhängigkeiten von der `novade-system` oder `novade-ui` Schicht.
    * KEINE direkte Interaktion mit externen Systemen (Datenbanken, Netzwerk, Dateisystem).
    * Datenstrukturen SIND für die Domänenlogik optimiert, nicht primär für Persistenz oder UI-Darstellung.
* **Struktur:** Untermodule pro Hauptdomänenkonzept (`settings`, `desktop`, `window`, etc.) zur Organisation der Modelldefinitionen. Das Untermodul `common.rs` kann für domänenübergreifende Werteobjekte (z.B. `ResourceIdentifier`, `Timestamp`) genutzt werden.

##### 8.2. Modul `services` (`novade_domain::services`)

* **Verantwortlichkeiten:**
    1.  **Implementierung von Domänenlogik, die mehrere Entitäten oder Modelle koordiniert:** Kapselung von Anwendungsfällen oder komplexen Geschäftsregeln (Policies), die nicht sinnvoll einer einzelnen Entität zugeordnet werden können.
        * **Beispiele:** `SettingsPolicyService::apply_theme(settings: &UserSettings, theme_name: &str) -> Result<UserSettings, DomainError>`, `WindowManagementPolicyService::calculate_next_window_position(current_layout: &WorkspaceLayout, new_window_hints: &WindowLayoutHint) -> Point2D<i32>`.
    2.  **Orchestrierung von Operationen auf Domänenmodellen:** Durchführung von schrittweisen Operationen, die mehrere Modelle betreffen.
    3.  **Zustandslose Natur (primär):** Domänenservices SOLLTEN idealerweise zustandslos sein und alle benötigten Daten als Parameter empfangen. Notwendiger Zustand SOLLTE in Domänenmodellen gehalten werden.
* **Grenzen:**
    * Operieren ausschließlich auf Domänenmodellen und Werten aus `novade_domain::models` und Typen aus `novade_core`.
    * KEINE Kenntnis von UI, Persistenzdetails oder System-APIs.
    * Können andere Domänenservices aufrufen, aber zyklische Abhängigkeiten SIND ZU VERMEIDEN.
* **Struktur:** Pro Domänenbereich ein Service-Modul (z.B. `settings_policy_service.rs`), das Traits und/oder konkrete Implementierungen enthält.

##### 8.3. Modul `repositories` (`novade_domain::repositories`)

* **Verantwortlichkeiten:**
    1.  **Definition von abstrakten Schnittstellen (Traits) für den Datenzugriff:** Spezifikation der Operationen, die zum Abrufen und Speichern von Domänenentitäten benötigt werden.
        * **Beispiele:** `trait UserSettingsRepository { fn get_by_id(&self, id: NovaUuid) -> Result<Option<UserSettings>, DomainError>; fn save(&self, settings: &UserSettings) -> Result<(), DomainError>; }`.
    2.  **Festlegung der "Verträge" für die Datenpersistenz:** Definieren, welche Modelle wie persistiert und abgerufen werden können, ohne die Implementierungsdetails festzulegen.
* **Grenzen:**
    * Enthält NUR Trait-Definitionen. Die konkreten Implementierungen residieren in der `novade-system` Schicht (oder `novade-core` für sehr einfache, generische Fälle).
    * Definiert keine spezifischen Datenbank-Queries oder Dateipfadlogiken.
* **Struktur:** Ein Modul pro Repository-Trait (z.B. `settings_repository.rs`).

##### 8.4. Modul `events` (`novade_domain::events`)

* **Verantwortlichkeiten:**
    1.  **Definition von Domänenereignissen:** Spezifikation von Structs oder Enums, die signifikante Zustandsänderungen oder Vorkommnisse innerhalb der Domäne repräsentieren.
        * **Beispiele:** `enum DomainEvent { UserSettingsChanged { user_id: NovaUuid, changed_settings: Vec<String> }, WindowCreated { window_id: WindowIdentifier, workspace_id: NovaUuid }, ApplicationLaunched { app_id: String } }`.
    2.  **Bereitstellung der Datenstruktur für Ereignisse:** Jedes Ereignis MUSS alle relevanten Informationen als Nutzlast tragen, um von interessierten Listenern verarbeitet zu werden.
* **Grenzen:**
    * Reine Datencontainer für Ereignisinformationen.
    * KEINE Logik zur Event-Verteilung oder -Handhabung (dies geschieht in höheren Schichten oder durch spezifische Event-Bus-Implementierungen).
* **Struktur:** Typischerweise eine einzelne `events.rs` Datei mit einem Haupt-Enum `DomainEvent` oder mehrere spezifische Event-Structs.

##### 8.5. Modul `error` (`novade_domain::error`)

* **Verantwortlichkeiten:**
    1.  **Definition eines spezifischen Fehlertyps für die Domänenschicht:** Bereitstellung eines `DomainError` Enum oder Struct, das Fehlerkategorien abbildet, die in der Domänenlogik auftreten können (z.B. `ValidationError`, `PolicyViolation`, `EntityNotFound`).
    2.  **Implementierung von `std::error::Error`:** Verwendung von `thiserror::Error`.
    3.  **Konvertierung oder Wrapping von `NovaCoreError`:** Fehler aus `novade_core` (z.B. bei der Validierung von Basistypen) KÖNNEN in `DomainError` gewrapped werden.
    4.  **Bereitstellung eines `DomainResult<T>` Alias:** `type DomainResult<T> = Result<T, DomainError>;`
* **Grenzen:**
    * Definiert NUR Fehler, die ihren Ursprung in der Domänenlogik oder der Validierung von Domänenmodellen haben.
* **Struktur:** Eine einzelne `error.rs` Datei.

---

#### 9. Spezifikation der Schnittstellen zwischen Modulen innerhalb von `novade-domain`

* **`models`:**
    * Wird von `services`, `repositories` (als Rückgabe-/Parametertypen in Traits) und `events` (als Teil der Event-Payload) verwendet.
    * Exportiert `pub` Structs und Enums der Domänenmodelle.
* **`services`:**
    * Verwendet `models` für Operationen und `error` für Fehlerbehandlung. Kann `novade_core::utils` für allgemeine Logik nutzen.
    * Kann andere Domänenservices aufrufen.
    * Exportiert `pub` Traits und/oder Structs, die Service-Methoden implementieren.
* **`repositories`:**
    * Verwendet `models` als Typen in den Trait-Methoden-Signaturen und `error` für `Result`-Typen.
    * Exportiert `pub` Traits.
* **`events`:**
    * Verwendet `models` und `novade_core::types` (z.B. `NovaUuid`) für die Event-Payload.
    * Exportiert `pub` Event-Structs/Enums.
* **`error`:**
    * Kann `novade_core::error::NovaCoreError` als `#[source]` verwenden.
    * Wird von `models` (für Validierungsfehler), `services` und `repositories` (in `Result`-Typen) verwendet.
    * Exportiert `pub DomainError` und `DomainResult`.

**Abhängigkeiten von `novade_core`:** Alle Module in `novade_domain` SIND berechtigt, auf öffentliche Elemente von `novade_core` zuzugreifen (`types`, `error`, `utils`, `constants`).

**Formale API-Definitionen (Konzeptioneller Auszug):**

* `pub mod novade_domain::models::settings { use novade_core::types::NovaUuid; pub struct UserSettings { pub id: NovaUuid, ... } ... }`
* `pub mod novade_domain::services::settings_policy_service { use crate::models::settings::UserSettings; use crate::error::DomainResult; pub trait SettingsPolicyService { fn validate_settings(&self, settings: &UserSettings) -> DomainResult<()>; } ... }`
* `pub mod novade_domain::repositories::settings_repository { use novade_core::types::NovaUuid; use crate::models::settings::UserSettings; use crate::error::DomainResult; #[async_trait::async_trait] // Falls Repositories für async Implementierungen in novade-system gedacht sind pub trait UserSettingsRepository: Send + Sync { async fn get_by_user_id(&self, user_id: NovaUuid) -> DomainResult<Option<UserSettings>>; async fn save(&self, settings: &UserSettings) -> DomainResult<()>; } }`
* `pub mod novade_domain::events { use novade_core::types::NovaUuid; pub struct UserSettingsUpdatedEvent { pub user_id: NovaUuid, pub updated_fields: Vec<String>, } ... }`
* `pub mod novade_domain::error { use novade_core::error::NovaCoreError; #[derive(Debug, thiserror::Error)] pub enum DomainError { #[error("Validation failed: {0}")] ValidationError(String), #[error("Entity with ID {0} not found")] EntityNotFound(String), #[error("Core error: {0}")] CoreError(#[from] NovaCoreError), ... } pub type DomainResult<T> = Result<T, DomainError>; }`

---

#### 10. Abhängigkeitsdiagramm für `novade-domain` Module

```mermaid
graph TD
    subgraph novade_domain
        models_mod[models]
        services_mod[services]
        repositories_mod[repositories]
        events_mod[events]
        error_mod_domain[error]

        services_mod --> models_mod
        services_mod --> error_mod_domain
        repositories_mod --> models_mod
        repositories_mod --> error_mod_domain
        events_mod --> models_mod
        events_mod -- uses types like --> novade_core_types[novade_core::types]
        models_mod --> error_mod_domain %% For validation results
        models_mod -- uses --> novade_core_types
        error_mod_domain -- wraps --> novade_core_error[novade_core::error]
        services_mod -- uses --> novade_core_utils[novade_core::utils]
    end

    %% Externe Abhängigkeit zu novade_core (symbolisch dargestellt, da alle Module in domain darauf zugreifen können)
    novade_core_types
    novade_core_error
    novade_core_utils

    %% Höhere Schicht (System) implementiert Repositories
    novade_system[novade_system] -- implements --> repositories_mod
    novade_system -- consumes / uses --> services_mod
    novade_system -- consumes / produces --> events_mod
    novade_system -- consumes / uses --> models_mod
```

**Interpretation des Diagramms:**

* `models` ist zentral und wird von `services`, `repositories` und `events` genutzt.
* `services` kapselt die Kernlogik und nutzt `models` und `error`.
* `repositories` definiert Schnittstellen, die `models` verwenden.
* `events` definiert Ereignisstrukturen, die Daten aus `models` enthalten können.
* `error` ist der schichtspezifische Fehlertyp, der auch Fehler aus `novade_core` wrappen kann.
* Alle `novade_domain` Module können auf `novade_core` zugreifen.
* Die `novade_system` Schicht wird die Repository-Traits implementieren und die Domänenservices und -modelle nutzen.

---

#### 11. & 12. Modul-Roadmap, Arbeitspakete und Implementierungsreihenfolge für `novade-domain`

Die Implementierung der `novade_domain` Module SOLLTE iterativ und nach Abhängigkeiten erfolgen:

1.  **`novade_domain::error`:** (Priorität: 1 - Sehr hoch)
    * **Arbeitspaket:** Definition von `DomainError` und `DomainResult`.
    * **Begründung:** Basis für alle Fehlerbehandlung innerhalb der Schicht.
2.  **`novade_domain::models` (Kernmodelle zuerst):** (Priorität: 1 - Sehr hoch)
    * **Arbeitspakete (iterativ):**
        * Definition von `models::common` (gemeinsame Werteobjekte).
        * Definition von `models::settings` (z.B. `UserSettings`, `Color`).
        * Definition von `models::desktop` (z.B. `Workspace`).
        * Definition von `models::application` (z.B. `ApplicationProfile`).
        * Definition von `models::window` (z.B. `WindowIdentifier`).
        * Definition von `models::notification` (z.B. `NotificationSpec`).
        * Implementierung von Validierungslogik und Invarianten für jedes Modell.
    * **Begründung:** Modelle sind die Grundlage für Services und Repositories. Eine iterative Entwicklung, beginnend mit den wichtigsten Modellen, ist sinnvoll.
3.  **`novade_domain::repositories`:** (Priorität: 2 - Hoch)
    * **Arbeitspakete (parallel zu Modellen, sobald diese stabil sind):**
        * Definition von `UserSettingsRepository` Trait.
        * Definition weiterer Repository-Traits entsprechend den Modellen.
    * **Begründung:** Definiert die Datenzugriffsverträge frühzeitig.
4.  **`novade_domain::events`:** (Priorität: 2 - Hoch)
    * **Arbeitspaket:** Definition der primären Domänenereignisse.
    * **Begründung:** Wichtig für die Entkopplung und reaktive Systemkomponenten.
5.  **`novade_domain::services`:** (Priorität: 3 - Mittel)
    * **Arbeitspakete (iterativ, nachdem Modelle und ggf. Repository-Traits definiert sind):**
        * Implementierung des `SettingsPolicyService` (z.B. Validierung, Default-Logik).
        * Implementierung des `WindowManagementPolicyService` (grundlegende Layout-Regeln).
        * Implementierung weiterer Services nach Bedarf.
    * **Begründung:** Die Services implementieren die komplexere Logik und können erst entwickelt werden, wenn die zugrundeliegenden Modelle und Datenzugriffsabstraktionen klar sind.

Für jedes Modul und jede wesentliche Funktionalität SIND Unit-Tests ZWINGEND erforderlich. Die Domänenschicht MUSS eine sehr hohe Testabdeckung aufweisen, da sie die Kernlogik des Systems enthält. Mocking von Abhängigkeiten (insbesondere für das Testen von Services, die andere Services oder hypothetische Repository-Antworten nutzen) wird notwendig sein.

---

## 7. Zerlegung der Schicht `novade-system` in kohärente Module

Die Systemschicht stellt die Verbindung zwischen der Domänenlogik und dem Betriebssystem/der Hardware her. Sie MUSS in klar definierte Module zerlegt werden, um die Komplexität zu beherrschen und die Wartbarkeit zu erhöhen.

1.  **`novade-system/src/compositor`**: (Priorität: 1 - Kritisch)
    * **Verantwortlichkeit:** Implementiert den Wayland-Compositor, der für das Fenster-Management, die Darstellung, die Eingabeverarbeitung und die Implementierung des Wayland-Protokolls verantwortlich ist.
    * **Module:**
        * `core`: Die Kernlogik des Compositors (Wayland-Server-Initialisierung, Event-Loop, etc.).
        * `state`: Verwaltet den globalen Zustand des Compositors (Surfaces, Displays, etc.).
        * `handlers`: Behandelt Wayland-Events und andere Systemereignisse.
        * `protocols`: Implementiert Wayland-Protokolle (z.B. `wl_compositor`, `xdg_shell`).
        * `render`: Abstrahiert die Rendering-API (zunächst OpenGL, später Vulkan).
    * **Abhängigkeiten:** `novade-core`, Wayland-Bibliotheken, OpenGL/Vulkan-Bibliotheken, Kernel-APIs (evdev, DRM).
2.  **`novade-system/src/input`**: (Priorität: 1 - Kritisch)
    * **Verantwortlichkeit:** Verarbeitet Eingabeereignisse von Tastatur, Maus, Touchpad und anderen Eingabegeräten.
    * **Module:**
        * `libinput_handler`: Nutzt die `libinput`-Bibliothek zur Abstraktion von Eingabegeräten.
        * `keyboard`: Behandelt Tastatureingaben und Keymapping.
        * `pointer`: Behandelt Maus- und Touchpad-Eingaben.
        * `gesture`: Erkennt und verarbeitet Gesten.
    * **Abhängigkeiten:** `novade-core`, `libinput`, Wayland-Bibliotheken (für die Integration mit dem Compositor).
3.  **`novade-system/src/dbus_interfaces`**: (Priorität: 2 - Hoch)
    * **Verantwortlichkeit:** Definiert und implementiert D-Bus-Schnittstellen für die Kommunikation mit anderen Systemdiensten.
    * **Module:**
        * `clients`: Implementiert D-Bus-Clients für die Interaktion mit externen Diensten (z.B. NetworkManager, UPower).
        * `server`: Implementiert D-Bus-Server, um Dienste für andere Anwendungen bereitzustellen (z.B. Benachrichtigungen).
    * **Abhängigkeiten:** `novade-core`, `dbus-rs`.
4.  **`novade-system/src/audio_management`**: (Priorität: 3 - Mittel)
    * **Verantwortlichkeit:** Verwaltet die Audioausgabe und -eingabe.
    * **Module:**
        * `pipewire_client`: Integriert mit dem PipeWire-Server.
    * **Abhängigkeiten:** `novade-core`, PipeWire-Client-Bibliotheken.
5.  **`novade-system/src/mcp_client`**: (Priorität: 3 - Mittel)
    * **Verantwortlichkeit:** Implementiert den Client für das Model-Context-Protocol (MCP) zur Kommunikation mit KI-Modellen.
    * **Module:**
        * `client`: Die MCP-Client-Implementierung.
    * **Abhängigkeiten:** `novade-core`, Netzwerkbibliotheken.
6.  **`novade-system/src/window_mechanics`**: (Priorität: 2 - Hoch)
    * **Verantwortlichkeit:** Implementiert die Logik für Fensterplatzierung, Layout und Fokus.
    * **Module:**
        * `layout_engine`: Berechnet Fensterlayouts basierend auf Richtlinien der Domänenschicht.
        * `surface_mapper`: Verwaltet die Zuordnung zwischen Wayland-Surfaces und Anwendungsfenstern.
    * **Abhängigkeiten:** `novade-core`, `novade-domain` (Window Management Policies), `compositor`.
7.  **`novade-system/src/power_management`**: (Priorität: 3 - Mittel)
    * **Verantwortlichkeit:** Verwaltet die Energieeinstellungen und -zustände des Systems.
    * **Module:**
        * `service`: Implementiert den Power Management Service.
    * **Abhängigkeiten:** `novade-core`, D-Bus (für die Kommunikation mit `UPower` und `systemd-logind`).
8.  **`novade-system/src/portals`**: (Priorität: 3 - Mittel)
    * **Verantwortlichkeit:** Implementiert die Freedesktop-Portals für die sichere Interaktion mit Systemressourcen.
    * **Module:**
        * `desktop_portals`: Implementiert die Desktop-spezifischen Portale.
    * **Abhängigkeiten:** `novade-core`, D-Bus.

### 7.1. Modulverantwortlichkeiten und -grenzen

Jedes Modul MUSS eine klare und abgegrenzte Verantwortlichkeit haben. Die Grenzen zwischen den Modulen MÜSSEN explizit definiert sein, um unerwünschte Abhängigkeiten und Seiteneffekte zu vermeiden.

* **`compositor`**: Ist ausschließlich für das Fenster-Management und die Darstellung zuständig. Es DÜRFEN keine Geschäftslogik oder Anwendungsinteraktionslogik implementiert werden.
* **`input`**: Ist ausschließlich für die Verarbeitung von Eingabeereignissen zuständig. Es DÜRFEN keine Fenster-Management- oder Darstellungslogiken implementiert werden.
* **`dbus_interfaces`**: Ist ausschließlich für die D-Bus-Kommunikation zuständig. Es DÜRFEN keine anderen Systemfunktionen implementiert werden.
* **`audio_management`**: Ist ausschließlich für die Audioverarbeitung zuständig. Es DÜRFEN keine anderen Systemfunktionen implementiert werden.
* **`mcp_client`**: Ist ausschließlich für die MCP-Kommunikation zuständig. Es DÜRFEN keine anderen Systemfunktionen implementiert werden.
* **`window_mechanics`**: Ist für die Fensterplatzierung und das Layout zuständig. Es DÜRFEN keine Rendering- oder Eingabeverarbeitungslogiken implementiert werden.
* **`power_management`**: Ist ausschließlich für die Energieverwaltung zuständig. Es DÜRFEN keine anderen Systemfunktionen implementiert werden.
* **`portals`**: Ist ausschließlich für die Portal-Implementierung zuständig. Es DÜRFEN keine anderen Systemfunktionen implementiert werden.

### 7.2. Schnittstellen zwischen Modulen

Die Kommunikation zwischen den Modulen MUSS über klar definierte Schnittstellen erfolgen. Direkte Zugriffe auf interne Datenstrukturen anderer Module SIND VERBOTEN.

* **`compositor`** kommuniziert mit **`input`** über Callbacks, um Eingabeereignisse zu empfangen.
* **`compositor`** kommuniziert mit **`window_mechanics`** über Methodenaufrufe, um Fenster zu platzieren und zu verwalten.
* **`dbus_interfaces`** kommuniziert mit anderen Modulen über Methodenaufrufe, um D-Bus-Nachrichten zu senden und zu empfangen.
* **`audio_management`** kommuniziert mit anderen Modulen über Methodenaufrufe, um Audioausgabe und -eingabe zu steuern.
* **`mcp_client`** kommuniziert mit anderen Modulen über Methodenaufrufe, um MCP-Nachrichten zu senden und zu empfangen.
* **`window_mechanics`** kommuniziert mit **`compositor`** über Methodenaufrufe, um Fenster zu platzieren und zu verwalten.
* **`power_management`** kommuniziert mit anderen Modulen über Methodenaufrufe, um Energieeinstellungen zu steuern.
* **`portals`** kommuniziert mit anderen Modulen über Methodenaufrufe, um Portal-Anfragen zu bearbeiten.

### 7.3. Abhängigkeitsdiagramme

Ein Abhängigkeitsdiagramm stellt die Abhängigkeiten zwischen den Modulen visuell dar. Es MUSS die Richtung der Abhängigkeiten und die Art der Abhängigkeiten (z.B. statisch, dynamisch) zeigen.

```

graph LR

A[novade-core] --> B(compositor)

A --> C(input)

A --> D(dbus_interfaces)

A --> E(audio_management)

A --> F(mcp_client)

A --> G(window_mechanics)

A --> H(power_management)

A --> I(portals)

B --> C

B --> G

D --> H

G --> B

````

### 7.4. Modul-Roadmaps mit Arbeitspaketen

Jedes Modul MUSS eine detaillierte Roadmap mit atomaren Arbeitspaketen haben. Jedes Arbeitspaket MUSS klar definierte Eingabe- und Ausgabekriterien sowie Abhängigkeiten haben.

#### 7.4.1. `novade-system/src/compositor` Roadmap

* **Phase 1: Wayland-Compositor-Grundlagen**
    * Arbeitspaket 1.1: Initialisierung des Wayland-Servers.
        * Eingabe: Konfigurationsparameter.
        * Ausgabe: Laufender Wayland-Server.
        * Abhängigkeiten: `novade-core`.
    * Arbeitspaket 1.2: Implementierung des `wl_compositor`-Protokolls.
        * Eingabe: Wayland-Client-Verbindungen.
        * Ausgabe: Erstellung von Wayland-Surfaces.
        * Abhängigkeiten: Arbeitspaket 1.1.
    * Arbeitspaket 1.3: Implementierung des Event-Loops.
        * Eingabe: Wayland-Events, Systemereignisse.
        * Ausgabe: Verarbeitung von Ereignissen.
        * Abhängigkeiten: Arbeitspaket 1.2.
* **Phase 2: Eingabe- und Rendering-Integration**
    * Arbeitspaket 2.1: Integration mit `libinput`.
        * Eingabe: Eingabeereignisse von `libinput`.
        * Ausgabe: Wayland-Eingabeereignisse.
        * Abhängigkeiten: `novade-system/src/input`.
    * Arbeitspaket 2.2: OpenGL-Rendering-Backend.
        * Eingabe: Wayland-Surfaces.
        * Ausgabe: Darstellung von Oberflächen auf dem Bildschirm.
        * Abhängigkeiten: Arbeitspaket 1.3.
* **Phase 3: Fenster-Management und Protokolle**
    * Arbeitspaket 3.1: Implementierung des `xdg_shell`-Protokolls.
        * Eingabe: `xdg_shell`-Anfragen von Clients.
        * Ausgabe: Fenster-Management-Funktionen.
        * Abhängigkeiten: Arbeitspaket 2.2.
    * Arbeitspaket 3.2: Implementierung von Fenster-Layout-Logik.
        * Eingabe: Fensterplatzierungsanfragen.
        * Ausgabe: Fensterpositionierung.
        * Abhängigkeiten: `novade-system/src/window_mechanics`.
* **Phase 4: Erweiterte Funktionen und Optimierungen**
    * Arbeitspaket 4.1: Vulkan-Rendering-Backend (optional).
    * Arbeitspaket 4.2: Unterstützung für weitere Wayland-Protokolle.
    * Arbeitspaket 4.3: Performance-Optimierungen.

Die Roadmaps für die anderen Module werden in ähnlicher Weise detailliert.

### 7.5. Priorisierung der Implementierungsreihenfolge

Die Implementierung der Module MUSS in einer bestimmten Reihenfolge erfolgen, um Abhängigkeiten zu berücksichtigen und die Entwicklung zu beschleunigen.

1.  **`novade-core`** (MUSS vollständig implementiert sein, bevor andere Schichten beginnen)
2.  **`compositor`** (Kernkomponente, MUSS frühzeitig implementiert werden)
3.  **`input`** (Kernkomponente, MUSS frühzeitig implementiert werden)
4.  **`window_mechanics`** (hängt von `compositor` und `novade-domain` ab)
5.  **`dbus_interfaces`**
6.  **`audio_management`**
7.  **`mcp_client`**
8.  **`power_management`**
9.  **`portals`**

### 7.6. Definition der vollständigen Modul-API

Für jedes Modul MUSS eine vollständige API definiert werden. Die API MUSS alle öffentlichen Funktionen, Methoden, Klassen und Datenstrukturen mit ihren Parametern, Rückgabetypen und Fehlerbedingungen spezifizieren.

#### 7.6.1. `novade-system/src/compositor` API (Auszug)

```rust
pub mod core {
    pub struct Compositor {
        // ...
    }

    impl Compositor {
        pub fn new(config: CompositorConfig) -> Result<Self, CompositorError>;
        pub fn run(&mut self) -> Result<(), CompositorError>;
        // ...
    }
}

pub mod protocols {
    pub mod xdg_shell {
        pub struct XdgShellHandler {
            // ...
        }

        impl XdgShellHandler {
            pub fn new() -> Self;
            pub fn handle_xdg_surface(
                &mut self,
                surface: WlSurface,
                xdg_surface: XdgSurface,
            ) -> Result<(), XdgShellError>;
            // ...
        }
    }

    // ...
}
````

### 7.7. Spezifikation aller Datenstrukturen

Alle Datenstrukturen, die von den Modulen verwendet werden, MÜSSEN detailliert spezifiziert werden. Dies umfasst Felder, Typen, Sichtbarkeit, Mutabilität und Invarianten.

#### 7.7.1. `novade-system/src/compositor` Datenstrukturen (Auszug)

Rust

```
pub struct CompositorConfig {
    pub display_width: u32,
    pub display_height: u32,
    pub vsync: bool,
    // ...
}

pub struct SurfaceAttributes {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub buffer: Option<Buffer>,
    // ...
}
```

### 7.8. Beschreibung von Algorithmen und Geschäftslogik

Komplexe Algorithmen und Geschäftslogik innerhalb der Module MÜSSEN detailliert beschrieben werden. Dies kann in Pseudocode oder als detaillierte textuelle Beschreibung der Schritte erfolgen.

#### 7.8.1. `novade-system/src/window_mechanics/layout_engine` Algorithmus (Auszug)

Der Algorithmus für die Fensterplatzierung in einem Tiling-Layout könnte wie folgt aussehen:

1. Berechne den verfügbaren Platz auf dem Bildschirm.
2. Teile den verfügbaren Platz in Bereiche für jedes Fenster auf.
3. Platziere jedes Fenster in seinem zugewiesenen Bereich.
4. Berücksichtige dabei die Fenstergröße, die Benutzerpräferenzen und die Layout-Richtlinien.

### 7.9. Definition der Fehlerbehandlung und Ausnahmen

Jedes Modul MUSS eine klare Fehlerbehandlungsstrategie haben. Alle möglichen Fehlerfälle und Ausnahmen MÜSSEN definiert und behandelt werden.

#### 7.9.1. `novade-system/src/compositor` Fehlerbehandlung (Auszug)

Rust

```
pub enum CompositorError {
    #[error("Failed to initialize Wayland server: {0}")]
    WaylandInitError(String),
    #[error("Failed to create display: {0}")]
    DisplayCreateError(String),
    #[error("Failed to bind socket: {0}")]
    SocketBindError(String),
    // ...
}
```

### 7.10. Erstellung von Testspezifikationen

Für jedes Modul MÜSSEN Testspezifikationen erstellt werden. Diese Spezifikationen MÜSSEN die zu testenden Funktionen, die erwarteten Eingaben und Ausgaben sowie die zu überprüfenden Fehlerbedingungen beschreiben.

#### 7.10.1. `novade-system/src/compositor` Testspezifikationen (Auszug)

- **Testfall 1:** Initialisierung des Wayland-Servers.
    - Eingabe: Gültige Konfigurationsparameter.
    - Erwartete Ausgabe: Laufender Wayland-Server.
    - Zu überprüfende Fehlerbedingungen: Ungültige Konfigurationsparameter.
- **Testfall 2:** Erstellung einer Wayland-Surface.
    - Eingabe: Wayland-Client-Verbindung.
    - Erwartete Ausgabe: Erstellte Wayland-Surface.
    - Zu überprüfende Fehlerbedingungen: Ungültige Client-Verbindung.
- **Testfall 3:** Verarbeitung eines Eingabeereignisses.
    - Eingabe: Eingabeereignis von `libinput`.
    - Erwartete Ausgabe: Wayland-Eingabeereignis.
    - Zu überprüfende Fehlerbedingungen: Ungültiges Eingabeereignis.


#### 7.11.1. `novade-system/src/compositor` Performance-Anforderungen (Auszug)

- Die Compositor MUSS in der Lage sein, eine Bildwiederholrate von mindestens 60 Hz auf einem 4K-Display zu erreichen.
- Die Eingabelatenz MUSS unter 16 ms liegen.
- Die CPU-Auslastung des Compositors MUSS unter 10 % liegen, wenn keine Fensterbewegungen oder Animationen stattfinden.
- Der Speicherverbrauch des Compositors MUSS auf ein Minimum beschränkt werden.

### 7.12. Erstellung detaillierter Implementierungsvorlagen

Für jede Funktion und Methode MÜSSEN detaillierte Implementierungsvorlagen erstellt werden. Diese Vorlagen MÜSSEN die Signatur der Funktion, die Eingabeparameter, den Rückgabetyp, die Fehlerbehandlung, die zu verwendenden Algorithmen und Datenstrukturen sowie alle relevanten Performance- und Sicherheitsüberlegungen enthalten.

#### 7.12.1. `novade-system/src/compositor/core.rs` Implementierungsvorlage für `Compositor::new()`

Rust

```
pub fn new(config: CompositorConfig) -> Result<Self, CompositorError> {
    // 1. Initialisiere den Wayland-Server.
    // 2. Erstelle den Display.
    // 3. Binde den Wayland-Socket.
    // 4. Erstelle die erforderlichen Wayland-Globale.
    // 5. Initialisiere den Renderer.
    // 6. Erstelle den Event-Loop.
    // 7. Erstelle den Compositor-Zustand.
    // 8. Gib den Compositor zurück.
}
```

### 7.13. Spezifikation jeder Funktion bis auf Zeilen-Ebene

Komplexe Funktionen MÜSSEN bis auf Zeilen-Ebene spezifiziert werden. Dies umfasst die genaue Beschreibung jedes Schritts, jeder Bedingung und jeder Schleife.

#### 7.13.1. `novade-system/src/compositor/core.rs` Spezifikation von Schritt 1 in `Compositor::new()` (Initialisierung des Wayland-Servers)

1. Erstelle eine Instanz von `smithay::Display`.
2. Setze den Namen des Wayland-Sockets auf "nova-de".
3. Behandle Fehler beim Erstellen des Wayland-Servers.

### 7.14. Definition aller Edge-Cases und Fehlerbehandlung

Alle möglichen Edge-Cases und Fehlerbedingungen MÜSSEN definiert und behandelt werden. Für jeden Fehler MUSS eine klare Fehlerbehandlungsstrategie festgelegt werden.

#### 7.14.1. `novade-system/src/compositor/core.rs` Edge-Cases und Fehlerbehandlung für `Compositor::new()`

- **Edge-Case 1:** Ungültige Konfigurationsparameter.
    - Fehlerbehandlung: Gib einen `CompositorError::InvalidConfig` zurück.
- **Edge-Case 2:** Fehler beim Erstellen des Wayland-Servers.
    - Fehlerbehandlung: Gib einen `CompositorError::WaylandInitError` zurück.
- **Edge-Case 3:** Fehler beim Binden des Wayland-Sockets.
    - Fehlerbehandlung: Gib einen `CompositorError::SocketBindError` zurück.

### 7.15. Entwicklung von Optimierungsstrategien

Für jedes Modul MÜSSEN Optimierungsstrategien entwickelt werden, um die Performance zu maximieren und den Ressourcenverbrauch zu minimieren.

#### 7.15.1. `novade-system/src/compositor` Optimierungsstrategien

- Verwende Zero-Copy-Techniken, um Daten zwischen CPU und GPU zu übertragen.
- Minimiere die Anzahl der Speicherallokationen und -deallokationen.
- Verwende effiziente Datenstrukturen und Algorithmen.
- Nutze Hardware-Beschleunigung, wo immer möglich.
- Optimiere die Rendering-Pipeline für die Zielhardware.

### 7.16. Erstellung vollständiger Unit-Tests

Für jedes Modul MÜSSEN vollständige Unit-Tests erstellt werden. Diese Tests MÜSSEN alle Funktionen, Methoden und Klassen des Moduls abdecken.

#### 7.16.1. `novade-system/src/compositor/core.rs` Unit-Tests (Auszug)

Rust

```
#[test]
fn test_compositor_new() {
    // 1. Erstelle eine gültige CompositorConfig.
    // 2. Erstelle eine Instanz von Compositor.
    // 3. Überprüfe, ob die Instanz erfolgreich erstellt wurde.
    // 4. Überprüfe, ob der Wayland-Server läuft.
}

#[test]
fn test_compositor_new_invalid_config() {
    // 1. Erstelle eine ungültige CompositorConfig.
    // 2. Versuche, eine Instanz von Compositor zu erstellen.
    // 3. Überprüfe, ob ein CompositorError::InvalidConfig zurückgegeben wird.
}
```

### 7.17. Spezifikation von Logging und Monitoring

Für jedes Modul MÜSSEN Logging- und Monitoring-Strategien spezifiziert werden. Dies umfasst die zu protokollierenden Ereignisse, die zu verwendenden Log-Level und die zu erfassenden Metriken.

#### 7.17.1. `novade-system/src/compositor` Logging und Monitoring

- Protokolliere alle wichtigen Ereignisse, einschließlich Fehler, Warnungen und Informationsmeldungen.
- Verwende Log-Level, um die Wichtigkeit der Meldungen anzugeben.
- Erfasse Metriken wie CPU-Auslastung, Speicherverbrauch und Bildwiederholrate.
- Stelle Mechanismen zur Verfügung, um die Logs und Metriken zu überwachen und zu analysieren.

---

### 8. Zerlegung der Schicht `novade-ui` in kohärente Module

Die UI-Schicht implementiert die Benutzeroberfläche von NovaDE. Sie MUSS in klar definierte Module zerlegt werden, um die Komplexität zu beherrschen und die Wartbarkeit zu erhöhen.

1. **`novade-ui/src/app`**: (Priorität: 1 - Kritisch)
    - **Verantwortlichkeit:** Implementiert die Hauptanwendung und das Hauptfenster der NovaDE Shell.
    - **Module:**
        - `main_window`: Das Hauptfenster der Shell.
        - `app_state`: Verwaltet den globalen Zustand der Anwendung.
        - `event_handlers`: Behandelt Anwendungsereignisse.
    - **Abhängigkeiten:** Alle anderen UI-Module, `novade-core`, `novade-domain`, `novade-system`, GTK4.
2. **`novade-ui/src/components`**: (Priorität: 1 - Kritisch)
    - **Verantwortlichkeit:** Implementiert wiederverwendbare UI-Komponenten wie Panel, Anwendungsstarter, Task-Manager, etc.
    - **Module:**
        - `panel`: Das obere Panel der Shell.
        - `app_launcher`: Der Anwendungsstarter.
        - `task_bar`: Die Task-Leiste.
        - `system_tray`: Der System-Tray.
        - `workspace_switcher`: Der Workspace-Umschalter.
        - `notification_center`: Das Benachrichtigungscenter.
        - `control_center`: Das Kontrollzentrum.
        - `speed_dial`: Das Speed-Dial.
        - `command_palette`: Die Befehlspalette.
    - **Abhängigkeiten:** `novade-core`, `novade-domain`, `novade-system`, GTK4.
3. **`novade-ui/src/style`**: (Priorität: 2 - Hoch)
    - **Verantwortlichkeit:** Verwaltet das Styling und Theming der UI.
    - **Module:**
        - `theming_engine`: Die Theming-Engine.
        - `css_provider`: Der CSS-Provider.
    - **Abhängigkeiten:** `novade-core`, GTK4.
4. **`novade-ui/src/views`**: (Priorität: 2 - Hoch)
    - **Verantwortlichkeit:** Implementiert verschiedene Ansichten der Shell, z.B. den Overview-Modus.
    - **Module:**
        - `overview_mode`: Der Overview-Modus.
    - **Abhängigkeiten:** Alle anderen UI-Module, `novade-core`, `novade-domain`, `novade-system`, GTK4.
5. **`novade-ui/src/utils`**: (Priorität: 3 - Mittel)
    - **Verantwortlichkeit:** Implementiert Hilfsfunktionen und -klassen für die UI.
    - **Module:**
        - `gtk_helpers`: GTK-Hilfsfunktionen.
        - `icon_loader`: Der Icon-Loader.
    - **Abhängigkeiten:** `novade-core`, GTK4.

### 8.1. Modulverantwortlichkeiten und -grenzen

Jedes Modul MUSS eine klare und abgegrenzte Verantwortlichkeit haben. Die Grenzen zwischen den Modulen MÜSSEN explizit definiert sein, um unerwünschte Abhängigkeiten und Seiteneffekte zu vermeiden.

- **`app`**: Ist ausschließlich für die Hauptanwendung und das Hauptfenster zuständig. Es DÜRFEN keine UI-Komponenten oder Styling-Logiken implementiert werden.
- **`components`**: Ist ausschließlich für die Implementierung wiederverwendbarer UI-Komponenten zuständig. Es DÜRFEN keine Anwendungslogiken oder Styling-Logiken implementiert werden.
- **`style`**: Ist ausschließlich für das Styling und Theming der UI zuständig. Es DÜRFEN keine Anwendungslogiken oder UI-Komponenten implementiert werden.
- **`views`**: Ist ausschließlich für die Implementierung verschiedener Ansichten der Shell zuständig. Es DÜRFEN keine Anwendungslogiken oder UI-Komponenten implementiert werden.
- **`utils`**: Ist ausschließlich für die Implementierung von Hilfsfunktionen und -klassen zuständig. Es DÜRFEN keine Anwendungslogiken oder UI-Komponenten implementiert werden.

### 8.2. Schnittstellen zwischen Modulen

Die Kommunikation zwischen den Modulen MUSS über klar definierte Schnittstellen erfolgen. Direkte Zugriffe auf interne Datenstrukturen anderer Module SIND VERBOTEN.

- **`app`** kommuniziert mit **`components`** über Methodenaufrufe, um UI-Komponenten zu erstellen und zu verwalten.
- **`app`** kommuniziert mit **`views`** über Methodenaufrufe, um Ansichten anzuzeigen.
- **`components`** kommunizieren untereinander über Methodenaufrufe, um Daten auszutauschen und Aktionen auszulösen.
- **`components`** kommunizieren mit **`style`** über Methodenaufrufe, um das Styling anzuwenden.
- **`views`** kommunizieren mit **`components`** über Methodenaufrufe, um UI-Komponenten zu verwenden.
- **`utils`** wird von allen anderen Modulen über Funktionsaufrufe verwendet.

### 8.3. Abhängigkeitsdiagramme

Ein Abhängigkeitsdiagramm stellt die Abhängigkeiten zwischen den Modulen visuell dar. Es MUSS die Richtung der Abhängigkeiten und die Art der Abhängigkeiten (z.B. statisch, dynamisch) zeigen.

```
graph LR
A[novade-core] --> B(app)
A --> C(components)
A --> D(style)
A --> E(views)
A --> F(utils)
B --> C
B --> E
C --> D
E --> C
```

### 8.4. Modul-Roadmaps mit Arbeitspaketen

Jedes Modul MUSS eine detaillierte Roadmap mit atomaren Arbeitspaketen haben. Jedes Arbeitspaket MUSS klar definierte Eingabe- und Ausgabekriterien sowie Abhängigkeiten haben.

#### 8.4.1. `novade-ui/src/app` Roadmap

- **Phase 1: Hauptfenster und Anwendungszustand**
    - Arbeitspaket 1.1: Implementierung des Hauptfensters.
        - Eingabe: Konfigurationsparameter.
        - Ausgabe: Hauptfenster der Shell.
        - Abhängigkeiten: `novade-core`, GTK4.
    - Arbeitspaket 1.2: Implementierung des Anwendungszustands.
        - Eingabe: Anwendungsereignisse.
        - Ausgabe: Globaler Anwendungszustand.
        - Abhängigkeiten: Arbeitspaket 1.1.
    - Arbeitspaket 1.3: Implementierung von Ereignisbehandlern.
        - Eingabe: Anwendungsereignisse.
        - Ausgabe: Verarbeitung von Ereignissen.
        - Abhängigkeiten: Arbeitspaket 1.2.
- **Phase 2: Integration der UI-Komponenten**
    - Arbeitspaket 2.1: Integration des Panels.
        - Eingabe: Panel-Daten.
        - Ausgabe: Anzeige des Panels.
        - Abhängigkeiten: `novade-ui/src/components/panel`.
    - Arbeitspaket 2.2: Integration des Anwendungsstarters.
        - Eingabe: Anwendungsdaten.
        - Ausgabe: Anzeige des Anwendungsstarters.
        - Abhängigkeiten: `novade-ui/src/components/app_launcher`.
    - Arbeitspaket 2.3: Integration der Task-Leiste.
        - Eingabe: Fensterdaten.
        - Ausgabe: Anzeige der Task-Leiste.
        - Abhängigkeiten: `novade-ui/src/components/task_bar`.
    - ...
- **Phase 3: Integration der Ansichten**
    - Arbeitspaket 3.1: Integration des Overview-Modus.
        - Eingabe: Fensterdaten.
        - Ausgabe: Anzeige des Overview-Modus.
        - Abhängigkeiten: `novade-ui/src/views/overview_mode`.
    - ...

Die Roadmaps für die anderen Module werden in ähnlicher Weise detailliert.

### 8.5. Priorisierung der Implementierungsreihenfolge

Die Implementierung der Module MUSS in einer bestimmten Reihenfolge erfolgen, um Abhängigkeiten zu berücksichtigen und die Entwicklung zu beschleunigen.

1. **`novade-core`** (MUSS vollständig implementiert sein, bevor andere Schichten beginnen)
2. **`novade-domain`** (MUSS vollständig implementiert sein, bevor die UI-Schicht beginnt)
3. **`novade-system`** (MUSS teilweise implementiert sein, bevor die UI-Schicht beginnt)
4. **`app`** (Kernkomponente, MUSS frühzeitig implementiert werden)
5. **`components`** (Kernkomponente, MUSS frühzeitig implementiert werden)
6. **`style`**
7. **`views`**
8. **`utils`**

### 8.6. Definition der vollständigen Modul-API

Für jedes Modul MUSS eine vollständige API definiert werden. Die API MUSS alle öffentlichen Funktionen, Methoden, Klassen und Datenstrukturen mit ihren Parametern, Rückgabetypen und Fehlerbedingungen spezifizieren.

#### 8.6.1. `novade-ui/src/app/main_window.rs` API (Auszug)

Rust

```
pub struct MainWindow {
    // ...
}

impl MainWindow {
    pub fn new(app: &gtk::Application) -> Self;
    pub fn set_panel(&self, panel: Panel);
    pub fn set_app_launcher(&self, app_launcher: AppLauncher);
    // ...
}
```

### 8.7. Spezifikation aller Datenstrukturen

Alle Datenstrukturen, die von den Modulen verwendet werden, MÜSSEN detailliert spezifiziert werden. Dies umfasst Felder, Typen, Sichtbarkeit, Mutabilität und Invarianten.

#### 8.7.1. `novade-ui/src/components/panel.rs` Datenstrukturen (Auszug)

Rust

```
pub struct PanelConfig {
    pub height: i32,
    pub background_color: String,
    pub show_applications_button: bool,
    // ...
}
```

### 8.8. Beschreibung von Algorithmen und Geschäftslogik

Komplexe Algorithmen und Geschäftslogik innerhalb der Module MÜSSEN detailliert beschrieben werden. Dies kann in Pseudocode oder als detaillierte textuelle Beschreibung der Schritte erfolgen.

#### 8.8.1. `novade-ui/src/components/app_launcher/search_bar.rs` Algorithmus (Auszug)

Der Algorithmus für die Suche nach Anwendungen könnte wie folgt aussehen:

1. Empfange den Suchbegriff vom Benutzer.
2. Durchsuche die Liste der installierten Anwendungen nach Anwendungen, deren Name oder Beschreibung den Suchbegriff enthält.
3. Sortiere die Suchergebnisse nach Relevanz.
4. Zeige die Suchergebnisse an.

### 8.9. Definition der Fehlerbehandlung und Ausnahmen

Jedes Modul MUSS eine klare Fehlerbehandlungsstrategie haben. Alle möglichen Fehlerfälle und Ausnahmen MÜSSEN definiert und behandelt werden.

/src/components/panel.rs` Fehlerbehandlung (Auszug)

* **Fehlerfälle:**
    * Fehler beim Laden von Icons.
    * Fehler beim Abrufen von Systeminformationen.
    * Fehler beim Erstellen von GTK-Widgets.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

### 8.10. Definition von Ereignissen und Callbacks

Ereignisse und Callbacks, die von den Modulen ausgelöst werden, MÜSSEN detailliert definiert werden. Dies umfasst den Typ des Ereignisses, die übertragenen Daten und die Bedingungen, unter denen das Ereignis ausgelöst wird.

#### 8.10.1. `novade-ui/src/components/panel.rs` Ereignisse und Callbacks (Auszug)

* **Ereignisse:**
    * `PanelButtonClicked(button_id: String)`: Wird ausgelöst, wenn ein Button im Panel geklickt wird.
* **Callbacks:**
    * `PanelCallbacks`: Ein Trait, der von der `app`-Schicht implementiert wird, um auf Panel-Ereignisse zu reagieren.

### 8.11. Dokumentation von Designentscheidungen

Wichtige Designentscheidungen, Alternativen und Trade-offs MÜSSEN dokumentiert werden. Dies kann in Architecture Decision Records (ADRs) oder in Kommentaren im Code erfolgen.

#### 8.11.1. `novade-ui/src/app/main_window.rs` Designentscheidungen

* **Entscheidung:** Verwendung von GTK4 für die UI.
* **Alternativen:** Qt, Flutter, Electron.
* **Trade-offs:** GTK4 bietet native Look and Feel, gute Integration mit dem GNOME-Desktop, aber weniger plattformübergreifend als Qt oder Flutter. Electron wäre ressourcenintensiver.

### 8.12. Teststrategie

Für jedes Modul MUSS eine Teststrategie definiert werden. Dies umfasst die Art der Tests (Unit-Tests, Integrationstests, UI-Tests), die zu testenden Funktionalitäten und die zu verwendenden Testwerkzeuge.

#### 8.12.1. `novade-ui/src/components/panel.rs` Teststrategie

* **Unit-Tests:**
    * Testen der korrekten Erstellung der Panel-Elemente.
    * Testen der korrekten Verarbeitung von Konfigurationsparametern.
    * Mocken von Systeminformationen und Icon-Ladevorgängen.
* **UI-Tests:**
    * Manuelles Testen der Interaktion mit dem Panel.
    * Automatisierte Tests mit GTK-Testwerkzeugen (falls verfügbar).

---

### 9. Detaillierte Spezifikation der `novade-system` Crate

Die `novade-system` Crate ist für die Interaktion mit dem Betriebssystem, der Hardware und externen Diensten verantwortlich. Sie MUSS eine klare und wohldefinierte API haben, um von der `novade-ui` Crate verwendet werden zu können.

### 9.1. `novade-system/src/compositor` Modul

Das `compositor` Modul ist für die Verwaltung und Darstellung von Fenstern, die Eingabeverarbeitung und die Implementierung des Wayland-Protokolls verantwortlich. Es ist das Kernstück der NovaDE-Architektur.

#### 9.1.1. `novade-system/src/compositor/core.rs`

* **Verantwortlichkeit:** Implementiert die Kernlogik des Wayland-Compositors. Initialisiert den Wayland-Server, verwaltet den Display, erstellt die erforderlichen Wayland-Globale und handhabt den Event-Loop.
* **Abhängigkeiten:** `smithay`, `wayland-server`, `wayland-protocols`, `calloop`, `nix`, `log`.

#### 9.1.2. `novade-system/src/compositor/state.rs`

* **Verantwortlichkeit:** Verwaltet den Zustand des Compositors, einschließlich der Liste der Clients, der Fenster, der Eingabegeräte und der aktiven Oberflächen.
* **Abhängigkeiten:** `smithay`, `wayland-server`, `wayland-protocols`.

#### 9.1.3. `novade-system/src/compositor/handlers.rs`

* **Verantwortlichkeit:** Implementiert die Handler für Wayland-Ereignisse, einschließlich Client-Verbindungen, Surface-Erstellung, Eingabeereignisse und Protokollanfragen.
* **Abhängigkeiten:** `smithay`, `wayland-server`, `wayland-protocols`.

#### 9.1.4. `novade-system/src/compositor/protocols`

* **Verantwortlichkeit:** Implementiert die Unterstützung für verschiedene Wayland-Protokolle, einschließlich `xdg-shell`, `wlr-layer-shell` und anderer relevanter Erweiterungen.
* **Abhängigkeiten:** `smithay`, `wayland-server`, `wayland-protocols`.

#### 9.1.5. `novade-system/src/compositor/render`

* **Verantwortlichkeit:** Implementiert die Rendering-Logik des Compositors, einschließlich der Erstellung von OpenGL- oder Vulkan-Kontexten, der Verwaltung von Texturen und der Durchführung von Rendering-Operationen.
* **Abhängigkeiten:** `smithay`, `wayland-server`, `wayland-protocols`, `gl`, `vulkano` (optional).

### 9.2. `novade-system/src/input` Modul

Das `input` Modul ist für die Verarbeitung von Eingabeereignissen von Tastatur, Maus, Touchpad und anderen Eingabegeräten verantwortlich. Es MUSS eine abstrakte Schnittstelle bereitstellen, um verschiedene Eingabequellen zu unterstützen.

#### 9.2.1. `novade-system/src/input/libinput_handler.rs`

* **Verantwortlichkeit:** Implementiert die Eingabeverarbeitung mit der `libinput`-Bibliothek.
* **Abhängigkeiten:** `libinput`, `smithay`, `wayland-server`.

#### 9.2.2. `novade-system/src/input/keyboard.rs`

* **Verantwortlichkeit:** Verwaltet Tastatureingaben, einschließlich Tastendrücke, Tastenkombinationen und Tastaturlayouts.
* **Abhängigkeiten:** `libinput`, `smithay`, `wayland-server`, `xkbcommon`.

#### 9.2.3. `novade-system/src/input/pointer.rs`

* **Verantwortlichkeit:** Verwaltet Mauseingaben, einschließlich Mausbewegungen, Mausklicks und Mausradbewegungen.
* **Abhängigkeiten:** `libinput`, `smithay`, `wayland-server`.

#### 9.2.4. `novade-system/src/input/gesture.rs`

* **Verantwortlichkeit:** Verwaltet Touchpad-Gesten.
* **Abhängigkeiten:** `libinput`, `smithay`, `wayland-server`.

### 9.3. `novade-system/src/dbus_interfaces` Modul

Das `dbus_interfaces` Modul ist für die Kommunikation mit anderen Prozessen über den D-Bus-Bus verantwortlich. Es MUSS sowohl Client- als auch Server-Schnittstellen bereitstellen.

#### 9.3.1. `novade-system/src/dbus_interfaces/clients`

* **Verantwortlichkeit:** Implementiert D-Bus-Clients für die Kommunikation mit anderen Diensten, z.B. dem NetworkManager, dem PowerManager und dem Notification-Service.
* **Abhängigkeiten:** `zbus`.

#### 9.3.2. `novade-system/src/dbus_interfaces/server`

* **Verantwortlichkeit:** Implementiert D-Bus-Server, um Dienste für andere Prozesse bereitzustellen, z.B. den Notification-Service.
* **Abhängigkeiten:** `zbus`.

### 9.4. `novade-system/src/audio_management` Modul

Das `audio_management` Modul ist für die Verwaltung von Audio-Ein- und -Ausgabegeräten und die Steuerung der Audiowiedergabe verantwortlich. Es MUSS eine abstrakte Schnittstelle bereitstellen, um verschiedene Audio-Backends zu unterstützen.

#### 9.4.1. `novade-system/src/audio_management/pipewire_client.rs`

* **Verantwortlichkeit:** Implementiert die Audioverwaltung mit dem PipeWire-Server.
* **Abhängigkeiten:** `pipewire`.

### 9.5. `novade-system/src/mcp_client` Modul

Das `mcp_client` Modul ist für die Kommunikation mit dem MCP (Model Control Plane) verantwortlich. Es MUSS eine Schnittstelle bereitstellen, um Anfragen an das MCP zu senden und Antworten zu empfangen.

#### 9.5.1. `novade-system/src/mcp_client/client.rs`

* **Verantwortlichkeit:** Implementiert den MCP-Client.
* **Abhängigkeiten:** `tokio`.

### 9.6. `novade-system/src/window_mechanics` Modul

Das `window_mechanics` Modul ist für die Implementierung der Fensterverwaltungslogik verantwortlich, einschließlich des Layouts, der Positionierung und des Z-Orderings von Fenstern.

#### 9.6.1. `novade-system/src/window_mechanics/layout_engine.rs`

* **Verantwortlichkeit:** Implementiert die Logik für das Layout von Fenstern.
* **Abhängigkeiten:** `smithay`.

#### 9.6.2. `novade-system/src/window_mechanics/surface_mapper.rs`

* **Verantwortlichkeit:** Verwaltet die Zuordnung von Wayland-Oberflächen zu Fenstern.
* **Abhängigkeiten:** `smithay`, `wayland-server`.

### 9.7. `novade-system/src/power_management` Modul

Das `power_management` Modul ist für die Verwaltung von Energieeinstellungen und die Durchführung von Energieverwaltungsaktionen wie Suspend, Hibernate und Shutdown verantwortlich.

#### 9.7.1. `novade-system/src/power_management/service.rs`

* **Verantwortlichkeit:** Implementiert den Power-Management-Service.
* **Abhängigkeiten:** `zbus`, `logind`.

### 9.8. `novade-system/src/portals` Modul

Das `portals` Modul ist für die Implementierung der Schnittstellen zu den Desktop Portals verantwortlich, um Anwendungen den Zugriff auf sensible Ressourcen zu ermöglichen.

#### 9.8.1. `novade-system/src/portals/desktop_portals.rs`

* **Verantwortlichkeit:** Implementiert die Desktop Portal-Schnittstellen.
* **Abhängigkeiten:** `zbus`.

### 9.9. Detaillierte Modulspezifikationen

Für jedes Modul in der `novade-system` Crate MÜSSEN detaillierte Spezifikationen erstellt werden, die die folgenden Aspekte abdecken:

* Modulverantwortlichkeiten und -grenzen.
* Schnittstellen zu anderen Modulen.
* Datenstrukturen und Algorithmen.
* Fehlerbehandlung und Ausnahmen.
* Ereignisse und Callbacks.
* Designentscheidungen und Trade-offs.
* Teststrategie.

Diese Spezifikationen werden in den folgenden Abschnitten detailliert beschrieben.

---

### 10. `novade-system/src/compositor` Modul - Detaillierte Spezifikation

Das `novade-system/src/compositor` Modul ist das Herzstück der NovaDE-Architektur. Es ist für die Verwaltung und Darstellung von Fenstern, die Eingabeverarbeitung und die Implementierung des Wayland-Protokolls verantwortlich. Eine korrekte und effiziente Implementierung dieses Moduls ist entscheidend für die Performance und Stabilität des gesamten Systems.

### 10.1. `novade-system/src/compositor/core.rs` - Detaillierte Spezifikation

#### 10.1.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Initialisierung des Wayland-Servers und des Displays.
    * Erstellung und Verwaltung des Event-Loops.
    * Erstellung und Verwaltung des Compositor-Zustands.
    * Implementierung der Hauptlogik des Compositors.
* **Grenzen:**
    * Die Rendering-Logik ist im `render`-Modul implementiert.
    * Die Handler für Wayland-Ereignisse sind im `handlers`-Modul implementiert.
    * Die Unterstützung für verschiedene Wayland-Protokolle ist im `protocols`-Modul implementiert.

#### 10.1.2. Schnittstellen zu anderen Modulen

* **`render`**: Wird verwendet, um Rendering-Operationen durchzuführen.
* **`handlers`**: Wird verwendet, um Wayland-Ereignisse zu behandeln.
* **`protocols`**: Wird verwendet, um Wayland-Protokolle zu implementieren.
* **`state`**: Wird verwendet, um den Zustand des Compositors zu verwalten.

#### 10.1.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * `Compositor`: Die Hauptstruktur, die den Zustand des Compositors und den Event-Loop verwaltet.
    * `CompositorState`: Eine Struktur, die den Zustand des Compositors speichert, einschließlich der Liste der Clients, der Fenster und der Eingabegeräte.
* **Algorithmen:**
    * Der Event-Loop-Algorithmus, der Wayland-Ereignisse verarbeitet und die entsprechenden Handler aufruft.
    * Der Algorithmus zur Verwaltung des Compositor-Zustands, der Clients, Fenster und Eingabegeräte hinzufügt und entfernt.

#### 10.1.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Initialisieren des Wayland-Servers.
    * Fehler beim Erstellen des Displays.
    * Fehler beim Binden des Wayland-Sockets.
    * Fehler beim Erstellen des Event-Loops.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 10.1.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Wayland-Ereignisse, die von Clients gesendet werden, z.B. Surface-Erstellung, Eingabeereignisse und Protokollanfragen.
* **Callbacks:**
    * Handler-Funktionen, die für Wayland-Ereignisse aufgerufen werden.

#### 10.1.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung der `smithay`-Bibliothek für die Implementierung des Wayland-Compositors.
* **Alternativen:** Eigene Implementierung des Wayland-Protokolls.
* **Trade-offs:** `smithay` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 10.1.7. Teststrategie

* **Unit-Tests:**
    * Testen der Initialisierung des Wayland-Servers und des Displays.
    * Testen der Erstellung des Event-Loops.
    * Mocken von Wayland-Clients und -Ereignissen.
* **Integrationstests:**
    * Testen der Interaktion mit Wayland-Clients.
    * Testen der Funktionalität der verschiedenen Wayland-Protokolle.

### 10.2. `novade-system/src/compositor/state.rs` - Detaillierte Spezifikation

#### 10.2.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Verwaltung des Zustands des Compositors.
    * Speicherung der Liste der Clients, der Fenster, der Eingabegeräte und der aktiven Oberflächen.
    * Bereitstellung von Methoden zum Hinzufügen, Entfernen und Abrufen von Clients, Fenstern und Eingabegeräten.
* **Grenzen:**
    * Die Logik zur Verarbeitung von Wayland-Ereignissen ist im `handlers`-Modul implementiert.
    * Die Rendering-Logik ist im `render`-Modul implementiert.

#### 10.2.2. Schnittstellen zu anderen Modulen

* **`core`**: Wird verwendet, um auf den Compositor-Zustand zuzugreifen und ihn zu ändern.
* **`handlers`**: Wird verwendet, um den Compositor-Zustand zu aktualisieren, wenn Wayland-Ereignisse empfangen werden.
* **`render`**: Wird verwendet, um auf den Compositor-Zustand zuzugreifen, um Rendering-Operationen durchzuführen.

#### 10.2.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * `CompositorState`: Die Hauptstruktur, die den Zustand des Compositors speichert.
    * Listen oder Hashmaps zur Speicherung von Clients, Fenstern und Eingabegeräten.
* **Algorithmen:**
    * Algorithmen zum Hinzufügen, Entfernen und Abrufen von Clients, Fenstern und Eingabegeräten.
    * Algorithmen zum Verwalten der aktiven Oberflächen und des Eingabefokus.

#### 10.2.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Zugriff auf den Compositor-Zustand.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 10.2.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Keine spezifischen Ereignisse werden von diesem Modul ausgelöst.
* **Callbacks:**
    * Methoden, die vom `core`-Modul aufgerufen werden, um den Compositor-Zustand zu ändern.

#### 10.2.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung von Rust-Strukturen und Datenstrukturen zur Verwaltung des Compositor-Zustands.
* **Alternativen:** Verwendung von globalen Variablen oder anderen Ansätzen zur Zustandsverwaltung.
* **Trade-offs:** Rust-Strukturen bieten Typsicherheit und Speichersicherheit, können aber die Komplexität erhöhen.

#### 10.2.7. Teststrategie

* **Unit-Tests:**
    * Testen der Methoden zum Hinzufügen, Entfernen und Abrufen von Clients, Fenstern und Eingabegeräten.
    * Testen der Verwaltung der aktiven Oberflächen und des Eingabefokus.
* **Integrationstests:**
    * Testen der Interaktion mit dem `core`-Modul.

### 10.3. `novade-system/src/compositor/handlers.rs` - Detaillierte Spezifikation

#### 10.3.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Implementierung der Handler für Wayland-Ereignisse.
    * Verarbeitung von Client-Verbindungen, Surface-Erstellung, Eingabeereignissen und Protokollanfragen.
    * Aktualisierung des Compositor-Zustands als Reaktion auf Wayland-Ereignisse.
* **Grenzen:**
    * Die Rendering-Logik ist im `render`-Modul implementiert.
    * Die Unterstützung für verschiedene Wayland-Protokolle ist im `protocols`-Modul implementiert.

#### 10.3.2. Schnittstellen zu anderen Modulen

* **`core`**: Wird verwendet, um auf den Compositor-Zustand zuzugreifen und ihn zu ändern.
* **`state`**: Wird verwendet, um den Compositor-Zustand zu aktualisieren.
* **`protocols`**: Wird verwendet, um Wayland-Protokollanfragen zu behandeln.
* **`input`**: Wird verwendet, um Eingabeereignisse zu verarbeiten.

#### 10.3.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Handler-Funktionen für verschiedene Wayland-Ereignisse.
* **Algorithmen:**
    * Algorithmen zur Verarbeitung von Wayland-Ereignissen.
    * Algorithmen zur Aktualisierung des Compositor-Zustands als Reaktion auf Wayland-Ereignisse.

#### 10.3.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Verarbeiten von Wayland-Ereignissen.
    * Ungültige Wayland-Anfragen von Clients.
    * Protokollverletzungen.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 10.3.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Wayland-Ereignisse, die von Clients gesendet werden, z.B. Surface-Erstellung, Eingabeereignisse und Protokollanfragen.
* **Callbacks:**
    * Funktionen, die vom `core`-Modul aufgerufen werden, um Wayland-Ereignisse zu behandeln.

#### 10.3.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung von Handler-Funktionen zur Verarbeitung von Wayland-Ereignissen.
* **Alternativen:** Verwendung von objektorientierten Ansätzen oder anderen Paradigmen zur Ereignisverarbeitung.
* **Trade-offs:** Handler-Funktionen sind einfach zu implementieren, können aber die Komplexität erhöhen, wenn die Anzahl der Ereignisse groß ist.

#### 10.3.7. Teststrategie

* **Unit-Tests:**
    * Testen der Handler-Funktionen für verschiedene Wayland-Ereignisse.
    * Mocken von Wayland-Clients und -Ereignissen.
* **Integrationstests:**
    * Testen der Interaktion mit dem `core`-Modul, dem `state`-Modul und dem `protocols`-Modul.

### 10.4. `novade-system/src/compositor/protocols` - Detaillierte Spezifikation

#### 10.4.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Implementierung der Unterstützung für verschiedene Wayland-Protokolle.
    * Unterstützung für `xdg-shell`, `wlr-layer-shell` und andere relevante Erweiterungen.
    * Bereitstellung von Schnittstellen für Clients, um Protokollanfragen zu stellen.
* **Grenzen:**
    * Die Kernlogik des Compositors ist im `core`-Modul implementiert.
    * Die Handler für Wayland-Ereignisse sind im `handlers`-Modul implementiert.

#### 10.4.2. Schnittstellen zu anderen Modulen

* **`core`**: Wird verwendet, um auf den Compositor-Zustand zuzugreifen und ihn zu ändern.
* **`handlers`**: Wird verwendet, um Wayland-Protokollanfragen zu behandeln.

#### 10.4.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Strukturen zur Darstellung von Wayland-Protokollobjekten, z.B. `xdg_surface`, `layer_surface`.
* **Algorithmen:**
    * Algorithmen zur Implementierung der Logik der verschiedenen Wayland-Protokolle.
    * Algorithmen zum Verarbeiten von Protokollanfragen von Clients.

#### 10.4.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Ungültige Protokollanfragen von Clients.
    * Protokollverletzungen.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 10.4.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Wayland-Protokollanfragen von Clients.
* **Callbacks:**
    * Funktionen, die vom `handlers`-Modul aufgerufen werden, um Protokollanfragen zu behandeln.

#### 10.4.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Implementierung der Wayland-Protokolle mit der `smithay`-Bibliothek.
* **Alternativen:** Eigene Implementierung der Wayland-Protokolle.
* **Trade-offs:** `smithay` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 10.4.7. Teststrategie

* **Unit-Tests:**
    * Testen der Implementierung der verschiedenen Wayland-Protokolle.
    * Mocken von Wayland-Clients und -Protokollanfragen.
* **Integrationstests:**
    * Testen der Interaktion mit dem `core`-Modul und dem `handlers`-Modul.

### 10.5. `novade-system/src/compositor/render` - Detaillierte Spezifikation

#### 10.5.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Implementierung der Rendering-Logik des Compositors.
    * Erstellung von OpenGL- oder Vulkan-Kontexten.
    * Verwaltung von Texturen.
    * Durchführung von Rendering-Operationen.
* **Grenzen:**
    * Die Kernlogik des Compositors ist im `core`-Modul implementiert.
    * Die Handler für Wayland-Ereignisse sind im `handlers`-Modul implementiert.

#### 10.5.2. Schnittstellen zu anderen Modulen

* **`core`**: Wird verwendet, um auf den Compositor-Zustand zuzugreifen und die Rendering-Operationen zu steuern.

#### 10.5.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * OpenGL- oder Vulkan-Kontext.
    * Texturen und andere Rendering-Ressourcen.
    * Rendering-Pipelines und Shader.
* **Algorithmen:**
    * Algorithmen zur Durchführung von Rendering-Operationen, z.B. das Zeichnen von Fenstern und Oberflächen.
    * Algorithmen zur Verwaltung von Texturen und anderen Rendering-Ressourcen.

#### 10.5.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Erstellen von OpenGL- oder Vulkan-Kontexten.
    * Fehler beim Laden von Texturen.
    * Fehler beim Ausführen von Rendering-Operationen.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 10.5.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Keine spezifischen Ereignisse werden von diesem Modul ausgelöst.
* **Callbacks:**
    * Funktionen, die vom `core`-Modul aufgerufen werden, um Rendering-Operationen durchzuführen.

#### 10.5.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung von OpenGL oder Vulkan für das Rendering.
* **Alternativen:** Andere Rendering-APIs, z.B. Direct3D.
* **Trade-offs:** OpenGL ist einfacher zu verwenden, aber Vulkan bietet mehr Kontrolle und potenziell bessere Performance.

#### 10.5.7. Teststrategie

* **Unit-Tests:**
    * Testen der Erstellung von OpenGL- oder Vulkan-Kontexten.
    * Testen des Ladens von Texturen.
    * Mocken von Rendering-Operationen.
* **Integrationstests:**
    * Testen der Interaktion mit dem `core`-Modul.
    * Visuelle Tests, um die korrekte Darstellung der Benutzeroberfläche sicherzustellen.

---

### 11. `novade-system/src/input` Modul - Detaillierte Spezifikation

Das `novade-system/src/input` Modul ist für die Verarbeitung von Eingabeereignissen von Tastatur, Maus, Touchpad und anderen Eingabegeräten verantwortlich. Es MUSS eine abstrakte Schnittstelle bereitstellen, um verschiedene Eingabequellen zu unterstützen.

### 11.1. `novade-system/src/input/libinput_handler.rs` - Detaillierte Spezifikation

#### 11.1.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Implementierung der Eingabeverarbeitung mit der `libinput`-Bibliothek.
    * Abstraktion der Eingabegeräte und Bereitstellung einer einheitlichen Schnittstelle für die Verarbeitung von Eingabeereignissen.
* **Grenzen:**
    * Die Logik zur Verwaltung von Tastatureingaben ist im `keyboard`-Modul implementiert.
    * Die Logik zur Verwaltung von Mauseingaben ist im `pointer`-Modul implementiert.
    * Die Logik zur Verwaltung von Touchpad-Gesten ist im `gesture`-Modul implementiert.

#### 11.1.2. Schnittstellen zu anderen Modulen

* **`keyboard`**: Wird verwendet, um Tastatureingaben zu verarbeiten.
* **`pointer`**: Wird verwendet, um Mauseingaben zu verarbeiten.
* **`gesture`**: Wird verwendet, um Touchpad-Gesten zu verarbeiten.
* **`compositor`**: Wird verwendet, um Eingabeereignisse an den Compositor weiterzuleiten.

#### 11.1.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * `LibInputHandler`: Die Hauptstruktur, die die `libinput`-Bibliothek verwaltet und Eingabeereignisse verarbeitet.
* **Algorithmen:**
    * Algorithmen zur Verarbeitung von `libinput`-Ereignissen.
    * Algorithmen zur Umwandlung von `libinput`-Ereignissen in abstrakte Eingabeereignisse.

#### 11.1.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler bei der Initialisierung der `libinput`-Bibliothek.
    * Fehler beim Verarbeiten von `libinput`-Ereignissen.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 11.1.5. Ereignisse und Callbacks

* **Ereignisse:**
    * `libinput`-Ereignisse, die von Eingabegeräten empfangen werden.
* **Callbacks:**
    * Funktionen, die vom `keyboard`-, `pointer`- und `gesture`-Modul aufgerufen werden, um Eingabeereignisse zu verarbeiten.

#### 11.1.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung der `libinput`-Bibliothek für die Eingabeverarbeitung.
* **Alternativen:** Eigene Implementierung der Eingabeverarbeitung.
* **Trade-offs:** `libinput` bietet eine hohe Abstraktionsebene und unterstützt viele Eingabegeräte, kann aber die Flexibilität einschränken.

#### 11.1.7. Teststrategie

* **Unit-Tests:**
    * Testen der Initialisierung der `libinput`-Bibliothek.
    * Testen der Verarbeitung von `libinput`-Ereignissen.
    * Mocken von `libinput`-Ereignissen.
* **Integrationstests:**
    * Testen der Interaktion mit dem `keyboard`-, `pointer`-, `gesture`- und `compositor`-Modul.

### 11.2. `novade-system/src/input/keyboard.rs` - Detaillierte Spezifikation

#### 11.2.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Verwaltung von Tastatureingaben.
    * Verarbeitung von Tastendrücken, Tastenkombinationen und Tastaturlayouts.
* **Grenzen:**
    * Die Verarbeitung von `libinput`-Ereignissen ist im `libinput_handler`-Modul implementiert.

#### 11.2.2. Schnittstellen zu anderen Modulen

* **`libinput_handler`**: Wird verwendet, um Tastatureingabeereignisse zu empfangen.
* **`compositor`**: Wird verwendet, um Tastatureingabeereignisse an den Compositor weiterzuleiten.

#### 11.2.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Datenstrukturen zur Darstellung von Tastendrücken und Tastenkombinationen.
    * Datenstrukturen zur Verwaltung von Tastaturlayouts.
* **Algorithmen:**
    * Algorithmen zur Verarbeitung von Tastendrücken und Tastenkombinationen.
    * Algorithmen zur Verwaltung von Tastaturlayouts.

#### 11.2.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Verarbeiten von Tastatureingabeereignissen.
    * Ungültige Tastenkombinationen.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 11.2.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Tastatureingabeereignisse, die vom `libinput_handler`-Modul empfangen werden.
* **Callbacks:**
    * Funktionen, die vom `compositor`-Modul aufgerufen werden, um Tastatureingabeereignisse zu verarbeiten.

#### 11.2.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung der `xkbcommon`-Bibliothek zur Verwaltung von Tastaturlayouts.
* **Alternativen:** Eigene Implementierung der Tastaturlayoutverwaltung.
* **Trade-offs:** `xkbcommon` bietet eine hohe Abstraktionsebene und unterstützt viele Tastaturlayouts, kann aber die Komplexität erhöhen.

#### 11.2.7. Teststrategie

* **Unit-Tests:**
    * Testen der Verarbeitung von Tastendrücken und Tastenkombinationen.
    * Testen der Verwaltung von Tastaturlayouts.
    * Mocken von Tastatureingabeereignissen.
* **Integrationstests:**
    * Testen der Interaktion mit dem `libinput_handler`-Modul und dem `compositor`-Modul.
      
### 11.3. `novade-system/src/input/pointer.rs` - Detaillierte Spezifikation

#### 11.3.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung von Mauseingaben.
    - Verarbeitung von Mausbewegungen, Mausklicks und Mausradereignissen.
- **Grenzen:**
    - Die Verarbeitung von `libinput`-Ereignissen ist im `libinput_handler`-Modul implementiert.

#### 11.3.2. Schnittstellen zu anderen Modulen

- **`libinput_handler`**: Wird verwendet, um Mauseingabeereignisse zu empfangen.
- **`compositor`**: Wird verwendet, um Mauseingabeereignisse an den Compositor weiterzuleiten.

#### 11.3.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Datenstrukturen zur Darstellung von Mausbewegungen, Mausklicks und Mausradereignissen.
- **Algorithmen:**
    - Algorithmen zur Verarbeitung von Mausbewegungen, Mausklicks und Mausradereignissen.

#### 11.3.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler beim Verarbeiten von Mauseingabeereignissen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 11.3.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Mauseingabeereignisse, die vom `libinput_handler`-Modul empfangen werden.
- **Callbacks:**
    - Funktionen, die vom `compositor`-Modul aufgerufen werden, um Mauseingabeereignisse zu verarbeiten.

#### 11.3.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung der `libinput`-Bibliothek für die Eingabeverarbeitung.
- **Alternativen:** Eigene Implementierung der Mauseingabeverarbeitung.
- **Trade-offs:** `libinput` bietet eine hohe Abstraktionsebene und unterstützt viele Mäuse, kann aber die Flexibilität einschränken.

#### 11.3.7. Teststrategie

- **Unit-Tests:**
    - Testen der Verarbeitung von Mausbewegungen, Mausklicks und Mausradereignissen.
    - Mocken von Mauseingabeereignissen.
- **Integrationstests:**
    - Testen der Interaktion mit dem `libinput_handler`-Modul und dem `compositor`-Modul.

### 11.4. `novade-system/src/input/gesture.rs` - Detaillierte Spezifikation

#### 11.4.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung von Touchpad-Gesten.
    - Erkennung und Verarbeitung von Gesten wie Scrollen, Zoomen, Drehen und Wischen.
- **Grenzen:**
    - Die Verarbeitung von `libinput`-Ereignissen ist im `libinput_handler`-Modul implementiert.

#### 11.4.2. Schnittstellen zu anderen Modulen

- **`libinput_handler`**: Wird verwendet, um Touchpad-Ereignisse zu empfangen.
- **`compositor`**: Wird verwendet, um Touchpad-Gesten an den Compositor weiterzuleiten.

#### 11.4.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Datenstrukturen zur Darstellung von Touchpad-Gesten.
- **Algorithmen:**
    - Algorithmen zur Erkennung und Verarbeitung von Gesten wie Scrollen, Zoomen, Drehen und Wischen.

#### 11.4.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler beim Verarbeiten von Touchpad-Ereignissen.
    - Nicht erkannte oder ungültige Gesten.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 11.4.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Touchpad-Ereignisse, die vom `libinput_handler`-Modul empfangen werden.
- **Callbacks:**
    - Funktionen, die vom `compositor`-Modul aufgerufen werden, um Touchpad-Gesten zu verarbeiten.

#### 11.4.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung der `libinput`-Bibliothek für die Gestenerkennung.
- **Alternativen:** Eigene Implementierung der Gestenerkennung.
- **Trade-offs:** `libinput` bietet eine hohe Abstraktionsebene und unterstützt viele Touchpads, kann aber die Flexibilität einschränken.

#### 11.4.7. Teststrategie

- **Unit-Tests:**
    - Testen der Erkennung und Verarbeitung von Gesten wie Scrollen, Zoomen, Drehen und Wischen.
    - Mocken von Touchpad-Ereignissen.
- **Integrationstests:**
    - Testen der Interaktion mit dem `libinput_handler`-Modul und dem `compositor`-Modul.

---

### 12. `novade-system/src/dbus_interfaces` Modul - Detaillierte Spezifikation

Das `novade-system/src/dbus_interfaces` Modul ist für die Kommunikation mit anderen Anwendungen und Diensten über den D-Bus Inter-Process Communication (IPC) Mechanismus verantwortlich. Es MUSS sowohl Client- als auch Server-Schnittstellen bereitstellen.

### 12.1. `novade-system/src/dbus_interfaces/clients` - Detaillierte Spezifikation

#### 12.1.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung von D-Bus Clients für die Kommunikation mit externen Diensten, z.B. NetworkManager, UPower, Logind und SecretService.
    - Bereitstellung von Schnittstellen zur Interaktion mit diesen Diensten.
- **Grenzen:**
    - Die Implementierung des D-Bus Servers für NovaDE-Dienste ist im `server`-Modul implementiert.

#### 12.1.2. Schnittstellen zu anderen Modulen

- **Keine direkten Abhängigkeiten von anderen Modulen innerhalb von `novade-system`.**
- **Abhängigkeiten von externen D-Bus-Diensten.**

#### 12.1.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Daten, die über D-Bus ausgetauscht werden.
- **Algorithmen:**
    - Algorithmen zur Serialisierung und Deserialisierung von Daten für die D-Bus-Kommunikation.
    - Algorithmen zur Fehlerbehandlung bei D-Bus-Kommunikation.

#### 12.1.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindungsherstellung mit D-Bus-Diensten.
    - Fehler beim Senden und Empfangen von D-Bus-Nachrichten.
    - Timeouts bei D-Bus-Anfragen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 12.1.5. Ereignisse und Callbacks

- **Ereignisse:**
    - D-Bus-Signale, die von externen Diensten empfangen werden.
- **Callbacks:**
    - Funktionen, die auf D-Bus-Signale reagieren.

#### 12.1.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung der `zbus`-Bibliothek für die D-Bus-Kommunikation.
- **Alternativen:** Andere D-Bus-Bibliotheken oder eigene Implementierung.
- **Trade-offs:** `zbus` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 12.1.7. Teststrategie

- **Unit-Tests:**
    - Mocken von D-Bus-Diensten.
    - Testen der Serialisierung und Deserialisierung von Daten.
    - Testen der Fehlerbehandlung.
- **Integrationstests:**
    - Testen der Interaktion mit realen D-Bus-Diensten.

### 12.2. `novade-system/src/dbus_interfaces/clients/network_manager.rs` - Detaillierte Spezifikation

#### 12.2.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung des D-Bus Clients für den NetworkManager-Dienst.
    - Bereitstellung von Schnittstellen zur Abfrage und Steuerung von Netzwerkverbindungen.
- **Grenzen:**
    - Die Implementierung anderer D-Bus Clients ist in anderen Modulen implementiert.

#### 12.2.2. Schnittstellen zu anderen Modulen

- **Keine direkten Abhängigkeiten von anderen Modulen innerhalb von `novade-system`.**
- **Abhängigkeit vom NetworkManager-D-Bus-Dienst.**

#### 12.2.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Netzwerkverbindungen, Geräten und Access Points.
- **Algorithmen:**
    - Algorithmen zur Abfrage von Netzwerkstatusinformationen.
    - Algorithmen zur Steuerung von Netzwerkverbindungen.

#### 12.2.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindungsherstellung mit dem NetworkManager-Dienst.
    - Fehler beim Senden und Empfangen von D-Bus-Nachrichten.
    - Timeouts bei D-Bus-Anfragen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 12.2.5. Ereignisse und Callbacks

- **Ereignisse:**
    - D-Bus-Signale, die vom NetworkManager-Dienst empfangen werden, z.B. Änderungen des Netzwerkstatus.
- **Callbacks:**
    - Funktionen, die auf NetworkManager-Signale reagieren.

#### 12.2.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung der `zbus`-Bibliothek für die D-Bus-Kommunikation.
- **Alternativen:** Andere D-Bus-Bibliotheken oder eigene Implementierung.
- **Trade-offs:** `zbus` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 12.2.7. Teststrategie

- **Unit-Tests:**
    - Mocken des NetworkManager-D-Bus-Dienstes.
    - Testen der Serialisierung und Deserialisierung von Daten.
    - Testen der Fehlerbehandlung.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen NetworkManager-Dienst.

### 12.3. `novade-system/src/dbus_interfaces/clients/upower.rs` - Detaillierte Spezifikation

#### 12.3.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung des D-Bus Clients für den UPower-Dienst.
    - Bereitstellung von Schnittstellen zur Abfrage von Informationen zum Energiestatus des Systems.
- **Grenzen:**
    - Die Implementierung anderer D-Bus Clients ist in anderen Modulen implementiert.

#### 12.3.2. Schnittstellen zu anderen Modulen

- **Keine direkten Abhängigkeiten von anderen Modulen innerhalb von `novade-system`.**
- **Abhängigkeit vom UPower-D-Bus-Dienst.**

#### 12.3.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Informationen zum Energiestatus, z.B. Batteriestatus, Energiequellen.
- **Algorithmen:**
    - Algorithmen zur Abfrage von Informationen zum Energiestatus.

#### 12.3.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindungsherstellung mit dem UPower-Dienst.
    - Fehler beim Senden und Empfangen von D-Bus-Nachrichten.
    - Timeouts bei D-Bus-Anfragen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 12.3.5. Ereignisse und Callbacks

- **Ereignisse:**
    - D-Bus-Signale, die vom UPower-Dienst empfangen werden, z.B. Änderungen des Batteriestatus.
- **Callbacks:**
    - Funktionen, die auf UPower-Signale reagieren.

#### 12.3.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung der `zbus`-Bibliothek für die D-Bus-Kommunikation.
- **Alternativen:** Andere D-Bus-Bibliotheken oder eigene Implementierung.
- **Trade-offs:** `zbus` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 12.3.7. Teststrategie

- **Unit-Tests:**
    - Mocken des UPower-D-Bus-Dienstes.
    - Testen der Serialisierung und Deserialisierung von Daten.
    - Testen der Fehlerbehandlung.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen UPower-Dienst.

### 12.4. `novade-system/src/dbus_interfaces/clients/logind.rs` - Detaillierte Spezifikation

#### 12.4.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung des D-Bus Clients für den Logind-Dienst.
    - Bereitstellung von Schnittstellen zur Steuerung der Benutzersitzung, z.B. Sperren, Abmelden, Herunterfahren.
- **Grenzen:**
    - Die Implementierung anderer D-Bus Clients ist in anderen Modulen implementiert.

#### 12.4.2. Schnittstellen zu anderen Modulen

- **Keine direkten Abhängigkeiten von anderen Modulen innerhalb von `novade-system`.**
- **Abhängigkeit vom Logind-D-Bus-Dienst.**

#### 12.4.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Benutzerinformationen und Sitzungsdaten.
- **Algorithmen:**
    - Algorithmen zur Steuerung der Benutzersitzung.

#### 12.4.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindungsherstellung mit dem Logind-Dienst.
    - Fehler beim Senden und Empfangen von D-Bus-Nachrichten.
    - Timeouts bei D-Bus-Anfragen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 12.4.5. Ereignisse und Callbacks

- **Ereignisse:**
    - D-Bus-Signale, die vom Logind-Dienst empfangen werden, z.B. Änderungen des Sitzungsstatus.
- **Callbacks:**
    - Funktionen, die auf Logind-Signale reagieren.

#### 12.4.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung der `zbus`-Bibliothek für die D-Bus-Kommunikation.
- **Alternativen:** Andere D-Bus-Bibliotheken oder eigene Implementierung.
- **Trade-offs:** `zbus` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 12.4.7. Teststrategie

- **Unit-Tests:**
    - Mocken des Logind-D-Bus-Dienstes.
    - Testen der Serialisierung und Deserialisierung von Daten.
    - Testen der Fehlerbehandlung.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen Logind-Dienst.

### 12.5. `novade-system/src/dbus_interfaces/clients/secret_service.rs` - Detaillierte Spezifikation

#### 12.5.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung des D-Bus Clients für den SecretService-Dienst.
    - Bereitstellung von Schnittstellen zum Speichern und Abrufen von Passwörtern und anderen sensiblen Daten.
- **Grenzen:**
    - Die Implementierung anderer D-Bus Clients ist in anderen Modulen implementiert.

#### 12.5.2. Schnittstellen zu anderen Modulen

- Keine direkten Abhängigkeiten von anderen Modulen innerhalb von `novade-system`.
- Abhängigkeit vom SecretService-D-Bus-Dienst.

#### 12.5.3. Datenstrukturen und Algorithmen

- Datenstrukturen:
    - Strukturen zur Darstellung von Passwörtern, Sammlungen und Attributen.
- Algorithmen:
    - Algorithmen zum Speichern und Abrufen von Passwörtern.
    - Algorithmen zur Verschlüsselung und Entschlüsselung von Passwörtern.

#### 12.5.4. Fehlerbehandlung und Ausnahmen

- Fehlerfälle:
    - Fehler bei der Verbindungsherstellung mit dem SecretService-Dienst.
    - Fehler beim Senden und Empfangen von D-Bus-Nachrichten.
    - Timeouts bei D-Bus-Anfragen.
    - Fehler beim Speichern oder Abrufen von Passwörtern.
    - Fehler bei der Verschlüsselung oder Entschlüsselung von Passwörtern.
- Ausnahmen:
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 12.5.5. Ereignisse und Callbacks

- Ereignisse:
    - D-Bus-Signale, die vom SecretService-Dienst empfangen werden, z.B. Änderungen an Passwortsammlungen.
- Callbacks:
    - Funktionen, die auf SecretService-Signale reagieren.

#### 12.5.6. Designentscheidungen und Trade-offs

- Entscheidung: Verwendung der `zbus`-Bibliothek für die D-Bus-Kommunikation.
- Alternativen: Andere D-Bus-Bibliotheken oder eigene Implementierung.
- Trade-offs: `zbus` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 12.5.7. Teststrategie

- Unit-Tests:
    - Mocken des SecretService-D-Bus-Dienstes.
    - Testen der Serialisierung und Deserialisierung von Daten.
    - Testen der Fehlerbehandlung.
    - Testen der Verschlüsselung und Entschlüsselung von Passwörtern.
- Integrationstests:
    - Testen der Interaktion mit dem realen SecretService-Dienst.

### 12.6. `novade-system/src/dbus_interfaces/server` - Detaillierte Spezifikation

#### 12.6.1. Modulverantwortlichkeiten und -grenzen

- Verantwortlichkeit:
    - Implementierung von D-Bus Servern für NovaDE-Dienste, die für andere Anwendungen und Dienste zugänglich gemacht werden sollen.
    - Bereitstellung von Schnittstellen zur Interaktion mit NovaDE-Diensten.
- Grenzen:
    - Die Implementierung von D-Bus Clients für die Kommunikation mit externen Diensten ist im `clients`-Modul implementiert.

#### 12.6.2. Schnittstellen zu anderen Modulen

- Abhängigkeiten von anderen Modulen innerhalb von `novade-system`, die D-Bus-Schnittstellen bereitstellen sollen.
- Keine direkten Abhängigkeiten von externen D-Bus-Diensten.

#### 12.6.3. Datenstrukturen und Algorithmen

- Datenstrukturen:
    - Strukturen zur Darstellung von Daten, die über D-Bus ausgetauscht werden.
- Algorithmen:
    - Algorithmen zur Serialisierung und Deserialisierung von Daten für die D-Bus-Kommunikation.
    - Algorithmen zur Fehlerbehandlung bei D-Bus-Kommunikation.
    - Algorithmen zur Autorisierung von D-Bus-Anfragen.

#### 12.6.4. Fehlerbehandlung und Ausnahmen

- Fehlerfälle:
    - Fehler bei der Registrierung von D-Bus-Diensten.
    - Fehler beim Senden und Empfangen von D-Bus-Nachrichten.
    - Fehler bei der Serialisierung oder Deserialisierung von Daten.
    - Nicht autorisierte D-Bus-Anfragen.
- Ausnahmen:
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 12.6.5. Ereignisse und Callbacks

- Ereignisse:
    - D-Bus-Signale, die von NovaDE-Diensten gesendet werden.
- Callbacks:
    - Funktionen, die auf D-Bus-Anfragen von anderen Anwendungen und Diensten reagieren.

#### 12.6.6. Designentscheidungen und Trade-offs

- Entscheidung: Verwendung der `zbus`-Bibliothek für die D-Bus-Kommunikation.
- Alternativen: Andere D-Bus-Bibliotheken oder eigene Implementierung.
- Trade-offs: `zbus` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 12.6.7. Teststrategie

- Unit-Tests:
    - Mocken von D-Bus-Clients.
    - Testen der Serialisierung und Deserialisierung von Daten.
    - Testen der Fehlerbehandlung.
    - Testen der Autorisierung von D-Bus-Anfragen.
- Integrationstests:
    - Testen der Interaktion mit realen D-Bus-Clients.

### 12.7. `novade-system/src/dbus_interfaces/server/notifications.rs` - Detaillierte Spezifikation

#### 12.7.1. Modulverantwortlichkeiten und -grenzen

- Verantwortlichkeit:
    - Implementierung des D-Bus Servers für den NovaDE Notification Service.
    - Bereitstellung von Schnittstellen zum Senden und Verwalten von Desktop-Benachrichtigungen.
- Grenzen:
    - Die Implementierung anderer D-Bus Server ist in anderen Modulen implementiert.

#### 12.7.2. Schnittstellen zu anderen Modulen

- Abhängigkeit vom `novade-system/src/notification` Modul.
- Keine direkten Abhängigkeiten von externen D-Bus-Diensten.

#### 12.7.3. Datenstrukturen und Algorithmen

- Datenstrukturen:
    - Strukturen zur Darstellung von Benachrichtigungen, Aktionen und Hinweisen.
- Algorithmen:
    - Algorithmen zur Verwaltung von Benachrichtigungen.
    - Algorithmen zur Serialisierung und Deserialisierung von Benachrichtigungen für die D-Bus-Kommunikation.

#### 12.7.4. Fehlerbehandlung und Ausnahmen

- Fehlerfälle:
    - Fehler bei der Registrierung des Notification-Dienstes.
    - Fehler beim Senden und Empfangen von D-Bus-Nachrichten.
    - Fehler bei der Serialisierung oder Deserialisierung von Benachrichtigungen.
    - Ungültige Benachrichtigungsdaten.
- Ausnahmen:
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 12.7.5. Ereignisse und Callbacks

- Ereignisse:
    - D-Bus-Signale, die vom Notification-Dienst gesendet werden, z.B. wenn eine Benachrichtigung angezeigt oder geschlossen wird.
- Callbacks:
    - Funktionen, die auf D-Bus-Anfragen zum Senden und Verwalten von Benachrichtigungen reagieren.

#### 12.7.6. Designentscheidungen und Trade-offs

- Entscheidung: Verwendung der `zbus`-Bibliothek für die D-Bus-Kommunikation.
- Alternativen: Andere D-Bus-Bibliotheken oder eigene Implementierung.
- Trade-offs: `zbus` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 12.7.7. Teststrategie

- Unit-Tests:
    - Mocken von D-Bus-Clients.
    - Testen der Serialisierung und Deserialisierung von Benachrichtigungen.
    - Testen der Fehlerbehandlung.
    - Testen der Verwaltung von Benachrichtigungen.
- Integrationstests:
    - Testen der Interaktion mit realen D-Bus-Clients.

---

### 13. `novade-system/src/ui` Modul - Detaillierte Spezifikation

Das `novade-system/src/ui` Modul ist für die Bereitstellung von Schnittstellen für die Kommunikation mit der UI-Schicht (`novade-ui`) verantwortlich. Es MUSS die Interaktion mit dem Compositor, dem Session Manager und anderen Systemdiensten abstrahieren.

### 13.1. `novade-system/src/ui/compositor.rs` - Detaillierte Spezifikation

#### 13.1.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Bereitstellung von Schnittstellen zur Interaktion mit dem Compositor.
    - Abstraktion der Wayland-Kommunikation für die UI-Schicht.
- **Grenzen:**
    - Die Implementierung des Wayland-Compositors ist im `novade-system/src/compositor` Modul implementiert.

#### 13.1.2. Schnittstellen zu anderen Modulen

- `novade-system/src/compositor`: Wird verwendet, um mit dem Wayland-Compositor zu kommunizieren.
- `novade-ui`: Wird verwendet, um Schnittstellen für die UI-Schicht bereitzustellen.

#### 13.1.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Fenstern, Oberflächen, Eingabeereignissen und Compositor-spezifischen Daten.
- **Algorithmen:**
    - Algorithmen zur Serialisierung und Deserialisierung von Wayland-Nachrichten.
    - Algorithmen zur Fehlerbehandlung bei der Wayland-Kommunikation.
    - Algorithmen zur Umwandlung von UI-Eingabeereignissen in Wayland-Eingabeereignisse.

#### 13.1.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindungsherstellung mit dem Wayland-Compositor.
    - Fehler beim Senden und Empfangen von Wayland-Nachrichten.
    - Timeouts bei Wayland-Anfragen.
    - Ungültige Wayland-Nachrichten.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 13.1.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Wayland-Ereignisse, die vom Compositor empfangen werden, z.B. Fensterbewegungen, Größenänderungen, Eingabeereignisse.
- **Callbacks:**
    - Funktionen, die auf Wayland-Ereignisse reagieren und diese an die UI-Schicht weiterleiten.

#### 13.1.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung der `smithay`-Bibliothek für die Wayland-Kommunikation.
- **Alternativen:** Andere Wayland-Bibliotheken oder eigene Implementierung.
- **Trade-offs:** `smithay` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 13.1.7. Teststrategie

- **Unit-Tests:**
    - Mocken des Wayland-Compositors.
    - Testen der Serialisierung und Deserialisierung von Wayland-Nachrichten.
    - Testen der Fehlerbehandlung.
    - Testen der Umwandlung von UI-Eingabeereignissen in Wayland-Eingabeereignisse.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen Wayland-Compositor.

### 13.2. `novade-system/src/ui/session_manager.rs` - Detaillierte Spezifikation

#### 13.2.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Bereitstellung von Schnittstellen zur Interaktion mit dem Session Manager.
    - Abstraktion der D-Bus-Kommunikation mit dem Session Manager für die UI-Schicht.
- **Grenzen:**
    - Die Implementierung des Session Managers ist im `novade-system/src/session_manager` Modul implementiert.

#### 13.2.2. Schnittstellen zu anderen Modulen

- `novade-system/src/session_manager`: Wird verwendet, um mit dem Session Manager zu kommunizieren.
- `novade-ui`: Wird verwendet, um Schnittstellen für die UI-Schicht bereitzustellen.

#### 13.2.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Benutzerinformationen, Sitzungsdaten und Session-Manager-spezifischen Daten.
- **Algorithmen:**
    - Algorithmen zur Serialisierung und Deserialisierung von D-Bus-Nachrichten.
    - Algorithmen zur Fehlerbehandlung bei der D-Bus-Kommunikation mit dem Session Manager.
    - Algorithmen zur Umwandlung von UI-Anfragen in Session-Manager-Anfragen.

#### 13.2.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindungsherstellung mit dem Session Manager.
    - Fehler beim Senden und Empfangen von D-Bus-Nachrichten.
    - Timeouts bei D-Bus-Anfragen.
    - Ungültige Session-Manager-Nachrichten.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 13.2.5. Ereignisse und Callbacks

- **Ereignisse:**
    - D-Bus-Ereignisse, die vom Session Manager empfangen werden, z.B. Änderungen des Sitzungsstatus, Benutzeranmeldung, Benutzerabmeldung.
- **Callbacks:**
    - Funktionen, die auf Session-Manager-Ereignisse reagieren und diese an die UI-Schicht weiterleiten.

#### 13.2.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung der `zbus`-Bibliothek für die D-Bus-Kommunikation.
- **Alternativen:** Andere D-Bus-Bibliotheken oder eigene Implementierung.
- **Trade-offs:** `zbus` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 13.2.7. Teststrategie

- **Unit-Tests:**
    - Mocken des Session Managers.
    - Testen der Serialisierung und Deserialisierung von D-Bus-Nachrichten.
    - Testen der Fehlerbehandlung.
    - Testen der Umwandlung von UI-Anfragen in Session-Manager-Anfragen.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen Session Manager.

### 13.3. `novade-system/src/ui/settings_daemon.rs` - Detaillierte Spezifikation

#### 13.3.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Bereitstellung von Schnittstellen zur Interaktion mit dem Settings Daemon.
    - Abstraktion der D-Bus-Kommunikation mit dem Settings Daemon für die UI-Schicht.
- **Grenzen:**
    - Die Implementierung des Settings Daemon ist im `novade-system/src/settings_daemon` Modul implementiert.

#### 13.3.2. Schnittstellen zu anderen Modulen

- `novade-system/src/settings_daemon`: Wird verwendet, um mit dem Settings Daemon zu kommunizieren.
- `novade-ui`: Wird verwendet, um Schnittstellen für die UI-Schicht bereitzustellen.

#### 13.3.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Einstellungen, Schemata und Settings-Daemon-spezifischen Daten.
- **Algorithmen:**
    - Algorithmen zur Serialisierung und Deserialisierung von D-Bus-Nachrichten.
    - Algorithmen zur Fehlerbehandlung bei der D-Bus-Kommunikation mit dem Settings Daemon.
    - Algorithmen zur Umwandlung von UI-Anfragen in Settings-Daemon-Anfragen.

#### 13.3.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindungsherstellung mit dem Settings Daemon.
    - Fehler beim Senden und Empfangen von D-Bus-Nachrichten.
    - Timeouts bei D-Bus-Anfragen.
    - Ungültige Settings-Daemon-Nachrichten.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 13.3.5. Ereignisse und Callbacks

- **Ereignisse:**
    - D-Bus-Ereignisse, die vom Settings Daemon empfangen werden, z.B. Änderungen von Einstellungen.
- **Callbacks:**
    - Funktionen, die auf Settings-Daemon-Ereignisse reagieren und diese an die UI-Schicht weiterleiten.

#### 13.3.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung der `zbus`-Bibliothek für die D-Bus-Kommunikation.
- **Alternativen:** Andere D-Bus-Bibliotheken oder eigene Implementierung.
- **Trade-offs:** `zbus` bietet eine hohe Abstraktionsebene und erleichtert die Implementierung, kann aber die Flexibilität einschränken.

#### 13.3.7. Teststrategie

- **Unit-Tests:**
    - Mocken des Settings Daemon.
    - Testen der Serialisierung und Deserialisierung von D-Bus-Nachrichten.
    - Testen der Fehlerbehandlung.
    - Testen der Umwandlung von UI-Anfragen in Settings-Daemon-Anfragen.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen Settings Daemon.

### 13.4. `novade-system/src/ui/window_manager.rs` - Detaillierte Spezifikation

#### 13.4.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Bereitstellung von Schnittstellen zur Interaktion mit dem Window Manager.
    - Abstraktion der Fensterverwaltungslogik für die UI-Schicht.
- **Grenzen:**
    - Die Implementierung des Window Managers ist teilweise im `novade-system/src/compositor` Modul und teilweise in der `novade-shell` implementiert. Dieses Modul dient als Bindeglied.

#### 13.4.2. Schnittstellen zu anderen Modulen

- `novade-system/src/compositor`: Wird verwendet, um mit dem Compositor zu kommunizieren (z.B. Fensterplatzierung, Fokus).
- `novade-shell`: Wird verwendet, um UI-Elemente für die Fensterverwaltung bereitzustellen (z.B. Titelleisten).
- `novade-ui`: Wird verwendet, um Schnittstellen für die UI-Schicht bereitzustellen.

#### 13.4.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Fenstern, Fenstereigenschaften, Layouts und Window-Manager-spezifischen Daten.
- **Algorithmen:**
    - Algorithmen zur Berechnung von Fensterpositionen und -größen.
    - Algorithmen zur Verwaltung von Fensterfokus.
    - Algorithmen zur Implementierung von Fenstereffekten (z.B. Animationen).

#### 13.4.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Kommunikation mit dem Compositor.
    - Fehler bei der Berechnung von Fensterpositionen und -größen.
    - Ungültige Fensterdaten.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 13.4.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Compositor-Ereignisse, die Fenster betreffen (z.B. Fenstererstellung, Fensterzerstörung, Fokusänderung).
    - UI-Ereignisse, die Fensteraktionen auslösen (z.B. Fenster schließen, Fenster maximieren).
- **Callbacks:**
    - Funktionen, die auf Fensterereignisse reagieren und diese zwischen Compositor und UI-Schicht vermitteln.

#### 13.4.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Abstraktion der Fensterverwaltungslogik, um die UI-Schicht vom Compositor zu entkoppeln.
- **Alternativen:** Direkte Interaktion der UI-Schicht mit dem Compositor.
- **Trade-offs:** Die Abstraktion erhöht die Komplexität, ermöglicht aber eine flexiblere UI-Entwicklung und bessere Testbarkeit.

#### 13.4.7. Teststrategie

- **Unit-Tests:**
    - Mocken des Compositors und der UI-Schicht.
    - Testen der Algorithmen zur Fensterpositionierung und -größenberechnung.
    - Testen der Fensterfokusverwaltung.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen Compositor und der realen UI-Schicht.

### 13.5. `novade-system/src/ui/mod.rs` - Modulübersicht

Das `novade-system/src/ui/mod.rs` Modul dient als zentrale Anlaufstelle für die UI-bezogenen Submodule. Es definiert die öffentliche API für die UI-Schicht (`novade-ui`) und koordiniert die Interaktion zwischen den einzelnen UI-Submodulen.

#### 13.5.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Definition der öffentlichen API für die UI-Schicht.
    - Koordination der Interaktion zwischen den UI-Submodulen.
    - Bereitstellung von Hilfsfunktionen und Datentypen für die UI-Kommunikation.
- **Grenzen:**
    - Die eigentliche Implementierung der UI-Submodule erfolgt in den jeweiligen Dateien (z.B. `compositor.rs`, `session_manager.rs`).

#### 13.5.2. Schnittstellen zu anderen Modulen

- `novade-ui`: Wird verwendet, um die öffentliche API für die UI-Schicht bereitzustellen.
- `novade-system/src/compositor`, `novade-system/src/session_manager`, `novade-system/src/settings_daemon`, `novade-system/src/window_manager`: Werden verwendet, um die Funktionalität der Submodule zu koordinieren.

#### 13.5.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Gemeinsame Datentypen für die UI-Kommunikation.
- **Algorithmen:**
    - Algorithmen zur Koordination der Interaktion zwischen den Submodulen.

#### 13.5.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Koordination der Submodule.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 13.5.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Ereignisse, die von den Submodulen an die UI-Schicht weitergeleitet werden.
- **Callbacks:**
    - Funktionen, die von der UI-Schicht aufgerufen werden, um mit den Submodulen zu interagieren.

#### 13.5.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung eines zentralen Moduls zur Koordination der UI-Kommunikation.
- **Alternativen:** Direkte Interaktion der UI-Schicht mit den Submodulen.
- **Trade-offs:** Die zentrale Koordination erleichtert die Verwaltung der UI-Kommunikation, kann aber die Komplexität des `mod.rs` Moduls erhöhen.

#### 13.5.7. Teststrategie

- **Unit-Tests:**
    - Testen der Koordination der Submodule.
    - Testen der Hilfsfunktionen und Datentypen.
- **Integrationstests:**
    - Testen der Interaktion mit der realen UI-Schicht und den realen Submodulen.

---

### 14. `novade-system/src/services` Modul - Detaillierte Spezifikation

Das `novade-system/src/services` Modul ist für die Implementierung von Systemdiensten verantwortlich, die von anderen Komponenten von NovaDE genutzt werden können. Diese Dienste abstrahieren komplexe Systemfunktionalitäten und stellen sie über wohldefinierte Schnittstellen zur Verfügung.

### 14.1. `novade-system/src/services/display_manager_service.rs` - Detaillierte Spezifikation

#### 14.1.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung eines Dienstes zur Verwaltung von Displays und Monitoren.
    - Abstraktion der Interaktion mit dem Compositor für die UI-Schicht und andere Dienste.
- **Grenzen:**
    - Die eigentliche Implementierung der Display-Verwaltung erfolgt im `novade-system/src/compositor` Modul.

#### 14.1.2. Schnittstellen zu anderen Modulen

- `novade-system/src/compositor`: Wird verwendet, um mit dem Compositor zu kommunizieren.
- `novade-ui`: Kann verwendet werden, um die Display-Verwaltung der UI-Schicht zur Verfügung zu stellen.
- Andere Dienste, die Informationen über Displays benötigen.

#### 14.1.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Displays, Monitoren, Auflösungen, Bildwiederholraten und Display-Konfigurationen.
- **Algorithmen:**
    - Algorithmen zur Abfrage und Konfiguration von Displays und Monitoren.
    - Algorithmen zur Berechnung optimaler Auflösungen und Bildwiederholraten.
    - Algorithmen zur Verwaltung von Multi-Monitor-Setups.

#### 14.1.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Kommunikation mit dem Compositor.
    - Ungültige Display-Konfigurationen.
    - Nicht unterstützte Monitoreinstellungen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 14.1.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Compositor-Ereignisse, die Displays betreffen (z.B. Monitor-Hinzufügen, Monitor-Entfernen, Auflösungsänderung).
- **Callbacks:**
    - Funktionen, die auf Display-Ereignisse reagieren und diese an andere Komponenten weiterleiten.

#### 14.1.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung eines dedizierten Display-Manager-Dienstes, um die Display-Verwaltung zu abstrahieren.
- **Alternativen:** Direkte Interaktion anderer Komponenten mit dem Compositor.
- **Trade-offs:** Der Dienst erhöht die Modularität und Testbarkeit, kann aber die Komplexität erhöhen.

#### 14.1.7. Teststrategie

- **Unit-Tests:**
    - Mocken des Compositors.
    - Testen der Algorithmen zur Display-Konfiguration.
    - Testen der Verwaltung von Multi-Monitor-Setups.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen Compositor.

### 14.2. `novade-system/src/services/power_manager_service.rs` - Detaillierte Spezifikation

#### 14.2.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung eines Dienstes zur Verwaltung der Energieeinstellungen des Systems.
    - Abstraktion der Interaktion mit dem Betriebssystem (z.B. UPower, `logind`) für die UI-Schicht und andere Dienste.
- **Grenzen:**
    - Die eigentliche Implementierung der Energieverwaltung erfolgt durch das Betriebssystem.

#### 14.2.2. Schnittstellen zu anderen Modulen

- Betriebssystem-APIs (z.B. UPower, `logind`).
- `novade-ui`: Kann verwendet werden, um die Energieverwaltung der UI-Schicht zur Verfügung zu stellen.
- Andere Dienste, die Informationen über den Energiezustand des Systems benötigen.

#### 14.2.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Batteriestatus, Energieprofilen, Energieeinstellungen und Power-Management-Ereignissen.
- **Algorithmen:**
    - Algorithmen zur Abfrage des Batteriestatus.
    - Algorithmen zur Verwaltung von Energieprofilen.
    - Algorithmen zur Implementierung von Aktionen bei bestimmten Energieereignissen (z.B. Suspend, Hibernate).

#### 14.2.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Kommunikation mit dem Betriebssystem.
    - Ungültige Energieeinstellungen.
    - Nicht unterstützte Energieverwaltungsfunktionen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 14.2.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Betriebssystem-Ereignisse, die den Energiezustand betreffen (z.B. Batteriestatusänderung, Netzteil angeschlossen/getrennt).
- **Callbacks:**
    - Funktionen, die auf Energieereignisse reagieren und diese an andere Komponenten weiterleiten.

#### 14.2.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung eines dedizierten Power-Manager-Dienstes, um die Energieverwaltung zu abstrahieren.
- **Alternativen:** Direkte Interaktion anderer Komponenten mit dem Betriebssystem.
- **Trade-offs:** Der Dienst erhöht die Modularität und Testbarkeit, kann aber die Komplexität erhöhen.

#### 14.2.7. Teststrategie

- **Unit-Tests:**
    - Mocken der Betriebssystem-APIs.
    - Testen der Algorithmen zur Verwaltung von Energieprofilen.
    - Testen der Implementierung von Aktionen bei Energieereignissen.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen Betriebssystem.

### 14.3. `novade-system/src/services/notification_service.rs` - Detaillierte Spezifikation

#### 14.3.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung eines Dienstes zur Verwaltung von Desktop-Benachrichtigungen.
    - Bereitstellung von Schnittstellen zum Senden und Verwalten von Benachrichtigungen für andere Komponenten.
- **Grenzen:**
    - Die eigentliche Anzeige der Benachrichtigungen kann durch die Shell oder einen anderen Dienst erfolgen.

#### 14.3.2. Schnittstellen zu anderen Modulen

- Andere Dienste und Komponenten, die Benachrichtigungen senden oder verwalten wollen.
- Die Shell oder ein anderer Dienst, der die Benachrichtigungen anzeigt.

#### 14.3.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Benachrichtigungen, Aktionen und Hinweisen.
- **Algorithmen:**
    - Algorithmen zur Verwaltung einer Benachrichtigungswarteschlange.
    - Algorithmen zur Priorisierung und Sortierung von Benachrichtigungen.
    - Algorithmen zur Speicherung und zum Abruf von Benachrichtigungshistorie (optional).

#### 14.3.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige Benachrichtigungsdaten.
    - Fehler beim Speichern oder Abrufen der Benachrichtigungshistorie (optional).
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 14.3.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Ereignisse, die Benachrichtigungen betreffen (z.B. Benachrichtigung hinzugefügt, Benachrichtigung geschlossen, Benachrichtigungsaktion ausgelöst).
- **Callbacks:**
    - Funktionen, die auf Benachrichtigungsereignisse reagieren und diese an andere Komponenten weiterleiten.

#### 14.3.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung eines dedizierten Notification-Dienstes, um die Benachrichtigungsverwaltung zu abstrahieren.
- **Alternativen:** Direkte Implementierung der Benachrichtigungslogik in anderen Komponenten.
- **Trade-offs:** Der Dienst erhöht die Modularität und Wiederverwendbarkeit, kann aber die Komplexität erhöhen.

#### 14.3.7. Teststrategie

- **Unit-Tests:**
    - Testen der Algorithmen zur Benachrichtigungsverwaltung.
    - Testen der Priorisierung und Sortierung von Benachrichtigungen.
    - Testen der Speicherung und des Abrufs der Benachrichtigungshistorie (optional).
- **Integrationstests:**
    - Testen der Interaktion mit anderen Komponenten, die Benachrichtigungen senden oder verwalten.
    - Testen der Interaktion mit dem Dienst, der die Benachrichtigungen anzeigt.

---

### 14.4. `novade-system/src/services/session_manager_service.rs` - Detaillierte Spezifikation

#### 14.4.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung eines Dienstes zur Verwaltung von Benutzersitzungen.
    - Abstraktion der Interaktion mit dem Betriebssystem (z.B. `logind`, systemd) für die UI-Schicht und andere Dienste.
- **Grenzen:**
    - Die eigentliche Implementierung der Sitzungsverwaltung erfolgt durch das Betriebssystem.

#### 14.4.2. Schnittstellen zu anderen Modulen

- Betriebssystem-APIs (z.B. `logind`, systemd).
- `novade-ui`: Kann verwendet werden, um die Sitzungsverwaltung der UI-Schicht zur Verfügung zu stellen.
- Andere Dienste, die Informationen über den Sitzungszustand benötigen.

#### 14.4.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Benutzern, Sitzungen, Sitzungszuständen und Sitzungsmanagement-Ereignissen.
- **Algorithmen:**
    - Algorithmen zur Anmeldung, Abmeldung und zum Sperren von Benutzern.
    - Algorithmen zur Verwaltung von Sitzungstypen (z.B. grafisch, Konsole).
    - Algorithmen zur Implementierung von Aktionen bei Sitzungsereignissen (z.B. Sitzungswechsel).

#### 14.4.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Kommunikation mit dem Betriebssystem.
    - Ungültige Benutzerdaten.
    - Nicht unterstützte Sitzungsverwaltungsfunktionen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 14.4.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Betriebssystem-Ereignisse, die Sitzungen betreffen (z.B. Benutzeranmeldung, Benutzerabmeldung, Sitzungswechsel).
- **Callbacks:**
    - Funktionen, die auf Sitzungsereignisse reagieren und diese an andere Komponenten weiterleiten.

#### 14.4.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung eines dedizierten Session-Manager-Dienstes, um die Sitzungsverwaltung zu abstrahieren.
- **Alternativen:** Direkte Interaktion anderer Komponenten mit dem Betriebssystem.
- **Trade-offs:** Der Dienst erhöht die Modularität und Testbarkeit, kann aber die Komplexität erhöhen.

#### 14.4.7. Teststrategie

- **Unit-Tests:**
    - Mocken der Betriebssystem-APIs.
    - Testen der Algorithmen zur Anmeldung, Abmeldung und zum Sperren von Benutzern.
    - Testen der Implementierung von Aktionen bei Sitzungsereignissen.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen Betriebssystem.

### 14.5. `novade-system/src/services/mod.rs` - Modulübersicht

Das `novade-system/src/services/mod.rs` Modul dient als zentrale Anlaufstelle für die Service-bezogenen Submodule. Es definiert die öffentliche API für die Systemdienste und koordiniert die Interaktion zwischen den einzelnen Service-Submodulen.

#### 14.5.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Definition der öffentlichen API für die Systemdienste.
    - Koordination der Interaktion zwischen den Service-Submodulen.
    - Bereitstellung von Hilfsfunktionen und Datentypen für die Service-Kommunikation.
- **Grenzen:**
    - Die eigentliche Implementierung der Service-Submodule erfolgt in den jeweiligen Dateien (z.B. `display_manager_service.rs`, `power_manager_service.rs`).

#### 14.5.2. Schnittstellen zu anderen Modulen

- Andere Module innerhalb von `novade-system`, `novade-ui` und `novade-domain`: Werden verwendet, um die Funktionalität der Submodule zu koordinieren und die Dienste zur Verfügung zu stellen.

#### 14.5.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Gemeinsame Datentypen für die Service-Kommunikation.
- **Algorithmen:**
    - Algorithmen zur Koordination der Interaktion zwischen den Submodulen.

#### 14.5.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Koordination der Submodule.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 14.5.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Ereignisse, die von den Submodulen an andere Komponenten weitergeleitet werden.
- **Callbacks:**
    - Funktionen, die von anderen Komponenten aufgerufen werden, um mit den Submodulen zu interagieren.

#### 14.5.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung eines zentralen Moduls zur Koordination der Service-Kommunikation.
- **Alternativen:** Direkte Interaktion anderer Komponenten mit den Service-Submodulen.
- **Trade-offs:** Die zentrale Koordination erleichtert die Verwaltung der Service-Kommunikation, kann aber die Komplexität des `mod.rs` Moduls erhöhen.

#### 14.5.7. Teststrategie

- **Unit-Tests:**
    - Testen der Koordination der Submodule.
    - Testen der Hilfsfunktionen und Datentypen.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Modulen und den realen Service-Submodulen.

---

### 15. `novade-system/src/audio` Modul - Detaillierte Spezifikation

Das `novade-system/src/audio` Modul ist für die Verwaltung von Audio-Funktionalitäten in NovaDE verantwortlich. Es abstrahiert die Interaktion mit dem Audio-Server (PipeWire 12) und stellt eine einheitliche Schnittstelle für andere Komponenten bereit.

#### 15.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung von Audio-Geräten (Sinks und Sources).
    - Steuerung von Audio-Streams.
    - Mischen und Routing von Audio.
    - Bereitstellung von Audio-Informationen für die UI-Schicht.
- **Grenzen:**
    - Die eigentliche Audio-Verarbeitung und das Mischen erfolgt durch PipeWire.

#### 15.2. Schnittstellen zu anderen Modulen

- `PipeWire`: Für die Interaktion mit dem Audio-Server.
- `novade-ui`: Zur Bereitstellung von Audio-Informationen und Steuerungsmöglichkeiten für den Benutzer.
- Andere Systemdienste, die Audio-Funktionalitäten benötigen.

#### 15.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Audio-Geräten (Name, Beschreibung, Typ, etc.).
    - Strukturen zur Darstellung von Audio-Streams (Anwendung, Format, Lautstärke, etc.).
    - Datenstrukturen für Audio-Formate und -Kanäle.
- **Algorithmen:**
    - Algorithmen zum Abrufen und Ändern von Audio-Geräten und -Streams.
    - Algorithmen zur Lautstärkeregelung und zum Muting.
    - Algorithmen zum Routing von Audio zwischen Geräten.

#### 15.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindung mit PipeWire.
    - Ungültige Audio-Geräte oder -Streams.
    - Nicht unterstützte Audio-Formate.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 15.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Ereignisse, die Audio-Geräte betreffen (z.B. Gerät hinzugefügt, Gerät entfernt, Lautstärke geändert).
    - Ereignisse, die Audio-Streams betreffen (z.B. Stream gestartet, Stream beendet, Lautstärke geändert).
- **Callbacks:**
    - Funktionen, die auf Audio-Ereignisse reagieren und diese an andere Komponenten weiterleiten.

#### 15.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung von PipeWire für die Audio-Verwaltung.
- **Alternativen:** Andere Audio-Server (z.B. PulseAudio).
- **Trade-offs:** PipeWire bietet moderne Funktionen und gute Leistung, kann aber komplexer zu konfigurieren sein.

#### 15.7. Teststrategie

- **Unit-Tests:**
    - Mocken der PipeWire-API.
    - Testen der Algorithmen zur Audio-Verwaltung.
    - Testen der Ereignisbehandlung.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen PipeWire-Server.

### 15.8. `novade-system/src/audio/audio_device.rs` - Detaillierte Spezifikation

#### 15.8.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Definition der Datenstrukturen zur Darstellung von Audio-Geräten.
    - Bereitstellung von Funktionen zur Abfrage und Änderung von Audio-Geräteigenschaften.
- **Grenzen:**
    - Die eigentliche Interaktion mit dem Audio-Server erfolgt im übergeordneten `audio` Modul.

#### 15.8.2. Schnittstellen zu anderen Modulen

- `novade-system/src/audio`: Wird verwendet, um mit dem übergeordneten Modul zu interagieren.
- Andere Module, die Informationen über Audio-Geräte benötigen.

#### 15.8.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `AudioDevice` Struct:
        - `id: u32`: Eindeutige ID des Geräts.
        - `name: String`: Name des Geräts.
        - `description: String`: Beschreibung des Geräts.
        - `device_type: AudioDeviceType`: Typ des Geräts (Sink oder Source).
        - `volume: Volume`: Aktuelle Lautstärke des Geräts.
        - `is_muted: bool`: Stummgeschaltet-Status des Geräts.
        - Weitere gerätespezifische Eigenschaften.
    - `AudioDeviceType` Enum:
        - `Sink`: Ausgabegerät (z.B. Lautsprecher, Kopfhörer).
        - `Source`: Aufnahmegerät (z.B. Mikrofon).
- **Algorithmen:**
    - Funktionen zur Abfrage von Geräteinformationen.
    - Funktionen zur Änderung der Gerätelautstärke und des Stummgeschaltet-Status.

#### 15.8.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige Geräte-IDs.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 15.8.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Ereignisse, die Geräte betreffen (z.B. Lautstärkeänderung, Stummgeschaltet-Status-Änderung), werden im übergeordneten `audio` Modul behandelt.
- **Callbacks:**
    - Keine spezifischen Callbacks sind definiert.

#### 15.8.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Definition von separaten Datenstrukturen für Audio-Geräte und -Streams, um die Modularität zu erhöhen.
- **Alternativen:** Kombinierte Datenstrukturen.
- **Trade-offs:** Separate Datenstrukturen erhöhen die Flexibilität, können aber die Komplexität erhöhen.

#### 15.8.7. Teststrategie

- **Unit-Tests:**
    - Testen der Datenstrukturen.
    - Testen der Funktionen zur Abfrage und Änderung von Geräteigenschaften.

### 15.9. `novade-system/src/audio/audio_stream.rs` - Detaillierte Spezifikation

#### 15.9.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Definition der Datenstrukturen zur Darstellung von Audio-Streams.
    - Bereitstellung von Funktionen zur Abfrage und Änderung von Audio-Stream-Eigenschaften.
- **Grenzen:**
    - Die eigentliche Interaktion mit dem Audio-Server erfolgt im übergeordneten `audio` Modul.

#### 15.9.2. Schnittstellen zu anderen Modulen

- `novade-system/src/audio`: Wird verwendet, um mit dem übergeordneten Modul zu interagieren.
- Andere Module, die Informationen über Audio-Streams benötigen.

#### 15.9.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `StreamInfo` Struct:
        - `id: u32`: Eindeutige ID des Streams.
        - `application_name: String`: Name der Anwendung, die den Stream erzeugt hat.
        - `format: AudioFormat`: Format des Streams.
        - `channels: u32`: Anzahl der Kanäle des Streams.
        - `volume: Volume`: Aktuelle Lautstärke des Streams.
        - `is_muted: bool`: Stummgeschaltet-Status des Streams.
        - Weitere stream-spezifische Eigenschaften.
    - `AudioFormat` Enum:
        - Verschiedene Audio-Formate (z.B. PCM, FLAC, MP3).
- **Algorithmen:**
    - Funktionen zur Abfrage von Stream-Informationen.
    - Funktionen zur Änderung der Stream-Lautstärke und des Stummgeschaltet-Status.

#### 15.9.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige Stream-IDs.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 15.9.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Ereignisse, die Streams betreffen (z.B. Lautstärkeänderung, Stummgeschaltet-Status-Änderung), werden im übergeordneten `audio` Modul behandelt.
- **Callbacks:**
    - Keine spezifischen Callbacks sind definiert.

#### 15.9.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Definition von separaten Datenstrukturen für Audio-Geräte und -Streams, um die Modularität zu erhöhen.
- **Alternativen:** Kombinierte Datenstrukturen.
- **Trade-offs:** Separate Datenstrukturen erhöhen die Flexibilität, können aber die Komplexität erhöhen.

#### 15.9.7. Teststrategie

- **Unit-Tests:**
    - Testen der Datenstrukturen.
    - Testen der Funktionen zur Abfrage und Änderung von Stream-Eigenschaften.

### 15.10. `novade-system/src/audio/mod.rs` - Modulübersicht

#### 15.10.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Definition der öffentlichen API für das Audio-Modul.
    - Koordination der Interaktion zwischen den Audio-Submodulen (`audio_device.rs`, `audio_stream.rs`).
    - Implementierung der Logik zur Interaktion mit PipeWire.
    - Bereitstellung von Hilfsfunktionen und Datentypen für die Audio-Kommunikation.
- **Grenzen:**
    - Die eigentliche Audio-Verarbeitung und das Mischen erfolgt durch PipeWire.

#### 15.10.2. Schnittstellen zu anderen Modulen

- `PipeWire`: Wird verwendet, um mit dem Audio-Server zu kommunizieren.
- `novade-ui`: Wird verwendet, um Audio-Informationen und Steuerungsmöglichkeiten für den Benutzer bereitzustellen.
- Andere Module innerhalb von `novade-system`.

#### 15.10.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `AudioDevice` (definiert in `audio_device.rs`).
    - `StreamInfo` (definiert in `audio_stream.rs`).
    - `AudioEvent` Enum:
        - Varianten zur Darstellung von Audio-Ereignissen (z.B. Gerät hinzugefügt, Stream beendet, Lautstärke geändert) (siehe Tabelle 5.3).
    - `Volume` Typ für die Lautstärkeregelung (0.0 bis 1.0).

- **Algorithmen:**
    - Algorithmen zur Interaktion mit PipeWire (z.B. Verbinden mit dem Server, Erstellen von Objekten, Senden von Befehlen).
    - Algorithmen zur Umwandlung von PipeWire-Daten in die Modul-eigenen Datenstrukturen.
    - Algorithmen zur Verarbeitung von Audio-Ereignissen und zur Benachrichtigung anderer Komponenten.

#### 15.10.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindung mit PipeWire.
    - Fehler bei der Interaktion mit PipeWire-Objekten.
    - Unerwartete PipeWire-Ereignisse.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 15.10.5. Ereignisse und Callbacks

- **Ereignisse:**
    - `AudioEvent` Enum (siehe Tabelle 5.3) zur Darstellung von Audio-Ereignissen.
- **Callbacks:**
    - Funktionen, die von anderen Komponenten implementiert werden, um auf Audio-Ereignisse zu reagieren.

#### 15.10.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung von separaten Submodulen für Audio-Geräte und -Streams, um die Modularität zu erhöhen.
- **Alternativen:** Kombinierte Submodule.
- **Trade-offs:** Separate Submodule erhöhen die Flexibilität, können aber die Komplexität erhöhen.

#### 15.10.7. Teststrategie

- **Unit-Tests:**
    - Mocken der PipeWire-API.
    - Testen der Algorithmen zur Interaktion mit PipeWire.
    - Testen der Ereignisbehandlung.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen PipeWire-Server.

### 15.11. Anwendungsfälle

- **Anwendungsfall 1: Lautstärkeregelung durch den Benutzer**
    1. Der Benutzer ändert die Lautstärke eines Audio-Geräts oder -Streams über die UI.
    2. Die UI ruft die entsprechende Funktion im `audio` Modul auf.
    3. Das `audio` Modul interagiert mit PipeWire, um die Lautstärke zu ändern.
    4. PipeWire ändert die Lautstärke des Geräts oder Streams.
    5. PipeWire sendet ein Ereignis, das die Lautstärkeänderung signalisiert.
    6. Das `audio` Modul empfängt das Ereignis und benachrichtigt die UI.
    7. Die UI aktualisiert die Anzeige.
- **Anwendungsfall 2: Automatisches Routing von Audio**
    1. Ein neues Audio-Gerät wird angeschlossen.
    2. PipeWire erkennt das Gerät und sendet ein Ereignis.
    3. Das `audio` Modul empfängt das Ereignis und entscheidet, ob das Audio automatisch auf das neue Gerät geroutet werden soll (basierend auf Konfigurationseinstellungen).
    4. Wenn ja, interagiert das `audio` Modul mit PipeWire, um das Routing zu ändern.
    5. PipeWire ändert das Routing.
    6. PipeWire sendet Ereignisse, die die Routing-Änderung signalisieren.
    7. Das `audio` Modul empfängt die Ereignisse und benachrichtigt die UI.
    8. Die UI aktualisiert die Anzeige.

---

### 16. `novade-system/src/network` Modul - Detaillierte Spezifikation

Das `novade-system/src/network` Modul ist für die Verwaltung von Netzwerkverbindungen in NovaDE verantwortlich. Es abstrahiert die Interaktion mit dem Netzwerk-Manager (z.B. NetworkManager) und stellt eine einheitliche Schnittstelle für andere Komponenten bereit.

#### 16.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung von Netzwerkgeräten (z.B. Ethernet, WLAN).
    - Verwaltung von Netzwerkverbindungen (z.B. aktive Verbindungen, verfügbare Verbindungen).
    - Verwaltung von Netzwerkprofilen (z.B. gespeicherte WLAN-Passwörter).
    - Bereitstellung von Netzwerk-Informationen für die UI-Schicht.
- **Grenzen:**
    - Die eigentliche Netzwerk-Verwaltung erfolgt durch den Netzwerk-Manager.

#### 16.2. Schnittstellen zu anderen Modulen

- `NetworkManager`: Für die Interaktion mit dem Netzwerk-Manager.
- `novade-ui`: Zur Bereitstellung von Netzwerk-Informationen und Steuerungsmöglichkeiten für den Benutzer.
- Andere Systemdienste, die Netzwerk-Funktionalitäten benötigen.

#### 16.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Darstellung von Netzwerkgeräten (Name, Typ, Status, etc.).
    - Strukturen zur Darstellung von Netzwerkverbindungen (SSID, Verschlüsselung, Status, etc.).
    - Strukturen zur Darstellung von Netzwerkprofilen (gespeicherte Passwörter, etc.).
- **Algorithmen:**
    - Algorithmen zum Abrufen und Ändern von Netzwerkgeräten und -Verbindungen.
    - Algorithmen zur Verwaltung von Netzwerkprofilen.
    - Algorithmen zur Herstellung und Trennung von Netzwerkverbindungen.

#### 16.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindung mit dem Netzwerk-Manager.
    - Ungültige Netzwerkgeräte oder -Verbindungen.
    - Fehler bei der Herstellung oder Trennung von Netzwerkverbindungen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 16.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Ereignisse, die Netzwerkgeräte betreffen (z.B. Gerät verbunden, Gerät getrennt, Status geändert).
    - Ereignisse, die Netzwerkverbindungen betreffen (z.B. Verbindung hergestellt, Verbindung getrennt, IP-Adresse geändert).
- **Callbacks:**
    - Funktionen, die auf Netzwerk-Ereignisse reagieren und diese an andere Komponenten weiterleiten.

#### 16.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung des NetworkManagers für die Netzwerk-Verwaltung.
- **Alternativen:** Andere Netzwerk-Manager oder direkte Interaktion mit den Netzwerk-Schnittstellen des Betriebssystems.
- **Trade-offs:** Der NetworkManager bietet viele Funktionen und gute Abstraktion, kann aber komplex sein.

#### 16.7. Teststrategie

- **Unit-Tests:**
    - Mocken der NetworkManager-API.
    - Testen der Algorithmen zur Netzwerk-Verwaltung.
    - Testen der Ereignisbehandlung.
- **Integrationstests:**
    - Testen der Interaktion mit dem realen NetworkManager.

### 16.8. `novade-system/src/network/network_device.rs` - Detaillierte Spezifikation

#### 16.8.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Definition der Datenstrukturen zur Darstellung von Netzwerkgeräten.
    - Bereitstellung von Funktionen zur Abfrage von Netzwerk-Geräteigenschaften.
- **Grenzen:**
    - Die eigentliche Interaktion mit dem Netzwerk-Manager erfolgt im übergeordneten `network` Modul.

#### 16.8.2. Schnittstellen zu anderen Modulen

- `novade-system/src/network`: Wird verwendet, um mit dem übergeordneten Modul zu interagieren.
- Andere Module, die Informationen über Netzwerkgeräte benötigen.

#### 16.8.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `NetworkDevice` Struct:
        - `name: String`: Name des Geräts (z.B. "eth0", "wlan0").
        - `device_type: NetworkDeviceType`: Typ des Geräts (Ethernet, WLAN, etc.).
        - `status: NetworkDeviceStatus`: Aktueller Status des Geräts (verbunden, getrennt, etc.).
        - `mac_address: String`: MAC-Adresse des Geräts.
        - `ip_address: Option<String>`: Aktuelle IP-Adresse des Geräts (falls vorhanden).
        - Weitere gerätespezifische Eigenschaften.
    - `NetworkDeviceType` Enum:
        - `Ethernet`: Ethernet-Gerät.
        - `Wifi`: WLAN-Gerät.
        - `Bluetooth`: Bluetooth-Gerät (für Netzwerkverbindungen).
        - `Modem`: Modem-Gerät.
        - Weitere Gerätetypen.
    - `NetworkDeviceStatus` Enum:
        - `Connected`: Gerät ist verbunden.
        - `Disconnected`: Gerät ist getrennt.
        - `Connecting`: Gerät verbindet sich.
        - `Unavailable`: Gerät ist nicht verfügbar.
        - Weitere Status.
- **Algorithmen:**
    - Funktionen zur Abfrage von Geräteinformationen.

#### 16.8.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige Gerätenamen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 16.8.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Ereignisse, die Geräte betreffen (z.B. Statusänderung), werden im übergeordneten `network` Modul behandelt.
- **Callbacks:**
    - Keine spezifischen Callbacks sind definiert.

#### 16.8.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Definition von separaten Datenstrukturen für Netzwerkgeräte und -Verbindungen, um die Modularität zu erhöhen.
- **Alternativen:** Kombinierte Datenstrukturen.
- **Trade-offs:** Separate Datenstrukturen erhöhen die Flexibilität, können aber die Komplexität erhöhen.

#### 16.8.7. Teststrategie

- **Unit-Tests:**
    - Testen der Datenstrukturen.
    - Testen der Funktionen zur Abfrage von Geräteigenschaften.

### 16.9. `novade-system/src/network/network_connection.rs` - Detaillierte Spezifikation

#### 16.9.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Definition der Datenstrukturen zur Darstellung von Netzwerkverbindungen.
    - Bereitstellung von Funktionen zur Abfrage und Änderung von Netzwerk-Verbindungseigenschaften.
- **Grenzen:**
    - Die eigentliche Interaktion mit dem Netzwerk-Manager erfolgt im übergeordneten `network` Modul.

#### 16.9.2. Schnittstellen zu anderen Modulen

- `novade-system/src/network`: Wird verwendet, um mit dem übergeordneten Modul zu interagieren.
- Andere Module, die Informationen über Netzwerkverbindungen benötigen.

#### 16.9.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `NetworkConnection` Struct:
        - `id: String`: Eindeutige ID der Verbindung.
        - `name: String`: Name der Verbindung (z.B. SSID für WLAN).
        - `connection_type: NetworkConnectionType`: Typ der Verbindung (z.B. Ethernet, WLAN).
        - `status: NetworkConnectionStatus`: Aktueller Status der Verbindung (aktiv, inaktiv, etc.).
        - `ip4_address: Option<String>`: IPv4-Adresse der Verbindung (falls vorhanden).
        - `ip6_address: Option<String>`: IPv6-Adresse der Verbindung (falls vorhanden).
        - `device: Option<NetworkDevice>`: Das Netzwerkgerät, das die Verbindung verwendet.
        - Weitere verbindungsspezifische Eigenschaften.
    - `NetworkConnectionType` Enum:
        - `Ethernet`: Ethernet-Verbindung.
        - `Wifi`: WLAN-Verbindung.
        - `Vpn`: VPN-Verbindung.
        - Weitere Verbindungstypen.
    - `NetworkConnectionStatus` Enum:
        - `Active`: Verbindung ist aktiv.
        - `Inactive`: Verbindung ist inaktiv.
        - `Connecting`: Verbindung wird hergestellt.
        - `Disconnecting`: Verbindung wird getrennt.
        - Weitere Status.
- **Algorithmen:**
    - Funktionen zur Abfrage von Verbindungsinformationen.
    - Funktionen zum Herstellen und Trennen von Verbindungen.

#### 16.9.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige Verbindungs-IDs.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 16.9.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Ereignisse, die Verbindungen betreffen (z.B. Statusänderung), werden im übergeordneten `network` Modul behandelt.
- **Callbacks:**
    - Keine spezifischen Callbacks sind definiert.

#### 16.9.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Definition von separaten Datenstrukturen für Netzwerkgeräte und -Verbindungen, um die Modularität zu erhöhen.
- **Alternativen:** Kombinierte Datenstrukturen.
- **Trade-offs:** Separate Datenstrukturen erhöhen die Flexibilität, können aber die Komplexität erhöhen.

#### 16.9.7. Teststrategie

- **Unit-Tests:**
    - Testen der Datenstrukturen.
    - Testen der Funktionen zur Abfrage und Änderung von Verbindungseigenschaften.

### 16.10. `novade-system/src/network/mod.rs` - Modulübersicht

#### 16.10.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Definition der öffentlichen API für das Netzwerk-Modul.
    - Koordination der Interaktion zwischen den Netzwerk-Submodulen (`network_device.rs`, `network_connection.rs`).
    - Implementierung der Logik zur Interaktion mit dem Netzwerk-Manager.
    - Bereitstellung von Hilfsfunktionen und Datentypen für die Netzwerk-Kommunikation.
- **Grenzen:**
    - Die eigentliche Netzwerk-Verwaltung erfolgt durch den Netzwerk-Manager.

#### 16.10.2. Schnittstellen zu anderen Modulen

- `NetworkManager`: Wird verwendet, um mit dem Netzwerk-Manager zu kommunizieren.
- `novade-ui`: Wird verwendet, um Netzwerk-Informationen und Steuerungsmöglichkeiten für den Benutzer bereitzustellen.
- Andere Module innerhalb von `novade-system`.

#### 16.10.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `NetworkDevice` (definiert in `network_device.rs`).
    - `NetworkConnection` (definiert in `network_connection.rs`).
    - `NetworkEvent` Enum:
        - Varianten zur Darstellung von Netzwerk-Ereignissen (z.B. Gerät verbunden, Verbindung getrennt, IP-Adresse geändert).
- **Algorithmen:**
    - Algorithmen zur Interaktion mit dem Netzwerk-Manager.
    - Algorithmen zur Umwandlung von NetworkManager-Daten in die Modul-eigenen Datenstrukturen.
    - Algorithmen zur Verarbeitung von Netzwerk-Ereignissen und zur Benachrichtigung anderer Komponenten.

#### 16.10.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verbindung mit dem Netzwerk-Manager.
    - Fehler bei der Interaktion mit dem Netzwerk-Manager.
    - Unerwartete NetworkManager-Ereignisse.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 16.10.5. Ereignisse und Callbacks

- **Ereignisse:**
    - `NetworkEvent` Enum zur Darstellung von Netzwerk-Ereignissen.
- **Callbacks:**
    - Funktionen, die von anderen Komponenten implementiert werden, um auf Netzwerk-Ereignisse zu reagieren.

#### 16.10.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung von separaten Submodulen für Netzwerkgeräte und -Verbindungen, um die Modularität zu erhöhen.
- **Alternativen:** Kombinierte Submodule.
- **Trade-offs:** Separate Submodule erhöhen die Flexibilität, können aber die Komplexität erhöhen.

#### 16.10.7. Teststrategie

- **Unit-Tests:**
    - Mocken von `ConfigServiceAsync`, `DbusEventBus`, und `Scheduler`. * Testen von `add_task()`, `update_task()`, `remove_task()`, `get_task()`, und `list_tasks()`. * Testen der Persistenz der Aufgabenliste (ggf. mit einem In-Memory-Mock für Tests). * Testen der Fehlerfälle, z.B. wenn eine Aufgabe mit einer bereits existierenden ID hinzugefügt wird. * **Integration Tests:** * Starten des NovaDE-Compositors. * Ändern einer Aufgabe über die UI. * Überprüfen, ob die Änderung korrekt im Scheduler persistiert wird und ob die Aufgabe zur richtigen Zeit ausgeführt wird.

**Geschätzter Aufwand:** Hoch (ca. 10-14 Tage, da asynchrone Operationen, Persistenz und komplexe Interaktionen mit anderen Modulen)

---

### 4. `novade-system` Schicht

Die `novade-system` Schicht ist für die Interaktion mit dem Betriebssystem, der Hardware und externen Diensten verantwortlich. Sie setzt die Richtlinien der Domänenschicht technisch um und stellt die Basis für die UI-Schicht dar.

#### 4.1. Hauptverantwortlichkeiten

- Abstraktion von Betriebssystem- und Hardware-Funktionen.
- Bereitstellung von Systemdiensten für die Domänen- und UI-Schichten.
- Implementierung von Low-Level-Funktionalitäten wie Display-Management, Eingabeverarbeitung und Audio-Management.
- Kommunikation mit externen Diensten über D-Bus und andere IPC-Mechanismen.
- Sicherstellung der Systemstabilität und Performance.

#### 4.2. Wichtige Designentscheidungen

- Verwendung von Wayland und Smithay für das Display-Management.
- Verwendung von `libinput` für die Eingabeverarbeitung.
- Verwendung von D-Bus für die Interprozesskommunikation.
- Modularer Aufbau zur Förderung der Wartbarkeit und Erweiterbarkeit.
- Asynchrone Programmierung zur Verbesserung der Reaktionsfähigkeit.

#### 4.3. Modulübersicht

Die `novade-system` Schicht ist in verschiedene Module unterteilt, die jeweils eine spezifische Funktionalität implementieren.

1. **`compositor`:** Verwaltet die Anzeige und das Rendering von Fenstern.
2. **`input`:** Verarbeitet Eingabeereignisse von Tastatur, Maus und anderen Geräten.
3. **`dbus_interfaces`:** Implementiert D-Bus-Schnittstellen für die Kommunikation mit anderen Diensten.
4. **`audio_management`:** Verwaltet die Audioausgabe und -eingabe.
5. **`mcp_client`:** Stellt eine Schnittstelle für die Kommunikation mit dem MCP-Dienst bereit.
6. **`window_mechanics`:** Implementiert Window-Management-Funktionalitäten.
7. **`power_management`:** Verwaltet die Energieeinstellungen des Systems.
8. **`portals`:** Implementiert die Freedesktop-Portals für die sichere Interaktion mit dem Desktop.

#### 4.4. Abhängigkeiten

Die `novade-system` Schicht hat Abhängigkeiten zu:

- `novade-core` für grundlegende Datentypen und Dienstprogramme.
- Externen Bibliotheken wie `wayland-client`, `wayland-server`, `smithay`, `libinput`, `dbus` und anderen Systembibliotheken.

#### 4.5. Teststrategie

Die `novade-system` Schicht erfordert umfangreiche Tests, um die korrekte Interaktion mit dem Betriebssystem und der Hardware sicherzustellen.

- **Unit-Tests:** Testen einzelner Module und Funktionen.
- **Integrationstests:** Testen der Interaktion zwischen Modulen und mit externen Diensten.
- **Systemtests:** Testen des Gesamtsystems auf einem realen System.

#### 4.6. Detaillierte Modulspezifikationen

Die folgenden Abschnitte enthalten detaillierte Spezifikationen für jedes Modul der `novade-system` Schicht.

---

### 5. `novade-system/src/compositor` Modul

Das `novade-system/src/compositor` Modul ist für die Verwaltung der Anzeige und des Renderings von Fenstern verantwortlich. Es implementiert einen Wayland-Compositor basierend auf der `smithay`-Bibliothek.

#### 5.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung von Wayland-Clients und deren Oberflächen.
    - Komposition der Fensterinhalte.
    - Rendering der Ausgabe auf den Bildschirm.
    - Verarbeitung von Eingabeereignissen.
    - Implementierung von Wayland-Protokollen.
- **Grenzen:**
    - Die eigentliche Fensterverwaltung (z.B. Platzierung, Größe) wird von der `novade-domain` Schicht gesteuert.
    - Das Rendern der Fensterinhalte erfolgt durch die Clients selbst.

#### 5.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-domain`: Für die Steuerung der Fensterverwaltung.
- `novade-system/src/input`: Für die Verarbeitung von Eingabeereignissen.
- `novade-system/src/dbus_interfaces`: Für die Kommunikation mit anderen Diensten.

#### 5.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `Client`: Repräsentiert einen Wayland-Client.
    - `Surface`: Repräsentiert eine von einem Client bereitgestellte Oberfläche.
    - `Output`: Repräsentiert einen angeschlossenen Bildschirm.
    - `CompositorState`: Verwaltet den Zustand des Compositors.
- **Algorithmen:**
    - Algorithmen zur Komposition der Fensterinhalte.
    - Algorithmen zur Verarbeitung von Eingabeereignissen.
    - Algorithmen zur Implementierung der Wayland-Protokolle.

#### 5.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler beim Verbinden mit Wayland-Clients.
    - Ungültige Wayland-Protokollnachrichten.
    - Fehler beim Rendern der Ausgabe.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Wayland-Protokollereignisse.
    - Eingabeereignisse.
    - Client-Verbindungsereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Ereignisse.

#### 5.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung von `smithay` als Wayland-Implementierung.
- **Alternativen:** Andere Wayland-Implementierungen.
- **Trade-offs:** `smithay` bietet eine hohe Flexibilität, kann aber komplex sein.

#### 5.7. Teststrategie

- **Unit-Tests:**
    - Testen einzelner Funktionen und Algorithmen.
    - Mocken von Wayland-Clients und -Server.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Modulen.
    - Testen der Implementierung der Wayland-Protokolle.
- **Systemtests:**
    - Testen des Compositors auf einem realen System.

#### 5.8. Submodule

Das `compositor` Modul ist in mehrere Submodule unterteilt:

- `core`: Implementiert die Kernfunktionalität des Compositors.
- `state`: Verwaltet den Zustand des Compositors.
- `handlers`: Verarbeitet Wayland-Protokollnachrichten.
- `protocols`: Implementiert Wayland-Protokolle.
- `render`: Implementiert das Rendering der Ausgabe.

#### 5.9. Detaillierte Submodulspezifikationen

Die folgenden Abschnitte enthalten detaillierte Spezifikationen für jedes Submodul des `compositor` Moduls.

---

### 5.10. `novade-system/src/compositor/core.rs`

#### 5.10.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Initialisierung des Wayland-Compositors.
    - Verwaltung der Display-Ausgabe.
    - Erstellung von Wayland-Listenern.
    - Ausführung der Haupt-Ereignisschleife.
- **Grenzen:**
    - Die Implementierung der Wayland-Protokolle erfolgt in anderen Submodulen.

#### 5.10.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.
- `novade-system/src/compositor/handlers`: Für die Verarbeitung von Wayland-Nachrichten.

#### 5.10.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `CompositorState`: (siehe `novade-system/src/compositor/state.rs`)
    - `Display`: Repräsentiert den Wayland-Display.
    - `EventLoop`: Die Haupt-Ereignisschleife von `smithay`.
- **Algorithmen:**
    - Algorithmen zur Initialisierung des Wayland-Compositors.
    - Algorithmen zur Verarbeitung der Ereignisschleife.

#### 5.10.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Initialisierung des Wayland-Compositors.
    - Fehler bei der Ausführung der Ereignisschleife.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.10.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Wayland-Ereignisse.
- **Callbacks:**
    - Callbacks zur Verarbeitung von Wayland-Ereignissen.

#### 5.10.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung von `smithay`s `EventLoop` für die Ereignisverarbeitung.
- **Alternativen:** Eigene Implementierung einer Ereignisschleife.
- **Trade-offs:** `smithay`s `EventLoop` bietet eine gute Integration mit Wayland, kann aber komplex sein.

#### 5.10.7. Teststrategie

- **Unit-Tests:**
    - Testen der Initialisierung des Wayland-Compositors.
    - Mocken von Wayland-Clients und -Server.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Submodulen.

---

### 5.11. `novade-system/src/compositor/state.rs`

#### 5.11.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung des Zustands des Wayland-Compositors.
    - Speicherung von Informationen über Clients, Oberflächen und Ausgaben.
    - Bereitstellung von Zugriff auf den Zustand für andere Submodule.
- **Grenzen:**
    - Die eigentliche Logik zur Verarbeitung von Wayland-Nachrichten wird in anderen Submodulen implementiert.

#### 5.11.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/core`: Für den Zugriff auf die Ereignisschleife und den Display.
- Andere Submodule des `compositor` Moduls: Für den Zugriff auf den Compositor-Zustand.

#### 5.11.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `CompositorState`:
        - `clients: Vec<Client>`: Liste der verbundenen Wayland-Clients.
        - `surfaces: Vec<Surface>`: Liste der verwalteten Oberflächen.
        - `outputs: Vec<Output>`: Liste der angeschlossenen Bildschirme.
        - Weitere Zustandsinformationen.
- **Algorithmen:**
    - Algorithmen zum Verwalten des Zustands.
    - Algorithmen zum Abrufen von Informationen aus dem Zustand.

#### 5.11.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler beim Zugriff auf den Zustand.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.11.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Keine spezifischen Ereignisse sind definiert.
- **Callbacks:**
    - Keine spezifischen Callbacks sind definiert.

#### 5.11.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung einer zentralen `CompositorState`-Struktur zur Verwaltung des Zustands.
- **Alternativen:** Verteilte Zustandsverwaltung.
- **Trade-offs:** Eine zentrale Struktur ist einfacher zu verwalten, kann aber zu einem Engpass werden.

#### 5.11.7. Teststrategie

- **Unit-Tests:**
    - Testen der Datenstrukturen.
    - Testen der Algorithmen zum Verwalten des Zustands.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Submodulen.

---

### 5.12. `novade-system/src/compositor/handlers.rs`

#### 5.12.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verarbeitung von Wayland-Protokollnachrichten.
    - Implementierung der Logik für die Wayland-Schnittstellen.
    - Interaktion mit dem Compositor-Zustand.
- **Grenzen:**
    - Die Initialisierung des Compositors und die Ausführung der Ereignisschleife erfolgen in anderen Submodulen.

#### 5.12.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.
- `novade-system/src/compositor/protocols`: Für die Implementierung der Wayland-Protokolle.

#### 5.12.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Wayland-Protokollnachrichten.
    - Zustandsinformationen (aus `CompositorState`).
- **Algorithmen:**
    - Algorithmen zur Verarbeitung von Wayland-Nachrichten.
    - Algorithmen zur Implementierung der Wayland-Schnittstellen.

#### 5.12.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige Wayland-Nachrichten.
    - Fehler bei der Verarbeitung von Wayland-Nachrichten.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.12.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Wayland-Ereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Wayland-Ereignisse.

#### 5.12.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung der Wayland-Protokolllogik in separaten Handler-Funktionen.
- **Alternativen:** Inline-Implementierung der Protokolllogik.
- **Trade-offs:** Separate Handler-Funktionen verbessern die Modularität, können aber die Komplexität erhöhen.

#### 5.12.7. Teststrategie

- **Unit-Tests:**
    - Testen einzelner Handler-Funktionen.
    - Mocken von Wayland-Clients und -Server.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Submodulen.
    - Testen der Implementierung der Wayland-Protokolle.

---

### 5.13. `novade-system/src/compositor/protocols`

Dieses Submodul enthält die Implementierungen der verschiedenen Wayland-Protokolle.

#### 5.13.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung der Logik für die Wayland-Protokolle.
    - Bereitstellung von Schnittstellen für die Interaktion mit den Protokollen.

---

### 5.13.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/handlers`: Für die Verarbeitung von Wayland-Nachrichten.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.

### 5.13.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Datenstrukturen zur Repräsentation der Wayland-Protokolle.
- **Algorithmen:**
    - Algorithmen zur Implementierung der Logik für die Wayland-Protokolle.

### 5.13.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Implementierung der Wayland-Protokolle.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

### 5.13.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Wayland-Protokollereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Wayland-Protokollereignisse.

### 5.13.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung der Protokolle in separaten Modulen.
- **Alternativen:** Implementierung der Protokolle in einem einzigen Modul.
- **Trade-offs:** Separate Module verbessern die Modularität, können aber die Komplexität erhöhen.

### 5.13.7. Teststrategie

- **Unit-Tests:**
    - Testen der Implementierung einzelner Protokolle.
    - Mocken von Wayland-Clients und -Server.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Submodulen.

### 5.13.8. Submodule

Das `protocols` Modul ist in mehrere Submodule unterteilt, die jeweils ein spezifisches Wayland-Protokoll implementieren.

- `wl_compositor`: Implementiert das `wl_compositor` Protokoll.
- `wl_shm`: Implementiert das `wl_shm` Protokoll.
- `wl_output`: Implementiert das `wl_output` Protokoll.
- `xdg_shell`: Implementiert das `xdg_shell` Protokoll.
- `layer_shell`: Implementiert das `layer_shell` Protokoll.
- Weitere Protokolle.

### 5.13.9. Detaillierte Submodulspezifikationen

Die folgenden Abschnitte enthalten detaillierte Spezifikationen für einige der wichtigsten Submodule des `protocols` Moduls.

---

### 5.14. `novade-system/src/compositor/protocols/wl_compositor.rs`

#### 5.14.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung des `wl_compositor` Protokolls.
    - Erstellung von Oberflächen (`wl_surface`).
- **Grenzen:**
    - Die eigentliche Verwaltung der Oberflächen erfolgt in anderen Modulen.

#### 5.14.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/handlers`: Für die Verarbeitung von Wayland-Nachrichten.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.

#### 5.14.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `wl_compositor`-Protokollnachrichten.
- **Algorithmen:**
    - Algorithmen zur Implementierung des `wl_compositor`-Protokolls.
    - Algorithmen zur Erstellung von Oberflächen.

#### 5.14.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige `wl_compositor`-Nachrichten.
    - Fehler bei der Erstellung von Oberflächen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.14.5. Ereignisse und Callbacks

- **Ereignisse:**
    - `wl_compositor`-Ereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über `wl_compositor`-Ereignisse.

#### 5.14.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung des `wl_compositor`-Protokolls in einem separaten Modul.
- **Alternativen:** Implementierung des Protokolls im `handlers`-Modul.
- **Trade-offs:** Ein separates Modul verbessert die Modularität, kann aber die Komplexität erhöhen.

#### 5.14.7. Teststrategie

- **Unit-Tests:**
    - Testen der Implementierung des `wl_compositor`-Protokolls.
    - Mocken von Wayland-Clients und -Server.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Submodulen.

---

### 5.15. `novade-system/src/compositor/protocols/wl_shm.rs`

#### 5.15.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung des `wl_shm` Protokolls.
    - Verwaltung von Shared Memory Buffers.
- **Grenzen:**
    - Die eigentliche Verwendung der Shared Memory Buffers erfolgt in den Clients.

#### 5.15.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/handlers`: Für die Verarbeitung von Wayland-Nachrichten.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.

#### 5.15.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `wl_shm`-Protokollnachrichten.
    - Shared Memory Buffers.
- **Algorithmen:**
    - Algorithmen zur Implementierung des `wl_shm`-Protokolls.
    - Algorithmen zur Verwaltung von Shared Memory Buffers.

#### 5.15.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige `wl_shm`-Nachrichten.
    - Fehler bei der Verwaltung von Shared Memory Buffers.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.15.5. Ereignisse und Callbacks

- **Ereignisse:**
    - `wl_shm`-Ereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über `wl_shm`-Ereignisse.

#### 5.15.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung des `wl_shm`-Protokolls in einem separaten Modul.
- **Alternativen:** Implementierung des Protokolls im `handlers`-Modul.
- **Trade-offs:** Ein separates Modul verbessert die Modularität, kann aber die Komplexität erhöhen.

#### 5.15.7. Teststrategie

- **Unit-Tests:**
    - Testen der Implementierung des `wl_shm`-Protokolls.
    - Mocken von Wayland-Clients und -Server.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Submodulen.

---

### 5.16. `novade-system/src/compositor/protocols/wl_output.rs`

#### 5.16.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung des `wl_output` Protokolls.
    - Verwaltung von Bildschirmausgaben.
- **Grenzen:**
    - Die eigentliche Konfiguration der Bildschirme erfolgt in anderen Modulen.

#### 5.16.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/handlers`: Für die Verarbeitung von Wayland-Nachrichten.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.

#### 5.16.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `wl_output`-Protokollnachrichten.
    - Informationen über Bildschirmausgaben.
- **Algorithmen:**
    - Algorithmen zur Implementierung des `wl_output`-Protokolls.
    - Algorithmen zur Verwaltung von Bildschirmausgaben.

#### 5.16.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige `wl_output`-Nachrichten.
    - Fehler bei der Verwaltung von Bildschirmausgaben.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.16.5. Ereignisse und Callbacks

- **Ereignisse:**
    - `wl_output`-Ereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über `wl_output`-Ereignisse.

#### 5.16.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung des `wl_output`-Protokolls in einem separaten Modul.
- **Alternativen:** Implementierung des Protokolls im `handlers`-Modul.
- **Trade-offs:** Ein separates Modul verbessert die Modularität, kann aber die Komplexität erhöhen.

#### 5.16.7. Teststrategie

- **Unit-Tests:**
    - Testen der Implementierung des `wl_output`-Protokolls.
    - Mocken von Wayland-Clients und -Server.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Submodulen.

---

### 5.17. `novade-system/src/compositor/protocols/xdg_shell.rs`

#### 5.17.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung des `xdg_shell` Protokolls.
    - Verwaltung von Desktop-spezifischen Oberflächen.
- **Grenzen:**
    - Die eigentliche Fensterverwaltung erfolgt in der `novade-domain` Schicht.

#### 5.17.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/handlers`: Für die Verarbeitung von Wayland-Nachrichten.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.
- `novade-domain`: Für die Steuerung der Fensterverwaltung.

#### 5.17.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `xdg_shell`-Protokollnachrichten.
    - Informationen über Desktop-spezifische Oberflächen.
- **Algorithmen:**
    - Algorithmen zur Implementierung des `xdg_shell`-Protokolls.
    - Algorithmen zur Verwaltung von Desktop-spezifischen Oberflächen.

#### 5.17.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige `xdg_shell`-Nachrichten.
    - Fehler bei der Verwaltung von Desktop-spezifischen Oberflächen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.17.5. Ereignisse und Callbacks

- **Ereignisse:**
    - `xdg_shell`-Ereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über `xdg_shell`-Ereignisse.

#### 5.17.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung des `xdg_shell`-Protokolls in einem separaten Modul.
- **Alternativen:** Implementierung des Protokolls im `handlers`-Modul.
- **Trade-offs:** Ein separates Modul verbessert die Modularität, kann aber die Komplexität erhöhen.

#### 5.17.7. Teststrategie

- **Unit-Tests:**
    - Testen der Implementierung des `xdg_shell`-Protokolls.
    - Mocken von Wayland-Clients und -Server.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Submodulen.

---

### 5.18. `novade-system/src/compositor/protocols/layer_shell.rs`

#### 5.18.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Implementierung des `layer_shell` Protokolls.
    - Verwaltung von Oberflächen, die auf Layern angeordnet sind (z.B. Panels, Benachrichtigungen).
- **Grenzen:**
    - Die eigentliche Platzierung der Oberflächen erfolgt in der `novade-domain` Schicht.

#### 5.18.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/handlers`: Für die Verarbeitung von Wayland-Nachrichten.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.
- `novade-domain`: Für die Steuerung der Platzierung der Oberflächen.

#### 5.18.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - `layer_shell`-Protokollnachrichten.
    - Informationen über Oberflächen, die auf Layern angeordnet sind.
- **Algorithmen:**
    - Algorithmen zur Implementierung des `layer_shell`-Protokolls.
    - Algorithmen zur Verwaltung von Oberflächen, die auf Layern angeordnet sind.

#### 5.18.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Ungültige `layer_shell`-Nachrichten.
    - Fehler bei der Verwaltung von Oberflächen, die auf Layern angeordnet sind.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.18.5. Ereignisse und Callbacks

- **Ereignisse:**
    - `layer_shell`-Ereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über `layer_shell`-Ereignisse.

#### 5.18.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung des `layer_shell`-Protokolls in einem separaten Modul.
- **Alternativen:** Implementierung des Protokolls im `handlers`-Modul.
- **Trade-offs:** Ein separates Modul verbessert die Modularität, kann aber die Komplexität erhöhen.

#### 5.18.7. Teststrategie

- **Unit-Tests:**
    - Testen der Implementierung des `layer_shell`-Protokolls.
    - Mocken von Wayland-Clients und -Server.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Submodulen.

---

### 5.19. `novade-system/src/compositor/render.rs`

#### 5.19.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Rendering der Ausgabe auf den Bildschirm.
    - Verwaltung von OpenGL-Kontexten und -Ressourcen.
- **Grenzen:**
    - Das Rendern der Fensterinhalte erfolgt durch die Clients selbst.

#### 5.19.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.

#### 5.19.3. Datenstrukturen und Algorithmen
- **Algorithmen:**
    - Algorithmen für das Rendering der Ausgabe.
    - Algorithmen zur Verwaltung von OpenGL-Kontexten und -Ressourcen.

#### 5.19.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Initialisierung von OpenGL.
    - Fehler beim Rendern der Ausgabe.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.19.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Keine spezifischen Ereignisse sind definiert.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Rendering-Ereignisse.

#### 5.19.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung von OpenGL für das Rendering.
- **Alternativen:** Verwendung von Vulkan.
- **Trade-offs:** OpenGL ist ausgereifter, Vulkan bietet potenziell bessere Performance.

#### 5.19.7. Teststrategie

- **Integrationstests:**
    - Testen des Renderings der Ausgabe auf den Bildschirm.
    - Vergleich von gerenderten Bildern mit Referenzbildern.

---

### 5.20. `novade-system/src/compositor/input.rs`

#### 5.20.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verarbeitung von Eingabeereignissen (Tastatur, Maus, Touch).
    - Weiterleitung der Ereignisse an die entsprechenden Clients.
- **Grenzen:**
    - Die eigentliche Interpretation der Eingabe erfolgt in den Clients.

#### 5.20.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.

#### 5.20.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Eingabeereignisse (Tastatur, Maus, Touch).
- **Algorithmen:**
    - Algorithmen zur Verarbeitung von Eingabeereignissen.
    - Algorithmen zur Weiterleitung der Ereignisse an die entsprechenden Clients.

#### 5.20.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verarbeitung von Eingabeereignissen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.20.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Eingabeereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Eingabeereignisse.

#### 5.20.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung von `libinput` für die Eingabeverarbeitung.
- **Alternativen:** Direkte Verwendung von Kernel-Ereignissen.
- **Trade-offs:** `libinput` bietet eine Abstraktionsebene, kann aber die Komplexität erhöhen.

#### 5.20.7. Teststrategie

- **Unit-Tests:**
    - Testen der Verarbeitung einzelner Eingabeereignisse.
    - Mocken von Eingabegeräten.
- **Integrationstests:**
    - Testen der Weiterleitung von Eingabeereignissen an Clients.

---

### 5.21. `novade-system/src/compositor/seat.rs`

#### 5.21.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung von "Seats" (Gruppen von Eingabe- und Ausgabegeräten).
    - Fokusverwaltung.
- **Grenzen:**
    - Die eigentliche Verarbeitung der Eingabe erfolgt im `input`-Modul.
    - Das Rendern der Ausgabe erfolgt im `render`-Modul.

#### 5.21.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/input`: Für die Verarbeitung von Eingabeereignissen.
- `novade-system/src/compositor/render`: Für das Rendern der Ausgabe.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.

#### 5.21.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Informationen über Seats.
    - Informationen über den Eingabefokus.
- **Algorithmen:**
    - Algorithmen zur Verwaltung von Seats.
    - Algorithmen zur Fokusverwaltung.

#### 5.21.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Verwaltung von Seats.
    - Fehler bei der Fokusverwaltung.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.21.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Seat-bezogene Ereignisse.
    - Fokusänderungsereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Seat-bezogene Ereignisse und Fokusänderungen.

#### 5.21.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung der Seat-Verwaltung in einem separaten Modul.
- **Alternativen:** Implementierung der Seat-Verwaltung im `input`- oder `render`-Modul.
- **Trade-offs:** Ein separates Modul verbessert die Modularität, kann aber die Komplexität erhöhen.

#### 5.21.7. Teststrategie

- **Unit-Tests:**
    - Testen der Verwaltung von Seats.
    - Testen der Fokusverwaltung.
    - Mocken von Eingabe- und Ausgabegeräten.
- **Integrationstests:**
    - Testen der Interaktion mit dem `input`- und `render`-Modul.

---

### 5.22. `novade-system/src/compositor/xwayland.rs`

#### 5.22.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Unterstützung für X11-Anwendungen.
    - Ausführung eines XWayland-Servers.
- **Grenzen:**
    - Die eigentliche Darstellung der X11-Fenster erfolgt über Wayland-Oberflächen.

#### 5.22.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor/state`: Für den Zugriff auf den Compositor-Zustand.

#### 5.22.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - X11-Fenster.
    - Wayland-Oberflächen.
- **Algorithmen:**
    - Algorithmen zur Ausführung des XWayland-Servers.
    - Algorithmen zur Darstellung von X11-Fenstern über Wayland-Oberflächen.

#### 5.22.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler beim Starten des XWayland-Servers.
    - Fehler bei der Darstellung von X11-Fenstern.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.22.5. Ereignisse und Callbacks

- **Ereignisse:**
    - X11-Fensterereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über X11-Fensterereignisse.

#### 5.22.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Unterstützung für X11-Anwendungen über XWayland.
- **Alternativen:** Keine Unterstützung für X11-Anwendungen.
- **Trade-offs:** XWayland ermöglicht die Ausführung älterer Anwendungen, kann aber die Komplexität erhöhen.

#### 5.22.7. Teststrategie

- **Integrationstests:**
    - Testen der Ausführung von X11-Anwendungen unter NovaDE.
    - Testen der Interaktion mit X11-Anwendungen.

---

### 5.23. `novade-system/src/compositor/state.rs`

#### 5.23.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung des globalen Zustands des Compositors.
    - Bereitstellung von Zugriff auf den Zustand für andere Module.
- **Grenzen:**
    - Die eigentliche Logik der Compositor-Funktionen ist in anderen Modulen implementiert.

#### 5.23.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- Alle anderen Submodule des `compositor`-Moduls.

#### 5.23.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Globale Zustandsinformationen des Compositors (z.B. Liste der Oberflächen, Liste der Ausgaben, Eingabefokus).
- **Algorithmen:**
    - Algorithmen zur Verwaltung des globalen Zustands.
    - Algorithmen zur Bereitstellung von Zugriff auf den Zustand.

#### 5.23.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler beim Zugriff auf den globalen Zustand.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.23.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Zustandsänderungsereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Zustandsänderungen.

#### 5.23.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwaltung des globalen Zustands in einem separaten Modul.
- **Alternativen:** Verteilung des Zustands auf andere Module.
- **Trade-offs:** Ein separates Modul verbessert die Übersichtlichkeit, kann aber die Kopplung erhöhen.

#### 5.23.7. Teststrategie

- **Unit-Tests:**
    - Testen der Verwaltung des globalen Zustands.
    - Testen der Bereitstellung von Zugriff auf den Zustand.
- **Integrationstests:**
    - Testen der Interaktion mit anderen Submodulen.

---

### 5.24. `novade-system/src/compositor/mod.rs`

#### 5.24.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Zusammenführung der Funktionalität der Submodule.
    - Bereitstellung der öffentlichen API des Compositor-Moduls.
- **Grenzen:**
    - Die eigentliche Implementierung der Compositor-Funktionen erfolgt in den Submodulen.

#### 5.24.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- Andere Module der `novade-system` Schicht.

#### 5.24.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Keine spezifischen Datenstrukturen.
- **Algorithmen:**
    - Algorithmen zur Zusammenführung der Funktionalität der Submodule.
    - Algorithmen zur Bereitstellung der öffentlichen API.

#### 5.24.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler bei der Zusammenführung der Funktionalität der Submodule.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.24.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Keine spezifischen Ereignisse sind definiert.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Compositor-Ereignisse.

#### 5.24.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung eines Moduls zur Zusammenführung der Funktionalität der Submodule.
- **Alternativen:** Keine Verwendung eines solchen Moduls.
- **Trade-offs:** Ein solches Modul verbessert die Übersichtlichkeit, kann aber die Komplexität erhöhen.

#### 5.24.7. Teststrategie

- **Integrationstests:**
    - Testen der Interaktion mit anderen Modulen der `novade-system` Schicht.

---

### 5.25. `novade-system/src/display_manager_service.rs`

#### 5.25.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung von Displays (Monitoren).
    - Setzen von Auflösung, Bildwiederholrate und anderen Display-Einstellungen.
    - Unterstützung für Multi-Monitor-Setups.
    - Implementierung des `wlr-output-management-unstable-v1`-Protokolls.14
- **Grenzen:**
    - Das Rendern der Inhalte auf den Displays erfolgt durch den Compositor.

#### 5.25.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor`: Für die Interaktion mit dem Compositor.
- `novade-system/src/state`: Für den Zugriff auf den globalen Zustand.

#### 5.25.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Informationen über Displays (Name, Auflösung, Bildwiederholrate, etc.).
    - Konfigurationen für Multi-Monitor-Setups.
- **Algorithmen:**
    - Algorithmen zum Abrufen von Display-Informationen.
    - Algorithmen zum Setzen von Display-Einstellungen.
    - Algorithmen zur Verwaltung von Multi-Monitor-Setups.

#### 5.25.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler beim Abrufen von Display-Informationen.
    - Fehler beim Setzen von Display-Einstellungen.
    - Fehler bei der Verwaltung von Multi-Monitor-Setups.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.25.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Ereignisse im Zusammenhang mit Display-Änderungen (z.B. Display hinzugefügt/entfernt, Einstellungen geändert).
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Display-Änderungen.

#### 5.25.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung der Display-Verwaltung in einem separaten Dienst.
- **Alternativen:** Integration der Display-Verwaltung in den Compositor.
- **Trade-offs:** Ein separater Dienst verbessert die Modularität und ermöglicht potenziell einen einfacheren Austausch der Display-Verwaltung, kann aber die Komplexität erhöhen.

#### 5.25.7. Teststrategie

- **Unit-Tests:**
    - Testen der Funktionen zum Abrufen und Setzen von Display-Einstellungen.
    - Mocken von Hardware-Abhängigkeiten.
- **Integrationstests:**
    - Testen der Interaktion mit dem Compositor und anderen Diensten.
    - Testen von Multi-Monitor-Szenarien.

#### 5.25.8. Detaillierte Spezifikation des `wlr-output-management-unstable-v1` Protokolls

Dieses Modul implementiert die serverseitige Logik für das `wlr-output-management-unstable-v1`-Protokoll, das es Clients (wie Kanshi 14) ermöglicht, Display-Konfigurationen abzufragen und zu ändern.14

- **Dateien**: `system/display_manager_service/wlr_output_management_handler.rs`
    
- **Abhängigkeiten**:
    
    - Smithay-Bibliotheken für Wayland-Protokollimplementierung
    - `zwlr_output_manager_v1` und verwandte Schnittstellen (aus `wayland-protocols`)
    - Compositor-Zustand für den Zugriff auf aktive Ausgaben
- **Spezifikation**:
    - **Funktionalität**:
        - Bereitstellung von Informationen über verbundene Ausgaben (Namen, Modi, Positionen, Skalierung, etc.).
        - Ermöglichung von Änderungen an diesen Einstellungen (Modus setzen, Position ändern, Skalierung ändern, Output aktivieren/deaktivieren).
        - Unterstützung für das Speichern und Laden von Output-Konfigurationen.
        - Signalübertragung von Änderungen an verbundene Clients.
    - **Wayland-Protokollinteraktion**:
        - Implementierung der `zwlr_output_manager_v1`-Schnittstelle.
        - Verwendung der zugehörigen Wayland-Typen (z.B. `zwlr_output_configuration_v1`, `zwlr_output_mode_v1`, etc.).
        - Verarbeitung von Anfragen von Clients (z.B. `get_outputs`, `set_mode`, `enable`, `disable`, `apply`).
        - Senden von Ereignissen an Clients (z.B. `output_added`, `output_removed`, `mode_changed`).
    - **Datenstrukturen**:
        - Strukturen zur Repräsentation von Output-Informationen (Name, Modus, Position, Skalierung, etc.).
        - Strukturen zur Repräsentation von Output-Konfigurationen.
        - Datenstrukturen für die Kommunikation mit dem Wayland-Protokoll.
    - **Algorithmen**:
        - Algorithmen zur Verarbeitung von Client-Anfragen.
        - Algorithmen zur Anwendung von Output-Konfigurationen.
        - Algorithmen zur Signalübertragung von Änderungen an Clients.
    - **Fehlerbehandlung**:
        - Definition von Fehlerfällen (z.B. ungültige Modus-Einstellungen, nicht unterstützte Konfigurationen).
        - Verwendung von `Result` und Logging zur Fehlerbehandlung.
    - **Teststrategie**:
        - Unit-Tests für einzelne Funktionen.
        - Integrationstests zur Überprüfung der Interaktion mit dem Compositor und Clients.
        - Manuelle Tests mit verschiedenen Hardware-Konfigurationen.

---

### 5.26. `novade-system/src/session_manager.rs`

#### 5.26.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung von Benutzer-Sessions (Anmeldung, Abmeldung, Sitzungssperre).
    - Starten der Kernkomponenten des Desktops (Compositor, Shell, etc.).
    - Verwaltung von Umgebungsvariablen.
    - Implementierung der `org.freedesktop.login1` D-Bus-Schnittstelle.8
- **Grenzen:**
    - Die eigentliche Authentifizierung des Benutzers erfolgt durch PAM.2
    - Die Darstellung der Benutzeroberfläche erfolgt durch den Compositor und die Shell.

#### 5.26.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- `novade-system/src/compositor`: Für das Starten des Compositors.
- `novade-system/src/shell`: Für das Starten der Shell.
- D-Bus: Für die Kommunikation mit anderen Diensten (z.B. `logind`).8

#### 5.26.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Informationen über die aktuelle Sitzung.
    - Informationen über den aktuellen Benutzer.
- **Algorithmen:**
    - Algorithmen für den Anmelde- und Abmeldeprozess.
    - Algorithmen zum Starten der Desktop-Komponenten.
    - Algorithmen zur Verwaltung von Umgebungsvariablen.

#### 5.26.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler beim Starten der Desktop-Komponenten.
    - Fehler bei der Verwaltung der Sitzung.
    - Authentifizierungsfehler.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.26.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Sitzungsbezogene Ereignisse (z.B. Sitzung gestartet, Sitzung beendet).
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Sitzungsereignisse.

#### 5.26.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung von `logind` zur Verwaltung von Sitzungen.8
- **Alternativen:** Eigene Implementierung der Sitzungsverwaltung.
- **Trade-offs:** `logind` bietet eine standardisierte Schnittstelle, kann aber die Komplexität erhöhen.

#### 5.26.7. Teststrategie

- **Unit-Tests:**
    - Testen der Funktionen zum Starten und Beenden von Sitzungen.
    - Mocken von D-Bus-Interaktionen.
- **Integrationstests:**
    - Testen des gesamten An- und Abmeldeprozesses.
    - Testen der Interaktion mit dem Compositor und der Shell.

#### 5.26.8. Detaillierte Spezifikation der `org.freedesktop.login1` D-Bus-Schnittstelle.8

- **Dateien**: `system/session_manager/logind_handler.rs`
    
- **Abhängigkeiten**:
    
    - `zbus` für D-Bus-Interaktion
    - `org.freedesktop.login1` Proxy-Objekt (generiert aus Introspektionsdaten)
- **Spezifikation**:
    
    - **Funktionalität**:
        - Bereitstellung von Methoden zur Steuerung von Sitzungen (z.B. `LockSession`, `UnlockSession`, `TerminateSession`, `PrepareForShutdown`).
        - Bereitstellung von Signalen zur Benachrichtigung über Sitzungsänderungen (z.B. `SessionNew`, `SessionRemoved`).
        - Verwaltung von Benutzern und deren Sitzungen.
    - **D-Bus-Interaktion**:
        - Implementierung der Methoden der `org.freedesktop.login1`-Schnittstelle.
        - Senden der Signale der `org.freedesktop.login1`-Schnittstelle.
        - Verwendung von `zbus` zur Registrierung des Dienstes auf dem System-Bus.
    - **Datenstrukturen**:
        - Datenstrukturen zur Repräsentation von Sitzungsinformationen.
        - Datenstrukturen zur Repräsentation von Benutzerinformationen.
        - D-Bus-spezifische Datentypen.
    - **Algorithmen**:
        - Algorithmen zur Verarbeitung von D-Bus-Methodenaufrufen.
        - Algorithmen zur Verwaltung von Sitzungen und Benutzern.
        - Algorithmen zur Signalübertragung.
    - **Fehlerbehandlung**:
        - Definition von Fehlerfällen (z.B. ungültige Sitzungs-IDs, fehlgeschlagene Authentifizierung).
        - Verwendung von D-Bus-Fehlermeldungen.
    - **Teststrategie**:
        - Unit-Tests für einzelne D-Bus-Methoden.
        - Integrationstests zur Überprüfung der Interaktion mit `logind`.

---

### 5.27. `novade-system/src/settings_daemon.rs`

#### 5.27.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Verwaltung von globalen und benutzerspezifischen Einstellungen (z.B. Theme, Schriftarten, Eingabegeräte, Monitoreinstellungen).
    - Bereitstellung von Einstellungen für Anwendungen und andere Dienste über D-Bus.9
    - Überwachung von Einstellungsänderungen und Benachrichtigung von Clients.
- **Grenzen:**
    - Die eigentliche Anwendung der Einstellungen erfolgt in den jeweiligen Anwendungen und Diensten.

#### 5.27.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- D-Bus: Für die Bereitstellung von Einstellungen und die Kommunikation mit Clients.9
- Konfigurationsdateien: Für das Speichern und Laden von Einstellungen.

#### 5.27.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Repräsentation von Einstellungen.
    - Datenstrukturen für die Kommunikation mit D-Bus.
- **Algorithmen:**
    - Algorithmen zum Laden und Speichern von Einstellungen.
    - Algorithmen zur Bereitstellung von Einstellungen über D-Bus.
    - Algorithmen zur Überwachung von Einstellungsänderungen und Benachrichtigung von Clients.

#### 5.27.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler beim Laden oder Speichern von Einstellungen.
    - Fehler bei der Bereitstellung von Einstellungen über D-Bus.
    - Ungültige Einstellungen.
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.27.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Einstellungsänderungsereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Einstellungsänderungen.

#### 5.27.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Verwendung von D-Bus zur Bereitstellung von Einstellungen.9
- **Alternativen:** Verwendung anderer IPC-Mechanismen.
- **Trade-offs:** D-Bus ist ein etablierter Standard, kann aber die Komplexität erhöhen.

#### 5.27.7. Teststrategie

- **Unit-Tests:**
    - Testen der Funktionen zum Laden und Speichern von Einstellungen.
    - Testen der Bereitstellung von Einstellungen über D-Bus.
    - Mocken von D-Bus-Interaktionen und Dateisystemzugriffen.
- **Integrationstests:**
    - Testen der Interaktion mit Anwendungen und anderen Diensten.

#### 5.27.8. Detaillierte Spezifikation der Einstellungsverwaltung

- **Dateien**:
    
    - `system/settings_daemon/settings_manager.rs` (Kernlogik)
    - `system/settings_daemon/d_bus_interface.rs` (D-Bus-Schnittstelle)
    - `system/settings_daemon/config.rs` (Konfigurationsverwaltung)
- **Abhängigkeiten**:
    
    - `zbus` für D-Bus-Interaktion
    - `serde` für Serialisierung/Deserialisierung von Einstellungen
    - Konfigurationsdateien (z.B. TOML, JSON)
- **Spezifikation**:
    
    - **Funktionalität**:
        - Laden von Standardeinstellungen aus Konfigurationsdateien.
        - Bereitstellung von Methoden zum Abrufen und Ändern von Einstellungen über D-Bus.
        - Speichern von geänderten Einstellungen in Konfigurationsdateien.
        - Unterstützung für verschiedene Datentypen (z.B. Integer, String, Boolean, Array).
        - Unterstützung für globale und benutzerspezifische Einstellungen.
        - Benachrichtigung von Clients über Einstellungsänderungen.
    - **D-Bus-Interaktion**:
        - Implementierung einer D-Bus-Schnittstelle zur Bereitstellung von Einstellungen.
        - Verwendung von `zbus` zur Registrierung des Dienstes auf dem Session-Bus.
        - Definition von Methoden zum Abrufen (`GetSetting`, `GetAllSettings`) und Ändern (`SetSetting`) von Einstellungen.
        - Definition von Signalen zur Benachrichtigung über Einstellungsänderungen (`SettingChanged`).
    - **Konfigurationsverwaltung**:
        - Verwendung eines flexiblen Konfigurationsformats (z.B. TOML, JSON).
        - Unterstützung für das Laden von Konfigurationen aus verschiedenen Quellen (z.B. Systemverzeichnis, Benutzerverzeichnis).
        - Verwaltung von Standardeinstellungen und Benutzerüberschreibungen.
        - Serialisierung und Deserialisierung von Einstellungen.
    - **Datenstrukturen**:
        - `SettingValue` (Enum zur Repräsentation verschiedener Datentypen).
        - `Setting` (Struktur zur Repräsentation einer einzelnen Einstellung).
        - `SettingsGroup` (Struktur zur Gruppierung von Einstellungen).
        - Datenstrukturen für die D-Bus-Kommunikation.
    - **Algorithmen**:
        - Algorithmen zum Laden und Speichern von Einstellungen.
        - Algorithmen zur Verarbeitung von D-Bus-Methodenaufrufen.
        - Algorithmen zur Serialisierung und Deserialisierung von Einstellungen.
        - Algorithmen zur Benachrichtigung von Clients über Einstellungsänderungen.
    - **Fehlerbehandlung**:
        - Definition von Fehlerfällen (z.B. ungültige Einstellungen, Konfigurationsfehler, D-Bus-Fehler).
        - Verwendung von `Result` und spezifischen Fehler-Enums.
    - **Teststrategie**:
        - Unit-Tests für einzelne Funktionen.
        - Integrationstests zur Überprüfung der D-Bus-Interaktion und der Konfigurationsverwaltung.

---

### 5.28. `novade-system/src/notification_service.rs`

#### 5.28.1. Modulverantwortlichkeiten und -grenzen

- **Verantwortlichkeit:**
    - Empfang, Verwaltung und Anzeige von Desktop-Benachrichtigungen von Anwendungen und Systemdiensten.
    - Implementierung der Freedesktop Notification Specification.
    - Verwaltung von Benachrichtigungsregeln (optional).
- **Grenzen:**
    - Die eigentliche Darstellung der Benachrichtigungen erfolgt durch die Shell.

#### 5.28.2. Schnittstellen zu anderen Modulen

- `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
- D-Bus: Für den Empfang von Benachrichtigungen und die Kommunikation mit Clients.
- `novade-system/src/shell`: Für die Darstellung der Benachrichtigungen.

#### 5.28.3. Datenstrukturen und Algorithmen

- **Datenstrukturen:**
    - Strukturen zur Repräsentation von Benachrichtigungen.
    - Datenstrukturen für die Kommunikation mit D-Bus.
    - Datenstrukturen zur Repräsentation von Benachrichtigungsregeln (optional).
- **Algorithmen:**
    - Algorithmen zum Empfangen und Verwalten von Benachrichtigungen.
    - Algorithmen zur Anzeige von Benachrichtigungen.
    - Algorithmen zur Auswertung von Benachrichtigungsregeln (optional).

#### 5.28.4. Fehlerbehandlung und Ausnahmen

- **Fehlerfälle:**
    - Fehler beim Empfangen oder Anzeigen von Benachrichtigungen.
    - Ungültige Benachrichtigungen.
    - Fehler bei der Auswertung von Benachrichtigungsregeln (optional).
- **Ausnahmen:**
    - Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

#### 5.28.5. Ereignisse und Callbacks

- **Ereignisse:**
    - Benachrichtigungsereignisse.
- **Callbacks:**
    - Callbacks zur Benachrichtigung anderer Module über Benachrichtigungsereignisse.

#### 5.28.6. Designentscheidungen und Trade-offs

- **Entscheidung:** Implementierung der Freedesktop Notification Specification.
- **Alternativen:** Eigene Implementierung des Benachrichtigungssystems.
- **Trade-offs:** Die Freedesktop Notification Specification ist ein etablierter Standard, kann aber die Flexibilität einschränken.

#### 5.28.7. Teststrategie

- **Unit-Tests:**
    - Testen der Funktionen zum Empfangen und Verwalten von Benachrichtigungen.
    - Testen der Anzeige von Benachrichtigungen.
    - Mocken von D-Bus-Interaktionen.
- **Integrationstests:**
    - Testen der Interaktion mit Anwendungen und anderen Diensten.
    - Testen der Interaktion mit der Shell.

#### 5.28.8. Detaillierte Spezifikation der Benachrichtigungsverwaltung

- **Dateien**:
    
    - `system/notification_service/notification_manager.rs` (Kernlogik)
    - `system/notification_service/d_bus_interface.rs` (D-Bus-Schnittstelle)
    - `system/notification_service/rules_engine.rs` (Regelverarbeitung, optional)
- **Abhängigkeiten**:
    
    - `zbus` für D-Bus-Interaktion
    - Freedesktop Notification Specification
    - Regelverarbeitungsbibliotheken (optional, z.B. eigene Implementierung oder externe Bibliothek)
- **Spezifikation**:
    * **Funktionalität**:
        * Empfang von Benachrichtigungen über D-Bus.
        * Verwaltung von Benachrichtigungen (Speicherung, Priorisierung, Ablauf).
        * Anzeige von Benachrichtigungen (in Zusammenarbeit mit der Shell).
        * Interaktion mit dem Benutzer (z.B. Antworten, Schließen).
        * Optionale Unterstützung für Benachrichtigungsregeln (Filterung, Drosselung).
    * **D-Bus-Interaktion**:
        * Implementierung der `org.freedesktop.Notifications`-Schnittstelle.
        * Verwendung von `zbus` zur Registrierung des Dienstes auf dem Session-Bus.
        * Definition von Methoden zum Senden (`Notify`), Schließen (`CloseNotification`) und Abrufen von Funktionen (`GetCapabilities`).
        * Definition von Signalen zur Benachrichtigung über Benachrichtigungsereignisse (`NotificationClosed`, `ActionInvoked`).
    * **Datenstrukturen**:
        * `Notification` (Struktur zur Repräsentation einer Benachrichtigung mit Titel, Text, Symbol, Aktionen, etc.).
        * `NotificationId` (Typ zur eindeutigen Identifizierung einer Benachrichtigung).
        * Datenstrukturen für die D-Bus-Kommunikation.
        * Datenstrukturen zur Repräsentation von Benachrichtigungsregeln (optional).
    * **Algorithmen**:
        * Algorithmen zum Empfangen und Verarbeiten von D-Bus-Nachrichten.
        * Algorithmen zur Verwaltung der Benachrichtigungswarteschlange.
        * Algorithmen zur Anzeige von Benachrichtigungen (in Zusammenarbeit mit der Shell).
        * Algorithmen zur Auswertung von Benachrichtigungsregeln (optional).
    * **Fehlerbehandlung**:
        * Definition von Fehlerfällen (z.B. ungültige Benachrichtigungen, D-Bus-Fehler).
        * Verwendung von `Result` und spezifischen Fehler-Enums.
    * **Teststrategie**:
        * Unit-Tests für einzelne Funktionen.
        * Integrationstests zur Überprüfung der D-Bus-Interaktion und der Zusammenarbeit mit der Shell.

---

### 5.29. `novade-system/src/power_manager.rs`

####   5.29.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Überwachung des Batteriestatus.
    * Verwaltung von Energieeinstellungen (z.B. Energiesparmodi).
    * Behandlung von Suspend/Hibernate-Zuständen.
    * Steuerung der Bildschirmhelligkeit.
* **Grenzen:**
    * Die eigentliche Interaktion mit der Hardware erfolgt über UPower oder Kernel-Schnittstellen.

####   5.29.2. Schnittstellen zu anderen Modulen

* `novade-core`:  Für grundlegende Datentypen und Dienstprogramme.
* D-Bus:  Für die Kommunikation mit UPower und `logind`.8
* Kernel-Schnittstellen:  Für die Steuerung der Bildschirmhelligkeit (optional).

####   5.29.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Strukturen zur Repräsentation des Batteriestatus.
    * Strukturen zur Repräsentation von Energieeinstellungen.
* **Algorithmen:**
    * Algorithmen zur Überwachung des Batteriestatus.
    * Algorithmen zur Anwendung von Energieeinstellungen.
    * Algorithmen zur Initiierung von Suspend/Hibernate.
    * Algorithmen zur Steuerung der Bildschirmhelligkeit.

####   5.29.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Abrufen des Batteriestatus.
    * Fehler beim Anwenden von Energieeinstellungen.
    * Fehler beim Initiieren von Suspend/Hibernate.
    * Fehler bei der Steuerung der Bildschirmhelligkeit.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

####   5.29.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Batteriestatusänderungsereignisse.
    * Energieeinstellungsänderungsereignisse.
* **Callbacks:**
    * Callbacks zur Benachrichtigung anderer Module über Energieereignisse.

####   5.29.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung von UPower zur Verwaltung der Energieversorgung.
* **Alternativen:** Direkte Interaktion mit Kernel-Schnittstellen.
* **Trade-offs:** UPower bietet eine Abstraktionsebene, kann aber die Komplexität erhöhen.

####   5.29.7. Teststrategie

* **Unit-Tests:**
    * Testen der Funktionen zur Überwachung des Batteriestatus.
    * Testen der Funktionen zur Anwendung von Energieeinstellungen.
    * Mocken von D-Bus-Interaktionen.
* **Integrationstests:**
    * Testen des gesamten Energieverwaltungszyklus.
    * Testen der Interaktion mit UPower und `logind`.

####   5.29.8. Detaillierte Spezifikation der Energieverwaltung

* **Dateien**:
    * `system/power_manager/power_manager.rs` (Kernlogik)
    * `system/power_manager/upower_interface.rs` (D-Bus-Schnittstelle zu UPower)
    * `system/power_manager/brightness_control.rs` (Bildschirmhelligkeitssteuerung, optional)

* **Abhängigkeiten**:
    * `zbus` für D-Bus-Interaktion mit UPower und `logind`
    * UPower D-Bus-Schnittstelle
    * Kernel-Schnittstellen für Bildschirmhelligkeit (optional)

* **Spezifikation**:
    * **Funktionalität**:
        * Abrufen des Batteriestatus (Ladestand, verbleibende Zeit, etc.).
        * Abrufen und Setzen von Energieeinstellungen (z.B. Energiesparmodi, Leerlaufverhalten).
        * Initiieren von Suspend/Hibernate-Aktionen.
        * Steuerung der Bildschirmhelligkeit (optional).
        * Benachrichtigung von Clients über Energieereignisse.
    * **D-Bus-Interaktion**:
        * Verwendung von `zbus` zur Kommunikation mit UPower und `logind`.
        * Abrufen von Informationen über den Batteriestatus und Energieeinstellungen von UPower.
        * Senden von Befehlen zum Initiieren von Suspend/Hibernate an `logind`.
    * **Kernel-Schnittstellen (optional)**:
        * Direkte Interaktion mit Kernel-Schnittstellen zur Steuerung der Bildschirmhelligkeit (z.B. über `/sys/class/backlight/`).
    * **Datenstrukturen**:
        * `BatteryStatus` (Struktur zur Repräsentation des Batteriestatus).
        * `PowerSettings` (Struktur zur Repräsentation von Energieeinstellungen).
        * Datenstrukturen für die D-Bus-Kommunikation.
    * **Algorithmen**:
        * Algorithmen zum Abrufen und Verarbeiten von Informationen von UPower.
        * Algorithmen zum Anwenden von Energieeinstellungen.
        * Algorithmen zum Berechnen von Batteriestatusinformationen.
        * Algorithmen zur Steuerung der Bildschirmhelligkeit (optional).
    * **Fehlerbehandlung**:
        * Definition von Fehlerfällen (z.B. D-Bus-Fehler, Hardware-Fehler).
        * Verwendung von `Result` und spezifischen Fehler-Enums.
    * **Teststrategie**:
        * Unit-Tests für einzelne Funktionen.
        * Integrationstests zur Überprüfung der D-Bus-Interaktion und der Hardware-Interaktion (falls möglich).

---

### 5.30. `novade-system/src/workspace_manager.rs`

####   5.30.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Verwaltung von virtuellen Desktops/Arbeitsbereichen.10
    * Erstellen, Löschen und Wechseln von Arbeitsbereichen.
    * Zuordnung von Fenstern zu Arbeitsbereichen.
    * Verwaltung von Arbeitsbereichseinstellungen (z.B. Anzahl, Namen).
* **Grenzen:**
    * Die eigentliche Darstellung der Arbeitsbereiche und Fenster erfolgt durch den Compositor und die Shell.

####   5.30.2. Schnittstellen zu anderen Modulen

* `novade-core`:  Für grundlegende Datentypen und Dienstprogramme.
* `novade-system/src/compositor`:  Für die Zuordnung und Darstellung von Fenstern.
* `novade-system/src/shell`:  Für die Bereitstellung der Benutzeroberfläche zur Arbeitsbereichsverwaltung.

####   5.30.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * `Workspace` (Struktur zur Repräsentation eines Arbeitsbereichs).
    * `WorkspaceId` (Typ zur eindeutigen Identifizierung eines Arbeitsbereichs).
    * Datenstrukturen zur Repräsentation von Arbeitsbereichseinstellungen.
* **Algorithmen:**
    * Algorithmen zum Erstellen, Löschen und Wechseln von Arbeitsbereichen.
    * Algorithmen zur Zuordnung von Fenstern zu Arbeitsbereichen.
    * Algorithmen zur Verwaltung von Arbeitsbereichseinstellungen.

####   5.30.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Erstellen, Löschen oder Wechseln von Arbeitsbereichen.
    * Fehler bei der Zuordnung von Fenstern.
    * Ungültige Arbeitsbereichseinstellungen.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

####   5.30.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Arbeitsbereichsänderungsereignisse (z.B. Arbeitsbereich erstellt, gelöscht, gewechselt).
* **Callbacks:**
    * Callbacks zur Benachrichtigung anderer Module über Arbeitsbereichsereignisse.

####   5.30.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwaltung der Arbeitsbereichslogik in einem separaten Modul.
* **Alternativen:** Integration der Arbeitsbereichslogik in den Compositor oder die Shell.
* **Trade-offs:** Die separate Verwaltung fördert die Modularität, kann aber die Komplexität der Interaktion mit dem Compositor und der Shell erhöhen.

####   5.30.7. Teststrategie

* **Unit-Tests:**
    * Testen der Funktionen zum Erstellen, Löschen und Wechseln von Arbeitsbereichen.
    * Testen der Funktionen zur Zuordnung von Fenstern.
    * Mocken der Interaktion mit dem Compositor und der Shell.
* **Integrationstests:**
    * Testen des gesamten Arbeitsbereichsverwaltungszyklus.
    * Testen der Interaktion mit dem Compositor und der Shell.

####   5.30.8. Detaillierte Spezifikation der Workspace-Verwaltung

* **Dateien**:
    * `system/workspace_manager/workspace_manager.rs` (Kernlogik)
    * `system/workspace_manager/workspace.rs` (Datenstrukturen für Arbeitsbereiche)
    * `system/workspace_manager/config.rs` (Verwaltung von Arbeitsbereichseinstellungen)

* **Abhängigkeiten**:
    * `novade-core` für grundlegende Datentypen und Dienstprogramme
    * `novade-system/src/compositor` (indirekt über Callbacks/Events)
    * `novade-system/src/shell` (indirekt über Callbacks/Events)

* **Spezifikation**:
    * **Funktionalität**:
        * Erstellen neuer Arbeitsbereiche mit optionalen Namen.
        * Löschen existierender Arbeitsbereiche (mit Regeln für die Behandlung von Fenstern in diesen Arbeitsbereichen).
        * Wechseln zwischen Arbeitsbereichen (mit Animationen, optional).
        * Zuordnen von Fenstern zu bestimmten Arbeitsbereichen (manuell oder automatisch basierend auf Regeln).
        * Abrufen von Informationen über die aktiven Arbeitsbereiche, die darin enthaltenen Fenster und deren Anordnung.
        * Verwalten von Arbeitsbereichseinstellungen, z.B. die Anzahl der Arbeitsbereiche, Standardnamen, Verhalten beim Erstellen/Löschen.
        * Unterstützung für das Speichern und Laden von Arbeitsbereichskonfigurationen.
    * **Datenstrukturen**:
        * `Workspace` (Struktur zur Repräsentation eines Arbeitsbereichs):
            * `id: WorkspaceId` (eindeutige ID)
            * `name: String` (optionaler Name)
            * `windows: Vec<WindowId>` (Liste der Fenster im Arbeitsbereich)
            * Weitere Metadaten (z.B. Layout-Informationen)
        * `WorkspaceId` (Typ zur eindeutigen Identifizierung eines Arbeitsbereichs, z.B. `u32`).
        * `WorkspaceConfig` (Struktur zur Repräsentation der Arbeitsbereichskonfiguration):
            * `number_of_workspaces: u32`
            * `default_names: Vec<String>`
            * Weitere Einstellungen
    * **Algorithmen**:
        * Algorithmus zum Erstellen eines neuen Arbeitsbereichs:
            1.  Generiere eine neue `WorkspaceId`.
            2.  Erstelle eine neue `Workspace`-Instanz mit der ID und dem optionalen Namen.
            3.  Füge den Arbeitsbereich zur Liste der verwalteten Arbeitsbereiche hinzu.
            4.  Löse ein `WorkspaceCreatedEvent` aus.
        * Algorithmus zum Wechseln zu einem Arbeitsbereich:
            1.  Überprüfe, ob der Zielarbeitsbereich existiert.
            2.  Aktualisiere den aktiven Arbeitsbereich.
            3.  Benachrichtige den Compositor und die Shell über den Wechsel.
            4.  Löse ein `ActiveWorkspaceChangedEvent` aus.
        * Algorithmus zum Zuordnen eines Fensters zu einem Arbeitsbereich:
            1.  Überprüfe, ob der Arbeitsbereich und das Fenster existieren.
            2.  Füge die Fenster-ID zur Liste der Fenster im Arbeitsbereich hinzu.
            3.  Benachrichtige den Compositor über die Zuordnung.
    * **Fehlerbehandlung**:
        * Definition von Fehlerfällen:
            * `WorkspaceNotFoundError`
            * `WorkspaceLimitExceededError`
            * `InvalidWorkspaceConfigError`
        * Verwendung von `Result` und spezifischen Fehler-Enums.
    * **Teststrategie**:
        * Unit-Tests für die Kernfunktionen des Workspace-Managements.
        * Integrationstests, die die Interaktion mit dem Compositor und der Shell simulieren (z.B. durch Mocking von Events).
        * Tests für die Konfigurationsverwaltung.

---

### 5.31. `novade-system/src/pipewire_integration.rs`

####   5.31.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Integration von PipeWire für Audio- und Video-Stream-Management.12
    * Bereitstellung von Audio- und Video-Funktionalität für Anwendungen.
    * Unterstützung für Screen-Sharing unter Wayland.
    * Verwaltung von Audiogeräten und -profilen.
* **Grenzen:**
    * Die eigentliche Verarbeitung der Audio- und Videoströme erfolgt durch PipeWire.

####   5.31.2. Schnittstellen zu anderen Modulen

* `novade-core`:  Für grundlegende Datentypen und Dienstprogramme.
* D-Bus:  Für die Kommunikation mit PipeWire und anderen Diensten.
* `novade-system/src/compositor`:  Für die Unterstützung von Screen-Sharing unter Wayland.

####   5.31.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Datenstrukturen zur Repräsentation von Audio- und Videogeräten.
    * Datenstrukturen zur Repräsentation von Audio- und Videoströmen.
    * Datenstrukturen für die Kommunikation mit PipeWire.
* **Algorithmen:**
    * Algorithmen zur Verwaltung von Audio- und Videogeräten und -profilen.
    * Algorithmen zur Steuerung von Audio- und Videoströmen.
    * Algorithmen zur Unterstützung von Screen-Sharing.

####   5.31.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Verbinden mit PipeWire.
    * Fehler bei der Verwaltung von Audio- und Videogeräten und -strömen.
    * Fehler beim Screen-Sharing.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über Result\<T, E\> und Logging behandelt.

####   5.31.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Audio- und Videogeräteänderungsereignisse.
    * Audio- und Videostromänderungsereignisse.
    * Screen-Sharing-Ereignisse.
* **Callbacks:**
    * Callbacks zur Benachrichtigung anderer Module über Multimedia-Ereignisse.

####   5.31.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung von PipeWire für Multimedia-Management.
* **Alternativen:** Verwendung von PulseAudio und anderen Lösungen.
* **Trade-offs:** PipeWire bietet eine moderne und flexible Lösung, erfordert aber eine komplexere Integration.

####   5.31.7. Teststrategie

* **Unit-Tests:**
    * Testen der Funktionen zur Verwaltung von Audio- und Videogeräten.
    * Testen der Funktionen zur Steuerung von Audio- und Videoströmen.
    * Mocken der Interaktion mit PipeWire.
* **Integrationstests:**
    * Testen des gesamten Multimedia-Verwaltungszyklus.
    * Testen der Interaktion mit PipeWire.

####   5.31.8. Detaillierte Spezifikation der PipeWire-Integration

* **Dateien**:
    * `system/pipewire_integration/pipewire_integration.rs` (Kernlogik)
    * `system/pipewire_integration/pw_objects.rs` (Datenstrukturen für PipeWire-Objekte)
    * `system/pipewire_integration/screen_sharing.rs` (Unterstützung für Screen-Sharing)

* **Abhängigkeiten**:
    * `libpipewire` (C-Bibliothek)
    * `zbus` (für die optionale D-Bus-Interaktion mit PipeWire)
    * `novade-system/src/compositor` (für Screen-Sharing)

* **Spezifikation**:
    * **Funktionalität**:
        * Auflisten und Verwalten von Audio- und Videogeräten (z.B. Mikrofon, Lautsprecher, Kamera).
        * Abspielen und Aufnehmen von Audio- und Videoströmen.
        * Konfigurieren von Audiogeräten (z.B. Lautstärke, Stummschaltung, Profile).
        * Unterstützung für verschiedene Audioformate und Codecs.
        * Unterstützung für Screen-Sharing unter Wayland (z.B. über XDG Desktop Portal).
        * Mischen und Routen von Audioströmen.
        * Verwalten von PipeWire-Sitzungen und -Knoten.
    * **Datenstrukturen**:
        * `AudioDevice` (Struktur zur Repräsentation eines Audiogeräts).
        * `VideoStream` (Struktur zur Repräsentation eines Videostroms).
        * Datenstrukturen zur Repräsentation von PipeWire-Objekten (z.B. Nodes, Ports).
    * **Algorithmen**:
        * Algorithmen zum Auflisten und Verwalten von Geräten.
        * Algorithmen zum Steuern von Strömen.
        * Algorithmen zum Mischen und Routen von Audio.
        * Algorithmen zur Implementierung von Screen-Sharing.
    * **Fehlerbehandlung**:
        * Definition von Fehlerfällen (z.B. Verbindungsfehler, Gerätefehler, Streamfehler).
        * Verwendung von `Result` und spezifischen Fehler-Enums.
    * **Teststrategie**:
        * Unit-Tests für die Kernfunktionen der PipeWire-Integration.
        * Integrationstests, die die Interaktion mit PipeWire simulieren (z.B. durch Mocking von PipeWire-Objekten).
        * Tests für Screen-Sharing-Funktionalität.

---

### 5.32. `novade-system/src/dbus_broker.rs`

####   5.32.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Bereitstellung eines D-Bus-Brokers für die Interprozesskommunikation.14
    * Verwaltung von D-Bus-Verbindungen und -Nachrichten.
    * Sicherstellung der Sicherheit und des Datenschutzes bei der D-Bus-Kommunikation.
* **Grenzen:**
    * Die eigentliche Implementierung des D-Bus-Protokolls erfolgt durch den D-Bus-Daemon (z.B. `dbus-daemon` oder `dbus-broker`).

####   5.32.2. Schnittstellen zu anderen Modulen

* Alle anderen Module in `novade-system` und `novade-ui` nutzen D-Bus für die Kommunikation.

####   5.32.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Datenstrukturen zur Repräsentation von D-Bus-Nachrichten.
    * Datenstrukturen zur Repräsentation von D-Bus-Verbindungen.
    * Datenstrukturen zur Repräsentation von D-Bus-Diensten und -Objekten.
* **Algorithmen:**
    * Algorithmen zum Senden und Empfangen von D-Bus-Nachrichten.
    * Algorithmen zur Verwaltung von D-Bus-Verbindungen.
    * Algorithmen zur Registrierung und zum Abrufen von D-Bus-Diensten.

####   5.32.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Verbinden mit dem D-Bus-Daemon.
    * Fehler beim Senden oder Empfangen von D-Bus-Nachrichten.
    * Fehler bei der Registrierung oder beim Abrufen von D-Bus-Diensten.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

####   5.32.5. Ereignisse und Callbacks

* **Ereignisse:**
    * D-Bus-Verbindungsereignisse.
    * D-Bus-Nachrichtenereignisse.
* **Callbacks:**
    * Callbacks zur Benachrichtigung anderer Module über D-Bus-Ereignisse.

####   5.32.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung von D-Bus für die Interprozesskommunikation.
* **Alternativen:** Andere IPC-Mechanismen (z.B. Sockets, Shared Memory).
* **Trade-offs:** D-Bus bietet eine standardisierte und flexible Lösung, kann aber die Komplexität erhöhen.

####   5.32.7. Teststrategie

* **Unit-Tests:**
    * Testen der Funktionen zum Senden und Empfangen von D-Bus-Nachrichten.
    * Testen der Funktionen zur Verwaltung von D-Bus-Verbindungen.
    * Mocken der Interaktion mit dem D-Bus-Daemon.
* **Integrationstests:**
    * Testen des gesamten D-Bus-Kommunikationszyklus.
    * Testen der Interaktion zwischen verschiedenen Modulen über D-Bus.

####   5.32.8. Detaillierte Spezifikation des D-Bus-Brokers

* **Dateien**:
    * `system/dbus_broker/dbus_broker.rs` (Kernlogik)
    * `system/dbus_broker/connection.rs` (Verwaltung von D-Bus-Verbindungen)
    * `system/dbus_broker/message.rs` (Datenstrukturen für D-Bus-Nachrichten)

* **Abhängigkeiten**:
    * `zbus` (Rust-Bibliothek für D-Bus)
    * D-Bus-Daemon (z.B. `dbus-daemon` oder `dbus-broker`)

* **Spezifikation**:
    * **Funktionalität**:
        * Herstellen von Verbindungen zum D-Bus-Daemon.
        * Senden und Empfangen von D-Bus-Nachrichten (Methodenaufrufe, Signale, Eigenschaften).
        * Registrieren von D-Bus-Diensten und -Objekten.
        * Bereitstellen von D-Bus-Schnittstellen.
        * Verwalten von D-Bus-Namespaces.
        * Sicherheitsüberprüfung von D-Bus-Nachrichten (optional).
    * **Datenstrukturen**:
        * `DBusConnection` (Struktur zur Repräsentation einer D-Bus-Verbindung).
        * `DBusMessage` (Struktur zur Repräsentation einer D-Bus-Nachricht).
        * `DBusService` (Struktur zur Repräsentation eines D-Bus-Dienstes).
        * `DBusObject` (Struktur zur Repräsentation eines D-Bus-Objekts).
    * **Algorithmen**:
        * Algorithmus zum Herstellen einer Verbindung zum D-Bus-Daemon.
        * Algorithmus zum Senden einer D-Bus-Nachricht.
        * Algorithmus zum Empfangen einer D-Bus-Nachricht.
        * Algorithmus zur Registrierung eines D-Bus-Dienstes.
        * Algorithmus zum Abrufen eines D-Bus-Dienstes.
    * **Fehlerbehandlung**:
        * Definition von Fehlerfällen (z.B. Verbindungsfehler, Nachrichtenfehler, Registrierungsfehler).
        * Verwendung von `Result` und spezifischen Fehler-Enums.
    * **Teststrategie**:
        * Unit-Tests für die Kernfunktionen der D-Bus-Kommunikation.
        * Integrationstests, die die Interaktion mit dem D-Bus-Daemon simulieren (z.B. durch Mocking von D-Bus-Nachrichten).
        * Tests für die Interaktion zwischen verschiedenen Modulen über D-Bus.

---

### 5.33. `novade-system/src/xwayland_server.rs`

####   5.33.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Bereitstellung eines XWayland-Servers zur Ausführung von X11-Anwendungen unter Wayland.15
    * Übersetzung von X11-Anfragen in Wayland-Protokollnachrichten.
    * Verwaltung von X11-Fenstern und -Eingabeereignissen.
* **Grenzen:**
    * Die eigentliche Ausführung der X11-Anwendungen erfolgt durch XWayland.

####   5.33.2. Schnittstellen zu anderen Modulen

* `novade-core`:  Für grundlegende Datentypen und Dienstprogramme.
* `novade-system/src/compositor`:  Für die Integration von XWayland-Fenstern in die Wayland-Umgebung.

####   5.33.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Datenstrukturen zur Repräsentation von X11-Fenstern und -Eingabeereignissen.
    * Datenstrukturen zur Repräsentation von Wayland-Oberflächen.
* **Algorithmen:**
    * Algorithmen zur Übersetzung von X11-Anfragen in Wayland-Protokollnachrichten.
    * Algorithmen zur Verwaltung von X11-Fenstern und -Eingabeereignissen.
    * Algorithmen zur Integration von XWayland-Fenstern in die Wayland-Umgebung.

####   5.33.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Starten des XWayland-Servers.
    * Fehler bei der Übersetzung von X11-Anfragen.
    * Fehler bei der Verwaltung von X11-Fenstern und -Eingabeereignissen.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

####   5.33.5. Ereignisse und Callbacks

* **Ereignisse:**
    * X11-Fensterereignisse.
    * X11-Eingabeereignisse.
* **Callbacks:**
    * Callbacks zur Benachrichtigung anderer Module über XWayland-Ereignisse.

####   5.33.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Bereitstellung eines XWayland-Servers zur Unterstützung von X11-Anwendungen.
* **Alternativen:** Keine Unterstützung für X11-Anwendungen.
* **Trade-offs:** Die Bereitstellung von XWayland ermöglicht die Ausführung von Legacy-Anwendungen, kann aber die Komplexität des Systems erhöhen.

####   5.33.7. Teststrategie

* **Unit-Tests:**
    * Testen der Funktionen zur Übersetzung von X11-Anfragen.
    * Testen der Funktionen zur Verwaltung von X11-Fenstern und -Eingabeereignissen.
    * Mocken der Interaktion mit dem XWayland-Server.
* **Integrationstests:**
    * Testen des gesamten XWayland-Kommunikationszyklus.
    * Testen der Interaktion zwischen X11-Anwendungen und dem Wayland-Compositor.

####   5.33.8. Detaillierte Spezifikation des XWayland-Servers

* **Dateien**:
    * `system/xwayland_server/xwayland_server.rs` (Kernlogik)
    * `system/xwayland_server/x11_requests.rs` (Verarbeitung von X11-Anfragen)
    * `system/xwayland_server/wayland_integration.rs` (Integration mit dem Wayland-Compositor)

* **Abhängigkeiten**:
    * `smithay` (Wayland-Bibliothek)
    * XWayland (X-Server)

* **Spezifikation**:
    * **Funktionalität**:
        * Starten und Verwalten des XWayland-Servers.
        * Empfangen und Verarbeiten von X11-Anfragen.
        * Übersetzen von X11-Anfragen in Wayland-Protokollnachrichten.
        * Erstellen und Verwalten von Wayland-Oberflächen für X11-Fenster.
        * Weiterleiten von Eingabeereignissen zwischen X11-Anwendungen und dem Wayland-Compositor.
        * Unterstützung für verschiedene X11-Erweiterungen (optional).
    * **Datenstrukturen**:
        * `X11Window` (Struktur zur Repräsentation eines X11-Fensters).
        * `X11InputEvent` (Struktur zur Repräsentation eines X11-Eingabeereignisses).
        * Datenstrukturen zur Repräsentation von X11-Clients und -Ressourcen.
        * Datenstrukturen zur Repräsentation von Wayland-Oberflächen.
    * **Algorithmen**:
        * Algorithmus zum Starten des XWayland-Servers.
        * Algorithmus zum Empfangen und Verarbeiten einer X11-Anfrage.
        * Algorithmus zum Übersetzen einer X11-Anfrage in Wayland-Protokollnachrichten.
        * Algorithmus zum Erstellen einer Wayland-Oberfläche für ein X11-Fenster.
        * Algorithmus zum Weiterleiten eines Eingabeereignisses.
    * **Fehlerbehandlung**:
        * Definition von Fehlerfällen (z.B. Startfehler, Übersetzungsfehler, Fensterverwaltungsfehler).
        * Verwendung von `Result` und spezifischen Fehler-Enums.
    * **Teststrategie**:
        * Unit-Tests für die Kernfunktionen der XWayland-Integration.
        * Integrationstests, die die Interaktion mit dem XWayland-Server und dem Wayland-Compositor simulieren.
        * Tests mit verschiedenen X11-Anwendungen.

---

### 5.34. `novade-system/src/mcp_integration.rs` (optional)

####   5.34.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Integration des Model Context Protocol (MCP) zur strukturierten Interaktion mit KI-Modellen.17
    * Bereitstellung einer Schnittstelle für KI-Modelle zum Zugriff auf Systemfunktionen und Daten.
    * Sicherheitsmanagement und Zugriffskontrolle für KI-Interaktionen.
* **Grenzen:**
    * Die eigentliche Ausführung der KI-Modelle erfolgt außerhalb von NovaDE.

####   5.34.2. Schnittstellen zu anderen Modulen

* `novade-core`:  Für grundlegende Datentypen und Dienstprogramme.
* Verschiedene Systemdienste (z.B. Kalender, Dateimanager) für den Zugriff auf Funktionen und Daten.

####   5.34.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Datenstrukturen zur Repräsentation von MCP-Nachrichten (Requests, Responses, Errors).
    * Datenstrukturen zur Repräsentation von Daten, die zwischen NovaDE und KI-Modellen ausgetauscht werden.
    * Datenstrukturen für die Zugriffskontrolle und Sicherheitsrichtlinien.
* **Algorithmen**
* Algorithmen zur Verarbeitung von MCP-Nachrichten.
    * Algorithmen zur Authentifizierung und Autorisierung von KI-Modellen.
    * Algorithmen zur sicheren Datenübertragung zwischen NovaDE und KI-Modellen.

####   5.34.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler bei der Kommunikation mit KI-Modellen.
    * Sicherheitsverletzungen (z.B. unautorisierter Zugriff).
    * Fehler bei der Datenvalidierung.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

####   5.34.5. Ereignisse und Callbacks

* **Ereignisse:**
    * MCP-Nachrichtenereignisse.
    * Sicherheitsereignisse.
* **Callbacks:**
    * Callbacks zur Benachrichtigung von KI-Modellen über Systemereignisse.
    * Callbacks zur Benachrichtigung anderer Module über MCP-Ereignisse.

####   5.34.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung von MCP für die strukturierte Interaktion mit KI-Modellen.
* **Alternativen:** Andere Protokolle oder Ad-hoc-Lösungen.
* **Trade-offs:** MCP bietet eine standardisierte und sichere Lösung, erfordert aber eine sorgfältige Implementierung.

####   5.34.7. Teststrategie

* **Unit-Tests:**
    * Testen der Funktionen zur Verarbeitung von MCP-Nachrichten.
    * Testen der Funktionen zur Authentifizierung und Autorisierung von KI-Modellen.
    * Mocken der Interaktion mit KI-Modellen.
* **Integrationstests:**
    * Testen des gesamten MCP-Kommunikationszyklus.
    * Testen der Interaktion zwischen NovaDE und KI-Modellen.

####   5.34.8. Detaillierte Spezifikation der MCP-Integration

* **Dateien**:
    * `system/mcp_integration/mcp_integration.rs` (Kernlogik)
    * `system/mcp_integration/mcp_messages.rs` (Definition von MCP-Nachrichten)
    * `system/mcp_integration/security.rs` (Sicherheitsmanagement)

* **Abhängigkeiten**:
    * `mcp_client_rs` (Rust-Bibliothek für MCP)
    * KI-Modelle (externe Prozesse)

* **Spezifikation**:
    * **Funktionalität**:
        * Implementierung des MCP-Protokolls (Requests, Responses, Errors).
        * Serialisierung und Deserialisierung von MCP-Nachrichten.
        * Authentifizierung und Autorisierung von KI-Modellen.
        * Sichere Datenübertragung zwischen NovaDE und KI-Modellen (z.B. durch Verschlüsselung).
        * Bereitstellung einer Schnittstelle für KI-Modelle zum Zugriff auf Systemfunktionen und Daten.
        * Verwaltung von KI-Modellsitzungen.
    * **Datenstrukturen**:
        * `MCPRequest` (Struktur zur Repräsentation einer MCP-Anfrage).
        * `MCPResponse` (Struktur zur Repräsentation einer MCP-Antwort).
        * `MCPError` (Struktur zur Repräsentation eines MCP-Fehlers).
        * Datenstrukturen zur Repräsentation von Daten, die zwischen NovaDE und KI-Modellen ausgetauscht werden.
    * **Algorithmen**:
        * Algorithmus zum Serialisieren einer MCP-Nachricht.
        * Algorithmus zum Deserialisieren einer MCP-Nachricht.
        * Algorithmus zur Authentifizierung eines KI-Modells.
        * Algorithmus zur Autorisierung eines KI-Modells.
        * Algorithmus zur sicheren Datenübertragung.
    * **Fehlerbehandlung**:
        * Definition von Fehlerfällen (z.B. Kommunikationsfehler, Sicherheitsfehler, Datenfehler).
        * Verwendung von `Result` und spezifischen Fehler-Enums.
    * **Teststrategie**:
        * Unit-Tests für die Kernfunktionen der MCP-Integration.
        * Integrationstests, die die Interaktion mit KI-Modellen simulieren (z.B. durch Mocking von KI-Modellantworten).
        * Tests für Sicherheitsfunktionen.

---

### 5.35. `novade-system/src/display_manager_service.rs`

####   5.35.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Verwaltung von Displays und Monitoren.
    * Setzen von Auflösung, Bildwiederholrate und anderen Anzeigeeinstellungen.
    * Unterstützung für Multi-Monitor-Konfigurationen.
    * Implementierung von Display Power Management Signaling (DPMS).
* **Grenzen:**
    * Die tatsächliche Ansteuerung der Hardware erfolgt durch den Wayland-Compositor.

####   5.35.2. Schnittstellen zu anderen Modulen

* `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
* `novade-system/src/compositor`: Für die Kommunikation mit dem Wayland-Compositor.
* `novade-domain/src/settings_service`: Für den Zugriff auf Benutzereinstellungen.

####   5.35.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Datenstrukturen zur Repräsentation von Displays und Monitoren.
    * Datenstrukturen zur Repräsentation von Anzeigeeinstellungen.
* **Algorithmen:**
    * Algorithmen zum Abrufen von Displayinformationen.
    * Algorithmen zum Setzen von Anzeigeeinstellungen.
    * Algorithmen zur Verwaltung von Multi-Monitor-Konfigurationen.
    * Algorithmen zur Implementierung von DPMS.

####   5.35.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Abrufen von Displayinformationen.
    * Fehler beim Setzen von Anzeigeeinstellungen.
    * Hardwarefehler.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

####   5.35.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Display-Verbindungs- und Trennungsereignisse.
    * Änderungen der Anzeigeeinstellungen.
* **Callbacks:**
    * Callbacks zur Benachrichtigung anderer Module über Display-Ereignisse.

####   5.35.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung des Wayland-Compositors zur Ansteuerung der Hardware.
* **Alternativen:** Direkte Ansteuerung der Hardware.
* **Trade-offs:** Die Verwendung des Wayland-Compositors vereinfacht die Implementierung, kann aber die Flexibilität einschränken.

####   5.35.7. Teststrategie

* **Unit-Tests:**
    * Testen der Funktionen zum Abrufen von Displayinformationen.
    * Testen der Funktionen zum Setzen von Anzeigeeinstellungen.
    * Mocken der Interaktion mit dem Wayland-Compositor.
* **Integrationstests:**
    * Testen des gesamten Display-Management-Zyklus.
    * Testen der Interaktion mit dem Wayland-Compositor.
    * Tests mit verschiedenen Monitoren und Hardware-Konfigurationen.

####   5.35.8. Detaillierte Spezifikation des Display-Management-Services

* **Dateien**:
    * `system/display_manager_service/display_manager_service.rs` (Kernlogik)
    * `system/display_manager_service/display.rs` (Datenstrukturen zur Repräsentation von Displays)
    * `system/display_manager_service/settings.rs` (Verwaltung von Anzeigeeinstellungen)

* **Abhängigkeiten**:
    * `smithay` (Wayland-Bibliothek)
    * `udev` (für Hardwareerkennung)

* **Spezifikation**:
    * **Funktionalität**:
        * Erkennen und Verwalten von angeschlossenen Displays und Monitoren.
        * Abrufen von Displayinformationen (Name, Auflösung, Bildwiederholrate, etc.).
        * Setzen von Anzeigeeinstellungen (Auflösung, Bildwiederholrate, Orientierung, etc.).
        * Unterstützung für Multi-Monitor-Konfigurationen (Anordnen von Displays, Spiegeln/Erweitern des Desktops).
        * Implementierung von Display Power Management Signaling (DPMS) zur Energieverwaltung.
        * Verwaltung von Farbprofilen (optional).
    * **Datenstrukturen**:
        * `Display` (Struktur zur Repräsentation eines Displays).
        * `DisplayMode` (Struktur zur Repräsentation einer Anzeigemodus).
        * `DisplaySettings` (Struktur zur Repräsentation von Anzeigeeinstellungen).
    * **Algorithmen**:
        * Algorithmus zum Erkennen von Displays (z.B. über `udev`).
        * Algorithmus zum Abrufen von Displayinformationen (z.B. über Wayland-Protokoll).
        * Algorithmus zum Setzen von Anzeigeeinstellungen (z.B. über Wayland-Protokoll).
        * Algorithmus zur Berechnung von optimalen Anzeigemodi.
        * Algorithmus zur Verwaltung von Multi-Monitor-Konfigurationen (z.B. zur Berechnung der Position von Fenstern).
        * Algorithmus zur Implementierung von DPMS (z.B. zum Senden von DPMS-Befehlen an den Monitor).
    * **Fehlerbehandlung**:
        * Definition von Fehlerfällen (z.B. Hardwarefehler, Verbindungsfehler, ungültige Einstellungen).
        * Verwendung von `Result` und spezifischen Fehler-Enums.
    * **Teststrategie**:
        * Unit-Tests für die Kernfunktionen des Display-Management-Services.
        * Integrationstests, die die Interaktion mit dem Wayland-Compositor simulieren.
        * Tests mit verschiedenen Monitoren und Hardware-Konfigurationen.
        * Tests für DPMS-Funktionalität.

---

### 5.36. `novade-system/src/network_manager_service.rs`

####   5.36.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Verwaltung von Netzwerkverbindungen (WLAN, Ethernet, Bluetooth, Mobilfunk).
    * Konfiguration von Netzwerkgeräten.
    * Bereitstellung von Informationen über den Netzwerkstatus.
    * Implementierung von Netzwerkrichtlinien (z.B. automatische Verbindungen).
* **Grenzen:**
    * Die tatsächliche Ansteuerung der Netzwerkhardware erfolgt über das Betriebssystem (z.B. über NetworkManager oder Kernel-Schnittstellen).

####   5.36.2. Schnittstellen zu anderen Modulen

* `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
* `novade-domain/src/settings_service`: Für den Zugriff auf Benutzereinstellungen.
* `novade-ui`: Für die Anzeige von Netzwerkstatusinformationen und die Interaktion mit dem Benutzer.

####   5.36.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Datenstrukturen zur Repräsentation von Netzwerkgeräten.
    * Datenstrukturen zur Repräsentation von Netzwerkverbindungen.
    * Datenstrukturen zur Repräsentation von Netzwerkprofilen.
* **Algorithmen:**
    * Algorithmen zum Abrufen von Informationen über Netzwerkgeräte.
    * Algorithmen zum Herstellen und Trennen von Netzwerkverbindungen.
    * Algorithmen zum Konfigurieren von Netzwerkgeräten.
    * Algorithmen zur Verwaltung von Netzwerkprofilen.
    * Algorithmen zur Implementierung von Netzwerkrichtlinien.

####   5.36.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Abrufen von Informationen über Netzwerkgeräte.
    * Fehler beim Herstellen oder Trennen von Netzwerkverbindungen.
    * Konfigurationsfehler.
    * Hardwarefehler.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

####   5.36.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Netzwerkverbindungsstatusänderungen.
    * Änderungen der Netzwerkkonfiguration.
* **Callbacks:**
    * Callbacks zur Benachrichtigung anderer Module über Netzwerkereignisse.

####   5.36.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung des Betriebssystems zur Ansteuerung der Netzwerkhardware.
* **Alternativen:** Direkte Ansteuerung der Netzwerkhardware.
* **Trade-offs:** Die Verwendung des Betriebssystems vereinfacht die Implementierung, kann aber die Flexibilität einschränken.

####   5.36.7. Teststrategie

* **Unit-Tests:**
    * Testen der Funktionen zum Abrufen von Informationen über Netzwerkgeräte.
    * Testen der Funktionen zum Herstellen und Trennen von Netzwerkverbindungen.
    * Mocken der Interaktion mit dem Betriebssystem.
* **Integrationstests:**
    * Testen des gesamten Netzwerk-Management-Zyklus.
    * Testen der Interaktion mit dem Betriebssystem.
    * Tests mit verschiedenen Netzwerkgeräten und -konfigurationen.

####   5.36.8. Detaillierte Spezifikation des Netzwerk-Management-Services

* **Dateien**:
    * `system/network_manager_service/network_manager_service.rs` (Kernlogik)
    * `system/network_manager_service/device.rs` (Datenstrukturen zur Repräsentation von Netzwerkgeräten)
    * `system/network_manager_service/connection.rs` (Datenstrukturen zur Repräsentation von Netzwerkverbindungen)
    * `system/network_manager_service/profile.rs` (Datenstrukturen zur Repräsentation von Netzwerkprofilen)

* **Abhängigkeiten**:
    * `zbus` (für die Kommunikation mit NetworkManager über D-Bus)
    * `nix` (für die direkte Interaktion mit Kernel-Schnittstellen)

* **Spezifikation**:
    * **Funktionalität**:
        * Erkennen und Verwalten von Netzwerkgeräten (WLAN, Ethernet, Bluetooth, Mobilfunk).
        * Abrufen von Informationen über Netzwerkgeräte (Name, Typ, Status, etc.).
        * Herstellen und Trennen von Netzwerkverbindungen.
        * Konfigurieren von Netzwerkgeräten (IP-Adresse, DNS-Server, etc.).
        * Verwalten von Netzwerkprofilen (Speichern von Verbindungseinstellungen).
        * Implementierung von Netzwerkrichtlinien (z.B. automatische Verbindungen, Roaming).
        * Bereitstellung von Informationen über den Netzwerkstatus (z.B. Verbindungsqualität, Datenverbrauch).
    * **Datenstrukturen**:
        * `NetworkDevice` (Struktur zur Repräsentation eines Netzwerkgeräts).
        * `NetworkConnection` (Struktur zur Repräsentation einer Netzwerkverbindung).
        * `NetworkProfile` (Struktur zur Repräsentation eines Netzwerkprofils).
    * **Algorithmen**:
        * Algorithmus zum Erkennen von Netzwerkgeräten (z.B. über NetworkManager oder Kernel-Schnittstellen).
        * Algorithmus zum Abrufen von Informationen über Netzwerkgeräte (z.B. über NetworkManager).
        * Algorithmus zum Herstellen einer Netzwerkverbindung (z.B. über NetworkManager).
        * Algorithmus zum Trennen einer Netzwerkverbindung (z.B. über NetworkManager).
        * Algorithmus zum Konfigurieren eines Netzwerkgeräts (z.B. über NetworkManager oder Kernel-Schnittstellen).
        * Algorithmus zum Verwalten von Netzwerkprofilen (z.B. zum Speichern und Laden von Verbindungseinstellungen).
        * Algorithmus zur Implementierung von Netzwerkrichtlinien (z.B. zur Auswahl des besten verfügbaren Netzwerks).
        * Algorithmus zur Berechnung der Verbindungsqualität (z.B. anhand der Signalstärke).
    * **Fehlerbehandlung**:
        * Definition von Fehlerfällen (z.B. Hardwarefehler, Verbindungsfehler, Konfigurationsfehler, Authentifizierungsfehler).
        * Verwendung von `Result` und spezifischen Fehler-Enums.
    * **Teststrategie**:
        * Unit-Tests für die Kernfunktionen des Netzwerk-Management-Services.
        * Integrationstests, die die Interaktion mit dem Betriebssystem simulieren.
        * Tests mit verschiedenen Netzwerkgeräten und -konfigurationen.
        * Tests für die Implementierung von Netzwerkrichtlinien.

---

Okay, hier ist die Vervollständigung von 5.37. `novade-system/src/power_manager_service.rs` und die Beendigung des Moduls, wie gewünscht:

####   5.37.1. Modulverantwortlichkeiten und -grenzen

* **Verantwortlichkeit:**
    * Überwachung des Batteriestatus.
    * Verwaltung von Energieeinstellungen (z.B. Energiesparmodi, Helligkeit).
    * Behandlung von Suspend/Hibernate-Zuständen.
    * Steuerung der Bildschirmhelligkeit.
    * Implementierung von Richtlinien für den Energieverbrauch.
    * Bereitstellung von Informationen über den aktuellen Energiezustand.
* **Grenzen:**
    * Die tatsächliche Ansteuerung der Hardware erfolgt über das Betriebssystem (z.B. über UPower oder Kernel-Schnittstellen).
    * Die genaue Implementierung von Suspend/Hibernate hängt vom Systemd ab.

####   5.37.2. Schnittstellen zu anderen Modulen

* `novade-core`: Für grundlegende Datentypen und Dienstprogramme.
* `novade-domain/src/settings_service`: Für den Zugriff auf Energieeinstellungen.
* `novade-ui`: Für die Anzeige von Informationen über den Energiezustand und die Interaktion mit dem Benutzer.

####   5.37.3. Datenstrukturen und Algorithmen

* **Datenstrukturen:**
    * Datenstrukturen zur Repräsentation des Batteriestatus.
    * Datenstrukturen zur Repräsentation von Energieeinstellungen.
    * Datenstrukturen zur Repräsentation von Energieprofilen.
* **Algorithmen:**
    * Algorithmen zum Abrufen von Informationen über den Batteriestatus.
    * Algorithmen zum Setzen von Energieeinstellungen.
    * Algorithmen zur Aktivierung von Energiesparmodi.
    * Algorithmen zum Initiieren von Suspend/Hibernate-Zuständen.
    * Algorithmen zur Steuerung der Bildschirmhelligkeit.
    * Algorithmen zur Implementierung von Richtlinien für den Energieverbrauch.

####   5.37.4. Fehlerbehandlung und Ausnahmen

* **Fehlerfälle:**
    * Fehler beim Abrufen von Informationen über den Batteriestatus.
    * Fehler beim Setzen von Energieeinstellungen.
    * Fehler beim Initiieren von Suspend/Hibernate-Zuständen.
    * Hardwarefehler.
* **Ausnahmen:**
    * Keine spezifischen Ausnahmen sind definiert. Fehler werden über `Result` und Logging behandelt.

####   5.37.5. Ereignisse und Callbacks

* **Ereignisse:**
    * Änderungen des Batteriestatus.
    * Änderungen der Energieeinstellungen.
    * Eintritt und Austritt von Suspend/Hibernate-Zuständen.
* **Callbacks:**
    * Callbacks zur Benachrichtigung anderer Module über Energieereignisse.

####   5.37.6. Designentscheidungen und Trade-offs

* **Entscheidung:** Verwendung des Betriebssystems zur Ansteuerung der Energieverwaltung.
* **Alternativen:** Direkte Ansteuerung der Hardware.
* **Trade-offs:** Die Verwendung des Betriebssystems vereinfacht die Implementierung, kann aber die Flexibilität einschränken.

####   5.37.7. Teststrategie

* **Unit-Tests:**
    * Testen der Funktionen zum Abrufen von Informationen über den Batteriestatus.
    * Testen der Funktionen zum Setzen von Energieeinstellungen.
    * Mocken der Interaktion mit dem Betriebssystem.
* **Integrationstests:**
    * Testen des gesamten Energie-Management-Zyklus.
    * Testen der Interaktion mit dem Betriebssystem.
    * Tests mit verschiedenen Hardware-Konfigurationen.
    * Tests für Suspend/Hibernate-Funktionalität.

####   5.37.8. Detaillierte Spezifikation des Power-Management-Services

* **Dateien**:
    * `system/power_manager_service/power_manager_service.rs` (Kernlogik)
    * `system/power_manager_service/battery.rs` (Datenstrukturen zur Repräsentation des Batteriestatus)
    * `system/power_manager_service/settings.rs` (Datenstrukturen zur Repräsentation von Energieeinstellungen)
    * `system/power_manager_service/profile.rs` (Datenstrukturen zur Repräsentation von Energieprofilen)

* **Abhängigkeiten**:
    * `zbus` (für die Kommunikation mit UPower über D-Bus)
    * `logind` (für Suspend/Hibernate)
    * Kernel-Schnittstellen (für direkte Hardwaresteuerung, falls erforderlich)

* **Spezifikation**:
    * **Funktionalität**:
        * Überwachen des Batteriestatus (Ladestand, verbleibende Zeit, Ladezustand, etc.).
        * Abrufen und Setzen von Energieeinstellungen (z.B. Energiesparmodi, Helligkeit, Leerlaufzeiten).
        * Verwalten von Energieprofilen (z.B. "Leistung", "Ausgewogen", "Energiesparen").
        * Initiieren und Verwalten von Suspend/Hibernate-Zuständen.
        * Steuern der Bildschirmhelligkeit (automatisch oder manuell).
        * Implementieren von Richtlinien für den Energieverbrauch (z.B. automatisches Dimmen des Bildschirms, Abschalten von Geräten).
        * Bereitstellen von Informationen über den aktuellen Energiezustand für andere Module und die UI.
    * **Datenstrukturen**:
        * `BatteryStatus` (Struktur zur Repräsentation des Batteriestatus).
        * `PowerSettings` (Struktur zur Repräsentation von Energieeinstellungen).
        * `PowerProfile` (Struktur zur Repräsentation eines Energieprofils).
    * **Algorithmen**:
        * Algorithmus zum Abrufen von Informationen über den Batteriestatus (z.B. über UPower).
        * Algorithmus zum Setzen von Energieeinstellungen (z.B. über UPower oder Kernel-Schnittstellen).
        * Algorithmus zur Aktivierung eines Energieprofils.
        * Algorithmus zum Initiieren eines Suspend/Hibernate-Zustands (z.B. über `logind`).
        * Algorithmus zur Berechnung der optimalen Bildschirmhelligkeit.
        * Algorithmus zur Anwendung von Richtlinien für den Energieverbrauch.
    * **Fehlerbehandlung**:
        * Definition von Fehlerfällen (z.B. Hardwarefehler, Kommunikationsfehler, ungültige Einstellungen).
        * Verwendung von `Result` und spezifischen Fehler-Enums.
    * **Teststrategie**:
        * Unit-Tests für die Kernfunktionen des Power-Management-Services.
        * Integrationstests, die die Interaktion mit dem Betriebssystem simulieren.
        * Tests mit verschiedenen Hardware-Konfigurationen (z.B. verschiedene Batterietypen).
        * Tests für Suspend/Hibernate-Funktionalität (inklusive Aufwachen aus verschiedenen Zuständen).
        * Tests für die korrekte Anwendung von Energieprofilen und -richtlinien.
