# Technische Spezifikation: LLM-Integriertes Desktop-System mit MCP

**1. Einleitung**

**1.1 Projektübersicht (Technischer Fokus)**

Dieses Dokument definiert die technische Spezifikation für die Entwicklung einer Desktop-Anwendung (im Folgenden als "System" bezeichnet). Das Kernziel ist die Bereitstellung erweiterter Funktionalitäten durch die Integration lokaler oder cloudbasierter Large Language Models (LLMs). Der Zugriff auf diese LLMs wird über das Model Context Protocol (MCP) standardisiert und durch ein differenziertes Berechtigungssystem gesteuert. Die Systemarchitektur folgt einem klar definierten 4-Schichten-Modell.

**1.2 Architekturvorstellung**

Das System ist in vier logische Schichten unterteilt, um eine klare Trennung der Verantwortlichkeiten, hohe Kohäsion und lose Kopplung zu gewährleisten:

1. **Kernschicht (Core):** Enthält anwendungsunabhängige Logik, Datentypen und Algorithmen.
2. **Domänenschicht (Domain):** Beinhaltet die anwendungsspezifische Geschäftslogik, Regeln und Zustände.
3. **Systemschicht (System):** Implementiert Schnittstellen der Domänenschicht und handhabt die Kommunikation mit externen Systemen und Infrastruktur.
4. **Benutzeroberflächenschicht (UI):** Verantwortlich für die Präsentation von Informationen und die Entgegennahme von Benutzereingaben.

**1.3 Integration des Model Context Protocol (MCP)**

Die Integration des Model Context Protocol (MCP) ist ein zentrales Architekturelement.1 Es ermöglicht eine sichere und standardisierte Kommunikation zwischen der Anwendung (die als MCP-Client fungiert) und verschiedenen LLM-Diensten (MCP-Server). Dies umfasst Funktionalitäten wie Sprachsteuerung, Dateibearbeitung, Verzeichnisanalyse und die Anzeige benutzerdefinierter Webansichten innerhalb der Anwendungsoberfläche. Die Implementierung folgt den MCP-Spezifikationen und Best Practices für Sicherheit und Benutzerkontrolle.2

**1.4 Zielgruppe und Zweck**

Dieses Dokument dient als definitive technische Blaupause für das Entwicklungsteam. Es detailliert die Implementierungsanforderungen für jede Komponente und jedes Modul innerhalb der definierten Architektur. Gemäß Anforderung werden triviale Erklärungen und Begründungen ausgelassen; der Fokus liegt auf präzisen technischen Details für erfahrene Entwickler.

**1.5 Tabelle 1: Schichtenübersicht**

|   |   |   |
|---|---|---|
|**Schicht**|**Hauptverantwortung**|**Wichtige Technologien/Konzepte (Beispiele)**|
|Kern (Core)|Anwendungsunabhängige Logik, Datenstrukturen, Algorithmen. Keine externen Abhängigkeiten (außer Standardbibliothek/Basiskisten).|Basisdatentypen (Structs, Enums), generische Algorithmen, Kernfehlerdefinitionen.|
|Domäne (Domain)|Anwendungsspezifische Geschäftslogik, Regeln, Zustand, Orchestrierung. Hängt nur vom Kern ab.|Aggregates, Entities, Value Objects, Domain Services, Repository Interfaces, Domain Events, Berechtigungslogik.|
|System|Implementierung von Domain-Interfaces, Infrastruktur-Interaktion, externe Dienste.|Datenbankzugriff (SQL, ORM), Dateisystem-API, MCP-Client-Implementierung (SDK), D-Bus (zbus), Secret Service API, Input/Output-Sicherheit (`ammonia`, `shlex`).|
|Benutzeroberfläche (UI)|Präsentation, Benutzereingabe, UI-Framework-spezifischer Code.|UI-Framework (GTK, Tauri), Views, ViewModels/Controllers, Widgets, MCP Consent UI, Event Handling, Theming.|

**2. Schicht 1: Kernschicht Spezifikation (Core Layer Specification)**

**2.1 Verantwortlichkeiten**

Die Kernschicht bildet das Fundament des Systems. Sie enthält ausschließlich Code, der unabhängig von spezifischen Anwendungsfällen oder externen Systemen ist. Dazu gehören grundlegende Datenstrukturen, wiederverwendbare Algorithmen und Kernkonfigurationstypen. Diese Schicht darf keinerlei Abhängigkeiten zu den Domänen-, System- oder UI-Schichten aufweisen. Ebenso sind Abhängigkeiten zu spezifischen Frameworks (z.B. UI-Toolkits, Datenbank-ORMs) untersagt.

**2.2 Submodul-Definitionen**

- **2.2.1 Submodul 1.1: `Core.DataTypes` (Kerndatentypen)**
    
    - **Zweck:** Definition fundamentaler, wiederverwendbarer Datenstrukturen (Structs, Enums), die potenziell über Domänengrenzen hinweg genutzt werden, aber keine domänenspezifische Logik enthalten. Beispiele: `UserID`, `Timestamp`, `FilePath`, `PermissionLevel`.
    - **Komponenten:** Struct-Definitionen, Enum-Definitionen.
    - **Technische Details:** Strukturen sollten, wo sinnvoll, unveränderlich (immutable) sein. Falls diese Typen häufig über Schichtgrenzen oder Prozessgrenzen hinweg serialisiert werden, sind entsprechende Traits (z.B. `serde::Serialize`, `serde::Deserialize` in Rust) zu implementieren.
