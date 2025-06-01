# SPEC-LAYER-SYSTEM-v1.0.0: NovaDE Systemschicht-Spezifikation (Teil 2)

## 6. Datenmodell (Fortsetzung)

### 6.4 Audio-Management-Typen

```
ENTITÄT: AudioDevice
BESCHREIBUNG: Repräsentation eines Audiogeräts
ATTRIBUTE:
  - NAME: id
    TYP: u32
    BESCHREIBUNG: Eindeutige ID des Geräts
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name des Geräts
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: description
    TYP: String
    BESCHREIBUNG: Beschreibung des Geräts
    WERTEBEREICH: Zeichenkette
    STANDARDWERT: ""
  - NAME: device_type
    TYP: AudioDeviceType
    BESCHREIBUNG: Typ des Audiogeräts
    WERTEBEREICH: {Sink, Source, Virtual}
    STANDARDWERT: Keiner
  - NAME: state
    TYP: AudioDeviceState
    BESCHREIBUNG: Zustand des Geräts
    WERTEBEREICH: {Running, Idle, Suspended, Error}
    STANDARDWERT: AudioDeviceState::Idle
  - NAME: volume
    TYP: f32
    BESCHREIBUNG: Lautstärke des Geräts (0.0 bis 1.0)
    WERTEBEREICH: [0.0, 1.0]
    STANDARDWERT: 0.5
  - NAME: muted
    TYP: bool
    BESCHREIBUNG: Ob das Gerät stummgeschaltet ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: channels
    TYP: u8
    BESCHREIBUNG: Anzahl der Audiokanäle
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 2
  - NAME: default
    TYP: bool
    BESCHREIBUNG: Ob das Gerät das Standardgerät ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: properties
    TYP: HashMap<String, String>
    BESCHREIBUNG: Zusätzliche Eigenschaften des Geräts
    WERTEBEREICH: Beliebige Schlüssel-Wert-Paare
    STANDARDWERT: Leere HashMap
INVARIANTEN:
  - id muss gültig sein
  - name darf nicht leer sein
  - volume muss im Bereich [0.0, 1.0] liegen
  - channels muss größer als 0 sein
```

```
ENTITÄT: StreamInfo
BESCHREIBUNG: Informationen über einen Audiostream
ATTRIBUTE:
  - NAME: id
    TYP: u32
    BESCHREIBUNG: Eindeutige ID des Streams
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name des Streams
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: app_name
    TYP: String
    BESCHREIBUNG: Name der Anwendung, die den Stream erzeugt
    WERTEBEREICH: Zeichenkette
    STANDARDWERT: ""
  - NAME: device_id
    TYP: u32
    BESCHREIBUNG: ID des zugehörigen Geräts
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
  - NAME: volume
    TYP: f32
    BESCHREIBUNG: Lautstärke des Streams (0.0 bis 1.0)
    WERTEBEREICH: [0.0, 1.0]
    STANDARDWERT: 0.5
  - NAME: muted
    TYP: bool
    BESCHREIBUNG: Ob der Stream stummgeschaltet ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: corked
    TYP: bool
    BESCHREIBUNG: Ob der Stream pausiert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: properties
    TYP: HashMap<String, String>
    BESCHREIBUNG: Zusätzliche Eigenschaften des Streams
    WERTEBEREICH: Beliebige Schlüssel-Wert-Paare
    STANDARDWERT: Leere HashMap
INVARIANTEN:
  - id muss gültig sein
  - name darf nicht leer sein
  - device_id muss gültig sein
  - volume muss im Bereich [0.0, 1.0] liegen
```

```
ENTITÄT: AudioCommand
BESCHREIBUNG: Befehle für die Audiosteuerung
ATTRIBUTE:
  - NAME: command_type
    TYP: Enum
    BESCHREIBUNG: Typ des Befehls
    WERTEBEREICH: {
      SetDeviceVolume { device_id: u32, volume: f32 },
      SetDeviceMute { device_id: u32, muted: bool },
      SetStreamVolume { stream_id: u32, volume: f32 },
      SetStreamMute { stream_id: u32, muted: bool },
      SetDefaultDevice { device_id: u32, device_type: AudioDeviceType },
      SuspendDevice { device_id: u32 },
      ResumeDevice { device_id: u32 },
      KillStream { stream_id: u32 }
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Bei SetDeviceVolume muss volume im Bereich [0.0, 1.0] liegen
  - Bei SetStreamVolume muss volume im Bereich [0.0, 1.0] liegen
```

