# SPEC-MODULE-SYSTEM-INPUT-v1.0.0: NovaDE Eingabemanager-Modul (Teil 1)

```
SPEZIFIKATION: SPEC-MODULE-SYSTEM-INPUT-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0, SPEC-LAYER-CORE-v1.0.0, SPEC-LAYER-SYSTEM-v1.0.0, SPEC-MODULE-SYSTEM-WINDOWMANAGER-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Diese Spezifikation definiert das Eingabemanager-Modul (`system::input`) der NovaDE-Systemschicht. Das Modul stellt die grundlegende Infrastruktur für die Verarbeitung von Benutzereingaben bereit und definiert die Mechanismen zur Erfassung, Verarbeitung und Weiterleitung von Tastatur-, Maus-, Touch- und anderen Eingabeereignissen. Der Geltungsbereich umfasst alle Komponenten und Schnittstellen des Eingabemanager-Moduls sowie deren Interaktionen mit anderen Modulen.

## 2. Definitionen

### 2.1 Allgemeine Begriffe

- **Eingabeereignis**: Ereignis, das durch eine Benutzereingabe ausgelöst wird
- **Eingabegerät**: Physisches Gerät zur Eingabe von Daten
- **Tastatur**: Eingabegerät mit Tasten
- **Maus**: Zeigegerät mit Tasten
- **Touchscreen**: Berührungsempfindliches Eingabegerät
- **Geste**: Bewegungsmuster auf einem Touchscreen
- **Hotkey**: Tastenkombination zur Auslösung einer Aktion
- **Fokus**: Zustand eines Fensters, das Eingaben empfängt
- **Grab**: Exklusiver Zugriff auf ein Eingabegerät

### 2.2 Modulspezifische Begriffe

- **InputManager**: Zentrale Komponente für die Verwaltung von Eingaben
- **InputDevice**: Repräsentation eines Eingabegeräts
- **InputEvent**: Repräsentation eines Eingabeereignisses
- **KeyboardManager**: Komponente für die Verwaltung von Tastatureingaben
- **MouseManager**: Komponente für die Verwaltung von Mauseingaben
- **TouchManager**: Komponente für die Verwaltung von Toucheingaben
- **GestureRecognizer**: Komponente zur Erkennung von Gesten
- **HotkeyManager**: Komponente für die Verwaltung von Hotkeys
- **InputGrab**: Komponente für den exklusiven Zugriff auf Eingabegeräte
- **InputFocus**: Komponente für die Verwaltung des Eingabefokus

## 3. Anforderungen

### 3.1 Funktionale Anforderungen

1. Das Modul MUSS Mechanismen zur Erfassung von Tastatureingaben bereitstellen.
2. Das Modul MUSS Mechanismen zur Erfassung von Mauseingaben bereitstellen.
3. Das Modul MUSS Mechanismen zur Erfassung von Toucheingaben bereitstellen.
4. Das Modul MUSS Mechanismen zur Erkennung von Gesten bereitstellen.
5. Das Modul MUSS Mechanismen zur Verwaltung von Hotkeys bereitstellen.
6. Das Modul MUSS Mechanismen zur Verwaltung des Eingabefokus bereitstellen.
7. Das Modul MUSS Mechanismen zur Weiterleitung von Eingabeereignissen bereitstellen.
8. Das Modul MUSS Mechanismen zur Filterung von Eingabeereignissen bereitstellen.
9. Das Modul MUSS Mechanismen zur Transformation von Eingabeereignissen bereitstellen.
10. Das Modul MUSS Mechanismen zur Integration mit dem Fenstermanager bereitstellen.
11. Das Modul MUSS Mechanismen zur Unterstützung von Barrierefreiheit bereitstellen.
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
9. Das Modul MUSS eine geringe Latenz bei der Eingabeverarbeitung bieten.
10. Das Modul MUSS eine hohe Zuverlässigkeit bieten.

## 4. Architektur

### 4.1 Komponentenstruktur

Das Eingabemanager-Modul besteht aus den folgenden Komponenten:

1. **InputManager** (`input_manager.rs`): Zentrale Komponente für die Verwaltung von Eingaben
2. **InputDevice** (`input_device.rs`): Repräsentation eines Eingabegeräts
3. **InputEvent** (`input_event.rs`): Repräsentation eines Eingabeereignisses
4. **KeyboardManager** (`keyboard_manager.rs`): Komponente für die Verwaltung von Tastatureingaben
5. **MouseManager** (`mouse_manager.rs`): Komponente für die Verwaltung von Mauseingaben
6. **TouchManager** (`touch_manager.rs`): Komponente für die Verwaltung von Toucheingaben
7. **GestureRecognizer** (`gesture_recognizer.rs`): Komponente zur Erkennung von Gesten
8. **HotkeyManager** (`hotkey_manager.rs`): Komponente für die Verwaltung von Hotkeys
9. **InputGrab** (`input_grab.rs`): Komponente für den exklusiven Zugriff auf Eingabegeräte
10. **InputFocus** (`input_focus.rs`): Komponente für die Verwaltung des Eingabefokus
11. **InputFilter** (`input_filter.rs`): Komponente zur Filterung von Eingabeereignissen
12. **InputTransformer** (`input_transformer.rs`): Komponente zur Transformation von Eingabeereignissen
13. **AccessibilityManager** (`accessibility_manager.rs`): Komponente für Barrierefreiheit
14. **InputBackend** (`input_backend.rs`): Abstraktion für Backend-spezifische Eingabeverarbeitung

### 4.2 Abhängigkeiten

Das Eingabemanager-Modul hat folgende Abhängigkeiten:

1. **Interne Abhängigkeiten**:
   - `core::errors`: Für die Fehlerbehandlung
   - `core::config`: Für die Konfiguration
   - `core::logging`: Für das Logging
   - `system::windowmanager`: Für die Fensterverwaltung
   - `system::display`: Für die Anzeige

2. **Externe Abhängigkeiten**:
   - `input-linux`: Für die Linux-Eingabeunterstützung
   - `xkbcommon`: Für die Tastaturlayout-Unterstützung
   - `libinput`: Für die Eingabegeräte-Unterstützung
   - `evdev`: Für die Ereignisgerät-Unterstützung
   - `udev`: Für die Geräteerkennung

## 5. Schnittstellen

### 5.1 InputManager

```
SCHNITTSTELLE: system::input::InputManager
BESCHREIBUNG: Zentrale Komponente für die Verwaltung von Eingaben
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue InputManager-Instanz
    PARAMETER:
      - NAME: config
        TYP: InputConfig
        BESCHREIBUNG: Konfiguration für den InputManager
        EINSCHRÄNKUNGEN: Muss eine gültige InputConfig sein
    RÜCKGABETYP: Result<InputManager, InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des InputManagers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue InputManager-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des InputManagers auftritt
  
  - NAME: initialize
    BESCHREIBUNG: Initialisiert den InputManager
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn ein Fehler bei der Initialisierung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der InputManager wird initialisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Initialisierung auftritt
  
  - NAME: shutdown
    BESCHREIBUNG: Fährt den InputManager herunter
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn ein Fehler beim Herunterfahren auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der InputManager wird heruntergefahren
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Herunterfahren auftritt
  
  - NAME: get_keyboard_manager
    BESCHREIBUNG: Gibt den KeyboardManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &KeyboardManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der KeyboardManager wird zurückgegeben
  
  - NAME: get_mouse_manager
    BESCHREIBUNG: Gibt den MouseManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &MouseManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der MouseManager wird zurückgegeben
  
  - NAME: get_touch_manager
    BESCHREIBUNG: Gibt den TouchManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &TouchManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der TouchManager wird zurückgegeben
  
  - NAME: get_gesture_recognizer
    BESCHREIBUNG: Gibt den GestureRecognizer zurück
    PARAMETER: Keine
    RÜCKGABETYP: &GestureRecognizer
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der GestureRecognizer wird zurückgegeben
  
  - NAME: get_hotkey_manager
    BESCHREIBUNG: Gibt den HotkeyManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &HotkeyManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der HotkeyManager wird zurückgegeben
  
  - NAME: get_input_focus
    BESCHREIBUNG: Gibt den InputFocus zurück
    PARAMETER: Keine
    RÜCKGABETYP: &InputFocus
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der InputFocus wird zurückgegeben
  
  - NAME: get_accessibility_manager
    BESCHREIBUNG: Gibt den AccessibilityManager zurück
    PARAMETER: Keine
    RÜCKGABETYP: &AccessibilityManager
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der AccessibilityManager wird zurückgegeben
  
  - NAME: process_events
    BESCHREIBUNG: Verarbeitet ausstehende Eingabeereignisse
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn ein Fehler bei der Verarbeitung der Ereignisse auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Ausstehende Eingabeereignisse werden verarbeitet
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Verarbeitung der Ereignisse auftritt
  
  - NAME: add_input_device
    BESCHREIBUNG: Fügt ein Eingabegerät hinzu
    PARAMETER:
      - NAME: device
        TYP: InputDevice
        BESCHREIBUNG: Eingabegerät
        EINSCHRÄNKUNGEN: Muss ein gültiges InputDevice sein
    RÜCKGABETYP: Result<InputDeviceId, InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn ein Fehler beim Hinzufügen des Geräts auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Eingabegerät wird hinzugefügt
      - Eine InputDeviceId wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Hinzufügen des Geräts auftritt
  
  - NAME: remove_input_device
    BESCHREIBUNG: Entfernt ein Eingabegerät
    PARAMETER:
      - NAME: id
        TYP: InputDeviceId
        BESCHREIBUNG: ID des Eingabegeräts
        EINSCHRÄNKUNGEN: Muss eine gültige InputDeviceId sein
    RÜCKGABETYP: Result<(), InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn ein Fehler beim Entfernen des Geräts auftritt oder das Gerät nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Eingabegerät wird entfernt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Entfernen des Geräts auftritt oder das Gerät nicht gefunden wird
  
  - NAME: get_input_device
    BESCHREIBUNG: Gibt ein Eingabegerät zurück
    PARAMETER:
      - NAME: id
        TYP: InputDeviceId
        BESCHREIBUNG: ID des Eingabegeräts
        EINSCHRÄNKUNGEN: Muss eine gültige InputDeviceId sein
    RÜCKGABETYP: Result<&InputDevice, InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn das Gerät nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Eingabegerät wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Gerät nicht gefunden wird
  
  - NAME: get_all_input_devices
    BESCHREIBUNG: Gibt alle Eingabegeräte zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<&InputDevice>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Alle Eingabegeräte werden zurückgegeben
  
  - NAME: add_input_filter
    BESCHREIBUNG: Fügt einen Eingabefilter hinzu
    PARAMETER:
      - NAME: filter
        TYP: Box<dyn InputFilter>
        BESCHREIBUNG: Eingabefilter
        EINSCHRÄNKUNGEN: Muss ein gültiger InputFilter sein
    RÜCKGABETYP: FilterId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Eingabefilter wird hinzugefügt
      - Eine FilterId wird zurückgegeben
  
  - NAME: remove_input_filter
    BESCHREIBUNG: Entfernt einen Eingabefilter
    PARAMETER:
      - NAME: id
        TYP: FilterId
        BESCHREIBUNG: ID des Filters
        EINSCHRÄNKUNGEN: Muss eine gültige FilterId sein
    RÜCKGABETYP: Result<(), InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn der Filter nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Eingabefilter wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Filter nicht gefunden wird
  
  - NAME: add_input_transformer
    BESCHREIBUNG: Fügt einen Eingabetransformer hinzu
    PARAMETER:
      - NAME: transformer
        TYP: Box<dyn InputTransformer>
        BESCHREIBUNG: Eingabetransformer
        EINSCHRÄNKUNGEN: Muss ein gültiger InputTransformer sein
    RÜCKGABETYP: TransformerId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Eingabetransformer wird hinzugefügt
      - Eine TransformerId wird zurückgegeben
  
  - NAME: remove_input_transformer
    BESCHREIBUNG: Entfernt einen Eingabetransformer
    PARAMETER:
      - NAME: id
        TYP: TransformerId
        BESCHREIBUNG: ID des Transformers
        EINSCHRÄNKUNGEN: Muss eine gültige TransformerId sein
    RÜCKGABETYP: Result<(), InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn der Transformer nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Eingabetransformer wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Transformer nicht gefunden wird
  
  - NAME: register_input_event_listener
    BESCHREIBUNG: Registriert einen Listener für Eingabeereignisse
    PARAMETER:
      - NAME: listener
        TYP: Box<dyn Fn(&InputEvent) -> bool + Send + Sync + 'static>
        BESCHREIBUNG: Listener-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: ListenerId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird registriert und eine ListenerId wird zurückgegeben
  
  - NAME: unregister_input_event_listener
    BESCHREIBUNG: Entfernt einen Listener für Eingabeereignisse
    PARAMETER:
      - NAME: id
        TYP: ListenerId
        BESCHREIBUNG: ID des Listeners
        EINSCHRÄNKUNGEN: Muss eine gültige ListenerId sein
    RÜCKGABETYP: Result<(), InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn der Listener nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Listener nicht gefunden wird
