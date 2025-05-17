# **Implementierungsleitfaden Systemschicht: D-Bus-Interaktion und Output-Management (Teil 2/4)**

Dieser Teil des Implementierungsleitfadens für die Systemschicht befasst sich mit zwei zentralen Aspekten der neuen Linux-Desktop-Umgebung: der Interaktion mit systemweiten D-Bus-Diensten und der umfassenden Verwaltung von Anzeigeausgängen. Diese Komponenten sind entscheidend für die Integration der Desktop-Umgebung in das Basissystem und für die Bereitstellung einer kohärenten Benutzererfahrung über verschiedene Hardwarekonfigurationen hinweg.

## **A. Modul: system::dbus – Interaktion mit System-D-Bus-Diensten**

Das Modul system::dbus ist verantwortlich für die Kommunikation mit verschiedenen Standard-D-Bus-Diensten, die für den Betrieb einer Desktop-Umgebung unerlässlich sind. Hierzu zählen Dienste für Energieverwaltung (UPower), Sitzungsmanagement (systemd-logind), Netzwerkmanagement (NetworkManager), Geheimnisverwaltung (Freedesktop Secret Service) und Rechteverwaltung (PolicyKit). Die Implementierung erfolgt unter Verwendung der zbus-Bibliothek.1

### **1\. Submodul: system::dbus::error – Fehlerbehandlung für D-Bus-Operationen**

Dieses Submodul definiert die spezifischen Fehlertypen für alle D-Bus-Interaktionen innerhalb der Systemschicht. Gemäß den Entwicklungsrichtlinien wird hierfür das thiserror-Crate genutzt, um pro Modul ein dediziertes Error-Enum zu erstellen \[User Query IV.4.3\]. Dies ermöglicht eine präzise Fehlerbehandlung und klare Fehlermeldungen.

* **Datei**: system/dbus/error.rs  
* **Spezifikation**:  
  * Es wird ein öffentliches Enum DBusError definiert, das die Traits thiserror::Error und std::fmt::Debug implementiert. Die Verwendung von thiserror vereinfacht die Erstellung idiomatischer Fehler.2  
  * Die \#\[from\]-Direktive von thiserror wird verwendet, um Fehler aus der zbus-Bibliothek (insbesondere zbus::Error 4 und zbus::zvariant::Error) transparent in spezifische Varianten von DBusError zu konvertieren. Dies ist entscheidend, da zbus-Operationen wie Verbindungsaufbau, Methodenaufrufe oder Signalabonnements fehlschlagen können.1  
  * **Varianten der DBusError Enum**:  
    * \# ConnectionFailed { service\_name: Option\<String\>, bus: BusType, \#\[source\] source: zbus::Error } Fehler beim Aufbau einer D-Bus-Verbindung. BusType ist ein Enum (Session, System).  
    * \# MethodCallFailed { service: String, path: String, interface: String, method: String, \#\[source\] source: zbus::Error } Fehler beim Aufruf einer D-Bus-Methode.  
    * \# ProxyCreationFailed { service: String, interface: String, \#\[source\] source: zbus::Error } Fehler bei der Erstellung eines D-Bus-Proxys.  
    * \# SignalSubscriptionFailed { interface: String, signal\_name: String, \#\[source\] source: zbus::Error } Fehler beim Abonnieren eines D-Bus-Signals.  
    * \# InvalidResponse { service: String, method: String, details: String } Unerwartete oder ungültige Antwort von einem D-Bus-Dienst.  
    * \# DataDeserializationError { context: String, \#\[source\] source: zbus::zvariant::Error } Fehler bei der Deserialisierung von D-Bus-Daten.  
    * \# PropertyAccessFailed { service: String, interface: String, property: String, \#\[source\] source: zbus::Error } Fehler beim Zugriff auf eine D-Bus-Eigenschaft.  
    * \# NameTaken { name: String, \#\[source\] source: zbus::Error } Tritt auf, wenn versucht wird, einen D-Bus-Namen zu beanspruchen, der bereits belegt ist (relevant für das Anbieten eigener D-Bus-Dienste, hier primär für Clients).  
    * \#\[error("Operation timed out: {operation}")\] Timeout { operation: String } Zeitüberschreitung bei einer D-Bus-Operation.  
* **Implementierungsschritte**:  
  1. Definition des BusType Enums: pub enum BusType { Session, System }.  
  2. Definition des DBusError Enums mit den oben genannten Varianten und den \#\[error(...)\]-Attributen für menschenlesbare Fehlermeldungen.  
  3. Sicherstellung, dass alle öffentlichen Funktionen im system::dbus-Modul und seinen Submodulen Result\<T, DBusError\> zurückgeben, um eine konsistente Fehlerbehandlung zu gewährleisten.

### **2\. Submodul: system::dbus::connection – D-Bus Verbindungsmanagement**

Dieses Submodul stellt einen zentralen Manager für D-Bus-Verbindungen bereit, um die Wiederverwendung von Verbindungen zu ermöglichen und deren Aufbau zu optimieren.

* **Datei**: system/dbus/connection.rs  
* **Spezifikation**:  
  * **Struktur**: DBusConnectionManager  
    * Felder:  
      * session\_bus: tokio::sync::OnceCell\<Arc\<zbus::Connection\>\>  
      * system\_bus: tokio::sync::OnceCell\<Arc\<zbus::Connection\>\> Die Verwendung von tokio::sync::OnceCell ermöglicht eine verzögerte Initialisierung der D-Bus-Verbindungen. Eine Verbindung wird erst beim ersten tatsächlichen Bedarf aufgebaut. Anschließend wird die Arc\<zbus::Connection\> für die zukünftige Wiederverwendung gespeichert.5 Dies ist effizient, da nicht bei jedem Start des Desktops sofort alle potenziellen D-Bus-Verbindungen etabliert werden müssen, und Arc stellt sicher, dass die einmal aufgebaute Verbindung sicher zwischen verschiedenen asynchronen Tasks geteilt werden kann, die möglicherweise parallel auf denselben Bus zugreifen (z.B. UPower-Client und Logind-Client auf dem Systembus).  
  * **Methoden** für DBusConnectionManager:  
    * pub fn new() \-\> Self: Konstruktor, initialisiert die leeren OnceCells.  
    * pub async fn get\_session\_bus(\&self) \-\> Result\<Arc\<zbus::Connection\>, DBusError\>: Gibt eine Arc-gekapselte zbus::Connection zum Session-Bus zurück. Nutzt self.session\_bus.get\_or\_try\_init() in Kombination mit zbus::Connection::session().await.1 Fehler beim Verbindungsaufbau werden in DBusError::ConnectionFailed gemappt.  
    * pub async fn get\_system\_bus(\&self) \-\> Result\<Arc\<zbus::Connection\>, DBusError\>: Analog zu get\_session\_bus, jedoch für den System-Bus unter Verwendung von zbus::Connection::system().await.1  
* **Implementierungsschritte**:  
  1. Definiere die DBusConnectionManager-Struktur.  
  2. Implementiere die new()-Methode.  
  3. Implementiere get\_session\_bus():  
     Rust  
     pub async fn get\_session\_bus(\&self) \-\> Result\<Arc\<zbus::Connection\>, DBusError\> {  
         self.session\_bus  
            .get\_or\_try\_init(|| async {  
                 zbus::Connection::session()  
                    .await  
                    .map(Arc::new)  
                    .map\_err(|e| DBusError::ConnectionFailed {  
                         service\_name: None, // Generic session bus connection  
                         bus: BusType::Session,  
                         source: e,  
                     })  
             })  
            .await  
            .cloned() // Clone the Arc for the caller  
     }

  4. Implementiere get\_system\_bus() analog.

### **3\. Submodul: system::dbus::upower\_client – UPower D-Bus Client**

Dieser Client interagiert mit dem org.freedesktop.UPower-Dienst, um Informationen über den Energiezustand des Systems und angeschlossene Geräte zu erhalten.6

* **Dateien**: system/dbus/upower\_client.rs, system/dbus/upower\_types.rs  
* **Spezifikation (upower\_types.rs)**:  
  * pub enum PowerDeviceType { Unknown \= 0, LinePower \= 1, Battery \= 2, Ups \= 3, Monitor \= 4, Mouse \= 5, Keyboard \= 6, Pda \= 7, Phone \= 8, /\* Display \= 9 (aus UPower.Device, nicht standardisiert in udev?) \*/ } (Werte basierend auf UPowerDeviceType in der UPower-Dokumentation).  
  * pub enum PowerState { Unknown \= 0, Charging \= 1, Discharging \= 2, Empty \= 3, FullyCharged \= 4, PendingCharge \= 5, PendingDischarge \= 6 }.8  
  * pub enum PowerWarningLevel { Unknown \= 0, None \= 1, Discharging \= 2, Low \= 3, Critical \= 4, Action \= 5 }.  
  * pub struct PowerDeviceDetails { pub object\_path: zbus::zvariant::OwnedObjectPath, pub vendor: String, pub model: String, pub serial: String, pub native\_path: String, pub device\_type: PowerDeviceType, pub state: PowerState, pub percentage: f64, pub temperature: f64, pub voltage: f64, pub energy: f64, pub energy\_empty: f64, pub energy\_full: f64, pub energy\_full\_design: f64, pub energy\_rate: f64, pub time\_to\_empty: i64, pub time\_to\_full: i64, pub is\_rechargeable: bool, pub is\_present: bool, pub warning\_level: PowerWarningLevel, pub icon\_name: String, pub capacity: f64, pub technology: String }.7  
    * Felder werden aus den Properties des org.freedesktop.UPower.Device-Interfaces abgeleitet.  
  * pub struct UPowerProperties { pub on\_battery: bool, pub lid\_is\_closed: bool, pub lid\_is\_present: bool, pub daemon\_version: String }.7  
  * Implementiere TryFrom\<u32\> für PowerDeviceType, PowerState, PowerWarningLevel zur Konvertierung von D-Bus-Werten.  
