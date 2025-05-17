# **Technische Gesamtspezifikation und Entwicklungsrichtlinien: Systemschicht Teil 4/4**

Dieses Dokument ist die Fortsetzung der detaillierten Spezifikation der Systemschicht und behandelt die Module system::audio, system::mcp und system::portals.

## **5\. system::audio \- PipeWire Client-Integration**

Das Modul system::audio ist die maßgebliche Komponente für alle audiobezogenen Operationen innerhalb der Desktop-Umgebung. Es nutzt das PipeWire Multimedia-Framework, um Audiogeräte (Sinks und Quellen), Lautstärke- und Stummschaltungszustände sowohl für Geräte als auch für Anwendungsströme zu verwalten und auf audiobezogene Systemereignisse zu reagieren. Dieses Modul agiert als PipeWire-Client und abstrahiert die Komplexität der PipeWire C-API durch die pipewire-rs Rust-Bindings.  
Die zentrale Designphilosophie dieses Moduls ist die Zentralisierung der gesamten PipeWire-Interaktionslogik, um eine saubere, übergeordnete API für andere Teile der Desktop-Umgebung bereitzustellen. Es basiert auf einer ereignisgesteuerten Architektur, die asynchron auf PipeWire-Ereignisse (Geräteänderungen, Stream-Status, Lautstärkeaktualisierungen) lauscht und diese in interne Systemereignisse übersetzt, die von der UI- und Domänenschicht konsumiert werden können. Eine robuste Fehlerbehandlung wird durch die Verwendung von thiserror für spezifische AudioError-Typen gewährleistet, die klar zwischen PipeWire-spezifischen Problemen und internen Logikfehlern unterscheiden.  
Die Architektur von PipeWire 1 dreht sich um eine MainLoop, einen Context, einen Core und eine Registry. Client-Anwendungen entdecken und interagieren mit entfernten Objekten (Nodes, Devices, Streams) über Proxys, die von der Registry bezogen werden. Die Ereignisbehandlung ist callback-basiert. Die Desktop-Umgebung muss sich dynamisch an Änderungen in der Audiolandschaft anpassen, beispielsweise beim Anschließen eines USB-Headsets oder wenn eine Anwendung die Audiowiedergabe startet oder stoppt. Dies erfordert eine kontinuierliche Überwachung des PipeWire-Status. Das Registry-Objekt sendet global- und global\_remove-Ereignisse für Objekte, die erscheinen oder verschwinden.4 Einzelne Objekte (Proxys für Nodes, Devices) senden Ereignisse für Eigenschaftsänderungen, z.B. param\_changed für Lautstärke/Stummschaltung eines Nodes.15 Die pipewire-rs Bibliothek stellt idiomatische Rust-Wrapper für diese Konzepte bereit.1 Beispiele wie 9 demonstrieren die Initialisierung der Main Loop, des Context, des Core, der Registry und das Hinzufügen von Listenern. Daraus folgt, dass system::audio seine eigene PipeWire MainLoop verwalten muss. Diese Schleife wird wahrscheinlich in einem dedizierten Thread ausgeführt, um ein Blockieren der Hauptereignisschleife der Desktop-Umgebung (z.B. Calloop) zu vermeiden. Asynchrone Kommunikationskanäle (wie tokio::sync::mpsc und tokio::sync::broadcast) werden verwendet, um Befehle und Ereignisse zwischen dem PipeWire-Thread und dem Rest des Systems zu überbrücken. Dies steht im Einklang mit den Multithreading-Richtlinien von pipewire-rs.1  
Die Lautstärkeregelung in PipeWire kann nuanciert sein und entweder Props auf einem Node (oft für Software-/Stream-Lautstärken) oder Route-Parameter auf einem Device (für Hardware-/Master-Lautstärken) betreffen. Benutzer erwarten, sowohl die Master-Ausgabelautstärke als auch die Lautstärke pro Anwendung steuern zu können. Kommandozeilenwerkzeuge wie pw-cli und wpctl demonstrieren das Setzen von channelVolumes über Props auf einem Node 26 oder über Route-Parameter auf einem Device.26 Die Parameter SPA\_PARAM\_Props und SPA\_PARAM\_Route sind zentrale PipeWire-Parameter (SPA \- Simple Plugin API). Die Methode Node::set\_param von pipewire-rs wird verwendet, was die Konstruktion von SpaPod-Objekten für diese Parameter erfordert.15 Das Modul system::audio muss daher zwischen der Steuerung der Master-Lautstärke des Geräts und der Lautstärke des Anwendungsstroms unterscheiden und die entsprechenden PipeWire-Objekte und \-Parameter verwenden. Lautstärkewerte erfordern oft eine kubische Skalierung für eine lineare Benutzerwahrnehmung.  
**Modulstruktur und Dateien:**

* system/audio/mod.rs: Öffentliche API des Audio-Moduls, Definition der AudioError Enum.  
* system/audio/client.rs: Kernstruktur PipeWireClient, verwaltet PipeWire-Verbindung, Hauptschleife, Ereignis-/Befehlskanäle.  
* system/audio/manager.rs: Handhabt die Erkennung, Verfolgung und Eigenschaftsaktualisierungen von AudioDevice- und StreamInfo-Objekten über Registry- und Proxy-Ereignisse.  
* system/audio/control.rs: Implementiert Logik für Lautstärke-/Stummschaltungsbefehle, Konstruktion von SpaPods und Aufruf von set\_param.  
* system/audio/types.rs: Definiert AudioDevice, StreamInfo, AudioEvent, AudioCommand, AudioDeviceType, Volume, etc.  
* system/audio/spa\_pod\_utils.rs: Hilfsfunktionen zur Konstruktion komplexer SpaPod-Objekte für Lautstärke, Stummschaltung und potenziell andere Parameter.  
* system/audio/error.rs: Fehlerbehandlung für das Audio-Modul.

### **5.3.1. Submodul: system::audio::client \- PipeWire Verbindungs- und Ereignisschleifenmanagement**

* **Datei:** system/audio/client.rs  
* **Zweck:** Dieses Submodul ist verantwortlich für die Verwaltung der Low-Level-Verbindung zu PipeWire. Es startet und unterhält die PipeWire-Haupt-Ereignisschleife in einem dedizierten Thread und dient als Brücke für die Weiterleitung von Befehlen an das PipeWire-System und die Verteilung von PipeWire-Ereignissen an andere Teile des Audio-Moduls.

#### **5.3.1.1. Strukuren**

* pub struct PipeWireClient:  
  * core: std::sync::Arc\<pipewire::Core\>: Ein Proxy zum PipeWire-Core, der die Hauptverbindung zum PipeWire-Daemon darstellt. Wird als Arc gehalten, um sicher zwischen Threads geteilt zu werden.  
  * mainloop\_thread\_handle: Option\<std::thread::JoinHandle\<()\>\>: Ein Handle für den dedizierten OS-Thread, in dem die PipeWire-Hauptereignisschleife läuft. Wird beim Beenden des Clients zum sauberen Herunterfahren des Threads verwendet.  
  * command\_sender: tokio::sync::mpsc::Sender\<AudioCommand\>: Ein asynchroner Sender zum Übermitteln von AudioCommands von anderen Teilen des Systems (z.B. UI-Interaktionen) an den PipeWire-Loop-Thread.  
  * internal\_event\_sender: tokio::sync::mpsc::Sender\<InternalAudioEvent\>: Ein interner Sender, der von Worker-Tasks innerhalb dieses Moduls (z.B. Registry-Listener) verwendet wird, um rohe PipeWire-Ereignisse an den Hauptverarbeitungslogik-Task im PipeWire-Thread zu senden.  
  * Initialwerte: core und registry werden während der Initialisierung gesetzt. mainloop\_thread\_handle ist anfangs None und wird nach dem Starten des Threads gesetzt. Die Sender werden beim Erstellen der Kanäle initialisiert.  
  * Invarianten: core und registry müssen immer gültig sein, solange der mainloop\_thread\_handle Some ist.  
* struct PipeWireLoopData: Diese Struktur kapselt alle Daten, die innerhalb des dedizierten PipeWire-Loop-Threads benötigt werden.  
  * core: std::sync::Arc\<pipewire::Core\>: Geteilter Zugriff auf den PipeWire Core.  
  * registry: std::sync::Arc\<pipewire::Registry\>: Geteilter Zugriff auf die PipeWire Registry.  
  * audio\_event\_broadcaster: tokio::sync::broadcast::Sender\<AudioEvent\>: Ein Sender zum Verteilen von aufbereiteten AudioEvents an alle interessierten Listener im System (z.B. UI-Komponenten).  
  * command\_receiver: tokio::sync::mpsc::Receiver\<AudioCommand\>: Empfängt Befehle, die an das Audio-System gesendet werden.  
  * internal\_event\_receiver: tokio::sync::mpsc::Receiver\<InternalAudioEvent\>: Empfängt interne Ereignisse von PipeWire-Callbacks.  
  * active\_devices: std::collections::HashMap\<u32, MonitoredDevice\>: Eine Map zur Verfolgung der aktuell aktiven Audiogeräte (Nodes oder Devices), ihrer Proxys, Eigenschaften und Listener-Hooks. Der Key ist die PipeWire Global ID.  
  * active\_streams: std::collections::HashMap\<u32, MonitoredStream\>: Eine Map zur Verfolgung aktiver Audio-Streams (Nodes mit Anwendungsbezug). Der Key ist die PipeWire Global ID.  
  * default\_sink\_id: Option\<u32\>: Die ID des aktuellen Standard-Audioausgabegeräts.  
  * default\_source\_id: Option\<u32\>: Die ID des aktuellen Standard-Audioeingabegeräts.  
  * pipewire\_mainloop: pipewire::MainLoop: Die PipeWire-Hauptereignisschleife.  
  * pipewire\_context: pipewire::Context: Der PipeWire-Kontext.  
  * metadata\_proxy: Option\<std::sync::Arc\<pipewire::metadata::Metadata\>\>: Proxy zum PipeWire Metadaten-Objekt, um Standardgeräte zu setzen/lesen.  
  * metadata\_listener\_hook: Option\<pipewire::spa::SpaHook\>: Listener für Änderungen am Metadaten-Objekt.  
* struct MonitoredDevice: Repräsentiert ein überwachtes Audiogerät.  
  * proxy: std::sync::Arc\<dyn pipewire::proxy::ProxyT \+ Send \+ Sync\>: Ein generischer Proxy, der entweder ein pw::node::Node oder pw::device::Device sein kann, abhängig davon, wie die Lautstärke/Stummschaltung gesteuert wird (Props vs. Route).  
  * proxy\_id: u32: Die ID des Proxy-Objekts.  
  * global\_id: u32: Die globale ID des PipeWire-Objekts.  
  * properties: pipewire::spa::SpaDict: Die zuletzt bekannten Eigenschaften des Geräts.  
  * param\_listener\_hook: Option\<pipewire::spa::SpaHook\>: Hook für den param\_changed Listener des Node/Device-Proxys.  
  * info: AudioDevice: Die zwischengespeicherte, aufbereitete AudioDevice-Struktur für die externe API.  
* struct MonitoredStream: Repräsentiert einen überwachten Audio-Stream.  
  * proxy: std::sync::Arc\<pipewire::node::Node\>: Proxy zum Stream-Node.  
  * proxy\_id: u32: Die ID des Proxy-Objekts.  
  * global\_id: u32: Die globale ID des PipeWire-Objekts.  
  * properties: pipewire::spa::SpaDict: Die zuletzt bekannten Eigenschaften des Streams.  
  * param\_listener\_hook: Option\<pipewire::spa::SpaHook\>: Hook für den param\_changed Listener des Node-Proxys.  
  * info: StreamInfo: Die zwischengespeicherte, aufbereitete StreamInfo-Struktur.  
* enum InternalAudioEvent: Interne Ereignisse zur Kommunikation innerhalb des Audio-Moduls.  
  * PwGlobalAdded(pipewire::registry::GlobalObject\<pipewire::spa::SpaDict\>)  
  * PwGlobalRemoved(u32)  
  * PwNodeParamChanged { node\_id: u32, param\_id: u32, pod: Option\<pipewire::spa::Pod\> }  
  * PwDeviceParamChanged { device\_id: u32, param\_id: u32, pod: Option\<pipewire::spa::Pod\> }  
  * PwMetadataPropsChanged { metadata\_id: u32, props: pipewire::spa::SpaDict }

#### **5.3.1.2. Methoden für PipeWireClient**

* pub async fn new(audio\_event\_broadcaster: tokio::sync::broadcast::Sender\<AudioEvent\>) \-\> Result\<Self, AudioError\>:  
  * **Vorbedingungen:** Keine.  
  * **Schritte:**  
    1. pipewire::init() aufrufen, um die PipeWire-Bibliothek zu initialisieren.4 Falls dies fehlschlägt, AudioError::PipeWireInitFailed zurückgeben.  
    2. Zwei tokio::sync::mpsc::channel erstellen:  
       * command\_channel für AudioCommand (Kapazität z.B. 32).  
       * internal\_event\_channel für InternalAudioEvent (Kapazität z.B. 64).  
    3. Die Sender (command\_sender, internal\_event\_sender) und Empfänger (command\_receiver, internal\_event\_receiver) aus den Kanälen extrahieren.  
    4. Einen tokio::sync::oneshot::channel erstellen (init\_signal\_tx, init\_signal\_rx) zur Signalisierung der erfolgreichen Initialisierung des PipeWire-Threads.  
    5. Einen neuen OS-Thread mit std::thread::spawn starten. Dieser Thread führt die Funktion run\_pipewire\_loop aus. Der audio\_event\_broadcaster, command\_receiver, internal\_event\_receiver, internal\_event\_sender\_clone (für Callbacks) und init\_signal\_tx werden in den Thread verschoben.  
       * **Thread-Logik (run\_pipewire\_loop Funktion):**  
         1. let mainloop \= MainLoop::new(None).map\_err(AudioError::MainLoopCreationFailed)?;.4  
         2. let context \= Context::new(\&mainloop).map\_err(AudioError::ContextCreationFailed)?;.4  
         3. let core \= Arc::new(context.connect(None).map\_err(AudioError::CoreConnectionFailed)?);.4  
         4. let registry \= Arc::new(core.get\_registry().map\_err(AudioError::RegistryCreationFailed)?);.4  
         5. Die erfolgreiche Initialisierung von core und registry über init\_signal\_tx.send(Ok((core.clone(), registry.clone()))) signalisieren.  
         6. Eine PipeWireLoopData-Instanz erstellen, die core, registry, den übergebenen audio\_event\_broadcaster, command\_receiver und internal\_event\_receiver enthält.  
         7. Einen Listener auf der registry mit add\_listener\_local() registrieren.4  
            * Im global-Callback: Ein InternalAudioEvent::PwGlobalAdded(global\_object) an den internal\_event\_sender\_clone senden. global\_object ist hier das Argument des Callbacks.  
            * Im global\_remove-Callback: Ein InternalAudioEvent::PwGlobalRemoved(id) an den internal\_event\_sender\_clone senden. id ist das Argument des Callbacks.  
         8. Eine Timer-Quelle zur mainloop hinzufügen (mainloop.loop\_().add\_timer(...)), die periodisch (z.B. alle 10ms) eine Funktion aufruft. Diese Funktion (process\_external\_messages) versucht, Nachrichten von command\_receiver und internal\_event\_receiver mit try\_recv() zu empfangen und verarbeitet diese.  
            * Die Integration von Tokio MPSC-Kanälen mit der blockierenden mainloop.run() erfordert einen Mechanismus, um die Schleife periodisch zu unterbrechen oder die MPSC-Empfänger nicht-blockierend abzufragen. Ein Timer ist ein gängiger Ansatz hierfür.1  
         9. mainloop.run() aufrufen. Diese Funktion blockiert den Thread und verarbeitet PipeWire-Ereignisse und Timer-Callbacks.  
    6. Auf das Ergebnis von init\_signal\_rx.await warten. Bei Erfolg die core und registry Arcs aus dem Ergebnis entnehmen. Bei Fehler AudioError::PipeWireThreadPanicked oder den empfangenen Fehler zurückgeben.  
    7. Den mainloop\_thread\_handle, die erhaltenen core und registry Arcs und den command\_sender in der PipeWireClient-Instanz speichern.  
    8. Ok(Self) zurückgeben.  
  * **Nachbedingungen:** Ein PipeWireClient ist initialisiert und der PipeWire-Loop-Thread läuft.  
  * **Fehlerfälle:** AudioError::PipeWireInitFailed, AudioError::MainLoopCreationFailed, AudioError::ContextCreationFailed, AudioError::CoreConnectionFailed, AudioError::RegistryCreationFailed, AudioError::PipeWireThreadPanicked.  
