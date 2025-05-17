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

#### **Referenzen**

1. Error Handling for Large Rust Projects \- Best Practice in GreptimeDB, Zugriff am Mai 13, 2025, [https://greptime.com/blogs/2024-05-07-error-rust](https://greptime.com/blogs/2024-05-07-error-rust)  
2. Error handling \- good/best practices : r/rust \- Reddit, Zugriff am Mai 13, 2025, [https://www.reddit.com/r/rust/comments/1bb7dco/error\_handling\_goodbest\_practices/](https://www.reddit.com/r/rust/comments/1bb7dco/error_handling_goodbest_practices/)  
3. Error in std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/trait.Error.html](https://doc.rust-lang.org/std/error/trait.Error.html)  
4. std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/index.html](https://doc.rust-lang.org/std/error/index.html)