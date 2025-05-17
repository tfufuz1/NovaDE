# **Implementierungsleitfaden Systemschicht (Teil 3/4)**

## **I. Einleitung zu den Spezifikationen der Systemschicht (Teil 3/4)**

### **Überblick**

Die Systemschicht, wie in der technischen Gesamtspezifikation dargelegt, bildet das kritische Bindeglied zwischen der abstrakten Logik der Domänenschicht, der Präsentationslogik der Benutzeroberflächenschicht und den konkreten Funktionalitäten des zugrundeliegenden Linux-Betriebssystems sowie der Hardware. Ihre Hauptaufgabe besteht darin, die "Mechanik" der Desktop-Umgebung zu implementieren, indem sie übergeordnete Richtlinien und Benutzerinteraktionen in handfeste Systemaktionen übersetzt. Dieser Prozess erfordert eine präzise und robuste Interaktion mit einer Vielzahl externer Komponenten, darunter Wayland-Protokolle, die über Bibliotheken wie Smithay gehandhabt werden, D-Bus-Systemdienste wie UPower und Logind sowie potenziell direkte Hardware-Interaktionen, beispielsweise über das Direct Rendering Manager (DRM)-Subsystem.  
Die Stabilität und Reaktionsfähigkeit der gesamten Desktop-Umgebung hängt maßgeblich von der Zuverlässigkeit der Systemschicht ab. Da diese Schicht intensiv mit externen, oft asynchronen Systemen kommuniziert, können Unvorhersehbarkeiten wie Latenzen, Fehler oder unerwartete Zustandsänderungen auftreten. Eine unzureichend robuste Systemschicht, die beispielsweise bei einem langsamen D-Bus-Aufruf blockiert, bei einem unerwarteten Wayland-Ereignis in Panik gerät oder den Ausfall eines Dienstes nicht korrekt behandelt, würde die Stabilität der gesamten Desktop-Umgebung direkt gefährden. Daher muss das Design jedes Moduls der Systemschicht Resilienz als oberste Priorität behandeln. Dies bedeutet konkret den Einsatz asynchroner Operationen für alle potenziell blockierenden E/A-Vorgänge, insbesondere bei D-Bus-Aufrufen (unterstützt durch zbus) und der Wayland-Ereignisverarbeitung. Ein umfassendes, typisiertes Fehlermanagement pro Modul (mittels thiserror) ist unerlässlich, um höheren Schichten eine angemessene Reaktion auf Fehlerzustände zu ermöglichen. Dies schließt die Behandlung von D-Bus-Fehlern, Wayland-Protokollfehlern und internen Logikfehlern ein. Wo immer möglich, sollten Interaktionen mit externen Diensten Timeouts beinhalten, und Fallback-Mechanismen oder eine graceful degradation der Funktionalität müssen in Betracht gezogen werden, falls ein Dienst nicht verfügbar oder nicht reaktionsfähig ist. Eine sorgfältige Zustandssynchronisation ist ebenfalls von entscheidender Bedeutung, insbesondere wenn der Zustand von externen Komponenten abgeleitet wird oder diese beeinflusst. Mechanismen zur Erkennung und Behebung von Zustandsdiskrepanzen, wie z.B. die Verwendung von Serialnummern in Wayland-Protokollen, müssen akribisch implementiert werden.

### **Zweck dieses Dokuments**

Dieses Dokument, "Teil 3/4" der Spezifikationen für die Systemschicht, legt vier detaillierte, ultrafeingranulare Implementierungspläne für Schlüsselmodule dieser Schicht vor. Ziel ist es, den Entwicklern so präzise Vorgaben an die Hand zu geben, dass eine direkte Implementierung ohne weitere architektonische oder tiefgreifende Designentscheidungen möglich wird.

### **Beziehung zur Gesamtarchitektur**

Die hier spezifizierten Module – system::outputs::output\_manager, system::outputs::power\_manager, system::dbus::upower\_interface und system::dbus::logind\_interface – sind fundamental für die Verwaltung der Display-Hardware und die Integration mit essenziellen Systemdiensten. Sie bauen auf den in der Kernschicht definierten grundlegenden Datentypen und Dienstprogrammen auf und stellen notwendige Funktionalitäten und Ereignisse für die Domänen- und Benutzeroberflächenschicht bereit.

## **II. Ultra-Feinspezifikation: system::outputs::output\_manager (Wayland Output Konfiguration)**

### **A. Modulübersicht und Zweck**

* **Verantwortlichkeit:** Dieses Modul implementiert die serverseitige Logik für das Wayland-Protokoll wlr-output-management-unstable-v1. Seine primäre Funktion besteht darin, Wayland-Clients – typischerweise Display-Konfigurationswerkzeuge – zu ermöglichen, verfügbare Display-Ausgänge zu erkennen, deren Fähigkeiten abzufragen (Modi, unterstützte Auflösungen, Bildwiederholraten, physische Dimensionen, Skalierung, Transformation) und atomare Änderungen an ihrer Konfiguration anzufordern (z.B. Setzen eines neuen Modus, Positionierung, Aktivieren/Deaktivieren eines Ausgangs).  
* **Interaktion:** Es interagiert mit der internen Repräsentation von Display-Ausgängen des Compositors, die wahrscheinlich durch Smithays Output- und OutputManagerState-Strukturen verwaltet werden.1 Über dieses Protokoll angeforderte Änderungen werden in Operationen auf diesen internen Smithay-Objekten übersetzt, die wiederum mit dem DRM-Backend (Direct Rendering Manager) interagieren können, um Hardware-Änderungen zu bewirken.  
* **Schlüsselprotokollelemente:** zwlr\_output\_manager\_v1, zwlr\_output\_head\_v1, zwlr\_output\_mode\_v1, zwlr\_output\_configuration\_v1.  
* **Relevante Referenzmaterialien & Analyse:**  
  * 2 (Protokollübersicht): Liefert die XML-Definition und detailliert Anfragen wie create\_configuration, apply, test sowie Ereignisse wie head, done, succeeded, failed. Dies ist die primäre Quelle für die Struktur der Protokollnachrichten.  
  * 1 (Smithay Output, OutputManagerState, OutputHandler): Diese Smithay-Komponenten sind fundamental. Output repräsentiert ein physisches Display im Compositor. OutputManagerState hilft bei der Verwaltung von wl\_output-Globalen. Der OutputHandler (oder ein spezifischerer Handler für dieses Protokoll) wird implementiert, um Client-Anfragen zu verarbeiten. Dieses Modul wird im Wesentlichen eine Brücke zwischen dem wlr-output-management-Protokoll und diesen Smithay-Abstraktionen schlagen.  
  * 26 (Anvil DRM Output Management): Zeigt ein praktisches Beispiel, wie Smithays Output basierend auf DRM-Geräteinformationen erstellt und konfiguriert wird. Während dieses Modul die Wayland-Protokollseite behandelt, werden die zugrundeliegenden Mechanismen zur Anwendung von Änderungen denen im DRM-Backend von Anvil ähneln.  
  * 1 (Smithay OutputHandler und wlr-output-management): Bestärken die Verbindung zwischen Smithays Output-Handling und dem wlr-output-management-Protokoll.

### **B. Entwicklungs-Submodule & Dateien**

* **1\. system::outputs::output\_manager::manager\_global**  
  * Dateien: system/outputs/output\_manager/manager\_global.rs  
  * Verantwortlichkeiten: Verwaltet den Lebenszyklus des zwlr\_output\_manager\_v1-Globals. Behandelt Bindeanfragen von Clients für dieses Global. Leitet Client-Anfragen zur Erstellung neuer zwlr\_output\_configuration\_v1-Objekte weiter.  
* **2\. system::outputs::output\_manager::head\_handler**  
  * Dateien: system/outputs/output\_manager/head\_handler.rs  
  * Verantwortlichkeiten: Verwaltet zwlr\_output\_head\_v1-Objekte. Sendet name, description, physical\_size, mode, enabled, current\_mode, position, transform, scale, finished, make, model, serial\_number-Ereignisse an den Client, basierend auf dem Zustand des entsprechenden smithay::output::Output.  
* **3\. system::outputs::output\_manager::mode\_handler**  
  * Dateien: system/outputs/output\_manager/mode\_handler.rs  
  * Verantwortlichkeiten: Verwaltet zwlr\_output\_mode\_v1-Objekte. Sendet size, refresh, preferred, finished-Ereignisse basierend auf den für ein smithay::output::Output verfügbaren Modi.  
* **4\. system::outputs::output\_manager::configuration\_handler**  
  * Dateien: system/outputs/output\_manager/configuration\_handler.rs  
  * Verantwortlichkeiten: Verwaltet zwlr\_output\_configuration\_v1- und zwlr\_output\_configuration\_head\_v1-Objekte. Speichert vom Client angeforderte, ausstehende Änderungen. Implementiert die Logik für test- und apply-Anfragen, interagiert mit dem Kern-Output-Zustand des Compositors und potenziell dem DRM-Backend. Sendet succeeded-, failed- oder cancelled-Ereignisse.  
* **5\. system::outputs::output\_manager::types**  
  * Dateien: system/outputs/output\_manager/types.rs  
  * Verantwortlichkeiten: Definiert Rust-Strukturen und \-Enums, die Protokolltypen widerspiegeln oder internen Zustand für die Verwaltung von Konfigurationen repräsentieren (z.B. PendingHeadConfiguration, AppliedConfigurationAttempt).  
* **6\. system::outputs::output\_manager::errors**  
  * Dateien: system/outputs/output\_manager/errors.rs  
  * Verantwortlichkeiten: Definiert das OutputManagerError-Enum mittels thiserror für Fehler, die spezifisch für die Operationen dieses Moduls sind.

### **C. Schlüsseldatenstrukturen**

* OutputManagerModuleState:  
  * output\_manager\_global: Option\<GlobalId\> (Smithay-Global für zwlr\_output\_manager\_v1)  
  * active\_configurations: HashMap\<ObjectId, Arc\<Mutex\<PendingOutputConfiguration\>\>\> (Verfolgt aktive zwlr\_output\_configuration\_v1-Instanzen)  
  * compositor\_output\_serial: u32 (Wird inkrementiert, wenn sich das Output-Layout des Compositors ändert)  
* PendingOutputConfiguration: Repräsentiert eine vom Client angeforderte Konfiguration über zwlr\_output\_configuration\_v1.  
  * serial: u32 (Vom Client bei Erstellung bereitgestellte Serialnummer)  
  * head\_configs: HashMap\<WlOutput, HeadConfigChange\> (Mappt wl\_output auf gewünschte Änderungen)  
  * is\_applied\_or\_tested: bool  
* HeadConfigChange:  
  * target\_output\_name: String (Interner Name/ID des Output-Objekts des Compositors)  
  * enabled: Option\<bool\>  
  * mode: Option\<OutputModeRequest\> (Könnte spezifische Mode-ID oder benutzerdefinierte Modusparameter sein)  
  * position: Option\<Point\<i32, Logical\>\>  
  * transform: Option\<wl\_output::Transform\>  
  * scale: Option\<f64\>  
* OutputModeRequest: Enum für ExistingMode(ModeId) oder CustomMode { width: i32, height: i32, refresh: i32 }.

**Tabelle: OutputManager-Datenstrukturen**

| Struct/Enum Name | Felder (Name, Rust-Typ, nullable, Mutabilität) | Beschreibung | Korrespondierendes Wayland-Protokollelement/Konzept |
| :---- | :---- | :---- | :---- |
| OutputManagerModuleState | output\_manager\_global: Option\<GlobalId\> (intern, veränderlich) \<br\> active\_configurations: HashMap\<ObjectId, Arc\<Mutex\<PendingOutputConfiguration\>\>\> (intern, veränderlich) \<br\> compositor\_output\_serial: u32 (intern, veränderlich) | Hauptzustand des Moduls, verwaltet das Global und aktive Konfigurationen. | zwlr\_output\_manager\_v1 |
| PendingOutputConfiguration | serial: u32 (intern, unveränderlich nach Erstellung) \<br\> head\_configs: HashMap\<WlOutput, HeadConfigChange\> (intern, veränderlich durch Client-Requests) \<br\> is\_applied\_or\_tested: bool (intern, veränderlich) | Speichert eine vom Client initiierte, aber noch nicht angewendete oder getestete Konfiguration. | zwlr\_output\_configuration\_v1 |
| HeadConfigChange | target\_output\_name: String (intern) \<br\> enabled: Option\<bool\> (optional) \<br\> mode: Option\<OutputModeRequest\> (optional) \<br\> position: Option\<Point\<i32, Logical\>\> (optional) \<br\> transform: Option\<wl\_output::Transform\> (optional) \<br\> scale: Option\<f64\> (optional) | Repräsentiert die gewünschten Änderungen für einen einzelnen Output (head). | zwlr\_output\_configuration\_head\_v1-Anfragen |
| OutputModeRequest | ExistingMode(ModeId) \<br\> CustomMode { width: i32, height: i32, refresh: i32 } | Unterscheidet zwischen der Auswahl eines existierenden Modus oder der Definition eines benutzerdefinierten Modus. | zwlr\_output\_configuration\_head\_v1.set\_mode, zwlr\_output\_configuration\_head\_v1.set\_custom\_mode |

Diese Datenstrukturen sind fundamental, um den Zustand der von Clients initiierten Output-Konfigurationen zu verfolgen. Die OutputManagerModuleState dient als zentraler Punkt für die Verwaltung des globalen zwlr\_output\_manager\_v1 und der damit verbundenen Konfigurationsobjekte. Jede PendingOutputConfiguration kapselt die Gesamtheit der Änderungen, die ein Client für eine Gruppe von Outputs vornehmen möchte, bevor diese getestet oder angewendet werden. Die compositor\_output\_serial ist entscheidend für die Synchronisation des Client-Wissens mit dem tatsächlichen Zustand der Outputs im Compositor.

### **D. Protokollbehandlung: zwlr\_output\_manager\_v1 (Interface Version: 3 2)**

* **Smithay Handler:** Die Zustandsverwaltung und Anforderungsbehandlung für das zwlr\_output\_manager\_v1-Global wird durch Implementierung der Traits GlobalDispatch\<ZwlrOutputManagerV1, GlobalData, YourCompositorState\> und Dispatch\<ZwlrOutputManagerV1, UserData, YourCompositorState\> für die OutputManagerModuleState-Struktur realisiert. GlobalData könnte hier leer sein oder minimale globale Informationen enthalten, während UserData für gebundene Manager-Instanzen spezifisch sein kann, falls erforderlich (oftmals ist für Singleton-Manager-Globale keine komplexe UserData nötig).  
* **Globalerstellung:** Das zwlr\_output\_manager\_v1-Global wird einmalig beim Start des Compositors oder bei der Initialisierung dieses Moduls mittels DisplayHandle::create\_global erstellt und dem Wayland-Display hinzugefügt. Die zurückgegebene GlobalId wird in OutputManagerModuleState::output\_manager\_global gespeichert.  
* **Anfrage: create\_configuration(id: New\<ZwlrOutputConfigurationV1\>, serial: u32)**  
  * Rust Signatur:  
    Rust  
    fn create\_configuration(  
        \&mut self,  
        \_client: \&Client, // wayland\_server::Client  
        \_manager: \&ZwlrOutputManagerV1, // wayland\_protocols::wlr::output\_management::v1::server::zwlr\_output\_manager\_v1::ZwlrOutputManagerV1  
        new\_id: New\<ZwlrOutputConfigurationV1\>, // wayland\_server::New\<ZwlrOutputConfigurationV1\>  
        serial: u32,  
        data\_init: \&mut DataInit\<'\_, YourCompositorState\> // wayland\_server::DataInit  
    ) {... }  
    (Hinweis: Die genaue Signatur hängt von der Implementierung des Dispatch-Traits ab; Result\<(), BindError\> ist bei GlobalDispatch nicht direkt der Rückgabewert der bind-Methode, sondern die Initialisierung erfolgt innerhalb.)  
  * Implementierung:  
    1. Die vom Client bereitgestellte serial wird mit der aktuellen self.compositor\_output\_serial verglichen. Obwohl das Protokoll nicht explizit eine Ablehnung bei Serial-Mismatch hier vorschreibt, ist es ein Indikator dafür, dass der Client möglicherweise veraltete Informationen hat. Eine Warnung kann geloggt werden. Die eigentliche Konsequenz eines Serial-Mismatchs wird typischerweise beim apply oder test relevant, wo eine cancelled-Nachricht gesendet werden kann.2  
    2. Eine neue Instanz von PendingOutputConfiguration wird mit der clientseitigen serial erstellt.  
    3. Diese PendingOutputConfiguration wird in einem Arc\<Mutex\<...\>\> verpackt und in OutputManagerModuleState::active\_configurations gespeichert, wobei die ObjectId des neuen zwlr\_output\_configuration\_v1-Objekts als Schlüssel dient.  
    4. Die zwlr\_output\_configuration\_v1-Ressource wird für den Client initialisiert und mit dem Arc\<Mutex\<PendingOutputConfiguration\>\> als UserData versehen. data\_init.init(new\_id, user\_data\_arc\_clone);  