- **2.2.2 Submodul 1.2: `Core.Algorithms` (Kernalgorithmen)**
    
    - **Zweck:** Implementierung fundamentaler, wiederverwendbarer Algorithmen, die von spezifischen Anwendungsmerkmalen entkoppelt sind. Beispiele: Generische Sortier-/Suchfunktionen, Basis-Textverarbeitungsroutinen, grundlegende kryptographische Hilfsfunktionen (z.B. Hashing-Wrapper unter Verwendung von `ring`).
    - **Komponenten:** Funktionen, ggf. Hilfsklassen/-strukturen.
    - **Technische Details:** Algorithmische Komplexität (O-Notation) ist bei Bedarf zu dokumentieren. Externe Abhängigkeiten (z.B. `ring` Crate) sind explizit zu benennen.
- **2.2.3 Submodul 1.3: `Core.Configuration` (Kernkonfiguration)**
    
    - **Zweck:** Definition von Strukturen zur Aufnahme von Anwendungskonfigurations_werten_. Diese Schicht ist nicht für das Laden der Konfiguration verantwortlich (dies erfolgt in der Systemschicht). Repräsentiert Einstellungen, die das Kernverhalten beeinflussen können.
    - **Komponenten:** Structs, die Konfigurationsabschnitte repräsentieren.
    - **Technische Details:** Strikte Typisierung verwenden. Standardwerte definieren. Sicherstellen, dass die Strukturen leicht serialisierbar/deserialisierbar sind (z.B. via `serde`).
- **2.2.4 Submodul 1.4: `Core.ErrorHandling` (Kernfehlerbehandlung)**
    
    - **Zweck:** Definition von Basis-Fehlertypen oder Traits, die systemweit für eine konsistente Fehlerbehandlung und -weitergabe verwendet werden.
    - **Komponenten:** Enum-basierte Fehlertypen (z.B. `CoreError`), ggf. unter Verwendung von Bibliotheken wie `thiserror` in Rust.
    - **Technische Details:** Fehler-Varianten klar definieren. Sicherstellen, dass Standard-Error-Traits (z.B. `std::error::Error`) implementiert sind.
- **2.2.5 Submodul 1.5: `Core.Events` (Kernereignisse)**
    
    - **Zweck:** Definition fundamentaler Ereignisstrukturen, die potenziell von einem domänenspezifischen Event-Bus verwendet werden könnten, aber generisch genug für die Kernschicht sind. Beispiele: `ApplicationStartedEvent`, `ConfigurationChangedEvent`.
    - **Komponenten:** Structs, die Ereignisdaten repräsentieren.
    - **Technische Details:** Ereignisse sollten serialisierbar sein, falls sie Prozessgrenzen überqueren müssen (typischerweise werden sie jedoch innerhalb desselben Prozesses konsumiert).

Die strikte Trennung der Kernschicht gewährleistet maximale Wiederverwendbarkeit und Testbarkeit ihrer Komponenten, unabhängig von Änderungen in der UI oder der Infrastruktur. Diese Isolation ermöglicht Unit-Tests ohne die Notwendigkeit, komplexe externe Systeme zu mocken. Änderungen an UI-Frameworks oder Datenbanktechnologien in äußeren Schichten erfordern keine Anpassungen im Kern, was Wartungsaufwand und Risiko reduziert. Entwickler müssen daher sorgfältig darauf achten, keine Abhängigkeiten von äußeren Schichten _in_ die Kernschicht einzuführen; Code-Reviews müssen diese Grenze strikt durchsetzen.

**3. Schicht 2: Domänenschicht Spezifikation (Domain Layer Specification)**

**3.1 Verantwortlichkeiten**

Die Domänenschicht enthält die Essenz der Anwendung: die spezifische Geschäftslogik, Regeln und den Anwendungszustand. Sie orchestriert Kernfunktionalitäten und definiert das Verhalten des Systems. Diese Schicht hängt ausschließlich von der Kernschicht ab und ist unabhängig von UI- und Infrastrukturdetails.

**3.2 Submodul-Definitionen**

- **3.3.1 Submodul 2.1: `Domain.UserManagement` (Benutzerverwaltung)**
    
    - **Zweck:** Verwaltung von Benutzerprofilen, Authentifizierungszuständen (nicht der Authentifizierungsmechanismus selbst) und potenziell benutzerspezifischer Einstellungslogik.
    - **Komponenten:** `UserService` (Anwendungslogik), `UserRepository` (Interface für Persistenz), `User` Aggregate Root (zentrale Entität), Domain Events (z.B. `UserLoggedIn`, `UserProfileUpdated`).
    - **Technische Details:** Aggregate-Grenzen definieren. Validierungsregeln für Benutzerdaten spezifizieren (z.B. E-Mail-Format, Passwortstärke-Anforderungen – die eigentliche Hash-Berechnung erfolgt im System Layer). Repository-Interface-Methoden definieren (z.B. `findById`, `save`, `findByEmail`).
- **3.3.2 Submodul 2.2: `Domain.FileOperations` (Dateiverwaltung)**
    
    - **Zweck:** Definition der Domänenlogik für Dateioperationen, die über MCP angefordert werden könnten (z.B. Analyse von Verzeichnissen, potenziell Bearbeiten von Dateien). Definiert die _Absicht_ der Operation, führt aber keine tatsächlichen I/O-Operationen durch.
    - **Komponenten:** `FileOperationService`, `DirectoryAnalysisRequest` (Value Object), `FileEditCommand` (Command Object), `FileSystemRepository` (Interface für Dateisystemzugriff).
    - **Technische Details:** Definition von Commands und Value Objects, die Dateioperationen repräsentieren. Spezifikation von Vor- und Nachbedingungen für Operationen. Definition von Repository-Interface-Methoden (z.B. `getDirectoryContents`, `readFileContent`, `writeFileContent`).
