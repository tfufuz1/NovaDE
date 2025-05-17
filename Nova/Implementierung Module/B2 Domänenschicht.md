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

#### **Referenzen**

1. Error Handling for Large Rust Projects \- Best Practice in GreptimeDB, Zugriff am Mai 13, 2025, [https://greptime.com/blogs/2024-05-07-error-rust](https://greptime.com/blogs/2024-05-07-error-rust)  
2. Error handling \- good/best practices : r/rust \- Reddit, Zugriff am Mai 13, 2025, [https://www.reddit.com/r/rust/comments/1bb7dco/error\_handling\_goodbest\_practices/](https://www.reddit.com/r/rust/comments/1bb7dco/error_handling_goodbest_practices/)  
3. Error in std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/trait.Error.html](https://doc.rust-lang.org/std/error/trait.Error.html)  
4. std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/index.html](https://doc.rust-lang.org/std/error/index.html)