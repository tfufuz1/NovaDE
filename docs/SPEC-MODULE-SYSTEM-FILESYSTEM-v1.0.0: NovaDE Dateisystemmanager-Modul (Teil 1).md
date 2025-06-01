# SPEC-MODULE-SYSTEM-FILESYSTEM-v1.0.0: NovaDE Dateisystemmanager-Modul (Teil 1)

```
SPEZIFIKATION: SPEC-MODULE-SYSTEM-FILESYSTEM-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0, SPEC-LAYER-CORE-v1.0.0, SPEC-LAYER-SYSTEM-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Diese Spezifikation definiert das Dateisystemmanager-Modul (`system::filesystem`) der NovaDE-Systemschicht. Das Modul stellt die grundlegende Infrastruktur für den Zugriff auf und die Verwaltung von Dateien und Verzeichnissen bereit und definiert die Mechanismen zur Interaktion mit dem Dateisystem, zur Überwachung von Dateisystemereignissen und zur Integration mit verschiedenen Dateisystemtypen. Der Geltungsbereich umfasst alle Komponenten und Schnittstellen des Dateisystemmanager-Moduls sowie deren Interaktionen mit anderen Modulen.

## 2. Definitionen

### 2.1 Allgemeine Begriffe

- **Dateisystem**: System zur Organisation von Dateien und Verzeichnissen
- **Datei**: Benannte Sammlung von Daten im Dateisystem
- **Verzeichnis**: Container für Dateien und andere Verzeichnisse
- **Pfad**: Zeichenkette, die den Speicherort einer Datei oder eines Verzeichnisses angibt
- **Dateisystemereignis**: Änderung im Dateisystem, wie das Erstellen, Ändern oder Löschen einer Datei
- **Dateisystemwächter**: Komponente, die Dateisystemereignisse überwacht
- **Dateisystemtreiber**: Komponente, die den Zugriff auf ein bestimmtes Dateisystem ermöglicht
- **Dateisystemmontage**: Prozess des Einbindens eines Dateisystems in die Verzeichnisstruktur
- **Dateisystemrechte**: Berechtigungen für den Zugriff auf Dateien und Verzeichnisse
- **Dateisystemquota**: Begrenzung des Speicherplatzes für Benutzer oder Gruppen

### 2.2 Modulspezifische Begriffe

- **FileSystemManager**: Zentrale Komponente für die Verwaltung des Dateisystems
- **FileSystemWatcher**: Komponente zur Überwachung von Dateisystemereignissen
- **FileSystemDriver**: Komponente für den Zugriff auf ein bestimmtes Dateisystem
- **FileSystemMount**: Komponente für die Verwaltung von Dateisystemmontagen
- **FileSystemPermissions**: Komponente für die Verwaltung von Dateisystemrechten
- **FileSystemQuota**: Komponente für die Verwaltung von Dateisystemquotas
- **FileSystemCache**: Komponente für das Caching von Dateisystemoperationen
- **FileSystemSearch**: Komponente für die Suche im Dateisystem
- **FileSystemMetadata**: Komponente für die Verwaltung von Dateisystemmetadaten
- **FileSystemTrash**: Komponente für die Verwaltung des Papierkorbs

## 3. Anforderungen

### 3.1 Funktionale Anforderungen

1. Das Modul MUSS Mechanismen zum Lesen von Dateien bereitstellen.
2. Das Modul MUSS Mechanismen zum Schreiben von Dateien bereitstellen.
3. Das Modul MUSS Mechanismen zum Erstellen von Verzeichnissen bereitstellen.
4. Das Modul MUSS Mechanismen zum Löschen von Dateien und Verzeichnissen bereitstellen.
5. Das Modul MUSS Mechanismen zum Kopieren von Dateien und Verzeichnissen bereitstellen.
6. Das Modul MUSS Mechanismen zum Verschieben von Dateien und Verzeichnissen bereitstellen.
7. Das Modul MUSS Mechanismen zum Umbenennen von Dateien und Verzeichnissen bereitstellen.
8. Das Modul MUSS Mechanismen zum Abfragen von Dateisysteminformationen bereitstellen.
9. Das Modul MUSS Mechanismen zum Überwachen von Dateisystemereignissen bereitstellen.
10. Das Modul MUSS Mechanismen zur Verwaltung von Dateisystemmontagen bereitstellen.
11. Das Modul MUSS Mechanismen zur Verwaltung von Dateisystemrechten bereitstellen.
12. Das Modul MUSS Mechanismen zur Verwaltung von Dateisystemquotas bereitstellen.
13. Das Modul MUSS Mechanismen zur Suche im Dateisystem bereitstellen.
14. Das Modul MUSS Mechanismen zur Verwaltung des Papierkorbs bereitstellen.

### 3.2 Nicht-funktionale Anforderungen

1. Das Modul MUSS effizient mit Ressourcen umgehen.
2. Das Modul MUSS thread-safe sein.
3. Das Modul MUSS eine klare und konsistente API bereitstellen.
4. Das Modul MUSS gut dokumentiert sein.
5. Das Modul MUSS leicht erweiterbar sein.
6. Das Modul MUSS robust gegen Fehleingaben sein.
7. Das Modul MUSS minimale externe Abhängigkeiten haben.
8. Das Modul MUSS eine hohe Performance bieten.
9. Das Modul MUSS eine geringe Latenz bei Dateisystemoperationen bieten.
10. Das Modul MUSS eine hohe Zuverlässigkeit bieten.

## 4. Architektur

### 4.1 Komponentenstruktur

Das Dateisystemmanager-Modul besteht aus den folgenden Komponenten:

1. **FileSystemManager** (`filesystem_manager.rs`): Zentrale Komponente für die Verwaltung des Dateisystems
2. **FileSystemWatcher** (`filesystem_watcher.rs`): Komponente zur Überwachung von Dateisystemereignissen
3. **FileSystemDriver** (`filesystem_driver.rs`): Komponente für den Zugriff auf ein bestimmtes Dateisystem
4. **FileSystemMount** (`filesystem_mount.rs`): Komponente für die Verwaltung von Dateisystemmontagen
5. **FileSystemPermissions** (`filesystem_permissions.rs`): Komponente für die Verwaltung von Dateisystemrechten
6. **FileSystemQuota** (`filesystem_quota.rs`): Komponente für die Verwaltung von Dateisystemquotas
7. **FileSystemCache** (`filesystem_cache.rs`): Komponente für das Caching von Dateisystemoperationen
8. **FileSystemSearch** (`filesystem_search.rs`): Komponente für die Suche im Dateisystem
9. **FileSystemMetadata** (`filesystem_metadata.rs`): Komponente für die Verwaltung von Dateisystemmetadaten
10. **FileSystemTrash** (`filesystem_trash.rs`): Komponente für die Verwaltung des Papierkorbs
11. **FileSystemConfig** (`filesystem_config.rs`): Komponente für die Konfiguration des Dateisystemmanagers
12. **FileSystemEvent** (`filesystem_event.rs`): Komponente für die Verwaltung von Dateisystemereignissen
13. **FileSystemPath** (`filesystem_path.rs`): Komponente für die Verwaltung von Dateisystempfaden
14. **FileSystemError** (`filesystem_error.rs`): Komponente für die Fehlerbehandlung im Dateisystemmanager

### 4.2 Abhängigkeiten

Das Dateisystemmanager-Modul hat folgende Abhängigkeiten:

1. **Interne Abhängigkeiten**:
   - `core::errors`: Für die Fehlerbehandlung
   - `core::config`: Für die Konfiguration
   - `core::logging`: Für das Logging
   - `system::process`: Für die Prozessverwaltung

2. **Externe Abhängigkeiten**:
   - `std::fs`: Für grundlegende Dateisystemoperationen
   - `std::path`: Für die Pfadverwaltung
   - `std::io`: Für Ein-/Ausgabeoperationen
   - `notify`: Für die Überwachung von Dateisystemereignissen
   - `walkdir`: Für das rekursive Durchlaufen von Verzeichnissen
   - `glob`: Für die Mustersuche in Dateipfaden
   - `xattr`: Für die Verwaltung von erweiterten Attributen
   - `users`: Für die Benutzerverwaltung
   - `mount`: Für die Verwaltung von Dateisystemmontagen

## 5. Schnittstellen

### 5.1 FileSystemManager

```
SCHNITTSTELLE: system::filesystem::FileSystemManager
BESCHREIBUNG: Zentrale Komponente für die Verwaltung des Dateisystems
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue FileSystemManager-Instanz
    PARAMETER:
      - NAME: config
        TYP: FileSystemConfig
        BESCHREIBUNG: Konfiguration für den FileSystemManager
        EINSCHRÄNKUNGEN: Muss eine gültige FileSystemConfig sein
    RÜCKGABETYP: Result<FileSystemManager, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des FileSystemManagers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue FileSystemManager-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des FileSystemManagers auftritt
  
  - NAME: initialize
    BESCHREIBUNG: Initialisiert den FileSystemManager
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn ein Fehler bei der Initialisierung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der FileSystemManager wird initialisiert
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Initialisierung auftritt
  
  - NAME: shutdown
    BESCHREIBUNG: Fährt den FileSystemManager herunter
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn ein Fehler beim Herunterfahren auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der FileSystemManager wird heruntergefahren
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Herunterfahren auftritt
  
  - NAME: read_file
    BESCHREIBUNG: Liest den Inhalt einer Datei
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
    RÜCKGABETYP: Result<Vec<u8>, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht existiert, nicht lesbar ist oder ein Fehler beim Lesen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Inhalt der Datei wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht existiert, nicht lesbar ist oder ein Fehler beim Lesen auftritt
  
  - NAME: read_file_to_string
    BESCHREIBUNG: Liest den Inhalt einer Datei als Zeichenkette
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
    RÜCKGABETYP: Result<String, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht existiert, nicht lesbar ist, kein gültiger UTF-8-Text ist oder ein Fehler beim Lesen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Inhalt der Datei wird als Zeichenkette zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht existiert, nicht lesbar ist, kein gültiger UTF-8-Text ist oder ein Fehler beim Lesen auftritt
  
  - NAME: write_file
    BESCHREIBUNG: Schreibt Daten in eine Datei
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: data
        TYP: &[u8]
        BESCHREIBUNG: Zu schreibende Daten
        EINSCHRÄNKUNGEN: Keine
      - NAME: options
        TYP: WriteOptions
        BESCHREIBUNG: Optionen für das Schreiben
        EINSCHRÄNKUNGEN: Muss gültige WriteOptions sein
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht geschrieben werden kann oder ein Fehler beim Schreiben auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Daten werden in die Datei geschrieben
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht geschrieben werden kann oder ein Fehler beim Schreiben auftritt
  
  - NAME: write_file_from_string
    BESCHREIBUNG: Schreibt eine Zeichenkette in eine Datei
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: content
        TYP: &str
        BESCHREIBUNG: Zu schreibender Inhalt
        EINSCHRÄNKUNGEN: Keine
      - NAME: options
        TYP: WriteOptions
        BESCHREIBUNG: Optionen für das Schreiben
        EINSCHRÄNKUNGEN: Muss gültige WriteOptions sein
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht geschrieben werden kann oder ein Fehler beim Schreiben auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Inhalt wird in die Datei geschrieben
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht geschrieben werden kann oder ein Fehler beim Schreiben auftritt
  
  - NAME: create_directory
    BESCHREIBUNG: Erstellt ein Verzeichnis
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zum Verzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: recursive
        TYP: bool
        BESCHREIBUNG: Ob übergeordnete Verzeichnisse bei Bedarf erstellt werden sollen
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn das Verzeichnis nicht erstellt werden kann oder ein Fehler beim Erstellen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Verzeichnis wird erstellt
      - Ein Fehler wird zurückgegeben, wenn das Verzeichnis nicht erstellt werden kann oder ein Fehler beim Erstellen auftritt
  
  - NAME: delete_file
    BESCHREIBUNG: Löscht eine Datei
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: use_trash
        TYP: bool
        BESCHREIBUNG: Ob die Datei in den Papierkorb verschoben werden soll
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht gelöscht werden kann oder ein Fehler beim Löschen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Datei wird gelöscht oder in den Papierkorb verschoben
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht gelöscht werden kann oder ein Fehler beim Löschen auftritt
  
  - NAME: delete_directory
    BESCHREIBUNG: Löscht ein Verzeichnis
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zum Verzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: recursive
        TYP: bool
        BESCHREIBUNG: Ob das Verzeichnis rekursiv gelöscht werden soll
        EINSCHRÄNKUNGEN: Keine
      - NAME: use_trash
        TYP: bool
        BESCHREIBUNG: Ob das Verzeichnis in den Papierkorb verschoben werden soll
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn das Verzeichnis nicht gelöscht werden kann oder ein Fehler beim Löschen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Verzeichnis wird gelöscht oder in den Papierkorb verschoben
      - Ein Fehler wird zurückgegeben, wenn das Verzeichnis nicht gelöscht werden kann oder ein Fehler beim Löschen auftritt
  
  - NAME: copy_file
    BESCHREIBUNG: Kopiert eine Datei
    PARAMETER:
      - NAME: source
        TYP: &Path
        BESCHREIBUNG: Pfad zur Quelldatei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: destination
        TYP: &Path
        BESCHREIBUNG: Pfad zur Zieldatei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: options
        TYP: CopyOptions
        BESCHREIBUNG: Optionen für das Kopieren
        EINSCHRÄNKUNGEN: Muss gültige CopyOptions sein
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht kopiert werden kann oder ein Fehler beim Kopieren auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Datei wird kopiert
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht kopiert werden kann oder ein Fehler beim Kopieren auftritt
  
  - NAME: copy_directory
    BESCHREIBUNG: Kopiert ein Verzeichnis
    PARAMETER:
      - NAME: source
        TYP: &Path
        BESCHREIBUNG: Pfad zum Quellverzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: destination
        TYP: &Path
        BESCHREIBUNG: Pfad zum Zielverzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: options
        TYP: CopyOptions
        BESCHREIBUNG: Optionen für das Kopieren
        EINSCHRÄNKUNGEN: Muss gültige CopyOptions sein
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn das Verzeichnis nicht kopiert werden kann oder ein Fehler beim Kopieren auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Verzeichnis wird kopiert
      - Ein Fehler wird zurückgegeben, wenn das Verzeichnis nicht kopiert werden kann oder ein Fehler beim Kopieren auftritt
  
  - NAME: move_file
    BESCHREIBUNG: Verschiebt eine Datei
    PARAMETER:
      - NAME: source
        TYP: &Path
        BESCHREIBUNG: Pfad zur Quelldatei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: destination
        TYP: &Path
        BESCHREIBUNG: Pfad zur Zieldatei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: options
        TYP: MoveOptions
        BESCHREIBUNG: Optionen für das Verschieben
        EINSCHRÄNKUNGEN: Muss gültige MoveOptions sein
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht verschoben werden kann oder ein Fehler beim Verschieben auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Datei wird verschoben
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht verschoben werden kann oder ein Fehler beim Verschieben auftritt
  
  - NAME: move_directory
    BESCHREIBUNG: Verschiebt ein Verzeichnis
    PARAMETER:
      - NAME: source
        TYP: &Path
        BESCHREIBUNG: Pfad zum Quellverzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: destination
        TYP: &Path
        BESCHREIBUNG: Pfad zum Zielverzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: options
        TYP: MoveOptions
        BESCHREIBUNG: Optionen für das Verschieben
        EINSCHRÄNKUNGEN: Muss gültige MoveOptions sein
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn das Verzeichnis nicht verschoben werden kann oder ein Fehler beim Verschieben auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Das Verzeichnis wird verschoben
      - Ein Fehler wird zurückgegeben, wenn das Verzeichnis nicht verschoben werden kann oder ein Fehler beim Verschieben auftritt
  
  - NAME: rename
    BESCHREIBUNG: Benennt eine Datei oder ein Verzeichnis um
    PARAMETER:
      - NAME: source
        TYP: &Path
        BESCHREIBUNG: Pfad zur Quelldatei oder zum Quellverzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: destination
        TYP: &Path
        BESCHREIBUNG: Pfad zur Zieldatei oder zum Zielverzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei oder das Verzeichnis nicht umbenannt werden kann oder ein Fehler beim Umbenennen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Datei oder das Verzeichnis wird umbenannt
      - Ein Fehler wird zurückgegeben, wenn die Datei oder das Verzeichnis nicht umbenannt werden kann oder ein Fehler beim Umbenennen auftritt
  
  - NAME: exists
    BESCHREIBUNG: Prüft, ob eine Datei oder ein Verzeichnis existiert
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei oder zum Verzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn die Datei oder das Verzeichnis existiert
      - false wird zurückgegeben, wenn die Datei oder das Verzeichnis nicht existiert
  
  - NAME: is_file
    BESCHREIBUNG: Prüft, ob ein Pfad auf eine Datei verweist
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn der Pfad auf eine Datei verweist
      - false wird zurückgegeben, wenn der Pfad nicht auf eine Datei verweist
  
  - NAME: is_directory
    BESCHREIBUNG: Prüft, ob ein Pfad auf ein Verzeichnis verweist
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zum Verzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn der Pfad auf ein Verzeichnis verweist
      - false wird zurückgegeben, wenn der Pfad nicht auf ein Verzeichnis verweist
  
  - NAME: get_metadata
    BESCHREIBUNG: Gibt Metadaten für eine Datei oder ein Verzeichnis zurück
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei oder zum Verzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
    RÜCKGABETYP: Result<FileSystemMetadata, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei oder das Verzeichnis nicht existiert oder ein Fehler beim Abrufen der Metadaten auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Metadaten werden zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn die Datei oder das Verzeichnis nicht existiert oder ein Fehler beim Abrufen der Metadaten auftritt
  
  - NAME: list_directory
    BESCHREIBUNG: Listet den Inhalt eines Verzeichnisses auf
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zum Verzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: options
        TYP: ListOptions
        BESCHREIBUNG: Optionen für das Auflisten
        EINSCHRÄNKUNGEN: Muss gültige ListOptions sein
    RÜCKGABETYP: Result<Vec<FileSystemEntry>, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn das Verzeichnis nicht existiert, nicht lesbar ist oder ein Fehler beim Auflisten auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Inhalt des Verzeichnisses wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Verzeichnis nicht existiert, nicht lesbar ist oder ein Fehler beim Auflisten auftritt
  
  - NAME: get_file_size
    BESCHREIBUNG: Gibt die Größe einer Datei zurück
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
    RÜCKGABETYP: Result<u64, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht existiert oder ein Fehler beim Abrufen der Größe auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Größe der Datei wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht existiert oder ein Fehler beim Abrufen der Größe auftritt
  
  - NAME: get_directory_size
    BESCHREIBUNG: Gibt die Größe eines Verzeichnisses zurück
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zum Verzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: recursive
        TYP: bool
        BESCHREIBUNG: Ob die Größe rekursiv berechnet werden soll
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: Result<u64, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn das Verzeichnis nicht existiert oder ein Fehler beim Abrufen der Größe auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Größe des Verzeichnisses wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn das Verzeichnis nicht existiert oder ein Fehler beim Abrufen der Größe auftritt
  
  - NAME: get_file_type
    BESCHREIBUNG: Gibt den Typ einer Datei zurück
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
    RÜCKGABETYP: Result<FileType, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht existiert oder ein Fehler beim Abrufen des Typs auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Typ der Datei wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht existiert oder ein Fehler beim Abrufen des Typs auftritt
  
  - NAME: get_file_permissions
    BESCHREIBUNG: Gibt die Berechtigungen einer Datei zurück
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
    RÜCKGABETYP: Result<FilePermissions, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht existiert oder ein Fehler beim Abrufen der Berechtigungen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Berechtigungen der Datei werden zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht existiert oder ein Fehler beim Abrufen der Berechtigungen auftritt
  
  - NAME: set_file_permissions
    BESCHREIBUNG: Setzt die Berechtigungen einer Datei
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zur Datei
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: permissions
        TYP: FilePermissions
        BESCHREIBUNG: Berechtigungen
        EINSCHRÄNKUNGEN: Muss gültige FilePermissions sein
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn die Datei nicht existiert oder ein Fehler beim Setzen der Berechtigungen auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Berechtigungen der Datei werden gesetzt
      - Ein Fehler wird zurückgegeben, wenn die Datei nicht existiert oder ein Fehler beim Setzen der Berechtigungen auftritt