* **Anfrage: stop() (seit Version 3\)**  
  * Rust Signatur:  
    Rust  
    fn stop(  
        \&mut self,  
        \_client: \&Client,  
        \_manager: \&ZwlrOutputManagerV1  
    ) {... }

  * Implementierung:  
    1. Wenn der Client die entsprechende Berechtigung hat (üblicherweise jeder Client, der den Manager gebunden hat), wird das zwlr\_output\_manager\_v1-Global zerstört.  
    2. Dies bedeutet, dass self.output\_manager\_global.take().map(|id| display\_handle.remove\_global(id)); aufgerufen wird, sodass keine neuen Clients mehr binden können.  
    3. Bestehende zwlr\_output\_configuration\_v1-Objekte könnten gemäß Protokollspezifikation weiterhin gültig bleiben, bis sie explizit vom Client zerstört werden oder ihre Operationen mit succeeded, failed oder cancelled abschließen. Die finished-Nachricht auf dem Manager signalisiert Clients, dass der Manager nicht mehr verwendet werden kann.  
* **Vom Compositor gesendete Ereignisse (beim Binden oder bei Änderung des Output-Zustands):**  
  * head(output: WlOutput): Für jedes aktuell vom Compositor verwaltete smithay::output::Output. Das WlOutput-Objekt wird dem Client übergeben.  
  * done(serial: u32): Nach allen head-Ereignissen wird die aktuelle compositor\_output\_serial gesendet.  
  * finished(): Wenn das Manager-Global zerstört wird (z.B. durch stop() oder beim Herunterfahren des Compositors).

**Tabelle: zwlr\_output\_manager\_v1 Interface-Behandlung**

| Anfrage/Ereignis | Richtung | Smithay Handler Signatur (Beispiel) | Parameter (Name, Wayland-Typ, Rust-Typ) | Vorbedingungen | Nachbedingungen | Fehlerbedingungen | Beschreibung |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| create\_configuration | Client \-\> Server | fn create\_configuration(..., new\_id: New\<ZwlrOutputConfigurationV1\>, serial: u32,...) | id: new\_id (New\<ZwlrOutputConfigurationV1\>), serial: uint (u32) | Manager-Global existiert. | Neues ZwlrOutputConfigurationV1-Objekt erstellt und mit PendingOutputConfiguration assoziiert. | Protokollfehler bei ungültiger ID. | Erstellt ein neues Konfigurationsobjekt. |
| stop | Client \-\> Server | fn stop(...) | \- | Manager-Global existiert. | Manager-Global wird für neue Bindungen deaktiviert/zerstört. finished-Ereignis wird gesendet. | \- | Stoppt den Output-Manager. |
| head | Server \-\> Client | \- (Intern ausgelöst) | output: object (WlOutput) | Output existiert im Compositor. | Client erhält Referenz auf ein WlOutput-Objekt. | \- | Informiert Client über einen verfügbaren Output. |
| done | Server \-\> Client | \- (Intern ausgelöst) | serial: uint (u32) | Alle head-Ereignisse für aktuellen Zustand gesendet. | Client kennt aktuelle Output-Serialnummer des Compositors. | \- | Signalisiert Ende der Output-Auflistung. |
| finished | Server \-\> Client | \- (Intern ausgelöst) | \- | Manager-Global wird zerstört. | Client weiß, dass der Manager nicht mehr nutzbar ist. | \- | Manager wurde beendet. |

### **E. Protokollbehandlung: zwlr\_output\_configuration\_v1 (Interface Version: 3\)**

* **Smithay Handler:** impl Dispatch\<ZwlrOutputConfigurationV1, Arc\<Mutex\<PendingOutputConfiguration\>\>, YourCompositorState\> for OutputManagerModuleState. Die UserData für jede zwlr\_output\_configuration\_v1-Ressource ist ein Arc\<Mutex\<PendingOutputConfiguration\>\>, das den Zustand der vom Client angeforderten, aber noch nicht angewendeten Konfiguration enthält.  
* **Anfragen vom Client (modifizieren PendingOutputConfiguration):**  
  * destroy(): Entfernt die zugehörige PendingOutputConfiguration aus OutputManagerModuleState::active\_configurations. Die Ressource wird von Smithay automatisch bereinigt.  
  * enable\_head(head: \&WlOutput): Setzt enabled \= Some(true) in der HeadConfigChange für den gegebenen head in PendingOutputConfiguration.  
  * disable\_head(head: \&WlOutput): Setzt enabled \= Some(false).  
  * set\_mode(head: \&WlOutput, mode: \&ZwlrOutputModeV1): Aktualisiert mode \= Some(OutputModeRequest::ExistingMode(mode\_id)) in HeadConfigChange. Die mode\_id muss aus dem ZwlrOutputModeV1-Objekt extrahiert werden (z.B. über dessen UserData).  
  * set\_custom\_mode(head: \&WlOutput, width: i32, height: i32, refresh: i32): Aktualisiert mode \= Some(OutputModeRequest::CustomMode { width, height, refresh }).  
  * set\_position(head: \&WlOutput, x: i32, y: i32): Aktualisiert position \= Some(Point::from((x, y))).  
  * set\_transform(head: \&WlOutput, transform: wl\_output::Transform): Aktualisiert transform \= Some(transform).  
  * set\_scale(head: \&WlOutput, scale: u32): Aktualisiert scale \= Some(scale as f64 / 256.0). Die Skalierung wird als Festkommazahl (multipliziert mit 256\) über das Protokoll gesendet. Alle diese Anfragen dürfen nur aufgerufen werden, wenn die Konfiguration noch nicht mit test() oder apply() verarbeitet wurde (PendingOutputConfiguration::is\_applied\_or\_tested \== false). Andernfalls ist es ein Protokollfehler (already\_applied\_or\_tested).  
* **Anfrage: test()**  
  * Implementierung:  
    1. Sperre den Mutex der PendingOutputConfiguration.  
    2. Wenn is\_applied\_or\_tested \== true, sende Protokollfehler already\_applied\_or\_tested und gib zurück.  
    3. Iteriere über head\_configs. Für jede HeadConfigChange:  
       * Identifiziere das Ziel-smithay::output::Output-Objekt anhand von WlOutput (z.B. über dessen UserData, das den Namen/ID des Smithay-Outputs enthält).  
       * Validiere die angeforderte Konfiguration:  
         * Existiert der Output noch?  
         * Wenn enabled \== Some(true):  
           * Ist der angeforderte Modus (existierend oder benutzerdefiniert) vom Output unterstützt? (Prüfe gegen Output::modes()).  
           * Ist die Position im Rahmen der Compositor-Policy gültig (z.B. keine unmöglichen Überlappungen, falls der Compositor dies prüft)?  
           * Sind Skalierung und Transformation gültige Werte?  
    4. Wenn alle Prüfungen erfolgreich sind, sende das succeeded()-Ereignis auf dem zwlr\_output\_configuration\_v1-Objekt.  
    5. Andernfalls sende das failed()-Ereignis.  
    6. Setze is\_applied\_or\_tested \= true.  
* **Anfrage: apply()**  
  * Implementierung:  
    1. Sperre den Mutex der PendingOutputConfiguration.  
    2. Wenn is\_applied\_or\_tested \== true, sende Protokollfehler already\_applied\_or\_tested und gib zurück.  
    3. Vergleiche PendingOutputConfiguration::serial mit OutputManagerModuleState::compositor\_output\_serial. Wenn sie nicht übereinstimmen, bedeutet dies, dass sich der Output-Zustand des Compositors geändert hat, seit der Client diese Konfiguration erstellt hat. Sende das cancelled()-Ereignis und gib zurück.  
    4. Führe Validierungen ähnlich wie bei test() durch. Wenn ungültig, sende failed() und gib zurück.  
    5. Versuche, die Konfiguration auf die tatsächlichen smithay::output::Output-Objekte des Compositors anzuwenden. Dies kann das Batchen von Änderungen beinhalten, wenn das DRM-Backend atomares Modesetting unterstützt.  
       * Für jede HeadConfigChange im PendingOutputConfiguration:  
         * Rufe output.change\_current\_state(...) mit den neuen Eigenschaften auf. Diese Methode in smithay::output::Output ist dafür verantwortlich, die Änderungen an das Backend (z.B. DRM) weiterzuleiten.  
         * Sammle die Ergebnisse dieser Operationen.  
    6. Wenn alle Hardware-Änderungen erfolgreich waren (oder erfolgreich simuliert wurden, falls kein echtes Backend):  
       * Inkrementiere OutputManagerModuleState::compositor\_output\_serial.  
       * Sende das succeeded()-Ereignis auf dem zwlr\_output\_configuration\_v1-Objekt.  
       * Benachrichtige alle zwlr\_output\_manager\_v1-Clients über den neuen Zustand, indem neue head-Ereignisse und ein done-Ereignis mit der neuen compositor\_output\_serial gesendet werden. Dies stellt sicher, dass alle Clients über die erfolgreiche Konfigurationsänderung informiert werden.  
    7. Wenn eine Hardware-Änderung fehlschlägt:  
       * Versuche, alle bereits teilweise angewendeten Änderungen dieser Konfiguration zurückzusetzen (Best-Effort-Basis). Dies ist ein komplexer Teil und hängt stark von den Fähigkeiten des Backends ab.  
       * Sende das failed()-Ereignis.  
    8. Setze is\_applied\_or\_tested \= true.  
* **Ereignisse an den Client:** succeeded(), failed(), cancelled().

**Tabelle: zwlr\_output\_configuration\_v1 Interface-Behandlung**

| Anfrage/Ereignis | Richtung | Smithay Handler Signatur (Beispiel) | Parameter (Name, Wayland-Typ, Rust-Typ) | Vorbedingungen | Nachbedingungen | Fehlerbedingungen | Beschreibung |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| destroy | Client \-\> Server | fn destroyed(..., \_data: \&Arc\<Mutex\<PendingOutputConfiguration\>\>) | \- | Konfigurationsobjekt existiert. | Konfigurationsobjekt und zugehöriger Zustand werden bereinigt. | \- | Zerstört das Konfigurationsobjekt. |
| enable\_head | Client \-\> Server | fn request(..., request: zwlr\_output\_configuration\_v1::Request, data: \&Arc\<Mutex\<PendingOutputConfiguration\>\>...) | head: object (WlOutput) | is\_applied\_or\_tested \== false. head ist valides WlOutput. | PendingOutputConfiguration für head wird auf enabled \= Some(true) gesetzt. | already\_applied\_or\_tested. | Aktiviert einen Output in der pend. Konfiguration. |
| disable\_head | Client \-\> Server | (wie enable\_head) | head: object (WlOutput) | (wie enable\_head) | PendingOutputConfiguration für head wird auf enabled \= Some(false) gesetzt. | already\_applied\_or\_tested. | Deaktiviert einen Output in der pend. Konfiguration. |
| set\_mode | Client \-\> Server | (wie enable\_head) | head: object (WlOutput), mode: object (ZwlrOutputModeV1) | (wie enable\_head). mode ist valider Modus für head. | PendingOutputConfiguration für head wird auf neuen Modus gesetzt. | already\_applied\_or\_tested. | Setzt einen existierenden Modus. |
| set\_custom\_mode | Client \-\> Server | (wie enable\_head) | head: object (WlOutput), width: int32, height: int32, refresh: int32 | (wie enable\_head) | PendingOutputConfiguration für head wird auf benutzerdef. Modus gesetzt. | already\_applied\_or\_tested. | Setzt einen benutzerdefinierten Modus. |
| set\_position | Client \-\> Server | (wie enable\_head) | head: object (WlOutput), x: int32, y: int32 | (wie enable\_head) | PendingOutputConfiguration für head wird auf neue Position gesetzt. | already\_applied\_or\_tested. | Setzt die Position eines Outputs. |
| set\_transform | Client \-\> Server | (wie enable\_head) | head: object (WlOutput), transform: uint (wl\_output::Transform) | (wie enable\_head) | PendingOutputConfiguration für head wird auf neue Transformation gesetzt. | already\_applied\_or\_tested. | Setzt die Transformation. |
| set\_scale | Client \-\> Server | (wie enable\_head) | head: object (WlOutput), scale: uint (Fixed-point 24.8) | (wie enable\_head) | PendingOutputConfiguration für head wird auf neue Skalierung gesetzt. | already\_applied\_or\_tested. | Setzt die Skalierung. |
| test | Client \-\> Server | (wie enable\_head) | \- | is\_applied\_or\_tested \== false. | is\_applied\_or\_tested \= true. succeeded oder failed wird gesendet. | already\_applied\_or\_tested. | Testet die pend. Konfiguration. |
| apply | Client \-\> Server | (wie enable\_head) | \- | is\_applied\_or\_tested \== false. | is\_applied\_or\_tested \= true. Konfiguration wird angewendet. succeeded, failed oder cancelled wird gesendet. Output-Serial wird ggf. aktualisiert & an Clients propagiert. | already\_applied\_or\_tested. | Wendet die pend. Konfiguration an. |
| succeeded | Server \-\> Client | \- | \- | test oder apply war erfolgreich. | Client weiß, dass Konfiguration gültig/angewendet ist. | \- | Konfiguration erfolgreich. |
| failed | Server \-\> Client | \- | \- | test oder apply ist fehlgeschlagen. | Client weiß, dass Konfiguration ungültig/nicht angewendet wurde. | \- | Konfiguration fehlgeschlagen. |
| cancelled | Server \-\> Client | \- | \- | apply wurde abgebrochen (z.B. Serial-Mismatch). | Client weiß, dass Konfiguration veraltet ist. | \- | Konfiguration abgebrochen. |

### **F. Fehlerbehandlung**

* OutputManagerError Enum (definiert in system/outputs/output\_manager/errors.rs):  
  Rust  
  use thiserror::Error;  
  use smithay::utils::Point; // Assuming Logical is part of Point's definition path  
  use wayland\_server::protocol::wl\_output;

  \#  
  pub enum OutputManagerError {  
      \#\[error("Invalid WlOutput reference provided by client.")\]  
      InvalidWlOutput,

      \#  
      InvalidModeForOutput,

      \#\[error("Configuration object has already been applied or tested and cannot be modified further.")\]  
      AlreadyProcessed,

      \#  
      BackendError(String),

      \#\[error("Client serial {client\_serial} does not match compositor output serial {server\_serial}; configuration cancelled.")\]  
      SerialMismatch { client\_serial: u32, server\_serial: u32 },

      \#\[error("Attempted to configure a non-existent or no longer available output: {output\_name}")\]  
      UnknownOutput { output\_name: String },

      \#  
      InvalidMode {  
          output\_name: String,  
          width: i32,  
          height: i32,  
          refresh: i32,  
      },

      \#\[error("Configuration test failed: {reason}")\]  
      TestFailed { reason: String },

      \#\[error("Configuration application failed: {reason}")\]  
      ApplyFailed { reason: String },

      \#\[error("Configuration was cancelled due to a concurrent output state change.")\]  
      Cancelled,

      \#\[error("A generic protocol error occurred: {0}")\]  
      ProtocolError(String), // For generic protocol violations by the client  
  }

