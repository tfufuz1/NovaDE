

# Wayland-Schnittstellenspezifikation Fundamentale Protokollarchitektur

## WAYLAND-PROTOKOLL-FUNDAMENT

### Objektbasierte Nachrichtenschicht

Das Wayland-Protokoll implementiert ein objektbasiertes Nachrichtensystem mit eindeutigen Objektidentifikatoren. Jedes Protokollobjekt besitzt eine numerische ID, einen definierten Schnittstellentyp und eine Versionsnummer. Die Objektlebensdauer wird durch explizite Erstellungs- und Zerstörungsnachrichten verwaltet. Der Server verwaltet eine globale Objekttabelle, während Clients lokale Objektreferenzen pflegen.

### Synchrone und Asynchrone Nachrichtenverteilung

Wayland unterscheidet zwischen Anfragen (Client-zu-Server) und Ereignissen (Server-zu-Client). Anfragen werden synchron verarbeitet, während Ereignisse asynchron übertragen werden. Der Server garantiert die Reihenfolge von Ereignissen pro Objekt, nicht jedoch objektübergreifend. Nachrichten enthalten einen Opcode, der die spezifische Operation identifiziert, gefolgt von typisierten Argumenten.

### Atomare Zustandsübergänge

Der Compositor implementiert atomare Zustandsübergänge durch Frame-Callbacks und Commit-Zyklen. Surface-Zustandsänderungen werden in Puffern gesammelt und erst beim Commit-Aufruf atomar angewendet. Dies gewährleistet visuelle Konsistenz und verhindert Teilzustandsdarstellungen.

## SMITHAY-RUST-INTEGRATION

### Typ-sichere Protokollrepräsentation

Smithay generiert typsichere Rust-Strukturen aus Wayland-XML-Protokolldefinitionen. Jede Schnittstelle wird als Trait mit assoziierten Typen für Nachrichten und Zustände repräsentiert. Objektreferenzen werden durch generische Typparameter abstrahiert, die zur Kompilierzeit validiert werden.

### Callback-basierte Ereignisverteilung

Die Ereignisverarbeitung erfolgt über Rust-Closures, die an spezifische Objekttypen und Operationen gebunden sind. Der Dispatcher verwendet Pattern Matching, um eingehende Nachrichten den entsprechenden Handlern zuzuordnen. Fehlerbehandlung erfolgt durch Result-Typen, die sowohl Protokoll- als auch Systemfehler kapseln.

### Ressourcenverwaltung und Lebensdauer

Smithay implementiert RAII-Prinzipien für Wayland-Objekte. Ressourcen werden automatisch freigegeben, wenn ihre Rust-Wrapper-Objekte den Gültigkeitsbereich verlassen. Referenzzählung verhindert vorzeitige Freigabe von noch referenzierten Objekten.

## GTK-DESKTOP-INTEGRATION

### Layer-Shell-Protokoll für Desktopkomponenten

GTK-basierte Desktopumgebungen nutzen das wlr-layer-shell-Protokoll für Panels, Docks und Overlays. Dieses definiert vier Layer (background, bottom, top, overlay) mit expliziter Z-Order-Kontrolle. Jeder Layer kann exklusive Zonen definieren, die andere Fenster verdrängen.

### Input-Method-Framework-Integration

Die Texteingabe erfolgt über das zwp_text_input_v3-Protokoll. Input-Method-Editoren kommunizieren über einen separaten Kanal mit dem Compositor, der Preedit-Strings und Commit-Ereignisse an fokussierte Clients weiterleitet. Cursor-Rechtecke und Sprach-Hints werden bidirektional übertragen.

### Clipboard-Synchronisation und Datentypen

Die Zwischenablage verwendet das wl_data_device-Protokoll mit MIME-Type-basierten Formaten. Datenübertragung erfolgt über Pipe-Deskriptoren zwischen Processes. Selection-Angebote werden mit Prioritätslisten von unterstützten Formaten erstellt.

## PROTOKOLL-KOMPATIBILITÄT UND VERSIONIERUNG

### Abwärtskompatible Erweiterungen

Neue Protokollfeatures werden durch separate Schnittstellenversionen oder Erweiterungsschnittstellen implementiert. Clients müssen explizit höhere Versionen anfordern. Unbekannte Nachrichten werden ignoriert, um Kompatibilität zu gewährleisten.

### Capability-Discovery-Mechanism

Der Registry-Mechanismus ermöglicht dynamische Protokollerkennung. Globale Objekte werden mit Namen, Schnittstelle und Version angekündigt. Clients können verfügbare Capabilities abfragen und selektiv binden.

### Fehlerbehandlung und Debugging

Protokollfehler resultieren in Verbindungsabbruch mit spezifischen Fehlercodes. Debug-Informationen werden über das Logging-Framework übertragen. Smithay bietet zusätzliche Validierungsebenen für Entwicklungsumgebungen.

# Wayland-Schnittstellenspezifikation Teil 2: Surface-Management und Buffer-Protokolle

## SURFACE-ZUSTANDSAUTOMAT DEFINITIONEN

### Surface-Lebensdauer-Zustände

Surface-Objekte durchlaufen sechs definierte Zustände: CREATED → CONFIGURED → MAPPED → ACTIVE → UNMAPPED → DESTROYED. Zustandsübergänge erfolgen ausschließlich durch spezifische Nachrichten: wl_surface.commit, xdg_surface.configure, xdg_toplevel.configure. Illegale Übergänge resultieren in PROTOCOL_ERROR mit Code 0x01.

### Buffer-Attachment-Semantik

Buffer-Attachment erfordert exakte Sequenz: wl_surface.attach → wl_surface.damage → wl_surface.commit. Zwischen attach und commit sind Buffer-Inhalte unveränderlich. Double-buffering implementiert zwei Buffer-Slots: PENDING und CURRENT. Commit-Operation transferiert PENDING zu CURRENT atomar.

### Damage-Accumulation-Algorithmus

Damage-Rechtecke akkumulieren zwischen Commits in pixelgenauer Koordinatenraumdarstellung. Overlappende Rechtecke werden NICHT automatisch vereinigt. Maximale Damage-Rechteck-Anzahl: 16 pro Commit-Zyklus. Overflow resultiert in Vollbild-Damage-Aktivierung.

## COMPOSITOR-PROTOKOLL-ZUSTANDSMASCHINE

### Frame-Callback-Scheduling

Frame-Callbacks registrieren sich mittels wl_surface.frame mit eindeutiger Callback-ID. Compositor sendet wl_callback.done bei nächstem Vertical-Blank-Intervall. Callback-Objekte werden nach done-Ereignis automatisch zerstört. Maximale parallele Callbacks pro Surface: 1.

### Subsurface-Hierarchie-Verwaltung

Subsurfaces definieren Parent-Child-Beziehungen mit expliziter Z-Order. Zustandsmodifikationen: wl_subsurface.place_above, wl_subsurface.place_below. Synchronisation-Modi: SYNCHRONIZED (Parent-gekoppelt) oder DESYNCHRONIZED (unabhängig). Mode-Wechsel erfolgt über wl_subsurface.set_sync/set_desync.

### Transform-Matrix-Berechnung

Surface-Transformationen akkumulieren durch Matrix-Multiplikation: Scale × Rotation × Translation. Buffer-Transform-Werte: 0°, 90°, 180°, 270° plus Flip-Flags. Skalierung erfordert wp_viewport mit exakten Source-Rectangle und Destination-Size-Parametern.

## SMITHAY-TYPSYSTEM-MAPPING

### Objektreferenz-Typologie

Wayland-Objekte mappen auf Rust-Typen: wl_surface → Surface<R: Role>, wl_buffer → Buffer<T: BufferType>. Generische Constraints validieren Objektkompatibilität zur Kompilierzeit. Invalid-Objektzugriff resultiert in panic! oder Option::None je nach Kontext.

### Nachrichtendeserialisierung-Schema

Eingehende Nachrichten deserialisieren über FromRawFd-Implementierungen für File-Deskriptoren und byteorder-Crate für Integer-Endianness. String-Parameter erfordern UTF-8-Validierung mit expliziter Fehlerbehandlung via WaylandError::InvalidUtf8.

### Callback-Handler-Dispatch-Logik

Handler-Funktionen implementieren trait WaylandHandler<T> mit einzelner handle-Methode. Dispatch erfolgt via match-Expression über Message-Enum-Variants. Handler-Registrierung verwendet TypeId-basierte HashMap für O(1)-Lookup-Performance.

## XDG-SHELL-PROTOKOLL-IMPLEMENTIERUNG

### Toplevel-Konfiguration-Lifecycle

XDG-Toplevel-Fenster erfordern configure-acknowledge-Zyklus: Compositor sendet xdg_toplevel.configure → Client antwortet xdg_surface.ack_configure → wl_surface.commit aktiviert Konfiguration. Nicht-acknowledgierte Konfigurationen bleiben pending ohne Timeout.

### Popup-Positionierung-Constraints

XDG-Popup-Positionierung verwendet xdg_positioner mit expliziten Constraints: anchor_rect, size, anchor, gravity, constraint_adjustment. Constraint-Resolution erfolgt clientseitig vor popup-Erstellung. Invalid-Positioner resultiert in INVALID_POSITIONER-Protokollfehler.

### Surface-Role-Zuweisungen

Surface-Rollen sind exklusiv und unveränderlich: wl_surface kann entweder xdg_toplevel ODER xdg_popup ODER wl_subsurface zugewiesen werden. Mehrfachzuweisung resultiert in ROLE-Protokollfehler. Role-Zuweisung erfolgt bei entsprechender get_*-Nachricht.

## BUFFER-MANAGEMENT-PRÄZISION

### SHM-Buffer-Memory-Layout

Shared-Memory-Buffer erfordern exakte Memory-Mapping: fd → mmap mit PROT_READ|PROT_WRITE. Pixel-Format definiert Byte-Order: ARGB8888 = [B,G,R,A] little-endian. Stride berechnet sich: width × bytes_per_pixel + row_alignment_padding.

### DMA-BUF-Handle-Übertragung

DMA-BUF-Import erfolgt über zwp_linux_dmabuf_v1 mit Modifier-Support. File-Deskriptor-Transfer nutzt SCM_RIGHTS über Unix-Domain-Socket. Buffer-Freigabe erfordert explizites wl_buffer.destroy nach Usage-Completion-Callback.

# Wayland-Schnittstellenspezifikation Teil 3: Input-Event-Verarbeitung und Hardware-Integration

## INPUT-EVENT-ZUSTANDSMASCHINEN

### Pointer-Event-Sequenz-Automaten

Pointer-Events folgen strikter Sequenz: wl_pointer.enter(serial=N) → wl_pointer.motion(time=T, surface_x=X, surface_y=Y) → wl_pointer.button(serial=N+1, time=T+Δ, button=BTN_LEFT, state=PRESSED|RELEASED) → wl_pointer.leave(serial=N+2). Serial-Nummern sind monoton steigend. Motion-Events limitiert auf 1000Hz maximum. Button-State-Werte: 0=RELEASED, 1=PRESSED.

### Keyboard-Focus-State-Machine

Keyboard-Focus: UNFOCUSED → wl_keyboard.enter(serial=S, surface=SURF, keys=ARRAY) → FOCUSED → wl_keyboard.leave(serial=S+N) → UNFOCUSED. Keys-Array enthält aktuell gedrückte Scancodes als uint32. Modifiers-Event sendet state: mods_depressed, mods_latched, mods_locked, group jeweils als uint32-Bitmask.

### Touch-Point-Lifecycle-Definition

Touch-Points: wl_touch.down(id=ID, serial=S, surface=SURF, x=X, y=Y) → wl_touch.motion(time=T, id=ID, x=X, y=Y)* → wl_touch.up(time=T, id=ID). Touch-ID-Range: 0-15. Koordinaten in fixed-point-Format: Wert × 256. Simultane Touch-Points maximum: 10.

## TIMING-CONSTRAINTS-SPEZIFIKATION

### Event-Timestamp-Präzision

Alle Input-Events enthalten timestamp in Millisekunden seit Compositor-Start als uint32. Timestamp-Overflow bei 4294967295ms (49.7 Tage). Relative-Motion-Events zusätzlich mit Mikrosekunden-Präzision über wp_relative_pointer_v1. Delta-Werte als wl_fixed (Festkomma × 256).

### Frame-Timing-Garantien

Frame-Callbacks garantieren Timing innerhalb 16.67ms (60Hz) ± 1ms. Presentation-Feedback über wp_presentation liefert: tv_sec (uint32), tv_nsec (uint32), refresh (uint32 in Nanosekunden), seq_hi+seq_lo (64-bit frame counter), flags (uint32 bitmask: VSYNC=1, HW_CLOCK=2, HW_COMPLETION=4, ZERO_COPY=8).

### Input-Rate-Limiting

Pointer-Motion-Events: Maximum 1000 Events/Sekunde. Scroll-Events über wl_pointer.axis: Maximum 120 detents/Sekunde, Wert in wl_fixed-Format. Discrete-Scroll über wl_pointer.axis_discrete: Integer-Werte -3 bis +3 pro Event.

## SMITHAY-HARDWARE-ABSTRAKTION

### DRM-Backend-Konfiguration

DRM-Backend erfordert Card-Path (/dev/dri/card0), Connector-ID (uint32), Mode-Struct mit: hdisplay, vdisplay, clock, flags. Atomic-Modesetting über Property-IDs: CRTC_ID, FB_ID, SRC_X/Y/W/H (16.16 fixed-point), CRTC_X/Y/W/H (uint32). Plane-Types: PRIMARY=1, CURSOR=2, OVERLAY=3.

### Buffer-Import-Validation

DMA-BUF-Import validiert: fourcc (uint32 format), width/height (uint32, maximum 16384×16384), modifier (uint64), plane_count (1-4). Plane-Offsets als uint32-Array, Strides als uint32-Array. File-Descriptor-Validation: stat() st_size >= offset + stride × height.

### GPU-Memory-Management

EGL-Integration: eglCreateImageKHR mit EGL_LINUX_DMA_BUF_EXT. Attribute-Array: EGL_WIDTH, EGL_HEIGHT, EGL_LINUX_DRM_FOURCC_EXT, EGL_DMA_BUF_PLANE0_FD, EGL_DMA_BUF_PLANE0_OFFSET, EGL_DMA_BUF_PLANE0_PITCH. Texture-Binding über glEGLImageTargetTexture2DOES.

## PROTOKOLL-ERROR-CODES-MATRIX

### Surface-Protocol-Errors

wl_surface errors: INVALID_SCALE=0, INVALID_TRANSFORM=1, INVALID_SIZE=2, INVALID_OFFSET=3. xdg_surface errors: NOT_CONSTRUCTED=1, ALREADY_CONSTRUCTED=2, UNCONFIGURED_BUFFER=3. xdg_toplevel errors: INVALID_RESIZE_EDGE=0, INVALID_PARENT=1, INVALID_SIZE=2.

### Buffer-Protocol-Errors

wl_shm errors: INVALID_FORMAT=0, INVALID_STRIDE=1, INVALID_FD=2, INVALID_SIZE=3. zwp_linux_dmabuf errors: PLANE_IDX=0, PLANE_SET=1, INCOMPLETE=2, INVALID_FORMAT=3, INVALID_DIMENSIONS=4, OUT_OF_BOUNDS=5, INVALID_WL_BUFFER=6.

### Input-Protocol-Errors

wl_seat errors: MISSING_CAPABILITY=0. wl_pointer errors: ROLE=0. wl_keyboard errors: NO_KEYMAP=0. wl_touch errors: ROLE=0. wp_text_input errors: ROLE=0, NO_SURFACE=1.

## GTK-COMPOSITOR-INTEGRATION-PARAMETER

### Layer-Shell-Z-Order-Berechnung

Layer-Shell-Layers mit Z-Values: BACKGROUND=-1000, BOTTOM=-500, TOP=500, OVERLAY=1000. Anchor-Bitmask: TOP=1, BOTTOM=2, LEFT=4, RIGHT=8. Exclusive-Zone: -1=full-exclusion, 0=no-exclusion, >0=pixel-count. Keyboard-Interactivity: NONE=0, EXCLUSIVE=1, ON_DEMAND=2.

### Window-Management-State-Tracking

XDG-Toplevel-States als Bitmask: MAXIMIZED=1, FULLSCREEN=2, RESIZING=4, ACTIVATED=8, TILED_LEFT=16, TILED_RIGHT=32, TILED_TOP=64, TILED_BOTTOM=128. Configure-Events enthalten: width, height (int32, 0=client-decides), states (wl_array von uint32).

### Decoration-Protocol-Implementation

Server-Side-Decorations via zxdg_decoration_manager_v1: MODE_CLIENT_SIDE=1, MODE_SERVER_SIDE=2. Client-Preferred-Modes via zxdg_toplevel_decoration_v1.set_mode. Configure-Response via zxdg_toplevel_decoration_v1.configure(mode).

Diese Spezifikation dient der präzisen Beschreibung der Wayland-Protokolle in Konjunktion mit Smithay/Rust, konzipiert für den Einsatz durch autonome Programmieragenten im Kontext der Entwicklung einer neuartigen GTK-Desktopumgebung. Der Fokus liegt auf maximaler technischer Granularität und Effizienz zur Gewährleistung einer unmissverständlichen Interpretation.

## 1. Wayland-Protokoll-Fundament

### 1.1 Objektbasierte Nachrichtenschicht

Das Wayland-Protokoll implementiert ein strikt objektorientiertes Nachrichtensystem. Jedem Protokollobjekt wird eine eindeutige, nicht-negative `uint32`-ID (`object_id`), ein spezifischer Schnittstellentyp (exemplarisch sei `wl_surface`, `wl_seat`, `wl_pointer` genannt) und eine Versionsnummer, ebenfalls vom Typ `uint32`, zugewiesen. Die Lebensdauer dieser Objekte wird durch explizite Client-Anfragen, namentlich `wl_registry.bind` zur Kreation, und Client-Anfragen (`wl_proxy_destroy`) oder Server-Ereignisse zur Destruktion gesteuert. Der Wayland-Server unterhält eine zentrale, globale Objekttabelle, welche sämtliche aktiven Objekte nebst ihrer Metadaten umfasst. Clients ihrerseits führen lokale Proxy-Objekte, welche die serverseitigen Entitäten repräsentieren und die Kommunikationsabstraktion bewerkstelligen. Die konsistente Zuweisung und Verwaltung dieser Identifikatoren ist für die Integrität der Protokollkommunikation von fundamentaler Bedeutung.

### 1.2 Synchrone und Asynchrone Nachrichtenverteilung

Das Wayland-Protokoll differenziert fundamental zwischen zwei Kommunikationsrichtungen, deren Charakteristika nachfolgend dargelegt werden:

- **Anfragen (Client-zu-Server):** Diese Nachrichten, als `requests` bezeichnet, werden vom Client initiiert und erfahren in der Regel eine synchrone Verarbeitung auf dem Server. Als Beispiele hierfür können `wl_surface.attach` zur Zuweisung eines Puffers an eine Oberfläche oder `wl_surface.commit` zur Applikation ausstehender Oberflächenzustandsänderungen angeführt werden. Die Abarbeitung dieser Anfragen erfolgt in der Reihenfolge ihres Empfangs.
    
- **Ereignisse (Server-zu-Client):** Diese Nachrichten, als `events` designiert, werden vom Server generiert und asynchron an den Client übertragen. Exemplarisch seien `wl_pointer.motion` für Mausbewegungen oder `wl_keyboard.key` für Tastatureingaben erwähnt. Der Server gewährleistet die sequentielle Reihenfolge von Ereignissen pro individuellem Objekt, jedoch nicht zwingend objektübergreifend. Die zugrundeliegende Transportmechanik bedient sich Unix-Domain-Sockets, über welche serialisierte Nachrichten, bestehend aus einem `uint16`-Opcode zur Identifizierung der spezifischen Operation, gefolgt von typisierten Argumenten (`int32`, `uint32`, `string`, `array`, `fd`, `new_id`, `object`), übermittelt werden. Ereignisse werden serverseitig gepuffert und in Batches an den Client gesendet, um den Overhead zu minimieren.
    

### 1.3 Atomare Zustandsübergänge

Der Compositor implementiert atomare Zustandsübergänge, um die visuelle Konsistenz zu gewährleisten und die Manifestation inkonsistenter Teildarstellungen zu verhindern. Dies wird primär durch Frame-Callbacks und den `wl_surface.commit`-Zyklus realisiert. Sämtliche Modifikationen des Zustands einer `wl_surface` (beispielsweise die Zuweisung eines neuen Puffers mittels `wl_surface.attach`, die Definition beschädigter Regionen mittels `wl_surface.damage`, oder die Applikation einer Transformation mittels `wl_surface.set_buffer_transform`) werden initial in einem "Pending State" auf der Client-Seite kumuliert. Erst der explizite Aufruf von `wl_surface.commit` übermittelt diese kumulierten Änderungen atomar an den Compositor. Der Compositor wendet diese Modifikationen sodann als eine einzige, unteilbare Transaktion an. Für explizite Synchronisationspunkte kann von Clients `wl_display.sync` verwendet werden, welches einen `wl_callback`-Event auslöst, sobald sämtliche vorherigen Anfragen prozessiert wurden. Dies ist von entscheidender Bedeutung zur Vermeidung visuellen Tearings, indem sichergestellt wird, dass ein Frame nur dann vollständig gerendert wird, wenn alle seine Komponenten zur Verfügung stehen.

## 2. Smithay-Rust-Integration

### 2.1 Typ-sichere Protokollrepräsentation

Smithay nutzt den `wayland-scanner`, ein proprietäres Werkzeug aus dem Wayland-Projekt, zur Generierung hochgradig typsicherer Rust-Strukturen und -Traits aus den formalen Wayland-XML-Protokolldefinitionen (z.B. `wayland.xml`, `xdg-shell.xml`). Jede Wayland-Schnittstelle (exemplarisch `wl_surface`, `wl_seat`) wird als ein spezifisches Rust-Trait repräsentiert (z.B. `wayland_server::protocol::wl_surface::WlSurface`), welches assoziierte Typen für die Nachrichten (`Requests`, `Events`) und deren Argumente definiert. Objektreferenzen werden durch generische Typparameter abstrahiert (z.B. `<R: Role>` für `Surface<R>`), was die Validierung der Korrektheit der Objektverwendung bereits zur Kompilierungszeit ermöglicht. Dies präveniert eine signifikante Klasse von Laufzeitfehlern, welche bei weniger typsicheren Bindungen auftreten könnten, indem beispielsweise die Assoziation einer `wl_surface` ausschließlich mit einer gültigen Rolle (z.B. `xdg_toplevel`) sichergestellt wird. Ungültige Objektzugriffe oder Typinkonsistenzen führen im Entwicklungsmodus zu einem `panic!` oder in robusten Anwendungen zu `Option::None` / `Result::Err`, wodurch eine eindeutige Fehlerbehandlung ermöglicht wird.

### 2.2 Callback-basierte Ereignisverteilung

Die Ereignisverarbeitung in Smithay basiert auf einem flexiblen Callback-System unter Verwendung von Rust-Closures. Diese Closures werden an spezifische Objekttypen und deren zugehörige Operationen gebunden. Der Kern der Ereignisverteilung ist der `Dispatch`-Trait, dessen Implementierung für jeden Typ obligatorisch ist, der Wayland-Nachrichten verarbeiten soll. Ein `EventLoop` in Smithay liest eingehende Nachrichten vom Wayland-Socket und nutzt Pattern Matching über `Message`-Enum-Variants, um die eingehenden Nachrichten den entsprechend registrierten Handlern zuzuordnen. Die Fehlerbehandlung erfolgt konsequent über Rusts `Result`-Typen, welche sowohl Protokollfehler (z.B. `wayland_server::protocol::WlError`) als auch zugrundeliegende Systemfehler (z.B. `std::io::Error` bei Socket-Operationen) kapseln. Diese `Result`-Typen werden durch die Aufrufkette propagiert, was eine granulare und explizite Fehlerbehandlung auf jeder Ebene der Anwendung ermöglicht.

### 2.3 Ressourcenverwaltung und Lebensdauer

Smithay implementiert das RAII-Prinzip (Resource Acquisition Is Initialization) für Wayland-Objekte, was eine robuste und automatische Ressourcenverwaltung ermöglicht. Wenn Rust-Wrapper-Objekte, welche Wayland-Objekte repräsentieren, ihren Gültigkeitsbereich verlassen, wird automatisch deren `Drop`-Implementierung aufgerufen. Diese `Drop`-Implementierung sendet sodann die entsprechende `wl_proxy_destroy`-Nachricht an den Server, um das serverseitige Wayland-Objekt freizugeben. Für Szenarien mit geteiltem Besitz von Wayland-Objekten werden intelligente Zeiger wie `Arc` (Atomic Reference Counted) und `Rc` (Reference Counted) verwendet. Diese implementieren Referenzzählung, welche präveniert, dass Objekte vorzeitig freigegeben werden, solange noch gültige Referenzen auf sie existieren. Die korrekte Verwendung von starken (`Arc<T>`) und schwachen (`Weak<T>`) Referenzen ist entscheidend, um Zyklen in Objektgraphen zu vermeiden, die zu Memory Leaks führen könnten.

## 3. GTK-Desktop-Integration

### 3.1 Layer-Shell-Protokoll für Desktopkomponenten

GTK-basierte Desktopumgebungen nutzen das `wlr-layer-shell`-Protokoll (oft in Konjunktion mit `zxdg_shell_v6` oder `xdg_shell` und `zxdg_output_manager_v1`) zur Positionierung von Desktop-Komponenten wie Panels, Docks, Desktophintergründen und Overlays. Dieses Protokoll definiert vier hierarchische Layer mit expliziter Z-Order-Kontrolle: `background` (Z=−1000), `bottom` (Z=−500), `top` (Z=500), und `overlay` (Z=1000). Jede Layer-Shell-Oberfläche ist befähigt, "exklusive Zonen" zu definieren, welche andere reguläre Fenster (XDG-Toplevels) verdrängen, um Raum für UI-Elemente zu schaffen. Die Positionierung innerhalb eines Layers erfolgt über `anchor`-Bitmasken, welche Kombinationen wie `TOP | LEFT` (oben links) oder `BOTTOM | RIGHT` (unten rechts) ermöglichen. Dies erlaubt eine präzise und flexible Anordnung von festen UI-Elementen, welche nicht von regulären Fenstern überdeckt werden sollen.