```

### 5.2 FileSystemWatcher

```
SCHNITTSTELLE: system::filesystem::FileSystemWatcher
BESCHREIBUNG: Komponente zur Überwachung von Dateisystemereignissen
VERSION: 1.0.0
OPERATIONEN:
  - NAME: new
    BESCHREIBUNG: Erstellt eine neue FileSystemWatcher-Instanz
    PARAMETER:
      - NAME: config
        TYP: FileSystemWatcherConfig
        BESCHREIBUNG: Konfiguration für den FileSystemWatcher
        EINSCHRÄNKUNGEN: Muss eine gültige FileSystemWatcherConfig sein
    RÜCKGABETYP: Result<FileSystemWatcher, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn ein Fehler bei der Erstellung des FileSystemWatchers auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Eine neue FileSystemWatcher-Instanz wird erstellt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler bei der Erstellung des FileSystemWatchers auftritt
  
  - NAME: start
    BESCHREIBUNG: Startet die Überwachung
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn ein Fehler beim Starten der Überwachung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Überwachung wird gestartet
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Starten der Überwachung auftritt
  
  - NAME: stop
    BESCHREIBUNG: Stoppt die Überwachung
    PARAMETER: Keine
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn ein Fehler beim Stoppen der Überwachung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Überwachung wird gestoppt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Stoppen der Überwachung auftritt
  
  - NAME: is_running
    BESCHREIBUNG: Prüft, ob die Überwachung läuft
    PARAMETER: Keine
    RÜCKGABETYP: bool
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - true wird zurückgegeben, wenn die Überwachung läuft
      - false wird zurückgegeben, wenn die Überwachung nicht läuft
  
  - NAME: add_watch
    BESCHREIBUNG: Fügt eine Überwachung hinzu
    PARAMETER:
      - NAME: path
        TYP: &Path
        BESCHREIBUNG: Pfad zum zu überwachenden Verzeichnis
        EINSCHRÄNKUNGEN: Muss ein gültiger Pfad sein
      - NAME: recursive
        TYP: bool
        BESCHREIBUNG: Ob das Verzeichnis rekursiv überwacht werden soll
        EINSCHRÄNKUNGEN: Keine
      - NAME: event_types
        TYP: Vec<FileSystemEventType>
        BESCHREIBUNG: Zu überwachende Ereignistypen
        EINSCHRÄNKUNGEN: Muss gültige FileSystemEventType-Werte enthalten
    RÜCKGABETYP: Result<WatchId, FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn ein Fehler beim Hinzufügen der Überwachung auftritt
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Überwachung wird hinzugefügt
      - Eine WatchId wird zurückgegeben
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Hinzufügen der Überwachung auftritt
  
  - NAME: remove_watch
    BESCHREIBUNG: Entfernt eine Überwachung
    PARAMETER:
      - NAME: id
        TYP: WatchId
        BESCHREIBUNG: ID der Überwachung
        EINSCHRÄNKUNGEN: Muss eine gültige WatchId sein
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn ein Fehler beim Entfernen der Überwachung auftritt oder die Überwachung nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Die Überwachung wird entfernt
      - Ein Fehler wird zurückgegeben, wenn ein Fehler beim Entfernen der Überwachung auftritt oder die Überwachung nicht gefunden wird
  
  - NAME: register_event_listener
    BESCHREIBUNG: Registriert einen Listener für Dateisystemereignisse
    PARAMETER:
      - NAME: listener
        TYP: Box<dyn Fn(&FileSystemEvent) -> bool + Send + Sync + 'static>
        BESCHREIBUNG: Listener-Funktion
        EINSCHRÄNKUNGEN: Keine
    RÜCKGABETYP: ListenerId
    FEHLER: Keine
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird registriert und eine ListenerId wird zurückgegeben
  
  - NAME: unregister_event_listener
    BESCHREIBUNG: Entfernt einen Listener für Dateisystemereignisse
    PARAMETER:
      - NAME: id
        TYP: ListenerId
        BESCHREIBUNG: ID des Listeners
        EINSCHRÄNKUNGEN: Muss eine gültige ListenerId sein
    RÜCKGABETYP: Result<(), FileSystemError>
    FEHLER:
      - TYP: FileSystemError
        BEDINGUNG: Wenn der Listener nicht gefunden wird
        BEHANDLUNG: Fehler wird an den Aufrufer zurückgegeben
    VORBEDINGUNGEN: Keine
    NACHBEDINGUNGEN:
      - Der Listener wird entfernt
      - Ein Fehler wird zurückgegeben, wenn der Listener nicht gefunden wird