- **3.3.3 Submodul 2.3: `Domain.LLMInteraction` (LLM-Interaktion)**
    
    - **Zweck:** Modellierung der Domänenkonzepte im Zusammenhang mit der Interaktion mit LLMs über MCP. Definiert, _was_ getan werden kann (z.B. Textgenerierung, Analyseaufgaben), aber nicht, _wie_ MCP technisch genutzt wird.
    - **Komponenten:** `LLMTaskService`, `LLMTask` (Entity/Value Object), `PromptTemplate` (Value Object), `LLMInteractionRepository` (Interface für die Ausführung).
    - **Technische Details:** Definition von Strukturen für verschiedene LLM-Aufgabentypen (z.B. `SummarizationTask`, `CodeGenerationTask`). Definition des Repository-Interfaces (`executeTask`).
- **3.3.4 Submodul 2.4: `Domain.Permissions` (Berechtigungslogik)**
    
    - **Zweck:** Implementierung der Kernlogik für das geforderte "clevere Berechtigungssystem". Bestimmt, ob ein Benutzer oder eine Sitzung das Recht hat, spezifische Aktionen durchzuführen (z.B. Zugriff auf ein bestimmtes MCP-Tool, Lesen eines bestimmten Dateityps).
    - **Komponenten:** `PermissionService`, `PermissionPolicy`, `RequiredPermission` (Value Object), `PermissionRepository` (Interface zum Laden von Rollen/Berechtigungen).
    - **Technische Details:** Definition der Berechtigungsprüfungslogik, z.B. mittels Role-Based Access Control (RBAC). Spezifikation, wie Berechtigungen strukturiert und gegen Benutzerrollen oder -attribute ausgewertet werden. Definition des Repository-Interfaces (`getUserPermissions`).
- **3.3.5 Submodul 2.5: `Domain.VoiceControl` (Sprachsteuerung)**
    
    - **Zweck:** Definition der Domänenlogik zur Interpretation von Sprachbefehlen und deren Übersetzung in Anwendungsaktionen oder LLM-Aufgaben.
    - **Komponenten:** `VoiceCommandParser` (Interface/Implementierung), `VoiceCommandInterpreterService`, `VoiceCommandRepository` (Interface, z.B. für benutzerdefinierte Befehle).
    - **Technische Details:** Definition der Struktur für geparste Sprachbefehle. Spezifikation der Logik zur Zuordnung von Befehlen zu Aktionen/Aufgaben. Definition des Repository-Interfaces (`getCustomCommands`).
- **3.3.6 Submodul 2.6: `Domain.WebViewWidget` (Webansicht-Widget Logik)**
    
    - **Zweck:** Handhabt die Domänenlogik im Zusammenhang mit der benutzerdefinierten Webansicht, die über MCP angefordert werden kann (z.B. Definition, welche Inhalte angezeigt werden dürfen, Verwaltung des Zustands der Ansicht).
    - **Komponenten:** `WebViewService`, `WebViewContentPolicy`, `WebViewState`.
    - **Technische Details:** Definition von Richtlinien für erlaubte URLs oder Inhaltstypen. Spezifikation der Zustandsverwaltungslogik für die Webansicht.

Die Domänenschicht kapselt den Kernwert und die Komplexität der Anwendung. Die Definition klarer Schnittstellen (Repositories) für externe Abhängigkeiten (wie Persistenz oder die tatsächliche MCP-Kommunikation) ist entscheidend für die Entkopplung. Diese Interfaces erlauben der Domänenschicht, ihre _Bedürfnisse_ auszudrücken (z.B. "speichere Benutzer", "führe LLM-Aufgabe aus"), ohne die konkrete _Implementierung_ zu kennen. Die Systemschicht liefert dann die Implementierungen. Dies folgt dem Dependency Inversion Principle und macht die Domänenschicht testbar und unabhängig von Infrastrukturentscheidungen. Das Submodul `Domain.Permissions` ist zentral für die Umsetzung des geforderten Berechtigungssystems, das den Zugriff auf MCP-Funktionen steuert. Diese Kontrolllogik ist eine Kerngeschäftsregel und gehört daher in die Domänenschicht, getrennt von der technischen Authentifizierung (System) oder der Einholung von Zustimmungen (UI/System). Das Design der Repository-Interfaces muss sorgfältig erfolgen, um die notwendigen Abstraktionen zu bieten, ohne Implementierungsdetails preiszugeben.

**4. Schicht 3: Systemschicht Spezifikation (System Layer Specification)**

**4.1 Verantwortlichkeiten**

Die Systemschicht fungiert als Brücke zwischen der Domänenschicht und der Außenwelt. Sie implementiert die von der Domänenschicht definierten Interfaces (z.B. Repositories) und handhabt die technische Kommunikation mit externen Systemen und Diensten. Dazu gehören Datenbanken, das Dateisystem, Netzwerkdienste (insbesondere MCP-Server) und Betriebssystemdienste (wie der D-Bus für die Secret Service API). Diese Schicht enthält infrastruktur-spezifischen Code und hängt von der Domänen- und Kernschicht ab.

**4.2 Submodul-Definitionen**