### 3.2 Input-Method-Framework-Integration

Die Texteingabe in Wayland-Umgebungen erfolgt über das `zwp_text_input_v3`-Protokoll, welches eine robuste Integration von Input-Method-Editoren (IMEs) ermöglicht. IMEs kommunizieren über einen separaten Kanal mit dem Compositor, welcher wiederum die Preedit-Strings (temporäre, unbestätigte Texteingaben, z.B. bei der Eingabe komplexer Zeichen in asiatischen Sprachen) und Commit-Ereignisse (bestätigte Texteingaben) an den aktuell fokussierten Client weiterleitet. Das Protokoll definiert eine Zustandsmaschine mit Nachrichten wie `activate` (IME wird für ein Surface aktiviert), `deactivate` (IME wird deaktiviert), `set_surrounding_text` (IME erhält Kontext des umgebenden Textes), `set_cursor_rectangle` (IME erhält die Position des Cursors). Die `cursor_rectangle` ist dabei entscheidend, damit der IME seine eigene Benutzeroberfläche (z.B. Kandidatenlisten) präzise relativ zum Texteingabefeld des Clients positionieren kann. Eine bidirektionale Übertragung von Sprach-Hints (`set_content_type`) optimiert die IME-Auswahl.

### 3.3 Clipboard-Synchronisation und Datentypen

Die Zwischenablage (Clipboard) und Drag-and-Drop-Funktionalität werden über das `wl_data_device`-Protokoll abgewickelt. Die Datenübertragung erfolgt MIME-Type-basiert, was die Kompatibilität zwischen verschiedenen Anwendungen sicherstellt. Wenn ein Client Daten in die Zwischenablage kopiert oder einen Drag-Vorgang initiiert, erstellt er ein `wl_data_source`-Objekt und offeriert eine Liste unterstützter MIME-Typen (z.B. `text/plain`, `image/png`, `application/x-wayland-custom-data`). Wenn ein anderer Client die Daten einfügen oder empfangen möchte, erstellt er ein `wl_data_offer`-Objekt. Die eigentliche Datenübertragung erfolgt über Pipe-Deskriptoren (`fd`), welche über Unix-Domain-Sockets zwischen den Prozessen ausgetauscht werden. Dies stellt einen sicheren und effizienten Mechanismus dar, insbesondere für große Datenmengen. Der Drag-and-Drop-Prozess involviert zusätzlich Nachrichten wie `wl_data_device.start_drag` und `wl_data_source.send`.

## 4. Surface-Management und Buffer-Protokolle

### 4.1 Surface-Lebensdauer-Zustände

Surface-Objekte durchlaufen einen klar definierten Lebenszyklus, welcher sechs Hauptzustände umfasst:

1. `CREATED`: Das Surface-Objekt existiert, ist jedoch noch nicht konfiguriert oder sichtbar.
    
2. `CONFIGURED`: Das Surface hat mindestens eine `xdg_surface.configure`-Nachricht empfangen und diese mittels `xdg_surface.ack_configure` bestätigt. Hierdurch werden die initiale Größe und weitere Eigenschaften festgelegt.
    
3. `MAPPED`: Das Surface ist auf dem Bildschirm sichtbar und wird vom Compositor gerendert.
    
4. `ACTIVE`: Das Surface ist nicht nur gemappt, sondern auch interaktiv und befähigt, Eingaben zu empfangen.
    
5. `UNMAPPED`: Das Surface ist nicht länger sichtbar, das Objekt existiert jedoch weiterhin.
    
6. DESTROYED: Das Surface-Objekt wurde vollständig vom Client zerstört und seine Ressourcen freigegeben.
    
    Zustandsübergänge erfolgen ausschließlich durch spezifische, protokollierte Nachrichten wie wl_surface.commit, xdg_surface.configure und xdg_toplevel.configure. Jeder illegale Übergang, welcher den Protokollregeln zuwiderläuft, resultiert in einem PROTOCOL_ERROR mit dem Code 0x01, was zum sofortigen Verbindungsabbruch führt.
    

### 4.2 Buffer-Attachment-Semantik

Das Anhängen eines Buffers an ein Surface erfordert eine exakte und atomare Sequenz von Operationen: `wl_surface.attach` → `wl_surface.damage` (oder `wl_surface.damage_buffer`) → `wl_surface.commit`. Das `wl_buffer`-Objekt repräsentiert den Speicherbereich, welcher die Pixeldaten enthält. Zwischen dem `attach`-Aufruf und dem `commit`-Aufruf sind die Inhalte des Buffers für den Compositor unveränderlich. Wayland implementiert ein Double-Buffering-Schema, bei welchem jede `wl_surface` über zwei logische Buffer-Slots verfügt: einen `PENDING`-Slot für den nächsten Frame und einen `CURRENT`-Slot für den aktuell angezeigten Frame. Die `commit`-Operation transferiert den Inhalt des `PENDING`-Slots atomar in den `CURRENT`-Slot, wodurch sichergestellt wird, dass der Compositor stets einen vollständigen und konsistenten Frame rendert und keine partiellen oder unfertigen Updates darstellt. Der `wl_buffer.release`-Event wird vom Compositor gesendet, sobald der Buffer nicht länger benötigt wird und vom Client wiederverwendet werden kann.

### 4.3 Damage-Accumulation-Algorithmus

Damage-Rechtecke (`wl_surface.damage` und `wl_surface.damage_buffer`) akkumulieren zwischen den `commit`-Operationen in einem pixelgenauen Koordinatenraum, welcher sich auf die Surface-lokalen Pixel bezieht. Dies impliziert, dass Clients lediglich die Bereiche ihrer Oberfläche als "beschädigt" markieren müssen, welche sich tatsächlich geändert haben, anstatt stets die gesamte Oberfläche neu zu zeichnen. Überlappende Damage-Rechtecke werden vom Compositor **nicht** automatisch vereinigt; die Optimierung der Damage-Regionen obliegt dem Client. Die maximale Anzahl von Damage-Rechtecken, welche pro Commit-Zyklus übermittelt werden können, beträgt 16. Eine Überschreitung dieser Anzahl führt zur Aktivierung eines Vollbild-Damages, was die Rendering-Effizienz beeinträchtigen kann. Clients können ferner `wl_surface.set_opaque_region` und `wl_surface.set_input_region` verwenden, um dem Compositor mitzuteilen, welche Bereiche der Oberfläche undurchsichtig sind und welche für Eingaben relevant sind, was weitere Optimierungen ermöglicht.

## 5. Compositor-Protokoll-Zustandsmaschine

### 5.1 Frame-Callback-Scheduling

Frame-Callbacks werden mittels `wl_surface.frame` mit einer eindeutigen `uint32`-Callback-ID registriert. Dies stellt den primären Mechanismus für Clients dar, um ihre Rendering-Operationen mit dem Compositor und der Bildwiederholrate des Displays zu synchronisieren. Der Compositor sendet den `wl_callback.done`-Event (mit der ursprünglichen Callback-ID) beim nächsten Vertical-Blank-Intervall (VSync), kurz vor der Anzeige des nächsten Frames auf dem Display. Dies signalisiert dem Client, dass der optimale Zeitpunkt für das Rendern und Committen des nächsten Frames erreicht ist. Callback-Objekte sind "Einweg"-Objekte und werden nach dem Empfang des `done`-Ereignisses automatisch zerstört. Die maximale Anzahl paralleler Callbacks pro Surface ist auf 1 beschränkt, was eine eindeutige Synchronisationskette gewährleistet.

### 5.2 Subsurface-Hierarchie-Verwaltung

Subsurfaces (`wl_subsurface`) ermöglichen die Definition von Parent-Child-Beziehungen zwischen `wl_surface`-Objekten, wodurch eine hierarchische Komposition von Oberflächen realisiert wird. Sie definieren eine explizite Z-Order relativ zu ihrem Parent-Surface. Zustandsmodifikationen erfolgen über Nachrichten wie `wl_subsurface.place_above` und `wl_subsurface.place_below`, um die Stapelreihenfolge zu manipulieren, sowie `wl_subsurface.set_position` zur Festlegung der relativen Position. Subsurfaces können in zwei Synchronisationsmodi betrieben werden:

- `SYNCHRONIZED`: In diesem Modus sind die Subsurfaces an den `commit`-Zyklus ihres Parent-Surfaces gekoppelt. Ein `commit` auf dem Parent-Surface wendet atomar auch die ausstehenden Änderungen sämtlicher synchronisierten Kind-Subsurfaces an.
    
- DESYNCHRONIZED: In diesem Modus können Subsurfaces unabhängig von ihrem Parent-Surface committet und aktualisiert werden.
    
    Der Moduswechsel erfolgt über wl_subsurface.set_sync und wl_subsurface.set_desync. Die korrekte Verwaltung dieser Hierarchien und Synchronisationsmodi ist für die Realisierung komplexer UI-Elemente wie Tooltips, Menüs oder Popups von entscheidender Bedeutung.
    

### 5.3 Transform-Matrix-Berechnung

Surface-Transformationen akkumulieren durch eine Kette von Matrix-Multiplikationen: Skalierung × Rotation × Translation. Das Wayland-Protokoll offeriert `wl_surface.set_buffer_transform` und `wl_surface.set_buffer_scale` zur Applikation dieser Transformationen. `set_buffer_transform` akzeptiert Werte wie 0∘, 90∘, 180∘, 270∘ sowie Flip-Flags (horizontal/vertikal). `set_buffer_scale` definiert einen Skalierungsfaktor für den Bufferinhalt. Für komplexere Skalierungs- und Cropping-Anforderungen kommt das `wp_viewport` Protokoll (aus `viewport-v1`) zum Einsatz. Dieses ermöglicht Clients, einen exakten Quellrechteck (`source_rectangle`) innerhalb des Buffers und eine Zielgröße (`destination_size`) auf der Oberfläche anzugeben. Der Compositor skaliert und schneidet den Bufferinhalt sodann entsprechend zu, was eine präzise Kontrolle über die Darstellung von Buffer-Inhalten ermöglicht.

## 6. XDG-Shell-Protokoll-Implementierung

### 6.1 Toplevel-Konfiguration-Lifecycle

XDG-Toplevel-Fenster (Standard-Anwendungsfenster) erfordern einen strikten Configure-Acknowledge-Zyklus zur Zustandsynchronisation. Wenn der Compositor eine Konfiguration für ein Toplevel-Fenster sendet (mittels `xdg_toplevel.configure`), enthält diese Nachricht eine `serial`-Nummer (`uint32`). Der Client ist verpflichtet, diese Konfiguration zu empfangen, zu prozessieren und innerhalb eines `wl_surface.commit`-Aufrufs mit `xdg_surface.ack_configure(serial)` zu bestätigen. Diese `serial`-Nummer ist entscheidend, um die korrekte Konfigurationsbestätigung durch den Client zu gewährleisten. Nicht-acknowledgierte Konfigurationen verbleiben auf der Compositor-Seite im "pending"-Zustand und werden nicht angewendet, was zu einem desynchronisierten Zustand zwischen Client und Compositor führen kann. Der Compositor kann in diesem Fall weitere Konfigurationen zurückhalten oder die Verbindung terminieren.

### 6.2 Popup-Positionierung-Constraints

XDG-Popup-Positionierung (`xdg_popup`) verwendet das `xdg_positioner`-Objekt mit expliziten Constraints, um die Platzierung von Popups (exemplarisch Kontextmenüs, Tooltips) zu steuern. Die wichtigsten Attribute sind:

- `anchor_rect`: Ein Rechteck (`int32` x, y, width, height) auf dem Parent-Surface, welches als Referenzpunkt für die Positionierung dient.
    
- `size`: Die gewünschte Größe des Popups (`int32` width, height).
    
- `anchor`: Eine Bitmaske (`uint32`), welche den Ankerpunkt des `anchor_rect` definiert (z.B. `XDG_POSITIONER_ANCHOR_TOP`, `XDG_POSITIONER_ANCHOR_BOTTOM_LEFT`).
    
- `gravity`: Eine Bitmaske (`uint32`), welche die bevorzugte Ausrichtung des Popups relativ zum Ankerpunkt definiert (z.B. `XDG_POSITIONER_GRAVITY_TOP`, `XDG_POSITIONER_GRAVITY_BOTTOM`).
    
- constraint_adjustment: Eine Bitmaske (uint32), welche die zulässigen Anpassungen der Position durch den Compositor spezifiziert, falls die bevorzugte Position außerhalb des Bildschirms liegt (z.B. XDG_POSITIONER_CONSTRAINT_ADJUSTMENT_FLIP_X).
    
    Die Constraint-Resolution erfolgt clientseitig vor der Popup-Erstellung, jedoch kann der Compositor die finale Position adaptieren. Ein ungültiger Positioner resultiert in einem INVALID_POSITIONER-Protokollfehler.
    

### 6.3 Surface-Role-Zuweisungen

Die Zuweisung von Rollen zu `wl_surface`-Objekten ist im Wayland-Protokoll streng definiert: Eine `wl_surface` kann stets nur EINE Rolle zugewiesen bekommen, und diese Rolle ist für die Lebensdauer des Surface unveränderlich. Eine `wl_surface` kann entweder ein `xdg_toplevel` (reguläres Fenster), ein `xdg_popup` (temporäres, transientes Fenster) ODER ein `wl_subsurface` (Kind-Oberfläche) sein. Eine Mehrfachzuweisung oder der Versuch, die Rolle eines Surfaces nachträglich zu modifizieren, führt zu einem `ROLE`-Protokollfehler, welcher die Verbindung terminiert. Die Rollenzuweisung erfolgt implizit, wenn der Client die entsprechende `get_*-Nachricht` (z.B. `xdg_surface.get_toplevel`) aufruft, welche ein neues Objekt der spezifischen Rolle zurückgibt. Dieses Design vereinfacht die Logik des Compositors erheblich, da dieser sich nicht mit dynamischen Rollenwechseln oder komplexen Zuständen auseinandersetzen muss.

## 7. Buffer-Management-Präzision

### 7.1 SHM-Buffer-Memory-Layout

Shared-Memory-Buffer (`wl_shm`) erfordern ein exaktes Memory-Mapping zwischen Client und Compositor. Der Client erstellt einen `wl_shm_pool` und mappt einen File-Deskriptor (`fd`) in seinen Adressraum mittels `mmap` mit `PROT_READ|PROT_WRITE`. Das Pixel-Format (`wl_shm_format` Enum, z.B. `WL_SHM_FORMAT_ARGB8888`, `WL_SHM_FORMAT_XRGB8888`) definiert die Byte-Order und die Kanalbelegung (z.B. `ARGB8888` = `[B,G,R,A]` little-endian). Der `stride` (Anzahl der Bytes pro Zeile) berechnet sich als `width` × `bytes_per_pixel` + `row_alignment_padding`. Das `row_alignment_padding` ist entscheidend, um sicherzustellen, dass jede Zeile des Buffers auf einer Speichergrenze beginnt, welche für die GPU-Zugriffe optimal ist, was die Rendering-Performance verbessert.

### 7.2 DMA-BUF-Handle-Übertragung

DMA-BUF-Import erfolgt über das `zwp_linux_dmabuf_v1`-Protokoll und ermöglicht den direkten Austausch von GPU-spezifischen Buffern zwischen Client und Compositor ohne Kopiervorgänge. Der Client erstellt `zwp_linux_dmabuf_v1.create_params` und fügt File-Deskriptoren (`fd`) für jede Plane des Buffers hinzu (`zwp_linux_dmabuf_v1.add_fd`). Ein kritischer Parameter ist der `modifier` (`uint64`), welcher ein herstellerspezifisches Tiling- oder Kompressionsformat angibt, das für optimale GPU-Performance genutzt wird (z.B. `DRM_FORMAT_MOD_LINEAR` für lineares Layout oder vendor-spezifische Werte). Der File-Deskriptor-Transfer selbst nutzt den `SCM_RIGHTS`-Mechanismus über Unix-Domain-Sockets, welcher einen sicheren und effizienten Weg bietet, Dateideskriptoren zwischen Prozessen zu übergeben. Die `wl_buffer.release`-Event-Nachricht ist hier von entscheidender Bedeutung, da sie dem Client signalisiert, dass der Compositor die Nutzung des DMA-BUF-Buffers beendet hat und dieser vom Client wiederverwendet oder freigegeben werden kann.

## 8. Input-Event-Verarbeitung und Hardware-Integration

### 8.1 Pointer-Event-Sequenz-Automaten

Pointer-Events folgen einer strengen, sequenziellen Automatenlogik, welche nachfolgend dargestellt wird:

1. `wl_pointer.enter(serial=N)`: Der Mauszeiger betritt die Oberfläche eines Clients. `serial` ist eine monoton steigende `uint32`-Nummer, welche die Reihenfolge der Ereignisse sicherstellt und Replay-Angriffe präveniert.
    
2. `wl_pointer.motion(time=T, surface_x=X, surface_y=Y)`: Der Mauszeiger bewegt sich über die Oberfläche. `time` ist der Timestamp in Millisekunden seit Compositor-Start. `surface_x` und `surface_y` sind Koordinaten im Fixed-Point-Format (`wl_fixed`, Wert ×256) für Sub-Pixel-Präzision. Motion-Events sind auf maximal 1000 Hz limitiert.
    
3. `wl_pointer.button(serial=N+1, time=T+Δ, button=BTN_LEFT, state=PRESSED|RELEASED)`: Eine Maustaste wird gedrückt oder losgelassen. `button` ist ein `uint32` Scancode (z.B. `BTN_LEFT`). `state` ist 0 für `RELEASED` und 1 für `PRESSED`. Während eines Button-Presses kann ein impliziter Grab auf dem Client stattfinden, welcher sämtliche weiteren Pointer-Events an diesen Client leitet, bis die Taste losgelassen wird.
    
4. wl_pointer.leave(serial=N+2): Der Mauszeiger verlässt die Oberfläche.
    
    Die serial-Nummern sind für die korrekte Zuordnung von Events zu bestimmten Aktionen und für die Vermeidung von Race Conditions unerlässlich.
    

### 8.2 Keyboard-Focus-State-Machine

Der Keyboard-Focus-Zustand wird durch eine präzise Zustandsmaschine verwaltet, deren Transitionen nachfolgend erläutert werden:

1. `UNFOCUSED`: Das Surface besitzt keinen Tastaturfokus.
    
2. → `wl_keyboard.enter(serial=S, surface=SURF, keys=ARRAY)`: Das Surface erlangt den Tastaturfokus. `serial` ist eine `uint32`-Nummer. `keys` ist ein `wl_array` von `uint32`-Scancodes der aktuell gedrückten Tasten.
    
3. → `FOCUSED`: Das Surface besitzt den Tastaturfokus. In diesem Zustand werden `wl_keyboard.key` (für Tasten-Druck/Loslassen) und `wl_keyboard.modifiers` (für Modifikatoren-Zustandsänderungen) gesendet.
    
    - `wl_keyboard.key(serial=S', time=T, key=K, state=PRESSED|RELEASED)`: Eine Taste wird gedrückt oder losgelassen.
        
    - `wl_keyboard.modifiers(serial=S'', mods_depressed=MD, mods_latched=ML, mods_locked=ML, group=G)`: Der Zustand der Modifikatortasten ändert sich. `MD` ist eine Bitmaske der aktuell gedrückten Modifikatoren (z.B. Shift, Ctrl). `ML` ist eine Bitmaske der Modifikatoren, die durch einen einzelnen Tastendruck umgeschaltet wurden (z.B. Caps Lock). `ML` ist eine Bitmaske der permanent aktivierten Modifikatoren (z.B. Num Lock). `G` ist die aktuell aktive Tastaturlayout-Gruppe.
        
4. → `wl_keyboard.leave(serial=S+N)`: Das Surface verliert den Tastaturfokus.
    
5. → UNFOCUSED.
    
    Das keys-Array in wl_keyboard.enter ist von Bedeutung, um den initialen Zustand sämtlicher gedrückter Tasten beim Fokusgewinn zu übermitteln.
    

### 8.3 Touch-Point-Lifecycle-Definition

Touch-Ereignisse werden als individuelle Touch-Points mit einem spezifischen Lebenszyklus behandelt, dessen Phasen nachfolgend beschrieben werden:

1. `wl_touch.down(id=ID, serial=S, surface=SURF, x=X, y=Y)`: Ein neuer Touch-Point wird detektiert. `ID` ist eine `uint32`-ID (`0-15`) zur eindeutigen Identifizierung des Touch-Points für dessen Lebensdauer. `x` und `y` sind die Koordinaten im Fixed-Point-Format (Wert ×256).
    
2. → `wl_touch.motion(time=T, id=ID, x=X, y=Y)*`: Der Touch-Point bewegt sich. Dies kann mehrfach pro Frame auftreten.
    
3. → wl_touch.up(time=T, id=ID): Der Touch-Point wird losgelassen.
    
    Sämtliche Touch-Events für einen einzelnen Frame werden durch einen wl_touch.frame-Event gruppiert, welcher das Ende einer Gruppe von Touch-Updates signalisiert. Die maximale Anzahl simultaner Touch-Points, welche vom Protokoll unterstützt wird, beträgt 10. Die ID ist entscheidend, um den Zustand und die Bewegung jedes einzelnen Fingers oder Eingabestifts über die Zeit zu verfolgen.
    

## 9. Timing-Constraints-Spezifikation

### 9.1 Event-Timestamp-Präzision

Alle Input-Events (Pointer, Keyboard, Touch) enthalten einen `uint32`-Timestamp in Millisekunden, welcher die Zeit seit dem Start des Compositors angibt. Es ist zu beachten, dass dieser Timestamp bei 4294967295 ms (approximativ 49.7 Tage) einen Überlauf erfährt, was bei sehr langlebigen Anwendungen Berücksichtigung finden muss. Für Anwendungen, welche eine höhere Präzision erfordern (z.B. Spiele, präzise Zeichenprogramme), offeriert das `wp_relative_pointer_v1`-Protokoll zusätzliche Mikrosekunden-Präzision für relative Mausbewegungen. Die Delta-Werte für relative Bewegungen werden ebenfalls im `wl_fixed`-Format (Festkomma ×256) übermittelt, um Sub-Pixel-Genauigkeit zu gewährleisten.

### 9.2 Frame-Timing-Garantien

Frame-Callbacks (`wl_surface.frame`) garantieren ein Timing innerhalb von 16.67 ms (60 Hz) ±1 ms, was der typischen Bildwiederholrate der meisten Displays korrespondiert. Für detaillierteres Feedback zur Frame-Präsentation wird das `wp_presentation`-Protokoll verwendet. Der `wp_presentation_feedback`-Event liefert umfangreiche Informationen, welche nachfolgend aufgeführt sind:

- `tv_sec` (`uint32`) und `tv_nsec` (`uint32`): Absolute Präsentationszeit des Frames.
    
- `refresh` (`uint32` in Nanosekunden): Die tatsächliche Bildwiederholperiode des Displays.
    
- `seq_hi` + `seq_lo` (64-bit): Ein Frame-Zähler zur eindeutigen Identifizierung des gerenderten Frames.
    
- `flags` (`uint32` Bitmaske): Zusätzliche Informationen über die Präsentation, z.B.:
    
    - `VSYNC=1`: Der Frame wurde während des Vertical-Blank-Intervalls präsentiert.
        
    - `HW_CLOCK=2`: Der Timestamp stammt von einer Hardware-Uhr.
        
    - `HW_COMPLETION=4`: Die Komposition des Frames wurde vollständig durch Hardware abgeschlossen.
        
    - ZERO_COPY=8: Der Buffer wurde ohne zusätzlichen Kopiervorgang präsentiert.
        
        Diese detaillierten Timing-Informationen sind für Anwendungen mit hohen Anforderungen an Latenz und Synchronisation unerlässlich.
        

### 9.3 Input-Rate-Limiting

Wayland implementiert Rate-Limiting für bestimmte Input-Events, um eine Überflutung des Compositors zu prävenieren und Ressourcen zu schonen. Pointer-Motion-Events sind auf maximal 1000 Events/Sekunde limitiert. Scroll-Events über `wl_pointer.axis` sind auf maximal 120 detents/Sekunde begrenzt, wobei der Wert im `wl_fixed`-Format übermittelt wird, um feine Scroll-Schritte zu ermöglichen. Für gestufte Scroll-Ereignisse (z.B. Mausrad-Klicks) wird `wl_pointer.axis_discrete` verwendet, welches Integer-Werte von −3 bis +3 pro Event liefert. Diese Limits sind entscheidend, um die Stabilität und Performance des Compositors unter hoher Eingabelast zu gewährleisten.

## 10. Smithay-Hardware-Abstraktion

### 10.1 DRM-Backend-Konfiguration

Das Smithay-DRM-Backend interagiert direkt mit dem Kernel-DRM-Subsystem über die `libdrm`-Schnittstelle. Die Konfiguration erfordert spezifische Parameter, welche nachfolgend gelistet sind:

- `Card-Path`: Der Pfad zum DRM-Gerät (z.B. `/dev/dri/card0`).
    
- `Connector-ID`: Eine `uint32`-ID, welche einen physischen Display-Anschluss (z.B. HDMI, DisplayPort) identifiziert.
    