```

## 6. Datenmodell (Teil 1)

### 6.1 FileSystemConfig

```
ENTITÄT: FileSystemConfig
BESCHREIBUNG: Konfiguration für den FileSystemManager
ATTRIBUTE:
  - NAME: cache_enabled
    TYP: bool
    BESCHREIBUNG: Ob das Caching aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: cache_size
    TYP: usize
    BESCHREIBUNG: Größe des Caches in Bytes
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 10485760 (10 MB)
  - NAME: cache_ttl
    TYP: Duration
    BESCHREIBUNG: Time-to-Live für Cache-Einträge
    WERTEBEREICH: Positive Zeitdauer
    STANDARDWERT: Duration::from_secs(60)
  - NAME: trash_enabled
    TYP: bool
    BESCHREIBUNG: Ob der Papierkorb aktiviert ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: trash_location
    TYP: PathBuf
    BESCHREIBUNG: Pfad zum Papierkorb
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: PathBuf::from("~/.local/share/Trash")
  - NAME: trash_retention_days
    TYP: u32
    BESCHREIBUNG: Anzahl der Tage, für die Dateien im Papierkorb aufbewahrt werden
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 30
  - NAME: default_permissions
    TYP: FilePermissions
    BESCHREIBUNG: Standardberechtigungen für neue Dateien
    WERTEBEREICH: Gültige FilePermissions
    STANDARDWERT: FilePermissions::default()
  - NAME: default_directory_permissions
    TYP: FilePermissions
    BESCHREIBUNG: Standardberechtigungen für neue Verzeichnisse
    WERTEBEREICH: Gültige FilePermissions
    STANDARDWERT: FilePermissions::default_directory()
  - NAME: follow_symlinks
    TYP: bool
    BESCHREIBUNG: Ob symbolischen Links gefolgt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: max_file_size
    TYP: Option<u64>
    BESCHREIBUNG: Maximale Dateigröße in Bytes
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: max_directory_depth
    TYP: Option<u32>
    BESCHREIBUNG: Maximale Verzeichnistiefe
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: temp_directory
    TYP: PathBuf
    BESCHREIBUNG: Pfad zum temporären Verzeichnis
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: PathBuf::from("/tmp")
  - NAME: drivers
    TYP: Vec<Box<dyn FileSystemDriver>>
    BESCHREIBUNG: Dateisystemtreiber
    WERTEBEREICH: Gültige FileSystemDriver-Implementierungen
    STANDARDWERT: Leerer Vec
