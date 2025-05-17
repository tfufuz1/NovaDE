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

#### **Referenzen**

1. Error handling \- good/best practices : r/rust \- Reddit, Zugriff am Mai 13, 2025, [https://www.reddit.com/r/rust/comments/1bb7dco/error\_handling\_goodbest\_practices/](https://www.reddit.com/r/rust/comments/1bb7dco/error_handling_goodbest_practices/)  
2. Error Handling for Large Rust Projects \- Best Practice in GreptimeDB, Zugriff am Mai 13, 2025, [https://greptime.com/blogs/2024-05-07-error-rust](https://greptime.com/blogs/2024-05-07-error-rust)  
3. Error in std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/trait.Error.html](https://doc.rust-lang.org/std/error/trait.Error.html)