```

### 5.2 KeyboardManager

```
SCHNITTSTELLE: system::input::KeyboardManager
BESCHREIBUNG: Komponente für die Verwaltung von Tastatureingaben
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue KeyboardManager-Instanz
    PARAMETER:
      - NAME: config
        TYP: KeyboardConfig
        BESCHREIBUNG: Konfiguration für den KeyboardManager
        EINSCHRÄNKUNGEN: Muss eine gültige KeyboardConfig sein
    RÜCKGABETYP: Result<KeyboardManager, InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des KeyboardManagers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue KeyboardManager-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des KeyboardManagers auftritt
  
  - NAME: process_key_event
    BESCHREIBUNG: Verarbeitet ein Tastaturereignis
    PARAMETER:
      - NAME: event
        TYP: &KeyEvent
        BESCHREIBUNG: Tastaturereignis
        EINSCHRÄNKUNGEN: Muss ein gültiges KeyEvent sein
    RÜCKGABETYP: Result<bool, InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn ein Fehler bei der Verarbeitung des Ereignisses auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Tastaturereignis wird verarbeitet
      - true wird zurückgegeben, wenn das Ereignis verarbeitet wurde
      - false wird zurückgegeben, wenn das Ereignis nicht verarbeitet wurde
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Verarbeitung des Ereignisses auftritt
  
  - NAME: is_key_pressed
    BESCHREIBUNG: Prüft, ob eine Taste gedrückt ist
    PARAMETER:
      - NAME: key_code
        TYP: KeyCode
        BESCHREIBUNG: Tastencode
        EINSCHRÄNKUNGEN: Muss ein gültiger KeyCode sein
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn die Taste gedrückt ist
      - false wird zurückgegeben, wenn die Taste nicht gedrückt ist
  
  - NAME: get_modifiers
    BESCHREIBUNG: Gibt die aktuellen Modifikatoren zurück
    PARAMETER: Keine
    RÜCKGABETYP: KeyModifiers
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die aktuellen Modifikatoren werden zurückgegeben
  
  - NAME: get_keyboard_layout
    BESCHREIBUNG: Gibt das aktuelle Tastaturlayout zurück
    PARAMETER: Keine
    RÜCKGABETYP: &KeyboardLayout
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das aktuelle Tastaturlayout wird zurückgegeben
  
  - NAME: set_keyboard_layout
    BESCHREIBUNG: Setzt das Tastaturlayout
    PARAMETER:
      - NAME: layout
        TYP: KeyboardLayout
        BESCHREIBUNG: Tastaturlayout
        EINSCHRÄNKUNGEN: Muss ein gültiges KeyboardLayout sein
    RÜCKGABETYP: Result<(), InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn ein Fehler beim Setzen des Tastaturlayouts auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Tastaturlayout wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen des Tastaturlayouts auftritt
  
  - NAME: get_available_keyboard_layouts
    BESCHREIBUNG: Gibt die verfügbaren Tastaturlayouts zurück
    PARAMETER: Keine
    RÜCKGABETYP: Vec<KeyboardLayout>
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die verfügbaren Tastaturlayouts werden zurückgegeben
  
  - NAME: get_key_repeat_rate
    BESCHREIBUNG: Gibt die Tastenwiederholungsrate zurück
    PARAMETER: Keine
    RÜCKGABETYP: KeyRepeatRate
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Tastenwiederholungsrate wird zurückgegeben
  
  - NAME: set_key_repeat_rate
    BESCHREIBUNG: Setzt die Tastenwiederholungsrate
    PARAMETER:
      - NAME: rate
        TYP: KeyRepeatRate
        BESCHREIBUNG: Tastenwiederholungsrate
        EINSCHRÄNKUNGEN: Muss eine gültige KeyRepeatRate sein
    RÜCKGABETYP: Result<(), InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn ein Fehler beim Setzen der Tastenwiederholungsrate auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Tastenwiederholungsrate wird gesetzt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Setzen der Tastenwiederholungsrate auftritt
  
  - NAME: register_key_event_listener
    BESCHREIBUNG: Registriert einen Listener für Tastaturereignisse
    PARAMETER:
      - NAME: listener
        TYP: Box<dyn Fn(&KeyEvent) -> bool + Send + Sync + 'static>
        BESCHREIBUNG: Listener-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: ListenerId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird registriert und eine ListenerId wird zurückgegeben
  
  - NAME: unregister_key_event_listener
    BESCHREIBUNG: Entfernt einen Listener für Tastaturereignisse
    PARAMETER:
      - NAME: id
        TYP: ListenerId
        BESCHREIBUNG: ID des Listeners
        EINSCHRÄNKUNGEN: Muss eine gültige ListenerId sein
    RÜCKGABETYP: Result<(), InputError>
    FEHLER:
      - TYP: InputError
        BEDINGUNG: Wenn der Listener nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Listener nicht gefunden wird