```
ENTITÄT: AudioEvent
BESCHREIBUNG: Ereignisse im Audiomanagement
ATTRIBUTE:
  - NAME: event_type
    TYP: Enum
    BESCHREIBUNG: Typ des Ereignisses
    WERTEBEREICH: {
      DeviceAdded { device: AudioDevice },
      DeviceRemoved { device_id: u32 },
      DeviceChanged { device: AudioDevice },
      StreamAdded { stream: StreamInfo },
      StreamRemoved { stream_id: u32 },
      StreamChanged { stream: StreamInfo },
      DefaultDeviceChanged { device_id: u32, device_type: AudioDeviceType }
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Bei DeviceAdded und DeviceChanged muss device gültig sein
  - Bei StreamAdded und StreamChanged muss stream gültig sein
```

### 6.5 MCP-Client-Typen

```
ENTITÄT: McpServerConfig
BESCHREIBUNG: Konfiguration für MCP-Server
ATTRIBUTE:
  - NAME: server_url
    TYP: String
    BESCHREIBUNG: URL des MCP-Servers
    WERTEBEREICH: Gültige URL
    STANDARDWERT: Keiner
  - NAME: api_key
    TYP: Option<String>
    BESCHREIBUNG: Optionaler API-Schlüssel für den Server
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
  - NAME: timeout_ms
    TYP: u32
    BESCHREIBUNG: Timeout für Serveranfragen in Millisekunden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 5000
  - NAME: retry_count
    TYP: u8
    BESCHREIBUNG: Anzahl der Wiederholungsversuche bei Fehlern
    WERTEBEREICH: Ganzzahlen
    STANDARDWERT: 3
  - NAME: retry_delay_ms
    TYP: u32
    BESCHREIBUNG: Verzögerung zwischen Wiederholungsversuchen in Millisekunden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 1000
INVARIANTEN:
  - server_url muss eine gültige URL sein
  - timeout_ms muss größer als 0 sein
```

```
ENTITÄT: McpClientSystemEvent
BESCHREIBUNG: Systemereignisse für MCP
ATTRIBUTE:
  - NAME: event_type
    TYP: Enum
    BESCHREIBUNG: Typ des Ereignisses
    WERTEBEREICH: {
      ConnectionEstablished { server_url: String },
      ConnectionLost { server_url: String, reason: String },
      ConnectionError { server_url: String, error: String },
      ModelLoaded { model_id: String },
      ModelUnloaded { model_id: String },
      ModelError { model_id: String, error: String },
      SystemResourceWarning { resource_type: String, usage_percent: f32 }
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Bei ConnectionEstablished, ConnectionLost und ConnectionError muss server_url eine gültige URL sein
  - Bei ModelLoaded, ModelUnloaded und ModelError darf model_id nicht leer sein
  - Bei SystemResourceWarning muss usage_percent im Bereich [0.0, 100.0] liegen
```

### 6.6 Power-Management-Typen

```
ENTITÄT: PowerState
BESCHREIBUNG: Energiezustand des Systems
ATTRIBUTE:
  - NAME: state_type
    TYP: Enum
    BESCHREIBUNG: Typ des Energiezustands
    WERTEBEREICH: {
      OnAC,
      OnBattery { percent: f32, time_remaining_minutes: Option<i32> },
      LowBattery { percent: f32, time_remaining_minutes: Option<i32> },
      CriticalBattery { percent: f32, time_remaining_minutes: Option<i32> }
    }
    STANDARDWERT: OnAC
INVARIANTEN:
  - Bei OnBattery, LowBattery und CriticalBattery muss percent im Bereich [0.0, 100.0] liegen
  - Bei time_remaining_minutes, wenn vorhanden, muss der Wert größer oder gleich 0 sein
```

```
ENTITÄT: DpmsState
BESCHREIBUNG: DPMS-Zustand (Display Power Management Signaling)
ATTRIBUTE:
  - NAME: state_type
    TYP: Enum
    BESCHREIBUNG: Typ des DPMS-Zustands
    WERTEBEREICH: {On, Standby, Suspend, Off}
    STANDARDWERT: On
```

```
ENTITÄT: IdleState
BESCHREIBUNG: Leerlaufzustand des Systems
ATTRIBUTE:
  - NAME: state_type
    TYP: Enum
    BESCHREIBUNG: Typ des Leerlaufzustands
    WERTEBEREICH: {
      Active,
      IdleWarning { idle_time_seconds: u32 },
      Idle { idle_time_seconds: u32 },
      LockScreen { idle_time_seconds: u32 },
      Suspend { idle_time_seconds: u32 }
    }
    STANDARDWERT: Active
INVARIANTEN:
  - Bei IdleWarning, Idle, LockScreen und Suspend muss idle_time_seconds größer als 0 sein
```