- Mode-Struct: Eine Struktur, welche die Anzeigemodus-Parameter definiert, einschließlich hdisplay (horizontale Pixel), vdisplay (vertikale Pixel), clock (Pixeltakt in kHz) und flags (z.B. DRM_MODE_FLAG_PHSYNC für positive horizontale Synchronisation).
    
    Atomic-Modesetting, der präferierte Mechanismus für die Konfiguration des Displays, erfolgt über Property-IDs, welche CRTC_ID (Cathode Ray Tube Controller), FB_ID (Framebuffer), SRC_X/Y/W/H (16.16 fixed-point für Quellrechteck im Framebuffer) und CRTC_X/Y/W/H (uint32 für Zielrechteck auf dem CRTC) umfassen. Plane-Types (PRIMARY=1, CURSOR=2, OVERLAY=3) definieren die Schicht, auf welcher ein Buffer dargestellt wird.
    

### 10.2 Buffer-Import-Validation

Der DMA-BUF-Import-Prozess in Smithay beinhaltet eine strenge Validierung der übergebenen Buffer-Parameter. Dies umfasst:

- `fourcc` (`uint32`): Ein Four Character Code, welcher das Pixelformat des Buffers angibt (z.B. `DRM_FORMAT_ARGB8888`).
    
- `width/height` (`uint32`): Die Dimensionen des Buffers, maximal 16384×16384 Pixel.
    
- `modifier` (`uint64`): Ein kritischer Parameter, welcher das Layout des Buffers im GPU-Speicher beschreibt (z.B. `DRM_FORMAT_MOD_LINEAR` für ein lineares Layout oder herstellerspezifische Tiling-Formate für optimale GPU-Performance).
    
- plane_count (1−4): Die Anzahl der Planes im Buffer (z.B. für YUV-Formate).
    
    Zusätzlich zu diesen Parametern wird eine stat()-Prüfung des übergebenen File-Deskriptors durchgeführt, um sicherzustellen, dass die st_size des Buffers ausreichend groß ist, um die angeforderten Daten (offset + stride × height) zu enthalten, was Out-of-Bounds-Speicherzugriffe präveniert.
    

### 10.3 GPU-Memory-Management

Smithay integriert sich mit EGL (Embedded-GL) für das GPU-Memory-Management. Die EGL-Integration erfolgt über die `EGL_EXT_image_dma_buf_import`-Erweiterung, welche es ermöglicht, EGL-Images direkt aus DMA-BUFs zu erstellen. Der Aufruf `eglCreateImageKHR` verwendet ein Attribute-Array, welches die DMA-BUF-Details (`EGL_WIDTH`, `EGL_HEIGHT`, `EGL_LINUX_DRM_FOURCC_EXT`, `EGL_DMA_BUF_PLANE0_FD`, `EGL_DMA_BUF_PLANE0_OFFSET`, `EGL_DMA_BUF_PLANE0_PITCH`) enthält. Dieses EGL-Image kann sodann mittels `glEGLImageTargetTexture2DOES` an eine OpenGL ES-Textur gebunden werden. Dies ermöglicht es dem Compositor, direkt auf die Pixeldaten im DMA-BUF zuzugreifen, ohne dass ein Kopiervorgang in den GPU-Speicher erforderlich ist, was die Rendering-Latenz minimiert und die Effizienz maximiert.

## 11. Protokoll-Error-Codes-Matrix

### 11.1 Surface-Protocol-Errors

Diese Fehler treten auf, wenn Clients die Regeln für `wl_surface` oder seine Rollenobjekte verletzen. Die Fehlercodes sind wie folgt definiert:

- `wl_surface` errors:
    
    - `INVALID_SCALE=0`: `set_buffer_scale` wurde mit einem ungültigen oder nicht unterstützten Skalierungsfaktor aufgerufen.
        
    - `INVALID_TRANSFORM=1`: `set_buffer_transform` wurde mit einem ungültigen Transformationswert aufgerufen.
        
    - `INVALID_SIZE=2`: Der Client versucht, eine ungültige Größe für das Surface zu setzen (z.B. 0×0).
        
    - `INVALID_OFFSET=3`: Ein ungültiger Offset wurde für `wl_surface.damage` oder `wl_surface.damage_buffer` angegeben.
        
- `xdg_surface` errors:
    
    - `NOT_CONSTRUCTED=1`: Eine `xdg_surface`-Anfrage (z.B. `get_toplevel`) wurde gesendet, bevor `xdg_surface.set_toplevel` oder `xdg_surface.set_popup` aufgerufen wurde.
        
    - `ALREADY_CONSTRUCTED=2`: `xdg_surface.set_toplevel` oder `xdg_surface.set_popup` wurde mehr als einmal für dasselbe `xdg_surface` aufgerufen.
        
    - `UNCONFIGURED_BUFFER=3`: Ein `wl_surface.commit` wurde ohne vorheriges `xdg_surface.ack_configure` gesendet, nachdem eine Konfiguration empfangen wurde.
        
- `xdg_toplevel` errors:
    
    - `INVALID_RESIZE_EDGE=0`: Ein ungültiger `resize_edge`-Wert wurde bei `set_maximized` oder `set_resizing` angegeben.
        
    - `INVALID_PARENT=1`: Ein ungültiges Parent-Surface wurde für ein Toplevel-Fenster angegeben.
        
    - `INVALID_SIZE=2`: Eine ungültige minimale oder maximale Größe wurde für das Toplevel-Fenster gesetzt.
        

### 11.2 Buffer-Protocol-Errors

Diese Fehler beziehen sich auf Probleme mit Buffer-Objekten. Die Fehlercodes sind wie folgt spezifiziert:

- `wl_shm` errors:
    
    - `INVALID_FORMAT=0`: Der Client hat ein nicht unterstütztes Pixelformat für einen Shared-Memory-Buffer angefordert.
        
    - `INVALID_STRIDE=1`: Der angegebene `stride` (Bytes pro Zeile) ist ungültig (z.B. 0 oder zu klein).
        
    - `INVALID_FD=2`: Der übergebene File-Deskriptor für den Shared-Memory-Pool ist ungültig.
        
    - `INVALID_SIZE=3`: Die angegebene Größe des Shared-Memory-Pools ist ungültig.
        
- `zwp_linux_dmabuf` errors:
    
    - `PLANE_IDX=0`: Ein ungültiger Planen-Index wurde für `add_fd` angegeben (z.B. außerhalb des Bereichs 0−3).
        
    - `PLANE_SET=1`: Eine Plane wurde mehrfach hinzugefügt.
        
    - `INCOMPLETE=2`: Nicht alle erforderlichen Planes wurden hinzugefügt, bevor der Buffer finalisiert wurde.
        
    - `INVALID_FORMAT=3`: Das angegebene DMA-BUF-Format ist ungültig oder nicht unterstützt.
        
    - `INVALID_DIMENSIONS=4`: Die Dimensionen des Buffers sind ungültig.
        
    - `OUT_OF_BOUNDS=5`: Die angegebene Offset- oder Stride-Kombination führt zu einem Zugriff außerhalb der Buffer-Grenzen.
        
    - `INVALID_WL_BUFFER=6`: Der `wl_buffer`-Objekt ist ungültig.
        

### 11.3 Input-Protocol-Errors

Diese Fehler treten bei Verstößen gegen die Input-Protokolle auf. Die Fehlercodes sind wie folgt definiert:

- `wl_seat` errors:
    
    - `MISSING_CAPABILITY=0`: Der Client versucht, eine Input-Fähigkeit (z.B. Pointer, Keyboard, Touch) zu binden, die vom Compositor nicht angeboten wird.
        
- `wl_pointer` errors:
    
    - `ROLE=0`: Ein Surface, das bereits eine andere Rolle hat, versucht, eine Pointer-Rolle zu übernehmen.
        
- `wl_keyboard` errors:
    
    - `NO_KEYMAP=0`: `wl_keyboard.set_keymap` wurde mit einem ungültigen oder nicht unterstützten Keymap-Format aufgerufen.
        
- `wl_touch` errors:
    
    - `ROLE=0`: Ein Surface, das bereits eine andere Rolle hat, versucht, eine Touch-Rolle zu übernehmen.
        
- `wp_text_input` errors:
    
    - `ROLE=0`: Ein Surface, das bereits eine andere Rolle hat, versucht, eine Text-Input-Rolle zu übernehmen.
        
    - `NO_SURFACE=1`: Eine Text-Input-Anfrage wurde ohne ein zugeordnetes Surface gesendet.
        

## 12. GTK-Compositor-Integration-Parameter

### 12.1 Layer-Shell-Z-Order-Berechnung

Das `wlr-layer-shell`-Protokoll definiert die Z-Order von Oberflächen durch explizite Layer und Z-Werte. Die Layer sind:

- `BACKGROUND` (Z=−1000): Für Desktophintergründe.
    
- `BOTTOM` (Z=−500): Für Widgets, welche unter normalen Fenstern liegen.
    
- `TOP` (Z=500): Für Panels, Docks, welche über normalen Fenstern liegen.
    
- OVERLAY (Z=1000): Für kritische Overlays wie Bildschirmsperren oder On-Screen-Keyboards.
    
    Die anchor-Bitmaske (TOP=1, BOTTOM=2, LEFT=4, RIGHT=8) kann kombiniert werden, um Ecken oder Kanten zu verankern (z.B. TOP | LEFT für die obere linke Ecke). Eine exclusive_zone (int32) definiert einen Bereich in Pixeln, welchen die Layer-Shell-Oberfläche vom verfügbaren Platz für normale Fenster abzieht: −1 für volle Exklusion, 0 für keine Exklusion, >0 für eine spezifische Pixelanzahl. Die keyboard_interactivity-Einstellung (NONE=0, EXCLUSIVE=1, ON_DEMAND=2) steuert, ob und wie die Layer-Shell-Oberfläche Tastatureingaben empfängt.
    

### 12.2 Window-Management-State-Tracking

XDG-Toplevel-States werden als `uint32`-Bitmasken im `xdg_toplevel.configure`-Event übermittelt, um den Zustand eines Fensters zu signalisieren. Wichtige States sind:

- `MAXIMIZED=1`: Das Fenster ist maximiert.
    
- `FULLSCREEN=2`: Das Fenster ist im Vollbildmodus.
    
- `RESIZING=4`: Das Fenster wird gerade vom Benutzer in der Größe geändert.
    
- `ACTIVATED=8`: Das Fenster hat den Fokus und ist aktiv.
    
- TILED_LEFT=16, TILED_RIGHT=32, TILED_TOP=64, TILED_BOTTOM=128: Das Fenster ist an eine der Bildschirmkanten gekachelt.
    
    Der configure-Event enthält auch width und height (int32). Ein Wert von 0 für width oder height bedeutet, dass der Client die Größe selbst bestimmen soll. Das states-Feld ist ein wl_array von uint32-Werten, welche die aktiven Zustände repräsentieren. Die korrekte Verarbeitung dieser Zustände ist für ein responsives Window-Management und die Anpassung der Client-UI an den Fensterzustand von entscheidender Bedeutung.
    

### 12.3 Decoration-Protocol-Implementation

Die Implementierung von Fensterdekorationen (Titelbalken, Ränder) wird durch das `zxdg_decoration_manager_v1`-Protokoll gesteuert. Es existieren zwei Hauptmodi:

- `MODE_CLIENT_SIDE=1`: Der Client ist für das Zeichnen seiner eigenen Fensterdekorationen verantwortlich.
    
- MODE_SERVER_SIDE=2: Der Compositor zeichnet die Fensterdekorationen.
    
    Clients können ihre präferierten Modi über zxdg_toplevel_decoration_v1.set_mode signalisieren. Der Compositor antwortet sodann mit zxdg_toplevel_decoration_v1.configure(mode), um den tatsächlich angewandten Modus zu bestätigen. Dies ermöglicht eine flexible Handhabung von Fensterdekorationen, welche entweder vom Client für maximale Anpassung oder vom Server für eine konsistente Desktop-Integration gezeichnet werden können.
    

## 13. Protokoll-Kompatibilität und Versionierung

### 13.1 Abwärtskompatible Erweiterungen

Wayland-Protokolle sind so konzipiert, dass sie abwärtskompatibel erweitert werden können. Neue Protokollfeatures werden typischerweise durch das Hinzufügen neuer Schnittstellenversionen oder separater Erweiterungsschnittstellen implementiert. Clients sind verpflichtet, explizit höhere Versionen einer Schnittstelle anzufordern, sofern sie neue Funktionalitäten nutzen möchten, indem sie die gewünschte Version im `wl_registry.bind`-Aufruf angeben. Unbekannte Nachrichten, welche von einem älteren Client an einen neueren Server gesendet werden, werden ignoriert, um die Kompatibilität zu gewährleisten. Der `wl_registry.global_remove`-Event wird verwendet, um Clients über die Entfernung oder Deprecation von globalen Objekten zu informieren.

### 13.2 Capability-Discovery-Mechanism

Der zentrale `wl_registry`-Mechanismus ermöglicht die dynamische Detektion verfügbarer Protokoll-Capabilities und globaler Objekte. Wenn ein Client eine Wayland-Verbindung etabliert, sendet der Compositor eine Sequenz von `wl_registry.global`-Ereignissen. Jedes dieser Ereignisse kündigt ein neues globales Objekt an, spezifiziert durch dessen `name` (`uint32`), den `interface`-String (z.B. "wl_compositor", "xdg_wm_base") und die `version` (`uint32`). Clients können diese Informationen abfragen und sodann `wl_registry.bind` verwenden, um eine lokale Proxy-Instanz des gewünschten globalen Objekts zu kreieren und somit die Interaktion mit dieser spezifischen Compositor-Fähigkeit zu initiieren. Dies befähigt Clients, sich dynamisch an die vom Compositor offerierten Funktionen zu adaptieren.

### 13.3 Fehlerbehandlung und Debugging

Protokollfehler in Wayland sind fatal und resultieren im sofortigen Verbindungsabbruch. Wenn ein Protokollfehler auftritt, sendet der Compositor einen `wl_display.error`-Event, welcher die `object_id` des verursachenden Objekts, einen spezifischen `error_code` (`uint32`) und eine beschreibende `message` (`string`) enthält, bevor die Verbindung geschlossen wird. Für Debugging-Zwecke kann die Umgebungsvariable `WAYLAND_DEBUG=1` gesetzt werden, um eine detaillierte Protokollierung sämtlicher gesendeter und empfangener Nachrichten zu aktivieren. Smithay bietet zusätzlich interne Validierungsebenen (z.B. `debug_assert!`), welche während der Entwicklung zur frühzeitigen Erkennung und Behebung von Protokollverletzungen beitragen. Die präzisen Fehlercodes ermöglichen es autonomen Agenten, die Ursache des Fehlers exakt zu identifizieren und entsprechende Korrekturmaßnahmen zu initiieren.

# Wayland

## 1.Einführung und Wayland-Protokollübersicht

Wayland ist ein modernes Display-Server-Protokoll, das darauf abzielt, X11 zu ersetzen. Es bietet eine sicherere, effizientere und einfachere Architektur, indem es die Verantwortung des Display-Servers und des Compositors vereint. Für NovaDE ist Wayland die Grundlage für eine performante und zukunftssichere Desktop-Erfahrung, die den "Rust-spezifischen Excellence-Standards" (Zero Unsafe Code, RAII, Compile-Time Guarantees, Fearless Concurrency, Zero-Cost Abstractions, Idiomatic Rust) aus der `Gesamtspezifikation.md` entspricht.

Das Wayland-Protokoll ist objektbasiert. Clients und der Compositor kommunizieren, indem sie Anfragen (Requests) an Objekte senden und Ereignisse (Events) von Objekten empfangen. Jedes Objekt hat eine eindeutige ID, eine definierte Schnittstelle (Interface) und eine Version. Änderungen am Zustand von Oberflächen (`wl_surface`) werden typischerweise gepuffert und durch einen expliziten `commit`-Request atomar angewendet, oft synchronisiert mit dem nächsten Frame-Callback, um visuelle Konsistenz zu gewährleisten und Tearing zu vermeiden.

Smithay ist ein in Rust geschriebenes Toolkit, das Bibliotheken und Hilfsprogramme zur Erstellung von Wayland-Compositoren bereitstellt. Es abstrahiert viele der Low-Level-Details des Wayland-Protokolls und der Interaktion mit dem Betriebssystem (z.B. DRM/KMS, libinput), sodass sich Entwickler auf die Implementierung der Desktop-spezifischen Logik konzentrieren können. Smithay legt Wert auf Sicherheit, Modularität und eine idiomatische Rust-API. Für NovaDE dient Smithay als primäre Abstraktionsschicht für alle Wayland-bezogenen Operationen.

## 2. Kern-Wayland-Protokolle (Core Protocol Implementation)

Diese Protokolle bilden das absolute Fundament jeder Wayland-Implementierung.

### 2.1. `wl_display`

- **Zweck:** Das `wl_display`-Objekt ist das Root-Objekt jeder Wayland-Verbindung. Es dient als Einstiegspunkt für Clients, um globale Objekte zu entdecken (`wl_registry`), Fehler zu behandeln (`error`-Event) und Protokollversionen zu synchronisieren (`sync`-Request, `delete_id`-Request). Es ist eng mit der Event-Loop des Compositors verbunden.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor initialisiert das `wl_display`-Objekt beim Start. Er ist verantwortlich für das Verwalten von Client-Verbindungen, das Abhören des Wayland-Sockets und das Dispatching von Client-Nachrichten.
        
    - **Smithay-Interna:** `smithay::wayland::display::Display<D>` ist die zentrale Struktur. `D` ist der Typ des Compositor-Zustands (`DesktopState` in NovaDE). Die `DisplayHandle` wird verwendet, um mit dem Display aus verschiedenen Teilen des Compositors zu interagieren. `smithay::wayland::compositor::CompositorHandler` wird vom `DesktopState` implementiert und seine `client_compositor_state`-Methode wird vom Display-Dispatcher aufgerufen, um den client-spezifischen Zustand zu erhalten.
        
- **Rust-Exzellenz:**
    
    - **RAII:** Die Lebensdauer von `Client`-Objekten, die mit dem `Display` assoziiert sind, wird durch Rusts Ownership-System verwaltet.
        
    - **Fehlerbehandlung:** `wl_display.error`-Events werden durch Smithays Infrastruktur in Rust-Fehlertypen (`DisconnectReason`) umgewandelt, die über `Result` propagiert und behandelt werden können, um robuste Fehlerbehandlung gemäß `Gesamtspezifikation.md` sicherzustellen.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Protokollfehler:** Ein `wl_display.error`-Event vom Client (z.B. ungültige Objekt-ID) oder vom Server (z.B. interner Fehler) führt zum sofortigen Abbruch der Client-Verbindung. Smithay handhabt dies und informiert den Compositor.
        
    - **Client-Fehler:** Fehlerhafte Client-Anfragen, die zu Protokollfehlern führen, werden isoliert und betreffen nur den jeweiligen Client.
        
    - **Wiederherstellung:** Keine direkte Wiederherstellung auf Protokollebene; der Client muss sich neu verbinden.
        
- **Performance-Optimierung:**
    
    - Minimaler Overhead, da primär zur Verwaltung und zum Event-Dispatching. Smithays Dispatcher ist für Effizienz optimiert.
        
    - Batching von Events durch `wl_display.sync` kann Latenzen reduzieren, wird aber eher clientseitig genutzt.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/core/display_global.rs` (Implementierung des Globals und Handhabung von Display-spezifischen Requests/Events). Die Haupt-Display-Logik ist in `novade-system/src/wayland_compositor/mod.rs` oder `main.rs` angesiedelt.
    
- **Aufwand:** Mittel (Initialisierung, robuste Fehlerbehandlung, Integration in die Haupt-Event-Loop).
    

### 2.2. `wl_registry`

- **Zweck:** Das `wl_registry`-Objekt ermöglicht es Clients, die vom Compositor angebotenen globalen Objekte (wie `wl_compositor`, `wl_shm`, `wl_seat`, `xdg_wm_base` etc.) zu entdecken und an diese zu binden.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor registriert beim Start alle globalen Objekte, die er unterstützt, beim `Display`. Wenn ein Client sich mit dem `wl_registry` verbindet, sendet der Compositor `global`-Events für jedes verfügbare globale Objekt. Bei Entfernung eines Globals wird `global_remove` gesendet.
        
    - **Smithay-Interna:** `smithay::wayland::display::registry::Global<D>` repräsentiert ein globales Objekt. `DisplayHandle::create_global()` wird verwendet, um Globals zu registrieren. Clients verwenden den `bind`-Request auf `wl_registry`, um ein globales Objekt an eine lokale ID zu binden. Smithay handhabt die Validierung und das Dispatching dieser `bind`-Anfragen.
        
- **Rust-Exzellenz:**
    
    - **Typsicherheit:** Smithay ermöglicht die Definition von UserData für Globals, was eine typsichere Handhabung der Bind-Logik erlaubt.
        
    - **Zero-Cost Abstractions:** Die Abstraktionen für Globals und die Registry haben minimalen Laufzeit-Overhead.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Ungültige Bindung:** Wenn ein Client versucht, an ein nicht existierendes Global oder mit einer ungültigen Version zu binden, kann der Compositor dies ignorieren oder einen Protokollfehler senden. Smithay hilft, dies zu verwalten.
        
    - **Client-Fehler:** Ein fehlerhafter `bind`-Request betrifft nur den Client.
        
- **Performance-Optimierung:**
    
    - Relevant hauptsächlich während des Client-Initialisierungsprozesses.
        
    - Effiziente Implementierung der Global-Liste und des Lookups in Smithay.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/core/registry_global.rs` (Verwaltung der globalen Objekte und ihrer Registrierung).
    
- **Aufwand:** Gering (Smithay übernimmt viel, Fokus auf korrekte Registrierung aller NovaDE-Globals).
    

### 2.3. `wl_compositor`

- **Zweck:** Das `wl_compositor`-Global ist eine Fabrik zum Erstellen von `wl_surface`-Objekten (visuelle Oberflächen) und `wl_region`-Objekten (zur Definition von Bereichen auf Oberflächen).
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `wl_compositor`-Global. Clients senden `create_surface`- und `create_region`-Requests.
        
    - **Smithay-Interna:** `smithay::wayland::compositor::CompositorState` verwaltet den Zustand, der mit dem `wl_compositor`-Global assoziiert ist. `CompositorHandler::new_surface()` wird aufgerufen, wenn ein Client eine neue Oberfläche erstellt. Der Compositor muss hier `SurfaceData` für die neue Oberfläche initialisieren.
        
- **Rust-Exzellenz:**
    
    - **RAII:** `WlSurface`-Objekte werden von Smithay verwaltet; ihre Lebensdauer ist an Rusts Ownership gebunden.
        
    - **Zustandsverwaltung:** `CompositorState` und `SurfaceData` ermöglichen eine sichere und strukturierte Verwaltung des Oberflächenzustands, oft unter Verwendung von `Arc<Mutex<T>>` oder ähnlichen Mustern für geteilten, veränderlichen Zustand gemäß `Gesamtspezifikation.md`.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Ungültige Anfragen:** Z.B. Erstellung einer Oberfläche mit einer bereits verwendeten ID (wird von Smithay/Wayland-Server behandelt).
        
    - **Ressourcenerschöpfung:** Wenn der Compositor keine weiteren Oberflächen erstellen kann (selten), könnte ein Protokollfehler gesendet werden.
        
- **Performance-Optimierung:**
    
    - Die Erstellung von Oberflächen und Regionen sollte effizient sein. Smithay ist hierfür optimiert.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/core/compositor_global.rs` (Implementierung des Globals) und `novade-system/src/wayland_compositor/surface_management.rs` (für `SurfaceData` und zugehörige Logik).
    
- **Aufwand:** Gering bis Mittel (Smithay stellt die Basis bereit, aber die Integration von `SurfaceData` und die Verbindung zur Rendering-Pipeline erfordern Sorgfalt).
    

### 2.4. `wl_shm` und `wl_shm_pool`

- **Zweck:** Das `wl_shm`-Protokoll ermöglicht Clients, Pixeldaten über Shared Memory (gemeinsam genutzten Speicher) mit dem Compositor auszutauschen. Clients erstellen einen `wl_shm_pool` aus einem Dateideskriptor, der auf ein Shared-Memory-Segment zeigt, und erstellen dann `wl_buffer` aus diesem Pool.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `wl_shm`-Global und die zugehörigen Requests (`create_pool`). Er muss die vom Client bereitgestellten Dateideskriptoren validieren und mappen.
        
    - **Smithay-Interna:** `smithay::wayland::shm::ShmState` verwaltet die unterstützten Pixelformate und den Zustand des `wl_shm`-Globals. `ShmHandler::shm_state()` stellt diesen Zustand bereit. Wenn ein Client einen Puffer aus einem SHM-Pool erstellt, validiert Smithay die Parameter. Der Compositor muss dann über `BufferHandler` (siehe 2.5) auf die Pufferdaten zugreifen können.
        
- **Rust-Exzellenz:**
    
    - **Sicherheit:** Sichere Wrapper für FFI-Aufrufe wie `mmap` sind entscheidend. Rusts Typ-System und Ownership-Regeln helfen, Fehler beim Umgang mit rohen Speicherzeigern zu vermeiden.
        
    - **RAII:** Dateideskriptoren und gemappte Speicherbereiche müssen über RAII verwaltet werden, um Leaks zu verhindern. Smithay unterstützt dies.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Ungültige Dateideskriptoren:** Wenn der Client einen ungültigen FD bereitstellt.
        
    - **Fehler beim `mmap`:** Wenn der Speicher nicht gemappt werden kann.
        
    - **Ungültige Pool-/Buffer-Parameter:** Z.B. Größe, Offset, stride. Smithay sendet hier Protokollfehler.
        
- **Performance-Optimierung:**
    
    - **Zero-Copy (bedingt):** SHM ist nicht per se Zero-Copy zur GPU, da die Daten oft von der CPU zur GPU kopiert werden müssen. Die Effizienz hängt vom Renderer ab.
        
    - Minimierung des Overheads beim Zugriff auf SHM-Daten.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/shm_global.rs` (Implementierung des Globals und Pool-Logik) und Integration mit `novade-system/src/wayland_compositor/buffer_management.rs`.
    