* **Spezifikation (upower\_client.rs)**:  
  * **Proxy-Definitionen** (mittels \#\[zbus::proxy(...)\] 1):  
    * UPowerManagerProxy (Name angepasst zur Klarheit) für org.freedesktop.UPower auf /org/freedesktop/UPower.  
      * Methoden:  
        * \# async fn enumerate\_devices(\&self) \-\> zbus::Result\<Vec\<zbus::zvariant::OwnedObjectPath\>\>; 7  
        * \# async fn get\_display\_device(\&self) \-\> zbus::Result\<zbus::zvariant::OwnedObjectPath\>\>; 7  
        * \#\[zbus(name \= "GetCriticalAction")\] async fn get\_critical\_action(\&self) \-\> zbus::Result\<String\>\>; 7  
      * Properties (als Methoden im Proxy generiert):  
        * \#\[zbus(property)\] async fn on\_battery(\&self) \-\> zbus::Result\<bool\>; 7  
        * \#\[zbus(property)\] async fn lid\_is\_closed(\&self) \-\> zbus::Result\<bool\>; 7  
        * \#\[zbus(property)\] async fn lid\_is\_present(\&self) \-\> zbus::Result\<bool\>; 7  
        * \#\[zbus(property)\] async fn daemon\_version(\&self) \-\> zbus::Result\<String\>; 7  
      * Signale (als Methoden im Proxy generiert, die einen SignalStream zurückgeben):  
        * \#\[zbus(signal)\] async fn device\_added(\&self, device\_path: zbus::zvariant::OwnedObjectPath) \-\> zbus::Result\<()\>; (Das Signal selbst hat Argumente, die receive\_ Methode wird diese liefern) 7  
        * \#\[zbus(signal)\] async fn device\_removed(\&self, device\_path: zbus::zvariant::OwnedObjectPath) \-\> zbus::Result\<()\>; 7  
        * (Das PropertiesChanged-Signal wird über zbus::Proxy::receive\_properties\_changed\_with\_args() oder ähnliche Methoden des generierten Proxys gehandhabt).  
    * UPowerDeviceProxy für org.freedesktop.UPower.Device (Pfad variabel, daher default\_path nicht im Makro).  
      * Properties (Beispiele):  
        * \#\[zbus(property)\] async fn type\_(\&self) \-\> zbus::Result\<u32\>; (Suffix \_ um Keyword-Kollision zu vermeiden)  
        * \#\[zbus(property)\] async fn state(\&self) \-\> zbus::Result\<u32\>;  
        * \#\[zbus(property)\] async fn percentage(\&self) \-\> zbus::Result\<f64\>;  
        * \#\[zbus(property)\] async fn time\_to\_empty(\&self) \-\> zbus::Result\<i64\>; 8  
        * \#\[zbus(property)\] async fn time\_to\_full(\&self) \-\> zbus::Result\<i64\>; 8  
        * \#\[zbus(property, name \= "IsPresent")\] async fn is\_present(\&self) \-\> zbus::Result\<bool\>;  
        * \#\[zbus(property, name \= "IconName")\] async fn icon\_name(\&self) \-\> zbus::Result\<String\>;  
        * (Weitere Properties analog definieren: Vendor, Model, Serial, NativePath, Temperature, Voltage, Energy, EnergyEmpty, EnergyFull, EnergyFullDesign, EnergyRate, IsRechargeable, WarningLevel, Capacity, Technology).  
  * **Struktur**: UPowerClient  
    * Felder: connection\_manager: Arc\<DBusConnectionManager\>, manager\_proxy\_path: Arc\<zbus::zvariant::ObjectPath\<'static\>\> (Cache für den Manager-Pfad).  
  * **Methoden** für UPowerClient:  
    * pub async fn new(conn\_manager: Arc\<DBusConnectionManager\>) \-\> Result\<Self, DBusError\>: Initialisiert den Client. Speichert den conn\_manager. Der manager\_proxy\_path wird auf /org/freedesktop/UPower gesetzt.  
    * async fn get\_manager\_proxy(\&self) \-\> Result\<UPowerManagerProxy\<'\_\>, DBusError\>: Private Hilfsmethode, um den UPowerManagerProxy zu erstellen. Holt die Systembus-Verbindung vom connection\_manager.  
    * async fn get\_device\_proxy\<'a\>(\&self, device\_path: &'a zbus::zvariant::ObjectPath\<'\_\>) \-\> Result\<UPowerDeviceProxy\<'a\>, DBusError\>: Private Hilfsmethode, um einen UPowerDeviceProxy für einen gegebenen Pfad zu erstellen.  
    * pub async fn get\_properties(\&self) \-\> Result\<UPowerProperties, DBusError\>: Ruft die on\_battery, lid\_is\_closed, lid\_is\_present und daemon\_version Properties vom UPowerManagerProxy ab und fasst sie in UPowerProperties zusammen.  
    * pub async fn enumerate\_devices(\&self) \-\> Result\<Vec\<zbus::zvariant::OwnedObjectPath\>, DBusError\>: Ruft UPowerManagerProxy::enumerate\_devices() auf.  
    * pub async fn get\_display\_device\_path(\&self) \-\> Result\<zbus::zvariant::OwnedObjectPath, DBusError\>: Ruft UPowerManagerProxy::get\_display\_device() auf.  
    * pub async fn get\_device\_details(\&self, device\_path: \&zbus::zvariant::ObjectPath\<'\_\>) \-\> Result\<PowerDeviceDetails, DBusError\>: Erstellt einen UPowerDeviceProxy für den device\_path. Ruft alle relevanten Properties ab und konvertiert sie in die PowerDeviceDetails-Struktur. Nutzt try\_into() für Enums.  
    * pub async fn on\_battery(\&self) \-\> Result\<bool, DBusError\>: Ruft die on\_battery Property vom UPowerManagerProxy ab.  
    * pub async fn subscribe\_device\_added(\&self) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<zbus::zvariant::OwnedObjectPath, DBusError\>\>, DBusError\>: Erstellt einen UPowerManagerProxy, ruft receive\_device\_added().await? auf.1 Mappt die Signaldaten ((OwnedObjectPath,)) und Fehler.  
    * pub async fn subscribe\_device\_removed(\&self) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<zbus::zvariant::OwnedObjectPath, DBusError\>\>, DBusError\>: Analog zu subscribe\_device\_added.  
    * pub async fn subscribe\_upower\_properties\_changed(\&self) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<HashMap\<String, zbus::zvariant::OwnedValue\>, DBusError\>\>, DBusError\>: Verwendet UPowerManagerProxy::receive\_properties\_changed().await?.  
    * pub async fn subscribe\_device\_properties\_changed(\&self, device\_path: zbus::zvariant::OwnedObjectPath) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<(String, HashMap\<String, zbus::zvariant::OwnedValue\>, Vec\<String\>), DBusError\>\>, DBusError\>: Erstellt einen UPowerDeviceProxy für den Pfad und verwendet receive\_properties\_changed\_with\_args().await?. Die Argumente des Signals sind (String, HashMap\<String, Value\>, Vec\<String\>).  
* **Implementierungsschritte**:  
  1. Definition der Typen in upower\_types.rs inklusive TryFrom\<u32\> für Enums.  
  2. Generierung der Proxy-Traits in upower\_client.rs.  
  3. Implementierung der UPowerClient-Struktur und ihrer Methoden. Die Methoden sollten die Proxy-Aufrufe kapseln und Fehler in DBusError umwandeln.  
  4. Signal-Abonnementmethoden geben einen Stream zurück, den der Aufrufer verarbeiten kann. Die Verarbeitung der Signaldaten (z.B. Extrahieren des device\_path aus dem Signal-Message-Body) und Fehlerbehandlung muss sorgfältig erfolgen.  
* **Publisher/Subscriber**:  
  * Publisher: org.freedesktop.UPower D-Bus Dienst.  
  * Subscriber: UPowerClient (bzw. die Systemschicht, die diesen Client nutzt).  
* Die Notwendigkeit, Signal-Streams korrekt zu verwalten, um Ressourcenlecks oder Callbacks auf ungültige Zustände zu vermeiden, ist ein wichtiger Aspekt. Wenn ein UPowerClient nicht mehr benötigt wird oder die Verbindung abbricht, müssen die assoziierten Streams ebenfalls beendet werden. Dies kann durch tokio::select\! in Kombination mit einem Shutdown-Signal oder durch das Droppen des Streams geschehen.

### **4\. Submodul: system::dbus::logind\_client – Systemd-Logind D-Bus Client**

Dieser Client interagiert mit org.freedesktop.login1 für Sitzungsmanagement, Sperr-/Entsperr-Operationen und Benachrichtigungen über Systemzustandsänderungen wie Suspend/Resume.10

* **Dateien**: system/dbus/logind\_client.rs, system/dbus/logind\_types.rs  
* **Spezifikation (logind\_types.rs)**:  
  * pub struct SessionInfo { pub id: String, pub user\_id: u32, pub user\_name: String, pub seat\_id: String, pub object\_path: zbus::zvariant::OwnedObjectPath }.10  
  * pub struct UserInfo { pub id: u32, pub name: String, pub object\_path: zbus::zvariant::OwnedObjectPath }.  
  * pub enum SessionState { Active, Online, Closing, Gone, Unknown } (basierend auf typischen Logind-Zuständen).  
* **Spezifikation (logind\_client.rs)**:  
  * **Proxy-Definitionen**:  
    * LogindManagerProxy für org.freedesktop.login1.Manager auf /org/freedesktop/login1.  
      * Methoden:  
        * \# async fn get\_session(\&self, session\_id: \&str) \-\> zbus::Result\<zbus::zvariant::OwnedObjectPath\>; 10  
        * \# async fn get\_session\_by\_pid(\&self, pid: u32) \-\> zbus::Result\<zbus::zvariant::OwnedObjectPath\>; 11  
        * \#\[zbus(name \= "GetUser")\] async fn get\_user(\&self, uid: u32) \-\> zbus::Result\<zbus::zvariant::OwnedObjectPath\>; 10  
        * \# async fn list\_sessions(\&self) \-\> zbus::Result\<Vec\<(String, u32, String, String, zbus::zvariant::OwnedObjectPath)\>\>; 10  
        * \# async fn lock\_session(\&self, session\_id: \&str) \-\> zbus::Result\<()\>; 10  
        * \# async fn unlock\_session(\&self, session\_id: \&str) \-\> zbus::Result\<()\>; 10  
        * \# async fn lock\_sessions(\&self) \-\> zbus::Result\<()\>; 10  
        * \# async fn unlock\_sessions(\&self) \-\> zbus::Result\<()\>; 10  
      * Signale:  
        * \#\[zbus(signal)\] async fn session\_new(\&self, session\_id: String, object\_path: zbus::zvariant::OwnedObjectPath) \-\> zbus::Result\<()\>; 12  
        * \#\[zbus(signal)\] async fn session\_removed(\&self, session\_id: String, object\_path: zbus::zvariant::OwnedObjectPath) \-\> zbus::Result\<()\>; 12  
        * \# async fn prepare\_for\_sleep(\&self, start\_or\_stop: bool) \-\> zbus::Result\<()\>; 10  
    * LogindSessionProxy für org.freedesktop.login1.Session (Pfad variabel).  
      * Methoden:  
        * \#\[zbus(name \= "Lock")\] async fn lock(\&self) \-\> zbus::Result\<()\>; 10  
        * \#\[zbus(name \= "Unlock")\] async fn unlock(\&self) \-\> zbus::Result\<()\>; 10  
        * \# async fn terminate(\&self) \-\> zbus::Result\<()\>; 10  
      * Properties (Beispiele): Id: String, User: (u32, zbus::zvariant::OwnedObjectPath), Name: String, Timestamp: u64, TimestampMonotonic: u64, VTNr: u32, Seat: (String, zbus::zvariant::OwnedObjectPath), TTY: String, Remote: bool, RemoteHost: String, Service: String, Scope: String, Leader: u32, Audit: u32, Type: String, Class: String, Active: bool, State: String, IdleHint: bool, IdleSinceHint: u64, IdleSinceHintMonotonic: u64.  
      * Signale:  
        * \#\[zbus(signal, name \= "Lock")\] async fn lock\_signal(\&self) \-\> zbus::Result\<()\>; 10  
        * \#\[zbus(signal, name \= "Unlock")\] async fn unlock\_signal(\&self) \-\> zbus::Result\<()\>; 10  
        * \#\[zbus(signal, name \= "PropertyChanged")\] async fn property\_changed\_signal(\&self, name: String, value: zbus::zvariant::OwnedValue) \-\> zbus::Result\<()\>; (Standard-Signal)  
    * LogindUserProxy für org.freedesktop.login1.User (Pfad variabel).  
      * Methoden:  
        * \# async fn terminate(\&self) \-\> zbus::Result\<()\>; 10  
      * Properties (Beispiele): UID: u32, GID: u32, Name: String, Timestamp: u64, TimestampMonotonic: u64, RuntimePath: String, Service: String, Slice: String, Display: (String, zbus::zvariant::OwnedObjectPath), State: String, Sessions: Vec\<(String, zbus::zvariant::OwnedObjectPath)\>, IdleHint: bool, IdleSinceHint: u64, IdleSinceHintMonotonic: u64, Linger: bool.  
  * **Struktur**: LogindClient  
    * Felder: connection\_manager: Arc\<DBusConnectionManager\>, manager\_proxy\_path: Arc\<zbus::zvariant::ObjectPath\<'static\>\>.  
  * **Methoden** für LogindClient:  
    * pub async fn new(conn\_manager: Arc\<DBusConnectionManager\>) \-\> Result\<Self, DBusError\>  
    * async fn get\_manager\_proxy(\&self) \-\> Result\<LogindManagerProxy\<'\_\>, DBusError\>  
    * async fn get\_session\_proxy\<'a\>(\&self, session\_path: &'a zbus::zvariant::ObjectPath\<'\_\>) \-\> Result\<LogindSessionProxy\<'a\>, DBusError\>  
    * pub async fn list\_sessions(\&self) \-\> Result\<Vec\<SessionInfo\>, DBusError\>: Ruft LogindManagerProxy::list\_sessions() auf und konvertiert das Tupel-Array in Vec\<SessionInfo\>.  
    * pub async fn get\_session\_details(\&self, session\_path: \&zbus::zvariant::ObjectPath\<'\_\>) \-\> Result\<SessionInfo, DBusError\>: Ruft Properties vom LogindSessionProxy ab.  
    * pub async fn lock\_session(\&self, session\_id: \&str) \-\> Result\<(), DBusError\>  
    * pub async fn unlock\_session(\&self, session\_id: \&str) \-\> Result\<(), DBusError\>  
    * pub async fn lock\_all\_sessions(\&self) \-\> Result\<(), DBusError\>  
    * pub async fn unlock\_all\_sessions(\&self) \-\> Result\<(), DBusError\>  
    * pub async fn subscribe\_session\_new(\&self) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<SessionInfo, DBusError\>\>, DBusError\>: Abonniert SessionNew, konvertiert die Daten in SessionInfo.  
    * pub async fn subscribe\_session\_removed(\&self) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<(String, zbus::zvariant::OwnedObjectPath), DBusError\>\>, DBusError\>  
    * pub async fn subscribe\_prepare\_for\_sleep(\&self) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<bool, DBusError\>\>, DBusError\> Das PrepareForSleep-Signal ist von besonderer Bedeutung. Wenn start\_or\_stop true ist, kündigt dies einen bevorstehenden Suspend- oder Hibernate-Vorgang an.10 Die Desktop-Umgebung muss darauf reagieren, indem sie beispielsweise den Bildschirm sperrt, laufende Anwendungen benachrichtigt (falls ein entsprechendes Protokoll existiert) und kritische Zustände sichert. Bei false signalisiert es das Aufwachen des Systems, woraufhin der Desktop entsperrt und Dienste reaktiviert werden können.  
    * pub async fn subscribe\_session\_lock(\&self, session\_path: zbus::zvariant::OwnedObjectPath) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<(), DBusError\>\>, DBusError\>: Abonniert das Lock-Signal des spezifischen Session-Objekts.  
    * pub async fn subscribe\_session\_unlock(\&self, session\_path: zbus::zvariant::OwnedObjectPath) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<(), DBusError\>\>, DBusError\>: Abonniert das Unlock-Signal des spezifischen Session-Objekts.  
* **Implementierungsschritte**: Analog zu UPowerClient. Besondere Aufmerksamkeit gilt der korrekten Handhabung der PrepareForSleep-Signale und der Interaktion mit den Session-spezifischen Lock/Unlock-Signalen.  
* **Publisher/Subscriber**:  
  * Publisher: org.freedesktop.login1 D-Bus Dienst.  
  * Subscriber: LogindClient.

### **5\. Submodul: system::dbus::networkmanager\_client – NetworkManager D-Bus Client**

Dieser Client interagiert mit org.freedesktop.NetworkManager, um Netzwerkinformationen abzurufen und auf Zustandsänderungen zu reagieren. Diese Informationen sind sowohl für die UI-Darstellung als auch für KI-Funktionen (z.B. Online-Status-Prüfung) relevant.

* **Dateien**: system/dbus/networkmanager\_client.rs, system/dbus/networkmanager\_types.rs  
* **Spezifikation (networkmanager\_types.rs)**:  
  * pub enum NetworkManagerState { Unknown \= 0, Asleep \= 10, Disconnected \= 20, Disconnecting \= 30, Connecting \= 40, ConnectedLocal \= 50, ConnectedSite \= 60, ConnectedGlobal \= 70 } (Werte gemäß NMState aus der NetworkManager-Dokumentation).  
  * pub enum NetworkDeviceType { Unknown \= 0, Ethernet \= 1, Wifi \= 2, Wimax \= 5, Modem \= 6, Bluetooth \= 7, /\*... weitere Typen... \*/ } (Werte gemäß NMDeviceType).  
  * pub enum NetworkConnectivityState { Unknown \= 0, None \= 1, Portal \= 2, Limited \= 3, Full \= 4 } (Werte gemäß NMConnectivityState).  
  * pub struct NetworkDevice { pub object\_path: zbus::zvariant::OwnedObjectPath, pub interface: String, pub ip\_interface: String, pub driver: String, pub device\_type: NetworkDeviceType, pub state: u32, /\* NMDeviceState \*/ pub available\_connections: Vec\<zbus::zvariant::OwnedObjectPath\>, pub managed: bool, pub firmare\_missing: bool, pub plugged: bool, /\*... weitere Felder... \*/ }.  
  * pub struct ActiveConnection { pub object\_path: zbus::zvariant::OwnedObjectPath, pub connection\_object\_path: zbus::zvariant::OwnedObjectPath, pub specific\_object\_path: zbus::zvariant::OwnedObjectPath, pub id: String, pub uuid: String, pub conn\_type: String, pub devices: Vec\<zbus::zvariant::OwnedObjectPath\>, pub state: u32, /\* NMActiveConnectionState \*/ pub default: bool, pub default6: bool, pub vpn: bool, /\*... weitere Felder... \*/ }.  
  * pub struct NetworkManagerProperties { pub state: NetworkManagerState, pub connectivity: NetworkConnectivityState, pub wireless\_enabled: bool, pub wwan\_enabled: bool, pub active\_connections: Vec\<zbus::zvariant::OwnedObjectPath\>, /\*... \*/ }.  
* **Spezifikation (networkmanager\_client.rs)**:  
  * **Proxy-Definitionen**:  
    * NetworkManagerProxy für org.freedesktop.NetworkManager auf /org/freedesktop/NetworkManager.  
      * Methoden: GetDevices() \-\> zbus::Result\<Vec\<zbus::zvariant::OwnedObjectPath\>\>, GetActiveConnections() \-\> zbus::Result\<Vec\<zbus::zvariant::OwnedObjectPath\>\>, ActivateConnection(connection: \&zbus::zvariant::ObjectPath\<'\_\>, device: \&zbus::zvariant::ObjectPath\<'\_\>, specific\_object: \&zbus::zvariant::ObjectPath\<'\_\>) \-\> zbus::Result\<zbus::zvariant::OwnedObjectPath\>.  
      * Properties: State: u32, Connectivity: u32, WirelessEnabled: bool, WwanEnabled: bool, ActiveConnections: Vec\<zbus::zvariant::OwnedObjectPath\>.  
      * Signale: StateChanged(state: u32), DeviceAdded(device\_path: zbus::zvariant::OwnedObjectPath), DeviceRemoved(device\_path: zbus::zvariant::OwnedObjectPath).  
    * NMDeviceProxy für org.freedesktop.NetworkManager.Device (Pfad variabel).  
      * Properties: Udi: String, Interface: String, IpInterface: String, Driver: String, DeviceType: u32, State: u32, Managed: bool, AvailableConnections: Vec\<zbus::zvariant::OwnedObjectPath\>, FirmwareMissing: bool, Plugged: bool.  
    * NMActiveConnectionProxy für org.freedesktop.NetworkManager.Connection.Active (Pfad variabel).  
      * Properties: Connection: zbus::zvariant::OwnedObjectPath, SpecificObject: zbus::zvariant::OwnedObjectPath, Id: String, Uuid: String, Type: String, Devices: Vec\<zbus::zvariant::OwnedObjectPath\>, State: u32, Default: bool, Default6: bool, Vpn: bool.  
  * **Struktur**: NetworkManagerClient  
    * Felder: connection\_manager: Arc\<DBusConnectionManager\>, manager\_proxy\_path: Arc\<zbus::zvariant::ObjectPath\<'static\>\>.  
  * **Methoden** für NetworkManagerClient:  
    * pub async fn new(conn\_manager: Arc\<DBusConnectionManager\>) \-\> Result\<Self, DBusError\>  
    * async fn get\_manager\_proxy(\&self) \-\> Result\<NetworkManagerProxy\<'\_\>, DBusError\>  
    * async fn get\_device\_proxy\<'a\>(\&self, device\_path: &'a zbus::zvariant::ObjectPath\<'\_\>) \-\> Result\<NMDeviceProxy\<'a\>, DBusError\>  
    * async fn get\_active\_connection\_proxy\<'a\>(\&self, ac\_path: &'a zbus::zvariant::ObjectPath\<'\_\>) \-\> Result\<NMActiveConnectionProxy\<'a\>, DBusError\>  
    * pub async fn get\_properties(\&self) \-\> Result\<NetworkManagerProperties, DBusError\>  
    * pub async fn get\_devices(\&self) \-\> Result\<Vec\<NetworkDevice\>, DBusError\>: Ruft Pfade über GetDevices ab, dann für jeden Pfad die Details über NMDeviceProxy.  
    * pub async fn get\_active\_connections(\&self) \-\> Result\<Vec\<ActiveConnection\>, DBusError\>: Ruft Pfade über GetActiveConnections ab, dann für jeden Pfad die Details über NMActiveConnectionProxy.  
    * pub async fn subscribe\_state\_changed(\&self) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<NetworkManagerState, DBusError\>\>, DBusError\>: Abonniert StateChanged, konvertiert u32 in NetworkManagerState.  
    * pub async fn subscribe\_device\_added(\&self) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<NetworkDevice, DBusError\>\>, DBusError\>: Abonniert DeviceAdded, ruft dann Details für den neuen Pfad ab.  
    * pub async fn subscribe\_device\_removed(\&self) \-\> Result\<impl futures\_core::Stream\<Item \= Result\<zbus::zvariant::OwnedObjectPath, DBusError\>\>, DBusError\>  
* **Implementierungsschritte**: Analog zu UPowerClient. Die Datenstrukturen müssen die komplexen Informationen von NetworkManager korrekt abbilden.  
* **Publisher/Subscriber**:  
  * Publisher: org.freedesktop.NetworkManager D-Bus Dienst.  
  * Subscriber: NetworkManagerClient.  
* Die reaktive Aktualisierung des Netzwerkstatus bei Signalempfang ist für eine responsive UI und zuverlässige KI-Funktionen von Bedeutung. Änderungen an der Liste der Geräte oder aktiven Verbindungen erfordern, dass der Client die entsprechenden Detailinformationen neu abruft, da die Signale oft nur die Objektpfade der geänderten Entitäten enthalten.

### **6\. Submodul: system::dbus::secrets\_client – Freedesktop Secret Service D-Bus Client**

Dieser Client interagiert mit dem org.freedesktop.secrets-Dienst zum sicheren Speichern und Abrufen von sensiblen Daten wie API-Schlüsseln für Cloud-LLMs.13

* **Dateien**: system/dbus/secrets\_client.rs, system/dbus/secrets\_types.rs  
* **Spezifikation (secrets\_types.rs)**:  
  * pub struct Secret { pub session: zbus::zvariant::OwnedObjectPath, pub parameters: Vec\<u8\>, pub value: Vec\<u8\>, pub content\_type: String }  
  * pub struct SecretItemInfo { pub object\_path: zbus::zvariant::OwnedObjectPath, pub label: String, pub attributes: HashMap\<String, String\>, pub created: u64, pub modified: u64, pub locked: bool }  
  * pub struct SecretCollectionInfo { pub object\_path: zbus::zvariant::OwnedObjectPath, pub label: String, pub created: u64, pub modified: u64, pub locked: bool }  
  * pub enum PromptCompletedResult { Dismissed, Continue(Option\<zbus::zvariant::OwnedValue\> )}  
* **Spezifikation (secrets\_client.rs)**:  
  * **Proxy-Definitionen**:  
    * SecretServiceProxy für org.freedesktop.Secret.Service auf /org/freedesktop/secrets.  
      * Methoden: OpenSession(algorithm: \&str, input: \&zbus::zvariant::Value\<'\_\>) \-\> zbus::Result\<(zbus::zvariant::OwnedValue, zbus::zvariant::OwnedObjectPath)\>, CreateCollection(properties: HashMap\<\&str, \&zbus::zvariant::Value\<'\_\>\>, alias: \&str) \-\> zbus::Result\<(zbus::zvariant::OwnedObjectPath, zbus::zvariant::OwnedObjectPath /\* prompt \*/)\>, SearchItems(attributes: HashMap\<\&str, \&str\>) \-\> zbus::Result\<(Vec\<zbus::zvariant::OwnedObjectPath\>, Vec\<zbus::zvariant::OwnedObjectPath\>) /\* unlocked, locked \*/\>, Unlock(objects: &\[\&zbus::zvariant::ObjectPath\<'\_\>\]) \-\> zbus::Result\<(Vec\<zbus::zvariant::OwnedObjectPath\>, zbus::zvariant::OwnedObjectPath /\* prompt \*/)\>, Lock(objects: &\[\&zbus::zvariant::ObjectPath\<'\_\>\]) \-\> zbus::Result\<(Vec\<zbus::zvariant::OwnedObjectPath\>, zbus::zvariant::OwnedObjectPath /\* prompt \*/)\>, GetSecrets(items: &\[\&zbus::zvariant::ObjectPath\<'\_\>\], session: \&zbus::zvariant::ObjectPath\<'\_\>) \-\> zbus::Result\<HashMap\<zbus::zvariant::OwnedObjectPath, Secret\>\>.  
      * Properties: Collections: Vec\<zbus::zvariant::OwnedObjectPath\>.  
      * Signale: CollectionCreated(collection\_path: zbus::zvariant::OwnedObjectPath), CollectionChanged(collection\_path: zbus::zvariant::OwnedObjectPath), CollectionDeleted(collection\_path: zbus::zvariant::OwnedObjectPath).  
    * SecretCollectionProxy für org.freedesktop.Secret.Collection (Pfad variabel).  
      * Methoden: CreateItem(properties: HashMap\<\&str, \&zbus::zvariant::Value\<'\_\>\>, secret: \&Secret, replace: bool) \-\> zbus::Result\<(zbus::zvariant::OwnedObjectPath /\* item \*/, zbus::zvariant::OwnedObjectPath /\* prompt \*/)\>, SearchItems(attributes: HashMap\<\&str, \&str\>) \-\> zbus::Result\<Vec\<zbus::zvariant::OwnedObjectPath\>\>, Delete() \-\> zbus::Result\<zbus::zvariant::OwnedObjectPath /\* prompt \*/\>.  
      * Properties: Label: String, Created: u64, Modified: u64, Locked: bool, Items: Vec\<zbus::zvariant::OwnedObjectPath\>.  
    * SecretItemProxy für org.freedesktop.Secret.Item (Pfad variabel).  
      * Methoden: GetSecret(session: \&zbus::zvariant::ObjectPath\<'\_\>) \-\> zbus::Result\<Secret\>, SetSecret(secret: \&Secret) \-\> zbus::Result\<()\>, Delete() \-\> zbus::Result\<zbus::zvariant::OwnedObjectPath /\* prompt \*/\>.  
      * Properties: Label: String, Attributes: HashMap\<String, String\>, Created: u64, Modified: u64, Locked: bool.  
    * SecretPromptProxy für org.freedesktop.Secret.Prompt (Pfad variabel).  
      * Methoden: Prompt(window\_id: \&str) \-\> zbus::Result\<()\>  
      * Signale: Completed(dismissed: bool, result: zbus::zvariant::Value\<'static\>)  
  * **Struktur**: SecretsClient  
    * Felder: connection\_manager: Arc\<DBusConnectionManager\>, service\_proxy\_path: Arc\<zbus::zvariant::ObjectPath\<'static\>\>.  
  * **Methoden** für SecretsClient:  
    * pub async fn new(conn\_manager: Arc\<DBusConnectionManager\>) \-\> Result\<Self, DBusError\>  
    * async fn get\_service\_proxy(\&self) \-\> Result\<SecretServiceProxy\<'\_\>, DBusError\>  
    * async fn get\_collection\_proxy\<'a\>(\&self, path: &'a zbus::zvariant::ObjectPath\<'\_\>) \-\> Result\<SecretCollectionProxy\<'a\>, DBusError\>  
    * async fn get\_item\_proxy\<'a\>(\&self, path: &'a zbus::zvariant::ObjectPath\<'\_\>) \-\> Result\<SecretItemProxy\<'a\>, DBusError\>  
    * async fn get\_prompt\_proxy\<'a\>(\&self, path: &'a zbus::zvariant::ObjectPath\<'\_\>) \-\> Result\<SecretPromptProxy\<'a\>, DBusError\>  
    * pub async fn open\_session(\&self) \-\> Result\<zbus::zvariant::OwnedObjectPath /\* session\_path \*/, DBusError\>: Verwendet "plain" Algorithmus und leeren Input.  
    * pub async fn get\_default\_collection(\&self) \-\> Result\<zbus::zvariant::OwnedObjectPath, DBusError\>: Sucht nach der Collection mit Alias "default" oder erstellt sie.  
    * pub async fn store\_secret(\&self, collection\_path: \&zbus::zvariant::ObjectPath\<'\_\>, label: \&str, secret\_value: &\[u8\], attributes: HashMap\<String, String\>, session\_path: \&zbus::zvariant::ObjectPath\<'\_\>, window\_id\_provider: impl Fn() \-\> String \+ Send \+ Sync) \-\> Result\<zbus::zvariant::OwnedObjectPath, DBusError\>: Erstellt ein Secret-Struct, ruft CreateItem auf der Collection auf. Behandelt den zurückgegebenen Prompt-Pfad mit handle\_prompt\_if\_needed.  
    * pub async fn retrieve\_secret(\&self, item\_path: \&zbus::zvariant::ObjectPath\<'\_\>, session\_path: \&zbus::zvariant::ObjectPath\<'\_\>, window\_id\_provider: impl Fn() \-\> String \+ Send \+ Sync) \-\> Result\<Option\<Vec\<u8\>\>, DBusError\>: Ruft GetSecret auf dem Item auf. Falls das Item oder die Collection gesperrt ist, wird Unlock auf dem Service-Proxy versucht, was einen Prompt auslösen kann.  
    * pub async fn search\_items(\&self, attributes: HashMap\<String, String\>) \-\> Result\<Vec\<SecretItemInfo\>, DBusError\>: Ruft SearchItems auf dem Service-Proxy auf, dann für jeden gefundenen Pfad die Properties vom SecretItemProxy.  
    * async fn handle\_prompt\_if\_needed(\&self, prompt\_path: \&zbus::zvariant::ObjectPath\<'\_\>, window\_id\_provider: impl Fn() \-\> String \+ Send \+ Sync) \-\> Result\<PromptCompletedResult, DBusError\>: Diese Methode ist zentral für die Benutzerinteraktion. Wenn prompt\_path nicht "/" ist (was "kein Prompt nötig" bedeutet), wird ein SecretPromptProxy erstellt. Prompt(window\_id) wird aufgerufen, wobei window\_id von der UI-Schicht über window\_id\_provider dynamisch bereitgestellt wird. Anschließend wird auf das Completed-Signal des Prompts gewartet. Das Ergebnis des Signals (dismissed, result) wird in PromptCompletedResult umgewandelt. Die Notwendigkeit einer window\_id für Prompts erfordert eine enge Kopplung oder einen Callback-Mechanismus mit der UI-Schicht, da die Systemschicht selbst keine Fensterkonzepte oder \-IDs direkt verwaltet.  
* **Implementierungsschritte**: Definition der Typen, Generierung der Proxies. Besondere Sorgfalt ist beim Management von Sessions und der Handhabung von Prompts geboten. Der secret-service-rs Crate 13 kann als Referenz für die korrekte Implementierung der komplexen Abläufe dienen.  
* **Publisher/Subscriber**:  
  * Publisher: org.freedesktop.secrets D-Bus Dienst.  
  * Subscriber: SecretsClient.

### **7\. Submodul: system::dbus::policykit\_client – PolicyKit D-Bus Client**

Dieser Client interagiert mit org.freedesktop.PolicyKit1.Authority zur Überprüfung von Berechtigungen für privilegierte Aktionen \[User Query III.11\].

* **Dateien**: system/dbus/policykit\_client.rs, system/dbus/policykit\_types.rs  
* **Spezifikation (policykit\_types.rs)**:  
  * Bitflags-Struktur PolicyKitCheckAuthFlags:  
    * None \= 0  
    * AllowUserInteraction \= 1  
    * NoUserInteraction \= 2 (obwohl AllowUserInteraction \= false dasselbe bewirkt)  
    * AllowDowngrade \= 4  
    * RetainAuthorization \= 8  
  * pub struct PolicyKitSubject\<'a\> { pub kind: &'a str, pub details: HashMap\<&'a str, zbus::zvariant::Value\<'a\>\> } (z.B. kind \= "unix-process", details \= {"pid" \-\> Value::U32(self\_pid)}).  
  * pub struct PolicyKitAuthorizationResult { pub is\_authorized: bool, pub is\_challenge: bool, pub details: HashMap\<String, zbus::zvariant::OwnedValue\> }  
* **Spezifikation (policykit\_client.rs)**:  
  * **Proxy-Definition**:  
    * PolicyKitAuthorityProxy für org.freedesktop.PolicyKit1.Authority auf /org/freedesktop/PolicyKit1/Authority.  
      * Methoden: CheckAuthorization\<'a\>(subject: PolicyKitSubject\<'a\>, action\_id: \&str, details: HashMap\<\&str, \&str\>, flags: u32, cancellation\_id: \&str) \-\> zbus::Result\<PolicyKitAuthorizationResult\>.  
  * **Struktur**: PolicyKitClient  
    * Felder: connection\_manager: Arc\<DBusConnectionManager\>, authority\_proxy\_path: Arc\<zbus::zvariant::ObjectPath\<'static\>\>.  
  * **Methoden** für PolicyKitClient:  
    * pub async fn new(conn\_manager: Arc\<DBusConnectionManager\>) \-\> Result\<Self, DBusError\>  
    * async fn get\_authority\_proxy(\&self) \-\> Result\<PolicyKitAuthorityProxy\<'\_\>, DBusError\>  
    * pub async fn check\_authorization(\&self, subject\_pid: Option\<u32\>, action\_id: \&str, details: HashMap\<String, String\>, allow\_interaction: bool) \-\> Result\<PolicyKitAuthorizationResult, DBusError\>: Erstellt ein PolicyKitSubject. Wenn subject\_pid Some(pid) ist, wird kind \= "unix-process" und details \= {"pid": Value::U32(pid)} verwendet. Andernfalls wird der PID des aktuellen Prozesses verwendet. Setzt die flags basierend auf allow\_interaction. cancellation\_id kann leer sein. Ruft PolicyKitAuthorityProxy::CheckAuthorization auf. Die korrekte Definition des subject ist sicherheitskritisch. Es muss klar sein, im Kontext welcher Entität (der Desktop-Umgebung selbst oder einer anfragenden Anwendung) die Berechtigung geprüft wird.  
* **Implementierungsschritte**: Proxy-Generierung, Implementierung der Client-Methoden. Die subject-Erstellung muss sorgfältig implementiert werden.  
* **Publisher/Subscriber**:  
  * Publisher: org.freedesktop.PolicyKit1.Authority D-Bus Dienst.  
  * Subscriber: PolicyKitClient.

## **B. Modul: system::outputs – Verwaltung der Anzeigeausgänge (Display Output Management)**

Dieses Modul ist für die Erkennung, Konfiguration und Verwaltung von Anzeigeausgängen (Monitoren) zuständig. Es implementiert die serverseitige Logik für die relevanten Wayland-Protokolle (wl\_output, xdg-output-unstable-v1, wlr-output-management-unstable-v1, wlr-output-power-management-unstable-v1) unter Verwendung der Abstraktionen von Smithay.14 Die korrekte Handhabung von Monitorkonfigurationen, Auflösungen, Skalierung und Hotplugging ist entscheidend für eine gute Benutzererfahrung, insbesondere in Multi-Monitor-Umgebungen.

### **1\. Submodul: system::outputs::error – Fehlerbehandlung für Output-Operationen**

Definiert spezifische Fehlertypen für Operationen im Zusammenhang mit Anzeigeausgängen.

* **Datei**: system/outputs/error.rs  
* **Spezifikation**:  
  * Öffentliches Enum OutputError mit thiserror::Error und Debug.  
  * **Varianten**:  
    * \# DeviceAccessFailed { device: String, \#\[source\] source: std::io::Error } (Relevant bei direktem DRM-Zugriff, z.B. über smithay::backend::drm).  
    * \#\[error("Wayland protocol error for '{protocol}': {message}")\] ProtocolError { protocol: String, message: String } (Für Fehler bei der Implementierung von Wayland-Protokollen).  
    * \#\[error("Output configuration conflict: {details}")\] ConfigurationConflict { details: String } (Wenn eine angeforderte Konfiguration nicht angewendet werden kann).  
    * \#\[error("Failed to create Wayland resource '{resource}': {reason}")\] ResourceCreationFailed { resource: String, reason: String }.  
    * \# SmithayOutputError { \#\[source\] source: smithay::output::OutputError } (Falls Smithay spezifische Fehler für smithay::output::Output-Operationen definiert).  
    * \#\[error("Output '{name}' not found")\] OutputNotFound { name: String }.  
    * \#\[error("Mode not supported by output '{output\_name}'")\] ModeNotSupported { output\_name: String, mode\_details: String }.  
* **Implementierungsschritte**: Definition des Enums, \#\[error(...)\]-Attribute und From-Implementierungen für zugrundeliegende Fehler (z.B. std::io::Error).

### **2\. Submodul: system::outputs::output\_device – Kernrepräsentation eines Anzeigeausgangs**

Diese Struktur kapselt den Zustand und die Logik eines einzelnen physischen Anzeigeausgangs.

* **Datei**: system/outputs/output\_device.rs  
* **Spezifikation**:  
  * **Struktur**: OutputDevice  
    * Felder:  
      * name: String (Eindeutiger Name des Outputs, z.B. "DP-1", "HDMI-A-2").  
      * smithay\_output: smithay::output::Output 15: Die Kernabstraktion von Smithay für einen Output. Enthält physische Eigenschaften, aktuelle und bevorzugte Modi.  
      * wl\_output\_global: Option\<wayland\_server::backend::GlobalId\>: Die ID des wl\_output-Globals, das diesen physischen Output repräsentiert.  
      * xdg\_output\_global: Option\<wayland\_server::backend::GlobalId\>: Die ID des zxdg\_output\_v1-Globals.  
      * wlr\_head\_global: Option\<wayland\_server::backend::GlobalId\>: Die ID des zwlr\_output\_head\_v1-Globals (für wlr-output-management).  
      * wlr\_power\_global: Option\<wayland\_server::backend::GlobalId\>: Die ID des zwlr\_output\_power\_v1-Globals (für wlr-output-power-management).  
      * enabled: bool: Gibt an, ob der Output aktuell aktiviert ist.  
      * current\_dpms\_state: DpmsState: Enum für den DPMS-Zustand (On, Standby, Suspend, Off).  
      * pending\_config\_serial: Option\<u32\>: Das Serial einer laufenden wlr-output-management-Konfiguration.  
  * **Struktur**: OutputDevicePendingState (für wlr-output-management)  
    * Felder: mode: Option\<smithay::output::Mode\>, position: Option\<smithay::utils::Point\<i32, smithay::utils::Logical\>\>, transform: Option\<smithay::utils::Transform\>, scale: Option\<f64\>, enabled: Option\<bool\>, adaptive\_sync\_enabled: Option\<bool\>.  
  * **Enum**: DpmsState { On, Standby, Suspend, Off }  
  * **Methoden** für OutputDevice:  
    * pub fn new(name: String, physical\_properties: smithay::output::PhysicalProperties, preferred\_mode: Option\<smithay::output::Mode\>, possible\_modes: Vec\<smithay::output::Mode\>, display\_handle: \&wayland\_server::DisplayHandle, compositor\_state: \&mut YourCompositorState) \-\> Result\<Self, OutputError\>: Erstellt ein neues OutputDevice. Initialisiert self.smithay\_output \= smithay::output::Output::new(name.clone(), physical\_properties.clone());. Fügt die possible\_modes und preferred\_mode zum smithay\_output hinzu (add\_mode(), set\_preferred\_mode()). Setzt einen initialen Zustand (z.B. bevorzugter Modus, Position (0,0), normale Transformation, Skalierung 1.0) via self.apply\_state\_internal(...). Das Erstellen der Globals (wl\_output\_global, xdg\_output\_global, etc.) erfolgt typischerweise durch den OutputManager oder die jeweiligen Protokoll-Handler, nicht direkt im Konstruktor des OutputDevice, da dies den globalen Display-Zustand modifiziert.  
    * pub fn name(\&self) \-\> \&str  
    * pub fn smithay\_output(\&self) \-\> \&smithay::output::Output  
    * pub fn current\_mode(\&self) \-\> Option\<smithay::output::Mode\>: Gibt den aktuellen Modus aus smithay\_output.current\_mode() zurück.  
    * pub fn current\_transform(\&self) \-\> smithay::utils::Transform: Gibt die aktuelle Transformation aus smithay\_output.current\_transform() zurück.  
    * pub fn current\_scale(\&self) \-\> smithay::output::Scale: Gibt die aktuelle Skalierung aus smithay\_output.current\_scale() zurück.  
    * pub fn current\_position(\&self) \-\> smithay::utils::Point\<i32, smithay::utils::Logical\>: Gibt die aktuelle Position aus smithay\_output.current\_position() zurück.  
    * pub fn is\_enabled(\&self) \-\> bool  
    * pub fn apply\_state(\&mut self, mode: Option\<smithay::output::Mode\>, transform: Option\<smithay::utils::Transform\>, scale: Option\<smithay::output::Scale\>, position: Option\<smithay::utils::Point\<i32, smithay::utils::Logical\>\>, enabled: bool) \-\> Result\<(), OutputError\>: Interne Methode, die self.smithay\_output.change\_current\_state(mode, transform, scale, position) aufruft.15 Aktualisiert self.enabled. Wenn enabled false ist, wird None für mode an change\_current\_state übergeben. Smithay sendet die wl\_output und xdg\_output Events (geometry, mode, scale, done, logical\_position, logical\_size) automatisch.  
    * pub fn set\_dpms\_state(\&mut self, state: DpmsState) \-\> Result\<(), OutputError\>: Ändert den DPMS-Zustand des Outputs (z.B. über DRM). Aktualisiert self.current\_dpms\_state. Löst ggf. Events für wlr-output-power-management aus.  
    * pub fn supported\_modes(\&self) \-\> Vec\<smithay::output::Mode\>: Gibt self.smithay\_output.modes() zurück.  
    * pub fn physical\_properties(\&self) \-\> smithay::output::PhysicalProperties: Gibt self.smithay\_output.physical\_properties() zurück.  
    * pub fn add\_mode(\&mut self, mode: smithay::output::Mode): Fügt einen Modus zu self.smithay\_output hinzu.  
    * pub fn set\_preferred\_mode(\&mut self, mode: smithay::output::Mode): Setzt den bevorzugten Modus in self.smithay\_output.  
    * Methoden zum Setzen und Abrufen der Global-IDs (wl\_output\_global, xdg\_output\_global, etc.).  
    * pub fn destroy\_globals(\&mut self, display\_handle: \&wayland\_server::DisplayHandle): Entfernt alle zugehörigen Globals vom DisplayHandle.  
* **Implementierungsschritte**:  
  1. Definiere OutputDevice, OutputDevicePendingState und DpmsState.  
  2. Implementiere new(): Initialisiert smithay::output::Output korrekt.  
  3. Implementiere apply\_state(): Ruft smithay\_output.change\_current\_state() auf.  
  4. Implementiere set\_dpms\_state(): Interagiert mit der DRM-Schicht oder dem entsprechenden Backend, um den Energiezustand zu ändern.

### **3\. Submodul: system::outputs::manager – Zentrales Management der Anzeigeausgänge**

Der OutputManager verwaltet eine Liste aller bekannten OutputDevice-Instanzen und behandelt Hotplug-Events.

* **Datei**: system/outputs/manager.rs  
* **Spezifikation**:  
  * **Struktur**: OutputManager  
    * Felder: outputs: HashMap\<String, Arc\<Mutex\<OutputDevice\>\>\> (HashMap mit Output-Name als Schlüssel), udev\_event\_source\_token: Option\<calloop::RegistrationToken\> (falls udev verwendet wird). Die Verwendung von Arc\<Mutex\<OutputDevice\>\> ist hier geboten, da OutputDevice-Instanzen von verschiedenen Teilen des Systems (z.B. DRM-Event-Handler, Wayland-Dispatcher für wlr-output-management, D-Bus-Handler für Power-Events) potenziell nebenläufig modifiziert werden könnten. Arc ermöglicht das Teilen des Besitzes, und Mutex stellt den exklusiven Zugriff für Schreiboperationen sicher, um Datenkonsistenz zu gewährleisten.5  
  * **Enum**: HotplugEvent  
    * DeviceAdded { name: String, path: std::path::PathBuf, physical\_properties: smithay::output::PhysicalProperties, modes: Vec\<smithay::output::Mode\>, preferred\_mode: Option\<smithay::output::Mode\>, enabled: bool, is\_drm: bool, drm\_device\_fd: Option\<std::os::unix::io::OwnedFd\> /\* nur wenn is\_drm true \*/ }  
    * DeviceRemoved { name: String }  
  * **Methoden** für OutputManager:  
    * pub fn new() \-\> Self  
    * pub fn add\_output(\&mut self, output\_device: Arc\<Mutex\<OutputDevice\>\>): Fügt ein OutputDevice zur outputs-Map hinzu.  
    * pub fn remove\_output(\&mut self, name: \&str, display\_handle: \&wayland\_server::DisplayHandle) \-\> Option\<Arc\<Mutex\<OutputDevice\>\>\>: Entfernt ein OutputDevice anhand seines Namens, zerstört dessen Globals und gibt es zurück.  
    * pub fn find\_output\_by\_name(\&self, name: \&str) \-\> Option\<Arc\<Mutex\<OutputDevice\>\>\>  
    * pub fn all\_outputs(\&self) \-\> Vec\<Arc\<Mutex\<OutputDevice\>\>\>: Gibt eine geklonte Liste aller Arc\<Mutex\<OutputDevice\>\> zurück.  
    * pub fn handle\_hotplug\_event(\&mut self, event: HotplugEvent, display\_handle: \&wayland\_server::DisplayHandle, compositor\_state: \&mut YourCompositorState) \-\> Result\<(), OutputError\>: Verarbeitet Hotplug-Events. Bei DeviceAdded: 1\. Prüft, ob ein Output mit diesem Namen bereits existiert. 2\. Erstellt ein neues OutputDevice mit den übergebenen Eigenschaften. 3\. Ruft output\_device\_created\_notifications auf, um die notwendigen Globals zu erstellen und Handler zu informieren. 4\. Fügt das neue OutputDevice zur outputs-Map hinzu. Bei DeviceRemoved: 1\. Sucht das OutputDevice anhand des Namens. 2\. Ruft output\_device\_removed\_notifications auf, um Globals zu zerstören und Handler zu informieren. 3\. Entfernt das OutputDevice aus der outputs-Map. Die Hotplug-Logik ist stark abhängig vom verwendeten Backend. Bei einem DRM/udev-Backend kommen die Events vom UdevBackend 18, die dann in HotplugEvent übersetzt werden müssen.  
    * fn output\_device\_created\_notifications(\&self, output\_device: \&Arc\<Mutex\<OutputDevice\>\>, display\_handle: \&wayland\_server::DisplayHandle, compositor\_state: \&mut YourCompositorState): Private Hilfsmethode. Erstellt wl\_output, zxdg\_output\_v1 und zwlr\_output\_head\_v1 Globals für das neue Gerät. Benachrichtigt die WlrOutputManagementState und WlrOutputPowerManagementState über das neue Gerät.  
    * fn output\_device\_removed\_notifications(\&self, output\_device: \&Arc\<Mutex\<OutputDevice\>\>, display\_handle: \&wayland\_server::DisplayHandle, compositor\_state: \&mut YourCompositorState): Private Hilfsmethode. Zerstört die Globals des entfernten Geräts. Benachrichtigt die relevanten Handler.  
* **Implementierungsschritte**:  
  1. Definiere OutputManager und HotplugEvent.  
  2. Implementiere CRUD-Methoden für OutputDevice-Instanzen.  
  3. Implementiere handle\_hotplug\_event. Die genaue Quelle der HotplugEvents (z.B. Udev-Integration) muss hier berücksichtigt werden.  
  4. Implementiere die ...\_notifications-Hilfsmethoden, um die Erstellung/Zerstörung von Globals und die Benachrichtigung anderer Handler zu zentralisieren.

### **4\. Submodul: system::outputs::wl\_output\_handler – Implementierung des wl\_output Protokolls**

Die Logik für wl\_output wird durch Smithays Output-Typ und den OutputHandler-Trait gehandhabt.15

* **Datei**: Integration in den globalen Compositor-Zustand und system::outputs::manager.rs.  
* **Spezifikation**:  
  * **Smithay Integration**:  
    * Der globale Compositor-Zustand (YourCompositorState) implementiert smithay::wayland::output::OutputHandler.  
    * smithay::delegate\_output\!(YourCompositorState); muss im globalen Zustand deklariert werden.  
    * Beim Hinzufügen eines neuen physischen Outputs im OutputManager::handle\_hotplug\_event (oder einer ähnlichen Funktion) wird für das neue OutputDevice (welches ein smithay::output::Output enthält) die Methode output\_dev.smithay\_output().create\_global::\<YourCompositorState\>(display\_handle) aufgerufen.15 Die zurückgegebene GlobalId wird im OutputDevice::wl\_output\_global gespeichert.  
  * **Implementierung des OutputHandler-Traits für YourCompositorState**:  
    * fn output\_state(\&mut self) \-\> \&mut smithay::wayland::output::OutputManagerState: Gibt eine Referenz zum OutputManagerState des Compositors zurück. Dieser OutputManagerState wird typischerweise im globalen Zustand des Compositors gehalten und bei der Initialisierung mit OutputManagerState::new() oder OutputManagerState::new\_with\_xdg\_output() 15 erstellt.  
    * fn new\_output(\&mut self, \_output: \&smithay::reexports::wayland\_server::protocol::wl\_output::WlOutput, \_output\_data: \&smithay::wayland::output::OutputData): Diese Methode wird aufgerufen, wenn ein Client an ein wl\_output-Global bindet. Hier kann client-spezifischer Zustand initialisiert werden, falls nötig. OutputData enthält eine Referenz zum smithay::output::Output.  
    * fn output\_destroyed(\&mut self, \_output: \&smithay::reexports::wayland\_server::protocol::wl\_output::WlOutput, \_output\_data: \&smithay::wayland::output::OutputData): Wird aufgerufen, wenn ein wl\_output-Global zerstört wird.  
  * Smithay sendet geometry, mode, scale, done Events an wl\_output-Clients automatisch, wenn Output::change\_current\_state() auf dem entsprechenden smithay::output::Output aufgerufen wird.15  
* **Implementierungsschritte**:  
  1. Stelle sicher, dass der globale Compositor-Zustand (YourCompositorState) ein Feld für OutputManagerState hat und den OutputHandler-Trait implementiert.  
  2. Integriere den Aufruf von smithay\_output().create\_global() in die Logik, die neue OutputDevice-Instanzen erstellt (z.B. in OutputManager::output\_device\_created\_notifications).  
  3. Implementiere die Methoden des OutputHandler-Traits. Oftmals ist hier keine spezifische Logik notwendig, da Smithay vieles übernimmt.

### **5\. Submodul: system::outputs::wlr\_output\_management\_handler – Implementierung des wlr-output-management-unstable-v1 Protokolls**

Dieses Submodul implementiert die serverseitige Logik für das wlr-output-management-unstable-v1-Protokoll, das es Clients (wie kanshi 19) ermöglicht, Display-Konfigurationen abzufragen und zu ändern.20

* **Dateien**: system/outputs/wlr\_output\_management/mod.rs, system/outputs/wlr\_output\_management/manager\_handler.rs, system/outputs/wlr\_output\_management/head\_handler.rs, system/outputs/wlr\_output\_management/mode\_handler.rs, system/outputs/wlr\_output\_management/configuration\_handler.rs  
* **Protokoll-Objekte**: zwlr\_output\_manager\_v1, zwlr\_output\_head\_v1, zwlr\_output\_mode\_v1, zwlr\_output\_configuration\_v1, zwlr\_output\_configuration\_head\_v1.  
* **Spezifikation**:  
  * **Struktur**: WlrOutputManagementState (im globalen Compositor-Zustand)  
    * Felder:  
      * output\_manager: Arc\<Mutex\<OutputManager\>\> (Referenz zum globalen OutputManager).  
      * configurations: HashMap\<wayland\_server::backend::ObjectId, Arc\<Mutex\<OutputConfigurationRequest\>\>\> (speichert laufende Konfigurationsanfragen, Schlüssel ist die ID des zwlr\_output\_configuration\_v1-Objekts).  
      * global\_serial: std::sync::atomic::AtomicU32 (für die done-Events des Managers).  
  * **Struktur**: OutputConfigurationRequest  
    * Felder: serial: u32 (Serial, mit dem die Konfiguration erstellt wurde), client: wayland\_server::Client, pending\_changes: HashMap\<String /\* OutputDevice name \*/, HeadChangeRequest\>, config\_resource: wayland\_server::Resource\<ZwlrOutputConfigurationV1\>.  
  * **Struktur**: HeadChangeRequest  
    * Felder: mode: Option\<smithay::output::Mode\>, position: Option\<smithay::utils::Point\<i32, smithay::utils::Logical\>\>, transform: Option\<smithay::utils::Transform\>, scale: Option\<f64\>, enabled: Option\<bool\>, adaptive\_sync\_enabled: Option\<bool\>.  
  * **User Data Structs**:  
    * WlrOutputManagerGlobalData { output\_manager\_state: Weak\<Mutex\<WlrOutputManagementState\>\> } (für zwlr\_output\_manager\_v1 Global).  
    * WlrOutputHeadGlobalData { output\_device: Weak\<Mutex\<OutputDevice\>\>, output\_manager\_state: Weak\<Mutex\<WlrOutputManagementState\>\> } (für zwlr\_output\_head\_v1 Ressourcen).  
    * WlrOutputModeGlobalData { mode: smithay::output::Mode } (für zwlr\_output\_mode\_v1 Ressourcen).  
    * WlrOutputConfigurationUserData { id: wayland\_server::backend::ObjectId, output\_manager\_state: Weak\<Mutex\<WlrOutputManagementState\>\> } (für zwlr\_output\_configuration\_v1 Ressourcen).  
    * WlrOutputConfigurationHeadUserData { output\_device\_name: String, config\_request\_id: wayland\_server::backend::ObjectId, output\_manager\_state: Weak\<Mutex\<WlrOutputManagementState\>\> } (für zwlr\_output\_configuration\_head\_v1 Ressourcen).  
  * **Smithay Integration**: Der globale Compositor-Zustand (YourCompositorState) implementiert:  
    * GlobalDispatch\<ZwlrOutputManagerV1, WlrOutputManagerGlobalData\>  
    * Dispatch\<ZwlrOutputManagerV1, WlrOutputManagerGlobalData, YourCompositorState\>  
    * Dispatch\<ZwlrOutputHeadV1, WlrOutputHeadGlobalData, YourCompositorState\>  
    * Dispatch\<ZwlrOutputModeV1, WlrOutputModeGlobalData, YourCompositorState\>  
    * Dispatch\<ZwlrOutputConfigurationV1, WlrOutputConfigurationUserData, YourCompositorState\>  
    * Dispatch\<ZwlrOutputConfigurationHeadV1, WlrOutputConfigurationHeadUserData, YourCompositorState\>  
    * smithay::delegate\_dispatch\!(YourCompositorState:);  
  * **Initialisierung**:  
    * Ein WlrOutputManagementState wird im globalen Compositor-Zustand erstellt.  
    * Ein zwlr\_output\_manager\_v1-Global wird mit display\_handle.create\_global() registriert.  
  * **Anfragebehandlung für zwlr\_output\_manager\_v1 (manager\_handler.rs)**:  
    * bind: Sendet den aktuellen Zustand aller Outputs (Heads und deren Modi) an den Client über die head, mode, done, finished Events des Managers.20  
    * destroy: Standard.  
    * create\_configuration(config\_resource: ZwlrOutputConfigurationV1, serial: u32):  
      1. Erstellt ein neues OutputConfigurationRequest mit dem gegebenen serial und der Client-ID. Speichert es in WlrOutputManagementState::configurations.  
      2. Sendet den aktuellen Zustand aller OutputDevices (als zwlr\_output\_head\_v1-Events: name, description, physical\_size, enabled, current\_mode, position, transform, scale, make, model, serial\_number, adaptive\_sync) und deren unterstützte Modi (als zwlr\_output\_mode\_v1-Events: size, refresh, preferred) an das neue config\_resource.  
      3. Jeder Kopf und Modus erhält eine eigene Ressource (ZwlrOutputHeadV1, ZwlrOutputModeV1), die mit den entsprechenden Daten initialisiert wird.  
      4. Beendet die Sequenz mit zwlr\_output\_head\_v1.done() für jeden Kopf und zwlr\_output\_manager\_v1.done(current\_serial) für den Manager selbst. Der serial-Parameter ist hierbei zentral: Die gesendeten Kopf- und Modusinformationen müssen dem Zustand entsprechen, den der Client mit diesem serial erwartet.  
  * **Anfragebehandlung für zwlr\_output\_configuration\_head\_v1 (configuration\_handler.rs)**:  
    * destroy: Standard.  
    * enable(), disable(): Aktualisiert enabled im HeadChangeRequest des zugehörigen OutputConfigurationRequest.  
    * set\_mode(mode: \&ZwlrOutputModeV1): Speichert den Modus (aus WlrOutputModeGlobalData) im HeadChangeRequest.  
    * set\_custom\_mode(...), set\_position(...), set\_transform(...), set\_scale(...), set\_adaptive\_sync(...): Speichern die angeforderten Änderungen im HeadChangeRequest.  
  * **Anfragebehandlung für zwlr\_output\_configuration\_v1 (configuration\_handler.rs)**:  
    * destroy: Verwirft die Konfigurationsanfrage und entfernt sie aus WlrOutputManagementState::configurations.  
    * apply():  
      1. Überprüft, ob der serial der Konfiguration noch aktuell ist (d.h. ob sich der globale Output-Zustand seit Erstellung der Konfiguration geändert hat, z.B. durch Hotplug). Wenn nicht, sendet cancelled und zerstört die Konfiguration.  
      2. Versucht, alle pending\_changes im OutputConfigurationRequest auf die entsprechenden OutputDevice-Instanzen (via OutputManager) anzuwenden.  
      3. Wenn alle Änderungen erfolgreich sind: Sendet succeeded an den Client und zerstört die Konfiguration. Aktualisiert den globalen OutputManager-Serial und sendet done an alle zwlr\_output\_manager\_v1-Instanzen.  
      4. Wenn Fehler auftreten: Sendet failed an den Client, macht Änderungen rückgängig (falls möglich) und zerstört die Konfiguration.  
    * test(): Ähnlich wie apply(), aber ohne die Änderungen tatsächlich anzuwenden. Validiert die Konfiguration.  
  * **Event-Generierung**: Der OutputManager (oder eine dedizierte Komponente) muss bei Änderungen am Output-Zustand (Hotplug, Modusänderung durch andere Quellen) die head, mode, done, finished Events an alle gebundenen zwlr\_output\_manager\_v1-Instanzen senden und den globalen Serial erhöhen.  
* **Implementierungsschritte**:  
  1. Definiere die Zustands- und UserData-Strukturen.  
  2. Implementiere GlobalDispatch für ZwlrOutputManagerV1.  
  3. Implementiere Dispatch für alle relevanten Protokollobjekte.  
  4. Die apply/test-Logik muss sorgfältig implementiert werden, um Atomarität (oder zumindest Fehlererkennung und \-behandlung) und korrekte Serial-Handhabung sicherzustellen.  
  5. Die Benachrichtigung über Änderungen im globalen Output-Zustand an alle Manager-Instanzen ist entscheidend. Dies kann über einen Listener-Mechanismus oder Callbacks im OutputManager erfolgen.  
* **Tabelle: WLR-Output-Management Protokoll Interaktionen**

| Client Aktion | Server Reaktion (Requests an Client, Events an Client) | Betroffene Zustände (Server) |
| :---- | :---- | :---- |
| Bindet an zwlr\_output\_manager\_v1 | Für jeden Output: head (mit Name, Desc, etc.), mode (für jeden Modus), enabled, current\_mode, position, etc. done (pro Kopf). Dann done(serial) vom Manager. | WlrOutputManagementState (neuer Client registriert), global\_serial |
| create\_configuration(serial) | Erstellt zwlr\_output\_configuration\_v1. Sendet aktuellen Output-Zustand (Heads, Modi) an diese Konfigurationsinstanz. | WlrOutputManagementState::configurations (neue Anfrage hinzugefügt) |
| zwlr\_output\_configuration\_head\_v1.set\_X(...) | Keine direkten Events an Client. | OutputConfigurationRequest::pending\_changes aktualisiert. |
| zwlr\_output\_configuration\_v1.apply() | Wenn serial aktuell & Konfig gültig: succeeded. Dann head/mode/done Events vom Manager mit neuem globalen Serial. Wenn serial veraltet: cancelled. Wenn Konfig ungültig: failed. | OutputManager::outputs (Zustand der OutputDevices geändert), global\_serial erhöht. WlrOutputManagementState::configurations (Anfrage entfernt). |
| zwlr\_output\_configuration\_v1.test() | Wenn serial aktuell & Konfig gültig: succeeded. Wenn serial veraltet: cancelled. Wenn Konfig ungültig: failed. | WlrOutputManagementState::configurations (Anfrage entfernt). Keine Zustandsänderung an Outputs. |
| Hotplug (z.B. Monitor angeschlossen/abgezogen) | An alle zwlr\_output\_manager\_v1: head (für neuen Output) / finished (für entfernten Output), done(new\_serial). | OutputManager::outputs aktualisiert, global\_serial erhöht. Laufende Konfigurationen werden bei nächstem apply/test als cancelled markiert. |

Diese Tabelle verdeutlicht die komplexen Interaktionsflüsse und die Bedeutung der Serial-Nummern für die Zustandssynchronisation zwischen Client und Compositor.

### **6\. Submodul: system::outputs::wlr\_output\_power\_management\_handler – Implementierung des wlr-output-power-management-unstable-v1 Protokolls**

Dieses Submodul implementiert die serverseitige Logik für das wlr-output-power-management-unstable-v1-Protokoll, das es Clients erlaubt, den Energiezustand von Monitoren zu steuern (z.B. An/Aus).22

* **Dateien**: system/outputs/wlr\_output\_power\_management/mod.rs, system/outputs/wlr\_output\_power\_management/manager\_handler.rs, system/outputs/wlr\_output\_power\_management/power\_control\_handler.rs  
* **Protokoll-Objekte**: zwlr\_output\_power\_manager\_v1, zwlr\_output\_power\_v1.  
* **Spezifikation**:  
  * **Struktur**: WlrOutputPowerManagementState (im globalen Compositor-Zustand)  
    * Felder:  
      * output\_manager: Arc\<Mutex\<OutputManager\>\>  
      * active\_controllers: HashMap\<String /\* OutputDevice name \*/, wayland\_server::Resource\<ZwlrOutputPowerV1\>\>: Speichert den aktiven Controller pro Output-Namen.  
  * **User Data Structs**:  
    * WlrOutputPowerManagerGlobalData { output\_power\_manager\_state: Weak\<Mutex\<WlrOutputPowerManagementState\>\> }.  
    * WlrOutputPowerControlUserData { output\_device\_name: String, output\_power\_manager\_state: Weak\<Mutex\<WlrOutputPowerManagementState\>\> }.  
  * **Smithay Integration**: Der globale Compositor-Zustand (YourCompositorState) implementiert:  
    * GlobalDispatch\<ZwlrOutputPowerManagerV1, WlrOutputPowerManagerGlobalData\>  
    * Dispatch\<ZwlrOutputPowerManagerV1, WlrOutputPowerManagerGlobalData, YourCompositorState\>  
    * Dispatch\<ZwlrOutputPowerV1, WlrOutputPowerControlUserData, YourCompositorState\>  
    * smithay::delegate\_dispatch\!(YourCompositorState:);  
  * **Initialisierung**: Ein WlrOutputPowerManagementState wird im globalen Zustand erstellt. Ein zwlr\_output\_power\_manager\_v1-Global wird registriert.  
  * **Anfragebehandlung für zwlr\_output\_power\_manager\_v1 (manager\_handler.rs)**:  
    * bind: Standard.  
    * destroy: Standard.  
    * get\_output\_power(output\_power\_resource: ZwlrOutputPowerV1, output: \&WlOutput):  
      1. Ermittelt den Namen des OutputDevice, das zum WlOutput gehört (z.B. über UserData des WlOutput).  
      2. Prüft, ob bereits ein aktiver Controller für diesen Output-Namen in active\_controllers existiert.  
      3. Wenn ja: Sendet failed an output\_power\_resource und zerstört es. Es darf nur einen Controller pro Output geben.22  
      4. Wenn nein: Speichert output\_power\_resource in active\_controllers für den Output-Namen. Sendet den aktuellen DPMS-Zustand des OutputDevice als initiales mode-Event an output\_power\_resource.  
  * **Anfragebehandlung für zwlr\_output\_power\_v1 (power\_control\_handler.rs)**:  
    * destroy: Entfernt den Controller aus active\_controllers.  
    * set\_mode(mode: u32):  
      1. Ermittelt das zugehörige OutputDevice anhand des in WlrOutputPowerControlUserData gespeicherten Namens.  
      2. Konvertiert mode (0 für Off, 1 für On 22) in den entsprechenden DpmsState.  
      3. Ruft output\_device.lock().unwrap().set\_dpms\_state(new\_dpms\_state) auf.  
      4. Wenn erfolgreich, sendet mode(mode) an den Client.  
      5. Wenn der Output den Modus nicht unterstützt oder ein anderer Fehler auftritt, sendet failed.  
  * **Event-Generierung**:  
    * Wenn sich der DPMS-Zustand eines OutputDevice ändert (auch extern, z.B. durch Inaktivität), muss der WlrOutputPowerManagementState dies erkennen und das mode-Event an den ggf. existierenden aktiven Controller für diesen Output senden.  
    * Wenn ein OutputDevice entfernt wird, muss ein failed-Event an den zugehörigen Controller gesendet und dieser zerstört werden.  
* **Implementierungsschritte**:  
  1. Definiere die Zustands- und UserData-Strukturen.  
  2. Implementiere GlobalDispatch für ZwlrOutputPowerManagerV1.  
  3. Implementiere Dispatch für ZwlrOutputPowerManagerV1 und ZwlrOutputPowerV1.  
  4. Die set\_mode-Anfrage muss mit der tatsächlichen Hardware-Steuerung (z.B. DRM DPMS über das OutputDevice) interagieren.  
  5. Sicherstellen, dass Änderungen des Power-Modus das mode-Event auslösen und die Exklusivität der Controller gewahrt bleibt.

### **7\. Submodul: system::outputs::xdg\_output\_handler – Implementierung des xdg-output-unstable-v1 Protokolls**

Dieses Submodul implementiert die serverseitige Logik für das xdg-output-unstable-v1-Protokoll, das Clients detailliertere Informationen über die logische Geometrie von Outputs liefert.

* **Datei**: system/outputs/xdg\_output\_handler.rs (kann auch als Integration in wl\_output\_handler oder manager erfolgen).  
* **Protokoll-Objekte**: zxdg\_output\_manager\_v1, zxdg\_output\_v1.  
* **Spezifikation**:  
  * **Smithay Integration**:  
    * Der globale Compositor-Zustand (YourCompositorState) implementiert:  
      * GlobalDispatch\<ZxdgOutputManagerV1, XdgOutputManagerGlobalData\>  
      * Dispatch\<ZxdgOutputManagerV1, XdgOutputManagerGlobalData, YourCompositorState\>  
      * Dispatch\<ZxdgOutputV1, XdgOutputGlobalData, YourCompositorState\>  
      * smithay::delegate\_dispatch\!(YourCompositorState:);  
    * XdgOutputManagerGlobalData { output\_manager: Weak\<Mutex\<OutputManager\>\> }.  
    * XdgOutputGlobalData { output\_device: Weak\<Mutex\<OutputDevice\>\> }.  
    * Die Erstellung der zxdg\_output\_manager\_v1-Globals und zxdg\_output\_v1-Ressourcen kann über Smithay's OutputManagerState::new\_with\_xdg\_output() 15 erfolgen, das automatisch ein zxdg\_output\_v1-Global erstellt, wenn ein wl\_output-Global erstellt wird. Alternativ kann dies manuell im OutputManager::output\_device\_created\_notifications geschehen.  
  * **Initialisierung**: Ein zxdg\_output\_manager\_v1-Global wird registriert.  
  * **Anfragebehandlung für zxdg\_output\_manager\_v1**:  
    * bind: Standard.  
    * destroy: Standard.  
    * get\_xdg\_output(xdg\_output\_resource: ZxdgOutputV1, output: \&WlOutput):  
      1. Ermittelt das OutputDevice, das zum WlOutput gehört.  
      2. Initialisiert xdg\_output\_resource mit den aktuellen logischen Daten des OutputDevice (Position, Größe) und sendet logical\_position, logical\_size, name, description, gefolgt von done.  
  * **Anfragebehandlung für zxdg\_output\_v1**:  
    * destroy: Standard.  
  * **Event-Generierung**:  
    * Wenn sich die logische Position, Größe, der Name oder die Beschreibung eines OutputDevice ändern, müssen die entsprechenden Events (logical\_position, logical\_size, name, description) an alle gebundenen zxdg\_output\_v1-Instanzen gesendet werden, gefolgt von einem done-Event. Dies wird typischerweise von Smithay gehandhabt, wenn Output::change\_current\_state() aufgerufen wird.  
* **Implementierungsschritte**:  
  1. Definiere die UserData-Strukturen.  
  2. Implementiere GlobalDispatch für ZxdgOutputManagerV1.  
  3. Implementiere Dispatch für ZxdgOutputManagerV1 und ZxdgOutputV1.  
  4. Sicherstellen, dass Änderungen an den relevanten OutputDevice-Eigenschaften (Position, Größe, Name, Beschreibung) die korrekten Events auslösen. Smithay's Output-Struktur sollte dies bei korrekter Verwendung von change\_current\_state bereits gewährleisten.

## **III. Implementierungsleitfaden (Implementation Guide)**

A. Allgemeine Hinweise: Die Implementierung aller hier spezifizierten Module und Submodule muss streng den in der technischen Gesamtspezifikation definierten Entwicklungsrichtlinien folgen. Dies umfasst insbesondere:  
\* Coding Style & Formatierung: Verbindliche Nutzung von rustfmt mit Standardkonfiguration und Einhaltung der Rust API Guidelines \[User Query IV.4.1\].  
\* API-Design: Befolgung der Rust API Guidelines Checklist für konsistente und idiomatische Schnittstellen \[User Query IV.4.2\].  
\* Fehlerbehandlung: Konsequente Verwendung des thiserror-Crates zur Definition spezifischer Fehler-Enums pro Modul (DBusError, OutputError) \[User Query IV.4.3\].  
\* Logging & Tracing: Einsatz des tracing-Crate-Frameworks für strukturiertes, kontextbezogenes Logging und Tracing von Operationen \[User Query IV.4.4\].  
B. Detaillierte Schritte pro Sub-Modul: Die oben in den Spezifikationen genannten Implementierungsschritte für jedes Submodul sind als detaillierte Arbeitsanweisungen zu verstehen. Dies beinhaltet:  
\* Strukturen und Enums: Exakte Definition aller Felder mit Typen und Sichtbarkeitsmodifikatoren (pub, pub(crate), private).  
\* Methodenimplementierung: Vollständige Implementierung aller öffentlichen Methoden gemäß den Signaturen. Vor- und Nachbedingungen sind zu beachten. Interne Logik muss robust und fehlerresistent sein.  
\* D-Bus Clients: Die generierten zbus-Proxies sind die primäre Schnittstelle zu den D-Bus-Diensten. Die Client-Wrapper-Klassen (UPowerClient, LogindClient, etc.) müssen die Rohdaten der Proxies in die anwendungsfreundlichen Typen aus den \*\_types.rs-Dateien konvertieren und Fehlerbehandlung durchführen. Signal-Handler müssen asynchron implementiert werden und die empfangenen Daten korrekt parsen.  
\* Wayland Protocol Handler: Die Implementierung der Dispatch- und GlobalDispatch-Traits für die Output-Protokolle erfordert sorgfältiges Management des Zustands, der oft in UserData-Strukturen der Wayland-Ressourcen gespeichert wird. Das korrekte Senden von Events an die Clients als Reaktion auf Anfragen oder Zustandsänderungen ist entscheidend.  
\* Interaktion der Submodule:  
\* Der OutputManager ist die zentrale Verwaltungsinstanz für OutputDevice-Objekte.  
\* Die Wayland-Protokoll-Handler für Outputs (wl\_output\_handler, wlr\_output\_management\_handler, etc.) greifen auf den OutputManager und die darin enthaltenen OutputDevice-Instanzen zu, um Informationen abzufragen oder Konfigurationen anzuwenden.  
\* Beispielsweise wird der wlr\_output\_management\_handler bei einer apply()-Anfrage die gewünschten Änderungen an die entsprechenden OutputDevice-Instanzen im OutputManager weiterleiten. Diese wiederum nutzen ihr internes smithay::output::Output-Objekt, um die Änderungen wirksam zu machen, was dann die notwendigen wl\_output- und xdg\_output-Events auslöst.  
\* Änderungen durch Hotplug-Events, die vom OutputManager verarbeitet werden, müssen Benachrichtigungen an die wlr-output-management und wlr-output-power-management Handler auslösen, damit diese ihre Clients über die geänderte Output-Konfiguration informieren können (z.B. Senden von head und done Events).

## **IV. Anhang (Appendix)**

### **A. D-Bus Schnittstellenübersicht**

Die folgende Tabelle fasst die wichtigsten D-Bus-Dienste zusammen, mit denen die Systemschicht interagiert:  
**Tabelle: D-Bus Service Details**

| Dienstname | Objektpfad (Manager/Service) | Interface (Haupt) | Relevante Methoden/Signale/Properties (Beispiele) | Korrespondierendes system::dbus Submodul |
| :---- | :---- | :---- | :---- | :---- |
| UPower | /org/freedesktop/UPower | org.freedesktop.UPower | EnumerateDevices(), GetDisplayDevice(), OnBattery (Prop), DeviceAdded (Sig), DeviceRemoved (Sig). Für Devices (org.freedesktop.UPower.Device): Type, State, Percentage, TimeToEmpty, TimeToFull (Props).7 | upower\_client |
| systemd-logind | /org/freedesktop/login1 | org.freedesktop.login1.Manager | ListSessions(), LockSession(), UnlockSession(), PrepareForSleep (Sig), SessionNew (Sig), SessionRemoved (Sig). Für Sessions (org.freedesktop.login1.Session): Lock() (Sig), Unlock() (Sig), Active (Prop).10 | logind\_client |
| NetworkManager | /org/freedesktop/NetworkManager | org.freedesktop.NetworkManager | GetDevices(), GetActiveConnections(), State (Prop), Connectivity (Prop), StateChanged (Sig), DeviceAdded (Sig). Für Devices (org.freedesktop.NetworkManager.Device): DeviceType, State (Props). Für Active Connections (org.freedesktop.NetworkManager.Connection.Active): Type, State, Default (Props). | networkmanager\_client |
| Freedesktop Secret Service | /org/freedesktop/secrets | org.freedesktop.Secret.Service | OpenSession(), CreateCollection(), SearchItems(), Unlock(), GetSecrets(), CollectionCreated (Sig). Für Collections (org.freedesktop.Secret.Collection): CreateItem(), Label (Prop). Für Items (org.freedesktop.Secret.Item): GetSecret(), SetSecret(), Label (Prop). Für Prompts (org.freedesktop.Secret.Prompt): Prompt(), Completed (Sig).13 | secrets\_client |
| PolicyKit | /org/freedesktop/PolicyKit1/Authority | org.freedesktop.PolicyKit1.Authority | CheckAuthorization() \[User Query III.11\]. | policykit\_client |

Diese Übersicht dient als Referenz für die spezifischen D-Bus-Interaktionen und deren Implementierungsort innerhalb des system::dbus-Moduls. Sie erleichtert das Verständnis der Abhängigkeiten von externen Systemdiensten.

### **B. Wayland Output Protokollübersicht**

Die folgende Tabelle gibt einen Überblick über die im system::outputs-Modul implementierten Wayland-Protokolle und deren Handler:  
**Tabelle: Wayland Output Protocol Handler**

| Protokollname | Hauptinterface(s) (Server) | Verantwortlicher Handler (Trait/Struktur im Code) | Wichtige Requests (vom Client an Server) | Wichtige Events (vom Server an Client) | Korrespondierendes system::outputs Submodul |
| :---- | :---- | :---- | :---- | :---- | :---- |
| Wayland Core Output | wl\_output | YourCompositorState (implementiert smithay::wayland::output::OutputHandler) | release | geometry, mode, done, scale 15 | wl\_output\_handler (Integration) |
| XDG Output | zxdg\_output\_manager\_v1, zxdg\_output\_v1 | YourCompositorState (implementiert GlobalDispatch und Dispatch für XDG Output Interfaces) | destroy (manager/output), get\_xdg\_output (manager) | logical\_position, logical\_size, done, name, description (output) | xdg\_output\_handler |
| WLR Output Management | zwlr\_output\_manager\_v1, zwlr\_output\_head\_v1, zwlr\_output\_mode\_v1, zwlr\_output\_configuration\_v1, zwlr\_output\_configuration\_head\_v1 | WlrOutputManagementState, YourCompositorState (implementiert relevante Dispatch-Traits) | create\_configuration (manager), apply, test (configuration), enable\_head, set\_mode (config\_head) 20 | head, done (manager), name, mode, current\_mode (head), succeeded, failed, cancelled (configuration) 20 | wlr\_output\_management\_handler |
| WLR Output Power Management | zwlr\_output\_power\_manager\_v1, zwlr\_output\_power\_v1 | WlrOutputPowerManagementState, YourCompositorState (implementiert relevante Dispatch-Traits) | get\_output\_power (manager), set\_mode (power\_control) 22 | mode, failed (power\_control) 22 | wlr\_output\_power\_management\_handler |

Diese Tabelle dient als Referenz für die implementierten Wayland-Protokolle im Bereich der Output-Verwaltung und zeigt die jeweiligen Zuständigkeiten der Handler-Komponenten auf. Sie ist nützlich, um die Struktur und die Verantwortlichkeiten innerhalb des system::outputs-Moduls nachzuvollziehen.

#### **Referenzen**

1. Writing a client proxy \- zbus: D-Bus for Rust made easy \- GitHub Pages, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/client.html](https://dbus2.github.io/zbus/client.html)  
2. Zugriff am Januar 1, 1970, [https://smithay.github.io/smithay/smithay/wayland/shell/xdg/struct.XdgShellState.html\#method.get\_grab\_start\_edges](https://smithay.github.io/smithay/smithay/wayland/shell/xdg/struct.XdgShellState.html#method.get_grab_start_edges)  
3. Error Handling for Large Rust Projects \- Best Practice in GreptimeDB, Zugriff am Mai 13, 2025, [https://greptime.com/blogs/2024-05-07-error-rust](https://greptime.com/blogs/2024-05-07-error-rust)  
4. Error in zbus \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/enum.Error.html](https://docs.rs/zbus/latest/zbus/enum.Error.html)  
5. Arc in std::sync \- Rust Documentation, Zugriff am Mai 14, 2025, [https://doc.rust-lang.org/std/sync/struct.Arc.html](https://doc.rust-lang.org/std/sync/struct.Arc.html)  
6. doc/dbus/org.freedesktop.UPower.ref.xml · debian/0.9.23-1 \- GitLab, Zugriff am Mai 14, 2025, [https://source.puri.sm/Librem5/upower/-/blob/debian/0.9.23-1/doc/dbus/org.freedesktop.UPower.ref.xml?ref\_type=tags](https://source.puri.sm/Librem5/upower/-/blob/debian/0.9.23-1/doc/dbus/org.freedesktop.UPower.ref.xml?ref_type=tags)  
7. org.freedesktop.UPower: UPower Reference Manual, Zugriff am Mai 14, 2025, [https://upower.freedesktop.org/docs/UPower.html](https://upower.freedesktop.org/docs/UPower.html)  
8. D-Bus API Reference: UPower Reference Manual, Zugriff am Mai 14, 2025, [https://upower.freedesktop.org/docs/ref-dbus.html](https://upower.freedesktop.org/docs/ref-dbus.html)  
9. "Connection" Search \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/?search=Connection](https://docs.rs/zbus/latest/zbus/?search=Connection)  
10. org.freedesktop.login1, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.html](https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.html)  
11. org.freedesktop.login1 \- The D-Bus interface of systemd-logind \- Ubuntu Manpage, Zugriff am Mai 14, 2025, [https://manpages.ubuntu.com/manpages/plucky/man5/org.freedesktop.login1.5.html](https://manpages.ubuntu.com/manpages/plucky/man5/org.freedesktop.login1.5.html)  
12. org.freedesktop.login1, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.html\#Signals](https://www.freedesktop.org/software/systemd/man/org.freedesktop.login1.html#Signals)  
13. Zugriff am Januar 1, 1970, [https://smithay.github.io/smithay/smithay/wayland/shell/xdg/struct.XdgShellState.html\#method.get\_grab\_start\_button](https://smithay.github.io/smithay/smithay/wayland/shell/xdg/struct.XdgShellState.html#method.get_grab_start_button)  
14. smithay/ output.rs, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/src/smithay/output.rs.html](https://smithay.github.io/smithay/src/smithay/output.rs.html)  
15. smithay::wayland::output \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/wayland/output/index.html](https://smithay.github.io/smithay/smithay/wayland/output/index.html)  
16. Zugriff am Januar 1, 1970, [https://smithay.github.io/smithay/smithay/output/struct.Output.html](https://smithay.github.io/smithay/smithay/output/struct.Output.html)  
17. tokio::sync \- Rust \- People @EECS, Zugriff am Mai 14, 2025, [https://people.eecs.berkeley.edu/\~pschafhalter/pub/erdos/doc/tokio/sync/](https://people.eecs.berkeley.edu/~pschafhalter/pub/erdos/doc/tokio/sync/)  
18. smithay::backend::udev \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/backend/udev/index.html](https://smithay.github.io/smithay/smithay/backend/udev/index.html)  
19. support wlr-output-management-unstable-v1? · YaLTeR niri · Discussion \#172 \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/YaLTeR/niri/discussions/172](https://github.com/YaLTeR/niri/discussions/172)  
20. wlr output management protocol | Wayland Explorer, Zugriff am Mai 14, 2025, [https://wayland.app/protocols/wlr-output-management-unstable-v1](https://wayland.app/protocols/wlr-output-management-unstable-v1)  
21. rcalixte/awesome-wayland: A curated list of Wayland resources \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/rcalixte/awesome-wayland](https://github.com/rcalixte/awesome-wayland)  
22. wlr output power management protocol | Wayland Explorer, Zugriff am Mai 14, 2025, [https://wayland.app/protocols/wlr-output-power-management-unstable-v1](https://wayland.app/protocols/wlr-output-power-management-unstable-v1)  
23. wayland-rs/historical\_changelog.md at master \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/Smithay/wayland-rs/blob/master/historical\_changelog.md](https://github.com/Smithay/wayland-rs/blob/master/historical_changelog.md)  
24. File: wlr-output-power-management-unstable-v1.xml \- Debian Sources, Zugriff am Mai 14, 2025, [https://sources.debian.org/src/phosh/0.8.0-1/protocol/wlr-output-power-management-unstable-v1.xml/](https://sources.debian.org/src/phosh/0.8.0-1/protocol/wlr-output-power-management-unstable-v1.xml/)