## 7. Verhaltensmodell

### 7.1 Compositor-Initialisierung

```
ZUSTANDSAUTOMAT: CompositorInitialization
BESCHREIBUNG: Prozess der Compositor-Initialisierung
ZUSTÄNDE:
  - NAME: Uninitialized
    BESCHREIBUNG: Compositor ist nicht initialisiert
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: CreatingDisplay
    BESCHREIBUNG: Wayland-Display wird erstellt
    EINTRITTSAKTIONEN: Display-Ressourcen allokieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: InitializingBackend
    BESCHREIBUNG: Backend wird initialisiert
    EINTRITTSAKTIONEN: Backend-Ressourcen allokieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: CreatingRenderer
    BESCHREIBUNG: Renderer wird erstellt
    EINTRITTSAKTIONEN: Renderer-Ressourcen allokieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: RegisteringGlobals
    BESCHREIBUNG: Wayland-Globals werden registriert
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: Ready
    BESCHREIBUNG: Compositor ist bereit
    EINTRITTSAKTIONEN: CompositorReadyEvent auslösen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler bei der Compositor-Initialisierung
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Uninitialized
    NACH: CreatingDisplay
    EREIGNIS: Initialisierung des Compositors
    BEDINGUNG: Keine
    AKTIONEN: Konfiguration laden
  - VON: CreatingDisplay
    NACH: InitializingBackend
    EREIGNIS: Display erfolgreich erstellt
    BEDINGUNG: Keine
    AKTIONEN: Backend-Typ bestimmen
  - VON: CreatingDisplay
    NACH: Error
    EREIGNIS: Fehler bei der Display-Erstellung
    BEDINGUNG: Keine
    AKTIONEN: CompositorCoreError erstellen
  - VON: InitializingBackend
    NACH: CreatingRenderer
    EREIGNIS: Backend erfolgreich initialisiert
    BEDINGUNG: Keine
    AKTIONEN: Renderer-Typ bestimmen
  - VON: InitializingBackend
    NACH: Error
    EREIGNIS: Fehler bei der Backend-Initialisierung
    BEDINGUNG: Keine
    AKTIONEN: CompositorCoreError erstellen
  - VON: CreatingRenderer
    NACH: RegisteringGlobals
    EREIGNIS: Renderer erfolgreich erstellt
    BEDINGUNG: Keine
    AKTIONEN: Wayland-Globals vorbereiten
  - VON: CreatingRenderer
    NACH: Error
    EREIGNIS: Fehler bei der Renderer-Erstellung
    BEDINGUNG: Keine
    AKTIONEN: CompositorCoreError erstellen
  - VON: RegisteringGlobals
    NACH: Ready
    EREIGNIS: Globals erfolgreich registriert
    BEDINGUNG: Keine
    AKTIONEN: Event-Loop starten
  - VON: RegisteringGlobals
    NACH: Error
    EREIGNIS: Fehler bei der Global-Registrierung
    BEDINGUNG: Keine
    AKTIONEN: CompositorCoreError erstellen
INITIALZUSTAND: Uninitialized
ENDZUSTÄNDE: [Ready, Error]
```

### 7.2 Input-Verarbeitung

