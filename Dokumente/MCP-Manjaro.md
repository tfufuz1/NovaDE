# Planung und Spezifikation einer KI-gestützten Desktop-Sidebar für Manjaro Linux

## I. Einleitung

### Zweck

Dieses Dokument beschreibt den Entwurf und die Spezifikation für die Entwicklung einer neuartigen, KI-gesteuerten Desktop-Komponente für das Manjaro Linux-Betriebssystem. Das Kernziel ist die Schaffung eines intelligenten Assistenten, der als persistente Sidebar in die Desktop-Umgebung integriert ist. Die technologische Basis bilden C++, das Qt-Framework (insbesondere Qt 6), QML für die Benutzeroberfläche und Qt-Wayland für die nahtlose Integration in moderne Display-Server-Umgebungen.

### Vision

Die Vision ist eine transformative Benutzererfahrung, bei der ein stets präsenter KI-Assistent den Anwendern zur Seite steht. Dieser Assistent soll natürliche Sprache verstehen und darauf basierend Systemoperationen und Aktionen in Manjaro ausführen können. Dies umfasst das Starten von Anwendungen, die Verwaltung von Systemressourcen, die Abfrage von Informationen und die Interaktion mit Systemeinstellungen. Die Sidebar soll die Produktivität steigern und die Interaktion mit dem Manjaro-System intuitiver gestalten.

### Kerninnovation: Manjaro Control Protocol (MCP)

Ein zentrales Element dieses Projekts ist die Definition und Spezifikation des "Manjaro Control Protocol" (MCP). Dieses Protokoll dient als standardisierte Schnittstelle zwischen der KI (speziell dem Large Language Model, LLM) und der Systemsteuerungsschicht von Manjaro. Eine wesentliche Anforderung ist, dass das MCP so präzise und selbsterklärend definiert wird, dass ein LLM dessen Funktionsweise und Semantik _ausschließlich_ anhand der in diesem Bericht enthaltenen Spezifikation verstehen und korrekt anwenden kann, ohne auf externes Wissen, Trainingsdaten oder Internetzugriff angewiesen zu sein.

### Umfang des Berichts

Dieser Bericht deckt alle wesentlichen Aspekte der Planung und Spezifikation ab:

1. **Anforderungsanalyse:** Definition der Kernfunktionen und Interaktionen.
2. **Technologieintegration:** Untersuchung der Integration von Qt/QML und Qt-Wayland in Manjaro-Desktop-Umgebungen.
3. **Anwendungsarchitektur:** Entwurf der Softwarekomponenten und ihres Zusammenspiels.
4. **MCP-Spezifikation:** Detaillierte Definition des Kommunikationsprotokolls.
5. **LLM-Integration:** Strategien zur Einbindung eines LLM und Sicherstellung der MCP-Interpretierbarkeit.
6. **C++ Backend-Logik:** Details zur Implementierung der serverseitigen Logik.
7. **Sicherheitsaspekte:** Analyse potenzieller Risiken und Definition von Schutzmaßnahmen.
8. **Entwicklungs- & Testplan:** Grober Plan für Implementierung und Verifizierung.

### Zielgruppe

Dieses Dokument richtet sich an ein technisch versiertes Publikum, insbesondere an Softwarearchitekten, Systementwickler und Projektleiter, die an der Konzeption und Implementierung des beschriebenen Systems beteiligt sind. Es dient als detaillierte technische Grundlage für die Entwicklung.

## II. Anforderungsanalyse

Die erfolgreiche Entwicklung der KI-gestützten Sidebar erfordert eine klare Definition der funktionalen und nicht-funktionalen Anforderungen.

### A. Kernfunktionalität der Sidebar

- **Persistenz:** Die Sidebar muss als dauerhaftes Element der Desktop-Umgebung fungieren. Sie soll über virtuelle Desktops und Arbeitsbereiche hinweg sichtbar bleiben und eine konsistente Positionierung (z. B. am linken oder rechten Bildschirmrand) beibehalten. Dies erfordert eine tiefe Integration in die Shell-Protokolle des Wayland-Compositors, um sicherzustellen, dass die Sidebar korrekt positioniert wird und den benötigten Platz auf dem Bildschirm reserviert.
- **Benutzeroberfläche (UI):** Die UI, implementiert in QML, muss grundlegende Elemente zur Interaktion bereitstellen. Dazu gehören ein Eingabebereich für Anfragen in natürlicher Sprache, ein Ausgabebereich zur Darstellung der KI-Antworten und Ergebnisse sowie potenziell Statusindikatoren (z. B. für laufende Operationen oder Verbindungsstatus zum LLM).
- **Responsivität:** Die Benutzeroberfläche muss flüssig und reaktionsschnell sein. QML bietet hierfür die notwendigen Werkzeuge, um eine moderne und ansprechende User Experience zu gewährleisten, auch bei laufenden Hintergrundoperationen des Backends.

### B. Fähigkeiten des LLM

- **Verständnis natürlicher Sprache (NLU):** Das zugrundeliegende LLM muss in der Lage sein, Benutzeranfragen in natürlicher Sprache (initial Deutsch, mit potenzieller Erweiterbarkeit auf andere Sprachen) zu verarbeiten und deren Bedeutung zu erfassen.
- **Intentionerkennung:** Aus der Benutzeranfrage muss die Absicht (Intent) extrahiert werden. Beispiele für Intents sind das Öffnen einer Anwendung, das Abfragen von Systeminformationen oder das Ändern einer Einstellung.
- **MCP-Befehlsgenerierung:** Dies ist ein kritischer Schritt. Das LLM muss die erkannte Absicht und die extrahierten Parameter (z. B. Anwendungsname, Dateipfad, Lautstärkepegel) in einen syntaktisch und semantisch korrekten MCP-Befehl im JSON-Format übersetzen. Die Fähigkeit des LLM, dies _allein_ auf Basis der MCP-Spezifikation (Abschnitt V) zu tun, ist eine Kernanforderung.
- **Antwortinterpretation:** Das LLM muss strukturierte MCP-Antworten (JSON-Format), die vom Backend zurückkommen, verarbeiten können. Dies kann bedeuten, Fehlercodes zu interpretieren oder erfolgreiche Ergebnisdaten in eine natürlichsprachliche Antwort für den Benutzer umzuwandeln.
- **Kontextuelles Bewusstsein (Optional, aber empfohlen):** Für eine natürlichere Interaktion wäre es wünschenswert, wenn das LLM den Gesprächskontext über mehrere Anfragen hinweg beibehalten könnte. Der initiale Fokus liegt jedoch auf der Verarbeitung einzelner, in sich geschlossener Anfragen, die über MCP abgebildet werden.

### C. Umfang der Systeminteraktion

Die KI soll über das MCP eine Reihe von Systemfunktionen in Manjaro steuern können. Der initiale Satz umfasst:

- **Anwendungsmanagement:** Starten von Anwendungen (`open_application`). Das Schließen von Anwendungen ist optional und erfordert zusätzliche Überlegungen bezüglich der Prozessidentifikation und Berechtigungen.
- **Dateisystemoperationen:** Auflisten von Dateien und Verzeichnissen in einem bestimmten Pfad (`list_files`). Grundlegende Dateioperationen (Kopieren, Verschieben, Löschen) sind denkbar, erfordern jedoch eine sehr sorgfältige Sicherheitsanalyse und Implementierung (siehe Abschnitt VIII).
- **Systemeinstellungen:**
    - Abfragen allgemeiner Systeminformationen (`query_system_info`), z. B. Betriebssystemversion, CPU-/Speicherauslastung, Batteriestatus.
    - Ändern der Systemlautstärke (`change_volume`).
    - Anpassen der Bildschirmhelligkeit (`set_brightness`) über dedizierte Tools wie `brightnessctl`.1
    - Modifizieren spezifischer Desktop-Einstellungen, die über `dconf`/`gsettings` (für GNOME/GTK-basierte Umgebungen) zugänglich sind (`modify_setting_dconf`).3 Eine äquivalente Funktionalität für KDE Plasma (KConfig) muss separat betrachtet werden.
- **Paketverwaltung:** Interaktion mit dem Pamac-Kommandozeilenwerkzeug (`pamac`) zum Suchen, Installieren und Entfernen von Paketen sowie zur Update-Verwaltung (`manage_packages_pamac`). Die unterstützten Aktionen müssen klar definiert werden, basierend auf den Fähigkeiten der Pamac-CLI.8
- **Zwischenablage:** Kopieren von Text in die Zwischenablage (`clipboard_copy`) und Einfügen von Text aus der Zwischenablage (`clipboard_paste`). Unter Wayland erfordert dies spezielle Werkzeuge wie `wl-clipboard`.12

### D. Interaktionsfluss

Der typische Ablauf einer Benutzerinteraktion ist wie folgt:

1. Der Benutzer gibt eine Anfrage in natürlicher Sprache in die QML-Sidebar ein.
2. Das QML-Frontend sendet die reine Textanfrage an das C++ Backend.
3. Das Backend leitet die Anfrage an das LLM-Integrationsmodul weiter.
4. Das LLM-Modul sendet die Anfrage an das LLM (lokal oder API).
5. Das LLM analysiert die Anfrage, erkennt die Absicht und generiert einen entsprechenden MCP-Befehl im JSON-Format.
6. Das LLM (oder das LLM-Modul) sendet den MCP-Befehl (als JSON-String) zurück an das Backend.
7. Der MCP Interface Handler im Backend empfängt und validiert den MCP-Befehl gegen die Spezifikation.
8. Bei Erfolg parst der Handler den Befehl und ruft die entsprechende Funktion in der System Interaction Layer auf, wobei die Parameter übergeben werden.
9. Die System Interaction Layer führt die angeforderte Systemaktion aus (z. B. Starten eines Prozesses via `QProcess`, Senden einer DBus-Nachricht via `QDBus`).
10. Die System Interaction Layer empfängt das Ergebnis, den Status oder einen Fehler von der Systemaktion.
11. Das Backend (MCP Interface Handler) formatiert das Ergebnis in eine MCP-Antwort (JSON-Format).
12. Das Backend sendet die MCP-Antwort entweder zurück an das LLM-Modul (zur Interpretation und Umwandlung in natürliche Sprache) oder direkt an das QML-Frontend.
13. Das QML-Frontend zeigt die finale Antwort oder das Ergebnis dem Benutzer an.

### E. Zentrale Randbedingung: MCP-Verständnis