- **Aufwand:** Mittel (Sicherheitsaspekte von Shared Memory und FFI erfordern Sorgfalt).
    

### 2.5. `wl_buffer`

- **Zweck:** Ein `wl_buffer`-Objekt repräsentiert einen Satz von Pixeldaten, die von einem Client bereitgestellt und vom Compositor angezeigt werden können. Es ist eine Abstraktion über verschiedene Puffer-Typen (z.B. SHM, DMABUF).
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor muss Puffer von Clients empfangen und verwalten. Wenn ein Client einen Puffer an eine Oberfläche anhängt (`wl_surface.attach`), muss der Compositor diesen Puffer für das Rendering verfügbar machen.
        
    - **Smithay-Interna:** `smithay::wayland::buffer::BufferHandler` ist ein Trait, das der Compositor-Zustand implementiert, um auf Puffer-Ereignisse zu reagieren, insbesondere `buffer_destroyed()`. `smithay::wayland::compositor::SurfaceData` speichert oft eine Referenz auf den aktuell angehängten Puffer. Der Compositor muss den Pufferinhalt interpretieren und an den Renderer übergeben.
        
- **Rust-Exzellenz:**
    
    - **Ownership:** Rusts Ownership-Regeln sind entscheidend für die korrekte Verwaltung der Lebensdauer von Puffern und der zugehörigen Ressourcen (z.B. GPU-Texturen). `Arc` wird oft für geteilten Besitz von Pufferdaten verwendet.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Nutzung eines zerstörten Puffers:** Wenn ein Client versucht, einen bereits freigegebenen Puffer zu verwenden.
        
    - **Ungültige Pufferattribute:** Z.B. falsches Format, inkonsistente Größe.
        
- **Performance-Optimierung:**
    
    - Effiziente Übergabe der Pufferdaten an den Renderer.
        
    - Vermeidung unnötiger Datenkopien, insbesondere bei der Interaktion mit der GPU.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/buffer_management.rs` (allgemeine Pufferlogik, Trait-Implementierungen).
    
- **Aufwand:** Mittel (zentral für das Rendering, erfordert enge Abstimmung mit dem Renderer).
    

### 2.6. `wl_surface`

- **Zweck:** Eine `wl_surface` ist eine rechteckige Fläche auf dem Bildschirm, auf der ein Client Inhalte darstellen kann. Sie ist die grundlegende Einheit für sichtbare Elemente. Clients können Puffer anhängen, Schaden melden, Transformationen anwenden und Sub-Oberflächen erstellen.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor verwaltet den Zustand jeder `wl_surface`. Der `commit`-Request ist zentral: Hier werden alle ausstehenden Änderungen (neuer Puffer, Schaden, Sub-Oberflächen-Positionen etc.) atomar angewendet.
        
    - **Smithay-Interna:** `smithay::wayland::compositor::SurfaceData` wird verwendet, um anwendungsspezifische Daten mit einer `WlSurface` zu assoziieren. Dies beinhaltet typischerweise den angehängten Puffer, die Schadensregion, Transformationen, die Rolle der Oberfläche (z.B. Toplevel, Popup) und den Zustand von Protokollerweiterungen. `CompositorHandler::commit()` wird aufgerufen, wenn ein Client `wl_surface.commit()` aufruft.
        
- **Rust-Exzellenz:**
    
    - **Atomare Zustandsübergänge:** Die Commit-Logik muss robust implementiert sein, um Konsistenz zu gewährleisten. Rusts Typ-System hilft, Zustandsfehler zu vermeiden.
        
    - **Datenstrukturen:** Effiziente Datenstrukturen für die Verwaltung von Oberflächenzuständen (z.B. `HashMap` für Oberflächen, `Vec` für Schadensregionen).
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Ungültige Operationen:** Z.B. Anhängen eines ungültigen Puffers, Setzen einer ungültigen Rolle.
        
    - **Commit ohne Puffer:** Wenn eine Oberfläche committet wird, ohne dass ein Puffer angehängt ist (abhängig von der Rolle).
        
- **Performance-Optimierung:**
    
    - **Damage Tracking:** Effizientes Verfolgen und Anwenden von Schadensregionen (`wl_surface.damage`, `wl_surface.damage_buffer`) ist entscheidend, um nur die Teile der Oberfläche neu zu zeichnen, die sich geändert haben.
        
    - **Effizientes Compositing:** Die Art und Weise, wie Oberflächen (insbesondere mit Transparenz und Überlappungen) zusammengesetzt werden, hat großen Einfluss auf die Performance.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/surface_management.rs` (Logik für `SurfaceData`, Commit-Verarbeitung) und Integration mit der Rendering-Pipeline in `novade-system/src/renderer/`.
    
- **Aufwand:** Hoch (zentrale Komponente, komplexe Zustandsverwaltung und Interaktion mit dem Renderer).
    

### 2.7. `wl_subcompositor` und `wl_subsurface`

- **Zweck:** Ermöglicht die Erstellung hierarchischer Oberflächenstrukturen. Eine `wl_subsurface` ist eine `wl_surface`, die relativ zu einer Eltern-`wl_surface` positioniert wird. Dies ist nützlich für komplexe UI-Elemente, die aus mehreren Teilen bestehen (z.B. ein Fenster mit eingebetteten Video-Playern oder benutzerdefinierten Titelleisten).
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor muss die Hierarchie von Sub-Oberflächen verwalten, deren Positionen relativ zur Elternoberfläche berechnen und sie korrekt rendern. Der Commit-Mechanismus muss auch für Sub-Oberflächen-Hierarchien synchronisiert werden.
        
    - **Smithay-Interna:** `CompositorHandler::new_subsurface()` wird aufgerufen, wenn eine Sub-Oberfläche erstellt wird. `SurfaceData` muss Informationen über Eltern- und Kind-Beziehungen speichern. Smithay bietet Mechanismen zur Synchronisation von Commits in Oberflächenbäumen (`wl_subsurface.set_sync`, `wl_subsurface.set_desync`).
        
- **Rust-Exzellenz:**
    
    - **Datenstrukturen:** Baumartige Datenstrukturen oder Referenzen (`Weak<WlSurface>`) zur Repräsentation der Hierarchie.
        
    - **Rekursive Algorithmen:** Für das Durchlaufen und Verarbeiten von Oberflächenbäumen (z.B. beim Rendern oder Anwenden von Transformationen).
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Zyklen in der Hierarchie:** Der Compositor muss verhindern, dass zirkuläre Abhängigkeiten entstehen.
        
    - **Ungültige Eltern-Kind-Beziehungen:**
        
- **Performance-Optimierung:**
    
    - Effiziente Aktualisierung und Traversierung der Oberflächenhierarchie.
        
    - Caching von transformierten Positionen von Sub-Oberflächen.
        
- **Modulstruktur:** Integration in `novade-system/src/wayland_compositor/surface_management.rs`.
    
- **Aufwand:** Mittel bis Hoch (erhöht die Komplexität der Oberflächenverwaltung und des Renderings).
    

### 2.8. `wl_output`

- **Zweck:** Ein `wl_output`-Objekt repräsentiert einen Bildschirm oder Monitor, der dem Compositor zur Verfügung steht. Clients verwenden es, um Informationen über die Geometrie, Auflösung, Skalierung und andere Eigenschaften des Outputs zu erhalten.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor erkennt physische Monitore (z.B. über DRM/KMS) und erstellt für jeden ein `wl_output`-Global. Er sendet `geometry`-, `mode`-, `scale`- und `done`-Events an Clients, um sie über die Output-Eigenschaften zu informieren.
        
    - **Smithay-Interna:** `smithay::output::Output` repräsentiert einen Output im Compositor. `OutputHandler` ist ein Trait, das der Compositor-Zustand implementiert, um auf Output-bezogene Ereignisse zu reagieren (z.B. wenn ein Client an ein `wl_output`-Global bindet). `OutputManagerState` kann verwendet werden, um `xdg-output` zu verwalten.
        
- **Rust-Exzellenz:**
    
    - **Zustandsverwaltung:** Konsistente Verwaltung des Zustands jedes Outputs.
        
    - **Event-Dispatching:** Effiziente Benachrichtigung der Clients über Output-Änderungen.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Hotplugging:** Robuste Handhabung von dynamisch hinzugefügten oder entfernten Monitoren.
        
    - **Inkonsistente Zustände:** Vermeidung von Diskrepanzen zwischen dem tatsächlichen Hardware-Zustand und den an Clients gesendeten Informationen.
        
- **Performance-Optimierung:**
    
    - Minimale Auswirkungen auf die Performance, da primär informativ.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/output_management.rs` (Erkennung, Verwaltung und Protokoll-Implementierung für Outputs).
    
- **Aufwand:** Mittel (erfordert Interaktion mit dem Backend, z.B. DRM/KMS, und sorgfältige Zustandssynchronisation).
    

### 2.9. `wl_seat`

- **Zweck:** Ein `wl_seat`-Objekt repräsentiert eine Gruppe von Eingabe- und Ausgabegeräten, die einem einzelnen Benutzer zugeordnet sind (typischerweise Tastatur, Maus/Touchpad, Touchscreen und die zugehörigen Bildschirme). Es ist der primäre Mechanismus für Clients, um Eingabefokus zu erhalten und Eingabeereignisse zu empfangen.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert mindestens ein `wl_seat`-Global. Er verwaltet die "Capabilities" des Seats (Tastatur, Zeiger, Touch) und sendet `capabilities`-Events an Clients. Er ist verantwortlich für das Fokusmanagement: Welcher Client/Oberfläche empfängt Tastatur-, Zeiger- oder Touch-Ereignisse.
        
    - **Smithay-Interna:** `smithay::input::SeatState<D>` verwaltet den Zustand eines Seats. `SeatHandler<D>` ist ein Trait, das der Compositor-Zustand implementiert, um auf Fokusänderungen und andere Seat-bezogene Ereignisse zu reagieren. `Seat::add_keyboard()`, `Seat::add_pointer()`, `Seat::add_touch()` werden verwendet, um Fähigkeiten hinzuzufügen.
        
- **Rust-Exzellenz:**
    
    - **Typsicherheit:** Starke Typisierung für verschiedene Eingabegeräte und Fokus-Targets.
        
    - **Zustandsmaschinen:** Klare Zustandsmaschinen für Fokus und Eingabegeräte-Status.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Inkonsistente Fokus-Zustände:** Vermeidung von Situationen, in denen der Fokus unklar ist oder an eine ungültige Oberfläche gesendet wird.
        
    - **Fehlende Capabilities:** Korrekte Handhabung, wenn ein Seat bestimmte Fähigkeiten (z.B. Touch) nicht besitzt.
        
- **Performance-Optimierung:**
    
    - Effizientes Fokus-Tracking und Event-Dispatching. Geringe Latenz bei der Weiterleitung von Eingabeereignissen ist entscheidend.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/input/seat_global.rs` (Implementierung des Globals) und `novade-system/src/input_manager/focus.rs` (Logik für Fokusmanagement).
    
- **Aufwand:** Hoch (komplexe Zustandsverwaltung für Fokus und Capabilities, enge Interaktion mit dem Input-Backend).
    

### 2.10. `wl_keyboard`

- **Zweck:** Das `wl_keyboard`-Interface wird verwendet, um Tastatureingabeereignisse an Clients zu senden.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Wenn ein Client den Tastaturfokus erhält, sendet der Compositor ein `enter`-Event. Anschließend werden `key` (Tastendruck/-loslassen), `modifiers` (Status von Shift, Ctrl, Alt etc.) und `keymap` (Tastaturbelegung) Events gesendet.
        
    - **Smithay-Interna:** `smithay::input::keyboard::KeyboardHandle<D>` wird verwendet, um Tastaturereignisse an den fokussierten Client zu senden. Die Integration mit `xkbcommon` ist notwendig, um Keycodes in Keysyms und UTF-8 zu übersetzen und den Zustand der Modifikatoren zu verwalten.
        
- **Rust-Exzellenz:**
    
    - **FFI-Sicherheit:** Sichere Bindings zu `libxkbcommon`.
        
    - **Korrekte Zustandsverwaltung:** Für Modifikatoren und Tastatur-Layouts.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Fehler bei der Keymap-Verarbeitung:**
        
    - **Inkonsistente Modifier-Zustände:**
        
- **Performance-Optimierung:**
    
    - Geringe Latenz bei der Übertragung von Tastendrücken.
        
- **Modulstruktur:** `novade-system/src/input_manager/keyboard.rs` (Integration mit `libinput` und `xkbcommon`, Event-Übersetzung).
    
- **Aufwand:** Mittel (Integration von `xkbcommon` erfordert Sorgfalt).
    

### 2.11. `wl_pointer`

- **Zweck:** Das `wl_pointer`-Interface wird verwendet, um Zeiger-basierte Eingabeereignisse (Maus, Touchpad-Zeiger) an Clients zu senden.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Wenn der Zeiger eine Oberfläche eines Clients betritt, sendet der Compositor ein `enter`-Event. Anschließend werden `motion` (Bewegung), `button` (Tastendruck/-loslassen) und `axis` (Scrollen) Events gesendet. Der Compositor ist auch für das Setzen des Cursor-Bildes zuständig (`set_cursor`-Request vom Client).
        
    - **Smithay-Interna:** `smithay::input::pointer::PointerHandle<D>` wird verwendet, um Zeigerereignisse an den Client zu senden, dessen Oberfläche sich unter dem Cursor befindet. `PointerHandle::motion()` erfordert die Transformation von globalen Bildschirmkoordinaten in lokale Oberflächenkoordinaten.
        
- **Rust-Exzellenz:**
    
    - **Präzise Koordinatentransformation:** Wichtig für korrekte Interaktion.
        
    - **Effizientes Event-Dispatching:**
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Ungenauer Fokus:** Wenn `enter`/`leave`-Events nicht korrekt gesendet werden.
        
    - **Verlorene Button-Events:**
        
- **Performance-Optimierung:**
    
    - Geringe Latenz, flüssiges Tracking der Zeigerbewegung.
        
- **Modulstruktur:** `novade-system/src/input_manager/pointer.rs` (Integration mit `libinput`, Event-Übersetzung, Koordinatentransformation).
    
- **Aufwand:** Mittel.
    

### 2.12. `wl_touch`

- **Zweck:** Das `wl_touch`-Interface wird verwendet, um Touch-Eingabeereignisse von Touchscreens oder Touchpads (im Touch-Modus) an Clients zu senden.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Wenn ein Touchpunkt auf einer Oberfläche eines Clients beginnt, sendet der Compositor ein `down`-Event. Anschließend `motion` (Bewegung des Touchpunkts) und `up` (Loslassen des Touchpunkts). Das Protokoll unterstützt Multi-Touch durch Touchpunkt-IDs.
        
    - **Smithay-Interna:** `smithay::input::touch::TouchHandle<D>` wird verwendet, um Touch-Ereignisse an den Client zu senden, dessen Oberfläche den jeweiligen Touchpunkt empfängt.
        
- **Rust-Exzellenz:**
    
    - **Multi-Touch-Verwaltung:** Korrekte Verfolgung und Zuordnung von mehreren gleichzeitigen Touchpunkten.
        
    - **Gesten-Erkennung (Basis):** Obwohl komplexe Gesten oft von Toolkits oder Anwendungen implementiert werden, muss der Compositor die grundlegenden Touch-Events zuverlässig liefern.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Verlorene `up`/`down`-Events:** Können zu "hängenden" Touchpunkten führen.
        
    - **Ungenauer Touch-Fokus:**
        
- **Performance-Optimierung:**
    
    - Geringe Latenz, flüssiges Tracking von Touch-Bewegungen.
        
- **Modulstruktur:** `novade-system/src/input_manager/touch.rs` (Integration mit `libinput`, Event-Übersetzung).
    
- **Aufwand:** Mittel bis Hoch (Multi-Touch-Logik kann komplex sein).
    

### 2.13. `wl_callback`

- **Zweck:** Ein `wl_callback`-Objekt wird verwendet, um einen Client zu benachrichtigen, wenn der Compositor einen Frame dargestellt hat und bereit ist, den nächsten zu zeichnen. Dies ist entscheidend für die Synchronisation von Client-Rendering mit der Bildwiederholrate des Displays, um Tearing zu vermeiden und flüssige Animationen zu ermöglichen.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Clients fordern einen Callback über `wl_surface.frame()` an. Der Compositor sendet das `done`-Event auf dem `wl_callback`-Objekt, typischerweise synchronisiert mit dem V-Sync-Signal des Monitors.
        
    - **Smithay-Interna:** `smithay::backend::renderer::Frame` (oder ein ähnliches Konzept im gewählten Renderer-Backend) signalisiert oft, wann ein Frame abgeschlossen ist. `SurfaceData` kann eine Liste von ausstehenden Callbacks speichern. Der Compositor-Zustand muss diese Callbacks zum richtigen Zeitpunkt auslösen.
        
- **Rust-Exzellenz:**
    
    - **Timing-Präzision:** Korrekte Integration mit der Render-Pipeline und dem V-Sync-Mechanismus des Backends.
        
    - **Nebenläufigkeit:** Sichere Verwaltung der Callback-Listen und deren Auslösung im Event-Loop.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Verpasste Callbacks:** Können zu ruckelnden Animationen führen.
        
    - **Zu frühe/späte Callbacks:**
        
- **Performance-Optimierung:**
    
    - Minimierung der Latenz zwischen Frame-Fertigstellung und Callback-Benachrichtigung.
        
- **Modulstruktur:** Integration in `novade-system/src/wayland_compositor/surface_management.rs` (Callback-Registrierung) und die Rendering-Pipeline (`novade-system/src/renderer/frame_scheduler.rs`).
    
- **Aufwand:** Mittel (erfordert genaue Synchronisation mit dem Rendering-Backend).
    

### 2.14. `wl_data_device_manager`, `wl_data_device`, `wl_data_source`, `wl_data_offer`

- **Zweck:** Diese Protokollgruppe implementiert die Funktionalität für die Zwischenablage (Copy-Paste) und Drag-and-Drop zwischen Wayland-Clients.
    
    - `wl_data_device_manager`: Global zum Erstellen von `wl_data_device`-Objekten pro Seat.
        
    - `wl_data_device`: Repräsentiert die Fähigkeit eines Seats, Daten auszutauschen. Bietet Operationen für Copy-Paste (`set_selection`) und Drag-and-Drop (`start_drag`).
        
    - `wl_data_source`: Wird von einem Client erstellt, der Daten anbieten möchte (Quelle). Der Client gibt die MIME-Typen an, die er bereitstellen kann, und sendet die Daten, wenn sie angefordert werden.
        
    - `wl_data_offer`: Wird einem Client präsentiert, der Daten empfangen kann (Ziel). Der Client wählt einen der angebotenen MIME-Typen und empfängt die Daten.
        
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor agiert als Vermittler. Er verwaltet die aktuelle Auswahl (Zwischenablage) und den Zustand von Drag-and-Drop-Operationen.
        
    - **Smithay-Interna:** `smithay::wayland::selection::data_device::DataDeviceState` verwaltet den Zustand. `DataDeviceHandler` wird vom Compositor-Zustand implementiert, um auf Client-Anfragen zu reagieren (`set_selection`, `start_drag`) und Events zu senden (`data_offer`, `selection`, `enter`, `leave`, `motion`, `drop`). Die Datenübertragung erfolgt oft über Pipes (Dateideskriptoren), die zwischen den Clients ausgetauscht werden.
        
- **Rust-Exzellenz:**
    
    - **Sichere IPC:** Korrekte Handhabung von Dateideskriptoren und asynchroner Datenübertragung.
        
    - **Zustandsverwaltung:** Robuste Verwaltung der Auswahl und des DND-Zustands.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Abgebrochene Übertragungen:**
        
    - **Ungültige MIME-Typen:**
        
    - **Race Conditions:** Beim Zugriff auf die Auswahl.
        
- **Performance-Optimierung:**
    
    - Effiziente Datenübertragung, insbesondere bei großen Datenmengen.
        
    - Lazy Loading von Daten (Daten werden erst gesendet, wenn der Ziel-Client sie tatsächlich anfordert).
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/data_exchange/` (mit Submodulen für `data_device_global.rs`, `data_source.rs`, `data_offer.rs`).
    
- **Aufwand:** Hoch (komplexes Protokoll mit vielen Interaktionen und Zuständen).
    

## 3. Erweiterte Wayland-Protokolle (Protocol Extensions)

Diese Protokolle erweitern die Kernfunktionalität und sind für eine moderne Desktop-Erfahrung unerlässlich.

### 3.1. `xdg_shell` (Standard-Anwendungsfenster)

- **Zweck:** Definiert das Verhalten von Top-Level-Fenstern (normale Anwendungsfenster) und Popups (Menüs, Tooltips etc.). Es ist der De-facto-Standard für die Fensterverwaltung in modernen Wayland-Desktops. Es umfasst `xdg_wm_base` (Global), `xdg_surface` (Basis für Toplevel/Popup), `xdg_toplevel` und `xdg_popup`.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert `xdg_wm_base` und die zugehörigen Handler. Er verwaltet den Lebenszyklus von XDG-Fenstern, deren Zustände (maximiert, minimiert, Vollbild, aktiviert), Dekorationen (über `zxdg_decoration_manager_v1`) und Interaktionen (Verschieben, Größe ändern).
        
    - **Smithay-Interna:** `smithay::wayland::shell::xdg::XdgShellState` verwaltet das Global. `XdgShellHandler` wird vom Compositor-Zustand implementiert, um auf Client-Anfragen zu reagieren (z.B. `xdg_toplevel.set_title`, `xdg_toplevel.move`). `smithay::desktop::Window` und `smithay::desktop::Space` werden intensiv genutzt, um XDG-Fenster im Desktop-Layout zu verwalten.
        
- **Rust-Exzellenz:**
    
    - **Robuste Zustandsmaschinen:** Für Fensterzustände (aktiv, maximiert etc.) und Konfigurationszyklen (`configure`/`ack_configure`).
        
    - **Klare API-Grenzen:** Zwischen der Protokoll-Implementierung und der eigentlichen Fensterverwaltungslogik in NovaDE.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Protokollverletzungen:** Z.B. ungültige Zustandsübergänge, fehlende `ack_configure`. Smithay sendet hier Protokollfehler.
        
    - **Inkonsistente Fensterzustände:** Vermeidung durch sorgfältige Implementierung der Handler.
        
- **Performance-Optimierung:**
    
    - Effiziente Aktualisierung von Fenstergeometrien und -zuständen.
        
    - Minimierung des Overheads bei `configure`-Events.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/shells/xdg_shell/` (mit Submodulen für `wm_base_global.rs`, `toplevel.rs`, `popup.rs`, `surface.rs`).
    
- **Aufwand:** Sehr Hoch (das komplexeste und wichtigste Shell-Protokoll).
    

### 3.2. `wlr_layer_shell_v1` (Desktop-Elemente)

- **Zweck:** Ermöglicht Clients das Erstellen von Oberflächen, die als Teil der Desktop-Shell fungieren, z.B. Panels, Docks, Hintergrundbilder, Benachrichtigungs-Popups und Sperrbildschirme. Diese Layer-Oberflächen können an Bildschirmkanten verankert werden, exklusiven Platz beanspruchen und ihre Position in der Z-Ordnung (Layer) festlegen.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-UI-Schicht:** Die Shell-Komponenten von NovaDE (Panel, Dock etc.) werden als Layer-Shell-Clients implementiert.
        
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `zwlr_layer_shell_v1`-Global und die zugehörigen Handler. Er verwaltet die Geometrie und Sichtbarkeit der Layer-Oberflächen gemäß den Client-Anfragen und der globalen Desktop-Layout-Policy.
        
    - **Smithay-Interna:** `smithay::wayland::shell::wlr_layer::WlrLayerShellState` verwaltet das Global. `LayerShellHandler` wird vom Compositor-Zustand implementiert. Smithays `desktop::Space` kann Layer-Oberflächen verwalten und deren exklusive Zonen bei der Platzierung normaler Fenster berücksichtigen.
        
- **Rust-Exzellenz:**
    
    - **Korrekte Z-Ordnung:** Sicherstellung, dass Layer-Oberflächen korrekt über oder unter normalen Fenstern gerendert werden.
        
    - **Layout-Konsistenz:** Exklusive Zonen müssen präzise berechnet und vom Fenstermanager berücksichtigt werden.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Ungültige Layer-Spezifikationen:** Z.B. widersprüchliche Anker.
        
    - **Konflikte zwischen Layer-Oberflächen:** Z.B. mehrere Panels, die denselben exklusiven Bereich beanspruchen.
        
- **Performance-Optimierung:**
    
    - Effizientes Compositing der Layer-Oberflächen, insbesondere wenn sie transparent sind oder sich häufig aktualisieren.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/shells/layer_shell/`
    
- **Aufwand:** Hoch (komplexe Layout-Interaktionen mit XDG-Shell-Fenstern).
    

### 3.3. `wp_presentation_time` (Präzise Frame-Synchronisation)

- **Zweck:** Ermöglicht Clients, genaue Zeitstempel und andere Informationen über die tatsächliche Präsentation ihrer Frames auf dem Bildschirm zu erhalten. Dies ist wichtig für flüssige Animationen, Video-Playback und die Synchronisation von Audio und Video.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `wp_presentation`-Global. Wenn ein Client einen Frame committet und Feedback anfordert, merkt sich der Compositor dies. Sobald der Frame tatsächlich auf dem Bildschirm angezeigt wird (oder verworfen wurde), sendet der Compositor ein `feedback`-Event mit den entsprechenden Zeitstempeln (z.B. von DRM/KMS VBlank).
        
    - **Smithay-Interna:** `smithay::wayland::presentation::PresentationState` verwaltet das Global. `PresentationHandler` wird vom Compositor-Zustand implementiert. Die Integration mit dem Renderer-Backend ist entscheidend, um die korrekten Präsentationszeitstempel zu erhalten.
        
- **Rust-Exzellenz:**
    
    - **Exakte Zeitmessung:** Verwendung hochauflösender Zeitquellen vom Backend.
        
    - **Zuverlässiges Event-Dispatching:** Sicherstellung, dass Feedback-Events zeitnah gesendet werden.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Verzögerte oder fehlende Zeitstempel:** Können zu Synchronisationsproblemen beim Client führen.
        
    - **Inkonsistente Zeitstempel:**
        
- **Performance-Optimierung:**
    
    - Minimierung des Overheads bei der Erfassung und Übermittlung von Zeitstempeln.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/rendering/presentation_time.rs` (Integration mit der Frame-Scheduling- und Rendering-Logik).
    