```
ZUSTANDSAUTOMAT: InputProcessing
BESCHREIBUNG: Prozess der Eingabeverarbeitung
ZUSTÄNDE:
  - NAME: Idle
    BESCHREIBUNG: Keine Eingabe wird verarbeitet
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: ProcessingKeyboard
    BESCHREIBUNG: Tastatureingabe wird verarbeitet
    EINTRITTSAKTIONEN: Tastaturzustand aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: ProcessingPointer
    BESCHREIBUNG: Mauseingabe wird verarbeitet
    EINTRITTSAKTIONEN: Mausposition aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: ProcessingTouch
    BESCHREIBUNG: Toucheingabe wird verarbeitet
    EINTRITTSAKTIONEN: Touch-Sequenz starten
    AUSTRITTSAKTIONEN: Keine
  - NAME: ProcessingGesture
    BESCHREIBUNG: Geste wird verarbeitet
    EINTRITTSAKTIONEN: Geste erkennen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler bei der Eingabeverarbeitung
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Idle
    NACH: ProcessingKeyboard
    EREIGNIS: Tastaturereignis empfangen
    BEDINGUNG: Keine
    AKTIONEN: Tastaturereignis klassifizieren
  - VON: Idle
    NACH: ProcessingPointer
    EREIGNIS: Mausereignis empfangen
    BEDINGUNG: Keine
    AKTIONEN: Mausereignis klassifizieren
  - VON: Idle
    NACH: ProcessingTouch
    EREIGNIS: Touchereignis empfangen
    BEDINGUNG: Keine
    AKTIONEN: Touchereignis klassifizieren
  - VON: ProcessingTouch
    NACH: ProcessingGesture
    EREIGNIS: Geste erkannt
    BEDINGUNG: Keine
    AKTIONEN: Geste klassifizieren
  - VON: ProcessingKeyboard
    NACH: Idle
    EREIGNIS: Tastaturereignis verarbeitet
    BEDINGUNG: Keine
    AKTIONEN: Tastaturereignis an Fokusziel weiterleiten
  - VON: ProcessingPointer
    NACH: Idle
    EREIGNIS: Mausereignis verarbeitet
    BEDINGUNG: Keine
    AKTIONEN: Mausereignis an Ziel unter Cursor weiterleiten
  - VON: ProcessingTouch
    NACH: Idle
    EREIGNIS: Touchereignis verarbeitet
    BEDINGUNG: Keine
    AKTIONEN: Touchereignis an Ziel unter Touch weiterleiten
  - VON: ProcessingGesture
    NACH: Idle
    EREIGNIS: Geste verarbeitet
    BEDINGUNG: Keine
    AKTIONEN: Gestenereignis an Ziel weiterleiten
  - VON: ProcessingKeyboard
    NACH: Error
    EREIGNIS: Fehler bei der Tastaturverarbeitung
    BEDINGUNG: Keine
    AKTIONEN: InputError erstellen
  - VON: ProcessingPointer
    NACH: Error
    EREIGNIS: Fehler bei der Mausverarbeitung
    BEDINGUNG: Keine
    AKTIONEN: InputError erstellen
  - VON: ProcessingTouch
    NACH: Error
    EREIGNIS: Fehler bei der Touchverarbeitung
    BEDINGUNG: Keine
    AKTIONEN: InputError erstellen
  - VON: ProcessingGesture
    NACH: Error
    EREIGNIS: Fehler bei der Gestenverarbeitung
    BEDINGUNG: Keine
    AKTIONEN: InputError erstellen
  - VON: Error
    NACH: Idle
    EREIGNIS: Fehler behandelt
    BEDINGUNG: Keine
    AKTIONEN: Fehler protokollieren
INITIALZUSTAND: Idle
ENDZUSTÄNDE: []
```

## 8. Fehlerbehandlung

### 8.1 Fehlerbehandlungsstrategie

1. Alle Fehler MÜSSEN über spezifische Fehlertypen zurückgegeben werden.
2. Fehlertypen MÜSSEN mit `thiserror` definiert werden.
3. Fehler MÜSSEN kontextuelle Informationen enthalten.
4. Fehlerketten MÜSSEN bei der Weitergabe oder beim Wrappen von Fehlern erhalten bleiben.
5. Panics sind VERBOTEN, außer in Fällen, die explizit dokumentiert sind.

### 8.2 Systemspezifische Fehlertypen

```
ENTITÄT: CompositorCoreError
BESCHREIBUNG: Kernfehler im Compositor-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      DisplayCreationFailed { reason: String, source: Option<Box<dyn std::error::Error>> },
      BackendInitializationFailed { backend_type: String, reason: String, source: Option<Box<dyn std::error::Error>> },
      RendererCreationFailed { renderer_type: String, reason: String, source: Option<Box<dyn std::error::Error>> },
      GlobalRegistrationFailed { global: String, reason: String },
      SurfaceError { surface_id: u32, reason: String },
      OutputError { output_id: u32, reason: String },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

```
ENTITÄT: XdgShellError
BESCHREIBUNG: Fehler im XDG-Shell-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      InvalidSurfaceRole { surface_id: u32, requested_role: String, current_role: String },
      InvalidSurfaceState { surface_id: u32, reason: String },
      InvalidGeometry { surface_id: u32, reason: String },
      InvalidConfiguration { surface_id: u32, reason: String },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

```
ENTITÄT: InputError
BESCHREIBUNG: Fehler im Input-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      DeviceInitializationFailed { device_name: String, reason: String, source: Option<Box<dyn std::error::Error>> },
      DeviceNotFound { device_id: u32 },
      KeymapError { reason: String, source: Option<Box<dyn std::error::Error>> },
      FocusError { reason: String },
      LibinputError { reason: String, source: Option<Box<dyn std::error::Error>> },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

```
ENTITÄT: AudioError
BESCHREIBUNG: Fehler im Audio-Management-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      PipeWireConnectionFailed { reason: String, source: Option<Box<dyn std::error::Error>> },
      DeviceNotFound { device_id: u32 },
      StreamNotFound { stream_id: u32 },
      InvalidVolume { value: f32, reason: String },
      OperationFailed { operation: String, reason: String },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