* pub fn get\_command\_sender(\&self) \-\> tokio::sync::mpsc::Sender\<AudioCommand\>:  
  * **Vorbedingungen:** Der PipeWireClient wurde erfolgreich initialisiert.  
  * **Schritte:** Gibt ein Klon des command\_sender zurück.  
  * **Nachbedingungen:** Keine Zustandsänderung.  
  * **Fehlerfälle:** Keine.

#### **5.3.1.3. Private statische Funktion run\_pipewire\_loop**

* fn run\_pipewire\_loop(audio\_event\_broadcaster: tokio::sync::broadcast::Sender\<AudioEvent\>, mut command\_receiver: tokio::sync::mpsc::Receiver\<AudioCommand\>, mut internal\_event\_receiver: tokio::sync::mpsc::Receiver\<InternalAudioEvent\>, internal\_event\_sender\_clone: tokio::sync::mpsc::Sender\<InternalAudioEvent\>, init\_signal\_tx: tokio::sync::oneshot::Sender\<Result\<(std::sync::Arc\<pipewire::Core\>, std::sync::Arc\<pipewire::Registry\>), AudioError\>\>):  
  * **Logik:** Wie oben unter PipeWireClient::new beschrieben (Schritt 5.1 bis 5.9).  
  * Die Funktion process\_external\_messages(loop\_data: \&mut PipeWireLoopData) wird vom Timer aufgerufen:  
    * **Befehlsverarbeitung (von loop\_data.command\_receiver.try\_recv()):**  
      * AudioCommand::SetDeviceVolume { device\_id, volume, curve }: Ruft system::audio::control::set\_device\_volume(\&loop\_data, device\_id, volume, curve) auf.  
      * AudioCommand::SetDeviceMute { device\_id, mute }: Ruft system::audio::control::set\_device\_mute(\&loop\_data, device\_id, mute) auf.  
      * AudioCommand::SetStreamVolume { stream\_id, volume, curve }: Ruft system::audio::control::set\_node\_volume(\&loop\_data, stream\_id, volume, curve) auf (da Streams als Nodes repräsentiert werden).  
      * AudioCommand::SetStreamMute { stream\_id, mute }: Ruft system::audio::control::set\_node\_mute(\&loop\_data, stream\_id, mute) auf.  
      * AudioCommand::SetDefaultDevice { device\_type, device\_id }: Ruft system::audio::control::set\_default\_device(\&loop\_data, device\_type, device\_id) auf.  
      * AudioCommand::RequestDeviceList: Sendet den aktuellen Stand von loop\_data.active\_devices über den audio\_event\_broadcaster als AudioEvent::DeviceListUpdated.  
      * AudioCommand::RequestStreamList: Sendet den aktuellen Stand von loop\_data.active\_streams über den audio\_event\_broadcaster als AudioEvent::StreamListUpdated.  
    * **Interne Ereignisverarbeitung (von loop\_data.internal\_event\_receiver.try\_recv()):**  
      * InternalAudioEvent::PwGlobalAdded(global): Ruft system::audio::manager::handle\_pipewire\_global\_added(\&mut loop\_data, global, \&internal\_event\_sender\_clone).  
      * InternalAudioEvent::PwGlobalRemoved(id): Ruft system::audio::manager::handle\_pipewire\_global\_removed(\&mut loop\_data, id).  
      * InternalAudioEvent::PwNodeParamChanged { node\_id, param\_id, pod }: Ruft system::audio::manager::handle\_node\_param\_changed(\&mut loop\_data, node\_id, param\_id, pod).  
      * InternalAudioEvent::PwDeviceParamChanged { device\_id, param\_id, pod }: Ruft system::audio::manager::handle\_device\_param\_changed(\&mut loop\_data, device\_id, param\_id, pod).  
      * InternalAudioEvent::PwMetadataPropsChanged { metadata\_id, props}: Ruft system::audio::manager::handle\_metadata\_props\_changed(\&mut loop\_data, metadata\_id, props).

### **5.3.2. Submodul: system::audio::manager \- Geräte- und Stream-Zustandsmanagement**

* **Datei:** system/audio/manager.rs  
* **Zweck:** Dieses Submodul enthält die Logik zur Verarbeitung von PipeWire-Registry-Ereignissen, zur Verwaltung der AudioDevice- und StreamInfo-Strukturen und zur Handhabung von Eigenschafts-/Parameteränderungen dieser Objekte. Es interagiert eng mit dem PipeWireClient, um auf Low-Level-Ereignisse zu reagieren und den Zustand der Audio-Entitäten im System zu aktualisieren.

#### **5.3.2.1. Funktionen (aufgerufen von PipeWireClient's Loop)**

* pub(super) fn handle\_pipewire\_global\_added(loop\_data: \&mut PipeWireLoopData, global: pipewire::registry::GlobalObject\<pipewire::spa::SpaDict\>, internal\_event\_sender: \&tokio::sync::mpsc::Sender\<InternalAudioEvent\>) \-\> Result\<(), AudioError\>:  
  * **Vorbedingungen:** loop\_data ist initialisiert. global ist ein neu entdecktes PipeWire-Global-Objekt.  
  * **Schritte:**  
    1. Loggt das neue globale Objekt: tracing::info\!("PipeWire Global Added: id={}, type={:?}, version={}, props={:?}", global.id, global.type\_, global.version, global.props.as\_ref().map\_or\_else(|| "None", |p| format\!("{:?}", p)));  
    2. Abhängig von global.type\_:  
       * ObjectType::Node:  
         1. Eigenschaften aus global.props extrahieren (falls vorhanden): media.class, node.name, node.description, application.process.id, application.name, audio.format, audio.channels, object.serial.  
         2. Bestimmen, ob es sich um ein Gerät (Sink/Source) oder einen Anwendungsstream handelt:  
            * **Gerät (Sink/Source Node):** Typischerweise media.class ist "Audio/Sink" oder "Audio/Source" und application.name ist nicht gesetzt oder verweist auf einen Systemdienst.  
              * Proxy binden: let node\_proxy \= Arc::new(loop\_data.registry.bind::\<pipewire::node::Node\>(\&global.into\_proxy\_properties(None)?)?);.8 Die into\_proxy\_properties Methode wird hier verwendet, um die bereits vorhandenen Properties direkt zu nutzen.  
              * Die global.id ist die ID des globalen Objekts, node\_proxy.id() ist die ID des gebundenen Proxys.  
              * Anfängliche Parameter abrufen (insbesondere SPA\_PARAM\_Props für Lautstärke/Mute): node\_proxy.enum\_params\_sync(pipewire::spa::param::ParamType::Props.as\_raw(), None, None, None) aufrufen. Den ersten zurückgegebenen SpaPod parsen, um channelVolumes und mute zu extrahieren (siehe spa\_pod\_utils).  
              * Eine AudioDevice-Instanz erstellen und mit den extrahierten und abgerufenen Informationen füllen. is\_default wird gesetzt, wenn global.id mit loop\_data.default\_sink\_id oder loop\_data.default\_source\_id übereinstimmt.  
              * Einen param\_changed-Listener auf dem node\_proxy registrieren:  
                Rust  
                let internal\_sender\_clone \= internal\_event\_sender.clone();  
                let proxy\_id \= node\_proxy.id(); // ID des gebundenen Proxys  
                let listener\_hook \= node\_proxy.add\_listener\_local()  
                   .param(move |\_id, \_seq, param\_id, \_index, \_next, pod| {  
                        let \_ \= internal\_sender\_clone.try\_send(InternalAudioEvent::PwNodeParamChanged {  
                            node\_id: proxy\_id, // Wichtig: ID des Proxys, nicht die globale ID  
                            param\_id,  
                            pod: pod.cloned(), // Klonen, da Pod nur als Referenz übergeben wird  
                        });  
                    })  
                   .register();

              * MonitoredDevice erstellen und in loop\_data.active\_devices mit global.id als Schlüssel einfügen. Die listener\_hook muss in MonitoredDevice gespeichert werden, um sie später entfernen zu können.  
              * AudioEvent::DeviceAdded(new\_device\_info) über loop\_data.audio\_event\_broadcaster senden.  
            * **Anwendungsstream:** Typischerweise ist application.name gesetzt.  
              * Proxy binden: let node\_proxy \= Arc::new(loop\_data.registry.bind::\<pipewire::node::Node\>(\&global.into\_proxy\_properties(None)?)?);  
              * Anfängliche Parameter (Lautstärke/Mute) wie bei Geräten abrufen.  
              * StreamInfo-Instanz erstellen.  
              * param\_changed-Listener auf node\_proxy registrieren (analog zu Geräten, sendet InternalAudioEvent::PwNodeParamChanged).  
              * MonitoredStream erstellen und in loop\_data.active\_streams mit global.id als Schlüssel einfügen.  
              * AudioEvent::StreamAdded(new\_stream\_info) über loop\_data.audio\_event\_broadcaster senden.  
       * ObjectType::Device:  
         1. Eigenschaften extrahieren: device.api, device.nick, device.description, media.class.  
         2. Wenn media.class "Audio/Sink" oder "Audio/Source" ist und dies ein "echtes" Hardware-Gerät darstellt (oft über device.api wie "alsa" identifizierbar), könnte dies für Master-Lautstärkeregelung über SPA\_PARAM\_Route relevant sein.  
            * Proxy binden: let device\_proxy \= Arc::new(loop\_data.registry.bind::\<pipewire::device::Device\>(\&global.into\_proxy\_properties(None)?)?);  
            * Anfängliche SPA\_PARAM\_Route-Parameter abrufen: device\_proxy.enum\_params\_sync(pipewire::spa::param::ParamType::Route.as\_raw(), None, None, None). Parsen, um aktive Route und deren Lautstärke/Mute zu finden.  
            * Eine AudioDevice-Instanz erstellen.  
            * Einen param\_changed-Listener auf dem device\_proxy registrieren, der InternalAudioEvent::PwDeviceParamChanged sendet.  
            * MonitoredDevice erstellen und in loop\_data.active\_devices einfügen.  
            * AudioEvent::DeviceAdded senden.  
       * ObjectType::Metadata:  
         1. Eigenschaften extrahieren: metadata.name.  
         2. Wenn metadata.name \== "default" ist:  
            * Proxy binden: let metadata\_proxy \= Arc::new(loop\_data.registry.bind::\<pipewire::metadata::Metadata\>(\&global.into\_proxy\_properties(None)?)?);  
            * loop\_data.metadata\_proxy \= Some(metadata\_proxy.clone());  
            * Die props-Eigenschaft des Metadatenobjekts enthält die Standardgeräte-IDs (z.B. default.audio.sink, default.audio.source).10 Diese parsen und loop\_data.default\_sink\_id/default\_source\_id aktualisieren.  
            * Einen props-Listener auf dem metadata\_proxy registrieren:  
              Rust  
              let internal\_sender\_clone \= internal\_event\_sender.clone();  
              let proxy\_id \= metadata\_proxy.id();  
              let listener\_hook \= metadata\_proxy.add\_listener\_local()  
                 .props(move |\_id, props| {  
                      let \_ \= internal\_sender\_clone.try\_send(InternalAudioEvent::PwMetadataPropsChanged {  
                          metadata\_id: proxy\_id,  
                          props: props.cloned(),  
                      });  
                  })  
                 .register();  
              loop\_data.metadata\_listener\_hook \= Some(listener\_hook);

            * AudioEvent::DefaultSinkChanged / DefaultSourceChanged senden, falls sich die IDs geändert haben.  
  * **Nachbedingungen:** Relevante Proxys sind gebunden, Listener registriert, und der Zustand in loop\_data ist aktualisiert. Entsprechende AudioEvents wurden gesendet.  
  * **Fehlerfälle:** AudioError::ProxyBindFailed, AudioError::ParameterEnumerationFailed.  
* pub(super) fn handle\_pipewire\_global\_removed(loop\_data: \&mut PipeWireLoopData, id: u32) \-\> Result\<(), AudioError\>:  
  * **Vorbedingungen:** id ist die globale ID eines entfernten PipeWire-Objekts.  
  * **Schritte:**  
    1. Loggt die Entfernung: tracing::info\!("PipeWire Global Removed: id={}", id);  
    2. Wenn id in loop\_data.active\_devices vorhanden ist:  
       * MonitoredDevice entfernen. Der param\_listener\_hook wird automatisch durch das Droppen des SpaHook-Objekts (oder durch explizites remove() auf dem Listener) entfernt, wenn der Proxy gedroppt wird. Der Proxy selbst wird gedroppt, wenn der Arc keine Referenzen mehr hat.  
       * AudioEvent::DeviceRemoved(id) über loop\_data.audio\_event\_broadcaster senden.  
    3. Wenn id in loop\_data.active\_streams vorhanden ist:  
       * MonitoredStream entfernen. Listener-Hook wird ebenfalls entfernt.  
       * AudioEvent::StreamRemoved(id) über loop\_data.audio\_event\_broadcaster senden.  
    4. Wenn die ID des loop\_data.metadata\_proxy (falls vorhanden) mit id übereinstimmt:  
       * loop\_data.metadata\_proxy \= None;  
       * loop\_data.metadata\_listener\_hook \= None; (wird gedroppt)  
  * **Nachbedingungen:** Das Objekt ist aus dem internen Zustand entfernt, Listener sind deregistriert. AudioEvent wurde gesendet.  
  * **Fehlerfälle:** Keine spezifischen Fehler erwartet, außer Logging-Fehler.  
