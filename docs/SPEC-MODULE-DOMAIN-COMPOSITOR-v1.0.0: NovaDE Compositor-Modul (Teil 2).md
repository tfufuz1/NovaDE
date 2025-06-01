# SPEC-MODULE-DOMAIN-COMPOSITOR-v1.0.0: NovaDE Compositor-Modul (Teil 2)

## 6. Datenmodell (Fortsetzung)

### 6.11 Surface

```
ENTITÄT: Surface
BESCHREIBUNG: Oberfläche für die Darstellung von Inhalten
ATTRIBUTE:
  - NAME: id
    TYP: SurfaceId
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Gültige SurfaceId
    STANDARDWERT: Keiner
  - NAME: size
    TYP: Size
    BESCHREIBUNG: Größe der Oberfläche
    WERTEBEREICH: Gültige Size
    STANDARDWERT: Size { width: 0, height: 0 }
  - NAME: format
    TYP: PixelFormat
    BESCHREIBUNG: Pixelformat der Oberfläche
    WERTEBEREICH: Gültiges PixelFormat
    STANDARDWERT: PixelFormat::RGBA8888
  - NAME: buffer
    TYP: Option<Buffer>
    BESCHREIBUNG: Puffer mit Pixeldaten
    WERTEBEREICH: Gültiger Buffer oder None
    STANDARDWERT: None
  - NAME: damage
    TYP: Option<Rectangle>
    BESCHREIBUNG: Beschädigter Bereich
    WERTEBEREICH: Gültiges Rectangle oder None
    STANDARDWERT: None
  - NAME: opaque
    TYP: bool
    BESCHREIBUNG: Ob die Oberfläche undurchsichtig ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: transform
    TYP: Transform
    BESCHREIBUNG: Transformation der Oberfläche
    WERTEBEREICH: Gültige Transform
    STANDARDWERT: Transform::identity()
  - NAME: scale_factor
    TYP: f32
    BESCHREIBUNG: Skalierungsfaktor der Oberfläche
    WERTEBEREICH: Positive reelle Zahlen
    STANDARDWERT: 1.0
  - NAME: source_rect
    TYP: Option<Rectangle>
    BESCHREIBUNG: Quellrechteck
    WERTEBEREICH: Gültiges Rectangle oder None
    STANDARDWERT: None
  - NAME: destination_rect
    TYP: Option<Rectangle>
    BESCHREIBUNG: Zielrechteck
    WERTEBEREICH: Gültiges Rectangle oder None
    STANDARDWERT: None
INVARIANTEN:
  - id muss eindeutig sein
  - scale_factor muss größer als 0 sein
```

### 6.12 Size

```
ENTITÄT: Size
BESCHREIBUNG: Größe einer Oberfläche oder eines Rechtecks
ATTRIBUTE:
  - NAME: width
    TYP: u32
    BESCHREIBUNG: Breite
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
  - NAME: height
    TYP: u32
    BESCHREIBUNG: Höhe
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
INVARIANTEN:
  - Keine
```

### 6.13 PixelFormat

```
ENTITÄT: PixelFormat
BESCHREIBUNG: Format der Pixel in einer Oberfläche
ATTRIBUTE:
  - NAME: format
    TYP: Enum
    BESCHREIBUNG: Format
    WERTEBEREICH: {
      RGB888,
      RGBA8888,
      BGRA8888,
      ARGB8888,
      RGB565,
      RGBA4444,
      RGBA5551,
      YUV420,
      NV12,
      NV21,
      Custom(u32)
    }
    STANDARDWERT: RGBA8888
INVARIANTEN:
  - Keine
```

### 6.14 Buffer