```

## 6. Datenmodell (Teil 1)

### 6.1 InputDeviceId

```
ENTITÄT: InputDeviceId
BESCHREIBUNG: Eindeutiger Bezeichner für ein Eingabegerät
ATTRIBUTE:
  - NAME: id
    TYP: u64
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
INVARIANTEN:
  - id muss eindeutig sein
```

### 6.2 InputDeviceType

```
ENTITÄT: InputDeviceType
BESCHREIBUNG: Typ eines Eingabegeräts
ATTRIBUTE:
  - NAME: device_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      Keyboard,
      Mouse,
      Touchpad,
      Touchscreen,
      Tablet,
      Gamepad,
      Joystick,
      Generic,
      Virtual
    }
    STANDARDWERT: Generic
INVARIANTEN:
  - Keine
```

### 6.3 InputDevice

```
ENTITÄT: InputDevice
BESCHREIBUNG: Repräsentation eines Eingabegeräts
ATTRIBUTE:
  - NAME: id
    TYP: InputDeviceId
    BESCHREIBUNG: Eindeutige ID
    WERTEBEREICH: Gültige InputDeviceId
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name des Geräts
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: "Unbekanntes Gerät"
  - NAME: device_type
    TYP: InputDeviceType
    BESCHREIBUNG: Typ des Geräts
    WERTEBEREICH: Gültiger InputDeviceType
    STANDARDWERT: InputDeviceType::Generic
  - NAME: capabilities
    TYP: InputCapabilities
    BESCHREIBUNG: Fähigkeiten des Geräts
    WERTEBEREICH: Gültige InputCapabilities
    STANDARDWERT: InputCapabilities::empty()
  - NAME: enabled
    TYP: bool
    BESCHREIBUNG: Ob das Gerät aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: vendor_id
    TYP: Option<u16>
    BESCHREIBUNG: Hersteller-ID
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: product_id
    TYP: Option<u16>
    BESCHREIBUNG: Produkt-ID
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: serial_number
    TYP: Option<String>
    BESCHREIBUNG: Seriennummer
    WERTEBEREICH: Nicht-leere Zeichenkette oder None
    STANDARDWERT: None