**Tabelle: OutputManagerError Varianten**

| Variantenname | Beschreibung | Typischer Auslöser | Empfohlene Client-Aktion |
| :---- | :---- | :---- | :---- |
| InvalidWlOutput | Eine ungültige WlOutput-Referenz wurde vom Client bereitgestellt. | Client sendet eine Anfrage mit einer WlOutput-Ressource, die dem Compositor nicht (mehr) bekannt ist. | Client sollte seine Output-Liste aktualisieren. Protokollfehler. |
| InvalidModeForOutput | Der referenzierte ZwlrOutputModeV1 ist für den gegebenen WlOutput nicht gültig. | Client versucht, einen Modus zu setzen, der nicht zu den vom Output angebotenen Modi gehört. | Client sollte die Modi des Outputs erneut prüfen. Protokollfehler. |
| AlreadyProcessed | Das Konfigurationsobjekt wurde bereits angewendet oder getestet und kann nicht weiter modifiziert werden. | Client sendet eine Modifikationsanfrage (z.B. set\_mode) an ein zwlr\_output\_configuration\_v1-Objekt, nachdem bereits test() oder apply() darauf aufgerufen wurde. | Client muss ein neues Konfigurationsobjekt erstellen. Protokollfehler. |
| BackendError | Ein Fehler im DRM- oder Hardware-Backend während der Konfigurationsanwendung. | Fehler beim Aufruf von DRM ioctls oder anderen Backend-spezifischen Operationen. | Client kann versuchen, die Operation später erneut auszuführen oder eine einfachere Konfiguration wählen. Der Compositor sendet failed(). |
| SerialMismatch | Die Serialnummer des Clients stimmt nicht mit der des Compositors überein; Konfiguration abgebrochen. | Der Output-Zustand des Compositors hat sich geändert, seit der Client die Konfiguration erstellt hat. | Client muss seine Output-Informationen aktualisieren (auf head/done-Ereignisse warten) und eine neue Konfiguration erstellen. Der Compositor sendet cancelled(). |
| UnknownOutput | Versuch, einen nicht existierenden oder nicht mehr verfügbaren Output zu konfigurieren. | Client referenziert einen Output (z.B. per Name/ID intern), der nicht (mehr) existiert. | Client sollte seine Output-Liste aktualisieren. Der Compositor sendet failed() oder cancelled(). |
| InvalidMode | Ein ungültiger Modus (Dimensionen, Refresh-Rate) wurde für einen Output spezifiziert. | Client spezifiziert einen custom\_mode mit Werten, die vom Output oder Compositor nicht unterstützt werden. | Client sollte unterstützte Modi verwenden oder Parameter anpassen. Der Compositor sendet failed(). |
| TestFailed | Der Konfigurationstest ist fehlgeschlagen. | Die vorgeschlagene Konfiguration ist aus Sicht des Compositors ungültig (z.B. ungültige Modi, Überlappungen). | Client sollte die Konfiguration anpassen. Der Compositor sendet failed(). |
| ApplyFailed | Die Anwendung der Konfiguration ist fehlgeschlagen. | Die Konfiguration war zwar gültig, konnte aber aufgrund eines Backend-Fehlers oder eines Laufzeitproblems nicht angewendet werden. | Client kann es erneut versuchen oder eine andere Konfiguration wählen. Der Compositor sendet failed(). |
| Cancelled | Die Konfiguration wurde aufgrund einer gleichzeitigen Zustandsänderung des Outputs abgebrochen. | Typischerweise durch einen Serial-Mismatch bei apply() oder wenn sich der Output-Zustand während des apply-Vorgangs ändert. | Client muss seine Output-Informationen aktualisieren und eine neue Konfiguration erstellen. Der Compositor sendet cancelled(). |
| ProtocolError | Ein generischer Protokollfehler seitens des Clients. | Client sendet eine Anfrage, die gegen die Protokollregeln verstößt (z.B. falsche Argumente, falsche Reihenfolge). | Client-Fehler. Der Compositor kann die Client-Verbindung beenden. |

### **G. Detaillierte Implementierungsschritte (Zusammenfassung)**

1. **Global Setup:** OutputManagerModuleState initialisieren. Das zwlr\_output\_manager\_v1-Global erstellen und im Wayland-Display bekannt machen. GlobalDispatch für dieses Global implementieren, um Client-Bindungen zu handhaben.  
2. **Manager Request Handling:** Dispatch für ZwlrOutputManagerV1 implementieren.  
   * Bei create\_configuration: Eine neue PendingOutputConfiguration-Instanz (eingebettet in Arc\<Mutex\<...\>\>) erstellen, diese mit der neuen zwlr\_output\_configuration\_v1-Ressource als UserData assoziieren und in active\_configurations speichern. Die aktuelle compositor\_output\_serial in PendingOutputConfiguration speichern.  
   * Bei stop: Das Global aus dem Display entfernen.  
3. **Configuration Request Handling:** Dispatch für ZwlrOutputConfigurationV1 implementieren.  
   * Anfragen wie enable\_head, disable\_head, set\_mode, set\_custom\_mode, set\_position, set\_transform, set\_scale modifizieren den Zustand der assoziierten PendingOutputConfiguration. Vor jeder Modifikation prüfen, ob is\_applied\_or\_tested false ist; andernfalls einen Protokollfehler (already\_applied\_or\_tested) senden.  
4. **Test/Apply Logic:**  
   * Für test(): Die in PendingOutputConfiguration gespeicherten Änderungen validieren. Dies beinhaltet die Prüfung, ob die referenzierten Outputs und Modi existieren und gültig sind und ob die Gesamtkonfiguration plausibel ist (z.B. keine unmöglichen Überlappungen gemäß Compositor-Policy). Ergebnis mit succeeded() oder failed() an den Client senden. is\_applied\_or\_tested auf true setzen.  
   * Für apply(): Zuerst die PendingOutputConfiguration::serial mit der aktuellen compositor\_output\_serial vergleichen. Bei Abweichung cancelled() senden. Andernfalls Validierung wie bei test() durchführen. Wenn gültig, versuchen, die Änderungen auf die internen smithay::output::Output-Objekte anzuwenden (z.B. via output.change\_current\_state(...)). Bei Erfolg succeeded() senden, die compositor\_output\_serial inkrementieren und alle Manager-Clients über den neuen Zustand und die neue Serial informieren. Bei Fehlschlag (z.B. Backend-Fehler) versuchen, Änderungen zurückzurollen und failed() senden. is\_applied\_or\_tested auf true setzen.  
5. **Event Emission:**  
   * Wenn sich der Zustand eines smithay::output::Output ändert (z.B. durch Hotplug oder erfolgreiches apply), müssen alle gebundenen zwlr\_output\_manager\_v1-Clients aktualisierte head-Informationen und ein done-Ereignis mit der neuen compositor\_output\_serial erhalten.  
   * zwlr\_output\_configuration\_v1 sendet succeeded, failed oder cancelled als Antwort auf test oder apply.  
6. **State Synchronization:** Die compositor\_output\_serial ist der Schlüssel zur Konsistenzerhaltung. Sie wird bei jeder erfolgreichen Anwendung einer Konfiguration oder bei jeder vom Compositor initiierten Änderung des Output-Layouts (z.B. Hotplug) inkrementiert. Clients verwenden diese Serial, um sicherzustellen, dass ihre Konfigurationsanfragen auf dem aktuellen Stand basieren.

### **H. Interaktionen**

* **Compositor Core (AnvilState oder Äquivalent):** Stellt die Liste der smithay::output::Output-Objekte bereit, deren aktuellen Zustände (Modi, Positionen, etc.) und die aktuelle compositor\_output\_serial. Nimmt Anfragen zur Zustandsänderung von Outputs entgegen.  
* **DRM Backend (oder anderes Hardware-Backend):** Die apply()-Logik ruft letztendlich Funktionen des Backends auf, um physische Display-Eigenschaften zu ändern (z.B. via DRM ioctls für Modesetting, Positionierung über CRTC-Konfiguration).  
* **UI Layer (indirekt):** Display-Konfigurationswerkzeuge (z.B. ein Einstellungsdialog) sind die primären Clients dieses Protokolls. Sie nutzen es, um dem Benutzer die Kontrolle über die Display-Einstellungen zu ermöglichen.

### **I. Vertiefende Betrachtungen & Implikationen**

Die Implementierung des wlr-output-management-unstable-v1-Protokolls erfordert sorgfältige Beachtung der Atomarität von Konfigurationsänderungen und der Synchronisation des Client-Zustands mit dem Compositor.  
Die Semantik von test() und apply() 2 legt nahe, dass der Compositor in der Lage sein muss, einen vollständigen Satz von Output-Änderungen zu validieren, *bevor* er versucht, sie anzuwenden. Dies ist entscheidend, um zu verhindern, dass das System in einem inkonsistenten oder unbrauchbaren Display-Zustand verbleibt. Scheitert ein apply(), sollte idealerweise ein Rollback zum vorherigen Zustand erfolgen. Dies kann komplex sein, wenn das zugrundeliegende DRM-Backend nicht für alle relevanten Eigenschaften atomare Updates unterstützt oder wenn eine Sequenz von Änderungen erforderlich ist. Ein robuster Compositor muss hier entweder auf Backend-Fähigkeiten für atomare Commits zurückgreifen oder eine eigene Logik implementieren, um den aktuellen Hardware-Zustand zu lesen, Änderungen zu versuchen und bei Fehlschlägen einzelne Schritte zurückzunehmen – letzteres ist deutlich komplexer. Smithays DRM-Abstraktionen 3 zielen darauf ab, dies zu vereinfachen, aber die Atomaritätsanforderung des Protokolls stellt eine Herausforderung dar.  
Das Management von Serialnummern ist ein weiterer kritischer Aspekt. Das serial-Argument in create\_configuration und das done-Ereignis des Managers 2 ermöglichen es Clients zu erkennen, ob ihr Verständnis des Output-Layouts aktuell ist. Ändert sich das Output-Layout des Compositors (z.B. durch Hotplugging eines Monitors), nachdem ein Client ein done-Ereignis empfangen hat, aber bevor er create\_configuration aufruft, ermöglicht der Serialnummern-Mismatch dem Compositor, die Konfiguration effektiv abzubrechen (typischerweise durch Senden von cancelled bei apply()). Dies zwingt den Client, den Output-Zustand neu zu evaluieren, und verhindert Operationen auf einem veralteten Setup.  
Schließlich ist die zuverlässige Zuordnung von clientseitigen WlOutput-Ressourcen zu den internen smithay::output::Output-Instanzen des Compositors unerlässlich. Das Protokoll operiert mit WlOutput-Objekten. Der Compositor muss diese clientseitigen Ressourcen eindeutig seinen internen Repräsentationen der physischen Outputs zuordnen können, um Fähigkeiten abzufragen und Änderungen anzuwenden. Diese Zuordnung wird typischerweise etabliert, wenn das wl\_output-Global vom Client gebunden wird. Smithays UserData-Mechanismus oder interne Maps, die ObjectIds als Schlüssel verwenden, sind hierfür gängige Lösungen. Die Output-Struktur von Smithay selbst verwaltet die WlOutput-Globale für Clients.1

## **III. Ultra-Feinspezifikation: system::outputs::power\_manager (Wayland Output Power Management)**

### **A. Modulübersicht und Zweck**

* **Verantwortlichkeit:** Dieses Modul implementiert die serverseitige Logik für das Wayland-Protokoll wlr-output-power-management-unstable-v1. Es ermöglicht autorisierten Wayland-Clients, typischerweise übergeordneten Shell-Komponenten, den Energiezustand (z.B. An, Aus) einzelner Display-Ausgänge zu steuern.  
* **Interaktion:** Es interagiert mit den internen smithay::output::Output-Objekten des Compositors. Anfragen zur Änderung des Energiezustands werden in Operationen auf diesen Objekten übersetzt, die dann typischerweise mit dem DRM-Backend (z.B. mittels DPMS) interagieren, um die physische Hardware zu steuern.  
* **Schlüsselprotokollelemente:** zwlr\_output\_power\_manager\_v1, zwlr\_output\_power\_v1.  
* **Relevante Referenzmaterialien & Analyse:**  
  * 28 (Protokoll-XML), 5 (Protokollspezifikation): Dies sind die primären Quellen, die Anfragen, Ereignisse und Enums (on, off) definieren. 5 merkt an, dass Modusänderungen "sofort wirksam" sind.  
  * 29 (wayland-rs Changelog): Weist auf die Verfügbarkeit des Protokolls in wayland-protocols hin.  
  * 5 (wayland.app Übersicht): Allgemeine Beschreibung und Links.  
  * 30 (lib.rs Erwähnung): Zeigt, dass es sich um ein bekanntes Protokoll handelt. Die Analyse dieser Quellen ergibt, dass dieses Protokoll im Vergleich zum Output-Management-Protokoll einfacher ist und sich auf zwei Zustände (An/Aus) konzentriert. Die Herausforderung liegt in der korrekten Autorisierung von Anfragen (implizit, da für "spezielle Clients" gedacht) und der zuverlässigen Weitergabe von Zustandsänderungen an die zugrundeliegende Display-Hardware.

### **B. Entwicklungs-Submodule & Dateien**

* **1\. system::outputs::power\_manager::manager\_global**  
  * Dateien: system/outputs/power\_manager/manager\_global.rs  
  * Verantwortlichkeiten: Verwaltet das zwlr\_output\_power\_manager\_v1-Global, behandelt Client-Bindungen und leitet get\_output\_power-Anfragen weiter.  
* **2\. system::outputs::power\_manager::power\_control\_handler**  
  * Dateien: system/outputs/power\_manager/power\_control\_handler.rs  
  * Verantwortlichkeiten: Verwaltet zwlr\_output\_power\_v1-Instanzen. Behandelt set\_mode-Anfragen von Clients und sendet mode- oder failed-Ereignisse.  
* **3\. system::outputs::power\_manager::types**  
  * Dateien: system/outputs/power\_manager/types.rs  
  * Verantwortlichkeiten: Definiert Rust-Enums für zwlr\_output\_power\_v1::Mode (z.B. InternalPowerMode { On, Off }).  
* **4\. system::outputs::power\_manager::errors**  
  * Dateien: system/outputs/power\_manager/errors.rs  
  * Verantwortlichkeiten: Definiert OutputPowerError.

### **C. Schlüsseldatenstrukturen**

* OutputPowerManagerModuleState:  
  * power\_manager\_global: Option\<GlobalId\> (Smithay-Global für zwlr\_output\_power\_manager\_v1)  
  * active\_power\_controls: HashMap\<ObjectId, Arc\<Mutex\<OutputPowerControlState\>\>\> (Verfolgt aktive zwlr\_output\_power\_v1-Instanzen, Schlüssel ist die ObjectId der ZwlrOutputPowerV1-Ressource)  
* OutputPowerControlState: Repräsentiert den Zustand einer zwlr\_output\_power\_v1-Instanz.  
  * wl\_output\_resource: WlOutput (Die clientgebundene WlOutput-Ressource, für die diese Kontrolle gilt)  
  * compositor\_output\_name: String (Ein eindeutiger Bezeichner für das interne smithay::output::Output-Objekt, das diesem WlOutput entspricht)  
  * current\_mode: InternalPowerMode (Spiegelt den zuletzt erfolgreich gesetzten Modus wider)  
* InternalPowerMode (Rust Enum): On, Off.

**Tabelle: OutputPowerManager-Datenstrukturen**