INVARIANTEN:
  - cache_size muss größer als 0 sein
  - trash_retention_days muss größer als 0 sein
```

### 6.2 WriteOptions

```
ENTITÄT: WriteOptions
BESCHREIBUNG: Optionen für das Schreiben von Dateien
ATTRIBUTE:
  - NAME: create
    TYP: bool
    BESCHREIBUNG: Ob die Datei erstellt werden soll, wenn sie nicht existiert
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: append
    TYP: bool
    BESCHREIBUNG: Ob an die Datei angehängt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: truncate
    TYP: bool
    BESCHREIBUNG: Ob die Datei abgeschnitten werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: create_new
    TYP: bool
    BESCHREIBUNG: Ob die Datei nur erstellt werden soll, wenn sie nicht existiert
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: permissions
    TYP: Option<FilePermissions>
    BESCHREIBUNG: Berechtigungen für die Datei
    WERTEBEREICH: Gültige FilePermissions oder None
    STANDARDWERT: None
  - NAME: sync
    TYP: bool
    BESCHREIBUNG: Ob die Datei synchron geschrieben werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
INVARIANTEN:
  - create und create_new können nicht beide true sein
  - append und truncate können nicht beide true sein
```

### 6.3 CopyOptions

```
ENTITÄT: CopyOptions
BESCHREIBUNG: Optionen für das Kopieren von Dateien und Verzeichnissen
ATTRIBUTE:
  - NAME: overwrite
    TYP: bool
    BESCHREIBUNG: Ob bestehende Dateien überschrieben werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: skip_existing
    TYP: bool
    BESCHREIBUNG: Ob bestehende Dateien übersprungen werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: copy_permissions
    TYP: bool
    BESCHREIBUNG: Ob Berechtigungen kopiert werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: copy_timestamps
    TYP: bool
    BESCHREIBUNG: Ob Zeitstempel kopiert werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: copy_xattrs
    TYP: bool
    BESCHREIBUNG: Ob erweiterte Attribute kopiert werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: recursive
    TYP: bool
    BESCHREIBUNG: Ob rekursiv kopiert werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: follow_symlinks
    TYP: bool
    BESCHREIBUNG: Ob symbolischen Links gefolgt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: progress_callback
    TYP: Option<Box<dyn Fn(u64, u64) -> bool + Send + Sync + 'static>>
    BESCHREIBUNG: Callback-Funktion für den Fortschritt
    WERTEBEREICH: Gültige Funktion oder None
    STANDARDWERT: None