INVARIANTEN:
  - id muss eindeutig sein
  - name darf nicht leer sein
```

### 6.4 InputCapabilities

```
ENTITÄT: InputCapabilities
BESCHREIBUNG: Fähigkeiten eines Eingabegeräts
ATTRIBUTE:
  - NAME: capabilities
    TYP: BitFlags
    BESCHREIBUNG: Fähigkeiten
    WERTEBEREICH: {
      KEYBOARD,
      POINTER,
      TOUCH,
      TABLET,
      GESTURE,
      SWITCH,
      BUTTON,
      SCROLL,
      ABSOLUTE,
      RELATIVE
    }
    STANDARDWERT: Leere BitFlags
INVARIANTEN:
  - Keine
```

### 6.5 InputEventType

```
ENTITÄT: InputEventType
BESCHREIBUNG: Typ eines Eingabeereignisses
ATTRIBUTE:
  - NAME: event_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      KeyPress,
      KeyRelease,
      KeyRepeat,
      MouseMove,
      MouseButtonPress,
      MouseButtonRelease,
      MouseScroll,
      TouchBegin,
      TouchUpdate,
      TouchEnd,
      GestureBegin,
      GestureUpdate,
      GestureEnd,
      DeviceAdded,
      DeviceRemoved,
      DeviceChanged,
      FocusIn,
      FocusOut
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Keine
```

### 6.6 InputEvent

```
ENTITÄT: InputEvent
BESCHREIBUNG: Repräsentation eines Eingabeereignisses
ATTRIBUTE:
  - NAME: event_type
    TYP: InputEventType
    BESCHREIBUNG: Typ des Ereignisses
    WERTEBEREICH: Gültiger InputEventType
    STANDARDWERT: Keiner
  - NAME: device_id
    TYP: InputDeviceId
    BESCHREIBUNG: ID des Geräts
    WERTEBEREICH: Gültige InputDeviceId
    STANDARDWERT: Keiner
  - NAME: timestamp
    TYP: u64
    BESCHREIBUNG: Zeitstempel in Mikrosekunden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: Keiner
  - NAME: window_id
    TYP: Option<WindowId>
    BESCHREIBUNG: ID des Fensters
    WERTEBEREICH: Gültige WindowId oder None
    STANDARDWERT: None
  - NAME: data
    TYP: InputEventData
    BESCHREIBUNG: Ereignisdaten
    WERTEBEREICH: Gültige InputEventData
    STANDARDWERT: Keiner