- **Aufwand:** Mittel (erfordert präzise Backend-Integration).
    

### 3.4. `zxdg_decoration_manager_v1` (Client-seitige Fensterdekorationen)

- **Zweck:** Ermöglicht die Aushandlung zwischen Client und Compositor, wer für das Zeichnen der Fensterdekorationen (Titelleiste, Ränder, Buttons) zuständig ist. Clients können Server-Side Decorations (SSD) anfordern, oder der Compositor kann Client-Side Decorations (CSD) bevorzugen.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `zxdg_decoration_manager_v1`-Global. Er entscheidet basierend auf globalen Einstellungen oder pro Fenster, ob SSD oder CSD verwendet wird, und sendet entsprechende `configure`-Events an den Client.
        
    - **Smithay-Interna:** `smithay::wayland::shell::xdg::decoration::XdgDecorationState` verwaltet das Global. Der `XdgShellHandler` (oder ein dedizierter `XdgDecorationHandler`) behandelt die Requests.
        
- **Rust-Exzellenz:**
    
    - **Flexible Konfiguration:** Ermöglicht einfaches Umschalten zwischen CSD und SSD, falls unterstützt.
        
    - **Konsistentes Erscheinungsbild:** Wenn SSD verwendet wird, muss der Compositor Dekorationen zeichnen, die zum NovaDE-Theme passen.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Inkonsistente Dekorationen:** Wenn Client und Server sich nicht auf einen Modus einigen können.
        
- **Performance-Optimierung:**
    
    - Wenn SSD verwendet wird, muss das Rendern der Dekorationen effizient sein.
        
- **Modulstruktur:** Integration in `novade-system/src/wayland_compositor/shells/xdg_shell/decoration_manager.rs`.
    
- **Aufwand:** Mittel (Protokoll ist einfach, aber SSD-Rendering kann aufwendig sein).
    

### 3.5. `wp_fractional_scale_v1` (Fractional Scaling)

- **Zweck:** Ermöglicht Clients, ihre Oberflächen für nicht-ganzzahlige Skalierungsfaktoren (z.B. 125%, 150%) zu rendern, um auf HiDPI-Displays eine schärfere Darstellung als mit reiner Bitmap-Skalierung zu erreichen.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor bewirbt unterstützte fraktionale Skalierungsfaktoren pro `wl_output`. Wenn ein Client eine Oberfläche mit einem fraktionalen Skalierungsfaktor committet, muss der Compositor diese Information an den Renderer weitergeben.
        
    - **Smithay-Interna:** `smithay::wayland::output::ScaleManager` (oder eine ähnliche Abstraktion) könnte verwendet werden. Die `wl_surface.set_buffer_scale`-Anfrage wird relevant. Der Renderer muss Oberflächen mit nicht-ganzzahligen Skalierungsfaktoren korrekt verarbeiten können (oft durch Über-Rendering seitens des Clients und anschließendes Downscaling durch den Compositor oder direkte Unterstützung im Toolkit).
        
- **Rust-Exzellenz:**
    
    - **Präzise Skalierung:** Korrekte mathematische Behandlung der Skalierungsfaktoren.
        
    - **Nahtlose Integration:** Mit dem Renderer und der GTK4-Skalierungslogik (falls GTK4 für Shell-Elemente verwendet wird).
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Nicht unterstützte Skalierungsfaktoren:**
        
    - **Visuelle Artefakte:** Bei falscher Implementierung.
        
- **Performance-Optimierung:**
    
    - Kann zu erhöhtem Rendering-Aufwand führen. Effiziente Skalierungsalgorithmen im Compositor sind wichtig.
        
- **Modulstruktur:** Integration in `novade-system/src/wayland_compositor/output_management.rs` und die Rendering-Pipeline.
    
- **Aufwand:** Mittel bis Hoch (Rendering-Implikationen sind komplex).
    

### 3.6. `wp_viewport_v1` (Surface Viewports)

- **Zweck:** Erlaubt Clients, nur einen Teil einer `wl_surface` (Quell-Rechteck) auf einen bestimmten Bereich einer anderen Größe (Ziel-Rechteck) zu skalieren und zu beschneiden, ohne den Inhalt des zugrundeliegenden `wl_buffer` ändern zu müssen. Nützlich für Cropping und Skalierung von Oberflächeninhalten.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `wp_viewporter`-Global. Wenn ein Client `wp_viewport.set_source` oder `set_destination` aufruft, muss der Compositor diese Informationen beim Rendern der Oberfläche berücksichtigen.
        
    - **Smithay-Interna:** `smithay::wayland::viewporter::ViewporterState` verwaltet das Global. `ViewportHandler` wird vom Compositor-Zustand implementiert. Die Rendering-Pipeline muss die Viewport-Parameter (Quell- und Zielrechtecke) anwenden.
        
- **Rust-Exzellenz:**
    
    - **Präzise Transformationen:** Korrekte Anwendung der Viewport-Transformationen auf der GPU.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Ungültige Viewport-Dimensionen:** Z.B. Quellrechteck außerhalb des Puffers.
        
- **Performance-Optimierung:**
    
    - Kann von Clients zur Optimierung genutzt werden (nur relevante Teile rendern). Der Compositor muss die Transformationen effizient durchführen.
        
- **Modulstruktur:** Integration in `novade-system/src/wayland_compositor/surface_management.rs` und die Rendering-Pipeline.
    
- **Aufwand:** Mittel.
    

### 3.7. `zxdg_output_manager_v1` (Erweitertes Output-Management)

- **Zweck:** Eine Ergänzung zu `wl_output`, die detailliertere und stabilere Informationen über Ausgabegeräte bereitstellt, wie Name, Beschreibung und logische Größe/Position. Dies ist nützlich für Clients, die konsistente Informationen über Outputs über Sitzungen hinweg benötigen.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `zxdg_output_manager_v1`-Global. Für jeden `wl_output` erstellt er ein entsprechendes `zxdg_output_v1`-Objekt und sendet dessen Eigenschaften.
        
    - **Smithay-Interna:** `smithay::wayland::output::xdg::XdgOutputManagerState` (oder ähnlich) verwaltet das Global. Der `OutputHandler` wird erweitert, um auch `zxdg_output_v1`-Objekte und deren Events zu verwalten.
        
- **Rust-Exzellenz:**
    
    - **Konsistente Zustandsverwaltung:** Synchronisation der Informationen zwischen `wl_output` und `zxdg_output_v1`.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Inkonsistente Output-Informationen:**
        
- **Performance-Optimierung:**
    
    - Gering, da primär informativ.
        
- **Modulstruktur:** Integration in `novade-system/src/wayland_compositor/output_management.rs`.
    
- **Aufwand:** Gering bis Mittel.
    

### 3.8. `zwp_linux_dmabuf_v1` (Direct Memory Access Buffer)

- **Zweck:** Ermöglicht Clients, Grafikpuffer direkt aus dem GPU-Speicher (DMA-BUF) mit dem Compositor zu teilen. Dies ist der Schlüssel für Zero-Copy-Rendering, da Daten nicht zwischen CPU- und GPU-Speicher kopiert werden müssen. Extrem wichtig für Performance, insbesondere bei grafikintensiven Anwendungen und Spielen.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `zwp_linux_dmabuf_v1`-Global. Er empfängt DMA-BUF-Dateideskriptoren, Ebenen (planes), Offsets, Strides und Modifikatoren vom Client. Er muss diese DMA-BUFs validieren und in eine vom Renderer nutzbare Textur importieren (z.B. über EGL und `eglCreateImageKHR` mit `EGL_LINUX_DMA_BUF_EXT`).
        
    - **Smithay-Interna:** `smithay::wayland::dmabuf::DmabufState` verwaltet das Global und die unterstützten Formate/Modifier. `DmabufHandler` wird vom Compositor-Zustand implementiert. Tiefe Integration mit dem DRM/KMS-Backend und dem Renderer ist erforderlich.
        
- **Rust-Exzellenz:**
    
    - **Sichere FFI-Handhabung:** Sicherer Umgang mit Dateideskriptoren und Grafik-API-Aufrufen (EGL, GBM).
        
    - **Vermeidung von Use-After-Free:** Korrekte Verwaltung der Lebensdauer von DMA-BUF-Handles und importierten Texturen.
        
- **Fehlerbehandlung und Stabilität:**
    
    - **Ungültige DMA-BUF-Importe:** Fehler beim Importieren des DMA-BUF durch den Renderer.
        
    - **Nicht unterstützte Formate/Modifier:**
        
    - **Treiberprobleme:** DMA-BUF ist stark von der Treiberqualität abhängig.
        
- **Performance-Optimierung:**
    
    - **Zero-Copy:** Das Hauptziel. Direkte Nutzung der GPU-Puffer ohne Kopien.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/buffers/dmabuf.rs` (Protokoll-Implementierung) und tiefe Integration in `novade-system/src/renderer/`.
    
- **Aufwand:** Sehr Hoch (technisch sehr anspruchsvoll, erfordert tiefes Verständnis von Linux-Grafik-Interna).
    

### 3.9. `wp_single_pixel_buffer_v1`

- **Zweck:** Ermöglicht das Erstellen von 1x1 Pixel großen Puffern mit einer einzelnen Farbe. Dies ist eine Optimierung für das Rendern kleiner, einfarbiger Flächen (z.B. einfarbiger Cursor, einfarbige Hintergründe für bestimmte UI-Elemente), da kein echter Speicher-Puffer alloziert und übertragen werden muss.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `wp_single_pixel_buffer_manager_v1`-Global. Wenn ein Client einen solchen Puffer anfordert, merkt sich der Compositor die Farbe.
        
    - **Smithay-Interna:** `smithay::wayland::buffer::single_pixel_buffer::SinglePixelBufferState` (oder ähnlich). Der Renderer muss in der Lage sein, diese "virtuellen" Puffer direkt mit der angegebenen Farbe zu zeichnen, ohne eine Textur zu erwarten.
        
- **Rust-Exzellenz:**
    
    - Effiziente Übergabe der Farbinformation an den Renderer.
        
- **Fehlerbehandlung und Stabilität:**
    
    - Ungültige Farbangaben (sollte vom Protokoll verhindert werden).
        
- **Performance-Optimierung:**
    
    - Reduziert Speicherbandbreite und Allokations-Overhead für kleine einfarbige Flächen.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/buffers/single_pixel.rs`.
    
- **Aufwand:** Gering bis Mittel.
    

### 3.10. `wp_relative_pointer_manager_v1`, `wp_locked_pointer_manager_v1` (Spiele und Kiosk-Modus)

- **Zweck:**
    
    - `wp_relative_pointer_manager_v1`: Stellt relative Mausbewegungs-Events bereit, die unabhängig von der absoluten Cursorposition sind. Wichtig für Spiele (z.B. Ego-Shooter) und 3D-Anwendungen.
        
    - `wp_locked_pointer_manager_v1`: Erlaubt das "Einsperren" des Cursors in einem bestimmten Bereich oder das Verstecken des Cursors, während weiterhin relative Bewegungen gemeldet werden. Wichtig für immersive Anwendungen und Kiosk-Modi.
        
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert die Manager-Globals. Wenn ein Client relative oder gesperrte Zeiger-Events anfordert, muss der Compositor diese Modi aktivieren und die entsprechenden Events (relative Deltas, gesperrt/entsperrt-Status) senden.
        
    - **Smithay-Interna:** `smithay::input::pointer_constraints::PointerConstraintsState` (oder ähnlich). Enge Integration mit dem `libinput`-Backend, um relative Bewegungsdaten zu erhalten, auch wenn der Cursor am Bildschirmrand "feststeckt".
        
- **Rust-Exzellenz:**
    
    - Akkurate Transformation von Input-Events.
        
    - Robuste Implementierung der Lock-Mechanismen.
        
- **Fehlerbehandlung und Stabilität:**
    
    - Konflikte zwischen mehreren Clients, die Pointer-Constraints anfordern.
        
    - Nicht korrekt freigegebene Locks.
        
- **Performance-Optimierung:**
    
    - Geringe Latenz bei der Übermittlung relativer Bewegungen.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/input/pointer_constraints.rs`.
    
- **Aufwand:** Mittel.
    

### 3.11. `input_method_unstable_v2` und `text_input_unstable_v3` (Eingabemethoden)

- **Zweck:** Ermöglichen komplexe Texteingaben, die über einfache Tastatur-Layouts hinausgehen, z.B. für ostasiatische Sprachen (IME - Input Method Editor) oder spezielle Symbolpaletten.
    
    - `input_method_unstable_v2`: Definiert die Kommunikation zwischen einem Input-Method-Editor (als Wayland-Client) und dem Compositor.
        
    - `text_input_unstable_v3`: Definiert die Kommunikation zwischen einer Anwendung, die Texteingabe empfängt, und dem Compositor.
        
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor agiert als Vermittler zwischen dem IME und der fokussierten Anwendung. Er implementiert die Manager-Globals. Er leitet Preedit-Strings, Commit-Strings und andere IME-Zustände weiter.
        
    - **Smithay-Interna:** `smithay::wayland::input_method::InputMethodManagerState` und `smithay::wayland::text_input::TextInputManagerState`. Die Handler müssen den Zustand der Texteingabe (Cursor-Position, umgebender Text) und die IME-Aktionen verwalten.
        
- **Rust-Exzellenz:**
    
    - Korrekte Verarbeitung von UTF-8 und komplexen Textzuständen.
        
- **Fehlerbehandlung und Stabilität:**
    
    - Inkonsistente Zustände zwischen IME, Compositor und Anwendung.
        
    - Verlorene oder falsch weitergeleitete Texteingabe-Events.
        
- **Performance-Optimierung:**
    
    - Effiziente Weiterleitung von Text-Events.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/input/input_method.rs`.
    
- **Aufwand:** Sehr Hoch (sehr komplexe Protokolle mit vielen Zuständen und Interaktionen).
    

### 3.12. `xdg_activation_v1` (Fensteraktivierung)

- **Zweck:** Ermöglicht es Clients, die Aktivierung (Fokussetzung) eines anderen Fensters auf eine benutzergesteuerte Weise anzufordern. Dies verhindert, dass Anwendungen willkürlich den Fokus stehlen.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `xdg_activation_v1`-Global. Wenn ein Client eine Aktivierung anfordert, validiert der Compositor dies (z.B. ob es eine zugehörige Benutzerinteraktion gab) und setzt dann den Fokus entsprechend.
        
    - **Smithay-Interna:** `smithay::wayland::xdg_activation::XdgActivationState`. Der Handler muss Aktivierungs-Tokens verwalten und Anfragen validieren.
        
- **Rust-Exzellenz:**
    
    - Sichere Implementierung der Token-Validierung.
        
- **Fehlerbehandlung und Stabilität:**
    
    - Missbrauch durch Clients, die versuchen, den Fokus zu stehlen.
        
- **Performance-Optimierung:**
    
    - Gering.
        
- **Modulstruktur:** Integration in `novade-system/src/wayland_compositor/shells/xdg_shell/activation.rs` oder `novade-system/src/input_manager/focus.rs`.
    
- **Aufwand:** Mittel.
    

### 3.13. `wlr_foreign_toplevel_management_unstable_v1` (Fensterliste für externe Tools)

- **Zweck:** Erlaubt externen Clients (z.B. Taskleisten, Docks, Fensterwechsler, die nicht Teil der primären Shell sind) Informationen über Top-Level-Fenster zu erhalten und diese rudimentär zu steuern (z.B. Aktivierung, Schließen).
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `zwlr_foreign_toplevel_manager_v1`-Global. Er sendet Events über erstellte, zerstörte und geänderte Top-Level-Fenster.
        
    - **Smithay-Interna:** `smithay::wayland::foreign_toplevel::ForeignToplevelState` (oder ähnlich). Erfordert Zugriff auf die Liste der `xdg_toplevel`-Fenster.
        
- **Rust-Exzellenz:**
    
    - Konsistente Bereitstellung von Fensterinformationen.
        
- **Fehlerbehandlung und Stabilität:**
    
    - Bereitstellung veralteter Informationen.
        