* pub(super) fn handle\_node\_param\_changed(loop\_data: \&mut PipeWireLoopData, node\_id: u32, param\_id: u32, pod: Option\<pipewire::spa::Pod\>) \-\> Result\<(), AudioError\>:  
  * **Vorbedingungen:** node\_id ist die Proxy-ID eines Nodes. param\_id gibt den Typ des geänderten Parameters an. pod enthält die neuen Parameterdaten.  
  * **Schritte:**  
    1. Loggt die Parameteränderung: tracing::debug\!("Node Param Changed: node\_id={}, param\_id={}, pod\_is\_some={}", node\_id, param\_id, pod.is\_some());  
    2. Suchen des MonitoredDevice oder MonitoredStream in loop\_data.active\_devices oder loop\_data.active\_streams, dessen proxy.id() mit node\_id übereinstimmt.  
    3. Wenn gefunden und param\_id \== pipewire::spa::param::ParamType::Props.as\_raw():  
       * Wenn pod Some ist, die neuen Lautstärke- (channelVolumes) und Mute- (mute) Werte aus dem SpaPod parsen (siehe spa\_pod\_utils).  
       * Die info (entweder AudioDevice oder StreamInfo) im MonitoredDevice/MonitoredStream aktualisieren.  
       * Das entsprechende AudioEvent (DeviceVolumeChanged, DeviceMuteChanged, StreamVolumeChanged, StreamMuteChanged) über loop\_data.audio\_event\_broadcaster senden.  
  * **Nachbedingungen:** Der interne Zustand des Geräts/Streams ist aktualisiert und ein AudioEvent wurde gesendet.  
  * **Fehlerfälle:** AudioError::SpaPodParseFailed.  
* pub(super) fn handle\_device\_param\_changed(loop\_data: \&mut PipeWireLoopData, device\_id: u32, param\_id: u32, pod: Option\<pipewire::spa::Pod\>) \-\> Result\<(), AudioError\>:  
  * **Vorbedingungen:** device\_id ist die Proxy-ID eines Devices.  
  * **Schritte:**  
    1. Loggt die Parameteränderung.  
    2. Suchen des MonitoredDevice in loop\_data.active\_devices, dessen proxy.id() mit device\_id übereinstimmt.  
    3. Wenn gefunden und param\_id \== pipewire::spa::param::ParamType::Route.as\_raw():  
       * Wenn pod Some ist, die neuen Routenparameter parsen, um Lautstärke/Mute der aktiven Route zu extrahieren.  
       * Die info (AudioDevice) im MonitoredDevice aktualisieren.  
       * AudioEvent::DeviceVolumeChanged / DeviceMuteChanged senden.  
  * **Nachbedingungen:** Der interne Zustand des Geräts ist aktualisiert.  
  * **Fehlerfälle:** AudioError::SpaPodParseFailed.  
* pub(super) fn handle\_metadata\_props\_changed(loop\_data: \&mut PipeWireLoopData, metadata\_id: u32, props: pipewire::spa::SpaDict) \-\> Result\<(), AudioError\>:  
  * **Vorbedingungen:** metadata\_id ist die Proxy-ID des Metadaten-Objekts. props sind die geänderten Eigenschaften.  
  * **Schritte:**  
    1. Loggt die Änderung.  
    2. Überprüfen, ob loop\_data.metadata\_proxy existiert und seine ID mit metadata\_id übereinstimmt.  
    3. Die neuen Standard-Sink/Source-IDs aus props extrahieren (z.B. props.get("default.audio.sink").and\_then(|s| s.parse().ok())).  
    4. Wenn sich loop\_data.default\_sink\_id geändert hat:  
       * Altes Standardgerät (falls vorhanden) in active\_devices suchen und is\_default \= false setzen. AudioEvent::DeviceUpdated senden.  
       * Neues Standardgerät in active\_devices suchen und is\_default \= true setzen. AudioEvent::DeviceUpdated senden.  
       * loop\_data.default\_sink\_id aktualisieren.  
       * AudioEvent::DefaultSinkChanged(new\_id) senden.  
    5. Analog für default\_source\_id.  
  * **Nachbedingungen:** Standardgeräte-IDs und is\_default-Flags sind aktualisiert. AudioEvents wurden gesendet.  
  * **Fehlerfälle:** Keine spezifischen Fehler erwartet.

### **5.3.3. Submodul: system::audio::control \- Lautstärke-, Stummschaltungs- und Gerätesteuerung**

* **Datei:** system/audio/control.rs  
* **Zweck:** Implementiert die Logik zum Senden von Steuerbefehlen an PipeWire-Objekte, insbesondere zum Setzen von Lautstärke und Stummschaltung sowie zur Auswahl von Standardgeräten.

#### **5.3.3.1. Funktionen (aufgerufen von PipeWireClient's Loop bei AudioCommand Verarbeitung)**

* pub(super) fn set\_node\_volume(loop\_data: \&PipeWireLoopData, node\_id: u32, volume: Volume, curve: VolumeCurve) \-\> Result\<(), AudioError\>:  
  * **Vorbedingungen:** node\_id ist eine gültige Proxy-ID eines MonitoredDevice (als Node) oder MonitoredStream in loop\_data.  
  * **Schritte:**  
    1. Sucht den MonitoredDevice oder MonitoredStream anhand der node\_id (Proxy-ID).  
    2. Wenn nicht gefunden, AudioError::DeviceOrStreamNotFound(node\_id) zurückgeben.  
    3. Den pipewire::node::Node-Proxy extrahieren.  
    4. Die volume.channel\_volumes (Array von f32) entsprechend der VolumeCurve (z.B. Linear, Cubic) anpassen. Für Cubic wäre das vadj​=v3.  
    5. Einen SpaPod für SPA\_PARAM\_Props erstellen, der channelVolumes enthält (siehe spa\_pod\_utils::build\_volume\_props\_pod).  
    6. node\_proxy.set\_param(pipewire::spa::param::ParamType::Props.as\_raw(), 0, \&pod) aufrufen.27  
    7. Bei Fehler AudioError::PipeWireCommandFailed zurückgeben.  
  * **Nachbedingungen:** Der Lautstärkebefehl wurde an den PipeWire-Node gesendet.  
  * **Fehlerfälle:** AudioError::DeviceOrStreamNotFound, AudioError::SpaPodBuildFailed, AudioError::PipeWireCommandFailed.  
* pub(super) fn set\_node\_mute(loop\_data: \&PipeWireLoopData, node\_id: u32, mute: bool) \-\> Result\<(), AudioError\>:  
  * **Vorbedingungen:** node\_id ist eine gültige Proxy-ID.  
  * **Schritte:**  
    1. Sucht den MonitoredDevice oder MonitoredStream.  
    2. Den pipewire::node::Node-Proxy extrahieren.  
    3. Einen SpaPod für SPA\_PARAM\_Props erstellen, der mute enthält (siehe spa\_pod\_utils::build\_mute\_props\_pod).  
    4. node\_proxy.set\_param(pipewire::spa::param::ParamType::Props.as\_raw(), 0, \&pod) aufrufen.  
  * **Nachbedingungen:** Der Stummschaltungsbefehl wurde gesendet.  
  * **Fehlerfälle:** AudioError::DeviceOrStreamNotFound, AudioError::SpaPodBuildFailed, AudioError::PipeWireCommandFailed.  
* pub(super) fn set\_device\_volume(loop\_data: \&PipeWireLoopData, device\_id: u32, volume: Volume, curve: VolumeCurve) \-\> Result\<(), AudioError\>:  
  * **Vorbedingungen:** device\_id ist eine gültige Proxy-ID eines MonitoredDevice, dessen Proxy ein pipewire::device::Device ist.  
  * **Schritte:**  
    1. Sucht den MonitoredDevice anhand der device\_id.  
    2. Den pipewire::device::Device-Proxy extrahieren.  
    3. Die volume.channel\_volumes entsprechend der VolumeCurve anpassen.  
    4. Die aktuelle aktive Route für das Gerät ermitteln (ggf. durch enum\_params\_sync für SPA\_PARAM\_Route und Auswahl der Route mit dem höchsten priority oder dem passenden index).  
    5. Einen SpaPod für SPA\_PARAM\_Route erstellen, der die index, device (oft 0 für die Route selbst) und die neuen props (mit channelVolumes) enthält (siehe spa\_pod\_utils::build\_route\_volume\_pod). 26  
    6. device\_proxy.set\_param(pipewire::spa::param::ParamType::Route.as\_raw(), 0, \&pod) aufrufen.  
  * **Nachbedingungen:** Der Lautstärkebefehl für die Geräteroute wurde gesendet.  
  * **Fehlerfälle:** AudioError::DeviceOrStreamNotFound, AudioError::SpaPodBuildFailed, AudioError::PipeWireCommandFailed, AudioError::NoActiveRouteFound.  
* pub(super) fn set\_device\_mute(loop\_data: \&PipeWireLoopData, device\_id: u32, mute: bool) \-\> Result\<(), AudioError\>:  
  * **Vorbedingungen:** device\_id ist eine gültige Proxy-ID eines MonitoredDevice (Device-Proxy).  
  * **Schritte:**  
    1. Sucht den MonitoredDevice.  
    2. Den pipewire::device::Device-Proxy extrahieren.  
    3. Aktive Route ermitteln.  
    4. Einen SpaPod für SPA\_PARAM\_Route erstellen, der die props (mit mute) enthält (siehe spa\_pod\_utils::build\_route\_mute\_pod).  
    5. device\_proxy.set\_param(pipewire::spa::param::ParamType::Route.as\_raw(), 0, \&pod) aufrufen.  
  * **Nachbedingungen:** Der Stummschaltungsbefehl für die Geräteroute wurde gesendet.  
  * **Fehlerfälle:** AudioError::DeviceOrStreamNotFound, AudioError::SpaPodBuildFailed, AudioError::PipeWireCommandFailed, AudioError::NoActiveRouteFound.  
* pub(super) fn set\_default\_device(loop\_data: \&mut PipeWireLoopData, device\_type: AudioDeviceType, global\_id: u32) \-\> Result\<(), AudioError\>:  
  * **Vorbedingungen:** loop\_data.metadata\_proxy ist Some. global\_id ist die globale ID des Geräts, das zum Standard werden soll.  
  * **Schritte:**  
    1. Wenn loop\_data.metadata\_proxy None ist, AudioError::MetadataProxyNotAvailable zurückgeben.  
    2. Den pipewire::metadata::Metadata-Proxy extrahieren.  
    3. Den Eigenschaftsnamen basierend auf device\_type bestimmen:  
       * AudioDeviceType::Sink \=\> "default.audio.sink"  
       * AudioDeviceType::Source \=\> "default.audio.source"  
    4. Den Wert als String der global\_id vorbereiten.  
    5. metadata\_proxy.set\_property(property\_name, "Spa:String:JSON", \&global\_id\_string) aufrufen. Die Typangabe "Spa:String:JSON" könnte auch einfach "string" sein, je nachdem was PipeWire erwartet.  
       * **Anmerkung:** Die genaue Methode zum Setzen von Metadaten-Eigenschaften muss anhand der pipewire-rs API für Metadata überprüft werden. Es könnte sein, dass ein SpaDict mit den zu setzenden Properties übergeben werden muss.  
    6. Bei Erfolg wird der PwMetadataPropsChanged-Event ausgelöst und von handle\_metadata\_props\_changed verarbeitet, was den internen Zustand und die is\_default-Flags aktualisiert.  
  * **Nachbedingungen:** Der Befehl zum Ändern des Standardgeräts wurde an PipeWire gesendet.  
  * **Fehlerfälle:** AudioError::MetadataProxyNotAvailable, AudioError::PipeWireCommandFailed.

### **5.3.4. Submodul: system::audio::types \- Kerndatenstrukturen für Audio**

* **Datei:** system/audio/types.rs  
* **Zweck:** Definiert die primären Datenstrukturen, die vom Audio-Modul verwendet und nach außen exponiert werden.

#### **5.3.4.1. Enums**

* \#  
  pub enum AudioError {... } (Definition im error.rs Modul, hier nur als Referenz)  
* \#  
  pub enum AudioDeviceType { Sink, Source, Unknown }  
  * **Zweck:** Repräsentiert den Typ eines Audiogeräts.  
  * **Ableitung:** Aus media.class Property von PipeWire-Objekten (z.B. "Audio/Sink", "Audio/Source").  
* \#  
  pub enum VolumeCurve { Linear, Cubic }  
  * **Zweck:** Definiert die Kurve, die bei der Lautstärkeanpassung verwendet wird. Cubic wird oft für eine natürlichere Wahrnehmung der Lautstärkeänderung verwendet.  
  * **Initialwert:** Typischerweise Cubic für UI-Interaktionen.  
* \#  
  pub enum AudioCommand {... } (siehe Tabelle 5.2)  
* \#  
  pub enum AudioEvent {... } (siehe Tabelle 5.3)

#### **5.3.4.2. Strukuren**

* \#  
  pub struct Volume { pub channel\_volumes: Vec\<f32\> }  
  * **Zweck:** Repräsentiert die Lautstärke für jeden Kanal eines Geräts oder Streams. Werte typischerweise zwischen 0.0 und 1.0 (oder höher, falls Übersteuerung erlaubt ist).  
  * **Invarianten:** channel\_volumes sollte nicht leer sein, wenn das Gerät aktiv ist. Alle Werte sollten ≥0.0.  
  * **Initialwert:** Abhängig vom Gerät; oft 1.0 für jeden Kanal.  
* \#  
  pub struct AudioDevice {... } (siehe Tabelle 5.1)  