Die entscheidende Anforderung ist, dass das LLM lernen muss, das MCP _ausschließlich_ auf Basis der in Abschnitt V dieses Dokuments bereitgestellten Spezifikation zu verwenden. Es darf kein Vorwissen über MCP oder Manjaro-spezifische Interna vorausgesetzt werden, und es darf kein externer Zugriff (z. B. Internet) zur Klärung benötigt werden. Dies stellt hohe Anforderungen an die Klarheit, Vollständigkeit und Eindeutigkeit der MCP-Spezifikation.

## III. Technologieintegrationsstrategie (Qt/QML & Wayland unter Manjaro)

Die Wahl der Technologien und deren Integration ist entscheidend für die Realisierung der persistenten Sidebar und ihrer Funktionalität unter Manjaro, insbesondere im Kontext von Wayland.

### A. Qt/QML Framework

- **Begründung:** Qt (Version 6 wird für die beste Wayland-Unterstützung empfohlen) wird als primäres Framework gewählt. Es bietet leistungsstarke C++-Bibliotheken, exzellente Werkzeuge und mit QML eine deklarative Sprache zur effizienten Entwicklung moderner Benutzeroberflächen.15 Obwohl Qt plattformübergreifend ist, liegt der Fokus hier klar auf Manjaro Linux.
- **QML für das Frontend:** Die Sidebar-UI wird vollständig in QML implementiert. Dies ermöglicht eine schnelle Entwicklung, einfache Anpassung des Erscheinungsbilds und die Nutzung von Qt Quick Controls für Standard-UI-Elemente.17 Die Logik im QML-Teil wird minimal gehalten und konzentriert sich auf die Präsentation und die Weiterleitung von Benutzeraktionen an das C++ Backend.
- **C++ für das Backend:** Die Kernlogik der Anwendung, die Kommunikation mit dem LLM, die Verarbeitung von MCP-Nachrichten und die gesamte Systeminteraktion werden in C++ implementiert. Dies gewährleistet die notwendige Performance, Robustheit und den Zugriff auf systemnahe APIs und Bibliotheken.16

### B. Wayland-Integration

- **Qt-Wayland Modul:** Die Basis für den Betrieb der Qt-Anwendung als nativer Wayland-Client bildet das `qt6-wayland` Paket.21 Dieses Modul stellt die notwendige Abstraktionsebene für die Kommunikation mit dem Wayland-Compositor bereit.
- **Implementierung der persistenten Sidebar:**
    - **Kernprotokoll:** Das `wlr-layer-shell-unstable-v1` Protokoll ist der De-facto-Standard für die Erstellung von Desktop-Shell-Komponenten wie Panels, Docks und Sidebars unter Wayland-Compositors, die dieses Protokoll unterstützen.22 Dazu gehören Compositors, die auf `wlroots` basieren (z. B. Sway) und auch KWin (KDE Plasma).
    - **Wichtige `wlr-layer-shell` Merkmale 22:**
        - _Anchoring (Verankerung):_ Erlaubt das Festlegen der Sidebar an einem oder mehreren Bildschirmrändern (z. B. `left` oder `right`, optional auch `top` und `bottom` für volle Höhe).
        - _Layering (Ebenen):_ Weist die Sidebar einer bestimmten Ebene zu (z. B. `top` oder `overlay`), um ihre Sichtbarkeit relativ zu anderen Anwendungsfenstern zu steuern.
        - _Exclusive Zone (Exklusivbereich):_ Ermöglicht der Sidebar, einen Bereich des Bildschirms für sich zu reservieren, sodass maximierte Fenster diesen Bereich nicht überlappen. Dies ist entscheidend für eine persistente Sidebar.
        - _Keyboard Interactivity (Tastaturinteraktivität):_ Steuert, ob und wie die Sidebar Tastatureingaben empfangen kann. Der Modus `on_demand` ist typischerweise für interaktive Elemente wie eine Sidebar geeignet, die Texteingaben ermöglichen soll.
    - **Qt-Integrationsbibliothek:** Um die Nutzung von `wlr-layer-shell` aus einer Qt-Anwendung heraus zu vereinfachen, wird die Verwendung der `layer-shell-qt` Bibliothek empfohlen.23 Diese Bibliothek, ein KDE-Projekt, stellt die Klasse `LayerShellQt::Window` bereit, mit der die spezifischen Eigenschaften einer Layer-Shell-Oberfläche (Layer, Anker, Exklusivzone etc.) für ein `QWindow` verwaltet werden können. Die Verwendung dieser Bibliothek ist deutlich einfacher als die direkte Interaktion mit Wayland-Protokollen über die Qt Wayland Compositor APIs 24, welche primär für die Entwicklung von Compositors selbst gedacht sind.
    - **Technische Abwägung:** Die Analyse der verfügbaren Technologien 22 zeigt klar, dass `wlr-layer-shell` das geeignete Protokoll für die geforderte persistente Sidebar ist. Die Existenz von `layer-shell-qt` als dedizierte Client-Bibliothek für Qt vereinfacht die Implementierung erheblich. Daher ist dies der bevorzugte Ansatz.

### C. Kompatibilität mit Desktop-Umgebungen (Manjaro)

Die nahtlose Integration der Sidebar hängt stark von der verwendeten Desktop-Umgebung und deren Wayland-Unterstützung ab.

- **KDE Plasma:**
    - **Compositor:** KWin ist der Wayland-Compositor von Plasma.26 KWin's Wayland-Unterstützung gilt als ausgereift 27 und unterstützt das `wlr-layer-shell` Protokoll.
    - **Integration:** Da `layer-shell-qt` ein KDE-Projekt ist 23 und KWin das zugrundeliegende Protokoll unterstützt, ist eine gute Kompatibilität und eine vergleichsweise reibungslose Integration zu erwarten. Die Wayland-spezifische Integration in Qt-Anwendungen wird durch Komponenten wie `kwayland-integration` (für Qt5) bzw. dessen Nachfolger in `kwindowsystem` (für Qt6) unterstützt.29
    - **Strategische Implikation:** KDE Plasma stellt aufgrund der technologischen Nähe (Qt) und der Unterstützung des Schlüsselprotokolls (`wlr-layer-shell`) durch KWin den wahrscheinlichsten Pfad für eine erfolgreiche und vollständige Implementierung der Sidebar dar. Die Entwicklung sollte initial auf Plasma abzielen.
- **GNOME:**
    - **Compositor:** Mutter ist der Wayland-Compositor für GNOME.21
    - **Integrationsherausforderungen:** Mutter unterstützt das `wlr-layer-shell` Protokoll _nicht_ nativ.21 GNOME verwendet eigene Mechanismen für Panels und Docks, die oft als GNOME Shell Extensions implementiert sind. Historisch gab es Kompatibilitätsprobleme zwischen Mutter und Nicht-GTK-Wayland-Anwendungen 32, und Regressionen mit QtWayland wurden beobachtet.33 Zudem fehlt Mutter unter Wayland die Unterstützung für Server-Side Decorations (SSD), was das Erscheinungsbild von Qt-Anwendungen beeinflussen kann, da diese dann Client-Side Decorations (CSD) zeichnen müssen.31
    - **Mögliche Lösungsansätze:**
        1. _GNOME Shell Extension:_ Entwicklung einer separaten Erweiterung, die die QML-Sidebar hostet oder mit ihr interagiert. Dies ist komplex und erfordert Kenntnisse in JavaScript/GJS und der GNOME Shell Extension API.
        2. _Standard-Fenster:_ Ausführung der Sidebar als reguläres Wayland-Fenster. Die Persistenz, Positionierung und das Reservieren von Platz müssten programmatisch (und potenziell unzuverlässig) über Standard-Wayland-Fensterverwaltung versucht werden.
        3. _Abwarten auf Mutter-Entwicklung:_ Beobachten, ob zukünftige Mutter-Versionen relevante Protokolle unterstützen.30 Dies ist kurzfristig unwahrscheinlich für `wlr-layer-shell`.
    - **Strategische Implikation:** Die Integration in GNOME stellt eine erhebliche Herausforderung dar. Ohne `wlr-layer-shell`-Unterstützung 21 wird die Sidebar wahrscheinlich nicht die gewünschte Persistenz und Platzreservierung erreichen, es sei denn, es wird erheblicher Zusatzaufwand betrieben (z. B. Extension-Entwicklung). Es muss akzeptiert werden, dass die Funktionalität unter GNOME möglicherweise eingeschränkt ist oder eine abweichende Implementierungsstrategie erfordert.
- **XFCE:**
    - **Wayland-Status:** Die Umstellung von XFCE auf Wayland ist ein laufender Prozess. Standardmäßig könnte Manjaro XFCE noch X11 verwenden, wo Persistenz über Fenstermanager-Hints realisiert wird. Wenn XFCE unter Wayland läuft (z. B. über `xfce4-session-wayland`), hängt die Unterstützung für `wlr-layer-shell` vom verwendeten Compositor ab. Viele Wayland-Implementierungen für XFCE setzen auf `wlroots`-basierte Compositors, die `wlr-layer-shell` unterstützen.
    - **Strategische Implikation:** Die Kompatibilität hängt vom Compositor ab. Bei Verwendung eines `wlroots`-basierten Compositors ist der `layer-shell-qt`-Ansatz gangbar. Unter X11 wären traditionelle Xlib-Methoden nötig. Der Fokus sollte zunächst auf den primären Wayland-DEs Plasma und GNOME liegen.

### D. Mechanismen zur Systeminteraktion

Das C++ Backend wird verschiedene Mechanismen nutzen, um mit dem Manjaro-System zu interagieren:

- **`QProcess`:** Zum Ausführen von Kommandozeilenwerkzeugen und Skripten. Dies ist der primäre Mechanismus für Interaktionen mit `pamac` 8, `brightnessctl` 1, `wl-clipboard` (`wl-copy`/`wl-paste`) 12 und `gsettings`.4 Erfordert sorgfältige Handhabung von Argumenten, Parsing der Ausgabe (stdout/stderr) und strikte Sicherheitsvorkehrungen (siehe Abschnitte VII und VIII).34
- **`QDBus`:** Zur Kommunikation mit Systemdiensten und Desktop-Daemons, die eine DBus-Schnittstelle anbieten.39 Anwendungsfälle sind z. B. die Steuerung der Lautstärke (über PulseAudio/PipeWire), das Senden von Benachrichtigungen oder die Interaktion mit Energieverwaltungsdiensten (z. B. `org.gnome.SettingsDaemon.Power` 44 oder KDE-Äquivalente).
- **`dconf`/`gsettings`:** Zum Lesen und Schreiben von Konfigurationseinstellungen von GNOME/GTK-Anwendungen, die in der dconf-Datenbank gespeichert sind. Der Zugriff erfolgt am sichersten über das `gsettings`-Kommandozeilenwerkzeug (via `QProcess`), da dieses Schema-Validierungen durchführt.3 Für KDE-Einstellungen (KConfig) sind andere Mechanismen erforderlich (wahrscheinlich DBus oder direkte Konfigurationsdatei-Interaktion).
- **Direkter Datei-/API-Zugriff:** Für spezifische Low-Level-Informationen, wie z. B. das Lesen von Helligkeitswerten aus `/sys/class/backlight/` 2, obwohl die Verwendung von `brightnessctl` vorzuziehen ist. Erfordert sorgfältige Prüfung der Berechtigungen und Fehlerbehandlung.