- **Performance-Optimierung:**
    
    - Effiziente Benachrichtigung über Fensteränderungen.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/utils/foreign_toplevel.rs`.
    
- **Aufwand:** Mittel.
    

### 3.14. `idle_notify_unstable_v1` (Inaktivitätsbenachrichtigungen)

- **Zweck:** Ermöglicht es Clients, vom Compositor benachrichtigt zu werden, wenn der Benutzer für eine bestimmte Zeit inaktiv war oder wieder aktiv wird. Nützlich für Bildschirmschoner, automatische Sperrbildschirme oder Energiesparmaßnahmen.
    
- **Interaktion mit NovaDE/Smithay:**
    
    - **NovaDE-Systemschicht:** Der Compositor implementiert das `zwp_idle_notifier_v1`-Global. Er überwacht Benutzereingaben (über `wl_seat`) und startet Timer für Inaktivität. Wenn ein Timeout erreicht wird, sendet er `idle`-Events an registrierte Clients. Bei neuer Aktivität sendet er `resumed`-Events.
        
    - **Smithay-Interna:** `smithay::wayland::idle_notify::IdleNotifierState`. Erfordert Integration mit dem Input-System und Timer-Management (z.B. über `calloop`).
        
- **Rust-Exzellenz:**
    
    - Zuverlässiges Timer-Management.
        
- **Fehlerbehandlung und Stabilität:**
    
    - Falsche Erkennung von Aktivität/Inaktivität.
        
- **Performance-Optimierung:**
    
    - Effiziente Timer.
        
- **Modulstruktur:** `novade-system/src/wayland_compositor/utils/idle_notify.rs`.
    
- **Aufwand:** Mittel.
    

## 4. Wayland-Protokoll-Interaktionen und Systemintegration

### 4.1. Lebenszyklus eines Fensters (XDG-Shell, wl_surface, wl_buffer, wl_callback)

1. **Client-Start & Registry-Bindung:** Client verbindet sich zum Compositor, erhält `wl_display`, bindet an `wl_registry`.
    
2. **Global-Discovery:** Client empfängt `global`-Events für `wl_compositor`, `wl_shm`, `xdg_wm_base` etc. und bindet an diese.
    
3. **Oberflächenerstellung:**
    
    - Client sendet `wl_compositor.create_surface`, erhält neue `wl_surface` (ID: S1).
        
    - Client sendet `xdg_wm_base.get_xdg_surface(S1)`, erhält `xdg_surface` (ID: XS1).
        
    - Client sendet `xdg_surface.get_toplevel(XS1)`, erhält `xdg_toplevel` (ID: XT1).
        
4. **Puffererstellung und -anhang:**
    
    - Client erstellt `wl_shm_pool` (oder `zwp_linux_dmabuf_v1`-Puffer).
        
    - Client erstellt `wl_buffer` (ID: B1) aus dem Pool, rendert Inhalt hinein.
        
    - Client sendet `wl_surface.attach(S1, B1, x, y)`.
        
    - Client sendet `wl_surface.damage_buffer(S1, dx, dy, width, height)` (oder `damage_surface`).
        
5. **Konfiguration und Commit:**
    
    - Compositor (NovaDE/Smithay) sendet `xdg_toplevel.configure(XT1, width, height, states, serial)`.
        
    - Client sendet `xdg_surface.ack_configure(XS1, serial)`.
        
    - Client fordert optional `wl_surface.frame(S1)`, erhält `wl_callback` (ID: C1).
        
    - Client sendet `wl_surface.commit(S1)`.
        
6. **Compositor-Verarbeitung und Rendering:**
    
    - NovaDE/Smithay (`CompositorHandler::commit`, `XdgShellHandler`) verarbeitet den Commit. Der neue Puffer B1 wird zum aktuellen Puffer von S1.
        
    - Die NovaDE-Rendering-Pipeline zeichnet den Inhalt von B1 für Oberfläche S1.
        
    - Domänenschicht (`WindowManagementPolicyService`, `WorkspaceManagerService`) wird über das neue/aktualisierte Fenster informiert. UI-Schicht (GTK4-Shell) aktualisiert ggf. Taskleiste etc.
        
7. **Frame-Callback:**
    
    - Nachdem der Frame mit dem Inhalt von B1 dargestellt wurde, sendet der Compositor `wl_callback.done(C1, time)`.
        
    - Client kann nun den nächsten Frame vorbereiten und `wl_buffer.release(B1)` senden (falls der Puffer nicht mehr benötigt wird oder wiederverwendet werden soll).
        
8. **Fensterinteraktionen (Beispiele):**
    
    - **Verschieben:** Client sendet `xdg_toplevel.move(XT1, seat, serial_des_grab_events)`. Compositor startet interaktiven Move.
        
    - **Schließen:** Client sendet `xdg_toplevel.destroy(XT1)`. Compositor zerstört das Fenster und zugehörige Ressourcen.
        

- **Fehlerbehandlung:** Protokollfehler bei ungültigen Sequenzen oder Parametern führen zum Verbindungsabbruch für den Client. Unerwartetes Schließen des Clients führt zur Zerstörung seiner Ressourcen durch den Compositor.
    

### 4.2. Input-Verarbeitungskette (wl_seat, wl_keyboard, wl_pointer, libinput, NovaDE Input Service)

1. **Hardware-Event:** `libinput` (im `novade-system/src/input_manager/backend/libinput_backend.rs`) empfängt ein rohes Input-Event (z.B. Tastendruck `KEY_A`).
    
2. **Smithay-Input-Backend:** Der `LibinputInputBackend` von Smithay verarbeitet das `libinput`-Event.
    
3. **NovaDE `InputHandler` (Teil von `DesktopState` oder delegiert):**
    
    - Empfängt das Smithay-Input-Event.
        
    - Transformiert es in ein internes NovaDE-Eingabeformat, falls nötig.
        
    - Interagiert mit `xkbcommon` (für Tastatur-Events), um Keycode in Keysym/UTF-8 zu übersetzen und Modifier-Status zu aktualisieren.
        
4. **Fokus-Management (NovaDE `FocusManager` in `input_manager/focus.rs`, interagiert mit `DesktopState.seat`):**
    
    - Ermittelt das aktuell fokussierte `WindowElement` (XDG-Toplevel, Layer-Surface etc.) basierend auf Zeigerposition (für Maus) oder explizitem Fokus (für Tastatur).
        
    - Nutzt `smithay::desktop::Space::surface_under()` oder ähnliche Mechanismen.
        
5. **Wayland-Event-Dispatch (über `DesktopState.seat` und die Handles `KeyboardHandle`, `PointerHandle`):**
    
    - `KeyboardHandle::key()`: Sendet `wl_keyboard.key` an die fokussierte `wl_surface`.
        
    - `PointerHandle::motion()`: Sendet `wl_pointer.motion` und ggf. `enter`/`leave` an die Oberflächen unter dem Zeiger.
        
    - Analog für `button`, `axis`, `touch`-Events.
        
6. **Interaktion mit NovaDE Domänen-Schicht:**
    
    - Ein `HotkeyDaemonService` (Domänenschicht) könnte globale Tastenkombinationen registrieren. Der `InputHandler` prüft, ob ein Event einer globalen Kombination entspricht, bevor es an einen Client gesendet wird. Wenn ja, wird die Aktion im `HotkeyDaemonService` ausgelöst.
        
    - Der `WorkspaceManagerService` wird über Fokusänderungen informiert, um z.B. das aktive Fenster im Workspace zu aktualisieren.
        

- **Fehlerbehandlung:** Event-Dropping bei Überlast (selten mit `libinput`), falscher Fokus, Race Conditions (z.B. wenn sich der Fokus ändert, während ein Event verarbeitet wird).
    

### 4.3. Multi-Monitor-Management (wl_output, zxdg_output_manager_v1, DRM/KMS, NovaDE Display Manager Service)

1. **Hardware-Erkennung (DRM/KMS-Backend in `novade-system/src/renderer/backends/drm_backend.rs`):**
    
    - Das DRM-Backend erkennt beim Start und durch Hotplug-Events (via `udev`) angeschlossene Monitore.
        
    - Für jeden Monitor werden dessen Eigenschaften (Name, Modi, aktuelle Auflösung, Position etc.) ausgelesen.
        
2. **Smithay `Output`-Erstellung (im `OutputHandler` von `DesktopState`):**
    
    - Für jeden physischen Monitor wird ein `smithay::output::Output`-Objekt erstellt und im `DesktopState.output_manager_state` (oder einer ähnlichen Struktur) verwaltet.
        
    - Das `wl_output`-Global wird für Clients erstellt.
        
3. **NovaDE `DisplayManagerService` (Domänenschicht):**
    
    - Wird über `OutputAdded` / `OutputRemoved` / `OutputModeChanged`-Events (vom Compositor über den `SystemEventBridge`) informiert.
        
    - Verwaltet die logische Konfiguration der Outputs (welcher ist primär, Anordnung, Skalierung pro Output). Diese Konfiguration wird persistiert.
        
    - Stellt eine API für das `novade-ui/src/control_center` bereit, um Monitoreinstellungen zu ändern.
        
4. **Anwendung der Konfiguration (Compositor):**
    
    - Wenn der `DisplayManagerService` eine neue Konfiguration anwendet, sendet er einen Befehl an den Compositor (Systemschicht).
        
    - Der Compositor (speziell `output_management.rs`) ruft `Output::change_current_state()` auf, um den Modus, die Position und Skalierung der `smithay::output::Output`-Objekte zu ändern. Dies interagiert mit dem DRM-Backend, um die Hardware-Einstellungen anzupassen.
        
5. **Wayland-Protokoll-Updates:**
    
    - Der Compositor sendet `wl_output.geometry`, `wl_output.mode`, `wl_output.scale`, `wl_output.done` Events an alle Clients, um sie über die (neue) Konfiguration jedes Outputs zu informieren.
        
    - Wenn `zxdg_output_manager_v1` unterstützt wird, werden zusätzlich `zxdg_output_v1.logical_position`, `logical_size` etc. Events gesendet.
        
6. **Client-Reaktion:** Wayland-Clients (Anwendungen, Shell) empfangen die Output-Events und passen ihr Layout und Rendering entsprechend an (z.B. Fenstergröße ändern, wenn sich die Output-Auflösung ändert, auf dem richtigen Monitor positionieren).
    

- **Fehlerbehandlung:** Ungültige Mode-Sets durch den Benutzer, Treiberprobleme beim Setzen von Modi, Fehler bei der Hotplug-Erkennung.
    

### 4.4. Drag-and-Drop / Copy-Paste-Fluss (wl_data_device, wl_data_source, wl_data_offer)

1. **Quelle bietet Daten an (z.B. Client A kopiert Text):**
    
    - Client A erstellt ein `wl_data_source`-Objekt.
        
    - Client A ruft `wl_data_source.offer(mime_type)` für jeden unterstützten MIME-Typ auf (z.B. "text/plain;charset=utf-8").
        
    - Client A ruft `wl_data_device.set_selection(source, serial)` auf (für Copy-Paste) oder `wl_data_device.start_drag(source, origin_surface, icon_surface, serial)` (für DND).
        
2. **Compositor-Verarbeitung (NovaDE/Smithay `DataDeviceHandler`):**
    
    - **Copy-Paste:** Der Compositor merkt sich die `wl_data_source` als aktuelle Auswahl.
        
    - **DND:** Der Compositor startet den DND-Modus, zeigt ggf. das `icon_surface` unter dem Cursor.
        
3. **Ziel empfängt Angebot (z.B. Client B, über dem der Mauszeiger ist, oder das den Fokus hat):**
    
    - Compositor sendet `wl_data_device.data_offer(new_id_offer)` an Client B.
        
    - Compositor sendet `wl_data_offer.offer(mime_type)` für jeden vom Quell-Client angebotenen MIME-Typ an das neue `wl_data_offer`-Objekt von Client B.
        
    - **DND:** Compositor sendet `wl_data_device.enter(serial, surface, x, y, offer)` an Client B, wenn der Drag über eine seiner Oberflächen eintritt. `wl_data_device.motion` bei Bewegung.
        
4. **Ziel akzeptiert Daten:**
    
    - Client B wählt einen bevorzugten MIME-Typ und ruft `wl_data_offer.accept(serial, mime_type)` auf.
        
    - **DND:** Client B ruft `wl_data_offer.set_actions(dnd_actions, preferred_action)` auf, um unterstützte Aktionen mitzuteilen. Compositor sendet `wl_data_source.target(mime_type)` (bei Copy-Paste) oder `wl_data_source.action(dnd_action)` (bei DND) an Client A.
        
    - Client B ruft `wl_data_offer.receive(mime_type, fd)` auf und stellt einen Dateideskriptor zum Schreiben bereit.
        
5. **Datenübertragung:**
    
    - Compositor leitet den `fd` an Client A weiter (sendet `wl_data_source.send(mime_type, fd)`).
        
    - Client A schreibt die Daten in den `fd` und schließt ihn.
        
    - Client B liest die Daten aus dem `fd` und schließt ihn.
        
6. **Abschluss:**
    
    - Client B ruft `wl_data_offer.finish()` auf (bei DND).
        
    - Compositor sendet `wl_data_source.finished()` oder `cancelled()` an Client A.
        
    - Compositor sendet `wl_data_device.leave()` an Client B (bei DND).
        
    - Alle relevanten Objekte (`wl_data_source`, `wl_data_offer`) werden zerstört.
        

- **Interaktion mit NovaDE Domänen-Schicht:** Ein `ClipboardManagerService` (Domänenschicht) könnte den Inhalt der Zwischenablage (die Daten selbst, nicht nur die `wl_data_source`) zwischenspeichern, um ihn auch für Nicht-Wayland-Anwendungen oder NovaDE-interne Funktionen verfügbar zu machen. Der Compositor würde mit diesem Service interagieren.
    
- **Fehlerbehandlung:** Abgebrochene Übertragungen, nicht unterstützte MIME-Typen, Fehler beim FD-Handling.
    

## 5. Rust-spezifische Implementierungsdetails und Qualitätsstandards

Die Implementierung aller Wayland-Protokolle und der zugehörigen Logik in NovaDE muss den in der `Gesamtspezifikation.md` definierten "Rust-spezifischen Excellence-Standards" strikt folgen. Diese Standards sind nicht nur Richtlinien, sondern fundamentale Prinzipien, die die Robustheit, Sicherheit und Wartbarkeit des gesamten Systems gewährleisten. Jeder dieser Standards erfordert eine explizite und methodische Herangehensweise bei der Implementierung.

- **Zero Unsafe Code (Wo immer möglich und praktisch):**
    
    - **Definition und Zielsetzung:** Das primäre Ziel ist die vollständige Eliminierung von `unsafe`-Blöcken im NovaDE-Compositor-Code, es sei denn, eine Interaktion mit nicht-Rust-Code (FFI) oder eine hardwarenahe Optimierung, deren Sicherheit nicht vom Rust-Compiler verifiziert werden kann, ist absolut unumgänglich. Jede Verwendung von `unsafe` stellt eine potenzielle Schwachstelle dar und untergräbt die Garantien, die Rust normalerweise bietet.
        
    - **Identifikationsprozess für notwendiges `unsafe`:**
        
        1. **FFI-Analyse:** Identifizieren Sie alle externen C-Bibliotheken, die für die Compositor-Funktionalität benötigt werden (z.B. `libinput`, `libxkbcommon`, `libdrm`, `libEGL`, `libgbm`).
            
        2. **Funktions-Mapping:** Für jede benötigte Funktion aus diesen Bibliotheken, überprüfen Sie, ob bereits sichere Rust-Bindings (Wrapper-Crates) existieren (z.B. `input-rs` für `libinput`, `xkbcommon-rs` für `libxkbcommon`).
            
        3. **Sicherheitsbewertung existierender Bindings:** Falls Bindings existieren, bewerten Sie deren Sicherheit und Vollständigkeit. Sind sie aktiv gewartet? Markieren sie ihre FFI-Aufrufe korrekt als `unsafe` und bieten sie eine sichere Abstraktion darüber?
            
        4. **Notwendigkeit eigener Bindings:** Falls keine adäquaten, sicheren Bindings existieren oder spezifische, nicht abgedeckte Funktionen benötigt werden, müssen eigene FFI-Bindings erstellt werden. Diese sind inhärent `unsafe`.
            
            - **Schritt 1: Deklaration des `extern "C"`-Blocks:**
                
                ```
                // In novade-system/src/ffi/drm_bindings.rs (Beispiel)
                extern "C" {
                    pub fn drmModeGetResources(fd: ::std::os::raw::c_int) -> *mut drmModeRes;
                    pub fn drmModeFreeResources(ptr: *mut drmModeRes);
                    // ... weitere DRM-Funktionen
                }
                ```
                
            - **Schritt 2: Erstellung sicherer Wrapper-Funktionen:** Jede `unsafe extern`-Funktion muss in einer Rust-Funktion gekapselt werden, die die `unsafe`-Aufrufe isoliert und nach außen hin eine sichere Schnittstelle bietet. Diese Wrapper müssen alle Sicherheitsinvarianten der C-Funktion prüfen und durchsetzen (z.B. Null-Pointer-Prüfungen, korrekte Ressourceneigentümerschaft).
                
                ```
                // In novade-system/src/backends/drm/device.rs (Beispiel)
                pub fn get_drm_resources(fd: RawFd) -> Result<*mut drmModeRes, DrmError> {
                    // SAFETY: Der Aufruf von drmModeGetResources ist sicher, wenn fd ein gültiger
                    // DRM-Dateideskriptor ist. Der zurückgegebene Zeiger muss später mit
                    // drmModeFreeResources freigegeben werden. Der Aufrufer ist dafür
                    // verantwortlich, dass fd gültig bleibt, solange der Zeiger verwendet wird.
                    // Null-Pointer-Prüfung ist erforderlich.
                    let resources_ptr = unsafe { ffi::drm_bindings::drmModeGetResources(fd) };
                    if resources_ptr.is_null() {
                        Err(DrmError::GetResourcesFailed(std::io::Error::last_os_error()))
                    } else {
                        Ok(resources_ptr)
                    }
                }
                ```
                
        5. **Hardware-Interaktion:** Für direkte Hardware-Interaktionen, die nicht über etablierte Bibliotheken laufen (z.B. spezielle `ioctl`-Aufrufe für nicht-standardisierte Features), sind `unsafe`-Blöcke notwendig. Diese müssen extrem sorgfältig dokumentiert und geprüft werden.
            
    - **Dokumentationspflicht für `unsafe`:**
        
        1. **`SAFETY` Kommentarblock:** Jeder `unsafe`-Block oder jede `unsafe fn` _muss_ direkt davor einen Kommentarblock mit der Überschrift `// SAFETY:` haben.
            
        2. **Invarianten:** Dieser Block _muss_ detailliert auflisten, welche Bedingungen (Invarianten) vom Programmierer manuell sichergestellt werden, die der Rust-Compiler nicht verifizieren kann.
            
        3. **Begründung:** Es _muss_ erklärt werden, warum der `unsafe`-Block notwendig ist und warum die Operationen darin trotz fehlender Compiler-Garantien als korrekt angesehen werden.
            
    - **Minimierung und Isolation:**
        
        1. **Granularität:** `unsafe`-Blöcke müssen so klein wie möglich sein und nur die absolut notwendigen Operationen umschließen.
            
        2. **Abstraktion:** Kapseln Sie `unsafe`-Operationen in separaten, eng fokussierten Funktionen oder Modulen. Diese Module sollten eine sichere öffentliche API bereitstellen, sodass der Rest des Codes `unsafe`-frei bleiben kann.
            
        
        ```
        // Modul novade-system/src/utils/raw_memory_access.rs
        // Enthält unsichere Funktionen zum direkten Speicherzugriff,
        // aber bietet sichere Wrapper an.
        // pub unsafe fn read_volatile<T: Copy>(source: *const T) -> T { ... }
        // pub fn safe_read_device_register(address: usize) -> Result<u32, MemoryAccessError> {
        //     // Interne Nutzung von read_volatile mit Adressvalidierung etc.
        // }
        ```
        
    - **Überprüfungstools:**
        
        1. **`cargo geiger` Ausführung:** Integrieren Sie `cargo geiger` in den CI-Prozess, um die Anzahl und den Kontext von `unsafe`-Verwendungen kontinuierlich zu überwachen.
            
        2. **Manuelle Reviews:** Jeder `unsafe`-Block unterliegt einer obligatorischen Code-Review durch mindestens zwei erfahrene Entwickler.
            
- **RAII (Resource Acquisition Is Initialization):**
    
    - **Definition und Prinzip:** Dieses fundamentale Rust-Prinzip _muss_ konsequent für _alle_ Ressourcen angewendet werden, deren Lebenszyklus über einen einzelnen Scope hinausgeht oder eine explizite Freigabe erfordert. Dies umfasst, ist aber nicht beschränkt auf:
        
        - Dateideskriptoren (Shared Memory FDs, DMA-BUF FDs, Pipe FDs, Socket FDs).
            
        - Speicher-Mappings (erhalten von `mmap`).
            
        - GPU-Ressourcen (Vulkan: `VkBuffer`, `VkImage`, `VkDeviceMemory`, `VkSemaphore`, `VkFence`, `VkCommandBuffer`; OpenGL: Textur-IDs, Buffer-IDs, Shader-Programm-IDs).
            
        - Wayland-Objekte, die nicht automatisch von Smithay verwaltet werden (selten, aber möglich bei benutzerdefinierten Protokollerweiterungen ohne Smithay-Unterstützung).
            
        - Handles von externen Bibliotheken (z.B. `xkb_context`, `xkb_keymap`, `xkb_state` von `libxkbcommon`).
            
    - **Implementierungsschritte:**
        
        1. **Struct-Definition:** Für jede zu verwaltende Ressource wird ein dediziertes Rust-Struct (Wrapper-Typ) definiert, das die Ressource (oder deren Handle/ID) als Feld enthält.
            
            ```
            // Beispiel für einen DMA-BUF File Descriptor Wrapper
            // In novade-system/src/buffers/dmabuf_fd.rs
            use std::os::unix::io::RawFd;
            
            pub struct DmabufFd {
                fd: RawFd,
                // Optional: Metadaten wie Größe, Format, etc.
            }
            
            impl DmabufFd {
                pub fnnew(fd: RawFd) -> Self {
                    // Validierung des FDs könnte hier erfolgen
                    DmabufFd { fd }
                }
                pub fn raw_fd(&self) -> RawFd { self.fd }
            }
            ```
            
        2. **`Drop` Trait Implementierung:** Für das definierte Struct _muss_ der `Drop` Trait implementiert werden. Die `drop` Methode _muss_ die notwendigen Operationen zur Freigabe der Ressource enthalten.
            
            ```
            // Fortsetzung von DmabufFd
            impl Drop for DmabufFd {
                fn drop(&mut self) {
                    // SAFETY: Der Aufruf von close ist sicher, wenn self.fd ein gültiger,
                    // offener Dateideskriptor ist, der dieser DmabufFd-Instanz exklusiv gehört.
                    // Nach dem close darf der FD nicht mehr verwendet werden.
                    let result = unsafe { libc::close(self.fd) };
                    if result != 0 {
                        // Loggen des Fehlers, aber nicht paniken in drop!
                        log::error!("Failed to close DMA-BUF fd {}: {}", self.fd, std::io::Error::last_os_error());
                    } else {
                        log::debug!("Closed DMA-BUF fd {}", self.fd);
                    }
                }
            }
            ```
            
        3. **Fehlerbehandlung in `drop`:** Die `drop`-Methode darf _niemals_ paniken. Fehler bei der Ressourcenfreigabe müssen geloggt werden. Wenn eine Freigabe fehlschlägt, ist dies oft ein Hinweis auf ein tieferliegendes Problem (z.B. doppeltes Schließen), das an anderer Stelle behoben werden muss.
            
        4. **Ownership-Übergabe:** Beim Erstellen des RAII-Wrappers wird die Ownership der Ressource an den Wrapper übergeben. Der Wrapper ist dann allein für die Freigabe verantwortlich.
            
    - **Vorteile und Konsequenzen:**
        
        - **Automatische Freigabe:** Garantiert, dass Ressourcen freigegeben werden, sobald der RAII-Wrapper den Gültigkeitsbereich verlässt (auch bei Exceptions/Panics oder frühen `return`-Anweisungen).
            
        - **Vermeidung von Leaks:** Reduziert drastisch die Wahrscheinlichkeit von Ressourcenlecks, die zu Systeminstabilität oder -abstürzen führen können.
            
        - **Zentralisierte Logik:** Die Freigabelogik ist klar an den Typ der Ressource gebunden und nicht über den Code verstreut.
            
- **Compile-Time Guarantees:**
    
    - **Maximale Ausnutzung des Typsystems:** Das Rust-Typsystem _muss_ als primäres Werkzeug zur Fehlervermeidung eingesetzt werden.
        
    - **Starke Typisierung für Identifikatoren (Newtype Pattern):**
        
        1. **Deklaration:** Für jede Art von ID (z.B. `WlSurfaceId`, `ClientId`, `OutputId`, `SeatId`) _muss_ ein separates `struct` unter Verwendung des Newtype-Patterns erstellt werden.
            
            ```
            // In novade-system/src/common_ids.rs
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct WlSurfaceId(pub u32); // pub(crate) u32, wenn interne Repräsentation nicht öffentlich sein soll
            
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub struct XdgToplevelId(pub u32); // Annahme: Wird auch als u32 repräsentiert
            ```
            
        2. **Verwendung:** Diese typisierten IDs _müssen_ anstelle von primitiven Typen wie `u32` oder `usize` in Funktionssignaturen, Struct-Feldern und `HashMap`-Schlüsseln verwendet werden.
            
        3. **Vorteil:** Verhindert zur Kompilierzeit die versehentliche Verwendung einer ID falschen Typs (z.B. Übergabe einer `WlSurfaceId` an eine Funktion, die eine `XdgToplevelId` erwartet).
            
    - **Zustandsmaschinen mit typisierten Enums oder PhantomData:**
        
        1. **Analyse komplexer Zustände:** Identifizieren Sie Entitäten mit komplexen Lebenszyklen oder Zustandsübergängen (z.B. `xdg_surface`-Konfiguration, DMA-BUF-Importprozess, Client-Verbindungsstatus).
            
        2. **Typisierte Enums für Zustände:** Wenn die Zustände diskret und bekannt sind, definieren Sie ein `enum`, das jeden Zustand repräsentiert. Speichern Sie Daten, die nur in bestimmten Zuständen relevant sind, direkt in den Enum-Varianten.
            
            ```
            // Beispiel für einen XDG-Surface-Zustand
            // In novade-system/src/shells/xdg/surface_state.rs
            pub enum XdgSurfaceConfigState {
                Unconfigured,
                PendingAckConfigure { last_serial: u32, pending_size: Option<(i32, i32)> },
                Configured { current_size: (i32, i32) },
            }
            ```
            
        3. **Methodenimplementierung basierend auf Zustand:** Implementieren Sie Methoden, die nur auf bestimmten Zuständen operieren, indem Sie `match` auf den Zustand anwenden.
            
            ```
            impl XdgSurfaceData { // Angenommen, XdgSurfaceData enthält ein Feld state: XdgSurfaceConfigState
                pub fn ack_configure(&mut self, serial: u32) -> Result<(), XdgShellError> {
                    match self.config_state {
                        XdgSurfaceConfigState::PendingAckConfigure { last_serial, pending_size } if serial == last_serial => {
                            self.config_state = XdgSurfaceConfigState::Configured {
                                current_size: pending_size.unwrap_or_default(), // Beispiel
                            };
                            Ok(())
                        }
                        _ => Err(XdgShellError::InvalidAckConfigure),
                    }
                }
            }
            ```
            
        4. **PhantomData für lineare Typzustände (fortgeschritten):** Für streng lineare Zustandsübergänge kann `PhantomData<StateType>` verwendet werden, um Typen wie `Surface<Unconfigured>` und `Surface<Configured>` zu erstellen, wobei Zustandsübergänge durch Funktionen erfolgen, die einen Typ konsumieren und einen anderen zurückgeben.
            
    - **Verwendung von Enums für Protokollnachrichten (Requests/Events):**
        
        1. **Generierung oder manuelle Definition:** Smithays `wayland-scanner`-Tool generiert typischerweise Rust-Enums für Wayland-Requests und -Events basierend auf den XML-Protokolldateien.
            
            ```
            // Beispiel (vereinfacht, wie von wayland-scanner generiert)
            // wl_compositor::Request
            // pub enum Request {
            // CreateSurface { id: NewId<WlSurface> },
            // CreateRegion { id: NewId<WlRegion> },
            // }
            ```
            
        2. **Pattern Matching in Handlern:** In den Request-Handlern des Compositors _muss_ Pattern Matching auf diese Enums verwendet werden, um typsicher auf die Argumente jeder Nachricht zuzugreifen. Dies verhindert Fehler durch falsche Interpretation von Argumenttypen oder -anzahlen.
            
- **Fearless Concurrency:**
    
    - **Identifikation von Nebenläufigkeitsgrenzen:**
        
        1. **Haupt-Event-Loop-Thread:** Der primäre Thread, der die `calloop`-Event-Loop ausführt und die meisten Wayland-Protokollnachrichten, Input-Events und Rendering-Callbacks verarbeitet.
            
        2. **Potenzielle separate Threads:**
            
            - **Langwierige I/O-Operationen:** D-Bus-Aufrufe, die blockieren könnten, oder komplexe Dateisystemoperationen.
                
            - **CPU-intensive Berechnungen:** Bestimmte Teile der Rendering-Vorbereitung (obwohl vieles auf der GPU stattfindet) oder komplexe Layout-Berechnungen, falls sie den Hauptthread blockieren würden.
                
            - **Dedizierte Hardware-Überwachungsthreads:** Z.B. ein Thread, der `libinput` in einem blockierenden Modus abfragt.
                
    - **Synchronisationsprimitive – Auswahl und Anwendung:**
        
        1. **`Arc<T>` für geteilten Besitz:** Um Datenstrukturen (wie den `DesktopState`) sicher zwischen mehreren Threads zu teilen, _muss_ `std::sync::Arc` (Atomic Reference Counting) verwendet werden.
            
        2. **`Mutex<T>` für exklusiven Zugriff:** Um den Zugriff auf veränderliche Daten innerhalb eines `Arc` zu synchronisieren, _muss_ `std::sync::Mutex` verwendet werden. Dies stellt sicher, dass immer nur ein Thread gleichzeitig auf die Daten zugreifen kann.
            
            ```
            // Definition des globalen Compositor-Zustands
            // In novade-system/src/state.rs
            use std::sync::{Arc, Mutex};
            pub struct DesktopStateInner {
                // ... viele Felder, z.B. clients: HashMap<ClientId, ClientData>, outputs: Vec<OutputState>, ...
            }
            pub type DesktopState = Arc<Mutex<DesktopStateInner>>;
            ```
            
        3. **`RwLock<T>` für Lese-Schreib-Sperren:** Wenn es viele Leser und wenige Schreiber für geteilte Daten gibt, kann `std::sync::RwLock` eine bessere Performance als `Mutex` bieten, da es mehreren Lesern gleichzeitigen Zugriff erlaubt. Dies _muss_ sorgfältig evaluiert werden, da `RwLock` komplexer in der Handhabung sein kann (Potenzial für Writer Starvation).
            
        4. **`std::sync::mpsc::channel` für Nachrichtenübergabe:** Für die Kommunikation zwischen Threads (z.B. Senden von verarbeiteten Input-Events vom `libinput`-Thread zum Haupt-Compositor-Thread) _müssen_ Message Passing Channels verwendet werden. Dies entkoppelt die Threads und vermeidet komplexe Sperrmechanismen.
            
            ```
            // In novade-system/src/input_manager/backend/libinput_handler.rs
            // fn run_libinput_thread(event_sender: mpsc::Sender<InputEvent>, mut libinput_context: Libinput) {
            // loop {
            //         libinput_context.dispatch().unwrap(); // Blockiert bis Events da sind
            // for event in &mut libinput_context {
            //             let nova_event = convert_libinput_event_to_nova_event(event);
            // if let Some(ne) = nova_event {
            //                 if event_sender.send(ne).is_err() {
            //                     // Hauptthread hat den Empfänger geschlossen, Thread beenden
            //                     return;
            //                 }
            //             }
            //         }
            //     }
            // }
            ```
            
        5. **`parking_lot::Mutex` / `parking_lot::RwLock` (optional):** Für Anwendungsfälle, in denen die Performance von Standardbibliotheks-Mutexen als Flaschenhals identifiziert wird, kann die Verwendung der `parking_lot`-Alternativen in Betracht gezogen werden, da diese oft effizienter sind und zusätzliche Features bieten. Dies erfordert eine explizite Begründung.
            
    - **Integration von Asynchronität (`async/await`) mit `calloop`:**
        
        1. **`tokio` oder anderer Async-Runtime:** Für die Ausführung von `async`-Code (z.B. für D-Bus mit `zbus_tokio`) _muss_ eine Async-Runtime wie `tokio` verwendet werden.
            
        2. **Ausführung von `async`-Blöcken:** Lange laufende `async`-Operationen sollten nicht direkt in `calloop`-Handlern blockieren. Stattdessen:
            
            - Starten Sie den `async`-Block mit `tokio::spawn` auf einem `tokio`-Worker-Thread.
                
            - Verwenden Sie einen `mpsc::channel` oder einen `calloop::channel::Channel` um das Ergebnis des `async`-Blocks zurück an die `calloop`-Event-Loop zu senden. Der `calloop`-Event-Loop wird eine `EventSource` für diesen Channel registrieren.
                
    - **Vermeidung von Deadlocks:**
        
        1. **Sperrreihenfolge (Lock Ordering):** Definieren Sie eine globale, konsistente Reihenfolge für das Ergreifen mehrerer Locks. Wenn Thread A Lock L1 und dann L2 benötigt, und Thread B Lock L2 und dann L1, kann ein Deadlock entstehen. Alle Threads _müssen_ Locks in der gleichen vordefinierten Reihenfolge anfordern.
            
        2. **Minimierung der Lock-Haltezeit:** Locks _sollten_ nur für die kürzestmögliche Zeit gehalten werden. Führen Sie langwierige Operationen außerhalb des gelockten Bereichs durch, wenn möglich.
            
        3. **Verwendung von `Mutex::try_lock`:** In Situationen, in denen ein Blockieren unerwünscht ist, kann `try_lock` verwendet werden, um nicht-blockierend ein Lock anzufordern.
            