- **4.3.1 Submodul 3.1: `System.Persistence` (Persistenz)**
    
    - **Zweck:** Bereitstellung konkreter Implementierungen für Repository-Interfaces aus der Domänenschicht (z.B. `UserRepository`, `PermissionRepository`). Interagiert mit dem gewählten Datenbanksystem.
    - **Komponenten:** `SqlUserRepository` (implementiert `Domain.UserManagement.UserRepository`), `DatabaseClientWrapper`, ORM-Entitäten/Mappings (falls ORM genutzt wird).
    - **Technische Details:** Spezifikation des Datenbanktyps (z.B. PostgreSQL, SQLite). Detaillierung relevanter Schema-Ausschnitte. Spezifikation des ORMs oder Datenbanktreibers (z.B. `sqlx`, `diesel` in Rust). Definition der Connection-Pooling-Strategie.
- **4.3.2 Submodul 3.2: `System.FileSystemAccess` (Dateisystemzugriff)**
    
    - **Zweck:** Implementiert das `FileSystemRepository`-Interface aus `Domain.FileOperations`. Führt tatsächliche Datei-I/O-Operationen durch.
    - **Komponenten:** `LocalFileSystemRepository` (implementiert `Domain.FileOperations.FileSystemRepository`).
    - **Technische Details:** Verwendung von Standardbibliotheksfunktionen für Dateizugriff (z.B. `std::fs` in Rust). Implementierung der Fehlerbehandlung für I/O-Ausnahmen. **Wichtig:** Falls Dateipfade oder verwandte Argumente (die aus Dateioperationen stammen) an externe Shell-Befehle übergeben werden, muss Shell Argument Escaping mittels der `shlex`-Bibliothek implementiert werden, um Command Injection zu verhindern.4
- **4.3.3 Submodul 3.3: `System.MCP.Client` (MCP Client Implementierung)**
    
    - **Zweck:** Implementiert das `LLMInteractionRepository`-Interface. Handhabt die technischen Details der MCP-Kommunikation: Verbindungsaufbau, Serialisierung/Deserialisierung von Nachrichten, Aufruf von MCP Resources und Tools. Fungiert als MCP _Host_ oder _Client_ gemäß MCP-Terminologie.2
    - **Komponenten:** `MCPClientService` (implementiert `Domain.LLMInteraction.LLMInteractionRepository`), `MCPConnectionManager`, `MCPMessageSerializer`.
    - **Technische Details:** Nutzung des offiziellen MCP SDK für Rust (`modelcontextprotocol/rust-sdk` 1). Implementierung des Verbindungslebenszyklus (Verbinden, Trennen, Wiederverbinden). Handhabung der JSON-RPC 2.0 Nachrichtenübermittlung über WebSockets.2 Implementierung der Logik zur Interaktion mit MCP `Resources`, `Tools` und potenziell `Prompts`.2 Verwaltung des Sitzungszustands, falls erforderlich.6 Implementierung von Rate Limiting und Timeouts für MCP-Anfragen.3
- **4.3.4 Submodul 3.4: `System.Security.Credentials` (Sichere Speicherung)**
    
    - **Zweck:** Sicheres Speichern und Abrufen sensibler Daten wie API-Schlüssel oder Tokens, die für den Zugriff auf MCP-Server oder andere Dienste benötigt werden. Implementiert potenziell ein in der Domäne definiertes Interface oder wird direkt von anderen Systemmodulen genutzt.
    - **Komponenten:** `SecretServiceClient`, `CredentialManager`.
    - **Technische Details:** Nutzung der D-Bus Secret Service API auf Linux/Desktop-Umgebungen.7 Verwendung der `zbus`-Bibliothek für die D-Bus-Kommunikation aufgrund ihrer reinen Rust-Implementierung und async-Unterstützung.9 Implementierung von Methoden, die den Secret Service API-Aufrufen entsprechen, wie `CreateItem`, `SearchItems`, `RetrieveSecrets`.7 Speicherung der Credentials in der Standard-Collection des Benutzers (`/org/freedesktop/secrets/aliases/default`), sofern keine spezifischen Anforderungen etwas anderes vorschreiben.7 Behandlung potenzieller Fehler wie gesperrte Keyrings.
- **4.3.5 Submodul 3.5: `System.Security.InputOutput` (Ein-/Ausgabe-Sicherheit)**
    
    - **Zweck:** Bereitstellung von Diensten zur Bereinigung (Sanitization) und Validierung von Daten, die von externen Quellen in das System gelangen (z.B. LLM-Antworten zur Anzeige oder Ausführung) und potenziell von Daten, die das System verlassen.
    - **Komponenten:** `HtmlSanitizerService`, `CommandArgumentSanitizer`.
    - **Technische Details:** Für HTML-Inhalte, die von LLMs oder MCP-Webansichten empfangen werden, ist die `ammonia`-Bibliothek in Rust zu verwenden.11 Diese ermöglicht eine robuste, Whitelist-basierte Bereinigung mit einer strikten Konfiguration (ähnliche Prinzipien wie beim OWASP Java Sanitizer 12), um Cross-Site Scripting (XSS) zu verhindern.13 Für Argumente, die an Shell-Befehle übergeben werden (z.B. über `System.FileSystemAccess`), ist die `shlex`-Bibliothek in Rust für korrektes Escaping zu verwenden, um Command Injection zu verhindern.4 Implementierung von Validierungslogik basierend auf erwarteten Datenformaten (z.B. mittels JSON Schema Validierung oder Konzepten wie `guardrails-ai` 14 für LLM-Ausgabestrukturen). Anwendung eines Zero-Trust-Ansatzes auf LLM-Ausgaben.13
- **4.3.6 Submodul 3.6: `System.ConfigurationLoader` (Konfigurationslader)**
    
    - **Zweck:** Lädt die Anwendungskonfiguration aus Dateien oder Umgebungsvariablen und füllt die in `Core.Configuration` definierten Strukturen.
    - **Komponenten:** `ConfigFileLoader`, `EnvVarLoader`.
    - **Technische Details:** Spezifikation des Konfigurationsdateiformats (z.B. TOML, YAML). Verwendung von Bibliotheken wie `config-rs` in Rust. Handhabung der Ladereihenfolge und von Overrides.