INVARIANTEN:
  - Keine
```

### 6.7 InputEventData

```
ENTITÄT: InputEventData
BESCHREIBUNG: Daten eines Eingabeereignisses
ATTRIBUTE:
  - NAME: data_type
    TYP: Enum
    BESCHREIBUNG: Typ der Daten
    WERTEBEREICH: {
      Key {
        key_code: KeyCode,
        scan_code: u32,
        modifiers: KeyModifiers,
        repeat: bool,
        text: Option<String>
      },
      MouseMove {
        x: f64,
        y: f64,
        dx: f64,
        dy: f64
      },
      MouseButton {
        button: MouseButton,
        x: f64,
        y: f64,
        click_count: u8
      },
      MouseScroll {
        x: f64,
        y: f64,
        dx: f64,
        dy: f64,
        scroll_type: ScrollType
      },
      Touch {
        touch_id: TouchId,
        x: f64,
        y: f64,
        pressure: f32
      },
      Gesture {
        gesture_id: GestureId,
        gesture_type: GestureType,
        x: f64,
        y: f64,
        dx: f64,
        dy: f64,
        scale: f32,
        rotation: f32
      },
      Device {
        device_id: InputDeviceId
      },
      Focus {
        window_id: WindowId
      }
    }
    STANDARDWERT: Keiner
