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

---


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

---

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


---



1. **Die detaillierte Aufteilung der Systemschicht in Hauptmodule und deren Submodule** basierend auf der Gesamtspezifikation () und Ihren Anforderungen festlegen.
2. Für **ein ausgewähltes, kritisches Hauptmodul (z.B. `system::compositor`) und dessen wichtigste Submodule eine vollständige Ultra-Feinspezifikation** mit der geforderten Detailtiefe erstellen, einschließlich Dateistruktur, Datentypen, API-Signaturen, Logikbeschreibungen und Implementierungsschritten.
3. Für die **weiteren Hauptmodule der Systemschicht die Struktur (Submodule) und die Kernverantwortlichkeiten skizzieren** sowie auf die relevanten Abschnitte in der Gesamtspezifikation () verweisen. Eine vollständige Ausarbeitung aller Module würde den Rahmen sprengen, aber die Methodik des detaillierten Moduls dient als Blaupause.

**Autonome Entscheidung zur Sequenzierung und Submodul-Aufteilung der Systemschicht:**

Basierend auf der Gesamtspezifikation () und der Notwendigkeit einer klaren Struktur, unterteile ich die Systemschicht (`novade-system` Crate) wie folgt:

1. **`system::compositor`**: Wayland-Compositor-Logik (Smithay-basiert).
    - `compositor::core`: Zentrale Compositor-Zustände und Handler.
    - `compositor::surface_management`: Verwaltung von `WlSurface`-Daten.
    - `compositor::shm`: SHM-Pufferbehandlung.
    - `compositor::xdg_shell`: XDG-Shell-Protokollintegration (Toplevels, Popups).
    - `compositor::layer_shell`: `wlr-layer-shell`-Integration für Panels, Docks etc.
    - `compositor::decoration`: `xdg-decoration`-Integration (Client/Server-Side Decorations).
    - `compositor::output_management`: `wlr-output-management` und `xdg-output` für Monitor-Konfiguration.
    - `compositor::input_method`: Integration von Eingabemethoden (IME).
    - `compositor::screencopy`: Screenshot- und Screencasting-Protokolle (z.B. `wlr-screencopy`).
    - `compositor::data_device`: Zwischenablage (Copy & Paste) und Drag & Drop.
    - `compositor::xwayland`: Integration und Verwaltung des XWayland-Servers.
    - `compositor::renderer_interface`: Abstrakte Schnittstelle zum Rendering-Backend.
    - `compositor::drm_gbm_renderer` (optional, eine konkrete Renderer-Implementierung).
    - `compositor::winit_renderer` (optional, eine weitere konkrete Renderer-Implementierung für verschachtelten Betrieb).
2. **`system::input`**: Eingabeverarbeitung (libinput-basiert).
    - `input::seat_manager`: Seat-Management, Fokus, Capabilities.
    - `input::libinput_handler`: Integration des Libinput-Backends.
    - `input::keyboard`: Tastaturereignis-Übersetzung, XKB-Management.
    - `input::pointer`: Maus-/Zeigerereignis-Verarbeitung, Cursor.
    - `input::touch`: Touch-Ereignis-Verarbeitung.
    - `input::gestures`: Gestenerkennung (aufbauend auf libinput-Events).
3. **`system::dbus_interfaces`**: Schnittstellen zu System-D-Bus-Diensten.
    - `dbus_interfaces::connection_manager`: Basis für D-Bus-Verbindungen.
    - `dbus_interfaces::network_manager`: Client für NetworkManager.
    - `dbus_interfaces::upower`: Client für UPower.
    - `dbus_interfaces::logind`: Client für systemd-logind.
    - `dbus_interfaces::notifications_server`: Implementierung des `org.freedesktop.Notifications`-Servers (nutzt `domain::user_centric_services::notifications_core`).
    - `dbus_interfaces::secrets_service`: Client für `org.freedesktop.secrets`.
    - `dbus_interfaces::policykit`: Client für PolicyKit.
    - `dbus_interfaces::xdg_desktop_portal_handler`: Backend-Logik für Portale (Interaktion mit Compositor etc.).
4. **`system::audio_management`**: PipeWire-Client-Integration.
    - `audio_management::client`: PipeWire-Core-Interaktion.
    - `audio_management::device_manager`: Verwaltung von Audio-Geräten.
    - `audio_management::stream_manager`: Verwaltung von Anwendungs-Streams.
    - `audio_management::volume_control`: Lautstärkeregelung.
5. **`system::mcp_client`**: Implementierung des Model Context Protocol Clients.
    - `mcp_client::connection`: Verwaltung der MCP-Server-Verbindung.
    - `mcp_client::protocol_handler`: Senden/Empfangen von MCP-Nachrichten.
    - `mcp_client::resource_provider`: Bereitstellung von Systemressourcen für KI-Modelle.
6. **`system::window_mechanics`**: Technische Umsetzung des Fenstermanagements.
    - `window_mechanics::placement`: Anwendung von Platzierungsrichtlinien.
    - `window_mechanics::tiling_engine`: Anwendung von Tiling-Layouts.
    - `window_mechanics::focus_arbitration`: Technische Fokushandhabung.
    - `window_mechanics::decoration_manager`: Koordination von SSD/CSD.
7. **`system::event_bridge`**: Brücke für Systemereignisse zur Domänenschicht und UI-Schicht.
8. **`system::power_management`**: Interaktion mit `wlr-output-power-management` und DPMS.

**Detailausarbeitung für `system::compositor` (Teil 1 - Kern & XDG Shell):**

---

## Ultra-Feinspezifikation und Implementierungsplan: Systemschicht (`novade-system` Crate)

**Allgemeine Entwicklungsrichtlinien für die Systemschicht:**

- **Sprache:** Rust (Edition 2021+).
- **Abhängigkeiten:** `novade-core`, `novade-domain`, `smithay` (spezifische Version, z.B. 0.10 oder neuer), `wayland-server`, `wayland-protocols`, `calloop`, `libinput`, `xkbcommon`, `zbus`, `pipewire-rs`, `mcp_client_rs`, `tracing`, `thiserror`, `uuid`, `serde`, `serde_json`, `parking_lot` (optional für Mutex).
- **Fehlerbehandlung:** Modulspezifische Fehler-Enums mit `thiserror`. Fehler aus `smithay` und anderen Bibliotheken werden gewrappt.
- **Asynchronität:** `calloop` ist die primäre Ereignisschleife für `smithay`. Andere asynchrone Operationen (z.B. `zbus`) müssen ggf. über Adapter integriert werden oder in eigenen Threads laufen und mit `calloop` über Kanäle/Signale kommunizieren. Wo `smithay` oder seine Abhängigkeiten `async/await` erfordern (z.B. einige `zbus`-Interaktionen), wird dies genutzt.
- **Zustandsverwaltung:** Eine zentrale `DesktopState`-Struktur wird die meisten `smithay`-Handler implementieren und die Zustände der Subsysteme halten.
- **Interaktion mit Domänenschicht:** Die Systemschicht ruft Services der Domänenschicht auf, um Geschäftslogik anzuwenden oder Zustände zu aktualisieren/abzufragen. Sie übersetzt Systemereignisse in Domänenereignisse oder -aufrufe.

**Cargo.toml für `novade-system` (Auszug):**

Ini, TOML

```
[dependencies]
novade-core = { path = "../novade-core" }
novade-domain = { path = "../novade-domain" }

smithay = { version = "0.10.0", features = ["renderer_gl", "backend_libinput", "backend_session", "backend_udev", "backend_drm", "backend_winit", "desktop", "xwayland", "use_system_lib"] } # Beispielversion, Features anpassen
wayland-server = "0.30" # Smithay-kompatible Version
wayland-protocols = { version = "0.30", features = ["server", "unstable_protocols"] } # Smithay-kompatible Version
calloop = "0.12"
libinput = "0.9"
xkbcommon = "0.7"
# ... weitere Abhängigkeiten ...
```

---

### Modul 1: `system::compositor`

Zweck: Implementierung des Wayland-Compositors unter Verwendung des Smithay-Toolkits.

Verantwortlichkeiten: Client-Verwaltung, Oberflächen-Lebenszyklus, Pufferbehandlung, Shell-Protokolle, Koordination des Renderings.

#### 1.1. Submodul: `system::compositor::core`

**Zweck:** Zentrale Compositor-Zustände, `DesktopState`-Definition, Basis-Handler-Implementierungen.

**Datei:** `src/compositor/core/errors.rs`

- **Enum `CompositorCoreError`**:
    
    Rust
    
    ```
    use thiserror::Error;
    use smithay::wayland::compositor::SurfaceRoleError;
    use wayland_server::{backend::ClientId, protocol::wl_surface::WlSurface};
    
    #[derive(Debug, Error)]
    pub enum CompositorCoreError {
        #[error("Failed to create Wayland global object '{0}'")]
        GlobalCreationFailed(String),
        #[error("Surface role error: {0}")]
        RoleError(#[from] SurfaceRoleError), // From smithay
        #[error("Client data not found for client ID {0:?}")]
        ClientDataMissing(ClientId),
        #[error("SurfaceData not found or of wrong type for WlSurface {0:?}")]
        SurfaceDataMissing(WlSurface),
        #[error("Invalid surface state: {0}")]
        InvalidSurfaceState(String),
        #[error("Renderer backend initialization failed: {0}")]
        RendererInitializationFailed(String),
        #[error("Display or EventLoop creation failed: {0}")]
        DisplayOrLoopCreationFailed(String),
        #[error("Failed to initialize XWayland: {0}")]
        XWaylandInitializationError(String),
        // Weitere spezifische Fehler
    }
    ```
    

**Datei:** `src/compositor/core/state.rs`

- **Struct `ClientCompositorData`** (für `Client::data_map`)
    
    Rust
    
    ```
    use smithay::wayland::compositor::CompositorClientState;
    use smithay::wayland::shell::xdg::XdgShellClientData; // Für XDG-Shell
    // Ggf. weitere Client-spezifische Zustände von Smithay-Modulen
    
    pub struct ClientCompositorData {
        pub compositor_state: CompositorClientState,
        // pub xdg_shell_data: XdgShellClientData, // Wird von XdgShellState::new_client verwaltet
    }
    
    impl ClientCompositorData {
        pub fn new() -> Self {
            Self {
                compositor_state: CompositorClientState::default(),
            }
        }
    }
    ```
    
- **Struct `DesktopState`** (Zentrale Zustandsstruktur)
    
    Rust
    
    ```
    use smithay::{
        backend::renderer::gles2::Gles2Renderer, // Beispiel-Renderer
        desktop::{Space, Window, WindowSurfaceType},
        input::{Seat, SeatState, pointer::CursorImageStatus},
        reexports::{
            calloop::{LoopHandle, Interest, Mode, PostAction},
            wayland_server::{Display, DisplayHandle, Client, backend::{GlobalId, ClientId}},
        },
        utils::{Clock, Logical, Point, Rectangle, Serial, Transform},
        wayland::{
            compositor::{CompositorState, CompositorClientState, CompositorHandler, SurfaceAttributes as WlSurfaceAttributes, add_destruction_hook},
            output::OutputManagerState, // Für wlr-output-management & xdg-output
            shell::{
                xdg::{XdgShellState, XdgShellHandler, XdgToplevelSurfaceData, XdgPopupSurfaceData, SurfaceCachedState, XdgWmBaseClientData},
                kde_decoration::KdeDecorationManagerState, // Beispiel für SSD
            },
            shm::{ShmState, ShmHandler},
            seat::WaylandSeatData, // Für wl_seat UserData
            // ... weitere Smithay-Module ...
            selection::data_device::{DataDeviceState, DataDeviceHandler},
            selection::primary_selection::{PrimarySelectionState, PrimarySelectionHandler},
            input_method::InputMethodManagerState,
            relative_pointer::RelativePointerManagerState,
            pointer_constraints::PointerConstraintsState,
            viewporter::ViewporterState,
            presentation::PresentationState,
            xdg_activation::XdgActivationState,
        },
    };
    use crate::domain::window_management_policy::{WindowManagementPolicyService, WindowPolicyOverrides, TilingMode, WorkspaceWindowLayout}; // Domain Service
    use crate::domain::workspaces::core::types::{WorkspaceId, WindowIdentifier as DomainWindowIdentifier};
    use crate::domain::workspaces::manager::WorkspaceManagerService;
    use std::{collections::HashMap, sync::{Arc, Mutex}}; // Mutex für Domain-Services
    use uuid::Uuid;
    use super::surface_management::{SurfaceData, RenderableElement}; // Eigene Typen
    use super::super::input::keyboard::xkb_config::XkbKeyboardData; // Aus system::input
    
    pub const CLOCK_ID: usize = 0;
    
    pub struct NovaDEWaylandState { /* Für Globals, die nur einmal existieren */
        pub shm_global: GlobalId,
        pub xdg_shell_global: GlobalId,
        pub output_manager_global: GlobalId,
        pub seat_global: GlobalId,
        pub data_device_global: GlobalId,
        // ... weitere GlobalIds ...
        pub xdg_activation_global: GlobalId,
    }
    
    pub struct DesktopState {
        pub display_handle: DisplayHandle,
        pub loop_handle: LoopHandle<'static, Self>, // 'static, wenn DesktopState global ist
        pub clock: Clock<u64>, // Für Timings, Animationen
    
        // Compositor & Core States
        pub compositor_state: CompositorState,
        pub shm_state: ShmState,
        pub presentation_state: PresentationState,
        pub viewporter_state: ViewporterState,
    
    
        // Shells & Window Management
        pub xdg_shell_state: XdgShellState,
        pub xdg_activation_state: XdgActivationState,
        // pub layer_shell_state: LayerShellState, // Für wlr-layer-shell
        // pub kde_decoration_state: KdeDecorationManagerState, // Für KWin SSD
    
        // Workspace & Window Tracking (Compositor-Sicht)
        pub space: Space<Window>, // Smithay's Desktop-Raum für Fensterverwaltung
        pub windows: HashMap<DomainWindowIdentifier, Window>, // Eigene Map für Zugriff via Domain ID
    
        // Input & Seat
        pub seat_state: SeatState<Self>,
        pub seat: Seat<Self>, // Der primäre Seat
        pub seat_name: String,
        pub input_method_manager_state: InputMethodManagerState, // Für IME
        pub relative_pointer_manager_state: RelativePointerManagerState,
        pub pointer_constraints_state: PointerConstraintsState,
        pub keyboard_data_map: HashMap<String /* seat_name oder device_id */, XkbKeyboardData>, // Für XKB
        pub current_cursor_status: Arc<Mutex<CursorImageStatus>>, // Für Cursor-Rendering
    
        // Output Management
        pub output_manager_state: OutputManagerState,
    
        // Data Exchange (Clipboard, DnD)
        pub data_device_state: DataDeviceState,
        // pub primary_selection_state: PrimarySelectionState,
    
        // XWayland
        // pub xwayland: XWayland, // Smithay's XWayland-Struktur
    
        // Domain Service Handles (Arc<Mutex<...>> oder Arc<dyn ...>)
        pub window_policy_service: Arc<dyn WindowManagementPolicyService>,
        pub workspace_manager_service: Arc<dyn WorkspaceManagerService>,
        // ... weitere Domain-Services ...
    
        // Renderer (wird später konkretisiert)
        // pub renderer: Gles2Renderer,
        // pub last_render_time: std::time::Instant,
    
        // Wayland Global IDs (um sie am Leben zu halten)
        pub wayland_globals: Option<NovaDEWaylandState>, // Wird nach Erstellung der Globals gefüllt
    }
    
    impl DesktopState {
        pub fn new(
            loop_handle: LoopHandle<'static, Self>,
            display_handle: DisplayHandle,
            window_policy_service: Arc<dyn WindowManagementPolicyService>,
            workspace_manager_service: Arc<dyn WorkspaceManagerService>,
        ) -> Self {
            let clock = Clock::new(Some(tracing::Span::current())); // tracing integration
            let compositor_state = CompositorState::new::<Self>(&display_handle, Some(tracing::Span::current()));
            let shm_state = ShmState::new::<Self>(&display_handle, vec![], Some(tracing::Span::current())); // Keine zusätzlichen Formate initial
            let presentation_state = PresentationState::new::<Self>(&display_handle, clock.id() as u32);
            let viewporter_state = ViewporterState::new::<Self>(&display_handle, Some(tracing::Span::current()));
    
            let xdg_shell_state = XdgShellState::new::<Self>(&display_handle, Some(tracing::Span::current()));
            let xdg_activation_state = XdgActivationState::new::<Self>(&display_handle, Some(tracing::Span::current()));
    
            let space = Space::new(Some(tracing::Span::current()));
    
            let mut seat_state = SeatState::new();
            let seat_name = "seat0".to_string();
            let seat = seat_state.new_wl_seat(&display_handle, seat_name.clone(), Some(tracing::Span::current()));
            // Capabilities (Keyboard, Pointer, Touch) werden später beim Input-Backend-Init hinzugefügt
    
            let input_method_manager_state = InputMethodManagerState::new::<Self>(&display_handle);
            let relative_pointer_manager_state = RelativePointerManagerState::new::<Self>(&display_handle);
            let pointer_constraints_state = PointerConstraintsState::new::<Self>(&display_handle, Some(tracing::Span::current()));
    
    
            let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&display_handle);
            let data_device_state = DataDeviceState::new::<Self>(&display_handle, Some(tracing::Span::current()));
    
            Self {
                display_handle,
                loop_handle,
                clock,
                compositor_state,
                shm_state,
                presentation_state,
                viewporter_state,
                xdg_shell_state,
                xdg_activation_state,
                space,
                windows: HashMap::new(),
                seat_state,
                seat,
                seat_name,
                input_method_manager_state,
                relative_pointer_manager_state,
                pointer_constraints_state,
                keyboard_data_map: HashMap::new(),
                current_cursor_status: Arc::new(Mutex::new(CursorImageStatus::Default)),
                output_manager_state,
                data_device_state,
                window_policy_service,
                workspace_manager_service,
                wayland_globals: None,
            }
        }
    }
    ```
    
    - **Initialisierung der Smithay-States:** Erfolgt im `new()` Konstruktor von `DesktopState`. Die `Logger` Parameter sind in neueren Smithay-Versionen oft durch `Option<tracing::Span>` ersetzt oder implizit.
    - **Domain Service Handles:** Werden als `Arc<dyn TraitName>` gespeichert, um Flexibilität und Testbarkeit zu gewährleisten. Sie werden von außen injiziert.
- **Implementierung `CompositorHandler` für `DesktopState`**:
    - **`compositor_state(&mut self) -> &mut CompositorState`**: Gibt `&mut self.compositor_state` zurück.
    - **`client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState`**:
        1. `client.get_data::<ClientCompositorData>().unwrap().compositor_state` (Annahme: `ClientCompositorData` wird beim Client-Connect in `Client::data_map` eingefügt).
    - **`commit(&mut self, surface: &WlSurface)`**:
        1. `tracing::debug!(surface_id = ?surface.id(), "Commit für WlSurface");`
        2. `smithay::wayland::compositor::with_states(surface, |states| { ... })` verwenden, um auf `SurfaceAttributes` und `SurfaceData` zuzugreifen.
        3. `let surface_attributes = states.cached_state.current::<WlSurfaceAttributes>();`
        4. **Puffer-Handling:**
            - Wenn `surface_attributes.buffer.is_some()` und `surface_attributes.buffer_delta != (0,0)` oder ein neuer Puffer angehängt wurde:
                - Die `SurfaceData` für diesen `surface` abrufen (aus `states.data_map`).
                - `surface_data.lock().unwrap().current_buffer = surface_attributes.buffer.clone();`
                - `surface_data.lock().unwrap().buffer_scale = surface_attributes.buffer_scale;`
                - `surface_data.lock().unwrap().buffer_transform = surface_attributes.buffer_transform;`
                - Renderer benachrichtigen, die Textur für diesen Puffer zu aktualisieren/erstellen (Details im Renderer-Modul).
        5. **Schadensverfolgung (Damage Tracking):**
            - `let damage = &surface_attributes.damage;` (Liste von `Rectangle<i32, BufferCoords>`).
            - Die `SurfaceData` mit diesen Schadensregionen aktualisieren: `surface_data.lock().unwrap().damage_buffer_coords.extend(damage.iter().cloned());`
        6. **Rollenbasierte Commit-Logik:**
            - `let role = smithay::wayland::compositor::get_role(surface);`
            - `match role { Some("xdg_toplevel") => { ... }, Some("xdg_popup") => { ... }, ... }`
            - Ruft spezifische Commit-Handler für XDG-Toplevels, Popups, Subsurfaces, Layer-Surfaces etc. auf. Diese könnten in `SurfaceData` als Callbacks/Hooks gespeichert sein oder direkt hier behandelt werden. Für XDG-Shell wird dies oft vom `XdgShellHandler` übernommen. Smithay's `desktop::Space` und `Window` handhaben vieles davon.
        7. **Synchronisierte Subsurfaces:** `if surface.is_sync_subsurface() { ... }` Logik für Parent-Commit ( Schritt 8).
        8. Oberfläche für Neuzeichnung im nächsten Frame markieren (z.B. `self.space.damage_window(&window_für_surface, ...)`).
    - **`new_surface(&mut self, surface: &WlSurface, client_data: &Arc<ClientCompositorData>)`** (Signatur kann variieren, je nachdem wie Client-Daten übergeben werden):
        1. `tracing::info!(surface_id = ?surface.id(), client_id = ?surface.client().unwrap().id(), "Neue WlSurface erstellt");`
        2. Initialisiere `SurfaceData::new(surface.client().unwrap().id())`.
        3. `surface.data_map().insert_if_missing_threadsafe(|| Arc::new(Mutex::new(SurfaceData::new(...))));`
        4. `add_destruction_hook(surface, |data_map| { ... Bereinigung von SurfaceData ... });`
    - **`new_subsurface(&mut self, surface: &WlSurface, parent: &WlSurface, client_data: &Arc<ClientCompositorData>)`**:
        1. `tracing::info!(surface_id = ?surface.id(), parent_id = ?parent.id(), "Neue WlSubsurface erstellt");`
        2. `SurfaceData` von `surface` aktualisieren: `surface_data.lock().unwrap().parent = Some(parent.downgrade());`
        3. `SurfaceData` von `parent` aktualisieren: `parent_surface_data.lock().unwrap().children.push(surface.downgrade());`
    - **`destroyed(&mut self, surface: &WlSurface)`**:
        1. `tracing::info!(surface_id = ?surface.id(), "WlSurface zerstört");`
        2. Smithay kümmert sich um das Entfernen aus der `UserDataMap`.
        3. Sicherstellen, dass alle Referenzen auf diese `WlSurface` in `DesktopState` (z.B. in `space`, `windows`, Fokus-Listen) entfernt werden. Dies geschieht oft über den `destruction_hook` der `SurfaceData`.

#### 1.2. Submodul: `system::compositor::surface_management`

**Zweck:** Definition und Verwaltung von `SurfaceData`.

**Datei:** `src/compositor/surface_management/mod.rs`

