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