```
ENTITÄT: Buffer
BESCHREIBUNG: Puffer mit Pixeldaten
ATTRIBUTE:
  - NAME: data
    TYP: Vec<u8>
    BESCHREIBUNG: Pixeldaten
    WERTEBEREICH: Bytes
    STANDARDWERT: Leerer Vec
  - NAME: size
    TYP: Size
    BESCHREIBUNG: Größe des Puffers
    WERTEBEREICH: Gültige Size
    STANDARDWERT: Size { width: 0, height: 0 }
  - NAME: format
    TYP: PixelFormat
    BESCHREIBUNG: Pixelformat des Puffers
    WERTEBEREICH: Gültiges PixelFormat
    STANDARDWERT: PixelFormat::RGBA8888
  - NAME: stride
    TYP: u32
    BESCHREIBUNG: Anzahl der Bytes pro Zeile
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
  - NAME: age
    TYP: u32
    BESCHREIBUNG: Alter des Puffers
    WERTEBEREICH: Nicht-negative Ganzzahlen
    STANDARDWERT: 0
INVARIANTEN:
  - data.len() muss mindestens stride * size.height sein
  - stride muss größer oder gleich size.width * bytes_per_pixel(format) sein
```

### 6.15 RenderTarget

```
ENTITÄT: RenderTarget
BESCHREIBUNG: Ziel für das Rendering
ATTRIBUTE:
  - NAME: target_type
    TYP: Enum
    BESCHREIBUNG: Typ des Ziels
    WERTEBEREICH: {
      Surface(SurfaceId),
      Framebuffer(FramebufferId),
      Texture(TextureId),
      Window(WindowId),
      Offscreen(Size)
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Keine
```

### 6.16 FramebufferId

```
ENTITÄT: FramebufferId
BESCHREIBUNG: Eindeutiger Bezeichner für einen Framebuffer
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.17 TextureId

```
ENTITÄT: TextureId
BESCHREIBUNG: Eindeutiger Bezeichner für eine Textur
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.18 RenderPass

```
ENTITÄT: RenderPass
BESCHREIBUNG: Einzelner Durchgang beim Rendering
ATTRIBUTE:
  - NAME: id
    TYP: RenderPassId
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Gültige RenderPassId
    STANDARDWERT: Keiner
  - NAME: target
    TYP: RenderTarget
    BESCHREIBUNG: Ziel des Renderpasses
    WERTEBEREICH: Gültiges RenderTarget
    STANDARDWERT: Keiner
  - NAME: clear_color
    TYP: Option<Color>
    BESCHREIBUNG: Farbe zum Löschen des Ziels
    WERTEBEREICH: Gültige Color oder None
    STANDARDWERT: None
  - NAME: viewport
    TYP: Option<Rectangle>
    BESCHREIBUNG: Viewport für den Renderpass
    WERTEBEREICH: Gültiges Rectangle oder None
    STANDARDWERT: None
  - NAME: scissor
    TYP: Option<Rectangle>
    BESCHREIBUNG: Scissor-Rechteck für den Renderpass
    WERTEBEREICH: Gültiges Rectangle oder None
    STANDARDWERT: None
  - NAME: blend_mode
    TYP: BlendMode
    BESCHREIBUNG: Blend-Modus für den Renderpass
    WERTEBEREICH: Gültiger BlendMode
    STANDARDWERT: BlendMode::Alpha
  - NAME: shader
    TYP: Option<ShaderId>
    BESCHREIBUNG: Shader für den Renderpass
    WERTEBEREICH: Gültige ShaderId oder None
    STANDARDWERT: None
  - NAME: commands
    TYP: Vec<RenderCommand>
    BESCHREIBUNG: Render-Befehle für den Renderpass
    WERTEBEREICH: Gültige RenderCommand-Werte
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.19 RenderPassId

```
ENTITÄT: RenderPassId
BESCHREIBUNG: Eindeutiger Bezeichner für einen Renderpass
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.20 BlendMode

```
ENTITÄT: BlendMode
BESCHREIBUNG: Blend-Modus für das Rendering
ATTRIBUTE:
  - NAME: mode
    TYP: Enum
    BESCHREIBUNG: Modus
    WERTEBEREICH: {
      None,
      Alpha,
      Add,
      Subtract,
      Multiply,
      Screen,
      Overlay,
      Darken,
      Lighten,
      ColorDodge,
      ColorBurn,
      HardLight,
      SoftLight,
      Difference,
      Exclusion,
      Custom {
        src_factor: BlendFactor,
        dst_factor: BlendFactor,
        equation: BlendEquation
      }
    }
    STANDARDWERT: Alpha
INVARIANTEN:
  - Keine
```

