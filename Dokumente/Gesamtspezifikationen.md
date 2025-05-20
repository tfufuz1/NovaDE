# NovaDE: Technische Gesamtspezifikation und Richtliniensammlung

## 1. Einleitung

Dieses Dokument dient als zentrale technische Gesamtspezifikation und Richtliniensammlung für die Linux Desktop-Umgebung 'NovaDE'. Es hat zum Ziel, ein umfassendes Verständnis des Gesamtsystems zu ermöglichen und sicherzustellen, dass alle technischen Anforderungen, die sich aus den Beschreibungen der Nutzererfahrung (UX) ergeben, adäquat berücksichtigt und integriert werden. Die Spezifikation richtet sich an Entwickler, Architekten, Tester und alle weiteren Stakeholder, die an der Entwicklung, Wartung und Erweiterung von NovaDE beteiligt sind.

Eine präzise und detaillierte technische Spezifikation ist unerlässlich für den Erfolg komplexer Softwareprojekte.1 Sie minimiert Risiken durch frühzeitige Definition von Erwartungen und Randbedingungen, verbessert die Kommunikation zwischen den Beteiligten durch Schaffung einer gemeinsamen Referenz und steigert die Effizienz im Entwicklungsprozess.1 Studien belegen, dass detaillierte Spezifikationen Implementierungszeiten und Kosten durch Last-Minute-Änderungen signifikant reduzieren können.1 Dieses Dokument legt den Grundstein für Qualität, Wartbarkeit und Skalierbarkeit von NovaDE.2

### 1.1. Zweck und Geltungsbereich

Der Zweck dieses Dokuments ist die Definition der technischen Architektur, der Kernkomponenten, der Entwicklungsprozesse und der Qualitätsstandards für NovaDE. Es umfasst:

- Eine detaillierte Beschreibung der Systemarchitektur und ihrer Komponenten.
- Funktionale und nicht-funktionale Anforderungen, die sich aus der angestrebten Nutzererfahrung ableiten.
- Technologische Entscheidungen und deren Begründungen.
- Richtlinien für Entwicklung, Code-Qualität, Testing, Dokumentation und Sicherheit.
- Schnittstellendefinitionen für die Interaktion zwischen Komponenten und mit externen Systemen.
- Richtlinien für die Systemintegration und Erweiterbarkeit.

Der Geltungsbereich erstreckt sich auf alle Aspekte der NovaDE-Software, von den Low-Level-Systemdiensten bis hin zu den Benutzeroberflächenkomponenten. Es dient als maßgebliche Referenz für alle Entwicklungsaktivitäten.

### 1.2. Zielgruppe

Dieses Dokument richtet sich primär an:

- **Softwarearchitekten und -entwickler:** Für das Design, die Implementierung und Wartung von NovaDE-Komponenten.
- **Qualitätssicherungsingenieure und Tester:** Für die Erstellung von Testplänen und die Verifizierung der Systemanforderungen.
- **Projektmanager und Produktmanager:** Für das Verständnis der technischen Grundlagen und zur Planung von Entwicklungszyklen.
- **Systemintegratoren und Drittanbieter-Entwickler:** Für die Integration von Anwendungen und die Entwicklung von Erweiterungen für NovaDE.
- **Technische Redakteure:** Für die Erstellung von Endbenutzer- und Entwicklerdokumentation.

Die klare und präzise Formulierung soll auch nicht-technischen Stakeholdern ein grundlegendes Verständnis ermöglichen, wobei technische Details für Experten ausreichend tiefgehend behandelt werden.1

### 1.3. Definitionen und Akronyme

Eine umfassende Liste von Definitionen und Akronymen, die in diesem Dokument verwendet werden, findet sich im Anhang (Abschnitt 8.1). Dies dient der Vermeidung von Mehrdeutigkeiten und stellt ein einheitliches Verständnis sicher.4

### 1.4. Referenzierte Dokumente

Alle externen Dokumente, Standards und Spezifikationen, auf die in diesem Text Bezug genommen wird, sind im Anhang (Abschnitt 8.2) aufgeführt. Dies gewährleistet die Nachvollziehbarkeit und ermöglicht den Zugriff auf weiterführende Informationen.2

## 2. Systemübersicht und Architektur

Dieser Abschnitt beschreibt die übergeordnete Architektur von NovaDE und die Kernkomponenten, aus denen sich die Desktop-Umgebung zusammensetzt. Die Architektur ist darauf ausgelegt, eine moderne, performante und flexible Nutzererfahrung zu ermöglichen, basierend auf aktuellen Technologien wie Wayland, GTK4 und Rust.

### 2.1. Systemarchitektur

NovaDE folgt einer modularen Architektur, die eine klare Trennung der Verantwortlichkeiten zwischen den einzelnen Komponenten anstrebt. Dies fördert die Wartbarkeit, Testbarkeit und unabhängige Entwicklung der Systemteile. Die Architektur basiert auf einem Wayland-Compositor als Kernstück, der für die Darstellung und Verwaltung von Fenstern zuständig ist. Darauf aufbauend existieren eine Desktop-Shell für die Nutzerinteraktion, ein Session-Manager für den Lebenszyklus der Desktop-Sitzung sowie diverse Systemdienste für Hintergrundaufgaben und die Integration mit der Hardware und anderen Software-Subsystemen.

Die Kommunikation zwischen den Prozessen erfolgt primär über D-Bus für Steuerungs- und Benachrichtigungsaufgaben sowie über das Wayland-Protokoll für die Display-Server-Kommunikation. PipeWire wird für das Multimedia-Routing und -Management eingesetzt. Die Verwendung von Rust als primäre Programmiersprache zielt auf Systemsicherheit und Performance ab.

Die Architektur muss die Skalierbarkeit und Zuverlässigkeit des Systems sicherstellen.1 Eine gut dokumentierte Architektur ist entscheidend, um das System zu verstehen, weiterzuentwickeln und zu skalieren, da sonst wichtige Informationen verloren gehen können.3

### 2.2. Kernkomponenten und ihre Verantwortlichkeiten

Die folgende Tabelle listet die Kernkomponenten von NovaDE, ihre Hauptverantwortlichkeiten, die eingesetzten Schlüsseltechnologien und ihre wesentlichen Interaktionen auf.

**Tabelle 1: Kernkomponenten und ihre Verantwortlichkeiten**

|   |   |   |   |
|---|---|---|---|
|**Komponente**|**Hauptverantwortlichkeiten**|**Schlüsseltechnologien**|**Wesentliche Interaktionen**|
|**NovaDE Compositor**|Fenster-Management, Darstellung, Eingabeverarbeitung, Implementierung des Wayland-Protokolls, Compositing-Effekte.|Wayland, Smithay (Rust-Bibliothek), EGL, evdev (Kernel)|Empfängt Eingabeereignisse vom Kernel (evdev), kommuniziert mit Clients über Wayland-Protokoll, rendert Fensterinhalte auf den Bildschirm (KMS).5 Stellt sicher, dass keine Bildrisse (Tearing) auftreten und Fensterattribute korrekt dargestellt werden.5|
|**NovaDE Shell**|Bereitstellung der primären Benutzeroberfläche (Panel, Anwendungsstarter, Task-Manager, System-Tray, Desktop-Hintergrund, Widgets).|GTK4, Rust|Interagiert mit dem Compositor zur Platzierung und Darstellung von Shell-Elementen, startet Anwendungen, kommuniziert mit dem Session Manager und Systemdiensten über D-Bus für Statusinformationen und Aktionen. Lädt Menüs aus Builder-Ressourcen.7|
|**NovaDE Session Manager**|Verwaltung des Lebenszyklus einer Benutzersitzung (Login, Logout, Sperren), Starten der Kernkomponenten (Compositor, Shell, erforderliche Dienste).|D-Bus, systemd (optional für Service-Management)|Authentifiziert Benutzer, startet den Compositor und die Shell, verwaltet Umgebungsvariablen, kommuniziert mit systemd für den Start/Stopp von User-Services, reagiert auf Power-Management-Events über D-Bus.8|
|**NovaDE Settings Daemon**|Verwaltung und Bereitstellung globaler und benutzerspezifischer Einstellungen (Theme, Schriftarten, Eingabegeräte, Monitoreinstellungen).|D-Bus, GSettings (oder äquivalente Rust-Implementierung)|Stellt Einstellungen über D-Bus bereit, auf die Anwendungen und Shell-Komponenten zugreifen können. Liest Einstellungen aus Konfigurationsdateien (z.B. `settings.ini` für GTK) und ggf. DConf.9 Ermöglicht Anwendungen, systemweite Einstellungen zu überschreiben (mit Vorsicht zu verwenden).9|
|**NovaDE Notification Service**|Empfang, Verwaltung und Anzeige von Desktop-Benachrichtigungen von Anwendungen und Systemdiensten.|D-Bus (Freedesktop Notification Specification), GTK4|Empfängt Benachrichtigungsanfragen über D-Bus, verwaltet eine Warteschlange, zeigt Benachrichtigungen gemäß den Benutzereinstellungen an (z.B. Pop-ups, Benachrichtigungscenter). Interagiert mit der Shell zur Darstellung.|
|**NovaDE Power Manager**|Überwachung des Batteriestatus, Verwaltung von Energieeinstellungen, Behandlung von Suspend/Hibernate-Zuständen, Bildschirmhelligkeit.|D-Bus, UPower (oder direkte Kernel-Schnittstellen)|Kommuniziert mit UPower (oder äquivalent) für Batteriestatus, implementiert Energieprofile, initiiert Suspend/Hibernate über D-Bus-Aufrufe an `logind` oder systemd.8|
|**NovaDE Workspace Manager**|Verwaltung virtueller Desktops/Arbeitsbereiche, Fensterplatzierung über Arbeitsbereiche hinweg.|Integriert in Shell und Compositor|Ermöglicht dem Benutzer das Erstellen, Wechseln und Verwalten von Arbeitsbereichen. Der Compositor ist für die tatsächliche Zuordnung von Fenstern zu Arbeitsbereichen und deren Darstellung zuständig. Die Shell stellt die UI-Elemente zur Interaktion bereit. Die Logik kann komplex sein und erfordert eine sorgfältige Verwaltung von Zuständen und Übergängen, ähnlich wie bei Projektmanagement-Tools.10|
|**PipeWire Integration**|Audio- und Video-Stream-Management, Hardware-Abstraktion für Multimedia, Screen-Sharing-Unterstützung.|PipeWire, WirePlumber (Session Manager)|Stellt Audio- und Video-Funktionalität für Anwendungen bereit, ermöglicht Screen-Sharing unter Wayland, verwaltet Audiogeräte und -profile. Interagiert mit dem Kernel (ALSA) und Anwendungen.12 Bietet Ersatz für PulseAudio und JACK.13|
|**D-Bus Broker**|Vermittlung der Interprozesskommunikation zwischen NovaDE-Komponenten und Anwendungen.|D-Bus Daemon (z.B. `dbus-daemon` oder `dbus-broker`)|Stellt System- und Session-Busse bereit, über die Dienste ihre Funktionalität anbieten (Methoden, Signale, Eigenschaften) und Clients diese nutzen können.8|
|**XWayland Server**|Ausführung von X11-Anwendungen unter der Wayland-basierten NovaDE-Umgebung.|XWayland, Xorg-Server-Codebase|Dient als Kompatibilitätsschicht, indem ein X-Server innerhalb der Wayland-Sitzung läuft und X11-Clients mit dem Wayland-Compositor kommunizieren können. Dies ist notwendig, da nicht alle Anwendungen native Wayland-Unterstützung bieten.15|
|**Model Context Protocol (MCP) Integration (optional)**|Ermöglicht die strukturierte Interaktion von KI-Modellen (z.B. in einem KI-Assistenten) mit externen Daten und Diensten innerhalb von NovaDE.|MCP (JSON-RPC basiert)|Ein MCP-Host (z.B. ein KI-Assistent in NovaDE) kommuniziert mit MCP-Servern (Dienste, die Werkzeuge/Ressourcen bereitstellen). Ermöglicht KI-gesteuerte Aktionen wie Kalenderverwaltung oder sicheren Datenzugriff.17 Erfordert strenge Sicherheitsmaßnahmen.19|

### 2.3. Technologiestack

Der Technologiestack von NovaDE ist sorgfältig ausgewählt, um moderne Anforderungen an Leistung, Sicherheit und Entwicklereffizienz zu erfüllen.

- **Programmiersprache:** Rust wird als primäre Sprache für die Entwicklung von Kernkomponenten eingesetzt. Die Stärken von Rust in Bezug auf Speichersicherheit ohne Garbage Collector, Nebenläufigkeit und Performance machen es zu einer idealen Wahl für Systemsoftware wie eine Desktop-Umgebung. Die Rust API Guidelines 21 und der Rust Style Guide 22 sind maßgeblich für die Codeentwicklung.
- **Display Server Protokoll:** Wayland 5 ist das Fundament für die Display-Server-Architektur. Es bietet im Vergleich zu X11 eine modernere, sicherere und effizientere Basis, indem es die Rolle des Display-Servers und des Compositors vereint und viele Altlasten von X11 vermeidet.5 Herausforderungen bei der Wayland-Adoption, wie Kompatibilität mit älteren Anwendungen und spezifische Funktionen (z.B. Screen-Sharing, Drag-and-Drop), werden durch XWayland und Protokollerweiterungen sowie Bibliotheken wie PipeWire adressiert.15
- **Compositor-Bibliothek:** Smithay 5 dient als Bibliothek für die Entwicklung des Wayland-Compositors in Rust. Smithay stellt Bausteine für Wayland-Compositoren bereit und zielt auf Sicherheit, Modularität und eine High-Level-API ab.24 Da Smithay eine Bibliothek und kein Framework ist, bietet es hohe Flexibilität, erfordert aber auch mehr Eigenverantwortung bei der Implementierung der Compositor-Logik. Die Dokumentation und Community-Unterstützung von Smithay sind wichtige Ressourcen.24 Aktuelle Herausforderungen und Entwicklungen in Smithay (z.B. bezüglich Input-Handling, XWayland-Integration, Puffer-Management) müssen kontinuierlich beobachtet werden.26
- **UI Toolkit:** GTK4 9 wird für die Entwicklung der Desktop-Shell und anderer grafischer Anwendungen innerhalb von NovaDE verwendet. GTK4 bietet moderne Rendering-Fähigkeiten, CSS-basiertes Theming 27 und gute Integration mit Wayland. Die Rust-Bindings (`gtk4-rs`) 7 ermöglichen eine idiomatische Entwicklung in Rust. Die Architektur von GTK4-Anwendungen, insbesondere die Nutzung von `GtkApplication` 7, fördert eine strukturierte Anwendungsentwicklung.
- **Interprozesskommunikation (IPC):** D-Bus 8 ist der Standardmechanismus für die IPC zwischen NovaDE-Komponenten und zwischen Anwendungen und dem System. D-Bus ermöglicht es Diensten, Methoden, Signale und Eigenschaften bereitzustellen. Design-Richtlinien für D-Bus APIs, wie die Minimierung von Roundtrips und die Nutzung des Typsystems, sind für Performance und Robustheit entscheidend.30 Sicherheitsaspekte, insbesondere auf dem System-Bus, müssen berücksichtigt werden.30
- **Multimedia-Framework:** PipeWire 12 wird für das Management von Audio- und Videoströmen eingesetzt. PipeWire bietet geringe Latenz, flexible Graphen-basierte Verarbeitung und Kompatibilität mit PulseAudio-, JACK- und ALSA-Anwendungen.12 WirePlumber dient als Session-Manager für PipeWire und handhabt die Verbindungslogik und Profilverwaltung.13 Die Konfiguration von PipeWire erfolgt über Dateien und D-Bus, und es integriert sich mit XDG Portals für Screen-Sharing.12
- **Build-System:** Meson wird in Kombination mit Cargo für das Bauen von Rust-Projekten und deren Integration mit C-Bibliotheken (wie GTK) verwendet.34 Meson handhabt Abhängigkeiten von Nicht-Rust-Komponenten und die Installation von systemweiten Dateien, während Cargo die Rust-spezifischen Bauprozesse steuert. Es ist wichtig, `extern crate` zu vermeiden und stattdessen Abhängigkeiten in Meson zu deklarieren, um eine korrekte Nachverfolgung zu gewährleisten.35 Meson generiert auch `rust-project.json` für eine bessere IDE-Integration mit `rust-analyzer`.35
- **Versionskontrolle:** Git wird für die Versionskontrolle des gesamten NovaDE-Quellcodes verwendet.

### 2.4. Architekturdiagramm

Ein detailliertes Architekturdiagramm, das die Kernkomponenten, ihre Beziehungen und die wichtigsten Kommunikationspfade visualisiert, ist für das Verständnis des Systems unerlässlich. Dieses Diagramm sollte die Interaktionen über Wayland, D-Bus und PipeWire klar darstellen. Die Verwendung von Modellen wie dem C4-Modell (Context, Containers, Components, Code) kann hierbei hilfreich sein, um verschiedene Abstraktionsebenen darzustellen.36 Softwarearchitektur-Dokumentation nutzt oft Diagramme (z.B. UML) und textuelle Beschreibungen, um das Systemdesign zu erklären.3

_(An dieser Stelle würde ein Diagramm eingefügt werden, das die oben beschriebenen Komponenten und ihre Interaktionen zeigt. Da dies textbasiert ist, wird das Diagramm hier nicht visuell dargestellt, sondern seine Erstellung als Teil der Spezifikation gefordert.)_

Das Diagramm sollte folgende Elemente umfassen:

- **Kontext-Diagramm (Level 1):** Zeigt NovaDE im Kontext des Betriebssystems und der Benutzer.
- **Container-Diagramm (Level 2):** Zeigt die wichtigsten laufenden Prozesse/Dienste von NovaDE (Compositor, Shell, Session Manager etc.) und deren Hauptkommunikationswege (Wayland, D-Bus, PipeWire).
- **Komponenten-Diagramm (Level 3):** Detailliert die internen Hauptmodule innerhalb eines Containers, z.B. die Module des Compositors oder der Shell.

Diese visuelle Darstellung ergänzt die textuellen Beschreibungen und erleichtert das schnelle Erfassen der Systemstruktur.3

## 3. Detaillierte Komponentenspezifikationen

Dieser Abschnitt geht detailliert auf die Spezifikationen der einzelnen Kernkomponenten von NovaDE ein, die in Abschnitt 2.2 identifiziert wurden. Für jede Komponente werden ihre spezifischen Verantwortlichkeiten, Schnittstellen, technischen Anforderungen und Designüberlegungen beschrieben.

### 3.1. Wayland Compositor (Smithay-basiert)

Der NovaDE Wayland Compositor ist die zentrale Komponente für die grafische Darstellung und Interaktion. Er basiert auf der Smithay-Bibliothek.5