* \#  
  pub struct StreamInfo {  
  * pub id: u32, // Globale PipeWire ID des Node-Objekts  
  * pub name: Option\<String\>, // Aus node.name oder application.name  
  * pub application\_name: Option\<String\>, // Aus application.name  
  * pub process\_id: Option\<u32\>, // Aus application.process.id  
  * pub volume: Volume,  
  * pub is\_muted: bool,  
  * pub media\_class: Option\<String\>, // z.B. "Stream/Output/Audio"  
  * pub node\_id\_pw: u32, // PipeWire interne Node ID (object.serial oder node.id) }  
  * **Zweck:** Repräsentiert einen aktiven Audio-Stream einer Anwendung.  
  * **Ableitung:** Aus den Eigenschaften eines PipeWire Node-Objekts.

#### **Tabelle 5.1: AudioDevice Strukturdefinition**

| Feldname | Rust-Typ | PipeWire Property / Quelle | Beschreibung | Initialwert (Beispiel) | Sichtbarkeit |
| :---- | :---- | :---- | :---- | :---- | :---- |
| id | u32 | global.id | Eindeutige globale ID des PipeWire-Objekts (Node oder Device). | \- | pub |
| proxy\_id | u32 | proxy.id() | ID des gebundenen Proxy-Objekts. | \- | pub(super) |
| name | Option\<String\> | node.nick, device.nick, node.name, device.name | Benutzerfreundlicher Name des Geräts. | None | pub |
| description | Option\<String\> | node.description, device.description | Detailliertere Beschreibung des Geräts. | None | pub |
| device\_type | AudioDeviceType | media.class (z.B. "Audio/Sink", "Audio/Source") | Typ des Audiogeräts (Sink oder Quelle). | Unknown | pub |
| volume | Volume | SPA\_PARAM\_Props (channelVolumes) oder SPA\_PARAM\_Route | Aktuelle Lautstärkeeinstellungen für jeden Kanal. | Volume { vols: vec\! } | pub |
| is\_muted | bool | SPA\_PARAM\_Props (mute) oder SPA\_PARAM\_Route | Gibt an, ob das Gerät stummgeschaltet ist. | false | pub |
| is\_default | bool | PipeWire:Interface:Metadata Objekt (default.audio.sink/source) | Gibt an, ob dies das Standardgerät seines Typs ist. | false | pub |
| ports | Option\<Vec\<PortInfo\>\> | SPA\_PARAM\_PortConfig / SPA\_PARAM\_EnumPortInfo | Informationen über die Ports des Geräts (optional, falls benötigt). | None | pub |
| properties\_spa | Option\<pipewire::spa::SpaDict\> | global.props / proxy.get\_properties() | Rohe PipeWire SPA-Eigenschaften (für Debugging oder erweiterte Infos). | None | pub(super) |
| is\_hardware\_device | bool | Abgeleitet aus device.api (z.B. "alsa", "bluez\_input") | Gibt an, ob es sich um ein physisches Hardwaregerät handelt. | false | pub |
| api\_name | Option\<String\> | device.api | Name der zugrundeliegenden API (z.B. "alsa", "v4l2", "libcamera"). | None | pub |

#### **Tabelle 5.2: AudioCommand Enum Varianten**

| Variante | Parameter | Beschreibung |
| :---- | :---- | :---- |
| SetDeviceVolume | device\_id: u32, volume: Volume, curve: VolumeCurve | Setzt die Lautstärke für ein bestimmtes Gerät. |
| SetDeviceMute | device\_id: u32, mute: bool | Schaltet ein bestimmtes Gerät stumm oder hebt die Stummschaltung auf. |
| SetStreamVolume | stream\_id: u32, volume: Volume, curve: VolumeCurve | Setzt die Lautstärke für einen bestimmten Anwendungsstream. |
| SetStreamMute | stream\_id: u32, mute: bool | Schaltet einen bestimmten Anwendungsstream stumm oder hebt die Stummschaltung auf. |
| SetDefaultDevice | device\_type: AudioDeviceType, device\_id: u32 | Setzt das Standardgerät für den angegebenen Typ (Sink/Source). |
| RequestDeviceList | \- | Fordert die aktuelle Liste aller bekannten Audiogeräte an. |
| RequestStreamList | \- | Fordert die aktuelle Liste aller bekannten Audio-Streams an. |

#### **Tabelle 5.3: AudioEvent Enum Varianten**

| Variante | Payload | Beschreibung |
| :---- | :---- | :---- |
| DeviceAdded | device: AudioDevice | Ein neues Audiogerät wurde dem System hinzugefügt. |
| DeviceRemoved | device\_id: u32 | Ein Audiogerät wurde vom System entfernt. |
| DeviceUpdated | device: AudioDevice | Eigenschaften eines Audiogeräts haben sich geändert (z.B. Name, Beschreibung). |
| DeviceVolumeChanged | device\_id: u32, new\_volume: Volume | Die Lautstärke eines Geräts hat sich geändert. |
| DeviceMuteChanged | device\_id: u32, is\_muted: bool | Der Stummschaltungsstatus eines Geräts hat sich geändert. |
| StreamAdded | stream: StreamInfo | Ein neuer Audio-Stream einer Anwendung wurde erkannt. |
| StreamRemoved | stream\_id: u32 | Ein Audio-Stream einer Anwendung wurde beendet. |
| StreamUpdated | stream: StreamInfo | Eigenschaften eines Streams haben sich geändert. |
| StreamVolumeChanged | stream\_id: u32, new\_volume: Volume | Die Lautstärke eines Anwendungsstreams hat sich geändert. |
| StreamMuteChanged | stream\_id: u32, is\_muted: bool | Der Stummschaltungsstatus eines Anwendungsstreams hat sich geändert. |
| DefaultSinkChanged | new\_device\_id: Option\<u32\> | Das Standard-Audioausgabegerät hat sich geändert. |
| DefaultSourceChanged | new\_device\_id: Option\<u32\> | Das Standard-Audioeingabegerät hat sich geändert. |
| AudioErrorOccurred | error: String | Ein Fehler im Audio-Subsystem ist aufgetreten. |
| DeviceListUpdated | devices: Vec\<AudioDevice\> | Antwort auf RequestDeviceList, enthält die aktuelle Geräteliste. |
| StreamListUpdated | streams: Vec\<StreamInfo\> | Antwort auf RequestStreamList, enthält die aktuelle Streamliste. |

### **5.3.5. Submodul: system::audio::spa\_pod\_utils \- SPA POD Konstruktionshilfsmittel**

* **Datei:** system/audio/spa\_pod\_utils.rs  
* **Zweck:** Enthält Hilfsfunktionen zur Erstellung von pipewire::spa::Pod (Simple Plugin API Plain Old Data) Objekten, die für das Setzen von Parametern wie Lautstärke und Stummschaltung über die PipeWire API benötigt werden.

#### **5.3.5.1. Funktionen**

* pub(super) fn build\_volume\_props\_pod(channel\_volumes: &\[f32\]) \-\> Result\<pipewire::spa::Pod, AudioError\>:  
  * **Vorbedingungen:** channel\_volumes enthält die gewünschten Lautstärkewerte pro Kanal (normalisiert, z.B. 0.0 bis 1.0).  
  * **Schritte:**  
    1. Erstellt einen pipewire::spa::pod::PodBuilder.  
    2. Beginnt ein Objekt (push\_object) vom Typ SPA\_TYPE\_OBJECT\_Props und ID SPA\_PARAM\_Props.  
    3. Fügt die Eigenschaft SPA\_PROP\_channelVolumes hinzu (prop(pipewire::spa::param::prop\_info::PropInfoType::channelVolumes.as\_raw(), 0)).  
    4. Fügt ein Array (push\_array) für die Float-Werte hinzu.  
    5. Iteriert über channel\_volumes und fügt jeden Wert als Float zum Array hinzu (float(vol)).  
    6. Schließt das Array (pop) und das Objekt (pop).  
    7. Gibt den erstellten Pod zurück.  
  * **Nachbedingungen:** Ein gültiger SpaPod für die Lautstärkeeinstellung ist erstellt.  
  * **Fehlerfälle:** AudioError::SpaPodBuildFailed, falls die Erstellung fehlschlägt.  
* pub(super) fn build\_mute\_props\_pod(mute: bool) \-\> Result\<pipewire::spa::Pod, AudioError\>:  
  * **Vorbedingungen:** mute enthält den gewünschten Stummschaltungsstatus.  
  * **Schritte:**  
    1. Erstellt einen pipewire::spa::pod::PodBuilder.  
    2. Beginnt ein Objekt (push\_object) vom Typ SPA\_TYPE\_OBJECT\_Props und ID SPA\_PARAM\_Props.  
    3. Fügt die Eigenschaft SPA\_PROP\_mute hinzu (prop(pipewire::spa::param::prop\_info::PropInfoType::mute.as\_raw(), 0)).  
    4. Fügt den booleschen Wert hinzu (boolean(mute)).  
    5. Schließt das Objekt (pop).  
    6. Gibt den erstellten Pod zurück.  
  * **Nachbedingungen:** Ein gültiger SpaPod für die Stummschaltung ist erstellt.  
  * **Fehlerfälle:** AudioError::SpaPodBuildFailed.  
* pub(super) fn build\_route\_volume\_pod(route\_index: u32, route\_device\_id: u32, channel\_volumes: &\[f32\]) \-\> Result\<pipewire::spa::Pod, AudioError\>:  
  * **Vorbedingungen:** route\_index und route\_device\_id identifizieren die Zielroute. channel\_volumes enthält die Lautstärkewerte.  
  * **Schritte:**  
    1. Erstellt einen pipewire::spa::pod::PodBuilder.  
    2. Beginnt ein Objekt (push\_object) vom Typ SPA\_TYPE\_OBJECT\_ParamRoute und ID SPA\_PARAM\_Route.  
    3. Fügt die Eigenschaft SPA\_PARAM\_ROUTE\_index mit route\_index hinzu (prop(...).int(route\_index)).  
    4. Fügt die Eigenschaft SPA\_PARAM\_ROUTE\_device mit route\_device\_id hinzu (prop(...).int(route\_device\_id)).  
    5. Fügt die Eigenschaft SPA\_PARAM\_ROUTE\_props hinzu.  
    6. Innerhalb von SPA\_PARAM\_ROUTE\_props ein weiteres Objekt (push\_object) vom Typ SPA\_TYPE\_OBJECT\_Props erstellen (ohne explizite ID, da es Teil der Route-Props ist).  
    7. Fügt SPA\_PROP\_channelVolumes und das Array der Float-Werte hinzu, wie in build\_volume\_props\_pod.  
    8. Schließt das innere Props-Objekt (pop) und das äußere Route-Objekt (pop).  
    9. Gibt den erstellten Pod zurück.  
  * **Nachbedingungen:** Ein SpaPod zum Setzen der Lautstärke einer spezifischen Route ist erstellt.  
  * **Fehlerfälle:** AudioError::SpaPodBuildFailed.  
* pub(super) fn build\_route\_mute\_pod(route\_index: u32, route\_device\_id: u32, mute: bool) \-\> Result\<pipewire::spa::Pod, AudioError\>:  
  * **Analoge Schritte** zu build\_route\_volume\_pod, aber für die SPA\_PROP\_mute-Eigenschaft innerhalb der SPA\_PARAM\_ROUTE\_props.  
* pub(super) fn parse\_props\_volume\_mute(pod: \&pipewire::spa::Pod) \-\> Result\<(Option\<Volume\>, Option\<bool\>), AudioError\>:  
  * **Vorbedingungen:** pod ist ein SpaPod, der vermutlich SPA\_PARAM\_Props repräsentiert.  
  * **Schritte:**  
    1. Iteriert durch die Eigenschaften des SpaPod-Objekts.  
    2. Sucht nach SPA\_PROP\_channelVolumes: Wenn gefunden, die Float-Werte aus dem Array extrahieren und in Volume verpacken.  
    3. Sucht nach SPA\_PROP\_mute: Wenn gefunden, den booleschen Wert extrahieren.  
    4. Gibt ein Tupel (Option\<Volume\>, Option\<bool\>) zurück.  
  * **Nachbedingungen:** Lautstärke und Mute-Status sind aus dem Pod extrahiert, falls vorhanden.  
  * **Fehlerfälle:** AudioError::SpaPodParseFailed, wenn die Struktur des Pods unerwartet ist.  
* pub(super) fn parse\_route\_props\_volume\_mute(pod: \&pipewire::spa::Pod) \-\> Result\<(Option\<Volume\>, Option\<bool\>), AudioError\>:  
  * **Vorbedingungen:** pod ist ein SpaPod, der SPA\_PARAM\_Route repräsentiert.  
  * **Schritte:**  
    1. Iteriert durch die Eigenschaften des SpaPod-Objekts (SPA\_TYPE\_OBJECT\_ParamRoute).  
    2. Sucht nach SPA\_PARAM\_ROUTE\_props.  
    3. Wenn gefunden, den inneren SpaPod (der SPA\_TYPE\_OBJECT\_Props sein sollte) mit parse\_props\_volume\_mute parsen.  
  * **Nachbedingungen:** Lautstärke und Mute-Status der Route sind extrahiert.  
  * **Fehlerfälle:** AudioError::SpaPodParseFailed.

### **5.3.6. Submodul: system::audio::error \- Fehlerbehandlung im Audio-Modul**

* **Datei:** system/audio/error.rs  
* **Zweck:** Definiert die spezifischen Fehlertypen für das system::audio-Modul unter Verwendung von thiserror.

#### **5.3.6.1. Enum AudioError**

* \# pub enum AudioError {  
  * \#\[error("PipeWire C API initialization failed.")\]  
    PipeWireInitFailed,  
  * \#\[error("Failed to create PipeWire MainLoop.")\]  
    MainLoopCreationFailed(\#\[source\] pipewire::Error),  
  * \#\[error("Failed to create PipeWire Context.")\]  
    ContextCreationFailed(\#\[source\] pipewire::Error),  
  * \#\[error("Failed to connect to PipeWire Core.")\]  
    CoreConnectionFailed(\#\[source\] pipewire::Error),  
  * \#  
    RegistryCreationFailed(\#\[source\] pipewire::Error),  
  * \#\[error("PipeWire thread panicked or failed to initialize.")\]  
    PipeWireThreadPanicked,  
  * \#\[error("Failed to bind to PipeWire proxy for global id {global\_id}: {source}")\]  
    ProxyBindFailed { global\_id: u32, \#\[source\] source: pipewire::Error },  
  * \#\[error("Failed to enumerate parameters for object id {object\_id}: {source}")\]  
    ParameterEnumerationFailed { object\_id: u32, \#\[source\] source: pipewire::Error },  
  * \#  
    SpaPodParseFailed { message: String },  
  * \#  
    SpaPodBuildFailed { message: String },  
  * \#\[error("PipeWire command failed for object {object\_id}: {source}")\]  
    PipeWireCommandFailed { object\_id: u32, \#\[source\] source: pipewire::Error },  
  * \#  
    DeviceOrStreamNotFound(u32),  
  * \#  
    NoActiveRouteFound(u32),  
  * \#\[error("PipeWire Metadata proxy is not available.")\]  
    MetadataProxyNotAvailable,  
  * \#  
    InternalChannelSendError(String),  
  * \#  
    InternalBroadcastSendError(String),  
    }  
  * **Begründung für thiserror**: thiserror wird verwendet, um Boilerplate-Code für die Implementierung von std::error::Error und std::fmt::Display zu reduzieren. Es ermöglicht klare, kontextbezogene Fehlermeldungen und die einfache Einbettung von Quellfehlern (\#\[from\] oder \#\[source\]). Dies ist entscheidend für die Diagnose von Problemen in einem komplexen Subsystem wie der Audioverwaltung.33 Die spezifischen Fehlervarianten ermöglichen es aufrufendem Code, differenziert auf Fehler zu reagieren.

## **6\. system::mcp \- Model Context Protocol Client**

Das Modul system::mcp implementiert einen Client für das Model Context Protocol (MCP). MCP ist ein offener Standard für die sichere und standardisierte Verbindung von KI-Modellen (LLMs) mit externen Werkzeugen, Datenquellen und Anwendungen, wie dieser Desktop-Umgebung.37 Dieses Modul ermöglicht es der Desktop-Umgebung, mit lokalen oder Cloud-basierten MCP-Servern zu kommunizieren, um KI-gestützte Funktionen bereitzustellen. Die Kommunikation erfolgt typischerweise über Stdio, wobei JSON-RPC-Nachrichten ausgetauscht werden.

* **Kernfunktionalität**:  
  * Senden von Anfragen an einen MCP-Server (z.B. tool\_run, resource\_list).  
  * Empfangen und Verarbeiten von Antworten und asynchronen Benachrichtigungen vom Server.  
  * Verwaltung des Verbindungsstatus zum MCP-Server.  
* **Verwendete Crates**: mcp\_client\_rs 37 oder mcpr 38 als Basis für die MCP-Client-Implementierung. Die Wahl fiel auf mcp\_client\_rs (von darinkishore) aufgrund seiner direkten Stdio-Transportunterstützung und klaren Client-API.  
* **Modulstruktur und Dateien**:  
  * system/mcp/mod.rs: Öffentliche API, McpError Enum.  
  * system/mcp/client.rs: McpClient-Struktur, Logik zum Senden von Anfragen und Empfangen von Antworten/Benachrichtigungen.  
  * system/mcp/transport.rs: Implementierung des Stdio-Transports, falls nicht vollständig vom Crate abgedeckt oder Anpassungen nötig sind.  
  * system/mcp/types.rs: Definitionen für MCP-Anfragen, \-Antworten und \-Benachrichtigungen, die für die Desktop-Umgebung relevant sind (ggf. Wrapper um Crate-Typen).  
  * system/mcp/error.rs: Fehlerbehandlung für das MCP-Modul.

### **5.4.1. Submodul: system::mcp::client \- MCP Client Kernlogik**

* **Datei:** system/mcp/client.rs  
* **Zweck:** Dieses Submodul enthält die Kernlogik für die Interaktion mit einem MCP-Server. Es ist verantwortlich für das Starten des MCP-Server-Prozesses (falls lokal), das Senden von Anfragen und das Verarbeiten von Antworten und serverseitigen Benachrichtigungen.

#### **5.4.1.1. Strukuren**

* pub struct McpClient:  
  * client\_handle: Option\<mcp\_client\_rs::client::Client\>: Die eigentliche Client-Instanz aus dem mcp\_client\_rs-Crate. Option, da die Verbindung fehlschlagen oder noch nicht etabliert sein kann.  
  * server\_process: Option\<tokio::process::Child\>: Handle für den Kindprozess des MCP-Servers, falls dieser lokal von der Desktop-Umgebung gestartet wird.  
  * command\_sender: tokio::sync::mpsc::Sender\<McpCommand\>: Sender für Befehle an den MCP-Verwaltungs-Task.  
  * notification\_broadcaster: tokio::sync::broadcast::Sender\<McpNotification\>: Sender zum Verteilen von MCP-Benachrichtigungen an interessierte Systemkomponenten.  
  * status\_broadcaster: tokio::sync::broadcast::Sender\<McpClientStatus\>: Sender zum Verteilen von Statusänderungen des MCP-Clients.  
  * request\_id\_counter: std::sync::Arc\<std::sync::atomic::AtomicU64\>: Atomarer Zähler zur Generierung eindeutiger Request-IDs für JSON-RPC.  
  * pending\_requests: std::sync::Arc\<tokio::sync::Mutex\<std::collections::HashMap\<String, tokio::sync::oneshot::Sender\<Result\<serde\_json::Value, McpError\>\>\>\>\>: Speichert oneshot::Sender für jede ausstehende Anfrage, um die Antwort an den ursprünglichen Aufrufer weiterzuleiten. Der Key ist die Request-ID.  
  * listen\_task\_handle: Option\<tokio::task::JoinHandle\<()\>\>: Handle für den Tokio-Task, der eingehende Nachrichten vom MCP-Server verarbeitet.  
* pub struct McpServerConfig:  
  * command: String: Der auszuführende Befehl zum Starten des MCP-Servers (z.B. "/usr/bin/my\_mcp\_server").  
  * args: Vec\<String\>: Argumente für den Server-Befehl.  
  * working\_directory: Option\<String\>: Arbeitsverzeichnis für den Serverprozess.

#### **5.4.1.2. Enums**

* pub enum McpClientStatus:  
  * Disconnected: Der Client ist nicht verbunden.  
  * Connecting: Der Client versucht, eine Verbindung herzustellen.  
  * Connected: Der Client ist verbunden und initialisiert.  
  * Error(String): Ein Fehler ist aufgetreten.  
* pub enum McpCommand:  
  * Initialize { params: mcp\_client\_rs::protocol::InitializeParams }  
  * ListResources { params: mcp\_client\_rs::protocol::ListResourcesParams, response\_tx: tokio::sync::oneshot::Sender\<Result\<mcp\_client\_rs::protocol::ListResourcesResult, McpError\>\> }  
  * ReadResource { params: mcp\_client\_rs::protocol::ReadResourceParams, response\_tx: tokio::sync::oneshot::Sender\<Result\<mcp\_client\_rs::protocol::ReadResourceResult, McpError\>\> }  
  * CallTool { params: mcp\_client\_rs::protocol::CallToolParams, response\_tx: tokio::sync::oneshot::Sender\<Result\<mcp\_client\_rs::protocol::CallToolResult, McpError\>\> }  
  * Shutdown  
  * SubscribeToNotifications { subscriber: tokio::sync::broadcast::Sender\<McpNotification\> } (Beispiel für eine spezifischere Benachrichtigungsbehandlung)

#### **5.4.1.3. Methoden für McpClient**

* pub async fn new(server\_config: McpServerConfig, notification\_broadcaster: tokio::sync::broadcast::Sender\<McpNotification\>, status\_broadcaster: tokio::sync::broadcast::Sender\<McpClientStatus\>) \-\> Result\<Self, McpError\>:  
  * **Vorbedingungen:** server\_config ist gültig.  
  * **Schritte:**  
    1. Erstellt einen tokio::sync::mpsc::channel für McpCommand.  
    2. Initialisiert request\_id\_counter und pending\_requests.  
    3. Startet den MCP-Serverprozess gemäß server\_config mit tokio::process::Command. Stdin, Stdout und Stderr des Kindprozesses müssen für die Kommunikation verfügbar gemacht werden (Pipes). 37  
       * let mut command \= tokio::process::Command::new(\&server\_config.command);  
       * command.args(\&server\_config.args).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped());  
       * if let Some(wd) \= \&server\_config.working\_directory { command.current\_dir(wd); }  
       * let child \= command.spawn().map\_err(|e| McpError::ServerSpawnFailed(e.to\_string()))?;  
    4. Nimmt stdin und stdout des Kindprozesses.  
    5. Erstellt einen mcp\_client\_rs::transport::stdio::StdioTransport mit den Pipes des Kindprozesses. 37  
    6. Erstellt eine mcp\_client\_rs::client::Client-Instanz mit dem Transport.  
    7. Speichert client\_handle und server\_process.  
    8. Startet den listen\_task (siehe unten) mit tokio::spawn.  
    9. Sendet McpClientStatus::Connecting über status\_broadcaster.  
    10. Sendet einen Initialize-Befehl an den command\_sender, um die MCP-Sitzung zu initialisieren. Wartet auf die Antwort.  
    11. Bei Erfolg: Sendet McpClientStatus::Connected über status\_broadcaster.  
    12. Bei Fehler: Sendet McpClientStatus::Error und gibt Fehler zurück.  
  * **Nachbedingungen:** MCP-Client ist initialisiert und verbunden, oder ein Fehler wird zurückgegeben. Der listen\_task läuft.  
  * **Fehlerfälle:** McpError::ServerSpawnFailed, McpError::TransportError, McpError::InitializationFailed.  
* async fn listen\_task(mut client\_transport\_rx: mcp\_client\_rs::transport::stdio::StdioTransportReceiver, /\*... \*/):  
  * **Logik:** Diese asynchrone Funktion läuft in einem eigenen Tokio-Task.  
  * Sie lauscht kontinuierlich auf eingehende Nachrichten vom StdioTransportReceiver (der rx-Teil des StdioTransport).  
  * Jede empfangene Nachricht (eine JSON-Zeichenkette) wird deserialisiert:  
    * Wenn es eine Antwort auf eine Anfrage ist (enthält id):  
      1. Sucht den passenden oneshot::Sender in pending\_requests anhand der id.  
      2. Sendet das Ergebnis (erfolgreiche Antwort oder Fehlerobjekt aus der Nachricht) über den oneshot::Sender.  
      3. Entfernt den Eintrag aus pending\_requests.  
    * Wenn es eine Benachrichtigung ist (enthält method, aber keine id):  
      1. Konvertiert die Benachrichtigung in eine McpNotification.  
      2. Sendet die McpNotification über den notification\_broadcaster.  
    * Wenn es eine Fehlermeldung ist, die nicht zu einer bestimmten Anfrage gehört (selten, aber möglich):  
      1. Loggt den Fehler.  
      2. Sendet ggf. McpClientStatus::Error.  
  * Behandelt Lese-/Deserialisierungsfehler und den Fall, dass der Server die Verbindung schließt (EOF auf Stdio). In solchen Fällen wird McpClientStatus::Disconnected oder McpClientStatus::Error gesendet und der Task beendet sich.  
  * Die mcp\_client\_rs Bibliothek könnte bereits einen Mechanismus zum Empfangen und Verarbeiten von Nachrichten bereitstellen (z.B. einen Stream von Nachrichten oder Callbacks). Diese Funktion würde diesen Mechanismus nutzen. 37  
* async fn send\_request\_generic\<P, R\>(\&self, method: \&str, params: P) \-\> Result\<R, McpError\>  
  where P: serde::Serialize \+ Send, R: serde::de::DeserializeOwned \+ Send:  
  * **Vorbedingungen:** Client ist verbunden.  
  * **Schritte:**  
    1. Wenn client\_handle None ist, McpError::NotConnected zurückgeben.  
    2. Generiert eine eindeutige request\_id (z.B. mit self.request\_id\_counter.fetch\_add(1, std::sync::atomic::Ordering::Relaxed).to\_string()).  
    3. Erstellt einen tokio::sync::oneshot::channel für die Antwort.  
    4. Speichert den response\_tx in pending\_requests mit der request\_id als Schlüssel.  
    5. Erstellt die JSON-RPC-Anfrage-Struktur (z.B. mcp\_client\_rs::protocol::Request).  
    6. Serialisiert die Anfrage zu einem JSON-String.  
    7. Sendet den JSON-String über den writer-Teil des StdioTransport des client\_handle. Dies wird von mcp\_client\_rs intern gehandhabt, z.B. durch eine Methode wie client.send\_request(req\_obj).await.  
    8. Wartet auf die Antwort über response\_rx.await.  
    9. Gibt das Ergebnis zurück.  
  * **Nachbedingungen:** Anfrage wurde gesendet und auf Antwort gewartet.  
  * **Fehlerfälle:** McpError::NotConnected, McpError::SerializationFailed, McpError::TransportError, McpError::RequestTimeout (falls implementiert), McpError::ServerReturnedError.  
* pub async fn list\_resources(\&self, params: mcp\_client\_rs::protocol::ListResourcesParams) \-\> Result\<mcp\_client\_rs::protocol::ListResourcesResult, McpError\>:  
  * Ruft self.send\_request\_generic("resource/list", params).await auf.  
* pub async fn read\_resource(\&self, params: mcp\_client\_rs::protocol::ReadResourceParams) \-\> Result\<mcp\_client\_rs::protocol::ReadResourceResult, McpError\>:  
  * Ruft self.send\_request\_generic("resource/read", params).await auf.  
* pub async fn call\_tool(\&self, params: mcp\_client\_rs::protocol::CallToolParams) \-\> Result\<mcp\_client\_rs::protocol::CallToolResult, McpError\>:  
  * Ruft self.send\_request\_generic("tool/run", params).await auf. 37  
* pub async fn shutdown(\&mut self) \-\> Result\<(), McpError\>:  
  * **Vorbedingungen:** Keine.  
  * **Schritte:**  
    1. Wenn client\_handle Some ist, eine shutdown-Anfrage an den Server senden (falls vom MCP-Protokoll spezifiziert und von mcp\_client\_rs unterstützt).  
    2. Den listen\_task abbrechen (self.listen\_task\_handle.as\_ref().map(|h| h.abort())).  
    3. Wenn server\_process Some ist, dem Kindprozess ein SIGTERM senden und auf sein Beenden warten (child.kill().await, child.wait().await).  
    4. self.client\_handle \= None; self.server\_process \= None;  
    5. Sendet McpClientStatus::Disconnected über status\_broadcaster.  
  * **Nachbedingungen:** Client ist heruntergefahren, Serverprozess (falls lokal) ist beendet.  
  * **Fehlerfälle:** McpError::TransportError.  
* pub fn get\_command\_sender(\&self) \-\> tokio::sync::mpsc::Sender\<McpCommand\>:  
  * Gibt einen Klon des command\_sender zurück.

### **5.4.2. Submodul: system::mcp::transport \- MCP Kommunikationstransport**

* **Datei:** system/mcp/transport.rs  
* **Zweck:** Dieses Submodul ist primär eine Abstraktionsebene, falls die verwendete mcp\_client\_rs-Bibliothek keine direkte oder anpassbare Stdio-Transportimplementierung bietet, die unseren Anforderungen genügt (z.B. spezifische Fehlerbehandlung, Logging-Integration). In den meisten Fällen wird die Transportlogik direkt vom mcp\_client\_rs::transport::stdio::StdioTransport gehandhabt.  
  * Die mcp\_client\_rs Bibliothek 37 und mcpr 38 bieten bereits Stdio-Transportmechanismen. Diese werden direkt in system::mcp::client verwendet.  
  * Dieses Modul würde nur dann eigene Implementierungen enthalten, wenn eine tiefgreifende Anpassung des Transports notwendig wäre, was aktuell nicht der Fall ist.

### **5.4.3. Submodul: system::mcp::types \- MCP Nachrichtenstrukturen und Datentypen**

* **Datei:** system/mcp/types.rs  
* **Zweck:** Definiert Rust-Strukturen, die MCP-Anfragen, \-Antworten und \-Benachrichtigungen entsprechen, sowie alle relevanten Datentypen. Diese können direkte Wrapper um die Typen aus mcp\_client\_rs::protocol und mcp\_client\_rs::types sein oder bei Bedarf eigene, anwendungsspezifische Abstraktionen darstellen.

#### **5.4.3.1. Strukuren und Enums (Beispiele, basierend auf mcp\_client\_rs und MCP-Spezifikation)**

Die meisten dieser Typen werden direkt aus dem mcp\_client\_rs::protocol und mcp\_client\_rs::types Modul re-exportiert oder als dünne Wrapper verwendet.

* pub use mcp\_client\_rs::protocol::{InitializeParams, InitializeResult, ErrorResponse, Notification, Request, Response, ListResourcesParams, ListResourcesResult, ReadResourceParams, ReadResourceResult, CallToolParams, CallToolResult, Resource, Tool}; 37  
* pub use mcp\_client\_rs::types::{Content, Document, ErrorCode, ErrorData, Message, MessageId, NotificationMessage, RequestMessage, ResponseMessage, Version}; 37  
* \#  
  pub struct McpNotification {  
  * pub method: String,  
  * pub params: Option\<serde\_json::Value\>, }  
  * **Zweck:** Eine generische Struktur für vom Server empfangene Benachrichtigungen.  
  * **Ableitung:** Aus mcp\_client\_rs::protocol::Notification.

### **5.4.4. Submodul: system::mcp::error \- MCP Client Fehlerbehandlung**

* **Datei:** system/mcp/error.rs  
* **Zweck:** Definiert die spezifischen Fehlertypen für das system::mcp-Modul.

#### **5.4.4.1. Enum McpError**

* \# pub enum McpError {  
  * \#\[error("Failed to spawn MCP server process: {0}")\]  
    ServerSpawnFailed(String),  
  * \#  
    TransportError(\#\[from\] mcp\_client\_rs::Error), // Direkte Konvertierung von Fehlern des mcp\_client\_rs Crates  
  * \#\[error("MCP client is not connected or initialized.")\]  
    NotConnected,  
  * \#\[error("Failed to initialize MCP session with server: {0}")\]  
    InitializationFailed(String), // Kann Details vom Server-Error enthalten  
  * \#\[error("Failed to serialize request: {0}")\]  
    SerializationFailed(\#\[from\] serde\_json::Error),  
  * \#  
    RequestTimeout,  
  * \#\[error("MCP server returned an error: {code} \- {message}")\]  
    ServerReturnedError { code: i64, message: String, data: Option\<serde\_json::Value\> }, // Basierend auf JSON-RPC Fehlerobjekt  
  * \#  
    UnexpectedResponse { request\_id: String },  
  * \#  
    ResponseChannelDropped { request\_id: String },  
  * \#\[error("Failed to send command to MCP client task: {0}")\]  
    CommandSendError(String),  
    }  
  * Die Felder in ServerReturnedError entsprechen typischen JSON-RPC-Fehlerobjekten.  
  * \#\[from\] wird verwendet, um Fehler von serde\_json und mcp\_client\_rs::Error direkt in McpError umzuwandeln, was die Fehlerbehandlung vereinfacht.33

## **7\. system::portals \- XDG Desktop Portals Backend**

Das Modul system::portals implementiert die Backend-Logik für ausgewählte XDG Desktop Portals.60 Diese Portale ermöglichen es sandboxed Anwendungen (wie Flatpaks, aber auch nativen Anwendungen), sicher auf Ressourcen außerhalb ihrer Sandbox zuzugreifen, z.B. für Dateiauswahldialoge oder Screenshots. Dieses Modul agiert als D-Bus-Dienst, der die Portal-Schnittstellen implementiert und Anfragen von Client-Anwendungen bearbeitet.

* **Kernfunktionalität**:  
  * Implementierung der D-Bus-Schnittstellen für org.freedesktop.portal.FileChooser und org.freedesktop.portal.Screenshot.  
  * Interaktion mit der UI-Schicht zur Anzeige von Dialogen (z.B. Dateiauswahl).  
  * Interaktion mit dem Compositor (Systemschicht) für Aktionen wie Screenshots.  
* **Verwendete Crates**: zbus für die D-Bus-Implementierung 83, ashpd (Rust-Bindings für XDG Desktop Portals, falls für Backend-Implementierung nützlich, ansonsten direkte D-Bus-Implementierung). Die Entscheidung fällt auf eine direkte Implementierung mit zbus, um volle Kontrolle zu behalten und keine unnötigen Abstraktionen einzuführen, da wir das Backend selbst bereitstellen.  
* **Modulstruktur und Dateien**:  
  * system/portals/mod.rs: Öffentliche API, PortalsError Enum, Startpunkt für den D-Bus-Dienst.  
  * system/portals/file\_chooser.rs: Implementierung des org.freedesktop.portal.FileChooser-Interfaces.  
  * system/portals/screenshot.rs: Implementierung des org.freedesktop.portal.Screenshot-Interfaces.  
  * system/portals/common.rs: Gemeinsame Hilfsfunktionen, D-Bus-Setup, Request-Handling-Logik.  
  * system/portals/error.rs: Fehlerbehandlung für das Portals-Modul.

### **5.5.1. Submodul: system::portals::file\_chooser \- FileChooser Portal Backend**

* **Datei:** system/portals/file\_chooser.rs  
* **Zweck:** Implementiert die D-Bus-Schnittstelle org.freedesktop.portal.FileChooser. Dieses Portal ermöglicht Anwendungen das Öffnen und Speichern von Dateien über einen systemeigenen Dialog, der vom Desktop-Environment bereitgestellt wird.

#### **5.5.1.1. Struktur FileChooserPortal**

* pub struct FileChooserPortal {  
  * connection: std::sync::Arc\<zbus::Connection\>,  
  * // Referenz auf UI-Service oder Kommunikationskanal zur UI-Schicht,  
  * // um Dateiauswahldialoge anzuzeigen.  
  * // z.B. ui\_event\_sender: tokio::sync::mpsc::Sender\<UiPortalCommand\> }  
  * **Initialwerte:** connection wird bei der Instanziierung übergeben. UI-Kommunikationskanäle werden ebenfalls initialisiert.  
  * **Invarianten:** connection muss eine gültige D-Bus-Verbindung sein.

#### **5.5.1.2. D-Bus Interface Implementierung (\#\[zbus::interface\])**

* **Interface-Name:** org.freedesktop.portal.FileChooser  
* **Objektpfad:** (Wird vom system::portals::common oder main.rs beim Starten des Dienstes festgelegt, typischerweise /org/freedesktop/portal/desktop)  
* **Methoden:**  
  * async fn OpenFile(\&self, parent\_window: String, title: String, options: std::collections::HashMap\<String, zbus::zvariant::Value\<'static\>\>) \-\> zbus::fdo::Result\<(u32, std::collections::HashMap\<String, zbus::zvariant::Value\<'static\>\>)\>  
    * **Spezifikation:** 66  
    * **Parameter parent\_window (s):** Kennung des Anwendungsfensters (oft leer, "x11:XID" oder "wayland:HANDLE"). Wird derzeit nicht streng validiert, aber für zukünftige Modalitätslogik gespeichert.  
    * **Parameter title (s):** Titel für den Dialog.  
    * **Parameter options (a{sv}):**  
      * handle\_token (s): Eindeutiges Token für die Anfrage.  
      * accept\_label (s): Optionaler Text für den "Öffnen"-Button.  
      * modal (b): Ob der Dialog modal sein soll (Standard: true).  
      * multiple (b): Ob Mehrfachauswahl erlaubt ist (Standard: false).  
      * directory (b): Ob Ordner statt Dateien ausgewählt werden sollen (Standard: false).  
      * filters (a(sa(us))): Liste von Dateifiltern. Jeder Filter: (String Name, Array\<Tuple\<u32 Typ, String Muster/MIME\>\>)  
      * current\_filter ((sa(us))): Standardmäßig ausgewählter Filter.  
      * choices (a(ssa(ss)s)): Zusätzliche Auswahlmöglichkeiten (Comboboxen/Checkboxen).  
      * current\_folder (ay): Vorgeschlagener Startordner (als Byte-Array, NUL-terminiert).  
    * **Rückgabe:** handle (o) \- Ein Objektpfad für das Request-Objekt. Die eigentlichen Ergebnisse (URIs) werden asynchron über das Response-Signal des Request-Objekts gesendet.  
      * Die Implementierung hier gibt ein Tupel (u32 response\_code, a{sv} results) direkt zurück, wie es in vielen Portal-Implementierungen üblich ist, wenn kein separates Request-Objekt für einfache Fälle erstellt wird. response\_code \= 0 für Erfolg.  
      * results enthält uris (as) und choices (a(ss)).  
    * **Implementierungsschritte:**  
      1. Generiert eine eindeutige request\_handle (z.B. basierend auf handle\_token oder UUID).  
      2. Extrahiert Optionen wie multiple, directory, filters aus options.  
      3. Sendet einen Befehl an die UI-Schicht, um einen Dateiauswahldialog mit den gegebenen Parametern anzuzeigen. Dies erfordert einen Mechanismus (z.B. einen MPSC-Kanal), um mit der UI-Schicht zu kommunizieren und das Ergebnis (ausgewählte URIs) zurückzuerhalten.  
      4. Wartet asynchron auf die Antwort von der UI-Schicht.  
      5. Wenn die UI einen oder mehrere Datei-URIs zurückgibt:  
         * Erstellt ein results Dictionary: {"uris": zbus::zvariant::Value::from(vec\!\["file:///path/to/file1",...\])}.  
         * Gibt Ok((0, results\_dict)) zurück.  
      6. Wenn der Benutzer abbricht oder ein Fehler auftritt:  
         * Gibt Ok((1, HashMap::new())) für Abbruch durch Benutzer oder einen entsprechenden Fehlercode für andere Fehler zurück.  
         * Alternativ einen D-Bus-Fehler werfen: Err(zbus::fdo::Error::Failed("Dialog cancelled by user".into())).  
  * async fn SaveFile(\&self, parent\_window: String, title: String, options: std::collections::HashMap\<String, zbus::zvariant::Value\<'static\>\>) \-\> zbus::fdo::Result\<(u32, std::collections::HashMap\<String, zbus::zvariant::Value\<'static\>\>)\>  
    * **Spezifikation:** 66  
    * **Parameter options (a{sv}):** Zusätzlich zu den OpenFile-Optionen:  
      * current\_name (s): Vorgeschlagener Dateiname.  
      * current\_file (ay): Pfad zur aktuell zu speichernden Datei (falls "Speichern unter" für eine vorhandene Datei).  
    * **Implementierungsschritte:** Ähnlich wie OpenFile, aber die UI zeigt einen "Speichern"-Dialog an. Die UI gibt einen einzelnen URI zurück.  
  * async fn SaveFiles(\&self, parent\_window: String, title: String, options: std::collections::HashMap\<String, zbus::zvariant::Value\<'static\>\>) \-\> zbus::fdo::Result\<(u32, std::collections::HashMap\<String, zbus::zvariant::Value\<'static\>\>)\>  
    * **Spezifikation:** 66  
    * **Parameter options (a{sv}):** Zusätzlich zu den OpenFile-Optionen (außer multiple, directory):  
      * files (aay): Array von Byte-Arrays, die die zu speichernden Dateinamen repräsentieren.  
    * **Implementierungsschritte:**  
      1. Die UI wird angewiesen, einen Ordnerauswahldialog anzuzeigen.  
      2. Nach Auswahl eines Ordners durch den Benutzer konstruiert dieses Backend die vollständigen URIs, indem die in options\["files"\] übergebenen Dateinamen an den ausgewählten Ordnerpfad angehängt werden.  
      3. Gibt die Liste der resultierenden URIs zurück.  
* **Signale:** Das FileChooser-Interface selbst definiert keine Signale. Antworten werden über das Response-Signal des Request-Objekts gesendet, das durch den handle-Ausgabeparameter der Methoden referenziert wird. Für eine vereinfachte Implementierung ohne explizite Request-Objekte werden die Ergebnisse direkt zurückgegeben.

### **5.5.2. Submodul: system::portals::screenshot \- Screenshot Portal Backend**

* **Datei:** system/portals/screenshot.rs  
* **Zweck:** Implementiert die D-Bus-Schnittstelle org.freedesktop.portal.Screenshot. Dieses Portal ermöglicht Anwendungen das Erstellen von Screenshots und das Auswählen von Bildschirmfarben.

#### **5.5.2.1. Struktur ScreenshotPortal**

* pub struct ScreenshotPortal {  
  * connection: std::sync::Arc\<zbus::Connection\>,  
  * // Referenz/Kanal zum Compositor (Systemschicht), um Screenshot-Aktionen auszulösen  
  * // z.B. compositor\_command\_sender: tokio::sync::mpsc::Sender\<CompositorScreenshotCommand\> }  
  * **Initialwerte:** connection wird bei der Instanziierung übergeben. Compositor-Kommunikationskanäle werden ebenfalls initialisiert.  
  * **Invarianten:** connection muss eine gültige D-Bus-Verbindung sein.

#### **5.5.2.2. D-Bus Interface Implementierung (\#\[zbus::interface\])**

* **Interface-Name:** org.freedesktop.portal.Screenshot  
* **Objektpfad:** (Wird vom system::portals::common oder main.rs beim Starten des Dienstes festgelegt, typischerweise /org/freedesktop/portal/desktop)  
* **Methoden:**  
  * async fn Screenshot(\&self, parent\_window: String, options: std::collections::HashMap\<String, zbus::zvariant::Value\<'static\>\>) \-\> zbus::fdo::Result\<(u32, std::collections::HashMap\<String, zbus::zvariant::Value\<'static\>\>)\>  
    * **Spezifikation:** 67  
    * **Parameter options (a{sv}):**  
      * handle\_token (s): Eindeutiges Token für die Anfrage.  
      * modal (b): Ob der Dialog modal sein soll (Standard: true).  
      * interactive (b): Ob der Benutzer Optionen zur Auswahl des Bereichs etc. erhalten soll (Standard: false). **Seit Version 2 des Protokolls.**  
    * **Rückgabe:** handle (o) \- Objektpfad für das Request-Objekt. Hier vereinfacht zu direkter Rückgabe.  
    * **Implementierungsschritte:**  
      1. Extrahiert interactive aus options.  
      2. Sendet einen Befehl an den Compositor (Systemschicht), einen Screenshot zu erstellen.  
         * Wenn interactive true ist, sollte der Compositor dem Benutzer erlauben, einen Bereich auszuwählen oder ein Fenster etc.  
         * Wenn interactive false ist, wird ein Screenshot des gesamten Bildschirms (oder des primären Bildschirms) erstellt.  
      3. Der Compositor speichert den Screenshot temporär (z.B. in $XDG\_RUNTIME\_DIR/screenshots) und gibt den Dateipfad zurück.  
      4. Konvertiert den Dateipfad in einen file:// URI.  
      5. Erstellt ein results Dictionary: {"uri": zbus::zvariant::Value::from(screenshot\_uri)}.  
      6. Gibt Ok((0, results\_dict)) zurück.  
      7. Bei Fehlern (Compositor-Fehler, Speicherfehler): Ok((error\_code,...)) oder Err(zbus::fdo::Error).  
  * async fn PickColor(\&self, parent\_window: String, options: std::collections::HashMap\<String, zbus::zvariant::Value\<'static\>\>) \-\> zbus::fdo::Result\<(u32, std::collections::HashMap\<String, zbus::zvariant::Value\<'static\>\>)\>  
    * **Spezifikation:** 67  
    * **Parameter options (a{sv}):**  
      * handle\_token (s): Eindeutiges Token.  
    * **Implementierungsschritte:**  
      1. Sendet einen Befehl an den Compositor, den Farbauswahlmodus zu starten (z.B. Anzeige einer Lupe unter dem Cursor).  
      2. Der Compositor meldet die ausgewählte Farbe (RGB-Werte, typischerweise als Tupel von f64 im Bereich ) zurück.  
      3. Erstellt ein results Dictionary: {"color": zbus::zvariant::Value::from((r, g, b))}.  
      4. Gibt Ok((0, results\_dict)) zurück.  
* **Properties (Version Property):**  
  * \#\[zbus(property(emits\_changed\_signal \= "const"))\] async fn version(\&self) \-\> u32 { 2 } // Oder die höchste unterstützte Version  
    * **Spezifikation:** 77  
    * Gibt die implementierte Version des Screenshot-Portals zurück.

### **5.5.3. Submodul: system::portals::common \- Gemeinsame Portal-Hilfsmittel & D-Bus Handhabung**

* **Datei:** system/portals/common.rs  
* **Zweck:** Enthält Code, der von mehreren Portal-Implementierungen gemeinsam genutzt wird, wie z.B. das Starten des D-Bus-Dienstes, die Registrierung von Objekten und Schnittstellen sowie Hilfsfunktionen für die Interaktion mit der UI- oder Systemschicht.

#### **5.5.3.1. Funktionen**

* pub async fn run\_portal\_service(ui\_command\_sender: tokio::sync::mpsc::Sender\<UiPortalCommand\>, compositor\_command\_sender: tokio::sync::mpsc::Sender\<CompositorScreenshotCommand\>) \-\> Result\<(), PortalsError\>:  
  * **Vorbedingungen:** Keine.  
  * **Schritte:**  
    1. Erstellt eine neue D-Bus-Verbindung zum Session-Bus: let connection \= zbus::ConnectionBuilder::session()?.build().await?;.83  
    2. Registriert den Dienstnamen org.freedesktop.portal.Desktop: connection.request\_name("org.freedesktop.portal.Desktop").await?;  
    3. Erstellt Instanzen der Portal-Implementierungen:  
       * let file\_chooser\_portal \= Arc::new(FileChooserPortal { connection: connection.clone(), /\* ui\_event\_sender \*/ });  
       * let screenshot\_portal \= Arc::new(ScreenshotPortal { connection: connection.clone(), /\* compositor\_command\_sender \*/ });  
    4. Registriert die Portal-Objekte und ihre Schnittstellen beim ObjectServer der Verbindung:  
       * connection.object\_server().at("/org/freedesktop/portal/desktop", file\_chooser\_portal).await?;  
       * connection.object\_server().at("/org/freedesktop/portal/desktop", screenshot\_portal).await?;  
         * **Hinweis:** zbus erlaubt das Hinzufügen mehrerer Interfaces zum selben Pfad, wenn die Interfaces unterschiedliche Namen haben. Wenn FileChooserPortal und ScreenshotPortal als separate Rust-Strukturen implementiert sind, die jeweils ein Interface bereitstellen, müssen sie entweder auf unterschiedlichen Pfaden registriert werden (was nicht der XDG-Spezifikation entspricht) oder eine einzelne Struktur muss alle Portal-Interfaces implementieren, die unter /org/freedesktop/portal/desktop angeboten werden.  
         * **Korrekter Ansatz:** Eine einzelne Struktur DesktopPortal erstellen, die alle Portal-Interfaces (FileChooser, Screenshot, etc.) als Traits implementiert oder Instanzen der spezifischen Portal-Handler hält und die Aufrufe an diese delegiert.

    Rust  
           // In system::portals::common.rs oder mod.rs  
           pub struct DesktopPortal {  
               file\_chooser: Arc\<FileChooserPortal\>,  
               screenshot: Arc\<ScreenshotPortal\>,  
               //... andere Portale  
           }

           \#\[zbus::interface(name \= "org.freedesktop.portal.FileChooser")\]  
           impl DesktopPortal {  
               async fn OpenFile(...) { self.file\_chooser.OpenFile(...).await }  
               //...  
           }

           \#  
           impl DesktopPortal {  
               async fn Screenshot(...) { self.screenshot.Screenshot(...).await }  
               //...  
           }  
           // In run\_portal\_service:  
           // let desktop\_portal\_impl \= Arc::new(DesktopPortal { file\_chooser, screenshot });  
           // connection.object\_server().at("/org/freedesktop/portal/desktop", desktop\_portal\_impl).await?;

    5. Die Funktion tritt in eine Schleife ein oder verwendet std::future::pending().await, um den Dienst am Laufen zu halten und auf D-Bus-Anfragen zu warten.  
  * **Nachbedingungen:** Der D-Bus-Dienst für die Portale läuft und ist bereit, Anfragen zu bearbeiten.  
  * **Fehlerfälle:** PortalsError::DBusConnectionFailed, PortalsError::DBusNameAcquisitionFailed, PortalsError::DBusInterfaceRegistrationFailed.  
* fn generate\_request\_handle(token\_prefix: \&str) \-\> String:  
  * Erzeugt einen eindeutigen Handle-String für Portal-Anfragen, typischerweise unter Verwendung eines Präfixes und einer UUID oder eines Zeitstempels. Beispiel: format\!("/org/freedesktop/portal/desktop/request/{}/{}", token\_prefix, uuid::Uuid::new\_v4().to\_string().replace('-', "")).

#### **5.5.3.2. Hilfsstrukturen (Beispiel)**

* pub enum UiPortalCommand {  
  * ShowOpenFile { request\_id: String, parent\_window: String, title: String, options: OpenFileOptions, response\_tx: tokio::sync::oneshot::Sender\<Result\<Vec\<String\>, PortalUiError\>\> },  
  * ShowSaveFile { request\_id: String, parent\_window: String, title: String, options: SaveFileOptions, response\_tx: tokio::sync::oneshot::Sender\<Result\<String, PortalUiError\>\> },  
  * //... weitere Befehle }  
* pub struct OpenFileOptions { /\* Felder entsprechend den D-Bus Optionen \*/ }  
* pub struct SaveFileOptions { /\* Felder entsprechend den D-Bus Optionen \*/ }  
* pub enum PortalUiError { CancelledByUser, InternalError(String) }  
* pub enum CompositorScreenshotCommand {  
  * TakeScreenshot { request\_id: String, interactive: bool, response\_tx: tokio::sync::oneshot::Sender\<Result\<String, CompositorError\>\> }, // String ist der URI  
  * PickColor { request\_id: String, response\_tx: tokio::sync::oneshot::Sender\<Result\<(f64, f64, f64), CompositorError\>\> }, }

### **5.5.4. Submodul: system::portals::error \- Fehlerbehandlung im Portals-Modul**

* **Datei:** system/portals/error.rs  
* **Zweck:** Definiert die spezifischen Fehlertypen für das system::portals-Modul.

#### **5.5.4.1. Enum PortalsError**

* \# pub enum PortalsError {  
  * \#  
    DBusConnectionFailed(\#\[from\] zbus::Error),  
  * \#  
    DBusNameAcquisitionFailed { service\_name: String, \#\[source\] source: zbus::Error },  
  * \#  
    DBusInterfaceRegistrationFailed { interface\_name: String, object\_path: String, \#\[source\] source: zbus::Error },  
  * \#\[error("Failed to send command to UI layer: {0}")\]  
    UiCommandSendError(String),  
  * \#\[error("Failed to send command to Compositor layer: {0}")\]  
    CompositorCommandSendError(String),  
  * \#\[error("UI interaction failed or was cancelled: {0}")\]  
    UiInteractionFailed(String),  
  * \#\[error("Compositor interaction failed: {0}")\]  
    CompositorInteractionFailed(String),  
  * \#\[error("Invalid options provided for portal request: {0}")\]  
    InvalidOptions(String),  
    }  
  * Die Verwendung von \#\[from\] für zbus::Error ermöglicht eine einfache Konvertierung von zbus-Fehlern.104

---

**Schlussfolgerung Systemschicht Teil 4/4**  
Mit der Spezifikation der Module system::audio, system::mcp und system::portals ist die detaillierte Ausarbeitung der Systemschicht abgeschlossen. Diese Module stellen kritische Schnittstellen zum Audiosystem, zur KI-Integration und zu Desktop-übergreifenden Diensten bereit. Die Implementierung gemäß dieser Ultra-Feinspezifikation wird eine robuste und gut integrierte Systemschicht gewährleisten, die als solide Grundlage für die darüberliegende Benutzeroberflächenschicht dient. Die konsequente Nutzung von Rust, PipeWire, D-Bus und etablierten Freedesktop-Standards sichert Modernität, Leistung und Kompatibilität. Die detaillierte Definition von Datenstrukturen, Methoden, Fehlerbehandlung und Interaktionsprotokollen minimiert Ambiguitäten und ermöglicht eine effiziente Implementierung.

#### **Referenzen**

1. PipeWire — multimedia in Rust // Lib.rs, Zugriff am Mai 14, 2025, [https://lib.rs/crates/pipewire](https://lib.rs/crates/pipewire)  
2. PipeWire: PipeWire, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/](https://docs.pipewire.org/)  
3. PipeWire API, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/page\_api.html](https://docs.pipewire.org/page_api.html)  
4. Crate pipewire \- Rust \- FreeDesktop.org, Zugriff am Mai 14, 2025, [https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/](https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/)  
5. Tutorial \- Part 2: Enumerating Objects \- PipeWire, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/1.4/page\_tutorial2.html](https://docs.pipewire.org/1.4/page_tutorial2.html)  
6. pw\_registry\_events Struct Reference \- PipeWire, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/structpw\_\_registry\_\_events.html](https://docs.pipewire.org/structpw__registry__events.html)  
7. Streams \- PipeWire, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/page\_streams.html](https://docs.pipewire.org/page_streams.html)  
8. Sending Messages to a Pipewire Node \- Frank's Reich, Zugriff am Mai 14, 2025, [https://franks-reich.net/posts/sending\_messages\_to\_pipewire/](https://franks-reich.net/posts/sending_messages_to_pipewire/)  
9. pipewire/Rust: How to list all Audio/Source nodes and those properties by using pipewire-rs, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/74793488/pipewire-rust-how-to-list-all-audio-source-nodes-and-those-properties-by-using](https://stackoverflow.com/questions/74793488/pipewire-rust-how-to-list-all-audio-source-nodes-and-those-properties-by-using)  
10. getting default audio/video input and ouput devices with rust and pipewire-rs, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/77379792/getting-default-audio-video-input-and-ouput-devices-with-rust-and-pipewire-rs](https://stackoverflow.com/questions/77379792/getting-default-audio-video-input-and-ouput-devices-with-rust-and-pipewire-rs)  
11. Native Protocol \- PipeWire, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/page\_native\_protocol.html](https://docs.pipewire.org/page_native_protocol.html)  
12. pipewire 0.8.0 \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/pipewire/latest/pipewire/](https://docs.rs/pipewire/latest/pipewire/)  
13. Zugriff am Januar 1, 1970, [https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/registry/struct.Registry.html](https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/registry/struct.Registry.html)  
14. Zugriff am Januar 1, 1970, [https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/struct.MainLoop.html](https://pipewire.pages.freedesktop.org/pipewire-rs/pipewire/struct.MainLoop.html)  
15. Zugriff am Januar 1, 1970, [https://docs.rs/pipewire/latest/pipewire/type.Node.html](https://docs.rs/pipewire/latest/pipewire/type.Node.html)  
16. Zugriff am Januar 1, 1970, [https://docs.rs/pipewire/latest/pipewire/struct.Proxy.html](https://docs.rs/pipewire/latest/pipewire/struct.Proxy.html)  
17. Zugriff am Januar 1, 1970, [https://github.com/pipewire-rs/pipewire-rs/tree/master/examples](https://github.com/pipewire-rs/pipewire-rs/tree/master/examples)  
18. Zugriff am Januar 1, 1970, [https://docs.rs/pipewire/latest/pipewire/registry/struct.Registry.html](https://docs.rs/pipewire/latest/pipewire/registry/struct.Registry.html)  
19. Zugriff am Januar 1, 1970, [https://docs.rs/pipewire/latest/pipewire/node/struct.Node.html](https://docs.rs/pipewire/latest/pipewire/node/struct.Node.html)  
20. Zugriff am Januar 1, 1970, [https://github.com/pipewire-rs/pipewire-rs/blob/master/examples/audio-capture.rs](https://github.com/pipewire-rs/pipewire-rs/blob/master/examples/audio-capture.rs)  
21. Zugriff am Januar 1, 1970, [https://github.com/pipewire-rs/pipewire-rs/blob/master/src/node.rs](https://github.com/pipewire-rs/pipewire-rs/blob/master/src/node.rs)  
22. Zugriff am Januar 1, 1970, [https://github.com/pipewire-rs/pipewire-rs/blob/master/examples/dump-objects.rs](https://github.com/pipewire-rs/pipewire-rs/blob/master/examples/dump-objects.rs)  
23. Zugriff am Januar 1, 1970, [https://github.com/pipewire-rs/pipewire-rs/blob/master/pipewire/src/node.rs](https://github.com/pipewire-rs/pipewire-rs/blob/master/pipewire/src/node.rs)  
24. How to Capture Audio Using Pipewire and Rust \- Eloy Coto, Zugriff am Mai 14, 2025, [https://acalustra.com/playing-with-pipewire-audio-streams-and-rust.html](https://acalustra.com/playing-with-pipewire-audio-streams-and-rust.html)  
25. \[ANN\] wiremix: A TUI audio mixer for PipeWire written in Rust \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/rust/comments/1kbyv7s/ann\_wiremix\_a\_tui\_audio\_mixer\_for\_pipewire/](https://www.reddit.com/r/rust/comments/1kbyv7s/ann_wiremix_a_tui_audio_mixer_for_pipewire/)  
26. PipeWire volume-change with pw-cli doesn't notify other programs \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/77953845/pipewire-volume-change-with-pw-cli-doesnt-notify-other-programs](https://stackoverflow.com/questions/77953845/pipewire-volume-change-with-pw-cli-doesnt-notify-other-programs)  
27. Node \- PipeWire, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/1.4/group\_\_spa\_\_node.html](https://docs.pipewire.org/1.4/group__spa__node.html)  
28. pipewire-props, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/page\_man\_pipewire-props\_7.html](https://docs.pipewire.org/page_man_pipewire-props_7.html)  
29. \[SOLVED\] Low volume using Pipewire / Multimedia and Games / Arch Linux Forums, Zugriff am Mai 14, 2025, [https://bbs.archlinux.org/viewtopic.php?id=287573](https://bbs.archlinux.org/viewtopic.php?id=287573)  
30. spa/examples/example-control.c \- PipeWire, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/spa\_2examples\_2example-control\_8c-example.html](https://docs.pipewire.org/spa_2examples_2example-control_8c-example.html)  
31. spa/param/route.h Source File \- PipeWire, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/devel/route\_8h\_source.html](https://docs.pipewire.org/devel/route_8h_source.html)  
32. spa/param/audio/raw.h File Reference \- PipeWire, Zugriff am Mai 14, 2025, [https://docs.pipewire.org/1.4/audio\_2raw\_8h.html](https://docs.pipewire.org/1.4/audio_2raw_8h.html)  
33. Multiple error types \- Rust By Example, Zugriff am Mai 14, 2025, [https://doc.rust-lang.org/rust-by-example/error/multiple\_error\_types.html](https://doc.rust-lang.org/rust-by-example/error/multiple_error_types.html)  
34. Error handling \- good/best practices : r/rust \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/rust/comments/1bb7dco/error\_handling\_goodbest\_practices/](https://www.reddit.com/r/rust/comments/1bb7dco/error_handling_goodbest_practices/)  
35. Simplify Error Handling in Rust with thiserror Crate \- w3resource, Zugriff am Mai 14, 2025, [https://www.w3resource.com/rust-tutorial/simplify-error-handling-rust-thiserror-crate.php](https://www.w3resource.com/rust-tutorial/simplify-error-handling-rust-thiserror-crate.php)  
36. smithay/anvil/README.md at master \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/Smithay/smithay/blob/master/anvil/README.md](https://github.com/Smithay/smithay/blob/master/anvil/README.md)  
37. mcp\_client\_rs \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/mcp\_client\_rs](https://docs.rs/mcp_client_rs)  
38. conikeec/mcpr: Model Context Protocol (MCP) implementation in Rust \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/conikeec/mcpr](https://github.com/conikeec/mcpr)  
39. mcp\_client\_rs \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/mcp\_client\_rs](https://crates.io/crates/mcp_client_rs)  
40. mcpr \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/mcpr](https://docs.rs/mcpr)  
41. 0xKoda/mcp-rust-docs: An MCP to retrieve rust crate documentation for LLM's \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/0xKoda/mcp-rust-docs](https://github.com/0xKoda/mcp-rust-docs)  
42. MCP Notify Server – A Model Context Protocol service that sends desktop notifications and alert sounds when AI agent tasks are completed, integrating with various LLM clients like Claude Desktop and Cursor. \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/mcp/comments/1jd6j7x/mcp\_notify\_server\_a\_model\_context\_protocol/](https://www.reddit.com/r/mcp/comments/1jd6j7x/mcp_notify_server_a_model_context_protocol/)  
43. An MCP server for the github notifications API for the OSS maintainer, Zugriff am Mai 14, 2025, [https://github.com/mcollina/mcp-github-notifications](https://github.com/mcollina/mcp-github-notifications)  
44. Notifications \- Laravel 11.x \- The PHP Framework For Web Artisans, Zugriff am Mai 14, 2025, [https://laravel.com/docs/11.x/notifications](https://laravel.com/docs/11.x/notifications)  
45. React Native Android Notification Listener \- Listen for status bar notifications from all applications \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/leandrosimoes/react-native-android-notification-listener](https://github.com/leandrosimoes/react-native-android-notification-listener)  
46. Building a Server-Sent Events (SSE) MCP Server with FastAPI \- Ragie, Zugriff am Mai 14, 2025, [https://www.ragie.ai/blog/building-a-server-sent-events-sse-mcp-server-with-fastapi](https://www.ragie.ai/blog/building-a-server-sent-events-sse-mcp-server-with-fastapi)  
47. How would I create an asynchronous notification system using RESTful web services?, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/1093900/how-would-i-create-an-asynchronous-notification-system-using-restful-web-service](https://stackoverflow.com/questions/1093900/how-would-i-create-an-asynchronous-notification-system-using-restful-web-service)  
48. mcp\_client\_rs \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/mcp\_client\_rs/0.1.1](https://crates.io/crates/mcp_client_rs/0.1.1)  
49. mcp\_client\_rs \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/mcp\_client\_rs/0.1.4](https://crates.io/crates/mcp_client_rs/0.1.4)  
50. darinkishore/mcp\_client\_rust \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/darinkishore/mcp\_client\_rust](https://github.com/darinkishore/mcp_client_rust)  
51. README.md \- Model Context Protocol (MCP) Rust SDK \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/darinkishore/mcp\_client\_rust/blob/main/README.md](https://github.com/darinkishore/mcp_client_rust/blob/main/README.md)  
52. mcp\_client\_rs \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/mcp\_client\_rs/latest/mcp\_client\_rs/](https://docs.rs/mcp_client_rs/latest/mcp_client_rs/)  
53. Zugriff am Januar 1, 1970, [https://docs.rs/mcp\_client\_rs/latest/mcp\_client\_rs/client/struct.Client.html](https://docs.rs/mcp_client_rs/latest/mcp_client_rs/client/struct.Client.html)  
54. Zugriff am Januar 1, 1970, [https://github.com/darinkishore/mcp\_client\_rust/tree/main/examples](https://github.com/darinkishore/mcp_client_rust/tree/main/examples)  
55. Zugriff am Januar 1, 1970, [https://github.com/darinkishore/mcp\_client\_rust/blob/main/src/client.rs](https://github.com/darinkishore/mcp_client_rust/blob/main/src/client.rs)  
56. Zugriff am Januar 1, 1970, [https://github.com/darinkishore/mcp\_client\_rust/blob/main/src/transport/stdio.rs](https://github.com/darinkishore/mcp_client_rust/blob/main/src/transport/stdio.rs)  
57. Zugriff am Januar 1, 1970, [https://raw.githubusercontent.com/darinkishore/mcp\_client\_rust/main/src/client.rs](https://raw.githubusercontent.com/darinkishore/mcp_client_rust/main/src/client.rs)  
58. Zugriff am Januar 1, 1970, [https://raw.githubusercontent.com/darinkishore/mcp\_client\_rust/main/src/transport/stdio.rs](https://raw.githubusercontent.com/darinkishore/mcp_client_rust/main/src/transport/stdio.rs)  
59. EventStream in rocket::response::stream \- Rust, Zugriff am Mai 14, 2025, [https://api.rocket.rs/v0.5/rocket/response/stream/struct.EventStream](https://api.rocket.rs/v0.5/rocket/response/stream/struct.EventStream)  
60. xdg-portal \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/xdg-portal](https://crates.io/crates/xdg-portal)  
61. We have made a xdg-desktop-portal which supports the remote of xdg-desktop-portal : r/swaywm \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/swaywm/comments/1kjabld/we\_have\_made\_a\_xdgdesktopportal\_which\_supports/](https://www.reddit.com/r/swaywm/comments/1kjabld/we_have_made_a_xdgdesktopportal_which_supports/)  
62. git: bf9c9f5197f2 \- main \- x11/xdg-desktop-portal-luminous: update to 0.1.10, Zugriff am Mai 14, 2025, [https://lists.freebsd.org/archives/dev-commits-ports-all/2025-May/159001.html](https://lists.freebsd.org/archives/dev-commits-ports-all/2025-May/159001.html)  
63. Can't run cosmic-screenshot in 24.04 · Issue \#75 \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/pop-os/cosmic-screenshot/issues/75](https://github.com/pop-os/cosmic-screenshot/issues/75)  
64. How to take a screenshot with QtDBus via org.freedesktop.portal? \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/74213740/how-to-take-a-screenshot-with-qtdbus-via-org-freedesktop-portal](https://stackoverflow.com/questions/74213740/how-to-take-a-screenshot-with-qtdbus-via-org-freedesktop-portal)  
65. xdg-desktop-portal-1.20.0 \- Linux From Scratch\!, Zugriff am Mai 14, 2025, [https://www.linuxfromscratch.org/blfs/view/12.3/x/xdg-desktop-portal.html](https://www.linuxfromscratch.org/blfs/view/12.3/x/xdg-desktop-portal.html)  
66. File Chooser \- XDG Desktop Portal documentation \- Flatpak, Zugriff am Mai 14, 2025, [https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.FileChooser.html](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.FileChooser.html)  
67. Backend D-Bus Interfaces \- XDG Desktop Portal documentation \- Flatpak, Zugriff am Mai 14, 2025, [https://flatpak.github.io/xdg-desktop-portal/docs/impl-dbus-interfaces.html](https://flatpak.github.io/xdg-desktop-portal/docs/impl-dbus-interfaces.html)  
68. XDG Desktop Portal \- Flatpak, Zugriff am Mai 14, 2025, [https://flatpak.github.io/xdg-desktop-portal/](https://flatpak.github.io/xdg-desktop-portal/)  
69. node-web-audio-api \- NPM, Zugriff am Mai 14, 2025, [https://www.npmjs.com/package/node-web-audio-api](https://www.npmjs.com/package/node-web-audio-api)  
70. XDG Desktop Portal \- ArchWiki, Zugriff am Mai 14, 2025, [https://wiki.archlinux.org/title/XDG\_Desktop\_Portal](https://wiki.archlinux.org/title/XDG_Desktop_Portal)  
71. Interacting with System Services using DBus Dart \- Aadarsha Dhakal, Zugriff am Mai 14, 2025, [https://blog.aadarshadhakal.com.np/interacting-with-system-services-using-dbus-dart](https://blog.aadarshadhakal.com.np/interacting-with-system-services-using-dbus-dart)  
72. Taking screenshots with Java under Wayland \- adangel.org, Zugriff am Mai 14, 2025, [https://adangel.org/2022/02/06/java-screenshot/](https://adangel.org/2022/02/06/java-screenshot/)  
73. xdg-desktop-portal/NEWS.md at main \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/flatpak/xdg-desktop-portal/blob/main/NEWS.md](https://github.com/flatpak/xdg-desktop-portal/blob/main/NEWS.md)  
74. Add API to know whether a certain portal is really available · Issue \#686 · flatpak/xdg-desktop-portal \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/flatpak/xdg-desktop-portal/issues/686](https://github.com/flatpak/xdg-desktop-portal/issues/686)  
75. Bug List \- freedesktop.org Bugzilla, Zugriff am Mai 14, 2025, [https://bugs.freedesktop.org/buglist.cgi?limit=0\&query\_format=advanced\&resolution=FIXED\&order=assigned\_to%2Cshort\_desc%2Cchangeddate%2Cproduct%20DESC%2Ccomponent%2Cresolution%20DESC%2Cbug\_id%20DESC\&query\_based\_on=](https://bugs.freedesktop.org/buglist.cgi?limit=0&query_format=advanced&resolution=FIXED&order=assigned_to,short_desc,changeddate,product+DESC,component,resolution+DESC,bug_id+DESC&query_based_on)  
76. Attachment 145572 Details for Bug 111848 – package log output \- freedesktop.org Bugzilla, Zugriff am Mai 14, 2025, [https://bugs.freedesktop.org/attachment.cgi?id=145572\&action=edit](https://bugs.freedesktop.org/attachment.cgi?id=145572&action=edit)  
77. Screenshot \- XDG Desktop Portal documentation \- Flatpak, Zugriff am Mai 14, 2025, [https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Screenshot.html)  
78. xdg-desktop-portal/data/org.freedesktop.portal.Screenshot.xml at main \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/flatpak/xdg-desktop-portal/blob/master/data/org.freedesktop.portal.Screenshot.xml](https://github.com/flatpak/xdg-desktop-portal/blob/master/data/org.freedesktop.portal.Screenshot.xml)  
79. D-Bus Specification, Zugriff am Mai 14, 2025, [https://dbus.freedesktop.org/doc/dbus-specification.html](https://dbus.freedesktop.org/doc/dbus-specification.html)  
80. waycrate/xdg-desktop-portal-luminous: A xdg-desktop ... \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/waycrate/xdg-desktop-portal-luminous](https://github.com/waycrate/xdg-desktop-portal-luminous)  
81. Zugriff am Januar 1, 1970, [https://github.com/flatpak/xdg-desktop-portal/tree/main/src](https://github.com/flatpak/xdg-desktop-portal/tree/main/src)  
82. Zugriff am Januar 1, 1970, [https://flatpak.github.io/xdg-desktop-portal/portal-docs.html](https://flatpak.github.io/xdg-desktop-portal/portal-docs.html)  
83. zbus \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/](https://docs.rs/zbus/latest/zbus/)  
84. How to set interface dynamically using zbus \- The Rust Programming Language Forum, Zugriff am Mai 14, 2025, [https://users.rust-lang.org/t/how-to-set-interface-dynamically-using-zbus/108691](https://users.rust-lang.org/t/how-to-set-interface-dynamically-using-zbus/108691)  
85. zbus \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/zbus/5.6.0](https://crates.io/crates/zbus/5.6.0)  
86. Writing a client proxy \- zbus: D-Bus for Rust made easy \- GitHub Pages, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/client.html](https://dbus2.github.io/zbus/client.html)  
87. proxy in zbus \- Rust \- openrr.github.io, Zugriff am Mai 14, 2025, [https://openrr.github.io/openrr/zbus/attr.proxy.html](https://openrr.github.io/openrr/zbus/attr.proxy.html)  
88. Event in winit::event \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/winit-gtk/latest/winit/event/enum.Event.html](https://docs.rs/winit-gtk/latest/winit/event/enum.Event.html)  
89. Smithay/calloop-wayland-source \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/Smithay/calloop-wayland-source](https://github.com/Smithay/calloop-wayland-source)  
90. wayland\_client \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/wayland-client/](https://docs.rs/wayland-client/)  
91. smithay::desktop \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/desktop/index.html](https://smithay.github.io/smithay/smithay/desktop/index.html)  
92. "Connection" Search \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/?search=Connection](https://docs.rs/zbus/latest/zbus/?search=Connection)  
93. Introduction \- zbus: D-Bus for Rust made easy, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/](https://dbus2.github.io/zbus/)  
94. zbus \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/zbus/3.15.2](https://crates.io/crates/zbus/3.15.2)  
95. zbus \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/zbus](https://crates.io/crates/zbus)  
96. Writing a service interface \- zbus: D-Bus for Rust made easy, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/server.html](https://dbus2.github.io/zbus/server.html)  
97. Some D-Bus concepts \- zbus: D-Bus for Rust made easy, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/concepts.html](https://dbus2.github.io/zbus/concepts.html)  
98. zbus \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/zbus/2.0.0](https://crates.io/crates/zbus/2.0.0)  
99. Builder in zbus::connection \- Rust \- openrr.github.io, Zugriff am Mai 14, 2025, [https://openrr.github.io/openrr/zbus/connection/struct.Builder.html](https://openrr.github.io/openrr/zbus/connection/struct.Builder.html)  
100. Zugriff am Januar 1, 1970, [https://docs.rs/zbus/latest/zbus/struct.ObjectServer.html](https://docs.rs/zbus/latest/zbus/struct.ObjectServer.html)  
101. Zugriff am Januar 1, 1970, [https://docs.rs/zbus/latest/zbus/server/struct.ObjectServer.html](https://docs.rs/zbus/latest/zbus/server/struct.ObjectServer.html)  
102. Zugriff am Januar 1, 1970, [https://dbus2.github.io/zbus/async\_server.html](https://dbus2.github.io/zbus/async_server.html)  
103. Zugriff am Januar 1, 1970, [https://github.com/dbus2/zbus/blob/main/examples/object\_server.rs](https://github.com/dbus2/zbus/blob/main/examples/object_server.rs)  
104. Error in zbus \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/enum.Error.html](https://docs.rs/zbus/latest/zbus/enum.Error.html)