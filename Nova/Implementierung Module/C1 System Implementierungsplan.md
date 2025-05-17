# **Ultra-Feinspezifikation und Implementierungsplan: Systemschicht \- Teil 1/4**

## **I. Einleitung**

### **A. Zweck und Geltungsbereich dieses Dokuments (Teil 1/4 der Systemschicht)**

Dieses Dokument stellt den ersten von vier Teilen der Ultra-Feinspezifikation und des Implementierungsplans für die Systemschicht der neuartigen Linux-Desktop-Umgebung dar. Sein primäres Ziel ist es, Entwicklern eine erschöpfende und unzweideutige Anleitung für die direkte Implementierung der Kernkomponenten des Compositors und der Eingabeverarbeitung zu liefern. Der Detaillierungsgrad zielt darauf ab, jegliche Interpretationsspielräume während der Entwicklung auszuschließen; alle algorithmischen Entscheidungen, Datenstrukturen und API-Signaturen sind hierin vordefiniert.  
Der Geltungsbereich dieses ersten Teils ist strikt auf die Module system::compositor und system::input beschränkt, wie sie in der "Technischen Gesamtspezifikation und Entwicklungsrichtlinien" (im Folgenden als "Gesamtspezifikation" bezeichnet) definiert sind. Diese Module bilden das Fundament für die visuelle Darstellung und Benutzerinteraktion und sind somit grundlegend für alle nachfolgenden Komponenten der Systemschicht sowie für die darüberliegenden Schichten der Desktop-Umgebung.

### **B. Bezug zur "Technischen Gesamtspezifikation und Entwicklungsrichtlinien"**

Dieses Dokument ist eine direkte und detaillierte Erweiterung der Gesamtspezifikation. Es übersetzt die dort getroffenen übergeordneten Architekturentscheidungen, die Auswahl des Technologie-Stacks (Rust, Smithay, libinput usw.) und die Entwicklungsrichtlinien (Programmierstil, Fehlerbehandlung mittels thiserror, API-Designprinzipien, tracing für Logging) \[Gesamtspezifikation: Abschnitte II, III, IV\] in konkrete, implementierbare Spezifikationen. Insbesondere werden die in Abschnitt V.3 der Gesamtspezifikation skizzierten Komponenten der Systemschicht – hier der Compositor und die Eingabesubsysteme – detailliert ausgeführt.  
Die strikte Einhaltung der Gesamtspezifikation ist bindend. Sollten während der detaillierten Spezifikationsphase Konflikte oder Unklarheiten auftreten, die nicht durch dieses Dokument aufgelöst werden können, so sind die Prinzipien und Entscheidungen der Gesamtspezifikation maßgeblich. Dies unterstreicht die Notwendigkeit eines Prozesses zur Klärung solcher Fälle, um die Integrität der Gesamtarchitektur zu wahren. Die Qualität und Voraussicht der Gesamtspezifikation sind entscheidend für den Erfolg der Spezifikationen der einzelnen Schichten, da Lücken oder Inkonsistenzen in der Gesamtspezifikation sich in den detaillierten Implementierungsplänen potenzieren würden.

### **C. Überblick über die behandelten Module: system::compositor und system::input**

Dieser erste Teil der Systemspezifikation konzentriert sich auf zwei grundlegende Module:

1. **system::compositor**: Dieses Modul implementiert die Kernlogik des Wayland-Compositors unter Verwendung des Smithay-Toolkits. Zu seinen Verantwortlichkeiten gehören die Verwaltung von Wayland-Client-Verbindungen, der Lebenszyklus von Oberflächen (Erstellung, Mapping, Rendering, Zerstörung), die Pufferbehandlung (Shared Memory, SHM) und die Integration mit Shell-Protokollen, insbesondere xdg\_shell für modernes Desktop-Fenstermanagement. Es orchestriert das Rendering, delegiert jedoch die eigentlichen Zeichenbefehle an eine Renderer-Schnittstelle, die in späteren Teilen dieser Spezifikation detailliert wird.  
2. **system::input**: Dieses Modul ist für die gesamte Verarbeitung von Benutzereingaben zuständig, die von Geräten wie Tastaturen, Mäusen und Touchpads stammen. Es nutzt primär libinput für die Erfassung von Rohdaten-Ereignissen und die Eingabeabstraktionen von Smithay für das Seat- und Fokusmanagement.

Die Auswahl dieser beiden Module für den ersten Teil der Spezifikation ist strategisch, da sie das absolute Fundament für die Benutzerinteraktion und die visuelle Präsentation der Desktop-Umgebung bilden. Ohne einen funktionierenden Compositor und ein zuverlässiges Eingabesystem können keine übergeordneten Systemfunktionen oder Benutzeroberflächen realisiert werden. Fehler oder Ineffizienzen in diesen grundlegenden Modulen hätten kaskadierende negative Auswirkungen auf die gesamte Benutzererfahrung, einschließlich Leistung, Reaktionsfähigkeit und Stabilität. Daher müssen die von diesen Modulen für andere Schichten (Domänen- und UI-Schicht) bereitgestellten APIs von Anfang an außergewöhnlich stabil und wohldefiniert sein, da Änderungen hier zu einem späteren Zeitpunkt sehr kostspielig wären.  
Die enge Verzahnung dieser beiden Module ist offensichtlich: Vom system::input-Modul verarbeitete Eingabeereignisse bestimmen oft Fokusänderungen (verwaltet durch den SeatHandler), die wiederum beeinflussen, wie der system::compositor Ereignisse an Client-Oberflächen (WlSurface) weiterleitet. Das Verständnis des Compositors für Oberflächenlayout und \-zustand (verwaltet durch XdgShellHandler, CompositorHandler) ist für das Eingabesystem unerlässlich, um Ereignisziele korrekt zu identifizieren. Die DesktopState-Struktur, die den Gesamtzustand des Compositors kapselt, wird der zentrale Punkt sein, der all diese Smithay-Zustandsstrukturen hält und die notwendigen Handler implementiert.

#### **Tabelle: Dokumentkonventionen**

Zur Gewährleistung von Klarheit und Konsistenz in der Terminologie und den Referenzen in diesem und den nachfolgenden Teilen der Systemschichtspezifikation werden folgende Konventionen verwendet:

| Begriff/Konvention | Beschreibung | Beispiel |
| :---- | :---- | :---- |
| DesktopState | Die zentrale Compositor-Zustandsstruktur, die alle Smithay-Handler-Traits implementieren wird. | impl CompositorHandler for DesktopState |
| Gesamtspezifikation | Bezieht sich auf das Dokument "Technische Gesamtspezifikation und Entwicklungsrichtlinien". | Gemäß Gesamtspezifikation Abschnitt III. |
| WlFoo | Bezieht sich auf Wayland-Protokollobjekte (z.B. WlSurface, WlSeat). | fn commit(surface: \&WlSurface) |
| XdgFoo | Bezieht sich auf XDG-Shell-Protokollobjekte (z.B. XdgSurface, XdgToplevel). | let toplevel: ToplevelSurface \=... |
| Snippet-ID | Verweise auf Recherchematerial, z.B..1 | Smithay verwendet calloop.2 |
| system::foo::bar | Bezieht sich auf Module innerhalb der aktuellen Projektstruktur. | system::compositor::core |
| \# | Standardattribut für Fehlerdefinitionen gemäß Entwicklungsrichtlinien. | Siehe CompositorCoreError Definition. |
| tracing::{info, debug, error} | Standardmakros für Logging gemäß Entwicklungsrichtlinien. | tracing::info\!("Neue Oberfläche erstellt"); |

*Begründung für den Wert dieser Tabelle:* Diese Tabelle etabliert ein klares, gemeinsames Vokabular und Referenzierungssystem, das für ein Dokument dieser technischen Tiefe und für ein Projekt mit mehreren Entwicklern unerlässlich ist. Sie minimiert Mehrdeutigkeiten und stellt sicher, dass alle Beteiligten Verweise auf externe Dokumente, interne Komponenten und Wayland/Smithay-Entitäten verstehen.

## **II. Entwicklungsmodul: system::compositor (Smithay-basierter Wayland Compositor)**

### **A. Modulübersicht**

Dieses Modul implementiert die Kernlogik des Wayland-Compositors unter Verwendung des Smithay-Toolkits.1 Seine Hauptverantwortlichkeiten umfassen:

* Verwaltung von Wayland-Client-Verbindungen und deren Lebenszyklus.  
* Handhabung von Wayland-Protokollobjekten: wl\_display, wl\_compositor, wl\_subcompositor, wl\_shm, wl\_surface und XDG-Shell-Objekte (xdg\_wm\_base, xdg\_surface, xdg\_toplevel, xdg\_popup).  
* Integration mit der calloop-Ereignisschleife für die Ereignisverteilung.1  
* Koordination mit dem Rendering-Backend (hier werden Abstraktionen definiert, die konkrete Implementierung erfolgt in späteren Teilen).  
* Verwaltung von Oberflächenhierarchien, Rollen und Zuständen (z.B. Pufferanhänge, Schadensverfolgung).

Die Designphilosophie von Smithay, modular zu sein und kein einschränkendes Framework darzustellen 5, bedeutet, dass das system::compositor-Modul zwar Bausteine erhält, aber für deren korrekte Assemblierung und Verwaltung selbst verantwortlich ist. Dies schließt ein signifikantes Zustandsmanagement und Logik innerhalb der zentralen DesktopState-Struktur ein. Smithay fördert die Verwendung einer zentralen, mutierbaren Zustandsstruktur, die an Callbacks übergeben wird, um exzessive Nutzung von Rc\<RefCell\<T\>\> oder Arc\<Mutex\<T\>\> zu vermeiden.2 Verschiedene Smithay-Komponenten wie CompositorState, XdgShellState und ShmState sind so konzipiert, dass sie Teil der Hauptzustandsstruktur des Entwicklers werden. Handler-Traits (CompositorHandler, XdgShellHandler etc.) werden von dieser Hauptzustandsstruktur implementiert.6 Folglich wird DesktopState zu einer zentralen Drehscheibe für Wayland-Protokollinteraktionen. Während Smithay Low-Level-Protokolldetails handhabt, müssen die einzigartigen Richtlinien des Compositors (Fensterplatzierung, Fokusregeln jenseits des Basisprotokolls usw.) oft innerhalb der Handler-Trait-Methoden implementiert werden. Dies erfordert ein sorgfältiges Design von DesktopState, um seine Verantwortlichkeiten zu verwalten, ohne zu einem "God-Objekt" zu werden.  
Die Wahl von Smithay, das nativ in Rust geschrieben ist, passt perfekt zur primären Sprachwahl des Projekts (Rust) \[Gesamtspezifikation: Abschn. 3.1, 3.4\]. Dies minimiert die FFI-Komplexität im Kern des Compositors und nutzt die Sicherheitsgarantien von Rust. Die Verwendung eines Rust-nativen Toolkits für ein Rust-basiertes Projekt reduziert die Risiken und den Overhead, die mit der Sprachinteroperabilität (FFI) verbunden sind, wie z.B. unsichere C-Bindungen, Nichtübereinstimmungen bei der Speicherverwaltung und komplexe Build-System-Integration. Dies sollte zu einem robusteren und wartbareren Compositor-Kern führen als die direkte Integration von C-basierten Bibliotheken. Die Leistungscharakteristik des Compositors wird sowohl von der Effizienz von Smithay als auch von der Qualität des eigenen Rust-Codes innerhalb der Handler stark beeinflusst.

### **B. Submodul 1: Compositor-Kern (system::compositor::core)**

Dieses Submodul etabliert die grundlegenden Elemente für die Verwaltung von Wayland-Oberflächen und die Kernoperationen des Compositors.

#### **1\. Datei: compositor\_state.rs**

* **Zweck**: Definiert und verwaltet den primären Zustand für die Globals wl\_compositor und wl\_subcompositor und handhabt den Client-spezifischen Compositor-Zustand.  
* **Struktur: CompositorCoreError**  
  * Definiert Fehler, die spezifisch für Kernoperationen des Compositors sind.  
  * Verwendet thiserror gemäß den Entwicklungsrichtlinien.8  
  * **Tabelle: CompositorCoreError-Varianten**