| Struct/Enum Name | Felder (Name, Rust-Typ, nullable, Mutabilität) | Beschreibung | Korrespondierendes Wayland-Protokollelement/Konzept |
| :---- | :---- | :---- | :---- |
| OutputPowerManagerModuleState | power\_manager\_global: Option\<GlobalId\> (intern, veränderlich) \<br\> active\_power\_controls: HashMap\<ObjectId, Arc\<Mutex\<OutputPowerControlState\>\>\> (intern, veränderlich) | Hauptzustand des Moduls, verwaltet das Global und aktive Energiezustandskontrollen. | zwlr\_output\_power\_manager\_v1 |
| OutputPowerControlState | wl\_output\_resource: WlOutput (intern, unveränderlich nach Erstellung) \<br\> compositor\_output\_name: String (intern, unveränderlich nach Erstellung) \<br\> current\_mode: InternalPowerMode (intern, veränderlich) | Speichert den Zustand einer einzelnen Energiezustandskontrolle für einen bestimmten Output. | zwlr\_output\_power\_v1 |
| InternalPowerMode | On, Off | Rust-interne Repräsentation der Energiezustände. | zwlr\_output\_power\_v1::mode Enum (on, off) |

Diese Strukturen sind notwendig, um den Überblick über die globalen Dienste und die individuellen Steuerungsobjekte für jeden Output zu behalten. active\_power\_controls ermöglicht es, auf Anfragen zu einem spezifischen zwlr\_output\_power\_v1-Objekt zu reagieren und dessen Zustand (insbesondere den current\_mode) zu verwalten.

### **D. Protokollbehandlung: zwlr\_output\_power\_manager\_v1 (Interface Version: 1\)**

* **Smithay Handler:** Die Implementierung erfolgt über GlobalDispatch\<ZwlrOutputPowerManagerV1, GlobalData, YourCompositorState\> und Dispatch\<ZwlrOutputPowerManagerV1, UserData, YourCompositorState\> für OutputPowerManagerModuleState. GlobalData ist hier typischerweise leer. UserData für den Manager ist ebenfalls oft nicht komplex.  
* **Anfrage: get\_output\_power(id: New\<ZwlrOutputPowerV1\>, output: WlOutput)**  
  * Rust Signatur (innerhalb des Dispatch-Traits für den Manager):  
    Rust  
    fn request(  
        \&mut self,  
        client: \&Client,  
        manager: \&ZwlrOutputPowerManagerV1,  
        request: zwlr\_output\_power\_manager\_v1::Request,  
        data: \&Self::UserData, // UserData des Managers  
        dhandle: \&DisplayHandle,  
        data\_init: \&mut DataInit\<'\_, YourCompositorState\>,  
    ) {  
        if let zwlr\_output\_power\_manager\_v1::Request::GetOutputPower { id, output: wl\_output\_resource } \= request {  
            //... Implementierungslogik...  
        }  
    }

  * Implementierung:  
    1. Identifiziere das interne smithay::output::Output-Objekt, das der vom Client übergebenen wl\_output\_resource entspricht. Dies geschieht typischerweise durch Abrufen von UserData, das mit der wl\_output\_resource assoziiert ist und den Namen oder eine ID des smithay::output::Output enthält. Wenn kein entsprechender interner Output gefunden wird, sollte das neu erstellte ZwlrOutputPowerV1-Objekt später ein failed-Ereignis senden.  
    2. Prüfe, ob bereits ein anderer Client die Energiekontrolle für diesen spezifischen wl\_output\_resource besitzt. Das Protokoll 5 deutet an, dass nur ein Client exklusive Kontrolle haben sollte ("Another client already has exclusive power management mode control"). Wenn ein Konflikt besteht, sollte das neu erstellte ZwlrOutputPowerV1-Objekt dem neuen Client ein failed-Ereignis senden, sobald es initialisiert ist oder bei der ersten set\_mode-Anfrage.  
    3. Erstelle eine neue Instanz von OutputPowerControlState. Der compositor\_output\_name wird auf den Bezeichner des internen Smithay-Outputs gesetzt. Der current\_mode wird durch Abfrage des tatsächlichen Energiezustands des physischen Outputs (z.B. über DRM DPMS) initialisiert.  
    4. Assoziiere diesen OutputPowerControlState (eingepackt in Arc\<Mutex\<...\>\>) mit der neuen id (New\<ZwlrOutputPowerV1\>) über data\_init.init(id, Arc::new(Mutex::new(power\_control\_state)));.  
    5. Sende unmittelbar nach der Erstellung des ZwlrOutputPowerV1-Objekts das initiale mode-Ereignis an den Client, das den aktuellen Energiezustand des Outputs widerspiegelt.5  
* **Anfrage: destroy()**  
  * Implementierung: Zerstört das zwlr\_output\_power\_manager\_v1-Global. Bestehende ZwlrOutputPowerV1-Objekte bleiben gemäß Protokoll 5 gültig. Das Entfernen des Globals aus dem DisplayHandle verhindert, dass neue Clients binden.

**Tabelle: zwlr\_output\_power\_manager\_v1 Interface-Behandlung**

| Anfrage/Ereignis | Richtung | Smithay Handler Signatur (Beispiel) | Parameter (Name, Wayland-Typ, Rust-Typ) | Vorbedingungen | Nachbedingungen | Fehlerbedingungen | Beschreibung |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| get\_output\_power | Client \-\> Server | Dispatch::request (match auf Request::GetOutputPower) | id: new\_id (New\<ZwlrOutputPowerV1\>), output: object (WlOutput) | Manager-Global existiert. output ist ein gültiges WlOutput-Objekt. | Neues ZwlrOutputPowerV1-Objekt erstellt und mit OutputPowerControlState assoziiert. Initiales mode-Ereignis wird an das neue Objekt gesendet. | Protokollfehler bei ungültiger ID. Interner Fehler, wenn output nicht zugeordnet werden kann (führt zu failed auf dem neuen Objekt). | Erstellt ein Energiekontroll-Objekt für einen Output. |
| destroy | Client \-\> Server | Dispatch::request (match auf Request::Destroy) | \- | Manager-Global existiert. | Manager-Global wird für neue Bindungen deaktiviert/zerstört. | \- | Zerstört das Manager-Objekt. |

### **E. Protokollbehandlung: zwlr\_output\_power\_v1 (Interface Version: 1\)**

* **Smithay Handler:** impl Dispatch\<ZwlrOutputPowerV1, Arc\<Mutex\<OutputPowerControlState\>\>, YourCompositorState\> for OutputPowerManagerModuleState. Die UserData ist hier der Arc\<Mutex\<OutputPowerControlState\>\>, der bei get\_output\_power erstellt wurde.  
* **Anfrage vom Client: set\_mode(mode: zwlr\_output\_power\_v1::Mode)**  
  * Implementierung:  
    1. Sperre den Mutex des OutputPowerControlState, um exklusiven Zugriff zu erhalten.  
    2. Übersetze das mode-Enum des Protokolls (On oder Off) in einen internen Steuerungswert (z.B. einen DPMS-Zustand für das DRM-Backend).  
    3. Versuche, diesen Energiezustand auf den physischen Output anzuwenden. Dies geschieht durch einen Aufruf an das entsprechende Backend (z.B. DRM-Backend, um den DPMS-Status zu setzen). Der compositor\_output\_name im OutputPowerControlState wird verwendet, um den korrekten internen smithay::output::Output zu identifizieren.  
    4. Wenn die Backend-Operation erfolgreich war:  
       * Aktualisiere OutputPowerControlState::current\_mode mit dem neuen Zustand.  
       * Sende das mode(actual\_new\_mode)-Ereignis über die ZwlrOutputPowerV1-Ressource an den Client. Der actual\_new\_mode sollte dem angeforderten Modus entsprechen.  
    5. Wenn die Backend-Operation fehlschlägt (z.B. der Output unterstützt den Modus nicht, ein Fehler im Backend tritt auf):  
       * Sende das failed()-Ereignis über die ZwlrOutputPowerV1-Ressource an den Client.  
* **Anfrage vom Client: destroy()**  
  * Implementierung: Die Dispatch::destroyed-Methode wird von Smithay aufgerufen, wenn der Client die Ressource zerstört. Hier wird der OutputPowerControlState aus der active\_power\_controls-Map im OutputPowerManagerModuleState entfernt, um Ressourcen freizugeben und sicherzustellen, dass keine veralteten Kontrollen mehr existieren.  
* **Ereignisse an den Client:**  
  * mode(mode: zwlr\_output\_power\_v1::Mode): Gesendet bei erfolgreicher set\_mode-Anfrage oder bei der Erstellung des ZwlrOutputPowerV1-Objekts, um den initialen Zustand zu übermitteln.  
  * failed(): Gesendet, wenn set\_mode fehlschlägt, der referenzierte Output ungültig wird (z.B. abgesteckt) oder ein anderer Client bereits die exklusive Kontrolle hat.

**Tabelle: zwlr\_output\_power\_v1 Interface-Behandlung**

| Anfrage/Ereignis | Richtung | Smithay Handler Signatur (Beispiel) | Parameter (Name, Wayland-Typ, Rust-Typ) | Vorbedingungen | Nachbedingungen | Fehlerbedingungen | Beschreibung |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| set\_mode | Client \-\> Server | Dispatch::request (match auf Request::SetMode) | mode: uint (zwlr\_output\_power\_v1::Mode) | ZwlrOutputPowerV1-Objekt existiert und ist gültig. | Energiezustand des Outputs wird geändert. mode oder failed Ereignis wird gesendet. | Output unterstützt Modus nicht. Backend-Fehler. | Setzt den Energiezustand des Outputs. |
| destroy | Client \-\> Server | Dispatch::destroyed | \- | ZwlrOutputPowerV1-Objekt existiert. | Zugehöriger OutputPowerControlState wird bereinigt. | \- | Zerstört das Energiekontroll-Objekt. |
| mode | Server \-\> Client | \- (Intern ausgelöst durch set\_mode oder Initialisierung) | mode: uint (zwlr\_output\_power\_v1::Mode) | Erfolgreiche Modusänderung oder Initialisierung. | Client kennt den aktuellen Energiezustand. | \- | Meldet eine Änderung des Energiezustands. |
| failed | Server \-\> Client | \- (Intern ausgelöst bei Fehlern) | \- | set\_mode fehlgeschlagen, Output ungültig, oder Kontrollkonflikt. | Client weiß, dass das Objekt ungültig ist. | \- | Objekt ist nicht mehr gültig. |

### **F. Fehlerbehandlung**

* OutputPowerError Enum (definiert in system/outputs/power\_manager/errors.rs):  
  Rust  
  use thiserror::Error;

  \#  
  pub enum OutputPowerError {  
      \#\[error("Output {output\_name:?} does not support power management.")\]  
      OutputDoesNotSupportPowerManagement { output\_name: String },

      \#\[error("Failed to set power mode for output {output\_name:?} due to backend error: {reason}")\]  
      BackendSetModeFailed { output\_name: String, reason: String },

      \#\[error("Output {output\_name:?} is no longer available.")\]  
      OutputVanished { output\_name: String },

      \#\[error("Another client already has exclusive power management control for output {output\_name:?}.")\]  
      ExclusiveControlConflict { output\_name: String },

      \#\[error("Invalid WlOutput reference provided by client.")\]  
      InvalidWlOutput,

      \#\[error("A generic protocol error occurred: {0}")\]  
      ProtocolError(String),  
  }

**Tabelle: OutputPowerError Varianten**

| Variantenname | Beschreibung | Typischer Auslöser | Empfohlene Client-Aktion (via failed Event) |
| :---- | :---- | :---- | :---- |
| OutputDoesNotSupportPowerManagement | Der angegebene Output unterstützt keine Energieverwaltung. | set\_mode für einen Output, der dies nicht kann. | Client sollte das ZwlrOutputPowerV1-Objekt zerstören. |
| BackendSetModeFailed | Das Setzen des Energiemodus im Backend ist fehlgeschlagen. | DRM/Hardware-Fehler während des DPMS-Aufrufs. | Client kann es später erneut versuchen oder den Fehler protokollieren; Objekt zerstören. |
| OutputVanished | Der Output, auf den sich das Kontrollobjekt bezieht, ist nicht mehr verfügbar. | Monitor wurde abgesteckt. | Client sollte das ZwlrOutputPowerV1-Objekt zerstören. |
| ExclusiveControlConflict | Ein anderer Client hat bereits die exklusive Kontrolle über die Energieverwaltung dieses Outputs. | get\_output\_power wird für einen bereits kontrollierten Output von einem anderen Client aufgerufen. | Client sollte das ZwlrOutputPowerV1-Objekt zerstören. |
| InvalidWlOutput | Eine ungültige WlOutput-Referenz wurde vom Client bereitgestellt. | Client sendet eine WlOutput-Ressource, die dem Compositor nicht bekannt ist, an get\_output\_power. | Client sollte seine Output-Liste aktualisieren. Protokollfehler. |
| ProtocolError | Ein generischer Protokollfehler seitens des Clients. | Client sendet eine Anfrage, die gegen die Protokollregeln verstößt. | Client-Fehler. Der Compositor kann die Client-Verbindung beenden. |

### **G. Detaillierte Implementierungsschritte (Zusammenfassung)**

1. **Global Setup:** OutputPowerManagerModuleState initialisieren. Das zwlr\_output\_power\_manager\_v1-Global erstellen und im Wayland-Display bekannt machen. GlobalDispatch für dieses Global implementieren.  
2. **Manager Request Handling:** Dispatch für ZwlrOutputPowerManagerV1 implementieren.  
   * Bei get\_output\_power: Internes smithay::output::Output-Objekt identifizieren. Prüfen auf exklusive Kontrolle. OutputPowerControlState erstellen (den aktuellen Energiezustand vom Backend abfragen und speichern). Das neue ZwlrOutputPowerV1-Objekt mit diesem Zustand als UserData initialisieren. Initiales mode-Ereignis an den Client senden.  
3. **Power Control Request Handling:** Dispatch für ZwlrOutputPowerV1 implementieren.  
   * Bei set\_mode: Den angeforderten Modus an das Backend (DRM DPMS) weiterleiten. Bei Erfolg den internen Zustand aktualisieren und mode-Ereignis senden. Bei Fehlschlag failed-Ereignis senden.  
4. **Output Disappearance:** Wenn ein physischer Output entfernt wird (z.B. durch Hot-Unplugging, das vom DRM-Modul erkannt wird), müssen alle zugehörigen ZwlrOutputPowerV1-Objekte ein failed-Ereignis erhalten. Der OutputPowerControlState für diesen Output sollte dann aus active\_power\_controls entfernt werden.  
5. **Compositor-Initiated Power Changes:** Wenn der Compositor selbst den Energiezustand eines Outputs ändert (z.B. durch eine Idle-Policy), muss er den current\_mode im entsprechenden OutputPowerControlState aktualisieren und ein mode-Ereignis an den gebundenen Client senden.

### **H. Interaktionen**

* **Compositor Core (AnvilState oder Äquivalent):** Stellt Zugriff auf smithay::output::Output-Instanzen und deren Zuordnung zu WlOutput-Ressourcen bereit. Benachrichtigt dieses Modul möglicherweise über das Verschwinden von Outputs.  
* **DRM Backend:** Wird aufgerufen, um DPMS-Zustände (Display Power Management Signaling) oder äquivalente hardwarenahe Energiesparfunktionen zu setzen (z.B. DRM\_MODE\_DPMS\_ON, DRM\_MODE\_DPMS\_OFF).  
* **Domain Layer:** Kann Energiesparrichtlinien auslösen (z.B. Bildschirm nach Inaktivität ausschalten), indem es entweder direkt D-Bus-Dienste aufruft, die dann dieses Protokoll verwenden könnten (wenn die Shell ein Client ist), oder indem es eine interne API des System-Layers aufruft, die letztendlich dieses Modul zur Steuerung der Output-Energie verwendet.