## IV. Anwendungsarchitektur

Die Architektur der Anwendung folgt bewährten Praktiken für Qt/QML-Anwendungen und trennt klar zwischen Benutzeroberfläche, Anwendungslogik, LLM-Interaktion und Systeminteraktion.16

### A. Überblick

Die Architektur ist modular aufgebaut:

Code-Snippet

```
graph LR
    subgraph User Interface
        A
    end
    subgraph Backend (C++)
        B[Core Application Logic]
        C[LLM Integration Module]
        D[MCP Interface Handler]
        E
    end
    subgraph External Systems
        F
        G
    end

    A -- User Input --> B
    B -- Query --> C
    C -- Query --> F
    F -- MCP Command (JSON) --> C
    C -- MCP Command (JSON) --> D
    D -- Parsed Command --> E
    E -- System Call --> G
    G -- System Result/Error --> E
    E -- Result/Error --> D
    D -- MCP Response (JSON) --> B
    B -- Response Data/Formatted Response --> A
    A -- Display Output --> User

    D -- Validation Failure --> B  // Error path
```

_Diagramm-Beschreibung:_ Das Diagramm zeigt die Hauptkomponenten: QML Frontend, C++ Backend (unterteilt in Kernlogik, LLM-Modul, MCP-Handler, Systeminteraktionsschicht), LLM Service und Manjaro System. Pfeile illustrieren den Datenfluss von der Benutzereingabe über die Verarbeitung im Backend und LLM bis zur Systemaktion und der finalen Ausgabe.

### B. QML Frontend (Sidebar UI)

- **Verantwortlichkeiten:** Rendern der Sidebar-Oberfläche, Erfassen der Benutzereingabe (Text), Anzeigen von KI-Antworten und Statusinformationen, Handhabung von UI-Animationen und Übergängen.
- **Implementierung:** Hauptsächlich deklaratives QML, eventuell unter Verwendung von Qt Quick Controls für Standardelemente.17 Die Logik beschränkt sich auf Präsentationsaspekte und die Delegation von Aktionen an das C++ Backend.
- **Kommunikation:** Interagiert mit dem C++ Backend über Qt's Signal-Slot-Mechanismus und durch Zugriff auf C++-Objekte und deren Eigenschaften (`Q_PROPERTY`), die dem QML-Kontext bekannt gemacht werden.19

### C. C++ Backend

Das Backend ist das Herzstück der Anwendung und beherbergt die Kernlogik und die Schnittstellen zu externen Systemen.

- **1. Core Application Logic:**
    - Verwaltet den globalen Zustand der Anwendung.
    - Orchestriert die Kommunikation zwischen dem Frontend, dem LLM-Modul und der Systeminteraktionsschicht.
    - Initialisiert die Anwendung und macht die notwendigen C++-Objekte (insbesondere den MCP Interface Handler oder ein übergeordnetes Controller-Objekt) dem QML-Kontext zugänglich, z. B. über `QQmlContext::setContextProperty()`.19
- **2. LLM Integration Module:**
    - **Verantwortlichkeiten:** Kapselt die gesamte Logik für die Kommunikation mit dem ausgewählten LLM (ob lokal oder über eine API). Sendet die Benutzeranfragen (als Text) an das LLM und empfängt die generierten MCP-Befehle (als JSON-String). Optional kann es auch MCP-Antworten vom Backend an das LLM senden, um diese in natürliche Sprache formatieren zu lassen.
    - **Schnittstelle:** Definiert eine klare C++-Schnittstelle (z. B. eine Klasse mit Signalen und Slots) für das Senden von Anfragen und das Empfangen von strukturierten MCP-Befehls-Strings.
- **3. MCP Interface Handler:**
    - **Verantwortlichkeiten:** Nimmt die MCP-Befehls-JSON-Strings vom LLM-Modul entgegen. Validiert die JSON-Struktur und die Syntax des Befehls rigoros gegen die MCP-Spezifikation (Abschnitt V). Parst valide Befehle und leitet sie an die System Interaction Layer weiter. Empfängt strukturierte Ergebnisse oder Fehler von der System Interaction Layer und formatiert diese in MCP-Antwort-JSON-Strings.
    - **Implementierung:** Eine C++-Klasse, die JSON-Parsing (z. B. mit `QJsonDocument`, `QJsonObject`, `QJsonArray`) und die gesamte Validierungslogik gemäß der MCP-Spezifikation implementiert.
- **4. System Interaction Layer:**
    - **Verantwortlichkeiten:** Führt die konkreten Systemaktionen aus, die durch die geparsten MCP-Befehle spezifiziert wurden. Interagiert mit dem Manjaro-System über die geeigneten Mechanismen (`QProcess`, `QDBus`, `gsettings`-Aufrufe, Dateisystemzugriffe etc.). Kapselt die Details der jeweiligen Systeminteraktion, behandelt Fehler auf Systemebene und liefert standardisierte Ergebnisse oder Fehlercodes an den MCP Interface Handler zurück.
    - **Implementierung:** Modulare Struktur mit separaten C++-Klassen oder Funktionsgruppen für jeden Interaktionstyp (z. B. `PamacManager`, `SettingsManager`, `ProcessRunner`, `ClipboardManager`, `DBusInterface`). Diese Schicht abstrahiert die Komplexität der Systemaufrufe vom Rest des Backends.

### D. Best Practices für die Architektur

- **Trennung der Belange (Separation of Concerns):** Strikte Trennung zwischen der UI-Logik (QML) und der Backend-/Geschäftslogik (C++).16 Das QML-Frontend sollte "dumm" sein und nur Daten anzeigen und Benutzerereignisse weiterleiten.
- **Model-View(-Controller/Delegate):** Anwendung von MVC-, MVVM- oder ähnlichen Mustern, wo immer Daten aus dem Backend in der UI dargestellt werden. C++-Datenmodelle (abgeleitet von `QAbstractListModel` etc.) oder Kontext-Properties (`Q_PROPERTY`) werden dem QML-Frontend zur Verfügung gestellt.19 Änderungen im Backend werden über Signale an das Frontend gemeldet, das sich daraufhin aktualisiert.
- **Asynchrone Operationen:** Alle potenziell blockierenden Operationen – insbesondere Netzwerkaufrufe zum LLM, das Starten und Warten auf externe Prozesse mit `QProcess` 34 und DBus-Aufrufe – müssen asynchron implementiert werden, um ein Einfrieren der Benutzeroberfläche zu verhindern. Qt's Signal-Slot-Mechanismus ist hierfür das zentrale Werkzeug.

## V. Manjaro Control Protocol (MCP) Spezifikation

Das Manjaro Control Protocol (MCP) ist die definierte Schnittstelle, über die das LLM Systemaktionen anfordert und Ergebnisse empfängt. Die folgende Spezifikation ist darauf ausgelegt, von einem LLM ohne externes Wissen verstanden zu werden.

### A. Zweck und Designziele

- **Zweck:** Bereitstellung einer standardisierten, eindeutigen und maschinenlesbaren Schnittstelle, die es einer KI/einem LLM ermöglicht, spezifische Systemaktionen unter Manjaro Linux anzufordern und strukturierte Ergebnisse zu erhalten.
- **LLM-Interpretierbarkeit:** Explizit entworfen, um von einem LLM _allein_ auf Basis dieser Spezifikation verstanden und genutzt zu werden. Dies erfordert höchste Klarheit, explizite Definitionen aller Elemente und eine in sich geschlossene Beschreibung.
- **Plattformspezifität:** Zugeschnitten auf Manjaro Linux, unter Berücksichtigung spezifischer Werkzeuge (`pamac`), Konfigurationsmechanismen (`dconf`/`gsettings`) und Systempfade/Dienste.
- **Erweiterbarkeit:** Die Struktur (JSON-basiert, klare Befehlsdefinition) ermöglicht die zukünftige Ergänzung neuer Befehle, ohne die bestehende Struktur zu brechen.
- **Sicherheit:** Das Format unterstützt die Validierung und Bereinigung von Befehlen und Parametern durch das Backend, bevor eine Ausführung stattfindet.

### B. Nachrichtenformat

- **Transport:** JSON-Objekte werden sowohl für Anfragen (LLM -> Backend) als auch für Antworten (Backend -> LLM/Frontend) verwendet.
    
- **Anfragestruktur (Request):**
    
    JSON
    
    ```
    {
      "mcp_version": "1.0",
      "request_id": "string",
      "command": "string",
      "parameters": {
        "param1_name": "value1", // Typ: string | integer | boolean | array[string] | object
        "param2_name": "value2",
        //... weitere Parameter
      }
    }
    ```
    
    - `mcp_version` (string, erforderlich): Die Version des MCP-Protokolls, die verwendet wird (z. B. "1.0"). Dies ermöglicht zukünftige Versionierung.
    - `request_id` (string, erforderlich): Ein eindeutiger Identifikator für diese spezifische Anfrage, generiert vom anfragenden System (LLM-Modul). Wird verwendet, um Antworten der entsprechenden Anfrage zuzuordnen.
    - `command` (string, erforderlich): Der Name der auszuführenden Aktion (z. B. `open_application`, `query_system_info`). Muss exakt einem der im Core Command Set definierten Befehle entsprechen.
    - `parameters` (object, erforderlich): Ein JSON-Objekt, das die für den spezifischen `command` benötigten Parameter als Schlüssel-Wert-Paare enthält. Die Namen, Datentypen (string, integer, boolean, array von strings, etc.) und die Erforderlichkeit (required: true/false) jedes Parameters sind für jeden Befehl streng definiert (siehe Core Command Set).