| Variantenname | Felder | \#\[error("...")\] Nachricht (Beispiel) |
| :---- | :---- | :---- |
| GlobalCreationFailed | (String) | "Erstellung des globalen Objekts {0} fehlgeschlagen" |
| RoleError | (\#\[from\] SurfaceRoleError) | "Fehler bei der Oberflächenrolle: {0}" |
| ClientDataMissing | (wayland\_server::backend::ClientId) | "Client-Daten für Client-ID {0:?} nicht gefunden" |
| SurfaceDataMissing | (wayland\_server::protocol::wl\_surface::WlSurface) | "SurfaceData für WlSurface {0:?} nicht gefunden oder falscher Typ" |
| InvalidSurfaceState | (String) | "Ungültiger Oberflächenzustand: {0}" |

\*Begründung für den Wert dieser Tabelle:\* Klare, spezifische Fehlertypen sind entscheidend für die Fehlersuche und eine robuste Fehlerbehandlung und stehen im Einklang mit den Qualitätszielen des Projekts. \`thiserror\` vereinfacht deren Definition erheblich.

* **Struktur: DesktopState (Teilweise Definition \- Fokus auf Compositor-Aspekte)**  
  * Diese Struktur wird den zentralen Zustand für den gesamten Desktop kapseln. Hier konzentrieren wir uns auf Felder, die für den CompositorHandler relevant sind.  
  * Felder:  
    * compositor\_state: CompositorState (aus smithay::wayland::compositor) 6  
    * display\_handle: DisplayHandle (aus smithay::wayland::display::DisplayHandle, ermöglicht Interaktion mit der Wayland-Anzeige) 11  
    * loop\_handle: LoopHandle\<Self\> (aus calloop::LoopHandle\<Self\>, zur Interaktion mit der Ereignisschleife) 2  
    * (Weitere Zustände wie ShmState, XdgShellState, SeatState etc. werden in ihren jeweiligen Abschnitten detailliert.)  
  * Konstruktor:  
    Rust  
    // system/src/compositor/core/compositor\_state.rs  
    use smithay::wayland::compositor::{CompositorState, CompositorClientState, CompositorHandler};  
    use smithay::reexports::wayland\_server::{Client, DisplayHandle, protocol::wl\_surface::WlSurface};  
    use smithay::reexports::calloop::LoopHandle;  
    use std::sync::Arc;  
    use parking\_lot::Mutex; // Gemäß Vorgabe: Rust-Standard-Mutex oder crossbeam/parking\_lot  
                            // Hier parking\_lot für potenziell bessere Performance in umkämpften Szenarien.  
    use super::surface\_management::SurfaceData; // Pfad anpassen  
    use super::error::CompositorCoreError; // Pfad anpassen

    pub struct ClientCompositorData {  
        // Wird benötigt, um CompositorClientState pro Client zu speichern  
        pub compositor\_state: CompositorClientState,  
    }

    pub struct DesktopState {  
        pub display\_handle: DisplayHandle,  
        pub loop\_handle: LoopHandle\<Self\>,  
        pub compositor\_state: CompositorState,  
        // Weitere Zustände hier einfügen  
    }

    impl DesktopState {  
        pub fn new(display\_handle: DisplayHandle, loop\_handle: LoopHandle\<Self\>) \-\> Self {  
            let compositor\_state \= CompositorState::new::\<Self\>(\&display\_handle);  
            Self {  
                display\_handle,  
                loop\_handle,  
                compositor\_state,  
                // Initialisierung weiterer Zustände  
            }  
        }  
    }

* **Implementierung: CompositorHandler für DesktopState** 6  
  * Dieses Trait ist zentral dafür, wie Smithay Compositor-Ereignisse an unsere Anwendungslogik delegiert.  
  * Die Implementierung von ClientData (oft eine UserDataMap) in Smithay ist entscheidend für die Zuordnung beliebiger, typsicherer Daten zu Wayland-Client-Objekten.1 Wenn ein neuer Client eine Verbindung herstellt oder zum ersten Mal mit dem Compositor-Global interagiert, muss CompositorClientState korrekt initialisiert und in ClientData eingefügt werden. Die Bereinigung dieses Client-spezifischen Zustands wird implizit von Smithay gehandhabt, wenn ein Client die Verbindung trennt, da ClientData und dessen Inhalt dann verworfen werden.  
  * Methodenimplementierungen werden in der folgenden Tabelle detailliert.  
* **Tabelle: CompositorHandler-Methodenimplementierungsdetails für DesktopState**

| Methodenname | Signatur | Detaillierte Schritt-für-Schritt-Logik | Wichtige Smithay Funktionen/Daten | Fehlerbehandlung |
| :---- | :---- | :---- | :---- | :---- |
| compositor\_state | fn compositor\_state(\&mut self) \-\> \&mut CompositorState | 1\. \&mut self.compositor\_state zurückgeben. | self.compositor\_state | N/A |
| client\_compositor\_state | fn client\_compositor\_state\<'a\>(\&self, client: &'a Client) \-\> &'a CompositorClientState | 1\. tracing::debug\!(client\_id \=?client.id(), "Anfrage für ClientCompositorState"); 2\. match client.get\_data::\<Arc\<Mutex\<ClientCompositorData\>\>\>() (Annahme: ClientCompositorData wird in einem Arc\<Mutex\<\>\> in ClientData gespeichert). 3\. Wenn Some(data), let guard \= data.lock(); \&guard.compositor\_state zurückgeben (Achtung: Lebensdauer des Guards beachten; Smithay erwartet einen direkten Verweis. Ggf. Box::leak oder unsicheren Code vermeiden, indem CompositorClientState direkt in ClientData ist, falls Smithay dies unterstützt, oder die Datenstruktur anpassen). Smithay erwartet, dass dieser Zustand existiert. Wenn nicht, ist das ein schwerwiegender Fehler. 4\. Wenn None, tracing::error\!("ClientCompositorData nicht für Client {:?} gefunden.", client.id()); panic\!("ClientCompositorData nicht gefunden"); (oder CompositorCoreError::ClientDataMissing zurückgeben, falls die Trait-Signatur dies erlaubt, was sie hier nicht tut). | Client::get\_data(), UserDataMap, ClientCompositorData | CompositorCoreError::ClientDataMissing (intern geloggt, Panic, da Trait Rückgabe erzwingt). |
| commit | fn commit(\&mut self, surface: \&WlSurface) | 1\. tracing::debug\!(surface\_id \=?surface.id(), "Commit für Oberfläche empfangen"); 2\. Mittels \`smithay::wayland::compositor::with\_states(surface, | states | ...)aufSurfaceDatazugreifen, das mit der Oberfläche assoziiert ist. 3.let data\_map \= states.data\_map.get::\<Arc\<Mutex\<SurfaceData\>\>\>().ok\_or(CompositorCoreError::SurfaceDataMissing(surface.clone()))?;(Fehlerbehandlung anpassen). 4.let mut surface\_data \= data\_map.lock();5. Prüfen, ob ein neuer Puffer angehängt wurde (surface\_data.pending\_buffer.is\_some()). Ggf. Validierung des Puffertyps (SHM, DMABUF \- letzteres später). 6\. Schadensverfolgungsinformationen für die Oberfläche aktualisieren basierend aufstates.cached\_state.current::\<smithay::wayland::compositor::SurfaceAttributes\>().damage..6 7\. Wenn die Oberfläche eine Rolle hat (z.B. Toplevel, Popup, Cursor), rollenspezifische Commit-Logik auslösen (z.B. Fenstermanager benachrichtigen, Cursor aktualisieren). Dies beinhaltet die Prüfung vonsurface\_data.role\_data. 8\. Wenn die Oberfläche eine synchronisierte Subsurface ist, wird ihr Zustand möglicherweise nicht sofort angewendet.surface.is\_sync\_subsurface()prüfen.10 9\. Ggf. synchronisierte Kind-Subsurfaces mittelswith\_surface\_tree\_upwardoderwith\_surface\_tree\_downward\` iterieren, um deren ausstehende Zustände anzuwenden.10 10\. Oberfläche für Neuzeichnung/Rekompilierung durch die Rendering-Pipeline markieren. |
| new\_surface | fn new\_surface(\&mut self, surface: \&WlSurface) | 1\. tracing::info\!(surface\_id \=?surface.id(), "Neue WlSurface erstellt"); 2\. let client\_id \= surface.client().expect("Oberfläche muss einen Client haben").id(); 3\. SurfaceData für diese WlSurface initialisieren und mittels \`surface.data\_map().insert\_if\_missing\_threadsafe( | Arc::new(Mutex::new(SurfaceData::new(client\_id))));speichern. 4\. Zerstörungshook mittelssmithay::wayland::compositor::add\_destruction\_hook(surface, | data\_map |
| new\_subsurface | fn new\_subsurface(\&mut self, surface: \&WlSurface, parent: \&WlSurface) | 1\. tracing::info\!(surface\_id \=?surface.id(), parent\_id \=?parent.id(), "Neue WlSubsurface erstellt"); 2\. Der Handler new\_surface wird bereits für surface aufgerufen worden sein. 3\. SurfaceData von surface aktualisieren, um auf parent zu verlinken (z.B. surface\_data.parent \= Some(parent.downgrade())). 4\. SurfaceData von parent aktualisieren, um surface in einer Liste von Kindern hinzuzufügen (z.B. parent\_surface\_data.children.push(surface.downgrade())). 5\. Die Rolle "subsurface" wird typischerweise von Smithays Compositor-Modul verwaltet, wenn wl\_subcompositor.get\_subsurface gehandhabt wird.10 | WlSurface::data\_map(), SurfaceData, Object::downgrade() | Fehler beim Zugriff auf SurfaceData. |
| destroyed | fn destroyed(\&mut self, surface: \&WlSurface) | 1\. tracing::info\!(surface\_id \=?surface.id(), "WlSurface zerstört"); 2\. Die primäre Bereinigung von SurfaceData (und anderen Benutzerdaten) wird von Smithay gehandhabt, wenn das WlSurface-Objekt zerstört und seine UserDataMap verworfen wird. 3\. Alle externen Referenzen oder Zustände (z.B. in Fenstermanagementlisten), die starke Referenzen oder IDs zu dieser Oberfläche halten, müssen hier oder über Zerstörungshooks bereinigt werden. | UserDataMap::drop (implizit) | Sicherstellen, dass alle Referenzen auf die Oberfläche bereinigt werden, um Use-after-Free zu verhindern, falls nicht durch Weak-Zeiger oder Ähnliches verwaltet. |

\*Begründung für den Wert dieser Tabelle:\* Diese Tabelle ist entscheidend, da sie die abstrakten Anforderungen des \`CompositorHandler\`-Traits in konkrete Implementierungsschritte für Entwickler übersetzt und somit direkt die Anforderung der "Ultra-Feinspezifikation" erfüllt. Sie detailliert, \*wie\* mit Smithays \`CompositorState\` und \`SurfaceData\` zu interagieren ist.

* **Implementierung: GlobalDispatch\<WlCompositor, ()\> für DesktopState** 10  
  * fn bind(state: \&mut Self, handle: \&DisplayHandle, client: \&Client, resource: New\<WlCompositor\>, global\_data: &(), data\_init: \&mut DataInit\<'\_, Self\>): Wird aufgerufen, wenn ein Client an wl\_compositor bindet.  
    * **Schritt 1**: Protokollieren der Bind-Anfrage: tracing::info\!(client\_id \=?client.id(), resource\_id \=?resource.id(), "Client bindet an wl\_compositor");  
    * **Schritt 2**: Initialisieren der Client-spezifischen Compositor-Daten, falls noch nicht geschehen. client.get\_data::\<Arc\<Mutex\<ClientCompositorData\>\>\>() prüfen und ggf. client.insert\_user\_data(|| Arc::new(Mutex::new(ClientCompositorData { compositor\_state: CompositorClientState::new() })), | | {}); (Syntax für insert\_user\_data prüfen).  
    * **Schritt 3**: data\_init.init(resource, ()); (Das () ist der UserData-Typ für das WlCompositor-Global selbst, nicht für den Client).  
    * Die Erstellung des globalen wl\_compositor-Objekts wird von CompositorState::new() gehandhabt.10  
* **Implementierung: GlobalDispatch\<WlSubcompositor, ()\> für DesktopState** 10  
  * fn bind(state: \&mut Self, handle: \&DisplayHandle, client: \&Client, resource: New\<WlSubcompositor\>, global\_data: &(), data\_init: \&mut DataInit\<'\_, Self\>): Wird aufgerufen, wenn ein Client an wl\_subcompositor bindet.  
    * **Schritt 1**: Protokollieren: tracing::info\!(client\_id \=?client.id(), resource\_id \=?resource.id(), "Client bindet an wl\_subcompositor");  
    * **Schritt 2**: data\_init.init(resource, ());  
    * Smithays CompositorState handhabt auch das globale wl\_subcompositor-Objekt intern, wenn CompositorState::new() aufgerufen wird.10

#### **2\. Datei: surface\_management.rs**

* **Zweck**: Definiert SurfaceData und zugehörige Hilfsfunktionen für die Verwaltung von Wayland-Oberflächen.  
* **Struktur: SurfaceData**  
  * Diese Struktur wird in der UserDataMap jeder WlSurface gespeichert.1  
  * Felder:  
    * pub id: uuid::Uuid (Generiert bei Erstellung, für internes Tracking, benötigt uuid-Crate mit v4- und serde-Features 14).  
    * pub role: Option\<String\> (Speichert die via give\_role zugewiesene Rolle 10).  
    * pub client\_id: wayland\_server::backend::ClientId (ID des Clients, dem die Oberfläche gehört).  
    * pub current\_buffer: Option\<wl\_buffer::WlBuffer\> (Der aktuell angehängte und committete Puffer).  
    * pub pending\_buffer: Option\<wl\_buffer::WlBuffer\> (Puffer angehängt, aber noch nicht committet).  
    * pub texture\_id: Option\<Box\<dyn RenderableTexture\>\> (Handle zur gerenderten Textur; Typ abhängig von Renderer-Abstraktion, Box\<dyn...\> für dynamische Dispatch). Muss Send \+ Sync sein, wenn SurfaceData in Arc\<Mutex\<\>\> ist.  
    * pub last\_commit\_serial: smithay::utils::Serial (Serial des letzten Commits).  
    * pub damage\_regions\_buffer\_coords: Vec\<smithay::utils::Rectangle\<i32, smithay::utils::Buffer\>\> (Akkumulierter Schaden seit dem letzten Frame, in Pufferkoordinaten).  
    * pub opaque\_region: Option\<smithay::utils::Region\<smithay::utils::Logical\>\> (Wie vom Client gesetzt).  
    * pub input\_region: Option\<smithay::utils::Region\<smithay::utils::Logical\>\> (Wie vom Client gesetzt).  
    * pub user\_data\_ext: UserDataMap (Für weitere Erweiterbarkeit durch andere Module, z.B. XDG-Shell-Daten).  
    * pub parent: Option\<wayland\_server::Weak\<wl\_surface::WlSurface\>\>  
    * pub children: Vec\<wayland\_server::Weak\<wl\_surface::WlSurface\>\>  
    * pub pre\_commit\_hooks: Vec\<Box\<dyn FnMut(\&mut DesktopState, \&wl\_surface::WlSurface) \+ Send \+ Sync\>\>  
    * pub post\_commit\_hooks: Vec\<Box\<dyn FnMut(\&mut DesktopState, \&wl\_surface::WlSurface) \+ Send \+ Sync\>\>  
    * pub destruction\_hooks: Vec\<Box\<dyn FnOnce(\&mut DesktopState, \&wl\_surface::WlSurface) \+ Send \+ Sync\>\>  
  * Methoden:  
    * pub fn new(client\_id: wayland\_server::backend::ClientId) \-\> Self  
    * pub fn set\_role(\&mut self, role: \&str) \-\> Result\<(), SurfaceRoleError\> (Fehler, wenn Rolle bereits gesetzt).  
    * pub fn get\_role(\&self) \-\> Option\<\&String\>  
    * pub fn attach\_buffer(\&mut self, buffer: Option\<wl\_buffer::WlBuffer\>, serial: smithay::utils::Serial)  
    * pub fn commit\_buffer(\&mut self) (Verschiebt pending\_buffer zu current\_buffer, löscht pending\_buffer).  
    * pub fn add\_damage\_buffer\_coords(\&mut self, damage: smithay::utils::Rectangle\<i32, smithay::utils::Buffer\>)  
    * pub fn take\_damage\_buffer\_coords(\&mut self) \-\> Vec\<smithay::utils::Rectangle\<i32, smithay::utils::Buffer\>\>  
  * **Tabelle: SurfaceData-Felder**

| Feldname | Rust-Typ | Initialwert (Beispiel) | Mutabilität | Beschreibung | Invarianten |
| :---- | :---- | :---- | :---- | :---- | :---- |
| id | uuid::Uuid | Uuid::new\_v4() | immutable (nach Init) | Eindeutiger interner Identifikator. | Muss eindeutig sein. |
| role | Option\<String\> | None | mutable (einmalig setzbar) | Zugewiesene Rolle der Oberfläche (z.B. "toplevel"). | Kann nur einmal gesetzt werden. |
| client\_id | wayland\_server::backend::ClientId | Parameter des Konstruktors | immutable | ID des besitzenden Clients. | \- |
| current\_buffer | Option\<wl\_buffer::WlBuffer\> | None | mutable | Aktuell dargestellter Puffer. | \- |
| pending\_buffer | Option\<wl\_buffer::WlBuffer\> | None | mutable | Für den nächsten Commit angehängter Puffer. | \- |
| texture\_id | Option\<Box\<dyn RenderableTexture\>\> | None | mutable | Handle zur gerenderten Textur im Renderer. | Muss mit current\_buffer synchron sein. |
| last\_commit\_serial | smithay::utils::Serial | Serial::INITIAL | mutable | Serial des letzten erfolgreichen Commits. | \- |
| damage\_regions\_buffer\_coords | Vec\<Rectangle\<i32, Buffer\>\> | vec\! | mutable | Regionen des Puffers, die sich seit dem letzten Frame geändert haben. | Koordinaten relativ zum Puffer. |
| opaque\_region | Option\<Region\<Logical\>\> | None | mutable | Vom Client definierte undurchsichtige Region. | Koordinaten in logischen Einheiten. |
| input\_region | Option\<Region\<Logical\>\> | None | mutable | Vom Client definierte Eingaberegion. | Koordinaten in logischen Einheiten. |
| user\_data\_ext | UserDataMap | UserDataMap::new() | mutable | Zusätzliche benutzerspezifische Daten. | \- |
| parent | Option\<Weak\<WlSurface\>\> | None | mutable | Schwache Referenz auf die Elternoberfläche (für Subsurfaces). | \- |
| children | Vec\<Weak\<WlSurface\>\> | vec\! | mutable | Schwache Referenzen auf Kindoberflächen. | \- |
| pre\_commit\_hooks | Vec\<Box\<dyn FnMut(\&mut DesktopState, \&WlSurface) \+ Send \+ Sync\>\> | vec\! | mutable | Callbacks vor dem Commit. | \- |
| post\_commit\_hooks | Vec\<Box\<dyn FnMut(\&mut DesktopState, \&WlSurface) \+ Send \+ Sync\>\> | vec\! | mutable | Callbacks nach dem Commit. | \- |
| destruction\_hooks | Vec\<Box\<dyn FnOnce(\&mut DesktopState, \&WlSurface) \+ Send \+ Sync\>\> | vec\! | mutable | Callbacks bei Zerstörung. | \- |

\*Begründung für den Wert dieser Tabelle:\* Diese Tabelle bietet eine klare, strukturierte Definition aller Zustände, die mit einer Wayland-Oberfläche verbunden sind. Dies ist für Entwickler unerlässlich, um deren Lebenszyklus und Eigenschaften zu verstehen. Die Unterscheidung zwischen Puffer- und Logikkoordinaten sowie die explizite Auflistung von Hooks und Regionen sind für eine präzise Implementierung entscheidend.

* **Fehler-Enum: SurfaceRoleError** (in compositor\_state.rs oder einer gemeinsamen error.rs definiert)  
  * \#  
  * Varianten:  
    * \# RoleAlreadySet { existing\_role: String, new\_role: String }  
* **Funktionen:**  
  * pub fn get\_surface\_data(surface: \&WlSurface) \-\> Option\<Arc\<Mutex\<SurfaceData\>\>\>: Ruft SurfaceData über surface.data\_map().get::\<Arc\<Mutex\<SurfaceData\>\>\>().cloned() ab.  
  * pub fn with\_surface\_data\<F, R\>(surface: \&WlSurface, f: F) \-\> Result\<R, CompositorCoreError\> where F: FnOnce(\&mut SurfaceData) \-\> R: Kapselt das Locken und Entsperren des Mutex für SurfaceData.  
    Rust  
    // Beispielimplementierung  
    pub fn with\_surface\_data\<F, R\>(  
        surface: \&WlSurface,  
        callback: F,  
    ) \-\> Result\<R, CompositorCoreError\>  
    where  
        F: FnOnce(\&mut SurfaceData) \-\> R,  
    {  
        let data\_map\_guard \= surface  
           .data\_map()  
           .get::\<Arc\<Mutex\<SurfaceData\>\>\>()  
           .ok\_or\_else(|| CompositorCoreError::SurfaceDataMissing(surface.clone()))?  
           .clone(); // Klonen des Arc, um den Borrow von data\_map() freizugeben

        let mut surface\_data\_guard \= data\_map\_guard.lock();  
        Ok(callback(\&mut \*surface\_data\_guard))  
    }

  * pub fn give\_surface\_role(surface: \&WlSurface, role: &'static str) \-\> Result\<(), SurfaceRoleError\>: Verwendet intern smithay::wayland::compositor::give\_role(surface, role). 10  
  * pub fn get\_surface\_role(surface: \&WlSurface) \-\> Option\<String\>: Verwendet intern smithay::wayland::compositor::get\_role(surface).map(String::from). 10

#### **3\. Datei: global\_objects.rs**

* **Zweck**: Zentralisiert die Erstellung der Kern-Wayland-Globals, die vom system::compositor::core-Modul verwaltet werden.  
* **Funktion: pub fn create\_core\_compositor\_globals(display\_handle: \&DisplayHandle, state: \&mut DesktopState)**  
  * **Schritt 1**: Erstellen von CompositorState: let compositor\_state \= CompositorState::new::\<DesktopState\>(display\_handle);.10  
  * Speichern von compositor\_state in state.compositor\_state.  
  * Dies registriert intern die Globals wl\_compositor (Version 6\) und wl\_subcompositor (Version 1).10  
  * Protokollieren der Erstellung dieser Globals: tracing::info\!("wl\_compositor (v6) und wl\_subcompositor (v1) Globals erstellt.");

### **C. Submodul 2: SHM-Pufferbehandlung (system::compositor::shm)**

Dieses Submodul implementiert die Unterstützung für wl\_shm, wodurch Clients Shared-Memory-Puffer mit dem Compositor teilen können.

#### **1\. Datei: shm\_state.rs**

* **Zweck**: Verwaltet das wl\_shm-Global und handhabt die Erstellung und den Zugriff auf SHM-Puffer.  
* **Struktur: ShmError**  
  * \#  
  * Varianten:  
    * \# PoolCreationFailed(String)  
    * \# BufferCreationFailed(String)  
    * \# InvalidFormat(wl\_shm::Format)  
    * \# AccessError(\#\[from\] smithay::wayland::shm::BufferAccessError)  
  * **Tabelle: ShmError-Varianten**

| Variantenname | Felder | \#\[error("...")\] Nachricht |
| :---- | :---- | :---- |
| PoolCreationFailed | (String) | "Erstellung des SHM-Pools fehlgeschlagen: {0}" |
| BufferCreationFailed | (String) | "Erstellung des SHM-Puffers fehlgeschlagen: {0}" |
| InvalidFormat | (wl\_shm::Format) | "Ungültiges SHM-Format: {0:?}" |
| AccessError | (\#\[from\] smithay::wayland::shm::BufferAccessError) | "Fehler beim Zugriff auf SHM-Puffer: {0}" |

\*Begründung für den Wert dieser Tabelle:\* Spezifische Fehler für SHM-Operationen helfen bei der Diagnose von Client-Problemen oder internen Compositor-Problemen im Zusammenhang mit Shared Memory.

* **Struktur: DesktopState (Teilweise \- Fokus auf SHM-Aspekte)**  
  * Felder:  
    * shm\_state: ShmState (aus smithay::wayland::shm) 17  
    * shm\_global: GlobalId (um das Global am Leben zu erhalten)  
* **Implementierung: ShmHandler für DesktopState** 17  
  * fn shm\_state(\&self) \-\> \&ShmState: Gibt \&self.shm\_state zurück.  
* **Implementierung: BufferHandler für DesktopState** 17  
  * fn buffer\_destroyed(\&mut self, buffer: \&wl\_buffer::WlBuffer):  
    * **Schritt 1**: Protokollieren der Pufferzerstörung: tracing::debug\!(buffer\_id \=?buffer.id(), "SHM WlBuffer zerstört");  
    * **Schritt 2**: Das Rendering-Backend benachrichtigen, dass dieser Puffer nicht mehr gültig ist und alle zugehörigen GPU-Ressourcen freigegeben werden können. Dies erfordert eine Schnittstelle zum Renderer (Details später).  
    * **Schritt 3**: Wenn ein interner Zustand diesen Puffer direkt verfolgt (z.B. in einem Cache oder einer Liste aktiver Puffer für eine Oberfläche), entfernen Sie ihn. Dies geschieht oft durch Iterieren über alle SurfaceData-Instanzen und Setzen von current\_buffer/pending\_buffer auf None, wenn sie mit dem zerstörten Puffer übereinstimmen.  
  * Die Trait BufferHandler ist nicht spezifisch für SHM-Puffer, sondern gilt für alle wl\_buffer-Instanzen. Das bedeutet, dass die Logik in buffer\_destroyed robust genug sein muss, um Puffer aus verschiedenen Quellen (SHM, zukünftig DMABUF) zu handhaben. Wenn ein Client einen wl\_buffer erstellt (z.B. über wl\_shm\_pool.create\_buffer) und diesen an eine WlSurface anhängt und committet, könnte der CompositorHandler::commit diesen WlBuffer in SurfaceData speichern und seinen Inhalt möglicherweise auf die GPU hochladen, wodurch eine Textur-ID erhalten wird. Wenn der Client später den wl\_buffer freigibt, erkennt Smithay dies und ruft BufferHandler::buffer\_destroyed auf. Die Implementierung muss dann herausfinden, wo dieser WlBuffer verwendet wurde (z.B. in SurfaceData für eine beliebige Oberfläche) und zugehörige Ressourcen (wie die GPU-Textur) bereinigen. SurfaceData muss daher WlBuffer korrekt verfolgen, und die Renderer-Abstraktion muss eine Möglichkeit bieten, Texturen freizugeben, die mit einem WlBuffer oder seiner abgeleiteten Textur-ID verbunden sind.  
* **Implementierung: GlobalDispatch\<WlShm, ()\> für DesktopState** 13  
  * fn bind(state: \&mut Self, handle: \&DisplayHandle, client: \&Client, resource: New\<WlShm\>, global\_data: &(), data\_init: \&mut DataInit\<'\_, Self\>):  
    * **Schritt 1**: Protokollieren der wl\_shm-Bindung: tracing::info\!(client\_id \=?client.id(), resource\_id \=?resource.id(), "Client bindet an wl\_shm");  
    * **Schritt 2**: data\_init.init(resource, ());  
    * Smithays ShmState handhabt das Senden der format-Ereignisse beim Binden.16 Die unterstützten Formate werden bei der Initialisierung von ShmState festgelegt.  
* **Funktion: pub fn create\_shm\_global(display\_handle: \&DisplayHandle, state: \&mut DesktopState)**  
  * **Schritt 1**: Definieren der unterstützten SHM-Formate (zusätzlich zu den standardmäßigen ARGB8888, XRGB8888). Gemäß Gesamtspezifikation sind vorerst keine weiteren spezifischen Formate erforderlich. let additional\_formats: Vec\<wl\_shm::Format\> \= vec\!;  
  * **Schritt 2**: let shm\_state \= ShmState::new::\<DesktopState\>(display\_handle, additional\_formats.clone()); (Smithays ShmState::new erwartet \&DisplayHandle und Vec\<Format\>. Die Logger-Parameter sind in neueren Smithay-Versionen oft implizit durch tracing.).17  
  * **Schritt 3**: let shm\_global \= shm\_state.global().clone(); (Die global()-Methode gibt eine GlobalId zurück, die geklont werden kann, um das Global am Leben zu erhalten).  
  * Speichern von shm\_state und shm\_global in state.  
  * Protokollieren der Erstellung des SHM-Globals und der unterstützten Formate (einschließlich der Standardformate): tracing::info\!("wl\_shm Global erstellt. Unterstützte zusätzliche Formate: {:?}. Standardformate ARGB8888 und XRGB8888 sind immer verfügbar.", additional\_formats);

#### **2\. Datei: shm\_buffer\_access.rs**

* **Zweck**: Bietet sicheren Zugriff auf Inhalte von SHM-Puffern.  
* **Funktion: pub fn with\_shm\_buffer\_contents\<F, T, E\>(buffer: \&wl\_buffer::WlBuffer, callback: F) \-\> Result\<T, ShmError\>** wobei F: FnOnce(\*const u8, usize, \&smithay::wayland::shm::BufferData) \-\> Result\<T, E\>, E: Into\<ShmError\>. (Angepasst an Smithays with\_buffer\_contents, das möglicherweise einen anderen Fehlertyp oder eine andere Callback-Signatur hat). 17  
  * **Schritt 1**: Intern smithay::wayland::shm::with\_buffer\_contents(buffer, |ptr, len, data| {... }) verwenden.  
  * **Schritt 2**: Innerhalb des Smithay-Callbacks den bereitgestellten callback(ptr, len, data) aufrufen.  
  * **Schritt 3**: BufferAccessError von Smithay in ShmError::AccessError umwandeln oder den Fehler von callback mittels .map\_err(Into::into) propagieren.  
  * **Sicherheitshinweis**: Der ptr ist nur für die Dauer des Callbacks gültig. Auf die Daten darf außerhalb dieses Bereichs nicht zugegriffen werden. Diese Funktion kapselt die Unsicherheit der Zeiger-Dereferenzierung.  
* **Wertobjekt: ShmBufferView** (optional, falls direkter, langlebiger Zugriff benötigt wird, obwohl dies aus Sicherheitsgründen im Allgemeinen nicht empfohlen wird; Callback-basierter Zugriff ist vorzuziehen)  
  * pub id: uuid::Uuid  
  * pub data: Arc\<Vec\<u8\>\> (erfordert das Kopieren des Puffers, um die Lebensdauer zu verwalten).  
  * pub metadata: smithay::wayland::shm::BufferData (aus smithay::wayland::shm).  
  * Methoden: pub fn width(\&self) \-\> i32, pub fn height(\&self) \-\> i32, pub fn stride(\&self) \-\> i32, pub fn format(\&self) \-\> wl\_shm::Format.

### **D. Submodul 3: XDG-Shell-Integration (system::compositor::xdg\_shell)**

Dieses Submodul implementiert das xdg\_shell-Protokoll zur Verwaltung moderner Desktop-Fenster (Toplevels und Popups). Das xdg\_shell-Protokoll ist komplex und umfasst mehrere interagierende Objekte (xdg\_wm\_base, xdg\_surface, xdg\_toplevel, xdg\_popup, xdg\_positioner). Smithays XdgShellState und XdgShellHandler abstrahieren einen Großteil dieser Komplexität, aber die Handler-Methoden erfordern dennoch eine signifikante Logik.7 Das Protokoll beinhaltet eine Zustandsmaschine für Oberflächen (z.B. initiale Konfiguration, ack\_configure, nachfolgende Konfigurationen).19 Anfragen wie set\_title, set\_app\_id, set\_maximized, move, resize müssen verarbeitet werden und führen oft zu neuen configure-Ereignissen, die an den Client gesendet werden.19 Popups haben eine komplizierte Positionierungslogik basierend auf xdg\_positioner.7 Daher werden die XdgShellHandler-Methoden in DesktopState umfangreich sein. Sie müssen Oberflächenzustände korrekt verwalten, mit der Fensterverwaltungsrichtlinie der Domänenschicht interagieren (hier nicht detailliert, aber ein Schnittstellenpunkt) und korrekte Wayland-Ereignisse an Clients senden. Eine robuste Fehlerbehandlung und Zustandsvalidierung sind bei der Implementierung von xdg\_shell von größter Bedeutung, um Abstürze des Compositors oder fehlverhaltende Client-Fenster zu verhindern. Smithays Zustandsverfolgung (z.B. SurfaceCachedState, ToplevelSurfaceData) hilft dabei, aber die Logik muss sie korrekt verwenden.7

#### **1\. Datei: xdg\_shell\_state.rs**

* **Zweck**: Verwaltet das xdg\_wm\_base-Global und die zugehörigen XDG-Oberflächenzustände.  
* **Struktur: XdgShellError**  
  * \#  
  * Varianten:  
    * \# InvalidSurfaceRole  
    * \# WindowHandlingError(uuid::Uuid)  
    * \#\[error("Fehler bei der Popup-Positionierung.")\] PopupPositioningError  
    * \# InvalidAckConfigureSerial(smithay::utils::Serial)  
    * \# ToplevelNotFound(uuid::Uuid)  
    * \# PopupNotFound(uuid::Uuid)  
  * **Tabelle: XdgShellError-Varianten** (Analog zu vorherigen Fehlertabellen)  
* **Struktur: DesktopState (Teilweise \- Fokus auf XDG-Shell-Aspekte)**  
  * Felder:  
    * xdg\_shell\_state: XdgShellState (aus smithay::wayland::shell::xdg) 7  
    * xdg\_shell\_global: GlobalId  
    * toplevels: std::collections::HashMap\<WlSurface, Arc\<Mutex\<ManagedToplevel\>\>\> (oder eine andere geeignete Struktur zur Verwaltung von ManagedToplevel-Instanzen, indiziert durch WlSurface oder eine interne ID).  
    * popups: std::collections::HashMap\<WlSurface, Arc\<Mutex\<ManagedPopup\>\>\>  
* **Implementierung: XdgShellHandler für DesktopState** 7  
  * fn xdg\_shell\_state(\&mut self) \-\> \&mut XdgShellState: Gibt \&mut self.xdg\_shell\_state zurück.  
  * Die Implementierung der einzelnen XdgShellHandler-Methoden wird in xdg\_handlers.rs detailliert.  
* **Implementierung: GlobalDispatch\<XdgWmBase, GlobalId\> für DesktopState** 7  
  * fn bind(state: \&mut Self, handle: \&DisplayHandle, client: \&Client, resource: New\<XdgWmBase\>, global\_data: \&GlobalId, data\_init: \&mut DataInit\<'\_, Self\>):  
    * **Schritt 1**: Protokollieren der xdg\_wm\_base-Bindung: tracing::info\!(client\_id \=?client.id(), resource\_id \=?resource.id(), "Client bindet an xdg\_wm\_base");  
    * **Schritt 2**: let shell\_client\_user\_data \= state.xdg\_shell\_state.new\_client(client); (Smithay's new\_client gibt ShellClientUserData zurück, das für die Initialisierung des XdgWmBase-Ressourcen-Userdatas verwendet werden kann). 7  
    * **Schritt 3**: data\_init.init(resource, shell\_client\_user\_data); (Assoziieren der ShellClientUserData mit der xdg\_wm\_base-Ressource).  
    * Das XdgWmBase-Global selbst sendet ein ping-Ereignis, wenn der Client nicht rechtzeitig mit pong antwortet; Smithays XdgShellState handhabt dies.7  
* **Funktion: pub fn create\_xdg\_shell\_global(display\_handle: \&DisplayHandle, state: \&mut DesktopState)**  
  * **Schritt 1**: let xdg\_shell\_state \= XdgShellState::new::\<DesktopState\>(display\_handle);.7  
  * **Schritt 2**: let xdg\_shell\_global \= xdg\_shell\_state.global().clone(); (Die global()-Methode von XdgShellState gibt die GlobalId des xdg\_wm\_base-Globals zurück).  
  * Speichern von xdg\_shell\_state und xdg\_shell\_global in state.  
  * Protokollieren der Erstellung des XDG-Shell-Globals: tracing::info\!("xdg\_wm\_base Global erstellt.");

#### **2\. Datei: toplevel\_management.rs**

* **Zweck**: Definiert Datenstrukturen und Logik, die spezifisch für XDG-Toplevel-Fenster sind.  
* **Struktur: ManagedToplevel**  
  * Diese Struktur kapselt eine smithay::wayland::shell::xdg::ToplevelSurface und fügt anwendungsspezifische Zustände und Logik hinzu.  
  * Felder:  
    * pub id: uuid::Uuid (Eindeutiger interner Identifikator).  
    * pub surface\_handle: ToplevelSurface (Das Smithay-Handle zur XDG-Toplevel-Oberfläche).7  
    * pub wl\_surface: WlSurface (Die zugrundeliegende WlSurface).  
    * pub app\_id: Option\<String\>  
    * pub title: Option\<String\>  
    * pub current\_state: ToplevelWindowState (z.B. maximiert, Vollbild, aktiv, Größe).  
    * pub pending\_state: ToplevelWindowState (Für den nächsten Configure-Zyklus).  
    * pub window\_geometry: smithay::utils::Rectangle\<i32, smithay::utils::Logical\> (Aktuelle Fenstergeometrie).  
    * pub min\_size: Option\<smithay::utils::Size\<i32, smithay::utils::Logical\>\>  
    * pub max\_size: Option\<smithay::utils::Size\<i32, smithay::utils::Logical\>\>  
    * pub parent: Option\<wayland\_server::Weak\<WlSurface\>\> (Für transiente Fenster).  
    * pub client\_provides\_decorations: bool (Abgeleitet aus Interaktion mit xdg-decoration).  
    * pub last\_configure\_serial: Option\<smithay::utils::Serial\>  
    * pub acked\_configure\_serial: Option\<smithay::utils::Serial\>  
  * Methoden:  
    * pub fn new(surface\_handle: ToplevelSurface, wl\_surface: WlSurface) \-\> Self  
    * pub fn send\_configure(\&mut self): Bereitet einen xdg\_toplevel.configure und xdg\_surface.configure vor und sendet ihn basierend auf dem pending\_state. Aktualisiert last\_configure\_serial.  
    * pub fn ack\_configure(\&mut self, serial: smithay::utils::Serial): Verarbeitet ein ack\_configure vom Client.  
    * Methoden zum Setzen von Zuständen im pending\_state (z.B. set\_maximized\_pending(bool)).  
* **Struktur: ToplevelWindowState**  
  * Felder:  
    * pub size: Option\<smithay::utils::Size\<i32, smithay::utils::Logical\>\>  
    * pub maximized: bool  
    * pub fullscreen: bool  
    * pub resizing: bool  
    * pub activated: bool  
    * pub suspended: bool (z.B. wenn minimiert oder nicht sichtbar)  
    * pub decorations: smithay::wayland::shell::xdg::decoration::XdgToplevelDecorationMode (Standard: ClientSide)  
* **Struktur: ToplevelSurfaceUserData** (Wird in WlSurface::data\_map() gespeichert, um auf ManagedToplevel zu verlinken)  
  * pub managed\_toplevel\_id: uuid::Uuid  
* **Tabelle: ManagedToplevel-Felder** (Analog zu SurfaceData-Felder-Tabelle)  
* **Tabelle: ToplevelWindowState-Felder** (Analog zu SurfaceData-Felder-Tabelle)

#### **3\. Datei: popup\_management.rs**

* **Zweck**: Definiert Datenstrukturen und Logik, die spezifisch für XDG-Popup-Fenster sind.  
* **Struktur: ManagedPopup**  
  * Kapselt eine smithay::wayland::shell::xdg::PopupSurface.  
  * Felder:  
    * pub id: uuid::Uuid  
    * pub surface\_handle: PopupSurface 7  
    * pub wl\_surface: WlSurface  
    * pub parent\_wl\_surface: wayland\_server::Weak\<WlSurface\> (Eltern-WlSurface, nicht unbedingt ein Toplevel).  
    * pub positioner\_state: smithay::wayland::shell::xdg::PositionerState 7  
    * pub current\_geometry: smithay::utils::Rectangle\<i32, smithay::utils::Logical\> (Berechnet aus Positioner und Elterngröße).  
    * pub last\_configure\_serial: Option\<smithay::utils::Serial\>  
    * pub acked\_configure\_serial: Option\<smithay::utils::Serial\>  
  * Methoden:  
    * pub fn new(surface\_handle: PopupSurface, wl\_surface: WlSurface, parent\_wl\_surface: WlSurface, positioner: PositionerState) \-\> Self  
    * pub fn send\_configure(\&mut self): Sendet xdg\_popup.configure und xdg\_surface.configure.  
    * pub fn ack\_configure(\&mut self, serial: smithay::utils::Serial)  
    * pub fn calculate\_geometry(\&self) \-\> smithay::utils::Rectangle\<i32, smithay::utils::Logical\>: Berechnet die Popup-Geometrie basierend auf positioner\_state und der Geometrie der Elternoberfläche.  
* **Struktur: PopupSurfaceUserData** (Wird in WlSurface::data\_map() gespeichert)  
  * pub managed\_popup\_id: uuid::Uuid  
* **Tabelle: ManagedPopup-Felder** (Analog zu SurfaceData-Felder-Tabelle)

#### **4\. Datei: xdg\_handlers.rs**

* **Zweck**: Detaillierte Implementierung der XdgShellHandler-Methoden für DesktopState.  
* **Implementierung XdgShellHandler für DesktopState:**  
  * fn new\_toplevel(\&mut self, surface: ToplevelSurface) 7:  
    * **Schritt 1**: Protokollieren: tracing::info\!(surface \=?surface.wl\_surface().id(), "Neues XDG Toplevel erstellt.");  
    * **Schritt 2**: let wl\_surface \= surface.wl\_surface().clone();  
    * **Schritt 3**: Erstellen einer neuen ManagedToplevel-Instanz: let managed\_toplevel \= ManagedToplevel::new(surface, wl\_surface.clone());  
    * **Schritt 4**: Speichern der managed\_toplevel.id in ToplevelSurfaceUserData und Einfügen in wl\_surface.data\_map().  
    * **Schritt 5**: self.toplevels.insert(wl\_surface.clone(), Arc::new(Mutex::new(managed\_toplevel)));  
    * **Schritt 6**: Initiale Konfiguration senden. let mut guard \= self.toplevels.get(\&wl\_surface).unwrap().lock(); guard.send\_configure();  
  * fn new\_popup(\&mut self, surface: PopupSurface, positioner: PositionerState) 7:  
    * **Schritt 1**: Protokollieren.  
    * **Schritt 2**: let wl\_surface \= surface.wl\_surface().clone();  
    * **Schritt 3**: let parent\_wl\_surface \= surface.get\_parent\_surface().expect("Popup muss eine Elternoberfläche haben.");  
    * **Schritt 4**: Erstellen ManagedPopup: let managed\_popup \= ManagedPopup::new(surface, wl\_surface.clone(), parent\_wl\_surface, positioner);  
    * **Schritt 5**: PopupSurfaceUserData in wl\_surface.data\_map() speichern.  
    * **Schritt 6**: self.popups.insert(wl\_surface.clone(), Arc::new(Mutex::new(managed\_popup)));  
    * **Schritt 7**: Initiale Konfiguration senden. let mut guard \= self.popups.get(\&wl\_surface).unwrap().lock(); guard.send\_configure();  
  * fn map\_toplevel(\&mut self, surface: \&ToplevelSurface):  
    * **Schritt 1**: Protokollieren.  
    * **Schritt 2**: let wl\_surface \= surface.wl\_surface();  
    * **Schritt 3**: let managed\_toplevel\_arc \= self.toplevels.get(wl\_surface).ok\_or\_else(|| XdgShellError::WindowHandlingError(Default::default()))?;  
    * **Schritt 4**: let mut managed\_toplevel \= managed\_toplevel\_arc.lock();  
    * **Schritt 5**: Logik für das Mapping des Toplevels ausführen (z.B. Sichtbarkeit im Fenstermanager aktualisieren, initiale Position/Größe gemäß Richtlinien festlegen, falls nicht vom Client spezifiziert).  
    * **Schritt 6**: Ggf. send\_configure aufrufen, wenn sich der Zustand durch das Mapping ändert (z.B. Aktivierung).  
  * fn ack\_configure(\&mut self, surface: WlSurface, configure: smithay::wayland::shell::xdg::XdgSurfaceConfigure) 7:  
    * **Schritt 1**: Protokollieren: tracing::debug\!(surface \=?surface.id(), serial \=?configure.serial, "XDG Surface ack\_configure empfangen.");  
    * **Schritt 2**: Herausfinden, ob es sich um ein Toplevel oder Popup handelt, basierend auf get\_role(\&surface).  
    * **Schritt 3**: Entsprechendes ManagedToplevel oder ManagedPopup aus self.toplevels oder self.popups abrufen.  
    * **Schritt 4**: managed\_entity.lock().ack\_configure(configure.serial);  
    * **Schritt 5**: Wenn dies ein ack auf eine Größenänderung war, muss der Fenstermanager ggf. Layoutanpassungen vornehmen.  
  * fn toplevel\_request\_set\_title(\&mut self, surface: \&ToplevelSurface, title: String):  
    * **Schritt 1**: let wl\_surface \= surface.wl\_surface();  
    * **Schritt 2**: let managed\_toplevel\_arc \= self.toplevels.get(wl\_surface).ok\_or\_else(...)?;  
    * **Schritt 3**: let mut managed\_toplevel \= managed\_toplevel\_arc.lock();  
    * **Schritt 4**: managed\_toplevel.title \= Some(title);  
    * **Schritt 5**: UI-Schicht benachrichtigen (z.B. über Event-Bus), um Titelleisten zu aktualisieren.  
  * (Weitere Handler für set\_app\_id, set\_maximized, unset\_maximized, set\_fullscreen, unset\_fullscreen, set\_minimized, move, resize, show\_window\_menu, destroy\_toplevel, destroy\_popup, grab\_popup, reposition\_popup usw. müssen analog implementiert werden, wobei jeweils der Zustand des entsprechenden ManagedToplevel oder ManagedPopup aktualisiert und ggf. ein neuer configure-Zyklus ausgelöst oder mit dem Input-System interagiert wird.)  
  * **Tabelle: XdgShellHandler-Kernmethodenimplementierungsdetails** (Auszug)

| Methodenname | Protokoll-Anfrage/-Ereignis | Detaillierte Schritt-für-Schritt-Logik | Wichtige Smithay-Strukturen/-Funktionen | Interaktion mit Fenstermanagement-Richtlinie | Wayland-Ereignisse an Client gesendet |
| :---- | :---- | :---- | :---- | :---- | :---- |
| new\_toplevel | xdg\_wm\_base.get\_xdg\_surface, xdg\_surface.get\_toplevel | Siehe oben. | ToplevelSurface, WlSurface::data\_map(), ManagedToplevel::new(), send\_configure() | Initiale Platzierung/Größe könnte von Richtlinie beeinflusst werden. | xdg\_toplevel.configure, xdg\_surface.configure |
| ack\_configure | xdg\_surface.ack\_configure | Siehe oben. | XdgSurfaceConfigure, ManagedToplevel/Popup::ack\_configure() | Richtlinie könnte auf Zustandsänderung reagieren (z.B. nach Größenänderung). | Keine direkt, aber Voraussetzung für weitere configure. |
| toplevel\_request\_set\_maximized | xdg\_toplevel.set\_maximized | 1\. ManagedToplevel finden. 2\. pending\_state.maximized \= true;. 3\. pending\_state.size ggf. anpassen. 4\. send\_configure() aufrufen. | ToplevelSurface, ManagedToplevel, send\_configure() | Richtlinie entscheidet, ob Maximierung erlaubt ist und wie sie umgesetzt wird (z.B. Größe des Outputs). | xdg\_toplevel.configure (mit Maximierungsstatus und neuer Größe), xdg\_surface.configure. |
| move\_request | xdg\_toplevel.move | 1\. ManagedToplevel finden. 2\. Input-System benachrichtigen, einen interaktiven Move-Grab zu starten. 3\. Seat::start\_pointer\_grab mit speziellem Grab-Handler. | ToplevelSurface, WlSeat, Serial, Seat::start\_pointer\_grab | Richtlinie kann interaktiven Move beeinflussen (z.B. Snapping). | Keine direkt während des Moves, aber Fokus-Events. |

\*Begründung für den Wert dieser Tabelle:\* Dies ist das Kernstück der XDG-Shell-Funktionalität. Detaillierte Schritte stellen sicher, dass Entwickler die Protokolllogik korrekt implementieren, einschließlich Zustandsübergängen und Interaktionen mit anderen Systemteilen.

### **E. Submodul 4: Display und Ereignisschleife (system::compositor::display\_loop)**

Dieses Submodul ist verantwortlich für die Einrichtung des Wayland-Display-Kernobjekts und dessen Integration in die calloop-Ereignisschleife. Die calloop-Ereignisschleife ist zentral für die Architektur von Smithay. Alle Ereignisquellen (Wayland-Client-FDs, libinput-FDs, Timer, ggf. D-Bus-FDs) werden bei ihr registriert, und ihre Callbacks treiben die Logik des Compositors an.1 Das Display-Objekt von Smithay stellt einen Dateideskriptor bereit, den calloop auf Lesbarkeit überwachen kann.11 Wenn der Wayland-Display-FD lesbar wird, wird Display::dispatch\_clients aufgerufen, was wiederum die entsprechenden Dispatch-Trait-Implementierungen aufruft (oft an Handler wie CompositorHandler, XdgShellHandler delegiert).1 Dies bedeutet, dass der gesamte Compositor ereignisgesteuert und größtenteils single-threaded ist (innerhalb des Haupt-calloop-Dispatches). Asynchrone Operationen, die nicht zum calloop-Modell passen (z.B. könnten einige D-Bus-Bibliotheken tokio bevorzugen), müssten sorgfältig integriert werden, möglicherweise indem sie in einem separaten Thread ausgeführt werden und über Kanäle oder benutzerdefinierte Ereignisquellen mit calloop kommunizieren. Die Leistung der Ereignisschleife (Dispatch-Latenz, Callback-Ausführungszeit) ist entscheidend für die Reaktionsfähigkeit der Benutzeroberfläche. Langlaufende Operationen in Callbacks müssen vermieden werden.

#### **1\. Datei: display\_setup.rs**

* **Zweck**: Initialisiert das Wayland Display und DisplayHandle.  
* **Struktur: ClientData** (Assoziiert mit wayland\_server::Client)  
  * pub id: uuid::Uuid (Generiert mit Uuid::new\_v4()).  
  * pub client\_name: Option\<String\> (Kann über wl\_display.sync und wl\_callback.done gesetzt werden, falls der Client es bereitstellt, oder über andere Mittel).  
  * pub user\_data: UserDataMap (aus wayland\_server::backend::UserDataMap) zum Speichern von Client-spezifischen Zuständen wie ClientCompositorData, XdgShellClientData usw..1  
  * **Tabelle: ClientData-Felder** (Analog zu SurfaceData-Felder-Tabelle).  
* **Funktion (konzeptionell, da die Initialisierung Teil von DesktopState::new ist): fn init\_wayland\_display\_and\_loop() \-\> Result\<(Display\<DesktopState\>, EventLoop\<DesktopState\>), InitError\>**  
  * **Schritt 1**: let event\_loop: EventLoop\<DesktopState\> \= EventLoop::try\_new().map\_err(|e| InitError::EventLoopCreationFailed(e.to\_string()))?;.2  
  * **Schritt 2**: let display \= Display::\<DesktopState\>::new().map\_err(|e| InitError::WaylandDisplayCreationFailed(e.to\_string()))?;.11  
  * Der DisplayHandle und LoopHandle werden in DesktopState gespeichert.  
* **Fehler-Enum: InitError**  
  * \#  
  * Varianten:  
    * \#\[error("Erstellung der Wayland-Anzeige fehlgeschlagen: {0}")\] WaylandDisplayCreationFailed(String)  
    * \#\[error("Erstellung der Ereignisschleife fehlgeschlagen: {0}")\] EventLoopCreationFailed(String)

#### **2\. Datei: event\_loop\_integration.rs**

* **Zweck**: Integriert die Wayland-Anzeige in die calloop-Ereignisschleife.  
* **Funktion: pub fn register\_wayland\_source(loop\_handle: \&LoopHandle\<DesktopState\>, display\_handle: \&DisplayHandle, desktop\_state\_accessor: impl FnMut() \-\> Arc\<Mutex\<DesktopState\>\> \+ 'static) \-\> Result\<calloop::RegistrationToken, std::io::Error\>**  
  * Die Verwaltung des mutierbaren Zugriffs auf Display innerhalb des calloop-Callbacks, während DesktopState ebenfalls mutierbar ist, erfordert sorgfältige Überlegungen zu Ownership/Borrowing. Smithay-Beispiele strukturieren dies oft, indem Display und EventLoop als Top-Level-Variablen vorhanden sind und DesktopState mutierbar an dispatch und Callbacks übergeben wird. Wenn Display Teil von DesktopState ist, könnte dies eine temporäre Entnahme oder RefCell beinhalten, falls geteilt. Für diese Spezifikation wird angenommen, dass desktop\_state.wayland\_display zugänglich und mutierbar ist. Eine gängige Methode ist die Verwendung eines Arc\<Mutex\<DesktopState\>\>, das im Callback geklont und gelockt wird, um Zugriff auf den Zustand einschließlich des DisplayHandle zu erhalten, und dann display\_handle.dispatch\_clients() aufzurufen.  
  * **Schritt 1**: Dateideskriptor der Wayland-Anzeige abrufen: let fd \= display\_handle.get\_fd(); (Die genaue Methode zum Abrufen des FD kann von der wayland-backend-Version abhängen; display.backend().poll\_fd() ist eine gängige Methode, wenn man Zugriff auf das Display-Objekt hat, nicht nur den DisplayHandle. Für calloop wird ein AsFd-kompatibler Typ benötigt.)  
  * **Schritt 2**: Erstellen einer Generic\<FileDescriptor\>-Ereignisquelle für calloop. let source \= calloop::generic::Generic::from\_fd(fd, calloop::Interest::READ, calloop::Mode::Level);  
  * **Schritt 3**: Einfügen der Quelle in die Ereignisschleife:  
    Rust  
    loop\_handle.insert\_source(source, move |event, \_metadata, shared\_data: \&mut DesktopState| {  
        // shared\_data ist hier \&mut DesktopState  
        // Zugriff auf display\_handle erfolgt über shared\_data.display\_handle  
        match shared\_data.display\_handle.dispatch\_clients(shared\_data) {  
            Ok(dispatched\_count) \=\> {  
                if dispatched\_count \> 0 {  
                    if let Err(e) \= shared\_data.display\_handle.flush\_clients() {  
                        tracing::error\!("Fehler beim Flushen der Wayland-Clients: {}", e);  
                    }  
                }  
            },  
            Err(e) \=\> {  
                tracing::error\!("Fehler beim Dispatch der Wayland-Clients: {}", e);  
            }  
        }  
        Ok(calloop::PostAction::Continue)  
    })

  .2

  * **Schritt 4**: Regelmäßiger Aufruf von display\_handle.flush\_clients() in der Ereignisschleife (z.B. nachdem alle Ereignisquellen verarbeitet wurden oder auf einem Timer), um sicherzustellen, dass alle gepufferten Wayland-Nachrichten gesendet werden.11 Dies ist entscheidend für die Reaktionsfähigkeit.

### **F. Submodul 5: Renderer-Schnittstelle (system::compositor::renderer\_interface)**

Dieses Submodul definiert abstrakte Schnittstellen für Rendering-Operationen und entkoppelt so die Kernlogik des Compositors von spezifischen Rendering-Backends (DRM/GBM, Winit/EGL). Diese Abstraktion ist entscheidend für die Unterstützung mehrerer Rendering-Backends (z.B. für den Betrieb in einem verschachtelten Fenster während der Entwicklung vs. direkter Hardwarezugriff auf einem TTY) und für die Testbarkeit. Smithays Renderer-Trait und verwandte Konzepte (z.B. Frame, Texture, Import\*-Traits) bilden eine Grundlage für diese Abstraktion.23 Durch die Definition eigener, übergeordneter Traits hier kann die Schnittstelle auf die spezifischen Bedürfnisse der Rendering-Pipeline des Compositors zugeschnitten werden (z.B. Umgang mit Ebenen, Effekten, Cursorn). Die konkreten Implementierungen dieser Traits (in system::compositor::drm\_gbm\_renderer und system::compositor::winit\_renderer – Details in späteren Teilen) werden komplex und stark von den gewählten Grafik-APIs (EGL, OpenGL ES) abhängen. Die Schadensverfolgung (Damage Tracking) ist für effizientes Rendering unerlässlich und muss in diese Renderer-Schnittstellen integriert werden; der Renderer sollte nur beschädigte Bereiche von Oberflächen neu zeichnen.

#### **1\. Datei: abstraction.rs**

* **Zweck**: Definiert Traits für Rendering-Operationen.  
* **Trait: FrameRenderer**  
  * fn new(???) \-\> Result\<Self, RendererError\> (Parameter abhängig vom Backend: z.B. DRM-Gerät, EGL-Kontext).  
  * fn render\_frame\<'a, E: RenderElement\<'a\> \+ 'a\>(\&mut self, elements: impl IntoIterator\<Item \= &'a E\>, output\_geometry: smithay::utils::Rectangle\<i32, smithay::utils::Physical\>, output\_scale: f64) \-\> Result\<(), RendererError\>.  
  * fn present\_frame(\&mut self) \-\> Result\<(), RendererError\> (Handhabt Puffertausch/Page-Flipping).  
  * fn create\_texture\_from\_shm(\&mut self, buffer: \&wl\_buffer::WlBuffer) \-\> Result\<Box\<dyn RenderableTexture\>, RendererError\>.  
  * fn create\_texture\_from\_dmabuf(\&mut self, dmabuf\_attributes: \&smithay::backend::allocator::dmabuf::Dmabuf) \-\> Result\<Box\<dyn RenderableTexture\>, RendererError\> (DMABUF-Unterstützung für spätere Teile).  
  * fn screen\_size(\&self) \-\> smithay::utils::Size\<i32, smithay::utils::Physical\>.  
* **Trait: RenderableTexture** (pub trait RenderableTexture: Send \+ Sync \+ std::fmt::Debug)  
  * fn id(\&self) \-\> uuid::Uuid (Eindeutige ID für diese Texturressource).  
  * fn bind(\&self, slot: u32) \-\> Result\<(), RendererError\> (Für Shader-Nutzung).  
  * fn width\_px(\&self) \-\> u32.  
  * fn height\_px(\&self) \-\> u32.  
  * fn format(\&self) \-\> Option\<smithay::backend::renderer::utils::Format\>. (FourCC or similar)  
* **Enum: RenderElement\<'a\>** (Konzeptionell, Smithay hat smithay::backend::renderer::element::Element)  
  * Surface { surface\_id: uuid::Uuid, texture: Arc\<dyn RenderableTexture\>, geometry: smithay::utils::Rectangle\<i32, smithay::utils::Logical\>, damage\_surface\_coords: &'a }  
  * SolidColor { color: Color, geometry: smithay::utils::Rectangle\<i32, smithay::utils::Logical\> }  
  * Cursor { texture: Arc\<dyn RenderableTexture\>, position\_logical: smithay::utils::Point\<i32, smithay::utils::Logical\>, hotspot\_logical: smithay::utils::Point\<i32, smithay::utils::Logical\> }  
* **Struktur: Color**  
  * pub r: f32 (0.0 bis 1.0)  
  * pub g: f32 (0.0 bis 1.0)  
  * pub b: f32 (0.0 bis 1.0)  
  * pub a: f32 (0.0 bis 1.0)  
* **Fehler-Enum: RendererError**  
  * \#  
  * Varianten:  
    * \# ContextCreationFailed(String)  
    * \# ShaderCompilationFailed(String)  
    * \# TextureUploadFailed(String)  
    * \#\[error("Fehler beim Puffertausch/Present: {0}")\] BufferSwapFailed(String)  
    * \# InvalidBufferType(String)  
    * \# DrmError(String) (Platzhalter für spezifischere DRM-Fehler)  
    * \#\[error("EGL-Fehler: {0}")\] EglError(String) (Platzhalter für spezifischere EGL-Fehler)  
    * \# Generic(String)  
* **Tabelle: RendererError-Varianten** (Analog zu vorherigen Fehlertabellen)  
* **Tabelle: FrameRenderer-Trait-Methoden**

| Methodenname | Signatur | Beschreibung | Hauptverantwortlichkeiten |
| :---- | :---- | :---- | :---- |
| new | fn new(???) \-\> Result\<Self, RendererError\> | Konstruktor für den Renderer. Parameter sind backend-spezifisch. | Initialisierung des Rendering-Kontexts, Laden von Shadern, etc. |
| render\_frame | fn render\_frame\<'a, E: RenderElement\<'a\> \+ 'a\>(\&mut self, elements: impl IntoIterator\<Item \= &'a E\>, output\_geometry: Rectangle\<i32, Physical\>, output\_scale: f64) \-\> Result\<(), RendererError\> | Rendert einen einzelnen Frame, bestehend aus mehreren RenderElement-Instanzen. | Iterieren über Elemente, Setzen von Transformationsmatrizen, Ausführen von Zeichenbefehlen, Schadensoptimierung. |
| present\_frame | fn present\_frame(\&mut self) \-\> Result\<(), RendererError\> | Präsentiert den gerenderten Frame auf dem Bildschirm. | Puffertausch (z.B. eglSwapBuffers), Page-Flip bei DRM. |
| create\_texture\_from\_shm | fn create\_texture\_from\_shm(\&mut self, buffer: \&wl\_buffer::WlBuffer) \-\> Result\<Box\<dyn RenderableTexture\>, RendererError\> | Erstellt eine renderbare Textur aus einem SHM-Puffer. | Zugriff auf SHM-Daten, Hochladen auf GPU, Erstellung eines RenderableTexture-Objekts. |
| create\_texture\_from\_dmabuf | fn create\_texture\_from\_dmabuf(\&mut self, dmabuf: \&Dmabuf) \-\> Result\<Box\<dyn RenderableTexture\>, RendererError\> | Erstellt eine renderbare Textur aus einem DMABUF. | Importieren von DMABUF in den Grafikstack (EGL/OpenGL), Erstellung eines RenderableTexture-Objekts. |
| screen\_size | fn screen\_size(\&self) \-\> Size\<i32, Physical\> | Gibt die aktuelle Größe des Renderziels in physischen Pixeln zurück. | Abrufen der aktuellen Ausgabegröße. |

\*Begründung für den Wert dieser Tabelle:\* Diese Tabelle definiert den Vertrag für jedes Rendering-Backend und stellt sicher, dass der Kern-Compositor konsistent mit verschiedenen Renderern (z.B. DRM/GBM, Winit) interagieren kann.

* **Tabelle: RenderableTexture-Trait-Methoden**

| Methodenname | Signatur | Beschreibung |
| :---- | :---- | :---- |
| id | fn id(\&self) \-\> uuid::Uuid | Gibt eine eindeutige ID für die Texturressource zurück. |
| bind | fn bind(\&self, slot: u32) \-\> Result\<(), RendererError\> | Bindet die Textur an einen bestimmten Texturslot für die Verwendung in Shadern. |
| width\_px | fn width\_px(\&self) \-\> u32 | Gibt die Breite der Textur in Pixeln zurück. |
| height\_px | fn height\_px(\&self) \-\> u32 | Gibt die Höhe der Textur in Pixeln zurück. |
| format | fn format(\&self) \-\> Option\<smithay::backend::renderer::utils::Format\> | Gibt das Pixelformat der Textur zurück. |

\*Begründung für den Wert dieser Tabelle:\* Abstrahiert die Texturbehandlung, was für die Verwaltung von GPU-Ressourcen, die mit Client-Puffern verbunden sind, unerlässlich ist.

## **III. Entwicklungsmodul: system::input (Libinput-basierte Eingabeverarbeitung)**

### **A. Modulübersicht**

Dieses Modul ist für die gesamte Verarbeitung von Benutzereingaben zuständig. Es initialisiert und verwaltet Eingabegeräte mittels libinput, übersetzt rohe Eingabeereignisse in ein für den Compositor und Wayland-Clients verwendbares Format und handhabt das Seat-Management, den Eingabefokus sowie die Darstellung von Zeigern/Cursorn. Die Integration von libinput erfolgt über Smithays LibinputInputBackend, das libinput in die calloop-Ereignisschleife einbindet. Smithays SeatState und SeatHandler bieten übergeordnete Abstraktionen für das Seat- und Fokusmanagement.23  
Das Eingabesystem bildet einen kritischen Pfad für die Benutzerinteraktion. Latenz oder fehlerhafte Ereignisverarbeitung hier würden die Benutzererfahrung erheblich beeinträchtigen. Die Transformation von libinput-Ereignissen in Wayland-Ereignisse, einschließlich Koordinatentransformationen und Fokuslogik, muss präzise sein. libinput liefert Low-Level-Ereignisse 25, die vom Eingabe-Stack von Smithay (LibinputInputBackend, Seat, KeyboardHandle, PointerHandle) verarbeitet und Wayland-Konzepten zugeordnet werden.26 Der Fokus bestimmt, welcher Client Eingaben empfängt; eine fehlerhafte Fokuslogik führt dazu, dass Eingaben an das falsche Fenster gehen.26 Koordinatentransformationen sind erforderlich, wenn Oberflächen skaliert oder gedreht werden. Eine gründliche Prüfung der Eingabebehandlung über verschiedene Geräte, Layouts und Fokusszenarien hinweg ist unerlässlich. Das Design muss erweiterte Eingabefunktionen wie Gesten berücksichtigen (libinput unterstützt sie 27), was möglicherweise eine komplexere Ereignisinterpretation im SeatHandler oder dedizierte Gestenmodule erfordert.  
xkbcommon ist grundlegend für die korrekte Interpretation von Tastatureingaben (Keymaps, Layouts, Modifikatoren). Sein Zustand muss pro Tastaturgerät oder pro Seat verwaltet werden.30 Rohe Keycodes von libinput sind für Anwendungen nicht direkt verwendbar. xkbcommon übersetzt Keycodes basierend auf der aktiven Keymap und dem Modifikatorstatus in Keysyms (z.B. 'A', 'Enter', 'Shift\_L') und UTF-8-Zeichen.30 Die Methode KeyboardHandle::input von Smithay verwendet typischerweise xkbcommon::State::key\_get\_syms. Der Compositor muss die korrekte XKB-Keymap laden (oft aus der Systemkonfiguration oder den Benutzereinstellungen, anfänglich ggf. Standardwerte) und einen xkbcommon::State für jede Tastatur pflegen. Änderungen des Tastaturlayouts (z.B. Sprachwechsel) erfordern eine Aktualisierung des xkbcommon::State und eine Benachrichtigung der Clients (z.B. über wl\_keyboard.keymap und wl\_keyboard.modifiers).

### **B. Submodul 1: Seat-Management (system::input::seat\_manager)**

#### **1\. Datei: seat\_state.rs**

* **Zweck**: Definiert und verwaltet SeatState und SeatHandler für Eingabefokus und die Bekanntmachung von Fähigkeiten (Capabilities).  
* **Struktur: InputError**  
  * \#  
  * Varianten:  
    * \# SeatCreationFailed(String)  
    * \# CapabilityAdditionFailed { seat\_name: String, capability: String, source: Box\<dyn std::error::Error \+ Send \+ Sync\> }  
    * \# XkbConfigError(String) (Sollte spezifischer sein, z.B. KeymapCompilationFailed)  
    * \#\[error("Libinput-Fehler: {0}")\] LibinputError(String)  
    * \# SeatNotFound(String)  
    * \# KeyboardHandleNotFound(String)  
    * \# PointerHandleNotFound(String)  
    * \# TouchHandleNotFound(String)  
  * **Tabelle: InputError-Varianten** (Analog zu vorherigen Fehlertabellen)  
* **Struktur: DesktopState (Teilweise \- Fokus auf Seat-Aspekte)**  
  * Felder:  
    * seat\_state: SeatState\<Self\> (aus smithay::input::SeatState) 26  
    * seats: std::collections::HashMap\<String, Seat\<Self\>\> (Speichert aktive Seats, indiziert nach Namen, z.B. "seat0")  
    * active\_seat\_name: Option\<String\> (Name des aktuell primären Seats)  
    * keyboards: std::collections::HashMap\<String, keyboard::xkb\_config::XkbKeyboardData\> (XKB-Daten pro Tastatur, Schlüssel könnte Gerätename oder Seat-Name sein)  
* **Implementierung: SeatHandler für DesktopState** 26  
  * type KeyboardFocus \= WlSurface;  
  * type PointerFocus \= WlSurface;  
  * type TouchFocus \= WlSurface;  
  * fn seat\_state(\&mut self) \-\> \&mut SeatState\<Self\>: Gibt \&mut self.seat\_state zurück.  
  * fn focus\_changed(\&mut self, seat: \&Seat\<Self\>, focused: Option\<\&Self::KeyboardFocus\>): (Smithays focus\_changed ist generisch; hier wird angenommen, es wird für Tastaturfokus aufgerufen oder als allgemeine Benachrichtigung, dass sich *ein* Fokus geändert hat. Für Zeiger- und Touch-Fokus werden separate Logiken in den jeweiligen Event-Handlern oder durch PointerHandle::enter/leave benötigt.)  
    * **Schritt 1**: Protokollieren der Fokusänderung: tracing::debug\!(seat\_name \= %seat.name(), new\_focus \=?focused.map(|s| s.id()), "Tastaturfokus geändert.");  
    * **Schritt 2**: Tastatur-Handle abrufen: let keyboard \= seat.get\_keyboard().ok\_or\_else(|| InputError::KeyboardHandleNotFound(seat.name().to\_string()))?; (Fehlerbehandlung anpassen).  
    * **Schritt 3**: Alten Fokus ermitteln (z.B. aus self.keyboards.get\_mut(seat.name()).unwrap().focused\_surface).  
    * **Schritt 4**: Wenn focused Some(new\_surface\_ref) ist:  
      * Wenn sich der Fokus geändert hat, keyboard.leave() an die alte fokussierte Oberfläche senden.  
      * keyboard.enter(new\_surface\_ref, &, Serial::now(), seat.get\_keyboard\_modifiers\_state()); (Aktuelle gedrückte Tasten und Modifikatoren senden).  
      * self.keyboards.get\_mut(seat.name()).unwrap().focused\_surface \= Some(new\_surface\_ref.downgrade());  
      * Interne Fenstermanagement-Zustände aktualisieren.  
    * **Schritt 5**: Wenn focused None ist:  
      * keyboard.leave() an die alte fokussierte Oberfläche senden.  
      * self.keyboards.get\_mut(seat.name()).unwrap().focused\_surface \= None;  
      * Interne Fenstermanagement-Zustände löschen/aktualisieren.  
  * fn cursor\_image(\&mut self, seat: \&Seat\<Self\>, image: smithay::input::pointer::CursorImageStatus):  
    * **Schritt 1**: Protokollieren der Cursor-Bild-Anfrage: tracing::trace\!(seat\_name \= %seat.name(), image\_status \=?image, "Cursor-Bild-Anfrage.");  
    * **Schritt 2**: Basierend auf image:  
      * CursorImageStatus::Hidden: Cursor ausblenden. Renderer anweisen, ihn nicht zu zeichnen.  
      * CursorImageStatus::Surface(cursor\_surface): Ein Client hat einen benutzerdefinierten Cursor mittels wl\_pointer.set\_cursor gesetzt.  
        * SurfaceData für cursor\_surface abrufen.  
        * Prüfen, ob cursor\_surface die Rolle "cursor" hat (mittels get\_surface\_role(\&cursor\_surface) \== Some("cursor")). 10  
        * Wenn gültig, Puffer und Hotspot aus SurfaceData oder den SurfaceAttributes der cursor\_surface abrufen.  
        * Renderer anweisen, diese Oberfläche als Cursor zu zeichnen.  
      * CursorImageStatus::Named(name): Ein Client fordert einen thematisierten Cursor an (z.B. "left\_ptr").  
        * Eine Cursor-Theming-Bibliothek (z.B. wayland-cursor oder eine benutzerdefinierte Lösung) verwenden, um die passende Cursor-Textur basierend auf name und dem aktuellen Thema zu laden.  
        * Renderer anweisen, diesen thematisierten Cursor zu zeichnen.  
    * **Schritt 3**: Renderer mit der neuen Cursor-Textur/Sichtbarkeit und dem Hotspot aktualisieren.  
* **Tabelle: SeatHandler-Methodenimplementierungsdetails für DesktopState**

| Methodenname | Signatur | Detaillierte Schritt-für-Schritt-Logik | Wichtige Smithay-Strukturen/-Funktionen | Wayland-Ereignisse gesendet |
| :---- | :---- | :---- | :---- | :---- |
| seat\_state | fn seat\_state(\&mut self) \-\> \&mut SeatState\<Self\> | \&mut self.seat\_state zurückgeben. | SeatState | Keine |
| focus\_changed | fn focus\_changed(\&mut self, seat: \&Seat\<Self\>, focused: Option\<\&WlSurface\>) | Siehe oben. | Seat, WlSurface, KeyboardHandle::enter(), KeyboardHandle::leave() | wl\_keyboard.enter, wl\_keyboard.leave, wl\_keyboard.modifiers |
| cursor\_image | fn cursor\_image(\&mut self, seat: \&Seat\<Self\>, image: CursorImageStatus) | Siehe oben. | Seat, CursorImageStatus, WlSurface (für Cursor), Renderer-API | Keine direkt, aber beeinflusst Cursor-Darstellung. |

\*Begründung für den Wert dieser Tabelle:\* Definiert, wie der Compositor auf zentrale Seat-Ereignisse wie Fokusänderungen und Cursor-Aktualisierungen reagiert, was für die grundlegende Interaktivität unerlässlich ist.

* **Funktion: pub fn create\_seat(state: \&mut DesktopState, display\_handle: \&DisplayHandle, seat\_name: String) \-\> Result\<(), InputError\>**  
  * **Schritt 1**: let seat \= state.seat\_state.new\_wl\_seat(display\_handle, seat\_name.clone());.29  
  * **Schritt 2**: Hinzufügen von Fähigkeiten (normalerweise nachdem das libinput-Backend aktiv ist und Geräte bekannt sind):  
    * Tastatur:  
      * let xkb\_config \= keyboard::xkb\_config::XkbConfig { rules: None, model: None, layout: Some("us".into()), variant: None, options: None }; (Standardkonfiguration, anpassbar).  
      * let keyboard\_handle \= seat.add\_keyboard(xkb\_config, 200, 25).map\_err(|e| InputError::CapabilityAdditionFailed { seat\_name: seat\_name.clone(), capability: "keyboard".to\_string(), source: Box::new(e) })?;.26  
      * Erstellen und Speichern von XkbKeyboardData für diese Tastatur/diesen Seat in state.keyboards.  
    * Zeiger: let \_pointer\_handle \= seat.add\_pointer().map\_err(|e| InputError::CapabilityAdditionFailed { seat\_name: seat\_name.clone(), capability: "pointer".to\_string(), source: Box::new(e) })?;.26  
    * Touch: let \_touch\_handle \= seat.add\_touch().map\_err(|e| InputError::CapabilityAdditionFailed { seat\_name: seat\_name.clone(), capability: "touch".to\_string(), source: Box::new(e) })?;.26  
  * **Schritt 3**: Speichern des Seat-Objekts: state.seats.insert(seat\_name.clone(), seat);.  
  * **Schritt 4**: Wenn dies der erste/primäre Seat ist, state.active\_seat\_name \= Some(seat\_name);.  
  * Protokollieren der Seat-Erstellung und Fähigkeitserweiterung.  
  * Ok(()) zurückgeben.

### **C. Submodul 2: Libinput-Backend (system::input::libinput\_handler)**

#### **1\. Datei: backend\_config.rs**

* **Zweck**: Initialisiert und konfiguriert das LibinputInputBackend.  
* **Struktur: LibinputSessionInterface** (Wrapper für Session-Trait zur Bereitstellung von input::LibinputInterface) 25  
  * Felder: session\_signal: calloop::LoopSignal (oder ähnlicher Mechanismus, um Sitzungsänderungen an die Ereignisschleife zu signalisieren).  
  * Implementiert input::LibinputInterface zum Öffnen/Schließen eingeschränkter Geräte über ein Session-Objekt (z.B. smithay::backend::session::direct::DirectSession oder smithay::backend::session::logind::LogindSession – Details zum Sitzungsmanagement folgen in späteren Teilen, aber diese Schnittstelle wird jetzt benötigt).23  
* **Funktion: pub fn init\_libinput\_backend(event\_loop\_handle: \&LoopHandle\<DesktopState\>, session\_interface: LibinputSessionInterface) \-\> Result\<LibinputInputBackend, InputError\>**  
  * **Schritt 1**: Erstellen eines libinput::Libinput-Kontexts: let mut libinput\_context \= Libinput::new\_from\_path(session\_interface);.25 Die session\_interface wird von libinput zum Öffnen/Schließen von Gerätedateien verwendet.  
  * **Schritt 2**: Zuweisen eines Seats zum Kontext: libinput\_context.udev\_assign\_seat("seat0").map\_err(|e| InputError::LibinputError(format\!("Zuweisung zu udev seat0 fehlgeschlagen: {:?}", e)))?;.32  
  * **Schritt 3**: let libinput\_backend \= LibinputInputBackend::new(libinput\_context.into()); (Die into() Konvertierung ist möglicherweise nicht direkt, ggf. LibinputInputBackend::new(libinput\_context, logger\_oder\_tracing\_span))..25  
  * Rückgabe des libinput\_backend. Die Registrierung als Ereignisquelle erfolgt separat.

#### **2\. Datei: event\_dispatcher.rs**

* **Zweck**: Verarbeitet InputEvent\<LibinputInputBackend\> und leitet an spezifische Handler weiter.  
* **Funktion: pub fn process\_input\_event(desktop\_state: \&mut DesktopState, event: InputEvent\<LibinputInputBackend\>, seat\_name: \&str)** (Aufgerufen vom calloop-Callback)  
  * **Schritt 1**: Aktiven Seat abrufen: let seat \= desktop\_state.seats.get(seat\_name).ok\_or\_else(|| InputError::SeatNotFound(seat\_name.to\_string()))?; (Fehlerbehandlung anpassen).  
  * **Schritt 2**: match event {... } 27  
    * InputEvent::Keyboard { event }: keyboard::key\_event\_translator::handle\_keyboard\_key\_event(desktop\_state, seat, event, seat\_name);  
    * InputEvent::PointerMotion { event }: pointer::pointer\_event\_translator::handle\_pointer\_motion\_event(desktop\_state, seat, event);  
    * InputEvent::PointerMotionAbsolute { event }: pointer::pointer\_event\_translator::handle\_pointer\_motion\_absolute\_event(desktop\_state, seat, event);  
    * InputEvent::PointerButton { event }: pointer::pointer\_event\_translator::handle\_pointer\_button\_event(desktop\_state, seat, event);  
    * InputEvent::PointerAxis { event }: pointer::pointer\_event\_translator::handle\_pointer\_axis\_event(desktop\_state, seat, event);  
    * InputEvent::TouchDown { event }: touch::touch\_event\_translator::handle\_touch\_down\_event(desktop\_state, seat, event);  
    * InputEvent::TouchUp { event }: touch::touch\_event\_translator::handle\_touch\_up\_event(desktop\_state, seat, event);  
    * InputEvent::TouchMotion { event }: touch::touch\_event\_translator::handle\_touch\_motion\_event(desktop\_state, seat, event);  
    * InputEvent::TouchFrame { event }: touch::touch\_event\_translator::handle\_touch\_frame\_event(desktop\_state, seat);  
    * InputEvent::TouchCancel { event }: touch::touch\_event\_translator::handle\_touch\_cancel\_event(desktop\_state, seat);  
    * InputEvent::GesturePinchBegin/Update/End, InputEvent::GestureSwipeBegin/Update/End usw. 27: Anfänglich diese Ereignisse protokollieren: tracing::debug\!("Gestenereignis empfangen: {:?}", event);. Vollständige Gestenbehandlung ist komplex und könnte Teil einer späteren Spezifikationsphase sein.  
    * InputEvent::DeviceAdded { device }:  
      * Protokollieren der Gerätehinzufügung: tracing::info\!("Eingabegerät hinzugefügt: {} ({:?})", device.name(), device.id());  
      * Seat-Fähigkeiten aktualisieren, falls erforderlich (z.B. wenn eine Tastatur angeschlossen wurde und der Seat noch keine hatte). device.has\_capability(DeviceCapability::Keyboard) usw. prüfen.28  
    * InputEvent::DeviceRemoved { device }:  
      * Protokollieren der Geräteentfernung: tracing::info\!("Eingabegerät entfernt: {} ({:?})", device.name(), device.id());  
      * Seat-Fähigkeiten aktualisieren.  
    * Andere Ereignisse (ToolAxis, ToolTip, TabletPadButton usw.): Protokollieren. Vollständige Tablet-Unterstützung ist umfangreich.  
* **Tabelle: InputEvent-Variantenverarbeitung**

| InputEvent-Variante | Zugehörige Handler-Funktion in event\_dispatcher.rs | Kurze Logikbeschreibung |
| :---- | :---- | :---- |
| Keyboard { event } | keyboard::key\_event\_translator::handle\_keyboard\_key\_event | Übersetzt Keycode in Keysym/UTF-8, aktualisiert Modifikatoren, sendet an Client. |
| PointerMotion { event } | pointer::pointer\_event\_translator::handle\_pointer\_motion\_event | Aktualisiert Cursorposition, sendet Motion-Ereignis an fokussierte Oberfläche. |
| PointerMotionAbsolute { event } | pointer::pointer\_event\_translator::handle\_pointer\_motion\_absolute\_event | Wie PointerMotion, aber mit absoluten Koordinaten. |
| PointerButton { event } | pointer::pointer\_event\_translator::handle\_pointer\_button\_event | Sendet Button-Ereignis, löst ggf. Fokusänderung oder Fenstermanagement-Aktionen aus. |
| PointerAxis { event } | pointer::pointer\_event\_translator::handle\_pointer\_axis\_event | Sendet Scroll-Ereignis (vertikal/horizontal). |
| TouchDown { event } | touch::touch\_event\_translator::handle\_touch\_down\_event | Startet einen Touchpunkt, sendet Down-Ereignis an Oberfläche unter dem Punkt. |
| TouchUp { event } | touch::touch\_event\_translator::handle\_touch\_up\_event | Beendet einen Touchpunkt, sendet Up-Ereignis. |
| TouchMotion { event } | touch::touch\_event\_translator::handle\_touch\_motion\_event | Aktualisiert Position eines Touchpunkts, sendet Motion-Ereignis. |
| TouchFrame { event } | touch::touch\_event\_translator::handle\_touch\_frame\_event | Signalisiert Ende eines Satzes von Touch-Ereignissen. |
| TouchCancel { event } | touch::touch\_event\_translator::handle\_touch\_cancel\_event | Signalisiert Abbruch der Touch-Interaktion. |
| DeviceAdded { device } | Direkt in process\_input\_event | Protokolliert neues Gerät, aktualisiert ggf. Seat-Fähigkeiten. |
| DeviceRemoved { device } | Direkt in process\_input\_event | Protokolliert entferntes Gerät, aktualisiert ggf. Seat-Fähigkeiten. |
| Gesture\* | Direkt in process\_input\_event | Protokolliert Gestenereignisse für spätere Implementierung. |

\*Begründung für den Wert dieser Tabelle:\* Bietet eine klare Zuordnung von rohen Smithay-Eingabeereignissen zu den spezifischen Verarbeitungsfunktionen innerhalb des Eingabesystems.

### **D. Submodul 3: Tastaturverarbeitung (system::input::keyboard)**

#### **1\. Datei: xkb\_config.rs**

* **Zweck**: Verwaltet XKB-Keymap und \-Status für Tastaturen.  
* **Struktur: XkbKeyboardData**  
  * pub context: xkbcommon::xkb::Context  
  * pub keymap: xkbcommon::xkb::Keymap  
  * pub state: xkbcommon::xkb::State  
  * pub repeat\_timer: Option\<calloop::TimerHandle\> (Für Tastenwiederholung)  
  * pub repeat\_info: Option\<(u32, KeyState, std::time::Duration, std::time::Duration)\> (Keycode, Zustand, anfängliche Verzögerung, Wiederholungsintervall)  
  * focused\_surface\_on\_seat: Option\<wayland\_server::Weak\<WlSurface\>\> (Cache des aktuellen Fokus für diesen Seat/diese Tastatur)  
  * repeat\_key\_serial: Option\<Serial\> (Serial des Tastenereignisses, das die Wiederholung ausgelöst hat)  
* **Tabelle: XkbKeyboardData-Felder** (Analog zu SurfaceData-Felder-Tabelle)  
* **Funktion: pub fn new\_xkb\_keyboard\_data(config: \&smithay::input::keyboard::XkbConfig\<'\_\>) \-\> Result\<XkbKeyboardData, InputError\>**  
  * **Schritt 1**: let context \= xkbcommon::xkb::Context::new(xkbcommon::xkb::CONTEXT\_NO\_FLAGS);  
  * **Schritt 2**: Erstellen von xkbcommon::xkb::RuleNames aus config.rules, config.model, config.layout, config.variant (oder Standardwerte wie "evdev", "pc105", "us", "").  
  * **Schritt 3**: let keymap \= xkbcommon::xkb::Keymap::new\_from\_names(\&context, \&rules, xkbcommon::xkb::KEYMAP\_COMPILE\_NO\_FLAGS).map\_err(|\_| InputError::XkbConfigError("Keymap-Kompilierung fehlgeschlagen".to\_string()))?;.30  
  * **Schritt 4**: let state \= xkbcommon::xkb::State::new(\&keymap);.30  
  * Gibt XkbKeyboardData zurück.  
* **Funktion: pub fn update\_xkb\_state\_from\_modifiers(xkb\_state: \&mut xkbcommon::xkb::State, modifiers\_state: \&smithay::input::keyboard::ModifiersState) \-\> bool**  
  * Ruft xkb\_state.update\_mask(modifiers\_state.depressed, modifiers\_state.latched, modifiers\_state.locked, modifiers\_state.layout\_depressed, modifiers\_state.layout\_latched, modifiers\_state.layout\_locked) auf.30  
  * Gibt true zurück, wenn sich der Zustand geändert hat, andernfalls false.

#### **2\. Datei: key\_event\_translator.rs**

* **Zweck**: Übersetzt KeyboardKeyEvent in Keysyms/UTF-8 und leitet an den Client weiter.  
* **Funktion: pub fn handle\_keyboard\_key\_event(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>, event: KeyboardKeyEvent\<LibinputInputBackend\>, seat\_name: \&str)**  
  * **Schritt 1**: Tastatur-Handle abrufen: let keyboard\_handle \= seat.get\_keyboard().ok\_or\_else(|| { tracing::warn\!("Kein Keyboard-Handle für Seat {} bei Key-Event.", seat\_name); InputError::KeyboardHandleNotFound(seat\_name.to\_string()) })?;  
  * **Schritt 2**: XkbKeyboardData für diesen Seat/diese Tastatur abrufen: let xkb\_data \= desktop\_state.keyboards.get\_mut(seat\_name).ok\_or\_else(|| { tracing::warn\!("Keine XKB-Daten für Seat {} bei Key-Event.", seat\_name); InputError::XkbConfigError("XKB-Daten nicht gefunden".to\_string()) })?;  
  * **Schritt 3**: xkbcommon::State aktualisieren: let key\_direction \= match event.state() { KeyState::Pressed \=\> xkbcommon::xkb::KeyDirection::Down, KeyState::Released \=\> xkbcommon::xkb::KeyDirection::Up, }; xkb\_data.state.update\_key(event.key\_code(), key\_direction);.30  
  * **Schritt 4**: ModifiersState von xkb\_data.state abrufen: let smithay\_mods\_state \= smithay::input::keyboard::ModifiersState { depressed: xkb\_data.state.serialize\_mods(xkbcommon::xkb::STATE\_MODS\_DEPRESSED), latched: xkb\_data.state.serialize\_mods(xkbcommon::xkb::STATE\_MODS\_LATCHED), locked: xkb\_data.state.serialize\_mods(xkbcommon::xkb::STATE\_MODS\_LOCKED), layout\_effective: xkb\_data.state.serialize\_layout(xkbcommon::xkb::STATE\_LAYOUT\_EFFECTIVE),..Default::default() };  
  * **Schritt 5**: KeyboardHandle über Modifikatoränderungen informieren: keyboard\_handle.modifiers(\&smithay\_mods\_state, event.serial());  
  * **Schritt 6**: Wenn event.state() \== KeyState::Pressed:  
    * let keysym \= xkb\_data.state.key\_get\_one\_sym(event.key\_code());  
    * let utf8 \= xkb\_data.state.key\_get\_utf8(event.key\_code());  
    * Protokollieren von Keysym und UTF-8: tracing::trace\!(keycode \= event.key\_code(), keysym \=?keysym, utf8 \= %utf8, "Taste gedrückt");  
    * keyboard\_handle.input(event.key\_code(), KeyState::Pressed, Some(keysym), if utf8.is\_empty() { None } else { Some(utf8) }, event.time(), event.serial());  
    * Tastenwiederholung einrichten/abbrechen unter Verwendung von xkb\_data.repeat\_timer und calloop::Timer. Die Wiederholungsrate und \-verzögerung kommen von keyboard\_handle.repeat\_info().  
      * Wenn eine Taste gedrückt wird, die Wiederholung unterstützt:  
        * Vorhandenen repeat\_timer abbrechen.  
        * Neuen Timer mit anfänglicher Verzögerung starten. Callback des Timers sendet das Key-Event erneut und plant sich selbst mit dem Wiederholungsintervall neu, bis die Taste losgelassen wird oder der Fokus wechselt.  
        * xkb\_data.repeat\_info und xkb\_data.repeat\_key\_serial speichern.  
  * **Schritt 7**: Wenn event.state() \== KeyState::Released:  
    * keyboard\_handle.input(event.key\_code(), KeyState::Released, None, None, event.time(), event.serial());  
    * Tastenwiederholung abbrechen, falls diese Taste die Wiederholung ausgelöst hat. xkb\_data.repeat\_timer.take().map(|t| t.cancel()); xkb\_data.repeat\_info \= None;

#### **3\. Datei: focus\_handler\_keyboard.rs**

* **Zweck**: Verwaltet den Tastaturfokus für WlSurface.  
* **Funktion: pub fn set\_keyboard\_focus(desktop\_state: \&mut DesktopState, seat\_name: \&str, surface: Option\<\&WlSurface\>, serial: Serial)**  
  * **Schritt 1**: Seat und KeyboardHandle abrufen. let seat \= desktop\_state.seats.get(seat\_name).ok\_or\_else(...)?.clone(); (Klonen des Seat-Handles). let keyboard\_handle \= seat.get\_keyboard().ok\_or\_else(...)?.clone(); (Klonen des KeyboardHandle).  
  * **Schritt 2**: XkbKeyboardData für den Seat abrufen. let xkb\_data \= desktop\_state.keyboards.get\_mut(seat\_name).ok\_or\_else(...)?;  
  * **Schritt 3**: Alten Fokus ermitteln: let old\_focus\_weak \= xkb\_data.focused\_surface\_on\_seat.clone();  
  * **Schritt 4**: Wenn surface Some(new\_focus\_ref) ist:  
    * Wenn old\_focus\_weak.as\_ref().and\_then(|w| w.upgrade()).as\_ref()\!= Some(\&new\_focus\_ref), dann hat sich der Fokus geändert.  
      * Wenn alter Fokus existierte und noch gültig ist (old\_focus.upgrade()), keyboard\_handle.leave(old\_focus.upgrade().unwrap(), serial); senden.  
      * keyboard\_handle.enter(new\_focus\_ref, \&xkb\_data.state.keycodes\_pressed().collect::\<Vec\<\_\>\>(), serial, seat.get\_keyboard\_modifiers\_state()); (Aktuell gedrückte Tasten und Modifikatoren senden).  
      * xkb\_data.focused\_surface\_on\_seat \= Some(new\_focus\_ref.downgrade());  
  * **Schritt 5**: Wenn surface None ist:  
    * Wenn old\_focus\_weak.as\_ref().and\_then(|w| w.upgrade()).is\_some(), keyboard\_handle.leave(old\_focus\_weak.unwrap().upgrade().unwrap(), serial); senden.  
    * xkb\_data.focused\_surface\_on\_seat \= None;  
  * **Schritt 6**: keyboard\_handle.set\_focus(surface, serial);.31

### **E. Submodul 4: Zeigerverarbeitung (system::input::pointer)**

#### **1\. Datei: pointer\_event\_translator.rs**

* **Zweck**: Verarbeitet Zeigerereignisse und leitet sie weiter.  
* **Funktion: pub fn handle\_pointer\_motion\_event(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>, event: PointerMotionEvent\<LibinputInputBackend\>)**  
  * **Schritt 1**: PointerHandle abrufen: let pointer\_handle \= seat.get\_pointer().ok\_or\_else(...)?.clone();  
  * **Schritt 2**: Globale Cursorposition aktualisieren (z.B. in DesktopState speichern, wenn nicht von PointerHandle verwaltet). Die event.delta() oder event.delta\_unaccel() können verwendet werden, um die neue globale Position zu berechnen.  
  * **Schritt 3**: Neuen Zeigerfokus bestimmen basierend auf der neuen globalen Cursorposition. Dies erfordert eine Iteration über sichtbare Toplevel-Oberflächen und deren Eingaberegionen unter Berücksichtigung der Stapelreihenfolge. let (new\_focus\_surface, surface\_local\_coords) \= find\_surface\_at\_global\_coords(\&desktop\_state.toplevels, global\_cursor\_pos);  
  * **Schritt 4**: Fokus- und Enter/Leave-Ereignisse senden: update\_pointer\_focus\_and\_send\_motion(desktop\_state, seat, \&pointer\_handle, new\_focus\_surface, surface\_local\_coords, event.time(), event.serial());  
  * **Schritt 5**: Renderer-Cursorposition aktualisieren.  
* **Funktion: pub fn handle\_pointer\_motion\_absolute\_event(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>, event: PointerMotionAbsoluteEvent\<LibinputInputBackend\>)**  
  * Ähnlich wie handle\_pointer\_motion\_event, aber verwendet absolute Koordinaten. event.x\_transformed(output\_width), event.y\_transformed(output\_height) können verwendet werden, um globale Bildschirmkoordinaten zu erhalten.27 (Benötigt die Größe des Outputs, auf dem sich das Gerät befindet).  
* **Funktion: pub fn handle\_pointer\_button\_event(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>, event: PointerButtonEvent\<LibinputInputBackend\>)**  
  * **Schritt 1**: PointerHandle abrufen.  
  * **Schritt 2**: let wl\_button\_state \= match event.button\_state() { ButtonState::Pressed \=\> wl\_pointer::ButtonState::Pressed, ButtonState::Released \=\> wl\_pointer::ButtonState::Released, }; pointer\_handle.button(event.button(), wl\_button\_state, event.serial(), event.time());  
  * **Schritt 3**: Wenn Taste gedrückt (Pressed):  
    * Tastaturfokus gemäß Fenstermanagement-Richtlinie ändern (z.B. Click-to-Focus). focus\_handler\_keyboard::set\_keyboard\_focus(...) aufrufen mit der Oberfläche unter dem Cursor.  
    * Fenstermanagement-Interaktionen behandeln (z.B. Move/Resize starten, wenn auf Dekoration geklickt wird). Dies kann das Starten eines Grabs beinhalten (seat.start\_pointer\_grab(...)).  
* **Funktion: pub fn handle\_pointer\_axis\_event(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>, event: PointerAxisEvent\<LibinputInputBackend\>)**  
  * **Schritt 1**: PointerHandle abrufen.  
  * **Schritt 2**: Achsenquelle bestimmen: let source \= match event.axis\_source() { Some(libinput::event::pointer::AxisSource::Wheel) \=\> wl\_pointer::AxisSource::Wheel, Some(libinput::event::pointer::AxisSource::Finger) \=\> wl\_pointer::AxisSource::Finger, Some(libinput::event::pointer::AxisSource::Continuous) \=\> wl\_pointer::AxisSource::Continuous, \_ \=\> wl\_pointer::AxisSource::Wheel, // Fallback };  
  * **Schritt 3**: Diskrete Scroll-Schritte: let v\_discrete \= event.axis\_value\_discrete(PointerAxis::Vertical); let h\_discrete \= event.axis\_value\_discrete(PointerAxis::Horizontal);  
  * **Schritt 4**: Kontinuierlicher Scroll-Wert: let v\_continuous \= event.axis\_value(PointerAxis::Vertical); let h\_continuous \= event.axis\_value(PointerAxis::Horizontal);  
  * **Schritt 5**: Wenn vertikales Scrollen (v\_discrete.is\_some() | | v\_continuous\!= 0.0): pointer\_handle.axis(wl\_pointer::Axis::VerticalScroll, source, v\_discrete, v\_continuous, event.serial(), event.time());  
  * **Schritt 6**: Wenn horizontales Scrollen (h\_discrete.is\_some() | | h\_continuous\!= 0.0): pointer\_handle.axis(wl\_pointer::Axis::HorizontalScroll, source, h\_discrete, h\_continuous, event.serial(), event.time());

#### **2\. Datei: focus\_handler\_pointer.rs**

* **Zweck**: Verwaltet Zeigerfokus, Enter/Leave-Ereignisse.  
* **Funktion: pub fn update\_pointer\_focus\_and\_send\_motion(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>, pointer\_handle: \&PointerHandle\<DesktopState\>, new\_focus\_surface: Option\<WlSurface\>, surface\_local\_coords: Point\<f64, Logical\>, time: u32, serial: Serial)**  
  * **Schritt 1**: Aktuellen Fokus vom pointer\_handle abrufen: let old\_focus\_surface \= pointer\_handle.current\_focus();  
  * **Schritt 2**: Wenn new\_focus\_surface\!= old\_focus\_surface.as\_ref():  
    * Wenn old\_focus\_surface existierte, pointer\_handle.leave(old\_focus\_surface.as\_ref().unwrap(), serial, time); senden.  
    * Wenn new\_focus\_surface existiert, pointer\_handle.enter(new\_focus\_surface.as\_ref().unwrap(), serial, time, surface\_local\_coords); senden.  
    * Internen Fokus des PointerHandle aktualisieren (Smithay macht dies oft implizit bei enter).  
  * **Schritt 3**: Wenn new\_focus\_surface existiert (auch wenn es dasselbe wie der alte Fokus ist), pointer\_handle.motion(new\_focus\_surface.as\_ref().unwrap(), time, serial, surface\_local\_coords); senden.

#### **3\. Datei: cursor\_updater.rs**

* **Zweck**: Behandelt die Logik von SeatHandler::cursor\_image.  
* Die Logik ist bereits oben in der Implementierung von SeatHandler::cursor\_image detailliert. Diese Datei würde Hilfsfunktionen enthalten, falls diese Logik zu komplex wird, z.B. für das Laden von Cursor-Themen.

### **F. Submodul 5: Touch-Verarbeitung (system::input::touch)**

#### **1\. Datei: touch\_event\_translator.rs**

* **Zweck**: Verarbeitet Touch-Ereignisse und leitet sie weiter.  
* **Funktion: pub fn handle\_touch\_down\_event(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>, event: TouchDownEvent\<LibinputInputBackend\>)**  
  * **Schritt 1**: TouchHandle abrufen: let touch\_handle \= seat.get\_touch().ok\_or\_else(...)?.clone();  
  * **Schritt 2**: Fokussierte Oberfläche für diesen Touchpunkt bestimmen. Dies kann die Oberfläche unter dem Touchpunkt sein. let (focused\_surface, surface\_local\_coords) \= find\_surface\_at\_global\_coords(\&desktop\_state.toplevels, event.position\_transformed(output\_size)); (Benötigt Output-Größe für Transformation).  
  * **Schritt 3**: Wenn eine Oberfläche anvisiert wird:  
    * touch\_handle.down(focused\_surface.as\_ref().unwrap(), event.serial(), event.time(), event.slot().unwrap(), surface\_local\_coords); (Smithay's slot() gibt Option\<TouchSlot\>).  
* **Funktion: pub fn handle\_touch\_up\_event(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>, event: TouchUpEvent\<LibinputInputBackend\>)**  
  * **Schritt 1**: TouchHandle abrufen.  
  * touch\_handle.up(event.serial(), event.time(), event.slot().unwrap());  
* **Funktion: pub fn handle\_touch\_motion\_event(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>, event: TouchMotionEvent\<LibinputInputBackend\>)**  
  * **Schritt 1**: TouchHandle abrufen.  
  * **Schritt 2**: Oberfläche abrufen, die aktuell von diesem Touch-Slot (event.slot().unwrap()) anvisiert wird (muss im TouchHandle oder DesktopState pro Slot gespeichert werden).  
  * **Schritt 3**: Koordinaten transformieren.  
  * touch\_handle.motion(focused\_surface\_for\_slot.as\_ref().unwrap(), event.serial(), event.time(), event.slot().unwrap(), surface\_local\_coords\_for\_slot);  
* **Funktion: pub fn handle\_touch\_frame\_event(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>)**  
  * TouchHandle abrufen.  
  * touch\_handle.frame();  
* **Funktion: pub fn handle\_touch\_cancel\_event(desktop\_state: \&mut DesktopState, seat: \&Seat\<DesktopState\>)**  
  * TouchHandle abrufen.  
  * touch\_handle.cancel();

#### **2\. Datei: focus\_handler\_touch.rs**

* **Zweck**: Verwaltet den Touch-Fokus.  
* Die Logik zur Bestimmung des Touch-Fokus ist ähnlich der des Zeigerfokus, aber pro Touchpunkt/Slot. TouchHandle selbst hat keine expliziten enter/leave-Methoden wie PointerHandle; der Fokus ist implizit in dem Oberflächenargument für down/motion. Der Zustand, welche Oberfläche von welchem Slot berührt wird, muss im DesktopState oder einer benutzerdefinierten Struktur, die mit dem TouchHandle assoziiert ist, verwaltet werden.

## **IV. Schlussfolgerungen**

Dieser erste Teil der Ultra-Feinspezifikation für die Systemschicht legt ein detailliertes Fundament für die Kernkomponenten des Wayland-Compositors und der Eingabeverarbeitung. Durch die systematische Zerlegung in Module und Submodule, die präzise Definition von Datenstrukturen, Schnittstellen und Fehlerfällen sowie die schrittweise Detaillierung der Implementierungslogik wird eine solide Basis für die nachfolgende Entwicklung geschaffen.  
Die enge Integration mit dem Smithay-Toolkit und dessen Designprinzipien, insbesondere das Handler-Trait-Muster und die zentrale Zustandsverwaltung in DesktopState, prägen die Struktur der Implementierung maßgeblich. Die Spezifikation berücksichtigt die Notwendigkeit einer klaren Abstraktion der Renderer-Schnittstelle und einer robusten Fehlerbehandlung mittels thiserror. Die detaillierte Ausarbeitung der XDG-Shell-Handler und der Input-Event-Übersetzer adressiert die Komplexität dieser Protokolle und Interaktionen.  
Die hier spezifizierten Module system::compositor und system::input sind grundlegend für jede weitere Funktionalität der Desktop-Umgebung. Ihre korrekte und performante Implementierung gemäß dieser Spezifikation ist entscheidend für die Stabilität, Reaktionsfähigkeit und das Gesamterlebnis des Systems. Die identifizierten Abhängigkeiten und Interaktionen zwischen diesen Modulen sowie die Notwendigkeit einer sorgfältigen Zustandsverwaltung wurden hervorgehoben, um potenziellen Herausforderungen proaktiv zu begegnen.  
Mit dieser Spezifikation sind Entwickler in der Lage, die Implementierung von Teil 1/4 der Systemschicht mit einem hohen Grad an Klarheit und Präzision zu beginnen. Die nachfolgenden Teile werden auf dieser Grundlage aufbauen und weitere systemnahe Dienste und Protokolle detaillieren.

#### **Referenzen**

1. smithay \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/](https://smithay.github.io/smithay/)  
2. Smithay/calloop: A callback-based Event Loop \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/Smithay/calloop](https://github.com/Smithay/calloop)  
3. calloop \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/calloop/](https://docs.rs/calloop/)  
4. smithay \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/smithay](https://docs.rs/smithay)  
5. View github: Smithay/smithay | OpenText Core SCA \- Debricked, Zugriff am Mai 14, 2025, [https://debricked.com/select/package/github-Smithay/smithay](https://debricked.com/select/package/github-Smithay/smithay)  
6. CompositorHandler in smithay::wayland::compositor \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/wayland/compositor/trait.CompositorHandler.html](https://smithay.github.io/smithay/smithay/wayland/compositor/trait.CompositorHandler.html)  
7. smithay::wayland::shell::xdg \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/wayland/shell/xdg/index.html](https://smithay.github.io/smithay/smithay/wayland/shell/xdg/index.html)  
8. Multiple error types \- Rust By Example, Zugriff am Mai 14, 2025, [https://doc.rust-lang.org/rust-by-example/error/multiple\_error\_types.html](https://doc.rust-lang.org/rust-by-example/error/multiple_error_types.html)  
9. Rust Error Handling: thiserror, anyhow, and When to Use Each | Momori Nakano, Zugriff am Mai 14, 2025, [https://momori.dev/posts/rust-error-handling-thiserror-anyhow/](https://momori.dev/posts/rust-error-handling-thiserror-anyhow/)  
10. smithay::wayland::compositor \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/wayland/compositor/index.html](https://smithay.github.io/smithay/smithay/wayland/compositor/index.html)  
11. Display in wayland\_server \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/wayland\_server/struct.Display.html](https://smithay.github.io/smithay/wayland_server/struct.Display.html)  
12. LoopHandle in calloop \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/calloop/latest/calloop/struct.LoopHandle.html](https://docs.rs/calloop/latest/calloop/struct.LoopHandle.html)  
13. GlobalDispatch in wayland\_server \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/wayland\_server/trait.GlobalDispatch.html](https://smithay.github.io/smithay/wayland_server/trait.GlobalDispatch.html)  
14. uuid \- Rust, Zugriff am Mai 14, 2025, [https://messense.github.io/bosonnlp-rs/uuid/index.html](https://messense.github.io/bosonnlp-rs/uuid/index.html)  
15. Uuid in rocket::serde::uuid \- Rust, Zugriff am Mai 14, 2025, [https://api.rocket.rs/v0.5/rocket/serde/uuid/struct.Uuid](https://api.rocket.rs/v0.5/rocket/serde/uuid/struct.Uuid)  
16. Appendix A. Wayland Protocol Specification, Zugriff am Mai 14, 2025, [https://wayland.freedesktop.org/docs/html/apa.html](https://wayland.freedesktop.org/docs/html/apa.html)  
17. smithay::wayland::shm \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/wayland/shm/index.html](https://smithay.github.io/smithay/smithay/wayland/shm/index.html)  
18. wayland\_client::protocol::wl\_shm \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/wayland-client/latest/wayland\_client/protocol/wl\_shm/index.html](https://docs.rs/wayland-client/latest/wayland_client/protocol/wl_shm/index.html)  
19. gdk/wayland/protocol/xdg-shell.xml · 3.13.5 · Zrythm / GTK · GitLab, Zugriff am Mai 14, 2025, [https://gitlab.zrythm.org/zrythm/gtk/-/blob/3.13.5/gdk/wayland/protocol/xdg-shell.xml](https://gitlab.zrythm.org/zrythm/gtk/-/blob/3.13.5/gdk/wayland/protocol/xdg-shell.xml)  
20. XdgToplevel in wayland\_protocols::xdg::shell::server::xdg\_toplevel, Zugriff am Mai 14, 2025, [https://smithay.github.io/wayland-rs/wayland\_protocols/xdg/shell/server/xdg\_toplevel/struct.XdgToplevel.html](https://smithay.github.io/wayland-rs/wayland_protocols/xdg/shell/server/xdg_toplevel/struct.XdgToplevel.html)  
21. ToplevelSurface in smithay::wayland::shell::xdg \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/wayland/shell/xdg/struct.ToplevelSurface.html](https://smithay.github.io/smithay/smithay/wayland/shell/xdg/struct.ToplevelSurface.html)  
22. Popup in smithay\_client\_toolkit::shell::xdg::popup \- Rust, Zugriff am Mai 14, 2025, [https://doc.servo.org/smithay\_client\_toolkit/shell/xdg/popup/struct.Popup.html](https://doc.servo.org/smithay_client_toolkit/shell/xdg/popup/struct.Popup.html)  
23. smithay::backend \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/backend/index.html](https://smithay.github.io/smithay/smithay/backend/index.html)  
24. smithay::backend::renderer::element \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/backend/renderer/element/index.html](https://smithay.github.io/smithay/smithay/backend/renderer/element/index.html)  
25. smithay::backend::libinput \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/backend/libinput/index.html](https://smithay.github.io/smithay/smithay/backend/libinput/index.html)  
26. smithay::input \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/input/index.html](https://smithay.github.io/smithay/smithay/input/index.html)  
27. LibinputInputBackend in smithay::backend::libinput \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/backend/libinput/struct.LibinputInputBackend.html](https://smithay.github.io/smithay/smithay/backend/libinput/struct.LibinputInputBackend.html)  
28. Device in input \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/input/struct.Device.html](https://smithay.github.io/smithay/input/struct.Device.html)  
29. smithay::wayland::seat \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/wayland/seat/index.html](https://smithay.github.io/smithay/smithay/wayland/seat/index.html)  
30. State in smithay::input::keyboard::xkb \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/smithay/smithay/input/keyboard/xkb/struct.State.html](https://smithay.github.io/smithay/smithay/input/keyboard/xkb/struct.State.html)  
31. smithay::wayland::seat \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/smithay/latest/smithay/wayland/seat/index.html](https://docs.rs/smithay/latest/smithay/wayland/seat/index.html)  
32. Seats — libinput 1.28.1 documentation \- Wayland, Zugriff am Mai 14, 2025, [https://wayland.freedesktop.org/libinput/doc/latest/seats.html](https://wayland.freedesktop.org/libinput/doc/latest/seats.html)