- **Verantwortlichkeiten:**
    
    - Implementierung des Wayland-Server-Protokolls und relevanter Erweiterungen (z.B. `xdg-shell`, `xdg-decoration`, `layer-shell`, `input-method`, `screencopy`).
    - Fenstermanagement: Platzierung, Stapelung, Größenänderung, Minimierung, Maximierung von Fenstern.
    - Compositing: Zusammenführen der Inhalte verschiedener Fenster und Oberflächen zu einem finalen Bild für den Bildschirm. Dies beinhaltet Transparenz, Schatten und optionale Animationen/Effekte.
    - Eingabeverarbeitung: Empfang von Eingabeereignissen (Maus, Tastatur, Touch) vom Kernel (via `libinput`/`evdev`) und Weiterleitung an die entsprechenden Client-Fenster oder die Shell.5 Dies beinhaltet die Transformation von Bildschirmkoordinaten zu lokalen Fensterkoordinaten.6
    - Ausgabemanagement: Konfiguration von Monitoren (Auflösung, Bildwiederholrate, Position), Unterstützung für Multi-Monitor-Setups und Hot-Plugging.
    - Verwaltung von Oberflächenrollen (`wl_surface` roles), insbesondere der Subsurface-Rolle.25
    - Koordination mit XWayland für die Unterstützung von X11-Anwendungen.
- **Schnittstellen:**
    
    - **Wayland-Protokoll:** Kommunikation mit Wayland-Clients (Anwendungen, Shell).
    - **Kernel Mode Setting (KMS) / Direct Rendering Manager (DRM):** Direkte Interaktion mit der Grafikhardware für die Bildausgabe.5
    - **`libinput` (oder äquivalente Smithay-Integration):** Für die Verarbeitung von Eingabegeräteereignissen.
    - **D-Bus:** Für die Kommunikation mit anderen NovaDE-Diensten (z.B. Settings Daemon für Theme-Änderungen, Power Manager für Helligkeitssteuerung).
- **Technische Anforderungen und Designüberlegungen:**
    
    - **Performance:** Geringe Latenz bei Eingabe und Darstellung ist kritisch. Effiziente Nutzung von GPU-Ressourcen durch EGL und OpenGL ES / Vulkan. Zero-Copy-Mechanismen, wo möglich, um die CPU-Auslastung zu minimieren.12
    - **Stabilität und Sicherheit:** Als Kernkomponente muss der Compositor extrem robust sein. Die Speichersicherheit von Rust trägt hierzu bei. Sicherheitslücken im X11-Protokoll sollen durch Waylands Design vermieden werden.5
    - **Modularität (Smithay-Ansatz):** Smithay ist eine Bibliothek, kein Framework.24 Dies gibt NovaDE-Entwicklern volle Kontrolle über die Compositor-Architektur und -Logik, bedeutet aber auch mehr Verantwortung für die Implementierung von Kernfunktionalitäten. Die Auswahl und Integration von Smithay-Modulen (z.B. für `xdg-shell`, Client-Handling, Rendering) muss sorgfältig erfolgen. Die Dokumentation von Smithay (`docs.rs` und für den Master-Branch) ist eine wichtige Ressource.24
    - **Atomare Updates:** Wayland ist darauf ausgelegt, atomare Updates zu ermöglichen, um Tearing und inkonsistente Fensterzustände zu vermeiden.5 Dies muss im Compositor korrekt implementiert werden, insbesondere bei der Anwendung von Zustandsänderungen von Oberflächen und Subsurfaces.25 Smithay's `CompositorState` und `SurfaceData` helfen bei der kohärenten Verwaltung von Oberflächenzuständen und der Anwendung von Double-Buffered State.25
    - **Damage Tracking:** Effizientes Damage Tracking ist notwendig, um nur die Teile des Bildschirms neu zu zeichnen, die sich geändert haben. Dies ist entscheidend für die Performance und Energieeffizienz.
    - **XWayland-Integration:** Nahtlose und performante Integration von XWayland ist für die Abwärtskompatibilität unerlässlich. Herausforderungen wie Tastatur-Grabs 26 oder spezifische Fenstergeometrien 26 müssen adressiert werden.
    - **Unterstützung für Protokollerweiterungen:** Eine klare Strategie für die Unterstützung und Implementierung neuer und optionaler Wayland-Protokollerweiterungen ist notwendig, um mit der Entwicklung des Wayland-Ökosystems Schritt zu halten.
    - **Fehlerbehandlung:** Robuste Fehlerbehandlung bei ungültigen Client-Anfragen oder internen Fehlern. Smithay bietet Mechanismen zur Erkennung von Problemen wie ungültige Fenstergeometrien oder Rollen-Neuzuweisungen vor der Zerstörung.26
    - **Konfigurierbarkeit:** Bereitstellung von Optionen zur Anpassung des Compositor-Verhaltens (z.B. Animationen, Effekte) über den Settings Daemon.

Die Entwicklung eines Wayland-Compositors mit Smithay erfordert ein tiefes Verständnis des Wayland-Protokolls und der Linux-Grafik- und Eingabe-Subsysteme. Die aktive Community und die Issue-Tracker von Smithay 26 sind wichtige Quellen für Problemlösungen und Best Practices.

### 3.2. Desktop Shell (GTK4-basiert)

Die NovaDE Shell ist die primäre Schnittstelle für den Benutzer und basiert auf GTK4 und Rust.

- **Verantwortlichkeiten:**
    
    - Bereitstellung und Verwaltung von UI-Elementen wie Panel(s), Anwendungsstarter (Launcher), Task-Manager, System-Tray-Bereich, Desktop-Hintergrund und Desktop-Widgets.
    - Starten von Anwendungen und Verwalten laufender Anwendungsfenster (in Koordination mit dem Compositor).
    - Anzeige von Systemstatusinformationen (Netzwerk, Lautstärke, Akku etc.).
    - Integration mit dem Notification Service zur Anzeige von Benachrichtigungen.
    - Verwaltung von Arbeitsbereichen/virtuellen Desktops.
    - Bereitstellung von Suchfunktionen (Anwendungen, Dateien, Einstellungen).
    - Integration von `GtkApplication` für Menüleisten und automatische Ressourcenladung (z.B. Icons, Menüdefinitionen aus UI-Dateien).7
- **Schnittstellen:**
    
    - **Wayland-Protokoll:** Als Wayland-Client kommuniziert die Shell mit dem NovaDE Compositor, um ihre UI-Elemente als Wayland-Oberflächen (z.B. mittels `layer-shell` für Panels) darzustellen und Eingaben zu empfangen.
    - **D-Bus:** Kommunikation mit dem Session Manager (z.B. für Logout/Shutdown-Aktionen), Settings Daemon (für Theme- und Einstellungsänderungen), Notification Service, Power Manager und anderen Systemdiensten.
    - **GTK4 API:** Für die Erstellung und Verwaltung der UI-Elemente.
    - **PipeWire:** Indirekt über Portale oder Bibliotheken für Funktionen wie Lautstärkeregelung oder Mediensteuerung.
- **Technische Anforderungen und Designüberlegungen:**
    
    - **Performance und Reaktionsfähigkeit:** Die Shell muss auch bei hoher Systemlast flüssig und reaktionsschnell bleiben. Effiziente Nutzung von GTK4-Rendering und Minimierung von blockierenden Operationen im Hauptthread.
    - **Anpassbarkeit und Theming:** Umfangreiche Anpassungsmöglichkeiten für den Benutzer (Position und Inhalt von Panels, Themes, Icons, Schriftarten). GTK4-CSS wird für das Theming verwendet.27 Die Shell muss auf Änderungen der GTK-Einstellungen (z.B. `gtk-application-prefer-dark-theme` 9) reagieren.
    - **Modularität:** Einzelne Shell-Komponenten (z.B. Panel, Launcher, Widgets) sollten möglichst modular und austauschbar gestaltet sein, um Erweiterbarkeit und alternative Implementierungen zu ermöglichen.
    - **Barrierefreiheit (Accessibility):** Einhaltung von Standards für Barrierefreiheit unter Verwendung der AT-Context-APIs von GTK.28
    - **Multi-Monitor-Unterstützung:** Korrekte Darstellung und Verwaltung von Shell-Elementen auf mehreren Monitoren mit unterschiedlichen Auflösungen und DPI-Einstellungen.
    - **Zustandsverwaltung:** Effiziente Verwaltung des Zustands der Shell (z.B. geöffnete Menüs, aktive Widgets). Rust-Muster für die Zustandsverwaltung (z.B. Verwendung von `Arc<RwLock<T>>` oder `Rc<RefCell<T>>` je nach Threading-Modell 37, oder dedizierte State-Management-Bibliotheken wie `stateflow` 38 bei komplexen Zustandsautomaten) sind relevant.
    - **Integration mit `GtkApplication`:** Nutzung von `GtkApplication` für die Hauptanwendungslogik der Shell, um von dessen Features wie Anwendungs-Uniqueness, Session-Management-Integration und automatischer Ressourcenladung (Menüs, Icons) zu profitieren.7 Menüs können über `gtk/menus.ui` geladen und über `menu_by_id()` dynamisch manipuliert werden.7
    - **Icon-Handling:** Nutzung des `GtkIconTheme` und automatisches Hinzufügen von Ressourcenpfaden für anwendungsspezifische Icons.7

Die Entwicklung der Shell erfordert ein gutes Verständnis von GTK4, ereignisgesteuerter Programmierung und der Interaktion mit dem Wayland-Compositor und D-Bus-Diensten.

### 3.3. Session Manager

Der NovaDE Session Manager ist verantwortlich für den Lebenszyklus der Benutzersitzung.

- **Verantwortlichkeiten:**
    
    - Benutzerauthentifizierung (typischerweise delegiert an PAM oder einen Display Manager).
    - Starten der Kernkomponenten von NovaDE beim Login (Compositor, Shell, Settings Daemon, etc.).
    - Verwaltung von Umgebungsvariablen für die Sitzung.
    - Behandlung von Sitzungsereignissen wie Sperren, Abmelden, Herunterfahren, Neustarten.
    - Koordination mit `systemd-logind` (falls vorhanden) für Sitzungsmanagement und Hardware-Zugriffsrechte.
    - Bereitstellung einer D-Bus-Schnittstelle für andere Komponenten, um Sitzungsaktionen auszulösen (z.B. Logout-Button in der Shell).
- **Schnittstellen:**
    
    - **PAM (Pluggable Authentication Modules):** Für die Benutzerauthentifizierung.
    - **D-Bus:** Kommunikation mit `systemd-logind` und Bereitstellung einer eigenen D-Bus-Schnittstelle für Sitzungssteuerung. Andere NovaDE-Komponenten (z.B. Shell, Power Manager) interagieren über D-Bus mit dem Session Manager.8
    - **Prozessmanagement:** Starten und Überwachen der Kernprozesse der Desktop-Umgebung.
- **Technische Anforderungen und Designüberlegungen:**
    
    - **Sicherheit:** Sichere Handhabung von Authentifizierungsdaten und Sitzungsinformationen.
    - **Robustheit:** Zuverlässiges Starten und Beenden der Desktop-Umgebung. Fehler beim Start einzelner Komponenten müssen ggf. abgefangen und behandelt werden.
    - **Integration mit Systemdiensten:** Enge Integration mit `systemd-logind` oder alternativen Mechanismen zur Verwaltung von Sitzungen und Gerätezugriff (z.B. für DRM/KMS). PipeWire erfordert beispielsweise eine korrekte Sitzungseinrichtung für Geräte-ACLs.12
    - **Konfigurierbarkeit:** Möglichkeit zur Konfiguration von Autostart-Anwendungen und Sitzungsparametern.
    - **Schneller Start:** Optimierung der Startsequenz für ein schnelles Hochfahren der Desktop-Umgebung.

### 3.4. Settings Daemon und Konfigurationsmanagement

Der Settings Daemon zentralisiert die Verwaltung von System- und Benutzereinstellungen.

- **Verantwortlichkeiten:**
    
    - Bereitstellung einer zentralen Anlaufstelle für das Lesen und Schreiben von Konfigurationseinstellungen.
    - Verwaltung von Einstellungen für Erscheinungsbild (Theme, Icons, Schriftarten, Hintergrund), Hardware (Monitore, Eingabegeräte, Energieoptionen) und Verhalten der Desktop-Umgebung.
    - Benachrichtigung von interessierten Anwendungen und Komponenten über Einstellungsänderungen.
    - Laden von Standardeinstellungen und Zusammenführen mit benutzerspezifischen Anpassungen.
    - GTK-Einstellungen: Der Daemon ist verantwortlich für die Bereitstellung von GTK-spezifischen Einstellungen (z.B. `gtk-theme-name`, `gtk-font-name`, `gtk-application-prefer-dark-theme`). Diese werden typischerweise über einen XSettings-Manager (unter X11) oder einen Settings-Portal/DConf (unter Wayland) geteilt.9 GTK liest auch Standardwerte aus `settings.ini`-Dateien.9
- **Schnittstellen:**
    
    - **D-Bus:** Hauptschnittstelle für Anwendungen und Shell-Komponenten, um Einstellungen abzufragen und (ggf. mit entsprechenden Rechten) zu ändern. Signale werden verwendet, um über Änderungen zu informieren.
    - **Konfigurations-Backends:** Interaktion mit Speichermechanismen für Einstellungen (z.B. GSettings/DConf, INI-Dateien, XML-Dateien).
    - **XSettings-Protokoll (optional, für XWayland-Kompatibilität):** Falls erforderlich, um Einstellungen für X11-Anwendungen bereitzustellen.
- **Technische Anforderungen und Designüberlegungen:**
    
    - **Konsistenz:** Sicherstellung, dass alle Komponenten auf einen konsistenten Satz von Einstellungen zugreifen.
    - **Performance:** Schneller Zugriff auf Einstellungen und effiziente Benachrichtigung bei Änderungen. Caching-Mechanismen können hier sinnvoll sein.39
    - **Granularität:** Feingranulare Einstellungsoptionen, um eine detaillierte Anpassung durch den Benutzer zu ermöglichen.
    - **Schema-Definition:** Ein klares Schema für alle Konfigurationsparameter, inklusive Datentypen, erlaubter Werte und Standardwerte (siehe Tabelle 4 im Anhang).
    - **Migration:** Strategien für die Migration von Einstellungen bei Updates der Desktop-Umgebung.
    - **Sicherheit:** Zugriffskontrolle für das Ändern systemweiter oder sicherheitsrelevanter Einstellungen.
    - **Global Settings Management:** Die Verwaltung globaler Einstellungen ist ein kritischer Aspekt für die Stabilität und Konsistenz großer Anwendungen.39 Eine zentrale Konfigurationsverwaltung reduziert das Risiko von Ausfällen und Sicherheitslücken durch Nachverfolgbarkeit von Änderungen und Durchsetzung von Richtlinien.40

Eine Herausforderung beim Konfigurationsmanagement ist die Balance zwischen Flexibilität für den Benutzer und der Komplexität der Verwaltung. Eine klare Struktur und gute Werkzeuge zur Konfiguration sind entscheidend.

### 3.5. Notification Service

Der Notification Service ist für die Anzeige von Desktop-Benachrichtigungen zuständig.

- **Verantwortlichkeiten:**
    
    - Implementierung der Freedesktop Desktop Notifications Specification.
    - Empfang von Benachrichtigungsanfragen von Anwendungen und Systemdiensten.
    - Verwaltung einer Warteschlange für Benachrichtigungen.
    - Anzeige von Benachrichtigungen als Pop-ups oder in einem Benachrichtigungscenter, gemäß den Benutzereinstellungen und dem aktuellen Systemzustand (z.B. "Nicht stören"-Modus).
    - Unterstützung für Aktionen in Benachrichtigungen.
    - Persistenz von Benachrichtigungen (optional, für ein Benachrichtigungscenter).
- **Schnittstellen:**
    
    - **D-Bus:** Empfängt Benachrichtigungsanfragen über die standardisierte D-Bus-Schnittstelle (`org.freedesktop.Notifications`).
    - **NovaDE Shell:** Interagiert mit der Shell zur Darstellung der Benachrichtigungs-UI.
    - **Settings Daemon:** Abfrage von Benutzereinstellungen bezüglich Benachrichtigungen (z.B. Position, Timeout, "Nicht stören"-Modus).
- **Technische Anforderungen und Designüberlegungen:**
    
    - **Zuverlässigkeit:** Benachrichtigungen müssen zuverlässig zugestellt und angezeigt werden.41
    - **Performance:** Der Dienst darf das System nicht übermäßig belasten, auch bei vielen eingehenden Benachrichtigungen.
    - **Anpassbarkeit:** Benutzer sollten das Aussehen und Verhalten von Benachrichtigungen anpassen können.
    - **Regel-Engine (optional):** Eine erweiterte Implementierung könnte eine Regel-Engine enthalten, um Benachrichtigungen basierend auf Quelle, Inhalt oder Systemzustand unterschiedlich zu behandeln (z.B. Priorisierung, Stummschaltung).42 Eine solche Engine würde aus einer Sammlung von Regeln, einem Eingabemechanismus, Trigger-Bedingungen und Aktionen bestehen.42
    - **Rate Limiting und Queuing:** Um eine Überlastung des Systems oder des Benutzers zu vermeiden, sollten Mechanismen für Rate Limiting und Queuing implementiert werden.41
    - **Logging:** Detailliertes Logging zur Fehlerbehebung und Nachverfolgung von Benachrichtigungsflüssen.41
    - **Datenbankschema (für Persistenz):** Falls Benachrichtigungen persistiert werden, ist ein gut designtes Datenbankschema für Benutzerpräferenzen, Anfragen und Zustellstatus erforderlich.41

Die Architektur eines Benachrichtigungssystems umfasst typischerweise einen Client (die Anwendung, die die Benachrichtigung sendet), einen Notification Server (der die Anfragen verarbeitet und in die Warteschlange stellt) und einen Notification Executor (der die Benachrichtigungen über die entsprechenden Kanäle zustellt).44

### 3.6. Weitere Systemdienste (z.B. Power Management, Workspace Management)

Neben den oben genannten Hauptkomponenten können weitere spezialisierte Systemdienste existieren.

- **Power Manager:**
    
    - **Verantwortlichkeiten:** Überwachung des Batteriestatus, Umschaltung zwischen Energieprofilen, Initiierung von Suspend/Hibernate, Steuerung der Bildschirmhelligkeit, Reaktion auf Deckel-Schließen-Events bei Laptops.
    - **Schnittstellen:** D-Bus (Interaktion mit `UPower`, `systemd-logind`, Settings Daemon, Shell).
    - **Anforderungen:** Geringer Ressourcenverbrauch, zuverlässige Reaktion auf Energieereignisse.
- **Workspace Manager:**
    
    - **Verantwortlichkeiten:** Logik für die Verwaltung von virtuellen Desktops/Arbeitsbereichen, Verschieben von Fenstern zwischen Arbeitsbereichen, Speichern und Wiederherstellen von Fensteranordnungen pro Arbeitsbereich (optional). Die Implementierung ist oft eng mit dem Compositor und der Shell verwoben.
    - **Schnittstellen:** Interne APIs mit Compositor und Shell, D-Bus für die Steuerung durch externe Tools oder Skripte.
    - **Anforderungen:** Effiziente Verwaltung von Fensterzuständen, intuitive Bedienung. Die Komplexität der Workspace-Verwaltung kann mit der von Projektmanagement-Tools verglichen werden, bei denen es darum geht, Aufgaben (Fenster) in verschiedenen Kontexten (Arbeitsbereichen) zu organisieren und den Überblick zu behalten.10 Funktionen wie das "Verpacken" von Fenstern in Ordner-ähnliche Strukturen oder das Speichern und Abrufen von Layouts (Screensets) sind denkbare Erweiterungen.10