### **I. Vertiefende Betrachtungen & Implikationen**

Die Implementierung des wlr-output-power-management-unstable-v1-Protokolls ist im Vergleich zum Output-Konfigurationsprotokoll geradliniger, birgt aber eigene spezifische Herausforderungen in Bezug auf Exklusivität und Synchronisation mit dem tatsächlichen Hardwarezustand.  
Die Protokollbeschreibung 5 legt nahe, dass Änderungen des Energiemodus "sofort wirksam" sind. Dies impliziert eine direkte Interaktion mit der Hardware ohne eine vorgelagerte Testphase, wie sie bei wlr-output-management existiert. Für den Compositor bedeutet dies, dass bei einer set\_mode-Anfrage unmittelbar versucht werden muss, den Hardwarezustand zu ändern. Die Komplexität der Zustandsverwaltung reduziert sich dadurch, da keine komplexen pendelnden Zustände für eine Testphase vorgehalten werden müssen. Die Rückmeldung an den Client ist binär: Entweder die Aktion war erfolgreich (signalisiert durch ein mode-Ereignis mit dem neuen Zustand) oder sie schlug fehl (signalisiert durch ein failed-Ereignis).  
Das failed-Ereignis 5 dient als umfassender Fehlermechanismus. Es wird nicht nur bei direkten Fehlschlägen von set\_mode verwendet, sondern auch, wenn der zugrundeliegende Output ungültig wird (z.B. durch Abstecken des Monitors) oder wenn ein anderer Client bereits die exklusive Kontrolle über den Energiezustand des Outputs hat. Dies erfordert vom Compositor eine proaktive Überwachung des Zustands der physischen Outputs. Bei Änderungen, wie dem Entfernen eines Outputs, muss der Compositor alle assoziierten zwlr\_output\_power\_v1-Objekte identifizieren und ihnen ein failed-Ereignis senden. Dies stellt sicher, dass Clients darüber informiert werden, dass ihre Kontrollobjekte nicht mehr gültig sind und zerstört werden sollten.  
Ein weiterer wichtiger Aspekt ist die Möglichkeit, dass der Compositor selbst den Energiezustand eines Outputs ändert, unabhängig von Client-Anfragen über dieses Protokoll (z.B. aufgrund einer systemweiten Idle-Richtlinie). Das Protokoll 5 spezifiziert, dass das mode-Ereignis auch gesendet wird, wenn "der Compositor entscheidet, den Modus eines Outputs zu ändern". Wenn also die interne Logik des Compositors einen Bildschirm ausschaltet, muss dies im OutputPowerControlState des betroffenen Outputs reflektiert und ein entsprechendes mode-Ereignis an alle gebundenen zwlr\_output\_power\_v1-Clients gesendet werden. Dies gewährleistet, dass Clients stets über den aktuellen Energiezustand des Outputs informiert sind, auch wenn die Änderung nicht durch sie initiiert wurde.

## **IV. Ultra-Feinspezifikation: system::dbus::upower\_interface (UPower D-Bus Client)**

### **A. Modulübersicht und Zweck**

* **Verantwortlichkeit:** Dieses Modul stellt eine Schnittstelle zum org.freedesktop.UPower-D-Bus-Dienst bereit. Es ist dafür zuständig, den Systemstromstatus zu überwachen, einschließlich Batteriestand, Netzteilverbindung und den Zustand des Laptopdeckels (geöffnet/geschlossen).  
* **Informationsbereitstellung:** Die gesammelten Informationen werden anderen Teilen der Desktop-Umgebung zur Verfügung gestellt. Beispielsweise kann die Benutzeroberflächenschicht diese Daten für Batterieanzeigen oder Warnungen bei niedrigem Akkustand nutzen, während die Domänenschicht sie für die Implementierung von Energiesparrichtlinien verwenden kann.  
* **Relevante Referenzmaterialien & Analyse:**  
  * 31 (UPower D-Bus ref.xml), 6 (UPower Interface-Details), 32 (UPower Methoden/Signale), 32 (UPower D-Bus API Referenz): Diese Dokumente beschreiben die D-Bus-Schnittstelle von UPower, einschließlich der relevanten Objekte, Methoden (EnumerateDevices, GetDisplayDevice), Signale (DeviceAdded, DeviceRemoved, PropertiesChanged) und Eigenschaften (OnBattery, LidIsClosed, Percentage, State, TimeToEmpty, TimeToFull).  
  * 11 (PropertiesChanged-Signal), 33 (DeviceAdded-Signal), 34 (DeviceRemoved-Signal): Spezifische Details zu wichtigen Signalen.  
  * zbus-Snippets 8: Diese demonstrieren die allgemeine Verwendung der zbus-Bibliothek für die D-Bus-Kommunikation, einschließlich Proxy-Generierung, Methodenaufrufe und Signalbehandlung, was direkt auf die Implementierung dieses Moduls anwendbar ist. Die Analyse dieser Quellen zeigt, dass dieses Modul zbus verwenden wird, um Proxys für die Interfaces org.freedesktop.UPower und org.freedesktop.UPower.Device zu generieren. Es muss eine Verbindung zum System-Bus herstellen, Geräte auflisten, das "Display-Gerät" abrufen und Signale wie PropertiesChanged auf relevanten Geräteobjekten sowie DeviceAdded/DeviceRemoved auf dem Manager-Objekt abonnieren.

### **B. Entwicklungs-Submodule & Dateien**

* **1\. system::dbus::upower\_interface::client**  
  * Dateien: system/dbus/upower\_interface/client.rs  
  * Verantwortlichkeiten: Verwaltet die D-Bus-Verbindung, Proxy-Objekte, Methodenaufrufe und die Behandlung von Signalen. Enthält die Hauptlogik des UPower-Clients.  
* **2\. system::dbus::upower\_interface::types**  
  * Dateien: system/dbus/upower\_interface/types.rs  
  * Verantwortlichkeiten: Definiert Rust-Strukturen und Enums, die UPower-Daten abbilden (z.B. PowerDeviceDetails, PowerDeviceState, PowerSupplyType, UPowerManagerProperties). Diese Strukturen dienen der internen Repräsentation der von D-Bus erhaltenen Daten.  
* **3\. system::dbus::upower\_interface::errors**  
  * Dateien: system/dbus/upower\_interface/errors.rs  
  * Verantwortlichkeiten: Definiert das UPowerError-Enum für spezifische Fehler dieses Moduls.

### **C. Schlüsseldatenstrukturen**

* UPowerClient: Hauptstruktur des Moduls, die den Zustand des UPower-Clients verwaltet.  
  * connection: zbus::Connection (Die aktive D-Bus-Verbindung)  
  * manager\_proxy: Arc\<UPowerManagerProxy\> (Proxy für org.freedesktop.UPower)  
  * devices: Arc\<Mutex\<HashMap\<ObjectPath\<'static\>, PowerDeviceDetails\>\>\> (Speichert Details zu allen bekannten Energiegeräten, geschützt durch einen Mutex für thread-sicheren Zugriff)  
  * display\_device\_path: Arc\<Mutex\<Option\<ObjectPath\<'static\>\>\>\> (Pfad zum "Display Device")  
  * manager\_properties: Arc\<Mutex\<UPowerManagerProperties\>\> (Aktuelle Eigenschaften des UPower-Managers wie OnBattery, LidIsClosed, LidIsPresent)  
  * internal\_event\_sender: tokio::sync::broadcast::Sender\<UPowerEvent\> (Sender für interne Ereignisse)  
* UPowerManagerProperties: Speichert die Eigenschaften des org.freedesktop.UPower-Managers.  
  * daemon\_version: String  
  * on\_battery: bool  
  * lid\_is\_closed: bool  
  * lid\_is\_present: bool  
* PowerDeviceDetails (Rust-Struktur zur Abbildung von org.freedesktop.UPower.Device-Eigenschaften):  
  * object\_path: ObjectPath\<'static\>  
  * vendor: String  
  * model: String  
  * kind: PowerSupplyType (Rust Enum, das uint32 UPowerDeviceLevel abbildet: Unknown, None, LinePower, Battery, Ups, Monitor, Mouse, Keyboard, Pda, Phone, GamingInput, BluetoothGeneric, Tablet, Camera, PortableAudioPlayer, Toy, Computer, Wireless, Last)  
  * percentage: f64  
  * state: PowerDeviceState (Rust Enum, das uint32 UPowerDeviceState abbildet: Unknown, Charging, Discharging, Empty, FullyCharged, PendingCharge, PendingDischarge)  
  * time\_to\_empty: Option\<std::time::Duration\>  
  * time\_to\_full: Option\<std::time::Duration\>  
  * icon\_name: String  
  * is\_rechargeable: bool  
  * capacity: f64 (in Prozent, normalisierte Kapazität)  
  * technology: PowerDeviceTechnology (Rust Enum: Unknown, LithiumIon, LithiumPolymer, LithiumIronPhosphate, LeadAcid, NickelCadmium, NickelMetalHydride)  
  * temperature: Option\<f64\> (in Grad Celsius)  
  * serial: String  
