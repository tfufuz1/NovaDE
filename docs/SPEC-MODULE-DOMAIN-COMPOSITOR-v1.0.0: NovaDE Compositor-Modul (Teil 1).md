# SPEC-MODULE-DOMAIN-COMPOSITOR-v1.0.0: NovaDE Compositor-Modul (Teil 1)

```
SPEZIFIKATION: SPEC-MODULE-DOMAIN-COMPOSITOR-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0, SPEC-LAYER-CORE-v1.0.0, SPEC-LAYER-DOMAIN-v1.0.0, SPEC-MODULE-SYSTEM-WINDOWMANAGER-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Diese Spezifikation definiert das Compositor-Modul (`domain::compositor`) der NovaDE-Domänenschicht. Das Modul stellt die grundlegende Infrastruktur für die Komposition von Fenstern und visuellen Effekten bereit und definiert die Mechanismen zur Fensterdarstellung, Überblendung, Schatten, Transparenz und Animation. Der Geltungsbereich umfasst alle Komponenten und Schnittstellen des Compositor-Moduls sowie deren Interaktionen mit anderen Modulen.

## 2. Definitionen

### 2.1 Allgemeine Begriffe

- **Compositor**: Komponente, die für die Zusammenstellung des finalen Bildschirminhalts zuständig ist
- **Komposition**: Prozess der Zusammenstellung mehrerer visueller Elemente zu einem Gesamtbild
- **Szenegraph**: Hierarchische Struktur zur Organisation visueller Elemente
- **Rendering**: Prozess der Erzeugung eines Bildes aus einer Szene
- **Textur**: Bilddaten, die auf Oberflächen angewendet werden
- **Shader**: Programm zur Berechnung von Farbwerten und Effekten
- **Framebuffer**: Speicherbereich für Bilddaten
- **Vsync**: Synchronisation der Bildausgabe mit der Bildwiederholrate des Monitors
- **Tearing**: Visueller Effekt, bei dem Teile verschiedener Frames gleichzeitig angezeigt werden

### 2.2 Modulspezifische Begriffe

- **CompositorManager**: Zentrale Komponente für die Verwaltung des Compositors
- **SceneGraph**: Komponente zur Verwaltung der Szene
- **SceneNode**: Knoten im Szenegraph
- **RenderTarget**: Ziel für das Rendering
- **RenderPass**: Einzelner Durchgang beim Rendering
- **Effect**: Visueller Effekt
- **EffectChain**: Kette von Effekten
- **SurfaceManager**: Komponente zur Verwaltung von Oberflächen
- **Surface**: Oberfläche für die Darstellung von Inhalten
- **Damage**: Bereich, der neu gerendert werden muss

## 3. Anforderungen

### 3.1 Funktionale Anforderungen

1. Das Modul MUSS Mechanismen zur Komposition von Fenstern bereitstellen.
2. Das Modul MUSS Mechanismen zur Verwaltung eines Szenegraphen bereitstellen.
3. Das Modul MUSS Mechanismen zur Anwendung visueller Effekte bereitstellen.
4. Das Modul MUSS Mechanismen zur Verwaltung von Oberflächen bereitstellen.
5. Das Modul MUSS Mechanismen zur Schatten- und Transparenzdarstellung bereitstellen.
6. Das Modul MUSS Mechanismen zur Animation von Fenstern und Effekten bereitstellen.
7. Das Modul MUSS Mechanismen zur Synchronisation mit der Bildwiederholrate bereitstellen.
8. Das Modul MUSS Mechanismen zur Beschleunigung durch Hardware bereitstellen.
9. Das Modul MUSS Mechanismen zur Behandlung von Schäden (Damage) bereitstellen.
10. Das Modul MUSS Mechanismen zur Integration mit dem Fenstermanager bereitstellen.
11. Das Modul MUSS Mechanismen zur Integration mit dem Theming-System bereitstellen.
12. Das Modul MUSS Mechanismen zur Unterstützung von Multi-Monitor-Setups bereitstellen.

### 3.2 Nicht-funktionale Anforderungen

1. Das Modul MUSS effizient mit Ressourcen umgehen.
2. Das Modul MUSS thread-safe sein.
3. Das Modul MUSS eine klare und konsistente API bereitstellen.
4. Das Modul MUSS gut dokumentiert sein.
5. Das Modul MUSS leicht erweiterbar sein.
6. Das Modul MUSS robust gegen Fehleingaben sein.
7. Das Modul MUSS minimale externe Abhängigkeiten haben.
8. Das Modul MUSS eine hohe Performance bieten.
9. Das Modul MUSS eine geringe Latenz bei der Darstellung bieten.
10. Das Modul MUSS eine hohe Zuverlässigkeit bieten.

## 4. Architektur

### 4.1 Komponentenstruktur

Das Compositor-Modul besteht aus den folgenden Komponenten:

1. **CompositorManager** (`compositor_manager.rs`): Zentrale Komponente für die Verwaltung des Compositors
2. **SceneGraph** (`scene_graph.rs`): Komponente zur Verwaltung der Szene
3. **SceneNode** (`scene_node.rs`): Komponente für Knoten im Szenegraph
4. **RenderManager** (`render_manager.rs`): Komponente für das Rendering
5. **RenderTarget** (`render_target.rs`): Komponente für Renderziele
6. **RenderPass** (`render_pass.rs`): Komponente für Renderdurchgänge
7. **EffectManager** (`effect_manager.rs`): Komponente zur Verwaltung von Effekten
8. **Effect** (`effect.rs`): Komponente für visuelle Effekte
9. **EffectChain** (`effect_chain.rs`): Komponente für Effektketten
10. **SurfaceManager** (`surface_manager.rs`): Komponente zur Verwaltung von Oberflächen
11. **Surface** (`surface.rs`): Komponente für Oberflächen
12. **DamageTracker** (`damage_tracker.rs`): Komponente zur Verfolgung von Schäden
13. **AnimationManager** (`animation_manager.rs`): Komponente zur Verwaltung von Animationen
14. **ShaderManager** (`shader_manager.rs`): Komponente zur Verwaltung von Shadern
15. **TextureManager** (`texture_manager.rs`): Komponente zur Verwaltung von Texturen

### 4.2 Abhängigkeiten

Das Compositor-Modul hat folgende Abhängigkeiten:

1. **Interne Abhängigkeiten**:
   - `core::errors`: Für die Fehlerbehandlung
   - `core::config`: Für die Konfiguration
   - `core::logging`: Für das Logging
   - `domain::theming`: Für das Theming
   - `system::windowmanager`: Für die Fensterverwaltung
   - `system::display`: Für die Anzeige

2. **Externe Abhängigkeiten**:
   - `skia`: Für das 2D-Rendering
   - `vulkan`: Für das 3D-Rendering
   - `glsl`: Für Shader
   - `egl`: Für OpenGL ES
   - `wayland-server`: Für die Wayland-Integration

## 5. Schnittstellen

### 5.1 CompositorManager

```
SCHNITTSTELLE: domain::compositor::CompositorManager
BESCHREIBUNG: Zentrale Komponente für die Verwaltung des Compositors
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue CompositorManager-Instanz
    PARAMETER:
      - NAME: config
        TYP: CompositorConfig
        BESCHREIBUNG: Konfiguration für den CompositorManager
        EINSCHRÄNKUNGEN: Muss eine gültige CompositorConfig sein
    RÜCKGABETYP: Result<CompositorManager, CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des CompositorManagers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue CompositorManager-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des CompositorManagers auftritt
  
  - NAME: initialize
    BESCHREIBUNG: Initialisiert den CompositorManager
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn ein Fehler bei der Initialisierung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der CompositorManager wird initialisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Initialisierung auftritt
  
  - NAME: shutdown
    BESCHREIBUNG: Fährt den CompositorManager herunter
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn ein Fehler beim Herunterfahren auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der CompositorManager wird heruntergefahren
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Herunterfahren auftritt
  
  - NAME: get_scene_graph
    BESCHREIBUNG: Gibt den Szenegraphen zurück
    PARAMETER: Keine
    RÜCKGABETYP: &SceneGraph
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Szenegraph wird zurückgegeben
  
  - NAME: get_render_manager
    BESCHREIBUNG: Gibt den RenderManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &RenderManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der RenderManager wird zurückgegeben
  
  - NAME: get_effect_manager
    BESCHREIBUNG: Gibt den EffectManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &EffectManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der EffectManager wird zurückgegeben
  
  - NAME: get_surface_manager
    BESCHREIBUNG: Gibt den SurfaceManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &SurfaceManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der SurfaceManager wird zurückgegeben
  
  - NAME: get_animation_manager
    BESCHREIBUNG: Gibt den AnimationManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &AnimationManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der AnimationManager wird zurückgegeben
  
  - NAME: get_shader_manager
    BESCHREIBUNG: Gibt den ShaderManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &ShaderManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der ShaderManager wird zurückgegeben
  
  - NAME: get_texture_manager
    BESCHREIBUNG: Gibt den TextureManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &TextureManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der TextureManager wird zurückgegeben
  
  - NAME: render_frame
    BESCHREIBUNG: Rendert einen Frame
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn ein Fehler beim Rendern auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein Frame wird gerendert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Rendern auftritt
  
  - NAME: add_window
    BESCHREIBUNG: Fügt ein Fenster zum Compositor hinzu
    PARAMETER:
      - NAME: window_id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
    RÜCKGABETYP: Result<SceneNodeId, CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn ein Fehler beim Hinzufügen des Fensters auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Fenster wird zum Compositor hinzugefügt
      - Eine SceneNodeId wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Hinzufügen des Fensters auftritt
  
  - NAME: remove_window
    BESCHREIBUNG: Entfernt ein Fenster aus dem Compositor
    PARAMETER:
      - NAME: window_id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn ein Fehler beim Entfernen des Fensters auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Fenster wird aus dem Compositor entfernt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Entfernen des Fensters auftritt
  
  - NAME: update_window
    BESCHREIBUNG: Aktualisiert ein Fenster im Compositor
    PARAMETER:
      - NAME: window_id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn ein Fehler beim Aktualisieren des Fensters auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Fenster wird im Compositor aktualisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Aktualisieren des Fensters auftritt
  
  - NAME: apply_effect
    BESCHREIBUNG: Wendet einen Effekt auf ein Fenster an
    PARAMETER:
      - NAME: window_id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
      - NAME: effect_id
        TYP: EffectId
        BESCHREIBUNG: ID des Effekts
        EINSCHRÄNKUNGEN: Muss eine gültige EffectId sein
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn ein Fehler beim Anwenden des Effekts auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Effekt wird auf das Fenster angewendet
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Anwenden des Effekts auftritt
  
  - NAME: remove_effect
    BESCHREIBUNG: Entfernt einen Effekt von einem Fenster
    PARAMETER:
      - NAME: window_id
        TYP: WindowId
        BESCHREIBUNG: ID des Fensters
        EINSCHRÄNKUNGEN: Muss eine gültige WindowId sein
      - NAME: effect_id
        TYP: EffectId
        BESCHREIBUNG: ID des Effekts
        EINSCHRÄNKUNGEN: Muss eine gültige EffectId sein
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn ein Fehler beim Entfernen des Effekts auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Effekt wird vom Fenster entfernt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Entfernen des Effekts auftritt
  
  - NAME: register_compositor_event_listener
    BESCHREIBUNG: Registriert einen Listener für Compositor-Ereignisse
    PARAMETER:
      - NAME: listener
        TYP: Box<dyn Fn(CompositorEvent) + Send + Sync + 'static>
        BESCHREIBUNG: Listener-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: ListenerId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird registriert und eine ListenerId wird zurückgegeben
  
  - NAME: unregister_compositor_event_listener
    BESCHREIBUNG: Entfernt einen Listener für Compositor-Ereignisse
    PARAMETER:
      - NAME: id
        TYP: ListenerId
        BESCHREIBUNG: ID des Listeners
        EINSCHRÄNKUNGEN: Muss eine gültige ListenerId sein
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn der Listener nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Listener nicht gefunden wird
```

### 5.2 SceneGraph

```
SCHNITTSTELLE: domain::compositor::SceneGraph
BESCHREIBUNG: Komponente zur Verwaltung der Szene
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue SceneGraph-Instanz
    PARAMETER: Keine
    RÜCKGABETYP: SceneGraph
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue SceneGraph-Instanz wird erstellt
  
  - NAME: get_root_node
    BESCHREIBUNG: Gibt den Wurzelknoten zurück
    PARAMETER: Keine
    RÜCKGABETYP: &SceneNode
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Wurzelknoten wird zurückgegeben
  
  - NAME: create_node
    BESCHREIBUNG: Erstellt einen neuen Knoten
    PARAMETER:
      - NAME: parent_id
        TYP: Option<SceneNodeId>
        BESCHREIBUNG: ID des Elternknotens
        EINSCHRÄNKUNGEN: Wenn vorhanden, muss eine gültige SceneNodeId sein
      - NAME: node_type
        TYP: SceneNodeType
        BESCHREIBUNG: Typ des Knotens
        EINSCHRÄNKUNGEN: Muss ein gültiger SceneNodeType sein
    RÜCKGABETYP: Result<SceneNodeId, CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des Knotens auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ein neuer Knoten wird erstellt und eine SceneNodeId wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des Knotens auftritt
  
  - NAME: get_node
    BESCHREIBUNG: Gibt einen Knoten zurück
    PARAMETER:
      - NAME: id
        TYP: SceneNodeId
        BESCHREIBUNG: ID des Knotens
        EINSCHRÄNKUNGEN: Muss eine gültige SceneNodeId sein
    RÜCKGABETYP: Result<&SceneNode, CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn der Knoten nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Knoten wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn der Knoten nicht gefunden wird
  
  - NAME: get_node_mut
    BESCHREIBUNG: Gibt einen veränderbaren Knoten zurück
    PARAMETER:
      - NAME: id
        TYP: SceneNodeId
        BESCHREIBUNG: ID des Knotens
        EINSCHRÄNKUNGEN: Muss eine gültige SceneNodeId sein
    RÜCKGABETYP: Result<&mut SceneNode, CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn der Knoten nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Knoten wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn der Knoten nicht gefunden wird
  
  - NAME: remove_node
    BESCHREIBUNG: Entfernt einen Knoten
    PARAMETER:
      - NAME: id
        TYP: SceneNodeId
        BESCHREIBUNG: ID des Knotens
        EINSCHRÄNKUNGEN: Muss eine gültige SceneNodeId sein
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn der Knoten nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Knoten wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Knoten nicht gefunden wird
  
  - NAME: move_node
    BESCHREIBUNG: Verschiebt einen Knoten zu einem neuen Elternknoten
    PARAMETER:
      - NAME: id
        TYP: SceneNodeId
        BESCHREIBUNG: ID des Knotens
        EINSCHRÄNKUNGEN: Muss eine gültige SceneNodeId sein
      - NAME: new_parent_id
        TYP: Option<SceneNodeId>
        BESCHREIBUNG: ID des neuen Elternknotens
        EINSCHRÄNKUNGEN: Wenn vorhanden, muss eine gültige SceneNodeId sein
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn der Knoten oder der neue Elternknoten nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Knoten wird zum neuen Elternknoten verschoben
      - Ein Fehler wird zurückgegeben, wenn der Knoten oder der neue Elternknoten nicht gefunden wird
  
  - NAME: get_children
    BESCHREIBUNG: Gibt die Kinder eines Knotens zurück
    PARAMETER:
      - NAME: id
        TYP: SceneNodeId
        BESCHREIBUNG: ID des Knotens
        EINSCHRÄNKUNGEN: Muss eine gültige SceneNodeId sein
    RÜCKGABETYP: Result<Vec<SceneNodeId>, CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn der Knoten nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Kinder des Knotens werden zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn der Knoten nicht gefunden wird
  
  - NAME: get_parent
    BESCHREIBUNG: Gibt den Elternknoten eines Knotens zurück
    PARAMETER:
      - NAME: id
        TYP: SceneNodeId
        BESCHREIBUNG: ID des Knotens
        EINSCHRÄNKUNGEN: Muss eine gültige SceneNodeId sein
    RÜCKGABETYP: Result<Option<SceneNodeId>, CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn der Knoten nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Elternknoten des Knotens wird zurückgegeben, wenn vorhanden
      - None wird zurückgegeben, wenn der Knoten keinen Elternknoten hat
      - Ein Fehler wird zurückgegeben, wenn der Knoten nicht gefunden wird
  
  - NAME: traverse
    BESCHREIBUNG: Durchläuft den Szenegraphen
    PARAMETER:
      - NAME: visitor
        TYP: &mut dyn FnMut(&SceneNode) -> bool
        BESCHREIBUNG: Visitor-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: ()
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Szenegraph wird durchlaufen und die Visitor-Funktion wird für jeden Knoten aufgerufen
  
  - NAME: traverse_from
    BESCHREIBUNG: Durchläuft den Szenegraphen von einem bestimmten Knoten aus
    PARAMETER:
      - NAME: id
        TYP: SceneNodeId
        BESCHREIBUNG: ID des Startknotens
        EINSCHRÄNKUNGEN: Muss eine gültige SceneNodeId sein
      - NAME: visitor
        TYP: &mut dyn FnMut(&SceneNode) -> bool
        BESCHREIBUNG: Visitor-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), CompositorError>
    FEHLER:
      - TYP: CompositorError
        BEDINGUNG: Wenn der Startknoten nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Szenegraph wird von dem angegebenen Knoten aus durchlaufen und die Visitor-Funktion wird für jeden Knoten aufgerufen
      - Ein Fehler wird zurückgegeben, wenn der Startknoten nicht gefunden wird