- **Enum `RenderableElement`** (kann auch direkt Smithay's `Element` sein oder dieses wrappen)
    
    Rust
    
    ```
    // Beispiel, wird durch Renderer-Schnittstelle konkretisiert
    pub enum RenderableElement {
        WaylandSurface {
            surface: WlSurface, // Oder eine ID/Wrapper, der die Textur hält
            position: Point<i32, Logical>,
            scale: f64,
            transform: Transform, // Bildschirmrotation etc.
            damage_surface_local: Vec<Rectangle<i32, Logical>>, // Schaden relativ zur Oberfläche
            opaque_regions_surface_local: Vec<Rectangle<i32, Logical>>,
        },
        SolidColor { /* ... */ },
        Cursor { /* ... */ },
    }
    ```
    
- **Struct `SurfaceData`**:
    - **Felder:**
        - `pub id: Uuid` (Eigene interne ID)
        - `pub client_id: ClientId`
        - `pub role: Mutex<Option<String>>` (Rolle, z.B. "toplevel", "popup", "cursor", "layer")
        - `pub current_buffer_info: Mutex<Option<AttachedBufferInfo>>` (Infos zum aktuellen Puffer)
        - `pub texture_handle: Mutex<Option<Box<dyn RenderableTexture>>>` (Handle vom Renderer, `RenderableTexture` Trait wird in `renderer_interface` definiert)
        - `pub damage_buffer_coords: Mutex<Vec<Rectangle<i32, smithay::utils::Buffer>>>`
        - `pub damage_surface_coords: Mutex<Vec<Rectangle<i32, Logical>>>` (Transformierter Schaden)
        - `pub opaque_region_surface_local: Mutex<Option<smithay::utils::Region<Logical>>>`
        - `pub input_region_surface_local: Mutex<Option<smithay::utils::Region<Logical>>>`
        - `pub user_data_ext: UserDataMap` (Für anwendungsspezifische Daten, die von anderen Modulen wie XDG-Shell oder Layer-Shell hier abgelegt werden)
        - `pub parent: Mutex<Option<wayland_server::Weak<WlSurface>>>`
        - `pub children: Mutex<Vec<wayland_server::Weak<WlSurface>>>`
        - `pub pre_commit_hooks: Mutex<Vec<Box<dyn FnMut(&mut DesktopState, &WlSurface) + Send + Sync>>>`
        - `pub post_commit_hooks: Mutex<Vec<Box<dyn FnMut(&mut DesktopState, &WlSurface) + Send + Sync>>>`
        - `destruction_callback: Mutex<Option<Box<dyn FnOnce(&mut DesktopState, &WlSurface) + Send + Sync>>>` (Ein dedizierter Callback statt Vec für einmalige Zerstörung)
        - `pub surface_viewporter_state: Mutex<smithay::wayland::viewporter::SurfaceState>`
        - `pub surface_presentation_state: Mutex<smithay::wayland::presentation::SurfaceState>`
        - `pub surface_scale_factor: Mutex<f64>` (Skalierungsfaktor, der auf diese Oberfläche angewendet wird, z.B. vom Output)
    - **Struct `AttachedBufferInfo`**:
        
        Rust
        
        ```
        #[derive(Debug, Clone)]
        pub struct AttachedBufferInfo {
            pub buffer: WlBuffer,
            pub scale: i32, // Smithay's buffer_scale
            pub transform: Transform, // Smithay's buffer_transform
            pub dimensions: Size<i32, smithay::utils::Buffer>, // Größe des Puffers
        }
        ```
        
    - **Methoden für `SurfaceData`**:
        - `pub fn new(client_id: ClientId) -> Self`
        - `pub fn set_role(&self, role: &str) -> Result<(), CompositorCoreError>`
        - `pub fn get_role(&self) -> Option<String>`
        - `pub fn attach_buffer(&self, buffer_info: Option<AttachedBufferInfo>)`
        - `pub fn take_damage_buffer_coords(&self) -> Vec<Rectangle<i32, smithay::utils::Buffer>>`
        - `pub fn add_pre_commit_hook(...)`, `add_post_commit_hook(...)`
        - `pub fn set_destruction_callback(...)`
        - `pub fn get_effective_damage_and_transform(&self, output_transform: Transform) -> (Vec<Rectangle<i32, Logical>>, Transform)` (Berechnet transformierten Schaden)
- **Funktionen:**
    - `pub fn get_surface_data(surface: &WlSurface) -> Option<Arc<SurfaceData>>`: Ruft `Arc<SurfaceData>` aus `surface.data_map()` ab.
    - `pub fn with_surface_data_mut<F, R>(surface: &WlSurface, callback: F) -> Result<R, CompositorCoreError> where F: FnOnce(&mut SurfaceData, &WlSurfaceAttributes) -> R`: Kapselt Locken und Zugriff. `WlSurfaceAttributes` wird über `with_states` geholt.

#### 1.3. Submodul: `system::compositor::shm`

**Zweck:** SHM-Pufferbehandlung (`wl_shm`).

**Datei:** `src/compositor/shm/errors.rs`

- **Enum `ShmError`**: ()
    - `PoolCreationFailed(String)`
    - `BufferCreationFailed(String)`
    - `InvalidFormat(wl_shm::Format)`
    - `AccessError(#[from] smithay::wayland::shm::BufferAccessError)`

**Datei:** `src/compositor/shm/mod.rs` (oder `state.rs`)

- **Implementierung `ShmHandler` für `DesktopState`**:
    - `shm_state(&self) -> &ShmState`: Gibt `&self.shm_state` zurück.
- **Implementierung `BufferHandler` für `DesktopState`** (Hier spezifisch für SHM-Puffer, obwohl der Trait generisch ist):
    - `buffer_destroyed(&mut self, buffer: &wl_buffer::WlBuffer)`:
        1. `tracing::debug!(buffer_id = ?buffer.id(), "SHM WlBuffer zerstört");`
        2. Finde alle `SurfaceData`-Instanzen, die diesen `buffer` in `current_buffer_info` verwenden.
        3. Für jede gefundene Instanz:
            - Entferne die Referenz auf den Puffer.
            - Benachrichtige den Renderer, die zugehörige Textur freizugeben (`surface_data.texture_handle.take()`).
            - Markiere die Oberfläche als beschädigt, da ihr Inhalt nun ungültig ist.
- **Implementierung `GlobalDispatch<WlShm, ()>` für `DesktopState`**: ()
    - `bind(...)`: `data_init.init(resource, ());` (Smithay's `ShmState` kümmert sich um das Senden von Formaten).
- **Funktion `pub fn create_shm_global(state: &mut DesktopState, display_handle: &DisplayHandle)`**: ()
    1. `let shm_global_id = state.shm_state.global().clone();` (Da `shm_state` bereits in `DesktopState::new` initialisiert wurde).
    2. Speichere `shm_global_id` in `state.wayland_globals.as_mut().unwrap().shm_global`.
    3. `tracing::info!("wl_shm Global für Clients verfügbar gemacht. Unterstützte Formate: ARGB8888, XRGB8888.");`

**Datei:** `src/compositor/shm/buffer_access.rs`

- **Funktion `pub fn with_shm_buffer_contents<F, T>(buffer: &wl_buffer::WlBuffer, callback: F) -> Result<T, ShmError> where F: FnOnce(*const u8, usize, &smithay::wayland::shm::BufferData) -> T`**:
    1. `smithay::wayland::shm::with_buffer_contents(buffer, callback).map_err(ShmError::from)`

#### 1.4. Submodul: `system::compositor::xdg_shell`

**Zweck:** XDG-Shell-Protokollintegration (`xdg_wm_base`, `xdg_surface`, `xdg_toplevel`, `xdg_popup`).

**Datei:** `src/compositor/xdg_shell/errors.rs`

- **Enum `XdgShellError`**:
    
    Rust
    
    ```
    use thiserror::Error;
    use smithay::{utils::Serial, wayland::shell::xdg::ToplevelConfigureError};
    use wayland_server::protocol::wl_surface::WlSurface;
    use uuid::Uuid; // Für interne Window-IDs
    use crate::compositor::core::errors::CompositorCoreError;
    
    #[derive(Debug, Error)]
    pub enum XdgShellError {
        #[error("Surface {0:?} already has a different XDG role or is uninitialized.")]
        InvalidSurfaceRole(WlSurface),
        #[error("Window handling error for window ID {0}: {1}")]
        WindowHandlingError(Uuid, String), // Uuid ist die interne ID des ManagedWindow
        #[error("Popup positioning failed: {0}")]
        PopupPositioningError(String),
        #[error("Client provided invalid serial {client_serial:?} for configure, expected around {expected_serial:?}.")]
        InvalidAckConfigureSerial { client_serial: Serial, expected_serial: Serial },
        #[error("ManagedToplevel with ID {0} not found.")]
        ToplevelNotFound(Uuid),
        #[error("ManagedPopup with ID {0} not found.")]
        PopupNotFound(Uuid),
        #[error("XDG Toplevel configure operation failed: {0}")]
        ToplevelConfigureFailed(#[from] ToplevelConfigureError),
        #[error("Core compositor error during XDG operation: {0}")]
        CoreError(#[from] CompositorCoreError),
        #[error("XDG WM Base client data not found.")]
        XdgWmBaseClientDataMissing,
    }
    ```
    

**Datei:** `src/compositor/xdg_shell/types.rs`

- **Struct `ManagedWindow`** (ersetzt `ManagedToplevel` und `ManagedPopup` für Smithay's `Space<Window>`)
    
    Rust
    
    ```
    use smithay::{
        desktop::{Window, WindowSurface, WindowSurfaceType, Space},
        output::Output,
        reexports::wayland_protocols::xdg::shell::server::xdg_toplevel,
        utils::{Logical, Point, Rectangle, Size},
        wayland::shell::xdg::{ToplevelSurface, PopupSurface, PositionerState, XdgPopupSurfaceData, XdgToplevelSurfaceData},
    };
    use wayland_server::protocol::wl_surface::WlSurface;
    use wayland_server::Weak;
    use uuid::Uuid;
    use crate::domain::workspaces::core::types::WindowIdentifier as DomainWindowIdentifier; // Domain-ID
    use crate::compositor::surface_management::SurfaceData; // Zugriff auf SurfaceData
    use std::sync::{Arc, Mutex};
    
    #[derive(Debug, Clone, PartialEq)] // PartialEq ggf. manuell oder nur auf ID
    pub struct ManagedWindow {
        pub id: Uuid, // Interne Compositor-ID
        pub domain_id: DomainWindowIdentifier, // Verknüpfung zur Domänenschicht
        pub xdg_surface: WindowSurface, // Smithay's WindowSurface (Toplevel oder Popup)
        // app_id und title werden über xdg_surface.get_app_id() / .get_title() geholt
        pub current_geometry: Rectangle<i32, Logical>, // Berechnete Geometrie
        pub requested_size: Option<Size<i32, Logical>>,
        pub min_size: Option<Size<i32, Logical>>,
        pub max_size: Option<Size<i32, Logical>>,
        pub parent_id: Option<Uuid>, // Für transiente Toplevels oder Popups
        pub is_mapped: bool,
        // Weitere Zustände wie maximized, fullscreen, activated werden über xdg_surface.toplevel()
        // und dessen Methoden (e.g. current_states()) oder XdgToplevelSurfaceData verwaltet.
    }
    
    impl ManagedWindow {
        pub fn new_toplevel(toplevel_surface: ToplevelSurface, domain_id: DomainWindowIdentifier) -> Self {
            // Initialgeometrie etc. wird später vom Layout-Manager gesetzt
            Self {
                id: Uuid::new_v4(),
                domain_id,
                xdg_surface: WindowSurface::Toplevel(toplevel_surface),
                current_geometry: Rectangle::from_loc_and_size((0,0), (0,0)),
                requested_size: None, min_size: None, max_size: None,
                parent_id: None, // TODO: Parent-Logik für transiente Toplevel
                is_mapped: false,
            }
        }
    
        pub fn new_popup(popup_surface: PopupSurface, parent_domain_id: DomainWindowIdentifier) -> Self {
             // Popups haben eine komplexere Geometrieberechnung
            Self {
                id: Uuid::new_v4(),
                domain_id: DomainWindowIdentifier::new(format!("popup-{}", Uuid::new_v4())).unwrap(), // Eigene ID für Popups
                xdg_surface: WindowSurface::Popup(popup_surface),
                current_geometry: Rectangle::from_loc_and_size((0,0), (0,0)), // Wird durch Positioner bestimmt
                requested_size: None, min_size: None, max_size: None,
                parent_id: Some(Uuid::default()), // TODO: parent_id korrekt setzen auf Uuid des Parent ManagedWindow
                is_mapped: false,
            }
        }
    
        pub fn wl_surface(&self) -> &WlSurface {
            self.xdg_surface.wl_surface()
        }
        // ... weitere Hilfsmethoden ...
    }
    
    // Implementierung von smithay::desktop::Window für ManagedWindow
    impl Window for ManagedWindow {
        fn id(&self) -> usize {
            // Smithay's Space benötigt usize. Wir können die Bytes unserer Uuid nehmen.
            // Dies muss stabil sein für die Lebenszeit des Fensters.
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            self.id.hash(&mut hasher);
            std::hash::Hasher::finish(&hasher) as usize
        }
        fn wl_surface(&self) -> Option<WlSurface> { Some(self.xdg_surface.wl_surface().clone()) }
        fn surface_type(&self) -> WindowSurfaceType { self.xdg_surface.surface_type() }
        fn geometry(&self) -> Rectangle<i32, Logical> { self.current_geometry }
        fn is_mapped(&self) -> bool { self.is_mapped && self.xdg_surface.alive() } // Und ob die Wayland-Oberfläche noch existiert
        fn is_suspended(&self) -> bool {
            // Abfragen von XdgToplevelSurfaceData, ob minimiert etc.
            if let WindowSurface::Toplevel(t) = &self.xdg_surface {
                let data = t.user_data().get::<XdgToplevelSurfaceData>().unwrap();
                return data.minimized || data.suspended;
            }
            false
        }
        // ... weitere Methoden des Window-Traits implementieren (send_configure, set_activated etc.)
        // Diese rufen oft Methoden auf self.xdg_surface.toplevel() oder .popup() auf.
        fn send_frame(&self, output: &Output, time: impl Into<Duration>, throttle: Option<Duration>, primary_scan_out_output: Option<&Output>) {
            // Für Frame-Callbacks des Presentation-Time Protokolls
            if let Some(wl_surface) = self.wl_surface() {
                 smithay::wayland::presentation::send_frames_surface_dest harming_region_transform,
                    &wl_surface,
                    output,
                    time,
                    throttle,
                    primary_scan_out_output,
                );
            }
        }
        // ...
    }
    ```
    
    - **Verwendung von `smithay::desktop::Window` Trait**: Die `ManagedWindow`-Struktur implementiert diesen Trait, um mit `smithay::desktop::Space` kompatibel zu sein. Dies vereinfacht die Fensterverwaltung, das Stapeln und die Schadensberechnung für den Renderer erheblich.

**Datei:** `src/compositor/xdg_shell/handlers.rs`

- **Implementierung `XdgShellHandler` für `DesktopState`**:
    - **`xdg_shell_state(&mut self) -> &mut XdgShellState`**: Gibt `&mut self.xdg_shell_state` zurück.
    - **`new_toplevel(&mut self, surface: ToplevelSurface)`**:
        1. `tracing::info!(surface = ?surface.wl_surface().id(), "Neues XDG Toplevel wird erstellt.");`
        2. `let domain_window_id = DomainWindowIdentifier::new(format!("xdg-toplevel-{}", Uuid::new_v4())).unwrap();`
        3. `let mut managed_window = ManagedWindow::new_toplevel(surface.clone(), domain_window_id.clone());`
        4. **Initialgeometrie von Domäne anfordern:**
            - `let window_layout_info = WindowLayoutInfo { id: domain_window_id.clone(), requested_min_size: None, ... };`
            - `let initial_geom_result = block_on(self.window_policy_service.get_initial_window_geometry(&window_layout_info, ...));` (Blockieren hier ist problematisch in `calloop`. Besser: `get_initial_window_geometry` synchron machen oder die Fenstererstellung in eine Task-Pipeline verschieben.)
            - Wenn `Ok(geom)`, setze `managed_window.current_geometry = geom;`. Sonst Standardgeometrie.
        5. Füge `XdgToplevelSurfaceData` zur UserDataMap des `surface.wl_surface()` hinzu (Smithay macht das oft schon).
        6. `surface.with_pending_state(|state| { state.size = Some(managed_window.current_geometry.size); });`
        7. `surface.send_configure();` (Sendet initiale Größe etc. an Client).
        8. Füge `managed_window` zu `self.space` hinzu: `let window_arc = Arc::new(managed_window); self.space.map_window(window_arc.clone(), (initial_x, initial_y), true);` (Aktivieren, falls es Fokus bekommen soll).
        9. `self.windows.insert(domain_window_id, window_arc);`
    - **`new_popup(&mut self, surface: PopupSurface, _client_data: &XdgWmBaseClientData)`**:
        1. `tracing::info!(surface = ?surface.wl_surface().id(), "Neues XDG Popup wird erstellt.");`
        2. `let parent_wl_surface = surface.get_parent_surface().ok_or_else(|| XdgShellError::PopupPositioningError("Popup hat keine Elternoberfläche".into()))?;`
        3. Finde `parent_managed_window` über `parent_wl_surface` in `self.space` oder `self.windows`.
        4. `let managed_popup = ManagedWindow::new_popup(surface.clone(), parent_managed_window.domain_id.clone());`
        5. Füge `XdgPopupSurfaceData` zum `surface.wl_surface().data_map()` hinzu (Smithay macht das oft).
        6. Berechne Popup-Geometrie: `let positioner = surface.get_positioner(); let popup_geom = calculate_popup_geometry(&positioner, parent_managed_window.geometry());`
        7. `managed_popup.current_geometry = popup_geom;`
        8. `surface.send_configure();`
        9. Füge Popup zu `self.space` hinzu (Smithay's Space kann auch Popups verwalten, oder sie werden relativ zum Parent gerendert).
        10. `self.windows.insert(managed_popup.domain_id.clone(), Arc::new(managed_popup));`
    - **`map_toplevel(&mut self, surface: &ToplevelSurface)`**:
        1. Finde `ManagedWindow` für `surface.wl_surface()`.
        2. `managed_window.is_mapped = true;`
        3. Benachrichtige Domänenschicht (z.B. `workspace_manager_service.assign_window_to_active_workspace(&managed_window.domain_id)`).
        4. Fordere ein Re-Layout für den Workspace an.
        5. `self.space.damage_all_outputs();` (Oder spezifischer Schaden).
    - **`unmap_toplevel(&mut self, surface: &ToplevelSurface)`**:
        1. Finde `ManagedWindow`. `managed_window.is_mapped = false;`
        2. Entferne Fenster aus Workspace (`workspace_manager_service.remove_window_from_its_workspace`).
        3. `self.space.unmap_window(&managed_window_arc);`
    - **`ack_configure(&mut self, surface: WlSurface, configure_data: XdgSurfaceConfigureUserData)`**:
        1. `tracing::debug!(surface = ?surface.id(), serial = ?configure_data.serial, "XDG Surface ack_configure empfangen.");`
        2. Finde `ManagedWindow`.
        3. Logik für `ack_configure` gemäß Smithay-Dokumentation (Serial-Vergleich, Zustandsanwendung).
        4. `if let SurfaceCachedState::Toplevel(toplevel_data) = configure_data.cached_state { ... }`
    - **Andere `XdgShellHandler`-Methoden (`*_request_*`):**
        - Finde das `ManagedWindow`.
        - Aktualisiere den Zustand im `ManagedWindow` und/oder dessen `XdgToplevelSurfaceData` (z.B. `title`, `app_id`, `maximized`, `fullscreen`).
        - Interagiere mit `self.window_policy_service` für Größen-/Zustandsänderungen.
        - Rufe `toplevel_surface.send_configure()` auf, um den Client über den neuen Zustand zu informieren.
        - Für `move` und `resize`: Starte einen interaktiven Grab über `self.seat.start_pointer_grab(...)` oder `self.seat.start_touch_grab(...)`.
    - **`toplevel_destroyed(&mut self, toplevel: ToplevelSurface)`**:
        1. Finde `ManagedWindow`.
        2. `self.space.unmap_window(&managed_window_arc);`
        3. `self.windows.remove(&managed_window.domain_id);`
        4. Benachrichtige Domäne.
    - **`popup_destroyed(&mut self, popup: PopupSurface)`**: Analog.

**Datei:** `src/compositor/xdg_shell/mod.rs` (oder `state.rs`)

- **Implementierung `GlobalDispatch<XdgWmBase, ()>` für `DesktopState`**: ()
    - `bind(...)`:
        1. `let client_data = client.get_data::<Arc<Mutex<XdgWmBaseClientData>>>().cloned();` (oder `state.xdg_shell_state.new_client(client)` und speichern).
        2. `data_init.init(resource, client_data.expect("XdgWmBase client data must be set").clone());`
- **Funktion `pub fn create_xdg_shell_global(state: &mut DesktopState, display_handle: &DisplayHandle)`**: ()
    1. `let xdg_shell_global_id = state.xdg_shell_state.global().clone();`
    2. Speichere in `state.wayland_globals`.
    3. `tracing::info!("xdg_wm_base Global v{} für Clients verfügbar gemacht.", XdgWmBase::VERSION);`

#### 1.5. Implementierungsschritte `system::compositor` (Teil 1)

1. **Grundgerüst**: Verzeichnisse anlegen, `Cargo.toml` für Smithay etc. anpassen.
2. **`core/errors.rs`**: `CompositorCoreError` definieren.
3. **`surface_management/mod.rs`**: `SurfaceData`, `AttachedBufferInfo` definieren. `get_surface_data`, `with_surface_data_mut` implementieren.
4. **`core/state.rs`**: `ClientCompositorData`. `DesktopState` Grundstruktur mit `compositor_state`, `display_handle`, `loop_handle`, `clock`, `space`, `windows`, `seat_state`, `seat`, Domain-Service-Handles. `new()`-Konstruktor.
5. **`core/state.rs`**: `CompositorHandler` für `DesktopState` implementieren (`compositor_state`, `client_compositor_state`, `commit`, `new_surface`, `new_subsurface`, `destroyed`).
6. **`shm/errors.rs`**: `ShmError`.
7. **`shm/mod.rs`**: `ShmHandler` und `BufferHandler` (SHM-Teil) für `DesktopState`. `GlobalDispatch<WlShm, ()>`. `create_shm_global`.
8. **`shm/buffer_access.rs`**: `with_shm_buffer_contents`.
9. **`xdg_shell/errors.rs`**: `XdgShellError`.
10. **`xdg_shell/types.rs`**: `ManagedWindow` (implementiert `smithay::desktop::Window`), `XdgToplevelSurfaceData`, `XdgPopupSurfaceData` (Smithay-Typen, die via `UserDataMap` an `WlSurface` gehängt werden).
11. **`xdg_shell/handlers.rs`**: `XdgShellHandler` für `DesktopState` implementieren (alle Methoden, Interaktion mit `ManagedWindow`, `Space`, `window_policy_service`).
12. **`xdg_shell/mod.rs`**: `GlobalDispatch<XdgWmBase, ()>`. `create_xdg_shell_global`.
13. **`src/compositor/mod.rs`**: Öffentliche API des Compositor-Moduls definieren (z.B. Initialisierungsfunktionen, Zugriff auf `DesktopState` falls nötig).
14. **Unit-Tests (sehr anspruchsvoll für Compositor-Logik, Fokus auf Handler-Teillogik):**
    - Testen der Zustandsübergänge von `ManagedWindow`.
    - Testen der `SurfaceData`-Manipulation.
    - Mocking von Domain-Services, um Interaktionen mit `window_policy_service` etc. zu testen.
    - Smithay bietet Test-Utilities (`smithay::test_render`, Test-Clients), die evaluiert werden sollten.

---

**Nächste Schritte für `system::compositor` (Skizze für weitere Teile):**

- **`layer_shell`**: Implementierung von `WlrLayerShellHandler` etc. für Panels, Docks.
- **`decoration`**: `KdeDecorationManagerHandler` oder `XdgDecorationHandler`.
- **`output_management`**: `OutputManagerState`-Nutzung, `WlrOutputManagementHandlerV1`.
- **`input_method`**: `InputMethodManagerHandler`.
- **`screencopy`**, `data_device`, `xwayland` etc. analog.
- **`renderer_interface`**: Definition der Traits `FrameRenderer`, `RenderableTexture`.
- **Konkrete Renderer**: Implementierung für DRM/GBM und/oder Winit.

Diese detaillierte Ausarbeitung für den Kern des Compositors und XDG-Shell legt den Grundstein. Die weiteren Module der Systemschicht (`system::input`, `system::dbus_interfaces` etc.) würden mit ähnlicher Tiefe und Struktur spezifiziert werden, wobei jeweils die spezifischen Smithay-Komponenten, Systembibliotheken und Domäneninteraktionen im Fokus stehen.

---

### Modul 2: `system::input`

Zweck: Umfassende Verarbeitung von Benutzereingaben von diversen Geräten (Tastatur, Maus, Touchpad, Touchscreen, Tablets). Dies beinhaltet die Initialisierung und Verwaltung von Eingabegeräten mittels libinput, die Übersetzung roher Eingabeereignisse in für den Compositor und Wayland-Clients verständliche Formate, sowie die Handhabung des Seat-Managements, des Eingabefokus und der Cursor-Darstellung.

Verantwortlichkeiten: Integration mit libinput, Seat-Erstellung und -Verwaltung, Capability-Management (Tastatur, Zeiger, Touch), Fokuslogik, Übersetzung von Keycodes zu Keysyms/UTF-8 mittels xkbcommon, Verarbeitung von Zeigerbewegungen, Klicks, Scroll-Events, Touch-Interaktionen und Basis-Gesten.

Design-Rationale: Kapselung der komplexen Eingabelogik. libinput als Standard für die Geräteabstraktion unter Linux. Enge Verzahnung mit smithay's Seat-Management und Event-Strukturen. Die Logik muss performant und präzise sein, um eine direkte und reaktionsschnelle Benutzerinteraktion zu gewährleisten.

Bestehende Spezifikation: (ausführliche Basis aus Systemschicht Teil 1/4 der Recherche)

#### 2.1. Submodul: `system::input::errors`

**Datei:** `src/input/errors.rs`

- **Enum `InputError`**:
    
    Rust
    
    ```
    use thiserror::Error;
    use smithay::input::{SeatError, keyboard::KeyboardError};
    use std::io;
    
    #[derive(Debug, Error)]
    pub enum InputError {
        #[error("Failed to create or configure a seat: {0}")]
        SeatCreationFailed(String), // Generischer Fehler für Seat-Erstellung
        #[error("Failed to add capability '{capability}' to seat '{seat_name}': {source}")]
        CapabilityAdditionFailed {
            seat_name: String,
            capability: String,
            #[source]
            source: Box<dyn std::error::Error + Send + Sync + 'static>, // Kann SeatError oder KeyboardError sein
        },
        #[error("XKB configuration error for seat '{seat_name}': {message}")]
        XkbConfigError { seat_name: String, message: String },
        #[error("Libinput backend initialization or processing error: {0}")]
        LibinputError(String), // Für Fehler direkt von libinput oder dem Smithay-Backend
        #[error("Libinput session error: {0}")] // Für Fehler von der LibinputInterface (open_restricted/close_restricted)
        LibinputSessionError(#[from] io::Error),
        #[error("Seat '{0}' not found.")]
        SeatNotFound(String),
        #[error("Keyboard handle not found for seat '{0}'.")]
        KeyboardHandleNotFound(String),
        #[error("Pointer handle not found for seat '{0}'.")]
        PointerHandleNotFound(String),
        #[error("Touch handle not found for seat '{0}'.")]
        TouchHandleNotFound(String),
        #[error("Failed to initialize input event source in event loop: {0}")]
        EventSourceSetupError(String),
        #[error("Internal error in input system: {0}")]
        InternalError(String),
    }
    ```
    
    - **Begründung:** Diese Fehlerstruktur deckt die in genannten Fehler ab und erweitert sie um spezifischere Fälle für `libinput` und die Ereignisschleifenintegration. Das `CapabilityAdditionFailed` fasst Fehler von `seat.add_keyboard/pointer/touch` generisch zusammen.

#### 2.2. Submodul: `system::input::seat_manager`

**Zweck:** Definiert und verwaltet `SeatState` und `SeatHandler` für Eingabefokus und Capabilities.

**Datei:** `src/input/seat_manager/mod.rs` (oder `state.rs` und `handler.rs`)

- **Struktur `DesktopState` (Erweiterung für Input-Aspekte)**:
    
    Rust
    
    ```
    // In src/compositor/core/state.rs (oder wo DesktopState definiert ist)
    // ... existing fields ...
    // pub seat_state: SeatState<Self>, // Bereits vorhanden
    // pub seat: Seat<Self>,           // Bereits vorhanden
    // pub seat_name: String,          // Bereits vorhanden
    // pub keyboard_data_map: HashMap<String /* seat_name */, XkbKeyboardData>, // Bereits vorhanden
    // pub current_cursor_status: Arc<Mutex<CursorImageStatus>>, // Bereits vorhanden
    
    // Neu oder verfeinert für Fokusmanagement:
    pub pointer_location: Point<f64, Logical>, // Aktuelle globale Zeigerposition
    pub last_active_window_per_workspace: HashMap<WorkspaceId, Weak<ManagedWindow>>, // Für Fokuswiederherstellung
    pub active_input_surface: Option<Weak<WlSurface>>, // Die Oberfläche, die aktuell den logischen Input-Fokus hat (Tastatur, Zeiger, Touch)
                                                       // Dies kann komplexer sein, wenn Zeiger- und Tastaturfokus getrennt sind.
                                                       // Smithay's Seat/KeyboardHandle/PointerHandle verwalten den Fokus auf Protokollebene.
                                                       // Dieses Feld könnte den "logischen" Anwendungsfokus speichern.
    ```
    
- **Implementierung `SeatHandler` für `DesktopState`**: ()
    
    - **`type KeyboardFocus = WlSurface;`**
    - **`type PointerFocus = WlSurface;`**
    - **`type TouchFocus = WlSurface;`**
    - **`fn seat_state(&mut self) -> &mut SeatState<Self>`**: Gibt `&mut self.seat_state` zurück.
    - **`fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&Self::KeyboardFocus>)`**:
        1. `tracing::debug!(seat_name = %seat.name(), old_focus = ?self.active_input_surface.as_ref().and_then(|w| w.upgrade()).map(|s| s.id()), new_focus = ?focused.map(|s| s.id()), "SeatHandler::focus_changed (keyboard) called");`
        2. **Wichtig**: Diese Methode wird von `KeyboardHandle::set_focus` aufgerufen. Sie sollte primär dazu dienen, _interne Compositor-Zustände_ zu aktualisieren, die von der Fokusänderung abhängen, nicht umgekehrt den Fokus erneut zu setzen.
        3. Die `KeyboardHandle` sendet bereits `wl_keyboard.enter/leave`.
        4. Aktualisiere `self.active_input_surface` (oder eine spezifischere Variable für Tastaturfokus).
        5. Benachrichtige die Domänenschicht (`workspace_manager_service` oder einen dedizierten `FocusManagerService` in der Domäne) über die Fokusänderung, damit diese z.B. Fenstertitel in der UI aktualisieren oder Policy-Entscheidungen treffen kann.
            
            Rust
            
            ```
            // Beispiel:
            // let domain_window_id = find_domain_window_id_for_surface(focused);
            // block_on(self.workspace_manager_service.notify_focus_changed(domain_window_id));
            ```
            
    - **`fn cursor_image(&mut self, seat: &Seat<Self>, image: CursorImageStatus)`**:
        1. `tracing::trace!(seat_name = %seat.name(), status = ?image, "Cursor-Image-Anfrage erhalten");`
        2. `let mut current_status_guard = self.current_cursor_status.lock().unwrap();`
        3. `*current_status_guard = image;`
        4. Renderer muss benachrichtigt werden, den Cursor neu zu zeichnen. Dies kann über ein Flag geschehen oder indem der Renderer den `current_cursor_status` direkt abfragt. Der Renderer braucht auch die `pointer_location`.
            - Wenn `image == CursorImageStatus::Hidden`, setzt der Renderer den Cursor unsichtbar.
            - Wenn `image == CursorImageStatus::Surface(surface)`, muss der Renderer den Puffer dieser `surface` als Cursor verwenden (Hotspot-Informationen sind in `SurfaceData` oder als Teil von `SurfaceAttributes`).
            - Wenn `image == CursorImageStatus::Named(name)`, muss eine Cursor-Theming-Logik den Namen in eine Textur auflösen (z.B. über `libwayland-cursor` oder eine eigene Implementierung, die XCursor-Themes parst). Diese Logik gehört ggf. in ein Hilfsmodul.
- **Funktion `pub fn create_seat(state: &mut DesktopState, display_handle: &DisplayHandle, seat_name: String) -> Result<Seat<DesktopState>, InputError>`**:
    
    1. `tracing::info!("Erstelle neuen Seat: {}", seat_name);`
    2. `let seat = state.seat_state.new_wl_seat(display_handle, seat_name.clone(), Some(tracing::Span::current()));`
    3. `seat.user_data().insert_if_missing(WaylandSeatData::default);` // Standard-UserData für wl_seat
    4. **Capabilities initialisieren (aber noch nicht setzen, wenn Geräte noch nicht bekannt):**
        - `state.keyboard_data_map.insert(seat_name.clone(), XkbKeyboardData::new(&Default::default())?);` (Mit Default-XKB-Config, wird später aktualisiert).
    5. `tracing::info!("Seat '{}' erfolgreich erstellt. Capabilities werden beim Hinzufügen von Geräten gesetzt.", seat_name);`
    6. Speichere `seat.clone()` in `state.seat` (falls dies der primäre Seat ist) und `state.active_seat_name`.
    7. `Ok(seat)`

#### 2.3. Submodul: `system::input::libinput_handler`

**Zweck:** Initialisiert und konfiguriert das `LibinputInputBackend` und leitet dessen Events an spezifische Handler weiter.

**Datei:** `src/input/libinput_handler/session_interface.rs`

- **Struct `LibinputSessionManager`**:
    
    Rust
    
    ```
    use smithay::backend::session::{Session, Signal as SessionSignal, SessionNotifier};
    use std::rc::Rc; // Oder Arc, wenn thread-übergreifend benötigt
    use calloop::LoopHandle;
    use super::super::core::state::DesktopState; // Pfad anpassen
    
    // Diese Struktur wird die Logik für das Öffnen/Schließen von Geräten kapseln,
    // basierend auf dem gewählten Session-Typ (logind, direct).
    // Für diese Spezifikation ist sie ein Platzhalter.
    pub struct LibinputSessionManager {
        // notifier: SessionNotifier, // Von Smithay's Session
        // session: Rc<dyn Session>, // Oder eine konkrete Session-Implementierung
    }
    
    impl LibinputSessionManager {
        // pub fn new(session: Rc<dyn Session>, loop_handle: LoopHandle<'static, DesktopState>) -> Self {
        //     let notifier = session.notifier(loop_handle).expect("Failed to create session notifier");
        //     Self { session, notifier }
        // }
    }
    
    // Implementiert smithay::backend::input::LibinputInterface
    impl smithay::backend::input::LibinputInterface for LibinputSessionManager {
        fn open_restricted(&mut self, path: &std::path::Path, flags: i32) -> Result<std::os::unix::io::RawFd, std::io::Error> {
            // self.session.open(path, flags)
            // Platzhalter:
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "Session Management nicht implementiert"))
        }
        fn close_restricted(&mut self, fd: std::os::unix::io::RawFd) {
            // self.session.close(fd);
            // Platzhalter:
            let _ = fd;
        }
    }
    ```
    
    - **Wichtig:** Die konkrete Implementierung hängt stark vom gewählten `Session`-Typ ab (`smithay::backend::session::direct::DirectSession` für Start ohne `logind`, `smithay::backend::session::logind::LogindSession` für `logind`-Integration). Die `Session` selbst muss korrekt initialisiert und in die `calloop`-Schleife integriert werden (Behandlung von `SessionSignal`). Dies ist ein komplexes Thema für sich und wird hier nur angerissen. Für eine minimale Lauffähigkeit kann eine Dummy-Implementierung verwendet werden, die immer Fehler zurückgibt oder `/dev/input/*` direkt öffnet (was Root-Rechte erfordert).

**Datei:** `src/input/libinput_handler/mod.rs` (oder `backend_init.rs` und `event_dispatcher.rs`)

- **Funktion `pub fn init_libinput_backend<S: Session + 'static>(loop_handle: &LoopHandle<'static, DesktopState>, session: Rc<S>) -> Result<LibinputInputBackend, InputError>`**:
    1. `tracing::info!("Initialisiere Libinput-Backend...");`
    2. `let session_interface = Rc::new(std::cell::RefCell::new(smithay::backend::session::libinput_session_interface(session)));` (Smithay stellt diese Hilfsfunktion bereit).
    3. `let mut libinput_context = libinput::Libinput::new_from_path(session_interface.clone());`
    4. `libinput_context.udev_assign_seat("seat0").map_err(|e| InputError::LibinputError(format!("Zuweisung zu udev seat0 fehlgeschlagen: {:?}", e)))?;`
    5. `let libinput_backend = LibinputInputBackend::new(libinput_context, Some(tracing::Span::current()));`
    6. `tracing::info!("Libinput-Backend erfolgreich initialisiert.");`
    7. `Ok(libinput_backend)`
- **Funktion `pub fn register_libinput_source(loop_handle: &LoopHandle<'static, DesktopState>, libinput_backend: LibinputInputBackend, seat_name: String) -> Result<calloop::Source<LibinputInputBackend>, InputError>`**:
    1. `let libinput_event_source = loop_handle.insert_source(libinput_backend, move |event, _metadata, desktop_state| { // desktop_state ist hier &mut DesktopState // Rufe den zentralen Event-Dispatcher auf super::event_dispatcher::process_input_event(desktop_state, event, &seat_name); }).map_err(|e| InputError::EventSourceSetupError(e.to_string()))?;`
    2. `Ok(libinput_event_source)` (Der Rückgabewert ist hier nicht ganz korrekt, `insert_source` gibt `RegistrationToken` oder `Source` zurück, abhängig von der calloop-Version und Methode). Korrekt wäre, dass der `LibinputInputBackend` selbst die Quelle ist. Die Logik ist, dass der `LibinputInputBackend` in die Schleife eingefügt wird.
- **Datei: `src/input/event_dispatcher.rs`**
    - **Funktion `pub fn process_input_event(desktop_state: &mut DesktopState, event: InputEvent<LibinputInputBackend>, seat_name: &str)`**: ()
        1. `let seat = match desktop_state.seat_state.seats().find(|s| s.name() == seat_name) { Some(s) => s.clone(), None => { tracing::error!("Seat '{}' nicht gefunden für Input-Event.", seat_name); return; } };`
        2. `match event { ... }` wie in detailliert.
            - **`InputEvent::DeviceAdded { device }`**:
                - `tracing::info!("Eingabegerät hinzugefügt: {} (Sys: {})", device.name(), device.sysname());`
                - Wenn `device.has_capability(libinput::DeviceCapability::Keyboard)` und `seat.get_keyboard().is_none()`:
                    - `let kbd_config = XkbConfig::default(); // Oder aus GlobalSettings laden`
                    - `match seat.add_keyboard(kbd_config, 200, 25) { Ok(_) => tracing::info!("Tastatur-Capability zu Seat '{}' hinzugefügt.", seat_name), Err(e) => tracing::error!("Fehler beim Hinzufügen der Tastatur-Capability: {}", e), };`
                    - (XkbKeyboardData muss ggf. aktualisiert werden)
                - Analog für `Pointer` und `Touch`.
            - **`InputEvent::DeviceRemoved { device }`**:
                - `tracing::info!("Eingabegerät entfernt: {}", device.name());`
                - Wenn `device.has_capability(libinput::DeviceCapability::Keyboard)`: `seat.remove_keyboard();`
                - Analog für `Pointer` und `Touch`.

#### 2.4. Submodul: `system::input::keyboard`

**Zweck:** Tastaturereignis-Übersetzung, XKB-Management.

**Datei:** `src/input/keyboard/xkb_config.rs`

- **Struct `XkbKeyboardData`**:
    
    Rust
    
    ```
    use xkbcommon::xkb;
    use smithay::input::keyboard::{KeyboardConfig, ModifiersState as SmithayModifiersState};
    use calloop::TimerHandle;
    use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
    use wayland_server::Weak;
    use smithay::utils::Serial;
    
    #[derive(Debug)] // TimerHandle ist nicht Debug
    pub struct XkbKeyboardData {
        pub context: xkb::Context,
        pub keymap: xkb::Keymap,
        pub state: xkb::State,
        pub repeat_timer: Option<TimerHandle>,
        pub repeat_info: Option<(u32 /* keycode */, xkb::Keycode /* xkb keycode */, SmithayModifiersState, std::time::Duration /* delay */, std::time::Duration /* rate */)>,
        pub focused_surface_on_seat: Option<Weak<WlSurface>>,
        pub repeat_key_serial: Option<Serial>,
        // Für Tastenwiederholung: Speichern des xkb-Keycodes, nicht nur des libinput-Keycodes.
    }
    
    impl XkbKeyboardData {
        pub fn new(config: &KeyboardConfig<'_>) -> Result<Self, InputError> {
            let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
            let keymap_name = config.keymap_name.as_deref().unwrap_or("default"); // Bessere Defaults nötig
            let rules = config.rules.as_deref().unwrap_or("evdev");
            let model = config.model.as_deref().unwrap_or("pc105");
            let layout = config.layout.as_deref().unwrap_or("us");
            let variant = config.variant.as_deref();
            let options = config.options.as_deref();
    
            tracing::debug!("Lade XKB Keymap: rules={}, model={}, layout={}, variant={:?}, options={:?}",
                rules, model, layout, variant, options);
    
            let mut keymap_builder = xkb::KeymapCompileArgsBuilder::new();
            keymap_builder.rules(rules);
            keymap_builder.model(model);
            keymap_builder.layout(layout);
            if let Some(v) = variant { keymap_builder.variant(v); }
            if let Some(o) = options { keymap_builder.options(o); }
    
    
            let keymap = match xkb::Keymap::new_from_names(
                &context,
                &keymap_builder.build(), // Verwende den Builder
                xkb::KEYMAP_COMPILE_NO_FLAGS,
            ) {
                Ok(km) => km,
                Err(_) => { // Fallback zu einfacherem Setup
                    tracing::warn!("Komplexe XKB-Keymap '{}' konnte nicht geladen werden, versuche Fallback (us).", keymap_name);
                    let fallback_args = xkb::KeymapCompileArgsBuilder::new()
                        .layout("us").build();
                    xkb::Keymap::new_from_names(&context, &fallback_args, xkb::KEYMAP_COMPILE_NO_FLAGS)
                        .map_err(|_| InputError::XkbConfigError { seat_name: "unknown".into(), message: "Fallback XKB Keymap (us) konnte nicht kompiliert werden".into() })?
                }
            };
    
            let state = xkb::State::new(&keymap);
            Ok(Self {
                context, keymap, state,
                repeat_timer: None, repeat_info: None, focused_surface_on_seat: None, repeat_key_serial: None
            })
        }
    }
    ```
    
- **`Default for KeyboardConfig`**: Wird benötigt, um `XkbKeyboardData::new(&Default::default())` aufrufen zu können.
    
    Rust
    
    ```
    // Ggf. in KeyboardConfig von Smithay oder hier lokal
    // impl Default for KeyboardConfig<'_> { ... }
    ```
    

**Datei:** `src/input/keyboard/key_event_translator.rs`

- **Funktion `pub fn handle_keyboard_key_event(...)`**: ()
    1. Hole `keyboard_handle = seat.get_keyboard().ok_or(...)?.clone();`
    2. Hole `xkb_data = desktop_state.keyboard_data_map.get_mut(seat_name).ok_or(...)?;`
    3. `let keycode = event.key_code();`
    4. `let xkb_keycode = keycode + 8; // Libinput keycodes sind XKB keycodes - 8`
    5. `let key_state_xkb = match event.state() { KeyState::Pressed => xkb::KeyDirection::Down, KeyState::Released => xkb::KeyDirection::Up, };`
    6. `xkb_data.state.update_key(xkb_keycode.into(), key_state_xkb);`
    7. `let smithay_mods_state = SmithayModifiersState { ... /* von xkb_data.state.serialize_mods etc. */ };`
    8. `keyboard_handle.modifiers(event.serial(), smithay_mods_state.clone(), Some(tracing::Span::current()));`
    9. Wenn `event.state() == KeyState::Pressed`:
        - `let serial = event.serial();`
        - `let time = event.time();`
        - `keyboard_handle.key(serial, time, xkb_keycode, KeyState::Pressed, Some(tracing::Span::current()));`
        - **Tastenwiederholung einrichten:**
            - `if let Some(timer) = xkb_data.repeat_timer.take() { timer.cancel(); }`
            - `if keyboard_handle.is_repeating(xkb_keycode) { ... }`
            - `let (delay, rate) = keyboard_handle.repeat_info();`
            - `xkb_data.repeat_info = Some((keycode, xkb_keycode.into(), smithay_mods_state, delay, rate));`
            - `xkb_data.repeat_key_serial = Some(serial);`
            - `let timer_seat_name = seat_name.to_string();`
            - `xkb_data.repeat_timer = Some(desktop_state.loop_handle.insert_timer(delay, move |ds: &mut DesktopState| { ... repeat_logic ... }).expect("Timer creation failed"));`
    10. Wenn `event.state() == KeyState::Released`:
        - `keyboard_handle.key(event.serial(), event.time(), xkb_keycode, KeyState::Released, Some(tracing::Span::current()));`
        - **Tastenwiederholung abbrechen:** `if xkb_data.repeat_info.map_or(false, |(_, rkc, ..)| rkc == xkb_keycode.into()) { ... cancel timer ... }`
- **Tastenwiederholungslogik im Timer-Callback:**
    1. Hole `xkb_data` für `timer_seat_name`.
    2. Wenn `xkb_data.repeat_info` `None` ist oder der Fokus gewechselt hat (prüfe `xkb_data.focused_surface_on_seat`), Timer abbrechen und `return;`.
    3. `let (keycode, xkb_keycode, mods_state, _, rate) = xkb_data.repeat_info.as_ref().unwrap().clone();`
    4. Hole aktuellen `seat` und `keyboard_handle`.
    5. `let new_serial = Serial::now();` // Wichtig: Neuer Serial für wiederholte Events
    6. `keyboard_handle.modifiers(new_serial, mods_state, Some(tracing::Span::current()));`
    7. `keyboard_handle.key(new_serial, current_time_ms(), xkb_keycode, KeyState::Pressed, Some(tracing::Span::current()));`
    8. `xkb_data.repeat_key_serial = Some(new_serial);`
    9. Timer mit `rate` neu planen.

**Datei:** `src/input/keyboard/focus.rs` (ersetzt `focus_handler_keyboard.rs`)

- **Funktion `pub fn set_keyboard_focus(desktop_state: &mut DesktopState, seat_name: &str, surface: Option<&WlSurface>, serial: Serial)`**:
    1. `tracing::debug!(seat = %seat_name, new_focus_surface = ?surface.map(|s| s.id()), ?serial, "Setze Tastaturfokus");`
    2. Hole `seat = desktop_state.seat_state.seats().find(|s| s.name() == seat_name).cloned().ok_or(...)`;
    3. Hole `keyboard = seat.get_keyboard().ok_or(...)?.clone();`
    4. Hole `xkb_data = desktop_state.keyboard_data_map.get_mut(seat_name).ok_or(...)?;`
    5. `let old_focus_wl_surface = xkb_data.focused_surface_on_seat.as_ref().and_then(|w| w.upgrade());`
    6. `if old_focus_wl_surface.as_ref() == surface { tracing::trace!("Tastaturfokus unverändert."); return Ok(()); }`
    7. `keyboard.set_focus(surface, serial, Some(tracing::Span::current()));` // Smithay sendet Enter/Leave
    8. `xkb_data.focused_surface_on_seat = surface.map(|s| s.downgrade());`
    9. // Domänenschicht über Fokusänderung informieren, falls `SeatHandler::focus_changed` nicht ausreicht
        
        Rust
        
        ```
        // let domain_window_id = surface.and_then(|s| find_domain_window_id_for_surface(desktop_state, s));
        // block_on(desktop_state.workspace_manager_service.notify_keyboard_focus_changed(domain_window_id));
        ```
        

#### 2.5. Submodul: `system::input::pointer`

**Zweck:** Maus-/Zeigerereignis-Verarbeitung, Cursor.

**Datei:** `src/input/pointer/mod.rs` (oder `event_translator.rs`, `focus.rs`, `cursor.rs`)

- **Funktion `pub fn handle_pointer_motion_event(...)`**: ()
    1. Hole `pointer_handle = seat.get_pointer().ok_or(...)?;`
    2. `desktop_state.pointer_location += event.delta();` // Einfache Akkumulation, ggf. an Bildschirmgrenzen klemmen.
    3. `let (new_focus_surface_option, surface_local_coords) = find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);`
    4. `pointer_handle.motion(event.time(), new_focus_surface_option.as_ref(), serial, desktop_state.pointer_location, surface_local_coords, Some(tracing::Span::current()));` (Smithay's `motion` sendet `enter`/`leave` und `motion`).
    5. Aktualisiere `desktop_state.active_input_surface` basierend auf `new_focus_surface_option`.
    6. Renderer Cursor-Position aktualisieren (indem Renderer `desktop_state.pointer_location` liest).
- **Funktion `find_surface_and_coords_at_global_point(desktop_state: &DesktopState, global_pos: Point<f64, Logical>) -> (Option<WlSurface>, Point<f64, Logical>)`**:
    1. Iteriere über `desktop_state.space.elements_under(global_pos)` (Smithay's Space liefert Fenster in korrekter Reihenfolge).
    2. Für jedes `Window` (unsere `ManagedWindow`-Implementierung):
        - Hole `wl_surface = window.wl_surface()`.
        - Prüfe, ob `wl_surface` eine Eingaberegion hat (`SurfaceData::input_region_surface_local`).
        - Transformiere `global_pos` in Oberflächen-lokale Koordinaten.
        - Wenn `global_pos` innerhalb der Eingaberegion (oder der Oberflächengeometrie, falls keine Eingaberegion):
            - Gib `(Some(wl_surface.clone()), surface_local_coords)` zurück.
    3. Sonst: `(None, global_pos)` (oder `(0.0, 0.0)` für lokale Coords).
- **Funktion `handle_pointer_button_event(...)`**: ()
    1. Hole `pointer_handle`.
    2. `pointer_handle.button(event.button(), event.button_state().into(), event.serial(), event.time(), Some(tracing::Span::current()));`
    3. Wenn `event.button_state() == ButtonState::Pressed`:
        - `let (focused_surface_option, _) = find_surface_and_coords_at_global_point(desktop_state, desktop_state.pointer_location);`
        - Wenn `focused_surface_option` `Some(surface_to_focus)`:
            - `set_keyboard_focus(desktop_state, seat_name, Some(&surface_to_focus), event.serial())?;` (Click-to-focus).
            - Hier könnte auch Logik für Fenstermanagement-Aktionen (Move/Resize-Start) basierend auf `surface_to_focus` und Klickposition (relativ zu Dekorationen) ausgelöst werden.
- **Funktion `handle_pointer_axis_event(...)`**: ()
    1. Hole `pointer_handle`.
    2. `pointer_handle.axis(event.time(), event.axis(), event.axis_source().into(), event.axis_value_discrete(), event.axis_value(smithay::utils:: SERIAL_COUNTER_RANGE), event.serial(), Some(tracing::Span::current()));` (Smithay-Signatur anpassen).

#### 2.6. Submodul: `system::input::touch`

**Zweck:** Touch-Ereignis-Verarbeitung.

**Datei:** `src/input/touch/mod.rs` (oder `event_translator.rs`, `focus.rs`)

- **Logik für `handle_touch_down_event`**: ()
    1. Hole `touch_handle = seat.get_touch().ok_or(...)?;`
    2. `let slot = event.slot().ok_or_else(|| InputError::InternalError("Touch down event ohne Slot ID".into()))?;`
    3. `let (focused_surface_option, surface_local_coords) = find_surface_and_coords_at_global_point(desktop_state, event.position_transformed(output_size));`
    4. Wenn `focused_surface_option` `Some(surface)`:
        - Speichere `surface.clone()` als Fokus für diesen `slot` (z.B. in einer `HashMap<TouchSlotId, WlSurface>` in `DesktopState`).
        - `touch_handle.down(event.serial(), event.time(), slot, surface_local_coords, &surface, Some(tracing::Span::current()));`
        - `set_keyboard_focus(desktop_state, seat_name, Some(&surface), event.serial())?;` (Touch-to-focus).
- **Logik für `handle_touch_up_event`**: ()
    1. Hole `touch_handle`. `let slot = event.slot().ok_or(...)`;
    2. `touch_handle.up(event.serial(), event.time(), slot, Some(tracing::Span::current()));`
    3. Entferne Fokus für diesen `slot` aus der internen Map.
- **Logik für `handle_touch_motion_event`**: ()
    1. Hole `touch_handle`. `let slot = event.slot().ok_or(...)`;
    2. Hole die fokussierte Oberfläche für diesen `slot` aus der internen Map.
    3. Transformiere `event.position_transformed(output_size)` in lokale Koordinaten dieser Oberfläche.
    4. `touch_handle.motion(event.serial(), event.time(), slot, surface_local_coords, Some(tracing::Span::current()));`
- **`handle_touch_frame_event`, `handle_touch_cancel_event`**: Rufen entsprechende `touch_handle`-Methoden auf.

#### 2.7. Submodul: `system::input::gestures`

Zweck: Grundlegende Gestenerkennung (Pinch, Swipe) aufbauend auf libinput-Events.

Datei: src/input/gestures/mod.rs

- **Initial:** Für Gesten wie `InputEvent::GesturePinchBegin/Update/End`, `InputEvent::GestureSwipeBegin/Update/End`:
    1. Logge das Ereignis mit `tracing::debug!`.
    2. **Zukünftige Erweiterung:**
        - Eine `GestureState`-Struktur pro aktivem Seat, die laufende Gesten verfolgt.
        - Bei `GestureSwipeBegin`: Starte eine "Swipe"-Geste.
        - Bei `GestureSwipeUpdate`: Akkumuliere `event.dx()`, `event.dy()`. Wenn ein Schwellenwert überschritten wird:
            - Wandle in eine Domänenaktion um (z.B. Workspace wechseln). Rufe z.B. `desktop_state.workspace_manager_service.switch_to_next_workspace().await;`.
        - Bei `GestureSwipeEnd`: Beende die Geste.
        - Ähnlich für Pinch-to-Zoom (könnte z.B. Skalierungsfaktor einer App oder des Desktops beeinflussen - komplexe Interaktion mit Compositor/Anwendung).
- **Abhängigkeiten:** Benötigt Zugriff auf Domänenservices (z.B. `WorkspaceManagerService`).

#### 2.8. Implementierungsschritte `system::input`

1. **Grundgerüst**: Verzeichnisse anlegen, `Cargo.toml` für `libinput`, `xkbcommon` etc.
2. **`errors.rs`**: `InputError` Enum definieren.
3. **`seat_manager/mod.rs`**:
    - `DesktopState`-Felder für Input/Fokus erweitern.
    - `SeatHandler` für `DesktopState` implementieren (`focus_changed`, `cursor_image`).
    - `create_seat` Funktion implementieren.
4. **`libinput_handler/session_interface.rs`**: `LibinputSessionManager` (ggf. mit Dummy-Implementierung für `open/close_restricted` initial).
5. **`libinput_handler/mod.rs`**: `init_libinput_backend`, `register_libinput_source`.
6. **`event_dispatcher.rs`**: `process_input_event` mit `match` für alle relevanten `InputEvent`-Typen und Delegation an Handler in `keyboard`, `pointer`, `touch`. Logik für `DeviceAdded/Removed`.
7. **`keyboard/xkb_config.rs`**: `XkbKeyboardData`-Struct und `new()`-Methode.
8. **`keyboard/key_event_translator.rs`**: `handle_keyboard_key_event` inklusive Tastenwiederholungslogik (Timer-Setup und Callback).
9. **`keyboard/focus.rs`**: `set_keyboard_focus` implementieren.
10. **`pointer/mod.rs`**: `handle_pointer_motion_event` (inkl. `find_surface_and_coords_at_global_point`), `handle_pointer_button_event`, `handle_pointer_axis_event`.
11. **`touch/mod.rs`**: `handle_touch_down/up/motion/frame/cancel_event`. Interne Verwaltung des Touch-Fokus pro Slot.
12. **`gestures/mod.rs`**: Basis-Logging für Gesten-Events.
13. **`src/input/mod.rs`**: Öffentliche API des Input-Moduls definieren (z.B. Initialisierungsfunktionen).
14. **Unit-Tests (anspruchsvoll, erfordert oft Mocking von `Seat`, `KeyboardHandle` etc. oder Integrationstests):**
    - Testen der XKB-Keymap-Erstellung.
    - Testen der Keycode-zu-Keysym/UTF-8-Übersetzung für einige Tasten.
    - Testen der Fokussetzungslogik (Keyboard, Pointer, Touch).
    - Testen der Event-Weiterleitung für verschiedene Eingabetypen.
    - Testen der `find_surface_and_coords_at_global_point`-Logik mit verschiedenen Fensterlayouts.

---

**Nächste Schritte für `system` (Skizze für weitere Module):**

- **`system::dbus_interfaces`**:
    - **Verantwortlichkeiten:** Clients für wichtige Systemdienste (NetworkManager, UPower, logind, Secrets, PolicyKit) und Server für `org.freedesktop.Notifications`.
    - **Technologie:** `zbus` (async).
    - **Struktur:** Pro Dienst ein Submodul (z.B. `dbus_interfaces::upower_client`).
    - **Jedes Client-Submodul:**
        - Definiert Proxy-Structs für die D-Bus-Interfaces des Dienstes.
        - Implementiert Methoden zum Abrufen von Eigenschaften und Aufrufen von Methoden des Dienstes.
        - Implementiert Signal-Handler, um auf D-Bus-Signale zu reagieren und diese in interne System-Events oder Domänenaufrufe zu übersetzen.
        - Fehlerbehandlung mit spezifischem `DBusInterfaceError`.
    - **`notifications_server`**: Implementiert den `org.freedesktop.Notifications`-D-Bus-Service. Leitet eingehende `Notify`-Aufrufe an `domain::user_centric_services::NotificationService::post_notification` weiter. Handhabt `GetCapabilities`, `CloseNotification`, `GetServerInformation`. Sendet `NotificationClosed`, `ActionInvoked` Signale.
- **`system::audio_management`**:
    - **Verantwortlichkeiten:** Steuerung der Systemlautstärke, Auswahl von Audio-Geräten, Verwaltung von Anwendungs-Streams.
    - **Technologie:** `pipewire-rs`.
    - **Struktur:** `client` (Core-Verbindung), `device_manager`, `stream_manager`, `volume_control`.
    - Interaktion mit PipeWire-Registry, um Geräte und Streams zu entdecken.
    - Nutzung von `PWStream` für Lautstärkeregelung etc.
    - Übersetzung von PipeWire-Events in interne System-Events oder Domänenaufrufe.
- **`system::mcp_client`**:
    - **Verantwortlichkeiten:** Sichere Kommunikation mit lokalen/remote MCP-Servern.
    - **Technologie:** `mcp_client_rs`.
    - Nimmt Anweisungen und Kontextdaten von `domain::user_centric_services::ai_interaction` entgegen.
    - Ruft Methoden des `mcp_client_rs::McpClient` auf.
    - Leitet Ergebnisse/Fehler an die Domänenschicht zurück.
    - Verwaltet API-Schlüssel sicher (über `dbus_interfaces::secrets_service`).
- **`system::window_mechanics`**:
    - **Verantwortlichkeiten:** Konkrete Umsetzung der Fenstermanagement-"Mechanik" basierend auf Richtlinien aus `domain::window_management_policy`.
    - Interagiert eng mit `system::compositor::xdg_shell` und `smithay::desktop::Space`.
    - Ruft `domain::window_management_policy_service::calculate_workspace_layout()` auf und wendet die resultierenden Geometrien auf `ManagedWindow`s an (z.B. über deren `send_configure`-Methoden oder direkte Manipulation im `Space`).
    - Implementiert Snapping während interaktiver Move/Resize-Operationen.
    - Koordiniert Client-Side vs. Server-Side Decorations.
- **`system::event_bridge`**:
    - Ein Modul, das `tokio::sync::broadcast` Kanäle oder ähnliche Mechanismen nutzt, um System-interne Events (z.B. "Neues Eingabegerät", "Netzwerkstatus geändert", "Batteriestand niedrig") für andere Systemmodule oder Adapter zur Domänenschicht bereitzustellen.
- **`system::power_management`**:
    - Interagiert mit `wlr-output-power-management-unstable-v1` (serverseitig im Compositor implementiert).
    - Interagiert mit `system::dbus_interfaces::logind` für Suspend/Hibernate.
    - Setzt DPMS-Modi für Outputs basierend auf `domain::global_settings_service` und Benutzeraktivität (`domain::common_events::UserActivityDetectedEvent`).

Diese detaillierte Ausarbeitung für `system::input` und die Skizzen für die weiteren Module sollten eine solide Basis für die Implementierung der gesamten Systemschicht bilden. Jedes dieser Module erfordert eine ähnliche Tiefe der Spezifikation, insbesondere bei der Interaktion mit externen Bibliotheken und Protokollen.

Kurzfassung: Rust (Edition 2021+), Nutzung von novade-core und novade-domain, zbus für D-Bus-Interaktionen, smithay (wo relevant für Compositor-Interaktionen), tokio (oft von zbus benötigt), thiserror für Fehler.

**Cargo.toml für `novade-system` (relevante Ergänzungen für D-Bus):**

Ini, TOML

```
[dependencies]
# ... (bestehende Abhängigkeiten wie novade-core, novade-domain, smithay, etc.) ...
zbus = { version = "3.15", default-features = false, features = ["tokio"] } # Aktuelle zbus 4.x Version prüfen, tokio feature ist wichtig
serde = { version = "1.0.197", features = ["derive"] }
serde_json = "1.0.115" # Falls einige D-Bus Properties/Methoden JSON verwenden
tracing = "0.1.40"
thiserror = "1.0.58"
async-trait = "0.1.79"
tokio = { version = "1.37.0", features = ["sync", "rt", "macros"] }
# Ggf. spezifische Crates für Freedesktop-Spezifikationen, falls zbus nicht alles abdeckt
# oder für komplexere Typen (z.B. `dbus-crossroads` für Server-Seite, obwohl zbus auch Server kann)
```

---

### Modul 3: `system::dbus_interfaces`

Zweck: Implementierung von Schnittstellen zur Interaktion mit etablierten System-D-Bus-Diensten sowie Bereitstellung eigener D-Bus-Schnittstellen, wo dies von der Architektur vorgesehen ist (z.B. org.freedesktop.Notifications).

Verantwortlichkeiten:

- Erstellen und Verwalten von D-Bus-Verbindungen (Session und System Bus).
- Implementierung von Clients (Proxies) für externe D-Bus-Dienste wie NetworkManager, UPower, logind, org.freedesktop.secrets, PolicyKit.
- Abrufen von Eigenschaften, Aufrufen von Methoden und Abonnieren von Signalen dieser Dienste.
- Übersetzung von D-Bus-Daten und -Signalen in interne System-Events oder Aufrufe an die Domänenschicht.
- Implementierung von D-Bus-Server-Objekten für Dienste, die NovaDE selbst bereitstellt (z.B. `org.freedesktop.Notifications`). **Design-Rationale:** Kapselung aller D-Bus-spezifischen Logik. Verwendung von `zbus` als moderne, asynchrone D-Bus-Bibliothek in Rust. Klare Trennung zwischen D-Bus-Protokoll-Interaktion und der Verarbeitungslogik in anderen System- oder Domänenmodulen.

#### 3.1. Submodul: `system::dbus_interfaces::common`

**Zweck:** Definition gemeinsamer Typen, Fehler und Hilfsfunktionen für alle D-Bus-Interaktionen.

**Datei:** `src/dbus_interfaces/common/errors.rs`

- **Enum `DBusInterfaceError`**:
    
    Rust
    
    ```
    use thiserror::Error;
    use zbus::Error as ZBusError;
    use zbus::names::ErrorName;
    
    #[derive(Debug, Error)]
    pub enum DBusInterfaceError {
        #[error("D-Bus connection failed: {0}")]
        ConnectionFailed(#[from] ZBusError), // Direkter Fehler von zbus beim Verbindungsaufbau
        #[error("Failed to create D-Bus proxy for service '{service}' path '{path}' interface '{interface}': {source}")]
        ProxyCreationFailed {
            service: String,
            path: String,
            interface: String,
            #[source]
            source: ZBusError,
        },
        #[error("D-Bus method call '{method}' on '{interface}' failed: {source}")]
        MethodCallFailed {
            method: String,
            interface: String,
            #[source]
            source: ZBusError,
        },
        #[error("Failed to get D-Bus property '{property}' from '{interface}': {source}")]
        PropertyGetFailed {
            property: String,
            interface: String,
            #[source]
            source: ZBusError,
        },
        #[error("Failed to set D-Bus property '{property}' on '{interface}': {source}")]
        PropertySetFailed {
            property: String,
            interface: String,
            #[source]
            source: ZBusError,
        },
        #[error("Failed to subscribe to D-Bus signal '{signal}' from '{interface}': {source}")]
        SignalSubscriptionFailed {
            signal: String,
            interface: String,
            #[source]
            source: ZBusError,
        },
        #[error("Received D-Bus error reply: {name} - {body}")]
        DBusErrorReply {
            name: ErrorName<'static>, // 'static hier, da ErrorName oft geklont wird
            body: String, // Oft der Message-Teil des D-Bus-Fehlers
        },
        #[error("Type conversion error during D-Bus operation: {0}")]
        TypeConversionError(String), // Wenn z.B. ein zvariant nicht in den erwarteten Rust-Typ passt
        #[error("Required D-Bus service '{service}' is not available or not activatable.")]
        ServiceUnavailable { service: String },
        #[error("D-Bus object path '{0}' not found for service.")]
        ObjectPathNotFound(String),
        #[error("D-Bus interface '{0}' not found on object.")]
        InterfaceNotFound(String),
        #[error("An internal error occurred in a D-Bus interface: {0}")]
        InternalError(String),
    }
    
    // Hilfsfunktion, um zbus::Error in DBusErrorReply zu konvertieren, falls es ein D-Bus-Fehler war
    impl From<ZBusError> for DBusInterfaceError {
        fn from(err: ZBusError) -> Self {
            if let ZBusError::MethodError(name, body, _) = err {
                DBusInterfaceError::DBusErrorReply { name: name.into_static(), body: body.unwrap_or_default() }
            } else {
                // Für andere ZBusError-Typen, die nicht MethodError sind,
                // könnte man spezifischere Mappings oder eine generische Variante haben.
                // Hier als Beispiel: fallback auf MethodCallFailed (kontextabhängig anpassen)
                DBusInterfaceError::MethodCallFailed {
                    method: "unknown".to_string(),
                    interface: "unknown".to_string(),
                    source: err,
                }
            }
        }
    }
    ```
    
    - **Begründung:** Fasst generische D-Bus-Fehler und spezifischere Fälle wie `ProxyCreationFailed` oder `ServiceUnavailable` zusammen. Die `From<ZBusError>` Implementierung hilft, `zbus`-Fehler direkt in benutzerdefinierte Fehler zu überführen.

**Datei:** `src/dbus_interfaces/common/connection_manager.rs`

- **Struct `DBusConnectionManager`**:
    
    Rust
    
    ```
    use zbus::{Connection, ConnectionBuilder, Address, Transport};
    use super::errors::DBusInterfaceError;
    use std::sync::{Arc, OnceLock}; // OnceLock für Singleton-Verbindungen
    use tokio::sync::Mutex; // Mutex, falls die Verbindung modifiziert werden kann (selten)
    
    static SESSION_BUS: OnceLock<Arc<Connection>> = OnceLock::new();
    static SYSTEM_BUS: OnceLock<Arc<Connection>> = OnceLock::new();
    
    #[derive(Debug, Clone)]
    pub struct DBusConnectionManager;
    
    impl DBusConnectionManager {
        /// Stellt die Session-Bus-Verbindung her (oder gibt die bestehende zurück).
        pub async fn session_bus() -> Result<Arc<Connection>, DBusInterfaceError> {
            if let Some(conn) = SESSION_BUS.get() {
                return Ok(conn.clone());
            }
            let conn = ConnectionBuilder::session()?
                .build()
                .await?;
            let arc_conn = Arc::new(conn);
            match SESSION_BUS.set(arc_conn.clone()) {
                Ok(_) => Ok(arc_conn),
                Err(existing_conn_arc) => Ok(existing_conn_arc.clone()), // Rennen gewonnen von anderem Thread
            }
        }
    
        /// Stellt die System-Bus-Verbindung her (oder gibt die bestehende zurück).
        pub async fn system_bus() -> Result<Arc<Connection>, DBusInterfaceError> {
            if let Some(conn) = SYSTEM_BUS.get() {
                return Ok(conn.clone());
            }
            let conn = ConnectionBuilder::system()?
                .build()
                .await?;
            let arc_conn = Arc::new(conn);
            match SYSTEM_BUS.set(arc_conn.clone()) {
                Ok(_) => Ok(arc_conn),
                Err(existing_conn_arc) => Ok(existing_conn_arc.clone()),
            }
        }
    
        /// Erstellt einen zbus Proxy.
        pub async fn create_proxy<'a, T: zbus::ProxyDefault + Send + Sync + 'static>(
            connection: Arc<Connection>,
            destination: &'static str, // Muss 'static sein für einige Proxy-Konstrukte
            path: &'static str,
        ) -> Result<T, DBusInterfaceError> {
            T::builder(&connection)
                .destination(destination)?
                .path(path)?
                .build()
                .await
                .map_err(|e| DBusInterfaceError::ProxyCreationFailed {
                    service: destination.to_string(),
                    path: path.to_string(),
                    interface: T::INTERFACE.unwrap_or("unknown").to_string(), // T::INTERFACE ist Option<&'static str>
                    source: e,
                })
        }
    }
    ```
    
    - **Zweck:** Stellt sicher, dass nur eine Verbindung pro Bus-Typ (Session/System) besteht und verwaltet wird (`OnceLock` für Singleton-Pattern). Bietet eine Hilfsfunktion zum Erstellen von Proxies.
    - **Methoden:** `session_bus() -> Result<Arc<Connection>>`, `system_bus() -> Result<Arc<Connection>>`, `create_proxy<T>(...)`.
    - **Zustand:** Die `OnceLock`-statischen Variablen halten die globalen Verbindungen.

**Datei:** `src/dbus_interfaces/common/mod.rs`

- `pub mod errors;`
- `pub mod connection_manager;`
- `pub use errors::DBusInterfaceError;`
- `pub use connection_manager::DBusConnectionManager;`

#### 3.2. Submodul: `system::dbus_interfaces::upower_client`

Zweck: Client für den org.freedesktop.UPower Dienst zur Abfrage von Energieinformationen (Batteriestatus, Deckelzustand etc.).

Interaktion mit Domäne: Sendet UPowerEvent (neu zu definierendes Event in system::event_bridge oder direkt an einen Domänen-Service) an die Domänenschicht (z.B. domain::power_management_policy oder einen allgemeinen SystemStatusService).

**Datei:** `src/dbus_interfaces/upower_client/types.rs`

- **Enums (Spiegelung der D-Bus-Typen von UPower):**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize}; // Für Events
    use zbus::zvariant::Type; // Für D-Bus Typ-Annotationen
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
    #[repr(u32)] // Entspricht den UPower-Enum-Werten
    pub enum PowerDeviceType {
        Unknown = 0,
        LinePower = 1,
        Battery = 2,
        Ups = 3,
        Monitor = 4,
        Mouse = 5,
        Keyboard = 6,
        Pda = 7,
        Phone = 8,
        MediaPlayer = 9,
        Tablet = 10,
        Computer = 11,
    }
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
    #[repr(u32)]
    pub enum PowerState {
        Unknown = 0,
        Charging = 1,
        Discharging = 2,
        Empty = 3,
        FullyCharged = 4,
        PendingCharge = 5,
        PendingDischarge = 6,
    }
    
    #[derive(Debug, Clone, PartialEq, Type, Serialize, Deserialize)]
    pub struct PowerDeviceDetails {
        pub native_path: String, // z.B. /sys/class/power_supply/BAT0
        pub vendor: String,
        pub model: String,
        pub serial: String,
        pub update_time: u64, // Unix-Timestamp
        pub device_type: PowerDeviceType,
        pub online: bool,
        pub energy: f64, // Wh (Watt-hours)
        pub energy_empty: f64,
        pub energy_full: f64,
        pub energy_full_design: f64,
        pub energy_rate: f64, // Watt (aktuelle Leistung)
        pub voltage: f64,
        pub time_to_empty: u64, // Sekunden
        pub time_to_full: u64, // Sekunden
        pub percentage: f64, // 0.0 - 100.0
        pub temperature: f64, // Celsius
        pub is_rechargeable: bool,
        pub capacity: f64, // Prozentsatz der Design-Kapazität
        pub technology: u32, // Enum UPowerTechnology
        pub warning_level: u32, // Enum UPowerWarningLevel
        pub state: PowerState,
        pub icon_name: String,
        // Weitere Felder nach Bedarf aus `org.freedesktop.UPower.Device`
    }
    ```
    
- **Event-Struktur (für `system::event_bridge`):**
    
    Rust
    
    ```
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum UPowerEvent {
        DeviceAdded(String /* object_path */),
        DeviceRemoved(String /* object_path */),
        DeviceChanged(String /* object_path */), // Wenn Eigenschaften eines Geräts sich ändern
        LidStateChanged(bool /* is_closed */),
        OnBatteryChanged(bool /* is_on_battery */),
        PowerSupplyChanged, // Generelles Event, wenn sich etwas an der Stromversorgung ändert
    }
    ```
    

**Datei:** `src/dbus_interfaces/upower_client/proxies.rs`

- **zbus Proxy-Definitionen (manuell oder mit `zbus::proxy` Makro):**
    - **`UPowerProxy` für `org.freedesktop.UPower` auf `/org/freedesktop/UPower`**:
        
        Rust
        
        ```
        use zbus::proxy;
        use zbus::zvariant::{OwnedObjectPath, Value};
        use super::types::PowerDeviceDetails; // Annahme: PowerDeviceDetails ist als Value deserialisierbar
        
        #[proxy(
            interface = "org.freedesktop.UPower",
            default_service = "org.freedesktop.UPower",
            default_path = "/org/freedesktop/UPower"
        )]
        trait UPower {
            fn enumerate_devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
            fn get_display_device(&self) -> zbus::Result<OwnedObjectPath>;
            fn get_critical_action(&self) -> zbus::Result<String>;
        
            #[zbus(property)]
            fn lid_is_closed(&self) -> zbus::Result<bool>;
            #[zbus(property)]
            fn lid_is_present(&self) -> zbus::Result<bool>;
            #[zbus(property)]
            fn on_battery(&self) -> zbus::Result<bool>;
            #[zbus(property)]
            fn daemon_version(&self) -> zbus::Result<String>;
        
            #[zbus(signal)]
            fn device_added(&self, device_path: OwnedObjectPath) -> zbus::Result<()>;
            #[zbus(signal)]
            fn device_removed(&self, device_path: OwnedObjectPath) -> zbus::Result<()>;
            // Weitere Signale: LidIsClosed, LidIsOpened, DeviceChanged (oft als PropertiesChanged auf Device-Interface)
        }
        ```
        
    - **`UPowerDeviceProxy` für `org.freedesktop.UPower.Device` (dynamischer Pfad)**:
        
        Rust
        
        ```
        #[proxy(
            interface = "org.freedesktop.UPower.Device",
            default_service = "org.freedesktop.UPower"
            // default_path wird dynamisch gesetzt
        )]
        trait UPowerDevice {
            fn refresh(&self) -> zbus::Result<()>;
            fn get_history(&self, type_: &str, timespan: u32, resolution: u32) -> zbus::Result<Vec<(u32, f64, u32)>>; // Array of (time, value, state)
            fn get_statistics(&self, type_: &str) -> zbus::Result<Vec<(f64, f64)>>; // Array of (value, accuracy)
        
            // Alle Eigenschaften aus PowerDeviceDetails als #[zbus(property)]
            #[zbus(property)] fn native_path(&self) -> zbus::Result<String>;
            #[zbus(property)] fn vendor(&self) -> zbus::Result<String>;
            #[zbus(property)] fn model(&self) -> zbus::Result<String>;
            #[zbus(property)] fn serial(&self) -> zbus::Result<String>;
            // ... alle weiteren Properties aus PowerDeviceDetails ...
            #[zbus(property)] fn state(&self) -> zbus::Result<super::types::PowerState>;
            #[zbus(property)] fn type_(&self) -> zbus::Result<super::types::PowerDeviceType>; // type ist ein Keyword
            #[zbus(property, name = "Type")] // Expliziter Name für D-Bus
            fn device_type_prop(&self) -> zbus::Result<super::types::PowerDeviceType>;
        
        
            // Oft wird `org.freedesktop.DBus.Properties.PropertiesChanged` Signal auf diesem Interface verwendet
        }
        ```
        
    - **Wichtig:** `zbus` erfordert, dass Enums, die direkt als D-Bus-Typen verwendet werden (wie `PowerState`, `PowerDeviceType`), `TryFrom<Value<'a>>` und `Into<Value<'static>>` implementieren, oder `zbus::zvariant::Type` und `serde` für automatische Konvertierung. `#[repr(u32)]` und `Type` sollten hier helfen.

**Datei:** `src/dbus_interfaces/upower_client/service.rs` (oder `client.rs`)

- **Struct `UPowerClientService`**:
    - **Felder:**
        - `connection: Arc<Connection>`
        - `proxy: UPowerProxy<'static>` (Proxy benötigt eine Lebenszeit, oft an die Connection gebunden)
        - `device_proxies: Arc<tokio::sync::Mutex<HashMap<OwnedObjectPath, UPowerDeviceProxy<'static>>>>`
        - `event_publisher: tokio::sync::broadcast::Sender<UPowerEvent>` (aus `system::event_bridge`)
        - `is_initialized: Arc<tokio::sync::watch::Sender<bool>>` (um Signale erst nach Initialisierung zu verarbeiten)
    - **Konstruktor `pub async fn new(event_publisher: tokio::sync::broadcast::Sender<UPowerEvent>) -> Result<Self, DBusInterfaceError>`**:
        1. `connection = DBusConnectionManager::system_bus().await?;`
        2. `proxy = UPowerProxy::new(&connection).await?;`
        3. Initialisiert `device_proxies` als leere Map.
        4. Initialisiert `is_initialized` Sender.
        5. Gibt `Self` zurück.
    - **Methode `pub async fn initialize_and_listen(&self) -> Result<(), DBusInterfaceError>`**:
        1. Ruft `proxy.enumerate_devices().await?` auf, um initiale Geräte zu bekommen.
        2. Für jedes Gerät: `self.add_device_proxy(device_path).await?;`
        3. Abonniert Signale des `UPowerProxy`:
            - `device_added_stream = proxy.receive_device_added().await?;` -> `self.handle_device_added(path).await;`
            - `device_removed_stream = proxy.receive_device_removed().await?;` -> `self.handle_device_removed(path).await;`
            - (Signale für LidIsClosed, LidIsOpened, OnBatteryChanged auf dem UPower-Interface, falls vorhanden, oder über PropertiesChanged)
        4. Abonniert `org.freedesktop.DBus.Properties.PropertiesChanged` auf dem `UPowerProxy` für Änderungen an `LidIsClosed`, `OnBattery`.
            
            Rust
            
            ```
            // let properties_changed_stream = self.proxy.receive_properties_changed().await?;
            // tokio::spawn(handle_property_changes_stream(properties_changed_stream, self.event_publisher.clone()));
            ```
            
        5. Setzt `*self.is_initialized.send(true).is_ok();`.
        6. Startet eine `tokio::task` für jeden Signal-Stream, der die Events verarbeitet.
    - **Private Methode `async fn add_device_proxy(&self, device_path: OwnedObjectPath) -> Result<(), DBusInterfaceError>`**:
        1. `let device_proxy = UPowerDeviceProxy::builder(&self.connection).path(device_path.clone())?.build().await?;`
        2. Abonniert `org.freedesktop.DBus.Properties.PropertiesChanged` auf `device_proxy`.
            
            Rust
            
            ```
            // let device_props_stream = device_proxy.receive_properties_changed().await?;
            // tokio::spawn(handle_device_property_changes_stream(device_path.clone(), device_props_stream, self.event_publisher.clone()));
            ```
            
        3. `self.device_proxies.lock().await.insert(device_path, device_proxy);`
        4. `self.event_publisher.send(UPowerEvent::DeviceAdded(device_path.into_inner().into_string())).ok();`
    - **Private Methode `async fn handle_device_added(&self, device_path: OwnedObjectPath)`**: Ruft `add_device_proxy`.
    - **Private Methode `async fn handle_device_removed(&self, device_path: OwnedObjectPath)`**: Entfernt aus `device_proxies`, sendet `DeviceRemoved` Event.
    - **Öffentliche Methoden zum Abrufen von Daten (Beispiele):**
        - `pub async fn get_all_device_details(&self) -> Result<Vec<PowerDeviceDetails>, DBusInterfaceError>`: Iteriert `device_proxies`, ruft alle Properties jedes Geräts ab und konstruiert `PowerDeviceDetails`.
        - `pub async fn is_lid_closed(&self) -> Result<bool, DBusInterfaceError>`: Ruft `proxy.lid_is_closed().await?`.
        - `pub async fn is_on_battery(&self) -> Result<bool, DBusInterfaceError>`: Ruft `proxy.on_battery().await?`.
- **Signal-Handler-Tasks (Beispiel für `PropertiesChanged` auf UPowerProxy):**
    
    Rust
    
    ```
    // async fn handle_upower_property_changes_stream(
    //     mut stream: zbus::fdo::PropertiesChangedStream<'_>, // Korrekter Stream-Typ
    //     event_publisher: tokio::sync::broadcast::Sender<UPowerEvent>,
    // ) {
    //     while let Some(signal) = stream.next().await {
    //         if let Ok(args) = signal.args() {
    //             if args.interface_name() == "org.freedesktop.UPower" {
    //                 if args.changed_properties().contains_key("LidIsClosed") {
    //                     if let Some(Value::Bool(closed)) = args.changed_properties().get("LidIsClosed") {
    //                         event_publisher.send(UPowerEvent::LidStateChanged(*closed)).ok();
    //                     }
    //                 }
    //                 if args.changed_properties().contains_key("OnBattery") {
    //                    // ... ähnlich ...
    //                    event_publisher.send(UPowerEvent::PowerSupplyChanged).ok(); // Generisches Event
    //                 }
    //             }
    //         }
    //     }
    // }
    ```
    

**Datei:** `src/dbus_interfaces/upower_client/mod.rs`

- `pub mod types;`
- `pub mod proxies;` // Ist oft intern, wenn Service die Fassade ist
- `pub mod service;`
- `pub use service::UPowerClientService;`
- `pub use types::{PowerDeviceDetails, PowerDeviceType, PowerState, UPowerEvent};`

#### 3.3. Implementierungsschritte `system::dbus_interfaces::upower_client`

1. **Grundgerüst**: Verzeichnis, `mod.rs`.
2. **`types.rs`**: `PowerDeviceType`, `PowerState`, `PowerDeviceDetails`, `UPowerEvent` definieren. `serde` und `zbus::zvariant::Type` implementieren.
3. **`proxies.rs`**: `UPowerProxy` und `UPowerDeviceProxy` mit `#[zbus::proxy]` definieren. Alle relevanten Methoden und Properties aus der UPower-Spezifikation aufnehmen.
4. **`service.rs`**: `UPowerClientService`-Struktur definieren.
    - `new()`-Konstruktor: D-Bus-Verbindung herstellen, Hauptproxy erstellen.
    - `initialize_and_listen()`: Initiale Geräte laden, Signal-Handler für `DeviceAdded`/`Removed` und `PropertiesChanged` (sowohl auf Hauptproxy als auch auf Geräteproxies) einrichten. Diese Handler laufen in eigenen `tokio::spawn`-Tasks.
    - `add_device_proxy()`: Erstellt, speichert und abonniert Signale für einen Geräteproxy.
    - Öffentliche Getter-Methoden (`get_all_device_details`, `is_lid_closed`, etc.) implementieren, die Properties von den Proxies abrufen.
5. **Event-Publishing**: Sicherstellen, dass bei relevanten Signalempfängen oder Zustandsänderungen die definierten `UPowerEvent`s über den `event_publisher` gesendet werden.
6. **Fehlerbehandlung**: Alle `zbus::Error`-Fälle in `DBusInterfaceError` umwandeln und propagieren.
7. **Unit-/Integrationstests**:
    - **Schwierig ohne laufenden D-Bus-Dienst.** Man könnte `zbus::MockConnection` verwenden, um D-Bus-Interaktionen zu mocken.
    - Testen der Proxy-Generierung.
    - Testen der Property-Abfrage-Logik gegen einen gemockten Dienst.
    - Testen der Signal-Verarbeitung (indem man Signale im Mock simuliert).
    - Testen der korrekten Event-Erzeugung.

---

**Nächste Schritte für `system::dbus_interfaces` (Skizze für weitere Dienste):**

- **`logind_client`**:
    - **Zweck:** Interaktion mit `org.freedesktop.login1` für Sitzungsmanagement (Sperren, Suspend, Shutdown-Signale).
    - **Proxies:** `LogindManagerProxy` (`/org/freedesktop/login1`), `LogindSessionProxy` (`/org/freedesktop/login1/session/self`).
    - **Signale:** `PrepareForShutdown`, `PrepareForSleep`, `Lock`, `Unlock` auf Session-Objekt.
    - **Events:** `LogindEvent::PrepareForShutdown(bool is_reboot)`, `LogindEvent::PrepareForSleep(bool is_suspending)`, `LogindEvent::SessionLocked`, `LogindEvent::SessionUnlocked`.
    - **Methoden:** `LockSession()`, `UnlockSession()`, `CanSuspend()`, `Suspend(bool interactive)`, etc. an Domäne weiterleiten oder von dort empfangen.
- **`network_manager_client`**:
    - **Zweck:** Abfrage von Netzwerkstatus, verfügbaren Verbindungen, Signalstärke (WLAN), IP-Adressen.
    - **Proxies:** `NetworkManagerProxy`, `NMDeviceProxy`, `NMActiveConnectionProxy`, `NMAccessPointProxy`, etc.
    - **Signale:** `StateChanged`, `DeviceAdded/Removed`, `PropertiesChanged` auf verschiedenen Objekten.
    - **Events:** `NetworkEvent::ConnectivityChanged(ConnectivityState)`, `NetworkEvent::WifiDeviceAdded/Removed`, `NetworkEvent::WiredDeviceAdded/Removed`, `NetworkEvent::ActiveConnectionChanged { ... }`.
    - **Typen:** `ConnectivityState` (Disconnected, Connecting, Limited, Full), `NetworkDeviceDetails`, `ActiveConnectionDetails`.
- **`notifications_server`**:
    - **Zweck:** Implementierung des `org.freedesktop.Notifications` D-Bus-Servers.
    - **Technologie:** `zbus` Server-Fähigkeiten (`#[dbus_interface(...)]` auf einem Struct).
    - **Methoden (D-Bus):** `Notify`, `CloseNotification`, `GetCapabilities`, `GetServerInformation`.
    - **Interaktion:** Leitet `Notify`-Aufrufe an `domain::user_centric_services::NotificationService::post_notification` weiter.
    - **Signale (D-Bus):** `NotificationClosed`, `ActionInvoked`. Reagiert auf `NotificationDismissedEvent` und `NotificationActionInvokedEvent` aus der Domäne, um diese D-Bus-Signale zu senden.
- **`secrets_service_client`**:
    - **Zweck:** Client für `org.freedesktop.secrets` zum sicheren Speichern und Abrufen von Geheimnissen (z.B. API-Keys für `mcp_client`).
    - **Proxies:** `SecretServiceProxy`, `SecretCollectionProxy`, `SecretItemProxy`.
    - **Methoden:** `CreateCollection`, `CreateItem`, `GetSecret`, `SearchItems`, `Unlock`.
    - **Interaktion:** Wird von anderen Systemmodulen (z.B. `mcp_client`) oder ggf. Domänendiensten genutzt. UI-Interaktion für Prompts (Unlock) wird oft vom Secret Service selbst gehandhabt (z.B. GNOME Keyring).
- **`policykit_client`**:
    - **Zweck:** Client für `org.freedesktop.PolicyKit1.Authority` zur Autorisierung privilegierter Aktionen.
    - **Proxy:** `PolicyKitAuthorityProxy`.
    - **Methode:** `CheckAuthorization`.
    - **Interaktion:** Wird von System- oder Domänenmodulen aufgerufen, bevor eine privilegierte Aktion ausgeführt wird. UI-Interaktion für Passwortabfragen wird vom PolicyKit-Agenten des Systems gehandhabt.
- **`xdg_desktop_portal_handler`**:
    - **Zweck:** Dies ist kein Client, sondern die Backend-Logik, die von den XDG Desktop Portal D-Bus-Server-Implementierungen (die NovaDE bereitstellt) aufgerufen wird.
    - **Schnittstellen:** Definiert Traits oder konkrete Methoden, die von den Portal-D-Bus-Objekten aufgerufen werden.
    - **Beispiele:**
        - Für `org.freedesktop.portal.FileChooser`: `async fn open_file_dialog(...) -> Result<Vec<PathBuf>, PortalError>`. Interagiert mit der UI-Schicht, um den Dialog anzuzeigen.
        - Für `org.freedesktop.portal.Screenshot`: `async fn take_screenshot(interactive: bool, region: Option<RectInt>) -> Result<PathBuf, PortalError>`. Interagiert mit `system::compositor::screencopy`.
        - Für `org.freedesktop.portal.ScreenCast`: Interagiert mit Compositor und PipeWire.
    - **Wichtig:** Die eigentlichen D-Bus-Server-Objekte für die Portale werden typischerweise in einem separaten Prozess oder zumindest einem dedizierten D-Bus-Dienst innerhalb von NovaDE laufen. Dieses Modul hier liefert die Logik, die diese D-Bus-Methoden ausführt.

Diese detaillierte Ausarbeitung für `system::dbus_interfaces::upower_client` und die Skizzen für die weiteren Dienste legen einen klaren Pfad für die Implementierung der D-Bus-Interaktionen fest. Jedes Client-Modul erfordert sorgfältige Definition der Proxy-Interfaces und die Übersetzung der D-Bus-spezifischen Daten und Signale in die internen Strukturen und Events von NovaDE.

---

### Modul 3: `system::dbus_interfaces` (Fortsetzung)

#### 3.4. Submodul: `system::dbus_interfaces::logind_client`

Zweck: Client für den org.freedesktop.login1 Dienst zur Abfrage und Steuerung von Sitzungsinformationen und Systemzuständen (Suspend, Shutdown).

Interaktion mit Domäne: Sendet LogindEvents an die Domänenschicht (z.B. domain::power_management_policy, domain::common_events). Empfängt Befehle (z.B. LockSession) von der Domäne oder UI über die Domäne.

**Datei:** `src/dbus_interfaces/logind_client/types.rs`

- **Event-Struktur (für `system::event_bridge` oder direkt an Domänen-Services):**
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    use crate::dbus_interfaces::common::DBusObjectPath; // Typalias für String oder OwnedObjectPath
    
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum LogindPowerOperation {
        Suspend,
        Hibernate,
        HybridSleep,
        SuspendThenHibernate,
        Reboot,
        PowerOff,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum LogindEvent {
        PrepareForShutdown(bool /* is_reboot */),
        PrepareForSleep(bool /* is_suspending_to_ram_not_disk */), // true für Suspend, false für Hibernate
        SessionLocked(DBusObjectPath /* session_id */),
        SessionUnlocked(DBusObjectPath /* session_id */),
        SessionRemoved(DBusObjectPath /* session_id */),
        SystemIdleHintChanged(bool /* is_idle */), // Falls logind IdleHint sendet
    }
    
    #[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq)] // zbus::zvariant::Type
    pub struct SessionDetails {
        pub id: String, // Session ID
        pub user_id: u32,
        pub user_name: String,
        pub seat_id: String,
        pub seat_path: DBusObjectPath,
        pub vtnr: u32,
        pub display: String, // z.B. ":0"
        pub remote: bool,
        pub remote_host: String,
        pub remote_user: String,
        pub service: String, // z.B. "gdm-password", "tty"
        pub desktop: String, // z.B. "NovaDE"
        pub scope: String, // z.B. "system-user"
        pub leader_pid: u32,
        pub audit_session_id: u32,
        pub session_class: String, // z.B. "user"
        pub session_type: String, // z.B. "wayland", "x11"
        pub active: bool,
        pub state: String, // z.B. "active", "online", "closing"
        pub idle_hint: bool,
        pub idle_since_hint_usec: u64, // Mikrosekunden
        pub locked_hint: bool,
    }
    ```
    

**Datei:** `src/dbus_interfaces/logind_client/proxies.rs`

- **`LogindManagerProxy` für `org.freedesktop.login1.Manager` auf `/org/freedesktop/login1`**:
    
    Rust
    
    ```
    use zbus::{proxy, zvariant::{OwnedObjectPath, Type, Value, Dict}};
    use super::types::SessionDetails;
    
    #[proxy(
        interface = "org.freedesktop.login1.Manager",
        default_service = "org.freedesktop.login1",
        default_path = "/org/freedesktop/login1"
    )]
    trait LogindManager {
        fn get_session(&self, session_id: &str) -> zbus::Result<OwnedObjectPath>;
        fn get_session_by_pid(&self, pid: u32) -> zbus::Result<OwnedObjectPath>;
        fn get_user(&self, uid: u32) -> zbus::Result<OwnedObjectPath>;
        fn get_user_by_pid(&self, pid: u32) -> zbus::Result<OwnedObjectPath>;
        fn get_seat(&self, seat_id: &str) -> zbus::Result<OwnedObjectPath>;
        fn list_sessions(&self) -> zbus::Result<Vec<(String, u32, String, String, OwnedObjectPath)>>; // (id, uid, user, seat, path)
        fn list_users(&self) -> zbus::Result<Vec<(u32, String, OwnedObjectPath)>>; // (uid, name, path)
        fn list_seats(&self) -> zbus::Result<Vec<(String, OwnedObjectPath)>>; // (id, path)
    
        fn inhibit(&self, what: &str, who: &str, why: &str, mode: &str) -> zbus::Result<zbus::zvariant::Fd>; // Returns FD for inhibitor lock
        fn can_power_off(&self) -> zbus::Result<String>; // "yes", "no", "challenge"
        fn can_reboot(&self) -> zbus::Result<String>;
        fn can_suspend(&self) -> zbus::Result<String>;
        fn can_hibernate(&self) -> zbus::Result<String>;
        fn can_hybrid_sleep(&self) -> zbus::Result<String>;
        fn can_suspend_then_hibernate(&self) -> zbus::Result<String>;
    
        fn power_off(&self, interactive: bool) -> zbus::Result<()>;
        fn reboot(&self, interactive: bool) -> zbus::Result<()>;
        fn suspend(&self, interactive: bool) -> zbus::Result<()>;
        fn hibernate(&self, interactive: bool) -> zbus::Result<()>;
        fn hybrid_sleep(&self, interactive: bool) -> zbus::Result<()>;
        fn suspend_then_hibernate(&self, interactive: bool) -> zbus::Result<()>;
        // TerminateSeat, TerminateSession, TerminateUser...
    
        #[zbus(signal)]
        fn session_new(&self, session_id: String, object_path: OwnedObjectPath) -> zbus::Result<()>;
        #[zbus(signal)]
        fn session_removed(&self, session_id: String, object_path: OwnedObjectPath) -> zbus::Result<()>;
        #[zbus(signal)]
        fn seat_new(&self, seat_id: String, object_path: OwnedObjectPath) -> zbus::Result<()>;
        #[zbus(signal)]
        fn seat_removed(&self, seat_id: String, object_path: OwnedObjectPath) -> zbus::Result<()>;
        #[zbus(signal)]
        fn prepare_for_shutdown(&self, start: bool) -> zbus::Result<()>; // true = about to shut down, false = cancelled
        #[zbus(signal)]
        fn prepare_for_sleep(&self, start: bool) -> zbus::Result<()>;   // true = about to suspend/hibernate, false = cancelled
    }
    ```
    
- **`LogindSessionProxy` für `org.freedesktop.login1.Session` (dynamischer Pfad)**:
    
    Rust
    
    ```
    #[proxy(
        interface = "org.freedesktop.login1.Session",
        default_service = "org.freedesktop.login1"
    )]
    trait LogindSession {
        fn terminate(&self) -> zbus::Result<()>;
        fn activate(&self) -> zbus::Result<()>;
        fn lock(&self) -> zbus::Result<()>;
        fn unlock(&self) -> zbus::Result<()>;
        fn set_idle_hint(&self, idle: bool) -> zbus::Result<()>;
        fn set_locked_hint(&self, locked: bool) -> zbus::Result<()>;
        // Kill(who: &str, signal_number: i32)
        // TakeControl(force: bool)
        // ReleaseControl()
        // TakeDevice(major: u32, minor: u32) -> zbus::Result<(zbus::zvariant::Fd, bool)>
        // ReleaseDevice(major: u32, minor: u32)
        // PauseDeviceComplete(major: u32, minor: u32)
        // SetBrightness(subsystem: &str, name: &str, value: u32)
    
        // Properties (viele, können über Properties.GetAll abgerufen werden)
        #[zbus(property)] fn id(&self) -> zbus::Result<String>;
        #[zbus(property)] fn user(&self) -> zbus::Result<(u32, OwnedObjectPath)>; // (uid, user_path)
        #[zbus(property)] fn name(&self) -> zbus::Result<String>; // Username
        #[zbus(property)] fn timestamp(&self) -> zbus::Result<u64>; // usec
        #[zbus(property)] fn timestamp_monotonic(&self) -> zbus::Result<u64>; // usec
        #[zbus(property)] fn vtnr(&self) -> zbus::Result<u32>;
        #[zbus(property)] fn seat(&self) -> zbus::Result<(String, OwnedObjectPath)>; // (seat_id, seat_path)
        #[zbus(property)] fn display(&self) -> zbus::Result<String>;
        #[zbus(property)] fn remote(&self) -> zbus::Result<bool>;
        #[zbus(property)] fn remote_host(&self) -> zbus::Result<String>;
        #[zbus(property)] fn remote_user(&self) -> zbus::Result<String>;
        #[zbus(property)] fn service(&self) -> zbus::Result<String>;
        #[zbus(property)] fn desktop(&self) -> zbus::Result<String>;
        #[zbus(property)] fn scope(&self) -> zbus::Result<String>;
        #[zbus(property)] fn leader(&self) -> zbus::Result<u32>; // PID
        #[zbus(property)] fn audit(&self) -> zbus::Result<u32>; // Audit Session ID
        #[zbus(property, name = "Class")] fn class_prop(&self) -> zbus::Result<String>; // "user", "greeter", ...
        #[zbus(property, name = "Type")] fn type_prop(&self) -> zbus::Result<String>;   // "x11", "wayland", "tty"
        #[zbus(property)] fn active(&self) -> zbus::Result<bool>;
        #[zbus(property)] fn state(&self) -> zbus::Result<String>;
        #[zbus(property)] fn idle_hint(&self) -> zbus::Result<bool>;
        #[zbus(property)] fn idle_since_hint(&self) -> zbus::Result<u64>; // usec
        #[zbus(property)] fn locked_hint(&self) -> zbus::Result<bool>;
    
        #[zbus(signal)]
        fn pause_device(&self, major: u32, minor: u32, type_: &str) -> zbus::Result<()>; // type: "pause", "force-pause", "timeout-pause"
        #[zbus(signal)]
        fn resume_device(&self, major: u32, minor: u32, fd_idx: zbus::zvariant::Fd, type_: &str) -> zbus::Result<()>; // type: "resume"
        #[zbus(signal)]
        fn lock(&self) -> zbus::Result<()>; // Sitzung wurde gesperrt
        #[zbus(signal)]
        fn unlock(&self) -> zbus::Result<()>; // Sitzung wurde entsperrt
    }
    ```
    

**Datei:** `src/dbus_interfaces/logind_client/service.rs`

- **Struct `LogindClientService`**:
    - **Felder:** `connection: Arc<Connection>`, `manager_proxy: LogindManagerProxy<'static>`, `session_proxies: Arc<tokio::sync::Mutex<HashMap<String /*session_id*/, LogindSessionProxy<'static>>>>`, `event_publisher: tokio::sync::broadcast::Sender<LogindEvent>`.
    - **Konstruktor `new(...)`**: Stellt Verbindung her, erstellt `manager_proxy`.
    - **Methode `initialize_and_listen()`**:
        1. `list_sessions()` vom `manager_proxy` abrufen, um initiale Sessions zu cachen und ggf. Proxies zu erstellen.
        2. `LogindManagerProxy`-Signale abonnieren (`SessionNew`, `SessionRemoved`, `PrepareForShutdown`, `PrepareForSleep`).
            - `SessionNew`: Erstelle `LogindSessionProxy`, speichere in `session_proxies`, abonniere dessen `Lock`/`Unlock`-Signale.
            - `SessionRemoved`: Entferne Proxy.
            - `PrepareForShutdown/Sleep`: Sende `LogindEvent` an `event_publisher`.
        3. `LogindSessionProxy`-Signale (`Lock`, `Unlock`) für jede aktive Session abonnieren und entsprechende `LogindEvent`s senden.
    - **Öffentliche Methoden (Beispiele):**
        - `pub async fn get_current_session_details(&self) -> Result<Option<SessionDetails>, DBusInterfaceError>`: Ruft `GetSessionByPid(std::process::id())` auf, dann alle Properties vom Session-Proxy.
        - `pub async fn lock_current_session(&self) -> Result<(), DBusInterfaceError>`: Ruft `Lock()` auf dem aktuellen Session-Proxy auf.
        - `pub async fn can_power_off(&self) -> Result<String, DBusInterfaceError>`: Ruft `manager_proxy.can_power_off()`.
        - `pub async fn power_off(&self, interactive: bool) -> Result<(), DBusInterfaceError>`: Ruft `manager_proxy.power_off(interactive)`. (Benötigt PolicyKit-Autorisierung, die von logind gehandhabt wird).

**Implementierungsschritte `logind_client`**:

1. `types.rs`: `LogindEvent`, `SessionDetails`, `LogindPowerOperation` definieren.
2. `proxies.rs`: `LogindManagerProxy` und `LogindSessionProxy` definieren.
3. `service.rs`: `LogindClientService` implementieren (Konstruktor, Initialisierung/Signal-Listener, öffentliche Methoden).
4. Tests mit gemockter D-Bus-Verbindung.

---

#### 3.5. Submodul: `system::dbus_interfaces::network_manager_client`

Zweck: Client für org.freedesktop.NetworkManager.

Interaktion: Sendet NetworkManagerEvents.

(Struktur analog zu upower_client und logind_client)

- **`types.rs`**: Enums (`NMState`, `NMDeviceType`, `NMConnectivityState`), Structs (`NetworkDeviceDetails`, `AccessPointDetails`, `ActiveConnectionDetails`), Event (`NetworkManagerEvent`).
- **`proxies.rs`**: `NetworkManagerProxy`, `NMDeviceProxy`, `NMActiveConnectionProxy`, `NMAccessPointProxy`, `NMSettingsConnectionProxy` etc.
- **`service.rs`**: `NetworkManagerClientService` mit Logik zum Auflisten von Geräten, Verbindungen, APs; Abonnieren von `StateChanged`, `DeviceAdded/Removed`, `PropertiesChanged` etc.

---

#### 3.6. Submodul: `system::dbus_interfaces::secrets_service_client`

Zweck: Client für org.freedesktop.Secret.Service.

Interaktion: Stellt Methoden zum Speichern/Abrufen von Geheimnissen bereit, die von anderen Systemmodulen (z.B. mcp_client) genutzt werden.

(Struktur analog zu upower_client)

- **`types.rs`**: Structs (`Secret`, `SecretItemAttributes`).
- **`proxies.rs`**: `SecretServiceProxy`, `SessionProxy` (für D-Bus-Session der Secret Service API), `CollectionProxy`, `ItemProxy`, `PromptProxy`.
- **`service.rs`**: `SecretsServiceClientService` mit Methoden wie `async fn store_secret(collection_alias: &str, label: &str, secret_content: &[u8], attributes: HashMap<String, String>, replace: bool) -> Result<DBusObjectPath, DBusInterfaceError>`, `async fn retrieve_secret(item_path: &DBusObjectPath) -> Result<Vec<u8>, DBusInterfaceError>`, `async fn search_items(attributes: HashMap<String, String>) -> Result<Vec<DBusObjectPath>, DBusInterfaceError>`. Handhabt Unlock-Prompts (oft delegiert an den Secret Service Agenten).

---

#### 3.7. Submodul: `system::dbus_interfaces::policykit_client`

Zweck: Client für org.freedesktop.PolicyKit1.Authority.

Interaktion: Stellt eine Methode zur Autorisierungsprüfung bereit.

(Struktur analog, aber einfacher, da meist nur eine Hauptmethode)

- **`types.rs`**: Enum `PolicyKitAuthorizationResult` (allow, challenge, deny). Structs für `Subject` (pid, uid), `ActionId`.
- **`proxies.rs`**: `PolicyKitAuthorityProxy`.
- **`service.rs`**: `PolicyKitClientService` mit Methode `async fn check_authorization(action_id: &str, subject_pid: Option<u32>, details: HashMap<String, String>, flags: u32 /* PolicyKitCheckAuthorizationFlags */) -> Result<PolicyKitAuthorizationResult, DBusInterfaceError>`.

---

#### 3.8. Submodul: `system::dbus_interfaces::notifications_server`

Zweck: Implementierung des org.freedesktop.Notifications D-Bus-Servers.

Interaktion: Empfängt Notify-Aufrufe und leitet sie an domain::user_centric_services::NotificationService weiter. Sendet NotificationClosed und ActionInvoked Signale basierend auf Events aus der Domänenschicht.

**Datei:** `src/dbus_interfaces/notifications_server/mod.rs` (kann `interface.rs`, `service_object.rs` enthalten)

- **Struct `FreedesktopNotificationsServer`** (Das D-Bus-Objekt):
    
    Rust
    
    ```
    use zbus::dbus_interface;
    use crate::domain::user_centric_services::{NotificationService, Notification, NotificationUrgency as DomainUrgency, NotificationAction as DomainAction}; // Domain Traits/Typen
    use crate::domain::user_centric_services::notifications_core::types::NotificationInput; // Für Notify
    use crate::domain::shared_types::ApplicationId;
    use std::sync::Arc;
    use tokio::sync::Mutex; // Für NotificationService Handle
    use zbus::zvariant::{Value, Dict, Array};
    use zbus::SignalContext;
    use super::common::DBusInterfaceError; // Eigener Fehlertyp
    
    pub struct FreedesktopNotificationsServer {
        notification_service: Arc<Mutex<dyn NotificationService>>, // Injizierter Domain-Service
        // Ggf. ein tokio::sync::broadcast::Receiver für Domain-Events (NotificationDismissedEvent etc.)
        // um D-Bus Signale zu senden.
    }
    
    impl FreedesktopNotificationsServer {
        pub fn new(notification_service: Arc<Mutex<dyn NotificationService>>) -> Self {
            Self { notification_service }
            // Hier den Event-Receiver von notification_service abonnieren und Task starten,
            // der Domain-Events in D-Bus-Signale umwandelt.
        }
    
        // Hilfsmethode zur Konvertierung von D-Bus Urgency zu Domain Urgency
        fn to_domain_urgency(level: u8) -> DomainUrgency {
            match level {
                0 => DomainUrgency::Low,
                1 => DomainUrgency::Normal,
                2 => DomainUrgency::Critical,
                _ => DomainUrgency::Normal, // Fallback
            }
        }
    }
    
    #[dbus_interface(name = "org.freedesktop.Notifications")]
    impl FreedesktopNotificationsServer {
        async fn get_capabilities(&self) -> Vec<String> {
            // Fähigkeiten, die NovaDE unterstützt, z.B. "body", "actions", "persistence", "icon-static"
            vec![
                "body".to_string(),
                "actions".to_string(),
                "persistence".to_string(), // Wenn Benachrichtigungen gespeichert werden
                "icon-static".to_string(),
                "body-markup".to_string(), // Wenn Pango-Markup im Body unterstützt wird
                // "sound"
            ]
        }
    
        async fn notify(
            &self,
            app_name: String,
            replaces_id: u32, // ID der zu ersetzenden Benachrichtigung (0 für neue)
            app_icon: String,  // Icon-Name oder Pfad
            summary: String,
            body: String,
            actions: Vec<String>, // Actions als flache Liste: [key1, label1, key2, label2, ...]
            hints: Dict<'_, String, Value<'_>>, // zbus Dict für a{sv}
            expire_timeout: i32, // Millisekunden, -1 für Default, 0 für persistent (laut Spezifikation)
        ) -> Result<u32, zbus::fdo::Error> { // Gibt die neue Notification ID zurück
            tracing::info!("D-Bus Notify: app='{}', summary='{}'", app_name, summary);
    
            let mut domain_actions = Vec::new();
            for chunk in actions.chunks_exact(2) {
                domain_actions.push(DomainAction {
                    key: chunk[0].clone(),
                    label: chunk[1].clone(),
                    // action_type wird hier nicht direkt übergeben, müsste ggf. aus Hints oder Konvention abgeleitet werden
                    // oder Aktionen sind immer "Callback" für D-Bus.
                    action_type: crate::domain::user_centric_services::notifications_core::types::NotificationActionType::Callback,
                });
            }
    
            let urgency_hint = hints.get("urgency")
                .and_then(|v| v.downcast_ref::<Value<'_>>()) // Value in Value ist seltsam, eher direkt u8 oder byte
                .and_then(|v_inner| v_inner.try_into().ok()) // u8
                .map(Self::to_domain_urgency)
                .unwrap_or(DomainUrgency::Normal);
    
            let category_hint = hints.get("category")
                                .and_then(|v| v.downcast_ref::<String>())
                                .cloned();
    
            // TODO: 'replaces_id' Logik implementieren (alte Notification mit dieser ID entfernen/aktualisieren)
            // TODO: 'app_icon' und 'hints' genauer verarbeiten (image-data, sound etc.)
    
            let notification_input = NotificationInput {
                application_name: app_name, // Optional: ApplicationId::new(app_name)
                application_icon: if app_icon.is_empty() { None } else { Some(app_icon) },
                summary,
                body: if body.is_empty() { None } else { Some(body) },
                actions: domain_actions,
                urgency: urgency_hint,
                transient: hints.get("transient").and_then(|v| v.try_into().ok()).unwrap_or(false),
                category: category_hint,
                hints: hints.iter().map(|(k,v)| (k.to_string(), serde_json::to_value(v).unwrap_or(serde_json::Value::Null))).collect(), // Konvertiere zbus::Value zu serde_json::Value
                timeout_ms: if expire_timeout == 0 { Some(0) } // 0 für persistent laut D-Bus
                             else if expire_timeout > 0 { Some(expire_timeout as u32) }
                             else { None }, // -1 für Default
            };
    
            let mut service_guard = self.notification_service.lock().await;
            match service_guard.post_notification(notification_input).await {
                // Die zurückgegebene u32 ID muss für D-Bus eindeutig sein.
                // Der Domain-Service verwendet Uuid. Hier muss eine Abbildung erfolgen,
                // z.B. eine laufende u32-ID, die der Uuid zugeordnet wird.
                // Für Einfachheit hier: Hash der Uuid (nicht ideal, da Kollisionen möglich)
                // Besser: Map<Uuid, u32> im Server halten.
                Ok(domain_id) => {
                    let dbus_id =贫穷的男子哈希(domain_id); // Vereinfacht
                    Ok(dbus_id)
                }
                Err(e) => {
                    tracing::error!("Fehler beim Posten der Benachrichtigung an den Domain-Service: {:?}", e);
                    Err(zbus::fdo::Error::Failed(format!("Interner Fehler beim Verarbeiten der Benachrichtigung: {}", e)))
                }
            }
        }
    
        async fn close_notification(&self, id: u32) -> zbus::fdo::Result<()> {
            tracing::info!("D-Bus CloseNotification für ID: {}", id);
            // TODO: ID von u32 (D-Bus) zu Uuid (Domain) mappen
            // let domain_id = map_dbus_id_to_domain_id(id);
            // let mut service_guard = self.notification_service.lock().await;
            // match service_guard.dismiss_notification(domain_id).await {
            //     Ok(_) => Ok(()),
            //     Err(domain::user_centric_services::NotificationError::NotFound(_)) => {
            //         // Gemäß Spezifikation kein Fehler, wenn ID unbekannt ist
            //         Ok(())
            //     }
            //     Err(e) => Err(zbus::fdo::Error::Failed(format!("Fehler beim Schließen: {}", e))),
            // }
            Ok(()) // Platzhalter
        }
    
        async fn get_server_information(&self) -> (String, String, String, String) {
            (
                "NovaDE Notification Server".to_string(), // name
                "NovaDE Team".to_string(),                // vendor
                "0.1.0".to_string(),                      // version
                "1.2".to_string(),                        // spec_version
            )
        }
    
        #[dbus_interface(signal)]
        async fn notification_closed(ctxt: &SignalContext<'_>, id: u32, reason: u32) -> zbus::Result<()>;
        // reason: 1=expired, 2=dismissed by user, 3=closed by call to CloseNotification, 4=undefined
    
        #[dbus_interface(signal)]
        async fn action_invoked(ctxt: &SignalContext<'_>, id: u32, action_key: String) -> zbus::Result<()>;
    }
    ```
    
- **Logik zur Signal-Weiterleitung:**
    - Der `FreedesktopNotificationsServer` muss `NotificationDismissedEvent` und `NotificationActionInvokedEvent` vom `NotificationService` abonnieren.
    - Wenn ein `NotificationDismissedEvent { notification_id, reason }` empfangen wird:
        - `dbus_id = map_domain_id_to_dbus_id(notification_id);`
        - `dbus_reason = match reason { DismissReason::User => 2, DismissReason::Timeout => 1, ... };`
        - `FreedesktopNotificationsServer::notification_closed(ctxt, dbus_id, dbus_reason).await;` (Benötigt `SignalContext`).
    - Wenn ein `NotificationActionInvokedEvent { notification_id, action_key }` empfangen wird:
        - `dbus_id = map_domain_id_to_dbus_id(notification_id);`
        - `FreedesktopNotificationsServer::action_invoked(ctxt, dbus_id, action_key).await;`

#### 3.9. Implementierungsschritte `system::dbus_interfaces` (Fortsetzung)

5. **`logind_client` implementieren**: Typen, Proxies, Service. Signal-Handler für `PrepareForShutdown/Sleep`, `SessionNew/Removed`, `Lock/Unlock`. Tests.
6. **`network_manager_client` implementieren**: Typen, Proxies, Service. Signal-Handler für relevante NM-Signale. Tests.
7. **`secrets_service_client` implementieren**: Typen, Proxies, Service. Methoden für Speichern/Abrufen. Tests.
8. **`policykit_client` implementieren**: Typen, Proxy, Service. `check_authorization`-Methode. Tests.
9. **`notifications_server` implementieren**:
    - D-Bus-Interface-Struct `FreedesktopNotificationsServer`.
    - Implementierung der Methoden (`Notify`, `CloseNotification`, etc.), die den `domain::NotificationService` aufrufen.
    - ID-Mapping zwischen D-Bus `u32` und Domain `Uuid` implementieren (z.B. `HashMap<u32, Uuid>` und `HashMap<Uuid, u32>`).
    - Task starten, der Domain-Events (`NotificationDismissedEvent`, `NotificationActionInvokedEvent`) abonniert und entsprechende D-Bus-Signale (`notification_closed`, `action_invoked`) über den `SignalContext` sendet.
    - Registrierung des D-Bus-Objekts auf dem Session-Bus.
10. **`xdg_desktop_portal_handler`** (wird später detailliert, da es von UI-Dialogen und Compositor-Funktionen abhängt).

---

### Modul 4: `system::audio_management`

Zweck: Integration mit PipeWire zur Steuerung der Systemlautstärke, Auswahl von Audio-Geräten und Verwaltung von Anwendungs-Streams.

Verantwortlichkeiten: Aufbau und Verwaltung der PipeWire-Verbindung, Auflisten von Audio-Geräten (Sinks, Sources) und Streams, Setzen/Abfragen von Lautstärke und Mute-Status, Auswahl von Standardgeräten.

Design-Rationale: PipeWire als moderner Standard für Audio unter Linux. Kapselung der PipeWire-spezifischen Logik. Bereitstellung einer abstrahierten Schnittstelle für die Domänen- und UI-Schicht.

Technologie: pipewire-rs Crate.

#### 4.1. Submodul: `system::audio_management::types`

**Datei:** `src/audio_management/types.rs`

- **Enum `AudioDeviceType`**:
    
    Rust
    
    ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum AudioDeviceType {
        Sink,    // Wiedergabegerät (z.B. Lautsprecher, Kopfhörer)
        Source,  // Aufnahmegerät (z.B. Mikrofon)
        Other,
    }
    ```
    
- **Struct `AudioDevice`**:
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    use uuid::Uuid; // Interne ID für das Domänenobjekt
    
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AudioDevice {
        pub internal_id: Uuid, // Eigene ID, da PipeWire IDs u32 sind und sich ändern können
        pub pipewire_id: u32,  // Die aktuelle PipeWire Node ID
        pub name: String,      // z.B. "alsa_output.pci-0000_00_1f.3.analog-stereo"
        pub description: String, // Menschenlesbar, z.B. "Built-in Audio Analog Stereo"
        pub device_type: AudioDeviceType,
        pub volume_percent: u8, // 0-100 (oder höher, falls >100% unterstützt)
        pub is_muted: bool,
        pub is_default: bool,   // Ob es das Standardgerät seines Typs ist
        // Optional: channel_map, sample_format, etc.
    }
    ```
    
- **Struct `AudioStream`** (repräsentiert einen Anwendungs-Audiostream):
    
    Rust
    
    ```
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct AudioStream {
        pub internal_id: Uuid,
        pub pipewire_id: u32, // PipeWire Client/Stream ID
        pub application_name: Option<String>, // Name der Anwendung, die den Stream erzeugt
        pub media_role: Option<String>, // z.B. "Music", "Video", "Game", "Notification"
        pub volume_percent: u8,
        pub is_muted: bool,
        pub target_device_pw_id: Option<u32>, // PipeWire ID des Geräts, mit dem der Stream verbunden ist
    }
    ```
    
- **Event-Struktur (für `system::event_bridge` oder direkt an Domäne):**
    
    Rust
    
    ```
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum AudioEvent {
        DeviceListChanged(Vec<AudioDevice>),
        DefaultDeviceChanged { device_type: AudioDeviceType, new_default_pw_id: Option<u32> },
        DeviceVolumeChanged { device_pw_id: u32, new_volume_percent: u8, is_muted: bool },
        StreamListChanged(Vec<AudioStream>),
        StreamVolumeChanged { stream_pw_id: u32, new_volume_percent: u8, is_muted: bool },
        ServerConnectionStateChanged(bool /* is_connected */),
    }
    ```
    

#### 4.2. Submodul: `system::audio_management::errors`

**Datei:** `src/audio_management/errors.rs`

- **Enum `AudioManagementError`**:
    
    Rust
    
    ```
    use thiserror::Error;
    
    #[derive(Debug, Error)]
    pub enum AudioManagementError {
        #[error("PipeWire connection failed: {0}")]
        ConnectionFailed(String),
        #[error("PipeWire context error: {0}")]
        ContextError(String),
        #[error("PipeWire main loop error: {0}")]
        MainLoopError(String),
        #[error("PipeWire core error: {0}")]
        CoreError(String), // Generischer Fehler von pipewire-rs
        #[error("Failed to create PipeWire proxy or object: {0}")]
        ProxyCreationFailed(String),
        #[error("PipeWire object not found (ID: {0})")]
        ObjectNotFound(u32),
        #[error("Invalid parameter for PipeWire operation: {0}")]
        InvalidParameter(String),
        #[error("Operation timed out: {0}")]
        Timeout(String),
        #[error("Type conversion error in PipeWire data: {0}")]
        TypeConversionError(String),
        #[error("Audio device or stream is in an unexpected state: {0}")]
        InvalidState(String),
        #[error("An internal error occurred in audio management: {0}")]
        InternalError(String),
    }
    // Hilfsimplementierungen, um Fehler von pipewire-rs zu wrappen
    impl From<pipewire::Error> for AudioManagementError {
        fn from(err: pipewire::Error) -> Self {
            AudioManagementError::CoreError(err.to_string())
        }
    }
    ```
    

#### 4.3. Submodul: `system::audio_management::client`

**Zweck:** Kernlogik für die PipeWire-Verbindung, Event-Loop-Integration und Objektverwaltung.

**Datei:** `src/audio_management/client/mod.rs` (oder `service.rs`)

- **Struct `PipeWireClientService`**:
    - **Felder:**
        - `main_loop: Arc<pipewire::MainLoop>` (muss in eigenem Thread laufen oder in `calloop` integriert werden)
        - `context: Arc<pipewire::Context>`
        - `core: Arc<pipewire::Core>`
        - `registry: Arc<pipewire::Registry>`
        - `devices: Arc<tokio::sync::RwLock<HashMap<u32 /*pw_id*/, AudioDevice>>>`
        - `streams: Arc<tokio::sync::RwLock<HashMap<u32 /*pw_id*/, AudioStream>>>`
        - `default_sink_id: Arc<tokio::sync::RwLock<Option<u32>>>`
        - `default_source_id: Arc<tokio::sync::RwLock<Option<u32>>>`
        - `event_publisher: tokio::sync::broadcast::Sender<AudioEvent>`
        - `registry_listener: Option<pipewire::registry::Listener>` (muss `'static` sein oder anders verwaltet werden)
        - `core_listener: Option<pipewire::core::CoreListener>`
        - `loop_thread_handle: Option<std::thread::JoinHandle<()>>` (Falls MainLoop in eigenem Thread)
    - **Konstruktor `pub async fn new(event_publisher: tokio::sync::broadcast::Sender<AudioEvent>) -> Result<Self, AudioManagementError>`**:
        1. `pipewire::init()` aufrufen.
        2. `MainLoop::new(None)?` erstellen.
        3. `Context::new(&main_loop)?` erstellen.
        4. `Core::new(&context, None)?` (Verbindung zum PipeWire-Daemon herstellen).
        5. `Registry::new(&core)?` erstellen.
        6. Referenzen (`Arc`) für diese Objekte erstellen.
        7. `registry_listener` einrichten:
            - Im `global` Callback: Objekte filtern nach Typ (`PipewireObject::Node` für Geräte, `PipewireObject::Client` oder `PipewireObject::Stream` für Streams).
            - Für Nodes: Prüfen, ob es Audio Sinks/Sources sind (via Properties). `AudioDevice` erstellen, in `devices` speichern. `AudioEvent::DeviceListChanged` senden.
            - Für Streams: `AudioStream` erstellen, in `streams` speichern. `AudioEvent::StreamListChanged` senden.
            - `Metadata`-Objekt beobachten, um Standardgeräte zu finden (`default.audio.sink`, `default.audio.source`). `AudioEvent::DefaultDeviceChanged` senden.
        8. `core_listener` für `info` (um Server-Verbindungsstatus zu bekommen) und `error` einrichten.
        9. Wenn `main_loop` in eigenem Thread: `std::thread::spawn(move || main_loop_ref.run());`.
        10. Gibt `Self` zurück.
    - **Methode `pub async fn shutdown(&self)`**: Stoppt den `main_loop`-Thread sauber.
    - **Öffentliche Getter-Methoden (Beispiele):**
        - `pub async fn get_audio_devices(&self) -> Vec<AudioDevice>`: Gibt Klon von `self.devices.read().await.values()` zurück.
        - `pub async fn get_audio_streams(&self) -> Vec<AudioStream>`
        - `pub async fn get_default_sink(&self) -> Option<AudioDevice>`
        - `pub async fn get_default_source(&self) -> Option<AudioDevice>`
    - **Öffentliche Setter-Methoden (Beispiele):**
        - `pub async fn set_device_volume(&self, device_pw_id: u32, volume_percent: u8, is_muted: bool) -> Result<(), AudioManagementError>`:
            1. Findet das `Device` oder `Node` Proxy-Objekt für `device_pw_id` (muss im Registry-Handler gecacht werden).
            2. Erstellt `SpaPodBuilder` mit den neuen Lautstärkeparametern (`Props` mit `mute`, `channelVolumes`).
            3. Ruft `node_proxy.set_param("Props", 0, &pod)` auf.
            4. (PipeWire sendet dann über den Listener ein Event über die Volumenänderung, das dann ein `AudioEvent::DeviceVolumeChanged` auslöst).
        - `pub async fn set_stream_volume(...)` (analog).
        - `pub async fn set_default_device(&self, device_pw_id: u32, device_type: AudioDeviceType) -> Result<(), AudioManagementError>`:
            1. Erstellt `Metadata` Proxy für das `core`-Objekt.
            2. Setzt die Eigenschaft `default.audio.sink` oder `default.audio.source` auf die `device_pw_id`.
- **Wichtig:** Die `pipewire-rs` API ist Callback-basiert und integriert sich in eine `MainLoop`. Diese `MainLoop` muss entweder in einem dedizierten Thread laufen oder, falls möglich und komplexer, in die `calloop`-Schleife des Compositors integriert werden (z.B. indem der FD des PipeWire-Loops in `calloop` überwacht wird). Ein eigener Thread für den PipeWire-`MainLoop` ist oft einfacher zu handhaben. Die Kommunikation zwischen diesem Thread und den `async` Methoden des `PipeWireClientService` erfolgt dann über `tokio::sync::mpsc` Kanäle oder indem die `Arc<RwLock<...>>`-geschützten Zustände aktualisiert und `watch` Kanäle für Benachrichtigungen verwendet werden.

#### 4.4. Implementierungsschritte `system::audio_management`

1. **Grundgerüst**: Verzeichnis, `mod.rs`, `Cargo.toml` um `pipewire-rs` und ggf. `libspa` (falls für Pods nötig) erweitern.
2. **`types.rs`**: Alle Audio-bezogenen Typen und Enums (`AudioDeviceType`, `AudioDevice`, `AudioStream`, `AudioEvent`) definieren.
3. **`errors.rs`**: `AudioManagementError` Enum mit `thiserror` und `From<pipewire::Error>` definieren.
4. **`client/mod.rs`**:
    - `PipeWireClientService`-Struktur definieren.
    - `new()`-Konstruktor implementieren: Initialisiert PipeWire-Objekte (`MainLoop`, `Context`, `Core`, `Registry`).
    - `RegistryListener`-Logik implementieren:
        - `global` Callback: Erkennt Nodes (Sinks/Sources) und Streams. Parst deren Properties, um `AudioDevice`- und `AudioStream`-Objekte zu erstellen/aktualisieren. Speichert sie in den `RwLock`-geschützten HashMaps.
        - `global` Callback: Beobachtet `Metadata` für Default-Geräte.
        - Sendet entsprechende `AudioEvent`s über den `event_publisher`.
    - `CoreListener`-Logik für Fehler und Server-Info.
    - Startet den `MainLoop` in einem eigenen Thread.
    - `shutdown()`-Methode implementieren.
    - Öffentliche `async` Getter-Methoden implementieren (lesen aus den `RwLock`-geschützten Maps).
    - Öffentliche `async` Setter-Methoden implementieren (`set_device_volume`, `set_default_device` etc.), die PipeWire-Befehle (oft über Setzen von Properties/Params auf Nodes/Streams via SPA Pods) asynchron ausführen. Dies könnte das Senden von Nachrichten an den MainLoop-Thread über einen Kanal erfordern, falls die `pipewire-rs` Proxies nicht direkt `Send+Sync` sind oder Operationen im MainLoop-Kontext erfordern.
5. **Unit-/Integrationstests**:
    - Sehr schwierig ohne laufenden PipeWire-Server. Fokus auf:
    - Korrekte Erstellung/Parsen der eigenen Typen (`AudioDevice` etc.).
    - Logik der Listener-Callbacks (mocken von `Registry`, `GlobalObject` etc. ist aufwendig). Eventuell Tests, die auf einem Test-PipeWire-Server laufen (falls headless möglich).
    - Testen der Setter-Methoden, ob sie die korrekten PipeWire-Parameter (SPA Pods) generieren würden.

---

Diese detaillierten Pläne für die Module `system::dbus_interfaces` (Fortsetzung) und `system::audio_management` sollten eine sehr solide Grundlage für die Implementierung bieten. Die Komplexität liegt hier oft in der korrekten asynchronen Integration und der genauen Abbildung der externen D-Bus/PipeWire-APIs und -Events auf die internen Strukturen und das Event-System von NovaDE.

---

### Modul 3: `system::dbus_interfaces` (Fortsetzung und Vervollständigung)

#### 3.4. Submodul: `system::dbus_interfaces::logind_client` (Vervollständigung)

Zweck: Client für org.freedesktop.login1 für Sitzungsmanagement.

Bestehende Spezifikation: Skizziert in der vorherigen Antwort.

**Datei:** `src/dbus_interfaces/logind_client/service.rs` (Fortsetzung)

- **`LogindClientService` Implementierung (Methoden-Details):**
    - **`async fn initialize_and_listen(&self) -> Result<(), DBusInterfaceError>`**:
        1. `sessions = self.manager_proxy.list_sessions().await?.into_iter().map(|(id, _uid, _user, _seat, path)| (id, path)).collect::<HashMap<_,_>>();`
        2. Für jede `(id, path)` in `sessions`:
            - `self.add_session_proxy_and_listen(&id, path).await?;`
        3. Starte `tokio::task` für `manager_proxy.receive_session_new()`:
            - Bei `SessionNew { session_id, object_path }`: `self.add_session_proxy_and_listen(&session_id, object_path).await;` `self.event_publisher.send(LogindEvent::SessionNew(...))`.
        4. Starte `tokio::task` für `manager_proxy.receive_session_removed()`:
            - Bei `SessionRemoved { session_id, object_path }`: `self.session_proxies.lock().await.remove(&session_id);` `self.event_publisher.send(LogindEvent::SessionRemoved(...))`.
        5. Starte `tokio::task` für `manager_proxy.receive_prepare_for_shutdown()`:
            - `self.event_publisher.send(LogindEvent::PrepareForShutdown(start_signal_arg)).ok();`
        6. Starte `tokio::task` für `manager_proxy.receive_prepare_for_sleep()`:
            - `self.event_publisher.send(LogindEvent::PrepareForSleep(start_signal_arg)).ok();`
    - **`async fn add_session_proxy_and_listen(&self, session_id: &str, object_path: OwnedObjectPath) -> Result<(), DBusInterfaceError>`**:
        1. `session_proxy = LogindSessionProxy::builder(&self.connection).path(object_path.clone())?.build().await?;`
        2. Starte `tokio::task` für `session_proxy.receive_lock()`: `self.event_publisher.send(LogindEvent::SessionLocked(object_path.clone())).ok();`
        3. Starte `tokio::task` für `session_proxy.receive_unlock()`: `self.event_publisher.send(LogindEvent::SessionUnlocked(object_path.clone())).ok();`
        4. `self.session_proxies.lock().await.insert(session_id.to_string(), session_proxy);`
    - **`get_current_session_details()`**:
        1. `current_pid = std::process::id();`
        2. `session_path = self.manager_proxy.get_session_by_pid(current_pid).await?;`
        3. `session_proxy = LogindSessionProxy::builder(&self.connection).path(session_path.clone())?.build().await?;`
        4. Rufe alle Properties von `session_proxy` ab (z.B. `id()`, `user()`, `name()`, etc.) und fülle `SessionDetails`.
    - **`lock_current_session()`**:
        1. `session_path = self.manager_proxy.get_session_by_pid(std::process::id()).await?;`
        2. `session_proxy = self.session_proxies.lock().await.get(session_path.as_str())` (oder neu erstellen, falls nicht gecacht).
        3. `session_proxy.lock().await?;`
    - Andere Methoden (`can_power_off`, `power_off`, etc.) rufen die entsprechenden `manager_proxy`-Methoden auf.

**Datei:** `src/dbus_interfaces/logind_client/mod.rs`

- `pub mod types;`
- `pub mod proxies;`
- `pub mod service;`
- `pub use service::LogindClientService;`
- `pub use types::{LogindEvent, SessionDetails, LogindPowerOperation};`

#### 3.5. Submodul: `system::dbus_interfaces::network_manager_client` (Vervollständigung)

**Zweck:** Client für `org.freedesktop.NetworkManager`.

**Datei:** `src/dbus_interfaces/network_manager_client/types.rs`

- **Enums:**
    - `NMState`: `Unknown, Asleep, Disconnected, Disconnecting, Connecting, ConnectedLocal, ConnectedSite, ConnectedGlobal`. `#[repr(u32)]`, `Type`.
    - `NMDeviceType`: `Unknown, Ethernet, Wifi, Wimax, Modem, Bluetooth, OlpcMesh, WifiP2p, Bond, Vlan, Adsl, Bridge, Generic, Team, Tun, IpTunnel, Macvlan, Vxlan, Veth, Dummy, Sriov`. `#[repr(u32)]`, `Type`.
    - `NMConnectivityState`: `Unknown, None, Portal, Limited, Full`. `#[repr(u32)]`, `Type`.
    - `NMWifiAccessPointFlags`, `NMWifiAccessPointSecurityFlags`.
- **Structs:**
    - `NetworkDeviceDetails { id: u32, path: DBusObjectPath, interface: String, device_type: NMDeviceType, state: u32 /* NMDeviceState */, ip4_address: Option<String>, ip6_address: Option<String>, hw_address: Option<String>, mtu: u32, managed: bool, firmware_missing: bool, driver: String, ... }`
    - `AccessPointDetails { path: DBusObjectPath, ssid: String, bssid: String, strength: u8, frequency: u32, flags: u32, wpa_flags: u32, rsn_flags: u32, max_bitrate: u32, ... }`
    - `ActiveConnectionDetails { path: DBusObjectPath, uuid: String, connection_type: String, id: String, specific_object_path: DBusObjectPath, state: u32 /* NMActiveConnectionState */, default: bool, default6: bool, vpn: bool, master_path: Option<DBusObjectPath>, ip4_config_path: Option<DBusObjectPath>, ... }`
- **Event:**
    
    Rust
    
    ```
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum NetworkManagerEvent {
        ConnectivityChanged(NMConnectivityState),
        StateChanged(NMState),
        DeviceAdded(NetworkDeviceDetails),
        DeviceRemoved(DBusObjectPath /* device path */),
        DeviceStateChanged { device_path: DBusObjectPath, new_state: u32 /* NMDeviceState */, old_state: u32 },
        ActiveConnectionAdded(ActiveConnectionDetails),
        ActiveConnectionRemoved(DBusObjectPath /* active connection path */),
        AccessPointAdded(AccessPointDetails),
        AccessPointRemoved(DBusObjectPath /* ap path */),
        PrimaryConnectionChanged(Option<ActiveConnectionDetails>),
    }
    ```
    

**Datei:** `src/dbus_interfaces/network_manager_client/proxies.rs`

- **`NetworkManagerProxy` für `org.freedesktop.NetworkManager`**: Properties (`Connectivity`, `State`, `NetworkingEnabled`, `WirelessEnabled`, `WirelessHardwareEnabled`, `WwanEnabled`, `WwanHardwareEnabled`, `ActiveConnections`, `PrimaryConnection`, `Devices`, etc.). Methoden (`GetDevices`, `GetActiveConnections`, `ActivateConnection`, `DeactivateConnection`, `ScanWifiAccessPoints`, etc.). Signale (`CheckPermissions`, `StateChanged`, `PropertiesChanged`, `DeviceAdded`, `DeviceRemoved`, `ActiveConnectionAdded`, `ActiveConnectionRemoved`).
- **`NMDeviceProxy` für `org.freedesktop.NetworkManager.Device`**: Viele Properties (Interface, IpInterface, Udi, State, DeviceType, AvailableConnections, Ip4Config, Dhcp4Config, etc.).
- **`NMWifiDeviceProxy` für `org.freedesktop.NetworkManager.Device.Wireless`**: Properties (`HwAddress`, `PermHwAddress`, `Mode`, `Bitrate`, `ActiveAccessPoint`, etc.). Methoden (`GetAccessPoints`, `RequestScan`, etc.). Signale (`AccessPointAdded`, `AccessPointRemoved`, `PropertiesChanged`).
- **`NMAccessPointProxy` für `org.freedesktop.NetworkManager.AccessPoint`**: Properties (`Ssid`, `Frequency`, `HwAddress`, `Mode`, `MaxBitrate`, `Strength`, `Flags`, `WpaFlags`, `RsnFlags`, etc.).
- **`NMActiveConnectionProxy` für `org.freedesktop.NetworkManager.Connection.Active`**: Properties (`Connection` (path), `SpecificObject` (path), `Id`, `Uuid`, `Type`, `Devices`, `State`, `Default`, `Ip4Config`, etc.). Signale (`StateChanged`, `PropertiesChanged`).
- **`NMSettingsProxy` für `org.freedesktop.NetworkManager.Settings`**: Methoden (`ListConnections`, `AddConnection`, `GetConnectionByUuid`, etc.).
- **`NMSettingsConnectionProxy` für `org.freedesktop.NetworkManager.Settings.Connection`**: Methoden (`Update`, `Delete`, `GetSettings`, `GetSecrets`, etc.).

**Datei:** `src/dbus_interfaces/network_manager_client/service.rs`

- **`NetworkManagerClientService`**:
    - **Felder:** Connection, Hauptproxy, Maps für Geräte-, AP-, aktive Verbindungsproxies. Event-Publisher.
    - **`initialize_and_listen()`**: Initiales Laden von Devices, aktiven Verbindungen. Signale abonnieren (Hauptproxy, Geräte, aktive Verbindungen).
    - **Öffentliche Methoden:** `get_connectivity_state()`, `get_primary_connection_details()`, `list_devices()`, `list_active_connections()`, `list_wifi_access_points(device_path: &DBusObjectPath)`, `activate_connection(connection_path: &DBusObjectPath, device_path: &DBusObjectPath)`, etc.

**Implementierungsschritte `network_manager_client`**:

1. `types.rs`: Alle NM-bezogenen Typen und Events.
2. `proxies.rs`: Alle benötigten NM-Proxies.
3. `service.rs`: `NetworkManagerClientService` implementieren.
4. Tests (schwierig ohne NM, `zbus::MockConnection` verwenden).

---

#### 3.6. Submodul: `system::dbus_interfaces::secrets_service_client` (Vervollständigung)

**Zweck:** Client für `org.freedesktop.Secret.Service`.

**Datei:** `src/dbus_interfaces/secrets_service_client/types.rs`

- **Struct `Secret`**: `session: DBusObjectPath`, `parameters: Vec<u8>`, `value: Vec<u8>`, `content_type: String`.
- **Struct `SecretItemProperties`**: `label: String`, `attributes: HashMap<String, String>`, `created: u64`, `modified: u64`, `locked: bool`.
- **Event:** `SecretServiceEvent::PromptRequired { prompt_path: DBusObjectPath }`, `SecretServiceEvent::PromptCompleted { prompt_path: DBusObjectPath, dismissed: bool }`.

**Datei:** `src/dbus_interfaces/secrets_service_client/proxies.rs`

- **`SecretServiceProxy` (`org.freedesktop.Secret.Service`)**: Methoden (`OpenSession`, `CreateCollection`, `SearchItems`, `Unlock`, `Lock`, `GetSecrets`, `ReadAlias`, `SetAlias`). Properties (`Collections`, `State`). Signal (`CollectionCreated`, `CollectionDeleted`, `CollectionChanged`).
- **`SecretSessionProxy` (`org.freedesktop.Secret.Session`)**: Methode (`Close`). (Lebensdauer beachten).
- **`SecretCollectionProxy` (`org.freedesktop.Secret.Collection`)**: Methoden (`Delete`, `SearchItems`, `CreateItem`). Properties (`Items`, `Label`, `Locked`, `Created`, `Modified`). Signal (`ItemCreated`, `ItemDeleted`, `ItemChanged`).
- **`SecretItemProxy` (`org.freedesktop.Secret.Item`)**: Methoden (`Delete`, `GetSecret`, `SetSecret`). Properties (`Locked`, `Attributes`, `Label`, `Type`, `Created`, `Modified`).
- **`SecretPromptProxy` (`org.freedesktop.Secret.Prompt`)**: Methode (`Prompt`). Signal (`Completed`).

**Datei:** `src/dbus_interfaces/secrets_service_client/service.rs`

- **Struct `SecretsServiceClientService`**:
    - **Felder:** Connection, `service_proxy: SecretServiceProxy`, `default_collection_alias: String` (z.B. "novade_default" oder "login"), `open_sessions: Arc<tokio::sync::Mutex<HashMap<DBusObjectPath, SecretSessionProxy<'static>>>>`. Event-Publisher für `SecretServiceEvent`.
    - **Konstruktor:** `new(...)`.
    - **`initialize_and_listen()`**: Öffnet eine initiale Session für die Default-Collection (oder "login"). Abonniert Signale von `SecretServiceProxy`.
    - **Öffentliche Methoden:**
        - `async fn ensure_collection_exists(&self, alias: &str, label: &str) -> Result<DBusObjectPath, DBusInterfaceError>`
        - `async fn store_secret_in_collection(&self, collection_path_or_alias: &str, item_label: &str, secret_content: &[u8], attributes: HashMap<String, String>, content_type: &str, replace: bool) -> Result<DBusObjectPath /* item_path */, DBusInterfaceError>`
        - `async fn retrieve_secret_from_item(&self, item_path: &DBusObjectPath) -> Result<Vec<u8>, DBusInterfaceError>`
        - `async fn search_items_in_collection(&self, collection_path_or_alias: &str, attributes: HashMap<String, String>) -> Result<Vec<(DBusObjectPath, SecretItemProperties)>, DBusInterfaceError>`
        - `async fn delete_item(&self, item_path: &DBusObjectPath) -> Result<(), DBusInterfaceError>`
        - `async fn unlock_items_or_paths(&self, paths: &[DBusObjectPath]) -> Result<(), DBusInterfaceError>`: Ruft `Unlock` auf `SecretServiceProxy`. Startet Task, um `Prompt.Completed`-Signal zu behandeln.
    - **Handhabung von `Prompt`**: Wenn `Unlock` einen `Prompt`-Pfad zurückgibt, wird ein `SecretPromptProxy` erstellt, `Prompt()` aufgerufen und das `Completed`-Signal abgewartet. Das `SecretServiceEvent` wird gesendet, um UI ggf. zu informieren.

**Implementierungsschritte `secrets_service_client`**:

1. Typen, Proxies, Service-Struktur.
2. Implementierung der Methoden, insbesondere der komplexen Abläufe mit Sessions und Collections.
3. Sorgfältige Handhabung von `Unlock`-Prompts und deren Signalen.
4. Tests mit gemocktem D-Bus.

---

#### 3.7. Submodul: `system::dbus_interfaces::policykit_client` (Vervollständigung)

**Zweck:** Client für `org.freedesktop.PolicyKit1.Authority`.

**Datei:** `src/dbus_interfaces/policykit_client/types.rs`

- **Enum `PolicyKitImplicitAuthorization`**: (aus Polkit-Spezifikation) `Unknown, NotAuthorized, AuthenticationRequired, AdministratorAuthenticationRequired, AuthenticationRequiredRetained, AdministratorAuthenticationRequiredRetained, Authorized`. `#[repr(u32)]`, `Type`.
- **Struct `PolicyKitAuthorizationResultDetails`**: Enthält zusätzliche Daten vom Result.
- **Struct `PolicyKitSubjectSystemBusName`**: `name: String`.
- **Enum `PolicyKitSubjectKind`**: `User { user_id: u32 }`, `SystemBusName { name: String }`, `Binary { path: String, pid: Option<u32> }`. (Vereinfacht, Polkit hat komplexere Subject-Typen).

**Datei:** `src/dbus_interfaces/policykit_client/proxies.rs`

- **`PolicyKitAuthorityProxy` für `org.freedesktop.PolicyKit1.Authority`**:
    
    Rust
    
    ```
    #[proxy(
        interface = "org.freedesktop.PolicyKit1.Authority",
        default_service = "org.freedesktop.PolicyKit1",
        default_path = "/org/freedesktop/PolicyKit1/Authority"
    )]
    trait PolicyKitAuthority {
        // flags: 0x00000001 (ALLOW_USER_INTERACTION)
        // cancellation_id: String (leer für keinen)
        async fn check_authorization(
            &self,
            subject: zbus::zvariant::Value<'_>, // ('sys', {'unix-process': <{'pid': <uint32 ProcessID>, 'start-time': <uint64 StartTime>}>})
            action_id: &str,
            details: std::collections::HashMap<&str, &str>,
            flags: u32,
            cancellation_id: &str,
        ) -> zbus::Result<(bool, bool, Dict<'static, String, Value<'static>>)>; // (is_authorized, is_challenge, details)
        // Weitere Methoden wie EnumerateActions, RegisterAuthenticationAgent etc.
    }
    ```
    
    - **Hinweis:** Der `subject`-Parameter ist komplex (`a{sv}`). `zbus` sollte dies als `Value` oder `Dict` handhaben können.

**Datei:** `src/dbus_interfaces/policykit_client/service.rs`

- **Struct `PolicyKitClientService`**:
    - **Felder:** Connection, `authority_proxy: PolicyKitAuthorityProxy`.
    - **Konstruktor:** `new(...)`.
    - **Öffentliche Methode `async fn check_authorization(...) -> Result<PolicyKitAuthorizationDetails, DBusInterfaceError>`**:
        1. Konstruiert das `subject`-Value (z.B. für den aktuellen Prozess `std::process::id()`).
        2. Ruft `authority_proxy.check_authorization(...)`.
        3. Parst das Ergebnis-Tuple in `PolicyKitAuthorizationDetails`.
        4. UI-Interaktion für Passwortabfragen wird vom systemweiten PolicyKit-Agenten gehandhabt, nicht von diesem Client.

**Implementierungsschritte `policykit_client`**:

1. Typen, Proxy, Service-Struktur.
2. `check_authorization` Implementierung mit korrekter Erstellung des `subject`-Parameters.
3. Tests mit gemocktem D-Bus.

---

#### 3.8. Submodul: `system::dbus_interfaces::notifications_server` (Vervollständigung)

**Zweck:** Implementierung des `org.freedesktop.Notifications` D-Bus-Servers.

**Datei:** `src/dbus_interfaces/notifications_server/service_object.rs`

- **Struct `FreedesktopNotificationsServer`**:
    - **Felder:**
        - `notification_service: Arc<Mutex<dyn NotificationService>>`
        - `id_map: Arc<tokio::sync::Mutex<HashMap<u32, Uuid>>>` (D-Bus ID -> Domain ID)
        - `reverse_id_map: Arc<tokio::sync::Mutex<HashMap<Uuid, u32>>>` (Domain ID -> D-Bus ID)
        - `next_dbus_id: Arc<tokio::sync::atomic::AtomicU32>` (Für eindeutige D-Bus IDs)
        - `signal_ctxt_sender: tokio::sync::mpsc::Sender<DbusSignalTask>` (Um Signale aus einem anderen Kontext senden zu können)
    - **Enum `DbusSignalTask`**: `NotificationClosed { dbus_id: u32, reason: u32 }`, `ActionInvoked { dbus_id: u32, action_key: String }`.
    - **Konstruktor `new(...)`**: Initialisiert Felder. Startet einen `tokio::task`, der Domain-Events (`NotificationDismissedEvent`, `NotificationActionInvokedEvent`) vom `notification_service` empfängt (über dessen `subscribe` Methode), die Domain-UUIDs in D-Bus-`u32`-IDs umwandelt und Tasks über `signal_ctxt_sender` sendet, um die D-Bus-Signale zu emittieren.
    - **D-Bus Methoden Implementierung (`#[dbus_interface(...)]`)**:
        - **`notify(...)`**:
            1. Konvertiert D-Bus Parameter in `NotificationInput`.
            2. Ruft `self.notification_service.lock().await.post_notification(input).await`.
            3. Bei Erfolg: Generiert neue `dbus_id` (aus `next_dbus_id`), speichert Mapping zu Domain-`Uuid`, gibt `dbus_id` zurück.
        - **`close_notification(id: u32)`**:
            1. Findet Domain-`Uuid` für `id` in `id_map`.
            2. Ruft `self.notification_service.lock().await.dismiss_notification(domain_id, DismissReason::DbusRequest).await`. (Grund anpassen)
        - **`get_server_information()`, `get_capabilities()`**: Wie zuvor.
    - **D-Bus Signale**: (`notification_closed`, `action_invoked`) werden von dem separaten Task gesendet, der auf Domain-Events lauscht und den `SignalContext` vom Server-Objekt benötigt. Der `SignalContext` kann geklont und an den Task übergeben werden oder man verwendet den `signal_ctxt_sender` um die Aufgabe ans Hauptobjekt zu delegieren.
- **Funktion zum Starten des D-Bus Dienstes:**
    
    Rust
    
    ```
    // In service_object.rs oder mod.rs
    pub async fn run_notifications_server(
        notification_service: Arc<Mutex<dyn NotificationService>>,
        // broadcast_receiver_for_domain_events: tokio::sync::broadcast::Receiver<NotificationEventEnum>
    ) -> Result<(), DBusInterfaceError> {
        let conn = DBusConnectionManager::session_bus().await?;
        let server_logic = Arc::new(FreedesktopNotificationsServer::new(notification_service));
    
        // Task starten, der Domain-Events in D-Bus-Signale umwandelt
        // Dieser Task benötigt eine Möglichkeit, Signale zu senden.
        // Entweder durch Klonen des SignalContext (falls möglich und sicher)
        // oder durch einen internen MPSC-Kanal zum Server-Objekt.
        // setup_domain_event_to_dbus_signal_handler(server_logic.clone(), broadcast_receiver_for_domain_events);
    
        conn.object_server().at("/org/freedesktop/Notifications", server_logic)?.await?;
        conn.request_name("org.freedesktop.Notifications", zbus:: stazione::RequestNameFlags::ReplaceExisting.into()).await?;
        tracing::info!("org.freedesktop.Notifications D-Bus Service gestartet und Name angefordert.");
        // Die Connection muss am Leben erhalten werden, z.B. indem der Server in einem blockierenden Task läuft
        // oder die Connection selbst in einem Arc gehalten und nie fallengelassen wird.
        // Für einen langlaufenden Dienst ist es üblich, dass diese Funktion nicht zurückkehrt oder
        // die Connection in einer globalen Variable/einem Manager gehalten wird.
        std::future::pending::<()>().await; // Hält den Server am Laufen
        Ok(())
    }
    ```
    

#### 3.10. `system::dbus_interfaces::mod.rs`

- Deklariert alle Submodule (`common`, `upower_client`, `logind_client`, etc.).
- Re-exportiert die öffentlichen Service-Structs/Traits und wichtigen Event-Typen/Fehler.

---

### Modul 4: `system::audio_management` (Vervollständigung)

Zweck: PipeWire-Integration für Audio-Steuerung.

Bestehende Spezifikation: Skizziert in der vorherigen Antwort.

**Datei:** `src/audio_management/types.rs` (Vervollständigung)

- **`AudioDevice`**:
    - Zusätzliche Felder: `ports: Vec<AudioPortInfo>`, `active_profile_index: Option<u32>`, `profiles: Vec<AudioProfileInfo>`, `form_factor: String` (z.B. "headset", "speaker", "microphone"), `bus_path: String`.
- **`AudioPortInfo`**: `id: u32`, `name: String`, `direction: pipewire::spa::Direction`, `available: bool`.
- **`AudioProfileInfo`**: `index: u32`, `name: String`, `description: String`, `available: bool`, `priority: u32`.
- **`AudioStream`**:
    - Zusätzliche Felder: `process_id: Option<u32>`, `process_binary_name: Option<String>`, `is_corked: bool`.
- **`AudioEvent`**:
    - `DefaultDeviceChanged` Payload: `{ device_type: AudioDeviceType, new_default_device: Option<AudioDevice> }` (ganzes Objekt statt nur ID).
    - `DevicePropertiesChanged(AudioDevice)` (Wenn sich andere Properties als nur Volume/Mute ändern).
    - `StreamPropertiesChanged(AudioStream)`.

**Datei:** `src/audio_management/client/pipewire_listener.rs` (Neues Submodul/Datei)

- **Struct `PipeWireRegistryEventHandler`**:
    - **Felder:** `devices: Arc<tokio::sync::RwLock<HashMap<u32, AudioDevice>>>`, `streams: Arc<tokio::sync::RwLock<HashMap<u32, AudioStream>>>`, `default_sink_id: Arc<tokio::sync::RwLock<Option<u32>>>`, `default_source_id: Arc<tokio::sync::RwLock<Option<u32>>>`, `event_publisher: tokio::sync::broadcast::Sender<AudioEvent>`, `core_ref: Weak<pipewire::Core>` (um Proxies zu erstellen).
    - **Methoden (Callbacks für `RegistryListener`):**
        - **`global(global_object)`**:
            1. Prüft Typ (`Node` für Geräte, `Client`/`Stream` für Streams, `Metadata` für Defaults).
            2. Für `Node`:
                - `node_proxy = registry.bind::<pipewire::node::Node>(&global_object)?;`
                - Properties parsen (media.class, device.description, device.api, etc.) um `AudioDeviceType` zu bestimmen.
                - Listener für `node_proxy.receive_info_changed()` und `node_proxy.receive_param_changed()` einrichten.
                    - `info_changed`: Aktualisiert `AudioDevice`-Properties, sendet `DevicePropertiesChanged` oder `DeviceListChanged`.
                    - `param_changed` (für "Props", "Route"): Aktualisiert Volume/Mute in `AudioDevice`, sendet `DeviceVolumeChanged`.
                - Erstellt `AudioDevice`, speichert in `devices`, sendet `DeviceListChanged`.
            3. Für `Stream` (oder `Client`, das Streams hat): Analog für `AudioStream`.
            4. Für `Metadata` (Name "default"):
                - Listener für `metadata_proxy.receive_property_changed()` einrichten.
                - Bei Änderung von "default.audio.sink" oder "default.audio.source": Aktualisiere `default_sink_id`/`default_source_id`, finde das `AudioDevice`-Objekt, setze `is_default`, sende `DefaultDeviceChanged`.
        - **`global_remove(id)`**: Entfernt Objekt aus Maps, sendet `DeviceListChanged`/`StreamListChanged`.
- **Struct `PipeWireCoreEventHandler`**:
    - **Felder:** `event_publisher: tokio::sync::broadcast::Sender<AudioEvent>`.
    - **Methoden (Callbacks für `CoreListener`):**
        - `info(info)`: Prüft `info.change_mask` für `CoreChangeMask::PROPS`, um Server-Verbindungsstatus zu erkennen. Sendet `ServerConnectionStateChanged`.
        - `error(...)`: Loggt Fehler.

**Datei:** `src/audio_management/client/service.rs` (oder `mod.rs`)

- **`PipeWireClientService`**:
    - **Konstruktor `new(...)`**:
        1. Initialisiert PipeWire-Objekte.
        2. Erstellt `PipeWireRegistryEventHandler` und `PipeWireCoreEventHandler`.
        3. `registry.add_listener_local(registry_event_handler_struct)` (oder `Weak` Referenzen verwenden, um Zyklen zu vermeiden, Listener müssen `'static` sein für `add_listener_local`).
        4. `core.add_listener_local(core_event_handler_struct)`.
        5. Startet `MainLoop`-Thread.
    - **Setter-Methoden (`set_device_volume`, `set_default_device` etc.):**
        - Müssen nun asynchron mit dem `MainLoop`-Thread kommunizieren, wenn die `pipewire-rs`-Proxies nicht `Send` sind oder Operationen im Loop-Kontext erfordern.
        - **Ansatz 1 (Kanal zum MainLoop-Thread):**
            - `PipeWireClientService` hält `command_sender: tokio::sync::mpsc::Sender<AudioCommand>`.
            - Im `MainLoop`-Thread wird ein `mpsc::Receiver<AudioCommand>` abgefragt.
            - `AudioCommand` Enum: `SetDeviceVolume { pw_id: u32, volume: u8, mute: bool }, SetDefaultDevice { ... }`.
            - Setter-Methoden senden Befehl über Kanal. `MainLoop` führt Aktion aus, aktualisiert internen Zustand und der Listener sendet dann das `AudioEvent`.
        - **Ansatz 2 (Proxies direkt nutzen, wenn `Send`):** Wenn `pipewire::node::Node` (der Proxy) `Send` ist, können die Setter-Methoden ihn direkt verwenden. Die Aktualisierung der `AudioDevice`-Struktur und das Senden des `AudioEvent` erfolgt dann immer noch über den Listener-Pfad als Reaktion auf das `param_changed`-Signal von PipeWire.
    - **`Youtube_property(core_proxy: &Arc<Core>, key: &str) -> Option<String>`**: Hilfsfunktion zum Lesen von Metadaten-Properties für Default-Geräte.

#### 4.4. Implementierungsschritte `system::audio_management` (Fortsetzung)

1. **`types.rs`**: `AudioDevice`, `AudioStream`, `AudioEvent` vervollständigen. `AudioPortInfo`, `AudioProfileInfo`.
2. **`errors.rs`**: `AudioManagementError` vervollständigen.
3. **`client/pipewire_listener.rs`**: `PipeWireRegistryEventHandler` und `PipeWireCoreEventHandler` implementieren, inklusive der detaillierten Logik in den Callbacks zum Parsen von Properties und Senden von Events.
4. **`client/service.rs`**:
    - `PipeWireClientService`-Struktur mit Feldern für PipeWire-Objekte, Listener-Handles und Zustand (Maps, Default-IDs) definieren.
    - `new()`-Konstruktor: Initialisierung, Listener-Registrierung, `MainLoop`-Thread starten.
    - Getter-Methoden implementieren (lesen aus den `RwLock`-geschützten Maps).
    - Setter-Methoden implementieren:
        - Entscheiden, ob Kommunikation mit MainLoop-Thread via Kanal nötig ist oder ob Proxies direkt verwendet werden können.
        - SPA Pods für Lautstärke etc. korrekt erstellen.
        - `set_default_device` über PipeWire-Metadaten implementieren.
    - `shutdown()`-Methode.
5. **Tests**: Fokus auf korrekte Erstellung von SPA Pods, Parsing von Properties in Listenern (mit Beispiel-Property-Dicts), korrekte Event-Erzeugung. Integrationstests mit laufendem PipeWire-Server sind ideal, aber schwer aufzusetzen.

---

**Nächste Schritte für `system` (Skizze für verbleibende Module):**

- **Modul 5: `system::mcp_client`**:
    - **Zweck:** Implementierung des Model Context Protocol Clients.
    - **Verantwortlichkeiten:** Verbindung zu MCP-Servern, Senden von Anfragen (Ressourcenauflistung, Tool-Aufrufe) basierend auf Anweisungen von `domain::user_centric_services::ai_interaction`, Empfangen und Weiterleiten von Antworten/Benachrichtigungen. API-Schlüssel-Management via `secrets_service_client`.
    - **Technologie:** `mcp_client_rs` Crate.
    - **Struktur:** `connection.rs` (Verwaltung der Verbindung, ggf. Starten eines lokalen MCP-Server-Prozesses), `protocol_handler.rs` (Wrapper um `mcp_client_rs::McpClient`), `resource_provider.rs` (stellt `mcp_client_rs::Resource`s bereit, z.B. für Dateisystemzugriff, Zwischenablage – interagiert mit anderen Systemmodulen).
    - **Fehler:** `McpClientError`.
    - **Events:** `McpResponseEvent`, `McpNotificationEvent`.
- **Modul 6: `system::window_mechanics`**:
    - **Zweck:** Technische Umsetzung des Fenstermanagements (Positionierung, Größe, Stapelreihenfolge, Tiling, Fokus, Dekorationen).
    - **Verantwortlichkeiten:** Nimmt `WorkspaceWindowLayout` von `domain::window_management_policy` entgegen und wendet es auf `ManagedWindow`s im `system::compositor::core::DesktopState::space` an. Sendet `configure`-Events an Wayland-Clients. Handhabt interaktive Operationen (Move/Resize-Grabs). Koordiniert Server-Side Decorations (SSD) mit Client-Side Decorations (CSD) basierend auf `xdg-decoration`-Protokoll und Domänen-Policy.
    - **Interaktion:** Eng mit `system::compositor` (insb. `xdg_shell` und `Space`) und `domain::window_management_policy`.
    - **Fehler:** `WindowMechanicsError`.
- **Modul 7: `system::event_bridge`**:
    - **Zweck:** Eine zentrale Stelle für System-interne Events, die nicht direkt an einen spezifischen D-Bus-Dienst oder ein Wayland-Protokoll gebunden sind.
    - **Verantwortlichkeiten:** Definition von generischen System-Event-Typen. Bereitstellung von `tokio::sync::broadcast` Kanälen für diese Events.
    - **Beispiel-Events:** `InputDeviceHotplugEvent { device_name: String, event_type: HotplugType (Added/Removed) }`, `SystemSuspendRequestEvent { operation: SuspendOperation (Suspend/Hibernate) }`, `ActiveOutputConfigurationChangedEvent`.
    - Dient als Entkopplungsschicht innerhalb des Systems.
- **Modul 8: `system::power_management`**:
    - **Zweck:** Implementierung von DPMS und Interaktion mit `wlr-output-power-management`.
    - **Verantwortlichkeiten:** Hört auf `UserActivityDetectedEvent` und `LogindEvent::SystemIdleHintChanged`. Liest Energieeinstellungen von `domain::global_settings_service`. Steuert den DPMS-Zustand von Outputs über die Compositor-Schnittstelle (z.B. direkte Aufrufe an DRM-Backend oder über `wlr-output-power-management`-Protokoll, falls als Client implementiert).
    - **Interaktion:** `system::compositor::output_management` (für serverseitige `wlr-output-power-management`-Implementierung), `system::dbus_interfaces::logind_client`, `domain::global_settings_service`.
    - **Fehler:** `PowerManagementError`.

Diese detaillierte Aufschlüsselung und die Pläne für die einzelnen Module sollten eine sehr klare Richtung für die Implementierung der Systemschicht geben. Der Fokus liegt auf der korrekten Integration externer Bibliotheken und Protokolle, der robusten Fehlerbehandlung und der Bereitstellung klarer Schnittstellen zur Domänen- und UI-Schicht.

---

### Modul 5: `system::mcp_client`

Zweck: Implementierung des Clients für das Model Context Protocol (MCP), um sicher mit lokalen oder Cloud-basierten KI-Modellen (LLMs) zu interagieren.

Verantwortlichkeiten:

- Aufbau und Verwaltung der Verbindung zu einem MCP-Server (kann ein lokaler Prozess oder ein Remote-Endpunkt sein).
- Senden von Anfragen an den MCP-Server (z.B. `ListResources`, `CallTool`) basierend auf Anweisungen von `domain::user_centric_services::ai_interaction`.
- Empfangen von Antworten und asynchronen Benachrichtigungen vom MCP-Server.
- Sichere Handhabung von API-Schlüsseln (via `system::dbus_interfaces::secrets_service_client`) für Cloud-basierte Modelle.
- Bereitstellung einer abstrahierten Schnittstelle für die Domänenschicht, um KI-Funktionen anzufordern, ohne MCP-Protokolldetails kennen zu müssen. **Design-Rationale:** Kapselung der MCP-spezifischen Kommunikationslogik. Nutzung des `mcp_client_rs` Crates als Basis. Ermöglichung einer flexiblen Anbindung verschiedener KI-Modelle, die das MCP unterstützen. **Technologie:** `mcp_client_rs` Crate, `tokio` für asynchrone Operationen, `serde` für Datenstrukturen.

#### 5.1. Untermodul: `system::mcp_client::types`

**Datei:** `src/mcp_client/types.rs`

- **Struct `McpServerConfig`** (Konfiguration für die Verbindung zu einem MCP-Server):
    
    Rust
    
    ```
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum McpServerType {
        LocalExecutable {
            command: String,
            args: Vec<String>,
            working_directory: Option<String>,
        },
        RemoteHttp {
            endpoint_url: String, // z.B. "http://localhost:8000/mcp"
            // api_key_secret_name: Option<String>, // Wird über AIModelProfile gehandhabt
        },
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct McpServerConfig {
        pub server_id: String, // Eindeutige ID für diese Serverkonfiguration
        pub server_type: McpServerType,
        #[serde(default)]
        pub default_request_timeout_ms: u64, // Standard-Timeout für Anfragen
    }
    
    impl Default for McpServerConfig {
        fn default() -> Self {
            Self {
                server_id: "default_local_mcp".to_string(),
                server_type: McpServerType::LocalExecutable {
                    command: "nova-mcp-server".to_string(), // Beispiel
                    args: vec![],
                    working_directory: None,
                },
                default_request_timeout_ms: 30000, // 30 Sekunden
            }
        }
    }
    ```
    
- **Re-Export und Wrapper für `mcp_client_rs::protocol` Typen (falls nötig):**
    - `pub use mcp_client_rs::protocol::{InitializeParams, InitializeResult, ListResourcesParams, ListResourcesResult, Resource, CallToolParams, CallToolResult, ToolCall, ToolResult, McpMessage, Notification, ErrorResponse, ErrorCode};`
    - Ggf. eigene Wrapper-Structs, wenn Felder hinzugefügt oder angepasst werden müssen.
- **Event-Struktur (für `system::event_bridge` oder direkt an Domäne):**
    
    Rust
    
    ```
    use mcp_client_rs::protocol::{Notification as McpNotification, ErrorResponse as McpErrorResponse, ToolResult as McpToolResult};
    use uuid::Uuid;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum McpClientEvent {
        McpNotificationReceived {
            server_id: String,
            interaction_id: Option<Uuid>, // Interne ID der AIInteractionContext
            notification: McpNotification,
        },
        McpToolCallSuccessful {
            server_id: String,
            interaction_id: Uuid,
            request_id: String, // Aus CallToolParams
            tool_result: McpToolResult,
        },
        McpRequestFailed {
            server_id: String,
            interaction_id: Option<Uuid>,
            request_id: Option<String>,
            error: McpErrorResponse, // Der MCP-Fehler
        },
        McpServerError { // Für Verbindungsfehler etc.
            server_id: String,
            message: String,
        },
        McpServerConnectionStateChanged {
            server_id: String,
            is_connected: bool,
        }
    }
    ```
    

#### 5.2. Untermodul: `system::mcp_client::errors`

**Datei:** `src/mcp_client/errors.rs`

- **Enum `McpSystemClientError`**:
    
    Rust
    
    ```
    use thiserror::Error;
    use mcp_client_rs::Error as McpLibError;
    use crate::dbus_interfaces::common::DBusInterfaceError; // Für Secrets Service Fehler
    
    #[derive(Debug, Error)]
    pub enum McpSystemClientError {
        #[error("MCP Server configuration not found for ID: {0}")]
        ServerConfigNotFound(String),
        #[error("Failed to start local MCP server (command: '{command}'): {source}")]
        LocalServerStartFailed {
            command: String,
            #[source]
            source: std::io::Error,
        },
        #[error("MCP client library error: {0}")]
        McpLibError(#[from] McpLibError),
        #[error("Failed to retrieve API key '{secret_name}' from secrets service: {source}")]
        ApiKeyRetrievalFailed {
            secret_name: String,
            #[source]
            source: DBusInterfaceError,
        },
        #[error("API key '{secret_name}' not found in secrets service.")]
        ApiKeyNotFound(String),
        #[error("MCP request timed out for request ID '{request_id}' to server '{server_id}'.")]
        RequestTimeout { server_id: String, request_id: String },
        #[error("MCP Server '{server_id}' is not connected or connection lost.")]
        ServerNotConnected(String),
        #[error("No active MCP server connection available.")]
        NoActiveConnection,
        #[error("Internal MCP client error: {0}")]
        InternalError(String),
    }
    ```
    

#### 5.3. Submodul: `system::mcp_client::connection_manager`

**Zweck:** Verwaltung von Verbindungen zu MCP-Servern (lokal oder remote), inklusive Starten lokaler Server.

**Datei:** `src/mcp_client/connection_manager.rs`

- **Struct `McpConnection`**:
    - **Felder:**
        - `server_id: String`
        - `client: mcp_client_rs::McpClient` (Der eigentliche Client aus dem Crate)
        - `local_process_handle: Option<tokio::process::Child>` (Für lokale Server)
        - `is_connected: Arc<tokio::sync::watch::Sender<bool>>`
    - **Methoden:**
        - `pub async fn new(config: &McpServerConfig, api_key: Option<String>) -> Result<Self, McpSystemClientError>`:
            1. Wenn `config.server_type` `LocalExecutable`:
                - Starte den Prozess mit `tokio::process::Command`. Speichere `Child` Handle.
                - `client = McpClient::connect_local_stdio(child_process_stdio).await?`
            2. Wenn `config.server_type` `RemoteHttp`:
                - `client = McpClient::connect_http(&config.endpoint_url, api_key).await?`
            3. Setze `is_connected` auf `true`.
        - `pub async fn close(&mut self)`: Schließt die Verbindung, beendet ggf. lokalen Prozess.
        - Getter für `client`.
- **Struct `McpConnectionManager`**:
    - **Felder:**
        - `connections: Arc<tokio::sync::Mutex<HashMap<String /* server_id */, Arc<McpConnection>>>>`
        - `server_configs: Arc<tokio::sync::RwLock<HashMap<String /* server_id */, McpServerConfig>>>` (Geladen aus `core::config` oder `GlobalSettingsService`)
        - `secrets_service: Arc<dyn crate::dbus_interfaces::secrets_service_client::SecretsServiceClient>` // Pfad anpassen
        - `event_publisher: tokio::sync::broadcast::Sender<McpClientEvent>`
    - **Konstruktor `new(...)`**: Nimmt `secrets_service` und `event_publisher`. Lädt `server_configs` initial.
    - **Methoden:**
        - `pub async fn load_server_configs(&self, configs: Vec<McpServerConfig>)`: Aktualisiert `self.server_configs`.
        - `pub async fn get_or_connect(&self, server_id: &str, ai_model_profile: Option<&crate::domain::user_centric_services::ai_interaction::types::AIModelProfile>) -> Result<Arc<McpConnection>, McpSystemClientError>`:
            1. Prüft, ob Verbindung in `connections` existiert und verbunden ist. Wenn ja, zurückgeben.
            2. Sucht `McpServerConfig` in `server_configs`. Wenn nicht -> `ServerConfigNotFound`.
            3. Wenn `config.server_type` `RemoteHttp` und `ai_model_profile.api_key_secret_name` gesetzt ist:
                - Rufe `self.secrets_service.retrieve_secret_by_label_or_item_path(...)` auf, um API-Key zu holen. Fehler bei Fehlschlag.
            4. Erstelle neue `McpConnection::new(&config, api_key)`.
            5. Speichere in `connections`.
            6. Starte einen Task, der auf Nachrichten/Notifications vom `mcp_connection.client.receive_message()` lauscht und `McpClientEvent`s publiziert.
            7. Sendet `McpServerConnectionStateChanged`.
        - `pub async fn disconnect(&self, server_id: &str) -> Result<(), McpSystemClientError>`: Schließt Verbindung, entfernt aus `connections`. Sendet `McpServerConnectionStateChanged`.
        - `pub async fn get_active_connection_for_model(&self, model_profile: &crate::domain::user_centric_services::ai_interaction::types::AIModelProfile) -> Result<Arc<McpConnection>, McpSystemClientError>`:
            - Bestimmt `server_id` basierend auf `model_profile` (z.B. wenn Profil eine `mcp_server_id` enthält oder ein Default verwendet wird).
            - Ruft `get_or_connect(server_id, Some(model_profile))`.

#### 5.4. Submodul: `system::mcp_client::service`

**Zweck:** Implementierung des `SystemMcpService` Traits, der die Abstraktion zur Domänenschicht darstellt.

**Datei:** `src/mcp_client/service.rs`

- **Trait `SystemMcpService`** (definiert, was die Domänenschicht vom MCP-Client erwartet):
    
    Rust
    
    ```
    use async_trait::async_trait;
    use super::types::*; // McpClientEvent, McpSystemClientError etc.
    use crate::domain::user_centric_services::ai_interaction::types::AIModelProfile;
    use uuid::Uuid;
    
    #[async_trait]
    pub trait SystemMcpService: Send + Sync {
        /// Initialisiert den MCP-Client mit Serverkonfigurationen.
        async fn configure_servers(&self, server_configs: Vec<McpServerConfig>) -> Result<(), McpSystemClientError>;
    
        /// Sendet eine `Initialize` Nachricht an einen spezifischen MCP-Server.
        async fn initialize_server(
            &self,
            server_id: &str,
            params: InitializeParams,
            model_profile: Option<&AIModelProfile>, // Für API-Key etc.
        ) -> Result<InitializeResult, McpSystemClientError>;
    
        /// Listet Ressourcen vom MCP-Server auf.
        async fn list_resources(
            &self,
            server_id: &str,
            params: ListResourcesParams,
            model_profile: Option<&AIModelProfile>,
            interaction_id: Option<Uuid>, // Für Event-Korrelation
        ) -> Result<ListResourcesResult, McpSystemClientError>;
    
        /// Ruft ein Tool auf dem MCP-Server auf.
        async fn call_tool(
            &self,
            server_id: &str,
            params: CallToolParams,
            model_profile: Option<&AIModelProfile>,
            interaction_id: Uuid, // Für Event-Korrelation und Timeout-Management
        ) -> Result<CallToolResult, McpSystemClientError>; // McpClient::call_tool gibt McpMessage zurück
    
        /// Abonniert MCP-Client-Events.
        fn subscribe_to_mcp_events(&self) -> tokio::sync::broadcast::Receiver<McpClientEvent>;
    }
    ```
    
- **Struct `DefaultSystemMcpService`**:
    - **Felder:** `connection_manager: Arc<McpConnectionManager>`.
    - **Konstruktor `new(connection_manager: Arc<McpConnectionManager>) -> Self`**.
    - **Implementierung von `SystemMcpService`**:
        - `configure_servers`: Ruft `connection_manager.load_server_configs()`.
        - `initialize_server`, `list_resources`, `call_tool`:
            1. `mcp_conn = self.connection_manager.get_or_connect(server_id, model_profile).await?;`
            2. `let client = &mcp_conn.client;`
            3. Erstelle `McpMessage` für die Anfrage.
            4. `response_message = client.send_request(request_message).await.map_err(McpSystemClientError::from)?;` (Timeout hier oder im `McpClient` Crate)
            5. Parse `response_message` in den erwarteten Ergebnistyp (z.B. `InitializeResult`). Bei Fehler `McpSystemClientError::McpLibError` oder spezifischer.
            6. Für `call_tool`, wenn erfolgreich, `McpToolCallSuccessful` Event senden (über `connection_manager.event_publisher`).
            7. Bei MCP-Fehlerantwort, `McpRequestFailed` Event senden.
        - `subscribe_to_mcp_events`: Gibt `connection_manager.event_publisher.subscribe()` zurück.

#### 5.5. `system::mcp_client::mod.rs`

- Deklariert Submodule.
- Re-exportiert `SystemMcpService`-Trait, `DefaultSystemMcpService` (als konkrete Implementierung), `McpClientEvent`, `McpSystemClientError`, `McpServerConfig`.

#### 5.6. Implementierungsschritte `system::mcp_client`

1. **Grundgerüst**: Verzeichnisse, `Cargo.toml` für `mcp_client_rs`.
2. **`types.rs`**: `McpServerConfig`, `McpClientEvent`, ggf. Wrapper definieren.
3. **`errors.rs`**: `McpSystemClientError` definieren.
4. **`connection_manager.rs`**: `McpConnection`, `McpConnectionManager` implementieren. Logik zum Starten lokaler Server, API-Key-Abruf via `SecretsServiceClientService`. Task für `client.receive_message()` und Event-Publishing.
5. **`service.rs`**: `SystemMcpService`-Trait und `DefaultSystemMcpService`-Implementierung.
6. **Unit-Tests**:
    - Testen von `McpServerConfig`-Serialisierung.
    - Testen der `McpConnectionManager`-Logik (Mocking von `SecretsServiceClientService` und `mcp_client_rs::McpClient` falls möglich, oder Integrationstests gegen einen Dummy-MCP-Server).
    - Testen der `DefaultSystemMcpService`-Methoden (Mocking von `McpConnectionManager`).

---

### Modul 6: `system::window_mechanics`

Zweck: Technische Umsetzung des Fenstermanagements basierend auf den Richtlinien der Domänenschicht.

Verantwortlichkeiten:

- Empfangen von `WorkspaceWindowLayout` von `domain::window_management_policy`.
- Anwenden dieser Geometrien auf die tatsächlichen Fenster (`ManagedWindow`s im Compositor).
- Senden von `configure`-Events an Wayland-Clients, um sie über neue Größen/Zustände zu informieren.
- Handhabung interaktiver Operationen (Move/Resize-Grabs), Anwendung von Snapping.
- Koordination von Server-Side Decorations (SSD) und Client-Side Decorations (CSD) in Absprache mit `system::compositor::decoration` und der Domänen-Policy.
- Technische Umsetzung des Fokuswechsels basierend auf Domänenentscheidungen. **Design-Rationale:** Trennt die "Mechanik" (Wie wird ein Fenster bewegt/gegrößert?) von der "Policy" (Wohin soll es bewegt/gegrößert werden?). Enge Kopplung mit dem Compositor (`DesktopState::space`, `ManagedWindow`).

#### 6.1. Submodul: `system::window_mechanics::types`

**Datei:** `src/window_mechanics/types.rs`

- **Struct `InteractiveOpState`** (für laufende Move/Resize Grabs):
    
    Rust
    
    ```
    use smithay::utils::{Logical, Point, Rectangle, Serial};
    use crate::compositor::core::state::ManagedWindow; // Pfad anpassen
    use std::sync::Arc;
    
    #[derive(Debug, Clone)]
    pub enum InteractiveOpType { Move, ResizeEdge(xdg_toplevel::ResizeEdge), ResizeCorner(/* ... */) }
    
    #[derive(Debug, Clone)]
    pub struct InteractiveOpState {
        pub window_arc: Arc<ManagedWindow>, // Das Fenster, das bewegt/vergrößert wird
        pub op_type: InteractiveOpType,
        pub start_pointer_pos_global: Point<f64, Logical>,
        pub initial_window_geometry: Rectangle<i32, Logical>,
        pub last_configure_serial: Option<Serial>, // Um Configure-Storms zu vermeiden
        // Ggf. weitere Felder für Snapping-Feedback etc.
    }
    ```
    
- **Event (System-intern, via `system::event_bridge`):**
    
    Rust
    
    ```
    #[derive(Debug, Clone)]
    pub enum WindowMechanicsEvent {
        WindowConfigured { window_domain_id: DomainWindowIdentifier, new_geometry: RectInt, new_state_flags: u32 },
        InteractiveOpStarted(DomainWindowIdentifier, InteractiveOpType),
        InteractiveOpEnded(DomainWindowIdentifier, InteractiveOpType),
    }
    ```
    

#### 6.2. Submodul: `system::window_mechanics::errors`

**Datei:** `src/window_mechanics/errors.rs`

- **Enum `WindowMechanicsError`**:
    
    Rust
    
    ```
    use thiserror::Error;
    use crate::compositor::xdg_shell::errors::XdgShellError; // Pfad anpassen
    
    #[derive(Debug, Error)]
    pub enum WindowMechanicsError {
        #[error("Window not found for mechanics operation: {0:?}")]
        WindowNotFound(crate::domain::workspaces::core::types::WindowIdentifier),
        #[error("Failed to apply layout from domain policy: {0}")]
        LayoutApplicationFailed(String),
        #[error("Error during interactive operation (move/resize): {0}")]
        InteractiveOpFailed(String),
        #[error("XDG Shell operation failed during window mechanics: {0}")]
        XdgShellError(#[from] XdgShellError), // Wenn Configure-Sends etc. fehlschlagen
        #[error("Failed to acquire necessary lock for window operation.")]
        LockFailed,
        #[error("Internal window mechanics error: {0}")]
        InternalError(String),
    }
    ```
    

#### 6.3. Submodul: `system::window_mechanics::layout_applier`

**Zweck:** Anwenden eines von der Domäne berechneten Layouts.

**Datei:** `src/window_mechanics/layout_applier.rs`

- **Funktion `pub async fn apply_workspace_layout(desktop_state: &Arc<Mutex<DesktopState>>, workspace_id: WorkspaceId, layout: &WorkspaceWindowLayout) -> Result<(), WindowMechanicsError>`**:
    1. Sperre `desktop_state`.
    2. Für jede `(domain_window_id, target_geometry)` in `layout.window_geometries`:
        - Finde das `Arc<ManagedWindow>` in `desktop_state.windows` (oder `desktop_state.space`). Wenn nicht: `WindowMechanicsError::WindowNotFound`.
        - `let mut window_guard = managed_window_arc.lock_blocking();` (Oder `async` Lock, wenn `ManagedWindow` selbst einen `async` Mutex hat. Hier Annahme: `ManagedWindow` ist `desktop::Window`, das intern ggf. synchron ist oder dessen `send_configure` etc. synchron sind).
        - **Geometrie setzen:** `window_guard.current_geometry = target_geometry;`
        - **Größe/Status an Client senden:**
            - `if let WindowSurface::Toplevel(toplevel_surface) = &window_guard.xdg_surface { ... }`
            - `toplevel_surface.with_pending_state(|state| { state.size = Some(target_geometry.size); /* state.maximized, .fullscreen etc. basierend auf Layout-Anforderungen setzen */ });`
            - `toplevel_surface.send_configure();` (Dies sendet `xdg_surface.configure` und `xdg_toplevel.configure`).
        - Ggf. `WindowMechanicsEvent::WindowConfigured` senden.
    3. Für Fenster, die im alten Layout waren, aber nicht im neuen (d.h. geschlossen oder auf anderen Workspace verschoben): `desktop_state.space.unmap_window(...)`.
    4. Für neue Fenster im Layout (noch nicht im Space): `desktop_state.space.map_window(...)`.
    5. `desktop_state.space.damage_all_outputs();` (Oder spezifischerer Schaden).

#### 6.4. Submodul: `system::window_mechanics::interactive_ops`

**Zweck:** Handhabung von interaktiven Fenster-Moves und -Resizes.

**Datei:** `src/window_mechanics/interactive_ops.rs`

- **Funktionen zum Starten von Grabs (aufgerufen von `XdgShellHandler` in `system::compositor`):**
    - `pub fn start_interactive_move(desktop_state: &Arc<Mutex<DesktopState>>, seat: &Seat<DesktopState>, window_arc: Arc<ManagedWindow>, serial: Serial, start_pointer_pos: Point<f64, Logical>)`
    - `pub fn start_interactive_resize(desktop_state: &Arc<Mutex<DesktopState>>, seat: &Seat<DesktopState>, window_arc: Arc<ManagedWindow>, serial: Serial, edge: xdg_toplevel::ResizeEdge, start_pointer_pos: Point<f64, Logical>)`
    - **Logik:**
        1. Erstelle `InteractiveOpState`.
        2. `seat.start_pointer_grab(...)` mit einem spezifischen `PointerGrabStartData` und einem `PointerGrab` Handler.
        3. Der `PointerGrab` Handler:
            - **`motion(...)`**:
                - Berechne neue Geometrie basierend auf `delta` und `InteractiveOpState`.
                - Rufe `domain::window_management_policy_service.calculate_snap_target(...)` auf, um Snapping anzuwenden.
                - Aktualisiere `window_arc.lock_blocking().current_geometry` (vorläufig, ohne Configure).
                - Optional: Zeige visuelles Feedback (z.B. Umriss des Fensters an neuer Position – dies ist Renderer-Aufgabe).
            - **`button(...)`**: Wenn Maustaste losgelassen:
                - Finalisiere Geometrie.
                - Sende `configure` an Client (via `window_arc.xdg_surface.toplevel().send_configure()`).
                - Beende den Grab (`pointer_handle.unset_grab()`).
                - Sende `WindowMechanicsEvent::InteractiveOpEnded`.
            - **`axis(...)`**: Ignorieren während Grab.
            - **`cancel(...)`**: Grab abbrechen, Fenster auf `initial_window_geometry` zurücksetzen.

#### 6.5. Submodul: `system::window_mechanics::focus_manager`

**Zweck:** Technische Umsetzung des Fokuswechsels.

**Datei:** `src/window_mechanics/focus_manager.rs`

- **Funktion `pub async fn set_application_focus(desktop_state: &Arc<Mutex<DesktopState>>, seat_name: &str, window_domain_id_to_focus: Option<&DomainWindowIdentifier>, serial: Serial)`**:
    1. Sperre `desktop_state`.
    2. Finde das `Arc<ManagedWindow>` für `window_domain_id_to_focus` (oder `None`).
    3. Rufe `system::input::keyboard::focus::set_keyboard_focus(desktop_state_guard, seat_name, target_wl_surface_option, serial)`.
    4. Aktualisiere `desktop_state_guard.active_input_surface`.
    5. Wenn `target_wl_surface_option` ein Toplevel ist: `target_toplevel.set_activated(true); target_toplevel.send_configure();`
    6. Für den vorherigen Fokus: `old_toplevel.set_activated(false); old_toplevel.send_configure();`
    7. Ggf. Fenster im `Space` anheben (`desktop_state_guard.space.raise_window(...)`).

#### 6.6. Implementierungsschritte `system::window_mechanics`

1. `types.rs`, `errors.rs` definieren.
2. `layout_applier.rs`: `apply_workspace_layout` implementieren.
3. `interactive_ops.rs`: Logik für Start und Handling von Pointer-Grabs für Move/Resize.
4. `focus_manager.rs`: `set_application_focus` implementieren.
5. Tests: Mocking von `DesktopState` (schwierig), `DomainWindowManagementPolicyService`. Testen der Geometrieanwendung. Testen der Grab-Logik (Zustandsübergänge).

---

### Modul 7: `system::power_management`

Zweck: Implementierung von DPMS und Interaktion mit Power-Management-Protokollen/Diensten.

Verantwortlichkeiten: Überwachen der Benutzeraktivität und System-Idle-Hinweise, Anwenden von Energieeinstellungen (Bildschirm-Timeout, Suspend-Verhalten) von der Domänenschicht, Steuerung des DPMS-Zustands von Bildschirmen.

Design-Rationale: Zentralisierung der Energieverwaltungslogik, die sowohl auf Benutzereingaben als auch auf Systemzustände reagiert.

#### 7.1. Untermodul: `system::power_management::types`

**Datei:** `src/power_management/types.rs`

- **Enum `DpmsState`**: `On, Standby, Suspend, Off`.
- **Event (System-intern, via `system::event_bridge`):**
    
    Rust
    
    ```
    #[derive(Debug, Clone)]
    pub enum PowerManagementEvent {
        OutputDpmsStateChanged { output_name: String, new_state: DpmsState },
        SystemSuspending(crate::dbus_interfaces::logind_client::types::LogindPowerOperation), // Pfad anpassen
        SystemResumed,
    }
    ```
    
- **Struct `IdleTimerState`**: `last_activity_ts: DateTime<Utc>`, `current_timeout_secs: u32`, `timer_handle: Option<calloop::TimerHandle>`.

#### 7.2. Untermodul: `system::power_management::errors`

**Datei:** `src/power_management/errors.rs`

- **Enum `PowerManagementError`**:
    
    Rust
    
    ```
    #[derive(Debug, Error)]
    pub enum PowerManagementError {
        #[error("Failed to set DPMS state for output '{output_name}': {reason}")]
        SetDpmsFailed { output_name: String, reason: String },
        #[error("Logind operation failed: {0}")]
        LogindError(#[from] crate::dbus_interfaces::common::DBusInterfaceError), // Pfad anpassen
        #[error("Failed to interact with compositor output management: {0}")]
        CompositorOutputError(String),
        #[error("Internal power management error: {0}")]
        InternalError(String),
    }
    ```
    

#### 7.3. Submodul: `system::power_management::service`

**Zweck:** Hauptlogik des Power-Management-Dienstes.

**Datei:** `src/power_management/service.rs`

- **Struct `PowerManagementService`**:
    - **Felder:**
        - `desktop_state_weak: Weak<Mutex<DesktopState>>` (oder direkter Zugriff, falls in `DesktopState` integriert)
        - `settings_service: Arc<dyn GlobalSettingsService>`
        - `logind_service: Arc<dyn LogindClientService>` // Annahme, dass LogindClientService ein Trait ist
        - `event_publisher: tokio::sync::broadcast::Sender<PowerManagementEvent>`
        - `user_activity_receiver: tokio::sync::broadcast::Receiver<UserActivityDetectedEvent>` (aus `common_events`)
        - `logind_event_receiver: tokio::sync::broadcast::Receiver<LogindEvent>`
        - `output_idle_timers: Arc<tokio::sync::Mutex<HashMap<String /* output_name */, IdleTimerState>>>`
        - `system_idle_timer: Arc<tokio::sync::Mutex<Option<IdleTimerState>>>` // Für automatischen Suspend
    - **Konstruktor `new(...)`**: Nimmt Abhängigkeiten, abonniert `UserActivityDetectedEvent` und `LogindEvent`.
    - **Methode `pub async fn run(&self)`**: Hauptschleife des Dienstes (läuft als `tokio::task`).
        1. Lädt initiale Energieeinstellungen vom `settings_service`.
        2. Startet Listener für `SettingChangedEvent` (um Energieeinstellungen neu zu laden).
        3. Verarbeitet eingehende `UserActivityDetectedEvent`: Setzt alle Idle-Timer zurück.
        4. Verarbeitet eingehende `LogindEvent::PrepareForSleep/Shutdown`: Führt Aktionen aus (z.B. DPMS Off).
        5. Verarbeitet `LogindEvent::SystemIdleHintChanged`.
        6. Periodisch (oder bei Timer-Ablauf):
            - Prüft `output_idle_timers`. Wenn Timeout erreicht: Setze DPMS-Status des Outputs (via Compositor/DRM-Backend). Sendet `OutputDpmsStateChanged`.
            - Prüft `system_idle_timer`. Wenn Timeout erreicht: Rufe `self.logind_service.suspend(false)` oder `hibernate(false)` auf, basierend auf Policy.
    - **Private Methoden:**
        - `async fn reset_idle_timers(&self, current_settings: &PowerManagementPolicySettings)`
        - `async fn apply_dpms_state(&self, output_name: &str, dpms_state: DpmsState)`: Interagiert mit `system::compositor::output_management` (oder direkt DRM-Backend), um DPMS zu setzen.
        - `async fn on_screen_blank_timeout(&self, output_name: &str, current_settings: &PowerManagementPolicySettings)`
        - `async fn on_system_suspend_timeout(&self, current_settings: &PowerManagementPolicySettings)`

#### 7.4. Implementierungsschritte `system::power_management`

1. `types.rs`, `errors.rs` definieren.
2. `service.rs`: `PowerManagementService` implementieren.
    - Event-Loops für `UserActivityDetectedEvent` und `LogindEvent`.
    - Logik für Idle-Timer-Management mit `calloop::Timer` (muss mit `tokio` synchronisiert werden, wenn Service `async` ist, z.B. Timer in `calloop`-Schleife, der Nachricht an `tokio`-Task sendet).
    - Interaktion mit `GlobalSettingsService` für Policies.
    - Interaktion mit `LogindClientService` für Suspend/Hibernate.
    - Interaktion mit Compositor (Output-Management) für DPMS.
3. Tests: Mocking von Abhängigkeiten, Testen der Timer-Logik und Zustandsübergänge.

---

### Modul 8: `system::event_bridge`

Zweck: Eine zentrale Stelle für System-interne Events, die nicht direkt an einen spezifischen D-Bus-Dienst oder ein Wayland-Protokoll gebunden sind. Dient der Entkopplung innerhalb der Systemschicht und als definierte Quelle für bestimmte Domänen-Events.

Verantwortlichkeiten: Definition von generischen System-Event-Typen. Bereitstellung von tokio::sync::broadcast Kanälen für diese Events.

Design-Rationale: Verhindert direkte Abhängigkeiten zwischen allen Systemmodulen. Ermöglicht es Modulen, relevante Ereignisse zu publizieren, ohne ihre Konsumenten explizit kennen zu müssen.

**Datei:** `src/event_bridge/mod.rs` (kann `types.rs` und `channels.rs` enthalten)

- **Struct `SystemEventBridge`**:
    - **Felder (Beispiele für `broadcast::Sender`):**
        - `upower_event_tx: tokio::sync::broadcast::Sender<UPowerEvent>`
        - `logind_event_tx: tokio::sync::broadcast::Sender<LogindEvent>`
        - `network_manager_event_tx: tokio::sync::broadcast::Sender<NetworkManagerEvent>`
        - `audio_event_tx: tokio::sync::broadcast::Sender<AudioEvent>`
        - `mcp_client_event_tx: tokio::sync::broadcast::Sender<McpClientEvent>`
        - `window_mechanics_event_tx: tokio::sync::broadcast::Sender<WindowMechanicsEvent>`
        - `power_management_event_tx: tokio::sync::broadcast::Sender<PowerManagementEvent>`
        - `input_device_hotplug_event_tx: tokio::sync::broadcast::Sender<InputDeviceHotplugEvent>`
        - **Domänen-Events, die von der Systemschicht ausgelöst werden:**
        - `user_activity_event_tx: tokio::sync::broadcast::Sender<crate::domain::common_events::UserActivityDetectedEvent>`
        - `system_shutdown_event_tx: tokio::sync::broadcast::Sender<crate::domain::common_events::SystemShutdownInitiatedEvent>`
    - **Konstruktor `new(capacity_per_channel: usize) -> Self`**: Initialisiert alle Sender.
    - **Methoden zum Abrufen von `Receiver`n**:
        - `pub fn subscribe_upower_events(&self) -> tokio::sync::broadcast::Receiver<UPowerEvent>` (analog für alle anderen).
    - **Methoden zum Senden von Events (intern von anderen Systemmodulen genutzt):**
        - `pub(crate) fn publish_upower_event(&self, event: UPowerEvent)` (analog).
- **Event-Typen (Beispiele, falls noch nicht in spezifischen Modulen definiert):**
    - `InputDeviceHotplugEvent { device_name: String, device_type: String /* z.B. "keyboard", "pointer" */, event_type: HotplugType /* Added, Removed */}`
    - Die meisten spezifischen Events (`UPowerEvent`, `LogindEvent` etc.) werden in ihren jeweiligen Modulen (`system::dbus_interfaces::upower_client::types`) definiert und hier nur die Sender verwaltet.

**Implementierungsschritte `system::event_bridge`**:

1. `SystemEventBridge`-Struktur definieren.
2. Konstruktor und `subscribe_`/`publish_`-Methoden implementieren.
3. Sicherstellen, dass alle Systemmodule, die Events publizieren oder konsumieren, eine Referenz (`Arc`) zum `SystemEventBridge` erhalten (z.B. bei der Initialisierung von `DesktopState` oder der Systemschicht).

---

Diese detaillierten Pläne für die Module 3 (Vervollständigung) bis 8 der Systemschicht bilden eine solide Grundlage für die Implementierung. Die Komplexität liegt weiterhin in der korrekten asynchronen Integration, der Interaktion mit externen Bibliotheken/Protokollen und der robusten Fehlerbehandlung.

---

### Modul 5: `system::mcp_client`

Zweck: Implementierung des Clients für das Model Context Protocol (MCP), um sicher mit lokalen oder Cloud-basierten KI-Modellen (LLMs) zu interagieren.

Verantwortlichkeiten:

- Aufbau und Verwaltung der Verbindung zu einem MCP-Server.
- Senden von Anfragen an den MCP-Server (z.B. `Initialize`, `ListResources`, `CallTool`) basierend auf Anweisungen von `domain::user_centric_services::ai_interaction`.
- Empfangen von Antworten und asynchronen Benachrichtigungen (`Notification`) vom MCP-Server.
- Sichere Handhabung von API-Schlüsseln (via `system::dbus_interfaces::secrets_service_client`) für Cloud-basierte Modelle.
- Bereitstellung einer abstrahierten Schnittstelle (`SystemMcpService`-Trait) für die Domänenschicht. **Design-Rationale:** Kapselung der MCP-spezifischen Kommunikationslogik. Nutzung des `mcp_client_rs` Crates als Basis. Ermöglichung einer flexiblen Anbindung verschiedener KI-Modelle. **Technologie:** `mcp_client_rs` Crate, `tokio` für asynchrone Operationen, `serde` für Datenstrukturen.

**Abhängigkeiten in `novade-system/Cargo.toml` (zusätzlich):**

Ini, TOML

```
mcp_client_rs = "0.2.0" # Aktuelle Version des mcp_client_rs Crates prüfen
# ggf. http_types oder reqwest, falls RemoteHttp direkt implementiert wird und mcp_client_rs dies nicht vollständig abstrahiert
```

#### 5.1. Untermodul: `system::mcp_client::types`

**Datei:** `src/mcp_client/types.rs`

- **Struct `McpServerConfig`**:
    
    - **Definition:** Wie in der vorherigen Antwort (Teil 4 der Systemschicht-Spezifikation).
        
        Rust
        
        ```
        use serde::{Serialize, Deserialize};
        use std::path::PathBuf; // Für working_directory
        
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub enum McpServerType {
            LocalExecutable {
                command: String,
                args: Vec<String>,
                #[serde(default, skip_serializing_if = "Option::is_none")]
                working_directory: Option<PathBuf>, // PathBuf verwenden
            },
            RemoteHttp {
                endpoint_url: String,
            },
        }
        
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
        pub struct McpServerConfig {
            pub server_id: String,
            pub server_type: McpServerType,
            #[serde(default = "default_request_timeout_ms_config")]
            pub default_request_timeout_ms: u64,
        }
        
        fn default_request_timeout_ms_config() -> u64 { 30000 } // 30 Sekunden
        
        impl Default for McpServerConfig { /* ... */ } // Sinnvoller Default, z.B. für einen häufig genutzten lokalen Server
        ```
        
- **Re-Export von `mcp_client_rs::protocol` Typen:**
    
    Rust
    
    ```
    pub use mcp_client_rs::protocol::{
        InitializeParams, InitializeResult, ListResourcesParams, ListResourcesResult,
        Resource, ToolDefinition, CallToolParams, CallToolResult, ToolCall, ToolResult,
        McpMessage, Notification as McpProtocolNotification, ErrorResponse, ErrorCode,
        // Weitere benötigte Typen aus dem Protokoll
    };
    ```
    
- **Event-Struktur (für `system::event_bridge` oder direkt an Domäne):**
    
    Rust
    
    ```
    use uuid::Uuid;
    // McpProtocolNotification, McpErrorResponse, McpToolResult sind bereits oben re-exportiert
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum McpClientSystemEvent { // Umbenannt von McpClientEvent zur Unterscheidung von Domain-Events
        McpNotificationReceived {
            server_id: String,
            interaction_id: Option<Uuid>,
            notification: McpProtocolNotification,
        },
        McpToolCallSuccessful {
            server_id: String,
            interaction_id: Uuid,
            request_id: String,
            tool_result: McpToolResult,
        },
        McpRequestFailed { // Wenn die Anfrage den Server erreicht hat, aber dieser einen Fehler zurückgibt
            server_id: String,
            interaction_id: Option<Uuid>,
            request_id: Option<String>,
            error: McpErrorResponse,
        },
        McpCommunicationError { // Für Verbindungsfehler, Timeouts etc. vor/während der Anfrage
            server_id: String,
            interaction_id: Option<Uuid>,
            request_id: Option<String>,
            message: String, // Detailliertere Fehlermeldung des Clients
        },
        McpServerConnectionStateChanged {
            server_id: String,
            is_connected: bool,
            error_message: Option<String>, // Grund für Disconnect
        }
    }
    ```
    

#### 5.2. Untermodul: `system::mcp_client::errors`

**Datei:** `src/mcp_client/errors.rs`

- **Enum `McpSystemClientError`**:
    
    - **Definition:** Wie in der vorherigen Antwort, aber `McpLibError` wird differenzierter behandelt. <!-- end list -->
    
    Rust
    
    ```
    use thiserror::Error;
    use mcp_client_rs::Error as McpLibInternalError; // Interner Fehler des mcp_client_rs Crates
    use crate::dbus_interfaces::common::DBusInterfaceError; // Für Secrets Service Fehler
    use super::types::ErrorResponse as McpErrorResponse; // MCP Protokoll-Fehler
    
    #[derive(Debug, Error)]
    pub enum McpSystemClientError {
        #[error("MCP Server configuration not found for ID: {0}")]
        ServerConfigNotFound(String),
        #[error("Failed to start local MCP server (command: '{command}'): {source}")]
        LocalServerStartFailed { command: String, #[source] source: std::io::Error },
        #[error("MCP client library internal error: {0}")]
        McpLibInternalError(#[from] McpLibInternalError),
        #[error("MCP server returned an error: {error_code:?} - {message}")]
        McpServerErrorReply { error_code: mcp_client_rs::protocol::ErrorCode, message: String, diagnostic_info: Option<String> }, // Von McpErrorResponse
        #[error("Failed to retrieve API key '{secret_name}' from secrets service: {source}")]
        ApiKeyRetrievalFailed { secret_name: String, #[source] source: DBusInterfaceError },
        #[error("API key '{secret_name}' not found in secrets service.")]
        ApiKeyNotFound(String),
        #[error("MCP request timed out for request ID '{request_id}' to server '{server_id}'.")]
        RequestTimeout { server_id: String, request_id: String },
        #[error("MCP Server '{server_id}' is not connected or connection lost.")]
        ServerNotConnected(String),
        #[error("No active MCP server connection available for the request.")]
        NoActiveConnection,
        #[error("Failed to serialize MCP request: {0}")]
        SerializationError(#[from] serde_json::Error), // Falls wir manuell serialisieren
        #[error("Failed to deserialize MCP response: {0}")]
        DeserializationError(serde_json::Error), // Falls wir manuell deserialisieren
        #[error("Unsupported MCP server type: {0:?}")]
        UnsupportedServerType(super::types::McpServerType),
        #[error("Internal MCP client error: {0}")]
        InternalError(String),
    }
    
    // Konvertierung von McpErrorResponse zu McpSystemClientError
    impl From<McpErrorResponse> for McpSystemClientError {
        fn from(err_resp: McpErrorResponse) -> Self {
            McpSystemClientError::McpServerErrorReply {
                error_code: err_resp.error.code,
                message: err_resp.error.message,
                diagnostic_info: err_resp.error.data.and_then(|v| serde_json::to_string(&v).ok()),
            }
        }
    }
    ```
    

#### 5.3. Untermodul: `system::mcp_client::connection_manager`

**Zweck:** Verwaltung von Verbindungen zu MCP-Servern.

**Datei:** `src/mcp_client/connection_manager.rs`

- **Struct `McpConnection`**:
    - **Felder:** Wie in der vorherigen Antwort.
        - `server_id: String`
        - `client: mcp_client_rs::McpClient`
        - `local_process_handle: Arc<tokio::sync::Mutex<Option<tokio::process::Child>>>` (Arc&lt;Mutex&lt;Option&lt;...>>> damit der Listener-Task den Prozess ggf. beenden kann)
        - `is_connected_state: Arc<tokio::sync::watch::Sender<bool>>` (Sender, damit der Manager den Status setzen kann)
        - `notification_task_handle: Option<tokio::task::JoinHandle<()>>` (Für den Task, der `client.receive_message()` lauscht)
    - **Methoden:**
        - **`pub async fn new(config: &McpServerConfig, api_key: Option<String>, event_publisher_clone: tokio::sync::broadcast::Sender<McpClientSystemEvent>, server_id_clone: String) -> Result<Self, McpSystemClientError>`**:
            1. `is_connected_state_tx = Arc::new(tokio::sync::watch::channel(false).0);`
            2. Wenn `LocalExecutable`: Starte Prozess, `client = McpClient::attach_stdio(child_process.stdin.take().unwrap(), child_process.stdout.take().unwrap()).await?`.
            3. Wenn `RemoteHttp`: `client = McpClient::connect_http(&config.endpoint_url, api_key).await?`.
            4. Setze `is_connected_state_tx.send(true).ok();`.
            5. Starte `notification_task`:
                
                Rust
                
                ```
                let client_clone = client.clone(); // McpClient muss Clone sein
                let is_connected_state_clone = is_connected_state_tx.clone();
                let publisher_clone = event_publisher_clone;
                let s_id_clone = server_id_clone;
                
                let handle = tokio::spawn(async move {
                    loop {
                        match client_clone.receive_message().await {
                            Ok(Some(McpMessage::Notification(notification))) => {
                                publisher_clone.send(McpClientSystemEvent::McpNotificationReceived {
                                    server_id: s_id_clone.clone(),
                                    interaction_id: None, // Muss aus Notification-Payload extrahiert werden, falls vorhanden
                                    notification,
                                }).ok();
                            }
                            Ok(Some(McpMessage::Response { .. })) => {
                                tracing::warn!("Unerwartete Response im Notification-Stream von MCP Server {}", s_id_clone);
                            }
                            Ok(None) => { // Stream beendet (Verbindung geschlossen)
                                tracing::info!("MCP Notification-Stream für Server {} beendet.", s_id_clone);
                                is_connected_state_clone.send(false).ok();
                                publisher_clone.send(McpClientSystemEvent::McpServerConnectionStateChanged {
                                    server_id: s_id_clone.clone(),
                                    is_connected: false,
                                    error_message: Some("Connection closed by server or stream ended.".to_string()),
                                }).ok();
                                break;
                            }
                            Err(e) => {
                                tracing::error!("Fehler beim Empfangen der MCP Notification von Server {}: {:?}", s_id_clone, e);
                                is_connected_state_clone.send(false).ok();
                                publisher_clone.send(McpClientSystemEvent::McpServerConnectionStateChanged {
                                    server_id: s_id_clone.clone(),
                                    is_connected: false,
                                    error_message: Some(format!("Receive error: {}", e)),
                                }).ok();
                                break;
                            }
                        }
                    }
                });
                ```
                
            6. Return `Self { ..., notification_task_handle: Some(handle), ... }`.
        - **`pub async fn close(&mut self)`**:
            1. `self.is_connected_state.send(false).ok();`
            2. `self.client.close().await;` (Wenn `McpClient` eine `close`-Methode hat).
            3. Wenn `notification_task_handle.take().is_some()`, `handle.abort();` (oder sanfter beenden).
            4. Wenn `local_process_handle.lock().await.take().is_some()`, `child.kill().await?`.
- **Struct `McpConnectionManager`**:
    - **Felder:** Wie in der vorherigen Antwort.
    - **Methoden:**
        - `load_server_configs` (wie zuvor).
        - `get_or_connect`:
            1. Prüft `connections` Cache. Wenn verbunden (`is_connected_state.borrow() == true`), zurückgeben.
            2. API-Key-Abruf via `secrets_service`.
            3. `McpConnection::new(...)` aufrufen.
            4. Verbindung in `connections` speichern.
            5. `event_publisher.send(McpServerConnectionStateChanged { is_connected: true, ... })`.
        - `disconnect` (wie zuvor, ruft `McpConnection::close()`).
        - `get_active_connection_for_model` (wie zuvor).

#### 5.4. Submodul: `system::mcp_client::service`

**Zweck:** Implementierung des `SystemMcpService` Traits.

**Datei:** `src/mcp_client/service.rs`

- **Trait `SystemMcpService`**: Wie in der vorherigen Antwort.
- **Struct `DefaultSystemMcpService`**:
    - **Felder:** `connection_manager: Arc<McpConnectionManager>`.
    - **Implementierung von `SystemMcpService`**:
        - **`initialize_server`, `list_resources`, `call_tool`**:
            1. `mcp_conn_arc = self.connection_manager.get_or_connect(server_id, model_profile).await?;`
            2. `let mcp_conn_guard = mcp_conn_arc; // Arc kann direkt verwendet werden, McpClient ist Clone`
            3. `let client_ref = &mcp_conn_guard.client;`
            4. Timeout erstellen: `tokio::time::timeout(Duration::from_millis(timeout_ms), client_ref.send_request_json(mcp_protocol_message)).await`
                - Wenn `Ok(Ok(response_message))`: Verarbeite `response_message`.
                - Wenn `Ok(Err(mcp_lib_err))`: `Err(McpSystemClientError::McpLibInternalError(mcp_lib_err))`.
                - Wenn `Err(_timeout_err)`: `Err(McpSystemClientError::RequestTimeout { ... })`.
            5. Wenn `response_message` eine `McpMessage::Error(err_resp)` ist:
                - `self.connection_manager.event_publisher.send(McpClientSystemEvent::McpRequestFailed { ..., error: err_resp.clone() }).ok();`
                - `Err(McpSystemClientError::from(err_resp))`
            6. Sonst: Parse Response in den erwarteten Typ (z.B. `InitializeResult`). Bei Erfolg, `McpToolCallSuccessful` Event senden (für `call_tool`).
        - **`subscribe_to_mcp_events`**: `self.connection_manager.event_publisher.subscribe()`.

#### 5.5. Implementierungsschritte `system::mcp_client`

(Wie in vorheriger Antwort, aber mit Fokus auf `tokio::sync::Mutex/RwLock/watch`, `tokio::process` und `tokio::task` für asynchrone Operationen und den Notification-Listener-Task in `McpConnection`.)

1. **Grundgerüst**: Verzeichnisse, `Cargo.toml` anpassen.
2. **`types.rs`**: `McpServerConfig` (mit `PathBuf`), `McpClientSystemEvent`, Protokoll-Typen re-exportieren.
3. **`errors.rs`**: `McpSystemClientError` (mit detaillierter Fehlerbehandlung für `McpLibInternalError` und `McpErrorResponse`).
4. **`connection_manager.rs`**:
    - `McpConnection`: `new` implementieren (Prozessstart, Verbindung, Notification-Listener-Task). `close` implementieren.
    - `McpConnectionManager`: `new`, `load_server_configs`, `get_or_connect` (mit API-Key-Abruf), `disconnect`, `get_active_connection_for_model`.
5. **`service.rs`**: `SystemMcpService`-Trait und `DefaultSystemMcpService`-Implementierung. Timeout-Logik für Anfragen. Korrektes Event-Publishing.
6. **`mod.rs`**: API re-exportieren.
7. **Unit-/Integrationstests**:
    - Mocking für `SecretsServiceClientService`.
    - Testen der lokalen Prozessstart- und Managementlogik.
    - Für HTTP-Verbindungen: Testen gegen einen einfachen Mock-MCP-HTTP-Server.
    - Testen der Timeout-Logik.
    - Testen des Notification-Listener-Tasks (Senden von Dummy-Notifications).

---

### Modul 6: `system::window_mechanics`

Zweck: Technische Umsetzung des Fenstermanagements (Positionierung, Größe, Stapelreihenfolge, Tiling, Fokus, Dekorationen) basierend auf Domänen-Policies.

Verantwortlichkeiten: Anwenden von WorkspaceWindowLayout auf Compositor-Fenster, Senden von configure-Events, Handhabung interaktiver Operationen (Move/Resize), Koordination von SSD/CSD, technische Fokusumsetzung.

Design-Rationale: Trennung von "Mechanik" und "Policy". Enge Kopplung mit system::compositor und domain::window_management_policy.

#### 6.1. Untermodul: `system::window_mechanics::types`

**Datei:** `src/window_mechanics/types.rs`

- **Struct `InteractiveOpState`**: Wie in der vorherigen Antwort.
    
    Rust
    
    ```
    use smithay::{
        utils::{Logical, Point, Rectangle, Serial},
        reexports::wayland_protocols::xdg::shell::server::xdg_toplevel::ResizeEdge,
    };
    use crate::compositor::xdg_shell::types::ManagedWindow; // Pfad anpassen
    use std::sync::Arc;
    
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum InteractiveOpType { Move, Resize(ResizeEdge) } // ResizeEdge aus xdg_toplevel
    
    #[derive(Debug, Clone)]
    pub struct InteractiveOpState {
        pub window_arc: Arc<ManagedWindow>, // Smithay's Window Trait-Objekt
        pub op_type: InteractiveOpType,
        pub start_pointer_pos_global: Point<f64, Logical>, // Globale Position beim Start des Grabs
        pub initial_window_geometry: Rectangle<i32, Logical>, // Geometrie des Fensters beim Start des Grabs
        pub last_configure_serial: Option<Serial>, // Um Configure-Storms zu vermeiden
        // Für Resize:
        pub initial_window_size_constraints: Option<(Option<Size<i32, Logical>>, Option<Size<i32, Logical>>)>, // (min_size, max_size)
    }
    ```
    
- **Event (System-intern, via `system::event_bridge`):**
    
    Rust
    
    ```
    use crate::domain::workspaces::core::types::WindowIdentifier as DomainWindowIdentifier;
    use crate::core::types::RectInt; // Aus novade-core
    use smithay::wayland::shell::xdg::ToplevelState; // Beispiel für Zustandsflags
    
    #[derive(Debug, Clone)]
    pub enum WindowMechanicsEvent {
        WindowConfiguredByMechanics { // Unterscheidung von Client-initiiertem Configure
            window_domain_id: DomainWindowIdentifier,
            new_geometry: RectInt, // Die tatsächlich angewendete Geometrie
            new_state: Vec<ToplevelState>, // z.B. Maximized, Activated etc.
        },
        InteractiveOpStarted { window_domain_id: DomainWindowIdentifier, op_type: InteractiveOpType },
        InteractiveOpEnded { window_domain_id: DomainWindowIdentifier, op_type: InteractiveOpType, final_geometry: RectInt },
        FocusSetByMechanics(Option<DomainWindowIdentifier>), // Wenn der Fokus durch Mechanics geändert wurde
    }
    ```
    

#### 6.2. Untermodul: `system::window_mechanics::errors`

**Datei:** `src/window_mechanics/errors.rs`

- **Enum `WindowMechanicsError`**: Wie in der vorherigen Antwort.
    - Zusätzlich: `#[error("Window {0:?} does not support the requested operation (e.g., trying to resize a non-resizable window).")] WindowOperationNotSupported(DomainWindowIdentifier)`

#### 6.3. Untermodul: `system::window_mechanics::layout_applier`

**Zweck:** Anwenden eines von der Domäne berechneten Layouts.

**Datei:** `src/window_mechanics/layout_applier.rs`

- **Funktion `pub async fn apply_workspace_layout(desktop_state_accessor: &impl Fn() -> Arc<Mutex<DesktopState>>, workspace_id: WorkspaceId, layout: &WorkspaceWindowLayout) -> Result<(), WindowMechanicsError>`**:
    
    - **Annahme:** `DesktopStateAccessor` ist ein Weg, um Zugriff auf `DesktopState` zu bekommen, da `DesktopState` selbst nicht einfach `Send` sein könnte für `async fn`. Einfacher: Wenn `apply_workspace_layout` von einem Ort aufgerufen wird, der bereits Zugriff auf `&mut DesktopState` hat (z.B. innerhalb eines `calloop` Callbacks oder eines `tokio::task::block_in_place`), dann kann es synchron sein. Für die Spezifikation nehmen wir an, dass es irgendwie Zugriff bekommt.
    - **Alternative (synchron, wenn im Compositor-Thread):** `pub fn apply_workspace_layout_blocking(desktop_state: &mut DesktopState, workspace_id: WorkspaceId, layout: &WorkspaceWindowLayout) -> Result<(), WindowMechanicsError>` <!-- end list -->
    
    1. `tracing::debug!("Wende Layout für Workspace {:?} an: {:?}", workspace_id, layout);`
    2. `let space = &mut desktop_state.space;`
    3. `let windows_map = &mut desktop_state.windows;` // Map von DomainWindowIdentifier zu Arc&lt;ManagedWindow>
    4. Für jede `(domain_id, target_geometry)` in `layout.window_geometries`:
        - `let managed_window_arc = match windows_map.get(&domain_id) { Some(w) => w.clone(), None => { tracing::warn!("Fenster {:?} im Layout nicht in DesktopState gefunden.", domain_id); continue; } };`
        - `let mut managed_window_ref = (*managed_window_arc).clone(); // Klone Arc für Smithay-Window-Trait-Methoden, wenn ManagedWindow selbst nicht Mutex-geschützt ist.`
        - `managed_window_ref.current_geometry = target_geometry;` // Internen Zustand aktualisieren
        - **Senden der Konfiguration an den Client (Beispiel für Toplevel):**
            - `if let WindowSurface::Toplevel(toplevel) = &managed_window_ref.xdg_surface { ... }`
            - `let mut new_xdg_states = Vec::new();`
            - // Logik, um layout.tiling_mode_applied in XDG-Zustände zu übersetzen (z.B. Maximized)
            - // if layout.tiling_mode_applied == TilingMode::MaximizedFocused && Some(&amp;domain_id) == layout.focused_window_id {
            - // new_xdg_states.push(xdg_toplevel::State::Maximized);
            - // }
            - `toplevel.with_pending_state(|xdg_state| { xdg_state.size = Some(target_geometry.size); xdg_state.states = ToplevelState::new(&new_xdg_states); });`
            - `toplevel.send_configure();`
        - `space.map_window(managed_window_arc.clone(), target_geometry.loc, true);` (Stellt sicher, dass es im Space ist und die Position aktualisiert wird. `true` für Aktivierung, falls es das fokussierte Fenster sein soll – Fokuslogik ist separat).
        - `desktop_state.event_bridge.publish_window_mechanics_event(WindowMechanicsEvent::WindowConfiguredByMechanics { ... });`
    5. Für Fenster, die im `space` sind, aber NICHT in `layout.window_geometries` (d.h. sollen nicht mehr auf diesem Workspace sichtbar sein, z.B. weil geschlossen oder auf anderen WS verschoben):
        - `if let Some(window_to_unmap) = windows_map.get(&domain_id_im_space) { space.unmap_window(window_to_unmap); }`
    6. `space.damage_all_outputs();` (Oder spezifischer: `space.damage_elements(betroffene_fenster)`).
    7. `Ok(())`

#### 6.4. Untermodul: `system::window_mechanics::interactive_ops`

**Zweck:** Handhabung von interaktiven Fenster-Moves und -Resizes.

**Datei:** `src/window_mechanics/interactive_ops.rs`

- **Struct `PointerMoveResizeGrab`**: Implementiert `smithay::input::pointer::PointerGrab<DesktopState>`.
    - **Felder:** `op_state: InteractiveOpState`, `desktop_state_accessor: impl Fn() -> Arc<Mutex<DesktopState>>` (oder `Weak<Mutex<DesktopState>>`), `window_policy_service: Arc<dyn WindowManagementPolicyService>`.
    - **`motion(...)` Logik:**
        1. `current_pointer_pos_global = global_grab_start_pos + (current_event_pos - op_state.start_pointer_pos_pointer_local);`
        2. `new_geometry = calculate_new_geometry_for_op(&op_state.initial_window_geometry, &op_state.op_type, current_pointer_pos_global, op_state.start_pointer_pos_global);`
        3. **Snapping:**
            - `other_windows_on_workspace = collect_other_windows_geometries(desktop_state_accessor, &op_state.window_arc);`
            - `snapping_policy = block_on(self.window_policy_service.get_effective_snapping_policy());`
            - `gap_settings = block_on(self.window_policy_service.get_effective_gap_settings_for_workspace(...));`
            - `if let Some(snapped_geom) = block_on(self.window_policy_service.calculate_snap_target(&op_state.window_arc.domain_id, new_geometry, &other_windows_on_workspace, workspace_area, &snapping_policy, &gap_settings)) { new_geometry = snapped_geom; }`
        4. **Größenbeschränkungen anwenden:** Klemme `new_geometry.size` auf `op_state.initial_window_size_constraints`.
        5. Aktualisiere `op_state.window_arc.current_geometry = new_geometry;` (visuelles Feedback, kein Configure).
        6. `desktop_state_accessor().lock().unwrap().space.damage_window(&op_state.window_arc, None, None);` (Alte und neue Position beschädigen).
    - **`button(...)` Logik:**
        1. Wenn Maustaste losgelassen:
            - Finalisiere `final_geometry = op_state.window_arc.current_geometry;`
            - `if let WindowSurface::Toplevel(toplevel) = &op_state.window_arc.xdg_surface { ... send_configure mit final_geometry ... }`
            - `pointer_handle.unset_grab(serial, time);`
            - `desktop_state_accessor().lock().unwrap().event_bridge.publish_window_mechanics_event(InteractiveOpEnded { ... });`
- **Funktionen `start_interactive_move` / `start_interactive_resize`**:
    1. Erstellen `InteractiveOpState`.
    2. `pointer_handle.set_grab(serial, PointerMoveResizeGrab { ... }, Focus::Clear);`

#### 6.5. Submodul: `system::window_mechanics::focus_manager` (Vervollständigung)

**Zweck:** Technische Umsetzung des Fokuswechsels basierend auf Domänenentscheidungen und Benutzerinteraktionen.

**Datei:** `src/window_mechanics/focus_manager.rs`

- **Funktion `pub async fn set_application_focus(desktop_state_accessor: &impl Fn() -> Arc<Mutex<DesktopState>>, seat_name: &str, window_domain_id_to_focus: Option<&DomainWindowIdentifier>, serial: Serial) -> Result<(), WindowMechanicsError>`**:
    1. `let mut ds_guard = desktop_state_accessor().lock().await;`
    2. `let seat = ds_guard.seat_state.seats().find(|s| s.name() == seat_name).cloned().ok_or(WindowMechanicsError::InternalError("Seat nicht gefunden".into()))?;`
    3. `let old_focused_window_domain_id = ds_guard.active_input_surface.as_ref().and_then(|weak_surf| weak_surf.upgrade()).and_then(|surf| find_domain_id_for_surface(&ds_guard, &surf));`
    4. Finde `target_managed_window_arc_option`:
        - Wenn `window_domain_id_to_focus` `Some(id)`, suche in `ds_guard.windows.get(id)`.
        - Sonst `None`.
    5. `let target_wl_surface_option = target_managed_window_arc_option.as_ref().map(|arc_win| arc_win.wl_surface().clone());`
    6. Rufen Sie `crate::input::keyboard::focus::set_keyboard_focus(&mut ds_guard, seat_name, target_wl_surface_option.as_ref(), serial)?;`
    7. // Aktivierungslogik für XDG Toplevel
        - Wenn `old_focused_window_domain_id` existiert und sich von `window_domain_id_to_focus` unterscheidet:
            - Finde altes `ManagedWindow`. Wenn Toplevel, `old_toplevel.set_activated(false); old_toplevel.send_configure();`.
        - Wenn `target_managed_window_arc_option` ein Toplevel ist (`newly_focused_toplevel`):
            - `newly_focused_toplevel.set_activated(true); newly_focused_toplevel.send_configure();`
            - `ds_guard.space.raise_window(&newly_focused_toplevel_arc, true);` // True für Fokus
            - `ds_guard.active_input_surface = target_wl_surface_option.map(|s| s.downgrade());`
    8. `ds_guard.event_bridge.publish_window_mechanics_event(FocusSetByMechanics(window_domain_id_to_focus.cloned()));`
    9. `Ok(())`
- **Funktion `fn find_domain_id_for_surface(ds: &DesktopState, surface: &WlSurface) -> Option<DomainWindowIdentifier>`**: Iteriert `ds.windows` und vergleicht `wl_surface()`.

#### 6.6. Implementierungsschritte `system::window_mechanics`

1. `types.rs`, `errors.rs` definieren.
2. `layout_applier.rs`: `apply_workspace_layout_blocking` implementieren. Fokus auf korrekte `configure`-Events.
3. `interactive_ops.rs`: `PointerMoveResizeGrab` mit `PointerGrab` Trait implementieren. `start_interactive_move/resize` Funktionen. Snapping-Logik integrieren.
4. `focus_manager.rs`: `set_application_focus` mit XDG-Aktivierungslogik und Space-Interaktion.
5. Unit-Tests (sehr komplex):
    - Testen der `apply_workspace_layout` für verschiedene Layouts (mock `DesktopState` und `ManagedWindow`s).
    - Testen der Grab-Handler-Logik (Zustandsübergänge, Geometrieberechnung).
    - Testen der Fokus-Aktivierungslogik.

---

### Modul 7: `system::power_management` (Vervollständigung)

Zweck: DPMS, Interaktion mit logind für Suspend/Hibernate, Reaktion auf Benutzerinaktivität.

Bestehende Spezifikation: Skizziert in der vorherigen Antwort.

#### 7.1. Untermodul: `system::power_management::types`

**Datei:** `src/power_management/types.rs`

- **Enum `DpmsState`**: Wie in der vorherigen Antwort (`On, Standby, Suspend, Off`). Serde für Konfiguration.
- **Event `PowerManagementSystemEvent`** (umbenannt von `PowerManagementEvent`):
    
    Rust
    
    ```
    use crate::dbus_interfaces::logind_client::types::LogindPowerOperation; // Pfad anpassen
    
    #[derive(Debug, Clone)]
    pub enum PowerManagementSystemEvent {
        OutputDpmsStateSet { output_name: String, new_state: DpmsState, success: bool },
        SystemSuspendingInitiated(LogindPowerOperation), // Vom logind_client erhalten
        SystemResumedNormally,
        ScreenBlankTimeoutReached(String /* output_name */),
        SystemIdleTimeoutReached, // Für Suspend/Hibernate
    }
    ```
    
- **Struct `IdleTimerState`**: Wie in der vorherigen Antwort, aber `timer_handle` muss `Send + Sync` sein, wenn der Service in einem `tokio::task` läuft und den Timer in `calloop` der Compositor-Schleife managt. Besser: Timer wird über einen Befehl an den Compositor-Thread gesetzt.
    
    Rust
    
    ```
    use chrono::{DateTime, Utc, Duration as ChronoDuration}; // Duration für Timer
    use calloop::TimerHandle; // Wenn Timer in calloop läuft
    
    #[derive(Debug)] // TimerHandle ist nicht Clone/Debug
    pub struct IdleTimerState {
        pub timer_id: String, // Eindeutige ID für den Timer (z.B. "output-HDMI-A-1-blank", "system-suspend")
        pub last_activity_ts: DateTime<Utc>,
        pub current_timeout_duration: ChronoDuration,
        // pub calloop_timer_handle: Option<TimerHandle>, // Wenn direkt in calloop
        // Alternativ: Timestamp, wann der Timer ablaufen soll
        pub scheduled_expiry_ts: Option<DateTime<Utc>>,
    }
    ```
    

#### 7.2. Untermodul: `system::power_management::errors`

**Datei:** `src/power_management/errors.rs`

- **Enum `PowerManagementError`**: Wie in der vorherigen Antwort.

#### 7.3. Untermodul: `system::power_management::service`

**Datei:** `src/power_management/service.rs`

- **Trait `PowerManagementControl`** (Schnittstelle zum Compositor/DRM-Backend für DPMS):
    
    Rust
    
    ```
    use async_trait::async_trait;
    use super::types::DpmsState;
    use super::errors::PowerManagementError;
    
    #[async_trait]
    pub trait PowerManagementControl: Send + Sync {
        async fn set_output_dpms_state(&self, output_name: &str, state: DpmsState) -> Result<(), PowerManagementError>;
        async fn list_outputs_for_dpms(&self) -> Result<Vec<String>, PowerManagementError>; // Gibt Namen der relevanten Outputs
    }
    ```
    
- **Struct `PowerManagementService`**:
    - **Felder:**
        - `settings_service: Arc<dyn GlobalSettingsService>`
        - `logind_service: Arc<dyn LogindClientService>`
        - `compositor_dpms_control: Arc<dyn PowerManagementControl>` (Injizierte Abhängigkeit)
        - `event_publisher: tokio::sync::broadcast::Sender<PowerManagementSystemEvent>`
        - `system_event_receiver: tokio::sync::broadcast::Receiver<crate::event_bridge::SystemLayerEvent>` (Empfängt `UserActivityDetectedEvent`, `LogindSystemEvent::PrepareForSleep/Shutdown`, `SettingChangedEvent` für Power-Settings).
        - `active_timers: Arc<tokio::sync::Mutex<HashMap<String /* timer_id */, IdleTimerState>>>`
        - `current_power_settings: Arc<tokio::sync::RwLock<crate::domain::global_settings_and_state_management::types::PowerManagementPolicySettings>>`
        - `on_ac_power: Arc<tokio::sync::RwLock<bool>>` (Wird durch UPower-Events aktualisiert)
    - **Konstruktor `new(...)`**: Nimmt Abhängigkeiten, abonniert Events vom `SystemEventBridge`.
    - **Methode `pub async fn run(&self)`**: Haupt-Task des Dienstes.
        1. Lädt initiale `PowerManagementPolicySettings` und `on_ac_power`-Status.
        2. Initialisiert/Resettet alle Idle-Timer basierend auf aktuellen Einstellungen und AC-Status.
        3. **Event-Loop (`tokio::select!`)**:
            - Hört auf `system_event_receiver`:
                - `UserActivityDetectedEvent`: `self.reset_all_idle_timers().await;`
                - `LogindSystemEvent::PrepareForSleep(is_suspending)`: Setze alle Outputs auf `DpmsState::Off`. `self.cancel_all_idle_timers().await;`
                - `LogindSystemEvent::SystemResumed`: `self.reset_all_idle_timers().await;` Setze Outputs auf `DpmsState::On`.
                - `SettingChangedEvent` für Power-Pfade: Lade `current_power_settings` neu, `self.reset_all_idle_timers().await;`
                - `UPowerSystemEvent::OnBatteryChanged(is_on_battery)`: Aktualisiere `self.on_ac_power`, `self.reset_all_idle_timers().await;`
            - Hört auf Timer-Abläufe (wenn Timer in `tokio` verwaltet werden, z.B. `tokio::time::sleep_until` für jeden Timer in einem separaten Task, der dann eine Nachricht an diesen Haupt-Task sendet).
                - Wenn "Screen Blank Timeout" für einen Output abläuft: `self.apply_dpms_state(output_name, DpmsState::Off).await;` Sende `ScreenBlankTimeoutReached`.
                - Wenn "System Suspend Timeout" abläuft:
                    - Rufe `self.logind_service.suspend(false).await` oder `hibernate(false).await` basierend auf Policy. Sende `SystemIdleTimeoutReached`.
    - **Private Methoden:**
        - `async fn reset_all_idle_timers(&self)`: Liest aktuelle Settings und AC-Status. Berechnet neue Timeout-Dauern (z.B. `screen_blank_timeout_ac_secs` vs. `_battery_secs`). Startet/Neustartet `tokio::time::sleep_until` für jeden Output-Timer und den System-Suspend-Timer. Speichert `ScheduledExpiryTs` in `IdleTimerState`.
        - `async fn cancel_all_idle_timers(&self)`: Bricht laufende `tokio::time::sleep_until` ab (indem die Tasks, die sie verwalten, beendet werden oder indem `scheduled_expiry_ts` auf `None` gesetzt wird).
        - `async fn apply_dpms_state(...)`: Ruft `self.compositor_dpms_control.set_output_dpms_state(...)`. Sendet `OutputDpmsStateSet`.

#### 7.4. Implementierungsschritte `system::power_management`

1. `types.rs`, `errors.rs` definieren. `PowerManagementControl`-Trait definieren.
2. `service.rs`: `PowerManagementService` implementieren.
    - Event-Loop-Logik mit `tokio::select!`.
    - Timer-Management mit `tokio::time::sleep_until` (oder Integration mit `calloop`, falls performanter/einfacher im Compositor-Kontext).
    - Interaktion mit `GlobalSettingsService`, `LogindClientService`, `PowerManagementControl`.
3. Sicherstellen, dass `PowerManagementControl` im Compositor-Modul implementiert wird (z.B. `impl PowerManagementControl for DesktopState`).
4. Tests: Mocking von Abhängigkeiten. Testen der Timer-Logik, korrekte Reaktion auf Events, korrekte Anwendung von Policies (AC vs. Batterie).

---

### Modul 8: `system::event_bridge` (Vervollständigung)

Zweck: Zentrale Event-Verteilung innerhalb der Systemschicht und ggf. an die Domänenschicht für System-level Events.

Bestehende Spezifikation: Skizziert in der vorherigen Antwort.

**Datei:** `src/event_bridge/events.rs` (Definition aller System-internen Events)

- Hier werden alle Events definiert, die in den `types.rs`-Dateien der Submodule (`UPowerEvent`, `LogindEvent`, `NetworkManagerEvent`, `AudioEvent`, `McpClientSystemEvent`, `WindowMechanicsEvent`, `PowerManagementSystemEvent`, `InputDeviceHotplugEvent`) definiert wurden, ggf. gewrappt in ein übergreifendes `SystemLayerEvent`-Enum.
    
    Rust
    
    ```
    // Beispiel:
    // use crate::dbus_interfaces::upower_client::types::UPowerEvent;
    // use crate::input::types::InputDeviceHotplugEvent; // Beispiel
    // ...
    
    #[derive(Debug, Clone)] // Ggf. Serialize/Deserialize wenn über Grenzen gesendet
    pub enum SystemLayerEvent {
        UPower(UPowerEvent),
        Logind(LogindEvent),
        NetworkManager(NetworkManagerEvent),
        Audio(AudioEvent),
        McpClient(McpClientSystemEvent),
        WindowMechanics(WindowMechanicsEvent),
        PowerManagement(PowerManagementSystemEvent),
        InputDeviceHotplug(InputDeviceHotplugEvent),
        // Auch Domänen-Events, die von Systemschicht ausgelöst werden
        DomainUserActivity(crate::domain::common_events::UserActivityDetectedEvent),
        DomainSystemShutdown(crate::domain::common_events::SystemShutdownInitiatedEvent),
    }
    ```
    

**Datei:** `src/event_bridge/mod.rs`

- **Struct `SystemEventBridge`**:
    - **Felder:** Hält einen einzelnen `tokio::sync::broadcast::Sender<SystemLayerEvent>`.
        - `event_tx: tokio::sync::broadcast::Sender<SystemLayerEvent>`
    - **Konstruktor `new(capacity: usize) -> Self`**.
    - **Methoden:**
        - `pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<SystemLayerEvent>`
        - `pub(crate) fn publish(&self, event: SystemLayerEvent)`: Sendet das Event. Loggt Fehler, wenn keine Subscriber vorhanden sind.
- **Globale Instanz (optional aber oft praktisch):**
    
    Rust
    
    ```
    use std::sync::OnceLock;
    static SYSTEM_EVENT_BRIDGE_INSTANCE: OnceLock<SystemEventBridge> = OnceLock::new();
    
    pub fn global_system_event_bridge() -> &'static SystemEventBridge {
        SYSTEM_EVENT_BRIDGE_INSTANCE.get().expect("SystemEventBridge nicht initialisiert")
    }
    
    pub fn initialize_global_system_event_bridge(capacity: usize) {
        if SYSTEM_EVENT_BRIDGE_INSTANCE.set(SystemEventBridge::new(capacity)).is_err() {
            tracing::warn!("SystemEventBridge bereits initialisiert.");
        }
    }
    ```
    
    - Die Initialisierung würde sehr früh im Startprozess der Systemschicht erfolgen.

**Implementierungsschritte `system::event_bridge`**:

1. Alle relevanten Event-Typen in `events.rs` (oder den jeweiligen Modulen und hier re-exportiert/gewrappt) definieren.
2. `SystemEventBridge`-Struktur und globale Instanz implementieren.
3. Alle Systemmodule, die Events publizieren, nutzen `global_system_event_bridge().publish(...)`.
4. Alle Systemmodule (und Adapter zur Domänenschicht), die Events konsumieren, nutzen `global_system_event_bridge().subscribe()` und filtern dann nach dem `SystemLayerEvent`-Typ.

---

**Finale `novade-system/src/lib.rs` (Struktur):**

Rust

```
// novade-system/src/lib.rs

// Kern- und Domänen-Abhängigkeiten
// use novade_core;
// use novade-domain;

// System-interne Module
pub mod audio_management;
pub mod compositor;
pub mod dbus_interfaces;
pub mod event_bridge; // Muss früh initialisiert werden
pub mod input;
pub mod mcp_client;
pub mod power_management;
pub mod window_mechanics;

// Öffentliche API der Systemschicht (Traits und wichtige Typen)
// Diese werden typischerweise von der Hauptanwendung (Binary) oder der UI-Schicht genutzt.
// Beispiel:
// pub use compositor::CompositorControlService; // Hypothetischer Trait
// pub use input::InputControlService;          // Hypothetischer Trait
pub use dbus_interfaces::{
    UPowerClientService, LogindClientService, NetworkManagerClientService,
    SecretsServiceClientService, PolicyKitClientService, FreedesktopNotificationsServer,
    // Re-export der Service-Traits wäre hier besser als konkrete Typen, falls Traits existieren
};
pub use audio_management::{PipeWireClientService /* oder Trait */, AudioEvent};
pub use mcp_client::{SystemMcpService, McpClientSystemEvent, McpServerConfig};
// ... usw.

// Initialisierungsfunktion für die gesamte Systemschicht
// pub async fn initialize_system_layer(
//     core_services: Arc<CoreServices>, // Hypothetische Sammlung von Kernschicht-Services
//     domain_services: Arc<DomainServices>, // Hypothetische Sammlung von Domänenschicht-Services
//     display_handle: DisplayHandle, // Vom Backend (DRM, Winit)
//     loop_handle: LoopHandle<'static, DesktopState>, // Vom Backend
// ) -> Result<SystemServices, SystemInitializationError> {
//
//     event_bridge::initialize_global_system_event_bridge(1024);
//     let event_bridge = event_bridge::global_system_event_bridge();
//
//     // DesktopState (Compositor-Herzstück)
//     let desktop_state = Arc::new(Mutex::new(DesktopState::new(loop_handle.clone(), display_handle.clone(), domain_services.window_policy_service.clone(), ...)));
//
//     // Input-System initialisieren und in calloop registrieren
//     // let libinput_backend = input::libinput_handler::init_libinput_backend(&loop_handle, session_interface).await?;
//     // input::libinput_handler::register_libinput_source(&loop_handle, libinput_backend, "seat0".to_string(), desktop_state.clone())?;
//     // input::seat_manager::create_seat(&mut desktop_state.lock().unwrap(), &display_handle, "seat0".to_string())?;
//
//     // D-Bus Clients initialisieren
//     // let upower_client = Arc::new(UPowerClientService::new(event_bridge.publisher_for_upower_events()).await?);
//     // upower_client.initialize_and_listen().await?;
//     // ... für andere D-Bus Clients ...
//
//     // PipeWire Client initialisieren
//     // let audio_service = Arc::new(PipeWireClientService::new(event_bridge.publisher_for_audio_events()).await?);
//
//     // MCP Client initialisieren
//     // let mcp_connection_manager = Arc::new(McpConnectionManager::new(secrets_service_client, event_bridge.publisher_for_mcp_events()));
//     // let mcp_service = Arc::new(DefaultSystemMcpService::new(mcp_connection_manager));
//     // mcp_service.configure_servers(domain_services.global_settings.mcp_server_configs).await?;
//
//     // Window Mechanics (ist oft eng mit DesktopState oder als Service)
//     // let window_mechanics_service = Arc::new(DefaultWindowMechanicsService::new(desktop_state.clone(), domain_services.window_policy_service.clone()));
//
//     // Power Management Service
//     // let power_management_dpms_control = desktop_state.clone(); // DesktopState implementiert PowerManagementControl
//     // let power_service = Arc::new(PowerManagementService::new(domain_services.settings_service, logind_client, power_management_dpms_control, ...));
//     // tokio::spawn(async move { power_service.run().await; });
//
//     // Alle Wayland Globals erstellen (nachdem alle States in DesktopState initialisiert sind)
//     // compositor::core::create_all_wayland_globals(&mut desktop_state.lock().unwrap(), &display_handle)?;
//
//     // D-Bus Notifications Server starten (wenn NovaDE ihn bereitstellt)
//     // tokio::spawn(async move {
//     //    dbus_interfaces::notifications_server::run_notifications_server(domain_services.notification_service).await;
//     // });
//
//     // SystemServices struct zurückgeben, das Handles zu den wichtigsten System-Services enthält
//     // Ok(SystemServices { desktop_state_handle: desktop_state, ... })
// }
```

Mit diesen detaillierten Plänen für die Module 5 bis 8 ist die Spezifikation der Systemschicht nun vollständig. Die Implementierung erfordert sorgfältige Beachtung der Asynchronität, der Inter-Prozess-Kommunikation und der Integration mit den Domänen- und Kernschichten. Die Testbarkeit bleibt eine Herausforderung und erfordert umfangreiches Mocking oder dedizierte Testumgebungen.