- **4.3.7 Submodul 3.7: `System.IPC.DBus` (D-Bus Kommunikation)**
    
    - **Zweck:** Verwaltung allgemeiner D-Bus-Verbindungen und Interaktionen über den Secret Service hinaus, falls für andere Integrationen erforderlich (z.B. Desktop-Benachrichtigungen, Mediensteuerung).
    - **Komponenten:** `DBusConnectionService`.
    - **Technische Details:** Nutzung der `zbus`-Bibliothek.9 Verwaltung des Verbindungsaufbaus und -lebenszyklus. Bereitstellung von Wrappern für gängige D-Bus-Muster (Methodenaufrufe, Signal-Empfang).

Diese Schicht bildet die entscheidende Verbindung zwischen der abstrakten Domänenlogik und der konkreten externen Welt. Ihre Korrektheit ist für Sicherheit und Funktionalität von zentraler Bedeutung. Während die Domänenschicht definiert, _was_ geschehen muss, implementiert die Systemschicht das _Wie_ unter Verwendung spezifischer Technologien. Diese Trennung lokalisiert Infrastrukturabhängigkeiten, was Anpassungen (z.B. Datenbankwechsel) und Tests (durch Mocking von Systemkomponenten) erleichtert. Fehler in dieser Schicht (z.B. unzureichende SQL-Injection-Prävention, fehlerhafte MCP-Nachrichtenformatierung) wirken sich jedoch direkt auf Funktion und Sicherheit aus. Die Integration externer Sicherheitsbibliotheken (`ammonia`, `shlex`) und OS-Dienste (Secret Service via `zbus`) in dieser Schicht zentralisiert kritische Sicherheitsmechanismen und verhindert deren Verstreuung im Code. Gründliche Tests, einschließlich Sicherheitstests, sind für Komponenten der Systemschicht unerlässlich. Die Konfiguration von Sicherheitsbibliotheken (z.B. `ammonia`-Richtlinien) muss strikt sein und sorgfältig überprüft werden. Die Fehlerbehandlung für externe Interaktionen muss robust sein.

**5. Schicht 4: Benutzeroberflächenschicht Spezifikation (UI Layer Specification)**

**5.1 Verantwortlichkeiten**

Die Benutzeroberflächenschicht (UI) ist für die Interaktion mit dem Benutzer verantwortlich. Sie präsentiert Informationen und nimmt Benutzereingaben entgegen. Sie interagiert typischerweise mit der System- oder Domänenschicht (oft über Application Services oder dedizierte ViewModels/Controller), um Daten abzurufen und Aktionen auszulösen. Diese Schicht enthält den UI-Framework-spezifischen Code.

**5.2 Submodul-Definitionen**

- **5.3.1 Submodul 4.1: `UI.MainWindow` (Hauptfenster)**
    
    - **Zweck:** Definition der Struktur des Hauptanwendungsfensters, des Layouts und der primären Navigationselemente (z.B. Seitenleiste, Menüleiste).
    - **Komponenten:** `MainWindowView`, `MainWindowViewModel` (oder Controller), `SidebarComponent`, `MenuBarComponent`.
    - **Technische Details:** Spezifikation des UI-Frameworks (z.B. GTK über `gtk-rs`, Tauri mit Web-Frontend, Qt). Definition der Layoutstruktur (z.B. mittels GtkBuilder UI-Definitionen, HTML/CSS in Tauri, oder programmatisch). Implementierung von Data Binding zwischen View und ViewModel. Handhabung grundlegender Fensterereignisse. Konzepte zur Organisation von UI-Kontexten wie "Tab Islands" 15 oder "Spaces" 16 können mittels der Fähigkeiten des gewählten UI-Frameworks implementiert werden (z.B. durch Tab-Container, Ansichtswechsel-Logik).
- **5.3.2 Submodul 4.2: `UI.Views.[Feature]` (Feature-Ansichten)**
    
    - **Zweck:** Definition spezifischer Ansichten für verschiedene Anwendungsfunktionen (z.B. Benutzerprofil-Editor, Dateibrowser-Ansicht, LLM-Chat-Interface).
    - **Komponenten:** `UserProfileView`, `UserProfileViewModel`, `FileBrowserView`, `FileBrowserViewModel`, etc.
    - **Technische Details:** Definition der UI-Elemente für jede Ansicht. Implementierung von Data Binding. Handhabung von Benutzerinteraktionen (Button-Klicks, Texteingabe) und Delegation von Aktionen an das ViewModel/Controller.
- **5.3.3 Submodul 4.3: `UI.MCP.Consent` (MCP Consent Dialoge)**
    
    - **Zweck:** Implementierung der Benutzeroberflächenelemente, die für die MCP-Zustimmungsflüsse gemäß der MCP-Spezifikation erforderlich sind.2 Präsentiert dem Benutzer Anfragen für Datenzugriff, Werkzeugausführung und Sampling zur Genehmigung.
    - **Komponenten:** `MCPConsentDialogView`, `MCPConsentViewModel`, `PermissionRequestDisplayComponent`.
    - **Technische Details:** Gestaltung klarer und unmissverständlicher Dialoge, die erklären, _welche_ Berechtigung angefordert wird, _welcher_ MCP-Server sie anfordert und (wenn möglich) _warum_. Bereitstellung klarer "Erlauben" / "Ablehnen"-Optionen. Implementierung der Logik zur Auslösung dieser Dialoge basierend auf Signalen vom `System.MCP.Client` oder der Domänenschicht. Sicherstellung, dass Benutzerentscheidungen sicher zurückgemeldet werden. Diese Komponente ist kritisch für die Erfüllung der MCP Host-Verantwortlichkeiten.2