INVARIANTEN:
  - Keine
```

### 6.8 KeyCode

```
ENTITÄT: KeyCode
BESCHREIBUNG: Code einer Taste
ATTRIBUTE:
  - NAME: key_code
    TYP: Enum
    BESCHREIBUNG: Code
    WERTEBEREICH: {
      A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
      Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
      F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
      Escape, Tab, CapsLock, Shift, Control, Alt, Super, Space, Return, BackSpace,
      Insert, Delete, Home, End, PageUp, PageDown,
      Left, Right, Up, Down,
      NumLock, NumpadDivide, NumpadMultiply, NumpadSubtract, NumpadAdd, NumpadEnter, NumpadDecimal,
      Numpad0, Numpad1, Numpad2, Numpad3, Numpad4, Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
      PrintScreen, ScrollLock, Pause,
      Grave, Minus, Equal, LeftBracket, RightBracket, Backslash, Semicolon, Apostrophe, Comma, Period, Slash,
      Unknown(u32)
    }
    STANDARDWERT: KeyCode::Unknown(0)
INVARIANTEN:
  - Keine
```

### 6.9 KeyModifiers

```
ENTITÄT: KeyModifiers
BESCHREIBUNG: Modifikatoren einer Taste
ATTRIBUTE:
  - NAME: modifiers
    TYP: BitFlags
    BESCHREIBUNG: Modifikatoren
    WERTEBEREICH: {
      SHIFT,
      CONTROL,
      ALT,
      SUPER,
      CAPS_LOCK,
      NUM_LOCK
    }
    STANDARDWERT: Leere BitFlags
INVARIANTEN:
  - Keine
```

### 6.10 MouseButton

```
ENTITÄT: MouseButton
BESCHREIBUNG: Maustaste
ATTRIBUTE:
  - NAME: button
    TYP: Enum
    BESCHREIBUNG: Taste
    WERTEBEREICH: {
      Left,
      Right,
      Middle,
      Back,
      Forward,
      Extra(u8)
    }
    STANDARDWERT: MouseButton::Left
INVARIANTEN:
  - Keine
```