- **Model Context Protocol (MCP) Host (optional, falls KI-Funktionen integriert werden):**
    
    - **Verantwortlichkeiten:** Dient als Container oder Koordinator für MCP-Client-Instanzen innerhalb von NovaDE (z.B. ein KI-Assistent). Verwaltet den Lebenszyklus, Sicherheitspolicies (Berechtigungen, Benutzerautorisierung, Zustimmung) und die Interaktion zwischen KI-Modellen und MCP-Servern (externe Werkzeuge und Datenquellen).17
    - **Schnittstellen:** MCP (JSON-RPC über lokale Prozesse oder HTTP-Streams), interne APIs mit der KI-Anwendung, D-Bus für die Systemintegration.
    - **Anforderungen:** Starke Authentifizierung zwischen Host, Client und Servern, explizite und bereichsbezogene Autorisierung, Eingabevalidierung und Ausgabesanitisierung, Ratenbegrenzung, Überwachung und Logging.19 Die Sicherheit ist hier von größter Bedeutung, da MCP den Zugriff auf sensible Daten und Systemfunktionen ermöglichen kann.19 Ein kompromittierter MCP-Server könnte Daten stehlen oder Befehle manipulieren.19

Die Notwendigkeit und das Design weiterer Dienste ergeben sich aus den spezifischen Anforderungen der Nutzererfahrung und den geplanten Features von NovaDE.

## 4. Nicht-funktionale Anforderungen (NFRs)

Nicht-funktionale Anforderungen definieren, _wie_ das System seine Funktionen ausführt, und sind entscheidend für die Qualität und Akzeptanz der Software.45 Sie müssen spezifisch, messbar, erreichbar, relevant und zeitgebunden (SMART) formuliert werden. Die Vernachlässigung von NFRs kann dazu führen, dass eine Software trotz voller Funktionalität die Erwartungen der Nutzer nicht erfüllt.46

### 4.1. Performance

- **Reaktionszeiten:**
    - Start der Desktop-Umgebung (vom Login bis zur vollen Nutzbarkeit): <5 Sekunden auf Zielhardware.
    - Start typischer Anwendungen (Texteditor, Browser): <2 Sekunden.
    - Öffnen des Anwendungsstarters: <300 ms.
    - Fensteroperationen (Öffnen, Schließen, Minimieren, Verschieben, Größenänderung): Visuell unmittelbar, Latenz <50 ms.
    - Shell-Animationen und -Effekte: Mindestens 60 FPS, um Flüssigkeit zu gewährleisten.
- **Ressourcennutzung:**
    - Leerlauf-CPU-Auslastung des Desktops (ohne laufende Anwendungen): <5% auf einem einzelnen Kern der Zielhardware.
    - RAM-Nutzung der Kern-Desktop-Prozesse (Compositor, Shell, Session Manager) im Leerlauf: <512 MB insgesamt.
    - Die Performance-Anforderungen müssen unter Berücksichtigung der aktuellen Systemlast spezifiziert werden.45
- **Datendurchsatz (falls relevant, z.B. für Dateioperationen im Dateimanager):** Spezifische Metriken je nach Anwendungsfall.
- **PipeWire Latenz:** Für Audio-Anwendungen ist eine geringe Latenz wichtig. PipeWire ermöglicht die Konfiguration von Quanten und Raten, um die Latenz zu beeinflussen (z.B. 11 ms für Bluetooth-Headsets, 90 ms für Video-Apps bei 48 kHz).12 Die Latenz wird als Verhältnis von Quantum zu Rate berechnet.12

### 4.2. Stabilität und Zuverlässigkeit

- **Mean Time Between Failures (MTBF):** Für Kernkomponenten (Compositor, Shell) wird ein MTBF von >1000 Betriebsstunden angestrebt.
- **Fehlertoleranz:** Absturz einer einzelnen Anwendung darf nicht zur Instabilität des gesamten Desktops führen. Der Compositor muss robust gegenüber fehlerhaften Clients sein.
- **Wiederherstellbarkeit:** Im Falle eines Absturzes einer Kernkomponente (z.B. Shell) soll ein automatischer Neustart ohne Datenverlust der laufenden Anwendungen (soweit möglich) erfolgen.
- **Verfügbarkeit:** Die Verfügbarkeit des Systems sollte hoch sein, z.B. 99.9%.46 Dies muss unter Berücksichtigung der finanziellen und kritischen Auswirkungen von Ausfallzeiten definiert werden.45

### 4.3. Sicherheit

- **Zugriffskontrolle:** Strikte Trennung von Benutzerprivilegien und Systemprivilegien. Prozesse laufen mit minimal notwendigen Rechten (Principle of Least Privilege).
- **Datenschutz:** Schutz sensibler Benutzerdaten (Passwörter, private Dateien) vor unbefugtem Zugriff. Einhaltung relevanter Datenschutzbestimmungen (z.B. DSGVO).
- **Wayland-Sicherheitsmodell:** Wayland ist inhärent sicherer als X11, da Clients isolierter sind und keinen direkten Zugriff auf die Eingaben oder Fensterinhalte anderer Clients haben.5 Der Compositor agiert als Gatekeeper.
- **D-Bus-Sicherheit:** Der System-Bus von D-Bus verfügt über Sicherheitsmechanismen, die einschränken, welche Prozesse Namen besitzen oder Methodenaufrufe senden dürfen.30 Session-Busse gelten typischerweise nicht als Sicherheitsgrenze innerhalb der Benutzersitzung.30
- **Sicherheitsupdates:** Ein klar definierter Prozess für die schnelle Bereitstellung von Sicherheitsupdates.
- **Schutz vor Angriffen:** Maßnahmen zum Schutz vor gängigen Angriffsvektoren (z.B. SQL-Injection, XSS bei Web-basierten Komponenten – falls vorhanden, Code Injection durch unsichere Plugin-Schnittstellen).
- **MCP-Sicherheit (falls implementiert):** Starke Authentifizierung, explizite Autorisierung, Eingabevalidierung, Ausgabesanitisierung, Ratenbegrenzung und Sandboxing sind für MCP-Komponenten unerlässlich.17 Die Verwendung von nicht überprüften MCP-Paketen stellt ein Risiko dar.20

### 4.4. Skalierbarkeit

- **Anzahl gleichzeitiger Anwendungen:** Das System muss auch bei einer großen Anzahl (>50) gleichzeitig laufender Anwendungen performant und stabil bleiben.
- **Multi-Monitor-Unterstützung:** Unterstützung für mindestens 4 Monitore mit unterschiedlichen Auflösungen und DPI-Einstellungen ohne signifikanten Performanceverlust.
- **Anpassung an Hardware:** Das System soll auf einer breiten Palette von Hardware, von Low-End-Laptops bis zu High-End-Workstations, zufriedenstellend laufen. Dies kann unterschiedliche Konfigurationsprofile erfordern.
- **PipeWire Skalierbarkeit:** PipeWire ist darauf ausgelegt, eine große Anzahl von Multimedia-Streams effizient zu verwalten.12

### 4.5. Wartbarkeit

- **Code-Komplexität:** Reduzierung der Code-Komplexität durch modulare Bauweise und klare Schnittstellen.47 Verwendung von Code-Qualitäts-Tools (Linters, statische Analyse).
- **Dokumentation:** Umfassende und aktuelle Entwicklerdokumentation (API-Referenzen, Architektur-Beschreibungen).1
- **Testbarkeit:** Hohe Testabdeckung durch Unit-, Integrations- und UI-Tests.2
- **Logging und Debugging:** Aussagekräftige Log-Meldungen und gute Debugging-Unterstützung.
- **Standardisierte Formate und Prozesse:** Einhaltung von Coding-Standards und Entwicklungsprozessen.1
- Die Wartbarkeit wird durch die Lesbarkeit und Verständlichkeit des Codes beeinflusst. Klare Namenskonventionen und eine konsistente Code-Struktur sind hierfür wichtig.48

### 4.6. Benutzbarkeit (Usability)

Obwohl primär durch UX-Spezifikationen definiert, hat die Benutzbarkeit auch technische Implikationen:

- **Konsistenz:** Einheitliches Erscheinungsbild und Verhalten über alle NovaDE-Komponenten hinweg.
- **Barrierefreiheit (Accessibility):** Einhaltung von a11y-Standards (z.B. WCAG für Web-Inhalte, entsprechende GTK-Richtlinien). Wayland selbst hat Herausforderungen im Bereich Barrierefreiheit, die durch Protokollerweiterungen und Toolkit-Unterstützung adressiert werden müssen.23
- **Internationalisierung (i18n) und Lokalisierung (l10n):** Unterstützung für verschiedene Sprachen und regionale Formate.
- **Konfigurierbarkeit:** Einfache und verständliche Anpassungsmöglichkeiten für den Benutzer.
- Messbare Usability-Ziele sollten auf Basis von Benutzertests (ggf. mit Prototypen oder Konkurrenzprodukten) festgelegt werden.45

### 4.7. Kompatibilität

- **Anwendungskompatibilität:**
    - Native Wayland-Anwendungen (GTK, Qt): Vollständige Unterstützung.
    - X11-Anwendungen: Unterstützung über XWayland.15 Es muss sichergestellt werden, dass gängige X11-Anwendungen (Browser, Office-Suiten, Spiele) unter XWayland zufriedenstellend funktionieren. Bekannte Probleme mit XWayland (z.B. bei spezifischen Anwendungen wie Krita, Discord, OBS 15) müssen adressiert oder dokumentiert werden.
    - Flatpak/Snap-Anwendungen: Integration über XDG Desktop Portals für den Zugriff auf Systemressourcen.6
- **Hardwarekompatibilität:** Unterstützung für gängige Grafikchips (Intel, AMD, NVIDIA – letztere oft mit proprietären Treibern und spezifischen Herausforderungen unter Wayland), Eingabegeräte und Peripherie.
- **Freedesktop.org-Standards:** Einhaltung relevanter Standards für Interoperabilität (z.B. Desktop Entry Specification, Icon Theme Specification, MIME-Type System, Notification Specification).

Die Dokumentation von NFRs sollte in Zusammenarbeit mit verschiedenen Stakeholdern erfolgen und NFRs sollten mit Geschäftszielen verknüpft werden.45 Es ist wichtig, Einschränkungen durch Drittanbieter-APIs oder bestehende Architekturen zu berücksichtigen.45 Eine separate Verfolgung von NFRs und regelmäßige Überprüfung wird empfohlen.46

## 5. Schnittstellenspezifikationen

Dieser Abschnitt definiert die wichtigsten internen und externen Schnittstellen von NovaDE. Eine klare Definition dieser Schnittstellen ist entscheidend für die modulare Entwicklung und die Interoperabilität der Komponenten.

### 5.1. Interne APIs (zwischen NovaDE-Komponenten)

- **D-Bus Schnittstellen:**
    
    - **Definition:** Jede NovaDE-Komponente, die Dienste für andere Komponenten bereitstellt (z.B. Settings Daemon, Session Manager, Notification Service), exponiert eine oder mehrere D-Bus-Schnittstellen. Diese Schnittstellen definieren Objekte, Methoden, Signale und Eigenschaften.8
    - **Designrichtlinien:**
        - Verwendung sprechender Namen für Dienste, Pfade, Schnittstellen, Methoden und Signale (z.B. `org.novade.Settings`, `/org/novade/Settings`, `org.novade.Settings.Interface`, `GetSetting`, `SettingChanged`). Versionierung sollte in Namen inkludiert werden, um API-inkompatible Änderungen zu handhaben (z.B. `org.novade.MyService1`).30
        - Minimierung der Anzahl von Roundtrips durch gut gestaltete Methoden (z.B. Rückgabe mehrerer verwandter Werte in einer Methode statt mehrerer einzelner Aufrufe).30
        - Nutzung des D-Bus-Typsystems: Strukturierte Daten (Structs, Arrays, Dictionaries) sollten nativen D-Bus-Typen anstelle von serialisierten Strings vorgezogen werden. Enumerierte Werte als Unsigned Integers übertragen, um String-Parsing zu vermeiden und die Nachrichten kompakter zu halten.30
        - Klare Dokumentation jeder Schnittstelle, ihrer Methoden, Argumente, Rückgabewerte und Signale. Die Bedeutung von Enum-Werten muss dokumentiert werden.30
        - Verwendung von `org.freedesktop.DBus.Properties` für den Zugriff auf Eigenschaften, anstatt spezifischer Get/Set-Methoden für jede Eigenschaft.30
    - **Sicherheit:** D-Bus-Richtlinien (Policy-Dateien) definieren, welche Benutzer oder Prozesse auf welche Schnittstellen und Methoden zugreifen dürfen, insbesondere auf dem System-Bus.
    - **Beispiele:**
        - Session Manager: `org.novade.SessionManager.Logout()`, `org.novade.SessionManager.CanShutdownChanged (Signal)`.
        - Settings Daemon: `org.novade.Settings.GetValue(category, key)`, `org.novade.Settings.SetValue(category, key, value)`, `org.novade.Settings.ValueChanged (Signal)`.
- **Wayland Protokollerweiterungen (falls spezifisch für NovaDE):**
    
    - Falls NovaDE eigene, private Wayland-Protokollerweiterungen für die Kommunikation zwischen dem Compositor und der Shell (oder anderen internen Clients) benötigt, müssen diese hier spezifiziert werden.
    - **Definition:** XML-basierte Protokolldateien, die neue Interfaces, Requests, Events und Enums definieren.6
    - **Generierung:** C-Header und Glue-Code können mit `wayland-scanner` generiert werden.5 Für Rust werden entsprechende Generatoren oder manuelle Bindings benötigt.
    - **Stabilität:** Private Protokolle sollten als instabil betrachtet und klar als solche gekennzeichnet werden, um externe Abhängigkeiten zu vermeiden.
- **Interne Rust-Bibliotheks-APIs (Crates):**
    
    - NovaDE wird voraussichtlich aus mehreren Rust-Crates bestehen. Die öffentlichen APIs dieser Crates (Funktionen, Structs, Traits, Enums, Module) müssen klar definiert und dokumentiert sein (`rustdoc`).
    - **Richtlinien:** Einhaltung der offiziellen Rust API Guidelines.21 Dies beinhaltet Aspekte wie Namenskonventionen (C-CASE, C-CONV, C-GETTER, C-ITER), Implementierung gängiger Traits (C-COMMON-TRAITS wie `Debug`, `Clone`, `Send`, `Sync`), Fehlerbehandlung (C-GOOD-ERR), Dokumentation (C-CRATE-DOC, C-EXAMPLE) und Zukunftsfähigkeit (C-STRUCT-PRIVATE, C-SEALED).
    - **Modularität:** Crates sollten klar definierte Verantwortlichkeiten haben und lose gekoppelt sein.

### 5.2. Externe APIs (für Drittanbieter-Anwendungen)

- **Wayland Protokoll:**
    
    - NovaDE implementiert Standard-Wayland-Protokolle und -Erweiterungen (z.B. `xdg-shell`, `xdg-decoration`, `wl_output`, `wl_seat`). Anwendungen interagieren mit NovaDE primär über diese Protokolle.
    - Die unterstützten Protokolle und deren Versionen müssen dokumentiert werden.
    - Wayland ist ein asynchrones, objektorientiertes Protokoll. Clients fordern Dienste vom Compositor an, indem sie Methoden auf Objekten aufrufen. Der Compositor sendet Informationen an Clients, indem Objekte Ereignisse auslösen.6
- **D-Bus Schnittstellen (öffentlich):**
    
    - NovaDE kann ausgewählte D-Bus-Schnittstellen für Drittanbieter-Anwendungen bereitstellen (z.B. für Statusinformationen, Abfrage von Desktop-Funktionen).
    - Diese Schnittstellen müssen stabil, versioniert und gut dokumentiert sein.
    - Beispiel: Eine Schnittstelle, um die aktuelle Theme-Farbe oder den "Nicht stören"-Status abzufragen.
- **XDG Desktop Portals:**
    
    - Für sandboxed Anwendungen (Flatpak, Snap) ist die Interaktion über XDG Desktop Portals der bevorzugte Weg, um auf Systemressourcen zuzugreifen (Dateiauswahl, Öffnen von URIs, Drucken, Screen-Sharing etc.).
    - NovaDE muss die Backend-Implementierungen für die relevanten Portale bereitstellen (z.B. `xdg-desktop-portal-novade` oder Nutzung von generischen Backends wie `xdg-desktop-portal-gtk`). PipeWire integriert sich mit Portalen für Screen-Sharing.6
- **Freedesktop.org Standards:**
    
    - Anwendungen verlassen sich auf die Einhaltung von Freedesktop.org-Standards durch die Desktop-Umgebung für eine nahtlose Integration. Dazu gehören:
        - Desktop Entry Specification (`.desktop`-Dateien für Anwendungsmenüs).
        - Icon Theme Specification (Auffinden und Anzeigen von Icons).
        - MIME Applications Associations (Zuordnung von Dateitypen zu Anwendungen).
        - Desktop Notifications Specification (bereits durch den Notification Service abgedeckt).
        - Status Notifier Item Specification (für System-Tray-Icons, falls unterstützt).

### 5.3. Model Context Protocol (MCP) Schnittstellen (falls zutreffend)

Falls NovaDE KI-gestützte Funktionen integriert, die auf dem Model Context Protocol basieren, sind dessen Schnittstellen relevant.

- **MCP-Architektur:** MCP verwendet eine Client-Server-Architektur. Die KI-Anwendung (z.B. ein Desktop-Assistent) fungiert als Host und betreibt einen MCP-Client. Externe Integrationen (Tools, Datenquellen) laufen als MCP-Server.17
- **Kommunikation:** Erfolgt über standardisierte JSON-RPC-Nachrichten.17
    - Beispielhafte Methoden: `tools/list` (um verfügbare Werkzeuge aufzulisten), `tools/call` (um ein Werkzeug mit Parametern aufzurufen).17
- **MCP-Primitive:**
    - **Tools:** Vom Modell gesteuerte API-Aufrufe oder Operationen (z.B. Datei schreiben, Kalendereintrag erstellen).18
    - **Resources:** Von der Anwendung gesteuerte kontextuelle Daten (z.B. Dateiinhalte, Git-Historie).18
    - **Prompts:** Vom Benutzer gesteuerte Vorlagen oder Anweisungen (z.B. Slash-Befehle).18
- **Sicherheitsaspekte:** Da MCP den Zugriff auf externe Dienste und potenziell sensible Daten ermöglicht, sind strenge Sicherheitsmaßnahmen unerlässlich:
    - **Authentifizierung:** Starke Authentifizierung zwischen Host, Client und Server (z.B. API-Keys, Tokens, mTLS).19 Clients sollten nur mit vertrauenswürdigen, whitelisted Servern verbinden.19
    - **Autorisierung:** Explizite, bereichsbezogene Autorisierung nach dem Prinzip der geringsten Rechte.19
    - **Eingabevalidierung und Ausgabesanitisierung:** Alle über MCP fließenden Daten müssen als potenziell bösartig behandelt und validiert/sanitisiert werden, um Injection-Angriffe oder Verwirrung des KI-Modells zu verhindern.19
    - **Ratenbegrenzung und Ressourcenbeschränkungen:** Um Missbrauch oder Überlastung zu verhindern.19
    - **Überwachung und Logging:** Zur Nachverfolgung von Aktivitäten und Erkennung von Anomalien.19
    - Die Integration von MCP erhöht das Risiko von Supply-Chain-Angriffen und Compliance-Verletzungen, wenn nicht sorgfältig gehandhabt.20

