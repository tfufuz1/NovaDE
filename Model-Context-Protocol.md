# MCP

## Strategien zur Speicherung von lokalem Wissen für austauschbare KI-Modelle

Um lokales Wissen effektiv für verschiedene, austauschbare KI-Modelle in NovaDE zu nutzen, ohne bei einem Modellwechsel den Kontext zu verlieren, kombinieren wir mehrere Techniken:

1.  **Zentraler Wissensspeicher (Knowledge Base):**
    * **Was:** Alle relevanten lokalen Daten (Dokumente, Notizen, Benutzerpräferenzen, Projektdaten, vergangene Konversationen mit *allen* Modellen) werden an einem zentralen Ort gespeichert.
    * **Wie:** Dies kann eine Kombination aus einer **NoSQL-Datenbank** (für strukturierte Metadaten und Konversationsprotokolle) und einem **Dateisystem-basierten Speicher** (für Originaldokumente) sein.
    * **Vorteil:** Einheitliche Datenquelle, erleichtert Backup und Management.

2.  **Retrieval Augmented Generation (RAG) mit Vektordatenbanken:** 🧠
    * **Was:** Textuelle Inhalte aus dem Wissensspeicher werden in kleinere Abschnitte (Chunks) zerlegt und mittels eines **Embedding-Modells** in numerische Vektoren umgewandelt. Diese Vektoren repräsentieren die semantische Bedeutung der Textabschnitte.
    * **Wie:** Diese Vektoren werden zusammen mit Referenzen auf die Originaldaten in einer **Vektordatenbank** (z.B. Pinecone, Weaviate, Milvus, oder auch lokale Lösungen wie FAISS oder spezialisierte SQLite-Erweiterungen) gespeichert.
    * **Nutzung:** Bei einer Benutzeranfrage wird diese ebenfalls in einen Vektor umgewandelt. Die Vektordatenbank führt dann eine Ähnlichkeitssuche durch, um die relevantesten Wissens-Chunks zu finden.
    * **Vorteil:** Ermöglicht das Auffinden thematisch relevanter Informationen, auch wenn die genauen Schlüsselwörter nicht verwendet werden. Die abgerufenen Informationen sind modellunabhängig. (Quelle: Databricks, ExxactCorp)
    * **Wichtig:** Das für die Erstellung der Embeddings genutzte Modell sollte idealerweise leistungsstark und generalisiert sein, um gute Repräsentationen zu erzeugen. Obwohl die *gespeicherten* Embeddings modellunabhängig sind (im Sinne der Speicherung), hängt die *Qualität* des Retrievals von der Qualität des Embedding-Modells ab.

3.  **Standardisiertes Kontextformat und Konversationshistorie:** 📝
    * **Was:** Definieren eines internen, standardisierten Formats zur Darstellung von Konversationshistorien und kontextuellen Metadaten (z.B. aktuelles Thema, beteiligte Entitäten, Benutzerabsicht).
    * **Wie:** Dieses Format könnte JSON-basiert sein und Elemente wie `{"role": "user", "content": "...", "timestamp": "...", "model_used": "model_A_id", "retrieved_knowledge_ids": ["doc1_chunk5", "note2_chunk1"]}` enthalten.
    * **Vorteil:** Erleichtert die Verarbeitung und das Management der Konversationsflüsse unabhängig vom gerade aktiven Modell.
    * **Umgang mit Modellwechsel:** Wenn ein Modell gewechselt wird, bleibt die standardisierte Historie erhalten.

4.  **Modellspezifische Prompt-Engineering-Schicht und Adapter:** ⚙️
    * **Was:** Eine Komponente, die den für das *aktuell ausgewählte* Modell optimalen Prompt dynamisch zusammenstellt.
    * **Wie:** Diese Schicht nimmt:
        * Die aktuelle Benutzeranfrage.
        * Relevante Chunks aus der Vektordatenbank (via RAG).
        * Die standardisierte Konversationshistorie.
        * Modellspezifische Instruktionen oder Formatierungsregeln (z.B. System-Prompts, spezielle Token).
    * Anschließend formatiert sie diese Informationen so, dass sie optimal in das Kontextfenster des gewählten Modells passen und dessen Stärken ausnutzen. Unterschiedliche Modelle haben unterschiedliche Token-Limits und bevorzugen möglicherweise unterschiedliche Prompt-Strukturen. (Quelle: PromptLayer, Reddit-Diskussionen)
    * **Vorteil:** Maximiert die Performanz für jedes einzelne Modell, während der zugrundeliegende Wissens- und Konversationskontext erhalten bleibt.

5.  **Explizites Kontextmanagement durch den Benutzer (optional aber empfohlen):**
    * **Was:** Dem Benutzer ermöglichen, explizit "Kontext-Sets" oder "Wissensquellen" für bestimmte Aufgaben oder Projekte zu definieren, die dann priorisiert für das RAG-System herangezogen werden.
    * **Wie:** UI-Elemente, über die der Benutzer Ordner, Dokumenttypen oder spezifische Notizen als relevant für den aktuellen Chat oder die aktuelle Aufgabe markieren kann.
    * **Vorteil:** Erhöht die Relevanz des bereitgestellten Kontexts und gibt dem Benutzer mehr Kontrolle.

---

## Implementierung in NovaDE

Innerhalb der NovaDE-Architektur würde dies folgende neue oder erweiterte Komponenten bedeuten:

1.  **`novade-domain::local_knowledge_service` (Neu):**
    * **Verantwortlichkeit:**
        * Verwaltung des Zugriffs auf den zentralen Wissensspeicher.
        * Orchestrierung des Embedding-Prozesses (ggf. unter Nutzung eines `embedding_service` in `novade-system`).
        * Schnittstelle zur Vektordatenbank für die semantische Suche (RAG).
        * Pflege der standardisierten Konversationshistorie.
    * **Kern-Aufgaben:**
        * Definieren von Typen für `KnowledgeItem`, `KnowledgeChunk`, `ConversationTurn`.
        * Funktionen zum Hinzufügen, Indexieren und Abrufen von Wissen.
        * Funktionen zum Speichern und Abrufen von Konversationshistorien.
    * **Spezifische Artefakte/Dateien:**
        * `novade-domain/src/local_knowledge_service/mod.rs`
        * `novade-domain/src/local_knowledge_service/types.rs` (z.B. `StoredKnowledgeItem`, `ConversationLog`)
        * `novade-domain/src/local_knowledge_service/retriever.rs` (Logik für RAG)
        * `novade-domain/src/local_knowledge_service/history_manager.rs`
    * **Abhängigkeiten:**
        * Intern: `novade-system::vector_db_client` (neu), `novade-system::file_storage_service` (ggf. Erweiterung eines bestehenden Dienstes), `novade-core::config`
        * Extern: Bibliotheken für die Interaktion mit der gewählten Vektordatenbank.

2.  **`novade-system::vector_db_client` (Neu oder als Teil eines generischen DB-Service):**
    * **Verantwortlichkeit:** Technische Anbindung an die gewählte Vektordatenbank-Implementierung (lokal oder Cloud). Kapselt die spezifischen API-Aufrufe.
    * **Kern-Aufgaben:** Verbindungsaufbau, Indexerstellung, Vektor-Upload, Ähnlichkeitssuche.

3.  **`novade-system::embedding_service` (Neu oder als Teil eines KI-Utility-Service):**
    * **Verantwortlichkeit:** Technische Anbindung an ein Embedding-Modell (lokal oder API-basiert) zur Umwandlung von Text in Vektoren.
    * **Kern-Aufgaben:** Bereitstellung einer Funktion `fn generate_embeddings(texts: Vec<String>) -> Result<Vec<EmbeddingVector>, Error>`.

4.  **Erweiterung des `novade-domain::mcp_integration_service` (aus vorheriger Antwort):**
    * **Verantwortlichkeit (Erweiterung):**
        * Vor dem Senden einer Anfrage an ein MCP-Modell: Abrufen relevanten Wissens vom `local_knowledge_service`.
        * Abrufen der aktuellen Konversationshistorie vom `local_knowledge_service`.
        * Nutzung der **modellspezifischen Prompt-Engineering-Schicht/Adapter**, um den finalen Prompt für das ausgewählte Modell zu erstellen. Dieser Adapter kennt die Spezifika des aktuellen Modells (z.B. Name, Token-Limit, bevorzugte Prompt-Struktur).
        * Speichern der Antwort des Modells und der Metadaten (welches Wissen wurde abgerufen etc.) über den `local_knowledge_service` in der Konversationshistorie.
    * **Kern-Aufgaben (Erweiterung):**
        * Integration mit `local_knowledge_service`.
        * Implementierung oder Nutzung einer Adapter-Logik, die pro unterstütztem Modell (oder Modellfamilie) spezifische Prompt-Formatierungsregeln anwendet.

### Beispielhafter Ablauf beim Modellwechsel:

1.  Der Benutzer interagiert mit **Modell A**. Die Konversation und das abgerufene Wissen werden im standardisierten Format über den `local_knowledge_service` gespeichert.
2.  Der Benutzer entscheidet sich, zu **Modell B** zu wechseln.
3.  Der `mcp_integration_service` wird über den Wechsel informiert.
4.  Für die nächste Anfrage des Benutzers:
    * Der `mcp_integration_service` ruft die standardisierte Konversationshistorie und relevantes Wissen (via RAG) vom `local_knowledge_service` ab. Diese Daten sind **identisch**, unabhängig davon, ob vorher Modell A oder B aktiv war.
    * Der `mcp_integration_service` verwendet nun den **Adapter für Modell B**, um diese Informationen in einen optimalen Prompt für Modell B zu formatieren (unter Berücksichtigung von dessen Kontextfenster, speziellen Tokens etc.).
    * Die Anfrage wird an Modell B gesendet.
5.  Die Interaktion setzt sich fort, wobei der Kontext effektiv an Modell B übergeben wurde.

Diese Kombination aus zentraler, modellunabhängiger Wissensspeicherung (insbesondere mittels RAG) und einer flexiblen, modellspezifischen Aufbereitungsschicht ist der Schlüssel, um den Kontextverlust beim Wechsel zwischen verschiedenen LLMs zu minimieren und gleichzeitig die Stärken jedes einzelnen Modells auszunutzen. Die Forschungsergebnisse (z.B. die Relevanz von RAG auch bei langen Kontextfenstern, wie auf Reddit und in Blogs wie Databricks diskutiert) stützen diesen Ansatz. Auch die Idee von "OpenMemory MCP" (mem0.ai) deutet auf die Nützlichkeit von externen, protokollbasierten Gedächtnisschichten hin.

## Integration des Model Context Protocol (MCP) in NovaDE

Die Integration des MCP erfolgt primär durch die Einführung eines neuen Dienstes in der Domänenschicht (`novade-domain`), der die Kernlogik des Protokolls handhabt. Dieser Dienst wird von Komponenten in der Systemschicht (`novade-system`) für die externe Kommunikation und von der UI-Schicht (`novade-ui`) für die Benutzerinteraktion unterstützt.

---

### PHASE 1: VERZEICHNISSTRUKTUR SPEZIFIKATION

1.  **Domänenschicht (`novade-domain`):**
    * `novade-domain/src/mcp_integration_service/`: Hauptverzeichnis für die MCP-Integrationslogik.
        * `novade-domain/src/mcp_integration_service/mod.rs`: Hauptmodul des MCP-Integrationsdienstes.
        * `novade-domain/src/mcp_integration_service/config.rs`: Konfigurationstypen für den MCP-Dienst (z.B. Server-Listen, Timeouts).
        * `novade-domain/src/mcp_integration_service/types.rs`: Kern-Datentypen für MCP-Interaktionen (Requests, Responses, Consent-Objekte etc.), basierend auf der MCP-Spezifikation.
        * `novade-domain/src/mcp_integration_service/protocol_handler.rs`: Logik zur Verarbeitung und Erstellung von MCP-Nachrichten.
        * `novade-domain/src/mcp_integration_service/consent_manager.rs`: Verwaltung von Benutzereinwilligungen für MCP-Aktionen (Tool-Nutzung, Ressourcenzugriff).
        * `novade-domain/src/mcp_integration_service/session_manager.rs`: Verwaltung aktiver Sitzungen mit MCP-Servern.
        * `novade-domain/src/mcp_integration_service/error.rs`: Fehlerdefinitionen spezifisch für den MCP-Dienst.
        * `novade-domain/src/mcp_integration_service/events.rs`: Definition von Events, die vom MCP-Dienst publiziert werden (z.B. `MCPActionRequested`, `MCPDataReceived`).

2.  **Systemschicht (`novade-system`):**
    * `novade-system/src/mcp_client_service/`: Verzeichnis für den Dienst, der die Kommunikation mit externen MCP-Servern abwickelt.
        * `novade-system/src/mcp_client_service/mod.rs`: Hauptmodul des MCP-Client-Dienstes.
        * `novade-system/src/mcp_client_service/client.rs`: Implementierung des HTTP-Clients oder einer anderen Transportmethode für MCP.
        * `novade-system/src/mcp_client_service/error.rs`: Fehlerdefinitionen für den MCP-Client.
        * `novade-system/src/mcp_client_service/sandbox_executor.rs`: (Optional, falls Tools lokal ausgeführt werden) Schnittstelle zu Sandboxing-Mechanismen für die sichere Ausführung von MCP-Tools.

3.  **UI-Schicht (`novade-ui`):**
    * `novade-ui/src/mcp_components/`: Verzeichnis für UI-Komponenten, die MCP-Interaktionen betreffen.
        * `novade-ui/src/mcp_components/consent_dialog.rs`: UI-Komponente zur Anzeige von Einwilligungsanfragen und zur Entgegennahme der Benutzerentscheidung.
        * `novade-ui/src/mcp_components/mcp_view_widgets.rs`: Widgets zur Darstellung von Daten, die von MCP-Servern empfangen werden.
        * `novade-ui/src/mcp_components/mcp_manager_panel.rs`: (Optional) Ein Panel zur Verwaltung von MCP-Verbindungen und -Einstellungen.

---

### PHASE 2: MODUL DEFINITIONEN (Fokus auf `novade-domain::mcp_integration_service`)

**Verzeichnis-/Modulname:** `novade-domain/src/mcp_integration_service/`
**Verantwortlichkeit:** Orchestriert die gesamte MCP-Funktionalität innerhalb von NovaDE. Dient als zentrale Anlaufstelle für MCP-bezogene Operationen, verwaltet Zustände, Einwilligungen und die Kommunikation mit MCP-Servern über die Systemschicht. Stellt sicher, dass alle Interaktionen den Prinzipien des MCP (Benutzerkontrolle, Sicherheit) entsprechen.

**Kern-Aufgaben (Tasks):**

1.  **MCPKonfiguration (`config.rs`) definieren:**
    * `MCPServiceConfig`: Struktur zur Aufnahme von Konfigurationsparametern.
        * `known_mcp_servers: Vec<ServerInfo>`: Liste bekannter MCP-Server mit Adressen und Metadaten.
        * `default_request_timeout_ms: u64`: Standard-Timeout für Anfragen an MCP-Server.
    * `ServerInfo`: Struktur zur Beschreibung eines MCP-Servers.
        * `id: String`: Eindeutige ID des Servers.
        * `address: String`: Netzwerkadresse des Servers.
        * `name: String`: Anzeigename des Servers.
        * `description: Option<String>`: Optionale Beschreibung des Servers.

2.  **MCP-Kerntypen (`types.rs`) definieren (Auszug, basierend auf MCP-Spezifikation):**
    * `MCPRequest`: Enum für verschiedene Anfragetypen an einen MCP-Server (z.B. `InitiateSession`, `ContextUpdate`, `ToolResponse`, `ResourceResponse`, `ConsentGrant`).
        * Jede Variante enthält die notwendigen Datenstrukturen gemäß MCP-Spezifikation.
    * `MCPResponse`: Enum für verschiedene Antworttypen von einem MCP-Server (z.B. `SessionEstablished`, `ToolRequest`, `ResourceRequest`, `ContextRequired`).
        * Jede Variante enthält die notwendigen Datenstrukturen gemäß MCP-Spezifikation.
    * `MCPTool`: Struktur zur Beschreibung eines Werkzeugs, das von einem MCP-Server angeboten wird.
        * `id: String`, `name: String`, `description: String`, `input_schema: JsonSchema`, `output_schema: JsonSchema`.
    * `MCPResource`: Struktur zur Beschreibung einer Ressource, die von einem MCP-Server angefragt wird.
        * `id: String`, `name: String`, `description: String`, `access_level_required: String`.
    * `MCPConsent`: Enum zur Darstellung des Einwilligungsstatus.
        * `Pending { request_id: String, tool_id: Option<String>, resource_id: Option<String>, details: String }`
        * `Granted { request_id: String, validity_duration_secs: Option<u64> }`
        * `Denied { request_id: String }`
        * `Revoked { original_request_id: String }`

3.  **MCP Protokoll Handler (`protocol_handler.rs`) implementieren:**
    * `MCPProtocolHandler`: Struktur ohne Felder, nur Methoden.
    * `parse_mcp_response(raw_data: Vec<u8>) -> Result<MCPResponse, MCPError>`:
        * Deserialisiert Rohdaten (z.B. JSON) in eine `MCPResponse`-Struktur.
        * Validiert die Struktur gegen die MCP-Spezifikation.
        * Gibt `MCPError::DeserializationError` oder `MCPError::InvalidMessageFormat` bei Fehlern zurück.
    * `serialize_mcp_request(request: &MCPRequest) -> Result<Vec<u8>, MCPError>`:
        * Serialisiert eine `MCPRequest`-Struktur in Rohdaten (z.B. JSON).
        * Gibt `MCPError::SerializationError` bei Fehlern zurück.

4.  **MCP Einwilligungsmanager (`consent_manager.rs`) implementieren:**
    * `MCPConsentManager`: Struktur.
        * `active_consents: HashMap<String, MCPConsent>`: Speichert erteilte Einwilligungen (z.B. pro `request_id` oder einer Kombination aus Server, Tool/Ressource).
        * `pending_consent_requests: HashMap<String, MCPConsentRequestDetails>`: Speichert Anfragen, die auf Benutzerinteraktion warten.
    * `request_consent(request_details: MCPConsentRequestDetails) -> Result<String, MCPError>`:
        * Generiert eine eindeutige `request_id`.
        * Speichert die `request_details` unter der `request_id` in `pending_consent_requests`.
        * Löst ein Event aus (z.B. `MCPEvent::ConsentUIDisplayRequested(request_id, details)`), das von der UI-Schicht abgefangen wird, um einen Einwilligungsdialog anzuzeigen.
        * Gibt die `request_id` zurück.
    * `grant_consent(request_id: String, user_decision: bool, validity_duration_secs: Option<u64>) -> Result<(), MCPError>`:
        * Wird von der UI-Schicht aufgerufen nach Benutzerinteraktion.
        * Entfernt Anfrage aus `pending_consent_requests`.
        * Wenn `user_decision` true ist, wird ein `MCPConsent::Granted` Objekt erstellt und in `active_consents` gespeichert.
        * Informiert den `MCPIntegrationService` über die erteilte oder verweigerte Einwilligung, um die entsprechende `ConsentGrant`-Nachricht an den MCP-Server zu senden.
        * Gibt `MCPError::ConsentRequestNotFound` oder `MCPError::InvalidState` bei Fehlern zurück.
    * `check_consent(server_id: &str, tool_id: Option<&str>, resource_id: Option<&str>) -> bool`:
        * Prüft, ob eine gültige Einwilligung für die gegebene Kombination vorliegt.
    * `revoke_consent(consent_id: String) -> Result<(), MCPError>`:
        * Entfernt eine Einwilligung aus `active_consents`.

5.  **MCP Session Manager (`session_manager.rs`) implementieren:**
    * `MCPSessionManager`: Struktur.
        * `active_sessions: HashMap<String, MCPSession>`: Speichert aktive Sitzungen mit MCP-Servern (Schlüssel ist `server_id`).
    * `MCPSession`: Struktur.
        * `server_id: String`, `session_id: String`, `connection_status: ConnectionStatus`, `last_activity_ts: u64`.
    * `establish_session(server_info: &ServerInfo) -> Result<String, MCPError>`:
        * Sendet eine `MCPRequest::InitiateSession` über den `mcp_client_service` an den Server.
        * Verarbeitet die `MCPResponse::SessionEstablished` und speichert die Sitzungsinformationen.
        * Gibt die `session_id` zurück.
        * Fehlerbehandlung für Netzwerkprobleme oder Ablehnung durch den Server.
    * `close_session(server_id: String) -> Result<(), MCPError>`:
        * Sendet eine `MCPRequest::EndSession` (falls im Protokoll spezifiziert) oder schließt die Verbindung.
        * Entfernt die Sitzung aus `active_sessions`.

6.  **MCP Integration Service (`mod.rs`) implementieren:**
    * `MCPIntegrationService`: Hauptstruktur, die Instanzen von `MCPConsentManager`, `MCPSessionManager`, `MCPProtocolHandler` und eine Referenz auf den `mcp_client_service` (aus der Systemschicht) hält.
    * `new(config: MCPServiceConfig, client_service: Arc<dyn IMCPClientService>) -> Self`: Konstruktor.
    * `process_incoming_mcp_message(server_id: String, raw_data: Vec<u8>) -> Result<(), MCPError>`:
        * Wird vom `mcp_client_service` aufgerufen, wenn eine Nachricht von einem MCP-Server empfangen wird.
        * Nutzt `MCPProtocolHandler::parse_mcp_response`.
        * Basierend auf dem `MCPResponse`-Typ:
            * `ToolRequest`: Ruft `consent_manager.request_consent` auf. Bei Einwilligung wird die Anfrage zur Bearbeitung weitergeleitet (ggf. an ein spezifisches NovaDE-Tool-Modul oder an den `sandbox_executor` in der Systemschicht).
            * `ResourceRequest`: Ruft `consent_manager.request_consent` auf. Bei Einwilligung wird der Zugriff auf die Ressource gewährt und die Daten an den MCP-Server gesendet.
            * `ContextRequired`: Sammelt die angeforderten Kontextinformationen (gemäß Einwilligung) und sendet sie als `ContextUpdate`.
            * Andere Nachrichtentypen entsprechend verarbeiten.
    * `send_mcp_message(server_id: String, request: MCPRequest) -> Result<(), MCPError>`:
        * Nutzt `MCPProtocolHandler::serialize_mcp_request`.
        * Sendet die serialisierte Nachricht über den `mcp_client_service`.
    * `handle_ui_consent_response(request_id: String, granted: bool, validity: Option<u64>)`:
        * Aktualisiert den Consent-Status über den `consent_manager`.
        * Wenn gewährt, wird die ursprüngliche `ToolRequest` oder `ResourceRequest` weiterverarbeitet oder eine `ConsentGrant` Nachricht an den MCP Server gesendet.

**Spezifische Artefakte/Dateien:**
* `novade-domain/src/mcp_integration_service/mod.rs`: Hauptmodul, öffentliche API des Dienstes.
* `novade-domain/src/mcp_integration_service/config.rs`: Enthält `MCPServiceConfig` und `ServerInfo`.
* `novade-domain/src/mcp_integration_service/types.rs`: Enthält `MCPRequest`, `MCPResponse`, `MCPTool`, `MCPResource`, `MCPConsent`.
* `novade-domain/src/mcp_integration_service/protocol_handler.rs`: Enthält `MCPProtocolHandler` mit Serialisierungs-/Deserialisierungslogik.
* `novade-domain/src/mcp_integration_service/consent_manager.rs`: Enthält `MCPConsentManager` und zugehörige Logik.
* `novade-domain/src/mcp_integration_service/session_manager.rs`: Enthält `MCPSessionManager` und `MCPSession`.
* `novade-domain/src/mcp_integration_service/error.rs`: Enthält `MCPError` Enum.
* `novade-domain/src/mcp_integration_service/events.rs`: Enthält `MCPEvent` Enum.

**Abhängigkeiten:**
* Intern:
    * `novade-core::errors` für Basis-Fehlertypen.
    * `novade-core::types` für generische Typen (falls benötigt).
    * (Potenziell) `novade-domain::global_settings_and_state_management` für globale Konfigurationen.
    * (Potenziell) `novade-domain::notifications_core` zur Benachrichtigung des Benutzers über MCP-Ereignisse.
* Extern:
    * `serde` (mit `serde_json` oder einem anderen Format gemäß MCP-Spezifikation) für Serialisierung/Deserialisierung.
    * `json_schema` (oder ähnliches) zur Validierung von Tool-Schemata.
    * Logging-Framework (z.B. `tracing`).

**Kommunikationsmuster:**
* Inbound:
    * Empfängt Anfragen von der UI-Schicht (z.B. zum Starten einer MCP-Interaktion, Benutzerentscheidungen zu Einwilligungen).
    * Empfängt Nachrichten von MCP-Servern über den `mcp_client_service` aus der Systemschicht.
* Outbound:
    * Sendet Anfragen/Antworten an MCP-Server über den `mcp_client_service`.
    * Sendet Events an die UI-Schicht (z.B. um Einwilligungsdialoge anzuzeigen, Daten zu aktualisieren).
    * Interagiert mit anderen Domänendiensten, um Kontextinformationen zu sammeln oder Aktionen auszuführen (immer unter Beachtung der Einwilligungen).

**Erwartete Ergebnisse/Outputs:**
* Eine robuste und sichere Implementierung der MCP-Host-Funktionalität.
* Klare Trennung von Protokolllogik, Einwilligungsmanagement und Sitzungsverwaltung.
* Ermöglichung von KI-gestützten Funktionen in NovaDE unter voller Benutzerkontrolle.
* Bereitstellung einer klaren Schnittstelle für die UI- und Systemschicht zur Interaktion mit MCP.

---

### PHASE 2: MODUL DEFINITIONEN (Kurzübersicht für System- und UI-Schicht)

**1. Verzeichnis-/Modulname:** `novade-system/src/mcp_client_service/`
    * **Verantwortlichkeit:** Abstrahiert die Netzwerkkommunikation mit MCP-Servern. Stellt Methoden zum Senden und Empfangen von MCP-Nachrichten bereit. Implementiert ggf. Wiederholungslogik, Timeout-Management und sichere Verbindungen (TLS). Kann auch die Schnittstelle zum `sandbox_executor` bereitstellen.
    * **Kern-Aufgaben:** Implementierung von `IMCPClientService` (Trait), HTTP/WebSocket-Client-Logik, Fehlerbehandlung für Netzwerkprobleme.
    * **Spezifische Artefakte/Dateien:** `client.rs` (Implementierung), `mod.rs` (öffentliche API), `error.rs`.
    * **Abhängigkeiten:** HTTP-Client-Bibliothek (z.B. `reqwest`, `hyper`), Tokio für asynchrone Operationen.
    * **Kommunikationsmuster:** Empfängt Sendeanfragen vom `MCPIntegrationService`, sendet Anfragen an externe MCP-Server, leitet empfangene Daten an den `MCPIntegrationService` weiter.

**2. Verzeichnis-/Modulname:** `novade-ui/src/mcp_components/`
    * **Verantwortlichkeit:** Stellt GTK4-basierte UI-Komponenten für die MCP-Interaktion bereit. Zeigt Einwilligungsdialoge an, visualisiert Daten von MCP-Servern und ermöglicht dem Benutzer die Verwaltung von MCP-Einstellungen.
    * **Kern-Aufgaben:** Implementierung von `ConsentDialog`, Widgets zur Anzeige von `MCPTool`-Informationen, `MCPResource`-Anfragen und Ergebnissen. Verbindung mit dem `MCPIntegrationService` über Events oder direkte Aufrufe.
    * **Spezifische Artefakte/Dateien:** `consent_dialog.rs`, `mcp_view_widgets.rs`.
    * **Abhängigkeiten:** `gtk4-rs`, `relm4` (oder das verwendete UI-Framework), Abonnement von Events des `MCPIntegrationService`.
    * **Kommunikationsmuster:** Reagiert auf Events vom `MCPIntegrationService` (z.B. `ConsentUIDisplayRequested`), sendet Benutzerentscheidungen (Einwilligungen) an den `MCPIntegrationService`.

---

### PHASE 3: TYP SPEZIFIKATION (Beispiele bereits in Phase 2 unter `types.rs` angedeutet)

Für alle Datenstrukturen (z.B. `MCPRequest`, `MCPResponse`, `MCPConsent`, `MCPTool`, `ServerInfo`) gilt:
1.  **Vollständige Eigenschaftenlisten mit Typen:** Wie in `novade-domain/src/mcp_integration_service/types.rs` beschrieben. Alle Felder sind explizit typisiert.
2.  **Validierungseinschränkungen:**
    * Für `ServerInfo::address`: Muss ein gültiges URI-Format sein.
    * Für `MCPTool::input_schema` und `output_schema`: Müssen gültige JSON-Schemata sein.
    * Längenbeschränkungen für Strings (z.B. IDs, Namen) können definiert werden.
3.  **Serialisierungs-/Deserialisierungsanforderungen:**
    * Alle Typen, die über das Netzwerk gesendet oder empfangen werden, müssen `serde::Serialize` und `serde::Deserialize` implementieren. Das Format ist typischerweise JSON, wie im MCP-Dokument angedeutet.
4.  **Memory Layout und Alignment:** In Rust meist durch den Compiler gehandhabt; bei FFI oder kritischen Performance-Pfaden ggf. `#[repr(C)]` oder ähnliches, hier aber voraussichtlich nicht primär relevant.
5.  **Lifetime und Ownership Semantics:** Standard Rust Ownership-Regeln. `Arc` und `Mutex`/`RwLock` werden für gemeinsam genutzte Zustände (z.B. in Managern) verwendet, um Thread-Sicherheit zu gewährleisten.

---

### PHASE 4: ALGORITHMUS SPEZIFIKATION (Beispiel für Einwilligungsanfrage)

**Funktion:** `MCPIntegrationService::process_incoming_mcp_message` (für den Fall `MCPResponse::ToolRequest`)

1.  **Algorithmische Schritte:**
    1.  Deserialisiere die eingehende Nachricht mit `MCPProtocolHandler::parse_mcp_response`. Wenn fehlerhaft, protokolliere Fehler und beende.
    2.  Prüfe, ob die deserialisierte Nachricht vom Typ `MCPResponse::ToolRequest` ist.
    3.  Extrahiere die `ToolRequestDetails` (enthält `tool_id`, `server_id`, `request_parameters`, `description_for_user`).
    4.  Überprüfe mit `MCPConsentManager::check_consent` (unter Verwendung von `server_id` und `tool_id`), ob bereits eine gültige, spezifische Einwilligung für dieses Tool von diesem Server vorliegt.
        * **Fall A: Einwilligung vorhanden und gültig:**
            1.  Bereite die `ToolExecutionParameters` vor.
            2.  Leite die Anfrage zur Ausführung weiter (z.B. an einen internen Tool-Handler oder via `mcp_client_service` an einen `sandbox_executor` falls das Tool so ausgeführt wird).
            3.  Nach der Ausführung: Forme das Ergebnis in eine `MCPRequest::ToolResponse` um und sende diese über `send_mcp_message` an den anfragenden MCP-Server.
        * **Fall B: Keine Einwilligung oder abgelaufen:**
            1.  Erstelle `MCPConsentRequestDetails` basierend auf den `ToolRequestDetails`.
            2.  Rufe `MCPConsentManager::request_consent` mit diesen Details auf. Dies speichert die Anfrage und löst ein UI-Event aus (`MCPEvent::ConsentUIDisplayRequested`). Die `request_id` wird zurückgegeben und für die spätere Zuordnung der Benutzerantwort gespeichert.
2.  **Verzweigungsbedingungen:**
    * Erfolg/Fehler der Deserialisierung.
    * Typ der MCP-Nachricht.
    * Vorhandensein und Gültigkeit einer bestehenden Einwilligung.
3.  **Fehlerbehandlung:**
    * `MCPError::DeserializationError`: Wenn die Nachricht nicht geparst werden kann.
    * `MCPError::InvalidMessageFormat`: Wenn die geparste Nachricht nicht der Erwartung entspricht.
    * `MCPError::ConsentDenied`: Wenn der Benutzer die Einwilligung verweigert (behandelt im Callback von der UI).
    * `MCPError::ToolExecutionFailed`: Wenn die Ausführung des Tools fehlschlägt.
    * Alle Fehler werden protokolliert. Kritische Fehler können zu Benachrichtigungen an den Benutzer führen.
4.  **Thread-Sicherheit:** Da der `MCPIntegrationService` und seine Manager (Consent, Session) von verschiedenen Threads genutzt werden können (Netzwerk-Callbacks, UI-Events), müssen interne Zustände (z.B. `active_consents`, `active_sessions`) mit `Arc<Mutex<...>>` oder `Arc<RwLock<...>>` geschützt werden.
5.  **Ressourcenmanagement:**
    * Netzwerkverbindungen werden vom `mcp_client_service` verwaltet.
    * Abgelaufene Einwilligungen und Sitzungen sollten periodisch oder bei Bedarf aufgeräumt werden.

---

Diese detaillierte Spezifikation legt den Grundstein für eine "optimale" Integration des MCP in NovaDE, die dessen Kernprinzipien von Sicherheit, Transparenz und Benutzerkontrolle vollständig berücksichtigt und gleichzeitig eine flexible und leistungsfähige KI-Interaktion ermöglicht. Die Umsetzung erfordert sorgfältige Implementierung aller beschriebenen Komponenten und deren Interaktionen.






Die tiefe Integration des Model Context Protocol (MCP) in die beschriebene Desktop-Umgebung eröffnet eine Vielzahl von Möglichkeiten, um eine KI-gestützte, aber stets benutzerkontrollierte und sichere Erfahrung zu schaffen. MCP fungiert als standardisierte Schnittstelle ("USB-Port für KI" ), die es der Desktop-Umgebung (dem "Host" ) ermöglicht, sicher und flexibel mit verschiedenen KI-Modellen (LLMs) sowie externen Datenquellen und Werkzeugen über sogenannte MCP-Server zu kommunizieren.

Hier sind die Kernmöglichkeiten, die sich daraus ergeben:

1. **Sichere und kontrollierte KI-Interaktionen:**
    
    - **Benutzereinwilligung im Zentrum:** Das Kernprinzip ist, dass keine KI-Aktion (Tool-Nutzung) oder kein Datenzugriff (Ressourcen-Nutzung) ohne die explizite, informierte Zustimmung des Benutzers erfolgen darf. Der Desktop als Host ist dafür verantwortlich, diese Einwilligungen über klare Dialoge einzuholen.
        
    - **Granulare Kontrolle:** Benutzer behalten die Kontrolle darüber, welche Daten geteilt und welche Aktionen ausgeführt werden. Dies schließt auch die fortgeschrittene "Sampling"-Funktion ein, bei der der Server LLM-Interaktionen anstoßen kann – auch hier ist explizite Benutzerkontrolle unerlässlich.
        
    - **Datenschutz:** Der Host stellt sicher, dass Benutzerdaten gemäß den erteilten Einwilligungen geschützt und nicht unbefugt weitergegeben werden.
        
2. **Zugriff auf externe Werkzeuge (Tools):**
    
    - **KI-gesteuerte Aktionen:** LLMs können über MCP definierte "Tools" aufrufen, um Aktionen in externen Systemen auszuführen. Die Entscheidung zur Tool-Nutzung trifft primär das LLM basierend auf der Benutzeranfrage.
        
    - **Vielfältige Anwendungsfälle:** Beispiele reichen vom Senden von E-Mails, Erstellen von Kalendereinträgen bis hin zur Interaktion mit Diensten wie GitHub (Issues erstellen/lesen, Code suchen) oder anderen APIs.
        
3. **Nutzung externer Datenquellen (Resources):**
    
    - **Kontextanreicherung:** LLMs können über "Resources" auf Daten aus externen Quellen zugreifen, um ihre Antworten mit aktuellem oder spezifischem Kontext anzureichern. Dies geschieht primär lesend, ohne Seiteneffekte.
        
    - **Beispiele:** Abruf von Benutzerprofilen, Produktkatalogen, Dokumentinhalten, Kalenderdaten oder auch (mit Zustimmung) lokalen Dateien.
        
4. **Standardisierte Interaktionsmuster (Prompts):**
    
    - **Benutzergeführte Interaktion:** "Prompts" sind vordefinierte Vorlagen, die der Benutzer (über den Desktop-Host) auswählen kann, um Interaktionen mit Tools oder Ressourcen optimal und standardisiert zu gestalten.
        
    - **Anwendungsbeispiele:** Standardisierte Abfragen (z.B. "Fasse Pull Request X zusammen"), geführte Workflows oder häufig genutzte Befehlssätze, die in der UI als Buttons o.ä. erscheinen können.
        
5. **Ermöglichung intelligenter Agenten (Sampling):**
    
    - **Proaktive KI:** Die "Sampling"-Funktion erlaubt es einem MCP-Server (mit expliziter Zustimmung und Kontrolle des Benutzers), das LLM über den Desktop-Client proaktiv zu Interaktionen aufzufordern.
        
    - **Potenzial:** Dies ermöglicht intelligentere, proaktivere Agenten, die auf externe Ereignisse reagieren oder komplexe, mehrstufige Aufgaben ausführen können. Aufgrund des hohen Potenzials für Missbrauch unterliegt diese Funktion strengsten Kontrollanforderungen.
        

**Konkrete Beispiele im Desktop-Kontext:**

- **Intelligente Sprachsteuerung:** Benutzer können Befehle wie "Öffne den Dateimanager" oder "Aktiviere den Dunkelmodus" sprechen. Die KI interpretiert dies und nutzt (nach Zustimmung) interne MCP-Tools, um die Desktop-Funktionen zu steuern.
- **Automatisierte Dateibearbeitung/-analyse:** Die KI kann (nach expliziter Freigabe durch den Benutzer) Inhalte von Dokumenten zusammenfassen, Daten analysieren oder Textentwürfe erstellen, indem sie auf das Dateisystem als MCP-Ressource zugreift oder spezielle Analyse-Tools nutzt.
- **Kontextbezogene Webansichten/Widgets:** Widgets können, gesteuert durch die KI und MCP, relevante Informationen aus dem Web oder anderen Quellen anzeigen, die zum aktuellen Arbeitskontext passen (z.B. über ein Web-Such-Tool ).
    
- **Entwickler-Workflows:** Direkte Interaktion mit GitHub aus der IDE/Desktop-Umgebung heraus, z.B. zum Zusammenfassen von Issues oder Analysieren von Pull Requests über einen GitHub-MCP-Server.
    

Zusammenfassend ermöglicht die tiefe MCP-Integration eine leistungsstarke und flexible KI-Unterstützung direkt im Desktop, wobei durch das Protokoll-Design und die Host-Implementierung Sicherheit und Benutzerkontrolle stets gewährleistet bleiben. Es standardisiert die Anbindung externer Fähigkeiten und Daten, reduziert die Integrationskomplexität und schafft die Basis für vielfältige, kontextbewusste KI-Anwendungen
# Entwicklungsrichtlinien und Spezifikation für die Integration und Implementierung des Model Context Protocol (MCP)

**Präambel:** Dieses Dokument dient als maßgebliche Ressource für Entwickler, die das Model Context Protocol (MCP) integrieren oder implementieren. Es legt die Spezifikationen des Protokolls dar und bietet detaillierte Richtlinien zur Gewährleistung robuster, sicherer und interoperabler Implementierungen. Die in diesem Dokument verwendeten Schlüsselwörter “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”,...[source](https://www.funkschau.de/office-kommunikation/sip-oder-sip-ein-protokoll-bereitet-probleme.82250/seite-4.html) “NOT RECOMMENDED”, “MAY”, und “OPTIONAL” sind gemäß BCP 14, zu interpretieren, wenn sie in Großbuchstaben erscheinen.1

**1. Einführung in das Model Context Protocol (MCP)**

Das Model Context Protocol (MCP) stellt einen Paradigmenwechsel in der Art und Weise dar, wie KI-Systeme mit externen Daten und Werkzeugen interagieren. Es wurde entwickelt, um die wachsenden Herausforderungen der Integration von Large Language Models (LLMs) in komplexe Anwendungslandschaften zu bewältigen.

- **1.1. Zweck und Vision des MCP**

Das Model Context Protocol (MCP), eingeführt von Anthropic Ende 2024, ist ein bahnbrechender offener Standard, der konzipiert wurde, um die Lücke zwischen KI-Assistenten und den datenreichen Ökosystemen, in denen sie operieren müssen, zu schließen.2 Die Kernvision des MCP besteht darin, die oft fragmentierten und ad-hoc entwickelten Integrationen durch ein universelles Framework zu ersetzen. Dieses Framework ermöglicht es KI-Systemen, nahtlos auf diverse Kontexte zuzugreifen und mit externen Tools und Datenquellen zu interagieren.2

Das primäre Ziel des MCP ist die Standardisierung der Art und Weise, wie KI-Anwendungen – seien es Chatbots, in IDEs integrierte Assistenten oder benutzerdefinierte Agenten – Verbindungen zu externen Werkzeugen, Datenquellen und Systemen herstellen.3 Man kann sich MCP als eine Art "USB-Port" für KI-Anwendungen vorstellen: eine universelle Schnittstelle, die es jedem KI-Assistenten erlaubt, sich ohne spezifischen Code für jede einzelne Verbindung an jede Datenquelle oder jeden Dienst anzuschließen.4

Die Bedeutung des MCP liegt in seiner Fähigkeit, die Reproduzierbarkeit von KI-Ergebnissen zu verbessern, indem der gesamte Modellkontext – Datensätze, Umgebungsspezifikationen und Hyperparameter – an einem Ort zusammengeführt wird. Darüber hinaus fördert es die Standardisierung und erleichtert die organisationsübergreifende Zusammenarbeit, da Unternehmen spezialisierte KI-Tools oder benutzerdefinierte Datenquellen auf einer gemeinsamen Basis teilen können.2

- **1.2. Kernvorteile für Entwickler und Organisationen**

Die Einführung und Adaption des MCP bietet signifikante Vorteile für Entwicklerteams und die Organisationen, in denen sie tätig sind. Diese Vorteile manifestieren sich in Effizienzsteigerungen, beschleunigter Innovation und verbesserter Systemstabilität.

Ein zentraler Vorteil ist die **Reduzierung der Integrationskomplexität**. Traditionell stehen Entwickler vor einem M×N-Integrationsproblem: M verschiedene KI-Anwendungen müssen mit N verschiedenen Tools oder Systemen (wie GitHub, Slack, Datenbanken etc.) verbunden werden. Dies führt oft zu M×N individuellen Integrationen, was erheblichen Mehraufwand, duplizierte Arbeit über Teams hinweg und inkonsistente Implementierungen zur Folge hat. MCP zielt darauf ab, dies zu vereinfachen, indem es das Problem in ein "M+N-Problem" transformiert: Tool-Ersteller entwickeln N MCP-Server (einen für jedes System), während Anwendungsentwickler M MCP-Clients (einen für jede KI-Anwendung) erstellen.3 Dieser Ansatz stellt einen fundamentalen Effizienzgewinn dar, da er die Notwendigkeit redundanter Integrationsarbeit eliminiert.

Direkt damit verbunden ist die **schnellere Tool-Integration**. MCP ermöglicht einen "Plug-and-Play"-Ansatz für die Anbindung neuer Fähigkeiten. Anstatt jede Integration von Grund auf neu zu entwickeln, können bestehende MCP-Server, die als standardisierte Schnittstellen für spezifische Tools oder Datenquellen dienen, einfach angebunden werden.6 Wenn beispielsweise ein MCP-Server für Google Drive oder eine SQL-Datenbank existiert, kann jede MCP-kompatible KI-Anwendung diesen Server nutzen und sofort die entsprechende Fähigkeit erlangen.6

Des Weiteren führt MCP zu einer **verbesserten Interoperabilität**. Indem es ein standardisiertes Protokoll bereitstellt, können verschiedene KI-gesteuerte Anwendungen dieselbe zugrundeliegende Infrastruktur für die Verbindung mit Tools, Ressourcen und Prompts nutzen.4 Dies bedeutet, dass Anfragen und Antworten über verschiedene Tools hinweg konsistent formatiert und gehandhabt werden, was die Entwicklung und Wartung vereinfacht.6

Schließlich **ermöglicht MCP die Entwicklung autonomerer Agenten**. KI-Agenten sind nicht länger auf ihr internes, vortrainiertes Wissen beschränkt. Sie können aktiv Informationen aus externen Quellen abrufen oder Aktionen in mehrstufigen, komplexen Workflows ausführen.6 Ein Agent könnte beispielsweise Daten aus einem CRM-System abrufen, darauf basierend eine E-Mail über ein Kommunikationstool senden und anschließend einen Eintrag in einer Datenbank protokollieren – alles über MCP-gesteuerte Interaktionen.6

Die Summe dieser Vorteile – reduzierte Komplexität, schnellere Integration, Interoperabilität und die Befähigung autonomer Agenten – positioniert MCP nicht nur als eine technische Verbesserung, sondern als einen fundamentalen Baustein. Dieser Baustein hat das Potenzial, die Entwicklung anspruchsvollerer, kontextbewusster und handlungsfähiger KI-Systeme maßgeblich voranzutreiben und zu beschleunigen. Für Unternehmen, die KI-gestützte Produkte entwickeln, ergeben sich hieraus strategische Implikationen hinsichtlich Entwicklungsgeschwindigkeit und Innovationsfähigkeit.

- **1.3. Abgrenzung zu bestehenden Standards**

Obwohl etablierte Standards wie OpenAPI, GraphQL oder SOAP für API-Interaktionen existieren und weit verbreitet sind, wurde das Model Context Protocol speziell mit den Anforderungen moderner KI-Agenten im Fokus entwickelt – es ist sozusagen "AI-Native".3 Während die genannten Standards primär auf den Datenaustausch zwischen Diensten ausgerichtet sind, adressiert MCP die spezifischen Bedürfnisse von LLMs, die nicht nur Daten konsumieren, sondern auch Aktionen ausführen und in komplexen, kontextabhängigen Dialogen agieren müssen.

MCP verfeinert und standardisiert Muster, die sich in der Entwicklung von KI-Agenten bereits abzeichnen. Eine Schlüsselunterscheidung ist die klare Trennung der exponierten Fähigkeiten in drei Kategorien: **Tools** (modellgesteuerte Aktionen), **Resources** (anwendungsgesteuerte Daten) und **Prompts** (benutzergesteuerte Interaktionsvorlagen).3 Diese granulare Unterscheidung ermöglicht eine feinere Steuerung und ein besseres Verständnis der Interaktionsmöglichkeiten eines LLMs mit seiner Umgebung, was über die typischen Request-Response-Zyklen traditioneller APIs hinausgeht.

- **1.4. Inspiration und Ökosystem-Vision**

Die Konzeption des MCP ist maßgeblich vom Language Server Protocol (LSP) inspiriert. Das LSP hat erfolgreich standardisiert, wie Entwicklungswerkzeuge (IDEs, Editoren) Unterstützung für verschiedene Programmiersprachen integrieren können, was zu einem florierenden Ökosystem von Sprachservern und kompatiblen Tools geführt hat.1

Analog dazu zielt MCP darauf ab, die Integration von zusätzlichem Kontext und externen Werkzeugen in das wachsende Ökosystem von KI-Anwendungen zu standardisieren.1 Diese Analogie deutet auf ein erhebliches Potenzial für eine breite Akzeptanz und das Wachstum einer aktiven Community hin. Die Vision ist ein Ökosystem, in dem Entwickler eine Vielzahl von vorgefertigten MCP-Servern für unterschiedlichste Dienste und Datenquellen finden und nutzen können, und ebenso einfach eigene Server bereitstellen können, die von einer breiten Palette von KI-Anwendungen konsumiert werden. Die "Offenheit" des Standards ist hierbei ein kritischer Erfolgsfaktor. Offene Standards, die nicht an einen einzelnen Anbieter gebunden sind und von einer Community weiterentwickelt werden können, fördern typischerweise eine breitere Akzeptanz.3 Die Existenz einer detaillierten und qualitativ hochwertigen Spezifikation, wie sie für MCP vorliegt 3, unterstreicht die Ernsthaftigkeit dieses offenen Ansatzes. Für Entwickler bedeutet dies eine höhere Wahrscheinlichkeit für langfristige Stabilität des Protokolls, eine größere Auswahl an kompatiblen Tools und Bibliotheken sowie die Möglichkeit, aktiv zum Ökosystem beizutragen. Eine Investition in MCP-Kenntnisse und -Implementierungen erscheint somit zukunftssicherer.

**2. MCP-Architektur und Komponenten**

Das Fundament des Model Context Protocol bildet eine klar definierte Architektur, die auf einem Client-Host-Server-Modell basiert. Dieses Modell strukturiert die Interaktionen und Verantwortlichkeiten der beteiligten Systeme und ist entscheidend für das Verständnis der Funktionsweise von MCP.

- **2.1. Das Client-Host-Server-Modell**

MCP verwendet ein Client-Host-Server-Muster, um die Kommunikation und den Austausch von "Kontext" zwischen KI-Anwendungen und externen Systemen zu standardisieren.2 Dieses Muster ist nicht nur eine technische Wahl, sondern eine grundlegende Designentscheidung, die Skalierbarkeit, Sicherheit und Wartbarkeit des Gesamtsystems beeinflusst. Die klare Trennung der Verantwortlichkeiten zwischen Host, Client und Server ermöglicht es, dass verschiedene Teams oder sogar Organisationen diese Komponenten unabhängig voneinander entwickeln und warten können. Dies ist ein direkter Lösungsansatz für das zuvor erwähnte M+N-Integrationsproblem.3

- Host (Anwendung):
    
    Der Host ist die primäre Anwendung, mit der der Endbenutzer direkt interagiert.3 Beispiele hierfür sind Desktop-Anwendungen wie Claude Desktop, integrierte Entwicklungsumgebungen (IDEs) wie Cursor oder auch speziell entwickelte, benutzerdefinierte KI-Agenten.3 Der Host fungiert als eine Art "Container" oder Koordinator für eine oder mehrere Client-Instanzen.2 Eine seiner zentralen Aufgaben ist die Verwaltung von Lebenszyklus- und Sicherheitsrichtlinien. Dies umfasst die Handhabung von Berechtigungen, die Benutzerautorisierung und insbesondere die Durchsetzung von Einwilligungsanforderungen für Datenzugriffe und Tool-Ausführungen.1 Diese Rolle ist kritisch für die Gewährleistung der Sicherheit und des Datenschutzes im MCP-Ökosystem. Der Host überwacht zudem, wie die Integration von KI- oder Sprachmodellen innerhalb jeder Client-Instanz erfolgt, und führt bei Bedarf Kontextinformationen aus verschiedenen Quellen zusammen.2 Eine unverzichtbare Anforderung an den Host ist, dass er die explizite Zustimmung des Benutzers einholen MUSS, bevor Benutzerdaten an MCP-Server weitergegeben werden.1
    
- Client (Konnektor):
    
    Der Client ist eine Komponente, die innerhalb der Host-Anwendung angesiedelt ist.2 Seine Hauptaufgabe ist die Verwaltung der Kommunikation und der Verbindung zu einem spezifischen MCP-Server. Es besteht eine strikte 1:1-Beziehung zwischen einer Client-Instanz und einer Server-Verbindung.2 Ein Host kann jedoch mehrere solcher Client-Instanzen initialisieren, um mit verschiedenen Servern gleichzeitig zu kommunizieren, falls die KI-Anwendung Zugriff auf unterschiedliche Tools oder Datenquellen benötigt.2 Der Client ist verantwortlich für die Aushandlung der Fähigkeiten (Capability Negotiation) mit dem Server und orchestriert den Nachrichtenfluss zwischen sich und dem Server.2 Ein wichtiger Aspekt ist die Wahrung von Sicherheitsgrenzen: Ein Client sollte nicht in der Lage sein, auf Ressourcen zuzugreifen oder Informationen einzusehen, die einem anderen Client (und somit einer anderen Server-Verbindung) zugeordnet sind.2 Er fungiert somit als dedizierter und isolierter Vermittler zwischen dem Host und den externen Ressourcen, die über einen bestimmten MCP-Server bereitgestellt werden.4 Die 1:1-Beziehung zwischen Client und Server vereinfacht das Design dieser beiden Komponenten erheblich, da sie sich jeweils nur auf eine einzige, klar definierte Kommunikationsbeziehung konzentrieren müssen. Dies verlagert jedoch die Komplexität der Orchestrierung mehrerer solcher Beziehungen in den Host, der Mechanismen für die Entdeckung, Initialisierung und Koordination der verschiedenen Clients implementieren muss.
    
- Server (Dienst):
    
    Der MCP-Server ist ein externes Programm oder ein Dienst, der Funktionalitäten in Form von Tools, Daten als Ressourcen und vordefinierte Interaktionsmuster als Prompts über eine standardisierte API bereitstellt.2 Ein Server kann entweder als lokaler Prozess auf derselben Maschine wie der Host/Client laufen oder als ein entfernter Dienst implementiert sein. Er kapselt typischerweise den Zugriff auf spezifische Datenquellen (z.B. Datenbanken, Dateisysteme), externe APIs (z.B. CRM-Systeme, Git-Repositories) oder andere Dienstprogramme.2 Der Server agiert als Brücke oder API zwischen der abstrakten MCP-Welt und der konkreten Funktionalität eines externen Systems.3 Dabei ist es unerlässlich, dass der Server die vom Host durchgesetzten Sicherheitsbeschränkungen und Benutzerberechtigungen strikt einhält.2
    

Die folgende Tabelle fasst die Rollen und Verantwortlichkeiten der MCP-Komponenten zusammen:

**Tabelle 1: MCP-Rollen und Verantwortlichkeiten**

|   |   |   |   |
|---|---|---|---|
|**Rolle**|**Hauptverantwortlichkeiten**|**Schlüsselfunktionen/Interaktionen**|**Wichtige Sicherheitsaspekte**|
|**Host**|Benutzerinteraktion, Koordination von Clients, Verwaltung von Lebenszyklus- und Sicherheitsrichtlinien, KI-Integration|Startet Clients, führt Kontext zusammen, zeigt UI für Einwilligungen an, leitet Anfragen an Clients weiter|**MUSS** Benutzereinwilligung für Datenzugriff/Tool-Nutzung einholen 1, Berechtigungsmanagement, Durchsetzung von Datenschutzrichtlinien, Schutz vor unautorisiertem Client-Zugriff|
|**Client**|Verwaltung der Verbindung zu einem spezifischen Server, Nachrichtenorchestrierung, Capability Negotiation|Stellt Verbindung zu einem Server her (1:1), handelt Fähigkeiten aus, sendet Anfragen an Server, empfängt Antworten, wahrt Sicherheitsgrenzen|Stellt sicher, dass Ressourcen nicht zwischen Clients geteilt werden 2, sichere Kommunikation mit dem Server (Transportverschlüsselung)|
|**Server**|Bereitstellung von Tools, Ressourcen und Prompts, Kapselung externer Systeme|Definiert und exponiert Fähigkeiten, verarbeitet Client-Anfragen, greift auf Backend-Systeme zu, liefert Ergebnisse/Daten zurück|**MUSS** vom Host durchgesetzte Sicherheitsbeschränkungen/Benutzerberechtigungen einhalten 2, sichere Anbindung an Backend-Systeme, Schutz der exponierten Daten und Funktionen|

Diese klare Abgrenzung der Rollen ist fundamental. Entwickler müssen die spezifische Rolle ihrer Komponente genau verstehen und die definierten Schnittstellen und Verantwortlichkeiten respektieren. Insbesondere Host-Entwickler tragen eine große Verantwortung für die korrekte Implementierung der Sicherheits- und Einwilligungsmechanismen, während Server-Entwickler sich darauf verlassen können müssen, dass der Host diese korrekt handhabt.

- **2.2. Interaktionsfluss zwischen den Komponenten**

Ein typischer Interaktionsfluss im MCP-Modell verdeutlicht das Zusammenspiel der Komponenten:

1. **KI-Anfrage:** Eine KI-Anwendung (oder ein Benutzer über den Host) initiiert eine Anfrage, beispielsweise um freie Zeitfenster im Kalender eines Benutzers abzurufen oder eine Zusammenfassung eines Dokuments zu erstellen.2 Diese Anfrage wird im Host verarbeitet.
2. **Weiterleitung an den Client:** Der Host identifiziert den zuständigen Client, der mit dem MCP-Server verbunden ist, welcher die benötigte Funktionalität (z.B. Kalenderzugriff) bereitstellt. Die Anfrage wird an diesen Client übergeben.
3. **Client-Server-Kommunikation:** Der Client formatiert die Anfrage gemäß dem MCP-Protokoll (JSON-RPC) und sendet sie an den verbundenen MCP-Server.
4. **Serververarbeitung:** Der MCP-Server empfängt die Anfrage, validiert sie und führt die entsprechende Aktion aus – beispielsweise den Abruf der Kalenderdaten des Benutzers aus einem Backend-Kalendersystem.2
5. **Antwort an den Client:** Der Server sendet das Ergebnis (z.B. die Liste der freien Zeitfenster) als MCP-Antwort zurück an den Client.2
6. **Weiterleitung an den Host/KI:** Der Client empfängt die Antwort und leitet die relevanten Daten an den Host oder direkt an die KI-Logik innerhalb des Hosts weiter.
7. **KI-Output/Aktion:** Die KI verarbeitet die erhaltenen Daten und erstellt eine passende Antwort für den Benutzer oder führt eine weiterführende Aktion aus, wie beispielsweise das automatische Planen eines Termins.2

Dieser exemplarische Workflow unterstreicht die zentrale Betonung von Benutzerkontrolle, Datenschutz, Sicherheit bei der Tool-Ausführung und Kontrollen für das LLM-Sampling. Diese Aspekte werden als grundlegende Pfeiler für die Entwicklung vertrauenswürdiger und praxistauglicher KI-Lösungen im Rahmen des MCP angesehen.2

**3. MCP Kernfunktionalitäten für Entwickler**

MCP-Server bieten Clients drei Hauptkategorien von Fähigkeiten (Capabilities) an: Tools, Resources und Prompts. Zusätzlich können Clients Servern die Fähigkeit zum Sampling anbieten. Diese Unterscheidung ist nicht nur terminologisch, sondern fundamental für das Design von MCP-Interaktionen, da sie verschiedene Kontroll- und Verantwortlichkeitsbereiche widerspiegelt: Das LLM entscheidet über die Nutzung von Tools, die Anwendung (Host) über den bereitzustellenden Ressourcenkontext und der Benutzer über die Auswahl von Prompts. Diese Trennung ermöglicht es Entwicklern, feingranulare Kontrollen darüber zu implementieren, wie und wann ein LLM auf externe Systeme zugreifen oder Aktionen ausführen darf.

- **3.1. Tools (Modellgesteuert)**

**Definition:** Tools sind im Wesentlichen Funktionen, die von Large Language Models (LLMs) aufgerufen werden können, um spezifische Aktionen in externen Systemen auszuführen.3 Man kann dies als eine standardisierte Form des "Function Calling" betrachten, wie es auch in anderen LLM-Frameworks bekannt ist.3 Die Entscheidung, wann und wie ein Tool basierend auf einer Benutzeranfrage oder einem internen Ziel des LLMs verwendet wird, liegt primär beim Modell selbst.

**Anwendungsfälle:** Die Bandbreite reicht von einfachen Aktionen wie dem Abruf aktueller Wetterdaten über eine API 3 bis hin zu komplexeren Operationen. Beispiele hierfür sind das Senden von E-Mails, das Erstellen von Kalendereinträgen, das Ausführen von Code-Snippets oder die Interaktion mit Diensten wie GitHub, um beispielsweise Issues zu erstellen, Code in Repositories zu suchen oder Pull Requests zu bearbeiten.8

Implementierungsaspekte:

MCP-Server sind dafür verantwortlich, die verfügbaren Tools zu definieren. Dies beinhaltet den Namen des Tools, eine Beschreibung seiner Funktion und ein Schema für die erwarteten Parameter [16 (Tool struct in mcpr)]. Diese Informationen werden dem Client während der Initialisierungsphase mitgeteilt.

Ein kritischer Aspekt bei der Implementierung ist die Sicherheit: Der Host MUSS die explizite Zustimmung des Benutzers einholen, bevor ein vom LLM initiiertes Tool tatsächlich aufgerufen wird.1 Dies wird oft durch ein UI-Element realisiert, das den Benutzer über die geplante Aktion informiert und eine Bestätigung erfordert.4

Weiterhin ist zu beachten, dass Beschreibungen des Tool-Verhaltens und eventuelle Annotationen, die vom Server bereitgestellt werden, als potenziell nicht vertrauenswürdig eingestuft werden sollten, es sei denn, der Server selbst gilt als vertrauenswürdig.1 Dies unterstreicht die Notwendigkeit für Hosts, Mechanismen zur Überprüfung oder Kennzeichnung von Servern zu implementieren.

- **3.2. Resources (Anwendungsgesteuert)**

**Definition:** Resources repräsentieren Datenquellen, auf die LLMs zugreifen können, um Informationen abzurufen, die für die Bearbeitung einer Anfrage oder die Anreicherung des Kontexts benötigt werden.3 Sie verhalten sich ähnlich wie GET-Endpunkte in einer REST-API, indem sie primär Daten liefern, ohne dabei signifikante serverseitige Berechnungen durchzuführen oder Seiteneffekte (wie Datenmodifikationen) auszulösen.3 Die abgerufenen Ressourcen werden Teil des Kontexts, der dem LLM für seine nächste Inferenzrunde zur Verfügung gestellt wird.3

**Anwendungsfälle:** Typische Beispiele sind der Abruf von Benutzerprofilinformationen, Produktdetails aus einem Katalog, Inhalten aus Dokumenten oder Datenbanken, aktuellen Kalenderdaten 2 oder der Zugriff auf Dateien im lokalen Dateisystem des Benutzers (mit dessen expliziter Zustimmung).4

Implementierungsaspekte:

Server definieren die Struktur und Verfügbarkeit der Ressourcen, die sie anbieten. Wie bei Tools muss der Host auch hier die explizite Zustimmung des Benutzers einholen, bevor Benutzerdaten (die als Ressourcen von einem Server abgerufen oder an einen Server gesendet werden sollen) transferiert werden.1 Der Host ist zudem verpflichtet, Benutzerdaten, die als Ressourcen gehandhabt werden, mit geeigneten Zugriffskontrollen zu schützen, um unautorisierten Zugriff zu verhindern.1

- **3.3. Prompts (Benutzergesteuert)**

**Definition:** Prompts im MCP-Kontext sind vordefinierte Vorlagen oder Schablonen, die dazu dienen, die Interaktion mit Tools oder Ressourcen auf eine optimale und standardisierte Weise zu gestalten.3 Im Gegensatz zu Tools, deren Nutzung vom LLM initiiert wird, werden Prompts typischerweise vom Benutzer (über die Host-Anwendung) ausgewählt, bevor eine Inferenz oder eine spezifische Aktion gestartet wird.3

**Nutzungsszenarien:** Prompts können für standardisierte Abfragen (z.B. "Fasse mir die Änderungen im Pull Request X zusammen"), geführte Workflows (z.B. ein mehrstufiger Prozess zur Fehlerbehebung) oder häufig verwendete Befehlssätze dienen. In Benutzeroberflächen können sie als dedizierte Schaltflächen oder Menüpunkte für benutzerdefinierte Aktionen erscheinen.4

Gestaltung:

Server können parametrisierbare Prompts anbieten, d.h. Vorlagen, die Platzhalter für benutzerspezifische Eingaben enthalten.7 Ein wichtiger Aspekt des Protokolldesigns ist, dass die Sichtbarkeit des Servers auf den Inhalt von Prompts, insbesondere während des LLM-Samplings (siehe unten), absichtlich begrenzt ist.1 Dies dient dem Schutz der Benutzerprivatsphäre und der Wahrung der Benutzerkontrolle über die an das LLM gesendeten Informationen.

- **3.4. Sampling (Server-initiierte Interaktionen)**

**Konzept:** "Sampling" ist eine fortgeschrittene Fähigkeit, die Clients den Servern anbieten können. Sie ermöglicht es dem _Server_, agentische Verhaltensweisen und rekursive LLM-Interaktionen über den Client zu initiieren.1 Dies stellt eine Abkehr vom typischen reaktiven Modell dar, bei dem der Client/Host Anfragen an den Server sendet. Beim Sampling kann der Server proaktiv das LLM (vermittelt durch den Client und Host) auffordern, basierend auf serverseitiger Logik, externen Ereignissen oder dem Ergebnis vorheriger Interaktionen zu "denken" oder zu handeln.

**Kontrollmechanismen:** Aufgrund der potenziellen Mächtigkeit und der damit verbundenen Risiken dieser Funktion legt die MCP-Spezifikation größten Wert auf strenge Benutzerkontrolle:

- Benutzer **MÜSSEN** allen LLM-Sampling-Anfragen, die von einem Server initiiert werden, explizit zustimmen.1
- Benutzer **SOLLTEN** die volle Kontrolle darüber haben, ob Sampling überhaupt stattfinden darf, welchen genauen Prompt-Inhalt das LLM erhält und welche Ergebnisse oder Zwischenschritte der Server einsehen kann.1

Diese Funktion ist zwar mächtig und kann zu intelligenteren, proaktiveren Agenten führen, die beispielsweise auf sich ändernde Umgebungsbedingungen reagieren, ohne für jeden Schritt eine direkte Benutzeraufforderung zu benötigen. Jedoch birgt sie auch erhebliche Sicherheitsimplikationen. Entwickler, die die Sampling-Funktion nutzen – sowohl auf Client- als auch auf Serverseite – müssen höchste Priorität auf transparente Benutzeraufklärung und robuste, unmissverständliche Einwilligungsmechanismen legen. Missbrauch oder unkontrolliertes Sampling könnten zu unerwünschtem Verhalten, exzessiver Ressourcennutzung oder Datenschutzverletzungen führen. Es ist die vielleicht wirkungsvollste, aber auch die verantwortungsvollste Funktion im MCP-Framework.

Die folgende Tabelle gibt eine vergleichende Übersicht über die Kernfunktionalitäten des MCP:

**Tabelle 2: Übersicht der MCP-Kernfunktionalitäten**

|   |   |   |   |   |
|---|---|---|---|---|
|**Funktionalität**|**Primäre Steuerungsebene**|**Kurzbeschreibung und Zweck**|**Typische Anwendungsbeispiele**|**Wichtige Sicherheitsüberlegung**|
|**Tool**|Modell (LLM)|Ausführbare Funktion für spezifische Aktionen; LLM entscheidet über Nutzung.|API-Aufrufe (Wetter, GitHub), E-Mail senden, Kalendereintrag erstellen.3|Host **MUSS** Benutzerzustimmung vor Aufruf einholen.1 Tool-Beschreibungen potenziell nicht vertrauenswürdig.1|
|**Resource**|Anwendung/Host|Datenquelle für Informationsabruf; liefert Kontext ohne Seiteneffekte.|Benutzerprofile, Produktdaten, Dokumentinhalte, Dateisystemzugriff.2|Host **MUSS** Benutzerzustimmung für Datenweitergabe/-abruf einholen.1 Datenschutz und Zugriffskontrollen sind kritisch.|
|**Prompt**|Benutzer|Vordefinierte Vorlage zur optimalen Nutzung von Tools/Ressourcen; vom Benutzer ausgewählt.|Standardisierte Abfragen, geführte Workflows, häufige Befehle.3|Serverseitige Sichtbarkeit auf Prompt-Inhalte ist begrenzt, um Benutzerkontrolle zu wahren.1|
|**Sampling**|Server / Benutzer|Server-initiierte agentische LLM-Interaktion; erfordert explizite Client-Fähigkeit.|Proaktive Agenten, rekursive LLM-Aufgaben, Reaktion auf externe Server-Events.1|Benutzer **MUSS** explizit zustimmen und Kontrolle über Prompt/Ergebnisse behalten.1 Hohes Missbrauchspotenzial.|

Entwickler von MCP-Servern müssen sorgfältig abwägen, welche Funktionalitäten sie als Tool, Ressource oder Prompt exponieren. Diese Entscheidung hat direkte Auswirkungen auf die Steuerungsmöglichkeiten, die Sicherheitsparadigmen und letztendlich die Benutzererfahrung, da sie bestimmt, wer die primäre Kontrolle über die jeweilige Interaktion ausübt.

**4. MCP Kommunikationsprotokoll: JSON-RPC 2.0**

Für die Kommunikation zwischen den Komponenten (Host, Client und Server) setzt das Model Context Protocol auf JSON-RPC 2.0.1 JSON-RPC ist ein leichtgewichtiges Remote Procedure Call (RPC) Protokoll, das sich durch seine Einfachheit und die Verwendung des weit verbreiteten JSON-Formats auszeichnet.

- **4.1. Grundlagen von JSON-RPC 2.0 im MCP-Kontext**

JSON-RPC 2.0 wurde als Basis für MCP gewählt, da es eine klare Struktur für Anfragen und Antworten bietet und gleichzeitig transportagnostisch ist, obwohl MCP spezifische Transportmechanismen vorschreibt, wie später erläutert wird.9 Die Verwendung von JSON macht die Nachrichten für Entwickler leicht lesbar und einfach zu parsen.

Die Kernkomponenten einer JSON-RPC 2.0 Nachricht sind:

- **Request-Objekt:** Eine Anfrage an den Server besteht aus den folgenden Feldern 7:
    - `jsonrpc`: Eine Zeichenkette, die die Version des JSON-RPC-Protokolls angibt, hier immer `"2.0"`.
    - `method`: Eine Zeichenkette, die den Namen der aufzurufenden Methode (Funktion) auf dem Server enthält.
    - `params`: Ein strukturiertes Objekt oder ein Array, das die Parameter für die aufzurufende Methode enthält. MCP verwendet typischerweise benannte Parameter (Objektform).
    - `id`: Ein eindeutiger Identifikator (String oder Integer, darf nicht Null sein), der vom Client generiert wird. Dieses Feld ist notwendig, um Antworten den entsprechenden Anfragen zuordnen zu können. Fehlt die `id`, handelt es sich um eine Notification.
- **Response-Objekt:** Eine Antwort vom Server auf eine Anfrage enthält 7:
    - `jsonrpc`: Ebenfalls `"2.0"`.
    - `id`: Derselbe Wert wie in der korrespondierenden Anfrage.
    - Entweder `result`: Dieses Feld ist bei einer erfolgreichen Ausführung der Methode vorhanden und enthält das Ergebnis der Operation. Der Datentyp des Ergebnisses ist methodenspezifisch.
    - Oder `error`: Dieses Feld ist vorhanden, wenn während der Verarbeitung der Anfrage ein Fehler aufgetreten ist.
- **Notification:** Eine Notification ist eine spezielle Form einer Anfrage, die keine `id` enthält. Da keine `id` vorhanden ist, sendet der Server keine Antwort auf eine Notification. Notifications eignen sich für unidirektionale Benachrichtigungen, bei denen der Client keine Bestätigung oder Ergebnis erwartet.
- **Error-Objekt:** Im Fehlerfall enthält das `error`-Feld ein Objekt mit den folgenden Feldern 7:
    - `code`: Ein numerischer Wert, der den Fehlertyp angibt (Standard-JSON-RPC-Fehlercodes oder anwendungsspezifische Codes).
    - `message`: Eine kurze, menschenlesbare Beschreibung des Fehlers.
    - `data` (optional): Ein Feld, das zusätzliche, anwendungsspezifische Fehlerinformationen enthalten kann.

Die folgende Tabelle fasst die JSON-RPC 2.0 Nachrichtenkomponenten im Kontext von MCP zusammen:

**Tabelle 3: JSON-RPC 2.0 Nachrichtenkomponenten im MCP**

|   |   |   |   |
|---|---|---|---|
|**Komponente**|**Datentyp (Beispiel)**|**Beschreibung im MCP-Kontext**|**Erforderlichkeit (Nachrichtentyp)**|
|`jsonrpc`|String (`"2.0"`)|Version des JSON-RPC Protokolls.|Request, Response, Notification|
|`id`|String, Integer, Null|Eindeutiger Identifikator zur Korrelation von Request und Response. `Null` ist nicht erlaubt.|Request (wenn Antwort erwartet), Response. Fehlt bei Notification.|
|`method`|String|Name der auf dem Server auszuführenden MCP-spezifischen Methode (z.B. `initialize`).|Request, Notification|
|`params`|Object / Array|Parameter für die aufzurufende Methode. MCP verwendet typischerweise benannte Parameter (Object).|Request (optional), Notification (optional)|
|`result`|Object / Array / Scalar|Ergebnis der erfolgreichen Methodenausführung.|Response (bei Erfolg)|
|`error`|Object|Strukturiertes Objekt, das Fehlerdetails enthält.|Response (bei Fehlschlag)|
|`error.code`|Integer|Numerischer Fehlercode.|Innerhalb des `error`-Objekts|
|`error.message`|String|Menschenlesbare Fehlerbeschreibung.|Innerhalb des `error`-Objekts|
|`error.data`|Any|Zusätzliche, anwendungsspezifische Fehlerinformationen.|Innerhalb des `error`-Objekts (optional)|

Ein klares Verständnis dieser Nachrichtenstruktur ist die Grundlage für die Implementierung der MCP-Kommunikation und unerlässlich für Entwickler, die MCP-Nachrichten direkt verarbeiten oder Debugging auf Protokollebene durchführen müssen.

- **4.2. Zustandsbehaftete Verbindungen (Stateful Connections)**

Ein wesentliches Merkmal des MCP ist, dass die etablierten Verbindungen zwischen Client und Server zustandsbehaftet ("stateful") sind.1 Dies bedeutet, dass der Server Informationen über den Zustand jedes verbundenen Clients über mehrere Anfragen und Antworten hinweg speichert und verwaltet.1 Dieser Zustand kann beispielsweise die während der Initialisierung ausgehandelten Fähigkeiten, Informationen über laufende Operationen oder sitzungsspezifische Konfigurationen umfassen.

Die Zustandsbehaftung von MCP-Verbindungen hat signifikante Implikationen für Entwickler:

- **Serverseitiges Zustandsmanagement:** Server müssen Mechanismen implementieren, um den individuellen Zustand für jede aktive Client-Sitzung zu verwalten.10 Dies erfordert sorgfältiges Design, um Ressourcenkonflikte zu vermeiden und die Integrität der Sitzungsdaten sicherzustellen.
- **Verbindungslebenszyklus:** Die Verbindung durchläuft einen definierten Lebenszyklus, der mindestens eine Initialisierungsphase, eine Phase des aktiven Nachrichtenaustauschs und eine Terminierungsphase umfasst.1 Jede dieser Phasen muss von Client und Server korrekt gehandhabt werden.
- **Unterschied zu zustandslosen Protokollen:** Dies unterscheidet MCP grundlegend von typischen zustandslosen Protokollen wie vielen REST-APIs, bei denen jede Anfrage unabhängig von vorherigen Anfragen behandelt wird. Die Zustandsbehaftung ermöglicht zwar kontextreichere und effizientere Interaktionen (da nicht bei jeder Anfrage der gesamte Kontext neu übertragen werden muss), sie stellt aber auch höhere Anforderungen an die Fehlerbehandlung und die Mechanismen zur Wiederherstellung nach Verbindungsabbrüchen oder Serverausfällen.10
- **Robustheit und Skalierbarkeit:** Die Zustandsbehaftung kann das Serverdesign komplexer machen.11 Der Server muss den Zustand für potenziell viele Clients verwalten, was Speicher- und Verarbeitungsressourcen beansprucht. Fehlerbehandlung und Wiederherstellung nach Ausfällen sind kritischer, da der Sitzungszustand möglicherweise wiederhergestellt oder zumindest sauber beendet werden muss, um Ressourcenlecks oder inkonsistente Zustände zu vermeiden.10 Auch die Skalierbarkeit kann im Vergleich zu zustandslosen Architekturen schwieriger zu erreichen sein, da Anfragen eines bestimmten Clients möglicherweise immer zum selben Server (oder zu einem Server mit Zugriff auf denselben verteilten Zustand) geleitet werden müssen.

Entwickler von MCP-Servern müssen daher Strategien für ein robustes Session-Management, eine umfassende Fehlerbehandlung (einschließlich Timeouts und gegebenenfalls Wiederverbindungslogik auf Client-Seite), eine zuverlässige Ressourcenbereinigung bei Verbindungsabbrüchen und potenziell für die Verteilung von Sitzungszuständen in skalierten Umgebungen entwickeln. Die vordergründige Einfachheit von JSON-RPC sollte nicht über diese systemischen Herausforderungen hinwegtäuschen, die mit dem zustandsbehafteten Charakter von MCP einhergehen.

- **4.3. Transportmechanismen**

MCP spezifiziert, wie die JSON-RPC-Nachrichten zwischen Client und Server transportiert werden. Derzeit sind zwei primäre Transportmechanismen definiert 3, deren Wahl direkte Auswirkungen auf Deployment-Szenarien und die Implementierungskomplexität hat.

- 4.3.1. Standard I/O (stdio)
    
    Dieser Mechanismus wird typischerweise verwendet, wenn sowohl der MCP-Client als auch der MCP-Server auf derselben physischen oder virtuellen Maschine laufen.3 In diesem Szenario startet der Client (bzw. der Host, in dem der Client läuft) den Serverprozess oft als einen Kindprozess. Die Kommunikation erfolgt dann über die Standard-Datenströme des Kindprozesses: Der Client sendet JSON-RPC-Anfragen an den Standard-Input (stdin) des Servers, und der Server sendet seine JSON-RPC-Antworten über seinen Standard-Output (stdout) zurück an den Client.7 Der Standard-Error-Stream (stderr) des Servers kann für Log-Meldungen oder unspezifische Fehlerausgaben genutzt werden, die nicht Teil des strukturierten JSON-RPC-Fehlerprotokolls sind.7
    
    stdio ist ein einfacher und effektiver Transport für lokale Integrationen, beispielsweise wenn eine Desktop-Anwendung (Host) auf lokale Tools zugreifen muss, die als MCP-Server implementiert sind (z.B. Zugriff auf das lokale Dateisystem oder Ausführung lokaler Skripte).3
    
    Bei der Implementierung, beispielsweise in Rust, ermöglichen Funktionen wie Stdio::piped() aus dem std::process-Modul die Einrichtung der notwendigen Pipes für die Kommunikation mit Kindprozessen.12 Es ist jedoch Vorsicht geboten: Wenn große Datenmengen über stdin geschrieben werden, ohne gleichzeitig von stdout (und stderr) zu lesen, kann es zu Deadlocks kommen, da die Pipe-Puffer volllaufen können.12 Die Größe dieser Puffer variiert je nach Betriebssystem.
    
- 4.3.2. HTTP mit Server-Sent Events (SSE)
    
    Für Szenarien, in denen Client und Server über ein Netzwerk kommunizieren, insbesondere wenn der Server die Fähigkeit benötigt, Nachrichten oder Ereignisse aktiv an den Client zu pushen, wird HTTP in Kombination mit Server-Sent Events (SSE) verwendet.3
    
    Der Kommunikationsaufbau ist hier mehrstufig: Der Client stellt zunächst eine HTTP-Verbindung zum Server her und initiiert einen SSE-Stream. Über diesen SSE-Stream kann der Server dann asynchron Nachrichten (Events) an den Client senden. Diese Verbindung bleibt persistent.3 Für Anfragen vom Client an den Server sieht der Prozess laut 7 wie folgt aus: Der Client öffnet eine SSE-Verbindung zum Server und empfängt als eines der ersten Events ein spezielles endpoint-Event. Dieses Event enthält eine URI. An diese spezifische URI sendet der Client dann seine JSON-RPC-Anfragen mittels HTTP POST. Der Server verarbeitet diese POST-Anfragen und sendet die JSON-RPC-Antworten wiederum über die bereits etablierte, persistente SSE-Verbindung zurück an den Client.7
    
    Dieser Mechanismus ist komplexer als stdio, ermöglicht aber die notwendige Flexibilität für verteilte Architekturen, Cloud-basierte MCP-Server oder die Anbindung an Software-as-a-Service (SaaS)-Produkte. Die Implementierung erfordert die Handhabung von HTTP-Verbindungen, das Management des SSE-Event-Streams und typischerweise auch robustere Authentifizierungs- und Sicherheitsmaßnahmen (z.B. die Verwendung von HTTPS). Für Rust-Entwickler bieten Bibliotheken wie actix-web-lab Unterstützung für die Implementierung von SSE-Endpunkten.14 Es ist zu beachten, dass SSE primär für die unidirektionale Kommunikation vom Server zum Client für Events gedacht ist; die Anfragen vom Client zum Server erfolgen über separate HTTP POST-Requests auf eine dynamisch während des SSE-Handshakes mitgeteilte URL.7
    

Die folgende Tabelle vergleicht die beiden Haupttransportmechanismen:

**Tabelle 4: Vergleich der MCP-Transportmechanismen**

|   |   |   |   |   |
|---|---|---|---|---|
|**Mechanismus**|**Typische Anwendungsfälle**|**Vorteile**|**Nachteile/Herausforderungen**|**Wichtige Implementierungsaspekte**|
|**stdio**|Lokale Integrationen (Client/Server auf derselben Maschine)|Einfach zu implementieren, geringer Overhead, effektiv für lokale Tools|Nicht für Netzwerkkommunikation geeignet, potenzielle Deadlocks bei unsachgemäßer Pufferbehandlung 12|Prozessmanagement (Starten/Stoppen des Servers), korrekte Handhabung von `stdin`/`stdout`/`stderr`, Vermeidung von Puffer-Deadlocks, Fehlerbehandlung bei Prozessende|
|**HTTP/SSE**|Verteilte Architekturen, Remote-Server, Web-Anwendungen|Ermöglicht Netzwerkkommunikation, Server-Push-Fähigkeit (via SSE)|Komplexer in der Implementierung, erfordert HTTP-Server/-Client-Logik, Management persistenter Verbindungen, Sicherheit (HTTPS)|HTTP-Request/Response-Handling, SSE-Event-Stream-Management, URI-Management für POST-Requests 7, Authentifizierung, Fehlerbehandlung bei Netzwerkproblemen|

Entwickler müssen den Transportmechanismus sorgfältig basierend auf dem geplanten Einsatzszenario ihres MCP-Servers oder -Clients auswählen. Die Spezifikation unterstützt beide Optionen, aber die Anforderungen an Entwicklung, Deployment und Betrieb unterscheiden sich erheblich.

**5. MCP Protokollspezifikation: Methoden und Nachrichtenfluss**

Dieser Abschnitt detailliert die spezifischen JSON-RPC-Methoden, die das Model Context Protocol definiert, sowie den typischen Nachrichtenfluss für Kerninteraktionen. Es ist essenziell zu verstehen, dass die exakten Schemata für Anfragen und Antworten in der offiziellen `schema.ts`-Datei des MCP-Projekts definiert sind.1 Implementierungen in Sprachen wie Rust, beispielsweise durch die `rust-mcp-schema`-Bibliothek 15, bieten typisierte Strukturen, die auf diesen Schemata basieren und die Entwicklung erleichtern. Die hier beschriebenen Methodennamen und Parameter sind repräsentativ und sollten stets mit der offiziellen Spezifikation abgeglichen werden.

- **5.1. Initialisierungsphase: `initialize` Methode**

**Zweck:** Die `initialize`-Methode ist der erste und grundlegende Schritt jeder MCP-Kommunikation nach dem Aufbau der Transportverbindung. Der Client initiiert diesen Aufruf, um eine Sitzung mit dem Server zu etablieren. Während dieses Austauschs werden Protokollversionen abgeglichen und, entscheidend, die Fähigkeiten (Capabilities) beider Seiten ausgetauscht.3

**Nachrichtenfluss:**

1. Client sendet `initialize` Request an den Server.
2. Server antwortet mit `initialize` Response (oft als `InitializeResult` in SDKs bezeichnet 15).

**Client `initialize` Request Parameter (Beispiel basierend auf 7):**

- `jsonrpc`: `"2.0"`
- `id`: Eine eindeutige Request-ID (z.B. `"4711"` 7).
- `method`: `"initialize"`
- `params`: Ein Objekt, das typischerweise folgende Felder enthält:
    - `protocolVersion` (String): Die Version des MCP-Protokolls, die der Client unterstützt (z.B. `"2024-11-05"` 7).
    - `capabilities` (Object): Ein Objekt, das die Fähigkeiten beschreibt, die der Client dem Server anbietet. Ein wichtiges Beispiel ist die `sampling`-Fähigkeit, die es dem Server erlaubt, LLM-Interaktionen über den Client zu initiieren.7
    - `clientInfo` (Object): Informationen über die Client-Anwendung, wie `name` (z.B. `"SomeClient"`) und `version` (z.B. `"1.2.3"`).7

**Server `initialize` Response (`InitializeResult`) Parameter (Beispiel basierend auf 15):**

- `jsonrpc`: `"2.0"`
- `id`: Die ID aus dem korrespondierenden Request.
- `result`: Ein Objekt, das typischerweise folgende Felder enthält:
    - `protocolVersion` (String): Die vom Server gewählte und unterstützte Protokollversion. Diese sollte mit der vom Client angebotenen Version kompatibel sein.
    - `serverInfo` (Object): Informationen über den Server, wie `name`, `version` und möglicherweise weitere Metadaten.
    - `capabilities` (Object): Ein Objekt, das die vom Server angebotenen Fähigkeiten detailliert beschreibt. Dies ist ein Kernstück der Antwort und beinhaltet typischerweise Unterobjekte für:
        - `prompts`: Definitionen der verfügbaren Prompts.
        - `resources`: Definitionen der verfügbaren Ressourcen.
        - `tools`: Definitionen der verfügbaren Tools, inklusive ihrer Parameter-Schemata und Beschreibungen.
    - `meta` (Object, optional): Zusätzliche, serverseitige Metadaten.
    - `instructions` (String, optional): Spezifische Anweisungen oder Hinweise vom Server an den Client.

Die `initialize`-Methode ist das Fundament jeder MCP-Interaktion. Sie legt die Spielregeln für die nachfolgende Kommunikation fest. Es geht nicht nur um den Austausch von Versionsinformationen, sondern vor allem um die Deklaration und Aushandlung der gegenseitigen Fähigkeiten. Der Client deklariert, welche serverseitig nutzbaren Funktionen er anbietet (z.B. `sampling`), und der Server legt umfassend dar, welche Tools, Ressourcen und Prompts er zur Verfügung stellt. Ohne eine erfolgreiche Initialisierung und eine klare Übereinkunft über die unterstützten Fähigkeiten können keine weiteren sinnvollen MCP-Operationen stattfinden. Entwickler müssen diese Sequenz daher mit größter Sorgfalt implementieren. Fehler oder Missverständnisse in dieser kritischen Phase führen unweigerlich zu Problemen in der weiteren Kommunikation. Die dynamische Natur der Fähigkeiten bedeutet auch, dass Clients und Server flexibel auf die vom jeweiligen Gegenüber angebotenen und unterstützten Funktionen reagieren müssen.

- **5.2. Aufruf von Tools (z.B. `mcp/tool_call`)**

**Zweck:** Diese Methode wird vom Client aufgerufen, wenn das LLM (oder in manchen Fällen die Host-Anwendung) die Ausführung eines vom Server bereitgestellten Tools anfordert. Der genaue Methodenname (hier als `mcp/tool_call` angenommen) ist der Spezifikation zu entnehmen.

**Nachrichtenfluss:**

1. Client sendet `mcp/tool_call` Request an den Server.
2. Server antwortet mit `mcp/tool_call` Response.

**Request Parameter:**

- `tool_name` (String): Der eindeutige Name des aufzurufenden Tools, wie vom Server in den `capabilities` während der Initialisierung deklariert.
- `params` (Object): Ein Objekt, das die Parameter für das Tool enthält. Die Struktur dieses Objekts muss dem Schema entsprechen, das der Server für dieses spezifische Tool definiert hat.

**Response (`result`):**

- Das Ergebnis der Tool-Ausführung. Die Struktur dieses Ergebnisses ist ebenfalls durch das vom Server definierte Schema für das jeweilige Tool bestimmt.

Rust-Bibliotheken wie `mcpr` abstrahieren diesen JSON-RPC-Nachrichtenaustausch durch Methodenaufrufe wie `client.call_tool("my_tool", &request)` 16, was die Entwicklung vereinfacht.

- **5.3. Zugriff auf Resources (z.B. `mcp/fetch_resource`)**

**Zweck:** Diese Methode dient dem Abruf von Daten aus einer vom Server bereitgestellten Ressource. Sie wird vom Client initiiert, wenn das LLM oder die Host-Anwendung kontextuelle Informationen benötigt. (Methodenname `mcp/fetch_resource` ist hier angenommen).

**Nachrichtenfluss:**

1. Client sendet `mcp/fetch_resource` Request an den Server.
2. Server antwortet mit `mcp/fetch_resource` Response.

**Request Parameter:**

- `resource_id` (String): Der eindeutige Bezeichner der Ressource, wie vom Server in den `capabilities` deklariert.
- `params` (Object, optional): Parameter zur weiteren Spezifizierung der Anfrage, z.B. Filterkriterien oder Paginierungsinformationen, falls die Ressource dies unterstützt.

**Response (`result`):**

- Die angeforderten Ressourcendaten in der vom Server für diese Ressource definierten Struktur.
    
- **5.4. Verwendung von Prompts (z.B. `mcp/execute_prompt`)**
    

**Zweck:** Ermöglicht dem Benutzer (über den Client), einen vom Server vordefinierten Prompt auszuwählen und auszuführen. Die Auflistung der verfügbaren Prompts und ihrer Parameter erfolgt typischerweise basierend auf den Informationen aus der `initialize`-Antwort des Servers.7 (Methodenname `mcp/execute_prompt` ist hier angenommen).

**Nachrichtenfluss:** Variiert je nach Design, aber typischerweise:

1. Client sendet `mcp/execute_prompt` Request an den Server (nachdem der Benutzer einen Prompt ausgewählt hat).
2. Server antwortet mit `mcp/execute_prompt` Response (z.B. das Ergebnis der Prompt-Ausführung oder eine Bestätigung).

**Request Parameter:**

- `prompt_id` (String): Der eindeutige Bezeichner des auszuführenden Prompts.
- `params` (Object, optional): Parameter, die in den Prompt eingesetzt werden sollen, falls dieser parametrisierbar ist.

**Response (`result`):**

- Das Ergebnis der Prompt-Ausführung, dessen Struktur vom spezifischen Prompt abhängt.
    
- **5.5. Durchführung von Sampling-Anfragen (z.B. `mcp/sampling_request`)**
    

**Zweck:** Diese Interaktion wird vom _Server_ initiiert, wenn dieser eine agentische LLM-Interaktion oder eine rekursive LLM-Nutzung durch den Client anstoßen möchte. Dies ist nur möglich, wenn der Client in seiner `initialize`-Anfrage die `sampling`-Fähigkeit angeboten und der Benutzer dem zugestimmt hat. (Methodenname `mcp/sampling_request` ist hier angenommen).

**Nachrichtenfluss:**

1. Server sendet `mcp/sampling_request` Request (oder Notification) an den Client.
2. Client verarbeitet die Anfrage (potenziell nach erneuter Benutzerzustimmung) und kann eine Response an den Server senden.

**Request Parameter (vom Server an Client):**

- `prompt` (String oder strukturiertes Objekt): Der Prompt, den das LLM verarbeiten soll.
- `sampling_params` (Object, optional): Spezifische Parameter für den Sampling-Prozess (z.B. Temperatur, max. Tokens).

**Response (vom Client an Server, falls keine Notification):**

- Das Ergebnis der LLM-Verarbeitung des vom Server initiierten Prompts.
    
- **5.6. Zusätzliche Utilities**
    

MCP definiert auch eine Reihe von Hilfsmethoden und -mechanismen, die für eine robuste Kommunikation unerlässlich sind.1

- **`Ping`:** Eine einfache Methode (Client -> Server Request, Server -> Client Response), um die Lebendigkeit der Verbindung und die Erreichbarkeit des Servers zu überprüfen. Enthält typischerweise keine signifikanten Parameter.
- **`$/cancelRequest` (JSON-RPC Standard):** Eine Notification vom Client an den Server, um eine zuvor gesendete, noch laufende Anfrage abzubrechen. Die Notification enthält die `id` der abzubrechenden Anfrage in ihren Parametern.
- **`$/progress` (JSON-RPC Standard für Progress Notification):** Eine Notification vom Server an den Client, um diesen über den Fortschritt einer langlaufenden Operation zu informieren. Die Notification enthält typischerweise eine `id` (die sich auf die ursprüngliche Anfrage bezieht) und Fortschrittsdetails.
- **Error Reporting:** Erfolgt über das Standard-JSON-RPC-Error-Objekt in Responses, wenn eine Methode nicht erfolgreich ausgeführt werden konnte.
- **Logging:** Kann über `stderr` (im `stdio`-Transportmodus) oder über spezifische, im Protokoll definierte Log-Notifications erfolgen.

Obwohl diese Utilities als "zusätzlich" bezeichnet werden, sollten Entwickler sie als integralen Bestandteil einer professionellen MCP-Implementierung betrachten. Ihre Implementierung verbessert die Stabilität, Reaktionsfähigkeit und Benutzererfahrung erheblich, insbesondere in verteilten oder zeitintensiven Szenarien. Beispielsweise hat der Benutzer ohne `Progress Tracking` keine Rückmeldung über den Status langlaufender Operationen. Ohne `Cancellation` können versehentlich gestartete oder zu lange dauernde Operationen nicht abgebrochen werden, was zu Ressourcenverschwendung oder Frustration führt. `Ping` ist entscheidend für Health Checks und die frühzeitige Erkennung von Verbindungsproblemen. Das Fehlen dieser Utilities kann zu schwer diagnostizierbaren Problemen und einer insgesamt schlechten User Experience führen.

- **5.7. Verbindungslebenszyklus und Zustandsmanagement**

Der Lebenszyklus einer MCP-Verbindung und das damit verbundene Zustandsmanagement sind kritische Aspekte:

1. **Aufbau:**
    - Herstellen der physischen Transportverbindung (`stdio` oder `HTTP/SSE`).
    - Durchführung der `initialize`-Sequenz (Client sendet Request, Server antwortet). Bei Erfolg ist die MCP-Sitzung etabliert.
2. **Betrieb:**
    - Austausch von anwendungsspezifischen MCP-Nachrichten: Tool-Aufrufe, Ressourcenanfragen, Prompt-Ausführungen, Sampling-Nachrichten.
    - Austausch von Utility-Nachrichten: `Ping`, `$/cancelRequest`, `$/progress`.
3. **Abbau:**
    - Explizit: Durch eine `shutdown`-Methode (z.B. `client.shutdown()` in 16), die der Client an den Server sendet, um die Sitzung ordnungsgemäß zu beenden. Der Server sollte daraufhin alle mit dieser Sitzung verbundenen Ressourcen freigeben.
    - Implizit: Durch das Schließen der zugrundeliegenden Transportverbindung (z.B. Schließen der Pipes bei `stdio` oder Trennen der HTTP-Verbindung bei SSE). Auch hier **SOLLTEN** Server versuchen, Ressourcen aufzuräumen.
4. **Zustandsmanagement:**
    - Server **MÜSSEN** den Zustand für jede aktive Client-Sitzung verwalten. Dazu gehören mindestens die während der `initialize`-Phase ausgehandelten Fähigkeiten, Informationen über aktuell laufende Anfragen (um z.B. Duplikate oder Konflikte zu erkennen) und sitzungsspezifische Daten.
    - Clients **MÜSSEN** den Verbindungsstatus zum Server verwalten und in der Lage sein, auf Verbindungsabbrüche oder Fehler zu reagieren (z.B. durch Wiederverbindungsversuche oder Information des Benutzers).

Die folgende Tabelle gibt einen exemplarischen Überblick über wichtige MCP-Methoden. Die genauen Namen und Parameter sind der offiziellen Spezifikation zu entnehmen.

**Tabelle 5: Wichtige MCP-Methoden und ihre Parameter (Beispiele)**

|   |   |   |   |   |
|---|---|---|---|---|
|**Methode (angenommen/Standard)**|**Richtung**|**Wichtige Parameter (Request)**|**Erwartete Antwort/Struktur (Response/Result)**|**Zweck im MCP**|
|`initialize`|Client -> Server|`protocolVersion`, `clientInfo`, `capabilities` (client-seitig, z.B. `sampling`) 7|`protocolVersion`, `serverInfo`, `capabilities` (serverseitig: `tools`, `resources`, `prompts`) 15|Aufbau der Sitzung, Aushandlung von Protokollversion und Fähigkeiten.|
|`mcp/tool_call` (angenommen)|Client -> Server|`tool_name`, `params` (toolspezifisch)|Ergebnis der Tool-Ausführung (toolspezifisch)|Ausführung einer vom Server bereitgestellten Funktion (Tool).|
|`mcp/fetch_resource` (angenommen)|Client -> Server|`resource_id`, `params` (ressourcenspezifisch, optional)|Angefragte Ressourcendaten|Abruf von Daten aus einer vom Server bereitgestellten Quelle (Resource).|
|`mcp/execute_prompt` (angenommen)|Client -> Server|`prompt_id`, `params` (promptspezifisch, optional)|Ergebnis der Prompt-Ausführung|Ausführung eines vom Benutzer ausgewählten, vordefinierten Prompts.|
|`mcp/sampling_request` (angenommen)|Server -> Client|`prompt`, `sampling_params` (optional)|Ergebnis der LLM-Verarbeitung (optional, falls keine Notification)|Server-initiierte LLM-Interaktion über den Client.|
|`Ping`|Client <-> Server|Keine oder minimale Parameter|Bestätigung (z.B. leeres Objekt oder Pong-Nachricht)|Überprüfung der Verbindungsintegrität und Serververfügbarkeit.|
|`$/cancelRequest`|Client -> Server|`id` der abzubrechenden Anfrage|Keine (Notification)|Abbruch einer zuvor gesendeten, noch laufenden Anfrage.|
|`$/progress`|Server -> Client|`id` der ursprünglichen Anfrage, Fortschrittsdetails (z.B. Prozent, Statusnachricht)|Keine (Notification)|Information des Clients über den Fortschritt einer langlaufenden serverseitigen Operation.|

Diese Tabelle dient als Referenz für Entwickler, um die grundlegenden Interaktionsmuster und die damit verbundenen Datenstrukturen im MCP zu verstehen, bevor sie sich in die Details der offiziellen Schemadateien vertiefen.

**6. Entwicklungsrichtlinien für MCP-Implementierungen**

Die erfolgreiche Implementierung von MCP-Komponenten erfordert die Beachtung spezifischer Designprinzipien und Best Practices. Diese Richtlinien zielen darauf ab, robuste, sichere, wartbare und interoperable MCP-Server und -Clients zu schaffen.

- **6.1. Server-Implementierung**

MCP-Server sind die Brücke zwischen der abstrakten Welt des Protokolls und den konkreten Funktionalitäten externer Systeme. Ihre Qualität bestimmt maßgeblich die Nützlichkeit des gesamten MCP-Systems.

- **Designprinzipien:**
    
    - **Robustheit:** Server **MÜSSEN** so konzipiert sein, dass sie Fehlerfälle, unerwartete Eingaben und ungültige Anfragen tolerant behandeln. Dies beinhaltet eine umfassende Fehlerbehandlung gemäß dem JSON-RPC-Standard und aussagekräftiges Logging für Diagnosezwecke.1 Ungültige Anfragen sollten mit entsprechenden Fehlermeldungen beantwortet und nicht zum Absturz des Servers führen.
    - **Erweiterbarkeit:** Das Design des Servers **SOLLTE** darauf ausgelegt sein, dass neue Tools, Ressourcen und Prompts mit minimalem Aufwand hinzugefügt oder bestehende modifiziert werden können. Eine modulare Architektur ist hier oft vorteilhaft.
    - **Effizienz:** Insbesondere bei häufig genutzten Funktionen oder beim Zugriff auf große Datenmengen ist auf eine performante Implementierung der Datenabfragen und Tool-Ausführungen zu achten. Langsame Server können die Benutzererfahrung der Host-Anwendung negativ beeinflussen.
    - **Zustandsmanagement:** Da MCP-Verbindungen zustandsbehaftet sind, **MUSS** ein sorgfältiges Session-Management implementiert werden.10 Dies umfasst die korrekte Initialisierung und Verwaltung des Zustands pro Client, die Behandlung von Verbindungsabbrüchen (z.B. durch Timeouts) und die zuverlässige Freigabe von Ressourcen (Speicher, Handles, etc.), wenn eine Sitzung beendet wird, um Ressourcenlecks zu vermeiden.
- Wrapper für externe Systeme:
    
    In vielen Fällen agieren MCP-Server als Wrapper oder Adapter für bereits bestehende APIs, Datenbanken, Dateisysteme oder andere unternehmensinterne oder externe Dienste.3 Die inhärente Komplexität dieser Backend-Systeme (z.B. unterschiedliche Authentifizierungsmethoden, Datenformate oder Fehlerbehandlungslogiken) SOLLTE vor dem MCP-Client verborgen werden. Der Server hat die Aufgabe, eine saubere, standardisierte MCP-Schnittstelle anzubieten, die diese Komplexität abstrahiert.
    
- Best Practices für Tool-, Resource- und Prompt-Definitionen:
    
    Die Qualität der Definitionen von Tools, Ressourcen und Prompts auf dem Server ist entscheidend, da sie die Schnittstelle darstellen, mit der LLMs und Benutzer interagieren.
    
    - **Klare Semantik:** Namen, Beschreibungen und Parameter von Tools, Ressourcen und Prompts **MÜSSEN** präzise, verständlich und eindeutig sein. Diese Informationen werden oft direkt in den Benutzeroberflächen der Host-Anwendungen angezeigt (wie z.B. Icons und Tooltips in 4) und dienen dem LLM als Grundlage für Entscheidungen (bei Tools).
    - **Granularität:** Es ist oft besser, mehrere spezifische, fokussierte Tools oder Ressourcen anzubieten, anstatt ein einziges, monolithisches Tool oder eine Ressource mit einer Vielzahl von Optionen und komplexer Logik. Dies erleichtert die Nutzung und das Verständnis.
    - **Schema-Validierung:** Eingabeparameter für Tools und die Struktur von Ressourcen **MÜSSEN** serverseitig strikt gegen die zuvor definierten Schemata validiert werden. Anfragen, die nicht dem Schema entsprechen, sind mit einem entsprechenden JSON-RPC-Fehler abzulehnen.
    - **Idempotenz:** Wo immer es sinnvoll und möglich ist, **SOLLTEN** Tools idempotent gestaltet sein. Das bedeutet, dass eine mehrfache Ausführung des Tools mit denselben Eingangsparametern immer zum selben Ergebnis führt und keine unerwünschten Mehrfach-Seiteneffekte verursacht.
- Sprachagnostische Überlegungen und SDK-Nutzung:
    
    MCP-Server können prinzipiell in jeder Programmiersprache entwickelt werden (z.B. Python, TypeScript, Java, Rust), solange die Implementierung die MCP-Spezifikation hinsichtlich JSON-RPC und der unterstützten Transportmechanismen (stdio, HTTP/SSE) einhält.3
    
    Die Verwendung von offiziellen oder von der Community bereitgestellten Software Development Kits (SDKs) kann die Entwicklung von MCP-Servern (und Clients) erheblich vereinfachen und beschleunigen. SDKs wie mcpr für Rust 15 oder das Python-Paket mcp[cli] 17 abstrahieren viele der Low-Level-Protokolldetails, wie die Serialisierung/Deserialisierung von JSON-RPC-Nachrichten oder das Management der Transportverbindung.7 Beispielsweise bieten rust-mcp-sdk und das zugehörige rust-mcp-schema 15 typensichere Implementierungen der MCP-Schemata für Rust-Entwickler, was die Fehleranfälligkeit reduziert. mcpr 16 geht noch einen Schritt weiter und bietet High-Level-Abstraktionen für Client und Server sowie einen Projektgenerator, um schnell mit der Entwicklung starten zu können.
    
    Die Nutzung solcher SDKs ist nicht nur eine Frage der Bequemlichkeit, sondern ein wichtiger Faktor für die Sicherstellung der Protokollkonformität und die Reduzierung von Implementierungsfehlern. Sie erlauben es Entwicklern, sich stärker auf die eigentliche Anwendungslogik ihrer Tools und Ressourcen zu konzentrieren, anstatt sich mit den Feinheiten der MCP-Protokollmechanik auseinandersetzen zu müssen.
    
- **6.2. Client-Implementierung**
    

MCP-Clients sind die Bindeglieder zwischen der Host-Anwendung und den MCP-Servern. Ihre korrekte Implementierung ist entscheidend für eine nahtlose Benutzererfahrung.

- Integration in Host-Anwendungen:
    
    Clients sind integraler Bestandteil der Host-Anwendung.3 Die Host-Anwendung ist verantwortlich für die Instanziierung, Konfiguration und Verwaltung des Lebenszyklus der Client-Instanzen. Dies beinhaltet auch die Bereitstellung der notwendigen Benutzeroberflächenelemente, insbesondere für die Einholung der Benutzereinwilligung vor dem Aufruf von Tools oder dem Zugriff auf Ressourcen.1
    
- Verbindungsmanagement und Fehlerbehandlung:
    
    Clients MÜSSEN den Status der Verbindung zu ihrem jeweiligen Server aktiv überwachen. Dies beinhaltet die Implementierung einer robusten Logik zur Handhabung von Verbindungsabbrüchen und gegebenenfalls automatische oder benutzerinitiierte Wiederverbindungsversuche. Eine umfassende Fehlerbehandlung für fehlgeschlagene Anfragen oder vom Server gemeldete Fehler (gemäß JSON-RPC-Error-Objekt) ist unerlässlich, um dem Benutzer aussagekräftiges Feedback geben zu können. Clients SOLLTEN auch Timeouts für Serverantworten implementieren, um zu verhindern, dass die Host-Anwendung bei einem nicht antwortenden Server blockiert.
    
- Umgang mit Server-Capabilities:
    
    Ein zentraler Aspekt der Client-Implementierung ist der dynamische Umgang mit den vom Server während der initialize-Phase angebotenen Fähigkeiten (capabilities). Clients MÜSSEN in der Lage sein, diese Informationen zu parsen und ihre Funktionalität bzw. die der Host-Anwendung entsprechend anzupassen. Beispielsweise SOLLTEN UI-Elemente, die dem Benutzer verfügbare Tools, Ressourcen oder Prompts anzeigen, dynamisch basierend auf den vom Server gemeldeten Fähigkeiten generiert und aktualisiert werden.4
    
- **6.3. Allgemeine Richtlinien**
    

Diese Richtlinien gelten sowohl für Server- als auch für Client-Implementierungen.

- Konfigurationsmanagement:
    
    Sensible Informationen wie API-Schlüssel, Authentifizierungstokens (z.B. das in 8 erwähnte GITHUB_PERSONAL_ACCESS_TOKEN) oder Datenbank-Zugangsdaten MÜSSEN sicher verwaltet werden. Sie DÜRFEN NICHT fest im Quellcode verankert sein. Stattdessen SOLLTEN Mechanismen wie Umgebungsvariablen, sicher gespeicherte Konfigurationsdateien mit restriktiven Zugriffsberechtigungen oder dedizierte Secrets-Management-Systeme verwendet werden.8
    
- Versionierung:
    
    Es wird RECOMMENDED, semantische Versionierung (SemVer) für MCP-Server und -Clients zu verwenden, um Änderungen und Kompatibilität klar zu kommunizieren. Die protocolVersion, die während des MCP-Handshakes (initialize-Methode) ausgetauscht wird 7, ist entscheidend für die Sicherstellung der grundlegenden Protokollkompatibilität zwischen Client und Server. Anwendungen MÜSSEN auf Inkompatibilitäten bei der Protokollversion angemessen reagieren.
    
- Teststrategien:
    
    Eine umfassende Teststrategie ist unerlässlich für die Entwicklung qualitativ hochwertiger MCP-Komponenten.
    
    - **Unit-Tests:** Testen Sie einzelne Module und Funktionen isoliert (z.B. die Logik eines spezifischen Tools auf dem Server, die Parsing-Logik für Server-Antworten im Client).
    - **Integrationstests:** Testen Sie den gesamten MCP-Fluss zwischen einem Client und einem Server, einschließlich des Handshakes, des Aufrufs von Tools/Ressourcen und der Fehlerbehandlung.
    - **Mocking:** Verwenden Sie Mocking-Frameworks, um Abhängigkeiten zu externen Systemen (z.B. Datenbanken, Drittanbieter-APIs, die ein Server wrappt) während der Tests zu isolieren und kontrollierbare Testbedingungen zu schaffen.
    - **Sicherheitstests:** Testen Sie explizit sicherheitsrelevante Aspekte wie die korrekte Implementierung von Einwilligungsabfragen (im Host), die Validierung von Eingaben und die Handhabung von Authentifizierung und Autorisierung.

**7. Sicherheitsrichtlinien und Trust & Safety im MCP**

Das Model Context Protocol ermöglicht durch seinen direkten Zugriff auf Daten und die Ausführung von Code potenziell sehr mächtige Funktionalitäten. Mit dieser Macht geht jedoch eine erhebliche Verantwortung einher. Alle Entwickler und Implementierer von MCP-Komponenten **MÜSSEN** den Sicherheits- und Vertrauensaspekten höchste Priorität einräumen.1 Die folgenden Prinzipien und Richtlinien sind nicht optional, sondern fundamental für den Aufbau eines vertrauenswürdigen MCP-Ökosystems.

- **7.1. Grundprinzipien (gemäß 1)**

Die MCP-Spezifikation selbst legt vier zentrale Sicherheitsprinzipien fest, die als Leitfaden für alle Implementierungen dienen müssen:

- **User Consent and Control (Benutzereinwilligung und -kontrolle):**
    
    - Benutzer **MÜSSEN** explizit allen Datenzugriffen und Operationen, die über MCP erfolgen, zustimmen. Es ist nicht ausreichend, dass eine Aktion technisch möglich ist; der Benutzer muss sie verstehen und ihr aktiv zustimmen.1
    - Benutzer **MÜSSEN** jederzeit die Kontrolle darüber behalten, welche ihrer Daten mit welchen Servern geteilt und welche Aktionen von Tools in ihrem Namen ausgeführt werden.1
    - Implementierer (insbesondere von Host-Anwendungen) **SOLLTEN** klare, verständliche und leicht zugängliche Benutzeroberflächen bereitstellen, über die Benutzer Aktivitäten überprüfen, genehmigen oder ablehnen können.1 Ein Beispiel hierfür ist das in 4 gezeigte Popup-Fenster, das vor der Nutzung eines Tools um Bestätigung bittet.
- **Data Privacy (Datenschutz):**
    
    - Hosts **MÜSSEN** die explizite Benutzereinwilligung einholen, _bevor_ irgendwelche Benutzerdaten an MCP-Server weitergegeben werden.1
    - Hosts **DÜRFEN** Ressourcendaten, die sie von Servern erhalten oder selbst verwalten, NICHT ohne erneute, spezifische Benutzereinwilligung an andere Stellen (z.B. andere Server, Dienste Dritter) übertragen.1
    - Alle Benutzerdaten, die im Kontext von MCP verarbeitet werden, **SOLLTEN** mit geeigneten technischen und organisatorischen Maßnahmen, einschließlich Zugriffskontrollen, geschützt werden.1
- **Tool Safety (Toolsicherheit):**
    
    - Tools, die von MCP-Servern angeboten werden, können potenziell beliebigen Code ausführen oder weitreichende Aktionen in externen Systemen initiieren. Sie **MÜSSEN** daher mit äußerster Vorsicht behandelt werden.1
    - Hosts **MÜSSEN** die explizite Benutzereinwilligung einholen, _bevor_ irgendein Tool aufgerufen wird.1
    - Benutzer **SOLLTEN** in die Lage versetzt werden zu verstehen, welche Aktionen ein Tool ausführt und welche potenziellen Konsequenzen dies hat, bevor sie dessen Nutzung autorisieren.1
    - Ein wichtiger Aspekt ist, dass Beschreibungen des Tool-Verhaltens (z.B. Annotationen, die vom Server geliefert werden) als potenziell nicht vertrauenswürdig betrachtet werden müssen, es sei denn, sie stammen von einem explizit als vertrauenswürdig eingestuften Server.1 Dies hat erhebliche Implikationen: Host-Anwendungen können sich nicht blind auf die Selbstauskunft eines Servers verlassen. Es könnten Mechanismen zur Verifizierung von Servern oder zur Warnung vor potenziell irreführenden Beschreibungen notwendig werden. Langfristig könnten Reputationssysteme oder Zertifizierungsstellen für MCP-Server entstehen, um die Vertrauenswürdigkeit zu erhöhen. Entwickler sollten sich dieser potenziellen Angriffsvektoren bewusst sein und defensive Designentscheidungen treffen.
- **LLM Sampling Controls (Kontrollen für LLM-Sampling):**
    
    - Da die Sampling-Funktion es Servern ermöglicht, LLM-Interaktionen proaktiv zu initiieren, **MÜSSEN** Benutzer allen solchen LLM-Sampling-Anfragen explizit zustimmen.1
    - Benutzer **SOLLTEN** die Kontrolle darüber haben: (a) ob Sampling durch einen bestimmten Server überhaupt erlaubt ist, (b) welchen genauen Prompt-Inhalt das LLM im Rahmen einer Sampling-Anfrage erhält, und (c) welche Ergebnisse dieser serverseitig initiierten LLM-Verarbeitung der Server einsehen darf.1 Die Protokollarchitektur begrenzt hier absichtlich die Sichtbarkeit des Servers auf Prompts, um die Benutzerkontrolle zu wahren.

Sicherheit im MCP ist eine geteilte Verantwortung zwischen Host, Client und Server. Die Spezifikation 1 betont jedoch wiederholt, dass der _Host_ die Hauptlast bei der direkten Benutzerinteraktion und der Einholung von Einwilligungen trägt. Der Server stellt die Funktionalität bereit, aber der Host ist das Tor zum Benutzer und kontrolliert, was dem Benutzer präsentiert wird und welche Berechtigungen letztendlich erteilt werden. Beispiele wie das Bestätigungs-Popup in Claude Desktop 4 illustrieren diese zentrale Rolle des Hosts in der Praxis. Entwickler von Host-Anwendungen haben daher eine immense Verantwortung, die Einwilligungs- und Kontrollmechanismen korrekt, transparent und benutzerfreundlich zu implementieren. Fehler oder Nachlässigkeiten in diesem Bereich können gravierende Datenschutz- und Sicherheitsverletzungen zur Folge haben. Server-Entwickler müssen sich darauf verlassen können, dass der Host diese kritische Aufgabe zuverlässig erfüllt.

- **7.2. Verantwortlichkeiten des Implementierers (gemäß 1)**

Obwohl das MCP-Protokoll selbst diese Sicherheitsprinzipien nicht auf technischer Ebene erzwingen kann (z.B. kann das Protokoll nicht überprüfen, ob eine UI-Einwilligung tatsächlich stattgefunden hat), legt die Spezifikation klare Erwartungen an die Implementierer fest. Diese **SOLLTEN**:

- Robuste und unmissverständliche Zustimmungs- und Autorisierungsflüsse als integralen Bestandteil ihrer Anwendungen entwerfen und implementieren.
    
- Eine klare und verständliche Dokumentation der Sicherheitsimplikationen ihrer MCP-Integrationen bereitstellen, sowohl für Endbenutzer als auch für andere Entwickler.
    
- Geeignete Zugriffskontrollen und Datenschutzmaßnahmen auf allen Ebenen ihrer Systeme implementieren.
    
- Anerkannte Sicherheits-Best-Practices (z.B. OWASP-Richtlinien) in ihren Integrationen befolgen.
    
- Die Datenschutzimplikationen neuer Funktionen oder Änderungen sorgfältig prüfen und in ihren Designs berücksichtigen (Privacy by Design).
    
- **7.3. Spezifische Sicherheitsrichtlinien für Entwickler**
    

Über die oben genannten Grundprinzipien hinaus gibt es konkrete technische Maßnahmen, die Entwickler ergreifen müssen:

- **Input Validierung:** Alle externen Eingaben – seien es JSON-RPC-Parameter von Clients, Daten von Backend-Systemen, die ein Server verarbeitet, oder Benutzereingaben in der Host-Anwendung – **MÜSSEN** serverseitig (oder an der jeweiligen Verarbeitungsgrenze) sorgfältig validiert werden. Dies ist entscheidend, um Injection-Angriffe (z.B. SQL-Injection, Command-Injection, wenn Tools Shell-Befehle ausführen), Cross-Site-Scripting (XSS, falls Tool-Ausgaben in Web-UIs gerendert werden) und andere datenbasierte Schwachstellen zu verhindern.9
- **Authentifizierung und Autorisierung:** Es **MÜSSEN** robuste Mechanismen zur Authentifizierung von Clients gegenüber Servern (und ggf. umgekehrt) implementiert werden, insbesondere wenn die Kommunikation über unsichere Netzwerke erfolgt oder sensible Daten übertragen werden. Nach erfolgreicher Authentifizierung **MUSS** eine Autorisierungsprüfung erfolgen, um sicherzustellen, dass der authentifizierte Akteur auch die Berechtigung für die angeforderte Operation oder den Datenzugriff hat.
- **Secrets Management:** API-Schlüssel, Datenbank-Passwörter, private Schlüssel und andere "Secrets" **MÜSSEN** sicher gespeichert und gehandhabt werden. Sie dürfen niemals im Quellcode hartcodiert oder unverschlüsselt in Konfigurationsdateien abgelegt werden, die leicht zugänglich sind.8 Mechanismen wie Umgebungsvariablen, verschlüsselte Konfigurations-Stores oder dedizierte Secrets-Management-Systeme sind zu verwenden.
- **Rate Limiting und Quotas:** Um Missbrauch durch übermäßige Anfragen (Denial-of-Service-Angriffe oder einfach fehlerhafte Clients) zu verhindern und die Stabilität des Servers zu gewährleisten, **SOLLTEN** Mechanismen für Rate Limiting (Begrenzung der Anzahl von Anfragen pro Zeiteinheit) und gegebenenfalls Quotas (Begrenzung des Gesamtressourcenverbrauchs) implementiert werden.
- **Audit Logging:** Es **SOLLTE** ein detailliertes Audit-Log aller sicherheitsrelevanten Ereignisse geführt werden. Dazu gehören mindestens: erteilte und abgelehnte Einwilligungen, Aufrufe kritischer Tools, fehlgeschlagene Authentifizierungs- und Autorisierungsversuche sowie signifikante Konfigurationsänderungen. Solche Logs sind unerlässlich für die spätere Analyse von Sicherheitsvorfällen (Forensik) und können für Compliance-Anforderungen notwendig sein.5
- **Abhängigkeitsmanagement:** Software-Abhängigkeiten (Bibliotheken, Frameworks) **MÜSSEN** regelmäßig auf bekannte Schwachstellen überprüft und zeitnah aktualisiert werden. Die Verwendung veralteter Komponenten mit bekannten Sicherheitslücken ist ein häufiges Einfallstor für Angreifer.

Die folgende Tabelle dient als Checkliste für Entwickler, um die Einhaltung der Sicherheitsprinzipien zu unterstützen:

**Tabelle 6: Checkliste der Sicherheitsprinzipien für MCP-Entwickler**

|   |   |   |   |
|---|---|---|---|
|**Sicherheitsprinzip**|**Konkrete "Do's" für die Implementierung**|**Konkrete "Don'ts" (zu vermeidende Praktiken)**|**Relevante MCP-Komponente(n)**|
|**User Consent & Control**|Klare, granulare Einwilligungsdialoge in der Host-UI implementieren. Benutzer über Zweck und Umfang jeder Aktion/jedes Datenzugriffs informieren. Widerruf ermöglichen.|Implizite Einwilligungen annehmen. Unklare oder versteckte Einwilligungsoptionen. Fehlende Möglichkeit zum Widerruf.|Host|
|**Data Privacy**|Datenminimierung praktizieren. Zugriffskontrollen implementieren. Sichere Übertragung (HTTPS für HTTP/SSE). Benutzereinwilligung vor _jeder_ Datenweitergabe einholen.|Unnötige Daten sammeln/speichern. Daten ohne explizite Zustimmung weitergeben. Schwache oder fehlende Verschlüsselung sensibler Daten.|Host, Client, Server|
|**Tool Safety**|Explizite Benutzerzustimmung vor _jedem_ Tool-Aufruf. Tool-Beschreibungen kritisch prüfen (wenn nicht von vertrauenswürdigem Server). Sandboxing erwägen.|Automatische Tool-Ausführung ohne Zustimmung. Blindes Vertrauen in Server-Beschreibungen. Ausführung von Tools mit übermäßigen Berechtigungen.|Host, Server|
|**LLM Sampling Controls**|Explizite Benutzerzustimmung für Sampling-Anfragen. Benutzerkontrolle über Prompt-Inhalt und Ergebnis-Sichtbarkeit für den Server sicherstellen.|Sampling ohne explizite Zustimmung aktivieren. Dem Server unkontrollierten Zugriff auf LLM-Interaktionen gewähren.|Host, Client, Server|
|**Input Validierung**|Alle Eingaben (Parameter, Daten) serverseitig strikt validieren (Typ, Länge, Format, erlaubte Werte).|Eingaben von Clients/Servern blind vertrauen. Fehlende oder unzureichende Validierung.|Server, Client (Host-UI)|
|**Authentifizierung/Autorisierung**|Starke Authentifizierungsmethoden für Clients/Server verwenden. Zugriff auf Ressourcen/Tools basierend auf Berechtigungen prüfen.|Schwache oder keine Authentifizierung. Fehlende Autorisierungsprüfungen (Zugriff für alle authentifizierten Entitäten).|Client, Server|
|**Secrets Management**|Secrets sicher speichern (Umgebungsvariablen, Vaults). Zugriff auf Secrets minimieren. Regelmäßige Rotation von Schlüsseln.|Secrets im Code hartcodieren. Secrets unverschlüsselt in Konfigurationsdateien speichern. Lange Gültigkeitsdauern für Secrets.|Client, Server|
|**Audit Logging**|Sicherheitsrelevante Ereignisse (Einwilligungen, Tool-Aufrufe, Fehler) detailliert protokollieren. Logs sicher speichern und regelmäßig überprüfen.|Fehlendes oder unzureichendes Logging. Logs an unsicheren Orten speichern oder nicht vor Manipulation schützen.|Host, Client, Server|

Diese Checkliste dient als praktisches Werkzeug während des gesamten Entwicklungszyklus, um sicherzustellen, dass kritische Sicherheitsaspekte nicht übersehen werden. Sie ist ein Muss für die Entwicklung vertrauenswürdiger MCP-Anwendungen.

**8. Anwendungsfälle und Beispiele (Kurzübersicht)**

Die Flexibilität des Model Context Protocol ermöglicht eine breite Palette von Anwendungsfällen, die von der Anreicherung von LLM-Antworten mit Echtzeitdaten bis hin zur Orchestrierung komplexer, agentischer Workflows reichen. Die Stärke von MCP liegt hierbei insbesondere in seiner Fähigkeit, domänenspezifisches Wissen und spezialisierte Tools für LLMs zugänglich zu machen. LLMs verfügen zwar über ein breites Allgemeinwissen, ihnen fehlt jedoch oft der aktuelle, spezifische Kontext oder die Fähigkeit zur direkten Interaktion mit proprietären Systemen – Lücken, die MCP schließen kann. Entwickler sollten MCP als ein Mittel betrachten, um das "Gehirn" eines LLMs mit den "Augen, Ohren und Händen" zu versehen, die es benötigt, um in spezifischen Domänen wertvolle und präzise Arbeit zu leisten. Der Wert einer MCP-Implementierung steigt somit mit der Relevanz, Einzigartigkeit und Leistungsfähigkeit der angebundenen Daten und Tools.

- 8.1. Real-time Grounding für Finanzrisikobewertung
    
    Finanzinstitute können MCP nutzen, um LLMs direkten Zugriff auf aktuelle Unternehmensdaten zu ermöglichen. Dies umfasst Transaktionshistorien, Betrugsdatenbanken und Kundeninformationen. Solche Integrationen erlauben es KI-Systemen, in Echtzeit Betrug zu erkennen, Risiken präziser zu bewerten und Identitäten zu verifizieren, während gleichzeitig strenge Compliance-Vorschriften eingehalten werden.5
    
- 8.2. Personalisierte Gesundheitsversorgung und Patientenreisen
    
    Im Gesundheitswesen können KI-gestützte Anwendungen, die über MCP angebunden sind, Patienten bei Aufgaben wie der Terminplanung oder der Erinnerung an Rezeptaktualisierungen unterstützen. MCP gewährleistet hierbei den sicheren und konformen Zugriff auf sensible Patientenhistorien, wodurch personalisierte Interaktionen unter Wahrung des Datenschutzes ermöglicht werden.5
    
- 8.3. Customer 360 für Handel und Telekommunikation
    
    Um personalisierte Kundenerlebnisse zu schaffen, benötigen Unternehmen im Einzelhandel und in der Telekommunikationsbranche einen umfassenden Echtzeit-Überblick über ihre Kunden. Ein MCP-Server kann diesen Kontext liefern, indem er Bestelldaten, frühere Interaktionen, Präferenzen und den aktuellen Servicestatus aus verschiedenen Backend-Systemen zusammenführt und der KI-Anwendung zur Verfügung stellt.5
    
- 8.4. Konversationelle und agentische KI-Workflows
    
    MCP ist ein Schlüssel-Enabler für anspruchsvolle konversationelle und agentische KI-Workflows, die komplexe Geschäftsoperationen autonom oder teilautonom durchführen. Ein LLM-basierter Agent könnte beispielsweise über MCP ein Support-Ticket in einem System erstellen, parallel dazu regulatorische Vorgaben in einer Wissensdatenbank prüfen und den Lieferstatus einer Bestellung über ein weiteres angebundenes System abfragen.5 MCP stellt hierfür sowohl den notwendigen Kontext als auch die Aktionsmöglichkeiten (Tools) bereit.
    
- 8.5. GitHub-Integration für Entwickler-Workflows
    
    Ein besonders anschauliches Beispiel ist die Integration von Entwicklungswerkzeugen mit GitHub über MCP. Ein MCP-Server, der die GitHub-API kapselt, kann es einer KI wie GitHub Copilot (oder einer anderen IDE-integrierten Assistenz) ermöglichen, direkt mit GitHub-Repositories zu interagieren.8
    
    - **Beispiel-Setup:** In Visual Studio Code kann beispielsweise das NPX-Paket `@modelcontextprotocol/server-github` als MCP-Server konfiguriert werden. Für die Authentifizierung gegenüber der GitHub-API wird ein `GITHUB_PERSONAL_ACCESS_TOKEN` sicher in der Konfiguration hinterlegt.8
    - **Mögliche Aktionen:** Die KI kann dann Issues zusammenfassen oder neu erstellen, Dateien im Repository lesen, Code durchsuchen oder sogar Pull Requests analysieren und kommentieren.8 Ein spezifischer Anwendungsfall ist ein PR-Review-Server, der automatisch Details zu Pull Requests und die geänderten Dateien von GitHub abruft, diese Code-Änderungen (z.B. mittels Claude Desktop über MCP) analysieren lässt und darauf basierend Zusammenfassungen oder Vorschläge für das Review generiert.17
- 8.6. Web-Suche und erweiterte Reasoning-Fähigkeiten
    
    Um LLMs mit aktuellen Informationen aus dem Internet zu versorgen, kann ein MCP-Server eine Websuchfunktion als Tool bereitstellen. Eine Host-Anwendung wie Claude Desktop kann dieses Tool dann nutzen, um Anfragen des Benutzers mit aktuellen Suchergebnissen zu beantworten oder seine Wissensbasis zu erweitern.4 Ein interessanter Aspekt ist, dass die KI das Such-Tool iterativ verwenden kann, um sich auf verschiedene Facetten einer komplexen Anfrage zu konzentrieren und so fundiertere Antworten zu generieren.4
    

Diese Beispiele illustrieren nur einen Bruchteil der Möglichkeiten. Die wahre Stärke von MCP entfaltet sich, wenn Entwickler beginnen, eigene, hochspezialisierte Server für ihre jeweiligen Domänen und Anwendungsfälle zu erstellen.

**9. Schlussfolgerungen und Empfehlungen**

Das Model Context Protocol (MCP) stellt einen signifikanten Fortschritt in der Standardisierung der Interaktion zwischen KI-Systemen und ihrer externen Umgebung dar. Es bietet ein robustes Framework, das darauf abzielt, die Komplexität von Integrationen zu reduzieren, die Entwicklungsgeschwindigkeit zu erhöhen und die Schaffung interoperabler, kontextbewusster und handlungsfähiger KI-Anwendungen zu fördern.

**Kernelemente für Entwickler:**

- **Architekturverständnis:** Ein tiefes Verständnis des Client-Host-Server-Modells und der jeweiligen Verantwortlichkeiten ist fundamental. Insbesondere die Rolle des Hosts bei der Durchsetzung von Sicherheitsrichtlinien und Benutzereinwilligungen kann nicht genug betont werden.
- **Protokollmechanik:** Vertrautheit mit JSON-RPC 2.0, den MCP-spezifischen Methoden (insbesondere `initialize`) und den Transportmechanismen (`stdio`, `HTTP/SSE`) ist für die Implementierung unerlässlich. Die Zustandsbehaftung der Verbindungen erfordert sorgfältiges Design im Hinblick auf Session-Management und Fehlerbehandlung.
- **Fähigkeitsdesign:** Die klare Unterscheidung und das durchdachte Design von Tools, Ressourcen und Prompts auf Serverseite sind entscheidend für die Nützlichkeit und Benutzerfreundlichkeit des MCP-Systems. Die Sampling-Funktion bietet mächtige Möglichkeiten, erfordert aber höchste Sorgfalt bei der Implementierung von Kontrollmechanismen.
- **Sicherheit als Priorität:** Die Sicherheitsprinzipien des MCP (User Consent and Control, Data Privacy, Tool Safety, LLM Sampling Controls) müssen von Beginn an in jedes Design und jede Implementierung integriert werden. Dies ist eine geteilte Verantwortung, bei der Hosts eine Schlüsselrolle spielen.

**Empfehlungen für die Implementierung:**

1. **SDKs nutzen:** Entwickler **SOLLTEN** wann immer möglich auf offizielle oder etablierte Community-SDKs zurückgreifen. Diese abstrahieren viele Protokolldetails, reduzieren die Fehleranfälligkeit und beschleunigen die Entwicklung (siehe 15).
2. **Sicherheitsrichtlinien strikt befolgen:** Die in Abschnitt 7 dargelegten Sicherheitsprinzipien und -richtlinien **MÜSSEN** als integraler Bestandteil des Entwicklungsprozesses betrachtet werden. Insbesondere die Implementierung robuster Einwilligungs- und Autorisierungsflüsse ist kritisch.
3. **Klare und granulare Schnittstellen definieren:** Server-Entwickler **SOLLTEN** großen Wert auf klare, verständliche und granulare Definitionen ihrer Tools, Ressourcen und Prompts legen. Dies verbessert die Nutzbarkeit sowohl für LLMs als auch für menschliche Benutzer.
4. **"Additional Utilities" implementieren:** Funktionen wie `Ping`, `Cancellation` und `Progress Tracking` **SOLLTEN** als Standard für robuste und benutzerfreundliche MCP-Anwendungen angesehen und implementiert werden, nicht als optionale Extras.
5. **Umfassend testen:** Eine gründliche Teststrategie, die Unit-, Integrations- und Sicherheitstests umfasst, ist unerlässlich, um die Qualität und Zuverlässigkeit von MCP-Komponenten sicherzustellen.
6. **Dokumentation pflegen:** Sowohl Server- als auch Client-Implementierungen **SOLLTEN** gut dokumentiert werden, um die Wartung, Weiterentwicklung und Nutzung durch andere Entwickler zu erleichtern.

Die Einführung von MCP hat das Potenzial, ein lebendiges Ökosystem von interoperablen KI-Anwendungen und -Diensten zu schaffen, ähnlich wie es das Language Server Protocol für Entwicklungswerkzeuge getan hat. Für Entwickler bietet MCP die Möglichkeit, sich von repetitiver Integrationsarbeit zu befreien und sich stattdessen auf die Schaffung innovativer KI-Funktionalitäten zu konzentrieren. Die Einhaltung der hier dargelegten Spezifikationen und Richtlinien ist der Schlüssel, um dieses Potenzial voll auszuschöpfen und vertrauenswürdige, leistungsfähige KI-Systeme der nächsten Generation zu entwickeln.

**Anhang A: Glossar der Begriffe**

- **Client:** Eine Komponente innerhalb einer Host-Anwendung, die eine 1:1-Verbindung zu einem MCP-Server verwaltet und die Kommunikation orchestriert.
- **Host:** Die primäre Anwendung, mit der der Benutzer interagiert und die MCP-Clients koordiniert sowie Sicherheitsrichtlinien durchsetzt.
- **HTTP/SSE:** Hypertext Transfer Protocol mit Server-Sent Events; ein Transportmechanismus für MCP über Netzwerke.
- **JSON-RPC 2.0:** Ein leichtgewichtiges Remote Procedure Call Protokoll, das von MCP für die Kommunikation verwendet wird.
- **MCP (Model Context Protocol):** Ein offener Standard zur Verbindung von KI-Anwendungen mit externen Tools, Datenquellen und Systemen.
- **Prompt (MCP):** Eine benutzergesteuerte, vordefinierte Vorlage zur optimalen Nutzung von Tools oder Ressourcen.
- **Resource (MCP):** Eine anwendungsgesteuerte Datenquelle, auf die LLMs zugreifen können, um Informationen abzurufen.
- **Sampling (MCP):** Eine serverinitiierte, agentische LLM-Interaktion, die explizite Client-Fähigkeit und Benutzerzustimmung erfordert.
- **Server (MCP):** Ein externes Programm oder Dienst, das Tools, Ressourcen und Prompts über eine standardisierte MCP-API bereitstellt.
- **stdio (Standard Input/Output):** Ein Transportmechanismus für MCP, wenn Client und Server auf derselben Maschine laufen.
- **Tool (MCP):** Eine modellgesteuerte Funktion, die LLMs aufrufen können, um spezifische Aktionen auszuführen.

**Anhang B: Referenzen und weiterführende Quellen**

- Offizielle MCP-Spezifikation: [https://modelcontextprotocol.io/specification/2025-03-26](https://modelcontextprotocol.io/specification/2025-03-26) (basierend auf 1)
- MCPR - Model Context Protocol für Rust (SDK): [https://github.com/conikeec/mcpr](https://github.com/conikeec/mcpr) (basierend auf 16)
- Rust MCP Schema (Typensichere MCP-Schemata für Rust): [https://github.com/rust-mcp-stack/rust-mcp-schema](https://github.com/rust-mcp-stack/rust-mcp-schema) (basierend auf 15)
- Einführung in MCP von Phil Schmid: [https://www.philschmid.de/mcp-introduction](https://www.philschmid.de/mcp-introduction) (basierend auf 3)
- OpenCV Blog zu MCP: [https://opencv.org/blog/model-context-protocol/](https://opencv.org/blog/model-context-protocol/) (basierend auf 2)

**Grundidee des Model Context Protocol (MCP):**

Ein MCP würde es verschiedenen Komponenten des Desktops (Anwendungen, Desktop-Shell, Widgets, Assistenten) ermöglichen, Informationen über den aktuellen Benutzerkontext sicher und effizient auszutauschen. "Modell" könnte sich hier auf ein Datenmodell für den Kontext oder auf KI-Modelle beziehen, die diesen Kontext nutzen.

**Phase 1: Konzeptuelle Architektur und Hypothesenformulierung**

1. **Epistemologischer Rahmen & Prämissen:**
    
    - **Ziel:** Verbesserung der Benutzererfahrung durch proaktive, kontextsensitive Unterstützung und Reduzierung repetitiver Aufgaben.
    - **Prämisse 1:** Ein standardisiertes Kontextprotokoll ist notwendig für Interoperabilität.
    - **Prämisse 2:** Benutzer müssen die volle Kontrolle über die Freigabe ihres Kontexts haben (Datenschutz).
    - **Prämisse 3:** Die Integration muss ressourcenschonend sein.
2. **Kernkonzepte & Taxonomie:**
    
    - **Context Provider:** Anwendungen (Texteditor, Browser, Kalender), Systemdienste (Standort, Netzwerk), Desktop-Shell.
    - **Context Consumer:** Desktop-Assistenten, Suchfunktionen, Automatisierungstools, App-Switcher, Benachrichtigungssysteme.
    - **Context Broker:** Eine zentrale Instanz (wahrscheinlich über D-Bus), die Kontextinformationen sammelt, filtert und verteilt.
    - **Context Data Model:** Ein standardisiertes Format (z.B. JSON-LD, ActivityStreams-ähnlich) zur Beschreibung von Kontext-Entitäten (Dokument, Aufgabe, Ort, Person, Ereignis) und deren Beziehungen.
    - **Permission Management:** System zur Verwaltung von Zugriffsrechten auf Kontextdaten.
3. **Hypothesen:**
    
    - **H1 (Sinnhaftigkeit):** Durch MCP können Anwendungen dem Benutzer relevantere Informationen und Aktionen anbieten.
    - **H2 (Benutzerfreundlichkeit):** Eine klare und granulare Kontrolle über die Kontextfreigabe erhöht die Akzeptanz.
    - **H3 (Effizienz):** MCP reduziert die Notwendigkeit für den Benutzer, Informationen manuell zwischen Anwendungen zu kopieren/übertragen.
4. **Operationalisierbare Variablen:**
    
    - Zeitersparnis bei Standardaufgaben.
    - Anzahl der Klicks/Aktionen reduziert.
    - Benutzerzufriedenheit (Umfragen).
    - Adoptionsrate des Protokolls durch Anwendungen.

**Phase 2: Systematische Literaturanalyse und Wissenskartographie**

1. **Recherche existierender Ansätze:**
    - **D-Bus:** Als zugrundeliegende IPC-Mechanismus in Linux-Desktops.
    - **Freedesktop.org-Spezifikationen:** z.B. für Benachrichtigungen, Status-Icons, MIME-Typen.
    - **Nepomuk/Baloo (KDE):** Frühere Versuche semantischer Desktops und deren Herausforderungen (Performance, Komplexität).
    - **ActivityStreams:** Web-Standard zur Beschreibung sozialer Aktivitäten, potenziell adaptierbar.
    - **Telepathy:** Framework für Echtzeitkommunikation.
    - **Mobile OS-Ansätze:** Android Intents, iOS App Intents/Shortcuts für App-Interaktion und Kontext.
2. **Identifikation von Lücken:** Aktuell kein umfassendes, desktopweites, standardisiertes Protokoll für feingranularen Anwendungskontext.

**Phase 3: Datenakquisition und Multi-Methoden-Triangulation (Design-Phase)**

Entwurf des MCP:

1. **Protokoll-Spezifikation:**
    - **Transport:** D-Bus ist die naheliegendste Wahl. Definition von D-Bus-Interfaces, -Methoden und -Signalen.
    - **Datenformat:** Z.B. JSON-basiert mit einem klaren Schema. Überlegung zu Vokabularen (Schema.org könnte Inspiration bieten).
    - **Kernkontext-Typen:** "AktivesDokument", "AusgewählterText", "AktuelleAufgabe", "Standort", "BevorstehendesEreignis", "Kommunikationspartner".
2. **API-Design:**
    - Bibliotheken (z.B. in C/GLib, Qt, Python, Vala) für Anwendungsentwickler zur einfachen Integration.
    - APIs für das Publizieren von Kontext und das Abonnieren von Kontextänderungen.
3. **Permission-Modell:**
    - Integration in bestehende Systeme (z.B. Flatpak Portals, systemweite Datenschutzeinstellungen).
    - Granulare Kontrolle: Pro Anwendung, pro Kontext-Typ.
    - Transparenz: Der Benutzer muss sehen können, welche Anwendung welchen Kontext teilt und wer darauf zugreift.

**Integration in die Linux Desktopumgebung (Sinnvoll & Benutzerfreundlich):**

1. **Zentrale Konfigurationsschnittstelle:**
    
    - Ein Modul in den Systemeinstellungen (z.B. GNOME Control Center, KDE System Settings).
    - **Benutzerfreundlich:** Klare Auflistung aller Apps, die Kontext teilen oder nutzen können. Einfache Schalter zum Aktivieren/Deaktivieren pro App und pro Kontext-Typ.
    - **Sinnvoll:** Standardeinstellungen, die einen guten Kompromiss zwischen Nutzen und Datenschutz bieten (z.B. Kontext nur mit explizit vertrauenswürdigen Systemkomponenten teilen).
2. **Integration in die Desktop-Shell (GNOME Shell, KDE Plasma, etc.):**
    
    - **Globale Suche:** Suchergebnisse basierend auf dem aktuellen Kontext priorisieren (z.B. suche "Bericht" – finde zuerst den Bericht, an dem ich gerade arbeite).
    - **Task-Switcher/Activity Overview:** Zusätzliche Kontextinformationen zu laufenden Anwendungen anzeigen.
    - **Benachrichtigungssystem:** Intelligentere Benachrichtigungen, die den aktuellen Fokus berücksichtigen (z.B. stumm schalten, wenn in Präsentation).
    - **Sinnvoll:** Macht die Shell proaktiver und informativer.
    - **Benutzerfreundlich:** Nahtlose Integration, keine zusätzliche Lernkurve.
3. **Integration in Kernanwendungen:**
    
    - **Dateimanager:** Kontextmenü-Optionen basierend auf dem globalen Kontext (z.B. "An E-Mail mit aktueller Aufgabe anhängen").
    - **Texteditor/IDE:** Code-Vervollständigung oder Dokumentationssuche basierend auf dem Projektkontext, der auch andere Tools umfasst.
    - **E-Mail-Client/Kalender:** Automatische Verknüpfung von E-Mails mit relevanten Dokumenten oder Kalendereinträgen basierend auf dem Kontext.
    - **Browser:** Vorschläge basierend auf dem Inhalt anderer aktiver Anwendungen.
    - **Sinnvoll:** Reduziert manuelle Schritte, fördert Workflows.
    - **Benutzerfreundlich:** Aktionen werden dort angeboten, wo sie gebraucht werden.
4. **Unterstützung für Desktop-Assistenten (Mycroft, Rhasspy, oder zukünftige):**
    
    - **Sinnvoll:** Ermöglicht Assistenten, wirklich "wissend" über die aktuelle Benutzeraktivität zu sein, ohne auf Screen-Scraping oder unsichere Methoden zurückgreifen zu müssen.
        - "Speichere dieses Dokument und sende einen Link an Max Mustermann bezüglich des Projekts Alpha."
        - "Erinnere mich an diese Webseite, wenn ich morgen an meinem Bericht arbeite."
    - **Benutzerfreundlich:** Natürlichsprachliche Interaktion wird mächtiger und relevanter.
5. **Entwicklerfreundlichkeit:**
    
    - **Sinnvoll:** Klare Dokumentation, Beispielimplementierungen, Test-Tools.
    - **Benutzerfreundlich (für Entwickler):** Einfach zu integrierende Bibliotheken, geringer Overhead. Anreize zur Adoption (z.B. Feature in Desktop-Umgebung hervorgehoben).
6. **Datenschutz und Sicherheit als Kernprinzip:**
    
    - **Sinnvoll & Benutzerfreundlich:**
        - **Transparenz:** Der Benutzer muss jederzeit sehen, welche Kontextdaten erfasst und von wem sie genutzt werden. Visuelle Indikatoren.
        - **Kontrolle:** Opt-in statt Opt-out für die meisten Kontextfreigaben. Möglichkeit, den Kontext-Verlauf zu löschen.
        - **Anonymisierung/Aggregation:** Wo möglich, Kontext aggregiert oder anonymisiert teilen.
        - **Sichere Speicherung:** Wenn Kontextdaten persistiert werden, dann verschlüsselt und lokal.
        - **Sandboxing:** Integration mit Flatpak/Snap-Portalen für sicheren Zugriff.

**Beispiele für sinnvolle und benutzerfreundliche Anwendungsfälle:**

- **Intelligente Zwischenablage:** Wenn Text aus einer E-Mail kopiert wird und in einen Task-Manager eingefügt wird, könnte das MCP den Betreff der E-Mail als potenziellen Task-Titel vorschlagen.
- **Automatisierte Workflows:** Beim Öffnen eines bestimmten Projektordners im Dateimanager könnte das MCP assoziierte Anwendungen (IDE, Terminal mit richtigem Pfad, Notizen-App mit Projektnotizen) vorschlagen oder automatisch starten.
- **Proaktive Hilfe:** Wenn der Benutzer in einer Anwendung auf einen Fehler stößt, könnte das MCP (mit Zustimmung) kontextrelevante Log-Dateien oder Hilfeseiten identifizieren.
- **Barrierefreiheit:** Assistive Technologien könnten den detaillierten Kontext nutzen, um Benutzern mit Einschränkungen eine reichhaltigere Interaktion zu ermöglichen.

**Herausforderungen:**

- **Standardisierung:** Einigung auf ein gemeinsames Protokoll und Datenmodell über verschiedene Desktop-Umgebungen und Toolkits hinweg. Freedesktop.org wäre die richtige Anlaufstelle.
- **Adoption:** Entwickler müssen den Mehrwert sehen, um das Protokoll in ihre Anwendungen zu integrieren.
- **Performance:** Der Kontextaustausch darf das System nicht verlangsamen.
- **Komplexität vs. Nutzen:** Das Protokoll muss mächtig genug sein, aber einfach genug für Entwickler und verständlich für Benutzer.
- **Sicherheitsrisiken:** Ein kompromittierter Context Broker könnte sensible Daten leaken.

**Fazit:**

Die Integration eines "Model Context Protocol" in eine Linux-Desktopumgebung erfordert einen durchdachten, standardisierten Ansatz mit starkem Fokus auf Benutzerkontrolle und Datenschutz. Über D-Bus als Transportmechanismus und eine klare API könnten Anwendungen und Systemdienste einen reichen, gemeinsamen Kontext aufbauen. Dies würde intelligentere, proaktivere und letztlich benutzerfreundlichere Desktop-Erfahrungen ermöglichen, indem Workflows vereinfacht, relevante Informationen zur richtigen Zeit bereitgestellt und die Effizienz gesteigert wird. Der Schlüssel zum Erfolg liegt in der offenen Zusammenarbeit, einer guten Entwicklererfahrung und dem unbedingten Schutz der Privatsphäre der Nutzer.
# Detaillierte Technische Spezifikation: KI-Integration (domain::ai)

## I. Einleitung

### A. Zweck und Geltungsbereich

Dieses Dokument spezifiziert die Implementierung des Moduls `domain::ai`, das für die Verwaltung der Interaktion der Desktop-Umgebung mit KI-Modellen verantwortlich ist.

Es legt den Schwerpunkt auf die sichere und kontrollierte Einbindung von KI-Funktionalitäten, insbesondere die Handhabung der Benutzereinwilligung bezüglich des Zugriffs auf deren Daten.

### B. Modulverantwortlichkeiten

Das `domain::ai`-Modul übernimmt folgende Aufgaben:

- Verwaltung des Lebenszyklus von KI-Interaktionskontexten.
    
    - Bereitstellung von Strukturen und Logik zur Nachverfolgung einzelner KI-Sitzungen oder Anfragen.
- Implementierung der Logik für das Einholen, Speichern und Überprüfen von Benutzereinwilligungen (AIConsent) für die Nutzung von KI-Modellen und den Zugriff auf spezifische Datenkategorien (AIDataCategory).
    
    - Definition von Mechanismen, um zu bestimmen, welche Daten für eine bestimmte KI-Aktion erforderlich sind und ob der Benutzer die Verwendung dieser Daten erlaubt hat.
- Verwaltung von Profilen verfügbarer KI-Modelle (AIModelProfile).
    
    - Katalogisierung der Fähigkeiten und Anforderungen verschiedener KI-Modelle, um eine korrekte Einwilligungsverwaltung zu gewährleisten.
- Bereitstellung einer Schnittstelle zur Initiierung von KI-Aktionen und zur Verarbeitung von deren Ergebnissen, unabhängig vom spezifischen KI-Modell oder dem MCP-Protokoll (welches in der Systemschicht implementiert wird).
    
    - Abstraktion der Kommunikation mit den KI-Modellen, um die Kompatibilität zu erhöhen und den Aufwand für andere Module zu minimieren.

### C. Nicht-Zuständigkeiten

Dieses Modul ist nicht verantwortlich für:

- Die Implementierung der UI-Elemente zur Darstellung von KI-Interaktionen oder Einwilligungsabfragen (Aufgabe der Benutzeroberflächenschicht).
    
- Die direkte Kommunikation mit KI-Modellen oder externen Diensten (Aufgabe der Systemschicht, insbesondere des MCP-Clients).
    
- Die Persistenz von Einwilligungen oder Modellprofilen (Delegiert an die Core Layer, z.B. core::config).
    

## II. Datenstrukturen

### A. Kernentitäten

1. **AIInteractionContext**
    
    - Zweck: Repräsentiert eine spezifische Interaktion oder einen Dialog mit einer KI.
        
    - Attribute:
        - `id`: `Uuid` (öffentlich): Eindeutiger Identifikator für den Kontext.
            
        - `creation_timestamp`: `DateTime<Utc>` (öffentlich): Zeitpunkt der Erstellung.
            
        - `active_model_id`: `Option<String>` (öffentlich): ID des aktuell für diesen Kontext relevanten KI-Modells.
            
        - `consent_status`: `AIConsentStatus` (öffentlich): Aktueller Einwilligungsstatus für diesen Kontext.
            
        - `associated_data_categories`: `Vec<AIDataCategory>` (öffentlich): Kategorien von Daten, die für diese Interaktion relevant sein könnten.
            
        - `interaction_history`: `Vec<String>` (privat): Eine einfache Historie der Konversation (z.B. Benutzeranfragen, KI-Antworten).
            
        - `attachments`: `Vec<AttachmentData>` (öffentlich): Angehängte Daten (z.B. Dateipfade, Text-Snippets).
            
    - Invarianten:
        - `id` und `creation_timestamp` sind nach der Erstellung unveränderlich.
            
    - Methoden (konzeptionell):
        - `new(relevant_categories: Vec<AIDataCategory>, initial_attachments: Option<Vec<AttachmentData>>) -> Self`: Erstellt einen neuen Kontext.
        - `update_consent_status(&mut self, status: AIConsentStatus)`: Aktualisiert den Einwilligungsstatus.
            
        - `set_active_model(&mut self, model_id: String)`: Legt das aktive Modell fest.
        - `add_history_entry(&mut self, entry: String)`: Fügt einen Eintrag zur Historie hinzu.
            
        - `add_attachment(&mut self, attachment: AttachmentData)`: Fügt einen Anhang hinzu.
2. **AIConsent**
    
    - Zweck: Repräsentiert die Einwilligung eines Benutzers für eine spezifische Kombination aus KI-Modell und Datenkategorien.
        
    - Attributes:
        - `id`: `Uuid` (öffentlich): Eindeutiger Identifikator für die Einwilligung.
            
        - `user_id`: `String` (öffentlich): Identifikator des Benutzers.
            
        - `model_id`: `String` (öffentlich): ID des KI-Modells, für das die Einwilligung gilt.
            
        - `data_categories`: `Vec<AIDataCategory>` (öffentlich): Datenkategorien, für die die Einwilligung erteilt wurde.
            
        - `granted_timestamp`: `DateTime<Utc>` (öffentlich): Zeitpunkt der Erteilung.
            
        - `expiry_timestamp`: `Option<DateTime<Utc>>` (öffentlich): Optionaler Ablaufzeitpunkt der Einwilligung.
            
        - `is_revoked`: `bool` (öffentlich, initial false): Gibt an, ob die Einwilligung widerrufen wurde.
            
    - Invarianten:
        - `id`, `user_id`, `model_id`, und `granted_timestamp` sind nach der Erstellung unveränderlich.
            
        - `data_categories` sollten nach der Erteilung nicht ohne expliziten Benutzerwunsch modifizierbar sein (neue Einwilligung erforderlich).
            
    - Methoden (konzeptionell):
        - `new(user_id: String, model_id: String, categories: Vec<AIDataCategory>, expiry: Option<DateTime<Utc>>) -> Self`: Erstellt eine neue Einwilligung.
        - `revoke(&mut self)`: Markiert die Einwilligung als widerrufen.
            
3. **AIModelProfile**
    
    - Zweck: Beschreibt ein verfügbares KI-Modell.
        
    - Attribute:
        - `model_id`: `String` (öffentlich): Eindeutiger Identifikator des Modells.
            
        - `display_name`: `String` (öffentlich): Anzeigename des Modells.
            
        - `description`: `String` (öffentlich): Kurze Beschreibung des Modells.
            
        - `provider`: `String` (öffentlich): Anbieter des Modells (z.B. "Local", "CloudProvider").
            
        - `required_consent_categories`: `Vec<AIDataCategory>` (öffentlich): Datenkategorien, für die dieses Modell typischerweise eine Einwilligung benötigt.
            
        - `capabilities`: `Vec<String>` (öffentlich): Liste der Fähigkeiten des Modells (z.B. "text_generation", "image_recognition").
            
    - Invarianten:
        - `model_id` ist eindeutig und unveränderlich.
            
    - Methoden (konzeptionell):
        - `new(...) -> Self`: Erstellt ein neues Modellprofil.
        - `requires_consent_for(&self, categories: &Vec<AIDataCategory>) -> bool`: Prüft, ob für die gegebenen Kategorien eine Einwilligung erforderlich ist.
            
4. **Notification**
    
    - Zweck: Repräsentiert eine einzelne Benachrichtigung.
        
    - Attribute:
        - `id`: `Uuid` (öffentlich): Eindeutiger Identifikator.
            
        - `application_name`: `String` (öffentlich): Name der Anwendung, die die Benachrichtigung gesendet hat.
            
        - `application_icon`: `Option<String>` (öffentlich): Optionaler Pfad oder Name des Icons der Anwendung.
            
        - `summary`: `String` (öffentlich): Kurze Zusammenfassung der Benachrichtigung.
            
        - `body`: `Option<String>` (öffentlich): Detaillierterer Text der Benachrichtigung.
            
        - `actions`: `Vec<NotificationAction>` (öffentlich): Verfügbare Aktionen für die Benachrichtigung.
            
        - `urgency`: `NotificationUrgency` (öffentlich): Dringlichkeitsstufe.
            
        - `timestamp`: `DateTime<Utc>` (öffentlich): Zeitpunkt des Eintreffens.
            
        - `is_read`: `bool` (privat, initial false): Status, ob gelesen.
            
        - `is_dismissed`: `bool` (privat, initial false): Status, ob vom Benutzer aktiv geschlossen.
            
        - `transient`: `bool` (öffentlich, default false): Ob die Benachrichtigung flüchtig ist und nicht in der Historie verbleiben soll.
            
    - Invarianten:
        - `id` und `timestamp` sind unveränderlich.
            
        - `summary` darf nicht leer sein.
            
    - Methoden (konzeptionell):
        - `new(app_name: String, summary: String, urgency: NotificationUrgency) -> Self`: Erstellt eine neue Benachrichtigung.
            
        - `mark_as_read(&mut self)`: Setzt den Lesestatus.
        - `dismiss(&mut self)`: Setzt den Entlassen-Status.
        - `add_action(&mut self, action: NotificationAction)`: Fügt eine Aktion hinzu.
            
5. **NotificationAction**
    
    - Zweck: Definiert eine Aktion, die im Kontext einer Benachrichtigung ausgeführt werden kann.
        
    - Attribute:
        - `key`: `String` (öffentlich): Eindeutiger Schlüssel für die Aktion (z.B. "reply", "archive").
            
        - `label`: `String` (öffentlich): Anzeigename der Aktion.
            
        - `action_type`: `NotificationActionType` (öffentlich): Typ der Aktion (z.B. Callback, Link).
            
6. **AttachmentData**
    
    - Zweck: Repräsentiert angehängte Daten an einen AIInteractionContext.
        
    - Attribute:
        - `id`: `Uuid` (öffentlich): Eindeutiger Identifikator des Anhangs.
            
        - `mime_type`: `String` (öffentlich): MIME-Typ der Daten (z.B. "text/plain", "image/png").
            
        - `source_uri`: `Option<String>` (öffentlich): URI zur Quelle der Daten (z.B. file:///path/to/file).
            
        - `content`: `Option<Vec<u8>>` (öffentlich): Direkter Inhalt der Daten, falls klein.
            
        - `description`: `Option<String>` (öffentlich): Optionale Beschreibung des Anhangs.
            

### B. Modulspezifische Enums

1. **AIConsentStatus**: Enum
    
    - Varianten: `Granted`, `Denied`, `PendingUserAction`, `NotRequired`.
        
2. **AIDataCategory**: Enum
    
    - Varianten: `UserProfile`, `ApplicationUsage`, `FileSystemRead`, `ClipboardAccess`, `LocationData`, `GenericText`, `GenericImage`.
        
3. **NotificationUrgency**: Enum
    
    - Varianten: `Low`, `Normal`, `Critical`.
        
4. **NotificationActionType**: Enum
    
    - Varianten: `Callback`, `OpenLink`.
        
5. **NotificationFilterCriteria**: Enum
    
    - Varianten: `Unread`, `Application(String)`, `Urgency(NotificationUrgency)`.
        
6. **NotificationSortOrder**: Enum
    
    - Varianten: `TimestampAscending`, `TimestampDescending`, `Urgency`.
        

### C. Modulspezifische Konstanten

- `const DEFAULT_NOTIFICATION_TIMEOUT_SECS: u64 = 5;`
    
- `const MAX_NOTIFICATION_HISTORY: usize = 100;`
    
- `const MAX_AI_INTERACTION_HISTORY: usize = 50;`
    

## III. Modulspezifische Funktionen

### A. Traits

1. **AIInteractionLogicService**
    
    Rust
    
    ```
    use crate::core::types::Uuid;
    use crate::core::errors::CoreError;
    use super::types::{AIInteractionContext, AIConsent, AIModelProfile, AIDataCategory, AttachmentData};
    use super::errors::AIInteractionError;
    use async_trait::async_trait;
    
    #[async_trait]
    pub trait AIInteractionLogicService: Send + Sync {
        /// Initiates a new AI interaction context.
        /// Returns the ID of the newly created context.
        async fn initiate_interaction(
            &mut self,
            relevant_categories: Vec<AIDataCategory>,
            initial_attachments: Option<Vec<AttachmentData>>
        ) -> Result<Uuid, AIInteractionError>;
    
        /// Retrieves an existing AI interaction context.
        async fn get_interaction_context(&self, context_id: Uuid) -> Result<AIInteractionContext, AIInteractionError>;
    
        /// Provides or updates consent for a given interaction context and model.
        async fn provide_consent(
            &mut self,
            context_id: Uuid,
            model_id: String,
            granted_categories: Vec<AIDataCategory>,
            consent_decision: bool // true for granted, false for denied
        ) -> Result<(), AIInteractionError>;
    
        /// Retrieves the consent status for a specific model and data categories,
        /// potentially within an interaction context.
        async fn get_consent_for_model(
            &self,
            model_id: &str,
            data_categories: &Vec<AIDataCategory>,
            context_id: Option<Uuid>
        ) -> Result<AIConsentStatus, AIInteractionError>;
    
        /// Adds an attachment to an existing interaction context.
        async fn add_attachment_to_context(
            &mut self,
            context_id: Uuid,
            attachment: AttachmentData
        ) -> Result<(), AIInteractionError>;
    
        /// Lists all available and configured AI model profiles.
        async fn list_available_models(&self) -> Result<Vec<AIModelProfile>, AIInteractionError>;
    
        /// Stores a user's consent decision persistently.
        /// This might be called after `provide_consent` if the consent is to be remembered globally.
        async fn store_consent(&self, consent: AIConsent) -> Result<(), AIInteractionError>;
    
        /// Retrieves all stored consents for a given user (simplified).
        async fn get_all_user_consents(&self, user_id: &str) -> Result<Vec<AIConsent>, AIInteractionError>;
    
        /// Loads AI model profiles, e.g., from a configuration managed by core::config.
        async fn load_model_profiles(&mut self) -> Result<(), AIInteractionError>;
    }
    ```
    
2. **NotificationService**
    
    Rust
    
    ```
    use crate::core::types::Uuid;
    use crate::core::errors::CoreError;
    use super::types::{Notification, NotificationUrgency, NotificationFilterCriteria, NotificationSortOrder};
    use super::errors::NotificationError;
    use async_trait::async_trait;
    
    #[async_trait]
    pub trait NotificationService: Send + Sync {
        /// Posts a new notification to the system.
        /// Returns the ID of the newly created notification.
        async fn post_notification(&mut self, notification_data: Notification) -> Result<Uuid, NotificationError>;
    
        /// Retrieves a specific notification by its ID.
        async fn get_notification(&self, notification_id: Uuid) -> Result<Notification, NotificationError>;
    
        /// Marks a notification as read.
        async fn mark_as_read(&mut self, notification_id: Uuid) -> Result<(), NotificationError>;
    
        /// Dismisses a notification, removing it from active view but possibly keeping it in history.
        async fn dismiss_notification(&mut self, notification_id: Uuid) -> Result<(), NotificationError>;
    
        /// Retrieves a list of currently active (not dismissed, potentially unread) notifications.
        /// Allows filtering and sorting.
        async fn get_active_notifications(
            &self,
            filter: Option<NotificationFilterCriteria>,
            sort_order: Option<NotificationSortOrder>
        ) -> Result<Vec<Notification>, NotificationError>;
    
        /// Retrieves the notification history.
        /// Allows filtering and sorting.
        async fn get_notification_history(
            &self,
            limit: Option<usize>,
            filter: Option<NotificationFilterCriteria>,
            sort_order: Option<NotificationSortOrder>
        ) -> Result<Vec<Notification>, NotificationError>;
    
        /// Clears all notifications from history.
        async fn clear_history(&mut self) -> Result<(), NotificationError>;
    
        /// Sets the "Do Not Disturb" mode.
        async fn set_do_not_disturb(&mut self, enabled: bool) -> Result<(), NotificationError>;
    
        /// Checks if "Do Not Disturb" mode is currently enabled.
        async fn is_do_not_disturb_enabled(&self) -> Result<bool, NotificationError>;
    
        /// Invokes a specific action associated with a notification.
        async fn invoke_action(&mut self, notification_id: Uuid, action_key: &str) -> Result<(), NotificationError>;
    }
    ```
    

### B. Methodenlogik

1. **AIInteractionLogicService::provide_consent**
    
    - Vorbedingung:
        - `context_id` muss einen existierenden AIInteractionContext referenzieren.
            
        - `model_id` muss einem bekannten AIModelProfile entsprechen.
            
    - Logik:
        1. Kontext und Modellprofil laden.
        2. Prüfen, ob die `granted_categories` eine Untermenge der vom Modell potenziell benötigten Kategorien sind.
            
        3. Einen neuen `AIConsent`-Eintrag erstellen oder einen bestehenden aktualisieren.
        4. Den `consent_status` im `AIInteractionContext` entsprechend anpassen.
            
        5. Falls `consent_decision` true ist und die Einwilligung global gespeichert werden soll, `store_consent()` aufrufen.
        6. `AIConsentUpdatedEvent` auslösen.
    - Nachbedingung:
        - Der Einwilligungsstatus des Kontexts ist aktualisiert.
        - Ein `AIConsent`-Objekt wurde potenziell erstellt/modifiziert.
        - Ein Event wurde ausgelöst.
            
2. **NotificationService::post_notification**
    
    - Vorbedingung:
        - `notification_data.summary` darf nicht leer sein.
            
    - Logik:
        1. Validieren der `notification_data`.
        2. Der `Notification` eine neue Uuid und einen `timestamp` zuweisen.
            
        3. Wenn DND-Modus aktiv ist und die `NotificationUrgency` nicht Critical ist, die Benachrichtigung ggf. unterdrücken oder nur zur Historie hinzufügen, ohne sie aktiv anzuzeigen.
            
        4. Die Benachrichtigung zur Liste der `active_notifications` hinzufügen.
        5. Wenn die Benachrichtigung nicht transient ist, sie zur `history` hinzufügen (unter Beachtung von `MAX_NOTIFICATION_HISTORY`).
        6. `NotificationPostedEvent` auslösen (ggf. mit Information, ob sie aufgrund von DND unterdrückt wurde).
    - Nachbedingung:
        - Die Benachrichtigung ist im System registriert und ein Event wurde ausgelöst.
            

## IV. Fehlerbehandlung

### A. AIInteractionError

Rust

```
use thiserror::Error;
use crate::core::types::Uuid;

pub enum AIInteractionError {
    ContextNotFound(Uuid),
    ConsentAlreadyProvided(Uuid),
    #[error("Consent required for model '{model_id}' but not granted for data categories: {missing_categories:?}")]
    ConsentRequired { model_id: String, missing_categories: Vec<String> },
    NoModelAvailable,
    ModelNotFound(String),
    InvalidAttachment(String),
    ConsentStorageError(String),
    ModelProfileLoadError(String),
    CoreError { #[from] source: crate::core::errors::CoreError },
    InternalError(String),
}
```

### B. NotificationError

Rust

```
use thiserror::Error;
use crate::core::types::Uuid;

pub enum NotificationError {
    NotFound(Uuid),
    InvalidData{ summary: String, details: String },
    #[error("Maximum notification history of {max_history} reached. Cannot add new notification: {summary}")]
    HistoryFull { max_history: usize, summary: String },
    ActionNotFound { notification_id: Uuid, action_id: String },
    CoreError { #[from] source: crate::core::errors::CoreError },
    InternalError(String),
}
```

## V. Ereignisse

### A. AIInteractionInitiatedEvent

- Payload-Struktur:
    
    Rust
    
    ```
    pub struct AIInteractionInitiatedEvent {
        pub context_id: Uuid,
        pub relevant_categories: Vec<AIDataCategory>
    }
    ```
    
- Typische Publisher: AIInteractionLogicService Implementierung.
    
- Typische Subscriber: UI-Komponenten, die eine KI-Interaktionsoberfläche öffnen oder vorbereiten; Logging-Systeme.
    
- Auslösebedingungen: Ein neuer AIInteractionContext wurde erfolgreich erstellt via initiate_interaction.
    

### B. AIConsentUpdatedEvent

- Payload-Struktur:
    
    Rust
    
    ```
    pub struct AIConsentUpdatedEvent {
        pub context_id: Option<Uuid>,
        pub model_id: String,
        pub granted_categories: Vec<AIDataCategory>,
        pub consent_status: AIConsentStatus
    }
    ```
    
- Typische Publisher: AIInteractionLogicService Implementierung.
    
- Typische Subscriber: UI-Komponenten, die den Einwilligungsstatus anzeigen oder Aktionen basierend darauf freischalten/sperren; die Komponente, die die eigentliche KI-Anfrage durchführt.
    
- Auslösebedingungen: Eine Einwilligung wurde erteilt, verweigert oder widerrufen (provide_consent, store_consent mit Widerruf).
    

### C. NotificationPostedEvent

- Payload-Struktur:
    
    Rust
    
    ```
    pub struct NotificationPostedEvent {
        pub notification: Notification,
        pub suppressed_by_dnd: bool
    }
    ```
    
- Typische Publisher: NotificationService Implementierung.
    
- Typische Subscriber: UI-Schicht (zur Anzeige der Benachrichtigung), System-Schicht (z.B. um einen Ton abzuspielen, falls nicht unterdrückt).
    
- Auslösebedingungen: Eine neue Benachrichtigung wurde erfolgreich via post_notification verarbeitet.
    

### D. NotificationDismissedEvent

- Payload-Struktur:
    
    Rust
    
    ```
    pub struct NotificationDismissedEvent {
        pub notification_id: Uuid
    }
    ```
    
- Typische Publisher: NotificationService Implementierung.
    
- Typische Subscriber: UI-Schicht (um die Benachrichtigung aus der aktiven Ansicht zu entfernen).
    
- Auslösebedingungen: Eine Benachrichtigung wurde erfolgreich via dismiss_notification geschlossen.
    

### E. NotificationReadEvent

- Payload-Struktur:
    
    Rust
    
    ```
    pub struct NotificationReadEvent {
        pub notification_id: Uuid
    }
    ```
    
- Typische Publisher: NotificationService Implementierung.
    
- Typische Subscriber: UI-Schicht (um den "gelesen"-Status zu aktualisieren).
    
- Auslösebedingungen: Eine Benachrichtigung wurde erfolgreich via mark_as_read als gelesen markiert.
    

### F. DoNotDisturbModeChangedEvent

- Payload-Struktur:
    
    Rust
    
    ```
    pub struct DoNotDisturbModeChangedEvent {
        pub dnd_enabled: bool
    }
    ```
    
- Typische Publisher: NotificationService Implementierung.
    
- Typische Subscriber: UI (DND-Statusanzeige), NotificationService (um Benachrichtigungen zu unterdrücken).
    
- Auslösebedingungen: Der DND-Modus wurde via set_do_not_disturb geändert.
    

## VI. Implementierungsrichtlinien

### A. Modulstruktur

```
src/domain/user_centric_services/
├── mod.rs                      // Deklariert Submodule, exportiert öffentliche Typen/Traits
├── ai_interaction_service.rs  // Implementierung von AIInteractionLogicService
├── notification_service.rs    // Implementierung von NotificationService
├── types.rs                    // Gemeinsame Enums und Structs
└── errors.rs                   // Definition der Fehler-Enums
```

### B. Implementierungsschritte

1. **errors.rs erstellen**: Definiere die AIInteractionError und NotificationError Enums mithilfe von `thiserror`. Stelle sicher, dass sie `Debug`, `Clone`, `PartialEq`, und `Eq` (falls benötigt) implementieren.
2. **types.rs erstellen**: Definiere alle modulspezifischen Enums (AIConsentStatus, AIDataCategory, etc.) und Structs (AIInteractionContext, AIConsent, etc.). Implementiere für diese Strukturen die notwendigen Traits: `Debug`, `Clone`, `PartialEq`, und `Serialize`/`Deserialize` (wo benötigt).
3. **ai_interaction_service.rs Basis**:
    - Definiere den Trait `AIInteractionLogicService`.
    - Erstelle eine Struktur `DefaultAIInteractionLogicService`. Diese Struktur wird Felder für den internen Zustand enthalten.
    - Beginne mit der Implementierung von `#[async_trait] impl AIInteractionLogicService for DefaultAIInteractionLogicService`.
4. **notification_service.rs Basis**:
    - Definiere den Trait `NotificationService`.
    - Erstelle eine Struktur `DefaultNotificationService`. Diese Struktur wird Felder für den internen Zustand enthalten.
    - Beginne mit der Implementierung von `#[async_trait] impl NotificationService for DefaultNotificationService`.
5. **Implementierung der AIInteractionLogicService-Methoden**: Implementiere jede Methode des Traits schrittweise. Achte auf korrekte Fehlerbehandlung und Rückgabe der definierten `AIInteractionError`-Varianten. Implementiere die Interaktion mit der Kernschicht (z.B. für Persistenz). Löse die entsprechenden Events aus.
6. **Implementierung der NotificationService-Methoden**: Implementiere jede Methode des Traits. Implementiere die Logik für DND, Historienbegrenzung, Filterung und Sortierung. Verwende `NotificationError`-Varianten für Fehlerfälle. Löse die spezifizierten Notification-Events aus.
7. **mod.rs erstellen**: Deklariere die Submodule und exportiere alle öffentlichen Typen, Traits, und Fehler-Enums, die von außerhalb dieses Moduls verwendet werden sollen.
8. **Unit-Tests**: Schreibe Unit-Tests parallel zur Implementierung jeder Methode und jeder komplexen Logikeinheit. Mocke dabei gegebenenfalls Abhängigkeiten zur Kernschicht.
# Executive Summary

Purpose and Scope: Dieses Dokument liefert eine Ultra-Feinspezifikation für sämtliche Schnittstellen und Implementierungen des Model Context Protocol (MCP) innerhalb des NovaDE-Projekts. Es dient als definitive technische Referenz für Entwickler und Architekten, die an der Integration von MCP beteiligt sind. Die Spezifikation zielt darauf ab, eine klare, präzise und unzweideutige Grundlage für die Entwicklung zu schaffen, die eine direkte Umsetzung ermöglicht.


MCP in NovaDE: Die strategische Entscheidung zur Adaption von MCP im NovaDE-Projekt basiert auf der Erwartung signifikanter Vorteile. Dazu zählen die standardisierte Integration von KI-Modellen, eine verbesserte kontextuelle Wahrnehmung für KI-Agenten und der modulare Zugriff auf die domänenspezifischen Funktionalitäten von NovaDE.1 MCP positioniert NovaDE so, dass es von einem wachsenden Ökosystem an KI-Werkzeugen und -Modellen profitieren kann, indem eine standardisierte Interaktionsebene bereitgestellt wird.1 Diese Ausrichtung deutet auf eine zukunftsorientierte Architektur hin, die auf Interoperabilität und Erweiterbarkeit abzielt. Da MCP als universeller Standard gilt und von führenden KI-Akteuren adaptiert wird 1, kann NovaDE durch dessen Nutzung einfacher mit diversen KI-Modellen integriert werden und von gemeinschaftlich entwickelten MCP-Servern oder -Clients profitieren.


Key Deliverables: Diese Spezifikation umfasst detaillierte MCP-Nachrichtenformate, NovaDE-spezifische Schnittstellendefinitionen (Ressourcen, Werkzeuge, Aufforderungen, Benachrichtigungen), Integrationsstrategien mit der (aktuell separaten) "Domänenschicht-Spezifikation", Implementierungsrichtlinien, Sicherheitsüberlegungen, Fehlerbehandlung und Versionierung.


Critical Dependencies: Es wird explizit auf die Abhängigkeit von der "Domänenschicht-Spezifikation" für die konkrete Abbildung von Domänenfunktionalitäten auf MCP-Konstrukte hingewiesen. Dieses Dokument stellt den Rahmen für solche Abbildungen bereit. Der Erfolg der MCP-Integration hängt maßgeblich von einer wohldefinierten "Domänenschicht-Spezifikation" ab; ohne diese bleiben die MCP-Schnittstellen abstrakt.


Intended Audience: Dieses Dokument richtet sich an technische Leiter, Softwarearchitekten und Senior-Entwickler des NovaDE-Projekts.

2. Model Context Protocol (MCP) Grundlagen für NovaDE

2.1. MCP Protokollübersicht

Definition und Ziele: Das Model Context Protocol (MCP) ist ein offener Standard, der entwickelt wurde, um die Art und Weise zu standardisieren, wie KI-Modelle, insbesondere Large Language Models (LLMs), mit externen Werkzeugen, Systemen und Datenquellen integriert werden und Daten austauschen.1 Es fungiert als universelle Schnittstelle für den Kontexaustausch zwischen KI-Assistenten und Software-Umgebungen, indem es modellagnostische Mechanismen zum Lesen von Dateien, Ausführen von Funktionen und Handhaben kontextueller Anfragen bereitstellt.1 Das primäre Ziel von MCP ist es, die Herausforderung isolierter Informationssilos und proprietärer Legacy-Systeme zu adressieren, die die Fähigkeiten selbst hochentwickelter KI-Modelle einschränken.1
Kernkonzepte:

Client-Host-Server-Architektur: MCP basiert auf einem Client-Host-Server-Muster.2

MCP Clients: Sind Protokoll-Clients, die typischerweise in KI-Anwendungen oder Agenten eingebettet sind und eine Eins-zu-Eins-Verbindung zu MCP-Servern herstellen. Sie sind für die Aushandlung von Fähigkeiten und die Orchestrierung von Nachrichten zwischen sich und dem Server zuständig.2
MCP Hosts: Agieren als Container oder Koordinatoren für eine oder mehrere Client-Instanzen. Sie verwalten den Lebenszyklus und die Sicherheitsrichtlinien (z.B. Berechtigungen, Benutzerautorisierung, Durchsetzung von Einwilligungsanforderungen) und überwachen, wie die KI-Integration innerhalb jedes Clients erfolgt, indem sie Kontext sammeln und zusammenführen.2 Ein Beispiel hierfür ist die Claude Desktop App.1
MCP Server: Sind Programme, die Datenquellen, APIs oder andere Dienstprogramme (wie CRM-Systeme, Git-Repositories oder Dateisysteme) umschließen und deren Fähigkeiten über die standardisierte MCP-Schnittstelle bereitstellen. Sie müssen Sicherheitsbeschränkungen und Benutzerberechtigungen, die vom Host durchgesetzt werden, einhalten.2


Ressourcen (Resources): Stellen Dateneinheiten dar, die von MCP-Servern exponiert werden. Sie können beliebige Entitäten sein – Dateien, API-Antworten, Datenbankabfragen, Systeminformationen etc..5 Sie sind vergleichbar mit GET-Endpunkten in einer Web-API und dienen dazu, Informationen in den Kontext des LLMs zu laden.6
Werkzeuge (Tools): Repräsentieren Funktionalitäten, die von MCP-Servern bereitgestellt werden und von LLMs aufgerufen werden können, um Aktionen auszuführen oder Berechnungen durchzuführen.3 Im Gegensatz zu Ressourcen wird von Werkzeugen erwartet, dass sie Seiteneffekte haben können. Sie sind vergleichbar mit POST-Endpunkten in einer REST-API.6
Aufforderungen (Prompts): Definieren wiederverwendbare Interaktionsmuster oder Vorlagen für LLM-Interaktionen, die Systemanweisungen, erforderliche Argumente, eingebettete Ressourcen und verschiedene Inhaltstypen umfassen können.5
Benachrichtigungen (Notifications): Sind asynchrone Nachrichten, die von einem MCP-Server an einen MCP-Client gesendet werden, typischerweise um über Zustandsänderungen oder Ereignisse zu informieren, ohne dass eine direkte vorherige Anfrage vom Client erfolgte.5


JSON-RPC Basis: MCP basiert auf JSON-RPC 2.0.2 Dies impliziert ein etabliertes Nachrichtenformat für Anfragen (Requests), Antworten (Responses) und Benachrichtigungen (Notifications), was die Implementierung und Interoperabilität erleichtert.



2.2. MCP-Architektur im NovaDE-Projekt

Identifizierung von MCP-Komponenten:

MCP Hosts in NovaDE: Es ist zu definieren, welche Komponenten des NovaDE-Projekts als MCP Hosts agieren werden. Dies könnte beispielsweise ein zentraler KI-Agenten-Orchestrator sein, der die Interaktionen zwischen verschiedenen KI-Modellen und den NovaDE MCP-Servern koordiniert und Sicherheitsrichtlinien durchsetzt, wie in 2 beschrieben.
MCP Server in NovaDE: Module oder Subsysteme von NovaDE, die spezifische Domänenfunktionalitäten oder Datenzugriffe bereitstellen, werden als MCP-Server implementiert. Diese Server exponieren dann über MCP definierte Ressourcen und Werkzeuge.
MCP Clients in NovaDE: Potenzielle MCP-Clients können interne KI-Agenten des NovaDE-Projekts sein oder auch externe KI-Modelle, die mit den Funktionalitäten von NovaDE interagieren sollen.


Transportmechanismen:

Stdio (Standard Input/Output): Dieser Mechanismus eignet sich für die lokale Interprozesskommunikation zwischen eng gekoppelten Komponenten innerhalb von NovaDE.3 Rust SDKs wie mcp_client_rs 7 und mcpr 10 unterstützen Stdio. Für Szenarien, in denen ein NovaDE-Host einen lokalen MCP-Server als Subprozess startet, ist Stdio eine einfache und effiziente Wahl.
HTTP/SSE (Server-Sent Events): Für die Kommunikation mit entfernten MCP-Servern oder wenn Echtzeit-Updates vom Server zum Client erforderlich sind (z.B. Benachrichtigungen über Änderungen in der Domänenschicht), ist HTTP mit SSE der empfohlene Transportmechanismus.3 Das mcpr Rust SDK 10 bietet explizite Unterstützung für SSE, einschließlich Mock-Implementierungen für Tests. Auch mcp-go unterstützt SSE.6 Die Fähigkeit, Server-Push-Benachrichtigungen zu empfangen, ist für viele KI-Anwendungen entscheidend, was SSE favorisiert.
Rationale für die Wahl: Die Auswahl des Transportmechanismus pro Komponente in NovaDE sollte auf den spezifischen Anforderungen basieren. Für eng integrierte lokale Prozesse, die keine unidirektionalen Echtzeit-Updates vom Server benötigen, kann Stdio ausreichend sein. Für alle Szenarien, die Server-Push-Benachrichtigungen oder die Anbindung externer/entfernter MCP-Server erfordern, sollte HTTP/SSE verwendet werden. Die "Domänenschicht-Spezifikation" muss analysiert werden, um festzustellen, welche Funktionalitäten asynchrone Updates erfordern, was die Wahl des Transports und potenziell des MCP-Server-SDKs für diese Teile leitet.


Datenflussdiagramme:

Diagramm 2.2.1: Allgemeiner MCP-Datenfluss in NovaDE (Illustriert einen NovaDE MCP Host, der mit einem internen NovaDE MCP Server und einem externen KI-Modell (Client) kommuniziert.)
Diagramm 2.2.2: Datenfluss für Werkzeugaufruf über Stdio
Diagramm 2.2.3: Datenfluss für Ressourcenabruf und Benachrichtigung über SSE



Die Unterscheidung zwischen Client, Host und Server im MCP-Modell 2 erfordert eine sorgfältige Zuweisung dieser Rollen innerhalb der NovaDE-Architektur. Der Host als Koordinator und Durchsetzer von Sicherheitsrichtlinien ist eine zentrale Komponente, insbesondere wenn mehrere KI-Agenten oder Clients mit verschiedenen NovaDE MCP-Servern interagieren. Das Design dieser Host-Komponente(n) wird entscheidend für die Sicherheit und Verwaltbarkeit des Gesamtsystems sein.

3. Standard-MCP-Nachrichtenspezifikationen für NovaDEDieser Abschnitt definiert die präzisen JSON-RPC 2.0 Strukturen für alle Standard-MCP-Nachrichten, angepasst mit NovaDE-spezifischen Überlegungen, wie beispielsweise gemeinsamen Metadatenfeldern. Die hier definierten Strukturen basieren auf den allgemeinen MCP-Konzepten 3 und werden durch spezifische Felder für den NovaDE-Kontext erweitert.

3.1. Initialize Request und ResponseDie Initialize-Nachricht dient dem Aufbau einer Verbindung und dem Aushandeln von Protokollversionen und Fähigkeiten zwischen Client und Server.3

InitializeParams: Parameter für den Initialize-Request.

Tabelle 3.1: InitializeParams Schema




FeldnameJSON-TypBeschreibungConstraintsprotocolVersionstringDie vom Client vorgeschlagene MCP-Protokollversion (z.B. "2025-03-26").ErforderlichclientNamestringOptionaler, menschenlesbarer Name der Client-Anwendung/Komponente.OptionalclientVersionstringOptionale Version der Client-Anwendung/Komponente.OptionalsupportedFeaturesarray of stringOptionale Liste von NovaDE-spezifischen MCP-Features, die der Client unterstützt.Optional*   **`InitializeResult`**: Ergebnis eines erfolgreichen Initialize-Requests.
    *   **Tabelle 3.2**: `InitializeResult` Schema
FeldnameJSON-TypBeschreibungConstraintsprotocolVersionstringDie vom Server gewählte und unterstützte MCP-Protokollversion.ErforderlichserverNamestringOptionaler, menschenlesbarer Name der Server-Anwendung/Komponente.OptionalserverVersionstringOptionale Version der Server-Anwendung/Komponente.OptionalsupportedFeaturesarray of stringOptionale Liste von NovaDE-spezifischen MCP-Features, die der Server unterstützt.Optionaltoolsarray of ToolDefinitionOptionale initiale Liste der vom Server bereitgestellten Werkzeuge.Optional, siehe Tabelle 3.8 für ToolDefinitionresourcesarray of ResourceDefinitionOptionale initiale Liste der vom Server bereitgestellten Ressourcen.Optional, Struktur analog zu Resource (Tabelle 3.5) aber ggf. ohne content    *Referenzen*: Die `mcp_client_rs` Bibliothek nutzt eine `spawn_and_initialize` Methode [9], und `mcpr` bietet eine `client.initialize()` Funktion [10], was die fundamentale Rolle dieser Nachricht unterstreicht.


3.2. ListResources Request und ResponseDiese Nachricht ermöglicht es einem Client, die vom Server verfügbaren Ressourcen abzufragen.3

ListResourcesParams: Parameter für den ListResources-Request.

Tabelle 3.3: ListResourcesParams Schema




FeldnameJSON-TypBeschreibungConstraintsfilterobjectOptionale, NovaDE-spezifische Kriterien zur Filterung der Ressourcen (z.B. nach Typ, Domänenentität).OptionalpageTokenstringOptionales Token zur Paginierung, um die nächste Seite der Ergebnisse abzurufen.Optional*   **`ListResourcesResult`**: Ergebnis eines erfolgreichen ListResources-Requests.
    *   **Tabelle 3.4**: `ListResourcesResult` Schema
FeldnameJSON-TypBeschreibungConstraintsresourcesarray of ResourceListe der Resource-Objekte, die den Filterkriterien entsprechen.Erforderlich, siehe Tabelle 3.5 für ResourcenextPageTokenstringOptionales Token, um die nächste Seite der Ergebnisse abzurufen, falls vorhanden.Optional*   **`Resource` Objektstruktur**: Definiert die Struktur einer einzelnen Ressource.
    *   **Tabelle 3.5**: `Resource` Objekt Schema
FeldnameJSON-TypBeschreibungConstraintsDomänenschicht-Mapping (Beispiel)uristringEindeutiger Resource Identifier (URI).ErforderlichDomainObject.IDnamestringMenschenlesbarer Name der Ressource.ErforderlichDomainObject.DisplayNamedescriptionstringOptionale, detaillierte Beschreibung der Ressource.OptionalDomainObject.DescriptionschemaobjectOptionales JSON-Schema, das die Datenstruktur des Ressourceninhalts beschreibt.Optional-novaDE_domain_typestringOptionaler Typbezeichner, der auf einen Typ in der "Domänenschicht-Spezifikation" verweist.OptionalName des DomänentypscontentTypestringOptionaler MIME-Typ oder NovaDE-spezifischer Inhaltstyp.OptionalDomainObject.MimeTypecanReadbooleanGibt an, ob die Ressource gelesen werden kann.Optional-canWritebooleanGibt an, ob die Ressource geschrieben werden kann (falls zutreffend).Optional-    *Referenzen*: Die `list_resources()` Methode in `mcp_client_rs` [9] und das allgemeine Konzept von Ressourcen in MCP [6] sind hier relevant.


3.3. CallTool Request und ResponseDiese Nachricht ermöglicht es einem Client, ein vom Server bereitgestelltes Werkzeug auszuführen.3

CallToolParams: Parameter für den CallTool-Request.

Tabelle 3.6: CallToolParams Schema




FeldnameJSON-TypBeschreibungConstraintstoolNamestringName des aufzurufenden Werkzeugs.ErforderlichargumentsobjectJSON-Objekt, das die Argumente für das Werkzeug enthält.ErforderlichprogressTokenstringOptionales Token zur Verfolgung des Fortschritts langlaufender Werkzeuge.Optional*   **`CallToolResult`**: Ergebnis eines erfolgreichen CallTool-Requests.
    *   **Tabelle 3.7**: `CallToolResult` Schema
FeldnameJSON-TypBeschreibungConstraintsresultanyOptionale Ausgabe der Werkzeugausführung. Die Struktur hängt vom Werkzeug ab.OptionalisErrorbooleanOptional. Gibt an, ob der Werkzeugaufruf zu einem anwendungsspezifischen Fehler geführt hat (Standard: false).Optional, Default falseerrorobjectOptionale, werkzeugspezifische Fehlerdetails, falls isError true ist.Optional*   **`ToolDefinition` Objektstruktur**: Definiert die Struktur eines Werkzeugs (verwendet in `InitializeResult` und potenziell in einer `ListTools`-Antwort).
    *   **Tabelle 3.8**: `ToolDefinition` Objekt Schema
FeldnameJSON-TypBeschreibungConstraintsDomänenschicht-Mapping (Beispiel)namestringEindeutiger Name des Werkzeugs.ErforderlichDomainFunction.NamedescriptionstringOptionale, menschenlesbare Beschreibung des Werkzeugs.OptionalDomainFunction.DocparametersSchemaobjectJSON-Schema, das die Eingabeparameter (arguments) des Werkzeugs beschreibt.Erforderlich-resultSchemaobjectOptionales JSON-Schema, das die erfolgreiche Ausgabe (result) des Werkzeugs beschreibt.Optional-novaDE_domain_functionstringOptionaler Bezeichner, der auf eine Funktion/Fähigkeit in der "Domänenschicht-Spezifikation" verweist.OptionalName der Domänenfunktion    *Referenzen*: Die `call_tool()` Methode in `mcp_client_rs` [9] und das Werkzeugkonzept in MCP [3, 6] sind hier relevant.


3.4. ReadResource Request und ResponseErmöglicht das Lesen des Inhalts einer spezifischen Ressource.

ReadResourceParams: Parameter für den ReadResource-Request.

Tabelle 3.9: ReadResourceParams Schema




FeldnameJSON-TypBeschreibungConstraintsuristringURI der zu lesenden Ressource.Erforderlich*   **`ReadResourceResult`**: Ergebnis eines erfolgreichen ReadResource-Requests.
    *   **Tabelle 3.10**: `ReadResourceResult` Schema
FeldnameJSON-TypBeschreibungConstraintscontentanyDer Inhalt der Ressource, konform zu ihrem Schema (falls definiert).ErforderlichcontentTypestringOptionaler MIME-Typ oder NovaDE-spezifischer Inhaltstyp der Ressource.Optional    *Referenzen*: Die `read_resource()` Methode in `mcp_client_rs`.[9]


3.5. Notification MessageAsynchrone Nachricht vom Server an den Client.5

Notification Struktur:

Tabelle 3.11: Generische Notification Struktur




FeldnameJSON-TypBeschreibungConstraintsjsonrpcstringMuss "2.0" sein.ErforderlichmethodstringName der Benachrichtigungsmethode (z.B. novaDE/resourceUpdated, novaDE/statusChanged).ErforderlichparamsobjectOptionales JSON-Objekt mit den Parametern der Benachrichtigung. Das Schema hängt von method ab.Optional    *Referenzen*: Die Notwendigkeit der Handhabung von Server-Push-Benachrichtigungen wird durch die SSE-Unterstützung in `mcpr` [10, 11, 12] und die Erwähnung in MCP-Konzepten [5] deutlich.


3.6. Response Message (Erfolg)Standard-JSON-RPC-Erfolgsantwort.

Response Struktur:

jsonrpc (string, required): "2.0".
id (string | number | null, required): Muss mit der ID der ursprünglichen Anfrage übereinstimmen.
result (any, required): Das Ergebnis der Anfrage, dessen Struktur vom jeweiligen Request-Typ abhängt (z.B. InitializeResult, ListResourcesResult).





3.7. ErrorResponse Message (Protokollfehler)Standard-JSON-RPC-Fehlerantwort.5

ErrorResponse Struktur:

Tabelle 3.12: Generische ErrorResponse Struktur




FeldnameJSON-TypBeschreibungConstraintsjsonrpcstringMuss "2.0" sein.Erforderlichidstring \number \nullerrorobjectEin Objekt, das den Fehler beschreibt.Erforderlicherror.codeintegerNumerischer Fehlercode.Erforderlicherror.messagestringMenschenlesbare Fehlerbeschreibung.Erforderlicherror.dataanyOptionale, zusätzliche Fehlerdetails.OptionalDie Standard-MCP-Nachrichten bilden ein robustes Fundament. Für NovaDE wird die Hauptaufgabe darin bestehen, spezifische Schemata für `Resource`-Inhalte, `ToolDefinition.parametersSchema`, `ToolDefinition.resultSchema` und `Notification.params` zu definieren, die auf der Domänenschicht des Projekts basieren. Die Verwendung von Rust-SDKs wie `mcp_client_rs` [7, 8, 9] unterstreicht die Bedeutung typsicherer Methoden für Kernanfragen, was wiederum voraussetzt, dass Serverantworten strikt den definierten Schemata entsprechen, um eine erfolgreiche Deserialisierung zu gewährleisten. Dies macht eine rigorose Schemavalidierung sowohl auf Client- als auch auf Serverseite unerlässlich für eine robuste Kommunikation.
4. NovaDE Domänenschicht-IntegrationsstrategieDie erfolgreiche Integration des Model Context Protocol (MCP) in das NovaDE-Projekt hängt entscheidend von einer klaren Strategie zur Abbildung der NovaDE-Domänenschicht auf MCP-Schnittstellen ab. Dieser Abschnitt legt die Methodik und Prinzipien für diese Abbildung fest und diskutiert, wie domänenspezifische Technologien, insbesondere im Kontext eines Desktop-Environments, über MCP zugänglich gemacht werden können. Da die detaillierte "Domänenschicht-Spezifikation" für NovaDE zum Zeitpunkt der Erstellung dieses Dokuments nicht vorliegt, dient dieser Abschnitt als Rahmenwerk und illustriert die Integrationsansätze beispielhaft.

4.1. Methodik zur Abbildung

Prinzipien: Der Prozess der Abbildung der "Domänenschicht-Spezifikation" auf MCP-Konstrukte erfordert eine systematische Analyse. Zunächst müssen die Kernentitäten, -funktionalitäten und -ereignisse der Domänenschicht identifiziert werden. Diese werden dann den entsprechenden MCP-Konzepten – Ressourcen (Resources), Werkzeuge (Tools) und Benachrichtigungen (Notifications) – zugeordnet. Es ist essenziell, dass diese Abbildung die Semantik der Domänenschicht korrekt widerspiegelt und gleichzeitig eine für KI-Agenten verständliche und nutzbare Schnittstelle schafft.
Granularität: Die Wahl der Granularität für MCP-Schnittstellen ist eine wichtige Designentscheidung. Es muss abgewogen werden, ob ein MCP-Server viele feingranulare Werkzeuge und Ressourcen exponiert, die spezifische, kleine Aufgaben abbilden, oder ob weniger, dafür aber grobgranularere Schnittstellen angeboten werden, die komplexere Operationen kapseln. Die optimale Granularität hängt von der Natur der NovaDE-Domänenschicht und den erwarteten Anwendungsfällen der interagierenden KI-Modelle ab. Feingranulare Schnittstellen bieten mehr Flexibilität, können aber zu komplexeren Interaktionsmustern führen, während grobgranulare Schnittstellen die Komplexität für den Client reduzieren, aber möglicherweise weniger flexibel sind.
Abstraktion vs. Direkte Abbildung: MCP ist als universeller Adapter konzipiert 3, was darauf hindeutet, dass es oft als eine Abstraktionsebene über darunterliegenden Systemen dient. Es muss entschieden werden, ob die MCP-Schnittstellen eine direkte Eins-zu-Eins-Abbildung von Funktionen der Domänenschicht darstellen oder ob sie eine höhere Abstraktionsebene bieten, die möglicherweise mehrere Domänenfunktionen zu einem kohärenten MCP-Werkzeug oder einer Ressource zusammenfasst. Eine Abstraktion kann die Komplexität für KI-Agenten reduzieren und die Schnittstelle stabiler gegenüber Änderungen in der Domänenschicht machen. Die Domänenschicht-Spezifikation ist hier der entscheidende Faktor.



4.2. Datenmodellierung für MCP-Schnittstellen

Namenskonventionen: Es müssen klare und konsistente Namenskonventionen für MCP-Ressourcen und -Werkzeuge definiert werden, die sich idealerweise an den Bezeichnern der entsprechenden Entitäten und Funktionen in der NovaDE-Domänenschicht orientieren. Dies fördert die Verständlichkeit und Wartbarkeit.
JSON-Schema-Richtlinien: Für die Inhalte von Ressourcen sowie für die Parameter und Ergebnisse von Werkzeugen müssen JSON-Schemata erstellt werden. Es sind Richtlinien für die Erstellung dieser Schemata festzulegen, um Konsistenz über alle NovaDE-MCP-Schnittstellen hinweg zu gewährleisten. Dies beinhaltet die Verwendung standardisierter Datentypen, Formatierungen und Validierungsregeln.
Datentransformation: Es ist zu analysieren, ob und welche Datentransformationen zwischen den Datenformaten der Domänenschicht und den MCP-Nachrichten-Payloads erforderlich sind. Diese Transformationen müssen klar definiert und implementiert werden, um eine korrekte Datenübertragung sicherzustellen.



4.3. Potenzielle Integrationspunkte mit Desktop-Technologien (Beispielhaft)Dieser Unterabschnitt dient als Illustration, wie domänenspezifische Technologien, die typischerweise in einem Desktop-Environment wie NovaDE vorkommen könnten, über MCP integriert werden könnten. Die konkreten Integrationspunkte hängen vollständig von der tatsächlichen "Domänenschicht-Spezifikation" von NovaDE ab.


D-Bus-Dienste: Viele Desktop-Umgebungen nutzen D-Bus für die Interprozesskommunikation und den Zugriff auf Systemdienste. Wenn die Domänenschicht von NovaDE Interaktionen mit solchen Diensten vorsieht, könnten MCP-Schnittstellen als Abstraktion dienen:

MCP-Werkzeuge (Tools) könnten D-Bus-Methodenaufrufe kapseln. Beispielsweise könnte ein Werkzeug novade/notifications/sendDesktopNotification die Methode Notify des org.freedesktop.Notifications D-Bus-Dienstes aufrufen.13 Ähnlich könnten Werkzeuge für die Interaktion mit org.freedesktop.secrets (z.B. zum Speichern oder Abrufen von Passwörtern 15), org.freedesktop.login1 (z.B. zum Sperren der Sitzung oder Abfragen von Benutzerinformationen 17) oder org.freedesktop.UPower (z.B. zum Abfragen des Batteriestatus 21) definiert werden.
MCP-Ressourcen (Resources) könnten abfragbare D-Bus-Eigenschaften oder den Zustand von D-Bus-Objekten repräsentieren. Beispielsweise könnte eine Ressource novade://power/status die Eigenschaften des org.freedesktop.UPower.Device exponieren.
MCP-Benachrichtigungen (Notifications) könnten D-Bus-Signale an MCP-Clients weiterleiten. Ein SessionLock-Signal von org.freedesktop.login1 könnte eine MCP-Benachrichtigung auslösen.
Zur Implementierung solcher MCP-Server in Rust, die mit D-Bus interagieren, ist die zbus-Bibliothek ein geeignetes Werkzeug.26



Wayland-Protokolle: Wenn NovaDE ein Wayland-Compositor ist oder tiefgreifend mit Wayland-basierten Funktionen der Domänenschicht interagiert, könnten MCP-Schnittstellen diese komplexen Protokolle abstrahieren:

MCP-Werkzeuge (Tools) könnten Aktionen wie Fensterverwaltung (Fokus setzen, Schließen, Größenänderung basierend auf xdg-shell 42), das Erstellen von Screenshots (möglicherweise über xdg-desktop-portal oder direktere Wayland-Protokolle wie wlr-screencopy-v1 falls NovaDE ein wlroots-basierter Compositor ist), oder die Synthese von Eingabeereignissen bereitstellen. Die Integration mit wlr-layer-shell 43 für Oberflächen wie Panels oder Hintergrundbilder könnte ebenfalls über MCP-Werkzeuge gesteuert werden.
MCP-Ressourcen (Resources) könnten den Zustand von Fenstern, Ausgabegeräten (Monitoren) oder Eingabegeräten repräsentieren.
Die Smithay-Bibliothek ist ein Rust-Framework, das Bausteine für Wayland-Compositoren bereitstellt und Handler für viele Wayland-Protokolle enthält.



PipeWire: Wenn die Domänenschicht von NovaDE Multimedia-Aspekte umfasst, könnten MCP-Werkzeuge PipeWire-Knoten (Sinks, Sources, Filter) für Lautstärke, Routing usw. steuern.44 MCP-Ressourcen könnten PipeWire-Objekteigenschaften darstellen. Die pipewire-rs-Bibliothek 47 bietet Rust-Bindings für PipeWire. Beispiele zeigen, wie Knoten aufgelistet 59 und Parameter wie Lautstärke gesetzt werden können.47


XDG Desktop Portals: Wenn NovaDE-Anwendungen sandboxed sind oder benutzervermittelten Zugriff auf Ressourcen (Dateien, Screenshots) benötigen, können MCP-Werkzeuge Aufrufe an XDG Desktop Portals kapseln.87 Die Schnittstellen org.freedesktop.portal.FileChooser 98 und org.freedesktop.portal.Screenshot 91 sind wohldefiniert. Rust-Crates wie xdg-portal 105 oder direkte zbus-Aufrufe können hierfür verwendet werden.


Die "Domänenschicht-Spezifikation" ist der kritischste Input für die Definition konkreter MCP-Schnittstellen. Die obigen Beispiele sind potenzielle Integrationspunkte, falls NovaDE ein Desktop-Environment ist. Die tatsächliche Domäne wird die Spezifika diktieren. Diese Spezifikation muss daher flexibel bleiben. Die Abstraktion komplexer Protokolle über einfachere MCP-Schnittstellen kann die Hürde für KI-Agenten zur Interaktion mit NovaDE signifikant senken, da Wayland 43 und D-Bus 15 komplexe APIs haben, während MCP eine standardisierte und potenziell einfachere Schnittstelle für KI anstrebt.1 Das Design der MCP-Schnittstellen sollte sich daher auf Anwendungsfälle konzentrieren, die für die KI-Interaktion relevant sind, und nicht notwendigerweise jede Nuance der zugrundeliegenden Domänenschicht-APIs exponieren.

5. NovaDE-spezifische MCP-SchnittstellendefinitionenDieser Abschnitt dient als Katalog der MCP-Server-Schnittstellen, die spezifisch für das NovaDE-Projekt entwickelt werden. Jede hier definierte Schnittstellengruppe repräsentiert eine logische Sammlung von Funktionalitäten innerhalb von NovaDE. Der Inhalt dieses Abschnitts ist als Vorlage zu verstehen und muss basierend auf der detaillierten "Domänenschicht-Spezifikation" des NovaDE-Projekts konkretisiert werden. Die Struktur orientiert sich an den Kernkonzepten von MCP (Ressourcen, Werkzeuge, Aufforderungen, Benachrichtigungen) 5, um sicherzustellen, dass alle NovaDE-spezifischen Erweiterungen auf dem Standard-MCP-Framework aufbauen.(Vorlagenstruktur - zu füllen basierend auf der Domänenschicht-Spezifikation)
5.1. Interface-Gruppe: de.nova.projekt.Kernfunktionalitaeten

Übersicht: Diese Schnittstellengruppe umfasst grundlegende Funktionalitäten des NovaDE-Kerns, die für KI-Agenten relevant sind, wie z.B. Systeminformationen oder grundlegende Konfigurationsaspekte.
Tabelle 5.1.1: MCP-Schnittstellen in Gruppe Kernfunktionalitaeten


Schnittstellen-ID (Interface ID)ZweckServer-Komponente (NovaDE-Modul)de.nova.mcp.core.systemInfoBereitstellung von SysteminformationenNovaDE.Core.SystemMonitorde.nova.mcp.core.userPreferencesZugriff auf BenutzereinstellungenNovaDE.Core.SettingsManager*   **5.1.1 Schnittstelle: `de.nova.mcp.core.systemInfo`**
    *   **Version**: `1.0.0`
    *   **Beschreibung**: Stellt Informationen über das NovaDE-System und die zugrundeliegende Hardware/Software-Umgebung bereit.
    *   **Abhängigkeiten**: Abschnitt X.Y der "Domänenschicht-Spezifikation" (Systeminformationen).
    *   **5.1.1.1 Ressourcen (Resources)**
        *   **Name**: `SystemStatus`
        *   **URI-Struktur**: `novade://core/system/status`
        *   **Tabelle 5.1.1.1.A**: Ressourcenschema für `SystemStatus`
FeldnameJSON-TypBeschreibungConstraintsDomänenschicht-EntitätosVersionstringVersion des BetriebssystemsErforderlichSystem.OS.VersionnovaDEVersionstringVersion von NovaDEErforderlichNovaDE.VersioncpuUsagenumberAktuelle CPU-Auslastung (Prozent)OptionalSystem.CPU.CurrentLoadmemoryUsageobjectInformationen zur SpeichernutzungOptionalSystem.Memory.StatsmemoryUsage.totalintegerGesamtspeicher in MBOptionalSystem.Memory.TotalmemoryUsage.availableintegerVerfügbarer Speicher in MBOptionalSystem.Memory.Available        *   **Unterstützte Operationen**: `ReadResource`.
        *   **Zugriffssteuerung**: Nur authentifizierte Systemagenten.

    *   **5.1.1.2 Werkzeuge (Tools)**: Keine für diese spezifische Schnittstelle definiert.
    *   **5.1.1.3 Aufforderungen (Prompts)**: Keine für diese spezifische Schnittstelle definiert.
    *   **5.1.1.4 Benachrichtigungen (Notifications)**
        *   **Name**: `systemLoadWarning`
        *   **Auslösebedingungen**: Wird gesendet, wenn die CPU-Auslastung für einen bestimmten Zeitraum einen Schwellenwert überschreitet.
        *   **Tabelle 5.1.1.4.A**: Payload-Schema für `systemLoadWarning`
FeldnameJSON-TypBeschreibungDomänenschicht-EreignisdatenlevelstringWarnstufe (HIGH, CRITICAL)SystemAlert.LevelcpuUsagenumberAktuelle CPU-Auslastung zum Zeitpunkt des AlarmsSystemAlert.CPULoad
5.2. Interface-Gruppe: de.nova.projekt.DesktopIntegration (Beispiel für D-Bus/Wayland)

Übersicht: Diese Schnittstellengruppe demonstriert, wie Desktop-spezifische Funktionalitäten, die typischerweise über D-Bus oder Wayland-Protokolle bereitgestellt werden, über MCP abstrahiert werden können.
Tabelle 5.2.1: MCP-Schnittstellen in Gruppe DesktopIntegration


Schnittstellen-ID (Interface ID)ZweckServer-Komponente (NovaDE-Modul)de.nova.mcp.desktop.notificationsSenden und Verwalten von Desktop-BenachrichtigungenNovaDE.NotificationServiceWrapperde.nova.mcp.desktop.secretsSicherer Speicher für GeheimnisseNovaDE.SecretsAgentde.nova.mcp.desktop.powerAbfragen und Steuern von EnergieoptionenNovaDE.PowerManagerWrapperde.nova.mcp.desktop.sessionVerwalten von BenutzersitzungenNovaDE.SessionManagerWrapperde.nova.mcp.desktop.fileChooserÖffnen von DateiauswahldialogenNovaDE.FileChooserPortalWrapperde.nova.mcp.desktop.screenshotErstellen von BildschirmfotosNovaDE.ScreenshotPortalWrapper*   **5.2.1 Schnittstelle: `de.nova.mcp.desktop.notifications`**
    *   **Version**: `1.0.0`
    *   **Beschreibung**: Ermöglicht das Senden von Desktop-Benachrichtigungen und das Abfragen von Server-Fähigkeiten, basierend auf `org.freedesktop.Notifications`.
    *   **Abhängigkeiten**: `org.freedesktop.Notifications` D-Bus Spezifikation.[13, 14]
    *   **5.2.1.1 Ressourcen (Resources)**: Keine direkt, Status wird über Werkzeuge/Benachrichtigungen gehandhabt.
    *   **5.2.1.2 Werkzeuge (Tools)**
        *   **Name**: `sendNotification`
        *   **Beschreibung**: Sendet eine Desktop-Benachrichtigung.
        *   **Tabelle 5.2.1.2.A**: Eingabeparameter für `sendNotification` (abgeleitet von `org.freedesktop.Notifications.Notify` [14])
ParameternameJSON-TypBeschreibungErforderlichDomänenschicht-Parameter (D-Bus)appNamestringName der Anwendung, die die Benachrichtigung sendet.Neinapp_name (STRING)replacesIdintegerID einer zu ersetzenden Benachrichtigung (0 für neu).Neinreplaces_id (UINT32)appIconstringPfad oder Name des Anwendungsicons.Neinapp_icon (STRING)summarystringZusammenfassung der Benachrichtigung.Jasummary (STRING)bodystringDetaillierter Text der Benachrichtigung.Neinbody (STRING)actionsarray of stringListe von Aktions-IDs und deren Beschriftungen (alternierend).Neinactions (as)hintsobjectZusätzliche Hinweise für den Server (z.B. Dringlichkeit).Neinhints (a{sv})expireTimeoutintegerTimeout in Millisekunden (-1 für Server-Default).Neinexpire_timeout (INT32)        *   **Tabelle 5.2.1.2.B**: Ausgabeparameter für `sendNotification`
ParameternameJSON-TypBeschreibungDomänenschicht-Rückgabe (D-Bus)notificationIdintegerEindeutige ID der Benachrichtigung.id (UINT32)        *   **Name**: `getNotificationCapabilities`
        *   **Beschreibung**: Frägt die Fähigkeiten des Benachrichtigungsservers ab.
        *   **Tabelle 5.2.1.2.C**: Ausgabeparameter für `getNotificationCapabilities` (abgeleitet von `org.freedesktop.Notifications.GetCapabilities` [14])
ParameternameJSON-TypBeschreibungDomänenschicht-Rückgabe (D-Bus)capabilitiesarray of stringListe der unterstützten Server-Fähigkeiten.capabilities (as)    *   **5.2.1.3 Benachrichtigungen (Notifications)**
        *   **Name**: `notificationClosed` (entspricht `org.freedesktop.Notifications.NotificationClosed` [14])
        *   **Tabelle 5.2.1.3.A**: Payload-Schema für `notificationClosed`
FeldnameJSON-TypBeschreibungDomänenschicht-Ereignisdaten (D-Bus)idintegerID der geschlossenen Benachrichtigung.id (UINT32)reasonintegerGrund für das Schließen (1=expired, 2=dismissed, 3=closed by call).reason (UINT32)*   *(Weitere Schnittstellen wie `de.nova.mcp.desktop.secrets`, `de.nova.mcp.desktop.power` etc. würden analog unter Verwendung der relevanten D-Bus Spezifikationen [15, 17, 21] und XDG Portal Spezifikationen [98, 100] detailliert werden.)*
Die explizite Abbildung auf Entitäten, Funktionen und Ereignisse der "Domänenschicht" in den Tabellen ist entscheidend, um die Nachvollziehbarkeit zu gewährleisten und zu verdeutlichen, wie die MCP-Schnittstellen mit dem zugrundeliegenden NovaDE-System zusammenhängen. Dies ist eine direkte Anforderung der Nutzeranfrage. Die Konsistenz zwischen dieser MCP-Spezifikation und der "Domänenschicht-Spezifikation" muss während der gesamten Entwicklung von NovaDE aufrechterhalten werden. Dieser Abschnitt wird der umfangreichste und detaillierteste sein und erfordert eine sorgfältige Definition von Schemata und Verhaltensweisen für jedes domänenspezifische MCP-Element, sobald die Domänenschicht-Spezifikation verfügbar ist.6. Implementierungsaspekte für NovaDEDieser Abschnitt behandelt empfohlene Technologien und Muster für die Implementierung von MCP-Clients und -Servern im NovaDE-Projekt, mit besonderem Fokus auf die Handhabung von Asynchronität und Verbindungsmanagement.

6.1. Empfohlene SDKs und BibliothekenDie Wahl der SDKs und Bibliotheken hängt von der jeweiligen Komponente und deren Anforderungen ab, insbesondere bezüglich des Transportmechanismus.

Rust:

Server-Implementierung: Für MCP-Server, die Server-Sent Events (SSE) für Benachrichtigungen nutzen müssen, wird das mcpr Crate empfohlen.10 Es bietet High-Level-Abstraktionen für Server, Werkzeuge und unterstützt verschiedene Transportmechanismen, einschließlich Stdio und SSE. Die Fähigkeit, Server-Push-Benachrichtigungen zu senden, ist für viele KI-Anwendungen kritisch, was mcpr favorisiert.
Client-Implementierung:

Das mcpr Crate 10 ist ebenfalls eine gute Wahl für Rust-basierte MCP-Clients, insbesondere wenn SSE-basierte Benachrichtigungen empfangen werden müssen. Es bietet eine konsistente API für Client- und Server-Entwicklung.
Das mcp_client_rs Crate von Darin Kishore 7 (basierend auf einer früheren Version von Derek-X-Wang/mcp-rust-sdk 109) ist eine weitere Option, primär für Stdio-basierte Kommunikation. Die Dokumentation ist jedoch weniger explizit bezüglich der Handhabung von asynchronen Server-Push-Benachrichtigungen über Stdio 9, was für reaktive Agenten ein Nachteil sein könnte. Die Unterstützung für WebSocket-Transport mit Wiederverbindungshandhabung ist zwar erwähnt, aber als "Coming Soon" markiert.109




Go: Für Komponenten des NovaDE-Projekts, die in Go implementiert werden, stellt mcp-go 6 eine valide Option dar. Dieses SDK unterstützt ebenfalls Stdio und bietet Abstraktionen für Server, Werkzeuge und Ressourcen.
Andere Sprachen: Da MCP auf JSON-RPC 2.0 basiert, können Clients und Server prinzipiell in jeder Sprache implementiert werden, die JSON-Verarbeitung und den gewählten Transportmechanismus (Stdio oder HTTP/SSE) unterstützt.



6.2. Handhabung von asynchronen Server-Sent NotificationsAsynchrone Benachrichtigungen vom Server zum Client sind ein Kernmerkmal von MCP, um KI-Agenten über Zustandsänderungen oder Ereignisse in der Domänenschicht zu informieren.3

Client-seitig:

Clients, die auf Server-Push-Benachrichtigungen reagieren müssen, sollten den SSE-Transportmechanismus verwenden. Das mcpr Crate in Rust bietet hierfür geeignete Abstraktionen, um einen SSE-Stream zu abonnieren und die eingehenden Nachrichten zu verarbeiten.10 Dies beinhaltet das Parsen der JSON-RPC-Benachrichtigungen und das Weiterleiten der params-Nutzlast an die zuständige Anwendungslogik.
Beispiele für MCP-Server, die Benachrichtigungen verwenden, wie der MCP Notify Server 122 oder die in mcp-go beschriebene Fähigkeit, Benachrichtigungen an spezifische Clients zu senden 6, unterstreichen die Wichtigkeit dieses Musters.
Für Stdio-Transporte ist die Handhabung von Server-Push-Benachrichtigungen komplexer, da Stdio primär für Request-Response-Interaktionen ausgelegt ist. mcp_client_rs müsste hierfür einen dedizierten Lesethread oder eine asynchrone Lese-Schleife implementieren, die kontinuierlich stdout des Servers auf neue Nachrichten überwacht und diese dann als Benachrichtigungen interpretiert.118 Die Dokumentation von mcp_client_rs ist hierzu nicht explizit.


Server-seitig:

NovaDE MCP-Server, die Benachrichtigungen senden müssen, sollten bei Verwendung von SSE die etablierten Mechanismen des gewählten Frameworks (z.B. mcpr in Rust oder FastAPI mit SSE-Support in Python 12) nutzen, um Nachrichten an alle oder ausgewählte verbundene Clients zu pushen.
Bei Stdio-Transport müssen Benachrichtigungen als reguläre JSON-RPC-Nachrichten auf stdout geschrieben werden, wobei der Client für das korrekte Parsen und Unterscheiden von regulären Antworten zuständig ist.





6.3. Behandlung von Verbindungsstatus-EreignissenEine robuste Behandlung von Verbindungsstatus ist essentiell für die Zuverlässigkeit.

Client-seitig:

Clients müssen Mechanismen zur Erkennung von Verbindungsabbrüchen implementieren. Dies kann durch Timeouts bei Requests, Fehler beim Lesen/Schreiben auf den Transportkanal oder spezifische Fehlermeldungen des Transport-SDKs geschehen.
Strategien für automatische Wiederverbindungsversuche sollten implementiert werden, idealerweise mit exponentiellem Backoff, um Server nicht zu überlasten.
Der mcpr-Client erwähnt die Handhabung von Prozessbeendigung und Pipe-Verbindungsproblemen bei Stdio.10 Die (geplante) WebSocket-Unterstützung in mcp_client_rs erwähnt "built-in reconnection handling".109
Allgemeine Prinzipien zur Fehlerbehebung bei Netzwerkverbindungen, wie in 123 für Azure Event Grid beschrieben (Port-Blockaden, Firewall-Regeln), können auch hier relevant sein, insbesondere bei HTTP/SSE.


Server-seitig:

MCP-Server sollten Client-Verbindungen aktiv verwalten, einschließlich Logging von Verbindungsaufbau und -abbau.
Bei Stdio-basierten Servern endet der Serverprozess typischerweise, wenn der Client die Verbindung trennt.10 Für langlebige Sitzungen muss dies bedacht werden.





6.4. Zustandsbehaftetes Sitzungsmanagement (Session Management)Einige Interaktionen mit KI-Modellen erfordern möglicherweise einen Zustand, der über mehrere MCP-Requests hinweg erhalten bleibt.

Server-seitig: Wenn NovaDE-Schnittstellen zustandsbehaftete Interaktionen erfordern, müssen MCP-Server Mechanismen zum Sitzungsmanagement implementieren. Das mcp-go SDK erwähnt explizit die Unterstützung für die Verwaltung separater Zustände für jeden verbundenen Client, das Verfolgen von Client-Sitzungen und die Möglichkeit, per-session Werkzeuganpassungen vorzunehmen.6
Dies könnte die Generierung und Verwaltung von Sitzungs-IDs beinhalten, die vom Client bei nachfolgenden Anfragen mitgesendet werden, oder die Nutzung inhärenter Sitzungsmerkmale des gewählten Transports (z.B. langlebige SSE-Verbindungen).
Die Notwendigkeit und Komplexität des Sitzungsmanagements hängt stark von den spezifischen Anwendungsfällen ab, die durch die "Domänenschicht-Spezifikation" definiert werden.


Die Wahl des SDKs und die Implementierung von Benachrichtigungs- und Verbindungsmanagement sind kritisch. Für NovaDE-Komponenten, die auf Server-Push-Benachrichtigungen angewiesen sind oder eine robustere Handhabung von Remote-Verbindungen benötigen, scheint mcpr aufgrund seiner expliziten SSE-Unterstützung die passendere Wahl in Rust zu sein. Die Client-Implementierungen in NovaDE müssen eine widerstandsfähige Logik für die Verarbeitung von Benachrichtigungsströmen und die Behandlung von Verbindungsfehlern enthalten, um die Stabilität und Reaktionsfähigkeit der KI-Agenten zu gewährleisten.7. Sicherheitsmodell für NovaDE MCP-SchnittstellenDie Sicherheit der MCP-Schnittstellen ist von größter Bedeutung, da sie potenziell Zugriff auf sensible Daten und kritische Funktionalitäten des NovaDE-Projekts ermöglichen. Das Sicherheitsmodell muss Authentifizierung, Autorisierung, Datensicherheit und Benutzereinwilligung umfassen. MCP selbst legt Wert auf Sicherheit 2, aber die konkrete Ausgestaltung obliegt dem NovaDE-Projekt.

7.1. Authentifizierung und Autorisierung

Client-Authentifizierung: Es müssen Mechanismen definiert werden, wie sich MCP-Clients gegenüber NovaDE-MCP-Servern authentifizieren.

Für Stdio-basierte Kommunikation ist die Authentifizierung oft implizit durch die Prozessgrenzen und Benutzerkontexte des Betriebssystems gegeben. Zusätzliche anwendungsspezifische Token können jedoch für eine feinere Kontrolle verwendet werden.
Für HTTP/SSE-basierte Kommunikation sind explizite Authentifizierungsmechanismen erforderlich. Optionen umfassen:

Token-basierte Authentifizierung (z.B. API-Keys, JWTs), die im HTTP-Header übertragen werden.
OAuth 2.0, falls externe Clients oder Benutzer im Namen von Benutzern agieren. MCP unterstützt prinzipiell OAuth.3
Es ist zu beachten, dass die MCP-Spezifikation zum Zeitpunkt einiger Referenzdokumente möglicherweise keinen standardisierten Authentifizierungsmechanismus für SSE-Server definierte.12 Daher muss NovaDE hier ggf. eine eigene Lösung implementieren oder auf Netzwerkebene absichern (z.B. über VPN, IP-Whitelisting oder einen Reverse-Proxy, der die Authentifizierung übernimmt).




Server-Authentifizierung: Clients müssen die Identität der NovaDE-MCP-Server überprüfen können, insbesondere bei HTTP/SSE-Kommunikation. Dies geschieht typischerweise durch TLS-Zertifikate, deren Validierung clientseitig erfolgen muss.
Autorisierungsrichtlinien: Nach erfolgreicher Authentifizierung muss die Autorisierung erfolgen. Es muss klar definiert werden, welche authentifizierten Clients (oder Benutzer, in deren Namen sie handeln) auf welche MCP-Server, Ressourcen und Werkzeuge zugreifen dürfen.

Dies erfordert eine Integration mit einem bestehenden oder neu zu definierenden Identitäts- und Zugriffsmanagementsystem (IAM) für NovaDE.
Das MCP Host-Konzept ist hier zentral: Der Host-Prozess ist für die Verwaltung von Sicherheitsrichtlinien und Benutzerautorisierung zuständig.2 Dies impliziert, dass der NovaDE MCP Host eine kritische Rolle bei der Durchsetzung von Zugriffsrechten spielt.
Wenn MCP-Server privilegierte Operationen im System ausführen (z.B. bei Integration mit Desktop-Technologien), könnte PolicyKit 108 für die Autorisierungsprüfungen auf Systemebene herangezogen werden. Der MCP-Server würde dann als Mechanismus im Sinne von PolicyKit agieren.





7.2. Berechtigungsmodell für Ressourcen und Werkzeuge

Es ist ein granulares Berechtigungsmodell zu definieren, das spezifische Aktionen auf MCP-Ressourcen (z.B. read, write, list) und die Ausführung von MCP-Werkzeugen (execute) abdeckt.
Diese Berechtigungen sollten an Rollen oder individuelle Client-Identitäten gebunden sein und vom MCP-Server bzw. dem MCP-Host bei jeder Anfrage überprüft werden.
Die Definition dieser Berechtigungen muss eng mit der "Domänenschicht-Spezifikation" und den dort definierten Zugriffsregeln verknüpft sein.



7.3. Datensicherheit

Verschlüsselung bei der Übertragung (Encryption in Transit): Für HTTP/SSE-basierte MCP-Kommunikation ist die Verwendung von TLS (HTTPS/WSS) zwingend erforderlich, um die Vertraulichkeit und Integrität der übertragenen Daten zu gewährleisten.
Verschlüsselung im Ruhezustand (Encryption at Rest): Falls NovaDE-MCP-Server Daten persistent speichern (z.B. Konfigurationen, zwischengespeicherte Ressourcendaten), müssen diese Daten im Ruhezustand verschlüsselt werden, um unbefugten Zugriff zu verhindern. Die Wahl der Verschlüsselungsmethoden sollte aktuellen Sicherheitsstandards entsprechen.
Geheimnisverwaltung (Secret Management): MCP-Server benötigen möglicherweise Geheimnisse (API-Schlüssel, Datenbank-Passwörter, Zugriffstoken für die Domänenschicht). Diese Geheimnisse müssen sicher gespeichert und verwaltet werden.

Die Nutzung der Freedesktop Secrets API über D-Bus (Schnittstelle org.freedesktop.Secrets 15) ist eine Option für NovaDE-MCP-Server, um solche Geheimnisse sicher im Benutzerkontext oder Systemkontext zu speichern und abzurufen. Dies ist besonders relevant, wenn Server im Auftrag des Benutzers auf geschützte Domänenressourcen zugreifen.





7.4. Benutzereinwilligung (User Consent)

Für Operationen, die auf sensible Benutzerdaten zugreifen oder signifikante Aktionen im Namen des Benutzers ausführen (z.B. das Ändern von Systemeinstellungen, Senden von Nachrichten), müssen Mechanismen zur Einholung der expliziten Zustimmung des Benutzers implementiert werden.
Der MCP Host-Prozess spielt auch hier eine Rolle bei der Durchsetzung von Einwilligungsanforderungen.2
Die Gestaltung der Einwilligungsdialoge muss transparent und verständlich sein, damit der Benutzer eine informierte Entscheidung treffen kann. XDG Desktop Portals 87 bieten Standardmechanismen für benutzervermittelte Zugriffsanfragen, die als Inspiration dienen oder direkt genutzt werden könnten, falls MCP-Werkzeuge solche Portale kapseln.


Die Sicherheitsarchitektur von NovaDE muss einen oder mehrere MCP Hosts definieren, die als Gatekeeper fungieren und die oben genannten Sicherheitsfunktionen koordinieren und durchsetzen. Ohne klar definierte Hosts könnten Sicherheitsrichtlinien inkonsistent angewendet werden.8. FehlerbehandlungsspezifikationEine konsistente und informative Fehlerbehandlung ist entscheidend für die Robustheit, Wartbarkeit und Benutzerfreundlichkeit der MCP-Schnittstellen im NovaDE-Projekt. Diese Spezifikation definiert Standardfehlercodes und Richtlinien für die Fehlerbehandlung.

8.1. Standard-MCP-Fehlercodes für NovaDEZusätzlich zu den Standard-JSON-RPC-2.0-Fehlercodes (Parse Error: -32700, Invalid Request: -32600, Method not found: -32601, Invalid params: -32602, Internal error: -32603) definiert NovaDE einen Satz erweiterter Fehlercodes, um spezifischere Fehlersituationen innerhalb des MCP-Kontexts zu signalisieren. Diese Codes sollten von allen NovaDE-MCP-Servern konsistent verwendet werden. Die Struktur der Fehlerantwort folgt dem Standard-JSON-RPC-Error-Objekt.5

Tabelle 8.1: NovaDE MCP Fehlercodes


CodeName (Konstante)Nachricht (Template)HTTP-Status (für SSE)Beschreibung-32000DOMAIN_SPECIFIC_ERROR"Domänenspezifischer Fehler: {details}"500Ein Fehler ist in der NovaDE-Domänenschicht aufgetreten. {details} kann spezifische Informationen enthalten.-32001RESOURCE_NOT_FOUND"Ressource '{uri}' nicht gefunden."404Die angeforderte MCP-Ressource existiert nicht oder ist nicht zugänglich.-32002TOOL_EXECUTION_FAILED"Ausführung des Werkzeugs '{toolName}' fehlgeschlagen."500Ein unerwarteter Fehler während der Ausführung eines MCP-Werkzeugs.-32003INVALID_TOOL_PARAMETERS"Ungültige Parameter für Werkzeug '{toolName}'."400Die für ein MCP-Werkzeug bereitgestellten Parameter sind ungültig oder unvollständig.-32004PERMISSION_DENIED"Zugriff für Operation '{operation}' auf '{target}' verweigert."403Dem aufrufenden Client fehlen die notwendigen Berechtigungen für die angeforderte Operation.-32005SERVER_UNAVAILABLE"MCP-Server ist temporär nicht verfügbar."503Der angefragte MCP-Server ist derzeit nicht erreichbar oder überlastet.-32006AUTHENTICATION_FAILED"Authentifizierung fehlgeschlagen."401Die Authentifizierung des Clients ist fehlgeschlagen.-32007PROTOCOL_VERSION_MISMATCH"Inkompatible Protokollversion. Client: {clientVersion}, Server unterstützt: {serverVersions}"400Client und Server konnten sich nicht auf eine gemeinsame MCP-Protokollversion einigen.Die Verwendung von Rust-Bibliotheken wie `thiserror` [125, 126, 127] oder `snafu` [128] wird für die Implementierung strukturierter Fehler in den Rust-basierten MCP-Servern von NovaDE empfohlen. Diese Bibliotheken erleichtern die Definition von Fehler-Enums, die automatische Implementierung von `std::error::Error` und `Display`, sowie das Anhängen von Kontextinformationen.


8.2. Fehlerweiterleitung (Error Propagation)

MCP-Server müssen Fehler, die in der darunterliegenden Domänenschicht oder von abhängigen Diensten (z.B. D-Bus-Dienste, externe APIs) auftreten, abfangen und in standardisierte MCP-Fehlerantworten umwandeln. Dabei ist es wichtig, eine Balance zu finden: Einerseits soll genügend Kontext für die Fehlerdiagnose bereitgestellt werden, andererseits dürfen keine sensiblen internen Implementierungsdetails oder Sicherheitsinformationen an den Client durchsickern.
Die source-Kette von Fehlern, wie sie von std::error::Error und Crates wie thiserror unterstützt wird, kann intern zur Diagnose verwendet werden, aber die an den MCP-Client gesendete Fehlernachricht sollte sorgfältig formuliert sein. Die Diskussion in 128 über das Gruppieren mehrerer Fehlertypen und das Hinzufügen von Kontext ist hier relevant.



8.3. Client-seitige Fehlerbehandlung

MCP-Clients im NovaDE-Projekt müssen robust auf Fehlerantworten reagieren. Dies beinhaltet das Parsen des error-Objekts, die Interpretation des code und der message, und gegebenenfalls die Nutzung der data-Komponente.
Abhängig vom Fehlercode und der Natur des Fehlers können verschiedene Strategien angewendet werden:

Wiederholungsversuche (Retries): Bei temporären Fehlern (z.B. SERVER_UNAVAILABLE oder bestimmten Netzwerkfehlern) können Clients Wiederholungsversuche mit exponentiellem Backoff implementieren.
Benutzerbenachrichtigung: Bei Fehlern, die eine Benutzerinteraktion erfordern oder den Benutzer über ein Problem informieren müssen (z.B. PERMISSION_DENIED, AUTHENTICATION_FAILED), sollte eine klare und verständliche Meldung angezeigt werden.
Graceful Degradation: Wenn eine Funktionalität aufgrund eines Fehlers nicht verfügbar ist, sollte der Client versuchen, in einem eingeschränkten Modus weiterzuarbeiten oder alternative Pfade anzubieten.


Die mcp_client_rs 7 und mcpr 10 SDKs stellen Result-Typen für ihre Operationen bereit, die eine Fehlerbehandlung über das Err-Variant ermöglichen.

Die Unterscheidung zwischen Protokollfehlern (die eine JSON-RPC ErrorResponse auslösen) und anwendungsspezifischen Werkzeug-Fehlern ist wichtig. Wie in 116 (impliziert durch isError in CallToolResult bei einigen SDK-Interpretationen) angedeutet, kann ein Werkzeugaufruf protokollkonform erfolgreich sein, die interne Logik des Werkzeugs jedoch fehlschlagen. In solchen Fällen sollte die CallToolResult isError: true und ein anwendungsspezifisches error-Objekt im result-Feld enthalten, anstatt einen JSON-RPC-Protokollfehler auszulösen. Dies ermöglicht eine differenziertere Fehlerbehandlung auf Client-Seite. Diese Spezifikation muss klar definieren, wann welche Art von Fehler gemeldet wird.

9. Versionierung und ProtokollevolutionUm die langfristige Wartbarkeit und Kompatibilität der MCP-Schnittstellen im NovaDE-Projekt sicherzustellen, ist eine klare Strategie für Versionierung und Protokollevolution unerlässlich.

9.1. MCP-Versionsstrategie für NovaDE

Globale MCP-Version: Das NovaDE-Projekt wird sich an der offiziellen Versionierung des Model Context Protocol orientieren, wie sie von den Standardisierungsgremien (z.B. Anthropic und die Community) vorgegeben wird. Aktuell wird auf eine Version wie "2025-03-26" referenziert.5 Die Initialize-Nachricht dient dem Aushandeln dieser Basis-Protokollversion zwischen Client und Server.3
NovaDE-spezifische Schnittstellenversionierung: Jede in Abschnitt 5 definierte, NovaDE-spezifische MCP-Schnittstelle (z.B. de.nova.mcp.core.systemInfo) erhält eine eigene semantische Versionierung (z.B. 1.0.0). Diese Version wird im serverVersion-Feld der InitializeResult-Nachricht für den jeweiligen Server und idealerweise als Teil der Metadaten einer Ressource oder eines Werkzeugs kommuniziert.
Granularität der Versionierung: Einzelne Ressourcen oder Werkzeuge innerhalb einer Schnittstelle können bei Bedarf ebenfalls versioniert werden, falls sich ihre Schemata oder Verhalten unabhängig von der Gesamtschnittstelle ändern. Dies sollte jedoch zugunsten der Einfachheit vermieden werden, wenn möglich.



9.2. Umgang mit abwärtskompatiblen ÄnderungenAbwärtskompatible Änderungen sind solche, die bestehende Clients nicht brechen.

Beispiele:

Hinzufügen neuer, optionaler Felder zu Anfrage- oder Antwort-Payloads.
Hinzufügen neuer, optionaler Parameter zu Werkzeugen.
Hinzufügen neuer Werkzeuge oder Ressourcen zu einer bestehenden Schnittstelle.
Hinzufügen neuer Werte zu Enums (Clients sollten unbekannte Enum-Werte tolerant behandeln).


Vorgehen: Solche Änderungen führen zu einer Erhöhung der Minor- oder Patch-Version der betroffenen NovaDE-spezifischen Schnittstelle (z.B. von 1.0.0 auf 1.1.0 oder 1.0.1). Clients, die für eine ältere Minor-Version entwickelt wurden, sollten weiterhin mit Servern funktionieren, die eine neuere Minor-Version derselben Major-Version implementieren.



9.3. Umgang mit abwärtsinkompatiblen ÄnderungenAbwärtsinkompatible Änderungen sind solche, die bestehende Clients potenziell brechen können.

Beispiele:

Entfernen von Feldern aus Anfrage- oder Antwort-Payloads.
Umbenennen von Feldern oder Ändern ihres Datentyps.
Ändern erforderlicher Parameter für Werkzeuge.
Entfernen von Werkzeugen oder Ressourcen.
Grundlegende Änderung der Semantik einer Operation.


Vorgehen:

Solche Änderungen erfordern eine Erhöhung der Major-Version der betroffenen NovaDE-spezifischen Schnittstelle (z.B. von 1.1.0 auf 2.0.0).
Es wird dringend empfohlen, abwärtsinkompatible Änderungen so weit wie möglich zu vermeiden.
Wenn eine solche Änderung unumgänglich ist, sollte idealerweise für eine Übergangszeit sowohl die alte als auch die neue Version der Schnittstelle parallel angeboten werden (z.B. unter einem anderen Endpunkt oder mit einer expliziten Versionsauswahl im Initialize-Request).
Eine klare Kommunikation und Migrationspfade für Clients müssen bereitgestellt werden.



Die Initialize-Nachricht spielt eine Schlüsselrolle bei der Versionierung, da sie es Clients und Servern ermöglicht, ihre unterstützten Protokollversionen und optional auch spezifische Feature-Flags auszutauschen.3 NovaDE-Clients sollten darauf vorbereitet sein, dass Server möglicherweise nicht alle angefragten Features oder die exakt gleiche Schnittstellenversion unterstützen, und entsprechend reagieren (z.B. durch Deaktivieren bestimmter Funktionalitäten oder Melden einer Inkompatibilität).

10. SchlussfolgerungenDie Implementierung des Model Context Protocol (MCP) im NovaDE-Projekt stellt einen strategisch wichtigen Schritt dar, um die Integration von KI-Funktionalitäten auf einer standardisierten, flexiblen und zukunftssicheren Basis zu ermöglichen. Diese Ultra-Feinspezifikation legt den detaillierten Rahmen für die MCP-Schnittstellen, Nachrichtenformate, Integrationsstrategien mit der Domänenschicht sowie für Implementierungs-, Sicherheits- und Fehlerbehandlungsaspekte fest.Wesentliche Erkenntnisse und Implikationen sind:
Standardisierung als Fundament: MCP bietet eine universelle Sprache für die Kommunikation zwischen KI-Modellen und den vielfältigen Datenquellen und Werkzeugen des NovaDE-Projekts.1 Dies reduziert den Aufwand für proprietäre Integrationen und fördert die Interoperabilität.
Abhängigkeit von der Domänenschicht: Die konkrete Ausgestaltung der NovaDE-spezifischen MCP-Ressourcen, -Werkzeuge und -Benachrichtigungen ist untrennbar mit der noch zu detaillierenden "Domänenschicht-Spezifikation" verbunden. Diese Spezifikation muss als Grundlage für die in Abschnitt 5 vorgesehenen Definitionen dienen.
Architektonische Entscheidungen: Die Wahl der Transportmechanismen (Stdio vs. HTTP/SSE) und die klare Definition von MCP Host-, Server- und Client-Rollen innerhalb der NovaDE-Architektur sind entscheidend für Leistung, Skalierbarkeit und Sicherheit.2 Für reaktive Agenten und Server-Push-Benachrichtigungen ist SSE der empfohlene Weg.
Rust SDKs: Für die Implementierung in Rust bieten sich mcpr 10 und mcp_client_rs 7 an, wobei mcpr aufgrund seiner expliziten SSE-Unterstützung und moderneren Anmutung für komplexere Szenarien mit Benachrichtigungen tendenziell vorzuziehen ist.
Sicherheit als Priorität: Ein robustes Sicherheitsmodell, das Authentifizierung, Autorisierung, Datensicherheit und Benutzereinwilligung umfasst, ist unerlässlich. Die Integration mit bestehenden Systemmechanismen (z.B. PolicyKit, Freedesktop Secrets API) sollte geprüft werden, falls die Domänenschicht dies erfordert.2
Konsistente Fehlerbehandlung und Versionierung: Standardisierte Fehlercodes und eine klare Versionierungsstrategie sind für die Wartbarkeit und Weiterentwicklung des Systems unabdingbar.
Empfehlungen für das weitere Vorgehen:
Priorisierung der Domänenschicht-Spezifikation: Die Fertigstellung und Detaillierung der "Domänenschicht-Spezifikation" ist der nächste kritische Schritt, um die in diesem Dokument vorbereiteten MCP-Schnittstellendefinitionen (Abschnitt 5) mit Leben zu füllen.
Prototypische Implementierung: Es wird empfohlen, frühzeitig mit der prototypischen Implementierung ausgewählter MCP-Server und -Clients zu beginnen, basierend auf den hier spezifizierten Standards und unter Verwendung der evaluierten SDKs. Dies hilft, die Konzepte zu validieren und praktische Erfahrungen zu sammeln.
Iterative Verfeinerung: Diese Spezifikation sollte als lebendes Dokument betrachtet und parallel zur Entwicklung der Domänenschicht und der MCP-Komponenten iterativ verfeinert werden.
Fokus auf Sicherheit: Sicherheitsaspekte müssen von Beginn an in Design und Implementierung aller MCP-Komponenten berücksichtigt werden.
Entwickler-Schulung: Sicherstellen, dass alle beteiligten Entwickler ein tiefes Verständnis von MCP und dieser Spezifikation erlangen.
Durch die konsequente Anwendung dieser Spezifikation kann das NovaDE-Projekt eine leistungsfähige und flexible MCP-Infrastruktur aufbauen, die es ermöglicht, das volle Potenzial moderner KI-Modelle auszuschöpfen.11. Anhang

11.1. Glossar

MCP (Model Context Protocol): Ein offener Standard zur Verbindung von KI-Modellen mit externen Datenquellen und Werkzeugen.
JSON-RPC 2.0: Ein leichtgewichtiges Remote Procedure Call Protokoll, das als Basis für MCP dient.
Ressource (Resource): Eine Dateneinheit, die von einem MCP-Server bereitgestellt und von einem Client gelesen werden kann.
Werkzeug (Tool): Eine Funktion oder Operation, die von einem MCP-Server bereitgestellt und von einem Client aufgerufen werden kann, um Aktionen auszuführen.
Aufforderung (Prompt): Eine vordefinierte Vorlage für Interaktionen mit einem LLM, die Systemanweisungen und Argumente umfassen kann.
Benachrichtigung (Notification): Eine asynchrone Nachricht vom Server an den Client, die über Ereignisse oder Zustandsänderungen informiert.
Stdio (Standard Input/Output): Ein Transportmechanismus für MCP, der auf Standard-Datenströmen basiert, typischerweise für lokale Prozesskommunikation.
SSE (Server-Sent Events): Ein Transportmechanismus für MCP über HTTP, der es einem Server ermöglicht, kontinuierlich Daten an einen Client zu senden.
Domänenschicht: Die spezifische Anwendungslogik und Datenmodelle des NovaDE-Projekts.
NovaDE: Name des Projekts, für das diese MCP-Spezifikation erstellt wird.
Client (MCP): Eine Softwarekomponente (oft Teil eines KI-Agenten oder einer Anwendung), die mit einem MCP-Server interagiert, um Kontext zu erhalten oder Aktionen auszuführen.
Server (MCP): Eine Softwarekomponente, die Daten oder Funktionalitäten über das MCP-Protokoll bereitstellt.
Host (MCP): Eine Anwendung oder Umgebung, die MCP-Clients beherbergt und deren Interaktionen mit MCP-Servern koordiniert und absichert.
URI (Uniform Resource Identifier): Eine Zeichenfolge zur eindeutigen Identifizierung einer Ressource.



11.2. JSON Schema Beispiele (Referenz)(Dieser Abschnitt würde exemplarische JSON-Schemata für typische Ressourcen oder Werkzeugparameter enthalten, um die in den Tabellen beschriebenen Strukturen zu illustrieren. Aufgrund der fehlenden Domänenschicht-Spezifikation sind dies allgemeine Beispiele.)


Beispiel: Ressourcenschema für ein einfaches Dateiobjekt
JSON{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "NovaDEFileResource",
  "description": "Repräsentiert eine Datei im NovaDE-System.",
  "type": "object",
  "properties": {
    "uri": {
      "type": "string",
      "format": "uri",
      "description": "Eindeutiger URI der Datei."
    },
    "name": {
      "type": "string",
      "description": "Dateiname."
    },
    "size": {
      "type": "integer",
      "description": "Dateigröße in Bytes.",
      "minimum": 0
    },
    "mimeType": {
      "type": "string",
      "description": "MIME-Typ der Datei."
    },
    "lastModified": {
      "type": "string",
      "format": "date-time",
      "description": "Zeitpunkt der letzten Änderung (ISO 8601)."
    },
    "contentPreview": {
      "type": "string",
      "description": "Optionale Vorschau des Dateiinhalts (z.B. erste Zeilen einer Textdatei)."
    }
  },
  "required":
}



Beispiel: Parameterschema für ein Werkzeug createDocument
JSON{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CreateDocumentToolParams",
  "description": "Parameter für das Werkzeug 'createDocument'.",
  "type": "object",
  "properties": {
    "parentFolderUri": {
      "type": "string",
      "format": "uri",
      "description": "URI des Ordners, in dem das Dokument erstellt werden soll."
    },
    "documentName": {
      "type": "string",
      "description": "Name des zu erstellenden Dokuments.",
      "minLength": 1
    },
    "initialContent": {
      "type": "string",
      "description": "Optionaler initialer Inhalt des Dokuments."
    },
    "templateId": {
      "type": "string",
      "description": "Optionale ID einer Vorlage, die für das neue Dokument verwendet werden soll."
    }
  },
  "required": [
    "parentFolderUri",
    "documentName"
  ]
}



Beispiel: Ergebnisschema für ein Werkzeug createDocument
JSON{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CreateDocumentToolResult",
  "description": "Ergebnis des Werkzeugs 'createDocument'.",
  "type": "object",
  "properties": {
    "documentUri": {
      "type": "string",
      "format": "uri",
      "description": "URI des neu erstellten Dokuments."
    },
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "Zeitpunkt der Erstellung (ISO 8601)."
    }
  },
  "required": [
    "documentUri",
    "timestamp"
  ]
}





# Ultra-Feinspezifikation der MCP-Schnittstellen und Implementierungen für das NovaDE-Projekt

## 1. Einleitung

### 1.1. Zweck des Dokuments

Dieses Dokument definiert die Ultra-Feinspezifikation aller Model Context Protocol (MCP) Schnittstellen und deren Implementierungen innerhalb des NovaDE-Projekts. Es dient als maßgebliche technische Referenz für die Entwicklung, Integration und Wartung von MCP-basierten Komponenten im NovaDE-Ökosystem. Die Spezifikation umfasst detaillierte Beschreibungen von Nachrichtenformaten, Datenstrukturen, Methoden, Ereignissen und Fehlerbehandlungsmechanismen. Ein besonderer Fokus liegt auf der Integration der Domänenschicht-Spezifikation des NovaDE-Projekts in die MCP-Schnittstellen.

### 1.2. Geltungsbereich

Diese Spezifikation bezieht sich auf sämtliche Aspekte des Model Context Protocol, wie es im Kontext des NovaDE-Projekts eingesetzt wird. Dies beinhaltet:

- Alle MCP-Schnittstellen, die im NovaDE-Projekt definiert oder genutzt werden.
- Die Interaktion dieser MCP-Schnittstellen mit anderen Systemkomponenten, einschließlich, aber nicht beschränkt auf D-Bus-Dienste, Wayland-Protokolle und PipeWire-Audio-Management.
- Implementierungsrichtlinien und -details, insbesondere unter Verwendung der Programmiersprache Rust und assoziierter Bibliotheken.
- Die nahtlose Einbindung der fachlichen Anforderungen und Datenmodelle aus der Domänenschicht-Spezifikation des NovaDE-Projekts.

### 1.3. Zielgruppe

Dieses Dokument richtet sich an folgende Personengruppen innerhalb des NovaDE-Projekts:

- Softwarearchitekten und -entwickler, die MCP-Schnittstellen und -Komponenten entwerfen, implementieren oder nutzen.
- Systemintegratoren, die für die Bereitstellung und Konfiguration von NovaDE-Systemen verantwortlich sind.
- Qualitätssicherungsingenieure, die MCP-Funktionalitäten testen.
- Technische Projektmanager, die die Entwicklung und Implementierung des NovaDE-Projekts überwachen.

### 1.4. Definitionen und Akronyme

- **MCP:** Model Context Protocol. Ein offener Standard zur Kommunikation zwischen KI-Modellen/Anwendungen und externen Werkzeugen oder Datenquellen.1
- **NovaDE-Projekt:** Das spezifische Projekt, für das diese MCP-Spezifikation erstellt wird. (Details zum Projekt selbst sind außerhalb des Geltungsbereichs der bereitgestellten Materialien).
- **Domänenschicht-Spezifikation:** Ein separates Dokument, das die fachlichen Entitäten, Geschäftsregeln und Datenmodelle des NovaDE-Projekts beschreibt. Diese Spezifikation wird als integraler Bestandteil der MCP-Schnittstellendefinitionen betrachtet.
- **API:** Application Programming Interface.
- **D-Bus:** Desktop Bus, ein System für Interprozesskommunikation (IPC).3
- **Wayland:** Ein Kommunikationsprotokoll zwischen einem Display-Server (Compositor) und seinen Clients.4
- **PipeWire:** Ein Multimedia-Framework für Audio- und Videoverarbeitung unter Linux.5
- **XDG Desktop Portals:** Ein Framework, das sandboxed Anwendungen den sicheren Zugriff auf Ressourcen außerhalb der Sandbox ermöglicht.6
- **JSON-RPC:** JavaScript Object Notation Remote Procedure Call. Ein leichtgewichtiges RPC-Protokoll.8
- **Stdio:** Standard Input/Output.
- **SSE:** Server-Sent Events. Eine Technologie, die es einem Server ermöglicht, Updates an einen Client über eine HTTP-Verbindung zu pushen.8
- **Smithay:** Eine Rust-Bibliothek zur Erstellung von Wayland-Compositoren.10
- **zbus:** Eine Rust-Bibliothek für die D-Bus-Kommunikation.12
- **pipewire-rs:** Rust-Bindungen für PipeWire.14
- **mcpr:** Eine Rust-Implementierung des Model Context Protocol.16
- **mcp_client_rs:** Eine weitere Rust-Client-SDK für MCP.17

### 1.5. Referenzierte Dokumente

- Model Context Protocol Specification (Version 2025-03-26 oder aktueller) 2
- Domänenschicht-Spezifikation des NovaDE-Projekts (externes Dokument)
- Freedesktop D-Bus Specification 3
- Wayland Protocol Specification 4
- PipeWire Documentation 5
- XDG Desktop Portal Documentation 6
- Spezifikationen der relevanten D-Bus-Schnittstellen (Secrets, PolicyKit, Portals, Login1, UPower, Notifications)
- Spezifikationen der relevanten Wayland-Protokolle und -Erweiterungen
- Dokumentation der verwendeten Rust-Bibliotheken (Smithay, zbus, pipewire-rs, mcpr, mcp_client_rs, tokio, serde, thiserror etc.)

## 2. Model Context Protocol (MCP) – Grundlagen

### 2.1. Überblick und Kernkonzepte

Das Model Context Protocol (MCP) ist ein offener Standard, der darauf abzielt, die Integration von Large Language Models (LLMs) mit externen Werkzeugen, Datenbanken und APIs zu standardisieren.1 Es fungiert als eine universelle Schnittstelle, die es KI-Modellen ermöglicht, dynamisch auf Kontextinformationen zuzugreifen und Aktionen in ihrer Umgebung auszuführen.9 MCP adressiert die Herausforderung der Informationssilos und proprietären Integrationen, indem es einen einheitlichen Rahmen für die KI-Tool-Kommunikation schafft.1

Die Kernprinzipien von MCP umfassen 2:

- **Standardisierte Schnittstelle:** Einheitliche Methoden für LLMs zum Zugriff auf Werkzeuge und Ressourcen.
- **Erweiterte Fähigkeiten:** Befähigung von LLMs zur Interaktion mit diversen Systemen.
- **Sicherheit und Kontrolle:** Strukturierte Zugriffsmuster mit integrierter Validierung und klaren Grenzen.
- **Modularität und Erweiterbarkeit:** Einfaches Hinzufügen neuer Fähigkeiten durch Server, ohne die Kernanwendung des LLMs modifizieren zu müssen.

MCP ist darauf ausgelegt, die Reproduzierbarkeit von KI-Interaktionen zu verbessern, indem der gesamte notwendige Kontext (Datensätze, Umgebungsspezifikationen, Hyperparameter) an einem Ort verwaltet wird.1

### 2.2. Architektur (Client-Host-Server-Modell)

MCP basiert auf einer Client-Host-Server-Architektur 8:

- **Host:** Eine LLM-Anwendung (z.B. Claude Desktop, IDEs), die Verbindungen initiiert und als Container oder Koordinator für mehrere Client-Instanzen fungiert. Der Host verwaltet den Lebenszyklus, Sicherheitsrichtlinien (Berechtigungen, Benutzerautorisierung) und die Integration des LLMs.1
- **Client:** Eine Protokoll-Client-Komponente innerhalb der Host-Anwendung, die eine 1:1-Verbindung zu einem MCP-Server herstellt. Der Client ist verantwortlich für die Aushandlung von Fähigkeiten und die Orchestrierung von Nachrichten zwischen sich und dem Server.1
- **Server:** Ein Dienst (oft ein leichtgewichtiger Prozess), der spezifische Kontexte, Werkzeuge und Prompts für den Client bereitstellt. Server können lokale Prozesse oder entfernte Dienste sein und kapseln den Zugriff auf Datenquellen, APIs oder andere Utilities.1

Diese Architektur ermöglicht eine klare Trennung der Verantwortlichkeiten und fördert die Entwicklung modularer und wiederverwendbarer MCP-Server.23 Die Kommunikation zwischen diesen Komponenten erfolgt über eine Transportschicht und eine Protokollschicht, die auf JSON-RPC aufbaut und zustandsbehaftete Sitzungen für den Kontextaustausch und das Sampling betont.1

### 2.3. Nachrichtenformate (JSON-RPC 2.0 Basis)

MCP verwendet JSON-RPC 2.0 als Grundlage für seine Nachrichtenformate.8 Dies gewährleistet eine strukturierte und standardisierte Kommunikation. Die Hauptnachrichtentypen sind 8:

- **Requests (Anfragen):** Vom Client oder Server gesendete Nachrichten, die eine Antwort erwarten. Sie enthalten typischerweise eine `method` (Methodenname) und optionale `params` (Parameter).
    - Beispiel: `{"jsonrpc": "2.0", "method": "tools/list", "id": 1}`
- **Responses (Antworten):** Erfolgreiche Antworten auf Requests. Sie enthalten ein `result`-Feld mit den Ergebnisdaten und die `id` des ursprünglichen Requests.
    - Beispiel: `{"jsonrpc": "2.0", "result": {"tools": [...]}, "id": 1}`
- **Error Responses (Fehlerantworten):** Antworten, die anzeigen, dass ein Request fehlgeschlagen ist. Sie enthalten ein `error`-Objekt mit `code`, `message` und optional `data`, sowie die `id` des ursprünglichen Requests.
    - Beispiel: `{"jsonrpc": "2.0", "error": {"code": -32601, "message": "Method not found"}, "id": 1}`
- **Notifications (Benachrichtigungen):** Einwegnachrichten, die keine Antwort erwarten. Sie enthalten eine `method` und optionale `params`, aber keine `id`.
    - Beispiel: `{"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {...}}`

Die spezifischen Methoden und Parameter für MCP-Nachrichten wie `initialize`, `tools/list`, `resources/read`, `tools/call` werden im weiteren Verlauf dieses Dokuments detailliert [2 (angenommen)].

### 2.4. Transportmechanismen

MCP unterstützt verschiedene Transportmechanismen für die Kommunikation zwischen Host/Client und Server 8:

- **Stdio (Standard Input/Output):** Dieser Mechanismus wird für die Kommunikation mit lokalen Prozessen verwendet. Der MCP-Server läuft als separater Prozess, und die Kommunikation erfolgt über dessen Standard-Eingabe- und Ausgabe-Streams. Dies ist ideal für Kommandozeilenwerkzeuge und lokale Entwicklungsszenarien.16 Die Rust-Bibliothek `mcpr` bietet beispielsweise `StdioTransport` 16, und `mcp_client_rs` fokussiert sich ebenfalls auf diesen Transportweg für lokal gespawnte Server.18
- **HTTP mit SSE (Server-Sent Events):** Dieser Mechanismus wird für netzwerkbasierte Kommunikation verwendet, insbesondere wenn der Server remote ist oder Echtzeit-Updates vom Server an den Client erforderlich sind. SSE ermöglicht es dem Server, asynchron Nachrichten an den Client zu pushen, während Client-zu-Server-Nachrichten typischerweise über HTTP POST gesendet werden.8 Die `mcpr`-Bibliothek erwähnt SSE-Transportunterstützung.16

Die Wahl des Transportmechanismus hängt von den spezifischen Anforderungen der NovaDE-Komponente ab, insbesondere davon, ob der MCP-Server lokal oder remote betrieben wird.

### 2.5. Sicherheitsaspekte

Sicherheit und Datenschutz sind zentrale Aspekte des Model Context Protocol, da es potenziell den Zugriff auf sensible Daten und die Ausführung von Code ermöglicht.2 Die Spezifikation legt folgende Schlüsselprinzipien fest 2:

- **Benutzereinwilligung und -kontrolle:**
    - Benutzer müssen explizit allen Datenzugriffen und Operationen zustimmen und diese verstehen.
    - Benutzer müssen die Kontrolle darüber behalten, welche Daten geteilt und welche Aktionen ausgeführt werden.
    - Implementierungen sollten klare Benutzeroberflächen zur Überprüfung und Autorisierung von Aktivitäten bereitstellen.
- **Datenschutz:**
    - Hosts müssen die explizite Zustimmung des Benutzers einholen, bevor Benutzerdaten an Server weitergegeben werden.
    - Hosts dürfen Ressourcendaten nicht ohne Zustimmung des Benutzers an andere Stellen übertragen.
    - Benutzerdaten sollten durch geeignete Zugriffskontrollen geschützt werden.
- **Werkzeugsicherheit (Tool Safety):**
    - Werkzeuge repräsentieren die Ausführung von beliebigem Code und müssen mit entsprechender Vorsicht behandelt werden. Beschreibungen des Werkzeugverhaltens (z.B. Annotationen) sind als nicht vertrauenswürdig zu betrachten, es sei denn, sie stammen von einem vertrauenswürdigen Server.
    - Hosts müssen die explizite Zustimmung des Benutzers einholen, bevor ein Werkzeug aufgerufen wird.
    - Benutzer sollten verstehen, was jedes Werkzeug tut, bevor sie dessen Verwendung autorisieren.
- **LLM Sampling Controls:**
    - Benutzer müssen explizit allen LLM-Sampling-Anfragen zustimmen.
    - Benutzer sollten kontrollieren, ob Sampling überhaupt stattfindet, den tatsächlichen Prompt, der gesendet wird, und welche Ergebnisse der Server sehen kann.

Obwohl MCP diese Prinzipien nicht auf Protokollebene erzwingen kann, **SOLLTEN** Implementierer robuste Zustimmungs- und Autorisierungsflüsse entwickeln, Sicherheitsimplikationen klar dokumentieren, geeignete Zugriffskontrollen und Datenschutzmaßnahmen implementieren und bewährte Sicherheitspraktiken befolgen.2 Die Architektur mit MCP-Servern als Vermittler kann eine zusätzliche Sicherheitsebene bieten, indem der Zugriff auf Ressourcen kontrolliert und potenziell in einer Sandbox ausgeführt wird.19

## 3. MCP-Schnittstellen im NovaDE-Projekt – Allgemeine Spezifikation

### 3.1. Namenskonventionen und Versionierung

Für alle MCP-Schnittstellen, die im Rahmen des NovaDE-Projekts definiert werden, gelten folgende Namenskonventionen und Versionierungsrichtlinien:

- **Schnittstellennamen:** Schnittstellennamen folgen dem Muster `nova.<KomponentenName>.<Funktionsbereich>.<Version>`. Beispiel: `nova.workspace.fileAccess.v1`. Dies gewährleistet Eindeutigkeit und Klarheit über den Ursprung und Zweck der Schnittstelle.
- **Methodennamen:** Methodennamen verwenden camelCase, beginnend mit einem Kleinbuchstaben (z.B. `listResources`, `callTool`).
- **Parameternamen:** Parameternamen verwenden ebenfalls camelCase.
- **Versionierung:** Jede MCP-Schnittstelle wird explizit versioniert. Die Version wird als Teil des Schnittstellennamens geführt (z.B. `v1`, `v2`). Änderungen, die die Abwärtskompatibilität brechen, erfordern eine Erhöhung der Hauptversionsnummer. Abwärtskompatible Erweiterungen können zu einer Erhöhung einer Nebenversionsnummer führen, falls ein solches Schema zusätzlich eingeführt wird. Das NovaDE-Projekt hält sich an die im MCP-Standard definierte Protokollversion (z.B. `2025-03-26`).2 Die aktuell unterstützte MCP-Protokollversion ist im `mcp_client_rs` Crate als `LATEST_PROTOCOL_VERSION` und `SUPPORTED_PROTOCOL_VERSIONS` definiert.27

### 3.2. Standardnachrichtenflüsse

Die Kommunikation im NovaDE-Projekt über MCP folgt etablierten Nachrichtenflüssen, die auf dem JSON-RPC 2.0 Standard basieren.8

1. **Initialisierung (Connection Lifecycle):** 8
    - Der MCP-Client (innerhalb des NovaDE-Hosts) sendet eine `initialize`-Anfrage an den MCP-Server. Diese Anfrage enthält die vom Client unterstützte Protokollversion und dessen Fähigkeiten (Capabilities).
    - Der MCP-Server antwortet mit seiner Protokollversion und seinen Fähigkeiten.
    - Der Client bestätigt die erfolgreiche Initialisierung mit einer `initialized`-Notification.
    - Anschließend beginnt der reguläre Nachrichtenaustausch.
2. **Anfrage-Antwort (Request-Response):** 8
    - Der Client sendet eine Anfrage (z.B. `tools/list`, `resources/read`, `tools/call`) mit einer eindeutigen ID.
    - Der Server verarbeitet die Anfrage und sendet entweder eine Erfolgsantwort mit dem Ergebnis (`result`) und derselben ID oder eine Fehlerantwort (`error`) mit Fehlercode, Nachricht und derselben ID.
3. **Benachrichtigungen (Notifications):** 8
    - Client oder Server können einseitige Benachrichtigungen senden, die keine direkte Antwort erwarten. Diese haben keine ID. Ein Beispiel ist die `initialized`-Notification oder serverseitige Push-Events.
4. **Beendigung (Termination):** 8
    - Die Verbindung kann durch eine `shutdown`-Anfrage vom Client initiiert werden, gefolgt von einer `exit`-Notification. Alternativ kann die Verbindung durch Schließen des zugrundeliegenden Transportkanals beendet werden.

Die Rust-Bibliotheken `mcpr` und `mcp_client_rs` implementieren diese grundlegenden Nachrichtenflüsse.16 `mcp_client_rs` beispielsweise nutzt Tokio für asynchrone Operationen und stellt Methoden wie `initialize()`, `list_resources()`, `call_tool()` zur Verfügung, die diesen Flüssen folgen.18

### 3.3. Fehlerbehandlung und Fehlercodes

Eine robuste Fehlerbehandlung ist entscheidend für die Stabilität der MCP-Kommunikation im NovaDE-Projekt. MCP-Fehlerantworten folgen dem JSON-RPC 2.0 Standard 8 und enthalten ein `error`-Objekt mit den Feldern `code` (Integer), `message` (String) und optional `data` (beliebiger Typ).

**Standard-Fehlercodes (basierend auf JSON-RPC 2.0):**

- `-32700 Parse error`: Ungültiges JSON wurde empfangen.
- `-32600 Invalid Request`: Die JSON-Anfrage war nicht wohlgeformt.
- `-32601 Method not found`: Die angeforderte Methode existiert nicht oder ist nicht verfügbar.
- `-32602 Invalid params`: Ungültige Methodenparameter.
- `-32603 Internal error`: Interner JSON-RPC-Fehler.
- `-32000` bis `-32099 Server error`: Reserviert für implementierungsspezifische Serverfehler.

NovaDE-spezifische Fehlercodes:

Zusätzlich zu den Standard-JSON-RPC-Fehlercodes definiert das NovaDE-Projekt spezifische Fehlercodes im Bereich -32000 bis -32099 für anwendungsspezifische Fehler, die während der Verarbeitung von MCP-Anfragen auftreten können. Diese Fehlercodes werden pro Schnittstelle und Methode dokumentiert.

Fehlerbehandlung in Rust-Implementierungen:

In Rust-basierten MCP-Implementierungen für NovaDE wird die Verwendung von thiserror für Bibliotheksfehler und potenziell anyhow für Anwendungsfehler empfohlen, um eine klare und kontextreiche Fehlerbehandlung zu gewährleisten.29 Die mcp_client_rs Bibliothek stellt einen Error-Typ bereit, der verschiedene Fehlerquellen kapselt.27 Die Struktur ErrorResponse und das Enum ErrorCode [240 (angenommen)] sind Teil der Protokolldefinitionen zur strukturierten Fehlerkommunikation.

**Beispiel für eine Fehlerantwort:**

JSON

```
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "NovaDE Domain Error: Ressource nicht gefunden.",
    "data": {
      "resourceUri": "nova://domain/entity/123"
    }
  },
  "id": 123
}
```

### 3.4. Integration der Domänenschicht-Spezifikation

Die Domänenschicht-Spezifikation des NovaDE-Projekts ist ein zentrales Element, das die fachlichen Entitäten, Operationen und Geschäftsregeln definiert. Die MCP-Schnittstellen im NovaDE-Projekt müssen diese Domänenspezifikation nahtlos integrieren. Dies bedeutet:

- **Abbildung von Domänenentitäten:** Datenstrukturen innerhalb der MCP-Nachrichten (Parameter von Methoden, Rückgabewerte, Event-Payloads) müssen die Entitäten der Domänenschicht widerspiegeln oder direkt verwenden. Dies stellt sicher, dass die MCP-Kommunikation die fachlichen Anforderungen korrekt abbildet.
- **Domänenoperationen als MCP-Methoden:** Viele MCP-Methoden werden direkte Abbildungen von Operationen sein, die in der Domänenschicht definiert sind. Die Parameter und Rückgabewerte dieser MCP-Methoden korrespondieren mit den Ein- und Ausgaben der Domänenoperationen.
- **Validierung und Geschäftsregeln:** Bevor MCP-Anfragen an die Domänenschicht weitergeleitet oder Ergebnisse von der Domänenschicht über MCP zurückgegeben werden, müssen Validierungen und Geschäftsregeln der Domänenschicht angewendet werden. Dies kann sowohl im MCP-Server als auch in einer zwischengeschalteten Logikschicht geschehen.
- **Konsistente Terminologie:** Die in den MCP-Schnittstellen verwendete Terminologie (Namen von Methoden, Parametern, Datenfeldern) sollte mit der Terminologie der Domänenschicht-Spezifikation übereinstimmen, um Missverständnisse zu vermeiden und die Kohärenz im gesamten System zu fördern.

Die genauen Details der Integration hängen von den spezifischen Inhalten der Domänenschicht-Spezifikation ab. Jede detaillierte MCP-Schnittstellenspezifikation in Abschnitt 4 wird explizit auf die relevanten Teile der Domänenschicht-Spezifikation verweisen und die Abbildung erläutern.

## 4. Detaillierte MCP-Schnittstellenspezifikationen

Für das NovaDE-Projekt werden spezifische MCP-Schnittstellen definiert, um die Interaktion mit verschiedenen Modulen und Diensten zu ermöglichen. Jede Schnittstelle wird nach dem folgenden Schema spezifiziert. _Da die konkreten Schnittstellen für NovaDE nicht in den bereitgestellten Materialien definiert sind, dient der folgende Abschnitt als Vorlage und Beispielstruktur._

---

**Beispiel-Schnittstelle: `nova.dataAccess.document.v1`**

### 4.1. Beschreibung und Zweck

Die Schnittstelle `nova.dataAccess.document.v1` dient dem Zugriff auf und der Verwaltung von Dokumenten innerhalb des NovaDE-Projekts. Sie ermöglicht es MCP-Clients, Dokumente basierend auf Kriterien der Domänenschicht zu lesen, zu schreiben, zu aktualisieren und zu löschen. Diese Schnittstelle interagiert eng mit den Entitäten und Operationen, die in der "Domänenschicht-Spezifikation, Abschnitt X.Y (Dokumentenverwaltung)" definiert sind.

### 4.2. Methoden

#### 4.2.1. `readDocument`

- **Beschreibung:** Liest den Inhalt und die Metadaten eines spezifischen Dokuments.
- **Parameter:**
    - `uri` (String, erforderlich): Der eindeutige URI des Dokuments, konform zum NovaDE-URI-Schema (z.B. `nova://documents/internal/doc123`).
    - `options` (Object, optional): Zusätzliche Optionen für den Lesevorgang.
        - `version` (String, optional): Die spezifische Version des Dokuments, die gelesen werden soll. Falls nicht angegeben, wird die neueste Version gelesen.
- **Rückgabewerte:**
    - `document` (Object): Ein Objekt, das das gelesene Dokument repräsentiert. Die Struktur dieses Objekts ist in der Domänenschicht-Spezifikation definiert und könnte Felder wie `uri`, `mimeType`, `content` (String oder Binary), `metadata` (Object), `version` (String), `lastModified` (Timestamp) enthalten.
- **Mögliche Fehler:**
    - `-32001`: `DOCUMENT_NOT_FOUND` - Das angeforderte Dokument existiert nicht.
    - `-32002`: `ACCESS_DENIED` - Der Client hat keine Berechtigung, das Dokument zu lesen.
    - `-32003`: `VERSION_NOT_FOUND` - Die angeforderte Version des Dokuments existiert nicht.

#### 4.2.2. `writeDocument`

- **Beschreibung:** Schreibt ein neues Dokument oder aktualisiert ein bestehendes Dokument.
- **Parameter:**
    - `uri` (String, erforderlich): Der URI, unter dem das Dokument geschrieben werden soll. Bei Aktualisierung eines bestehenden Dokuments dessen URI.
    - `content` (String oder Binary, erforderlich): Der Inhalt des Dokuments. Der Typ (String oder Base64-kodiertes Binary) hängt vom `mimeType` ab.
    - `mimeType` (String, erforderlich): Der MIME-Typ des Dokuments (z.B. `text/plain`, `application/pdf`).
    - `metadata` (Object, optional): Domänenspezifische Metadaten für das Dokument.
    - `options` (Object, optional):
        - `overwrite` (Boolean, optional, default: `false`): Wenn `true` und ein Dokument unter dem URI existiert, wird es überschrieben. Andernfalls schlägt der Aufruf fehl, wenn das Dokument existiert.
- **Rückgabewerte:**
    - `newUri` (String): Der URI des geschriebenen oder aktualisierten Dokuments (kann sich bei Neuerstellung ändern, falls der Server URIs generiert).
    - `version` (String): Die Versionskennung des geschriebenen Dokuments.
- **Mögliche Fehler:**
    - `-32002`: `ACCESS_DENIED` - Keine Schreibberechtigung.
    - `-32004`: `DOCUMENT_EXISTS` - Dokument existiert bereits und `overwrite` ist `false`.
    - `-32005`: `INVALID_CONTENT` - Der bereitgestellte Inhalt ist für den `mimeType` ungültig.

_(Weitere Methoden wie `deleteDocument`, `listDocuments` würden hier analog spezifiziert werden.)_

### 4.3. Events/Notifications

#### 4.3.1. `documentChanged`

- **Beschreibung:** Wird vom Server gesendet, wenn ein Dokument, für das der Client möglicherweise Interesse bekundet hat (z.B. durch vorheriges Lesen), geändert wurde.
- **Parameter:**
    - `uri` (String): Der URI des geänderten Dokuments.
    - `changeType` (String): Art der Änderung (z.B. `UPDATED`, `DELETED`).
    - `newVersion` (String, optional): Die neue Versionskennung, falls `changeType` `UPDATED` ist.

### 4.4. Datenstrukturen

Die für diese Schnittstelle relevanten Datenstrukturen (z.B. die Struktur eines `Document`-Objekts, `Metadata`-Objekts) werden primär durch die Domänenschicht-Spezifikation des NovaDE-Projekts definiert. MCP-Nachrichten verwenden JSON-Repräsentationen dieser domänenspezifischen Strukturen.

**Beispiel `Document` (basierend auf einer hypothetischen Domänenspezifikation):**

JSON

```
{
  "uri": "nova://documents/internal/doc123",
  "mimeType": "text/plain",
  "content": "Dies ist der Inhalt des Dokuments.",
  "metadata": {
    "author": "NovaUser",
    "tags": ["wichtig", "projektA"],
    "customDomainField": "spezifischerWert"
  },
  "version": "v1.2.3",
  "lastModified": "2024-07-15T10:30:00Z"
}
```

### 4.5. Beispiele für Nachrichten

**Anfrage `readDocument`:**

JSON

```
{
  "jsonrpc": "2.0",
  "method": "nova.dataAccess.document.v1/readDocument",
  "params": {
    "uri": "nova://documents/internal/doc123"
  },
  "id": 1
}
```

**Antwort `readDocument` (Erfolg):**

JSON

```
{
  "jsonrpc": "2.0",
  "result": {
    "document": {
      "uri": "nova://documents/internal/doc123",
      "mimeType": "text/plain",
      "content": "Dies ist der Inhalt des Dokuments.",
      "metadata": {"author": "NovaUser"},
      "version": "v1.0.0",
      "lastModified": "2024-07-15T10:00:00Z"
    }
  },
  "id": 1
}
```

### 4.6. Interaktion mit der Domänenschicht

Die Methode `readDocument` ruft intern die Funktion `DomainLayer.getDocumentByUri(uri, options.version)` der Domänenschicht auf. Die zurückgegebenen Domänenobjekte werden gemäß den MCP-Datenstrukturen serialisiert. Die Methode `writeDocument` validiert die Eingaben anhand der Geschäftsregeln der Domänenschicht (z.B. `DomainLayer.validateDocumentContent(content, mimeType)`) und ruft dann `DomainLayer.saveDocument(documentData)` auf. Berechtigungsprüfungen erfolgen ebenfalls über dedizierte Domänenschicht-Services (z.B. `DomainLayer.Security.canReadDocument(userContext, uri)`).

---

_(Dieser beispielhafte Abschnitt würde für jede spezifische MCP-Schnittstelle im NovaDE-Projekt wiederholt werden.)_

## 5. Implementierung der MCP-Schnittstellen im NovaDE-Projekt

### 5.1. Verwendete Technologien

Die Kernimplementierung der MCP-Schnittstellen und der zugehörigen Logik im NovaDE-Projekt erfolgt in **Rust**. Dies schließt sowohl Client- als auch (potenzielle) Server-seitige Komponenten ein. Die Wahl von Rust begründet sich in dessen Stärken hinsichtlich Systemsicherheit, Performance und Nebenläufigkeit, welche für ein robustes Desktop Environment Projekt wie NovaDE essentiell sind.

Folgende Rust-Bibliotheken (Crates) sind für die MCP-Implementierung von zentraler Bedeutung:

- **MCP-Protokoll-Handling:**
    - `mcp_client_rs` (von darinkishore) [17 (angenommen), 241 (angenommen), 28 (angenommen), 243 (angenommen), 244 (angenommen), 243 (angenommen), 242 (angenommen), 245 (angenommen), 246 (angenommen), 246 (angenommen)] oder alternativ `mcpr` (von conikeec) 16 für die Client-seitige Implementierung. Die Entscheidung für eine spezifische Bibliothek hängt von den detaillierten Anforderungen und der Reife der jeweiligen Bibliothek zum Zeitpunkt der Implementierung ab. Beide bieten Mechanismen zur Serialisierung/Deserialisierung von MCP-Nachrichten und zur Verwaltung der Kommunikation.
- **Asynchrone Laufzeitumgebung:** `tokio` wird als primäre asynchrone Laufzeitumgebung für die nebenläufige Verarbeitung von MCP-Nachrichten und Interaktionen mit anderen Systemdiensten verwendet.25
- **Serialisierung/Deserialisierung:** `serde` und `serde_json` für die Umwandlung von Rust-Datenstrukturen in und aus dem JSON-Format, das von JSON-RPC verwendet wird.25
- **Fehlerbehandlung:** `thiserror` für die Definition von benutzerdefinierten Fehlertypen in Bibliotheks-Code und potenziell `anyhow` für eine vereinfachte Fehlerbehandlung in Anwendungscode.29
- **UUID-Generierung:** Das `uuid` Crate mit den Features `v4` und `serde` wird für die Erzeugung und Handhabung von eindeutigen Identifikatoren verwendet, die in MCP-Nachrichten oder domänenspezifischen Daten benötigt werden könnten.41
- **D-Bus-Kommunikation:** `zbus` für die Interaktion mit Systemdiensten über D-Bus.12
- **Wayland Compositing (falls NovaDE ein Compositor ist oder tief integriert):** `smithay` als Framework für Wayland-spezifische Interaktionen.10
- **PipeWire-Integration:** `pipewire-rs` für die Interaktion mit dem PipeWire Multimedia-Framework.14

### 5.2. MCP-Client-Implementierung (Rust)

Die MCP-Client-Komponenten im NovaDE-Projekt sind für die Kommunikation mit verschiedenen MCP-Servern zuständig, die Werkzeuge und Ressourcen bereitstellen.

#### 5.2.1. Initialisierung und Verbindungsaufbau

Die Initialisierung eines MCP-Clients beginnt mit der Konfiguration des Transports und der Erstellung einer Client-Instanz. Am Beispiel von `mcp_client_rs` (darinkishore):

- **Server-Spawning (für lokale Server via Stdio):** Die `ClientBuilder`-API ermöglicht das Starten eines lokalen MCP-Serverprozesses und die Verbindung zu dessen Stdio-Kanälen.17
    
    Rust
    
    ```
    // Beispielhafte Initialisierung (Pseudocode, da Servername und Argumente spezifisch für NovaDE sind)
    // use mcp_client_rs::client::ClientBuilder;
    // let client = ClientBuilder::new("nova-mcp-server-executable")
    //    .arg("--config-path")
    //    .arg("/etc/nova/mcp_server_config.json")
    //    .spawn_and_initialize().await?;
    ```
    
    Es ist wichtig zu beachten, dass `mcp_client_rs` (darinkishore) primär für lokal gespawnte Server konzipiert ist und keine direkte Unterstützung für Remote-Server plant.17 Für Remote-Verbindungen via HTTP/SSE müsste eine andere Bibliothek oder eine Erweiterung dieses Ansatzes in Betracht gezogen werden, wie sie z.B. in `mcpr` (conikeec) angedeutet ist.16
    
- **Verwendung eines existierenden Transports:** Alternativ kann ein Client mit einem bereits existierenden Transportobjekt initialisiert werden.14
    
    Rust
    
    ```
    // use std::sync::Arc;
    // use mcp_client_rs::client::Client;
    // use mcp_client_rs::transport::stdio::StdioTransport;
    // use tokio::io::{stdin, stdout};
    //
    // let transport = StdioTransport::with_streams(stdin(), stdout());
    // let client = Client::new(Arc::new(transport));
    ```
    
- **`initialize`-Nachricht:** Nach dem Aufbau der Transportverbindung sendet der Client eine `initialize`-Anfrage an den Server, um Protokollversionen und Fähigkeiten auszutauschen.8 Die `spawn_and_initialize()`-Methode von `mcp_client_rs` (darinkishore) handhabt dies implizit.17 Die `initialize()`-Methode auf der Client-Instanz von `mcpr` (conikeec) führt dies explizit durch.16
    

Die `InitializeParams` [240 (angenommen)] würden typischerweise die `protocolVersion` (z.B. "2025-03-26"), `clientName`, `clientVersion` und `supportedFeatures` enthalten. Die `InitializeResult` [240 (angenommen)] vom Server bestätigt die `protocolVersion` und listet die `serverCapabilities` und `serverInfo` auf.

#### 5.2.2. Senden von Requests

MCP-Clients im NovaDE-Projekt senden Anfragen an Server, um Ressourcen aufzulisten, Werkzeuge aufzurufen oder andere definierte Operationen auszuführen.

- **`ListResources`:**
    - Parameter: `ListResourcesParams` [240 (angenommen)] können Filterkriterien oder Paginierungsinformationen enthalten.
    - Antwort: `ListResourcesResult` [240 (angenommen)] enthält eine Liste von `Resource`-Objekten [240 (angenommen)], die jeweils URI, Name, Beschreibung und unterstützte Operationen definieren.
    - Beispielaufruf mit `mcp_client_rs`: `let resources = client.list_resources().await?;` 18
- **`CallTool`:**
    - Parameter: `CallToolParams` [240 (angenommen)] enthalten den `toolName` (String) und `arguments` (JSON-Objekt).
    - Antwort: `CallToolResult` [240 (angenommen)] enthält das Ergebnis der Werkzeugausführung, typischerweise als JSON-Objekt.
    - Beispielaufruf mit `mcp_client_rs`: `let tool_result = client.call_tool("domain.action.calculateSum", serde_json::json!({"op1": 10, "op2": 20})).await?;` 18
    - Die Definition von Werkzeugen (`ToolDefinition` [240 (angenommen)]) umfasst Name, Beschreibung und ein JSON-Schema für die Parameter.
- **`ReadResource`:** (und andere domänenspezifische Requests)
    - Parameter: Typischerweise ein URI und optionale Parameter.
    - Antwort: Der Inhalt oder Zustand der Ressource.
    - Beispielaufruf mit `mcp_client_rs`: `let read_result = client.read_resource("nova://domain/entity/123").await?;` 18

Alle diese Anfragen werden asynchron über den konfigurierten Transportmechanismus gesendet. Die `mcp_client_rs` Bibliothek nutzt Tokio für diese asynchronen Operationen.25

#### 5.2.3. Empfangen von Responses und Notifications

Der Empfang von Nachrichten ist ein kritischer Aspekt der MCP-Client-Implementierung.

- **Responses:** Antworten auf Client-Anfragen werden typischerweise über `async/await` Konstrukte direkt als Rückgabewerte der aufrufenden Methoden empfangen (z.B. `ListResourcesResult` von `list_resources().await?`).18 Die zugrundeliegende Transportlogik (z.B. in `StdioTransport` [242 (angenommen), 244 (angenommen), 242 (angenommen)]) liest die Rohdaten, parst sie als `McpMessage` [240 (angenommen)] und leitet sie an den entsprechenden wartenden Task weiter.
- **Notifications (Server Push Events):** Asynchrone Benachrichtigungen vom Server (z.B. `documentChanged` aus dem Beispiel in Abschnitt 4.3.1) erfordern einen dedizierten Mechanismus zum Empfang und zur Verarbeitung.
    - Die `mcpr` Bibliothek (conikeec) deutet auf Unterstützung für Server-Sent Events (SSE) hin, was einen Stream von Ereignissen impliziert, den der Client verarbeiten müsste.16
    - Die `mcp_client_rs` Bibliothek (darinkishore) ist primär auf Stdio ausgerichtet. Die Handhabung von Server-Push-Benachrichtigungen über Stdio würde erfordern, dass der `StdioTransport` kontinuierlich die Standardeingabe liest und eingehende Nachrichten (die keine direkten Antworten auf Anfragen sind) als `Notification` [240 (angenommen)] identifiziert und an einen Handler oder einen Ereignis-Stream weiterleitet. Die genaue Implementierung (z.B. ein dedizierter Empfangs-Loop oder ein Stream von `McpMessage`) ist in den bereitgestellten Snippets nicht vollständig ersichtlich [17 (fehlend), 246 (fehlend), 241 (fehlend), 243 (fehlend), 243 (fehlend), 245 (fehlend), 246 (fehlend), 246 (fehlend)]. Es ist davon auszugehen, dass eine `async_stream`-basierte Lösung oder ein `tokio::sync::broadcast` Kanal 36 verwendet wird, um diese Nachrichten an interessierte Teile der Anwendung zu verteilen.
    - Die `mcp_client_rs` Version 0.1.1 erwähnt "WebSocket Transport (Coming Soon)" mit "built-in reconnection handling", was auf zukünftige robustere Mechanismen für Server-Push und Verbindungsstatus hindeutet.25
- **Connection Status Events:** Die Überwachung des Verbindungsstatus (z.B. Verbindungsabbruch, Wiederverbindung) ist für robuste Anwendungen wichtig. Explizite Mechanismen hierfür sind in den Snippets zu `mcp_client_rs` (darinkishore) nicht detailliert, könnten aber Teil des `WebSocketTransport` sein 25 oder müssten auf der Transportebene (z.B. durch Überwachung der Stdio-Pipes) implementiert werden. Für SSE-Transporte könnten HTTP-Fehlercodes oder das Schließen des Event-Streams als Indikatoren dienen.26

#### 5.2.4. Fehlerbehandlung

Fehler können auf verschiedenen Ebenen auftreten: Transportfehler, JSON-RPC-Parsingfehler, oder anwendungsspezifische Fehler, die vom Server als `ErrorResponse` [240 (angenommen)] gesendet werden.

- Die `mcp_client_rs` Bibliothek verwendet `thiserror` zur Definition ihres `Error`-Typs, der verschiedene Fehlerquellen aggregiert.27
- Client-Code sollte `Result`-Typen sorgfältig behandeln, um auf Fehler angemessen reagieren zu können (z.B. Wiederholungsversuche, Benutzerbenachrichtigung, Logging).
- Spezifische `ErrorCode`-Werte [240 (angenommen)] in `ErrorResponse`-Nachrichten ermöglichen eine differenzierte Fehlerbehandlung basierend auf der Art des serverseitigen Fehlers.

#### 5.2.5. Transport Layer

- **StdioTransport:** Für die Kommunikation mit lokalen Serverprozessen. Implementierungen in `mcpr` 16 und `mcp_client_rs` [25 (angenommen), 244 (angenommen), 242 (angenommen)] lesen von `stdin` und schreiben nach `stdout` des Subprozesses. Die `StdioTransport` in `mcp_client_rs` verwendet typischerweise `tokio::io::AsyncRead` und `tokio::io::AsyncWrite` für die asynchrone Verarbeitung. Eingehende Nachrichten werden zeilenbasiert oder durch Längenpräfixe (gemäß JSON-RPC Framing) gelesen und dann als `McpMessage` deserialisiert.
- **SSETransport (Server-Sent Events):** Für webbasierte oder Remote-Server. `mcpr` erwähnt dessen Unterstützung.16 Dies involviert einen HTTP-Client, der eine Verbindung zu einem SSE-Endpunkt des Servers herstellt und einen kontinuierlichen Stream von Ereignissen empfängt.26

### 5.3. MCP-Server-Implementierung (Rust)

Obwohl der primäre Fokus des NovaDE-Projekts auf der Client-Seite liegen mag, könnten bestimmte Komponenten des Projekts auch als MCP-Server fungieren, um Fähigkeiten für andere Teile des Systems oder externe LLMs bereitzustellen.

- **Struktur:** Ein MCP-Server in Rust, beispielsweise unter Verwendung der `mcpr`-Bibliothek 16, würde eine `ServerConfig` definieren, die Name, Version und eine Liste der bereitgestellten `Tool`-Definitionen enthält. Jedes `Tool` spezifiziert seinen Namen, eine Beschreibung und ein JSON-Schema für seine Parameter.
- **Tool-Registrierung:** Für jedes definierte Werkzeug wird ein Handler registriert, der die Werkzeugparameter entgegennimmt, die Logik ausführt und ein Ergebnis (oder einen Fehler) zurückgibt.
    
    Rust
    
    ```
    // Beispielhafte Server-Konfiguration mit mcpr (conikeec)
    // use mcpr::{server::{Server, ServerConfig}, transport::stdio::StdioTransport, Tool, error::MCPError};
    // use serde_json::Value;
    //
    // let server_config = ServerConfig::new()
    //    .with_name("NovaDE.DomainService.v1")
    //    .with_version("1.0.0")
    //    .with_tool(Tool {
    //         name: "nova.domain.getEntityDetails".to_string(),
    //         description: Some("Ruft Details zu einer Domänenentität ab.".to_string()),
    //         parameters_schema: serde_json::json!({
    //             "type": "object",
    //             "properties": {
    //                 "entityUri": {"type": "string", "description": "URI der Entität"}
    //             },
    //             "required": ["entityUri"]
    //         }),
    //     });
    // let mut server: Server<StdioTransport> = Server::new(server_config);
    // server.register_tool_handler("nova.domain.getEntityDetails", |params: Value| {
    //     // Implementierung der Domänenlogik hier
    //     //...
    //     Ok(serde_json::json!({"status": "success", "data": { /*... */ }}))
    // })?;
    // let transport = StdioTransport::new();
    // server.start(transport)?;
    ```
    

Die Serverimplementierung ist verantwortlich für das Parsen eingehender Anfragen, das Weiterleiten an die entsprechenden Handler und das Senden von Antworten oder Benachrichtigungen über den gewählten Transportmechanismus.

### 5.4. Interaktion mit Systemdiensten und Protokollen

Die MCP-Schnittstellen im NovaDE-Projekt sind nicht isoliert, sondern interagieren intensiv mit bestehenden Systemdiensten und Protokollen. Diese Interaktionen sind entscheidend für den Zugriff auf Systemressourcen, die Verwaltung von Berechtigungen und die Integration in die Desktop-Umgebung. Die folgenden Abschnitte detaillieren diese Interaktionen.

## 6. Interaktion mit D-Bus-Diensten

Die Kommunikation mit systemweiten und benutzerspezifischen Diensten im NovaDE-Projekt erfolgt primär über D-Bus. Die Rust-Bibliothek `zbus` wird für diese Interaktionen verwendet.12

### 6.1. Allgemeine D-Bus-Integration mit `zbus`

`zbus` ermöglicht eine typsichere und asynchrone Kommunikation mit D-Bus-Diensten.

- **Proxy-Generierung:** Für die Interaktion mit D-Bus-Schnittstellen werden Proxys verwendet. Das `#[dbus_proxy]` (oder `#[proxy]`) Makro von `zbus` generiert Rust-Traits und Proxy-Strukturen aus D-Bus-Interface-Definitionen [12 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich)].
    
    Rust
    
    ```
    // use zbus::{dbus_proxy, Connection, Result};
    // #
    // trait ExampleProxy {
    //     async fn some_method(&self, param: &str) -> Result<String>;
    //     #[dbus_proxy(signal)]
    //     async fn some_signal(&self, value: u32) -> Result<()>;
    // }
    ```
    
- **Verbindungsaufbau:** Eine Verbindung zum Session- oder Systembus wird mit `zbus::Connection::session().await?` bzw. `zbus::Connection::system().await?` hergestellt.45
- **Methodenaufrufe:** Methoden auf D-Bus-Schnittstellen werden asynchron über die generierten Proxy-Methoden aufgerufen.45
- **Signalempfang:** Signale werden als asynchrone Streams (`futures_util::stream::StreamExt`) über die Proxy-Methoden `receive_<signal_name>()` empfangen.46 Die Argumente des Signals können aus der `zbus::Message` deserialisiert werden [46 (nicht zugänglich), 65 (nicht zugänglich)].
- **Fehlerbehandlung:** `zbus`-Operationen geben `zbus::Result` zurück. Fehler werden durch das `zbus::Error`-Enum repräsentiert, das verschiedene Fehlerquellen wie I/O-Fehler, ungültige Nachrichten oder Fehler vom D-Bus-Dienst selbst abdeckt.52
- **Server-Implementierung:** `zbus` ermöglicht auch die Implementierung von D-Bus-Diensten. Mittels `ConnectionBuilder::serve_at` können Interfaces auf bestimmten Objektpfaden bereitgestellt werden [13 (nicht zugänglich), 57 (nicht zugänglich), 12 (nicht zugänglich), 56 (nicht zugänglich), 57 (nicht zugänglich), 12 (nicht zugänglich), 60]. Das `ObjectServer`-API kann für komplexere Szenarien mit mehreren Objekten und Interfaces auf derselben Verbindung genutzt werden [48 (nicht zugänglich), 57 (nicht zugänglich), 12 (nicht zugänglich), 56 (nicht zugänglich), 57 (nicht zugänglich), 58 (nicht zugänglich), 60].

### 6.2. `org.freedesktop.secrets` – Sichere Speicherung von Geheimnissen

Das NovaDE-Projekt nutzt die `org.freedesktop.Secrets`-Schnittstelle für die sichere Speicherung und Verwaltung von sensiblen Daten wie Passwörtern oder API-Tokens, die von MCP-Komponenten benötigt werden.70

- **Schnittstellenspezifikation:** 70
    - **`org.freedesktop.Secrets.Service`:** Verwalter von Collections und Sessions.
        - Methoden: `OpenSession`, `CreateCollection`, `SearchCollections`, `RetrieveSecrets`, `LockService`, `DeleteCollection`.
        - Signale: `CollectionCreated`, `CollectionDeleted`.
        - Properties: `Collections` (RO), `DefaultCollection` (RW).
    - **`org.freedesktop.Secrets.Collection`:** Eine Sammlung von Items (Geheimnissen).
        - Methoden: `Delete`, `SearchItems`, `CreateItem`.
        - Signale: `CreatedItem`, `DeletedItem`.
        - Properties: `Items` (RO), `Private` (RO), `Label` (RW), `Locked` (RO), `Created` (RO), `Modified` (RO).
    - **`org.freedesktop.Secrets.Item`:** Ein einzelnes Geheimnis mit Attributen.
        - Methoden: `Delete`.
        - Signale: `changed`.
        - Properties: `Locked` (RO), `Attributes` (RW), `Label` (RW), `Secret` (RW), `Created` (RO), `Modified` (RO).
    - **`org.freedesktop.Secrets.Session`:** Repräsentiert eine Sitzung zwischen Client und Dienst.
        - Methoden: `Close`, `Negotiate`, `BeginAuthenticate`, `CompleteAuthenticate`.
        - Signale: `Authenticated`.
- **Datentyp `Secret`:** 70
    - `algorithm` (String): Algorithmus zur Kodierung des Geheimnisses (z.B. "PLAIN").
    - `parameters` (Array<Byte>): Algorithmus-spezifische Parameter.
    - `value` (Array<Byte>): Der möglicherweise kodierte Geheimniswert.
- **Fehlerdomäne:** `org.freedesktop.Secrets.Error.*` (z.B. `AlreadyExists`, `IsLocked`, `NotSupported`).70
- **Nutzung im NovaDE-Projekt für MCP:**
    - MCP-Server oder -Clients im NovaDE-Projekt, die Zugriff auf persistente, sichere Anmeldeinformationen oder Tokens benötigen, interagieren mit diesem Dienst.
    - Beispiel: Ein MCP-Server, der eine Verbindung zu einem externen API herstellt, könnte das API-Token sicher über `org.freedesktop.Secrets` speichern und abrufen.
    - Die `CreateCollection`-Methode wird verwendet, um spezifische Sammlungen für NovaDE-Komponenten anzulegen, potenziell mit `private = true`, um den Zugriff einzuschränken.
    - `SearchItems` mit spezifischen `Attributes` wird verwendet, um gezielt nach Geheimnissen zu suchen.
    - Die `Negotiate`-Methode kann für eine verschlüsselte Übertragung der Geheimnisse genutzt werden, falls erforderlich.

### 6.3. `org.freedesktop.PolicyKit1.Authority` – Berechtigungsprüfung

PolicyKit wird im NovaDE-Projekt eingesetzt, um granulare Berechtigungsprüfungen für Aktionen durchzuführen, die von MCP-Schnittstellen oder den dahinterliegenden Diensten ausgelöst werden.71

- **Schnittstellenspezifikation:** `org.freedesktop.PolicyKit1.Authority` am Pfad `/org/freedesktop/PolicyKit1/Authority`.71
    - **Methoden:**
        - `CheckAuthorization(IN Subject subject, IN String action_id, IN Dict<String,String> details, IN CheckAuthorizationFlags flags, IN String cancellation_id, OUT AuthorizationResult result)`: Prüft, ob ein Subjekt berechtigt ist, eine Aktion auszuführen. `details` können verwendet werden, um kontextspezifische Informationen für die Autorisierungsentscheidung oder die dem Benutzer angezeigte Nachricht bereitzustellen. `AllowUserInteraction` im `flags`-Parameter ermöglicht eine interaktive Authentifizierung.
        - `CancelCheckAuthorization(IN String cancellation_id)`: Bricht eine laufende Autorisierungsprüfung ab.
        - `EnumerateActions(IN String locale, OUT Array<ActionDescription> action_descriptions)`: Listet alle registrierten Aktionen auf.
        - `RegisterAuthenticationAgent(IN Subject subject, IN String locale, IN String object_path)`: Registriert einen Authentifizierungsagenten.
    - **Signale:**
        - `Changed()`: Wird emittiert, wenn sich Aktionen oder Autorisierungen ändern.
- **Wichtige Datenstrukturen:** 71
    - `Subject`: Beschreibt das handelnde Subjekt (z.B. `unix-process`, `unix-session`).
    - `ActionDescription`: Beschreibt eine registrierte Aktion (ID, Beschreibung, Nachricht, Standardberechtigungen).
    - `AuthorizationResult`: Ergebnis der Prüfung (`is_authorized`, `is_challenge`, `details`).
- **Nutzung im NovaDE-Projekt für MCP-Zugriffssteuerung:**
    - Bevor eine MCP-Methode eine potenziell privilegierte Operation ausführt (z.B. Systemkonfiguration ändern, auf geschützte Benutzerdaten zugreifen), muss der MCP-Server oder die aufgerufene NovaDE-Komponente `CheckAuthorization` aufrufen.
    - Die `action_id` entspricht einer vordefinierten Aktion im NovaDE-PolicyKit-Regelwerk (z.B. `org.novade.mcp.filesystem.writeFile`).
    - Die `details` können MCP-spezifische Parameter enthalten, die für die Entscheidung oder die Benutzerinteraktion relevant sind.
    - Das Ergebnis von `CheckAuthorization` bestimmt, ob die MCP-Operation fortgesetzt oder mit einem Berechtigungsfehler abgelehnt wird.

### 6.4. XDG Desktop Portals

XDG Desktop Portals bieten sandboxed Anwendungen (und auch nicht-sandboxed Anwendungen) einen standardisierten Weg, um mit der Desktop-Umgebung zu interagieren, z.B. für Dateiauswahl oder Screenshots.6 MCP-Schnittstellen im NovaDE-Projekt können diese Portale nutzen, um solche Interaktionen sicher und konsistent zu gestalten.

#### 6.4.1. `org.freedesktop.portal.FileChooser`

Wird verwendet, um dem Benutzer Dialoge zur Datei- oder Ordnerauswahl anzuzeigen.6

- **Methoden:** 73
    - `OpenFile(IN String parent_window, IN String title, IN Dict<String,Variant> options, OUT ObjectPath handle)`: Öffnet einen Dialog zur Auswahl einer oder mehrerer Dateien.
        - Optionen (`a{sv}`): `handle_token` (s), `accept_label` (s), `modal` (b), `multiple` (b), `directory` (b), `filters` (`a(sa(us))`), `current_filter` (`(sa(us))`), `choices` (`a(ssa(ss)s)`), `current_folder` (ay).
        - Antwort über `org.freedesktop.portal.Request::Response`: `uris` (as), `choices` (`a(ss)`), `current_filter` (`(sa(us))`).
    - `SaveFile(IN String parent_window, IN String title, IN Dict<String,Variant> options, OUT ObjectPath handle)`: Öffnet einen Dialog zum Speichern einer Datei.
        - Optionen (`a{sv}`): Ähnlich wie `OpenFile`, zusätzlich `current_name` (s), `current_file` (ay).
        - Antwort über `org.freedesktop.portal.Request::Response`: `uris` (as), `choices` (`a(ss)`), `current_filter` (`(sa(us))`).
    - `SaveFiles(IN String parent_window, IN String title, IN Dict<String,Variant> options, OUT ObjectPath handle)`: Öffnet einen Dialog zum Speichern mehrerer Dateien in einem Ordner.
        - Optionen (`a{sv}`): Ähnlich wie `SaveFile`, zusätzlich `files` (aay).
        - Antwort über `org.freedesktop.portal.Request::Response`: `uris` (as), `choices` (`a(ss)`).
- **Nutzung im NovaDE-Projekt:** MCP-Methoden, die Dateiinteraktionen erfordern (z.B. das Hochladen eines Dokuments durch den Benutzer, das Speichern von generierten Inhalten), rufen diese Portalmethoden auf. Die `parent_window`-Kennung muss korrekt übergeben werden. Die `options` werden basierend auf dem Kontext der MCP-Aktion befüllt (z.B. Dateifilter basierend auf erwarteten MIME-Typen der Domänenschicht).

#### 6.4.2. `org.freedesktop.portal.Screenshot`

Ermöglicht das Erstellen von Screenshots und das Auswählen von Pixelfarben.3

- **Methoden (Version 2):** 79
    - `Screenshot(IN String parent_window, IN Dict<String,Variant> options, OUT ObjectPath handle)`: Erstellt einen Screenshot.
        - Optionen (`a{sv}`): `handle_token` (s), `modal` (b, default: true), `interactive` (b, default: false, seit v2).
        - Antwort über `org.freedesktop.portal.Request::Response`: `uri` (s) des Screenshots.
    - `PickColor(IN String parent_window, IN Dict<String,Variant> options, OUT ObjectPath handle)`: Wählt die Farbe eines Pixels aus.
        - Optionen (`a{sv}`): `handle_token` (s).
        - Antwort über `org.freedesktop.portal.Request::Response`: `color` (`(ddd)`) als RGB-Werte .
- **Nutzung im NovaDE-Projekt:** MCP-Schnittstellen, die visuelle Informationen vom Desktop benötigen (z.B. ein Werkzeug zur Fehlerberichterstattung, das einen Screenshot anhängt, oder ein Design-Tool, das Farben vom Bildschirm aufnimmt), verwenden diese Portalmethoden.
- **Backend-Implementierung:** Für Wayland-basierte Desktops wie potenziell NovaDE ist eine Backend-Implementierung wie `xdg-desktop-portal-wlr` 6, `xdg-desktop-portal-gnome` 6, `xdg-desktop-portal-kde` 6 oder eine spezifische Implementierung wie `xdg-desktop-portal-luminous` (Rust-basiert, nutzt `libwayshot` und `zwlr_screencopy`) 83 erforderlich. `xdg-desktop-portal-luminous` ist ein Beispiel für eine Rust-basierte Implementierung, die `zbus` für D-Bus verwenden könnte und über das `zwlr_screencopy`-Protokoll mit wlroots-basierten Compositors interagiert.88

### 6.5. `org.freedesktop.login1` – Sitzungs- und Benutzerverwaltung

Der `systemd-logind`-Dienst stellt über D-Bus Informationen und Kontrollfunktionen für Benutzersitzungen, Benutzer und Seats bereit [90 (nicht zugänglich), 253 (nicht zugänglich), 254 (nicht zugänglich)]. MCP-Komponenten im NovaDE-Projekt können diese Schnittstelle nutzen, um kontextbezogene Informationen über den aktuellen Benutzer oder die Sitzung zu erhalten oder um sitzungsbezogene Aktionen auszulösen.

- **Manager-Interface (`org.freedesktop.login1.Manager` auf `/org/freedesktop/login1`):** 90
    - **Methoden:**
        - `GetSession(IN String session_id, OUT ObjectPath object_path)`
        - `GetUser(IN UInt32 uid, OUT ObjectPath object_path)`
        - `ListSessions(OUT Array<(String session_id, UInt32 user_id, String user_name, String seat_id, ObjectPath session_path)> sessions)`
        - `LockSession(IN String session_id)`
        - `UnlockSession(IN String session_id)`
    - **Signale:**
        - `SessionNew(String session_id, ObjectPath object_path)` 91
        - `SessionRemoved(String session_id, ObjectPath object_path)` 91
        - `PrepareForSleep(Boolean start)` 90
- **Session-Interface (`org.freedesktop.login1.Session` auf dem jeweiligen Session-Pfad):**
    - **Signale:**
        - `Lock()` [90 (nicht zugänglich)]
        - `Unlock()` [90 (nicht zugänglich)]
- **Nutzung im NovaDE-Projekt für MCP-Kontext:**
    - Abfrage der aktuellen Sitzungs-ID oder Benutzer-ID, um MCP-Aktionen zu personalisieren oder Berechtigungen feingranularer zu steuern.
    - Reaktion auf `PrepareForSleep`-Signale, um z.B. laufende MCP-Operationen zu pausieren oder Daten zu sichern.
    - Auslösen von `LockSession` durch eine MCP-Methode, um den Bildschirm zu sperren.

### 6.6. `org.freedesktop.UPower` – Energieverwaltung

UPower liefert Informationen über Energiequellen und deren Status.92 Dies kann für MCP-Komponenten relevant sein, die ihr Verhalten an den Energiestatus anpassen müssen.

- **UPower-Interface (`org.freedesktop.UPower` auf `/org/freedesktop/UPower`):** 93
    - **Methoden:**
        - `EnumerateDevices(OUT Array<ObjectPath> devices)`
        - `GetDisplayDevice(OUT ObjectPath device)`
        - `GetCriticalAction(OUT String action)`
    - **Signale:**
        - `DeviceAdded(ObjectPath device)` [93 (nicht zugänglich)]
        - `DeviceRemoved(ObjectPath device)` [93 (nicht zugänglich)]
        - `DeviceChanged(ObjectPath device)` (impliziert durch `PropertiesChanged` auf Device-Objekt)
    - **Properties:**
        - `DaemonVersion` (String, RO)
        - `OnBattery` (Boolean, RO)
        - `LidIsClosed` (Boolean, RO)
        - `LidIsPresent` (Boolean, RO)
- **Device-Interface (`org.freedesktop.UPower.Device` auf dem jeweiligen Gerätepfad):** 93
    - **Properties (Auswahl):**
        - `Type` (UInt32, z.B. Battery, UPS, LinePower)
        - `State` (UInt32, z.B. Charging, Discharging, FullyCharged)
        - `Percentage` (Double)
        - `TimeToEmpty` (Int64, Sekunden)
        - `TimeToFull` (Int64, Sekunden)
        - `IsPresent` (Boolean)
        - `IconName` (String)
        - `WarningLevel` (UInt32)
    - **Signale:**
        - `PropertiesChanged` (via `org.freedesktop.DBus.Properties`) [257 (nicht zugänglich)]
- **Nutzung im NovaDE-Projekt für MCP-Kontext:**
    - MCP-Werkzeuge könnten den Batteriestatus abfragen (`OnBattery`, `Percentage`, `TimeToEmpty`), um langlaufende Operationen zu vermeiden oder den Benutzer zu warnen.
    - Anpassung des Verhaltens von NovaDE-Komponenten basierend auf dem Energiestatus (z.B. Reduzierung der Hintergrundaktivität bei niedrigem Akkustand).

### 6.7. `org.freedesktop.Notifications` – Desktop-Benachrichtigungen

Diese Schnittstelle ermöglicht es Anwendungen, Desktop-Benachrichtigungen anzuzeigen.95 MCP-Komponenten im NovaDE-Projekt können dies nutzen, um Benutzer über wichtige Ereignisse, den Abschluss von Aufgaben oder Fehler zu informieren.

- **Schnittstellenspezifikation (`org.freedesktop.Notifications` auf `/org/freedesktop/Notifications`):** 96
    - **Methoden:**
        - `Notify(IN String app_name, IN UInt32 replaces_id, IN String app_icon, IN String summary, IN String body, IN Array<String> actions, IN Dict<String,Variant> hints, IN Int32 expire_timeout, OUT UInt32 notification_id)`
        - `CloseNotification(IN UInt32 id)`
        - `GetCapabilities(OUT Array<String> capabilities)`
        - `GetServerInformation(OUT String name, OUT String vendor, OUT String version, OUT String spec_version)`
    - **Signale:**
        - `NotificationClosed(UInt32 id, UInt32 reason)`
        - `ActionInvoked(UInt32 id, String action_key)`
- **Wichtige Parameter und Hinweise:**
    - `actions`: Liste von Aktions-IDs und deren lesbaren Bezeichnungen.
    - `hints`: Diktionär für zusätzliche Hinweise (z.B. `urgency`, `sound-file`, `image-data`).
    - `expire_timeout`: `-1` für Server-Default, `0` für niemals auslaufend.
- **Nutzung im NovaDE-Projekt durch MCP:**
    - Ein MCP-Tool, das eine langlaufende Aufgabe abschließt, kann `Notify` aufrufen, um den Benutzer zu informieren.
    - Fehler, die in MCP-Operationen auftreten und Benutzereingriffe erfordern, können als Benachrichtigungen signalisiert werden.
    - Aktionen in Benachrichtigungen (`actions`-Parameter) können mit spezifischen MCP-Folgeaktionen im NovaDE-Client verknüpft werden.

## 7. Interaktion mit Wayland (Smithay)

Falls das NovaDE-Projekt einen eigenen Wayland-Compositor beinhaltet oder tief mit einem solchen interagiert (z.B. für spezifische Desktop-Umgebungsfeatures), kommt das Smithay-Framework zum Einsatz.10 Smithay ist eine Rust-Bibliothek zum Erstellen von Wayland-Compositoren.

### 7.1. Smithay Architekturüberblick

Smithay bietet Bausteine für Wayland-Compositoren und ist modular aufgebaut.10

- **Display und EventLoop:** Das Herzstück ist der `Display`-Typ (aus `wayland-server`) und ein `calloop::EventLoop`.98 `DisplayHandle` wird für Interaktionen mit dem Wayland-Protokoll verwendet [214 (nicht zugänglich)]. Der `EventLoopHandle` von `calloop` dient zur Verwaltung von Event-Quellen.234
- **State Management:** Ein zentraler `State`-Typ (z.B. `AnvilState` im Smithay-Beispielcompositor Anvil) hält den Zustand des Compositors [258 (nicht zugänglich), 124 (nicht zugänglich), 124 (nicht zugänglich), 98 (nicht zugänglich), 261 (nicht zugänglich), 262 (nicht zugänglich), 170 (nicht zugänglich)]. `ClientData` (oder `UserDataMap` auf Ressourcen) wird verwendet, um client-spezifischen Zustand zu speichern [98 (nicht zugänglich)].
- **Handler und Delegation:** Für verschiedene Wayland-Protokolle und -Objekte implementiert der `State`-Typ spezifische Handler-Traits (z.B. `CompositorHandler`, `ShmHandler`, `OutputHandler`, `SeatHandler`, `DataDeviceHandler`, `XdgShellHandler`, etc.). Smithay verwendet `delegate_<protocol>!` Makros, um die Dispatch-Logik zu vereinfachen [98 (nicht zugänglich), 136 (nicht zugänglich), 201 (nicht zugänglich), 205 (nicht zugänglich), 200 (nicht zugänglich), 200 (nicht zugänglich), 145 (nicht zugänglich), 222 (nicht zugänglich), 222 (nicht zugänglich), 200 (nicht zugänglich)].

### 7.2. Wayland Core Protokolle und ihre Handhabung durch MCP

#### 7.2.1. `wl_compositor`, `wl_subcompositor`, `wl_surface`, `wl_buffer`

Diese sind grundlegend für jede Wayland-Anzeige.

- **`CompositorState` und `CompositorHandler`:** Smithay stellt `CompositorState` zur Verwaltung von `wl_surface`-Objekten und deren Hierarchien (Subsurfaces) bereit.235 Der `CompositorHandler` im NovaDE-State muss implementiert werden, um auf Surface-Commits und -Zerstörungen zu reagieren.134 `SurfaceData` [263 (nicht zugänglich)] und `CompositorClientState` [201 (nicht zugänglich)] speichern oberflächen- bzw. clientbezogene Zustände. `SurfaceAttributes` enthalten Informationen wie die zugewiesene Rolle [123 (nicht zugänglich)].
- **`wl_buffer`:** Repräsentiert den Inhalt einer Surface. `BufferHandler` [145 (nicht zugänglich)] wird implementiert, um auf die Zerstörung von Buffern zu reagieren.
- **MCP-Interaktion:** MCP-Komponenten könnten indirekt mit diesen Objekten interagieren, z.B. wenn eine MCP-gesteuerte Anwendung eine UI auf dem Desktop darstellt. Die Spezifikation von Fenstergeometrien oder das Anfordern von Neuzeichnungen könnte über MCP-Methoden erfolgen, die dann auf die entsprechenden `wl_surface`-Operationen abgebildet werden.

#### 7.2.2. `wl_shm` – Shared Memory Buffers

Ermöglicht Clients, Buffer über Shared Memory bereitzustellen.

- **`ShmState` und `ShmHandler`:** `ShmState` verwaltet den `wl_shm`-Global und die unterstützten Formate. Der `ShmHandler` im NovaDE-State stellt den Zugriff auf `ShmState` sicher.136
- **Buffer-Import und Rendering:** `with_buffer_contents` erlaubt den Zugriff auf SHM-Buffer-Daten.145 Renderer wie `GlesRenderer` können SHM-Buffer importieren (`import_shm_buffer`) und rendern.171 MCP-Aktionen, die die Anzeige von Inhalten erfordern, die von einem Client als SHM-Buffer bereitgestellt werden, nutzen diese Mechanismen.

#### 7.2.3. `wl_output` und `xdg-output` – Output Management

Verwaltung von Bildschirmausgaben.

- **`Output` und `OutputHandler`:** Ein `Output`-Objekt repräsentiert eine physische Anzeige. `Output::new()` erstellt ein Output-Objekt, `Output::create_global()` macht es für Clients sichtbar [137 (nicht zugänglich), 137]. `Output::change_current_state()` aktualisiert Modus, Transformation, Skalierung und Position. Der `OutputHandler` im NovaDE-State behandelt clientseitige Interaktionen.101
- **`OutputManagerState`:** Kann verwendet werden, um `xdg-output` zusätzlich zu `wl_output` zu verwalten [137 (nicht zugänglich)].
- **MCP-Interaktion:** MCP-Methoden könnten es ermöglichen, Informationen über verfügbare Ausgaben abzurufen oder anwendungsspezifische Fenster auf bestimmten Ausgaben zu positionieren, basierend auf den von diesen Modulen bereitgestellten Informationen.

#### 7.2.4. `wl_seat`, `wl_keyboard`, `wl_pointer`, `wl_touch` – Input Handling

Verwaltung von Eingabegeräten und Fokus.

- **`SeatState` und `SeatHandler`:** `SeatState` verwaltet einen oder mehrere `Seat`-Instanzen. Der `SeatHandler` im NovaDE-State definiert, wie auf Eingabeereignisse und Fokusänderungen reagiert wird.113
- **Fokus-Management:** `KeyboardFocus`, `PointerFocus`, `TouchFocus` werden typischerweise auf `WlSurface` gesetzt, um den Eingabefokus zu lenken.113
- **Input Grabs:** Mechanismen wie `PointerGrab` und `KeyboardGrab` ermöglichen es, Eingabeereignisse exklusiv für eine bestimmte Oberfläche oder Aktion abzufangen [187 (nicht zugänglich)].
- **MCP-Interaktion:** MCP-gesteuerte Aktionen könnten den Fokus anfordern oder auf Eingabeereignisse reagieren, die über diese Seat-Abstraktionen verarbeitet werden. Beispielsweise könnte ein MCP-Tool eine Texteingabe erfordern, was das Setzen des Tastaturfokus auf ein entsprechendes Eingabefeld des MCP-Clients zur Folge hätte.


---

# Ultra-Feinspezifikation der MCP-Schnittstellen und Implementierungen für das NovaDE-Projekt

## 1. Einleitung

### 1.1. Zweck des Dokuments

Dieses Dokument definiert die Ultra-Feinspezifikation aller Model Context Protocol (MCP) Schnittstellen und deren Implementierungen innerhalb des NovaDE-Projekts. Es dient als maßgebliche technische Referenz für die Entwicklung, Integration und Wartung von MCP-basierten Komponenten im NovaDE-Ökosystem. Die Spezifikation umfasst detaillierte Beschreibungen von Nachrichtenformaten, Datenstrukturen, Methoden, Ereignissen und Fehlerbehandlungsmechanismen. Ein besonderer Fokus liegt auf der Integration der Domänenschicht-Spezifikation des NovaDE-Projekts in die MCP-Schnittstellen.

### 1.2. Geltungsbereich

Diese Spezifikation bezieht sich auf sämtliche Aspekte des Model Context Protocol, wie es im Kontext des NovaDE-Projekts eingesetzt wird. Dies beinhaltet:

- Alle MCP-Schnittstellen, die im NovaDE-Projekt definiert oder genutzt werden.
- Die Interaktion dieser MCP-Schnittstellen mit anderen Systemkomponenten, einschließlich, aber nicht beschränkt auf D-Bus-Dienste, Wayland-Protokolle und PipeWire-Audio-Management.
- Implementierungsrichtlinien und -details, insbesondere unter Verwendung der Programmiersprache Rust und assoziierter Bibliotheken.
- Die nahtlose Einbindung der fachlichen Anforderungen und Datenmodelle aus der Domänenschicht-Spezifikation des NovaDE-Projekts.

### 1.3. Zielgruppe

Dieses Dokument richtet sich an folgende Personengruppen innerhalb des NovaDE-Projekts:

- Softwarearchitekten und -entwickler, die MCP-Schnittstellen und -Komponenten entwerfen, implementieren oder nutzen.
- Systemintegratoren, die für die Bereitstellung und Konfiguration von NovaDE-Systemen verantwortlich sind.
- Qualitätssicherungsingenieure, die MCP-Funktionalitäten testen.
- Technische Projektmanager, die die Entwicklung und Implementierung des NovaDE-Projekts überwachen.

### 1.4. Definitionen und Akronyme

- **MCP:** Model Context Protocol. Ein offener Standard zur Kommunikation zwischen KI-Modellen/Anwendungen und externen Werkzeugen oder Datenquellen.1
- **NovaDE-Projekt:** Das spezifische Projekt, für das diese MCP-Spezifikation erstellt wird. (Details zum Projekt selbst sind außerhalb des Geltungsbereichs der bereitgestellten Materialien).
- **Domänenschicht-Spezifikation:** Ein separates Dokument, das die fachlichen Entitäten, Geschäftsregeln und Datenmodelle des NovaDE-Projekts beschreibt. Diese Spezifikation wird als integraler Bestandteil der MCP-Schnittstellendefinitionen betrachtet.
- **API:** Application Programming Interface.
- **D-Bus:** Desktop Bus, ein System für Interprozesskommunikation (IPC).3
- **Wayland:** Ein Kommunikationsprotokoll zwischen einem Display-Server (Compositor) und seinen Clients.4
- **PipeWire:** Ein Multimedia-Framework für Audio- und Videoverarbeitung unter Linux.5
- **XDG Desktop Portals:** Ein Framework, das sandboxed Anwendungen den sicheren Zugriff auf Ressourcen außerhalb der Sandbox ermöglicht.6
- **JSON-RPC:** JavaScript Object Notation Remote Procedure Call. Ein leichtgewichtiges RPC-Protokoll.8
- **Stdio:** Standard Input/Output.
- **SSE:** Server-Sent Events. Eine Technologie, die es einem Server ermöglicht, Updates an einen Client über eine HTTP-Verbindung zu pushen.8
- **Smithay:** Eine Rust-Bibliothek zur Erstellung von Wayland-Compositoren.10
- **zbus:** Eine Rust-Bibliothek für die D-Bus-Kommunikation.12
- **pipewire-rs:** Rust-Bindungen für PipeWire.14
- **mcpr:** Eine Rust-Implementierung des Model Context Protocol.16
- **mcp_client_rs:** Eine weitere Rust-Client-SDK für MCP.17

### 1.5. Referenzierte Dokumente

- Model Context Protocol Specification (Version 2025-03-26 oder aktueller) 2
- Domänenschicht-Spezifikation des NovaDE-Projekts (externes Dokument)
- Freedesktop D-Bus Specification 3
- Wayland Protocol Specification 4
- PipeWire Documentation 5
- XDG Desktop Portal Documentation 6
- Spezifikationen der relevanten D-Bus-Schnittstellen (Secrets, PolicyKit, Portals, Login1, UPower, Notifications)
- Spezifikationen der relevanten Wayland-Protokolle und -Erweiterungen
- Dokumentation der verwendeten Rust-Bibliotheken (Smithay, zbus, pipewire-rs, mcpr, mcp_client_rs, tokio, serde, thiserror etc.)

## 2. Model Context Protocol (MCP) – Grundlagen

### 2.1. Überblick und Kernkonzepte

Das Model Context Protocol (MCP) ist ein offener Standard, der darauf abzielt, die Integration von Large Language Models (LLMs) mit externen Werkzeugen, Datenbanken und APIs zu standardisieren.1 Es fungiert als eine universelle Schnittstelle, die es KI-Modellen ermöglicht, dynamisch auf Kontextinformationen zuzugreifen und Aktionen in ihrer Umgebung auszuführen.9 MCP adressiert die Herausforderung der Informationssilos und proprietären Integrationen, indem es einen einheitlichen Rahmen für die KI-Tool-Kommunikation schafft.1

Die Kernprinzipien von MCP umfassen 2:

- **Standardisierte Schnittstelle:** Einheitliche Methoden für LLMs zum Zugriff auf Werkzeuge und Ressourcen.
- **Erweiterte Fähigkeiten:** Befähigung von LLMs zur Interaktion mit diversen Systemen.
- **Sicherheit und Kontrolle:** Strukturierte Zugriffsmuster mit integrierter Validierung und klaren Grenzen.
- **Modularität und Erweiterbarkeit:** Einfaches Hinzufügen neuer Fähigkeiten durch Server, ohne die Kernanwendung des LLMs modifizieren zu müssen.

MCP ist darauf ausgelegt, die Reproduzierbarkeit von KI-Interaktionen zu verbessern, indem der gesamte notwendige Kontext (Datensätze, Umgebungsspezifikationen, Hyperparameter) an einem Ort verwaltet wird.1

### 2.2. Architektur (Client-Host-Server-Modell)

MCP basiert auf einer Client-Host-Server-Architektur 8:

- **Host:** Eine LLM-Anwendung (z.B. Claude Desktop, IDEs), die Verbindungen initiiert und als Container oder Koordinator für mehrere Client-Instanzen fungiert. Der Host verwaltet den Lebenszyklus, Sicherheitsrichtlinien (Berechtigungen, Benutzerautorisierung) und die Integration des LLMs.1
- **Client:** Eine Protokoll-Client-Komponente innerhalb der Host-Anwendung, die eine 1:1-Verbindung zu einem MCP-Server herstellt. Der Client ist verantwortlich für die Aushandlung von Fähigkeiten und die Orchestrierung von Nachrichten zwischen sich und dem Server.1
- **Server:** Ein Dienst (oft ein leichtgewichtiger Prozess), der spezifische Kontexte, Werkzeuge und Prompts für den Client bereitstellt. Server können lokale Prozesse oder entfernte Dienste sein und kapseln den Zugriff auf Datenquellen, APIs oder andere Utilities.1

Diese Architektur ermöglicht eine klare Trennung der Verantwortlichkeiten und fördert die Entwicklung modularer und wiederverwendbarer MCP-Server.23 Die Kommunikation zwischen diesen Komponenten erfolgt über eine Transportschicht und eine Protokollschicht, die auf JSON-RPC aufbaut und zustandsbehaftete Sitzungen für den Kontextaustausch und das Sampling betont.1

### 2.3. Nachrichtenformate (JSON-RPC 2.0 Basis)

MCP verwendet JSON-RPC 2.0 als Grundlage für seine Nachrichtenformate.8 Dies gewährleistet eine strukturierte und standardisierte Kommunikation. Die Hauptnachrichtentypen sind 8:

- **Requests (Anfragen):** Vom Client oder Server gesendete Nachrichten, die eine Antwort erwarten. Sie enthalten typischerweise eine `method` (Methodenname) und optionale `params` (Parameter).
    - Beispiel: `{"jsonrpc": "2.0", "method": "tools/list", "id": 1}`
- **Responses (Antworten):** Erfolgreiche Antworten auf Requests. Sie enthalten ein `result`-Feld mit den Ergebnisdaten und die `id` des ursprünglichen Requests.
    - Beispiel: `{"jsonrpc": "2.0", "result": {"tools": [...]}, "id": 1}`
- **Error Responses (Fehlerantworten):** Antworten, die anzeigen, dass ein Request fehlgeschlagen ist. Sie enthalten ein `error`-Objekt mit `code`, `message` und optional `data`, sowie die `id` des ursprünglichen Requests.
    - Beispiel: `{"jsonrpc": "2.0", "error": {"code": -32601, "message": "Method not found"}, "id": 1}`
- **Notifications (Benachrichtigungen):** Einwegnachrichten, die keine Antwort erwarten. Sie enthalten eine `method` und optionale `params`, aber keine `id`.
    - Beispiel: `{"jsonrpc": "2.0", "method": "textDocument/didChange", "params": {...}}`

Die spezifischen Methoden und Parameter für MCP-Nachrichten wie `initialize`, `tools/list`, `resources/read`, `tools/call` werden im weiteren Verlauf dieses Dokuments detailliert [2 (angenommen)].

### 2.4. Transportmechanismen

MCP unterstützt verschiedene Transportmechanismen für die Kommunikation zwischen Host/Client und Server 8:

- **Stdio (Standard Input/Output):** Dieser Mechanismus wird für die Kommunikation mit lokalen Prozessen verwendet. Der MCP-Server läuft als separater Prozess, und die Kommunikation erfolgt über dessen Standard-Eingabe- und Ausgabe-Streams. Dies ist ideal für Kommandozeilenwerkzeuge und lokale Entwicklungsszenarien.16 Die Rust-Bibliothek `mcpr` bietet beispielsweise `StdioTransport` 16, und `mcp_client_rs` fokussiert sich ebenfalls auf diesen Transportweg für lokal gespawnte Server.18
- **HTTP mit SSE (Server-Sent Events):** Dieser Mechanismus wird für netzwerkbasierte Kommunikation verwendet, insbesondere wenn der Server remote ist oder Echtzeit-Updates vom Server an den Client erforderlich sind. SSE ermöglicht es dem Server, asynchron Nachrichten an den Client zu pushen, während Client-zu-Server-Nachrichten typischerweise über HTTP POST gesendet werden.8 Die `mcpr`-Bibliothek erwähnt SSE-Transportunterstützung.16

Die Wahl des Transportmechanismus hängt von den spezifischen Anforderungen der NovaDE-Komponente ab, insbesondere davon, ob der MCP-Server lokal oder remote betrieben wird.

### 2.5. Sicherheitsaspekte

Sicherheit und Datenschutz sind zentrale Aspekte des Model Context Protocol, da es potenziell den Zugriff auf sensible Daten und die Ausführung von Code ermöglicht.2 Die Spezifikation legt folgende Schlüsselprinzipien fest 2:

- **Benutzereinwilligung und -kontrolle:**
    - Benutzer müssen explizit allen Datenzugriffen und Operationen zustimmen und diese verstehen.
    - Benutzer müssen die Kontrolle darüber behalten, welche Daten geteilt und welche Aktionen ausgeführt werden.
    - Implementierungen sollten klare Benutzeroberflächen zur Überprüfung und Autorisierung von Aktivitäten bereitstellen.
- **Datenschutz:**
    - Hosts müssen die explizite Zustimmung des Benutzers einholen, bevor Benutzerdaten an Server weitergegeben werden.
    - Hosts dürfen Ressourcendaten nicht ohne Zustimmung des Benutzers an andere Stellen übertragen.
    - Benutzerdaten sollten durch geeignete Zugriffskontrollen geschützt werden.
- **Werkzeugsicherheit (Tool Safety):**
    - Werkzeuge repräsentieren die Ausführung von beliebigem Code und müssen mit entsprechender Vorsicht behandelt werden. Beschreibungen des Werkzeugverhaltens (z.B. Annotationen) sind als nicht vertrauenswürdig zu betrachten, es sei denn, sie stammen von einem vertrauenswürdigen Server.
    - Hosts müssen die explizite Zustimmung des Benutzers einholen, bevor ein Werkzeug aufgerufen wird.
    - Benutzer sollten verstehen, was jedes Werkzeug tut, bevor sie dessen Verwendung autorisieren.
- **LLM Sampling Controls:**
    - Benutzer müssen explizit allen LLM-Sampling-Anfragen zustimmen.
    - Benutzer sollten kontrollieren, ob Sampling überhaupt stattfindet, den tatsächlichen Prompt, der gesendet wird, und welche Ergebnisse der Server sehen kann.

Obwohl MCP diese Prinzipien nicht auf Protokollebene erzwingen kann, **SOLLTEN** Implementierer robuste Zustimmungs- und Autorisierungsflüsse entwickeln, Sicherheitsimplikationen klar dokumentieren, geeignete Zugriffskontrollen und Datenschutzmaßnahmen implementieren und bewährte Sicherheitspraktiken befolgen.2 Die Architektur mit MCP-Servern als Vermittler kann eine zusätzliche Sicherheitsebene bieten, indem der Zugriff auf Ressourcen kontrolliert und potenziell in einer Sandbox ausgeführt wird.19

## 3. MCP-Schnittstellen im NovaDE-Projekt – Allgemeine Spezifikation

### 3.1. Namenskonventionen und Versionierung

Für alle MCP-Schnittstellen, die im Rahmen des NovaDE-Projekts definiert werden, gelten folgende Namenskonventionen und Versionierungsrichtlinien:

- **Schnittstellennamen:** Schnittstellennamen folgen dem Muster `nova.<KomponentenName>.<Funktionsbereich>.<Version>`. Beispiel: `nova.workspace.fileAccess.v1`. Dies gewährleistet Eindeutigkeit und Klarheit über den Ursprung und Zweck der Schnittstelle.
- **Methodennamen:** Methodennamen verwenden camelCase, beginnend mit einem Kleinbuchstaben (z.B. `listResources`, `callTool`).
- **Parameternamen:** Parameternamen verwenden ebenfalls camelCase.
- **Versionierung:** Jede MCP-Schnittstelle wird explizit versioniert. Die Version wird als Teil des Schnittstellennamens geführt (z.B. `v1`, `v2`). Änderungen, die die Abwärtskompatibilität brechen, erfordern eine Erhöhung der Hauptversionsnummer. Abwärtskompatible Erweiterungen können zu einer Erhöhung einer Nebenversionsnummer führen, falls ein solches Schema zusätzlich eingeführt wird. Das NovaDE-Projekt hält sich an die im MCP-Standard definierte Protokollversion (z.B. `2025-03-26`).2 Die aktuell unterstützte MCP-Protokollversion ist im `mcp_client_rs` Crate als `LATEST_PROTOCOL_VERSION` und `SUPPORTED_PROTOCOL_VERSIONS` definiert.27

### 3.2. Standardnachrichtenflüsse

Die Kommunikation im NovaDE-Projekt über MCP folgt etablierten Nachrichtenflüssen, die auf dem JSON-RPC 2.0 Standard basieren.8

1. **Initialisierung (Connection Lifecycle):** 8
    - Der MCP-Client (innerhalb des NovaDE-Hosts) sendet eine `initialize`-Anfrage an den MCP-Server. Diese Anfrage enthält die vom Client unterstützte Protokollversion und dessen Fähigkeiten (Capabilities).
    - Der MCP-Server antwortet mit seiner Protokollversion und seinen Fähigkeiten.
    - Der Client bestätigt die erfolgreiche Initialisierung mit einer `initialized`-Notification.
    - Anschließend beginnt der reguläre Nachrichtenaustausch.
2. **Anfrage-Antwort (Request-Response):** 8
    - Der Client sendet eine Anfrage (z.B. `tools/list`, `resources/read`, `tools/call`) mit einer eindeutigen ID.
    - Der Server verarbeitet die Anfrage und sendet entweder eine Erfolgsantwort mit dem Ergebnis (`result`) und derselben ID oder eine Fehlerantwort (`error`) mit Fehlercode, Nachricht und derselben ID.
3. **Benachrichtigungen (Notifications):** 8
    - Client oder Server können einseitige Benachrichtigungen senden, die keine direkte Antwort erwarten. Diese haben keine ID. Ein Beispiel ist die `initialized`-Notification oder serverseitige Push-Events.
4. **Beendigung (Termination):** 8
    - Die Verbindung kann durch eine `shutdown`-Anfrage vom Client initiiert werden, gefolgt von einer `exit`-Notification. Alternativ kann die Verbindung durch Schließen des zugrundeliegenden Transportkanals beendet werden.

Die Rust-Bibliotheken `mcpr` und `mcp_client_rs` implementieren diese grundlegenden Nachrichtenflüsse.16 `mcp_client_rs` beispielsweise nutzt Tokio für asynchrone Operationen und stellt Methoden wie `initialize()`, `list_resources()`, `call_tool()` zur Verfügung, die diesen Flüssen folgen.18

### 3.3. Fehlerbehandlung und Fehlercodes

Eine robuste Fehlerbehandlung ist entscheidend für die Stabilität der MCP-Kommunikation im NovaDE-Projekt. MCP-Fehlerantworten folgen dem JSON-RPC 2.0 Standard 8 und enthalten ein `error`-Objekt mit den Feldern `code` (Integer), `message` (String) und optional `data` (beliebiger Typ).

**Standard-Fehlercodes (basierend auf JSON-RPC 2.0):**

- `-32700 Parse error`: Ungültiges JSON wurde empfangen.
- `-32600 Invalid Request`: Die JSON-Anfrage war nicht wohlgeformt.
- `-32601 Method not found`: Die angeforderte Methode existiert nicht oder ist nicht verfügbar.
- `-32602 Invalid params`: Ungültige Methodenparameter.
- `-32603 Internal error`: Interner JSON-RPC-Fehler.
- `-32000` bis `-32099 Server error`: Reserviert für implementierungsspezifische Serverfehler.

NovaDE-spezifische Fehlercodes:

Zusätzlich zu den Standard-JSON-RPC-Fehlercodes definiert das NovaDE-Projekt spezifische Fehlercodes im Bereich -32000 bis -32099 für anwendungsspezifische Fehler, die während der Verarbeitung von MCP-Anfragen auftreten können. Diese Fehlercodes werden pro Schnittstelle und Methode dokumentiert.

Fehlerbehandlung in Rust-Implementierungen:

In Rust-basierten MCP-Implementierungen für NovaDE wird die Verwendung von thiserror für Bibliotheksfehler und potenziell anyhow für Anwendungsfehler empfohlen, um eine klare und kontextreiche Fehlerbehandlung zu gewährleisten.29 Die mcp_client_rs Bibliothek stellt einen Error-Typ bereit, der verschiedene Fehlerquellen kapselt.27 Die Struktur ErrorResponse und das Enum ErrorCode [240 (angenommen)] sind Teil der Protokolldefinitionen zur strukturierten Fehlerkommunikation.

**Beispiel für eine Fehlerantwort:**

JSON

```
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "NovaDE Domain Error: Ressource nicht gefunden.",
    "data": {
      "resourceUri": "nova://domain/entity/123"
    }
  },
  "id": 123
}
```

### 3.4. Integration der Domänenschicht-Spezifikation

Die Domänenschicht-Spezifikation des NovaDE-Projekts ist ein zentrales Element, das die fachlichen Entitäten, Operationen und Geschäftsregeln definiert. Die MCP-Schnittstellen im NovaDE-Projekt müssen diese Domänenspezifikation nahtlos integrieren. Dies bedeutet:

- **Abbildung von Domänenentitäten:** Datenstrukturen innerhalb der MCP-Nachrichten (Parameter von Methoden, Rückgabewerte, Event-Payloads) müssen die Entitäten der Domänenschicht widerspiegeln oder direkt verwenden. Dies stellt sicher, dass die MCP-Kommunikation die fachlichen Anforderungen korrekt abbildet.
- **Domänenoperationen als MCP-Methoden:** Viele MCP-Methoden werden direkte Abbildungen von Operationen sein, die in der Domänenschicht definiert sind. Die Parameter und Rückgabewerte dieser MCP-Methoden korrespondieren mit den Ein- und Ausgaben der Domänenoperationen.
- **Validierung und Geschäftsregeln:** Bevor MCP-Anfragen an die Domänenschicht weitergeleitet oder Ergebnisse von der Domänenschicht über MCP zurückgegeben werden, müssen Validierungen und Geschäftsregeln der Domänenschicht angewendet werden. Dies kann sowohl im MCP-Server als auch in einer zwischengeschalteten Logikschicht geschehen.
- **Konsistente Terminologie:** Die in den MCP-Schnittstellen verwendete Terminologie (Namen von Methoden, Parametern, Datenfeldern) sollte mit der Terminologie der Domänenschicht-Spezifikation übereinstimmen, um Missverständnisse zu vermeiden und die Kohärenz im gesamten System zu fördern.

Die genauen Details der Integration hängen von den spezifischen Inhalten der Domänenschicht-Spezifikation ab. Jede detaillierte MCP-Schnittstellenspezifikation in Abschnitt 4 wird explizit auf die relevanten Teile der Domänenschicht-Spezifikation verweisen und die Abbildung erläutern.

## 4. Detaillierte MCP-Schnittstellenspezifikationen

Für das NovaDE-Projekt werden spezifische MCP-Schnittstellen definiert, um die Interaktion mit verschiedenen Modulen und Diensten zu ermöglichen. Jede Schnittstelle wird nach dem folgenden Schema spezifiziert. _Da die konkreten Schnittstellen für NovaDE nicht in den bereitgestellten Materialien definiert sind, dient der folgende Abschnitt als Vorlage und Beispielstruktur._

---

**Beispiel-Schnittstelle: `nova.dataAccess.document.v1`**

### 4.1. Beschreibung und Zweck

Die Schnittstelle `nova.dataAccess.document.v1` dient dem Zugriff auf und der Verwaltung von Dokumenten innerhalb des NovaDE-Projekts. Sie ermöglicht es MCP-Clients, Dokumente basierend auf Kriterien der Domänenschicht zu lesen, zu schreiben, zu aktualisieren und zu löschen. Diese Schnittstelle interagiert eng mit den Entitäten und Operationen, die in der "Domänenschicht-Spezifikation, Abschnitt X.Y (Dokumentenverwaltung)" definiert sind.

### 4.2. Methoden

#### 4.2.1. `readDocument`

- **Beschreibung:** Liest den Inhalt und die Metadaten eines spezifischen Dokuments.
- **Parameter:**
    - `uri` (String, erforderlich): Der eindeutige URI des Dokuments, konform zum NovaDE-URI-Schema (z.B. `nova://documents/internal/doc123`).
    - `options` (Object, optional): Zusätzliche Optionen für den Lesevorgang.
        - `version` (String, optional): Die spezifische Version des Dokuments, die gelesen werden soll. Falls nicht angegeben, wird die neueste Version gelesen.
- **Rückgabewerte:**
    - `document` (Object): Ein Objekt, das das gelesene Dokument repräsentiert. Die Struktur dieses Objekts ist in der Domänenschicht-Spezifikation definiert und könnte Felder wie `uri`, `mimeType`, `content` (String oder Binary), `metadata` (Object), `version` (String), `lastModified` (Timestamp) enthalten.
- **Mögliche Fehler:**
    - `-32001`: `DOCUMENT_NOT_FOUND` - Das angeforderte Dokument existiert nicht.
    - `-32002`: `ACCESS_DENIED` - Der Client hat keine Berechtigung, das Dokument zu lesen.
    - `-32003`: `VERSION_NOT_FOUND` - Die angeforderte Version des Dokuments existiert nicht.

#### 4.2.2. `writeDocument`

- **Beschreibung:** Schreibt ein neues Dokument oder aktualisiert ein bestehendes Dokument.
- **Parameter:**
    - `uri` (String, erforderlich): Der URI, unter dem das Dokument geschrieben werden soll. Bei Aktualisierung eines bestehenden Dokuments dessen URI.
    - `content` (String oder Binary, erforderlich): Der Inhalt des Dokuments. Der Typ (String oder Base64-kodiertes Binary) hängt vom `mimeType` ab.
    - `mimeType` (String, erforderlich): Der MIME-Typ des Dokuments (z.B. `text/plain`, `application/pdf`).
    - `metadata` (Object, optional): Domänenspezifische Metadaten für das Dokument.
    - `options` (Object, optional):
        - `overwrite` (Boolean, optional, default: `false`): Wenn `true` und ein Dokument unter dem URI existiert, wird es überschrieben. Andernfalls schlägt der Aufruf fehl, wenn das Dokument existiert.
- **Rückgabewerte:**
    - `newUri` (String): Der URI des geschriebenen oder aktualisierten Dokuments (kann sich bei Neuerstellung ändern, falls der Server URIs generiert).
    - `version` (String): Die Versionskennung des geschriebenen Dokuments.
- **Mögliche Fehler:**
    - `-32002`: `ACCESS_DENIED` - Keine Schreibberechtigung.
    - `-32004`: `DOCUMENT_EXISTS` - Dokument existiert bereits und `overwrite` ist `false`.
    - `-32005`: `INVALID_CONTENT` - Der bereitgestellte Inhalt ist für den `mimeType` ungültig.

_(Weitere Methoden wie `deleteDocument`, `listDocuments` würden hier analog spezifiziert werden.)_

### 4.3. Events/Notifications

#### 4.3.1. `documentChanged`

- **Beschreibung:** Wird vom Server gesendet, wenn ein Dokument, für das der Client möglicherweise Interesse bekundet hat (z.B. durch vorheriges Lesen), geändert wurde.
- **Parameter:**
    - `uri` (String): Der URI des geänderten Dokuments.
    - `changeType` (String): Art der Änderung (z.B. `UPDATED`, `DELETED`).
    - `newVersion` (String, optional): Die neue Versionskennung, falls `changeType` `UPDATED` ist.

### 4.4. Datenstrukturen

Die für diese Schnittstelle relevanten Datenstrukturen (z.B. die Struktur eines `Document`-Objekts, `Metadata`-Objekts) werden primär durch die Domänenschicht-Spezifikation des NovaDE-Projekts definiert. MCP-Nachrichten verwenden JSON-Repräsentationen dieser domänenspezifischen Strukturen.

**Beispiel `Document` (basierend auf einer hypothetischen Domänenspezifikation):**

JSON

```
{
  "uri": "nova://documents/internal/doc123",
  "mimeType": "text/plain",
  "content": "Dies ist der Inhalt des Dokuments.",
  "metadata": {
    "author": "NovaUser",
    "tags": ["wichtig", "projektA"],
    "customDomainField": "spezifischerWert"
  },
  "version": "v1.2.3",
  "lastModified": "2024-07-15T10:30:00Z"
}
```

### 4.5. Beispiele für Nachrichten

**Anfrage `readDocument`:**

JSON

```
{
  "jsonrpc": "2.0",
  "method": "nova.dataAccess.document.v1/readDocument",
  "params": {
    "uri": "nova://documents/internal/doc123"
  },
  "id": 1
}
```

**Antwort `readDocument` (Erfolg):**

JSON

```
{
  "jsonrpc": "2.0",
  "result": {
    "document": {
      "uri": "nova://documents/internal/doc123",
      "mimeType": "text/plain",
      "content": "Dies ist der Inhalt des Dokuments.",
      "metadata": {"author": "NovaUser"},
      "version": "v1.0.0",
      "lastModified": "2024-07-15T10:00:00Z"
    }
  },
  "id": 1
}
```

### 4.6. Interaktion mit der Domänenschicht

Die Methode `readDocument` ruft intern die Funktion `DomainLayer.getDocumentByUri(uri, options.version)` der Domänenschicht auf. Die zurückgegebenen Domänenobjekte werden gemäß den MCP-Datenstrukturen serialisiert. Die Methode `writeDocument` validiert die Eingaben anhand der Geschäftsregeln der Domänenschicht (z.B. `DomainLayer.validateDocumentContent(content, mimeType)`) und ruft dann `DomainLayer.saveDocument(documentData)` auf. Berechtigungsprüfungen erfolgen ebenfalls über dedizierte Domänenschicht-Services (z.B. `DomainLayer.Security.canReadDocument(userContext, uri)`).

---

_(Dieser beispielhafte Abschnitt würde für jede spezifische MCP-Schnittstelle im NovaDE-Projekt wiederholt werden.)_

## 5. Implementierung der MCP-Schnittstellen im NovaDE-Projekt

### 5.1. Verwendete Technologien

Die Kernimplementierung der MCP-Schnittstellen und der zugehörigen Logik im NovaDE-Projekt erfolgt in **Rust**. Dies schließt sowohl Client- als auch (potenzielle) Server-seitige Komponenten ein. Die Wahl von Rust begründet sich in dessen Stärken hinsichtlich Systemsicherheit, Performance und Nebenläufigkeit, welche für ein robustes Desktop Environment Projekt wie NovaDE essentiell sind.

Folgende Rust-Bibliotheken (Crates) sind für die MCP-Implementierung von zentraler Bedeutung:

- **MCP-Protokoll-Handling:**
    - `mcp_client_rs` (von darinkishore) [17 (angenommen), 241 (angenommen), 28 (angenommen), 243 (angenommen), 244 (angenommen), 243 (angenommen), 242 (angenommen), 245 (angenommen), 246 (angenommen), 246 (angenommen)] oder alternativ `mcpr` (von conikeec) 16 für die Client-seitige Implementierung. Die Entscheidung für eine spezifische Bibliothek hängt von den detaillierten Anforderungen und der Reife der jeweiligen Bibliothek zum Zeitpunkt der Implementierung ab. Beide bieten Mechanismen zur Serialisierung/Deserialisierung von MCP-Nachrichten und zur Verwaltung der Kommunikation.
- **Asynchrone Laufzeitumgebung:** `tokio` wird als primäre asynchrone Laufzeitumgebung für die nebenläufige Verarbeitung von MCP-Nachrichten und Interaktionen mit anderen Systemdiensten verwendet.25
- **Serialisierung/Deserialisierung:** `serde` und `serde_json` für die Umwandlung von Rust-Datenstrukturen in und aus dem JSON-Format, das von JSON-RPC verwendet wird.25
- **Fehlerbehandlung:** `thiserror` für die Definition von benutzerdefinierten Fehlertypen in Bibliotheks-Code und potenziell `anyhow` für eine vereinfachte Fehlerbehandlung in Anwendungscode.29
- **UUID-Generierung:** Das `uuid` Crate mit den Features `v4` und `serde` wird für die Erzeugung und Handhabung von eindeutigen Identifikatoren verwendet, die in MCP-Nachrichten oder domänenspezifischen Daten benötigt werden könnten.41
- **D-Bus-Kommunikation:** `zbus` für die Interaktion mit Systemdiensten über D-Bus.12
- **Wayland Compositing (falls NovaDE ein Compositor ist oder tief integriert):** `smithay` als Framework für Wayland-spezifische Interaktionen.10
- **PipeWire-Integration:** `pipewire-rs` für die Interaktion mit dem PipeWire Multimedia-Framework.14

### 5.2. MCP-Client-Implementierung (Rust)

Die MCP-Client-Komponenten im NovaDE-Projekt sind für die Kommunikation mit verschiedenen MCP-Servern zuständig, die Werkzeuge und Ressourcen bereitstellen.

#### 5.2.1. Initialisierung und Verbindungsaufbau

Die Initialisierung eines MCP-Clients beginnt mit der Konfiguration des Transports und der Erstellung einer Client-Instanz. Am Beispiel von `mcp_client_rs` (darinkishore):

- **Server-Spawning (für lokale Server via Stdio):** Die `ClientBuilder`-API ermöglicht das Starten eines lokalen MCP-Serverprozesses und die Verbindung zu dessen Stdio-Kanälen.17
    
    Rust
    
    ```
    // Beispielhafte Initialisierung (Pseudocode, da Servername und Argumente spezifisch für NovaDE sind)
    // use mcp_client_rs::client::ClientBuilder;
    // let client = ClientBuilder::new("nova-mcp-server-executable")
    //    .arg("--config-path")
    //    .arg("/etc/nova/mcp_server_config.json")
    //    .spawn_and_initialize().await?;
    ```
    
    Es ist wichtig zu beachten, dass `mcp_client_rs` (darinkishore) primär für lokal gespawnte Server konzipiert ist und keine direkte Unterstützung für Remote-Server plant.17 Für Remote-Verbindungen via HTTP/SSE müsste eine andere Bibliothek oder eine Erweiterung dieses Ansatzes in Betracht gezogen werden, wie sie z.B. in `mcpr` (conikeec) angedeutet ist.16
    
- **Verwendung eines existierenden Transports:** Alternativ kann ein Client mit einem bereits existierenden Transportobjekt initialisiert werden.14
    
    Rust
    
    ```
    // use std::sync::Arc;
    // use mcp_client_rs::client::Client;
    // use mcp_client_rs::transport::stdio::StdioTransport;
    // use tokio::io::{stdin, stdout};
    //
    // let transport = StdioTransport::with_streams(stdin(), stdout());
    // let client = Client::new(Arc::new(transport));
    ```
    
- **`initialize`-Nachricht:** Nach dem Aufbau der Transportverbindung sendet der Client eine `initialize`-Anfrage an den Server, um Protokollversionen und Fähigkeiten auszutauschen.8 Die `spawn_and_initialize()`-Methode von `mcp_client_rs` (darinkishore) handhabt dies implizit.17 Die `initialize()`-Methode auf der Client-Instanz von `mcpr` (conikeec) führt dies explizit durch.16
    

Die `InitializeParams` [240 (angenommen)] würden typischerweise die `protocolVersion` (z.B. "2025-03-26"), `clientName`, `clientVersion` und `supportedFeatures` enthalten. Die `InitializeResult` [240 (angenommen)] vom Server bestätigt die `protocolVersion` und listet die `serverCapabilities` und `serverInfo` auf.

#### 5.2.2. Senden von Requests

MCP-Clients im NovaDE-Projekt senden Anfragen an Server, um Ressourcen aufzulisten, Werkzeuge aufzurufen oder andere definierte Operationen auszuführen.

- **`ListResources`:**
    - Parameter: `ListResourcesParams` [240 (angenommen)] können Filterkriterien oder Paginierungsinformationen enthalten.
    - Antwort: `ListResourcesResult` [240 (angenommen)] enthält eine Liste von `Resource`-Objekten [240 (angenommen)], die jeweils URI, Name, Beschreibung und unterstützte Operationen definieren.
    - Beispielaufruf mit `mcp_client_rs`: `let resources = client.list_resources().await?;` 18
- **`CallTool`:**
    - Parameter: `CallToolParams` [240 (angenommen)] enthalten den `toolName` (String) und `arguments` (JSON-Objekt).
    - Antwort: `CallToolResult` [240 (angenommen)] enthält das Ergebnis der Werkzeugausführung, typischerweise als JSON-Objekt.
    - Beispielaufruf mit `mcp_client_rs`: `let tool_result = client.call_tool("domain.action.calculateSum", serde_json::json!({"op1": 10, "op2": 20})).await?;` 18
    - Die Definition von Werkzeugen (`ToolDefinition` [240 (angenommen)]) umfasst Name, Beschreibung und ein JSON-Schema für die Parameter.
- **`ReadResource`:** (und andere domänenspezifische Requests)
    - Parameter: Typischerweise ein URI und optionale Parameter.
    - Antwort: Der Inhalt oder Zustand der Ressource.
    - Beispielaufruf mit `mcp_client_rs`: `let read_result = client.read_resource("nova://domain/entity/123").await?;` 18

Alle diese Anfragen werden asynchron über den konfigurierten Transportmechanismus gesendet. Die `mcp_client_rs` Bibliothek nutzt Tokio für diese asynchronen Operationen.25

#### 5.2.3. Empfangen von Responses und Notifications

Der Empfang von Nachrichten ist ein kritischer Aspekt der MCP-Client-Implementierung.

- **Responses:** Antworten auf Client-Anfragen werden typischerweise über `async/await` Konstrukte direkt als Rückgabewerte der aufrufenden Methoden empfangen (z.B. `ListResourcesResult` von `list_resources().await?`).18 Die zugrundeliegende Transportlogik (z.B. in `StdioTransport` [242 (angenommen), 244 (angenommen), 242 (angenommen)]) liest die Rohdaten, parst sie als `McpMessage` [240 (angenommen)] und leitet sie an den entsprechenden wartenden Task weiter.
- **Notifications (Server Push Events):** Asynchrone Benachrichtigungen vom Server (z.B. `documentChanged` aus dem Beispiel in Abschnitt 4.3.1) erfordern einen dedizierten Mechanismus zum Empfang und zur Verarbeitung.
    - Die `mcpr` Bibliothek (conikeec) deutet auf Unterstützung für Server-Sent Events (SSE) hin, was einen Stream von Ereignissen impliziert, den der Client verarbeiten müsste.16
    - Die `mcp_client_rs` Bibliothek (darinkishore) ist primär auf Stdio ausgerichtet. Die Handhabung von Server-Push-Benachrichtigungen über Stdio würde erfordern, dass der `StdioTransport` kontinuierlich die Standardeingabe liest und eingehende Nachrichten (die keine direkten Antworten auf Anfragen sind) als `Notification` [240 (angenommen)] identifiziert und an einen Handler oder einen Ereignis-Stream weiterleitet. Die genaue Implementierung (z.B. ein dedizierter Empfangs-Loop oder ein Stream von `McpMessage`) ist in den bereitgestellten Snippets nicht vollständig ersichtlich [17 (fehlend), 246 (fehlend), 241 (fehlend), 243 (fehlend), 243 (fehlend), 245 (fehlend), 246 (fehlend), 246 (fehlend)]. Es ist davon auszugehen, dass eine `async_stream`-basierte Lösung oder ein `tokio::sync::broadcast` Kanal 36 verwendet wird, um diese Nachrichten an interessierte Teile der Anwendung zu verteilen.
    - Die `mcp_client_rs` Version 0.1.1 erwähnt "WebSocket Transport (Coming Soon)" mit "built-in reconnection handling", was auf zukünftige robustere Mechanismen für Server-Push und Verbindungsstatus hindeutet.25
- **Connection Status Events:** Die Überwachung des Verbindungsstatus (z.B. Verbindungsabbruch, Wiederverbindung) ist für robuste Anwendungen wichtig. Explizite Mechanismen hierfür sind in den Snippets zu `mcp_client_rs` (darinkishore) nicht detailliert, könnten aber Teil des `WebSocketTransport` sein 25 oder müssten auf der Transportebene (z.B. durch Überwachung der Stdio-Pipes) implementiert werden. Für SSE-Transporte könnten HTTP-Fehlercodes oder das Schließen des Event-Streams als Indikatoren dienen.26

#### 5.2.4. Fehlerbehandlung

Fehler können auf verschiedenen Ebenen auftreten: Transportfehler, JSON-RPC-Parsingfehler, oder anwendungsspezifische Fehler, die vom Server als `ErrorResponse` [240 (angenommen)] gesendet werden.

- Die `mcp_client_rs` Bibliothek verwendet `thiserror` zur Definition ihres `Error`-Typs, der verschiedene Fehlerquellen aggregiert.27
- Client-Code sollte `Result`-Typen sorgfältig behandeln, um auf Fehler angemessen reagieren zu können (z.B. Wiederholungsversuche, Benutzerbenachrichtigung, Logging).
- Spezifische `ErrorCode`-Werte [240 (angenommen)] in `ErrorResponse`-Nachrichten ermöglichen eine differenzierte Fehlerbehandlung basierend auf der Art des serverseitigen Fehlers.

#### 5.2.5. Transport Layer

- **StdioTransport:** Für die Kommunikation mit lokalen Serverprozessen. Implementierungen in `mcpr` 16 und `mcp_client_rs` [25 (angenommen), 244 (angenommen), 242 (angenommen)] lesen von `stdin` und schreiben nach `stdout` des Subprozesses. Die `StdioTransport` in `mcp_client_rs` verwendet typischerweise `tokio::io::AsyncRead` und `tokio::io::AsyncWrite` für die asynchrone Verarbeitung. Eingehende Nachrichten werden zeilenbasiert oder durch Längenpräfixe (gemäß JSON-RPC Framing) gelesen und dann als `McpMessage` deserialisiert.
- **SSETransport (Server-Sent Events):** Für webbasierte oder Remote-Server. `mcpr` erwähnt dessen Unterstützung.16 Dies involviert einen HTTP-Client, der eine Verbindung zu einem SSE-Endpunkt des Servers herstellt und einen kontinuierlichen Stream von Ereignissen empfängt.26

### 5.3. MCP-Server-Implementierung (Rust)

Obwohl der primäre Fokus des NovaDE-Projekts auf der Client-Seite liegen mag, könnten bestimmte Komponenten des Projekts auch als MCP-Server fungieren, um Fähigkeiten für andere Teile des Systems oder externe LLMs bereitzustellen.

- **Struktur:** Ein MCP-Server in Rust, beispielsweise unter Verwendung der `mcpr`-Bibliothek 16, würde eine `ServerConfig` definieren, die Name, Version und eine Liste der bereitgestellten `Tool`-Definitionen enthält. Jedes `Tool` spezifiziert seinen Namen, eine Beschreibung und ein JSON-Schema für seine Parameter.
- **Tool-Registrierung:** Für jedes definierte Werkzeug wird ein Handler registriert, der die Werkzeugparameter entgegennimmt, die Logik ausführt und ein Ergebnis (oder einen Fehler) zurückgibt.
    
    Rust
    
    ```
    // Beispielhafte Server-Konfiguration mit mcpr (conikeec)
    // use mcpr::{server::{Server, ServerConfig}, transport::stdio::StdioTransport, Tool, error::MCPError};
    // use serde_json::Value;
    //
    // let server_config = ServerConfig::new()
    //    .with_name("NovaDE.DomainService.v1")
    //    .with_version("1.0.0")
    //    .with_tool(Tool {
    //         name: "nova.domain.getEntityDetails".to_string(),
    //         description: Some("Ruft Details zu einer Domänenentität ab.".to_string()),
    //         parameters_schema: serde_json::json!({
    //             "type": "object",
    //             "properties": {
    //                 "entityUri": {"type": "string", "description": "URI der Entität"}
    //             },
    //             "required": ["entityUri"]
    //         }),
    //     });
    // let mut server: Server<StdioTransport> = Server::new(server_config);
    // server.register_tool_handler("nova.domain.getEntityDetails", |params: Value| {
    //     // Implementierung der Domänenlogik hier
    //     //...
    //     Ok(serde_json::json!({"status": "success", "data": { /*... */ }}))
    // })?;
    // let transport = StdioTransport::new();
    // server.start(transport)?;
    ```
    

Die Serverimplementierung ist verantwortlich für das Parsen eingehender Anfragen, das Weiterleiten an die entsprechenden Handler und das Senden von Antworten oder Benachrichtigungen über den gewählten Transportmechanismus.

### 5.4. Interaktion mit Systemdiensten und Protokollen

Die MCP-Schnittstellen im NovaDE-Projekt sind nicht isoliert, sondern interagieren intensiv mit bestehenden Systemdiensten und Protokollen. Diese Interaktionen sind entscheidend für den Zugriff auf Systemressourcen, die Verwaltung von Berechtigungen und die Integration in die Desktop-Umgebung. Die folgenden Abschnitte detaillieren diese Interaktionen.

## 6. Interaktion mit D-Bus-Diensten

Die Kommunikation mit systemweiten und benutzerspezifischen Diensten im NovaDE-Projekt erfolgt primär über D-Bus. Die Rust-Bibliothek `zbus` wird für diese Interaktionen verwendet.12

### 6.1. Allgemeine D-Bus-Integration mit `zbus`

`zbus` ermöglicht eine typsichere und asynchrone Kommunikation mit D-Bus-Diensten.

- **Proxy-Generierung:** Für die Interaktion mit D-Bus-Schnittstellen werden Proxys verwendet. Das `#[dbus_proxy]` (oder `#[proxy]`) Makro von `zbus` generiert Rust-Traits und Proxy-Strukturen aus D-Bus-Interface-Definitionen [12 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich), 62 (nicht zugänglich)].
    
    Rust
    
    ```
    // use zbus::{dbus_proxy, Connection, Result};
    // #
    // trait ExampleProxy {
    //     async fn some_method(&self, param: &str) -> Result<String>;
    //     #[dbus_proxy(signal)]
    //     async fn some_signal(&self, value: u32) -> Result<()>;
    // }
    ```
    
- **Verbindungsaufbau:** Eine Verbindung zum Session- oder Systembus wird mit `zbus::Connection::session().await?` bzw. `zbus::Connection::system().await?` hergestellt.45
- **Methodenaufrufe:** Methoden auf D-Bus-Schnittstellen werden asynchron über die generierten Proxy-Methoden aufgerufen.45
- **Signalempfang:** Signale werden als asynchrone Streams (`futures_util::stream::StreamExt`) über die Proxy-Methoden `receive_<signal_name>()` empfangen.46 Die Argumente des Signals können aus der `zbus::Message` deserialisiert werden [46 (nicht zugänglich), 65 (nicht zugänglich)].
- **Fehlerbehandlung:** `zbus`-Operationen geben `zbus::Result` zurück. Fehler werden durch das `zbus::Error`-Enum repräsentiert, das verschiedene Fehlerquellen wie I/O-Fehler, ungültige Nachrichten oder Fehler vom D-Bus-Dienst selbst abdeckt.52
- **Server-Implementierung:** `zbus` ermöglicht auch die Implementierung von D-Bus-Diensten. Mittels `ConnectionBuilder::serve_at` können Interfaces auf bestimmten Objektpfaden bereitgestellt werden [13 (nicht zugänglich), 57 (nicht zugänglich), 12 (nicht zugänglich), 56 (nicht zugänglich), 57 (nicht zugänglich), 12 (nicht zugänglich), 60]. Das `ObjectServer`-API kann für komplexere Szenarien mit mehreren Objekten und Interfaces auf derselben Verbindung genutzt werden [48 (nicht zugänglich), 57 (nicht zugänglich), 12 (nicht zugänglich), 56 (nicht zugänglich), 57 (nicht zugänglich), 58 (nicht zugänglich), 60].

### 6.2. `org.freedesktop.secrets` – Sichere Speicherung von Geheimnissen

Das NovaDE-Projekt nutzt die `org.freedesktop.Secrets`-Schnittstelle für die sichere Speicherung und Verwaltung von sensiblen Daten wie Passwörtern oder API-Tokens, die von MCP-Komponenten benötigt werden.70

- **Schnittstellenspezifikation:** 70
    - **`org.freedesktop.Secrets.Service`:** Verwalter von Collections und Sessions.
        - Methoden: `OpenSession`, `CreateCollection`, `SearchCollections`, `RetrieveSecrets`, `LockService`, `DeleteCollection`.
        - Signale: `CollectionCreated`, `CollectionDeleted`.
        - Properties: `Collections` (RO), `DefaultCollection` (RW).
    - **`org.freedesktop.Secrets.Collection`:** Eine Sammlung von Items (Geheimnissen).
        - Methoden: `Delete`, `SearchItems`, `CreateItem`.
        - Signale: `CreatedItem`, `DeletedItem`.
        - Properties: `Items` (RO), `Private` (RO), `Label` (RW), `Locked` (RO), `Created` (RO), `Modified` (RO).
    - **`org.freedesktop.Secrets.Item`:** Ein einzelnes Geheimnis mit Attributen.
        - Methoden: `Delete`.
        - Signale: `changed`.
        - Properties: `Locked` (RO), `Attributes` (RW), `Label` (RW), `Secret` (RW), `Created` (RO), `Modified` (RO).
    - **`org.freedesktop.Secrets.Session`:** Repräsentiert eine Sitzung zwischen Client und Dienst.
        - Methoden: `Close`, `Negotiate`, `BeginAuthenticate`, `CompleteAuthenticate`.
        - Signale: `Authenticated`.
- **Datentyp `Secret`:** 70
    - `algorithm` (String): Algorithmus zur Kodierung des Geheimnisses (z.B. "PLAIN").
    - `parameters` (Array<Byte>): Algorithmus-spezifische Parameter.
    - `value` (Array<Byte>): Der möglicherweise kodierte Geheimniswert.
- **Fehlerdomäne:** `org.freedesktop.Secrets.Error.*` (z.B. `AlreadyExists`, `IsLocked`, `NotSupported`).70
- **Nutzung im NovaDE-Projekt für MCP:**
    - MCP-Server oder -Clients im NovaDE-Projekt, die Zugriff auf persistente, sichere Anmeldeinformationen oder Tokens benötigen, interagieren mit diesem Dienst.
    - Beispiel: Ein MCP-Server, der eine Verbindung zu einem externen API herstellt, könnte das API-Token sicher über `org.freedesktop.Secrets` speichern und abrufen.
    - Die `CreateCollection`-Methode wird verwendet, um spezifische Sammlungen für NovaDE-Komponenten anzulegen, potenziell mit `private = true`, um den Zugriff einzuschränken.
    - `SearchItems` mit spezifischen `Attributes` wird verwendet, um gezielt nach Geheimnissen zu suchen.
    - Die `Negotiate`-Methode kann für eine verschlüsselte Übertragung der Geheimnisse genutzt werden, falls erforderlich.

### 6.3. `org.freedesktop.PolicyKit1.Authority` – Berechtigungsprüfung

PolicyKit wird im NovaDE-Projekt eingesetzt, um granulare Berechtigungsprüfungen für Aktionen durchzuführen, die von MCP-Schnittstellen oder den dahinterliegenden Diensten ausgelöst werden.71

- **Schnittstellenspezifikation:** `org.freedesktop.PolicyKit1.Authority` am Pfad `/org/freedesktop/PolicyKit1/Authority`.71
    - **Methoden:**
        - `CheckAuthorization(IN Subject subject, IN String action_id, IN Dict<String,String> details, IN CheckAuthorizationFlags flags, IN String cancellation_id, OUT AuthorizationResult result)`: Prüft, ob ein Subjekt berechtigt ist, eine Aktion auszuführen. `details` können verwendet werden, um kontextspezifische Informationen für die Autorisierungsentscheidung oder die dem Benutzer angezeigte Nachricht bereitzustellen. `AllowUserInteraction` im `flags`-Parameter ermöglicht eine interaktive Authentifizierung.
        - `CancelCheckAuthorization(IN String cancellation_id)`: Bricht eine laufende Autorisierungsprüfung ab.
        - `EnumerateActions(IN String locale, OUT Array<ActionDescription> action_descriptions)`: Listet alle registrierten Aktionen auf.
        - `RegisterAuthenticationAgent(IN Subject subject, IN String locale, IN String object_path)`: Registriert einen Authentifizierungsagenten.
    - **Signale:**
        - `Changed()`: Wird emittiert, wenn sich Aktionen oder Autorisierungen ändern.
- **Wichtige Datenstrukturen:** 71
    - `Subject`: Beschreibt das handelnde Subjekt (z.B. `unix-process`, `unix-session`).
    - `ActionDescription`: Beschreibt eine registrierte Aktion (ID, Beschreibung, Nachricht, Standardberechtigungen).
    - `AuthorizationResult`: Ergebnis der Prüfung (`is_authorized`, `is_challenge`, `details`).
- **Nutzung im NovaDE-Projekt für MCP-Zugriffssteuerung:**
    - Bevor eine MCP-Methode eine potenziell privilegierte Operation ausführt (z.B. Systemkonfiguration ändern, auf geschützte Benutzerdaten zugreifen), muss der MCP-Server oder die aufgerufene NovaDE-Komponente `CheckAuthorization` aufrufen.
    - Die `action_id` entspricht einer vordefinierten Aktion im NovaDE-PolicyKit-Regelwerk (z.B. `org.novade.mcp.filesystem.writeFile`).
    - Die `details` können MCP-spezifische Parameter enthalten, die für die Entscheidung oder die Benutzerinteraktion relevant sind.
    - Das Ergebnis von `CheckAuthorization` bestimmt, ob die MCP-Operation fortgesetzt oder mit einem Berechtigungsfehler abgelehnt wird.

### 6.4. XDG Desktop Portals

XDG Desktop Portals bieten sandboxed Anwendungen (und auch nicht-sandboxed Anwendungen) einen standardisierten Weg, um mit der Desktop-Umgebung zu interagieren, z.B. für Dateiauswahl oder Screenshots.6 MCP-Schnittstellen im NovaDE-Projekt können diese Portale nutzen, um solche Interaktionen sicher und konsistent zu gestalten.

#### 6.4.1. `org.freedesktop.portal.FileChooser`

Wird verwendet, um dem Benutzer Dialoge zur Datei- oder Ordnerauswahl anzuzeigen.6

- **Methoden:** 73
    - `OpenFile(IN String parent_window, IN String title, IN Dict<String,Variant> options, OUT ObjectPath handle)`: Öffnet einen Dialog zur Auswahl einer oder mehrerer Dateien.
        - Optionen (`a{sv}`): `handle_token` (s), `accept_label` (s), `modal` (b), `multiple` (b), `directory` (b), `filters` (`a(sa(us))`), `current_filter` (`(sa(us))`), `choices` (`a(ssa(ss)s)`), `current_folder` (ay).
        - Antwort über `org.freedesktop.portal.Request::Response`: `uris` (as), `choices` (`a(ss)`), `current_filter` (`(sa(us))`).
    - `SaveFile(IN String parent_window, IN String title, IN Dict<String,Variant> options, OUT ObjectPath handle)`: Öffnet einen Dialog zum Speichern einer Datei.
        - Optionen (`a{sv}`): Ähnlich wie `OpenFile`, zusätzlich `current_name` (s), `current_file` (ay).
        - Antwort über `org.freedesktop.portal.Request::Response`: `uris` (as), `choices` (`a(ss)`), `current_filter` (`(sa(us))`).
    - `SaveFiles(IN String parent_window, IN String title, IN Dict<String,Variant> options, OUT ObjectPath handle)`: Öffnet einen Dialog zum Speichern mehrerer Dateien in einem Ordner.
        - Optionen (`a{sv}`): Ähnlich wie `SaveFile`, zusätzlich `files` (aay).
        - Antwort über `org.freedesktop.portal.Request::Response`: `uris` (as), `choices` (`a(ss)`).
- **Nutzung im NovaDE-Projekt:** MCP-Methoden, die Dateiinteraktionen erfordern (z.B. das Hochladen eines Dokuments durch den Benutzer, das Speichern von generierten Inhalten), rufen diese Portalmethoden auf. Die `parent_window`-Kennung muss korrekt übergeben werden. Die `options` werden basierend auf dem Kontext der MCP-Aktion befüllt (z.B. Dateifilter basierend auf erwarteten MIME-Typen der Domänenschicht).

#### 6.4.2. `org.freedesktop.portal.Screenshot`

Ermöglicht das Erstellen von Screenshots und das Auswählen von Pixelfarben.3

- **Methoden (Version 2):** 79
    - `Screenshot(IN String parent_window, IN Dict<String,Variant> options, OUT ObjectPath handle)`: Erstellt einen Screenshot.
        - Optionen (`a{sv}`): `handle_token` (s), `modal` (b, default: true), `interactive` (b, default: false, seit v2).
        - Antwort über `org.freedesktop.portal.Request::Response`: `uri` (s) des Screenshots.
    - `PickColor(IN String parent_window, IN Dict<String,Variant> options, OUT ObjectPath handle)`: Wählt die Farbe eines Pixels aus.
        - Optionen (`a{sv}`): `handle_token` (s).
        - Antwort über `org.freedesktop.portal.Request::Response`: `color` (`(ddd)`) als RGB-Werte .
- **Nutzung im NovaDE-Projekt:** MCP-Schnittstellen, die visuelle Informationen vom Desktop benötigen (z.B. ein Werkzeug zur Fehlerberichterstattung, das einen Screenshot anhängt, oder ein Design-Tool, das Farben vom Bildschirm aufnimmt), verwenden diese Portalmethoden.
- **Backend-Implementierung:** Für Wayland-basierte Desktops wie potenziell NovaDE ist eine Backend-Implementierung wie `xdg-desktop-portal-wlr` 6, `xdg-desktop-portal-gnome` 6, `xdg-desktop-portal-kde` 6 oder eine spezifische Implementierung wie `xdg-desktop-portal-luminous` (Rust-basiert, nutzt `libwayshot` und `zwlr_screencopy`) 83 erforderlich. `xdg-desktop-portal-luminous` ist ein Beispiel für eine Rust-basierte Implementierung, die `zbus` für D-Bus verwenden könnte und über das `zwlr_screencopy`-Protokoll mit wlroots-basierten Compositors interagiert.88

### 6.5. `org.freedesktop.login1` – Sitzungs- und Benutzerverwaltung

Der `systemd-logind`-Dienst stellt über D-Bus Informationen und Kontrollfunktionen für Benutzersitzungen, Benutzer und Seats bereit [90 (nicht zugänglich), 253 (nicht zugänglich), 254 (nicht zugänglich)]. MCP-Komponenten im NovaDE-Projekt können diese Schnittstelle nutzen, um kontextbezogene Informationen über den aktuellen Benutzer oder die Sitzung zu erhalten oder um sitzungsbezogene Aktionen auszulösen.

- **Manager-Interface (`org.freedesktop.login1.Manager` auf `/org/freedesktop/login1`):** 90
    - **Methoden:**
        - `GetSession(IN String session_id, OUT ObjectPath object_path)`
        - `GetUser(IN UInt32 uid, OUT ObjectPath object_path)`
        - `ListSessions(OUT Array<(String session_id, UInt32 user_id, String user_name, String seat_id, ObjectPath session_path)> sessions)`
        - `LockSession(IN String session_id)`
        - `UnlockSession(IN String session_id)`
    - **Signale:**
        - `SessionNew(String session_id, ObjectPath object_path)` 91
        - `SessionRemoved(String session_id, ObjectPath object_path)` 91
        - `PrepareForSleep(Boolean start)` 90
- **Session-Interface (`org.freedesktop.login1.Session` auf dem jeweiligen Session-Pfad):**
    - **Signale:**
        - `Lock()` [90 (nicht zugänglich)]
        - `Unlock()` [90 (nicht zugänglich)]
- **Nutzung im NovaDE-Projekt für MCP-Kontext:**
    - Abfrage der aktuellen Sitzungs-ID oder Benutzer-ID, um MCP-Aktionen zu personalisieren oder Berechtigungen feingranularer zu steuern.
    - Reaktion auf `PrepareForSleep`-Signale, um z.B. laufende MCP-Operationen zu pausieren oder Daten zu sichern.
    - Auslösen von `LockSession` durch eine MCP-Methode, um den Bildschirm zu sperren.

### 6.6. `org.freedesktop.UPower` – Energieverwaltung

UPower liefert Informationen über Energiequellen und deren Status.92 Dies kann für MCP-Komponenten relevant sein, die ihr Verhalten an den Energiestatus anpassen müssen.

- **UPower-Interface (`org.freedesktop.UPower` auf `/org/freedesktop/UPower`):** 93
    - **Methoden:**
        - `EnumerateDevices(OUT Array<ObjectPath> devices)`
        - `GetDisplayDevice(OUT ObjectPath device)`
        - `GetCriticalAction(OUT String action)`
    - **Signale:**
        - `DeviceAdded(ObjectPath device)` [93 (nicht zugänglich)]
        - `DeviceRemoved(ObjectPath device)` [93 (nicht zugänglich)]
        - `DeviceChanged(ObjectPath device)` (impliziert durch `PropertiesChanged` auf Device-Objekt)
    - **Properties:**
        - `DaemonVersion` (String, RO)
        - `OnBattery` (Boolean, RO)
        - `LidIsClosed` (Boolean, RO)
        - `LidIsPresent` (Boolean, RO)
- **Device-Interface (`org.freedesktop.UPower.Device` auf dem jeweiligen Gerätepfad):** 93
    - **Properties (Auswahl):**
        - `Type` (UInt32, z.B. Battery, UPS, LinePower)
        - `State` (UInt32, z.B. Charging, Discharging, FullyCharged)
        - `Percentage` (Double)
        - `TimeToEmpty` (Int64, Sekunden)
        - `TimeToFull` (Int64, Sekunden)
        - `IsPresent` (Boolean)
        - `IconName` (String)
        - `WarningLevel` (UInt32)
    - **Signale:**
        - `PropertiesChanged` (via `org.freedesktop.DBus.Properties`) [257 (nicht zugänglich)]
- **Nutzung im NovaDE-Projekt für MCP-Kontext:**
    - MCP-Werkzeuge könnten den Batteriestatus abfragen (`OnBattery`, `Percentage`, `TimeToEmpty`), um langlaufende Operationen zu vermeiden oder den Benutzer zu warnen.
    - Anpassung des Verhaltens von NovaDE-Komponenten basierend auf dem Energiestatus (z.B. Reduzierung der Hintergrundaktivität bei niedrigem Akkustand).

### 6.7. `org.freedesktop.Notifications` – Desktop-Benachrichtigungen

Diese Schnittstelle ermöglicht es Anwendungen, Desktop-Benachrichtigungen anzuzeigen.95 MCP-Komponenten im NovaDE-Projekt können dies nutzen, um Benutzer über wichtige Ereignisse, den Abschluss von Aufgaben oder Fehler zu informieren.

- **Schnittstellenspezifikation (`org.freedesktop.Notifications` auf `/org/freedesktop/Notifications`):** 96
    - **Methoden:**
        - `Notify(IN String app_name, IN UInt32 replaces_id, IN String app_icon, IN String summary, IN String body, IN Array<String> actions, IN Dict<String,Variant> hints, IN Int32 expire_timeout, OUT UInt32 notification_id)`
        - `CloseNotification(IN UInt32 id)`
        - `GetCapabilities(OUT Array<String> capabilities)`
        - `GetServerInformation(OUT String name, OUT String vendor, OUT String version, OUT String spec_version)`
    - **Signale:**
        - `NotificationClosed(UInt32 id, UInt32 reason)`
        - `ActionInvoked(UInt32 id, String action_key)`
- **Wichtige Parameter und Hinweise:**
    - `actions`: Liste von Aktions-IDs und deren lesbaren Bezeichnungen.
    - `hints`: Diktionär für zusätzliche Hinweise (z.B. `urgency`, `sound-file`, `image-data`).
    - `expire_timeout`: `-1` für Server-Default, `0` für niemals auslaufend.
- **Nutzung im NovaDE-Projekt durch MCP:**
    - Ein MCP-Tool, das eine langlaufende Aufgabe abschließt, kann `Notify` aufrufen, um den Benutzer zu informieren.
    - Fehler, die in MCP-Operationen auftreten und Benutzereingriffe erfordern, können als Benachrichtigungen signalisiert werden.
    - Aktionen in Benachrichtigungen (`actions`-Parameter) können mit spezifischen MCP-Folgeaktionen im NovaDE-Client verknüpft werden.

## 7. Interaktion mit Wayland (Smithay)

Falls das NovaDE-Projekt einen eigenen Wayland-Compositor beinhaltet oder tief mit einem solchen interagiert (z.B. für spezifische Desktop-Umgebungsfeatures), kommt das Smithay-Framework zum Einsatz.10 Smithay ist eine Rust-Bibliothek zum Erstellen von Wayland-Compositoren.

### 7.1. Smithay Architekturüberblick

Smithay bietet Bausteine für Wayland-Compositoren und ist modular aufgebaut.10

- **Display und EventLoop:** Das Herzstück ist der `Display`-Typ (aus `wayland-server`) und ein `calloop::EventLoop`.98 `DisplayHandle` wird für Interaktionen mit dem Wayland-Protokoll verwendet [214 (nicht zugänglich)]. Der `EventLoopHandle` von `calloop` dient zur Verwaltung von Event-Quellen.234
- **State Management:** Ein zentraler `State`-Typ (z.B. `AnvilState` im Smithay-Beispielcompositor Anvil) hält den Zustand des Compositors [258 (nicht zugänglich), 124 (nicht zugänglich), 124 (nicht zugänglich), 98 (nicht zugänglich), 261 (nicht zugänglich), 262 (nicht zugänglich), 170 (nicht zugänglich)]. `ClientData` (oder `UserDataMap` auf Ressourcen) wird verwendet, um client-spezifischen Zustand zu speichern [98 (nicht zugänglich)].
- **Handler und Delegation:** Für verschiedene Wayland-Protokolle und -Objekte implementiert der `State`-Typ spezifische Handler-Traits (z.B. `CompositorHandler`, `ShmHandler`, `OutputHandler`, `SeatHandler`, `DataDeviceHandler`, `XdgShellHandler`, etc.). Smithay verwendet `delegate_<protocol>!` Makros, um die Dispatch-Logik zu vereinfachen [98 (nicht zugänglich), 136 (nicht zugänglich), 201 (nicht zugänglich), 205 (nicht zugänglich), 200 (nicht zugänglich), 200 (nicht zugänglich), 145 (nicht zugänglich), 222 (nicht zugänglich), 222 (nicht zugänglich), 200 (nicht zugänglich)].

### 7.2. Wayland Core Protokolle und ihre Handhabung durch MCP

#### 7.2.1. `wl_compositor`, `wl_subcompositor`, `wl_surface`, `wl_buffer`

Diese sind grundlegend für jede Wayland-Anzeige.

- **`CompositorState` und `CompositorHandler`:** Smithay stellt `CompositorState` zur Verwaltung von `wl_surface`-Objekten und deren Hierarchien (Subsurfaces) bereit.235 Der `CompositorHandler` im NovaDE-State muss implementiert werden, um auf Surface-Commits und -Zerstörungen zu reagieren.134 `SurfaceData` [263 (nicht zugänglich)] und `CompositorClientState` [201 (nicht zugänglich)] speichern oberflächen- bzw. clientbezogene Zustände. `SurfaceAttributes` enthalten Informationen wie die zugewiesene Rolle [123 (nicht zugänglich)].
- **`wl_buffer`:** Repräsentiert den Inhalt einer Surface. `BufferHandler` [145 (nicht zugänglich)] wird implementiert, um auf die Zerstörung von Buffern zu reagieren.
- **MCP-Interaktion:** MCP-Komponenten könnten indirekt mit diesen Objekten interagieren, z.B. wenn eine MCP-gesteuerte Anwendung eine UI auf dem Desktop darstellt. Die Spezifikation von Fenstergeometrien oder das Anfordern von Neuzeichnungen könnte über MCP-Methoden erfolgen, die dann auf die entsprechenden `wl_surface`-Operationen abgebildet werden.

#### 7.2.2. `wl_shm` – Shared Memory Buffers

Ermöglicht Clients, Buffer über Shared Memory bereitzustellen.

- **`ShmState` und `ShmHandler`:** `ShmState` verwaltet den `wl_shm`-Global und die unterstützten Formate. Der `ShmHandler` im NovaDE-State stellt den Zugriff auf `ShmState` sicher.136
- **Buffer-Import und Rendering:** `with_buffer_contents` erlaubt den Zugriff auf SHM-Buffer-Daten.145 Renderer wie `GlesRenderer` können SHM-Buffer importieren (`import_shm_buffer`) und rendern.171 MCP-Aktionen, die die Anzeige von Inhalten erfordern, die von einem Client als SHM-Buffer bereitgestellt werden, nutzen diese Mechanismen.

#### 7.2.3. `wl_output` und `xdg-output` – Output Management

Verwaltung von Bildschirmausgaben.

- **`Output` und `OutputHandler`:** Ein `Output`-Objekt repräsentiert eine physische Anzeige. `Output::new()` erstellt ein Output-Objekt, `Output::create_global()` macht es für Clients sichtbar [137 (nicht zugänglich), 137]. `Output::change_current_state()` aktualisiert Modus, Transformation, Skalierung und Position. Der `OutputHandler` im NovaDE-State behandelt clientseitige Interaktionen.101
- **`OutputManagerState`:** Kann verwendet werden, um `xdg-output` zusätzlich zu `wl_output` zu verwalten [137 (nicht zugänglich)].
- **MCP-Interaktion:** MCP-Methoden könnten es ermöglichen, Informationen über verfügbare Ausgaben abzurufen oder anwendungsspezifische Fenster auf bestimmten Ausgaben zu positionieren, basierend auf den von diesen Modulen bereitgestellten Informationen.

#### 7.2.4. `wl_seat`, `wl_keyboard`, `wl_pointer`, `wl_touch` – Input Handling

Verwaltung von Eingabegeräten und Fokus.

- **`SeatState` und `SeatHandler`:** `SeatState` verwaltet einen oder mehrere `Seat`-Instanzen. Der `SeatHandler` im NovaDE-State definiert, wie auf Eingabeereignisse und Fokusänderungen reagiert wird.113
- **Fokus-Management:** `KeyboardFocus`, `PointerFocus`, `TouchFocus` werden typischerweise auf `WlSurface` gesetzt, um den Eingabefokus zu lenken.113
- **Input Grabs:** Mechanismen wie `PointerGrab` und `KeyboardGrab` ermöglichen es, Eingabeereignisse exklusiv für eine bestimmte Oberfläche oder Aktion abzufangen [187 (nicht zugänglich)].
- **MCP-Interaktion:** MCP-gesteuerte Aktionen könnten den Fokus anfordern oder auf Eingabeereignisse reagieren, die über diese Seat-Abstraktionen verarbeitet werden. Beispielsweise könnte ein MCP-Tool eine Texteingabe erfordern, was das Setzen des Tastaturfokus auf ein entsprechendes Eingabefeld des MCP-Clients zur Folge hätte.

#### 7.2.5. `wl_data_


# Entwickler-Implementierungsleitfaden: MCP in der UI-Schicht (Ultra-Feinspezifikation)

**Vorwort des Architekten**

Die Integration des Model Context Protocol (MCP) in die Benutzeroberfläche (UI) einer Anwendung stellt einen signifikanten Schritt zur Ermöglichung einer tiefgreifenden und kontextbewussten Kollaboration mit künstlicher Intelligenz dar. Die UI fungiert hierbei als zentrale Schnittstelle, die dem Benutzer nicht nur die Interaktion mit KI-Funktionen ermöglicht, sondern auch die Kontrolle und Transparenz über die zugrundeliegenden MCP-Operationen gewährleisten muss. Dieser Implementierungsleitfaden ist das Ergebnis einer sorgfältigen Analyse der offiziellen MCP-Spezifikationen, existierender Implementierungen und bewährter Praktiken im UI-Design. Er zielt darauf ab, eine robuste, wartbare und benutzerfreundliche Implementierung der UI-Schicht zu ermöglichen, indem er eine präzise und lückenlose Spezifikation aller relevanten Komponenten, Dienste, Datenstrukturen und Prozesse bereitstellt. Die Einhaltung dieses Leitfadens soll sicherstellen, dass Entwickler eine konsistente und qualitativ hochwertige MCP-Integration ohne eigene, grundlegende Designentscheidungen umsetzen können.

## 1. Einleitung und Protokollgrundlagen für UI-Entwickler

Dieser Abschnitt legt die fundamentalen Konzepte des Model Context Protocol (MCP) dar, die für Entwickler der UI-Schicht von entscheidender Bedeutung sind. Ein solides Verständnis dieser Grundlagen ist unerlässlich, um die nachfolgenden detaillierten Spezifikationen korrekt interpretieren und implementieren zu können.

### 1.1. Zielsetzung dieses Implementierungsleitfadens

Das primäre Ziel dieses Dokuments ist die Bereitstellung einer finalen, lückenlosen Entwickler-Implementierungsanleitung für die UI-Schicht im Kontext der MCP-Integration. Diese Spezifikation ist als "Ultra-Feinspezifikation" konzipiert, was bedeutet, dass sie so detailliert ist, dass Entwickler sie direkt zur Implementierung verwenden können, ohne eigene architektonische Entscheidungen treffen oder grundlegende Logiken und Algorithmen entwerfen zu müssen. Alle relevanten Aspekte wurden recherchiert, entschieden und werden hier präzise spezifiziert. Dieser Leitfaden soll jegliche Ambiguität eliminieren und eine konsistente Implementierung über das gesamte Entwicklungsteam hinweg sicherstellen.

### 1.2. MCP-Überblick: Kernkonzepte für die UI-Integration

Das Model Context Protocol (MCP) ist ein offener Standard, der darauf abzielt, die Art und Weise zu standardisieren, wie KI-Anwendungen mit externen Werkzeugen, Datenquellen und Systemen interagieren.1 Für die UI-Schicht, die typischerweise als Host für MCP-Interaktionen agiert, sind folgende Kernkonzepte maßgeblich.

#### 1.2.1. MCP-Architektur: Host, Client, Server

Die MCP-Architektur basiert auf drei Hauptkomponenten 1:

- **Host:** Die Anwendung, mit der der Benutzer direkt interagiert, beispielsweise eine Desktop-Applikation, eine IDE oder ein Chat-Interface. In diesem Leitfaden ist die UI-Anwendung der Host. Der Host ist verantwortlich für die Verwaltung der MCP-Clients und die Durchsetzung von Sicherheitsrichtlinien, insbesondere der Benutzerzustimmung.2
- **Client:** Eine Komponente, die innerhalb des Hosts residiert und die Verbindung zu einem spezifischen MCP-Server verwaltet. Es besteht eine Eins-zu-Eins-Beziehung zwischen einer Client-Instanz und einer Server-Verbindung.1 Wenn eine Host-Anwendung startet, kann sie mehrere MCP-Clients erstellen, von denen jeder für die Verbindung zu einem anderen MCP-Server vorgesehen ist.
- **Server:** Ein externes Programm oder ein Dienst, der Funktionalitäten (Tools), Datenquellen (Ressourcen) und vordefinierte Interaktionsvorlagen (Prompts) über eine standardisierte API bereitstellt, auf die der Client zugreift.1

Die Eins-zu-Eins-Beziehung zwischen einem MCP-Client und einem MCP-Server 1 hat direkte Auswirkungen auf die Architektur der UI-Schicht. Wenn die UI-Anwendung als Host mit mehreren externen Systemen (die jeweils durch einen MCP-Server repräsentiert werden) interagieren soll, muss sie eine robuste Verwaltungslogik für mehrere, potenziell gleichzeitig aktive Client-Instanzen implementieren. Dies erfordert nicht nur Mechanismen zur Kommunikation, sondern auch ein ausgefeiltes Zustandsmanagement für jede einzelne Verbindung sowie eine effiziente Ressourcenverwaltung (z.B. für Threads oder Netzwerkverbindungen, die pro Client benötigt werden könnten). Die UI muss in der Lage sein, diese Client-Instanzen zu erstellen, zu überwachen, ordnungsgemäß zu beenden und deren Status dem Benutzer transparent darzustellen.

#### 1.2.2. MCP-Fähigkeiten: Tools, Ressourcen, Prompts

MCP-Server können drei Haupttypen von Fähigkeiten (Capabilities) anbieten, die für die Interaktion mit dem LLM und dem Benutzer relevant sind 1:

- **Tools (Modellgesteuert):** Dies sind Funktionen, die ein Large Language Model (LLM) aufrufen kann, um spezifische Aktionen auszuführen, beispielsweise eine API abzufragen oder eine Datei zu ändern.1 Die UI muss dem Benutzer klar anzeigen, welche Tools verfügbar sind, und die Ausführung dieser Tools – nach expliziter Zustimmung des Benutzers – orchestrieren und überwachen.
- **Ressourcen (Anwendungsgesteuert):** Dies sind Datenquellen, auf die das LLM zugreifen kann, um Informationen abzurufen, z.B. den Inhalt einer Datei, Ergebnisse einer Datenbankabfrage oder Kontextinformationen aus der Anwendung.1 Die UI muss den Zugriff auf diese Ressourcen ermöglichen, die abgerufenen Daten gegebenenfalls visualisieren oder sie dem LLM zur weiteren Verarbeitung zuführen.
- **Prompts (Benutzergesteuert):** Dies sind vordefinierte Vorlagen oder parametrisierbare Anfragen, die entwickelt wurden, um die Nutzung von Tools oder Ressourcen in einer optimalen und standardisierten Weise zu lenken.1 Die UI muss diese Prompts auflisten und dem Benutzer zur Auswahl und Konfiguration anbieten.

Die unterschiedliche Steuerung dieser Fähigkeiten – modellgesteuert für Tools, anwendungsgesteuert für Ressourcen und benutzergesteuert für Prompts – hat direkte und wichtige Konsequenzen für das Design der Benutzeroberfläche, insbesondere im Hinblick auf Interaktionsabläufe und die Einholung der Benutzerzustimmung.

Für "Tools" ist die explizite Zustimmung des Benutzers vor jeder Ausführung kritisch, da diese Aktionen in externen Systemen auslösen und potenziell Seiteneffekte haben können.3 Die UI muss dem Benutzer klar kommunizieren, welches Tool mit welchen Parametern ausgeführt werden soll und welche Konsequenzen dies haben könnte.

Für "Ressourcen" ist die Zustimmung zum Datenabruf und zur potenziellen Weitergabe dieser Daten an das LLM oder den MCP-Server von zentraler Bedeutung.3 Auch hier muss der Benutzer die Kontrolle darüber behalten, welche Informationen preisgegeben werden.

"Prompts" hingegen stellen primär eine Auswahlmöglichkeit für den Benutzer dar, die den Kontext oder die Art der Interaktion mit Tools und Ressourcen vorstrukturieren. Hier steht die Benutzerfreundlichkeit der Auswahl und Parametrisierung im Vordergrund, während das direkte Sicherheitsrisiko im Vergleich zu Tool-Ausführungen geringer sein kann, aber dennoch die zugrundeliegenden Tool- und Ressourcenzugriffe den üblichen Zustimmungsprozessen unterliegen müssen. Diese Unterscheidungen müssen sich in klar differenzierten UI-Flüssen, Informationsdarstellungen und Zustimmungsdialogen widerspiegeln.

#### 1.2.3. MCP-Zusatzfunktionen (Sampling, Konfiguration, Fortschritt, Abbruch, Fehler, Logging)

Neben den Kernfähigkeiten definiert MCP auch eine Reihe von unterstützenden Protokollfunktionen ("Additional Utilities"), die für eine robuste und benutzerfreundliche UI-Integration von Bedeutung sind 3:

- **Sampling:** Ermöglicht serverseitig initiierte agentische Verhaltensweisen und rekursive LLM-Interaktionen. Die UI muss hierfür strenge Benutzerkontrollen und Zustimmungsmechanismen implementieren.3
- **Konfiguration:** Mechanismen zur Konfiguration von Servern oder der Verbindung.
- **Fortschrittsverfolgung (Progress Tracking):** Erlaubt es Servern, den Fortschritt langlaufender Operationen an den Client zu melden.
- **Abbruch (Cancellation):** Ermöglicht es dem Client, eine laufende Operation auf dem Server abzubrechen.
- **Fehlerberichterstattung (Error Reporting):** Standardisierte Wege zur Meldung von Fehlern.
- **Logging:** Mechanismen für das Logging von Informationen auf Client- oder Serverseite.

Insbesondere Funktionen wie `Progress Tracking` und `Cancellation` sind für die UI von hoher Relevanz. Langlaufende KI-Operationen oder Tool-Aufrufe sind im MCP-Kontext häufig zu erwarten. Ohne eine sichtbare FortschR_S1Anzeige könnte die UI als eingefroren wahrgenommen werden, was zu einer negativen Benutzererfahrung führt. Die Möglichkeit, Operationen abzubrechen, gibt dem Benutzer die notwendige Kontrolle zurück. `Error Reporting` muss in der UI so umgesetzt werden, dass Fehler nicht nur als technische Codes, sondern als verständliche Meldungen mit möglichen Handlungsanweisungen für den Benutzer dargestellt werden. Die UI-Schicht muss also nicht nur die entsprechenden MCP-Nachrichten senden und empfangen, sondern auch die zugehörigen UI-Elemente (z.B. Fortschrittsbalken, Abbrechen-Schaltflächen, detaillierte Fehlermeldungsdialoge) bereitstellen und deren Logik präzise implementieren.

### 1.3. Kommunikationsprotokoll: JSON-RPC 2.0 und Transportmechanismen

Die Kommunikation zwischen MCP-Clients und -Servern basiert auf etablierten Standards.

#### 1.3.1. JSON-RPC 2.0 als Basis

MCP verwendet JSON-RPC 2.0 für den Nachrichtenaustausch.3 JSON-RPC ist ein leichtgewichtetes Remote Procedure Call Protokoll.

Eine Request-Nachricht enthält typischerweise folgende Felder 5:

- `jsonrpc`: Eine Zeichenkette, die die Version des JSON-RPC-Protokolls angibt (muss "2.0" sein).
- `id`: Ein eindeutiger Identifikator (String oder Zahl), der vom Client festgelegt wird. Bei Notifications wird dieses Feld weggelassen.
- `method`: Eine Zeichenkette, die den Namen der aufzurufenden Methode enthält (z.B. "initialize", "tools/list").
- `params`: Ein strukturiertes Objekt oder Array, das die Parameter für die Methode enthält.

Eine **Response-Nachricht** enthält 5:

- `jsonrpc`: Muss "2.0" sein.
- `id`: Muss mit der `id` der korrespondierenden Request-Nachricht übereinstimmen.
- `result`: Dieses Feld enthält das Ergebnis des Methodenaufrufs bei Erfolg. Der Datentyp ist methodenspezifisch.
- `error`: Dieses Feld ist nur bei einem Fehler vorhanden und enthält ein Fehlerobjekt mit den Feldern `code` (eine Zahl), `message` (eine Zeichenkette) und optional `data`.

Für die UI bedeutet dies, dass sie in der Lage sein muss, diese JSON-Strukturen korrekt zu serialisieren (für ausgehende Requests) und zu deserialisieren (für eingehende Responses und Notifications). Die Fehlerbehandlung in der UI muss auf den empfangenen JSON-RPC-Fehlerobjekten basieren und diese in anwendungsspezifische Ausnahmen oder benutzerfreundliche Meldungen umwandeln. JSON-RPC ist besonders gut für aktions- oder funktionsorientierte APIs geeignet, was gut zur Natur von MCP passt, bei dem es um das Aufrufen von Tools und den Zugriff auf Ressourcen geht.6

#### 1.3.2. Transportmechanismen: stdio und HTTP/SSE

MCP unterstützt primär zwei Transportmechanismen für die Übertragung der JSON-RPC-Nachrichten 1:

- **stdio (Standard Input/Output):** Dieser Mechanismus wird typischerweise verwendet, wenn der MCP-Server als lokaler Kindprozess des Hosts (der UI-Anwendung) ausgeführt wird. Der Host sendet JSON-RPC-Requests über den Standard-Input (`stdin`) des Serverprozesses und empfängt Antworten über dessen Standard-Output (`stdout`). Der Standard-Error (`stderr`) kann für Log-Meldungen oder separate Fehlerkanäle genutzt werden.5 Die Verbindung wird typischerweise durch Schließen des `stdin` und Warten auf die Beendigung des Kindprozesses terminiert.
- **HTTP/SSE (Server-Sent Events):** Dieser Mechanismus ist für die Kommunikation mit potenziell entfernten Servern über das Netzwerk vorgesehen. Der Client initiiert eine HTTP-Verbindung zu einem speziellen SSE-Endpunkt des Servers. Nach dem Verbindungsaufbau kann der Server asynchron Ereignisse (JSON-RPC-Responses oder Notifications) an den Client pushen.15 spezifiziert, dass der Client bei diesem Transport eine SSE-Verbindung öffnet und vom Server ein `endpoint` Event mit einer URI erhält. An diese URI sendet der Client dann seine Requests via HTTP POST, während die Antworten des Servers über die bestehende SSE-Verbindung eintreffen.

Die Wahl des Transportmechanismus hat direkte Implikationen für die UI. Sie muss in der Lage sein, beide Mechanismen zu konfigurieren und zu handhaben. Für `stdio` bedeutet dies, dass die UI Pfade zu ausführbaren Dateien und Startargumente verwalten muss.7 Für `HTTP/SSE` sind es URLs und potenziell Authentifizierungsdaten. Die UI muss auch Sicherheitsaspekte berücksichtigen, insbesondere bei `HTTP/SSE`, wo Netzwerkzugriffe und damit verbundene Risiken (Firewalls, Zertifikate, Datensicherheit bei der Übertragung) eine Rolle spielen. Eine flexible UI sollte dem Benutzer oder Administrator die Konfiguration beider Transporttypen ermöglichen, oder es muss eine fundierte Entscheidung für die ausschließliche Unterstützung eines Typs getroffen werden, basierend auf den Anforderungen der Anwendung. Die `mcpr` Rust-Bibliothek demonstriert beispielsweise, wie solche Transportmechanismen abstrahiert werden können.9 Cursor unterstützt und konfiguriert ebenfalls beide Transportarten.10

#### 1.3.3. Zustandsbehaftete Verbindungen (Stateful Connections)

MCP-Verbindungen sind explizit als zustandsbehaftet (stateful) definiert.3 Dies bedeutet, dass der Server Informationen über den Zustand einer Verbindung mit einem bestimmten Client über mehrere Anfragen hinweg speichert und berücksichtigt.11 Der typische Lebenszyklus einer Verbindung beginnt mit einer `initialize`-Nachricht, in der Client und Server Protokollversionen und Fähigkeiten austauschen.5 Erst nach erfolgreicher Initialisierung sind weitere Aufrufe (z.B. `tools/list` oder `tools/call`) gültig und sinnvoll.

Für die UI-Implementierung ist diese Zustandsbehaftung von großer Bedeutung. Die UI muss nicht nur einzelne Nachrichten austauschen, sondern den gesamten Lebenszyklus jeder MCP-Sitzung aktiv managen. Dies beinhaltet:

- Korrekte Initialisierung jeder Verbindung.
- Speicherung und Verwaltung des ausgetauschten Fähigkeitsstatus (`capabilities`) pro Verbindung.5
- Sicherstellung, dass Operationen nur auf korrekt initialisierten und aktiven Verbindungen ausgeführt werden.
- Sauberes Beenden von Verbindungen (`shutdown`).
- Visualisierung des aktuellen Verbindungsstatus (z.B. "verbindend", "initialisiert", "verbunden", "getrennt", "Fehler") für den Benutzer.

Fehler in einer frühen Phase des Verbindungsaufbaus, wie z.B. ein Fehlschlagen der `initialize`-Nachricht, können die gesamte Sitzung für diesen Server ungültig machen. Die UI muss solche Zustände erkennen und entsprechend reagieren, beispielsweise indem sie Operationen für diesen Server deaktiviert oder den Benutzer informiert.

### 1.4. Sicherheits- und Zustimmungserwägungen in der UI (User Consent)

Sicherheit und Benutzerkontrolle sind fundamentale Aspekte des MCP-Protokolls. Die Spezifikation legt großen Wert auf folgende Kernprinzipien 3:

- **Benutzerzustimmung und -kontrolle (User Consent and Control):** Benutzer müssen explizit zustimmen und verstehen, auf welche Daten zugegriffen wird und welche Operationen ausgeführt werden. Sie müssen die Kontrolle darüber behalten, welche Daten geteilt und welche Aktionen durchgeführt werden.
- **Datenschutz (Data Privacy):** Hosts (UI-Anwendungen) **MÜSSEN** explizite Benutzerzustimmung einholen, bevor Benutzerdaten an Server weitergegeben werden. Ressourcendaten dürfen nicht ohne Zustimmung des Benutzers an andere Stellen übertragen werden.
- **Toolsicherheit (Tool Safety):** Tools repräsentieren potenziell beliebige Codeausführung und müssen mit Vorsicht behandelt werden. Beschreibungen des Tool-Verhaltens (Annotationen) sind als nicht vertrauenswürdig zu betrachten, es sei denn, sie stammen von einem vertrauenswürdigen Server. Hosts **MÜSSEN** explizite Benutzerzustimmung einholen, bevor ein Tool aufgerufen wird.
- **LLM-Sampling-Kontrollen:** Benutzer müssen explizit allen LLM-Sampling-Anfragen zustimmen und kontrollieren können, ob Sampling stattfindet, welcher Prompt gesendet wird und welche Ergebnisse der Server sehen kann.

Die Notwendigkeit der Benutzerzustimmung ist nicht nur ein formales Erfordernis, sondern erfordert ein durchdachtes UI/UX-Design. Es reicht nicht aus, ein einfaches Kontrollkästchen anzubieten. Der Benutzer muss klar und unmissverständlich darüber informiert werden, _wozu_ er seine Zustimmung gibt: Welches spezifische Tool soll ausgeführt werden? Mit welchen Parametern? Welche Daten werden von welcher Ressource abgerufen? Welche potenziellen Auswirkungen hat die Aktion? Dies kann granulare und kontextsensitive Zustimmungsdialoge erfordern. Die UI muss zudem den "Vertrauensstatus" eines MCP-Servers berücksichtigen und dem Benutzer signalisieren, falls ein Tool oder eine Beschreibung von einem als "untrusted" eingestuften Server stammt 3, möglicherweise durch eine deutlichere Warnung oder zusätzliche Bestätigungsschritte. Cursor implementiert beispielsweise einen "Tool Approval Flow", bei dem der Benutzer die Argumente sieht, mit denen ein Tool aufgerufen werden soll, bevor er zustimmt.10

Für Desktop-Anwendungen, insbesondere unter Linux-basierten Betriebssystemen, bieten **XDG Desktop Portals** eine standardisierte Methode, um Berechtigungen vom Benutzer über systemeigene Dialoge anzufordern.14 Die Nutzung von XDG Portals (z.B. über Bibliotheken wie `ashpd` in Rust 16) kann die Implementierung von Zustimmungsdialogen erheblich verbessern, da sie eine konsistente Benutzererfahrung über verschiedene Desktop-Umgebungen hinweg gewährleistet und die Anwendung besser in das Betriebssystem integriert. Die `ashpd`-Bibliothek ermöglicht beispielsweise die Interaktion mit Portalen für Farbauswahl oder Kamerazugriff nach Benutzerzustimmung.16 Ein ähnlicher Ansatz wäre für MCP-spezifische Zustimmungen denkbar, wobei `WindowIdentifier` 16 verwendet wird, um den Zustimmungsdialog korrekt dem Elternfenster der Anwendung zuzuordnen. XDG Portals unterstützen sogar Konzepte wie "Pre-Authorization" 14, was für fortgeschrittene Benutzer relevant sein könnte, die bestimmten MCP-Servern oder Tools dauerhaft vertrauen möchten.

### Tabelle 1: Wichtige MCP JSON-RPC Methoden (Client-Sicht)

Die folgende Tabelle fasst die wichtigsten JSON-RPC-Methoden zusammen, die von der UI-Schicht (als MCP-Client) typischerweise initiiert werden, um mit MCP-Servern zu interagieren. Sie dient als Referenz für die Implementierung der Kommunikationslogik.

|   |   |   |   |   |   |
|---|---|---|---|---|---|
|**MCP Funktion**|**JSON-RPC Methode (Request)**|**Richtung**|**Schlüsselparameter (Request)**|**Erwartete Antwortstruktur (Result/Error)**|**Referenz-Snippet**|
|Initialisierung|`initialize`|Client -> Server|`protocolVersion: string`, `capabilities: ClientCapabilities`, `clientInfo: ClientInfo`|`ServerInfo`, `capabilities: ServerCapabilities` (tools, resources, prompts), `protocolVersion: string`|5|
|Tools auflisten|`tools/list`|Client -> Server|`{}` (oft leer, ggf. Filteroptionen)|`ListOf<ToolDefinition>`|17|
|Tool aufrufen|`tools/call`|Client -> Server|`name: string` (Tool-Name), `arguments: object` (Tool-Parameter)|`ToolResult` (methodenspezifisch) oder `ErrorObject`|17|
|Ressourcen auflisten|`resources/list`|Client -> Server|`{}` (oft leer, ggf. Filteroptionen)|`ListOf<ResourceDefinition>`|(Analog zu Tools)|
|Ressource abrufen|`resources/get`|Client -> Server|`name: string` (Ressourcen-Name), `params: object` (optionale Parameter)|`ResourceData` (methodenspezifisch) oder `ErrorObject`|(Analog zu Tools)|
|Prompts auflisten|`prompts/list`|Client -> Server|`{}` (oft leer, ggf. Filteroptionen)|`ListOf<PromptDefinition>`|(Analog zu Tools)|
|Prompt ausführen|`prompts/invoke`|Client -> Server|`name: string` (Prompt-Name), `arguments: object` (Prompt-Parameter)|`PromptResult` (methodenspezifisch) oder `ErrorObject`|(Analog zu Tools)|
|Ping (Lebenszeichen)|`ping`|Client -> Server|`{}` (oder spezifische Ping-Daten)|`PongResponse` (oder spezifische Pong-Daten)|5|
|Operation abbrechen|`$/cancelRequest`|Client -> Server|`id: string \|number` (ID der abzubrechenden Anfrage)|(Notification, keine direkte Antwort erwartet)|
|Fortschrittsbenachrichtigung|`$/progress`|Server -> Client|`token: string \|number`(Fortschrittstoken),`value: any` (Fortschrittsdaten)|(Notification, vom Client zu verarbeiten)|
|Shutdown|`shutdown`|Client -> Server|`{}`|`null` oder `ErrorObject` (oder keine Antwort, wenn als Notification implementiert)|9|

_Hinweis: Die genauen Methodennamen für Ressourcen und Prompts (`resources/list`, `resources/get`, `prompts/list`, `prompts/invoke`) können je nach MCP-Serverimplementierung oder spezifischeren MCP-Erweiterungen variieren. Die Tabelle listet plausible Namen basierend auf der Analogie zu `tools/list` und `tools/call`. Die Methoden `$/cancelRequest` und `$/progress` sind typische JSON-RPC-Benachrichtigungen (Notifications), wobei `$/` eine Konvention für protokollinterne Nachrichten ist._

## 2. Architektur der UI-Schicht mit MCP-Integration

Dieser Abschnitt beschreibt die übergeordnete Architektur der UI-Schicht und wie die MCP-Integration darin verankert ist. Ziel ist es, eine modulare, wartbare und erweiterbare Struktur zu definieren, die den Anforderungen des MCP gerecht wird.

### 2.1. Gesamtarchitektur: Die UI als MCP-Host und ihre Interaktion mit MCP-Servern

Die UI-Anwendung agiert als MCP-Host. Innerhalb dieser Host-Anwendung werden eine oder mehrere MCP-Client-Instanzen verwaltet, wobei jede Client-Instanz für die Kommunikation mit genau einem MCP-Server zuständig ist.1 Die UI-Komponenten selbst (z.B. Buttons, Menüs, Ansichten) interagieren nicht direkt mit den rohen JSON-RPC-Nachrichten oder den Transportmechanismen. Stattdessen greifen sie auf eine Reihe von internen Diensten zurück, die die Komplexität der MCP-Kommunikation kapseln und eine abstrahierte Schnittstelle bereitstellen.

Eine schematische Darstellung der Architektur könnte wie folgt aussehen:

```

     ^
| Interaktion
     v
+---+
| UI-Schicht (MCP Host) |
| +---+ |
| | UserInterfaceModule (Widgets, Views, Controller)| |
| | ^                               V | |
| | | Interaktion       Daten/Events | |
| | +---+ |
| | | Kern-UI-Interaktionsdienste für MCP | |
| | | (ToolOrchestration, ResourceAccess, ConsentUI)| |
| | ^                               V | |
| | | Abstrahierte Aufrufe  Status/Ergebnisse | |
| | +---+ |
| | | MCP-Client-Management-Komponenten | |
| | | (MCPConnectionService, MCPClientInstance) | |
| | ^                           V | |
| +---| JSON-RPC über Transport |---+ |
| (stdio / HTTP+SSE) |
          v                           ^
+---+     +---+
| Externer MCP Server 1 | | Externer MCP Server 2 |
| (Tools, Ressourcen) | | (Tools, Ressourcen) |
+---+     +---+
```

Diese Architektur fördert die Entkopplung:

- **UI-Komponenten** sind für die Darstellung und Benutzerinteraktion zuständig. Sie kennen die MCP-spezifischen Details nur über die Schnittstellen der Kern-UI-Interaktionsdienste.
- **Kern-UI-Interaktionsdienste** (siehe Abschnitt 4) orchestrieren komplexere Abläufe wie Tool-Aufrufe inklusive Zustimmung und aggregieren Daten von verschiedenen Servern.
- **MCP-Client-Management-Komponenten** (siehe Abschnitt 3) kümmern sich um den Lebenszyklus der Verbindungen und die grundlegende JSON-RPC-Kommunikation.

Diese Schichtung ermöglicht es, Änderungen in der MCP-Spezifikation oder bei einzelnen MCP-Servern primär in den unteren Schichten zu behandeln, ohne dass umfangreiche Anpassungen an den eigentlichen UI-Widgets erforderlich werden.

### 2.2. Kernmodule der UI-Schicht und ihre Verantwortlichkeiten im MCP-Kontext

Um die oben beschriebene Architektur umzusetzen, wird die UI-Schicht in mehrere Kernmodule unterteilt, die spezifische Verantwortlichkeiten im MCP-Kontext tragen:

- **`MCPConnectionModule`**:
    
    - **Verantwortung:** Verwaltung des Lebenszyklus aller MCP-Client-Instanzen. Stellt Verbindungen zu MCP-Servern her, überwacht diese und beendet sie. Kapselt die Logik für `MCPConnectionService` und `MCPClientInstance`.
    - **Primäre MCP-Interaktionen:** Senden von `initialize` und `shutdown` Nachrichten, Handling der Transportebene (stdio/SSE).
- **`ToolInteractionModule`**:
    
    - **Verantwortung:** Orchestrierung der Interaktion mit MCP-Tools. Stellt Funktionen zum Auflisten verfügbarer Tools, zum Aufrufen von Tools (nach Zustimmung) und zur Verarbeitung der Ergebnisse bereit. Kapselt den `ToolOrchestrationService`.
    - **Primäre MCP-Interaktionen:** Senden von `tools/list` und `tools/call` Nachrichten, Verarbeitung der Antworten.
- **`ResourceInteractionModule`**:
    
    - **Verantwortung:** Analog zum `ToolInteractionModule`, jedoch für MCP-Ressourcen. Kapselt den `ResourceAccessService`.
    - **Primäre MCP-Interaktionen:** Senden von `resources/list` und `resources/get` (oder äquivalenten) Nachrichten.
- **`PromptInteractionModule`**:
    
    - **Verantwortung:** Handhabung von MCP-Prompts, inklusive Auflistung, Auswahl und Ausführung. Kapselt den `PromptExecutionService`.
    - **Primäre MCP-Interaktionen:** Senden von `prompts/list` und `prompts/invoke` (oder äquivalenten) Nachrichten.
- **`UserInterfaceModule`**:
    
    - **Verantwortung:** Enthält die eigentlichen UI-Komponenten (Widgets, Dialoge, Ansichten), mit denen der Benutzer interagiert (z.B. Kontextmenüs, Sidebar, Chat-Interface). Diese Komponenten nutzen die Dienste der anderen Module, um MCP-Funktionalität darzustellen und zugänglich zu machen. Kapselt Komponenten wie `MCPContextualMenuController`, `MCPSidebarView`, `MCPWidgetFactory`, `AICoPilotInterface`.
- **`ConsentModule`**:
    
    - **Verantwortung:** Zentralisierte Verwaltung und Darstellung von Zustimmungsdialogen für alle MCP-Operationen, die eine explizite Benutzerfreigabe erfordern. Kapselt den `UserConsentUIManager`.
    - **Primäre MCP-Interaktionen:** Keine direkten MCP-Nachrichten, aber eng gekoppelt an die Ausführung von Tool-Aufrufen und Ressourcenzugriffen.
- **`StateManagementModule`**:
    
    - **Verantwortung:** Hält den globalen, reaktiven Zustand aller MCP-bezogenen Informationen (verbundene Server, verfügbare Tools/Ressourcen, laufende Operationen etc.). Kapselt den `MCPGlobalContextManager`.
    - **Primäre MCP-Interaktionen:** Empfängt Status-Updates von anderen Modulen.

Die Modularisierung muss die inhärente Asynchronität der MCP-Kommunikation berücksichtigen. Module, die Netzwerkkommunikation oder Interprozesskommunikation durchführen (insbesondere `MCPConnectionModule`, `ToolInteractionModule`, `ResourceInteractionModule`, `PromptInteractionModule`), müssen dies auf nicht-blockierende Weise tun. Sie sollten asynchrone Programmiermuster (z.B. `async/await`, Promises, Futures) verwenden und Callbacks, Events oder andere reaktive Mechanismen bereitstellen, um das `UserInterfaceModule` und das `StateManagementModule` über abgeschlossene Operationen, empfangene Daten oder Fehler zu informieren, ohne den Haupt-UI-Thread zu blockieren. Dies ist entscheidend für eine responsive Benutzeroberfläche.18

### Tabelle 2: Kern-UI-Module und MCP-Verantwortlichkeiten

|   |   |   |   |
|---|---|---|---|
|**Modulname**|**Kurzbeschreibung der Gesamtverantwortung**|**Primäre MCP-Interaktionen/Aufgaben**|**Wichtige Abhängigkeiten (Beispiele)**|
|`MCPConnectionModule`|Verwaltung des Lebenszyklus von MCP-Client-Verbindungen|`initialize`, `shutdown`, Transport-Handling (stdio/SSE), Senden/Empfangen roher JSON-RPC Nachrichten|Betriebssystem (Prozessmanagement, Netzwerk), JSON-Bibliothek|
|`ToolInteractionModule`|Orchestrierung von Tool-Auflistung und -Ausführung|`tools/list`, `tools/call`|`MCPConnectionModule`, `ConsentModule`, `StateManagementModule`|
|`ResourceInteractionModule`|Orchestrierung von Ressourcen-Auflistung und -Zugriff|`resources/list`, `resources/get`|`MCPConnectionModule`, `ConsentModule`, `StateManagementModule`|
|`PromptInteractionModule`|Handhabung von Prompt-Auflistung, -Auswahl und -Ausführung|`prompts/list`, `prompts/invoke`|`MCPConnectionModule`, `ConsentModule`, `StateManagementModule`, potenziell `ToolInteractionModule` / `ResourceInteractionModule`|
|`UserInterfaceModule`|Darstellung und Benutzerinteraktion mit MCP-Funktionen|Aufruf von Diensten der Interaktionsmodule, Anzeige von Daten und Zuständen|`StateManagementModule`, alle Interaktionsmodule, UI-Toolkit (z.B. GTK, Qt, Web-Framework)|
|`ConsentModule`|Einholung der Benutzerzustimmung für MCP-Aktionen|Anzeige von Zustimmungsdialogen, Verwaltung von Zustimmungsentscheidungen|`UserInterfaceModule` (für Dialogdarstellung), XDG Portal Bibliothek (optional)|
|`StateManagementModule`|Zentraler Speicher für reaktiven MCP-Zustand|Empfang und Bereitstellung von Status-Updates (Server, Tools, Ressourcen, Operationen)|Alle anderen MCP-Module (als Datenquelle oder -konsument)|

Diese Tabelle bietet eine klare Übersicht über die Aufteilung der Verantwortlichkeiten und dient als Grundlage für das detaillierte Design der einzelnen Module und ihrer Schnittstellen. Sie hilft Entwicklern, den Kontext ihrer Arbeit innerhalb der Gesamtarchitektur zu verstehen und die Interaktionspunkte zwischen den Modulen zu identifizieren.

### 2.3. Datenflussdiagramme für typische MCP-Operationen

Um das Zusammenspiel der Komponenten zu visualisieren, werden im Folgenden Datenflussdiagramme für typische MCP-Operationen skizziert. Diese basieren auf dem allgemeinen Workflow, wie er auch in 17 beschrieben wird (Connect, Discover, LLM chooses, Invoke, Return result).

#### 2.3.1. Tool-Auflistung und -Auswahl durch den Benutzer

Code-Snippet

```
sequenceDiagram
    participant Benutzer
    participant UserInterfaceModule (z.B. MCPSidebarView)
    participant ToolInteractionModule (ToolOrchestrationService)
    participant MCPConnectionModule (MCPClientInstance)
    participant ExternerMCPServer

    Benutzer->>UserInterfaceModule: Fordert Tool-Liste an (z.B. Klick auf "Tools anzeigen")
    UserInterfaceModule->>ToolInteractionModule: listAvailableTools()
    ToolInteractionModule->>MCPConnectionModule: Für jede aktive ClientInstance: listTools()
    MCPConnectionModule->>ExternerMCPServer: JSON-RPC Request (method: "tools/list")
    ExternerMCPServer-->>MCPConnectionModule: JSON-RPC Response (result:)
    MCPConnectionModule-->>ToolInteractionModule: Tool-Listen der Server
    ToolInteractionModule-->>UserInterfaceModule: Aggregierte und aufbereitete Tool-Liste
    UserInterfaceModule->>Benutzer: Zeigt verfügbare Tools an
    Benutzer->>UserInterfaceModule: Wählt ein Tool aus
    UserInterfaceModule->>Benutzer: Zeigt Parameter-Eingabefelder für ausgewähltes Tool an (via MCPWidgetFactory)
```

#### 2.3.2. Tool-Aufruf mit Benutzerzustimmung

Code-Snippet

```
sequenceDiagram
    participant Benutzer
    participant UserInterfaceModule (z.B. AICoPilotInterface oder Tool-Widget)
    participant ConsentModule (UserConsentUIManager)
    participant ToolInteractionModule (ToolOrchestrationService)
    participant MCPConnectionModule (MCPClientInstance)
    participant ExternerMCPServer
    participant XDGPortal (optional)

    Benutzer->>UserInterfaceModule: Löst Tool-Aufruf aus (z.B. mit eingegebenen Parametern)
    UserInterfaceModule->>ToolInteractionModule: callTool(toolId, params, parentWindowId)
    ToolInteractionModule->>ConsentModule: requestConsentForTool(toolDefinition, params, parentWindowId)
    ConsentModule->>XDGPortal: (Optional) Fordert System-Dialog an
    XDGPortal-->>ConsentModule: (Optional) Dialog-Ergebnis
    ConsentModule->>Benutzer: Zeigt Zustimmungsdialog an (falls nicht XDG oder als Fallback)
    Benutzer->>ConsentModule: Erteilt/Verweigert Zustimmung
    alt Zustimmung erteilt
        ConsentModule-->>ToolInteractionModule: Zustimmung = true
        ToolInteractionModule->>MCPConnectionModule: callTool(toolName, params) auf spezifischer ClientInstance
        MCPConnectionModule->>ExternerMCPServer: JSON-RPC Request (method: "tools/call", params: {name, arguments})
        ExternerMCPServer-->>MCPConnectionModule: JSON-RPC Response (result: ToolResult oder error)
        MCPConnectionModule-->>ToolInteractionModule: Ergebnis des Tool-Aufrufs
        ToolInteractionModule-->>UserInterfaceModule: Ergebnis/Fehler
        UserInterfaceModule->>Benutzer: Zeigt Ergebnis oder Fehlermeldung an
    else Zustimmung verweigert
        ConsentModule-->>ToolInteractionModule: Zustimmung = false
        ToolInteractionModule-->>UserInterfaceModule: Fehler (MCPConsentDeniedError)
        UserInterfaceModule->>Benutzer: Informiert über verweigerte Zustimmung
    end
```

#### 2.3.3. Ressourcenabruf

Der Datenfluss für den Ressourcenabruf ist analog zum Tool-Aufruf, wobei `ResourceInteractionModule` und `resources/get` (oder äquivalent) verwendet werden. Der Zustimmungsdialog würde sich auf den Zugriff auf spezifische Daten beziehen.

Diese Diagramme illustrieren die typischen Interaktionspfade und die involvierten Module. Sie verdeutlichen die Notwendigkeit einer klaren Aufgabenverteilung und gut definierter Schnittstellen zwischen den Modulen.

### 2.4. Spezifikation der globalen Ausnahmeklassen und Fehlerbehandlungsstrategie

Eine robuste Fehlerbehandlung ist entscheidend für die Stabilität und Benutzerfreundlichkeit der Anwendung. MCP-Interaktionen können aus vielfältigen Gründen fehlschlagen (Netzwerkprobleme, Serverfehler, ungültige Parameter, verweigerte Zustimmung etc.). Die UI muss diese Fehler angemessen behandeln und dem Benutzer verständliches Feedback geben.

Es wird eine Hierarchie von spezifischen Exception-Klassen für MCP-bezogene Fehler definiert. Alle MCP-spezifischen Ausnahmen sollten von einer gemeinsamen Basisklasse `MCPError` erben.

- **`MCPError` (Basisklasse)**
    
    - Attribute:
        - `message: string` (Benutzerfreundliche Standardnachricht oder Nachrichtenschlüssel für Internationalisierung)
        - `originalError?: Error` (Die ursprüngliche Ausnahme, z.B. ein Netzwerkfehler)
        - `jsonRpcError?: JsonRpcErrorObject` (Das JSON-RPC-Fehlerobjekt vom Server, falls vorhanden 5)
        - `isRecoverable: boolean` (Gibt an, ob der Fehler potenziell behebbar ist, z.B. durch einen erneuten Versuch)
    - Methoden: `getUserFriendlyMessage(locale: string): string`
- **Spezifische Ausnahmeklassen (erben von `MCPError`):**
    
    - **`MCPConnectionError extends MCPError`**: Fehler im Zusammenhang mit dem Aufbau oder der Aufrechterhaltung der Verbindung zum MCP-Server (z.B. Server nicht erreichbar, Transportfehler).
        - Zusätzliche Attribute: `serverId: ServerId`, `transportType: 'stdio' | 'sse'`.
    - **`MCPInitializationError extends MCPConnectionError`**: Fehler während der `initialize`-Phase der Verbindung.
    - **`MCPToolExecutionError extends MCPError`**: Fehler bei der Ausführung eines Tools auf dem Server, nachdem die Verbindung erfolgreich hergestellt und das Tool aufgerufen wurde.
        - Zusätzliche Attribute: `toolName: string`, `toolParams: object`.
    - **`MCPResourceAccessError extends MCPError`**: Fehler beim Zugriff auf eine Ressource.
        - Zusätzliche Attribute: `resourceName: string`.
    - **`MCPConsentDeniedError extends MCPError`**: Spezieller Fall, der signalisiert, dass der Benutzer die Zustimmung für eine Aktion verweigert hat. Dies ist technisch gesehen kein "Fehler", aber ein Grund für den Abbruch eines Workflows.
        - `isRecoverable` ist hier typischerweise `false` ohne erneute Benutzerinteraktion.
    - **`MCPInvalidResponseError extends MCPError`**: Die Antwort vom Server entsprach nicht dem erwarteten Format oder der MCP-Spezifikation.
    - **`MCPTimeoutError extends MCPError`**: Zeitüberschreitung beim Warten auf eine Antwort vom Server.

**Fehlerbehandlungsstrategie:**

1. **Erkennung:** Fehler werden entweder in der Transportlogik (z.B. Netzwerk-Timeouts), durch Prüfung der JSON-RPC-Error-Objekte in Serverantworten oder durch interne Validierungen im Client erkannt.
2. **Kapselung:** Der erkannte Fehler wird in eine der oben definierten spezifischen `MCPError`-Ausnahmeklassen gekapselt.
3. **Propagation:** Fehler werden von den unteren Schichten (z.B. `MCPClientInstance`) an die aufrufenden Dienste (z.B. `ToolOrchestrationService`) weitergegeben. Diese Dienste können versuchen, den Fehler zu behandeln (z.B. Retry bei `isRecoverable = true`) oder ihn an die UI-Komponenten weiterzureichen.
4. **Darstellung:** Die UI-Komponenten sind dafür verantwortlich, dem Benutzer eine verständliche Rückmeldung zu geben. Dies kann eine Benachrichtigung, ein Dialog oder eine Statusanzeige sein. Die Nachricht sollte auf `MCPError.getUserFriendlyMessage()` basieren.
    - Es muss klar zwischen technischen Fehlern (z.B. `MCPConnectionError`) und anwendungsspezifischen Fehlern (z.B. `MCPToolExecutionError` aufgrund ungültiger Parameter, die vom Server gemeldet werden) unterschieden werden. `MCPConsentDeniedError` sollte nicht als technischer Fehler, sondern als normaler, vom Benutzer initiierter Abbruch des Vorgangs dargestellt werden.
5. **Logging:** Alle MCP-Fehler **MÜSSEN** detailliert geloggt werden (siehe Abschnitt 7.4), inklusive des ursprünglichen Fehlers und des JSON-RPC-Fehlerobjekts, um die Diagnose zu erleichtern.

Diese strukturierte Fehlerbehandlung stellt sicher, dass Fehler konsistent gehandhabt werden und sowohl Entwickler als auch Benutzer angemessen informiert werden.

### Tabelle 4: Definierte Ausnahmeklassen für MCP-Interaktionen

|   |   |   |   |   |   |
|---|---|---|---|---|---|
|**Klassenname**|**Erbt von**|**Beschreibung des Fehlerszenarios**|**Typische Auslöser**|**Wichtige Attribute (Beispiele)**|**Behandlungsempfehlung in der UI**|
|`MCPError`|(Basis)|Generischer MCP-Fehler|-|`message`, `originalError`, `jsonRpcError`, `isRecoverable`|Basis für spezifischere Meldungen, ggf. generische Fehlermeldung|
|`MCPConnectionError`|`MCPError`|Fehler beim Verbindungsaufbau oder -erhalt|Netzwerkprobleme, Server nicht gestartet, falsche Konfiguration (URL/Pfad)|`serverId`, `transportType`|Meldung "Verbindung zu Server X fehlgeschlagen", Option zum erneuten Versuch oder Überprüfung der Konfiguration|
|`MCPInitializationError`|`MCPConnectionError`|Fehler während der `initialize`-Phase|Inkompatible Protokollversionen, Server lehnt Client ab|-|Meldung "Initialisierung mit Server X fehlgeschlagen", Details aus `jsonRpcError` anzeigen|
|`MCPToolExecutionError`|`MCPError`|Fehler bei der Ausführung eines Tools serverseitig|Ungültige Tool-Parameter, serverseitige Logikfehler im Tool, fehlende Berechtigungen des Servers|`toolName`, `toolParams`|Meldung "Tool X konnte nicht ausgeführt werden", Details aus `jsonRpcError` (falls vorhanden) anzeigen|
|`MCPResourceAccessError`|`MCPError`|Fehler beim Zugriff auf eine Ressource|Ressource nicht gefunden, Zugriff verweigert (serverseitig)|`resourceName`|Meldung "Ressource X konnte nicht abgerufen werden", Details anzeigen|
|`MCPConsentDeniedError`|`MCPError`|Benutzer hat die Zustimmung verweigert|Benutzer klickt "Ablehnen" im Zustimmungsdialog|-|Keine Fehlermeldung, sondern neutrale Info "Aktion vom Benutzer abgebrochen" oder UI kehrt zum vorherigen Zustand zurück|
|`MCPInvalidResponseError`|`MCPError`|Antwort vom Server ist nicht valide (Format, Schema)|Server-Bug, Protokollverletzung|-|Technische Fehlermeldung (primär für Logs), Benutzerinfo "Unerwartete Antwort vom Server"|
|`MCPTimeoutError`|`MCPError`|Zeitüberschreitung beim Warten auf Serverantwort|Langsames Netzwerk, überlasteter Server, Server antwortet nicht|`timeoutDuration`|Meldung "Keine Antwort von Server X innerhalb der Zeitgrenze", Option zum erneuten Versuch|

## 3. Spezifikation der MCP-Client-Management-Komponenten

Dieser Abschnitt detailliert die Komponenten innerhalb der UI-Host-Anwendung, die für die Erstellung, Verwaltung und Kommunikation der MCP-Client-Instanzen zuständig sind. Diese Komponenten bilden das Fundament für alle MCP-Interaktionen.

### 3.1. `MCPConnectionService`

- Zweck:
    
    Der MCPConnectionService ist der zentrale Dienst für die Verwaltung des gesamten Lebenszyklus aller MCPClientInstance-Objekte. Er ist verantwortlich für das dynamische Erstellen, Starten, Stoppen und Überwachen von Verbindungen zu verschiedenen MCP-Servern. Diese Aktionen basieren auf Benutzerkonfigurationen (z.B. aus einer mcp.json-Datei 10) oder auf dynamischen Anforderungen der Anwendung. Der Dienst stellt sicher, dass die UI stets einen aktuellen Überblick über alle aktiven und potenziellen MCP-Verbindungen hat.
    
- **Eigenschaften:**
    
    - `private static instance: MCPConnectionService | null = null;`
        - Für Singleton-Implementierung.
    - `private activeConnections: Map<ServerId, MCPClientInstance> = new Map();`
        - Eine Map, die alle aktiven `MCPClientInstance`-Objekte verwaltet. Der Schlüssel `ServerId` ist eine eindeutige Kennung für einen MCP-Server (z.B. eine aus der Konfiguration abgeleitete ID oder die Server-URL).
    - `private serverConfigurations: Map<ServerId, MCPServerConfig> = new Map();`
        - Eine Map, die die Konfigurationen aller bekannten MCP-Server speichert, typischerweise geladen beim Start der Anwendung.
- **Methoden:**
    
    - `public static getInstance(): MCPConnectionService`
        - **Signatur:** `public static getInstance(): MCPConnectionService noexcept`
        - **Beschreibung:** Implementiert das Singleton-Pattern. Gibt die einzige Instanz des `MCPConnectionService` zurück. Erstellt die Instanz beim ersten Aufruf.
        - **Vorbedingungen:** Keine.
        - **Nachbedingungen:** Gibt eine valide Instanz von `MCPConnectionService` zurück.
    - `public async loadAndInitializeConnections(configs: MCPServerConfig): Promise<void>`
        - **Signatur:** `public async loadAndInitializeConnections(configs: MCPServerConfig): Promise<void>`
        - **Beschreibung:** Lädt eine Liste von Serverkonfigurationen, speichert sie in `serverConfigurations` und versucht, für jede Konfiguration eine Verbindung herzustellen und zu initialisieren. Iteriert über `configs`, erstellt für jede eine `MCPClientInstance` (falls nicht bereits vorhanden und unterschiedlich konfiguriert) und ruft deren `connectAndInitialize()` Methode auf. Fehler beim Verbindungsaufbau zu einzelnen Servern dürfen den Prozess für andere Server nicht blockieren.
        - **Parameter:**
            - `configs: MCPServerConfig`: Eine Liste von Serverkonfigurationsobjekten.
        - **Vorbedingungen:** `configs` ist ein valides Array.
        - **Nachbedingungen:** Für jede Konfiguration in `configs` wurde versucht, eine `MCPClientInstance` zu erstellen und zu initialisieren. `activeConnections` und `serverConfigurations` sind aktualisiert. Entsprechende Events (`ServerConnectionStatusChanged`, `ClientInstanceAdded`) wurden ausgelöst.
        - **Ausnahmen:** Kann `MCPError` werfen, wenn ein grundlegender Fehler beim Laden der Konfigurationen auftritt (selten, da einzelne Verbindungsfehler intern behandelt werden sollten).
    - `public async connectToServer(config: MCPServerConfig): Promise<MCPClientInstance | MCPError>`
        - **Signatur:** `public async connectToServer(config: MCPServerConfig): Promise<MCPClientInstance | MCPError>`
        - **Beschreibung:** Stellt explizit eine Verbindung zu einem einzelnen, spezifizierten MCP-Server her und initialisiert diese. Erstellt eine neue `MCPClientInstance` basierend auf der `config`, fügt sie zu `activeConnections` hinzu und ruft `connectAndInitialize()` auf. Gibt die `MCPClientInstance` bei Erfolg oder ein `MCPError`-Objekt bei Fehlschlag zurück.
        - **Parameter:**
            - `config: MCPServerConfig`: Die Konfiguration des zu verbindenden Servers.
        - **Vorbedingungen:** `config` ist ein valides Objekt.
        - **Nachbedingungen:** Eine `MCPClientInstance` wurde erstellt und versucht zu verbinden. `activeConnections` ist aktualisiert. Events wurden ausgelöst.
    - `public async disconnectFromServer(serverId: ServerId): Promise<void | MCPError>`
        - **Signatur:** `public async disconnectFromServer(serverId: ServerId): Promise<void | MCPError>`
        - **Beschreibung:** Trennt die Verbindung zu einem bestimmten MCP-Server und entfernt die zugehörige `MCPClientInstance` aus der Verwaltung. Ruft `shutdown()` auf der `MCPClientInstance` auf, bevor sie aus `activeConnections` entfernt wird.
        - **Parameter:**
            - `serverId: ServerId`: Die ID des Servers, dessen Verbindung getrennt werden soll.
        - **Vorbedingungen:** `serverId` ist eine gültige ID eines potenziell aktiven Servers.
        - **Nachbedingungen:** Die Verbindung zum Server wurde (versucht zu) getrennt und die `MCPClientInstance` wurde aus `activeConnections` entfernt. `ClientInstanceRemoved`-Event wurde ausgelöst.
    - `public getClientInstance(serverId: ServerId): MCPClientInstance | undefined`
        - **Signatur:** `public getClientInstance(serverId: ServerId): MCPClientInstance | undefined noexcept`
        - **Beschreibung:** Gibt die aktive `MCPClientInstance` für eine gegebene `ServerId` zurück, falls vorhanden.
        - **Parameter:**
            - `serverId: ServerId`: Die ID des gesuchten Servers.
        - **Rückgabewert:** Die `MCPClientInstance` oder `undefined`.
    - `public getAllClientInstances(): MCPClientInstance`
        - **Signatur:** `public getAllClientInstances(): MCPClientInstance noexcept`
        - **Beschreibung:** Gibt eine Liste aller aktuell aktiven `MCPClientInstance`-Objekte zurück.
        - **Rückgabewert:** Ein Array von `MCPClientInstance`-Objekten.
    - `public subscribeToServerStatusChanges(serverId: ServerId, callback: (status: ConnectionStatus, clientInstance?: MCPClientInstance, error?: MCPError) => void): UnsubscribeFunction`
        - **Signatur:** `public subscribeToServerStatusChanges(serverId: ServerId, callback: (status: ConnectionStatus, clientInstance?: MCPClientInstance, error?: MCPError) => void): UnsubscribeFunction noexcept`
        - **Beschreibung:** Ermöglicht anderen UI-Teilen oder Diensten, Änderungen im Verbindungsstatus eines spezifischen Servers zu abonnieren. Der Callback wird aufgerufen, wenn sich der `connectionStatus` der entsprechenden `MCPClientInstance` ändert.
        - **Parameter:**
            - `serverId: ServerId`: Die ID des zu beobachtenden Servers.
            - `callback`: Die Funktion, die bei Statusänderungen aufgerufen wird.
        - **Rückgabewert:** Eine `UnsubscribeFunction`, die aufgerufen werden kann, um das Abonnement zu beenden.
    - `public subscribeToClientListChanges(callback: (clients: MCPClientInstance) => void): UnsubscribeFunction`
        - **Signatur:** `public subscribeToClientListChanges(callback: (clients: MCPClientInstance) => void): UnsubscribeFunction noexcept`
        - **Beschreibung:** Benachrichtigt Abonnenten, wenn `MCPClientInstance`s hinzugefügt oder entfernt werden (d.h., die Liste der aktiven Verbindungen ändert sich).
        - **Parameter:**
            - `callback`: Die Funktion, die bei Änderungen aufgerufen wird und die aktuelle Liste der Clients erhält.
        - **Rückgabewert:** Eine `UnsubscribeFunction`.
- **Events (ausgehend, intern über ein Event-Bus-System oder direkt an Abonnenten):**
    
    - **`ServerConnectionStatusChanged`**
        - **Payload:** `{ serverId: ServerId, newStatus: ConnectionStatus, clientInstance?: MCPClientInstance, error?: MCPError }`
        - **Beschreibung:** Wird ausgelöst, wenn sich der `connectionStatus` einer `MCPClientInstance` ändert.
    - **`ClientInstanceAdded`**
        - **Payload:** `{ client: MCPClientInstance }`
        - **Beschreibung:** Wird ausgelöst, nachdem eine neue `MCPClientInstance` erfolgreich erstellt und initial mit dem Verbindungsaufbau begonnen wurde.
    - **`ClientInstanceRemoved`**
        - **Payload:** `{ serverId: ServerId, reason?: 'disconnected' | 'error' }`
        - **Beschreibung:** Wird ausgelöst, nachdem eine `MCPClientInstance` entfernt wurde (z.B. nach `disconnectFromServer` oder einem fatalen Fehler).
- **Zustandsdiagramm für `MCPConnectionService`:**
    
    Code-Snippet
    
    ```
    stateDiagram-v2
        [*] --> Idle
        Idle --> InitializingConnections : loadAndInitializeConnections()
        InitializingConnections --> Running : Alle initialen Verbindungsversuche abgeschlossen
        Running --> Running : connectToServer() / disconnectFromServer()
        Running --> Idle : shutdownAllConnections() (hypothetische Methode für Anwendungsende)
    ```
    
    (Hinweis: Die Zustände einer einzelnen `MCPClientInstance` sind komplexer und werden dort beschrieben.)
    
- Fehlerbehandlung:
    
    Der MCPConnectionService fängt Fehler von den MCPClientInstance-Methoden (connectAndInitialize, shutdown) ab. Diese Fehler werden geloggt und über das ServerConnectionStatusChanged-Event mit dem Status Error und dem entsprechenden MCPError-Objekt signalisiert. Kritische Fehler, die den Service selbst betreffen (z.B. Speicherprobleme), sollten als schwerwiegende Anwendungsfehler behandelt werden.
    

Der `MCPConnectionService` ist der zentrale Dreh- und Angelpunkt für die gesamte MCP-Konnektivität der UI. Seine Fähigkeit, mehrere Verbindungen – auch fehlerhafte – effizient und robust zu managen, ist entscheidend für die Stabilität der MCP-Funktionen. Da Verbindungen potenziell parallel aufgebaut oder abgebaut werden könnten (z.B. durch Benutzeraktionen oder bei Anwendungsstart), muss der Zugriff auf geteilte Zustände wie `activeConnections` und `serverConfigurations` Thread-sicher gestaltet sein, falls die zugrundeliegende Plattform dies erfordert (z.B. durch Mutexe oder andere Synchronisationsprimitive).

### 3.2. `MCPClientInstance`

- Zweck:
    
    Die MCPClientInstance repräsentiert und verwaltet die aktive Kommunikationssitzung mit einem einzelnen MCP-Server. Sie kapselt die Details der JSON-RPC-Nachrichtenübertragung für diesen spezifischen Server, den Verbindungslebenszyklus (Initialisierung, Betrieb, Beendigung) und den aktuellen Zustand dieser Verbindung. Jede Instanz ist für genau einen Server zuständig, wie durch ihre Konfiguration definiert.
    
- **Eigenschaften:**
    
    - `public readonly serverId: ServerId`
        - Eindeutige Kennung des Servers, abgeleitet aus der `MCPServerConfig`.
    - `public readonly config: MCPServerConfig`
        - Das Konfigurationsobjekt, das zur Erstellung dieser Instanz verwendet wurde. Enthält Informationen wie Transporttyp, URL/Kommando etc.
    - `private currentProtocolVersion: string | null = null;`
        - Die vom Server während der `initialize`-Phase gemeldete Protokollversion.5
    - `private serverCapabilitiesInternal: ServerCapabilities | null = null;`
        - Die vom Server während der `initialize`-Phase gemeldeten Fähigkeiten (unterstützte Tools, Ressourcen, Prompts etc.).5
    - `public readonly clientCapabilities: ClientCapabilities;`
        - Die Fähigkeiten, die dieser Client dem Server anbietet (z.B. Unterstützung für `sampling` 3). Wird im Konstruktor gesetzt.
    - `private currentConnectionStatus: ConnectionStatus = ConnectionStatus.Idle;`
        - Der aktuelle Zustand der Verbindung. Enum: `Idle`, `Connecting`, `Initializing`, `Connected`, `Reconnecting`, `Disconnecting`, `Disconnected`, `Error`.
    - `private lastErrorEncountered: MCPError | null = null;`
        - Das letzte aufgetretene `MCPError`-Objekt für diese Verbindung.
    - `private transportHandler: IMCPTransport;`
        - Eine Instanz eines Transport-Handlers (z.B. `StdioTransportHandler` oder `SSETransportHandler`), der für die tatsächliche Nachrichtenübertragung zuständig ist. Wird basierend auf `config.transportType` instanziiert.
    - `private pendingRequests: Map<string | number, (response: JsonRpcResponse | JsonRpcError) => void> = new Map();`
        - Verwaltet Callbacks für ausstehende JSON-RPC-Anfragen anhand ihrer `id`.
    - `private notificationSubscribers: Map<string, Array<(notification: JsonRpcNotification) => void>> = new Map();` // Key: method name or '*' for all
        - Verwaltet Abonnenten für serverseitige Notifications.
- **Methoden:**
    
    - `public constructor(config: MCPServerConfig, clientCapabilities: ClientCapabilities)`
        - **Signatur:** `public constructor(config: MCPServerConfig, clientCapabilities: ClientCapabilities)`
        - **Beschreibung:** Initialisiert eine neue `MCPClientInstance`. Setzt `serverId`, `config`, `clientCapabilities`. Instanziiert den passenden `transportHandler` basierend auf `config.transportType`. Registriert einen internen Handler beim `transportHandler` für eingehende Nachrichten (Responses, Notifications).
        - **Vorbedingungen:** `config` und `clientCapabilities` sind valide.
        - **Nachbedingungen:** Die Instanz ist initialisiert und bereit für `connectAndInitialize()`.
    - `public async connectAndInitialize(): Promise<void | MCPError>`
        - **Signatur:** `public async connectAndInitialize(): Promise<void | MCPError>`
        - **Beschreibung:**
            1. Setzt `currentConnectionStatus` auf `Connecting`. Löst `StatusChanged`-Event aus.
            2. Ruft `transportHandler.connect()` auf. Bei Fehler: Setzt Status auf `Error`, speichert Fehler, löst Event aus, gibt Fehler zurück.
            3. Setzt `currentConnectionStatus` auf `Initializing`. Löst Event aus.
            4. Baut die `initialize`-Nachricht zusammen (siehe unten, basierend auf 5).
            5. Sendet die `initialize`-Nachricht über `this.sendRequestInternal(...)`.
            6. Bei Erfolg: Verarbeitet die Antwort, setzt `currentProtocolVersion` und `serverCapabilitiesInternal`. Setzt `currentConnectionStatus` auf `Connected`. Löst `StatusChanged`- und `CapabilitiesChanged`-Events aus. Gibt `void` zurück.
            7. Bei Fehler: Setzt Status auf `Error`, speichert Fehler, löst Event aus, gibt `MCPInitializationError` zurück.
        - **`initialize`-Request-Struktur (Beispiel):**
            
            JSON
            
            ```
            {
              "jsonrpc": "2.0",
              "id": "generierte_eindeutige_id_1",
              "method": "initialize",
              "params": {
                "protocolVersion": "2025-03-26", // Aktuell unterstützte MCP-Version
                "capabilities": { // this.clientCapabilities
                  "sampling": { /* ggf. Optionen für Sampling */ }
                },
                "clientInfo": {
                  "name": "MeineSuperUIAnwendung",
                  "version": "1.0.0"
                }
              }
            }
            ```
            
        - **`initialize`-Response-Verarbeitung:** Speichert `result.serverInfo`, `result.capabilities` (z.B. `result.capabilities.tools`, `result.capabilities.resources`, `result.capabilities.prompts`), `result.protocolVersion` in den internen Eigenschaften.
    - `public async shutdown(): Promise<void>`
        - **Signatur:** `public async shutdown(): Promise<void>`
        - **Beschreibung:**
            1. Setzt `currentConnectionStatus` auf `Disconnecting`. Löst Event aus.
            2. Versucht, eine `shutdown`-Nachricht an den Server zu senden (falls im MCP-Standard für den Client vorgesehen und der Server verbunden ist). Dies ist oft eine Notification.
            3. Ruft `transportHandler.disconnect()` auf.
            4. Setzt `currentConnectionStatus` auf `Disconnected`. Löst Event aus. Bereinigt interne Zustände (z.B. `pendingRequests`).
    - `public async callTool(toolName: string, params: object): Promise<ToolResult | MCPError>`
        - **Signatur:** `public async callTool(toolName: string, params: object): Promise<any | MCPError>` (Rückgabetyp `any` für `ToolResult`, da tool-spezifisch)
        - **Beschreibung:** Sendet eine `tools/call`-Nachricht an den Server.17
            1. Prüft, ob `currentConnectionStatus === ConnectionStatus.Connected`. Wenn nicht, gibt `MCPConnectionError` zurück.
            2. Baut die `tools/call`-Request-Nachricht:
                
                JSON
                
                ```
                {
                  "jsonrpc": "2.0",
                  "id": "generierte_eindeutige_id_N",
                  "method": "tools/call",
                  "params": { "name": toolName, "arguments": params }
                }
                ```
                
            3. Sendet die Nachricht über `this.sendRequestInternal(...)`.
            4. Gibt das `result` der Antwort oder ein `MCPToolExecutionError` zurück.
    - `public async listTools(): Promise<ToolDefinition | MCPError>`
        - **Signatur:** `public async listTools(): Promise<ToolDefinition | MCPError>`
        - **Beschreibung:** Sendet eine `tools/list`-Nachricht.17
            1. Prüft `currentConnectionStatus`.
            2. Request: `{ "jsonrpc": "2.0", "id": "...", "method": "tools/list", "params": {} }`
            3. Sendet via `this.sendRequestInternal(...)`.
            4. Gibt `result` (Array von `ToolDefinition`) oder `MCPError` zurück.
    - `public async getResource(resourceName: string, params?: object): Promise<any | MCPError>` (analog zu `callTool`, Methode z.B. `resources/get`)
    - `public async listResources(): Promise<ResourceDefinition | MCPError>` (analog zu `listTools`, Methode z.B. `resources/list`)
    - `public async invokePrompt(promptName: string, params?: object): Promise<any | MCPError>` (analog zu `callTool`, Methode z.B. `prompts/invoke`)
    - `public async listPrompts(): Promise<PromptDefinition | MCPError>` (analog zu `listTools`, Methode z.B. `prompts/list`)
    - `public async ping(): Promise<any | MCPError>`
        - **Signatur:** `public async ping(): Promise<any | MCPError>`
        - **Beschreibung:** Sendet eine `ping`-Nachricht.5
            1. Prüft `currentConnectionStatus`.
            2. Request: `{ "jsonrpc": "2.0", "id": "...", "method": "ping", "params": {} }` (oder spezifische Ping-Daten)
            3. Sendet via `this.sendRequestInternal(...)`.
            4. Gibt `result` oder `MCPError` zurück.
    - `public async cancelRequest(idToCancel: string | number): Promise<void | MCPError>`
        - **Signatur:** `public async cancelRequest(idToCancel: string | number): Promise<void | MCPError>`
        - **Beschreibung:** Sendet eine `$/cancelRequest`-Notification, um eine vorherige Anfrage abzubrechen.3
            1. Prüft `currentConnectionStatus`.
            2. Notification: `{ "jsonrpc": "2.0", "method": "$/cancelRequest", "params": { "id": idToCancel } }`
            3. Sendet via `this.sendNotificationInternal(...)`.
    - `private async sendRequestInternal<TParams, TResult>(method: string, params: TParams): Promise<TResult | MCPError>`
        - **Beschreibung:** Interne Hilfsmethode. Generiert eine eindeutige `id`, erstellt das `JsonRpcRequest`-Objekt, registriert einen Callback in `pendingRequests` und sendet die Nachricht über `transportHandler.sendMessage()`. Gibt ein Promise zurück, das mit dem Ergebnis oder einem Fehlerobjekt aufgelöst wird.
    - `private async sendNotificationInternal<TParams>(method: string, params: TParams): Promise<void | MCPError>`
        - **Beschreibung:** Interne Hilfsmethode zum Senden von JSON-RPC-Notifications (ohne `id`). Sendet über `transportHandler.sendMessage()`.
    - `private handleIncomingMessage(message: JsonRpcResponse | JsonRpcError | JsonRpcNotification): void`
        - **Beschreibung:** Wird vom `transportHandler` aufgerufen. Unterscheidet, ob es eine Response auf eine `pendingRequest` ist (dann Callback aufrufen und aus Map entfernen) oder eine Notification (dann registrierte `notificationSubscribers` informieren).
    - `public subscribeToNotifications(methodFilter: string | null, callback: (notification: JsonRpcNotification) => void): UnsubscribeFunction`
        - **Signatur:** `public subscribeToNotifications(methodFilter: string | null, callback: (notification: JsonRpcNotification<any>) => void): UnsubscribeFunction noexcept`
        - **Beschreibung:** Ermöglicht das Abonnieren von serverseitigen Notifications. `methodFilter` kann ein spezifischer Methodenname (z.B. `$/progress`) oder `null` (oder `'*'`) für alle Notifications sein.
        - **Rückgabewert:** Eine `UnsubscribeFunction`.
    - `public getConnectionStatus(): ConnectionStatus`
        - **Signatur:** `public getConnectionStatus(): ConnectionStatus noexcept`
    - `public getLastError(): MCPError | null`
        - **Signatur:** `public getLastError(): MCPError | null noexcept`
    - `public getServerCapabilities(): ServerCapabilities | null`
        - **Signatur:** `public getServerCapabilities(): ServerCapabilities | null noexcept`
- **Events (ausgehend, typischerweise an den `MCPConnectionService` oder einen internen Event-Bus):**
    
    - **`StatusChanged`**
        - **Payload:** `{ newStatus: ConnectionStatus, error?: MCPError }`
    - **`CapabilitiesChanged`**
        - **Payload:** `{ newCapabilities: ServerCapabilities }` (nach erfolgreicher Initialisierung)
    - **`NotificationReceived`**
        - **Payload:** `{ notification: JsonRpcNotification }` (z.B. für `$/progress`)
- Interaktion mit IMCPTransport:
    
    Die MCPClientInstance verwendet eine Instanz, die die folgende Schnittstelle IMCPTransport implementiert:
    
    TypeScript
    
    ```
    interface IMCPTransport {
        connect(): Promise<void | MCPError>;
        disconnect(): Promise<void>;
        sendMessage(message: JsonRpcRequest | JsonRpcNotification): Promise<void | MCPError>; // Sendet, erwartet keine direkte Antwort hier
        registerMessageHandler(handler: (message: JsonRpcResponse | JsonRpcError | JsonRpcNotification) => void): void;
        // Optional: getTransportStatus(): TransportStatusEnum;
    }
    ```
    
    Konkrete Implementierungen sind `StdioTransportHandler` und `SSETransportHandler`. Der `StdioTransportHandler` würde Methoden zum Starten und Überwachen des Kindprozesses sowie zum Lesen/Schreiben von dessen `stdin`/`stdout` kapseln.7 Der `SSETransportHandler` würde die HTTP-Verbindung und den SSE-Eventstream verwalten.20
    

Die MCPClientInstance ist der Kern der Protokollimplementierung für eine einzelne Serververbindung. Sie muss die JSON-RPC-Spezifikation exakt umsetzen, die Zustandsübergänge der Verbindung sauber managen und eine klare Schnittstelle für das Senden von Anfragen und den Empfang von Antworten und Notifications bieten. Die Abstraktion des Transports durch IMCPTransport ist entscheidend für die Flexibilität, verschiedene Kommunikationswege zu unterstützen, ohne die Kernlogik der MCPClientInstance ändern zu müssen.

Die während der Initialisierung vom Server empfangenen serverCapabilities 5 sind von entscheidender Bedeutung. Sie informieren die UI darüber, welche Tools, Ressourcen und Prompts der verbundene Server überhaupt anbietet. Diese Informationen müssen von der MCPClientInstance persistent gehalten (für die Dauer der Sitzung) und den übergeordneten UI-Diensten (wie ToolOrchestrationService, siehe Abschnitt 4) zur Verfügung gestellt werden. Diese Dienste nutzen die Fähigkeiten, um die Benutzeroberfläche dynamisch anzupassen – beispielsweise, um zu entscheiden, welche Menüeinträge, Schaltflächen oder Optionen dem Benutzer für die Interaktion mit diesem spezifischen Server angezeigt werden. Ohne Kenntnis der serverCapabilities wüsste die UI nicht, welche Operationen sie dem Server anbieten kann.

## 4. Spezifikation der Kern-UI-Interaktionsdienste für MCP

Diese Dienste bauen auf dem `MCPConnectionService` und den einzelnen `MCPClientInstance`s auf. Sie bieten eine höhere Abstraktionsebene für UI-Komponenten, um mit MCP-Funktionalitäten zu interagieren. Ihre Hauptaufgaben umfassen die Aggregation von Informationen über mehrere Server hinweg, die Orchestrierung von komplexeren Arbeitsabläufen (wie Tool-Aufrufe inklusive Benutzerzustimmung) und die Bereitstellung eines konsolidierten Zustands für die UI.

### 4.1. `ToolOrchestrationService`

- Zweck:
    
    Der ToolOrchestrationService ist der zentrale Dienst für alle Interaktionen, die MCP-Tools betreffen. Er bietet Funktionen zur Auflistung aller verfügbaren Tools von allen verbundenen und initialisierten MCP-Servern, zur Initiierung von Tool-Aufrufen (wobei er die notwendige Benutzerzustimmung über den UserConsentUIManager einholt) und zur Weiterleitung und initialen Verarbeitung der Ergebnisse dieser Aufrufe.
    
- **Eigenschaften:**
    
    - `private mcpConnectionService: MCPConnectionService;`
        - Abhängigkeit zum `MCPConnectionService`, um Zugriff auf die aktiven `MCPClientInstance`s zu erhalten. Wird typischerweise per Dependency Injection injiziert.
    - `private userConsentUIManager: UserConsentUIManager;`
        - Abhängigkeit zum `UserConsentUIManager` (siehe Abschnitt 4.4) für die Einholung der Benutzerzustimmung.
    - `private availableToolsCache: Map<GlobalToolId, ToolDefinitionExtended> = new Map();`
        - Ein interner Cache, der eine aggregierte Liste aller bekannten Tools von allen verbundenen Servern hält. `GlobalToolId` ist eine eindeutige Kennung für ein Tool über alle Server hinweg (z.B. eine Kombination aus `ServerId` und `tool.name`, um Namenskonflikte zwischen Tools verschiedener Server zu vermeiden). `ToolDefinitionExtended` erweitert die Standard-`ToolDefinition` um die `ServerId` und ggf. weitere UI-relevante Metadaten.
    - `private static instance: ToolOrchestrationService | null = null;`
- **Methoden:**
    
    - `public static getInstance(connService: MCPConnectionService, consentUIManager: UserConsentUIManager): ToolOrchestrationService`
        - **Signatur:** `public static getInstance(connService: MCPConnectionService, consentUIManager: UserConsentUIManager): ToolOrchestrationService noexcept`
        - **Beschreibung:** Singleton-Zugriffsmethode.
    - `public async refreshAvailableTools(): Promise<ToolDefinitionExtended>`
        - **Signatur:** `public async refreshAvailableTools(): Promise<ToolDefinitionExtended>`
        - **Beschreibung:** Fordert von allen aktiven und verbundenen `MCPClientInstance`s (via `mcpConnectionService.getAllClientInstances()`) deren Tool-Listen an (durch Aufruf von `client.listTools()`). Aggregiert diese Listen, erstellt `GlobalToolId`s, aktualisiert den `availableToolsCache` und gibt die vollständige, aktualisierte Liste zurück. Löst das `ToolListUpdated`-Event aus.
        - **Rückgabewert:** Ein Promise, das mit einem Array von `ToolDefinitionExtended` aufgelöst wird.
        - **Ausnahmen:** Kann Fehler von `client.listTools()` sammeln und aggregiert melden oder einzelne Fehler loggen und nur erfolgreiche Ergebnisse zurückgeben.
    - `public getAvailableTools(): ToolDefinitionExtended`
        - **Signatur:** `public getAvailableTools(): ToolDefinitionExtended noexcept`
        - **Beschreibung:** Gibt die aktuell im Cache gehaltene Liste aller verfügbaren Tools zurück. Ruft nicht aktiv `refreshAvailableTools` auf.
    - `public async callTool(toolId: GlobalToolId, params: object, parentWindowId?: WindowIdentifier): Promise<any | MCPError | MCPConsentDeniedError>`
        - **Signatur:** `public async callTool(toolId: GlobalToolId, params: object, parentWindowId?: WindowIdentifier): Promise<any | MCPError | MCPConsentDeniedError>`
        - **Beschreibung:** Führt ein spezifisches Tool aus:
            1. Ermittelt die `ToolDefinitionExtended` und die zugehörige `ServerId` aus `toolId` und dem `availableToolsCache`. Falls nicht gefunden, wird ein Fehler zurückgegeben.
            2. Ermittelt die zuständige `MCPClientInstance` über `mcpConnectionService.getClientInstance(serverId)`. Falls nicht gefunden oder nicht verbunden, wird ein `MCPConnectionError` zurückgegeben.
            3. Ruft `userConsentUIManager.requestConsentForTool(toolDefinition, params, parentWindowId)` auf, um die explizite Zustimmung des Benutzers einzuholen.10
            4. Wenn die Zustimmung verweigert wird, wird ein `MCPConsentDeniedError` zurückgegeben.
            5. Wenn die Zustimmung erteilt wird: Löst das `ToolCallStarted`-Event aus. Ruft `clientInstance.callTool(toolDefinition.name, params)` auf.9
            6. Das Ergebnis (Erfolg oder Fehler von `clientInstance.callTool`) wird zurückgegeben. Löst das `ToolCallCompleted`-Event aus.
        - **Parameter:**
            - `toolId: GlobalToolId`: Die eindeutige ID des auszuführenden Tools.
            - `params: object`: Die Parameter für den Tool-Aufruf.
            - `parentWindowId?: WindowIdentifier`: Optionale Kennung des Elternfensters für den Zustimmungsdialog.16
        - **Rückgabewert:** Ein Promise, das mit dem Tool-Ergebnis, einem `MCPError` oder einem `MCPConsentDeniedError` aufgelöst wird.
    - `public getToolDefinition(toolId: GlobalToolId): ToolDefinitionExtended | undefined`
        - **Signatur:** `public getToolDefinition(toolId: GlobalToolId): ToolDefinitionExtended | undefined noexcept`
        - **Beschreibung:** Gibt die zwischengespeicherte `ToolDefinitionExtended` für eine gegebene `GlobalToolId` zurück.
- **Events (ausgehend, über einen Event-Bus oder direkt an Abonnenten):**
    
    - **`ToolListUpdated`**
        - **Payload:** `{ tools: ToolDefinitionExtended }`
        - **Beschreibung:** Wird ausgelöst, nachdem `refreshAvailableTools` erfolgreich neue Tool-Definitionen geladen hat.
    - **`ToolCallStarted`**
        - **Payload:** `{ toolId: GlobalToolId, params: object }`
        - **Beschreibung:** Wird ausgelöst, unmittelbar bevor `clientInstance.callTool` aufgerufen wird (nach erteilter Zustimmung).
    - **`ToolCallCompleted`**
        - **Payload:** `{ toolId: GlobalToolId, result: any | MCPError }` (wobei `result` nicht `MCPConsentDeniedError` sein wird, da dies vorher behandelt wird)
        - **Beschreibung:** Wird ausgelöst, nachdem der Aufruf von `clientInstance.callTool` abgeschlossen ist, entweder erfolgreich oder mit einem Fehler.

Dieser Dienst entkoppelt die spezifische UI-Logik (z.B. ein Button-Klick in einem Widget) vom direkten Management der `MCPClientInstance`. Er zentralisiert die Logik für Tool-Interaktionen, insbesondere die kritische Überprüfung der Benutzerzustimmung, und stellt eine konsistente Schnittstelle für alle UI-Teile bereit, die Tools ausführen müssen. Die Verwendung einer `GlobalToolId` und der `ToolDefinitionExtended` (welche die `ServerId` enthält) ist hierbei entscheidend. Es ist durchaus möglich, dass zwei verschiedene MCP-Server Tools mit identischen Namen anbieten (z.B. ein Tool namens `search`). Um diese eindeutig identifizieren und den Aufruf an die korrekte `MCPClientInstance` weiterleiten zu können, muss die `ServerId` Teil der globalen Tool-Identifikation sein. Der `ToolOrchestrationService` stellt diese Eindeutigkeit sicher und leitet Anfragen korrekt weiter.

### 4.2. `ResourceAccessService`

- Zweck:
    
    Der ResourceAccessService ist das Pendant zum ToolOrchestrationService, jedoch spezialisiert auf MCP-Ressourcen. Er stellt Funktionen zur Auflistung aller verfügbaren Ressourcen von allen verbundenen MCP-Servern, zum Abruf von Ressourcendaten (inklusive Einholung der Benutzerzustimmung für den Datenzugriff) und zur Verarbeitung der Ergebnisse bereit.
    
- **Eigenschaften:**
    
    - `private mcpConnectionService: MCPConnectionService;` (Abhängigkeit)
    - `private userConsentUIManager: UserConsentUIManager;` (Abhängigkeit)
    - `private availableResourcesCache: Map<GlobalResourceId, ResourceDefinitionExtended> = new Map();`
        - Analoger Cache wie bei Tools. `GlobalResourceId` (z.B. `serverId + ":" + resourceName`). `ResourceDefinitionExtended` enthält die `ResourceDefinition` plus `serverId`.
    - `private static instance: ResourceAccessService | null = null;`
- **Methoden:**
    
    - `public static getInstance(connService: MCPConnectionService, consentUIManager: UserConsentUIManager): ResourceAccessService`
        - **Signatur:** `public static getInstance(connService: MCPConnectionService, consentUIManager: UserConsentUIManager): ResourceAccessService noexcept`
    - `public async refreshAvailableResources(): Promise<ResourceDefinitionExtended>`
        - **Signatur:** `public async refreshAvailableResources(): Promise<ResourceDefinitionExtended>`
        - **Beschreibung:** Analog zu `refreshAvailableTools`, ruft `client.listResources()` auf allen aktiven Clients auf. Aktualisiert `availableResourcesCache`. Löst `ResourceListUpdated`-Event aus.
    - `public getAvailableResources(): ResourceDefinitionExtended`
        - **Signatur:** `public getAvailableResources(): ResourceDefinitionExtended noexcept`
        - **Beschreibung:** Gibt den aktuellen Cache der verfügbaren Ressourcen zurück.
    - `public async getResourceData(resourceId: GlobalResourceId, params?: object, parentWindowId?: WindowIdentifier): Promise<any | MCPError | MCPConsentDeniedError>`
        - **Signatur:** `public async getResourceData(resourceId: GlobalResourceId, params?: object, parentWindowId?: WindowIdentifier): Promise<any | MCPError | MCPConsentDeniedError>`
        - **Beschreibung:** Ruft Daten einer spezifischen Ressource ab:
            1. Ermittelt `ResourceDefinitionExtended` und `ServerId` aus `resourceId`.
            2. Ermittelt die `MCPClientInstance`.
            3. Ruft `userConsentUIManager.requestConsentForResource(resourceDefinition, parentWindowId)` auf.3
            4. Bei Ablehnung: `MCPConsentDeniedError`.
            5. Bei Zustimmung: Ruft `clientInstance.getResource(resourceDefinition.name, params)` auf.
            6. Gibt Ergebnis oder Fehler zurück. Löst `ResourceAccessCompleted`-Event aus.
        - **Parameter:**
            - `resourceId: GlobalResourceId`: Die eindeutige ID der Ressource.
            - `params?: object`: Optionale Parameter für den Ressourcenzugriff.
            - `parentWindowId?: WindowIdentifier`: Für den Zustimmungsdialog.
    - `public getResourceDefinition(resourceId: GlobalResourceId): ResourceDefinitionExtended | undefined`
        - **Signatur:** `public getResourceDefinition(resourceId: GlobalResourceId): ResourceDefinitionExtended | undefined noexcept`
        - **Beschreibung:** Gibt die Definition einer Ressource aus dem Cache zurück.
- **Events (ausgehend):**
    
    - **`ResourceListUpdated`**
        - **Payload:** `{ resources: ResourceDefinitionExtended }`
    - **`ResourceAccessCompleted`**
        - **Payload:** `{ resourceId: GlobalResourceId, data: any | MCPError }`

Die Trennung von Tool- und Ressourcenzugriff in separate Dienste (`ToolOrchestrationService` und `ResourceAccessService`) ist trotz vieler Ähnlichkeiten im Ablauf sinnvoll. Tools implizieren typischerweise die Ausführung von Aktionen, die Seiteneffekte haben können, während Ressourcen primär dem Abruf von Daten dienen.1 Diese semantische Unterscheidung kann sich in unterschiedlichen Zustimmungsanforderungen, Caching-Strategien oder Fehlerbehandlungen niederschlagen. Ein eigener Dienst für Ressourcen macht die API der UI-Schicht klarer und ermöglicht spezifische Optimierungen oder Darstellungslogiken für Ressourcendaten.

### 4.3. `PromptExecutionService`

- Zweck:
    
    Der PromptExecutionService ist für die Handhabung von MCP-Prompts zuständig. Prompts sind benutzergesteuerte, vordefinierte Vorlagen oder parametrisierbare Anfragen, die die Nutzung von Tools oder Ressourcen optimieren oder komplexe Interaktionsflüsse standardisieren können.1 Dieser Dienst ermöglicht das Auflisten verfügbarer Prompts, die Auswahl durch den Benutzer und die Initiierung der Prompt-Ausführung.
    
- **Eigenschaften:**
    
    - `private mcpConnectionService: MCPConnectionService;` (Abhängigkeit)
    - `private toolOrchestrationService: ToolOrchestrationService;` (Potenzielle Abhängigkeit, falls Prompts Tools aufrufen)
    - `private resourceAccessService: ResourceAccessService;` (Potenzielle Abhängigkeit, falls Prompts Ressourcen abrufen)
    - `private availablePromptsCache: Map<GlobalPromptId, PromptDefinitionExtended> = new Map();`
        - Cache für Prompts. `GlobalPromptId` (z.B. `serverId + ":" + promptName`). `PromptDefinitionExtended` enthält die `PromptDefinition` plus `serverId`.
    - `private static instance: PromptExecutionService | null = null;`
- **Methoden:**
    
    - `public static getInstance(connService: MCPConnectionService, toolService: ToolOrchestrationService, resourceService: ResourceAccessService): PromptExecutionService`
        - **Signatur:** `public static getInstance(connService: MCPConnectionService, toolService: ToolOrchestrationService, resourceService: ResourceAccessService): PromptExecutionService noexcept`
    - `public async refreshAvailablePrompts(): Promise<PromptDefinitionExtended>`
        - **Signatur:** `public async refreshAvailablePrompts(): Promise<PromptDefinitionExtended>`
        - **Beschreibung:** Analog zu `refreshAvailableTools`, ruft `client.listPrompts()` auf. Aktualisiert `availablePromptsCache`. Löst `PromptListUpdated`-Event aus.
    - `public getAvailablePrompts(): PromptDefinitionExtended`
        - **Signatur:** `public getAvailablePrompts(): PromptDefinitionExtended noexcept`
    - `public async invokePrompt(promptId: GlobalPromptId, params: object, parentWindowId?: WindowIdentifier): Promise<any | MCPError | MCPConsentDeniedError>`
        - **Signatur:** `public async invokePrompt(promptId: GlobalPromptId, params: object, parentWindowId?: WindowIdentifier): Promise<any | MCPError | MCPConsentDeniedError>`
        - **Beschreibung:** Führt einen Prompt aus:
            1. Ermittelt `PromptDefinitionExtended` und `ServerId`.
            2. Ermittelt die `MCPClientInstance`.
            3. **Wichtig:** Die Ausführung eines Prompts kann komplex sein. Sie kann serverseitig gesteuert sein oder clientseitig eine Sequenz von Tool-Aufrufen und/oder Ressourcenabrufen erfordern, die jeweils eigene Zustimmungen benötigen.
            4. Wenn der Prompt direkt über eine MCP-Methode (z.B. `prompts/invoke`) aufgerufen wird:
                - Ggf. Zustimmung für den Prompt selbst einholen (falls der Prompt als Ganzes eine "Aktion" darstellt).
                - Ruft `clientInstance.invokePrompt(promptDefinition.name, params)` auf.
            5. Wenn der Prompt clientseitig orchestriert wird (basierend auf der `PromptDefinition`):
                - Der `PromptExecutionService` interpretiert die Prompt-Definition und ruft nacheinander die notwendigen Methoden des `ToolOrchestrationService` oder `ResourceAccessService` auf. Jeder dieser Aufrufe durchläuft den dortigen Zustimmungsflow.
            6. Gibt das finale Ergebnis des Prompts oder einen Fehler zurück. Löst `PromptExecutionCompleted`-Event aus.
    - `public getPromptDefinition(promptId: GlobalPromptId): PromptDefinitionExtended | undefined`
        - **Signatur:** `public getPromptDefinition(promptId: GlobalPromptId): PromptDefinitionExtended | undefined noexcept`
- **Events (ausgehend):**
    
    - **`PromptListUpdated`**
        - **Payload:** `{ prompts: PromptDefinitionExtended }`
    - **`PromptExecutionStarted`**
        - **Payload:** `{ promptId: GlobalPromptId, params: object }`
    - **`PromptExecutionCompleted`**
        - **Payload:** `{ promptId: GlobalPromptId, result: any | MCPError }`

Prompts sind als "user-controlled" 1 und "templated messages and workflows" 3 charakterisiert. Dies impliziert, dass die UI dem Benutzer diese Prompts optimal präsentieren und die notwendigen Parameter für den Aufruf eines Prompts abfragen muss. Die Ausführung eines Prompts ist potenziell mehr als nur ein einzelner Request-Response-Zyklus; sie kann eine geführte Interaktion oder eine Kaskade von Operationen darstellen. Der `PromptExecutionService` muss diese Komplexität kapseln. Wenn ein Prompt beispielsweise definiert ist als "Suche Dokumente (Ressource), fasse sie mit Tool A zusammen und sende das Ergebnis an Tool B", dann muss der `PromptExecutionService` diese Schritte koordinieren und dabei sicherstellen, dass für jeden einzelnen Schritt die notwendigen Zustimmungen eingeholt werden.

### 4.4. `UserConsentUIManager`

- Zweck:
    
    Der UserConsentUIManager ist der zentrale Dienst für die Anzeige von Zustimmungsdialogen und die Einholung der expliziten Benutzerzustimmung für alle MCP-Aktionen, die dies erfordern. Dazu gehören Tool-Aufrufe, Ressourcenzugriffe und potenziell LLM-Sampling-Anfragen, die vom Server initiiert werden.3 Dieser Manager ist kritisch für die Einhaltung der Sicherheits- und Datenschutzprinzipien von MCP.
    
- **Methoden:**
    
    - `public async requestConsentForTool(toolDefinition: ToolDefinitionExtended, params: object, parentWindowId?: WindowIdentifier): Promise<boolean>`
        - **Signatur:** `public async requestConsentForTool(toolDefinition: ToolDefinitionExtended, params: object, parentWindowId?: WindowIdentifier): Promise<boolean>`
        - **Beschreibung:** Zeigt einen modalen Dialog an, der den Benutzer über das aufzurufende Tool informiert. Der Dialog **MUSS** folgende Informationen klar und verständlich darstellen:
            - Name und Beschreibung des Tools (aus `toolDefinition`).
            - Der MCP-Server, der das Tool bereitstellt (`toolDefinition.serverId`, ggf. mit Name des Servers).
            - Die Parameter (`params`), mit denen das Tool aufgerufen werden soll. Diese sollten dem Benutzer lesbar präsentiert werden.10
            - Eine klare Frage, ob der Benutzer der Ausführung zustimmt.
            - Buttons für "Zustimmen" und "Ablehnen".
        - Optional kann der Dialog eine Option "Details anzeigen" bieten, um z.B. das vollständige `parameters_schema` oder eine längere Beschreibung des Tools anzuzeigen.
        - Gibt `true` zurück, wenn der Benutzer zustimmt, andernfalls `false` (bei Ablehnung oder Schließen des Dialogs ohne Zustimmung).
        - **Parameter:**
            - `toolDefinition: ToolDefinitionExtended`: Die Definition des Tools.
            - `params: object`: Die Parameter für den Aufruf.
            - `parentWindowId?: WindowIdentifier`: ID des Elternfensters für korrekte modale Darstellung.16
    - `public async requestConsentForResource(resourceDefinition: ResourceDefinitionExtended, accessParams?: object, parentWindowId?: WindowIdentifier): Promise<boolean>`
        - **Signatur:** `public async requestConsentForResource(resourceDefinition: ResourceDefinitionExtended, accessParams?: object, parentWindowId?: WindowIdentifier): Promise<boolean>`
        - **Beschreibung:** Analog zu `requestConsentForTool`, aber für den Zugriff auf eine Ressource. Der Dialog informiert über die Ressource, den Server und die Art des Zugriffs (z.B. "Daten von Ressource X lesen").
    - `public async requestConsentForSampling(samplingRequestDetails: object, serverId: ServerId, parentWindowId?: WindowIdentifier): Promise<boolean>`
        - **Signatur:** `public async requestConsentForSampling(samplingRequestDetails: object, serverId: ServerId, parentWindowId?: WindowIdentifier): Promise<boolean>`
        - **Beschreibung:** Fordert Zustimmung für eine vom Server (`serverId`) initiierte LLM-Sampling-Operation an.3 Der Dialog muss Details der Anfrage (`samplingRequestDetails`) klar darstellen.
    - `public async showUntrustedServerWarning(serverConfig: MCPServerConfig, parentWindowId?: WindowIdentifier): Promise<UserTrustDecision>`
        - **Signatur:** `public async showUntrustedServerWarning(serverConfig: MCPServerConfig, parentWindowId?: WindowIdentifier): Promise<UserTrustDecision>` (`UserTrustDecision` könnte ein Enum sein: `AllowOnce`, `AllowAlways`, `Block`)
        - **Beschreibung:** Zeigt eine Warnung an, wenn versucht wird, eine Verbindung zu einem Server herzustellen, der als nicht vertrauenswürdig markiert ist oder dessen Vertrauensstatus unbekannt ist. Dies ist besonders relevant, wenn Tool-Beschreibungen als potenziell unsicher gelten.3
        - Der Dialog sollte Optionen bieten, dem Server einmalig zu vertrauen, dauerhaft zu vertrauen (was eine Speicherung dieser Entscheidung erfordert) oder die Verbindung abzulehnen.
- **UI-Anforderungen für Zustimmungsdialoge:**
    
    - **Klarheit und Verständlichkeit:** Die Informationen müssen so aufbereitet sein, dass ein durchschnittlicher Benutzer die Konsequenzen seiner Entscheidung versteht. Fachjargon ist zu vermeiden oder zu erklären.
    - **Transparenz:** Es muss klar sein, welche Anwendung (der Host) die Zustimmung anfordert und welcher externe MCP-Server involviert ist.
    - **Granularität:** Zustimmungen sollten so granular wie möglich sein (z.B. pro Tool-Aufruf, nicht pauschal für einen ganzen Server, es sei denn, der Benutzer wählt dies explizit).
    - **Sicherheitshinweise:** Bei potenziell riskanten Operationen oder nicht vertrauenswürdigen Servern sollten explizite Warnungen angezeigt werden.
    - **Option "Immer erlauben/blockieren":** Wenn diese Option angeboten wird, muss es eine Möglichkeit für den Benutzer geben, diese gespeicherten Entscheidungen einzusehen und zu widerrufen (z.B. in den Anwendungseinstellungen). Die Speicherung dieser Präferenzen muss sicher erfolgen. 14 erwähnt `flatpak permission-set kde-authorized` für KDE, was auf systemseitige Mechanismen zur Speicherung solcher Berechtigungen hindeutet, die ggf. genutzt werden könnten.
- **Integration mit XDG Desktop Portals (Empfohlen für Desktop-Anwendungen unter Linux):**
    
    - Für eine nahtlose Integration in Desktop-Umgebungen **SOLLTE** die Verwendung von XDG Desktop Portals für Zustimmungsdialoge in Betracht gezogen werden. Bibliotheken wie `ashpd` für Rust 16 können die Interaktion mit diesen Portalen vereinfachen.
    - Der `parentWindowId` Parameter (als `WindowIdentifier` 16) ist hierbei wichtig, um dem Portal-Backend mitzuteilen, zu welchem Anwendungsfenster der Dialog gehören soll.
    - Dies würde systemeigene Dialoge verwenden, was die Benutzerakzeptanz und Konsistenz erhöht.

Der `UserConsentUIManager` ist eine kritische Komponente für die Sicherheit und das Vertrauen der Benutzer in die MCP-Funktionen der Anwendung. Die Dialoge müssen sorgfältig gestaltet werden, um eine informierte Entscheidungsfindung zu ermöglichen. Die Verwaltung von dauerhaften Zustimmungsentscheidungen ("Immer erlauben") ist ein komplexes Thema, das über einfache Dialoganzeige hinausgeht und eine Persistenzschicht sowie UI-Elemente zur Verwaltung dieser Einstellungen erfordert.

## 5. Spezifikation der UI-Komponenten und Widgets für die MCP-gestützte KI-Kollaboration

Dieser Abschnitt beschreibt die konkreten UI-Elemente (Widgets, Ansichten, Controller), die der Benutzer sieht und mit denen er interagiert, um die durch MCP bereitgestellten KI-Kollaborationsfunktionen zu nutzen. Diese Komponenten bauen auf den Diensten aus Abschnitt 4 auf und nutzen den globalen Zustand aus dem `MCPGlobalContextManager`.

### 5.1. `MCPGlobalContextManager` (oder `MCPStateService`)

- Zweck:
    
    Der MCPGlobalContextManager dient als zentraler, global zugänglicher Speicher (Store) oder Dienst, der den übergreifenden, reaktiven Zustand aller MCP-Interaktionen für die gesamte UI-Anwendung bereithält. Er fungiert als "Single Source of Truth" für MCP-bezogene Daten, auf die verschiedene UI-Komponenten zugreifen und auf deren Änderungen sie reagieren können. Dies kann durch ein State-Management-Framework (wie Redux, Vuex, Zustand in Web-Technologien oder entsprechende Äquivalente in Desktop-Frameworks) oder durch ein implementiertes Observable-Pattern erreicht werden.
    
- **Eigenschaften (Beispiele, als reaktive Datenfelder konzipiert):**
    
    - `public readonly allConnectedServers: Computed<MCPServerInfo>`
        - Eine reaktive Liste der aktuell verbundenen und initialisierten MCP-Server, inklusive Basisinformationen wie `ServerId`, Name, Status, ggf. Icon.
    - `public readonly allAvailableTools: Computed<ToolDefinitionExtended>`
        - Eine reaktive, aggregierte Liste aller Tools, die von den verbundenen Servern angeboten werden. Aktualisiert durch den `ToolOrchestrationService`.
    - `public readonly allAvailableResources: Computed<ResourceDefinitionExtended>`
        - Analog für alle verfügbaren Ressourcen. Aktualisiert durch den `ResourceAccessService`.
    - `public readonly allAvailablePrompts: Computed<PromptDefinitionExtended>`
        - Analog für alle verfügbaren Prompts. Aktualisiert durch den `PromptExecutionService`.
    - `public readonly pendingToolCalls: Computed<Map<CallId, ToolCallState>>`
        - Eine reaktive Map, die den Status laufender Tool-Aufrufe verfolgt (z.B. `CallId` als eindeutige ID des Aufrufs, `ToolCallState` mit Infos wie `toolId`, `startTime`, `progress`, `status`).
    - `public readonly recentMcpErrors: Computed<MCPError>`
        - Eine reaktive Liste der zuletzt aufgetretenen MCP-Fehler, die UI-weit angezeigt werden könnten oder für Debugging-Zwecke nützlich sind.
    - `public readonly mcpFeatureEnabled: Computed<boolean>`
        - Ein Flag, das anzeigt, ob die MCP-Funktionalität global aktiviert ist.
- **Methoden:**
    
    - Primär Getter-Methoden für die oben genannten reaktiven Eigenschaften.
    - Interne Setter-Methoden oder Mechanismen, die von den MCP-Interaktionsdiensten (aus Abschnitt 4) aufgerufen werden, um den Zustand zu aktualisieren (z.B. `updateToolList(tools: ToolDefinitionExtended)`, `addPendingToolCall(callId: CallId, initialState: ToolCallState)`). Diese sollten nicht direkt von UI-Widgets aufgerufen werden.
    - `public getToolDefinitionById(toolId: GlobalToolId): ToolDefinitionExtended | undefined`
    - `public getResourceDefinitionById(resourceId: GlobalResourceId): ResourceDefinitionExtended | undefined`
    - `public getPromptDefinitionById(promptId: GlobalPromptId): PromptDefinitionExtended | undefined`
- Abonnementmechanismus:
    
    Der MCPGlobalContextManager MUSS einen Mechanismus bereitstellen, der es UI-Komponenten ermöglicht, auf Änderungen spezifischer Teile des MCP-Zustands zu reagieren (zu "abonnieren"). Wenn sich beispielsweise die Liste der allAvailableTools ändert, sollten alle UI-Komponenten, die diese Liste anzeigen oder davon abhängen, automatisch benachrichtigt und neu gerendert werden.
    
- Relevanz:
    
    Dieser Manager ist entscheidend für die Entwicklung einer reaktiven und konsistenten Benutzeroberfläche. Er entkoppelt die Datenerzeugung und -aktualisierung (durch die Services) von der Datenkonsumption (durch die UI-Widgets). Wenn beispielsweise ein neuer MCP-Server verbunden wird und dieser neue Tools bereitstellt, aktualisiert der ToolOrchestrationService den MCPGlobalContextManager, welcher wiederum automatisch alle abhängigen UI-Elemente (wie Kontextmenüs oder Seitenleisten) dazu veranlasst, sich neu darzustellen und die neuen Tools anzuzeigen. Ohne einen solchen zentralen State Manager wäre es schwierig, den UI-Zustand über viele Komponenten hinweg synchron zu halten, was zu Inkonsistenzen und einer schlechten Benutzererfahrung führen würde.
    

### 5.2. `MCPContextualMenuController`

- Zweck:
    
    Der MCPContextualMenuController ist dafür verantwortlich, dynamisch Kontextmenüeinträge zu generieren, die MCP-bezogene Aktionen anbieten. Diese Einträge basieren auf dem aktuellen Kontext der Benutzeroberfläche (z.B. ausgewählter Text, eine Datei im Explorer, das aktive UI-Element) und den über den MCPGlobalContextManager bekannten, verfügbaren MCP-Tools, -Ressourcen und -Prompts.
    
- **Eigenschaften:**
    
    - `private mcpGlobalContextManager: MCPGlobalContextManager;` (Abhängigkeit)
    - `private toolOrchestrationService: ToolOrchestrationService;` (Abhängigkeit, um Aktionen auszulösen)
    - `private resourceAccessService: ResourceAccessService;` (Abhängigkeit)
    - `private promptExecutionService: PromptExecutionService;` (Abhängigkeit)
    - `private currentAppContext: AppSpecificContext | null = null;`
        - Hält den Kontext, für den das Menü generiert werden soll. `AppSpecificContext` ist ein Platzhalter für eine Struktur, die den relevanten Kontext der Host-Anwendung beschreibt (z.B. `{ type: 'textSelection', content: string }` oder `{ type: 'file', path: string, mimeType: string }`).
- **Methoden:**
    
    - `public constructor(contextManager: MCPGlobalContextManager, toolService: ToolOrchestrationService, /*...andere Dienste... */)`
    - `public updateCurrentAppContext(context: AppSpecificContext): void`
        - **Signatur:** `public updateCurrentAppContext(context: AppSpecificContext): void noexcept`
        - **Beschreibung:** Wird von der UI aufgerufen, wenn sich der Kontext ändert, auf den sich ein potenzielles Kontextmenü beziehen würde (z.B. bei Fokuswechsel, neuer Auswahl).
    - `public generateContextMenuItems(): MenuItem`
        - **Signatur:** `public generateContextMenuItems(): MenuItem noexcept`
        - **Beschreibung:**
            1. Greift auf `this.currentAppContext` zu. Wenn kein Kontext vorhanden ist oder dieser für MCP-Aktionen irrelevant ist, wird ein leeres Array oder ein Standardmenü zurückgegeben.
            2. Ruft die Listen der verfügbaren Tools, Ressourcen und Prompts vom `mcpGlobalContextManager` ab.
            3. Filtert diese Listen basierend auf `this.currentAppContext`. Die Relevanz eines Tools/einer Ressource/eines Prompts für einen gegebenen Kontext kann durch Metadaten in deren Definitionen bestimmt werden (z.B. ein Feld `applicableContextTypes: string` in `ToolDefinitionExtended`, das MIME-Typen oder abstrakte Kontexttypen wie "text", "code", "image" enthält).
            4. Für jede relevante MCP-Aktion wird ein `MenuItem`-Objekt erstellt. Ein `MenuItem` sollte mindestens enthalten:
                - `label: string` (Anzeigetext, z.B. Tool-Name)
                - `icon?: string` (Optionales Icon)
                - `action: () => Promise<void>` (Eine Funktion, die bei Auswahl des Eintrags ausgeführt wird. Diese Funktion ruft die entsprechende Methode des zuständigen Dienstes auf, z.B. `toolOrchestrationService.callTool(...)` mit den notwendigen Parametern, die ggf. aus `currentAppContext` extrahiert werden).
                - `isEnabled: boolean` (Ob der Eintrag aktiv ist).
                - Optional: Untermenüs für Tools/Ressourcen von verschiedenen Servern oder nach Kategorien.
            5. Gibt das Array der generierten `MenuItem`-Objekte zurück.
    - `public registerContextProvider(provider: () => AppSpecificContext | null): void` (Alternativer Ansatz zu `updateCurrentAppContext`)
        - **Signatur:** `public registerContextProvider(provider: () => AppSpecificContext | null): void noexcept`
        - **Beschreibung:** Ermöglicht verschiedenen Teilen der UI (z.B. einem Texteditor, einem Dateibrowser), eine Funktion zu registrieren, die bei Bedarf den aktuellen Kontext liefert. `generateContextMenuItems` würde dann diesen Provider aufrufen.
- Logik zur Aktionsauswahl:
    
    Die "Relevanz" von MCP-Aktionen für einen bestimmten Kontext ist der Schlüssel zu einem nützlichen Kontextmenü. Ein einfaches Auflisten aller verfügbaren Tools ist selten benutzerfreundlich. Der Controller MUSS intelligent filtern und idealerweise priorisieren. Dies kann erreicht werden durch:
    
    - **Explizite Metadaten:** Tool-/Ressourcen-/Prompt-Definitionen enthalten Informationen darüber, auf welche Kontexttypen sie anwendbar sind.
    - **Heuristiken:** Basierend auf dem Typ und Inhalt des Kontexts (z.B. Dateiendung, ausgewählter Textinhalt).
    - **Benutzerkonfiguration:** Der Benutzer kann bevorzugte Aktionen für bestimmte Kontexte definieren.
    - **(Fortgeschritten) LLM-basierte Vorschläge:** Eine kleine, schnelle LLM-Anfrage könnte basierend auf dem Kontext und den verfügbaren Aktionen die relevantesten vorschlagen (dies würde jedoch eine weitere LLM-Interaktion bedeuten und muss sorgfältig abgewogen werden).
- Relevanz:
    
    Ein gut implementiertes kontextsensitives Menü macht MCP-Funktionen nahtlos im Arbeitsfluss des Benutzers zugänglich. Es reduziert die Notwendigkeit, separate Dialoge oder Paletten zu öffnen, und steigert so die Effizienz und Akzeptanz der KI-Kollaborationsfeatures. Die Intelligenz bei der Auswahl der angezeigten Aktionen ist dabei entscheidend für die Qualität der Benutzererfahrung.
    

### 5.3. `MCPSidebarView` (oder `MCPToolPalette`)

- Zweck:
    
    Die MCPSidebarView ist eine dedizierte, persistentere UI-Komponente (z.B. eine Seitenleiste, ein andockbares Fenster oder eine Werkzeugpalette), die dem Benutzer einen umfassenden Überblick und direkte Interaktionsmöglichkeiten mit allen Aspekten der MCP-Integration bietet. Sie dient als zentrale Anlaufstelle für die Verwaltung von MCP-Servern, das Entdecken von Tools, Ressourcen und Prompts sowie die Überwachung laufender Operationen. 4 beschreibt eine ähnliche Funktionalität ("Attach from MCP" Icon mit Popup-Menü). 10 zeigt, wie Cursor MCP-Tools in einer Liste darstellt.
    
- **Unterkomponenten (als separate Widgets oder Bereiche innerhalb der Sidebar):**
    
    - **`ServerListView`**:
        - **Anzeige:** Listet alle konfigurierten und/oder dynamisch erkannten MCP-Server auf. Zeigt für jeden Server:
            - Name/ID des Servers.
            - Verbindungsstatus (z.B. "Verbunden", "Getrennt", "Fehler") mit Icon.
            - Optionale Details (z.B. Protokollversion, Anzahl der bereitgestellten Tools/Ressourcen).
        - **Interaktion:**
            - Manuelles Verbinden/Trennen einzelner Server (ruft Methoden des `MCPConnectionService` auf).
            - Öffnen eines Konfigurationsdialogs für einen Server (falls serverseitige Konfiguration über MCP unterstützt wird oder für clientseitige Einstellungen wie Umgebungsvariablen 10).
            - Anzeigen von Server-Logs oder Fehlerdetails.
        - **Datenquelle:** Abonniert `allConnectedServers` und Statusänderungen vom `MCPGlobalContextManager` bzw. `MCPConnectionService`.
    - **`ToolListView`**:
        - **Anzeige:** Listet alle verfügbaren Tools von allen (oder einem ausgewählten) verbundenen Server(n).
            - Filteroptionen (nach Server, Kategorie, Suchbegriff).
            - Gruppierungsoptionen (z.B. nach Server, nach Funktionalität).
            - Für jedes Tool: Name, Beschreibung, Serverzugehörigkeit.
        - **Interaktion:**
            - Auswahl eines Tools führt zur Anzeige eines Parameter-Eingabebereichs (ggf. generiert durch `MCPWidgetFactory`).
            - Button zum Auslösen des Tools (ruft `toolOrchestrationService.callTool()` auf).
        - **Datenquelle:** Abonniert `allAvailableTools` vom `MCPGlobalContextManager`.
    - **`ResourceListView`**:
        - **Anzeige:** Analog zur `ToolListView` für MCP-Ressourcen.
        - **Interaktion:** Auswahl einer Ressource ermöglicht ggf. Eingabe von Zugriffsparametern und löst den Abruf über `resourceAccessService.getResourceData()` aus. Die abgerufenen Daten können direkt in der Sidebar oder in einem dedizierten Viewer angezeigt werden.
        - **Datenquelle:** Abonniert `allAvailableResources` vom `MCPGlobalContextManager`.
    - **`PromptListView`**:
        - **Anzeige:** Analog zur `ToolListView` für MCP-Prompts.
        - **Interaktion:** Auswahl eines Prompts führt zur Anzeige eines Parameter-Eingabebereichs für den Prompt und löst dessen Ausführung über `promptExecutionService.invokePrompt()` aus.
        - **Datenquelle:** Abonniert `allAvailablePrompts` vom `MCPGlobalContextManager`.
    - **`ActiveOperationsView`**:
        - **Anzeige:** Listet alle aktuell laufenden MCP-Operationen (Tool-Aufrufe, Ressourcenabrufe, Prompt-Ausführungen).
            - Für jede Operation: Name des Tools/Ressource/Prompts, Zielserver, Startzeit.
            - Fortschrittsanzeige (Balken oder Text), falls der Server `$/progress`-Notifications sendet und die `MCPClientInstance` diese weiterleitet.
        - **Interaktion:**
            - Möglichkeit, laufende Operationen abzubrechen (ruft `clientInstance.cancelRequest()` über den entsprechenden Service auf).
            - Anzeigen von Detail-Logs für eine Operation.
        - **Datenquelle:** Abonniert `pendingToolCalls` (und äquivalente Zustände für Ressourcen/Prompts) vom `MCPGlobalContextManager` sowie `Progress`-Events.
- **Eigenschaften (der gesamten `MCPSidebarView`):**
    
    - Abonniert relevante Zustände und Listen vom `MCPGlobalContextManager`, um ihre Unterkomponenten zu aktualisieren.
    - Kann einen eigenen internen Zustand für Filter, Sortierungen oder ausgewählte Elemente haben.
- **Methoden (primär interne Handler für Benutzerinteraktionen):**
    
    - Interagiert mit den Diensten aus Abschnitt 4 (`MCPConnectionService`, `ToolOrchestrationService` etc.), um Aktionen basierend auf Benutzereingaben in den Unterkomponenten auszulösen.
- **Event-Handling:**
    
    - Reagiert auf Klicks, Eingaben, Auswahländerungen in ihren Unterkomponenten.
    - Löst ggf. eigene UI-Events aus, um andere Teile der Anwendung zu benachrichtigen (z.B. "ToolXYWurdeAusgewählt").
- Relevanz:
    
    Die MCPSidebarView bietet einen zentralen und persistenten Ort für den Benutzer, um einen umfassenden Überblick über die verfügbaren MCP-Fähigkeiten zu erhalten und diese gezielt zu nutzen. Sie ergänzt das schnell zugängliche, aber flüchtige Kontextmenü. Die Sidebar könnte auch der Ort sein, an dem der Benutzer serverseitige Konfigurationen vornimmt, falls dies vom MCP-Server oder der Anwendung unterstützt wird.10
    

### 5.4. `MCPWidgetFactory`

- Zweck:
    
    Die MCPWidgetFactory ist eine Hilfskomponente, die dafür zuständig ist, dynamisch spezifische UI-Widgets für die Interaktion mit bestimmten MCP-Tools oder für die Anzeige von MCP-Ressourcendaten zu erzeugen. Die Generierung basiert auf den Schema-Definitionen, die von den MCP-Servern bereitgestellt werden (z.B. das JSON-Schema für Tool-Parameter 9).
    
- **Methoden:**
    
    - `public createWidgetForToolParams(toolDefinition: ToolDefinitionExtended, currentValues?: object): UIElement | null`
        - **Signatur:** `public createWidgetForToolParams(toolDefinition: ToolDefinitionExtended, currentValues?: object): UIElement | null`
        - **Beschreibung:**
            1. Analysiert das `toolDefinition.parametersSchema` (typischerweise ein JSON-Schema-Objekt 9).
            2. Basierend auf dem Schema generiert die Methode ein UI-Element (oder eine Sammlung von UI-Elementen), das Formularfelder für jeden Parameter des Tools bereitstellt.
            3. Unterstützte JSON-Schema-Typen und ihre UI-Entsprechungen (Beispiele):
                - `"type": "string"`: Text-Eingabefeld.
                - `"type": "string", "format": "date-time"`: Datums-/Zeitauswahl-Widget.
                - `"type": "number"`, `"type": "integer"`: Numerisches Eingabefeld (ggf. mit Min/Max-Validierung aus dem Schema).
                - `"type": "boolean"`: Checkbox oder Umschalter.
                - `"type": "array"` (mit `items` definiert): Liste von Eingabefeldern, ggf. mit Buttons zum Hinzufügen/Entfernen von Elementen.
                - `"type": "object"` (mit `properties` definiert): Gruppe von verschachtelten Eingabefeldern.
                - `"enum"`: Dropdown-Liste oder Radio-Buttons.
            4. Die generierten Widgets sollten Beschriftungen (aus `title` oder Property-Name im Schema), Platzhalter (aus `description` oder `examples`) und Validierungsregeln (aus `required`, `minLength`, `pattern` etc. im Schema) berücksichtigen.
            5. `currentValues` kann verwendet werden, um die Widgets mit vorhandenen Werten vorzubelegen.
            6. Gibt das Wurzelelement der generierten UI zurück oder `null`, wenn kein Schema vorhanden ist oder keine Parameter benötigt werden.
        - **Parameter:**
            - `toolDefinition: ToolDefinitionExtended`: Die Definition des Tools, inklusive seines Parameter-Schemas.
            - `currentValues?: object`: Optionale aktuelle Werte für die Parameter.
        - **Rückgabewert:** Ein `UIElement` (plattformspezifischer Typ für ein UI-Steuerelement oder einen Container) oder `null`.
    - `public createWidgetForResourceDisplay(resourceDefinition: ResourceDefinitionExtended, data: ResourceData, options?: DisplayOptions): UIElement | null`
        - **Signatur:** `public createWidgetForResourceDisplay(resourceDefinition: ResourceDefinitionExtended, data: ResourceData, options?: DisplayOptions): UIElement | null`
        - **Beschreibung:**
            1. Analysiert den Typ und die Struktur der `data` (ggf. unter Zuhilfenahme von Metadaten aus `resourceDefinition` oder MIME-Typen).
            2. Generiert ein UI-Element zur angemessenen Darstellung dieser Daten. Beispiele:
                - Textdaten: Mehrzeiliges Textfeld (ggf. mit Syntaxhervorhebung, wenn es sich um Code handelt).
                - JSON/XML-Daten: Strukturierte Baumansicht oder formatierter Text.
                - Bilddaten: Bildanzeige-Widget.10
                - Tabellarische Daten: Tabellenansicht.
                - Binärdaten: Hex-Viewer oder Download-Link.
            3. `options` können steuern, wie die Daten dargestellt werden (z.B. ob sie editierbar sein sollen, welche Felder angezeigt werden etc.).
        - **Parameter:**
            - `resourceDefinition: ResourceDefinitionExtended`: Die Definition der Ressource.
            - `data: ResourceData`: Die abgerufenen Ressourcendaten.
            - `options?: DisplayOptions`: Optionale Darstellungsoptionen.
        - **Rückgabewert:** Ein `UIElement` oder `null`.
- Relevanz:
    
    Die MCPWidgetFactory ermöglicht eine hochgradig flexible und typsichere Benutzeroberfläche für variable MCP-Interaktionen. Anstatt für jedes einzelne Tool oder jeden Ressourcentyp eine feste UI im Code zu implementieren, kann die UI dynamisch auf die vom Server bereitgestellten Schemata reagieren. Dies reduziert den Entwicklungsaufwand erheblich, wenn neue Tools oder Server mit unterschiedlichen Parameterstrukturen integriert werden, und stellt sicher, dass die UI immer die korrekten Eingabefelder und Darstellungen anbietet.
    
- Herausforderungen:
    
    Die Komplexität dieser Factory hängt stark von der Vielfalt und Komplexität der unterstützten JSON-Schema-Konstrukte und Ressourcendatenformate ab. Eine umfassende Implementierung, die alle Aspekte von JSON-Schema (bedingte Logik, komplexe Abhängigkeiten etc.) und eine breite Palette von Datenformaten abdeckt, kann sehr anspruchsvoll sein. Es ist ratsam, mit einer Unterstützung für die gängigsten Typen zu beginnen und die Factory iterativ zu erweitern.
    

### 5.5. `AICoPilotInterface` (oder `ChatInteractionManager`)

- Zweck:
    
    Die AICoPilotInterface ist die primäre UI-Komponente, über die der Benutzer direkt mit der KI-Funktionalität der Anwendung interagiert. Dies ist oft ein Chat-Fenster, eine erweiterte Eingabeaufforderung oder ein ähnliches Interface. Diese Komponente ist dafür verantwortlich, Benutzereingaben entgegenzunehmen, diese ggf. an ein LLM (entweder ein internes oder ein über MCP angebundenes) weiterzuleiten, MCP-Aktionen zu initiieren (basierend auf Benutzerbefehlen oder LLM-Vorschlägen) und die Ergebnisse – angereichert durch MCP-Tool-Ausgaben oder Ressourcendaten – dem Benutzer darzustellen. 4 beschreibt, wie Claude Desktop nach Bestätigung eines Tools dieses nutzt und Ergebnisse anzeigt. 23 erläutert die Interaktion mit GitHub Copilot über MCP.
    
- **Eigenschaften:**
    
    - `private conversationHistory: ChatMessage =;`
        - Eine Liste von `ChatMessage`-Objekten, die den bisherigen Dialogverlauf speichert.
    - `private inputField: TextInputElement;` (Plattformspezifisches UI-Element für Texteingabe)
    - `private sendButton: ButtonElement;`
    - `private mcpGlobalContextManager: MCPGlobalContextManager;` (Abhängigkeit)
    - `private toolOrchestrationService: ToolOrchestrationService;` (Abhängigkeit)
    - `private resourceAccessService: ResourceAccessService;` (Abhängigkeit)
    - `private promptExecutionService: PromptExecutionService;` (Abhängigkeit)
    - `private userConsentUIManager: UserConsentUIManager;` (Abhängigkeit)
    - `private currentLLMContext: any;` (Kontext, der an das LLM gesendet wird, z.B. vorherige Nachrichten, System-Prompt)
- **Methoden:**
    
    - `public constructor(...)`
        - Initialisiert UI-Elemente und Abhängigkeiten. Registriert Event-Listener für Eingabefeld (Enter-Taste) und Sende-Button.
    - `public async handleUserInput(text: string): Promise<void>`
        - **Signatur:** `public async handleUserInput(text: string): Promise<void>`
        - **Beschreibung:**
            1. Fügt die Benutzereingabe als `ChatMessage` zur `conversationHistory` hinzu und aktualisiert die UI.
            2. Leert das `inputField`.
            3. **Logik zur Intent-Erkennung:**
                - Prüft, ob `text` ein direkter Befehl zur Nutzung eines MCP-Tools/Ressource/Prompts ist (z.B. "/callTool meinTool --paramWert X").
                - Andernfalls wird `text` (zusammen mit `currentLLMContext`) an das zuständige LLM gesendet (dies kann ein internes LLM sein oder ein Aufruf an einen MCP-Server, der LLM-Funktionalität bereitstellt).
            4. Wenn ein direkter Befehl erkannt wurde: Ruft die entsprechende Methode des zuständigen MCP-Dienstes auf (z.B. `toolOrchestrationService.callTool`). Das Ergebnis wird dann über `displayAIResponse` oder `displayError` angezeigt.
            5. Wenn die Eingabe an ein LLM geht: Wartet auf die Antwort des LLMs. Die LLM-Antwort kann Text, einen Vorschlag zur Tool-Nutzung oder eine Kombination davon sein.
    - `public displayAIResponse(response: AIResponse): void`
        - **Signatur:** `public displayAIResponse(response: AIResponse): void noexcept` (`AIResponse` könnte `{ text?: string, toolCallSuggestion?: ModelInitiatedToolCall, mcpData?: any }` sein)
        - **Beschreibung:**
            1. Fügt die KI-Antwort als `ChatMessage` zur `conversationHistory` hinzu und aktualisiert die UI.
            2. Wenn `response.toolCallSuggestion` vorhanden ist, wird `this.handleToolSuggestion(response.toolCallSuggestion)` aufgerufen.
            3. Wenn `response.mcpData` vorhanden ist (z.B. direkt abgerufene Ressourcendaten, die Teil der Antwort sind), wird dies entsprechend formatiert und angezeigt (ggf. mit `MCPWidgetFactory`).
    - `private async handleToolSuggestion(toolCallRequest: ModelInitiatedToolCall): Promise<void>`
        - **Signatur:** `private async handleToolSuggestion(toolCallRequest: ModelInitiatedToolCall): Promise<void>` (`ModelInitiatedToolCall` enthält `toolId`, `params`)
        - **Beschreibung:** Wird aufgerufen, wenn das LLM vorschlägt, ein MCP-Tool zu verwenden.
            1. Ruft `toolOrchestrationService.getToolDefinition(toolCallRequest.toolId)` ab.
            2. Ruft `userConsentUIManager.requestConsentForTool(definition, toolCallRequest.params, this.getWindowId())` auf.
            3. Bei Zustimmung: Ruft `toolOrchestrationService.callTool(toolCallRequest.toolId, toolCallRequest.params, this.getWindowId())` auf. Das Ergebnis dieses Aufrufs wird dann typischerweise wieder an das LLM gesendet (als Teil des nächsten `currentLLMContext`), damit es seine Antwort darauf basierend formulieren kann. Dieser Schritt ist Teil des "Agenten-Loops".
            4. Bei Ablehnung: Informiert das LLM (optional) oder zeigt eine entsprechende Nachricht an.
    - `public displayError(error: MCPError | Error): void`
        - **Signatur:** `public displayError(error: MCPError | Error): void noexcept`
        - **Beschreibung:** Zeigt eine Fehlermeldung im Chat-Interface an.
    - `public clearConversation(): void`
        - **Signatur:** `public clearConversation(): void noexcept`
        - **Beschreibung:** Leert die `conversationHistory` und aktualisiert die UI.
- Relevanz:
    
    Die AICoPilotInterface ist oft das "Gesicht" der KI-Kollaboration für den Benutzer. Ihre Fähigkeit, nahtlos zwischen reiner Textkonversation, der Nutzung von MCP-Tools (initiiert durch Benutzer oder LLM) und der Darstellung von Ergebnissen zu wechseln, ist entscheidend für eine positive Benutzererfahrung. Sie muss eng mit dem zugrundeliegenden LLM (falls die UI-Anwendung eines direkt steuert) oder dem MCP-Server (falls dieser das LLM steuert und Tool-Aufrufe vorschlägt) zusammenarbeiten. Sie ist der primäre Ort, an dem der komplexe "Dialog" zwischen Benutzer, LLM und den über MCP angebundenen externen Fähigkeiten stattfindet und sichtbar wird.
    

## 6. Detaillierte Event-Spezifikationen und Datenstrukturen

Dieser Abschnitt definiert die detaillierten Strukturen für UI-interne Events, die für die Kommunikation zwischen den MCP-Modulen verwendet werden, sowie die zentralen Datenobjekte (Entitäten und Wertobjekte), die MCP-Konzepte innerhalb der UI-Schicht repräsentieren. Zusätzlich werden die exakten JSON-RPC-Nachrichtenstrukturen aus Sicht des Clients spezifiziert.

### 6.1. UI-Interne Events für MCP-Operationen

Um eine lose Kopplung zwischen den verschiedenen UI-Modulen und -Komponenten zu erreichen, wird ein internes Event-System (z.B. basierend auf dem Observer-Pattern oder einem dedizierten Pub/Sub-Mechanismus) verwendet. Dies ermöglicht es Komponenten, auf Zustandsänderungen und abgeschlossene Operationen zu reagieren, ohne direkte Abhängigkeiten voneinander zu haben. Ein robustes Event-System ist entscheidend für die Skalierbarkeit und Wartbarkeit der UI, insbesondere bei der Handhabung asynchroner Operationen wie MCP-Aufrufen, und hilft, komplexe Callback-Ketten ("Callback Hell") zu vermeiden.

Für jedes definierte Event werden folgende Aspekte spezifiziert:

- **Eindeutiger Event-Name/Typ:** Eine klare und eindeutige Bezeichnung für das Event (z.B. als String-Konstante oder Enum-Wert).
- **Payload-Struktur (Typdefinition):** Die genaue Definition der Daten, die mit dem Event transportiert werden.
- **Typische Publisher:** Die Komponente(n) oder der/die Dienst(e), die dieses Event typischerweise auslösen.
- **Typische Subscriber:** Die Komponenten oder Dienste, die typischerweise auf dieses Event reagieren.
- **Beschreibung:** Kurze Erläuterung des Zwecks und des Kontexts des Events.

**Beispiele für UI-interne Events:**

- **Event: `mcp:ServerConnectionStatusChanged`**
    - **Payload:** `{ serverId: ServerId, newStatus: ConnectionStatus, clientInstance?: MCPClientInstance, error?: MCPError }`
    - **Publisher:** `MCPConnectionService` (via `MCPClientInstance`)
    - **Subscriber:** `MCPGlobalContextManager`, `MCPSidebarView.ServerListView`, ggf. andere UI-Komponenten, die den Serverstatus anzeigen.
    - **Beschreibung:** Wird ausgelöst, wenn sich der Verbindungsstatus eines MCP-Servers ändert.
- **Event: `mcp:ClientInstanceAdded`**
    - **Payload:** `{ client: MCPClientInstance }`
    - **Publisher:** `MCPConnectionService`
    - **Subscriber:** `MCPGlobalContextManager`, `MCPSidebarView.ServerListView`
    - **Beschreibung:** Wird ausgelöst, nachdem eine neue `MCPClientInstance` erstellt und der initiale Verbindungsversuch gestartet wurde.
- **Event: `mcp:ClientInstanceRemoved`**
    - **Payload:** `{ serverId: ServerId, reason?: 'disconnected' | 'error' }`
    - **Publisher:** `MCPConnectionService`
    - **Subscriber:** `MCPGlobalContextManager`, `MCPSidebarView.ServerListView`
    - **Beschreibung:** Wird ausgelöst, nachdem eine `MCPClientInstance` entfernt wurde.
- **Event: `mcp:ToolListUpdated`**
    - **Payload:** `{ tools: ToolDefinitionExtended }`
    - **Publisher:** `ToolOrchestrationService`
    - **Subscriber:** `MCPGlobalContextManager`, `MCPSidebarView.ToolListView`, `MCPContextualMenuController`
    - **Beschreibung:** Wird ausgelöst, wenn die Liste der verfügbaren Tools aktualisiert wurde.
- **Event: `mcp:ResourceListUpdated`**
    - **Payload:** `{ resources: ResourceDefinitionExtended }`
    - **Publisher:** `ResourceAccessService`
    - **Subscriber:** `MCPGlobalContextManager`, `MCPSidebarView.ResourceListView`, `MCPContextualMenuController`
    - **Beschreibung:** Wird ausgelöst, wenn die Liste der verfügbaren Ressourcen aktualisiert wurde.
- **Event: `mcp:PromptListUpdated`**
    - **Payload:** `{ prompts: PromptDefinitionExtended }`
    - **Publisher:** `PromptExecutionService`
    - **Subscriber:** `MCPGlobalContextManager`, `MCPSidebarView.PromptListView`, `MCPContextualMenuController`
    - **Beschreibung:** Wird ausgelöst, wenn die Liste der verfügbaren Prompts aktualisiert wurde.
- **Event: `mcp:ToolCallStarted`**
    - **Payload:** `{ callId: string, toolId: GlobalToolId, params: object }` (callId ist eine eindeutige ID für diesen spezifischen Aufruf)
    - **Publisher:** `ToolOrchestrationService`
    - **Subscriber:** `MCPGlobalContextManager` (zur Aktualisierung von `pendingToolCalls`), `MCPSidebarView.ActiveOperationsView`
    - **Beschreibung:** Wird ausgelöst, bevor ein Tool-Aufruf an den Server gesendet wird (nach Zustimmung).
- **Event: `mcp:ToolCallCompleted`**
    - **Payload:** `{ callId: string, toolId: GlobalToolId, result: any | MCPError }`
    - **Publisher:** `ToolOrchestrationService`
    - **Subscriber:** `MCPGlobalContextManager`, `MCPSidebarView.ActiveOperationsView`, `AICoPilotInterface`
    - **Beschreibung:** Wird ausgelöst, nachdem ein Tool-Aufruf abgeschlossen ist (erfolgreich oder fehlerhaft).
- **Event: `mcp:ResourceAccessCompleted`** (analog zu `ToolCallCompleted`)
- **Event: `mcp:PromptExecutionCompleted`** (analog zu `ToolCallCompleted`)
- **Event: `mcp:ProgressNotificationReceived`**
    - **Payload:** `{ callId: string, progressToken: string | number, progressData: any }`
    - **Publisher:** `MCPClientInstance` (nach Empfang einer `$/progress` Notification)
    - **Subscriber:** `MCPGlobalContextManager` (zur Aktualisierung von `pendingToolCalls`), `MCPSidebarView.ActiveOperationsView`
    - **Beschreibung:** Wird ausgelöst, wenn eine Fortschrittsbenachrichtigung vom Server empfangen wird.
- **Event: `ui:ContextMenuRequestMcptool`**
    - **Payload:** `{ context: AppSpecificContext, position: {x: number, y: number} }`
    - **Publisher:** UI-Elemente, auf denen ein Rechtsklick erfolgt.
    - **Subscriber:** `MCPContextualMenuController` (oder ein übergeordneter UI-Manager, der das Kontextmenü anzeigt).
    - **Beschreibung:** Signalisiert, dass ein Kontextmenü mit MCP-Aktionen für den gegebenen Kontext angefordert wird.

### Tabelle 3: UI-Interne MCP-Events

|   |   |   |   |   |
|---|---|---|---|---|
|**Event-Name/Typ**|**Payload-Schema (Beispiel)**|**Typische(r) Publisher**|**Typische(r) Subscriber**|**Kurzbeschreibung des Zwecks**|
|`mcp:ServerConnectionStatusChanged`|`{ serverId, newStatus, clientInstance?, error? }`|`MCPConnectionService`|`MCPGlobalContextManager`, `MCPSidebarView.ServerListView`|Änderung des Server-Verbindungsstatus.|
|`mcp:ClientInstanceAdded`|`{ client }`|`MCPConnectionService`|`MCPGlobalContextManager`, `MCPSidebarView.ServerListView`|Neue MCP-Client-Instanz hinzugefügt.|
|`mcp:ClientInstanceRemoved`|`{ serverId, reason? }`|`MCPConnectionService`|`MCPGlobalContextManager`, `MCPSidebarView.ServerListView`|MCP-Client-Instanz entfernt.|
|`mcp:ToolListUpdated`|`{ tools }`|`ToolOrchestrationService`|`MCPGlobalContextManager`, `MCPSidebarView.ToolListView`, `MCPContextualMenuController`|Liste der verfügbaren Tools aktualisiert.|
|`mcp:ResourceListUpdated`|`{ resources }`|`ResourceAccessService`|`MCPGlobalContextManager`, `MCPSidebarView.ResourceListView`, `MCPContextualMenuController`|Liste der verfügbaren Ressourcen aktualisiert.|
|`mcp:PromptListUpdated`|`{ prompts }`|`PromptExecutionService`|`MCPGlobalContextManager`, `MCPSidebarView.PromptListView`, `MCPContextualMenuController`|Liste der verfügbaren Prompts aktualisiert.|
|`mcp:ToolCallCompleted`|`{ callId, toolId, result }`|`ToolOrchestrationService`|`MCPGlobalContextManager`, `AICoPilotInterface`|Ein Tool-Aufruf wurde abgeschlossen.|
|`mcp:ProgressNotificationReceived`|`{ callId, progressToken, progressData }`|`MCPClientInstance`|`MCPGlobalContextManager`, `MCPSidebarView.ActiveOperationsView`|Fortschrittsinfo vom Server erhalten.|

### 6.2. Objekte und Wertobjekte (Entitäten) für MCP-bezogene Daten

Dieser Unterabschnitt definiert die zentralen Datenstrukturen (Objekte und Wertobjekte), die MCP-Konzepte innerhalb der UI-Schicht repräsentieren. Diese Strukturen werden für die interne Datenhaltung, die Kommunikation zwischen Modulen und die Konfiguration verwendet. MCP-Nachrichten selbst enthalten Daten (Tool-Parameter, Ressourcen-Inhalte), die in diese Strukturen abgebildet werden müssen. Beispielsweise muss das `parameters_schema` eines Tools 9 in einer internen `ToolDefinition`-Struktur gespeichert werden können, damit die `MCPWidgetFactory` daraus eine UI generieren kann. Die `rust-mcp-schema` Bibliothek 13 dient als gute Referenz für typsichere Schemata, auch wenn die Zielsprache dieses Dokuments nicht Rust ist.

Für jede Entität (mit Identität, potenziell veränderlich) und jedes Wertobjekt (unveränderlich, durch seine Werte definiert) werden folgende Aspekte spezifiziert:

- **Name:** Der Klassen- oder Typname.
- **Typ:** Entität oder Wertobjekt.
- **Attribute:**
    - `name: string` (Attributname)
    - `type: DataType` (z.B. `string`, `number`, `boolean`, `JSONSchemaObject`, `URI`, oder ein anderer definierter Typ)
    - `visibility: public | private | protected` (aus Sicht der Klasse)
    - `initialValue?: any` (Optionaler Initialwert)
    - `readonly?: boolean` (Ob das Attribut nach Initialisierung unveränderbar ist)
    - `invariants: string` (Bedingungen, die für das Objekt immer gelten müssen, als textuelle Beschreibung)
- **Methoden (falls zutreffend, insbesondere für Entitäten mit Verhalten):**
    - Signaturen (Parameter: Name, Typ; Rückgabetyp; `const` und `noexcept` sind hier weniger relevant, da es sich um Sprachkonstrukte handelt, die von der Zielsprache abhängen. Wichtig sind Parameter und Rückgabetypen).
    - Vor- und Nachbedingungen.
    - Geschäftsregeln, die sie durchsetzen.
- **Beziehungen zu anderen Entitäten/Wertobjekten.**

**Beispiele für Entitäten und Wertobjekte:**

- **`MCPServerConfig` (Wertobjekt)**
    
    - Basierend auf.10
    - Attribute:
        - `id: string` (public, readonly): Eindeutige ID für diese Serverkonfiguration (z.B. ein Hash des Namens oder manuell vergeben).
        - `name: string` (public, readonly): Anzeigename des Servers.
        - `transportType: 'stdio' | 'sse'` (public, readonly): Der zu verwendende Transportmechanismus.
        - `command?: string` (public, readonly): Das auszuführende Kommando (nur bei `transportType === 'stdio'`).
        - `args?: string` (public, readonly): Argumente für das Kommando (nur bei `transportType === 'stdio'`).
        - `url?: string` (public, readonly): Die URL des SSE-Endpunkts (nur bei `transportType === 'sse'`).
        - `env?: Record<string, string>` (public, readonly): Umgebungsvariablen für den Serverprozess (primär für `stdio`).
        - `isTrusted?: boolean` (public, readonly, initialValue: `false`): Gibt an, ob diesem Server standardmäßig vertraut wird.
    - Invarianten:
        - "Wenn `transportType` 'stdio' ist, MUSS `command` definiert sein."
        - "Wenn `transportType` 'sse' ist, MUSS `url` definiert sein."
- **`ClientCapabilities` (Wertobjekt)**
    
    - Attribute:
        - `sampling?: { [key: string]: any }` (public, readonly): Optionen für Sampling, falls vom Client unterstützt.3
        - `otherCapabilities?: { [key: string]: any }` (public, readonly): Platz für weitere Client-spezifische Fähigkeiten.
- **`ServerInfo` (Wertobjekt)**
    
    - Empfangen vom Server während `initialize`.5
    - Attribute:
        - `name: string` (public, readonly): Name des Servers.
        - `version: string` (public, readonly): Version des Servers.
        - `meta?: { [key: string]: any }` (public, readonly): Zusätzliche Metadaten über den Server.
- **`ServerCapabilities` (Wertobjekt)**
    
    - Empfangen vom Server während `initialize`.5
    - Attribute:
        - `tools?: { [toolName: string]: ToolDefinitionFromServer }` (public, readonly): Map von Tool-Namen zu deren Definitionen.
        - `resources?: { [resourceName: string]: ResourceDefinitionFromServer }` (public, readonly): Map von Ressourcen-Namen zu deren Definitionen.
        - `prompts?: { [promptName: string]: PromptDefinitionFromServer }` (public, readonly): Map von Prompt-Namen zu deren Definitionen.
        - `protocolExtensions?: string` (public, readonly): Liste der unterstützten Protokollerweiterungen.
- **`ToolDefinitionFromServer` (Wertobjekt)** (Basis für `ToolDefinitionExtended`)
    
    - Attribute:
        - `description: string` (public, readonly): Beschreibung des Tools.9
        - `parametersSchema?: JSONSchemaObject` (public, readonly): JSON-Schema für die Parameter des Tools.9
        - `responseSchema?: JSONSchemaObject` (public, readonly): JSON-Schema für das Ergebnis des Tools (optional).
        - `annotations?: { [key: string]: any }` (public, readonly): Zusätzliche Annotationen, z.B. Kategorien, anwendbare Kontexte.
- **`ToolDefinitionExtended` (Wertobjekt)** (Intern in der UI verwendet)
    
    - Erbt/kombiniert `ToolDefinitionFromServer`.
    - Zusätzliche Attribute:
        - `name: string` (public, readonly): Der Name des Tools (Schlüssel aus `ServerCapabilities.tools`).
        - `globalId: GlobalToolId` (public, readonly): Eindeutige ID über alle Server.
        - `serverId: ServerId` (public, readonly): ID des Servers, der dieses Tool bereitstellt.
- **`ResourceDefinitionFromServer` / `ResourceDefinitionExtended`** (analog zu Tools)
    
- **`PromptDefinitionFromServer` / `PromptDefinitionExtended`** (analog zu Tools)
    
- **`JSONSchemaObject` (Wertobjekt)**
    
    - Repräsentiert ein JSON-Schema. Die genaue Struktur ist durch die JSON-Schema-Spezifikation definiert (z.B. `type`, `properties`, `items`, `required`, etc.).
- **`ChatMessage` (Wertobjekt)**
    
    - Attribute:
        - `id: string` (public, readonly): Eindeutige ID der Nachricht.
        - `sender: 'user' | 'ai' | 'system'` (public, readonly): Absender der Nachricht.
        - `text?: string` (public, readonly): Textinhalt der Nachricht.
        - `toolCallRequest?: ModelInitiatedToolCall` (public, readonly): Falls die KI ein Tool aufrufen möchte.
        - `toolCallResult?: { toolId: GlobalToolId, resultData: any }` (public, readonly): Ergebnis eines Tool-Aufrufs, das angezeigt wird.
        - `timestamp: Date` (public, readonly): Zeitstempel der Nachricht.
        - `relatedMcpCallId?: string` (public, readonly): ID des zugehörigen MCP-Aufrufs (für Korrelation).
        - `uiElement?: UIElement` (public, readonly): Optional ein spezielles UI-Element zur Darstellung (z.B. für Bilder, Karten).
- **`ConnectionStatus` (Enum/String-Literal Union)**
    
    - Werte: `Idle`, `Connecting`, `Initializing`, `Connected`, `Reconnecting`, `Disconnecting`, `Disconnected`, `Error`.

### Tabelle 5: Entitäten und Wertobjekte – Schlüsselliste

|   |   |   |   |   |
|---|---|---|---|---|
|**Objektname**|**Typ (Entität/Wertobjekt)**|**Kurzbeschreibung/Zweck**|**Wichtige Attribute (Beispiele)**|**Beziehung zu anderen Objekten (Beispiele)**|
|`MCPServerConfig`|Wertobjekt|Konfiguration für die Verbindung zu einem MCP-Server.|`id`, `name`, `transportType`, `command`/`url`|-|
|`ClientCapabilities`|Wertobjekt|Fähigkeiten, die der UI-Client dem Server anbietet.|`sampling`|-|
|`ServerInfo`|Wertobjekt|Vom Server empfangene Metainformationen.|`name`, `version`|-|
|`ServerCapabilities`|Wertobjekt|Vom Server empfangene Liste seiner Fähigkeiten.|`tools`, `resources`, `prompts`|Enthält `ToolDefinitionFromServer` etc.|
|`ToolDefinitionFromServer`|Wertobjekt|Definition eines Tools, wie vom Server bereitgestellt.|`description`, `parametersSchema`|Verwendet `JSONSchemaObject`.|
|`ToolDefinitionExtended`|Wertobjekt|UI-interne, erweiterte Tool-Definition.|`globalId`, `serverId`, `name`|Basiert auf `ToolDefinitionFromServer`.|
|`JSONSchemaObject`|Wertobjekt|Repräsentation eines JSON-Schemas.|`type`, `properties`, `required`|-|
|`ChatMessage`|Wertobjekt|Einzelne Nachricht in einer Konversation (z.B. im Chat).|`sender`, `text`, `timestamp`, `toolCallRequest`|-|
|`MCPError`|Entität (da Zustand wie `originalError` sich ändern könnte, aber oft als Wertobjekt behandelt)|Basisklasse für MCP-spezifische Fehler.|`message`, `jsonRpcError`|Kann `JsonRpcErrorObject` enthalten.|
|`ConnectionStatus`|Enum/Wertobjekt|Mögliche Zustände einer MCP-Verbindung.|- (`Idle`, `Connected`, etc.)|-|

### 6.3. JSON-RPC Nachrichtenstrukturen (Client-Perspektive) für MCP-Kommunikation

Dieser Unterabschnitt spezifiziert die exakten JSON-Payloads für die wichtigsten MCP-Methoden, die der Client (die UI-Anwendung) an den Server sendet, sowie die Struktur der erwarteten Antworten. Dies ist kritisch für Entwickler, die die Kommunikationsschicht in `MCPClientInstance` implementieren. Die `id` in JSON-RPC Requests 5 muss sorgfältig verwaltet werden (eindeutig pro Request), um Antworten den richtigen Anfragen zuordnen zu können, insbesondere bei nebenläufigen Aufrufen an denselben Server.

**Allgemeine JSON-RPC Struktur:**

- **Request:**
    
    JSON
    
    ```
    {
      "jsonrpc": "2.0",
      "method": "method_name",
      "params": { /* Parameterobjekt */ } /* oder [Parameterarray] */,
      "id": "eindeutige_id_string_oder_zahl" /* oder weggelassen für Notifications */
    }
    ```
    
- **Response (Erfolg):**
    
    JSON
    
    ```
    {
      "jsonrpc": "2.0",
      "result": { /* Ergebnisobjekt oder Primitivwert */ },
      "id": "gleiche_id_wie_request"
    }
    ```
    
- **Response (Fehler):**
    
    JSON
    
    ```
    {
      "jsonrpc": "2.0",
      "error": {
        "code": -32xxx, /* Fehlercode (Integer) */
        "message": "Fehlerbeschreibung (String)",
        "data": { /* Optionale zusätzliche Fehlerdetails */ }
      },
      "id": "gleiche_id_wie_request" /* oder null bei bestimmten Fehlern vor ID-Verarbeitung */
    }
    ```
    

**Spezifische Methoden:**

1. **`initialize`** 5
    
    - **Request Payload:**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "method": "initialize",
          "params": {
            "protocolVersion": "2025-03-26", // Die vom Client unterstützte MCP-Version
            "capabilities": { // ClientCapabilities Objekt
              "sampling": {}, // Beispiel
              // weitere Client-Fähigkeiten
            },
            "clientInfo": { // ClientInfo Objekt
              "name": "UIAnwendungsName",
              "version": "UIAnwendungsVersion",
              "meta": { /* optionale Metadaten über den Client */ }
            }
          },
          "id": "init_1"
        }
        ```
        
    - **Response Payload (Erfolg):**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "result": { // ServerInfo & ServerCapabilities Objekt
            "protocolVersion": "2025-03-26", // Die vom Server gewählte/bestätigte MCP-Version
            "serverInfo": {
              "name": "MCPTestServer",
              "version": "0.1.0",
              "meta": { /* optionale Metadaten über den Server */ }
            },
            "capabilities": {
              "tools": { /* Map von ToolDefinitionFromServer */ },
              "resources": { /* Map von ResourceDefinitionFromServer */ },
              "prompts": { /* Map von PromptDefinitionFromServer */ },
              "protocolExtensions": ["ext1", "ext2"]
            },
            "instructions": "Optionale Anweisungen vom Server an den Client"
          },
          "id": "init_1"
        }
        ```
        
    - **Response Payload (Error Beispiel):**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "error": {
            "code": -32602, // Invalid params
            "message": "Unsupported protocolVersion",
            "data": { "supportedVersions": ["2024-11-05"] }
          },
          "id": "init_1"
        }
        ```
        
2. **`tools/list`** 17
    
    - **Request Payload:**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "method": "tools/list",
          "params": {
            // Optionale Filterparameter, z.B. "categories": ["cat1"]
          },
          "id": "tools_list_1"
        }
        ```
        
    - **Response Payload (Erfolg):**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "result":
              }
            }
            //... weitere Tools
          ],
          "id": "tools_list_1"
        }
        ```
        
3. **`tools/call`** 17
    
    - **Request Payload:**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "method": "tools/call",
          "params": {
            "name": "get_weather", // Name des aufzurufenden Tools
            "arguments": { // Objekt mit den Tool-Parametern
              "location": "Berlin"
            }
          },
          "id": "tool_call_123"
        }
        ```
        
    - **Response Payload (Erfolg):**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "result": { /* Ergebnis des Tool-Aufrufs, Struktur ist tool-spezifisch */
            "temperature": "15°C",
            "condition": "Cloudy"
          },
          "id": "tool_call_123"
        }
        ```
        
4. **`resources/list`** (analog zu `tools/list`)
    
    - **Request Payload:**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "method": "resources/list",
          "params": {},
          "id": "res_list_1"
        }
        ```
        
    - **Response Payload (Erfolg):** Array von `ResourceDefinitionFromServer`-Objekten.
5. **`resources/get`** (analog zu `tools/call` für den Abruf)
    
    - **Request Payload:**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "method": "resources/get",
          "params": {
            "name": "document.txt",
            "accessParams": { /* optionale Zugriffsparameter */ }
          },
          "id": "res_get_1"
        }
        ```
        
    - **Response Payload (Erfolg):** `result` enthält die Ressourcendaten (Struktur ist ressourcenspezifisch).
6. **`ping`** 5
    
    - **Request Payload:**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "method": "ping",
          "params": { "payload": "optional_client_data" },
          "id": "ping_1"
        }
        ```
        
    - **Response Payload (Erfolg):**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "result": { "payload": "optional_server_data_echoing_client_data" },
          "id": "ping_1"
        }
        ```
        
7. **`$/cancelRequest` (Notification)** 3
    
    - **Request Payload (Notification, daher keine `id` im Request und keine Response erwartet):**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "method": "$/cancelRequest",
          "params": {
            "id": "tool_call_123" // ID des Requests, der abgebrochen werden soll
          }
        }
        ```
        
8. **`$/progress` (Notification vom Server an Client)** 3
    
    - **Payload (vom Server empfangen):**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "method": "$/progress",
          "params": {
            "token": "progress_token_fuer_tool_call_123", // Korreliert mit einem laufenden Request
            "value": { /* Fortschrittsdaten, Struktur ist operationsspezifisch */
              "percentage": 50,
              "message": "Processing data..."
            }
          }
        }
        ```
        
9. **`shutdown`** 9
    
    - **Request Payload (kann Request oder Notification sein, je nach Server-Erwartung):**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "method": "shutdown",
          "params": {},
          "id": "shutdown_1" // falls als Request
        }
        ```
        
    - **Response Payload (Erfolg, falls als Request):**
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "result": null, // Typischerweise null bei Erfolg
          "id": "shutdown_1"
        }
        ```
        

Diese detaillierten Strukturen sind essenziell für die korrekte Implementierung der Kommunikationslogik. Abweichungen können zu Inkompatibilitäten mit MCP-Servern führen.

## 7. Implementierungsrichtlinien und Lebenszyklusmanagement

Dieser Abschnitt bietet praktische Anleitungen für typische Implementierungsaufgaben im Kontext der MCP-Integration und behandelt wichtige Aspekte des Lebenszyklusmanagements von UI-Komponenten sowie der Nebenläufigkeit.

### 7.1. Schritt-für-Schritt-Anleitungen für typische Implementierungsaufgaben

Diese Anleitungen sollen Entwicklern den Einstieg erleichtern und konsistente Implementierungsmuster fördern.

#### 7.1.1. Hinzufügen eines neuen MCP-Servers zur Konfiguration und UI

1. **Konfiguration erweitern:**
    - Der Benutzer (oder Administrator) fügt die Details des neuen MCP-Servers zur zentralen Konfigurationsquelle hinzu (z.B. die `mcp.json`-Datei 10 oder eine Datenbank). Dies beinhaltet `id`, `name`, `transportType` und die transport-spezifischen Details (`command`/`args` für stdio, `url` für SSE).
2. **`MCPConnectionService` informieren:**
    - Beim Start der Anwendung oder bei einer dynamischen Konfigurationsänderung lädt der `MCPConnectionService` die aktualisierten Konfigurationen (z.B. über `loadAndInitializeConnections()`).
    - Für den neuen Server wird eine `MCPClientInstance` erstellt und `connectAndInitialize()` aufgerufen.
3. **Status-Updates verarbeiten:**
    - Der `MCPGlobalContextManager` und die `MCPSidebarView.ServerListView` abonnieren Status-Events vom `MCPConnectionService`.
    - Sobald die neue `MCPClientInstance` hinzugefügt wird und ihren Status ändert (z.B. zu `Connected`), wird die UI automatisch aktualisiert, um den neuen Server anzuzeigen.
4. **Fähigkeiten abrufen und anzeigen:**
    - Nach erfolgreicher Initialisierung des neuen Servers rufen die Dienste (`ToolOrchestrationService`, `ResourceAccessService`, `PromptExecutionService`) dessen Fähigkeiten ab (via `client.listTools()` etc.).
    - Diese Dienste aktualisieren den `MCPGlobalContextManager`.
    - UI-Komponenten wie `MCPSidebarView.ToolListView` reagieren auf die Aktualisierung im `MCPGlobalContextManager` und zeigen die neuen Tools/Ressourcen/Prompts an.

#### 7.1.2. Implementierung eines neuen UI-Widgets, das ein MCP-Tool aufruft

1. **Widget-Design:**
    - Entwurf des UI-Widgets (z.B. ein Button mit Beschriftung oder ein komplexeres Formular).
2. **Abhängigkeiten injizieren:**
    - Das Widget erhält Zugriff auf den `ToolOrchestrationService` und ggf. den `UserConsentUIManager` (oder löst Events aus, die von einem Controller mit diesen Diensten verarbeitet werden).
3. **Aktion auslösen:**
    - Bei einer Benutzerinteraktion (z.B. Klick) ruft das Widget die Methode `toolOrchestrationService.callTool(toolId, params, parentWindowId)` auf.
    - `toolId` ist die `GlobalToolId` des gewünschten Tools.
    - `params` werden entweder im Widget selbst gesammelt (z.B. aus Eingabefeldern) oder sind vordefiniert.
    - `parentWindowId` wird übergeben, falls das Widget Teil eines modalen Dialogs ist oder um den Zustimmungsdialog korrekt zuzuordnen.
4. **Ergebnisverarbeitung:**
    - Das Widget behandelt das zurückgegebene Promise von `callTool`.
    - Bei Erfolg: Zeigt das Ergebnis an oder löst ein weiteres Event mit dem Ergebnis aus.
    - Bei Fehler (`MCPError` oder `MCPConsentDeniedError`): Zeigt eine benutzerfreundliche Fehlermeldung an.
5. **Statusanzeige (optional):**
    - Das Widget kann den `MCPGlobalContextManager` abonnieren, um den Status des Tool-Aufrufs (aus `pendingToolCalls`) anzuzeigen und z.B. während der Ausführung deaktiviert zu werden.

#### 7.1.3. Anzeigen von Daten aus einer MCP-Ressource in einer neuen Ansicht

1. **Ansicht-Design:**
    - Entwurf der UI-Ansicht, die die Ressourcendaten darstellen soll.
2. **Datenabruf initiieren:**
    - Die Ansicht (oder ihr Controller) ruft `resourceAccessService.getResourceData(resourceId, params, parentWindowId)` auf, um die Daten zu laden.
3. **Datenaufbereitung und -darstellung:**
    - Nach erfolgreichem Abruf werden die Rohdaten (`ResourceData`) empfangen.
    - Die `MCPWidgetFactory` kann verwendet werden (`createWidgetForResourceDisplay()`), um ein passendes UI-Element für die Darstellung der Daten zu generieren, basierend auf dem Datentyp oder der `ResourceDefinition`.
    - Das generierte Widget wird in die Ansicht eingefügt.
4. **Fehlerbehandlung:**
    - Fehler beim Abruf werden in der Ansicht angezeigt.

#### 7.1.4. Behandlung eines neuen Typs von MCP-Notification

1. **`MCPClientInstance` erweitern:**
    - In `MCPClientInstance.handleIncomingMessage()`: Logik hinzufügen, um Notifications mit dem neuen Methodennamen zu erkennen.
2. **Event definieren:**
    - Ein neues UI-internes Event (z.B. `mcp:CustomNotificationReceived`) mit einer passenden Payload-Struktur definieren (siehe Abschnitt 6.1).
3. **Event auslösen:**
    - Die `MCPClientInstance` löst dieses neue Event aus, wenn die entsprechende Notification empfangen wird.
4. **Subscriber implementieren:**
    - Relevante Dienste oder UI-Komponenten (z.B. `MCPGlobalContextManager` oder spezifische Widgets) abonnieren dieses neue Event.
    - Die Subscriber implementieren die Logik zur Verarbeitung der Notification-Payload und zur Aktualisierung des UI-Zustands oder der Anzeige.

### 7.2. Lebenszyklusmanagement für MCP-bezogene UI-Komponenten mit komplexem Zustand

UI-Komponenten, die MCP-Daten halten, MCP-Verbindungen repräsentieren oder auf MCP-Events reagieren (wie die Unterkomponenten der `MCPSidebarView` oder dynamisch generierte Widgets), erfordern ein sorgfältiges Lebenszyklusmanagement, um Speicherlecks, veraltete Zustände und unnötige Ressourcenbindung zu vermeiden.

- **Initialisierung:**
    - Komponenten sollten ihre Abhängigkeiten (Dienste, ContextManager) im Konstruktor oder einer Initialisierungsmethode erhalten.
    - Abonnements auf Events oder reaktive Zustände sollten bei der Initialisierung oder wenn die Komponente sichtbar/aktiv wird, eingerichtet werden.
    - Initialdaten sollten von den Diensten oder dem `MCPGlobalContextManager` abgerufen werden.
- **Aktualisierung:**
    - Komponenten müssen auf Änderungen im globalen MCP-Zustand oder auf spezifische Events reagieren und ihre Darstellung entsprechend aktualisieren. Dies sollte effizient geschehen, um die UI-Performance nicht zu beeinträchtigen.
    - Bei der Aktualisierung von Daten (z.B. einer Tool-Liste) sollte darauf geachtet werden, bestehende UI-Elemente intelligent wiederzuverwenden oder zu aktualisieren, anstatt die gesamte Ansicht neu zu erstellen, falls das UI-Toolkit dies unterstützt.
- **Zerstörung (Deregistrierung):**
    - Wenn eine Komponente zerstört wird oder nicht mehr sichtbar/aktiv ist, **MÜSSEN** alle Abonnements auf Events oder reaktive Zustände explizit beendet werden (durch Aufruf der zurückgegebenen `UnsubscribeFunction` oder äquivalenter Mechanismen). Dies ist entscheidend zur Vermeidung von Speicherlecks, da sonst Callbacks auf nicht mehr existierende Objekte zeigen könnten.
    - Event-Listener, die direkt an UI-Elementen registriert wurden, müssen entfernt werden.
    - Alle gehaltenen Referenzen auf externe Objekte, die nicht mehr benötigt werden, sollten freigegeben werden, um die Garbage Collection zu unterstützen.

### 7.3. Aspekte der Nebenläufigkeit und UI-Aktualisierungen (Threading-Modell)

MCP-Interaktionen sind inhärent asynchron, da sie oft Netzwerkkommunikation (HTTP/SSE) oder Interprozesskommunikation (stdio) beinhalten. Es ist absolut kritisch, dass diese Operationen den Haupt-UI-Thread nicht blockieren, da dies zum Einfrieren der Benutzeroberfläche führen würde.

- **Asynchrone Operationen:**
    - Alle Methoden in den MCP-Diensten (`MCPConnectionService`, `ToolOrchestrationService` etc.), die I/O-Operationen durchführen, **MÜSSEN** asynchron implementiert sein (z.B. `async/await` in JavaScript/TypeScript/C#, Futures in Rust, Coroutinen in Kotlin).
    - Die `MCPClientInstance` muss ihre Kommunikation mit dem `IMCPTransport` ebenfalls asynchron gestalten.
- **UI-Aktualisierungen aus Hintergrund-Threads/Callbacks:**
    - Die meisten UI-Toolkits erlauben UI-Aktualisierungen nur aus dem Haupt-UI-Thread. Ergebnisse von asynchronen MCP-Operationen (die typischerweise in einem Hintergrund-Thread oder einem Callback-Kontext ankommen) müssen daher sicher an den UI-Thread übergeben werden, bevor UI-Elemente modifiziert werden.
    - **Plattformspezifische Mechanismen:**
        - **GTK (mit Rust und `gtk-rs`):** `glib::MainContext::spawn_local()` oder `glib::MainContext::channel()` können verwendet werden, um Code im Haupt-Loop auszuführen oder Nachrichten an diesen zu senden.18
        - **WPF (C#):** `Dispatcher.Invoke()` oder `Dispatcher.BeginInvoke()`.
        - **Android (Java/Kotlin):** `Activity.runOnUiThread()` oder Handler, die mit dem Main Looper assoziiert sind.
        - **Web (JavaScript):** Da JavaScript single-threaded ist, aber eine Event-Loop hat, werden UI-Aktualisierungen nach `await` oder in Promise-`.then()`-Blöcken typischerweise korrekt von der Event-Loop behandelt. Dennoch ist Vorsicht bei langlaufenden synchronen Berechnungen innerhalb dieser Callbacks geboten.
- **Vermeidung von Race Conditions:**
    - Beim Zugriff auf geteilte Zustände (z.B. Caches in den Diensten oder der Zustand im `MCPGlobalContextManager`) aus verschiedenen asynchronen Kontexten müssen geeignete Synchronisationsmechanismen verwendet werden, falls die Plattform dies erfordert (z.B. Mutexe, Semaphore, atomare Operationen), um Race Conditions und inkonsistente Daten zu vermeiden.
    - Reaktive State-Management-Frameworks bieten oft eingebaute Mechanismen zur sicheren Zustandsaktualisierung.

Eine klare Strategie für Nebenläufigkeit und UI-Thread-Management ist unerlässlich für eine responsive, stabile und korrekte Anwendung.

### 7.4. Logging, Monitoring und Debugging von MCP-Interaktionen

Umfassendes Logging und Möglichkeiten zum Monitoring sind entscheidend für die Entwicklung, Wartung und Fehleranalyse von MCP-Integrationen. Das MCP-Protokoll selbst erwähnt "Logging" als eine der "Additional Utilities".3

- **Logging-Spezifikation:**
    
    - **Was loggen?**
        - **Verbindungsmanagement:** Start/Ende von Verbindungsversuchen, erfolgreiche Verbindungen, Trennungen, Fehler beim Verbindungsaufbau (mit `MCPServerConfig`-Details und Fehlermeldung).
        - **JSON-RPC-Nachrichten:** Alle ausgehenden Requests und eingehenden Responses/Notifications (optional auf einem detaillierten Loglevel, um die Log-Größe zu kontrollieren). Dies ist extrem nützlich für das Debugging von Kommunikationsproblemen. Die `id` der Nachricht sollte immer geloggt werden.
        - **Tool-/Ressourcen-/Prompt-Aufrufe:** Start eines Aufrufs (mit Name, Parametern), Erfolg (mit Zusammenfassung des Ergebnisses), Fehler (mit Fehlerdetails).
        - **Zustimmungsentscheidungen:** Welche Aktion wurde angefragt, welche Entscheidung hat der Benutzer getroffen.
        - **Fehler:** Alle `MCPError`-Instanzen und andere relevante Ausnahmen mit Stack-Trace und Kontextinformationen.
        - **Wichtige Zustandsänderungen:** z.B. Aktualisierung von Server-Capabilities.
    - **Log-Level:** Verwendung von Standard-Log-Levels (DEBUG, INFO, WARN, ERROR) zur Kategorisierung der Nachrichten. JSON-RPC-Nachrichten-Dumps sollten typischerweise auf DEBUG-Level geloggt werden.
    - **Format:** Konsistentes Log-Format mit Zeitstempel, Modulname, Loglevel und Nachricht. Strukturierte Logs (z.B. JSON-Format) können die spätere Analyse erleichtern.
    - **Sensible Daten:** Parameter oder Ergebnisse von MCP-Aufrufen können sensible Daten enthalten. Es muss eine Strategie zur Maskierung oder zum selektiven Logging solcher Daten implementiert werden, um Datenschutzanforderungen zu genügen.
- **Monitoring:**
    
    - Die UI sollte intern (oder über externe Tools, falls angebunden) den Zustand der MCP-Verbindungen und -Operationen überwachen können.
    - Der `MCPGlobalContextManager` kann hierfür Daten bereitstellen (z.B. Anzahl aktiver Verbindungen, Fehlerraten, durchschnittliche Antwortzeiten).
    - Eine dedizierte Debugging-/Statusansicht in der UI (ggf. nur in Entwickler-Builds aktiviert) kann nützlich sein, um diese Informationen live anzuzeigen.
- **Debugging-Techniken:**
    
    - **Nachrichteninspektion:** Die Möglichkeit, die tatsächlich gesendeten und empfangenen JSON-RPC-Nachrichten einzusehen (über Logs oder eine Debug-UI), ist oft der schnellste Weg, um Kommunikationsprobleme zu identifizieren.
    - **Haltepunkte und Tracing:** Standard-Debugging-Tools der Entwicklungsumgebung.
    - **Transport-spezifisches Debugging:**
        - Für `stdio`: Überprüfung der Standard-Input/Output-Ströme des Serverprozesses.
        - Für `HTTP/SSE`: Verwendung von Netzwerk-Sniffern (z.B. Wireshark) oder Browser-Entwicklertools (für SSE-Verbindungen, die über einen Browser-Client getestet werden).
    - **Isolierte Tests:** Testen einzelner `MCPClientInstance`s gegen einen Mock-Server oder einen bekannten, funktionierenden MCP-Server.

Durch die Implementierung dieser Richtlinien wird die Entwicklung und Wartung der MCP-Integration erheblich erleichtert und die Fähigkeit zur schnellen Problemlösung verbessert.

## Anhang

### A.1. Glossar der MCP- und UI-spezifischen Begriffe

- **AI:** Artificial Intelligence (Künstliche Intelligenz).
- **API:** Application Programming Interface (Anwendungsprogrammierschnittstelle).
- **Client (MCP):** Eine Komponente innerhalb des Hosts, die die Verbindung zu einem spezifischen MCP-Server verwaltet.
- **GlobalResourceId, GlobalToolId, GlobalPromptId:** UI-intern verwendete, eindeutige Bezeichner für Ressourcen, Tools oder Prompts über alle verbundenen Server hinweg (typischerweise eine Kombination aus `ServerId` und dem lokalen Namen des Elements).
- **Host (MCP):** Die Anwendung, mit der der Benutzer interagiert und die MCP-Clients beherbergt (in diesem Dokument die UI-Anwendung).
- **HTTP:** Hypertext Transfer Protocol.
- **IDE:** Integrated Development Environment (Integrierte Entwicklungsumgebung).
- **IMCPTransport:** Die in diesem Dokument definierte Schnittstelle für Transport-Handler.
- **JSON:** JavaScript Object Notation.
- **JSON-RPC:** Ein Remote Procedure Call Protokoll, das JSON für den Nachrichtenaustausch verwendet.
- **LLM:** Large Language Model (Großes Sprachmodell).
- **MCP:** Model Context Protocol.
- **MCPClientInstance:** Eine Klasse in der UI-Schicht, die eine einzelne Verbindung zu einem MCP-Server verwaltet.
- **MCPConnectionService:** Ein Dienst in der UI-Schicht, der alle `MCPClientInstance`-Objekte verwaltet.
- **MCPServerConfig:** Eine Datenstruktur, die die Konfigurationsdetails für die Verbindung zu einem MCP-Server enthält.
- **Notification (JSON-RPC):** Eine JSON-RPC-Request-Nachricht ohne `id`-Feld, für die keine Antwort vom Server erwartet wird.
- **Prompt (MCP):** Eine benutzergesteuerte, vordefinierte Vorlage oder parametrisierbare Anfrage zur optimalen Nutzung von Tools oder Ressourcen.
- **Resource (MCP):** Eine anwendungsgesteuerte Datenquelle, auf die ein LLM zugreifen kann.
- **Server (MCP):** Ein externes Programm oder Dienst, das Tools, Ressourcen und Prompts über MCP bereitstellt.
- **ServerCapabilities:** Die Fähigkeiten (Tools, Ressourcen, Prompts), die ein MCP-Server während der Initialisierung meldet.
- **ServerId:** Eine eindeutige Kennung für einen MCP-Server innerhalb der UI-Anwendung.
- **SSE:** Server-Sent Events. Ein Mechanismus, der es einem Server erlaubt, asynchron Daten an einen Client über eine persistente HTTP-Verbindung zu senden.
- **stdio:** Standard Input/Output/Error Streams eines Prozesses.
- **Tool (MCP):** Eine modellgesteuerte Funktion, die ein LLM aufrufen kann, um Aktionen auszuführen.
- **ToolDefinition, ResourceDefinition, PromptDefinition:** Strukturen, die die Metadaten eines Tools, einer Ressource oder eines Prompts beschreiben.
- **UI:** User Interface (Benutzeroberfläche).
- **UX:** User Experience (Benutzererfahrung).
- **WindowIdentifier:** Eine plattformunabhängige Kennung für ein Anwendungsfenster, oft verwendet für XDG Portals.
- **XDG Desktop Portals:** Ein Framework unter Linux, das sandboxed Anwendungen den sicheren Zugriff auf Ressourcen außerhalb der Sandbox über benutzergenehmigte Dialoge ermöglicht.

### A.2. Referenzen auf externe Spezifikationen

- **Model Context Protocol (MCP) Specification:** Die offizielle Spezifikation ist die primäre Referenz. (z.B. 3 und die Website modelcontextprotocol.io)
- **JSON-RPC 2.0 Specification:** [https://www.jsonrpc.org/specification](https://www.jsonrpc.org/specification) (5)
- **XDG Desktop Portal Specification:** [https://flatpak.github.io/xdg-desktop-portal/docs/](https://flatpak.github.io/xdg-desktop-portal/docs/) (15)
- **RFC2119 / RFC8174 (BCP 14):** Für die Interpretation von Schlüsselwörtern wie MUST, SHOULD, MAY in Speifikationen (3).

### A.3. Vollständige JSON-Schemata für Tool/Ressourcen-Parameter (Beispiele)

Dieser Anhang würde vollständige Beispiele für JSON-Schemata enthalten, wie sie in `ToolDefinition.parametersSchema` oder `ResourceDefinition.parametersSchema` (falls Ressourcen parametrisiert sind) vorkommen könnten. Diese dienen als Referenz für die Implementierung der `MCPWidgetFactory` und das Verständnis der Datenstrukturen, die von MCP-Servern erwartet oder geliefert werden.

**Beispiel 1: JSON-Schema für ein einfaches "get_weather" Tool**

JSON

```
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GetWeatherParameters",
  "description": "Parameters for the get_weather tool.",
  "type": "object",
  "properties": {
    "location": {
      "type": "string",
      "description": "The city name or zip code for which to fetch the weather."
    },
    "unit": {
      "type": "string",
      "description": "Temperature unit.",
      "
```

# Eine explizit spezifizierte MCP-Infrastruktur zur Widget-Integration für vereinfachte Linux-Interaktionen

## 1. Einführung

Der Übergang von Betriebssystemen wie Windows oder macOS zu Linux kann für Benutzer eine Herausforderung darstellen, insbesondere hinsichtlich der Interaktion mit Systemfunktionen, die sich oft hinter Kommandozeilen-Tools oder komplexen grafischen Oberflächen verbergen. Eine Möglichkeit, diese Umstellung erheblich zu vereinfachen, ist die Integration intuitiver Desktop-Widgets, die direkten Zugriff auf häufig genutzte Systemaktionen und -informationen bieten. Um eine robuste, standardisierte und erweiterbare Grundlage für solche Widgets zu schaffen, schlägt dieser Bericht die Implementierung einer Infrastruktur vor, die auf dem **Model Context Protocol (MCP)** basiert.

MCP ist ein offenes Protokoll, das ursprünglich von Anthropic entwickelt wurde, um die Integration zwischen Anwendungen für große Sprachmodelle (LLMs) und externen Datenquellen sowie Werkzeugen zu standardisieren.1 Es adressiert das sogenannte „M×N-Integrationsproblem“, bei dem M verschiedene Anwendungen (in unserem Fall Widgets oder die Desktop-Umgebung) mit N verschiedenen Systemfunktionen oder Datenquellen interagieren müssen.5 Anstatt M×N individuelle Integrationen zu erstellen, ermöglicht MCP die Entwicklung von M Clients und N Servern, die über ein standardisiertes Protokoll kommunizieren, wodurch die Komplexität auf M+N reduziert wird.5

Obwohl MCP ursprünglich für LLM-Anwendungen konzipiert wurde, eignet sich seine flexible Client-Server-Architektur und sein Fokus auf standardisierte Schnittstellen hervorragend für die Abstraktion von Linux-Systeminteraktionen. Durch die Definition spezifischer MCP-Server, die als Adapter für zugrunde liegende Linux-Mechanismen (wie D-Bus, Kommandozeilen-Tools und Freedesktop-Standards) fungieren, können Widgets (als MCP-Clients) Systemfunktionen auf eine Weise nutzen, die für Benutzer von Windows und macOS intuitiv und verständlich ist. Dieser Bericht legt eine explizite Architektur und Spezifikation für eine solche MCP-basierte Infrastruktur dar, die darauf abzielt, die Benutzerfreundlichkeit von Linux-Desktops für Umsteiger drastisch zu verbessern.

## 2. Grundlagen des Model Context Protocol (MCP)

Um die vorgeschlagene Infrastruktur zu verstehen, ist ein grundlegendes Verständnis der Kernkomponenten und Konzepte von MCP erforderlich. MCP definiert eine standardisierte Methode für die Kommunikation zwischen Anwendungen (Hosts), die Kontext benötigen, und Diensten (Servern), die diesen Kontext oder zugehörige Funktionen bereitstellen.1

### 2.1 Kernarchitektur: Host, Client und Server

MCP basiert auf einer Client-Server-Architektur mit drei Hauptkomponenten 3:

1. **Host:** Die Anwendung, die die Interaktion initiiert und den Kontext oder die Funktionalität benötigt. Im Kontext dieses Berichts ist der Host typischerweise die Desktop-Umgebung oder eine übergeordnete Widget-Verwaltungskomponente, die die Widgets selbst enthält und deren Kommunikation koordiniert.
2. **Client:** Eine Komponente, die innerhalb des Hosts läuft und eine dedizierte 1:1-Verbindung zu einem bestimmten MCP-Server aufbaut und verwaltet.3 Das Widget selbst oder eine vom Host bereitgestellte Abstraktionsschicht fungiert als Client.
3. **Server:** Ein (oft leichtgewichtiger) Prozess, der spezifische Fähigkeiten (Daten, Aktionen, Vorlagen) über das MCP-Protokoll bereitstellt.1 Im vorgeschlagenen Szenario kapseln diese Server spezifische Linux-Systemfunktionen (z. B. Netzwerkverwaltung, Energieoptionen, Dateisuche).

Diese Architektur ermöglicht eine klare Trennung von Belangen: Widgets (Clients) müssen nur das standardisierte MCP-Protokoll verstehen, während die Server die Komplexität der Interaktion mit den spezifischen Linux-Subsystemen kapseln.1

### 2.2 MCP-Primitive: Bausteine der Interaktion

Die Kommunikation und die Fähigkeiten innerhalb von MCP werden durch sogenannte _Primitive_ definiert. Diese legen fest, welche Arten von Interaktionen zwischen Client und Server möglich sind.5

**Server-seitige Primitive** (vom Server dem Client angeboten):

- **Tools:** Repräsentieren ausführbare Funktionen oder Aktionen, die der Client (im Auftrag des Benutzers oder einer KI) auf dem Server aufrufen kann.3 Beispiele im Desktop-Kontext wären das Umschalten von WLAN, das Ändern der Lautstärke oder das Herunterfahren des Systems. Tools können Parameter entgegennehmen und Ergebnisse zurückgeben. Sie sind typischerweise _modellgesteuert_ (im ursprünglichen MCP-Kontext) oder _widget-gesteuert_ (in unserem Kontext), da die Aktion vom Client initiiert wird.
- **Resources:** Stellen Daten oder Inhalte dar, die der Client vom Server lesen kann, um sie anzuzeigen oder als Kontext zu verwenden.3 Beispiele wären der aktuelle Batteriestatus, der Name des verbundenen WLAN-Netzwerks oder eine Liste kürzlich verwendeter Dateien. Ressourcen sind in der Regel schreibgeschützt aus Sicht des Clients und _anwendungsgesteuert_, d. h., die Host-Anwendung entscheidet, wann und wie sie verwendet werden.31
- **Prompts:** Sind vordefinierte Vorlagen oder Arbeitsabläufe, die vom Server bereitgestellt werden, um komplexe Interaktionen zu strukturieren oder zu vereinfachen.3 Im Widget-Kontext könnten sie weniger relevant sein, aber potenziell für geführte Konfigurationsdialoge genutzt werden, die von einem Widget ausgelöst werden. Sie sind typischerweise _benutzergesteuert_.31

**Client-seitige Primitive** (vom Client dem Server angeboten):

- **Roots:** Repräsentieren Einstiegspunkte oder definierte Bereiche im Dateisystem oder der Umgebung des Hosts, auf die der Server zugreifen darf, wenn die Berechtigung erteilt wird.5 Dies ist relevant für MCP-Server, die mit lokalen Dateien interagieren müssen (z. B. ein Dateisuche-Server).
- **Sampling:** Ermöglicht es dem Server, eine Anfrage zur Generierung von Inhalten (z. B. Text) durch ein LLM auf der Client-Seite zu stellen.3 Für die primäre Widget-Integration ist dies weniger relevant, könnte aber für zukünftige, KI-gestützte Widgets von Bedeutung sein. Anthropic betont die Notwendigkeit einer menschlichen Genehmigung für Sampling-Anfragen.5

Für die hier beschriebene Desktop-Widget-Infrastruktur sind **Tools** und **Resources** die wichtigsten serverseitigen Primitive, während **Roots** für dateibezogene Server relevant sind.

### 2.3 Ökosystem und Standardisierung

MCP wird als offener Standard entwickelt, unterstützt durch SDKs in verschiedenen Sprachen (Python, TypeScript, Java, C#, Kotlin, Rust, Swift) und eine wachsende Community.1 Es gibt bereits zahlreiche Open-Source-MCP-Server für gängige Dienste wie Google Drive, Slack, GitHub, Datenbanken und Betriebssysteminteraktionen.1 Frühe Anwender wie Block und Apollo haben MCP bereits in ihre Systeme integriert.1 Diese Standardisierung und das wachsende Ökosystem sind entscheidend für die Schaffung einer interoperablen und zukunftssicheren Infrastruktur für Desktop-Widgets.

## 3. Kernarchitektur und Kommunikation der MCP-Infrastruktur

Aufbauend auf den MCP-Grundlagen wird nun die spezifische Architektur für die Integration von Desktop-Widgets in Linux-Systemen detailliert beschrieben. Diese Architektur legt fest, wie Widgets (als Clients) über das MCP-Protokoll mit spezialisierten Servern kommunizieren, die Systemfunktionen kapseln.

### 3.1 Detaillierte Host-Client-Server-Interaktionen am Beispiel eines Widgets

Betrachten wir einen typischen Interaktionsfluss, ausgelöst durch ein Widget, z. B. ein "WLAN umschalten"-Widget:

1. **Benutzeraktion:** Der Benutzer klickt auf das Widget, um WLAN zu aktivieren.
2. **Host-Übersetzung:** Der Host (die Desktop-Umgebung oder Widget-Verwaltung) empfängt das Klick-Ereignis und identifiziert den zuständigen MCP-Server (z. B. den Netzwerk-MCP-Server). Der Host weist den entsprechenden MCP-Client an, eine Aktion auszuführen.
3. **MCP-Anfrage (Client -> Server):** Der Client formuliert eine MCP `tools/call`-Anfrage. Diese wird als JSON-RPC 2.0-Nachricht über den gewählten Transportkanal gesendet.
    - Beispiel JSON-RPC-Anfrage (vereinfacht):
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "id": 123,
          "method": "tools/call",
          "params": {
            "tool_name": "network.setWifiEnabled",
            "parameters": { "enabled": true }
          }
        }
        ```
        
4. **Server-Verarbeitung:** Der Netzwerk-MCP-Server empfängt die JSON-RPC-Nachricht über den Transportkanal (z. B. STDIO). Er parst die Anfrage, validiert die Parameter und identifiziert die angeforderte Aktion (`network.setWifiEnabled`).
5. **Systeminteraktion (Server -> D-Bus):** Der Server übersetzt die MCP-Anfrage in einen entsprechenden Aufruf an das zugrunde liegende Linux-System, in diesem Fall wahrscheinlich über D-Bus an den NetworkManager-Dienst.63 Er könnte beispielsweise eine Methode wie `ActivateConnection` oder eine gerätespezifische Methode aufrufen, um das WLAN-Gerät zu aktivieren.
6. **Systemantwort (D-Bus -> Server):** Der NetworkManager führt die Aktion aus und sendet eine Antwort (Erfolg oder Fehler) über D-Bus zurück an den MCP-Server.
7. **MCP-Antwort (Server -> Client):** Der MCP-Server empfängt die D-Bus-Antwort, formatiert sie als JSON-RPC 2.0-Antwortnachricht und sendet sie über den Transportkanal zurück an den Client.
    - Beispiel JSON-RPC-Antwort (Erfolg, vereinfacht):
        
        JSON
        
        ```
        {
          "jsonrpc": "2.0",
          "id": 123,
          "result": { "output": { "success": true } }
        }
        ```
        
8. **Client-Verarbeitung:** Der Client empfängt die Antwort und leitet das Ergebnis an den Host weiter.
9. **Host-Aktualisierung:** Der Host aktualisiert den Zustand des Widgets, um den neuen WLAN-Status widerzuspiegeln (z. B. Änderung des Icons, Anzeige des verbundenen Netzwerks).

Dieser Ablauf demonstriert, wie MCP als standardisierte Zwischenschicht fungiert, die die Komplexität der direkten Systeminteraktion vor dem Widget verbirgt.

### 3.2 JSON-RPC 2.0 Nachrichtenstruktur

Die gesamte Kommunikation innerhalb der MCP-Infrastruktur basiert auf dem JSON-RPC 2.0-Protokoll.5 Dies gewährleistet eine klare, strukturierte und sprachunabhängige Nachrichtenübermittlung.

Die grundlegenden Nachrichtentypen sind 70:

- **Request:** Wird gesendet, um eine Methode auf der Gegenseite aufzurufen (z. B. `tools/call`, `resources/read`). Enthält `jsonrpc`, `id`, `method` und `params`.
- **Response:** Die Antwort auf eine Request-Nachricht. Enthält `jsonrpc`, die `id` der ursprünglichen Anfrage und entweder ein `result`-Objekt (bei Erfolg) oder ein `error`-Objekt.
- **Notification:** Eine einseitige Nachricht, die keine Antwort erwartet (z. B. `initialized`, `notifications/resources/updated`). Enthält `jsonrpc` und `method`, optional `params`, aber keine `id`.

Der Verbindungsaufbau beginnt mit einem **Handshake**, bei dem Client und Server Informationen über ihre unterstützten Protokollversionen und Fähigkeiten austauschen 6:

1. **Client -> Server:** `initialize` (Request) mit Client-Infos und -Fähigkeiten.
2. **Server -> Client:** `initialize` (Response) mit Server-Infos und -Fähigkeiten.
3. **Client -> Server:** `initialized` (Notification) zur Bestätigung des erfolgreichen Handshakes.

Danach kann der reguläre Austausch von Nachrichten beginnen. Die genauen JSON-Strukturen für spezifische MCP-Methoden wie `tools/list`, `tools/call`, `resources/list`, `resources/read` (`getResourceData` im Schema) usw. sind im offiziellen MCP JSON-Schema definiert.70

### 3.3 Überlegungen zur Transportschicht: STDIO als primäre Wahl

MCP unterstützt nativ zwei Haupttransportmechanismen für die JSON-RPC-Nachrichten 3:

1. **Standard Input/Output (STDIO):** Die Kommunikation erfolgt über die Standard-Eingabe- und Ausgabe-Streams zwischen dem Host-Prozess (der den Client enthält) und dem Server-Prozess. Der Host startet und verwaltet den Server-Prozess direkt.
2. **HTTP mit Server-Sent Events (SSE):** Die Kommunikation erfolgt über das Netzwerk. Der Client sendet Anfragen per HTTP POST, während der Server Nachrichten (insbesondere Notifications) über eine persistente SSE-Verbindung an den Client sendet.

Für die vorgeschlagene lokale Desktop-Widget-Integration ist **STDIO die empfohlene primäre Transportschicht**.3 Die Gründe hierfür sind:

- **Effizienz:** Direkte Prozesskommunikation auf derselben Maschine ist in der Regel performanter als Netzwerkkommunikation.
- **Einfachheit:** Es entfallen die Komplexitäten der Netzwerkkonfiguration, Port-Verwaltung und komplexer Authentifizierungsschemata, die bei SSE auftreten können. Der Host kann die Server-Prozesse einfach starten und über Pipes kommunizieren.
- **Sicherheit:** Die Kommunikation bleibt lokal auf der Maschine beschränkt, was das Risiko von Netzwerkangriffen wie DNS-Rebinding (ein spezifisches Risiko bei SSE 69) eliminiert. Die Sicherheit konzentriert sich auf die Kontrolle der gestarteten Server-Prozesse durch den Host.

Der Host wäre dafür verantwortlich, die benötigten MCP-Server-Prozesse (z. B. beim Systemstart oder bei Bedarf) zu starten und deren Lebenszyklus zu verwalten. Die Kommunikation über `stdin` und `stdout` der Server-Prozesse ist ein etabliertes Muster für lokale Interprozesskommunikation.

**HTTP+SSE** bleibt eine Option für zukünftige Erweiterungen, beispielsweise wenn Widgets Fernsteuerungsfunktionen ermöglichen oder auf Cloud-Dienste zugreifen sollen. Die Architektur sollte idealerweise so gestaltet sein, dass die Kernlogik der MCP-Server von der Transportschicht getrennt ist, um einen späteren Wechsel oder eine parallele Unterstützung von SSE zu erleichtern. Die Implementierung von SSE würde jedoch zusätzliche Sicherheitsüberlegungen erfordern, insbesondere robuste Authentifizierungs- und Autorisierungsmechanismen.28

### 3.4 Verwaltung des Verbindungslebenszyklus

Der MCP Host spielt eine zentrale Rolle bei der Verwaltung des Lebenszyklus jeder Client-Server-Verbindung.6 Dies umfasst die drei Hauptphasen:

1. **Initialisierung:** Der Host startet den MCP-Server-Prozess (bei STDIO) und initiiert über den Client den Handshake (`initialize`/`initialized`) zur Aushandlung von Protokollversionen und Fähigkeiten.
2. **Nachrichtenaustausch:** Der Host leitet Benutzeraktionen aus Widgets an den Client weiter, der daraus Requests an den Server generiert. Eingehende Responses und Notifications vom Server werden vom Client empfangen und an den Host zur Aktualisierung der Widgets oder zur weiteren Verarbeitung weitergeleitet.
3. **Terminierung:** Der Host ist dafür verantwortlich, die Verbindung sauber zu beenden, wenn das Widget geschlossen wird oder der Server nicht mehr benötigt wird. Dies kann durch ein explizites `shutdown`-Signal oder durch Beenden des Server-Prozesses geschehen. Fehlerbedingungen oder unerwartete Trennungen der Transportverbindung (z. B. Absturz des Server-Prozesses) müssen ebenfalls vom Host gehandhabt werden.

Eine robuste Verwaltung des Lebenszyklus durch den Host ist entscheidend für die Stabilität und Ressourceneffizienz der gesamten Infrastruktur.

## 4. Brückenschlag zwischen MCP und Linux-Desktop-Mechanismen

Das Herzstück der vorgeschlagenen Infrastruktur sind die MCP-Server, die als Adapter zwischen der standardisierten MCP-Welt und den vielfältigen Mechanismen des Linux-Desktops fungieren. Sie empfangen generische MCP-Anfragen und übersetzen diese in spezifische Aufrufe an D-Bus, Kommandozeilen-Tools oder andere relevante Schnittstellen.

### 4.1 Strategie für die Interaktion von MCP-Servern

Die Kernstrategie besteht darin, für jede logische Gruppe von Systemfunktionen (Netzwerk, Energie, Einstellungen, Dateien usw.) einen dedizierten MCP-Server zu erstellen. Jeder Server implementiert die MCP-Spezifikation und kapselt die Logik für die Interaktion mit dem entsprechenden Linux-Subsystem. Widgets kommunizieren ausschließlich über MCP mit diesen Servern und bleiben somit von den Implementierungsdetails der Linux-Seite isoliert.

### 4.2 Schnittstelle zu D-Bus

D-Bus ist der _de facto_ Standard für die Interprozesskommunikation (IPC) auf modernen Linux-Desktops und bietet Zugriff auf eine Vielzahl von Systemdiensten.72 MCP-Server können D-Bus nutzen, um Systemzustände abzufragen und Aktionen auszulösen.

Ein typischer MCP-Server (z. B. in Python geschrieben, unter Verwendung von Bibliotheken wie `dasbus` 73 oder `pydbus` 74) würde folgende Schritte ausführen:

1. **Verbindung zum Bus:** Aufbau einer Verbindung zum entsprechenden Bus – dem **System Bus** für systemweite Dienste (wie NetworkManager, logind, UPower) oder dem **Session Bus** für benutzerspezifische Dienste (wie Benachrichtigungen, anwendungsspezifische Schnittstellen).72
2. **Proxy-Objekt erhalten:** Anfordern eines Proxy-Objekts für einen bestimmten Dienst (über dessen wohlbekannten Busnamen, z. B. `org.freedesktop.NetworkManager`) und Objektpfad (z. B. `/org/freedesktop/NetworkManager/Devices/0`).65
3. **Methodenaufruf:** Aufrufen von Methoden auf der D-Bus-Schnittstelle des Proxy-Objekts basierend auf der empfangenen MCP `tools/call`-Anfrage. Zum Beispiel würde eine MCP-Anfrage `network.disconnectWifi` zu einem D-Bus-Aufruf wie `proxy.Disconnect()` auf der `org.freedesktop.NetworkManager.Device`-Schnittstelle führen.65
4. **Signal-Überwachung (optional):** Registrieren für D-Bus-Signale (z. B. `StateChanged` von NetworkManager 64 oder `PrepareForShutdown` von logind 81), um auf Systemänderungen zu reagieren. Diese Signale könnten dann als MCP-Notifications an den Client weitergeleitet werden, um Widgets proaktiv zu aktualisieren.72

Zahlreiche Beispiele und Tutorials für die D-Bus-Interaktion mit Python sind verfügbar und können als Grundlage für die Serverentwicklung dienen.63

### 4.3 Nutzung von Kommandozeilen-Tools (CLIs)

Für Aufgaben, die nicht direkt oder einfach über D-Bus zugänglich sind, können MCP-Server als Wrapper für Kommandozeilen-Tools fungieren.

- **Dateisuche:** Ein MCP-Server könnte das `plocate`-Kommando nutzen, um schnelle Dateisuchen durchzuführen.59 Ein MCP-Tool `filesystem.searchFiles` würde die Suchanfrage des Benutzers als Parameter entgegennehmen, `plocate <query>` ausführen und die formatierte Ausgabe als Ergebnis zurückgeben. Bestehende MCP-Server wie `Lilith-Shell` oder `Terminal-Control` demonstrieren bereits die Ausführung von Shell-Befehlen.46
- **Systemeinstellungen:** Das `gsettings`-Tool ermöglicht das Lesen und Schreiben von Konfigurationseinstellungen, die von vielen GNOME-basierten Anwendungen und der Desktop-Umgebung selbst verwendet werden.91 Ein MCP-Tool `settings.setGSetting` könnte Schema, Schlüssel und Wert als Parameter akzeptieren und den entsprechenden `gsettings set <schema> <key> <value>`-Befehl ausführen.

Beim Kapseln von CLIs ist äußerste Vorsicht geboten. MCP-Server **müssen** alle Eingaben, die zur Konstruktion von Kommandozeilenbefehlen verwendet werden, sorgfältig validieren und bereinigen (sanitizing), um Command-Injection-Schwachstellen zu verhindern.27

### 4.4 Schnittstelle zu Freedesktop-Standards

MCP-Server können auch mit etablierten Freedesktop.org-Standards interagieren:

- **Desktop-Einträge (`.desktop`-Dateien):** Diese Dateien beschreiben installierte Anwendungen und deren Startverhalten gemäß der Desktop Entry Specification.97 Ein MCP-Server könnte diese Dateien parsen (unter Verwendung von Bibliotheken wie `freedesktop-file-parser` 98 oder `freedesktop-desktop-entry` 100 für Rust, oder entsprechenden Bibliotheken für andere Sprachen 101), um eine Liste installierter Anwendungen als MCP `Resource` bereitzustellen oder das Starten einer Anwendung über ein MCP `Tool` zu ermöglichen (z. B. durch Ausführen von `gtk-launch <app.desktop>` oder über D-Bus-Aktivierung).
- **Benachrichtigungen:** Das Senden von Desktop-Benachrichtigungen erfolgt standardmäßig über die `org.freedesktop.Notifications`-D-Bus-Schnittstelle.75 Ein MCP-Server könnte ein einfaches `notifications.send`-Tool bereitstellen, das Titel, Text und optional ein Icon entgegennimmt und an den D-Bus-Dienst weiterleitet.

### 4.5 Tabelle: Zuordnung von Desktop-Aufgaben zu Linux-Mechanismen

Um die Implementierung der MCP-Server zu erleichtern, bietet die folgende Tabelle eine Zuordnung gängiger Desktop-Aufgaben, die für Umsteiger relevant sind, zu den primären zugrunde liegenden Linux-Mechanismen und spezifischen Schnittstellen oder Befehlen. Diese Zuordnung dient als Blaupause für die Entwicklung der Server-Logik.

|   |   |   |
|---|---|---|
|**Gängige Aufgabe für Umsteiger**|**Primärer Linux-Mechanismus**|**Spezifische Schnittstelle / Befehl / Datei (Beispiele)**|
|WLAN ein-/ausschalten|D-Bus: NetworkManager|`org.freedesktop.NetworkManager.Device` Methoden (z.B. `Disconnect`, `ActivateConnection`) 64|
|Mit WLAN verbinden|D-Bus: NetworkManager|`org.freedesktop.NetworkManager.ActivateConnection` 65|
|Lautstärke ändern|D-Bus: PulseAudio/PipeWire/DE|DE-spezifisch (z.B. `org.gnome.settings-daemon.plugins.media-keys.volume-up`) oder Audio-Server API|
|Display-Helligkeit ändern|D-Bus: UPower/logind/DE|DE-spezifisch oder `org.freedesktop.login1.Manager` (Backlight API)|
|Dunkelmodus umschalten|`gsettings` / DE-spezifisch D-Bus|`gsettings set org.gnome.desktop.interface color-scheme 'prefer-dark'` 91|
|Hintergrundbild ändern|`gsettings` / DE-spezifisch D-Bus|`gsettings set org.gnome.desktop.background picture-uri 'file:///...'` 91|
|Datei suchen|CLI: `plocate`|`plocate <pattern>` 86|
|Anwendung starten|`.desktop` / D-Bus Activation|`gtk-launch <app.desktop>` oder `org.freedesktop.Application.Activate`|
|Installierte Apps auflisten|`.desktop` Parsing|Parsen von `.desktop`-Dateien in Standardverzeichnissen 97|
|Batteriestatus prüfen|D-Bus: UPower / `sysfs`|`org.freedesktop.UPower.Device.Percentage`, `...State`|
|Bildschirm sperren|D-Bus: Session Lock / DE|DE-spezifisch (z.B. `org.gnome.ScreenSaver.Lock`) oder `loginctl lock-session`|
|Herunterfahren / Neustarten|D-Bus: logind|`org.freedesktop.login1.Manager.PowerOff`, `...Reboot` 81|
|Ruhezustand / Standby|D-Bus: logind|`org.freedesktop.login1.Manager.Suspend`, `...Hibernate` 81|

Diese Tabelle verdeutlicht, dass für die meisten gängigen Desktop-Interaktionen etablierte Linux-Mechanismen existieren, die von den MCP-Servern gekapselt werden können. Die Herausforderung für Entwickler besteht darin, die spezifischen D-Bus-Schnittstellen oder Kommandozeilenbefehle zu identifizieren und korrekt in den MCP-Servern zu implementieren. Die Tabelle dient hierbei als wertvolle Referenz und stellt sicher, dass die richtigen APIs angesprochen werden, was die Entwicklungszeit verkürzt und die Korrektheit der Implementierung fördert.

## 5. Gestaltung von MCP-Servern für die Bedürfnisse von Windows/Mac-Umsteigern

Ein zentrales Ziel dieser Infrastruktur ist es, die Interaktion für Benutzer zu vereinfachen, die von Windows oder macOS kommen. Dies erfordert ein durchdachtes Design der MCP-Server und der von ihnen bereitgestellten Schnittstellen (Tools und Resources).

### 5.1 Definition von MCP-Primitiven für Desktop-Aktionen

Die MCP-Primitive müssen so eingesetzt werden, dass sie den Interaktionen in Desktop-Widgets entsprechen 3:

- **Tools:** Werden primär für **Aktionen** verwendet, die durch Widget-Interaktionen wie Klicks, Umschalter oder Schieberegler ausgelöst werden.
    - _Beispiele:_ `network.setWifiEnabled(enabled: boolean)`, `audio.setVolume(level: integer)`, `power.shutdown()`, `files.moveToTrash(path: string)`.
    - Die Parameter für Tools sollten einfach, typisiert und intuitiv verständlich sein. Komplexe Konfigurationsobjekte sollten vermieden werden.
- **Resources:** Dienen dazu, System**zustände** oder **Daten** für die Anzeige in Widgets bereitzustellen.
    - _Beispiele:_ `network.getWifiState() -> {enabled: boolean, ssid: string, strength: integer}`, `power.getBatteryStatus() -> {level: integer, charging: boolean}`, `filesystem.listFiles(directory: string) -> list<object>`.
    - Ressourcen sollten aus Sicht des Clients schreibgeschützt sein.31 Änderungen erfolgen über Tools. Sie können optional Abonnementmechanismen unterstützen, um den Client über Änderungen zu informieren (`notifications/resources/updated`).51
- **Prompts:** Spielen für einfache Status- und Aktions-Widgets eine untergeordnete Rolle. Sie könnten jedoch verwendet werden, um komplexere, geführte Abläufe zu initiieren, die über das Widget gestartet werden (z. B. das Einrichten einer neuen VPN-Verbindung).
- **Roots:** Definieren Dateisystembereiche, auf die bestimmte Server zugreifen dürfen (z. B. der Home-Ordner für einen Dateisuche-Server).5 Der Host verwaltet diese und holt die Zustimmung des Benutzers ein.
- **Sampling:** Ist für die Kernfunktionalität der Widgets zunächst nicht erforderlich, bietet aber Potenzial für zukünftige KI-gestützte Widget-Funktionen.6

### 5.2 Abstraktion Linux-spezifischer Konzepte

Ein entscheidender Aspekt ist die **Abstraktion**. Die MCP-Schnittstellen (Tool-/Resource-Namen, Parameter, Rückgabewerte) dürfen keine Linux-spezifischen Details wie D-Bus-Pfade (`/org/freedesktop/...`), interne Servicenamen (`org.gnome.SettingsDaemon.Plugins.Color`) oder komplexe `gsettings`-Schemas offenlegen.

Die Benennung sollte klar, konsistent und plattformagnostisch sein, orientiert an der Terminologie, die Windows/Mac-Benutzer erwarten würden.

- **Statt:** `org.freedesktop.NetworkManager.Device.Disconnect`
    
- **Verwende:** MCP Tool `network.disconnectWifi()`
    
- **Statt:** `gsettings get org.gnome.desktop.interface color-scheme`
    
- **Verwende:** MCP Resource `settings.getColorScheme() -> string` (z.B. 'light' oder 'dark')
    

Diese Abstraktionsebene ist es, die MCP für die Vereinfachung der Linux-Benutzererfahrung so wertvoll macht. Sie entkoppelt die Benutzeroberfläche (Widgets) vollständig von der darunterliegenden Systemimplementierung.

### 5.3 Tabelle: MCP-Primitive im Kontext der Desktop-Widget-Integration

Die folgende Tabelle verdeutlicht die spezifische Rolle jedes MCP-Primitivs im Kontext der Desktop-Widget-Integration und liefert konkrete Beispiele. Dies hilft Architekten und Entwicklern, die Primitive konsistent und gemäß ihrer vorgesehenen Funktion in diesem spezifischen Anwendungsfall einzusetzen.

|   |   |   |   |
|---|---|---|---|
|**MCP Primitive**|**Definition (gemäß MCP-Spezifikation)**|**Rolle in der Desktop-Widget-Integration**|**Beispielhafte Widget-Interaktion**|
|**Tool**|Ausführbare Funktion, die vom Client aufgerufen wird, um eine Aktion auszuführen oder Informationen abzurufen 5|**Aktion auslösen:** Wird verwendet, wenn ein Widget eine Zustandsänderung im System bewirken soll (z. B. Umschalten, Wert setzen, Befehl ausführen).|Klick auf "Herunterfahren"-Button löst `power.shutdown()` Tool aus. Verschieben eines Lautstärkereglers löst `audio.setVolume(level)` Tool aus.|
|**Resource**|Strukturierte Daten oder Inhalte, die vom Server bereitgestellt und vom Client gelesen werden können, um Kontext bereitzustellen 5|**Zustand anzeigen:** Wird verwendet, um aktuelle Systeminformationen oder Daten abzurufen, die in einem Widget angezeigt werden sollen (z. B. Status, Wert, Liste).|Ein Batterie-Widget liest periodisch die `power.getBatteryStatus()` Resource, um die Anzeige zu aktualisieren. Ein Netzwerk-Widget liest `network.getWifiState()` Resource beim Start.|
|**Prompt**|Vorbereitete Anweisung oder Vorlage, die vom Server bereitgestellt wird, um Interaktionen zu leiten 5|**Geführter Arbeitsablauf (seltener):** Kann verwendet werden, um komplexere Konfigurations- oder Einrichtungsaufgaben zu initiieren, die über die Host-UI laufen.|Klick auf "VPN konfigurieren" in einem Netzwerk-Widget könnte einen `network.configureVPN` Prompt auslösen, der einen Dialog im Host startet.|
|**Root**|Einstiegspunkt in das Dateisystem/die Umgebung des Hosts, auf den der Server zugreifen darf 5|**Zugriffsbereich definieren:** Legt fest, auf welche Teile des Dateisystems ein Server (z. B. Dateisuche) zugreifen darf, nach Zustimmung des Benutzers durch den Host.|Ein Dateisuche-Widget verwendet einen Server, der nur auf die per Root definierten Ordner (z. B. `/home/user/Documents`) zugreifen darf.|
|**Sampling**|Mechanismus, der es dem Server ermöglicht, eine LLM-Vervollständigung vom Client anzufordern 5|**Zukünftige KI-Funktionen (optional):** Nicht für grundlegende Widgets erforderlich, könnte aber für erweiterte, KI-gestützte Widget-Aktionen genutzt werden.|Ein "Organisiere Downloads"-Widget könnte einen Server nutzen, der via Sampling den Host-LLM bittet, eine Ordnungsstrategie vorzuschlagen.|

Diese klare Zuordnung stellt sicher, dass die MCP-Primitive im Sinne der Vereinfachung und Abstraktion für Windows/Mac-Umsteiger korrekt eingesetzt werden.

## 6. Beispielhafte MCP-Server-Implementierungen

Um die vorgeschlagene Architektur zu konkretisieren, werden im Folgenden einige Beispiele für MCP-Server skizziert, die typische Bedürfnisse von Umsteigern adressieren. Für jeden Server werden Zweck, beispielhafte MCP-Schnittstellen (Tools/Resources) und die wahrscheinlich genutzten Linux-Mechanismen beschrieben.

### 6.1 Vereinfachter Dateiverwaltungs-Server

- **Zweck:** Ermöglicht schnelles Finden und grundlegende Operationen mit Dateien, ohne dass Benutzer sich mit komplexen Dateimanagern oder der Kommandozeile auseinandersetzen müssen. Adressiert die oft als umständlich empfundene Dateisuche unter Linux.
- **MCP-Schnittstellen:**
    - **Tools:**
        - `files.search(query: string) -> list<object>`: Führt eine schnelle Suche im indizierten Dateisystem durch.
        - `files.open(path: string) -> boolean`: Öffnet die angegebene Datei mit der Standardanwendung.
        - `files.moveToTrash(path: string) -> boolean`: Verschiebt die Datei sicher in den Papierkorb.
    - **Resources:**
        - `files.list(directory: string) -> list<object>`: Listet den Inhalt eines Verzeichnisses auf (unter Berücksichtigung der per Roots definierten Berechtigungen).
        - `files.getRecentFiles() -> list<object>`: Ruft eine Liste der zuletzt verwendeten Dateien ab (z. B. über Desktop-Suchindizes oder Lesezeichen).
- **Zugrunde liegende Mechanismen:**
    - Suche: `plocate`-Kommandozeilentool für schnelle, indizierte Suche.59
    - Öffnen: D-Bus-Aufrufe (`org.freedesktop.FileManager1.ShowItems` oder `xdg-open` CLI).
    - Papierkorb: Implementierung gemäß Freedesktop.org Trash Specification (oft über GLib/GIO-Bibliotheken).
    - Dateilisting/Recent: Standard-Dateisystem-APIs, Desktop-Suchdienste (z. B. Tracker).
- **Implementierung:** Python mit `subprocess` für `plocate` und Dateisystem-APIs, ggf. `pydbus`/`dasbus` für Öffnen/Papierkorb. Zugriffsbereiche sollten über MCP Roots gesteuert werden.29 Bestehende Filesystem-MCP-Server 59 können als Vorlage dienen.

### 6.2 Vereinheitlichter Systemeinstellungs-Server

- **Zweck:** Bietet einfache Umschalter und Schieberegler für häufig geänderte Einstellungen (z. B. Dunkelmodus, Helligkeit, Lautstärke, Maus-/Touchpad-Geschwindigkeit), die oft in verschachtelten Menüs versteckt sind.
- **MCP-Schnittstellen:**
    - **Tools:**
        - `settings.setDarkMode(enabled: boolean) -> boolean`
        - `settings.setBrightness(level: integer) -> boolean` (Level 0-100)
        - `settings.setVolume(level: integer) -> boolean` (Level 0-100)
        - `settings.setMouseSpeed(level: float) -> boolean` (Skala definieren, z. B. 0.0-1.0)
    - **Resources:**
        - `settings.getDarkMode() -> boolean`
        - `settings.getBrightness() -> integer`
        - `settings.getVolume() -> integer`
        - `settings.getMouseSpeed() -> float`
- **Zugrunde liegende Mechanismen:**
    - Primär: `gsettings`-Kommandozeilentool zum Lesen/Schreiben von Schemas wie `org.gnome.desktop.interface`, `org.gnome.desktop.peripherals` etc..91
    - Alternativ/Ergänzend: Direkte D-Bus-Aufrufe an spezifische Dienste der Desktop-Umgebung (z. B. GNOME Settings Daemon, KDE Powerdevil/KWin). Dies kann notwendig sein für Einstellungen, die nicht über GSettings verfügbar sind oder sofortige UI-Updates erfordern.
- **Implementierung:** Python mit `subprocess` für `gsettings` und/oder `pydbus`/`dasbus` für D-Bus. **Herausforderung:** Die spezifischen GSettings-Schemas oder D-Bus-Schnittstellen können sich zwischen Desktop-Umgebungen (GNOME, KDE, XFCE etc.) unterscheiden. Der Server muss entweder DE-spezifische Logik enthalten oder sich auf möglichst universelle Mechanismen konzentrieren.

### 6.3 Anwendungsstarter/-manager-Server

- **Zweck:** Bietet eine einfache Möglichkeit, installierte Anwendungen zu finden und zu starten, ähnlich dem Startmenü oder Launchpad.
- **MCP-Schnittstellen:**
    - **Tools:**
        - `apps.launch(appId: string) -> boolean`: Startet die Anwendung mit der gegebenen ID (typischerweise der Name der `.desktop`-Datei ohne Endung).
    - **Resources:**
        - `apps.listInstalled() -> list<{id: string, name: string, icon: string}>`: Gibt eine Liste aller gefundenen Anwendungen mit ID, Namen und Icon-Namen zurück.
- **Zugrunde liegende Mechanismen:**
    - Auflisten: Parsen von `.desktop`-Dateien in Standardverzeichnissen (`/usr/share/applications`, `~/.local/share/applications`) gemäß Desktop Entry Specification.97
    - Starten: Ausführen von `gtk-launch <appId>` oder Verwenden von D-Bus-Aktivierungsmechanismen (z. B. `org.freedesktop.Application.Activate`).
- **Implementierung:** Python mit einer Bibliothek zum Parsen von `.desktop`-Dateien und `subprocess` oder D-Bus-Bindings zum Starten.

### 6.4 Netzwerkkonfigurations-Server

- **Zweck:** Vereinfacht die Verwaltung von WLAN-Verbindungen und das Umschalten von VPNs, Aufgaben, die für Umsteiger oft verwirrend sind.
- **MCP-Schnittstellen:**
    - **Tools:**
        - `network.setWifiEnabled(enabled: boolean) -> boolean`
        - `network.connectWifi(ssid: string, password?: string) -> boolean`: Verbindet mit einem bekannten oder neuen Netzwerk.
        - `network.disconnectWifi() -> boolean`
        - `network.setVpnEnabled(vpnId: string, enabled: boolean) -> boolean`: Aktiviert/Deaktiviert eine konfigurierte VPN-Verbindung.
    - **Resources:**
        - `network.getWifiState() -> {enabled: boolean, connected: boolean, ssid?: string, strength?: integer}`: Gibt den aktuellen WLAN-Status zurück.
        - `network.listAvailableWifi() -> list<{ssid: string, strength: integer, security: string}>`: Listet sichtbare WLAN-Netzwerke auf.
        - `network.listVpns() -> list<{id: string, name: string, connected: boolean}>`: Listet konfigurierte VPN-Verbindungen auf.
- **Zugrunde liegende Mechanismen:** Ausschließlich die D-Bus-API von NetworkManager (`org.freedesktop.NetworkManager` und zugehörige Objekte/Schnittstellen).63 Diese API bietet umfassende Funktionen zur Abfrage und Steuerung von Netzwerkverbindungen.
- **Implementierung:** Python mit `pydbus` oder `dasbus`, um die komplexen D-Bus-Interaktionen mit NetworkManager zu kapseln.

### 6.5 Energieverwaltungs-Server

- **Zweck:** Bietet direkten Zugriff auf Aktionen wie Herunterfahren, Neustarten, Ruhezustand und das Abrufen des Batteriestatus.
- **MCP-Schnittstellen:**
    - **Tools:**
        - `power.shutdown() -> boolean`
        - `power.restart() -> boolean`
        - `power.suspend() -> boolean`
        - `power.hibernate() -> boolean`
        - `power.lockScreen() -> boolean`
    - **Resources:**
        - `power.getBatteryStatus() -> {level: integer, charging: boolean, timeRemaining?: string}`: Gibt den aktuellen Batteriestatus zurück (falls zutreffend).
- **Zugrunde liegende Mechanismen:**
    - Aktionen (Shutdown, Restart, Suspend, Hibernate): D-Bus-Aufrufe an `org.freedesktop.login1.Manager`.81 Diese Methoden berücksichtigen Inhibit-Locks und PolicyKit-Berechtigungen.
    - Bildschirm sperren: D-Bus-Aufruf an den Bildschirmschoner der Desktop-Umgebung (z. B. `org.gnome.ScreenSaver.Lock`) oder `loginctl lock-session`.
    - Batteriestatus: D-Bus-Aufrufe an `org.freedesktop.UPower` oder direktes Lesen aus `/sys/class/power_supply/`.
- **Implementierung:** Python mit `pydbus` oder `dasbus` für die D-Bus-Interaktionen.

Diese Beispiele zeigen, wie spezifische Linux-Funktionen hinter einfachen, benutzerfreundlichen MCP-Schnittstellen abstrahiert werden können, die direkt von Desktop-Widgets genutzt werden können.

## 7. Sicherheitsarchitektur und Best Practices

Da die MCP-Server potenziell sensible Systemaktionen ausführen und auf Benutzerdaten zugreifen können, ist eine robuste Sicherheitsarchitektur unerlässlich. MCP selbst betont die Bedeutung von Sicherheit und Benutzerkontrolle.6

### 7.1 Authentifizierung und Autorisierung für lokale Server

Während die MCP-Spezifikation für HTTP-basierte Transporte (SSE) ein auf OAuth 2.1 basierendes Autorisierungsmodell vorsieht 18, ist dieser Ansatz für lokale Server, die über STDIO kommunizieren, weniger praktikabel und oft überdimensioniert. Stattdessen sollte die Autorisierung für lokale Desktop-Interaktionen durch den **MCP Host** (die Desktop-Umgebung) verwaltet werden.

Vorgeschlagener Mechanismus:

1. **Server-Registrierung:** Der Host verwaltet eine Liste vertrauenswürdiger, installierter MCP-Server. Diese könnten über Paketverwaltung oder ein dediziertes Verzeichnis bereitgestellt werden.
2. **Berechtigungsdefinition:** Für jeden Server oder jede Server-Kategorie werden granulare Berechtigungsbereiche (Scopes) definiert, die die Aktionen beschreiben, die der Server ausführen darf (z. B. `network:read`, `network:manage`, `settings:read`, `settings:write:appearance`, `files:read:home`, `power:control`). Diese Scopes sollten in der Server-Metadatenbeschreibung enthalten sein.
3. **Benutzerzustimmung (Consent):** Wenn ein Widget zum ersten Mal versucht, ein MCP-Tool aufzurufen, das eine bestimmte Berechtigung erfordert (z. B. `network:manage` für `network.setWifiEnabled`), prüft der Host, ob der Benutzer dieser Berechtigung für diesen spezifischen Server bereits zugestimmt hat.
4. **Consent Prompt:** Falls keine Zustimmung vorliegt, zeigt der Host dem Benutzer einen klaren Dialog an, der erklärt:
    - _Welches Widget_ (oder welche Anwendung)
    - _Welchen Server_
    - _Welche Aktion_ (basierend auf der Tool-Beschreibung) ausführen möchte
    - _Welche Berechtigung_ dafür erforderlich ist. Der Benutzer kann die Berechtigung erteilen (einmalig oder dauerhaft) oder ablehnen.
5. **Speicherung der Zustimmung:** Erteilte Berechtigungen werden sicher vom Host gespeichert (z. B. in der dconf-Datenbank des Benutzers).
6. **Durchsetzung:** Der Host erlaubt dem Client nur dann den Aufruf eines Tools, wenn die entsprechende Berechtigung für den Server erteilt wurde.

Dieser Ansatz verlagert die Komplexität der Autorisierung vom einzelnen Server zum zentralen Host, was besser zum Sicherheitsmodell von Desktop-Anwendungen passt und dem Benutzer eine zentrale Kontrolle über die Berechtigungen ermöglicht. Er spiegelt die Kernprinzipien von MCP wider: explizite Benutzerzustimmung und Kontrolle.6

### 7.2 Verwaltung der Benutzerzustimmung

Die explizite Zustimmung des Benutzers ist ein Eckpfeiler der MCP-Sicherheit.6 Der Host **muss** sicherstellen, dass:

- Benutzer klar verstehen, welche Aktionen ausgeführt werden sollen und welche Daten betroffen sind, bevor sie zustimmen. Die von den Servern bereitgestellten Beschreibungen für Tools und Resources sind hierfür entscheidend.
- Benutzer die Möglichkeit haben, erteilte Berechtigungen jederzeit einzusehen und zu widerrufen (z. B. über ein zentrales Einstellungsmodul in der Desktop-Umgebung).

### 7.3 Transportsicherheit (STDIO)

Obwohl STDIO eine lokale Kommunikationsform ist, muss der Host sicherstellen, dass er nur vertrauenswürdige, validierte MCP-Server-Executables startet. Die Server selbst sollten grundlegende Validierungen der über STDIO empfangenen Daten durchführen, um unerwartetes Verhalten durch fehlerhafte oder manipulierte Eingaben zu verhindern.

### 7.4 Prinzip der geringsten Rechte (Least Privilege)

MCP-Server sollten nur mit den minimal erforderlichen Berechtigungen laufen, um ihre definierte Funktion zu erfüllen. Das Ausführen von Servern als Root sollte unbedingt vermieden werden. Wenn erhöhte Rechte erforderlich sind (z. B. zum Ändern bestimmter Systemeinstellungen), sollten etablierte Mechanismen wie PolicyKit genutzt werden, idealerweise indem der MCP-Server einen bereits privilegierten D-Bus-Dienst kontaktiert, der die PolicyKit-Interaktion übernimmt, anstatt selbst Root-Rechte anzufordern.

### 7.5 Eingabevalidierung und -bereinigung (Input Sanitization)

Dies ist besonders kritisch für MCP-Server, die Kommandozeilen-Tools kapseln oder mit Dateipfaden arbeiten. Alle vom Client empfangenen Parameter, die zur Konstruktion von Befehlen, Dateipfaden oder D-Bus-Aufrufen verwendet werden, **müssen** rigoros validiert und bereinigt werden, um Sicherheitslücken wie Command Injection oder Directory Traversal zu verhindern.27 JSON-Schema-Validierung für Tool-Parameter ist ein erster wichtiger Schritt.27

Durch die Kombination aus Host-verwalteter Autorisierung, expliziter Benutzerzustimmung und sorgfältiger Implementierung der Server unter Beachtung der Sicherheitsprinzipien kann eine robuste und vertrauenswürdige MCP-Infrastruktur für Desktop-Widgets geschaffen werden.

## 8. Empfehlungen und Implementierungs-Roadmap

Basierend auf der Analyse des Model Context Protocol und seiner Anwendbarkeit auf die Vereinfachung von Linux-Desktop-Interaktionen für Umsteiger werden folgende Empfehlungen und eine mögliche Roadmap für die Implementierung vorgeschlagen.

### 8.1 Schlüsselempfehlungen

1. **MCP als Standard etablieren:** MCP sollte als standardisierte Schnittstelle zwischen Desktop-Widgets und den zugrunde liegenden Systemfunktionen für die Ziel-Linux-Distribution(en) eingeführt werden. Dies fördert Modularität, Wiederverwendbarkeit und Interoperabilität.
2. **Priorisierung der Server:** Die Entwicklung von MCP-Servern sollte sich zunächst auf die Bereiche konzentrieren, die für Windows/Mac-Umsteiger die größten Hürden darstellen und den größten Nutzen bringen: Netzwerkverwaltung, grundlegende Systemeinstellungen (Helligkeit, Lautstärke, Dark Mode), einfache Dateisuche und Anwendungsstart.
3. **Robuste Host-Komponente:** Die Entwicklung einer soliden MCP-Host-Komponente innerhalb der Desktop-Umgebung ist entscheidend. Diese Komponente ist verantwortlich für das Management der Client-Server-Verbindungen (insbesondere über STDIO), die Implementierung des vorgeschlagenen Consent- und Berechtigungsmodells und die Bereitstellung von APIs für Widget-Entwickler.
4. **Implementierungssprache:** Python erscheint aufgrund seiner ausgezeichneten Unterstützung für D-Bus-Interaktion (`dasbus`, `pydbus`), einfacher Handhabung von Subprozessen (für CLIs) und umfangreicher Standardbibliothek als geeignete Wahl für die Entwicklung der meisten MCP-Server. Alternativen wie Rust oder Go sind ebenfalls möglich, insbesondere wenn Performance kritisch ist.
5. **API-Design:** Der Fokus bei der Gestaltung der MCP Tool- und Resource-Schnittstellen muss auf Einfachheit, Klarheit und Plattformunabhängigkeit liegen, um die Bedürfnisse der Zielgruppe (Umsteiger) zu erfüllen.

### 8.2 Phasierter Implementierungsansatz

Eine schrittweise Einführung wird empfohlen, um frühzeitig Feedback zu sammeln und die Komplexität zu managen:

- **Phase 1: Grundlage schaffen (Proof of Concept)**
    - Entwicklung der Kernfunktionen des MCP Hosts (Client-Management für STDIO, grundlegende Consent-UI).
    - Implementierung von 1-2 fundamentalen MCP-Servern (z. B. Netzwerkstatus/-umschaltung, Helligkeits-/Lautstärkeregelung).
    - Erstellung einfacher Proof-of-Concept-Widgets, die diese Server nutzen.
    - Definition des initialen Satzes von Berechtigungs-Scopes.
- **Phase 2: Erweiterung und Stabilisierung**
    - Implementierung weiterer priorisierter Server (z. B. Energieverwaltung, Dateisuche, Anwendungsstart).
    - Verfeinerung des Sicherheitsmodells und der Consent-Verwaltung im Host.
    - Entwicklung eines breiteren Satzes von Widgets für gängige Anwendungsfälle.
    - Einführung von Mechanismen zur Server-Entdeckung und -Installation.
- **Phase 3: Fortgeschrittene Funktionen und Ökosystem**
    - Erkundung fortgeschrittener MCP-Funktionen wie Ressourcen-Abonnements für Echtzeit-Updates.51
    - Untersuchung potenzieller Anwendungsfälle für serverübergreifende Interaktionen oder die Nutzung von Prompts.
    - Evaluierung der Notwendigkeit von SSE für spezifische Remote- oder Cloud-Anwendungsfälle.
    - Integration in weitere Desktop-Umgebungen (falls erforderlich).
    - Förderung von Community-Beiträgen zur Entwicklung neuer MCP-Server.

### 8.3 Zukünftige Überlegungen

- **Desktop-Umgebungs-Vielfalt:** Die Anpassung der Server oder der Host-Logik an die spezifischen D-Bus-Schnittstellen oder `gsettings`-Schemas verschiedener Desktop-Umgebungen (GNOME, KDE Plasma, etc.) wird eine Herausforderung darstellen, wenn eine breite Kompatibilität angestrebt wird. Eine sorgfältige Abstraktion innerhalb der Server ist hierbei wichtig.
- **Remote-Management/Cloud-Integration:** Die Nutzung von MCP über SSE könnte zukünftig Szenarien wie die Fernsteuerung des Desktops oder die Integration von Cloud-Diensten in Widgets ermöglichen, erfordert aber signifikante zusätzliche Arbeit im Bereich Sicherheit und Authentifizierung.
- **KI-Integration (Sampling):** Die `Sampling`-Primitive von MCP 6 eröffnet die Möglichkeit, LLM-Funktionen direkt in Widgets zu integrieren, die über den Host bereitgestellt werden. Dies könnte für komplexere Aufgaben wie die Organisation von Dateien oder die Zusammenfassung von Systeminformationen genutzt werden, erfordert jedoch strenge Sicherheitskontrollen und Benutzerzustimmung.5
- **Community-Aufbau:** Die Schaffung einer Dokumentation und von Richtlinien für Drittentwickler zur Erstellung eigener MCP-Server könnte das Ökosystem erheblich erweitern und Nischenanwendungsfälle abdecken.

## 9. Schlussfolgerung

Die Implementierung einer auf dem Model Context Protocol basierenden Infrastruktur bietet einen vielversprechenden Ansatz, um die Interaktion mit Linux-Systemen für Benutzer, die von Windows oder macOS wechseln, erheblich zu vereinfachen. Durch die Standardisierung der Kommunikation zwischen Desktop-Widgets und Systemfunktionen über eine klar definierte Client-Server-Architektur und die Kapselung Linux-spezifischer Mechanismen (wie D-Bus und Kommandozeilen-Tools) in dedizierten MCP-Servern, kann eine intuitive und benutzerfreundliche Oberfläche geschaffen werden.

Die Verwendung von STDIO als primärem Transportmechanismus für die lokale Kommunikation vereinfacht die initiale Implementierung und erhöht die Sicherheit. Ein durch den Host verwaltetes Consent- und Berechtigungsmodell stellt sicher, dass der Benutzer die Kontrolle über Systemzugriffe behält, im Einklang mit den Sicherheitsprinzipien von MCP.

Die vorgeschlagene Roadmap ermöglicht eine schrittweise Einführung, beginnend mit den wichtigsten Funktionen für Umsteiger. Der Erfolg dieses Ansatzes hängt von der sorgfältigen Gestaltung der MCP-Schnittstellen zur Abstraktion der Systemkomplexität und der robusten Implementierung sowohl der Host-Komponente als auch der einzelnen MCP-Server ab. Durch die Nutzung des offenen MCP-Standards wird eine flexible, erweiterbare und potenziell über verschiedene Desktop-Umgebungen hinweg interoperable Lösung geschaffen, die das Potenzial hat, die Akzeptanz von Linux als Desktop-Betriebssystem maßgeblich zu fördern.
# Ganzheitliche Spezifikation des Model-Context-Protocol (MCP) und Entwicklungsplan für Desktop-Widgets unter Linux

## 1. Einführung in das Model-Context-Protocol (MCP)

Das Model-Context-Protocol (MCP) stellt einen wegweisenden offenen Standard dar, der von Anthropic Ende 2024 eingeführt wurde.1 Seine primäre Funktion besteht darin, die Interaktion zwischen KI-Assistenten und den komplexen, datenreichen Ökosystemen, in denen sie operieren, zu standardisieren und zu vereinfachen. MCP adressiert die Herausforderung fragmentierter und ad-hoc entwickelter Integrationen, indem es ein universelles Framework für die Anbindung von Werkzeugen (Tools), Datenquellen (Resources) und vordefinierten Arbeitsabläufen (Prompts) bereitstellt.1 Dies ermöglicht KI-Systemen einen nahtlosen und sicheren Zugriff auf vielfältige Kontexte, was für die Entwicklung kontextbewusster und leistungsfähiger KI-Anwendungen unerlässlich ist. Die Analogie eines "USB-C-Ports für KI-Anwendungen" verdeutlicht das Ziel von MCP: eine standardisierte Schnittstelle für den Datenaustausch und die Funktionserweiterung von KI-Modellen.2

Die Relevanz von MCP ergibt sich aus mehreren Schlüsselfaktoren. Erstens fördert es die **Reproduzierbarkeit** von KI-Modellen, indem alle notwendigen Details – Datensätze, Umgebungsspezifikationen und Hyperparameter – zentralisiert und standardisiert zugänglich gemacht werden.1 Zweitens ermöglicht es eine verbesserte **Standardisierung und Kollaboration**, insbesondere bei der organisationsübergreifenden Nutzung spezialisierter KI-Werkzeuge oder proprietärer Datenquellen.1 Drittens adressiert MCP direkt die Herausforderungen der Interoperabilität, Skalierbarkeit und Sicherheit, die mit der Anbindung von Large Language Models (LLMs) an externe Systeme einhergehen.1 Durch die Bereitstellung eines offenen Protokolls wird die Entwicklungszeit für Integrationen reduziert, die Wartbarkeit durch selbstdokumentierende Schnittstellen verbessert und die Flexibilität erhöht, Komponenten auszutauschen oder zu aktualisieren.2

MCP ist nicht nur darauf ausgelegt, Informationen abzurufen, sondern auch Aktionen auszuführen, wie das Aktualisieren von Dokumenten oder das Automatisieren von Workflows, wodurch die Lücke zwischen isolierter Intelligenz und dynamischer, kontextabhängiger Funktionalität geschlossen wird.1 Die Entwicklung und Pflege des MCP-Standards erfolgt durch eine Arbeitsgruppe und wird durch eine offene Governance-Struktur vorangetrieben, die eine kollaborative Gestaltung durch KI-Anbieter und die Community sicherstellt.2

## 2. Kernziele und Designprinzipien des MCP

Das Model-Context-Protocol (MCP) verfolgt zentrale Ziele, die seine Architektur und Funktionalität maßgeblich prägen. Ein primäres Ziel ist die **Standardisierung der Kontextbereitstellung** für LLMs.3 Anstatt für jede Kombination aus KI-Modell und externem System eine individuelle Integrationslösung entwickeln zu müssen, bietet MCP eine einheitliche Methode, um LLMs mit Daten und Werkzeugen zu verbinden.6 Dies ist vergleichbar mit dem Language Server Protocol (LSP), das die Integration von Programmiersprachen in Entwicklungsumgebungen standardisiert.6

Weitere Kernziele umfassen:

- **Verbesserte Interoperabilität:** Ermöglichung der nahtlosen Zusammenarbeit verschiedener KI-Systeme und externer Dienste.1
- **Erhöhte Skalierbarkeit:** Vereinfachung der Erweiterung von KI-Anwendungen durch modulare Anbindung neuer Datenquellen und Werkzeuge.2
- **Gewährleistung von Sicherheit und Kontrolle:** Implementierung robuster Mechanismen für Benutzerzustimmung, Datenschutz und sichere Werkzeugausführung.1
- **Reduzierung des Entwicklungsaufwands:** Schnellere Integration durch standardisierte Muster und Protokolle.2

Diese Ziele spiegeln sich in den fundamentalen Designprinzipien des MCP wider, die insbesondere die Server-Implementierung und -Interaktion betreffen 10:

1. **Einfachheit der Server-Erstellung:** MCP-Server sollen extrem einfach zu erstellen sein. Host-Anwendungen übernehmen komplexe Orchestrierungsaufgaben, während sich Server auf spezifische, klar definierte Fähigkeiten konzentrieren. Einfache Schnittstellen und eine klare Trennung der Zuständigkeiten minimieren den Implementierungsaufwand und fördern wartbaren Code.10
2. **Hohe Komponierbarkeit der Server:** Jeder Server bietet isoliert eine fokussierte Funktionalität. Mehrere Server können nahtlos kombiniert werden, da das gemeinsame Protokoll Interoperabilität ermöglicht. Dieses modulare Design unterstützt die Erweiterbarkeit des Gesamtsystems.10
3. **Datenisolation und kontrollierter Kontextzugriff:** Server sollen nicht die gesamte Konversation lesen oder Einblick in andere Server erhalten können. Sie empfangen nur die notwendigen kontextuellen Informationen. Die vollständige Konversationshistorie verbleibt beim Host, und jede Serververbindung ist isoliert. Interaktionen zwischen Servern werden vom Host gesteuert, der die Sicherheitsgrenzen durchsetzt.10
4. **Progressive Erweiterbarkeit von Funktionen:** Funktionen können schrittweise zu Servern und Clients hinzugefügt werden. Das Kernprotokoll bietet eine minimale erforderliche Funktionalität, und zusätzliche Fähigkeiten können bei Bedarf ausgehandelt werden. Dies ermöglicht eine unabhängige Entwicklung von Servern und Clients und stellt die zukünftige Erweiterbarkeit des Protokolls unter Wahrung der Abwärtskompatibilität sicher.10

Diese Prinzipien unterstreichen das Bestreben von MCP, ein flexibles, sicheres und entwicklerfreundliches Ökosystem für die Integration von KI-Modellen mit ihrer Umgebung zu schaffen. Die Betonung der Benutzerkontrolle, des Datenschutzes und der Sicherheit von Werkzeugen sind dabei zentrale Säulen für vertrauenswürdige KI-Lösungen in realen Anwendungen.1

## 3. Die Architektur des Model-Context-Protocol

Das Model-Context-Protocol (MCP) basiert auf einer Client-Host-Server-Architektur, die darauf ausgelegt ist, KI-Anwendungen (Agenten) mit externen Systemen, Datenquellen und Werkzeugen zu verbinden, während klare Sicherheitsgrenzen gewahrt bleiben.1 Diese Architektur nutzt JSON-RPC für die Kommunikation und etabliert zustandsbehaftete Sitzungen zur Koordination des Kontexaustauschs und des Samplings.1

Die Kernkomponenten dieser Architektur sind:

### 3.1. MCP Host: Der Orchestrator

Der Host-Prozess fungiert als Container oder Koordinator für eine oder mehrere Client-Instanzen.1 Er ist die zentrale Anwendungsinstanz, die MCP nutzt, um auf Daten und Werkzeuge zuzugreifen, beispielsweise eine Desktop-Anwendung wie Claude Desktop, eine integrierte Entwicklungsumgebung (IDE) oder eine andere KI-gestützte Applikation.2

Zu den Hauptverantwortlichkeiten des Hosts gehören 1:

- Erstellung und Verwaltung des Lebenszyklus von Client-Instanzen.
- Kontrolle der Verbindungserlaubnisse für Clients.
- Durchsetzung von Sicherheitsrichtlinien, Benutzerautorisierung und Einholung von Zustimmungen (Consent).
- Koordination der Integration von KI- oder Sprachmodellen innerhalb jedes Clients, einschließlich des Sammelns und Zusammenführens von Kontextinformationen.
- Verwaltung der Kontextaggregation über verschiedene Clients hinweg.

Der Host spielt eine entscheidende Rolle bei der Wahrung der Sicherheit und des Datenschutzes, indem er sicherstellt, dass Benutzer explizit zustimmen und die Kontrolle über den Datenzugriff und die ausgeführten Operationen behalten.8

### 3.2. MCP Client: Der Vermittler

Jede Client-Instanz wird vom Host erstellt und läuft innerhalb des Host-Prozesses.1 Ein Client stellt eine dedizierte, zustandsbehaftete Eins-zu-Eins-Sitzung mit einem spezifischen MCP-Server her.1 Er fungiert als Vermittler, der die Kommunikation zwischen dem Host (und dem darin integrierten LLM) und dem Server handhabt.

Die Aufgaben des Clients umfassen 1:

- Aushandlung von Protokollversionen und Fähigkeiten (Capabilities) mit dem Server.
- Orchestrierung und Weiterleitung von Nachrichten zwischen sich und dem Server.
- Verwaltung von Abonnements und Benachrichtigungen.
- Aufrechterhaltung von Sicherheitsgrenzen, sodass ein Client nicht auf Ressourcen zugreifen kann, die einem anderen Client oder Server zugeordnet sind.
- Übersetzung der Anforderungen des Hosts in das MCP-Format und umgekehrt.

Die Client-Komponente ist somit für die zuverlässige und sichere Kommunikation sowie für die Verwaltung des Zustands der Verbindung zu einem einzelnen Server zuständig.2

### 3.3. MCP Server: Der Fähigkeitsanbieter

Ein MCP-Server ist ein eigenständiges Programm oder ein Dienst, der spezifische Datenquellen, APIs oder andere Dienstprogramme (wie CRMs, Git-Repositories oder Dateisysteme) kapselt und deren Fähigkeiten über das MCP-Protokoll bereitstellt.1 Server können lokal als Subprozess der Anwendung oder entfernt als über Netzwerk erreichbare Dienste betrieben werden.1

Die Hauptfunktionen eines Servers sind 1:

- Definition und Bereitstellung von "Tools" (ausführbare Funktionen), "Resources" (Datenquellen) und "Prompts" (vordefinierte Vorlagen), die der Client abrufen oder ausführen kann.
- Verarbeitung von Anfragen des Clients (z.B. Ausführung eines Tools, Lesen einer Ressource).
- Rückgabe von Ergebnissen oder Daten an den Client in einem standardisierten Format.
- Einhaltung der vom Host durchgesetzten Sicherheitsbeschränkungen und Benutzerberechtigungen.
- Potenzielles Anfordern von Sampling-Operationen über Client-Schnittstellen.

Server sind darauf ausgelegt, fokussierte Verantwortlichkeiten zu übernehmen und hochgradig komponierbar zu sein.10 Beispiele für MCP-Server sind der offizielle Dateisystem-Server 3, der PiecesOS-Server für personalisierten Kontext 11 oder der Merge MCP-Server, der Zugriff auf Hunderte von APIs über eine einzige Schnittstelle ermöglicht.12 Es gibt auch von der Community entwickelte Server für verschiedenste Anwendungen wie PostgreSQL, Slack, Git, GitHub und viele mehr.6

Die klare Trennung der Verantwortlichkeiten zwischen Host, Client und Server ermöglicht eine modulare und skalierbare Architektur. Der Host kann komplexe Orchestrierungslogik handhaben, während Server sich auf die Bereitstellung spezifischer Fähigkeiten konzentrieren. Dies erleichtert die Entwicklung und Wartung von sowohl den Host-Anwendungen als auch den einzelnen Server-Komponenten erheblich.10

## 4. Details des Model-Context-Protocol

Das Model-Context-Protocol (MCP) definiert die genauen Regeln und Formate für die Kommunikation zwischen den Komponenten seiner Architektur. Es baut auf etablierten Standards auf und erweitert diese um spezifische Mechanismen für den Austausch von Kontext und die Steuerung von KI-Interaktionen.

### 4.1. Kommunikationsgrundlage: JSON-RPC 2.0

MCP verwendet JSON-RPC 2.0 als zugrundeliegendes Nachrichtenformat für die gesamte Kommunikation zwischen Clients und Servern.1 JSON-RPC 2.0 ist ein leichtgewichtiges, zustandsloses Protokoll für Remote Procedure Calls, das sich durch seine Einfachheit und breite Unterstützung auszeichnet.4

Die Nachrichtenstruktur in JSON-RPC 2.0 umfasst drei Haupttypen 14:

1. **Requests (Anfragen):** Nachrichten, die eine Operation auf dem entfernten System initiieren und eine Antwort erwarten. Sie enthalten:
    - `jsonrpc: "2.0"`
    - `id: string | number` (eine eindeutige Kennung für die Anfrage, darf nicht `null` sein und nicht innerhalb derselben Sitzung vom Anforderer wiederverwendet werden 16)
    - `method: string` (Name der auszuführenden Methode/Prozedur)
    - `params?: object | array` (Parameter für die Methode)
2. **Responses (Antworten):** Nachrichten, die als Reaktion auf eine Anfrage gesendet werden. Sie enthalten:
    - `jsonrpc: "2.0"`
    - `id: string | number` (muss mit der ID der ursprünglichen Anfrage übereinstimmen 16)
    - Entweder `result: any` (bei erfolgreicher Ausführung) oder `error: object` (bei einem Fehler). Eine Antwort darf nicht sowohl `result` als auch `error` enthalten.16
    - Das `error`-Objekt enthält `code: number` (ein Integer-Fehlercode), `message: string` und optional `data: any` für zusätzliche Fehlerinformationen.16
3. **Notifications (Benachrichtigungen):** Nachrichten, die gesendet werden, um das entfernte System zu informieren, aber keine direkte Antwort erwarten. Sie enthalten:
    - `jsonrpc: "2.0"`
    - `method: string`
    - `params?: object | array`
    - Notifications dürfen keine `id` enthalten.16

Obwohl JSON-RPC 2.0 an sich zustandslos ist, baut MCP darauf **zustandsbehaftete Sitzungen** (stateful sessions) auf.1 Das bedeutet, dass die MCP-Schicht oberhalb von JSON-RPC für die Verwaltung des Sitzungskontexts, der Sequenz von Operationen und der ausgehandelten Fähigkeiten verantwortlich ist. Diese Zustandsbehaftung ist entscheidend für Funktionen wie Ressourcenabonnements oder die Verfolgung laufender Operationen.

#### 4.1.2. Standard-MCP-Methoden (z.B. `initialize`, `shutdown`, `ping`, `$/cancelRequest`)

Über die spezifischen Methoden für Tools, Resources und Prompts hinaus definiert MCP eine Reihe von Standard-JSON-RPC-Methoden, die für die Verwaltung der Sitzung und grundlegende Protokolloperationen unerlässlich sind.

Die folgende Tabelle gibt einen Überblick über wichtige Standardmethoden im MCP:

|   |   |   |   |   |   |
|---|---|---|---|---|---|
|**Methodenname**|**Richtung**|**Zweck**|**Wichtige Parameter (Beispiele)**|**Erwartete Antwort/Verhalten**|**Referenzen**|
|`initialize`|Client → Server|Startet die Sitzung, handelt Protokollversion und Fähigkeiten aus.|`protocolVersion`, `capabilities` (Client), `clientInfo`|Antwort mit `protocolVersion` (Server), `capabilities` (Server), `serverInfo`|10|
|`initialized`|Client → Server (Notification)|Bestätigt den erfolgreichen Abschluss der Initialisierung durch den Client.|Keine|Keine (Notification)|14|
|`shutdown`|Client → Server (oder Server → Client)|Fordert ein sauberes Herunterfahren der Verbindung an.|Keine|Leere Erfolgsantwort oder Fehler|14|
|`exit`|Server → Client (oder Client → Server) (Notification)|Benachrichtigt die Gegenseite, dass der Sender sich beendet.|Keine|Keine (Notification)|19|
|`ping`|Client ↔ Server|Überprüft die Verbindung und misst ggf. Latenz.|Optional: `payload`|`pong`-Antwort mit demselben `payload`|8 (impliziert)|
|`$/cancelRequest`|Client ↔ Server (Notification)|Fordert den Abbruch einer zuvor gesendeten Anfrage mit einer bestimmten ID.|`id` der abzubrechenden Anfrage|Keine (Notification)|8|
|`notifications/message`|Server → Client (Notification)|Sendet Log- oder andere Informationsnachrichten vom Server an den Client.|`level` (z.B. "error", "info"), `logger`, `data`|Keine (Notification)|8|

Die `initialize`-Handshake-Sequenz ist von fundamentaler Bedeutung, da sie die Kompatibilität der Protokollversionen sicherstellt und die Fähigkeiten von Client und Server austauscht.10 Dies bildet den "Vertrag" für die Dauer der Sitzung und stellt sicher, dass beide Seiten verstehen, welche Operationen die andere Seite unterstützt, wodurch Fehler durch den Versuch, nicht unterstützte Funktionen zu nutzen, vermieden werden. Eine korrekte Implementierung von `shutdown` und `exit` ist ebenso kritisch, um Ressourcenlecks und verwaiste Prozesse zu verhindern, insbesondere bei `stdio`-basierten Servern, wo das Schließen von Streams und das Senden von Signalen Teil des geordneten Beendigungsprozesses sind.19

### 4.2. Transportmechanismen

MCP definiert zwei primäre Transportmechanismen für die Übermittlung der JSON-RPC-Nachrichten.3

#### 4.2.1. Standard Input/Output (stdio) für lokale Server

Bei diesem Transportmechanismus wird der MCP-Server als Subprozess der Client-Anwendung (des Hosts) gestartet.3 Die Kommunikation erfolgt über die Standard-Eingabe (`stdin`) und Standard-Ausgabe (`stdout`) des Subprozesses.14 Nachrichten werden typischerweise als JSON-RPC-formatierte Strings gesendet, die durch Zeilenumbrüche voneinander getrennt sind.14

**Anwendungsfälle:**

- Lokale Integrationen, bei denen Client und Server auf derselben Maschine laufen.6
- Kommandozeilenwerkzeuge (CLI-Tools), die MCP-Fähigkeiten bereitstellen.14

**Sicherheitsaspekte:** Die Sicherheit ist bei `stdio`-Transporten tendenziell einfacher zu handhaben, da die Kommunikation lokal erfolgt und oft in einer vertrauenswürdigen Umgebung stattfindet.15 Dennoch ist die Validierung von Eingaben weiterhin wichtig.

**Beispiel Serverstart (Merge MCP):**

JSON

```
{
  "mcpServers": {
    "merge-mcp": {
      "command": "uvx",
      "args": ["merge-mcp"],
      "env": {
        "MERGE_API_KEY": "your_api_key",
        "MERGE_ACCOUNT_TOKEN": "your_account_token"
      }
    }
  }
}
```

Dieses Beispiel aus der Merge-Dokumentation zeigt, wie ein `stdio`-basierter MCP-Server über einen Befehl und Argumente gestartet wird.12

#### 4.2.2. HTTP mit Server-Sent Events (SSE) für entfernte Server

Für die Kommunikation mit entfernten Servern unterstützt MCP HTTP in Kombination mit Server-Sent Events (SSE).3 Dabei werden Anfragen vom Client an den Server typischerweise über HTTP POST gesendet, während der Server SSE nutzt, um Nachrichten und Updates asynchron an den Client zu streamen.6

**Anwendungsfälle:**

- Entfernte MCP-Server, die über ein Netzwerk erreichbar sind.3
- Web-basierte Anwendungen, die MCP-Funktionalitäten integrieren.14

**Sicherheitsaspekte:** Dieser Transportmechanismus erfordert besondere Aufmerksamkeit hinsichtlich der Sicherheit 15:

- **Authentifizierung und Autorisierung:** Verbindungen müssen gesichert werden, z.B. durch Token-basierte Authentifizierung.
- **Transportverschlüsselung:** TLS (HTTPS) ist unerlässlich, um die Datenübertragung zu verschlüsseln.14
- **Origin-Header-Validierung:** Um Cross-Site-Request-Forgery (CSRF) und andere Angriffe zu verhindern, müssen Server die `Origin`-Header eingehender SSE-Verbindungen validieren.15
- **DNS Rebinding Schutz:** Server sollten nur an `localhost` (127.0.0.1) binden, wenn sie lokal laufen, und nicht an `0.0.0.0`, um DNS-Rebinding-Angriffe zu erschweren, bei denen entfernte Webseiten versuchen, mit lokalen MCP-Servern zu interagieren.15

Die Wahl des Transportmechanismus hat erhebliche Auswirkungen auf die Sicherheitsarchitektur und die Komplexität der Bereitstellung. Während `stdio` für lokale, vertrauenswürdige Umgebungen einfacher ist, erfordert HTTP/SSE robuste Netzwerksicherheitsmaßnahmen.15 Entwickler haben zudem die Möglichkeit, eigene, benutzerdefinierte Transportmechanismen zu implementieren, sofern diese die `Transport`-Schnittstelle erfüllen und die MCP-Nachrichtenformate und den Lebenszyklus korrekt handhaben.14 Dies bietet Flexibilität für spezialisierte Kommunikationskanäle, verlagert aber auch die Verantwortung für die korrekte und sichere Implementierung auf den Entwickler.

### 4.3. Management des Sitzungslebenszyklus

Eine MCP-Sitzung durchläuft klar definierte Phasen, von der Initialisierung über den aktiven Nachrichtenaustausch bis hin zur Beendigung.1

#### 4.3.1. Initialisierung und bidirektionale Fähigkeitsaushandlung (Capability Negotiation)

Die Sitzung beginnt mit einer Initialisierungsphase, die vom Client initiiert wird.14

1. Der Client sendet eine `initialize`-Anfrage an den Server. Diese Anfrage enthält:
    - Die vom Client unterstützte Protokollversion (sollte die neueste sein, die der Client unterstützt).19
    - Die Fähigkeiten (Capabilities) des Clients (z.B. Unterstützung für Sampling).10
    - Informationen zur Client-Implementierung (z.B. Name, Version).19
2. Der Server antwortet auf die `initialize`-Anfrage. Die Antwort enthält:
    - Die vom Server für diese Sitzung gewählte Protokollversion (muss mit der vom Client angeforderten übereinstimmen, wenn unterstützt).19
    - Die Fähigkeiten des Servers (z.B. welche Tools, Resources, Prompts er anbietet, Unterstützung für Ressourcenabonnements).10
    - Informationen zur Server-Implementierung.19
3. Nach Erhalt der erfolgreichen `initialize`-Antwort sendet der Client eine `initialized`-Benachrichtigung an den Server, um den Abschluss der Initialisierungsphase zu bestätigen.14

Während dieser Phase dürfen Client und Server vor der `initialize`-Antwort bzw. der `initialized`-Benachrichtigung keine anderen Anfragen als `ping` oder Logging-Nachrichten senden.19 Beide Parteien müssen die ausgehandelte Protokollversion respektieren und dürfen nur Fähigkeiten nutzen, die erfolgreich ausgehandelt wurden.19 Diese Fähigkeitsaushandlung ist ein Eckpfeiler der Erweiterbarkeit von MCP. Sie ermöglicht es dem Protokoll, sich mit neuen Funktionen weiterzuentwickeln, ohne bestehende Implementierungen zu brechen, falls diese neuere Fähigkeiten nicht unterstützen.10

#### 4.3.2. Aktiver Nachrichtenaustausch

Nach erfolgreicher Initialisierung beginnt der eigentliche Nachrichtenaustausch.14 Clients und Server können nun Anfragen (Request-Response) und Benachrichtigungen (Notifications) gemäß den ausgehandelten Fähigkeiten austauschen. Dies umfasst beispielsweise das Auflisten und Aufrufen von Tools, das Lesen von Ressourcen, das Abonnieren von Ressourcenänderungen oder das Abrufen von Prompts.

#### 4.3.3. Saubere Beendigung und Shutdown-Prozeduren

Die Verbindung kann auf verschiedene Weisen beendet werden 14:

- **Sauberes Herunterfahren:** Eine Seite (Client oder Server) sendet eine `shutdown`-Anfrage an die andere. Nach erfolgreicher Antwort auf `shutdown` sendet die initiierende Seite eine `exit`-Benachrichtigung, woraufhin beide Seiten die Verbindung schließen und Ressourcen freigeben.
- **Spezifische Prozeduren für `stdio`-Transport 19:**
    1. Der Client sollte das Herunterfahren initiieren, indem er zuerst den Eingabe-Stream zum Kindprozess (Server) schließt.
    2. Der Client wartet, bis der Server sich beendet.
    3. Wenn der Server sich nicht innerhalb einer angemessenen Zeit beendet, sendet der Client `SIGTERM`.
    4. Wenn der Server nach `SIGTERM` immer noch nicht beendet ist, sendet der Client `SIGKILL`.
- **Transport-Diskonnektion:** Eine unerwartete Unterbrechung der zugrundeliegenden Transportverbindung.
- **Fehlerbedingungen:** Kritische Fehler können ebenfalls zur Beendigung führen.

Ein robustes Management des Lebenszyklus ist entscheidend für zuverlässige, langlebige MCP-Integrationen, um Ressourcenlecks oder blockierte Zustände zu vermeiden.

### 4.4. Zustandsmanagement und Synchronisation in zustandsbehafteten Sitzungen

Obwohl JSON-RPC 2.0 ein zustandsloses Protokoll ist, sind MCP-Sitzungen explizit als zustandsbehaftet (stateful) konzipiert.1 Dieser Zustand wird über die Dauer der Verbindung zwischen einem Client und einem Server aufrechterhalten.

**Wie Zustand verwaltet wird:**

- **Initialisierungsphase:** Der grundlegende Zustand wird durch die während der `initialize`-Sequenz ausgehandelten Fähigkeiten (Capabilities) etabliert.19 Diese definieren, welche Operationen während der Sitzung gültig sind.
- **Serverseitiger Kontext:** Server müssen oft sitzungsspezifischen Zustand verwalten. Ein wichtiges Beispiel ist das Management von Ressourcenabonnements: Wenn ein Client eine Ressource abonniert (`resources/subscribe`), muss der Server sich diesen Client und die abonnierte Ressource merken, um bei Änderungen `notifications/resources/updated`-Benachrichtigungen senden zu können.23
- **Clientseitiges Wissen:** Clients müssen ebenfalls den Zustand der Verbindung und die Fähigkeiten des Servers kennen, um gültige Anfragen zu stellen.
- **Sequenz von Operationen:** Bestimmte Operationen können von vorherigen Aktionen abhängen (z.B. kann ein `tools/call` erst nach einem `tools/list` sinnvoll sein, wenn der Toolname nicht vorab bekannt ist).

Synchronisation:

Die Synchronisation des Zustands erfolgt implizit durch den definierten Nachrichtenfluss von Anfragen, Antworten und Benachrichtigungen.

- **Anfragen und Antworten:** Modifizieren oder fragen den Zustand ab (z.B. `resources/subscribe` ändert den Abonnementstatus auf dem Server).
- **Benachrichtigungen:** Informieren über Zustandsänderungen (z.B. `notifications/resources/updated` informiert den Client über eine geänderte Ressource, `notifications/tools/list_changed` über eine neue Werkzeugliste 25).

Entwickler von MCP-Servern, insbesondere solche, die Ressourcenabonnements oder langlebige Werkzeuge anbieten, müssen den sitzungsspezifischen Zustand sorgfältig verwalten. Da ein Server potenziell Verbindungen zu mehreren Clients gleichzeitig handhaben kann (obwohl jede Client-Instanz eine 1:1-Sitzung mit einem Server hat 1), ist die Isolation des Zustands zwischen den Sitzungen entscheidend, um Fehlinformationen oder "Cross-Talk" zu verhindern. Beispielsweise darf ein Update für eine von Client A abonnierte Ressource nicht an Client B gesendet werden, es sei denn, Client B hat dieselbe Ressource ebenfalls abonniert.

### 4.5. Umfassende Fehlerbehandlung und standardisierte Fehlercodes

MCP nutzt das Standard-Fehlerobjekt von JSON-RPC 2.0 für die Meldung von Fehlern.14 Dieses Objekt enthält die Felder `code` (eine Ganzzahl), `message` (eine menschenlesbare Beschreibung) und optional `data` (für zusätzliche, anwendungsspezifische Fehlerdetails).

MCP unterscheidet zwischen:

1. **Protokollfehlern:** Fehler, die auf der Ebene des JSON-RPC-Protokolls oder der grundlegenden MCP-Interaktion auftreten (z.B. eine falsch formatierte Anfrage, eine unbekannte Methode). Hierfür werden oft die Standard-JSON-RPC-Fehlercodes verwendet.
2. **Anwendungs-/Werkzeugausführungsfehlern:** Fehler, die während der Ausführung einer serverseitigen Logik auftreten (z.B. ein Tool kann eine externe API nicht erreichen). Diese werden oft innerhalb einer erfolgreichen JSON-RPC-Antwort signalisiert, typischerweise durch ein `isError: true`-Flag im `result`-Objekt eines `tools/call`.26

Die folgende Tabelle listet einige bekannte Standardfehlercodes und ihre Bedeutung im Kontext von MCP auf:

|   |   |   |   |   |
|---|---|---|---|---|
|**Fehlercode**|**Symbolischer Name (JSON-RPC)**|**Beschreibung**|**Typische Ursache im MCP**|**Referenzen**|
|-32700|Parse error|Ungültiges JSON wurde vom Server empfangen.|Fehlerhafte JSON-Serialisierung beim Client.|JSON-RPC 2.0 Spec|
|-32600|Invalid Request|Die gesendete JSON ist keine gültige Anfrage.|Anfrageobjekt entspricht nicht der JSON-RPC-Spezifikation.|18 (impliziert)|
|-32601|Method not found|Die angeforderte Methode existiert nicht oder ist nicht verfügbar.|Client ruft eine nicht unterstützte MCP-Methode auf.|18 (impliziert)|
|-32602|Invalid params|Ungültige Methodenparameter.|Falsche oder fehlende Parameter bei einem Methodenaufruf (z.B. unbekanntes Tool 27, ungültiger Prompt-Name 28, ungültiger Log-Level 20).|20|
|-32603|Internal error|Interner JSON-RPC-Fehler oder serverseitiger Fehler.|Nicht spezifizierter Fehler auf dem Server während der Protokollverarbeitung oder Ausführung (z.B. bei Ressourcen 24, Prompts 28).|24|
|-32000 bis -32099|Server error|Reserviert für implementierungsdefinierte Server-Fehler.||JSON-RPC 2.0 Spec|
|-32002|(MCP-spezifisch)|Ressource nicht gefunden.|Client fordert eine Ressource an, die nicht existiert oder nicht zugänglich ist.|24|

Diese mehrschichtige Fehlerbehandlung – Unterscheidung zwischen Protokollfehlern und anwendungsspezifischen Fehlern innerhalb der Tool-Ergebnisse – ermöglicht eine präzise Fehlerdiagnose. Clients müssen darauf vorbereitet sein, beide Arten von Fehlern adäquat zu verarbeiten, um eine stabile Benutzererfahrung zu gewährleisten und aussagekräftige Fehlermeldungen oder Fallback-Strategien zu implementieren.

## 5. MCP-Primitive: Tools, Resources und Prompts im Detail

MCP definiert drei Kernprimitive – Tools, Resources und Prompts – über die Server ihre Fähigkeiten und Daten für LLM-Anwendungen bereitstellen.1 Jedes Primitiv hat einen spezifischen Zweck und ein eigenes Interaktionsmodell.

Die folgende Tabelle gibt einen vergleichenden Überblick:

|   |   |   |   |   |
|---|---|---|---|---|
|**Primitiv**|**Primärer Zweck**|**Wichtige JSON-RPC-Methoden**|**Kontrolle durch**|**Typische Anwendungsfälle**|
|**Tools**|Ausführung von Aktionen, Interaktion mit Systemen|`tools/list`, `tools/call`, `notifications/tools/list_changed`|Modell (mit Benutzerzustimmung)|API-Aufrufe, Datenbankabfragen, Dateimanipulation, Berechnungen, Codeausführung|
|**Resources**|Bereitstellung von Daten und Kontext|`resources/list`, `resources/read`, `resources/subscribe`, `resources/unsubscribe`, `notifications/resources/list_changed`, `notifications/resources/updated`|Anwendung/Benutzer (ggf. Modell)|Dateiinhalte, Datenbankeinträge, API-Antworten, Systemstatus, Bilder, Logdateien|
|**Prompts**|Strukturierung von LLM-Interaktionen, Workflows|`prompts/list`, `prompts/get`, `notifications/prompts/list_changed`|Benutzer (oft über UI-Elemente)|Vordefinierte Abfragen, Code-Review-Anfragen, Zusammenfassungen, Slash-Befehle in Chats|

Diese Unterscheidung hilft Entwicklern, die passende Methode zur Integration ihrer spezifischen Funktionalitäten in das MCP-Ökosystem zu wählen.

### 5.1. Tools: Ermöglichung von KI-Aktionen

Tools sind ausführbare Funktionen, die von LLMs (modellgesteuert) aufgerufen werden können, um mit externen Systemen zu interagieren, Berechnungen durchzuführen oder Aktionen in der realen Welt auszulösen.2 Eine entscheidende Komponente ist dabei die explizite Zustimmung des Benutzers ("human in the loop") vor der Ausführung eines Tools, um Sicherheit und Kontrolle zu gewährleisten.2

#### 5.1.1. Definition, JSON Schema (Input/Output) und Annotationen

Eine Tool-Definition im MCP umfasst typischerweise 6:

- **`name: string`**: Ein eindeutiger Bezeichner für das Tool.
- **`description?: string`**: Eine menschenlesbare Beschreibung der Funktionalität des Tools.
- **`inputSchema: object`**: Ein JSON-Schema, das die erwarteten Eingabeparameter des Tools definiert. Dies ermöglicht Validierung und Typüberprüfung. In TypeScript-SDKs wird hierfür oft `zod` verwendet.21
- **`annotations?: object`**: Optionale Hinweise zum Verhalten des Tools, die primär für die Benutzeroberfläche gedacht sind und nicht den Modellkontext beeinflussen. Beispiele 25:
    - `title?: string`: Ein menschenlesbarer Titel für das Tool.
    - `readOnlyHint?: boolean`: Gibt an, ob das Tool seine Umgebung nicht verändert.
    - `destructiveHint?: boolean`: Gibt an, ob das Tool potenziell destruktive Änderungen vornehmen kann.
    - `idempotentHint?: boolean`: Gibt an, ob wiederholte Aufrufe mit denselben Argumenten keinen zusätzlichen Effekt haben.
    - `openWorldHint?: boolean`: Gibt an, ob das Tool mit der "offenen Welt" (z.B. Internet) interagiert.

Diese Annotationen sind besonders wertvoll, da sie es Host-Anwendungen ermöglichen, Benutzer transparent über die potenziellen Auswirkungen eines Tool-Aufrufs zu informieren, bevor diese ihre Zustimmung geben.25 Die Verwendung von JSON Schema für `inputSchema` fördert zudem robuste und typsichere Interaktionen, da sie eine standardisierte Validierung von Parametern erlaubt.25

**JSON Schema Beispiel für ein Tool (abgeleitet von 25):**

JSON

```
{
  "name": "get_weather",
  "description": "Get current weather information for a location",
  "inputSchema": {
    "type": "object",
    "properties": {
      "location": {
        "type": "string",
        "description": "City name or zip code"
      }
    },
    "required": ["location"]
  },
  "annotations": { "readOnlyHint": true }
}
```

#### 5.1.2. Entdeckung (`tools/list`) und Aufruf (`tools/call`)

- **`tools/list`**: Clients verwenden diese Methode, um eine Liste der vom Server bereitgestellten Tools zu erhalten.3 Die Antwort enthält die Definitionen der verfügbaren Tools. Clients können diese Liste zwischenspeichern, um Latenz zu reduzieren, sollten aber beachten, dass sich die Tool-Liste ändern kann (siehe `notifications/tools/list_changed`).3
- **`tools/call`**: Mit dieser Methode ruft ein Client ein spezifisches Tool auf dem Server auf, indem er den Tool-Namen und die erforderlichen Argumente übergibt.3

**JSON Beispiel für eine `tools/call`-Anfrage (abgeleitet von 7):**

JSON

```
{
  "jsonrpc": "2.0",
  "id": "call123",
  "method": "tools/call",
  "params": {
    "name": "get_weather",
    "arguments": { "location": "New York" }
  }
}
```

**JSON Beispiel für eine `tools/call`-Antwort (abgeleitet von 27):**

JSON

```
{
  "jsonrpc": "2.0",
  "id": "call123",
  "result": {
    "content": [{ "type": "text", "text": "Current weather in New York: 72°F, Partly cloudy" }],
    "isError": false
  }
}
```

Server können Clients über Änderungen in der Tool-Liste mittels der `notifications/tools/list_changed`-Benachrichtigung informieren.25

#### 5.1.3. Handhabung von Tool-Ergebnissen und Ausführungsfehlern

Die Antwort auf einen `tools/call`-Aufruf hat eine definierte Struktur 26:

- **`content: array`**: Ein Array von Inhaltsobjekten, die das Ergebnis der Tool-Ausführung darstellen. Jedes Objekt kann verschiedene Typen haben (z.B. `text`, `image`, `resource`).
- **`isError: boolean`**: Ein Flag, das angibt, ob bei der Ausführung des Tools ein Fehler aufgetreten ist.

Es ist wichtig, zwischen Protokollfehlern (z.B. Tool nicht gefunden, ungültige Parameter, gemeldet über das JSON-RPC `error`-Objekt) und Tool-Ausführungsfehlern (gemeldet via `isError: true` und einer Beschreibung im `content`-Array) zu unterscheiden.26

#### 5.1.4. Sicherheitsimperative für Tool-Design und -Ausführung

Aufgrund der potenziellen Mächtigkeit von Tools sind strenge Sicherheitsmaßnahmen unerlässlich 8:

- **Serverseitig:**
    - Strikte Validierung aller Eingabeparameter gegen das `inputSchema`.
    - Implementierung von Zugriffskontrollen (wer darf welche Tools aufrufen?).
    - Rate Limiting, um Missbrauch oder Überlastung zu verhindern.
    - Sorgfältige Behandlung und Bereinigung von Ausgaben.
- **Clientseitig (Host):**
    - Einholen expliziter Benutzerzustimmung vor jedem Tool-Aufruf.
    - Anzeige der Tool-Eingaben für den Benutzer vor dem Senden an den Server, um versehentliche oder böswillige Datenexfiltration zu vermeiden.
    - Validierung der Tool-Ergebnisse, bevor sie dem LLM oder Benutzer präsentiert werden.
    - Implementierung von Timeouts für Tool-Aufrufe.
    - Protokollierung von Tool-Nutzung für Audits.

### 5.2. Resources: Bereitstellung von Kontextdaten

Resources dienen dazu, Daten und Inhalte für LLMs als Kontext bereitzustellen.2 Im Gegensatz zu Tools, die modellgesteuert sind, ist die Verwendung von Resources typischerweise anwendungs- oder benutzergesteuert.23 Das bedeutet, die Host-Anwendung oder der Benutzer entscheidet, welche Ressourcen dem LLM zur Verfügung gestellt werden.

#### 5.2.1. Definition, URI-Schemata und Inhaltstypen (Text, Binär)

Eine Ressourcendefinition umfasst 23:

- **`uri: string`**: Ein eindeutiger Uniform Resource Identifier, der die Ressource adressiert. MCP unterstützt gängige URI-Schemata wie `file:///` für lokale Dateien oder `https://` für Webinhalte, erlaubt aber auch Servern, eigene benutzerdefinierte Schemata zu definieren (z.B. `postgres://`, `screen://`).14
- **`name: string`**: Ein menschenlesbarer Name für die Ressource.
- **`description?: string`**: Eine optionale Beschreibung.
- **`mimeType?: string`**: Der optionale MIME-Typ der Ressource (z.B. `text/plain`, `application/pdf`, `image/png`).

Ressourcen können zwei Arten von Inhalten haben 14:

- **Textressourcen**: Enthalten UTF-8-kodierten Text (z.B. Quellcode, Konfigurationsdateien, Logdateien).
- **Binärressourcen**: Enthalten Rohdaten, die Base64-kodiert übertragen werden (z.B. Bilder, PDFs, Audiodateien).

**JSON Beispiel für eine Ressourcendefinition (in einer `resources/list`-Antwort, abgeleitet von 23):**

JSON

```
{
  "uri": "file:///home/user/report.pdf",
  "name": "Project Report",
  "description": "Q3 Project Status Report",
  "mimeType": "application/pdf"
}
```

#### 5.2.2. Entdeckung (`resources/list`, Resource Templates) und Lesen (`resources/read`)

- **`resources/list`**: Clients verwenden diese Methode, um eine Liste der direkt vom Server bereitgestellten, konkreten Ressourcen zu erhalten.23
- **Resource Templates**: Für dynamisch generierte oder parametrisierte Ressourcen können Server URI-Vorlagen bereitstellen (z.B. `logs://{date}` oder `file:///logs/{filename}`).14 Clients können diese Vorlagen verwenden, um spezifische Ressourcen-URIs zu konstruieren.
- **`resources/read`**: Mit dieser Methode fordert ein Client den Inhalt einer oder mehrerer Ressourcen anhand ihrer URIs an.14 Ein Server kann auf eine einzelne `resources/read`-Anfrage mit den Inhalten mehrerer Ressourcen antworten, z.B. wenn die Anfrage-URI auf ein Verzeichnis zeigt und der Server die Inhalte der darin enthaltenen Dateien zurückgibt.23

**JSON Beispiel für eine `resources/read`-Antwort (abgeleitet von 23):**

JSON

```
{
  "jsonrpc": "2.0",
  "id": "read789",
  "result": {
    "contents":
  }
}
```

#### 5.2.3. Echtzeit-Updates: Abonnements (`resources/subscribe`, `notifications/resources/updated`) und Listenänderungen (`notifications/resources/list_changed`)

MCP unterstützt dynamische Aktualisierungen von Ressourcen 14:

- **`notifications/resources/list_changed`**: Der Server kann diese Benachrichtigung senden, um Clients darüber zu informieren, dass sich die Liste der verfügbaren Ressourcen geändert hat.
- **`resources/subscribe`**: Ein Client kann diese Methode verwenden, um Änderungen am Inhalt einer spezifischen Ressource zu abonnieren.
- **`notifications/resources/updated`**: Wenn eine abonnierte Ressource sich ändert, sendet der Server diese Benachrichtigung an den Client. Der Client kann dann mit `resources/read` den neuesten Inhalt abrufen.
- **`resources/unsubscribe`**: Ein Client verwendet diese Methode, um ein Abonnement für eine Ressource zu beenden.

Die Unterstützung für Abonnements (`subscribe`) und Benachrichtigungen über Listenänderungen (`listChanged`) wird während der Initialisierungsphase über die Server-Fähigkeiten ausgehandelt.24 Dieses Abonnementmodell ermöglicht es LLMs, mit dynamischen, sich in Echtzeit ändernden Kontexten zu arbeiten, was für Anwendungen, die aktuelle Informationen benötigen, von großer Bedeutung ist. Die Implementierung von Ressourcenabonnements erfordert jedoch auf Serverseite eine sorgfältige Verwaltung des Zustands der Abonnenten und der Ressourcen, um zeitnahe und korrekte Benachrichtigungen sicherzustellen.

### 5.3. Prompts: Strukturierung von KI-Interaktionen

Prompts im MCP sind wiederverwendbare Vorlagen und Arbeitsabläufe, die dazu dienen, Interaktionen mit LLMs zu standardisieren und zu vereinfachen.2 Sie sind typischerweise benutzergesteuert, d.h. der Benutzer wählt oft explizit einen Prompt aus, z.B. über UI-Elemente wie Slash-Befehle in einem Chat.14

#### 5.3.1. Definition, dynamische Argumente und Nachrichtenstruktur

Eine Prompt-Definition umfasst 14:

- **`name: string`**: Ein eindeutiger Bezeichner für den Prompt.
- **`description?: string`**: Eine menschenlesbare Beschreibung des Prompts.
- **`arguments?: array`**: Eine optionale Liste von Argumenten, die der Prompt akzeptiert. Jedes Argumentobjekt kann Felder wie `name`, `description`, `required` (boolean) und optional ein Schema zur Validierung enthalten.

Wenn ein Prompt abgerufen wird (`prompts/get`), liefert der Server eine Sequenz von Nachrichten, die an das LLM gesendet werden sollen. Jede Nachricht in dieser Sequenz hat 28:

- **`role: string`**: Entweder `"user"` oder `"assistant"`, um den Sprecher anzugeben.
- **`content: object`**: Der Inhalt der Nachricht, der verschiedene Typen annehmen kann:
    - **Text Content**: `{ "type": "text", "text": "..." }`
    - **Image Content**: `{ "type": "image", "data": "BASE64_ENCODED_IMAGE_DATA", "mimeType": "image/png" }` (muss Base64-kodiert sein und einen gültigen MIME-Typ haben)
    - **Embedded Resources**: `{ "type": "resource", "resource": { "uri": "...", "mimeType": "...", "text": "..." / "blob": "..." } }` (ermöglicht das direkte Einbetten von Server-verwalteten Ressourceninhalten)

**JSON Beispiel für eine Prompt-Definition (in einer `prompts/list`-Antwort, abgeleitet von 30):**

JSON

```
{
  "name": "analyze-code",
  "description": "Analyze code for potential improvements",
  "arguments":
}
```

#### 5.3.2. Entdeckung (`prompts/list`) und Abruf (`prompts/get`)

- **`prompts/list`**: Clients verwenden diese Methode, um eine Liste der vom Server angebotenen Prompts zu erhalten.14
- **`prompts/get`**: Mit dieser Methode ruft ein Client einen spezifischen Prompt ab. Dabei können Argumente übergeben werden, um den Prompt zu personalisieren oder mit spezifischen Daten zu füllen.14 Die Serverantwort enthält die resultierenden Nachrichten für das LLM.

Die Fähigkeit des Servers, über Änderungen in der Prompt-Liste zu informieren (`listChanged`), wird ebenfalls während der Initialisierung ausgehandelt.28

**JSON Beispiel für eine `prompts/get`-Antwort (abgeleitet von 28):**

JSON

```
{
  "jsonrpc": "2.0",
  "id": "getPrompt456",
  "result": {
    "description": "Analyze Python code for potential improvements",
    "messages":
  }
}
```

#### 5.3.3. Einbetten von Ressourcenkontext in Prompts

Prompts können Kontext aus Ressourcen einbetten, indem sie entweder direkt Ressourceninhalte in die Nachrichtenstruktur aufnehmen (wie im `Embedded Resources`-Typ oben gezeigt) oder indem sie auf Ressourcen-URIs verweisen, die der Client dann separat laden könnte.14 Dies ermöglicht es, LLM-Interaktionen mit spezifischen, aktuellen Informationen zu grundieren, die von MCP-Servern verwaltet werden, und fördert so reichhaltige, kontextualisierte Dialoge. Prompts dienen somit als Mechanismus zur Kapselung gängiger Interaktionsmuster, was die Konsistenz und Wiederverwendbarkeit fördert und die Benutzererfahrung durch klare, geführte Abläufe verbessert.14

## 6. Absicherung von MCP: Sicherheits- und Autorisierungsframework

Die Mächtigkeit des Model-Context-Protocol, das den Zugriff auf beliebige Daten und die Ausführung von Code ermöglicht, erfordert ein robustes Sicherheits- und Autorisierungsframework. Alle Implementierer müssen diese Aspekte sorgfältig berücksichtigen.8

### 6.1. Fundamentale Sicherheitsprinzipien: Benutzerzustimmung, Datenschutz, Werkzeugsicherheit

MCP basiert auf mehreren Kernprinzipien, um Vertrauen und Sicherheit zu gewährleisten 1:

- **Benutzerzustimmung und -kontrolle (User Consent and Control):** Benutzer müssen explizit allen Datenzugriffen und Operationen zustimmen und deren Umfang verstehen. Sie müssen die Kontrolle darüber behalten, welche Daten geteilt und welche Aktionen ausgeführt werden. Implementierungen sollten klare Benutzeroberflächen für die Überprüfung und Autorisierung von Aktivitäten bereitstellen.8 Der Host spielt hierbei eine zentrale Rolle bei der Verwaltung dieser Zustimmungsprozesse.1
- **Datenschutz (Data Privacy):** Hosts müssen die explizite Zustimmung des Benutzers einholen, bevor Benutzerdaten an Server weitergegeben werden. Benutzerdaten dürfen nicht ohne Zustimmung an anderer Stelle übertragen werden und sollten durch angemessene Zugriffskontrollen geschützt werden.8
- **Werkzeugsicherheit (Tool Safety):** Tools repräsentieren die Ausführung von beliebigem Code und müssen mit entsprechender Vorsicht behandelt werden. Beschreibungen des Tool-Verhaltens (z.B. Annotationen) sollten als nicht vertrauenswürdig betrachtet werden, es sei denn, sie stammen von einem vertrauenswürdigen Server. Hosts müssen die explizite Zustimmung des Benutzers einholen, bevor ein Tool aufgerufen wird, und Benutzer sollten verstehen, was jedes Tool tut, bevor sie dessen Verwendung autorisieren.8 Klare visuelle Indikatoren bei der Tool-Ausführung sind empfehlenswert.26 Das Prinzip des "Menschen im Kontrollkreis" (human in the loop) ist hierbei zentral.2
- **Kontrollen für LLM-Sampling (LLM Sampling Controls):** Benutzer müssen explizit allen LLM-Sampling-Anfragen zustimmen und kontrollieren können, ob Sampling überhaupt stattfindet, welcher Prompt gesendet wird und welche Ergebnisse der Server sehen kann. Das Protokoll schränkt die Sichtbarkeit des Servers auf Prompts absichtlich ein.8

Obwohl MCP diese Prinzipien nicht immer auf Protokollebene erzwingen kann, sollten Implementierer robuste Zustimmungs- und Autorisierungsflüsse in ihre Anwendungen integrieren und Sicherheitsbest Practices befolgen.8 Die Verantwortung für die korrekte Implementierung dieser Mechanismen liegt maßgeblich bei der Host-Anwendung.

### 6.2. Autorisierungsstrategien: OAuth 2.1 mit PKCE

Mit der zunehmenden Verbreitung von MCP, insbesondere im Kontext von entfernten Servern, wurde ein standardisierter Autorisierungsmechanismus notwendig. MCP hat OAuth 2.1 als Standard für die Autorisierung übernommen, insbesondere für Verbindungen zu Servern, die nicht lokal und vertrauenswürdig sind.31 Dies ist in der Protokollrevision `2025-03-26` formalisiert.31

Ein Schlüsselelement ist die **verbindliche Nutzung von PKCE (Proof Key for Code Exchange)** für öffentliche Clients (wie Desktop-Anwendungen oder CLI-Tools).31 PKCE schützt vor dem Abfangen des Autorisierungscodes, einem kritischen Angriffsszenario bei OAuth-Flüssen mit öffentlichen Clients.33 Die Integration von OAuth 2.1 spiegelt die Reifung des Protokolls und die Notwendigkeit wider, Interaktionen mit potenziell von Dritten betriebenen MCP-Servern abzusichern.

### 6.3. Integration mit Identity Providern (IdPs)

Die ursprüngliche MCP-Autorisierungsspezifikation legte nahe, dass der MCP-Server sowohl als Ressourcenserver als auch als Autorisierungsserver fungieren könnte, was eine erhebliche Implementierungskomplexität für Server-Entwickler darstellt.33 Ein Request For Comments (RFC) und die Community-Diskussion zielen darauf ab, diesen Ansatz zu verbessern.31

Die empfohlene Vorgehensweise ist nun, dass MCP-Server als **OAuth 2.1 Ressourcenserver** agieren und sich für die Ausstellung von Zugriffstokens auf **etablierte Identity Provider (IdPs)** verlassen.31 Dies hat mehrere Vorteile:

- Entwickler von MCP-Servern müssen keine OAuth-Experten sein oder komplexe Autorisierungsserver von Grund auf neu erstellen.31
- Es fördert die Standardisierung um gängige OAuth-Muster.
- Es sorgt für eine klare Trennung der Zuständigkeiten: Der IdP ist für die Authentifizierung und Token-Ausstellung zuständig, der MCP-Server für die Validierung der Tokens und die Durchsetzung von Berechtigungen.

Ein Beispiel für einen solchen externen IdP ist Stytch, das OAuth-Flüsse, Client-Registrierung und Token-Ausstellung übernehmen kann.32 MCP-Clients würden Benutzer zum IdP umleiten, um Tokens zu erhalten, die dann zur Authentifizierung gegenüber dem MCP-Server verwendet werden.33

### 6.4. Definition und Verwaltung von Scopes für granulare Zugriffskontrolle

Scopes (Berechtigungsbereiche) sind ein integraler Bestandteil von OAuth und spielen eine wichtige Rolle bei der Definition granularer Zugriffsberechtigungen im MCP.12 Sie bestimmen, welche Tools, Ressourcen oder spezifischen Operationen ein Client (und damit das LLM) im Namen des Benutzers ausführen darf.

Ein Beispiel ist der Merge MCP-Server, der Scopes im Format `<Kategorie>.<CommonModelName>:<Berechtigung>` verwendet, z.B. `ats.Candidate:read` für Lesezugriff auf Kandidatenobjekte im Bewerbermanagementsystem (ATS) oder `hris.Employee:write` für Schreibzugriff auf Mitarbeiterobjekte im HRIS.12

Wichtige Aspekte bei der Verwendung von Scopes:

- **Validierung:** MCP-Server müssen die vom Client angeforderten Scopes gegen die für das verknüpfte Konto oder den Benutzer tatsächlich aktivierten Berechtigungen validieren. Nur Tools und Ressourcen, die den gültigen und autorisierten Scopes entsprechen, werden aktiviert.12
- **Fehlerbehandlung:** Clients müssen darauf vorbereitet sein, dass angeforderte Scopes möglicherweise nicht gewährt werden (z.B. aufgrund von Kategorie- oder Berechtigungs-Nichtübereinstimmungen) und entsprechende Fehlermeldungen oder alternative Pfade implementieren.12

Scopes ermöglichen die Umsetzung des Prinzips der geringsten Rechte (Principle of Least Privilege), indem sichergestellt wird, dass Clients nur auf die Daten und Funktionen zugreifen, für die sie explizit autorisiert wurden. Dies ist besonders wichtig beim Umgang mit potenziell sensiblen Daten in Unternehmenssystemen.

### 6.5. Best Practices für sichere Client- und Server-Implementierungen

Zusätzlich zu den spezifischen Autorisierungsmechanismen sollten Entwickler von MCP-Clients und -Servern allgemeine Sicherheitsbest Practices befolgen 14:

- **Eingabevalidierung und -bereinigung:** Alle von Clients empfangenen Eingaben (z.B. Tool-Parameter, Ressourcen-URIs) müssen serverseitig rigoros validiert und bereinigt werden, um Injection-Angriffe und andere Sicherheitslücken zu verhindern.
- **Sichere Transporte:** Bei Netzwerktransporten wie HTTP/SSE ist die Verwendung von TLS zur Verschlüsselung der Datenübertragung unerlässlich.
- **Verschlüsselung sensibler Daten:** Sensible Daten sollten sowohl bei der Übertragung als auch im Ruhezustand (at rest) verschlüsselt werden.
- **Validierung der Nachrichtenintegrität:** Mechanismen zur Sicherstellung, dass Nachrichten während der Übertragung nicht manipuliert wurden.
- **Begrenzung der Nachrichtengröße:** Implementierung von Limits für die Größe von Nachrichten, um Denial-of-Service-Angriffe durch übergroße Nachrichten zu verhindern.
- **Vorsicht bei Binärdaten:** Sorgfältige Handhabung von Binärdaten, um Pufferüberläufe oder andere damit verbundene Schwachstellen zu vermeiden.

Durch die Kombination dieser fundamentalen Sicherheitsprinzipien, der standardisierten OAuth 2.1-Autorisierung und allgemeiner Best Practices strebt MCP danach, ein sicheres und vertrauenswürdiges Ökosystem für die Erweiterung von KI-Fähigkeiten zu schaffen.

## 7. Integration von MCP in Linux Desktop Widgets: Ein praktischer Leitfaden

Die Integration des Model-Context-Protocol (MCP) in Linux Desktop-Widgets eröffnet spannende Möglichkeiten, um diese kleinen, fokussierten Anwendungen intelligenter, kontextbewusster und stärker vernetzt zu gestalten. Dieser Abschnitt untersucht, wie MCP in gängige Linux-Widget-Technologien eingebettet werden kann.

### 7.1. Überblick über Linux Desktop-Widget-Technologien

Verschiedene Frameworks eignen sich für die Entwicklung von Desktop-Widgets unter Linux. Die Wahl hängt oft von der Ziel-Desktop-Umgebung, den bevorzugten Programmiersprachen und den spezifischen Anforderungen des Widgets ab.

#### 7.1.1. GTK (Gtk3/Gtk4) mit C/Python

GTK (GIMP Toolkit) ist ein weit verbreitetes, plattformübergreifendes Widget-Toolkit, das die Grundlage für die GNOME-Desktop-Umgebung bildet, aber auch in anderen Umgebungen eingesetzt wird.34 Es bietet einen umfassenden Satz an UI-Elementen und ist für Projekte jeder Größenordnung geeignet.35 GTK ist in C geschrieben, verfügt aber über stabile Bindungen zu vielen anderen Sprachen, darunter C++, Python, JavaScript und Rust, was die Integration von MCP-SDKs (insbesondere Python und JavaScript) erleichtert.35 GTK ist Open Source unter der LGPL lizenziert.35

#### 7.1.2. Qt/QML mit C++/Python

Qt ist ein leistungsstarkes, plattformübergreifendes Anwendungsframework, das häufig für die Entwicklung grafischer Benutzeroberflächen verwendet wird.36 Es bietet die Qt Widgets für traditionelle UIs und QML, eine deklarative Sprache, für moderne, flüssige Benutzeroberflächen.36 Qt wird mit dem Qt Creator, einer umfangreichen IDE, geliefert und unterstützt primär C++, bietet aber auch exzellente Python-Bindungen (PyQt oder PySide).36 Dies macht es ebenfalls zu einem guten Kandidaten für die Integration von MCP-SDKs.

#### 7.1.3. KDE Plasma Widgets (Plasmoids)

Plasma Widgets, auch Plasmoids genannt, sind speziell für die KDE Plasma Desktop-Umgebung konzipiert.38 Sie ermöglichen eine tiefe Integration in den Desktop und können vielfältige Funktionen bereitstellen, von einfachen Anzeigen (z.B. Wörterbuch, Ordneransicht 38) bis hin zu komplexeren Interaktionen. Die Entwicklung von Plasmoids erfolgt häufig mit QML und JavaScript, was eine direkte Nutzung des JavaScript/TypeScript MCP SDKs ermöglicht.39 Entwickler können bestehende Widgets als Vorlage nutzen und anpassen.39

#### 7.1.4. GNOME Shell Extensions

GNOME Shell Extensions erweitern die Funktionalität der GNOME Shell und werden typischerweise in JavaScript unter Verwendung von GJS (GNOME JavaScript Bindings) und Clutter für die UI-Darstellung geschrieben.40 Sie können UI-Elemente zur oberen Leiste hinzufügen, das Verhalten des Aktivitäten-Overviews ändern oder neue Dialoge und Popups erstellen.40 Die JavaScript-Basis macht sie zu einem natürlichen Kandidaten für die Integration des TypeScript/JavaScript MCP SDK.

Die folgende Tabelle vergleicht diese Technologien im Hinblick auf eine MCP-Integration:

|   |   |   |   |   |   |
|---|---|---|---|---|---|
|**Technologie**|**Primäre Sprache(n)**|**UI-Paradigma**|**Eignung für MCP SDK-Integration (Python/JS Fokus)**|**Sandboxing/Sicherheit (typisch)**|**Darstellung reichhaltiger Inhalte (z.B. HTML/CSS)**|
|GTK (Gtk3/Gtk4)|C, Python, JS, Rust|Imperativ|Sehr gut (Python, JS)|Anwendungsabhängig|WebKitGTK für HTML/CSS, Pango für Rich Text|
|Qt/QML|C++, Python|Imperativ (Widgets), Deklarativ (QML)|Sehr gut (Python, JS in QML)|Anwendungsabhängig|QtWebEngine für HTML/CSS, Rich Text in Widgets|
|KDE Plasma Widgets|QML/JS, C++|Deklarativ/Imperativ|Exzellent (JS in QML)|Plasma-spezifisch|QtWebEngine über QML|
|GNOME Shell Ext.|JavaScript (GJS)|Imperativ (Clutter)|Exzellent (JS)|GNOME Shell-spezifisch|Begrenzt (St.Label mit Pango Markup), keine direkte Webview-Einbettung im Panel|

Die meisten dieser Technologien bieten robuste Entwicklungsumgebungen und unterstützen Sprachen, für die MCP SDKs existieren oder leicht angebunden werden können. Die Wahl wird oft von der gewünschten Integrationstiefe in die Desktop-Umgebung und der Komplexität der darzustellenden MCP-Informationen beeinflusst.

### 7.2. Architekturelle Überlegungen für MCP-fähige Widgets

Bei der Entwicklung eines MCP-fähigen Desktop-Widgets muss dessen Rolle innerhalb der MCP-Architektur klar definiert werden.

#### 7.2.1. Widget als MCP Host vs. Client innerhalb eines größeren Hosts

Es gibt zwei Hauptmuster:

1. **Das Widget als MCP Host:** Das Desktop-Widget agiert selbstständig als MCP Host-Anwendung.1 Es initialisiert und verwaltet seine eigenen MCP Client-Instanzen, um sich mit einem oder mehreren MCP Servern zu verbinden (z.B. ein Wetter-Widget, das sich mit einem Wetter-MCP-Server verbindet). Dieses Modell ist in sich geschlossen und gibt dem Widget volle Kontrolle über seine MCP-Interaktionen.
2. **Das Widget als reiner UI-Client für einen größeren Host:** Das Widget ist Teil einer umfassenderen Desktop-Anwendung oder eines Dienstes (z.B. vergleichbar mit PiecesOS 11 oder Claude Desktop 43), der als zentraler MCP Host für den Benutzer fungiert. In diesem Szenario ist das Widget primär für die Darstellung von Daten oder die Bereitstellung von UI-Elementen zuständig, die vom übergeordneten Host orchestriert werden. Das Widget selbst würde dann keine direkten MCP-Client-Verbindungen zu externen Servern aufbauen, sondern mit dem lokalen, zentralen Host kommunizieren (möglicherweise über proprietäre IPC oder eine vereinfachte Schnittstelle). Dieses Modell kann die Komplexität des einzelnen Widgets reduzieren und eine zentralisierte Verwaltung von MCP-Verbindungen und Benutzerberechtigungen ermöglichen.

Die Entscheidung zwischen diesen Mustern beeinflusst die Komplexität, die Verantwortlichkeiten und das Ressourcenmanagement des Widgets.

#### 7.2.2. Interprozesskommunikation (IPC), falls das Widget ein separater Prozess ist

Wenn das Widget als eigenständige Anwendung läuft (z.B. eine separate GTK- oder Qt-Anwendung) und mit einem zentralen MCP-Host-Prozess (z.B. einem Hintergrunddienst, der MCP-Verbindungen für den Benutzer verwaltet) kommunizieren muss, sind Mechanismen zur Interprozesskommunikation (IPC) erforderlich. Unter Linux kommen hierfür häufig D-Bus oder Sockets in Frage. Dieses Szenario ist relevant, wenn eine zentralisierte Verwaltung von MCP-Kontext und -Sicherheit über mehrere Widgets oder Anwendungen hinweg gewünscht wird.

### 7.3. Strategien zur Darstellung dynamischer UI-Inhalte von MCP-Servern

Ein Kernaspekt MCP-fähiger Widgets ist die dynamische Darstellung von Informationen, die von MCP-Servern stammen. Dies kann von einfachem Text bis hin zu komplexen, interaktiven UI-Elementen reichen.

#### 7.3.1. Serverseitig gerenderte UI-Schnipsel (HTML/CSS via MCP)

Ein vielversprechendes Muster, demonstriert durch das `mcp-widgets`-Projekt 44, besteht darin, dass der MCP-Server direkt HTML/CSS-Schnipsel als Teil seiner Antwort liefert. Das Widget auf dem Desktop, das eine Web-Rendering-Engine einbetten kann, ist dann lediglich für die Darstellung dieses HTML/CSS zuständig.

- **Vorteile:** Die UI-Logik und das Rendering-Know-how können auf dem Server liegen, was das Widget selbst vereinfacht. Änderungen am UI-Aussehen können serverseitig erfolgen, ohne das Widget neu kompilieren oder verteilen zu müssen.
- **Nachteile:** Weniger Flexibilität für tiefgreifende native Integrationen oder die Nutzung nativer Widget-Funktionen. Erfordert, dass der Server UI-Komponenten generiert.

#### 7.3.2. Clientseitiges Rendering unter Verwendung von Daten aus MCP (Native Widgets oder eingebettete Webansichten)

Alternativ empfängt das Widget strukturierte Daten (typischerweise JSON) vom MCP-Server und ist selbst für das Rendering der Benutzeroberfläche verantwortlich. Dies kann durch native UI-Elemente des gewählten Widget-Frameworks oder durch dynamische Generierung von HTML/CSS für eine eingebettete Webansicht geschehen.

##### 7.3.2.1. Einbetten von HTML/CSS in GTK: `WebKitWebView`

GTK-Anwendungen können `WebKitWebView` (oder `WebView` in neueren GTK-Versionen, die WebKitGTK verwenden) nutzen, um Webinhalte darzustellen.45 Dies ist ideal, um von MCP-Servern gelieferte HTML/CSS-Schnipsel anzuzeigen oder um auf Basis von MCP-Daten dynamisch HTML zu generieren.

- `webkit_web_view_load_html(webview, html_string, base_uri)`: Lädt einen HTML-String direkt.48 Der `base_uri` ist wichtig für die Auflösung relativer Pfade (z.B. für Bilder, CSS-Dateien innerhalb des HTML).
- `webkit_web_view_load_uri(webview, uri)`: Lädt Inhalte von einer URL.
- Sicherheitsaspekte beim Laden lokaler Dateien über `file:///`-URIs müssen beachtet werden.48

##### 7.3.2.2. Einbetten von HTML/CSS in Qt/QML: `QWebEngineView`

Qt bietet `QWebEngineView` für die Integration von Webinhalten in Qt Widgets und QML-Anwendungen.50

- `loadHtml(html_string, base_url)`: Methode des `WebEngineView` QML-Typs (oder der C++ Klasse) zum Laden eines HTML-Strings.54
- `setUrl(url)`: Lädt Inhalte von einer URL.
- **Kommunikation zwischen QML/C++ und der Webseite:** Qt WebChannel (`webChannel`-Eigenschaft in QML) ermöglicht eine bidirektionale Kommunikation zwischen dem QML/C++ Code und JavaScript innerhalb der geladenen Webseite.50 Dies kann nützlich sein, um Interaktionen innerhalb des HTML-Widgets zurück an die native Widget-Logik zu leiten.

##### 7.3.2.3. Natives Styling und Rich Text

Für weniger komplexe Darstellungen oder wenn eine Webview nicht gewünscht ist:

- **GTK CSS:** GTK-Widgets können mit CSS-ähnlichen Regeln gestaltet werden, was eine flexible Anpassung des Erscheinungsbilds nativer Widgets ermöglicht.57
- **Pango Markup (GTK/GNOME Shell):** Für Rich-Text-Darstellungen in GTK-Labels (und `St.Label` in GNOME Shell Extensions, das intern Pango verwendet) kann Pango Markup genutzt werden. Dies ist eine XML-ähnliche Syntax, um Textformatierungen wie Fett, Kursiv, Farben und Schriftarten direkt im Textstring zu definieren [60 (Qt-Kontext, aber Pango ist ähnlich), 59].
    - Beispiel Pango Markup: `<span foreground="blue" size="x-large">Blauer Text</span> ist <i>cool</i>!`.59
- **Qt Rich Text:** Qt-Widgets wie `QLabel` unterstützen eine Untermenge von HTML 4 für Rich-Text-Formatierungen.60

Die `mcp-widgets`-Strategie 44, bei der Server HTML/CSS liefern, ist für Desktop-Widgets besonders attraktiv, da sowohl GTK als auch Qt ausgereifte Webview-Komponenten bieten. Dies kann die Logik im Widget-Client erheblich vereinfachen. Die Wahl zwischen serverseitig gerenderter UI und clientseitigem Rendering basierend auf MCP-Daten ist jedoch ein Kompromiss: Serverseitiges Rendering vereinfacht die Client-Logik, ist aber möglicherweise weniger flexibel für eine tiefe native Integration; clientseitiges Rendering bietet mehr Kontrolle, erfordert aber mehr UI-Code im Widget.

### 7.4. Implementierung der MCP-Client-Logik in Widgets

Die Kernfunktionalität eines MCP-fähigen Widgets ist seine Fähigkeit, als MCP-Client zu agieren (oder mit einem übergeordneten Host zu kommunizieren, der als Client agiert).

#### 7.4.1. Nutzung offizieller MCP SDKs (Python, C++ über Bindings oder direktes JSON-RPC)

Die Model Context Protocol Organisation stellt offizielle SDKs für verschiedene Sprachen zur Verfügung, die die Implementierung von MCP-Clients und -Servern erheblich vereinfachen.61

- **Python SDK:** (]) Weit verbreitet und gut geeignet für die Entwicklung mit GTK (über PyGObject) und Qt (über PyQt/PySide).3 Das OpenAI Agents SDK enthält ebenfalls Unterstützung für MCP-Interaktionen mit Python.3
- **TypeScript/JavaScript SDK:** (`@modelcontextprotocol/sdk` 21) Ideal für GNOME Shell Extensions (GJS) und QML-basierte Plasma Widgets, die JavaScript als Skriptsprache verwenden.61
- **C# SDK:** (61) Könnte relevant sein, wenn.NET/Mono für die Widget-Entwicklung unter Linux verwendet wird.
- **Java und Kotlin SDKs:** (61) Weniger typisch für Linux Desktop-Widgets, aber vorhanden.
- **Rust SDK:** (61) Eine Option für performance-kritische Komponenten oder wenn Rust bevorzugt wird.
- **C++:** Zum Zeitpunkt der Recherche ist kein offizielles, breit hervorgehobenes C++ SDK so prominent wie die Python- oder JS-SDKs. Entwickler, die C++ für GTK oder Qt verwenden, müssten möglicherweise:
    1. Eine generische JSON-RPC-Bibliothek für C++ verwenden und die MCP-spezifischen Nachrichten und den Sitzungslebenszyklus manuell implementieren.
    2. Auf ein offizielles C++ SDK warten oder dazu beitragen.
    3. Wrapper um das C-API eines potenziellen zukünftigen C-SDKs erstellen.

Die Verfügbarkeit von Python- und JavaScript-SDKs passt gut zu den gängigen Skriptsprachen in der Linux-Desktop-Widget-Entwicklung. Für C++-basierte Widgets stellt dies eine größere Herausforderung dar, die entweder durch Eigenimplementierung des Protokolls oder durch Nutzung von Bindings zu anderen SDKs (falls möglich und performant) gelöst werden muss.

### 7.5. Beispielintegration 1: "Smart Clipboard"-Widget (GTK/Python mit Textverarbeitungs-MCP-Server)

Dieses Beispiel skizziert ein GTK-Widget, das den Inhalt der Zwischenablage überwacht und bei Bedarf eine Analyse über einen MCP-Server anbietet.

#### 7.5.1. Konzeptuelles Design und UI-Mockup

- **UI:** Ein einfaches GTK-Fenster oder Panel-Applet.
    - Ein mehrzeiliges Textfeld (`GtkTextView`), das den aktuellen Inhalt der Zwischenablage anzeigt (optional).
    - Ein Button "Zwischenablage analysieren (MCP)".
    - Ein Bereich zur Anzeige der Analyseergebnisse (z.B. als formatierter Text oder in strukturierten `GtkLabel`s).
- **Funktionalität:**
    1. Das Widget überwacht Änderungen in der Systemzwischenablage.
    2. Wenn neuer Textinhalt erkannt wird, wird der Button "Analysieren" aktiv.
    3. Bei Klick auf den Button:
        - Der Widget-Client verbindet sich mit einem (hypothetischen) `text_analyzer_mcp_server`.
        - Der Inhalt der Zwischenablage wird an ein Tool dieses Servers gesendet.
        - Das Ergebnis (z.B. Sentiment, Entitätenextraktion, Zusammenfassung) wird im Widget angezeigt.

#### 7.5.2. MCP-Client-Implementierung in Python (mit GTK)

Python

```
import gi
gi.require_version('Gtk', '4.0') # Oder '3.0'
from gi.repository import Gtk, Gdk, GLib
# Annahme: Das Python MCP SDK ist installiert und importierbar
# from modelcontextprotocol import MCPServerStdio, MCPServerSse # Beispielhafte Importe

# Hypothetischer MCP Server (lokal via stdio)
TEXT_ANALYZER_SERVER_COMMAND = ["python", "path/to/text_analyzer_mcp_server.py"]

class SmartClipboardWidget(Gtk.ApplicationWindow):
    def __init__(self, app):
        super().__init__(application=app, title="Smart Clipboard (MCP)")
        self.set_default_size(400, 300)

        self.clipboard = Gdk.Display.get_default().get_primary_clipboard()
        self.clipboard.connect("notify::text", self.on_clipboard_changed)

        self.vbox = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        self.set_child(self.vbox)

        self.clipboard_display = Gtk.Label(label="Zwischenablage wird überwacht...")
        self.vbox.append(self.clipboard_display)

        self.analyze_button = Gtk.Button(label="Zwischenablage analysieren (MCP)")
        self.analyze_button.connect("clicked", self.on_analyze_clicked)
        self.analyze_button.set_sensitive(False)
        self.vbox.append(self.analyze_button)

        self.result_display = Gtk.Label(label="Analyseergebnis hier.")
        self.result_display.set_wrap(True)
        self.vbox.append(self.result_display)

        self.mcp_server_instance = None # Für die MCP-Server-Verbindung

    def on_clipboard_changed(self, clipboard, _props):
        text = clipboard.read_text_async(None, self._clipboard_read_callback)

    def _clipboard_read_callback(self, clipboard, result):
        text = clipboard.read_text_finish(result)
        if text:
            self.clipboard_display.set_text(f"Aktuell: {text[:50]}...")
            self.analyze_button.set_sensitive(True)
            self.current_clipboard_text = text
        else:
            self.analyze_button.set_sensitive(False)

    async def initialize_mcp_client(self):
        # Dieser Teil ist pseudocodeartig, da das genaue SDK-API variieren kann
        # Beispiel für stdio server
        # self.mcp_server_instance = MCPServerStdio(params={
        # "command": TEXT_ANALYZER_SERVER_COMMAND,
        # "args": TEXT_ANALYZER_SERVER_COMMAND[1:],
        # })
        # await self.mcp_server_instance.connect() # Annahme einer connect Methode
        # tools = await self.mcp_server_instance.list_tools()
        # if any(tool.name == "analyze_clipboard_content" for tool in tools):
        #     return True
        # return False
        print("MCP Client Initialisierung (Platzhalter)")
        return True # Simuliere Erfolg

    async def call_mcp_tool(self, tool_name, arguments):
        # if self.mcp_server_instance:
        #     try:
        #         result = await self.mcp_server_instance.call_tool(name=tool_name, arguments=arguments)
        #         return result
        #     except Exception as e:
        #         return {"isError": True, "content": [{"type": "text", "text": f"MCP Fehler: {e}"}]}
        print(f"MCP Tool Aufruf (Platzhalter): {tool_name} mit {arguments}")
        # Simuliere eine Antwort
        return {"isError": False, "content": [{"type": "text", "text": f"Analyse für '{arguments.get('text', '')[:20]}...': Positiv."}]}


    def on_analyze_clicked(self, _widget):
        if not hasattr(self, "current_clipboard_text") or not self.current_clipboard_text:
            self.result_display.set_text("Kein Text in der Zwischenablage.")
            return

        self.result_display.set_text("Analysiere...")

        async def analyze_task():
            if not self.mcp_server_instance: # Oder eine bessere Zustandsprüfung
                initialized = await self.initialize_mcp_client()
                if not initialized:
                    self.result_display.set_text("MCP Server nicht initialisierbar.")
                    return

            tool_result = await self.call_mcp_tool(
                tool_name="analyze_clipboard_content",
                arguments={"text": self.current_clipboard_text}
            )

            if tool_result.get("isError"):
                error_message = tool_result.get("content", [{"type": "text", "text": "Unbekannter Fehler"}]).get("text")
                self.result_display.set_markup(f"<span foreground='red'>Fehler: {GLib.markup_escape_text(error_message)}</span>")
            else:
                # Annahme: Ergebnis ist Text
                analysis = tool_result.get("content",).get("text", "Kein Ergebnis.")
                # Pango Markup für Formatierung verwenden [59]
                self.result_display.set_markup(f"<b>Analyse:</b>\n{GLib.markup_escape_text(analysis)}")

        # Ausführung der asynchronen Aufgabe in GTK
        GLib.idle_add(lambda: GLib.ensure_future(analyze_task()) and False)


class SmartClipboardApp(Gtk.Application):
    def __init__(self):
        super().__init__(application_id="org.example.smartclipboardmcp")

    def do_activate(self):
        win = SmartClipboardWidget(self)
        win.present()

# app = SmartClipboardApp()
# exit_status = app.run() # Deaktiviert für diesen Bericht, da es eine laufende Anwendung erfordert
```

_Hinweis: Der obige Python-Code ist konzeptionell und verwendet Platzhalter für die eigentliche MCP SDK-Interaktion, da die genauen API-Aufrufe vom spezifischen Python MCP SDK abhängen. Die GTK-Teile sind jedoch funktional._

#### 7.5.3. Interaktion mit einem hypothetischen Textanalyse-MCP-Server (Tool: `analyze_clipboard_content`)

- **Tool-Definition auf dem Server (konzeptionell):**
    - Name: `analyze_clipboard_content`
    - `inputSchema`: `{ "type": "object", "properties": { "text": { "type": "string" } }, "required": ["text"] }`
    - Funktionalität: Nimmt einen Textstring entgegen, führt NLP-Aufgaben durch (z.B. Sentimentanalyse, Entitätserkennung, Schlüsselworterkennung, kurze Zusammenfassung).
    - `result`: `{ "content": }`
- **Widget-Interaktion:**
    1. Der Client im Widget sendet eine `tools/call`-Anfrage an den `text_analyzer_mcp_server` mit der Methode `analyze_clipboard_content` und dem Zwischenablageninhalt als `text`-Parameter.
    2. Der Server verarbeitet den Text und gibt das strukturierte Ergebnis zurück.
    3. Das GTK-Widget parst die Antwort und zeigt die Analyseergebnisse an. Pango Markup 59 kann verwendet werden, um die Ergebnisse formatiert in einem `GtkLabel` oder `GtkTextView` darzustellen (z.B. verschiedene Farben für Sentiment, fette Überschriften für Entitäten).

### 7.6. Beispielintegration 2: "Kontextueller Aufgabenhelfer"-Widget (Qt/QML/C++ mit Kalender- & Dateisystem-MCP-Servern)

Dieses Beispiel beschreibt ein Widget, das kontextbezogene Informationen aus Kalender- und Dateisystemdaten aggregiert, um den Benutzer bei seinen aktuellen Aufgaben zu unterstützen.

#### 7.6.1. Konzeptuelles Design und UI-Mockup

- **UI (QML):**
    - Eine Liste oder Ansicht (`ListView`, `GridView`) für bevorstehende Kalenderereignisse für den aktuellen Tag.
    - Ein Bereich, der relevante Dateien oder Dokumente anzeigt, die mit den aktuellen Kalenderereignissen oder kürzlich bearbeiteten Projekten in Verbindung stehen.
    - Möglicherweise eine Suchfunktion, um innerhalb des kontextuellen Aufgabenbereichs zu suchen.
    - Wenn die MCP-Server HTML/CSS-Snippets zurückgeben (inspiriert von `mcp-widgets` 44), könnte ein `WebEngineView` 54 zur Darstellung verwendet werden.
- **Funktionalität:**
    1. Das Widget verbindet sich beim Start mit einem `calendar_mcp_server` und einem `filesystem_mcp_server`.
    2. Es ruft Kalenderereignisse für den aktuellen Tag/die nahe Zukunft ab.
    3. Basierend auf den Ereignissen (z.B. Projektnamen, Teilnehmer) oder kürzlichen Aktivitäten fragt es den `filesystem_mcp_server` nach relevanten Dateien.
    4. Die aggregierten Informationen werden dem Benutzer übersichtlich präsentiert.

#### 7.6.2. MCP-Client-Implementierung in C++ (mit Qt/QML)

- **Architektur:** Die C++-Backend-Logik des Widgets würde als MCP Host agieren und zwei MCP Client-Instanzen verwalten, eine für jeden Server.
- **Verbindungsaufbau:** Code zum Herstellen von Verbindungen zu `calendar_mcp_server` und `filesystem_mcp_server` (z.B. über `stdio` für lokale Server oder HTTP/SSE für entfernte). Dies würde die Implementierung des JSON-RPC-Austauschs und des MCP-Lebenszyklus erfordern, wenn kein C++ SDK verwendet wird.
- **Datenmodell in C++:** Klassen zur Repräsentation von Kalenderereignissen und Dateiinformationen, die von den MCP-Servern empfangen werden.
- **Exposition gegenüber QML:** Die C++-Logik würde die verarbeiteten Daten und Funktionen über das Qt-Eigenschaftssystem und invokable Methoden für die QML-Frontend-Schicht verfügbar machen.

C++

```
// Konzeptioneller C++ Code-Ausschnitt (stark vereinfacht)
// Annahme: Eine JSON-RPC Bibliothek und manuelle MCP-Implementierung oder ein C++ MCP SDK existiert.

// #include <QObject>
// #include <QJsonObject>
// #include <QJsonArray>
// #include <QQmlApplicationEngine>
// #include <QtWebEngineQuick/QtWebEngineQuick> // Für QtWebEngine::initialize() in main

// class McpClientWrapper : public QObject { /*... */ }; // Wrapper für MCP-Client-Logik

class TaskHelperBackend : public QObject {
    Q_OBJECT
    // Q_PROPERTY(QVariantList calendarEvents READ calendarEvents NOTIFY calendarEventsChanged)
    // Q_PROPERTY(QVariantList relevantFiles READ relevantFiles NOTIFY relevantFilesChanged)

public:
    explicit TaskHelperBackend(QObject *parent = nullptr) : QObject(parent) {
        // m_calendarClient = new McpClientWrapper("calendar_mcp_server_config");
        // m_filesystemClient = new McpClientWrapper("filesystem_mcp_server_config");
        // connect_mcp_servers_and_fetch_initial_data();
    }

// Q_INVOKABLE void refreshData() { /*... */ }

// private:
    // McpClientWrapper* m_calendarClient;
    // McpClientWrapper* m_filesystemClient;
    // QVariantList m_calendarEvents;
    // QVariantList m_relevantFiles;

    // void connect_mcp_servers_and_fetch_initial_data() {
        // Placeholder: Hier würde die Logik zum Verbinden und Abrufen von Daten stehen
        // z.B. m_calendarClient->callMethod("resources/read", {"uri": "calendar://today/events"},...);
        // z.B. m_filesystemClient->callMethod("resources/read", {"uri": "file:///projects/current?relevant=true"},...);
    // }

// signals:
    // void calendarEventsChanged();
    // void relevantFilesChanged();
};

// In main.cpp:
// QtWebEngineQuick::initialize(); // Wenn WebEngineView verwendet wird [56]
// QQmlApplicationEngine engine;
// qmlRegisterType<TaskHelperBackend>("com.example.taskhelper", 1, 0, "TaskHelperBackend");
// engine.load(QUrl(QStringLiteral("qrc:/main.qml")));
```

#### 7.6.3. Aggregation von Kontext aus Kalender- (`calendar/events`) und Dateisystem- (`file:///relevant_project_docs`) MCP-Servern

- **Kalender-Server:**
    - Das Widget (bzw. dessen C++ Backend) sendet eine `resources/read`-Anfrage an den `calendar_mcp_server` für eine Ressource wie `calendar://today/events` oder `calendar://project_alpha/next_meeting`.
    - Der Server antwortet mit einer Liste von Ereignisobjekten (z.B. Titel, Zeit, Ort, Teilnehmer).
- **Dateisystem-Server:**
    - Basierend auf Schlüsselwörtern aus den Kalenderereignissen (z.B. Projektname) oder einer Liste kürzlich verwendeter Projekte sendet das Widget `resources/read`-Anfragen an den `filesystem_mcp_server`. Beispiele für Ressourcen-URIs: `file:///projects/alpha/docs?recent=5` oder `search:///keywords=MCP,Widget&limit=10`.
    - Der Server antwortet mit einer Liste von Datei- oder Verzeichnisinformationen (Name, Pfad, Typ, Änderungsdatum).
- **Aggregation und Korrelation:**
    - Die C++-Logik im Widget aggregiert diese Daten.
    - Es könnte eine einfache Korrelation implementiert werden, z.B. Dateien anzeigen, die um die Zeit eines Kalenderereignisses herum geändert wurden oder deren Pfad Projektnamen aus Kalendereinträgen enthält.
- **Darstellung in QML:**
    - Die aggregierten und korrelierten Daten werden über das Qt-Eigenschaftssystem an die QML-Schicht übergeben.
    - QML-Elemente (`ListView`, `Repeater` etc.) rendern die Informationen. Wenn der Server HTML/CSS-Snippets liefert (z.B. eine schön formatierte Kalenderansicht), kann ein `WebEngineView` 54 in QML diese direkt anzeigen. Die `loadHtml()`-Methode des `WebEngineView` 54 wäre hierfür geeignet.

Diese Beispiele illustrieren, wie MCP-Widgets spezifische Probleme lösen können, indem sie die standardisierte Schnittstelle des MCP nutzen, um auf vielfältige Datenquellen und Werkzeuge zuzugreifen und diese intelligent zu kombinieren.

## 8. Entwicklungsplan: Erstellung MCP-gestützter Linux Desktop-Widgets

Dieser Entwicklungsplan skizziert einen strukturierten Ansatz zur Erstellung von Linux Desktop-Widgets, die das Model-Context-Protocol (MCP) nutzen. Der Plan ist in Phasen unterteilt, um eine systematische Entwicklung, Integration und Bereitstellung zu gewährleisten.

### 8.1. Phase 1: Fundament, Forschung und Prototyping

Diese initiale Phase legt den Grundstein für das gesamte Projekt.

#### 8.1.1. Detaillierte Anforderungserhebung & Anwendungsfalldefinition

- **Aktivität:** Klare Definition der spezifischen Funktionalität des/der Widgets. Wer ist die Zielgruppe? Welchen Mehrwert bietet die MCP-Integration (z.B. Zugriff auf welche Daten/Tools, welche Art von KI-Unterstützung)?
- **Entscheidung:** Identifikation der benötigten MCP-Server. Sind diese bereits vorhanden (z.B. offizielle oder Community-Server 9) oder müssen sie im Rahmen des Projekts neu entwickelt werden? Welche spezifischen Tools, Resources oder Prompts dieser Server werden benötigt?
- **Ergebnis:** Ein detailliertes Anforderungsdokument und klar definierte Anwendungsfälle.

#### 8.1.2. Auswahl des Technologie-Stacks

Basierend auf den Anforderungen und den Ergebnissen aus Abschnitt 7.1 werden hier kritische Entscheidungen getroffen:

- **Entscheidung (Widget-Framework):** Wahl des Desktop-Widget-Frameworks.
    - **Optionen:** GTK, Qt/QML, KDE Plasma, GNOME Shell Extensions.
    - **Kriterien:** Gewünschte Integrationstiefe in die Desktop-Umgebung (z.B. Plasma für KDE), vorhandene Teamkompetenzen, Komplexität der geplanten UI, Portabilitätsanforderungen.
    - **Fakt:** Für eine tiefe Integration in KDE Plasma wären Plasma Widgets (QML/JS) ideal.39 Für GNOME eignen sich GNOME Shell Extensions (JS).40 GTK und Qt sind universeller.
    - **Entscheidung für diesen Plan:** **Qt/QML** wird als primäres Framework gewählt, da es eine gute Balance zwischen nativer Performance (C++ Backend), flexibler UI-Gestaltung (QML mit JavaScript) und plattformübergreifenden Möglichkeiten bietet. Es ermöglicht auch die einfache Einbettung von Web-Inhalten über `QWebEngineView`.50
- **Entscheidung (Primäre Programmiersprache):**
    - **Optionen:** Python, C++, JavaScript.
    - **Kriterien:** Performance-Anforderungen, Verfügbarkeit von MCP SDKs, Teamkompetenzen, Kompatibilität mit dem gewählten Widget-Framework.
    - **Fakt:** Qt/QML unterstützt C++ für das Backend und JavaScript in QML.36 Python-Bindungen (PySide/PyQt) sind ebenfalls exzellent.
    - **Entscheidung für diesen Plan:** **C++** für die Kernlogik und MCP-Client-Implementierung (falls kein C++ SDK direkt nutzbar ist, dann Implementierung des JSON-RPC-Protokolls) und **QML/JavaScript** für die UI. Dies ermöglicht hohe Performance und volle Qt-Integration.
- **Entscheidung (MCP SDK / Implementierung):**
    - **Optionen:** Nutzung eines offiziellen MCP SDK (TypeScript/JS für QML-Teil, Python mit Bindings, oder direkte C++ Implementierung).
    - **Kriterien:** Reifegrad des SDKs, Sprachpräferenz, Performance.
    - **Fakt:** Es gibt offizielle TypeScript/JS und Python SDKs.61 Ein C++ SDK ist weniger prominent.
    - **Entscheidung für diesen Plan:** Das **TypeScript/JavaScript SDK** wird für Interaktionen innerhalb der QML-Schicht evaluiert. Für das C++ Backend wird zunächst die **direkte Implementierung der MCP JSON-RPC-Kommunikation** unter Verwendung einer robusten C++ JSON-Bibliothek in Betracht gezogen, falls kein adäquates C++ SDK verfügbar ist oder die Overhead-Kosten eines Bindings zu hoch sind. Die Python SDK-Option wird als Alternative für schnellere Prototypenentwicklung beibehalten.
- **Entscheidung (Ziel-MCP-Server):**
    - **Kriterien:** Verfügbarkeit, Stabilität, bereitgestellte Fähigkeiten.
    - **Entscheidung für diesen Plan:** Für die Prototyping-Phase wird zunächst der offizielle **Filesystem MCP Server** 3 und ein einfacher, selbst entwickelter **Echo- oder Test-MCP-Server** verwendet, um die Client-Implementierung zu validieren.

#### 8.1.3. Initiale MCP-Client-Implementierung

- **Aktivität:** Entwicklung einer grundlegenden MCP-Client-Logik im gewählten Technologie-Stack (C++).
- **Schritte:** Implementierung des Verbindungsaufbaus (z.B. `stdio` oder HTTP/SSE, je nach Testserver), Senden der `initialize`-Anfrage, Verarbeiten der Server-Antwort, Aushandeln der Fähigkeiten und Senden der `initialized`-Benachrichtigung.19
- **Ergebnis:** Eine Codebibliothek, die eine grundlegende MCP-Sitzung aufbauen kann.

#### 8.1.4. Proof-of-Concept (PoC)

- **Aktivität:** Erstellung eines minimalen Qt/QML-Widgets mit einer rudimentären Benutzeroberfläche.
- **Schritte:** Das Widget soll eine einfache MCP-Interaktion durchführen, z.B. die `tools/list`-Methode eines Test-MCP-Servers aufrufen und die Namen der zurückgegebenen Tools in einem QML-Textfeld anzeigen.
- **Ergebnis:** Ein funktionierender Prototyp, der die technische Machbarkeit der MCP-Integration im gewählten Stack demonstriert.

### 8.2. Phase 2: Kernfunktionsentwicklung und MCP-Integration

In dieser Phase werden die Hauptfunktionen des Widgets entwickelt und die MCP-Integration vertieft.

#### 8.2.1. Implementierung der Widget-UI/UX für MCP-Interaktionen

- **Aktivität:** Entwurf und Implementierung der QML-Benutzeroberfläche.
- **Aspekte:** UI-Elemente zur Entdeckung und Auswahl von Ressourcen, zum Aufrufen von Tools (inklusive klarer Zustimmungsdialoge für den Benutzer 8), zur Anzeige von Prompts und zur Darstellung der von MCP-Servern gelieferten Ergebnisse.
- **Technologie:** Nutzung von QML für die UI-Struktur und JavaScript für die UI-Logik. Für die Darstellung von HTML/CSS-Inhalten von MCP-Servern wird `QWebEngineView` 54 verwendet. Für native Darstellungen werden Standard-QML-Elemente gestylt.

#### 8.2.2. Robuste Integration mit ausgewählten MCP-Servern

- **Aktivität:** Implementierung der vollständigen Interaktionslogik mit den Ziel-MCP-Servern (gemäß Phase 1).
- **Schritte:** Verarbeitung aller benötigten Tools, Resources und Prompts. Handhabung verschiedener Datentypen, Parameter und Antwortstrukturen. Implementierung einer umfassenden Fehlerbehandlung für die MCP-Kommunikation (basierend auf JSON-RPC-Fehlercodes und anwendungsspezifischen Fehlern 26).
- **Ergebnis:** Stabile und zuverlässige Kommunikation mit den MCP-Servern.

#### 8.2.3. Implementierung von Sicherheits- und Autorisierungsflüssen

- **Aktivität:** Absicherung der MCP-Interaktionen.
- **Schritte:**
    - Wenn entfernte oder gesicherte MCP-Server verwendet werden: Integration der OAuth 2.1 Client-Logik (Authorization Code Flow mit PKCE 31). Anforderung notwendiger Scopes.12 Sichere Speicherung und Handhabung von Tokens.
    - Implementierung klarer Benutzer-Zustimmungsmechanismen im UI für den Zugriff auf Ressourcen und die Ausführung von Tools, wie von den MCP-Sicherheitsprinzipien gefordert.8
- **Ergebnis:** Sichere Authentifizierung und Autorisierung sowie Einhaltung der MCP-Sicherheitsrichtlinien.

#### 8.2.4. Zustandsmanagement innerhalb des Widgets

- **Aktivität:** Verwaltung des internen Zustands des Widgets in Bezug auf MCP-Daten.
- **Aspekte:** Zwischenspeicherung von Ressourcenlisten oder Tool-Definitionen (unter Berücksichtigung von `list_changed`-Benachrichtigungen 23), Verfolgung laufender Tool-Operationen (für Abbruch oder Fortschrittsanzeige), Speicherung von Benutzereinstellungen für MCP-Interaktionen.
- **Technologie:** Nutzung von C++ Datenstrukturen und Qt-Signalen/Slots für die Aktualisierung der QML-UI.

### 8.3. Phase 3: Erweiterte Funktionen, Tests und Verfeinerung

Diese Phase konzentriert sich auf fortgeschrittene MCP-Funktionen, Qualitätssicherung und Optimierung.

#### 8.3.1. Implementierung erweiterter MCP-Funktionen (optional)

- **Aktivität:** Falls für die Widget-Funktionalität erforderlich, Implementierung von:
    - Ressourcenabonnements (`resources/subscribe`, `notifications/resources/updated`) für Echtzeit-Datenaktualisierungen.23
    - Verarbeitung komplexer, mehrstufiger Prompts.14
    - Clientseitige Anfragen für Sampling-Operationen (falls vom Host unterstützt und relevant).8

#### 8.3.2. Umfassende Tests

- **Aktivität:** Sicherstellung der Qualität und Stabilität des Widgets.
- **Methoden:**
    - **Unit-Tests:** Für die C++ MCP-Client-Logik und QML/JS UI-Komponenten (z.B. mit Qt Test).
    - **Integrationstests:** Mit realen oder gemockten MCP-Servern, um das Zusammenspiel zu testen.
    - **UI/UX-Tests:** Überprüfung der Benutzerfreundlichkeit, Klarheit der MCP-Interaktionen und der Zustimmungsdialoge.
    - **Sicherheitsaudit:** Insbesondere der Autorisierungsflüsse und der Handhabung sensibler Daten.
    - **Nutzung des MCP Inspector:** Ein Tool zur visuellen Prüfung und zum Debugging von Interaktionen mit MCP-Servern.61
- **Ergebnis:** Ein gut getestetes, stabiles Widget.

#### 8.3.3. Performance-Profiling und -Optimierung

- **Aktivität:** Identifizierung und Behebung von Leistungsengpässen.
- **Bereiche:** MCP-Kommunikationslatenz, Datenverarbeitung (JSON-Parsing, -Serialisierung), UI-Rendering in QML (insbesondere bei `QWebEngineView`). Optimierung der CPU- und Speichernutzung.
- **Ergebnis:** Ein performantes und ressourcenschonendes Widget.

#### 8.3.4. Benutzerakzeptanztests (UAT) und iterative Verfeinerung

- **Aktivität:** Einholung von Feedback von Zielbenutzern.
- **Schritte:** Durchführung von UATs, Sammlung von Feedback zu Funktionalität, Benutzerfreundlichkeit und dem Nutzen der MCP-Integration. Iterative Anpassungen basierend auf dem Feedback.
- **Ergebnis:** Ein benutzerorientiertes Widget, das den Bedürfnissen der Zielgruppe entspricht.

### 8.4. Phase 4: Paketierung, Bereitstellung und Wartung

Die letzte Phase befasst sich mit der Verteilung und dem langfristigen Support des Widgets.

#### 8.4.1. Paketierung für Linux-Distributionen

- **Aktivität:** Erstellung von Installationspaketen.
- **Optionen:** Flatpak, Snap, traditionelle Pakete (.deb,.rpm).
- **Überlegungen:** Abhängigkeiten (Qt-Versionen, WebEngine), Desktop-Integration (z.B. `.desktop`-Dateien für den Anwendungsstarter, Icons, ggf. Integration in Plasma- oder GNOME-spezifische Widget-Systeme, falls nicht direkt als solches entwickelt).
- **Ergebnis:** Einfach installierbare Pakete für Endbenutzer.

#### 8.4.2. Dokumentation

- **Aktivität:** Erstellung notwendiger Dokumentationen.
- **Typen:**
    - **Endbenutzer-Dokumentation:** Anleitung zur Installation, Konfiguration und Nutzung des Widgets und seiner MCP-Funktionen.
    - **Entwickler-Dokumentation:** Falls das Widget erweiterbar ist oder als Teil eines größeren Systems dient (z.B. API-Beschreibungen, Architekturübersicht).
- **Ergebnis:** Umfassende Dokumentation für verschiedene Zielgruppen.

#### 8.4.3. Etablierung einer Wartungs- und Update-Strategie

- **Aktivität:** Planung für den langfristigen Support.
- **Aspekte:** Mechanismen zur Meldung und Behebung von Fehlern. Umgang mit Sicherheitslücken. Anpassung an zukünftige Änderungen der MCP-Spezifikationen oder der APIs der genutzten MCP-Server. Regelmäßige Updates.
- **Ergebnis:** Ein Plan für die nachhaltige Pflege des Widgets.

### 8.5. Zusammenfassung der wichtigsten Entscheidungen, Meilensteine und Ressourcenüberlegungen

- **Wichtige Entscheidungen (Zusammenfassung):**
    - Widget-Framework: **Qt/QML**.
    - Programmiersprachen: **C++ (Backend), QML/JS (Frontend)**.
    - MCP-Implementierung: **Direkte JSON-RPC-Implementierung in C++** (primär), Evaluierung des JS SDK für QML.
    - Fehlerberichterstattung an Benutzer: Klare, verständliche Meldungen, die zwischen Protokoll- und Anwendungsfehlern unterscheiden.
    - Daten-Caching: Implementierung einer Caching-Strategie für `tools/list` und `resources/list` Ergebnisse, mit Invalidierung durch `list_changed` Benachrichtigungen.
- **Meilensteine (Beispiele):**
    - M1: PoC für MCP-Grundverbindung und UI-Darstellung abgeschlossen.
    - M2: Kern-MCP-Integration mit Zielservern (Tools, Resources, Prompts) funktionsfähig.
    - M3: Sicherheits- und Autorisierungsfunktionen implementiert und getestet.
    - M4: Umfassende Tests (Unit, Integration, UI) bestanden; Performance-Optimierung abgeschlossen.
    - M5: Beta-Version für UAT freigegeben.
    - M6: Finale Version paketiert und dokumentiert.
- **Ressourcenallokation (Überlegungen):**
    - **Entwicklungszeit:** Abhängig von der Komplexität des Widgets und der Anzahl der zu integrierenden MCP-Server. Die Phasenstruktur hilft bei der Schätzung.
    - **Benötigte Fähigkeiten:** Expertise in Qt/QML und C++; Verständnis von Netzwerkprotokollen (JSON-RPC, HTTP, SSE); Kenntnisse in Sicherheitskonzepten (OAuth 2.1); UI/UX-Design-Fähigkeiten; Testautomatisierung.
    - **Testaufwand:** Signifikanter Aufwand für alle Testebenen, insbesondere Integrationstests mit verschiedenen MCP-Servern und Sicherheitstests.

Dieser Entwicklungsplan berücksichtigt die Notwendigkeit einer frühen Technologieauswahl, da diese weitreichende Auswirkungen auf den Entwicklungsaufwand, die Performance und die Wartbarkeit hat. Die Verwendung von Qt/QML mit einem C++ Backend bietet eine solide Basis für leistungsstarke und ansprechende Desktop-Widgets, während die Flexibilität bei der MCP-SDK-Wahl bzw. -Implementierung eine Anpassung an die spezifischen Projektanforderungen ermöglicht.

## 9. Einhaltung von MCP-Standards und Best Practices

Die erfolgreiche und interoperable Implementierung von MCP-fähigen Desktop-Widgets hängt entscheidend von der strikten Einhaltung der offiziellen MCP-Spezifikationen und etablierter Best Practices ab.

### 9.1. Konformität mit MCP-Spezifikationsversionen

MCP ist ein sich entwickelnder Standard.1 Es ist unerlässlich, dass Entwicklungen gegen eine spezifische, stabile Version der MCP-Spezifikation erfolgen (z.B. die Version `2025-03-26`, die in mehreren offiziellen Dokumenten referenziert wird 8). Entwickler sollten die offizielle Dokumentation auf [modelcontextprotocol.io](https://modelcontextprotocol.io/) 8 und das Spezifikations-Repository auf GitHub 61 regelmäßig auf Updates, neue Versionen und Migrationsleitfäden prüfen. Ein Plan für den Umgang mit zukünftigen Protokollrevisionen sollte Teil der Wartungsstrategie sein, um die langfristige Kompatibilität und Funktionalität der Widgets sicherzustellen.

### 9.2. Empfohlene Praktiken für Client- und Server-Entwicklung

Obwohl dieser Bericht sich auf die Client-Seite (Widgets) konzentriert, ist das Verständnis serverseitiger Best Practices hilfreich. Die offiziellen MCP-Entwicklungsleitfäden (z.B. der MCP Server Development Guide 14) und die Dokumentationen der SDKs (z.B. für TypeScript 21) enthalten wertvolle Empfehlungen:

- **Klare Benennung und detaillierte Schemata:** Tools, Resources und Prompts sollten aussagekräftige Namen und Beschreibungen haben. Ihre `inputSchema` (für Tools und Prompt-Argumente) und Datenstrukturen sollten präzise mit JSON Schema definiert werden.14
- **Korrekte Fehlerbehandlung:** Implementierung einer robusten Fehlerbehandlung, die sowohl Protokollfehler als auch anwendungsspezifische Fehler abdeckt und klare Fehlermeldungen liefert.14
- **Sicherheit:** Strikte Einhaltung der MCP-Sicherheitsprinzipien (Benutzerzustimmung, Datenschutz, Werkzeugsicherheit) und Implementierung der Autorisierungsmechanismen wie OAuth 2.1.8
- **Zustandsmanagement:** Sorgfältige Verwaltung des Sitzungszustands, insbesondere bei Servern, die Abonnements oder langlebige Operationen unterstützen.8
- **Performance:** Effiziente Implementierungen, um Latenzen gering zu halten und Ressourcen zu schonen.

### 9.3. Einbindung in die MCP-Community und Nutzung von Ressourcen

Als offener Standard lebt MCP von seiner Community.61 Entwickler von MCP-fähigen Widgets sollten die offiziellen Ressourcen aktiv nutzen:

- **GitHub Repositories:** Die `modelcontextprotocol` Organisation auf GitHub hostet die Spezifikation, SDKs, Beispielserver und andere wichtige Werkzeuge wie den MCP Inspector.13 Diese sind primäre Quellen für Code, Dokumentation und zur Verfolgung der Entwicklung.
- **Offizielle Dokumentation:** Die Website [modelcontextprotocol.io](https://modelcontextprotocol.io/) dient als zentraler Anlaufpunkt für Einführungen, Anleitungen, die Spezifikation und Neuigkeiten.8
- **Community-Kanäle:** (Falls vorhanden, z.B. Diskussionsforen, Mailinglisten, Chat-Kanäle) Aktive Teilnahme kann helfen, Probleme zu lösen, Feedback zu geben und über neue Entwicklungen informiert zu bleiben.

Die Behandlung der offiziellen Spezifikation 8 als maßgebliche Quelle und die Nutzung der bereitgestellten SDKs 61 sind entscheidend, um Konformität sicherzustellen, Implementierungsfehler zu reduzieren und die Interoperabilität mit anderen Komponenten im MCP-Ökosystem zu gewährleisten.

## 10. Schlussfolgerung und zukünftige Entwicklung von MCP in Desktop-Umgebungen

Das Model-Context-Protocol (MCP) besitzt das transformative Potenzial, die Art und Weise, wie Linux Desktop-Widgets und -Anwendungen mit KI-Systemen und externen Datenquellen interagieren, grundlegend zu verändern. Durch die Bereitstellung eines standardisierten, sicheren und erweiterbaren Frameworks ermöglicht MCP die Entwicklung von Widgets, die nicht nur Informationen passiv anzeigen, sondern aktiv Kontext verstehen, intelligente Aktionen vorschlagen oder ausführen und nahtlos mit einem breiten Ökosystem von KI-Werkzeugen und -Diensten zusammenarbeiten können.

Die in diesem Bericht detaillierte Spezifikation – von der Client-Host-Server-Architektur über die JSON-RPC-basierte Kommunikation und die Kernprimitive (Tools, Resources, Prompts) bis hin zum robusten Sicherheits- und Autorisierungsframework – bildet eine solide Grundlage für Entwickler. Die klare Definition von Verantwortlichkeiten, der Fokus auf Benutzerkontrolle und -zustimmung sowie die Betonung der Komponierbarkeit und einfachen Servererstellung sind Schlüsselfaktoren, die die Adaption von MCP fördern dürften.

Für Linux Desktop-Widgets bedeutet dies konkret:

- **Erhöhte Intelligenz:** Widgets können auf kontextuelle Informationen zugreifen (z.B. Kalender, lokale Dateien, Anwendungszustände), die über MCP-Server bereitgestellt werden, um relevantere und proaktivere Unterstützung zu bieten.
- **Erweiterte Funktionalität:** Durch die Anbindung an MCP-Tools können Widgets komplexe Aufgaben delegieren (z.B. Datenanalyse, API-Interaktionen, Code-Generierung), die weit über ihre traditionellen Fähigkeiten hinausgehen.
- **Verbesserte Benutzererfahrung:** Standardisierte Interaktionsmuster (Prompts) und die Möglichkeit, reichhaltige, dynamische UIs (ggf. serverseitig gerendert) darzustellen, können zu intuitiveren und ansprechenderen Widgets führen.
- **Nahtlose Integration:** MCP kann die Grenzen zwischen lokalen Desktop-Anwendungen und Cloud-basierten KI-Diensten verwischen und so eine hybride Computing-Erfahrung schaffen, bei der KI-Fähigkeiten allgegenwärtig und leicht zugänglich sind.

Die zukünftige Entwicklung und der Erfolg von MCP im Desktop-Bereich werden von mehreren Faktoren abhängen:

1. **Wachstum des MCP-Server-Ökosystems:** Die Verfügbarkeit einer breiten Palette nützlicher und stabiler MCP-Server für verschiedenste Anwendungsfälle (von Produktivitätswerkzeugen bis hin zu spezialisierten Branchenlösungen) ist entscheidend.9
2. **Einfachheit der Client-Implementierung:** Die Qualität und Benutzerfreundlichkeit der MCP SDKs für gängige Desktop-Entwicklungssprachen (insbesondere C++, Python, JavaScript) wird die Bereitschaft der Entwickler beeinflussen, MCP zu adoptieren.
3. **Demonstration konkreter Mehrwerte:** Es bedarf überzeugender Anwendungsbeispiele und Widgets, die den Benutzern klare Vorteile durch die MCP-Integration bieten.
4. **Weiterentwicklung des Standards:** Das MCP-Konsortium muss den Standard kontinuierlich pflegen, auf Feedback aus der Community reagieren und ihn an neue Anforderungen und technologische Entwicklungen im KI-Bereich anpassen, beispielsweise hinsichtlich neuer Modalitäten oder komplexerer Agentenarchitekturen.
5. **Sicherheitsvertrauen:** Die konsequente Umsetzung und Weiterentwicklung der Sicherheits- und Autorisierungsmechanismen ist unerlässlich, um das Vertrauen der Benutzer und Entwickler in die Plattform zu gewinnen und zu erhalten.

Der vorgestellte Entwicklungsplan für MCP-gestützte Linux Desktop-Widgets unter Verwendung von Qt/QML und C++ bietet einen pragmatischen Weg, um die Potenziale von MCP zu erschließen. Die sorgfältige Auswahl des Technologie-Stacks, die phasenweise Entwicklung und die strikte Einhaltung der MCP-Standards sind dabei erfolgskritisch.

Zusammenfassend lässt sich sagen, dass das Model-Context-Protocol gut positioniert ist, um eine Schlüsselrolle in der nächsten Generation intelligenter Desktop-Anwendungen zu spielen. Es bietet die notwendige Standardisierung und Flexibilität, um die wachsende Leistungsfähigkeit von LLMs sicher und effektiv in die täglichen Arbeitsabläufe der Benutzer zu integrieren. Die Reise hat gerade erst begonnen, aber die Richtung ist vielversprechend.

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


# Granulare Technische Implementierungsspezifikation (Pflichtenheft)

Dieses Dokument beschreibt die detaillierten technischen Spezifikationen für die Implementierung des Projekts. Es konzentriert sich auf technische Details, die für Entwickler relevant sind, einschließlich spezifischer Bibliotheken, Methoden und Protokolle.

## 1. Kernarchitektur und Setup

### 1.1. Programmiersprache und Laufzeitumgebung

Die primäre Programmiersprache für dieses Projekt ist Rust. Rust wird aufgrund seiner Betonung auf Sicherheit (insbesondere Speichersicherheit), Leistung und Konkurrenzfähigkeit ausgewählt.1 Die asynchrone Natur vieler Aufgaben (UI-Events, D-Bus-Kommunikation, Netzwerk-I/O, LLM-Interaktionen) erfordert eine robuste asynchrone Laufzeitumgebung.

### 1.2. Build-System

Das Standard-Build-System und Paketmanagement-Tool für Rust, Cargo, wird für die Verwaltung von Abhängigkeiten, das Kompilieren des Projekts und die Ausführung von Tests verwendet.

### 1.3. Asynchrone Laufzeitumgebung

Tokio wird als asynchrone Laufzeitumgebung eingesetzt.1 Tokio bietet eine leistungsstarke, multi-threaded Laufzeitumgebung, die für I/O-gebundene Anwendungen optimiert ist und eine umfangreiche Sammlung von asynchronen APIs und ein breites Ökosystem an kompatiblen Bibliotheken bereitstellt. Die Haupt-Event-Schleife der Anwendung (sofern nicht durch spezifische UI-Frameworks wie Smithay/Calloop vorgegeben, siehe Abschnitt 4) wird mit Tokio implementiert. Asynchrone Funktionen werden mittels `async fn` deklariert und mit `.await` aufgerufen. Der Einstiegspunkt der Anwendung wird mit dem `#[tokio::main]` Makro versehen.

### 1.4. Fehlerbehandlung

Ein robustes und typisiertes Fehlerbehandlungsmodell ist entscheidend. Das Crate `thiserror` wird verwendet, um benutzerdefinierte Fehlertypen zu definieren.2

- **Zentraler Fehlertyp:** Eine zentrale `enum AppError` wird im Haupt-Crate definiert, die alle möglichen Fehlerquellen der Anwendung aggregiert.
- **Modulspezifische Fehler:** Jedes Modul, das potenziell Fehler erzeugen kann (z.B. D-Bus-Interaktion, MCP-Client, Datenbankzugriff), definiert seine eigene `enum` für spezifische Fehler, ebenfalls unter Verwendung von `#[derive(thiserror::Error)]`.
- **Fehlerkonvertierung:** Das Attribut `#[from]` wird in der zentralen `AppError` verwendet, um die automatische Konvertierung von modulspezifischen Fehlern in Varianten des zentralen Fehlertyps zu ermöglichen.2 Dies vermeidet Boilerplate-Code für die Fehlerkonvertierung.
- **Rückgabetypen:** Funktionen, die fehlschlagen können, geben `Result<T, AppError>` (oder einen modulspezifischen Fehlertyp, der dann konvertiert wird) zurück. Dies erzwingt eine explizite Fehlerbehandlung an der Aufrufstelle.

Diese Strategie, die sich an der Verwendung von `std::io::Error` in der Standardbibliothek orientiert 2, bietet einen Kompromiss zwischen Granularität (spezifische Fehler pro Modul) und Benutzerfreundlichkeit (einheitlicher Fehlertyp auf höherer Ebene), ohne die Aufrufer mit unerreichbaren Fehlerfällen zu belasten.

## 2. Textverarbeitung und Bereinigung

### 2.1. HTML-Bereinigung

Jeglicher nicht vertrauenswürdiger HTML-Inhalt, insbesondere von LLM-Ausgaben oder externen Webquellen, muss vor der Darstellung bereinigt werden, um Cross-Site-Scripting (XSS) und andere Angriffe zu verhindern.3

- **Bibliothek:** Das Crate `ammonia` wird für die HTML-Bereinigung verwendet.3 `ammonia` basiert auf einer Whitelist und nutzt `html5ever` für das Parsen, was es robust gegen Verschleierungstechniken macht.5
- **Konfiguration:** Die Bereinigung wird über das `ammonia::Builder` Pattern konfiguriert.5
    - **Erlaubte Tags:** Eine strikte Whitelist von erlaubten HTML-Tags (z.B. `p`, `b`, `i`, `ul`, `ol`, `li`, `br`, `a`, `img`, `code`, `pre`) wird mittels `builder.tags()` definiert.5 Potenziell gefährliche Tags wie `<script>`, `<style>`, `<iframe`> sind standardmäßig verboten und dürfen nicht hinzugefügt werden.
    - **Erlaubte Attribute:** Eine strikte Whitelist von erlaubten Attributen pro Tag wird mittels `builder.attributes()` definiert.5 Event-Handler-Attribute (`onerror`, `onload` etc.) und `style`-Attribute sollten generell vermieden oder stark eingeschränkt werden. Globale Attribute wie `lang` können über `("*", vec!["lang"])` erlaubt werden.
    - **Link-Attribute:** Für `<a>`-Tags muss das `rel`-Attribut mittels `builder.link_rel()` konfiguriert werden, um mindestens `noopener`, `noreferrer` und `nofollow` für externe Links zu erzwingen.5 URL-Schemata für `href`-Attribute sollten auf `http`, `https` und `mailto` beschränkt werden.
- **Anwendung:** Die Methode `builder.clean(dirty_html)` wird aufgerufen, um den Bereinigungsprozess durchzuführen.5 Der `Builder` sollte einmal konfiguriert und für mehrere Bereinigungsoperationen wiederverwendet werden.

### 2.2. Kommandozeilenargument-Maskierung

Bei der Interaktion mit externen Prozessen (siehe Abschnitt 10) ist die korrekte Behandlung von Kommandozeilenargumenten entscheidend, um Command-Injection-Schwachstellen zu verhindern.6

- **Bevorzugte Methode:** Die sicherste Methode ist die Verwendung von `std::process::Command` ohne Einbeziehung einer Shell. Das Kommando und jedes Argument werden separat über `.arg()` oder `.args()` übergeben.8 Dies verhindert, dass die Shell spezielle Zeichen im Argument interpretiert.
    
    Rust
    
    ```
    use std::process::Command;
    let user_input = "some potentially unsafe string; rm -rf /";
    let output = Command::new("plocate")
       .arg("--basename") // Example argument
       .arg(user_input) // Argument passed directly, not interpreted by shell
       .output()?;
    ```
    
- **Alternative (Nur wenn unvermeidbar):** Wenn Argumente dynamisch zu einem String zusammengesetzt werden müssen, der von einer Shell (`sh -c`) interpretiert wird (stark abgeraten), muss jedes Argument rigoros maskiert werden.
    
    - **Bibliothek:** Das Crate `shlex` wird verwendet.
    - **Funktion:** Die Funktion `shlex::quote(argument_string)` wird für jedes einzelne Argument aufgerufen, bevor es in den Befehlsstring eingefügt wird.7
    
    Rust
    
    ```
    // Strongly discouraged approach
    use std::process::Command;
    use shlex::Shlex;
    let user_input = "file with spaces; dangerous command";
    let quoted_input = Shlex::quote(user_input); // Escapes the input for shell safety
    let command_string = format!("ls {}", quoted_input);
    let output = Command::new("sh")
       .arg("-c")
       .arg(&command_string) // Shell executes the constructed string
       .output()?;
    ```
    

Die bevorzugte Methode (direkte Argumentübergabe) ist anzuwenden, wann immer dies möglich ist.

## 3. Benutzeroberfläche (Wayland-Integration)

Diese Spezifikation geht primär von einer Implementierung mittels des Smithay-Frameworks aus, was auf die Entwicklung einer spezialisierten Desktop-Shell oder eines Compositor-Bestandteils hindeutet. Alternative Ansätze mittels GTK oder Tauri werden nachrangig behandelt. Die Wahl des UI-Ansatzes hat tiefgreifende Auswirkungen auf die Implementierungsdetails dieses Abschnitts.

### 3.1. Compositor/Shell-Integration (Smithay)

- **Initialisierung:** Die Initialisierung des Compositors erfolgt unter Verwendung der Backend-Module von Smithay.9
    - **Grafik:** `smithay::backend::renderer` (mit Adaptern für EGL/GBM/DRM), `smithay::backend::drm` für die Verwaltung von Displays und Modi. Die Verwendung von `backend_egl` und `backend_drm` ist für typische Linux-Systeme erforderlich.
    - **Input:** `smithay::backend::input` oder bevorzugt `colpetto` für die Integration mit `libinput` und Tokio (siehe unten). `smithay::backend::session` (z.B. `libseat`) für das Session- und Gerätemanagement.
    - **Event Loop:** Die zentrale Event-Schleife basiert auf `calloop`, wie von Smithay vorgegeben.9 Alle Ereignisse (Wayland-Protokoll, Input, Timer) werden über Callbacks in dieser Schleife verarbeitet. Der zentrale Anwendungszustand wird als mutable Referenz an die Callbacks übergeben.
- **Fensterverwaltung (Window Management):** Die Verwaltung von Anwendungsfenstern erfolgt durch die Implementierung des `xdg-shell`-Protokolls.10
    - **Protokoll-Implementierung:** Smithay's Delegations-Makros (`delegate_xdg_shell`, `delegate_xdg_toplevel`, `delegate_xdg_popup`, `delegate_xdg_decoration`, etc.) werden genutzt, um die Server-seitige Logik für `xdg-shell` zu implementieren.9
    - **`xdg_toplevel` Handling:**
        - Anfragen verarbeiten: `set_title`, `set_app_id`, `set_maximized`, `unset_maximized`, `set_fullscreen`, `unset_fullscreen`, `set_minimized`, `move`, `resize`.10
        - Events beantworten: Auf `configure`-Events reagieren (Größe/Status anpassen) und mit `ack_configure` bestätigen. Auf `close`-Events reagieren.10
    - **`xdg_popup` Handling:**
        - Anfragen verarbeiten: `grab`, `reposition`.10
        - Events beantworten: Auf `configure`-Events reagieren (Position/Größe setzen) und mit `ack_configure` bestätigen. Auf `popup_done`-Events reagieren (Popup zerstören).10
    - **Tiling/Snapping:** Implementierung einer benutzerdefinierten Logik für Fensteranordnung (Tiling) oder Andocken (Snapping), inspiriert von Konzepten wie in Tiling Shell oder Snap Assistant.11 Algorithmen definieren, wie Fenster basierend auf Benutzeraktionen (z.B. Ziehen an den Rand), Tastenkürzeln oder der Anzahl der Fenster positioniert und in der Größe angepasst werden.
- **Eingabeverarbeitung (Input Handling):** Die Verarbeitung von Eingabeereignissen von Tastatur, Maus, Touchpad etc. erfolgt über `libinput`.
    - **Bibliothek:** Das Crate `colpetto` wird für die asynchrone Integration von `libinput` mit Tokio verwendet.12 `colpetto` bietet eine Stream-basierte API und berücksichtigt Thread-Sicherheitsaspekte von `libinput` in Tokio-Tasks.12
    - **Initialisierung:** Eine `colpetto::Libinput`-Instanz wird mit `Libinput::new()` erstellt, wobei Closures für das Öffnen und Schließen von Gerätedateien (mittels `rustix::fs::open`) übergeben werden.12 Ein Sitz wird mittels `libinput.assign_seat(c"seat0")` zugewiesen.
    - **Event Stream:** Der asynchrone Event-Stream wird mit `libinput.event_stream()` abgerufen.12
    - **Event Verarbeitung:** Der Stream wird asynchron mittels `while let Some(event) = stream.try_next().await?` verarbeitet.12 Eingehende `colpetto::Event`-Objekte werden mittels Pattern Matching auf `event.event_type()` unterschieden:
        - `EventType::KeyboardKey`: Downcast zu `KeyboardEvent` für Tastencode, Status (Pressed/Released).
        - `EventType::PointerMotion`, `PointerButton`, `PointerAxis`: Downcast zu entsprechenden `Pointer...Event`-Typen für Mausbewegungen, Klicks, Scrollen.
        - `EventType::TouchDown`, `TouchUp`, `TouchMotion`: Downcast zu `Touch...Event`-Typen für Touch-Interaktionen.
        - `EventType::GestureSwipe...`, `GesturePinch...`: Downcast zu `Gesture...Event`-Typen für Gesten.12
    - Die extrahierten Event-Daten werden verwendet, um Aktionen in der Anwendung oder Fensterverwaltungsbefehle auszulösen.
- **Theming:**
    - **Ansatz:** Implementierung eines benutzerdefinierten Theming-Systems. Dies kann auf einem System von Design Tokens basieren, ähnlich wie bei Material Design 3 oder USWDS.13 Tokens definieren Farbpaletten, Typografie, Abstände etc.
    - **Implementierung:** Die Token-Werte werden (z.B. aus einer Konfigurationsdatei) geladen und zur Laufzeit beim Rendern der UI-Elemente angewendet. Alternativ kann eine Integration mit Systemeinstellungen über D-Bus/GSettings erfolgen (siehe Abschnitt 5.8), um z.B. das System-Theme (hell/dunkel) zu übernehmen.

### 3.2. Framework-Integration (Alternativ: GTK/Tauri)

- **GTK:**
    - **Bibliothek:** `gtk4-rs` Bindings verwenden.15
    - **Wayland:** `gdk4-wayland` für spezifische Wayland-Interaktionen nutzen, falls erforderlich.16 Das Standard-GTK-Wayland-Backend übernimmt die meiste Integration.
    - **Systemeinstellungen:** `Gtk.Settings` abfragen, z.B. `is_gtk_application_prefer_dark_theme()`.15
    - **Styling:** `GtkCssProvider` verwenden, um CSS-Daten zu laden und auf Widgets anzuwenden. CSS-Selektoren zielen auf GTK-Widget-Namen und -Klassen. (Hinweis: Detaillierte `GtkCssProvider`-API-Dokumentation muss extern konsultiert werden, da 17 nicht verfügbar war).
- **Tauri:**
    - **Framework:** Tauri-Framework nutzen.18
    - **Backend-Kommunikation:** Rust-Funktionen mit `#[tauri::command]` annotieren.19 Aufruf vom Frontend mittels `invoke()`. Datenübergabe (Argumente, Rückgabewerte, Fehler) zwischen Frontend und Backend definieren.
    - **Events:** Tauri's Event-System (`emit`, `listen`) für asynchrone Benachrichtigungen nutzen.
    - **Frontend:** UI und Styling erfolgen mit Standard-Webtechnologien (HTML, CSS, JavaScript-Framework) innerhalb der Tauri-Webview.

## 4. Systemdienste-Integration (D-Bus APIs)

Die Interaktion mit verschiedenen Systemdiensten erfolgt über deren D-Bus-Schnittstellen.

### 4.1. D-Bus Bibliothek

Die `zbus`-Bibliothek wird für sämtliche D-Bus-Interaktionen verwendet.20 Die `tokio`-Integration von `zbus` wird aktiviert (`features = ["tokio"]`, `default-features = false`), um eine nahtlose Integration in die asynchrone Architektur der Anwendung zu gewährleisten.22 Das `#[proxy]`-Makro von `zbus` wird zur Definition von Client-seitigen Proxies für die D-Bus-Schnittstellen verwendet.22

### 4.2. Geheimnisverwaltung (Freedesktop Secret Service)

Zur sicheren Speicherung von sensiblen Daten wie API-Schlüsseln wird die Freedesktop Secret Service API genutzt.23

- **Schnittstelle:** `org.freedesktop.Secrets` auf dem **Session Bus**.23
- **Proxy:** Es werden `zbus`-Proxy-Traits für die Schnittstellen `org.freedesktop.Secrets.Service`, `org.freedesktop.Secrets.Collection` und `org.freedesktop.Secrets.Item` definiert.22
- **Schlüsselmethoden und Eigenschaften:**
    - `Service::OpenSession()`: Erforderlich vor Operationen wie `CreateItem`. Nur eine Session pro Client.23
    - `Service::DefaultCollection` (Eigenschaft): Pfad zur Standard-Collection abrufen (`/org/freedesktop/secrets/aliases/default`).23 Geheimnisse sollten standardmäßig hier gespeichert werden.
    - `Collection::CreateItem(fields: Dict<String,String>, secret: Secret, label: String, replace: bool)`: Speichert ein neues Geheimnis. `fields` sind Suchattribute. `secret` ist eine Struktur mit `algorithm` (z.B. "PLAIN"), `parameters` (`Array<Byte>`) und `value` (`Array<Byte>`).23
    - `Collection::SearchItems(fields: Dict<String,String>)`: Sucht nach Items innerhalb der Collection anhand von Attributen.23
    - `Service::RetrieveSecrets(items: Array<ObjectPath>)`: Ruft die Geheimniswerte für gegebene Item-Pfade ab.23
    - `Item::Delete()`: Löscht ein spezifisches Geheimnis.23
    - `Item::Secret` (Eigenschaft): Lesen/Schreiben des Geheimniswerts (als `Secret`-Struktur).23
    - `Item::Attributes` (Eigenschaft): Lesen/Schreiben der Suchattribute.23
- **Sperren/Entsperren:** Der `Locked`-Status wird über Eigenschaften der Collection/Item geprüft. Falls `true`, muss die `org.freedesktop.Secrets.Session`-Schnittstelle (erhalten von `OpenSession`) verwendet werden: `Session::BeginAuthenticate()` initiiert den Entsperrvorgang.23
- **Datenstrukturen:** `std::collections::HashMap<String, String>` für Attribute. Für die `Secret`-Struktur und andere D-Bus-Typen werden entsprechende Rust-Typen oder `zbus::zvariant::Value` / `OwnedValue` in den Proxy-Definitionen verwendet.22

### 4.3. Netzwerkverwaltung (NetworkManager)

Zur Abfrage des Netzwerkstatus und zur Verwaltung von Verbindungen wird NetworkManager über D-Bus angesprochen.

- **Schnittstelle:** `org.freedesktop.NetworkManager` und zugehörige Schnittstellen (z.B. `.Device`, `.Connection.Active`) auf dem **System Bus**.26
- **Proxy:** `zbus`-Proxy-Traits definieren.
- **Schlüsselmethoden, Eigenschaften und Signale:**
    - `Manager::GetDevices()`: Liste der Netzwerkgeräte abrufen.
    - `Manager::ActivateConnection()`, `Manager::DeactivateConnection()`: Netzwerkverbindungen aktivieren/deaktivieren (erfordert PolicyKit-Berechtigungen).
    - `Manager::State` (Eigenschaft): Globalen Netzwerkstatus abrufen (z.B. verbunden, getrennt).
    - `Manager::ActiveConnections` (Eigenschaft): Liste der aktiven Verbindungspfade.
    - `Manager::StateChanged` (Signal): Änderungen im globalen Netzwerkstatus überwachen.27
    - `Device::State` (Eigenschaft): Status eines spezifischen Geräts.
    - `ActiveConnection::State` (Eigenschaft): Status einer aktiven Verbindung.

### 4.4. Energieverwaltung (UPower)

Informationen über den Batteriestatus und die Stromversorgung werden über UPower abgefragt.

- **Schnittstelle:** `org.freedesktop.UPower`, `org.freedesktop.UPower.Device` auf dem **System Bus**.28
- **Proxy:** `zbus`-Proxy-Traits definieren oder das Crate `upower_dbus` verwenden.29
- **Schlüsselmethoden, Eigenschaften und Signale:**
    - `UPower::EnumerateDevices()`: Liste der Energieverwaltungsgeräte.
    - `UPower::GetDisplayDevice()`: Primäres Anzeigegerät (Batterie/USV) abrufen.
    - `UPower::DeviceAdded`, `UPower::DeviceRemoved` (Signale): Geräteänderungen überwachen.
    - `Device::OnBattery` (Eigenschaft): Prüfen, ob auf Batteriebetrieb.
    - `Device::Percentage` (Eigenschaft): Ladezustand in Prozent.
    - `Device::State` (Eigenschaft): Lade-/Entladezustand (z.B. Charging, Discharging, FullyCharged).
    - `Device::TimeToEmpty`, `Device::TimeToFull` (Eigenschaften): Geschätzte Restlaufzeit/Ladezeit in Sekunden.
    - `Device::Changed` (Signal): Änderungen an Geräteeigenschaften überwachen.28

### 4.5. Sitzungs- und Systemsteuerung (logind)

Systemweite Aktionen wie Suspend, Reboot oder das Sperren der Sitzung werden über `systemd-logind` gesteuert.

- **Schnittstelle:** `org.freedesktop.login1.Manager`, `org.freedesktop.login1.Session` auf dem **System Bus**.30
- **Proxy:** `zbus`-Proxy-Traits definieren oder das Crate `logind-dbus` verwenden.31
- **Schlüsselmethoden, Eigenschaften und Signale:**
    - `Manager::Suspend(interactive: false)`, `Hibernate(false)`, `Reboot(false)`, `PowerOff(false)`: Systemzustandsänderungen initiieren (erfordert PolicyKit-Berechtigungen).30 Der Parameter `interactive=false` wird verwendet, um Benutzerinteraktion für die Autorisierung zu vermeiden.
    - `Manager::LockSessions()`: Alle aktiven Sitzungen sperren.
    - `Session::Lock()`: Die spezifische Sitzung sperren, die dem Session-Objekt zugeordnet ist.30
    - `Manager::GetSession(session_id)`, `Manager::GetUser(uid)`: Objektpfade für spezifische Sitzungen/Benutzer abrufen.
    - `Manager::IdleHint` (Eigenschaft): System-Idle-Status abfragen.
    - `Manager::PrepareForShutdown(start: bool)` (Signal): Signal vor (`true`) und nach (`false`) dem Beginn des Shutdown-Prozesses.30 Kann für Aufräumarbeiten genutzt werden (ggf. mit Inhibitor Locks).

### 4.6. Benachrichtigungen (Freedesktop Notifications)

Desktop-Benachrichtigungen werden über die standardisierte Notifications-Schnittstelle gesendet.

- **Schnittstelle:** `org.freedesktop.Notifications` auf dem **Session Bus**.32
- **Proxy:** `zbus`-Proxy-Trait definieren.22
- **Schlüsselmethoden und Signale:**
    - `Notify(app_name: String, replaces_id: u32, app_icon: String, summary: String, body: String, actions: Array<String>, hints: Dict<String, Variant>, expire_timeout: i32) -> u32`: Sendet eine Benachrichtigung. `actions` ist ein Array von `[action_key1, display_name1, action_key2, display_name2,...]`. Der Standard-Aktionsschlüssel ist `"default"`. `hints` können z.B. Dringlichkeit (`urgency`) oder Kategorie (`category`) enthalten. `expire_timeout` in ms (-1 = default, 0 = nie).32 Gibt die Benachrichtigungs-ID zurück.
    - `CloseNotification(id: u32)`: Schließt eine Benachrichtigung anhand ihrer ID.32
    - `NotificationClosed(id: u32, reason: u32)` (Signal): Wird gesendet, wenn eine Benachrichtigung geschlossen wird (Grund: 1=expired, 2=dismissed, 3=closed by call, 4=undefined).32
    - `ActionInvoked(id: u32, action_key: String)` (Signal): Wird gesendet, wenn der Benutzer auf eine Aktion (oder den Benachrichtigungskörper für `"default"`) klickt.32

### 4.7. Berechtigungsverwaltung (PolicyKit)

Für Aktionen, die erhöhte Rechte erfordern, wird PolicyKit zur Autorisierungsprüfung verwendet.

- **Schnittstelle:** `org.freedesktop.PolicyKit1.Authority` auf dem **System Bus**.33
- **Proxy:** `zbus`-Proxy-Trait definieren.
- **Verwendung:** Notwendig für privilegierte Operationen wie `logind`-Energieaktionen oder `NetworkManager`-Verbindungsänderungen.27
- **Schlüsselmethode:** `CheckAuthorization(subject, action_id, details, flags, cancellation_id) -> AuthorizationResult`: Prüft, ob das anfragende Subjekt (Prozess) die Berechtigung für die angegebene `action_id` hat.
    - `subject`: Identifiziert den Prozess/Benutzer, für den die Prüfung erfolgt (oft der aufrufende Prozess).
    - `action_id`: Die spezifische PolicyKit-Aktions-ID (z.B. `org.freedesktop.login1.power-off`). Diese IDs müssen für alle privilegierten Aktionen der Anwendung identifiziert und dokumentiert werden.
    - `details`: Zusätzliche kontextabhängige Informationen.
    - `flags`: Steuert das Verhalten (z.B. ob Interaktion erlaubt ist).
    - **Rückgabe (`AuthorizationResult`):** Enthält Informationen, ob die Aktion erlaubt ist (`authorized`), ob Benutzerinteraktion/Authentifizierung erforderlich ist (`challenge`) oder ob sie verboten ist (`not_authorized`).
- **Authentifizierungsagenten:** Wenn das Ergebnis `challenge` ist, muss die Anwendung möglicherweise mit einem PolicyKit Authentication Agent interagieren, um den Benutzer zur Eingabe eines Passworts aufzufordern.33 Die genaue Interaktion hängt von der Systemkonfiguration und den `flags` ab.

Die Notwendigkeit von PolicyKit-Prüfungen impliziert, dass für die korrekte Funktion der Anwendung auf dem Zielsystem entsprechende PolicyKit-Regeln konfiguriert sein müssen, die der Anwendung die notwendigen Berechtigungen erteilen (ggf. nach Authentifizierung). Dies ist ein wichtiger Aspekt für die Installation und Systemadministration.

### 4.8. Systemeinstellungen (GSettings/DConf)

Zum Lesen von systemweiten oder benutzerspezifischen Einstellungen (z.B. Theme, Schriftarten) wird GSettings verwendet, das typischerweise DConf als Backend nutzt.

- **Schnittstelle:** Direkte Interaktion mit der D-Bus-Schnittstelle des DConf-Dienstes (z.B. `ca.desrt.dconf` auf dem **Session Bus**) mittels `zbus` oder Verwendung von GIO-Bindings (`gtk-rs`/`gio`), falls GTK integriert ist. Das Crate `gnome-dbus-api` 34 bietet spezifische Abstraktionen, ist aber möglicherweise zu GNOME-spezifisch.
- **Proxy:** Bei direkter D-Bus-Nutzung: `zbus`-Proxy für die DConf-Schnittstelle (z.B. `ca.desrt.dconf.Read`).
- **Verwendung:** Lesen von relevanten Schlüsseln (z.B. unter `/org/gnome/desktop/interface/` für GTK-Theme, Schriftart; `/org/gnome/desktop/a11y/` für Barrierefreiheit). Überwachung von Schlüsseländerungen mittels D-Bus-Signalen (`ca.desrt.dconf.Watch`).

### 4.9. D-Bus Schnittstellenübersicht

|   |   |   |   |   |   |
|---|---|---|---|---|---|
|**Schnittstellenname**|**D-Bus Pfad**|**Bus Typ**|**Schlüsselmethoden/Eigenschaften/Signale**|**Zweck in der Anwendung**|**Erforderliche Berechtigungen (PolicyKit Action ID)**|
|`org.freedesktop.Secrets.Service`|`/org/freedesktop/secrets`|Session|`OpenSession`, `DefaultCollection`, `RetrieveSecrets`|Sichere Speicherung/Abruf von API-Schlüsseln etc.|-|
|`org.freedesktop.Secrets.Collection`|`/org/freedesktop/secrets/collection/*`|Session|`CreateItem`, `SearchItems`, `Locked` (Prop)|Verwaltung von Geheimnissen in einer Collection|-|
|`org.freedesktop.Secrets.Item`|`/org/freedesktop/secrets/item/*`|Session|`Delete`, `Secret` (Prop), `Attributes` (Prop), `Locked` (Prop)|Zugriff/Verwaltung einzelner Geheimnisse|-|
|`org.freedesktop.Secrets.Session`|(von `OpenSession` erhalten)|Session|`BeginAuthenticate`|Entsperren von Collections/Items|-|
|`org.freedesktop.NetworkManager`|`/org/freedesktop/NetworkManager`|System|`GetDevices`, `ActivateConnection`, `DeactivateConnection`, `State` (Prop), `ActiveConnections` (Prop), `StateChanged` (Sig)|Netzwerkstatus abfragen, Verbindungen verwalten|`org.freedesktop.NetworkManager.network-control`|
|`org.freedesktop.UPower`|`/org/freedesktop/UPower`|System|`EnumerateDevices`, `GetDisplayDevice`, `DeviceAdded` (Sig), `DeviceRemoved` (Sig)|Energiegeräte erkennen|-|
|`org.freedesktop.UPower.Device`|`/org/freedesktop/UPower/devices/*`|System|`OnBattery` (Prop), `Percentage` (Prop), `State` (Prop), `TimeToEmpty` (Prop), `TimeToFull` (Prop), `Changed` (Sig)|Batteriestatus/Energiequelle abfragen|-|
|`org.freedesktop.login1.Manager`|`/org/freedesktop/login1`|System|`Suspend`, `Hibernate`, `Reboot`, `PowerOff`, `LockSessions`, `GetSession`, `GetUser`, `IdleHint` (Prop), `PrepareForShutdown` (Sig)|Systemsteuerung (Energie, Idle, Sitzungen sperren)|`org.freedesktop.login1.suspend`, `.hibernate`, `.reboot`, `.power-off`, `.lock-sessions`|
|`org.freedesktop.login1.Session`|`/org/freedesktop/login1/session/*`|System|`Lock`|Einzelne Sitzung sperren|`org.freedesktop.login1.lock-session` (implizit)|
|`org.freedesktop.Notifications`|`/org/freedesktop/Notifications`|Session|`Notify`, `CloseNotification`, `NotificationClosed` (Sig), `ActionInvoked` (Sig)|Desktop-Benachrichtigungen senden/verwalten|-|
|`org.freedesktop.PolicyKit1.Authority`|`/org/freedesktop/PolicyKit1/Authority`|System|`CheckAuthorization`|Berechtigungen für privilegierte Aktionen prüfen|-|
|`ca.desrt.dconf` (Beispiel)|`/ca/desrt/dconf`|Session|`Read`, `Watch` (Signale)|Systemeinstellungen (Theme, Fonts etc.) lesen/überwachen|-|

## 5. LLM-Integration (Model Context Protocol - MCP)

Die Integration mit Large Language Models (LLMs) erfolgt über das Model Context Protocol (MCP).35 Die Anwendung agiert als MCP-Host/Client.

### 5.1. MCP Client Implementierungsstrategie

Die Implementierung des MCP-Clients erfolgt unter Verwendung des offiziellen Rust SDKs (`modelcontextprotocol/rust-sdk`), sofern dieses bei Projektstart ausreichend stabil und vollständig ist.35 Sollte das offizielle SDK nicht verfügbar oder unzureichend sein, wird das inoffizielle SDK (`jeanlucthumm/modelcontextprotocol-rust-sdk`) evaluiert und ggf. genutzt.37 Als Fallback-Option wird der MCP-Client manuell implementiert, basierend auf der JSON-RPC 2.0 Spezifikation unter Verwendung des `jsonrpc-v2`-Crates 38 und `serde` für die (De-)Serialisierung. **Die gewählte Strategie ist:**.

### 5.2. Transportmechanismus

Der für die MCP-Kommunikation zu unterstützende Transportmechanismus ist ****.

- **WebSocket:** Die Implementierung erfolgt mittels `tokio-tungstenite` oder einer äquivalenten, Tokio-kompatiblen WebSocket-Client-Bibliothek.40
- **Standard I/O (stdio):** Nachrichten werden über die Standard-Ein-/Ausgabe des Prozesses gesendet/empfangen, wobei JSON-RPC-Nachrichten korrekt gerahmt (z.B. durch Längenpräfixe oder Trennzeichen) und geparst werden müssen.
- **Server-Sent Events (SSE):** Eine HTTP-Verbindung wird aufgebaut, und Nachrichten vom Server werden als SSE empfangen. Anfragen vom Client an den Server erfordern einen separaten Mechanismus (typischerweise HTTP POST an einen definierten Endpunkt).

### 5.3. Verbindungsaufbau und Initialisierung

Die Logik zum Aufbau der Verbindung über den gewählten Transportmechanismus wird implementiert. Nach erfolgreichem Verbindungsaufbau erfolgt der MCP-Initialisierungs-Handshake gemäß Spezifikation 36:

1. Client sendet `initialize`-Request mit seinen Fähigkeiten (`ClientCapabilities`).
2. Server antwortet mit `initialize`-Response, die seine Fähigkeiten (`ServerCapabilities`) enthält.
3. Client sendet `initialized`-Notification an den Server.

### 5.4. Anfrage/Antwort-Verarbeitung (JSON-RPC 2.0)

Alle MCP-Nachrichten folgen dem JSON-RPC 2.0 Format.36

- **Serialisierung/Deserialisierung:** Das `serde`-Crate 41 wird verwendet, um Rust-Datenstrukturen (die die MCP-Schema-Typen abbilden) in JSON zu serialisieren (für Requests/Notifications) und JSON-Antworten/Notifications in Rust-Strukturen zu deserialisieren. Die MCP-Schema-Definitionen 36 sind maßgeblich für die Struktur der Rust-Typen.
- **Methoden-Handler (Server -> Client):** Implementierung von Handlern für vom Server initiierte Anfragen:
    - **`tool/call`:**
        1. Empfange `tool/call`-Request vom Server.
        2. **Einwilligungsprüfung:** Zeige dem Benutzer eine Aufforderung zur expliziten Bestätigung an, die klar beschreibt, welche Aktion das Tool (`toolId`) mit den gegebenen Argumenten (`inputs`) ausführen wird.36 Warte auf Benutzerinteraktion.
        3. Bei Zustimmung: Führe die lokale Funktion aus, die dem `toolId` entspricht.
        4. Bei Ablehnung oder Fehler: Sende eine entsprechende JSON-RPC-Fehlerantwort an den Server.
        5. Bei erfolgreicher Ausführung: Serialisiere das Ergebnis und sende eine `tool/result`-Antwort an den Server.
    - **`resource/read`:**
        1. Empfange `resource/read`-Request vom Server.
        2. **Einwilligungsprüfung:** Zeige dem Benutzer eine Aufforderung zur expliziten Bestätigung an, die klar beschreibt, welche Daten (`resourceId`) angefragt werden.36 Warte auf Benutzerinteraktion.
        3. Bei Zustimmung: Rufe die angeforderten Ressourcendaten ab (z.B. Dateiinhalt, Datenbankabfrage).
        4. Bei Ablehnung oder Fehler: Sende eine entsprechende JSON-RPC-Fehlerantwort.
        5. Bei Erfolg: Serialisiere die Ressourcendaten und sende eine `resource/result`-Antwort.
    - **`sampling/request`:**
        1. Empfange `sampling/request`-Request vom Server.
        2. **Einwilligungsprüfung (Stufe 1):** Prüfe, ob der Benutzer Sampling generell erlaubt hat.
        3. **Einwilligungsprüfung (Stufe 2 - Kritisch):** Zeige dem Benutzer den exakten Prompt (`prompt`), der an das LLM gesendet werden soll, zur expliziten Genehmigung an.36 Der Benutzer muss die Möglichkeit haben, den Prompt zu ändern oder abzulehnen.
        4. **Einwilligungsprüfung (Stufe 3):** Konfiguriere, welche Teile der LLM-Antwort der Server sehen darf, basierend auf Benutzereinstellungen/-genehmigung.36
        5. Bei Zustimmung: Interagiere mit dem LLM (lokal oder über API).
        6. Filtere die LLM-Antwort gemäß Stufe 3 der Einwilligung.
        7. Bei Ablehnung oder Fehler: Sende eine entsprechende JSON-RPC-Fehlerantwort.
        8. Bei Erfolg: Serialisiere die (gefilterte) LLM-Antwort und sende eine `sampling/response`-Antwort.

### 5.5. Notification-Verarbeitung (Server -> Client)

Implementierung von Handlern für eingehende MCP-Notifications vom Server (z.B. `$/progress`, Statusänderungen), um den UI-Zustand entsprechend zu aktualisieren.

### 5.6. Einwilligungsmanagement (Consent Management)

Die Verwaltung der Benutzereinwilligung ist ein **zentraler und kritischer Aspekt** der MCP-Implementierung.36

- **Explizite Zustimmung:** Für _jede_ `tool/call`-, `resource/read`- und `sampling`-Anfrage vom Server _muss_ eine explizite, informierte Zustimmung des Benutzers eingeholt werden, _bevor_ die Aktion ausgeführt oder Daten preisgegeben werden.
- **UI-Fluss:** Implementierung klarer und verständlicher UI-Dialoge für Einwilligungsanfragen. Diese müssen präzise angeben:
    - Welches Tool ausgeführt werden soll und was es tut.
    - Welche Ressource gelesen werden soll und welche Daten sie enthält.
    - Welcher genaue Prompt für das Sampling verwendet wird (mit Änderungs-/Ablehnungsoption).
    - Welche Ergebnisse der Server sehen darf (bei Sampling).
- **Persistenz:** Einwilligungsentscheidungen können optional persistent gespeichert werden (z.B. "Für diese Sitzung merken", "Immer erlauben/ablehnen für dieses Tool/diese Ressource"). Diese persistenten Zustimmungen müssen sicher gespeichert werden, idealerweise über die Freedesktop Secret Service API (siehe Abschnitt 4.2), falls sie sensible Berechtigungen abdecken.

### 5.7. Sicherheitsaspekte

Die Implementierung muss die MCP-Sicherheitsprinzipien strikt befolgen 36:

- **User Consent and Control:** Absolute Priorität (siehe 5.6).
- **Data Privacy:** Keine Datenweitergabe ohne explizite Zustimmung. Strenge Zugriffskontrollen auf lokale Daten.
- **Tool Safety:** Tool-Beschreibungen vom Server als potenziell nicht vertrauenswürdig behandeln.36 Tools mit minimal notwendigen Rechten ausführen. Kritische Aktionen erfordern menschliche Bestätigung.
- **LLM Sampling Controls:** Benutzerkontrolle über Prompt und Sichtbarkeit der Ergebnisse sicherstellen.36
- **Input Validation:** Alle vom Server empfangenen Daten (insbesondere in `tool/call`-Argumenten) validieren.42
- **Rate Limiting/Timeouts:** Implementierung von Timeouts für MCP-Anfragen. Falls die Anwendung auch als MCP-Server agiert, ist Rate Limiting erforderlich.42

Die Sicherheit des Gesamtsystems hängt maßgeblich von der korrekten Implementierung der Einwilligungs- und Kontrollmechanismen im MCP-Client ab, da das Protokoll selbst diese nicht erzwingt.

### 5.8. MCP Nachrichtenverarbeitung

|   |   |   |   |   |
|---|---|---|---|---|
|**MCP Methode/Notification**|**Richtung**|**Schlüsselparameter**|**Aktion im Client**|**Einwilligungsanforderung**|
|`initialize`|C -> S|`processId`, `clientInfo`, `capabilities`|Sende Client-Fähigkeiten an Server.|-|
|`initialize`|S -> C|`serverInfo`, `capabilities`|Empfange und speichere Server-Fähigkeiten.|-|
|`initialized`|C -> S|-|Bestätige erfolgreiche Initialisierung.|-|
|`shutdown`|C -> S|-|Informiere Server über bevorstehende Trennung.|-|
|`shutdown`|S -> C|-|Empfange Bestätigung für Shutdown.|-|
|`exit`|C -> S|-|Informiere Server über sofortige Trennung.|-|
|`exit`|S -> C|-|Informiere Client über sofortige Trennung durch Server.|-|
|`tool/call`|S -> C|`callId`, `toolId`, `inputs`|**Fordere explizite Zustimmung an.** Bei Zustimmung: Führe Tool aus. Sende `tool/result` oder Fehlerantwort.|**Ja (Explizit, pro Aufruf)** für Ausführung des Tools mit gegebenen Parametern.36|
|`tool/result`|C -> S|`callId`, `result` / `error`|Sende Ergebnis oder Fehler der Tool-Ausführung an Server.|- (Einwilligung erfolgte vor Ausführung)|
|`resource/read`|S -> C|`readId`, `resourceId`, `params`|**Fordere explizite Zustimmung an.** Bei Zustimmung: Lese Ressource. Sende `resource/result` oder Fehlerantwort.|**Ja (Explizit, pro Lesezugriff)** für Zugriff auf die spezifische Ressource.36|
|`resource/result`|C -> S|`readId`, `resource` / `error`|Sende Ressourcendaten oder Fehler an Server.|- (Einwilligung erfolgte vor Lesezugriff)|
|`sampling/request`|S -> C|`sampleId`, `prompt`, `params`|**Fordere explizite Zustimmung an (Prompt-Review!).** Bei Zustimmung: Führe LLM-Sampling aus. Sende `sampling/response`.|**Ja (Explizit, pro Anfrage)**, muss Genehmigung des _exakten Prompts_ und Kontrolle über Ergebnis-Sichtbarkeit beinhalten.36|
|`sampling/response`|C -> S|`sampleId`, `response` / `error`|Sende (gefiltertes) LLM-Ergebnis oder Fehler an Server.|- (Einwilligung erfolgte vor Sampling)|
|`$/progress`|S -> C|`token`, `value`|Aktualisiere UI, um Fortschritt anzuzeigen.|-|
|_Weitere Notifications_|S -> C|_Spezifisch_|Verarbeite server-spezifische Benachrichtigungen.|-|

## 6. Sicherheitsimplementierungsdetails

Eine umfassende Sicherheitsstrategie ist erforderlich, die verschiedene Angriffsvektoren berücksichtigt.

### 6.1. Eingabebereinigung

- **HTML:** Wie in Abschnitt 2.1 beschrieben, wird `ammonia` mit einer strikten Whitelist-Konfiguration verwendet, um jeglichen von externen Quellen (insbesondere LLM-Ausgaben) stammenden HTML-Code zu bereinigen.3
- **Kommandozeilenargumente:** Wie in Abschnitt 2.2 beschrieben, wird die direkte Übergabe von Argumenten an `std::process::Command` bevorzugt, um Shell-Injection zu verhindern.7 Bei unvermeidbarer Shell-Nutzung wird `shlex::quote` verwendet.

### 6.2. LLM-Interaktionssicherheit

LLM-Interaktionen bergen spezifische Risiken, die adressiert werden müssen.

- **Ausgabebewertung/-bereinigung:**
    - **Zero-Trust-Ansatz:** Jede LLM-Ausgabe wird als nicht vertrauenswürdig behandelt.4
    - **Validierung:** Wenn strukturierte Ausgabe (z.B. JSON) erwartet wird, muss diese gegen ein Schema validiert werden.43 Ungültige oder unerwartete Strukturen werden abgelehnt.
    - **Bereinigung:** Freitextausgaben, die potenziell Markup enthalten könnten, werden mit `ammonia` bereinigt (siehe 6.1).4
    - **Downstream-Schutz:** Es muss sichergestellt werden, dass LLM-Ausgaben keine schädlichen Aktionen in nachgelagerten Komponenten auslösen können (z.B. Ausführung von generiertem Code, Einschleusung von Befehlen, XSS in Webviews).4
- **Prompt-Injection-Mitigation:** Maßnahmen gegen Prompt Injection (OWASP LLM #1 44) sind unerlässlich:
    - **Eingabefilterung:** Benutzereingaben, die Teil eines Prompts werden, werden gefiltert, um bekannte Angriffsmuster zu erkennen und zu neutralisieren.44
    - **Trennung von Instruktionen und Daten:** Innerhalb des Prompts werden Systeminstruktionen klar von Benutzereingaben oder externen Daten getrennt (z.B. durch spezielle Markierungen oder strukturierte Formate wie ChatML, falls vom LLM unterstützt).45
    - **Least Privilege:** Über MCP bereitgestellte Tools, die vom LLM aufgerufen werden können, dürfen nur die minimal notwendigen Berechtigungen haben.44
    - **Menschliche Bestätigung:** Hoch-Risiko-Aktionen, die durch LLM-Interaktion ausgelöst werden (z.B. Dateilöschung, Senden von E-Mails), erfordern eine explizite Bestätigung durch den Benutzer über die MCP-Einwilligungsmechanismen (siehe 5.6).44

### 6.3. Sichere Speicherung

Sensible Daten wie API-Schlüssel oder persistente Benutzereinwilligungen werden ausschließlich über die Freedesktop Secret Service API gespeichert (siehe Abschnitt 4.2).23 Sie dürfen niemals im Klartext in Konfigurationsdateien oder im Quellcode gespeichert werden.

Die Kombination dieser Maßnahmen (Input Sanitization, Output Validation, Prompt Injection Mitigation, Secure Storage) bildet eine mehrschichtige Verteidigung (Defense in Depth), die für die Sicherheit der Anwendung entscheidend ist. Die Orientierung an den OWASP Top 10 für LLMs 4 hilft dabei, die relevantesten Risiken zu adressieren.

## 7. Konfigurationsmanagement

### 7.1. Format

Die Konfiguration der Anwendung erfolgt über Dateien im TOML-Format. TOML ist gut lesbar und wird von `serde` unterstützt.41

### 7.2. Parsen

- **Bibliothek:** Das `serde`-Crate 41 in Kombination mit `serde_toml` wird zum Parsen der TOML-Dateien verwendet. Eine zentrale `Config`-Struktur wird mit `#` annotiert.
- **Optional:** Das `config-rs`-Crate kann alternativ verwendet werden, um das Mergen von Konfigurationen aus verschiedenen Quellen (Datei, Umgebungsvariablen) zu vereinfachen.
- **Beispielgenerierung:** Das `toml-example`-Crate 47 kann optional genutzt werden, um automatisch Beispiel-Konfigurationsdateien basierend auf der `Config`-Struktur und deren Dokumentationskommentaren zu generieren.

### 7.3. Speicherort

Konfigurationsdateien werden an standardkonformen Orten gemäß der XDG Base Directory Specification gesucht:

1. Benutzerspezifisch: `$XDG_CONFIG_HOME/app-name/config.toml` (Fallback: `~/.config/app-name/config.toml`)
2. Systemweit: `/etc/xdg/app-name/config.toml` (Fallback: `/etc/app-name/config.toml`)

Benutzerspezifische Einstellungen überschreiben systemweite Einstellungen.

### 7.4. Parameter

Alle konfigurierbaren Parameter werden in der zentralen `Config`-Struktur definiert und in der folgenden Tabelle dokumentiert.

### 7.5. Konfigurationsparameter

|   |   |   |   |   |
|---|---|---|---|---|
|**Parameter Name (TOML Schlüssel)**|**Rust Typ**|**Standardwert**|**Beschreibung**|**Erforderlich**|
|`mcp.transport_type`|`String`|`"websocket"`|Transportmechanismus für MCP ("websocket", "stdio", "sse").|Nein|
|`mcp.server_address`|`Option<String>`|`None`|Adresse des MCP-Servers (z.B. "ws://localhost:8080" für WebSocket).|Ja (falls!= stdio)|
|`llm.api_key_secret_service_key`|`Option<String>`|`None`|Attribut-Schlüssel (z.B. `llm_api_key`) zum Suchen des LLM-API-Schlüssels im Secret Service.|Nein|
|`ui.theme`|`Option<String>`|`None`|Pfad zu einer benutzerdefinierten Theme-Datei oder Name eines System-Themes.|Nein|
|`logging.level`|`String`|`"info"`|Log-Level (z.B. "trace", "debug", "info", "warn", "error").|Nein|
|`persistence.database_path`|`Option<String>`|`None`|Pfad zur SQLite-Datenbankdatei (falls Persistenz aktiviert).|Nein|
|**|**|**|**|_[Ja/Nein]_|

Diese klare Definition der Konfiguration verbessert die Benutzerfreundlichkeit und Wartbarkeit der Anwendung.

## 8. Datenpersistenz (Falls zutreffend)

### 8.1. Anforderung

Persistente Speicherung wird benötigt für: ****

### 8.2. Datenbanksystem

SQLite wird als Datenbanksystem verwendet.48 Es ist dateibasiert, erfordert keine separate Serverinstallation und eignet sich gut für Desktop-Anwendungen.

### 8.3. ORM/Query Builder

`sqlx` wird als primäre Bibliothek für die Datenbankinteraktion eingesetzt.48 `sqlx` bietet asynchrone Operationen, Compile-Zeit-geprüfte SQL-Abfragen und integriertes Migrationsmanagement.

### 8.4. Schema-Definition & Migrationen

- **Schema:** Das Datenbankschema wird durch SQL-Dateien im Verzeichnis `migrations/` definiert. Jede Datei repräsentiert eine Migration und hat einen Zeitstempel als Präfix (z.B. `20250101120000_create_users_table.sql`).
- **Migrationen zur Laufzeit:** Die Migrationen werden zur Laufzeit beim Anwendungsstart automatisch angewendet. Dies geschieht durch Einbetten der Migrationsdateien mittels des `sqlx::migrate!`-Makros und Ausführen von `.run(&pool).await?` auf dem Migrator-Objekt.51
    
    Rust
    
    ```
    // Example in main application setup
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
       .connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    ```
    
- **Entwicklung:** Während der Entwicklung kann `sqlx-cli migrate run` (nach Installation mit `cargo install sqlx-cli --features sqlite`) verwendet werden, um Migrationen manuell anzuwenden und zu testen.51 Der `DATABASE_URL` muss entsprechend gesetzt sein.

Die Einbettung von Migrationen stellt sicher, dass die Datenbankstruktur immer mit der Version des Anwendungscodes übereinstimmt, was die Bereitstellung vereinfacht.

### 8.5. Datenzugriffsschicht (Data Access Layer)

- **Strukturen:** Rust-Strukturen, die Datenbanktabellen oder Abfrageergebnisse repräsentieren, werden mit `#` annotiert.51
- **Abfragen:** SQL-Abfragen werden mittels der Makros `sqlx::query!("...")` (für Abfragen ohne Rückgabewert oder mit einfachen Typen) oder `sqlx::query_as!(OutputType, "...")` (zum Mappen von Ergebnissen auf `FromRow`-annotierte Strukturen) ausgeführt.51 Diese Makros prüfen die Abfragen zur Compile-Zeit gegen die Datenbank (erfordert gesetzten `DATABASE_URL` während des Builds).
- **Verbindungspooling:** Ein `sqlx::sqlite::SqlitePool` wird mittels `SqlitePoolOptions` konfiguriert und initialisiert, um Datenbankverbindungen effizient zu verwalten.51 Alle Datenbankoperationen werden über den Pool ausgeführt.

Die Compile-Zeit-Prüfung von `sqlx` reduziert das Risiko von Laufzeitfehlern aufgrund syntaktisch falscher oder typ-inkompatibler SQL-Abfragen erheblich.

## 9. Interaktion mit externen Prozessen

### 9.1. Anforderung

Die Anwendung muss mit folgenden externen Kommandozeilen-Tools interagieren: ****.8

### 9.2. Ausführung

Die Ausführung externer Prozesse erfolgt über die `std::process::Command`-API.8

- **Sicherheit:** Es wird **keine** Shell (`sh -c`, `bash -c` etc.) zur Ausführung verwendet, um Command Injection zu verhindern.7 Das auszuführende Programm wird direkt angegeben, und alle Argumente werden einzeln mittels `.arg()` oder `.args()` hinzugefügt.8
    
    Rust
    
    ```
    use std::process::{Command, Stdio};
    let search_term = "config.toml";
    let output = Command::new("/usr/bin/plocate") // Full path or ensure it's in PATH
       .arg("--ignore-case")
       .arg(search_term) // Argument passed directly
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .spawn()?
       .wait_with_output()?;
    ```
    

### 9.3. Ein-/Ausgabebehandlung

- **Standard Streams:** `stdout` und `stderr` werden mittels `Stdio::piped()` umgeleitet, um die Ausgabe des Kindprozesses lesen zu können.8 `stdin` kann ebenfalls mit `Stdio::piped()` umgeleitet werden, um Daten an den Kindprozess zu senden, indem auf den `stdin`-Handle geschrieben wird.8
- **Asynchrone Verarbeitung:** Falls die Ausgabe des Kindprozesses kontinuierlich oder nebenläufig verarbeitet werden muss, wird `tokio::process::Command` verwendet oder die Standard-Handles von `std::process` werden mit Tokio's I/O-Utilities (z.B. `tokio::io::BufReader`) integriert.

### 9.4. Argument-Maskierung

Da keine Shell verwendet wird, ist eine spezielle Maskierung von Argumenten im Allgemeinen nicht notwendig. Die Argumente werden vom Betriebssystem direkt an den Prozess übergeben. Sollte es _zwingende_ Gründe geben, einen Befehlsstring für eine Shell zu konstruieren (stark abgeraten), muss `shlex::quote` verwendet werden (siehe Abschnitt 2.2).7

### 9.5. Fehlerbehandlung

Der `ExitStatus` des beendeten Prozesses wird überprüft (`output.status.success()`).8 Ein nicht erfolgreicher Exit-Code (ungleich Null) wird als Fehler behandelt. Die `stderr`-Ausgabe wird gelesen und geloggt oder zur Fehleranalyse verwendet.8 Mögliche I/O-Fehler beim Lesen/Schreiben der Streams werden ebenfalls behandelt.

## 10. Schlussfolgerung

Diese Spezifikation legt die technischen Grundlagen für die Entwicklung der Anwendung fest, wobei ein starker Fokus auf Sicherheit, Robustheit und Integration in moderne Linux-Desktop-Umgebungen gelegt wird. Die Wahl von Rust und Tokio bildet die Basis für eine performante und nebenläufige Architektur.

Die detaillierte Spezifikation der D-Bus-Schnittstellen (Secret Service, NetworkManager, UPower, logind, Notifications, PolicyKit, GSettings) ermöglicht eine tiefe Integration mit Systemdiensten. Die konsequente Nutzung von `zbus` vereinheitlicht die D-Bus-Kommunikation. Besondere Aufmerksamkeit erfordert die korrekte Handhabung von PolicyKit für privilegierte Aktionen.

Die Integration des Model Context Protocols (MCP) ist ein Kernbestandteil für die LLM-Funktionalität. Die Implementierung muss die Sicherheitsprinzipien von MCP, insbesondere das explizite Einholen der Benutzereinwilligung für Tool-Ausführungen, Ressourcenzugriffe und LLM-Sampling (inklusive Prompt-Review), strikt umsetzen, da der Client hier als kritischer Gatekeeper fungiert.

Die Sicherheitsimplementierung adressiert bekannte Risiken durch Input-Sanitization (HTML mit `ammonia`, Kommandozeilenargumente), rigorose Behandlung von LLM-Ausgaben (Validierung, Bereinigung, Zero-Trust) und Maßnahmen gegen Prompt Injection gemäß OWASP LLM Top 10. Die sichere Speicherung sensibler Daten über den Secret Service ist obligatorisch.

Die Wahl des UI-Frameworks (primär Smithay für eine Compositor/Shell-Komponente, alternativ GTK/Tauri) bestimmt maßgeblich die Implementierung der Benutzeroberfläche und der Wayland-Integration. Bei Verwendung von Smithay ist die korrekte Handhabung von `xdg-shell` und die asynchrone Eingabeverarbeitung mittels `colpetto` entscheidend.

Die Verwendung von `sqlx` für die Datenpersistenz (falls erforderlich) mit Compile-Zeit-geprüften Abfragen und eingebetteten Migrationen erhöht die Zuverlässigkeit der Datenbankinteraktion.

Die Einhaltung dieser Spezifikationen, insbesondere in den Bereichen Sicherheit, Einwilligungsmanagement und Systemintegration, ist entscheidend für den Erfolg und die Vertrauenswürdigkeit des Projekts.

# Tiefenanalyse des Model Context Protocol (MCP) für Standardisierte Plug-and-Play-Interaktionen mit LLMs unter Linux

## I. Einleitung

Die rasante Entwicklung von Large Language Models (LLMs) hat zu einer neuen Generation von KI-gestützten Anwendungen geführt. Diese Modelle besitzen beeindruckende Fähigkeiten zur Sprachverarbeitung und Generierung, sind jedoch oft von den Datenquellen und Werkzeugen isoliert, die für kontextbezogene und relevante Antworten in realen Szenarien notwendig sind.1 Jede Integration eines LLMs mit einem externen System – sei es eine Datenbank, eine API oder ein lokales Dateisystem – erforderte bisher oft maßgeschneiderte Implementierungen. Diese Fragmentierung behindert die Skalierbarkeit und Interoperabilität von KI-Systemen erheblich.

Als Antwort auf diese Herausforderung wurde Ende November 2024 von Anthropic das Model Context Protocol (MCP) vorgestellt.1 MCP ist ein offener Standard, der darauf abzielt, die Art und Weise zu vereinheitlichen, wie KI-Anwendungen, insbesondere solche, die auf LLMs basieren, mit externen Datenquellen, Werkzeugen und Diensten interagieren.3 Es fungiert als universelle Schnittstelle, vergleichbar mit einem „USB-C-Anschluss für KI-Anwendungen“ 3, und ermöglicht eine standardisierte Plug-and-Play-Konnektivität.

Dieser Bericht bietet eine Tiefenanalyse des Model Context Protocol, mit besonderem Fokus auf dessen Definition, technische Architektur, Kernkomponenten, Sicherheitsmechanismen und die spezifischen Aspekte der Integration in Desktop-Betriebssysteme, insbesondere Linux. Ziel ist es, eine umfassende Wissensquelle zu schaffen, die alle im MCP-Protokoll festgelegten Standards und Regeln detailliert darlegt.

## II. Grundlagen des Model Context Protocol (MCP)

### A. Definition und Zielsetzung

Das Model Context Protocol (MCP) ist ein **offener Standard**, der von Anthropic initiiert wurde, um die Verbindung zwischen KI-Modellen (wie LLMs) und externen Datenquellen sowie Werkzeugen zu standardisieren.1 Sein Hauptziel ist es, KI-Systeme aus ihrer Isolation zu befreien, indem es ihnen eine einheitliche Methode bietet, um auf relevanten Kontext zuzugreifen und Aktionen in anderen Systemen auszuführen.1 MCP definiert eine gemeinsame Sprache und einen Satz von Regeln für die Kommunikation, wodurch die Notwendigkeit entfällt, für jede Kombination aus KI-Anwendung und externem Dienst eine eigene Integrationslösung zu entwickeln.1 Es wird oft als „USB-C für KI“ beschrieben, da es eine universelle Schnittstelle bereitstellt, die es jeder KI-Anwendung ermöglicht, sich mit jeder Datenquelle oder jedem Dienst zu verbinden, der den MCP-Standard unterstützt, ohne dass dafür spezifischer Code erforderlich ist.3

### B. Problemstellung: Lösung der „M×N-Integrationsproblematik“

Vor der Einführung von MCP standen Entwickler vor dem sogenannten **„M×N-Integrationsproblem“**.3 Dieses Problem beschreibt die kombinatorische Komplexität, die entsteht, wenn _M_ verschiedene KI-Anwendungen oder LLMs mit _N_ verschiedenen externen Werkzeugen, Datenquellen oder Diensten verbunden werden müssen. Ohne einen gemeinsamen Standard müsste potenziell für jede der M×N Kombinationen eine individuelle, maßgeschneiderte Schnittstelle entwickelt und gewartet werden.3 Dies führt zu einem enormen Entwicklungsaufwand, erhöht die Fehleranfälligkeit und behindert die Skalierbarkeit und Wartbarkeit von KI-Systemen erheblich.1

MCP löst dieses Problem grundlegend, indem es die M×N-Komplexität in ein wesentlich einfacheres **M+N-Setup** umwandelt.3 Anstatt unzähliger Punkt-zu-Punkt-Integrationen müssen Werkzeuge (als MCP-Server) und KI-Anwendungen (als MCP-Clients/Hosts) nur einmalig den MCP-Standard implementieren. Sobald dies geschehen ist, kann prinzipiell jedes MCP-konforme Modell mit jedem MCP-konformen Werkzeug interagieren.3 Dies reduziert den Integrationsaufwand drastisch und fördert die Interoperabilität innerhalb des KI-Ökosystems.7

Durch die Definition eines standardisierten _Protokolls_ agiert MCP auf einer fundamentalen Kommunikationsebene. Diese Abstraktion ermöglicht es unterschiedlichen KI-Systemen und Werkzeugen, miteinander zu interagieren, ohne die internen Implementierungsdetails des jeweils anderen kennen zu müssen. Dies fördert nicht nur die Interoperabilität, sondern reduziert auch die Abhängigkeit von spezifischen Anbietern (Vendor Lock-in) und schafft die Grundlage für ein skalierbares und flexibles Ökosystem.7

### C. Entstehungskontext und frühe Anwender

MCP wurde Ende November 2024 von Anthropic, dem Unternehmen hinter der Claude-Familie von Sprachmodellen, initiiert und als Open-Source-Projekt veröffentlicht.1 Die Motivation war die Erkenntnis, dass selbst die fortschrittlichsten LLMs durch ihre Isolation von Echtzeitdaten und externen Systemen eingeschränkt sind.1 Anthropic positionierte MCP von Anfang an als kollaboratives Projekt, das auf die Beiträge der gesamten Community angewiesen ist, um ein breites Ökosystem zu fördern.1

Bereits kurz nach der Veröffentlichung zeigten sich frühe Anwender (Early Adopters), die das Potenzial von MCP erkannten und es in ihre Systeme integrierten. Dazu gehörten namhafte Unternehmen wie **Block** (ehemals Square) und **Apollo**, die MCP nutzten, um internen KI-Systemen den Zugriff auf proprietäre Wissensdatenbanken, CRM-Systeme und Entwicklerwerkzeuge zu ermöglichen.1 Auch Unternehmen aus dem Bereich der Entwicklerwerkzeuge wie **Zed, Replit, Codeium** und **Sourcegraph** begannen frühzeitig, mit MCP zu arbeiten, um die KI-Funktionen ihrer Plattformen zu verbessern, indem sie KI-Agenten einen besseren Zugriff auf relevanten Kontext für Programmieraufgaben ermöglichten.1 Diese frühe Validierung durch Industrieunternehmen unterstrich den praktischen Nutzen und die Relevanz des Protokolls.14

## III. Technische Architektur und Kernkomponenten

MCP basiert auf einer Client-Server-Architektur, die speziell für die sichere und standardisierte Kommunikation zwischen LLM-Anwendungen und externen Systemen konzipiert ist.3 Die Architektur umfasst drei Hauptkomponenten: Host, Client und Server.

### A. Das Client-Host-Server-Modell

1. **Host:**
    
    - **Definition:** Der Host ist die KI-gestützte Anwendung oder Agentenumgebung, mit der der Endbenutzer interagiert.3 Beispiele hierfür sind Desktop-Anwendungen wie Claude Desktop, IDE-Plugins (z. B. für VS Code), Chat-Schnittstellen oder jede benutzerdefinierte LLM-basierte Anwendung.3
    - **Rolle:** Der Host fungiert als Koordinator oder Container für eine oder mehrere Client-Instanzen.4 Er initiiert die Verbindungen zu MCP-Servern über die Clients.6 Entscheidend ist, dass der Host für die Verwaltung des Lebenszyklus der Client-Verbindungen und die Durchsetzung von Sicherheitsrichtlinien verantwortlich ist. Dazu gehören die Einholung der Zustimmung des Benutzers (Consent Management), die Benutzerautorisierung und die Verwaltung von Berechtigungen.4 Der Host überwacht auch, wie die KI- oder LLM-Integration innerhalb jedes Clients erfolgt, und führt bei Bedarf Kontextinformationen von mehreren Servern zusammen.3
2. **Client:**
    
    - **Definition:** Der Client ist eine Komponente oder Instanz, die innerhalb des Hosts läuft und als Vermittler für die Kommunikation mit _einem_ spezifischen MCP-Server dient.3
    - **Rolle:** Jeder Client verwaltet eine **1:1-Verbindung** zu einem MCP-Server.3 Diese Eins-zu-eins-Beziehung ist ein wichtiges Sicherheitsmerkmal, da sie die Verbindungen zu verschiedenen Servern voneinander isoliert (Sandboxing).3 Der Client ist für die Aushandlung der Protokollfähigkeiten mit dem Server verantwortlich und orchestriert den Nachrichtenaustausch (Anfragen, Antworten, Benachrichtigungen) gemäß dem MCP-Standard.4 Der Host startet für jeden benötigten Server eine eigene Client-Instanz.3
3. **Server:**
    
    - **Definition:** Ein MCP-Server ist ein (oft leichtgewichtiger) Prozess oder Dienst, der spezifische externe Datenquellen, Werkzeuge oder Fähigkeiten über das standardisierte MCP-Protokoll zugänglich macht.3 Server können lokal auf dem Rechner des Benutzers oder remote (z. B. in der Cloud oder im Unternehmensnetzwerk) laufen.8
    - **Rolle:** Der Server stellt dem verbundenen Client seine Fähigkeiten zur Verfügung. Diese Fähigkeiten werden durch die MCP-Primitive **Tools**, **Ressourcen** und **Prompts** definiert.3 Er empfängt Anfragen vom Client (z. B. zum Ausführen eines Tools oder zum Lesen einer Ressource), verarbeitet diese (indem er z. B. eine API aufruft, eine Datenbank abfragt oder auf lokale Dateien zugreift) und sendet die Ergebnisse oder Daten an den Client zurück.4

Die klare Trennung zwischen Host, Client und Server in der MCP-Architektur fördert die Modularität und Wiederverwendbarkeit. Ein einmal entwickelter MCP-Server kann von verschiedenen Hosts und Clients genutzt werden, und ein Host kann problemlos Verbindungen zu neuen Servern hinzufügen, um seine Fähigkeiten zu erweitern.8 Diese Struktur ist fundamental für die Lösung des M×N-Integrationsproblems.

### B. Kommunikationsprotokoll: JSON-RPC 2.0

MCP verwendet **JSON-RPC 2.0** als Nachrichtenformat für die gesamte Kommunikation zwischen Clients und Servern.4 JSON-RPC 2.0 ist ein leichtgewichtiger Standard für Remote Procedure Calls (RPC), der auf JSON (JavaScript Object Notation) basiert.

- **Nachrichtenstruktur:** Die Kommunikation erfolgt über strukturierte JSON-Nachrichten. MCP nutzt die drei von JSON-RPC 2.0 definierten Nachrichtentypen 21:
    
    - **Requests (Anfragen):** Nachrichten, die eine Operation auf der Gegenseite auslösen sollen und eine Antwort erwarten. Sie enthalten `jsonrpc: "2.0"`, eine eindeutige `id` (Zahl oder String), den `method` (Name der aufzurufenden Methode, z. B. `tools/call`) und optional `params` (ein strukturiertes Objekt oder Array mit den Parametern für die Methode).
    - **Responses (Antworten):** Nachrichten, die als Antwort auf eine Anfrage gesendet werden. Sie enthalten `jsonrpc: "2.0"`, die `id` der ursprünglichen Anfrage und entweder ein `result`-Feld (bei Erfolg) oder ein `error`-Objekt (bei einem Fehler).
    - **Notifications (Benachrichtigungen):** Nachrichten, die wie Anfragen eine Operation auslösen, aber keine Antwort erwarten. Sie enthalten `jsonrpc: "2.0"`, den `method` und optional `params`, aber keine `id`.
- **Vorteile:** Die Wahl von JSON-RPC 2.0 bietet mehrere Vorteile:
    
    - **Standardisierung:** Es ist ein etablierter Standard, was die Implementierung und Interoperabilität erleichtert.
    - **Lesbarkeit:** JSON ist menschenlesbar, was die Fehlersuche und Entwicklung vereinfacht.
    - **Leichtgewichtigkeit:** Es erzeugt relativ wenig Overhead im Vergleich zu anderen RPC-Mechanismen wie XML-RPC oder SOAP.
    - **Transportunabhängigkeit:** JSON-RPC 2.0 definiert das Nachrichtenformat, nicht den Transportmechanismus, was MCP Flexibilität bei der Wahl der Transportprotokolle gibt.26

Die Verwendung eines bewährten Standards wie JSON-RPC 2.0, der auch im Language Server Protocol (LSP) genutzt wird, von dem MCP Inspiration zog 6, unterstreicht das Ziel, eine robuste und interoperable Kommunikationsgrundlage zu schaffen.

### C. Transport Layer: STDIO und HTTP+SSE

MCP definiert, wie Nachrichten strukturiert sind (JSON-RPC 2.0), überlässt aber die Wahl des tatsächlichen Transportmechanismus für diese Nachrichten den Implementierungen. Die Spezifikation und die offiziellen SDKs unterstützen zwei primäre Transportmethoden 17:

1. **Standard Input/Output (STDIO):**
    
    - **Funktionsweise:** Bei diesem Transport startet der Host (oder der Client im Host) den MCP-Server als lokalen Kindprozess. Die Kommunikation erfolgt dann über die Standard-Eingabe (`stdin`) und Standard-Ausgabe (`stdout`) dieses Prozesses.17 JSON-RPC-Nachrichten werden über diese Pipes gesendet und empfangen, oft zeilenbasiert getrennt.22 Die Standard-Fehlerausgabe (`stderr`) wird häufig für Logging-Zwecke verwendet.22
    - **Anwendungsfälle:** STDIO eignet sich **ideal für lokale Integrationen**, bei denen Client und Server auf derselben Maschine laufen.17 Dies ist besonders relevant für die Integration in Desktop-Anwendungen (wie IDEs oder lokale KI-Assistenten unter Linux), die auf lokale Ressourcen zugreifen oder lokale Kommandozeilenwerkzeuge kapseln müssen.20
    - **Vorteile:** Einfachheit (keine Netzwerk-Konfiguration erforderlich), Effizienz (geringer Overhead für lokale Kommunikation), gute Integration mit bestehenden Kommandozeilen-Tools.19
    - **Sicherheitsaspekte:** Da die Kommunikation lokal erfolgt, sind die Hauptbedenken die Sicherheit des lokalen Systems und der beteiligten Prozesse. Ein Angreifer mit lokalem Zugriff könnte die Kommunikation potenziell abfangen oder manipulieren.26
2. **HTTP mit Server-Sent Events (SSE):**
    
    - **Funktionsweise:** Dieser Transportmechanismus ist für **Netzwerkkommunikation und Remote-Integrationen** konzipiert.17 Er verwendet eine Kombination aus Standard-HTTP-Methoden und Server-Sent Events:
        - **Client-zu-Server:** Der Client sendet JSON-RPC-Anfragen und -Benachrichtigungen über HTTP POST-Requests an den Server.17
        - **Server-zu-Client:** Der Server nutzt Server-Sent Events (SSE), einen Standard für unidirektionales Streaming vom Server zum Client über eine persistente HTTP-Verbindung, um JSON-RPC-Antworten und -Benachrichtigungen an den Client zu senden.17
    - **Anwendungsfälle:** Geeignet für Szenarien, in denen Client und Server über ein Netzwerk kommunizieren, z. B. wenn ein Desktop-Client auf einen zentral gehosteten Unternehmens-MCP-Server zugreift oder wenn MCP-Server als Webdienste bereitgestellt werden.18 Auch nützlich, wenn nur Server-zu-Client-Streaming benötigt wird oder in restriktiven Netzwerkumgebungen, die Standard-HTTP erlauben.26
    - **Vorteile:** Nutzt etablierte Web-Technologien, ermöglicht verteilte Architekturen, kann Firewalls oft leichter passieren als andere Protokolle.24
    - **Sicherheitsaspekte:** HTTP-basierte Transporte erfordern besondere Aufmerksamkeit bezüglich der Sicherheit:
        - **Transportverschlüsselung:** Die Verwendung von TLS (HTTPS) ist unerlässlich, um die Kommunikation abzusichern.22
        - **Authentifizierung/Autorisierung:** Da die Verbindung über ein potenziell unsicheres Netzwerk erfolgt, sind Mechanismen zur Authentifizierung des Clients und zur Autorisierung von Anfragen oft notwendig. MCP spezifiziert hierfür optional die Verwendung von OAuth 2.1 (siehe Abschnitt V.B).19
        - **DNS Rebinding:** SSE-Transporte können anfällig für DNS-Rebinding-Angriffe sein, insbesondere wenn lokale Server auf unsichere Weise an Netzwerkschnittstellen gebunden werden. Schutzmaßnahmen umfassen die Validierung des `Origin`-Headers, das Binden an `localhost` (127.0.0.1) statt `0.0.0.0` für lokale Server und die Implementierung von Authentifizierung.26

Die Wahl des Transports hängt vom spezifischen Anwendungsfall ab, wobei STDIO die natürliche Wahl für lokale Desktop-Integrationen (insbesondere unter Linux) darstellt, während HTTP+SSE für vernetzte Szenarien vorgesehen ist. Beide nutzen jedoch das gleiche JSON-RPC 2.0 Nachrichtenformat, was die Konsistenz des Protokolls über verschiedene Transportwege hinweg gewährleistet.19

### D. Kernprimitive des Protokolls

MCP definiert eine Reihe von Kernkonzepten, sogenannte „Primitive“, die die Art der Fähigkeiten beschreiben, die Server anbieten und Clients nutzen können. Diese Primitive strukturieren die Interaktion und ermöglichen es dem LLM bzw. der Host-Anwendung zu verstehen, welche Art von Kontext oder Funktionalität verfügbar ist.3

1. **Server-seitige Primitive (Angeboten vom Server):**
    
    - **Tools (Werkzeuge):**
        
        - **Definition:** Ausführbare Funktionen oder Aktionen, die das LLM (über den Client und Host) beim Server aufrufen kann.3 Tools repräsentieren typischerweise Operationen, die einen Zustand ändern können oder externe Systeme aktiv beeinflussen (z. B. eine E-Mail senden, einen Datenbankeintrag erstellen, eine Suche durchführen, Code ausführen).3
        - **Struktur:** Jedes Tool hat einen Namen, eine Beschreibung (die dem LLM hilft zu verstehen, wann es das Tool verwenden soll) und typischerweise ein definiertes Schema (oft JSON Schema) für seine Eingabeparameter und manchmal auch für die erwartete Ausgabe.7
        - **Verwendung:** Tools sind dafür gedacht, vom KI-Modell initiiert zu werden, wobei die Ausführung in der Regel die explizite Zustimmung des Benutzers erfordert (verwaltet durch den Host).6 MCP definiert JSON-RPC-Methoden wie `tools/list` (um verfügbare Tools auf einem Server zu entdecken) und `tools/call` (um ein bestimmtes Tool mit Parametern aufzurufen).7 Dieses Konzept ähnelt dem „Function Calling“ in anderen LLM-APIs, ist aber in MCP Teil eines breiteren, standardisierten Frameworks.7 Tools repräsentieren potenziell die Ausführung von beliebigem Code und MÜSSEN daher mit Vorsicht behandelt werden.6 Beschreibungen von Tools SOLLTEN als nicht vertrauenswürdig betrachtet werden, es sei denn, sie stammen von einem vertrauenswürdigen Server.6
    - **Resources (Ressourcen):**
        
        - **Definition:** Strukturierte Daten oder Kontextinformationen, die der Server dem Client (und damit dem LLM oder Benutzer) zur Verfügung stellt.3 Ressourcen sind in der Regel schreibgeschützt (read-only) und dienen dazu, den Kontext des LLMs anzureichern.7 Beispiele sind Dateiinhalte, Codefragmente, Datenbankeinträge, Log-Auszüge oder beliebige Informationen, die in den Prompt des Modells eingefügt werden können.3
        - **Struktur:** Ressourcen werden typischerweise über einen eindeutigen URI (Uniform Resource Identifier) identifiziert.29
        - **Verwendung:** Der Client kann Ressourcen vom Server anfordern (z. B. über eine Methode wie `resources/get` oder `read_resource` in den SDKs 29), um dem LLM relevante Informationen für seine aktuelle Aufgabe bereitzustellen. Der Host MUSS die Zustimmung des Benutzers einholen, bevor Benutzerdaten als Ressourcen an einen Server übermittelt oder von diesem abgerufen werden, und darf diese Daten nicht ohne Zustimmung weitergeben.6
    - **Prompts (Vorlagen):**
        
        - **Definition:** Vordefinierte Anweisungsvorlagen, Prompt-Templates oder Skripte für Arbeitsabläufe, die der Server dem Client anbieten kann, um komplexe Interaktionen zu steuern oder zu vereinfachen.3 Sie sind oft für den Benutzer oder den Host gedacht, um sie auszuwählen und anzuwenden.7
        - **Struktur:** Prompts können Argumente akzeptieren und potenziell mehrere Schritte verketten, z. B. eine Sequenz von Tool-Aufrufen oder Ressourcenabrufen spezifizieren.7
        - **Verwendung:** Sie dienen als wiederverwendbare „Rezepte“ für die Interaktion mit dem Server und dem LLM, um häufige Aufgaben zu erleichtern.7 Ein Beispiel wäre ein Prompt wie „Überprüfe diesen Code auf Fehler“, der intern möglicherweise ein Linter-Tool aufruft und relevante Dokumentation als Ressource abruft.7 Clients können verfügbare Prompts über eine Methode wie `prompts/list` abfragen.7
2. **Client-seitige Primitive (Angeboten vom Client an den Server):**
    
    - **Roots:**
        
        - **Definition:** Obwohl in einigen frühen Diskussionen oder Dokumenten erwähnt 3, wird das „Roots“-Primitive in der offiziellen Spezifikation 6 und den Kern-SDK-Dokumentationen 29 nicht explizit als eigenständiges, standardisiertes Primitiv für Client-Angebote definiert. Die ursprüngliche Idee 3 schien sich auf Einstiegspunkte in das Dateisystem oder die Umgebung des Hosts zu beziehen, auf die ein Server mit Erlaubnis zugreifen könnte. In der aktuellen Spezifikation wird der Zugriff auf lokale Ressourcen eher durch Server (die lokal laufen und Ressourcen anbieten) oder als Teil der allgemeinen Sicherheits- und Consent-Mechanismen des Hosts gehandhabt.
    - **Sampling (Stichprobennahme):**
        
        - **Definition:** Ein Mechanismus, der es dem _Server_ erlaubt, den _Host_ (über den Client) aufzufordern, eine Textvervollständigung durch das LLM basierend auf einem vom Server bereitgestellten Prompt zu generieren.3 Dies ermöglicht server-initiierte agentische Verhaltensweisen und rekursive oder verschachtelte LLM-Aufrufe.3
        - **Verwendung:** Dies ist eine fortgeschrittene Funktion, die komplexe, mehrstufige Denkprozesse ermöglichen kann, bei denen ein Agent auf der Serverseite das LLM im Host für Teilaufgaben aufrufen könnte.3
        - **Sicherheitsaspekte:** Anthropic betont, dass Sampling-Anfragen **immer die explizite Zustimmung des Benutzers erfordern MÜSSEN** 3, um unkontrollierte, sich selbst aufrufende Schleifen zu verhindern. Der Benutzer SOLLTE kontrollieren können, ob Sampling überhaupt stattfindet, welcher Prompt gesendet wird und welche Ergebnisse der Server sehen darf.6 Das Protokoll schränkt bewusst die Sichtbarkeit des Servers auf die Prompts während des Samplings ein.6

Diese Primitive bilden das Kernvokabular von MCP und ermöglichen eine strukturierte und standardisierte Art und Weise, wie LLM-Anwendungen sowohl Kontext (über Ressourcen und Prompts) abrufen als auch Aktionen (über Tools) auslösen können, wobei fortgeschrittene Interaktionsmuster (über Sampling) ebenfalls unterstützt werden.3

### E. Verbindungslebenszyklus

Die Interaktion zwischen einem MCP-Client und einem MCP-Server folgt einem definierten Lebenszyklus, der sicherstellt, dass beide Seiten über die Fähigkeiten des anderen informiert sind und die Kommunikation geordnet abläuft.18

1. **Initialisierung (Initialization):**
    
    - Der Prozess beginnt, wenn der Client eine Verbindung zum Server herstellt (über den gewählten Transportmechanismus).
    - Der Client sendet eine `initialize`-Anfrage an den Server. Diese Anfrage MUSS die vom Client unterstützte Protokollversion und optional dessen Fähigkeiten (z. B. Unterstützung für Sampling) enthalten.18
    - Der Server antwortet auf die `initialize`-Anfrage mit seiner eigenen unterstützten Protokollversion und einer Liste seiner Fähigkeiten (advertised capabilities), d. h. welche Tools, Ressourcen und Prompts er anbietet.18
    - Nachdem der Server geantwortet hat, sendet der Client eine `initialized`-Benachrichtigung an den Server, um zu bestätigen, dass der Handshake abgeschlossen ist und die normale Kommunikation beginnen kann.18
    - Dieser Aushandlungsprozess stellt sicher, dass beide Parteien kompatibel sind und die Fähigkeiten des Gegenübers kennen, bevor sie mit dem Austausch von Anwendungsdaten beginnen.18
2. **Nachrichtenaustausch (Message Exchange):**
    
    - Nach erfolgreicher Initialisierung können Client und Server Nachrichten gemäß dem JSON-RPC 2.0-Format austauschen.18
    - Dies umfasst Anfragen vom Client an den Server (z. B. `tools/call`, `resources/get`, `prompts/activate`), Anfragen vom Server an den Client (z. B. `sampling/request`, falls vom Client unterstützt und vom Benutzer genehmigt), die entsprechenden Antworten auf diese Anfragen sowie unidirektionale Benachrichtigungen in beide Richtungen (z. B. für Fortschritts-Updates oder Logging).6
3. **Beendigung (Termination):**
    
    - Die Verbindung kann auf verschiedene Weisen beendet werden 18:
        - **Sauberes Herunterfahren (Clean Shutdown):** Client oder Server können die Verbindung explizit und geordnet schließen (z. B. durch eine `shutdown`-Anfrage gefolgt von einer `exit`-Benachrichtigung, ähnlich wie im Language Server Protocol, oder spezifische Methoden im SDK).
        - **Transport-Trennung:** Eine Unterbrechung der zugrunde liegenden Transportverbindung (z. B. Schließen des STDIO-Streams, Trennung der HTTP-Verbindung) führt zur Beendigung der MCP-Sitzung.
        - **Fehlerbedingungen:** Kritische Fehler auf einer der beiden Seiten können ebenfalls zur sofortigen Beendigung der Verbindung führen.

Dieser klar definierte Lebenszyklus trägt zur Robustheit und Vorhersagbarkeit von MCP-Interaktionen bei.18

## IV. Implementierungspraktiken

Die Implementierung von MCP umfasst typischerweise das Erstellen von MCP-Servern, die externe Systeme kapseln, und die Integration von MCP-Clients in Host-Anwendungen, um diese Server zu nutzen.

### A. Erstellung von MCP-Servern

Das Erstellen eines MCP-Servers bedeutet, eine Brücke zwischen einem externen System (wie einer API, einer Datenbank oder dem lokalen Dateisystem) und dem MCP-Protokoll zu bauen.

- **Werkzeuge und SDKs:** Entwickler können MCP-Server erstellen, indem sie die offiziellen Software Development Kits (SDKs) nutzen, die von Anthropic und Partnern bereitgestellt werden. Diese SDKs sind für gängige Programmiersprachen wie **TypeScript, Python, Java, Kotlin, C# (in Zusammenarbeit mit Microsoft), Rust** und **Swift (in Zusammenarbeit mit loopwork-ai)** verfügbar.1 Die SDKs abstrahieren viele der Low-Level-Details des Protokolls (JSON-RPC-Handling, Transportmanagement) und bieten einfache Schnittstellen zur Definition von Server-Fähigkeiten.4 Alternativ kann das Protokoll auch direkt implementiert werden, basierend auf der Spezifikation.6 Die Verfügbarkeit dieser SDKs ist entscheidend für die Akzeptanz des Protokolls, da sie die Einstiegshürde für Entwickler erheblich senken. Ohne sie müssten Entwickler die Komplexität der Netzwerkprotokoll-Implementierung selbst bewältigen, einschließlich Nachrichten-Framing, Serialisierung, Transportbesonderheiten und Verbindungslebenszyklusmanagement.22 Die SDKs kapseln diese Komplexität und ermöglichen es Entwicklern, sich auf die Implementierung der eigentlichen Logik für ihre Tools, Ressourcen und Prompts zu konzentrieren, was die Erstellung neuer Server beschleunigt und das Wachstum des Ökosystems fördert.1
    
- **Prozess:**
    
    1. **Server-Instanziierung:** Ein Server-Objekt wird mithilfe des entsprechenden SDKs instanziiert (z. B. `FastMCP` in Python 29, `McpServer` in TypeScript 30).
    2. **Fähigkeiten definieren:** Tools, Ressourcen und Prompts werden mithilfe von Decorators (Python: `@mcp.tool()`, `@mcp.resource()`, `@mcp.prompt()` 25) oder spezifischen Methoden (TypeScript: `server.tool()`, `server.resource()`, `server.prompt()` 30) des SDKs definiert.
    3. **Logik implementieren:** Innerhalb der Funktionen, die diese Fähigkeiten definieren, wird die eigentliche Logik implementiert. Dies kann API-Aufrufe, Datenbankabfragen, Dateisystemoperationen oder andere Berechnungen umfassen.3
    4. **Server starten:** Der Server wird gestartet, um auf eingehende Verbindungen vom Client über den gewählten Transportmechanismus (STDIO oder HTTP+SSE) zu lauschen.
- **Beispiele:** Es gibt zahlreiche Referenzimplementierungen und Community-Beiträge für MCP-Server, die eine Vielzahl von Systemen integrieren, darunter Google Drive, Slack, GitHub, Git, Postgres, Puppeteer, Dateisystemzugriff, Shell-Ausführung und viele mehr.1 Diese dienen als Vorlagen und Bausteine für eigene Implementierungen.
    
- **Best Practices:** Bei der Entwicklung von Servern sollten bewährte Praktiken befolgt werden:
    
    - Klare und aussagekräftige Namen und Beschreibungen für Fähigkeiten verwenden.
    - Detaillierte Schemata für Tool-Parameter definieren (z. B. mit Zod in TypeScript 22).
    - Robuste Fehlerbehandlung implementieren.19
    - Tool-Operationen fokussiert und atomar halten.22
    - Rate Limiting implementieren, falls externe APIs genutzt werden.22
    - Umfassendes Logging implementieren (z. B. nach `stderr` bei STDIO 22 oder über `server.sendLoggingMessage()` 22).
    - Sicherheitsaspekte berücksichtigen: Eingabevalidierung und -sanitisierung, Schutz sensibler Daten.19
- **Debugging:** Werkzeuge wie der **MCP Inspector** können verwendet werden, um MCP-Server während der Entwicklung zu testen, zu inspizieren und zu validieren.8
    

### B. Integration von MCP-Clients

MCP-Clients sind die Komponenten innerhalb von Host-Anwendungen, die die tatsächliche Kommunikation mit den MCP-Servern durchführen.

- **Integration in Hosts:** Clients werden in Host-Anwendungen wie Claude Desktop, IDEs oder benutzerdefinierten Agenten integriert.3
    
- **Prozess:**
    
    1. **SDK verwenden:** Client-Bibliotheken aus den offiziellen SDKs werden genutzt (z. B. die `Client`-Klasse in TypeScript 30, `stdio_client` in Python 29).
    2. **Transport wählen:** Der passende Transportmechanismus (STDIO für lokale Server, HTTP+SSE für remote Server) wird ausgewählt und konfiguriert, um die Verbindung zum Zielserver herzustellen.23
    3. **Verbindung herstellen:** Eine Verbindung zum Server wird aufgebaut, und der Initialisierungs-Handshake (Aushandlung von Version und Fähigkeiten) wird durchgeführt.19
- **Interaktion mit Servern:**
    
    1. **Fähigkeiten entdecken:** Der Client kann die vom Server angebotenen Tools, Ressourcen und Prompts auflisten (z. B. über `list_tools`, `list_resources`, `list_prompts` 29).
    2. **Fähigkeiten nutzen:** Der Client ruft Tools auf (`tools/call` 29), liest Ressourcen (`resources/get` oder `read_resource` 29) oder aktiviert Prompts (`prompts/activate`) mithilfe der vom SDK bereitgestellten Methoden.
    3. **Antworten verarbeiten:** Der Client empfängt und verarbeitet die Antworten, Fehler und Benachrichtigungen vom Server und leitet sie gegebenenfalls an die Host-Anwendung oder das LLM weiter.19
- **Verantwortlichkeiten des Hosts:** Es ist wichtig zu verstehen, dass der Client selbst primär für die Protokollkommunikation zuständig ist. Die eigentliche Steuerung und Intelligenz liegt in der **Host-Anwendung**, die den Client einbettet.3 Der Host entscheidet, _welche_ Server wann verbunden werden sollen, basierend auf Benutzerinteraktionen oder der Logik des KI-Agenten. Er ist verantwortlich für die Verwaltung der Verbindungen und vor allem für die **Durchsetzung der Sicherheitsrichtlinien**. Dies umfasst das Einholen und Verwalten der **Benutzerzustimmung (Consent)** für den Zugriff auf Ressourcen oder die Ausführung von Tools.3 Der Host kann auch für die Abwicklung von Authentifizierungsflüssen (wie OAuth) verantwortlich sein und muss möglicherweise Kontextinformationen von mehreren verbundenen Servern integrieren und für das LLM oder den Benutzer aufbereiten.4 Der Host fungiert somit als zentrale Kontroll- und Sicherheitsebene, während der Client als gesteuerter Kommunikationskanal dient.
    

## V. Sicherheit und Governance in MCP

Sicherheit und Vertrauenswürdigkeit sind zentrale Aspekte des Model Context Protocol, insbesondere da es den Zugriff von KI-Modellen auf potenziell sensible Daten und die Ausführung von Aktionen in externen Systemen ermöglicht.3 Die Spezifikation legt daher großen Wert auf klare Sicherheitsprinzipien und -mechanismen.

### A. Fundamentale Sicherheitsprinzipien

Die MCP-Spezifikation 6 definiert mehrere Schlüsselprinzipien, die von allen Implementierern (Hosts und Server) beachtet werden MÜSSEN oder SOLLTEN:

- **Benutzerzustimmung und -kontrolle (User Consent and Control):**
    
    - Benutzer MÜSSEN explizit jeder Datenzugriffs- oder Tool-Ausführungsoperation zustimmen und deren Umfang verstehen.
    - Benutzer MÜSSEN die Kontrolle darüber behalten, welche Daten geteilt und welche Aktionen ausgeführt werden.
    - Hosts SOLLTEN klare Benutzeroberflächen zur Überprüfung und Autorisierung von Aktivitäten bereitstellen.
- **Datenschutz (Data Privacy):**
    
    - Hosts MÜSSEN explizite Benutzerzustimmung einholen, bevor Benutzerdaten an Server weitergegeben werden.
    - Ressourcendaten DÜRFEN NICHT ohne Benutzerzustimmung an andere Stellen übertragen werden.
    - Benutzerdaten SOLLTEN durch angemessene Zugriffskontrollen geschützt werden. MCP ermöglicht es, sensible Daten innerhalb der eigenen Infrastruktur zu halten, indem Server lokal oder im eigenen Netzwerk betrieben werden.7
- **Tool-Sicherheit (Tool Safety):**
    
    - Tools repräsentieren potenziell beliebige Codeausführung und MÜSSEN mit entsprechender Vorsicht behandelt werden.
    - Hosts MÜSSEN explizite Benutzerzustimmung einholen, bevor ein Tool aufgerufen wird.
    - Benutzer SOLLTEN verstehen, was jedes Tool tut, bevor sie dessen Verwendung autorisieren.
    - Beschreibungen des Tool-Verhaltens (z. B. Annotationen) SOLLTEN als nicht vertrauenswürdig betrachtet werden, es sei denn, sie stammen von einem vertrauenswürdigen Server.
- **Kontrolle über LLM-Sampling (LLM Sampling Controls):**
    
    - Benutzer MÜSSEN explizit allen LLM-Sampling-Anfragen vom Server zustimmen.
    - Benutzer SOLLTEN kontrollieren können, ob Sampling überhaupt stattfindet, welcher Prompt tatsächlich gesendet wird und welche Ergebnisse der Server sehen darf.
    - Das Protokoll schränkt die Sichtbarkeit des Servers auf die Prompts während des Samplings bewusst ein.

Obwohl das Protokoll selbst nicht alle diese Prinzipien auf Protokollebene erzwingen kann, SOLLTEN Implementierer robuste Zustimmungs- und Autorisierungsabläufe entwickeln, klare Dokumentationen der Sicherheitsimplikationen bereitstellen, angemessene Zugriffskontrollen und Datenschutzmaßnahmen implementieren, Sicherheitspraktiken befolgen und Datenschutzaspekte bei der Gestaltung von Funktionen berücksichtigen.6

Die detaillierte Ausformulierung dieser Prinzipien und die explizite Zuweisung von Verantwortlichkeiten, insbesondere an den Host, direkt in der Kernspezifikation 6 deuten darauf hin, dass Sicherheit und Benutzervertrauen von Anfang an zentrale Designziele waren. Angesichts der potenziellen Risiken, die mit der Verbindung leistungsfähiger KI-Modelle zu externen Systemen verbunden sind 2, ist dieser Fokus auf eine starke, transparente Sicherheitsgrundlage entscheidend für die Akzeptanz des Protokolls, insbesondere im Unternehmensumfeld.

### B. Authentifizierung und Autorisierung: OAuth 2.1 Integration

Für HTTP-basierte Transporte bietet MCP **optionale** Autorisierungsfähigkeiten auf Transportebene, die auf dem modernen **OAuth 2.1**-Standard basieren.27 Dies ermöglicht es MCP-Clients, Anfragen an geschützte MCP-Server im Namen von Ressourcenbesitzern (typischerweise Endbenutzern) zu stellen.

- **Rollen im OAuth-Fluss:**
    
    - **MCP-Server:** Agiert als OAuth 2.1 **Resource Server**, der geschützte Ressourcen (Tools, Ressourcen, Prompts) bereitstellt und Access Tokens validiert.
    - **MCP-Client:** Agiert als OAuth 2.1 **Client**, der im Namen des Benutzers Access Tokens von einem Authorization Server anfordert und diese bei Anfragen an den Resource Server (MCP-Server) mitsendet.
    - **Authorization Server:** Eine separate Entität (kann vom Server-Betreiber bereitgestellt werden), die Benutzer authentifiziert, deren Zustimmung einholt und Access Tokens (und ggf. Refresh Tokens) an den Client ausstellt.27
- **Unterstützte Grant Types:** MCP-Server SOLLTEN OAuth Grant Types unterstützen, die zum Anwendungsfall passen 27:
    
    - **Authorization Code Grant (mit PKCE):** Der empfohlene Fluss, wenn der Client im Namen eines menschlichen Endbenutzers handelt (z. B. ein KI-Agent ruft ein externes SaaS-Tool auf). **PKCE (Proof Key for Code Exchange) ist für alle Clients OBLIGATORISCH (REQUIRED)**, um Code Interception Attacks zu verhindern.27
    - **Client Credentials Grant:** Geeignet für Machine-to-Machine-Kommunikation, bei der der Client nicht im Namen eines Benutzers handelt (z. B. ein interner Agent ruft ein gesichertes internes Tool auf).27
- **Server Discovery und Client Registration:** Um die Interoperabilität und das Plug-and-Play-Ziel zu unterstützen, definiert die Spezifikation Mechanismen für Discovery und Registrierung:
    
    - **Server Metadata Discovery:** Clients MÜSSEN dem **OAuth 2.0 Authorization Server Metadata**-Protokoll (RFC8414) folgen, um Informationen über den Authorization Server zu erhalten (z. B. Endpunkte für Autorisierung und Token-Austausch).27 Server MÜSSEN entweder **OAuth 2.0 Protected Resource Metadata** (RFC9728, über den `WWW-Authenticate`-Header bei 401-Antworten) implementieren oder SOLLTEN RFC8414 unterstützen, um dem Client den Weg zum Authorization Server zu weisen.27 Fallback-URLs MÜSSEN unterstützt werden, falls keine Metadaten-Discovery verfügbar ist.28
    - **Dynamic Client Registration:** Clients und Authorization Servers SOLLTEN das **OAuth 2.0 Dynamic Client Registration Protocol** (RFC7591) unterstützen.27 Dies ermöglicht es Clients, sich automatisch bei neuen Authorization Servern zu registrieren und eine Client-ID zu erhalten, ohne dass manuelle Konfiguration durch den Benutzer erforderlich ist. Dies ist entscheidend für eine nahtlose Verbindung zu bisher unbekannten Servern.27 Ohne dynamische Registrierung müssten Clients möglicherweise auf hartcodierte IDs zurückgreifen oder den Benutzer auffordern, Registrierungsdetails manuell einzugeben.27
- **Token-Handhabung:**
    
    - Access Tokens MÜSSEN im `Authorization`-HTTP-Header als Bearer Token gesendet werden (`Authorization: Bearer <token>`).28 Sie DÜRFEN NICHT im URI-Query-String enthalten sein.28
    - Clients DÜRFEN KEINE Tokens an einen MCP-Server senden, die nicht vom zugehörigen Authorization Server dieses MCP-Servers ausgestellt wurden.27
    - Authorization Servers MÜSSEN sicherstellen, dass sie nur Tokens akzeptieren, die für ihre eigenen Ressourcen gültig sind.27 MCP-Server DÜRFEN KEINE anderen Tokens akzeptieren oder weiterleiten.27
    - Die Verwendung kurzlebiger Access Tokens wird EMPFOHLEN (RECOMMENDED), um die Auswirkungen gestohlener Tokens zu minimieren.27 Token-Rotation (mittels Refresh Tokens) SOLLTE implementiert werden.28
    - Clients MÜSSEN Tokens sicher speichern.27
- **Sicherheitsbest Practices:** Implementierungen MÜSSEN den Sicherheitspraktiken von OAuth 2.1 folgen.27 Dazu gehören die Verwendung von PKCE, die Validierung von Redirect URIs zur Verhinderung von Open Redirection Attacks und der Schutz vor Token-Diebstahl.27
    
- **Third-Party Authorization:** Die Spezifikation beschreibt auch Szenarien, in denen ein MCP-Server als Frontend für einen anderen Dienst fungiert, der seine eigene Authentifizierung erfordert (z. B. das Kapseln einer SaaS-API). Dies erfordert eine sichere Handhabung und Zuordnung von Tokens.28
    

Die Wahl von OAuth 2.1 als Standard für die optionale Autorisierung ist ein Schlüsselelement für die Interoperabilität und Unternehmensreife von MCP. Es bietet einen bekannten, robusten Rahmen, um den Zugriff zwischen potenziell heterogenen Clients und Servern abzusichern, ohne auf proprietäre Mechanismen angewiesen zu sein. Insbesondere die Unterstützung für dynamische Client-Registrierung unterstreicht die Vision eines flexiblen Plug-and-Play-Ökosystems, in dem Clients nahtlos und sicher mit neuen Diensten interagieren können, ohne dass umfangreiche manuelle Konfigurationen erforderlich sind.

### C. Zugriffskontroll- und Datenisolationsmechanismen

MCP implementiert Zugriffskontrolle und Isolation auf mehreren Ebenen, um die Sicherheit zu erhöhen:

- **Host-basierte Zustimmung:** Die primäre Kontrollebene ist der Host, der die explizite Zustimmung des Benutzers für den Zugriff auf Ressourcen und die Ausführung von Tools einholt.3 Dies stellt sicher, dass der Benutzer die ultimative Kontrolle behält.
- **Architektonische Isolation:** Das 1:1-Verhältnis zwischen Client und Server in der Architektur sorgt für eine natürliche Isolation (Sandboxing) zwischen verschiedenen Server-Verbindungen innerhalb des Hosts.3 Ein Client, der mit Server A verbunden ist, kann nicht auf die Ressourcen oder Daten zugreifen, die über einen anderen Client von Server B gehandhabt werden.4
- **OAuth Scopes:** Innerhalb des OAuth-Autorisierungsflusses können Scopes verwendet werden, um fein granulare Berechtigungen zu definieren und zu erzwingen. Der Authorization Server kann Tokens ausstellen, die nur den Zugriff auf bestimmte Aktionen oder Datenbereiche erlauben (impliziert durch OAuth-Nutzung, erwähnt in Fehlercodes für ungültige Scopes 27).
- **Server-seitige Logik:** MCP-Server können und sollten zusätzliche, anwendungsspezifische Zugriffskontrollen implementieren, basierend auf der Identität des authentifizierten Clients oder Benutzers, die über das OAuth-Token übermittelt wird.

Dieser mehrschichtige Ansatz (Host-Zustimmung, architektonische Isolation, transportbasierte Authentifizierung/Autorisierung via OAuth und server-seitige Logik) schafft eine robuste "Defense-in-Depth"-Strategie. Es wird erschwert, dass ein einzelner Fehlerpunkt das gesamte System kompromittiert, was die Gesamtsicherheit des MCP-Ökosystems stärkt.

### D. Zusammenfassung der Sicherheitsanforderungen

Die folgende Tabelle fasst die wesentlichen Sicherheitsanforderungen gemäß der MCP-Spezifikation und den referenzierten Standards zusammen und dient als Referenz für Implementierer und Prüfer. Die Schlüsselwörter MUSS (MUST), MUSS NICHT (MUST NOT), SOLLTE (SHOULD), SOLLTE NICHT (SHOULD NOT), KANN (MAY) sind gemäß RFC2119/RFC8174 zu interpretieren.6

|   |   |   |   |   |
|---|---|---|---|---|
|**Komponente**|**Kategorie**|**Spezifische Anforderung**|**Schlüsselwort**|**Standard / Referenz**|
|**Host**|Consent (Tool)|Explizite Benutzerzustimmung vor Tool-Aufruf einholen.|MUST|MCP Spec 6|
|**Host**|Consent (Resource)|Explizite Benutzerzustimmung vor Weitergabe von Benutzerdaten an Server einholen.|MUST|MCP Spec 6|
|**Host**|Consent (Sampling)|Explizite Benutzerzustimmung vor jeder Sampling-Anfrage einholen.|MUST|MCP Spec 6|
|**Host**|Data Privacy|Ressourcendaten nicht ohne Benutzerzustimmung an Dritte weitergeben.|MUST NOT|MCP Spec 6|
|**Host**|UI/UX|Klare UI für Überprüfung/Autorisierung von Aktivitäten bereitstellen.|SHOULD|MCP Spec 6|
|**Host/Client/Server**|General Security|Sicherheitspraktiken befolgen, Access Controls implementieren, Datenschutzaspekte berücksichtigen.|SHOULD|MCP Spec 6|
|**Server**|Tool Safety|Tool-Beschreibungen als nicht vertrauenswürdig betrachten (außer von vertrauenswürdigen Servern).|SHOULD|MCP Spec 6|
|**Client (HTTP)**|Authorization (PKCE)|PKCE für alle Authorization Code Grant Flows verwenden.|MUST|MCP Auth Spec 27, OAuth 2.1|
|**Client (HTTP)**|Authorization (Token)|Nur Tokens an Server senden, die vom zugehörigen Auth Server ausgestellt wurden.|MUST NOT|MCP Auth Spec 27|
|**Client (HTTP)**|Authorization (Token)|Access Tokens im Authorization Header senden (Bearer).|MUST|MCP Auth Spec 28|
|**Client (HTTP)**|Authorization (Token)|Access Tokens nicht im URI Query String senden.|MUST NOT|MCP Auth Spec 28|
|**Client (HTTP)**|Auth Discovery|RFC8414 zur Ermittlung von Auth Server Metadaten folgen.|MUST|MCP Auth Spec 27, RFC8414|
|**Client (HTTP)**|Dynamic Registration|RFC7591 für dynamische Client-Registrierung unterstützen.|SHOULD|MCP Auth Spec 27, RFC7591|
|**Server (HTTP)**|Auth Discovery|RFC9728 (via WWW-Authenticate) implementieren oder RFC8414 unterstützen. Fallbacks unterstützen, falls keine Metadaten-Discovery.|MUST/SHOULD|MCP Auth Spec 27, RFC9728/8414|
|**Server (HTTP)**|Authorization (Token)|Nur Tokens akzeptieren, die für eigene Ressourcen gültig sind.|MUST|MCP Auth Spec 27|
|**Server (HTTP)**|Authorization (Token)|Keine anderen Tokens akzeptieren oder weiterleiten.|MUST NOT|MCP Auth Spec 27|
|**Auth Server**|Dynamic Registration|RFC7591 für dynamische Client-Registrierung unterstützen.|SHOULD|MCP Auth Spec 27, RFC7591|
|**Auth Server**|Token Lifetime|Kurzlebige Access Tokens ausstellen.|SHOULD|MCP Auth Spec 27|
|**Auth Server**|Redirect URI|Redirect URIs exakt validieren (gegen vorregistrierte Werte).|MUST|MCP Auth Spec 27|
|**Client/Server (All)**|Transport Security|TLS für Remote-Verbindungen verwenden (impliziert für HTTP+SSE).|SHOULD/MUST|General Best Practice 22|
|**Client/Server (SSE)**|Transport Security|Origin Header validieren, nur an localhost binden (lokal), Authentifizierung implementieren (gegen DNS Rebinding).|MUST/SHOULD|MCP Transport Spec 26|

## VI. MCP-Integration in Desktop-Betriebssysteme (Linux-Fokus)

Ein Kernanliegen der Nutzeranfrage ist die standardisierte Integration von MCP in Desktop-Betriebssysteme, speziell Linux. MCP bietet durch seine Architektur und Transportmechanismen gute Voraussetzungen hierfür.

### A. Nutzung des STDIO-Transports für lokale Integration

Der **STDIO-Transport** ist der primäre und am besten geeignete Mechanismus für die Integration von MCP-Komponenten auf einem lokalen Desktop-System, einschließlich Linux.17

- **Funktionsweise unter Linux:** Eine Host-Anwendung (z. B. ein Desktop-KI-Assistent, eine IDE-Erweiterung) startet den MCP-Server als Kindprozess. Die Kommunikation erfolgt über die Standard-Datenströme (`stdin`, `stdout`), die unter Linux und anderen Unix-artigen Systemen ein fundamentaler Mechanismus für die Interprozesskommunikation (IPC) mittels Pipes sind.20 JSON-RPC-Nachrichten werden über diese Pipes ausgetauscht.22
- **Vorteile für Desktop-Integration:**
    - **Einfachheit:** Es ist keine Netzwerk-Konfiguration (Ports, Firewalls) erforderlich.19
    - **Effizienz:** Die lokale IPC über Pipes hat einen sehr geringen Overhead.19
    - **Kompatibilität:** Viele bestehende Linux-Tools und -Dienstprogramme sind Kommandozeilen-basiert und interagieren über STDIO, was die Kapselung als MCP-Server erleichtert.20
    - **Sicherheit:** Die Kommunikation bleibt auf die lokale Maschine beschränkt, was die Angriffsfläche im Vergleich zu Netzwerkdiensten reduziert (obwohl lokale Sicherheit weiterhin wichtig ist).

Der STDIO-Transport stellt somit eine natürliche Brücke dar, um MCP-Funktionalitäten in die lokale Linux-Desktop-Umgebung zu integrieren. Er ermöglicht es Host-Anwendungen, auf einfache und standardisierte Weise mit lokalen MCP-Servern zu kommunizieren, die Zugriff auf das Dateisystem, lokale Datenbanken oder andere Systemressourcen bieten.

### B. Beispiele für OS-interagierende MCP-Server unter Linux

Die Flexibilität von MCP zeigt sich in der Vielfalt der bereits existierenden Server, die direkt mit dem Betriebssystem interagieren. Viele dieser Beispiele sind plattformübergreifend oder leicht an Linux anpassbar:

- **Dateisystemzugriff:** Server, die Lese-, Schreib- und Auflistungsoperationen auf dem Dateisystem ermöglichen, oft mit konfigurierbaren Zugriffsbeschränkungen 33 ('Filesystem', 'Golang Filesystem Server'). Unter Linux würden diese auf Standard-POSIX-Dateisystem-APIs zugreifen.
- **Dateisuche:** Server, die systemeigene Suchwerkzeuge nutzen. Für Linux wird explizit die Verwendung von `locate` oder `plocate` erwähnt 33 ('Everything Search').
- **Shell-Ausführung:** Server, die die sichere Ausführung von Shell-Befehlen ermöglichen. Beispiele wie 'Terminal-Control' oder 'Windows CLI' 33 konzentrieren sich auf Windows, aber das Konzept ist direkt auf Linux übertragbar (z. B. durch Kapselung von `bash` oder anderen Shells). Projekte wie 'Lilith-Shell' 32 oder Container-basierte Code-Executor 32 demonstrieren dies.
- **Anwendungssteuerung:** Während AppleScript 33 macOS-spezifisch ist, könnten unter Linux ähnliche Server entwickelt werden, die z. B. über D-Bus (siehe unten) oder andere IPC-Mechanismen mit Desktop-Anwendungen interagieren. Browser-Automatisierung 32 und CAD-Steuerung 32 sind weitere Beispiele, die OS-Interaktion erfordern.

Diese Beispiele verdeutlichen, dass MCP nicht nur für den Zugriff auf Remote-APIs oder Datenbanken dient, sondern auch als **standardisierte und sichere Schnittstelle zu lokalen OS-Funktionen** fungieren kann. Anstatt LLMs direkt potenziell gefährliche Shell-Befehle generieren zu lassen, kann ein MCP-Server als Vermittler dienen. Das LLM fordert eine spezifische Aktion über ein MCP-Tool an (z. B. `filesystem/delete_file`), der Host holt die Benutzerzustimmung ein, und erst dann führt der Server die Aktion kontrolliert aus, möglicherweise mit zusätzlichen Sicherheitsprüfungen.6 MCP bietet somit einen sichereren Weg, die Fähigkeiten von LLMs mit den Möglichkeiten des Betriebssystems zu verbinden.

### C. Etablierung von Konventionen für die Linux-Desktop-Integration

Während MCP das _Kommunikationsprotokoll_ standardisiert, definiert es selbst keine spezifischen Konventionen dafür, _wie_ lokale Server auf einem Desktop-System wie Linux entdeckt, gestartet oder verwaltet werden sollen, oder wie gängige Desktop-Funktionen einheitlich abgebildet werden. Für eine nahtlose „Plug-and-Play“-Erfahrung sind jedoch solche Konventionen wahrscheinlich notwendig.

- **Aktueller Stand:** Die Entdeckung und Verwaltung lokaler Server ist oft anwendungsspezifisch. Claude Desktop beispielsweise erlaubt Benutzern das manuelle Hinzufügen von Servern.5
- **Potenzielle Konventionen (Diskussion):**
    - **Server Discovery:** Wie findet eine Host-Anwendung verfügbare lokale MCP-Server?
        - **Dateisystem-basiert:** Standardisierte Verzeichnisse (z. B. `~/.local/share/mcp-servers/` für Benutzer, `/usr/share/mcp-servers/` für systemweite Server) könnten Manifest-Dateien (z. B. im JSON- oder INI-Format) enthalten, die den Server beschreiben (Name, Fähigkeiten, Startbefehl für STDIO). Dies ähnelt dem Vorgehen bei `.desktop`-Dateien oder Systemd Unit-Files.
        - **Registrierungsdienst:** Ein zentraler Dienst (möglicherweise über D-Bus implementiert) könnte eine Liste verfügbarer Server verwalten.
    - **Server Management:** Wie werden lokale Server gestartet und gestoppt?
        - **On-Demand durch Host:** Der Host startet den Serverprozess bei Bedarf über STDIO und beendet ihn danach.23 Dies ist der einfachste Ansatz für STDIO-Server.
        - **Systemd User Services:** Für persistent laufende lokale Server könnten Systemd User Services genutzt werden.
        - **D-Bus Activation:** Falls eine D-Bus-Integration erfolgt, könnte dessen Aktivierungsmechanismus genutzt werden.34
    - **Standardisierte Schnittstellen:** Analog zu Freedesktop.org D-Bus-Schnittstellen (z. B. `org.freedesktop.Notifications`) könnten sich Community-Standards für MCP-Tool- und Ressourcen-Namen für gängige Desktop-Aufgaben entwickeln (z. B. `org.mcpstandard.FileManager.ReadFile`, `org.mcpstandard.Notifications.Send`). Dies würde die Interoperabilität zwischen verschiedenen Hosts und Servern, die ähnliche Funktionen anbieten, erheblich verbessern.

Die Erkenntnis hieraus ist, dass für eine echte Plug-and-Play-Integration auf dem Linux-Desktop wahrscheinlich **zusätzliche Konventionen über das Kern-MCP-Protokoll hinaus** erforderlich sind. Ähnlich wie Freedesktop.org-Standards die Interoperabilität im traditionellen Linux-Desktop ermöglichen, könnten solche Konventionen für MCP die Entdeckung, Verwaltung und konsistente Nutzung lokaler Server vereinfachen. Dies stellt einen Bereich für zukünftige Standardisierungsbemühungen oder die Etablierung von Best Practices durch die Community dar.

### D. Diskussion: MCP und D-Bus – Potenzielle Synergien und Herausforderungen

D-Bus ist der etablierte Standard für lokale IPC und Service-Messaging auf modernen Linux-Desktops.34 Er bietet Mechanismen für Methodenaufrufe, Signale (Events), Properties, Service Discovery und Aktivierung über zentrale Bus-Daemons (Session und System).34 Ein Vergleich mit MCP ergibt:

- **Ziele und Fokus:** Beide ermöglichen lokale IPC, aber mit unterschiedlichen Schwerpunkten. MCP ist speziell auf die Integration von KI/LLMs mit Kontext und Tools ausgerichtet, plattformübergreifend konzipiert und enthält KI-spezifische Primitive wie Sampling.3 D-Bus ist ein allgemeiner IPC-Mechanismus, primär für Linux.34
- **Potenzielle Synergien:**
    - **Discovery/Activation:** D-Bus könnte von MCP-Hosts genutzt werden, um lokal verfügbare MCP-Server zu finden (über registrierte D-Bus-Namen) oder sie bei Bedarf zu starten (D-Bus Activation), insbesondere für Server, die nicht über STDIO laufen.34
    - **Bridging:** Ein MCP-Server könnte als Brücke fungieren und bestehende D-Bus-Dienste als MCP-Tools/Ressourcen für einen KI-Host verfügbar machen. Umgekehrt könnte ein D-Bus-Dienst einen MCP-Client einbetten.
    - **Benachrichtigungen:** D-Bus-Signale könnten von lokalen MCP-Servern genutzt werden, um Hosts über asynchrone Ereignisse zu informieren, obwohl MCP selbst auch Benachrichtigungen unterstützt.
- **Herausforderungen:**
    - **Komplexität:** Eine Integration könnte zusätzliche Komplexität einführen.
    - **Mapping:** Die Abbildung von MCP-Primitiven auf D-Bus-Konzepte (Methoden, Signale, Properties) ist möglicherweise nicht immer direkt oder trivial.
    - **Plattformunabhängigkeit:** Eine starke Abhängigkeit von D-Bus könnte die Portierbarkeit von MCP-Hosts und -Servern auf andere Plattformen erschweren, was dem plattformübergreifenden Ziel von MCP widerspräche.13

MCP und D-Bus erscheinen eher als **komplementäre Technologien** denn als direkte Konkurrenten im Kontext der Linux-Desktop-Integration. MCP liefert das standardisierte, KI-zentrierte Kommunikationsprotokoll, während D-Bus etablierte Mechanismen für Service-Management (Discovery, Activation) und allgemeine IPC auf dem Linux-Desktop bietet. Eine durchdachte Integration könnte die Stärken beider Systeme nutzen, beispielsweise indem D-Bus für das Management lokaler MCP-Server verwendet wird, während die eigentliche Kommunikation über MCP (z. B. via STDIO) läuft. Ein direkter Ersatz des einen durch das andere erscheint unwahrscheinlich und für die jeweiligen Ziele nicht sinnvoll.

### E. Empfehlungen für standardisierte Linux-Integrationsmuster

Basierend auf der Analyse lassen sich folgende Empfehlungen für die Förderung einer standardisierten MCP-Integration unter Linux ableiten:

1. **Priorisierung von STDIO:** Die Verwendung des STDIO-Transports für lokale Linux-Desktop-Server sollte aufgrund seiner Einfachheit, Effizienz und Kompatibilität mit der Prozessverwaltung unter Linux als primärer Mechanismus empfohlen und gefördert werden.
2. **Dateisystem-basierte Discovery:** Eine einfache Konvention zur Server-Entdeckung mittels Manifest-Dateien in standardisierten Verzeichnissen (z. B. `~/.local/share/mcp-servers/`, `/usr/share/mcp-servers/`) sollte etabliert werden. Diese Manifeste sollten Metadaten über den Server und dessen Startmechanismus enthalten.
3. **Definition von Freedesktop-Style-Schnittstellen:** Die Community sollte ermutigt werden, gemeinsame MCP-Tool- und Ressourcen-Schnittstellen für Standard-Desktop-Aufgaben zu definieren (z. B. Dateiverwaltung, Benachrichtigungen, Kalenderzugriff), wobei eine Namenskonvention ähnlich zu D-Bus (z. B. `org.mcpstandard.Namespace.Operation`) verwendet werden könnte, um Interoperabilität zu fördern.
4. **Optionale D-Bus-Integration für Aktivierung:** Muster für die Nutzung von D-Bus zur Aktivierung von Servern (insbesondere für nicht-STDIO-Server oder komplexere Szenarien) könnten als optionale Erweiterung dokumentiert werden. Es sollte jedoch sichergestellt werden, dass die Kernfunktionalität für plattformübergreifende Kompatibilität auch ohne D-Bus erreichbar bleibt.

## VII. MCP in der Praxis: Anwendungsfälle und Beispiele

Die praktische Relevanz von MCP wird durch eine wachsende Zahl von Anwendungsfällen und Implementierungen in verschiedenen Bereichen unterstrichen.

### A. Workflow-Automatisierung

- **Meeting-Planung:** Ein KI-Assistent kann über einen MCP-Server für Google Calendar die Verfügbarkeit prüfen, Zeiten vorschlagen und Meetings planen.4
- **Echtzeit-Datenabfragen:** KI-Systeme können über MCP-Server auf Live-Daten aus Datenbanken wie Postgres zugreifen, um aktuelle Informationen in ihre Antworten einzubeziehen.1
- **Unternehmens-Chatbots:** Ein Chatbot kann über verschiedene MCP-Server hinweg Informationen aus unterschiedlichen internen Systemen (z. B. HR-Datenbank, Projektmanagement-Tool, Slack) in einer einzigen Konversation abrufen und kombinieren.3

### B. Verbesserung von Entwicklerwerkzeugen

- **Kontextbezogene Code-Generierung/-Überprüfung:** IDEs können über MCP-Server für GitHub oder Git auf den spezifischen Kontext eines Projekts (Repository-Inhalte, Issues) zugreifen, wodurch KI-Assistenten relevantere Code-Vorschläge oder Code-Reviews liefern können.1
- **Steuerung von CI/CD-Pipelines:** Integrationen mit Git-Servern über MCP können die Steuerung von Continuous Integration/Continuous Deployment-Prozessen ermöglichen.3
- **Integration in Entwicklungsplattformen:** Werkzeuge wie Zed, Replit, Codeium und Sourcegraph nutzen MCP, um ihre KI-Funktionen zu erweitern.1
- **Debugging-Werkzeuge:** Der MCP Inspector hilft Entwicklern beim Testen und Debuggen ihrer MCP-Server-Implementierungen.8

### C. Integration in Unternehmenssysteme

- **CRM-Zugriff:** KI-Agenten für Vertriebsmitarbeiter können über MCP auf CRM-Systeme wie HubSpot 10 oder Salesforce (impliziert) zugreifen, um Kundeninformationen abzurufen.
- **Kommunikationsanalyse:** MCP-Server für Plattformen wie Slack ermöglichen die Analyse und Priorisierung von Nachrichten.1
- **Interne Systeme bei Early Adopters:** Unternehmen wie Block (Square) und Apollo setzen MCP ein, um internen KI-Assistenten den Zugriff auf proprietäre Dokumente, Wissensdatenbanken, CRM-Daten und Entwicklerwerkzeuge zu ermöglichen.1
- **Zahlungsabwicklung:** Es existieren MCP-Server für die Integration mit Zahlungsdienstleistern wie PayPal.12

Die Breite dieser Anwendungsfälle – von persönlicher Produktivität über spezialisierte Entwicklerwerkzeuge bis hin zu komplexen Unternehmenssystemen – unterstreicht das Potenzial von MCP als universeller Integrationsstandard. Die Flexibilität der Architektur und der Primitive scheint ausreichend zu sein, um Interaktionen mit einer Vielzahl externer Systeme zu modellieren, was die Vision des „USB-C für KI“ 3 stützt und MCP nicht auf eine bestimmte Nische beschränkt.

## VIII. MCP im Vergleich: Kontext im Ökosystem

Um die Positionierung von MCP zu verstehen, ist ein Vergleich mit anderen Ansätzen zur Verbindung von LLMs mit externen Fähigkeiten sinnvoll.

### A. MCP vs. ChatGPT Plugins

- **Standardisierung:** MCP ist als offener, universeller Standard konzipiert, der modell- und anbieterunabhängig ist.7 ChatGPT Plugins sind hingegen spezifisch für das OpenAI-Ökosystem und basieren auf proprietären Spezifikationen.7
- **Architektur:** MCP nutzt eine Client-Server-Architektur, bei der der Host die Clients verwaltet.7 Plugins werden als vom Entwickler gehostete APIs implementiert, die von ChatGPT aufgerufen werden.7
- **Fähigkeiten:** MCP definiert klar die Primitive Tools, Ressourcen und Prompts.7 Plugins konzentrieren sich primär auf Tools (von OpenAI als „Functions“ bezeichnet).7
- **Sicherheit:** MCP legt den Fokus auf Host-seitige Benutzerzustimmung, Sandboxing und optionale OAuth 2.1-Integration.6 Die Sicherheit von Plugins hängt stärker von der Implementierung der Entwickler-API und dem Review-Prozess von OpenAI ab.7
- **Ökosystem:** MCP zielt auf ein breites, herstellerneutrales Ökosystem ab.7 Das Plugin-Ökosystem ist an die ChatGPT-Plattform gebunden.7

### B. MCP vs. LangChain

- **Standardisierung:** MCP ist ein **Kommunikationsprotokoll-Standard**.7 LangChain ist ein **Framework** und eine Bibliothek, kein Protokollstandard.7
- **Architektur:** MCP definiert die Kommunikation zwischen separaten Prozessen oder über Netzwerke (Client-Server).7 LangChain stellt Komponenten bereit, die direkt in den Code der KI-Anwendung integriert werden (Bibliotheks-Ansatz).7
- **Fähigkeiten:** MCP standardisiert die Primitive Tools, Ressourcen und Prompts als Teil des Protokolls.7 LangChain bietet Framework-Abstraktionen für Konzepte wie Tools, Agents, Chains und Prompt Templates.7
- **Sicherheit:** MCP implementiert Kontrollen auf Protokoll- und Host-Ebene (Zustimmung, OAuth).7 Bei LangChain liegt die Verantwortung für die sichere Nutzung externer Ressourcen beim Entwickler der Anwendung.7
- **Ökosystem:** MCP konzentriert sich auf interoperable Server und Clients.7 Das LangChain-Ökosystem fokussiert sich auf Framework-Komponenten, Integrationen und Vorlagen für den Aufbau von Anwendungen.7

### C. Analyse: Standardisierung, Offenheit, Fähigkeiten, Sicherheit

Das Hauptunterscheidungsmerkmal und der primäre Vorteil von MCP liegen in seinem Fokus darauf, ein **offener, interoperabler Protokollstandard** zu sein.1 Ziel ist es, KI-Anwendungen von spezifischen Werkzeugen und Plattformen zu entkoppeln.

MCP, ChatGPT Plugins und LangChain adressieren zwar ähnliche Probleme (Verbindung von LLMs mit externen Fähigkeiten), tun dies jedoch auf unterschiedlichen Ebenen oder mit unterschiedlichen Philosophien. Plugins erweitern eine spezifische Plattform (ChatGPT). LangChain bietet ein Framework zum _Bauen_ von Anwendungen, _innerhalb_ derer Integrationen stattfinden. MCP hingegen konzentriert sich auf die Standardisierung des **Kommunikationskanals** zwischen potenziell unterschiedlichen Systemen (Hosts und Servern). Dieser Fokus auf das "Wire Protocol" positioniert MCP einzigartig, um ein heterogenes Ökosystem zu fördern, in dem Komponenten von verschiedenen Anbietern oder Entwicklern zusammenarbeiten können.

Es besteht auch Potenzial für **Komplementarität**. Eine mit LangChain gebaute Anwendung könnte als MCP-Host fungieren und die Logik von LangChain-Agents nutzen, um Interaktionen mit externen Systemen über standardisierte MCP-Clients und -Server zu orchestrieren.12 Bestehende LangChain-Tools könnten als MCP-Server gekapselt werden. MCP definiert die _Schnittstelle_ (den Stecker), während Frameworks wie LangChain die _Logik_ hinter dem Agenten bereitstellen können, der diesen Stecker verwendet.

## IX. Das MCP-Ökosystem und zukünftige Richtungen

Seit seiner Einführung Ende 2024 hat MCP schnell an Dynamik gewonnen und ein wachsendes Ökosystem aufgebaut.

### A. Aktueller Stand: SDKs, Server-Repositories, Community-Beiträge

- **SDKs:** Offizielle SDKs sind für eine breite Palette von Sprachen verfügbar (TypeScript, Python, Java, Kotlin, C#, Rust, Swift), was die Entwicklung erleichtert.13 Einige davon werden in Zusammenarbeit mit wichtigen Akteuren der Branche wie Microsoft, JetBrains, Spring AI und loopwork-ai gepflegt.13
- **Server-Repositories:** Ein offizielles Repository (`modelcontextprotocol/servers`) enthält Referenzimplementierungen für gängige Systeme.1 Darüber hinaus katalogisieren Community-Listen wie "Awesome MCP Servers" Hunderte oder sogar Tausende von Servern 31, was auf ein schnelles Wachstum hindeutet.4
- **Community und Tooling:** MCP wird als offenes Projekt von Anthropic betrieben und ist offen für Beiträge.1 Es entstehen inoffizielle SDKs (z. B. für.NET 15) und ergänzende Werkzeuge.15 Der MCP Inspector ist ein wichtiges Werkzeug für das Debugging.8

### B. Adoption und Schlüsselakteure

- **Initiator:** Anthropic nutzt MCP selbst in seiner Claude Desktop App.1
- **Frühe Anwender:** Unternehmen wie Block, Apollo, Sourcegraph, Zed, Replit und Codeium haben MCP frühzeitig adaptiert.1
- **Breitere Akzeptanz:** Es gibt Berichte über eine Übernahme durch OpenAI und Google DeepMind 11 (wobei diese über die vorliegenden Quellen hinaus verifiziert werden müssten). Die Zusammenarbeit bei SDKs mit Microsoft, JetBrains und Spring AI 13 sowie Integrationen wie die von PayPal 12 deuten auf eine breitere Akzeptanz hin.

Das schnelle Wachstum von SDKs, Community-Servern und die Adoption durch diverse Unternehmen kurz nach dem Start deuten auf eine starke anfängliche Dynamik und einen wahrgenommenen Wert des Standards hin.1 Die Kollaborationen bei den SDKs sind besonders bemerkenswert, da sie MCP tief in populäre Entwicklungsökosysteme integrieren und signalisieren, dass MCP ein echtes Problem (das M×N-Problem 3) auf eine Weise löst, die bei Industrie und Community Anklang findet.

### C. Potenzielle Roadmap und zukünftige Erweiterungen

Offizielle, detaillierte Roadmap-Informationen sind in den analysierten Quellen begrenzt.35 Es gibt jedoch Hinweise und plausible Annahmen über zukünftige Entwicklungen:

- **Fokus auf Enterprise Deployment:** Anthropic hat Pläne für Entwickler-Toolkits zur Bereitstellung von Remote-Produktions-MCP-Servern für Unternehmenskunden (Claude for Work) erwähnt.1 Dies deutet auf einen Fokus hin, MCP für den stabilen, skalierbaren und managebaren Einsatz in Unternehmen zu härten.
- **Ökosystem-Reifung:** Zukünftige Arbeiten werden wahrscheinlich die Verbesserung der Entwicklererfahrung (bessere Werkzeuge, Dokumentation 4), die Erweiterung des Server-Ökosystems 4 und potenziell die Ergänzung von Funktionen für komplexere Orchestrierung oder Governance basierend auf Praxis-Feedback umfassen.
- **Mögliche neue Funktionen:** In frühen Planungsdokumenten wurden Ideen wie erweiterte Prompt-Vorlagen oder Multi-Server-Orchestrierung genannt (dies bleibt spekulativ ohne offizielle Bestätigung). Community-Vorschläge wie MCPHub als Discovery Service 15 könnten ebenfalls Einfluss nehmen.
- **Weitere SDKs:** Die Unterstützung weiterer Programmiersprachen ist denkbar.15

Die Weiterentwicklung wird sich wahrscheinlich darauf konzentrieren, MCP robuster für den Unternehmenseinsatz zu machen und das Ökosystem durch verbesserte Werkzeuge und eine wachsende Zahl von Servern weiter zu stärken. Die genauen Features werden sich vermutlich aus den Bedürfnissen der frühen Anwender und der Community ergeben.

## X. Fazit: MCP als fundamentaler Standard

Das Model Context Protocol (MCP) positioniert sich als eine potenziell transformative Technologie im Bereich der künstlichen Intelligenz. Durch die Bereitstellung eines **offenen, standardisierten Protokolls** adressiert es effektiv das **M×N-Integrationsproblem**, das bisher die nahtlose Verbindung von LLMs mit der Außenwelt behinderte.1

Die Kernvorteile von MCP liegen in der Förderung von **Interoperabilität**, der **Reduzierung von Entwicklungskomplexität** und der **Erhöhung der Flexibilität**, da Anwendungen und Werkzeuge unabhängig von spezifischen LLM-Anbietern oder Plattformen entwickelt werden können.3 Die klare Client-Host-Server-Architektur, gepaart mit definierten Primitiven (Tools, Ressourcen, Prompts) und Transportmechanismen (STDIO, HTTP+SSE), schafft eine robuste Grundlage für die Kommunikation.3

Besonders hervorzuheben ist der **integrierte Fokus auf Sicherheit und Governance**. Die Betonung der Benutzerzustimmung, die architektonische Isolation und die optionale Integration von modernen Standards wie OAuth 2.1 tragen dazu bei, Vertrauen aufzubauen und den Einsatz in sensiblen Umgebungen zu ermöglichen.6

Für die **Integration in Desktop-Betriebssysteme wie Linux** bietet MCP mit dem STDIO-Transport einen natürlichen und effizienten Mechanismus für lokale Interaktionen.17 Um jedoch das volle Potenzial einer Plug-and-Play-Erfahrung zu realisieren, sind wahrscheinlich zusätzliche Konventionen für die Server-Entdeckung und -Verwaltung sowie für standardisierte Schnittstellen für gängige Desktop-Aufgaben erforderlich, möglicherweise inspiriert von bestehenden Freedesktop.org-Standards.

MCP erleichtert die Entwicklung von **leistungsfähigeren, kontextbezogeneren und agentischeren KI-Anwendungen**, indem es ihnen einen universellen Zugang zu den benötigten externen Fähigkeiten ermöglicht.2 Die schnelle anfängliche Adoption und das wachsende Ökosystem deuten darauf hin, dass MCP das Potenzial hat, sich als **fundamentale Schicht für die nächste Generation integrierter KI-Systeme** zu etablieren.1 Sein langfristiger Erfolg wird jedoch von der kontinuierlichen Weiterentwicklung des Standards und vor allem von der breiten Annahme und den Beiträgen der Entwickler-Community abhängen.


# Planung und Spezifikation einer KI-gestützten Desktop-Sidebar für Manjaro Linux

## I. Einleitung

### Zweck

Dieses Dokument beschreibt den Entwurf und die Spezifikation für die Entwicklung einer neuartigen, KI-gesteuerten Desktop-Komponente für das Manjaro Linux-Betriebssystem. Das Kernziel ist die Schaffung eines intelligenten Assistenten, der als persistente Sidebar in die Desktop-Umgebung integriert ist. Die technologische Basis bilden C++, das Qt-Framework (insbesondere Qt 6), QML für die Benutzeroberfläche und Qt-Wayland für die nahtlose Integration in moderne Display-Server-Umgebungen.

### Vision

Die Vision ist eine transformative Benutzererfahrung, bei der ein stets präsenter KI-Assistent den Anwendern zur Seite steht. Dieser Assistent soll natürliche Sprache verstehen und darauf basierend Systemoperationen und Aktionen in Manjaro ausführen können. Dies umfasst das Starten von Anwendungen, die Verwaltung von Systemressourcen, die Abfrage von Informationen und die Interaktion mit Systemeinstellungen. Die Sidebar soll die Produktivität steigern und die Interaktion mit dem Manjaro-System intuitiver gestalten.

### Kerninnovation: Manjaro Control Protocol (MCP)

Ein zentrales Element dieses Projekts ist die Definition und Spezifikation des "Manjaro Control Protocol" (MCP). Dieses Protokoll dient als standardisierte Schnittstelle zwischen der KI (speziell dem Large Language Model, LLM) und der Systemsteuerungsschicht von Manjaro. Eine wesentliche Anforderung ist, dass das MCP so präzise und selbsterklärend definiert wird, dass ein LLM dessen Funktionsweise und Semantik _ausschließlich_ anhand der in diesem Bericht enthaltenen Spezifikation verstehen und korrekt anwenden kann, ohne auf externes Wissen, Trainingsdaten oder Internetzugriff angewiesen zu sein.

### Umfang des Berichts

Dieser Bericht deckt alle wesentlichen Aspekte der Planung und Spezifikation ab:

1. **Anforderungsanalyse:** Definition der Kernfunktionen und Interaktionen.
2. **Technologieintegration:** Untersuchung der Integration von Qt/QML und Qt-Wayland in Manjaro-Desktop-Umgebungen.
3. **Anwendungsarchitektur:** Entwurf der Softwarekomponenten und ihres Zusammenspiels.
4. **MCP-Spezifikation:** Detaillierte Definition des Kommunikationsprotokolls.
5. **LLM-Integration:** Strategien zur Einbindung eines LLM und Sicherstellung der MCP-Interpretierbarkeit.
6. **C++ Backend-Logik:** Details zur Implementierung der serverseitigen Logik.
7. **Sicherheitsaspekte:** Analyse potenzieller Risiken und Definition von Schutzmaßnahmen.
8. **Entwicklungs- & Testplan:** Grober Plan für Implementierung und Verifizierung.

### Zielgruppe

Dieses Dokument richtet sich an ein technisch versiertes Publikum, insbesondere an Softwarearchitekten, Systementwickler und Projektleiter, die an der Konzeption und Implementierung des beschriebenen Systems beteiligt sind. Es dient als detaillierte technische Grundlage für die Entwicklung.

## II. Anforderungsanalyse

Die erfolgreiche Entwicklung der KI-gestützten Sidebar erfordert eine klare Definition der funktionalen und nicht-funktionalen Anforderungen.

### A. Kernfunktionalität der Sidebar

- **Persistenz:** Die Sidebar muss als dauerhaftes Element der Desktop-Umgebung fungieren. Sie soll über virtuelle Desktops und Arbeitsbereiche hinweg sichtbar bleiben und eine konsistente Positionierung (z. B. am linken oder rechten Bildschirmrand) beibehalten. Dies erfordert eine tiefe Integration in die Shell-Protokolle des Wayland-Compositors, um sicherzustellen, dass die Sidebar korrekt positioniert wird und den benötigten Platz auf dem Bildschirm reserviert.
- **Benutzeroberfläche (UI):** Die UI, implementiert in QML, muss grundlegende Elemente zur Interaktion bereitstellen. Dazu gehören ein Eingabebereich für Anfragen in natürlicher Sprache, ein Ausgabebereich zur Darstellung der KI-Antworten und Ergebnisse sowie potenziell Statusindikatoren (z. B. für laufende Operationen oder Verbindungsstatus zum LLM).
- **Responsivität:** Die Benutzeroberfläche muss flüssig und reaktionsschnell sein. QML bietet hierfür die notwendigen Werkzeuge, um eine moderne und ansprechende User Experience zu gewährleisten, auch bei laufenden Hintergrundoperationen des Backends.

### B. Fähigkeiten des LLM

- **Verständnis natürlicher Sprache (NLU):** Das zugrundeliegende LLM muss in der Lage sein, Benutzeranfragen in natürlicher Sprache (initial Deutsch, mit potenzieller Erweiterbarkeit auf andere Sprachen) zu verarbeiten und deren Bedeutung zu erfassen.
- **Intentionerkennung:** Aus der Benutzeranfrage muss die Absicht (Intent) extrahiert werden. Beispiele für Intents sind das Öffnen einer Anwendung, das Abfragen von Systeminformationen oder das Ändern einer Einstellung.
- **MCP-Befehlsgenerierung:** Dies ist ein kritischer Schritt. Das LLM muss die erkannte Absicht und die extrahierten Parameter (z. B. Anwendungsname, Dateipfad, Lautstärkepegel) in einen syntaktisch und semantisch korrekten MCP-Befehl im JSON-Format übersetzen. Die Fähigkeit des LLM, dies _allein_ auf Basis der MCP-Spezifikation (Abschnitt V) zu tun, ist eine Kernanforderung.
- **Antwortinterpretation:** Das LLM muss strukturierte MCP-Antworten (JSON-Format), die vom Backend zurückkommen, verarbeiten können. Dies kann bedeuten, Fehlercodes zu interpretieren oder erfolgreiche Ergebnisdaten in eine natürlichsprachliche Antwort für den Benutzer umzuwandeln.
- **Kontextuelles Bewusstsein (Optional, aber empfohlen):** Für eine natürlichere Interaktion wäre es wünschenswert, wenn das LLM den Gesprächskontext über mehrere Anfragen hinweg beibehalten könnte. Der initiale Fokus liegt jedoch auf der Verarbeitung einzelner, in sich geschlossener Anfragen, die über MCP abgebildet werden.

### C. Umfang der Systeminteraktion

Die KI soll über das MCP eine Reihe von Systemfunktionen in Manjaro steuern können. Der initiale Satz umfasst:

- **Anwendungsmanagement:** Starten von Anwendungen (`open_application`). Das Schließen von Anwendungen ist optional und erfordert zusätzliche Überlegungen bezüglich der Prozessidentifikation und Berechtigungen.
- **Dateisystemoperationen:** Auflisten von Dateien und Verzeichnissen in einem bestimmten Pfad (`list_files`). Grundlegende Dateioperationen (Kopieren, Verschieben, Löschen) sind denkbar, erfordern jedoch eine sehr sorgfältige Sicherheitsanalyse und Implementierung (siehe Abschnitt VIII).
- **Systemeinstellungen:**
    - Abfragen allgemeiner Systeminformationen (`query_system_info`), z. B. Betriebssystemversion, CPU-/Speicherauslastung, Batteriestatus.
    - Ändern der Systemlautstärke (`change_volume`).
    - Anpassen der Bildschirmhelligkeit (`set_brightness`) über dedizierte Tools wie `brightnessctl`.1
    - Modifizieren spezifischer Desktop-Einstellungen, die über `dconf`/`gsettings` (für GNOME/GTK-basierte Umgebungen) zugänglich sind (`modify_setting_dconf`).3 Eine äquivalente Funktionalität für KDE Plasma (KConfig) muss separat betrachtet werden.
- **Paketverwaltung:** Interaktion mit dem Pamac-Kommandozeilenwerkzeug (`pamac`) zum Suchen, Installieren und Entfernen von Paketen sowie zur Update-Verwaltung (`manage_packages_pamac`). Die unterstützten Aktionen müssen klar definiert werden, basierend auf den Fähigkeiten der Pamac-CLI.8
- **Zwischenablage:** Kopieren von Text in die Zwischenablage (`clipboard_copy`) und Einfügen von Text aus der Zwischenablage (`clipboard_paste`). Unter Wayland erfordert dies spezielle Werkzeuge wie `wl-clipboard`.12

### D. Interaktionsfluss

Der typische Ablauf einer Benutzerinteraktion ist wie folgt:

1. Der Benutzer gibt eine Anfrage in natürlicher Sprache in die QML-Sidebar ein.
2. Das QML-Frontend sendet die reine Textanfrage an das C++ Backend.
3. Das Backend leitet die Anfrage an das LLM-Integrationsmodul weiter.
4. Das LLM-Modul sendet die Anfrage an das LLM (lokal oder API).
5. Das LLM analysiert die Anfrage, erkennt die Absicht und generiert einen entsprechenden MCP-Befehl im JSON-Format.
6. Das LLM (oder das LLM-Modul) sendet den MCP-Befehl (als JSON-String) zurück an das Backend.
7. Der MCP Interface Handler im Backend empfängt und validiert den MCP-Befehl gegen die Spezifikation.
8. Bei Erfolg parst der Handler den Befehl und ruft die entsprechende Funktion in der System Interaction Layer auf, wobei die Parameter übergeben werden.
9. Die System Interaction Layer führt die angeforderte Systemaktion aus (z. B. Starten eines Prozesses via `QProcess`, Senden einer DBus-Nachricht via `QDBus`).
10. Die System Interaction Layer empfängt das Ergebnis, den Status oder einen Fehler von der Systemaktion.
11. Das Backend (MCP Interface Handler) formatiert das Ergebnis in eine MCP-Antwort (JSON-Format).
12. Das Backend sendet die MCP-Antwort entweder zurück an das LLM-Modul (zur Interpretation und Umwandlung in natürliche Sprache) oder direkt an das QML-Frontend.
13. Das QML-Frontend zeigt die finale Antwort oder das Ergebnis dem Benutzer an.

### E. Zentrale Randbedingung: MCP-Verständnis

Die entscheidende Anforderung ist, dass das LLM lernen muss, das MCP _ausschließlich_ auf Basis der in Abschnitt V dieses Dokuments bereitgestellten Spezifikation zu verwenden. Es darf kein Vorwissen über MCP oder Manjaro-spezifische Interna vorausgesetzt werden, und es darf kein externer Zugriff (z. B. Internet) zur Klärung benötigt werden. Dies stellt hohe Anforderungen an die Klarheit, Vollständigkeit und Eindeutigkeit der MCP-Spezifikation.

## III. Technologieintegrationsstrategie (Qt/QML & Wayland unter Manjaro)

Die Wahl der Technologien und deren Integration ist entscheidend für die Realisierung der persistenten Sidebar und ihrer Funktionalität unter Manjaro, insbesondere im Kontext von Wayland.

### A. Qt/QML Framework

- **Begründung:** Qt (Version 6 wird für die beste Wayland-Unterstützung empfohlen) wird als primäres Framework gewählt. Es bietet leistungsstarke C++-Bibliotheken, exzellente Werkzeuge und mit QML eine deklarative Sprache zur effizienten Entwicklung moderner Benutzeroberflächen.15 Obwohl Qt plattformübergreifend ist, liegt der Fokus hier klar auf Manjaro Linux.
- **QML für das Frontend:** Die Sidebar-UI wird vollständig in QML implementiert. Dies ermöglicht eine schnelle Entwicklung, einfache Anpassung des Erscheinungsbilds und die Nutzung von Qt Quick Controls für Standard-UI-Elemente.17 Die Logik im QML-Teil wird minimal gehalten und konzentriert sich auf die Präsentation und die Weiterleitung von Benutzeraktionen an das C++ Backend.
- **C++ für das Backend:** Die Kernlogik der Anwendung, die Kommunikation mit dem LLM, die Verarbeitung von MCP-Nachrichten und die gesamte Systeminteraktion werden in C++ implementiert. Dies gewährleistet die notwendige Performance, Robustheit und den Zugriff auf systemnahe APIs und Bibliotheken.16

### B. Wayland-Integration

- **Qt-Wayland Modul:** Die Basis für den Betrieb der Qt-Anwendung als nativer Wayland-Client bildet das `qt6-wayland` Paket.21 Dieses Modul stellt die notwendige Abstraktionsebene für die Kommunikation mit dem Wayland-Compositor bereit.
- **Implementierung der persistenten Sidebar:**
    - **Kernprotokoll:** Das `wlr-layer-shell-unstable-v1` Protokoll ist der De-facto-Standard für die Erstellung von Desktop-Shell-Komponenten wie Panels, Docks und Sidebars unter Wayland-Compositors, die dieses Protokoll unterstützen.22 Dazu gehören Compositors, die auf `wlroots` basieren (z. B. Sway) und auch KWin (KDE Plasma).
    - **Wichtige `wlr-layer-shell` Merkmale 22:**
        - _Anchoring (Verankerung):_ Erlaubt das Festlegen der Sidebar an einem oder mehreren Bildschirmrändern (z. B. `left` oder `right`, optional auch `top` und `bottom` für volle Höhe).
        - _Layering (Ebenen):_ Weist die Sidebar einer bestimmten Ebene zu (z. B. `top` oder `overlay`), um ihre Sichtbarkeit relativ zu anderen Anwendungsfenstern zu steuern.
        - _Exclusive Zone (Exklusivbereich):_ Ermöglicht der Sidebar, einen Bereich des Bildschirms für sich zu reservieren, sodass maximierte Fenster diesen Bereich nicht überlappen. Dies ist entscheidend für eine persistente Sidebar.
        - _Keyboard Interactivity (Tastaturinteraktivität):_ Steuert, ob und wie die Sidebar Tastatureingaben empfangen kann. Der Modus `on_demand` ist typischerweise für interaktive Elemente wie eine Sidebar geeignet, die Texteingaben ermöglichen soll.
    - **Qt-Integrationsbibliothek:** Um die Nutzung von `wlr-layer-shell` aus einer Qt-Anwendung heraus zu vereinfachen, wird die Verwendung der `layer-shell-qt` Bibliothek empfohlen.23 Diese Bibliothek, ein KDE-Projekt, stellt die Klasse `LayerShellQt::Window` bereit, mit der die spezifischen Eigenschaften einer Layer-Shell-Oberfläche (Layer, Anker, Exklusivzone etc.) für ein `QWindow` verwaltet werden können. Die Verwendung dieser Bibliothek ist deutlich einfacher als die direkte Interaktion mit Wayland-Protokollen über die Qt Wayland Compositor APIs 24, welche primär für die Entwicklung von Compositors selbst gedacht sind.
    - **Technische Abwägung:** Die Analyse der verfügbaren Technologien 22 zeigt klar, dass `wlr-layer-shell` das geeignete Protokoll für die geforderte persistente Sidebar ist. Die Existenz von `layer-shell-qt` als dedizierte Client-Bibliothek für Qt vereinfacht die Implementierung erheblich. Daher ist dies der bevorzugte Ansatz.

### C. Kompatibilität mit Desktop-Umgebungen (Manjaro)

Die nahtlose Integration der Sidebar hängt stark von der verwendeten Desktop-Umgebung und deren Wayland-Unterstützung ab.

- **KDE Plasma:**
    - **Compositor:** KWin ist der Wayland-Compositor von Plasma.26 KWin's Wayland-Unterstützung gilt als ausgereift 27 und unterstützt das `wlr-layer-shell` Protokoll.
    - **Integration:** Da `layer-shell-qt` ein KDE-Projekt ist 23 und KWin das zugrundeliegende Protokoll unterstützt, ist eine gute Kompatibilität und eine vergleichsweise reibungslose Integration zu erwarten. Die Wayland-spezifische Integration in Qt-Anwendungen wird durch Komponenten wie `kwayland-integration` (für Qt5) bzw. dessen Nachfolger in `kwindowsystem` (für Qt6) unterstützt.29
    - **Strategische Implikation:** KDE Plasma stellt aufgrund der technologischen Nähe (Qt) und der Unterstützung des Schlüsselprotokolls (`wlr-layer-shell`) durch KWin den wahrscheinlichsten Pfad für eine erfolgreiche und vollständige Implementierung der Sidebar dar. Die Entwicklung sollte initial auf Plasma abzielen.
- **GNOME:**
    - **Compositor:** Mutter ist der Wayland-Compositor für GNOME.21
    - **Integrationsherausforderungen:** Mutter unterstützt das `wlr-layer-shell` Protokoll _nicht_ nativ.21 GNOME verwendet eigene Mechanismen für Panels und Docks, die oft als GNOME Shell Extensions implementiert sind. Historisch gab es Kompatibilitätsprobleme zwischen Mutter und Nicht-GTK-Wayland-Anwendungen 32, und Regressionen mit QtWayland wurden beobachtet.33 Zudem fehlt Mutter unter Wayland die Unterstützung für Server-Side Decorations (SSD), was das Erscheinungsbild von Qt-Anwendungen beeinflussen kann, da diese dann Client-Side Decorations (CSD) zeichnen müssen.31
    - **Mögliche Lösungsansätze:**
        1. _GNOME Shell Extension:_ Entwicklung einer separaten Erweiterung, die die QML-Sidebar hostet oder mit ihr interagiert. Dies ist komplex und erfordert Kenntnisse in JavaScript/GJS und der GNOME Shell Extension API.
        2. _Standard-Fenster:_ Ausführung der Sidebar als reguläres Wayland-Fenster. Die Persistenz, Positionierung und das Reservieren von Platz müssten programmatisch (und potenziell unzuverlässig) über Standard-Wayland-Fensterverwaltung versucht werden.
        3. _Abwarten auf Mutter-Entwicklung:_ Beobachten, ob zukünftige Mutter-Versionen relevante Protokolle unterstützen.30 Dies ist kurzfristig unwahrscheinlich für `wlr-layer-shell`.
    - **Strategische Implikation:** Die Integration in GNOME stellt eine erhebliche Herausforderung dar. Ohne `wlr-layer-shell`-Unterstützung 21 wird die Sidebar wahrscheinlich nicht die gewünschte Persistenz und Platzreservierung erreichen, es sei denn, es wird erheblicher Zusatzaufwand betrieben (z. B. Extension-Entwicklung). Es muss akzeptiert werden, dass die Funktionalität unter GNOME möglicherweise eingeschränkt ist oder eine abweichende Implementierungsstrategie erfordert.
- **XFCE:**
    - **Wayland-Status:** Die Umstellung von XFCE auf Wayland ist ein laufender Prozess. Standardmäßig könnte Manjaro XFCE noch X11 verwenden, wo Persistenz über Fenstermanager-Hints realisiert wird. Wenn XFCE unter Wayland läuft (z. B. über `xfce4-session-wayland`), hängt die Unterstützung für `wlr-layer-shell` vom verwendeten Compositor ab. Viele Wayland-Implementierungen für XFCE setzen auf `wlroots`-basierte Compositors, die `wlr-layer-shell` unterstützen.
    - **Strategische Implikation:** Die Kompatibilität hängt vom Compositor ab. Bei Verwendung eines `wlroots`-basierten Compositors ist der `layer-shell-qt`-Ansatz gangbar. Unter X11 wären traditionelle Xlib-Methoden nötig. Der Fokus sollte zunächst auf den primären Wayland-DEs Plasma und GNOME liegen.

### D. Mechanismen zur Systeminteraktion

Das C++ Backend wird verschiedene Mechanismen nutzen, um mit dem Manjaro-System zu interagieren:

- **`QProcess`:** Zum Ausführen von Kommandozeilenwerkzeugen und Skripten. Dies ist der primäre Mechanismus für Interaktionen mit `pamac` 8, `brightnessctl` 1, `wl-clipboard` (`wl-copy`/`wl-paste`) 12 und `gsettings`.4 Erfordert sorgfältige Handhabung von Argumenten, Parsing der Ausgabe (stdout/stderr) und strikte Sicherheitsvorkehrungen (siehe Abschnitte VII und VIII).34
- **`QDBus`:** Zur Kommunikation mit Systemdiensten und Desktop-Daemons, die eine DBus-Schnittstelle anbieten.39 Anwendungsfälle sind z. B. die Steuerung der Lautstärke (über PulseAudio/PipeWire), das Senden von Benachrichtigungen oder die Interaktion mit Energieverwaltungsdiensten (z. B. `org.gnome.SettingsDaemon.Power` 44 oder KDE-Äquivalente).
- **`dconf`/`gsettings`:** Zum Lesen und Schreiben von Konfigurationseinstellungen von GNOME/GTK-Anwendungen, die in der dconf-Datenbank gespeichert sind. Der Zugriff erfolgt am sichersten über das `gsettings`-Kommandozeilenwerkzeug (via `QProcess`), da dieses Schema-Validierungen durchführt.3 Für KDE-Einstellungen (KConfig) sind andere Mechanismen erforderlich (wahrscheinlich DBus oder direkte Konfigurationsdatei-Interaktion).
- **Direkter Datei-/API-Zugriff:** Für spezifische Low-Level-Informationen, wie z. B. das Lesen von Helligkeitswerten aus `/sys/class/backlight/` 2, obwohl die Verwendung von `brightnessctl` vorzuziehen ist. Erfordert sorgfältige Prüfung der Berechtigungen und Fehlerbehandlung.

## IV. Anwendungsarchitektur

Die Architektur der Anwendung folgt bewährten Praktiken für Qt/QML-Anwendungen und trennt klar zwischen Benutzeroberfläche, Anwendungslogik, LLM-Interaktion und Systeminteraktion.16

### A. Überblick

Die Architektur ist modular aufgebaut:

Code-Snippet

```
graph LR
    subgraph User Interface
        A
    end
    subgraph Backend (C++)
        B[Core Application Logic]
        C[LLM Integration Module]
        D[MCP Interface Handler]
        E
    end
    subgraph External Systems
        F
        G
    end

    A -- User Input --> B
    B -- Query --> C
    C -- Query --> F
    F -- MCP Command (JSON) --> C
    C -- MCP Command (JSON) --> D
    D -- Parsed Command --> E
    E -- System Call --> G
    G -- System Result/Error --> E
    E -- Result/Error --> D
    D -- MCP Response (JSON) --> B
    B -- Response Data/Formatted Response --> A
    A -- Display Output --> User

    D -- Validation Failure --> B  // Error path
```

_Diagramm-Beschreibung:_ Das Diagramm zeigt die Hauptkomponenten: QML Frontend, C++ Backend (unterteilt in Kernlogik, LLM-Modul, MCP-Handler, Systeminteraktionsschicht), LLM Service und Manjaro System. Pfeile illustrieren den Datenfluss von der Benutzereingabe über die Verarbeitung im Backend und LLM bis zur Systemaktion und der finalen Ausgabe.

### B. QML Frontend (Sidebar UI)

- **Verantwortlichkeiten:** Rendern der Sidebar-Oberfläche, Erfassen der Benutzereingabe (Text), Anzeigen von KI-Antworten und Statusinformationen, Handhabung von UI-Animationen und Übergängen.
- **Implementierung:** Hauptsächlich deklaratives QML, eventuell unter Verwendung von Qt Quick Controls für Standardelemente.17 Die Logik beschränkt sich auf Präsentationsaspekte und die Delegation von Aktionen an das C++ Backend.
- **Kommunikation:** Interagiert mit dem C++ Backend über Qt's Signal-Slot-Mechanismus und durch Zugriff auf C++-Objekte und deren Eigenschaften (`Q_PROPERTY`), die dem QML-Kontext bekannt gemacht werden.19

### C. C++ Backend

Das Backend ist das Herzstück der Anwendung und beherbergt die Kernlogik und die Schnittstellen zu externen Systemen.

- **1. Core Application Logic:**
    - Verwaltet den globalen Zustand der Anwendung.
    - Orchestriert die Kommunikation zwischen dem Frontend, dem LLM-Modul und der Systeminteraktionsschicht.
    - Initialisiert die Anwendung und macht die notwendigen C++-Objekte (insbesondere den MCP Interface Handler oder ein übergeordnetes Controller-Objekt) dem QML-Kontext zugänglich, z. B. über `QQmlContext::setContextProperty()`.19
- **2. LLM Integration Module:**
    - **Verantwortlichkeiten:** Kapselt die gesamte Logik für die Kommunikation mit dem ausgewählten LLM (ob lokal oder über eine API). Sendet die Benutzeranfragen (als Text) an das LLM und empfängt die generierten MCP-Befehle (als JSON-String). Optional kann es auch MCP-Antworten vom Backend an das LLM senden, um diese in natürliche Sprache formatieren zu lassen.
    - **Schnittstelle:** Definiert eine klare C++-Schnittstelle (z. B. eine Klasse mit Signalen und Slots) für das Senden von Anfragen und das Empfangen von strukturierten MCP-Befehls-Strings.
- **3. MCP Interface Handler:**
    - **Verantwortlichkeiten:** Nimmt die MCP-Befehls-JSON-Strings vom LLM-Modul entgegen. Validiert die JSON-Struktur und die Syntax des Befehls rigoros gegen die MCP-Spezifikation (Abschnitt V). Parst valide Befehle und leitet sie an die System Interaction Layer weiter. Empfängt strukturierte Ergebnisse oder Fehler von der System Interaction Layer und formatiert diese in MCP-Antwort-JSON-Strings.
    - **Implementierung:** Eine C++-Klasse, die JSON-Parsing (z. B. mit `QJsonDocument`, `QJsonObject`, `QJsonArray`) und die gesamte Validierungslogik gemäß der MCP-Spezifikation implementiert.
- **4. System Interaction Layer:**
    - **Verantwortlichkeiten:** Führt die konkreten Systemaktionen aus, die durch die geparsten MCP-Befehle spezifiziert wurden. Interagiert mit dem Manjaro-System über die geeigneten Mechanismen (`QProcess`, `QDBus`, `gsettings`-Aufrufe, Dateisystemzugriffe etc.). Kapselt die Details der jeweiligen Systeminteraktion, behandelt Fehler auf Systemebene und liefert standardisierte Ergebnisse oder Fehlercodes an den MCP Interface Handler zurück.
    - **Implementierung:** Modulare Struktur mit separaten C++-Klassen oder Funktionsgruppen für jeden Interaktionstyp (z. B. `PamacManager`, `SettingsManager`, `ProcessRunner`, `ClipboardManager`, `DBusInterface`). Diese Schicht abstrahiert die Komplexität der Systemaufrufe vom Rest des Backends.

### D. Best Practices für die Architektur

- **Trennung der Belange (Separation of Concerns):** Strikte Trennung zwischen der UI-Logik (QML) und der Backend-/Geschäftslogik (C++).16 Das QML-Frontend sollte "dumm" sein und nur Daten anzeigen und Benutzerereignisse weiterleiten.
- **Model-View(-Controller/Delegate):** Anwendung von MVC-, MVVM- oder ähnlichen Mustern, wo immer Daten aus dem Backend in der UI dargestellt werden. C++-Datenmodelle (abgeleitet von `QAbstractListModel` etc.) oder Kontext-Properties (`Q_PROPERTY`) werden dem QML-Frontend zur Verfügung gestellt.19 Änderungen im Backend werden über Signale an das Frontend gemeldet, das sich daraufhin aktualisiert.
- **Asynchrone Operationen:** Alle potenziell blockierenden Operationen – insbesondere Netzwerkaufrufe zum LLM, das Starten und Warten auf externe Prozesse mit `QProcess` 34 und DBus-Aufrufe – müssen asynchron implementiert werden, um ein Einfrieren der Benutzeroberfläche zu verhindern. Qt's Signal-Slot-Mechanismus ist hierfür das zentrale Werkzeug.

## V. Manjaro Control Protocol (MCP) Spezifikation

Das Manjaro Control Protocol (MCP) ist die definierte Schnittstelle, über die das LLM Systemaktionen anfordert und Ergebnisse empfängt. Die folgende Spezifikation ist darauf ausgelegt, von einem LLM ohne externes Wissen verstanden zu werden.

### A. Zweck und Designziele

- **Zweck:** Bereitstellung einer standardisierten, eindeutigen und maschinenlesbaren Schnittstelle, die es einer KI/einem LLM ermöglicht, spezifische Systemaktionen unter Manjaro Linux anzufordern und strukturierte Ergebnisse zu erhalten.
- **LLM-Interpretierbarkeit:** Explizit entworfen, um von einem LLM _allein_ auf Basis dieser Spezifikation verstanden und genutzt zu werden. Dies erfordert höchste Klarheit, explizite Definitionen aller Elemente und eine in sich geschlossene Beschreibung.
- **Plattformspezifität:** Zugeschnitten auf Manjaro Linux, unter Berücksichtigung spezifischer Werkzeuge (`pamac`), Konfigurationsmechanismen (`dconf`/`gsettings`) und Systempfade/Dienste.
- **Erweiterbarkeit:** Die Struktur (JSON-basiert, klare Befehlsdefinition) ermöglicht die zukünftige Ergänzung neuer Befehle, ohne die bestehende Struktur zu brechen.
- **Sicherheit:** Das Format unterstützt die Validierung und Bereinigung von Befehlen und Parametern durch das Backend, bevor eine Ausführung stattfindet.

### B. Nachrichtenformat

- **Transport:** JSON-Objekte werden sowohl für Anfragen (LLM -> Backend) als auch für Antworten (Backend -> LLM/Frontend) verwendet.
    
- **Anfragestruktur (Request):**
    
    JSON
    
    ```
    {
      "mcp_version": "1.0",
      "request_id": "string",
      "command": "string",
      "parameters": {
        "param1_name": "value1", // Typ: string | integer | boolean | array[string] | object
        "param2_name": "value2",
        //... weitere Parameter
      }
    }
    ```
    
    - `mcp_version` (string, erforderlich): Die Version des MCP-Protokolls, die verwendet wird (z. B. "1.0"). Dies ermöglicht zukünftige Versionierung.
    - `request_id` (string, erforderlich): Ein eindeutiger Identifikator für diese spezifische Anfrage, generiert vom anfragenden System (LLM-Modul). Wird verwendet, um Antworten der entsprechenden Anfrage zuzuordnen.
    - `command` (string, erforderlich): Der Name der auszuführenden Aktion (z. B. `open_application`, `query_system_info`). Muss exakt einem der im Core Command Set definierten Befehle entsprechen.
    - `parameters` (object, erforderlich): Ein JSON-Objekt, das die für den spezifischen `command` benötigten Parameter als Schlüssel-Wert-Paare enthält. Die Namen, Datentypen (string, integer, boolean, array von strings, etc.) und die Erforderlichkeit (required: true/false) jedes Parameters sind für jeden Befehl streng definiert (siehe Core Command Set).
- **Antwortstruktur (Response):**
    
    JSON
    
    ```
    {
      "mcp_version": "1.0",
      "request_id": "string",
      "status": "string", // "success" oder "error"
      "data": {... }, // Optional: Nur bei status="success"
      "error": {         // Optional: Nur bei status="error"
        "code": "string",
        "message": "string"
      }
    }
    ```
    
    - `mcp_version` (string, erforderlich): Die Version des MCP-Protokolls (z. B. "1.0").
    - `request_id` (string, erforderlich): Der eindeutige Identifikator aus der korrespondierenden Anfrage.
    - `status` (string, erforderlich): Gibt an, ob die Ausführung des Befehls erfolgreich war (`"success"`) oder fehlgeschlagen ist (`"error"`).
    - `data` (object, optional): Ein JSON-Objekt, das die Ergebnisse des Befehls enthält, falls `status` `"success"` ist. Die Struktur dieses Objekts hängt vom ausgeführten Befehl ab (z. B. eine Liste von Dateien, abgefragte Systeminformationen, eine Bestätigungsnachricht). Dieses Feld ist nur vorhanden, wenn `status` `"success"` ist.
    - `error` (object, optional): Ein JSON-Objekt, das nur vorhanden ist, wenn `status` `"error"` ist.
        - `code` (string, erforderlich): Ein vordefinierter Fehlercode-String, der die Art des Fehlers klassifiziert (z. B. `INVALID_COMMAND`, `PERMISSION_DENIED`, `EXECUTION_FAILED`, `TIMEOUT`, `INVALID_PARAMETER`). Eine Liste der Standard-Fehlercodes befindet sich am Ende dieses Abschnitts.
        - `message` (string, erforderlich): Eine menschenlesbare Beschreibung des Fehlers, primär für Logging- und Debugging-Zwecke. Diese Nachricht sollte vom LLM interpretiert werden, bevor sie dem Endbenutzer angezeigt wird.

### C. Definition des Kernbefehlssatzes (Core Command Set)

Die folgende Tabelle definiert die initialen Befehle, die das MCP unterstützt. Das LLM muss in der Lage sein, aus natürlicher Sprache auf diese Befehle zu schließen und die Anfragen gemäß den hier definierten Parametern zu strukturieren.

**Tabelle: MCP Core Commands (Version 1.0)**

|   |   |   |   |   |
|---|---|---|---|---|
|**Command Name (string)**|**Description**|**Parameters (object: {name: {type, required, description}})**|**Success Data Structure (object)**|**Potential Error Codes (array[string])**|
|`open_application`|Startet eine Desktop-Anwendung.|`{"name": {"type": "string", "required": true, "description": "Name oder ausführbarer Pfad der Anwendung (z.B. 'firefox', '/usr/bin/gimp')."}}`|`{"pid": {"type": "integer", "description": "Prozess-ID der gestarteten Anwendung (optional, falls ermittelbar)"}, "message": {"type": "string", "description": "Bestätigungsnachricht, z.B. 'Anwendung [Name] gestartet.'"}}`|`EXECUTION_FAILED`, `APP_NOT_FOUND`, `INVALID_PARAMETER`|
|`list_files`|Listet Dateien und Verzeichnisse in einem Pfad auf.|`{"path": {"type": "string", "required": true, "description": "Absoluter Pfad zum Verzeichnis (muss mit '/' beginnen). Symbolische Links werden nicht aufgelöst."}}`|`{"path": {"type": "string", "description": "Der abgefragte Pfad"}, "entries": {"type": "array", "items": {"type": "object", "properties": {"name": {"type": "string"}, "type": {"type": "string", "enum": ["file", "directory"]}, "size_bytes": {"type": "integer", "description": "Dateigröße in Bytes (nur für Typ 'file')"}}}}, "description": "Liste der Einträge im Verzeichnis."}}`|`PATH_NOT_FOUND`, `PERMISSION_DENIED`, `INVALID_PARAMETER`, `FILESYSTEM_ERROR`|
|`change_volume`|Stellt die Systemlautstärke ein oder passt sie an.|`{"level": {"type": "integer", "required": false, "description": "Absoluter Lautstärkepegel in Prozent (0-100)."}, "change": {"type": "integer", "required": false, "description": "Relative Änderung in Prozentpunkten (+/-). 'level' und 'change' schließen sich gegenseitig aus."}}`|`{"new_level": {"type": "integer", "description": "Der resultierende Lautstärkepegel in Prozent (0-100)."}}`|`INVALID_PARAMETER`, `EXECUTION_FAILED`, `DBUS_ERROR` (falls DBus verwendet)|
|`query_system_info`|Ruft spezifische Systeminformationen ab.|`{"query": {"type": "string", "required": true, "description": "Art der angeforderten Information. Gültige Werte: 'os_version', 'hostname', 'cpu_usage' (als Prozentwert), 'memory_total_mb', 'memory_available_mb', 'memory_usage' (als Prozentwert), 'battery_status' (als Objekt mit 'percentage', 'charging_status' [boolean]), 'uptime_seconds'."}}`|`{"query": {"type": "string", "description": "Die gestellte Abfrage"}, "value": {"type": "string|integer|
|`manage_packages_pamac`|Interagiert mit dem Pamac CLI zur Paketverwaltung.9|`{"action": {"type": "string", "required": true, "enum": ["search", "install", "remove", "update_check", "update_all", "list_installed", "list_orphans", "remove_orphans"], "description": "Die auszuführende Pamac-Aktion."}, "package_name": {"type": "string", "required": false, "description": "Ziel-Paketname (erforderlich für 'install', 'remove', 'search')."}, "include_aur": {"type": "boolean", "required": false, "default": false, "description": "AUR in die Aktion einbeziehen ('search', 'update_check', 'update_all')."}}`|Hängt von `action` ab: `search`: `{"results": array[{"name": string, "version": string, "repository": string, "description": string}]}`. `install`/`remove`: `{"message": string}`. `update_check`: `{"updates_available": boolean, "packages": array[string]}`. `update_all`: `{"message": string}`. `list_installed`/`list_orphans`: `{"packages": array[string]}`. `remove_orphans`: `{"message": string}`.|`PAMAC_ERROR`, `INVALID_ACTION`, `INVALID_PARAMETER`, `PACKAGE_NOT_FOUND`, `PERMISSION_DENIED`, `EXECUTION_FAILED`|
|`modify_setting_dconf`|Ändert eine dconf-Einstellung (primär für GNOME/GTK) via `gsettings`.4|`{"schema": {"type": "string", "required": true, "description": "Das GSettings-Schema (z.B. 'org.gnome.desktop.interface')."}, "key": {"type": "string", "required": true, "description": "Der Schlüssel innerhalb des Schemas (z.B. 'gtk-theme')."}, "value": {"type": "string|integer|boolean", "required": true, "description": "Der neue Wert für den Schlüssel. Muss dem Typ des Schlüssels im Schema entsprechen."}}`|
|`set_brightness`|Passt die Bildschirmhelligkeit an via `brightnessctl`.1|`{"level": {"type": "integer", "required": true, "description": "Absoluter Helligkeitspegel in Prozent (0-100)."}}`|`{"new_level": {"type": "integer", "description": "Der resultierende Helligkeitspegel in Prozent (0-100)."}}`|`INVALID_PARAMETER`, `EXECUTION_FAILED`, `BRIGHTNESS_CONTROL_ERROR`|
|`clipboard_copy`|Kopiert Text in die System-Zwischenablage via `wl-copy`.12|`{"text": {"type": "string", "required": true, "description": "Der zu kopierende Text."}}`|`{"message": {"type": "string", "description": "Text erfolgreich kopiert."}}`|`EXECUTION_FAILED`, `CLIPBOARD_ERROR`|
|`clipboard_paste`|Ruft Text aus der System-Zwischenablage ab via `wl-paste`.12|`{}` (Keine Parameter benötigt)|`{"text": {"type": "string", "description": "Der Text aus der Zwischenablage."}}`|`EXECUTION_FAILED`, `CLIPBOARD_EMPTY`, `CLIPBOARD_ERROR`|

_Anmerkung zur LLM-Interpretierbarkeit:_ Die `description`-Felder in der Tabelle sind entscheidend. Sie liefern dem LLM den notwendigen Kontext, um die Parameter korrekt zu interpretieren und zu befüllen (z. B. was unter `os_version` bei `query_system_info` zu verstehen ist oder welche Werte für `action` bei `manage_packages_pamac` gültig sind). Die `enum`-Angabe bei `action` und `type` (in `list_files`) schränkt die möglichen Werte explizit ein.

### D. Kommunikationsfluss

1. **Anfragegenerierung:** Das LLM empfängt die natürlichsprachliche Anfrage des Benutzers, analysiert sie und identifiziert den passenden MCP-Befehl sowie die erforderlichen Parameter gemäß der obigen Tabelle. Es konstruiert das MCP Request JSON-Objekt, inklusive einer eindeutigen `request_id`.
2. **Anfrageübermittlung:** Das LLM-Modul sendet den JSON-String an den MCP Interface Handler im C++ Backend.
3. **Validierung im Backend:** Der MCP Interface Handler parst den JSON-String. Er überprüft die `mcp_version`, die Gültigkeit des `command`-Namens und ob alle erforderlichen `parameters` vorhanden sind und den korrekten Datentyp haben. Bei Fehlern wird sofort eine MCP Error Response generiert und zurückgesendet.
4. **Dispatching:** Bei erfolgreicher Validierung ruft der MCP Interface Handler die zuständige Methode in der System Interaction Layer auf und übergibt die extrahierten und validierten Parameter.
5. **Systemaktion:** Die System Interaction Layer führt die Aktion aus (z. B. Starten eines `QProcess`, Senden einer `QDBus`-Nachricht). Dies geschieht asynchron.
6. **Ergebnisverarbeitung:** Nach Abschluss der Systemaktion (erfolgreich oder fehlerhaft) meldet die System Interaction Layer das Ergebnis (Daten oder Fehlercode/-nachricht) an den MCP Interface Handler zurück.
7. **Antwortgenerierung:** Der MCP Interface Handler konstruiert das MCP Response JSON-Objekt. Er füllt `request_id` (aus der Anfrage), `status` (`success` oder `error`) und entweder das `data`-Objekt (bei Erfolg) oder das `error`-Objekt (bei Fehler) gemäß der Spezifikation.
8. **Antwortübermittlung:** Der JSON-String der Antwort wird zurück an das LLM-Modul oder direkt an das Frontend gesendet.

### E. Konkrete Beispiele (Request/Response Paare)

- **Beispiel 1: Firefox starten**
    - Request:
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-123",
          "command": "open_application",
          "parameters": {
            "name": "firefox"
          }
        }
        ```
        
    - Response (Success):
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-123",
          "status": "success",
          "data": {
            "pid": 12345,
            "message": "Anwendung firefox gestartet."
          }
        }
        ```
        
- **Beispiel 2: Dateien im Home-Verzeichnis auflisten**
    - Request:
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-124",
          "command": "list_files",
          "parameters": {
            "path": "/home/user"
          }
        }
        ```
        
    - Response (Success):
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-124",
          "status": "success",
          "data": {
            "path": "/home/user",
            "entries": [
              {"name": "Documents", "type": "directory"},
              {"name": "image.jpg", "type": "file", "size_bytes": 102400},
              {"name": ".bashrc", "type": "file", "size_bytes": 3500}
            ]
          }
        }
        ```
        
- **Beispiel 3: Pamac nach 'gimp' durchsuchen (inkl. AUR)**
    - Request:
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-125",
          "command": "manage_packages_pamac",
          "parameters": {
            "action": "search",
            "package_name": "gimp",
            "include_aur": true
          }
        }
        ```
        
    - Response (Success):
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-125",
          "status": "success",
          "data": {
            "results":
          }
        }
        ```
        
- **Beispiel 4: Helligkeit auf 75% setzen**
    - Request:
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-126",
          "command": "set_brightness",
          "parameters": {
            "level": 75
          }
        }
        ```
        
    - Response (Success):
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-126",
          "status": "success",
          "data": {
            "new_level": 75
          }
        }
        ```
        
- **Beispiel 5: Fehler beim Installieren eines nicht existierenden Pakets**
    - Request:
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-127",
          "command": "manage_packages_pamac",
          "parameters": {
            "action": "install",
            "package_name": "nonexistent_package_xyz"
          }
        }
        ```
        
    - Response (Error):
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-127",
          "status": "error",
          "error": {
            "code": "PACKAGE_NOT_FOUND",
            "message": "Pamac Fehler: Ziel nicht gefunden: nonexistent_package_xyz"
          }
        }
        ```
        

### F. Fehlerbehandlung und Fehlercodes

Eine robuste Fehlerbehandlung ist essenziell. Das Backend muss Fehler auf verschiedenen Ebenen abfangen und in standardisierte MCP-Fehlercodes übersetzen.

- **Standard-Fehlercodes:**
    - `INVALID_COMMAND`: Der angegebene `command` ist nicht im MCP definiert.
    - `INVALID_PARAMETER`: Ein oder mehrere Parameter sind ungültig (falscher Typ, fehlender erforderlicher Parameter, ungültiger Wert, z. B. Pfad existiert nicht, wo erwartet).
    - `PERMISSION_DENIED`: Die Aktion erfordert höhere Berechtigungen, die der Backend-Prozess nicht hat.
    - `EXECUTION_FAILED`: Ein externer Prozess (`QProcess`) konnte nicht gestartet werden oder ist mit einem Fehler abgestürzt.
    - `TIMEOUT`: Eine Operation hat das Zeitlimit überschritten.
    - `APP_NOT_FOUND`: Die zu startende Anwendung wurde nicht gefunden.
    - `PATH_NOT_FOUND`: Ein angegebener Datei- oder Verzeichnispfad existiert nicht.
    - `FILESYSTEM_ERROR`: Allgemeiner Fehler bei Dateisystemoperationen.
    - `DBUS_ERROR`: Fehler bei der Kommunikation über DBus.
    - `PAMAC_ERROR`: Spezifischer Fehler bei der Interaktion mit Pamac CLI.
    - `GSETTINGS_ERROR`: Spezifischer Fehler bei der Interaktion mit `gsettings` CLI.
    - `BRIGHTNESS_CONTROL_ERROR`: Spezifischer Fehler bei der Helligkeitssteuerung.
    - `CLIPBOARD_ERROR`: Allgemeiner Fehler bei der Interaktion mit der Zwischenablage.
    - `CLIPBOARD_EMPTY`: Versuch, aus einer leeren Zwischenablage zu lesen.
    - `INVALID_QUERY`: Der Wert für `query` in `query_system_info` ist ungültig.
    - `FAILED_TO_RETRIEVE`: Konnte die angeforderten Informationen nicht abrufen (`query_system_info`).
    - `LLM_ERROR`: Fehler bei der Kommunikation mit dem LLM oder bei der Verarbeitung durch das LLM.
    - `BACKEND_ERROR`: Interner Fehler im C++ Backend.
    - `UNKNOWN_ERROR`: Ein nicht klassifizierter Fehler ist aufgetreten.
- **Fehlermeldungen (`message`):** Sollten präzise genug für Entwickler-Debugging sein (z. B. die exakte Fehlermeldung von `stderr` eines `QProcess`), aber nicht unbedingt für die direkte Anzeige an den Benutzer gedacht. Das LLM kann beauftragt werden, diese technischen Meldungen in eine benutzerfreundlichere Form zu übersetzen.

## VI. LLM-Integrationsplan

Die Integration des Large Language Models (LLM) ist der Schlüssel zur Übersetzung natürlicher Sprache in MCP-Befehle und zur Interpretation der Ergebnisse. Die Strategie muss die Kernanforderung berücksichtigen, dass das LLM das MCP allein durch die Spezifikation in diesem Bericht verstehen muss.

### A. LLM-Auswahlkriterien

Die Wahl des geeigneten LLM hängt von mehreren Faktoren ab:

- **Function Calling / Strukturierte Ausgabe:** Dies ist das wichtigste Kriterium. Das LLM muss zuverlässig strukturierte Ausgaben, idealerweise im JSON-Format, generieren können, die exakt der MCP-Spezifikation entsprechen. Modelle mit expliziter "Function Calling" oder "Tool Calling" Fähigkeit sind zu bevorzugen.45 Benchmarks wie BFCL (Berkeley Function-Calling Leaderboard) 49 und APIBank 50 können bei der Bewertung helfen. Aktuelle Kandidaten sind Cloud-Modelle wie GPT-4o, Claude 3.5 Sonnet, Gemini 1.5 Flash 46 oder potenziell leistungsfähige lokale Modelle (z. B. Llama 3, Mistral, Qwen), die entweder speziell für Tool Use feinabgestimmt wurden oder deren Ausgabe durch Techniken wie Constrained Generation 53 auf das korrekte JSON-Format gezwungen wird. Die Pythonic-Ansätze 57 sind hier weniger relevant, da MCP auf JSON basiert.
- **Lokal vs. API:**
    - _API-basiert (z. B. OpenAI, Anthropic):_ Bietet oft höhere Genauigkeit und einfachere initiale Einrichtung der Function Calling-Fähigkeit.45 Nachteile sind die Abhängigkeit von einer Internetverbindung, laufende Kosten und potenzielle Datenschutzbedenken, da Benutzeranfragen an einen externen Dienst gesendet werden.
    - _Lokal (z. B. Ollama + Llama 3, llama.cpp + Mistral):_ Bietet maximale Privatsphäre, Offline-Fähigkeit und keine direkten API-Kosten.52 Erfordert jedoch signifikante lokale Hardware-Ressourcen (CPU, RAM, VRAM) und die Implementierung robuster Mechanismen zur Erzeugung strukturierter Ausgaben (Constrained Generation), da die Genauigkeit bei der reinen Befolgung von Formatierungsanweisungen im Prompt geringer sein kann als bei spezialisierten APIs.56
- **Leistung (Latenz/Genauigkeit):** Die Antwortzeit des LLM (Latenz) und die Genauigkeit bei der Generierung korrekter MCP-Befehle müssen gegeneinander abgewogen werden.58 Zu hohe Latenz beeinträchtigt die Benutzererfahrung.
- **Kosten:** API-Nutzungsgebühren oder die Anschaffungs- und Betriebskosten für die Hardware zum lokalen Betrieb müssen berücksichtigt werden.

### B. Integrationsstrategie

Die Integration erfolgt im LLM Integration Module des C++ Backends.

- **Prompt Engineering:** Ein sorgfältig gestalteter System-Prompt ist unerlässlich. Er muss dem LLM seine Rolle als Manjaro-Assistent erklären, die verfügbaren "Werkzeuge" (implizit durch die MCP-Befehle in Abschnitt V definiert) beschreiben und das exakte JSON-Format für Anfragen (MCP Request) vorgeben. Der Prompt muss klarstellen, dass die Ausgabe _nur_ im spezifizierten JSON-Format erfolgen darf.
- **Function Calling Mechanismus:**
    - _Bei Nutzung einer API mit nativer Unterstützung (z. B. OpenAI Tools API 48, Anthropic Tools):_ Die MCP-Befehle aus Abschnitt V werden in das spezifische Format der API für Funktions-/Werkzeugdefinitionen übersetzt (Name, Beschreibung, Parameter-Schema). Das LLM wird dann direkt von der API aufgefordert, das passende Werkzeug (MCP-Befehl) und die Argumente zu nennen. Das LLM Integration Module parst die API-Antwort und extrahiert den MCP-Befehl und die Parameter zur Weiterleitung an den MCP Interface Handler.
    - _Bei Nutzung lokaler Modelle oder APIs ohne native Unterstützung:_ Hier ist Prompt Engineering entscheidend. Der Prompt muss das LLM anweisen, direkt das vollständige MCP Request JSON zu generieren. Zusätzlich _muss_ im LLM Interface Layer eine Technik zur **Constrained Generation** implementiert werden. Dies stellt sicher, dass die Ausgabe des LLM syntaktisch korrektes JSON ist und dem in Abschnitt V definierten Schema entspricht. Bibliotheken und Frameworks wie `instructor` (Python) 53, `outlines` (Python) 56, JSON Schema in Ollama 53 oder die Grammatik-Funktion (GBNF) von `llama.cpp` 55 bieten solche Möglichkeiten. Diese Technik filtert oder steuert die Token-Generierung des LLM, sodass nur gültige Ausgaben gemäß dem Schema erzeugt werden.54 Ohne Constrained Generation ist die Wahrscheinlichkeit hoch, dass lokale Modelle vom geforderten Format abweichen.56
- **Antwortbehandlung:** Das LLM Integration Module empfängt die MCP Response JSON vom Backend. Abhängig von der gewünschten Benutzererfahrung kann diese JSON-Antwort entweder direkt (nach einfacher Formatierung) an das Frontend weitergegeben werden, oder sie wird erneut an das LLM gesendet mit der Aufforderung, eine natürlichsprachliche Zusammenfassung oder Erklärung für den Benutzer zu generieren (z. B. "Ich habe Firefox gestartet" statt nur `{"status": "success",...}`).

### C. Anforderung an das MCP-Verständnis

- **Strikte Vorgabe:** Es muss sichergestellt werden, dass die gesamte Logik der LLM-Integration davon ausgeht, dass das LLM _kein_ Vorwissen über MCP hat und _ausschließlich_ auf die Informationen in Abschnitt V dieses Berichts zugreift.
- **Ableitung aus Spezifikation:** Alle Prompts, Funktions-/Werkzeugdefinitionen oder Grammatiken, die dem LLM zur Verfügung gestellt werden, müssen direkt und nachvollziehbar aus der MCP-Spezifikation in Abschnitt V abgeleitet sein.
- **Verifizierung:** Eine kritische Testphase muss überprüfen, ob das ausgewählte LLM, wenn ihm die MCP-Spezifikation als Kontext gegeben wird (z. B. als Teil eines langen System-Prompts oder über die Werkzeugbeschreibung), in der Lage ist, korrekte MCP-JSON-Anfragen für diverse natürlichsprachliche Eingaben zu generieren, ohne auf externes Wissen zurückzugreifen.

Die Notwendigkeit, dass das LLM MCP allein aus diesem Bericht lernt, unterstreicht die Bedeutung einer exzellenten "Function Calling" bzw. "Structured Output"-Fähigkeit.45 Da Standard-Trainingsdaten MCP nicht enthalten, muss die Definition zur Laufzeit bereitgestellt werden. Das LLM muss dann zuverlässig die Abbildung von natürlicher Sprache auf den korrekten MCP-Befehl und dessen JSON-Struktur durchführen. Dies macht Modelle mit starker Instruktionsbefolgung und Format-Treue unerlässlich. Für lokale Modelle wird Constrained Generation 53 quasi zur Pflicht, um die strikte Einhaltung des MCP-Formats zu garantieren, was die Integration im Vergleich zu APIs mit eingebauter, zuverlässiger Funktion aufwändiger macht.

## VII. C++ Backend Implementierungsdetails

Das C++ Backend bildet die Brücke zwischen der QML-Oberfläche, dem LLM und dem Manjaro-System. Die Implementierung muss robust, sicher und asynchron sein.

### A. Verarbeitung von MCP-Nachrichten

Der MCP Interface Handler ist für die Entgegennahme, Validierung und Weiterleitung von MCP-Befehlen sowie die Erzeugung von MCP-Antworten zuständig.

- **Empfang:** Eine Funktion oder ein Slot (verbunden mit dem LLM Integration Module) empfängt den MCP-Befehl als JSON-String.
- **Validierung:**
    1. **JSON-Parsing:** Verwendung von `QJsonDocument::fromJson()`, um den String in ein JSON-Objekt zu parsen. Bei Parsing-Fehlern wird sofort eine `INVALID_PARAMETER` (oder spezifischer `JSON_PARSE_ERROR`) MCP-Antwort generiert.
    2. **Strukturprüfung:** Überprüfung auf das Vorhandensein und die korrekten Basistypen (string, object) der Top-Level-Felder: `mcp_version`, `request_id`, `command`, `parameters`.
    3. **Versionsprüfung:** Abgleich der `mcp_version` mit der vom Backend unterstützten Version.
    4. **Befehlsprüfung:** Überprüfung, ob der Wert von `command` einem der in Abschnitt V.C definierten Befehle entspricht. Bei unbekanntem Befehl: `INVALID_COMMAND` Fehler.
    5. **Parameterprüfung:** Detaillierte Validierung des `parameters`-Objekts basierend auf der Definition für den spezifischen `command` aus Abschnitt V.C: Sind alle erforderlichen Parameter vorhanden? Haben alle Parameter den korrekten Datentyp (string, integer, boolean, array[string])? Sind Enum-Werte gültig? Bei Fehlern: `INVALID_PARAMETER` Fehler mit spezifischer Meldung.
- **Dispatching:** Nach erfolgreicher Validierung wird die entsprechende Methode in der System Interaction Layer aufgerufen. Die validierten und typisierten Parameter werden dabei übergeben.
- **Antwortgenerierung:** Die Methode empfängt das Ergebnis (als Datenstruktur oder Objekt) oder einen Fehler (als Fehlercode und Nachricht) von der System Interaction Layer. Sie konstruiert das MCP Response JSON unter Verwendung von `QJsonObject`, `QJsonArray` etc. und `QJsonDocument::toJson()`. Die `request_id` aus der Anfrage wird übernommen, `status` wird auf `success` oder `error` gesetzt, und entsprechend wird das `data`- oder `error`-Objekt befüllt.

### B. Implementierung der System Interaction Layer

Diese Schicht kapselt die tatsächliche Interaktion mit dem Manjaro-System.

- **Verwendung von `QProcess`:**
    - **Anwendungsfälle:** Ausführung von Kommandozeilenbefehlen für MCP-Kommandos wie `manage_packages_pamac`, `modify_setting_dconf`, `set_brightness`, `clipboard_copy`, `clipboard_paste`.
    - **Methoden:** `QProcess::start()` wird für asynchrone Ausführung verwendet. Die Signale `finished(int exitCode, QProcess::ExitStatus exitStatus)` und `errorOccurred(QProcess::ProcessError error)` müssen verbunden werden, um das Ergebnis oder Fehler zu behandeln.34 `QProcess::execute()` ist eine statische, blockierende Methode; sie sollte nur mit Vorsicht und idealerweise in einem separaten Worker-Thread verwendet werden, um die Haupt-Event-Loop nicht zu blockieren.34 `QProcess::startDetached()` ist ungeeignet, da keine Rückmeldung über Erfolg/Misserfolg oder Ausgabe benötigt wird.64 Der `QProcess`-Instanz muss eine ausreichende Lebensdauer gegeben werden (z.B. als Member-Variable oder Heap-Allokation mit Parent), da der Prozess sonst terminiert wird, wenn das `QProcess`-Objekt zerstört wird.64
    - **Argumentübergabe:** Kommandozeilenargumente müssen _immer_ als `QStringList` an `start()` übergeben werden.34 Dies verhindert Shell-Injection-Angriffe, da Qt die Argumente korrekt escaped und direkt an den auszuführenden Prozess übergibt, ohne eine Shell dazwischenzuschalten.37 Niemals Befehle durch String-Konkatenation mit Benutzereingaben zusammenbauen.
    - **Ausgabe lesen:** `stdout` und `stderr` werden über die Signale `readyReadStandardOutput()` und `readyReadStandardError()` oder nach Beendigung des Prozesses mit `readAllStandardOutput()` und `readAllStandardError()` gelesen.34 Die Ausgabe muss ggf. geparst werden (z. B. JSON-Ausgabe von Pamac, Textausgabe von `gsettings get`).
    - **Fehlerbehandlung:** Fehler wie "Programm nicht gefunden" (`QProcess::FailedToStart`), Absturz des Prozesses oder ein Exit-Code ungleich Null müssen abgefangen und in entsprechende MCP-Fehlercodes übersetzt werden.34
- **Verwendung von `QDBus`:**
    - **Anwendungsfälle:** Interaktion mit Diensten, die DBus-Schnittstellen anbieten (z. B. Lautstärkeregelung, Benachrichtigungen, Energieverwaltung).
    - **Identifikation:** Dienste, Objektpfade, Interfaces und Methoden/Signale müssen identifiziert werden (z. B. mit `qdbusviewer` oder durch Dokumentation der Desktop-Umgebung).39
    - **Implementierung:** Verwendung von `QDBusInterface` zum Aufrufen von Methoden oder `QDBusConnection::connect()` zum Verbinden mit Signalen.40 Asynchrone Aufrufe (`QDBusPendingCallWatcher`) sind zu bevorzugen. DBus-Fehler (`QDBusError`) müssen behandelt werden.
- **Interaktion mit `gsettings`/`dconf`:**
    - **Bevorzugter Ansatz:** Verwendung des `gsettings`-Kommandozeilenwerkzeugs via `QProcess`, da dies Schema-Validierung durchführt und als stabiler gilt als die direkte Interaktion mit der dconf-API.4
    - **Befehle:** Konstruktion von Befehlen wie `gsettings get <schema> <key>` oder `gsettings set <schema> <key> <value>`. Werte müssen korrekt für die Kommandozeile escaped/quotiert werden. Der Datentyp des Wertes muss dem Schema entsprechen.
    - **Ergebnis:** Bei `get`-Befehlen wird die `stdout`-Ausgabe geparst. Bei `set`-Befehlen wird der Exit-Code überprüft (0 für Erfolg). Fehler werden als `GSETTINGS_ERROR` gemeldet.
- **Allgemeine Fehlerbehandlung:** Jede Interaktionsmethode muss robust Fehler behandeln (Kommando nicht gefunden, Berechtigungsfehler, ungültige Argumente, Zeitüberschreitungen, unerwartete Ausgabeformate) und diese in die definierten MCP-Fehlercodes und aussagekräftige Meldungen übersetzen.

### C. Sicherheitsaspekte bei der Implementierung

Sicherheit muss auf Implementierungsebene berücksichtigt werden:

- **Eingabevalidierung und -bereinigung:** Obwohl das LLM das MCP generiert, muss das Backend _jede_ eingehende MCP-Anfrage und _alle_ Parameter erneut rigoros validieren und bereinigen, bevor sie in Systemaufrufen verwendet werden. Dies gilt insbesondere für Dateipfade, Paketnamen, Shell-Befehle (falls Skripte ausgeführt werden) und Konfigurationswerte.
- **Sichere Befehlskonstruktion:** Wie oben erwähnt, niemals Shell-Befehle durch String-Konkatenation erstellen. Immer `QProcess` mit `QStringList` für Argumente verwenden, um Shell-Interpretation zu umgehen.34
- **Privilegientrennung:** Der Backend-Prozess muss mit den Rechten des angemeldeten Benutzers laufen, nicht mit Root-Rechten. Wenn Aktionen höhere Rechte erfordern (z. B. Paketinstallation), sollte dies über etablierte Mechanismen wie Polkit erfolgen, die eine feingranulare Rechteverwaltung ermöglichen. Die direkte Verwendung von `sudo` im Backend ist zu vermeiden. Die Komplexität und Angriffsfläche erhöhen sich jedoch durch Polkit-Integration.

## VIII. Sicherheitsanalyse und Mitigation

Die Möglichkeit, Systemaktionen über eine KI-Schnittstelle auszulösen, birgt inhärente Sicherheitsrisiken, die sorgfältig analysiert und mitigiert werden müssen.

### A. Bedrohungsmodell

- **Angreifer:**
    - Ein böswilliger Benutzer, der versucht, durch geschickte Eingaben (Prompt Injection) das LLM zur Generierung schädlicher MCP-Befehle zu verleiten.
    - Ein kompromittiertes LLM (insbesondere bei Nutzung externer APIs).
    - Malware, die bereits auf dem System des Benutzers aktiv ist und versucht, die Sidebar oder deren Backend-Prozess auszunutzen.
- **Schützenswerte Güter (Assets):**
    - Benutzerdaten (persönliche Dateien, Konfigurationen, potenziell Zugangsdaten).
    - Systemintegrität (stabile Funktion des Betriebssystems und installierter Software).
    - Benutzerprivilegien und -identität.
    - Systemressourcen (CPU, Speicher, Netzwerkbandbreite).
- **Angriffsvektoren:**
    - **Prompt Injection:** Manipulation der LLM-Eingabe, um unerwünschte MCP-Befehle zu generieren.
    - **Exploitation von Befehlsausführung:** Ausnutzung von Schwachstellen in der Art, wie `QProcess` externe Befehle startet und verarbeitet, oder in den aufgerufenen Tools selbst.
    - **Unsichere DBus-Interaktion:** Ausnutzung von Schwachstellen in DBus-Diensten oder unsichere Kommunikation.
    - **Missbrauch von Dateisystemzugriff:** Generierung von MCP-Befehlen (`list_files` oder potenziell zukünftige Schreibbefehle), die auf sensible Bereiche zugreifen oder diese verändern.
    - **Unsichere Handhabung sensibler Daten:** Falls die Sidebar jemals Passwörter oder API-Schlüssel verarbeiten sollte (was vermieden werden sollte).

### B. Risikoidentifikation

Basierend auf dem Bedrohungsmodell ergeben sich folgende Hauptrisiken:

- **R1: Ausführung beliebigen Codes/Befehle (Arbitrary Code/Command Execution):** Höchstes Risiko. Ein manipuliertes LLM könnte MCP-Befehle generieren, die schädliche Aktionen auslösen (z. B. `open_application` mit Shell-Metazeichen im Namen, `manage_packages_pamac` zur Installation von Malware, `list_files` kombiniert mit Shell-Pipes in unsicherer Ausführung).
- **R2: Privilegieneskalation:** Wenn das Backend mit erhöhten Rechten läuft oder unsicher mit privilegierten Prozessen (z. B. via Polkit oder `sudo`) interagiert, könnte ein Angreifer Root-Zugriff erlangen.
- **R3: Informationspreisgabe:** MCP-Befehle wie `query_system_info` oder `list_files` könnten, wenn sie auf sensible Pfade oder Informationen angewendet werden, Daten an das LLM oder den Angreifer leaken.
- **R4: Denial of Service (DoS):** Gezielte MCP-Befehle könnten Systemressourcen überlasten (z. B. `list_files /`, exzessive `pamac`-Aufrufe) oder das System instabil machen.
- **R5: Datenkorruption/-löschung:** Befehle, die Einstellungen (`modify_setting_dconf`) oder potenziell Dateien ändern, könnten bei unzureichender Parameter-Validierung zu Datenverlust führen.
- **R6: LLM-Schwachstellen:** Eine Kompromittierung des LLM selbst (insbesondere bei Cloud-Diensten) oder erfolgreiche Prompt-Injection-Angriffe könnten zur Generierung schädlicher MCP-Befehle führen.

### C. Mitigationsstrategien

Um die identifizierten Risiken zu minimieren, müssen mehrere Verteidigungslinien implementiert werden:

1. **Strikte MCP-Validierung:** Das Backend _muss_ jede eingehende MCP-Anfrage rigoros gegen die in Abschnitt V definierte Spezifikation validieren. Dies umfasst die Struktur, den Befehlsnamen, die Anwesenheit und Typen aller Parameter sowie gültige Enum-Werte. Jede Abweichung führt zur sofortigen Ablehnung der Anfrage mit einem Fehler. (Adressiert R1, R5, R6)
2. **Parameter-Sanitisierung/-Escaping:** Alle Parameter, die in Systemaufrufen verwendet werden, müssen sorgfältig bereinigt und/oder escaped werden. Für `QProcess` ist die Verwendung von `QStringList` zur Argumentübergabe essenziell, um Shell-Interpretation zu vermeiden.34 Dateipfade und andere Strings müssen auf gefährliche Zeichen oder Sequenzen geprüft werden. (Adressiert R1, R5)
3. **Prinzip der geringsten Rechte (Least Privilege):** Der Backend-Prozess muss mit den Standardrechten des angemeldeten Benutzers laufen. Root-Rechte oder `sudo` sind zu vermeiden. Falls einzelne Aktionen erhöhte Rechte benötigen (z. B. systemweite Paketinstallation), ist eine feingranulare Autorisierung über Polkit zu prüfen, wobei die zusätzliche Komplexität und Angriffsfläche bedacht werden muss. (Adressiert R2)
4. **Command Whitelisting/Allowlisting (Optional):** Wenn möglich, sollte der Satz der erlaubten Aktionen weiter eingeschränkt werden. Beispielsweise könnte `open_application` nur auf Anwendungen aus einem vordefinierten, sicheren Satz beschränkt werden, oder `modify_setting_dconf` nur auf bestimmte, ungefährliche Schemata/Schlüssel. Dies reduziert die Angriffsfläche, kann aber die Flexibilität einschränken. (Adressiert R1, R5)
5. **Sandboxing der `QProcess`-Ausführung:** Dies ist eine kritische Maßnahme zur Eindämmung von R1.
    - _Konzept:_ Externe Prozesse, die über `QProcess` gestartet werden (insbesondere `pamac`, `gsettings`, `wl-clipboard`, `brightnessctl`), sollten in einer isolierten Umgebung (Sandbox) ausgeführt werden, die ihre Zugriffsrechte auf das System stark einschränkt.69
    - _Werkzeuge:_ `firejail` 71 und `bubblewrap` 73 sind geeignete Werkzeuge unter Linux. `firejail` bietet oft vordefinierte Profile, verwendet aber standardmäßig ein SUID-Binary, was eigene Risiken birgt.71 `bubblewrap` ist die Basis für Flatpak-Sandboxing, erfordert oft mehr manuelle Konfiguration, kann aber potenziell ohne SUID (mit User Namespaces) genutzt werden, wenn die Kernel-Unterstützung gegeben ist.73
    - _Implementierung:_ Statt `process->start("pamac", args)` würde man `process->start("firejail", QStringList() << "--profile=custom_pamac_profile" << "pamac" << args)` oder einen äquivalenten `bwrap`-Aufruf verwenden.
    - _Vorteile:_ Begrenzt den Schaden, den ein kompromittierter oder fehlgeleiteter Befehl anrichten kann, erheblich, indem Dateisystemzugriff, Netzwerkzugriff und erlaubte Systemaufrufe (via Seccomp) eingeschränkt werden.71
    - _Herausforderungen:_ Erfordert die Erstellung und Pflege spezifischer Sandbox-Profile für jedes verwendete externe Werkzeug. Kann zu Kompatibilitätsproblemen führen, wenn das Werkzeug legitime Zugriffe benötigt, die vom Profil blockiert werden. Potenzieller Performance-Overhead.
    - _Abwägung:_ Angesichts des Risikos, dass ein LLM unvorhersehbare oder manipulierte Befehle generiert, bietet Sandboxing eine essenzielle zusätzliche Sicherheitsebene. Die Komplexität der Profilerstellung muss gegen den Sicherheitsgewinn abgewogen werden. Es ist eine stark empfohlene Maßnahme. (Adressiert R1, R3, R4, R5)
6. **Rate Limiting:** Implementierung einer Begrenzung der Häufigkeit, mit der MCP-Befehle (insbesondere ressourcenintensive wie `pamac`) ausgeführt werden können, um DoS-Angriffe zu erschweren. (Adressiert R4)
7. **Benutzerbestätigung (Optional):** Für potenziell destruktive oder sicherheitskritische Aktionen (z. B. `pamac remove`, `pamac install`, Ändern wichtiger Systemeinstellungen) könnte eine explizite Bestätigung durch den Benutzer über einen Dialog im Frontend erforderlich sein, selbst wenn der Befehl vom LLM generiert wurde. Dies erhöht die Sicherheit, verringert aber die Automatisierung. (Adressiert R1, R5)
8. **Sichere LLM-Interaktion:** Bei Nutzung einer externen API muss die Kommunikation über HTTPS erfolgen. API-Schlüssel müssen sicher gespeichert und übertragen werden. Es ist zu überlegen, welche Daten (Benutzereingaben) an externe Dienste gesendet werden (Datenschutz). (Adressiert R6)

### D. Sicherheitsfokussiertes Testen

Zusätzlich zu den funktionalen Tests sind spezifische Sicherheitstests erforderlich:

- Penetration Testing: Gezielte Versuche, die Sicherheitsmechanismen zu umgehen.
- Fuzzing: Testen des MCP-Parsers und der System Interaction Layer mit ungültigen oder unerwarteten Eingaben.
- Prompt Injection Testing: Versuche, das LLM durch speziell gestaltete Eingaben zur Generierung unerwünschter MCP-Befehle zu bringen.
- Sandbox-Effektivität: Überprüfung, ob die implementierten Sandboxes (falls verwendet) die erwarteten Einschränkungen durchsetzen.

### Tabelle: Risikobewertung und Mitigation

|   |   |   |   |   |   |
|---|---|---|---|---|---|
|**Risiko ID**|**Beschreibung**|**Wahrscheinlichkeit**|**Auswirkung**|**Mitigationsstrategie(n) (Ref. C.x)**|**Restrisiko**|
|R1|Ausführung beliebigen Codes/Befehle|Hoch (ohne Mitigation)|Kritisch|C.1, C.2, C.4, C.5, C.7|Mittel (mit C.5), Hoch (ohne C.5)|
|R2|Privilegieneskalation|Mittel|Kritisch|C.3|Niedrig|
|R3|Informationspreisgabe|Mittel|Hoch|C.1, C.2, C.5|Niedrig-Mittel|
|R4|Denial of Service (DoS)|Mittel|Mittel|C.5, C.6|Niedrig|
|R5|Datenkorruption/-löschung|Mittel|Hoch|C.1, C.2, C.5, C.7|Niedrig-Mittel|
|R6|LLM-Schwachstellen / Prompt Injection|Hoch (API), Mittel (Lokal)|Hoch|C.1, C.2, C.5, C.7, C.8|Mittel|

_Anmerkung zur Tabelle:_ Die Bewertungen (Wahrscheinlichkeit, Auswirkung, Restrisiko) sind qualitativ und dienen der Priorisierung. Die Effektivität der Mitigationen, insbesondere von C.5 (Sandboxing), beeinflusst das Restrisiko maßgeblich. Diese Tabelle erzwingt eine systematische Betrachtung der Risiken und stellt sicher, dass für jedes identifizierte Risiko eine geplante Gegenmaßnahme existiert.

## IX. Grober Entwicklungs- und Testplan

Dieser Plan skizziert die Hauptphasen der Entwicklung und die dazugehörigen Testaktivitäten.

### A. Entwicklungsphasen

1. **Phase 1: Kern-Backend & Basis-MCP (ca. 4-6 Wochen)**
    - Implementierung der grundlegenden C++ Backend-Struktur (Core Logic, leere Module für LLM, MCP, System Interaction).
    - Implementierung des MCP Interface Handlers für das Parsen und Validieren von JSON-Anfragen und das Generieren von Antworten.
    - Implementierung der System Interaction Layer für eine kleine Teilmenge von MCP-Befehlen (z. B. `query_system_info`, `open_application`) unter Verwendung von `QProcess` und ggf. `QDBus` für einfache Tests.
    - Fokus: Robuste MCP-Verarbeitung und grundlegende Systeminteraktion.
2. **Phase 2: Sidebar UI & Wayland-Integration (ca. 3-4 Wochen)**
    - Entwicklung der initialen QML-Benutzeroberfläche für die Sidebar (Eingabefeld, Ausgabebereich).
    - Integration des QML-Frontends mit dem C++ Backend für einen einfachen Request/Response-Fluss (initial mit fest kodierten oder simulierten MCP-Nachrichten).
    - Implementierung der persistenten Sidebar-Funktionalität unter Wayland mithilfe von `layer-shell-qt`.22 Initialer Fokus auf KDE Plasma.
    - Fokus: Funktionierende UI und korrekte Darstellung/Positionierung unter Wayland (Plasma).
3. **Phase 3: LLM-Integration & MCP-Generierung (ca. 5-7 Wochen)**
    - Auswahl des initialen LLM (API-basiert für schnellere Iteration empfohlen, oder lokal mit Fokus auf Constrained Generation).
    - Implementierung des LLM Integration Module zur Kommunikation mit dem LLM.
    - Entwicklung des Prompt Engineerings bzw. der Function/Tool-Definitionen, um das LLM zur Generierung von MCP-Befehlen basierend auf natürlicher Sprache zu bewegen.
    - **Kritischer Test:** Überprüfung, ob das LLM valide MCP-Befehle _ausschließlich_ basierend auf der Spezifikation aus Abschnitt V generieren kann.45
    - Fokus: Übersetzung von natürlicher Sprache in korrekte MCP-JSON-Anfragen.
4. **Phase 4: Erweiterung des MCP-Befehlssatzes (ca. 6-8 Wochen)**
    - Implementierung der verbleibenden MCP-Befehle aus Abschnitt V.C.
    - Implementierung der entsprechenden Logik in der System Interaction Layer (Interaktion mit `pamac` 9, `gsettings` 4, `brightnessctl` 1, `wl-clipboard` 12 etc.).
    - Umfassende Tests der einzelnen Systeminteraktionen.
    - Fokus: Abdeckung der definierten Systemfunktionalität.
5. **Phase 5: Sicherheits-Hardening & Sandboxing (ca. 4-5 Wochen)**
    - Implementierung der definierten Sicherheitsmitigationen (strikte Validierung, Parameter-Sanitisierung).
    - Falls entschieden: Implementierung des Sandboxings für `QProcess`-Aufrufe mittels `firejail` oder `bubblewrap`, inklusive Erstellung der notwendigen Profile.68
    - Durchführung initialer Sicherheitstests.
    - Fokus: Absicherung der Anwendung gegen die identifizierten Risiken.
6. **Phase 6: Cross-DE Testing & Verfeinerung (ca. 3-4 Wochen)**
    - Testen der Anwendung unter verschiedenen Manjaro Desktop-Umgebungen (insbesondere GNOME und ggf. XFCE/Wayland).
    - Identifikation von Kompatibilitätsproblemen (speziell bei GNOME bzgl. `wlr-layer-shell` 21) und Entwicklung von Anpassungen oder Dokumentation von Einschränkungen.
    - Verfeinerung der UI/UX basierend auf Testergebnissen.
    - Fokus: Sicherstellung der bestmöglichen Funktion und Integration über verschiedene Umgebungen hinweg.
7. **Phase 7: Beta-Testing & Release (kontinuierlich)**
    - Durchführung von Beta-Tests mit einer breiteren Benutzergruppe.
    - Sammeln von Feedback, Behebung von Fehlern.
    - Erstellung von Benutzer- und Entwicklerdokumentation.
    - Vorbereitung des Releases.

### B. Teststrategie

Eine mehrschichtige Teststrategie ist erforderlich:

- **Unit-Tests:** Testen einzelner C++-Klassen und Funktionen im Backend (MCP-Parser, Validierer, einzelne Module der System Interaction Layer) isoliert voneinander unter Verwendung eines Test-Frameworks (z. B. Qt Test).
- **Integrationstests:** Testen des Zusammenspiels der Komponenten: QML-Frontend -> Core Logic -> LLM Module -> MCP Handler -> System Interaction Layer -> System -> Response -> Frontend. Simulation von LLM-Antworten und Systemverhalten.
- **MCP-Konformitätstests:**
    - _LLM-Generierung:_ Systematisches Testen, ob das LLM für eine breite Palette von natürlichsprachlichen Anfragen die korrekten MCP-JSON-Anfragen gemäß Spezifikation V generiert (Genauigkeit, Format, Parameter). Dies muss _ohne_ externes Wissen erfolgen.
    - _Backend-Verarbeitung:_ Testen, ob das Backend alle in V.C definierten Befehle korrekt validiert, verarbeitet und die erwarteten `data`- oder `error`-Strukturen in der MCP-Antwort zurückgibt. Testen aller definierten Fehlerfälle.
- **Systeminteraktionstests:** Verifizierung, dass jede Systemaktion (Pamac, gsettings, Helligkeit, Zwischenablage etc.) auf einem realen Manjaro-System korrekt ausgeführt wird. Testen von Grenzfällen (z. B. Paket nicht gefunden, Berechtigung verweigert, ungültige Eingaben). Tests sollten idealerweise auf den Ziel-Desktop-Umgebungen (Plasma, GNOME) durchgeführt werden.
- **Sicherheitstests:** Gezielte Tests zur Überprüfung der Sicherheitsmitigationen: Penetration Testing, Versuche von Prompt Injection, Überprüfung der Effektivität der Sandboxing-Maßnahmen (falls implementiert).
- **UI/UX-Tests:** Überprüfung der Benutzerfreundlichkeit, Responsivität und visuellen Integration der Sidebar auf den Ziel-Desktop-Umgebungen (Plasma, GNOME, XFCE).
- **Performancetests:** Messung der Ende-zu-Ende-Latenz von Benutzeranfrage bis zur Antwort, insbesondere der Latenz des LLM und der Systembefehlsausführung. Identifikation von Flaschenhälsen.

## X. Schlussfolgerung

### Zusammenfassung

Dieser Bericht hat einen detaillierten Plan und eine technische Spezifikation für die Entwicklung einer KI-gestützten Desktop-Sidebar für Manjaro Linux unter Verwendung von C++, Qt, QML und Qt-Wayland vorgestellt. Die vorgeschlagene Architektur trennt klar zwischen Frontend, Backend-Logik, LLM-Interaktion und Systemzugriff. Das Kernstück bildet das Manjaro Control Protocol (MCP), eine JSON-basierte Schnittstelle, die speziell darauf ausgelegt ist, von einem LLM allein anhand dieser Spezifikation verstanden und genutzt zu werden. Die Integration in Wayland-Umgebungen, insbesondere die Nutzung des `wlr-layer-shell`-Protokolls mittels `layer-shell-qt`, wurde ebenso detailliert wie die notwendigen Mechanismen zur Systeminteraktion (`QProcess`, `QDBus`, `gsettings`) und die Strategien zur LLM-Integration (lokal vs. API, strukturierte Ausgabe). Ein besonderer Fokus lag auf der Analyse von Sicherheitsrisiken und der Definition von Mitigationsstrategien, einschließlich der Möglichkeit des Sandboxing für externe Prozessaufrufe.

### Potenzial

Die Realisierung dieses Projekts bietet erhebliches Potenzial. Eine nahtlos integrierte, sprachgesteuerte KI-Assistenz kann die Interaktion mit dem Manjaro-System erheblich vereinfachen und beschleunigen. Aufgaben wie das Starten von Anwendungen, das Verwalten von Paketen oder das Anpassen von Einstellungen werden intuitiver. Dies stellt eine moderne und leistungsfähige Erweiterung der Desktop-Erfahrung dar und positioniert Manjaro als innovative Plattform.

### Herausforderungen

Die Umsetzung birgt auch Herausforderungen. Die Gewährleistung einer konsistenten Funktionalität und visuellen Integration über verschiedene Wayland-basierte Desktop-Umgebungen hinweg, insbesondere die Kompatibilität mit GNOME/Mutter aufgrund der fehlenden `wlr-layer-shell`-Unterstützung 21, erfordert sorgfältige Planung und möglicherweise umgebungsspezifische Anpassungen. Die Absicherung des Systems gegen Missbrauch durch die KI-Schnittstelle, insbesondere die Risiken der Befehlsausführung (R1) und der LLM-Manipulation (R6), bedarf rigoroser Implementierung der Sicherheitsmaßnahmen, wobei Sandboxing 70 eine wichtige, aber komplexe Komponente darstellt. Die Sicherstellung, dass das LLM das MCP korrekt und zuverlässig _allein_ aus der Spezifikation anwendet, ist eine zentrale Anforderung, die sorgfältiges Prompt Engineering und möglicherweise den Einsatz von Constrained Generation Techniken erfordert.

### Nächste Schritte

Basierend auf dieser detaillierten Speifikation wird empfohlen, mit der Entwicklung gemäß Phase 1 des vorgeschlagenen Plans zu beginnen. Dies umfasst die Implementierung des Kern-Backends und der Basis-MCP-Verarbeitung, um eine solide Grundlage für die weiteren Schritte zu schaffen. Parallel dazu sollte die Auswahl des LLM und die Verfeinerung der Integrationsstrategie unter Berücksichtigung der strukturierten Ausgabeanforderungen erfolgen.