- **Antwortstruktur (Response):**
    
    JSON
    
    ```
    {
      "mcp_version": "1.0",
      "request_id": "string",
      "status": "string", // "success" oder "error"
      "data": {... }, // Optional: Nur bei status="success"
      "error": {         // Optional: Nur bei status="error"
        "code": "string",
        "message": "string"
      }
    }
    ```
    
    - `mcp_version` (string, erforderlich): Die Version des MCP-Protokolls (z. B. "1.0").
    - `request_id` (string, erforderlich): Der eindeutige Identifikator aus der korrespondierenden Anfrage.
    - `status` (string, erforderlich): Gibt an, ob die Ausführung des Befehls erfolgreich war (`"success"`) oder fehlgeschlagen ist (`"error"`).
    - `data` (object, optional): Ein JSON-Objekt, das die Ergebnisse des Befehls enthält, falls `status` `"success"` ist. Die Struktur dieses Objekts hängt vom ausgeführten Befehl ab (z. B. eine Liste von Dateien, abgefragte Systeminformationen, eine Bestätigungsnachricht). Dieses Feld ist nur vorhanden, wenn `status` `"success"` ist.
    - `error` (object, optional): Ein JSON-Objekt, das nur vorhanden ist, wenn `status` `"error"` ist.
        - `code` (string, erforderlich): Ein vordefinierter Fehlercode-String, der die Art des Fehlers klassifiziert (z. B. `INVALID_COMMAND`, `PERMISSION_DENIED`, `EXECUTION_FAILED`, `TIMEOUT`, `INVALID_PARAMETER`). Eine Liste der Standard-Fehlercodes befindet sich am Ende dieses Abschnitts.
        - `message` (string, erforderlich): Eine menschenlesbare Beschreibung des Fehlers, primär für Logging- und Debugging-Zwecke. Diese Nachricht sollte vom LLM interpretiert werden, bevor sie dem Endbenutzer angezeigt wird.

### C. Definition des Kernbefehlssatzes (Core Command Set)

Die folgende Tabelle definiert die initialen Befehle, die das MCP unterstützt. Das LLM muss in der Lage sein, aus natürlicher Sprache auf diese Befehle zu schließen und die Anfragen gemäß den hier definierten Parametern zu strukturieren.

**Tabelle: MCP Core Commands (Version 1.0)**

|   |   |   |   |   |
|---|---|---|---|---|
|**Command Name (string)**|**Description**|**Parameters (object: {name: {type, required, description}})**|**Success Data Structure (object)**|**Potential Error Codes (array[string])**|
|`open_application`|Startet eine Desktop-Anwendung.|`{"name": {"type": "string", "required": true, "description": "Name oder ausführbarer Pfad der Anwendung (z.B. 'firefox', '/usr/bin/gimp')."}}`|`{"pid": {"type": "integer", "description": "Prozess-ID der gestarteten Anwendung (optional, falls ermittelbar)"}, "message": {"type": "string", "description": "Bestätigungsnachricht, z.B. 'Anwendung [Name] gestartet.'"}}`|`EXECUTION_FAILED`, `APP_NOT_FOUND`, `INVALID_PARAMETER`|
|`list_files`|Listet Dateien und Verzeichnisse in einem Pfad auf.|`{"path": {"type": "string", "required": true, "description": "Absoluter Pfad zum Verzeichnis (muss mit '/' beginnen). Symbolische Links werden nicht aufgelöst."}}`|`{"path": {"type": "string", "description": "Der abgefragte Pfad"}, "entries": {"type": "array", "items": {"type": "object", "properties": {"name": {"type": "string"}, "type": {"type": "string", "enum": ["file", "directory"]}, "size_bytes": {"type": "integer", "description": "Dateigröße in Bytes (nur für Typ 'file')"}}}}, "description": "Liste der Einträge im Verzeichnis."}}`|`PATH_NOT_FOUND`, `PERMISSION_DENIED`, `INVALID_PARAMETER`, `FILESYSTEM_ERROR`|
|`change_volume`|Stellt die Systemlautstärke ein oder passt sie an.|`{"level": {"type": "integer", "required": false, "description": "Absoluter Lautstärkepegel in Prozent (0-100)."}, "change": {"type": "integer", "required": false, "description": "Relative Änderung in Prozentpunkten (+/-). 'level' und 'change' schließen sich gegenseitig aus."}}`|`{"new_level": {"type": "integer", "description": "Der resultierende Lautstärkepegel in Prozent (0-100)."}}`|`INVALID_PARAMETER`, `EXECUTION_FAILED`, `DBUS_ERROR` (falls DBus verwendet)|
|`query_system_info`|Ruft spezifische Systeminformationen ab.|`{"query": {"type": "string", "required": true, "description": "Art der angeforderten Information. Gültige Werte: 'os_version', 'hostname', 'cpu_usage' (als Prozentwert), 'memory_total_mb', 'memory_available_mb', 'memory_usage' (als Prozentwert), 'battery_status' (als Objekt mit 'percentage', 'charging_status' [boolean]), 'uptime_seconds'."}}`|`{"query": {"type": "string", "description": "Die gestellte Abfrage"}, "value": {"type": "string|integer|
|`manage_packages_pamac`|Interagiert mit dem Pamac CLI zur Paketverwaltung.9|`{"action": {"type": "string", "required": true, "enum": ["search", "install", "remove", "update_check", "update_all", "list_installed", "list_orphans", "remove_orphans"], "description": "Die auszuführende Pamac-Aktion."}, "package_name": {"type": "string", "required": false, "description": "Ziel-Paketname (erforderlich für 'install', 'remove', 'search')."}, "include_aur": {"type": "boolean", "required": false, "default": false, "description": "AUR in die Aktion einbeziehen ('search', 'update_check', 'update_all')."}}`|Hängt von `action` ab: `search`: `{"results": array[{"name": string, "version": string, "repository": string, "description": string}]}`. `install`/`remove`: `{"message": string}`. `update_check`: `{"updates_available": boolean, "packages": array[string]}`. `update_all`: `{"message": string}`. `list_installed`/`list_orphans`: `{"packages": array[string]}`. `remove_orphans`: `{"message": string}`.|`PAMAC_ERROR`, `INVALID_ACTION`, `INVALID_PARAMETER`, `PACKAGE_NOT_FOUND`, `PERMISSION_DENIED`, `EXECUTION_FAILED`|
|`modify_setting_dconf`|Ändert eine dconf-Einstellung (primär für GNOME/GTK) via `gsettings`.4|`{"schema": {"type": "string", "required": true, "description": "Das GSettings-Schema (z.B. 'org.gnome.desktop.interface')."}, "key": {"type": "string", "required": true, "description": "Der Schlüssel innerhalb des Schemas (z.B. 'gtk-theme')."}, "value": {"type": "string|integer|boolean", "required": true, "description": "Der neue Wert für den Schlüssel. Muss dem Typ des Schlüssels im Schema entsprechen."}}`|
|`set_brightness`|Passt die Bildschirmhelligkeit an via `brightnessctl`.1|`{"level": {"type": "integer", "required": true, "description": "Absoluter Helligkeitspegel in Prozent (0-100)."}}`|`{"new_level": {"type": "integer", "description": "Der resultierende Helligkeitspegel in Prozent (0-100)."}}`|`INVALID_PARAMETER`, `EXECUTION_FAILED`, `BRIGHTNESS_CONTROL_ERROR`|
|`clipboard_copy`|Kopiert Text in die System-Zwischenablage via `wl-copy`.12|`{"text": {"type": "string", "required": true, "description": "Der zu kopierende Text."}}`|`{"message": {"type": "string", "description": "Text erfolgreich kopiert."}}`|`EXECUTION_FAILED`, `CLIPBOARD_ERROR`|
|`clipboard_paste`|Ruft Text aus der System-Zwischenablage ab via `wl-paste`.12|`{}` (Keine Parameter benötigt)|`{"text": {"type": "string", "description": "Der Text aus der Zwischenablage."}}`|`EXECUTION_FAILED`, `CLIPBOARD_EMPTY`, `CLIPBOARD_ERROR`|

_Anmerkung zur LLM-Interpretierbarkeit:_ Die `description`-Felder in der Tabelle sind entscheidend. Sie liefern dem LLM den notwendigen Kontext, um die Parameter korrekt zu interpretieren und zu befüllen (z. B. was unter `os_version` bei `query_system_info` zu verstehen ist oder welche Werte für `action` bei `manage_packages_pamac` gültig sind). Die `enum`-Angabe bei `action` und `type` (in `list_files`) schränkt die möglichen Werte explizit ein.

### D. Kommunikationsfluss

1. **Anfragegenerierung:** Das LLM empfängt die natürlichsprachliche Anfrage des Benutzers, analysiert sie und identifiziert den passenden MCP-Befehl sowie die erforderlichen Parameter gemäß der obigen Tabelle. Es konstruiert das MCP Request JSON-Objekt, inklusive einer eindeutigen `request_id`.
2. **Anfrageübermittlung:** Das LLM-Modul sendet den JSON-String an den MCP Interface Handler im C++ Backend.
3. **Validierung im Backend:** Der MCP Interface Handler parst den JSON-String. Er überprüft die `mcp_version`, die Gültigkeit des `command`-Namens und ob alle erforderlichen `parameters` vorhanden sind und den korrekten Datentyp haben. Bei Fehlern wird sofort eine MCP Error Response generiert und zurückgesendet.
4. **Dispatching:** Bei erfolgreicher Validierung ruft der MCP Interface Handler die zuständige Methode in der System Interaction Layer auf und übergibt die extrahierten und validierten Parameter.
5. **Systemaktion:** Die System Interaction Layer führt die Aktion aus (z. B. Starten eines `QProcess`, Senden einer `QDBus`-Nachricht). Dies geschieht asynchron.
6. **Ergebnisverarbeitung:** Nach Abschluss der Systemaktion (erfolgreich oder fehlerhaft) meldet die System Interaction Layer das Ergebnis (Daten oder Fehlercode/-nachricht) an den MCP Interface Handler zurück.
7. **Antwortgenerierung:** Der MCP Interface Handler konstruiert das MCP Response JSON-Objekt. Er füllt `request_id` (aus der Anfrage), `status` (`success` oder `error`) und entweder das `data`-Objekt (bei Erfolg) oder das `error`-Objekt (bei Fehler) gemäß der Spezifikation.
8. **Antwortübermittlung:** Der JSON-String der Antwort wird zurück an das LLM-Modul oder direkt an das Frontend gesendet.

### E. Konkrete Beispiele (Request/Response Paare)

- **Beispiel 1: Firefox starten**
    - Request:
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-123",
          "command": "open_application",
          "parameters": {
            "name": "firefox"
          }
        }
        ```
        
    - Response (Success):
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-123",
          "status": "success",
          "data": {
            "pid": 12345,
            "message": "Anwendung firefox gestartet."
          }
        }
        ```
        
- **Beispiel 2: Dateien im Home-Verzeichnis auflisten**
    - Request:
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-124",
          "command": "list_files",
          "parameters": {
            "path": "/home/user"
          }
        }
        ```
        
    - Response (Success):
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-124",
          "status": "success",
          "data": {
            "path": "/home/user",
            "entries": [
              {"name": "Documents", "type": "directory"},
              {"name": "image.jpg", "type": "file", "size_bytes": 102400},
              {"name": ".bashrc", "type": "file", "size_bytes": 3500}
            ]
          }
        }
        ```
        
- **Beispiel 3: Pamac nach 'gimp' durchsuchen (inkl. AUR)**
    - Request:
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-125",
          "command": "manage_packages_pamac",
          "parameters": {
            "action": "search",
            "package_name": "gimp",
            "include_aur": true
          }
        }
        ```
        
    - Response (Success):
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-125",
          "status": "success",
          "data": {
            "results":
          }
        }
        ```
        
- **Beispiel 4: Helligkeit auf 75% setzen**
    - Request:
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-126",
          "command": "set_brightness",
          "parameters": {
            "level": 75
          }
        }
        ```
        
    - Response (Success):
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-126",
          "status": "success",
          "data": {
            "new_level": 75
          }
        }
        ```
        
- **Beispiel 5: Fehler beim Installieren eines nicht existierenden Pakets**
    - Request:
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-127",
          "command": "manage_packages_pamac",
          "parameters": {
            "action": "install",
            "package_name": "nonexistent_package_xyz"
          }
        }
        ```
        
    - Response (Error):
        
        JSON
        
        ```
        {
          "mcp_version": "1.0",
          "request_id": "req-127",
          "status": "error",
          "error": {
            "code": "PACKAGE_NOT_FOUND",
            "message": "Pamac Fehler: Ziel nicht gefunden: nonexistent_package_xyz"
          }
        }
        ```
        