* UPowerEvent (internes Event-Enum):  
  * DeviceAdded { path: ObjectPath\<'static\>, details: PowerDeviceDetails }  
  * DeviceRemoved { path: ObjectPath\<'static\> }  
  * DeviceUpdated { path: ObjectPath\<'static\>, details: PowerDeviceDetails }  
  * ManagerPropertiesChanged { properties: UPowerManagerProperties }

**Tabelle: UPower Interface-Datenstrukturen**

| Struct/Enum Name | Felder (Name, Rust-Typ, nullable, Mutabilität) | Beschreibung | Korrespondierendes D-Bus-Element/Konzept |
| :---- | :---- | :---- | :---- |
| UPowerClient | connection: zbus::Connection \<br\> manager\_proxy: Arc\<UPowerManagerProxy\> \<br\> devices: Arc\<Mutex\<HashMap\<ObjectPath\<'static\>, PowerDeviceDetails\>\>\> \<br\> display\_device\_path: Arc\<Mutex\<Option\<ObjectPath\<'static\>\>\>\> \<br\> manager\_properties: Arc\<Mutex\<UPowerManagerProperties\>\> \<br\> internal\_event\_sender: tokio::sync::broadcast::Sender\<UPowerEvent\> | Hauptclientstruktur, verwaltet Verbindung, Proxys und aggregierten Zustand. | Gesamte Interaktion mit UPower |
| UPowerManagerProperties | daemon\_version: String \<br\> on\_battery: bool \<br\> lid\_is\_closed: bool \<br\> lid\_is\_present: bool | Speichert die Eigenschaften des UPower-Managers. | Eigenschaften von org.freedesktop.UPower |
| PowerDeviceDetails | object\_path: ObjectPath\<'static\> \<br\> vendor: String \<br\> model: String \<br\> kind: PowerSupplyType \<br\> percentage: f64 \<br\> state: PowerDeviceState \<br\> time\_to\_empty: Option\<Duration\> \<br\> time\_to\_full: Option\<Duration\> \<br\> icon\_name: String \<br\> is\_rechargeable: bool \<br\> capacity: f64 \<br\> technology: PowerDeviceTechnology \<br\> temperature: Option\<f64\> \<br\> serial: String | Detaillierte Informationen über ein einzelnes Energiegerät. | Eigenschaften von org.freedesktop.UPower.Device |
| PowerSupplyType (Enum) | Varianten wie LinePower, Battery, etc. | Typ des Energieversorgungsgeräts. | Type Eigenschaft von org.freedesktop.UPower.Device (eine uint32) |
| PowerDeviceState (Enum) | Varianten wie Charging, Discharging, etc. | Aktueller Lade-/Entladezustand des Geräts. | State Eigenschaft von org.freedesktop.UPower.Device (eine uint32) |
| PowerDeviceTechnology (Enum) | Varianten wie LithiumIon, etc. | Technologie des Energiegeräts. | Technology Eigenschaft von org.freedesktop.UPower.Device (eine uint32) |
| UPowerEvent (Enum) | DeviceAdded, DeviceRemoved, DeviceUpdated, ManagerPropertiesChanged | Interne Ereignisse zur Signalisierung von Zustandsänderungen. | D-Bus Signale von UPower |

Die sorgfältige Definition dieser Rust-Strukturen und Enums ist entscheidend, um die über D-Bus empfangenen Daten typsicher und ergonomisch in der Rust-Umgebung zu verarbeiten. Die Verwendung von Arc\<Mutex\<...\>\> für gemeinsam genutzte Zustände wie devices und manager\_properties ist notwendig, um thread-sicheren Zugriff aus asynchronen Signal-Handlern zu gewährleisten. Der tokio::sync::broadcast::Sender ermöglicht es, interne Zustandsänderungen an andere Teile des Systems zu propagieren.

### **D. D-Bus Interface Proxys (Generiert durch zbus::proxy)**

* UPowerManagerProxy für org.freedesktop.UPower auf /org/freedesktop/UPower.  
  * Methoden:  
    * async fn enumerate\_devices(\&self) \-\> zbus::Result\<Vec\<ObjectPath\<'static\>\>\>; 6  
    * async fn get\_display\_device(\&self) \-\> zbus::Result\<ObjectPath\<'static\>\>; 6  
    * async fn get\_critical\_action(\&self) \-\> zbus::Result\<String\>; 6  
  * Eigenschaften (mittels \#\[zbus(property)\] auf Getter-Methoden):  
    * async fn daemon\_version(\&self) \-\> zbus::Result\<String\>; 6  
    * async fn on\_battery(\&self) \-\> zbus::Result\<bool\>; 6  
    * async fn lid\_is\_closed(\&self) \-\> zbus::Result\<bool\>; 6  
    * async fn lid\_is\_present(\&self) \-\> zbus::Result\<bool\>; 6  
  * Signale (mittels \#\[zbus(signal)\] auf Handler-Methoden im Trait, die dann Streams zurückgeben):  
    * async fn receive\_device\_added(\&self) \-\> zbus::Result\<zbus::SignalStream\<'\_, ObjectPath\<'static\>\>\>; (für DeviceAdded(o object\_path)) 6  
    * async fn receive\_device\_removed(\&self) \-\> zbus::Result\<zbus::SignalStream\<'\_, ObjectPath\<'static\>\>\>; (für DeviceRemoved(o object\_path)) 6  
    * 6  
* UPowerDeviceProxy für org.freedesktop.UPower.Device auf gerätespezifischen Pfaden.  
  * Eigenschaften (Beispiele, alle als async fn name(\&self) \-\> zbus::Result\<Type\>;):  
    * vendor (String)  
    * model (String)  
    * type\_ (u32) \-\> wird zu PowerSupplyType gemappt  
    * percentage (f64)  
    * state (u32) \-\> wird zu PowerDeviceState gemappt  
    * time\_to\_empty (i64) \-\> wird zu Option\<Duration\> gemappt  
    * time\_to\_full (i64) \-\> wird zu Option\<Duration\> gemappt  
    * icon\_name (String)  
    * is\_rechargeable (bool)  
    * capacity (f64)  
    * technology (u32) \-\> wird zu PowerDeviceTechnology gemappt  
    * temperature (f64) (kann nicht vorhanden sein, daher Option\<f64\>)  
    * serial (String)  
  * Signal:  
    * async fn receive\_properties\_changed(\&self) \-\> zbus::Result\<zbus::SignalStream\<'\_, PropertiesChangedArgs\>\>;  
      * PropertiesChangedArgs struct:  
        Rust  
        \#  
        pub struct PropertiesChangedArgs {  
            pub interface\_name: String,  
            pub changed\_properties: std::collections::HashMap\<String, zbus::zvariant::OwnedValue\>,  
            pub invalidated\_properties: Vec\<String\>,  
        }  
        7

**Tabelle: UPower D-Bus Proxys und Member**

| Proxy Name | D-Bus Interface | Schlüsselelemente (Methoden/Eigenschaften/Signale) | Rust Signatur (Beispiel) | Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| UPowerManagerProxy | org.freedesktop.UPower | EnumerateDevices (Methode) | async fn enumerate\_devices(\&self) \-\> zbus::Result\<Vec\<ObjectPath\<'static\>\>\> | Listet alle bekannten Energiegeräte auf. |
|  |  | GetDisplayDevice (Methode) | async fn get\_display\_device(\&self) \-\> zbus::Result\<ObjectPath\<'static\>\> | Gibt den Pfad des primären Anzeigegeräts zurück. |
|  |  | OnBattery (Eigenschaft) | \#\[zbus(property)\] async fn on\_battery(\&self) \-\> zbus::Result\<bool\> | Gibt an, ob das System im Akkubetrieb läuft. |
|  |  | LidIsClosed (Eigenschaft) | \#\[zbus(property)\] async fn lid\_is\_closed(\&self) \-\> zbus::Result\<bool\> | Gibt an, ob der Laptopdeckel geschlossen ist. |
|  |  | DeviceAdded (Signal) | \#\[zbus(signal)\] async fn device\_added(\&self, device\_path: ObjectPath\<'static\>) \-\> zbus::Result\<()\>; (Stream-Methode: receive\_device\_added) | Wird gesendet, wenn ein neues Energiegerät hinzugefügt wird. |
|  |  | DeviceRemoved (Signal) | \#\[zbus(signal)\] async fn device\_removed(\&self, device\_path: ObjectPath\<'static\>) \-\> zbus::Result\<()\>; (Stream-Methode: receive\_device\_removed) | Wird gesendet, wenn ein Energiegerät entfernt wird. |
| UPowerDeviceProxy | org.freedesktop.UPower.Device | Percentage (Eigenschaft) | \#\[zbus(property)\] async fn percentage(\&self) \-\> zbus::Result\<f64\> | Aktueller Ladestand in Prozent. |
|  |  | State (Eigenschaft) | \#\[zbus(property)\] async fn state(\&self) \-\> zbus::Result\<u32\> | Aktueller Zustand des Geräts (Laden, Entladen, etc.). |
|  |  | TimeToEmpty (Eigenschaft) | \#\[zbus(property)\] async fn time\_to\_empty(\&self) \-\> zbus::Result\<i64\> | Geschätzte verbleibende Zeit bis leer (Sekunden). |
|  |  | PropertiesChanged (Signal) | \#\[zbus(signal)\] async fn properties\_changed(\&self, interface\_name: String, changed\_properties: HashMap\<String, zvariant::OwnedValue\>, invalidated\_properties: Vec\<String\>) \-\> zbus::Result\<()\>; (Stream-Methode: receive\_properties\_changed) | Wird gesendet, wenn sich Eigenschaften des Geräts ändern. |

Diese Tabellenstruktur verdeutlicht die direkte Abbildung zwischen den D-Bus-Spezifikationen und der Rust-Proxy-Implementierung, was für Entwickler, die diese Schnittstelle nutzen oder erweitern müssen, von großem Wert ist. Die Verwendung des \#\[zbus(proxy)\]-Makros 9 automatisiert die Generierung des Boilerplate-Codes für diese Proxys erheblich.

### **E. Fehlerbehandlung**

* UPowerError Enum (definiert in system/dbus/upower\_interface/errors.rs):  
  Rust  
  use thiserror::Error;  
  use zbus::zvariant::ObjectPath;

  \#  
  pub enum UPowerError {  
      \#  
      Connection(\#\[from\] zbus::Error),

      \#  
      ServiceUnavailable,

      \#  
      MethodCall { method: String, error: zbus::Error },

      \#\[error("Invalid data received from UPower service: {context}")\]  
      InvalidData { context: String },

      \#\[error("UPower device not found at path: {path}")\]  
      DeviceNotFound { path: String }, // Früher: path: ObjectPath\<'static\> \- String ist einfacher für Display

      \#  
      SignalSubscriptionFailed { signal\_name: String, error: zbus::Error },

      \#\[error("Internal error during UPower client operation: {0}")\]  
      Internal(String),  
  }

**Tabelle: UPowerError Varianten**

| Variantenname | Beschreibung | Typischer Auslöser |
| :---- | :---- | :---- |
| Connection | Fehler beim Herstellen der D-Bus-Verbindung oder allgemeiner D-Bus-Fehler. | zbus::Connection::system().await schlägt fehl; zugrundeliegende D-Bus-Fehler von zbus. |
| ServiceUnavailable | Der UPower-Dienst (org.freedesktop.UPower) ist auf dem System-Bus nicht erreichbar. | UPower-Daemon läuft nicht oder ist nicht korrekt registriert. |
| MethodCall | Fehler beim Aufrufen einer D-Bus-Methode auf einem UPower-Interface. | Methode existiert nicht, falsche Parameter, Dienst antwortet mit Fehler. |
| InvalidData | Ungültige oder unerwartete Daten vom UPower-Dienst empfangen. | Unerwartete Variant-Typen, Enum-Werte außerhalb des definierten Bereichs. |
| DeviceNotFound | Ein spezifisches UPower-Gerät konnte unter dem erwarteten Pfad nicht gefunden werden. | GetDisplayDevice gibt einen Pfad zurück, der nicht mehr gültig ist; veraltete Gerätepfade. |
| SignalSubscriptionFailed | Fehler beim Abonnieren eines D-Bus-Signals von UPower. | Probleme mit Match-Regeln, Dienst unterstützt Signal nicht wie erwartet. |
| Internal | Ein interner Fehler im UPower-Client-Modul. | Logische Fehler in der Client-Implementierung. |

### **F. Detaillierte Implementierungsschritte**

1. **Proxy-Definitionen:** Definiere die Rust-Traits UPowerManagerProxy und UPowerDeviceProxy mit dem \#\[zbus::proxy\]-Attribut, die die Methoden, Eigenschaften und Signale der entsprechenden D-Bus-Interfaces (org.freedesktop.UPower und org.freedesktop.UPower.Device) abbilden.9  
2. **UPowerClient::connect\_and\_initialize() asynchrone Funktion:**  
   * Stelle eine Verbindung zum D-Bus System-Bus her: let connection \= zbus::Connection::system().await.map\_err(UPowerError::Connection)?;  
   * Erstelle den UPowerManagerProxy: let manager\_proxy \= Arc::new(UPowerManagerProxy::new(\&connection).await.map\_err(|e| UPowerError::MethodCall { method: "UPowerManagerProxy::new".to\_string(), error: e })?);  
   * Initialisiere devices: Arc\<Mutex\<HashMap\<ObjectPath\<'static\>, PowerDeviceDetails\>\>\> als leer.  
   * Initialisiere manager\_properties: Arc\<Mutex\<UPowerManagerProperties\>\> durch Abrufen aller Manager-Eigenschaften (daemon\_version, on\_battery, lid\_is\_closed, lid\_is\_present) über den manager\_proxy.  
   * Rufe manager\_proxy.enumerate\_devices().await auf. Für jeden zurückgegebenen ObjectPath:  
     * Erstelle einen UPowerDeviceProxy für diesen Pfad: let device\_proxy \= UPowerDeviceProxy::builder(\&connection).path(path.clone())?.build().await?;  
     * Rufe alle relevanten Eigenschaften dieses device\_proxy ab (z.B. percentage(), state(), kind(), time\_to\_empty(), time\_to\_full(), icon\_name(), vendor(), model(), etc.).  
     * Konvertiere die Rohdaten (z.B. u32 für state und kind) in die entsprechenden Rust-Enums (PowerDeviceState, PowerSupplyType). Konvertiere i64 Sekunden in Option\<Duration\>.  
     * Erstelle eine PowerDeviceDetails-Instanz und füge sie zur devices-HashMap hinzu.  
   * Rufe manager\_proxy.get\_display\_device().await auf und speichere den Pfad in display\_device\_path.  
   * Erstelle den tokio::sync::broadcast::channel für UPowerEvent.  
   * Gib eine UPowerClient-Instanz mit der Verbindung, den Proxys, dem initialen Zustand und dem Sender des Broadcast-Kanals zurück.  
3. **Signalbehandlung (in separaten tokio::spawn-Tasks oder integriert in einen Haupt-Event-Loop-Dispatcher):**  
   * **Manager-Signale:**  
     * Abonniere manager\_proxy.receive\_device\_added().await?. In der Schleife:  
       * Wenn ein DeviceAdded(path)-Signal empfangen wird: Erstelle einen neuen UPowerDeviceProxy für path, rufe alle seine Eigenschaften ab, erstelle PowerDeviceDetails, füge es zu devices (unter Mutex-Sperre) hinzu und sende ein UPowerEvent::DeviceAdded über den Broadcast-Kanal.  
     * Abonniere manager\_proxy.receive\_device\_removed().await?. In der Schleife:  
       * Wenn ein DeviceRemoved(path)-Signal empfangen wird: Entferne den Eintrag aus devices (unter Mutex-Sperre) und sende ein UPowerEvent::DeviceRemoved über den Broadcast-Kanal.  
     * Abonniere manager\_proxy.receive\_properties\_changed().await? (für Eigenschaften des Manager-Objekts selbst, wie OnBattery, LidIsClosed). In der Schleife:  
       * Aktualisiere die Felder in manager\_properties (unter Mutex-Sperre) basierend auf den changed\_properties im Signal.  
       * Sende ein UPowerEvent::ManagerPropertiesChanged über den Broadcast-Kanal.  
   * **Device-Signale (für jedes Gerät in devices):**  
     * Beim Hinzufügen eines Geräts (oder bei der Initialisierung), abonniere dessen device\_proxy.receive\_properties\_changed().await?. In der Schleife für jedes Gerät:  
       * Wenn ein PropertiesChanged-Signal für dieses Gerät empfangen wird:  
         * Extrahiere changed\_properties und invalidated\_properties aus den Signal-Argumenten.  
         * Aktualisiere die entsprechenden Felder in der PowerDeviceDetails-Instanz für dieses Gerät in der devices-HashMap (unter Mutex-Sperre). Achte auf die korrekte Deserialisierung der zbus::zvariant::OwnedValue.  
         * Sende ein UPowerEvent::DeviceUpdated mit dem Pfad und den aktualisierten Details über den Broadcast-Kanal.  
4. **Öffentliche Methoden auf UPowerClient:**  
   * fn is\_on\_battery(\&self) \-\> bool: Gibt den Wert aus self.manager\_properties zurück.  
   * fn is\_lid\_closed(\&self) \-\> bool: Gibt den Wert aus self.manager\_properties zurück.  
   * fn get\_all\_devices(\&self) \-\> Vec\<PowerDeviceDetails\>: Gibt eine Kopie der Werte aus self.devices zurück.  
   * fn get\_display\_device\_details(\&self) \-\> Option\<PowerDeviceDetails\>: Gibt die Details für das Gerät unter self.display\_device\_path zurück.  
   * fn subscribe\_events(\&self) \-\> tokio::sync::broadcast::Receiver\<UPowerEvent\>: Gibt einen neuen Empfänger für den internen Event-Kanal zurück.

### **G. Interaktionen**

* **Core Layer:** Stellt die async-Laufzeitumgebung (z.B. tokio) bereit, die für zbus und die asynchrone Signalbehandlung benötigt wird.  
* **Domain Layer:** Abonniert die von UPowerClient über den internen Event-Bus (Broadcast-Kanal) gesendeten UPowerEvent-Ereignisse. Nutzt diese Informationen, um Energiesparrichtlinien zu implementieren (z.B. Bildschirm dimmen bei niedrigem Akkustand, System in den Ruhezustand versetzen bei kritischem Akkustand, Aktionen bei geschlossenem Deckel).  
* **UI Layer:** Abonniert ebenfalls die UPowerEvent-Ereignisse. Verwendet die Informationen, um Energiestatusanzeigen (Batterie-Icon, verbleibende Zeit, Ladestatus), Warnungen und ggf. Einstellungsoptionen für Energieverwaltung darzustellen.  
* **Event Bus:** Der UPowerClient fungiert als Herausgeber von UPowerEvent-Ereignissen (DeviceAdded, DeviceRemoved, DeviceUpdated, ManagerPropertiesChanged) auf einem internen, systemweiten Event-Bus (hier implementiert mit tokio::sync::broadcast).

### **H. Vertiefende Betrachtungen & Implikationen**

Die Implementierung eines robusten UPower-Clients erfordert eine sorgfältige Handhabung von asynchronen Signalen und die korrekte Interpretation der feingranularen Eigenschaftsänderungen.  
UPower's PropertiesChanged-Signal 7 liefert detaillierte Informationen darüber, welche Eigenschaften sich geändert haben und welche ungültig geworden sind. Anstatt bei jedem Signal alle Eigenschaften eines Geräts neu abzufragen, sollte der Client die changed\_properties (ein Dictionary von Eigenschaftsnamen zu neuen Werten) und invalidated\_properties (eine Liste von Eigenschaftsnamen, deren Werte nicht mehr gültig sind) auswerten. Dies erfordert eine effiziente Aktualisierung der lokalen PowerDeviceDetails-Struktur, indem nur die betroffenen Felder modifiziert werden. Eine sorgfältige Zuordnung zwischen den D-Bus-Eigenschaftsnamen (Strings) und den Feldern der Rust-Struktur sowie eine robuste Deserialisierung der zbus::zvariant::Value-Typen sind hierbei unerlässlich. Dieser Ansatz minimiert die D-Bus-Kommunikation und verbessert die Reaktionsfähigkeit.  
Das Konzept des "Display Device" 6 unter /org/freedesktop/UPower/devices/DisplayDevice ist eine wichtige Abstraktion, die UPower für Desktop-Umgebungen bereitstellt. Es handelt sich um ein zusammengesetztes Gerät, das den Gesamtstatus der Energieversorgung repräsentiert, der typischerweise in der Benutzeroberfläche angezeigt wird. Obwohl dieses Gerät einen bequemen Zugriff auf aggregierte Informationen bietet, ist es für ein vollständiges Bild der Energieversorgung – insbesondere in Systemen mit mehreren Batterien oder komplexen Energiekonfigurationen – notwendig, dass der Client alle Geräte über EnumerateDevices erfasst und deren Zustand individuell überwacht. Die UI-Schicht wird wahrscheinlich primär das "Display Device" für ihre Hauptanzeige nutzen, aber die Systemschicht sollte über diesen Client Zugriff auf die Details aller einzelnen Geräte ermöglichen.  
Die asynchrone Natur der D-Bus-Signalbehandlung erfordert besondere Aufmerksamkeit bei der Verwaltung des gemeinsamen Zustands. Da Signale wie DeviceAdded oder PropertiesChanged für verschiedene Geräte potenziell gleichzeitig eintreffen und verarbeitet werden könnten (abhängig von der Konfiguration des async-Executors), muss der Zugriff auf gemeinsam genutzte Datenstrukturen wie die Liste der Geräte (devices in UPowerClient) synchronisiert werden. Die Verwendung von Arc\<Mutex\<...\>\> ist hier ein gängiges Muster in Rust, um Datenkorruption oder inkonsistente Lesezugriffe zu verhindern. Die internen Ereignisse, die dieses Modul über den Broadcast-Kanal aussendet, sollten entweder unveränderliche Momentaufnahmen der Daten transportieren, oder die Abonnenten dieser Ereignisse müssen ebenfalls für eine korrekte Synchronisation sorgen, falls sie auf gemeinsam genutzte Zustände zugreifen, die durch diese Ereignisse modifiziert werden könnten.

## **V. Ultra-Feinspezifikation: system::dbus::logind\_interface (Logind D-Bus Client)**

### **A. Modulübersicht und Zweck**

* **Verantwortlichkeit:** Dieses Modul interagiert mit den D-Bus-Diensten org.freedesktop.login1.Manager und org.freedesktop.login1.Session. Es überwacht Benutzersitzungen, den Status von "Seats" (logische Gruppierungen von Eingabe-/Ausgabegeräten) und Systemereignisse wie das Vorbereiten des Ruhezustands (PrepareForSleep) und das Aufwachen.  
* **Funktionen:** Es ermöglicht der Desktop-Umgebung, auf das Sperren/Entsperren von Sitzungen, Benutzerwechsel und das Vorbereiten des Systems auf den Ruhezustand zu reagieren. Es kann auch Aktionen wie das Anfordern einer Sitzungssperre initiieren.  
* **Relevante Referenzmaterialien & Analyse:**  
  * 12 (logind man page Übersicht), 13 (logind Manager Methoden), 13 (logind Manager Methoden/Signale): Geben einen Überblick über die org.freedesktop.login1.Manager-Schnittstelle, einschließlich Methoden wie GetSession, ListSessions, LockSession, UnlockSession, Inhibit und Signale wie SessionNew, SessionRemoved, PrepareForSleep.  
  * 14 (SessionNew/SessionRemoved Signale), 15 (PrepareForSleep Signal), 16 (Lock/Unlock Signale auf Session-Objekt): Spezifische Details zu wichtigen Signalen. Die Analyse dieser Quellen zeigt, dass dieses Modul zbus für die Interaktion mit logind nutzen wird. Zentrale Aspekte sind das Verfolgen der aktiven Sitzung, das Reagieren auf das PrepareForSleep-Signal zur Durchführung notwendiger Aktionen vor dem Suspend (und das zuverlässige Freigeben von Inhibit-Locks) sowie das Reagieren auf Lock/Unlock-Signale zur Steuerung des Sitzungszustands (z.B. Aktivierung des Sperrbildschirms).

### **B. Entwicklungs-Submodule & Dateien**

* **1\. system::dbus::logind\_interface::client**  
  * Dateien: system/dbus/logind\_interface/client.rs  
  * Verantwortlichkeiten: Hauptlogik des Logind-Clients, D-Bus-Verwaltung, Proxy-Interaktionen, Signalbehandlung.  
* **2\. system::dbus::logind\_interface::types**  
  * Dateien: system/dbus/logind\_interface/types.rs  
  * Verantwortlichkeiten: Definition von Rust-Strukturen und \-Enums zur Abbildung von Logind-Daten (z.B. SessionInfo, ActiveSessionState, SleepPreparationState).  
* **3\. system::dbus::logind\_interface::errors**  
  * Dateien: system/dbus/logind\_interface/errors.rs  
  * Verantwortlichkeiten: Definition des LogindError-Enums.

### **C. Schlüsseldatenstrukturen**

* LogindClient: Hauptstruktur des Moduls.  
  * connection: zbus::Connection  
  * manager\_proxy: Arc\<LogindManagerProxy\> (Proxy für org.freedesktop.login1.Manager)  
  * active\_session\_id: Arc\<Mutex\<Option\<String\>\>\> (ID der aktuellen aktiven Sitzung)  
  * active\_session\_path: Arc\<Mutex\<Option\<ObjectPath\<'static\>\>\>\>  
  * active\_session\_proxy: Arc\<Mutex\<Option\<LogindSessionProxy\>\>\> (Proxy für die aktive org.freedesktop.login1.Session)  
  * sleep\_inhibitor\_lock: Arc\<Mutex\<Option\<zbus::zvariant::OwnedFd\>\>\> (File Descriptor für den Sleep-Inhibitor-Lock)  
  * internal\_event\_sender: tokio::sync::broadcast::Sender\<LogindEvent\>  
* SessionInfo: Repräsentiert Informationen über eine Benutzersitzung.  
  * id: String  
  * user\_id: u32  
  * user\_name: String  
  * seat\_id: String  
  * object\_path: ObjectPath\<'static\>  
  * is\_active: bool  
  * is\_locked\_hint: bool (Basierend auf der LockedHint-Eigenschaft der Session)  
* LogindEvent (internes Event-Enum):  
  * PrepareForSleep { starting: bool }  
  * ActiveSessionLocked  
  * ActiveSessionUnlocked  
  * ActiveSessionChanged { new\_session\_id: Option\<String\> }  
  * SessionListChanged { sessions: Vec\<SessionInfo\> }

**Tabelle: Logind Interface-Datenstrukturen**

| Struct/Enum Name | Felder (Name, Rust-Typ, nullable, Mutabilität) | Beschreibung | Korrespondierendes D-Bus-Element/Konzept |
| :---- | :---- | :---- | :---- |
| LogindClient | connection: zbus::Connection \<br\> manager\_proxy: Arc\<LogindManagerProxy\> \<br\> active\_session\_id: Arc\<Mutex\<Option\<String\>\>\> \<br\> active\_session\_path: Arc\<Mutex\<Option\<ObjectPath\<'static\>\>\>\> \<br\> active\_session\_proxy: Arc\<Mutex\<Option\<LogindSessionProxy\>\>\> \<br\> sleep\_inhibitor\_lock: Arc\<Mutex\<Option\<zbus::zvariant::OwnedFd\>\>\> \<br\> internal\_event\_sender: tokio::sync::broadcast::Sender\<LogindEvent\> | Hauptclientstruktur, verwaltet Verbindung, Proxys, aktive Sitzungsinformationen und Inhibit-Locks. | Gesamte Interaktion mit Logind |
| SessionInfo | id: String \<br\> user\_id: u32 \<br\> user\_name: String \<br\> seat\_id: String \<br\> object\_path: ObjectPath\<'static\> \<br\> is\_active: bool \<br\> is\_locked\_hint: bool | Detaillierte Informationen über eine einzelne Benutzersitzung. | Struktur der Rückgabewerte von ListSessions und Eigenschaften von org.freedesktop.login1.Session |
| LogindEvent (Enum) | PrepareForSleep { starting: bool } \<br\> ActiveSessionLocked \<br\> ActiveSessionUnlocked \<br\> ActiveSessionChanged {... } \<br\> SessionListChanged {... } | Interne Ereignisse zur Signalisierung von Zustandsänderungen im Logind-Kontext. | D-Bus Signale von Logind (PrepareForSleep, Lock, Unlock auf Session-Objekt, SessionNew, SessionRemoved) |

Die LogindClient-Struktur kapselt die gesamte Logik für die Interaktion mit logind. Die active\_session\_id und der zugehörige Proxy sind zentral, da viele Aktionen sitzungsspezifisch sind. Der sleep\_inhibitor\_lock ist kritisch für die korrekte Handhabung von Suspend-Zyklen.

### **D. D-Bus Interface Proxys (Generiert durch zbus::proxy)**

* LogindManagerProxy für org.freedesktop.login1.Manager auf /org/freedesktop/login1.  
  * Methoden:  
    * async fn get\_session(\&self, session\_id: \&str) \-\> zbus::Result\<ObjectPath\<'static\>\>; 12  
    * async fn list\_sessions(\&self) \-\> zbus::Result\<Vec\<(String, u32, String, String, ObjectPath\<'static\>)\>\>; (session\_id, uid, user\_name, seat\_id, object\_path) 13  
    * async fn lock\_session(\&self, session\_id: \&str) \-\> zbus::Result\<()\>; 13  
    * async fn unlock\_session(\&self, session\_id: \&str) \-\> zbus::Result\<()\>; 13  
    * async fn inhibit(\&self, what: \&str, who: \&str, why: \&str, mode: \&str) \-\> zbus::Result\<zbus::zvariant::OwnedFd\>; (z.B. what: "sleep:shutdown:idle", who: "Desktop Environment", why: "Saving state", mode: "delay") 13  
  * Signale:  
    * async fn receive\_session\_new(\&self) \-\> zbus::Result\<zbus::SignalStream\<'\_, SessionNewArgs\>\>; (struct SessionNewArgs { session\_id: String, object\_path: ObjectPath\<'static\> }) 14  
    * async fn receive\_session\_removed(\&self) \-\> zbus::Result\<zbus::SignalStream\<'\_, SessionRemovedArgs\>\>; (struct SessionRemovedArgs { session\_id: String, object\_path: ObjectPath\<'static\> }) 14  
    * async fn receive\_prepare\_for\_sleep(\&self) \-\> zbus::Result\<zbus::SignalStream\<'\_, bool\>\>; (start: bool) 15  
* LogindSessionProxy für org.freedesktop.login1.Session auf sitzungsspezifischen Pfaden.  
  * Eigenschaften:  
    * \#\[zbus(property)\] async fn active(\&self) \-\> zbus::Result\<bool\>;  
    * \#\[zbus(property)\] async fn locked\_hint(\&self) \-\> zbus::Result\<bool\>;  
    * \#\[zbus(property)\] async fn id(\&self) \-\> zbus::Result\<String\>;  
    * \#\[zbus(property)\] async fn user(\&self) \-\> zbus::Result\<(u32, ObjectPath\<'static\>)\>; (uid, user\_path)  
    * \#\[zbus(property)\] async fn seat(\&self) \-\> zbus::Result\<(String, ObjectPath\<'static\>)\>; (seat\_id, seat\_path)  
  * Signale (die der Session-Manager der DE abhört, nicht unbedingt dieser Client direkt, aber relevant für das Verständnis):  
    * Lock() 16  
    * Unlock() 16

**Tabelle: Logind D-Bus Proxys und Member**

| Proxy Name | D-Bus Interface | Schlüsselelemente (Methoden/Eigenschaften/Signale) | Rust Signatur (Beispiel) | Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| LogindManagerProxy | org.freedesktop.login1.Manager | ListSessions (Methode) | async fn list\_sessions(\&self) \-\> zbus::Result\<Vec\<(String, u32, String, String, ObjectPath\<'static\>)\>\> | Listet alle aktuellen Benutzersitzungen auf. |
|  |  | LockSession (Methode) | async fn lock\_session(\&self, session\_id: \&str) \-\> zbus::Result\<()\> | Fordert das Sperren einer bestimmten Sitzung an. |
|  |  | Inhibit (Methode) | async fn inhibit(\&self, what: \&str, who: \&str, why: \&str, mode: \&str) \-\> zbus::Result\<zbus::zvariant::OwnedFd\> | Nimmt einen Inhibit-Lock, um Systemaktionen (z.B. Suspend) zu verzögern. |
|  |  | SessionNew (Signal) | \#\[zbus(signal)\] async fn session\_new(\&self, session\_id: String, object\_path: ObjectPath\<'static\>) \-\> zbus::Result\<()\>; | Wird gesendet, wenn eine neue Sitzung erstellt wird. |
|  |  | PrepareForSleep (Signal) | \#\[zbus(signal)\] async fn prepare\_for\_sleep(\&self, start: bool) \-\> zbus::Result\<()\>; | Wird gesendet, bevor das System in den Ruhezustand geht oder nachdem es aufwacht. |
| LogindSessionProxy | org.freedesktop.login1.Session | Active (Eigenschaft) | \#\[zbus(property)\] async fn active(\&self) \-\> zbus::Result\<bool\>; | Gibt an, ob die Sitzung aktiv ist. |
|  |  | LockedHint (Eigenschaft) | \#\[zbus(property)\] async fn locked\_hint(\&self) \-\> zbus::Result\<bool\>; | Gibt an, ob die Sitzung als gesperrt markiert ist. |
|  |  | Lock (Signal) | \#\[zbus(signal)\] async fn lock(\&self) \-\> zbus::Result\<()\>; | Signalisiert, dass die Sitzung gesperrt werden soll (wird vom Session-Manager empfangen). |

### **E. Fehlerbehandlung**

* LogindError Enum (definiert in system/dbus/logind\_interface/errors.rs):  
  Rust  
  use thiserror::Error;  
  use zbus::zvariant::OwnedObjectPath; // Korrigiert von ObjectPath zu OwnedObjectPath für SignalArgs

  \#  
  pub enum LogindError {  
      \#  
      Connection(\#\[from\] zbus::Error),

      \#  
      ServiceUnavailable,

      \#  
      MethodCall { method: String, error: zbus::Error },

      \#  
      SessionNotFound { session\_id: String },

      \#\[error("Failed to take inhibitor lock from logind: {reason}")\]  
      InhibitFailed { reason: String },

      \#\[error("No active session found for this desktop environment.")\]  
      NoActiveSession,

      \#  
      SignalSubscriptionFailed { signal\_name: String, error: zbus::Error },

      \#\[error("Internal error during logind client operation: {0}")\]  
      Internal(String),  
  }

**Tabelle: LogindError Varianten**

| Variantenname | Beschreibung | Typischer Auslöser |
| :---- | :---- | :---- |
| Connection | Fehler beim Herstellen der D-Bus-Verbindung oder allgemeiner D-Bus-Fehler. | zbus::Connection::system().await schlägt fehl. |
| ServiceUnavailable | Der Logind-Dienst ist auf dem System-Bus nicht erreichbar. | systemd-logind läuft nicht oder ist nicht korrekt registriert. |
| MethodCall | Fehler beim Aufrufen einer D-Bus-Methode auf einem Logind-Interface. | Methode existiert nicht, falsche Parameter, Dienst antwortet mit Fehler. |
| SessionNotFound | Eine Sitzung mit der angegebenen ID konnte nicht gefunden werden. | LockSession mit einer ungültigen ID aufgerufen. |
| InhibitFailed | Fehler beim Anfordern eines Inhibit-Locks von Logind. | Logind verweigert den Lock (z.B. keine Berechtigung, ungültige Parameter). |
| NoActiveSession | Es konnte keine aktive Sitzung für die laufende Desktop-Umgebung identifiziert werden. | Fehler bei der Logik zur Erkennung der aktiven Sitzung. |
| SignalSubscriptionFailed | Fehler beim Abonnieren eines D-Bus-Signals von Logind. | Probleme mit Match-Regeln. |
| Internal | Ein interner Fehler im Logind-Client-Modul. | Logische Fehler in der Client-Implementierung. |

### **F. Detaillierte Implementierungsschritte**

1. **Proxy-Definitionen:** Definiere die Rust-Traits LogindManagerProxy und LogindSessionProxy mit dem \#\[zbus::proxy\]-Attribut für die D-Bus-Interfaces org.freedesktop.login1.Manager und org.freedesktop.login1.Session.  
2. **LogindClient::connect\_and\_initialize() asynchrone Funktion:**  
   * Stelle Verbindung zum D-Bus System-Bus her und erstelle LogindManagerProxy.  
   * Identifiziere die aktive Sitzung:  
     * Rufe manager\_proxy.list\_sessions().await auf.  
     * Iteriere durch die Liste der Sessions. Für jede Session, erstelle temporär einen LogindSessionProxy für deren ObjectPath.  
     * Rufe die active().await-Eigenschaft auf diesem Session-Proxy ab.  
     * Die erste Session mit active \== true (und idealerweise passendem seat\_id, falls bekannt) wird als die aktive Sitzung betrachtet. Speichere deren session\_id, object\_path und den LogindSessionProxy in den Arc\<Mutex\<...\>\>-Feldern von LogindClient.  
     * Wenn keine aktive Sitzung gefunden wird, gib LogindError::NoActiveSession zurück.  
   * Erstelle den tokio::sync::broadcast::channel für LogindEvent.  
   * Gib eine LogindClient-Instanz zurück.  
3. **Signalbehandlung (in separaten tokio::spawn-Tasks):**  
   * **PrepareForSleep-Signal:**  
     * Abonniere manager\_proxy.receive\_prepare\_for\_sleep().await?.  
     * In der Signal-Schleife:  
       * Wenn start \== true (System bereitet sich auf Suspend vor):  
         * Versuche, einen Inhibit-Lock zu nehmen: let fd \= manager\_proxy.inhibit("sleep", "MyDesktopEnvironment", "Preparing for sleep", "delay").await.map\_err(|e| LogindError::InhibitFailed { reason: e.to\_string() })?;  
         * Speichere den OwnedFd (File Descriptor) in sleep\_inhibitor\_lock (unter Mutex-Sperre).  
         * Sende LogindEvent::PrepareForSleep { starting: true } über den Broadcast-Kanal.  
         * (Die Domänen-/UI-Schicht muss auf dieses Event reagieren und ihre Vorbereitungen treffen. Nach Abschluss oder Timeout muss ein Mechanismus existieren, um den Inhibit-Lock freizugeben.)  
       * Wenn start \== false (System wacht auf):  
         * Gib den Inhibit-Lock frei, falls einer gehalten wird: if let Some(fd) \= self.sleep\_inhibitor\_lock.lock().await.take() { drop(fd); } (Das drop auf OwnedFd schließt den FD und gibt den Lock frei).  
         * Sende LogindEvent::PrepareForSleep { starting: false } über den Broadcast-Kanal.  
   * **SessionNew / SessionRemoved-Signale:**  
     * Abonniere manager\_proxy.receive\_session\_new().await? und manager\_proxy.receive\_session\_removed().await?.  
     * Bei Empfang: Aktualisiere die interne Liste der bekannten Sitzungen (falls eine solche geführt wird, ansonsten primär für die ActiveSessionChanged-Logik relevant). Prüfe, ob sich die aktive Sitzung geändert hat. Wenn ja, aktualisiere active\_session\_id, active\_session\_path, active\_session\_proxy und sende LogindEvent::ActiveSessionChanged. Sende auch LogindEvent::SessionListChanged.  
   * **Lock / Unlock-Signale der aktiven Session (optional, falls die DE nicht selbst der Session-Manager ist, der diese direkt verarbeitet):**  
     * Wenn ein active\_session\_proxy vorhanden ist, abonniere dessen receive\_lock\_signal().await? und receive\_unlock\_signal().await? (falls diese Signale vom LogindSessionProxy so generiert werden; alternativ PropertiesChanged für LockedHint überwachen).  
     * Bei Lock-Signal: Sende LogindEvent::ActiveSessionLocked.  
     * Bei Unlock-Signal: Sende LogindEvent::ActiveSessionUnlocked.  
     * Bei PropertiesChanged auf LockedHint der aktiven Session: Entsprechend ActiveSessionLocked/Unlocked senden.  
4. **Öffentliche Methoden auf LogindClient:**  
   * async fn request\_lock\_active\_session(\&self) \-\> Result\<(), LogindError\>:  
     * Rufe die active\_session\_id ab (unter Mutex-Sperre).  
     * Wenn vorhanden, rufe self.manager\_proxy.lock\_session(\&session\_id).await.  
   * async fn request\_unlock\_active\_session(\&self) \-\> Result\<(), LogindError\>:  
     * Analog zu request\_lock\_active\_session mit unlock\_session.  
   * fn subscribe\_events(\&self) \-\> tokio::sync::broadcast::Receiver\<LogindEvent\>: Gibt einen neuen Empfänger für den internen Event-Kanal zurück.  
   * fn release\_sleep\_inhibitor(\&self): Methode, die von anderen Teilen des Systems aufgerufen werden kann, um den Sleep-Inhibitor explizit freizugeben, nachdem die Vorbereitungen für den Suspend abgeschlossen sind.

### **G. Interaktionen**

* **Core Layer:** Stellt die async-Laufzeitumgebung und FD-Handling-Fähigkeiten bereit (für den Inhibit-Lock).  
* **Domain Layer:** Empfängt LogindEvent::PrepareForSleep, um Zustände zu speichern oder laufende Operationen zu pausieren. Reagiert auf ActiveSessionLocked/Unlocked für Policy-Anpassungen (z.B. Deaktivierung bestimmter Hintergrunddienste).  
* **UI Layer:** Empfängt ActiveSessionLocked/Unlocked, um den Sperrbildschirm anzuzeigen/auszublenden oder andere UI-Anpassungen vorzunehmen. Kann request\_lock\_active\_session aufrufen.  
* **Event Bus:** Der LogindClient gibt LogindEvent-Ereignisse (PrepareForSleep, ActiveSessionLocked, ActiveSessionUnlocked, ActiveSessionChanged, SessionListChanged) auf einem internen Event-Bus aus.

### **H. Vertiefende Betrachtungen & Implikationen**

Die korrekte Handhabung von Inhibit-Locks im Kontext des PrepareForSleep-Signals ist für die Systemstabilität von entscheidender Bedeutung. Wenn die Desktop-Umgebung einen solchen Lock nimmt, um sich auf den Suspend-Vorgang vorzubereiten (z.B. durch Speichern von Zuständen, sicheres Beenden von Anwendungen, Dimmen des Bildschirms), muss dieser Lock unbedingt wieder freigegeben werden, sobald diese Vorbereitungen abgeschlossen sind oder ein definierter Timeout erreicht ist. Ein nicht freigegebener Inhibit-Lock kann den Suspend- oder Shutdown-Vorgang des gesamten Systems blockieren.13 Die Implementierung muss daher sicherstellen, dass der durch manager\_proxy.inhibit(...) erhaltene File Deskriptor zuverlässig geschlossen wird, auch im Fehlerfall oder bei einem unerwarteten Beenden der Desktop-Komponente. Dies erfordert eine robuste Fehlerbehandlung und möglicherweise den Einsatz von RAII-Mustern (Resource Acquisition Is Initialization), um sicherzustellen, dass der OwnedFd beim Verlassen des Gültigkeitsbereichs automatisch geschlossen wird.  
Die Unterscheidung zwischen dem *Anfordern* einer Sitzungssperre und dem tatsächlichen *gesperrten Zustand* der Sitzung ist ebenfalls wichtig. logind selbst sperrt den Bildschirm nicht direkt. Die Methode LockSession auf dem Manager-Objekt bewirkt, dass logind ein Lock-Signal an das entsprechende Session-Objekt sendet.13 Der Session-Manager, der typischerweise Teil der Desktop-Umgebung ist (oft in der UI-Schicht angesiedelt), lauscht auf dieses Lock-Signal auf seinem *eigenen* Session-D-Bus-Objekt. Nach Empfang dieses Signals ist der Session-Manager dafür verantwortlich, den Sperrbildschirm zu aktivieren. Sobald der Sperrbildschirm aktiv ist, sollte der Session-Manager logind darüber informieren, indem er die Eigenschaft LockedHint des Session-Objekts auf true setzt. Dieses Modul (system::dbus::logind\_interface) kann primär dafür zuständig sein, Sperr- und Entsperranforderungen über die Manager-Methoden zu initiieren und das PrepareForSleep-Signal zu überwachen. Die eigentliche UI des Sperrbildschirms und das Setzen von LockedHint wären Aufgaben der UI-Schicht, obwohl dieses Modul Änderungen der LockedHint-Eigenschaft der aktiven Sitzung überwachen könnte, um ein vollständiges Bild des Sitzungszustands zu erhalten.  
Die zuverlässige Identifizierung und Verfolgung der "aktiven" Sitzung ist eine weitere Herausforderung. Ein System kann mehrere Benutzersitzungen gleichzeitig haben (z.B. durch Fast User Switching oder Remote-Logins). Die Desktop-Umgebung läuft jedoch typischerweise innerhalb einer einzigen "aktiven" grafischen Sitzung. Viele logind-Operationen sind sitzungsspezifisch und erfordern eine Session-ID. Das logind\_interface-Modul muss daher zuverlässig die Session-ID ermitteln, die zur aktuell laufenden Desktop-Umgebung gehört. Dies kann durch Aufrufen von ListSessions und Überprüfen der Active-Eigenschaft jedes Session-Objekts geschehen.12 Alternativ, wenn die Desktop-Umgebung ihre eigene Session-ID kennt (z.B. aus Umgebungsvariablen, die von pam\_systemd gesetzt wurden), kann sie diese direkt verwenden. Das Modul muss auch Änderungen der aktiven Sitzung behandeln können, falls Funktionen wie Benutzerwechsel unterstützt werden sollen.

## **VI. Schlussfolgerung für Systemschicht (Teil 3/4)**

Die in diesem Teil spezifizierten Module – system::outputs::output\_manager, system::outputs::power\_manager, system::dbus::upower\_interface und system::dbus::logind\_interface – bilden wesentliche Komponenten der Systemschicht. Sie ermöglichen eine detaillierte Steuerung und Überwachung der Display-Hardware sowie die Integration mit grundlegenden Systemdiensten für Energieverwaltung und Sitzungsmanagement.  
Die dargelegten Ultra-Feinspezifikationen folgen dem Prinzip höchster Präzision und Detailgenauigkeit. Sie definieren exakte Schnittstellen, Datenstrukturen, Methoden-Signaturen, Fehlerbehandlungspfade und Interaktionsmuster. Ziel war es, einen direkten Implementierungsleitfaden für Entwickler bereitzustellen, der die Notwendigkeit eigener architektonischer oder logischer Entwurfsentscheidungen minimiert und eine konsistente und robuste Implementierung sicherstellt. Die sorgfältige Beachtung der Atomarität bei Konfigurationsänderungen, die Synchronisation von Zuständen mit externen Diensten und die robuste Fehlerbehandlung sind wiederkehrende Themen, die für die Stabilität der gesamten Desktop-Umgebung von entscheidender Bedeutung sind.  
Der nächste und letzte Teil der Systemschichtspezifikationen (Teil 4/4) wird sich mit weiteren kritischen Aspekten befassen, darunter die XWayland-Integration, die Implementierung von XDG Desktop Portals und die Audio-Management-Schnittstelle, um die Funktionalität der Systemschicht zu vervollständigen.

#### **Referenzen**

1. smithay::wayland::output \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/wayland/output/index.html](https://smithay.github.io/smithay/smithay/wayland/output/index.html)  
2. wlr output management protocol | Wayland Explorer, Zugriff am Mai 14, 2025, [https://wayland.app/protocols/wlr-output-management-unstable-v1](https://wayland.app/protocols/wlr-output-management-unstable-v1)  
3. smithay::backend::drm \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/backend/drm/index.html](https://smithay.github.io/smithay/smithay/backend/drm/index.html)  
4. Zugriff am Januar 1, 1970, [https://smithay.github.io/smithay/smithay/output/struct.Output.html](https://smithay.github.io/smithay/smithay/output/struct.Output.html)  
5. wlr output power management protocol | Wayland Explorer, Zugriff am Mai 14, 2025, [https://wayland.app/protocols/wlr-output-power-management-unstable-v1](https://wayland.app/protocols/wlr-output-power-management-unstable-v1)  
6. org.freedesktop.UPower: UPower Reference Manual, Zugriff am Mai 14, 2025, [https://upower.freedesktop.org/docs/UPower.html](https://upower.freedesktop.org/docs/UPower.html)  
7. D-BUS Protocol | Desktop Notifications Specification, Zugriff am Mai 14, 2025, [https://specifications.freedesktop.org/notification-spec/1.2/protocol.html](https://specifications.freedesktop.org/notification-spec/1.2/protocol.html)  
8. zbus \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/](https://docs.rs/zbus/latest/zbus/)  
9. Writing a client proxy \- zbus: D-Bus for Rust made easy \- GitHub Pages, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/client.html](https://dbus2.github.io/zbus/client.html)  
10. proxy in zbus \- Rust \- openrr.github.io, Zugriff am Mai 14, 2025, [https://openrr.github.io/openrr/zbus/attr.proxy.html](https://openrr.github.io/openrr/zbus/attr.proxy.html)  
11. Zugriff am Januar 1, 1970, [https://upower.freedesktop.org/docs/UPower.Device.html\#UPower.Device.PropertiesChanged](https://upower.freedesktop.org/docs/UPower.Device.html#UPower.Device.PropertiesChanged)  
12. org.freedesktop.login1 \- The D-Bus interface of systemd-logind \- Ubuntu Manpage, Zugriff am Mai 14, 2025, [https://manpages.ubuntu.com/manpages/plucky/man5/org.freedesktop.login1.5.html](https://manpages.ubuntu.com/manpages/plucky/man5/org.freedesktop.login1.5.html)  
13. org.freedesktop.login1, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.html](https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.html)  
14. org.freedesktop.login1, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.html\#Signals](https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.html#Signals)  
15. Zugriff am Januar 1, 1970, [https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.Manager.html\#Signals](https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.Manager.html#Signals)  
16. egui\_backend \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/egui\_backend](https://docs.rs/egui_backend)  
17. Error in zbus \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/enum.Error.html](https://docs.rs/zbus/latest/zbus/enum.Error.html)  
18. "Connection" Search \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/?search=Connection](https://docs.rs/zbus/latest/zbus/?search=Connection)  
19. Zbus create proxy builder without destination \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/78174269/zbus-create-proxy-builder-without-destination](https://stackoverflow.com/questions/78174269/zbus-create-proxy-builder-without-destination)  
20. zbus\_xmlgen \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/zbus\_xmlgen](https://crates.io/crates/zbus_xmlgen)  
21. zbus::fdo \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/fdo/index.html](https://docs.rs/zbus/latest/zbus/fdo/index.html)  
22. Introduction \- zbus: D-Bus for Rust made easy, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/](https://dbus2.github.io/zbus/)  
23. zbus \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/zbus/5.6.0](https://crates.io/crates/zbus/5.6.0)  
24. How to set interface dynamically using zbus \- The Rust Programming Language Forum, Zugriff am Mai 14, 2025, [https://users.rust-lang.org/t/how-to-set-interface-dynamically-using-zbus/108691](https://users.rust-lang.org/t/how-to-set-interface-dynamically-using-zbus/108691)  
25. Zugriff am Januar 1, 1970, [https://docs.rs/zbus/latest/zbus/attr.proxy.html](https://docs.rs/zbus/latest/zbus/attr.proxy.html)  
26. smithay/anvil/src/udev.rs at master \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/Smithay/smithay/blob/master/anvil/src/udev.rs](https://github.com/Smithay/smithay/blob/master/anvil/src/udev.rs)  
27. support wlr-output-management-unstable-v1? · YaLTeR niri · Discussion \#172 \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/YaLTeR/niri/discussions/172](https://github.com/YaLTeR/niri/discussions/172)  
28. File: wlr-output-power-management-unstable-v1.xml \- Debian Sources, Zugriff am Mai 14, 2025, [https://sources.debian.org/src/phosh/0.8.0-1/protocol/wlr-output-power-management-unstable-v1.xml/](https://sources.debian.org/src/phosh/0.8.0-1/protocol/wlr-output-power-management-unstable-v1.xml/)  
29. wayland-rs/historical\_changelog.md at master \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/Smithay/wayland-rs/blob/master/historical\_changelog.md](https://github.com/Smithay/wayland-rs/blob/master/historical_changelog.md)  
30. GUI — list of Rust libraries/crates // Lib.rs, Zugriff am Mai 14, 2025, [https://lib.rs/gui](https://lib.rs/gui)  
31. doc/dbus/org.freedesktop.UPower.ref.xml · debian/0.9.23-1 \- GitLab, Zugriff am Mai 14, 2025, [https://source.puri.sm/Librem5/upower/-/blob/debian/0.9.23-1/doc/dbus/org.freedesktop.UPower.ref.xml?ref\_type=tags](https://source.puri.sm/Librem5/upower/-/blob/debian/0.9.23-1/doc/dbus/org.freedesktop.UPower.ref.xml?ref_type=tags)  
32. D-Bus API Reference: UPower Reference Manual, Zugriff am Mai 14, 2025, [https://upower.freedesktop.org/docs/ref-dbus.html](https://upower.freedesktop.org/docs/ref-dbus.html)  
33. Zugriff am Januar 1, 1970, [https://upower.freedesktop.org/docs/UPower.Device.html\#UPower.Device.DeviceAdded](https://upower.freedesktop.org/docs/UPower.Device.html#UPower.Device.DeviceAdded)  
34. Zugriff am Januar 1, 1970, [https://upower.freedesktop.org/docs/UPower.Device.html\#UPower.Device.DeviceRemoved](https://upower.freedesktop.org/docs/UPower.Device.html#UPower.Device.DeviceRemoved)