INVARIANTEN:
  - overwrite und skip_existing können nicht beide true sein
```

### 6.4 MoveOptions

```
ENTITÄT: MoveOptions
BESCHREIBUNG: Optionen für das Verschieben von Dateien und Verzeichnissen
ATTRIBUTE:
  - NAME: overwrite
    TYP: bool
    BESCHREIBUNG: Ob bestehende Dateien überschrieben werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: skip_existing
    TYP: bool
    BESCHREIBUNG: Ob bestehende Dateien übersprungen werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: atomic
    TYP: bool
    BESCHREIBUNG: Ob die Operation atomar sein soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: progress_callback
    TYP: Option<Box<dyn Fn(u64, u64) -> bool + Send + Sync + 'static>>
    BESCHREIBUNG: Callback-Funktion für den Fortschritt
    WERTEBEREICH: Gültige Funktion oder None
    STANDARDWERT: None
INVARIANTEN:
  - overwrite und skip_existing können nicht beide true sein
```

### 6.5 ListOptions

```
ENTITÄT: ListOptions
BESCHREIBUNG: Optionen für das Auflisten von Verzeichnissen
ATTRIBUTE:
  - NAME: recursive
    TYP: bool
    BESCHREIBUNG: Ob rekursiv aufgelistet werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: include_hidden
    TYP: bool
    BESCHREIBUNG: Ob versteckte Dateien eingeschlossen werden sollen
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: follow_symlinks
    TYP: bool
    BESCHREIBUNG: Ob symbolischen Links gefolgt werden soll
    WERTEBEREICH: {true, false}
    STANDARDWERT: true
  - NAME: sort_by
    TYP: Option<FileSortCriteria>
    BESCHREIBUNG: Kriterium für die Sortierung
    WERTEBEREICH: Gültige FileSortCriteria oder None
    STANDARDWERT: None
  - NAME: sort_order
    TYP: SortOrder
    BESCHREIBUNG: Sortierreihenfolge
    WERTEBEREICH: Gültige SortOrder
    STANDARDWERT: SortOrder::Ascending
  - NAME: max_depth
    TYP: Option<u32>
    BESCHREIBUNG: Maximale Verzeichnistiefe
    WERTEBEREICH: Positive Ganzzahlen oder None
    STANDARDWERT: None
  - NAME: filter
    TYP: Option<Box<dyn Fn(&FileSystemEntry) -> bool + Send + Sync + 'static>>
    BESCHREIBUNG: Filterfunktion
    WERTEBEREICH: Gültige Funktion oder None
    STANDARDWERT: None
