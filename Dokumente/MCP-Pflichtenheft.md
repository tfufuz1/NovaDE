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