### F. Fehlerbehandlung und Fehlercodes

Eine robuste Fehlerbehandlung ist essenziell. Das Backend muss Fehler auf verschiedenen Ebenen abfangen und in standardisierte MCP-Fehlercodes übersetzen.

- **Standard-Fehlercodes:**
    - `INVALID_COMMAND`: Der angegebene `command` ist nicht im MCP definiert.
    - `INVALID_PARAMETER`: Ein oder mehrere Parameter sind ungültig (falscher Typ, fehlender erforderlicher Parameter, ungültiger Wert, z. B. Pfad existiert nicht, wo erwartet).
    - `PERMISSION_DENIED`: Die Aktion erfordert höhere Berechtigungen, die der Backend-Prozess nicht hat.
    - `EXECUTION_FAILED`: Ein externer Prozess (`QProcess`) konnte nicht gestartet werden oder ist mit einem Fehler abgestürzt.
    - `TIMEOUT`: Eine Operation hat das Zeitlimit überschritten.
    - `APP_NOT_FOUND`: Die zu startende Anwendung wurde nicht gefunden.
    - `PATH_NOT_FOUND`: Ein angegebener Datei- oder Verzeichnispfad existiert nicht.
    - `FILESYSTEM_ERROR`: Allgemeiner Fehler bei Dateisystemoperationen.
    - `DBUS_ERROR`: Fehler bei der Kommunikation über DBus.
    - `PAMAC_ERROR`: Spezifischer Fehler bei der Interaktion mit Pamac CLI.
    - `GSETTINGS_ERROR`: Spezifischer Fehler bei der Interaktion mit `gsettings` CLI.
    - `BRIGHTNESS_CONTROL_ERROR`: Spezifischer Fehler bei der Helligkeitssteuerung.
    - `CLIPBOARD_ERROR`: Allgemeiner Fehler bei der Interaktion mit der Zwischenablage.
    - `CLIPBOARD_EMPTY`: Versuch, aus einer leeren Zwischenablage zu lesen.
    - `INVALID_QUERY`: Der Wert für `query` in `query_system_info` ist ungültig.
    - `FAILED_TO_RETRIEVE`: Konnte die angeforderten Informationen nicht abrufen (`query_system_info`).
    - `LLM_ERROR`: Fehler bei der Kommunikation mit dem LLM oder bei der Verarbeitung durch das LLM.
    - `BACKEND_ERROR`: Interner Fehler im C++ Backend.
    - `UNKNOWN_ERROR`: Ein nicht klassifizierter Fehler ist aufgetreten.
- **Fehlermeldungen (`message`):** Sollten präzise genug für Entwickler-Debugging sein (z. B. die exakte Fehlermeldung von `stderr` eines `QProcess`), aber nicht unbedingt für die direkte Anzeige an den Benutzer gedacht. Das LLM kann beauftragt werden, diese technischen Meldungen in eine benutzerfreundlichere Form zu übersetzen.

## VI. LLM-Integrationsplan

Die Integration des Large Language Models (LLM) ist der Schlüssel zur Übersetzung natürlicher Sprache in MCP-Befehle und zur Interpretation der Ergebnisse. Die Strategie muss die Kernanforderung berücksichtigen, dass das LLM das MCP allein durch die Spezifikation in diesem Bericht verstehen muss.

### A. LLM-Auswahlkriterien

Die Wahl des geeigneten LLM hängt von mehreren Faktoren ab:

- **Function Calling / Strukturierte Ausgabe:** Dies ist das wichtigste Kriterium. Das LLM muss zuverlässig strukturierte Ausgaben, idealerweise im JSON-Format, generieren können, die exakt der MCP-Spezifikation entsprechen. Modelle mit expliziter "Function Calling" oder "Tool Calling" Fähigkeit sind zu bevorzugen.45 Benchmarks wie BFCL (Berkeley Function-Calling Leaderboard) 49 und APIBank 50 können bei der Bewertung helfen. Aktuelle Kandidaten sind Cloud-Modelle wie GPT-4o, Claude 3.5 Sonnet, Gemini 1.5 Flash 46 oder potenziell leistungsfähige lokale Modelle (z. B. Llama 3, Mistral, Qwen), die entweder speziell für Tool Use feinabgestimmt wurden oder deren Ausgabe durch Techniken wie Constrained Generation 53 auf das korrekte JSON-Format gezwungen wird. Die Pythonic-Ansätze 57 sind hier weniger relevant, da MCP auf JSON basiert.
- **Lokal vs. API:**
    - _API-basiert (z. B. OpenAI, Anthropic):_ Bietet oft höhere Genauigkeit und einfachere initiale Einrichtung der Function Calling-Fähigkeit.45 Nachteile sind die Abhängigkeit von einer Internetverbindung, laufende Kosten und potenzielle Datenschutzbedenken, da Benutzeranfragen an einen externen Dienst gesendet werden.
    - _Lokal (z. B. Ollama + Llama 3, llama.cpp + Mistral):_ Bietet maximale Privatsphäre, Offline-Fähigkeit und keine direkten API-Kosten.52 Erfordert jedoch signifikante lokale Hardware-Ressourcen (CPU, RAM, VRAM) und die Implementierung robuster Mechanismen zur Erzeugung strukturierter Ausgaben (Constrained Generation), da die Genauigkeit bei der reinen Befolgung von Formatierungsanweisungen im Prompt geringer sein kann als bei spezialisierten APIs.56
- **Leistung (Latenz/Genauigkeit):** Die Antwortzeit des LLM (Latenz) und die Genauigkeit bei der Generierung korrekter MCP-Befehle müssen gegeneinander abgewogen werden.58 Zu hohe Latenz beeinträchtigt die Benutzererfahrung.
- **Kosten:** API-Nutzungsgebühren oder die Anschaffungs- und Betriebskosten für die Hardware zum lokalen Betrieb müssen berücksichtigt werden.

### B. Integrationsstrategie

Die Integration erfolgt im LLM Integration Module des C++ Backends.

- **Prompt Engineering:** Ein sorgfältig gestalteter System-Prompt ist unerlässlich. Er muss dem LLM seine Rolle als Manjaro-Assistent erklären, die verfügbaren "Werkzeuge" (implizit durch die MCP-Befehle in Abschnitt V definiert) beschreiben und das exakte JSON-Format für Anfragen (MCP Request) vorgeben. Der Prompt muss klarstellen, dass die Ausgabe _nur_ im spezifizierten JSON-Format erfolgen darf.
- **Function Calling Mechanismus:**
    - _Bei Nutzung einer API mit nativer Unterstützung (z. B. OpenAI Tools API 48, Anthropic Tools):_ Die MCP-Befehle aus Abschnitt V werden in das spezifische Format der API für Funktions-/Werkzeugdefinitionen übersetzt (Name, Beschreibung, Parameter-Schema). Das LLM wird dann direkt von der API aufgefordert, das passende Werkzeug (MCP-Befehl) und die Argumente zu nennen. Das LLM Integration Module parst die API-Antwort und extrahiert den MCP-Befehl und die Parameter zur Weiterleitung an den MCP Interface Handler.
    - _Bei Nutzung lokaler Modelle oder APIs ohne native Unterstützung:_ Hier ist Prompt Engineering entscheidend. Der Prompt muss das LLM anweisen, direkt das vollständige MCP Request JSON zu generieren. Zusätzlich _muss_ im LLM Interface Layer eine Technik zur **Constrained Generation** implementiert werden. Dies stellt sicher, dass die Ausgabe des LLM syntaktisch korrektes JSON ist und dem in Abschnitt V definierten Schema entspricht. Bibliotheken und Frameworks wie `instructor` (Python) 53, `outlines` (Python) 56, JSON Schema in Ollama 53 oder die Grammatik-Funktion (GBNF) von `llama.cpp` 55 bieten solche Möglichkeiten. Diese Technik filtert oder steuert die Token-Generierung des LLM, sodass nur gültige Ausgaben gemäß dem Schema erzeugt werden.54 Ohne Constrained Generation ist die Wahrscheinlichkeit hoch, dass lokale Modelle vom geforderten Format abweichen.56
- **Antwortbehandlung:** Das LLM Integration Module empfängt die MCP Response JSON vom Backend. Abhängig von der gewünschten Benutzererfahrung kann diese JSON-Antwort entweder direkt (nach einfacher Formatierung) an das Frontend weitergegeben werden, oder sie wird erneut an das LLM gesendet mit der Aufforderung, eine natürlichsprachliche Zusammenfassung oder Erklärung für den Benutzer zu generieren (z. B. "Ich habe Firefox gestartet" statt nur `{"status": "success",...}`).

### C. Anforderung an das MCP-Verständnis

