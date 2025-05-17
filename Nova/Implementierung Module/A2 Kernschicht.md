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

#### **Referenzen**

1. Error Handling for Large Rust Projects \- Best Practice in GreptimeDB, Zugriff am Mai 13, 2025, [https://greptime.com/blogs/2024-05-07-error-rust](https://greptime.com/blogs/2024-05-07-error-rust)  
2. Error handling \- good/best practices : r/rust \- Reddit, Zugriff am Mai 13, 2025, [https://www.reddit.com/r/rust/comments/1bb7dco/error\_handling\_goodbest\_practices/](https://www.reddit.com/r/rust/comments/1bb7dco/error_handling_goodbest_practices/)  
3. std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/index.html](https://doc.rust-lang.org/std/error/index.html)  
4. Error in std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/trait.Error.html](https://doc.rust-lang.org/std/error/trait.Error.html)