## 7. Verhaltensmodell (Fortsetzung)

### 7.1 Compositor-Initialisierung

```
ZUSTANDSAUTOMAT: CompositorInitialization
BESCHREIBUNG: Prozess der Initialisierung des Compositors
ZUSTÄNDE:
  - NAME: Uninitialized
    BESCHREIBUNG: Compositor ist nicht initialisiert
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: Initializing
    BESCHREIBUNG: Compositor wird initialisiert
    EINTRITTSAKTIONEN: Konfiguration laden
    AUSTRITTSAKTIONEN: Keine
  - NAME: CreatingSceneGraph
    BESCHREIBUNG: Szenegraph wird erstellt
    EINTRITTSAKTIONEN: SceneGraph initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: InitializingRenderManager
    BESCHREIBUNG: RenderManager wird initialisiert
    EINTRITTSAKTIONEN: RenderManager initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: InitializingEffectManager
    BESCHREIBUNG: EffectManager wird initialisiert
    EINTRITTSAKTIONEN: EffectManager initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: InitializingSurfaceManager
    BESCHREIBUNG: SurfaceManager wird initialisiert
    EINTRITTSAKTIONEN: SurfaceManager initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: ConnectingToWindowManager
    BESCHREIBUNG: Verbindung zum WindowManager wird hergestellt
    EINTRITTSAKTIONEN: WindowManager-Verbindung initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Initialized
    BESCHREIBUNG: Compositor ist initialisiert
    EINTRITTSAKTIONEN: Listener benachrichtigen
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler bei der Initialisierung
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Uninitialized
    NACH: Initializing
    EREIGNIS: initialize aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Konfiguration validieren
  - VON: Initializing
    NACH: CreatingSceneGraph
    EREIGNIS: Konfiguration erfolgreich geladen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: Initializing
    NACH: Error
    EREIGNIS: Fehler beim Laden der Konfiguration
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: CreatingSceneGraph
    NACH: InitializingRenderManager
    EREIGNIS: Szenegraph erfolgreich erstellt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: CreatingSceneGraph
    NACH: Error
    EREIGNIS: Fehler bei der Erstellung des Szenegraphen
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: InitializingRenderManager
    NACH: InitializingEffectManager
    EREIGNIS: RenderManager erfolgreich initialisiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: InitializingRenderManager
    NACH: Error
    EREIGNIS: Fehler bei der Initialisierung des RenderManagers
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: InitializingEffectManager
    NACH: InitializingSurfaceManager
    EREIGNIS: EffectManager erfolgreich initialisiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: InitializingEffectManager
    NACH: Error
    EREIGNIS: Fehler bei der Initialisierung des EffectManagers
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: InitializingSurfaceManager
    NACH: ConnectingToWindowManager
    EREIGNIS: SurfaceManager erfolgreich initialisiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: InitializingSurfaceManager
    NACH: Error
    EREIGNIS: Fehler bei der Initialisierung des SurfaceManagers
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: ConnectingToWindowManager
    NACH: Initialized
    EREIGNIS: Verbindung zum WindowManager erfolgreich hergestellt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ConnectingToWindowManager
    NACH: Error
    EREIGNIS: Fehler bei der Verbindung zum WindowManager
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
INITIALZUSTAND: Uninitialized
ENDZUSTÄNDE: [Initialized, Error]
```

### 7.2 Rendering-Prozess