```

## 6. Datenmodell (Teil 1)

### 6.1 SceneNodeId

```
ENTITÄT: SceneNodeId
BESCHREIBUNG: Eindeutiger Bezeichner für einen Knoten im Szenegraph
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.2 SceneNodeType

```
ENTITÄT: SceneNodeType
BESCHREIBUNG: Typ eines Knotens im Szenegraph
ATTRIBUTE:
  - NAME: node_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      Root,
      Window,
      Group,
      Surface,
      Decoration,
      Effect,
      Custom(String)
    }
    STANDARDWERT: Group
INVARIANTEN:
  - Bei Custom darf die Zeichenkette nicht leer sein
```

### 6.3 SceneNode

```
ENTITÄT: SceneNode
BESCHREIBUNG: Knoten im Szenegraph
ATTRIBUTE:
  - NAME: id
    TYP: SceneNodeId
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Gültige SceneNodeId
    STANDARDWERT: Keiner
  - NAME: node_type
    TYP: SceneNodeType
    BESCHREIBUNG: Typ des Knotens
    WERTEBEREICH: Gültiger SceneNodeType
    STANDARDWERT: SceneNodeType::Group
  - NAME: transform
    TYP: Transform
    BESCHREIBUNG: Transformation des Knotens
    WERTEBEREICH: Gültige Transform
    STANDARDWERT: Transform::identity()
  - NAME: visible
    TYP: bool
    BESCHREIBUNG: Ob der Knoten sichtbar ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: opacity
    TYP: f32
    BESCHREIBUNG: Deckkraft des Knotens
    WERTEBEREICH: [0.0, 1.0]
    STANDARDWERT: 1.0
  - NAME: clip
    TYP: Option<Rectangle>
    BESCHREIBUNG: Clipping-Rechteck
    WERTEBEREICH: Gültiges Rectangle oder None
    STANDARDWERT: None
  - NAME: data
    TYP: Option<Box<dyn Any + Send + Sync>>
    BESCHREIBUNG: Benutzerdefinierte Daten
    WERTEBEREICH: Beliebige Daten oder None
    STANDARDWERT: None
INVARIANTEN:
  - id muss eindeutig sein
  - opacity muss im Bereich [0.0, 1.0] liegen
```