Die Definition und Dokumentation dieser Schnittstellen muss klar, präzise und für die jeweilige Zielgruppe verständlich sein.1 Die Verwendung von API-Beschreibungssprachen (z.B. OpenAPI für HTTP-basierte D-Bus-Alternativen, falls vorhanden, oder XML für Wayland-Protokolle) kann die Konsistenz und Maschinenlesbarkeit verbessern.

## 6. Entwicklungsrichtlinien und -standards

Dieser Abschnitt legt die Richtlinien und Standards für den Entwicklungsprozess von NovaDE fest. Ziel ist es, eine hohe Codequalität, Wartbarkeit, Konsistenz und Effizienz im gesamten Projekt sicherzustellen. Die Einhaltung dieser Richtlinien ist für alle Mitwirkenden verbindlich.

### 6.1. Programmierrichtlinien (Rust)

Die Entwicklung von NovaDE-Komponenten in Rust folgt den offiziellen und Community-etablierten Best Practices.

- **Rust Edition:** Verwendung der neuesten stabilen Rust Edition (z.B. 2021 oder neuer), um von aktuellen Sprachfeatures und Verbesserungen zu profitieren. Für Meson-Projekte ist die Angabe `rust_std=2018` (oder neuer) in den `project(default_options)` wichtig, um `extern crate` zu vermeiden.35
- **Offizielle Rust API Guidelines:** Strikte Einhaltung der Rust API Guidelines.21 Dies umfasst:
    - **Naming Conventions (C-CASE, C-CONV, C-GETTER, C-ITER, C-ITER-TY, C-FEATURE, C-WORD-ORDER):** Konforme Benennung von Crates, Modulen, Typen, Funktionen, Variablen etc.
    - **Interoperabilität (C-COMMON-TRAITS, C-CONV-TRAITS, C-COLLECT, C-SEND-SYNC, C-GOOD-ERR, C-NUM-FMT, C-RW-VALUE):** Implementierung gängiger Traits (`Debug`, `Clone`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Display`, `Default`, `Send`, `Sync`, `From`, `AsRef`, `AsMut`, `FromIterator`, `Extend`), sinnvolle Fehlertypen, Formatierungsoptionen für Zahlentypen.
    - **Macros (C-EVOCATIVE, C-MACRO-ATTR, C-ANYWHERE, C-MACRO-VIS, C-MACRO-TY):** Wohlgeformte und intuitive Makros.
    - **Dokumentation (C-CRATE-DOC, C-EXAMPLE, C-QUESTION-MARK, C-FAILURE, C-LINK, C-METADATA, C-RELNOTES, C-HIDDEN):** Umfassende Dokumentation (siehe auch Abschnitt 6.7).
    - **Vorhersagbarkeit (C-SMART-PTR, C-CONV-SPECIFIC, C-METHOD, C-NO-OUT, C-OVERLOAD, C-DEREF, C-CTOR):** Code soll sich so verhalten, wie er aussieht.
    - **Flexibilität (C-INTERMEDIATE, C-CALLER-CONTROL, C-GENERIC, C-OBJECT):** APIs sollen diverse Anwendungsfälle unterstützen.
    - **Typsicherheit (C-NEWTYPE, C-CUSTOM-TYPE, C-BITFLAG, C-BUILDER):** Effektive Nutzung des Typsystems.
    - **Verlässlichkeit (C-VALIDATE, C-DTOR-FAIL, C-DTOR-BLOCK):** Robuster Code, der Argumente validiert und in Destruktoren nicht fehlschlägt oder blockiert.
    - **Debuggability (C-DEBUG, C-DEBUG-NONEMPTY):** Alle öffentlichen Typen implementieren `Debug`.
    - **Zukunftsfähigkeit (C-SEALED, C-STRUCT-PRIVATE, C-NEWTYPE-HIDE, C-STRUCT-BOUNDS, C-STABLE, C-PERMISSIVE):** Design für Weiterentwicklung ohne Breaking Changes.
- **Rust Style Guide:** Einhaltung des offiziellen Rust Style Guide 22 für Code-Formatierung (Einrückung, Zeilenlänge, Kommentare, Attribute). Die Verwendung von `rustfmt` mit der Standardkonfiguration wird dringend empfohlen, um Konsistenz zu gewährleisten und unnötige Diskussionen über Stilfragen zu vermeiden.22
    - Einrückung: 4 Leerzeichen, keine Tabs.
    - Maximale Zeilenlänge: 100 Zeichen.
    - Kommentare: Bevorzugt Zeilenkommentare (`//`).
    - Attribute: Jedes Attribut auf eigener Zeile.
- **Clippy:** Regelmäßige Verwendung von Clippy (`cargo clippy`) zur Identifizierung von idiomatischen Verbesserungen und häufigen Fehlern. Alle Clippy-Warnungen (mindestens die Standard-Lints) müssen vor dem Mergen von Code behoben werden.
- **Fehlerbehandlung:**
    - Bevorzugung von `Result<T, E>` für Fehler, die behandelt werden können.
    - Verwendung der `thiserror`-Crate zur Erstellung spezifischer, aussagekräftiger Fehlertypen für Bibliotheken.49 `thiserror` reduziert Boilerplate durch automatische Implementierung von `std::error::Error` und `Display` und ermöglicht einfaches Hinzufügen von Kontext und Komposition von Fehlertypen.50 Die `#[from]`-Attribut ist nützlich für die Konvertierung von Fehlern aus anderen Bibliotheken.51
    - Die `anyhow`-Crate kann für Anwendungs-Code verwendet werden, wo es weniger auf spezifische Fehlertypen ankommt und einfache Fehlerpropagation gewünscht ist.52 `anyhow` ist jedoch ein opaquer Fehlertyp, der eine Fehlerbehandlung durch den Aufrufer erschwert und eher für das "Durchreichen" von Fehlern gedacht ist.52
    - Panics sollten auf nicht behebbare Fehler beschränkt sein (z.B. Programmierfehler, inkonsistente Zustände). Bibliotheks-Code sollte möglichst nicht panicen.
    - Fehlermeldungen sollten kontextbezogen und für Entwickler hilfreich sein.
- **Unsafe Code:** Die Verwendung von `unsafe` Rust ist auf ein absolutes Minimum zu beschränken und nur dort einzusetzen, wo es unumgänglich ist (z.B. FFI, direkte Hardware-Interaktion). Jeder `unsafe`-Block muss detailliert kommentiert werden, um die Notwendigkeit und die erfüllten Sicherheitsbedingungen zu rechtfertigen.
- **Nebenläufigkeit (Concurrency):** Nutzung der Rust-eigenen Sicherheitsmechanismen für Nebenläufigkeit (Ownership, Borrowing, `Send`, `Sync`). Bevorzugung von High-Level-Abstraktionen (z.B. `async/await`, `tokio`, `crossbeam`) gegenüber manueller Thread- und Mutex-Verwaltung.
- **Abhängigkeitsmanagement (Cargo):**
    - Sorgfältige Auswahl von externen Crates. Bevorzugung von gut gewarteten, stabilen und weit verbreiteten Bibliotheken.
    - Regelmäßige Überprüfung und Aktualisierung von Abhängigkeiten (`cargo update`).
    - Minimierung der Anzahl von Abhängigkeiten, um Build-Zeiten und Angriffsflächen klein zu halten.
    - Verwendung von semantischer Versionierung.
- **Modulstruktur:** Code sollte logisch in Modulen und Crates organisiert sein, um die Lesbarkeit und Wartbarkeit zu verbessern.53 Domain-Driven Design (DDD) Prinzipien können bei der Strukturierung komplexer Domänen helfen 54, ebenso wie Event-Sourcing-Muster für zustandsbehaftete Systeme, die eine Historie von Änderungen benötigen.56
- **Performance:** Code sollte unter Berücksichtigung von Performance geschrieben werden, ohne jedoch verfrühte Optimierung zu betreiben. Profiling-Tools sind zur Identifizierung von Engpässen einzusetzen.

### 6.2. Code-Review-Prozess

Code-Reviews sind ein integraler Bestandteil des Entwicklungsprozesses, um Code-Qualität, Konsistenz und Wissensaustausch zu gewährleisten.47

- **Review-Pflicht:** Jeder Code-Beitrag (Feature, Bugfix) muss vor dem Mergen in den Hauptentwicklungszweig von mindestens einer anderen Person (idealerweise zwei) gereviewt werden.
- **Review-Kriterien:**
    - Korrektheit und Funktionalität gemäß den Anforderungen.
    - Einhaltung der Programmierrichtlinien (Abschnitt 6.1).
    - Lesbarkeit, Verständlichkeit und Wartbarkeit des Codes.
    - Angemessene Testabdeckung (Unit- und Integrationstests).
    - Aktualität und Korrektheit der Dokumentation (API-Docs, Kommentare).
    - Performance-Implikationen.
    - Sicherheitsaspekte.
    - Keine Einführung von Regressionen.
- **Werkzeuge:** Verwendung von Plattformen wie GitLab oder GitHub für Merge Requests und Inline-Kommentare.
- **Konstruktives Feedback:** Reviews sollten konstruktiv, respektvoll und auf den Code fokussiert sein. Ziel ist die Verbesserung des Codes und des gemeinsamen Verständnisses.
- **Verantwortung des Autors:** Der Autor ist verantwortlich für die Adressierung der Review-Kommentare und die Sicherstellung der Code-Qualität.
- **Zeitnahe Reviews:** Reviews sollten zeitnah durchgeführt werden, um den Entwicklungsfluss nicht unnötig zu blockieren.

Durch Code-Reviews können Fehler frühzeitig erkannt, die Codequalität verbessert und das Wissen im Team verteilt werden. Ein Senior-Kollege kann beispielsweise ineffiziente Datenbankabfragen identifizieren und Optimierungen vorschlagen, die die Performance erheblich verbessern.47

### 6.3. Versionskontrollstrategie (Git)

Eine konsistente Versionskontrollstrategie ist entscheidend für die Zusammenarbeit und Nachverfolgbarkeit.

- **Branching-Modell:** Empfohlen wird ein Modell wie Git Flow oder ein einfacheres GitHub/GitLab Flow.
    - `main` (oder `master`): Enthält stabilen, produktionsreifen Code. Direkte Commits sind verboten.
    - `develop`: Hauptentwicklungszweig, von dem Feature-Branches abzweigen und in den sie zurückgemerged werden.
    - Feature-Branches (`feature/name-des-features`): Für die Entwicklung neuer Funktionen.
    - Release-Branches (`release/version-nummer`): Zur Vorbereitung von Releases (Stabilisierung, Bugfixing).
    - Hotfix-Branches (`hotfix/problem-beschreibung`): Für dringende Korrekturen an produktivem Code.
- **Commit-Nachrichten:**
    - Müssen klar, prägnant und im Imperativ formuliert sein (z.B. "Fix: Behebe Absturz beim Fenster schließen").
    - Sollten eine Zusammenfassungszeile (max. 50 Zeichen) und optional einen detaillierteren Body enthalten.
    - Bezugnahme auf Issue-Nummern (z.B. "Fixes #123").
- **Merging:**
    - Bevorzugung von Merge Requests (Pull Requests) für alle Änderungen an `develop` und `main`.
    - Squash-Merges oder Rebase-Merges können verwendet werden, um die Commit-Historie sauber zu halten, abhängig von der Teampräferenz.
- **Tagging:** Releases auf dem `main`-Branch müssen mit einer Versionsnummer getaggt werden (gemäß Semantischer Versionierung, z.B. `v1.2.3`).
- **Code-Formatierung vor Commit:** Es wird empfohlen, Code vor dem Commit automatisch mit `rustfmt` zu formatieren (z.B. über Git Hooks).

### 6.4. Teststrategie und -richtlinien

Eine umfassende Teststrategie ist unerlässlich, um die Qualität und Zuverlässigkeit von NovaDE sicherzustellen.2

- **Testarten:**
    
    - **Unit-Tests (Rust: `#[test]` in Modulen):** Testen isolierter Code-Einheiten (Funktionen, Methoden).59 Sie werden typischerweise in derselben Datei wie der zu testende Code oder in einem Untermodul `tests` (z.B. `#[cfg(test)] mod tests {... }`) definiert.59
        - Jede öffentliche Funktion und Methode sollte Unit-Tests haben.
        - Private Funktionen können indirekt über öffentliche Schnittstellen oder direkt getestet werden, wenn sie komplexe Logik enthalten.
    - **Integrationstests (Rust: in `tests`-Verzeichnis):** Testen die Interaktion zwischen verschiedenen Modulen oder Crates.59 Sie greifen auf die öffentliche API der Library zu.59
        - Fokus auf Schnittstellen und Datenflüsse zwischen Komponenten.
        - Beispiel: Testen der D-Bus-Kommunikation zwischen zwei NovaDE-Diensten.
    - **UI-/Funktionstests:** Testen der Benutzeroberfläche und End-to-End-Workflows aus Sicht des Benutzers.
        - Werkzeuge wie `dogtail` (für Accessibility-basierte UI-Automatisierung) oder spezifische Test-Frameworks für Wayland/GTK könnten evaluiert werden.
        - Diese Tests sind oft aufwendiger zu erstellen und zu warten.
    - **Performancetests:** Überprüfung der nicht-funktionalen Anforderungen bezüglich Performance (Reaktionszeiten, Ressourcennutzung).58
        - Definierte Benchmarks und Testszenarien.
    - **Dokumentationstests (`rustdoc`):** Sicherstellung, dass Code-Beispiele in der Dokumentation korrekt sind und kompilieren.59 Alle Beispiele in der Dokumentation sollten testbar sein (`/// ```rust... ///`).
- **Testabdeckung:** Anstreben einer hohen Code-Abdeckung (z.B. > 80% für Kernkomponenten). Tools wie `tarpaulin` oder `grcov` können zur Messung verwendet werden. Testabdeckung allein ist jedoch kein Garant für Qualität; die Qualität der Tests ist ebenso wichtig.47
    
- **Testdatenmanagement:** Strategien für die Erstellung und Verwaltung von Testdaten, insbesondere für UI- und Integrationstests.
    
- **Mocking/Stubbing:** Verwendung von Mocking-Frameworks (z.B. `mockall` in Rust) oder manuellen Stubs, um Komponenten während Unit- und Integrationstests zu isolieren.
    
- **Rust Testing Best Practices:**
    
    - Verwendung von Assertions-Makros (`assert!`, `assert_eq!`, `assert_ne!`, `should_panic`).59 `pretty_assertions` kann für bessere Fehlermeldungen bei Vergleichen großer Objekte verwendet werden.62
    - Deskriptive Testfunktionsnamen (z.B. `test_login_success`, `test_render_panel_with_widgets`).59
    - Klare Definition des Testzwecks.59
    - Setup- und Teardown-Logik für Tests, die einen bestimmten Zustand erfordern (z.B. temporäre Dateien mit `tempfile` 62, Initialisierung von Testdatenbanken).
    - Testen von asynchronem Code mit `#[tokio::test]` oder äquivalenten Attributen für den verwendeten Async-Runtime.62
    - Organisation von gemeinsam genutzten Test-Hilfsfunktionen in einem `tests/common` Modul oder einer lokalen `dev-dependency` Crate.62
    - Erwägung von Property-basiertem Testen (z.B. mit `proptest` 62) für Funktionen, die eine breite Palette von Eingaben verarbeiten, um Eigenschaften über zufällige Eingaben zu verifizieren.
    - Erwägung von Mutations-Tests (z.B. mit `cargo-mutants` 62) für kritische Komponenten, um die Effektivität der Testsuite zu bewerten. Ein "gefangener" Mutant (Test schlägt fehl) ist ein gutes Zeichen, ein "verpasster" Mutant (kein Test schlägt fehl) deutet auf eine Lücke in der Testabdeckung hin.62
    - Für Kommandozeilenanwendungen (falls Teile von NovaDE als solche implementiert sind oder Test-Harnesses dies erfordern), können `assert_cmd` und `predicates` für Blackbox-Tests verwendet werden.60 `assert_fs` hilft beim Setup von Dateisystemzuständen für Tests.60

Die Testanforderungen 2 und die Notwendigkeit, verschiedene Aspekte wie Unit-, Integrations-, Benutzerakzeptanz- und Performancetests abzudecken 58, unterstreichen die Bedeutung einer mehrschichtigen Teststrategie.

### 6.5. Continuous Integration und Continuous Delivery (CI/CD)

Automatisierte CI/CD-Pipelines sind entscheidend für eine konsistente und zuverlässige Softwareentwicklung und -bereitstellung.47

- **Implementierung von CI/CD-Pipelines:** Verwendung von Werkzeugen wie GitLab CI, GitHub Actions oder Jenkins.
- **Automatisierte Schritte:**
    - **Build:** Kompilierung des Codes bei jedem Commit/Merge Request auf allen Zielplattformen.
    - **Tests:** Automatische Ausführung aller Unit-, Integrations- und Dokumentationstests.
    - **Static Analysis:** Ausführung von `rustfmt --check`, `cargo clippy` und ggf. weiteren statischen Analysewerkzeugen.
    - **Code Coverage Reports:** Generierung von Testabdeckungsberichten.
- **Deployment-Strategie:**
    - Automatisierte Bereitstellung auf Staging-/Testumgebungen nach erfolgreichen Builds auf dem `develop`-Zweig.
    - Für Produktionsreleases:
        - **Kleine Batches (Small Deploys):** Änderungen in kleineren, inkrementellen Updates häufig bereitstellen, um das Risiko kritischer Fehler zu reduzieren und Rollbacks zu erleichtern.47 Anstatt ein komplettes Redesign auf einmal auszurollen, einzelne Komponenten über mehrere Tage verteilen.47
        - **Feature Flags:** Neue Features zunächst für eine Untergruppe von Benutzern aktivieren, um sie in einer kontrollierten Umgebung zu testen, bevor sie für alle ausgerollt werden.47
- **Artefakt-Management:** Speicherung und Versionierung von Build-Artefakten (Binärdateien, Debug-Symbole, Pakete).

CI/CD reduziert die Bereitstellungszeit von Stunden auf Minuten und stellt sicher, dass Qualitätsprüfungen niemals übersprungen werden.47

### 6.6. Fehlerbehandlung, Logging und Debugging

Robuste Mechanismen für Fehlerbehandlung, Logging und Debugging sind für die Entwicklung und Wartung eines komplexen Systems wie NovaDE unerlässlich.

- **Fehlerbehandlung (Error Handling):**
    
    - **Rusts `Result<T, E>`:** Standard für fallible Operationen.
    - **`thiserror`:** Für die Definition spezifischer, semantisch reicher Fehlertypen in Bibliotheks-Crates.49 Dies ermöglicht es Aufrufern, Fehler programmatisch zu behandeln und zu unterscheiden. `thiserror` hilft, Boilerplate zu reduzieren und klare Fehlermeldungen zu generieren.50 Es wird empfohlen, eine Hierarchie von Fehlertypen zu erstellen, wobei spezifischere Fehler in allgemeinere Fehler überführt werden können (z.B. `ParseError::DecryptionError(DecryptionError)`).49
    - **`anyhow`:** Kann für Anwendungs-Code oder an den oberen Rändern von Bibliotheken verwendet werden, um Fehler einfach zu propagieren, wenn eine detaillierte Fehlerbehandlung durch den Aufrufer nicht erforderlich ist.52 Es ist jedoch zu beachten, dass `anyhow::Error` ein opaker Typ ist, der die Introspektion erschwert.52
    - **Panics:** Sollten vermieden werden, außer bei nicht behebbaren Programmierfehlern oder inkonsistenten Zuständen, die ein sofortiges Beenden erfordern. Bibliotheken dürfen nicht panicen (C-FAILURE 21).
    - **Fehlerkontext:** Fehler sollten ausreichend Kontextinformationen enthalten, um die Ursache des Fehlers nachvollziehen zu können. Bibliotheken wie `eyre` (ähnlich `anyhow`, aber mit Fokus auf Kontext) können hierfür verwendet werden, obwohl das Hinzufügen von Kontext ergonomische Herausforderungen mit sich bringen kann.52
    - Die Entscheidung, ob ein Fehler spezifisch (mit `thiserror`) oder generisch (mit `anyhow`) behandelt wird, hängt davon ab, ob Aufrufer die Fehler introspektieren und unterschiedlich darauf reagieren müssen.49
- **Logging:**
    
    - **Logging Facade:** Verwendung der `log`-Crate als Standard-Logging-Fassade.
    - **Logging Backend:** Ein konfigurierbares Backend wie `env_logger` (einfach) oder `tracing` (strukturierter, leistungsfähiger) sollte verwendet werden.
    - **Log-Level:** Klare Definition der Verwendung von Log-Levels (ERROR, WARN, INFO, DEBUG, TRACE).
        - ERROR: Kritische Fehler, die die Funktionalität beeinträchtigen.
        - WARN: Unerwartete Situationen oder potenzielle Probleme.
        - INFO: Allgemeine Betriebsinformationen, wichtige Ereignisse.
        - DEBUG: Detaillierte Informationen zur Fehlerdiagnose für Entwickler.
        - TRACE: Sehr detaillierte Informationen, typischerweise für tiefgreifendes Debugging.
    - **Log-Nachrichten-Inhalt:** Log-Nachrichten sollten Zeitstempel, Modul-/Funktionsname, Thread-ID und die eigentliche Nachricht enthalten. Strukturierte Logs (z.B. JSON-Format) können die Analyse erleichtern.
    - **PII (Personally Identifiable Information):** Keine sensiblen Benutzerdaten (Passwörter, private Inhalte) in Logs schreiben.
- **Debugging:**
    
    - **`Debug` Trait:** Alle öffentlichen Typen müssen `std::fmt::Debug` implementieren (C-DEBUG 21). Die `Debug`-Repräsentation sollte niemals leer sein (C-DEBUG-NONEMPTY 21).
    - **Debug-Symbole:** Entwicklungs-Builds müssen Debug-Symbole enthalten, um die Verwendung von Debuggern wie GDB oder LLDB zu ermöglichen. `rust-gdb` oder `rust-lldb` können die Debugging-Erfahrung für Rust-Code verbessern.
    - **Spezifische Debugging-Werkzeuge:**
        - Für Wayland: Debugging-Tools wie `weston-debug` oder Wayland-Protokoll-Logger.
        - Für GTK: GTK Inspector.
        - Für PipeWire: `pw-dump`, `pw-top`, `pw-cli`, `helvum`, `qpwgraph`.12
    - **Core Dumps:** Konfiguration des Systems zur Erstellung von Core Dumps bei Abstürzen von NovaDE-Prozessen zur Post-Mortem-Analyse.

### 6.7. Dokumentationsstandards und -pflege

Eine umfassende und aktuelle Dokumentation ist entscheidend für die Wartbarkeit, Erweiterbarkeit und Nutzung von NovaDE.1 Sie sollte als integraler Bestandteil des Entwicklungsprozesses betrachtet werden, nicht als nachträgliche Aufgabe.3

- **`rustdoc` als primäres Werkzeug:** Die API-Dokumentation wird direkt im Rust-Quellcode mit `rustdoc`-Kommentaren (`///` für Items, `//!` für Module) geschrieben.61
    
- **Inhaltsanforderungen für API-Dokumentation:**
    
    - **Crate-Level-Dokumentation (`//!` in `lib.rs` oder `main.rs`):** Eine gründliche Einführung in das Crate, seinen Zweck, Hauptfunktionen und ein Anwendungsbeispiel.21
    - **Dokumentation aller öffentlichen Items:** Jede öffentliche Funktion, Methode, Struct, Enum, Trait und Makro muss dokumentiert werden.61 Die erste Zeile sollte eine prägnante Zusammenfassung sein.63
    - **Code-Beispiele:** Für jedes öffentliche Item sollte mindestens ein lauffähiges Code-Beispiel (`/// ```rust... ///`) bereitgestellt werden, das mit `cargo test --doc` getestet werden kann.21 Beispiele sollten den `?`-Operator verwenden, nicht `try!` oder `unwrap` (C-QUESTION-MARK 21).
    - **Fehler, Panics, Sicherheit:** Dokumentation von Fehlerbedingungen (welche Fehler können zurückgegeben werden), Situationen, die zu Panics führen können, und Sicherheitsüberlegungen bei `unsafe` Code (C-FAILURE 21, 61).
    - **Markdown-Nutzung:** Effektive Verwendung von Markdown für Strukturierung (Überschriften wie `# Examples`, `# Panics`, `# Errors`, `# Safety` 61), Listen, Code-Blöcke, Tabellen und Links zu anderen Teilen der Dokumentation (Intra-Doc-Links) oder externen Ressourcen.61
    - Vermeidung von Implementierungsdetails in der öffentlichen API-Dokumentation, wenn sie für den Benutzer nicht relevant sind (C-HIDDEN 21).
- **`Cargo.toml` Metadaten:** Die `Cargo.toml`-Datei jedes Crates muss vollständige Metadaten enthalten: `authors`, `description`, `license`, `homepage`, `documentation` (Link zur `docs.rs`-Seite), `repository`, `keywords`, `categories` (C-METADATA 21).
    
- **Release Notes / Changelog:** Jedes Release muss von Release Notes begleitet werden, die alle signifikanten Änderungen, neuen Features, Bugfixes und Breaking Changes dokumentieren (C-RELNOTES 21). Das Format "Keep a Changelog" wird empfohlen.
    
- **Technische Gesamtspezifikation (dieses Dokument):** Dieses Dokument selbst ist eine zentrale Säule der Dokumentation. Es muss als "lebendiges Dokument" gepflegt und bei jeder signifikanten Architektur- oder Designänderung aktualisiert werden.1 Agile Methoden helfen, mit sich ändernden Anforderungen umzugehen, und die Dokumentation muss diesen Änderungen folgen.65
    
- **UX-Dokumentation Anbindung:** UX-Anforderungen und Design-Entscheidungen sollten mit den technischen Komponenten und Spezifikationen verknüpft werden.66 Dies stellt sicher, dass die technische Implementierung die beabsichtigte Nutzererfahrung widerspiegelt. UX-Dokumentation sollte als Produkt-Asset behandelt und versioniert werden.67
    
- **Zielgruppenorientierung:** Dokumentation sollte für die jeweilige Zielgruppe (Entwickler, Endbenutzer, Systemadministratoren) verständlich und relevant sein.3
    
- **Konsistenz und Klarheit:** Verwendung einheitlicher Terminologie und klarer, präziser Sprache. Vermeidung von Ambiguität und unnötigem Jargon.1
    
- **Pflegeprozess:**
    
    - Dokumentationsupdates sind Teil der "Definition of Done" für jede Entwicklungsaufgabe.
    - Dokumentationsänderungen unterliegen dem Code-Review-Prozess.
    - Regelmäßige Überprüfung der gesamten Dokumentation auf Aktualität und Korrektheit.

Die Herausforderung, Dokumentation synchron mit einer sich schnell entwickelnden Desktop-Umgebung zu halten, ist erheblich. Dies erfordert Disziplin und die Integration von Dokumentationsaufgaben in den Kernentwicklungsworkflow. Wenn Code-Änderungen erst dann als abgeschlossen gelten, wenn die zugehörige Dokumentation (API-Dokumente, Benutzerhandbücher bei Betroffenheit und diese Spezifikation selbst) ebenfalls aktualisiert und überprüft wurde, wird die Dokumentation zu einem proaktiven Bestandteil der Entwicklung und nicht zu einem nachträglichen Gedanken.3 Dies kann durch die Verknüpfung von Dokumentationsaufgaben mit Entwicklungsaufgaben in einem Issue-Tracker unterstützt werden.

## 7. Richtlinien für Systemintegration und Erweiterungen

Dieser Abschnitt behandelt, wie NovaDE erweitert werden kann und wie es sich in das breitere Linux-Ökosystem integriert. Eine gut definierte Strategie für Erweiterbarkeit und Integration ist entscheidend für die Flexibilität und Akzeptanz der Desktop-Umgebung.

### 7.1. Plugin-Architektur und -entwicklung (Falls zutreffend)

Sollte NovaDE ein Plugin-System zur Erweiterung seiner Kernkomponenten (z.B. für Shell-Widgets, Compositor-Effekte, Dateimanager-Erweiterungen) unterstützen, müssen folgende Aspekte definiert werden:

- **Plugin-API:**
    - **Stabilität:** Klare Definition der Stabilität der Plugin-API (z.B. SemVer für die API). Breaking Changes müssen sorgfältig verwaltet und kommuniziert werden.
    - **Versionierung:** Ein Mechanismus zur Versionierung von Plugins und der Plugin-API, um Kompatibilitätsprobleme zu vermeiden.
    - **Sprachbindung:** Definition, in welchen Sprachen Plugins entwickelt werden können (z.B. nur Rust, oder auch C, Python über FFI).
    - **Funktionsumfang:** Welche Aspekte von NovaDE können durch Plugins erweitert oder modifiziert werden?
- **Verpackung und Entdeckung:** Wie werden Plugins paketiert, installiert und vom System erkannt (z.B. spezifische Verzeichnisse, Metadaten-Dateien)?
- **Ressourcenmanagement und Sandboxing:**
    - Wie werden Ressourcen (Speicher, CPU) von Plugins verwaltet?
    - Gibt es Sandboxing-Mechanismen, um die Auswirkungen fehlerhafter oder bösartiger Plugins zu begrenzen? Dies ist besonders wichtig, wenn Plugins in privilegierten Prozessen laufen.
- **Entwicklungsrichtlinien für Plugin-Ersteller:**
    - Dokumentation der Plugin-API.
    - Best Practices für die Entwicklung stabiler und sicherer Plugins.
    - Beispiel-Plugins.

Eine gut durchdachte Plugin-Architektur kann die Flexibilität und das Community-Engagement erheblich steigern, birgt aber auch Komplexität und potenzielle Sicherheitsrisiken. Die Balance zwischen diesen Aspekten ist entscheidend. Die Prinzipien des Model Context Protocol (MCP) 17, obwohl für KI-Systeme entwickelt, bieten wertvolle Anregungen für die Gestaltung sicherer Plugin-Systeme, insbesondere hinsichtlich Authentifizierung, Autorisierung und Sandboxing 19, falls Plugins weitreichende Fähigkeiten besitzen. Wenn Plugins beispielsweise auf D-Bus-Schnittstellen zugreifen oder beliebigen Code ausführen können, ist das Sicherheitsmodell (siehe Abschnitt 7.3) von größter Bedeutung.

### 7.2. Integration von Drittanbieter-Anwendungen

NovaDE muss eine nahtlose Integration von Standard-Linux-Anwendungen gewährleisten.

- **Verhalten von Anwendungen:**
    - **Native Wayland-Anwendungen:** GTK- und Qt-Anwendungen, die Wayland nativ unterstützen, sollten ohne Probleme laufen und alle relevanten Wayland-Protokolle (z.B. `xdg-shell` für Fensterdekorationen, `input-method-unstable-v2` für Eingabemethoden) nutzen können.
    - **X11-Anwendungen:** Unterstützung über XWayland ist kritisch für die Abwärtskompatibilität.15 Die Performance und Stabilität von XWayland muss für gängige Anwendungen sichergestellt sein. Probleme, die bei der Nutzung von XWayland auftreten (z.B. Fensterdekorationen, globale Tastenkürzel, Screen-Sharing bei bestimmten Apps 15), müssen aktiv angegangen oder klar dokumentiert werden.
    - **Electron-basierte Anwendungen:** Diese laufen oft standardmäßig unter X11 (via XWayland) oder können mit Flags (`--enable-features=UseOzonePlatform --ozone-platform=wayland`) für native Wayland-Unterstützung gestartet werden. NovaDE sollte beide Szenarien unterstützen.
- **Freedesktop.org Standards:** Die Einhaltung von Freedesktop.org-Standards ist fundamental für eine gute Integration:
    - **Desktop Entries (`.desktop`-Dateien):** Für die Anzeige von Anwendungen in Menüs und Startern, inklusive Aktionen und MIME-Typ-Assoziationen.
    - **Icon Themes:** Korrekte Handhabung von Icon-Themes gemäß der Spezifikation.
    - **MIME Types:** Systemweite Datenbank für MIME-Typen und deren zugeordnete Anwendungen.
    - **Notifications:** Empfang und Anzeige von Benachrichtigungen gemäß der Freedesktop Notification Specification.
- **XDG Desktop Portals:** Für sandboxed Anwendungen (Flatpak, Snap) sind XDG Desktop Portals der Standardmechanismus, um sicher auf Systemressourcen zuzugreifen.6 NovaDE muss die notwendigen Portal-Backends bereitstellen oder sicherstellen, dass generische Backends (wie `xdg-desktop-portal-gtk`) korrekt funktionieren. Dies betrifft Funktionen wie:
    - Dateiauswahl (`org.freedesktop.portal.FileChooser`).
    - Öffnen von URIs (`org.freedesktop.portal.OpenURI`).
    - Drucken (`org.freedesktop.portal.Print`).
    - Screen-Sharing und -Aufnahme (`org.freedesktop.portal.ScreenCast`), oft in Verbindung mit PipeWire.12
    - Globale Einstellungen (`org.freedesktop.portal.Settings`).

### 7.3. Sicherheitsrichtlinien für Erweiterungen und Plugins

Die Sicherheit von Erweiterungen und Plugins ist von größter Bedeutung, da sie potenziell tief in das System eingreifen können.

- **Berechtigungsmodell:**
    - Falls ein Plugin-System existiert, muss ein klares Berechtigungsmodell definiert werden. Welche Ressourcen und APIs darf ein Plugin standardmäßig nutzen?
    - Können Benutzer Berechtigungen für Plugins granular verwalten?
    - Das Prinzip der geringsten Rechte (Least Privilege) muss angewendet werden: Ein Plugin sollte nur die Berechtigungen erhalten, die es für seine Funktion unbedingt benötigt.
- **Code-Review und Vetting:**
    - Für offiziell unterstützte oder in einem Community-Repository angebotene Plugins sollte ein Code-Review-Prozess etabliert werden, um Sicherheitslücken und bösartigen Code zu identifizieren.
    - Klare Richtlinien für die Aufnahme von Plugins in offizielle Repositories.
- **Richtlinien für sichere Plugin-Entwicklung:**
    - **Eingabevalidierung:** Alle Eingaben, die ein Plugin von externen Quellen oder dem Benutzer erhält, müssen sorgfältig validiert werden.
    - **Ausgabesanitisierung:** Daten, die von einem Plugin an andere Systemteile oder zur Anzeige weitergegeben werden, müssen ggf. sanitisiert werden.
    - **Fehlerbehandlung:** Robuste Fehlerbehandlung, um Abstürze oder instabile Zustände zu vermeiden.
    - **Ressourcenmanagement:** Sorgfältiger Umgang mit Speicher und anderen Systemressourcen, um Lecks oder übermäßige Nutzung zu vermeiden.
- **Sandboxing (falls möglich):** Wenn technisch machbar, sollten Plugins in einer Sandbox-Umgebung ausgeführt werden, um ihren Zugriff auf das System zu beschränken.
- **Anlehnung an MCP-Sicherheitsprinzipien:** Die für das Model Context Protocol diskutierten Sicherheitsüberlegungen 19 sind auch für NovaDE-Plugins relevant, falls diese signifikante Fähigkeiten haben:
    - **Starke Authentifizierung:** Falls Plugins als separate Prozesse laufen oder von extern geladen werden, Mechanismen zur Verifizierung ihrer Identität.
    - **Bereichsbezogene Autorisierung:** Ein Plugin, das z.B. nur das Wetter anzeigt, benötigt keinen Zugriff auf das Dateisystem.
    - **Ratenbegrenzung:** Verhindern, dass ein fehlerhaftes oder bösartiges Plugin das System durch exzessive API-Aufrufe lahmlegt.

Ein kompromittiertes Plugin kann die Sicherheit der gesamten Desktop-Umgebung untergraben. Daher müssen diese Richtlinien streng durchgesetzt und Entwickler entsprechend geschult werden.

## 8. Anhang

### 8.1. Glossar technischer Begriffe

|   |   |
|---|---|
|**Begriff**|**Definition**|
|**API**|Application Programming Interface: Eine Schnittstelle, die von einer Softwarekomponente bereitgestellt wird, damit andere Softwarekomponenten mit ihr interagieren können.|
|**ALSA**|Advanced Linux Sound Architecture: Eine Kernel-Komponente für Soundkarten-Treiber und eine API für Audio-Anwendungen.12|
|**Compositor**|Eine Software, die die grafischen Ausgaben verschiedener Anwendungen entgegennimmt und zu einem Gesamtbild für den Bildschirm zusammensetzt. Unter Wayland ist der Compositor auch der Display-Server.5|
|**Crate**|Eine Kompilierungs-Einheit in Rust, entweder eine Bibliothek oder eine ausführbare Datei.|
|**CSS**|Cascading Style Sheets: Eine Stylesheet-Sprache, die verwendet wird, um das Aussehen von Dokumenten (typischerweise Webseiten, aber auch GTK4-Anwendungen) zu beschreiben.27|
|**D-Bus**|Ein Nachrichtenbus-System, das die Interprozesskommunikation (IPC) zwischen Anwendungen und Diensten unter Linux ermöglicht.8|
|**DRM**|Direct Rendering Manager: Ein Subsystem des Linux-Kernels, das eine Schnittstelle zu modernen Grafikkarten für Operationen wie Mode-Setting und Speicherverwaltung bereitstellt. Wayland-Compositoren nutzen DRM direkt.5|
|**EGL**|Eine Schnittstelle zwischen Khronos Rendering APIs (wie OpenGL ES oder OpenVG) und dem zugrundeliegenden nativen Plattform-Fenstersystem. Wayland-Compositoren verwenden EGL.6|
|**evdev**|Ein generisches Eingabeereignis-Interface im Linux-Kernel, das Roh-Eingabeereignisse von Geräten wie Tastaturen, Mäusen und Touchscreens bereitstellt.5|
|**FFI**|Foreign Function Interface: Ein Mechanismus, mit dem eine in einer Programmiersprache geschriebene Funktion von einer anderen Programmiersprache aufgerufen werden kann.|
|**Freedesktop.org (XDG)**|Ein Projekt, das Standards und Spezifikationen für die Interoperabilität von Desktop-Umgebungen unter Linux und anderen Unix-artigen Systemen entwickelt.|
|**GSettings**|Ein High-Level-API für Einstellungen in GLib-basierten Anwendungen, das typischerweise DConf als Backend verwendet.|
|**GTK (GIMP Toolkit)**|Ein plattformübergreifendes Widget-Toolkit zur Erstellung grafischer Benutzeroberflächen. GTK4 ist die aktuelle Hauptversion.28|
|**IPC**|Inter-Process Communication: Mechanismen, die es verschiedenen Prozessen ermöglichen, miteinander zu kommunizieren und Daten auszutauschen.|
|**KMS**|Kernel Mode Setting: Ein Verfahren, bei dem der Kernel für die Einstellung des Anzeigemodus (Auflösung, Farbtiefe, Bildwiederholfrequenz) zuständig ist, anstatt des X-Servers oder eines Userspace-Treibers.6|
|**libinput**|Eine Bibliothek, die Eingabegeräte über Kernel-Events (wie evdev) verarbeitet und eine standardisierte Schnittstelle für Wayland-Compositoren und X.Org-Treiber bereitstellt.|
|**MCP**|Model Context Protocol: Ein offener Standard, der die strukturierte Interaktion von KI-Modellen mit externen Daten und Diensten ermöglicht.17|
|**Meson**|Ein Build-System, das auf Benutzerfreundlichkeit, Geschwindigkeit und die Erzeugung korrekter Builds ausgelegt ist. Wird oft für C/C++-Projekte und zunehmend auch für Rust-Projekte in Kombination mit Cargo verwendet.34|
|**NFR**|Non-Functional Requirement: Eine Anforderung, die beschreibt, wie ein System eine bestimmte Funktion ausführen soll (z.B. Performance, Sicherheit), im Gegensatz zu funktionalen Anforderungen, die beschreiben, was das System tun soll.45|
|**PAM**|Pluggable Authentication Modules: Ein Mechanismus unter Linux, der es ermöglicht, Authentifizierungsdienste modular zu integrieren.|
|**PipeWire**|Ein Server und eine API für die Verarbeitung von Audio- und Videoströmen unter Linux. Es zielt darauf ab, PulseAudio und JACK zu ersetzen und bietet geringe Latenz und Flexibilität.12|
|**Rust**|Eine Systemprogrammiersprache, die auf Sicherheit (insbesondere Speichersicherheit) und Performance ausgelegt ist.|
|**rustdoc**|Das Standardwerkzeug in Rust zur Generierung von Dokumentation aus Quellcode-Kommentaren.61|
|**Shell (Desktop Shell)**|Die grafische Benutzeroberfläche einer Desktop-Umgebung, die Elemente wie Panels, Anwendungsstarter und Fensterdekorationen bereitstellt.|
|**Smithay**|Eine Rust-Bibliothek zur Erstellung von Wayland-Compositoren und Utilities.5|
|**Wayland**|Ein Kommunikationsprotokoll, das die Kommunikation zwischen einem Display-Server (dem Wayland-Compositor) und seinen Clients (Anwendungen) spezifiziert. Es soll X11 ersetzen.5|
|**WirePlumber**|Ein modularer Session- und Policy-Manager für PipeWire, der Verbindungen zwischen Geräten und Streams verwaltet und Richtlinien durchsetzt.13|
|**XDG Desktop Portals**|Eine Schnittstelle, die es sandboxed Anwendungen (wie Flatpak oder Snap) ermöglicht, sicher auf Ressourcen außerhalb ihrer Sandbox zuzugreifen (z.B. Dateiauswahl, Drucken).6|
|**XSettings**|Ein Protokoll für X11, das es ermöglicht, Einstellungen (wie Theme, Schriftarten) systemweit zu teilen. GTK kann XSettings unter X11 oder XWayland nutzen.9|
|**XWayland**|Ein X-Server, der als Wayland-Client läuft und es ermöglicht, X11-Anwendungen in einer Wayland-Sitzung auszuführen.15|

### 8.2. Referenzierte Dokumente und Standards

- **Rust Language & Tooling:**
    - The Rust Programming Language Book: [https://doc.rust-lang.org/book/](https://doc.rust-lang.org/book/)
    - Rust API Guidelines: [https://rust-lang.github.io/api-guidelines/](https://rust-lang.github.io/api-guidelines/) (21)
    - Rust Style Guide: [https://doc.rust-lang.org/nightly/style-guide/](https://doc.rust-lang.org/nightly/style-guide/) (22)
    - Rustdoc - How to write documentation: [https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html) (63)
    - Cargo Book: [https://doc.rust-lang.org/cargo/](https://doc.rust-lang.org/cargo/)
    - The `thiserror` Crate: [https://crates.io/crates/thiserror](https://crates.io/crates/thiserror)
    - The `anyhow` Crate: [https://crates.io/crates/anyhow](https://crates.io/crates/anyhow)
- **Wayland & Compositing:**
    - Wayland Protocol Documentation: [https://wayland.freedesktop.org/docs/html/](https://wayland.freedesktop.org/docs/html/) (6)
    - Smithay Project:(https://github.com/Smithay/smithay) (24)
    - Smithay Documentation (master): [https://smithay.github.io/smithay/smithay/](https://smithay.github.io/smithay/smithay/) (25)
    - XDG Shell Protocol: [https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/stable/xdg-shell/xdg-shell.xml](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/blob/main/stable/xdg-shell/xdg-shell.xml)
- **GTK4 & UI:**
    - GTK4 API Reference: [https://docs.gtk.org/gtk4/](https://docs.gtk.org/gtk4/) (28)
    - GTK4 Rust Bindings (`gtk4-rs`): [https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/) (7)
    - GTK CSS Properties: [https://docs.gtk.org/gtk4/css-properties.html](https://docs.gtk.org/gtk4/css-properties.html) (27)
- **Interprozesskommunikation & Multimedia:**
    - D-Bus Specification: [https://dbus.freedesktop.org/doc/dbus-specification.html](https://dbus.freedesktop.org/doc/dbus-specification.html) (14)
    - PipeWire Documentation: [https://pipewire.org/](https://pipewire.org/) (12)
    - WirePlumber Documentation: [https://pipewire.pages.freedesktop.org/wireplumber/](https://pipewire.pages.freedesktop.org/wireplumber/)
- **Freedesktop.org Standards:**
    - Desktop Entry Specification: [https://specifications.freedesktop.org/desktop-entry-spec/latest/](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
    - Icon Theme Specification: [https://specifications.freedesktop.org/icon-theme-spec/latest/](https://specifications.freedesktop.org/icon-theme-spec/latest/)
    - Desktop Notifications Specification: [https://specifications.freedesktop.org/notification-spec/latest/](https://specifications.freedesktop.org/notification-spec/latest/)
    - XDG Base Directory Specification: [https://specifications.freedesktop.org/basedir-spec/latest/](https://specifications.freedesktop.org/basedir-spec/latest/)
    - XDG Desktop Portal Documentation: [https://flatpak.github.io/xdg-desktop-portal/docs/](https://flatpak.github.io/xdg-desktop-portal/docs/)
- **Build System:**
    - Meson Build System Manual: [https://mesonbuild.com/Manual.html](https://mesonbuild.com/Manual.html) (35)
- **Model Context Protocol (MCP) (falls zutreffend):**
    - MCP Introduction (Stytch Blog): 17
    - MCP Overview (OpenCV Blog): 18
    - MCP Security Considerations (Writer Engineering Blog): 19
- **Software Development Best Practices (Allgemein):**
    - OpsLevel - Standards in Software Development: 47
    - Bocasay - Guide to Writing Functional and Technical Specifications: 1
    - Heretto - Technical Specifications: 2
    - WWG - How to Write a Technical Specification: 58
    - Document360 - Software Architecture Documentation: 3
    - Softkraft - How to Write Software Requirements: 4
    - Pulsion - Software Development Best Practices: 48

### Tabelle 4: Konfigurationsparameter und Standardwerte (Auszug/Beispiel)

Diese Tabelle dient als Referenz für wichtige konfigurierbare Parameter innerhalb von NovaDE. Eine vollständige Liste wird im Laufe der Entwicklung gepflegt und erweitert.

|   |   |   |   |   |   |   |
|---|---|---|---|---|---|---|
|**Parametername**|**Komponente**|**Beschreibung**|**Datentyp**|**Standardwert**|**Erlaubte Werte / Bereich**|**Geltungsbereich**|
|`gtk-theme-name`|GTK Settings|Name des zu verwendenden GTK-Themes.|String|"NovaDE-Default"|Name eines installierten GTK-Themes (z.B. "Adwaita", "NovaDE-Dark")|User/System|
|`gtk-font-name`|GTK Settings|Standardschriftart und -größe (z.B. "Cantarell 11").|String|"Sans 10"|Gültige Pango-Schriftartbeschreibung|User/System|
|`gtk-application-prefer-dark-theme`|GTK Settings|Ob Anwendungen ein dunkles Theme bevorzugen sollen, falls vom Theme unterstützt.9|Boolean|`false`|`true`, `false`|User|
|`org.novade.compositor.enableAnimations`|NovaDE Compositor|Aktiviert/Deaktiviert Fensteranimationen (z.B. Minimieren, Öffnen).|Boolean|`true`|`true`, `false`|User|
|`org.novade.compositor.animationSpeedFactor`|NovaDE Compositor|Faktor zur Skalierung der Animationsgeschwindigkeit (1.0 = normal).|Float|`1.0`|`0.1` - `5.0`|User|
|`org.novade.shell.panel.position`|NovaDE Shell|Position des Hauptpanels auf dem Bildschirm.|Enum|"top"|"top", "bottom", "left", "right"|User|
|`org.novade.shell.panel.autohide`|NovaDE Shell|Ob das Panel automatisch ausgeblendet werden soll.|Boolean|`false`|`true`, `false`|User|
|`org.novade.notifications.popupTimeoutSeconds`|NovaDE Notification Service|Anzeigedauer von Benachrichtigungs-Popups in Sekunden (0 = manuell schließen).|Integer|`5`|`0` - `60`|User|
|`org.novade.power.suspendOnIdleMinutes`|NovaDE Power Manager|Zeit in Minuten, nach der das System bei Inaktivität in den Suspend-Modus wechselt (0 = deaktiviert).|Integer|`30`|`0` - `1440`|User|
|`org.novade.input.mouse.accelerationProfile`|NovaDE Settings Daemon (Input)|Beschleunigungsprofil für die Maus.|Enum|"adaptive"|"adaptive", "flat"|User|
|`pipewire.alsa.force-rate`|PipeWire (ALSA Emulation)|Erzwingt eine bestimmte Samplerate für ALSA-Anwendungen (z.B. 48000 Hz).|Integer|`0` (auto)|`0`, `44100`, `48000`, `96000` etc. (siehe PipeWire Doku)|System/User|
|`pipewire.bluetooth.autoswitch-to-headset-profile`|PipeWire (WirePlumber Bluetooth Config)|Automatisches Umschalten auf HSP/HFP-Profil bei eingehendem Anruf/Aufnahme.13|Boolean|`true`|`true`, `false`|User|

Diese Tabelle ist entscheidend für Administratoren, fortgeschrittene Benutzer und Entwickler, um das Systemverhalten zu verstehen, anzupassen und Fehler zu diagnostizieren. Sie zentralisiert Informationen, die andernfalls über verschiedene Komponentendokumentationen verstreut wären, und verbessert so die Transparenz und Verwaltbarkeit von NovaDE.39

## 9. Schlussfolgerungen und Empfehlungen

Diese Technische Gesamtspezifikation und Richtliniensammlung für NovaDE legt den Grundstein für eine strukturierte, qualitativ hochwertige und zukunftssichere Entwicklung der Desktop-Umgebung. Durch die detaillierte Ausarbeitung der Architektur, der Kernkomponenten, der nicht-funktionalen Anforderungen, der Schnittstellen und der Entwicklungsrichtlinien wird ein gemeinsames Verständnis geschaffen und ein Rahmen für alle Beteiligten etabliert.

**Wesentliche Prinzipien und Erkenntnisse:**

1. **Modularität und klare Schnittstellen:** Die Zerlegung von NovaDE in klar definierte Kernkomponenten mit wohldefinierten Schnittstellen (primär D-Bus und Wayland) ist fundamental für Wartbarkeit, Testbarkeit und parallele Entwicklung.1 Die Verwendung von Rust fördert zusätzlich die Erstellung sicherer und performanter Module.
2. **Technologische Basis:** Die Wahl von Wayland, Smithay, GTK4, PipeWire und Rust als Kerntechnologien positioniert NovaDE als moderne und leistungsfähige Desktop-Umgebung. Dies bringt jedoch auch spezifische Herausforderungen mit sich (z.B. Wayland-Adoption, Komplexität von Smithay als Bibliothek), die kontinuierliche Aufmerksamkeit erfordern.5
3. **Nicht-funktionale Anforderungen (NFRs):** Performance, Stabilität, Sicherheit und Benutzbarkeit sind keine nachrangigen Aspekte, sondern müssen von Beginn an in Design und Implementierung berücksichtigt und durch messbare Kriterien spezifiziert werden.45
4. **Entwicklungsdisziplin:** Die Einhaltung der festgelegten Programmierrichtlinien (insbesondere der Rust API und Style Guides), des Code-Review-Prozesses und der Teststrategie ist unerlässlich für die langfristige Gesundheit des Projekts.21
5. **Lebendige Dokumentation:** Diese Spezifikation und die zugehörige API-Dokumentation (`rustdoc`) müssen kontinuierlich gepflegt und als integraler Bestandteil des Entwicklungsprozesses behandelt werden. Veraltete Dokumentation ist ein erhebliches Projektrisiko.3 Die Verknüpfung von UX-Anforderungen mit technischen Spezifikationen ist hierbei ein wichtiger Aspekt.66
6. **Sicherheit als Fundament:** Insbesondere bei Systemkomponenten wie einem Compositor und bei der Integration von Erweiterungen oder Protokollen wie MCP müssen Sicherheitsaspekte (Least Privilege, Sandboxing, Validierung) höchste Priorität haben.19
7. **Iterative Entwicklung und Anpassungsfähigkeit:** Die Softwareentwicklung, insbesondere für ein komplexes System wie eine Desktop-Umgebung, ist ein iterativer Prozess. Die hier festgelegten Richtlinien sollten Flexibilität für Anpassungen und Weiterentwicklungen ermöglichen, beispielsweise durch agile Methoden und die Bereitschaft, auf Feedback und neue technologische Entwicklungen zu reagieren.65

**Empfehlungen für die weitere Vorgehensweise:**

1. **Implementierung der Kernarchitektur:** Fokus auf die stabile Implementierung des Wayland-Compositors, der grundlegenden Shell-Funktionen und der D-Bus-basierten Dienste als Fundament.
2. **Aufbau der CI/CD-Infrastruktur:** Frühzeitige Einrichtung von CI/CD-Pipelines, um automatisierte Tests und Qualitätsprüfungen von Beginn an zu etablieren.47
3. **Detaillierte API-Spezifikation und -Dokumentation:** Parallele Entwicklung der `rustdoc`-API-Dokumentation für alle öffentlichen Schnittstellen der Kern-Crates.
4. **Kontinuierliche Überprüfung und Anpassung dieser Spezifikation:** Regelmäßige Reviews dieses Dokuments im Entwicklungsteam, um sicherzustellen, dass es den aktuellen Stand und die zukünftige Ausrichtung von NovaDE widerspiegelt.
5. **Community-Einbindung (falls zutreffend):** Bei Open-Source-Projekten ist die frühzeitige Einbindung einer Community für Feedback und Beiträge wertvoll. Klare Dokumentation und Richtlinien sind hierfür eine Voraussetzung.

Die konsequente Anwendung der in diesem Dokument dargelegten Prinzipien und Richtlinien wird maßgeblich dazu beitragen, dass NovaDE zu einer robusten, performanten und benutzerfreundlichen Desktop-Umgebung wird, die den Anforderungen der Nutzer gerecht wird und eine solide Basis für zukünftige Erweiterungen bietet.


**Allgemeine Projekt-Gesamtspezifikation: Linux Desktop-Umgebung "NovaDE"**

**I. Vision und Projektziele**

- **Vision:** NovaDE (Nova Desktop Environment) ist eine innovative Linux-Desktop-Umgebung, die eine moderne, schnelle, intuitive und KI-gestützte Benutzererfahrung schafft. Sie ist optimiert für Entwickler, Kreative und alltägliche Nutzer und zielt darauf ab, Produktivität und Freude an der Interaktion mit dem System zu maximieren.
- **Kernziele:**
    1. **Performance:** Eine durchgehend schnelle und reaktionsschnelle Umgebung ohne spürbare Verzögerungen.
    2. **Intuition:** Eine Benutzeroberfläche, die sich natürlich anfühlt, leicht erlernbar ist und den Nutzer intelligent unterstützt, ohne bevormundend zu wirken.
    3. **Modernität:** Einsatz aktueller Technologien und Designprinzipien, um eine zeitgemäße und zukunftssichere Plattform zu bieten.
    4. **Modularität & Wartbarkeit:** Eine klare Architektur, die einfache Erweiterbarkeit, Testbarkeit und Wartung ermöglicht.
    5. **Anpassbarkeit:** Dem Benutzer weitreichende Möglichkeiten zur Personalisierung des Erscheinungsbildes und Verhaltens der Umgebung bieten.
    6. **KI-Integration:** Nahtlose und sichere Integration von KI-Funktionen als hilfreiche Assistenz, stets unter voller Benutzerkontrolle.
    7. **Stabilität & Sicherheit:** Höchste Priorität auf Systemstabilität und Datensicherheit durch den Einsatz geeigneter Technologien und Entwicklungspraktiken.

**II. Architektonischer Überblick: Geschichtete Architektur**

NovaDE basiert auf einer strengen, vier-schichtigen Architektur, um Modularität, lose Kopplung und hohe Kohäsion innerhalb der Schichten zu gewährleisten. Jede Schicht hat klar definierte Verantwortlichkeiten und kommuniziert mit anderen Schichten ausschließlich über wohldefinierte Schnittstellen.

1. **Kernschicht (Core Layer)**
    
    - **Verantwortlichkeiten:** Enthält die absolut grundlegendsten, systemweit genutzten Elemente. Dies umfasst:
        - Fundamentale Datentypen (z.B. `Point<T>`, `Size<T>`, `Rect<T>`, `Color`, `WorkspaceId`, `WindowIdentifier`, `TokenIdentifier`, `SettingKey`).
        - Allgemeine Dienstprogramme und Hilfsfunktionen (z.B. String-Manipulation, mathematische Helfer, die keine spezifische Domänenlogik enthalten).
        - Basis-Infrastruktur für Konfigurationsparsing und -zugriff (Primitive zum Laden/Speichern, aber nicht die Konfigurationswerte selbst).
        - Initialisierung und Konfiguration des globalen Logging-Frameworks (`tracing`).
        - Allgemeine Fehlerdefinitionen und -Traits (z.B. ein Basis-`CoreError`-Enum mit `thiserror`), die als Grundlage für spezifischere Fehler in höheren Schichten dienen.
    - **Abhängigkeiten:** Keine Abhängigkeiten zu anderen Schichten von NovaDE. Minimale externe Abhängigkeiten (Rust `std`, `uuid`, `chrono`, `thiserror`, `tracing`, `serde`, `toml`, `once_cell`).
    - **Interaktionen:** Stellt seine Funktionalität allen darüberliegenden Schichten (Domäne, System, UI) zur Verfügung.
2. **Domänenschicht (Domain Layer)**
    
    - **Verantwortlichkeiten:** Beinhaltet die gesamte Kernlogik und die Geschäftsregeln der Desktop-Umgebung, unabhängig von spezifischen UI-Implementierungen oder Systemdetails (wie D-Bus oder Wayland-Protokollen). Dies umfasst:
        - **`domain::theming`:** Logik der Theming-Engine, Verwaltung von Design-Tokens (`TokenIdentifier`, `TokenValue`, `RawToken`, `ThemeDefinition`), Regeln zur Anwendung von Themes (Hell/Dunkel, Akzentfarben), dynamische Theme-Wechsel. Stellt den `AppliedThemeState` bereit.
        - **`domain::workspaces`:** Umfassende Verwaltungslogik für Arbeitsbereiche ("Spaces"). Definiert die `Workspace`-Entität, Regeln für die Zuweisung von Fenstern (`WindowIdentifier`) zu `WorkspaceId`s, Orchestrierung aller Workspaces (`WorkspaceManager`), Verwaltung des aktiven Workspace und Persistenz der Workspace-Konfiguration (`WorkspaceConfigProvider`, `WorkspaceSnapshot`).
        - **`domain::user_centric_services`:**
            - **KI-Interaktionsmanagement:** Logik für KI-Interaktionen (`AIInteractionContext`), Verwaltung von Benutzereinwilligungen (`AIConsent`, `AIConsentStatus`) für KI-Modelle und Datenkategorien (`AIDataCategory`), Verwaltung von KI-Modellprofilen (`AIModelProfile`).
            - **Benachrichtigungsmanagement:** Kernlogik zur Verwaltung von Benachrichtigungen (`Notification`, `NotificationAction`), Historie, Filterung, Priorisierung (`NotificationUrgency`), "Bitte nicht stören"-Modus.
        - **`domain::notifications_rules`:** Logik zur dynamischen Verarbeitung von Benachrichtigungen basierend auf konfigurierbaren Regeln (`NotificationRule`, `RuleCondition`, `RuleAction`).
        - **`domain::global_settings_and_state_management`:** Repräsentation, Logik zur Verwaltung und Konsistenz globaler Desktop-Einstellungen (`GlobalDesktopSettings`, `SettingPath`, `SettingValue`). Validierung und Persistenz von Einstellungen (delegiert an Kernschicht).
        - **`domain::window_management_policy`:** Definition von High-Level-Regeln und Richtlinien für Fensterplatzierung, Logik für automatisches Tiling (Layout-Typen wie Spalten, Spiralen), Snapping-Verhalten, Fenstergruppierung und Gap-Management. _Diese Schicht definiert die "Policy", die Systemschicht die "Mechanik"._
    - **Abhängigkeiten:** Nutzt ausschließlich die Kernschicht.
    - **Interaktionen:** Stellt Logik, Zustand und Geschäftsregeln für die Systemschicht und die Benutzeroberflächenschicht bereit, oft über definierte Service-Traits und durch das Aussenden von domain-spezifischen Events (z.B. `ThemeChangedEvent`, `WorkspaceEvent`, `SettingChangedEvent`).
3. **Systemschicht (System Layer)**
    
    - **Verantwortlichkeiten:** Implementiert die Interaktion mit dem zugrundeliegenden Betriebssystem, der Hardware und externen Systemdiensten. Setzt die von der Domänenschicht definierten Richtlinien technisch um.
        - **`system::compositor`:** Smithay-basierter Wayland-Compositor. Verwaltung von Wayland-Clients, Oberflächen (`WlSurface`), Pufferbehandlung (SHM, DMABUF), Implementierung von Wayland-Protokollen serverseitig (`xdg-shell`, `wlr-layer-shell-unstable-v1`, `xdg-decoration-unstable-v1`, `wlr-output-management-unstable-v1`, `wlr-output-power-management-unstable-v1` etc.), XWayland-Integration. Orchestriert das Rendering (delegiert an Renderer-Interface).
        - **`system::input`:** `libinput`-basierte Eingabeverarbeitung. Verarbeitung von Tastatur-, Maus-, Touchpad-Ereignissen. Gestenerkennung. Seat-Management (`SeatState`, `SeatHandler`), Fokus-Handling auf Wayland-Ebene, xkbcommon-Integration für Tastaturlayouts.
        - **`system::dbus`:** Schnittstellenmodule (via `zbus`) für die Interaktion mit System-D-Bus-Diensten: `NetworkManager`, `UPower`, `logind`, `org.freedesktop.Notifications` (Service-Seite, die `domain::user_centric_services::NotificationService` nutzt), `org.freedesktop.secrets` (Freedesktop Secret Service API), `PolicyKit (polkit)`. Enthält D-Bus Service- und Client-Implementierungen.
        - **`system::outputs`:** Verwaltung von Anzeigeausgängen. Implementierung der Server-Seite von `wlr-output-management-unstable-v1` und `wlr-output-power-management-unstable-v1`. Handhabung von Monitorkonfigurationen (Auflösung, Position, Aktivierung, Skalierung, DPMS).
        - **`system::audio`:** Client-Integration für PipeWire (via `pipewire-rs`) zur Steuerung der Systemlautstärke, Auswahl von Audio-Geräten und Verwaltung von Anwendungs-Streams.
        - **`system::mcp`:** Implementierung des Model Context Protocol (MCP) Clients (via `mcp_client_rs`), der die sichere Kommunikation mit MCP-Servern (lokal oder remote) abwickelt, basierend auf Anweisungen und Kontextdaten aus `domain::user_centric_services::AIInteractionLogicService`.
        - **`system::portals`:** Backend-Implementierung für XDG Desktop Portals (`org.freedesktop.portal.FileChooser`, `org.freedesktop.portal.Screenshot` etc.) zur Bereitstellung sicherer Schnittstellen für sandboxed und native Anwendungen.
        - **`system::window_mechanics`:** Konkrete Implementierung der Fenstermanagement-"Mechanik" basierend auf den Richtlinien aus `domain::window_management_policy`. Dies umfasst Fensterpositionierung, Größenänderung, Stapelreihenfolge, Anwendung von Tiling-Layouts, Fokusübergänge und die technische Umsetzung von Fensterdekorationen (serverseitig falls SSD oder in Koordination mit CSD). Stellt die technische Basis für die "Intelligente Tab-Leiste" bereit, indem sie Fensterinformationen und Gruppierungen verwaltet.
    - **Abhängigkeiten:** Nutzt Funktionalität der Kernschicht und der Domänenschicht (z.B. um Domänenregeln anzuwenden, Zustände abzufragen, Konfigurationen zu lesen).
    - **Interaktionen:** Stellt systemnahe Dienste und Ereignisse für die Benutzeroberflächenschicht bereit (z.B. Fenstergeometrie, Eingabeereignisse, Statusänderungen von Systemdiensten). Empfängt Befehle von der UI-Schicht (z.B. Fenster verschieben, Space wechseln, Fokus anfordern) und setzt diese technisch um.
4. **Benutzeroberflächenschicht (User Interface Layer)**
    
    - **Verantwortlichkeiten:** Ist verantwortlich für die Darstellung der gesamten grafischen Benutzeroberfläche und die direkte Interaktion mit dem Benutzer. Basiert vollständig auf GTK4 und den `gtk4-rs` Bindings.
        - **`ui::shell`:** Implementierung der Haupt-Shell-UI:
            - `PanelWidget`: Kontroll- und Systemleiste(n) mit Modulen (`AppMenuButton`, `WorkspaceIndicatorWidget`, `ClockDateTimeWidget`, `SystemTrayEquivalentWidget`, `QuickSettingsButtonWidget`, `NotificationCenterButtonWidget`, `NetworkIndicatorWidget`, `PowerIndicatorWidget`, `AudioIndicatorWidget`).
            - `SmartTabBarWidget`: Intelligente Tab-Leiste pro "Space" mit `ApplicationTabWidget`s.
            - `QuickSettingsPanelWidget`: Ausklappbares Panel für Schnelleinstellungen.
            - `WorkspaceSwitcherWidget`: Adaptive linke Seitenleiste mit `SpaceIconWidget`s zur Navigation zwischen "Spaces".
            - `QuickActionDockWidget`: Konfigurierbares Dock für Schnellzugriffe mit `DockItemWidget`s.
            - `NotificationCenterPanelWidget`: Anzeige von Benachrichtigungsliste und -historie mit `NotificationWidget`s.
        - **`ui::control_center`:** Modulare GTK4-Anwendung für alle Systemeinstellungen (`AppearanceSettingsWidget`, `NetworkSettingsWidget` etc.).
        - **`ui::widgets`:** Implementierung des Widget-Systems:
            - `RightSidebarWidget`: Adaptive rechte Seitenleiste.
            - `WidgetManagerService` (logische UI-Komponente).
            - `WidgetPickerPopover`.
            - `PlacedWidgetWidget` und `ActualWidget` (Basis für `ClockWidget`, `CalendarWidget`, `WeatherWidget`, `SystemMonitorWidget` etc.).
        - **`ui::window_manager_frontend`:** UI-Aspekte des Fenstermanagements:
            - Client-Side Decorations (CSD) Logik (z.B. via `Gtk::HeaderBar`).
            - `OverviewModeWidget`: Fenster- und Workspace-Übersicht.
            - `AltTabSwitcherWidget`: Traditioneller Fensterwechsler.
        - **`ui::notifications_frontend`:** `NotificationPopupWidget` zur Anzeige von Pop-up-Benachrichtigungen.
        - **`ui::theming_gtk`:** Anwendung der von `domain::theming` generierten CSS-Stile auf GTK-Widgets über `Gtk::CssProvider`. Logik zur Anforderung von Theme-Wechseln.
        - **`ui::components`:** Sammlung wiederverwendbarer, anwendungsspezifischer GTK4-Widgets (z.B. `StyledButtonWidget`, `ModalDialogWidget`).
        - **`ui::speed_dial`:** GTK4-Implementierung der Speed-Dial-Startansicht für leere Workspaces.
        - **`ui::command_palette`:** GTK4-Implementierung der kontextuellen Befehlspaletten-UI (`CommandPalettePopover`).
    - **Abhängigkeiten:** Nutzt Funktionalität aller darunterliegenden Schichten (Kern, Domäne, System).
    - **Interaktionen:** Empfängt Eingabeereignisse von der Systemschicht. Visualisiert Zustände und Daten aus der Domänen- und Systemschicht. Löst Aktionen und Zustandsänderungen in der Domänen- und Systemschicht aus, basierend auf Benutzerinteraktionen. Abonniert Events der unteren Schichten, um sich dynamisch zu aktualisieren.

**III. Technologie-Stack (Verbindliche Auswahl)**

|Bereich|Technologie/Standard|Begründung|
|:--|:--|:--|
|Programmiersprache|**Rust**|Überlegene Speichersicherheit ohne Garbage Collector, Performance "close-to-the-metal", starkes Typsystem zur Fehlervermeidung zur Kompilierzeit, moderne Nebenläufigkeitskonzepte. Zukunftssichere Wahl für Systemsoftware.|
|Build-System|**Meson**|Modern, einfach, schnell, exzellente Unterstützung für Rust und C/C++ (für externe Abhängigkeiten), Integration von CMake-basierten Abhängigkeiten via Subprojekte (WrapDB, CMake-Modul).|
|GUI-Toolkit|**GTK4** (mit `gtk4-rs` Bindings)|Modernes, aktiv entwickeltes Toolkit mit erstklassiger Wayland-Unterstützung, robusten Rust-Bindings. CSS-basiertes Theming für flexible und standardisierte Anpassung. Dynamischer Theme-Wechsel zur Laufzeit. Starke Integration ins GNOME-Ökosystem.|
|Wayland Compositor & Bibliotheken|**Smithay Toolkit**|In Rust geschriebene Bibliothek zum Erstellen von Wayland-Compositoren. Perfekte Passung zur Hauptprogrammiersprache Rust, vermeidet C-Bindings für Kernkomponenten des Compositors. Modular, flexibel, aktive Entwicklung, Fokus auf Sicherheit.|
|Essentielle Wayland-Protokolle|`wayland.xml` (Basis), `xdg-shell` (Fenster), `wlr-layer-shell-unstable-v1` (Shell-UI), `xdg-decoration-unstable-v1` (Dekorationen), `wlr-foreign-toplevel-management-unstable-v1` (Fensterliste), `wlr-output-management-unstable-v1` (Monitore), `wlr-output-power-management-unstable-v1` (Monitor-Energie), `input-method-unstable-v1`, `text-input-unstable-v3` (Eingabemethoden), Wayland Core (Zwischenablage/DND), `presentation-time`, `viewporter`, `linux-dmabuf-unstable-v1`, `idle-notify-unstable-v1`. XWayland für Kompatibilität (unterstützt durch Smithay).|Standardisierte Schnittstellen für alle Kernfunktionen einer Desktop-Umgebung unter Wayland. Die Auswahl von `wlr-*`-Protokollen erfolgt zugunsten der verfügbaren Funktionalität in der Rust/Smithay-Welt.|
|Inter-Prozess-Kommunikation (IPC)|**D-Bus** (mit `zbus` Crate)|De-facto-Standard für IPC im Linux-Desktop-Bereich. Nahtlose Integration mit essenziellen Systemdiensten (NetworkManager, UPower, logind, PolicyKit, Benachrichtigungsdienst, Secret Service).|
|KI-Integration|**Model Context Protocol (MCP)** (mit `mcp_client_rs` Crate)|Offener Standard für sichere, standardisierte Verbindung von KI-Modellen (LLMs) mit externen Werkzeugen und Anwendungen. Client-Server-Architektur, definierte Nachrichtenformate. Ermöglicht Anbindung lokaler und Cloud-basierter Modelle, Benutzerkontrolle über Datenzugriffe.|
|Eingabeverarbeitung|**libinput** (integriert via Smithay)|Standardbibliothek unter Linux für die Verarbeitung von Eingabeereignissen auf niedrigem Niveau. Robuste Unterstützung für Touchpad-Gesten. Konsistente und präzise Eingabebehandlung.|
|Audio-Management|**PipeWire** (mit `pipewire-rs` Crate)|Moderner Standard für Audio- und Video-Management unter Linux. Geringe Latenz, flexible Routing-Möglichkeiten, sandboxed Architektur. Kompatibilitätsschichten für PulseAudio-, JACK- und ALSA-Anwendungen.|
|Geheimnisverwaltung|**Freedesktop Secret Service API** (via D-Bus, z.B. mit `secret-service-rs` Crate)|Standardmechanismus zum sicheren Speichern von sensiblen Daten wie Passwörtern oder API-Schlüsseln. Implementierungen wie GNOME Keyring oder KWallet stellen diesen Dienst bereit.|
|Rechteverwaltung|**PolicyKit (polkit)** (via D-Bus)|Standardmechanismus unter Linux zur Autorisierung privilegierter Aktionen durch unprivilegierte Anwendungen nach Benutzerauthentifizierung.|
|Theming-Implementierung (UI)|**Token-basiertes System**, implementiert über **GTK4 CSS Custom Properties (`var()`)** und `@define-color` (als Fallback/Ergänzung).|Abstraktionsebene über konkreten Designwerten (Farben, Schriftgrößen, Abstände). Änderungen an Token-Definitionen (Domänenschicht) werden zur Laufzeit angewendet, was dynamische Theme-Umschaltung ohne Neustart ermöglicht.|
|Sandboxing-Interaktion|**XDG Desktop Portals** (via D-Bus, z.B. mit `xdg-portal-rs` Crate)|Standardisierte D-Bus-Schnittstellen für sandboxed Anwendungen (und native Apps) zum sicheren Zugriff auf Ressourcen außerhalb ihrer Sandbox (Dateidialoge, Kamera-/Mikrofonzugriff, Screencasting etc.). Backend-Implementierung durch NovaDE.|

In Google Sheets exportieren

**IV. Entwicklungsrichtlinien (Verbindlich)**

Diese Richtlinien gewährleisten Konsistenz, Qualität und Wartbarkeit des Codes über das gesamte Projekt.

1. **Programmiersprache Rust:**
    
    - **Stil und Formatierung:** Verbindliche Nutzung von `rustfmt` mit der Standardkonfiguration des Projekts. Maximale Zeilenlänge: 100 Zeichen. Einrückung: 4 Leerzeichen. Kommentare: `//` bevorzugt, `///` oder `//!` für `rustdoc`.
    - **API-Design:** Strikte Befolgung der Rust API Guidelines Checklist (Namensgebung RFC 430 Casing, Trait-Implementierungen, Fehlerbehandlung, Dokumentation, Generics, Newtypes, Builder-Pattern, private Felder, versiegelte Traits wo sinnvoll).
    - **Fehlerbehandlung:**
        - Ausschließliche Verwendung des `thiserror`-Crates zur Definition von benutzerdefinierten, spezifischen Fehler-Enums pro Modul in allen Schichten.
        - Fehler werden über `Result<T, E>` zurückgegeben.
        - Panics (`unwrap()`, `expect()`) sind in Bibliotheks-/Kern-/Domänen-/System-Code strengstens verboten. Ausnahme: `expect()` in Tests oder bei nachweislich nicht behebbaren internen Invariantenverletzungen mit aussagekräftiger Begründung im `expect()`-Aufruf.
        - `#[error("...")]`-Nachrichten müssen klar, prägnant und kontextbezogen sein. Sensible Daten dürfen niemals in Fehlermeldungen erscheinen.
        - Die `source()`-Kette von Fehlern muss durch korrekte Verwendung von `#[source]` und `#[from]` erhalten bleiben.
    - **Logging & Tracing:**
        - Verbindliche Nutzung des `tracing`-Crate-Frameworks für strukturiertes, kontextbezogenes Logging.
        - Verwendung von Spans (`#[tracing::instrument]`) für wichtige Funktionen und Operationen zur Kontextualisierung.
        - Definition und Einhaltung von Standard-Logging-Levels (ERROR, WARN, INFO, DEBUG, TRACE).
        - Sensible Daten dürfen niemals geloggt werden.
    - **Nebenläufigkeit:** Nutzung von Rusts `async/await` für I/O-gebundene oder langlaufende Operationen, insbesondere in der UI- und Systemschicht, um Blockaden zu vermeiden. Verwendung von `tokio` als primäre asynchrone Runtime, wo eine dedizierte Runtime benötigt wird (z.B. für Netzwerkdienste in der Systemschicht). Für GTK-spezifische asynchrone Aufgaben in der UI-Schicht `glib::MainContext::spawn_local`. Thread-sichere Datenstrukturen (`Arc`, `Mutex` von `parking_lot` oder `std::sync`) für geteilte Zustände.
    - **Crate-Struktur:** Logische Aufteilung des Projekts in mehrere Crates (z.B. `novade-core`, `novade-domain`, `novade-system`, `novade-ui`, sowie spezifische Crates für größere Komponenten wie `novade-compositor`, `novade-shell`). Klare öffentliche APIs (`pub`) und interne Modulstruktur (`pub(crate)`).
2. **Versionskontrolle & Branching:**
    
    - **System:** Git.
    - **Modell:** GitHub Flow. Der `main`-Branch repräsentiert stets den neuesten stabilen, potenziell auslieferbaren Zustand.
    - **Workflow:** Feature-Entwicklung erfolgt in kurzlebigen Branches, die von `main` abzweigen. Pull Requests (PRs) sind für das Mergen in `main` obligatorisch. Jeder PR erfordert mindestens ein positives Code-Review von einem anderen Teammitglied und das Bestehen aller CI-Prüfungen.
    - **Commit-Nachrichten:** Einhaltung der Conventional Commits Spezifikation zur Nachvollziehbarkeit und automatisierten Changelog-Generierung.
3. **Teststrategie:**
    
    - **Unit-Tests:** Obligatorisch für alle Komponenten der Kern- und Domänenschicht sowie für Logik-Komponenten der System- und UI-Schicht. Direkt neben dem zu testenden Code (`#[cfg(test)]`). Hohe Codeabdeckung wird angestrebt.
    - **Integrationstests:** Im `tests/`-Verzeichnis auf Crate-Ebene. Testen das Zusammenspiel verschiedener Module oder Schichten und die Interaktion mit (gemockten) externen Schnittstellen.
    - **Compositor-Tests (Systemschicht):** Evaluierung und Einsatz von verschachtelten/headless Wayland-Backends (unterstützt durch Smithay) oder spezialisierten Test-Clients zur Verifizierung der Kernlogik des Compositors (Fensterplatzierung, Protokollimplementierungen).
    - **UI-Tests (UI-Schicht):** Nutzung von Accessibility-APIs zur Inspektion und Steuerung von Widgets, Simulation von Eingabeereignissen auf Wayland-Ebene. Evaluierung von Frameworks für visuelle Regressionstests für GTK4/Rust.
    - **Continuous Integration (CI) Pipeline:**
        - Automatische Ausführung aller definierten Tests (Unit, Integration) bei jedem Push und PR.
        - Prüfungen für Code-Formatierung (`cargo fmt --check`).
        - Linting (`cargo clippy -- -D warnings` mit projektspezifischer Clippy-Konfiguration).
        - Prüfung auf Sicherheitslücken in Abhängigkeiten (`cargo audit`).
        - Ein Merge in `main` ist nur bei erfolgreichem Durchlauf aller CI-Prüfungen gestattet.
4. **Dokumentation:**
    
    - **Code-Dokumentation (`rustdoc`):**
        - Umfassende `rustdoc`-Kommentare (`///` oder `//!`) sind für alle öffentlichen APIs (Crates, Module, Funktionen, Typen, Felder, Traits) zwingend erforderlich.
        - Dokumentation muss nicht nur beschreiben, _was_ eine Komponente tut, sondern auch _warum_ sie existiert und _wie_ sie korrekt verwendet wird.
        - Muss Erklärungen zur Logik, zu Invarianten, zu möglichen Fehlerbedingungen (`# Errors`-Sektion) und zu Sicherheitsaspekten (`# Safety`-Sektion für `unsafe` Code, der strikt zu minimieren ist) enthalten.
        - Aussagekräftige Code-Beispiele (`#[doc(inline)]` oder in separaten `examples/`-Verzeichnissen) sind essenziell.
    - **Architektur-Dokumentation:**
        - Dieses Gesamtspezifikationsdokument dient als zentrales Referenzdokument.
        - High-Level-Architekturdiagramme (Schichtendiagramme, Komponentendiagramme, C4-Modell-Elemente wo sinnvoll) müssen erstellt und aktuell gehalten werden, um das Gesamtsystem und die Interaktionen zwischen den Hauptkomponenten zu beschreiben. Diese sind Teil dieses Dokuments oder als separate, verlinkte Artefakte zu pflegen.
    - **READMEs:** Jedes Crate und jede größere, eigenständige Komponente muss eine `README.md`-Datei besitzen, die den Zweck, die Hauptfunktionen, Build-Anweisungen und grundlegende Nutzungsanweisungen beschreibt.
    - **Metadaten:** Alle `Cargo.toml`-Dateien müssen vollständige und korrekte Metadaten enthalten (`authors`, `description`, `license`, `repository`, `homepage`, `documentation`, `keywords`, `categories`). Dies ist wichtig für die interne Verwaltung, potenzielle Veröffentlichung und Auffindbarkeit.
    - **Benutzer- und Entwicklerdokumentation:** Parallel zur Entwicklung sind Konzepte für eine separate Benutzerdokumentation (Anleitung zur Bedienung der Desktop-Umgebung) und weiterführende Entwicklerdokumentation (z.B. Anleitungen zum Erstellen von Widgets oder Plugins, detaillierte API-Referenzen über `rustdoc` hinaus) zu erstellen.

**V. Initiale Schicht- und Komponentenspezifikationen (Struktur)**

Die detaillierten Spezifikationen für jede Komponente jeder Schicht (wie in den vorherigen Antworten teilweise bereits skizziert) werden als separate, untergeordnete Dokumente oder Abschnitte dieser Gesamtspezifikation geführt. Sie folgen einer einheitlichen Struktur:

1. **Modul-/Komponentenübersicht:** Zweck, Verantwortlichkeiten, Design-Rationale.
2. **Datenstrukturen und Typdefinitionen:** Detaillierte Definition aller relevanten `struct`s, `enum`s, `trait`s und Typaliase in Rust-Syntax, inklusive aller Felder, Varianten und abgeleiteten Traits (z.B. `Debug`, `Clone`, `Serialize`, `Deserialize`, `Default`, `thiserror::Error`). Tabellarische Übersichten für komplexe Typen.
3. **Öffentliche API und Interne Schnittstellen:** Exakte Signaturen aller öffentlichen Funktionen/Methoden. Beschreibung von Vor- und Nachbedingungen, Logik/Algorithmen der Methoden. Spezifikation von Events (Payload-Struktur, Publisher, Subscriber, Auslösebedingungen).
4. **Fehlerbehandlung:** Definition des modulspezifischen Fehler-Enums mit `thiserror`, inklusive aller Varianten, `#[error(...)]`-Nachrichten und der Handhabung von `#[source]` und `#[from]`.
5. **Interaktionen und Abhängigkeiten:** Beschreibung der Interaktionen mit anderen Modulen/Komponenten derselben Schicht und mit Modulen/Komponenten anderer Schichten.
6. **Detaillierte Implementierungsschritte:** Eine schrittweise Anleitung für die Implementierung, die die Erstellung von Dateien, die Definition von Typen und die Implementierung von Methoden und Logik umfasst.
7. **Testaspekte:** Identifikation testkritischer Logik, Beispiele für Testfälle, Hinweise zu Mocking-Strategien.

**VI. Deployment-Überlegungen**

1. **Paketierung:**
    - **Zielformate:** Primär native Pakete für gängige Linux-Distributionen (z.B. `.deb` für Debian/Ubuntu, `.rpm` für Fedora/openSUSE).
    - **Flatpak:** Evaluierung und potenzielle Bereitstellung der gesamten Desktop-Umgebung oder wichtiger Teile davon als Flatpak zur distributionsunabhängigen Installation und für Entwickler-SDKs.
    - **Build-Prozess:** Der Meson-Build-Prozess muss Artefakte für diese Paketformate erstellen können, inklusive korrekter Handhabung von Abhängigkeiten, Installation von Desktop-Dateien (`.desktop`), Icons, Themes, Konfigurationsdateien und systemd-Units.
2. **Systemintegration:**
    - **Display Manager:** Konfiguration für gängige Display Manager (GDM, LightDM, SDDM) zur Auswahl von NovaDE als Sitzung.
    - **systemd User Sessions:** Korrekte Einrichtung von systemd User Sessions für NovaDE.
    - **PAM-Module:** Sicherstellung der Kompatibilität und ggf. Bereitstellung spezifischer PAM-Konfigurationen für Login und Authentifizierung.
    - **XDG Base Directory Specification:** Strikte Einhaltung für alle benutzer- und systemweiten Konfigurations-, Daten- und Cache-Dateien (`$XDG_CONFIG_HOME`, `$XDG_DATA_HOME`, `$XDG_CACHE_HOME`, `$XDG_CONFIG_DIRS`, `$XDG_DATA_DIRS`).
3. **Konfiguration:**
    - Bereitstellung einer sinnvollen Standardkonfiguration für alle Aspekte der Desktop-Umgebung.
    - Klare Mechanismen für Benutzer, um diese Standardeinstellungen über grafische Oberflächen (`ui::control_center`) und ggf. Konfigurationsdateien zu überschreiben.
    - Trennung von Systemkonfiguration (typischerweise in `/etc`) und Benutzerkonfiguration (in `$XDG_CONFIG_HOME`).
4. **Updates:**
    - **Strategie:** Primär über die Paketmanager der jeweiligen Distributionen. Für Flatpak-Installationen über den Flatpak-Update-Mechanismus.
    - **Versionierung:** Semantische Versionierung (SemVer) für das Gesamtprojekt und für kritische Bibliotheks-Crates.
    - **Konfigurationsmigration:** Mechanismen oder Richtlinien zur Handhabung von Änderungen an Konfigurationsformaten oder -schlüsseln bei Updates, um Datenverlust oder Inkompatibilitäten zu vermeiden (z.B. automatische Migration, klare Hinweise für Benutzer).

**VII. Schlussfolgerung**

Diese technische Gesamtspezifikation legt das verbindliche Fundament für die Entwicklung der Linux-Desktop-Umgebung NovaDE. Sie definiert eine klare, geschichtete Architektur, wählt einen modernen und robusten Technologie-Stack basierend auf Rust, Wayland, GTK4 und Smithay und etabliert detaillierte Entwicklungsrichtlinien für Qualität, Konsistenz und Wartbarkeit.

Die präzisen Entscheidungen bezüglich Architektur, Technologien und Richtlinien bilden die unumstößliche Grundlage für die nachfolgenden, detaillierten Implementierungsleitfäden der einzelnen Schichten und Komponenten. Durch die konsequente und disziplinierte Einhaltung dieser Spezifikation wird eine hochwertige, moderne, sichere, performante und anpassungsfähige Desktop-Umgebung entstehen, die den hohen Anforderungen von Entwicklern, Kreativen und alltäglichen Nutzern gerecht wird und eine intuitive, effiziente und persönliche Benutzererfahrung bietet. Dieses Dokument ist lebendig und wird bei Bedarf unter strenger Einhaltung der übergeordneten Vision und Architekturprinzipien aktualisiert.

---

**Technische Gesamtspezifikation & Richtlinien: Linux Desktop-Umgebung "NovaDE" (Kompakte Gesamtdefinition inkl. Features)**

**I. Vision und Kernziele**

- **Vision:** NovaDE (Nova Desktop Environment) ist eine innovative Linux-Desktop-Umgebung, die eine moderne, schnelle, intuitive und KI-gestützte Benutzererfahrung schafft. Sie ist optimiert für Entwickler, Kreative und alltägliche Nutzer und zielt darauf ab, Produktivität und Freude an der Interaktion mit dem System zu maximieren.
- **Kernziele:** Performance, Intuition, Modernität, Modularität & Wartbarkeit, Anpassbarkeit, sichere KI-Integration, Stabilität & Sicherheit.

**II. Architektonischer Überblick: Vier-Schichten-Architektur**

NovaDE basiert auf einer strengen, vier-schichtigen Architektur (Kern, Domäne, System, Benutzeroberfläche) für Modularität, lose Kopplung und hohe Kohäsion. Kommunikation erfolgt über wohldefinierte Schnittstellen.

1. **Kernschicht (Core Layer):**
    
    - **Verantwortlichkeiten:** Fundamentale Datentypen (z.B. `Point<T>`, `Color`), Dienstprogramme, Konfigurationsprimitive (TOML, Serde), Logging (`tracing`), Basis-Fehler (`thiserror`).
    - **Featurespiegelung:** Stellt die atomaren Bausteine für alle visuellen und logischen Elemente bereit.
2. **Domänenschicht (Domain Layer):**
    
    - **Verantwortlichkeiten:** UI-unabhängige Geschäftslogik.
        - `domain::theming`: Logik für das Erscheinungsbild, Design-Token-Verwaltung, dynamische Theme-Wechsel (Hell/Dunkel, Akzentfarben).
        - `domain::workspaces`: Verwaltung von Arbeitsbereichen ("Spaces"), Fensterzuweisung, Workspace-Orchestrierung und -Persistenz.
        - `domain::user_centric_services`: Logik für KI-Interaktionen (inkl. Einwilligungsmanagement für Datenkategorien wie `FileSystemRead`, `ClipboardAccess`), Benachrichtigungsverwaltung.
        - `domain::notifications_rules`: Regelbasierte, dynamische Verarbeitung von Benachrichtigungen.
        - `domain::global_settings_and_state_management`: Verwaltung globaler Desktop-Einstellungen.
        - `domain::window_management_policy`: Richtlinien für Fensterplatzierung, automatisches Tiling (Layouts: Spalten, Spiralen), Snapping, Fenstergruppierung, Gap-Management.
    - **Featurespiegelung:** Definiert _was_ personalisierbar ist (Themes, Akzente), _wie_ Arbeitsbereiche funktionieren (Spaces mit Icons, gepinnten Apps), _wie_ KI sicher und mit Zustimmung agiert und _welche_ Regeln für Fenster gelten.
3. **Systemschicht (System Layer):**
    
    - **Verantwortlichkeiten:** OS-Interaktion, technische Umsetzung der Domänenrichtlinien.
        - `system::compositor`: Smithay-basierter Wayland-Compositor (Implementierung von `xdg-shell`, `wlr-layer-shell-unstable-v1`, etc.), XWayland.
        - `system::input`: `libinput`-basierte Eingabeverarbeitung, Gestenerkennung, Seat-Management (`xkbcommon`).
        - `system::dbus`: `zbus`-Schnittstellen zu Systemdiensten (NetworkManager, UPower, logind, org.freedesktop.Notifications, org.freedesktop.secrets, PolicyKit).
        - `system::outputs`: Monitorkonfiguration (Auflösung, Skalierung, DPMS über `wlr-output-management`).
        - `system::audio`: PipeWire-Client (`pipewire-rs`) für Audio-Management.
        - `system::mcp`: MCP-Client (`mcp_client_rs`) für KI-Modell-Kommunikation.
        - `system::portals`: Backend für XDG Desktop Portals (FileChooser, Screenshot).
        - `system::window_mechanics`: Technische Umsetzung des Fenstermanagements (Positionierung, Anwendung von Tiling-Layouts, Fokus, Fensterdekorationen). Technische Basis für die "Intelligente Tab-Leiste".
    - **Featurespiegelung:** Ermöglicht flüssige Darstellung (Wayland), präzise Eingabe (`libinput`, Gesten), Integration mit Systemdiensten für Energie, Netzwerk, Sound (PipeWire) und sichere KI-Kommunikation (MCP). Setzt Fensterregeln (Tiling, Snapping) technisch um.
4. **Benutzeroberflächenschicht (User Interface Layer):**
    
    - **Verantwortlichkeiten:** Grafische Darstellung, Benutzerinteraktion (GTK4, `gtk4-rs`).
        - `ui::shell`:
            - **Kontroll-/Systemleiste(n) (PanelWidget):** Module für AppMenu, Workspace-Indikator, Uhr, System-Tray, Schnelleinstellungen, Benachrichtigungszentrum, Netzwerk-, Energie-, Audio-Indikatoren. _Elegante Leiste mit optionalem Leuchtakzent._
            - **Intelligente Tab-Leiste (SmartTabBarWidget):** Pro "Space", mit ApplicationTabWidgets für "angepinnte" Apps/Split-Views, aktive Tabs mit Akzentfarbe. _Moderne Tabs mit abgerundeten oberen Ecken._
            - **Schnelleinstellungs-Panel (QuickSettingsPanelWidget):** Ausklappbar für WLAN, Bluetooth, Lautstärke, Dark Mode.
            - **Workspace-Switcher (WorkspaceSwitcherWidget):** Adaptive linke Seitenleiste mit SpaceIconWidgets (Icons der gepinnten App oder benutzerdefiniert) für schnelle Navigation zwischen "Spaces", mit Hervorhebung des aktiven Space. _Bei Mouse-Over/Geste aufklappbar mit Namen/Vorschau._
            - **Schnellaktionsdock (QuickActionDockWidget):** Konfigurierbares Dock (schwebend/angedockt) für Apps, Dateien, Aktionen; intelligente Vorschläge, Tastaturbedienung.
            - **Benachrichtigungszentrum (NotificationCenterPanelWidget):** Anzeige von Benachrichtigungsliste und -historie.
        - `ui::control_center`: Modulare GTK4-Anwendung für alle Systemeinstellungen (Erscheinungsbild, Netzwerk, etc.) mit Live-Vorschau.
        - `ui::widgets`:
            - **Adaptive rechte Seitenleiste (RightSidebarWidget):** Optional, mit dezent transluzentem Hintergrund für informative Widgets (Uhr, Kalender, Wetter, Systemmonitor), per Drag & Drop anpassbar.
            - WidgetManagerService, WidgetPickerPopover.
        - `ui::window_manager_frontend`:
            - **Client-Side Decorations (CSD):** Logik (z.B. via `Gtk::HeaderBar`).
            - **Übersichtsmodus (OverviewModeWidget):** Fenster- und Workspace-Übersicht als interaktive Kacheln mit Live-Vorschau, Drag & Drop von Fenstern zwischen Spaces. _Hintergrund abgedunkelt/unscharf._
            - AltTabSwitcherWidget.
        - `ui::notifications_frontend`: **Pop-up-Benachrichtigungen (NotificationPopupWidget):** Dezent, im Dark Mode Stil mit Akzentfarbe für Dringlichkeit.
        - `ui::theming_gtk`: Anwendung von CSS-Stilen aus `domain::theming` via `GtkCssProvider`.
        - `ui::speed_dial`: GTK4-Implementierung der Startansicht für leere Workspaces mit Favoriten und intelligenten Vorschlägen.
        - `ui::command_palette`: GTK4-Implementierung der kontextuellen Befehlspalette (`Super+Space`).
    - **Featurespiegelung:** Setzt die gesamte beschriebene Nutzererfahrung um: dunkle Ästhetik mit Akzentfarben, Panel(s), intelligente Tab-Leiste, adaptive Seitenleisten mit Widgets, Workspace-Switcher, Schnellaktionsdock, Control Center, Speed-Dial, Übersichtsmodus und die kontextuelle Befehlspalette. Ermöglicht die Personalisierung und direkte Manipulation.

**III. Technologie-Stack (Verbindliche Auswahl)**

Rust, Meson, GTK4 (`gtk4-rs`), Smithay Toolkit, Wayland (xdg-shell, wlr-Protokolle), D-Bus (`zbus`), Model Context Protocol (MCP), `libinput`, PipeWire (`pipewire-rs`), Freedesktop Secret Service API, PolicyKit, Token-basiertes CSS-Theming, XDG Desktop Portals.

**IV. Entwicklungsrichtlinien (Verbindlich)**

- **Rust:** `rustfmt`, Rust API Guidelines, `thiserror` pro Modul, `Result<T,E>`, `tracing` für Logging, `async/await` (Tokio, GLib).
- **Allgemein:** Git (GitHub Flow), Conventional Commits, umfassende Tests (Unit, Integration, Compositor, UI), CI-Pipeline, detaillierte Dokumentation (rustdoc, Architektur, READMEs).

**V. Deployment-Überlegungen**

Native Pakete (.deb, .rpm), Flatpak (evaluieren), Integration mit Display Managern, `systemd` User Sessions, PAM, XDG Base Directory Specification. SemVer.
