# SPEC-GLOSSARY-v1.0.0: NovaDE Glossar

```
SPEZIFIKATION: SPEC-GLOSSARY-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Dieses Dokument definiert ein zentrales Glossar für alle Begriffe, die in den NovaDE-Spezifikationen verwendet werden. Es dient als verbindliche Referenz für die einheitliche Terminologie über alle Spezifikationsdokumente hinweg und gewährleistet ein konsistentes Verständnis aller Konzepte.

## 2. Allgemeine Begriffe

### 2.1 Architektur und Struktur

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| NovaDE | Moderne Linux Desktop-Umgebung, implementiert in Rust | Projektbezeichnung |
| Schicht | Hauptebene der Architekturhierarchie (Core, Domain, System, UI) | Architektur |
| Modul | Funktionale Einheit innerhalb einer Schicht | Architektur |
| Komponente | Funktionale Einheit innerhalb eines Moduls | Architektur |
| Schnittstelle | Definierte Interaktionspunkte zwischen Komponenten oder Modulen | Architektur |
| Service | Komponente, die eine bestimmte Funktionalität über eine definierte Schnittstelle bereitstellt | Architektur |
| Trait | Rust-spezifisches Konzept für die Definition von Schnittstellen | Implementierung |
| Implementierung | Konkrete Umsetzung einer Schnittstelle | Implementierung |

### 2.2 Spezifikationsbegriffe

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| Spezifikation | Formale, maschinenlesbare Beschreibung einer Komponente, eines Moduls oder einer Schicht | Dokumentation |
| Anforderung | Funktionale oder nicht-funktionale Eigenschaft, die das System erfüllen muss | Dokumentation |
| Entität | Definierte Datenstruktur mit Attributen und Invarianten | Datenmodell |
| Attribut | Eigenschaft einer Entität | Datenmodell |
| Invariante | Bedingung, die immer erfüllt sein muss | Datenmodell |
| Zustandsautomat | Formale Beschreibung des Verhaltens eines Systems durch Zustände und Übergänge | Verhaltensmodell |
| Zustand | Definierter Systemzustand in einem Zustandsautomaten | Verhaltensmodell |
| Übergang | Wechsel zwischen Zuständen in einem Zustandsautomaten | Verhaltensmodell |

## 3. Schichtspezifische Begriffe

### 3.1 Kernschicht (Core Layer)

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| Fehlerbehandlung | Mechanismen zur Erkennung, Meldung und Behandlung von Fehlern | Core Layer |
| Konfiguration | Mechanismen zum Laden, Parsen und Zugreifen auf Konfigurationseinstellungen | Core Layer |
| Logging | Mechanismen zur Protokollierung von Ereignissen und Zuständen | Core Layer |
| Typen | Grundlegende Datenstrukturen für die Verwendung im gesamten System | Core Layer |
| Utilities | Hilfsfunktionen für allgemeine Aufgaben | Core Layer |

### 3.2 Domänenschicht (Domain Layer)

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| Theming | Verwaltung des Erscheinungsbilds und Stylings der Desktop-Umgebung | Domain Layer |
| Workspace | Virtueller Desktop zur Organisation von Fenstern | Domain Layer |
| Notification | Benachrichtigung an den Benutzer | Domain Layer |
| Global Settings | Desktop-weite Einstellungen und Zustandsverwaltung | Domain Layer |
| Window Policy | Richtlinien für die Fensterverwaltung | Domain Layer |
| AI Interaction | KI-gestützte Funktionen und Benutzereinwilligung | Domain Layer |
| Token | Designelement mit definiertem Wert (z.B. Farbe, Abstand) | Theming |
| Theme | Sammlung von Tokens, die das Erscheinungsbild definieren | Theming |

### 3.3 Systemschicht (System Layer)

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| Compositor | Wayland-Compositor für die Fensterverwaltung | System Layer |
| Input | Eingabegeräte und -ereignisse | System Layer |
| D-Bus | Inter-Prozess-Kommunikationssystem für Linux | System Layer |
| Audio | Audioverwaltung und -steuerung | System Layer |
| MCP | Model Context Protocol für die Kommunikation mit KI-Modellen | System Layer |
| Portals | XDG Desktop Portals für die Interaktion mit Anwendungen | System Layer |
| Power Management | Energieverwaltung und -steuerung | System Layer |
| Window Mechanics | Technische Aspekte der Fensterverwaltung | System Layer |

### 3.4 UI-Schicht (UI Layer)

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| Shell | Hauptkomponenten der Desktop-Umgebung (Panel, Dock, etc.) | UI Layer |
| Control Center | Anwendung zur Konfiguration der Desktop-Umgebung | UI Layer |
| Widget | Wiederverwendbare UI-Komponente | UI Layer |
| View | Darstellungskomponente für Daten | UI Layer |
| Controller | Komponente zur Steuerung der Benutzerinteraktion | UI Layer |
| Panel | Leiste am Bildschirmrand mit Statusinformationen und Schnellzugriff | Shell |
| Dock | Bereich für Anwendungsstarter und laufende Anwendungen | Shell |
| Workspace Switcher | Komponente zum Wechseln zwischen virtuellen Desktops | Shell |
| Notification Center | Komponente zur Anzeige und Verwaltung von Benachrichtigungen | Shell |

## 4. Technologiespezifische Begriffe

### 4.1 Wayland und Compositor

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| Wayland | Modernes Display-Server-Protokoll für Linux | Compositor |
| Surface | Grundlegende Darstellungseinheit in Wayland | Compositor |
| XDG Shell | Erweiterung des Wayland-Protokolls für Desktop-Fenster | Compositor |
| Layer Shell | Erweiterung des Wayland-Protokolls für Overlay-Fenster | Compositor |
| Output | Ausgabegerät (z.B. Monitor) in Wayland | Compositor |
| Seat | Abstraktion für Eingabegeräte in Wayland | Input |
| Buffer | Speicherbereich für Pixeldaten in Wayland | Compositor |
| Smithay | Rust-Framework für Wayland-Compositoren | Compositor |

### 4.2 GTK und UI

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| GTK | GIMP Toolkit, eine Bibliothek für die Erstellung von Benutzeroberflächen | UI Layer |
| Widget | UI-Element in GTK | UI Layer |
| Container | Widget, das andere Widgets enthalten kann | UI Layer |
| Signal | Ereignisbenachrichtigung in GTK | UI Layer |
| GObject | Objektsystem von GTK | UI Layer |
| CSS | Cascading Style Sheets, verwendet für das Styling von GTK-Widgets | Theming |
| Libadwaita | Moderne Widget-Bibliothek für GTK | UI Layer |
| GSettings | Konfigurationssystem für GTK-Anwendungen | UI Layer |

## 5. Protokolle und Kommunikation

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| D-Bus | Inter-Prozess-Kommunikationssystem für Linux | System Layer |
| MCP | Model Context Protocol für die Kommunikation mit KI-Modellen | System Layer |
| XDG Desktop Portal | Standardisierte Schnittstelle für Desktop-Funktionen | System Layer |
| PipeWire | Multimedia-Framework für Linux | Audio |
| Event | Nachricht über eine Zustandsänderung | Alle Schichten |
| Signal | Benachrichtigung über ein Ereignis | Alle Schichten |
| Broadcast | Verteilung einer Nachricht an mehrere Empfänger | Alle Schichten |
| Subscription | Anmeldung zum Empfang von Ereignissen | Alle Schichten |

## 6. Sicherheit und Fehlerbehandlung

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| Error | Fehlertyp in Rust | Alle Schichten |
| Result | Rückgabetyp für Operationen, die fehlschlagen können | Alle Schichten |
| Panic | Nicht behandelbarer Fehler in Rust | Alle Schichten |
| Fehlerbehandlung | Mechanismen zur Erkennung, Meldung und Behandlung von Fehlern | Alle Schichten |
| Fehlerkette | Verkettung von Fehlern zur Nachverfolgung der Ursache | Alle Schichten |
| Validierung | Überprüfung von Eingaben auf Gültigkeit | Alle Schichten |
| Sanitization | Bereinigung von Eingaben zur Vermeidung von Injection-Angriffen | Alle Schichten |
| Zugriffskontrollen | Mechanismen zur Beschränkung des Zugriffs auf Ressourcen | Alle Schichten |

## 7. Modalverben und Anforderungsstufen

| Begriff | Definition | Verwendungskontext |
|---------|------------|-------------------|
| MUSS | Zwingende Anforderung | Anforderungen |
| SOLLTE | Empfohlene Anforderung | Anforderungen |
| KANN | Optionale Anforderung | Anforderungen |
| VERBOTEN | Unzulässige Aktion | Anforderungen |
| ERFORDERLICH | Notwendige Bedingung | Anforderungen |
| OPTIONAL | Nicht notwendige, aber mögliche Bedingung | Anforderungen |