- **Zero-Cost Abstractions:**
    
    - **Prinzipielle Anwendung:** Rust-Features wie Generics, Traits, Closures und `async/await` _müssen_ so eingesetzt werden, dass sie zur Kompilierzeit aufgelöst werden und keinen (oder vernachlässigbaren) Laufzeit-Overhead im Vergleich zu einer manuellen, weniger abstrakten Implementierung erzeugen.
        
    - **Generics und Monomorphisierung:**
        
        1. **Verwendung für Flexibilität:** Definieren Sie generische Structs und Funktionen, wo unterschiedliche Typen mit der gleichen Logik behandelt werden können (z.B. ein `Renderer`-Trait mit verschiedenen Implementierungen wie `VulkanRenderer`, `OpenGlEsRenderer`).
            
            ```
            // In novade-system/src/renderer/traits.rs
            pub trait GpuBuffer {
                fn size(&self) -> usize;
                fn map_memory(&self) -> Result<*mut u8, GpuError>;
                fn unmap_memory(&self);
            }
            
            pub struct GenericRenderer<B: GpuBuffer> {
                // ...
                _phantom_buffer: std::marker::PhantomData<B>,
            }
            ```
            
        2. **Kompilierzeit-Auflösung:** Der Rust-Compiler generiert für jede konkrete Verwendung eines generischen Typs spezialisierten Code (Monomorphisierung), wodurch der Overhead von dynamischem Dispatch vermieden wird.
            
    - **Traits und statischer Dispatch:**
        
        1. **Definition von Schnittstellen:** Verwenden Sie Traits, um gemeinsame Schnittstellen für verschiedene Implementierungen zu definieren (z.B. `smithay::backend::renderer::Renderer`, `smithay::input::KeyboardHandler`).
            
        2. **Bevorzugung von statischem Dispatch:** Wo immer möglich, _sollte_ statischer Dispatch (Verwendung von Trait Bounds bei Generics: `fn foo<T: MyTrait>(item: &T)`) gegenüber dynamischem Dispatch (`fn bar(item: &Box<dyn MyTrait>)`) bevorzugt werden, da dies Inlining und weitere Optimierungen ermöglicht.
            
    - **Closures:** Closures in Rust sind typischerweise Zero-Cost, da der Compiler sie oft inlinet oder in äquivalente Structs mit `impl Fn*` Traits umwandelt. Sie _sollten_ für kurze, prägnante Operationen (z.B. als Argumente für Iterator-Methoden) verwendet werden.
        
    - **`async/await` Transformation:** Die `async/await`-Syntax wird vom Compiler in Zustandsmaschinen transformiert, die auf Futures basieren. Diese Transformation ist darauf ausgelegt, effizient zu sein.
        
- **Idiomatic Rust:**
    
    - **Strikte Einhaltung von Konventionen:** Der Code _muss_ den etablierten Rust-Idiomen und Best Practices folgen, wie sie in "The Rust Programming Language", den Rust API Guidelines und durch `rust-clippy` Lints definiert sind.
        
    - **`Result` und `Option` für Fehler und optionale Werte:**
        
        1. **`Result<T, E>`:** _Muss_ für alle Operationen verwendet werden, die fehlschlagen können. Der `?`-Operator _muss_ für die prägnante Fehlerpropagation verwendet werden.
            
        2. **`Option<T>`:** _Muss_ für Werte verwendet werden, die optional sind. Methoden wie `map`, `and_then`, `ok_or`, `unwrap_or_else` _sollten_ genutzt werden, um explizite `match`-Blöcke zu vermeiden, wo es die Lesbarkeit verbessert.
            
    - **Iteratoren und funktionale Kombinatoren:**
        
        1. **Bevorzugung von Iteratoren:** Anstelle von manuellen `for`-Schleifen mit Indexvariablen _sollten_ Iteratoren und deren Methoden (`iter()`, `iter_mut()`, `into_iter()`) verwendet werden.
            
        2. **Kombinatoren:** Methoden wie `map()`, `filter()`, `fold()`, `collect()`, `for_each()` _sollten_ intensiv genutzt werden, um Datenverarbeitungslogik deklarativ und prägnant auszudrücken.
            
            ```
            // Beispiel: Filtern und Sammeln von aktiven Clients
            // Statt:
            // let mut active_client_names = Vec::new();
            // for client_data in self.clients.values() {
            //     if client_data.is_active() {
            //         active_client_names.push(client_data.name().to_string());
            //     }
            // }
            // Besser (idiomatisch):
            let active_client_names: Vec<String> = self.clients.values()
                .filter(|cd| cd.is_active())
                .map(|cd| cd.name().to_string())
                .collect();
            ```
            
    - **Pattern Matching (`match`, `if let`, `while let`):**
        
        1. **`match` für Enums:** _Muss_ für die Behandlung aller Varianten eines Enums verwendet werden, um Vollständigkeit sicherzustellen (Compiler-Warnung bei fehlenden Armen).
            
        2. **`if let` / `while let` für einzelne Varianten:** _Sollte_ verwendet werden, wenn nur eine oder wenige Varianten eines Enums oder der `Some`-Variante eines `Option` von Interesse sind, um die Verschachtelung von `match`-Ausdrücken zu reduzieren.
            
    - **Modulsystem und Sichtbarkeit:** Das Rust-Modulsystem _muss_ verwendet werden, um den Code logisch zu strukturieren. Die Sichtbarkeitsmodifikatoren (`pub`, `pub(crate)`, `pub(in path)`) _müssen_ sorgfältig eingesetzt werden, um klare öffentliche APIs zu definieren und interne Implementierungsdetails zu kapseln.
        
    - **Code-Formatierung mit `rustfmt`:** Alle Code-Beiträge _müssen_ vor dem Commit mit `rustfmt` (unter Verwendung der im Projekt definierten Konfiguration, z.B. `rustfmt.toml`) formatiert werden. Dies wird durch Pre-Commit-Hooks und CI-Checks erzwungen.
        
    - **`clippy` Lints:** Alle Code-Beiträge _müssen_ frei von `clippy::pedantic` und `clippy::nursery` Warnungen sein (oder diese müssen explizit per `#[allow(...)]` mit Begründung deaktiviert werden). `clippy::all` und `clippy::cargo` _müssen_ ebenfalls fehlerfrei sein. Dies wird durch CI-Checks erzwungen.
        
- **Error Handling:**
    
    - **Umfassende Verwendung von `Result<T, E>`:** Jede Funktion, deren Ausführung fehlschlagen kann (I/O-Fehler, Protokollverletzungen, ungültige Zustände, Ressourcenerschöpfung), _muss_ einen `Result<T, E>`-Typ zurückgeben.
        
    - **Definition spezifischer Fehlertypen (`thiserror`):**
        
        1. **Pro Modul/Komponente:** Für jede logische Komponente (z.B. `DmabufHandler`, `XdgShellManager`, `DrmBackend`) _muss_ ein dediziertes Fehler-Enum definiert werden.
            
        2. **`thiserror::Error` Ableitung:** Diese Enums _müssen_ den `thiserror::Error` Trait ableiten, um die Implementierung von `std::error::Error` und `std::fmt::Display` zu vereinfachen.
            
        3. **Varianten für Fehlerursachen:** Jede Variante des Fehler-Enums _muss_ eine spezifische Fehlerursache repräsentieren.
            
        4. **`#[source]` für Fehlerverkettung:** Wenn ein Fehler einen anderen Fehler kapselt (z.B. ein `DmabufImportError` aufgrund eines `EglError`), _muss_ das `#[source]`-Attribut (oder `#[from]` für direkte Konvertierung) verwendet werden, um die Fehlerkette für Diagnosezwecke zu erhalten.
            
        5. **Aussagekräftige Fehlermeldungen:** Die `#[error("...")]`-Attribute _müssen_ klare, für Menschen verständliche Fehlermeldungen erzeugen, die kontextuelle Informationen enthalten.
            
            ```
            // In novade-system/src/buffers/dmabuf/error.rs
            use thiserror::Error;
            use smithay::protocols::wp::linux_dmabuf::zv1::server::zwp_linux_buffer_params_v1;
            
            #[derive(Debug, Error)]
            pub enum DmabufError {
                #[error("DRM format modifier {modifier:?} is not supported for format {drm_format:?}")]
                UnsupportedModifier { drm_format: u32, modifier: u64 },
            
                #[error("Failed to import DMA-BUF via GBM: {0}")]
                GbmImportFailed(#[from] std::io::Error), // Wenn GBM-Fehler als io::Error kommen
            
                #[error("Client provided an invalid number of FDs for DMA-BUF import: expected {expected}, got {got}")]
                InvalidFdCount { expected: usize, got: usize },
            
                #[error("Wayland protocol error: {0:?}")] // 0:? für Debug-Ausgabe des Protokollfehlers
                Protocol(zwp_linux_buffer_params_v1::Error),
            
                #[error("Internal renderer error during DMA-BUF texture creation: {0}")]
                RendererError(String), // Ersetze String durch spezifischen Renderer-Fehlertyp
            }
            ```
            
    - **Kein `unwrap()` oder `expect()` auf `Result`/`Option` im Produktionscode:** Die Methoden `unwrap()` und `expect()` auf `Result`- und `Option`-Typen _dürfen nicht_ im Produktionscode verwendet werden, da sie bei einem `Err`- oder `None`-Wert zu einem Panic führen. Ausnahmen sind Tests oder Situationen, in denen eine Invariante absolut sicherstellt, dass kein `Err`/`None` auftreten kann (dies muss dann explizit dokumentiert werden, aber Pattern Matching oder `expect` mit sehr detaillierter Begründung sind vorzuziehen).
        
    - **Protokollfehler vs. Interne Fehler:**
        
        1. **Protokollfehler:** Wenn ein Client eine ungültige Anfrage sendet oder gegen das Wayland-Protokoll verstößt, _muss_ der Compositor den Client mit einem spezifischen Protokollfehler (unter Verwendung von `display_handle.send_error` oder den von Smithay bereitgestellten Mechanismen) informieren und dessen Verbindung trennen.
            
        2. **Interne Fehler:** Wenn im Compositor ein unerwarteter interner Fehler auftritt (z.B. Ressourcenerschöpfung, Fehler im Backend), _muss_ dieser Fehler detailliert geloggt werden. Der Compositor _sollte_ versuchen, sich davon zu erholen, wenn möglich, oder andernfalls kontrolliert herunterfahren, um Datenverlust zu minimieren. Ein solcher Fehler darf nicht zu einem Protokollfehler für einen nicht schuldigen Client führen.
            
- **Testing:**
    
    - **Testabdeckung als Ziel:** Eine hohe Testabdeckung (Unit- und Integrationstests) _ist anzustreben_. Die genaue Prozentzahl hängt von der Kritikalität des Moduls ab, aber wichtige Logikpfade und Fehlerfälle _müssen_ abgedeckt sein.
        
    - **Unit-Tests (`#[test]`):**
        
        1. **Fokus:** Testen einzelner Funktionen, Methoden oder kleiner logischer Einheiten in Isolation.
            
        2. **Mocking von Abhängigkeiten:** Externe Abhängigkeiten (wie der `DesktopState`, `DisplayHandle`, `ClientData` oder Hardware-Backends) _müssen_ für Unit-Tests gemockt werden. Das `mockall`-Crate oder manuell implementierte Mock-Objekte (Structs, die die benötigten Traits implementieren und konfigurierbares Verhalten zeigen) _können_ verwendet werden.
            
            ```
            // In novade-system/src/shells/xdg/tests.rs
            // #[cfg(test)]
            // mod tests {
            // use super::*;
            // use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
            // // ... weitere Mocks
            //
            //     struct MockCompositorState { /* ... relevante Felder für den Test ... */ }
            //     impl CompositorHandler for MockCompositorState { /* ... Mock-Implementierungen ... */ }
            //     // ... weitere Trait-Implementierungen für den MockState
            //
            // #[test]
            //     fn test_xdg_surface_creation_and_role_assignment() {
            //         let mut state = MockCompositorState::new();
            //         let display_handle = MockDisplayHandle::new(); // Smithay bietet keine einfache MockDisplayHandle
            //                                                       // Hier muss man ggf. anders vorgehen oder
            //                                                       // nur die Logik testen, die keine DisplayHandle braucht.
            //
            //         // ... Testlogik, die Handler-Funktionen direkt aufruft
            //     }
            // }
            ```
            
        3. **Spezifische Assertions:** Verwendung von `assert_eq!`, `assert_ne!`, `assert!(...)` und `matches!(...)` um das Verhalten und die Ergebnisse zu überprüfen.
            
    - **Integrationstests (in `tests/` Verzeichnis):**
        
        1. **Fokus:** Testen der Interaktion zwischen mehreren Modulen oder Komponenten des Compositors.
            
        2. **Headless Backend:** Für Tests, die keinen sichtbaren Output benötigen, _sollte_ `smithay::backend::headless` verwendet werden, um einen minimalen Compositor-Stack zu instanziieren.
            
        3. **Test-Wayland-Client:** Ein Wayland-Client (implementiert mit `wayland-client` oder einem spezifischen Test-Framework) _muss_ verwendet werden, um Wayland-Protokollnachrichten an den im Test laufenden Compositor zu senden und dessen Antworten (Events) zu überprüfen.
            
        4. **Szenario-Tests:** Abbildung typischer Benutzerinteraktionen oder Protokollsequenzen:
            
            - Client verbindet sich, bindet Globals, erstellt ein `xdg_toplevel`.
                
            - Client hängt einen SHM-Buffer an, committet, erhält Frame-Callback.
                
            - Fokuswechsel zwischen zwei Clients.
                
            - Drag-and-Drop zwischen zwei Test-Client-Oberflächen.
                
    - **Property-Based Testing (optional):** Für komplexe Zustandsmaschinen oder Datenstrukturen _kann_ die Verwendung von Property-Based Testing (z.B. mit dem `proptest`-Crate) in Betracht gezogen werden, um eine breitere Palette von Eingaben automatisch zu testen.
        
    - **CI-Integration:** Alle Tests _müssen_ bei jedem Commit und Pull Request im Continuous Integration (CI) System ausgeführt werden. Fehlschlagende Tests _müssen_ das Mergen von Code blockieren.
        
- **Dokumentation (`rustdoc`):**
    
    - **Umfassende Abdeckung:** _Alle_ öffentlichen Elemente (Module, Structs, Enums, Funktionen, Traits, Methoden, öffentliche Konstanten) _müssen_ mit `rustdoc`-Kommentaren versehen werden.
        
    - **Modul-Level-Dokumentation (`//! ...`):** Jedes Modul (`mod.rs` oder `lib.rs`) _muss_ mit einem Modul-Level-Kommentar beginnen, der:
        
        1. Den Zweck und die Hauptverantwortlichkeit des Moduls beschreibt.
            
        2. Die wichtigsten öffentlichen Typen und Funktionen des Moduls auflistet oder zusammenfasst.
            
        3. Beziehungen und Interaktionen mit anderen wichtigen Modulen erläutert.
            
    - **Struct- und Enum-Dokumentation:**
        
        1. Beschreibung des Zwecks des Typs.
            
        2. Erläuterung der Bedeutung jedes öffentlichen Feldes (für Structs) oder jeder Variante (für Enums).
            
    - **Funktions- und Methoden-Dokumentation:**
        
        1. Klare Beschreibung dessen, was die Funktion/Methode tut.
            
        2. Erläuterung jedes Parameters (Bedeutung, erwartete Werte, Einschränkungen).
            
        3. Beschreibung des Rückgabewertes, einschließlich der Bedeutung von `Ok`- und `Err`-Varianten bei `Result`.
            
        4. Auflistung möglicher Panics (Abschnitt `# Panics`), falls die Funktion unter bestimmten Umständen paniken kann (sollte vermieden werden, siehe Fehlerbehandlung).
            
        5. Auflistung von Sicherheitsinvarianten (Abschnitt `# Safety`) für `unsafe fn`.
            
    - **Code-Beispiele in `rustdoc`:**
        
        1. Für wichtige öffentliche Funktionen oder typische Anwendungsfälle _sollten_ lauffähige Code-Beispiele direkt in den `rustdoc`-Kommentaren (eingeschlossen in `rust ...` Blöcke) bereitgestellt werden.
            
        2. Diese Beispiele _werden_ automatisch mit `cargo test --doc` getestet.
            
    - **Verlinkung:**
        
        1. Verwenden Sie Markdown-Links, um auf andere Typen, Module oder externe Dokumentation (z.B. Wayland-Protokollspezifikationen auf `wayland.freedesktop.org`) zu verweisen.
            
        2. Innerhalb von `rustdoc` können Typen direkt verlinkt werden: `[MyStruct]`, `[crate::module::OtherType]`.
            
    - **Konsistenz und Klarheit:** Die Dokumentation _muss_ klar, präzise, konsistent und frei von Mehrdeutigkeiten sein. Sie _sollte_ in englischer Sprache verfasst sein, es sei denn, das Projektteam entscheidet sich explizit für Deutsch.
        
    - **Aktualität:** Die Dokumentation _muss_ bei jeder Code-Änderung, die die öffentliche API oder das Verhalten betrifft, entsprechend aktualisiert werden. Veraltete Dokumentation ist schlimmer als keine Dokumentation.
        

Die strikte Einhaltung dieser extrem detaillierten Qualitätsstandards und Implementierungsrichtlinien ist der Grundpfeiler für die Entwicklung eines robusten, sicheren, performanten und wartbaren NovaDE Wayland-Compositors. Jeder einzelne Schritt, von der Deklaration einer Variable bis zur Implementierung eines komplexen Protokoll-Handlers, muss unter Berücksichtigung dieser Prinzipien erfolgen.

---

## 5. Fehlerbehandlung und Edge-Cases

Eine robuste Fehlerbehandlung und die präzise Definition von Edge-Cases sind von entscheidender Bedeutung für die Stabilität und Zuverlässigkeit eines Wayland-Compositors. Rust bietet hierfür leistungsstarke Mechanismen, die es ermöglichen, Fehler zur Compile-Zeit zu erkennen und Laufzeitfehler explizit zu handhaben.

### 5.1. Definition von Fehlerbehandlungsstrategien

Die Fehlerbehandlung in Rust unterscheidet grundlegend zwischen wiederherstellbaren Fehlern, die mit dem `Result<T, E>`-Enum repräsentiert werden, und nicht wiederherstellbaren Fehlern, die einen Programmabbruch (`panic!`) auslösen.1 Für einen Wayland-Compositor ist es entscheidend, diese Unterscheidung bewusst zu nutzen, um auf unterschiedliche Fehlerzustände angemessen reagieren zu können.

#### 5.1.1. Rust-spezifische Fehlerbehandlung

Für wiederherstellbare Fehler, die beispielsweise durch ungültige Client-Eingaben oder temporäre Systemzustände entstehen, sollte `Result` verwendet werden. Dies zwingt den Entwickler, alle potenziellen Fehlerpfade explizit zu behandeln, was die Robustheit des Codes erheblich steigert. Ein gängiges Muster in Rust ist die Definition spezifischer Fehler-Enums für jede Modul- oder Funktionskategorie, oft unterstützt durch die `thiserror`-Kiste, die das Schreiben von Boilerplate-Code reduziert und eine klare Fehlerhierarchie ermöglicht.2 Diese Enums können dann über den `#[from]`-Attribut in übergeordnete, allgemeinere Fehler-Enums konvertiert werden, was die Fehler-Propagation über verschiedene Schichten hinweg vereinfacht.2 Der `?`-Operator ist dabei ein idiomatisch genutztes Sprachmerkmal, das die Fehler-Propagation in Funktionen, die `Result` zurückgeben, erheblich vereinfacht.

Nicht wiederherstellbare Fehler, die auf Programmierfehler oder inkonsistente Zustände hindeuten, sollten einen `panic!` auslösen. Dies führt zu einem sofortigen Programmabbruch, was in Systemprogrammen wie Compositors oft die sicherste Option ist, um Datenkorruption oder unvorhersehbares Verhalten zu verhindern. Rusts `panic!`-Mechanismus ist dabei so konzipiert, dass er im besten Fall einen Stack-Unwind durchführt und Destruktoren aufruft, um Ressourcen freizugeben.6

#### 5.1.2. Strategien für die Fehlerprotokollierung und -meldung

Eine effektive Fehlerprotokollierung ist unerlässlich für die Diagnose und Behebung von Problemen im Wayland-Compositor. Die Integration mit einem strukturierten Logging-Framework wie `tracing` ist hierfür die bevorzugte Methode.8 `tracing` ermöglicht es, Kontextinformationen (z.B. Client-ID, Surface-ID, Zeitstempel) an Log-Einträge anzuhängen, was die Nachverfolgung von Ereignissen und Fehlern erheblich erleichtert.

Die Definition und konsistente Anwendung von Log-Levels (Error, Warn, Info, Debug, Trace) ist entscheidend, um die Granularität der ausgegebenen Informationen zu steuern.12 In Produktionsumgebungen kann der Log-Level zur Compile-Zeit begrenzt werden, um die Performance zu optimieren und sensible Daten nicht zu protokollieren.8 Eine zentrale Fehlerberichterstattung, die Log-Daten aggregiert und möglicherweise an ein externes Monitoring-System sendet, ist für den Betrieb eines stabilen Compositors unerlässlich. Dies ermöglicht proaktive Benachrichtigungen bei kritischen Fehlern und eine schnellere Reaktion auf Systeminstabilitäten.

### 5.2. Definition von Edge-Cases und deren Behandlung

Die Behandlung von Edge-Cases ist ein kritischer Aspekt der Compositor-Entwicklung, da sie oft unvorhergesehene Interaktionen oder Systemzustände betreffen.

#### 5.2.1. Unerwartetes Client-Verhalten

Wayland-Compositors müssen robust gegenüber fehlerhaftem oder bösartigem Client-Verhalten sein. Dies umfasst:

- **Clients, die ungültige Argumente senden:** Wayland-Protokolle definieren spezifische Fehlercodes für ungültige Anfragen (z.B. negative Größen für Oberflächen, ungültige IDs).13 Der Compositor sollte in solchen Fällen einen Protokollfehler an den Client senden (`wl_display::post_error`) und die Verbindung zum Client gegebenenfalls beenden, anstatt selbst zu crashen.15
- **Clients, die nicht auf `ping`/`pong`-Anfragen antworten:** Wayland-Compositors senden `ping`-Anfragen an Clients, um deren Lebendigkeit zu überprüfen. Clients, die nicht innerhalb einer bestimmten Frist mit einem `pong` antworten, können als tot betrachtet und ihre Verbindungen geschlossen werden.14 Dies verhindert, dass nicht reagierende Clients Systemressourcen blockieren.
- **Clients, die Ressourcen nicht korrekt freigeben oder zu viele Ressourcen anfordern:** Der Compositor muss Mechanismen implementieren, um zu verhindern, dass Clients durch übermäßigen Ressourcenverbrauch (z.B. zu viele Oberflächen, Puffer) das System destabilisieren. Rusts RAII-Prinzip hilft hierbei, da Ressourcen automatisch freigegeben werden, wenn Objekte den Gültigkeitsbereich verlassen 6, aber explizite Aufräumlogik für Wayland-Objekte ist dennoch erforderlich.

#### 5.2.2. Hardware-Ausfälle

Die dynamische Natur von Hardware erfordert spezielle Behandlungsstrategien:

- **Hotplug von Monitoren (An- und Abstecken):** Wayland-Compositors müssen auf das Hinzufügen oder Entfernen von Ausgabegeräten reagieren können.16 Dies beinhaltet die Anpassung des Ausgabelayouts, die Benachrichtigung von Clients über neue oder entfernte Ausgaben (`wl_output::enter`, `wl_output::leave`) und die Neuanordnung von Fenstern. Smithay bietet hierfür Module wie `smithay::output` und `smithay::backend::drm`.16
- **Ausfall von Eingabegeräten:** Ähnlich wie bei Monitoren muss der Compositor auf das Hinzufügen oder Entfernen von Eingabegeräten (Tastaturen, Mäuse, Touchscreens) reagieren und die `SeatState` entsprechend aktualisieren.21
- **DRM-Fehler oder GPU-Probleme:** Fehler im Direct Rendering Manager (DRM) oder bei der GPU-Kommunikation können kritisch sein. Der Compositor muss in der Lage sein, solche Fehler zu erkennen und gegebenenfalls auf Software-Rendering zurückzugreifen oder den Benutzer zu informieren.23

#### 5.2.3. System-Events

Der Compositor muss auch auf allgemeine Systemereignisse reagieren:

- **Sitzungswechsel (TTY-Wechsel):** Wenn der Compositor direkt auf einem TTY läuft (z.B. mit dem `tty-udev`-Backend von Anvil 25), muss er in der Lage sein, Ressourcen freizugeben und den Zustand zu speichern, wenn der Benutzer zu einem anderen TTY wechselt.21
- **Out-of-Memory-Situationen:** Obwohl Rust Speichersicherheit fördert, können Out-of-Memory-Situationen auftreten. Eine Strategie könnte sein, weniger kritische Clients oder Ressourcen zu beenden, um das System stabil zu halten.
- **Absturz von XWayland:** Wenn XWayland als Kompatibilitätsschicht verwendet wird, muss der Compositor dessen Absturz erkennen und entsprechend reagieren, z.B. XWayland neu starten oder X11-Anwendungen beenden.30

#### Erkenntnis: Die Notwendigkeit einer umfassenden Fehlerklassifizierung und -reaktion

Die Entwicklung eines Wayland-Compositors erfordert eine umfassende Strategie zur Fehlerklassifizierung und -reaktion. Ein Wayland-Compositor ist die einzige Autorität für Rendering, Hardware und Fensterverwaltung.31 Daher muss er nicht nur Fehler erkennen, sondern auch eine angemessene Reaktion darauf haben, die von der Art des Fehlers abhängt. Ein ungültiger Client-Request sollte nicht den gesamten Compositor zum Absturz bringen, während ein kritischer interner Fehler (z.B. Speicher-Korruption durch `unsafe` Code) einen Panic rechtfertigen könnte.1