- **5.3.4 Submodul 4.4: `UI.Widgets.WebView` (Webansicht Widget)**
    
    - **Zweck:** Implementierung der UI-Komponente zur Anzeige der benutzerdefinierten Webansicht, die über MCP angefordert werden kann.
    - **Komponenten:** `WebViewWidgetComponent`.
    - **Technische Details:** Nutzung der Web-View-Komponente des UI-Frameworks (z.B. `WebKitGTK`, `WebView2` via Tauri). Implementierung einer Kommunikationsbrücke, falls Interaktion zwischen Webinhalt und Hauptanwendung erforderlich ist. **Wichtig:** Sicherstellen, dass jeder geladene HTML-Inhalt (insbesondere wenn er durch LLM-Ausgaben oder MCP beeinflusst wird) entweder aus einer vertrauenswürdigen Quelle stammt oder vor dem Rendern durch `System.Security.InputOutput.HtmlSanitizerService` bereinigt wird, um XSS zu verhindern.13
- **5.3.5 Submodul 4.5: `UI.Theming` (Theming/Styling)**
    
    - **Zweck:** Verwaltung des visuellen Erscheinungsbilds (Farben, Schriftarten, Stile) der Anwendung.
    - **Komponenten:** CSS-Dateien, Stildefinitionen, Theme-Manager-Service.
    - **Technische Details:** Spezifikation des Styling-Mechanismus (z.B. CSS, QSS). Definition der Theme-Struktur. Implementierung der Logik zum Wechseln von Themes (z.B. Hell/Dunkel-Modus, ähnlich wie in Arc 16).
- **5.3.6 Submodul 4.6: `UI.Notifications` (Benachrichtigungen)**
    
    - **Zweck:** Anzeige von Benachrichtigungen für den Benutzer (z.B. Abschluss von Operationen, Fehler, MCP-Ereignisse).
    - **Komponenten:** `NotificationView`, `NotificationService`.
    - **Technische Details:** Nutzung des Benachrichtigungssystems des UI-Frameworks oder Integration mit Desktop-Benachrichtigungsstandards (potenziell über `System.IPC.DBus`).

Die UI-Schicht ist der primäre Interaktionspunkt für das benutzerzentrierte Sicherheitsmodell von MCP (Zustimmung). Ihr Design beeinflusst direkt die Benutzerfreundlichkeit und die Wirksamkeit der Sicherheitsmaßnahmen. Da MCP explizite Benutzerzustimmung für kritische Operationen vorschreibt 2, ist das `UI.MCP.Consent`-Submodul nicht nur ein UI-Feature, sondern eine kritische Sicherheitskomponente. Schlecht gestaltete Zustimmungsdialoge können dazu führen, dass Benutzer Berechtigungen erteilen, die sie nicht verstehen, was das Sicherheitsmodell untergräbt. Klare, informative und kontextbezogene Zustimmungsaufforderungen sind daher unerlässlich. Darüber hinaus erfordert die Anzeige potenziell von LLMs generierter Inhalte (z.B. in Chat-Ansichten oder dem WebView-Widget) eine sorgfältige Behandlung, um clientseitige Angriffe wie XSS zu verhindern. LLMs können Ausgaben mit Markup oder Code generieren.13 Wenn diese direkt im UI gerendert werden, ohne Bereinigung, könnte schädlicher Inhalt im Kontext des Benutzers ausgeführt werden. Daher MÜSSEN alle Komponenten, die potenziell unsichere Inhalte rendern, die Bereinigungsdienste der Systemschicht (`System.Security.InputOutput.HtmlSanitizerService`) nutzen.11 Die UI-Entwicklung muss Klarheit und Sicherheit priorisieren, insbesondere bei Zustimmungsflüssen und der Darstellung externer Inhalte.

**6. Querschnittsthema: Model Context Protocol (MCP) Integration**

**6.1 Architekturüberblick**

Die MCP-Integration ist ein Querschnittsthema, das mehrere Schichten durchdringt: Die UI-Schicht ist für die Einholung der Benutzerzustimmung (`UI.MCP.Consent`) verantwortlich. Die Systemschicht implementiert den eigentlichen MCP-Client (`System.MCP.Client`), handhabt die sichere Kommunikation und die Ein-/Ausgabe-Sicherheit (`System.Security.*`). Die Domänenschicht definiert die Logik der LLM-Interaktionen (`Domain.LLMInteraction`) und die Berechtigungsregeln (`Domain.Permissions`). Das Modul `System.MCP.Client` agiert als MCP Host/Client im Sinne der MCP-Spezifikation.2

**6.2 MCP Client Implementierung (UI & Core Apps)**