### 6.4 Transform

```
ENTITÄT: Transform
BESCHREIBUNG: Transformation eines Knotens
ATTRIBUTE:
  - NAME: matrix
    TYP: Matrix4x4
    BESCHREIBUNG: Transformationsmatrix
    WERTEBEREICH: Gültige Matrix4x4
    STANDARDWERT: Identitätsmatrix
INVARIANTEN:
  - Keine
```

### 6.5 Matrix4x4

```
ENTITÄT: Matrix4x4
BESCHREIBUNG: 4x4-Matrix für Transformationen
ATTRIBUTE:
  - NAME: values
    TYP: [f32; 16]
    BESCHREIBUNG: Matrixwerte in Zeilenreihenfolge
    WERTEBEREICH: Reelle Zahlen
    STANDARDWERT: [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]
INVARIANTEN:
  - Keine
```

### 6.6 EffectId

```
ENTITÄT: EffectId
BESCHREIBUNG: Eindeutiger Bezeichner für einen Effekt
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.7 Effect

```
ENTITÄT: Effect
BESCHREIBUNG: Visueller Effekt
ATTRIBUTE:
  - NAME: id
    TYP: EffectId
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Gültige EffectId
    STANDARDWERT: Keiner
  - NAME: effect_type
    TYP: EffectType
    BESCHREIBUNG: Typ des Effekts
    WERTEBEREICH: Gültiger EffectType
    STANDARDWERT: EffectType::None
  - NAME: parameters
    TYP: HashMap<String, EffectParameter>
    BESCHREIBUNG: Parameter des Effekts
    WERTEBEREICH: Gültige String-EffectParameter-Paare
    STANDARDWERT: Leere HashMap
  - NAME: enabled
    TYP: bool
    BESCHREIBUNG: Ob der Effekt aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.8 EffectType

```
ENTITÄT: EffectType
BESCHREIBUNG: Typ eines Effekts
ATTRIBUTE:
  - NAME: effect_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      None,
      Blur,
      Shadow,
      Glow,
      ColorMatrix,
      Desaturate,
      Brightness,
      Contrast,
      Opacity,
      Reflection,
      Custom(String)
    }
    STANDARDWERT: None
INVARIANTEN:
  - Bei Custom darf die Zeichenkette nicht leer sein
```

### 6.9 EffectParameter

```
ENTITÄT: EffectParameter
BESCHREIBUNG: Parameter eines Effekts
ATTRIBUTE:
  - NAME: parameter_type
    TYP: Enum
    BESCHREIBUNG: Typ des Parameters
    WERTEBEREICH: {
      Float(f32),
      Int(i32),
      Bool(bool),
      Color(Color),
      Vector2(f32, f32),
      Vector3(f32, f32, f32),
      Vector4(f32, f32, f32, f32),
      Matrix(Matrix4x4),
      String(String)
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Bei String darf die Zeichenkette nicht leer sein
```

### 6.10 SurfaceId

```
ENTITÄT: SurfaceId
BESCHREIBUNG: Eindeutiger Bezeichner für eine Oberfläche
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```