INVARIANTEN:
  - Keine
```

### 6.6 FileSortCriteria

```
ENTITÄT: FileSortCriteria
BESCHREIBUNG: Kriterium für die Sortierung von Dateien
ATTRIBUTE:
  - NAME: criteria
    TYP: Enum
    BESCHREIBUNG: Kriterium
    WERTEBEREICH: {
      Name,
      Size,
      ModificationTime,
      CreationTime,
      AccessTime,
      Type,
      Extension
    }
    STANDARDWERT: Name
INVARIANTEN:
  - Keine
```

### 6.7 SortOrder

```
ENTITÄT: SortOrder
BESCHREIBUNG: Sortierreihenfolge
ATTRIBUTE:
  - NAME: order
    TYP: Enum
    BESCHREIBUNG: Reihenfolge
    WERTEBEREICH: {
      Ascending,
      Descending
    }
    STANDARDWERT: Ascending
INVARIANTEN:
  - Keine
```

### 6.8 FileSystemEntry

```
ENTITÄT: FileSystemEntry
BESCHREIBUNG: Eintrag im Dateisystem
ATTRIBUTE:
  - NAME: path
    TYP: PathBuf
    BESCHREIBUNG: Pfad zum Eintrag
    WERTEBEREICH: Gültiger Pfad
    STANDARDWERT: Keiner
  - NAME: name
    TYP: String
    BESCHREIBUNG: Name des Eintrags
    WERTEBEREICH: Nicht-leere Zeichenkette
    STANDARDWERT: Keiner
  - NAME: entry_type
    TYP: FileType
    BESCHREIBUNG: Typ des Eintrags
    WERTEBEREICH: Gültiger FileType
    STANDARDWERT: Keiner
  - NAME: size
    TYP: u64
    BESCHREIBUNG: Größe des Eintrags in Bytes
    WERTEBEREICH: Positive Ganzzahlen
    STANDARDWERT: 0
  - NAME: modified_time
    TYP: Option<SystemTime>
    BESCHREIBUNG: Zeitpunkt der letzten Änderung
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
  - NAME: creation_time
    TYP: Option<SystemTime>
    BESCHREIBUNG: Zeitpunkt der Erstellung
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
  - NAME: access_time
    TYP: Option<SystemTime>
    BESCHREIBUNG: Zeitpunkt des letzten Zugriffs
    WERTEBEREICH: Gültiger Zeitpunkt oder None
    STANDARDWERT: None
  - NAME: permissions
    TYP: Option<FilePermissions>
    BESCHREIBUNG: Berechtigungen
    WERTEBEREICH: Gültige FilePermissions oder None
    STANDARDWERT: None
  - NAME: is_hidden
    TYP: bool
    BESCHREIBUNG: Ob der Eintrag versteckt ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: is_symlink
    TYP: bool
    BESCHREIBUNG: Ob der Eintrag ein symbolischer Link ist
    WERTEBEREICH: {true, false}
    STANDARDWERT: false
  - NAME: symlink_target
    TYP: Option<PathBuf>
    BESCHREIBUNG: Ziel des symbolischen Links
    WERTEBEREICH: Gültiger Pfad oder None
    STANDARDWERT: None