- **SDK-Wahl:** Das offizielle MCP SDK für Rust (`modelcontextprotocol/rust-sdk`) wird verwendet.1
- **Verbindungsmanagement:** Implementierung in `System.MCP.Client`. Umfasst den Aufbau von WebSocket-Verbindungen zu MCP-Servern (lokal oder Cloud), Fehlerbehandlung bei Verbindungsabbrüchen, Wiederverbindungslogik und die Sicherstellung sicherer Verbindungen mittels TLS.
- **Resource/Tool Handling:** Der Client (`System.MCP.Client`) implementiert die Logik zur Entdeckung und Interaktion mit `Resources` (Bereitstellung von Kontext für LLMs) und `Tools` (Ausführung von Aktionen), die vom MCP-Server angeboten werden.2 MCP-Tool-Aufrufe werden an entsprechende Aktionen in der Domänen- oder Systemschicht gemappt.
- **Sampling Handling:** Implementierung der clientseitigen Logik zur Handhabung von server-initiierten `sampling`-Anfragen.2 Der Prozess umfasst:
    1. Empfang der Sampling-Anfrage durch `System.MCP.Client`.
    2. Auslösen des `UI.MCP.Consent`-Flusses zur Einholung der Benutzergenehmigung. Der Benutzer MUSS explizit zustimmen.2
    3. Dem Benutzer SOLLTE die Möglichkeit gegeben werden, den zu sendenden Prompt zu überprüfen und zu kontrollieren.2
    4. Senden des Prompts an das LLM (entweder über eine weitere MCP-Interaktion oder direkt, abhängig von der Architektur).
    5. Kontrolle darüber, welche Ergebnisse der Sampling-Operation an den anfragenden MCP-Server zurückgesendet werden dürfen (Benutzerkontrolle über `UI.MCP.Consent`).2

**6.3 Interaktion mit MCP Servern**

- **Protokolldetails:** Strikte Einhaltung von JSON-RPC 2.0 über WebSocket.2 Definition der erwarteten Nachrichtenformate für Anfragen und Antworten bezüglich benutzerdefinierter Tools und Ressourcen.
- **Datenflüsse:** Klare Definition und ggf. Diagramme der Datenflüsse für Schlüsselinteraktionen. Beispiel: Benutzer fordert Verzeichnisanalyse an -> UI sendet Anfrage -> Domänenlogik (`Domain.FileOperations`) -> System ruft MCP Tool über `System.MCP.Client` auf -> MCP Server führt Analyse durch -> Antwort über MCP -> UI zeigt Ergebnis an.
- **Server Discovery/Configuration:** Die Konfiguration, zu welchen MCP-Servern (lokale LLM-Wrapper, Cloud-Dienste) eine Verbindung hergestellt werden soll, erfolgt über `System.ConfigurationLoader`, basierend auf Konfigurationsdateien oder Umgebungsvariablen.

**6.4 Sicherheit & Berechtigungen**

Die sichere Integration von MCP erfordert einen mehrschichtigen Ansatz, der über die reine Protokollimplementierung hinausgeht.

- **Authentifizierungs-/Autorisierungsstrategie:**
    
    - _Client-Authentifizierung:_ Falls MCP-Server eine Authentifizierung des Clients (dieser Anwendung) erfordern, sind Mechanismen wie API-Schlüssel oder Tokens zu verwenden. Diese Credentials MÜSSEN sicher über `System.Security.Credentials` (Secret Service API) gespeichert werden.7 Standardisierte Protokolle wie OAuth 2.0 oder JWTs sollten bevorzugt werden, wenn vom Server unterstützt.3
    - _Benutzer-Authentifizierung:_ Die Authentifizierung des Benutzers _innerhalb_ der Anwendung wird durch `Domain.UserManagement` und entsprechende System-Layer-Mechanismen gehandhabt und ist von der MCP-Client-Authentifizierung getrennt.
- **Consent Management Flow:** Der Prozess zur Einholung der Benutzerzustimmung ist zentral für die MCP-Sicherheit 2:
    
    1. Ein MCP-Server fordert Zugriff auf eine Ressource, ein Tool oder initiiert Sampling. `System.MCP.Client` empfängt die Anfrage.
    2. Die System-/Domänenschicht prüft, ob für diese spezifische Aktion und diesen Server eine Zustimmung erforderlich ist (basierend auf der Aktion und ggf. gecachten Benutzerentscheidungen) und ob der Benutzer gemäß `Domain.Permissions` überhaupt dazu berechtigt ist.
    3. Falls Zustimmung benötigt wird, wird `UI.MCP.Consent` ausgelöst, um eine klare und verständliche Anfrage anzuzeigen.
    4. Der Benutzer erteilt oder verweigert die Erlaubnis über die UI.
    5. Die Entscheidung wird sicher gehandhabt (z.B. temporär in der Sitzung oder persistent in Benutzereinstellungen).
    6. Die Aktion wird basierend auf der Zustimmung ausgeführt oder abgelehnt.
    
    - Dieser Fluss implementiert die Kernprinzipien von MCP.2 Granularität (Zustimmung pro Tool/Ressourcentyp/Server) ist anzustreben.
- **Eingabevalidierung/-sanitisierung:**
    
    - _Prompt Injection Abwehr:_ Bevor Prompts (aus Benutzereingaben oder MCP-Interaktionen konstruiert) an ein LLM gesendet werden, MÜSSEN Filterung und Kontexttrennung implementiert werden. Techniken wie die Kennzeichnung der Vertrauenswürdigkeit von Eingabequellen (Trennung von Benutzer-Prompts und potenziell unvertrauenswürdigen Daten aus MCP-Ressourcen) sind anzuwenden.18 Parameter, die an MCP-Tools übergeben werden, MÜSSEN rigoros validiert werden (`System.MCP.Client` oder empfangendes Systemmodul).3 Tool-Beschreibungen von Servern sind als potenziell unvertrauenswürdig zu betrachten, es sei denn, der Server ist verifiziert.2 Maßnahmen gegen OWASP LLM Top 10 Risiken wie Prompt Injection sind zu implementieren.18
    - _Parameter Validation:_ Typen, Bereiche, Formate und Größen von Parametern, die an MCP-Tools gesendet werden, sind zu validieren.3