Eine detaillierte Fehlerklassifizierung (z.B. Wayland-Protokollfehler, Systemfehler, interne Logikfehler) und eine definierte Fehlerbehandlungsstrategie für jeden Typ sind entscheidend für die Robustheit und Benutzerfreundlichkeit des Compositors. Für Wayland-Protokollfehler ist das Senden eines `wl_display::post_error` an den Client und gegebenenfalls das Beenden der Client-Verbindung die korrekte Reaktion.15 Für wiederherstellbare interne Fehler sollte `Result` zurückgegeben und der Fehler propagiert werden. Kritische, nicht wiederherstellbare Fehler sollten einen `panic!` auslösen, begleitet von detailliertem Logging und Crash-Reporting. Edge-Cases wie Hotplug-Ereignisse erfordern die Implementierung spezifischer Handler, die den Systemzustand dynamisch anpassen. Eine unzureichende Fehlerbehandlung kann zu Systeminstabilität, Datenverlust oder einer schlechten Benutzererfahrung führen. Eine proaktive Behandlung von Edge-Cases erhöht die Zuverlässigkeit erheblich, indem der Compositor auf unerwartete, aber mögliche Szenarien vorbereitet ist und sich entsprechend anpassen kann. Der Bericht muss daher eine klare Fehlerbehandlungs-Hierarchie und -Strategie definieren, die sowohl Rusts Sprachfeatures als auch die spezifischen Anforderungen eines Wayland-Compositors berücksichtigt.

---

## 6. Implementierungsdetails und Qualitätssicherung

Dieser Abschnitt beschreibt detaillierte Implementierungsansätze und Strategien zur Qualitätssicherung, die für den Aufbau eines Wayland-Compositors in Rust unter Verwendung des Smithay-Frameworks relevant sind.

### 6.1. Detaillierte Implementierungsvorlagen

Smithay ist ein Framework, das grundlegende Bausteine für Wayland-Compositors in Rust bereitstellt, indem es sich auf Low-Level-Helfer und Abstraktionen für System- und Wayland-Protokollinteraktionen konzentriert.8 Es ist modular aufgebaut, um Flexibilität zu gewährleisten und Entwickler nicht zu zwingen, Teile zu verwenden, die sie nicht benötigen.8

#### 6.1.1. Illustrative Codebeispiele für kritische Funktionen und Interaktionen

Die Kernfunktionalität eines Wayland-Compositors in Smithay basiert auf der Verwaltung des Zustands und der Interaktion mit Wayland-Clients über eine Event-Loop.

Compositor-Initialisierung:

Die Initialisierung des Compositors beginnt mit der Erstellung eines DisplayHandle und der verschiedenen State-Objekte, die die Wayland-Protokolle verwalten. Jedes Wayland-Modul in Smithay (z.B. Compositor, Shell, Seat) bietet ein spezifisches *State-Struktur, das bei der Initialisierung Globals in das DisplayHandle einfügt.8

Ein Beispiel für die Initialisierung des Compositor-Zustands würde die Erstellung einer Haupt-`State`-Struktur umfassen, die alle modularen `*State`-Objekte enthält. Der `CompositorState` ist dabei zentral, da er die grundlegende Logik zur Handhabung von Oberflächen (`wl_surface`) bereitstellt, die Clients zum Erstellen ihrer Fenster verwenden.36

Rust

```
// Beispielstruktur für den globalen Compositor-Zustand
struct MyCompositorState {
    compositor_state: CompositorState<Self>,
    // Weitere Zustände für andere Wayland-Protokolle, z.B. XdgShellState, SeatState
    xdg_shell_state: XdgShellState<Self>,
    seat_state: SeatState<Self>,
    //...
}

impl MyCompositorState {
    fn new(display_handle: &DisplayHandle) -> Self {
        // Erstellung des CompositorState
        let compositor_state = CompositorState::new::<Self>(display_handle);
        // Erstellung des XdgShellState
        let xdg_shell_state = XdgShellState::new::<Self>(display_handle);
        // Erstellung des SeatState
        let seat_state = SeatState::<Self>::new();
        let _seat = seat_state.new_seat(display_handle, "seat-0"); // Erstellt ein Wayland-Seat

        MyCompositorState {
            compositor_state,
            xdg_shell_state,
            seat_state,
            //...
        }
    }
}

// Implementierung der notwendigen Handler-Traits und Delegation
impl CompositorHandler for MyCompositorState {
    fn compositor_state(&mut self) -> &mut CompositorState<Self> {
        &mut self.compositor_state
    }
    // Implementierung weiterer erforderlicher Methoden wie client_compositor_state und commit
    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        // Annahme: ClientState ist eine Struktur, die client-spezifischen Zustand enthält
        // und CompositorClientState ist Teil davon.
        &client.get_data::<ClientState>().unwrap().compositor_state
    }
    fn commit(&mut self, surface: &WlSurface) {
        // Logik zur Verarbeitung von Surface-Updates
        // Zugriff auf SurfaceData mit with_states
        compositor::with_states(surface, |states| {
            //...
        });
    }
}
delegate_compositor!(MyCompositorState); // Delegiert die Implementierung des Compositor-Protokolls

// Ähnlich für XdgShellHandler und SeatHandler
impl XdgShellHandler for MyCompositorState {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState<Self> {
        &mut self.xdg_shell_state
    }
    // Implementierung von new_toplevel, new_popup, etc.
    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        //...
    }
}
delegate_xdg_shell!(MyCompositorState);

impl SeatHandler for MyCompositorState {
    type KeyboardFocus = WlSurface; // Oder eine andere Target-Type
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }
    // Implementierung von focus_changed, cursor_image, etc.
    fn focus_changed(&mut self, seat: &Seat<Self>, focused: Option<&Self::KeyboardFocus>) {
        //...
    }
    fn cursor_image(&mut self, seat: &Seat<Self>, image: CursorImageStatus) {
        //...
    }
}
delegate_seat!(MyCompositorState);
```

Die `delegate_*!`-Makros von Smithay sind ein wesentlicher Bestandteil, um die Implementierung der Wayland-Server-Traits zu vereinfachen und Boilerplate-Code zu reduzieren.8

CompositorHandler::commit Implementierung:

Die commit-Methode des CompositorHandler-Traits wird bei jedem Puffer-Commit einer Oberfläche aufgerufen.36 Dies ist der zentrale Punkt, an dem der Compositor auf Änderungen der Client-Oberflächen reagiert. Innerhalb dieser Methode kann auf den aktuellen Zustand der Oberfläche über with_states zugegriffen werden, um SurfaceData abzurufen, die alle Attribute der Oberfläche enthält.36

Die Verarbeitung von Surface-Updates folgt einer definierten Reihenfolge: Zuerst werden Pre-Commit-Hooks aufgerufen (z.B. zur Validierung von Client-Anfragen), dann wird der ausstehende Zustand angewendet oder für synchronisierte Subsurfaces gecached, gefolgt von Post-Commit-Hooks (z.B. für weitere Zustandsverarbeitung). Erst danach wird die `commit`-Implementierung des Compositors aufgerufen, um auf den neuen Zustand zuzugreifen.36

#### 6.1.2. Datenstrukturen und Algorithmen

Das Wayland-Protokoll ist objektorientiert und asynchron. Clients und Compositor kommunizieren über Unix-Domain-Sockets, wobei Nachrichten als 32-Bit-Wörter strukturiert sind.31 Die Schnittstellen, Anfragen und Ereignisse sind in XML-Dateien definiert, aus denen dann der Code generiert wird.31

Zustandsverwaltung:

Smithay verwendet einen callback-basierten Event-Loop (calloop), der gut zur reaktiven Natur eines Wayland-Compositors passt.8 Um die Notwendigkeit von Rc und Arc (Referenzzähler) zu minimieren, erlaubt calloop die Übergabe einer veränderlichen Referenz zum Hauptzustand des Compositors an die Callbacks. Dies ermöglicht den sequentiellen Zugriff auf einen zentralisierten, veränderlichen Zustand ohne zusätzliche Synchronisationsmechanismen.8

Für Objekte, die mehrere Instanzen mit eigenem Zustand haben können (insbesondere auf Client-Seite), bietet Smithay eine Schnittstelle, um beliebige Werte direkt mit dem Objekt zu verknüpfen, anstatt sie in einem separaten Container suchen zu müssen.8 Dies wird oft durch `UserDataMap` realisiert.

Oberflächen- und Fensterverwaltung:

Wayland-Oberflächen (wl_surface) sind die grundlegenden Bausteine für Fenster. Eine Oberfläche erhält erst eine "Rolle" (z.B. Cursor, Subsurface, Toplevel-Fenster) und kann nur eine Rolle während ihrer Lebensdauer haben.36 Das smithay::wayland::compositor-Modul verwaltet die Zustände von Oberflächenbäumen (mit Subsurfaces) und ermöglicht den Zugriff auf die Baumstruktur und Attribute.36

Für die Fensterverwaltung auf Desktop-Ebene bietet Smithay das `smithay::desktop::space`-Modul.44 Dieses Modul repräsentiert eine zweidimensionale Ebene, auf der Fenster und Ausgaben abgebildet werden können. Es bietet Funktionen zum Rendern von Ausgaben und zum Abrufen von Renderelementen für einen bestimmten Output.44 Die `Window`-Struktur innerhalb dieses Moduls repräsentiert ein vollständiges Fenster, einschließlich Subsurfaces und Popups, und kann XWayland-Fenster transparent behandeln.45 Wichtige Eigenschaften eines Fensters sind seine logische Position, seine Geometrie (der tatsächliche Inhalt der Toplevel-Oberfläche) und seine Bounding Box (der Bereich, in dem Inhalte gezeichnet werden).45

### 6.2. Testspezifikationen

Qualitätssicherung ist entscheidend für die Stabilität eines Compositors. Rust bietet integrierte Test-Frameworks, die Unit- und Integrationstests unterstützen.46

#### 6.2.1. Unit-Tests

Unit-Tests konzentrieren sich darauf, einzelne Code-Einheiten isoliert zu testen, um schnell zu identifizieren, wo Probleme auftreten.46 Sie werden typischerweise im selben `src`-Verzeichnis wie der zu testende Code platziert, oft in einem `mod tests`-Modul, das mit `#[cfg(test)]` annotiert ist. Diese Annotation stellt sicher, dass der Testcode nur beim Ausführen von `cargo test` kompiliert wird, nicht bei `cargo build`, was Kompilierzeiten und die Größe des Binärartefakts reduziert.46

#### 6.2.2. Integrationstests

Integrationstests sind extern zur Bibliothek und nutzen nur deren öffentliche Schnittstelle, um zu überprüfen, ob mehrere Teile der Bibliothek korrekt zusammenarbeiten.46 Sie werden in einem separaten `tests`-Verzeichnis auf der obersten Ebene des Projekts platmiert. Jede `.rs`-Datei in diesem Verzeichnis wird als separate Crate kompiliert.46 Gemeinsamer Code für Integrationstests kann in einem `tests/common/mod.rs`-Modul abgelegt werden.48

#### 6.2.3. End-to-End-Tests mit virtuellen Clients/Backends

Das Testen eines Wayland-Compositors erfordert oft die Simulation von Client-Interaktionen und Hardware-Backends.

- **Mocking von externem Zustand:** Rust-Entwickler bevorzugen es, Dinge zu mocken, die globalen oder externen Zustand erfordern, wie I/O, und diese durch isolierbare, In-Process-Optionen zu ersetzen, um Tests schnell und synchron zu machen.49 Traditionelles Mocking, das spezifische Methodenaufrufe in einer bestimmten Reihenfolge erwartet, kann die Tests zu stark an die Implementierung koppeln.49
- **Interne Mutabilität für Mock-Verhalten:** Für einfache, single-threaded Fälle können `Cell` und `RefCell` verwendet werden, um interne Mutabilität zu ermöglichen und Mock-Verhalten zu implementieren.49 Für Multi-Threading sind `Mutex` und `RwLock` erforderlich.50
- **Virtuelle Backends und Clients:** Smithay selbst bietet Backends wie `--x11` und `--winit`, die es ermöglichen, den Compositor als Client in einer bestehenden X11- oder Wayland-Session zu starten, was für Entwicklung und Debugging nützlich ist.25 Für automatisierte Tests können virtuelle Clients oder Backends verwendet werden, um das Wayland-Protokoll zu simulieren und die Reaktion des Compositors zu überprüfen. Projekte wie `wprs` implementieren einen Wayland-Compositor, der den Wayland-Sitzungszustand serialisiert und an einen Client sendet, was für Remote-Desktop-Zugriff und möglicherweise für Testzwecke genutzt werden kann.54

### 6.3. Performance-Anforderungen und Optimierungsstrategien

Performance ist ein kritischer nicht-funktionaler Aspekt für einen Wayland-Compositor. Rusts Fokus auf Zero-Cost Abstractions und statische Speicherverwaltung ermöglicht oft eine hohe Performance, selbst bei naivem Code.55

#### 6.3.1. Benchmarking und Profiling

Blindes Optimieren sollte vermieden werden.56 Stattdessen sollten Benchmarking- und Profiling-Tools eingesetzt werden, um Engpässe zu identifizieren.57 Dies beinhaltet die Analyse von CPU-Nutzung, Speicherzugriffsmustern und I/O-Operationen.49 Werkzeuge wie `perf` oder `Valgrind` (für C-FFI-Teile) können hierbei wertvolle Einblicke liefern.

#### 6.3.2. Speicherverwaltung und Zuweisungsstrategien

Effiziente Speicherverwaltung ist entscheidend.

- **Minimierung von Heap-Allokationen:** Wo immer möglich, sollte die Stack-Allokation für kleine, festgroße Daten bevorzugt werden. Heap-Allokationen sollten minimiert werden, indem Speicher wiederverwendet oder Datenstrukturen verwendet werden, die dynamisch wachsen und schrumpfen können.57
- **Rusts Ownership-Modell:** Das Ownership-System von Rust gewährleistet, dass Speicher automatisch freigegeben wird, wenn er nicht mehr benötigt wird, was Speicherlecks verhindert.6 Obwohl Rust Lecks in bestimmten, expliziten Szenarien zulässt (z.B. `Box::leak` oder Referenzzähler-Zyklen 58), sind unabsichtliche Lecks in idiomatischem Safe Rust selten.
- **Caching-Strategien:** Für häufig benötigte Daten kann In-Memory-Caching implementiert werden, um die Zugriffszeiten zu reduzieren. Bibliotheken wie `cached` oder `moka` bieten verschiedene Cache-Implementierungen (z.B. LRU) und Funktionen zur Memoization.61 Für Persistenz können Strategien wie Write-Through-Caching mit Redis oder dateibasierte Lösungen wie `persy` in Betracht gezogen werden.63

#### 6.3.3. Zero-Cost Abstractions und Compile-Time Optimierungen

Rusts "Zero-Cost Abstractions" bedeuten, dass Abstraktionen keine Laufzeitkosten verursachen.55 Typzustände und `const`-Generics sind Beispiele dafür, wie das Typsystem zur Compile-Zeit Validierungen und Optimierungen ermöglicht, die zur Laufzeit keinen Overhead haben.55 Die Verwendung von `const` und `static` für unveränderliche Werte sowie das Schreiben kleiner, reiner Funktionen, die inlined werden können, tragen zu Compile-Time-Optimierungen bei.56

#### 6.3.4. Concurrency und Parallelisierung

Rusts Ownership- und Typsysteme sind darauf ausgelegt, Datenrennen und andere häufige Concurrency-Fehler zur Compile-Zeit zu verhindern, was als "Fearless Concurrency" bezeichnet wird.69

- **Shared-State Concurrency:** Für den Zugriff auf gemeinsame Daten von mehreren Threads werden `Mutex` und `Arc` (Atomic Reference Counted) verwendet.70 `Arc` ermöglicht die sichere gemeinsame Besitznahme von Werten über Threads hinweg, während `Mutex` den exklusiven Zugriff auf die Daten gewährleistet.52 Die `MutexGuard`-Smart-Pointer implementieren `Drop`, um das Lock automatisch freizugeben, wenn sie den Gültigkeitsbereich verlassen, was das Risiko des Vergessens des Freigebens minimiert.52
- **Callback-basierte Event-Loops:** Smithay ist um `calloop` herum aufgebaut, eine Event-Loop, die eine zentrale mutable Referenz zum Compositor-Zustand an Callbacks übergibt. Dies ermöglicht den sequentiellen Zugriff auf den Zustand ohne zusätzliche Synchronisationsmechanismen, da die Callback-Aufrufe immer sequenziell erfolgen.8

### 6.4. Logging und Monitoring

Eine umfassende Logging- und Monitoring-Strategie ist für den Betrieb eines Wayland-Compositors unerlässlich.

#### 6.4.1. Strukturierte Protokollierung

`tracing` ist das empfohlene Framework für die Anwendungsebene-Protokollierung in Rust.8 Es ermöglicht die Erfassung strukturierter Telemetriedaten, einschließlich Metriken, Traces und Logs.11 Strukturierte Logs sind maschinenlesbar und erleichtern die Analyse und Fehlersuche erheblich. Es wird empfohlen, direkte Integrationen mit den Kern-OpenTelemetry-Bibliotheken zu verwenden, anstatt sich auf Zwischen-Dependencies zu verlassen.11 Für Release-Builds sollte der Log-Level zur Compile-Zeit begrenzt werden, um Performance zu optimieren und sensible Daten zu schützen.8

#### 6.4.2. Echtzeit-Monitoring und Metriken

Zusätzlich zur Protokollierung ist das Sammeln von Metriken für das Echtzeit-Monitoring wichtig. `tracing` kann auch für Metriken verwendet werden, um z.B. die Verarbeitungszeit von Frames oder die Anzahl der aktiven Clients zu verfolgen.11 Die Integration mit externen Monitoring-Systemen (z.B. Datadog über OpenTelemetry) ermöglicht eine umfassende Observability, einschließlich Sicherheitsmonitoring, Live-Container-Monitoring und Live-Prozess-Monitoring.11 Dies hilft, Anomalien zu erkennen, Performance-Engpässe zu identifizieren und die allgemeine Systemgesundheit zu überwachen.

---

## Fazit und Empfehlungen

Die Entwicklung eines Wayland-Compositors in Rust unter Verwendung des Smithay-Frameworks bietet eine robuste Grundlage für leistungsstarke und sichere Desktop-Umgebungen. Die Architektur profitiert von Rusts inhärenten Sicherheitsgarantien, insbesondere im Hinblick auf Speichersicherheit und Fearless Concurrency, was die Entwicklung komplexer Systemsoftware erheblich vereinfacht und die Wahrscheinlichkeit von Laufzeitfehlern reduziert.

Die detaillierte Spezifikation funktionaler und nicht-funktionaler Anforderungen ist entscheidend, um den Umfang des Projekts klar zu definieren und die Erwartungen an das System zu erfüllen. Die Nutzung modularer Komponenten von Smithay für Kern-Wayland-Protokolle, Eingabe- und Ausgabeverwaltung sowie die Integration mit System-Backends wie DRM/KMS und `libinput` ermöglicht eine effiziente Entwicklung.

Eine proaktive Fehlerbehandlung, die zwischen wiederherstellbaren und nicht wiederherstellbaren Fehlern unterscheidet, ist unerlässlich. Die Implementierung spezifischer Fehler-Enums, die Nutzung des `?`-Operators und die Integration eines strukturierten Logging-Frameworks wie `tracing` tragen maßgeblich zur Diagnostizierbarkeit und Robustheit bei. Die umfassende Behandlung von Edge-Cases, von unerwartetem Client-Verhalten bis hin zu Hardware-Hotplugs, gewährleistet die Stabilität des Compositors unter variierenden Betriebsbedingungen.

Für die Qualitätssicherung sind Unit-, Integrations- und End-to-End-Tests mit virtuellen Clients und Backends von großer Bedeutung. Diese Teststrategien ermöglichen eine gründliche Validierung der Systemfunktionalität und -interaktionen. Performance-Optimierungen sollten auf fundierten Benchmarking- und Profiling-Ergebnissen basieren, wobei Rusts Zero-Cost Abstractions und Concurrency-Modelle optimal genutzt werden.

Zusammenfassend lässt sich festhalten, dass der Aufbau eines Wayland-Compositors in Rust eine vielversprechende Strategie darstellt, die moderne Programmierprinzipien mit den spezifischen Anforderungen eines Display-Servers verbindet. Die Einhaltung der beschriebenen Architekturen, Designmuster und Qualitätssicherungsmaßnahmen wird zu einem stabilen, leistungsfähigen und wartbaren System führen.

**Empfehlungen:**

1. **Priorisierung der Kernfunktionalität:** Zuerst die Implementierung der grundlegenden Wayland-Protokolle und der Kern-Compositor-Logik sicherstellen, bevor erweiterte Funktionen oder Desktop-Shell-Komponenten hinzugefügt werden.
2. **Iterative Entwicklung und Tests:** Eine agile Entwicklung mit häufigen Testzyklen (Unit, Integration) ist entscheidend, um Fehler frühzeitig zu erkennen und die Komplexität des Systems schrittweise zu bewältigen.
3. **Dokumentation und Code-Qualität:** Eine umfassende Dokumentation der API und der Implementierungsdetails, kombiniert mit idiomatischem Rust-Code und der Nutzung von Linting-Tools (z.B. Clippy), ist für die langfristige Wartbarkeit und die Zusammenarbeit im Team unerlässlich.
4. **Kontinuierliches Monitoring:** Ein robustes Logging- und Monitoring-System sollte von Anfang an integriert werden, um die Systemgesundheit in Echtzeit zu überwachen und proaktiv auf Probleme reagieren zu können.
5. **Community-Engagement:** Die Wayland- und Rust-Ökosysteme sind aktiv. Das Engagement in der Community kann Zugang zu Best Practices, Bibliotheken und Unterstützung bieten, insbesondere bei der Bewältigung komplexer Wayland-Protokollerweiterungen oder Hardware-Integrationen.

## 8. Risiken und Mitigation

- **Komplexität der Wayland-Protokolle:** Viele Protokolle haben subtile Details und erfordern sorgfältige Implementierung.
    
    - **Mitigation:** Intensives Studium der Spezifikationen, Nutzung von Smithays Abstraktionen, Testen mit verschiedenen Clients.
        
- **Smithay API-Änderungen:** Smithay ist ein sich entwickelndes Projekt.
    
    - **Mitigation:** Regelmäßige Beobachtung von Smithay-Releases, flexible Architektur, um Anpassungen zu erleichtern.
        
- **Treiber- und Hardware-Kompatibilität:** Insbesondere für DMA-BUF und fortgeschrittene Rendering-Features.
    
    - **Mitigation:** Testen auf verschiedener Hardware, Fallback-Mechanismen (z.B. SHM statt DMA-BUF, wenn Import fehlschlägt).
        
- **Performance-Engpässe:** In Rendering, Input-Verarbeitung oder Event-Dispatching.
    
    - **Mitigation:** Kontinuierliches Profiling, Optimierung kritischer Pfade, effizientes Damage Tracking.
        
- **Sicherheitslücken:** Bei FFI, Shared Memory oder fehlerhafter Protokoll-Implementierung.
    
    - **Mitigation:** Strenge Code-Reviews, Minimierung von `unsafe`-Code, Nutzung von Rusts Sicherheitsfeatures.
        
- **Zustandsmanagement-Komplexität:** Der Compositor-Zustand kann sehr komplex werden.
    
    - **Mitigation:** Klare Datenstrukturen, modulare Architektur, sorgfältige Verwendung von Synchronisationsprimitiven.
        

## 9. Geschätzter Gesamt-Aufwand und Zeitplan

Die Implementierung eines voll funktionsfähigen Wayland-Compositors mit den genannten Protokollen ist ein sehr umfangreiches Unterfangen.

- **Phase 1 (Kern-Protokolle - Fundament):** `wl_display`, `wl_registry`, `wl_compositor`, `wl_surface`, `wl_shm`, `wl_buffer`, `wl_callback`, `wl_output`, `wl_subcompositor`.
    
    - Aufwand: Sehr Hoch (ca. 12-16 Personenwochen). Fokus auf stabile Basis, Event-Loop, grundlegendes Rendering (einfarbige Flächen).
        
- **Phase 2 (Input-System und Basis-Shell):** `wl_seat`, `wl_keyboard`, `wl_pointer`, `wl_touch`, `xdg_shell` (Basis-Toplevels).
    
    - Aufwand: Sehr Hoch (ca. 14-18 Personenwochen). Fokus auf Input-Verarbeitung, Fokusmanagement, einfache Fensteranzeige.
        
- **Phase 3 (Erweiterte Shell-Funktionen und Desktop-Elemente):** `wlr_layer_shell_v1`, `zxdg_decoration_manager_v1`, `xdg_activation_v1`.
    
    - Aufwand: Hoch (ca. 10-14 Personenwochen).
        
- **Phase 4 (Daten-Austausch und fortgeschrittene Features):** `wl_data_device_manager` (und zugehörige), `zwp_linux_dmabuf_v1`, `wp_presentation_time`, `input_method_unstable_v2`, `text_input_unstable_v3`.
    
    - Aufwand: Sehr Hoch (ca. 15-20 Personenwochen). DMA-BUF und IME sind besonders komplex.
        
- **Phase 5 (Spezifische Optimierungen und weitere Protokolle):** `wp_fractional_scale_v1`, `wp_viewport_v1`, `zxdg_output_manager_v1`, `wp_single_pixel_buffer_v1`, `wp_relative_pointer_manager_v1`, `wp_locked_pointer_manager_v1`, `wlr_foreign_toplevel_management_unstable_v1`, `idle_notify_unstable_v1`.
    
    - Aufwand: Hoch (ca. 10-15 Personenwochen).
        
- **Kontinuierliche Integration, Testing, Bugfixing, Dokumentation:** Laufender Aufwand über alle Phasen.
    

**Gesamtschätzung (grob):** 60 - 85+ Personenwochen, abhängig von Teamgröße, Erfahrung und Komplexität der NovaDE-spezifischen Anforderungen. Dies ist eine konservative Schätzung für eine qualitativ hochwertige Implementierung.

Dieser Plan ist ein lebendes Dokument und sollte regelmäßig überprüft und an den Projektfortschritt und neue Erkenntnisse angepasst werden.