INVARIANTEN:
  - path muss ein gültiger Pfad sein
  - name darf nicht leer sein
  - Wenn is_symlink true ist, muss symlink_target vorhanden sein
```

### 6.9 FileType

```
ENTITÄT: FileType
BESCHREIBUNG: Typ einer Datei
ATTRIBUTE:
  - NAME: file_type
    TYP: Enum
    BESCHREIBUNG: Typ
    WERTEBEREICH: {
      Regular,
      Directory,
      Symlink,
      BlockDevice,
      CharDevice,
      Fifo,
      Socket,
      Unknown
    }
    STANDARDWERT: Unknown
INVARIANTEN:
  - Keine
```

### 6.10 FilePermissions

```
ENTITÄT: FilePermissions
BESCHREIBUNG: Berechtigungen für eine Datei
ATTRIBUTE:
  - NAME: mode
    TYP: u32
    BESCHREIBUNG: Berechtigungsmodus
    WERTEBEREICH: [0, 0o7777]
    STANDARDWERT: 0o644
  - NAME: owner
    TYP: Option<String>
    BESCHREIBUNG: Eigentümer
    WERTEBEREICH: Gültiger Benutzername oder None
    STANDARDWERT: None
  - NAME: group
    TYP: Option<String>
    BESCHREIBUNG: Gruppe
    WERTEBEREICH: Gültiger Gruppenname oder None
    STANDARDWERT: None
INVARIANTEN:
  - mode muss im Bereich [0, 0o7777] liegen
```