- **Ausgabeverarbeitung:**
    
    - _LLM Response Validation/Sanitization:_ Alle über MCP empfangenen LLM-Ausgaben sind als unvertrauenswürdig zu behandeln.13 Die Struktur ist zu validieren, wenn ein bestimmtes Format erwartet wird (z.B. JSON 14). HTML/Markdown MUSS mittels `System.Security.InputOutput.HtmlSanitizerService` (`ammonia` 11) bereinigt werden, bevor es im UI gerendert wird, um XSS zu verhindern.12 Auf Inkonsistenzen oder potenzielle Halluzinationen ist, wo möglich, zu prüfen.13 Unbeabsichtigte Befehlsausführung basierend auf der Ausgabe ist zu verhindern.
    - _Tool Output Validation:_ Struktur und Inhalt der von MCP-Tool-Ausführungen empfangenen Ergebnisse sind zu validieren.3
- **Sichere Speicherung von Credentials:** Erneute Betonung der Verwendung von `System.Security.Credentials` mit der D-Bus Secret Service API über `zbus` 7 zur Speicherung von Authentifizierungsdaten für MCP-Server.
    
- **Least Privilege:** Das Prinzip der geringsten Rechte ist durchzusetzen. Die Anwendung (als MCP Host/Client) sollte nur die Berechtigungen anfordern, die sie benötigt. Die Logik in `Domain.Permissions` stellt sicher, dass Benutzer/Sitzungen nur mit den minimal notwendigen Rechten operieren.18 Für risikoreiche Aktionen ist eine menschliche Bestätigung über `UI.MCP.Consent` (Human-in-the-Loop) unerlässlich.18
    
- **6.4.1 Tabelle 3: MCP Sicherheitsmaßnahmen**
    

|   |   |   |   |
|---|---|---|---|
|**Risikobereich**|**Maßnahme**|**Verantwortliche(s) Modul(e)**|**Referenz (Beispiele)**|
|Prompt Injection|Eingabefilterung, Kontexttrennung (User vs. External Data), Parameter-Validierung, Tool-Beschreibungen als unsicher behandeln.|`System.Security.InputOutput`, `System.MCP.Client`|3|
|Unsichere Tool-Ausführung|Explizite Benutzerzustimmung (Consent UI), Berechtigungsprüfung (RBAC), Parameter-Validierung, Rate Limiting, Timeouts.|`UI.MCP.Consent`, `Domain.Permissions`, `System.MCP.Client`|2|
|Datenschutzverletzung|Explizite Benutzerzustimmung für Datenzugriff/Übertragung, Sichere Speicherung von Credentials, Zugriffskontrolle.|`UI.MCP.Consent`, `System.Security.Credentials`, `Domain.Permissions`|2|
|Unsichere Ausgabeverarbeitung|Zero-Trust für LLM/Tool-Output, Output-Sanitization (HTML/Markdown), Output-Validierung (Struktur, Inhalt), Verhinderung von Code Execution.|`System.Security.InputOutput`, `UI.Widgets.WebView`, UI-Komponenten (z.B. Chat)|11|
|Unautorisierter Zugriff|Client-Authentifizierung bei MCP-Servern (Tokens/Keys), Benutzer-Authentifizierung in der App, RBAC, Least Privilege Prinzip.|`System.Security.Credentials`, `Domain.UserManagement`, `Domain.Permissions`, `System.MCP.Client`|3|
|Unerwünschtes Sampling|Explizite Benutzerzustimmung pro Anfrage, Benutzerkontrolle über Prompt & Ergebnis-Sichtbarkeit.|`UI.MCP.Consent`, `System.MCP.Client`|2|

Eine sichere MCP-Integration ist nicht nur eine Frage der Protokollimplementierung, sondern erfordert einen ganzheitlichen Sicherheitsansatz. Dieser umfasst robuste Eingabevalidierung, Ausgabebereinigung, klare und sichere Zustimmungsmechanismen sowie sicheres Credential Management. Dabei müssen sowohl allgemeine Best Practices der Anwendungssicherheit als auch LLM-spezifische Risiken berücksichtigt werden.2 Das "clevere Berechtigungssystem" ist untrennbar mit dem MCP-Zustimmungsfluss verbunden. Die in `Domain.Permissions` definierten Berechtigungen müssen die Notwendigkeit und Granularität der Zustimmung beeinflussen, die auf der UI/System-Ebene für MCP-Aktionen eingeholt wird. Berechtigungen gewähren die generelle Fähigkeit, während die Zustimmung die spezifische Ausführung autorisiert. Dies erfordert eine sorgfältige Koordination zwischen `Domain.Permissions`, `System.MCP.Client` und `UI.MCP.Consent`. Die Sicherheit der gesamten Kette hängt vom schwächsten Glied ab.

**7. Anhang**

**7.1 Verwendete Technologien und Bibliotheken (Auswahl)**

- **Programmiersprache:** Rust
- **MCP Integration:** `modelcontextprotocol/rust-sdk` 1
- **D-Bus Kommunikation:** `zbus` 9
- **Sichere Speicherung (Linux):** D-Bus Secret Service API (via `zbus`) 7
- **HTML Sanitization:** `ammonia` 11
- **Shell Argument Escaping:** `shlex` 4
- **UI Framework:** Zu spezifizieren (Optionen: GTK via `gtk-rs`, Tauri, Qt via Bindings)
- **Datenbankzugriff:** Zu spezifizieren (Optionen: `sqlx`, `diesel`)
- **Konfiguration:** `config-rs` (oder äquivalent)
- **Fehlerbehandlung:** `thiserror` (oder äquivalent)
- **Serialisierung:** `serde`