```
ENTITÄT: McpSystemClientError
BESCHREIBUNG: Fehler im MCP-Client-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      ConnectionFailed { server_url: String, reason: String, source: Option<Box<dyn std::error::Error>> },
      AuthenticationFailed { server_url: String, reason: String },
      RequestFailed { request_type: String, reason: String, source: Option<Box<dyn std::error::Error>> },
      ModelNotFound { model_id: String },
      ModelLoadFailed { model_id: String, reason: String },
      ProtocolError { message: String, source: Option<Box<dyn std::error::Error>> },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

## 9. Leistungsanforderungen

### 9.1 Allgemeine Leistungsanforderungen

1. Die Systemschicht MUSS hohe Performance und geringe Latenz bieten.
2. Die Systemschicht MUSS effizient mit Ressourcen umgehen.
3. Die Systemschicht MUSS für Echtzeit-Anwendungen geeignet sein.

### 9.2 Spezifische Leistungsanforderungen

1. Der Compositor MUSS eine Framerate von mindestens 60 FPS aufrechterhalten.
2. Die Eingabeverarbeitung MUSS eine Latenz von unter 10ms haben.
3. Die Audiolatenz MUSS unter 20ms liegen.
4. Die D-Bus-Kommunikation MUSS in unter 5ms abgeschlossen sein.
5. Die MCP-Kommunikation MUSS in unter 100ms abgeschlossen sein.

## 10. Sicherheitsanforderungen

### 10.1 Allgemeine Sicherheitsanforderungen

1. Die Systemschicht MUSS memory-safe sein.
2. Die Systemschicht MUSS thread-safe sein.
3. Die Systemschicht MUSS robust gegen Fehleingaben sein.

### 10.2 Spezifische Sicherheitsanforderungen

1. Der Compositor MUSS Clients isolieren, um gegenseitige Beeinträchtigung zu verhindern.
2. Die Eingabeverarbeitung MUSS Eingaben validieren, um Injection-Angriffe zu verhindern.
3. Die D-Bus-Kommunikation MUSS Zugriffskontrollen implementieren.
4. Die MCP-Kommunikation MUSS verschlüsselt sein.
5. Die Portals MÜSSEN Sandboxing-Mechanismen implementieren.

## 11. Testkriterien

### 11.1 Allgemeine Testkriterien

1. Jedes Modul MUSS Einheitstests haben.
2. Jede öffentliche Funktion MUSS getestet sein.
3. Jeder Fehlerfall MUSS getestet sein.
4. Leistungskritische Pfade MÜSSEN Benchmark-Tests haben.

### 11.2 Spezifische Testkriterien

1. Der Compositor MUSS mit verschiedenen Fensterkonstellationen getestet sein.
2. Die Eingabeverarbeitung MUSS mit verschiedenen Eingabegeräten getestet sein.
3. Die Audiokomponente MUSS mit verschiedenen Audiogeräten getestet sein.
4. Die D-Bus-Schnittstellen MÜSSEN mit verschiedenen Diensten getestet sein.
5. Die MCP-Kommunikation MUSS mit verschiedenen Modellen getestet sein.

## 12. Anhänge

### 12.1 Referenzierte Dokumente

1. SPEC-ROOT-v1.0.0: NovaDE Spezifikationswurzel
2. SPEC-LAYER-CORE-v1.0.0: Spezifikation der Kernschicht
3. SPEC-LAYER-DOMAIN-v1.0.0: Spezifikation der Domänenschicht
4. SPEC-MODULE-SYSTEM-COMPOSITOR-v1.0.0: Spezifikation des Compositor-Moduls
5. SPEC-MODULE-SYSTEM-INPUT-v1.0.0: Spezifikation des Input-Moduls
6. SPEC-MODULE-SYSTEM-DBUS-v1.0.0: Spezifikation des D-Bus-Moduls
7. SPEC-MODULE-SYSTEM-AUDIO-v1.0.0: Spezifikation des Audio-Moduls
8. SPEC-MODULE-SYSTEM-MCP-v1.0.0: Spezifikation des MCP-Moduls

### 12.2 Externe Abhängigkeiten

1. `smithay`: Für den Wayland-Compositor
2. `libinput`: Für die Eingabeverarbeitung
3. `zbus`: Für D-Bus-Kommunikation
4. `pipewire-rs`: Für Audioverarbeitung
5. `mcp_client_rs`: Für Model Context Protocol
6. `xkbcommon`: Für Tastaturverarbeitung
