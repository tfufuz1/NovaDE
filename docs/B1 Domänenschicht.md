# Implementierungsplan Domänenschicht: Teil 1 – Kerndomäne, Konfiguration und Basissystemintegration

## I. Einleitung

### A. Zielsetzung des Dokuments

Dieses Dokument legt den detaillierten Implementierungsplan für den ersten Teil der Domänenschicht einer neu zu entwickelnden Softwareanwendung dar. Ziel ist es, eine robuste, wartbare und erweiterbare Grundlage für die Geschäftslogik zu schaffen. Der Fokus liegt auf der Definition der Kernkomponenten der Domäne, dem Aufbau eines soliden Konfigurationsmanagements, der Implementierung einer umfassenden Fehlerbehandlungs- und Logging-Strategie sowie der Integration grundlegender Systemdienste und der Etablierung von API-Designrichtlinien.

### B. Umfang und Abgrenzung

Teil 1 dieses Implementierungsplans umfasst die Entwicklung der fundamentalen Domänenlogik, einschließlich Entitäten, Wertobjekten und Repository-Schnittstellen. Des Weiteren werden das Konfigurationsmanagement, die Fehlerbehandlung und das Logging spezifiziert. Die Integration mit Systemdiensten beschränkt sich auf die Geheimnisverwaltung und die Grundlagen der Interprozesskommunikation (IPC) via D-Bus. Schließlich werden API-Designrichtlinien und projektinterne Konventionen festgelegt. Ausdrücklich nicht Teil dieses ersten Plans sind die Entwicklung der Benutzeroberfläche (UI), die Implementierung spezifischer Anwendungsdienste, die über die reine Domänenlogik hinausgehen, sowie alle Funktionalitäten, die für Teil 2 des Implementierungsplans vorgesehen sind.

### C. Zielgruppe

Dieses Dokument richtet sich primär an Softwareentwickler und -architekten, die an der Konzeption und Implementierung der Domänenschicht beteiligt sind. Es dient als technische Grundlage und Referenz für die Entwicklung.

### D. Methodik

Die Erstellung dieses Plans basiert auf einer sorgfältigen Analyse der funktionalen und nicht-funktionalen Anforderungen an die Domänenschicht. Dies beinhaltet die Auswertung von Best Practices im Bereich Softwarearchitektur und Domänendesign, insbesondere im Kontext der Programmiersprache Rust. Eine Auswahl etablierter Bibliotheken (Crates) und Technologien wurde getroffen, um eine hohe Qualität und Effizienz in der Entwicklung sicherzustellen.

## II. Grundlegende Design-Prinzipien und Architektur

### A. Architekturüberblick

Die Domänenschicht wird das Herzstück der Anwendung bilden und die Geschäftslogik sowie die Domänenobjekte kapseln. Sie wird streng nach den Prinzipien der Clean Architecture (alternativ als Hexagonal Architecture oder Ports & Adapters bekannt) entworfen. Dieses Architekturmuster gewährleistet eine klare Trennung der Belange und macht die Domänenschicht unabhängig von äußeren Schichten wie der Benutzeroberfläche, Datenbankimplementierungen oder externen Frameworks. Die Domänenschicht definiert ihre eigenen Schnittstellen (Ports), über die sie mit anderen Schichten kommuniziert, welche die entsprechenden Adapter implementieren. Diese Entkopplung fördert die Testbarkeit, Wartbarkeit und Flexibilität des Systems, da technologische Entscheidungen in den äußeren Schichten geändert werden können, ohne die Kernlogik der Domäne zu beeinträchtigen.

### B. Sprach- und Werkzeugwahl

Als primäre Programmiersprache für die Implementierung der Domänenschicht wurde Rust gewählt. Rust bietet eine einzigartige Kombination aus Performance, Speichersicherheit ohne Garbage Collector und modernen Sprachfeatures, die es besonders geeignet für systemnahe Programmierung und die Entwicklung komplexer, zuverlässiger Anwendungen machen.1 Die strengen Typsystem- und Ownership-Regeln von Rust helfen, viele gängige Programmierfehler bereits zur Kompilierzeit zu verhindern.3 Das Ökosystem von Rust, insbesondere die Verfügbarkeit hochwertiger Crates, unterstützt die schnelle Entwicklung robuster Software.

Für das Build-System und die Paketverwaltung wird Cargo, das offizielle Werkzeug von Rust, eingesetzt.5 Cargo vereinfacht das Kompilieren von Code, das Verwalten von Abhängigkeiten und das Ausführen von Tests erheblich. Zur Sicherstellung einer einheitlichen Codeformatierung wird `rustfmt` mit den Standardeinstellungen verbindlich eingesetzt. Zusätzlich wird `clippy`, ein Linter für Rust, verwendet, um idiomatischen und fehlerfreien Code zu fördern.6 Diese Werkzeuge tragen maßgeblich zur Codequalität und Lesbarkeit bei und erleichtern die Zusammenarbeit im Entwicklungsteam.

### C. Kernkonzepte der Domänenschicht

Die Domänenschicht wird um mehrere Kernkonzepte herum aufgebaut sein, die typisch für Domain-Driven Design (DDD) sind:

- **Entitäten (Entities):** Objekte, die nicht primär durch ihre Attribute, sondern durch eine eindeutige Identität und einen Lebenszyklus definiert sind. Änderungen an Entitäten werden über die Zeit verfolgt.
- **Wertobjekte (Value Objects):** Objekte, die einen beschreibenden Aspekt der Domäne darstellen und keine konzeptionelle Identität besitzen. Sie werden durch ihre Attribute definiert und sind typischerweise unveränderlich (immutable). Die Gleichheit von Wertobjekten basiert auf dem Vergleich ihrer Attribute.
- **Aggregate:** Eine Gruppe von Entitäten und Wertobjekten, die als eine einzelne Einheit für Datenänderungen behandelt werden. Jedes Aggregat hat eine Wurzelentität (Aggregate Root), die der einzige Einstiegspunkt für Modifikationen innerhalb des Aggregats ist und dessen Konsistenz sicherstellt.
- **Repositories (Schnittstellen):** Definieren Schnittstellen für den Zugriff auf und die Persistenz von Aggregaten. Sie abstrahieren die Details der Datenspeicherung und ermöglichen es der Domänenschicht, agnostisch gegenüber der konkreten Datenbanktechnologie zu bleiben.
- **Domänendienste (Domain Services):** Enthalten Domänenlogik, die nicht natürlich einer einzelnen Entität oder einem Wertobjekt zugeordnet werden kann. Solche Dienste operieren oft auf mehreren Domänenobjekten.
- **Domänenereignisse (Domain Events):** Repräsentieren signifikante Vorkommnisse innerhalb der Domäne, die für andere Teile des Systems von Interesse sein könnten.

Ein zentrales Element wird die Entwicklung einer **Ubiquitous Language** sein – einer gemeinsamen, präzisen Sprache, die von allen Teammitgliedern (Entwicklern und Domänenexperten) verwendet wird, um Konzepte der Domäne unmissverständlich zu beschreiben. Diese Sprache wird sich direkt im Code (Namen von Typen, Methoden, Modulen) widerspiegeln.

## III. Domänenmodell-Spezifikation

### A. Entitäten, Wertobjekte und Aggregate

Die korrekte Modellierung von Entitäten, Wertobjekten und Aggregaten ist von fundamentaler Bedeutung für die Fähigkeit der Anwendung, Geschäftskonzepte präzise abzubilden und zu manipulieren. Fehler in dieser frühen Phase können später zu erheblichem Refactoring-Aufwand führen.

**Entitäten** sind durch eine eindeutige Identität und einen Lebenszyklus gekennzeichnet. Ihre Attribute können sich im Laufe der Zeit ändern, aber ihre Identität bleibt bestehen. Ein Beispiel wäre ein `Benutzer` mit einer eindeutigen Benutzer-ID.

**Wertobjekte** hingegen haben keine eigene Identität über ihre Attribute hinaus. Sie beschreiben Merkmale und sind typischerweise unveränderlich. Ein `Geldbetrag` (bestehend aus Währung und Wert) oder eine `Adresse` (bestehend aus Straße, Stadt, Postleitzahl) sind klassische Beispiele. Würde eine Adresse fälschlicherweise als Entität modelliert, könnte dies zu unnötiger Komplexität bei der Identitätsverwaltung und bei Gleichheitsprüfungen führen, wo eigentlich nur ein struktureller Vergleich notwendig wäre.

**Aggregate** fassen Entitäten und Wertobjekte zu einer Konsistenzeinheit zusammen. Jedes Aggregat hat eine Wurzel (Aggregate Root), die als einziger Einstiegspunkt für Modifikationen dient und die Invarianten des Aggregats sicherstellt. Die Grenzen von Aggregaten müssen sorgfältig gewählt werden, um transaktionale Konsistenz zu gewährleisten und gleichzeitig eine übermäßige Kopplung zu vermeiden. Eine falsch gezogene Aggregatgrenze kann es erschweren, atomare Operationen konsistent durchzuführen oder führt zu unnötig großen Transaktionen.

Die folgende Tabelle fasst die initial identifizierten Kernentitäten zusammen:

**Tabelle 1: Kern-Domänenentitäten**

|   |   |   |   |
|---|---|---|---|
|**Entitätsname**|**Beschreibung**|**Schlüsselattribute**|**Identitätsmechanismus**|
|`BenutzerProfil`|Repräsentiert einen Benutzer des Systems.|`benutzer_id`, `email`|UUID (`benutzer_id`)|
|`Aufgabe`|Stellt eine zu erledigende Aufgabe dar.|`aufgaben_id`, `titel`|UUID (`aufgaben_id`)|
|`Projekt`|Gruppiert zusammengehörige Aufgaben.|`projekt_id`, `name`|UUID (`projekt_id`)|
|`Konfiguration`|Speichert benutzerspezifische Einstellungen.|`konfigurations_id`|UUID (`konfigurations_id`)|

Die Unterscheidung zwischen Entitäten und Wertobjekten ist oft subtil, aber entscheidend. Die nachstehende Tabelle listet wichtige Wertobjekte auf:

**Tabelle 2: Schlüssel-Wertobjekte**

|   |   |   |   |
|---|---|---|---|
|**Wertobjekt-Name**|**Beschreibung**|**Attribute**|**Unveränderlichkeits-Hinweise**|
|`EmailAdresse`|Repräsentiert eine E-Mail-Adresse.|`adresse` (String)|Ja, nach Erstellung fix.|
|`Zeitstempel`|Ein spezifischer Zeitpunkt.|`datum_uhrzeit` (z.B. `DateTime<Utc>`)|Ja, repräsentiert einen Punkt.|
|`Status`|Der Zustand einer Aufgabe oder eines Projekts.|`wert` (Enum: z.B. Offen, InArbeit)|Ja, Änderung erzeugt neuen Status.|
|`Prioritaet`|Die Dringlichkeit einer Aufgabe.|`stufe` (Enum: z.B. Niedrig, Hoch)|Ja.|

### B. Repository-Schnittstellen

Repository-Schnittstellen definieren die Verträge für den Zugriff auf und die Persistenz von Domänenobjekten, insbesondere Aggregaten. Sie werden als Rust-Traits implementiert und enthalten Methoden für typische CRUD-Operationen (Create, Read, Update, Delete) sowie ggf. spezifischere Abfragemethoden.

Ein fundamentaler Aspekt dieser Schnittstellen ist die Abstraktion von der konkreten Persistenztechnologie. Die Domänenschicht soll nicht wissen, ob Daten in einer SQL-Datenbank, einem NoSQL-Speicher oder einfachen Dateien abgelegt werden. Diese Unabhängigkeit wird erreicht, indem die Domänenschicht ausschließlich gegen die Repository-Traits programmiert wird. Die konkreten Implementierungen dieser Traits (z.B. ein `PostgresAufgabenRepository` oder ein `InMemoryBenutzerProfilRepository`) befinden sich außerhalb der Domänenschicht, typischerweise in der Infrastrukturschicht.

Diese Vorgehensweise bietet erhebliche Vorteile:

1. **Testbarkeit:** Für Unit-Tests der Domänenlogik können einfache In-Memory-Implementierungen der Repositories verwendet werden, was schnelle und isolierte Tests ohne externe Abhängigkeiten ermöglicht.
2. **Flexibilität:** Die Wahl der Datenbanktechnologie kann zu einem späteren Zeitpunkt geändert oder für verschiedene Umgebungen (z.B. Entwicklung vs. Produktion) unterschiedlich getroffen werden, ohne dass die Domänenschicht angepasst werden muss. Würde die Domänenschicht direkt Typen und Funktionen spezifischer Datenbank-Crates wie `sqlx` oder `diesel` verwenden, wäre sie fest an diese Technologie gebunden, was zukünftige Änderungen erschwert.

Beispielhafte Repository-Schnittstelle:

Rust

```
use std::error::Error;

// Platzhalter für Domänenfehler und Entität
type DomainError = Box<dyn Error + Send + Sync>;
struct Aufgabe { aufgaben_id: String, /*... */ }
struct Projekt { projekt_id: String, /*... */ }


pub trait AufgabenRepository {
    async fn finde_nach_id(&self, id: &str) -> Result<Option<Aufgabe>, DomainError>;
    async fn speichere(&self, aufgabe: &Aufgabe) -> Result<(), DomainError>;
    async fn loesche(&self, id: &str) -> Result<(), DomainError>;
    async fn finde_fuer_projekt(&self, projekt_id: &str) -> Result<Vec<Aufgabe>, DomainError>;
}

pub trait ProjektRepository {
    async fn finde_nach_id(&self, id: &str) -> Result<Option<Projekt>, DomainError>;
    async fn speichere(&self, projekt: &Projekt) -> Result<(), DomainError>;
}
```

### C. Serialisierung und Deserialisierung

Für die Umwandlung von Domänenobjekten in persistierbare Formate oder für die Übertragung über Prozessgrenzen hinweg (IPC) wird der `serde` Crate eingesetzt.7 `serde` ist der De-facto-Standard für Serialisierung und Deserialisierung im Rust-Ökosystem und bietet durch die Traits `Serialize` und `Deserialize` eine flexible und performante Lösung. Die aktuelle stabile Version (z.B. v1.0.219 gemäß 7) wird verwendet.

Die Wahl des konkreten Datenformats hängt vom Anwendungsfall ab:

- **JSON (JavaScript Object Notation):** Für menschenlesbare Konfigurationsdateien oder einfache IPC-Szenarien, bei denen Interoperabilität und Lesbarkeit im Vordergrund stehen.
- **CBOR (Concise Binary Object Representation) oder Bincode:** Für die effiziente binäre Persistenz großer Datenmengen oder performanzkritische IPC. Diese Formate sind kompakter und schneller zu verarbeiten als JSON, aber nicht direkt menschenlesbar.

Die Entscheidung für ein Format hat direkte Auswirkungen auf die Performance, die Speichergröße und die Debugfähigkeit. Beispielsweise wäre die Verwendung von JSON für große binäre Daten ineffizient, während ein benutzerdefiniertes Binärformat für Konfigurationsdateien deren manuelle Bearbeitung erschweren würde. `serde` stellt den Mechanismus bereit, die Wahl des Formats muss jedoch kontextspezifisch getroffen werden.

Bei der Verwendung von `serde` werden je nach Bedarf Features wie `std`, `derive`, `alloc` und `rc` aktiviert.8 Die `derive`-Makros (`#`) werden intensiv genutzt, um Boilerplate-Code für die Implementierung der Traits zu vermeiden.

## IV. Konfigurationsmanagement

### A. Struktur und Speicherort der Konfigurationsdateien

Die Konfiguration der Anwendung wird in Dateien im TOML-Format (Tom's Obvious, Minimal Language) gespeichert. TOML wurde aufgrund seiner guten Lesbarkeit und seiner Verbreitung im Rust-Ökosystem gewählt.9 Der `toml` Crate (z.B. Version 0.8.22 9) wird für das Parsen dieser Dateien verwendet. Die Konfigurationsdaten selbst werden in Rust-Strukturen abgebildet, die dann mittels `serde` deserialisiert werden.

Für die Ablage der Konfigurationsdateien wird die XDG Base Directory Specification 10 befolgt, um eine konsistente Benutzererfahrung auf Linux-Desktops zu gewährleisten. Der `directories` Crate (z.B. Version 6.0.0 11) wird verwendet, um die standardisierten Pfade systemunabhängig zu ermitteln:

- **Benutzerspezifische Konfiguration:** `$XDG_CONFIG_HOME/your_app_name/config.toml`. Falls `$XDG_CONFIG_HOME` nicht gesetzt ist, wird standardmäßig `$HOME/.config/your_app_name/config.toml` verwendet.
- **Systemweite Konfiguration (falls anwendbar):** `/etc/your_app_name/config.toml`.

Es wird eine klare Präzedenz festgelegt, wobei benutzerspezifische Konfigurationen systemweite Einstellungen überschreiben. Die Einhaltung der XDG-Standards ist von Bedeutung, da Benutzer erwarten, Konfigurationsdateien an vorhersagbaren Orten zu finden. Dies erleichtert die Verwaltung für Endanwender und Systemadministratoren und sorgt dafür, dass sich die Anwendung wie andere gut integrierte Linux-Anwendungen verhält.

Die Konfiguration kann bei Bedarf in logische Abschnitte innerhalb der `config.toml`-Datei unterteilt werden. Für den Anfang wird eine einzelne Konfigurationsdatei als ausreichend erachtet.

### B. Laden und Validieren der Konfiguration

Das Laden der Konfiguration erfolgt beim Anwendungsstart. Der Inhalt der TOML-Datei wird mithilfe des `toml` Crates und `serde` in die dafür vorgesehenen Rust-Strukturen deserialisiert.

Ein entscheidender Schritt ist die Validierung der geladenen Konfigurationswerte. Diese Validierung umfasst beispielsweise Bereichsprüfungen für numerische Werte, Formatprüfungen für Zeichenketten oder die Überprüfung der Existenz referenzierter Ressourcen. Hierfür können entweder benutzerdefinierte Validierungsfunktionen direkt in den Konfigurationsstrukturen oder, bei höherer Komplexität, ein dedizierter Validierungs-Crate implementiert werden.

Das Verhalten bei fehlgeschlagener Validierung wird klar definiert: Die Anwendung soll in einem solchen Fall mit einer aussagekräftigen Fehlermeldung beendet werden ("fail fast"). Dies ist vorzuziehen gegenüber einem Betrieb mit potenziell inkonsistenten oder fehlerhaften Einstellungen, der zu unvorhersehbarem Verhalten und schwer diagnostizierbaren Fehlern führen kann.

Standardwerte für Konfigurationsparameter werden über Implementierungen des `Default`-Traits für die Konfigurationsstrukturen oder durch explizite Fallback-Werte im Code bereitgestellt. `serde`-Attribute wie `#[serde(default)]` können hierbei nützlich sein, um das Verhalten bei fehlenden Feldern in der TOML-Datei zu steuern.

Die Frage der dynamischen Neuladung (Hot-Reloading) von Konfigurationsdateien während der Laufzeit wird als optionale, fortgeschrittene Anforderung betrachtet und ist, falls notwendig, eher Teil von Implementierungsplan Teil 2.

Die folgende Tabelle dient als Referenz für die konfigurierbaren Parameter der Domänenschicht:

**Tabelle 3: Konfigurationsparameter**

|   |   |   |   |   |
|---|---|---|---|---|
|**Parametername (Pfad in TOML)**|**Datentyp**|**Beschreibung**|**Standardwert**|**Validierungsregeln**|
|`logging.level`|String|Globaler Log-Level (z.B. "INFO", "DEBUG")|"INFO"|Muss gültiger Log-Level sein.|
|`database.url`|String|Verbindungs-URL zur Datenbank.|""|Muss valides URL-Format haben.|
|`secrets.collection_name`|String|Name der Secret-Service-Kollektion.|"default"|Darf nicht leer sein.|
|`performance.thread_pool_size`|u32|Größe des Thread-Pools für Hintergrundaufgaben.|4|Muss > 0 und < 1024 sein.|

## V. Fehlerbehandlung und Logging

### A. Fehlerarten und -hierarchie

Eine robuste Fehlerbehandlung ist essentiell für die Stabilität und Wartbarkeit der Domänenschicht. Es werden benutzerdefinierte Fehlertypen für die Domänenschicht definiert, wobei spezifische Fehler-Enums gegenüber generischen Fehler-Strings bevorzugt werden.

Der `thiserror` Crate (z.B. Version 2.0.12 12) wird zur Definition dieser Fehler-Enums verwendet. `thiserror` vereinfacht die Erstellung idiomatischer Fehlertypen erheblich:

- Das Attribut `#[error("...")]` wird genutzt, um menschenlesbare `Display`-Implementierungen für Fehler zu generieren.
- Mittels `#[from]` können zugrundeliegende Fehler (z.B. `std::io::Error`, `serde_json::Error`) automatisch in spezifische Varianten des Domänenfehlers konvertiert werden.
- Das Attribut `#[source]` ermöglicht die Verkettung von Fehlern, um den ursprünglichen Kontext für eine bessere Diagnose zu bewahren.

Bei der Definition der Fehler wird eine ausgewogene Granularität angestrebt.14 Fehler sollten spezifisch genug sein, um vom aufrufenden Code sinnvoll behandelt werden zu können, aber nicht so zahlreich, dass die Fehlerbehandlung unübersichtlich wird. Es ist denkbar, Fehler-Enums auf Modulebene zu definieren, die bei Bedarf in einem übergeordneten Domänenfehler-Enum aggregiert werden.

Ein grundlegendes Prinzip ist, dass die Domänenschicht als Bibliothek bei wiederherstellbaren Fehlern nicht paniken darf.15 Stattdessen wird `Result<T, E>` zurückgegeben. Panics sind ausschließlich für nicht wiederherstellbare Zustände reserviert, die auf einen Programmierfehler hindeuten (z.B. gebrochene Invarianten).

Gemäß den Rust API Guidelines 16 werden alle Fehlertypen die Traits `std::error::Error` und `std::fmt::Debug` implementieren (C-GOOD-ERR, C-DEBUG). Die Verwendung von `thiserror` stellt sicher, dass diese Implementierungen korrekt und mit minimalem Boilerplate-Code generiert werden, was die Ergonomie der Fehlerbehandlung sowohl für Entwickler der Domänenschicht als auch für deren Konsumenten signifikant verbessert.

Die folgende Tabelle listet beispielhaft Domänenfehlertypen auf:

**Tabelle 4: Domänenfehlertypen**

|   |   |   |   |
|---|---|---|---|
|**Fehler-Enum-Variante**|**Assoziierte Daten**|**Beschreibung (Wann tritt er auf)**|**#[source] (falls zutreffend)**|
|`KonfigurationNichtGefunden`|`Pfad: String`|Die Konfigurationsdatei konnte am erwarteten Ort nicht gefunden werden.|`std::io::Error`|
|`KonfigurationUngueltig`|`Fehler: String`|Die Konfigurationsdatei ist fehlerhaft oder enthält ungültige Werte.|`toml::de::Error`|
|`DatenbankFehler`|`Ursache: String`|Ein allgemeiner Fehler bei der Datenbankinteraktion.|Spezifischer DB-Fehler|
|`EntitaetNichtGefunden`|`Id: String, Typ: String`|Eine angeforderte Entität konnte nicht gefunden werden.|-|
|`GeheimnisdienstFehler`|`Meldung: String`|Fehler bei der Interaktion mit dem Secret Service.|`secret_service::Error`|

### B. Logging-Strategie und -Implementierung

Für ein strukturiertes und kontextbezogenes Logging wird das `tracing` Ökosystem eingesetzt, bestehend aus dem `tracing` Crate 17 und dem `tracing-subscriber` Crate.19 `tracing` bietet gegenüber dem einfacheren `log` Crate den Vorteil, reichhaltigere diagnostische Informationen zu liefern, insbesondere in asynchronen Kontexten oder komplexen Arbeitsabläufen.

Es wird zwischen `Events` (zeitpunktbezogene Ereignisse) und `Spans` (zeitraumbezogene Kontexte) unterschieden.17 Spans ermöglichen es, den Ausführungsfluss und die Kausalität von Ereignissen besser nachzuvollziehen. Das Makro `tracing::instrument` wird verwendet, um Funktionen einfach mit Spans zu versehen.

Standard-Log-Level (TRACE, DEBUG, INFO, WARN, ERROR) werden definiert und konsistent verwendet. Das Logging erfolgt strukturiert, d.h., Log-Ereignisse werden mit Schlüssel-Wert-Paaren versehen, um die maschinelle Verarbeitung und Analyse durch Log-Management-Tools zu erleichtern.

Die Konfiguration des Loggings erfolgt über `tracing-subscriber`. Die `fmt`-Schicht dient als gängiger Ausgangspunkt für die Formatierung der Log-Ausgaben (z.B. einfacher Text, JSON) und die Steuerung des Outputs (z.B. stdout, Datei). Der `env-filter` ermöglicht die Steuerung der Log-Level über eine Umgebungsvariable wie `RUST_LOG`.20 Diese flexible Konfiguration erlaubt es, die Ausführlichkeit der Logs ohne Codeänderungen an verschiedene Umgebungen (Entwicklung, Produktion) anzupassen.

Es wird sichergestellt, dass Logs relevante kontextuelle Informationen enthalten, wie z.B. Request-IDs, Benutzer-IDs (falls zutreffend) und Span-IDs, um die Fehlersuche zu unterstützen. Bei der Implementierung des Loggings werden Performance-Aspekte berücksichtigt. Obwohl `tracing` auf Effizienz ausgelegt ist, kann exzessives Logging, insbesondere auf hohen Verbositätsstufen, die Anwendungsleistung beeinträchtigen.

## VI. Integration mit Systemdiensten

### A. Geheimnisverwaltung (Secret Management)

Für die sichere Speicherung sensibler Daten wie API-Schlüssel oder Passwörter wird die Freedesktop Secret Service API genutzt. Die Integration erfolgt über den `secret-service-rs` Crate (z.B. Version 5.0.0 23). Dieser Crate ermöglicht die Interaktion mit Diensten wie GNOME Keyring oder KWallet.

Die Verwendung von `secret-service-rs` setzt eine D-Bus-Verbindung und eine asynchrone Laufzeitumgebung (Async Runtime) voraus. Die Wahl des spezifischen Runtime-Features des Crates (z.B. `rt-tokio-crypto-rust` oder `rt-async-io-crypto-openssl` 24) muss mit der übergreifenden asynchronen Strategie des Projekts abgestimmt sein. Die Kernoperationen umfassen das sichere Speichern, Abrufen und Löschen von Geheimnissen. Diese Geheimnisse werden in Kollektionen organisiert, wobei typischerweise die Standardkollektion verwendet wird. Ein typisches Nutzungsmuster beinhaltet das Verbinden zum `SecretService`, das Abrufen der Standardkollektion und dann das Erstellen, Suchen, Abrufen oder Löschen von Items, die ein Label, Attribute (für die Suche) und die eigentliche geheime Nutzlast besitzen.23

Fehler, die vom `secret-service` Crate zurückgegeben werden, müssen in domänenspezifische Fehlertypen der Anwendung verpackt werden, um eine konsistente Fehlerbehandlung zu gewährleisten. Die Abhängigkeit vom Freedesktop Secret Service bedeutet, dass die Anwendung auf Linux-Desktop-Umgebungen angewiesen ist, die diesen Dienst bereitstellen. Für die Portabilität auf Nicht-Linux-Plattformen oder in Headless-Umgebungen wären alternative Strategien zur Geheimnisverwaltung erforderlich, was potenziell plattformspezifische Abstraktionen nach sich zieht, falls plattformübergreifende Unterstützung ein langfristiges Ziel ist.

### B. Interprozesskommunikation (IPC) Grundlagen

D-Bus wird als fundamentales IPC-Mechanismus auf Linux-Desktops anerkannt 25, zumal die Geheimnisverwaltung über `secret-service-rs` bereits darauf basiert. Sollte die Domänenschicht direkt mit anderen D-Bus-Diensten interagieren müssen, die über die Abstraktionen von `secret-service-rs` hinausgehen, oder eine eigene D-Bus-Schnittstelle bereitstellen, käme der `dbus` Crate (z.B. Version 0.9.7 26) zum Einsatz. Dies würde ein Verständnis von D-Bus-Objekten, -Methoden, -Signalen und -Schnittstellen erfordern.

Die Nutzung von D-Bus für die Geheimnisverwaltung und potenziell andere IPC-Aufgaben führt eine asynchrone Abhängigkeit ein. Die Domänenschicht, oder zumindest die Teile, die mit D-Bus interagieren, müssen asynchron-fähig sein. Dies beeinflusst die Wahl der Rust Async Runtime (z.B. Tokio, async-std) für das gesamte Projekt, da D-Bus-Operationen inhärent asynchron sind und der `secret-service` Crate dies widerspiegelt.

Es ist entscheidend, eine robuste Fehlerbehandlung für Szenarien zu implementieren, in denen D-Bus-Dienste nicht verfügbar sind oder fehlerhaft reagieren. Die Domänenschicht kann nicht davon ausgehen, dass diese Dienste immer perfekt funktionieren, und muss Verbindungsfehler, Zeitüberschreitungen und dienstspezifische Fehler adäquat behandeln, beispielsweise durch das Deaktivieren abhängiger Funktionen oder durch informative Fehlermeldungen an den Benutzer.

Obwohl viele Wayland-Protokolle (z.B. `xdg-decoration` 27, `wlr-foreign-toplevel-management` 28) primär der UI- und Kompositor-Interaktion dienen und eine Form von IPC darstellen, wird die Domänenschicht diese nicht direkt implementieren. Sie könnte jedoch Daten bereitstellen oder auf Ereignisse reagieren, die von höheren Schichten orchestriert werden, welche diese Protokolle nutzen. Ähnlich verhält es sich mit XDG Desktop Portals 30, die sandboxed Anwendungen den Zugriff auf Systemressourcen über D-Bus ermöglichen; eine Interaktion hiermit würde ebenfalls auf einer höheren Ebene als der Domänenschicht stattfinden.

## VII. API-Design-Richtlinien und Best Practices

### A. Rust API Guidelines

Die offiziellen Rust API Guidelines 16 werden als verbindliche Grundlage für das Design aller öffentlichen Schnittstellen der Domänenschicht übernommen. Eine konsequente Einhaltung dieser Richtlinien ist ein starker Indikator für die langfristige Nutzbarkeit und Wartbarkeit einer Bibliothek, da sie die kognitive Last für Entwickler reduziert und Konsistenz fördert.

Besonderer Wert wird auf folgende Bereiche gelegt 16:

- **Namensgebung (C-CASE, C-CONV, C-GETTER, C-ITER, C-ITER-TY):** Einheitliche Verwendung von `snake_case` für Funktionen/Variablen und `PascalCase` für Typen. Standardisierte Namen für Konvertierungsmethoden (`as_`, `to_`, `into_`), Getter-Konventionen und Iterator-Benennungen.
- **Interoperabilität (C-COMMON-TRAITS, C-CONV-TRAITS, C-SEND-SYNC, C-GOOD-ERR):** Implementierung gängiger Traits wie `Debug`, `Clone`, `Eq`, `PartialEq`, `Default` sowie `Send` und `Sync`, wo angebracht. Verwendung von Standard-Konvertierungstraits (`From`, `AsRef`). Sicherstellung, dass Fehlertypen sich gut verhalten.
- **Dokumentation (C-CRATE-DOC, C-EXAMPLE, C-FAILURE, C-LINK, C-METADATA):** Umfassende Dokumentation auf Crate- und Item-Ebene, Beispiele für alle öffentlichen Elemente, Dokumentation von Fehlerbedingungen (Errors, Panics). Vollständige Metadaten in `Cargo.toml`.
- **Vorhersagbarkeit (C-METHOD, C-CTOR):** Funktionen mit einem klaren Empfänger (Receiver) werden als Methoden implementiert. Konstruktoren sind statische, inhärente Methoden (z.B. `new()`).
- **Flexibilität (C-GENERIC, C-CUSTOM-TYPE):** Einsatz von Generics, wo sinnvoll. Verwendung spezifischer Typen für Argumente anstelle von Booleans oder `Option`-Typen, um Bedeutung zu transportieren.
- **Verlässlichkeit (C-VALIDATE, C-DTOR-FAIL):** Funktionen validieren ihre Argumente. Destruktoren dürfen nicht fehlschlagen.
- **Debugfähigkeit (C-DEBUG, C-DEBUG-NONEMPTY):** Alle öffentlichen Typen implementieren `Debug`.
- **Zukunftssicherheit (C-STRUCT-PRIVATE, C-NEWTYPE-HIDE):** Strukturfelder sind standardmäßig privat, um interne Änderungen ohne API-Bruch zu ermöglichen. Newtypes kapseln Implementierungsdetails. Diese "Future Proofing"-Richtlinien sind besonders wichtig für eine Domänenschicht, da sich deren Kerndatenstrukturen weiterentwickeln könnten. Kapselung erlaubt solche Änderungen mit minimalen Auswirkungen auf abhängigen Code.

### B. Projektinterne Konventionen

Zusätzlich zu den offiziellen Rust API Guidelines werden folgende projektinterne Konventionen festgelegt:

- **Modulstruktur:** Eine standardisierte Modulstruktur für die Domänenschicht (z.B. `entities/`, `repositories/`, `services/`, `errors.rs`, `config.rs`).
- **Fehlerbehandlung:** Konsequente Anwendung der in Abschnitt V.A beschriebenen Muster unter Verwendung von `thiserror`.
- **Logging:** Konsequente Anwendung der in Abschnitt V.B beschriebenen Muster unter Verwendung von `tracing` für strukturiertes Logging.
- **Teststrategie:** Unit-Tests werden direkt neben dem zu testenden Code platziert. Integrationstests für Repositories verwenden Mocks oder In-Memory-Implementierungen.
- **Codeformatierung:** Die Verwendung von `rustfmt` mit den Standardeinstellungen ist obligatorisch.6
- **Clippy Lints:** Ein strenger Satz von Clippy-Lints wird durchgesetzt, um die Codequalität weiter zu erhöhen.
- **Sichere Programmierpraktiken 4:**
    - Die Verwendung von `unsafe`-Blöcken wird minimiert. Jeder Einsatz erfordert eine gründliche Überprüfung und stichhaltige Begründung.
    - Alle Daten, die Vertrauensgrenzen überschreiten, müssen validiert werden (obwohl die Domänenschicht idealerweise bereits validierte Daten von Anwendungsdiensten erhalten sollte).
    - Abhängigkeiten werden regelmäßig mit `cargo update` aktualisiert und mit Werkzeugen wie `cargo audit` auf bekannte Sicherheitslücken überprüft. Die Integration dieser Sicherheitspraktiken direkt in die Entwicklungsrichtlinien, anstatt sie als nachträglichen Gedanken zu behandeln, ist essentiell für den Aufbau eines vertrauenswürdigen Systems. Rusts Features (Ownership, Typsystem) bieten eine starke Grundlage, erfordern aber dennoch bewusste Anstrengungen.

### C. Versionsmanagement und Branching-Strategie

Für die Versionierung der Domänenschicht-Crate, insbesondere wenn sie unabhängig veröffentlicht oder versioniert wird, wird Semantic Versioning (SemVer) angewendet.

Als Git-Branching-Modell wird GitHub Flow empfohlen.33 GitHub Flow ist einfacher als GitFlow und eignet sich gut für kontinuierliche Integration und Auslieferung (CI/CD). Es basiert auf einem Haupt-Branch (z.B. `main`), von dem Feature-Branches abgeleitet werden. Nach Abschluss und Review werden diese Feature-Branches direkt zurück in den Haupt-Branch gemerged. Dies fördert schnelle Iterationen und eine stets auslieferungsbereite Codebasis.

## VIII. Anhänge

### A. Glossar

- **Aggregat (Aggregate):** Eine Gruppe von Entitäten und Wertobjekten, die als eine einzelne Einheit für Datenänderungen behandelt wird, mit einer Wurzelentität, die Konsistenz sicherstellt.
- **Clean Architecture:** Ein Softwarearchitekturmuster, das auf der Trennung von Belangen basiert und die Unabhängigkeit der Geschäftslogik von äußeren Schichten wie UI und Datenbank betont.
- **D-Bus:** Ein Interprozesskommunikationssystem, das auf Linux-Systemen weit verbreitet ist.
- **Domänenschicht (Domain Layer):** Der Teil einer Anwendung, der die Kernlogik und die Geschäftsregeln enthält.
- **Entität (Entity):** Ein Objekt in der Domäne, das durch eine eindeutige Identität und einen Lebenszyklus definiert ist.
- **Freedesktop Secret Service API:** Eine standardisierte Schnittstelle unter Linux zur sicheren Speicherung von Geheimnissen.
- **IPC (Inter-Process Communication):** Kommunikation zwischen verschiedenen Prozessen.
- **Repository:** Eine Abstraktion, die den Zugriff auf und die Persistenz von Domänenobjekten kapselt.
- **Rust:** Eine Systemprogrammiersprache, die auf Sicherheit und Performance ausgelegt ist.
- **serde:** Ein populärer Rust-Crate für Serialisierung und Deserialisierung.
- **SemVer (Semantic Versioning):** Ein Standard für die Versionierung von Software.
- **thiserror:** Ein Rust-Crate zur einfachen Erstellung von Fehler-Enums.
- **TOML (Tom's Obvious, Minimal Language):** Ein Konfigurationsdateiformat.
- **tracing:** Ein Rust-Framework für instrumentiertes, strukturiertes Logging.
- **Ubiquitous Language:** Eine gemeinsame Sprache, die von Entwicklern und Domänenexperten verwendet wird, um Domänenkonzepte präzise zu beschreiben.
- **Wertobjekt (Value Object):** Ein Objekt, das einen beschreibenden Aspekt der Domäne darstellt und keine Identität über seine Attribute hinaus besitzt; typischerweise unveränderlich.
- **XDG Base Directory Specification:** Ein Standard von freedesktop.org, der festlegt, wo benutzerspezifische Daten- und Konfigurationsdateien gespeichert werden sollen.

### B. Referenzierte Crates und Versionen

Die Entwicklung der Domänenschicht (Teil 1) wird auf der stabilen Rust-Version 1.85.0 (oder neuer, falls zum Entwicklungsstart verfügbar) und der Rust 2024 Edition basieren.35 Die Wahl einer aktuellen Edition ermöglicht die Nutzung der neuesten Sprachfeatures und Idiome. Die Pflege einer Liste der referenzierten Crates und ihrer Versionen ist entscheidend für die Reproduzierbarkeit von Builds und das Management von Abhängigkeitsupdates.

**Tabelle 7: Externe Crate-Abhängigkeiten (Teil 1)**

|   |   |   |   |   |
|---|---|---|---|---|
|**Crate-Name**|**Version (Beispiel)**|**Lizenz**|**Hauptzweck in der Domänenschicht**|**Genutzte Schlüsselfunktionen**|
|`serde`|1.0.219|MIT/Apache-2.0|Serialisierung und Deserialisierung von Datenstrukturen.|`Serialize`, `Deserialize` Traits, `derive` Makros|
|`thiserror`|2.0.12|MIT/Apache-2.0|Ergonomische Definition von benutzerdefinierten Fehlertypen.|`#[derive(Error)]`, `#[error(...)]`, `#[from]`, `#[source]`|
|`tracing`|0.1.x|MIT|Strukturiertes, kontextbezogenes Logging und Tracing.|`span!`, `event!`, `#[instrument]`|
|`tracing-subscriber`|0.3.x|MIT|Konfiguration von Logging-Ausgabe und -Filterung.|`fmt::Layer`, `EnvFilter`|
|`toml`|0.8.22|MIT/Apache-2.0|Parsen und Serialisieren von Konfigurationsdateien im TOML-Format.|`from_str`, `to_string` (via `serde`)|
|`directories`|6.0.0|MIT/Apache-2.0|Ermittlung von Standardverzeichnispfaden (XDG).|`ProjectDirs`, `UserDirs`, `BaseDirs`|
|`secret-service`|5.0.0|MIT/Apache-2.0|Sichere Speicherung und Abruf von Geheimnissen via D-Bus.|`SecretService::connect`, Collection- und Item-Operationen|
|`dbus` (optional)|0.9.7|MIT/Apache-2.0|Direkte D-Bus Interprozesskommunikation (falls benötigt).|Verbindungshandling, Methodenaufrufe, Signalempfang|

## IX. Schlussfolgerungen

Der vorliegende Implementierungsplan für Teil 1 der Domänenschicht legt eine solide Basis für die Entwicklung einer robusten und wartbaren Anwendung. Die Wahl von Rust als Programmiersprache, kombiniert mit einer klaren Architektur nach den Prinzipien der Clean Architecture, verspricht eine hohe Codequalität und Performance. Die konsequente Nutzung etablierter Crates wie `serde` für die Datenverarbeitung, `thiserror` für eine präzise Fehlerbehandlung und `tracing` für ein aufschlussreiches Logging wird die Entwicklungseffizienz steigern und die Diagnosefähigkeit des Systems verbessern.

Die Standardisierung des Konfigurationsmanagements mittels TOML und der XDG Base Directory Specification sowie die Integration mit dem Freedesktop Secret Service für die Geheimnisverwaltung gewährleisten eine gute Einbettung in Linux-Desktop-Umgebungen. Die definierten API-Designrichtlinien und projektinternen Konventionen, einschließlich der Betonung sicherer Programmierpraktiken, werden zur langfristigen Stabilität und Sicherheit der Domänenschicht beitragen.

Durch die klare Abgrenzung der Verantwortlichkeiten und die Schaffung testbarer Komponenten wird eine hohe Softwarequalität angestrebt. Die in diesem Plan getroffenen Entscheidungen zielen darauf ab, eine Domänenschicht zu schaffen, die nicht nur die aktuellen Anforderungen erfüllt, sondern auch flexibel genug ist, um zukünftige Erweiterungen und Änderungen aufzunehmen. Die sorgfältige Modellierung der Domänenkonzepte und die Abstraktion von externen Abhängigkeiten sind hierbei Schlüsselfaktoren für den Erfolg.

# Domänenschicht Implementierungsplan (Ultra-Feinspezifikation)

## 1. Grundlagen und Architektur der Domänenschicht

Die Domänenschicht bildet das Herzstück der Desktop-Umgebung und beinhaltet die Kernlogik sowie die Geschäftsregeln. Ihre Hauptverantwortung liegt in der Verwaltung von Workspaces ("Spaces"), dem Theming-System, der Logik für KI-Interaktionen einschließlich des Einwilligungsmanagements, der Verwaltung von Benachrichtigungen und der Definition von Richtlinien für das Fenstermanagement (z.B. Tiling-Regeln). Ein fundamentaler Aspekt dieser Schicht ist ihre Unabhängigkeit von spezifischen UI-Implementierungen oder Systemdetails wie D-Bus oder Wayland. Sie nutzt Funktionalitäten der Kernschicht und stellt Logik sowie Zustand für die System- und Benutzeroberflächenschicht bereit.

### 1.1. Programmiersprache und Entwicklungsumgebung

- **Sprache:** Rust (Version 1.85.0 oder neuer, Stand Februar 2025 1). Die Wahl von Rust begründet sich durch dessen Fokus auf Speichersicherheit ohne Garbage Collector, exzellente Performance und moderne Concurrency-Features, was für ein System wie eine Desktop-Umgebung von entscheidender Bedeutung ist.2
- **Build-System:** Cargo, das Standard-Build-System und Paketmanager für Rust.5 Meson wird zwar als fähig erachtet, Rust-Projekte zu handhaben 6, jedoch ist Cargo die natürliche Wahl im Rust-Ökosystem.
- **Entwicklungsrichtlinien:**
    - Rust API Guidelines Checklist 7: Strikte Einhaltung dieser Richtlinien für Namenskonventionen (C-CASE, C-CONV, C-GETTER, C-ITER), Interoperabilität (C-COMMON-TRAITS, C-SEND-SYNC), Dokumentation (C-CRATE-DOC, C-EXAMPLE) und weitere Aspekte.
    - Rust Style Guide 8: Formatierungskonventionen (Einrückung, Zeilenlänge), Kommentierungsstil.
    - Secure Coding Practices in Rust 9: Minimierung von `unsafe` Blöcken, sorgfältige Prüfung von Abhängigkeiten, Validierung von Eingabedaten (obwohl die Domänenschicht primär interne Daten verarbeitet, ist das Prinzip wichtig).
    - Git Branching Modell: GitHub Flow wird für seine Einfachheit und den Fokus auf Continuous Delivery bevorzugt, besonders wenn schnelle Iterationen und häufige Releases angestrebt werden.10 Feature-Branches werden von `main` erstellt und nach Fertigstellung und Review direkt zurück in `main` gemerged. Für stabilere Release-Zyklen könnte GitFlow in Betracht gezogen werden, aber für die iterative Entwicklung einer neuen Desktop-Umgebung erscheint GitHub Flow agiler.

Die Entscheidung für Rust basiert auf dessen Fähigkeit, systemnahe Software zu entwickeln, die sowohl sicher als auch performant ist. Die strengen Compiler-Prüfungen von Rust helfen, viele gängige Fehlerklassen bereits zur Compile-Zeit zu eliminieren, was die Stabilität der Domänenschicht maßgeblich erhöht. Die Einhaltung etablierter Entwicklungsrichtlinien stellt sicher, dass der Code verständlich, wartbar und konsistent bleibt.

### 1.2. Kernabhängigkeiten und Basistechnologien

Die Domänenschicht wird so entworfen, dass sie minimale direkte Abhängigkeiten zu externen Systembibliotheken hat. Notwendige Interaktionen mit dem System (Dateizugriff, Prozessmanagement, etc.) erfolgen über Abstraktionen (Ports), die von der Kernschicht implementiert werden.

- **1.2.1. Externe Rust-Crates**
    - Eine detaillierte Liste der verwendeten Crates mit Versionen und Features findet sich in Anhang C.
    - Die Auswahl der Crates folgt dem Prinzip der Stabilität, Verbreitung und Wartungsfreundlichkeit.

|   |   |   |   |
|---|---|---|---|
|**Crate-Name**|**Version (Beispielhaft)**|**Kurzbeschreibung/Zweck**|**Relevante Features (Beispielhaft)**|
|`serde`|`1.0.219` 12|Serialisierung und Deserialisierung von Datenstrukturen.|`derive`|
|`thiserror`|`2.0.12` 14|Einfache Erstellung von benutzerdefinierten Fehlertypen.||
|`tracing`|`0.1.40` 15|Framework für anwendungsspezifische Diagnoseaufzeichnungen (Logging und Tracing).||
|`tracing-subscriber`|`0.3.18` 15|Implementierungen für `tracing` Subscriber (z.B. für formatiertes Logging).|`env-filter`, `fmt`|
|`toml`|`0.8.22` 18|Parsen von TOML-Konfigurationsdateien (indirekt, da Domäne geparste Daten erhält).||
|`uuid`|`1.8.0` (optional)|Generierung und Handhabung von UUIDs für eindeutige IDs.|`v4`, `serde`|
|`log`|`0.4.x`|Logging-Fassade, falls `tracing-log` verwendet wird.||
|`directories-next`|`2.0.0` (oder `directories` v6.0.0 19)|Auflösung von XDG-Standardverzeichnissen (indirekt, durch Kernschicht genutzt).||

- **1.2.2. Fehlerbehandlungsstrategie**
    
    - Verwendung des `thiserror` Crates (Version 2.0.12 14) zur Definition von benutzerdefinierten Fehlertypen.
    - Jedes Modul der Domänenschicht definiert sein eigenes spezifisches Error-Enum (z.B. `WorkspaceError`, `ThemingError`).
    - Ein globales `DomainError` Enum fasst alle modulspezifischen Fehler zusammen (siehe Abschnitt 7).
    - Die `#[from]` Annotation wird genutzt, um Fehler aus abhängigen Operationen (z.B. IO-Fehler aus der Kernschicht, die durchgereicht werden) elegant in domänenspezifische Fehler umzuwandeln.20 Dies vermeidet generische Fehlertypen und macht den Code für Aufrufer verständlicher und besser handhabbar. Die klare Strukturierung von Fehlern, beginnend bei spezifischen Fehlern pro Operation oder Modul und aggregiert in einem übergeordneten `DomainError`, erleichtert sowohl die Fehlerbehandlung innerhalb der Domänenschicht als auch die Kommunikation von Fehlern an höhere Schichten.21
- **1.2.3. Logging- und Tracing-Strategie**
    
    - Verwendung des `tracing` Crates (Core: `tracing-core` Version ~0.1.31+ 16, Subscriber: `tracing-subscriber` Version ~0.3.17+ 15).
    - Strukturierte Logs mit Span-basiertem Tracing zur Nachverfolgung von Abläufen über Modulgrenzen hinweg. Die `tracing` Bibliothek ist dem `log` Crate vorzuziehen, da sie durch Spans zusätzlichen Kontext für Diagnoseinformationen bereitstellt, was besonders bei der Analyse komplexer Abläufe in der Domänenschicht vorteilhaft ist.22
    - Konfigurierbare Loglevel (TRACE, DEBUG, INFO, WARN, ERROR).
    - Die Domänenschicht emittiert Traces und Logs; die Konfiguration des Subscribers (z.B. Format, Output) obliegt der Anwendungsschicht oder dem Hauptprogramm.
    - Die Notwendigkeit eines "lückenlosen Entwickler-Implementierungsleitfadens" schließt die Beobachtbarkeit der Software im Betrieb ein. `tracing` ermöglicht die detaillierte Erfassung des Kontrollflusses und wichtiger Zustandsänderungen. Durch die Verwendung von Spans (z.B. `span!(Level::INFO, "operation_xyz");`) können Operationen, die mehrere Schritte umfassen, logisch gruppiert werden, was die Analyse von Log-Daten erheblich vereinfacht.
- **1.2.4. Konfigurationsmanagement**
    
    - Konfigurationsdaten für Domänendienste (z.B. Standard-Theme, Standard-Workspace-Layout) werden der Domänenschicht von außen (typischerweise von der Anwendungsschicht beim Start, geladen durch die Kernschicht) übergeben.
    - Die Domänenschicht definiert Strukturen für ihre Konfigurationsparameter. Diese Strukturen sollen `serde::Deserialize` implementieren.
    - Das bevorzugte Format für Konfigurationsdateien ist TOML (Crate: `toml` Version 0.8.22+ 5).
    - Pfade zu Konfigurationsdateien werden gemäß XDG Base Directory Specification 19 von der Kernschicht aufgelöst (z.B. `$XDG_CONFIG_HOME/your_app_name/domain_settings.toml`). Die `directories` Crate (Version 6.0.0 19 oder `directories-next` 2.0.0) kann hierfür von der Kernschicht genutzt werden.
    - Die Domänenschicht selbst führt keine Dateisystemoperationen durch, um ihre Unabhängigkeit und Testbarkeit zu wahren. Sie erhält Konfigurationen als bereits geparste Datenstrukturen. Diese Trennung der Verantwortlichkeiten ist entscheidend, da Dateisystemzugriffe als Systemdetails gelten, von denen die Domänenschicht abstrahiert sein soll. `serde` und `toml` sind etablierte Standards im Rust-Ökosystem für diese Aufgabe.
- **1.2.5. Serialisierung/Deserialisierung**
    
    - Verwendung des `serde` Crates (Version 1.0.219+ 12) für die Serialisierung und Deserialisierung von Datenstrukturen, die persistiert oder über Schichtgrenzen hinweg ausgetauscht werden müssen.
    - Alle relevanten Entitäten und Wertobjekte, die persistiert oder als Teil von Events/Signalen übertragen werden, müssen `serde::Serialize` und `serde::Deserialize` implementieren.
- **1.2.6. Asynchrone Operationen**
    
    - Die Domänenschicht wird primär synchron entworfen, um die Komplexität niedrig zu halten. Langlaufende Operationen oder Interaktionen mit I/O-bound Systemen (z.B. komplexe KI-Anfragen, die über die Kernschicht laufen) können jedoch asynchrone Schnittstellen erfordern.
    - Wo Asynchronität notwendig ist, wird `async/await` mit einer durch die Kerninfrastruktur vorgegebenen Runtime (z.B. Tokio) verwendet. Die Domänenschicht selbst startet keine eigenen Runtimes.
    - Kommunikation zwischen synchronen und asynchronen Teilen erfolgt über klar definierte Kanäle (z.B. `tokio::sync::mpsc` oder `async_channel` 25), die von der Kernschicht oder der Anwendungsschicht bereitgestellt werden.
    - Die primär synchrone Natur der Domänenschicht vereinfacht das Design und die Testbarkeit erheblich. Asynchronität wird nur dort eingeführt, wo sie unumgänglich ist, und die Verwaltung des Runtimes wird an die Kernschicht delegiert. Komplexe Geschäftslogik ist oft einfacher synchron zu verstehen und zu implementieren. Würde die Domänenschicht selbst durchgängig asynchron sein, müsste sie sich um Executor, Task-Spawning etc. kümmern, was ihre Komplexität erhöht und sie stärker an eine spezifische async-Runtime bindet. Indem sie synchrone Schnittstellen anbietet und für langlaufende Operationen auf von der Kernschicht bereitgestellte `Future`s oder asynchrone Funktionen zurückgreift, bleibt sie fokussierter.

### 1.3. Interaktionsmuster mit der Kernschicht und anderen Schichten

- **Mit der Kernschicht:**
    - Die Domänenschicht definiert Traits (abstrakte Schnittstellen, sogenannte Ports), die von der Kernschicht implementiert werden müssen (Adapter), um Zugriff auf systemnahe Funktionen zu erhalten. Beispiele hierfür sind `PersistencePort`, `SystemClockPort`, `SecureStoragePort`.
    - Beispiel: Das `ThemingService` (siehe Abschnitt 3) könnte ein `ThemePersistencePort` Trait definieren, um Themes zu laden und zu speichern. Die Kernschicht würde dieses Trait implementieren und dabei z.B. auf das Dateisystem zugreifen.
- **Mit der System- und UI-Schicht:**
    - Die Domänenschicht stellt konkrete Services mit wohldefinierten Methoden bereit.
    - Die Kommunikation von Zustandsänderungen und Ereignissen aus der Domänenschicht an höhere Schichten erfolgt über ein Event/Signal-System (siehe Abschnitt 1.4.2).
    - Die Verwendung von Ports und Adapters (ein Muster der Hexagonalen Architektur) für die Interaktion mit der Kernschicht stellt sicher, dass die Domänenschicht vollständig von den Implementierungsdetails der Kernschicht entkoppelt ist. Die Domänenschicht "nutzt Funktionalität der Kernschicht". Um die Unabhängigkeit zu wahren, darf die Domänenschicht die Kernschicht nicht direkt aufrufen oder deren konkrete Typen kennen. Stattdessen definiert die Domänenschicht, _was_ sie benötigt (z.B. "speichere dieses Objekt"), und die Kernschicht liefert die Implementierung dafür. Dies ist ein Kernprinzip der Inversion of Control.

### 1.4. Allgemeine Datentypen, Traits und Hilfsfunktionen der Domänenschicht

- **1.4.1. Basis-Identifikatoren**
    
    - Typalias `DomainId`: Vorerst wird `String` für Flexibilität und einfache Serialisierung gewählt.
        
        Rust
        
        ```
        pub type DomainId = String;
        ```
        
        Alternativ könnte `uuid::Uuid` verwendet werden, falls global eindeutige IDs über Systemgrenzen hinweg erforderlich sind und dies in der Kerninfrastruktur-Spezifikation festgelegt wurde. Für rein interne Zwecke könnten auch Newtype-Strukturen um `usize` in Betracht gezogen werden. Da die Domänenschicht UI- und systemunabhängig ist, sind einfache, serialisierbare IDs oft ausreichend.
- **1.4.2. Event-System Abstraktion**
    
    - Ein generischer `DomainEvent` Enum kapselt alle Domänenereignisse. Jedes Modul definiert seine eigenen spezifischen Event-Typen, die als Varianten in `DomainEvent` aufgenommen werden.
        
        Rust
        
        ```
        #
        pub enum DomainEvent {
            Workspace(crate::workspace_manager::WorkspaceEvent),
            Theming(crate::theming_manager::ThemingEvent),
            AIConsent(crate::ai_manager::consent_manager::AIConsentEvent),
            AIFeature(crate::ai_manager::feature_service::AIFeatureEvent),
            Notification(crate::notification_manager::NotificationEvent),
            WindowPolicy(crate::window_policy_engine::WindowPolicyEvent),
            // Weitere Event-Kategorien können hier hinzugefügt werden.
        }
        ```
        
    - **Beispiel für eine spezifische Event-Kategorie (WorkspaceEvent):**
        
        Rust
        
        ```
        // Definiert in workspace_manager/events.rs oder workspace_manager/mod.rs
        #
        pub enum WorkspaceEvent {
            SpaceCreated {
                space_id: DomainId,
                name: String,
                layout_type: crate::workspace_manager::LayoutType, // Vollständiger Pfad zum Typ
                // Weitere relevante Felder
            },
            SpaceDeleted { space_id: DomainId },
            SpaceRenamed { space_id: DomainId, new_name: String },
            // Weitere Workspace-spezifische Events
        }
        ```
        
        Ähnliche Enums (`ThemingEvent`, `AIConsentEvent`, etc.) werden in den jeweiligen Modulen definiert.
    - **Publisher:** Typischerweise die Services innerhalb der Domänenschicht (z.B. `SpaceService`).
    - **Subscriber:** Andere Services innerhalb der Domänenschicht, die auf bestimmte Ereignisse reagieren müssen, oder die System-/UI-Schicht, die über Änderungen informiert werden wollen. Die konkrete Event-Bus-Implementierung wird von der Kerninfrastruktur bereitgestellt; die Domänenschicht definiert nur die Events und identifiziert typische Sender und Empfänger.
    - Ein klar definiertes, typisiertes Event-System ist fundamental für eine entkoppelte Architektur. Die Verwendung von `serde` für Events ermöglicht deren einfache Serialisierung, falls sie z.B. über Prozessgrenzen hinweg gesendet oder persistiert werden müssten. Die Anforderung, "Identifikation der typischen Publisher und Subscriber für jedes Event" und "Eindeutiger Event-Name/Typ", wird durch dieses strukturierte Event-System erfüllt. Ein übergreifender `DomainEvent` Enum mit untergeordneten Enums pro Modul schafft eine klare Hierarchie und ermöglicht es Subscribern, sich gezielt für Event-Kategorien oder spezifische Events zu registrieren.
- **1.4.3. Standardisierte Rückgabetypen**
    
    - Verwendung von `Result<T, DomainError>` für alle öffentlichen Operationen der Domänenschicht, die fehlschlagen können.
        
        Rust
        
        ```
        #
        pub enum DomainError {
            #[error("Workspace error: {0}")]
            Workspace(#[from] crate::workspace_manager::WorkspaceError),
            #
            Theming(#[from] crate::theming_manager::ThemingError),
            #[error("AI consent error: {0}")]
            AIConsent(#[from] crate::ai_manager::consent_manager::AIConsentError),
            #[error("AI feature error: {0}")]
            AIFeature(#[from] crate::ai_manager::feature_service::AIFeatureError),
            #[error("Notification error: {0}")]
            Notification(#[from] crate::notification_manager::NotificationError),
            #[error("Window policy error: {0}")]
            WindowPolicy(#[from] crate::window_policy_engine::WindowPolicyError),
        
            #[error("Persistence error: {0}")]
            Persistence(String), // Fehler von der Persistenzschicht (Kernschicht)
            #[error("Configuration error: {0}")]
            Configuration(String), // Fehler beim Verarbeiten von Konfigurationen
            #[error("Invariant violation: {0}")]
            InvariantViolation(String), // Wenn eine Geschäftsregel verletzt wurde
            #[error("Unauthorized operation: {0}")]
            Unauthorized(String),
            #
            NotFound { resource_type: String, resource_id: String },
            #[error("Invalid input: {message}")]
            InvalidInput { message: String },
            #[error("An unexpected internal error occurred: {0}")]
            Internal(String), // Für nicht spezifisch behandelte Fehler
        }
        ```
        
    - Die Definition eines übergreifenden `DomainError` Enums, der spezifischere Fehler aus den einzelnen Modulen aggregiert (mittels `#[from]`), bietet eine konsistente Fehlerbehandlungsschnittstelle für die aufrufenden Schichten. Wenn jeder Service seinen eigenen, nicht verwandten Fehlertyp zurückgibt, wird die Fehlerbehandlung in der aufrufenden Schicht komplex. Ein gemeinsamer `DomainError` mit Varianten für jeden Modulfehler (z.B. `DomainError::Workspace(WorkspaceError::SpaceNotFound)`) ermöglicht es dem Aufrufer, entweder generisch auf `DomainError` zu reagieren oder spezifisch auf `WorkspaceError` oder sogar `SpaceNotFound` zu matchen. `thiserror` erleichtert diese Struktur erheblich.

---

## 2. Modul: Workspace-Management (`workspace_manager`)

### 2.1. Übersicht und Verantwortlichkeiten

Das Modul `workspace_manager` ist für die Verwaltung von "Spaces" – virtuellen Desktops oder Arbeitsbereichen – zuständig. Es kümmert sich um die Zuordnung von Fenstern, die durch abstrakte `WindowHandle`-Identifikatoren repräsentiert werden, zu diesen Spaces. Des Weiteren verwaltet es die Layout-Konfiguration für jeden Space, beispielsweise ob Fenster gekachelt (Tiling) oder frei beweglich (Floating) angeordnet werden. Das Modul stellt Informationen über den aktuellen Zustand der Spaces und der darin enthaltenen Fenster bereit. Eine wichtige Interaktion besteht mit dem `WindowPolicyEngine` (siehe Abschnitt 6), um Standardverhalten oder spezifische Regeln bei Fensteroperationen oder Wechseln zwischen Spaces anzuwenden.

Die zentrale Rolle dieses Moduls für die Organisation der Arbeitsumgebung des Benutzers erfordert eine effiziente und klare Zustandsverwaltung, die maßgeblich zur User Experience beiträgt. Die Abstraktion von "Fenstern" als `WindowHandle` ist hierbei kritisch, um die Unabhängigkeit von spezifischen Fenstersystemen wie Wayland oder X11 zu gewährleisten. Die Domänenschicht darf keine Wayland- oder X11-spezifischen Fenster-IDs direkt kennen. Ein `WindowHandle` (z.B. eine `String` oder `uuid::Uuid`) dient als stabiler, systemunabhängiger Identifikator. Die Systemschicht ist dafür verantwortlich, die Übersetzung zwischen diesem `WindowHandle` und den tatsächlichen Fenster-IDs des jeweiligen Windowing-Systems vorzunehmen.

### 2.2. Entitäten, Wertobjekte und Enums

- **2.2.1. Entität: `Space`**
    
    - Repräsentiert einen einzelnen Workspace.
    - **Attribute:**
        - `id: DomainId` (Eindeutige ID des Space, z.B. generiert via `uuid::Uuid::new_v4().to_string()`). Sichtbarkeit: `pub(crate)`. Initialwert: Generiert bei Erstellung. Invarianten: Eindeutig, unveränderlich nach Erstellung.
        - `name: String` (Benutzerdefinierter Name, z.B. "Arbeit", "Freizeit"). Sichtbarkeit: `pub`. Initialwert: Bei Erstellung übergeben. Invarianten: Nicht leer.
        - `layout_type: LayoutType` (Aktueller Layout-Modus des Space). Sichtbarkeit: `pub`. Initialwert: Bei Erstellung übergeben, Default `LayoutType::Floating`.
        - `windows: std::collections::VecDeque<WindowHandle>` (Geordnete Liste der Fenster-Handles in diesem Space; `VecDeque` für effizientes Hinzufügen/Entfernen an beiden Enden und Beibehaltung der Reihenfolge, was für Stacking-Order oder Tiling-Reihenfolge relevant sein kann). Sichtbarkeit: `pub(crate)`. Initialwert: Leer.
        - `tiling_config: Option<TilingConfiguration>` (Spezifische Konfiguration, wenn `layout_type == LayoutType::Tiling`). Sichtbarkeit: `pub`. Initialwert: `None`. Invarianten: `Some` gdw. `layout_type` eine Tiling-Variante ist.
        - `creation_timestamp: u64` (Unix-Timestamp in Millisekunden der Erstellung). Sichtbarkeit: `pub(crate)`. Initialwert: Zeitstempel bei Erstellung.
        - `last_accessed_timestamp: u64` (Unix-Timestamp in Millisekunden des letzten Zugriffs/Aktivierung). Sichtbarkeit: `pub(crate)`. Initialwert: Zeitstempel bei Erstellung.
    - **Methoden (interne Logik der `Space`-Entität):**
        - `pub(crate) fn new(id: DomainId, name: String, layout_type: LayoutType, tiling_config: Option<TilingConfiguration>, current_timestamp: u64) -> Self`
            - Vorbedingungen: `id` und `name` nicht leer. Wenn `layout_type` eine Tiling-Variante ist, muss `tiling_config` `Some` und valide sein.
            - Nachbedingungen: Ein neues `Space`-Objekt wird mit den initialen Werten erstellt.
        - `pub(crate) fn add_window(&mut self, window_handle: WindowHandle) -> Result<(), WorkspaceError>`
            - Fügt ein Fenster am Ende der `windows`-Liste hinzu.
            - Vorbedingungen: Fenster ist nicht bereits im Space.
            - Nachbedingungen: Fenster ist im Space enthalten.
            - Geschäftsregel: Verhindert Duplikate.
        - `pub(crate) fn remove_window(&mut self, window_handle: &WindowHandle) -> Result<(), WorkspaceError>`
            - Entfernt ein Fenster aus dem Space.
            - Vorbedingungen: Fenster ist im Space enthalten.
            - Nachbedingungen: Fenster ist nicht mehr im Space.
            - Geschäftsregel: Gibt Fehler zurück, wenn Fenster nicht gefunden wird.
        - `pub(crate) fn set_layout(&mut self, layout_type: LayoutType, config: Option<TilingConfiguration>) -> Result<(), WorkspaceError>`
            - Aktualisiert `layout_type` und `tiling_config`.
            - Vorbedingungen: Wenn `layout_type` eine Tiling-Variante ist, muss `config` `Some` und valide sein.
            - Nachbedingungen: Layout-Informationen sind aktualisiert.
            - Geschäftsregel: Validiert die `config` für Tiling-Layouts.
        - `pub(crate) fn is_empty(&self) -> bool`
            - Gibt `true` zurück, wenn keine Fenster im Space sind.
        - `pub(crate) fn contains_window(&self, window_handle: &WindowHandle) -> bool`
            - Prüft, ob das Fenster im Space enthalten ist.
        - `pub(crate) fn update_last_accessed(&mut self, current_timestamp: u64)`
            - Aktualisiert `last_accessed_timestamp`.
    - **Beziehungen:** Enthält eine Sammlung von `WindowHandle`s.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub struct Space {
            pub(crate) id: DomainId,
            pub name: String,
            pub layout_type: LayoutType,
            pub(crate) windows: std::collections::VecDeque<WindowHandle>,
            pub tiling_config: Option<TilingConfiguration>,
            pub(crate) creation_timestamp: u64,
            pub(crate) last_accessed_timestamp: u64,
        }
        ```
        
- **2.2.2. Wertobjekt: `WindowHandle`**
    
    - Eine reine ID-Abstraktion für ein Anwendungsfenster. Die Domänenschicht besitzt keine Kenntnisse über Größe, Position oder Inhalt des Fensters; diese Details werden von der UI- oder Systemschicht verwaltet.
    - **Attribute:**
        - `id: DomainId` (Eindeutiger, persistenter Identifikator). Sichtbarkeit: `pub`.
    - **Invarianten:** `id` ist nicht leer und eindeutig im Kontext aller verwalteten Fenster.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub struct WindowHandle {
            pub id: DomainId,
        }
        ```
        
- **2.2.3. Enum: `LayoutType`**
    
    - Definiert die möglichen Layout-Modi für einen Space.
    - **Varianten:** `Tiling`, `Floating`, `Monocle` (Vollbild für ein einzelnes Fenster im Space), `Custom(String)` (für erweiterbare, benutzerdefinierte Layouts).
    - **Implementiert:** `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Default` (z.B. `Floating`).
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum LayoutType {
            Tiling,
            #[default]
            Floating,
            Monocle,
            Custom(String),
        }
        ```
        
- **2.2.4. Wertobjekt: `TilingConfiguration`**
    
    - Spezifische Konfigurationseinstellungen für Tiling-Layouts.
    - **Attribute:**
        - `master_slave_ratio: f32` (Verhältnis der Master- zur Slave-Fläche, z.B. 0.5 für 50/50). Sichtbarkeit: `pub`. Initialwert: z.B. `0.5`. Invarianten: 0.1≤ratio≤0.9.
        - `num_master_windows: u32` (Anzahl der Fenster im Master-Bereich). Sichtbarkeit: `pub`. Initialwert: z.B. `1`. Invarianten: ≥1.
        - `orientation: TilingOrientation` (Ausrichtung der Tiling-Anordnung). Sichtbarkeit: `pub`. Initialwert: `TilingOrientation::Vertical`.
        - `gap_size: u32` (Abstand zwischen Fenstern in logischen Einheiten). Sichtbarkeit: `pub`. Initialwert: z.B. `5`. Invarianten: ≥0.
    - **Implementiert:** `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`, `PartialEq`.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub struct TilingConfiguration {
            pub master_slave_ratio: f32,
            pub num_master_windows: u32,
            pub orientation: TilingOrientation,
            pub gap_size: u32,
        }
        
        impl Default for TilingConfiguration {
            fn default() -> Self {
                Self {
                    master_slave_ratio: 0.5,
                    num_master_windows: 1,
                    orientation: TilingOrientation::Vertical,
                    gap_size: 5,
                }
            }
        }
        ```
        
- **2.2.5. Enum: `TilingOrientation`**
    
    - Definiert die Hauptausrichtung für Tiling-Layouts.
    - **Varianten:** `Horizontal` (Master-Bereich links/rechts), `Vertical` (Master-Bereich oben/unten).
    - **Implementiert:** `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Default` (z.B. `Vertical`).
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum TilingOrientation {
            Horizontal,
            #[default]
            Vertical,
        }
        ```
        
- **2.2.6. Fehler-Enum: `WorkspaceError`**
    
    - Spezifische Fehler, die im `workspace_manager`-Modul auftreten können. Definiert mit `thiserror`.
    - **Varianten:**
        - `#` `SpaceNotFound { space_id: DomainId }`
        - `#` `WindowAlreadyInSpace { window_id: DomainId, space_id: DomainId }`
        - `#` `WindowNotInSpace { window_id: DomainId, space_id: DomainId }`
        - `#[error("Invalid layout configuration: {reason}")]` `InvalidLayoutConfiguration { reason: String }`
        - `#[error("A space with the name '{name}' already exists.")]` `DuplicateSpaceName { name: String }`
        - `#[error("Maximum number of spaces reached.")]` `MaxSpacesReached`
        - `#` `CannotDeleteLastSpace`
        - `#` `CannotDeleteNonEmptySpace { space_id: DomainId, window_count: usize }`
        - `#` `EmptySpaceName`
        - `#` `MissingTilingConfiguration`
        - `#` `UnexpectedTilingConfiguration`
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum WorkspaceError {
            #
            SpaceNotFound { space_id: DomainId },
            #
            WindowAlreadyInSpace { window_id: DomainId, space_id: DomainId },
            #
            WindowNotInSpace { window_id: DomainId, space_id: DomainId },
            #[error("Invalid layout configuration: {reason}")]
            InvalidLayoutConfiguration { reason: String },
            #[error("A space with the name '{name}' already exists.")]
            DuplicateSpaceName { name: String },
            #[error("Maximum number of spaces reached.")]
            MaxSpacesReached,
            #
            CannotDeleteLastSpace,
            #
            CannotDeleteNonEmptySpace { space_id: DomainId, window_count: usize },
            #
            EmptySpaceName,
            #
            MissingTilingConfiguration,
            #
            UnexpectedTilingConfiguration,
        }
        ```
        

### 2.3. Service: `SpaceService`

Der `SpaceService` ist die zentrale Komponente für die Orchestrierung aller Operationen im Zusammenhang mit Spaces. Er hält den Zustand aller bekannten Spaces (typischerweise in einer `HashMap<DomainId, Space>`) und nutzt intern die Methoden der `Space`-Entität zur Manipulation einzelner Spaces.

- **2.3.1. Eigenschaften (Interner Zustand des `SpaceService`)**
    
    - `active_space_id: Option<DomainId>`: Die ID des aktuell aktiven/fokussierten Space. Zugriff über Methoden.
    - `spaces: std::collections::HashMap<DomainId, Space>`: Eine Map, die alle bekannten Spaces anhand ihrer ID speichert.
    - `space_order: Vec<DomainId>`: Eine geordnete Liste der Space-IDs, um eine konsistente Reihenfolge (z.B. für UI-Anzeige oder Space-Navigation) beizubehalten.
    - `max_spaces: usize`: Maximale Anzahl erlaubter Spaces (konfigurierbar, z.B. Default 10).
    - `event_publisher: Box<dyn DomainEventPublisher>`: Eine Abstraktion zum Senden von Domänenereignissen (wird bei Initialisierung injiziert).
- **2.3.2. Methoden (Öffentliche API des `SpaceService`)**
    
    Alle Methoden, die den Zustand verändern (`&mut self`), sollten bei Erfolg relevante `WorkspaceEvent`s über den `event_publisher` emittieren. Die Zeitstempel werden typischerweise von einer `Clock`-Abstraktion bezogen, die von der Kernschicht bereitgestellt und dem Service injiziert wird.
    
    - `pub fn new(max_spaces: usize, event_publisher: Box<dyn DomainEventPublisher>, initial_spaces_config: Option<Vec<InitialSpaceConfig>>) -> Self`
        - Konstruktor. Initialisiert den Service. Erstellt einen Default-Space, falls `initial_spaces_config` `None` oder leer ist.
        - `event_publisher` ist eine Trait-Implementierung, die von der Anwendungsschicht bereitgestellt wird, um Events zu publizieren.
        - `InitialSpaceConfig { name: String, layout_type: LayoutType, tiling_config: Option<TilingConfiguration> }`
    - `pub fn create_space(&mut self, name: String, layout_type: LayoutType, tiling_config: Option<TilingConfiguration>) -> Result<DomainId, WorkspaceError>`
        - **Beschreibung:** Erstellt einen neuen Space.
        - **Parameter:**
            - `name: String`: Der gewünschte Name für den neuen Space.
            - `layout_type: LayoutType`: Der initiale Layout-Typ des Space.
            - `tiling_config: Option<TilingConfiguration>`: Konfiguration für Tiling, falls `layout_type` dies erfordert.
        - **Rückgabe:** `Result<DomainId, WorkspaceError>` - Die ID des neu erstellten Space oder ein Fehler.
        - **Vorbedingungen:**
            - `name` darf nicht leer sein (`WorkspaceError::EmptySpaceName`).
            - Anzahl der Spaces < `max_spaces` (`WorkspaceError::MaxSpacesReached`).
            - `name` sollte eindeutig sein (optional, sonst `WorkspaceError::DuplicateSpaceName` oder automatische Umbenennung).
            - Wenn `layout_type == LayoutType::Tiling`, muss `tiling_config` `Some` sein (`WorkspaceError::MissingTilingConfiguration`).
            - Wenn `layout_type!= LayoutType::Tiling`, sollte `tiling_config` `None` sein (optional, sonst `WorkspaceError::UnexpectedTilingConfiguration` oder Ignorieren).
        - **Nachbedingungen:** Ein neuer `Space` existiert im Service. `space_order` wird aktualisiert.
        - **Emittiert:** `WorkspaceEvent::SpaceCreated { space_id, name, layout_type, tiling_config }`.
    - `pub fn get_space(&self, space_id: &DomainId) -> Result<&Space, WorkspaceError>`
        - **Beschreibung:** Gibt eine unveränderliche Referenz auf einen Space anhand seiner ID zurück.
        - **Parameter:** `space_id: &DomainId`.
        - **Rückgabe:** `Result<&Space, WorkspaceError>` (`WorkspaceError::SpaceNotFound`).
    - `pub fn list_spaces(&self) -> Vec<&Space>`
        - **Beschreibung:** Gibt eine Liste von Referenzen auf alle Spaces in der durch `space_order` definierten Reihenfolge zurück.
        - **Rückgabe:** `Vec<&Space>`.
    - `pub fn update_space_name(&mut self, space_id: &DomainId, new_name: String) -> Result<(), WorkspaceError>`
        - **Beschreibung:** Aktualisiert den Namen eines existierenden Space.
        - **Parameter:** `space_id: &DomainId`, `new_name: String`.
        - **Vorbedingungen:** `new_name` nicht leer und (optional) eindeutig.
        - **Emittiert:** `WorkspaceEvent::SpaceRenamed { space_id: space_id.clone(), new_name }`.
    - `pub fn delete_space(&mut self, space_id: &DomainId, force_delete_windows: bool) -> Result<(), WorkspaceError>`
        - **Beschreibung:** Löscht einen Space.
        - **Parameter:** `space_id: &DomainId`, `force_delete_windows: bool`.
        - **Vorbedingungen:**
            - Es muss mehr als ein Space existieren (`WorkspaceError::CannotDeleteLastSpace`).
            - Wenn `force_delete_windows == false` und der Space Fenster enthält, wird `WorkspaceError::CannotDeleteNonEmptySpace` zurückgegeben.
        - **Logik:** Wenn `force_delete_windows == true` und Fenster im Space sind, werden diese Fenster in einen Default-Space (z.B. den ersten in `space_order` oder den aktiven, falls nicht der zu löschende) verschoben. Für jedes verschobene Fenster wird `WorkspaceEvent::WindowMovedBetweenSpaces` emittiert.
        - **Nachbedingungen:** Der Space ist entfernt. `space_order` ist aktualisiert. Wenn der gelöschte Space aktiv war, wird ein anderer Space (z.B. der nächste in der Liste) aktiv gesetzt (`ActiveSpaceChanged` Event).
        - **Emittiert:** `WorkspaceEvent::SpaceDeleted { space_id: space_id.clone() }`.
    - `pub fn add_window_to_space(&mut self, space_id: &DomainId, window_handle: WindowHandle) -> Result<(), WorkspaceError>`
        - **Beschreibung:** Fügt ein Fenster zu einem spezifischen Space hinzu. Wenn das Fenster bereits in einem anderen Space ist, wird es implizit daraus entfernt (oder es wird ein Fehler zurückgegeben, je nach Designentscheidung – hier wird angenommen, es wird verschoben).
        - **Vorbedingungen:** Der Ziel-Space existiert. Das Fenster ist nicht bereits im Ziel-Space.
        - **Logik:** Prüft, ob das Fenster in einem anderen Space ist. Falls ja, `remove_window_from_space` für den Quell-Space aufrufen. Dann zum Ziel-Space hinzufügen.
        - **Emittiert:** `WorkspaceEvent::WindowAddedToSpace { space_id: space_id.clone(), window_id: window_handle.id.clone() }`. Wenn es von einem anderen Space verschoben wurde, wird stattdessen `WindowMovedBetweenSpaces` emittiert.
    - `pub fn remove_window_from_space(&mut self, space_id: &DomainId, window_handle: &WindowHandle) -> Result<(), WorkspaceError>`
        - **Beschreibung:** Entfernt ein Fenster aus einem spezifischen Space.
        - **Vorbedingungen:** Der Space existiert und enthält das Fenster.
        - **Emittiert:** `WorkspaceEvent::WindowRemovedFromSpace { space_id: space_id.clone(), window_id: window_handle.id.clone() }`.
    - `pub fn move_window_to_space(&mut self, window_handle: &WindowHandle, target_space_id: &DomainId) -> Result<(), WorkspaceError>`
        - **Beschreibung:** Verschiebt ein Fenster von seinem aktuellen Space (falls vorhanden) in den `target_space_id`.
        - **Logik:** Findet den Quell-Space des Fensters. Ruft `remove_window_from_space` für den Quell-Space und `add_window_to_space` für den Ziel-Space auf.
        - **Emittiert:** `WorkspaceEvent::WindowMovedBetweenSpaces { window_id: window_handle.id.clone(), source_space_id, target_space_id: target_space_id.clone() }`.
    - `pub fn set_space_layout(&mut self, space_id: &DomainId, layout_type: LayoutType, tiling_config: Option<TilingConfiguration>) -> Result<(), WorkspaceError>`
        - **Beschreibung:** Ändert das Layout eines Space.
        - **Vorbedingungen:** Validierung der `tiling_config` analog zu `create_space`.
        - **Emittiert:** `WorkspaceEvent::SpaceLayoutChanged { space_id: space_id.clone(), new_layout: layout_type, new_config: tiling_config }`.
    - `pub fn get_active_space_id(&self) -> Option<DomainId>`
        - **Beschreibung:** Gibt die ID des aktuell aktiven Space zurück.
    - `pub fn set_active_space(&mut self, space_id: &DomainId) -> Result<(), WorkspaceError>`
        - **Beschreibung:** Setzt den aktiven Space.
        - **Vorbedingungen:** Der Space mit `space_id` existiert.
        - **Nachbedingungen:** `active_space_id` ist aktualisiert. `last_accessed_timestamp` des neuen aktiven Space wird aktualisiert.
        - **Emittiert:** `WorkspaceEvent::ActiveSpaceChanged { old_space_id: self.active_space_id.clone(), new_space_id: space_id.clone() }`.
    - `pub fn get_windows_in_space(&self, space_id: &DomainId) -> Result<Vec<WindowHandle>, WorkspaceError>`
        - **Beschreibung:** Gibt eine Kopie der Liste der Fenster-Handles für einen gegebenen Space zurück.
    - `pub fn find_space_for_window(&self, window_handle: &WindowHandle) -> Option<DomainId>`
        - **Beschreibung:** Gibt die ID des Space zurück, der das angegebene Fenster enthält, falls vorhanden.
    - `pub fn reorder_window_in_space(&mut self, space_id: &DomainId, window_handle: &WindowHandle, new_index: usize) -> Result<(), WorkspaceError>`
        - **Beschreibung:** Ändert die Position eines Fensters innerhalb der `windows`-Liste eines Space (relevant für Stacking-Order oder Tiling).
        - **Emittiert:** `WorkspaceEvent::WindowReorderedInSpace { space_id: space_id.clone(), window_id: window_handle.id.clone(), new_index }`.
    - `pub fn reorder_space(&mut self, space_id: &DomainId, new_index: usize) -> Result<(), WorkspaceError>`
        - **Beschreibung:** Ändert die Position eines Space in der globalen `space_order`-Liste.
        - **Emittiert:** `WorkspaceEvent::SpaceReordered { space_id: space_id.clone(), new_index }`.
- **2.3.3. Signale/Events (als Varianten von `WorkspaceEvent` im `DomainEvent` Enum)**
    
    Diese Events werden vom `SpaceService` emittiert und über den injizierten `DomainEventPublisher` verteilt.
    
    - `SpaceCreated { space_id: DomainId, name: String, layout_type: LayoutType, tiling_config: Option<TilingConfiguration> }`
    - `SpaceDeleted { space_id: DomainId }`
    - `SpaceRenamed { space_id: DomainId, new_name: String }`
    - `SpaceLayoutChanged { space_id: DomainId, new_layout: LayoutType, new_config: Option<TilingConfiguration> }`
    - `WindowAddedToSpace { space_id: DomainId, window_id: DomainId }`
    - `WindowRemovedFromSpace { space_id: DomainId, window_id: DomainId }`
    - `WindowMovedBetweenSpaces { window_id: DomainId, source_space_id: DomainId, target_space_id: DomainId }`
    - `ActiveSpaceChanged { old_space_id: Option<DomainId>, new_space_id: DomainId }`
    - `WindowReorderedInSpace { space_id: DomainId, window_id: DomainId, new_index: usize }`
    - `SpaceReordered { space_id: DomainId, new_index: usize }`
    - **Typische Publisher:** `SpaceService`.
    - **Typische Subscriber:** UI-Schicht (zur Aktualisierung der Darstellung), `WindowPolicyEngine` (um ggf. auf Änderungen zu reagieren, z.B. um Fenster neu anzuordnen, wenn sich der aktive Space ändert), Persistenzmechanismus in der Kernschicht (um Änderungen zu speichern).
- **2.3.4. Trait: `DomainEventPublisher` (von Anwendungsschicht zu implementieren)**
    
    Rust
    
    ```
    pub trait DomainEventPublisher: Send + Sync {
        fn publish(&self, event: DomainEvent);
    }
    ```
    
    Dieser Trait wird benötigt, damit der `SpaceService` (und andere Services) Ereignisse publizieren können, ohne eine konkrete Implementierung eines Event-Busses zu kennen.
    

### 2.4. Geschäftsregeln und Invarianten (Beispiele)

- Ein Fenster kann immer nur in genau einem Space sein. Dies wird durch die Logik in `add_window_to_space` und `move_window_to_space` sichergestellt, die ein Fenster implizit aus seinem vorherigen Space entfernt.
- Es muss immer mindestens ein Space geben. `delete_space` verhindert das Löschen des letzten Space.
- Der Name eines Space sollte eindeutig sein (optional, konfigurierbar, ob Duplikate mit Suffix versehen oder abgelehnt werden). Die Methode `create_space` prüft dies.
- Spezifische Regeln für Tiling-Layouts (z.B. Mindestgröße von Fenstern, Verhalten bei Hinzufügen/Entfernen) werden innerhalb der `TilingConfiguration` und der Logik, die dieses Layout anwendet (potenziell in der UI-Schicht oder einer spezialisierten Layout-Engine, die von der Domänenschicht gesteuert wird), durchgesetzt. Die Domänenschicht speichert nur die Konfiguration.
- Maximale Anzahl von Spaces (`max_spaces`): Wird in `create_space` geprüft.

Die folgende Tabelle fasst die Attribute der `Space`-Entität zusammen:

**Tabelle 2.2.1: Entität `Space` - Attribute**

|   |   |   |   |   |
|---|---|---|---|---|
|**Attribut**|**Typ**|**Sichtbarkeit**|**Initialwert (Beispiel)**|**Invarianten**|
|`id`|`DomainId`|`pub(crate)`|Generiert|Eindeutig, unveränderlich|
|`name`|`String`|`pub`|Bei Erstellung|Nicht leer|
|`layout_type`|`LayoutType`|`pub`|`LayoutType::Floating`|Gültiger `LayoutType`|
|`windows`|`std::collections::VecDeque<WindowHandle>`|`pub(crate)`|Leer|Enthält nur gültige `WindowHandle`s|
|`tiling_config`|`Option<TilingConfiguration>`|`pub`|`None`|`Some` gdw. `layout_type` ist Tiling-Variante|
|`creation_timestamp`|`u64`|`pub(crate)`|Zeitstempel bei Erstellung||
|`last_accessed_timestamp`|`u64`|`pub(crate)`|Zeitstempel bei Erstellung||

Die öffentliche API des `SpaceService` ist in der folgenden Tabelle dargestellt:

**Tabelle 2.3.2: `SpaceService` - Öffentliche API (Auswahl)**

|   |   |   |   |
|---|---|---|---|
|**Methode**|**Parameter**|**Rückgabetyp**|**Kurzbeschreibung**|
|`create_space`|`name: String`, `layout: LayoutType`, `config: Option<TilingConfiguration>`|`Result<DomainId, WorkspaceError>`|Erstellt einen neuen Space.|
|`get_space`|`space_id: &DomainId`|`Result<&Space, WorkspaceError>`|Ruft einen Space anhand seiner ID ab.|
|`list_spaces`||`Vec<&Space>`|Listet alle Spaces auf.|
|`delete_space`|`space_id: &DomainId`, `force: bool`|`Result<(), WorkspaceError>`|Löscht einen Space.|
|`add_window_to_space`|`space_id: &DomainId`, `window: WindowHandle`|`Result<(), WorkspaceError>`|Fügt ein Fenster zu einem Space hinzu.|
|`move_window_to_space`|`window: &WindowHandle`, `target_space_id: &DomainId`|`Result<(), WorkspaceError>`|Verschiebt ein Fenster in einen anderen Space.|
|`set_active_space`|`space_id: &DomainId`|`Result<(), WorkspaceError>`|Setzt den aktiven Space.|
|`get_active_space_id`||`Option<DomainId>`|Gibt die ID des aktiven Space zurück.|

---

## 3. Modul: Theming-System (`theming_manager`)

### 3.1. Übersicht und Verantwortlichkeiten

Das `theming_manager`-Modul ist für die Verwaltung von Themes und deren Design-Tokens zuständig. Es ermöglicht die Anwendung eines aktiven Themes und dessen Variante (z.B. Hell/Dunkel) und stellt Mechanismen bereit, über die die UI-Schicht Token-Werte abfragen kann. Das Laden und Speichern von Theme-Definitionen erfolgt über Abstraktionen (Ports), die von der Kernschicht implementiert werden.

Ein Token-basiertes Theming-System ist hierbei von zentraler Bedeutung.26 Die Domänenschicht verwaltet die _Definition_ und _Auswahl_ der Tokens. Die _Interpretation und Anwendung_ dieser Tokens (z.B. das Rendern von CSS für GTK-Anwendungen) ist Aufgabe der UI-Schicht. GTK4-CSS-Konzepte, wie sie in 28 beschrieben sind (z.B. Custom Properties wie `--prop: value; color: var(--prop);`), dienen als starkes konzeptionelles Vorbild für die Struktur der Tokens, auch wenn die Domänenschicht selbst kein CSS direkt verarbeitet oder generiert. Die UI-Unabhängigkeit der Domänenschicht bedingt, dass das Theming-System keine direkten Zeichenoperationen durchführt. Stattdessen liefert es die notwendigen Informationen, beispielsweise einen Token `primary_background_color: "#FFFFFF"`. Die UI-Schicht fragt diesen Token-Wert ab und verwendet ihn in ihrem spezifischen Rendering-System (z.B. GTK CSS, Qt Stylesheets oder Web CSS). Dieser Ansatz fördert Konsistenz, ermöglicht schnellere Updates und verbessert die Skalierbarkeit des Theming-Systems.26

### 3.2. Entitäten, Wertobjekte und Enums

- **3.2.1. Entität: `Theme`**
    
    - Repräsentiert eine vollständige Theme-Definition.
    - **Attribute:**
        - `id: DomainId` (Eindeutige ID des Themes, z.B. "arc-dark-custom"). Sichtbarkeit: `pub`. Invarianten: Eindeutig, nicht leer.
        - `name: String` (Anzeigename, z.B. "Arc Dark Custom"). Sichtbarkeit: `pub`. Invarianten: Nicht leer.
        - `description: Option<String>` (Optionale Beschreibung des Themes). Sichtbarkeit: `pub`.
        - `author: Option<String>` (Optionaler Autor des Themes). Sichtbarkeit: `pub`.
        - `version: String` (Version des Themes, z.B. "1.0.0"). Sichtbarkeit: `pub`. Invarianten: Nicht leer.
        - `supported_variants: Vec<ThemeVariantType>` (Liste der unterstützten Varianten, z.B. ``). Sichtbarkeit: `pub`. Invarianten: Muss mindestens eine Variante enthalten.
        - `tokens: std::collections::HashMap<String, ThemeToken>` (Schlüssel ist der hierarchische Token-Name, z.B. "color.background.primary"). Sichtbarkeit: `pub(crate)`.
        - `metadata: std::collections::HashMap<String, String>` (Zusätzliche Metadaten, z.B. Pfad zur Quelldatei, Lizenz). Sichtbarkeit: `pub`.
    - **Methoden (interne Logik der `Theme`-Entität):**
        - `pub(crate) fn get_token_value(&self, token_name: &str, variant: ThemeVariantType) -> Option<&ThemeTokenValue>`
            - Sucht den Token mit `token_name`.
            - Gibt `value_dark` zurück, wenn `variant == Dark` und `value_dark` `Some` ist.
            - Gibt ansonsten `value_light` zurück, wenn der Token die Variante unterstützt (implizit, da `value_light` obligatorisch ist).
            - Gibt `None` zurück, wenn der Token nicht existiert oder die spezifische Variante nicht explizit definiert ist und keine Ableitungsregel existiert (für diese Spezifikation wird keine komplexe Ableitung angenommen).
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub struct Theme {
            pub id: DomainId,
            pub name: String,
            pub description: Option<String>,
            pub author: Option<String>,
            pub version: String,
            pub supported_variants: Vec<ThemeVariantType>,
            pub(crate) tokens: std::collections::HashMap<String, ThemeToken>,
            pub metadata: std::collections::HashMap<String, String>,
        }
        ```
        
- **3.2.2. Wertobjekt: `ThemeToken`**
    
    - Definiert einen einzelnen Design-Token. Die Trennung von `value_light` und `value_dark` direkt im Token ermöglicht eine explizite Definition pro Variante und vereinfacht die Abfrage.
    - **Attribute:**
        - `name: String` (Eindeutiger, hierarchischer Name, z.B. "color.text.primary", "font.body.family", "spacing.medium"). Sichtbarkeit: `pub`. Invarianten: Nicht leer, folgt einer Namenskonvention (z.B. `kebab-case` oder `snake_case`).
        - `token_type: ThemeTokenType` (Typ des Tokens). Sichtbarkeit: `pub`.
        - `value_light: ThemeTokenValue` (Wert für die helle Variante). Sichtbarkeit: `pub`. Invarianten: Muss zum `token_type` passen.
        - `value_dark: Option<ThemeTokenValue>` (Optionaler spezifischer Wert für die dunkle Variante; falls `None`, wird `value_light` verwendet). Sichtbarkeit: `pub`. Invarianten: Falls `Some`, muss zum `token_type` passen.
        - `description: Option<String>` (Beschreibung des Tokens und seines Verwendungszwecks). Sichtbarkeit: `pub`.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub struct ThemeToken {
            pub name: String,
            pub token_type: ThemeTokenType,
            pub value_light: ThemeTokenValue,
            pub value_dark: Option<ThemeTokenValue>,
            pub description: Option<String>,
        }
        ```
        
- **3.2.3. Enum: `ThemeTokenType`**
    
    - Klassifiziert die Art eines `ThemeToken`.
    - **Varianten:** `Color` (String, z.B. "#RRGGBBAA" oder "rgba(r,g,b,a)"), `FontSize` (String, z.B. "12pt", "1.2em"), `FontFamily` (String, z.B. "Noto Sans"), `Spacing` (String, z.B. "8px"), `BorderRadius` (String, z.B. "4px"), `Shadow` (String, CSS-ähnliche Definition, z.B. "2px 2px 5px rgba(0,0,0,0.3)"), `IconSet` (String, Name eines Icon-Sets), `Custom(String)` (für anwendungsspezifische Tokens, z.B. "animation.duration.fast").
    - **Implementiert:** `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum ThemeTokenType {
            Color,
            FontSize,
            FontFamily,
            Spacing,
            BorderRadius,
            Shadow,
            IconSet,
            Custom(String),
        }
        ```
        
- **3.2.4. Wertobjekt: `ThemeTokenValue`**
    
    - Repräsentiert den konkreten Wert eines Tokens. Für Einfachheit wird hier `String` verwendet; die UI-Schicht interpretiert den String basierend auf `ThemeTokenType`.
    - **Attribute:**
        - `value: String`. Sichtbarkeit: `pub`.
    - **Implementiert:** `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub struct ThemeTokenValue {
            pub value: String,
        }
        ```
        
- **3.2.5. Wertobjekt: `ActiveThemeState`**
    
    - Speichert den aktuell aktiven Theme-Zustand.
    - **Attribute:**
        - `theme_id: DomainId` (ID des aktiven Themes). Sichtbarkeit: `pub`.
        - `variant: ThemeVariantType` (Aktive Variante). Sichtbarkeit: `pub`.
    - **Implementiert:** `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Default`.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub struct ActiveThemeState {
            pub theme_id: DomainId,
            pub variant: ThemeVariantType,
        }
        ```
        
- **3.2.6. Enum: `ThemeVariantType`**
    
    - Definiert die möglichen Varianten eines Themes.
    - **Varianten:** `Light`, `Dark`.
    - **Implementiert:** `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Default` (z.B. `Light`).
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum ThemeVariantType {
            #[default]
            Light,
            Dark,
        }
        ```
        
- **3.2.7. Fehler-Enum: `ThemingError`**
    
    - Spezifische Fehler für das `theming_manager`-Modul. Definiert mit `thiserror`.
    - **Varianten:**
        - `#` `ThemeNotFound { theme_id: DomainId }`
        - `#` `TokenNotFound { theme_id: DomainId, token_name: String }`
        - `#[error("Variant '{variant:?}' not supported by theme '{theme_id}'.")]` `VariantNotSupported { theme_id: DomainId, variant: ThemeVariantType }`
        - `#[error("Invalid token value for '{token_name}': '{value}'. Expected type: {expected_type:?}.")]` `InvalidTokenValue { token_name: String, value: String, expected_type: ThemeTokenType }`
        - `#[error("Failed to load theme from '{path}': {reason}")]` `ThemeLoadError { path: String, reason: String }`
        - `#[error("Failed to save theme to '{path}': {reason}")]` `ThemeSaveError { path: String, reason: String }`
        - `#` `ThemeIdExists { theme_id: DomainId }`
        - `#` `DefaultThemeActivationFailed { theme_id: DomainId, reason: String }`
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum ThemingError {
            #
            ThemeNotFound { theme_id: DomainId },
            #
            TokenNotFound { theme_id: DomainId, token_name: String },
            #[error("Variant '{variant:?}' not supported by theme '{theme_id}'.")]
            VariantNotSupported { theme_id: DomainId, variant: ThemeVariantType },
            #[error("Invalid token value for '{token_name}': '{value}'. Expected type: {expected_type:?}.")]
            InvalidTokenValue { token_name: String, value: String, expected_type: ThemeTokenType },
            #[error("Failed to load theme from '{path}': {reason}")]
            ThemeLoadError { path: String, reason: String },
            #[error("Failed to save theme to '{path}': {reason}")]
            ThemeSaveError { path: String, reason: String },
            #
            ThemeIdExists { theme_id: DomainId },
            #
            DefaultThemeActivationFailed { theme_id: DomainId, reason: String },
        }
        ```
        

### 3.3. Service: `ThemingService`

Der `ThemingService` verwaltet die geladenen Themes und den aktiven Theme-Zustand. Er interagiert mit einem `ThemePersistencePort` (implementiert von der Kernschicht) zum Laden und Speichern von Theme-Daten und dem aktiven Zustand.

- **3.3.1. Eigenschaften (Interner Zustand des `ThemingService`)**
    
    - `loaded_themes: std::collections::HashMap<DomainId, Theme>`: Map aller geladenen Themes.
    - `active_theme_state: ActiveThemeState`: Der aktuell ausgewählte Theme und dessen Variante.
    - `theme_persistence_port: Box<dyn ThemePersistencePort>`: Injizierter Port für Persistenz.
    - `event_publisher: Box<dyn DomainEventPublisher>`: Injizierter Publisher für Domänenereignisse.
    - `default_theme_id: DomainId`: ID eines fest einkompilierten oder als sicher bekannten Fallback-Themes.
- **3.3.2. Methoden (Öffentliche API des `ThemingService`)**
    
    - `pub fn new(default_theme_id: DomainId, default_theme: Theme, theme_persistence_port: Box<dyn ThemePersistencePort>, event_publisher: Box<dyn DomainEventPublisher>) -> Self`
        - Konstruktor. Lädt initial verfügbare Themes über den `theme_persistence_port` und den gespeicherten `ActiveThemeState`.
        - Fügt das `default_theme` zu `loaded_themes` hinzu.
        - Versucht, den gespeicherten aktiven Theme-Zustand zu laden. Falls nicht erfolgreich oder inkonsistent, wird `default_theme_id` mit `ThemeVariantType::default()` aktiviert.
    - `pub fn list_available_themes(&self) -> Vec<(DomainId, String)>`
        - Gibt eine Liste von Tupeln `(id, name)` aller geladenen Themes zurück.
    - `pub fn get_theme_details(&self, theme_id: &DomainId) -> Result<&Theme, ThemingError>`
        - Gibt Details zu einem spezifischen Theme zurück.
    - `pub fn set_active_theme(&mut self, theme_id: &DomainId) -> Result<(), ThemingError>`
        - Setzt das aktive Theme. Die Variante bleibt unverändert, sofern vom neuen Theme unterstützt, sonst wird die Default-Variante des neuen Themes gewählt.
        - Vorbedingungen: Theme mit `theme_id` muss existieren und die aktuelle (oder eine Default-) Variante unterstützen.
        - Nachbedingungen: `active_theme_state.theme_id` ist aktualisiert.
        - Emittiert: `ThemingEvent::ActiveThemeChanged { new_theme_id: theme_id.clone(), new_variant: self.active_theme_state.variant }`.
        - Persistiert den neuen Zustand über `theme_persistence_port.save_active_theme_state()`.
    - `pub fn set_active_variant(&mut self, variant: ThemeVariantType) -> Result<(), ThemingError>`
        - Setzt die aktive Theme-Variante.
        - Vorbedingungen: Das aktuell aktive Theme muss die `variant` unterstützen.
        - Nachbedingungen: `active_theme_state.variant` ist aktualisiert.
        - Emittiert: `ThemingEvent::ThemeVariantChanged { new_variant: variant }`.
        - Persistiert den neuen Zustand.
    - `pub fn get_active_theme_id(&self) -> &DomainId`
    - `pub fn get_active_variant(&self) -> ThemeVariantType`
    - `pub fn get_token_value(&self, token_name: &str) -> Result<ThemeTokenValue, ThemingError>`
        - Gibt den Wert des angeforderten Tokens für das aktuell aktive Theme und die aktive Variante zurück.
        - Nutzt intern `Theme::get_token_value()`.
        - Fallback: Wenn Token im spezifischen Theme nicht gefunden, könnte ein Fallback auf das Default-Theme erfolgen (optional, muss klar definiert sein).
    - `pub fn get_specific_token_value(&self, theme_id: &DomainId, variant: ThemeVariantType, token_name: &str) -> Result<ThemeTokenValue, ThemingError>`
        - Gibt den Wert eines Tokens für ein spezifisches (nicht notwendigerweise aktives) Theme und eine spezifische Variante zurück.
    - `pub fn reload_themes(&mut self) -> Result<usize, ThemingError>`
        - Lädt alle Themes von den bekannten Pfaden (die der `ThemePersistencePort` kennt) neu.
        - Aktualisiert `loaded_themes`. Stellt sicher, dass das `active_theme_state` gültig bleibt (ggf. Fallback auf Default-Theme).
        - Gibt die Anzahl der erfolgreich geladenen Themes zurück.
        - Emittiert: `ThemingEvent::ThemesReloaded { num_loaded }`.
    - `pub fn add_theme(&mut self, theme: Theme) -> Result<(), ThemingError>`
        - Fügt ein neues Theme dynamisch hinzu (z.B. von Benutzer importiert).
        - Vorbedingungen: `theme.id` darf nicht bereits existieren.
        - Speichert das Theme über den `ThemePersistencePort`.
        - Emittiert: `ThemingEvent::ThemeAdded { theme_id: theme.id.clone() }`.
    - `pub fn remove_theme(&mut self, theme_id: &DomainId) -> Result<(), ThemingError>`
        - Entfernt ein Theme.
        - Vorbedingungen: Das Theme darf nicht das aktive Theme sein (oder es muss ein Fallback-Mechanismus greifen). Darf nicht das Default-Theme sein.
        - Löscht das Theme über den `ThemePersistencePort`.
        - Emittiert: `ThemingEvent::ThemeRemoved { theme_id: theme_id.clone() }`.
- **3.3.3. Signale/Events (als Varianten von `ThemingEvent` im `DomainEvent` Enum)**
    
    - `ActiveThemeChanged { new_theme_id: DomainId, new_variant: ThemeVariantType }`
    - `ThemeVariantChanged { new_variant: ThemeVariantType }`
    - `ThemesReloaded { num_loaded: usize }`
    - `ThemeAdded { theme_id: DomainId }`
    - `ThemeRemoved { theme_id: DomainId }`
    - `TokenValueChanged { theme_id: DomainId, variant: ThemeVariantType, token_name: String, new_value: ThemeTokenValue }` (Nur relevant, falls einzelne Tokens zur Laufzeit änderbar sein sollen, was typischerweise nicht der Fall ist für persistierte Themes, sondern eher für dynamische Anpassungen).
    - **Typische Publisher:** `ThemingService`.
    - **Typische Subscriber:** UI-Schicht (um auf Theme-Änderungen zu reagieren und die UI neu zu zeichnen/stilisieren), andere Domänendienste, die themenabhängige Logik haben könnten (selten).

### 3.4. Geschäftsregeln

- **Fallback-Mechanismen:** Wenn ein Token im aktiven Theme nicht definiert ist oder die spezifische Variante nicht abdeckt, wird der Wert des Tokens aus dem `default_theme_id` für die entsprechende Variante verwendet. Wenn auch dort nicht vorhanden, muss ein fest kodierter, anwendungsweiter Standardwert greifen (dieser ist außerhalb des ThemingService zu definieren, z.B. in der UI-Schicht als letzte Instanz).
- **Validierung von Theme-Dateien:** Beim Laden (durch den `ThemePersistencePort`) müssen Themes auf syntaktische Korrektheit und das Vorhandensein essentieller Tokens (z.B. Basisfarben, Standardschriftgrößen) geprüft werden. Fehlerhafte Themes werden nicht geladen.
- Das System muss immer ein gültiges aktives Theme haben. Das `default_theme` (mit `default_theme_id`) dient als garantierter Fallback.

### 3.5. Trait: `ThemePersistencePort` (von Kernschicht zu implementieren)

Dieser Port definiert die Schnittstelle, über die der `ThemingService` mit der Kernschicht für die Persistenz von Theme-Daten interagiert. Dies entkoppelt den Service von der konkreten Speicherimplementierung.

- `fn load_all_themes(&self) -> Result<Vec<Theme>, DomainError>;`
    - Lädt alle verfügbaren Theme-Definitionen von konfigurierten Speicherorten.
- `fn save_theme(&self, theme: &Theme) -> Result<(), DomainError>;`
    - Speichert eine einzelne Theme-Definition.
- `fn delete_theme(&self, theme_id: &DomainId) -> Result<(), DomainError>;`
    - Löscht eine Theme-Definition.
- `fn load_active_theme_state(&self) -> Result<Option<ActiveThemeState>, DomainError>;`
    - Lädt den zuletzt gespeicherten aktiven Theme-Zustand. Gibt `Ok(None)` zurück, wenn kein Zustand gespeichert ist.
- `fn save_active_theme_state(&self, state: &ActiveThemeState) -> Result<(), DomainError>;`
    - Speichert den aktuellen aktiven Theme-Zustand.

Die Implementierung dieses Traits in der Kernschicht würde typischerweise TOML-Dateien in XDG-Verzeichnissen (z.B. `$XDG_DATA_HOME/your_app_name/themes/` für Themes, `$XDG_CONFIG_HOME/your_app_name/theming_state.toml` für den aktiven Zustand) lesen und schreiben.

Die folgenden Tabellen fassen wichtige Aspekte des Theming-Systems zusammen:

**Tabelle 3.2.1: Entität `Theme` - Attribute**

|   |   |   |   |
|---|---|---|---|
|**Attribut**|**Typ**|**Sichtbarkeit**|**Invarianten**|
|`id`|`DomainId`|`pub`|Eindeutig, nicht leer|
|`name`|`String`|`pub`|Nicht leer|
|`description`|`Option<String>`|`pub`||
|`author`|`Option<String>`|`pub`||
|`version`|`String`|`pub`|Nicht leer|
|`supported_variants`|`Vec<ThemeVariantType>`|`pub`|Mindestens eine Variante|
|`tokens`|`std::collections::HashMap<String, ThemeToken>`|`pub(crate)`|Enthält gültige `ThemeToken`-Definitionen|
|`metadata`|`std::collections::HashMap<String, String>`|`pub`||

**Tabelle 3.2.2: Wertobjekt `ThemeToken` - Attribute**

|   |   |   |   |
|---|---|---|---|
|**Attribut**|**Typ**|**Sichtbarkeit**|**Invarianten**|
|`name`|`String`|`pub`|Nicht leer, hierarchisch (z.B. "color.text.primary")|
|`token_type`|`ThemeTokenType`|`pub`|Gültiger `ThemeTokenType`|
|`value_light`|`ThemeTokenValue`|`pub`|Passend zu `token_type`|
|`value_dark`|`Option<ThemeTokenValue>`|`pub`|Falls `Some`, passend zu `token_type`|
|`description`|`Option<String>`|`pub`||

**Tabelle 3.3.2: `ThemingService` - Öffentliche API (Auswahl)**

|   |   |   |   |
|---|---|---|---|
|**Methode**|**Parameter**|**Rückgabetyp**|**Kurzbeschreibung**|
|`list_available_themes`||`Vec<(DomainId, String)>`|Listet IDs und Namen aller geladenen Themes.|
|`set_active_theme`|`theme_id: &DomainId`|`Result<(), ThemingError>`|Setzt das aktive Theme.|
|`set_active_variant`|`variant: ThemeVariantType`|`Result<(), ThemingError>`|Setzt die aktive Theme-Variante.|
|`get_active_theme_id`||`&DomainId`|Gibt die ID des aktiven Themes zurück.|
|`get_active_variant`||`ThemeVariantType`|Gibt die aktive Variante zurück.|
|`get_token_value`|`token_name: &str`|`Result<ThemeTokenValue, ThemingError>`|Ruft Token-Wert für aktives Theme/Variante ab.|
|`reload_themes`||`Result<usize, ThemingError>`|Lädt alle Themes neu.|
|`add_theme`|`theme: Theme`|`Result<(), ThemingError>`|Fügt ein neues Theme hinzu und persistiert es.|
|`remove_theme`|`theme_id: &DomainId`|`Result<(), ThemingError>`|Entfernt ein Theme und löscht es aus der Persistenz.|

---

## 4. Modul: KI-Interaktionen (`ai_manager`)

### 4.1. Übersicht und Verantwortlichkeiten

Das `ai_manager`-Modul dient als zentrale Anlaufstelle für die Verwaltung aller KI-gestützten Funktionen innerhalb der Desktop-Umgebung. Es ist in zwei Hauptuntermodule gegliedert: `consent_manager` für die Verwaltung von Benutzereinwilligungen bezüglich der Datennutzung und Funktionsausführung durch KI, und `feature_service` für die Implementierung der eigentlichen Logik der KI-Features. Eine Kernaufgabe des Moduls ist es, sicherzustellen, dass KI-Funktionen nur mit expliziter, informierter und granularer Zustimmung des Benutzers ausgeführt werden. Es abstrahiert die Kommunikation mit potenziellen KI-Modellen oder -Diensten, deren Anbindung über die Kernschicht erfolgt.

Die Architektur dieses Moduls kann von den Konzepten des Model Context Protocol (MCP) profitieren, wie in 31 beschrieben. MCP schlägt eine Client-Server-Architektur vor, bei der "Hosts" (Anwendungen) über "Clients" mit "MCP-Servern" kommunizieren, die Zugriff auf Datenquellen und Werkzeuge bereitstellen. In diesem Kontext könnte die Domänenschicht als "Host" (oder Teil davon) agieren, der Anfragen an KI-Funktionen stellt. Die Kernschicht wäre dann verantwortlich für die Implementierung des "MCP-Clients" und die Anbindung an externe "MCP-Server" oder äquivalente KI-Dienste. Die Domänenschicht definiert dabei die _Struktur_ der Anfragen und der erwarteten Antworten sowie die Logik des Einwilligungsmanagements. KI-Funktionen benötigen oft Kontext (Daten) und die Fähigkeit, Aktionen auszuführen. MCP zielt darauf ab, diesen Zugriff zu standardisieren. Die Domänenschicht muss klar definieren, _welchen_ Kontext eine spezifische KI-Funktion benötigt (z.B. "aktueller Workspace", "aktives Fenster", "Benachrichtigungshistorie") und welche Aktionen sie ausführen darf (z.B. "Fenster anordnen", "Text vorschlagen"). Das `ConsentManager`-Untermodul stellt sicher, dass der Zugriff auf diesen Kontext und die Ausführung dieser Aktionen nur mit gültiger Benutzereinwilligung erfolgt.

### 4.2. Untermodul: Einwilligungsmanagement (`consent_manager`)

Dieses Untermodul ist verantwortlich für die Verwaltung und Persistenz der Benutzereinwilligungen für verschiedene KI-Funktionen und Datenzugriffe.

- **4.2.1. Entität: `UserConsent`**
    
    - Repräsentiert die Einwilligung eines Benutzers für ein spezifisches KI-Feature.
    - **Attribute:**
        - `user_id: DomainId` (Identifikator des Benutzers; bei Single-User-Systemen könnte dies ein konstanter Wert sein oder weggelassen werden, hier als `DomainId` für Flexibilität). Sichtbarkeit: `pub`. Invarianten: Nicht leer.
        - `feature_id: String` (Eindeutige ID des KI-Features, z.B. "ai.window_organizer", "ai.smart_reply.email"). Sichtbarkeit: `pub`. Invarianten: Nicht leer.
        - `is_granted: bool` (Status der Einwilligung). Sichtbarkeit: `pub`. Initialwert: `false`.
        - `last_updated_timestamp: u64` (Unix-Timestamp in Millisekunden der letzten Änderung). Sichtbarkeit: `pub(crate)`.
        - `scope: Option<String>` (Optionaler Geltungsbereich, z.B. "application:org.example.Mail", "global"; dient zur Verfeinerung der Einwilligung). Sichtbarkeit: `pub`.
        - `expires_at: Option<u64>` (Optionaler Unix-Timestamp in Millisekunden für den Ablauf der Einwilligung). Sichtbarkeit: `pub`.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub struct UserConsent {
            pub user_id: DomainId,
            pub feature_id: String,
            pub is_granted: bool,
            #[serde(default = "current_timestamp_ms")] // Helferfunktion für Default
            pub(crate) last_updated_timestamp: u64,
            pub scope: Option<String>,
            pub expires_at: Option<u64>,
        }
        
        // Beispiel für eine Helferfunktion für Default-Zeitstempel
        // Diese müsste im Kontext des Moduls verfügbar sein.
        // fn current_timestamp_ms() -> u64 {
        //     std::time::SystemTime::now()
        //        .duration_since(std::time::UNIX_EPOCH)
        //        .unwrap_or_default()
        //        .as_millis() as u64
        // }
        ```
        
- **4.2.2. Service: `ConsentService`**
    
    - Verwaltet den Zustand aller Benutzereinwilligungen. Interagiert mit einem `ConsentPersistencePort` (implementiert von der Kernschicht) zum Laden/Speichern von Einwilligungen. Die Kernschicht könnte zur sicheren Speicherung sensibler Einwilligungsdaten den Freedesktop Secret Service nutzen 33, was über den `ConsentPersistencePort` abstrakt angefordert würde.
    - **Eigenschaften (Interner Zustand):**
        - `consents: std::collections::HashMap<(DomainId, String), UserConsent>` (Schlüssel: `(user_id, feature_id)`).
        - `persistence_port: Box<dyn ConsentPersistencePort>`.
        - `event_publisher: Box<dyn DomainEventPublisher>`.
        - `clock: Box<dyn Clock>` (Für Zeitstempel, injiziert).
    - **Methoden (Öffentliche API):**
        - `pub fn new(persistence_port: Box<dyn ConsentPersistencePort>, event_publisher: Box<dyn DomainEventPublisher>, clock: Box<dyn Clock>, user_id_for_initial_load: &DomainId) -> Self`
            - Konstruktor. Lädt Einwilligungen für den `user_id_for_initial_load` beim Start.
        - `pub fn grant_consent(&mut self, user_id: DomainId, feature_id: String, scope: Option<String>, expires_at: Option<u64>) -> Result<(), AIConsentError>`
            - Erstellt oder aktualisiert eine `UserConsent`-Entität mit `is_granted = true`.
            - Setzt `last_updated_timestamp` auf die aktuelle Zeit.
            - Speichert über `persistence_port`.
            - Emittiert `AIConsentEvent::ConsentGranted { user_id, feature_id, scope, expires_at }`.
        - `pub fn revoke_consent(&mut self, user_id: &DomainId, feature_id: &str) -> Result<(), AIConsentError>`
            - Aktualisiert eine existierende `UserConsent`-Entität auf `is_granted = false`.
            - Setzt `last_updated_timestamp`.
            - Speichert über `persistence_port`.
            - Emittiert `AIConsentEvent::ConsentRevoked { user_id: user_id.clone(), feature_id: feature_id.to_string() }`.
        - `pub fn get_consent_status(&self, user_id: &DomainId, feature_id: &str) -> Result<&UserConsent, AIConsentError>`
            - Gibt die `UserConsent`-Entität zurück. Prüft intern auf Ablauf (`expires_at`). Wenn abgelaufen, wird `is_granted` als `false` interpretiert, auch wenn es `true` gespeichert ist (oder es wird ein `ConsentExpired` Fehler/Event ausgelöst).
            - Gibt `AIConsentError::ConsentNotFound` zurück, wenn keine explizite Einwilligung existiert (impliziert nicht gewährt).
        - `pub fn list_consents_for_user(&self, user_id: &DomainId) -> Vec<&UserConsent>`
            - Listet alle Einwilligungen für einen Benutzer auf.
        - `pub fn list_all_consents(&self) -> Vec<&UserConsent>`
            - Listet alle Einwilligungen im System (z.B. für administrative Zwecke).
        - `pub fn cleanup_expired_consents(&mut self) -> Result<u32, AIConsentError>`
            - Iteriert durch alle Einwilligungen und entfernt abgelaufene Einträge oder markiert sie als ungültig.
            - Gibt die Anzahl der entfernten/aktualisierten Einwilligungen zurück.
            - Emittiert `AIConsentEvent::ConsentExpired` für jede entfernte/aktualisierte Einwilligung.
            - Speichert Änderungen über `persistence_port`.
    - **Signale/Events (als Varianten von `AIConsentEvent` im `DomainEvent` Enum):**
        - `ConsentGranted { user_id: DomainId, feature_id: String, scope: Option<String>, expires_at: Option<u64> }`
        - `ConsentRevoked { user_id: DomainId, feature_id: String }`
        - `ConsentExpired { user_id: DomainId, feature_id: String }`
        - **Typische Publisher:** `ConsentService`.
        - **Typische Subscriber:** `AIFeatureService` (um zu prüfen, ob Features ausgeführt werden dürfen), UI-Schicht (um Einwilligungs-Dialoge und -Statusanzeigen zu aktualisieren).
- **4.2.3. Trait: `ConsentPersistencePort` (von Kernschicht zu implementieren)**
    
    - Definiert die Schnittstelle für die Persistenz von Einwilligungsdaten.
    - `fn load_consents_for_user(&self, user_id: &DomainId) -> Result<Vec<UserConsent>, DomainError>;`
    - `fn save_consent(&self, consent: &UserConsent) -> Result<(), DomainError>;`
    - `fn delete_consent(&self, user_id: &DomainId, feature_id: &str) -> Result<(), DomainError>;`
    - `fn load_all_consents(&self) -> Result<Vec<UserConsent>, DomainError>;` (Für Admin-Zwecke oder globalen Cleanup)
- **4.2.4. Trait: `Clock` (von Kernschicht zu implementieren)**
    
    Rust
    
    ```
    pub trait Clock: Send + Sync {
        fn current_timestamp_ms(&self) -> u64;
    }
    ```
    
- **4.2.5. Fehler-Enum: `AIConsentError`**
    
    - Spezifische Fehler für das `consent_manager`-Modul.
    - **Varianten:**
        - `#[error("Consent for user '{user_id}' and feature '{feature_id}' not found or not granted.")]` `ConsentNotFoundOrNotGranted { user_id: DomainId, feature_id: String }`
        - `#` `FeatureNotKnown { feature_id: String }`
        - `#` `StorageError { message: String }`
        - `#[error("Consent for user '{user_id}' and feature '{feature_id}' has expired.")]` `ConsentExpiredError { user_id: DomainId, feature_id: String }` // Interner Fehler, der zu ConsentNotFoundOrNotGranted führen kann.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum AIConsentError {
            #[error("Consent for user '{user_id}' and feature '{feature_id}' not found or not granted.")]
            ConsentNotFoundOrNotGranted { user_id: DomainId, feature_id: String },
            #
            FeatureNotKnown { feature_id: String },
            #
            StorageError { message: String },
            #[error("Consent for user '{user_id}' and feature '{feature_id}' has expired.")]
            ConsentExpiredError { user_id: DomainId, feature_id: String },
        }
        ```
        

### 4.3. Untermodul: KI-Funktionslogik (`feature_service`)

Dieses Untermodul enthält die Logik zur Ausführung spezifischer KI-Funktionen, nachdem die Einwilligung geprüft wurde.

- **4.3.1. Service: `AIFeatureService`**
    
    - Abhängig vom `ConsentService`, um Berechtigungen zu prüfen.
    - Definiert Schnittstellen für spezifische KI-Funktionen. Die Implementierung dieser Funktionen (d.h. die Interaktion mit den eigentlichen KI-Modellen) erfolgt in der Kernschicht oder einer dedizierten KI-Infrastrukturschicht, die über die Kernschicht angebunden ist. Der `AIFeatureService` orchestriert den Aufruf und verarbeitet die Ergebnisse.
    - **Eigenschaften (Interner Zustand):**
        - `consent_service: Arc<Mutex<ConsentService>>` (oder eine andere Form des geteilten Zugriffs, wenn `ConsentService` nicht `&mut self` für seine Methoden benötigt).
        - `ai_backend_port: Box<dyn AIBackendPort>` (Injizierter Port für die Kommunikation mit der KI-Infrastruktur).
        - `event_publisher: Box<dyn DomainEventPublisher>`.
    - **Methoden (Beispiele, stark abhängig von den konkreten KI-Features):**
        - `pub async fn suggest_window_layout(&self, user_id: &DomainId, current_windows: Vec<WindowHandle>, context: AIRequestContext) -> Result<AISuggestion<WindowLayoutSuggestion>, AIFeatureError>`
            - Prüft `self.consent_service.lock().unwrap().get_consent_status(user_id, "ai.window_organizer")`. Wenn nicht gewährt oder abgelaufen, gibt `AIFeatureError::ConsentNotGranted` zurück.
            - Bereitet die Anfrage für `ai_backend_port.request_window_layout_suggestion(...)` vor.
            - Verarbeitet die Antwort und gibt `AISuggestion` zurück.
            - Emittiert `AIFeatureEvent::SuggestionProvided`.
        - `pub async fn generate_smart_reply(&self, user_id: &DomainId, notification_content: String, context: AIRequestContext) -> Result<AISuggestion<SmartReplySuggestion>, AIFeatureError>`
            - Prüft `self.consent_service.lock().unwrap().get_consent_status(user_id, "ai.smart_reply")`.
            - Bereitet Anfrage für `ai_backend_port.request_smart_reply_suggestion(...)` vor.
            - Emittiert `AIFeatureEvent::SuggestionProvided`.
    - **Signale/Events (als Varianten von `AIFeatureEvent` im `DomainEvent` Enum):**
        - `SuggestionProvided { feature_id: String, user_id: DomainId, suggestion_id: DomainId, suggestion_payload_summary: String }` (Summary statt vollem Payload, um Eventgröße zu begrenzen)
        - `ActionTakenBasedOnAISuggestion { feature_id: String, user_id: DomainId, suggestion_id: DomainId, action_id: String }`
        - **Typische Publisher:** `AIFeatureService`.
        - **Typische Subscriber:** UI-Schicht (um Vorschläge anzuzeigen und Aktionen auszulösen), andere Domänendienste (um Aktionen basierend auf Vorschlägen auszuführen, z.B. `WorkspaceService` für Layout-Änderungen).
- **4.3.2. Trait: `AIBackendPort` (von Kernschicht zu implementieren)**
    
    - Definiert die Schnittstelle zur eigentlichen KI-Modellinteraktion.
    - `async fn request_window_layout_suggestion(&self, windows: Vec<WindowHandle>, context: AIRequestContext) -> Result<WindowLayoutSuggestion, DomainError>;`
    - `async fn request_smart_reply_suggestion(&self, text_to_reply_to: String, context: AIRequestContext) -> Result<SmartReplySuggestion, DomainError>;`
    - Weitere Methoden für andere KI-Features.
- **4.3.3. Datenstrukturen für KI-Anfragen/Antworten**
    
    - `AIRequestContext`: Enthält kontextuelle Daten, die für eine KI-Anfrage relevant sind und für die eine Einwilligung vorliegt.
        - **Attribute:** `source_application_id: Option<String>`, `current_activity_description: Option<String>`, `user_preferences: std::collections::HashMap<String, String>` (z.B. bevorzugte Sprache, Datenschutzeinstellungen für KI), `timestamp_ms: u64`.
        - **Rust-Definition:**
            
            Rust
            
            ```
            #
            pub struct AIRequestContext {
                pub source_application_id: Option<String>,
                pub current_activity_description: Option<String>,
                pub user_preferences: std::collections::HashMap<String, String>,
                pub timestamp_ms: u64,
            }
            ```
            
    - `AISuggestion<T>`: Generische Struktur für KI-Vorschläge.
        - **Attribute:** `suggestion_id: DomainId`, `feature_id: String`, `confidence_score: Option<f32>` (Wert zwischen 0.0 und 1.0), `payload: T`, `explanation: Option<String>`, `feedback_token: Option<String>` (Für implizites/explizites Feedback).
        - **Rust-Definition:**
            
            Rust
            
            ```
            #
            pub struct AISuggestion<T> {
                pub suggestion_id: DomainId,
                pub feature_id: String,
                pub confidence_score: Option<f32>,
                pub payload: T,
                pub explanation: Option<String>,
                pub feedback_token: Option<String>, // Token für Feedback-Mechanismen
            }
            ```
            
    - `WindowLayoutSuggestion`: Spezifischer Payload für Layout-Vorschläge.
        - **Attribute:** `suggested_space_id: Option<DomainId>` (Wenn ein spezifischer Space vorgeschlagen wird), `window_placements: Vec<(WindowHandle, SuggestedPlacement)>`.
        - **Rust-Definition:**
            
            Rust
            
            ```
            #
            pub struct WindowLayoutSuggestion {
                pub suggested_space_id: Option<DomainId>,
                pub window_placements: Vec<(WindowHandle, SuggestedPlacement)>,
            }
            ```
            
    - `SuggestedPlacement`: Details zur Platzierung eines Fensters.
        - **Attribute:** `target_space_id: Option<DomainId>` (Falls das Fenster in einen anderen Space verschoben werden soll), `relative_x: f32`, `relative_y: f32`, `relative_width: f32`, `relative_height: f32` (Werte zwischen 0.0 und 1.0, relativ zur Space-Größe), `stacking_order: Option<u32>`.
        - **Rust-Definition:**
            
            Rust
            
            ```
            #
            pub struct SuggestedPlacement {
                pub target_space_id: Option<DomainId>, // Wenn None, aktueller Space des Fensters
                pub relative_x: f32, // 0.0 bis 1.0
                pub relative_y: f32, // 0.0 bis 1.0
                pub relative_width: f32, // 0.0 bis 1.0
                pub relative_height: f32, // 0.0 bis 1.0
                pub stacking_order: Option<u32>, // z-Index
            }
            ```
            
    - `SmartReplySuggestion`: Spezifischer Payload für Antwortvorschläge.
        - **Attribute:** `suggested_replies: Vec<String>`.
        - **Rust-Definition:**
            
            Rust
            
            ```
            #
            pub struct SmartReplySuggestion {
                pub suggested_replies: Vec<String>,
            }
            ```
            
    - Alle diese Strukturen implementieren `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`.
- **4.3.4. Fehler-Enum: `AIFeatureError`**
    
    - Spezifische Fehler für das `feature_service`-Modul.
    - **Varianten:**
        - `#[error("Consent not granted for user '{user_id}' and feature '{feature_id}'.")]` `ConsentNotGranted { user_id: DomainId, feature_id: String }` (Kann `#[from] AIConsentError` nutzen)
        - `#[error("Error interacting with AI model/backend: {message}")]` `ModelInteractionError { message: String }` (Typischerweise von `AIBackendPort` propagiert)
        - `#[error("Invalid or insufficient context provided for AI feature '{feature_id}': {reason}")]` `InvalidContext { feature_id: String, reason: String }`
        - `#[error("No suggestion available for feature '{feature_id}' with the given context.")]` `SuggestionNotAvailable { feature_id: String }`
        - `#` `BackendPortError(String)` // Generischer Fehler vom Port
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum AIFeatureError {
            #[error("Consent not granted for user '{user_id}' and feature '{feature_id}'.")]
            ConsentNotGranted { user_id: DomainId, feature_id: String },
            #[error("Error interacting with AI model/backend: {message}")]
            ModelInteractionError { message: String },
            #[error("Invalid or insufficient context provided for AI feature '{feature_id}': {reason}")]
            InvalidContext { feature_id: String, reason: String },
            #[error("No suggestion available for feature '{feature_id}' with the given context.")]
            SuggestionNotAvailable { feature_id: String },
            #
            BackendPortError(String),
        }
        
        // Mögliche Konvertierung von AIConsentError
        impl From<AIConsentError> for AIFeatureError {
            fn from(err: AIConsentError) -> Self {
                match err {
                    AIConsentError::ConsentNotFoundOrNotGranted { user_id, feature_id } => {
                        AIFeatureError::ConsentNotGranted { user_id, feature_id }
                    }
                    // Andere Mappings oder ein generischer Fehler
                    _ => AIFeatureError::ModelInteractionError {
                        message: format!("Consent error: {}", err),
                    },
                }
            }
        }
        ```
        

### 4.4. Geschäftsregeln

- **Strikte Einwilligungsprüfung:** Vor jeder Ausführung einer KI-Funktion oder jedem Zugriff auf potenziell sensible Daten durch eine KI-Funktion _muss_ eine gültige, nicht abgelaufene Einwilligung des Benutzers für das spezifische Feature und den spezifischen Datenumfang vorliegen. Dies wird durch den `AIFeatureService` sichergestellt, der den `ConsentService` konsultiert.
- **Datenminimierung und -relevanz:** An KI-Modelle (über den `AIBackendPort`) werden nur die Daten gesendet, die für die jeweilige Funktion unbedingt notwendig sind und für die eine Einwilligung vorliegt. Die Domänenschicht definiert die Struktur dieser Daten (`AIRequestContext`).
- **Anonymisierung/Pseudonymisierung:** Falls von der Kernschicht (Implementierung des `AIBackendPort`) unterstützt, kann die Domänenschicht anfordern, dass Daten vor der Übermittlung an externe KI-Dienste anonymisiert oder pseudonymisiert werden. Die Domänenschicht selbst führt diese Operationen nicht durch, sondern spezifiziert die Notwendigkeit.
- **Fallback-Verhalten:** Wenn KI-Dienste nicht verfügbar sind (Fehler vom `AIBackendPort`) oder keine sinnvollen Vorschläge liefern (`SuggestionNotAvailable`), muss die Anwendung ein definiertes Fallback-Verhalten zeigen (z.B. Standardfunktionalität ohne KI anbieten, Fehlermeldung anzeigen). Dies wird vom Aufrufer des `AIFeatureService` gehandhabt.
- **Transparenz:** Dem Benutzer sollte (über die UI-Schicht) nachvollziehbar gemacht werden, wann und warum eine KI-Funktion aktiv wird und welche Daten dafür verwendet wurden (z.B. durch `AISuggestion::explanation`).

Die Tabellen fassen die Kernkomponenten des KI-Managements zusammen:

**Tabelle 4.2.1: Entität `UserConsent` - Attribute**

|   |   |   |   |
|---|---|---|---|
|**Attribut**|**Typ**|**Sichtbarkeit**|**Invarianten/Initialwert**|
|`user_id`|`DomainId`|`pub`|Nicht leer|
|`feature_id`|`String`|`pub`|Nicht leer|
|`is_granted`|`bool`|`pub`|Initial `false`|
|`last_updated_timestamp`|`u64`|`pub(crate)`|Aktueller Zeitstempel|
|`scope`|`Option<String>`|`pub`||
|`expires_at`|`Option<u64>`|`pub`||

**Tabelle 4.2.2: `ConsentService` - Öffentliche API (Auswahl)**

|   |   |   |   |
|---|---|---|---|
|**Methode**|**Parameter**|**Rückgabetyp**|**Kurzbeschreibung**|
|`grant_consent`|`user_id: DomainId`, `feature_id: String`, `scope: Option<String>`, `expires_at: Option<u64>`|`Result<(), AIConsentError>`|Erteilt eine Einwilligung.|
|`revoke_consent`|`user_id: &DomainId`, `feature_id: &str`|`Result<(), AIConsentError>`|Widerruft eine Einwilligung.|
|`get_consent_status`|`user_id: &DomainId`, `feature_id: &str`|`Result<&UserConsent, AIConsentError>`|Prüft den aktuellen Einwilligungsstatus.|
|`list_consents_for_user`|`user_id: &DomainId`|`Vec<&UserConsent>`|Listet alle Einwilligungen eines Benutzers.|

**Tabelle 4.3.1: `AIFeatureService` - Beispielhafte Öffentliche API**

|   |   |   |   |
|---|---|---|---|
|**Methode**|**Parameter**|**Rückgabetyp**|**Kurzbeschreibung**|
|`suggest_window_layout`|`user_id: &DomainId`, `windows: Vec<WindowHandle>`, `context: AIRequestContext`|`Result<AISuggestion<WindowLayoutSuggestion>, AIFeatureError>`|Schlägt ein Fensterlayout vor.|
|`generate_smart_reply`|`user_id: &DomainId`, `notification_content: String`, `context: AIRequestContext`|`Result<AISuggestion<SmartReplySuggestion>, AIFeatureError>`|Generiert Antwortvorschläge.|

---

## 5. Modul: Benachrichtigungsverwaltung (`notification_manager`)

### 5.1. Übersicht und Verantwortlichkeiten

Das `notification_manager`-Modul ist für die Entgegennahme, Verwaltung und (logische) Anzeige von Benachrichtigungen zuständig, die von Anwendungen und Systemkomponenten stammen. Es unterstützt interaktive Aktionen innerhalb von Benachrichtigungen, ermöglicht deren Priorisierung und Deduplizierung und stellt eine Historie vergangener Benachrichtigungen bereit.

Die Domänenschicht definiert hierbei die _Struktur_ und die _Logik_ von Benachrichtigungen. Die tatsächliche visuelle Darstellung, beispielsweise als Pop-up-Fenster oder Eintrag in einer Benachrichtigungszentrale, ist Aufgabe der UI-Schicht. Die Notification API des XDG Desktop Portals 35 dient als gute Inspiration für die Definition der Felder einer Benachrichtigung, wie ID, Titel, Textkörper, Priorität und mögliche Aktionen. Die Domänenschicht verwaltet Benachrichtigungen als Datenobjekte. Essentielle Felder sind `title`, `body` und `priority`. Darüber hinaus sind `application_name` (als Quelle der Benachrichtigung) und `actions` (zur Ermöglichung von Interaktivität) wichtig. Die UI-Schicht konsumiert diese Datenobjekte und erzeugt daraus die entsprechende visuelle Repräsentation.

### 5.2. Entitäten, Wertobjekte und Enums

- **5.2.1. Entität: `Notification`**
    
    - Repräsentiert eine einzelne Benachrichtigung.
    - **Attribute:**
        - `id: DomainId` (Eindeutige ID der Benachrichtigung, z.B. generiert via `uuid::Uuid::new_v4().to_string()`). Sichtbarkeit: `pub`. Invarianten: Eindeutig, nicht leer.
        - `application_name: String` (Name der sendenden Anwendung/Komponente). Sichtbarkeit: `pub`. Invarianten: Nicht leer.
        - `application_icon: Option<String>` (Name oder Pfad zu einem Icon, das von der UI-Schicht interpretiert wird). Sichtbarkeit: `pub`.
        - `summary: String` (Titel/Zusammenfassung der Benachrichtigung). Sichtbarkeit: `pub`. Invarianten: Nicht leer.
        - `body: Option<String>` (Detaillierter Text der Benachrichtigung). Sichtbarkeit: `pub`.
        - `actions: Vec<NotificationAction>` (Liste möglicher Aktionen, die der Benutzer ausführen kann). Sichtbarkeit: `pub`.
        - `urgency: NotificationUrgency` (Dringlichkeit der Benachrichtigung). Sichtbarkeit: `pub`. Initialwert: `NotificationUrgency::Normal`.
        - `category: Option<String>` (Kategorie zur Filterung/Gruppierung, z.B. "email.new", "chat.message", "system.update.available"). Sichtbarkeit: `pub`.
        - `timestamp_ms: u64` (Unix-Timestamp in Millisekunden der Erstellung). Sichtbarkeit: `pub(crate)`.
        - `expires_timeout_ms: Option<u32>` (Zeit in Millisekunden, nach der die Benachrichtigung automatisch geschlossen wird; `0` oder `None` bedeutet, sie läuft nicht automatisch ab). Sichtbarkeit: `pub`.
        - `is_persistent: bool` (Ob die Benachrichtigung in der Historie verbleibt, auch nachdem sie geschlossen wurde). Sichtbarkeit: `pub`. Initialwert: `true`.
        - `resident: bool` (Ob die Benachrichtigung permanent sichtbar bleiben soll, bis sie explizit geschlossen wird – ähnlich "sticky" Notifications; Freedesktop-Spezifikation "resident"). Sichtbarkeit: `pub`. Initialwert: `false`.
        - `transient: bool` (Ob die Benachrichtigung nur kurz angezeigt und nicht in der Historie gespeichert werden soll, auch wenn `is_persistent` true wäre; Freedesktop-Spezifikation "transient"). Sichtbarkeit: `pub`. Initialwert: `false`.
        - `custom_data: std::collections::HashMap<String, String>` (Für anwendungsspezifische Daten, die von der sendenden Anwendung mitgegeben werden können). Sichtbarkeit: `pub`.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub struct Notification {
            pub id: DomainId,
            pub application_name: String,
            pub application_icon: Option<String>,
            pub summary: String,
            pub body: Option<String>,
            pub actions: Vec<NotificationAction>,
            pub urgency: NotificationUrgency,
            pub category: Option<String>,
            #[serde(default = "crate::ai_manager::consent_manager::current_timestamp_ms")] // Wiederverwendung der Helferfunktion
            pub(crate) timestamp_ms: u64,
            pub expires_timeout_ms: Option<u32>,
            #[serde(default = "default_true")]
            pub is_persistent: bool,
            #[serde(default)]
            pub resident: bool,
            #[serde(default)]
            pub transient: bool,
            #[serde(default)]
            pub custom_data: std::collections::HashMap<String, String>,
        }
        
        fn default_true() -> bool { true }
        // fn current_timestamp_ms() -> u64 {... } // Siehe oben
        ```
        
- **5.2.2. Wertobjekt: `NotificationAction`**
    
    - Definiert eine Aktion, die im Kontext einer Benachrichtigung ausgeführt werden kann.
    - **Attribute:**
        - `key: String` (Eindeutiger Schlüssel für die Aktion innerhalb der Benachrichtigung, z.B. "reply", "archive", "mark-as-read"). Sichtbarkeit: `pub`. Invarianten: Nicht leer.
        - `label: String` (Anzeigetext für den Button in der UI). Sichtbarkeit: `pub`. Invarianten: Nicht leer.
    - **Implementiert:** `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub struct NotificationAction {
            pub key: String,
            pub label: String,
        }
        ```
        
- **5.2.3. Enum: `NotificationUrgency`**
    
    - Definiert die Dringlichkeitsstufe einer Benachrichtigung, inspiriert von der Freedesktop Notification Specification.
    - **Varianten:** `Low`, `Normal`, `Critical`.
    - **Implementiert:** `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`, `Default` (`Normal`).
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum NotificationUrgency {
            Low,
            #[default]
            Normal,
            Critical,
        }
        ```
        
- **5.2.4. Fehler-Enum: `NotificationError`**
    
    - Spezifische Fehler für das `notification_manager`-Modul. Definiert mit `thiserror`.
    - **Varianten:**
        - `#` `NotificationNotFound { notification_id: DomainId }`
        - `#[error("Action with key '{action_key}' not found for notification '{notification_id}'.")]` `ActionNotFound { notification_id: DomainId, action_key: String }`
        - `#[error("Invalid notification data provided: {reason}")]` `InvalidNotificationData { reason: String }`
        - `#[error("Notification history is full. Maximum size: {max_size}.")]` `HistoryFull { max_size: usize }`
        - `#[error("Maximum number of active notifications reached: {max_active}.")]` `MaxActiveNotificationsReached { max_active: usize }`
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum NotificationError {
            #
            NotificationNotFound { notification_id: DomainId },
            #[error("Action with key '{action_key}' not found for notification '{notification_id}'.")]
            ActionNotFound { notification_id: DomainId, action_key: String },
            #[error("Invalid notification data provided: {reason}")]
            InvalidNotificationData { reason: String },
            #[error("Notification history is full. Maximum size: {max_size}.")]
            HistoryFull { max_size: usize },
            #[error("Maximum number of active notifications reached: {max_active}.")]
            MaxActiveNotificationsReached { max_active: usize },
        }
        ```
        

### 5.3. Service: `NotificationService`

Der `NotificationService` hält den Zustand aller aktiven und ggf. historischen Benachrichtigungen und stellt Methoden zu deren Verwaltung bereit.

- **5.3.1. Eigenschaften (Interner Zustand)**
    
    - `active_notifications: std::collections::VecDeque<Notification>`: Eine Queue für aktive, potenziell sichtbare Benachrichtigungen. `VecDeque` ermöglicht effizientes FIFO-Verhalten, wenn `max_active_notifications` erreicht ist.
    - `notification_history: std::collections::VecDeque<Notification>`: Eine Queue für die Historie geschlossener, persistenter Benachrichtigungen, begrenzt durch `max_history_size`.
    - `next_internal_id_counter: u64`: Ein interner Zähler zur Generierung sequenzieller Teile von IDs, falls UUIDs nicht allein verwendet werden oder zur Deduplizierung.
    - `max_active_notifications: usize`: Konfigurierbare maximale Anzahl aktiver Benachrichtigungen (z.B. Default 5).
    - `max_history_size: usize`: Konfigurierbare maximale Größe der Historie (z.B. Default 100).
    - `persistence_port: Box<dyn NotificationPersistencePort>`: Injizierter Port für Persistenz der Historie.
    - `event_publisher: Box<dyn DomainEventPublisher>`.
    - `clock: Box<dyn crate::ai_manager::consent_manager::Clock>`. // Wiederverwendung des Clock-Traits
- **5.3.2. Methoden (Öffentliche API)**
    
    - `pub fn new(max_active: usize, max_history: usize, persistence_port: Box<dyn NotificationPersistencePort>, event_publisher: Box<dyn DomainEventPublisher>, clock: Box<dyn crate::ai_manager::consent_manager::Clock>) -> Self`
        - Konstruktor. Lädt ggf. die Historie über den `persistence_port`.
    - `pub fn post_notification(&mut self, app_name: String, app_icon: Option<String>, summary: String, body: Option<String>, actions: Vec<NotificationAction>, urgency: NotificationUrgency, category: Option<String>, expires_ms: Option<u32>, persistent: bool, resident: bool, transient: bool, custom_data: std::collections::HashMap<String, String>) -> Result<DomainId, NotificationError>`
        - **Validierung:** Prüft, ob `app_name` und `summary` nicht leer sind (`NotificationError::InvalidNotificationData`).
        - **ID-Generierung:** Erzeugt eine eindeutige `DomainId` (z.B. `uuid::Uuid::new_v4().to_string()`).
        - **Timestamp:** Setzt `timestamp_ms` mittels `self.clock.current_timestamp_ms()`.
        - **Erstellung:** Erstellt das `Notification`-Objekt.
        - **Deduplizierung (Optional):** Implementiert Logik, um Duplikate zu erkennen und ggf. zu ersetzen oder zu ignorieren. (Für diese Spezifikation vorerst nicht detailliert).
        - **Aktive Liste:** Wenn `active_notifications.len() >= self.max_active_notifications`, wird die älteste Benachrichtigung entfernt (und ggf. in die Historie verschoben, falls `is_persistent` und nicht `transient`).
        - Fügt die neue Benachrichtigung zu `active_notifications` hinzu.
        - **Emittiert:** `NotificationEvent::NotificationPosted { notification: new_notification.clone() }`.
        - **Rückgabe:** Die ID der neuen Benachrichtigung.
    - `pub fn close_notification(&mut self, notification_id: &DomainId, reason: NotificationCloseReason) -> Result<(), NotificationError>`
        - Sucht die Benachrichtigung in `active_notifications`. Wenn nicht gefunden, `NotificationError::NotificationNotFound`.
        - Entfernt die Benachrichtigung aus `active_notifications`.
        - **Historie:** Wenn die Benachrichtigung `is_persistent` ist und nicht `transient`, wird sie zu `notification_history` hinzugefügt. Wenn `notification_history.len() >= self.max_history_size`, wird die älteste Benachrichtigung aus der Historie entfernt.
        - **Persistenz:** Speichert die aktualisierte Historie über `persistence_port.save_history()`.
        - **Emittiert:** `NotificationEvent::NotificationClosed { notification_id: notification_id.clone(), reason }`.
    - `pub fn trigger_action(&mut self, notification_id: &DomainId, action_key: &str) -> Result<(), NotificationError>`
        - Sucht die Benachrichtigung in `active_notifications`. Wenn nicht gefunden, `NotificationError::NotificationNotFound`.
        - Sucht die Aktion mit `action_key` in `notification.actions`. Wenn nicht gefunden, `NotificationError::ActionNotFound`.
        - **Emittiert:** `NotificationEvent::NotificationActionTriggered { notification_id: notification_id.clone(), action_key: action_key.to_string() }`.
        - Schließt typischerweise die Benachrichtigung danach: `self.close_notification(notification_id, NotificationCloseReason::ActionTaken)?`.
    - `pub fn get_active_notifications(&self) -> Vec<&Notification>`
        - Gibt eine Kopie der aktiven Benachrichtigungen als Slice oder Vec von Referenzen zurück.
    - `pub fn get_notification_history(&self) -> Vec<&Notification>`
        - Gibt eine Kopie der Benachrichtigungshistorie zurück.
    - `pub fn clear_history(&mut self) -> Result<(), NotificationError>`
        - Leert `notification_history`.
        - Speichert die leere Historie über `persistence_port.save_history()`.
        - **Emittiert:** `NotificationEvent::NotificationHistoryCleared`.
    - `pub fn get_notification_by_id(&self, notification_id: &DomainId) -> Option<&Notification>`
        - Sucht eine Benachrichtigung zuerst in `active_notifications`, dann in `notification_history`.
- **5.3.3. Signale/Events (als Varianten von `NotificationEvent` im `DomainEvent` Enum)**
    
    - `NotificationPosted { notification: Notification }`
    - `NotificationClosed { notification_id: DomainId, reason: NotificationCloseReason }`
    - `NotificationActionTriggered { notification_id: DomainId, action_key: String }`
    - `NotificationHistoryCleared`
    - `NotificationUpdated { notification: Notification }` (Falls Benachrichtigungen nach dem Posten noch modifizierbar sein sollen, z.B. Fortschrittsbalken. Für diese Spezifikation vorerst nicht im Fokus.)
    - **Typische Publisher:** `NotificationService`.
    - **Typische Subscriber:** UI-Schicht (zur Anzeige/Aktualisierung von Benachrichtigungen und der Historie), `AIFeatureService` (z.B. um auf neue Benachrichtigungen zu reagieren und Smart Replies vorzuschlagen).
- **5.3.4. Enum: `NotificationCloseReason`**
    
    - Gibt den Grund an, warum eine Benachrichtigung geschlossen wurde.
    - **Varianten:** `Expired` (Timeout erreicht), `DismissedByUser` (Benutzer hat sie aktiv geschlossen), `ActionTaken` (Eine Aktion wurde ausgeführt), `ProgrammaticallyClosed` (Durch die Anwendung/System geschlossen), `SourceClosed` (Die sendende Anwendung hat das Schließen angefordert).
    - **Implementiert:** `serde::Serialize`, `serde::Deserialize`, `Debug`, `Clone`, `PartialEq`, `Eq`.
    - **Rust-Definition:**
        
        Rust
        
        ```
        #
        pub enum NotificationCloseReason {
            Expired,
            DismissedByUser,
            ActionTaken,
            ProgrammaticallyClosed,
            SourceClosed, // z.B. wenn die App die Notification zurückzieht
        }
        ```
        
- **5.3.5. Trait: `NotificationPersistencePort` (von Kernschicht zu implementieren)**
    
    - `fn load_history(&self) -> Result<Vec<Notification>, DomainError>;`
    - `fn save_history(&self, history: &std::collections::VecDeque<Notification>) -> Result<(), DomainError>;`

### 5.4. Geschäftsregeln

- **Priorisierung:** Kritische Benachrichtigungen (`NotificationUrgency::Critical`) könnten an der Spitze der `active_notifications`-Queue eingefügt werden oder andere weniger wichtige Benachrichtigungen verdrängen, falls `max_active_notifications` erreicht ist. Normale und niedrige Dringlichkeiten werden am Ende der Queue hinzugefügt.
- **Deduplizierung:** (Optional, für spätere Erweiterung) Regeln, um identische oder sehr ähnliche Benachrichtigungen (z.B. gleicher `application_name`, `summary` und `category` innerhalb eines kurzen Zeitfensters) zusammenzufassen oder zu unterdrücken. Dies könnte durch einen Hash über relevante Felder oder eine "replaces_id"-Mechanik implementiert werden.
- **Maximale Anzahl aktiver Benachrichtigungen:** Wenn `max_active_notifications` überschritten wird, wird die älteste nicht-residente Benachrichtigung geschlossen (Grund: `ProgrammaticallyClosed`) und ggf. in die Historie verschoben.
- **Maximale Größe der Historie:** Wenn `max_history_size` beim Hinzufügen einer Benachrichtigung zur Historie überschritten wird, wird der älteste Eintrag aus der Historie entfernt.
- **Verhalten bei `expires_timeout_ms`:** Ein Mechanismus (z.B. ein Timer-Service in der Kernschicht, der vom `NotificationService` über den `event_publisher` oder einen dedizierten Port gesteuert wird) muss dafür sorgen, dass Benachrichtigungen mit `expires_timeout_ms` nach Ablauf der Zeit mit `NotificationCloseReason::Expired` geschlossen werden. Die Domänenschicht selbst verwaltet keine aktiven Timer.
- **`transient` vs. `is_persistent`:** Eine als `transient` markierte Benachrichtigung wird niemals in die Historie aufgenommen, unabhängig vom Wert von `is_persistent`.

Die folgenden Tabellen bieten eine Übersicht über die `Notification`-Entität und die API des `NotificationService`.

**Tabelle 5.2.1: Entität `Notification` - Attribute**

|   |   |   |   |
|---|---|---|---|
|**Attribut**|**Typ**|**Sichtbarkeit**|**Invarianten/Initialwert (Beispiel)**|
|`id`|`DomainId`|`pub`|Eindeutig, nicht leer|
|`application_name`|`String`|`pub`|Nicht leer|
|`application_icon`|`Option<String>`|`pub`||
|`summary`|`String`|`pub`|Nicht leer|
|`body`|`Option<String>`|`pub`||
|`actions`|`Vec<NotificationAction>`|`pub`||
|`urgency`|`NotificationUrgency`|`pub`|`Normal`|
|`category`|`Option<String>`|`pub`||
|`timestamp_ms`|`u64`|`pub(crate)`|Zeitstempel bei Erstellung|
|`expires_timeout_ms`|`Option<u32>`|`pub`|`None` (läuft nicht ab)|
|`is_persistent`|`bool`|`pub`|`true`|
|`resident`|`bool`|`pub`|`false`|
|`transient`|`bool`|`pub`|`false`|
|`custom_data`|`std::collections::HashMap<String, String>`|`pub`|Leer|

**Tabelle 5.3.2: `NotificationService` - Öffentliche API (Auswahl)**

|   |   |   |   |
|---|---|---|---|
|**Methode**|**Parameter (Auszug)**|**Rückgabetyp**|**Kurzbeschreibung**|
|`post_notification`|`app_name: String`, `summary: String`, `urgency: NotificationUrgency`,...|`Result<DomainId, NotificationError>`|Postet eine neue Benachrichtigung.|
|`close_notification`|`notification_id: &DomainId`, `reason: NotificationCloseReason`|`Result<(), NotificationError>`|Schließt eine aktive Benachrichtigung.|
|`trigger_action`|`notification_id: &DomainId`, `action_key: &str`|`Result<(), NotificationError>`|Löst eine Aktion einer Benachrichtigung aus.|
|`get_active_notifications`||`Vec<&Notification>`|Ruft alle aktiven Benachrichtigungen ab.|
|`get_notification_history`||`Vec<&Notification>`|Ruft die Historie der Benachrichtigungen ab.|
|`clear_history`||`Result<(), NotificationError>`|Leert die Benachrichtigungshistorie.|

---

## 6. Modul: Fenstermanagement-Richtlinien (`window_policy_engine`)

### 6.1. Übersicht und Verantwortlichkeiten

Das Modul `window_policy_engine` ist dafür zuständig, Regeln für das Verhalten von Fenstern zu definieren und anzuwenden. Diese Regeln können beispielsweise die automatische Zuweisung von Fenstern zu bestimmten Spaces, Standard-Tiling-Verhalten für spezifische Anwendungen oder andere Aspekte des Fensterverhaltens umfassen. Dieses Modul entkoppelt spezifische Fensterverwaltungsentscheidungen von der allgemeinen Workspace-Verwaltung im `WorkspaceManager`. Es reagiert auf Domänenereignisse wie "Fenster geöffnet" (signalisiert von der Systemschicht über die Kernschicht und dann als Domänenereignis weitergeleitet) oder "Space gewechselt".

Die Verwendung eines solchen Policy-Engines ermöglicht eine hohe Anpassbarkeit des Fenstermanagements, ohne die Kernlogik des `WorkspaceManager` zu verändern. Die Regeln werden als Datenstrukturen repräsentiert und können potenziell zur Laufzeit modifiziert werden (z.B. durch Benutzereingaben in einer Konfigurations-UI). Inspiration für die Definition von Fenstereigenschaften und -zuständen kann von Wayland-Protokollen wie `wlr-foreign-toplevel-management-unstable-v1` 36 oder `xdg-shell` und dessen Erweiterungen (z.B. `xdg-decoration` 40) abgeleitet werden. Konzepte wie `app_id`, `title`, `maximized`, `minimized`, `fullscreen` werden von der Domänenschicht jedoch abstrakt implementiert, ohne direkte Abhängigkeiten zu diesen Protokollen. Die Domänenschicht definiert, welche Informationen sie über ein Fenster benötigt (`WindowStateContext`), und die Systemschicht ist verantwortlich, diese Informationen aus dem jeweiligen Fenstersystem (Wayland, X11) zu extrahieren und bereitzustellen. Die Aktionen, die aus den Regeln resultieren (z.


# **Domänenschicht: Theming-Engine – Ultra-Feinspezifikation (Teil 1/4)**

## **1\. Einleitung zum Modul domain::theming**

Das Modul domain::theming ist eine Kernkomponente der Domänenschicht und trägt die Verantwortung für die gesamte Logik des Erscheinungsbilds (Theming) der Desktop-Umgebung. Seine Hauptaufgabe besteht darin, Design-Tokens zu verwalten, Theme-Definitionen zu interpretieren, Benutzereinstellungen für das Theming zu berücksichtigen und den finalen, aufgelösten Theme-Zustand für die Benutzeroberflächenschicht bereitzustellen. Dieses Modul ermöglicht dynamische Theme-Wechsel zur Laufzeit, einschließlich Änderungen des Farbschemas (Hell/Dunkel) und der Akzentfarben, basierend auf einem robusten, Token-basierten System. Es ist so konzipiert, dass es unabhängig von spezifischen UI-Toolkits oder Systemdetails agiert und eine klare Trennung zwischen der Logik des Erscheinungsbilds und dessen Darstellung gewährleistet. Diese Spezifikation dient als direkter Implementierungsleitfaden für Entwickler.

## **2\. Datenstrukturen (domain::theming::types)**

Die folgenden Datenstrukturen definieren die Entitäten und Wertobjekte, die für die Verwaltung und Anwendung von Themes und Design-Tokens notwendig sind. Sie sind für die Serialisierung und Deserialisierung mittels serde vorbereitet, um das Laden von Konfigurationen und Definitionen aus Dateien (z.B. JSON) zu ermöglichen.

### **2.1. Token-bezogene Datenstrukturen**

Diese Strukturen repräsentieren einzelne Design-Tokens und deren Werte.

* TokenIdentifier (Wertobjekt):  
  Ein eindeutiger, hierarchischer Bezeichner für ein Design-Token (z.B. "color.background.primary", "font.size.default"). Die hierarchische Struktur erleichtert die Organisation und das Verständnis der Tokens.  
  Rust  
  \#  
  pub struct TokenIdentifier(String);

  impl TokenIdentifier {  
      pub fn new(id: impl Into\<String\>) \-\> Self {  
          Self(id.into())  
      }  
      pub fn as\_str(\&self) \-\> \&str {  
          \&self.0  
      }  
  }

  impl std::fmt::Display for TokenIdentifier {  
      fn fmt(\&self, f: \&mut std::fmt::Formatter\<'\_\>) \-\> std::fmt::Result {  
          write\!(f, "{}", self.0)  
      }  
  }

* TokenValue (Enum):  
  Repräsentiert die möglichen Wertetypen eines Design-Tokens. Die String-Werte für Farben, Dimensionen etc. sind so gestaltet, dass sie direkt CSS-kompatibel sind. Die Variante Reference ermöglicht die Erstellung von Alias-Tokens, die auf andere Tokens verweisen, was die Wiederverwendbarkeit und Konsistenz fördert.  
  Rust  
  \#  
  \#\[serde(rename\_all \= "kebab-case")\]  
  pub enum TokenValue {  
      Color(String),      // z.B., "\#FF0000", "rgba(255,0,0,0.5)", "transparent"  
      Dimension(String),  // z.B., "16px", "2rem", "100%"  
      FontSize(String),   // z.B., "12pt", "1.5em"  
      FontFamily(String), // z.B., "Inter, sans-serif"  
      FontWeight(String), // z.B., "normal", "bold", "700"  
      LineHeight(String), // z.B., "1.5", "150%"  
      LetterSpacing(String),// z.B., "0.5px", "0.05em"  
      Border(String),     // z.B., "1px solid \#CCCCCC"  
      Shadow(String),     // z.B., "2px 2px 5px rgba(0,0,0,0.3)"  
      Radius(String),     // z.B., "4px", "50%"  
      Spacing(String),    // z.B., "8px" (generische Abstände für padding, margin)  
      ZIndex(i32),  
      Opacity(f64),       // 0.0 bis 1.0  
      Text(String),       // Für beliebige String-Werte  
      Reference(TokenIdentifier), // Alias zu einem anderen Token  
  }

* RawToken (Struct):  
  Repräsentiert ein einzelnes Design-Token, wie es typischerweise aus einer Konfigurationsdatei (z.B. JSON) geladen wird. Enthält den Identifikator, den Wert und optionale Metadaten wie Beschreibung und Gruppierung.  
  Rust  
  \#  
  pub struct RawToken {  
      pub id: TokenIdentifier,  
      pub value: TokenValue,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub description: Option\<String\>,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub group: Option\<String\>, // z.B., "colors", "spacing", "typography"  
  }

* TokenSet (Typalias):  
  Eine Sammlung von RawTokens, die für eine effiziente Suche und Verwaltung als HashMap implementiert ist, wobei der TokenIdentifier als Schlüssel dient.  
  Rust  
  pub type TokenSet \= std::collections::HashMap\<TokenIdentifier, RawToken\>;

### **2.2. Theme-Definitionsstrukturen**

Diese Strukturen definieren ein vollständiges Theme, seine Varianten (z.B. Hell/Dunkel) und unterstützte Anpassungen.

* ThemeIdentifier (Wertobjekt):  
  Ein eindeutiger Bezeichner für ein Theme (z.B. "adwaita-ng", "material-you-like").  
  Rust  
  \#  
  pub struct ThemeIdentifier(String);

  impl ThemeIdentifier {  
      pub fn new(id: impl Into\<String\>) \-\> Self {  
          Self(id.into())  
      }  
      pub fn as\_str(\&self) \-\> \&str {  
          \&self.0  
      }  
  }  
  impl std::fmt::Display for ThemeIdentifier {  
      fn fmt(\&self, f: \&mut std::fmt::Formatter\<'\_\>) \-\> std::fmt::Result {  
          write\!(f, "{}", self.0)  
      }  
  }

* ColorSchemeType (Enum):  
  Definiert die grundlegenden Farbschemata, die ein Theme unterstützen kann.  
  Rust  
  \#  
  pub enum ColorSchemeType {  
      Light,  
      Dark,  
  }

* AccentColor (Struct / Wertobjekt):  
  Repräsentiert eine Akzentfarbe, die entweder einen vordefinierten Namen oder einen direkten Farbwert haben kann.  
  Rust  
  \#  
  pub struct AccentColor {  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub name: Option\<String\>, // z.B., "Blue", "ForestGreen"  
      pub value: String,        // z.B., "\#3498db" (tatsächlicher CSS-Farbwert)  
  }

* ThemeVariantDefinition (Struct):  
  Definiert die spezifischen Token-Werte oder Überschreibungen für eine bestimmte Variante eines Themes (z.B. das Dunkel-Schema). Der TokenSet hier enthält nur die Tokens, die sich von den base\_tokens des Themes unterscheiden oder spezifisch für diese Variante sind.  
  Rust  
  \#  
  pub struct ThemeVariantDefinition {  
      pub applies\_to\_scheme: ColorSchemeType,  
      pub tokens: TokenSet, // Token-Überschreibungen oder spezifische Definitionen für diese Variante  
  }

* ThemeDefinition (Struct):  
  Die vollständige Definition eines Themes, inklusive Metadaten, Basis-Tokens, Varianten und unterstützten Akzentfarben.  
  Rust  
  \#  
  pub struct ThemeDefinition {  
      pub id: ThemeIdentifier,  
      pub name: String, // Anzeigename, z.B. "Adwaita Next Generation"  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub description: Option\<String\>,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub author: Option\<String\>,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub version: Option\<String\>,  
      pub base\_tokens: TokenSet, // Grundlegende Tokens, die für alle Varianten gelten  
      \#\[serde(default, skip\_serializing\_if \= "Vec::is\_empty")\]  
      pub variants: Vec\<ThemeVariantDefinition\>, // Definitionen für Hell, Dunkel etc.  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub supported\_accent\_colors: Option\<Vec\<AccentColor\>\>, // Vordefinierte Akzentfarben  
  }

### **2.3. Konfigurations- und Zustandsstrukturen**

Diese Strukturen repräsentieren die vom Benutzer gewählten Theming-Einstellungen und den daraus resultierenden, angewendeten Theme-Zustand.

* AppliedThemeState (Struct):  
  Repräsentiert den aktuell im System aktiven Theme-Zustand. Entscheidend ist hier das Feld resolved\_tokens, welches alle Design-Tokens auf ihre endgültigen, CSS-kompatiblen String-Werte abbildet. Diese Struktur ist das primäre Ergebnis der Theming-Logik und wird von der UI-Schicht konsumiert.  
  Eine wichtige Invariante ist, dass resolved\_tokens keine TokenValue::Reference mehr enthalten darf; alle Werte müssen endgültig aufgelöst sein.  
  Rust  
  \# // Deserialize ist hier nicht zwingend nötig  
  pub struct AppliedThemeState {  
      pub theme\_id: ThemeIdentifier,  
      pub color\_scheme: ColorSchemeType,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub active\_accent\_color: Option\<AccentColor\>,  
      // Schlüssel: TokenIdentifier (z.B., "color.background.default")  
      // Wert: Final aufgelöster CSS-String (z.B., "\#FFFFFF")  
      pub resolved\_tokens: std::collections::HashMap\<TokenIdentifier, String\>,  
  }

* ThemingConfiguration (Struct):  
  Speichert die benutzerspezifischen Einstellungen für das Theming. Diese Konfiguration wird typischerweise von einer übergeordneten Einstellungsverwaltung (domain::settings) bereitgestellt und dient als Eingabe für die ThemingEngine. Sie ermöglicht es Benutzern, ihr bevorzugtes Theme, Farbschema, Akzentfarbe und sogar einzelne Tokens global zu überschreiben.  
  Rust  
  \#  
  pub struct ThemingConfiguration {  
      pub selected\_theme\_id: ThemeIdentifier,  
      pub preferred\_color\_scheme: ColorSchemeType, // Präferenz des Benutzers  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub selected\_accent\_color: Option\<AccentColor\>,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      // Ermöglicht Power-Usern, spezifische Tokens für jedes Theme zu überschreiben  
      pub custom\_user\_token\_overrides: Option\<TokenSet\>,  
  }

### **2.4. Tabellen für Datenstrukturen**

Die folgenden Tabellen fassen die Schlüsseleigenschaften der wichtigsten Datenstrukturen zusammen und dienen als schnelle Referenz für Entwickler. Sie verdeutlichen die Struktur und die Bedeutung der einzelnen Felder, was für die korrekte Implementierung und Nutzung dieser Typen unerlässlich ist. Die explizite Angabe von serde-Attributen und abgeleiteten Traits stellt sicher, dass die Strukturen direkt für die Datenpersistenz und den internen Gebrauch geeignet sind.

* **Tabelle 2.1: RawToken Felder**

| Feldname | Rust-Typ | Sichtbarkeit | Initialwert (JSON Default) | Invarianten/Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| id | TokenIdentifier | pub | N/A (erforderlich) | Eindeutiger, hierarchischer Bezeichner des Tokens. |
| value | TokenValue | pub | N/A (erforderlich) | Der Wert des Tokens, kann ein primitiver Typ oder eine Referenz auf ein anderes Token sein. |
| description | Option\<String\> | pub | None | Optionale Beschreibung des Tokens und seines Verwendungszwecks. |
| group | Option\<String\> | pub | None | Optionale Gruppierung (z.B. "Farben", "Typografie") zur besseren Organisation. |

* **Tabelle 2.2: ThemeDefinition Felder**

| Feldname | Rust-Typ | Sichtbarkeit | Initialwert (JSON Default) | Invarianten/Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| id | ThemeIdentifier | pub | N/A (erforderlich) | Eindeutiger Bezeichner des Themes. |
| name | String | pub | N/A (erforderlich) | Menschenlesbarer Name des Themes. |
| description | Option\<String\> | pub | None | Optionale Beschreibung des Themes. |
| author | Option\<String\> | pub | None | Optionaler Autor des Themes. |
| version | Option\<String\> | pub | None | Optionale Version des Themes. |
| base\_tokens | TokenSet | pub | N/A (erforderlich, kann leer sein) | Set von Basis-Tokens, die für alle Varianten gelten, falls nicht spezifisch überschrieben. |
| variants | Vec\<ThemeVariantDefinition\> | pub | \`\` (leerer Vektor) | Definitionen für spezifische Varianten (z.B. Hell, Dunkel). |
| supported\_accent\_colors | Option\<Vec\<AccentColor\>\> | pub | None | Optionale Liste vordefinierter Akzentfarben, die gut mit diesem Theme harmonieren. |

* **Tabelle 2.3: AppliedThemeState Felder**

| Feldname | Rust-Typ | Sichtbarkeit | Beschreibung |
| :---- | :---- | :---- | :---- |
| theme\_id | ThemeIdentifier | pub | ID des aktuell angewendeten Themes. |
| color\_scheme | ColorSchemeType | pub | Das aktuell angewendete Farbschema (Hell/Dunkel). |
| active\_accent\_color | Option\<AccentColor\> | pub | Die aktuell angewendete Akzentfarbe, falls eine ausgewählt wurde. |
| resolved\_tokens | std::collections::HashMap\<TokenIdentifier, String\> | pub | Eine Map aller Design-Tokens, aufgelöst zu ihren finalen, CSS-kompatiblen String-Werten. Enthält keine Referenzen. |

## **3\. Kernlogik und Geschäftsregeln (domain::theming::logic)**

Dieser Abschnitt beschreibt die internen Algorithmen und Regeln, die das Verhalten der Theming-Engine steuern. Diese Logik wird in privaten (priv) oder modul-internen (pub(crate)) Funktionen und Untermodulen innerhalb von domain::theming implementiert und von der in Abschnitt 4 definierten öffentlichen API genutzt.

### **3.1. Laden, Parsen und Validieren von Token- und Theme-Definitionen**

Die Theming-Engine muss in der Lage sein, Token- und Theme-Definitionen aus externen Quellen, typischerweise JSON-Dateien, zu laden, zu parsen und auf ihre Gültigkeit zu überprüfen.

* **Token-Dateien (\*.tokens.json):**  
  * **Ladepfade:** Token-Definitionen werden von standardisierten Pfaden geladen. Systemweite Tokens befinden sich beispielsweise unter /usr/share/desktop-environment/themes/tokens/, während benutzerspezifische Tokens unter $XDG\_CONFIG\_HOME/desktop-environment/themes/tokens/ (gemäß XDG Base Directory Specification) abgelegt werden können. Benutzerspezifische Dateien haben Vorrang und können systemweite Tokens überschreiben oder ergänzen.  
  * **Einlesen und Parsen:** Es wird eine Logik implementiert, die JSON-Dateien einliest, welche entweder ein Vec\<RawToken\> oder direkt ein TokenSet (als JSON-Objekt, bei dem Schlüssel Token-IDs sind) enthalten. Für das Parsen wird die serde\_json-Bibliothek verwendet.  
  * **Validierung:**  
    * **Eindeutigkeit der TokenIdentifier:** Beim Laden mehrerer Token-Dateien muss sichergestellt werden, dass Token-Identifier eindeutig sind. Bei Konflikten (gleiche ID aus verschiedenen Quellen) wird eine klare Strategie verfolgt: Benutzerspezifische Tokens haben Vorrang vor systemweiten Tokens. Bei gleichrangigen Konflikten wird eine Warnung geloggt, und das zuletzt geladene Token überschreibt das vorherige.  
    * **Zyklische Referenzen:** Es muss geprüft werden, ob TokenValue::Reference-Abhängigkeiten Zyklen bilden (z.B. Token A verweist auf B, B verweist auf A). Dies erfordert einen Graphenalgorithmus, wie z.B. eine Tiefensuche (DFS), um solche Zyklen zu erkennen. Ein erkannter Zyklus führt zu einem ThemingError::CyclicTokenReference.  
    * **Fehlerbehandlung:** Parse-Fehler (ungültiges JSON) oder ungültige Werte innerhalb der Tokens (z.B. ein fehlerhaftes Farbformat, das nicht CSS-kompatibel ist) führen zu einem ThemingError::TokenFileParseError bzw. ThemingError::InvalidTokenData.  
* **Theme-Definitionsdateien (\*.theme.json):**  
  * **Ladepfade:** Analog zu Token-Dateien, z.B. /usr/share/desktop-environment/themes/\[theme\_id\]/\[theme\_id\].theme.json für systemweite Themes und $XDG\_CONFIG\_HOME/desktop-environment/themes/\[theme\_id\]/\[theme\_id\].theme.json für benutzerspezifische Themes.  
  * **Einlesen und Parsen:** Es wird eine Logik implementiert, die JSON-Dateien einliest, die eine ThemeDefinition-Struktur repräsentieren. Auch hier kommt serde\_json zum Einsatz.  
  * **Validierung:**  
    * **Referenzierte Tokens:** Es muss sichergestellt werden, dass Tokens, die in base\_tokens oder variants\[\*\].tokens als TokenValue::Reference definiert sind, entweder auf bekannte globale Tokens (aus den geladenen \*.tokens.json-Dateien) verweisen oder innerhalb derselben ThemeDefinition (z.B. in base\_tokens) definiert sind. Fehlende Referenzen führen zu einem Fehler.  
    * **Vollständigkeit der Varianten:** Es sollte geprüft werden, ob für gängige ColorSchemeType-Werte (insbesondere Light und Dark) entsprechende ThemeVariantDefinitions existieren oder ob die base\_tokens als ausreichend für alle Schemata betrachtet werden können. Fehlende, aber erwartete Varianten könnten zu Warnungen führen.  
    * **Fehlerbehandlung:** Fehler beim Parsen oder ungültige Datenstrukturen führen zu ThemingError::ThemeFileLoadError oder ThemingError::InvalidThemeData.  
* **Logging:** Während des Lade-, Parse- und Validierungsprozesses wird das tracing-Framework intensiv genutzt:  
  * tracing::debug\!: Für Informationen über geladene Dateien und erfolgreich geparste Definitionen.  
  * tracing::warn\!: Für nicht-kritische Probleme, wie das Überschreiben von Tokens durch benutzerspezifische Definitionen oder kleinere Validierungsfehler, die nicht das Laden des gesamten Themes verhindern.  
  * tracing::error\!: Für kritische Fehler, die das Laden oder die Verwendung eines Tokensets oder einer Theme-Definition unmöglich machen (z.B. Parse-Fehler, zyklische Referenzen).

### **3.2. Mechanismus zur Auflösung und Vererbung von Tokens (Token Resolution Pipeline)**

Dies ist die zentrale Logikkomponente der Theming-Engine. Sie ist dafür verantwortlich, aus den rohen RawTokens, der ausgewählten ThemeDefinition und der aktuellen ThemingConfiguration die endgültigen, anwendbaren Token-Werte zu berechnen, die im AppliedThemeState.resolved\_tokens gespeichert werden. Dieser Prozess stellt sicher, dass alle Referenzen aufgelöst, Überschreibungen korrekt angewendet und spezifische Anpassungen (wie Akzentfarben) berücksichtigt werden.  
Die Auflösung erfolgt in einer klar definierten Reihenfolge von Schritten für eine gegebene ThemingConfiguration:

1. **Basissatz globaler Tokens bestimmen:**  
   * Lade alle RawTokens aus den systemweiten und benutzerspezifischen Token-Dateien (\*.tokens.json).  
   * Diese Sammlung bildet den "Foundation Layer" oder den globalen Token-Pool, auf den sich Themes beziehen können. Bei Namenskonflikten haben benutzerspezifische Tokens Vorrang.  
2. **Theme-spezifische Tokens laden und anwenden:**  
   * Identifiziere und lade die ThemeDefinition für die in ThemingConfiguration.selected\_theme\_id angegebene ID.  
   * Beginne mit einer Kopie der base\_tokens aus dieser ThemeDefinition. Diese Tokens können entweder eigenständige Werte definieren oder Referenzen auf Tokens im globalen Pool (aus Schritt 1\) sein.  
3. **Varianten-spezifische Tokens anwenden:**  
   * Ermittle die preferred\_color\_scheme (z.B. Light oder Dark) aus der ThemingConfiguration.  
   * Suche in der ThemeDefinition.variants nach einer ThemeVariantDefinition, deren applies\_to\_scheme mit der bevorzugten Einstellung übereinstimmt.  
   * Wenn eine passende Variante gefunden wird, merge deren tokens über das bisherige Set (aus Schritt 2). "Merging" bedeutet hier, dass Tokens aus der Variante gleichnamige Tokens aus den base\_tokens (oder dem globalen Pool, falls die Basis-Tokens Referenzen waren) überschreiben.  
4. **Akzentfarben-Logik anwenden (falls ThemingConfiguration.selected\_accent\_color vorhanden ist):**  
   * Dieser Schritt ist komplex und hängt stark davon ab, wie ein Theme die Integration von Akzentfarben definiert.  
   * **Ansatz 1: Direkte Ersetzung über spezielle Token-IDs:** Das Theme definiert Tokens mit speziellen, reservierten IDs (z.B. color.accent.primary.value, color.accent.secondary.value). Die Werte dieser Tokens werden direkt durch den value-Teil der selected\_accent\_color (z.B. "\#3498db") ersetzt. Das Theme kann auch Tokens definieren, die auf diese Akzent-Tokens verweisen (z.B. button.background.active verweist auf color.accent.primary.value).  
   * **Ansatz 2: Farbmanipulation (fortgeschritten):** Basierend auf der selected\_accent\_color.value könnten andere verwandte Farben dynamisch generiert werden (z.B. hellere/dunklere Schattierungen für Hover/Active-Zustände, kontrastierende Textfarben). Dies würde eine Farbmanipulationsbibliothek erfordern. Für die Erstimplementierung wird die direkte Ersetzung (Ansatz 1\) bevorzugt, da sie einfacher umzusetzen ist und weniger Abhängigkeiten erfordert.  
   * Die ThemeDefinition könnte ein Feld enthalten, das auflistet, welche ihrer Tokens als "akzentfähig" gelten und wie sie von der selected\_accent\_color beeinflusst werden.  
5. **Benutzerdefinierte globale Token-Overrides anwenden:**  
   * Wenn in der ThemingConfiguration ein custom\_user\_token\_overrides-Set vorhanden ist, merge diese Tokens über das bisherige, aus den vorherigen Schritten resultierende Set. Diese benutzerdefinierten Überschreibungen haben die höchste Priorität und überschreiben jeden zuvor festgelegten Wert für ein Token mit derselben ID.  
6. **Referenzen auflösen (rekursiv):**  
   * Nachdem alle Überschreibungen angewendet wurden, iteriere durch alle Tokens im aktuellen Set.  
   * Wenn ein Token den Wert TokenValue::Reference(target\_id) hat:  
     * Suche das Token mit der target\_id im aktuellen Set.  
     * **Erfolgreiche Auflösung:** Wenn target\_id gefunden wird und dessen Wert *kein* weiterer Reference ist (d.h., es ist ein konkreter Wert wie Color, Dimension etc.), ersetze den Wert des ursprünglichen Tokens (das die Referenz enthielt) durch den aufgelösten Wert des Ziel-Tokens.  
     * **Kaskadierte Referenz:** Wenn target\_id gefunden wird, aber dessen Wert ebenfalls ein Reference ist, muss diese Referenz ebenfalls aufgelöst werden. Dieser Prozess wird rekursiv fortgesetzt.  
     * **Fehlende Referenz:** Wenn target\_id nicht im aktuellen Set gefunden wird, ist dies ein Fehler, der als ThemingError::MissingTokenReference behandelt wird. Das referencing Token kann nicht aufgelöst werden.  
     * **Zyklenerkennung:** Während der rekursiven Auflösung muss ein Mechanismus zur Erkennung von Zyklen aktiv sein (z.B. durch Verfolgung des Auflösungspfads). Ein Zyklus (z.B. A → B → C → A) würde zu einer Endlosschleife führen und muss als ThemingError::CyclicTokenReference abgefangen werden. Die Validierung in Schritt 3.1 sollte Zyklen bereits erkennen, aber eine zusätzliche Prüfung hier dient als Sicherheitsnetz.  
     * **Maximale Rekursionstiefe:** Eine maximale Tiefe für die Auflösung von Referenzen (z.B. 10-20 Ebenen) sollte festgelegt werden, um bei unentdeckten Fehlern oder extrem verschachtelten (aber gültigen) Strukturen eine Endlosschleife zu verhindern und einen ThemingError::MaxReferenceDepthExceeded auszulösen.  
7. **Finale Wertkonvertierung und Erstellung des AppliedThemeState:**  
   * Nachdem alle Referenzen erfolgreich aufgelöst wurden, enthält das Token-Set nur noch konkrete TokenValue-Varianten (außer Reference).  
   * Konvertiere alle diese TokenValues in ihre finalen String-Repräsentationen, die direkt von der UI-Schicht (z.B. als CSS-Werte) verwendet werden können. Beispielsweise wird TokenValue::Color("\#aabbcc".to\_string()) zu String::from("\#aabbcc").  
   * Das Ergebnis dieser Konvertierung ist eine HashMap\<TokenIdentifier, String\>, die zusammen mit der theme\_id, color\_scheme und active\_accent\_color aus der ThemingConfiguration den neuen AppliedThemeState bildet.  
* **Caching:** Da die Token-Auflösung potenziell rechenintensiv sein kann (insbesondere bei vielen Tokens, komplexen Referenzen und häufigen Theme-Wechseln), sollte ein Caching-Mechanismus in Betracht gezogen werden.  
  * Ein aufgelöstes AppliedThemeState (oder zumindest das resolved\_tokens-Set) kann für eine gegebene Kombination aus (ThemeIdentifier, ColorSchemeType, Option\<AccentColor\>, HashOfUserOverrides) gecacht werden.  
  * Der Cache muss invalidiert werden, wenn sich zugrundeliegende Token-Dateien (\*.tokens.json) oder Theme-Definitionen (\*.theme.json) ändern (z.B. durch Aufruf von reload\_themes\_and\_tokens() in der ThemingEngine) oder wenn sich die custom\_user\_token\_overrides ändern.

### **3.3. Regeln für dynamische Theme-Wechsel und Aktualisierung des Theme-Zustands**

Die Theming-Engine muss in der Lage sein, auf Änderungen der ThemingConfiguration (z.B. durch Benutzereingaben in den Einstellungen) dynamisch zur Laufzeit zu reagieren.

1. **Benachrichtigung über Konfigurationsänderung:** Die ThemingEngine wird über eine Änderung der ThemingConfiguration informiert, typischerweise durch einen Methodenaufruf ihrer öffentlichen API (z.B. update\_configuration(new\_config)).  
2. **Neuberechnung des Theme-Zustands:** Nach Erhalt der neuen Konfiguration führt die ThemingEngine die vollständige Token Resolution Pipeline (wie in Abschnitt 3.2 beschrieben) erneut aus, unter Verwendung der new\_config.  
3. **Aktualisierung des internen Zustands:** Der resultierende AppliedThemeState wird zum neuen internen aktuellen Zustand der ThemingEngine.  
4. **Event-Benachrichtigung:** Wenn sich der neu berechnete AppliedThemeState vom vorherigen Zustand unterscheidet, emittiert die ThemingEngine ein ThemeChangedEvent. Dieses Event enthält den neuen AppliedThemeState und ermöglicht es anderen Teilen des Systems (insbesondere der UI-Schicht), auf die Änderung zu reagieren und ihr Erscheinungsbild entsprechend zu aktualisieren.

### **3.4. Invarianten und Konsistenzprüfungen**

Um die Stabilität und Korrektheit des Theming-Systems zu gewährleisten, müssen bestimmte Invarianten jederzeit gelten:

* **Keine Referenzen im AppliedThemeState:** Das Feld resolved\_tokens eines AppliedThemeState-Objekts darf unter keinen Umständen TokenValue::Reference-Typen (oder deren String-Äquivalente, falls die Auflösung fehlschlägt) enthalten. Alle Werte müssen endgültig und direkt verwendbar sein.  
* **Gültiger Fallback-Zustand:** Die ThemingEngine muss auch dann einen gültigen (wenn auch möglicherweise minimalen) AppliedThemeState bereitstellen können, wenn Konfigurationsdateien fehlerhaft, unvollständig oder nicht vorhanden sind. Hierfür ist ein Default-Fallback-Theme erforderlich. Dieses Fallback-Theme sollte entweder fest im Code einkompiliert sein (z.B. über include\_str\! aus eingebetteten JSON-Ressourcen) oder aus einer garantierten, immer verfügbaren Quelle geladen werden können. Ein Fehlschlagen beim Laden des Fallback-Themes ist ein kritischer Fehler (ThemingError::FallbackThemeLoadError).  
* **Zuverlässige Zyklenerkennung:** Zyklische Abhängigkeiten in Token-Referenzen müssen bei der Validierung (3.1) und spätestens bei der Auflösung (3.2) zuverlässig erkannt und als Fehler (ThemingError::CyclicTokenReference) behandelt werden, um Endlosschleifen und Systeminstabilität zu verhindern.  
* **Konsistenz der ThemeIdentifier:** Alle in ThemingConfiguration oder intern verwendeten ThemeIdentifier müssen auf tatsächlich geladene und validierte ThemeDefinitions verweisen, es sei denn, es handelt sich um den expliziten Fallback-Zustand.

## **4\. Öffentliche API-Spezifikation (domain::theming::api)**

Dieser Abschnitt definiert die öffentliche Schnittstelle des domain::theming-Moduls. Die Interaktion mit der Theming-Logik erfolgt primär über den ThemingEngine-Service. Diese API ist so gestaltet, dass sie klar, robust und einfach von anderen Modulen, insbesondere der UI-Schicht und der Einstellungsverwaltung, genutzt werden kann.

### **4.1. Haupt-Service: ThemingEngine**

Der ThemingEngine-Service ist die zentrale Struktur, die die gesamte Theming-Logik kapselt, den aktuellen Theme-Zustand verwaltet und als Schnittstelle für andere Systemteile dient. Er wird typischerweise als eine gemeinsam genutzte, langlebige Instanz im System existieren (z.B. als Singleton oder über Dependency Injection bereitgestellt).  
Die Implementierung muss Thread-Sicherheit gewährleisten (Send \+ Sync), da von verschiedenen Threads (z.B. UI-Thread, Hintergrund-Threads für Konfigurationsaktualisierungen) darauf zugegriffen werden könnte. Dies wird üblicherweise durch die Verwendung von Arc\<Mutex\<ThemingEngineInternalState\>\> für den internen, veränderlichen Zustand erreicht.

Rust

// Angenommen in domain::theming::mod.rs oder domain::theming::api.rs

use crate::core::errors::CoreError; // Basis-Fehlertyp, falls benötigt  
use super::types::\*;  
use super::errors::ThemingError;  
use std::sync::{Arc, Mutex};  
use std::path::PathBuf;  
// Für Eventing wird eine robuste Multi-Producer, Multi-Consumer (MPMC) Broadcast-Lösung  
// oder eine sorgfältig verwaltete Liste von mpsc-Sendern empfohlen.  
// Hier als Beispiel mit einer Liste von mpsc::Sendern für Einfachheit,  
// aber tokio::sync::broadcast oder crossbeam\_channel::Sender (cloneable) wären bessere Optionen.  
use std::sync::mpsc;

pub struct ThemingEngine {  
    internal\_state: Arc\<Mutex\<ThemingEngineInternalState\>\>,  
    // Hält Sender-Enden für alle Subscriber.  
    event\_subscribers: Arc\<Mutex\<Vec\<mpsc::Sender\<ThemeChangedEvent\>\>\>\>,  
}

struct ThemingEngineInternalState {  
    current\_config: ThemingConfiguration,  
    available\_themes: Vec\<ThemeDefinition\>, // Geladen beim Start/Refresh  
    global\_raw\_tokens: TokenSet, // Globale Tokens, nicht Teil eines Themes  
    applied\_state: AppliedThemeState,  
    // Pfade, von denen Tokens und Themes geladen wurden, für \`reload\_themes\_and\_tokens\`  
    theme\_load\_paths: Vec\<PathBuf\>,  
    token\_load\_paths: Vec\<PathBuf\>,  
    // Optional: Cache für aufgelöste Token-Sets  
    // resolved\_state\_cache: HashMap\<CacheKey, AppliedThemeState\>,  
}

impl ThemingEngine {  
    // Konstruktor und Methoden werden unten definiert  
}

#### **4.1.1. Deklarierte Eigenschaften (Properties)**

Diese Eigenschaften repräsentieren den Kernzustand der ThemingEngine. Der Zugriff erfolgt ausschließlich über die unten definierten Methoden, um Kapselung und kontrollierte Zustandsänderungen zu gewährleisten.

* **Aktueller AppliedThemeState:** Der vollständig aufgelöste und angewendete Theme-Zustand. Zugänglich über get\_current\_theme\_state().  
* **Liste der verfügbaren Themes (Vec\<ThemeDefinition\>):** Eine Liste aller erfolgreich geladenen und validierten Theme-Definitionen. Zugänglich über get\_available\_themes().  
* **Aktuelle ThemingConfiguration:** Die derzeit von der Engine verwendete Benutzerkonfiguration. Zugänglich über get\_current\_configuration().

#### **4.1.2. Methoden**

Die Methoden der ThemingEngine ermöglichen die Initialisierung, Abfrage des Zustands, Aktualisierung der Konfiguration und die Registrierung für Benachrichtigungen über Zustandsänderungen.

* **Konstruktoren/Builder:**  
  * pub fn new(initial\_config: ThemingConfiguration, theme\_load\_paths: Vec\<PathBuf\>, token\_load\_paths: Vec\<PathBuf\>) \-\> Result\<Self, ThemingError\>  
    * **Beschreibung:** Initialisiert die ThemingEngine. Lädt alle verfügbaren Themes und Tokens von den angegebenen theme\_load\_paths und token\_load\_paths. Wendet die initial\_config an, um den ersten AppliedThemeState zu berechnen. Wenn dieser Prozess fehlschlägt, wird versucht, ein Fallback-Theme zu laden.  
    * **Parameter:**  
      * initial\_config: ThemingConfiguration: Die anfängliche Benutzerkonfiguration für das Theming.  
      * theme\_load\_paths: Vec\<PathBuf\>: Eine Liste von Verzeichnispfaden, in denen nach Theme-Definitionen (\*.theme.json) gesucht wird.  
      * token\_load\_paths: Vec\<PathBuf\>: Eine Liste von Verzeichnispfaden, in denen nach globalen Token-Dateien (\*.tokens.json) gesucht wird.  
    * **Rückgabe:** Result\<Self, ThemingError\>. Gibt die initialisierte ThemingEngine oder einen Fehler zurück.  
    * **Vorbedingungen:** initial\_config sollte semantisch valide sein (obwohl die Engine dies prüft). Die angegebenen Pfade müssen für das Programm lesbar sein.  
    * **Nachbedingungen:** Bei Erfolg ist die Engine initialisiert, verfügt über einen gültigen applied\_state (entweder basierend auf initial\_config oder einem Fallback) und hat alle verfügbaren Themes/Tokens geladen. event\_subscribers ist initialisiert (leer).  
    * **Mögliche Fehler:** ThemingError::TokenFileParseError, ThemingError::ThemeFileLoadError, ThemingError::CyclicTokenReference, ThemingError::InitialConfigurationError (wenn initial\_config zu einem unauflösbaren Zustand führt), ThemingError::FallbackThemeLoadError (wenn selbst das Laden des Fallback-Themes fehlschlägt).  
* **Zustandsabfrage:**  
  * pub fn get\_current\_theme\_state(\&self) \-\> Result\<AppliedThemeState, ThemingError\>  
    * **Beschreibung:** Gibt eine Kopie (Clone) des aktuellen AppliedThemeState zurück. Dies ist der primäre Weg für die UI-Schicht, die aktuellen Theme-Werte abzurufen.  
    * **Rückgabe:** Result\<AppliedThemeState, ThemingError\>. Ein Fehler ist hier unwahrscheinlich, könnte aber bei schwerwiegenden internen Inkonsistenzen auftreten (z.B. ThemingError::InternalStateError).  
    * **Thread-Sicherheit:** Diese Methode ist lesend und greift auf den internen Zustand über einen Mutex zu.  
  * pub fn get\_available\_themes(\&self) \-\> Result\<Vec\<ThemeDefinition\>, ThemingError\>  
    * **Beschreibung:** Gibt eine Kopie (Clone) der Liste aller geladenen und validierten ThemeDefinitions zurück. Nützlich für UI-Elemente, die eine Theme-Auswahl anbieten.  
    * **Rückgabe:** Result\<Vec\<ThemeDefinition\>, ThemingError\>. Fehler wie bei get\_current\_theme\_state().  
  * pub fn get\_current\_configuration(\&self) \-\> Result\<ThemingConfiguration, ThemingError\>  
    * **Beschreibung:** Gibt eine Kopie (Clone) der aktuell von der Engine verwendeten ThemingConfiguration zurück.  
    * **Rückgabe:** Result\<ThemingConfiguration, ThemingError\>. Fehler wie bei get\_current\_theme\_state().  
* **Zustandsänderung:**  
  * pub fn update\_configuration(\&self, new\_config: ThemingConfiguration) \-\> Result\<(), ThemingError\>  
    * **Beschreibung:** Aktualisiert die Konfiguration der ThemingEngine mit der new\_config. Dies löst die Token Resolution Pipeline (Abschnitt 3.2) neu aus. Der interne applied\_state wird aktualisiert. Wenn sich der applied\_state dadurch tatsächlich ändert, wird ein ThemeChangedEvent an alle registrierten Subscriber gesendet.  
    * **Parameter:**  
      * new\_config: ThemingConfiguration: Die neue anzuwendende Benutzerkonfiguration.  
    * **Rückgabe:** Result\<(), ThemingError\>.  
    * **Vorbedingungen:** new\_config sollte semantisch valide sein.  
    * **Nachbedingungen:** Der interne Zustand (current\_config, applied\_state) ist aktualisiert. Bei einer relevanten Änderung wurde ein ThemeChangedEvent gesendet.  
    * **Mögliche Fehler:** ThemingError::ThemeNotFound (wenn new\_config.selected\_theme\_id ungültig ist), ThemingError::TokenResolutionError (z.B. MissingTokenReference, CyclicTokenReference während der Anwendung der neuen Konfiguration), ThemingError::ThemeApplicationError für allgemeinere Probleme.  
  * pub fn reload\_themes\_and\_tokens(\&self) \-\> Result\<(), ThemingError\>  
    * **Beschreibung:** Veranlasst die ThemingEngine, alle Theme-Definitionen und Token-Dateien von den beim Konstruktor angegebenen Pfaden neu zu laden. Dies ist nützlich, wenn der Benutzer z.B. neue Themes manuell installiert oder bestehende Token-Dateien extern bearbeitet hat. Nach dem Neuladen wird die *aktuell gespeicherte* ThemingConfiguration auf die neu geladenen Daten angewendet. Wenn sich der applied\_state dadurch ändert, wird ein ThemeChangedEvent gesendet.  
    * **Rückgabe:** Result\<(), ThemingError\>.  
    * **Nachbedingungen:** Der interne Bestand an available\_themes und global\_raw\_tokens ist aktualisiert. Der applied\_state ist basierend auf der aktuellen Konfiguration und den neuen Daten neu berechnet. Ein Event wurde ggf. gesendet.  
    * **Mögliche Fehler:** ThemingError::TokenFileIoError, ThemingError::TokenFileParseError, ThemingError::ThemeFileIoError, ThemingError::ThemeFileLoadError (beim Neuladen), sowie Fehler, die auch bei update\_configuration auftreten können, da der Zustand neu angewendet wird.  
* **Event-Handling (Subscription):**  
  * pub fn subscribe\_to\_theme\_changes(\&self) \-\> Result\<mpsc::Receiver\<ThemeChangedEvent\>, ThemingError\>  
    * **Beschreibung:** Ermöglicht anderen Teilen des Systems (Subscriber), sich für Benachrichtigungen über Änderungen am AppliedThemeState zu registrieren. Jeder Aufruf dieser Methode erstellt einen neuen Kommunikationskanal.  
    * **Rückgabe:** Result\<mpsc::Receiver\<ThemeChangedEvent\>, ThemingError\>. Der zurückgegebene Receiver kann verwendet werden, um ThemeChangedEvents asynchron zu empfangen.  
    * **Implementierungsdetails:** Die ThemingEngine hält eine Liste von mpsc::Sender\<ThemeChangedEvent\>-Enden (in event\_subscribers). Diese Methode erstellt ein neues mpsc::channel(), fügt den Sender-Teil zur Liste hinzu und gibt den Receiver-Teil zurück. Beim Senden eines Events iteriert die Engine über alle gespeicherten Sender und versucht, das Event zu senden. Sender, deren korrespondierender Receiver nicht mehr existiert (Kanal geschlossen), werden aus der Liste entfernt.  
    * **Mögliche Fehler:** ThemingError::EventSubscriptionError (z.B. bei Problemen mit der internen Verwaltung der Subscriber-Liste, obwohl dies bei korrekter Implementierung selten sein sollte).

#### **4.1.3. Signale/Events**

Die ThemingEngine verwendet Events, um andere Systemkomponenten über relevante Zustandsänderungen zu informieren, ohne eine enge Kopplung zu erfordern.

* **ThemeChangedEvent (Struct):**  
  * **Beschreibung:** Dieses Event wird von der ThemingEngine immer dann gesendet, wenn sich der AppliedThemeState erfolgreich geändert hat, sei es durch eine neue Benutzerkonfiguration oder durch das Neuladen von Theme-Daten.  
  * **Payload:**  
    Rust  
    \# // Clone ist wichtig, damit das Event an mehrere Subscriber gesendet werden kann.  
                            // Serialize ist nicht unbedingt nötig für interne Events.  
    pub struct ThemeChangedEvent {  
        pub new\_state: AppliedThemeState,  
        // Optional könnte hier auch der alte Zustand für Vergleiche mitgesendet werden:  
        // pub old\_state: Option\<AppliedThemeState\>,  
    }

  * **Typischer Publisher:** Die ThemingEngine selbst, innerhalb der Methoden update\_configuration und reload\_themes\_and\_tokens.  
  * **Typische Subscriber:**  
    * ui::theming\_gtk (oder ein äquivalentes Modul in der UI-Schicht): Um die GTK4-CSS-Provider mit den neuen, in new\_state.resolved\_tokens enthaltenen Werten zu aktualisieren.  
    * Andere UI-Komponenten oder Widgets, die direkt auf spezifische Token-Werte reagieren müssen, ohne den Umweg über CSS (obwohl dies seltener sein sollte).  
    * Potenziell andere Domänen- oder Systemdienste, die ihr Verhalten an das aktuelle Theme anpassen müssen.

### **4.2. Tabellen für API-Spezifikation**

Diese Tabellen bieten eine kompakte Übersicht über die Methoden der ThemingEngine und die von ihr emittierten Events. Sie sind entscheidend für Entwickler, die die Engine nutzen, da sie klare Erwartungen an Signaturen, Verhalten und Fehlerfälle setzen.

* **Tabelle 4.1: ThemingEngine-Methoden**

| Name | Signatur | Zugriff | Kurzbeschreibung | Vorbedingungen | Nachbedingungen | ThemingError-Varianten (Beispiele) |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| new | (initial\_config: ThemingConfiguration, theme\_load\_paths: Vec\<PathBuf\>, token\_load\_paths: Vec\<PathBuf\>) \-\> Result\<Self, ThemingError\> | pub | Konstruktor. Initialisiert Engine, lädt Themes/Tokens, wendet Erstkonfiguration an, richtet Fallback ein. | initial\_config valide, Pfade lesbar. | Engine initialisiert, applied\_state gültig. | ThemeLoadError, TokenParseError, InitialConfigurationError, FallbackThemeLoadError |
| get\_current\_theme\_state | (\&self) \-\> Result\<AppliedThemeState, ThemingError\> | pub | Gibt den aktuell angewendeten AppliedThemeState zurück. | Engine muss initialisiert sein. | Eine Kopie des Zustands wird zurückgegeben. | InternalStateError (selten) |
| get\_available\_themes | (\&self) \-\> Result\<Vec\<ThemeDefinition\>, ThemingError\> | pub | Gibt eine Liste aller verfügbaren, geladenen Theme-Definitionen zurück. | Engine muss initialisiert sein. | Eine Kopie der Liste wird zurückgegeben. | InternalStateError (selten) |
| get\_current\_configuration | (\&self) \-\> Result\<ThemingConfiguration, ThemingError\> | pub | Gibt die aktuell verwendete ThemingConfiguration zurück. | Engine muss initialisiert sein. | Eine Kopie der Konfiguration wird zurückgegeben. | InternalStateError (selten) |
| update\_configuration | (\&self, new\_config: ThemingConfiguration) \-\> Result\<(), ThemingError\> | pub | Aktualisiert Konfiguration, berechnet neuen Zustand und sendet ggf. ThemeChangedEvent. | new\_config valide. | Interner Zustand aktualisiert, ThemeChangedEvent ggf. gesendet. | ThemeApplicationError, TokenResolutionError, ThemeNotFound |
| reload\_themes\_and\_tokens | (\&self) \-\> Result\<(), ThemingError\> | pub | Lädt alle Theme- und Token-Dateien neu und wendet aktuelle Konfiguration an. Sendet ggf. ThemeChangedEvent. | Konfigurierte Pfade müssen weiterhin zugänglich sein. | Interner Datenbestand aktualisiert, ThemeChangedEvent ggf. gesendet. | ThemeLoadError, TokenParseError, ThemeApplicationError |
| subscribe\_to\_theme\_changes | (\&self) \-\> Result\<mpsc::Receiver\<ThemeChangedEvent\>, ThemingError\> | pub | Registriert einen Listener für ThemeChangedEvent und gibt einen Receiver zurück. | Engine muss initialisiert sein. | Ein mpsc::Receiver wird zurückgegeben, Sender intern registriert. | EventSubscriptionError |

* **Tabelle 4.2: ThemeChangedEvent**

| Event-Name/Typ | Payload-Struktur (pub fields: Type) | Typische Publisher | Typische Subscriber | Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| ThemeChangedEvent | new\_state: AppliedThemeState | ThemingEngine | ui::theming\_gtk (und Äquivalente), UI-Komponenten, die direkt auf Tokens reagieren | Wird ausgelöst, nachdem der AppliedThemeState der ThemingEngine erfolgreich aktualisiert und geändert wurde. |

## **5\. Fehlerbehandlung (domain::theming::errors)**

Eine robuste und aussagekräftige Fehlerbehandlung ist entscheidend für die Stabilität und Wartbarkeit des domain::theming-Moduls. Gemäß den übergeordneten Entwicklungsrichtlinien (Abschnitt 4.3 der Gesamtspezifikation) wird das thiserror-Crate verwendet, um spezifische, benutzerdefinierte Fehler-Enums pro Modul zu definieren. Dies ermöglicht eine klare Kommunikation von Fehlerzuständen sowohl innerhalb des Moduls als auch an dessen Aufrufer.  
Die Fehlerbehandlung in Rust, die sich um das Result\<T, E\>-Enum dreht 1, erfordert eine sorgfältige Definition der Fehlertypen E. Während std::error::Error eine Basistrait ist 2, bieten Crates wie thiserror erhebliche Erleichterungen bei der Erstellung benutzerdefinierter Fehlertypen, die diesen Trait implementieren.1

### **5.1. Definition des ThemingError Enums**

Das ThemingError-Enum fasst alle spezifischen Fehler zusammen, die innerhalb des domain::theming-Moduls auftreten können. Jede Variante des Enums repräsentiert einen distinkten Fehlerfall und ist mit einer aussagekräftigen Fehlermeldung versehen, die Kontextinformationen für Entwickler bereitstellt. Die Verwendung von \#\[from\] für Fehler aus tieferliegenden Bibliotheken (wie std::io::Error oder serde\_json::Error) ermöglicht eine einfache Fehlerkonvertierung und erhält die Kausalkette (source()).

Rust

// In domain::theming::errors.rs  
use thiserror::Error;  
use super::types::{TokenIdentifier, ThemeIdentifier}; // Annahme: types.rs ist im selben Modul  
use std::path::PathBuf;

\#  
pub enum ThemingError {  
    \#\[error("Failed to parse token file '{path}': {source}")\]  
    TokenFileParseError {  
        path: PathBuf,  
        \#\[source\]  
        source: serde\_json::Error,  
    },

    \#\[error("I/O error while processing token file '{path}': {source}")\]  
    TokenFileIoError {  
        path: PathBuf,  
        \#\[source\]  
        source: std::io::Error,  
    },

    \#\[error("Invalid token data in file '{path}': {message}")\]  
    InvalidTokenData {  
        path: PathBuf,  
        message: String,  
    },

    \#\[error("Cyclic dependency detected involving token '{token\_id}' during token validation or resolution")\]  
    CyclicTokenReference {  
        token\_id: TokenIdentifier,  
        // Optional: path\_to\_cycle: Vec\<TokenIdentifier\> // Zur besseren Diagnose  
    },

    \#\[error("Failed to load theme definition '{theme\_id}' from file '{path}': {source}")\]  
    ThemeFileLoadError {  
        theme\_id: ThemeIdentifier,  
        path: PathBuf,  
        \#\[source\]  
        source: serde\_json::Error,  
    },

    \#\[error("I/O error while loading theme definition '{theme\_id}' from file '{path}': {source}")\]  
    ThemeFileIoError {  
        theme\_id: ThemeIdentifier,  
        path: PathBuf,  
        \#\[source\]  
        source: std::io::Error,  
    },

    \#\[error("Invalid theme data for theme '{theme\_id}' in file '{path}': {message}")\]  
    InvalidThemeData {  
        theme\_id: ThemeIdentifier,  
        path: PathBuf,  
        message: String,  
    },

    \#  
    ThemeNotFound {  
        theme\_id: ThemeIdentifier,  
    },

    \#  
    MissingTokenReference {  
        referencing\_token\_id: TokenIdentifier,  
        target\_token\_id: TokenIdentifier,  
    },

    \#  
    MaxReferenceDepthExceeded {  
        token\_id: TokenIdentifier,  
    },

    \#\[error("Failed to apply theming configuration: {message}")\]  
    ThemeApplicationError {  
        message: String,  
        // Optional: \#\[source\] source: Option\<Box\<dyn std::error::Error \+ Send \+ Sync \+ 'static\>\>,  
    },

    \#\[error("Critical error: Failed to initialize theming engine because no suitable fallback theme could be loaded.")\]  
    FallbackThemeLoadError,

    \#  
    InitialConfigurationError(String),  
      
    \#  
    InternalStateError(String),

    \#\[error("Failed to subscribe to theme change events: {0}")\]  
    EventSubscriptionError(String),

    // Beispiel für einen Wrapper für Core-Fehler, falls das Projekt einen zentralen CoreError hat.  
    // Dies ist oft weniger spezifisch als dedizierte Fehler, kann aber für die Integration nützlich sein.  
    // \#\[error("Core system error: {source}")\]  
    // CoreError(\#\[from\] crate::core::errors::CoreError),  
}

Die gewählte Granularität – ein Fehler-Enum pro Modul (ThemingError) mit spezifischen Varianten – stellt einen guten Kompromiss dar. Es vermeidet eine übermäßige Anzahl von Fehlertypen über das gesamte Projekt hinweg, bietet aber dennoch genügend Spezifität, um Fehlerquellen innerhalb des Moduls klar zu identifizieren und darauf reagieren zu können.4 Die Fehlermeldungen sind so gestaltet, dass sie möglichst viel Kontext liefern (z.B. Dateipfade, Token-IDs), was die Fehlersuche erheblich erleichtert und der Anforderung nach aussagekräftigen Fehlerberichten entspricht.1  
Die \#\[from\]-Annotation von thiserror wird genutzt, um Fehler von Abhängigkeiten wie serde\_json::Error und std::io::Error nahtlos in spezifische ThemingError-Varianten zu überführen. Dies vereinfacht den Code, da der ?-Operator direkt verwendet werden kann, und stellt sicher, dass die ursprüngliche Fehlerquelle (source) erhalten bleibt.1 Die Unterscheidung zwischen TokenFileIoError und ThemeFileIoError, obwohl beide potenziell von std::io::Error stammen, ist hier gerechtfertigt, da sie unterschiedliche logische Operationen (Lesen einer Token-Datei vs. Lesen einer Theme-Datei) und unterschiedliche Kontextinformationen (nur path vs. theme\_id und path) repräsentieren. Dies vermeidet die in 1 erwähnte Problematik, dass der Kontext bei der reinen Verwendung von \#\[from\] für denselben Quelltyp verschwimmen kann, wenn nicht genügend differenzierende Felder vorhanden sind.

### **5.2. Richtlinien zur Fehlerbehandlung und \-weitergabe innerhalb des Moduls**

* **Fehlerkonvertierung:** Innerhalb der privaten Logikfunktionen des domain::theming-Moduls (Abschnitt 3\) werden auftretende Fehler (z.B. I/O-Fehler beim Dateizugriff, Parsing-Fehler von serde\_json) systematisch in die entsprechenden Varianten von ThemingError umgewandelt. Dies geschieht häufig automatisch durch die Verwendung des ?-Operators in Verbindung mit den \#\[from\]-Annotationen im ThemingError-Enum oder, falls notwendig, manuell durch Aufrufe von .map\_err().  
* **Vermeidung von Panics:** Panics, ausgelöst durch unwrap() oder expect(), sind im Code des domain::theming-Moduls strikt zu vermeiden. Die einzige Ausnahme bilden potenziell Situationen, in denen ein absolut inkonsistenter Zustand eine sichere Fortführung des Programms unmöglich macht (z.B. ein kritischer, nicht behebbarer Fehler beim Laden des essentiellen Fallback-Themes während der Initialisierung der ThemingEngine). Solche Fälle müssen extrem selten sein, sorgfältig dokumentiert und begründet werden. Falls ein expect() in einer solchen Ausnahmesituation verwendet wird, sollte die Nachricht dem "expect as precondition"-Stil folgen, der beschreibt, warum der Entwickler erwartet hat, dass die Operation erfolgreich sein würde.2  
* **Fehlerweitergabe durch die API:** Alle öffentlichen Methoden der ThemingEngine (Abschnitt 4), die fehlschlagen können, geben Result\<T, ThemingError\> zurück. Dies zwingt den aufrufenden Code, Fehler explizit zu behandeln und ermöglicht eine differenzierte Reaktion auf verschiedene Fehlerzustände.  
* **Nutzung der source()-Kette:** Durch die korrekte Verwendung von \#\[source\] in den thiserror-Definitionen wird die Kausalkette von Fehlern bewahrt. Dies ist besonders nützlich für das Debugging, da es ermöglicht, einen Fehler bis zu seiner ursprünglichen Ursache zurückzuverfolgen, auch über Modul- oder Bibliotheksgrenzen hinweg.3

### **5.3. Tabelle für Fehlerbehandlung**

Die folgende Tabelle listet eine Auswahl der wichtigsten ThemingError-Varianten auf, beschreibt ihre Bedeutung und die typischen Umstände ihres Auftretens. Dies dient Entwicklern als Referenz für die Implementierung der Fehlerbehandlung im aufrufenden Code und für das Debugging.

* **Tabelle 5.1: ThemingError-Varianten (Auswahl)**

| Variante | \#\[error("...")\] String (Beispiel) | Gekapselter Quellfehler (via \#\[from\] oder Feld) | Beschreibung des Fehlerfalls |
| :---- | :---- | :---- | :---- |
| TokenFileParseError | "Failed to parse token file '{path}': {source}" | path: PathBuf, source: serde\_json::Error | Fehler beim Parsen einer JSON-Datei, die Tokens enthält (z.B. Syntaxfehler im JSON). |
| TokenFileIoError | "I/O error while processing token file '{path}': {source}" | path: PathBuf, source: std::io::Error | Ein-/Ausgabefehler beim Lesen oder Schreiben einer Token-Datei (z.B. Datei nicht gefunden, keine Leserechte). |
| CyclicTokenReference | "Cyclic dependency detected involving token '{token\_id}'..." | token\_id: TokenIdentifier | Eine zirkuläre Referenz zwischen Tokens wurde gefunden (z.B. Token A verweist auf B, und B verweist zurück auf A). |
| ThemeNotFound | "Theme with ID '{theme\_id}' not found among available themes" | theme\_id: ThemeIdentifier | Ein angefordertes Theme (z.B. in ThemingConfiguration) konnte nicht in den geladenen Definitionen gefunden werden. |
| MissingTokenReference | "Token resolution failed: Referenced token '{target\_token\_id}' not found (referenced by '{referencing\_token\_id}')" | referencing\_token\_id: TokenIdentifier, target\_token\_id: TokenIdentifier | Ein Token verweist auf ein anderes Token (target\_token\_id), das jedoch nicht im aktuellen Auflösungskontext existiert. |
| ThemeApplicationError | "Failed to apply theming configuration: {message}" | message: String | Allgemeiner Fehler während des Prozesses, eine neue ThemingConfiguration anzuwenden und den AppliedThemeState zu generieren. |
| FallbackThemeLoadError | "Critical error: Failed to initialize theming engine because no suitable fallback theme could be loaded." | \- | Kritischer Initialisierungsfehler: Das essentielle Fallback-Theme konnte nicht geladen oder verarbeitet werden. |
| InternalStateError | "An internal, unrecoverable error occurred in the ThemingEngine: {0}" | String (Fehlermeldung) | Ein unerwarteter, interner Fehler in der Engine, der auf einen Programmierfehler oder eine Datenkorruption hindeutet. |

## **6\. Vorgeschlagene Dateistruktur für das Modul domain::theming**

Eine klare und logische Dateistruktur ist entscheidend für die Wartbarkeit und Verständlichkeit eines Moduls. Für domain::theming wird folgende Struktur vorgeschlagen:

domain/  
└── theming/  
    ├── mod.rs           // Hauptmoduldatei (public API: ThemingEngine, Re-Exports)  
    ├── types.rs         // Definition aller Datenstrukturen (Token\*, Theme\*, Config\*, Event\*)  
    ├── errors.rs        // Definition des ThemingError Enums und zugehöriger Typen  
    ├── logic.rs         // Interne Implementierung der Kernlogik (Token-Laden, \-Auflösung etc.)  
    │                    // Kann bei Bedarf in Untermodule aufgeteilt werden:  
    │                    //   logic/token\_parser.rs  
    │                    //   logic/theme\_loader.rs  
    │                    //   logic/token\_resolver.rs  
    │                    //   logic/accent\_color\_processor.rs  
    ├── default\_themes/  // Verzeichnis für eingebettete Fallback-Theme-Dateien (JSON)  
    │   └── fallback.theme.json  
    │   └── base.tokens.json // Minimale Basis-Tokens für das Fallback-Theme  
    └──Cargo.toml        // Falls domain::theming als eigenes Crate innerhalb eines Workspace konzipiert ist

* **Begründung der Struktur:**  
  * mod.rs: Dient als Fassade des Moduls. Es deklariert die ThemingEngine-Struktur und re-exportiert die öffentlich zugänglichen Typen aus types.rs und errors.rs. Hier wird die öffentliche API des Moduls definiert und zugänglich gemacht.  
  * types.rs: Zentralisiert alle theming-spezifischen Datenstrukturen (wie RawToken, ThemeDefinition, AppliedThemeState etc.). Dies verbessert die Übersichtlichkeit und hilft, zyklische Abhängigkeiten zu vermeiden, da diese Typen sowohl von der API (mod.rs) als auch von der internen Logik (logic.rs) benötigt werden.  
  * errors.rs: Enthält ausschließlich die Definition des ThemingError-Enums und eventuell zugehöriger Hilfstypen für Fehler. Dies entspricht der Richtlinie, Fehlerdefinitionen pro Modul zu gruppieren.  
  * logic.rs: Kapselt die gesamte interne Implementierungslogik der Theming-Engine. Dazu gehören das Laden, Parsen und Validieren von Token- und Theme-Dateien, die komplexe Token Resolution Pipeline und die Handhabung von dynamischen Theme-Wechseln. Um die Komplexität zu bewältigen, kann logic.rs selbst wiederum in spezialisierte Untermodule (z.B. token\_parser.rs, token\_resolver.rs) aufgeteilt werden, die jeweils einen spezifischen Teilaspekt der Logik behandeln. Diese internen Module und Funktionen sind nicht Teil der öffentlichen API (pub(crate)).  
  * default\_themes/: Dieses Verzeichnis enthält die JSON-Dateien für das Fallback-Theme und die dafür notwendigen Basis-Tokens. Diese Dateien können zur Kompilierzeit mittels include\_str\! direkt in die Binärdatei eingebettet werden, um sicherzustellen, dass das Fallback-Theme immer verfügbar ist, selbst wenn externe Konfigurationsdateien fehlen oder beschädigt sind.  
  * Cargo.toml: Wäre vorhanden, wenn domain::theming als separates Crate innerhalb eines Rust-Workspace verwaltet wird. In diesem Fall würde es die Abhängigkeiten (wie serde, serde\_json, thiserror, tracing) und Metadaten spezifisch für dieses Crate deklarieren.

Diese Struktur fördert eine klare Trennung der Belange ("Separation of Concerns"): Die API-Definition ist von der Implementierungslogik getrennt, Datentypen sind zentralisiert, und Fehlerbehandlung sowie Ressourcen sind ebenfalls in eigenen Bereichen organisiert. Dies erleichtert neuen Entwicklern den Einstieg und vereinfacht die Wartung und Weiterentwicklung des Moduls.

## **7\. Detaillierter Implementierungsleitfaden (Schritt-für-Schritt)**

Dieser Leitfaden beschreibt die empfohlene Reihenfolge und die Details für die Implementierung des domain::theming-Moduls. Jeder Schritt sollte von umfassenden Unit-Tests begleitet werden, um die Korrektheit der Implementierung sicherzustellen.

### **7.1. Schrittweise Implementierung der Datenstrukturen (Abschnitt 2\)**

1. **Datei erstellen:** domain/theming/types.rs.  
2. **TokenIdentifier implementieren:**  
   * Struct-Definition mit String-Feld.  
   * new()-Methode, as\_str()-Methode.  
   * Ableitungen: Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize.  
   * Implementierung von std::fmt::Display.  
3. **TokenValue implementieren:**  
   * Enum-Definition mit allen Varianten (Color, Dimension,..., Reference(TokenIdentifier)).  
   * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
   * \#\[serde(rename\_all \= "kebab-case")\] Attribut für konsistente JSON-Serialisierung.  
4. **RawToken implementieren:**  
   * Struct-Definition mit Feldern id: TokenIdentifier, value: TokenValue, description: Option\<String\>, group: Option\<String\>.  
   * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
   * \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\] für optionale Felder.  
5. **TokenSet Typalias definieren:**  
   * pub type TokenSet \= std::collections::HashMap\<TokenIdentifier, RawToken\>;  
6. **ThemeIdentifier implementieren:** Analog zu TokenIdentifier.  
7. **ColorSchemeType implementieren:**  
   * Enum-Definition mit Varianten Light, Dark.  
   * Ableitungen: Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize.  
8. **AccentColor implementieren:**  
   * Struct-Definition mit Feldern name: Option\<String\>, value: String.  
   * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
9. **ThemeVariantDefinition implementieren:**  
   * Struct-Definition mit Feldern applies\_to\_scheme: ColorSchemeType, tokens: TokenSet.  
   * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
10. **ThemeDefinition implementieren:**  
    * Struct-Definition mit allen Feldern (id, name, description, author, version, base\_tokens, variants, supported\_accent\_colors).  
    * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
    * \#\[serde(default,...)\] für optionale Felder und Vektoren.  
11. **AppliedThemeState implementieren:**  
    * Struct-Definition mit Feldern theme\_id, color\_scheme, active\_accent\_color, resolved\_tokens: std::collections::HashMap\<TokenIdentifier, String\>.  
    * Ableitungen: Debug, Clone, PartialEq, serde::Serialize. Deserialize ist hier optional, da dieser Zustand typischerweise von der Engine konstruiert wird.  
12. **ThemingConfiguration implementieren:**  
    * Struct-Definition mit Feldern selected\_theme\_id, preferred\_color\_scheme, selected\_accent\_color, custom\_user\_token\_overrides.  
    * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
13. **Unit-Tests für Datenstrukturen:**  
    * Für jede serialisierbare Struktur Tests schreiben, die die korrekte Serialisierung zu JSON und Deserialisierung von JSON überprüfen.  
    * Beispieldaten für JSON-Strings verwenden, die alle Felder und Varianten abdecken.  
    * Korrektheit der serde-Attribute (rename\_all, default, skip\_serializing\_if) verifizieren.  
    * Die Display-Implementierung für TokenIdentifier und ThemeIdentifier testen.

### **7.2. Implementierung des ThemingError Enums (Abschnitt 5\)**

1. **Datei erstellen:** domain/theming/errors.rs.  
2. **Abhängigkeit hinzufügen:** thiserror zur Cargo.toml des domain-Crates (oder des Workspace-Root, falls domain::theming ein eigenes Crate wird, bzw. zum Projekt-Crate).  
   Ini, TOML  
   \[dependencies\]  
   thiserror \= "1.0"  
   serde\_json \= "1.0" // Bereits für Typen benötigt, aber auch für Fehlerquellen relevant  
   \# weitere Abhängigkeiten

3. **ThemingError Enum definieren:**  
   * Das Enum wie in Abschnitt 5.1 spezifiziert implementieren.  
   * Alle Varianten mit den entsprechenden Feldern für Kontextinformationen (Pfade, IDs etc.) definieren.  
   * \#\[error("...")\] Attribute für jede Variante mit aussagekräftigen Fehlermeldungen versehen.  
   * \#\[source\] für gekapselte Fehler und \#\[from\] für automatische Konvertierung von std::io::Error und serde\_json::Error verwenden, wo passend.  
4. **Unit-Tests für ThemingError:**  
   * Tests schreiben, die sicherstellen, dass die Display-Implementierung (generiert durch \#\[error("...")\]) die erwarteten, formatierten Fehlermeldungen erzeugt.  
   * Für Fehler-Varianten, die einen \#\[source\]-Fehler kapseln, testen, ob die source()-Methode den korrekten zugrundeliegenden Fehler zurückgibt.  
   * Testen der From-Implementierungen (generiert durch \#\[from\]), indem Quellfehler manuell erzeugt und in ThemingError konvertiert werden.

### **7.3. Implementierung der Kernlogik-Funktionen und Geschäftsregeln (Abschnitt 3\)**

Diese Funktionen werden typischerweise in domain/theming/logic.rs oder dessen Untermodulen implementiert und als pub(crate) deklariert.

* **7.3.1. Token- und Theme-Definitionen laden, parsen und validieren:**  
  1. **Funktion pub(crate) fn load\_raw\_tokens\_from\_file(path: \&std::path::Path) \-\> Result\<TokenSet, ThemingError\>:**  
     * Datei öffnen und Inhalt lesen (std::fs::read\_to\_string). Fehlerbehandlung für I/O (ThemingError::TokenFileIoError).  
     * JSON-Inhalt parsen (serde\_json::from\_str) zu Vec\<RawToken\>. Fehlerbehandlung für Parsing (ThemingError::TokenFileParseError).  
     * Vec\<RawToken\> in TokenSet (HashMap) konvertieren. Dabei auf doppelte TokenIdentifier prüfen. Bei Duplikaten eine Warnung loggen (tracing::warn\!) und das zuletzt gelesene Token verwenden oder einen Fehler (ThemingError::InvalidTokenData) auslösen, je nach definierter Strategie (z.B. Duplikate innerhalb einer Datei sind ein Fehler).  
     * tracing::debug\! für erfolgreiches Laden verwenden.  
  2. **Funktion pub(crate) fn load\_theme\_definition\_from\_file(path: \&std::path::Path, theme\_id\_from\_path: ThemeIdentifier) \-\> Result\<ThemeDefinition, ThemingError\>:**  
     * Datei öffnen und Inhalt lesen. Fehlerbehandlung (ThemingError::ThemeFileIoError mit theme\_id und path).  
     * JSON-Inhalt parsen zu ThemeDefinition. Fehlerbehandlung (ThemingError::ThemeFileLoadError mit theme\_id und path).  
     * Validieren, ob theme\_def.id mit theme\_id\_from\_path (abgeleitet vom Dateinamen/Pfad) übereinstimmt. Bei Diskrepanz ThemingError::InvalidThemeData.  
  3. **Funktion pub(crate) fn validate\_tokenset\_for\_cycles(tokens: \&TokenSet) \-\> Result\<(), ThemingError\>:**  
     * Implementiert einen Algorithmus zur Zyklenerkennung (z.B. Tiefensuche) für TokenValue::Reference-Beziehungen.  
     * Hält eine Liste der besuchten Tokens während eines Auflösungspfads, um Zyklen zu erkennen.  
     * Gibt bei Zykluserkennung ThemingError::CyclicTokenReference { token\_id } zurück (wobei token\_id das erste im Zyklus erkannte Token ist oder ein Token, das Teil des Zyklus ist).  
  4. **Funktion pub(crate) fn validate\_theme\_definition\_references(theme\_def: \&ThemeDefinition, global\_tokens: \&TokenSet) \-\> Result\<(), ThemingError\>:**  
     * Iteriert durch alle Tokens in theme\_def.base\_tokens und in allen theme\_def.variants\[\*\].tokens.  
     * Für jedes Token, das ein TokenValue::Reference(target\_id) ist, prüfen, ob target\_id entweder in global\_tokens oder in theme\_def.base\_tokens (falls das aktuelle Token aus einer Variante stammt und sich auf ein Basistoken des Themes bezieht) existiert.  
     * Gibt bei einer fehlenden Referenz ThemingError::InvalidThemeData (oder einen spezifischeren Fehler wie MissingThemeTokenReference) zurück.  
  5. **Unit-Tests für Lade- und Validierungsfunktionen:**  
     * Tests mit gültigen JSON-Dateien für Tokens und Themes.  
     * Tests mit fehlerhaften JSON-Dateien (Syntaxfehler, falsche Typen).  
     * Tests mit semantisch ungültigen Daten (z.B. doppelte Token-IDs in einer Datei, zyklische Referenzen in einem TokenSet, fehlende Referenzen in einer ThemeDefinition).  
     * Sicherstellen, dass die korrekten ThemingError-Varianten zurückgegeben werden.  
* **7.3.2. Token Resolution Pipeline implementieren:**  
  1. **Hauptfunktion pub(crate) fn resolve\_applied\_state(config: \&ThemingConfiguration, available\_themes: &, global\_tokens: \&TokenSet) \-\> Result\<AppliedThemeState, ThemingError\>:**  
     * Implementiere die in Abschnitt 3.2 detailliert beschriebenen Schritte:  
       * **Theme auswählen:** Finde die ThemeDefinition für config.selected\_theme\_id in available\_themes. Bei Nichtauffinden ThemingError::ThemeNotFound.  
       * **Initiales Token-Set:** Beginne mit einer Kopie von global\_tokens. Merge (überschreibe) mit selected\_theme.base\_tokens.  
       * **Variante anwenden:** Finde die passende ThemeVariantDefinition für config.preferred\_color\_scheme. Merge deren tokens.  
       * **Akzentfarbe anwenden:** Implementiere die Logik zur Verarbeitung von config.selected\_accent\_color. Für die Erstimplementierung: Ersetze spezielle Token-IDs (z.B. {{ACCENT\_COLOR\_VALUE}} oder token.system.accent) durch accent\_color.value.  
       * **Benutzer-Overrides anwenden:** Merge config.custom\_user\_token\_overrides.  
       * **Referenzen auflösen:** Implementiere eine rekursive Funktion resolve\_references(current\_tokens: \&mut TokenSet, max\_depth: u8) \-\> Result\<(), ThemingError\>. Diese Funktion iteriert, bis keine TokenValue::Reference mehr vorhanden sind oder max\_depth erreicht ist. Sie muss Zyklenerkennung beinhalten (kann validate\_tokenset\_for\_cycles nutzen oder eine eigene Implementierung haben) und Fehler wie ThemingError::MissingTokenReference und ThemingError::MaxReferenceDepthExceeded behandeln.  
       * **Finale Werte konvertieren:** Konvertiere die nun aufgelösten TokenValues in String-Werte für das resolved\_tokens-Feld des AppliedThemeState.  
     * Konstruiere und gib den AppliedThemeState zurück.  
  2. **Hilfsfunktionen:**  
     * merge\_token\_sets(base: \&mut TokenSet, overrides: \&TokenSet): Fügt Tokens aus overrides zu base hinzu, wobei bestehende Tokens in base überschrieben werden.  
  3. **Unit-Tests für die Resolution Pipeline:**  
     * Szenarien mit einfachen Themes ohne Varianten oder Overrides.  
     * Szenarien mit Hell/Dunkel-Varianten.  
     * Szenarien mit Akzentfarben (einfache Ersetzung testen).  
     * Szenarien mit Benutzer-Overrides.  
     * Tests für mehrstufige Token-Referenzen (Aliase).  
     * Explizite Tests für Fehlerfälle: fehlende Referenzen, zyklische Referenzen während der Auflösung, Überschreitung der maximalen Tiefe.  
* **7.3.3. Fallback-Theme Logik:**  
  1. **Fallback-Ressourcen erstellen:** Erstelle domain/theming/default\_themes/fallback.theme.json und domain/theming/default\_themes/base.tokens.json mit minimalen, aber funktionsfähigen Werten. Diese sollten keine externen Referenzen enthalten und in sich geschlossen sein.  
  2. **Funktion pub(crate) fn load\_fallback\_applied\_state() \-\> Result\<AppliedThemeState, ThemingError\>:**  
     * Verwende include\_str\! Makros, um den Inhalt der JSON-Dateien zur Kompilierzeit einzubetten.  
     * Parse die eingebetteten Strings zu ThemeDefinition und TokenSet.  
     * Erzeuge einen AppliedThemeState direkt aus diesen Fallback-Daten (die Auflösung sollte hier trivial sein, da keine komplexen Referenzen erwartet werden).  
     * Diese Funktion sollte robust sein und nur im äußersten Notfall fehlschlagen (z.B. wenn die eingebetteten JSONs fehlerhaft sind, was ein Build-Problem wäre). Ein Fehler hier wäre ThemingError::FallbackThemeLoadError.

### **7.4. Implementierung des ThemingEngine-Service und seiner API (Abschnitt 4\)**

1. **Datei anpassen/erstellen:** domain/theming/mod.rs.  
2. **Strukturen definieren:**  
   * pub struct ThemingEngine { internal\_state: Arc\<Mutex\<ThemingEngineInternalState\>\>, event\_subscribers: Arc\<Mutex\<Vec\<mpsc::Sender\<ThemeChangedEvent\>\>\>\> }  
   * struct ThemingEngineInternalState {... } (Felder wie in 4.1 definiert, inklusive theme\_load\_paths, token\_load\_paths für reload).  
3. **Event-Struktur ThemeChangedEvent in types.rs definieren** (bereits in 7.1, hier nur zur Erinnerung).  
4. **Konstruktor ThemingEngine::new(...) implementieren:**  
   * Initialisiere event\_subscribers mit Arc::new(Mutex::new(Vec::new())).  
   * Initialisiere internal\_state.theme\_load\_paths und internal\_state.token\_load\_paths mit den übergebenen Pfaden.  
   * **Laden der globalen Tokens:** Iteriere über token\_load\_paths, rufe logic::load\_raw\_tokens\_from\_file für jede Datei auf und merge die Ergebnisse in internal\_state.global\_raw\_tokens. Führe logic::validate\_tokenset\_for\_cycles für das finale Set aus.  
   * **Laden der verfügbaren Themes:** Iteriere über theme\_load\_paths, finde \*.theme.json-Dateien, lade sie mit logic::load\_theme\_definition\_from\_file. Validiere jede ThemeDefinition mit logic::validate\_theme\_definition\_references gegen die global\_raw\_tokens. Sammle gültige Themes in internal\_state.available\_themes.  
   * **Anfänglichen Zustand anwenden:**  
     * Versuche, logic::resolve\_applied\_state mit initial\_config, internal\_state.available\_themes und internal\_state.global\_raw\_tokens aufzurufen.  
     * Bei Erfolg: Speichere initial\_config als internal\_state.current\_config und das Ergebnis als internal\_state.applied\_state.  
     * Bei Fehler: Logge den Fehler (tracing::warn\!). Versuche, logic::load\_fallback\_applied\_state() aufzurufen.  
       * Wenn Fallback erfolgreich: Speichere eine entsprechende Fallback-ThemingConfiguration (z.B. mit der ID des Fallback-Themes) und den Fallback-AppliedThemeState.  
       * Wenn Fallback fehlschlägt: Gib ThemingError::FallbackThemeLoadError zurück.  
   * Konstruiere und gib Ok(Self) zurück.  
5. **Implementiere get\_current\_theme\_state():** Sperre internal\_state-Mutex, klone internal\_state.applied\_state, gib Ok(cloned\_state) zurück.  
6. **Implementiere get\_available\_themes():** Sperre Mutex, klone internal\_state.available\_themes, gib Ok(cloned\_list) zurück.  
7. **Implementiere get\_current\_configuration():** Sperre Mutex, klone internal\_state.current\_config, gib Ok(cloned\_config) zurück.  
8. **Implementiere update\_configuration(new\_config: ThemingConfiguration):**  
   * Sperre internal\_state-Mutex.  
   * Speichere den alten applied\_state (für späteren Vergleich).  
   * Rufe logic::resolve\_applied\_state mit new\_config, \&self.internal\_state.available\_themes und \&self.internal\_state.global\_raw\_tokens auf.  
   * Bei Erfolg (Ok(new\_applied\_state)):  
     * Aktualisiere self.internal\_state.current\_config \= new\_config.  
     * Aktualisiere self.internal\_state.applied\_state \= new\_applied\_state.  
     * Wenn self.internal\_state.applied\_state sich vom alten applied\_state unterscheidet:  
       * Erzeuge ThemeChangedEvent { new\_state: self.internal\_state.applied\_state.clone() }.  
       * Sperre event\_subscribers-Mutex. Iteriere über die Sender und sende das geklonte Event. Entferne Sender, bei denen send() fehlschlägt (Kanal geschlossen).  
     * Gib Ok(()) zurück.  
   * Bei Fehler (Err(e)): Gib Err(e) zurück.  
9. **Implementiere reload\_themes\_and\_tokens():**  
   * Sperre internal\_state-Mutex.  
   * Lade globale Tokens und verfügbare Themes neu (wie im Konstruktor, unter Verwendung der gespeicherten theme\_load\_paths und token\_load\_paths). Aktualisiere internal\_state.global\_raw\_tokens und internal\_state.available\_themes. Fehler hierbei sollten geloggt und ggf. zurückgegeben werden.  
   * Speichere den alten applied\_state.  
   * Rufe logic::resolve\_applied\_state mit der *aktuellen* self.internal\_state.current\_config (die nicht geändert wurde) und den neu geladenen Daten auf.  
   * Aktualisiere self.internal\_state.applied\_state und sende Event wie bei update\_configuration, falls eine Änderung vorliegt.  
   * Gib Ok(()) oder den entsprechenden Lade-/Anwendungsfehler zurück.  
10. **Implementiere subscribe\_to\_theme\_changes():**  
    * Erzeuge ein neues mpsc::channel().  
    * Sperre event\_subscribers-Mutex. Füge den sender-Teil des Kanals zur Liste self.event\_subscribers hinzu.  
    * Gib Ok(receiver) zurück.  
11. **Unit-Tests für ThemingEngine:**  
    * **new():** Teste erfolgreiche Initialisierung mit gültigen Konfigurationen und Pfaden. Teste das Fallback-Verhalten, wenn initiale Konfigurationen fehlerhaft sind oder Pfade ungültig. Teste kritischen Fehler, wenn selbst Fallback fehlschlägt.  
    * **get\_\*() Methoden:** Teste, ob die korrekten Daten (Klone des internen Zustands) zurückgegeben werden.  
    * **update\_configuration():** Teste erfolgreiche Zustandsänderungen. Verifiziere, dass der applied\_state korrekt aktualisiert wird. Teste, dass ThemeChangedEvent nur gesendet wird, wenn sich der applied\_state tatsächlich ändert. Teste Fehlerfälle (z.B. ungültige ThemeIdentifier in new\_config).  
    * **reload\_themes\_and\_tokens():** Erstelle temporäre Theme-/Token-Dateien, modifiziere sie und teste, ob reload die Änderungen korrekt aufnimmt und den Zustand aktualisiert. Teste Event-Auslösung.  
    * **Event-System (subscribe\_to\_theme\_changes und Senden):** Registriere mehrere Subscriber. Löse eine Zustandsänderung aus und verifiziere, dass alle aktiven Subscriber das Event empfangen. Teste, dass Subscriber, deren Receiver fallengelassen wurde, korrekt aus der internen Liste entfernt werden und keine Fehler verursachen.  
    * **Thread-Sicherheit (konzeptionell):** Obwohl direkte Unit-Tests für Thread-Sicherheit komplex sind, stelle sicher, dass alle Zugriffe auf internal\_state und event\_subscribers korrekt durch Mutexe geschützt sind. Integrationstests könnten parallele Aufrufe simulieren.

### **7.5. Richtlinien für Unit-Tests (Zusammenfassung)**

* **Hohe Codeabdeckung:** Strebe eine hohe Testabdeckung für alle Logik-Komponenten in logic.rs und alle öffentlichen API-Methoden der ThemingEngine in mod.rs an.  
* **Fokus der Testfälle:**  
  * **Parsing und Validierung:** Korrekte Verarbeitung gültiger und ungültiger Eingabedaten (JSON-Dateien, Token-Strukturen).  
  * **Token-Auflösung:** Korrekte Auflösung von einfachen und komplexen Token-Referenzen (Aliase, Vererbung). Explizite Tests für Fehlerfälle wie fehlende Referenzen und zyklische Abhängigkeiten.  
  * **Theme-Anwendung:** Korrekte Anwendung von Basis-Themes, Varianten (Hell/Dunkel), Akzentfarben und Benutzer-Overrides.  
  * **ThemingEngine-Verhalten:** Korrekte Zustandsübergänge, Event-Auslösung und Fehlerbehandlung für alle API-Methoden.  
  * **Grenzwertanalyse:** Teste Randbedingungen (z.B. leere Token-Sets, Themes ohne Varianten, maximale Rekursionstiefe bei Referenzen).  
* **Testdaten und Fixtures:** Verwende kleine, fokussierte JSON-Beispieldateien für Tokens und Themes als Test-Fixtures. Diese können als Strings direkt in die Testfunktionen eingebettet oder aus einem Test-Ressourcenverzeichnis geladen werden.  
* **Mocking:** Für dieses Modul der Domänenschicht ist Mocking von externen Abhängigkeiten (hauptsächlich das Dateisystem) in der Regel nicht notwendig für Unit-Tests der Kernlogik. Die Ladefunktionen können mit temporären Dateien oder In-Memory-Daten getestet werden. Der Fokus liegt auf der internen Verarbeitungslogik.  
* **Testorganisation:** Unit-Tests sollten direkt neben dem zu testenden Code in Untermodulen tests liegen (\#\[cfg(test)\] mod tests {... }).

Durch die konsequente Befolgung dieses Implementierungsleitfadens und die sorgfältige Erstellung von Unit-Tests kann ein robustes, korrekt funktionierendes und wartbares domain::theming-Modul entwickelt werden.

---

# **Domänenschicht: Implementierungsleitfaden Teil 2/4 – Workspaces (domain::workspaces)**

## **1\. Einleitung zur Komponente domain::workspaces**

Die Komponente domain::workspaces ist ein zentraler Bestandteil der Domänenschicht und verantwortlich für die gesamte Logik und Verwaltung von Arbeitsbereichen, oft als "Spaces" oder virtuelle Desktops bezeichnet. Sie definiert die Struktur eines einzelnen Workspace, die Regeln für die Zuweisung von Fenstern zu Workspaces, die Orchestrierung aller Workspaces inklusive des aktiven Workspace und die Persistenz der Workspace-Konfiguration. Diese Komponente ist UI-unabhängig und stellt ihre Funktionalität über klar definierte Schnittstellen bereit, die von der System- und Benutzeroberflächenschicht genutzt werden können.  
Die Implementierung ist in vier primäre Module unterteilt, um eine hohe Kohäsion und lose Kopplung zu gewährleisten:

* workspaces::core: Definiert die grundlegende Entität eines Workspace und zugehörige Typen.  
* workspaces::assignment: Beinhaltet die Logik für die Zuweisung von Fenstern zu Workspaces.  
* workspaces::manager: Orchestriert die Verwaltung aller Workspaces und publiziert relevante Events.  
* workspaces::config: Verantwortlich für das Laden und Speichern der Workspace-Konfiguration.

Dieser Implementierungsleitfaden spezifiziert jedes dieser Module im Detail, einschließlich Datenstrukturen, APIs, Fehlerbehandlung und Implementierungsschritten, um eine direkte Umsetzung durch das Entwicklungsteam zu ermöglichen.

## **2\. Entwicklungsmodul 1: workspaces::core – Fundamentale Workspace-Definition**

Das Modul workspaces::core legt das Fundament für das Workspace-System, indem es die Kernentität Workspace sowie die damit verbundenen grundlegenden Datentypen und Fehlerdefinitionen bereitstellt.

### **2.1. Verantwortlichkeiten und Design-Rationale**

Dieses Modul ist ausschließlich dafür zuständig, die intrinsischen Eigenschaften und das Verhalten eines einzelnen, isolierten Workspace zu definieren. Es kapselt Attribute wie Name, ID, Layout-Typ und die Menge der zugeordneten Fensteridentifikatoren. Die Design-Entscheidung, diese Kernfunktionalität zu isolieren, stellt sicher, dass die grundlegende Definition eines Workspace unabhängig von komplexerer Verwaltungs- oder Zuweisungslogik bleibt, was die Wartbarkeit und Testbarkeit des Moduls verbessert. Es hat keine Kenntnis von anderen Workspaces oder dem Konzept eines "aktiven" Workspace.

### **2.2. Datentypen und Entitäten**

Die folgenden Rust-Datentypen sind für die Definition eines Workspace und seiner Attribute spezifiziert.

#### **2.2.1. Struct: Workspace**

Das Workspace-Struct repräsentiert einen einzelnen Arbeitsbereich.

* **Rust-Definition:**  
  Rust  
  // src/domain/workspaces/core/mod.rs  
  use std::collections::HashSet;  
  use uuid::Uuid;  
  use crate::domain::workspaces::core::types::{WorkspaceId, WindowIdentifier, WorkspaceLayoutType};  
  use crate::domain::workspaces::core::errors::WorkspaceCoreError;

  \#  
  pub struct Workspace {  
      id: WorkspaceId,  
      name: String,  
      persistent\_id: Option\<String\>, // Für Persistenz über Sitzungen hinweg  
      layout\_type: WorkspaceLayoutType,  
      window\_ids: HashSet\<WindowIdentifier\>, // IDs der Fenster auf diesem Workspace  
      created\_at: chrono::DateTime\<chrono::Utc\>, // Zeitstempel der Erstellung  
  }

* **Attribute und deren Bedeutung:**  
  * id: WorkspaceId: Ein eindeutiger Laufzeit-Identifikator für den Workspace, generiert bei der Erstellung (z.B. mittels uuid::Uuid::new\_v4()).  
  * name: String: Der vom Benutzer definierbare oder automatisch generierte Name des Workspace (z.B. "Arbeit", "Workspace 1").  
    * Invarianten: Darf nicht leer sein. Muss eine maximale Länge (z.B. 255 Zeichen) einhalten. Validierung erfolgt bei Erstellung und Umbenennung.  
  * persistent\_id: Option\<String\>: Eine optionale, eindeutige ID, die über Sitzungen hinweg stabil bleibt und zum Wiederherstellen von Workspaces verwendet wird. Kann vom Benutzer festgelegt oder automatisch generiert werden.  
    * Invarianten: Falls Some, darf der String nicht leer sein und sollte bestimmten Formatierungsregeln folgen (z.B. keine Sonderzeichen, um Dateisystem- oder Konfigurationsprobleme zu vermeiden).  
  * layout\_type: WorkspaceLayoutType: Definiert das aktuelle Layout-Verhalten für Fenster auf diesem Workspace (z.B. Floating, TilingHorizontal).  
  * window\_ids: HashSet\<WindowIdentifier\>: Eine Menge von eindeutigen Identifikatoren für Fenster, die aktuell diesem Workspace zugeordnet sind. Die Reihenfolge der Fenster ist hier nicht relevant; diese wird ggf. von der Systemschicht (Compositor) oder domain::window\_management verwaltet.  
  * created\_at: chrono::DateTime\<chrono::Utc\>: Der Zeitstempel der Erstellung des Workspace-Objekts.

#### **2.2.2. Struct: WindowIdentifier**

Ein Newtype für Fensteridentifikatoren zur Verbesserung der Typsicherheit.

* **Rust-Definition:**  
  Rust  
  // src/domain/workspaces/core/types.rs  
  \#  
  pub struct WindowIdentifier(String);

  impl WindowIdentifier {  
      pub fn new(id: String) \-\> Result\<Self, &'static str\> {  
          if id.is\_empty() {  
              Err("WindowIdentifier cannot be empty")  
          } else {  
              Ok(Self(id))  
          }  
      }

      pub fn as\_str(\&self) \-\> \&str {  
          \&self.0  
      }  
  }

  impl From\<String\> for WindowIdentifier {  
      fn from(s: String) \-\> Self {  
          // In einem realen Szenario könnte hier eine Validierung stattfinden oder  
          // es wird davon ausgegangen, dass der String bereits validiert ist.  
          // Für die einfache Konvertierung wird hier keine Validierung erzwungen,  
          // die \`new\` Methode ist für explizite Validierung vorgesehen.  
          Self(s)  
      }  
  }

  impl std::fmt::Display for WindowIdentifier {  
      fn fmt(\&self, f: \&mut std::fmt::Formatter\<'\_\>) \-\> std::fmt::Result {  
          write\!(f, "{}", self.0)  
      }  
  }

* **Verwendung:** Repräsentiert einen eindeutigen Identifikator für ein Fenster. Dieser Identifikator wird typischerweise von der Systemschicht (z.B. als Wayland Surface ID oder eine interne Anwendungs-ID) vergeben. Die Domänenschicht behandelt diesen Identifikator als einen opaken Wert, dessen genaues Format und Ursprung für die Logik innerhalb von domain::workspaces nicht von primärer Bedeutung sind, solange er Eindeutigkeit gewährleistet.  
* **Invarianten:** Der interne String darf nicht leer sein. Diese Invariante wird durch die new-Methode sichergestellt.

#### **2.2.3. Enum: WorkspaceLayoutType**

Definiert die möglichen Layout-Modi eines Workspace.

* **Rust-Definition:**  
  Rust  
  // src/domain/workspaces/core/types.rs  
  \#  
  pub enum WorkspaceLayoutType {  
      Floating,  
      TilingHorizontal,  
      TilingVertical,  
      Maximized, // Ein einzelnes Fenster ist maximiert, andere sind ggf. verborgen oder minimiert  
  }

  impl Default for WorkspaceLayoutType {  
      fn default() \-\> Self {  
          WorkspaceLayoutType::Floating  
      }  
  }

* **Verwendung:** Steuert, wie Fenster innerhalb des Workspace standardmäßig angeordnet oder verwaltet werden. Die konkrete Implementierung der Layout-Logik erfolgt in domain::window\_management und der Systemschicht, basierend auf diesem Typ.  
* **Standardwert:** Floating.

#### **2.2.4. Typalias: WorkspaceId**

Ein Typalias für die ID eines Workspace zur Verbesserung der Lesbarkeit und Konsistenz.

* **Rust-Definition:**  
  Rust  
  // src/domain/workspaces/core/types.rs  
  pub type WorkspaceId \= uuid::Uuid;

### **2.3. Öffentliche API: Methoden und Funktionen**

Alle hier definierten Methoden sind Teil der impl Workspace {... }.  
**Tabelle: API-Methoden für workspaces::core::Workspace**

| Methode (Rust-Signatur) | Kurzbeschreibung | Vorbedingungen | Nachbedingungen | Ausgelöste Events (indirekt) | Mögliche Fehler (WorkspaceCoreError) |
| :---- | :---- | :---- | :---- | :---- | :---- |
| pub fn new(name: String, persistent\_id: Option\<String\>) \-\> Result\<Self, WorkspaceCoreError\> | Erstellt einen neuen Workspace. | name darf nicht leer sein und muss die Längenbeschränkung einhalten. persistent\_id (falls Some) muss gültig sein. | Ein neues Workspace-Objekt wird mit einer eindeutigen id und created\_at Zeitstempel initialisiert. | \- | InvalidName, NameCannotBeEmpty, NameTooLong, InvalidPersistentId |
| pub fn id(\&self) \-\> WorkspaceId | Gibt die eindeutige Laufzeit-ID des Workspace zurück. | \- | \- | \- | \- |
| pub fn name(\&self) \-\> \&str | Gibt den aktuellen Namen des Workspace zurück. | \- | \- | \- | \- |
| pub fn rename(\&mut self, new\_name: String) \-\> Result\<(), WorkspaceCoreError\> | Benennt den Workspace um. | new\_name darf nicht leer sein und muss die Längenbeschränkung einhalten. | Der name des Workspace ist auf new\_name gesetzt. | WorkspaceRenamed (via manager) | InvalidName, NameCannotBeEmpty, NameTooLong |
| pub fn layout\_type(\&self) \-\> WorkspaceLayoutType | Gibt den aktuellen Layout-Typ des Workspace zurück. | \- | \- | \- | \- |
| pub fn set\_layout\_type(\&mut self, layout\_type: WorkspaceLayoutType) \-\> () | Setzt den Layout-Typ des Workspace. | \- | Der layout\_type des Workspace ist auf den übergebenen Wert gesetzt. | WorkspaceLayoutChanged (via manager) | \- |
| pub(crate) fn add\_window\_id(\&mut self, window\_id: WindowIdentifier) \-\> bool | Fügt eine Fenster-ID zur Menge der Fenster auf diesem Workspace hinzu. Intern verwendet vom assignment-Modul. | \- | window\_id ist in window\_ids enthalten. Gibt true zurück, wenn die ID neu hinzugefügt wurde, sonst false. | WindowAddedToWorkspace (via manager) | \- |
| pub(crate) fn remove\_window\_id(\&mut self, window\_id: \&WindowIdentifier) \-\> bool | Entfernt eine Fenster-ID aus der Menge der Fenster auf diesem Workspace. Intern verwendet vom assignment-Modul. | \- | window\_id ist nicht mehr in window\_ids enthalten. Gibt true zurück, wenn die ID entfernt wurde, sonst false. | WindowRemovedFromWorkspace (via manager) | \- |
| pub fn window\_ids(\&self) \-\> \&HashSet\<WindowIdentifier\> | Gibt eine unveränderliche Referenz auf die Menge der Fenster-IDs zurück. | \- | \- | \- | \- |
| pub fn persistent\_id(\&self) \-\> Option\<\&str\> | Gibt die optionale persistente ID des Workspace zurück. | \- | \- | \- | \- |
| pub fn set\_persistent\_id(\&mut self, pid: Option\<String\>) \-\> Result\<(), WorkspaceCoreError\> | Setzt oder entfernt die persistente ID des Workspace. | pid (falls Some) muss gültig sein. | Die persistent\_id des Workspace ist entsprechend gesetzt. | \- | InvalidPersistentId |
| pub fn created\_at(\&self) \-\> chrono::DateTime\<chrono::Utc\> | Gibt den Erstellungszeitstempel des Workspace zurück. | \- | \- | \- | \- |

Diese Tabelle definiert die exakte Schnittstelle für die Interaktion mit einem Workspace-Objekt. Die präzise Spezifikation von Signaturen, Vor- und Nachbedingungen sowie potenziellen Fehlern ist entscheidend für eine korrekte Implementierung und Nutzung durch andere Systemkomponenten.

### **2.4. Interner Zustand und Lebenszyklusmanagement**

Ein Workspace-Objekt wird typischerweise vom workspaces::manager-Modul erstellt und dessen Lebensdauer von diesem verwaltet. Es besitzt keinen komplexen internen Zustandsautomaten; sein Zustand wird vollständig durch seine Attribute (Felder des Structs) definiert. Änderungen am Zustand erfolgen durch Aufruf der in Abschnitt 2.3 definierten Methoden.

### **2.5. Events: Definition und Semantik (Event-Datenstrukturen)**

Das Modul workspaces::core definiert selbst keine Event-Enums und ist auch nicht für das Publizieren von Events zuständig. Es stellt jedoch die Datenstrukturen (Payloads) bereit, die von höherliegenden Modulen (insbesondere workspaces::manager) verwendet werden, um den Inhalt von Events zu definieren, die sich auf Änderungen an Workspace-Objekten beziehen.

* Beispielhafte Event-Datenstrukturen (Payloads):  
  Diese Strukturen werden im Untermodul event\_data definiert (src/domain/workspaces/core/event\_data.rs).  
  Rust  
  // src/domain/workspaces/core/event\_data.rs  
  use crate::domain::workspaces::core::types::{WorkspaceId, WindowIdentifier, WorkspaceLayoutType};

  \#  
  pub struct WorkspaceRenamedData {  
      pub id: WorkspaceId,  
      pub old\_name: String,  
      pub new\_name: String,  
  }

  \#  
  pub struct WorkspaceLayoutChangedData {  
      pub id: WorkspaceId,  
      pub old\_layout: WorkspaceLayoutType,  
      pub new\_layout: WorkspaceLayoutType,  
  }

  \#  
  pub struct WindowAddedToWorkspaceData {  
      pub workspace\_id: WorkspaceId,  
      pub window\_id: WindowIdentifier,  
  }

  \#  
  pub struct WindowRemovedFromWorkspaceData {  
      pub workspace\_id: WorkspaceId,  
      pub window\_id: WindowIdentifier,  
  }

  \#  
  pub struct WorkspacePersistentIdChangedData {  
      pub id: WorkspaceId,  
      pub old\_persistent\_id: Option\<String\>,  
      pub new\_persistent\_id: Option\<String\>,  
  }

  Die eigentlichen Event-Enums (z.B. WorkspaceEvent), die diese Datenstrukturen verwenden, werden im workspaces::manager-Modul definiert.

### **2.6. Fehlerbehandlung: WorkspaceCoreError**

Für die Fehlerbehandlung innerhalb des workspaces::core-Moduls wird ein spezifisches Error-Enum WorkspaceCoreError definiert. Dieses Enum nutzt das thiserror-Crate, um die Erstellung idiomatischer Fehlertypen zu vereinfachen, wie in Richtlinie 4.3 der Gesamtspezifikation und basierend auf etablierten Praktiken 1 empfohlen.

* **Definition:**  
  Rust  
  // src/domain/workspaces/core/errors.rs  
  use thiserror::Error;  
  use crate::core::errors::ValidationError; // Annahme: Ein allgemeiner Validierungsfehler aus der Kernschicht

  pub const MAX\_WORKSPACE\_NAME\_LENGTH: usize \= 64; // Beispielhafte Maximallänge

  \#  
  pub enum WorkspaceCoreError {  
      \#  
      InvalidName(String), // Enthält den ungültigen Namen

      \#\[error("Workspace name cannot be empty.")\]  
      NameCannotBeEmpty,

      \#\[error("Workspace name exceeds maximum length of {max\_len} characters: '{name}' is {actual\_len} characters long.")\]  
      NameTooLong { name: String, max\_len: usize, actual\_len: usize },

      \#  
      InvalidPersistentId(String), // Enthält die ungültige ID

      \#\[error("A core validation rule was violated: {0}")\]  
      ValidationError(\#\[from\] ValidationError), // Ermöglicht das Wrapping von Fehlern aus der Kernschicht

      \#\[error("An internal error occurred in workspace core logic: {context}")\]  
      Internal { context: String }, // Für unerwartete interne Fehlerzustände  
  }

* Erläuterung und Anwendung von Fehlerbehandlungsprinzipien:  
  Die Gestaltung von WorkspaceCoreError folgt mehreren wichtigen Prinzipien der Fehlerbehandlung in Rust:  
  1. **Spezifität und Kontext:** Jede Variante des Enums repräsentiert einen klar definierten Fehlerfall, der innerhalb des workspaces::core-Moduls auftreten kann. Varianten wie InvalidName(String) und NameTooLong { name, max\_len, actual\_len } enthalten die problematischen Werte oder relevanten Kontextinformationen direkt im Fehlertyp. Dies ist entscheidend, um das Problem des "Context Blurring" zu vermeiden, bei dem ein generischer Fehlertyp nicht genügend Informationen über die Fehlerursache liefert.1 Durch die Aufnahme dieser Daten kann der aufrufende Code nicht nur den Fehlertyp programmatisch behandeln, sondern auch detaillierte Fehlermeldungen für Benutzer oder Entwickler generieren.  
  2. **thiserror für Ergonomie:** Die Verwendung von \#\[derive(Error)\] und dem \#\[error("...")\]-Attribut von thiserror reduziert Boilerplate-Code erheblich und stellt sicher, dass das std::error::Error-Trait korrekt implementiert wird, inklusive einer sinnvollen Display-Implementierung.1  
  3. **Fehler-Wrapping mit \#\[from\]:** Die Variante ValidationError(\#\[from\] ValidationError) demonstriert die Nutzung von \#\[from\]. Dies ermöglicht die automatische Konvertierung eines ValidationError (aus crate::core::errors) in einen WorkspaceCoreError mittels des ?-Operators. Entscheidend ist hierbei, dass die source()-Methode des Error-Traits automatisch so implementiert wird, dass der ursprüngliche ValidationError als Ursache des WorkspaceCoreError zugänglich bleibt.3 Dies ist für die Fehlerdiagnose über Modulgrenzen hinweg unerlässlich.  
  4. **Vermeidung von Panics:** Die API-Methoden von Workspace geben Result\<\_, WorkspaceCoreError\> zurück. Dies stellt sicher, dass vorhersehbare Fehlerzustände (z.B. ungültige Eingaben) explizit behandelt und nicht durch panic\! abgebrochen werden, was für Bibliotheks- und Domänencode als Best Practice gilt.4  
  5. **Klare Fehlernachrichten:** Die \#\[error("...")\]-Nachrichten sind primär für Entwickler konzipiert (z.B. für Logging und Debugging). Sie sind präzise und beschreiben das technische Problem. Die Benutzeroberflächenschicht ist dafür verantwortlich, diese technischen Fehler gegebenenfalls in benutzerfreundlichere Meldungen zu übersetzen.

Ein wichtiger Aspekt bei der Fehlerdefinition ist die Balance zwischen der Anzahl der Fehlervarianten und der Notwendigkeit, spezifische Informationen für die Fehlerbehandlung bereitzustellen. Wenn ein generischer Fehler wie ValidationError aus einer tieferen Schicht stammt, ist es oft nicht ausreichend, ihn einfach nur zu wrappen. Wenn der Kontext, *welche* spezifische Validierung innerhalb von workspaces::core fehlgeschlagen ist, für den Aufrufer relevant ist, sollte eine spezifischere Variante in WorkspaceCoreError in Betracht gezogen werden. Alternativ kann die Internal { context: String }-Variante genutzt werden, wobei context die fehlgeschlagene Operation detailliert beschreibt. Entwickler müssen beim Mappen von Fehlern (z.B. mittels map\_err) darauf achten, präzise Kontextinformationen hinzuzufügen, falls \#\[from\] allein nicht genügend semantische Information transportiert.  
**Tabelle: WorkspaceCoreError Varianten**

| Variante | \#\[error("...")\]-Meldung (Auszug) | Semantische Bedeutung/Ursache | Enthaltene Datenfelder | Mögliche Quellfehler (source()) |
| :---- | :---- | :---- | :---- | :---- |
| InvalidName(String) | "Invalid workspace name: {0}..." | Der angegebene Workspace-Name ist ungültig (z.B. aufgrund von Formatierungsregeln, die über Leerstring/Länge hinausgehen). | Der ungültige Name (String). | \- |
| NameCannotBeEmpty | "Workspace name cannot be empty." | Es wurde versucht, einen Workspace mit einem leeren Namen zu erstellen oder einen bestehenden Workspace in einen leeren Namen umzubenennen. | \- | \- |
| NameTooLong | "Workspace name exceeds maximum length..." | Der angegebene Name überschreitet die definierte Maximallänge. | name: String, max\_len: usize, actual\_len: usize. | \- |
| InvalidPersistentId(String) | "Persistent ID is invalid: {0}..." | Die angegebene persistente ID ist ungültig (z.B. leer oder falsches Format). | Die ungültige ID (String). | \- |
| ValidationError(\#\[from\] ValidationError) | "A core validation rule was violated: {0}" | Eine allgemeine Validierungsregel aus der Kernschicht wurde verletzt. | Der ursprüngliche ValidationError. | ValidationError |
| Internal { context: String } | "An internal error occurred..." | Ein unerwarteter Fehler oder eine nicht behandelte Bedingung innerhalb der Modullogik. | context: String (Beschreibung des internen Fehlers). | Variiert |

Diese Tabelle dient Entwicklern als Referenz, um die möglichen Fehlerursachen im workspaces::core-Modul zu verstehen und eine robuste Fehlerbehandlung in aufrufenden Modulen zu implementieren.

### **2.7. Detaillierte Implementierungsschritte und Dateistruktur**

* **Dateistruktur innerhalb von src/domain/workspaces/core/:**  
  * mod.rs: Enthält die Definition des Workspace-Structs und die Implementierung seiner Methoden (impl Workspace). Exportiert öffentliche Typen und Module.  
  * types.rs: Beinhaltet die Definitionen von WorkspaceId, WindowIdentifier und WorkspaceLayoutType.  
  * errors.rs: Enthält die Definition des WorkspaceCoreError-Enums und zugehörige Konstanten wie MAX\_WORKSPACE\_NAME\_LENGTH.  
  * event\_data.rs: Enthält die Definitionen der Event-Payload-Strukturen (z.B. WorkspaceRenamedData).  
* **Implementierungsschritte:**  
  1. Definiere die Typen WorkspaceId, WindowIdentifier (inkl. new, as\_str, From\<String\>, Display) und WorkspaceLayoutType (inkl. Default) in types.rs.  
  2. Definiere das WorkspaceCoreError-Enum in errors.rs gemäß der Spezifikation in Abschnitt 2.6. Implementiere die Konstante MAX\_WORKSPACE\_NAME\_LENGTH.  
  3. Definiere die Event-Payload-Strukturen (z.B. WorkspaceRenamedData, WorkspaceLayoutChangedData, etc.) in event\_data.rs.  
  4. Implementiere das Workspace-Struct in mod.rs mit allen Attributen wie in Abschnitt 2.2.1 spezifiziert.  
  5. Implementiere die Methode pub fn new(name: String, persistent\_id: Option\<String\>) \-\> Result\<Self, WorkspaceCoreError\>:  
     * Validiere name: Prüfe auf Leerstring (Fehler: NameCannotBeEmpty) und Überschreitung von MAX\_WORKSPACE\_NAME\_LENGTH (Fehler: NameTooLong). Ggf. weitere Validierungen für InvalidName.  
     * Validiere persistent\_id (falls Some): Prüfe auf Leerstring und ggf. Format (Fehler: InvalidPersistentId).  
     * Initialisiere id mit Uuid::new\_v4().  
     * Initialisiere created\_at mit chrono::Utc::now().  
     * Initialisiere window\_ids als leeres HashSet.  
     * Initialisiere layout\_type mit WorkspaceLayoutType::default().  
     * Gib bei Erfolg Ok(Self {... }) zurück.  
  6. Implementiere alle Getter-Methoden (id(), name(), layout\_type(), window\_ids(), persistent\_id(), created\_at()) als einfache Rückgaben der entsprechenden Felder.  
  7. Implementiere pub fn rename(\&mut self, new\_name: String) \-\> Result\<(), WorkspaceCoreError\>:  
     * Validiere new\_name analog zur new()-Methode.  
     * Bei Erfolg: self.name \= new\_name; Ok(()).  
  8. Implementiere pub fn set\_layout\_type(\&mut self, layout\_type: WorkspaceLayoutType) \-\> (): self.layout\_type \= layout\_type;.  
  9. Implementiere pub fn set\_persistent\_id(\&mut self, pid: Option\<String\>) \-\> Result\<(), WorkspaceCoreError\>:  
     * Validiere pid (falls Some) analog zur new()-Methode.  
     * Bei Erfolg: self.persistent\_id \= pid; Ok(()).  
  10. Implementiere die pub(crate) Methoden add\_window\_id(\&mut self, window\_id: WindowIdentifier) \-\> bool und remove\_window\_id(\&mut self, window\_id: \&WindowIdentifier) \-\> bool unter Verwendung der entsprechenden HashSet-Methoden (insert bzw. remove) und gib deren booleschen Rückgabewert zurück.  
  11. Stelle sicher, dass alle öffentlichen Typen, Methoden und Felder (falls öffentlich) umfassend mit rustdoc-Kommentaren dokumentiert sind. Die Kommentare müssen Vor- und Nachbedingungen, ausgelöste Fehler (mit Verweis auf die WorkspaceCoreError-Varianten) und ggf. Code-Beispiele enthalten, gemäß Richtlinie 4.7 der Gesamtspezifikation.  
  12. Erstelle Unit-Tests im Untermodul tests (d.h. \#\[cfg(test)\] mod tests {... }) innerhalb von mod.rs. Teste jede Methode gründlich, insbesondere:  
      * Erfolgreiche Erstellung von Workspace-Objekten.  
      * Fehlerfälle bei der Erstellung (ungültige Namen, ungültige persistente IDs).  
      * Erfolgreiche Umbenennung und Fehlerfälle dabei.  
      * Setzen und Abrufen des Layout-Typs.  
      * Setzen und Abrufen der persistenten ID und Fehlerfälle dabei.  
      * Hinzufügen und Entfernen von Fenster-IDs, inklusive Überprüfung der Rückgabewerte und des Zustands von window\_ids.  
      * Überprüfung der Invarianten (z.B. dass id und created\_at korrekt initialisiert werden).

## **3\. Entwicklungsmodul 2: workspaces::assignment – Logik zur Fensterzuweisung**

Das Modul workspaces::assignment ist für die spezifische Geschäftslogik zuständig, die das Zuweisen von Fenstern zu Workspaces und das Entfernen von Fenstern aus Workspaces regelt.

### **3.1. Verantwortlichkeiten und Design-Rationale**

Die Hauptverantwortung dieses Moduls liegt in der Implementierung der Regeln und Operationen, die steuern, wie Fenster (repräsentiert durch WindowIdentifier) Workspaces zugeordnet werden. Dies beinhaltet die Durchsetzung von Regeln wie "ein Fenster darf nur einem Workspace gleichzeitig zugewiesen sein" (falls diese Regel gilt). Das Modul agiert als Dienstleister für den workspaces::manager, der die übergeordnete Workspace-Sammlung hält.  
Die Auslagerung dieser Logik in ein eigenes Modul dient mehreren Zwecken:

* **Trennung der Belange (Separation of Concerns):** Das workspaces::core-Modul bleibt fokussiert auf die Definition eines einzelnen Workspace, während workspaces::manager sich um die Verwaltung der Sammlung und Lebenszyklen kümmert. workspaces::assignment spezialisiert sich auf die Interaktionslogik zwischen Fenstern und Workspaces.  
* **Komplexitätsmanagement:** Regeln für Fensterzuweisungen können komplex werden (z.B. automatische Zuweisung basierend auf Fenstertyp, Anwendungsregeln). Ein dediziertes Modul erleichtert die Handhabung dieser Komplexität.  
* **Testbarkeit:** Die Zuweisungslogik kann isoliert getestet werden.

Dieses Modul interagiert eng mit workspaces::core (um Fenster-IDs in einem Workspace-Objekt zu modifizieren) und wird typischerweise vom workspaces::manager aufgerufen.

### **3.2. Datenstrukturen und Interaktionen**

Dieses Modul operiert primär mit Workspace-Instanzen (aus workspaces::core) und WindowIdentifier-Typen. Es führt selbst keine persistenten Datenstrukturen ein, sondern modifiziert die ihm übergebenen Workspace-Objekte. Für seine Operationen benötigt es Zugriff auf die Sammlung aller relevanten Workspaces, die typischerweise vom workspaces::manager als HashMap\<WorkspaceId, Workspace\> bereitgestellt wird.  
Spezifische temporäre Datenstrukturen könnten hier definiert werden, falls komplexe Zuweisungsalgorithmen (z.B. für automatische Platzierung in Tiling-Layouts) implementiert werden müssten. Für die grundlegende Zuweisung eines Fensters zu einem bestimmten Workspace sind solche Strukturen jedoch in der Regel nicht erforderlich. Die Logik für Layout-spezifische Platzierung ist eher im Modul domain::window\_management angesiedelt.

### **3.3. Öffentliche API: Methoden und Funktionen**

Die Funktionalität dieses Moduls wird durch freistehende Funktionen bereitgestellt, die auf einer veränderbaren Sammlung von Workspaces operieren. Diese Funktionen befinden sich im Modul domain::workspaces::assignment.  
**Tabelle: API-Funktionen für workspaces::assignment**

| Funktion (Rust-Signatur) | Kurzbeschreibung | Vorbedingungen | Nachbedingungen | Ausgelöste Events (indirekt) | Mögliche Fehler (WindowAssignmentError) |
| :---- | :---- | :---- | :---- | :---- | :---- |
| pub fn assign\_window\_to\_workspace(workspaces: \&mut std::collections::HashMap\<WorkspaceId, Workspace\>, target\_workspace\_id: WorkspaceId, window\_id: \&WindowIdentifier, ensure\_unique\_assignment: bool) \-\> Result\<(), WindowAssignmentError\> | Weist ein Fenster einem spezifischen Workspace zu. | target\_workspace\_id muss als Schlüssel in workspaces existieren. | Das Fenster window\_id ist dem Workspace target\_workspace\_id zugeordnet. Falls ensure\_unique\_assignment true ist, wird das Fenster von allen anderen Workspaces in der workspaces-Sammlung entfernt. | WindowAddedToWorkspace, WindowRemovedFromWorkspace (via manager) | WorkspaceNotFound (für target\_workspace\_id), WindowAlreadyAssigned (falls bereits auf Ziel-WS und ensure\_unique\_assignment ist false oder irrelevant), RuleViolation |
| pub fn remove\_window\_from\_workspace(workspaces: \&mut std::collections::HashMap\<WorkspaceId, Workspace\>, source\_workspace\_id: WorkspaceId, window\_id: \&WindowIdentifier) \-\> Result\<bool, WindowAssignmentError\> | Entfernt ein Fenster von einem spezifischen Workspace. | source\_workspace\_id muss als Schlüssel in workspaces existieren. | Das Fenster window\_id ist nicht mehr dem Workspace source\_workspace\_id zugeordnet. Gibt true zurück, wenn das Fenster entfernt wurde, false wenn es nicht auf dem Workspace war. | WindowRemovedFromWorkspace (via manager) | WorkspaceNotFound (für source\_workspace\_id) |
| pub fn move\_window\_to\_workspace(workspaces: \&mut std::collections::HashMap\<WorkspaceId, Workspace\>, source\_workspace\_id: WorkspaceId, target\_workspace\_id: WorkspaceId, window\_id: \&WindowIdentifier) \-\> Result\<(), WindowAssignmentError\> | Verschiebt ein Fenster von einem Quell-Workspace zu einem Ziel-Workspace. | source\_workspace\_id und target\_workspace\_id müssen in workspaces existieren. window\_id muss dem source\_workspace\_id zugeordnet sein. source\_workspace\_id und target\_workspace\_id dürfen nicht identisch sein. | Das Fenster window\_id ist vom source\_workspace\_id entfernt und dem target\_workspace\_id hinzugefügt. Andere Workspaces bleiben unberührt (d.h. es wird nicht implizit von einem dritten Workspace entfernt, falls es dort auch war, es sei denn, die interne Logik von assign\_window\_to\_workspace mit ensure\_unique\_assignment=true wird genutzt). | WindowRemovedFromWorkspace, WindowAddedToWorkspace (via manager) | SourceWorkspaceNotFound, TargetWorkspaceNotFound, WindowNotOnSourceWorkspace, CannotMoveToSameWorkspace, RuleViolation |
| pub fn find\_workspace\_for\_window(workspaces: \&std::collections::HashMap\<WorkspaceId, Workspace\>, window\_id: \&WindowIdentifier) \-\> Option\<WorkspaceId\> | Findet die ID des Workspace, dem ein bestimmtes Fenster aktuell zugeordnet ist. | \- | Gibt Some(WorkspaceId) zurück, wenn das Fenster einem Workspace in der Sammlung zugeordnet ist, sonst None. | \- | \- |

Die explizite Übergabe der workspaces-Sammlung an jede Funktion unterstreicht die Rolle dieses Moduls als Dienstleister, der auf Daten operiert, die vom workspaces::manager gehalten und verwaltet werden. Der Parameter ensure\_unique\_assignment in assign\_window\_to\_workspace ermöglicht es dem Aufrufer (typischerweise dem manager), die globale Regel "ein Fenster nur auf einem Workspace" durchzusetzen.

### **3.4. Events: Definition und Semantik**

Das Modul workspaces::assignment löst selbst keine Events aus. Änderungen an den Workspace-Objekten (Hinzufügen oder Entfernen von window\_ids) werden direkt auf diesen Objekten vorgenommen. Der workspaces::manager, der die Funktionen dieses Moduls aufruft, ist dafür verantwortlich, die entsprechenden Events zu publizieren (z.B. WindowAddedToWorkspace oder WindowRemovedFromWorkspace, unter Verwendung der in workspaces::core::event\_data definierten Payload-Strukturen). Diese Entkopplung hält das assignment-Modul fokussiert auf seine Kernlogik.

### **3.5. Fehlerbehandlung: WindowAssignmentError**

Für Fehler, die spezifisch bei Fensterzuweisungsoperationen auftreten, wird das WindowAssignmentError-Enum definiert.

* **Definition:**  
  Rust  
  // src/domain/workspaces/assignment/errors.rs  
  use thiserror::Error;  
  use crate::domain::workspaces::core::types::{WorkspaceId, WindowIdentifier};

  \#  
  pub enum WindowAssignmentError {  
      \#  
      WorkspaceNotFound(WorkspaceId), // Gilt für Ziel- oder Quell-Workspaces, je nach Kontext

      \#\[error("Window '{window\_id}' is already assigned to workspace '{workspace\_id}'. No action taken.")\]  
      WindowAlreadyAssigned { workspace\_id: WorkspaceId, window\_id: WindowIdentifier },

      \#\[error("Window '{window\_id}' is not assigned to workspace '{workspace\_id}', so it cannot be removed from it.")\]  
      WindowNotAssigned { workspace\_id: WorkspaceId, window\_id: WindowIdentifier }, // Spezifischer für Entfernungsoperationen

      \#  
      SourceWorkspaceNotFound(WorkspaceId),

      \#  
      TargetWorkspaceNotFound(WorkspaceId),

      \#\[error("Window '{window\_id}' not found on source workspace '{workspace\_id}' and thus cannot be moved.")\]  
      WindowNotOnSourceWorkspace { workspace\_id: WorkspaceId, window\_id: WindowIdentifier },

      \#\[error("Cannot move window '{window\_id}' from workspace '{workspace\_id}' to itself.")\]  
      CannotMoveToSameWorkspace { workspace\_id: WorkspaceId, window\_id: WindowIdentifier },

      \#  
      RuleViolation {  
          reason: String,  
          window\_id: Option\<WindowIdentifier\>,  
          target\_workspace\_id: Option\<WorkspaceId\>,  
      }, // Für spezifische, nicht abgedeckte Regeln

      \#\[error("An internal error occurred in window assignment logic: {context}")\]  
      Internal { context: String },  
  }

* Erläuterung und Anwendung von Fehlerbehandlungsprinzipien:  
  Die Definition von WindowAssignmentError folgt denselben Prinzipien wie WorkspaceCoreError unter Verwendung von thiserror.1 Die Varianten sind spezifisch für Zuweisungsoperationen und beinhalten relevante Identifikatoren, um den Kontext des Fehlers klar zu machen.1  
  Ein wichtiger Aspekt ist die Behandlung von Geschäftsregeln. Die Variante RuleViolation { reason,... } dient als flexibler Mechanismus, um Verletzungen von Zuweisungsregeln zu signalisieren, die nicht durch spezifischere Fehlervarianten abgedeckt sind. Es ist jedoch zu bedenken, dass eine programmatische Reaktion auf einen Fehler, der nur einen allgemeinen reason: String enthält, schwierig ist. Daher gilt: Für klar definierte, häufig auftretende oder kritische Geschäftsregeln der Fensterzuweisung *sollten* spezifische Fehlervarianten erstellt werden. Beispielsweise, wenn eine Regel besagt, dass bestimmte Fenstertypen nicht auf bestimmten Workspaces platziert werden dürfen, wäre ein Fehler wie DisallowedWindowTypeForWorkspace { window\_type: String, workspace\_id: WorkspaceId } aussagekräftiger als eine generische RuleViolation. Die RuleViolation-Variante dient dann als Fallback für dynamischere oder weniger häufige Regeln. Die Spezifikation sollte die wichtigsten Zuweisungsregeln identifizieren und dafür sorgen, dass dedizierte Fehler definiert werden, falls eine spezifische programmatische Behandlung durch den Aufrufer erforderlich ist. Dies steht im Einklang mit der Diskussion über die Granularität von Fehlertypen.2

**Tabelle: WindowAssignmentError Varianten**

| Variante | \#\[error("...")\]-Meldung (Auszug) | Semantische Bedeutung/Ursache | Enthaltene Datenfelder |
| :---- | :---- | :---- | :---- |
| WorkspaceNotFound(WorkspaceId) | "Workspace with ID '{0}' not found." | Ein angegebener Workspace (Quelle oder Ziel) existiert nicht in der übergebenen Sammlung. | Die ID des nicht gefundenen Workspace (WorkspaceId). |
| WindowAlreadyAssigned | "Window '{window\_id}' is already assigned..." | Es wurde versucht, ein Fenster einem Workspace zuzuweisen, dem es bereits zugeordnet ist (und keine weitere Aktion ist nötig/erwünscht). | workspace\_id: WorkspaceId, window\_id: WindowIdentifier. |
| WindowNotAssigned | "Window '{window\_id}' is not assigned..." | Es wurde versucht, ein Fenster von einem Workspace zu entfernen, dem es nicht zugeordnet ist. | workspace\_id: WorkspaceId, window\_id: WindowIdentifier. |
| SourceWorkspaceNotFound(WorkspaceId) | "Source workspace with ID '{0}' not found..." | Der Quell-Workspace für eine Verschiebungsoperation wurde nicht gefunden. | Die ID des Quell-Workspace (WorkspaceId). |
| TargetWorkspaceNotFound(WorkspaceId) | "Target workspace with ID '{0}' not found..." | Der Ziel-Workspace für eine Verschiebungsoperation wurde nicht gefunden. | Die ID des Ziel-Workspace (WorkspaceId). |
| WindowNotOnSourceWorkspace | "Window '{window\_id}' not found on source..." | Das zu verschiebende Fenster befindet sich nicht auf dem angegebenen Quell-Workspace. | workspace\_id: WorkspaceId, window\_id: WindowIdentifier. |
| CannotMoveToSameWorkspace | "Cannot move window... to itself." | Es wurde versucht, ein Fenster auf denselben Workspace zu verschieben, auf dem es sich bereits befindet. | workspace\_id: WorkspaceId, window\_id: WindowIdentifier. |
| RuleViolation | "A window assignment rule was violated: {reason}..." | Eine spezifische Geschäftsregel der Fensterzuweisung wurde verletzt. | reason: String, window\_id: Option\<WindowIdentifier\>, target\_workspace\_id: Option\<WorkspaceId\>. |
| Internal { context: String } | "An internal error occurred..." | Ein unerwarteter Fehler in der Zuweisungslogik. | context: String. |

### **3.6. Detaillierte Implementierungsschritte und Dateistruktur**

* **Dateistruktur innerhalb von src/domain/workspaces/assignment/:**  
  * mod.rs: Enthält die Implementierung der öffentlichen Zuweisungsfunktionen (assign\_window\_to\_workspace, remove\_window\_from\_workspace, move\_window\_to\_workspace, find\_workspace\_for\_window).  
  * errors.rs: Enthält die Definition des WindowAssignmentError-Enums.  
  * rules.rs (optional): Dieses Modul könnte interne Hilfsfunktionen oder Datenstrukturen enthalten, die spezifische Zuweisungsregeln kapseln (z.B. Überprüfung der "Fenster-Exklusivität"). Diese würden dann von den Hauptfunktionen in mod.rs genutzt.  
* **Implementierungsschritte:**  
  1. Definiere das WindowAssignmentError-Enum in errors.rs gemäß der Spezifikation in Abschnitt 3.5.  
  2. Implementiere pub fn assign\_window\_to\_workspace(...) in mod.rs:  
     * Überprüfe, ob target\_workspace\_id in workspaces existiert. Falls nicht, gib Err(WindowAssignmentError::WorkspaceNotFound(target\_workspace\_id)) zurück.  
     * Hole eine veränderbare Referenz auf den target\_workspace.  
     * Falls ensure\_unique\_assignment true ist:  
       * Iteriere über alle Workspaces in der workspaces-Sammlung (außer dem target\_workspace).  
       * Wenn ein anderer Workspace das window\_id enthält, rufe dessen remove\_window\_id(window\_id) Methode auf.  
     * Rufe target\_workspace.add\_window\_id(window\_id.clone()) auf. Wenn diese false zurückgibt (Fenster war bereits vorhanden), und dies als Fehlerfall betrachtet wird (abhängig von der genauen Semantik/Regeln), gib Err(WindowAssignmentError::WindowAlreadyAssigned {... }) zurück.  
     * Gib Ok(()) zurück.  
  3. Implementiere pub fn remove\_window\_from\_workspace(...) in mod.rs:  
     * Überprüfe, ob source\_workspace\_id in workspaces existiert. Falls nicht, gib Err(WindowAssignmentError::WorkspaceNotFound(source\_workspace\_id)) zurück.  
     * Hole eine veränderbare Referenz auf den source\_workspace.  
     * Rufe source\_workspace.remove\_window\_id(window\_id) auf und gib Ok(result) zurück. (Der Fehlerfall WindowNotAssigned wird hier nicht direkt von dieser Funktion erzeugt, da Workspace::remove\_window\_id nur bool zurückgibt. Der manager könnte dies interpretieren oder es wird angenommen, dass ein Aufruf zum Entfernen eines nicht vorhandenen Fensters kein Fehler ist, sondern einfach keine Aktion bewirkt und false zurückgibt). Alternativ könnte hier geprüft werden, ob das Fenster vorher drin war und bei false ein WindowNotAssigned Fehler erzeugt werden, falls das die gewünschte Semantik ist. Gemäß der Tabelle soll remove\_window\_from\_workspace Result\<bool,...\> zurückgeben, also ist die aktuelle Signatur von Workspace::remove\_window\_id ausreichend.  
  4. Implementiere pub fn move\_window\_to\_workspace(...) in mod.rs:  
     * Überprüfe, ob source\_workspace\_id und target\_workspace\_id identisch sind. Falls ja, gib Err(WindowAssignmentError::CannotMoveToSameWorkspace {... }) zurück.  
     * Überprüfe Existenz von source\_workspace (Fehler: SourceWorkspaceNotFound) und target\_workspace (Fehler: TargetWorkspaceNotFound).  
     * Hole Referenzen zu beiden Workspaces.  
     * Versuche, window\_id vom source\_workspace zu entfernen. Rufe source\_workspace.remove\_window\_id(window\_id) auf. Wenn dies false zurückgibt (Fenster war nicht auf Quelle), gib Err(WindowAssignmentError::WindowNotOnSourceWorkspace {... }) zurück.  
     * Füge window\_id zum target\_workspace hinzu. Rufe target\_workspace.add\_window\_id(window\_id.clone()) auf. (Die ensure\_unique\_assignment-Logik ist hier nicht direkt anwendbar, da wir explizit von einer Quelle zu einem Ziel verschieben. Es wird angenommen, dass das Fenster nach dem Entfernen von der Quelle nur noch dem Ziel hinzugefügt werden muss.)  
     * Gib Ok(()) zurück.  
  5. Implementiere pub fn find\_workspace\_for\_window(...) in mod.rs:  
     * Iteriere über die workspaces-Sammlung.  
     * Für jeden Workspace, prüfe, ob dessen window\_ids das gesuchte window\_id enthält.  
     * Wenn gefunden, gib Some(workspace.id()) zurück.  
     * Wenn die Iteration ohne Fund endet, gib None zurück.  
  6. Füge umfassende rustdoc-Kommentare für alle öffentlichen Funktionen hinzu.  
  7. Erstelle Unit-Tests im Untermodul tests in mod.rs. Teste alle Funktionen gründlich, einschließlich:  
     * Erfolgreiche Zuweisung, Entfernung und Verschiebung von Fenstern.  
     * Korrekte Handhabung der ensure\_unique\_assignment-Logik.  
     * Alle Fehlerfälle (nicht gefundene Workspaces, Fenster nicht auf Quell-Workspace, etc.).  
     * Randbedingungen (z.B. leere workspaces-Sammlung).  
     * Funktionalität von find\_workspace\_for\_window.

## **4\. Entwicklungsmodul 3: workspaces::manager – Orchestrierung und übergeordnete Verwaltung**

Das Modul workspaces::manager agiert als zentraler Orchestrator für alle Workspace-bezogenen Operationen. Es verwaltet die Gesamtheit der Workspaces, den Zustand des aktiven Workspace und dient als primäre Schnittstelle für andere Systemteile.

### **4.1. Verantwortlichkeiten und Design-Rationale**

Die Kernverantwortlichkeiten des WorkspaceManager sind:

* **Verwaltung der Workspace-Sammlung:** Halten und Pflegen einer Liste aller existierenden Workspace-Instanzen.  
* **Lebenszyklusmanagement:** Erstellung, Löschung und Modifikation von Workspaces.  
* **Zustandsmanagement des aktiven Workspace:** Verfolgen, welcher Workspace aktuell aktiv ist, und Ermöglichen des Wechsels.  
* **Orchestrierung von Operationen:** Koordination von Aktionen, die mehrere Workspaces betreffen oder globale Auswirkungen haben.  
* **Event-Publikation:** Benachrichtigung anderer Systemteile über signifikante Änderungen im Workspace-System (z.B. Erstellung, Löschung, Aktivierung eines Workspace, Fensterzuweisungen).  
* **Schnittstelle:** Bereitstellung einer kohärenten API für die System- und UI-Schicht zur Interaktion mit dem Workspace-System.

Das Design zielt darauf ab, die Komplexität der Workspace-Verwaltung an einem zentralen Ort zu bündeln. Dies fördert die Konsistenz des Gesamtzustands und vereinfacht die Interaktion für andere Komponenten, da sie nur mit dem WorkspaceManager und nicht mit einzelnen Workspace-Objekten oder dem assignment-Modul direkt kommunizieren müssen.

### **4.2. Interaktion mit anderen Modulen und externen Schnittstellen**

Der WorkspaceManager interagiert mit mehreren anderen Modulen:

* **workspaces::core:** Erstellt und hält Instanzen von Workspace-Objekten. Ruft Methoden auf diesen Objekten auf (z.B. rename, set\_layout\_type).  
* **workspaces::assignment:** Nutzt die Funktionen dieses Moduls (z.B. assign\_window\_to\_workspace) zur Durchführung der Logik für Fensterzuweisungen.  
* **workspaces::config:** Interagiert mit einem WorkspaceConfigProvider (aus workspaces::config), um die Workspace-Konfiguration beim Start zu laden und Änderungen zu persistieren.  
* **Event-System (nicht spezifiziert, aber implizit):** Benötigt einen Mechanismus zum Publizieren von WorkspaceEvents. Dies könnte ein interner Event-Bus, ein tokio::sync::broadcast Channel oder eine ähnliche Struktur sein. Für diese Spezifikation wird angenommen, dass ein solcher Mechanismus existiert und vom WorkspaceManager genutzt werden kann.  
* **Systemschicht:** Wird vom WorkspaceManager über Änderungen informiert (z.B. welcher Workspace aktiv ist, welche Fenster wo sind) und informiert den WorkspaceManager über Systemereignisse (z.B. neue Fenster).  
* **UI-Schicht:** Nutzt die API des WorkspaceManager zur Darstellung und Manipulation von Workspaces und reagiert auf WorkspaceEvents.

### **4.3. Öffentliche API: Methoden und Funktionen**

Die öffentliche API wird durch das WorkspaceManager-Struct und dessen Methoden bereitgestellt.

* **Struct-Definition:**  
  Rust  
  // src/domain/workspaces/manager/mod.rs  
  use std::collections::HashMap;  
  use std::sync::Arc;  
  use uuid::Uuid;  
  use crate::domain::workspaces::core::types::{WorkspaceId, WindowIdentifier, WorkspaceLayoutType};  
  use crate::domain::workspaces::core::Workspace;  
  use crate::domain::workspaces::core::event\_data::\*;  
  use crate::domain::workspaces::assignment;  
  use crate::domain::workspaces::config::{WorkspaceConfigProvider, WorkspaceSetSnapshot, WorkspaceSnapshot};  
  use crate::domain::workspaces::manager::errors::WorkspaceManagerError;  
  use crate::domain::workspaces::manager::events::WorkspaceEvent; // Und Event-Publisher

  // Annahme: Ein Event-Publisher Trait oder eine konkrete Implementierung  
  pub trait EventPublisher\<E\>: Send \+ Sync {  
      fn publish(\&self, event: E);  
  }

  pub struct WorkspaceManager {  
      workspaces: HashMap\<WorkspaceId, Workspace\>,  
      active\_workspace\_id: Option\<WorkspaceId\>,  
      // Hält die Reihenfolge der Workspaces für UI-Darstellung oder Wechsel-Logik  
      ordered\_workspace\_ids: Vec\<WorkspaceId\>,  
      next\_workspace\_number: u32, // Für Standardnamen wie "Workspace 1"  
      config\_provider: Arc\<dyn WorkspaceConfigProvider\>,  
      event\_publisher: Arc\<dyn EventPublisher\<WorkspaceEvent\>\>, // Zum Publizieren von Events  
      ensure\_unique\_window\_assignment: bool, // Konfigurierbare Regel  
  }

  Die ordered\_workspace\_ids sind wichtig, um eine konsistente Reihenfolge für UI-Elemente wie Pager oder für "Nächster/Vorheriger Workspace"-Aktionen zu gewährleisten. ensure\_unique\_window\_assignment macht die wichtige Regel der Fensterzuweisung explizit konfigurierbar.  
* **Methoden der impl WorkspaceManager:**

**Tabelle: API-Methoden für workspaces::manager::WorkspaceManager**

| Methode (Rust-Signatur) | Kurzbeschreibung | Vor-/Nachbedingungen | Ausgelöste Events | Mögliche Fehler (WorkspaceManagerError) |
| :---- | :---- | :---- | :---- | :---- |
| pub fn new(config\_provider: Arc\<dyn WorkspaceConfigProvider\>, event\_publisher: Arc\<dyn EventPublisher\<WorkspaceEvent\>\>, ensure\_unique\_window\_assignment: bool) \-\> Result\<Self, WorkspaceManagerError\> | Initialisiert den Manager. Lädt Konfiguration, setzt Standard-Workspaces falls keine Konfig vorhanden. | \- | Manager ist initialisiert. Workspaces sind geladen oder Standard-Workspaces erstellt. active\_workspace\_id ist gesetzt. | WorkspaceCreated (falls Standard-WS erstellt), ActiveWorkspaceChanged |
| pub fn create\_workspace(\&mut self, name: Option\<String\>, persistent\_id: Option\<String\>) \-\> Result\<WorkspaceId, WorkspaceManagerError\> | Erstellt einen neuen Workspace, fügt ihn zur Sammlung hinzu. | Name (falls Some) und persistent\_id (falls Some) müssen gültig sein. | Neuer Workspace ist erstellt, zur Sammlung und ordered\_workspace\_ids hinzugefügt. | WorkspaceCreated |
| pub fn delete\_workspace(\&mut self, id: WorkspaceId, fallback\_id\_for\_windows: Option\<WorkspaceId\>) \-\> Result\<(), WorkspaceManagerError\> | Löscht einen Workspace. Fenster werden ggf. auf einen Fallback-Workspace verschoben. | Darf nicht der letzte Workspace sein. fallback\_id\_for\_windows muss existieren, falls Fenster verschoben werden müssen und der Workspace nicht leer ist. | Workspace ist gelöscht. Fenster sind verschoben. Ggf. neuer aktiver Workspace. | WorkspaceDeleted, ActiveWorkspaceChanged, WindowRemovedFromWorkspace, WindowAddedToWorkspace |
| pub fn get\_workspace(\&self, id: WorkspaceId) \-\> Option\<\&Workspace\> | Gibt eine Referenz auf einen Workspace anhand seiner ID zurück. | \- | \- | \- |
| pub fn get\_workspace\_mut(\&mut self, id: WorkspaceId) \-\> Option\<\&mut Workspace\> | Gibt eine veränderbare Referenz auf einen Workspace anhand seiner ID zurück. | \- | \- | \- |
| pub fn all\_workspaces\_ordered(\&self) \-\> Vec\<\&Workspace\> | Gibt eine geordnete Liste aller Workspaces zurück. | \- | \- | \- |
| pub fn active\_workspace\_id(\&self) \-\> Option\<WorkspaceId\> | Gibt die ID des aktuell aktiven Workspace zurück. | \- | \- | \- |
| pub fn set\_active\_workspace(\&mut self, id: WorkspaceId) \-\> Result\<(), WorkspaceManagerError\> | Setzt den aktiven Workspace. | id muss ein existierender Workspace sein. | active\_workspace\_id ist auf id gesetzt. | ActiveWorkspaceChanged |
| pub fn assign\_window\_to\_active\_workspace(\&mut self, window\_id: \&WindowIdentifier) \-\> Result\<(), WorkspaceManagerError\> | Weist ein Fenster dem aktiven Workspace zu. | Ein aktiver Workspace muss existieren. | Fenster ist dem aktiven Workspace zugeordnet. | WindowAddedToWorkspace, WindowRemovedFromWorkspace (falls ensure\_unique\_window\_assignment) |
| pub fn assign\_window\_to\_specific\_workspace(\&mut self, workspace\_id: WorkspaceId, window\_id: \&WindowIdentifier) \-\> Result\<(), WorkspaceManagerError\> | Weist ein Fenster einem spezifischen Workspace zu. | workspace\_id muss existieren. | Fenster ist dem workspace\_id zugeordnet. | WindowAddedToWorkspace, WindowRemovedFromWorkspace (falls ensure\_unique\_window\_assignment) |
| pub fn remove\_window\_from\_its\_workspace(\&mut self, window\_id: \&WindowIdentifier) \-\> Result\<Option\<WorkspaceId\>, WorkspaceManagerError\> | Entfernt ein Fenster von dem Workspace, dem es aktuell zugeordnet ist. Gibt die ID des Workspace zurück, von dem es entfernt wurde. | \- | Fenster ist keinem Workspace mehr zugeordnet (oder dem, dem es explizit zugewiesen war). | WindowRemovedFromWorkspace |
| pub fn move\_window\_to\_specific\_workspace(\&mut self, target\_workspace\_id: WorkspaceId, window\_id: \&WindowIdentifier) \-\> Result\<(), WorkspaceManagerError\> | Verschiebt ein Fenster von seinem aktuellen Workspace zu einem spezifischen Ziel-Workspace. | target\_workspace\_id muss existieren. Fenster muss einem Workspace zugeordnet sein. | Fenster ist dem target\_workspace\_id zugeordnet und vom vorherigen entfernt. | WindowRemovedFromWorkspace, WindowAddedToWorkspace |
| pub fn rename\_workspace(\&mut self, id: WorkspaceId, new\_name: String) \-\> Result\<(), WorkspaceManagerError\> | Benennt einen Workspace um. | id muss existieren. new\_name muss gültig sein. | Workspace ist umbenannt. | WorkspaceRenamed |
| pub fn set\_workspace\_layout(\&mut self, id: WorkspaceId, layout\_type: WorkspaceLayoutType) \-\> Result\<(), WorkspaceManagerError\> | Ändert den Layout-Typ eines Workspace. | id muss existieren. | Layout-Typ ist geändert. | WorkspaceLayoutChanged |
| pub fn save\_configuration(\&self) \-\> Result\<(), WorkspaceManagerError\> | Speichert die aktuelle Workspace-Konfiguration (Namen, persistente IDs, Reihenfolge, aktiver Workspace). | \- | Konfiguration ist gespeichert. | \- |

### **4.4. Events: Definition und Semantik**

Der WorkspaceManager ist der primäre Publisher für alle Workspace-bezogenen Events. Diese Events informieren andere Teile des Systems über Zustandsänderungen.

* **Event-Enum: WorkspaceEvent**  
  Rust  
  // src/domain/workspaces/manager/events.rs  
  use crate::domain::workspaces::core::types::{WorkspaceId, WindowIdentifier, WorkspaceLayoutType};  
  use crate::domain::workspaces::core::event\_data::\*; // Importiert Payloads wie WorkspaceRenamedData

  \#  
  pub enum WorkspaceEvent {  
      WorkspaceCreated {  
          id: WorkspaceId,  
          name: String,  
          persistent\_id: Option\<String\>,  
          position: usize, // Position in der geordneten Liste  
      },  
      WorkspaceDeleted {  
          id: WorkspaceId,  
          // ID des Workspace, auf den Fenster verschoben wurden, falls zutreffend  
          windows\_moved\_to\_workspace\_id: Option\<WorkspaceId\>,  
      },  
      ActiveWorkspaceChanged {  
          old\_id: Option\<WorkspaceId\>,  
          new\_id: WorkspaceId,  
      },  
      WorkspaceRenamed(WorkspaceRenamedData), // Nutzt Payload aus core::event\_data  
      WorkspaceLayoutChanged(WorkspaceLayoutChangedData), // Nutzt Payload aus core::event\_data  
      WindowAddedToWorkspace(WindowAddedToWorkspaceData), // Nutzt Payload aus core::event\_data  
      WindowRemovedFromWorkspace(WindowRemovedFromWorkspaceData), // Nutzt Payload aus core::event\_data  
      WorkspaceOrderChanged(Vec\<WorkspaceId\>), // Die neue, vollständige Reihenfolge der Workspace-IDs  
      WorkspacesReloaded(Vec\<WorkspaceId\>), // Signalisiert, dass Workspaces neu geladen wurden (z.B. aus Konfig)  
      WorkspacePersistentIdChanged(WorkspacePersistentIdChangedData), // Nutzt Payload aus core::event\_data  
  }

* **Publisher:** WorkspaceManager (über den injizierten event\_publisher).  
* **Typische Subscriber:**  
  * **UI-Schicht:** Aktualisiert die Darstellung von Workspaces, Panels, Fensterlisten etc.  
  * **domain::window\_management:** Reagiert auf Layout-Änderungen oder Änderungen des aktiven Workspace, um Fenster entsprechend anzuordnen oder Fokus zu setzen.  
  * **Systemschicht (Compositor):** Passt die Sichtbarkeit von Fenstern/Surfaces an, wenn sich der aktive Workspace ändert.  
  * **Logging/Tracing-Systeme:** Protokollieren Workspace-bezogene Aktivitäten.

**Tabelle: WorkspaceEvent Varianten**

| Event-Variante | Payload-Struktur/Daten | Semantische Bedeutung | Typische Auslöser (Manager-Methode) |
| :---- | :---- | :---- | :---- |
| WorkspaceCreated | id, name, persistent\_id, position | Ein neuer Workspace wurde erstellt und der Sammlung hinzugefügt. | create\_workspace, Initialisierung |
| WorkspaceDeleted | id, windows\_moved\_to\_workspace\_id | Ein Workspace wurde gelöscht. | delete\_workspace |
| ActiveWorkspaceChanged | old\_id, new\_id | Der aktive Workspace hat sich geändert. | set\_active\_workspace, delete\_workspace (falls aktiver gelöscht) |
| WorkspaceRenamed | WorkspaceRenamedData | Ein Workspace wurde umbenannt. | rename\_workspace |
| WorkspaceLayoutChanged | WorkspaceLayoutChangedData | Der Layout-Typ eines Workspace wurde geändert. | set\_workspace\_layout |
| WindowAddedToWorkspace | WindowAddedToWorkspaceData | Ein Fenster wurde einem Workspace hinzugefügt. | assign\_window\_to\_active\_workspace, assign\_window\_to\_specific\_workspace, move\_window\_to\_specific\_workspace |
| WindowRemovedFromWorkspace | WindowRemovedFromWorkspaceData | Ein Fenster wurde von einem Workspace entfernt. | remove\_window\_from\_its\_workspace, move\_window\_to\_specific\_workspace, delete\_workspace |
| WorkspaceOrderChanged | Vec\<WorkspaceId\> | Die Reihenfolge der Workspaces hat sich geändert. | (Noch nicht spezifizierte Methoden wie move\_workspace\_left/right) |
| WorkspacesReloaded | Vec\<WorkspaceId\> | Die Workspace-Konfiguration wurde neu geladen. | new (bei Initialisierung aus Konfig) |
| WorkspacePersistentIdChanged | WorkspacePersistentIdChangedData | Die persistente ID eines Workspace wurde geändert. | (Indirekt durch Workspace::set\_persistent\_id via Manager) |

### **4.5. Fehlerbehandlung: WorkspaceManagerError**

Das WorkspaceManagerError-Enum fasst Fehler zusammen, die auf der Ebene des Managers auftreten können, einschließlich Fehlern aus den unterlagerten Modulen.

* **Definition:**  
  Rust  
  // src/domain/workspaces/manager/errors.rs  
  use thiserror::Error;  
  use crate::domain::workspaces::core::types::WorkspaceId;  
  use crate::domain::workspaces::core::errors::WorkspaceCoreError;  
  use crate::domain::workspaces::assignment::errors::WindowAssignmentError;  
  use crate::domain::workspaces::config::errors::WorkspaceConfigError;

  \#  
  pub enum WorkspaceManagerError {  
      \#  
      WorkspaceNotFound(WorkspaceId),

      \#\[error("Cannot delete the last workspace. At least one workspace must remain.")\]  
      CannotDeleteLastWorkspace,

      \#  
      DeleteRequiresFallbackForWindows(WorkspaceId),

      \#  
      FallbackWorkspaceNotFound(WorkspaceId),

      \#\[error("A workspace core operation failed: {source}")\]  
      CoreError { \#\[from\] source: WorkspaceCoreError },

      \#\[error("A window assignment operation failed: {source}")\]  
      AssignmentError { \#\[from\] source: WindowAssignmentError },

      \#\[error("A workspace configuration operation failed: {source}")\]  
      ConfigError { \#\[from\] source: WorkspaceConfigError },

      \#\[error("Attempted to set a non-existent workspace '{0}' as active.")\]  
      SetActiveWorkspaceNotFound(WorkspaceId),

      \#\[error("No active workspace is set, but the operation requires one.")\]  
      NoActiveWorkspace,

      \#  
      DuplicatePersistentId(String),

      \#\[error("An internal error occurred in the workspace manager: {context}")\]  
      Internal { context: String },  
  }

* Erläuterung und Anwendung von Fehlerbehandlungsprinzipien:  
  WorkspaceManagerError verwendet thiserror und das \#\[from\]-Attribut, um Fehler aus den Modulen core, assignment und config elegant zu wrappen.1 Dies ist ein zentrales Muster für die Fehleraggregation in übergeordneten Komponenten. Die source()-Kette bleibt dabei erhalten, was für die Fehlerdiagnose kritisch ist.3 Wenn beispielsweise WorkspaceManager::rename\_workspace aufgerufen wird und intern Workspace::rename einen WorkspaceCoreError::NameTooLong zurückgibt, wird dieser Fehler in einen WorkspaceManagerError::CoreError { source: WorkspaceCoreError::NameTooLong } umgewandelt. Der Aufrufer des WorkspaceManager kann dann error.source() verwenden, um an den ursprünglichen WorkspaceCoreError zu gelangen und dessen spezifische Details zu untersuchen. Diese Fähigkeit, die Fehlerursache über mehrere Abstraktionsebenen hinweg zurückzuverfolgen, ist für die Entwicklung robuster Software unerlässlich und wird durch die konsequente Anwendung von \#\[from\] und dem std::error::Error-Trait ermöglicht.1  
  Zusätzlich definiert das Enum spezifische Fehler, die nur in der Logik des Managers auftreten können, wie CannotDeleteLastWorkspace oder NoActiveWorkspace.

**Tabelle: WorkspaceManagerError Varianten**

| Variante | \#\[error("...")\]-Meldung (Auszug) | Semantische Bedeutung/Ursache | Enthaltene Daten/Quellfehler |
| :---- | :---- | :---- | :---- |
| WorkspaceNotFound(WorkspaceId) | "Workspace with ID '{0}' not found." | Ein referenzierter Workspace existiert nicht. | WorkspaceId des nicht gefundenen Workspace. |
| CannotDeleteLastWorkspace | "Cannot delete the last workspace..." | Es wurde versucht, den einzigen verbleibenden Workspace zu löschen. | \- |
| DeleteRequiresFallbackForWindows(WorkspaceId) | "Cannot delete workspace '{0}' because it contains windows..." | Ein Workspace mit Fenstern soll gelöscht werden, ohne einen Fallback anzugeben. | WorkspaceId des zu löschenden Workspace. |
| FallbackWorkspaceNotFound(WorkspaceId) | "The specified fallback workspace with ID '{0}' was not found..." | Der angegebene Fallback-Workspace existiert nicht. | WorkspaceId des nicht gefundenen Fallback-Workspace. |
| CoreError | "A workspace core operation failed..." | Fehler aus workspaces::core. | source: WorkspaceCoreError. |
| AssignmentError | "A window assignment operation failed..." | Fehler aus workspaces::assignment. | source: WindowAssignmentError. |
| ConfigError | "A workspace configuration operation failed..." | Fehler aus workspaces::config. | source: WorkspaceConfigError. |
| SetActiveWorkspaceNotFound(WorkspaceId) | "Attempted to set a non-existent workspace '{0}' as active." | Ein nicht existierender Workspace sollte als aktiv gesetzt werden. | WorkspaceId des nicht gefundenen Workspace. |
| NoActiveWorkspace | "No active workspace is set..." | Eine Operation wurde aufgerufen, die einen aktiven Workspace erfordert, aber keiner ist gesetzt. | \- |
| DuplicatePersistentId(String) | "Attempted to create a workspace with a persistent ID ('{0}') that already exists." | Eine persistente ID, die bereits verwendet wird, wurde für einen neuen Workspace angegeben. | Die duplizierte String ID. |
| Internal { context: String } | "An internal error occurred..." | Ein unerwarteter interner Fehler im Manager. | context: String. |

### **4.6. Detaillierte Implementierungsschritte und Dateistruktur**

* **Dateistruktur innerhalb von src/domain/workspaces/manager/:**  
  * mod.rs: Enthält die Definition des WorkspaceManager-Structs und die Implementierung seiner Methoden.  
  * errors.rs: Enthält die Definition des WorkspaceManagerError-Enums.  
  * events.rs: Enthält die Definition des WorkspaceEvent-Enums und ggf. des EventPublisher-Traits.  
* **Implementierungsschritte:**  
  1. Definiere WorkspaceEvent (und EventPublisher-Trait, falls nicht global vorhanden) in events.rs.  
  2. Definiere WorkspaceManagerError in errors.rs.  
  3. Implementiere das WorkspaceManager-Struct in mod.rs.  
  4. Implementiere pub fn new(...) \-\> Result\<Self, WorkspaceManagerError\>:  
     * Initialisiere workspaces als leere HashMap, ordered\_workspace\_ids als leeren Vec.  
     * Setze next\_workspace\_number auf 1\.  
     * Speichere config\_provider, event\_publisher, ensure\_unique\_window\_assignment.  
     * Versuche, die Konfiguration mittels self.config\_provider.load\_workspace\_config() zu laden.  
       * Bei Erfolg (Ok(snapshot)): Rekonstruiere Workspace-Objekte aus snapshot.workspaces. Füge sie zu self.workspaces und self.ordered\_workspace\_ids hinzu (Reihenfolge aus Snapshot beachten). Setze self.active\_workspace\_id basierend auf snapshot.active\_workspace\_persistent\_id (Suche nach Workspace mit passender persistent\_id). Aktualisiere next\_workspace\_number ggf. basierend auf den Namen der geladenen Workspaces. Publiziere WorkspacesReloaded und ActiveWorkspaceChanged.  
       * Bei Fehler (Err(config\_err)):  
         * Wenn der Fehler anzeigt, dass keine Konfiguration vorhanden ist (z.B. CoreConfigError::NotFound), erstelle einen Standard-Workspace (z.B. "Workspace 1"). Füge ihn hinzu, setze ihn als aktiv. Publiziere WorkspaceCreated, ActiveWorkspaceChanged.  
         * Andernfalls mappe den config\_err zu WorkspaceManagerError::ConfigError und gib ihn zurück.  
  5. Implementiere pub fn create\_workspace(...) \-\> Result\<WorkspaceId, WorkspaceManagerError\>:  
     * Falls persistent\_id Some ist, prüfe, ob bereits ein Workspace mit dieser persistent\_id existiert. Falls ja, Fehler DuplicatePersistentId.  
     * Bestimme den Namen: Falls name None ist, generiere einen Standardnamen (z.B. "Workspace {next\_workspace\_number}").  
     * Erstelle ein neues Workspace-Objekt via Workspace::new(final\_name, persistent\_id). Mappe WorkspaceCoreError zu CoreError.  
     * Füge den neuen Workspace zu self.workspaces und self.ordered\_workspace\_ids hinzu (z.B. am Ende).  
     * Inkrementiere next\_workspace\_number falls ein Standardname verwendet wurde.  
     * Publiziere WorkspaceEvent::WorkspaceCreated mit ID, Name, persistent\_id und Position.  
     * Rufe self.save\_configuration() auf.  
     * Gib Ok(new\_workspace.id()) zurück.  
  6. Implementiere pub fn delete\_workspace(...) \-\> Result\<(), WorkspaceManagerError\>:  
     * Prüfe, ob id existiert (Fehler: WorkspaceNotFound).  
     * Prüfe, ob es der letzte Workspace ist (Fehler: CannotDeleteLastWorkspace).  
     * Hole den zu löschenden Workspace. Wenn er Fenster enthält und fallback\_id\_for\_windows None ist, Fehler DeleteRequiresFallbackForWindows.  
     * Falls Fenster verschoben werden müssen:  
       * Prüfe, ob fallback\_id\_for\_windows existiert (Fehler: FallbackWorkspaceNotFound).  
       * Nutze assignment::move\_window\_to\_workspace (oder eine ähnliche Logik) für jedes Fenster, um es vom zu löschenden Workspace zum Fallback-Workspace zu verschieben. Mappe WindowAssignmentError zu AssignmentError. Publiziere WindowRemovedFromWorkspace und WindowAddedToWorkspace für jedes verschobene Fenster.  
     * Entferne den Workspace aus self.workspaces und self.ordered\_workspace\_ids.  
     * Falls der gelöschte Workspace der aktive war: Setze einen anderen Workspace als aktiv (z.B. den ersten in ordered\_workspace\_ids). Publiziere ActiveWorkspaceChanged.  
     * Publiziere WorkspaceEvent::WorkspaceDeleted.  
     * Rufe self.save\_configuration() auf.  
     * Gib Ok(()) zurück.  
  7. Implementiere die Getter-Methoden (get\_workspace, get\_workspace\_mut, all\_workspaces\_ordered, active\_workspace\_id). Für all\_workspaces\_ordered iteriere über ordered\_workspace\_ids und hole die entsprechenden Workspace-Referenzen aus workspaces.  
  8. Implementiere pub fn set\_active\_workspace(...) \-\> Result\<(), WorkspaceManagerError\>:  
     * Prüfe, ob id existiert (Fehler: SetActiveWorkspaceNotFound).  
     * Wenn id bereits aktiv ist, keine Aktion.  
     * Setze self.active\_workspace\_id \= Some(id).  
     * Publiziere WorkspaceEvent::ActiveWorkspaceChanged mit alter und neuer ID.  
     * Rufe self.save\_configuration() auf (optional, je nachdem ob der aktive Workspace persistiert werden soll).  
     * Gib Ok(()) zurück.  
  9. Implementiere Fensterzuweisungsmethoden (assign\_window\_to\_active\_workspace, assign\_window\_to\_specific\_workspace, remove\_window\_from\_its\_workspace, move\_window\_to\_specific\_workspace):  
     * Nutze die entsprechenden Funktionen aus dem workspaces::assignment-Modul.  
     * Übergebe \&mut self.workspaces und self.ensure\_unique\_window\_assignment (wo relevant).  
     * Mappe WindowAssignmentError zu WorkspaceManagerError::AssignmentError.  
     * Publiziere die relevanten Events (WindowAddedToWorkspace, WindowRemovedFromWorkspace) nach erfolgreicher Operation.  
  10. Implementiere rename\_workspace und set\_workspace\_layout:  
      * Hole \&mut Workspace (Fehler: WorkspaceNotFound).  
      * Rufe die entsprechende Methode auf dem Workspace-Objekt auf (rename oder set\_layout\_type). Mappe WorkspaceCoreError zu CoreError.  
      * Publiziere das entsprechende Event (WorkspaceRenamed oder WorkspaceLayoutChanged).  
      * Rufe self.save\_configuration() auf.  
  11. Implementiere pub fn save\_configuration(\&self) \-\> Result\<(), WorkspaceManagerError\>:  
      * Erstelle ein WorkspaceSetSnapshot. Fülle workspaces durch Iteration über self.ordered\_workspace\_ids und Erstellung von WorkspaceSnapshots für jeden Workspace (Name, persistente ID, Layout).  
      * Setze active\_workspace\_persistent\_id im Snapshot basierend auf der persistent\_id des aktuellen active\_workspace\_id.  
      * Rufe self.config\_provider.save\_workspace\_config(\&snapshot) auf. Mappe WorkspaceConfigError zu ConfigError.  
  12. Stelle sicher, dass alle Methoden umfassend mit rustdoc dokumentiert sind.  
  13. Erstelle Unit- und Integrationstests, die das Zusammenspiel der Module core, assignment, config und des Event-Publishings testen. Mocke WorkspaceConfigProvider und EventPublisher für die Tests.

## **5\. Entwicklungsmodul 4: workspaces::config – Konfigurations- und Persistenzlogik**

Das Modul workspaces::config ist dediziert für das Laden und Speichern der Konfiguration des Workspace-Systems zuständig.

### **5.1. Verantwortlichkeiten und Design-Rationale**

Die Hauptverantwortung dieses Moduls besteht darin, eine Abstraktion für die Persistenz von Workspace-bezogenen Daten bereitzustellen. Dies umfasst typischerweise:

* Namen und persistente IDs der Workspaces.  
* Standard-Layout-Typen pro Workspace.  
* Die Reihenfolge der Workspaces.  
* Die ID des zuletzt aktiven Workspace.

Es interagiert mit der core::config-Komponente der Kernschicht, um die tatsächlichen Lese- und Schreiboperationen aus bzw. in Konfigurationsdateien (oder andere Persistenzmechanismen) durchzuführen.  
Das Design-Rationale für dieses separate Modul ist die Entkopplung der Workspace-Verwaltungslogik (workspaces::manager) von den spezifischen Details der Konfigurationsspeicherung. Dies ermöglicht es, das Speicherformat (z.B. JSON, TOML, SQLite) oder den Speicherort zu ändern, ohne den WorkspaceManager modifizieren zu müssen, solange die WorkspaceConfigProvider-Schnittstelle eingehalten wird.

### **5.2. Datenstrukturen für Konfiguration und Interaktion mit core::config**

Für die Serialisierung und Deserialisierung der Workspace-Konfiguration werden spezielle Snapshot-Strukturen verwendet. Diese Strukturen sind so gestaltet, dass sie nur die Daten enthalten, die tatsächlich persistiert werden sollen.

* Struct: WorkspaceSnapshot  
  Eine serialisierbare Repräsentation der zu persistierenden Daten eines einzelnen Workspace.  
  Rust  
  // src/domain/workspaces/config/mod.rs  
  use serde::{Serialize, Deserialize};  
  use crate::domain::workspaces::core::types::{WorkspaceLayoutType, WorkspaceId}; // WorkspaceId nur für Referenz, nicht persistiert

  \#  
  pub struct WorkspaceSnapshot {  
      // Die \`persistent\_id\` ist der Schlüssel zur Wiedererkennung eines Workspace über Sitzungen.  
      // Die Laufzeit-\`WorkspaceId\` (uuid) wird bei jedem Start neu generiert und ist nicht Teil des Snapshots.  
      pub persistent\_id: String,  
      pub name: String,  
      pub layout\_type: WorkspaceLayoutType,  
      // \`window\_ids\` werden nicht persistiert, da sie von laufenden Anwendungen abhängen und transient sind.  
      // \`created\_at\` wird ebenfalls nicht standardmäßig persistiert, es sei denn, es gibt eine Anforderung dafür.  
  }

* Struct: WorkspaceSetSnapshot  
  Eine serialisierbare Repräsentation der gesamten Workspace-Konfiguration, die eine Liste von WorkspaceSnapshot-Instanzen und die persistente ID des aktiven Workspace enthält.  
  Rust  
  // src/domain/workspaces/config/mod.rs  
  \#  
  pub struct WorkspaceSetSnapshot {  
      pub workspaces: Vec\<WorkspaceSnapshot\>,  
      // Speichert die \`persistent\_id\` des Workspace, der beim letzten Speichern aktiv war.  
      pub active\_workspace\_persistent\_id: Option\<String\>,  
      // Die Reihenfolge der \`workspaces\` in diesem Vec definiert die persistierte Reihenfolge.  
  }

* Trait: WorkspaceConfigProvider  
  Definiert die Schnittstelle, die dieses Modul dem WorkspaceManager zur Verfügung stellt. Dies ermöglicht die Entkopplung von der konkreten Implementierung der Persistenzlogik.  
  Rust  
  // src/domain/workspaces/config/mod.rs  
  use crate::domain::workspaces::config::errors::WorkspaceConfigError;

  pub trait WorkspaceConfigProvider: Send \+ Sync {  
      fn load\_workspace\_config(\&self) \-\> Result\<WorkspaceSetSnapshot, WorkspaceConfigError\>;  
      fn save\_workspace\_config(\&self, config\_snapshot: \&WorkspaceSetSnapshot) \-\> Result\<(), WorkspaceConfigError\>;  
  }

* Struct: FilesystemConfigProvider (Beispielimplementierung)  
  Eine konkrete Implementierung von WorkspaceConfigProvider, die core::config (oder eine ähnliche Abstraktion der Kernschicht für Dateizugriffe) nutzt, um die Konfiguration als Datei (z.B. JSON oder TOML) zu speichern und zu laden.  
  Rust  
  // src/domain/workspaces/config/mod.rs  
  use std::sync::Arc;  
  use crate::core::config::ConfigService; // Annahme: Ein Service aus der Kernschicht

  pub struct FilesystemConfigProvider {  
      config\_service: Arc\<dyn ConfigService\>, // Service aus der Kernschicht  
      config\_file\_name: String, // z.B. "workspaces\_v1.json"  
  }

  impl FilesystemConfigProvider {  
      pub fn new(config\_service: Arc\<dyn ConfigService\>, config\_file\_name: String) \-\> Self {  
          Self { config\_service, config\_file\_name }  
      }  
  }

  // Die Implementierung von \`WorkspaceConfigProvider\` für \`FilesystemConfigProvider\` folgt in Abschnitt 5.6

### **5.3. Öffentliche API: Methoden und Funktionen**

Die öffentliche API dieses Moduls wird durch das WorkspaceConfigProvider-Trait definiert. Konkrete Implementierungen wie FilesystemConfigProvider setzen dieses Trait um.  
**Tabelle: API-Methoden für WorkspaceConfigProvider**

| Methode (Rust-Signatur) | Kurzbeschreibung | Mögliche Fehler (WorkspaceConfigError) |
| :---- | :---- | :---- |
| fn load\_workspace\_config(\&self) \-\> Result\<WorkspaceSetSnapshot, WorkspaceConfigError\> | Lädt die Workspace-Konfiguration aus dem persistenten Speicher. | LoadError, InvalidData, DeserializationError, PersistentIdNotFound (falls Konsistenzchecks fehlschlagen) |
| fn save\_workspace\_config(\&self, config\_snapshot: \&WorkspaceSetSnapshot) \-\> Result\<(), WorkspaceConfigError\> | Speichert die übergebene Workspace-Konfiguration in den persistenten Speicher. | SaveError, SerializationError |

Diese Schnittstelle ermöglicht es dem WorkspaceManager, die Konfiguration zu laden und zu speichern, ohne Details über den Speicherort oder das Format kennen zu müssen. Dies verbessert die Testbarkeit, da der WorkspaceConfigProvider im WorkspaceManager leicht durch eine Mock-Implementierung ersetzt werden kann.

### **5.4. Events: Definition und Semantik**

Das Modul workspaces::config ist typischerweise nicht dafür verantwortlich, eigene Events zu publizieren. Ein erfolgreicher Lade- oder Speichervorgang wird durch Result::Ok(()) signalisiert, während Fehler über das WorkspaceConfigError-Enum zurückgegeben werden. Der WorkspaceManager kann nach einem erfolgreichen Ladevorgang (z.B. bei der Initialisierung) ein WorkspacesReloaded-Event auslösen, um andere Systemteile über die Verfügbarkeit der geladenen Konfiguration zu informieren.

### **5.5. Fehlerbehandlung: WorkspaceConfigError**

Für Fehler, die spezifisch bei Konfigurations- und Persistenzoperationen auftreten, wird das WorkspaceConfigError-Enum definiert.

* **Definition:**  
  Rust  
  // src/domain/workspaces/config/errors.rs  
  use thiserror::Error;  
  // Annahme: Ein allgemeiner Konfigurationsfehler aus der Kernschicht,  
  // der I/O-Fehler, Berechtigungsfehler etc. kapseln kann.  
  use crate::core::config::ConfigError as CoreConfigError;

  \#  
  pub enum WorkspaceConfigError {  
      \#\[error("Failed to load workspace configuration from '{path}': {source}")\]  
      LoadError {  
          path: String,  
          \#\[source\]  
          source: CoreConfigError,  
      },

      \#\[error("Failed to save workspace configuration to '{path}': {source}")\]  
      SaveError {  
          path: String,  
          \#\[source\]  
          source: CoreConfigError,  
      },

      \#\[error("Workspace configuration data is invalid or corrupt: {reason}. Path: '{path:?}'")\]  
      InvalidData { reason: String, path: Option\<String\> },

      \#  
      SerializationError {  
          message: String,  
          \#\[source\]  
          source: Option\<serde\_json::Error\>, // Beispiel für serde\_json  
      },

      \#  
      DeserializationError {  
          message: String,  
          snippet: Option\<String\>, // Ein kleiner Teil des fehlerhaften Inhalts  
          \#\[source\]  
          source: Option\<serde\_json::Error\>, // Beispiel für serde\_json  
      },

      \#  
      PersistentIdNotFound { persistent\_id: String },

      \#  
      DuplicatePersistentId { persistent\_id: String },

      \#  
      VersionMismatch { expected: Option\<String\>, found: Option\<String\> },

      \#\[error("An internal error occurred in workspace configuration logic: {context}")\]  
      Internal { context: String },  
  }

* Erläuterung und Anwendung von Fehlerbehandlungsprinzipien:  
  Auch WorkspaceConfigError nutzt thiserror. Die Varianten LoadError und SaveError verwenden \#\[source\], um den zugrundeliegenden CoreConfigError (aus core::config) als Ursache einzubetten.3 Dies ist wichtig, um die Fehlerkette bis zum ursprünglichen I/O- oder Berechtigungsfehler zurückverfolgen zu können.  
  Ein besonderer Aspekt ist der Umgang mit Fehlern aus externen Bibliotheken, wie z.B. serde\_json::Error für die (De-)Serialisierung. Die Varianten SerializationError und DeserializationError sind so gestaltet, dass sie den ursprünglichen serde-Fehler als source aufnehmen können. Dies ist der direkten Konvertierung des Fehlers in einen String vorzuziehen, da so mehr Informationen für die Diagnose erhalten bleiben.  
  * Wenn serde\_json::Error direkt als source verwendet wird (z.B. \#\[source\] source: serde\_json::Error), kann der Aufrufer den Fehler heruntercasten und spezifische Details des serde-Fehlers untersuchen.  
  * Die message-Felder in diesen Varianten können entweder die Display-Ausgabe des serde-Fehlers oder eine benutzerdefinierte, kontextreichere Nachricht enthalten.  
  * Das Feld snippet in DeserializationError kann einen kleinen Ausschnitt der fehlerhaften Daten enthalten, was die Fehlersuche erheblich erleichtert.

Die Varianten PersistentIdNotFound und DuplicatePersistentId dienen der Validierung der semantischen Korrektheit der geladenen Konfigurationsdaten. VersionMismatch ist vorgesehen, um zukünftige Änderungen am Konfigurationsformat handhaben zu können.  
**Tabelle: WorkspaceConfigError Varianten**

| Variante | \#\[error("...")\]-Meldung (Auszug) | Semantische Bedeutung/Ursache | Enthaltene Daten/Quellfehler |
| :---- | :---- | :---- | :---- |
| LoadError | "Failed to load workspace configuration from '{path}'..." | Fehler beim Lesen der Konfigurationsdatei (I/O, Berechtigungen). | path: String, source: CoreConfigError. |
| SaveError | "Failed to save workspace configuration to '{path}'..." | Fehler beim Schreiben der Konfigurationsdatei (I/O, Berechtigungen, Speicherplatz). | path: String, source: CoreConfigError. |
| InvalidData | "Workspace configuration data is invalid or corrupt: {reason}..." | Die gelesenen Daten sind nicht im erwarteten Format oder semantisch inkonsistent (über (De-)Serialisierungsfehler hinaus). | reason: String, path: Option\<String\>. |
| SerializationError | "Serialization error for workspace configuration: {message}" | Fehler bei der Umwandlung der WorkspaceSetSnapshot-Struktur in ein serialisiertes Format (z.B. JSON). | message: String, source: Option\<serde\_json::Error\>. |
| DeserializationError | "Deserialization error for workspace configuration: {message}..." | Fehler bei der Umwandlung von serialisierten Daten (z.B. JSON-String) in die WorkspaceSetSnapshot-Struktur. | message: String, snippet: Option\<String\>, source: Option\<serde\_json::Error\>. |
| PersistentIdNotFound | "Persistent ID '{persistent\_id}' referenced in configuration..." | Eine in der Konfiguration referenzierte persistente ID (z.B. für den aktiven Workspace) existiert nicht in der Liste der geladenen Workspaces. | persistent\_id: String. |
| DuplicatePersistentId | "Duplicate persistent ID '{persistent\_id}' found..." | Mindestens zwei Workspaces in der Konfiguration haben dieselbe persistente ID. | persistent\_id: String. |
| VersionMismatch | "The configuration version is incompatible..." | Die Version der geladenen Konfigurationsdatei stimmt nicht mit der erwarteten Version überein. | expected: Option\<String\>, found: Option\<String\>. |
| Internal { context: String } | "An internal error occurred..." | Ein unerwarteter interner Fehler in der Konfigurationslogik. | context: String. |

### **5.6. Detaillierte Implementierungsschritte und Dateistruktur**

* **Dateistruktur innerhalb von src/domain/workspaces/config/:**  
  * mod.rs: Enthält die Definitionen der Snapshot-Strukturen (WorkspaceSnapshot, WorkspaceSetSnapshot), des WorkspaceConfigProvider-Traits und der konkreten Implementierung(en) wie FilesystemConfigProvider.  
  * errors.rs: Enthält die Definition des WorkspaceConfigError-Enums.  
* **Implementierungsschritte für FilesystemConfigProvider (Beispiel):**  
  1. Definiere das WorkspaceConfigError-Enum in errors.rs.  
  2. Definiere die Structs WorkspaceSnapshot und WorkspaceSetSnapshot in mod.rs und leite serde::Serialize sowie serde::Deserialize für sie ab.  
  3. Definiere das WorkspaceConfigProvider-Trait in mod.rs.  
  4. Implementiere das FilesystemConfigProvider-Struct (wie in 5.2 gezeigt) in mod.rs.  
  5. Implementiere das WorkspaceConfigProvider-Trait für FilesystemConfigProvider:  
     * **load\_workspace\_config():**  
       1. Rufe self.config\_service.read\_config\_file(\&self.config\_file\_name) auf, um den Inhalt der Konfigurationsdatei als String zu lesen.  
       2. Bei einem Fehler vom config\_service (z.B. Datei nicht gefunden, keine Leseberechtigung), mappe diesen CoreConfigError zu WorkspaceConfigError::LoadError { path: self.config\_file\_name.clone(), source: core\_err } und gib ihn zurück.  
          * Speziell der Fall "Datei nicht gefunden" (CoreConfigError::NotFound oder ähnlich) sollte vom Aufrufer (dem WorkspaceManager) ggf. als nicht-kritischer Fehler behandelt werden (z.B. um Standard-Workspaces zu erstellen). Diese Methode sollte den Fehler jedoch korrekt signalisieren.  
       3. Versuche, den gelesenen String-Inhalt mittels serde\_json::from\_str::\<WorkspaceSetSnapshot\>(content\_str) (oder dem entsprechenden Parser für das gewählte Format) zu deserialisieren.  
       4. Bei einem Deserialisierungsfehler, mappe den serde\_json::Error zu WorkspaceConfigError::DeserializationError { message: serde\_err.to\_string(), snippet: Some(...), source: Some(serde\_err) } und gib ihn zurück. Der snippet sollte einen kleinen Teil des problematischen Inhalts enthalten.  
       5. Führe nach erfolgreicher Deserialisierung Validierungen auf dem WorkspaceSetSnapshot durch:  
          * Prüfe auf doppelte persistent\_ids in snapshot.workspaces. Falls Duplikate gefunden werden, gib Err(WorkspaceConfigError::DuplicatePersistentId {... }) zurück.  
          * Wenn snapshot.active\_workspace\_persistent\_id Some(active\_pid) ist, prüfe, ob ein Workspace mit dieser persistent\_id auch in snapshot.workspaces existiert. Falls nicht, gib Err(WorkspaceConfigError::PersistentIdNotFound {... }) zurück.  
       6. Gib bei Erfolg Ok(snapshot) zurück.  
     * **save\_workspace\_config(config\_snapshot: \&WorkspaceSetSnapshot):**  
       1. Serialisiere das config\_snapshot-Objekt mittels serde\_json::to\_string\_pretty(config\_snapshot) (oder dem entsprechenden Serialisierer) in einen String. to\_string\_pretty wird für bessere Lesbarkeit der Konfigurationsdatei empfohlen.  
       2. Bei einem Serialisierungsfehler, mappe den serde\_json::Error zu WorkspaceConfigError::SerializationError { message: serde\_err.to\_string(), source: Some(serde\_err) } und gib ihn zurück.  
       3. Rufe self.config\_service.write\_config\_file(\&self.config\_file\_name, serialized\_content) auf, um den serialisierten String in die Konfigurationsdatei zu schreiben.  
       4. Bei einem Fehler vom config\_service (z.B. keine Schreibberechtigung, kein Speicherplatz), mappe diesen CoreConfigError zu WorkspaceConfigError::SaveError { path: self.config\_file\_name.clone(), source: core\_err } und gib ihn zurück.  
       5. Gib bei Erfolg Ok(()) zurück.  
  6. Stelle sicher, dass alle öffentlichen Elemente (Traits, Structs, Methoden) umfassend mit rustdoc dokumentiert sind.  
  7. Erstelle Unit-Tests für FilesystemConfigProvider. Diese Tests sollten:  
     * Einen gemockten ConfigService verwenden, um Lese- und Schreiboperationen zu simulieren, ohne auf das tatsächliche Dateisystem zuzugreifen.  
     * Erfolgreiches Laden und Speichern von gültigen WorkspaceSetSnapshot-Daten testen.  
     * Alle Fehlerfälle testen: I/O-Fehler (simuliert durch den Mock), (De-)Serialisierungsfehler mit ungültigen Daten, Validierungsfehler (doppelte IDs, nicht gefundene aktive ID).  
     * Testen des Verhaltens, wenn die Konfigurationsdatei nicht existiert (simulierter CoreConfigError::NotFound).

## **6\. Integrationsleitfaden für die Komponente domain::workspaces**

Dieser Abschnitt beschreibt das Zusammenspiel der vier Module innerhalb der domain::workspaces-Komponente und deren Interaktion mit anderen Teilen des Systems.

### **6.1. Zusammenwirken der Module**

Die vier Module (core, assignment, manager, config) der domain::workspaces-Komponente sind so konzipiert, dass sie eng zusammenarbeiten, wobei jedes Modul klar definierte Verantwortlichkeiten hat:

1. **workspaces::manager als zentraler Koordinator:**  
   * Der WorkspaceManager ist die Hauptschnittstelle und der Orchestrator für alle Workspace-Operationen.  
   * Er initialisiert sich selbst, indem er über einen WorkspaceConfigProvider (aus workspaces::config) die gespeicherte Workspace-Konfiguration lädt.  
   * Er hält eine interne Sammlung (HashMap und Vec) von Workspace-Instanzen (definiert in workspaces::core).  
   * Für Operationen, die die Zuweisung von Fenstern zu Workspaces betreffen (z.B. assign\_window\_to\_active\_workspace), delegiert der WorkspaceManager die Logik an die Funktionen des workspaces::assignment-Moduls und übergibt dabei seine interne Workspace-Sammlung.  
   * Bei Änderungen, die persistiert werden müssen (z.B. Erstellung eines neuen Workspace, Umbenennung, Änderung der Reihenfolge, Änderung des aktiven Workspace), erstellt der WorkspaceManager einen WorkspaceSetSnapshot und nutzt den WorkspaceConfigProvider aus workspaces::config, um diesen zu speichern.  
   * Der WorkspaceManager ist verantwortlich für das Publizieren von WorkspaceEvents, um andere Systemteile über relevante Änderungen zu informieren.  
2. **workspaces::core als Fundament:**  
   * Stellt die Definition des Workspace-Structs und zugehöriger Typen (WindowIdentifier, WorkspaceLayoutType) sowie der Event-Payload-Datenstrukturen bereit.  
   * Workspace-Instanzen werden vom WorkspaceManager gehalten und modifiziert (z.B. durch Aufruf von Workspace::rename()).  
3. **workspaces::assignment als Dienstleister für Zuweisungslogik:**  
   * Stellt zustandslose Funktionen bereit, die auf der vom WorkspaceManager übergebenen Sammlung von Workspace-Objekten operieren, um Fenster zuzuweisen, zu entfernen oder zu verschieben.  
   * Modifiziert die window\_ids-Mengen innerhalb der Workspace-Objekte.  
4. **workspaces::config als Persistenzabstraktion:**  
   * Definiert die Schnittstelle (WorkspaceConfigProvider) und die Datenstrukturen (WorkspaceSnapshot, WorkspaceSetSnapshot) für das Laden und Speichern der Workspace-Konfiguration.  
   * Konkrete Implementierungen (z.B. FilesystemConfigProvider) nutzen Dienste der Kernschicht (core::config) für den eigentlichen Dateizugriff.

Dieses Design fördert Modularität und Testbarkeit. Der WorkspaceManager kann beispielsweise mit gemockten WorkspaceConfigProvider- und EventPublisher-Implementierungen getestet werden.

### **6.2. Abhängigkeiten und Schnittstellen zu anderen Domänenkomponenten und Schichten**

Die domain::workspaces-Komponente interagiert mit und hat Abhängigkeiten zu folgenden anderen Teilen des Systems:

* **Kernschicht (Core Layer):**  
  * **core::config:** Wird von workspaces::config (konkret von FilesystemConfigProvider) genutzt, um auf das Dateisystem zuzugreifen und Konfigurationsdateien zu lesen/schreiben.  
  * **core::errors:** Basisfehlertypen (z.B. ValidationError, ConfigError aus core::config) können von den spezifischen Fehler-Enums der Workspace-Module (WorkspaceCoreError, WorkspaceConfigError) via \#\[from\] referenziert und gewrappt werden.  
  * **core::types:** Fundamentale Typen wie uuid::Uuid (für WorkspaceId) werden direkt genutzt. Andere Typen (z.B. chrono::DateTime) für Zeitstempel.  
  * **core::logging (implizit):** Alle Module der domain::workspaces-Komponente sollten das tracing-Framework der Kernschicht für Logging und Tracing verwenden, wie in Richtlinie 4.4 spezifiziert.  
* **Andere Domänenkomponenten (Domain Layer):**  
  * **domain::window\_management (Policy):**  
    * Diese Komponente definiert die übergeordneten Regeln für Fensterplatzierung und \-verhalten. Sie könnte auf WorkspaceEvents (z.B. ActiveWorkspaceChanged, WindowAddedToWorkspace, WorkspaceLayoutChanged) vom workspaces::manager lauschen, um ihre Layout-Algorithmen oder Fensteranordnungen anzupassen.  
    * Umgekehrt könnte domain::window\_management Regeln bereitstellen (z.B. "Anwendung X immer auf Workspace Y öffnen"), die der workspaces::manager oder workspaces::assignment bei der initialen Zuweisung eines neuen Fensters berücksichtigen muss. Dies könnte über eine direkte Abfrage oder eine Konfigurationsschnittstelle erfolgen.  
  * **domain::settings:**  
    * Globale Desktop-Einstellungen (z.B. "Standardanzahl der Workspaces beim ersten Start", "Verhalten beim Schließen des letzten Fensters auf einem Workspace") könnten das Initialisierungs- oder Betriebsverhalten des workspaces::manager beeinflussen. Der WorkspaceManager könnte diese Einstellungen beim Start abfragen.  
  * **domain::ai (indirekt):**  
    * KI-Funktionen könnten kontextabhängig von Workspaces agieren (z.B. "fasse die Fenster auf dem aktuellen Workspace zusammen"). In diesem Fall würde domain::ai Informationen über den aktiven Workspace und dessen Fenster vom workspaces::manager abfragen.  
* **Systemschicht (System Layer):**  
  * **Compositor (system::compositor):**  
    * Informiert den workspaces::manager (oder eine übergeordnete Fassade in der Systemschicht, die mit dem Manager kommuniziert), wenn neue Fenster (Wayland Surfaces) erstellt oder zerstört werden. Diese Information ist notwendig, damit der WorkspaceManager die Fenster den Workspaces zuordnen kann.  
    * Wird vom workspaces::manager (oft indirekt über domain::window\_management) angewiesen, welche Fenster auf dem aktuell aktiven Workspace sichtbar gemacht und welche verborgen werden sollen.  
    * Setzt Fokusregeln basierend auf dem aktiven Workspace und den Anweisungen aus der Domänenschicht um.  
  * **D-Bus-Schnittstellen (system::dbus):**  
    * Der WorkspaceManager könnte seine API (oder Teile davon) über D-Bus exponieren, um externen Werkzeugen oder Skripten die Steuerung von Workspaces zu ermöglichen.  
    * Umgekehrt könnte der WorkspaceManager auf D-Bus-Signale von Systemdiensten lauschen, falls diese für die Workspace-Logik relevant sind.  
* **Benutzeroberflächenschicht (User Interface Layer):**  
  * **Shell-UI (ui::shell), Pager, Fensterwechsler (ui::window\_manager\_frontend):**  
    * Nutzt die API des workspaces::manager intensiv, um die Liste der Workspaces abzurufen und darzustellen, den aktiven Workspace hervorzuheben, das Wechseln zwischen Workspaces zu ermöglichen und die Erstellung/Löschung/Umbenennung von Workspaces durch Benutzeraktionen anzustoßen.  
    * Reagiert auf WorkspaceEvents vom WorkspaceManager, um die Benutzeroberfläche dynamisch zu aktualisieren, wenn sich der Workspace-Zustand ändert (z.B. neuer Workspace erscheint im Pager, Fensterliste für aktiven Workspace wird aktualisiert).

### **6.3. Sequenzdiagramme für typische Anwendungsfälle**

Die folgenden Beschreibungen skizzieren die Interaktionen für typische Anwendungsfälle. In einer vollständigen grafischen Dokumentation würden hier UML-Sequenzdiagramme stehen.

1. **Erstellung eines neuen Workspace durch Benutzeraktion:**  
   * User interagiert mit der UI-Schicht (z.B. Klick auf "Neuer Workspace"-Button).  
   * UI-Schicht ruft WorkspaceManager::create\_workspace(name, persistent\_id) auf.  
   * WorkspaceManager validiert Eingaben, generiert ggf. Standardnamen.  
   * WorkspaceManager ruft Workspace::new(final\_name, persistent\_id) (aus workspaces::core) auf, um eine neue Workspace-Instanz zu erstellen.  
     * Workspace::new gibt Ok(new\_workspace) oder Err(WorkspaceCoreError) zurück.  
   * WorkspaceManager fügt new\_workspace seiner internen Sammlung hinzu.  
   * WorkspaceManager publiziert ein WorkspaceEvent::WorkspaceCreated über seinen EventPublisher.  
   * WorkspaceManager ruft self.save\_configuration() auf, was intern den WorkspaceConfigProvider::save\_workspace\_config() (aus workspaces::config) aufruft.  
     * WorkspaceConfigProvider serialisiert den Zustand und nutzt core::config::ConfigService zum Schreiben.  
   * WorkspaceManager gibt Ok(new\_workspace\_id) an die UI-Schicht zurück.  
   * UI-Schicht (als Subscriber des WorkspaceEvent::WorkspaceCreated) aktualisiert die Darstellung (z.B. fügt neuen Workspace-Tab hinzu).  
2. **Ein neues Fenster wird erstellt und dem aktiven Workspace zugewiesen:**  
   * Systemschicht (Compositor) erkennt ein neues Fenster (z.B. neues Wayland Surface) und generiert eine WindowIdentifier.  
   * Systemschicht benachrichtigt den WorkspaceManager (ggf. über eine Fassade oder einen System-Event) über das neue Fenster: handle\_new\_window(window\_id).  
   * WorkspaceManager::handle\_new\_window (oder eine ähnliche Methode) ruft intern WorkspaceManager::assign\_window\_to\_active\_workspace(\&window\_id) auf.  
   * WorkspaceManager::assign\_window\_to\_active\_workspace prüft, ob ein aktiver Workspace existiert.  
   * WorkspaceManager ruft workspaces::assignment::assign\_window\_to\_workspace(\&mut self.workspaces, active\_ws\_id, \&window\_id, self.ensure\_unique\_window\_assignment) auf.  
     * assignment::assign\_window\_to\_workspace modifiziert das Workspace-Objekt des aktiven Workspace (fügt window\_id zu dessen window\_ids-Set hinzu) und entfernt es ggf. von anderen Workspaces.  
     * Gibt Ok(()) oder Err(WindowAssignmentError) zurück.  
   * WorkspaceManager publiziert WorkspaceEvent::WindowAddedToWorkspace (und ggf. WindowRemovedFromWorkspace falls von einem anderen WS entfernt) über seinen EventPublisher.  
   * WorkspaceManager gibt Erfolg/Fehler an den Aufrufer (Systemschicht) zurück.  
   * UI-Schicht (als Subscriber) aktualisiert ggf. die Fensterliste für den aktiven Workspace.  
   * domain::window\_management (als Subscriber) könnte auf das Event reagieren, um das neue Fenster gemäß den Layout-Regeln des aktiven Workspace zu positionieren.  
3. **Laden der Workspace-Konfiguration beim Start des WorkspaceManager:**  
   * Eine übergeordnete Komponente (z.B. Desktop-Initialisierungsdienst) ruft WorkspaceManager::new(config\_provider, event\_publisher,...) auf.  
   * WorkspaceManager::new ruft config\_provider.load\_workspace\_config() (aus workspaces::config) auf.  
   * FilesystemConfigProvider::load\_workspace\_config (Implementierung von WorkspaceConfigProvider):  
     * Ruft core::config::ConfigService::read\_config\_file(...) auf, um Rohdaten zu laden.  
     * Deserialisiert die Rohdaten in ein WorkspaceSetSnapshot.  
     * Validiert den Snapshot (z.B. auf doppelte persistente IDs).  
     * Gibt Ok(snapshot) oder Err(WorkspaceConfigError) zurück.  
   * WorkspaceManager::new verarbeitet das Result:  
     * Bei Ok(snapshot): Erstellt Workspace-Instanzen aus den WorkspaceSnapshots, füllt self.workspaces und self.ordered\_workspace\_ids. Setzt self.active\_workspace\_id basierend auf snapshot.active\_workspace\_persistent\_id. Publiziere WorkspacesReloaded und ActiveWorkspaceChanged.  
     * Bei Err(WorkspaceConfigError::LoadError { source: CoreConfigError::NotFound,.. }) (oder ähnlicher Fehler, der "Datei nicht gefunden" anzeigt): Erstellt einen oder mehrere Standard-Workspaces, fügt sie hinzu, setzt einen als aktiv. Publiziere WorkspaceCreated und ActiveWorkspaceChanged.  
     * Bei anderen Err(config\_err): Gibt Err(WorkspaceManagerError::ConfigError(config\_err)) zurück.  
   * WorkspaceManager::new gibt Ok(self) oder Err(WorkspaceManagerError) an den Aufrufer zurück.

## **7\. Anhang: Referenzierte Richtlinien zur Fehlerbehandlung**

Dieser Anhang fasst die zentralen Prinzipien und Entscheidungen zur Fehlerbehandlung zusammen, die für die Implementierung der domain::workspaces-Komponente und darüber hinaus im gesamten Projekt gelten. Diese basieren auf Richtlinie 4.3 der Gesamtspezifikation und den Erkenntnissen aus der Analyse etablierter Rust-Fehlerbehandlungspraktiken.1

* **Verwendung von thiserror pro Modul:** Jedes Modul (z.B. workspaces::core, workspaces::assignment) definiert sein eigenes spezifisches Fehler-Enum unter Verwendung des thiserror-Crates. Dies reduziert Boilerplate und fördert klar definierte Fehlergrenzen zwischen Modulen.1  
* **Klare und kontextreiche Fehlernachrichten:** Jede Variante eines Fehler-Enums muss eine präzise, entwicklerorientierte Fehlermeldung über das \#\[error("...")\]-Attribut bereitstellen. Diese Nachricht sollte den Fehler eindeutig beschreiben.  
* **Fehlervarianten mit Datenanreicherung:** Wo immer es für die Fehlerdiagnose oder die programmatische Fehlerbehandlung durch den Aufrufer nützlich ist, sollen Fehlervarianten relevante Daten als Felder enthalten. Dies können ungültige Eingabewerte, Zustandsinformationen zum Zeitpunkt des Fehlers oder andere kontextrelevante Details sein. Dies hilft, das "Context Blurring"-Problem zu vermeiden, bei dem generische Fehler nicht genügend Informationen liefern.1  
* **Nutzung von \#\[from\] für Fehlerkonvertierung:** Das \#\[from\]-Attribut von thiserror soll verwendet werden, um Fehler aus abhängigen Modulen oder Bibliotheken einfach in den Fehlertyp des aktuellen Moduls zu konvertieren. Dies erleichtert die Fehlerpropagierung mit dem ?-Operator und stellt sicher, dass die std::error::Error::source()-Kette erhalten bleibt, sodass die ursprüngliche Fehlerursache zurückverfolgt werden kann.3  
* **Spezifische Varianten bei unzureichendem Kontext durch \#\[from\]:** Wenn ein via \#\[from\] gewrappter Fehler zu generisch ist und der spezifische Kontext der fehlgeschlagenen Operation im aktuellen Modul verloren ginge, soll eine spezifischere Fehlervariante im aktuellen Modul-Error-Enum erstellt werden. Diese spezifischere Variante sollte den ursprünglichen Fehler explizit über das \#\[source\]-Attribut einbetten und zusätzliche Felder für den Kontext der aktuellen Operation enthalten.  
* **Vermeidung von unwrap() und expect():** In Bibliotheks-, Kern- und Domänencode ist die Verwendung von unwrap() und expect() zur Fehlerbehandlung strikt zu vermeiden. Alle vorhersehbaren Fehler müssen über das Result\<T, E\>-Typsystem explizit behandelt und propagiert werden. Panics sind nur für nicht behebbare Fehler oder in Tests und Beispielen akzeptabel.1  
* **Semantik der Display-Implementierung:** Die durch \#\[error("...")\] generierte Display-Implementierung von Fehlern ist primär für Entwickler (Logging, Debugging) gedacht. Die Benutzeroberflächenschicht ist dafür verantwortlich, diese technischen Fehler – basierend auf der semantischen Bedeutung der jeweiligen Fehlervariante – in benutzerfreundliche und ggf. lokalisierte Nachrichten zu übersetzen.  
* **Umgang mit Fehlern aus externen Bibliotheken:** Fehler aus externen Bibliotheken (z.B. serde\_json::Error) sollten ebenfalls in die modul-spezifischen Fehler-Enums integriert werden, idealerweise unter Beibehaltung des Originalfehlers als source. Dies kann durch \#\[from\] oder durch eine Variante mit einem \#\[source\]-Feld geschehen. Die direkte Konvertierung des externen Fehlers in einen String sollte vermieden werden, wenn dadurch wertvolle Diagnoseinformationen verloren gehen.

Die konsequente Anwendung dieser Richtlinien ist entscheidend für die Entwicklung einer robusten, wartbaren und gut diagnostizierbaren Desktop-Umgebung. Sie stellt sicher, dass Fehler nicht verschleiert werden, sondern klar und mit ausreichend Kontext an die entsprechenden Stellen im System weitergeleitet werden können.

---

# **B3 Domänenschicht: Detaillierte Spezifikation – Teil 3/4: Benutzerzentrierte Dienste und Globale Einstellungsverwaltung**

Dieser Abschnitt des Dokuments setzt die detaillierte Spezifikation der Domänenschicht fort und konzentriert sich auf zwei Entwicklungsmodule: domain::user\_centric\_services und domain::global\_settings\_and\_state\_management. Diese Module sind entscheidend für die Implementierung intelligenter Benutzerinteraktionen, die Verwaltung von Benachrichtigungen und die Konfiguration des Desktops.  
---

**Entwicklungsmodul C: domain::user\_centric\_services**  
Dieses Modul bündelt die Logik für Dienste, die direkt auf die Bedürfnisse und Interaktionen des Benutzers ausgerichtet sind. Es umfasst die Verwaltung von KI-Interaktionen, einschließlich des Einwilligungsmanagements, sowie ein umfassendes Benachrichtigungssystem.  
**1\. Modulübersicht und Verantwortlichkeiten (domain::user\_centric\_services)**

* **Zweck:** Das Modul domain::user\_centric\_services dient als zentrale Komponente für die Orchestrierung von Benutzerinteraktionen, die über Standard-Desktop-Funktionen hinausgehen. Es stellt die Domänenlogik für KI-gestützte Assistenzfunktionen und ein robustes System zur Verwaltung von Benachrichtigungen bereit.  
* **Kernaufgaben:**  
  * **KI-Interaktionsmanagement:**  
    * Verwaltung des Lebenszyklus von KI-Interaktionskontexten.  
    * Implementierung der Logik für das Einholen, Speichern und Überprüfen von Benutzereinwilligungen (AIConsent) für die Nutzung von KI-Modellen und den Zugriff auf spezifische Datenkategorien (AIDataCategory).  
    * Verwaltung von Profilen verfügbarer KI-Modelle (AIModelProfile).  
    * Bereitstellung einer Schnittstelle zur Initiierung von KI-Aktionen und zur Verarbeitung von deren Ergebnissen, unabhängig vom spezifischen KI-Modell oder dem MCP-Protokoll (welches in der Systemschicht implementiert wird).  
  * **Benachrichtigungsmanagement:**  
    * Entgegennahme, Verarbeitung und Speicherung von Benachrichtigungen (Notification).  
    * Verwaltung des Zustands von Benachrichtigungen (aktiv, gelesen, abgewiesen).  
    * Implementierung einer Benachrichtigungshistorie mit konfigurierbarer Größe.  
    * Unterstützung für verschiedene Dringlichkeitsstufen (NotificationUrgency) und Aktionen (NotificationAction).  
    * Bereitstellung einer "Bitte nicht stören" (DND) Funktionalität.  
    * Ermöglichung des Filterns und Sortierens von Benachrichtigungen.  
* **Abgrenzung:**  
  * Dieses Modul implementiert *nicht* die UI-Elemente zur Darstellung von KI-Interaktionen oder Benachrichtigungen (dies ist Aufgabe der User Interface Layer).  
  * Es implementiert *nicht* die direkte Kommunikation mit KI-Modellen oder Systemdiensten wie dem D-Bus Notification Daemon (dies ist Aufgabe der System Layer). Es definiert die Logik und den Zustand, die von diesen Schichten genutzt werden.  
  * Die Persistenz von Einwilligungen oder Modellprofilen wird an die Core Layer (z.B. core::config) delegiert.  
* **Zugehörige Komponenten aus der Gesamtübersicht:** domain::ai, domain::notifications.

**2\. Datenstrukturen und Typdefinitionen (Rust) für domain::user\_centric\_services**  
Die folgenden Datenstrukturen definieren die Kernentitäten und Wertobjekte des Moduls. Sie sind so konzipiert, dass sie die notwendigen Informationen für die KI-Interaktions- und Benachrichtigungslogik kapseln.

* **2.1. Entitäten und Wertobjekte:**  
  * **AIInteractionContext (Entität):** Repräsentiert eine spezifische Interaktion oder einen Dialog mit einer KI.  
    * Attribute:  
      * id: Uuid (öffentlich): Eindeutiger Identifikator für den Kontext.  
      * creation\_timestamp: DateTime\<Utc\> (öffentlich): Zeitpunkt der Erstellung.  
      * active\_model\_id: Option\<String\> (öffentlich): ID des aktuell für diesen Kontext relevanten KI-Modells.  
      * consent\_status: AIConsentStatus (öffentlich): Aktueller Einwilligungsstatus für diesen Kontext.  
      * associated\_data\_categories: Vec\<AIDataCategory\> (öffentlich): Kategorien von Daten, die für diese Interaktion relevant sein könnten.  
      * interaction\_history: Vec\<String\> (privat, modifizierbar über Methoden): Eine einfache Historie der Konversation (z.B. Benutzeranfragen, KI-Antworten).  
      * attachments: Vec\<AttachmentData\> (öffentlich): Angehängte Daten (z.B. Dateipfade, Text-Snippets).  
    * Invarianten: id ist unveränderlich nach Erstellung. creation\_timestamp ist unveränderlich.  
    * Methoden (konzeptionell):  
      * new(relevant\_categories: Vec\<AIDataCategory\>) \-\> Self: Erstellt einen neuen Kontext.  
      * update\_consent\_status(\&mut self, status: AIConsentStatus): Aktualisiert den Einwilligungsstatus.  
      * set\_active\_model(\&mut self, model\_id: String): Legt das aktive Modell fest.  
      * add\_history\_entry(\&mut self, entry: String): Fügt einen Eintrag zur Historie hinzu.  
      * add\_attachment(\&mut self, attachment: AttachmentData): Fügt einen Anhang hinzu.  
  * **AIConsent (Entität):** Repräsentiert die Einwilligung eines Benutzers für eine spezifische Kombination aus KI-Modell und Datenkategorien.  
    * Attribute:  
      * id: Uuid (öffentlich): Eindeutiger Identifikator für die Einwilligung.  
      * user\_id: String (öffentlich, vereinfacht): Identifikator des Benutzers.  
      * model\_id: String (öffentlich): ID des KI-Modells, für das die Einwilligung gilt.  
      * data\_categories: Vec\<AIDataCategory\> (öffentlich): Datenkategorien, für die die Einwilligung erteilt wurde.  
      * granted\_timestamp: DateTime\<Utc\> (öffentlich): Zeitpunkt der Erteilung.  
      * expiry\_timestamp: Option\<DateTime\<Utc\>\> (öffentlich): Optionaler Ablaufzeitpunkt der Einwilligung.  
      * is\_revoked: bool (öffentlich, initial false): Gibt an, ob die Einwilligung widerrufen wurde.  
    * Invarianten: id, user\_id, model\_id, granted\_timestamp sind nach Erstellung unveränderlich. data\_categories sollten nach Erteilung nicht ohne Weiteres modifizierbar sein (neue Einwilligung erforderlich).  
    * Methoden (konzeptionell):  
      * new(user\_id: String, model\_id: String, categories: Vec\<AIDataCategory\>, expiry: Option\<DateTime\<Utc\>\>) \-\> Self.  
      * revoke(\&mut self): Markiert die Einwilligung als widerrufen.  
  * **AIModelProfile (Entität):** Beschreibt ein verfügbares KI-Modell.  
    * Attribute:  
      * model\_id: String (öffentlich): Eindeutiger Identifikator des Modells.  
      * display\_name: String (öffentlich): Anzeigename des Modells.  
      * description: String (öffentlich): Kurze Beschreibung des Modells.  
      * provider: String (öffentlich): Anbieter des Modells (z.B. "Local", "OpenAI").  
      * required\_consent\_categories: Vec\<AIDataCategory\> (öffentlich): Datenkategorien, für die dieses Modell typischerweise eine Einwilligung benötigt.  
      * capabilities: Vec\<String\> (öffentlich): Liste der Fähigkeiten des Modells (z.B. "text\_generation", "summarization").  
    * Invarianten: model\_id ist eindeutig und unveränderlich.  
    * Methoden (konzeptionell):  
      * new(...) \-\> Self.  
      * requires\_consent\_for(\&self, categories: &) \-\> bool: Prüft, ob für die gegebenen Kategorien eine Einwilligung erforderlich ist.  
  * **Notification (Entität):** Repräsentiert eine einzelne Benachrichtigung.  
    * Attribute:  
      * id: Uuid (öffentlich): Eindeutiger Identifikator.  
      * application\_name: String (öffentlich): Name der Anwendung, die die Benachrichtigung gesendet hat.  
      * application\_icon: Option\<String\> (öffentlich): Optionaler Pfad oder Name des Icons der Anwendung.  
      * summary: String (öffentlich): Kurze Zusammenfassung der Benachrichtigung.  
      * body: Option\<String\> (öffentlich): Detaillierterer Text der Benachrichtigung.  
      * actions: Vec\<NotificationAction\> (öffentlich): Verfügbare Aktionen für die Benachrichtigung.  
      * urgency: NotificationUrgency (öffentlich): Dringlichkeitsstufe.  
      * timestamp: DateTime\<Utc\> (öffentlich): Zeitpunkt des Eintreffens.  
      * is\_read: bool (privat, initial false): Status, ob gelesen.  
      * is\_dismissed: bool (privat, initial false): Status, ob vom Benutzer aktiv geschlossen.  
      * transient: bool (öffentlich, default false): Ob die Benachrichtigung flüchtig ist und nicht in der Historie verbleiben soll.  
    * Invarianten: id, timestamp sind unveränderlich. summary darf nicht leer sein.  
    * Methoden (konzeptionell):  
      * new(app\_name: String, summary: String, urgency: NotificationUrgency) \-\> Self.  
      * mark\_as\_read(\&mut self).  
      * dismiss(\&mut self).  
      * add\_action(\&mut self, action: NotificationAction).  
  * **NotificationAction (Wertobjekt):** Definiert eine Aktion, die im Kontext einer Benachrichtigung ausgeführt werden kann.  
    * Attribute:  
      * key: String (öffentlich): Eindeutiger Schlüssel für die Aktion (z.B. "reply", "archive").  
      * label: String (öffentlich): Anzeigename der Aktion.  
      * action\_type: NotificationActionType (öffentlich): Typ der Aktion (z.B. Callback, Link).  
  * **AttachmentData (Wertobjekt):** Repräsentiert angehängte Daten an einen AIInteractionContext.  
    * Attribute:  
      * id: Uuid (öffentlich): Eindeutiger Identifikator des Anhangs.  
      * mime\_type: String (öffentlich): MIME-Typ der Daten (z.B. "text/plain", "image/png").  
      * source\_uri: Option\<String\> (öffentlich): URI zur Quelle der Daten (z.B. file:///path/to/file).  
      * content: Option\<Vec\<u8\>\> (öffentlich): Direkter Inhalt der Daten, falls klein.  
      * description: Option\<String\> (öffentlich): Optionale Beschreibung des Anhangs.  
* **2.2. Modulspezifische Enums, Konstanten und Konfigurationsstrukturen:**  
  * **Enums:**  
    * AIConsentStatus: Enum (Granted, Denied, PendingUserAction, NotRequired).  
    * AIDataCategory: Enum (UserProfile, ApplicationUsage, FileSystemRead, ClipboardAccess, LocationData, GenericText, GenericImage).  
    * NotificationUrgency: Enum (Low, Normal, Critical).  
    * NotificationActionType: Enum (Callback, OpenLink).  
    * NotificationFilterCriteria: Enum (Unread, Application(String), Urgency(NotificationUrgency)).  
    * NotificationSortOrder: Enum (TimestampAscending, TimestampDescending, Urgency).  
  * **Konstanten:**  
    * const DEFAULT\_NOTIFICATION\_TIMEOUT\_SECS: u64 \= 5;  
    * const MAX\_NOTIFICATION\_HISTORY: usize \= 100;  
    * const MAX\_AI\_INTERACTION\_HISTORY: usize \= 50;  
* **2.3. Definition aller deklarierten Eigenschaften (Properties):**  
  * Für AIInteractionLogicService (als Trait implementiert):  
    * Keine direkten öffentlichen Eigenschaften, Zustand wird intern in der implementierenden Struktur gehalten (z.B. active\_contexts: HashMap\<Uuid, AIInteractionContext\>, consents: Vec\<AIConsent\>, model\_profiles: Vec\<AIModelProfile\>).  
  * Für NotificationService (als Trait implementiert):  
    * Keine direkten öffentlichen Eigenschaften, Zustand wird intern gehalten (z.B. active\_notifications: Vec\<Notification\>, history: VecDeque\<Notification\>, dnd\_enabled: bool).  
* **Wichtige Tabelle: Entitäten und Wertobjekte für domain::user\_centric\_services**

| Entität/Wertobjekt | Wichtige Attribute (Typ) | Kurzbeschreibung | Methoden (Beispiele) | Invarianten (Beispiele) |
| :---- | :---- | :---- | :---- | :---- |
| AIInteractionContext | id: Uuid, consent\_status: AIConsentStatus, associated\_data\_categories: Vec\<AIDataCategory\>, attachments: Vec\<AttachmentData\> | Repräsentiert eine laufende KI-Interaktion. | update\_consent\_status(), add\_attachment() | id ist unveränderlich. |
| AIConsent | model\_id: String, data\_categories: Vec\<AIDataCategory\>, granted\_timestamp: DateTime\<Utc\>, is\_revoked: bool | Speichert die Benutzereinwilligung für KI-Modell und Daten. | revoke() | model\_id, granted\_timestamp sind unveränderlich. |
| AIModelProfile | model\_id: String, display\_name: String, required\_consent\_categories: Vec\<AIDataCategory\>, capabilities: Vec\<String\> | Beschreibt ein verfügbares KI-Modell und dessen Anforderungen. | requires\_consent\_for() | model\_id ist eindeutig. |
| Notification | id: Uuid, summary: String, body: Option\<String\>, urgency: NotificationUrgency, is\_read: bool, actions: Vec\<NotificationAction\> | Repräsentiert eine System- oder Anwendungsbenachrichtigung. | mark\_as\_read(), dismiss(), add\_action() | id, timestamp sind unveränderlich. summary nicht leer. |
| NotificationAction | key: String, label: String, action\_type: NotificationActionType | Definiert eine ausführbare Aktion innerhalb einer Benachrichtigung. | \- | key ist eindeutig im Kontext der Benachrichtigung. |
| AttachmentData | id: Uuid, mime\_type: String, source\_uri: Option\<String\>, content: Option\<Vec\<u8\>\> | Repräsentiert angehängte Daten an einen AIInteractionContext. | \- | id ist eindeutig. Entweder source\_uri oder content sollte vorhanden sein. |

Diese tabellarische Übersicht fasst die zentralen Datenstrukturen zusammen. Die genaue Ausgestaltung der Attribute und Methoden ist für die korrekte Implementierung der Geschäftslogik entscheidend. Beispielsweise stellt die AIModelProfile-Struktur sicher, dass die Anforderungen eines Modells bezüglich der Dateneinwilligung klar definiert sind, was eine Kernanforderung für die KI-Integration darstellt.  
**3\. Öffentliche API und Interne Schnittstellen (Rust) für domain::user\_centric\_services**  
Die öffentliche API dieses Moduls wird durch Traits definiert, die von konkreten Service-Implementierungen erfüllt werden.

* **3.1. Exakte Signaturen aller öffentlichen Funktionen/Methoden:**  
  * **AIInteractionLogicService Trait:**  
    Rust  
    use crate::core::types::Uuid; // Standard Uuid Typ aus der Kernschicht  
    use crate::core::errors::CoreError; // Fehler aus der Kernschicht  
    use super::types::{AIInteractionContext, AIConsent, AIModelProfile, AIDataCategory, AttachmentData};  
    use super::errors::AIInteractionError;  
    use async\_trait::async\_trait;

    \#\[async\_trait\]  
    pub trait AIInteractionLogicService: Send \+ Sync {  
        /// Initiates a new AI interaction context.  
        /// Returns the ID of the newly created context.  
        async fn initiate\_interaction(  
            \&mut self,  
            relevant\_categories: Vec\<AIDataCategory\>,  
            initial\_attachments: Option\<Vec\<AttachmentData\>\>  
        ) \-\> Result\<Uuid, AIInteractionError\>;

        /// Retrieves an existing AI interaction context.  
        async fn get\_interaction\_context(\&self, context\_id: Uuid) \-\> Result\<AIInteractionContext, AIInteractionError\>;

        /// Provides or updates consent for a given interaction context and model.  
        async fn provide\_consent(  
            \&mut self,  
            context\_id: Uuid,  
            model\_id: String,  
            granted\_categories: Vec\<AIDataCategory\>,  
            consent\_decision: bool // true for granted, false for denied  
        ) \-\> Result\<(), AIInteractionError\>;

        /// Retrieves the consent status for a specific model and data categories,  
        /// potentially within an interaction context.  
        async fn get\_consent\_for\_model(  
            \&self,  
            model\_id: \&str,  
            data\_categories: &,  
            context\_id: Option\<Uuid\>  
        ) \-\> Result\<super::types::AIConsentStatus, AIInteractionError\>;

        /// Adds an attachment to an existing interaction context.  
        async fn add\_attachment\_to\_context(  
            \&mut self,  
            context\_id: Uuid,  
            attachment: AttachmentData  
        ) \-\> Result\<(), AIInteractionError\>;

        /// Lists all available and configured AI model profiles.  
        async fn list\_available\_models(\&self) \-\> Result\<Vec\<AIModelProfile\>, AIInteractionError\>;

        /// Stores a user's consent decision persistently.  
        /// This might be called after \`provide\_consent\` if the consent is to be remembered globally.  
        async fn store\_consent(\&self, consent: AIConsent) \-\> Result\<(), AIInteractionError\>;

        /// Retrieves all stored consents for a given user (simplified).  
        async fn get\_all\_user\_consents(\&self, user\_id: \&str) \-\> Result\<Vec\<AIConsent\>, AIInteractionError\>;

        /// Loads AI model profiles, e.g., from a configuration managed by core::config.  
        async fn load\_model\_profiles(\&mut self) \-\> Result\<(), AIInteractionError\>;  
    }

  * **NotificationService Trait:**  
    Rust  
    use crate::core::types::Uuid;  
    use crate::core::errors::CoreError;  
    use super::types::{Notification, NotificationUrgency, NotificationFilterCriteria, NotificationSortOrder};  
    use super::errors::NotificationError;  
    use async\_trait::async\_trait;

    \#\[async\_trait\]  
    pub trait NotificationService: Send \+ Sync {  
        /// Posts a new notification to the system.  
        /// Returns the ID of the newly created notification.  
        async fn post\_notification(\&mut self, notification\_data: Notification) \-\> Result\<Uuid, NotificationError\>;

        /// Retrieves a specific notification by its ID.  
        async fn get\_notification(\&self, notification\_id: Uuid) \-\> Result\<Notification, NotificationError\>;

        /// Marks a notification as read.  
        async fn mark\_as\_read(\&mut self, notification\_id: Uuid) \-\> Result\<(), NotificationError\>;

        /// Dismisses a notification, removing it from active view but possibly keeping it in history.  
        async fn dismiss\_notification(\&mut self, notification\_id: Uuid) \-\> Result\<(), NotificationError\>;

        /// Retrieves a list of currently active (not dismissed, potentially unread) notifications.  
        /// Allows filtering and sorting.  
        async fn get\_active\_notifications(  
            \&self,  
            filter: Option\<NotificationFilterCriteria\>,  
            sort\_order: Option\<NotificationSortOrder\>  
        ) \-\> Result\<Vec\<Notification\>, NotificationError\>;

        /// Retrieves the notification history.  
        /// Allows filtering and sorting.  
        async fn get\_notification\_history(  
            \&self,  
            limit: Option\<usize\>,  
            filter: Option\<NotificationFilterCriteria\>,  
            sort\_order: Option\<NotificationSortOrder\>  
        ) \-\> Result\<Vec\<Notification\>, NotificationError\>;

        /// Clears all notifications from history.  
        async fn clear\_history(\&mut self) \-\> Result\<(), NotificationError\>;

        /// Sets the "Do Not Disturb" mode.  
        async fn set\_do\_not\_disturb(\&mut self, enabled: bool) \-\> Result\<(), NotificationError\>;

        /// Checks if "Do Not Disturb" mode is currently enabled.  
        async fn is\_do\_not\_disturb\_enabled(\&self) \-\> Result\<bool, NotificationError\>;

        /// Invokes a specific action associated with a notification.  
        async fn invoke\_action(\&mut self, notification\_id: Uuid, action\_key: \&str) \-\> Result\<(), NotificationError\>;  
    }

* **3.2. Vor- und Nachbedingungen, Beschreibung der Logik/Algorithmen:**  
  * AIInteractionLogicService::provide\_consent:  
    * Vorbedingung: context\_id muss einen existierenden AIInteractionContext referenzieren. model\_id muss einem bekannten AIModelProfile entsprechen.  
    * Logik:  
      1. Kontext und Modellprofil laden.  
      2. Prüfen, ob die granted\_categories eine Untermenge der vom Modell potenziell benötigten Kategorien sind.  
      3. Einen neuen AIConsent-Eintrag erstellen oder einen bestehenden aktualisieren.  
      4. Den consent\_status im AIInteractionContext entsprechend anpassen.  
      5. Falls consent\_decision true ist und die Einwilligung global gespeichert werden soll, store\_consent() aufrufen.  
      6. AIConsentUpdatedEvent auslösen.  
    * Nachbedingung: Der Einwilligungsstatus des Kontexts ist aktualisiert. Ein AIConsent-Objekt wurde potenziell erstellt/modifiziert. Ein Event wurde ausgelöst.  
  * NotificationService::post\_notification:  
    * Vorbedingung: notification\_data.summary darf nicht leer sein.  
    * Logik:  
      1. Validieren der notification\_data.  
      2. Der Notification eine neue Uuid und einen timestamp zuweisen.  
      3. Wenn DND-Modus aktiv ist und die NotificationUrgency nicht Critical ist, die Benachrichtigung ggf. unterdrücken oder nur zur Historie hinzufügen, ohne sie aktiv anzuzeigen.  
      4. Die Benachrichtigung zur Liste der active\_notifications hinzufügen.  
      5. Wenn die Benachrichtigung nicht transient ist, sie zur history hinzufügen (unter Beachtung von MAX\_NOTIFICATION\_HISTORY).  
      6. NotificationPostedEvent auslösen (ggf. mit Information, ob sie aufgrund von DND unterdrückt wurde).  
    * Nachbedingung: Die Benachrichtigung ist im System registriert und ein Event wurde ausgelöst.  
* **3.3. Modulspezifische Trait-Definitionen und relevante Implementierungen:**  
  * AIInteractionLogicService und NotificationService sind die primären Traits.  
  * Implementierende Strukturen (z.B. DefaultAIInteractionLogicService, DefaultNotificationService) werden den Zustand halten (z.B. in HashMaps oder Vecs) und die Logik implementieren. Diese Strukturen sind nicht Teil der öffentlichen API, sondern interne Implementierungsdetails des Moduls.  
* **3.4. Exakte Definition aller Methoden für Komponenten mit komplexem internen Zustand oder Lebenszyklus:**  
  * DefaultAIInteractionLogicService:  
    * Hält intern Zustände wie active\_contexts: HashMap\<Uuid, AIInteractionContext\>, consents: Vec\<AIConsent\> (oder eine persistentere Speicherung über core::config), model\_profiles: Vec\<AIModelProfile\>.  
    * Die Methode load\_model\_profiles wäre typischerweise beim Start des Service aufgerufen, um die Profile aus einer Konfigurationsquelle zu laden.  
    * Die Methode store\_consent würde mit der Kernschicht interagieren, um Einwilligungen persistent zu machen.  
  * DefaultNotificationService:  
    * Hält intern Zustände wie active\_notifications: Vec\<Notification\>, history: VecDeque\<Notification\> (eine VecDeque ist hier passend für eine FIFO-artige Historie mit Limit), dnd\_enabled: bool, subscribers: Vec\<Weak\<dyn NotificationEventSubscriber\>\> (für den Event-Mechanismus, falls nicht über einen globalen Event-Bus gelöst).  
    * Methoden wie post\_notification und dismiss\_notification modifizieren diese Listen und müssen die Logik für die Historienbegrenzung und DND-Modus berücksichtigen.

**4\. Event-Spezifikationen für domain::user\_centric\_services**  
Events signalisieren Zustandsänderungen oder wichtige Ereignisse innerhalb des Moduls, die für andere Teile des Systems relevant sein können.

* **Event: AIInteractionInitiatedEvent**  
  * Event-Typ (Rust-Typ): pub struct AIInteractionInitiatedEvent { pub context\_id: Uuid, pub relevant\_categories: Vec\<AIDataCategory\> }  
  * Payload-Struktur: Enthält die ID des neuen Kontexts und die initial relevanten Datenkategorien.  
  * Typische Publisher: AIInteractionLogicService Implementierung.  
  * Typische Subscriber: UI-Komponenten, die eine KI-Interaktionsoberfläche öffnen oder vorbereiten; Logging-Systeme.  
  * Auslösebedingungen: Ein neuer AIInteractionContext wurde erfolgreich erstellt via initiate\_interaction.  
* **Event: AIConsentUpdatedEvent**  
  * Event-Typ (Rust-Typ): pub struct AIConsentUpdatedEvent { pub context\_id: Option\<Uuid\>, pub model\_id: String, pub granted\_categories: Vec\<AIDataCategory\>, pub consent\_status: AIConsentStatus }  
  * Payload-Struktur: Enthält die Kontext-ID (falls zutreffend), Modell-ID, die betroffenen Datenkategorien und den neuen Einwilligungsstatus.  
  * Typische Publisher: AIInteractionLogicService Implementierung.  
  * Typische Subscriber: UI-Komponenten, die den Einwilligungsstatus anzeigen oder Aktionen basierend darauf freischalten/sperren; die Komponente, die die eigentliche KI-Anfrage durchführt.  
  * Auslösebedingungen: Eine Einwilligung wurde erteilt, verweigert oder widerrufen (provide\_consent, store\_consent mit Widerruf).  
* **Event: NotificationPostedEvent**  
  * Event-Typ (Rust-Typ): pub struct NotificationPostedEvent { pub notification: Notification, pub suppressed\_by\_dnd: bool }  
  * Payload-Struktur: Enthält die vollständige Notification-Datenstruktur und ein Flag, ob sie aufgrund des DND-Modus unterdrückt wurde.  
  * Typische Publisher: NotificationService Implementierung.  
  * Typische Subscriber: UI-Schicht (zur Anzeige der Benachrichtigung), Systemschicht (z.B. um einen Ton abzuspielen, falls nicht unterdrückt).  
  * Auslösebedingungen: Eine neue Benachrichtigung wurde erfolgreich via post\_notification verarbeitet.  
* **Event: NotificationDismissedEvent**  
  * Event-Typ (Rust-Typ): pub struct NotificationDismissedEvent { pub notification\_id: Uuid }  
  * Payload-Struktur: Enthält die ID der entfernten Benachrichtigung.  
  * Typische Publisher: NotificationService Implementierung.  
  * Typische Subscriber: UI-Schicht (um die Benachrichtigung aus der aktiven Ansicht zu entfernen).  
  * Auslösebedingungen: Eine Benachrichtigung wurde erfolgreich via dismiss\_notification geschlossen.  
* **Event: NotificationReadEvent**  
  * Event-Typ (Rust-Typ): pub struct NotificationReadEvent { pub notification\_id: Uuid }  
  * Payload-Struktur: Enthält die ID der als gelesen markierten Benachrichtigung.  
  * Typische Publisher: NotificationService Implementierung.  
  * Typische Subscriber: UI-Schicht (um den "gelesen"-Status zu aktualisieren).  
  * Auslösebedingungen: Eine Benachrichtigung wurde erfolgreich via mark\_as\_read als gelesen markiert.  
* **Event: DoNotDisturbModeChangedEvent**  
  * Event-Typ (Rust-Typ): pub struct DoNotDisturbModeChangedEvent { pub dnd\_enabled: bool }  
  * Payload-Struktur: Enthält den neuen Status des DND-Modus.  
  * Typische Publisher: NotificationService Implementierung.  
  * Typische Subscriber: UI-Schicht (um ein Icon anzuzeigen), NotificationService selbst (um zukünftige Benachrichtigungen entsprechend zu behandeln).  
  * Auslösebedingungen: Der DND-Modus wurde via set\_do\_not\_disturb geändert.  
* **Wichtige Tabelle: Event-Spezifikationen für domain::user\_centric\_services**

| Event-Name/Typ (Rust) | Payload-Struktur (Felder, Typen) | Typische Publisher | Typische Subscriber | Auslösebedingungen |
| :---- | :---- | :---- | :---- | :---- |
| AIInteractionInitiatedEvent | context\_id: Uuid, relevant\_categories: Vec\<AIDataCategory\> | AIInteractionLogicService | UI für KI-Interaktion, Logging | Neuer AIInteractionContext erstellt. |
| AIConsentUpdatedEvent | context\_id: Option\<Uuid\>, model\_id: String, granted\_categories: Vec\<AIDataCategory\>, consent\_status: AIConsentStatus | AIInteractionLogicService | UI für Einwilligungsstatus, KI-Anfragekomponente | Einwilligung geändert (erteilt, verweigert, widerrufen). |
| NotificationPostedEvent | notification: Notification, suppressed\_by\_dnd: bool | NotificationService | UI zur Benachrichtigungsanzeige, System-Sound-Service | Neue Benachrichtigung verarbeitet. |
| NotificationDismissedEvent | notification\_id: Uuid | NotificationService | UI zur Benachrichtigungsanzeige | Benachrichtigung geschlossen. |
| NotificationReadEvent | notification\_id: Uuid | NotificationService | UI zur Benachrichtigungsanzeige | Benachrichtigung als gelesen markiert. |
| DoNotDisturbModeChangedEvent | dnd\_enabled: bool | NotificationService | UI (DND-Statusanzeige), NotificationService | DND-Modus geändert. |

Diese Event-Definitionen sind fundamental, um eine lose Kopplung zwischen diesem Domänenmodul und anderen Teilen des Systems, insbesondere der UI-Schicht, zu erreichen. Die UI kann auf diese Events reagieren, um sich dynamisch an Zustandsänderungen anzupassen, ohne die Interna dieses Moduls kennen zu müssen.  
**5\. Fehlerbehandlung (Rust mit thiserror) für domain::user\_centric\_services**  
Gemäß den Entwicklungsrichtlinien (Abschnitt 4.3) wird thiserror zur Definition spezifischer Fehler-Enums pro Sub-Modul verwendet. Dies ermöglicht eine klare und kontextbezogene Fehlerbehandlung.1

* **Definition der modulspezifischen Error-Enums:**  
  * AIInteractionError  
  * NotificationError  
* **Detaillierte Varianten, Nutzung von \#\[error(...)\] und \#\[from\]:**  
  * **AIInteractionError:**  
    Rust  
    use thiserror::Error;  
    use crate::core::types::Uuid; // Standard Uuid Typ aus der Kernschicht

    \#  
    pub enum AIInteractionError {  
        \#  
        ContextNotFound(Uuid),

        \#  
        ConsentAlreadyProvided(Uuid), // Spezifischer Fall, wenn ein erneutes explizites provide\_consent für bereits erteilte Zustimmung erfolgt

        \#\[error("Consent required for model '{model\_id}' but not granted for data categories: {missing\_categories:?}")\]  
        ConsentRequired { model\_id: String, missing\_categories: Vec\<String\> }, // String für AIDataCategory hier vereinfacht

        \#\[error("No suitable AI model available or configured.")\]  
        NoModelAvailable,

        \#\[error("AI Model '{model\_id}' not found or not configured.")\]  
        ModelNotFound(String),

        \#\[error("Invalid attachment data provided: {0}")\]  
        InvalidAttachment(String), // z.B. ungültiger Pfad, nicht unterstützter MIME-Typ

        \#\[error("Failed to store or retrieve consent: {0}")\]  
        ConsentStorageError(String), // Generisch für Fehler beim Speichern/Laden von AIConsent

        \#\[error("Failed to load AI model profiles: {0}")\]  
        ModelProfileLoadError(String),

        \#\[error("An underlying core error occurred: {source}")\]  
        CoreError { \#\[from\] source: crate::core::errors::CoreError }, // Annahme: Es gibt einen CoreError in der Kernschicht

        \#\[error("An unexpected internal error occurred: {0}")\]  
        InternalError(String),  
    }

  * **NotificationError:**  
    Rust  
    use thiserror::Error;  
    use crate::core::types::Uuid;

    \#  
    pub enum NotificationError {  
        \#  
        NotFound(Uuid),

        \# // z.B. leerer Summary  
        InvalidData{ summary: String, details: String },

        \#\[error("Maximum notification history of {max\_history} reached. Cannot add new notification: {summary}")\]  
        HistoryFull { max\_history: usize, summary: String },

        \#  
        ActionNotFound { notification\_id: Uuid, action\_id: String },

        \#\[error("An underlying core error occurred: {source}")\]  
        CoreError { \#\[from\] source: crate::core::errors::CoreError },

        \#\[error("An unexpected internal error occurred: {0}")\]  
        InternalError(String),  
    }

* **Spezifikation der Verwendung:**  
  * Diese Fehler werden als Err-Variante in Result\<T, E\>-Typen der öffentlichen API-Methoden der jeweiligen Services zurückgegeben.2  
  * Die \#\[from\]-Direktive wird genutzt, um Fehler aus der Kernschicht (z.B. CoreError beim Speichern/Laden von Konfigurationen für Einwilligungen oder Modellprofile) transparent in AIInteractionError oder NotificationError umzuwandeln. Dies erleichtert die Fehlerweitergabe (?-Operator) und erhält gleichzeitig die Fehlerquelle über die source()-Methode des std::error::Error-Traits.3  
  * Die \#\[error("...")\]-Nachrichten sind prägnant formuliert, um den Fehlerzustand klar zu beschreiben, wie in den Rust API Guidelines und 3 empfohlen (kleingeschrieben, ohne abschließende Interpunktion).  
  * Die Definition spezifischer Fehler-Enums pro logischem Service (AIInteractionError, NotificationError) folgt der Projektrichtlinie (4.3) und der Empfehlung aus 1, um Klarheit in der Fehlerbehandlung zu schaffen und es dem aufrufenden Code zu ermöglichen, spezifisch auf Fehlerfälle zu reagieren.  
  * Ein wichtiger Aspekt, der bei der Verwendung von thiserror mit \#\[from\] zu beachten ist, wurde in 2 hervorgehoben: Wenn mehrere Operationen innerhalb eines Services potenziell denselben *Basistyp* eines Fehlers aus einer unteren Schicht (z.B. std::io::Error, gekapselt in CoreError) für *unterschiedliche logische Fehlerfälle* im aktuellen Service erzeugen könnten, kann die alleinige Verwendung von \#\[from\] für eine generische CoreError-Variante den spezifischen Kontext verwischen.  
    * Beispiel: Sowohl das Speichern einer AIConsent als auch das Laden von AIModelProfile könnten intern eine CoreError::IoError verursachen. Wenn AIInteractionError nur CoreError { \#\[from\] source: CoreError } hätte, wäre aus dem Fehlertyp allein nicht ersichtlich, welche der beiden Operationen fehlgeschlagen ist.  
    * **Lösung und Spezifikation:** Für solche Fälle werden spezifischere Fehlervarianten ohne \#\[from\] für CoreError definiert, die stattdessen die CoreError (oder die relevante Information daraus) als Feld halten. Die \#\[error("...")\]-Nachricht dieser spezifischen Variante muss dann den Kontext klarstellen.  
      * Im obigen AIInteractionError sind ConsentStorageError(String) und ModelProfileLoadError(String) Beispiele dafür. Sie würden manuell in der Service-Logik konstruiert, z.B. indem ein von core::config zurückgegebener CoreError abgefangen und in diese spezifischeren Varianten umgewandelt wird, wobei die String-Payload die Details des Fehlers enthält.  
      * Die generische AIInteractionError::CoreError { \#\[from\] source: CoreError } Variante dient dann als Catch-All für andere, nicht spezifisch behandelte CoreError-Fälle aus diesem Service. Dies stellt sicher, dass der semantische Kontext des Domänenfehlers erhalten bleibt, während die Fehlerquelle (source()) weiterhin zugänglich ist, was für Debugging und Fehleranalyse von großer Bedeutung ist.2  
* **Wichtige Tabelle: Fehler-Enums für domain::user\_centric\_services**

| Fehler-Enum | Variante | \#\[error(...)\] Nachricht (Beispiel) | Felder (Typen) | Beschreibung / Auslösekontext |
| :---- | :---- | :---- | :---- | :---- |
| AIInteractionError | ContextNotFound | "AI interaction context not found for ID: {0}" | Uuid | Eine angeforderte AIInteractionContext ID existiert nicht. |
|  | ConsentRequired | "Consent required for model '{model\_id}' but not granted for data categories: {missing\_categories:?}" | model\_id: String, missing\_categories: Vec\<String\> | Für die geplante Aktion/Modell fehlt die notwendige Einwilligung. |
|  | ModelNotFound | "AI Model '{0}' not found or not configured." | String | Ein spezifisches KI-Modell wurde nicht gefunden oder ist nicht konfiguriert. |
|  | ConsentStorageError | "Failed to store or retrieve consent: {0}" | String | Fehler beim persistenten Speichern oder Laden einer AIConsent. |
|  | ModelProfileLoadError | "Failed to load AI model profiles: {0}" | String | Fehler beim Laden der AIModelProfile Konfigurationen. |
|  | CoreError | "An underlying core error occurred: {source}" | \#\[from\] source: crate::core::errors::CoreError | Ein nicht spezifisch behandelter Fehler aus der Kernschicht ist aufgetreten und wurde weitergeleitet. |
| NotificationError | NotFound | "Notification not found for ID: {0}" | Uuid | Eine angeforderte Benachrichtigungs-ID existiert nicht. |
|  | InvalidData | "Invalid notification data: {summary} (Details: {details})" | summary: String, details: String | Die übergebenen Daten zur Erstellung einer Benachrichtigung sind ungültig (z.B. leerer Summary). |
|  | HistoryFull | "Maximum notification history of {max\_history} reached. Cannot add new notification: {summary}" | max\_history: usize, summary: String | Das konfigurierte Benachrichtigungslimit in der Historie wurde erreicht. |
|  | ActionNotFound | "Action '{action\_id}' not found for notification ID: {notification\_id}" | notification\_id: Uuid, action\_id: String | Eine angeforderte Aktion für eine Benachrichtigung existiert nicht. |
|  | CoreError | "An underlying core error occurred: {source}" | \#\[from\] source: crate::core::errors::CoreError | Ein nicht spezifisch behandelter Fehler aus der Kernschicht ist aufgetreten und wurde weitergeleitet. |

Diese strukturierte Fehlerbehandlung ist für die Entwicklung robuster Software unerlässlich. Sie ermöglicht nicht nur eine präzise Fehlerdiagnose während der Entwicklung, sondern auch die Implementierung einer differenzierten Fehlerbehandlung im aufrufenden Code, bis hin zur Anzeige benutzerfreundlicher Fehlermeldungen in der UI.  
**6\. Detaillierte Implementierungsschritte und Dateistruktur für domain::user\_centric\_services**

* **6.1. Vorgeschlagene Dateistruktur:**  
  src/domain/user\_centric\_services/  
  ├── mod.rs               // Deklariert Submodule, exportiert öffentliche Typen/Traits  
  ├── ai\_interaction\_service.rs // Implementierung von AIInteractionLogicService (z.B. DefaultAIInteractionLogicService)  
  ├── notification\_service.rs   // Implementierung von NotificationService (z.B. DefaultNotificationService)  
  ├── types.rs             // Gemeinsame Enums (AIConsentStatus, AIDataCategory etc.) und Wertobjekte, Entitätsdefinitionen  
  └── errors.rs            // Definition von AIInteractionError und NotificationError

* **6.2. Nummerierte, schrittweise Anleitung zur Implementierung:**  
  1. **errors.rs erstellen:** Definieren Sie die AIInteractionError und NotificationError Enums mithilfe von thiserror wie im vorherigen Abschnitt spezifiziert. Stellen Sie sicher, dass sie Debug, Clone, PartialEq, Eq (falls für Testzwecke oder spezifische Logik benötigt) implementieren.  
  2. **types.rs erstellen:**  
     * Definieren Sie alle modulspezifischen Enums: AIConsentStatus, AIDataCategory, NotificationUrgency, NotificationActionType, NotificationFilterCriteria, NotificationSortOrder.  
     * Definieren Sie die Wertobjekte: NotificationAction, AttachmentData.  
     * Definieren Sie die Entitätsstrukturen: AIInteractionContext, AIConsent, AIModelProfile, Notification. Implementieren Sie für diese Debug, Clone, PartialEq und ggf. Serialize/Deserialize (von serde), falls sie direkt persistiert oder über IPC-Grenzen gesendet werden sollen. Fügen Sie Konstruktor-Methoden (new()) und andere relevante Logik direkt zu diesen Strukturen hinzu.  
  3. **ai\_interaction\_service.rs Basis:**  
     * Definieren Sie den Trait AIInteractionLogicService (wie in Abschnitt 3.1).  
     * Erstellen Sie eine Struktur DefaultAIInteractionLogicService. Diese Struktur wird Felder für den internen Zustand enthalten, z.B. active\_contexts: std::collections::HashMap\<Uuid, AIInteractionContext\>, consents: Vec\<AIConsent\> (oder eine Abstraktion für die Persistenz), model\_profiles: Vec\<AIModelProfile\>. Sie benötigt möglicherweise eine Abhängigkeit zu einer Komponente der Kernschicht für Persistenz.  
     * Beginnen Sie mit der Implementierung von \#\[async\_trait\] impl AIInteractionLogicService for DefaultAIInteractionLogicService {... }.  
  4. **notification\_service.rs Basis:**  
     * Definieren Sie den Trait NotificationService (wie in Abschnitt 3.1).  
     * Erstellen Sie eine Struktur DefaultNotificationService. Diese Struktur wird Felder für den internen Zustand enthalten, z.B. active\_notifications: Vec\<Notification\>, history: std::collections::VecDeque\<Notification\>, dnd\_enabled: bool.  
     * Beginnen Sie mit der Implementierung von \#\[async\_trait\] impl NotificationService for DefaultNotificationService {... }.  
  5. **Implementierung der AIInteractionLogicService-Methoden in DefaultAIInteractionLogicService:**  
     * Implementieren Sie jede Methode des Traits schrittweise. Achten Sie auf die korrekte Fehlerbehandlung und Rückgabe der definierten AIInteractionError-Varianten.  
     * Für Methoden, die Persistenz erfordern (z.B. store\_consent, load\_model\_profiles), definieren Sie die Interaktion mit der (noch abstrakten) Kernschichtkomponente.  
     * Stellen Sie sicher, dass die entsprechenden Events (z.B. AIInteractionInitiatedEvent, AIConsentUpdatedEvent) an den dafür vorgesehenen Stellen ausgelöst werden. Der genaue Mechanismus zur Event-Veröffentlichung (z.B. ein globaler Event-Bus, direkte Callbacks) muss projektweit definiert sein; hier wird nur das logische Auslösen spezifiziert.  
  6. **Implementierung der NotificationService-Methoden in DefaultNotificationService:**  
     * Implementieren Sie jede Methode des Traits. Achten Sie auf die Logik für DND, Historienbegrenzung (MAX\_NOTIFICATION\_HISTORY), Filterung und Sortierung.  
     * Verwenden Sie NotificationError-Varianten für Fehlerfälle.  
     * Lösen Sie die spezifizierten Notification-Events aus.  
  7. **mod.rs erstellen:**  
     * Deklarieren Sie die Submodule: pub mod errors;, pub mod types;, pub mod ai\_interaction\_service;, pub mod notification\_service;.  
     * Exportieren Sie die öffentlichen Typen, Traits und Fehler-Enums, die von außerhalb dieses Moduls verwendet werden sollen:  
       Rust  
       pub use errors::{AIInteractionError, NotificationError};  
       pub use types::{  
           AIInteractionContext, AIConsent, AIModelProfile, Notification, NotificationAction, AttachmentData,  
           AIConsentStatus, AIDataCategory, NotificationUrgency, NotificationActionType,  
           NotificationFilterCriteria, NotificationSortOrder  
       };  
       pub use ai\_interaction\_service::AIInteractionLogicService;  
       pub use notification\_service::NotificationService;

       // Optional: Konkrete Service-Typen exportieren, wenn sie direkt instanziiert werden sollen  
       // pub use ai\_interaction\_service::DefaultAIInteractionLogicService;  
       // pub use notification\_service::DefaultNotificationService;

  8. **Unit-Tests:** Schreiben Sie parallel zur Implementierung jeder Methode und jeder komplexen Logikeinheit Unit-Tests in den jeweiligen Service-Dateien (z.B. in einem \#\[cfg(test)\] mod tests {... } Block).

**7\. Interaktionen und Abhängigkeiten (domain::user\_centric\_services)**

* **Nutzung von Funktionalitäten der Kernschicht:**  
  * core::types: Verwendung von Uuid für eindeutige Identifikatoren und chrono::DateTime\<Utc\> für Zeitstempel.  
  * core::errors: Die CoreError-Typen der Kernschicht werden über \#\[from\] in die modulspezifischen Fehler AIInteractionError und NotificationError überführt, um Fehlerursachen aus der Kernschicht weiterzuleiten.  
  * core::config: Für das Laden von AIModelProfile-Konfigurationen und das persistente Speichern/Laden von AIConsent-Daten. Die Services in diesem Domänenmodul delegieren die eigentlichen Lese-/Schreiboperationen an die Kernschicht.  
  * core::logging: Das tracing-Framework wird innerhalb der Service-Implementierungen für strukturiertes Logging verwendet, um den Ablauf und mögliche Fehler nachvollziehen zu können.  
* **Schnittstellen zu System- und UI-Schicht:**  
  * Die definierten Traits AIInteractionLogicService und NotificationService stellen die primären Schnittstellen für höhere Schichten dar.  
  * Die **Systemschicht** wird diese Services nutzen:  
    * Der MCP-Client (in system::mcp) wird mit dem AIInteractionLogicService interagieren, um Einwilligungen zu prüfen und Interaktionskontexte zu verwalten.  
    * D-Bus Handler (in system::dbus), die z.B. den org.freedesktop.Notifications-Standard implementieren, werden den NotificationService verwenden, um Benachrichtigungen zu empfangen und Aktionen weiterzuleiten.  
  * Die **Benutzeroberflächenschicht (UI Layer)** wird ebenfalls mit diesen Services interagieren:  
    * UI-Komponenten für KI-Interaktionen (z.B. eine Befehlspalette oder ein Chat-Fenster) rufen Methoden des AIInteractionLogicService auf.  
    * Das ui::control\_center könnte Einstellungen für KI-Modelle oder Einwilligungen über den AIInteractionLogicService verwalten.  
    * Die Benachrichtigungsanzeige (ui::notifications) abonniert Events wie NotificationPostedEvent und ruft Methoden wie get\_active\_notifications oder mark\_as\_read des NotificationService auf.  
  * Events, die in diesem Domänenmodul ausgelöst werden (z.B. NotificationPostedEvent, AIConsentUpdatedEvent), werden primär von der UI-Schicht abonniert, um die Benutzeroberfläche entsprechend zu aktualisieren.  
* **Interaktionen mit anderen Modulen der Domänenschicht:**  
  * domain::global\_settings\_and\_state\_management: Globale Einstellungen, die das Verhalten der KI oder der Benachrichtigungen beeinflussen (z.B. Standard-KI-Modell, globale Einwilligungs-Standardeinstellungen, Standard-DND-Verhalten, maximale Historienlänge für Benachrichtigungen), könnten aus dem GlobalSettingsService gelesen werden. Änderungen an diesen Einstellungen könnten wiederum das Verhalten der Services in diesem Modul beeinflussen.  
  * domain::workspaces: Der AIInteractionContext könnte Informationen über den aktuellen Workspace (z.B. aktive Anwendung, Fenstertitel) enthalten, um den KI-Modellen besseren Kontext zu liefern. Diese Informationen würden vom AIInteractionLogicService aus dem domain::workspaces Modul bezogen.

**8\. Testaspekte für Unit-Tests (domain::user\_centric\_services)**  
Umfassende Unit-Tests sind entscheidend, um die Korrektheit der komplexen Logik in diesem Modul sicherzustellen.

* **Identifikation testkritischer Logik:**  
  * **AIInteractionLogicService:**  
    * Korrekte Erstellung, Aktualisierung und Abruf von AIInteractionContext.  
    * Logik der Einwilligungsprüfung (get\_consent\_for\_model), insbesondere die korrekte Auswertung von required\_consent\_categories der AIModelProfile gegen angefragte und erteilte AIDataCategory.  
    * Korrekte Erstellung und Speicherung (Mock) von AIConsent-Objekten.  
    * Laden und Filtern von AIModelProfile.  
    * Fehlerbehandlung für alle definierten AIInteractionError-Fälle.  
    * Korrekte Auslösung von Events.  
  * **NotificationService:**  
    * Korrekte Erstellung von Notification-Objekten und Zuweisung von IDs/Timestamps.  
    * Verwaltung der active\_notifications-Liste und der history-Deque, insbesondere die Einhaltung von MAX\_NOTIFICATION\_HISTORY.  
    * Logik des DND-Modus (Unterdrückung von Benachrichtigungen, Ausnahmen für Critical).  
    * Filter- und Sortierlogik für get\_active\_notifications und get\_notification\_history.  
    * Zustandsübergänge von Benachrichtigungen (is\_read, is\_dismissed).  
    * Korrekte Auslösung von Events.  
    * Fehlerbehandlung für alle definierten NotificationError-Fälle.  
* **Beispiele für Testfälle:**  
  * **AIInteractionLogicService Tests:**  
    * test\_initiate\_interaction\_creates\_context\_with\_unique\_id\_and\_fires\_event  
    * test\_provide\_consent\_granted\_updates\_context\_status\_and\_stores\_consent\_fires\_event  
    * test\_provide\_consent\_denied\_updates\_context\_status\_fires\_event  
    * test\_get\_consent\_for\_model\_no\_consent\_needed\_returns\_not\_required  
    * test\_get\_consent\_for\_model\_consent\_pending\_returns\_pending  
    * test\_get\_consent\_for\_model\_consent\_granted\_returns\_granted  
    * test\_get\_consent\_for\_model\_missing\_categories\_returns\_pending\_or\_error  
    * test\_list\_available\_models\_returns\_correctly\_loaded\_profiles  
    * test\_add\_attachment\_to\_context\_succeeds  
    * test\_get\_interaction\_context\_not\_found\_returns\_error  
    * test\_load\_model\_profiles\_error\_from\_core\_propagates\_as\_model\_profile\_load\_error  
  * **NotificationService Tests:**  
    * test\_post\_notification\_adds\_to\_active\_and\_history\_fires\_event  
    * test\_post\_notification\_when\_history\_full\_evicts\_oldest  
    * test\_post\_notification\_transient\_not\_added\_to\_history  
    * test\_post\_notification\_dnd\_active\_normal\_urgency\_suppressed\_fires\_event\_with\_suppressed\_flag  
    * test\_post\_notification\_dnd\_active\_critical\_urgency\_not\_suppressed  
    * test\_dismiss\_notification\_removes\_from\_active\_sets\_flag\_fires\_event  
    * test\_mark\_as\_read\_sets\_flag\_fires\_event  
    * test\_get\_active\_notifications\_filters\_unread\_correctly  
    * test\_get\_notification\_history\_sorted\_by\_timestamp\_descending  
    * test\_clear\_history\_empties\_history\_list  
    * test\_set\_do\_not\_disturb\_updates\_state\_and\_fires\_event  
    * test\_invoke\_action\_unknown\_notification\_id\_returns\_not\_found\_error  
    * test\_invoke\_action\_unknown\_action\_key\_returns\_action\_not\_found\_error  
* **Mocking:**  
  * Für Tests, die von der Kernschicht abhängen (z.B. core::config für das Laden/Speichern von AIConsent oder AIModelProfile), müssen Mocks dieser Kernschichtkomponenten erstellt werden. Dies kann durch Definition von Traits in der Kernschicht geschehen, die dann im Test durch Mock-Implementierungen ersetzt werden (z.B. mit dem mockall-Crate).  
  * Der Event-Mechanismus sollte ebenfalls mockbar sein, um zu überprüfen, ob Events korrekt ausgelöst werden.

---

**Entwicklungsmodul D: domain::global\_settings\_and\_state\_management**  
Dieses Modul ist für die Repräsentation, die Logik zur Verwaltung und die Konsistenz des globalen Zustands und der Einstellungen der Desktop-Umgebung zuständig, die nicht spezifisch einem anderen Domänenmodul zugeordnet sind oder von mehreren Modulen gemeinsam genutzt werden. Es fungiert als zentrale Anlaufstelle innerhalb der Domänenschicht für den Zugriff auf Konfigurationen und deren Modifikation.  
**1\. Modulübersicht und Verantwortlichkeiten (domain::global\_settings\_and\_state\_management)**

* **Zweck:** Bereitstellung einer kohärenten, typsicheren und validierten Abstraktion über die vielfältigen globalen Einstellungen und Zustände der Desktop-Umgebung. Dieses Modul definiert die "Quelle der Wahrheit" für diese Einstellungen innerhalb der Domänenschicht und stellt sicher, dass Änderungen konsistent angewendet und kommuniziert werden.  
* **Kernaufgaben:**  
  * Definition einer oder mehrerer umfassender Datenstrukturen (z.B. GlobalDesktopSettings), die alle globalen Desktop-Einstellungen kategorisiert repräsentieren (z.B. Erscheinungsbild, Verhalten, Eingabeoptionen, Energieverwaltungsrichtlinien, Standardanwendungen).  
  * Bereitstellung von Logik zur Validierung von Einstellungsänderungen anhand vordefinierter Regeln (z.B. Wertebereiche, gültige Optionen).  
  * Verwaltung des Lebenszyklus dieser Einstellungen: Laden von Standardwerten, Initialisierung aus persistenten Speichern (Delegation an die Kernschicht) und Persistierung von Änderungen.  
  * Benachrichtigung anderer Systemteile (innerhalb der Domänenschicht sowie höhere Schichten) über erfolgte Einstellungsänderungen mittels eines Event-Mechanismus.  
  * Verwaltung von globalen, nicht-persistenten Zuständen, die für die Dauer einer Benutzersitzung relevant sind und nicht direkt durch Systemdienste wie logind abgedeckt werden (z.B. ein anwendungsdefinierter "Desktop gesperrt"-Zustand, falls komplexere Logik als reine Sitzungssperrung benötigt wird).  
* **Abgrenzung:**  
  * Dieses Modul implementiert **nicht** die grafische Benutzeroberfläche zur Darstellung oder Änderung der Einstellungen. Diese Aufgabe obliegt der Komponente ui::control\_center in der Benutzeroberflächenschicht.  
  * Es implementiert **nicht** die tatsächliche Speicherung und das Laden von Konfigurationsdateien vom Dateisystem. Diese Low-Level-Operationen werden an eine Komponente der Kernschicht (z.B. core::config) delegiert. Das domain::global\_settings\_and\_state\_management-Modul definiert *was* gespeichert wird, die Struktur der Daten und die Regeln für deren Gültigkeit.  
  * Es verwaltet **keine** anwendungsspezifischen Einstellungen einzelner Drittanwendungen. Der Fokus liegt auf den globalen Einstellungen der Desktop-Umgebung selbst.  
* **Zugehörige Komponenten aus der Gesamtübersicht:** domain::settings.

**2\. Datenstrukturen und Typdefinitionen (Rust) für domain::global\_settings\_and\_state\_management**  
Die Datenstrukturen sind darauf ausgelegt, eine breite Palette von Einstellungen hierarchisch und typsicher abzubilden. Alle Einstellungsstrukturen müssen serde::Serialize und serde::Deserialize implementieren, um die Interaktion mit der Persistenzschicht (core::config) und die Verarbeitung von Einstellungsänderungen über serde\_json::Value zu ermöglichen.

* **2.1. Entitäten und Wertobjekte (primär Konfigurationsstrukturen):**  
  * **GlobalDesktopSettings (Hauptstruktur):**  
    Rust  
    use serde::{Serialize, Deserialize};  
    // Annahme: Pfade zu untergeordneten Typen sind korrekt  
    // use super::types::{AppearanceSettings, WorkspaceSettings,...};

    \#  
    pub struct GlobalDesktopSettings {  
        \#\[serde(default)\]  
        pub appearance: AppearanceSettings,  
        \#\[serde(default)\]  
        pub workspace\_config: WorkspaceSettings, // Umbenannt von workspace\_settings zur Klarheit (Konfiguration vs. Laufzeit)  
        \#\[serde(default)\]  
        pub input\_behavior: InputBehaviorSettings,  
        \#\[serde(default)\]  
        pub power\_management\_policy: PowerManagementPolicySettings,  
        \#\[serde(default)\]  
        pub default\_applications: DefaultApplicationsSettings,  
        // Weitere Kategorien können hier hinzugefügt werden, z.B.:  
        // \#\[serde(default)\]  
        // pub accessibility: AccessibilitySettings,  
        // \#\[serde(default)\]  
        // pub privacy: PrivacySettings,  
    }

    Die Verwendung von \#\[serde(default)\] stellt sicher, dass beim Deserialisieren einer unvollständigen Konfiguration die Standardwerte für fehlende Felder verwendet werden, was die Robustheit gegenüber Konfigurationsänderungen über Versionen hinweg erhöht.  
  * **AppearanceSettings:**  
    * Attribute:  
      * active\_theme\_name: String (z.B. "Adwaita-dark", "Nordic")  
      * color\_scheme: ColorScheme (Enum: Light, Dark, AutoSystem)  
      * accent\_color\_token: String (CSS-Token-Name, z.B. "--accent-blue", "--accent-custom-hexFFA07A")  
      * font\_settings: FontSettings  
      * icon\_theme\_name: String (z.B. "Papirus", "Numix")  
      * cursor\_theme\_name: String (z.B. "Adwaita", "Bibata-Modern-Ice")  
      * enable\_animations: bool  
      * interface\_scaling\_factor: f64 (z.B. 1.0, 1.25, 2.0; Validierung: \> 0.0)  
    * Methoden (konzeptionell): validate() prüft die Gültigkeit der Werte (z.B. Skalierungsfaktor \> 0).  
  * **WorkspaceSettings (Domänenlogik für Einstellungen, nicht der Workspace-Manager selbst):**  
    * Attribute:  
      * dynamic\_workspaces: bool (Workspaces werden bei Bedarf erstellt/entfernt)  
      * default\_workspace\_count: u8 (Nur relevant, wenn dynamic\_workspaces false ist; Validierung: \> 0\)  
      * workspace\_switching\_behavior: WorkspaceSwitchingBehavior (Enum: WrapAround, StopAtEdges)  
      * show\_workspace\_indicator: bool (Ob ein Indikator (z.B. im Panel) angezeigt wird)  
  * **FontSettings:**  
    * Attribute:  
      * default\_font\_family: String (z.B. "Noto Sans", "Cantarell")  
      * default\_font\_size: u8 (in Punkten, z.B. 10, 11; Validierung: z.B. 6-72)  
      * monospace\_font\_family: String (z.B. "Fira Code", "DejaVu Sans Mono")  
      * document\_font\_family: String (z.B. "Liberation Serif")  
      * hinting: FontHinting (Enum: None, Slight, Medium, Full)  
      * antialiasing: FontAntialiasing (Enum: None, Grayscale, Rgba)  
  * **InputBehaviorSettings:**  
    * Attribute:  
      * mouse\_acceleration\_profile: MouseAccelerationProfile (Enum: Flat, Adaptive, Custom(f32))  
      * mouse\_sensitivity: f32 (Validierung: z.B. 0.1 \- 10.0)  
      * natural\_scrolling\_mouse: bool  
      * natural\_scrolling\_touchpad: bool  
      * tap\_to\_click\_touchpad: bool  
      * touchpad\_pointer\_speed: f32 (Validierung: z.B. 0.1 \- 10.0)  
      * keyboard\_repeat\_delay\_ms: u32 (Validierung: z.B. 100-2000)  
      * keyboard\_repeat\_rate\_cps: u32 (Zeichen pro Sekunde; Validierung: z.B. 10-100)  
  * **PowerManagementPolicySettings (High-Level Richtlinien, die systemnahe Implementierung erfolgt in der Systemschicht):**  
    * Attribute:  
      * screen\_blank\_timeout\_ac\_secs: u32 (0 für nie; Validierung: z.B. 0 oder \>= 60\)  
      * screen\_blank\_timeout\_battery\_secs: u32 (0 für nie; Validierung: z.B. 0 oder \>= 30\)  
      * suspend\_action\_on\_lid\_close\_ac: LidCloseAction (Enum: Suspend, Hibernate, Shutdown, DoNothing, LockScreen)  
      * suspend\_action\_on\_lid\_close\_battery: LidCloseAction  
      * automatic\_suspend\_delay\_ac\_secs: u32 (0 für nie)  
      * automatic\_suspend\_delay\_battery\_secs: u32 (0 für nie)  
      * show\_battery\_percentage: bool  
  * **DefaultApplicationsSettings:**  
    * Attribute:  
      * web\_browser\_desktop\_file: String (Name der.desktop-Datei, z.B. "firefox.desktop")  
      * email\_client\_desktop\_file: String (z.B. "thunderbird.desktop")  
      * terminal\_emulator\_desktop\_file: String (z.B. "org.gnome.Console.desktop")  
      * file\_manager\_desktop\_file: String (z.B. "org.gnome.Nautilus.desktop")  
      * music\_player\_desktop\_file: String  
      * video\_player\_desktop\_file: String  
      * image\_viewer\_desktop\_file: String  
      * text\_editor\_desktop\_file: String  
* **2.2. Modulspezifische Enums, Konstanten und Konfigurationsstrukturen:**  
  * **Enums (alle mit Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default):**  
    * ColorScheme: \#\[default\] Light, Dark, AutoSystem.  
    * FontHinting: None, Slight, \#\[default\] Medium, Full.  
    * FontAntialiasing: None, Grayscale, \#\[default\] Rgba.  
    * MouseAccelerationProfile: \#\[default\] Adaptive, Flat, Custom(SerdeF32) (Wrapper für f32 für Default).  
    * LidCloseAction: \#\[default\] Suspend, Hibernate, Shutdown, LockScreen, DoNothing.  
    * WorkspaceSwitchingBehavior: \#\[default\] WrapAround, StopAtEdges.  
    * **Hilfsstruktur für f32 Default in Enums (da f32 nicht Eq ist):**  
      Rust  
      \#  
      pub struct SerdeF32(pub f32);  
      impl Default for SerdeF32 { fn default() \-\> Self { SerdeF32(1.0) } } // Beispiel-Default

  * **SettingPath (Strukturierter Enum für typsicheren Zugriff):**  
    Rust  
    \#  
    pub enum SettingPath {  
        Appearance(AppearanceSettingPath),  
        WorkspaceConfig(WorkspaceSettingPath),  
        InputBehavior(InputBehaviorSettingPath),  
        PowerManagementPolicy(PowerManagementPolicySettingPath),  
        DefaultApplications(DefaultApplicationsSettingPath),  
        // Weitere Top-Level Kategorien  
    }

    \#  
    pub enum AppearanceSettingPath {  
        ActiveThemeName, ColorScheme, AccentColorToken,  
        FontSettings(FontSettingPath), // Verschachtelt  
        IconThemeName, CursorThemeName, EnableAnimations, InterfaceScalingFactor,  
    }

    \#  
    pub enum FontSettingPath { // Beispiel für weitere Verschachtelung  
        DefaultFontFamily, DefaultFontSize, MonospaceFontFamily, DocumentFontFamily, Hinting, Antialiasing,  
    }  
    // Ähnliche Enums für WorkspaceSettingPath, InputBehaviorSettingPath etc. definieren.  
    // Diese Struktur ermöglicht eine präzise Adressierung einzelner Einstellungen.  
    // Für die Implementierung von \`get\_setting\` und \`update\_setting\` ist eine  
    // Konvertierung von/zu String-basierten Pfaden (z.B. "appearance.font\_settings.default\_font\_size")  
    // oder eine direkte Verarbeitung dieser Enum-Pfade erforderlich.

    Die SettingPath-Struktur ist entscheidend für die update\_setting-Methode, da sie eine typsichere und explizite Weise bietet, auf spezifische Einstellungen zuzugreifen, anstatt fehleranfällige String-Pfade zu verwenden.  
* **2.3. Definition aller deklarierten Eigenschaften (Properties):**  
  * Für GlobalSettingsService (als Trait implementiert):  
    * current\_settings: GlobalDesktopSettings (logisch): Der aktuelle Satz aller globalen Einstellungen. Der Zugriff erfolgt über Methoden wie get\_current\_settings() oder get\_setting(\&SettingPath). Modifikationen erfolgen über update\_setting(...).  
* **Wichtige Tabelle: Ausgewählte globale Einstellungen und ihre Eigenschaften**

| Struktur/Kategorie | Attribut/Einstellung | Rust-Typ | Standardwert (Beispiel) | Beschreibung / Gültigkeitsbereich / Validierungsregeln (Beispiele) |
| :---- | :---- | :---- | :---- | :---- |
| AppearanceSettings | active\_theme\_name | String | "default\_light\_theme" | Name des aktuell aktiven GTK-Themes. Muss ein installierter Theme-Name sein. |
|  | color\_scheme | ColorScheme | AutoSystem | Bevorzugtes Farbschema (Hell, Dunkel, Systemeinstellung folgen). |
|  | accent\_color\_token | String | "--accent-blue" | CSS-Token-Name der Akzentfarbe (z.B. "--accent-color-1"). |
|  | enable\_animations | bool | true | Ob Desktop-Animationen (Fenster, Übergänge etc.) aktiviert sind. |
|  | interface\_scaling\_factor | f64 | 1.0 | Globaler Skalierungsfaktor für die UI. Validierung: 0.5 \<= x \<= 3.0. |
| FontSettings | default\_font\_family | String | "Cantarell" | Standard-Schriftart für UI-Elemente. Muss eine installierte Schriftart sein. |
|  | default\_font\_size | u8 | 11 | Standard-Schriftgröße in Punkten. Validierung: 6 \<= size \<= 72\. |
| InputBehaviorSettings | natural\_scrolling\_touchpad | bool | true | Ob natürliches Scrollen (Inhaltsbewegung mit Fingerbewegung) für Touchpads aktiviert ist. |
|  | tap\_to\_click\_touchpad | bool | true | Ob Tippen zum Klicken für Touchpads aktiviert ist. |
|  | keyboard\_repeat\_delay\_ms | u32 | 500 | Verzögerung in ms bis Tastenwiederholung einsetzt. Validierung: 100 \<= delay \<= 2000\. |
| PowerManagementPolicySettings | screen\_blank\_timeout\_ac\_secs | u32 | 600 (10 Min.) | Timeout in Sekunden bis Bildschirmabschaltung im Netzbetrieb. 0 für nie. Validierung: 0 oder 30 \<= secs \<= 7200\. |
|  | suspend\_action\_on\_lid\_close\_battery | LidCloseAction | Suspend | Aktion beim Schließen des Laptop-Deckels im Akkubetrieb. |
| DefaultApplicationsSettings | web\_browser\_desktop\_file | String | "firefox.desktop" | Name der.desktop-Datei des Standard-Webbrowsers. Muss eine gültige, installierte.desktop-Datei sein. |

Diese Tabelle hebt einige der wichtigsten konfigurierbaren Aspekte des Desktops hervor. Die Definition von Standardwerten und Validierungsregeln ist entscheidend für die Robustheit des Systems und eine gute Benutzererfahrung, da sie ungültige Konfigurationen verhindert.  
**3\. Öffentliche API und Interne Schnittstellen (Rust) für domain::global\_settings\_and\_state\_management**  
Die öffentliche API wird durch den GlobalSettingsService-Trait definiert.

* **3.1. Exakte Signaturen aller öffentlichen Funktionen/Methoden:**  
  * **GlobalSettingsService Trait:**  
    Rust  
    use crate::core::errors::CoreError;  
    use super::types::{GlobalDesktopSettings, SettingPath}; // SettingPath wie oben definiert  
    use super::errors::GlobalSettingsError;  
    use async\_trait::async\_trait;  
    use serde\_json::Value as JsonValue; // Alias für Klarheit

    // SubscriptionId für das Abbestellen von Änderungen  
    // pub type SubscriptionId \= Uuid; // Beispiel

    \#\[async\_trait\]  
    pub trait GlobalSettingsService: Send \+ Sync {  
        /// Lädt die Einstellungen aus der persistenten Speicherung (via Kernschicht).  
        /// Falls keine Konfiguration vorhanden ist oder Fehler auftreten, werden Standardwerte verwendet  
        /// und ggf. eine Fehlermeldung geloggt oder ein spezifischer Fehler zurückgegeben.  
        async fn load\_settings(\&mut self) \-\> Result\<(), GlobalSettingsError\>;

        /// Speichert die aktuellen Einstellungen persistent (via Kernschicht).  
        async fn save\_settings(\&self) \-\> Result\<(), GlobalSettingsError\>;

        /// Gibt eine (tiefe) Kopie der aktuellen \`GlobalDesktopSettings\` zurück.  
        fn get\_current\_settings(\&self) \-\> GlobalDesktopSettings;

        /// Aktualisiert eine spezifische Einstellung unter dem gegebenen \`SettingPath\`.  
        /// Der \`value\`-Parameter ist ein \`serde\_json::Value\`, um Flexibilität zu gewährleisten.  
        /// Interne Logik muss diesen Wert in den korrekten Rust-Typ der Zieleinstellung  
        /// deserialisieren und validieren.  
        async fn update\_setting(  
            \&mut self,  
            path: SettingPath,  
            value: JsonValue  
        ) \-\> Result\<(), GlobalSettingsError\>;

        /// Gibt den Wert einer spezifischen Einstellung unter dem gegebenen \`SettingPath\`  
        /// als \`serde\_json::Value\` zurück.  
        fn get\_setting(\&self, path: \&SettingPath) \-\> Result\<JsonValue, GlobalSettingsError\>;

        /// Setzt alle Einstellungen auf ihre definierten Standardwerte zurück.  
        /// Die Änderungen werden anschließend persistent gespeichert.  
        async fn reset\_to\_defaults(\&mut self) \-\> Result\<(), GlobalSettingsError\>;

        // Die Implementierung von \`subscribe\_to\_setting\_changes\` und \`unsubscribe\`  
        // ist komplex und hängt stark vom gewählten Event-Mechanismus des Projekts ab.  
        // Für eine erste Iteration könnte ein globales \`SettingChangedEvent\` ausreichen,  
        // das den Pfad und den neuen Wert enthält.  
        //  
        // async fn subscribe\_to\_setting\_changes(  
        //     \&self,  
        //     path\_filter: Option\<SettingPath\>, // None für alle Änderungen  
        //     // Der Callback erhält den Pfad und den neuen Wert  
        //     callback: Box\<dyn Fn(SettingPath, JsonValue) \+ Send \+ Sync \+ 'static\>  
        // ) \-\> Result\<SubscriptionId, GlobalSettingsError\>;  
        //  
        // async fn unsubscribe(\&self, id: SubscriptionId) \-\> Result\<(), GlobalSettingsError\>;  
    }

* **3.2. Vor- und Nachbedingungen, Beschreibung der Logik/Algorithmen:**  
  * GlobalSettingsService::update\_setting(path: SettingPath, value: JsonValue):  
    * Vorbedingung:  
      * path muss auf eine gültige, existierende Einstellung innerhalb der GlobalDesktopSettings-Struktur verweisen.  
      * value (JsonValue) muss in den Ziel-Rust-Typ der durch path adressierten Einstellung deserialisierbar sein.  
      * Der deserialisierte Wert muss alle anwendungsspezifischen Validierungsregeln für diese Einstellung erfüllen (z.B. Wertebereich, gültige Enum-Variante).  
    * Logik:  
      1. **Pfad-Navigation:** Navigiere innerhalb der intern gehaltenen GlobalDesktopSettings-Instanz zum durch path spezifizierten Feld. Dies erfordert eine Mapping-Logik vom SettingPath-Enum zu den tatsächlichen Struct-Feldern.  
      2. **Typ-Prüfung und Deserialisierung:** Ermittle den erwarteten Rust-Typ des Zielfeldes. Versuche, das JsonValue in diesen Typ zu deserialisieren (z.B. serde\_json::from\_value::\<TargetType\>(value)).  
         * Bei Fehlschlag: Rückgabe von GlobalSettingsError::InvalidValueType mit Details zum erwarteten und erhaltenen Typ.  
      3. **Validierung:** Führe spezifische Validierungsregeln für die Einstellung durch. Diese Regeln sind Teil der Domänenlogik (z.B. appearance.interface\_scaling\_factor muss zwischen 0.5 und 3.0 liegen).  
         * Bei Fehlschlag: Rückgabe von GlobalSettingsError::ValidationError mit einer beschreibenden Nachricht.  
      4. **Aktualisierung:** Wenn Deserialisierung und Validierung erfolgreich waren, aktualisiere den Wert des Zielfeldes in der internen GlobalDesktopSettings-Instanz.  
      5. **Event-Auslösung:** Löse ein SettingChangedEvent aus, das den path und das (ggf. serialisierte) new\_value enthält, um andere Systemteile zu informieren.  
      6. **Persistenz (optional, konfigurierbar):** Rufe intern save\_settings() auf, um die Änderung sofort persistent zu machen. Alternativ könnten Änderungen gesammelt und später oder auf explizite Anforderung gespeichert werden, um die I/O-Last zu reduzieren. Für eine Desktop-Umgebung ist eine zeitnahe Persistenz meist erwünscht.  
    * Nachbedingung:  
      * Entweder wurde die Einstellung erfolgreich aktualisiert, ein SettingChangedEvent wurde ausgelöst und die Änderung wurde (ggf.) persistiert.  
      * Oder es wurde ein GlobalSettingsError (z.B. PathNotFound, InvalidValueType, ValidationError) zurückgegeben, und der Zustand der Einstellungen bleibt unverändert.  
  * GlobalSettingsService::load\_settings():  
    * Vorbedingung: Keine spezifischen, außer dass der Service initialisiert ist.  
    * Logik:  
      1. Interagiere mit der Kernschicht-Komponente (z.B. core::config), um die GlobalDesktopSettings-Struktur aus einem persistenten Speicher (z.B. Konfigurationsdatei) zu laden.  
      2. Die Kernschicht-Komponente ist für die Deserialisierung der Daten verantwortlich.  
      3. **Fehlerbehandlung beim Laden:**  
         * Wenn die Konfigurationsdatei nicht existiert oder nicht lesbar ist: Verwende die Default::default()-Implementierung von GlobalDesktopSettings (oder eine explizite Methode zur Erzeugung von Standardwerten). Logge eine Warnung.  
         * Wenn die Konfigurationsdatei korrupt ist oder nicht deserialisiert werden kann: Verwende Standardwerte. Logge einen Fehler. GlobalSettingsError::PersistenceError könnte zurückgegeben werden, oder der Service initialisiert sich mit Defaults und loggt den Fehler. Für eine robuste Nutzererfahrung ist das Laden von Defaults oft besser als ein harter Fehler.  
         * Wenn die geladene Konfiguration veraltet ist (z.B. Felder fehlen): serde füllt dank \#\[serde(default)\] fehlende Felder mit ihren Standardwerten auf.  
      4. Speichere die geladenen (oder Standard-) Einstellungen in der internen Instanz von GlobalDesktopSettings.  
      5. Löse ein SettingsLoadedEvent mit den initialisierten Einstellungen aus.  
    * Nachbedingung: Die interne GlobalDesktopSettings-Instanz des Service ist mit den geladenen oder Standardeinstellungen initialisiert. Ein SettingsLoadedEvent wurde ausgelöst.  
* **3.3. Modulspezifische Trait-Definitionen und relevante Implementierungen:**  
  * Der GlobalSettingsService-Trait ist die zentrale öffentliche Schnittstelle.  
  * Alle Einstellungsstrukturen (GlobalDesktopSettings, AppearanceSettings, etc.) müssen std::fmt::Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize und Default implementieren.  
    * Serialize und Deserialize sind fundamental für die Interaktion mit core::config (Persistenz) und für die update\_setting/get\_setting-API, die serde\_json::Value verwendet.  
    * Default ist wichtig für die Erzeugung von Standardkonfigurationen und für \#\[serde(default)\].  
    * PartialEq ist nützlich für Tests und um festzustellen, ob sich ein Wert tatsächlich geändert hat.  
* **3.4. Exakte Definition aller Methoden für Komponenten mit komplexem internen Zustand oder Lebenszyklus:**  
  * Die Hauptkomponente mit komplexem Zustand ist die Implementierung von GlobalSettingsService (z.B. DefaultGlobalSettingsService). Diese Struktur hält die current\_settings: GlobalDesktopSettings als ihren primären Zustand.  
  * Die Komplexität in den Methoden update\_setting und get\_setting liegt in der robusten und korrekten Handhabung des SettingPath:  
    * **Pfad-Auflösung:** Eine effiziente Methode, um von einem SettingPath-Enum-Wert auf das entsprechende Feld in der verschachtelten GlobalDesktopSettings-Struktur zuzugreifen und dessen Typ zu kennen. Dies könnte über match-Anweisungen oder eine komplexere Makro-basierte Lösung erfolgen, um Boilerplate-Code zu reduzieren.  
    * **Dynamische Typkonvertierung:** Die Konvertierung zwischen serde\_json::Value und den stark typisierten Rust-Feldern erfordert sorgfältige Fehlerbehandlung bei der Deserialisierung.  
    * **Validierungslogik:** Die Implementierung der spezifischen Validierungsregeln für jede Einstellung.

**4\. Event-Spezifikationen für domain::global\_settings\_and\_state\_management**  
Events dienen der Benachrichtigung anderer Systemkomponenten über Änderungen an den globalen Einstellungen.

* **Event: SettingChangedEvent**  
  * Event-Typ (Rust-Typ): pub struct SettingChangedEvent { pub path: SettingPath, pub new\_value: JsonValue }  
  * Payload-Struktur: Enthält den SettingPath der geänderten Einstellung und deren neuen Wert als JsonValue. Die Verwendung von JsonValue hier bietet Flexibilität, da der Subscriber den Wert bei Bedarf in den spezifischen Typ deserialisieren kann.  
  * Typische Publisher: Die Implementierung von GlobalSettingsService (nach einem erfolgreichen Aufruf von update\_setting oder reset\_to\_defaults).  
  * Typische Subscriber:  
    * ui::control\_center: Um die Anzeige der Einstellungen in der UI zu aktualisieren.  
    * domain::theming\_engine: Um auf Änderungen in AppearanceSettings (z.B. active\_theme\_name, accent\_color\_token) zu reagieren und das Theme dynamisch neu zu laden/anzuwenden.  
    * system::compositor: Könnte auf Änderungen wie appearance.enable\_animations oder appearance.interface\_scaling\_factor reagieren.  
    * Andere Domänenmodule oder Systemdienste, deren Verhalten von globalen Einstellungen abhängt (z.B. system::input für Mausempfindlichkeit, system::outputs für Standard-Bildschirmhelligkeit basierend auf Energieeinstellungen).  
  * Auslösebedingungen: Eine einzelne Einstellung wurde erfolgreich geändert und validiert. Bei reset\_to\_defaults wird für jede geänderte Einstellung ein separates Event ausgelöst oder ein übergreifendes "Reset"-Event.  
* **Event: SettingsLoadedEvent**  
  * Event-Typ (Rust-Typ): pub struct SettingsLoadedEvent { pub settings: GlobalDesktopSettings }  
  * Payload-Struktur: Enthält eine Kopie der vollständig geladenen GlobalDesktopSettings.  
  * Typische Publisher: Die Implementierung von GlobalSettingsService (nach einem erfolgreichen Aufruf von load\_settings während der Initialisierung).  
  * Typische Subscriber: Initialisierungscode anderer Module, die auf die ersten geladenen Einstellungen warten, um sich zu konfigurieren. UI-Komponenten, um ihren initialen Zustand zu setzen.  
  * Auslösebedingungen: Die globalen Einstellungen wurden erfolgreich initial aus dem persistenten Speicher geladen oder mit Standardwerten initialisiert.  
* **Event: SettingsSavedEvent**  
  * Event-Typ (Rust-Typ): pub struct SettingsSavedEvent; (Kann leer sein, da der reine Akt des Speicherns signalisiert wird. Optional könnten Details wie der Zeitpunkt oder Erfolg/Misserfolg von Teiloperationen enthalten sein, falls relevant.)  
  * Payload-Struktur: In der Regel keine, dient als reines Signal.  
  * Typische Publisher: Die Implementierung von GlobalSettingsService (nach einem erfolgreichen Aufruf von save\_settings).  
  * Typische Subscriber: Logging-Systeme; UI-Komponenten, die dem Benutzer eine kurze Bestätigung anzeigen könnten (z.B. "Einstellungen gespeichert").  
  * Auslösebedingungen: Die aktuellen globalen Einstellungen wurden erfolgreich in den persistenten Speicher geschrieben.  
* **Wichtige Tabelle: Event-Spezifikationen für domain::global\_settings\_and\_state\_management**

| Event-Name/Typ (Rust) | Payload-Struktur (Felder, Typen) | Typische Publisher | Typische Subscriber | Auslösebedingungen |
| :---- | :---- | :---- | :---- | :---- |
| SettingChangedEvent | path: SettingPath, new\_value: JsonValue | GlobalSettingsService | ui::control\_center, domain::theming\_engine, system::compositor, andere Module, die von Einstellungen abhängen | Eine spezifische Einstellung wurde erfolgreich geändert und validiert. |
| SettingsLoadedEvent | settings: GlobalDesktopSettings | GlobalSettingsService | Initialisierungscode von Modulen, UI-Komponenten für initialen Zustand | Globale Einstellungen wurden beim Start erfolgreich geladen oder mit Standardwerten initialisiert. |
| SettingsSavedEvent | (Normalerweise keine, oder Details zum Speichervorgang) | GlobalSettingsService | Logging-Systeme, UI für Feedback | Aktuelle globale Einstellungen wurden erfolgreich persistent gespeichert. |

Diese Event-Struktur ist entscheidend für die Reaktionsfähigkeit und Konsistenz der Desktop-Umgebung. Sie ermöglicht es verschiedenen Teilen des Systems, auf Änderungen der globalen Konfiguration zu reagieren, ohne direkt an den GlobalSettingsService gekoppelt zu sein.  
**5\. Fehlerbehandlung (Rust mit thiserror) für domain::global\_settings\_and\_state\_management**  
Die Fehlerbehandlung folgt den etablierten Projektrichtlinien unter Verwendung von thiserror.

* **Definition des modulspezifischen Error-Enums:**  
  * GlobalSettingsError  
* **Detaillierte Varianten, Nutzung von \#\[error(...)\] und \#\[from\]:**  
  Rust  
  use thiserror::Error;  
  use crate::core::errors::CoreError; // Fehler aus der Kernschicht  
  use super::types::SettingPath; // Annahme: SettingPath implementiert Display oder wird hier formatiert  
  use serde\_json::Error as SerdeJsonError; // Für die Kapselung von serde\_json Fehlern

  // Wrapper für serde\_json::Error, um es Cloneable etc. zu machen, falls GlobalSettingsError das sein muss.  
  // Alternativ kann man auch nur die String-Repräsentation des Fehlers speichern.  
  \#  
  \#  
  pub struct WrappedSerdeJsonError(\#\[from\] SerdeJsonError);

  // Um Clone, PartialEq, Eq für WrappedSerdeJsonError zu ermöglichen, wenn benötigt:  
  // impl Clone for WrappedSerdeJsonError { fn clone(\&self) \-\> Self { WrappedSerdeJsonError(self.0.to\_string()) } } // Vereinfacht  
  // impl PartialEq for WrappedSerdeJsonError { fn eq(\&self, other: \&Self) \-\> bool { self.0.to\_string() \== other.0.to\_string() } }  
  // impl Eq for WrappedSerdeJsonError {}

  \# // Clone, PartialEq, Eq können hinzugefügt werden, wenn die Fehler verglichen werden müssen.  
                         // Dies erfordert, dass alle \#\[source\] Fehler dies ebenfalls unterstützen oder gewrapped werden.  
  pub enum GlobalSettingsError {  
      \#  
      PathNotFound { path\_description: String }, // String-Repräsentation des SettingPath

      \#\[error("Invalid value type provided for setting '{path\_description}'. Expected '{expected\_type}', but got value '{actual\_value\_preview}'.")\]  
      InvalidValueType {  
          path\_description: String,  
          expected\_type: String,  
          actual\_value\_preview: String, // Eine kurze Vorschau des fehlerhaften JSON-Wertes  
      },

      \#\[error("Validation failed for setting '{path\_description}': {message}")\]  
      ValidationError { path\_description: String, message: String },

      \#  
      SerializationError {  
          path\_description: String,  
          \#\[source\] source: WrappedSerdeJsonError,  
      },

      \#  
      DeserializationError {  
          path\_description: String,  
          \#\[source\] source: WrappedSerdeJsonError,  
      },

      // Spezifischer Fehler für Persistenzprobleme, der die CoreError kapselt  
      \#\[error("Persistence error ({operation}) for settings: {message}")\]  
      PersistenceError {  
          operation: String, // "load" oder "save"  
          message: String,  
          \#\[source\] source: Option\<CoreError\>, // CoreError ist hier optional, da der Fehler auch direkt hier entstehen kann  
      },

      // Generischer Fallback für andere CoreErrors, die nicht durch PersistenceError abgedeckt sind  
      \#\[error("An underlying core error occurred: {source}")\]  
      CoreError { \#\[from\] source: CoreError },

      \#\[error("An unexpected internal error occurred in settings management: {0}")\]  
      InternalError(String),  
  }

  // Implementierung, um aus einem serde\_json::Error und Kontext einen GlobalSettingsError zu machen  
  impl GlobalSettingsError {  
      pub fn from\_serde\_deserialize(err: SerdeJsonError, path: \&SettingPath) \-\> Self {  
          GlobalSettingsError::DeserializationError {  
              path\_description: format\!("{:?}", path), // Bessere Formatierung für SettingPath wäre hier gut  
              source: WrappedSerdeJsonError(err),  
          }  
      }  
      pub fn from\_serde\_serialize(err: SerdeJsonError, path: \&SettingPath) \-\> Self {  
          GlobalSettingsError::SerializationError {  
              path\_description: format\!("{:?}", path),  
              source: WrappedSerdeJsonError(err),  
          }  
      }  
  }

  Die WrappedSerdeJsonError-Struktur dient dazu, serde\_json::Error zu kapseln, da dieser Typ selbst nicht unbedingt alle Traits implementiert (wie Clone oder Eq), die für GlobalSettingsError gewünscht sein könnten. Die from\_serde\_deserialize und from\_serde\_serialize Hilfsmethoden erleichtern die Konvertierung.  
* **Spezifikation der Verwendung:**  
  * GlobalSettingsError wird als Err-Variante in den Result-Typen der Methoden des GlobalSettingsService zurückgegeben.  
  * \#\[from\] für CoreError wird für die generische CoreError-Variante verwendet, um nicht anderweitig behandelte Fehler von der Kernschicht (z.B. beim tatsächlichen Lesen/Schreiben von Dateien durch core::config) zu konvertieren.  
  * Die spezifische Variante PersistenceError wird für Fehler verwendet, die direkt beim Laden oder Speichern der Einstellungen auftreten und eine CoreError als Ursache haben können. Dies gibt mehr Kontext als ein generischer CoreError.  
  * SerializationError und DeserializationError kapseln Fehler von serde\_json, die bei der Konvertierung von/zu JsonValue oder beim Speichern/Laden auftreten können.  
  * Die Fehler-Enums und ihre Varianten sind so gestaltet, dass sie den Empfehlungen aus 2 und 1 folgen: spezifische Fehler pro Modul, klare \#\[error(...)\]-Nachrichten und die Möglichkeit des Fehler-Chainings mittels \#\[source\].  
  * Die Granularität der Fehlervarianten wie InvalidValueType und ValidationError ist besonders hervorzuheben. Sie sind nicht nur für das Logging und Debugging durch Entwickler von Bedeutung, sondern können auch dazu dienen, der Benutzeroberflächenschicht (ui::control\_center) präzise Informationen zu liefern, warum eine Einstellungsänderung fehlgeschlagen ist. Beispielsweise kann die UI die path\_description verwenden, um das fehlerhafte Eingabefeld hervorzuheben, und die message aus ValidationError direkt dem Benutzer anzeigen. Dies verbessert die Benutzererfahrung erheblich im Vergleich zu generischen Fehlermeldungen und ist ein direktes Ergebnis der Überlegung, Fehler so zu gestalten, dass sie die Perspektive des Benutzers berücksichtigen, wie in 2 angedeutet ("What happens from the user's perspective.").  
* **Wichtige Tabelle: Fehler-Enum GlobalSettingsError**

| Fehler-Enum | Variante | \#\[error(...)\] Nachricht (Beispiel) | Felder (Typen) | Beschreibung / Auslösekontext |
| :---- | :---- | :---- | :---- | :---- |
| GlobalSettingsError | PathNotFound | "Setting path not found: {path\_description}" | path\_description: String | Der angegebene SettingPath zu einer Einstellung existiert nicht in der GlobalDesktopSettings-Struktur. |
|  | InvalidValueType | "Invalid value type provided for setting '{path\_description}'. Expected '{expected\_type}', got '{actual\_value\_preview}'." | path\_description: String, expected\_type: String, actual\_value\_preview: String | Der für eine Einstellung übergebene JsonValue konnte nicht in den erwarteten Rust-Typ deserialisiert werden. |
|  | ValidationError | "Validation failed for setting '{path\_description}': {message}" | path\_description: String, message: String | Der Wert für eine Einstellung ist zwar vom korrekten Typ, aber ungültig gemäß den Domänenregeln (z.B. außerhalb des erlaubten Wertebereichs). |
|  | SerializationError | "Serialization error for setting '{path\_description}': {source}" | path\_description: String, source: WrappedSerdeJsonError | Fehler bei der Serialisierung eines Einstellungs-Wertes nach JsonValue (z.B. für die get\_setting-Methode oder Event-Payloads). |
|  | DeserializationError | "Deserialization error for setting '{path\_description}': {source}" | path\_description: String, source: WrappedSerdeJsonError | Fehler bei der Deserialisierung eines JsonValue in einen Rust-Typ (z.B. in update\_setting oder beim Laden aus der Kernschicht). |
|  | PersistenceError | "Persistence error ({operation}) for settings: {message}" | operation: String, message: String, source: Option\<CoreError\> | Ein Fehler ist beim Laden ("load") oder Speichern ("save") der Einstellungen durch die Kernschicht aufgetreten. |
|  | CoreError | "An underlying core error occurred: {source}" | \#\[from\] source: CoreError | Ein allgemeiner, nicht spezifisch durch PersistenceError abgedeckter Fehler aus der Kernschicht ist aufgetreten und wurde weitergeleitet. |

Diese detaillierte Fehlerklassifizierung ist für ein robustes Einstellungsmanagement unerlässlich. Sie ermöglicht es aufrufendem Code, differenziert auf Probleme zu reagieren und dem Benutzer kontextsensitive Rückmeldungen zu geben.  
**6\. Detaillierte Implementierungsschritte und Dateistruktur für domain::global\_settings\_and\_state\_management**

* **6.1. Vorgeschlagene Dateistruktur:**  
  src/domain/global\_settings\_management/ // Alternativ: src/domain/settings/  
  ├── mod.rs               // Deklariert Submodule, exportiert öffentliche Typen/Traits  
  ├── service.rs           // Implementierung des GlobalSettingsService (z.B. DefaultGlobalSettingsService)  
  ├── types.rs             // Definition von GlobalDesktopSettings und allen untergeordneten Einstellungs-Structs und \-Enums  
  ├── paths.rs             // Definition von SettingPath und ggf. Hilfsfunktionen zur Pfad-Konvertierung/Navigation  
  └── errors.rs            // Definition von GlobalSettingsError und WrappedSerdeJsonError

* **6.2. Nummerierte, schrittweise Anleitung zur Implementierung:**  
  1. **errors.rs erstellen:** Definieren Sie GlobalSettingsError und die Hilfsstruktur WrappedSerdeJsonError wie im vorherigen Abschnitt spezifiziert.  
  2. **types.rs erstellen:**  
     * Definieren Sie die Hauptstruktur GlobalDesktopSettings.  
     * Definieren Sie alle untergeordneten Einstellungs-Structs (AppearanceSettings, FontSettings, WorkspaceSettings, etc.).  
     * Definieren Sie alle zugehörigen Enums (ColorScheme, FontHinting, LidCloseAction, etc.).  
     * Implementieren Sie für alle diese Strukturen und Enums die notwendigen Traits: Debug, Clone, PartialEq, Serialize, Deserialize und Default. Achten Sie auf die korrekte Verwendung von \#\[serde(default)\] für Felder in Strukturen und \#\[default\] für Enum-Varianten.  
     * Implementieren Sie Default für GlobalDesktopSettings und alle ihre Felder, um einen vollständigen Satz von Standardeinstellungen zu definieren.  
  3. **paths.rs erstellen:**  
     * Definieren Sie die SettingPath-Enum-Hierarchie (z.B. SettingPath, AppearanceSettingPath, FontSettingPath, etc.) wie skizziert.  
     * Implementieren Sie Serialize und Deserialize für SettingPath, falls es über Events oder APIs in serialisierter Form verwendet wird.  
     * Optional: Entwickeln Sie Hilfsfunktionen oder Makros, die das Navigieren in einer GlobalDesktopSettings-Instanz basierend auf einem SettingPath erleichtern oder die Konvertierung zu/von einem String-basierten Pfad (z.B. "appearance.font\_settings.default\_font\_size") ermöglichen.  
  4. **service.rs Basis:**  
     * Definieren Sie den Trait GlobalSettingsService (wie in Abschnitt 3.1).  
     * Erstellen Sie eine Struktur DefaultGlobalSettingsService. Diese wird eine Instanz von GlobalDesktopSettings als internen Zustand halten: settings: GlobalDesktopSettings.  
     * Diese Struktur benötigt eine Abhängigkeit zu einer Komponente der Kernschicht (z.B. einem ConfigManager Trait), um Einstellungen zu laden und zu speichern. Diese Abhängigkeit sollte über den Konstruktor injiziert werden.  
     * Beginnen Sie mit der Implementierung von \#\[async\_trait\] impl GlobalSettingsService for DefaultGlobalSettingsService {... }.  
  5. **Implementierung der GlobalSettingsService-Methoden in DefaultGlobalSettingsService:**  
     * load\_settings: Implementieren Sie die Logik zum Laden der GlobalDesktopSettings von der Kernschicht-Abhängigkeit. Behandeln Sie Fehler beim Laden (Datei nicht vorhanden, korrupt) durch Rückgriff auf GlobalDesktopSettings::default(). Lösen Sie das SettingsLoadedEvent aus.  
     * save\_settings: Implementieren Sie die Logik zum Speichern der aktuellen internen settings über die Kernschicht-Abhängigkeit. Lösen Sie das SettingsSavedEvent aus.  
     * get\_current\_settings: Gibt einen Klon der internen settings-Instanz zurück.  
     * update\_setting: Dies ist die komplexeste Methode.  
       * Implementieren Sie die Pfad-Navigationslogik, um das spezifische Feld innerhalb von self.settings basierend auf dem SettingPath zu identifizieren.  
       * Deserialisieren Sie das JsonValue in den Zieltyp.  
       * Führen Sie die Validierung durch.  
       * Bei Erfolg: Aktualisieren Sie das Feld, lösen Sie das SettingChangedEvent aus und rufen Sie self.save\_settings().await auf.  
       * Geben Sie bei Fehlern die entsprechenden GlobalSettingsError-Varianten zurück.  
     * get\_setting: Implementieren Sie die Pfad-Navigation und serialisieren Sie den gefundenen Wert nach JsonValue.  
     * reset\_to\_defaults: Setzen Sie self.settings \= GlobalDesktopSettings::default();. Lösen Sie für jede (geänderte) Einstellung ein SettingChangedEvent aus (oder ein globales Reset-Event). Rufen Sie self.save\_settings().await auf.  
  6. **mod.rs erstellen:** Deklarieren Sie die Submodule (errors, types, paths, service) und exportieren Sie alle öffentlichen Typen, Traits und Fehler, die von anderen Teilen des Systems verwendet werden sollen.  
  7. **Unit-Tests:** Schreiben Sie umfassende Unit-Tests parallel zur Implementierung jeder Methode. Testen Sie insbesondere die Pfad-Navigation, (De-)Serialisierung, Validierungslogik und Fehlerfälle in update\_setting und get\_setting. Mocken Sie die Kernschicht-Abhängigkeit für Lade-/Speicheroperationen.

**7\. Interaktionen und Abhängigkeiten (domain::global\_settings\_and\_state\_management)**

* **Nutzung von Funktionalitäten der Kernschicht:**  
  * core::config (oder eine äquivalente Komponente/Trait): Dies ist die Hauptabhängigkeit für die Persistenz. Der GlobalSettingsService delegiert das tatsächliche Lesen von und Schreiben in Konfigurationsdateien (oder andere Speicherorte) an diese Kernschichtkomponente. Der Service stellt die Logik und die Datenstruktur (GlobalDesktopSettings) bereit, während core::config die I/O-Operationen und die (De-)Serialisierung von/zu einem bestimmten Dateiformat (z.B. TOML, JSON) übernimmt.  
  * core::errors: CoreError-Typen, die von core::config zurückgegeben werden (z.B. I/O-Fehler, Formatierungsfehler), werden in spezifischere GlobalSettingsError::PersistenceError oder die generische GlobalSettingsError::CoreError Variante umgewandelt.  
  * core::logging: Das tracing-Framework wird für internes Logging verwendet, z.B. um das Laden von Einstellungen, aufgetretene Fehler oder erfolgreiche Speicheroperationen zu protokollieren.  
* **Schnittstellen zu System- und UI-Schicht:**  
  * ui::control\_center: Dies ist der primäre Konsument des GlobalSettingsService in der UI-Schicht. Das Control Center wird:  
    * get\_current\_settings() oder multiple get\_setting() Aufrufe verwenden, um die aktuellen Werte für die Anzeige zu laden.  
    * update\_setting() aufrufen, wenn der Benutzer eine Einstellung ändert.  
    * Das SettingChangedEvent abonnieren, um die UI dynamisch zu aktualisieren, falls Einstellungen anderweitig (z.B. durch reset\_to\_defaults oder programmatisch) geändert werden.  
  * **Systemschicht-Komponenten:** Verschiedene Komponenten der Systemschicht können Einstellungen aus dem GlobalSettingsService lesen, um ihr Verhalten anzupassen:  
    * system::compositor: Könnte AppearanceSettings.enable\_animations, AppearanceSettings.interface\_scaling\_factor oder InputBehaviorSettings.mouse\_acceleration\_profile lesen.  
    * system::input: Könnte Einstellungen für Tastaturwiederholrate, Mausempfindlichkeit oder Touchpad-Verhalten (InputBehaviorSettings) anwenden.  
    * system::outputs (Display-Management): Könnte Standardwerte für Bildschirmhelligkeit oder Timeouts bis zum Blanking des Bildschirms aus PowerManagementPolicySettings beziehen.  
    * system::audio: Könnte eine globale Lautstärkeeinstellung oder Standardausgabegeräte hierüber beziehen, falls solche Einstellungen als global definiert werden.  
* **Interaktionen mit anderen Modulen der Domänenschicht:**  
  * domain::theming\_engine: Ein sehr enger Konsument. Liest alle relevanten AppearanceSettings (Theme-Name, Akzentfarbe, Schriftarten, Icons, Cursor) und muss auf SettingChangedEvent für diese Pfade reagieren, um das Desktop-Theme dynamisch neu zu generieren und anzuwenden.  
  * domain::workspace\_and\_window\_policy (oder domain::workspaces und domain::window\_management): Liest WorkspaceSettings (z.B. dynamische Workspaces) und relevante InputBehaviorSettings (z.B. Mausverhalten für Fensterinteraktionen).  
  * domain::user\_centric\_services: Könnte globale Standardeinstellungen für KI-Interaktionen (z.B. default\_ai\_model\_id, falls als globale Einstellung definiert) oder Benachrichtigungen (z.B. global\_do\_not\_disturb\_default\_state, max\_notification\_history\_override) aus dem GlobalSettingsService beziehen.

**8\. Testaspekte für Unit-Tests (domain::global\_settings\_and\_state\_management)**  
Die Testbarkeit dieses Moduls ist entscheidend für die Stabilität der gesamten Desktop-Umgebung.

* **Identifikation testkritischer Logik:**  
  * Die korrekte Deserialisierung von JsonValue in den spezifischen Rust-Typ der Zieleinstellung und die anschließende Validierung dieses Wertes in update\_setting. Dies umfasst die Behandlung von Typ-Mismatch und Wertebereichsverletzungen.  
  * Die korrekte Navigation zu verschachtelten Feldern innerhalb der GlobalDesktopSettings-Struktur mittels SettingPath in update\_setting und get\_setting.  
  * Die Fehlerbehandlung für ungültige Pfade (PathNotFound), falsche Wertetypen (InvalidValueType) und ungültige Werte (ValidationError).  
  * Die Logik zum Laden von Standardwerten (Default::default()) und das korrekte Mergen mit einer möglicherweise unvollständigen, aber gültigen Konfiguration aus dem persistenten Speicher (Sicherstellung, dass \#\[serde(default)\] wie erwartet funktioniert).  
  * Die korrekte Auslösung von SettingChangedEvent mit dem korrekten SettingPath und JsonValue als Payload nach einer erfolgreichen Aktualisierung.  
  * Die Interaktion mit der (gemockten) core::config-Schicht für Lade- und Speicheroperationen, einschließlich der korrekten Fehlerweitergabe.  
  * Die Funktionalität von reset\_to\_defaults.  
* **Beispiele für Testfälle:**  
  * test\_load\_settings\_new\_system\_uses\_defaults\_and\_fires\_loaded\_event  
  * test\_load\_settings\_existing\_config\_loads\_correctly\_and\_fires\_loaded\_event  
  * test\_load\_settings\_partial\_config\_fills\_missing\_with\_defaults  
  * test\_load\_settings\_corrupted\_config\_falls\_back\_to\_defaults\_logs\_error (benötigt Mock für core::config, der Fehler simuliert)  
  * test\_update\_setting\_valid\_value\_updates\_internal\_state\_fires\_changed\_event\_and\_saves  
  * test\_update\_setting\_valid\_value\_for\_nested\_path  
  * test\_update\_setting\_invalid\_json\_type\_returns\_invalid\_value\_type\_error (z.B. String für boolesches Feld)  
  * test\_update\_setting\_value\_violates\_validation\_rule\_returns\_validation\_error (z.B. Schriftgröße 200, wenn max 72\)  
  * test\_update\_setting\_nonexistent\_path\_returns\_path\_not\_found\_error  
  * test\_get\_setting\_existing\_path\_returns\_correct\_value\_as\_json  
  * test\_get\_setting\_nonexistent\_path\_returns\_path\_not\_found\_error  
  * test\_reset\_to\_defaults\_restores\_all\_settings\_fires\_changed\_events\_and\_saves  
  * Für jede Einstellungsstruktur (AppearanceSettings, etc.): Testen der (De-)Serialisierungslogik (serde) und der Default-Implementierung.  
  * Testen der SettingPath-Navigation: Sicherstellen, dass jeder definierte Pfad korrekt auf ein Feld zugreift.  
* **Mocking:**  
  * Eine Mock-Implementierung für die von core::config bereitgestellte Schnittstelle (z.B. ein trait ConfigPersistence) ist unerlässlich. Diese Mock-Implementierung muss es ermöglichen, erfolgreiche Lade-/Speicheroperationen sowie verschiedene Fehlerszenarien (Datei nicht gefunden, Lesefehler, Schreibfehler, korrupte Daten) zu simulieren. Crates wie mockall können hierfür verwendet werden.  
  * Der Event-Auslösemechanismus sollte ebenfalls mockbar sein, um zu verifizieren, dass Events korrekt und mit den richtigen Payloads gesendet werden.

---

**Zusammenfassende Betrachtungen zur Domänenschicht (für Teil 3/4)**  
Die in diesem Dokument detailliert spezifizierten Module domain::user\_centric\_services und domain::global\_settings\_and\_state\_management bilden zwei zentrale Säulen der Domänenschicht. Sie sind maßgeblich dafür verantwortlich, die Kernlogik für eine intelligente, personalisierte und anpassbare Benutzererfahrung bereitzustellen.  
Das Modul domain::user\_centric\_services kapselt die komplexe Logik für KI-gestützte Funktionen und das Benachrichtigungssystem. Die sorgfältige Definition von Entitäten wie AIInteractionContext und AIConsent, gepaart mit robusten Prozessen für das Einwilligungsmanagement, stellt sicher, dass KI-Funktionen verantwortungsvoll und unter Wahrung der Benutzerkontrolle integriert werden können. Das NotificationService bietet eine flexible und erweiterbare Grundlage für die Verwaltung aller System- und Anwendungsbenachrichtigungen.  
Das Modul domain::global\_settings\_and\_state\_management schafft die Voraussetzung für eine hochgradig konfigurierbare Desktop-Umgebung. Durch die zentrale, typsichere und validierte Verwaltung aller globalen Einstellungen in der GlobalDesktopSettings-Struktur und dem zugehörigen GlobalSettingsService wird Konsistenz über das gesamte System hinweg gewährleistet. Die Verwendung von serde für die (De-)Serialisierung und die klare Definition von SettingPath ermöglichen eine flexible und dennoch robuste Handhabung von Konfigurationsänderungen.  
Für beide Module ist die detaillierte Spezifikation der Fehlerbehandlung mittels thiserror von entscheidender Bedeutung. Die bewusste Entscheidung für spezifische Fehlervarianten und kontextreiche Fehlermeldungen, wie sie auch durch die Analyse der Referenzmaterialien 1 gestützt wird, erhöht nicht nur die Wartbarkeit und Debugfähigkeit des Codes, sondern ermöglicht es auch, dem Benutzer über die UI präzisere und hilfreichere Rückmeldungen bei Problemen zu geben. Die konsequente Auslösung von Events bei relevanten Zustandsänderungen ist fundamental für die Entkopplung der Module und die dynamische Reaktion der Benutzeroberfläche.  
Die hier vorgelegten Ultrafeinspezifikationen bieten eine solide Grundlage für die Implementierung dieser Domänenkomponenten. Die Entwickler können diese Pläne nutzen, um Module zu erstellen, die nicht nur funktional korrekt sind, sondern auch den hohen Anforderungen an Stabilität, Erweiterbarkeit und Benutzerfreundlichkeit der geplanten Desktop-Umgebung gerecht werden.


---

# **B4 Domänenschicht (Domain Layer) – Teil 4/4: Einstellungs- und Benachrichtigungs-Subsysteme**

Dieser Teil der Spezifikation widmet sich den verbleibenden Kernkomponenten der Domänenschicht: dem Subsystem für die Verwaltung von Einstellungen (domain::settings\_core und domain::settings\_persistence\_iface) sowie dem Subsystem für die Verarbeitung und Regelung von Benachrichtigungen (domain::notifications\_core und domain::notifications\_rules). Diese Module sind entscheidend für die Konfigurierbarkeit und das reaktive Verhalten der Desktop-Umgebung.

## **4.1. Entwicklungsmodul: Kernlogik für Einstellungen (domain::settings\_core)**

Dieses Modul bildet das Herzstück der Einstellungsverwaltung innerhalb der Domänenschicht. Es ist verantwortlich für die Definition, Validierung, Speicherung (über eine Abstraktionsschicht) und den Zugriff auf alle Konfigurationseinstellungen der Desktop-Umgebung.

### **4.1.1. Detaillierte Verantwortlichkeiten und Ziele**

* **Verantwortlichkeiten:**  
  * Definition der Struktur und Typen von Einstellungen (SettingKey, SettingValue, SettingMetadata, Setting).  
  * Bereitstellung einer zentralen Logik (SettingsCoreManager) zur Verwaltung dieser Einstellungen.  
  * Validierung von Einstellungswerten gegen definierte Metadaten (Typ, Bereich, erlaubte Werte).  
  * Koordination des Ladens und Speicherns von Einstellungen über eine abstrakte Persistenzschnittstelle (SettingsProvider).  
  * Benachrichtigung anderer Systemteile über Einstellungsänderungen mittels interner Events (SettingChangedEvent).  
* **Ziele:**  
  * Schaffung einer typsicheren und validierten Verwaltung von Einstellungen.  
  * Entkopplung der Einstellungslogik von der konkreten Speicherung und der Benutzeroberfläche.  
  * Ermöglichung einer reaktiven Anpassung des Systemverhaltens basierend auf Konfigurationsänderungen.  
  * Sicherstellung der Konsistenz und Integrität der Einstellungen.

### **4.1.2. Entitäten und Wertobjekte**

Die folgenden Datenstrukturen sind in domain/src/settings\_core/types.rs zu definieren. Sie müssen Debug, Clone und PartialEq implementieren. Für die Persistenz und den Datenaustausch ist zudem die Implementierung von serde::Serialize und serde::Deserialize für SettingValue und die darin enthaltenen Typen essenziell. Die uuid Crate wird für eindeutige IDs verwendet, wobei die Features v4 (zur Generierung) und serde (zur Serialisierung) aktiviert sein müssen.1 Für Zeitstempel wird chrono mit dem aktivierten serde-Feature eingesetzt.3

* **SettingKey (Newtype für String)**  
  * **Zweck:** Ein typsicherer Wrapper für den Schlüssel einer Einstellung (z.B. "appearance.theme.name", "notifications.do\_not\_disturb.enabled").  
  * **Warum wertvoll:** Erhöht die Typsicherheit und verhindert die versehentliche Verwendung beliebiger Strings als Einstellungsschlüssel. Fördert Klarheit im Code.  
  * **Implementierungsdetails:**  
    * Interner Typ: String.  
    * Sollte Display, Hash, Eq, PartialEq, Ord, PartialOrd, From\<String\>, AsRef\<str\> implementieren.  
    * Konstruktion z.B. über SettingKey::new("my.setting.key") oder From::from("my.setting.key").  
* **SettingValue (Enum)**  
  * **Zweck:** Ein Enum, das alle möglichen Typen von Einstellungswerten repräsentiert.  
  * **Warum wertvoll:** Ermöglicht eine flexible, aber dennoch typsichere Behandlung unterschiedlicher Einstellungsdatentypen an einer zentralen Stelle.  
  * **Varianten:**  
    * Boolean(bool)  
    * Integer(i64)  
    * Float(f64)  
    * String(String)  
    * Color(String): Hex-Farbcode (z.B. "\#RRGGBBAA").  
    * FilePath(String): Ein Pfad zu einer Datei oder einem Verzeichnis.  
    * List(Vec\<SettingValue\>): Eine geordnete Liste von SettingValue.  
    * Map(std::collections::HashMap\<String, SettingValue\>): Eine Schlüssel-Wert-Map.  
  * **Methoden (Beispiele):**  
    * pub fn as\_bool(\&self) \-\> Option\<bool\>  
    * pub fn as\_str(\&self) \-\> Option\<\&str\>  
    * Weitere as\_TYPE und try\_into\_TYPE Methoden für bequemen Zugriff und Konvertierung.  
* **SettingMetadata Struktur**  
  * **Zweck:** Enthält Metadaten zu einer Einstellung, wie Beschreibung, Standardwert, mögliche Werte (für Enums), Validierungsregeln.  
  * **Warum wertvoll:** Ermöglicht eine deklarative Definition von Einstellungen und deren Eigenschaften. Dies ist fundamental, um die Verwaltung, die automatische Generierung von Benutzeroberflächen für Einstellungen und die Validierung zu vereinfachen. Ohne Metadaten wäre jede Einstellungslogik ad-hoc und schwer zu warten.

| Attribut | Typ | Sichtbarkeit | Beschreibung |
| :---- | :---- | :---- | :---- |
| description | Option\<String\> | pub | Menschenlesbare Beschreibung der Einstellung. |
| default\_value | SettingValue | pub | Der Standardwert, der verwendet wird, wenn kein Wert gesetzt ist. |
| value\_type\_hint | String | pub | Hinweis auf den erwarteten SettingValue-Typ (z.B. "Boolean", "Integer"). |
| possible\_values | Option\<Vec\<SettingValue\>\> | pub | Für Enum-Typen: eine Liste der erlaubten Werte. |
| validation\_regex | Option\<String\> | pub | Für String-Typen: ein regulärer Ausdruck zur Validierung. |
| min\_value | Option\<SettingValue\> | pub | Für numerische Typen: der minimale erlaubte Wert. |
| max\_value | Option\<SettingValue\> | pub | Für numerische Typen: der maximale erlaubte Wert. |
| is\_sensitive | bool | pub | Gibt an, ob der Wert sensibel ist (z.B. Passwort, nicht loggen). Default: false. |
| requires\_restart | Option\<String\> | pub | Wenn Some(app\_id\_or\_service\_name), deutet an, dass eine Änderung einen Neustart der genannten Komponente erfordert. None bedeutet keinen Neustart. |

* **Setting Struktur (Entität)**  
  * **Zweck:** Repräsentiert eine einzelne, konkrete Einstellung mit ihrem aktuellen Wert und Metadaten.  
  * **Warum wertvoll:** Das zentrale Objekt, das eine Einstellung im System darstellt und deren Zustand und Verhalten kapselt.

| Attribut | Typ | Sichtbarkeit | Beschreibung | Invarianten |
| :---- | :---- | :---- | :---- | :---- |
| id | uuid::Uuid | pub | Eindeutige ID der Einstellung (intern verwendet). | Muss eindeutig sein. Generiert via Uuid::new\_v4(). |
| key | SettingKey | pub | Der eindeutige Schlüssel der Einstellung. | Muss eindeutig sein. |
| current\_value | SettingValue | pub(crate) | Der aktuell gesetzte Wert der Einstellung. | Muss den Validierungsregeln in metadata entsprechen, falls gesetzt. |
| metadata | SettingMetadata | pub | Metadaten, die diese Einstellung beschreiben. |  |
| last\_modified | chrono::DateTime\<Utc\> | pub(crate) | Zeitstempel der letzten Änderung. | Wird bei jeder erfolgreichen Wertänderung aktualisiert. |
| is\_dirty | bool | pub(crate) | true, wenn current\_value geändert wurde, aber noch nicht persistiert ist. |  |

\*   \*\*Methoden:\*\*  
    \*   \`pub fn new(key: SettingKey, metadata: SettingMetadata) \-\> Self\`: Erstellt eine neue Einstellung. Der \`current\_value\` wird initial auf \`metadata.default\_value\` gesetzt. \`id\` wird generiert. \`last\_modified\` wird auf \`Utc::now()\` gesetzt.  
    \*   \`pub fn value(\&self) \-\> \&SettingValue\`: Gibt eine Referenz auf den aktuellen Wert zurück.  
    \*   \`pub(crate) fn set\_value(\&mut self, new\_value: SettingValue, timestamp: chrono::DateTime\<Utc\>) \-\> Result\<(), SettingsCoreError\>\`: Setzt einen neuen Wert, nachdem dieser erfolgreich gegen \`self.metadata\` validiert wurde (interner Aufruf von \`validate\_value\`). Aktualisiert \`current\_value\` und \`last\_modified\`.  
    \*   \`pub fn validate\_value(value: \&SettingValue, metadata: \&SettingMetadata) \-\> Result\<(), SettingsCoreError\>\`: Statische Methode zur Validierung eines Wertes gegen die gegebenen Metadaten. Diese Methode ist separat, um auch externe Validierung zu ermöglichen, bevor \`set\_value\` aufgerufen wird.  
    \*   \`pub fn reset\_to\_default(\&mut self, timestamp: chrono::DateTime\<Utc\>)\`: Setzt den \`current\_value\` auf \`self.metadata.default\_value\` zurück und aktualisiert \`last\_modified\`.

### **4.1.3. Öffentliche API des Moduls (SettingsCoreManager)**

Der SettingsCoreManager, definiert in domain/src/settings\_core/mod.rs, ist die zentrale Schnittstelle zur Einstellungslogik. Er kapselt die Verwaltung der Setting-Objekte und die Interaktion mit dem Persistenz-Provider.  
Die Operationen zum Laden und Speichern von Einstellungen können I/O-intensiv sein. Um die Domänenschicht nicht zu blockieren, werden diese Methoden als async deklariert. Dies erfordert, dass der SettingsProvider ebenfalls asynchrone Methoden anbietet und als Arc\<dyn SettingsProvider \+ Send \+ Sync\> übergeben wird, um Thread-Sicherheit in asynchronen Kontexten zu gewährleisten.5  
Wenn Einstellungen geändert werden, müssen andere Teile der Domänenschicht (z.B. die NotificationRulesEngine oder das Theming-System) potenziell darüber informiert werden. Um eine lose Kopplung zu erreichen, sendet der SettingsCoreManager interne Events (SettingChangedEvent) über einen tokio::sync::broadcast::Sender.6 Interessierte Module können einen broadcast::Receiver abonnieren und auf diese Events reagieren, ohne dass der SettingsCoreManager explizite Kenntnis von ihnen haben muss. Dieser Mechanismus ist entscheidend für eine reaktive Architektur.

Rust

// domain/src/settings\_core/mod.rs  
use crate::settings\_persistence\_iface::{SettingsProvider, SettingsPersistenceError};  
use crate::settings\_core::types::{Setting, SettingKey, SettingValue, SettingMetadata};  
use crate::settings\_core::error::SettingsCoreError;  
use std::collections::HashMap;  
use std::sync::Arc;  
use tokio::sync::{RwLock, broadcast};  
use chrono::Utc;

\# // Clone ist wichtig für den broadcast::Sender  
pub struct SettingChangedEvent {  
    pub key: SettingKey,  
    pub new\_value: SettingValue,  
    pub old\_value: Option\<SettingValue\>,  
}

pub struct SettingsCoreManager {  
    settings: RwLock\<HashMap\<SettingKey, Setting\>\>,  
    provider: Arc\<dyn SettingsProvider \+ Send \+ Sync\>,  
    event\_sender: broadcast::Sender\<SettingChangedEvent\>,  
    registered\_metadata: RwLock\<HashMap\<SettingKey, SettingMetadata\>\>, // RwLock auch hier für dynamische Registrierung  
}

impl SettingsCoreManager {  
    pub fn new(  
        provider: Arc\<dyn SettingsProvider \+ Send \+ Sync\>,  
        initial\_metadata: Vec\<(SettingKey, SettingMetadata)\>,  
        event\_channel\_capacity: usize  
    ) \-\> Self {  
        let (event\_sender, \_) \= broadcast::channel(event\_channel\_capacity);  
        let mut metadata\_map \= HashMap::new();  
        for (key, meta) in initial\_metadata {  
            metadata\_map.insert(key, meta);  
        }

        SettingsCoreManager {  
            settings: RwLock::new(HashMap::new()),  
            provider,  
            event\_sender,  
            registered\_metadata: RwLock::new(metadata\_map),  
        }  
    }

    // Weitere Methoden folgen  
}

* **Tabelle: Methoden des SettingsCoreManager**

| Methode | Signatur | Kurzbeschreibung | Vorbedingungen | Nachbedingungen (Erfolg) | Nachbedingungen (Fehler) |
| :---- | :---- | :---- | :---- | :---- | :---- |
| new | pub fn new(provider: Arc\<dyn SettingsProvider \+ Send \+ Sync\>, initial\_metadata: Vec\<(SettingKey, SettingMetadata)\>, event\_channel\_capacity: usize) \-\> Self | Konstruktor. Initialisiert den Manager mit Provider, Metadaten und Event-Kanal-Kapazität. | provider ist valide. event\_channel\_capacity \> 0\. | SettingsCoreManager ist initialisiert. event\_sender ist erstellt. settings ist leer. registered\_metadata ist gefüllt. | \- |
| register\_setting\_metadata | pub async fn register\_setting\_metadata(\&self, key: SettingKey, metadata: SettingMetadata) \-\> Result\<(), SettingsCoreError\> | Registriert Metadaten für eine neue Einstellung zur Laufzeit. | key ist noch nicht registriert. | Metadaten für key sind in registered\_metadata gespeichert. | SettingsCoreError::SettingKeyAlreadyExists |
| load\_all\_settings | pub async fn load\_all\_settings(\&self) \-\> Result\<(), SettingsCoreError\> | Lädt alle Einstellungen, für die Metadaten registriert sind, vom SettingsProvider. | provider ist erreichbar. | Interne settings-Map ist mit geladenen Werten (oder Defaults aus Metadaten) gefüllt. | SettingsCoreError::PersistenceError, SettingsCoreError::ValidationError |
| get\_setting\_value | pub async fn get\_setting\_value(\&self, key: \&SettingKey) \-\> Result\<SettingValue, SettingsCoreError\> | Ruft den aktuellen Wert einer Einstellung ab. Lädt ggf. nach, falls nicht im Speicher. | key muss registriert sein. | SettingValue des Schlüssels wird zurückgegeben. | SettingsCoreError::SettingNotFound, SettingsCoreError::UnregisteredKey, SettingsCoreError::PersistenceError |
| set\_setting\_value | pub async fn set\_setting\_value(\&self, key: \&SettingKey, value: SettingValue) \-\> Result\<(), SettingsCoreError\> | Setzt den Wert einer Einstellung. Validiert und persistiert den Wert. Sendet ein Event. | key muss registriert sein. value muss valide sein gemäß Metadaten. | Wert ist intern gesetzt, persistiert via provider. SettingChangedEvent wird gesendet. last\_modified im Setting aktualisiert. | SettingsCoreError::SettingNotFound, SettingsCoreError::UnregisteredKey, SettingsCoreError::ValidationError, SettingsCoreError::PersistenceError |
| reset\_setting\_to\_default | pub async fn reset\_setting\_to\_default(\&self, key: \&SettingKey) \-\> Result\<(), SettingsCoreError\> | Setzt eine Einstellung auf ihren Standardwert (aus Metadaten) zurück. Persistiert und sendet Event. | key muss registriert sein. | Wert ist intern auf Default gesetzt, persistiert. SettingChangedEvent wird gesendet. last\_modified aktualisiert. | SettingsCoreError::SettingNotFound, SettingsCoreError::UnregisteredKey, SettingsCoreError::PersistenceError |
| get\_all\_settings\_with\_metadata | pub async fn get\_all\_settings\_with\_metadata(\&self) \-\> Result\<Vec\<Setting\>, SettingsCoreError\> | Gibt eine Liste aller aktuell verwalteten Einstellungen (inkl. ihrer Werte und Metadaten) zurück. | \- | Eine Vec\<Setting\> mit Klonen aller Einstellungen. | SettingsCoreError::PersistenceError (falls Nachladen nötig und fehlschlägt) |
| subscribe\_to\_changes | pub fn subscribe\_to\_changes(\&self) \-\> broadcast::Receiver\<SettingChangedEvent\> | Gibt einen Receiver für SettingChangedEvents zurück, um auf Einstellungsänderungen zu reagieren. | \- | Ein neuer broadcast::Receiver\<SettingChangedEvent\>. | \- |

### **4.1.4. Interne Events (SettingChangedEvent)**

Definiert in domain/src/settings\_core/mod.rs (siehe oben).

* **Zweck:** Entkoppelte Benachrichtigung anderer Domänenkomponenten über Einstellungsänderungen.  
* **Warum wertvoll:** Ermöglicht eine reaktive Architektur innerhalb der Domänenschicht. Module können auf Änderungen reagieren, ohne dass der SettingsCoreManager sie kennen muss. Dies reduziert die Kopplung und erhöht die Wartbarkeit und Erweiterbarkeit des Systems. Beispielsweise kann das Theming-Modul auf Änderungen der Akzentfarbe reagieren, ohne dass der SettingsCoreManager das Theming-Modul explizit aufrufen muss.  
* **Struktur SettingChangedEvent:**

| Feld | Typ | Beschreibung |
| :---- | :---- | :---- |
| key | SettingKey | Der Schlüssel der geänderten Einstellung. |
| new\_value | SettingValue | Der neue Wert der Einstellung. |
| old\_value | Option\<SettingValue\> | Der vorherige Wert der Einstellung (falls vorhanden oder nicht Standardwert). |

* **Typische Publisher:** SettingsCoreManager (nach erfolgreichem set\_setting\_value oder reset\_setting\_to\_default).  
* **Typische Subscriber (intern in Domänenschicht):** NotificationRulesEngine (um z.B. auf "Nicht stören"-Modus zu reagieren), ThemingEngine (um auf Theme- oder Akzentfarbänderungen zu reagieren), potenziell andere Domänenmodule, die einstellungsabhängige Logik haben.

### **4.1.5. Fehlerbehandlung (SettingsCoreError)**

Definiert in domain/src/settings\_core/error.rs unter Verwendung der thiserror-Crate, gemäß Richtlinie 4.3 der Gesamtspezifikation. Die Verwendung von thiserror für Bibliotheks-Code ist vorteilhaft, da sie spezifische, typisierte Fehler ermöglicht, die von Aufrufern explizit behandelt werden können, im Gegensatz zu generischen Fehlertypen wie anyhow::Error oder Box\<dyn std::error::Error\>.8 Die \#\[from\]-Annotation erleichtert die Konvertierung von Fehlern aus anderen Modulen (z.B. SettingsPersistenceError) in Varianten von SettingsCoreError.10

Rust

// domain/src/settings\_core/error.rs  
use thiserror::Error;  
use crate::settings\_core::types::SettingKey;  
use crate::settings\_persistence\_iface::SettingsPersistenceError;

\#  
pub enum SettingsCoreError {  
    \#  
    SettingNotFound { key: SettingKey },

    \#  
    SettingKeyAlreadyExists { key: SettingKey },

    \#\[error("Validation failed for setting '{key}': {message}")\]  
    ValidationError { key: SettingKey, message: String },

    \#\[error("Persistence operation failed for setting '{key\_str}': {source}")\]  
    PersistenceError {  
        key\_str: String, // String, da SettingKey nicht immer verfügbar oder relevant für globalen Fehler  
        \#\[source\]  
        source: SettingsPersistenceError,  
    },

    \#\[error("Attempted to operate on an unregistered setting key: '{key}'")\]  
    UnregisteredKey { key: SettingKey },

    \#\[error("An underlying I/O error occurred: {0}")\]  
    IoError(\#\[from\] std::io::Error), // Für den Fall, dass das Modul selbst I/O machen würde (selten)

    \#\[error("Event channel error while processing key '{key\_str}': {message}")\]  
    EventChannelError{ key\_str: String, message: String },  
}

// Konvertierung von SettingsPersistenceError zu SettingsCoreError  
// Dies ist nützlich, wenn ein Persistenzfehler auftritt, der nicht direkt einem Schlüssel zugeordnet ist.  
impl From\<SettingsPersistenceError\> for SettingsCoreError {  
    fn from(err: SettingsPersistenceError) \-\> Self {  
        SettingsCoreError::PersistenceError {  
            key\_str: err.get\_key().map\_or\_else(|| "global".to\_string(), |k| k.as\_str().to\_string()),  
            source: err,  
        }  
    }  
}

(Hinweis: SettingsPersistenceError müsste eine Methode get\_key() \-\> Option\<\&SettingKey\> haben, um dies sauber zu implementieren.)

* **Tabelle: SettingsCoreError Varianten**

| Variante | Beschreibung | Kontext/Ursache |
| :---- | :---- | :---- |
| SettingNotFound | Eine angeforderte Einstellung existiert nicht in der internen settings-Map. | get\_setting\_value, set\_setting\_value für einen Schlüssel, der zwar registriert, aber nicht geladen ist. |
| SettingKeyAlreadyExists | Versuch, Metadaten für einen bereits existierenden Schlüssel zu registrieren. | register\_setting\_metadata. |
| ValidationError | Ein neuer Wert für eine Einstellung entspricht nicht den Validierungsregeln. | set\_setting\_value, interne Validierung durch Setting::validate\_value. |
| PersistenceError | Fehler bei der Interaktion mit dem SettingsProvider. | Wrappt Fehler vom SettingsProvider (z.B. SettingsPersistenceError::StorageError). Verwendet \#\[source\]. |
| UnregisteredKey | Operation auf einem Schlüssel ohne registrierte Metadaten. | Wenn eine Operation Metadaten erfordert (z.B. set\_setting\_value), diese aber für den Schlüssel fehlen. |
| IoError | Generischer I/O-Fehler (eher selten direkt hier, mehr für SettingsProvider). | Beispiel für \#\[from\] std::io::Error. |
| EventChannelError | Fehler beim Senden eines SettingChangedEvent über den broadcast::Sender. | Wenn der broadcast::Sender::send() einen Fehler zurückgibt (z.B. keine aktiven Receiver und Puffer voll). |

### **4.1.6. Detaillierte Implementierungsschritte und Algorithmen**

1. **Initialisierung (SettingsCoreManager::new):**  
   * Speichere den übergebenen provider und die initial\_metadata (in RwLock\<HashMap\<...\>\>).  
   * Erstelle den broadcast::channel für SettingChangedEvent mit der spezifizierten Kapazität.  
   * Die settings-Map (RwLock\<HashMap\<SettingKey, Setting\>\>) bleibt initial leer. Einstellungen werden lazy oder durch load\_all\_settings geladen.  
2. **SettingsCoreManager::register\_setting\_metadata:**  
   * Erwirb Schreibsperre für registered\_metadata.  
   * Prüfe, ob key bereits existiert. Wenn ja, Err(SettingsCoreError::SettingKeyAlreadyExists).  
   * Füge (key, metadata) zu registered\_metadata hinzu. Ok(()).  
3. **SettingsCoreManager::load\_all\_settings:**  
   * Erwirb Lesesperre für registered\_metadata und Schreibsperre für settings.  
   * Iteriere über alle (key, metadata) in registered\_metadata.  
   * Für jeden key:  
     * Rufe self.provider.load\_setting(\&key).await auf.  
     * Bei Ok(Some(loaded\_value)):  
       * Validiere loaded\_value gegen metadata mittels Setting::validate\_value(\&loaded\_value, \&metadata). Bei Fehler: Err(SettingsCoreError::ValidationError).  
       * Erstelle ein Setting-Objekt: let setting \= Setting { id: uuid::Uuid::new\_v4(), key: key.clone(), current\_value: loaded\_value, metadata: metadata.clone(), last\_modified: Utc::now(), is\_dirty: false };  
       * Füge (key.clone(), setting) zur settings-Map hinzu.  
     * Bei Ok(None) (kein Wert persistiert):  
       * Verwende metadata.default\_value. Erstelle Setting-Objekt wie oben, aber mit metadata.default\_value.clone().  
       * Füge zur settings-Map hinzu.  
     * Bei Err(persistence\_error): Konvertiere zu SettingsCoreError::PersistenceError und gib Fehler zurück. Breche den Ladevorgang ab.  
   * Ok(()) bei Erfolg.  
4. **SettingsCoreManager::get\_setting\_value:**  
   * Erwirb Lesesperre für registered\_metadata. Prüfe, ob key registriert ist. Wenn nein, Err(SettingsCoreError::UnregisteredKey).  
   * Erwirb Lesesperre für settings.  
   * Wenn key in settings vorhanden ist, gib settings.get(key).unwrap().value().clone() zurück.  
   * Wenn key nicht in settings vorhanden (nicht geladen):  
     * Gib Lesesperre für settings frei.  
     * Rufe self.provider.load\_setting(key).await auf.  
     * Erwirb Schreibsperre für settings.  
     * Bei Ok(Some(loaded\_value)):  
       * Hole metadata aus registered\_metadata.  
       * Validiere loaded\_value. Bei Fehler: Err(SettingsCoreError::ValidationError).  
       * Erstelle Setting-Objekt, füge zu settings hinzu. Gib loaded\_value.clone() zurück.  
     * Bei Ok(None):  
       * Hole metadata aus registered\_metadata.  
       * Erstelle Setting-Objekt mit metadata.default\_value. Füge zu settings hinzu. Gib metadata.default\_value.clone() zurück.  
     * Bei Err(persistence\_error): Err(SettingsCoreError::from(persistence\_error)).  
   * Stelle sicher, dass Sperren korrekt freigegeben werden, besonders bei frühen Returns.  
5. **SettingsCoreManager::set\_setting\_value:**  
   * Erwirb Lesesperre für registered\_metadata. Hole metadata für key. Wenn nicht gefunden: Err(SettingsCoreError::UnregisteredKey).  
   * Validiere value gegen metadata mittels Setting::validate\_value(\&value, \&retrieved\_metadata). Bei Fehler: Err(SettingsCoreError::ValidationError).  
   * Erwirb Schreibsperre für settings.  
   * Hole das (mutable) Setting-Objekt für key. Wenn nicht gefunden (sollte nach get\_setting\_value-Logik oder load\_all\_settings existieren, aber zur Sicherheit prüfen oder entry() API verwenden): Err(SettingsCoreError::SettingNotFound).  
   * Speichere old\_value \= current\_setting.value().clone().  
   * Rufe current\_setting.set\_value(value.clone(), Utc::now()) auf (dies validiert intern nicht erneut, da bereits geschehen).  
   * Setze current\_setting.is\_dirty \= true.  
   * Rufe self.provider.save\_setting(key, \&value).await auf.  
     * Bei Err(persistence\_error):  
       * Setze current\_setting.set\_value(old\_value, Utc::now()) (Rollback der In-Memory-Änderung).  
       * Setze current\_setting.is\_dirty \= false.  
       * Err(SettingsCoreError::from(persistence\_error)).  
   * Setze current\_setting.is\_dirty \= false.  
   * Erstelle SettingChangedEvent { key: key.clone(), new\_value: value, old\_value: Some(old\_value) }.  
   * Sende das Event via self.event\_sender.send(). Bei Fehler (z.B. wenn keine Subscriber da sind und der Kanal voll ist, was bei broadcast selten zu einem harten Fehler führt, aber Err zurückgeben kann): Err(SettingsCoreError::EventChannelError).  
   * Ok(()).  
6. **Validierungslogik (Setting::validate\_value):**  
   * Prüfe Typkompatibilität von value mit metadata.value\_type\_hint (z.B. SettingValue::Integer mit "Integer").  
   * Wenn metadata.possible\_values Some(list) ist, prüfe, ob value in list enthalten ist.  
   * Wenn metadata.validation\_regex Some(regex\_str) ist und value ein SettingValue::String(s) ist, kompiliere Regex und prüfe s dagegen.  
   * Prüfe metadata.min\_value / metadata.max\_value für numerische Typen (Integer, Float).  
   * Bei Verletzung: Err(SettingsCoreError::ValidationError) mit passender Nachricht.

### **4.1.7. Überlegungen zur Nebenläufigkeit und Zustandssynchronisierung**

* Die internen Zustände settings und registered\_metadata werden mit tokio::sync::RwLock geschützt. Dies erlaubt parallele Lesezugriffe, während Schreibzugriffe exklusiv sind, was für typische Einstellungs-Workloads (viele Lesezugriffe, wenige Schreibzugriffe) performant ist.  
* Der SettingsProvider wird als Arc\<dyn SettingsProvider \+ Send \+ Sync\> gehalten. Send und Sync sind notwendig, da die async-Methoden des SettingsCoreManager potenziell auf verschiedenen Threads durch den Tokio-Executor ausgeführt werden können und der Provider über Thread-Grenzen hinweg sicher geteilt werden muss.5  
* Der broadcast::Sender für SettingChangedEvent ist Thread-sicher und für die Verwendung in asynchronen Kontexten konzipiert.6

## **4.2. Entwicklungsmodul: Persistenzabstraktion und Schema für Einstellungen (domain::settings\_persistence\_iface)**

Dieses Modul definiert die Schnittstelle, über die der SettingsCoreManager Einstellungen lädt und speichert, ohne die konkrete Implementierung der Persistenz zu kennen.

### **4.2.1. Detaillierte Verantwortlichkeiten und Ziele**

* **Verantwortlichkeiten:**  
  * Definition eines abstrakten Traits (SettingsProvider), der die Operationen zum Laden und Speichern von Einstellungen vorschreibt.  
  * Definition der Fehlertypen (SettingsPersistenceError), die bei Persistenzoperationen auftreten können.  
* **Ziele:**  
  * Vollständige Entkopplung der Domänenlogik (domain::settings\_core) von spezifischen Speichertechnologien (z.B. GSettings, Konfigurationsdateien im TOML/JSON-Format, Datenbank).  
  * Ermöglichung der Testbarkeit des SettingsCoreManager durch Mocking des SettingsProvider.  
  * Flexibilität bei der Auswahl oder dem Wechsel der Speichertechnologie, ohne dass Änderungen an der Domänenschicht erforderlich sind.

Die Verwendung eines Trait-Objekts (Arc\<dyn SettingsProvider \+ Send \+ Sync\>) ist hier entscheidend. Die Send \+ Sync-Bounds sind unerlässlich, da der Provider in async-Funktionen verwendet wird, die von einem Multi-Threaded-Executor wie Tokio ausgeführt werden können. Ohne diese Bounds könnte der Compiler die Thread-Sicherheit nicht garantieren.5

### **4.2.2. Trait-Definitionen (SettingsProvider)**

Definiert in domain/src/settings\_persistence\_iface/mod.rs. Die Verwendung von async\_trait ist notwendig, um async fn in Traits zu deklarieren, solange dies nicht nativ in stabilem Rust unterstützt wird.

Rust

// domain/src/settings\_persistence\_iface/mod.rs  
use async\_trait::async\_trait;  
use crate::settings\_core::types::{SettingKey, SettingValue};  
use crate::settings\_persistence\_iface::error::SettingsPersistenceError; // Eigener Fehlertyp

\#\[async\_trait\]  
pub trait SettingsProvider {  
    async fn load\_setting(\&self, key: \&SettingKey) \-\> Result\<Option\<SettingValue\>, SettingsPersistenceError\>;  
    async fn save\_setting(\&self, key: \&SettingKey, value: \&SettingValue) \-\> Result\<(), SettingsPersistenceError\>;  
    async fn load\_all\_settings(\&self) \-\> Result\<Vec\<(SettingKey, SettingValue)\>, SettingsPersistenceError\>;  
    async fn delete\_setting(\&self, key: \&SettingKey) \-\> Result\<(), SettingsPersistenceError\>;  
    async fn setting\_exists(\&self, key: \&SettingKey) \-\> Result\<bool, SettingsPersistenceError\>;  
}

* **Tabelle: Methoden des SettingsProvider Traits**

| Methode | Signatur | Kurzbeschreibung |
| :---- | :---- | :---- |
| load\_setting | async fn load\_setting(\&self, key: \&SettingKey) \-\> Result\<Option\<SettingValue\>, SettingsPersistenceError\> | Lädt den Wert für einen Schlüssel. Ok(None) wenn nicht vorhanden. |
| save\_setting | async fn save\_setting(\&self, key: \&SettingKey, value: \&SettingValue) \-\> Result\<(), SettingsPersistenceError\> | Speichert einen Wert für einen Schlüssel. Überschreibt, falls existent. |
| load\_all\_settings | async fn load\_all\_settings(\&self) \-\> Result\<Vec\<(SettingKey, SettingValue)\>, SettingsPersistenceError\> | Lädt alle Einstellungen, die dieser Provider verwaltet (z.B. unter einem Schema). |
| delete\_setting | async fn delete\_setting(\&self, key: \&SettingKey) \-\> Result\<(), SettingsPersistenceError\> | Löscht eine Einstellung aus dem persistenten Speicher. |
| setting\_exists | async fn setting\_exists(\&self, key: \&SettingKey) \-\> Result\<bool, SettingsPersistenceError\> | Prüft, ob eine Einstellung im persistenten Speicher existiert. |

### **4.2.3. Datenstrukturen für die Persistenzschnittstelle**

Die primären Datenstrukturen, die über diese Schnittstelle ausgetauscht werden, sind SettingKey und SettingValue aus dem Modul domain::settings\_core::types. Es wird implizit erwartet, dass Implementierungen des SettingsProvider-Traits mit serialisierbaren Formen von SettingValue arbeiten können. Daher müssen SettingValue und die darin enthaltenen Typen serde::Serialize und serde::Deserialize implementieren. Die konkrete Serialisierungslogik (z.B. zu JSON, GVariant für GSettings, etc.) ist Aufgabe der jeweiligen Provider-Implementierung in der Systemschicht, nicht der Domänenschicht.

### **4.2.4. Fehlerbehandlung (SettingsPersistenceError)**

Definiert in domain/src/settings\_persistence\_iface/error.rs unter Verwendung von thiserror. Diese Fehler sind spezifisch für Persistenzoperationen und werden vom SettingsCoreManager in SettingsCoreError::PersistenceError gewrappt.

Rust

// domain/src/settings\_persistence\_iface/error.rs  
use thiserror::Error;  
use crate::settings\_core::types::SettingKey;

\#  
pub enum SettingsPersistenceError {  
    \#  
    BackendUnavailable { message: String },

    \#\[error("Failed to access storage for key '{key}': {message}")\]  
    StorageAccessError { key: SettingKey, message: String },

    \#\[error("Failed to serialize setting '{key}': {message}")\]  
    SerializationError { key: SettingKey, message: String },

    \#\[error("Failed to deserialize setting '{key}': {message}")\]  
    DeserializationError { key: SettingKey, message: String },

    \#  
    SettingNotFoundInStorage { key: SettingKey }, // Eindeutiger als der allgemeine SettingNotFound

    \#\[error("An I/O error occurred while accessing storage for key '{key\_opt:?}': {source}")\]  
    IoError {  
        key\_opt: Option\<SettingKey\>,  
        \#\[source\]  
        source: std::io::Error,  
    },

    \#\[error("An unknown persistence error occurred for key '{key\_opt:?}': {message}")\]  
    UnknownError { key\_opt: Option\<SettingKey\>, message: String },  
}

impl SettingsPersistenceError {  
    /// Hilfsmethode, um den Schlüssel aus dem Fehler zu extrahieren, falls vorhanden.  
    pub fn get\_key(\&self) \-\> Option\<\&SettingKey\> {  
        match self {  
            SettingsPersistenceError::StorageAccessError { key,.. } \=\> Some(key),  
            SettingsPersistenceError::SerializationError { key,.. } \=\> Some(key),  
            SettingsPersistenceError::DeserializationError { key,.. } \=\> Some(key),  
            SettingsPersistenceError::SettingNotFoundInStorage { key,.. } \=\> Some(key),  
            SettingsPersistenceError::IoError { key\_opt,.. } \=\> key\_opt.as\_ref(),  
            SettingsPersistenceError::UnknownError { key\_opt,.. } \=\> key\_opt.as\_ref(),  
            \_ \=\> None,  
        }  
    }  
}

* **Tabelle: SettingsPersistenceError Varianten**

| Variante | Beschreibung |
| :---- | :---- |
| BackendUnavailable | Das Speichersystem (z.B. D-Bus Dienst, Datenbankverbindung) ist nicht erreichbar. |
| StorageAccessError | Allgemeiner Fehler beim Zugriff auf den Speicher für einen bestimmten Schlüssel. |
| SerializationError | Fehler beim Serialisieren eines SettingValue für die Speicherung. |
| DeserializationError | Fehler beim Deserialisieren eines Wertes aus dem Speicher in ein SettingValue. |
| SettingNotFoundInStorage | Spezifischer Fehler, wenn ein Schlüssel im Persistenzlayer nicht existiert. |
| IoError | Wrappt std::io::Error für dateibasierte Provider. Enthält optional den Schlüssel. |
| UnknownError | Ein anderer, nicht spezifisch klassifizierter Fehler. Enthält optional den Schlüssel. |

### **4.2.5. Detaillierte Implementierungsschritte für die Interaktion mit der Schnittstelle**

Die konkreten Implementierungen des SettingsProvider-Traits (z.B. GSettingsProvider, FileConfigProvider) befinden sich typischerweise in der Systemschicht oder einer dedizierten Infrastrukturschicht, da sie systemspezifische Details oder externe Bibliotheken involvieren.  
Der SettingsCoreManager interagiert wie folgt mit dem Provider:

1. Der SettingsCoreManager hält eine Instanz von Arc\<dyn SettingsProvider \+ Send \+ Sync\>.  
2. Bei Operationen wie set\_setting\_value ruft der SettingsCoreManager die entsprechende Methode des Providers auf, z.B. provider.save\_setting(\&key, \&value).await.  
3. Gibt die Provider-Methode Ok(...) zurück, fährt der SettingsCoreManager mit seiner Logik fort (internen Zustand aktualisieren, Event senden).  
4. Gibt die Provider-Methode Err(SettingsPersistenceError) zurück, konvertiert der SettingsCoreManager diesen Fehler in eine SettingsCoreError::PersistenceError-Variante (unter Beibehaltung des ursprünglichen Fehlers als source mittels \#\[from\] oder manueller Implementierung) und gibt diesen an seinen Aufrufer weiter. Der interne Zustand des SettingsCoreManager wird gegebenenfalls auf den Stand vor dem fehlgeschlagenen Persistenzversuch zurückgesetzt (Rollback).

Diese klare Trennung stellt sicher, dass die Domänenlogik agnostisch gegenüber der Persistenztechnologie bleibt und erleichtert das Testen erheblich, da der Provider durch einen Mock ersetzt werden kann.

## **4.3. Entwicklungsmodul: Kernlogik der Benachrichtigungsverwaltung (domain::notifications\_core)**

Dieses Modul ist für die zentrale Logik der Verwaltung von Desktop-Benachrichtigungen zuständig. Es definiert, was eine Benachrichtigung ist, wie sie verarbeitet, gespeichert und ihr Lebenszyklus verwaltet wird.

### **4.3.1. Detaillierte Verantwortlichkeiten und Ziele**

* **Verantwortlichkeiten:**  
  * Definition der Datenstruktur einer Benachrichtigung (Notification) und zugehöriger Typen (NotificationId, NotificationAction, NotificationUrgency).  
  * Verwaltung des Lebenszyklus von Benachrichtigungen: Erstellung, Anzeige (konzeptionell, die Darstellung erfolgt in der UI-Schicht), Aktualisierung, Schließen.  
  * Bereitstellung einer API (NotificationCoreManager) zum programmatischen Hinzufügen und Verwalten von Benachrichtigungen.  
  * Führung einer Liste aktiver Benachrichtigungen.  
  * Verwaltung einer Benachrichtigungshistorie mit konfigurierbarer Größe und Persistenzlogik (FIFO).  
  * Unterstützung für interaktive Benachrichtigungen durch NotificationAction.  
  * Implementierung von Logik zur Deduplizierung oder zum Ersetzen von Benachrichtigungen (z.B. basierend auf replaces\_id).  
  * Interaktion mit der NotificationRulesEngine (domain::notifications\_rules) zur Anwendung von Filter-, Priorisierungs- und Modifikationsregeln.  
  * Versenden interner Events (NotificationEvent) über Zustandsänderungen von Benachrichtigungen.  
* **Ziele:**  
  * Schaffung einer zentralen, konsistenten und robusten Logik für das gesamte Benachrichtigungssystem.  
  * Strikte Trennung der Benachrichtigungslogik von der UI-Darstellung und den Transportmechanismen (wie D-Bus). Die Domänenschicht definiert *was* eine Benachrichtigung ist und *wie* sie verwaltet wird, nicht wie sie konkret aussieht oder über welche Kanäle sie empfangen/gesendet wird.  
  * Ermöglichung eines flexiblen und durch Regeln steuerbaren Benachrichtigungsflusses.

### **4.3.2. Entitäten und Wertobjekte**

Alle Typen sind in domain/src/notifications\_core/types.rs zu definieren. Sie benötigen standardmäßig Debug, Clone, PartialEq. Für die Persistenz der Historie und die Verwendung in Events ist auch serde::Serialize und serde::Deserialize für die Hauptstrukturen (Notification, NotificationAction etc.) erforderlich. uuid::Uuid (mit Features v4, serde) 1 und chrono::DateTime\<Utc\> (mit Feature serde) 3 werden für IDs bzw. Zeitstempel verwendet.

* **NotificationId (Newtype für uuid::Uuid)**  
  * **Zweck:** Eine typsichere ID für Benachrichtigungen.  
  * **Warum wertvoll:** Verhindert Verwechslungen mit anderen Uuid-basierten IDs im System und macht die API expliziter.  
  * **Implementierungsdetails:**  
    * Interner Typ: uuid::Uuid.  
    * Sollte Display, Hash, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize, Copy (da Uuid Copy ist) implementieren.  
    * Methoden: pub fn new() \-\> Self { Self(uuid::Uuid::new\_v4()) }, pub fn as\_uuid(\&self) \-\> \&uuid::Uuid { \&self.0 }, From\<uuid::Uuid\>, Into\<uuid\_Uuid\>.  
* **NotificationUrgency (Enum)**  
  * **Zweck:** Definiert die Dringlichkeitsstufe einer Benachrichtigung.  
  * **Warum wertvoll:** Standardisiert die Dringlichkeit und ermöglicht darauf basierende Logik in Regeln und UI (z.B. Sortierung, Hervorhebung, unterschiedliche Töne).

| Variante | Wert (intern, z.B. u8) | Beschreibung |
| :---- | :---- | :---- |
| Low | 0 | Niedrige Dringlichkeit (z.B. informative Updates). |
| Normal | 1 | Normale Dringlichkeit (Standard). |
| Critical | 2 | Hohe Dringlichkeit (z.B. Fehler, wichtige Alarme). |

\*   Sollte \`serde::Serialize\`, \`serde::Deserialize\`, \`Copy\` implementieren.

* **NotificationAction (Struktur, Wertobjekt)**  
  * **Zweck:** Repräsentiert eine Aktion, die der Benutzer im Kontext einer Benachrichtigung ausführen kann.  
  * **Warum wertvoll:** Ermöglicht interaktive Benachrichtigungen, die über reine Informationsanzeige hinausgehen.

| Attribut | Typ | Sichtbarkeit | Beschreibung |
| :---- | :---- | :---- | :---- |
| key | String | pub | Eindeutiger Schlüssel der Aktion innerhalb der Benachrichtigung (z.B. "reply", "archive"). |
| label | String | pub | Menschenlesbare Beschriftung für den Button/Menüeintrag (z.B. "Antworten"). |
| icon\_name | Option\<String\> | pub | Optionaler Name eines Icons für die Aktion (gemäß Freedesktop Icon Naming Spec). |

\*   Sollte \`serde::Serialize\`, \`serde::Deserialize\` implementieren.

* **Notification (Struktur, Entität)**  
  * **Zweck:** Das zentrale Objekt, das eine einzelne Benachrichtigung mit all ihren Attributen darstellt.  
  * **Warum wertvoll:** Kapselt alle Informationen einer Benachrichtigung und dient als Hauptdatentyp für die Benachrichtigungslogik.

| Attribut | Typ | Sichtbarkeit | Beschreibung | Invarianten |
| :---- | :---- | :---- | :---- | :---- |
| id | NotificationId | pub | Eindeutige ID der Benachrichtigung. | Muss eindeutig sein. Wird bei Erstellung generiert. |
| app\_name | String | pub | Name der Anwendung, die die Benachrichtigung gesendet hat (z.B. "Thunderbird", "System Update"). | Nicht leer. |
| app\_icon | Option\<String\> | pub | Pfad oder Name des Icons der Anwendung (gemäß Freedesktop Icon Naming Spec). |  |
| summary | String | pub | Kurze Zusammenfassung/Titel der Benachrichtigung. | Nicht leer. |
| body | Option\<String\> | pub | Ausführlicherer Text der Benachrichtigung. Kann Markup enthalten (abhängig von UI-Interpretation). |  |
| actions | Vec\<NotificationAction\> | pub | Liste von Aktionen, die mit der Benachrichtigung verbunden sind. | Schlüssel (key) jeder Aktion müssen innerhalb dieser Liste eindeutig sein. |
| hints | HashMap\<String, SettingValue\> | pub | Zusätzliche, anwendungsspezifische Daten oder UI-Hinweise (z.B. "image-path", "progress", "resident"). |  |
| urgency | NotificationUrgency | pub | Dringlichkeitsstufe. Default: Normal. |  |
| timestamp\_created | chrono::DateTime\<Utc\> | pub | Zeitstempel der Erstellung der Benachrichtigung *in der Domänenschicht*. | Wird bei Instanziierung gesetzt. |
| timestamp\_displayed | Option\<chrono::DateTime\<Utc\>\> | pub(crate) | Zeitstempel, wann die Benachrichtigung (potenziell) dem Benutzer angezeigt wurde (von NotificationCoreManager gesetzt). |  |
| expires\_at | Option\<chrono::DateTime\<Utc\>\> | pub | Zeitstempel, wann die Benachrichtigung automatisch geschlossen werden soll (None \= kein Timeout). |  |
| is\_persistent | bool | pub | true, wenn die Benachrichtigung nach dem Schließen in der Historie verbleiben soll. Default: false. |  |
| replaces\_id | Option\<NotificationId\> | pub | ID der Benachrichtigung, die durch diese ersetzt werden soll. |  |
| category | Option\<String\> | pub | Kategorie der Benachrichtigung (z.B. "email.new", "download.complete", "chat.incoming\_message"). Standardisierte Kategorien können für Regeln nützlich sein. |  |

\*   Sollte \`serde::Serialize\`, \`serde::Deserialize\` implementieren.

* **NotificationHistory (Struktur, Aggregatwurzel)**  
  * **Zweck:** Verwaltet die Sammlung der vergangenen (geschlossenen, persistenten) Benachrichtigungen.  
  * **Warum wertvoll:** Stellt die Logik für die Historie bereit, insbesondere die Begrenzung der Größe und den Zugriff.  
  * **Implementierungsdetails:**  
    * notifications: VecDeque\<Notification\>: Eine VecDeque ist geeignet, da sie effizientes Hinzufügen am einen Ende und Entfernen am anderen Ende (für die Größenbeschränkung) ermöglicht.  
    * max\_size: usize: Maximale Anzahl an Benachrichtigungen in der Historie.  
    * Methoden:  
      * pub fn new(max\_size: usize) \-\> Self  
      * pub fn add(\&mut self, notification: Notification): Fügt eine Benachrichtigung hinzu. Wenn max\_size überschritten wird, wird die älteste entfernt (pop\_front).  
      * pub fn get\_all(\&self) \-\> Vec\<Notification\>: Gibt eine Kopie aller historischen Benachrichtigungen zurück (neueste zuerst oder älteste zuerst, je nach Anforderung).  
      * pub fn get\_paged(\&self, limit: usize, offset: usize) \-\> Vec\<Notification\>: Gibt eine Seite der Historie zurück.  
      * pub fn clear(\&mut self): Leert die Historie.  
      * pub fn current\_size(\&self) \-\> usize.  
  * Sollte serde::Serialize, serde::Deserialize implementieren, um die gesamte Historie persistieren zu können (optional, aber nützlich).

### **4.3.3. Öffentliche API des Moduls (NotificationCoreManager)**

Definiert in domain/src/notifications\_core/mod.rs. Der NotificationCoreManager ist die Fassade für die Benachrichtigungslogik. Er verwaltet intern Listen für aktive Benachrichtigungen und eine Instanz von NotificationHistory. Er interagiert eng mit der NotificationRulesEngine.

Rust

// domain/src/notifications\_core/mod.rs  
use crate::notifications\_core::types::{Notification, NotificationId, NotificationAction, NotificationHistory, NotificationUrgency}; // NotificationUrgency für Defaults  
use crate::notifications\_core::error::NotificationCoreError;  
use crate::notifications\_core::events::{NotificationEvent, CloseReason};  
use crate::notifications\_rules::{NotificationRulesEngine, RuleProcessingResult}; // Abhängigkeit  
use std::collections::{HashMap, VecDeque};  
use std::sync::Arc;  
use tokio::sync::{RwLock, broadcast};  
use chrono::Utc;

pub struct NotificationCoreManager {  
    active\_notifications: RwLock\<HashMap\<NotificationId, Notification\>\>,  
    history: RwLock\<NotificationHistory\>,  
    rules\_engine: Arc\<NotificationRulesEngine\>,  
    event\_sender: broadcast::Sender\<NotificationEvent\>,  
    // next\_internal\_id: RwLock\<u32\>, // Für Freedesktop Notification Spec Server ID, falls benötigt  
}

impl NotificationCoreManager {  
    pub fn new(  
        rules\_engine: Arc\<NotificationRulesEngine\>,  
        history\_max\_size: usize,  
        event\_channel\_capacity: usize  
    ) \-\> Self {  
        let (event\_sender, \_) \= broadcast::channel(event\_channel\_capacity);  
        NotificationCoreManager {  
            active\_notifications: RwLock::new(HashMap::new()),  
            history: RwLock::new(NotificationHistory::new(history\_max\_size)),  
            rules\_engine,  
            event\_sender,  
        }  
    }

    // Weitere Methoden folgen  
}

* **Tabelle: Methoden des NotificationCoreManager**

| Methode | Signatur | Kurzbeschreibung |
| :---- | :---- | :---- |
| new | pub fn new(rules\_engine: Arc\<NotificationRulesEngine\>, history\_max\_size: usize, event\_channel\_capacity: usize) \-\> Self | Konstruktor. Initialisiert den Manager mit der Regel-Engine, maximaler Historiengröße und Event-Kanal-Kapazität. |
| add\_notification | pub async fn add\_notification(\&self, mut new\_notification: Notification) \-\> Result\<NotificationId, NotificationCoreError\> | Fügt eine neue Benachrichtigung hinzu. Wendet Regeln an, prüft auf Ersetzung. Sendet NotificationAdded oder NotificationSuppressedByRule Event. Gibt die ID der (ggf. modifizierten) Benachrichtigung zurück. |
| get\_active\_notification | pub async fn get\_active\_notification(\&self, id: \&NotificationId) \-\> Result\<Option\<Notification\>, NotificationCoreError\> | Ruft eine aktive Benachrichtigung anhand ihrer ID ab (als Klon). |
| get\_all\_active\_notifications | pub async fn get\_all\_active\_notifications(\&self) \-\> Result\<Vec\<Notification\>, NotificationCoreError\> | Ruft eine Liste aller derzeit aktiven Benachrichtigungen ab (als Klone). |
| close\_notification | pub async fn close\_notification(\&self, id: \&NotificationId, reason: CloseReason) \-\> Result\<(), NotificationCoreError\> | Schließt eine aktive Benachrichtigung. Verschiebt sie ggf. in die Historie (basierend auf is\_persistent und reason). Sendet NotificationClosed Event. |
| invoke\_action | pub async fn invoke\_action(\&self, notification\_id: \&NotificationId, action\_key: \&str) \-\> Result\<(), NotificationCoreError\> | Löst eine Aktion für eine Benachrichtigung aus. Sendet NotificationActionInvoked Event. Die eigentliche Ausführung der Aktion ist nicht Teil dieser Domänenlogik. |
| get\_history | pub async fn get\_history(\&self, limit: Option\<usize\>, offset: Option\<usize\>) \-\> Result\<Vec\<Notification\>, NotificationCoreError\> | Ruft Benachrichtigungen aus der Historie ab (paginiert). |
| clear\_history | pub async fn clear\_history(\&self) \-\> Result\<(), NotificationCoreError\> | Leert die Benachrichtigungshistorie. Sendet NotificationHistoryCleared Event. |
| clear\_app\_notifications | pub async fn clear\_app\_notifications(\&self, app\_name: \&str, reason: CloseReason) \-\> Result\<usize, NotificationCoreError\> | Schließt alle aktiven Benachrichtigungen einer bestimmten App. Gibt Anzahl geschlossener Benachrichtigungen zurück. |
| subscribe\_to\_events | pub fn subscribe\_to\_events(\&self) \-\> broadcast::Receiver\<NotificationEvent\> | Gibt einen Receiver für NotificationEvents zurück, um auf Benachrichtigungs-Events zu reagieren. |

### **4.3.4. Interne Events (NotificationEvent)**

Definiert in domain/src/notifications\_core/events.rs. Diese Events werden über tokio::sync::broadcast 6 verteilt, was eine entkoppelte Kommunikation innerhalb des Systems ermöglicht.

* **Zweck:** Andere Teile des Systems (primär die UI-Schicht über Adaptoren in der Systemschicht, aber auch andere Domänenmodule oder Logging-Dienste) über signifikante Änderungen im Benachrichtigungssystem zu informieren.  
* **Warum wertvoll:** Entkoppelte Kommunikation ist ein Schlüsselprinzip für modulare und wartbare Systeme. Die UI muss nicht direkt vom NotificationCoreManager aufgerufen werden; sie reagiert stattdessen auf Events.

Rust

// domain/src/notifications\_core/events.rs  
use crate::notifications\_core::types::{Notification, NotificationId};  
use chrono::{DateTime, Utc}; // Für Zeitstempel in Events

\# // Clone für Sender, PartialEq für Tests, Serde für ggf. externe Weiterleitung  
pub enum CloseReason {  
    DismissedByUser,  
    Expired,  
    Replaced,  
    AppClosed,      // App hat explizit CloseNotification gerufen  
    SystemShutdown,  
    AppScopeClear,  // Durch clear\_app\_notifications  
    Other(String),  
}

\# // Clone für Sender, Serde für ggf. externe Weiterleitung  
pub enum NotificationEvent {  
    NotificationAdded {  
        notification: Notification, // Die tatsächlich hinzugefügte (ggf. modifizierte) Notification  
        timestamp: DateTime\<Utc\>,  
    },  
    NotificationUpdated { // Falls Benachrichtigungen aktualisiert werden können (z.B. Fortschritt)  
        notification: Notification, // Die aktualisierte Notification  
        timestamp: DateTime\<Utc\>,  
    },  
    NotificationClosed {  
        notification\_id: NotificationId,  
        app\_name: String, // Nützlich für UI, um schnell zuordnen zu können  
        summary: String,  // Nützlich für UI  
        reason: CloseReason,  
        timestamp: DateTime\<Utc\>,  
    },  
    NotificationActionInvoked {  
        notification\_id: NotificationId,  
        action\_key: String,  
        timestamp: DateTime\<Utc\>,  
    },  
    NotificationHistoryCleared {  
        timestamp: DateTime\<Utc\>,  
    },  
    NotificationSuppressedByRule {  
        original\_summary: String, // Nur einige Infos, nicht die ganze Notification  
        app\_name: String,  
        rule\_id: String, // ID der verantwortlichen Regel  
        timestamp: DateTime\<Utc\>,  
    }  
}

* **Tabelle: NotificationEvent Varianten**

| Variante | Payload-Felder | Beschreibung |
| :---- | :---- | :---- |
| NotificationAdded | notification: Notification, timestamp | Eine neue Benachrichtigung wurde dem System hinzugefügt und ist (nach Regelprüfung) aktiv. |
| NotificationUpdated | notification: Notification, timestamp | Eine bestehende aktive Benachrichtigung wurde aktualisiert (z.B. Fortschrittsbalken). |
| NotificationClosed | notification\_id: NotificationId, app\_name, summary, reason: CloseReason, timestamp | Eine aktive Benachrichtigung wurde geschlossen. app\_name und summary für leichtere UI-Verarbeitung. |
| NotificationActionInvoked | notification\_id: NotificationId, action\_key: String, timestamp | Eine Aktion einer Benachrichtigung wurde ausgelöst. |
| NotificationHistoryCleared | timestamp | Die Benachrichtigungshistorie wurde geleert. |
| NotificationSuppressedByRule | original\_summary: String, app\_name: String, rule\_id: String, timestamp | Eine eingehende Benachrichtigung wurde aufgrund einer Regel unterdrückt und nicht aktiv angezeigt. |

* **Typische Publisher:** NotificationCoreManager.  
* **Typische Subscriber:** Die UI-Schicht (über einen Adapter in der Systemschicht, der D-Bus-Signale oder Wayland-Events generiert), Logging-Dienste, potenziell andere Domänenmodule, die auf Benachrichtigungsstatus reagieren müssen.

### **4.3.5. Fehlerbehandlung (NotificationCoreError)**

Definiert in domain/src/notifications\_core/error.rs mit thiserror.9

Rust

// domain/src/notifications\_core/error.rs  
use thiserror::Error;  
use crate::notifications\_core::types::NotificationId;  
use crate::notifications\_rules::error::NotificationRulesError;

\#  
pub enum NotificationCoreError {  
    \#  
    NotificationNotFound(NotificationId),

    \#\[error("Action '{action\_key}' not found for notification '{notification\_id}'.")\]  
    ActionNotFound {  
        notification\_id: NotificationId,  
        action\_key: String,  
    },

    \#\[error("Failed to apply notification rules: {source}")\]  
    RuleApplicationError {  
        \#\[from\] // Direkte Konvertierung von NotificationRulesError  
        source: NotificationRulesError  
    },

    \#\[error("Notification history is full (max size: {max\_size}). Cannot add notification '{summary}'.")\]  
    HistoryFull { max\_size: usize, summary: String },

    \#\[error("Invalid notification data: {message}")\]  
    InvalidNotificationData { message: String },

    \#\[error("Event channel error: {message}")\]  
    EventChannelError { message: String },

    \#  
    DuplicateNotificationId(NotificationId),

    \#  
    ReplacedNotificationNotFound(NotificationId),  
}

* **Tabelle: NotificationCoreError Varianten**

| Variante | Beschreibung |
| :---- | :---- |
| NotificationNotFound | Eine angeforderte Benachrichtigung (aktiv) wurde nicht gefunden. |
| ActionNotFound | Eine angeforderte Aktion für eine Benachrichtigung existiert nicht. |
| RuleApplicationError | Fehler bei der Anwendung von Regeln aus NotificationRulesEngine. Nutzt \#\[from\] für direkte Konvertierung. |
| HistoryFull | Die Benachrichtigungshistorie hat ihre maximale Kapazität erreicht und eine weitere kann nicht hinzugefügt werden. |
| InvalidNotificationData | Die Daten der hinzuzufügenden Benachrichtigung sind ungültig (z.B. fehlender summary). |
| EventChannelError | Fehler beim Senden eines NotificationEvent über den broadcast::Sender. |
| DuplicateNotificationId | Versuch, eine Benachrichtigung mit einer bereits existierenden ID zu den aktiven Benachrichtigungen hinzuzufügen. |
| ReplacedNotificationNotFound | Die in replaces\_id angegebene Benachrichtigung wurde nicht gefunden. |

### **4.3.6. Detaillierte Implementierungsschritte und Algorithmen**

1. **NotificationCoreManager::add\_notification:**  
   * Validiere new\_notification (z.B. app\_name, summary nicht leer, id muss gesetzt sein). Bei Fehler: Err(NotificationCoreError::InvalidNotificationData).  
   * Erwirb Schreibsperre für active\_notifications.  
   * Wenn new\_notification.id bereits in active\_notifications existiert: Err(NotificationCoreError::DuplicateNotificationId).  
   * **Regelanwendung:** Rufe self.rules\_engine.process\_notification(\&new\_notification).await auf.  
     * Bei Err(rules\_error): Err(NotificationCoreError::from(rules\_error)).  
     * Bei Ok(RuleProcessingResult::Suppress(rule\_id)):  
       * Sende NotificationSuppressedByRule Event.  
       * Die Benachrichtigung wird nicht aktiv. Ggf. zur Historie hinzufügen, falls die Regel dies impliziert oder new\_notification.is\_persistent ist (abhängig von Designentscheidung).  
       * Ok(new\_notification.id) zurückgeben (die ID der ursprünglichen, nun unterdrückten Benachrichtigung).  
     * Bei Ok(RuleProcessingResult::Allow(mut processed\_notification)):  
       * processed\_notification.timestamp\_displayed \= Some(Utc::now()).  
       * **Ersetzungslogik:** Wenn processed\_notification.replaces\_id ein Some(id\_to\_replace) ist:  
         * Versuche, die Benachrichtigung mit id\_to\_replace aus active\_notifications zu entfernen.  
         * Wenn erfolgreich entfernt, sende NotificationClosed Event für id\_to\_replace mit CloseReason::Replaced.  
         * Wenn nicht gefunden: Err(NotificationCoreError::ReplacedNotificationNotFound(id\_to\_replace)).  
       * Füge processed\_notification.clone() zu active\_notifications hinzu (mit ihrer eigenen id).  
       * Sende NotificationAdded { notification: processed\_notification.clone(),... } Event.  
       * Ok(processed\_notification.id).  
2. **NotificationCoreManager::close\_notification:**  
   * Erwirb Schreibsperren für active\_notifications und history.  
   * Entferne Benachrichtigung mit id aus active\_notifications. Wenn nicht gefunden: Err(NotificationCoreError::NotificationNotFound(id)).  
   * Sei closed\_notification die entfernte Benachrichtigung.  
   * Wenn closed\_notification.is\_persistent oder reason dies nahelegt (z.B. DismissedByUser, aber nicht Expired wenn nicht persistent):  
     * Rufe history.write().await.add(closed\_notification.clone()) auf. Handle HistoryFull Fehler, falls add diesen zurückgeben kann (oder logge es).  
   * Sende NotificationClosed { notification\_id: id, app\_name: closed\_notification.app\_name, summary: closed\_notification.summary, reason,... } Event.  
   * Ok(()).  
3. **NotificationHistory::add (interne Methode von NotificationHistory):**  
   * Wenn self.notifications.len() \>= self.max\_size und self.max\_size \> 0:  
     * self.notifications.pop\_front() (entferne die älteste).  
   * self.notifications.push\_back(notification).  
4. **NotificationCoreManager::invoke\_action:**  
   * Erwirb Lesesperre für active\_notifications.  
   * Hole Benachrichtigung mit notification\_id. Wenn nicht gefunden: Err(NotificationCoreError::NotificationNotFound).  
   * Prüfe, ob die Aktion mit action\_key in notification.actions existiert. Wenn nicht: Err(NotificationCoreError::ActionNotFound).  
   * Sende NotificationActionInvoked { notification\_id, action\_key: action\_key.to\_string(),... } Event.  
   * Ok(()). (Die Domänenschicht löst nur das Event aus; die tatsächliche Aktionsausführung erfolgt in höheren Schichten oder der Anwendung selbst).

### **4.3.7. Überlegungen zur Nebenläufigkeit und Zustandssynchronisierung**

* active\_notifications und history (bzw. dessen interne VecDeque) benötigen tokio::sync::RwLock für Thread-sicheren Lese- und Schreibzugriff, da mehrere Tasks (z.B. durch D-Bus-Aufrufe oder interne Timer) gleichzeitig auf Benachrichtigungen zugreifen könnten.  
* Die rules\_engine wird als Arc\<NotificationRulesEngine\> übergeben, da sie von mehreren Aufrufen (z.B. für jede neue Benachrichtigung) nebenläufig genutzt werden kann und ihr Zustand (die Regeln) ebenfalls Thread-sicher sein muss.  
* Der broadcast::Sender für NotificationEvent ist inhärent Thread-sicher.13

## **4.4. Entwicklungsmodul: Priorisierung und Regel-Engine für Benachrichtigungen (domain::notifications\_rules)**

Dieses Modul implementiert die Logik zur dynamischen Verarbeitung von Benachrichtigungen basierend auf einem Satz von konfigurierbaren Regeln.

### **4.4.1. Detaillierte Verantwortlichkeiten und Ziele**

* **Verantwortlichkeiten:**  
  * Definition der Struktur von Benachrichtigungsregeln (NotificationRule), deren Bedingungen (RuleCondition) und Aktionen (RuleAction).  
  * Bereitstellung einer Engine (NotificationRulesEngine), die eingehende Benachrichtigungen anhand dieser Regeln bewertet.  
  * Ermöglichung von Modifikationen an Benachrichtigungen durch Regeln (z.B. Dringlichkeit ändern, Ton festlegen, Aktionen hinzufügen).  
  * Ermöglichung der Unterdrückung von Benachrichtigungen basierend auf Regelbedingungen.  
  * Interaktion mit domain::settings\_core (durch Empfang von SettingChangedEvents und Abfrage von Einstellungswerten), um kontextsensitive Regeln zu ermöglichen (z.B. "Nicht stören"-Modus, anwendungsspezifische Stummschaltungen).  
  * Laden und Verwalten von Regeldefinitionen. Diese können initial fest kodiert sein, sollten aber idealerweise aus einer externen Konfiguration (z.B. via SettingsProvider) geladen werden können, um Flexibilität zu gewährleisten.  
* **Ziele:**  
  * Schaffung einer flexiblen und erweiterbaren Logik zur dynamischen Anpassung des Benachrichtigungsverhaltens.  
  * Ermöglichung einer feingranularen Steuerung des Benachrichtigungsflusses durch den Benutzer (implizit über Systemeinstellungen) oder durch Systemadministratoren.  
  * Reduzierung von "Notification Fatigue" durch intelligente Filterung und Priorisierung.

Ein wichtiger Aspekt beim Design der Regel-Engine ist die Frage, ob Regeln fest im Code verankert oder datengetrieben (z.B. aus einer Konfigurationsdatei) sind. Ein datengetriebener Ansatz erhöht die Flexibilität und Wartbarkeit erheblich, da Regeln ohne Neukompilierung des Systems geändert oder hinzugefügt werden können. Dies erfordert, dass die Regelstrukturen (NotificationRule, RuleCondition, RuleAction) serde::Serialize und serde::Deserialize implementieren. Selbst wenn die erste Version mit fest kodierten Regeln startet, sollte das Design eine spätere Umstellung ermöglichen.

### **4.4.2. Entitäten und Wertobjekte**

Alle Typen sind in domain/src/notifications\_rules/types.rs zu definieren. Sie benötigen Debug, Clone, PartialEq und, für datengetriebene Regeln, serde::Serialize und serde::Deserialize.

* **RuleCondition (Enum, Wertobjekt)**  
  * **Zweck:** Definiert die Bedingungen, die erfüllt sein müssen, damit eine Regel ausgelöst wird.  
  * **Warum wertvoll:** Ermöglicht die flexible und kompositorische Definition von Kriterien für Regeln, von einfachen Vergleichen bis zu komplexen logischen Verknüpfungen.

| Variante | Assoziierte Daten | Beschreibung |
| :---- | :---- | :---- |
| AppNameIs | String | Der app\_name der Benachrichtigung entspricht exakt dem Wert (case-sensitive). |
| AppNameMatches | String (als Regex-Pattern zu interpretieren) | Der app\_name der Benachrichtigung entspricht dem regulären Ausdruck. |
| SummaryContains | String | Der summary der Benachrichtigung enthält den Text (case-insensitive). |
| SummaryMatches | String (Regex-Pattern) | Der summary der Benachrichtigung entspricht dem regulären Ausdruck. |
| BodyContains | String | Der body der Benachrichtigung (falls vorhanden) enthält den Text (case-insensitive). |
| UrgencyIs | NotificationUrgency | Die urgency der Benachrichtigung entspricht dem Wert. |
| CategoryIs | String | Die category der Benachrichtigung (falls vorhanden) entspricht exakt dem Wert. |
| HintExists | String (Schlüssel des Hints) | Ein bestimmter Schlüssel existiert in den hints der Benachrichtigung. |
| HintValueIs | (String (Hint-Schlüssel), SettingValue (erwarteter Wert)) | Ein bestimmter Hint-Schlüssel existiert und sein Wert entspricht dem SettingValue. |
| SettingIsTrue | SettingKey (Schlüssel zu einer Boolean-Einstellung) | Eine globale Systemeinstellung (aus SettingsCoreManager) ist auf true gesetzt. |
| SettingIsFalse | SettingKey (Schlüssel zu einer Boolean-Einstellung) | Eine globale Systemeinstellung ist auf false gesetzt. |
| SettingValueEquals | (SettingKey, SettingValue) | Eine globale Systemeinstellung hat exakt den spezifizierten Wert. |
| LogicalAnd | Vec\<RuleCondition\> | Alle Unterbedingungen in der Liste müssen wahr sein. |
| LogicalOr | Vec\<RuleCondition\> | Mindestens eine der Unterbedingungen in der Liste muss wahr sein. |
| LogicalNot | Box\<RuleCondition\> | Die umschlossene Unterbedingung muss falsch sein. |

* **RuleAction (Enum, Wertobjekt)**  
  * **Zweck:** Definiert die Aktionen, die ausgeführt werden, wenn die Bedingungen einer Regel erfüllt sind.  
  * **Warum wertvoll:** Beschreibt, wie eine Benachrichtigung als Reaktion auf eine Regel modifiziert oder behandelt wird.

| Variante | Assoziierte Daten | Beschreibung |
| :---- | :---- | :---- |
| SuppressNotification | \- | Unterdrückt die Benachrichtigung vollständig. Sie wird nicht aktiv und typischerweise auch nicht in der Historie gespeichert. |
| SetUrgency | NotificationUrgency | Ändert die urgency der Benachrichtigung auf den neuen Wert. |
| AddAction | NotificationAction | Fügt eine zusätzliche NotificationAction zur Liste der Aktionen der Benachrichtigung hinzu. |
| SetHint | (String (Hint-Schlüssel), SettingValue (Wert)) | Setzt oder überschreibt einen Wert in den hints der Benachrichtigung. |
| PlaySound | Option\<String\> (Sound-Datei/Name oder Event-Name) | Signalisiert, dass ein Ton abgespielt werden soll. None für einen Standard-Benachrichtigungston, Some(name) für einen spezifischen Ton. Die Implementierung des Abspielens erfolgt in der System- oder UI-Schicht. |
| MarkAsPersistent | bool | Setzt das is\_persistent-Flag der Benachrichtigung. |
| SetExpiration | Option\<i64\> (Millisekunden relativ zu jetzt) | Setzt oder ändert die Ablaufzeit der Benachrichtigung. None entfernt eine existierende Ablaufzeit. Ein positiver Wert gibt die Dauer in ms an. |
| LogMessage | (String (Level: "info", "warn", "debug"), String (Nachricht)) | Schreibt eine Nachricht ins System-Log (über das tracing-Framework). Nützlich für das Debugging von Regeln. |

* **NotificationRule (Struktur, Entität)**  
  * **Zweck:** Repräsentiert eine einzelne, vollständige Regel mit Bedingungen und Aktionen.  
  * **Warum wertvoll:** Die atomaren Bausteine der Regel-Engine. Eine Sammlung dieser Regeln definiert das Verhalten des Benachrichtigungssystems.

| Attribut | Typ | Sichtbarkeit | Beschreibung |
| :---- | :---- | :---- | :---- |
| id | String | pub | Eindeutige, menschenlesbare ID der Regel (z.B. "suppress-low-priority-chat", "urgentify-calendar-reminders"). |
| description | Option\<String\> | pub | Optionale, menschenlesbare Beschreibung des Zwecks der Regel. |
| conditions | RuleCondition | pub | Die Bedingung(en), die erfüllt sein müssen, damit die Regel angewendet wird. Oft eine LogicalAnd oder LogicalOr. |
| actions | Vec\<RuleAction\> | pub | Die Liste der Aktionen, die ausgeführt werden, wenn die conditions zutreffen. Die Reihenfolge kann relevant sein. |
| is\_enabled | bool | pub | Gibt an, ob die Regel aktiv ist und ausgewertet werden soll. Default: true. |
| priority | i32 | pub | Priorität der Regel. Regeln mit höherem Wert werden typischerweise früher ausgewertet. Default: 0\. |
| stop\_after | bool | pub | Wenn true und diese Regel zutrifft und Aktionen ausführt, werden keine weiteren (niedriger priorisierten) Regeln für diese Benachrichtigung mehr ausgewertet. Default: false. |

### **4.4.3. Öffentliche API des Moduls (NotificationRulesEngine)**

Definiert in domain/src/notifications\_rules/mod.rs.

Rust

// domain/src/notifications\_rules/mod.rs  
use crate::notifications\_core::types::{Notification, NotificationUrgency, SettingValue as NotificationSettingValue}; // SettingValue hier umbenannt zur Klarheit  
use crate::notifications\_rules::types::{NotificationRule, RuleCondition, RuleAction, NotificationAction as RuleNotificationAction};  
use crate::notifications\_rules::error::NotificationRulesError;  
use crate::settings\_core::{SettingsCoreManager, SettingChangedEvent, SettingKey, SettingValue};  
use std::sync::Arc;  
use tokio::sync::{RwLock, broadcast::Receiver as BroadcastReceiver}; // Receiver explizit benannt  
use tracing; // Für LogMessage Aktion

\#  
pub enum RuleProcessingResult {  
    Allow(Notification),  
    Suppress(String), // Enthält die ID der Regel, die zur Unterdrückung geführt hat  
}

pub struct NotificationRulesEngine {  
    rules: RwLock\<Vec\<NotificationRule\>\>,  
    settings\_manager: Arc\<SettingsCoreManager\>,  
    // settings\_update\_receiver: RwLock\<Option\<BroadcastReceiver\<SettingChangedEvent\>\>\>, // Für das Lauschen auf Einstellungsänderungen  
}

impl NotificationRulesEngine {  
    pub fn new(  
        settings\_manager: Arc\<SettingsCoreManager\>,  
        initial\_rules: Vec\<NotificationRule\>,  
        // mut settings\_event\_receiver: BroadcastReceiver\<SettingChangedEvent\> // Wird übergeben  
    ) \-\> Arc\<Self\> { // Gibt Arc\<Self\> zurück, um das Klonen für den Listener-Task zu erleichtern  
        let mut sorted\_rules \= initial\_rules;  
        sorted\_rules.sort\_by\_key(|r| \-r.priority); // Höchste Priorität zuerst

        let engine \= Arc::new(NotificationRulesEngine {  
            rules: RwLock::new(sorted\_rules),  
            settings\_manager,  
            // settings\_update\_receiver: RwLock::new(Some(settings\_event\_receiver)),  
        });

        // Hier könnte ein Task gestartet werden, der auf settings\_event\_receiver lauscht  
        // und self.handle\_setting\_changed aufruft.  
        // let engine\_clone \= Arc::clone(\&engine);  
        // tokio::spawn(async move {  
        //     if let Some(mut rx) \= engine\_clone.settings\_update\_receiver.write().await.take() {  
        //         while let Ok(event) \= rx.recv().await {  
        //             engine\_clone.handle\_setting\_changed(\&event).await;  
        //         }  
        //     }  
        // });

        engine  
    }

    pub async fn load\_rules(\&self, new\_rules: Vec\<NotificationRule\>) {  
        let mut rules\_guard \= self.rules.write().await;  
        \*rules\_guard \= new\_rules;  
        rules\_guard.sort\_by\_key(|r| \-r.priority); // Höchste Priorität zuerst  
        tracing::info\!("Notification rules reloaded. {} rules active.", rules\_guard.len());  
    }

    pub async fn process\_notification(  
        \&self,  
        notification: \&Notification,  
    ) \-\> Result\<RuleProcessingResult, NotificationRulesError\> {  
        let rules\_guard \= self.rules.read().await;  
        let mut current\_notification \= notification.clone();  
        let mut suppressed\_by\_rule\_id: Option\<String\> \= None;

        for rule in rules\_guard.iter().filter(|r| r.is\_enabled) {  
            if self.evaluate\_condition(\&rule.conditions, \&current\_notification, rule).await? {  
                tracing::debug\!("Rule '{}' matched for notification '{}'", rule.id, notification.summary);  
                for action in \&rule.actions {  
                    match self.apply\_action(action, \&mut current\_notification, rule).await? {  
                        RuleProcessingResult::Suppress(\_) \=\> {  
                            suppressed\_by\_rule\_id \= Some(rule.id.clone());  
                            break; // Aktion "Suppress" beendet Aktionsschleife für diese Regel  
                        }  
                        RuleProcessingResult::Allow(modified\_notification) \=\> {  
                            current\_notification \= modified\_notification;  
                        }  
                    }  
                }  
                if suppressed\_by\_rule\_id.is\_some() |  
| rule.stop\_after {  
                    break; // Regelverarbeitung für diese Benachrichtigung beenden  
                }  
            }  
        }

        if let Some(rule\_id) \= suppressed\_by\_rule\_id {  
            Ok(RuleProcessingResult::Suppress(rule\_id))  
        } else {  
            Ok(RuleProcessingResult::Allow(current\_notification))  
        }  
    }

    async fn evaluate\_condition(  
        \&self,  
        condition: \&RuleCondition,  
        notification: \&Notification,  
        rule: \&NotificationRule, // Für Kontext in Fehlermeldungen  
    ) \-\> Result\<bool, NotificationRulesError\> {  
        match condition {  
            RuleCondition::AppNameIs(name) \=\> Ok(\&notification.app\_name \== name),  
            RuleCondition::AppNameMatches(pattern) \=\> {  
                // Hier Regex-Implementierung, z.B. mit \`regex\` Crate  
                // Für dieses Beispiel: einfache Prüfung  
                match regex::Regex::new(pattern) {  
                    Ok(re) \=\> Ok(re.is\_match(\&notification.app\_name)),  
                    Err(e) \=\> Err(NotificationRulesError::ConditionEvaluationError{ rule\_id: Some(rule.id.clone()), message: format\!("Invalid regex pattern '{}': {}", pattern, e) })  
                }  
            }  
            RuleCondition::SummaryContains(text) \=\> Ok(notification.summary.to\_lowercase().contains(\&text.to\_lowercase())),  
            //... Implementierung für alle RuleCondition-Varianten...  
            RuleCondition::SettingIsTrue(key) \=\> {  
                match self.settings\_manager.get\_setting\_value(key).await {  
                    Ok(SettingValue::Boolean(b)) \=\> Ok(b),  
                    Ok(other\_type) \=\> {  
                        tracing::warn\!("Rule '{}' expected boolean for setting '{}', got {:?}", rule.id, key.as\_str(), other\_type);  
                        Ok(false) // Falscher Typ, als false bewerten  
                    }  
                    Err(SettingsCoreError::SettingNotFound{..}) | Err(SettingsCoreError::UnregisteredKey{..}) \=\> {  
                        tracing::debug\!("Rule '{}': Setting '{}' not found or unregistered, condition evaluates to false.", rule.id, key.as\_str());  
                        Ok(false) // Einstellung nicht gefunden, als false bewerten  
                    }  
                    Err(e) \=\> Err(NotificationRulesError::SettingsAccessError(e)) // Anderer Fehler beim Holen  
                }  
            }  
            RuleCondition::LogicalAnd(sub\_conditions) \=\> {  
                for sub\_cond in sub\_conditions {  
                    if\!self.evaluate\_condition(sub\_cond, notification, rule).await? {  
                        return Ok(false);  
                    }  
                }  
                Ok(true)  
            }  
            RuleCondition::LogicalOr(sub\_conditions) \=\> {  
                for sub\_cond in sub\_conditions {  
                    if self.evaluate\_condition(sub\_cond, notification, rule).await? {  
                        return Ok(true);  
                    }  
                }  
                Ok(false)  
            }  
            RuleCondition::LogicalNot(sub\_condition) \=\> {  
                Ok(\!self.evaluate\_condition(sub\_condition, notification, rule).await?)  
            }  
            // Standard-Fallback für nicht implementierte Bedingungen (sollte nicht passieren bei vollständiger Impl.)  
            \_ \=\> {  
                tracing::warn\!("Unimplemented condition met in rule '{}': {:?}", rule.id, condition);  
                Ok(false)  
            }  
        }  
    }

    async fn apply\_action(  
        \&self,  
        action: \&RuleAction,  
        notification: \&mut Notification,  
        rule: \&NotificationRule, // Für Kontext  
    ) \-\> Result\<RuleProcessingResult, NotificationRulesError\> {  
        tracing::debug\!("Applying action {:?} from rule '{}' to notification '{}'", action, rule.id, notification.summary);  
        match action {  
            RuleAction::SuppressNotification \=\> return Ok(RuleProcessingResult::Suppress(rule.id.clone())),  
            RuleAction::SetUrgency(new\_urgency) \=\> notification.urgency \= \*new\_urgency,  
            RuleAction::AddAction(new\_action) \=\> {  
                // Prüfen, ob Aktion mit gleichem Key schon existiert, um Duplikate zu vermeiden  
                if\!notification.actions.iter().any(|a| a.key \== new\_action.key) {  
                    notification.actions.push(new\_action.clone());  
                }  
            }  
            RuleAction::SetHint((key, value)) \=\> {  
                notification.hints.insert(key.clone(), value.clone().into\_setting\_value()); // Annahme: value ist hier ein Domänen-SettingValue  
            }  
            RuleAction::PlaySound(sound\_name\_opt) \=\> {  
                // Diese Aktion setzt typischerweise einen Hint, den die UI/Systemschicht interpretiert  
                let hint\_key \= "sound-name".to\_string();  
                if let Some(sound\_name) \= sound\_name\_opt {  
                    notification.hints.insert(hint\_key, NotificationSettingValue::String(sound\_name.clone()));  
                } else {  
                    // Signal für Standardton, z.B. spezieller Wert oder Entfernen des Hints  
                    notification.hints.remove(\&hint\_key);  
                }  
            }  
            RuleAction::MarkAsPersistent(is\_persistent) \=\> notification.is\_persistent \= \*is\_persistent,  
            RuleAction::SetExpiration(duration\_ms\_opt) \=\> {  
                if let Some(duration\_ms) \= duration\_ms\_opt {  
                    if \*duration\_ms \> 0 {  
                        notification.expires\_at \= Some(Utc::now() \+ chrono::Duration::milliseconds(\*duration\_ms));  
                    } else {  
                        notification.expires\_at \= None; // Negative oder Null-Dauer entfernt Expiration  
                    }  
                } else {  
                    notification.expires\_at \= None;  
                }  
            }  
            RuleAction::LogMessage((level, message)) \=\> {  
                let full\_message \= format\!(" {}", rule.id, message);  
                match level.as\_str() {  
                    "info" \=\> tracing::info\!("{}", full\_message),  
                    "warn" \=\> tracing::warn\!("{}", full\_message),  
                    "debug" \=\> tracing::debug\!("{}", full\_message),  
                    \_ \=\> tracing::trace\!("{}", full\_message), // Default zu trace  
                }  
            }  
        }  
        Ok(RuleProcessingResult::Allow(notification.clone()))  
    }

    // Diese Methode wird aufgerufen, wenn ein SettingChangedEvent empfangen wird.  
    // Sie könnte z.B. einen internen Cache für Settings aktualisieren, falls verwendet,  
    // oder Regeln neu bewerten, die von dieser Einstellung abhängen (komplexer).  
    // Für eine einfache Implementierung ohne Cache ist diese Methode ggf. leer  
    // oder löst nur einen Log-Eintrag aus.  
    pub async fn handle\_setting\_changed(\&self, event: \&SettingChangedEvent) {  
        tracing::debug\!("NotificationRulesEngine received SettingChangedEvent for key: {}", event.key.as\_str());  
        // Hier könnte Logik stehen, um z.B. interne Caches zu invalidieren,  
        // falls die Performance der direkten Abfrage des SettingsCoreManager ein Problem darstellt.  
        // Für die meisten Fälle sollte die direkte Abfrage bei Bedarf ausreichend sein.  
    }  
}

// Hilfskonvertierung für RuleAction::SetHint, falls SettingValue aus notifications\_rules::types  
// und settings\_core::types nicht identisch sind (sollten sie aber sein).  
// Hier wird angenommen, dass SettingValue aus settings\_core verwendet wird.  
trait IntoSettingValue {  
    fn into\_setting\_value(self) \-\> SettingValue;  
}  
impl IntoSettingValue for NotificationSettingValue { // Hier NotificationSettingValue ist Alias für settings\_core::SettingValue  
    fn into\_setting\_value(self) \-\> SettingValue {  
        self // Direkte Konvertierung, da Typen identisch sein sollten  
    }  
}

(Hinweis: Die regex-Crate müsste als Abhängigkeit hinzugefügt werden. Der Listener-Task für Einstellungsänderungen ist auskommentiert, da seine Implementierung von der genauen Architektur des Event-Handlings abhängt und den Rahmen sprengen könnte, aber das Prinzip ist wichtig.)

* **Tabelle: Methoden der NotificationRulesEngine**

| Methode | Signatur | Kurzbeschreibung |
| :---- | :---- | :---- |
| new | pub fn new(settings\_manager: Arc\<SettingsCoreManager\>, initial\_rules: Vec\<NotificationRule\>/\*, settings\_event\_receiver: BroadcastReceiver\<SettingChangedEvent\>\*/) \-\> Arc\<Self\> | Konstruktor. Lädt initiale Regeln, sortiert sie nach Priorität. Speichert Referenz auf SettingsCoreManager. Startet optional einen Task, um auf SettingChangedEvents zu lauschen. Gibt Arc\<Self\> zurück. |
| load\_rules | pub async fn load\_rules(\&self, new\_rules: Vec\<NotificationRule\>) | Lädt einen neuen Satz von Regeln, ersetzt die alten und sortiert sie neu nach Priorität. |
| process\_notification | pub async fn process\_notification(\&self, notification: \&Notification) \-\> Result\<RuleProcessingResult, NotificationRulesError\> | Verarbeitet eine eingehende Benachrichtigung anhand der geladenen, aktivierten Regeln. Gibt entweder eine (potenziell modifizierte) Benachrichtigung (Allow) oder ein Signal zur Unterdrückung (Suppress) mit der verantwortlichen Regel-ID zurück. |
| handle\_setting\_changed | pub async fn handle\_setting\_changed(\&self, event: \&SettingChangedEvent) | Wird (intern, z.B. durch einen dedizierten Task) aufgerufen, wenn sich eine für Regeln relevante Systemeinstellung ändert. Ermöglicht der Engine, ihren Zustand oder ihr Verhalten anzupassen (z.B. Cache-Invalidierung). |

Die Entscheidung, SettingValue aus settings\_core auch in den RuleCondition und RuleAction zu verwenden, vereinfacht die Typisierung und vermeidet unnötige Konvertierungen.

### **4.4.4. Fehlerbehandlung (NotificationRulesError)**

Definiert in domain/src/notifications\_rules/error.rs mit thiserror.

Rust

// domain/src/notifications\_rules/error.rs  
use thiserror::Error;  
use crate::settings\_core::error::SettingsCoreError; // Für Fehler beim Zugriff auf Settings

\#  
pub enum NotificationRulesError {  
    \#  
    InvalidRuleDefinition {  
        rule\_id: Option\<String\>,  
        message: String,  
    },

    \#\[error("Failed to evaluate condition for rule '{rule\_id:?}': {message}")\]  
    ConditionEvaluationError {  
        rule\_id: Option\<String\>,  
        message: String,  
    },

    \#\[error("Failed to apply action for rule '{rule\_id:?}': {message}")\]  
    ActionApplicationError {  
        rule\_id: Option\<String\>,  
        message: String,  
    },

    \#\[error("Error accessing settings for rule evaluation: {source}")\]  
    SettingsAccessError{  
        \#\[from\] // Direkte Konvertierung von SettingsCoreError  
        source: SettingsCoreError  
    },

    \# // Wird intern verwendet, falls Regeln auf andere verweisen  
    RuleNotFound(String),  
}

* **Tabelle: NotificationRulesError Varianten**

| Variante | Beschreibung |
| :---- | :---- |
| InvalidRuleDefinition | Eine geladene Regel ist ungültig (z.B. fehlerhaftes Regex-Pattern in AppNameMatches, widersprüchliche Bedingungen, unbekannte Aktionstypen). |
| ConditionEvaluationError | Ein Fehler trat während der Auswertung einer Bedingung auf (z.B. Regex-Kompilierungsfehler, interner Logikfehler). |
| ActionApplicationError | Ein Fehler trat während der Anwendung einer Aktion auf (z.B. ungültige Parameter für eine Aktion). |
| SettingsAccessError | Fehler beim Zugriff auf SettingsCoreManager für die Auswertung von Bedingungen, die auf Systemeinstellungen basieren. Nutzt \#\[from\]. |
| RuleNotFound | Eine referenzierte Regel-ID (z.B. in einer komplexen Regelstruktur) existiert nicht. |

### **4.4.5. Detaillierte Implementierungsschritte und Algorithmen**

1. **Initialisierung (NotificationRulesEngine::new):**  
   * Speichere den Arc\<SettingsCoreManager\>.  
   * Lade die initial\_rules.  
   * Sortiere die Regeln nach priority (absteigend, d.h. höhere numerische Werte zuerst) und dann ggf. nach id für deterministische Reihenfolge bei gleicher Priorität.  
   * **Abonnement von Einstellungsänderungen:** Es ist entscheidend, dass die Regel-Engine auf Änderungen von Systemeinstellungen reagieren kann, die in RuleConditions verwendet werden (z.B. "Nicht stören"-Modus).  
     * Der NotificationRulesEngine sollte beim Erstellen einen broadcast::Receiver\<SettingChangedEvent\> vom SettingsCoreManager erhalten (oder der SettingsCoreManager registriert die Engine als Listener).  
     * Ein dedizierter tokio::task sollte gestartet werden, der diesen Receiver konsumiert. Bei Empfang eines SettingChangedEvent ruft dieser Task engine.handle\_setting\_changed(\&event).await auf.  
     * handle\_setting\_changed kann dann z.B. einen internen Cache von oft benötigten Einstellungswerten invalidieren oder aktualisieren, um zu vermeiden, dass für jede Regelauswertung der SettingsCoreManager abgefragt werden muss (Performance-Optimierung, falls nötig). Für den Anfang kann es ausreichen, dass evaluate\_condition immer live den SettingsCoreManager abfragt.  
2. **NotificationRulesEngine::process\_notification:**  
   * Erwirb eine Lesesperre auf self.rules.  
   * Klone die eingehende notification, um Modifikationen zu ermöglichen (current\_notification).  
   * Iteriere durch die sortierten, aktivierten (rule.is\_enabled) Regeln.  
   * Für jede Regel:  
     * Evaluiere rule.conditions rekursiv mittels self.evaluate\_condition(\&rule.conditions, \&current\_notification, \&rule).await.  
     * Wenn die Bedingungen erfüllt sind (true):  
       * Iteriere durch rule.actions.  
       * Wende jede Aktion auf current\_notification an mittels self.apply\_action(action, \&mut current\_notification, \&rule).await.  
       * Wenn eine Aktion RuleProcessingResult::Suppress zurückgibt (z.B. RuleAction::SuppressNotification), speichere die rule.id und brich die Verarbeitung der Aktionen *dieser Regel* ab.  
       * Wenn RuleProcessingResult::Allow(modified\_notification) zurückgegeben wird, aktualisiere current\_notification \= modified\_notification.  
       * Wenn suppressed\_by\_rule\_id gesetzt wurde oder rule.stop\_after \== true ist, brich die Iteration über *weitere Regeln* ab.  
   * Wenn am Ende suppressed\_by\_rule\_id gesetzt ist, gib Ok(RuleProcessingResult::Suppress(rule\_id)) zurück.  
   * Andernfalls gib Ok(RuleProcessingResult::Allow(current\_notification)) zurück.  
3. **NotificationRulesEngine::evaluate\_condition (rekursiv):**  
   * Implementiere die Logik für jede RuleCondition-Variante:  
     * Einfache Vergleiche (AppNameIs, SummaryContains, UrgencyIs, etc.) sind direkte Vergleiche der Felder der notification.  
     * Regex-basierte Vergleiche (AppNameMatches, SummaryMatches) verwenden die regex-Crate. Fehler bei der Regex-Kompilierung (sollten idealerweise beim Laden der Regeln abgefangen werden) führen zu Err(NotificationRulesError::ConditionEvaluationError).  
     * HintExists, HintValueIs: Zugriff auf notification.hints.  
     * SettingIsTrue, SettingIsFalse, SettingValueEquals: Asynchroner Aufruf von self.settings\_manager.get\_setting\_value(\&key).await.  
       * Fehler wie SettingsCoreError::SettingNotFound oder UnregisteredKey sollten die Bedingung typischerweise als false bewerten lassen, anstatt einen harten Fehler in der Regel-Engine auszulösen, um die Robustheit zu erhöhen. Ein Log-Eintrag (Warnung oder Debug) ist hier angebracht. Andere SettingsCoreError (z.B. PersistenceError) sollten als Err(NotificationRulesError::SettingsAccessError) propagiert werden.  
     * LogicalAnd: Gibt true zurück, wenn alle Unterbedingungen true sind (Kurzschlussauswertung).  
     * LogicalOr: Gibt true zurück, wenn mindestens eine Unterbedingung true ist (Kurzschlussauswertung).  
     * LogicalNot: Negiert das Ergebnis der Unterbedingung.  
   * Alle Pfade müssen Result\<bool, NotificationRulesError\> zurückgeben.  
4. **NotificationRulesEngine::apply\_action:**  
   * Implementiere die Logik für jede RuleAction-Variante.  
   * Die meisten Aktionen modifizieren die übergebene \&mut Notification direkt (z.B. SetUrgency, AddAction, SetHint, MarkAsPersistent, SetExpiration).  
   * SuppressNotification gibt Ok(RuleProcessingResult::Suppress(...)) zurück.  
   * PlaySound könnte einen speziellen Hint setzen (z.B. sound-event: "message-new-instant"), den die UI-Schicht interpretiert.  
   * LogMessage verwendet das tracing-Makro (z.B. tracing::info\!).  
   * Alle Pfade geben Result\<RuleProcessingResult, NotificationRulesError\> zurück (meist Ok(RuleProcessingResult::Allow(notification.clone())) nach Modifikation).

### **4.4.6. Erweiterbarkeit und Konfiguration der Regeln**

* Um Regeln dynamisch (z.B. aus Konfigurationsdateien) laden zu können, müssen NotificationRule und alle eingebetteten Typen (RuleCondition, RuleAction) serde::Serialize und serde::Deserialize implementieren.  
* Die NotificationRulesEngine könnte eine Methode async fn load\_rules\_from\_provider(\&self, settings\_provider: Arc\<dyn SettingsProvider\>, config\_key: \&SettingKey) anbieten. Diese Methode würde:  
  1. Den settings\_provider verwenden, um eine serialisierte Regelmenge (z.B. als JSON-String oder eine Liste von serialisierten Regelobjekten) unter config\_key zu laden.  
  2. Die geladenen Daten deserialisieren in Vec\<NotificationRule\>.  
  3. Diese neuen Regeln über self.load\_rules(...) aktivieren.  
* Das Format der Serialisierung (z.B. JSON, YAML, TOML) muss sorgfältig entworfen werden, um sowohl menschenlesbar als auch maschinell verarbeitbar zu sein. Validierungsschemata (z.B. JSON Schema) können helfen, die Korrektheit der Regeldefinitionen sicherzustellen, bevor sie geladen werden.  
* Die Fehlerbehandlung beim Laden und Deserialisieren von Regeln muss robust sein (InvalidRuleDefinition).

Diese detaillierte Ausarbeitung der Einstellungs- und Benachrichtigungs-Subsysteme vervollständigt die Spezifikation der Domänenschicht und legt eine solide Grundlage für deren Implementierung. Die Betonung von klar definierten Schnittstellen, Typsicherheit, Fehlerbehandlung und Entkopplung durch Events und Abstraktionen ist entscheidend für die Entwicklung einer modernen, wartbaren und erweiterbaren Desktop-Umgebung.


---


**Struktur der Domänenschicht-Spezifikation:**

Die Domänenschicht (`novade-domain` Crate) wird in folgende logische Hauptmodule unterteilt, die teilweise bereits in Ihren Dokumenten skizziert wurden. Ich werde diese Struktur beibehalten und verfeinern:

1. **`domain::theming`**: Logik der Theming-Engine. (Basierend auf)
2. **`domain::workspaces`**: Umfassende Verwaltungslogik für Arbeitsbereiche. (Basierend auf)
    - `workspaces::core`
    - `workspaces::assignment`
    - `workspaces::manager`
    - `workspaces::config`
3. **`domain::user_centric_services`**: KI-Interaktion und Benachrichtigungsmanagement. (Basierend auf)
    - `user_centric_services::ai_interaction`
    - `user_centric_services::notifications_core`
4. **`domain::notifications_rules`**: Logik zur dynamischen Verarbeitung von Benachrichtigungen basierend auf Regeln. (Basierend auf der Gesamtspezifikation und )
5. **`domain::global_settings_and_state_management`**: Repräsentation und Logik globaler Desktop-Einstellungen. (Basierend auf der Gesamtspezifikation und)
    - `global_settings::types` (Definition der Einstellungsstrukturen)
    - `global_settings::service` (Der `GlobalSettingsService` Trait und Implementierung)
    - `global_settings::paths` (Der `SettingPath` Enum)
    - `global_settings::persistence_iface` (Trait für die Persistenz, Interaktion mit `core::config`)
6. **`domain::window_management_policy`**: High-Level-Regeln und Richtlinien für Fensterplatzierung, Tiling etc. (Basierend auf der Gesamtspezifikation)
7. **`domain::common_events`**: Definition von Domänen-übergreifenden Events, die nicht spezifisch einem einzelnen Service zugeordnet sind, oder als gemeinsame Payloads dienen.
8. **`domain::shared_types`**: Wiederverwendbare, domänenspezifische Typen, die von mehreren Domänenmodulen genutzt werden, aber nicht in `core::types` gehören (z.B. spezifische IDs, Status-Enums).

**Allgemeine Entwicklungsrichtlinien für die Domänenschicht (Wiederholung und Erweiterung):**

- **UI-Unabhängigkeit:** Die Domänenschicht darf keine direkten Abhängigkeiten zu UI-Toolkits (GTK4) oder spezifischen UI-Implementierungen haben.
- **Systemunabhängigkeit:** Keine direkte Abhängigkeit von Systemdetails wie D-Bus oder Wayland-Protokollen. Diese werden von der Systemschicht gehandhabt.
- **Fokus auf Geschäftslogik:** Enthält die Kernregeln und -prozesse der Desktop-Umgebung.
- **API-Design:** Öffentliche Schnittstellen werden primär über Traits definiert, um Testbarkeit (Mocking) und lose Kopplung zu fördern.
- **Zustandsverwaltung:** Veränderliche Zustände innerhalb von Services werden threadsicher gekapselt (z.B. `Arc<Mutex<...>>` oder `Arc<RwLock<...>>`).
- **Asynchronität:** `async/await` und `async_trait` werden für Operationen verwendet, die potenziell blockieren könnten (z.B. Warten auf Ergebnisse von der Systemschicht, komplexe Berechnungen, die ausgelagert werden können). Die primäre Runtime (z.B. `tokio`) wird von der Anwendung bereitgestellt, die die Domänenschicht nutzt.
- **Events:** Ein klar definierter Event-Mechanismus (z.B. `tokio::sync::broadcast` oder ein dedizierter Event-Bus-Trait) wird für die Kommunikation von Zustandsänderungen zwischen Domänenmodulen und an höhere Schichten verwendet.
- **Fehlerbehandlung:** Konsequente Nutzung von `thiserror` für modulspezifische Fehler-Enums. Fehler aus der Kernschicht werden ggf. gewrappt (`#[from]`).
- **Validierung:** Eingabedaten und Zustandsänderungen werden aktiv validiert.
- **Serialisierung:** `serde` wird für Datenstrukturen verwendet, die persistiert oder über Schnittstellen ausgetauscht werden müssen.
- **Abhängigkeit zur Kernschicht:** Die Domänenschicht nutzt ausschließlich die Kernschicht (`core::*`) für fundamentale Typen, Fehlerbasis, Logging und Konfigurationsprimitive.

---

## Ultra-Feinspezifikation: Domänenschicht (`novade-domain` Crate)

### Modul 1: `domain::theming`

**Zweck:** Logik des Erscheinungsbilds (Theming), Verwaltung von Design-Tokens, Interpretation von Theme-Definitionen, dynamische Theme-Wechsel.

**Bestehende Spezifikation:** wird als Basis verwendet und hier integriert/verfeinert.

#### Untermodul: `domain::theming::types`

**Datei:** `src/theming/types.rs`

- **Struct `TokenIdentifier`**: Wie in spezifiziert.
    - **Ableitungen zusätzlich:** `Ord, PartialOrd` (für konsistente Sortierung in HashMaps/Sets, falls Schlüssel Iteriert werden).
- **Enum `TokenValue`**: Wie in spezifiziert.
    - **Ableitungen zusätzlich:** `Eq, Hash` (falls `TokenIdentifier` es ist und für `Color`, `Dimension` etc. eine Hash-Implementierung sinnvoll ist – für String-basierte Werte ist dies der Fall).
- **Struct `RawToken`**: Wie in spezifiziert.
- **Typalias `TokenSet`**: Wie in spezifiziert.
- **Struct `ThemeIdentifier`**: Wie in spezifiziert.
    - **Ableitungen zusätzlich:** `Ord, PartialOrd`.
- **Enum `ColorSchemeType`**: Wie in spezifiziert.
    - **Ableitungen zusätzlich:** `Eq, Hash`.
- **Struct `AccentColor`**: Wie in spezifiziert.
    - **Ableitungen zusätzlich:** `Eq, Hash` (falls `name` `Option<String>` ist und `value` `String`).
- **Struct `ThemeVariantDefinition`**: Wie in spezifiziert.
- **Struct `ThemeDefinition`**: Wie in spezifiziert.
- **Struct `AppliedThemeState`**: Wie in spezifiziert.
    - **Felder:**
        - `pub theme_id: ThemeIdentifier`
        - `pub color_scheme: ColorSchemeType`
        - `pub active_accent_color: Option<AccentColor>`
        - `pub resolved_tokens: std::collections::HashMap<TokenIdentifier, String>`
    - **Invarianten:** `resolved_tokens` darf keine Referenzen enthalten, alle Werte sind finale CSS-Strings.
- **Struct `ThemingConfiguration`**: Wie in spezifiziert.

#### Untermodul: `domain::theming::errors`

**Datei:** `src/theming/errors.rs`

- **Enum `ThemingError`**: Wie in spezifiziert.
    - **Varianten (Beispiele, konsolidiert):**
        - `TokenFileParseError { path: PathBuf, #[source] source: serde_json::Error }`
        - `TokenFileIoError { path: PathBuf, #[source] source: std::io::Error }`
        - `InvalidTokenData { path: PathBuf, message: String }`
        - `CyclicTokenReference { token_id: TokenIdentifier, cycle_path: Vec<TokenIdentifier> }`
        - `ThemeFileLoadError { theme_id: ThemeIdentifier, path: PathBuf, #[source] source: serde_json::Error }`
        - `ThemeFileIoError { theme_id: ThemeIdentifier, path: PathBuf, #[source] source: std::io::Error }`
        - `InvalidThemeData { theme_id: ThemeIdentifier, path: PathBuf, message: String }`
        - `ThemeNotFound { theme_id: ThemeIdentifier }`
        - `MissingTokenReference { referencing_token_id: TokenIdentifier, target_token_id: TokenIdentifier }`
        - `MaxReferenceDepthExceeded { token_id: TokenIdentifier, depth: u8 }`
        - `ThemeApplicationError { message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> }`
        - `FallbackThemeLoadError { #[source] source: Box<dyn std::error::Error + Send + Sync + 'static> }`
        - `InitialConfigurationError(String)`
        - `InternalStateError(String)`
        - `EventSubscriptionError(String)`
        - `AccentColorProcessingError { theme_id: ThemeIdentifier, accent_value: String, message: String }` (Neu, für Fehler bei Akzentfarben-Anwendung)
        - `TokenResolutionError { token_id: TokenIdentifier, message: String }` (Allgemeiner Fehler während der Auflösung)

#### Untermodul: `domain::theming::logic` (oder `domain::theming::engine_internal`)

**Datei:** `src/theming/logic.rs` (oder aufgeteilt in `token_loader.rs`, `theme_loader.rs`, `token_resolver.rs` etc.)

- **Kernlogik und Geschäftsregeln** wie in spezifiziert:
    - Laden, Parsen, Validieren von Token- (_.tokens.json) und Theme-Definitionen (_.theme.json).
        - **Funktion:** `pub(crate) async fn load_and_validate_token_files(paths: &[PathBuf]) -> Result<TokenSet, ThemingError>`
        - **Funktion:** `pub(crate) async fn load_and_validate_theme_files(paths: &[PathBuf], global_tokens: &TokenSet) -> Result<Vec<ThemeDefinition>, ThemingError>`
    - Token Resolution Pipeline:
        - **Funktion:** `pub(crate) fn resolve_tokens_for_config(config: &ThemingConfiguration, theme_def: &ThemeDefinition, global_tokens: &TokenSet, max_depth: u8) -> Result<std::collections::HashMap<TokenIdentifier, String>, ThemingError>`
            - Schritt 1: Basissatz (Globale Tokens + Theme Base Tokens)
            - Schritt 2: Varianten-spezifische Tokens anwenden
            - Schritt 3: Akzentfarben-Logik anwenden (Direkte Ersetzung spezifischer Tokens, z.B. `token.system.accent.primary`, `token.system.accent.secondary`)
            - Schritt 4: Benutzerdefinierte globale Token-Overrides anwenden
            - Schritt 5: Rekursive Auflösung von `TokenValue::Reference` mit Zyklenerkennung und Tiefenbegrenzung.
            - Schritt 6: Finale Wertkonvertierung zu `String`.
    - Caching-Logik für `AppliedThemeState`.
        - **Typ:** `CacheKey(ThemeIdentifier, ColorSchemeType, Option<AccentColorHash>, UserOverridesHash)`
        - Cache-Struktur: `std::collections::HashMap<CacheKey, AppliedThemeState>`
    - Laden des Fallback-Themes (siehe).
        - **Funktion:** `pub(crate) fn load_fallback_theme_definition() -> Result<(ThemeDefinition, TokenSet), ThemingError>` (aus eingebetteten Strings)
        - **Funktion:** `pub(crate) fn generate_fallback_applied_state() -> AppliedThemeState`

#### Öffentliche API: `ThemingEngine` Service

**Datei:** `src/theming/mod.rs` (oder `src/theming/api.rs` oder `src/theming/service.rs`)

- **Struct `ThemingEngine`**: Wie in spezifiziert, verwendet `Arc<Mutex<ThemingEngineInternalState>>`.
    - **`ThemingEngineInternalState` Felder:**
        - `current_config: ThemingConfiguration`
        - `available_themes: Vec<ThemeDefinition>`
        - `global_raw_tokens: TokenSet`
        - `applied_state: AppliedThemeState`
        - `theme_load_paths: Vec<PathBuf>`
        - `token_load_paths: Vec<PathBuf>`
        - `resolved_state_cache: std::collections::HashMap<(ThemeIdentifier, ColorSchemeType, Option<String>, u64), AppliedThemeState>` (Cache-Schlüssel muss Hashable sein; Option&lt;AccentColor> könnte zu Option&lt;String> für den Farbwert vereinfacht werden, UserOverrides zu einem Hash).
        - `event_sender: tokio::sync::broadcast::Sender<ThemeChangedEvent>` (anstelle von `mpsc`)
- **Methoden der `ThemingEngine`**:
    - `pub async fn new(initial_config: ThemingConfiguration, theme_load_paths: Vec<PathBuf>, token_load_paths: Vec<PathBuf>, broadcast_capacity: usize) -> Result<Self, ThemingError>`
        - Initialisiert `event_sender = tokio::sync::broadcast::channel(broadcast_capacity).0;`
        - Lädt Themes und Tokens asynchron.
        - Berechnet initialen `applied_state` oder Fallback.
    - `pub async fn get_current_theme_state(&self) -> AppliedThemeState` (gibt Klon von `applied_state` zurück, kein `Result` wenn interner Zustand immer gültig ist).
    - `pub async fn get_available_themes(&self) -> Vec<ThemeDefinition>` (gibt Klon zurück).
    - `pub async fn get_current_configuration(&self) -> ThemingConfiguration` (gibt Klon zurück).
    - `pub async fn update_configuration(&self, new_config: ThemingConfiguration) -> Result<(), ThemingError>`
        - Berechnet neuen `applied_state`.
        - Wenn geändert, `self.event_sender.send(ThemeChangedEvent { ... }).map_err(...)`. Ignoriere Fehler, wenn keine Subscriber da sind (`Ok(_)`).
    - `pub async fn reload_themes_and_tokens(&self) -> Result<(), ThemingError>`
        - Lädt Themes/Tokens neu.
        - Wendet `current_config` neu an, aktualisiert `applied_state`.
        - Sendet Event, falls geändert.
        - Invalidiert Cache.
    - `pub fn subscribe_to_theme_changes(&self) -> tokio::sync::broadcast::Receiver<ThemeChangedEvent>`
        - Gibt `self.event_sender.subscribe()` zurück.
- **Event `ThemeChangedEvent`**: Wie in spezifiziert.
    - **Payload:** `pub new_state: AppliedThemeState`
    - **Publisher:** `ThemingEngine`
    - **Subscriber:** `ui::theming_gtk`, andere UI-Komponenten, `domain::global_settings_service` (falls Theming von globalen Einstellungen abhängt).

#### Implementierungsschritte `domain::theming`

1. **Dateistruktur anlegen:** Gemäß.
2. **`types.rs` implementieren:** Alle Datenstrukturen mit `serde`-Attributen und Ableitungen.
3. **`errors.rs` implementieren:** `ThemingError`-Enum mit `thiserror`.
4. **`logic.rs` (oder Submodule) implementieren:**
    - Token-/Theme-Lade- und Validierungsfunktionen (asynchron).
    - Token Resolution Pipeline (synchron, da CPU-gebunden nach dem Laden).
    - Fallback-Theme-Logik.
    - Caching-Logik.
5. **`ThemingEngine`-Service (`mod.rs` oder `service.rs`) implementieren:**
    - `ThemingEngineInternalState` und `ThemingEngine` Strukturen.
    - `new()`-Konstruktor (asynchron).
    - Alle öffentlichen API-Methoden (asynchron, wo sinnvoll).
    - Event-Versand mit `tokio::sync::broadcast`.
6. **Unit-Tests:**
    - Für alle Datenstrukturen: Serialisierung/Deserialisierung.
    - Für `ThemingError`: `Display`-Implementierung und `source()`-Verhalten.
    - Für Lade-/Validierungslogik: Gültige/ungültige Dateien, Zyklen, fehlende Referenzen.
    - Für Token Resolution Pipeline: Verschiedene Szenarien (Basis, Varianten, Overrides, Akzente, komplexe Referenzen, Fehlerfälle).
    - Für `ThemingEngine`: API-Methodenverhalten, Zustandsänderungen, Event-Auslösung, Cache-Verhalten, Thread-Sicherheit (konzeptionell, durch korrekte Mutex-Nutzung).

---

### Modul 2: `domain::workspaces`

**Zweck:** Umfassende Verwaltungslogik für Arbeitsbereiche ("Spaces").

**Bestehende Spezifikation:** wird als Basis verwendet.

#### Untermodul: `domain::workspaces::core`

**Datei:** `src/workspaces/core/types.rs` (konsolidiert Typen hier)

- **Typalias `WorkspaceId`**: `pub type WorkspaceId = uuid::Uuid;`
- **Struct `WindowIdentifier`**: `#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)] pub struct WindowIdentifier(String);`
    - Methoden: `new(id: String) -> Result<Self, WorkspaceCoreError>`, `as_str()`. `From<String>` für einfache Konvertierung (kann Validierung in `new` haben). `Display`-Implementierung.
- **Enum `WorkspaceLayoutType`**: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)] pub enum WorkspaceLayoutType { #[default] Floating, TilingHorizontal, TilingVertical, Maximized }`
- **Struct `Workspace`**: Wie in spezifiziert.
    - **Felder:** `id: WorkspaceId`, `name: String`, `persistent_id: Option<String>`, `layout_type: WorkspaceLayoutType`, `window_ids: HashSet<WindowIdentifier>`, `created_at: chrono::DateTime<chrono::Utc>`.
    - **Ableitungen:** `#[derive(Debug, Clone, Serialize, Deserialize)]` (`PartialEq` ggf. manuell wegen `HashSet`).
    - **Methoden:** `new(...) -> Result<Self, WorkspaceCoreError>`, `id()`, `name()`, `rename(...) -> Result<...>`, `layout_type()`, `set_layout_type(...)`, `add_window_id(&mut self, ...)` (pub(crate)), `remove_window_id(&mut self, ...)` (pub(crate)), `window_ids()`, `persistent_id()`, `set_persistent_id(...) -> Result<...>`, `created_at()`.

**Datei:** `src/workspaces/core/event_data.rs`

- **Event-Payload-Strukturen**: `WorkspaceRenamedData`, `WorkspaceLayoutChangedData`, `WindowAddedToWorkspaceData`, `WindowRemovedFromWorkspaceData`, `WorkspacePersistentIdChangedData` - wie in spezifiziert. Alle mit `#[derive(Debug, Clone, Serialize, Deserialize)]`.

**Datei:** `src/workspaces/core/errors.rs`

- **Konstante `MAX_WORKSPACE_NAME_LENGTH`**: `pub const MAX_WORKSPACE_NAME_LENGTH: usize = 64;`
- **Enum `WorkspaceCoreError`**: Wie in spezifiziert.
    - **Varianten (konsolidiert):**
        - `InvalidName(String)`
        - `NameCannotBeEmpty`
        - `NameTooLong { name: String, max_len: usize, actual_len: usize }`
        - `InvalidPersistentId(String)` (z.B. leer oder ungültige Zeichen)
        - `Internal { context: String }`
        - `WindowIdentifierEmpty` (Für `WindowIdentifier::new`)

#### Untermodul: `domain::workspaces::assignment`

**Datei:** `src/workspaces/assignment/mod.rs`

- **Öffentliche API-Funktionen**: Operieren auf `&mut std::collections::HashMap<WorkspaceId, Workspace>`. Wie in spezifiziert.
    - `pub fn assign_window_to_workspace(workspaces: &mut HashMap<WorkspaceId, Workspace>, target_workspace_id: WorkspaceId, window_id: &WindowIdentifier, ensure_unique_assignment: bool) -> Result<(), WindowAssignmentError>`
    - `pub fn remove_window_from_workspace(workspaces: &mut HashMap<WorkspaceId, Workspace>, source_workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<bool, WindowAssignmentError>`
    - `pub fn move_window_to_workspace(workspaces: &mut HashMap<WorkspaceId, Workspace>, source_workspace_id: WorkspaceId, target_workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WindowAssignmentError>`
    - `pub fn find_workspace_for_window(workspaces: &HashMap<WorkspaceId, Workspace>, window_id: &WindowIdentifier) -> Option<WorkspaceId>`

**Datei:** `src/workspaces/assignment/errors.rs`

- **Enum `WindowAssignmentError`**: Wie in spezifiziert.
    - **Varianten (konsolidiert):**
        - `WorkspaceNotFound(WorkspaceId)`
        - `WindowAlreadyAssigned { workspace_id: WorkspaceId, window_id: WindowIdentifier }`
        - `WindowNotAssignedToWorkspace { workspace_id: WorkspaceId, window_id: WindowIdentifier }`
        - `SourceWorkspaceNotFound(WorkspaceId)`
        - `TargetWorkspaceNotFound(WorkspaceId)`
        - `WindowNotOnSourceWorkspace { workspace_id: WorkspaceId, window_id: WindowIdentifier }`
        - `CannotMoveToSameWorkspace { workspace_id: WorkspaceId, window_id: WindowIdentifier }`
        - `RuleViolation { reason: String, window_id: Option<WindowIdentifier>, target_workspace_id: Option<WorkspaceId> }`
        - `Internal { context: String }`

#### Untermodul: `domain::workspaces::manager`

**Datei:** `src/workspaces/manager/events.rs`

- **Enum `WorkspaceEvent`**: Wie in spezifiziert. Alle Payloads mit `#[derive(Debug, Clone, Serialize, Deserialize)]`.
    - Payload-Strukturen aus `workspaces::core::event_data` werden hier verwendet.

**Datei:** `src/workspaces/manager/errors.rs`

- **Enum `WorkspaceManagerError`**: Wie in spezifiziert.
    - **Varianten (konsolidiert):**
        - `WorkspaceNotFound(WorkspaceId)`
        - `CannotDeleteLastWorkspace`
        - `DeleteRequiresFallbackForWindows { workspace_id: WorkspaceId, window_count: usize }`
        - `FallbackWorkspaceNotFound(WorkspaceId)`
        - `CoreError { #[from] source: crate::domain::workspaces::core::errors::WorkspaceCoreError }`
        - `AssignmentError { #[from] source: crate::domain::workspaces::assignment::errors::WindowAssignmentError }`
        - `ConfigError { #[from] source: crate::domain::workspaces::config::errors::WorkspaceConfigError }`
        - `SetActiveWorkspaceNotFound(WorkspaceId)`
        - `NoActiveWorkspace`
        - `DuplicatePersistentId(String)`
        - `Internal { context: String }`

**Datei:** `src/workspaces/manager/mod.rs` (oder `service.rs`)

- **Trait `EventPublisher<E>` (Beispiel, falls nicht global):** `pub trait EventPublisher<E: Clone + Send + 'static>: Send + Sync { fn publish(&self, event: E); }`
- **Struct `WorkspaceManager`**: Wie in spezifiziert.
    - **Felder:**
        - `workspaces: HashMap<WorkspaceId, Workspace>`
        - `active_workspace_id: Option<WorkspaceId>`
        - `ordered_workspace_ids: Vec<WorkspaceId>`
        - `next_workspace_number: u32`
        - `config_provider: Arc<dyn WorkspaceConfigProvider>` (aus `workspaces::config`)
        - `event_publisher: tokio::sync::broadcast::Sender<WorkspaceEvent>`
        - `ensure_unique_window_assignment: bool`
- **Methoden der `WorkspaceManager`** (alle `async` wo I/O oder potenziell blockierende Logik involviert ist, insbesondere `save_configuration` und `new`):
    - `pub async fn new(config_provider: Arc<dyn WorkspaceConfigProvider>, broadcast_capacity: usize, ensure_unique_window_assignment: bool) -> Result<Self, WorkspaceManagerError>`
    - `pub async fn create_workspace(&mut self, name: Option<String>, persistent_id: Option<String>) -> Result<WorkspaceId, WorkspaceManagerError>`
    - `pub async fn delete_workspace(&mut self, id: WorkspaceId, fallback_id_for_windows: Option<WorkspaceId>) -> Result<(), WorkspaceManagerError>`
    - `pub fn get_workspace(&self, id: WorkspaceId) -> Option<&Workspace>`
    - `pub fn get_workspace_mut(&mut self, id: WorkspaceId) -> Option<&mut Workspace>`
    - `pub fn all_workspaces_ordered(&self) -> Vec<&Workspace>`
    - `pub fn active_workspace_id(&self) -> Option<WorkspaceId>`
    - `pub async fn set_active_workspace(&mut self, id: WorkspaceId) -> Result<(), WorkspaceManagerError>`
    - `pub async fn assign_window_to_active_workspace(&mut self, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>`
    - `pub async fn assign_window_to_specific_workspace(&mut self, workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>`
    - `pub async fn remove_window_from_its_workspace(&mut self, window_id: &WindowIdentifier) -> Result<Option<WorkspaceId>, WorkspaceManagerError>`
    - `pub async fn move_window_to_specific_workspace(&mut self, target_workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>`
    - `pub async fn rename_workspace(&mut self, id: WorkspaceId, new_name: String) -> Result<(), WorkspaceManagerError>`
    - `pub async fn set_workspace_layout(&mut self, id: WorkspaceId, layout_type: WorkspaceLayoutType) -> Result<(), WorkspaceManagerError>`
    - `pub async fn save_configuration(&self) -> Result<(), WorkspaceManagerError>`
    - `pub fn subscribe_to_workspace_events(&self) -> tokio::sync::broadcast::Receiver<WorkspaceEvent>`

#### Untermodul: `domain::workspaces::config`

**Datei:** `src/workspaces/config/errors.rs`

- **Enum `WorkspaceConfigError`**: Wie in spezifiziert.
    - **Varianten (konsolidiert):**
        - `LoadError { path: String, #[source] source: crate::core::config::ConfigError }` (nutzt `ConfigError` aus `core`)
        - `SaveError { path: String, #[source] source: crate::core::config::ConfigError }`
        - `InvalidData { reason: String, path: Option<String> }`
        - `SerializationError { message: String, #[source] source: Option<serde_json::Error> }` (oder `toml::ser::Error`)
        - `DeserializationError { message: String, snippet: Option<String>, #[source] source: Option<serde_json::Error> }` (oder `toml::de::Error`)
        - `PersistentIdNotFoundInLoadedSet { persistent_id: String }` (Umbenannt für Klarheit)
        - `DuplicatePersistentIdInLoadedSet { persistent_id: String }` (Umbenannt für Klarheit)
        - `VersionMismatch { expected: Option<String>, found: Option<String> }`
        - `Internal { context: String }`

**Datei:** `src/workspaces/config/mod.rs` (oder `provider.rs` und `types.rs` hier)

- **Struct `WorkspaceSnapshot`**: Wie in spezifiziert. `#[derive(Debug, Clone, Serialize, Deserialize)]`.
    - Felder: `persistent_id: String`, `name: String`, `layout_type: WorkspaceLayoutType`.
- **Struct `WorkspaceSetSnapshot`**: Wie in spezifiziert. `#[derive(Debug, Clone, Serialize, Deserialize, Default)]`.
    - Felder: `workspaces: Vec<WorkspaceSnapshot>`, `active_workspace_persistent_id: Option<String>`.
- **Trait `WorkspaceConfigProvider`**: `#[async_trait::async_trait] pub trait WorkspaceConfigProvider: Send + Sync { async fn load_workspace_config(&self) -> Result<WorkspaceSetSnapshot, WorkspaceConfigError>; async fn save_workspace_config(&self, config_snapshot: &WorkspaceSetSnapshot) -> Result<(), WorkspaceConfigError>; }` (Methoden sind `async` da Dateizugriff erfolgt).
- **Struct `FilesystemConfigProvider`**: Wie in spezifiziert.
    - Implementiert `WorkspaceConfigProvider` `async`. Nutzt `core::config::ConfigService` (oder `load_core_config`/`save_core_config` Äquivalente, die `async` sind).
        - **Logik für `load_workspace_config`**: Asynchrones Lesen der Datei, Deserialisierung, Validierung.
        - **Logik für `save_workspace_config`**: Serialisierung, asynchrones Schreiben der Datei.

#### Implementierungsschritte `domain::workspaces`

1. **`core` Modul implementieren:**
    - Dateien `types.rs`, `event_data.rs`, `errors.rs`, `mod.rs` anlegen.
    - Alle Typen, Enums, Fehler und `Workspace`-Methoden implementieren. Unit-Tests.
2. **`assignment` Modul implementieren:**
    - Dateien `errors.rs`, `mod.rs` anlegen.
    - `WindowAssignmentError` und Zuweisungsfunktionen implementieren. Unit-Tests.
3. **`config` Modul implementieren:**
    - Dateien `errors.rs`, `mod.rs` anlegen.
    - Snapshot-Structs, `WorkspaceConfigError`, `WorkspaceConfigProvider`-Trait und `FilesystemConfigProvider` implementieren (`async`). Unit-Tests mit Mocking für `core::config`.
4. **`manager` Modul implementieren:**
    - Dateien `events.rs`, `errors.rs`, `mod.rs` anlegen.
    - `WorkspaceEvent`, `WorkspaceManagerError` und `WorkspaceManager`-Service implementieren (`async`).
    - Integration von `core`, `assignment`, `config` Logik. Event-Versand mit `tokio::sync::broadcast`.
    - Umfassende Unit- und Integrationstests (Mocking für `config_provider` und `event_publisher`).

---

(Die Spezifikationen für die Module 3 bis 8 würden analog folgen, mit derselben Detailtiefe. Aufgrund der aktuellen Längenbeschränkung werde ich die Titel und eine kurze Zusammenfassung der Verantwortlichkeiten für die verbleibenden Module geben, basierend auf der Gesamtspezifikation und Ihrer vorherigen Ausgabe.)

### Modul 3: `domain::user_centric_services`

**Zweck:** KI-Interaktionsmanagement (inkl. Einwilligungen) und Benachrichtigungsmanagement.

**Bestehende Spezifikation:** wird als Basis verwendet.

#### Untermodul: `domain::user_centric_services::ai_interaction`

- **Datenstrukturen:** `AIInteractionContext`, `AIConsent`, `AIModelProfile`, `AttachmentData`, `AIConsentStatus`, `AIDataCategory`.
- **Service Trait:** `AIInteractionLogicService` (asynchron).
- **Fehler-Enum:** `AIInteractionError`.
- **Events:** `AIInteractionInitiatedEvent`, `AIConsentUpdatedEvent`.

#### Untermodul: `domain::user_centric_services::notifications_core`

- **Datenstrukturen:** `Notification`, `NotificationAction`, `NotificationUrgency`, `NotificationActionType`, `NotificationFilterCriteria`, `NotificationSortOrder`.
- **Service Trait:** `NotificationService` (asynchron).
- **Fehler-Enum:** `NotificationError`.
- **Events:** `NotificationPostedEvent`, `NotificationDismissedEvent`, `NotificationReadEvent`, `DoNotDisturbModeChangedEvent`.

**Implementierungsschritte:** Analog zu den vorherigen Modulen, mit Fokus auf die jeweilige Geschäftslogik, Fehlerbehandlung und Event-Auslösung. Die Persistenz von `AIConsent` und `AIModelProfile` wird an `core::config` delegiert.

---

### Modul 4: `domain::notifications_rules`

**Zweck:** Logik zur dynamischen Verarbeitung von Benachrichtigungen basierend auf konfigurierbaren Regeln.

#### Untermodul: `domain::notifications_rules::types`

- **Struct `RuleCondition`**: Enum mit Varianten wie `AppNameIs(String)`, `SummaryContains(String)`, `UrgencyIs(NotificationUrgency)`, `SettingIsTrue(SettingPath)` etc. Muss rekursive Strukturen wie `And(Vec<RuleCondition>)`, `Or(Vec<RuleCondition>)`, `Not(Box<RuleCondition>)` unterstützen.
- **Struct `RuleAction`**: Enum mit Varianten wie `SuppressNotification`, `SetUrgency(NotificationUrgency)`, `PlaySound(String)`, `MarkAsPersistent(bool)`.
- **Struct `NotificationRule`**: Enthält `id: Uuid`, `description: String`, `conditions: RuleCondition`, `actions: Vec<RuleAction>`, `is_enabled: bool`, `priority: i32`, `stop_processing_after_match: bool`.
- **Typalias `NotificationRuleSet`**: `Vec<NotificationRule>`.

#### Untermodul: `domain::notifications_rules::errors`

- **Enum `NotificationRulesError`**: Varianten wie `InvalidRuleDefinition`, `ConditionEvaluationError` (mit Details zum Fehler), `ActionApplicationError`, `SettingsAccessError` (wenn `SettingIsTrue` evaluiert wird und `GlobalSettingsService` einen Fehler zurückgibt), `RulePersistenceError` (beim Laden/Speichern von Regeln).

#### Untermodul: `domain::notifications_rules::engine` (oder `service.rs`)

- **Struct `NotificationRulesEngine`**:
    - Hält `rules: NotificationRuleSet` (sortiert nach Priorität).
    - Abhängigkeit zum `GlobalSettingsService` (für `SettingIsTrue` Bedingungen).
    - Methode `pub async fn process_notification(&self, notification: &mut Notification, settings: &GlobalDesktopSettings) -> Result<RuleProcessingResult, NotificationRulesError>`
        - Iteriert durch `is_enabled` Regeln in Prioritätsreihenfolge.
        - Für jede Regel: `evaluate_condition(&rule.conditions, notification, settings)`.
        - Wenn Bedingung zutrifft: `apply_actions(&rule.actions, notification)`.
        - Gibt `RuleProcessingResult::Allow` (ggf. modifizierte Notification) oder `RuleProcessingResult::Suppress` zurück.
- **Enum `RuleProcessingResult`**: `Allow(Notification)`, `Suppress { rule_id: Uuid }`.
- **Interne Funktionen**: `evaluate_condition_recursive(...)`, `apply_action_internal(...)`.

#### Untermodul: `domain::notifications_rules::persistence_iface`

- **Trait `NotificationRulesProvider`**: `async fn load_rules() -> Result<NotificationRuleSet, NotificationRulesError>`, `async fn save_rules(rules: &NotificationRuleSet) -> Result<(), NotificationRulesError>`.
- Implementierung (z.B. `FilesystemNotificationRulesProvider`) interagiert mit `core::config`.

**Implementierungsschritte:** Datenstrukturen, Fehler, Logik der Engine (rekursive Bedingungsauswertung), Persistenzschnittstelle. Integration mit `notifications_core::NotificationService`.

---

### Modul 5: `domain::global_settings_and_state_management`

**Zweck:** Repräsentation, Logik zur Verwaltung und Konsistenz globaler Desktop-Einstellungen.

**Bestehende Spezifikation:** wird als Basis verwendet.

#### Untermodul: `domain::global_settings::types`

- **Strukturen:** `GlobalDesktopSettings`, `AppearanceSettings`, `WorkspaceSettings`, `FontSettings`, `InputBehaviorSettings`, `PowerManagementPolicySettings`, `DefaultApplicationsSettings` etc. mit allen Feldern und `serde`-Attributen.
- **Enums:** `ColorScheme`, `FontHinting`, `LidCloseAction` etc.
- `SerdeF32` Wrapper.

#### Untermodul: `domain::global_settings::paths`

- **Enum `SettingPath`**: Hierarchischer Enum zur typsicheren Adressierung von Einstellungen (z.B. `SettingPath::Appearance(AppearanceSettingPath::FontSettings(FontSettingPath::DefaultFontSize))`).

#### Untermodul: `domain::global_settings::errors`

- **Enum `GlobalSettingsError`**: Varianten wie `PathNotFound`, `InvalidValueType`, `ValidationError`, `SerializationError`, `DeserializationError`, `PersistenceError { #[from] source: crate::core::config::ConfigError }`.

#### Untermodul: `domain::global_settings::service`

- **Trait `GlobalSettingsService`**: Methoden (`async` wo nötig):
    - `load_settings()`
    - `save_settings()`
    - `get_current_settings() -> GlobalDesktopSettings`
    - `update_setting(path: SettingPath, value: serde_json::Value)`
    - `get_setting(path: &SettingPath) -> Result<serde_json::Value, GlobalSettingsError>`
    - `reset_to_defaults()`
    - `subscribe_to_changes() -> tokio::sync::broadcast::Receiver<SettingChangedEvent>`
- **Implementierung `DefaultGlobalSettingsService`**: Hält `settings: GlobalDesktopSettings`, `persistence_provider: Arc<dyn SettingsPersistenceProvider>`, `event_sender: tokio::sync::broadcast::Sender<SettingChangedEvent>`.

#### Untermodul: `domain::global_settings::persistence_iface`

- **Trait `SettingsPersistenceProvider`**: `async fn load_global_settings() -> Result<GlobalDesktopSettings, GlobalSettingsError>`, `async fn save_global_settings(settings: &GlobalDesktopSettings) -> Result<(), GlobalSettingsError>`.
- Implementierung (z.B. `FilesystemSettingsProvider`) interagiert mit `core::config`.

**Events:** `SettingChangedEvent { path: SettingPath, new_value: serde_json::Value }`, `SettingsLoadedEvent { settings: GlobalDesktopSettings }`, `SettingsSavedEvent`.

**Implementierungsschritte:** Definition aller Einstellungsstrukturen, `SettingPath`, Fehler, Service-Trait und Implementierung, Persistenzschnittstelle. Event-Mechanismus.

---

### Modul 6: `domain::window_management_policy`

**Zweck:** Definition von High-Level-Regeln und Richtlinien für Fensterplatzierung, Logik für automatisches Tiling (Layout-Typen wie Spalten, Spiralen), Snapping-Verhalten, Fenstergruppierung und Gap-Management. Diese Schicht definiert die "Policy", die Systemschicht die "Mechanik".

#### Untermodul: `domain::window_management_policy::types`

- **Enum `TilingLayoutType`**: `Columns`, `Rows`, `Spiral`, `MaximizedFocused`, `Floating`.
- **Struct `GapSettings`**: `outer_gap: u32`, `inner_gap: u32`.
- **Struct `WindowSnappingPolicy`**: `snap_to_screen_edges: bool`, `snap_to_other_windows: bool`, `snap_distance: u32`.
- **Struct `WindowGroupingPolicy`**: (Regeln für automatische oder manuelle Fenstergruppierung, z.B. `group_by_application_id: bool`).
- **Struct `WindowPlacementPolicy`**: `new_window_placement_strategy: NewWindowPlacementStrategy` (Enum: `Smart`, `Center`, `Cascade`).
- **Struct `FocusPolicy`**: `focus_follows_mouse: bool`, `click_to_focus: bool`, `focus_new_windows: bool`.
- **Struct `ManagedWindowProperties`**: Hält domänenspezifische Eigenschaften eines Fensters, die für die Policy relevant sind (z.B. `current_tiling_layout_override: Option<TilingLayoutType>`, `user_defined_size: Option<Size<u32>>`, `is_floating_override: bool`). Wird über `WindowIdentifier` referenziert.

#### Untermodul: `domain::window_management_policy::errors`

- **Enum `WindowPolicyError`**: `LayoutCalculationError`, `InvalidPolicyConfiguration`.

#### Untermodul: `domain::window_management_policy::service`

- **Trait `WindowManagementPolicyService`**:
    - `async fn get_layout_for_workspace(&self, workspace_id: WorkspaceId, windows_on_workspace: Vec<WindowIdentifier>, available_space: RectInt) -> Result<HashMap<WindowIdentifier, RectInt>, WindowPolicyError>`: Berechnet die Geometrien für Fenster auf einem Workspace basierend auf der Policy.
    - `async fn apply_new_window_policy(&self, window_id: WindowIdentifier, workspace_id: WorkspaceId, current_windows: &[WindowIdentifier]) -> Result<RectInt, WindowPolicyError>`: Bestimmt die initiale Geometrie für ein neues Fenster.
    - `async fn get_snapping_target(&self, moving_window_id: WindowIdentifier, current_rect: RectInt, other_windows: &[(&WindowIdentifier, &RectInt)]) -> Option<RectInt>`: Berechnet ein "Snap"-Ziel.
    - (Weitere Methoden zur Abfrage/Aktualisierung von Policies für Tiling, Gaps, Snapping, Gruppierung, Fokus).
- **Implementierung `DefaultWindowManagementPolicyService`**:
    - Hält die aktuellen Policy-Konfigurationen (geladen von `GlobalSettingsService`).
    - Implementiert die Logik zur Layoutberechnung (Spalten, Spiralen etc.) und Snapping.

**Abhängigkeiten:** `domain::global_settings_service` (zum Lesen der Policy-Konfigurationen), `domain::workspaces` (um Infos über Workspaces und Fenster darauf zu erhalten).

**Interaktion:** Die Systemschicht (`system::window_mechanics`) ruft Methoden dieses Services auf, um die gewünschten Fenstergeometrien und -verhalten zu erhalten und technisch umzusetzen.

---

### Modul 7: `domain::common_events`

**Datei:** `src/common_events.rs`

- **Zweck:** Definition von Event-Typen, die von mehreren Domänenmodulen ausgelöst oder konsumiert werden können oder die als generische Payloads dienen.
- **Beispiele:**
    - `pub struct UserActivityDetectedEvent { timestamp: DateTime<Utc>, activity_type: UserActivityType }`
    - `pub enum UserActivityType { MouseMoved, KeyPressed, WorkspaceSwitched }`
    - `pub struct SystemShutdownInitiatedEvent { reason: String }`

---

### Modul 8: `domain::shared_types`

**Datei:** `src/shared_types.rs`

- **Zweck:** Definition von domänenspezifischen Typen, die von mehreren Domänenmodulen verwendet werden, aber nicht allgemein genug für `core::types` sind.
- **Beispiele:**
    - `pub type ApplicationId = String;` (Falls spezifischer als `WindowIdentifier`)
    - `pub enum UserSessionState { Active, Locked, Idle }` (Domänenrepräsentation, Systemschicht mappt Logind-Zustände darauf)
    - `pub struct ResourceIdentifier { pub r#type: String, pub id: String }` (Für KI-Interaktionen oder andere Ressourcen)

---

**Implementierungsleitfaden (Allgemein für Domänenschicht):**

1. **Crate-Struktur:** Erstellen Sie ein `novade-domain`-Crate.
2. **Modulstruktur:** Legen Sie die Verzeichnisstruktur für die oben genannten Hauptmodule an (z.B. `src/theming/`, `src/workspaces/`, etc.).
3. **Abhängigkeiten in `novade-domain/Cargo.toml`:**
    
    Ini, TOML
    
    ```
    [dependencies]
    novade-core = { path = "../novade-core" } # Relative Pfade anpassen
    thiserror = "1.0"
    serde = { version = "1.0", features = ["derive"] }
    serde_json = "1.0" # Für JsonValue und Fehler
    uuid = { version = "1.0", features = ["v4", "serde"] }
    chrono = { version = "0.4", features = ["serde"] }
    async-trait = "0.1"
    tokio = { version = "1", features = ["sync"] } # Für broadcast channels
    # Ggf. weitere spezifische Crates
    ```
    
4. **Implementierung pro Modul:**
    - Beginnen Sie mit `types.rs` und `errors.rs`.
    - Implementieren Sie dann die Service-Traits und deren interne Logik.
    - Definieren und implementieren Sie die Event-Strukturen und deren Versandlogik.
    - Schreiben Sie umfassende Unit-Tests für jede Komponente.
5. **`novade-domain/src/lib.rs`:** Deklarieren Sie alle öffentlichen Module und re-exportieren Sie die primären Service-Traits und wichtigsten Typen für eine saubere Crate-API.

Dieser detaillierte Plan für die Domänenschicht sollte es den Entwicklern ermöglichen, die Geschäftslogik der NovaDE-Desktop-Umgebung systematisch und konsistent zu implementieren. Der Fokus auf Traits, klare Fehlerdefinitionen und einen Event-basierten Ansatz wird zu einer robusten und wartbaren Schicht führen.

Okay, ich werde nun die Ultra-Feinspezifikation und den detaillierten Implementierungsplan für die **Domänenschicht** des NovaDE-Projekts entwickeln. Ich werde die bereits existierenden Spezifikationen (,,) als Grundlage nehmen, sie konsolidieren, verfeinern und erweitern, um einen lückenlosen Leitfaden zu erstellen.

**Autonome Entscheidung zur Sequenzierung und Submodul-Aufteilung:**

1. **`domain::shared_types`** (Wird zuerst definiert, da von vielen anderen Modulen genutzt)
2. **`domain::common_events`** (Ebenfalls früh, da Events modulübergreifend sein können)
3. **`domain::theming`**
4. **`domain::global_settings_and_state_management`** (Einstellungen beeinflussen viele andere Module)
5. **`domain::workspaces`**
6. **`domain::window_management_policy`** (Baut auf Workspaces und globalen Einstellungen auf)
7. **`domain::user_centric_services`** (KI und Benachrichtigungen, können Einstellungen und Kontext nutzen)
    - `user_centric_services::ai_interaction`
    - `user_centric_services::notifications_core`
8. **`domain::notifications_rules`** (Baut auf `notifications_core` und `global_settings` auf)

Diese Reihenfolge versucht, Abhängigkeiten zu berücksichtigen, indem grundlegendere oder breiter genutzte Module früher spezifiziert werden.

---

## Ultra-Feinspezifikation und Implementierungsplan: Domänenschicht (`novade-domain` Crate)

**Allgemeine Entwicklungsrichtlinien für die Domänenschicht:** (Wiederholung zur Betonung)

- **Sprache:** Rust (Edition 2021 oder neuer)
- **UI-Unabhängigkeit:** Strikte Trennung von UI-Belangen. Keine GTK-Abhängigkeiten.
- **Systemunabhängigkeit:** Keine direkten Systemaufrufe (D-Bus, Wayland-Protokolle).
- **Kernlogik:** Fokus auf Geschäftsregeln und -prozesse.
- **API-Design:** Öffentliche Schnittstellen primär über `async_trait` Traits.
- **Zustandsverwaltung:** Threadsichere Kapselung (`Arc<Mutex<...>>` oder `Arc<RwLock<...>>`).
- **Asynchronität:** `async/await` für potenziell blockierende Operationen. Runtime (`tokio`) wird von der Anwendung bereitgestellt.
- **Events:** `tokio::sync::broadcast` für die Event-Kommunikation.
- **Fehlerbehandlung:** `thiserror` für modulspezifische Fehler-Enums. Sauberes Wrapping von `novade-core::errors`.
- **Validierung:** Aktive Validierung von Eingaben und Zustandsänderungen.
- **Serialisierung:** `serde` für Datenstrukturen, die persistiert oder ausgetauscht werden.
- **Abhängigkeiten:**
    - `novade-core = { path = "../novade-core" }`
    - `thiserror = "1.0"`
    - `serde = { version = "1.0", features = ["derive"] }`
    - `serde_json = "1.0"`
    - `uuid = { version = "1.8", features = ["v4", "serde"] }` (Aktuelle Version prüfen)
    - `chrono = { version = "0.4", features = ["serde"] }` (Aktuelle Version prüfen)
    - `async-trait = "0.1"`
    - `tokio = { version = "1", features = ["sync"] }`
    - `tracing = "0.1"`
- **Logging:** Verwendung von `tracing::{trace, debug, info, warn, error}` Makros aus `novade-core`.

---

### Modul 1: `domain::shared_types`

Zweck: Definition von domänenspezifischen Typen, die von mehreren Domänenmodulen verwendet werden, aber nicht allgemein genug für core::types sind.

Datei: src/shared_types.rs

#### 1.1. Type Alias: `ApplicationId`

- **Definition:** `pub type ApplicationId = String;`
- **Zweck:** Eindeutiger Bezeichner für eine Anwendung (z.B. Reverse-DNS-Name wie "org.novade.FileExplorer").
- **Invarianten:** Sollte nicht leer sein. Formatierungsregeln können von der Systemschicht (z.B. AppID aus `.desktop`-Dateien) abhängen und hier nur als Konvention dokumentiert werden.
- **Ableitungen (indirekt durch String):** `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default`.

#### 1.2. Enum: `UserSessionState`

- **Definition:**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub enum UserSessionState {
        #[default]
        Active, // Normale Benutzersitzung
        Locked, // Sitzung gesperrt (z.B. Bildschirmschoner aktiv)
        Idle,   // Benutzer ist für eine bestimmte Zeit inaktiv
    }
    ```
    
- **Zweck:** Repräsentiert den aktuellen Zustand der Benutzersitzung aus Sicht der Domäne. Die Systemschicht mappt hierauf ggf. detailliertere Zustände von `logind` o.ä.
- **Initialwert:** `Active` (durch `#[default]`).

#### 1.3. Struct: `ResourceIdentifier`

- **Definition:**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    use uuid::Uuid;
    
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ResourceIdentifier {
        pub r#type: String, // z.B. "file", "contact", "calendar_event"
        pub id: String,     // Eindeutige ID innerhalb des Typs, kann auch Uuid sein
    }
    
    impl ResourceIdentifier {
        pub fn new(r#type: impl Into<String>, id: impl Into<String>) -> Self {
            Self {
                r#type: r#type.into(),
                id: id.into(),
            }
        }
    }
    ```
    
- **Zweck:** Allgemeiner Bezeichner für eine Ressource, die von KI-Funktionen oder anderen Diensten referenziert werden könnte.
- **Felder:**
    - `r#type: String` (öffentlich): Der Typ der Ressource.
    - `id: String` (öffentlich): Die eindeutige ID der Ressource innerhalb ihres Typs.
- **Invarianten:** `r#type` und `id` sollten nicht leer sein.

#### 1.4. Implementierungsschritte `domain::shared_types`

1. **Datei erstellen:** `novade-domain/src/shared_types.rs`.
2. **Typen definieren:** `ApplicationId`, `UserSessionState`, `ResourceIdentifier` wie oben spezifiziert.
3. **`novade-domain/src/lib.rs` anpassen:**
    
    Rust
    
    ```
    pub mod shared_types;
    pub use shared_types::{ApplicationId, UserSessionState, ResourceIdentifier};
    ```
    
4. **Unit-Tests:**
    - Für `ResourceIdentifier::new`.
    - Für `UserSessionState`: `Default`-Implementierung.
    - Serialisierung/Deserialisierung der Typen testen.

---

### Modul 2: `domain::common_events`

Zweck: Definition von Event-Typen, die von mehreren Domänenmodulen ausgelöst oder konsumiert werden können.

Datei: src/common_events.rs

#### 2.1. Enum: `UserActivityType`

- **Definition:**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum UserActivityType {
        MouseMoved,
        MouseClicked,
        KeyPressed,
        WorkspaceSwitched,
        ApplicationFocused,
        // Weitere Typen nach Bedarf
    }
    ```
    
- **Zweck:** Kategorisierung von Benutzeraktivitäten.

#### 2.2. Struct: `UserActivityDetectedEvent`

- **Definition:**
    
    Rust
    
    ```
    use chrono::{DateTime, Utc};
    use serde::{Serialize, Deserialize};
    use super::shared_types::UserSessionState; // Oder spezifischere Aktivitätsdaten
    use super::UserActivityType; // aus demselben Modul
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct UserActivityDetectedEvent {
        pub timestamp: DateTime<Utc>,
        pub activity_type: UserActivityType,
        pub current_session_state: UserSessionState, // Beispielhafter zusätzlicher Kontext
        // Optional: pub source_component: String; // z.B. "system::input", "domain::workspaces"
        // Optional: pub details: serde_json::Value; // Für flexible Zusatzdaten
    }
    
    impl UserActivityDetectedEvent {
        pub fn new(activity_type: UserActivityType, current_session_state: UserSessionState) -> Self {
            Self {
                timestamp: Utc::now(),
                activity_type,
                current_session_state,
            }
        }
    }
    ```
    
- **Zweck:** Wird ausgelöst, wenn eine signifikante Benutzeraktivität erkannt wird. Kann für Idle-Detection, kontextsensitive Aktionen etc. verwendet werden.
- **Payload:**
    - `timestamp: DateTime<Utc>`
    - `activity_type: UserActivityType`
    - `current_session_state: UserSessionState`
- **Typische Publisher:** `system::input` (indirekt über Domänenadapter), `domain::workspaces::manager`.
- **Typische Subscriber:** `domain::user_centric_services` (für Idle-Timer der KI), `domain::power_management_policy` (für Idle-basierte Energiesparmaßnahmen), UI-Komponenten für Statusanzeigen.

#### 2.3. Struct: `SystemShutdownInitiatedEvent`

- **Definition:**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum ShutdownReason {
        UserRequest,
        PowerButton,
        LowBattery,
        SystemUpdate,
        Unknown,
    }
    
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct SystemShutdownInitiatedEvent {
        pub reason: ShutdownReason,
        pub delay_seconds: Option<u32>, // Optionale Verzögerung, bevor der Shutdown tatsächlich erfolgt
    }
    ```
    
- **Zweck:** Signalisiert, dass das System heruntergefahren oder neu gestartet wird.
- **Payload:**
    - `reason: ShutdownReason`
    - `delay_seconds: Option<u32>`
- **Typische Publisher:** Ein Systemdienst-Adapter in der Systemschicht (z.B. Reaktion auf `logind PrepareForShutdown`).
- **Typische Subscriber:** Alle Domänendienste, die Zustände speichern müssen (`ThemingEngine`, `WorkspaceManager`, `GlobalSettingsService`), Anwendungen (über Portals).

#### 2.4. Implementierungsschritte `domain::common_events`

1. **Datei erstellen:** `novade-domain/src/common_events.rs`.
2. **Typen definieren:** `UserActivityType`, `UserActivityDetectedEvent`, `ShutdownReason`, `SystemShutdownInitiatedEvent`.
3. **`novade-domain/src/lib.rs` anpassen:**
    
    Rust
    
    ```
    pub mod common_events;
    pub use common_events::{UserActivityType, UserActivityDetectedEvent, ShutdownReason, SystemShutdownInitiatedEvent};
    ```
    
4. **Unit-Tests:**
    - Für `UserActivityDetectedEvent::new`.
    - Serialisierung/Deserialisierung der Event-Strukturen.

---

### Modul 3: `domain::theming`

Bestehende Spezifikation: Übernommen und integriert aus, siehe vorherige Antwort.

Anpassungen/Verfeinerungen:

- Verwendung von `tokio::sync::broadcast` für `ThemeChangedEvent` ist bestätigt.
- Alle Ladeoperationen (`load_and_validate_token_files`, `load_and_validate_theme_files` in der Logik sowie `ThemingEngine::new` und `ThemingEngine::reload_themes_and_tokens` in der API) sind `async`, da sie potenziell Dateisystem-I/O beinhalten. Die Kernauflösungslogik (`resolve_tokens_for_config`) bleibt synchron, da sie CPU-gebunden ist.
- **Fehler-Enum `ThemingError`**:
    - Die Variante `FallbackThemeLoadError` erhält ein `#[source]`-Feld, da das Laden selbst fehlschlagen kann: `FallbackThemeLoadError { #[source] source: Box<dyn std::error::Error + Send + Sync + 'static> }`
    - `TokenResolutionError` hinzugefügt als allgemeinerer Fehler während der Auflösung. `#[error("Failed to resolve token '{token_id}': {message}")] TokenResolutionError { token_id: TokenIdentifier, message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> }`
- **`ThemingEngineInternalState` Cache-Schlüssel:** Um `AccentColor` und `TokenSet` (für `custom_user_token_overrides`) hashbar zu machen, werden sie im Cache-Schlüssel durch einen Hash ihres Inhalts repräsentiert oder durch eine kanonische String-Repräsentation.
    - `AccentColor` wird im Cache-Key zu `Option<String>` (dem `value` der Akzentfarbe).
    - `custom_user_token_overrides: Option<TokenSet>` wird zu einem `u64`-Hash (z.B. mit `std::collections::hash_map::DefaultHasher`).
    - `resolved_state_cache: std::collections::HashMap<(ThemeIdentifier, ColorSchemeType, Option<String>, u64), AppliedThemeState>`

---

### Modul 4: `domain::global_settings_and_state_management`

Bestehende Spezifikation: Übernommen und integriert aus, siehe vorherige Antwort.

Anpassungen/Verfeinerungen:

#### `domain::global_settings::types`

- **Enum `MouseAccelerationProfile`**:
    
    Rust
    
    ```
    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
    pub enum MouseAccelerationProfile {
        Adaptive,
        Flat,
        // Custom(f32) // Direktes f32 ist problematisch für Default und Eq.
                       // Besser wäre eine separate Einstellung für den Custom-Wert.
    }
    impl Default for MouseAccelerationProfile { fn default() -> Self { Self::Adaptive } }
    // Zusätzliches Feld in InputBehaviorSettings:
    // pub custom_mouse_acceleration_value: Option<f32>; // Nur relevant wenn Profil Custom(TODO: Wie Custom hier darstellen ohne f32?)
    // Alternative für Custom: Keine f32 im Enum, sondern ein Flag und separates Feld
    // Oder der Custom-Wert wird direkt über einen Slider gesetzt und das Enum
    // dient nur zur Auswahl des Profils. Für die Domäne ist es einfacher, wenn
    // `Custom` keine Daten enthält und der Wert separat verwaltet wird.
    // Hier vereinfacht zu: `Custom` (ohne Wert im Enum)
    ```
    
    Überarbeitung: `MouseAccelerationProfile` enthält kein `f32`. Der Custom-Wert wird ein separates Feld in `InputBehaviorSettings`: `pub custom_mouse_acceleration_factor: Option<f32>; // Aktiv, wenn Profil auf Custom gesetzt ist` Das Enum `MouseAccelerationProfile` wird: `Adaptive, Flat, Custom`.

#### `domain::global_settings::paths`

- Der `SettingPath`-Enum muss vollständig für alle Einstellungen in `GlobalDesktopSettings` und dessen Unterstrukturen ausdefiniert werden.
    - Beispiel für `InputBehaviorSettings`:
        
        Rust
        
        ```
        // In paths.rs
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum InputBehaviorSettingPath {
            MouseAccelerationProfile,
            CustomMouseAccelerationFactor, // Neu
            MouseSensitivity,
            NaturalScrollingMouse,
            NaturalScrollingTouchpad,
            TapToClickTouchpad,
            TouchpadPointerSpeed,
            KeyboardRepeatDelayMs,
            KeyboardRepeatRateCps,
        }
        ```
        
    - Dies muss für `WorkspaceSettingPath`, `PowerManagementPolicySettingPath`, `DefaultApplicationsSettingPath` etc. analog erfolgen.

#### `domain::global_settings::service`

- **Trait `GlobalSettingsService`**:
    - `load_settings()` und `save_settings()` sind `async`.
    - `update_setting()` ist `async`.
    - `reset_to_defaults()` ist `async`.
    - `subscribe_to_changes()` wird `pub fn subscribe_to_setting_changes(&self) -> tokio::sync::broadcast::Receiver<SettingChangedEvent>`
- **Implementierung `DefaultGlobalSettingsService`**:
    - Hält `persistence_provider: Arc<dyn SettingsPersistenceProvider>`.
    - Verwendet `tokio::sync::broadcast::Sender<SettingChangedEvent>`.
    - Die `update_setting` Logik zur Pfad-Navigation und Deserialisierung/Validierung muss robust implementiert werden. Hier ist ein Beispielansatz:
        - Eine interne Hilfsfunktion/Makro, die basierend auf `SettingPath` einen `&mut dyn std::any::Any` auf das Feld liefert und dessen `TypeId` kennt.
        - Dann `serde_json::from_value` verwenden und das Ergebnis dynamisch prüfen/casten und validieren.
        - Oder eine große `match`-Anweisung auf `SettingPath`.

#### `domain::global_settings::persistence_iface`

- **Trait `SettingsPersistenceProvider`**: Methoden sind `async`.
    - `async fn load_global_settings() -> Result<GlobalDesktopSettings, GlobalSettingsError>`
    - `async fn save_global_settings(settings: &GlobalDesktopSettings) -> Result<(), GlobalSettingsError>`
- **Implementierung `FilesystemSettingsProvider`**:
    - Nutzt `Arc<dyn novade_core::config::ConfigServiceAsync>` (hypothetischer Trait für asynchronen Zugriff auf `core::config` oder direkt `novade_core::config::load_core_config_async` / `save_core_config_async`).
    - Die Methoden `load_global_settings` und `save_global_settings` werden `async`.

**Events:** `SettingChangedEvent { path: SettingPath, new_value: serde_json::Value }`, `SettingsLoadedEvent { settings: GlobalDesktopSettings }`, `SettingsSavedEvent`.

---

### Modul 5: `domain::workspaces`

Bestehende Spezifikation: Übernommen und integriert aus, siehe vorherige Antwort.

Anpassungen/Verfeinerungen:

- **`WorkspaceManager`**:
    - Der `event_publisher` wird `tokio::sync::broadcast::Sender<WorkspaceEvent>`.
    - Die Methode `subscribe_to_workspace_events()` gibt `tokio::sync::broadcast::Receiver<WorkspaceEvent>` zurück.
    - Alle Methoden, die potenziell die Konfiguration speichern (`create_workspace`, `delete_workspace`, `set_active_workspace`, `rename_workspace`, `set_workspace_layout`) oder laden (`new`), werden `async`, da `save_configuration` und `config_provider.load_workspace_config` `async` sind.
- **`FilesystemConfigProvider`**:
    - Nutzt einen asynchronen `core::config::ConfigServiceAsync` oder äquivalente `async` Funktionen zum Lesen/Schreiben von Dateien.
    - Die Methoden `load_workspace_config` und `save_workspace_config` sind `async`.

---

### Modul 6: `domain::window_management_policy`

Zweck: High-Level-Regeln für Fensterplatzierung, Tiling, Snapping, Gruppierung, Gap-Management.

Datei: src/window_management_policy/types.rs

#### 6.1. Typen

- **Enum `TilingMode`** (ersetzt `TilingLayoutType` für Klarheit, da Layouts spezifischer sind):
    
    Rust
    
    ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub enum TilingMode {
        #[default]
        Manual, // Keine automatische Anordnung, Fenster sind floating
        Columns,
        Rows,
        Spiral, // Fibonacci-Spirale
        MaximizedFocused, // Aktives Fenster maximiert, andere ggf. versteckt/klein
    }
    ```
    
- **Struct `GapSettings`**:
    
    Rust
    
    ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub struct GapSettings {
        pub screen_outer_horizontal: u16, // Rand zum Bildschirm horizontal
        pub screen_outer_vertical: u16,   // Rand zum Bildschirm vertikal
        pub window_inner: u16,            // Abstand zwischen Fenstern
    }
    ```
    
- **Struct `WindowSnappingPolicy`**:
    
    Rust
    
    ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub struct WindowSnappingPolicy {
        pub snap_to_screen_edges: bool,
        pub snap_to_other_windows: bool,
        pub snap_to_workspace_gaps: bool, // Snapping an virtuelle Gap-Grenzen
        pub snap_distance_px: u16,
    }
    ```
    
- **Struct `WindowGroupingPolicy`**:
    
    Rust
    
    ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub struct WindowGroupingPolicy {
        pub enable_manual_grouping: bool,
        // Automatische Gruppierungsregeln (komplexer, für V2):
        // pub auto_group_by_application_id: bool,
        // pub auto_group_transients_with_parent: bool,
    }
    ```
    
- **Enum `NewWindowPlacementStrategy`**:
    
    Rust
    
    ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub enum NewWindowPlacementStrategy {
        #[default]
        Smart,    // Versucht intelligent zu platzieren (z.B. nicht überlappend, im größten freien Bereich)
        Center,   // Zentriert auf dem Bildschirm/Workspace
        Cascade,  // Kaskadierend vom letzten Fenster
        UnderMouse, // Unter dem Mauszeiger (falls zutreffend)
    }
    ```
    
- **Enum `FocusStealingPreventionLevel`**:
    
    Rust
    
    ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub enum FocusStealingPreventionLevel {
        None,   // Jedes Fenster darf Fokus anfordern
        #[default]
        Moderate, // Verhindert aggressives Stehlen, erlaubt aber legitime Anforderungen
        Strict, // Nur explizite Benutzeraktion kann Fokus ändern
    }
    ```
    
- **Struct `FocusPolicy`**:
    
    Rust
    
    ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub struct FocusPolicy {
        pub focus_follows_mouse: bool, // Mausbewegung ändert Fokus
        pub click_to_focus: bool,      // Klick erforderlich
        pub focus_new_windows_on_creation: bool, // Neue Fenster erhalten sofort Fokus
        pub focus_new_windows_on_workspace_switch: bool, // Beim Workspace-Wechsel wird das zuletzt fokussierte Fenster des Ziel-WS fokussiert
        pub focus_stealing_prevention: FocusStealingPreventionLevel,
    }
    ```
    
- **Struct `WindowPolicyOverrides`** (pro Fenster, optional, von Benutzer oder Regeln setzbar):
    
    Rust
    
    ```
    use crate::core::types::RectInt; // Aus der Kernschicht
    use uuid::Uuid;
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    pub struct WindowPolicyOverrides {
        pub preferred_tiling_mode: Option<TilingMode>,
        pub is_always_floating: Option<bool>, // Überschreibt Workspace-Tiling-Modus
        pub fixed_size: Option<(u32, u32)>, // Breite, Höhe
        pub fixed_position: Option<(i32, i32)>, // x, y (relativ zum Workspace)
        pub prevent_focus_stealing: Option<bool>, // Individuelle Überschreibung der globalen Policy
        pub min_size_override: Option<(u32, u32)>,
        pub max_size_override: Option<(u32, u32)>,
    }
    ```
    
    - Dieses Struct würde nicht direkt in `GlobalDesktopSettings` sein, sondern dynamisch pro Fenster verwaltet (z.B. in einer `HashMap<WindowIdentifier, WindowPolicyOverrides>`).
- **Struct `WorkspaceWindowLayout`**: Repräsentiert das berechnete Layout für einen Workspace.
    
    Rust
    
    ```
    use crate::core::types::RectInt;
    use crate::domain::workspaces::core::types::WindowIdentifier;
    use std::collections::HashMap;
    
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct WorkspaceWindowLayout {
        // Fenster-ID zu seiner berechneten Geometrie (Position und Größe)
        pub window_geometries: HashMap<WindowIdentifier, RectInt>,
        // Bereich, der vom Layout genutzt wird (kann kleiner sein als available_space, z.B. bei zentrierten Layouts)
        pub occupied_area: Option<RectInt>,
    }
    ```
    

**Datei:** `src/window_management_policy/errors.rs`

- **Enum `WindowPolicyError`**:
    
    Rust
    
    ```
    use thiserror::Error;
    use crate::domain::workspaces::core::types::WorkspaceId;
    
    #[derive(Debug, Error)]
    pub enum WindowPolicyError {
        #[error("Failed to calculate layout for workspace '{workspace_id}': {reason}")]
        LayoutCalculationError { workspace_id: WorkspaceId, reason: String },
        #[error("Invalid window management policy configuration: {setting_path}, reason: {reason}")]
        InvalidPolicyConfiguration { setting_path: String, reason: String },
        #[error("Window referenced by identifier '{0:?}' not found for policy application.")]
        WindowNotFoundForPolicy(crate::domain::workspaces::core::types::WindowIdentifier),
        #[error("An internal error occurred in window management policy: {0}")]
        InternalError(String),
    }
    ```
    

**Datei:** `src/window_management_policy/service.rs` (oder `mod.rs`)

- **Trait `WindowManagementPolicyService`**:
    
    Rust
    
    ```
    use async_trait::async_trait;
    use std::collections::HashMap;
    use crate::core::types::{RectInt, Size, Point};
    use crate::domain::workspaces::core::types::{WorkspaceId, WindowIdentifier, WorkspaceLayoutType};
    use super::types::{TilingMode, GapSettings, WindowSnappingPolicy, NewWindowPlacementStrategy, WorkspaceWindowLayout, WindowPolicyOverrides};
    use super::errors::WindowPolicyError;
    use crate::domain::global_settings_and_state_management::types::GlobalDesktopSettings; // Für den Zugriff auf globale Policies
    
    #[async_trait]
    pub trait WindowManagementPolicyService: Send + Sync {
        /// Berechnet das Layout für alle Fenster auf einem gegebenen Workspace.
        /// `windows_on_workspace`: Liste der Fenster-IDs auf dem Workspace und ihre aktuellen (oder gewünschten Mindest-)Größen.
        /// `workspace_tiling_mode`: Der vom Workspace gewünschte Tiling-Modus.
        /// `global_settings`: Aktuelle globale Einstellungen, die Policies enthalten.
        /// `window_specific_overrides`: Map von Fenster-IDs zu ihren spezifischen Policy-Überschreibungen.
        async fn calculate_workspace_layout(
            &self,
            workspace_id: WorkspaceId,
            windows_on_workspace: &[(WindowIdentifier, Size<u32>)], // Fenster und ihre (Mindest-)Größen
            available_area: RectInt, // Der für Fenster verfügbare Bereich auf dem Workspace
            workspace_tiling_mode: TilingMode, // Vom Workspace Manager festgelegter Modus
            gap_settings: &GapSettings, // Aktuelle Gap-Einstellungen
            window_specific_overrides: &HashMap<WindowIdentifier, WindowPolicyOverrides>
        ) -> Result<WorkspaceWindowLayout, WindowPolicyError>;
    
        /// Bestimmt die initiale Geometrie und den Zustand für ein neues Fenster.
        async fn get_initial_window_geometry(
            &self,
            window_id: &WindowIdentifier,
            requested_size: Option<Size<u32>>, // Vom Fenster gewünschte Größe
            is_transient_for: Option<&WindowIdentifier>, // Elternfenster für transiente Fenster
            workspace_id: WorkspaceId,
            active_layout_on_workspace: &WorkspaceWindowLayout, // Aktuelles Layout des Ziel-Workspace
            available_area: RectInt,
            placement_strategy: NewWindowPlacementStrategy,
            window_specific_overrides: &Option<WindowPolicyOverrides>
        ) -> Result<RectInt, WindowPolicyError>;
    
        /// Berechnet ein potenzielles "Snap"-Ziel für ein sich bewegendes oder größenveränderndes Fenster.
        async fn calculate_snap_target(
            &self,
            moving_window_id: &WindowIdentifier,
            current_geometry: RectInt, // Aktuelle Geometrie des bewegten Fensters
            other_windows_on_workspace: &[(&WindowIdentifier, &RectInt)], // Andere Fenster
            workspace_area: RectInt, // Gesamtbereich des Workspace
            snapping_policy: &WindowSnappingPolicy,
            gap_settings: &GapSettings
        ) -> Option<RectInt>;
    
        /// Gibt die Fokus-Policy zurück.
        async fn get_focus_policy(&self, global_settings: &GlobalDesktopSettings) -> FocusPolicy;
    
        /// Gibt die Policy für neue Fensterplatzierung zurück.
        async fn get_new_window_placement_strategy(&self, global_settings: &GlobalDesktopSettings) -> NewWindowPlacementStrategy;
    
        // Ggf. Methoden zum Abrufen spezifischer Policy-Objekte wie GapSettings, WindowSnappingPolicy etc.
        // basierend auf global_settings oder Workspace-spezifischen Einstellungen.
    }
    ```
    
- **Implementierung `DefaultWindowManagementPolicyService`**:
    - **Konstruktor:** `pub fn new(settings_service: Arc<dyn GlobalSettingsService>) -> Self`
        - Hält eine Referenz (`Arc`) zum `GlobalSettingsService`, um bei Bedarf aktuelle Policy-Einstellungen abzurufen.
    - **`calculate_workspace_layout` Logik:**
        1. Ruft globale Einstellungen (Tiling-Modus-Präferenz, Gaps) vom `settings_service` ab (oder erhält sie als Parameter).
        2. Filtert Fenster heraus, die `is_always_floating == Some(true)` haben. Diese werden ignoriert für Tiling.
        3. Basierend auf dem `workspace_tiling_mode` (und ggf. Fenster-spezifischen `preferred_tiling_mode` Overrides):
            - `TilingMode::Manual`: Gibt aktuelle Geometrien zurück (oder platziert sie initial gemäß `NewWindowPlacementStrategy`).
            - `TilingMode::Columns`: Teilt `available_area` (unter Berücksichtigung von `gap_settings`) vertikal für jedes nicht-floating Fenster auf. Berücksichtigt `min_size_override`.
            - `TilingMode::Rows`: Teilt horizontal auf.
            - `TilingMode::Spiral`: Implementiert Fibonacci-Spiral-Layout-Algorithmus.
            - `TilingMode::MaximizedFocused`: Macht das "aktive" Fenster (muss als Parameter übergeben werden oder heuristisch bestimmt werden) bildschirmfüllend (innerhalb `available_area` minus Gaps), andere minimiert/versteckt.
        4. Floating-Fenster werden über den gekachelten Fenstern platziert, ihre Positionen bleiben relativ erhalten oder werden initial gemäß `get_initial_window_geometry` platziert.
        5. Erstellt und gibt `WorkspaceWindowLayout` zurück.
    - **`get_initial_window_geometry` Logik:**
        1. Berücksichtigt `requested_size`, `is_transient_for` (z.B. zentriert über Parent).
        2. Wendet `placement_strategy` an (`Smart` könnte versuchen, Überlappungen mit `active_layout_on_workspace` zu vermeiden).
        3. Berücksichtigt `window_specific_overrides.fixed_position` und `fixed_size`.
    - **`calculate_snap_target` Logik:**
        1. Prüft Distanz zu Bildschirmrändern, Gap-Grenzen und Kanten/Mittelpunkten anderer Fenster.
        2. Wenn innerhalb `snap_distance_px`, gibt die neue "eingerastete" `RectInt` zurück.

**Abhängigkeiten & Interaktion:**

- Liest Policy-Konfigurationen von `domain::global_settings_service`.
- Wird von der Systemschicht (`system::window_mechanics`) aufgerufen, um Layouts und Platzierungen zu bestimmen.
- Könnte von `domain::workspaces::manager` Events abonnieren (z.B. `WindowAddedToWorkspaceEvent`), um Layouts proaktiv neu zu berechnen.

**Implementierungsschritte:**

1. `types.rs` und `errors.rs` definieren.
2. `service.rs`: `WindowManagementPolicyService`-Trait definieren.
3. `DefaultWindowManagementPolicyService` implementieren:
    - Konstruktor.
    - Implementierung der Layout-Algorithmen (Columns, Rows, Spiral etc.) als private Hilfsfunktionen.
    - Implementierung der Snapping-Logik.
    - Implementierung der öffentlichen Trait-Methoden.
4. Unit-Tests für jeden Layout-Algorithmus, Snapping-Logik und die Service-Methoden mit verschiedenen Szenarien und Konfigurationen.

---

### Modul 7: `domain::user_centric_services`

Bestehende Spezifikation: Übernommen und integriert aus, siehe vorherige Antwort.

Anpassungen/Verfeinerungen:

#### `domain::user_centric_services::ai_interaction`

- **Strukturen/Enums:** Wie definiert in.
- **`AIInteractionLogicService` Trait & Implementierung `DefaultAIInteractionLogicService`**:
    - Methoden sind `async`.
    - `initiate_interaction`: Benötigt ggf. Zugriff auf `GlobalSettingsService` für Standardmodell oder globale KI-Einstellungen.
    - `provide_consent`: Muss den `AIConsentStatus` im `AIInteractionContext` und ggf. einen persistenten `AIConsent`-Eintrag aktualisieren.
    - `store_consent` / `get_all_user_consents`: Delegieren die Persistenz an einen `AIConsentProvider` (neuer Trait, ähnlich `SettingsPersistenceProvider`, interagiert mit `core::config`).
    - `load_model_profiles`: Lädt von `core::config` über einen `AIModelProfileProvider`.
- **Events:** `AIInteractionInitiatedEvent`, `AIConsentUpdatedEvent` (gesendet via `tokio::sync::broadcast`).

#### `domain::user_centric_services::notifications_core`

- **Strukturen/Enums:** Wie definiert in.
- **`NotificationService` Trait & Implementierung `DefaultNotificationService`**:
    - Methoden sind `async`.
    - `post_notification`: Berücksichtigt `DoNotDisturbModeChangedEvent` und DND-Status. Interagiert mit `domain::notifications_rules::NotificationRulesEngine` (wird als Abhängigkeit injiziert), um Benachrichtigungen vor dem Posten zu verarbeiten.
- **Events:** `NotificationPostedEvent`, `NotificationDismissedEvent`, `NotificationReadEvent`, `DoNotDisturbModeChangedEvent` (gesendet via `tokio::sync::broadcast`).

**Neue Sub-Traits/Provider für Persistenz:**

- `domain::user_centric_services::ai_interaction::persistence_iface`:
    - `AIConsentProvider`: `async fn load_consents(user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError>`, `async fn save_consent(consent: &AIConsent) -> Result<(), AIInteractionError>`.
    - `AIModelProfileProvider`: `async fn load_model_profiles() -> Result<Vec<AIModelProfile>, AIInteractionError>`.
    - Implementierungen nutzen `core::config`.

---

### Modul 8: `domain::notifications_rules`

Bestehende Spezifikation: Übernommen und integriert aus der vorherigen Antwort.

Anpassungen/Verfeinerungen:

#### `domain::notifications_rules::engine` (oder `service.rs`)

- **Struct `NotificationRulesEngine`**:
    - Konstruktor `pub async fn new(rules_provider: Arc<dyn NotificationRulesProvider>, settings_service: Arc<dyn GlobalSettingsService>) -> Self`
    - `load_rules()`-Methode, um Regeln vom `rules_provider` neu zu laden.
    - `process_notification`: Nimmt `&Notification` und gibt `Result<Option<Notification>, NotificationRulesError>` zurück (None, wenn unterdrückt, Some(modified_notification) wenn erlaubt/geändert).

#### `domain::notifications_rules::persistence_iface`

- **Trait `NotificationRulesProvider`**:
    - `async fn load_rules() -> Result<NotificationRuleSet, NotificationRulesError>`
    - `async fn save_rules(rules: &NotificationRuleSet) -> Result<(), NotificationRulesError>`
- Implementierung `FilesystemNotificationRulesProvider` nutzt `core::config`.

**Interaktion:**

- Der `notifications_core::DefaultNotificationService` wird eine Instanz des `NotificationRulesEngine` halten und `process_notification` aufrufen, bevor eine Benachrichtigung tatsächlich "gepostet" wird (d.h. bevor das `NotificationPostedEvent` ausgelöst und die Benachrichtigung den aktiven/historischen Listen hinzugefügt wird).
- Der `NotificationRulesEngine` benötigt Zugriff auf den `GlobalSettingsService`, um `SettingIsTrue`-Bedingungen auszuwerten.

---

**Finale `novade-domain/src/lib.rs` (Struktur):**

Rust

```
// novade-domain/src/lib.rs

// Kern-Abhängigkeiten (Beispiel)
// use novade_core; // Wird implizit durch Pfade wie crate::core::... genutzt

// Öffentliche Module deklarieren
pub mod common_events;
pub mod global_settings_and_state_management;
pub mod notifications_rules;
pub mod shared_types;
pub mod theming;
pub mod user_centric_services;
pub mod window_management_policy;
pub mod workspaces;

// Öffentliche API Re-Exports (Beispiele)
pub use common_events::{UserActivityDetectedEvent, SystemShutdownInitiatedEvent, /* ... */};
pub use global_settings_and_state_management::{
    GlobalSettingsService, GlobalDesktopSettings, SettingPath, GlobalSettingsError,
    SettingsPersistenceProvider, /* ... Untertypen und Enums ... */
};
pub use theming::{ThemingEngine, ThemeDefinition, AppliedThemeState, ThemingError, /* ... */};
pub use workspaces::{
    WorkspaceManager, Workspace, WorkspaceId, WindowIdentifier, WorkspaceEvent, WorkspaceLayoutType,
    WorkspaceConfigProvider, WorkspaceCoreError, WindowAssignmentError, WorkspaceManagerError, WorkspaceConfigError, /* ... */
};
pub use user_centric_services::{
    AIInteractionLogicService, NotificationService, AIInteractionContext, Notification, AIConsent, AIModelProfile,
    AIInteractionError, NotificationError, /* ... Events und andere Typen ... */
    // Ggf. auch die Provider-Traits für Persistenz
    ai_interaction::persistence_iface::{AIConsentProvider, AIModelProfileProvider},
};
pub use notifications_rules::{
    NotificationRulesEngine, NotificationRule, RuleCondition, RuleAction, NotificationRulesError,
    persistence_iface::NotificationRulesProvider,
};
pub use window_management_policy::{
    WindowManagementPolicyService, TilingMode, GapSettings, WorkspaceWindowLayout, WindowPolicyError, /* ... */
};
pub use shared_types::{ApplicationId, UserSessionState, ResourceIdentifier};

// Initialisierungsfunktion für die gesamte Domänenschicht könnte hier existieren,
// die alle Services instanziiert und ggf. miteinander verbindet.
// pub async fn initialize_domain_layer(
//     core_config_service: Arc<dyn novade_core::config::ConfigServiceAsync>,
//     // ... andere notwendige Abhängigkeiten aus der Kernschicht ...
// ) -> Result<DomainServices, DomainInitializationError> {
//     // ... Instanziiere alle Provider und Services ...
//     // Beispiel:
//     // let settings_persistence_provider = Arc::new(global_settings_and_state_management::FilesystemSettingsProvider::new(core_config_service.clone()));
//     // let settings_service = Arc::new(global_settings_and_state_management::DefaultGlobalSettingsService::new(settings_persistence_provider));
//     // settings_service.load_settings().await?;
//     // ...
//     // Ok(DomainServices { settings_service, ... })
// }

// pub struct DomainServices {
//     pub settings_service: Arc<dyn GlobalSettingsService>,
//     pub theming_engine: Arc<ThemingEngine>,
//     pub workspace_manager: Arc<WorkspaceManager>,
//     // ... andere Services
// }

// #[derive(Debug, thiserror::Error)]
// pub enum DomainInitializationError {
//     #[error("Failed to initialize settings service: {0}")]
//     SettingsInitError(#[from] global_settings_and_state_management::GlobalSettingsError),
//     // ... andere Initialisierungsfehler
// }
```

Diese umfassende Spezifikation für die Domänenschicht sollte eine sehr solide Grundlage für die Implementierung bieten. Jedes Modul ist klar abgegrenzt, und die Interaktionen sowie Abhängigkeiten sind definiert. Der nächste Schritt wäre, für jedes dieser Module die detaillierten Implementierungsschritte (wie im Beispiel für `core::types` gezeigt) und die Unit-Tests vollständig auszuarbeiten.

---

## Ultra-Feinspezifikation und Implementierungsplan: Domänenschicht (`novade-domain` Crate)

Allgemeine Entwicklungsrichtlinien für die Domänenschicht: (Siehe vorherige Antwort für eine vollständige Liste)

Kurzfassung: Rust (Edition 2021+), UI- und Systemunabhängigkeit, Fokus auf Geschäftslogik, APIs über async_trait Traits, threadsichere Zustandsverwaltung, tokio für Asynchronität und Events, thiserror für Fehler, serde für Serialisierung, Nutzung von novade-core.

**Cargo.toml für `novade-domain`:**

Ini, TOML

```
[package]
name = "novade-domain"
version = "0.1.0"
edition = "2021" # oder neuer

[dependencies]
novade-core = { path = "../novade-core" } # Pfad anpassen
thiserror = "1.0.58" # Aktuelle Version prüfen
serde = { version = "1.0.197", features = ["derive"] } # Aktuelle Version prüfen
serde_json = "1.0.115" # Aktuelle Version prüfen
uuid = { version = "1.8.0", features = ["v4", "serde"] } # Aktuelle Version prüfen
chrono = { version = "0.4.38", features = ["serde"] } # Aktuelle Version prüfen
async-trait = "0.1.79" # Aktuelle Version prüfen
tokio = { version = "1.37.0", features = ["sync", "macros", "rt-multi-thread"] } # Aktuelle Version prüfen, rt-multi-thread für broadcast ggf.
tracing = "0.1.40" # Aktuelle Version prüfen

# Optional, falls für spezifische Algorithmen benötigt
# parking_lot = "0.12" # Für Mutex/RwLock Alternativen
# im = { version = "15.1.0", features = ["serde"] } # Für persistente Datenstrukturen, falls HashMap/Vec nicht ausreichen
```

---

### Modul 1: `domain::shared_types`

Zweck: Definition von domänenspezifischen Typen, die von mehreren Domänenmodulen verwendet werden, aber nicht allgemein genug für core::types sind. Diese Typen sind oft einfache Wrapper oder Enums, die die Semantik im Domänencode verbessern.

Verantwortlichkeiten: Bereitstellung dieser gemeinsam genutzten Typen.

Design-Rationale: Zentralisierung vermeidet Duplikation und fördert Konsistenz.

**Datei:** `src/shared_types.rs`

#### 1.1. Type Alias: `ApplicationId`

- **Definition:**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    
    /// Eindeutiger Bezeichner für eine Anwendung.
    ///
    /// Repräsentiert typischerweise einen Reverse-DNS-Namen (z.B. "org.novade.FileExplorer")
    /// oder den Namen der .desktop-Datei ohne Erweiterung.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord)]
    pub struct ApplicationId(String);
    
    impl ApplicationId {
        /// Erstellt eine neue `ApplicationId`.
        ///
        /// # Panics
        /// Paniert, wenn die `id` leer ist (im Debug-Modus).
        pub fn new(id: impl Into<String>) -> Self {
            let id_str = id.into();
            debug_assert!(!id_str.is_empty(), "ApplicationId darf nicht leer sein.");
            Self(id_str)
        }
    
        /// Gibt die `ApplicationId` als String-Slice zurück.
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }
    
    impl From<String> for ApplicationId {
        fn from(s: String) -> Self {
            debug_assert!(!s.is_empty(), "ApplicationId (from String) darf nicht leer sein.");
            Self(s)
        }
    }
    
    impl From<&str> for ApplicationId {
        fn from(s: &str) -> Self {
            debug_assert!(!s.is_empty(), "ApplicationId (from &str) darf nicht leer sein.");
            Self(s.to_string())
        }
    }
    
    impl std::fmt::Display for ApplicationId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }
    ```
    
- **Zweck:** Eindeutiger Bezeichner für eine Anwendung.
- **Invarianten:** Der interne String darf nicht leer sein. Diese Invariante wird durch `debug_assert!` in `new()` und `From`-Implementierungen im Debug-Modus geprüft. Für Release-Builds wird auf die Korrektheit der Eingabe vertraut oder höhere Schichten validieren.
- **Methoden:** `new(id: impl Into<String>) -> Self`, `as_str(&self) -> &str`.
- **Trait-Implementierungen:** `From<String>`, `From<&str>`, `std::fmt::Display`.
- **Ableitungen:** `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord`.

#### 1.2. Enum: `UserSessionState`

- **Definition:**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    
    /// Repräsentiert den aktuellen Zustand der Benutzersitzung aus Sicht der Domäne.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub enum UserSessionState {
        #[default]
        Active, // Normale Benutzersitzung, Benutzer ist aktiv
        Locked, // Sitzung gesperrt (z.B. durch Bildschirmsperre)
        Idle,   // Benutzer ist für eine definierte Zeit inaktiv
    }
    ```
    
- **Zweck:** Abstraktion des Sitzungszustands.
- **Initialwert:** `Active`.
- **Ableitungen:** `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default`.

#### 1.3. Struct: `ResourceIdentifier`

- **Definition:**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    use uuid::Uuid; // Wird für das Beispiel eines Uuid-basierten IDs verwendet
    
    /// Allgemeiner Bezeichner für eine Ressource.
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct ResourceIdentifier {
        /// Der Typ der Ressource (z.B. "file", "contact", "calendar-event", "web-url").
        /// Sollte ein konsistenter, definierter Satz von Strings sein.
        pub r#type: String,
        /// Die eindeutige ID der Ressource innerhalb ihres Typs.
        /// Dies kann ein Pfad, eine URL, eine Datenbank-ID oder eine UUID sein.
        pub id: String,
        /// Optionale menschenlesbare Beschreibung oder Name der Ressource.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub label: Option<String>,
    }
    
    impl ResourceIdentifier {
        /// Erstellt einen neuen `ResourceIdentifier`.
        ///
        /// # Panics
        /// Paniert, wenn `resource_type` oder `resource_id` leer sind (im Debug-Modus).
        pub fn new(resource_type: impl Into<String>, resource_id: impl Into<String>, label: Option<String>) -> Self {
            let type_str = resource_type.into();
            let id_str = resource_id.into();
            debug_assert!(!type_str.is_empty(), "ResourceIdentifier type darf nicht leer sein.");
            debug_assert!(!id_str.is_empty(), "ResourceIdentifier id darf nicht leer sein.");
            Self {
                r#type: type_str,
                id: id_str,
                label,
            }
        }
    
        /// Erstellt einen `ResourceIdentifier` für eine Datei.
        pub fn file(path: impl Into<String>, label: Option<String>) -> Self {
            Self::new("file", path, label)
        }
    
        /// Erstellt einen `ResourceIdentifier` für eine URL.
        pub fn url(url_str: impl Into<String>, label: Option<String>) -> Self {
            Self::new("web-url", url_str, label)
        }
    
        /// Erstellt einen `ResourceIdentifier` mit einer generierten UUID.
        pub fn new_uuid(resource_type: impl Into<String>, label: Option<String>) -> Self {
            Self::new(resource_type, Uuid::new_v4().to_string(), label)
        }
    }
    ```
    
- **Zweck:** Allgemeiner, typisierter Bezeichner für Ressourcen.
- **Felder:**
    - `r#type: String` (öffentlich)
    - `id: String` (öffentlich)
    - `label: Option<String>` (öffentlich, optional)
- **Invarianten:** `r#type` und `id` dürfen nicht leer sein (geprüft via `debug_assert!`).
- **Methoden:** `new(...)`, `file(...)`, `url(...)`, `new_uuid(...)`.
- **Ableitungen:** `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`.

#### 1.4. Implementierungsschritte `domain::shared_types`

1. **Datei erstellen:** `novade-domain/src/shared_types.rs`.
2. **Typen definieren:** `ApplicationId`, `UserSessionState`, `ResourceIdentifier` wie oben spezifiziert, inklusive aller Methoden, Trait-Implementierungen und `serde`-Attributen.
3. **Unit-Tests erstellen (`novade-domain/src/shared_types.rs` -> `#[cfg(test)] mod tests { ... }`):**
    - Für `ApplicationId`:
        - `test_application_id_new()`: Erstellung, `as_str()`.
        - `test_application_id_from_string()`: Konvertierung.
        - `test_application_id_from_str()`: Konvertierung.
        - `test_application_id_display()`: `Display`-Trait.
        - `test_application_id_serde()`: Serialisierung zu JSON und Deserialisierung.
        - `#[should_panic]` (im Debug-Modus) für `ApplicationId::new("")`.
    - Für `UserSessionState`:
        - `test_user_session_state_default()`: Prüft `Active` als Default.
        - `test_user_session_state_serde()`: Serialisierung/Deserialisierung.
    - Für `ResourceIdentifier`:
        - `test_resource_identifier_new()`: Korrekte Erstellung.
        - `test_resource_identifier_file_url_uuid()`: Hilfskonstruktoren.
        - `test_resource_identifier_serde()`: Serialisierung/Deserialisierung (auch mit `Option<String>` für Label).
        - `#[should_panic]` (im Debug-Modus) für `ResourceIdentifier::new("", "id", None)` und `ResourceIdentifier::new("type", "", None)`.
4. **`novade-domain/src/lib.rs` anpassen:**
    
    Rust
    
    ```
    // In novade-domain/src/lib.rs
    pub mod shared_types;
    // Re-export für einfacheren Zugriff von anderen Crates/Modulen
    pub use shared_types::{ApplicationId, UserSessionState, ResourceIdentifier};
    ```
    

---

### Modul 2: `domain::common_events`

Zweck: Definition von Event-Typen, die von mehreren Domänenmodulen ausgelöst oder konsumiert werden können oder die als generische Payloads dienen.

Design-Rationale: Fördert lose Kopplung und eine klare Ereignis-basierte Architektur. Events sind Datenstrukturen, die Zustandsänderungen oder signifikante Vorkommnisse repräsentieren.

Event-Mechanismus: Es wird tokio::sync::broadcast für die Verteilung dieser Events angenommen, wo ein globaler oder Service-spezifischer broadcast::Sender verwendet wird.

**Datei:** `src/common_events.rs`

#### 2.1. Enum: `UserActivityType`

- **Definition:**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    
    /// Kategorisiert die Art einer erkannten Benutzeraktivität.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum UserActivityType {
        MouseMoved,
        MouseClicked,
        MouseWheelScrolled,
        KeyPressed,
        TouchInteraction, // Generisch für Touch-Events
        WorkspaceSwitched,
        ApplicationFocused,
        WindowOpened,
        WindowClosed,
        // Weitere spezifische Aktivitätstypen nach Bedarf
    }
    ```
    
- **Zweck:** Granulare Unterscheidung von Benutzeraktivitäten für verschiedene Zwecke (z.B. Idle-Detection, kontextuelle Aktionen).
- **Ableitungen:** `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize`.

#### 2.2. Struct: `UserActivityDetectedEvent`

- **Definition:**
    
    Rust
    
    ```
    use chrono::{DateTime, Utc};
    use serde::{Serialize, Deserialize};
    use super::shared_types::{UserSessionState, ApplicationId}; // Pfad anpassen
    use super::UserActivityType; // Aus demselben Modul
    use uuid::Uuid;
    
    /// Wird ausgelöst, wenn eine signifikante Benutzeraktivität im System erkannt wird.
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct UserActivityDetectedEvent {
        /// Eindeutige ID des Events.
        pub event_id: Uuid,
        /// Zeitstempel der Aktivitätserkennung.
        pub timestamp: DateTime<Utc>,
        /// Art der erkannten Aktivität.
        pub activity_type: UserActivityType,
        /// Der Sitzungszustand des Benutzers zum Zeitpunkt der Aktivität.
        pub current_session_state: UserSessionState,
        /// Optional: ID der Anwendung, die im Fokus war oder die Aktivität ausgelöst hat.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub active_application_id: Option<ApplicationId>,
        /// Optional: ID des Workspaces, auf dem die Aktivität stattfand.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub active_workspace_id: Option<crate::workspaces::core::types::WorkspaceId>, // Pfad anpassen
        // Zukünftig: Optionale, spezifischere Daten zum Event, z.B. welcher Key gedrückt wurde (mit Bedacht auf PII)
        // pub details: Option<serde_json::Value>,
    }
    
    impl UserActivityDetectedEvent {
        pub fn new(
            activity_type: UserActivityType,
            current_session_state: UserSessionState,
            active_application_id: Option<ApplicationId>,
            active_workspace_id: Option<crate::workspaces::core::types::WorkspaceId> // Pfad anpassen
        ) -> Self {
            Self {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                activity_type,
                current_session_state,
                active_application_id,
                active_workspace_id,
            }
        }
    }
    ```
    
- **Zweck:** Zentrales Event zur Signalisierung von Benutzeraktivität.
- **Payload:**
    - `event_id: Uuid` (öffentlich): Eindeutige ID für das Event selbst.
    - `timestamp: DateTime<Utc>` (öffentlich)
    - `activity_type: UserActivityType` (öffentlich)
    - `current_session_state: UserSessionState` (öffentlich)
    - `active_application_id: Option<ApplicationId>` (öffentlich, optional)
    - `active_workspace_id: Option<WorkspaceId>` (öffentlich, optional, Typ aus `domain::workspaces`)
- **Typische Publisher:** Ein Adapter in der Systemschicht, der rohe Input-Events von `system::input` konsumiert und aggregiert, oder spezifische Domänendienste wie `domain::workspaces::manager` bei einem Workspace-Wechsel.
- **Typische Subscriber:**
    - `domain::user_centric_services::ai_interaction` (z.B. für Reset von Idle-Timern für KI-Kontext-Timeouts).
    - `domain::power_management_policy` (für System-Idle-Detection und Auslösen von Energiesparmaßnahmen).
    - UI-Komponenten, die auf Benutzeraktivität reagieren (z.B. "Zuletzt aktiv"-Anzeigen, obwohl dies eher UI-Zustand ist).
    - Logging/Auditing-Systeme.

#### 2.3. Enum: `ShutdownReason`

- **Definition:**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    
    /// Definiert den Grund für ein System-Shutdown oder einen Neustart.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub enum ShutdownReason {
        #[default]
        UserRequest,     // Direkte Anforderung durch den Benutzer (z.B. über Menü)
        PowerButtonPress,  // Physischer Power-Button wurde gedrückt
        LowBattery,        // Kritischer Batteriestand erfordert Shutdown
        SystemUpdate,      // Shutdown/Neustart aufgrund eines Systemupdates
        ApplicationRequest,// Eine Anwendung hat einen Shutdown angefordert (selten, braucht spezielle Rechte)
        OsError,           // Kritischer OS-Fehler erfordert Neustart (hypothetisch für Domäne)
        Unknown,           // Unbekannter Grund
    }
    ```
    
- **Zweck:** Klare Angabe des Grundes für einen Shutdown.
- **Initialwert:** `UserRequest`.
- **Ableitungen:** `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default`.

#### 2.4. Struct: `SystemShutdownInitiatedEvent`

- **Definition:**
    
    Rust
    
    ```
    use chrono::{DateTime, Utc};
    use serde::{Serialize, Deserialize};
    use super::ShutdownReason; // Aus demselben Modul
    use uuid::Uuid;
    
    /// Wird ausgelöst, wenn der Prozess des Herunterfahrens oder Neustarts des Systems initiiert wird.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct SystemShutdownInitiatedEvent {
        pub event_id: Uuid,
        pub timestamp: DateTime<Utc>,
        /// Der Grund für den Shutdown.
        pub reason: ShutdownReason,
        /// Gibt an, ob es sich um einen Neustart (`true`) oder ein Herunterfahren (`false`) handelt.
        pub is_reboot: bool,
        /// Optionale Verzögerung in Sekunden, bevor der eigentliche Shutdown/Neustart ausgeführt wird.
        /// Dies gibt Anwendungen Zeit, ihre Daten zu speichern.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub delay_seconds: Option<u32>,
        /// Optionale Nachricht, die dem Benutzer angezeigt werden könnte.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub message: Option<String>,
    }
    
    impl SystemShutdownInitiatedEvent {
        pub fn new(reason: ShutdownReason, is_reboot: bool, delay_seconds: Option<u32>, message: Option<String>) -> Self {
            Self {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                reason,
                is_reboot,
                delay_seconds,
                message,
            }
        }
    }
    ```
    
- **Zweck:** Signalisiert bevorstehenden System-Shutdown/Neustart.
- **Payload:**
    - `event_id: Uuid` (öffentlich)
    - `timestamp: DateTime<Utc>` (öffentlich)
    - `reason: ShutdownReason` (öffentlich)
    - `is_reboot: bool` (öffentlich): `true` für Neustart, `false` für Herunterfahren.
    - `delay_seconds: Option<u32>` (öffentlich, optional)
    - `message: Option<String>` (öffentlich, optional)
- **Typische Publisher:** Ein Adapter in der Systemschicht, der auf Signale von `logind` (z.B. `PrepareForShutdown(true/false)`) reagiert.
- **Typische Subscriber:**
    - Alle Domänendienste, die einen sauberen Shutdown-Prozess benötigen, um Zustände zu speichern (z.B. `GlobalSettingsService::save_settings`, `WorkspaceManager::save_configuration`, `ThemingEngine` falls er Caches persistiert, `AIInteractionLogicService` für `AIConsent`).
    - Die Systemschicht selbst, um z.B. Anwendungen über XDG Portals oder andere Mechanismen zu benachrichtigen.
    - Die UI-Schicht, um einen Shutdown-Dialog anzuzeigen.

#### 2.5. Implementierungsschritte `domain::common_events`

1. **Datei erstellen:** `novade-domain/src/common_events.rs`.
2. **Typen definieren:** `UserActivityType`, `UserActivityDetectedEvent`, `ShutdownReason`, `SystemShutdownInitiatedEvent` wie oben spezifiziert, inklusive aller Methoden und `serde`-Attribute.
3. **`novade-domain/src/lib.rs` anpassen:**
    
    Rust
    
    ```
    // In novade-domain/src/lib.rs
    pub mod common_events;
    // Re-export für einfacheren Zugriff
    pub use common_events::{
        UserActivityType, UserActivityDetectedEvent,
        ShutdownReason, SystemShutdownInitiatedEvent,
    };
    ```
    
4. **Unit-Tests erstellen (`novade-domain/src/common_events.rs` -> `#[cfg(test)] mod tests { ... }`):**
    - Für `UserActivityDetectedEvent`:
        - `test_user_activity_event_new()`: Korrekte Initialisierung von `event_id` und `timestamp`.
        - `test_user_activity_event_serde()`: Serialisierung/Deserialisierung.
    - Für `SystemShutdownInitiatedEvent`:
        - `test_system_shutdown_event_new()`: Korrekte Initialisierung.
        - `test_system_shutdown_event_serde()`: Serialisierung/Deserialisierung.
    - Für Enums: Teste `Default`-Implementierung und Serialisierung/Deserialisierung.

---

### Modul 3: `domain::theming`

Bestehende Spezifikation: und vorherige Antwort.

Verantwortlichkeiten: Logik für Erscheinungsbild, Design-Token-Verwaltung, dynamische Theme-Wechsel.

#### Verfeinerungen und Ergänzungen:

**3.1. `domain::theming::types` (`src/theming/types.rs`)**

- **`TokenIdentifier`**:
    - **Validierung in `new()`**: `debug_assert!(!id_str.is_empty() && id_str.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'), "TokenIdentifier darf nur ASCII-Alphanumerisch, Punkte und Bindestriche enthalten und nicht leer sein.");`
- **`TokenValue`**:
    - **`Reference(TokenIdentifier)`**: Stellt sicher, dass Alias-Tokens klar definiert sind.
    - **Validierung**: Die String-Werte für `Color`, `Dimension` etc. sollten idealerweise bei der Erstellung oder beim Parsen einer Basisvalidierung unterzogen werden (z.B. Hex-Format für Farben, Suffix "px"/"rem" für Dimensionen). Dies kann durch spezifische Newtype-Wrapper oder Validierungsfunktionen in der Logikschicht erfolgen. Für die `TokenValue` selbst bleiben es Strings, die Validierung erfolgt in der `TokenResolutionPipeline`.
- **`AccentColor`**:
    - **`value: novade_core::types::Color`**: Anstatt `String` direkt den `Color`-Typ aus der Kernschicht verwenden. Dies erfordert, dass `Color` `serde` mit `from_hex`/`to_hex_string` implementiert.
        - **Konsequenz:** Die `from_hex`-Logik und `ColorParseError` aus `core::types::color` bzw. `core::errors` wird hier relevant.
- **`AppliedThemeState`**:
    - `resolved_tokens: std::collections::BTreeMap<TokenIdentifier, String>`: `BTreeMap` statt `HashMap` verwenden, um eine deterministische Reihenfolge der Tokens zu gewährleisten, was für Tests und das Debugging von generiertem CSS nützlich sein kann. `TokenIdentifier` muss `Ord` implementieren.
- **`ThemingConfiguration`**:
    - `selected_accent_color: Option<novade_core::types::Color>`: Verwendet `core::types::Color`.

**3.2. `domain::theming::errors` (`src/theming/errors.rs`)**

- **`ThemingError`**:
    - `InvalidTokenValue { token_id: TokenIdentifier, value_string: String, reason: String }`: Neuer Fehler für ungültige Werte innerhalb eines `RawToken` nach dem Parsen, aber vor der Auflösung.
    - `AccentColorApplicationError { theme_id: ThemeIdentifier, accent_color: novade_core::types::Color, details: String }`: Spezifischer Fehler für die Akzentfarbenanwendung.

**3.3. `domain::theming::logic` (oder `engine_internal`) (`src/theming/logic.rs`)**

- **Token- und Theme-Laden (`async fn load_and_validate_token_files`, `async fn load_and_validate_theme_files`):**
    - Nutzen `novade_core::config::ConfigServiceAsync` (hypothetischer Trait) für asynchronen Dateizugriff. Die Implementierung dieses Traits in `novade-core` würde `tokio::fs` verwenden.
    - Beim Parsen von `RawToken.value` (z.B. `TokenValue::Color(s)`) könnte eine erste Basisvalidierung des `s` erfolgen.
- **Token Resolution Pipeline (`resolve_tokens_for_config`):**
    - **Akzentfarben-Logik:**
        1. Die `ThemeDefinition` sollte definieren, welche Tokens "akzentfähig" sind und wie sie modifiziert werden (z.B. eine Liste von `(TokenIdentifier, AccentModificationType)` wobei `AccentModificationType` `DirectReplace`, `Lighten(f32)`, `Darken(f32)` sein könnte).
        2. Wenn `config.selected_accent_color` gesetzt ist:
            - Iteriere über die akzentfähigen Tokens des Themes.
            - `DirectReplace`: Der Wert des akzentfähigen Tokens (z.B. `token.system.accent.primary`) wird direkt auf den Wert der `selected_accent_color` gesetzt.
            - `Lighten/Darken`: Der Basiswert des akzentfähigen Tokens wird mit der `selected_accent_color` als Referenz aufgehellt/abgedunkelt (erfordert Farbmodifikationslogik, die ggf. `novade_core::types::Color` Methoden nutzt).
    - **Caching-Schlüssel für `resolved_state_cache`**:
        - `CacheKey(ThemeIdentifier, ColorSchemeType, Option<novade_core::types::Color>, u64)`: `AccentColor` direkt (da `Color` hashbar sein kann, wenn f32-Felder mit einer Wrapper-Struct gehasht werden, die Bit-Repräsentationen vergleicht oder eine feste Präzision annimmt – einfacher ist, `color.to_hex_string(true)` zu hashen oder zu verwenden). Für `u64` den Hash der `custom_user_token_overrides` verwenden.
- **Fallback-Theme Laden (`load_fallback_theme_definition`):**
    - JSON-Strings für Fallback-Theme und -Tokens werden mittels `include_str!("default_themes/fallback.theme.json")` etc. einkompiliert.

**3.4. `ThemingEngine` Service (`src/theming/service.rs`)**

- **`ThemingEngineInternalState`**:
    - `event_sender: tokio::sync::broadcast::Sender<ThemeChangedEvent>`: Wird korrekt initialisiert.
    - `config_service: Arc<dyn novade_core::config::ConfigServiceAsync>`: Wird injiziert für das Laden von Dateien.
- **Methoden:**
    - `new(...)`: Benötigt `Arc<dyn novade_core::config::ConfigServiceAsync>`.
    - `reload_themes_and_tokens()`: Nutzt den injizierten `config_service` für asynchrones Neuladen.
    - Event-Versand: `if self.event_sender.send(event).is_err() { tracing::warn!("ThemingEngine: Keine aktiven Subscriber für ThemeChangedEvent vorhanden."); }`

**3.5. Detaillierte Implementierungsschritte `domain::theming`**

1. **Grundgerüst schaffen:** Verzeichnisstruktur anlegen, `Cargo.toml`-Abhängigkeiten prüfen/ergänzen.
2. **`types.rs` implementieren:**
    - `TokenIdentifier`, `TokenValue`, `RawToken`, `TokenSet` definieren.
    - `ThemeIdentifier`, `ColorSchemeType`, `AccentColor` (mit `core::types::Color`), `ThemeVariantDefinition`, `ThemeDefinition` definieren.
    - `AppliedThemeState` (mit `BTreeMap`), `ThemingConfiguration` (mit `core::types::Color`) definieren.
    - Alle notwendigen `derive`s (`Debug`, `Clone`, `PartialEq`, `Serialize`, `Deserialize`, `Default`, `Ord`, `Hash` wo sinnvoll) und `serde`-Attribute hinzufügen.
    - Unit-Tests für Serialisierung/Deserialisierung und `Default`-Werte schreiben.
3. **`errors.rs` implementieren:**
    - `ThemingError`-Enum mit allen Varianten und `thiserror`-Attributen.
    - Unit-Tests für `Display`-Format und `source()`-Verhalten.
4. **`logic.rs` (oder Submodule) implementieren:**
    - `load_and_validate_token_files_async`: Liest JSON-Dateien (via `config_service`), parst zu `TokenSet`, validiert (Duplikate, Basis-Format von Werten).
    - `load_and_validate_theme_files_async`: Liest JSON, parst zu `ThemeDefinition`, validiert (Token-Referenzen).
    - `validate_tokenset_for_cycles`: Implementiert Zyklenerkennung.
    - `resolve_tokens_for_config`: Implementiert die vollständige Pipeline (Basis, Variante, Akzent, Overrides, Referenzauflösung).
        - Private Hilfsfunktionen für jeden Schritt der Pipeline.
        - Funktion zur Anwendung von Akzentfarben (ggf. mit `Lighten`/`Darken`-Logik).
    - `generate_fallback_applied_state`: Lädt einkompilierte JSONs und erzeugt daraus einen minimalen `AppliedThemeState`.
    - Unit-Tests für jeden Logik-Schritt und Fehlerfall.
5. **`service.rs` (oder `mod.rs`) `ThemingEngine` implementieren:**
    - `ThemingEngineInternalState`-Struktur definieren.
    - `ThemingEngine`-Struktur mit `Arc<Mutex<ThemingEngineInternalState>>` und `broadcast::Sender`.
    - `new()`-Konstruktor: Initialisiert `event_sender`, `config_service`, lädt initial Themes/Tokens, berechnet ersten `applied_state` (oder Fallback), füllt Cache.
    - `get_current_theme_state()`, `get_available_themes()`, `get_current_configuration()`: Implementieren Klonen aus dem internen Zustand.
    - `update_configuration()`: Berechnet neuen Zustand, prüft auf Änderungen, aktualisiert internen Zustand, sendet Event, aktualisiert Cache.
    - `reload_themes_and_tokens()`: Lädt neu, berechnet neu, sendet Event, invalidiert Cache.
    - `subscribe_to_theme_changes()`: Gibt `event_sender.subscribe()` zurück.
    - Unit-Tests für alle API-Methoden, Zustandsänderungen, Event-Auslösung, Cache-Verhalten (Invalidierung, Treffer). Mocking des `ConfigServiceAsync`.
6. **`default_themes/` Verzeichnis anlegen:** `fallback.theme.json` und `base.tokens.json` mit minimalen, gültigen Werten erstellen.
7. **`src/theming/mod.rs` erstellen:** Module deklarieren und öffentliche API re-exportieren (`ThemingEngine`, `ThemeChangedEvent`, wichtige Typen aus `types.rs`, `ThemingError`).

---

Die Spezifikationen für die Module `domain::global_settings_and_state_management`, `domain::workspaces`, `domain::window_management_policy`, `domain::user_centric_services` und `domain::notifications_rules` würden mit derselben Detailtiefe folgen, aufbauend auf den bereits existierenden Zitationen und den hier gezeigten Verfeinerungen und Implementierungsschritten.

**Beispielhafte Verfeinerungen für `domain::global_settings_and_state_management`:**

- **`types.rs`**:
    - Für alle Einstellungs-Unterstrukturen (z.B. `AppearanceSettings`) eine `validate(&self) -> Result<(), GlobalSettingsError>`-Methode hinzufügen, die spezifische Konsistenzprüfungen durchführt (z.B. "wenn `color_scheme == AutoSystem`, dann darf `manual_dark_mode_active` nicht `true` sein" – hypothetisch). Diese wird von `DefaultGlobalSettingsService::update_setting` nach der Deserialisierung aufgerufen.
- **`paths.rs`**:
    - Implementierung von `impl std::fmt::Display for SettingPath` für menschenlesbare Pfade in Logs/Fehlern.
    - Implementierung von `impl TryFrom<&str> for SettingPath` und `impl From<SettingPath> for String` für die Konvertierung zu/von String-basierten Pfaden (nützlich für D-Bus APIs oder Debugging).
- **`service.rs`**:
    - `DefaultGlobalSettingsService::update_setting`:
        1. Auflösen des `SettingPath` zu einem mutierbaren Verweis auf das spezifische Feld in `self.settings` (z.B. mittels Makro oder einer großen `match`-Anweisung).
        2. Deserialisieren des `serde_json::Value` in den erwarteten Typ des Feldes. `Err(GlobalSettingsError::InvalidValueType)` bei Fehlschlag.
        3. Validieren des deserialisierten Wertes mit einer spezifischen Validierungsfunktion für diese Einstellung (oder einer Methode auf der Einstellungs-Unterstruktur). `Err(GlobalSettingsError::ValidationError)` bei Fehlschlag.
        4. Wert aktualisieren, `SettingChangedEvent` senden, `save_settings()` aufrufen.

Dieser extrem granulare Ansatz ist zwar aufwendig, aber unerlässlich, um das Ziel eines "finalen lückenlosen Entwickler-Implementierungsleitfadens" zu erreichen. Jede Komponente ist bis ins Detail durchdacht, was die Implementierung erheblich beschleunigt und die Fehleranfälligkeit reduziert.


---

### Modul 3: `domain::theming`

Zweck: Logik des Erscheinungsbilds (Theming), Verwaltung von Design-Tokens, Interpretation von Theme-Definitionen, dynamische Theme-Wechsel (Farbschema, Akzentfarben). Dieses Modul stellt sicher, dass die UI-Schicht die korrekten, aufgelösten Stilwerte erhält.

Verantwortlichkeiten: Parsen von Token- und Theme-Dateien, Auflösen von Token-Referenzen und -Hierarchien, Anwenden von Benutzerkonfigurationen auf Themes, Bereitstellen des finalen AppliedThemeState und Benachrichtigen bei Änderungen.

Design-Rationale: Trennung der Theming-Logik von der UI-Darstellung. Ein Token-basiertes System ermöglicht hohe Flexibilität und Konsistenz. Dynamische Wechsel zur Laufzeit sind ein Kernziel.

Bestehende Spezifikation: und vorherige Antwort.

#### 3.1. Untermodul: `domain::theming::types`

**Datei:** `src/theming/types.rs`

- **Struct `TokenIdentifier`**
    
    - **Definition:**
        
        Rust
        
        ```
        use serde::{Serialize, Deserialize};
        use std::fmt;
        
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
        pub struct TokenIdentifier(String);
        
        impl TokenIdentifier {
            pub fn new(id: impl Into<String>) -> Self {
                let id_str = id.into();
                // Invariante: Nicht leer und nur erlaubte Zeichen.
                // Hier nur Debug-Assert, da Validierung auch beim Parsen erfolgen kann.
                debug_assert!(!id_str.is_empty(), "TokenIdentifier darf nicht leer sein.");
                debug_assert!(
                    id_str.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'),
                    "TokenIdentifier ({}) enthält ungültige Zeichen.", id_str
                );
                Self(id_str)
            }
            pub fn as_str(&self) -> &str { &self.0 }
        }
        impl fmt::Display for TokenIdentifier { /* ... */ } // Wie in [cite:656]
        impl From<&str> for TokenIdentifier { fn from(s: &str) -> Self { Self::new(s) } }
        ```
        
    - **Invarianten:** String nicht leer, enthält nur `a-zA-Z0-9.-`.
- **Enum `TokenValue`**
    
    - **Definition:**
        
        Rust
        
        ```
        // ... (andere Varianten wie in)
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        pub enum TokenValue {
            Color(String),      // CSS-kompatibler Farbwert, z.B. "#RRGGBB", "rgba(...)", "var(--andere-farbe)"
            Dimension(String),  // z.B. "16px", "2rem", "calc(100% - 20px)"
            FontSize(String),
            FontFamily(String),
            FontWeight(String), // z.B. "400", "bold"
            LineHeight(String), // z.B. "1.5", "150%"
            LetterSpacing(String),
            Border(String),     // z.B. "1px solid var(--border-color)"
            Shadow(String),
            Radius(String),
            Spacing(String),
            ZIndex(i32),
            Opacity(f64),       // Validierung: 0.0 <= opacity <= 1.0
            Text(String),
            Reference(TokenIdentifier), // Verweis auf eine andere TokenIdentifier
        }
        ```
        
    - **Invarianten:** Für `Opacity`, Wert muss zwischen 0.0 und 1.0 liegen. Die Strings in den anderen Varianten sollten gültige CSS-Werte sein (Validierung erfolgt später in der Pipeline oder bei der Anwendung).
- **Struct `RawToken`**
    
    - **Definition:** Wie in. `id: TokenIdentifier`, `value: TokenValue`, `description: Option<String>`, `group: Option<String>`.
    - **Ableitungen:** `Debug, Clone, PartialEq, Serialize, Deserialize`.
- **Typalias `TokenSet`**: `pub type TokenSet = std::collections::BTreeMap<TokenIdentifier, RawToken>;` (BTreeMap für deterministische Reihenfolge).
    
- **Struct `ThemeIdentifier`**
    
    - **Definition:** Analog zu `TokenIdentifier`.
        
        Rust
        
        ```
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
        pub struct ThemeIdentifier(String);
        // ... impls wie TokenIdentifier ...
        ```
        
    - **Invarianten:** String nicht leer, empfohlene Zeichen: `a-zA-Z0-9-`.
- **Enum `ColorSchemeType`**
    
    - **Definition:** Wie in. `Light`, `Dark`.
    - **Ableitungen:** `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default`. (`#[default]` für `Light` oder `Dark` festlegen).
- **Struct `AccentColor`**
    
    - **Definition:**
        
        Rust
        
        ```
        use novade_core::types::Color as CoreColor; // Verwendung des Kerntyps
        
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct AccentColor {
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub name: Option<String>, // z.B. "Blau", "Waldgrün"
            pub value: CoreColor,     // Der tatsächliche Farbwert
        }
        // Eq und Hash manuell implementieren, wenn CoreColor::f32 nicht direkt Eq/Hash ist.
        // Für Cache-Zwecke kann der Hex-String von CoreColor verwendet werden.
        ```
        
- **Struct `ThemeVariantDefinition`**: Wie in. `applies_to_scheme: ColorSchemeType`, `tokens: TokenSet`.
    
- **Struct `ThemeDefinition`**: Wie in. Enthält `id`, `name`, `base_tokens`, `variants`, `supported_accent_colors: Option<Vec<AccentColor>>`.
    
- **Struct `AppliedThemeState`**
    
    - **Definition:** Wie in vorheriger Antwort verfeinert (mit `BTreeMap`).
        
        Rust
        
        ```
        use std::collections::BTreeMap;
        // ...
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // Serialize/Deserialize für Caching/Events
        pub struct AppliedThemeState {
            pub theme_id: ThemeIdentifier,
            pub color_scheme: ColorSchemeType,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub active_accent_color: Option<AccentColor>,
            pub resolved_tokens: BTreeMap<TokenIdentifier, String>, // CSS-finale Werte
        }
        ```
        
- **Struct `ThemingConfiguration`**
    
    - **Definition:** Wie in vorheriger Antwort verfeinert.
        
        Rust
        
        ```
        use novade_core::types::Color as CoreColor;
        // ...
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct ThemingConfiguration {
            pub selected_theme_id: ThemeIdentifier,
            pub preferred_color_scheme: ColorSchemeType,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub selected_accent_color: Option<CoreColor>, // Verwendet CoreColor
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub custom_user_token_overrides: Option<TokenSet>,
        }
        impl Default for ThemingConfiguration { /* Sinnvolle Standardwerte, z.B. Fallback-Theme */ }
        ```
        

#### 3.2. Untermodul: `domain::theming::errors`

**Datei:** `src/theming/errors.rs`

- **Enum `ThemingError`**: Wie in vorheriger Antwort verfeinert (konsolidierte Liste aus und Ergänzungen).
    
    Rust
    
    ```
    use thiserror::Error;
    use std::path::PathBuf;
    use super::types::{TokenIdentifier, ThemeIdentifier}; // Lokale Typen
    use novade_core::types::Color as CoreColor; // Kerntyp
    
    #[derive(Debug, Error)]
    pub enum ThemingError {
        #[error("Failed to parse token file '{path}': {source}")]
        TokenFileParseError { path: PathBuf, #[source] source: serde_json::Error },
        #[error("I/O error while processing token file '{path}': {source}")]
        TokenFileIoError { path: PathBuf, #[source] source: std::io::Error },
        #[error("Invalid token data in file '{path}': {message}")]
        InvalidTokenData { path: PathBuf, message: String },
        #[error("Invalid token value for '{token_id}': '{value_string}'. Reason: {reason}")]
        InvalidTokenValue { token_id: TokenIdentifier, value_string: String, reason: String },
        #[error("Cyclic dependency detected involving token '{token_id}'. Cycle path: {cycle_path:?}")]
        CyclicTokenReference { token_id: TokenIdentifier, cycle_path: Vec<TokenIdentifier> },
        #[error("Failed to load theme definition '{theme_id}' from file '{path}': {source}")]
        ThemeFileLoadError { theme_id: ThemeIdentifier, path: PathBuf, #[source] source: serde_json::Error },
        #[error("I/O error while loading theme definition '{theme_id}' from file '{path}': {source}")]
        ThemeFileIoError { theme_id: ThemeIdentifier, path: PathBuf, #[source] source: std::io::Error },
        #[error("Invalid theme data for theme '{theme_id}' in file '{path}': {message}")]
        InvalidThemeData { theme_id: ThemeIdentifier, path: PathBuf, message: String },
        #[error("Theme with ID '{theme_id}' not found.")]
        ThemeNotFound { theme_id: ThemeIdentifier },
        #[error("Referenced token '{target_token_id}' not found (referenced by '{referencing_token_id}').")]
        MissingTokenReference { referencing_token_id: TokenIdentifier, target_token_id: TokenIdentifier },
        #[error("Maximum token reference depth ({depth}) exceeded while resolving '{token_id}'.")]
        MaxReferenceDepthExceeded { token_id: TokenIdentifier, depth: u8 },
        #[error("Failed to apply theming configuration: {message}")]
        ThemeApplicationError { message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
        #[error("Critical error: Failed to initialize theming engine because no suitable fallback theme could be loaded: {source}")]
        FallbackThemeLoadError { #[source] source: Box<dyn std::error::Error + Send + Sync + 'static> },
        #[error("Initial theming configuration is invalid: {0}")]
        InitialConfigurationError(String),
        #[error("Internal state error in ThemingEngine: {0}")]
        InternalStateError(String),
        #[error("Failed to subscribe to theme change events: {0}")]
        EventSubscriptionError(String),
        #[error("Error applying accent color '{accent_color}' to theme '{theme_id}': {details}")]
        AccentColorApplicationError { theme_id: ThemeIdentifier, accent_color: CoreColor, details: String },
        #[error("Failed to resolve token '{token_id}': {message}")]
        TokenResolutionError { token_id: TokenIdentifier, message: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
        #[error("Filesystem operation error for ThemingEngine: {0}")] // Neuer Fehler für ConfigServiceAsync-Fehler
        FilesystemError(#[from] novade_core::errors::CoreError), // Annahme: ConfigServiceAsync gibt CoreError zurück
    }
    ```
    

#### 3.3. Untermodul: `domain::theming::logic` (oder `engine_internal`)

**Datei:** `src/theming/logic.rs` (und ggf. `token_loader.rs`, `theme_loader.rs`, `token_resolver.rs`)

- **Konstante:** `const MAX_TOKEN_RESOLUTION_DEPTH: u8 = 16;`
- **Funktion: `async fn load_raw_tokens_from_file(path: &Path, config_service: &Arc<dyn novade_core::config::ConfigServiceAsync>) -> Result<TokenSet, ThemingError>`**
    1. `content = config_service.read_file_to_string(path).await.map_err(|e| ThemingError::TokenFileIoError { path: path.to_path_buf(), source: e.into_std_io_error_or_generic() })?;` (Annahme: `ConfigServiceAsync` gibt einen Fehler zurück, der in `std::io::Error` oder einen generischen `CoreError` konvertiert werden kann).
    2. `raw_tokens: Vec<RawToken> = serde_json::from_str(&content).map_err(|e| ThemingError::TokenFileParseError { path: path.to_path_buf(), source: e })?;`
    3. In `TokenSet` (`BTreeMap`) konvertieren. Bei Duplikaten: `ThemingError::InvalidTokenData`.
    4. Basisvalidierung jedes `RawToken.value` (z.B. `TokenValue::Opacity(o)` prüfen, ob `0.0 <= o <= 1.0`). Bei Fehler: `ThemingError::InvalidTokenValue`.
- **Funktion: `async fn load_and_validate_token_files(paths: &[PathBuf], config_service: &Arc<dyn novade_core::config::ConfigServiceAsync>) -> Result<TokenSet, ThemingError>`**
    1. Iteriert über `paths`, ruft `load_raw_tokens_from_file` für jede Datei auf.
    2. Mergt die `TokenSet`s (Benutzer-spezifische überschreiben System-spezifische, falls Pfade so interpretiert werden). Loggt Überschreibungen mit `tracing::debug!`.
    3. Ruft `validate_tokenset_for_cycles` für das finale Set auf.
- **Funktion: `async fn load_theme_definition_from_file(...)`**: Analog zu Tokens, parst zu `ThemeDefinition`.
- **Funktion: `async fn load_and_validate_theme_files(...)`**: Analog, validiert zusätzlich Referenzen mittels `validate_theme_definition_references`.
- **Funktion: `validate_tokenset_for_cycles(tokens: &TokenSet) -> Result<(), ThemingError>`**
    - Implementiert Tiefensuche. Verfolgt den aktuellen Pfad (`Vec<TokenIdentifier>`). Wenn ein bereits besuchtes Token im aktuellen Pfad erneut angetroffen wird -> Zyklus.
- **Funktion: `validate_theme_definition_references(theme_def: &ThemeDefinition, global_tokens: &TokenSet) -> Result<(), ThemingError>`** (Wie in).
- **Funktion: `resolve_tokens_for_config(...) -> Result<BTreeMap<TokenIdentifier, String>, ThemingError>`**
    1. **Ausgangspunkt:** Erstelle `current_resolved_tokens: BTreeMap<TokenIdentifier, TokenValue>` (noch nicht final Strings).
    2. **Basissatz:** Kopiere `global_tokens` nach `current_resolved_tokens`. Überschreibe/Merge mit `theme_def.base_tokens`.
    3. **Variante anwenden:** Finde passende `ThemeVariantDefinition` für `config.preferred_color_scheme`. Merge deren `tokens` in `current_resolved_tokens`.
    4. **Akzentfarbe anwenden:**
        - Wenn `config.selected_accent_color` (Typ `CoreColor`) vorhanden ist:
        - Iteriere über `theme_def.accentable_tokens` (neues Feld in `ThemeDefinition`: `Option<HashMap<TokenIdentifier, AccentModificationType>>`).
        - Für jeden `token_id_to_accent` und `modification_type`:
            - Hole den Basiswert des `token_id_to_accent` aus `current_resolved_tokens` (muss ein `TokenValue::Color` sein).
            - Wende `modification_type` an (z.B. `DirectReplace` -> setze auf `selected_accent_color`; `Lighten(0.2)` -> helle `selected_accent_color` um 20% auf und setze das als neuen Wert).
            - Aktualisiere `current_resolved_tokens`. Bei Fehlern: `ThemingError::AccentColorApplicationError`.
    5. **Benutzer-Overrides:** Merge `config.custom_user_token_overrides` (falls vorhanden) in `current_resolved_tokens`.
    6. **Rekursive Referenzauflösung:**
        - Iteriere über `current_resolved_tokens`. Für jedes `(id, value)`:
        - `final_value = resolve_single_token_value(id, value, &current_resolved_tokens, Vec::new(), max_depth)?`
        - Speichere `(id, final_value_as_string)` in `final_css_tokens: BTreeMap<TokenIdentifier, String>`.
        - Die `resolve_single_token_value` Funktion ist rekursiv:
            
            Rust
            
            ```
            fn resolve_single_token_value(
                original_id: &TokenIdentifier,
                current_value: &TokenValue,
                all_tokens: &BTreeMap<TokenIdentifier, TokenValue>, // Zustand vor String-Konvertierung
                visited_path: &mut Vec<TokenIdentifier>, // Für Zyklenerkennung
                max_depth: u8,
            ) -> Result<String, ThemingError> {
                if visited_path.len() > max_depth as usize {
                    return Err(ThemingError::MaxReferenceDepthExceeded { token_id: original_id.clone(), depth: max_depth });
                }
                if visited_path.contains(original_id) {
                    return Err(ThemingError::CyclicTokenReference { token_id: original_id.clone(), cycle_path: visited_path.clone() });
                }
                visited_path.push(original_id.clone());
            
                let result = match current_value {
                    TokenValue::Reference(target_id) => {
                        let target_raw_value = all_tokens.get(target_id)
                            .ok_or_else(|| ThemingError::MissingTokenReference {
                                referencing_token_id: original_id.clone(),
                                target_token_id: target_id.clone(),
                            })?;
                        // Rekursiver Aufruf für das Ziel
                        resolve_single_token_value(target_id, target_raw_value, all_tokens, visited_path, max_depth)
                    }
                    TokenValue::Color(s) => Ok(s.clone()),
                    TokenValue::Dimension(s) => Ok(s.clone()),
                    // ... andere direkte Typen zu String ...
                    TokenValue::Opacity(o) => Ok(format!("{:.2}", o.clamp(0.0, 1.0))),
                    TokenValue::ZIndex(z) => Ok(z.to_string()),
                    TokenValue::Text(s) => Ok(s.clone()),
                };
                visited_path.pop(); // Wichtig: Beim Verlassen des Rekursionsschritts entfernen
                result
            }
            ```
            
    7. Gib `final_css_tokens` zurück.
- **Caching-Logik:**
    - **Cache-Schlüssel:** `(ThemeIdentifier, ColorSchemeType, Option<String> /* hex von AccentColor */, u64 /* hash von Overrides */)`.
    - Vor der Auflösung im Cache nachsehen. Bei Treffer direkt zurückgeben.
    - Nach erfolgreicher Auflösung Ergebnis im Cache speichern.
- **Fallback-Theme Laden:**
    - `pub(crate) fn generate_fallback_applied_state() -> AppliedThemeState`: Parst einkompilierte JSONs für `fallback.theme.json` und `base.tokens.json`, führt minimale Auflösung durch (sollte keine komplexen Referenzen haben).

#### 3.4. Öffentliche API: `ThemingEngine` Service

**Datei:** `src/theming/service.rs` (oder `mod.rs`)

- **Struct `ThemingEngineInternalState`**:
    - `config_service: Arc<dyn novade_core::config::ConfigServiceAsync>` (neu)
    - Rest wie in vorheriger Antwort (Cache-Typ angepasst).
- **Struct `ThemingEngine`**:
    - `internal_state: Arc<tokio::sync::Mutex<ThemingEngineInternalState>>` (Verwendung von `tokio::sync::Mutex` für `async` Methoden).
    - `event_sender: tokio::sync::broadcast::Sender<ThemeChangedEvent>`.
- **Methoden der `ThemingEngine`**:
    - `pub async fn new(initial_config: ThemingConfiguration, theme_load_paths: Vec<PathBuf>, token_load_paths: Vec<PathBuf>, config_service: Arc<dyn novade_core::config::ConfigServiceAsync>, broadcast_capacity: usize) -> Result<Self, ThemingError>`:
        1. Initialisiert `event_sender`, `config_service_ref`.
        2. Sperrt `internal_state` (initial leer).
        3. Speichert `theme_load_paths`, `token_load_paths`, `config_service_ref` in `internal_state`.
        4. Ruft `internal_load_themes_and_tokens_locked(&mut internal_state_guard).await?` auf.
        5. Ruft `internal_apply_configuration_locked(&mut internal_state_guard, initial_config, true /* is_initial */).await?` auf.
        6. Gibt `Self` zurück.
    - `async fn internal_load_themes_and_tokens_locked(&mut self_internal: &mut ThemingEngineInternalState) -> Result<(), ThemingError>`: Interne Methode zum Neuladen.
    - `async fn internal_apply_configuration_locked(&mut self_internal: &mut ThemingEngineInternalState, config: ThemingConfiguration, is_initial: bool) -> Result<(), ThemingError>`: Interne Methode zum Anwenden einer Konfig, prüft Cache, löst Pipeline aus, aktualisiert `applied_state`, `current_config`, sendet Event. Wenn `is_initial` und Auflösung fehlschlägt, wird `generate_fallback_applied_state` verwendet.
    - `pub async fn get_current_theme_state(&self) -> AppliedThemeState`: Sperrt `internal_state`, klont und gibt `applied_state` zurück.
    - `pub async fn get_available_themes(&self) -> Vec<ThemeDefinition>`: Sperrt, klont, gibt zurück.
    - `pub async fn get_current_configuration(&self) -> ThemingConfiguration`: Sperrt, klont, gibt zurück.
    - `pub async fn update_configuration(&self, new_config: ThemingConfiguration) -> Result<(), ThemingError>`: Sperrt `internal_state`, ruft `internal_apply_configuration_locked(..., new_config, false).await`.
    - `pub async fn reload_themes_and_tokens(&self) -> Result<(), ThemingError>`: Sperrt `internal_state`, ruft `internal_load_themes_and_tokens_locked().await`, dann `internal_apply_configuration_locked(..., self_internal.current_config.clone(), false).await`. Invalidiert kompletten Cache.
    - `pub fn subscribe_to_theme_changes(&self) -> tokio::sync::broadcast::Receiver<ThemeChangedEvent>`: Gibt `self.event_sender.subscribe()` zurück.
- **Event `ThemeChangedEvent`**:
    - **Payload:** `pub new_state: AppliedThemeState`
    - **Publisher:** `ThemingEngine` (via `event_sender`).
    - **Subscriber:** `ui::theming_gtk` (wendet CSS an), `domain::global_settings_service` (wenn Theming-Einstellungen sich auf globale Nicht-Theme-Einstellungen auswirken, z.B. Kontrastmodus).

#### 3.5. Implementierungsschritte `domain::theming`

(Wie in vorheriger Antwort, aber mit `async` für Ladeoperationen und `tokio::sync::Mutex` für `ThemingEngineInternalState`.)

1. **Grundgerüst** und `Cargo.toml` aktualisieren.
2. **`types.rs`**: `AccentColor` mit `CoreColor`, `AppliedThemeState` mit `BTreeMap`, `ThemingConfiguration` mit `CoreColor`. `TokenIdentifier` Validierung.
3. **`errors.rs`**: `ThemingError` um `InvalidTokenValue`, `AccentColorApplicationError`, `FilesystemError` erweitern. `FallbackThemeLoadError` mit `#[source]`.
4. **`logic.rs`**:
    - Ladefunktionen (`load_raw_tokens_from_file`, etc.) `async` machen, `ConfigServiceAsync` nutzen.
    - `resolve_tokens_for_config`: Akzentfarben-Logik detaillieren (Nutzung von `accentable_tokens` aus `ThemeDefinition`). `MAX_TOKEN_RESOLUTION_DEPTH` verwenden. Rekursive `resolve_single_token_value` Funktion implementieren.
    - Caching-Logik mit `CacheKey` (inkl. Hash für Overrides) implementieren.
    - `generate_fallback_applied_state`: JSONs parsen und minimalen State erzeugen.
5. **`service.rs`**: `ThemingEngine` mit `tokio::sync::Mutex` für `internal_state`. `async` API-Methoden. Interne `_locked`-Methoden für die Hauptlogik. Event-Versand über `tokio::sync::broadcast`.
6. **Unit-Tests**: An `async` anpassen. Mocking des `ConfigServiceAsync`. Tests für Akzentfarben, Cache-Logik (Treffer, Fehlschlag, Invalidierung).
7. **Fallback-JSONs** in `default_themes/` erstellen.
8. **`mod.rs`**: Öffentliche API re-exportieren.

---

### Modul 4: `domain::global_settings_and_state_management`

Zweck: Repräsentation, Logik zur Verwaltung und Konsistenz globaler Desktop-Einstellungen.

Bestehende Spezifikation: und vorherige Antwort.

#### Verfeinerungen und Ergänzungen:

**4.1. `domain::global_settings::types` (`src/global_settings/types.rs`)**

- Alle Einstellungs-Structs (z.B. `AppearanceSettings`, `InputBehaviorSettings`) müssen vollständig ausdefiniert werden, inklusive aller Felder, Typen, `serde`-Attribute (`#[serde(default)]`, `#[serde(rename_all = "kebab-case")]` für TOML-Kompatibilität) und `Default`-Implementierungen.
    
    Rust
    
    ```
    // Beispiel für InputBehaviorSettings
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(default, rename_all = "kebab-case")]
    pub struct InputBehaviorSettings {
        pub mouse_acceleration_profile: MouseAccelerationProfile,
        pub custom_mouse_acceleration_factor: Option<f32>, // Validierung: 0.0 < factor
        pub mouse_sensitivity: f32, // Validierung: z.B. 0.1 - 10.0
        pub natural_scrolling_mouse: bool,
        pub natural_scrolling_touchpad: bool,
        pub tap_to_click_touchpad: bool,
        pub touchpad_pointer_speed: f32, // Validierung: z.B. 0.1 - 10.0
        pub keyboard_repeat_delay_ms: u32, // Validierung: z.B. 100-2000
        pub keyboard_repeat_rate_cps: u32, // Zeichen pro Sekunde; Validierung: z.B. 10-100
    }
    impl Default for InputBehaviorSettings { /* ... */ }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
    pub enum MouseAccelerationProfile { #[default] Adaptive, Flat, Custom }
    ```
    
- **Validierungsmethoden**: Jede Einstellungs-Unterstruktur sollte eine `pub fn validate(&self) -> Result<(), String>`-Methode (oder `Result<(), GlobalSettingsError::ValidationError>`) haben, die interne Konsistenz und Wertebereiche prüft.
    - Beispiel: `InputBehaviorSettings::validate(&self)` prüft, ob `custom_mouse_acceleration_factor` nur `Some` ist, wenn `mouse_acceleration_profile == Custom`, und ob Faktoren/Raten in gültigen Bereichen liegen.

**4.2. `domain::global_settings::paths` (`src/global_settings/paths.rs`)**

- Der `SettingPath`-Enum muss die gesamte Hierarchie von `GlobalDesktopSettings` exakt abbilden.
    
    Rust
    
    ```
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum SettingPath {
        Appearance(AppearanceSettingPath),
        WorkspaceConfig(WorkspaceSettingPath),
        InputBehavior(InputBehaviorSettingPath),
        // ...
    }
    // ... für jede Unterstruktur
    ```
    
- **Implementierung von `TryFrom<&str>` und `Display` für `SettingPath`:**
    
    Rust
    
    ```
    // Beispielhaft für einen Teilpfad
    impl fmt::Display for AppearanceSettingPath {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                AppearanceSettingPath::ActiveThemeName => write!(f, "appearance.active-theme-name"),
                AppearanceSettingPath::FontSettings(fp) => write!(f, "appearance.font-settings.{}", fp),
                // ...
            }
        }
    }
    // TryFrom<&str> erfordert sorgfältiges Parsen des Strings.
    ```
    

**4.3. `domain::global_settings::errors` (`src/global_settings/errors.rs`)**

- **`GlobalSettingsError`**:
    - `ValidationError { path: SettingPath, reason: String }`: Verwendet `SettingPath` statt `String`.
    - `PathNotFound { path: SettingPath }`.
    - `PersistenceError` sollte den spezifischen Fehler aus dem `SettingsPersistenceProvider` wrappen.

**4.4. `domain::global_settings::persistence_iface` (`src/global_settings/persistence_iface.rs`)**

- **Trait `SettingsPersistenceProvider`**:
    
    Rust
    
    ```
    #[async_trait]
    pub trait SettingsPersistenceProvider: Send + Sync {
        async fn load_global_settings(&self) -> Result<GlobalDesktopSettings, GlobalSettingsError>;
        async fn save_global_settings(&self, settings: &GlobalDesktopSettings) -> Result<(), GlobalSettingsError>;
    }
    ```
    
- **Implementierung `FilesystemSettingsProvider`**:
    - Nutzt `Arc<dyn novade_core::config::ConfigServiceAsync>`.
    - `load_global_settings`: Liest TOML-Datei, deserialisiert zu `GlobalDesktopSettings`. Bei Deserialisierungsfehlern (z.B. unbekannte Felder, wenn `deny_unknown_fields` aktiv ist, oder Typfehler), wird ein `GlobalSettingsError::DeserializationError` zurückgegeben. Wenn die Datei nicht existiert, wird `Ok(GlobalDesktopSettings::default())` zurückgegeben.
    - `save_global_settings`: Serialisiert `GlobalDesktopSettings` zu TOML, schreibt in Datei.

**4.5. `domain::global_settings::service` (`src/global_settings/service.rs`)**

- **`DefaultGlobalSettingsService`**:
    - Hält `settings: Arc<tokio::sync::RwLock<GlobalDesktopSettings>>` für threadsicheren Lese-/Schreibzugriff.
    - **`update_setting(path: SettingPath, value: serde_json::Value)`**:
        1. Holt eine Schreibsperre für `self.settings`.
        2. Erstellt einen Klon der aktuellen `settings` für die Modifikation (`let mut new_settings = (*settings_guard).clone();`).
        3. **Pfad-Navigation und Aktualisierung (komplex):**
            - Eine große `match path { ... }`-Anweisung, die für jede `SettingPath`-Variante:
                - Das entsprechende Feld in `new_settings` referenziert.
                - `serde_json::from_value::<TargetType>(value)` versucht. Bei Fehler -> `InvalidValueType`.
                - Den deserialisierten Wert in `new_settings` setzt.
        4. `new_settings.validate_recursive() -> Result<(), GlobalSettingsError::ValidationError>` aufrufen (eine Methode, die alle `validate()`-Methoden der Unterstrukturen aufruft). Bei Fehler, Änderung nicht anwenden, Fehler zurückgeben.
        5. Wenn Validierung erfolgreich: Ersetze `*settings_guard = new_settings;`.
        6. `self.event_sender.send(SettingChangedEvent { path, new_value: value /* oder serialisierter neuer Wert */})`.
        7. `self.save_settings_internal(settings_guard).await` (interne Methode, die die Sperre nutzt).
    - **`get_setting(path: &SettingPath) -> Result<serde_json::Value, GlobalSettingsError>`**:
        1. Holt eine Lesesperre.
        2. Navigiert zum Wert via `match path`.
        3. Serialisiert den Wert zu `serde_json::Value`.
- **Events:** `SettingChangedEvent`, `SettingsLoadedEvent`, `SettingsSavedEvent` (via `tokio::sync::broadcast`).

**4.6. Detaillierte Implementierungsschritte `domain::global_settings`**

1. **Grundgerüst**: Verzeichnisse, `Cargo.toml`.
2. **`types.rs`**: Alle Einstellungs-`struct`s und `enum`s mit `serde`, `Default`, `validate()`-Methoden.
3. **`paths.rs`**: `SettingPath` Enum vollständig definieren, `Display` und `TryFrom<&str>` implementieren.
4. **`errors.rs`**: `GlobalSettingsError` definieren.
5. **`persistence_iface.rs`**: `SettingsPersistenceProvider`-Trait. `FilesystemSettingsProvider` implementieren (mit Mocking von `ConfigServiceAsync` für Tests).
6. **`service.rs`**: `GlobalSettingsService`-Trait. `DefaultGlobalSettingsService` implementieren:
    - Konstruktor, `RwLock` für `settings`.
    - `load_settings`, `save_settings` (interagieren mit Provider).
    - `update_setting` mit detaillierter Pfad-Navigation, Deserialisierung, Validierung.
    - `get_setting`, `reset_to_defaults`.
    - Event-Versand (`tokio::sync::broadcast`).
7. **Unit-Tests**: Für jede Einstellungsstruktur (Validierung, Serde, Default). Für `SettingPath` (Konvertierungen). Für `FilesystemSettingsProvider` (Mocking). Für `DefaultGlobalSettingsService` (alle API-Methoden, Fehlerfälle, Event-Auslösung, korrekte Aktualisierung verschachtelter Einstellungen).
8. **`mod.rs`**: Module deklarieren, öffentliche API re-exportieren.

---

Dieser detaillierte Plan für die Module `domain::theming` und `domain::global_settings_and_state_management` sollte es Entwicklern ermöglichen, diese Komponenten präzise zu implementieren. Die verbleibenden Module (Workspaces, Window Management Policy, User-Centric Services, Notifications Rules) würden nach demselben Schema und mit derselben Detailtiefe ausgearbeitet.


---

### Modul 5: `domain::workspaces`

Zweck: Umfassende Verwaltungslogik für Arbeitsbereiche ("Spaces"), einschließlich ihrer Definition, Fensterzuweisung, Orchestrierung und Persistenz.

Verantwortlichkeiten: Definition der Workspace-Entität, Regeln für Fensterzuweisung, Verwaltung der Workspace-Sammlung (inkl. aktivem Workspace), Laden und Speichern der Workspace-Konfiguration, Publizieren von Workspace-bezogenen Events.

Design-Rationale: Kapselung aller Workspace-bezogenen Logik an einem Ort, um Konsistenz und Wartbarkeit zu gewährleisten. Strikte Trennung von UI- und Systemdetails.

Bestehende Spezifikation: und vorherige Antworten.

#### 5.1. Untermodul: `domain::workspaces::core`

**Zweck:** Fundamentale Definition der `Workspace`-Entität und zugehöriger Typen.

**Datei:** `src/workspaces/core/types.rs`

- **Typalias `WorkspaceId`**
    - **Definition:** `pub type WorkspaceId = uuid::Uuid;`
    - **Ableitungen:** (Keine direkt, `uuid::Uuid` hat eigene)
- **Struct `WindowIdentifier`**
    - **Definition:**
        
        Rust
        
        ```
        use serde::{Serialize, Deserialize};
        use std::fmt;
        use super::errors::WorkspaceCoreError; // Für Validierungsfehler
        
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
        pub struct WindowIdentifier(String);
        
        impl WindowIdentifier {
            pub fn new(id: impl Into<String>) -> Result<Self, WorkspaceCoreError> {
                let id_str = id.into();
                if id_str.is_empty() {
                    return Err(WorkspaceCoreError::WindowIdentifierEmpty);
                }
                // Ggf. weitere Validierungen (z.B. erlaubte Zeichen)
                Ok(Self(id_str))
            }
            pub fn as_str(&self) -> &str { &self.0 }
        }
        impl fmt::Display for WindowIdentifier { /* ... */ }
        impl From<&str> for WindowIdentifier { fn from(s: &str) -> Self { Self::new(s).expect("Ungültiger WindowIdentifier aus &str") } }
        // From<String> ist riskanter ohne Fehlerbehandlung, new() bevorzugen
        ```
        
    - **Invarianten:** String nicht leer.
- **Enum `WorkspaceLayoutType`**
    - **Definition:** Wie in und vorheriger Antwort.
        
        Rust
        
        ```
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
        pub enum WorkspaceLayoutType {
            #[default]
            Floating,
            TilingHorizontal, // Fenster nebeneinander
            TilingVertical,   // Fenster untereinander
            // Zukünftig ggf. komplexere Tiling-Modi direkt hier definieren oder durch
            // domain::window_management_policy referenzieren. Fürs Erste sind diese fix.
            Maximized,        // Ein Fenster ist maximiert, andere ggf. verborgen/minimiert
        }
        ```
        

**Datei:** `src/workspaces/core/mod.rs` (enthält `Workspace`-Struct-Definition)

- **Struct `Workspace`**
    - **Definition:**
        
        Rust
        
        ```
        use std::collections::HashSet;
        use uuid::Uuid;
        use chrono::{DateTime, Utc};
        use serde::{Serialize, Deserialize};
        use super::types::{WorkspaceId, WindowIdentifier, WorkspaceLayoutType};
        use super::errors::WorkspaceCoreError;
        use super::errors::MAX_WORKSPACE_NAME_LENGTH;
        
        #[derive(Debug, Clone, Serialize, Deserialize)]
        // PartialEq manuell implementieren wegen HashSet, falls nötig, oder nur auf IDs vergleichen.
        // Für die meisten Anwendungsfälle ist der Vergleich über `id` ausreichend.
        pub struct Workspace {
            id: WorkspaceId,
            name: String,
            persistent_id: Option<String>,
            layout_type: WorkspaceLayoutType,
            window_ids: HashSet<WindowIdentifier>,
            created_at: DateTime<Utc>,
            // Neu: Optionale Metadaten für den Workspace, z.B. benutzerdefiniertes Icon/Farbe
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub icon_name: Option<String>, // Name eines Icons aus dem System-Theme
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub accent_color_hex: Option<String>, // z.B. "#RRGGBB"
        }
        
        impl Workspace {
            pub fn new(name: String, persistent_id: Option<String>, icon_name: Option<String>, accent_color_hex: Option<String>) -> Result<Self, WorkspaceCoreError> {
                if name.is_empty() {
                    return Err(WorkspaceCoreError::NameCannotBeEmpty);
                }
                if name.len() > MAX_WORKSPACE_NAME_LENGTH {
                    return Err(WorkspaceCoreError::NameTooLong {
                        name: name.clone(),
                        max_len: MAX_WORKSPACE_NAME_LENGTH,
                        actual_len: name.len(),
                    });
                }
                if let Some(pid) = &persistent_id {
                    if pid.is_empty() || pid.chars().any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_') {
                        return Err(WorkspaceCoreError::InvalidPersistentId(pid.clone()));
                    }
                }
                if let Some(hex) = &accent_color_hex {
                    // Basis-Validierung für Hex-Farbe
                    if !(hex.starts_with('#') && (hex.len() == 7 || hex.len() == 9) && hex[1..].chars().all(|c| c.is_ascii_hexdigit())) {
                        return Err(WorkspaceCoreError::InvalidAccentColorFormat(hex.clone()));
                    }
                }
        
                Ok(Self {
                    id: Uuid::new_v4(),
                    name,
                    persistent_id,
                    layout_type: WorkspaceLayoutType::default(),
                    window_ids: HashSet::new(),
                    created_at: Utc::now(),
                    icon_name,
                    accent_color_hex,
                })
            }
        
            pub fn id(&self) -> WorkspaceId { self.id }
            pub fn name(&self) -> &str { &self.name }
            pub fn persistent_id(&self) -> Option<&str> { self.persistent_id.as_deref() }
            pub fn layout_type(&self) -> WorkspaceLayoutType { self.layout_type }
            pub fn window_ids(&self) -> &HashSet<WindowIdentifier> { &self.window_ids }
            pub fn created_at(&self) -> DateTime<Utc> { self.created_at }
            pub fn icon_name(&self) -> Option<&str> { self.icon_name.as_deref() }
            pub fn accent_color_hex(&self) -> Option<&str> { self.accent_color_hex.as_deref() }
        
            pub fn rename(&mut self, new_name: String) -> Result<(), WorkspaceCoreError> {
                if new_name.is_empty() { /* ... NameCannotBeEmpty ... */ }
                if new_name.len() > MAX_WORKSPACE_NAME_LENGTH { /* ... NameTooLong ... */ }
                self.name = new_name;
                Ok(())
            }
        
            pub fn set_layout_type(&mut self, layout_type: WorkspaceLayoutType) {
                self.layout_type = layout_type;
            }
        
            pub(crate) fn add_window_id(&mut self, window_id: WindowIdentifier) -> bool {
                self.window_ids.insert(window_id)
            }
        
            pub(crate) fn remove_window_id(&mut self, window_id: &WindowIdentifier) -> bool {
                self.window_ids.remove(window_id)
            }
        
            pub fn set_persistent_id(&mut self, pid: Option<String>) -> Result<(), WorkspaceCoreError> {
                if let Some(p) = &pid {
                    if p.is_empty() || p.chars().any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '_') {
                        return Err(WorkspaceCoreError::InvalidPersistentId(p.clone()));
                    }
                }
                self.persistent_id = pid;
                Ok(())
            }
        
            pub fn set_icon_name(&mut self, icon: Option<String>) {
                self.icon_name = icon;
            }
        
            pub fn set_accent_color_hex(&mut self, color_hex: Option<String>) -> Result<(), WorkspaceCoreError> {
                 if let Some(hex) = &color_hex {
                    if !(hex.starts_with('#') && (hex.len() == 7 || hex.len() == 9) && hex[1..].chars().all(|c| c.is_ascii_hexdigit())) {
                        return Err(WorkspaceCoreError::InvalidAccentColorFormat(hex.clone()));
                    }
                }
                self.accent_color_hex = color_hex;
                Ok(())
            }
        }
        ```
        
    - **Felder:** Wie in vorheriger Antwort, plus `icon_name: Option<String>`, `accent_color_hex: Option<String>`.
    - **Methoden:** Wie in vorheriger Antwort, plus `set_icon_name`, `set_accent_color_hex`. Die `new`-Methode wird angepasst, um die neuen Felder zu akzeptieren und zu validieren.

**Datei:** `src/workspaces/core/event_data.rs`

- **Event-Payload-Strukturen**: Wie in. Zusätzlich:
    - `pub struct WorkspaceIconChangedData { pub id: WorkspaceId, pub old_icon_name: Option<String>, pub new_icon_name: Option<String> }`
    - `pub struct WorkspaceAccentChangedData { pub id: WorkspaceId, pub old_color_hex: Option<String>, pub new_color_hex: Option<String> }`
    - Alle mit `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]`.

**Datei:** `src/workspaces/core/errors.rs`

- **Konstante `MAX_WORKSPACE_NAME_LENGTH`**: `pub const MAX_WORKSPACE_NAME_LENGTH: usize = 64;`
- **Enum `WorkspaceCoreError`**: Wie in. Zusätzlich:
    - `WindowIdentifierEmpty`
    - `InvalidAccentColorFormat(String)`

#### 5.2. Untermodul: `domain::workspaces::assignment`

**Zweck:** Geschäftslogik für die Zuweisung von Fenstern zu Workspaces.

**Datei:** `src/workspaces/assignment/errors.rs`

- **Enum `WindowAssignmentError`**: Wie in. Keine Änderungen zur vorherigen Spezifikation notwendig, da es bereits umfassend war.

**Datei:** `src/workspaces/assignment/mod.rs`

- **API-Funktionen**: Wie in. Die Implementierung der Funktionen muss detailliert werden:
    - **`assign_window_to_workspace(...)` Logik:**
        1. Prüfe `target_workspace_id` in `workspaces`. Wenn nicht gefunden: `Err(WorkspaceNotFound)`.
        2. Wenn `ensure_unique_assignment` `true` ist:
            - Iteriere über alle `(ws_id, ws)` in `workspaces`.
            - Wenn `ws_id != target_workspace_id` UND `ws.window_ids().contains(window_id)`:
                - `ws.remove_window_id(window_id);` (Diese Methode ist `pub(crate)` in `Workspace`).
        3. Hole `target_ws = workspaces.get_mut(&target_workspace_id).unwrap()`.
        4. Wenn `target_ws.add_window_id(window_id.clone())` `false` zurückgibt (Fenster war bereits da):
            - `Ok(())` (Kein Fehler, wenn es bereits auf dem Ziel-Workspace ist, auch wenn `ensure_unique_assignment` `false` war. Die Semantik ist "stelle sicher, dass es auf dem Ziel ist").
        5. Sonst (wurde neu hinzugefügt): `Ok(())`.
    - **`remove_window_from_workspace(...)` Logik:**
        1. Prüfe `source_workspace_id`. Wenn nicht gefunden: `Err(WorkspaceNotFound)`.
        2. Hole `source_ws = workspaces.get_mut(&source_workspace_id).unwrap()`.
        3. `Ok(source_ws.remove_window_id(window_id))`
    - **`move_window_to_workspace(...)` Logik:**
        1. Wenn `source_workspace_id == target_workspace_id`: `Err(CannotMoveToSameWorkspace)`.
        2. Prüfe `source_workspace_id`. Wenn nicht: `Err(SourceWorkspaceNotFound)`.
        3. Prüfe `target_workspace_id`. Wenn nicht: `Err(TargetWorkspaceNotFound)`.
        4. `source_ws = workspaces.get_mut(&source_workspace_id).unwrap()`.
        5. Wenn `!source_ws.remove_window_id(window_id)`: `Err(WindowNotOnSourceWorkspace)`.
        6. `target_ws = workspaces.get_mut(&target_workspace_id).unwrap()`.
        7. `target_ws.add_window_id(window_id.clone());` (Rückgabewert hier ignorieren, da wir wissen, dass es von der Quelle entfernt wurde).
        8. `Ok(())`.
    - **`find_workspace_for_window(...)` Logik:**
        1. Iteriere `workspaces.values()`.
        2. Wenn `ws.window_ids().contains(window_id)`, gib `Some(ws.id())` zurück.
        3. Sonst `None`.

#### 5.3. Untermodul: `domain::workspaces::config`

**Zweck:** Persistenzlogik für Workspace-Konfigurationen.

**Datei:** `src/workspaces/config/errors.rs`

- **Enum `WorkspaceConfigError`**: Wie in vorheriger Antwort (basierend auf).

**Datei:** `src/workspaces/config/mod.rs` (oder `provider.rs` und `types.rs` hier)

- **Struct `WorkspaceSnapshot`**: Wie in vorheriger Antwort. Zusätzlich `icon_name: Option<String>`, `accent_color_hex: Option<String>`.
    
    Rust
    
    ```
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkspaceSnapshot {
        pub persistent_id: String, // Eindeutig über Sitzungen
        pub name: String,
        pub layout_type: WorkspaceLayoutType,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub icon_name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub accent_color_hex: Option<String>,
    }
    ```
    
- **Struct `WorkspaceSetSnapshot`**: Wie in vorheriger Antwort (`workspaces: Vec<WorkspaceSnapshot>`, `active_workspace_persistent_id: Option<String>`).
- **Trait `WorkspaceConfigProvider`**: Wie in vorheriger Antwort (`async fn load_workspace_config`, `async fn save_workspace_config`).
- **Struct `FilesystemConfigProvider`**:
    - **Felder:** `config_service: Arc<dyn novade_core::config::ConfigServiceAsync>`, `config_key_or_path: String` (z.B. "workspaces.toml").
    - **Konstruktor:** `pub fn new(config_service: Arc<dyn novade_core::config::ConfigServiceAsync>, config_key_or_path: String) -> Self`.
    - **Implementierung von `WorkspaceConfigProvider`**:
        - **`load_workspace_config`**:
            1. `content_result = self.config_service.read_config_file_string(&self.config_key_or_path).await;`
            2. `match content_result { Ok(content_str) => { ... } Err(core_err) => { ... } }`
            3. Wenn `core_err` "nicht gefunden" signalisiert (z.B. `CoreError::Io` mit `ErrorKind::NotFound` oder spezifischer `ConfigError`-Typ): `Ok(WorkspaceSetSnapshot::default())` zurückgeben (Manager erstellt dann Standard).
            4. Wenn anderer `core_err`: `Err(WorkspaceConfigError::LoadError { ..., source: core_err })`.
            5. Wenn `Ok(content_str)`: `toml::from_str(&content_str).map_err(|e| DeserializationError { ... source: Some(e) })`.
            6. Nach Deserialisierung: Validierung (doppelte `persistent_id`s in `snapshot.workspaces`, Existenz von `active_workspace_persistent_id` im Set). Bei Fehlern `InvalidData` oder `PersistentIdNotFoundInLoadedSet`.
        - **`save_workspace_config`**:
            1. `serialized_content = toml::to_string_pretty(config_snapshot).map_err(|e| SerializationError { ... source: Some(e) })?;`
            2. `self.config_service.write_config_file_string(&self.config_key_or_path, serialized_content).await.map_err(|e| SaveError { ..., source: e })?;`

#### 5.4. Untermodul: `domain::workspaces::manager`

**Zweck:** Zentraler Orchestrator für Workspace-Operationen.

**Datei:** `src/workspaces/manager/events.rs`

- **Enum `WorkspaceEvent`**: Wie in vorheriger Antwort. Zusätzlich:
    - `WorkspaceIconChanged(WorkspaceIconChangedData)`
    - `WorkspaceAccentChanged(WorkspaceAccentChangedData)`

**Datei:** `src/workspaces/manager/errors.rs`

- **Enum `WorkspaceManagerError`**: Wie in vorheriger Antwort. Keine wesentlichen Änderungen.

**Datei:** `src/workspaces/manager/mod.rs` (oder `service.rs`)

- **Trait `WorkspaceManagerService`**: (Neuer Trait für die öffentliche API, um die Implementierung zu entkoppeln).
    
    Rust
    
    ```
    use async_trait::async_trait;
    // ... imports ...
    #[async_trait]
    pub trait WorkspaceManagerService: Send + Sync {
        async fn create_workspace(&self, name: Option<String>, persistent_id: Option<String>, icon_name: Option<String>, accent_color_hex: Option<String>) -> Result<WorkspaceId, WorkspaceManagerError>;
        async fn delete_workspace(&self, id: WorkspaceId, fallback_id_for_windows: Option<WorkspaceId>) -> Result<(), WorkspaceManagerError>;
        fn get_workspace(&self, id: WorkspaceId) -> Option<Workspace>; // Gibt Klon zurück
        fn all_workspaces_ordered(&self) -> Vec<Workspace>; // Gibt Klone zurück
        fn active_workspace_id(&self) -> Option<WorkspaceId>;
        async fn set_active_workspace(&self, id: WorkspaceId) -> Result<(), WorkspaceManagerError>;
        async fn assign_window_to_active_workspace(&self, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>;
        async fn assign_window_to_specific_workspace(&self, workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>;
        async fn remove_window_from_its_workspace(&self, window_id: &WindowIdentifier) -> Result<Option<WorkspaceId>, WorkspaceManagerError>;
        async fn move_window_to_specific_workspace(&self, target_workspace_id: WorkspaceId, window_id: &WindowIdentifier) -> Result<(), WorkspaceManagerError>;
        async fn rename_workspace(&self, id: WorkspaceId, new_name: String) -> Result<(), WorkspaceManagerError>;
        async fn set_workspace_layout(&self, id: WorkspaceId, layout_type: WorkspaceLayoutType) -> Result<(), WorkspaceManagerError>;
        async fn set_workspace_icon(&self, id: WorkspaceId, icon_name: Option<String>) -> Result<(), WorkspaceManagerError>;
        async fn set_workspace_accent_color(&self, id: WorkspaceId, color_hex: Option<String>) -> Result<(), WorkspaceManagerError>;
        async fn save_configuration(&self) -> Result<(), WorkspaceManagerError>;
        fn subscribe_to_workspace_events(&self) -> tokio::sync::broadcast::Receiver<WorkspaceEvent>;
        // Neu: Methode zur Änderung der Reihenfolge
        async fn reorder_workspace(&self, workspace_id: WorkspaceId, new_index: usize) -> Result<(), WorkspaceManagerError>;
    }
    ```
    
- **Struct `WorkspaceManager`** (umbenannt zu `DefaultWorkspaceManager` für die Implementierung).
    - **Felder:** `internal: Arc<tokio::sync::Mutex<WorkspaceManagerInternalState>>`.
- **Struct `WorkspaceManagerInternalState`**:
    - `workspaces: HashMap<WorkspaceId, Workspace>`
    - `active_workspace_id: Option<WorkspaceId>`
    - `ordered_workspace_ids: Vec<WorkspaceId>`
    - `next_workspace_number: u32`
    - `config_provider: Arc<dyn WorkspaceConfigProvider>`
    - `event_publisher: tokio::sync::broadcast::Sender<WorkspaceEvent>`
    - `ensure_unique_window_assignment: bool`
- **Implementierung `#[async_trait] impl WorkspaceManagerService for DefaultWorkspaceManager`**:
    - **`new(...)`**:
        1. Sperrt `internal`, initialisiert Felder.
        2. `snapshot = self.internal.config_provider.load_workspace_config().await.map_err(WorkspaceManagerError::from)?;`
        3. Wenn `snapshot.workspaces` leer ist (oder `load_config` "nicht gefunden" signalisiert und Default zurückgibt):
            - Ruft `internal_create_workspace_locked` auf, um einen Standard-Workspace ("Workspace 1") zu erstellen.
            - Setzt diesen als aktiv.
        4. Sonst: Rekonstruiert `workspaces` und `ordered_workspace_ids` aus `snapshot`. Setzt `active_workspace_id` basierend auf `snapshot.active_workspace_persistent_id`.
        5. Aktualisiert `next_workspace_number`.
        6. Sendet `WorkspacesReloaded` und `ActiveWorkspaceChanged` Events.
    - **`create_workspace(...)`**:
        1. Sperrt `internal`.
        2. Prüft auf `DuplicatePersistentId`.
        3. Ruft `Workspace::new(...)`.
        4. Fügt zu `workspaces` und `ordered_workspace_ids` hinzu.
        5. Sendet `WorkspaceCreated`.
        6. Ruft `internal_save_configuration_locked()`.
    - **`delete_workspace(...)`**: Sperrt, prüft Bedingungen, verschiebt Fenster via `assignment`-Modul, sendet Events, speichert.
    - `get_workspace()` / `all_workspaces_ordered()`: Sperrt, klont die angeforderten `Workspace`-Objekte und gibt sie zurück.
    - **`set_active_workspace(...)`**: Sperrt, prüft, aktualisiert, sendet Event, speichert (optional).
    - **Fensterzuweisungsmethoden**: Sperren `internal`, rufen Funktionen aus `domain::workspaces::assignment` mit `&mut internal.workspaces` auf, senden Events.
    - **`rename_workspace(...)`, `set_workspace_layout(...)`, `set_workspace_icon(...)`, `set_workspace_accent_color(...)`**:
        1. Sperrt `internal`.
        2. Findet `workspace_mut` in `internal.workspaces`. Wenn nicht: `Err(WorkspaceNotFound)`.
        3. Ruft entsprechende `workspace_mut.set_...(...)` Methode auf.
        4. Sendet entsprechendes Event (`WorkspaceRenamed`, `WorkspaceLayoutChanged`, `WorkspaceIconChanged`, `WorkspaceAccentChanged`).
        5. Ruft `internal_save_configuration_locked()`.
    - **`reorder_workspace(...)`**:
        1. Sperrt `internal`.
        2. Validiert `workspace_id` und `new_index`.
        3. Entfernt `workspace_id` aus `ordered_workspace_ids` und fügt es an `new_index` wieder ein.
        4. Sendet `WorkspaceOrderChanged` mit der neuen `ordered_workspace_ids` (als Klon).
        5. Ruft `internal_save_configuration_locked()`.
    - **`save_configuration()`**: Sperrt `internal`, ruft `internal_save_configuration_locked()`.
    - **`internal_save_configuration_locked()`**: Private Hilfsmethode, die den Snapshot erstellt und `config_provider.save_workspace_config()` aufruft.
    - **`subscribe_to_workspace_events()`**: `self.internal.lock().await.event_publisher.subscribe()`.

#### 5.5. Implementierungsschritte `domain::workspaces`

(Reihenfolge und Tests wie in vorheriger Antwort, aber mit Fokus auf `async` wo spezifiziert und `tokio::sync::Mutex`.)

1. **`core` Modul**: `Workspace` um neue Felder und Methoden erweitern. Neue Event-Payloads. `WorkspaceCoreError` erweitern. Tests.
2. **`assignment` Modul**: Implementierungslogik der Funktionen detaillieren und testen.
3. **`config` Modul**: `WorkspaceSnapshot` anpassen. `FilesystemConfigProvider` mit `async` Methoden und `ConfigServiceAsync`. Tests (Mocking).
4. **`manager` Modul**: `WorkspaceEvent` erweitern. `WorkspaceManagerService` Trait definieren. `DefaultWorkspaceManager` mit `tokio::sync::Mutex` und `async` Methoden implementieren. `reorder_workspace` Methode hinzufügen. Umfassende Tests.
5. **`src/workspaces/mod.rs`**: Module deklarieren, öffentliche API (Service-Trait, wichtige Typen, Fehler, Events) re-exportieren.

---

### Modul 6: `domain::window_management_policy`

Zweck: Definition von High-Level-Regeln und Richtlinien für Fensterplatzierung, Tiling, Snapping, Gruppierung, Fokus und Gap-Management. Definiert die "Policy", die Systemschicht die "Mechanik".

Verantwortlichkeiten: Bereitstellung von Algorithmen zur Berechnung von Fenstergeometrien basierend auf aktuellen Policies und Workspace-Zuständen.

Design-Rationale: Entkopplung der komplexen Layout- und Policy-Logik von der technischen Umsetzung im Compositor. Ermöglicht flexible und austauschbare Fensterverwaltungsstrategien.

#### 6.1. Untermodul: `domain::window_management_policy::types`

**Datei:** `src/window_management_policy/types.rs`

- **Enum `TilingMode`**: Wie in vorheriger Antwort (Manual, Columns, Rows, Spiral, MaximizedFocused).
- **Struct `GapSettings`**: Wie in vorheriger Antwort.
- **Struct `WindowSnappingPolicy`**: Wie in vorheriger Antwort.
- **Struct `WindowGroupingPolicy`**: Wie in vorheriger Antwort.
- **Enum `NewWindowPlacementStrategy`**: Wie in vorheriger Antwort.
- **Enum `FocusStealingPreventionLevel`**: Wie in vorheriger Antwort.
- **Struct `FocusPolicy`**: Wie in vorheriger Antwort.
- **Struct `WindowPolicyOverrides`**: Wie in vorheriger Antwort.
- **Struct `WorkspaceWindowLayout`**: Wie in vorheriger Antwort.
    - **Zusatzfeld**: `pub tiling_mode_applied: TilingMode` (Speichert, welcher Modus tatsächlich für dieses Layout verwendet wurde).
- **Struct `WindowLayoutInfo`** (neu, für die Übergabe an `calculate_workspace_layout`):
    
    Rust
    
    ```
    use crate::domain::workspaces::core::types::WindowIdentifier;
    use crate::core::types::Size; // u32 Annahme
    
    #[derive(Debug, Clone, PartialEq)]
    pub struct WindowLayoutInfo {
        pub id: WindowIdentifier,
        pub requested_min_size: Option<Size<u32>>, // Vom Client oder Policy
        pub requested_base_size: Option<Size<u32>>,// Für Größeninkremente (zukünftig)
        pub is_fullscreen_requested: bool,
        pub is_maximized_requested: bool, // Expliziter Maximierungswunsch vom Client/User
        // Weitere Flags, die das Layout beeinflussen könnten
    }
    ```
    

#### 6.2. Untermodul: `domain::window_management_policy::errors`

**Datei:** `src/window_management_policy/errors.rs`

- **Enum `WindowPolicyError`**: Wie in vorheriger Antwort.

#### 6.3. Untermodul: `domain::window_management_policy::service`

**Datei:** `src/window_management_policy/service.rs` (oder `mod.rs`)

- **Trait `WindowManagementPolicyService`**:
    
    - **`calculate_workspace_layout` Signatur angepasst:**
        
        Rust
        
        ```
        async fn calculate_workspace_layout(
            &self,
            workspace_id: WorkspaceId,
            windows_to_layout: &[WindowLayoutInfo], // Geänderter Typ
            available_area: RectInt,
            // Policy-Einstellungen werden jetzt intern vom Service über GlobalSettingsService bezogen
            // oder es gibt spezifische Methoden, um sie zu setzen/abzurufen.
            // Hier gehen wir davon aus, dass sie intern über GlobalSettingsService bezogen werden.
            workspace_current_tiling_mode: TilingMode, // Tiling-Modus, der für diesen Workspace gilt
            focused_window_id: Option<&WindowIdentifier>, // Optional, für MaximizedFocused
            window_specific_overrides: &HashMap<WindowIdentifier, WindowPolicyOverrides>
        ) -> Result<WorkspaceWindowLayout, WindowPolicyError>;
        ```
        
    - **`get_initial_window_geometry` Signatur angepasst:**
        
        Rust
        
        ```
        async fn get_initial_window_geometry(
            &self,
            window_info: &WindowLayoutInfo, // Enthält requested_size etc.
            is_transient_for: Option<&WindowIdentifier>,
            parent_geometry: Option<RectInt>, // Geometrie des Elternfensters für transiente Fenster
            workspace_id: WorkspaceId,
            active_layout_on_workspace: &WorkspaceWindowLayout,
            available_area: RectInt,
            // placement_strategy wird intern vom Service via GlobalSettingsService bezogen
            window_specific_overrides: &Option<WindowPolicyOverrides>
        ) -> Result<RectInt, WindowPolicyError>;
        ```
        
    - **`calculate_snap_target`**: Signatur bleibt ähnlich.
    - **Neue Methoden zum Abruf von Teil-Policies (statt Übergabe von `GlobalDesktopSettings`):**
        
        Rust
        
        ```
        async fn get_effective_tiling_mode_for_workspace(&self, workspace_id: WorkspaceId) -> Result<TilingMode, WindowPolicyError>;
        async fn get_effective_gap_settings_for_workspace(&self, workspace_id: WorkspaceId) -> Result<GapSettings, WindowPolicyError>;
        async fn get_effective_snapping_policy(&self) -> Result<WindowSnappingPolicy, WindowPolicyError>;
        async fn get_effective_focus_policy(&self) -> Result<FocusPolicy, WindowPolicyError>;
        async fn get_effective_new_window_placement_strategy(&self) -> Result<NewWindowPlacementStrategy, WindowPolicyError>;
        // Diese Methoden würden intern den GlobalSettingsService konsultieren und
        // ggf. Workspace-spezifische Overrides berücksichtigen (falls diese in Zukunft eingeführt werden).
        ```
        
- **Implementierung `DefaultWindowManagementPolicyService`**:
    
    - **Konstruktor:** `pub fn new(settings_service: Arc<dyn GlobalSettingsService>) -> Self`.
    - **`calculate_workspace_layout` Logik (verfeinert):**
        1. Holt `GapSettings` über `self.get_effective_gap_settings_for_workspace()`.
        2. `effective_area = available_area` abzüglich äußerer `screen_outer_horizontal/vertical` Gaps.
        3. **Floating-Fenster herausfiltern:** Identifiziere Fenster mit `is_always_floating == Some(true)` aus `window_specific_overrides` oder solche, die aufgrund von Client-Hints (z.B. feste Größe, Dialoge - Information nicht direkt hier verfügbar, muss von Systemschicht kommen und in `WindowLayoutInfo` oder `WindowPolicyOverrides` reflektiert werden) als floating behandelt werden sollen. Für diese wird keine Tiling-Geometrie berechnet; ihre Positionen/Größen bleiben (oder werden initial gesetzt).
        4. `tiled_windows: Vec<&WindowLayoutInfo>` = verbleibende Fenster.
        5. Wenn `tiled_windows` leer ist oder `workspace_current_tiling_mode == TilingMode::Manual`:
            - Für jedes `window_info` in `windows_to_layout` (auch die "floating" markierten):
                - Wenn es ein Override für `fixed_position`/`fixed_size` gibt, dieses verwenden.
                - Sonst: Rufe `self.get_initial_window_geometry(...)` für dieses Fenster auf, um eine initiale Platzierung zu erhalten (oder behalte die aktuelle Position, falls es ein Re-Layout ist).
            - `WorkspaceWindowLayout` mit diesen Geometrien und `tiling_mode_applied = TilingMode::Manual` zurückgeben.
        6. **Tiling-Logik (Beispiel für `TilingMode::Columns`):**
            - `num_tiled_windows = tiled_windows.len()`.
            - `total_inner_gaps_width = gap_settings.window_inner * (num_tiled_windows - 1) as u16`.
            - `allocatable_width = effective_area.width - total_inner_gaps_width`.
            - `width_per_window = allocatable_width / num_tiled_windows as u32`. (Restbreite könnte verteilt oder ignoriert werden).
            - `current_x = effective_area.x`.
            - Für jedes `window_info` in `tiled_windows`:
                - `height = effective_area.height`.
                - `actual_width = width_per_window`. Ggf. Mindestbreite aus `window_info.requested_min_size` oder `WindowPolicyOverrides` berücksichtigen und `actual_width` anpassen (komplexere Verteilung nötig, wenn Mindestbreiten Summe überschreiten).
                - Speichere `RectInt::new(current_x, effective_area.y, actual_width, height)` für `window_info.id`.
                - `current_x += actual_width as i32 + gap_settings.window_inner as i32`.
            - `WorkspaceWindowLayout` mit diesen Geometrien und `tiling_mode_applied = TilingMode::Columns` zurückgeben.
        7. **`TilingMode::MaximizedFocused` Logik:**
            - Wenn `focused_window_id` und dieses Fenster in `tiled_windows` ist:
                - Geometrie für `focused_window_id` ist `effective_area`.
                - Andere `tiled_windows` erhalten eine (0,0)-Größe oder werden nicht in die `window_geometries` Map aufgenommen (signalisiert "versteckt").
            - Sonst (kein Fokus oder fokussiertes Fenster ist floating): Falle zurück auf `Manual` oder einen anderen Default-Tiling-Modus.
        8. Algorithmen für `Rows` (analog zu Columns), `Spiral` (Fibonacci-Partitionierung des `effective_area`) implementieren.
    - **Interne Hilfsfunktionen für Layout-Algorithmen.**

#### 6.4. Detaillierte Implementierungsschritte `domain::window_management_policy`

1. **Grundgerüst**: Verzeichnisse, `Cargo.toml`.
2. **`types.rs`**: Alle Policy-Typen (`TilingMode`, `GapSettings`, etc.), `WindowLayoutInfo`, `WorkspaceWindowLayout` definieren. `serde` und `Default`.
3. **`errors.rs`**: `WindowPolicyError` definieren.
4. **`service.rs`**:
    - `WindowManagementPolicyService`-Trait definieren mit den verfeinerten Signaturen.
    - `DefaultWindowManagementPolicyService`-Struktur mit `Arc<dyn GlobalSettingsService>`.
    - `new()`-Konstruktor implementieren.
    - `get_effective_*_policy()`-Methoden implementieren (lesen aus `GlobalSettingsService`).
    - `calculate_workspace_layout()`:
        - Implementiere Logik zum Herausfiltern von Floating-Fenstern.
        - Implementiere die spezifischen Layout-Algorithmen (Manual, Columns, Rows, Spiral, MaximizedFocused) als private Methoden, die `available_area`, `gap_settings` und die Liste der zu kachelnden Fenster berücksichtigen.
        - Stelle sicher, dass Mindest-/Maximalgrößen und Overrides aus `WindowPolicyOverrides` beachtet werden.
    - `get_initial_window_geometry()`: Implementiere verschiedene Strategien (`Smart`, `Center` etc.).
    - `calculate_snap_target()`: Implementiere Snapping-Logik.
5. **Unit-Tests**:
    - Für jeden Layout-Algorithmus: Verschiedene Anzahlen von Fenstern, verschiedene `available_area`, verschiedene Gap-Settings. Teste auch mit Mindestgrößen.
    - Für `get_initial_window_geometry`: Teste alle Platzierungsstrategien.
    - Für `calculate_snap_target`: Teste Snapping an Bildschirmränder und andere Fenster.
    - Für die Service-Methoden: Mocking des `GlobalSettingsService`. Teste, dass die korrekten Policies abgerufen und angewendet werden.

---

Die detaillierten Spezifikationen für die Module 7 (`user_centric_services`) und 8 (`notifications_rules`) werden in der nächsten Antwort folgen, basierend auf dieser Struktur und Methodik.
### Modul 7: `domain::user_centric_services`

Zweck: Bündelt die Logik für Dienste, die direkt auf die Bedürfnisse und Interaktionen des Benutzers ausgerichtet sind, insbesondere KI-Interaktionen (inklusive Einwilligungsmanagement) und ein umfassendes Benachrichtigungssystem.

Verantwortlichkeiten: Verwaltung von KI-Interaktionskontexten, Benutzereinwilligungen und KI-Modellprofilen. Entgegennahme, Verarbeitung, Speicherung und Verwaltung von System- und Anwendungsbenachrichtigungen.

Design-Rationale: Zentralisierung benutzerorientierter Dienste, um eine kohärente und kontrollierte Benutzererfahrung in Bezug auf Assistenzfunktionen und Benachrichtigungen zu ermöglichen. Trennung der Domänenlogik von der technischen Umsetzung (MCP-Kommunikation, D-Bus-Notification-Daemon-Interaktion) in der Systemschicht und der Darstellung in der UI-Schicht.

Bestehende Spezifikation: und vorherige Antworten.

#### 7.1. Untermodul: `domain::user_centric_services::ai_interaction`

**Zweck:** Verwaltung von KI-Interaktionen, Benutzereinwilligungen und KI-Modellprofilen.

**Datei:** `src/user_centric_services/ai_interaction/types.rs`

- **Enum `AIDataCategory`**
    
    - **Definition:** Wie in (UserProfile, ApplicationUsage, FileSystemRead, ClipboardAccess, LocationData, GenericText, GenericImage).
    - **Ableitungen:** `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize`.
    - **Zusatz:**
        
        Rust
        
        ```
        // Optional: Methode zur menschenlesbaren Beschreibung
        impl AIDataCategory {
            pub fn description(&self) -> &'static str {
                match self {
                    AIDataCategory::UserProfile => "Persönliche Profilinformationen (z.B. Name, Einstellungen)",
                    AIDataCategory::ApplicationUsage => "Informationen über genutzte Anwendungen und deren Aktivität",
                    AIDataCategory::FileSystemRead => "Lesezugriff auf das Dateisystem",
                    AIDataCategory::ClipboardAccess => "Zugriff auf den Inhalt der Zwischenablage",
                    AIDataCategory::LocationData => "Standortinformationen",
                    AIDataCategory::GenericText => "Allgemeiner Textinhalt (z.B. vom Benutzer eingegeben)",
                    AIDataCategory::GenericImage => "Allgemeiner Bildinhalt",
                }
            }
        }
        ```
        
- **Enum `AIConsentStatus`**
    
    - **Definition:** Wie in (Granted, Denied, PendingUserAction, NotRequired).
    - **Ableitungen:** `Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize`.
- **Struct `AttachmentData`**
    
    - **Definition:** Wie in.
        
        Rust
        
        ```
        use uuid::Uuid;
        use serde::{Serialize, Deserialize};
        
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct AttachmentData {
            pub id: Uuid,
            pub mime_type: String, // z.B. "text/plain", "image/png", "application/pdf"
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub source_uri: Option<String>, // z.B. "file:///path/to/file.txt"
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub content_base64: Option<String>, // Base64-kodierter Inhalt für Binärdaten
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub text_content: Option<String>, // Für reinen Textinhalt
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub description: Option<String>,
        }
        
        impl AttachmentData {
            pub fn new_text(text: String, description: Option<String>) -> Self { /* ... */ }
            pub fn new_from_uri(uri: String, mime_type: String, description: Option<String>) -> Self { /* ... */ }
            // new_from_binary_content(content: Vec<u8>, mime_type: String, description: Option<String>) -> Self
        }
        ```
        
    - **Verfeinerung:** `content: Option<Vec<u8>>` wird zu `content_base64: Option<String>` für leichtere Serialisierung (JSON) und `text_content: Option<String>`. `source_uri` bleibt für Verweise.
- **Struct `AIInteractionContext`**
    
    - **Definition:** Wie in.
        
        Rust
        
        ```
        use chrono::{DateTime, Utc};
        // ... andere Imports ...
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct AIInteractionContext {
            pub id: Uuid,
            pub creation_timestamp: DateTime<Utc>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub active_model_id: Option<String>, // ID des KI-Modells
            pub consent_status: AIConsentStatus,
            pub associated_data_categories: Vec<AIDataCategory>,
            // interaction_history als separate Struktur für mehr Flexibilität
            pub history_entries: Vec<InteractionHistoryEntry>,
            pub attachments: Vec<AttachmentData>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub user_prompt_template: Option<String>, // Vorlage für den initialen Prompt
            #[serde(default)]
            pub is_active: bool, // Ob dieser Kontext gerade "offen" oder aktiv in der UI ist
        }
        ```
        
    - **Neue Struktur `InteractionHistoryEntry`**:
        
        Rust
        
        ```
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub enum InteractionParticipant { User, Assistant, System }
        
        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        pub struct InteractionHistoryEntry {
            pub entry_id: Uuid,
            pub timestamp: DateTime<Utc>,
            pub participant: InteractionParticipant,
            pub content: String, // Text der Nachricht/Aktion
            #[serde(default, skip_serializing_if = "Vec::is_empty")]
            pub related_attachment_ids: Vec<Uuid>, // IDs von Attachments, die sich auf diesen Eintrag beziehen
        }
        ```
        
- **Struct `AIConsent`**
    
    - **Definition:** Wie in.
        
        Rust
        
        ```
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct AIConsent {
            pub id: Uuid, // Eindeutige ID der Einwilligung selbst
            pub user_id: String, // Vereinfacht, könnte komplexer sein
            pub model_id: String, // Für welches spezifische Modell oder "*" für alle
            pub data_category: AIDataCategory, // Einwilligung pro Kategorie
            pub granted_timestamp: DateTime<Utc>,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub expiry_timestamp: Option<DateTime<Utc>>,
            pub is_revoked: bool,
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub last_used_timestamp: Option<DateTime<Utc>>, // Wann zuletzt genutzt
            pub consent_scope: AIConsentScope, // Neu
        }
        
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
        pub enum AIConsentScope { #[default] SessionOnly, PersistentUntilRevoked, SpecificDuration }
        ```
        
    - **Verfeinerung:** `data_categories: Vec<AIDataCategory>` wird zu `data_category: AIDataCategory`, um granularere Einwilligungen pro Kategorie zu ermöglichen (d.h. ein `AIConsent`-Objekt pro (user, model, category)-Tupel). `AIConsentScope` hinzugefügt.
- **Struct `AIModelProfile`**
    
    - **Definition:** Wie in.
        
        Rust
        
        ```
        #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
        pub struct AIModelProfile {
            pub model_id: String, // Eindeutig, z.B. "local-llama3-8b", "openai-gpt-4o"
            pub display_name: String,
            pub description: String,
            pub provider: String, // z.B. "Local", "OpenAI", "Groq"
            pub required_consent_categories: Vec<AIDataCategory>,
            pub capabilities: Vec<AIModelCapability>, // Enum statt String
            pub supports_streaming: bool, // Gibt das Modell Antworten im Stream zurück?
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub endpoint_url: Option<String>, // Für Remote-Modelle
            #[serde(default, skip_serializing_if = "Option::is_none")]
            pub api_key_secret_name: Option<String>, // Name des Secrets im Secret Service
            #[serde(default)]
            pub is_default_model: bool, // Kann nur ein Modell Default sein
            #[serde(default)]
            pub sort_order: i32, // Für die Anzeige in UI-Auswahlen
        }
        
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub enum AIModelCapability { TextGeneration, CodeGeneration, Summarization, Translation, ImageAnalysis, FunctionCalling }
        ```
        
    - **Verfeinerung:** `capabilities` als Enum `AIModelCapability`. Zusätzliche Felder `supports_streaming`, `endpoint_url`, `api_key_secret_name`, `is_default_model`, `sort_order`.

**Datei:** `src/user_centric_services/ai_interaction/errors.rs`

- **Enum `AIInteractionError`**: Wie in. Zusätzlich/Verfeinert:
    - `ConsentCheckFailed { model_id: String, category: AIDataCategory, reason: String }`
    - `ApiKeyNotFoundInSecrets { secret_name: String }`
    - `ModelEndpointUnreachable { model_id: String, url: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> }` (Dieser Fehler käme eher von der Systemschicht, wird hier aber als möglicher Domänenfehler bei der Modellvalidierung aufgeführt)
    - `NoDefaultModelConfigured`
    - `CoreConfigError(#[from] novade_core::config::ConfigError)` (Wenn Laden von Profilen/Consents fehlschlägt)

**Datei:** `src/user_centric_services/ai_interaction/persistence_iface.rs`

- **Trait `AIConsentProvider`**:
    
    Rust
    
    ```
    #[async_trait]
    pub trait AIConsentProvider: Send + Sync {
        async fn load_consents_for_user(&self, user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError>;
        async fn save_consent(&self, consent: &AIConsent) -> Result<(), AIInteractionError>;
        async fn revoke_consent(&self, consent_id: Uuid, user_id: &str) -> Result<(), AIInteractionError>;
        // Ggf. Methode zum Löschen abgelaufener Consents
    }
    ```
    
- **Trait `AIModelProfileProvider`**:
    
    Rust
    
    ```
    #[async_trait]
    pub trait AIModelProfileProvider: Send + Sync {
        async fn load_model_profiles(&self) -> Result<Vec<AIModelProfile>, AIInteractionError>;
        // Ggf. async fn save_model_profiles(&self, profiles: &[AIModelProfile]) -> Result<(), AIInteractionError>;
    }
    ```
    
- **Implementierungen** (z.B. `FilesystemAIConsentProvider`, `FilesystemAIModelProfileProvider`) in einem `persistence.rs` Untermodul, die `Arc<dyn novade_core::config::ConfigServiceAsync>` nutzen, um Daten als TOML/JSON in spezifischen Dateien unter `$XDG_CONFIG_HOME/novade/ai/` zu speichern/laden.

**Datei:** `src/user_centric_services/ai_interaction/service.rs` (oder `mod.rs`)

- **Trait `AIInteractionLogicService`**:
    - **Signaturen:** Wie in.
    - **Zusätzliche Methoden/Verfeinerungen:**
        - `async fn get_consent_status_for_interaction(&self, context_id: Uuid, model_id: &str, required_categories: &[AIDataCategory]) -> Result<AIConsentStatus, AIInteractionError>;` (Prüft spezifisch für einen Kontext)
        - `async fn get_default_model(&self) -> Result<Option<AIModelProfile>, AIInteractionError>;`
        - `async fn update_interaction_history(&mut self, context_id: Uuid, entry: InteractionHistoryEntry) -> Result<(), AIInteractionError>;`
- **Implementierung `DefaultAIInteractionLogicService`**:
    - **Felder:**
        - `active_contexts: Arc<tokio::sync::Mutex<HashMap<Uuid, AIInteractionContext>>>`
        - `model_profiles: Arc<tokio::sync::RwLock<Vec<AIModelProfile>>>`
        - `user_consents: Arc<tokio::sync::Mutex<HashMap<String /* user_id */, Vec<AIConsent>>>>`
        - `consent_provider: Arc<dyn AIConsentProvider>`
        - `profile_provider: Arc<dyn AIModelProfileProvider>`
        - `event_publisher: tokio::sync::broadcast::Sender<super::events::AIInteractionEventEnum>` (Wrapper-Enum für Events)
    - **Konstruktor `new(...)`**: Nimmt Provider, lädt initial Profile und Consents für den aktuellen Benutzer.
    - **Logik `initiate_interaction`**: Erstellt `AIInteractionContext`, speichert in `active_contexts`, sendet `AIInteractionInitiatedEvent`.
    - **Logik `provide_consent`**:
        1. Findet oder erstellt `AIConsent`-Objekt(e) basierend auf `model_id`, `granted_categories`, `consent_decision`.
        2. Ruft `consent_provider.save_consent()` auf.
        3. Aktualisiert `user_consents` Cache.
        4. Aktualisiert `consent_status` im `AIInteractionContext` (falls `context_id` gegeben).
        5. Sendet `AIConsentUpdatedEvent`.
    - **Logik `get_consent_status_for_interaction`**:
        1. Lädt `active_model_id` aus `AIInteractionContext` (falls nicht direkt übergeben).
        2. Iteriert `required_categories`. Für jede Kategorie:
            - Sucht in `self.user_consents` nach einem gültigen (nicht abgelaufen, nicht widerrufen) `AIConsent` für den `user_id` (aus Kontext, hier vereinfacht), `model_id` und die `category`.
            - Wenn für eine Kategorie keine explizite Zustimmung (Granted) gefunden wird -> `AIConsentStatus::PendingUserAction` oder `Denied` (wenn zuvor explizit verweigert).
        3. Wenn für alle `Granted` -> `AIConsentStatus::Granted`.
        4. Wenn `model_profile.required_consent_categories` leer ist -> `AIConsentStatus::NotRequired`.
- **Events:** (Wrapper-Enum `AIInteractionEventEnum` für `tokio::sync::broadcast`)
    - `AIInteractionInitiatedEvent { context: AIInteractionContext }` (Payload enthält ganzen Kontext)
    - `AIConsentUpdatedEvent { user_id: String, model_id: String, category: AIDataCategory, new_status: AIConsentStatus, scope: AIConsentScope }`
    - `AIContextUpdatedEvent { context_id: Uuid, updated_field: String /* z.B. "history", "attachment" */}`
    - `AIModelProfilesReloadedEvent { profiles: Vec<AIModelProfile> }`

#### 7.2. Untermodul: `domain::user_centric_services::notifications_core`

**Zweck:** Kernlogik für das Verwalten von Benachrichtigungen.

**Datei:** `src/user_centric_services/notifications_core/types.rs`

- **Enum `NotificationUrgency`**: Wie in (Low, Normal, Critical).
- **Enum `NotificationActionType`**: Wie in (Callback, OpenLink).
- **Struct `NotificationAction`**: Wie in.
- **Struct `Notification`**: Wie in.
    
    Rust
    
    ```
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Notification {
        pub id: Uuid,
        pub application_name: String, // Optional: ApplicationId statt String
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub application_icon: Option<String>, // Icon-Name oder Pfad
        pub summary: String, // Darf nicht leer sein
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub body: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub actions: Vec<NotificationAction>,
        pub urgency: NotificationUrgency,
        pub timestamp: DateTime<Utc>, // Zeitpunkt des Eintreffens in diesem Service
        #[serde(default)]
        pub is_read: bool,
        #[serde(default)]
        pub is_dismissed: bool, // Vom Benutzer aktiv geschlossen
        #[serde(default)]
        pub transient: bool, // Nicht in Historie speichern, wenn true
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub category: Option<String>, // Standardkategorien: "device", "email", "im", "transfer" etc.
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        pub hints: HashMap<String, serde_json::Value>, // Für zusätzliche Daten (z.B. x-coordinates, image-data)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub timeout_ms: Option<u32>, // 0 für persistent, None für Standard-Timeout
    }
    impl Notification { /* ... new() ... mark_as_read(), dismiss() ... */ }
    ```
    
    - **Verfeinerung:** Zusätzliche Felder `category`, `hints`, `timeout_ms`.
- **Enum `NotificationFilterCriteria`**:
    
    Rust
    
    ```
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum NotificationFilterCriteria {
        Unread(bool), // true für nur ungelesene, false für nur gelesene
        Application(ApplicationId),
        Urgency(NotificationUrgency),
        Category(String),
        HasAction(String), // Action Key
        BodyContains(String),
        SummaryContains(String),
        IsTransient(bool),
        AndTimeRange { // Neu
            start: Option<DateTime<Utc>>,
            end: Option<DateTime<Utc>>,
        },
        And(Vec<NotificationFilterCriteria>), // Neu
        Or(Vec<NotificationFilterCriteria>),  // Neu
        Not(Box<NotificationFilterCriteria>), // Neu
    }
    ```
    
- **Enum `NotificationSortOrder`**: Wie in (TimestampAscending, TimestampDescending, Urgency). Zusätzlich: `ApplicationNameAscending`, `SummaryAscending`.

**Datei:** `src/user_centric_services/notifications_core/errors.rs`

- **Enum `NotificationError`**: Wie in. Zusätzlich:
    - `InvalidFilterCriteria(String)`
    - `ActionInvocationFailed { notification_id: Uuid, action_id: String, reason: String }`

**Datei:** `src/user_centric_services/notifications_core/service.rs` (oder `mod.rs`)

- **Trait `NotificationService`**:
    - **Signaturen:** Wie in.
    - `post_notification` nimmt `notification_data: NotificationInput` (eine vereinfachte Struktur ohne `id`, `timestamp`, `is_read`, `is_dismissed`).
    - **Neue Methoden:**
        - `async fn get_stats(&self) -> Result<NotificationStats, NotificationError>;`
        - `async fn clear_all_for_app(&mut self, app_id: &ApplicationId) -> Result<usize, NotificationError>;` (gibt Anzahl gelöschter zurück)
- **Implementierung `DefaultNotificationService`**:
    - **Felder:**
        - `active_notifications: Arc<tokio::sync::RwLock<VecDeque<Notification>>>` (VecDeque für einfaches FIFO-Verhalten, wenn ein Limit für aktive Popups existiert)
        - `history: Arc<tokio::sync::RwLock<VecDeque<Notification>>>`
        - `dnd_enabled: Arc<tokio::sync::RwLock<bool>>`
        - `rules_engine: Arc<dyn domain::notifications_rules::NotificationRulesEngine>` (Abhängigkeit injiziert)
        - `settings_service: Arc<dyn domain::global_settings_and_state_management::GlobalSettingsService>` (für MAX_HISTORY etc.)
        - `event_publisher: tokio::sync::broadcast::Sender<super::events::NotificationEventEnum>`
        - `max_active_popups: usize` (aus Einstellungen)
        - `max_history_items: usize` (aus Einstellungen)
    - **Konstruktor `new(...)`**: Nimmt Abhängigkeiten, lädt `max_active_popups` und `max_history_items` aus `settings_service`.
    - **Logik `post_notification(input: NotificationInput)`**:
        1. Erstellt `Notification`-Objekt aus `input`, generiert `id`, setzt `timestamp`.
        2. `processed_notification_option = self.rules_engine.process_notification(&mut notification_from_input, &current_global_settings).await?`
        3. Wenn `processed_notification_option` `None` ist (Regel hat unterdrückt) -> `Ok(notification_from_input.id)` (oder spezifischer Rückgabewert/Event).
        4. `let final_notification = processed_notification_option.unwrap();`
        5. Prüfe `dnd_enabled` und `final_notification.urgency`.
        6. Wenn nicht unterdrückt: Füge zu `active_notifications` hinzu. Wenn `max_active_popups` überschritten, ältestes entfernen (und ggf. Event für "Popup abgelaufen" senden). Sendet `NotificationPostedEvent { notification: final_notification.clone(), suppressed_by_dnd: false }`.
        7. Wenn unterdrückt: Sendet `NotificationPostedEvent { notification: final_notification.clone(), suppressed_by_dnd: true }`.
        8. Wenn `!final_notification.transient`: Füge zu `history` hinzu. Wenn `max_history_items` überschritten, ältestes aus `history` entfernen.
        9. `Ok(final_notification.id)`.
    - **Logik `get_active_notifications` / `get_notification_history`**: Implementiert Filterung (rekursiv für And/Or/Not) und Sortierung.
- **Struct `NotificationInput`**: Enthält Felder, die ein Client zum Erstellen einer Benachrichtigung bereitstellt (ohne `id`, `timestamp`, `is_read`, `is_dismissed`).
- **Struct `NotificationStats`**: `num_active: usize`, `num_history: usize`, `num_unread: usize`.
- **Events:** (Wrapper-Enum `NotificationEventEnum`)
    - `NotificationPostedEvent { notification: Notification, suppressed_by_dnd: bool }`
    - `NotificationDismissedEvent { notification_id: Uuid, reason: DismissReason }` (Grund für Dismiss, z.B. User, Timeout, Replaced)
    - `NotificationReadEvent { notification_id: Uuid }`
    - `NotificationActionInvokedEvent { notification_id: Uuid, action_key: String }`
    - `DoNotDisturbModeChangedEvent { dnd_enabled: bool }`
    - `NotificationHistoryClearedEvent`
    - `NotificationPopupExpiredEvent { notification_id: Uuid }` (Wenn aus aktiven Popups entfernt wegen Limit)

**Datei:** `src/user_centric_services/mod.rs`

- Deklariert Submodule `ai_interaction`, `notifications_core`, und ein gemeinsames `events.rs`.
- Re-exportiert öffentliche Traits (`AIInteractionLogicService`, `NotificationService`), Event-Enums und wichtige Typen.

#### 7.3. Implementierungsschritte `domain::user_centric_services`

1. **Grundgerüst:** Verzeichnisse für `ai_interaction` und `notifications_core` sowie gemeinsames `events.rs`.
2. **`ai_interaction` Modul:**
    - `types.rs`: Alle KI-bezogenen Typen und Enums.
    - `errors.rs`: `AIInteractionError`.
    - `persistence_iface.rs`: `AIConsentProvider`, `AIModelProfileProvider` Traits.
    - `persistence.rs` (intern): Implementierungen der Provider-Traits (z.B. `Filesystem...Provider`), die `Arc<dyn novade_core::config::ConfigServiceAsync>` nutzen.
    - `service.rs`: `AIInteractionLogicService`-Trait und `DefaultAIInteractionLogicService`-Implementierung.
    - Unit-Tests für Typen, Fehler, Provider-Implementierungen (Mocking `ConfigServiceAsync`), Service-Logik.
3. **`notifications_core` Modul:**
    - `types.rs`: Alle Benachrichtigungs-bezogenen Typen und Enums. `NotificationInput`.
    - `errors.rs`: `NotificationError`.
    - `service.rs`: `NotificationService`-Trait und `DefaultNotificationService`-Implementierung (nimmt `NotificationRulesEngine`, `GlobalSettingsService` als Abhängigkeiten).
    - Unit-Tests für Typen, Fehler, Service-Logik (insb. DND, History-Limit, Filter/Sort). Mocking von `NotificationRulesEngine` und `GlobalSettingsService`.
4. **`user_centric_services/events.rs`**: Event-Wrapper-Enums (`AIInteractionEventEnum`, `NotificationEventEnum`) definieren.
5. **`user_centric_services/mod.rs`**: Öffentliche API re-exportieren.
6. **Abhängigkeiten in `Cargo.toml`** für `user_centric_services` prüfen (insbesondere `uuid`, `chrono`, `serde`, `thiserror`, `async-trait`, `tokio`).

---

### Modul 8: `domain::notifications_rules`

Zweck: Logik zur dynamischen Verarbeitung von Benachrichtigungen basierend auf konfigurierbaren Regeln. Diese Regeln modifizieren oder unterdrücken Benachrichtigungen, bevor sie dem Benutzer präsentiert oder in der Historie gespeichert werden.

Verantwortlichkeiten: Definition von Regelstrukturen, Laden/Speichern von Regeldefinitionen, Auswerten von Regeln gegen eingehende Benachrichtigungen.

Design-Rationale: Entkopplung der komplexen Regellogik vom Kern-Benachrichtigungsdienst. Ermöglicht es Benutzern oder Administratoren, das Benachrichtigungsverhalten fein granular anzupassen.

**Datei:** `src/notifications_rules/types.rs`

- **Enum `RuleConditionValue`** (neu, für Vergleiche):
    
    Rust
    
    ```
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum RuleConditionValue {
        String(String),
        Integer(i64),
        Boolean(bool),
        Urgency(NotificationUrgency), // NotificationUrgency aus notifications_core::types
        Regex(String), // Für Regex-Matching
    }
    ```
    
- **Enum `RuleConditionOperator`** (neu):
    
    Rust
    
    ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum RuleConditionOperator {
        Is, IsNot,
        Contains, NotContains,
        StartsWith, EndsWith,
        MatchesRegex, NotMatchesRegex, // Für String-Werte gegen Regex
        GreaterThan, LessThan, GreaterThanOrEqual, LessThanOrEqual, // Für Integer
    }
    ```
    
- **Struct `RuleConditionField`** (neu, um das Feld der Notification zu spezifizieren):
    
    Rust
    
    ```
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum RuleConditionField {
        ApplicationName,
        Summary,
        Body,
        Urgency, // Vergleicht mit NotificationUrgency
        Category,
        HintExists(String), // Prüft Existenz eines Hints
        HintValue(String),  // Prüft Wert eines Hints (benötigt Operator und RuleConditionValue)
        // Zukünftig: ApplicationId, etc.
    }
    ```
    
- **Struct `SimpleRuleCondition`** (neu, für atomare Bedingungen):
    
    Rust
    
    ```
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct SimpleRuleCondition {
        pub field: RuleConditionField,
        pub operator: RuleConditionOperator,
        pub value: RuleConditionValue, // Wert, mit dem verglichen wird
    }
    ```
    
- **Enum `RuleCondition`** (rekursiv):
    
    Rust
    
    ```
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum RuleCondition {
        Simple(SimpleRuleCondition),
        SettingIsTrue(crate::global_settings_and_state_management::paths::SettingPath), // Pfad zu einer booleschen Einstellung
        And(Vec<RuleCondition>),
        Or(Vec<RuleCondition>),
        Not(Box<RuleCondition>),
    }
    ```
    
- **Enum `RuleAction`**:
    
    Rust
    
    ```
    use novade_core::types::Color as CoreColor; // Für das Setzen von Akzenten
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum RuleAction {
        SuppressNotification, // Benachrichtigung komplett unterdrücken
        SetUrgency(NotificationUrgency),
        AddActionToNotification(NotificationAction), // NotificationAction aus notifications_core::types
        SetHint(String /* key */, serde_json::Value /* value */),
        PlaySound(String /* sound_name_or_path */),
        MarkAsPersistent(bool), // Überschreibt transient-Flag der Notification
        SetTimeoutMs(Option<u32>), // Überschreibt Timeout
        SetCategory(String),
        // Neu:
        SetSummary(String), // Kann Template-Variablen enthalten, z.B. "{{original_summary}} - Wichtig!"
        SetBody(String),    // dito
        SetIcon(String),    // Icon-Name oder Pfad
        SetAccentColor(Option<CoreColor>), // Spezifische Akzentfarbe für diese Benachrichtigung
        StopProcessingFurtherRules, // Verhindert, dass nachfolgende Regeln ausgewertet werden
        LogMessage(String), // Loggt eine Nachricht, wenn die Regel zutrifft (für Debugging)
    }
    ```
    
- **Struct `NotificationRule`**:
    
    Rust
    
    ```
    use uuid::Uuid;
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct NotificationRule {
        pub id: Uuid,
        pub name: String, // Menschenlesbarer Name/Beschreibung der Regel
        pub condition: RuleCondition, // Die Bedingung(en)
        pub actions: Vec<RuleAction>, // Aktionen, die ausgeführt werden, wenn Bedingung zutrifft
        pub is_enabled: bool,
        pub priority: i32, // Höhere Zahl = höhere Priorität (wird früher ausgewertet)
        // stop_processing_after_match ist jetzt eine RuleAction::StopProcessingFurtherRules
    }
    impl Default for NotificationRule { /* ... id = new_v4(), is_enabled = true, priority = 0 ... */ }
    ```
    
- **Typalias `NotificationRuleSet`**: `pub type NotificationRuleSet = Vec<NotificationRule>;`

**Datei:** `src/notifications_rules/errors.rs`

- **Enum `NotificationRulesError`**:
    
    Rust
    
    ```
    use thiserror::Error;
    use super::types::NotificationRule; // Pfad ggf. anpassen
    
    #[derive(Debug, Error)]
    pub enum NotificationRulesError {
        #[error("Invalid rule definition for rule '{rule_name}' (ID: {rule_id}): {reason}")]
        InvalidRuleDefinition { rule_id: uuid::Uuid, rule_name: String, reason: String },
        #[error("Error evaluating condition for rule '{rule_name}' (ID: {rule_id}): {details}")]
        ConditionEvaluationError { rule_id: uuid::Uuid, rule_name: String, details: String, #[source] source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
        #[error("Error applying action for rule '{rule_name}' (ID: {rule_id}): {details}")]
        ActionApplicationError { rule_id: uuid::Uuid, rule_name: String, details: String },
        #[error("Error accessing global settings for rule condition evaluation: {0}")]
        SettingsAccessError(#[from] crate::global_settings_and_state_management::GlobalSettingsError), // Pfad anpassen
        #[error("Error loading or saving notification rules: {0}")]
        RulePersistenceError(#[from] novade_core::errors::CoreError), // Annahme: Provider nutzt CoreError für I/O
        #[error("Invalid regex in rule condition: {0}")]
        InvalidRegex(String),
        #[error("An internal error occurred in notification rules engine: {0}")]
        InternalError(String),
    }
    ```
    

**Datei:** `src/notifications_rules/persistence_iface.rs`

- **Trait `NotificationRulesProvider`**: Wie in vorheriger Antwort.

**Datei:** `src/notifications_rules/persistence.rs` (intern)

- **Struct `FilesystemNotificationRulesProvider`**: Implementiert `NotificationRulesProvider`, nutzt `Arc<dyn novade_core::config::ConfigServiceAsync>` zum Laden/Speichern der `NotificationRuleSet` (z.B. als JSON-Array in `$XDG_CONFIG_HOME/novade/notification_rules.json`).

**Datei:** `src/notifications_rules/engine.rs` (oder `service.rs` / `mod.rs`)

- **Enum `RuleProcessingResult`**:
    
    Rust
    
    ```
    use crate::user_centric_services::notifications_core::types::Notification; // Pfad anpassen
    
    #[derive(Debug, Clone, PartialEq)]
    pub enum RuleProcessingResult {
        Allow(Notification),      // Benachrichtigung (ggf. modifiziert) erlauben
        Suppress { rule_id: uuid::Uuid }, // Benachrichtigung aufgrund dieser Regel unterdrücken
    }
    ```
    
- **Trait `NotificationRulesEngine`**:
    
    Rust
    
    ```
    use async_trait::async_trait;
    use crate::user_centric_services::notifications_core::types::Notification;
    use crate::global_settings_and_state_management::types::GlobalDesktopSettings;
    use super::types::NotificationRuleSet; // Eigene Typen
    use super::errors::NotificationRulesError;
    
    #[async_trait]
    pub trait NotificationRulesEngine: Send + Sync {
        /// Lädt oder aktualisiert die im System verwendeten Regeln.
        async fn reload_rules(&self) -> Result<(), NotificationRulesError>;
    
        /// Verarbeitet eine eingehende Benachrichtigung gegen die geladenen Regeln.
        /// Kann die Benachrichtigung modifizieren oder deren Unterdrückung signalisieren.
        async fn process_notification(
            &self,
            notification: Notification, // Nimmt Ownership, gibt ggf. modifizierte zurück
            // settings_snapshot: &GlobalDesktopSettings, // Benötigt aktuellen Snapshot für SettingIsTrue
        ) -> Result<RuleProcessingResult, NotificationRulesError>;
    
        /// Gibt die aktuell geladenen Regeln zurück (z.B. für UI zur Anzeige/Bearbeitung).
        async fn get_rules(&self) -> Result<NotificationRuleSet, NotificationRulesError>;
    
        /// Speichert einen neuen Satz von Regeln.
        async fn update_rules(&self, new_rules: NotificationRuleSet) -> Result<(), NotificationRulesError>;
    }
    ```
    
- **Implementierung `DefaultNotificationRulesEngine`**:
    - **Felder:**
        - `rules: Arc<tokio::sync::RwLock<NotificationRuleSet>>`
        - `rules_provider: Arc<dyn NotificationRulesProvider>`
        - `settings_service: Arc<dyn crate::global_settings_and_state_management::GlobalSettingsService>`
    - **Konstruktor `new(...)`**: Nimmt Provider, lädt initial Regeln via `reload_rules_internal_locked()`.
    - **`reload_rules()`**: Sperrt `rules`-Lock, ruft `rules_provider.load_rules()`, sortiert nach `priority`, aktualisiert `self.rules`.
    - **`process_notification(...)` Logik:**
        1. Holt Lesesperre für `self.rules`. Holt aktuellen Snapshot der `GlobalDesktopSettings` vom `settings_service`.
        2. Iteriert durch eine Kopie der `enabled_rules` (sortiert nach `priority` DESC).
        3. `let mut current_notification = notification;` (mutable Kopie für mögliche Modifikationen).
        4. Für jede Regel:
            - `match self.evaluate_condition_recursive(&rule.condition, &current_notification, &settings_snapshot).await { ... }`
            - Wenn `Ok(true)` (Bedingung erfüllt):
                - `let stop_after_this = self.apply_actions_internal(&rule.actions, &mut current_notification, &rule).await?;`
                - Wenn eine Aktion `SuppressNotification` war: `return Ok(RuleProcessingResult::Suppress { rule_id: rule.id });`
                - Wenn `stop_after_this` (durch `RuleAction::StopProcessingFurtherRules`): `break;` (aus der Regelschleife).
            - Bei `Err(e)`: Logge Fehler, fahre ggf. mit nächster Regel fort oder gib Fehler zurück.
        5. `Ok(RuleProcessingResult::Allow(current_notification))`.
    - **`evaluate_condition_recursive(...)` Logik:**
        - `Simple(simple_cond)`: Wertet Feld gegen `operator` und `value` aus. Für `HintValue`, `Regex` etc.
        - `SettingIsTrue(setting_path)`: Ruft `settings_service.get_setting(&setting_path)` ab, prüft ob `serde_json::Value::Bool(true)`.
        - `And`, `Or`, `Not`: Rekursive Auswertung.
    - **`apply_actions_internal(...)` Logik:** Modifiziert `current_notification` basierend auf `RuleAction`s. Gibt `bool` zurück, ob Verarbeitung gestoppt werden soll.
    - **`get_rules()`**: Sperrt, klont, gibt `self.rules` zurück.
    - **`update_rules(...)`**: Sperrt, ersetzt `self.rules`, ruft `rules_provider.save_rules()` auf.

**Datei:** `src/notifications_rules/mod.rs`

- Deklariert Submodule. Re-exportiert `NotificationRulesEngine`-Trait, `NotificationRule`, `RuleCondition`, `RuleAction`, `NotificationRulesError`, `RuleProcessingResult` und den `NotificationRulesProvider`-Trait.

#### 7.4. Implementierungsschritte `domain::notifications_rules`

1. **Grundgerüst**: Verzeichnisse, `Cargo.toml`.
2. **`types.rs`**: Alle Regel-Typen (`RuleConditionValue`, `Operator`, `Field`, `SimpleRuleCondition`, `RuleCondition`, `RuleAction`, `NotificationRule`, `NotificationRuleSet`). `serde` und `Default`.
3. **`errors.rs`**: `NotificationRulesError` definieren.
4. **`persistence_iface.rs`**: `NotificationRulesProvider`-Trait definieren.
5. **`persistence.rs`**: `FilesystemNotificationRulesProvider` implementieren (nutzt `ConfigServiceAsync`). Tests mit Mocking.
6. **`engine.rs`**: `RuleProcessingResult` Enum. `NotificationRulesEngine`-Trait. `DefaultNotificationRulesEngine` implementieren:
    - Konstruktor, `RwLock` für `rules`.
    - `reload_rules()`, `process_notification()` (mit rekursiver Bedingungsauswertung und Aktionsanwendung), `get_rules()`, `update_rules()`.
    - Tests für Bedingungsauswertung (alle Operatoren, Feldtypen, logische Verknüpfungen, `SettingIsTrue`). Tests für Aktionsanwendung. Tests für Regelpriorisierung und `StopProcessingFurtherRules`. Mocking von `NotificationRulesProvider` und `GlobalSettingsService`.
7. **`mod.rs`**: Öffentliche API re-exportieren.

---

Diese detaillierten Pläne für die Module 1 bis 8 der Domänenschicht sollten eine sehr solide Grundlage für die Implementierung bieten. Jede Komponente ist klar abgegrenzt, und die Interaktionen sowie Abhängigkeiten sind definiert. Der nächste Schritt wäre, für jedes dieser Module die detaillierten Implementierungsschritte (wie im Beispiel für `core::types` und jetzt für die Domänenmodule gezeigt) und die Unit-Tests vollständig auszuarbeiten.