- **Strikte Vorgabe:** Es muss sichergestellt werden, dass die gesamte Logik der LLM-Integration davon ausgeht, dass das LLM _kein_ Vorwissen über MCP hat und _ausschließlich_ auf die Informationen in Abschnitt V dieses Berichts zugreift.
- **Ableitung aus Spezifikation:** Alle Prompts, Funktions-/Werkzeugdefinitionen oder Grammatiken, die dem LLM zur Verfügung gestellt werden, müssen direkt und nachvollziehbar aus der MCP-Spezifikation in Abschnitt V abgeleitet sein.
- **Verifizierung:** Eine kritische Testphase muss überprüfen, ob das ausgewählte LLM, wenn ihm die MCP-Spezifikation als Kontext gegeben wird (z. B. als Teil eines langen System-Prompts oder über die Werkzeugbeschreibung), in der Lage ist, korrekte MCP-JSON-Anfragen für diverse natürlichsprachliche Eingaben zu generieren, ohne auf externes Wissen zurückzugreifen.

Die Notwendigkeit, dass das LLM MCP allein aus diesem Bericht lernt, unterstreicht die Bedeutung einer exzellenten "Function Calling" bzw. "Structured Output"-Fähigkeit.45 Da Standard-Trainingsdaten MCP nicht enthalten, muss die Definition zur Laufzeit bereitgestellt werden. Das LLM muss dann zuverlässig die Abbildung von natürlicher Sprache auf den korrekten MCP-Befehl und dessen JSON-Struktur durchführen. Dies macht Modelle mit starker Instruktionsbefolgung und Format-Treue unerlässlich. Für lokale Modelle wird Constrained Generation 53 quasi zur Pflicht, um die strikte Einhaltung des MCP-Formats zu garantieren, was die Integration im Vergleich zu APIs mit eingebauter, zuverlässiger Funktion aufwändiger macht.

## VII. C++ Backend Implementierungsdetails

Das C++ Backend bildet die Brücke zwischen der QML-Oberfläche, dem LLM und dem Manjaro-System. Die Implementierung muss robust, sicher und asynchron sein.

### A. Verarbeitung von MCP-Nachrichten

Der MCP Interface Handler ist für die Entgegennahme, Validierung und Weiterleitung von MCP-Befehlen sowie die Erzeugung von MCP-Antworten zuständig.

- **Empfang:** Eine Funktion oder ein Slot (verbunden mit dem LLM Integration Module) empfängt den MCP-Befehl als JSON-String.
- **Validierung:**
    1. **JSON-Parsing:** Verwendung von `QJsonDocument::fromJson()`, um den String in ein JSON-Objekt zu parsen. Bei Parsing-Fehlern wird sofort eine `INVALID_PARAMETER` (oder spezifischer `JSON_PARSE_ERROR`) MCP-Antwort generiert.
    2. **Strukturprüfung:** Überprüfung auf das Vorhandensein und die korrekten Basistypen (string, object) der Top-Level-Felder: `mcp_version`, `request_id`, `command`, `parameters`.
    3. **Versionsprüfung:** Abgleich der `mcp_version` mit der vom Backend unterstützten Version.
    4. **Befehlsprüfung:** Überprüfung, ob der Wert von `command` einem der in Abschnitt V.C definierten Befehle entspricht. Bei unbekanntem Befehl: `INVALID_COMMAND` Fehler.
    5. **Parameterprüfung:** Detaillierte Validierung des `parameters`-Objekts basierend auf der Definition für den spezifischen `command` aus Abschnitt V.C: Sind alle erforderlichen Parameter vorhanden? Haben alle Parameter den korrekten Datentyp (string, integer, boolean, array[string])? Sind Enum-Werte gültig? Bei Fehlern: `INVALID_PARAMETER` Fehler mit spezifischer Meldung.
- **Dispatching:** Nach erfolgreicher Validierung wird die entsprechende Methode in der System Interaction Layer aufgerufen. Die validierten und typisierten Parameter werden dabei übergeben.
- **Antwortgenerierung:** Die Methode empfängt das Ergebnis (als Datenstruktur oder Objekt) oder einen Fehler (als Fehlercode und Nachricht) von der System Interaction Layer. Sie konstruiert das MCP Response JSON unter Verwendung von `QJsonObject`, `QJsonArray` etc. und `QJsonDocument::toJson()`. Die `request_id` aus der Anfrage wird übernommen, `status` wird auf `success` oder `error` gesetzt, und entsprechend wird das `data`- oder `error`-Objekt befüllt.

### B. Implementierung der System Interaction Layer

Diese Schicht kapselt die tatsächliche Interaktion mit dem Manjaro-System.

- **Verwendung von `QProcess`:**
    - **Anwendungsfälle:** Ausführung von Kommandozeilenbefehlen für MCP-Kommandos wie `manage_packages_pamac`, `modify_setting_dconf`, `set_brightness`, `clipboard_copy`, `clipboard_paste`.
    - **Methoden:** `QProcess::start()` wird für asynchrone Ausführung verwendet. Die Signale `finished(int exitCode, QProcess::ExitStatus exitStatus)` und `errorOccurred(QProcess::ProcessError error)` müssen verbunden werden, um das Ergebnis oder Fehler zu behandeln.34 `QProcess::execute()` ist eine statische, blockierende Methode; sie sollte nur mit Vorsicht und idealerweise in einem separaten Worker-Thread verwendet werden, um die Haupt-Event-Loop nicht zu blockieren.34 `QProcess::startDetached()` ist ungeeignet, da keine Rückmeldung über Erfolg/Misserfolg oder Ausgabe benötigt wird.64 Der `QProcess`-Instanz muss eine ausreichende Lebensdauer gegeben werden (z.B. als Member-Variable oder Heap-Allokation mit Parent), da der Prozess sonst terminiert wird, wenn das `QProcess`-Objekt zerstört wird.64
    - **Argumentübergabe:** Kommandozeilenargumente müssen _immer_ als `QStringList` an `start()` übergeben werden.34 Dies verhindert Shell-Injection-Angriffe, da Qt die Argumente korrekt escaped und direkt an den auszuführenden Prozess übergibt, ohne eine Shell dazwischenzuschalten.37 Niemals Befehle durch String-Konkatenation mit Benutzereingaben zusammenbauen.
    - **Ausgabe lesen:** `stdout` und `stderr` werden über die Signale `readyReadStandardOutput()` und `readyReadStandardError()` oder nach Beendigung des Prozesses mit `readAllStandardOutput()` und `readAllStandardError()` gelesen.34 Die Ausgabe muss ggf. geparst werden (z. B. JSON-Ausgabe von Pamac, Textausgabe von `gsettings get`).
    - **Fehlerbehandlung:** Fehler wie "Programm nicht gefunden" (`QProcess::FailedToStart`), Absturz des Prozesses oder ein Exit-Code ungleich Null müssen abgefangen und in entsprechende MCP-Fehlercodes übersetzt werden.34
- **Verwendung von `QDBus`:**
    - **Anwendungsfälle:** Interaktion mit Diensten, die DBus-Schnittstellen anbieten (z. B. Lautstärkeregelung, Benachrichtigungen, Energieverwaltung).
    - **Identifikation:** Dienste, Objektpfade, Interfaces und Methoden/Signale müssen identifiziert werden (z. B. mit `qdbusviewer` oder durch Dokumentation der Desktop-Umgebung).39
    - **Implementierung:** Verwendung von `QDBusInterface` zum Aufrufen von Methoden oder `QDBusConnection::connect()` zum Verbinden mit Signalen.40 Asynchrone Aufrufe (`QDBusPendingCallWatcher`) sind zu bevorzugen. DBus-Fehler (`QDBusError`) müssen behandelt werden.
- **Interaktion mit `gsettings`/`dconf`:**
    - **Bevorzugter Ansatz:** Verwendung des `gsettings`-Kommandozeilenwerkzeugs via `QProcess`, da dies Schema-Validierung durchführt und als stabiler gilt als die direkte Interaktion mit der dconf-API.4
    - **Befehle:** Konstruktion von Befehlen wie `gsettings get <schema> <key>` oder `gsettings set <schema> <key> <value>`. Werte müssen korrekt für die Kommandozeile escaped/quotiert werden. Der Datentyp des Wertes muss dem Schema entsprechen.
    - **Ergebnis:** Bei `get`-Befehlen wird die `stdout`-Ausgabe geparst. Bei `set`-Befehlen wird der Exit-Code überprüft (0 für Erfolg). Fehler werden als `GSETTINGS_ERROR` gemeldet.
- **Allgemeine Fehlerbehandlung:** Jede Interaktionsmethode muss robust Fehler behandeln (Kommando nicht gefunden, Berechtigungsfehler, ungültige Argumente, Zeitüberschreitungen, unerwartete Ausgabeformate) und diese in die definierten MCP-Fehlercodes und aussagekräftige Meldungen übersetzen.

### C. Sicherheitsaspekte bei der Implementierung

Sicherheit muss auf Implementierungsebene berücksichtigt werden:

- **Eingabevalidierung und -bereinigung:** Obwohl das LLM das MCP generiert, muss das Backend _jede_ eingehende MCP-Anfrage und _alle_ Parameter erneut rigoros validieren und bereinigen, bevor sie in Systemaufrufen verwendet werden. Dies gilt insbesondere für Dateipfade, Paketnamen, Shell-Befehle (falls Skripte ausgeführt werden) und Konfigurationswerte.
- **Sichere Befehlskonstruktion:** Wie oben erwähnt, niemals Shell-Befehle durch String-Konkatenation erstellen. Immer `QProcess` mit `QStringList` für Argumente verwenden, um Shell-Interpretation zu umgehen.34
- **Privilegientrennung:** Der Backend-Prozess muss mit den Rechten des angemeldeten Benutzers laufen, nicht mit Root-Rechten. Wenn Aktionen höhere Rechte erfordern (z. B. Paketinstallation), sollte dies über etablierte Mechanismen wie Polkit erfolgen, die eine feingranulare Rechteverwaltung ermöglichen. Die direkte Verwendung von `sudo` im Backend ist zu vermeiden. Die Komplexität und Angriffsfläche erhöhen sich jedoch durch Polkit-Integration.

## VIII. Sicherheitsanalyse und Mitigation

Die Möglichkeit, Systemaktionen über eine KI-Schnittstelle auszulösen, birgt inhärente Sicherheitsrisiken, die sorgfältig analysiert und mitigiert werden müssen.

### A. Bedrohungsmodell