```
ZUSTANDSAUTOMAT: RenderingProcess
BESCHREIBUNG: Prozess des Renderings eines Frames
ZUSTÄNDE:
  - NAME: Idle
    BESCHREIBUNG: Kein Rendering aktiv
    EINTRITTSAKTIONEN: Keine
    AUSTRITTSAKTIONEN: Keine
  - NAME: PreparingScene
    BESCHREIBUNG: Szene wird vorbereitet
    EINTRITTSAKTIONEN: Szenegraph aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: CollectingDamage
    BESCHREIBUNG: Beschädigte Bereiche werden gesammelt
    EINTRITTSAKTIONEN: DamageTracker initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: CreatingRenderPasses
    BESCHREIBUNG: Renderpässe werden erstellt
    EINTRITTSAKTIONEN: RenderPass-Liste initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: ApplyingEffects
    BESCHREIBUNG: Effekte werden angewendet
    EINTRITTSAKTIONEN: EffectChain initialisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: ExecutingRenderPasses
    BESCHREIBUNG: Renderpässe werden ausgeführt
    EINTRITTSAKTIONEN: RenderManager aktivieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Presenting
    BESCHREIBUNG: Frame wird präsentiert
    EINTRITTSAKTIONEN: Vsync abwarten
    AUSTRITTSAKTIONEN: Keine
  - NAME: Completed
    BESCHREIBUNG: Rendering abgeschlossen
    EINTRITTSAKTIONEN: Statistiken aktualisieren
    AUSTRITTSAKTIONEN: Keine
  - NAME: Error
    BESCHREIBUNG: Fehler beim Rendering
    EINTRITTSAKTIONEN: Fehler protokollieren
    AUSTRITTSAKTIONEN: Keine
ÜBERGÄNGE:
  - VON: Idle
    NACH: PreparingScene
    EREIGNIS: render_frame aufgerufen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: PreparingScene
    NACH: CollectingDamage
    EREIGNIS: Szene erfolgreich vorbereitet
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: PreparingScene
    NACH: Error
    EREIGNIS: Fehler bei der Vorbereitung der Szene
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: CollectingDamage
    NACH: CreatingRenderPasses
    EREIGNIS: Beschädigte Bereiche erfolgreich gesammelt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: CollectingDamage
    NACH: Error
    EREIGNIS: Fehler beim Sammeln der beschädigten Bereiche
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: CreatingRenderPasses
    NACH: ApplyingEffects
    EREIGNIS: Renderpässe erfolgreich erstellt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: CreatingRenderPasses
    NACH: Error
    EREIGNIS: Fehler bei der Erstellung der Renderpässe
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: ApplyingEffects
    NACH: ExecutingRenderPasses
    EREIGNIS: Effekte erfolgreich angewendet
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ApplyingEffects
    NACH: Error
    EREIGNIS: Fehler bei der Anwendung der Effekte
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: ExecutingRenderPasses
    NACH: Presenting
    EREIGNIS: Renderpässe erfolgreich ausgeführt
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: ExecutingRenderPasses
    NACH: Error
    EREIGNIS: Fehler bei der Ausführung der Renderpässe
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: Presenting
    NACH: Completed
    EREIGNIS: Frame erfolgreich präsentiert
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: Presenting
    NACH: Error
    EREIGNIS: Fehler bei der Präsentation des Frames
    BEDINGUNG: Keine
    AKTIONEN: CompositorError erstellen
  - VON: Completed
    NACH: Idle
    EREIGNIS: Rendering abgeschlossen
    BEDINGUNG: Keine
    AKTIONEN: Keine
  - VON: Error
    NACH: Idle
    EREIGNIS: Fehler behandelt
    BEDINGUNG: Keine
    AKTIONEN: Keine
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

### 8.2 Modulspezifische Fehlertypen

```
ENTITÄT: CompositorError
BESCHREIBUNG: Fehler im Compositor-Modul
ATTRIBUTE:
  - NAME: variant
    TYP: Enum
    BESCHREIBUNG: Fehlervariante
    WERTEBEREICH: {
      RenderError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      SceneGraphError { node_id: Option<SceneNodeId>, message: String },
      SurfaceError { surface_id: Option<SurfaceId>, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      EffectError { effect_id: Option<EffectId>, message: String },
      ShaderError { shader_id: Option<ShaderId>, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      TextureError { texture_id: Option<TextureId>, message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      WindowManagerError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      BackendError { message: String, source: Option<Box<dyn std::error::Error + Send + Sync + 'static>> },
      ResourceError { resource_type: String, resource_id: String, message: String },
      ConfigurationError { message: String },
      InternalError { message: String }
    }
    STANDARDWERT: Keiner
```

## 9. Leistungsanforderungen

### 9.1 Allgemeine Leistungsanforderungen

1. Der Compositor MUSS effizient mit Ressourcen umgehen.
2. Der Compositor MUSS eine geringe Latenz haben.
3. Der Compositor MUSS skalierbar sein.

### 9.2 Spezifische Leistungsanforderungen

1. Der Compositor MUSS eine Framerate von mindestens 60 FPS auf Standard-Hardware erreichen.
2. Die Latenz zwischen Fenstererstellung und Anzeige MUSS unter 16ms liegen.
3. Die Latenz zwischen Fensteraktualisierung und Anzeige MUSS unter 16ms liegen.
4. Der Compositor MUSS mit mindestens 100 gleichzeitigen Fenstern umgehen können.
5. Der Compositor MUSS mit mindestens 10 gleichzeitigen Effekten umgehen können.
6. Der Compositor MUSS mit mindestens 4 gleichzeitigen Monitoren umgehen können.
7. Der Compositor DARF nicht mehr als 5% CPU-Auslastung im Leerlauf verursachen.
8. Der Compositor DARF nicht mehr als 100MB Speicher im Leerlauf verbrauchen.

## 10. Sicherheitsanforderungen

### 10.1 Allgemeine Sicherheitsanforderungen

1. Der Compositor MUSS memory-safe sein.
2. Der Compositor MUSS thread-safe sein.
3. Der Compositor MUSS robust gegen Fehleingaben sein.

### 10.2 Spezifische Sicherheitsanforderungen

1. Der Compositor MUSS Eingaben validieren, um Injection-Angriffe zu verhindern.
2. Der Compositor MUSS Zugriffskontrollen für Fensteroperationen implementieren.
3. Der Compositor MUSS sichere Standardwerte verwenden.
4. Der Compositor MUSS Ressourcenlimits implementieren, um Denial-of-Service-Angriffe zu verhindern.
5. Der Compositor MUSS verhindern, dass Fenster auf geschützte Bereiche des Bildschirms zugreifen.
6. Der Compositor MUSS verhindern, dass Fenster Inhalte anderer Fenster auslesen.

## 11. Testkriterien

### 11.1 Allgemeine Testkriterien

1. Jede Komponente MUSS Einheitstests haben.
2. Jede öffentliche Funktion MUSS getestet sein.
3. Jeder Fehlerfall MUSS getestet sein.

### 11.2 Spezifische Testkriterien

1. Der Compositor MUSS mit verschiedenen Fenstertypen getestet sein.
2. Der Compositor MUSS mit verschiedenen Effekten getestet sein.
3. Der Compositor MUSS mit verschiedenen Oberflächenformaten getestet sein.
4. Der Compositor MUSS mit verschiedenen Renderzielen getestet sein.
5. Der Compositor MUSS mit verschiedenen Monitor-Konfigurationen getestet sein.
6. Der Compositor MUSS mit verschiedenen Fehlerszenarien getestet sein.
7. Der Compositor MUSS mit verschiedenen Leistungsszenarien getestet sein.
8. Der Compositor MUSS mit verschiedenen Backend-Konfigurationen getestet sein.

## 12. Anhänge

### 12.1 Referenzierte Dokumente

1. SPEC-ROOT-v1.0.0: NovaDE Spezifikationswurzel
2. SPEC-LAYER-CORE-v1.0.0: Spezifikation der Kernschicht
3. SPEC-LAYER-DOMAIN-v1.0.0: Spezifikation der Domänenschicht
4. SPEC-MODULE-SYSTEM-WINDOWMANAGER-v1.0.0: Spezifikation des Fenstermanager-Moduls

### 12.2 Externe Abhängigkeiten

1. `skia`: Für das 2D-Rendering
2. `vulkan`: Für das 3D-Rendering
3. `glsl`: Für Shader
4. `egl`: Für OpenGL ES
5. `wayland-server`: Für die Wayland-Integration