- **Angreifer:**
    - Ein böswilliger Benutzer, der versucht, durch geschickte Eingaben (Prompt Injection) das LLM zur Generierung schädlicher MCP-Befehle zu verleiten.
    - Ein kompromittiertes LLM (insbesondere bei Nutzung externer APIs).
    - Malware, die bereits auf dem System des Benutzers aktiv ist und versucht, die Sidebar oder deren Backend-Prozess auszunutzen.
- **Schützenswerte Güter (Assets):**
    - Benutzerdaten (persönliche Dateien, Konfigurationen, potenziell Zugangsdaten).
    - Systemintegrität (stabile Funktion des Betriebssystems und installierter Software).
    - Benutzerprivilegien und -identität.
    - Systemressourcen (CPU, Speicher, Netzwerkbandbreite).
- **Angriffsvektoren:**
    - **Prompt Injection:** Manipulation der LLM-Eingabe, um unerwünschte MCP-Befehle zu generieren.
    - **Exploitation von Befehlsausführung:** Ausnutzung von Schwachstellen in der Art, wie `QProcess` externe Befehle startet und verarbeitet, oder in den aufgerufenen Tools selbst.
    - **Unsichere DBus-Interaktion:** Ausnutzung von Schwachstellen in DBus-Diensten oder unsichere Kommunikation.
    - **Missbrauch von Dateisystemzugriff:** Generierung von MCP-Befehlen (`list_files` oder potenziell zukünftige Schreibbefehle), die auf sensible Bereiche zugreifen oder diese verändern.
    - **Unsichere Handhabung sensibler Daten:** Falls die Sidebar jemals Passwörter oder API-Schlüssel verarbeiten sollte (was vermieden werden sollte).

### B. Risikoidentifikation

Basierend auf dem Bedrohungsmodell ergeben sich folgende Hauptrisiken:

- **R1: Ausführung beliebigen Codes/Befehle (Arbitrary Code/Command Execution):** Höchstes Risiko. Ein manipuliertes LLM könnte MCP-Befehle generieren, die schädliche Aktionen auslösen (z. B. `open_application` mit Shell-Metazeichen im Namen, `manage_packages_pamac` zur Installation von Malware, `list_files` kombiniert mit Shell-Pipes in unsicherer Ausführung).
- **R2: Privilegieneskalation:** Wenn das Backend mit erhöhten Rechten läuft oder unsicher mit privilegierten Prozessen (z. B. via Polkit oder `sudo`) interagiert, könnte ein Angreifer Root-Zugriff erlangen.
- **R3: Informationspreisgabe:** MCP-Befehle wie `query_system_info` oder `list_files` könnten, wenn sie auf sensible Pfade oder Informationen angewendet werden, Daten an das LLM oder den Angreifer leaken.
- **R4: Denial of Service (DoS):** Gezielte MCP-Befehle könnten Systemressourcen überlasten (z. B. `list_files /`, exzessive `pamac`-Aufrufe) oder das System instabil machen.
- **R5: Datenkorruption/-löschung:** Befehle, die Einstellungen (`modify_setting_dconf`) oder potenziell Dateien ändern, könnten bei unzureichender Parameter-Validierung zu Datenverlust führen.
- **R6: LLM-Schwachstellen:** Eine Kompromittierung des LLM selbst (insbesondere bei Cloud-Diensten) oder erfolgreiche Prompt-Injection-Angriffe könnten zur Generierung schädlicher MCP-Befehle führen.

### C. Mitigationsstrategien

Um die identifizierten Risiken zu minimieren, müssen mehrere Verteidigungslinien implementiert werden:

1. **Strikte MCP-Validierung:** Das Backend _muss_ jede eingehende MCP-Anfrage rigoros gegen die in Abschnitt V definierte Spezifikation validieren. Dies umfasst die Struktur, den Befehlsnamen, die Anwesenheit und Typen aller Parameter sowie gültige Enum-Werte. Jede Abweichung führt zur sofortigen Ablehnung der Anfrage mit einem Fehler. (Adressiert R1, R5, R6)
2. **Parameter-Sanitisierung/-Escaping:** Alle Parameter, die in Systemaufrufen verwendet werden, müssen sorgfältig bereinigt und/oder escaped werden. Für `QProcess` ist die Verwendung von `QStringList` zur Argumentübergabe essenziell, um Shell-Interpretation zu vermeiden.34 Dateipfade und andere Strings müssen auf gefährliche Zeichen oder Sequenzen geprüft werden. (Adressiert R1, R5)
3. **Prinzip der geringsten Rechte (Least Privilege):** Der Backend-Prozess muss mit den Standardrechten des angemeldeten Benutzers laufen. Root-Rechte oder `sudo` sind zu vermeiden. Falls einzelne Aktionen erhöhte Rechte benötigen (z. B. systemweite Paketinstallation), ist eine feingranulare Autorisierung über Polkit zu prüfen, wobei die zusätzliche Komplexität und Angriffsfläche bedacht werden muss. (Adressiert R2)
4. **Command Whitelisting/Allowlisting (Optional):** Wenn möglich, sollte der Satz der erlaubten Aktionen weiter eingeschränkt werden. Beispielsweise könnte `open_application` nur auf Anwendungen aus einem vordefinierten, sicheren Satz beschränkt werden, oder `modify_setting_dconf` nur auf bestimmte, ungefährliche Schemata/Schlüssel. Dies reduziert die Angriffsfläche, kann aber die Flexibilität einschränken. (Adressiert R1, R5)
5. **Sandboxing der `QProcess`-Ausführung:** Dies ist eine kritische Maßnahme zur Eindämmung von R1.
    - _Konzept:_ Externe Prozesse, die über `QProcess` gestartet werden (insbesondere `pamac`, `gsettings`, `wl-clipboard`, `brightnessctl`), sollten in einer isolierten Umgebung (Sandbox) ausgeführt werden, die ihre Zugriffsrechte auf das System stark einschränkt.69
    - _Werkzeuge:_ `firejail` 71 und `bubblewrap` 73 sind geeignete Werkzeuge unter Linux. `firejail` bietet oft vordefinierte Profile, verwendet aber standardmäßig ein SUID-Binary, was eigene Risiken birgt.71 `bubblewrap` ist die Basis für Flatpak-Sandboxing, erfordert oft mehr manuelle Konfiguration, kann aber potenziell ohne SUID (mit User Namespaces) genutzt werden, wenn die Kernel-Unterstützung gegeben ist.73
    - _Implementierung:_ Statt `process->start("pamac", args)` würde man `process->start("firejail", QStringList() << "--profile=custom_pamac_profile" << "pamac" << args)` oder einen äquivalenten `bwrap`-Aufruf verwenden.
    - _Vorteile:_ Begrenzt den Schaden, den ein kompromittierter oder fehlgeleiteter Befehl anrichten kann, erheblich, indem Dateisystemzugriff, Netzwerkzugriff und erlaubte Systemaufrufe (via Seccomp) eingeschränkt werden.71
    - _Herausforderungen:_ Erfordert die Erstellung und Pflege spezifischer Sandbox-Profile für jedes verwendete externe Werkzeug. Kann zu Kompatibilitätsproblemen führen, wenn das Werkzeug legitime Zugriffe benötigt, die vom Profil blockiert werden. Potenzieller Performance-Overhead.
    - _Abwägung:_ Angesichts des Risikos, dass ein LLM unvorhersehbare oder manipulierte Befehle generiert, bietet Sandboxing eine essenzielle zusätzliche Sicherheitsebene. Die Komplexität der Profilerstellung muss gegen den Sicherheitsgewinn abgewogen werden. Es ist eine stark empfohlene Maßnahme. (Adressiert R1, R3, R4, R5)
6. **Rate Limiting:** Implementierung einer Begrenzung der Häufigkeit, mit der MCP-Befehle (insbesondere ressourcenintensive wie `pamac`) ausgeführt werden können, um DoS-Angriffe zu erschweren. (Adressiert R4)
7. **Benutzerbestätigung (Optional):** Für potenziell destruktive oder sicherheitskritische Aktionen (z. B. `pamac remove`, `pamac install`, Ändern wichtiger Systemeinstellungen) könnte eine explizite Bestätigung durch den Benutzer über einen Dialog im Frontend erforderlich sein, selbst wenn der Befehl vom LLM generiert wurde. Dies erhöht die Sicherheit, verringert aber die Automatisierung. (Adressiert R1, R5)
8. **Sichere LLM-Interaktion:** Bei Nutzung einer externen API muss die Kommunikation über HTTPS erfolgen. API-Schlüssel müssen sicher gespeichert und übertragen werden. Es ist zu überlegen, welche Daten (Benutzereingaben) an externe Dienste gesendet werden (Datenschutz). (Adressiert R6)

### D. Sicherheitsfokussiertes Testen

Zusätzlich zu den funktionalen Tests sind spezifische Sicherheitstests erforderlich:

- Penetration Testing: Gezielte Versuche, die Sicherheitsmechanismen zu umgehen.
- Fuzzing: Testen des MCP-Parsers und der System Interaction Layer mit ungültigen oder unerwarteten Eingaben.
- Prompt Injection Testing: Versuche, das LLM durch speziell gestaltete Eingaben zur Generierung unerwünschter MCP-Befehle zu bringen.
- Sandbox-Effektivität: Überprüfung, ob die implementierten Sandboxes (falls verwendet) die erwarteten Einschränkungen durchsetzen.

### Tabelle: Risikobewertung und Mitigation

|   |   |   |   |   |   |
|---|---|---|---|---|---|
|**Risiko ID**|**Beschreibung**|**Wahrscheinlichkeit**|**Auswirkung**|**Mitigationsstrategie(n) (Ref. C.x)**|**Restrisiko**|
|R1|Ausführung beliebigen Codes/Befehle|Hoch (ohne Mitigation)|Kritisch|C.1, C.2, C.4, C.5, C.7|Mittel (mit C.5), Hoch (ohne C.5)|
|R2|Privilegieneskalation|Mittel|Kritisch|C.3|Niedrig|
|R3|Informationspreisgabe|Mittel|Hoch|C.1, C.2, C.5|Niedrig-Mittel|
|R4|Denial of Service (DoS)|Mittel|Mittel|C.5, C.6|Niedrig|
|R5|Datenkorruption/-löschung|Mittel|Hoch|C.1, C.2, C.5, C.7|Niedrig-Mittel|
|R6|LLM-Schwachstellen / Prompt Injection|Hoch (API), Mittel (Lokal)|Hoch|C.1, C.2, C.5, C.7, C.8|Mittel|

_Anmerkung zur Tabelle:_ Die Bewertungen (Wahrscheinlichkeit, Auswirkung, Restrisiko) sind qualitativ und dienen der Priorisierung. Die Effektivität der Mitigationen, insbesondere von C.5 (Sandboxing), beeinflusst das Restrisiko maßgeblich. Diese Tabelle erzwingt eine systematische Betrachtung der Risiken und stellt sicher, dass für jedes identifizierte Risiko eine geplante Gegenmaßnahme existiert.

## IX. Grober Entwicklungs- und Testplan

Dieser Plan skizziert die Hauptphasen der Entwicklung und die dazugehörigen Testaktivitäten.

### A. Entwicklungsphasen

1. **Phase 1: Kern-Backend & Basis-MCP (ca. 4-6 Wochen)**
    - Implementierung der grundlegenden C++ Backend-Struktur (Core Logic, leere Module für LLM, MCP, System Interaction).
    - Implementierung des MCP Interface Handlers für das Parsen und Validieren von JSON-Anfragen und das Generieren von Antworten.
    - Implementierung der System Interaction Layer für eine kleine Teilmenge von MCP-Befehlen (z. B. `query_system_info`, `open_application`) unter Verwendung von `QProcess` und ggf. `QDBus` für einfache Tests.
    - Fokus: Robuste MCP-Verarbeitung und grundlegende Systeminteraktion.
2. **Phase 2: Sidebar UI & Wayland-Integration (ca. 3-4 Wochen)**
    - Entwicklung der initialen QML-Benutzeroberfläche für die Sidebar (Eingabefeld, Ausgabebereich).
    - Integration des QML-Frontends mit dem C++ Backend für einen einfachen Request/Response-Fluss (initial mit fest kodierten oder simulierten MCP-Nachrichten).
    - Implementierung der persistenten Sidebar-Funktionalität unter Wayland mithilfe von `layer-shell-qt`.22 Initialer Fokus auf KDE Plasma.
    - Fokus: Funktionierende UI und korrekte Darstellung/Positionierung unter Wayland (Plasma).
3. **Phase 3: LLM-Integration & MCP-Generierung (ca. 5-7 Wochen)**
    - Auswahl des initialen LLM (API-basiert für schnellere Iteration empfohlen, oder lokal mit Fokus auf Constrained Generation).
    - Implementierung des LLM Integration Module zur Kommunikation mit dem LLM.
    - Entwicklung des Prompt Engineerings bzw. der Function/Tool-Definitionen, um das LLM zur Generierung von MCP-Befehlen basierend auf natürlicher Sprache zu bewegen.
    - **Kritischer Test:** Überprüfung, ob das LLM valide MCP-Befehle _ausschließlich_ basierend auf der Spezifikation aus Abschnitt V generieren kann.45
    - Fokus: Übersetzung von natürlicher Sprache in korrekte MCP-JSON-Anfragen.
4. **Phase 4: Erweiterung des MCP-Befehlssatzes (ca. 6-8 Wochen)**
    - Implementierung der verbleibenden MCP-Befehle aus Abschnitt V.C.
    - Implementierung der entsprechenden Logik in der System Interaction Layer (Interaktion mit `pamac` 9, `gsettings` 4, `brightnessctl` 1, `wl-clipboard` 12 etc.).
    - Umfassende Tests der einzelnen Systeminteraktionen.
    - Fokus: Abdeckung der definierten Systemfunktionalität.
5. **Phase 5: Sicherheits-Hardening & Sandboxing (ca. 4-5 Wochen)**
    - Implementierung der definierten Sicherheitsmitigationen (strikte Validierung, Parameter-Sanitisierung).
    - Falls entschieden: Implementierung des Sandboxings für `QProcess`-Aufrufe mittels `firejail` oder `bubblewrap`, inklusive Erstellung der notwendigen Profile.68
    - Durchführung initialer Sicherheitstests.
    - Fokus: Absicherung der Anwendung gegen die identifizierten Risiken.
6. **Phase 6: Cross-DE Testing & Verfeinerung (ca. 3-4 Wochen)**
    - Testen der Anwendung unter verschiedenen Manjaro Desktop-Umgebungen (insbesondere GNOME und ggf. XFCE/Wayland).
    - Identifikation von Kompatibilitätsproblemen (speziell bei GNOME bzgl. `wlr-layer-shell` 21) und Entwicklung von Anpassungen oder Dokumentation von Einschränkungen.
    - Verfeinerung der UI/UX basierend auf Testergebnissen.
    - Fokus: Sicherstellung der bestmöglichen Funktion und Integration über verschiedene Umgebungen hinweg.
7. **Phase 7: Beta-Testing & Release (kontinuierlich)**
    - Durchführung von Beta-Tests mit einer breiteren Benutzergruppe.
    - Sammeln von Feedback, Behebung von Fehlern.
    - Erstellung von Benutzer- und Entwicklerdokumentation.
    - Vorbereitung des Releases.

### B. Teststrategie

Eine mehrschichtige Teststrategie ist erforderlich:

- **Unit-Tests:** Testen einzelner C++-Klassen und Funktionen im Backend (MCP-Parser, Validierer, einzelne Module der System Interaction Layer) isoliert voneinander unter Verwendung eines Test-Frameworks (z. B. Qt Test).
- **Integrationstests:** Testen des Zusammenspiels der Komponenten: QML-Frontend -> Core Logic -> LLM Module -> MCP Handler -> System Interaction Layer -> System -> Response -> Frontend. Simulation von LLM-Antworten und Systemverhalten.
- **MCP-Konformitätstests:**
    - _LLM-Generierung:_ Systematisches Testen, ob das LLM für eine breite Palette von natürlichsprachlichen Anfragen die korrekten MCP-JSON-Anfragen gemäß Spezifikation V generiert (Genauigkeit, Format, Parameter). Dies muss _ohne_ externes Wissen erfolgen.
    - _Backend-Verarbeitung:_ Testen, ob das Backend alle in V.C definierten Befehle korrekt validiert, verarbeitet und die erwarteten `data`- oder `error`-Strukturen in der MCP-Antwort zurückgibt. Testen aller definierten Fehlerfälle.
- **Systeminteraktionstests:** Verifizierung, dass jede Systemaktion (Pamac, gsettings, Helligkeit, Zwischenablage etc.) auf einem realen Manjaro-System korrekt ausgeführt wird. Testen von Grenzfällen (z. B. Paket nicht gefunden, Berechtigung verweigert, ungültige Eingaben). Tests sollten idealerweise auf den Ziel-Desktop-Umgebungen (Plasma, GNOME) durchgeführt werden.
- **Sicherheitstests:** Gezielte Tests zur Überprüfung der Sicherheitsmitigationen: Penetration Testing, Versuche von Prompt Injection, Überprüfung der Effektivität der Sandboxing-Maßnahmen (falls implementiert).
- **UI/UX-Tests:** Überprüfung der Benutzerfreundlichkeit, Responsivität und visuellen Integration der Sidebar auf den Ziel-Desktop-Umgebungen (Plasma, GNOME, XFCE).
- **Performancetests:** Messung der Ende-zu-Ende-Latenz von Benutzeranfrage bis zur Antwort, insbesondere der Latenz des LLM und der Systembefehlsausführung. Identifikation von Flaschenhälsen.

## X. Schlussfolgerung

### Zusammenfassung

Dieser Bericht hat einen detaillierten Plan und eine technische Spezifikation für die Entwicklung einer KI-gestützten Desktop-Sidebar für Manjaro Linux unter Verwendung von C++, Qt, QML und Qt-Wayland vorgestellt. Die vorgeschlagene Architektur trennt klar zwischen Frontend, Backend-Logik, LLM-Interaktion und Systemzugriff. Das Kernstück bildet das Manjaro Control Protocol (MCP), eine JSON-basierte Schnittstelle, die speziell darauf ausgelegt ist, von einem LLM allein anhand dieser Spezifikation verstanden und genutzt zu werden. Die Integration in Wayland-Umgebungen, insbesondere die Nutzung des `wlr-layer-shell`-Protokolls mittels `layer-shell-qt`, wurde ebenso detailliert wie die notwendigen Mechanismen zur Systeminteraktion (`QProcess`, `QDBus`, `gsettings`) und die Strategien zur LLM-Integration (lokal vs. API, strukturierte Ausgabe). Ein besonderer Fokus lag auf der Analyse von Sicherheitsrisiken und der Definition von Mitigationsstrategien, einschließlich der Möglichkeit des Sandboxing für externe Prozessaufrufe.

### Potenzial

Die Realisierung dieses Projekts bietet erhebliches Potenzial. Eine nahtlos integrierte, sprachgesteuerte KI-Assistenz kann die Interaktion mit dem Manjaro-System erheblich vereinfachen und beschleunigen. Aufgaben wie das Starten von Anwendungen, das Verwalten von Paketen oder das Anpassen von Einstellungen werden intuitiver. Dies stellt eine moderne und leistungsfähige Erweiterung der Desktop-Erfahrung dar und positioniert Manjaro als innovative Plattform.

### Herausforderungen

Die Umsetzung birgt auch Herausforderungen. Die Gewährleistung einer konsistenten Funktionalität und visuellen Integration über verschiedene Wayland-basierte Desktop-Umgebungen hinweg, insbesondere die Kompatibilität mit GNOME/Mutter aufgrund der fehlenden `wlr-layer-shell`-Unterstützung 21, erfordert sorgfältige Planung und möglicherweise umgebungsspezifische Anpassungen. Die Absicherung des Systems gegen Missbrauch durch die KI-Schnittstelle, insbesondere die Risiken der Befehlsausführung (R1) und der LLM-Manipulation (R6), bedarf rigoroser Implementierung der Sicherheitsmaßnahmen, wobei Sandboxing 70 eine wichtige, aber komplexe Komponente darstellt. Die Sicherstellung, dass das LLM das MCP korrekt und zuverlässig _allein_ aus der Spezifikation anwendet, ist eine zentrale Anforderung, die sorgfältiges Prompt Engineering und möglicherweise den Einsatz von Constrained Generation Techniken erfordert.

### Nächste Schritte

Basierend auf dieser detaillierten Speifikation wird empfohlen, mit der Entwicklung gemäß Phase 1 des vorgeschlagenen Plans zu beginnen. Dies umfasst die Implementierung des Kern-Backends und der Basis-MCP-Verarbeitung, um eine solide Grundlage für die weiteren Schritte zu schaffen. Parallel dazu sollte die Auswahl des LLM und die Verfeinerung der Integrationsstrategie unter Berücksichtigung der strukturierten Ausgabeanforderungen erfolgen.