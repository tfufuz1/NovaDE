# Detaillierter Bauplan für die VivoX-Aktionsbibliothek

## 1. Einleitung

### Zweck

Eine zentrale Aktionsbibliothek (Action Library) ist eine kritische Komponente für die VivoX-Desktopanwendung. Sie dient der Verwaltung aller benutzerinitiierter Befehle und Aktionen innerhalb der Applikation. Ihre Hauptaufgaben sind die Gewährleistung von Konsistenz über verschiedene UI-Elemente hinweg, die Verbesserung der Wartbarkeit durch Zentralisierung der Aktionslogik und die Schaffung einer Grundlage für zukünftige Erweiterbarkeit. Durch die Abstraktion von Aktionen können diese einheitlich über Menüs, Werkzeugleisten, Tastaturkürzel oder Befehlspaletten ausgelöst werden, während die zugrundeliegende Logik an einer Stelle definiert und verwaltet wird.

### Blueprint-Ziele

Dieses Dokument stellt einen umfassenden technischen Bauplan für die Konzeption und Implementierung der VivoX-Aktionsbibliothek dar. Es legt die architektonischen Grundlagen, definiert die Verantwortlichkeiten der Kernkomponenten, spezifiziert die notwendigen APIs und beschreibt Integrationsstrategien mit anderen Systemteilen. Das Ziel ist es, eine detaillierte technische Spezifikation bereitzustellen, die dem Entwicklungsteam als Leitfaden für die Implementierung dient.

### Zielgruppe und Umfang

Dieser Bauplan richtet sich primär an Softwarearchitekten und erfahrene Entwickler des VivoX-Projekts, die für die Implementierung dieser Kernkomponente verantwortlich sind. Der Umfang umfasst die Analyse relevanter Designmuster, die Integration mit Qt 6 (C++ und QML), den Entwurf der zentralen C++-Verwaltungskomponente (`ActionManager`) und der QML-Integrationsschicht (`ActionRegistry`), die Definition der Aktionsentität selbst (`VivoXAction`), die Mechanismen zur Handhabung von Eingaben (Tastaturkürzel, Gesten) sowie die Integration in ein Undo/Redo-Framework.

### Kernprinzipien

Die Entwicklung der VivoX-Aktionsbibliothek wird von folgenden Kernprinzipien geleitet:

- **Entkopplung:** Strikte Trennung zwischen der Aktionsdefinition (Was soll passieren?), der Aktionsauslösung (Wie wird es initiiert?) und der Aktionsausführung (Wer führt es durch?). UI-Elemente sollen Aktionen nur über eine abstrakte Schnittstelle (z.B. eine eindeutige ID) ansprechen.
- **Wiederverwendbarkeit:** Aktionen werden einmal definiert und können von verschiedenen UI-Elementen (Menüeinträge, Toolbar-Buttons, Befehlspalette) und durch verschiedene Eingabemethoden (Mausklick, Shortcut, Geste) wiederverwendet werden.
- **Kontextsensitivität:** Der Zustand einer Aktion (z.B. aktiviert/deaktiviert, sichtbar/unsichtbar) muss dynamisch an den aktuellen Anwendungszustand (Kontext) angepasst werden können.
- **Testbarkeit:** Durch die Entkopplung sollen sowohl die Aktionslogik als auch die Verwaltungsmechanismen isoliert testbar sein.
- **Benutzererfahrung (User Experience):** Die Bibliothek muss moderne Interaktionsmuster wie Tastaturkürzel, eine Befehlspalette für schnellen Zugriff und eine nahtlose Integration mit der Undo/Redo-Funktionalität unterstützen, um eine effiziente und intuitive Bedienung zu ermöglichen.

## 2. Grundlegende Designmuster für Aktionsverwaltung

Die Architektur der VivoX-Aktionsbibliothek basiert auf etablierten Software-Designmustern, die sich für die Verwaltung von Befehlen und Aktionen bewährt haben. Insbesondere das Command Pattern und die Prinzipien der Aktionsabstraktion bilden das Fundament.

### Das Command Pattern 1

Das Command Pattern ist ein Verhaltensmuster, dessen Kernidee darin besteht, eine Anfrage oder Operation als eigenständiges Objekt zu kapseln.1 Dieses "Command"-Objekt enthält alle notwendigen Informationen, um die Aktion zu einem späteren Zeitpunkt auszuführen. Dazu gehören typischerweise die auszuführende Methode, das Objekt (der "Receiver"), auf dem die Methode aufgerufen wird, und die Werte für die Methodenparameter.3

Die Schlüsselkomponenten dieses Musters sind 1:

1. **Command Interface/Abstract Class:** Definiert eine einheitliche Schnittstelle für alle Befehle, typischerweise eine `execute()`-Methode.
2. **Concrete Command:** Implementiert die Command-Schnittstelle und kapselt eine spezifische Aktion sowie die Verbindung zum Receiver.
3. **Receiver:** Das Objekt, das die eigentliche Arbeit ausführt, wenn das Kommando ausgeführt wird. Es kennt die Details der Operation (z.B. ein `Document`-Objekt mit einer `save()`-Methode).
4. **Invoker:** Löst die Ausführung eines Kommandos aus. Es hält eine Referenz auf ein Command-Objekt, muss aber weder das Kommando noch den Receiver im Detail kennen. Beispiele für Invoker sind Menüeinträge, Buttons oder Shortcut-Handler.
5. **Client:** Erzeugt die Concrete Command-Objekte und konfiguriert sie mit dem passenden Receiver.

Für das VivoX-Projekt bietet das Command Pattern entscheidende Vorteile:

- **Entkopplung:** Es trennt den Sender einer Anfrage (z.B. ein Button-Klick) vom Empfänger (z.B. die Backend-Logik, die eine Datei speichert).1 Der Button muss nur das Command-Objekt kennen und dessen `execute()`-Methode aufrufen.
- **Erweiterbarkeit:** Neue Aktionen können einfach durch das Hinzufügen neuer Concrete Command-Klassen implementiert werden, ohne bestehenden Code ändern zu müssen.1
- **Unterstützung für Undo/Redo:** Da jede Aktion als Objekt gekapselt ist, können diese Objekte in einer Historie (z.B. einem Stack) gespeichert werden. Wenn jedes Command-Objekt zusätzlich eine `undo()`-Methode implementiert, die seine Aktion rückgängig macht, lässt sich eine Undo/Redo-Funktionalität realisieren, indem die `undo()`- bzw. `redo()`-Methoden der gespeicherten Kommandos aufgerufen werden.1 Dies ist eine direkte Antwort auf die Anforderung (8) der Benutzeranfrage.
- **Weitere Möglichkeiten:** Das Muster erleichtert auch das Queuing von Befehlen, das Logging von Aktionen oder die Implementierung von Makro-Befehlen (Composite Commands).1

Bei der Implementierung ist zu beachten, dass eine große Anzahl von Aktionen zu einer entsprechenden Anzahl von Command-Klassen führen kann, was die Komplexität erhöhen kann.1 Eine klare Schnittstellendefinition ist daher wichtig.2 Moderne C++-Ansätze mit `std::variant` und `std::visit` könnten eine Alternative zur klassischen Vererbungshierarchie darstellen, um verschiedene Befehlstypen zu verwalten.7 Allerdings harmoniert das klassische Modell gut mit den bestehenden Qt-Klassen wie `QAction` und insbesondere `QUndoCommand`.

### Aktionsabstraktion-Prinzipien 8

Während das Command Pattern den _Mechanismus_ der Aktionsausführung beschreibt, fokussiert die Aktionsabstraktion auf die Trennung zwischen der _Benutzerabsicht_ (dem "Was", z.B. "Speichern") und der konkreten _Implementierung_ (dem "Wie", z.B. Speichern einer neuen Datei vs. Überschreiben einer bestehenden Datei). Dieses Prinzip ist eng mit UI-Designmustern verbunden, die darauf abzielen, dem Benutzer klare und kontextabhängige Handlungsoptionen anzubieten.8

Für VivoX ist dieses Prinzip zentral, um die Kontextabhängigkeit von Aktionen zu managen. Eine abstrakte Aktion wie "Kopieren" ist konzeptionell immer dieselbe, aber ihre Verfügbarkeit (`enabled`-Status) und möglicherweise ihre spezifische Ausführung hängen davon ab, was gerade im Anwendungsfenster ausgewählt ist oder welcher Zustand aktiv ist. Die Aktionsabstraktion ermöglicht es, diese kontextuellen Regeln zentral zu verwalten und die Benutzeroberfläche entsprechend anzupassen.13 Dies adressiert die Anforderung (4) der Benutzeranfrage bezüglich Kontextabhängigkeiten.

Die Aktionsabstraktion definiert das benutzerseitige Konzept einer Aktion, während das Command Pattern den Mechanismus zur Implementierung dieser Abstraktion bereitstellt. Die später definierte `VivoXAction` wird die Abstraktion repräsentieren, und konkrete Command-Klassen (oftmals integriert mit `QUndoCommand`) werden deren Ausführung implementieren. Dies steht im Einklang mit UI-Mustern wie "Progressive Disclosure" (nur relevante Optionen anzeigen) 8 oder dem Bereitstellen klarer primärer Aktionen.8

### Begründung der Musterauswahl

Die kombinierte Anwendung des Command Patterns und der Aktionsabstraktion wird als optimale Grundlage für die VivoX-Aktionsbibliothek erachtet. Das Command Pattern liefert die technische Basis für Entkopplung, Erweiterbarkeit und insbesondere die Undo/Redo-Funktionalität. Die Aktionsabstraktion stellt sicher, dass Aktionen aus Benutzersicht konsistent und kontextabhängig verwaltet werden können.

Die Beobachtung, dass das Command Pattern explizit mit Undo/Redo-Fähigkeiten in Verbindung gebracht wird 1, unterstreicht dessen Eignung. Indem jede Operation als Objekt gekapselt wird, das potenziell auch seine Umkehrung kennt (`undo()`), entsteht eine natürliche Struktur für eine Befehlshistorie. Die Implementierung von Undo/Redo wird dadurch erheblich vereinfacht, da sie auf diesem Muster aufbauen kann.

Gleichzeitig erfordert die Verwaltung kontextsensitiver Aktionen [Anforderung (4)] eine Trennung von Absicht und Implementierung, wie sie die Aktionsabstraktion vorsieht.9 Eine abstrakte Aktion wie "Einfügen" muss ihren Zustand (aktiviert/deaktiviert) basierend auf dem Kontext (z.B. Inhalt der Zwischenablage, fokussiertes Element) ändern können. Die Aktionsbibliothek muss daher Mechanismen zur Kontextbewertung und Zustandsaktualisierung beinhalten, was durch die Abstraktion erleichtert wird.

## 3. Nutzung der Qt 6 Aktionsinfrastruktur

Qt bietet eine etablierte Infrastruktur für die Verwaltung von Aktionen und Shortcuts, die als Basis für die VivoX-Aktionsbibliothek dienen sollte. Die Nutzung dieser vorhandenen Klassen beschleunigt die Entwicklung und gewährleistet eine gute Integration in das Qt-Ökosystem.

### Leveraging `QAction` 15

`QAction` ist die zentrale Qt-Klasse zur Repräsentation abstrakter Benutzeraktionen.15 Sie kapselt wichtige Eigenschaften wie Text (`text`), Icon (`icon`), Tooltip (`toolTip`), Status-Text (`statusTip`), Tastaturkürzel (`shortcut`), sowie Zustandsinformationen wie Aktivierungsstatus (`enabled`), Prüfstatus (`checkable`, `checked`).15 Entscheidend sind auch die Signale `triggered()` (wird ausgelöst, wenn die Aktion aktiviert wird) und `toggled()` (bei Zustandsänderung einer prüfbaren Aktion).15

`QAction` selbst verkörpert bereits viele Aspekte des "Command"-Objekts aus dem Command Pattern.18 Es hält Metadaten zur Aktion und signalisiert deren Aktivierung. Für VivoX wird vorgeschlagen, `QAction` oder eine davon abgeleitete bzw. sie kapselnde Klasse als konkrete Implementierung einer `VivoXAction` im C++ Backend zu verwenden. Dies ermöglicht eine nahtlose Integration mit Qt Widgets wie Menüs (`QMenu`) und Werkzeugleisten (`QToolBar`), die direkt mit `QAction`-Objekten arbeiten. Eigenschaften wie `enabled` und `checked` können dann dynamisch durch den `ActionManager` basierend auf dem Anwendungskontext verwaltet und aktualisiert werden.

### Utilizing `QShortcut` 19

`QShortcut` dient der Definition von Tastaturkürzeln und deren direkter Verbindung mit Signalen und Slots, insbesondere mit dem `triggered()`-Signal einer `QAction`.20

Ein wesentliches Merkmal von `QShortcut` ist der Aktivierungskontext, definiert durch die Enumeration `Qt::ShortcutContext`.20 Die drei Hauptkontexte sind:

- `Qt::WidgetShortcut`: Aktiv nur, wenn das Eltern-Widget des Shortcuts den Fokus hat.
- `Qt::WindowShortcut`: Aktiv, wenn das Top-Level-Fenster, das das Eltern-Widget enthält, aktiv ist (Standard).
- `Qt::ApplicationShortcut`: Global in der gesamten Anwendung aktiv.

Die Wahl des korrekten Kontexts ist entscheidend für das erwartete Verhalten.20 Für die meisten Aktionen in VivoX, die sich auf ein bestimmtes Dokumentenfenster beziehen, wird `Qt::WindowShortcut` empfohlen. `WidgetShortcut` ist für sehr spezifische, widget-gebundene Aktionen sinnvoll, während `ApplicationShortcut` für globale Befehle (z.B. Einstellungen öffnen) mit Vorsicht verwendet werden sollte, um Konflikte zu vermeiden.20

Der `ActionManager` sollte für die Erstellung und Verwaltung der `QShortcut`-Objekte verantwortlich sein. Bei der Registrierung einer Aktion mit einem definierten Shortcut sollte der Manager ein entsprechendes `QShortcut`-Objekt mit dem korrekten Kontext erstellen und dessen `activated()`-Signal mit der Auslösungslogik der Aktion verbinden (z.B. via `ActionManager::triggerAction`).

### Using `QActionGroup` 19

`QActionGroup` dient der Verwaltung von Aktionsgruppen, insbesondere zur Sicherstellung, dass innerhalb einer Gruppe von prüfbaren Aktionen (`checkable`) immer nur eine Aktion aktiv (checked) sein kann (Exklusivität).19 Dies ist nützlich für Szenarien wie die Auswahl eines Werkzeugs aus einer Werkzeugleiste oder die Wahl eines Ansichtsmodus. Der `ActionManager` könnte optional die Verwaltung solcher Gruppen unterstützen, falls dies für VivoX relevant ist.

### Bridging C++ `QAction` and QML `Action` 15

Qt Quick Controls bietet einen eigenen `Action`-Typ (`QtQuick.Controls.Action`), der viele Eigenschaften und Signale des C++ `QAction` widerspiegelt.15 Wenn die VivoX-UI (teilweise) in QML implementiert wird (z.B. für Menüs, Toolbars oder die Befehlspalette), ist eine Synchronisation zwischen dem Backend C++ `QAction` (verwaltet durch `ActionManager`) und den Frontend QML `Action`-Instanzen notwendig. Eigenschaften wie `enabled`, `checked`, `text` und `icon` müssen konsistent gehalten werden. Die später beschriebene `ActionRegistry` wird diese Brücke schlagen.

### Best Practices

Es wird dringend empfohlen, die vorhandene Qt-Infrastruktur (`QAction`, `QShortcut`, `QActionGroup`) intensiv zu nutzen, um die Neuerfindung von Standardfunktionalität zu vermeiden. Auf korrekte Eltern-Kind-Beziehungen zur Speicherverwaltung von `QAction`- und `QShortcut`-Objekten ist zu achten.

Die Analyse der Qt-Klassen zeigt eine starke Übereinstimmung mit dem Command Pattern. `QAction` kapselt die Aktion und deren Auslösung.15 `QShortcut` und UI-Elemente wie `QToolButton` oder `QMenu` fungieren als Invoker, die `QAction` auslösen.20 `QUndoStack` integriert sich nahtlos mit `QAction` für die Undo/Redo-UI.25 Diese integrierte Implementierung des Command Patterns legt nahe, dass der VivoX `ActionManager` primär diese Qt-Komponenten orchestrieren sollte, anstatt das Muster von Grund auf neu zu implementieren. Die Kernaufgabe verschiebt sich somit auf die Definition des _Inhalts_ der Aktionen und der _Kontextregeln_.

Die verschiedenen Aktivierungskontexte von `QShortcut` (`WidgetShortcut`, `WindowShortcut`, `ApplicationShortcut`) 20 erfordern eine sorgfältige Designentscheidung für jede Aktion. Eine falsche Wahl kann dazu führen, dass Shortcuts nicht wie erwartet funktionieren oder unerwünschte Konflikte verursachen. `WindowShortcut` stellt für die meisten fensterbezogenen Aktionen einen sinnvollen Standard dar.20

## 4. Integration moderner Interaktion: Die Befehlspalette

Moderne Desktop-Anwendungen, insbesondere solche für professionelle Anwender, setzen zunehmend auf Befehlspaletten (Command Palettes) als effizientes Interaktionsmittel.

### Konzept und Zweck 28

Eine Befehlspalette ist ein UI-Pattern, das typischerweise über ein Tastaturkürzel (oft `Ctrl+Shift+P` oder `Cmd+K`) aufgerufen wird und eine durchsuchbare, vereinheitlichte Schnittstelle zum Entdecken und Ausführen von Anwendungsbefehlen bietet.28 Ihr Hauptvorteil liegt darin, dass Benutzer, insbesondere Power-User, schnell auf eine große Anzahl von Aktionen zugreifen können, ohne durch komplexe Menüstrukturen oder überladene Werkzeugleisten navigieren zu müssen.28 Sie dient nicht nur dem Finden, sondern primär dem _Ausführen_ von Aktionen.28

### Analyse existierender Implementierungen (z.B. VS Code) 33

Anwendungen wie Visual Studio Code (VS Code) demonstrieren die Leistungsfähigkeit dieses Musters. Zu den Kernfunktionen gehören 31:

- **Fuzzy-Suche/Filterung:** Ermöglicht das schnelle Finden von Befehlen auch bei ungenauer Eingabe oder Tippfehlern.
- **Anzeige von Metadaten:** Zeigt neben dem Befehlsnamen oft auch das zugehörige Tastaturkürzel an, was das Lernen fördert.
- **Kategorisierung/Gruppierung:** Befehle können zur besseren Übersichtlichkeit gruppiert werden (z.B. nach "Datei", "Bearbeiten", "Git").
- **Direkte Ausführung:** Befehle werden direkt aus der Palette heraus ausgeführt. VS Code integriert seine Befehlspalette (aufrufbar über `Ctrl+Shift+P`) tief in die Anwendung 33 und listet alle verfügbaren Befehle auf. Sie interagiert auch mit der Kommandozeilenschnittstelle 34 und ist Teil der Gesamt-UI-Struktur.35

### UI/UX Designprinzipien 28

Bei der Gestaltung einer Befehlspalette für VivoX sollten folgende Prinzipien beachtet werden:

- **Tastaturfokus:** Unmittelbarer Fokus auf das Eingabefeld beim Öffnen der Palette ist essentiell für einen schnellen Workflow.28
- **Suche vs. Befehl:** Es muss klar unterschieden werden, ob die Palette nur Befehle (Aktionen) oder auch Inhalte (Dateien, Objekte) durchsuchen soll. Eine Vermischung kann zu Unklarheit führen.28 Es ist zu entscheiden, ob eine separate Suchfunktion benötigt wird oder ob die Suche als Aktion in die Palette integriert werden kann.
- **Kontextsensitivität:** Die Palette sollte idealerweise nur Aktionen anzeigen, die im aktuellen Kontext relevant und aktiviert sind.28 Das Anzeigen aller Aktionen, auch deaktivierter, reduziert die Effizienz.
- **Auffindbarkeit:** Das Anzeigen von Tastaturkürzeln neben den Befehlen hilft Benutzern, diese zu lernen und die Anwendung effizienter zu nutzen.31 Klare und konsistente Benennung der Befehle ist ebenfalls wichtig.33

### Architektonische Integration mit VivoX Action Library

Die Befehlspalette integriert sich nahtlos in die vorgeschlagene Aktionsbibliothek:

- **Datenquelle:** Die UI der Befehlspalette fragt den `ActionManager` (oder die `ActionRegistry`) nach der Liste der aktuell verfügbaren (aktivierten und kontextuell relevanten) Aktionen ab.
- **Auslösung:** Wählt der Benutzer eine Aktion in der Palette aus, löst die Paletten-UI diese Aktion über den `ActionManager` aus (z.B. mittels `triggerAction(actionId)`), analog zu einem Menüeintrag oder Shortcut.
- **Implementierung:** Die Befehlspalette kann als QML-Komponente implementiert werden. Ein `ListView` oder `Repeater`, gebunden an ein von der `ActionRegistry` bereitgestelltes Modell, das die verfügbaren Aktionen enthält, ist ein geeigneter Ansatz. Filterfunktionen im Modell oder in der View ermöglichen die Suche.

Die Befehlspalette fungiert als universeller "Invoker" im Sinne des Command Patterns. Sie bietet einen zentralen Zugangspunkt zu potenziell allen Aktionen der Anwendung.28 Für eine wachsende Anwendung wie VivoX ist dies ein skalierbarer Ansatz, um Funktionalität zugänglich zu halten, ohne die traditionellen UI-Elemente zu überladen. Die zentrale Verwaltung aller Aktionen in der Aktionsbibliothek liefert die notwendige Datenbasis für die Palette.

Die Effektivität der Befehlspalette hängt maßgeblich von ihrer Kontextsensitivität ab.28 Eine Palette, die nur relevante und aktivierte Aktionen anzeigt, ist deutlich benutzerfreundlicher als eine, die alle jemals registrierten Aktionen auflistet. Daher ist die Logik zur Kontextbewertung im `ActionManager` nicht nur für Menüs und Toolbars, sondern auch für die dynamische Filterung der in der Befehlspalette angezeigten Aktionen von entscheidender Bedeutung. Die Paletten-UI muss den `ActionManager` dynamisch nach den _aktuell verfügbaren_ Aktionen abfragen.

## 5. VivoX-Aktion: Struktur und Semantik

Das Herzstück der Aktionsbibliothek ist die Definition einer `VivoXAction`. Diese stellt die zentrale Abstraktion für jeden benutzerinvokierbaren Befehl dar und kapselt alle notwendigen Metadaten und Zustandsinformationen.

### Definition des `VivoXAction`-Konzepts

`VivoXAction` repräsentiert eine logische Aktion, unabhängig davon, wie sie ausgelöst wird. Sie dient als Schnittstelle zwischen der UI und der Backend-Logik. Implementierungsseitig könnte dies durch ein C++ Interface `IVivoXAction` geschehen, das von konkreten Befehlsklassen implementiert wird. Alternativ, und wahrscheinlich pragmatischer im Qt-Kontext, könnte eine zentrale C++ Klasse `VivoXActionBase` definiert werden, die entweder von `QAction` erbt oder eine `QAction` Instanz kapselt. Konkrete Aktionen würden dann diese Basisklasse spezialisieren oder konfigurieren.

### Kerneigenschaften

Jede `VivoXAction` sollte mindestens folgende Eigenschaften besitzen:

- **`id` (QString/Enum):** Ein eindeutiger Bezeichner (z.B. `"File.Save"`, `ActionID::FileSave`), der für die Registrierung, das Nachschlagen im `ActionManager` und die Referenzierung aus der UI (z.B. QML) verwendet wird. Dies ist der Schlüssel zur Entkopplung.
- **`text` (QString):** Der für den Benutzer sichtbare Text, der in Menüs, Buttons oder der Befehlspalette angezeigt wird. Muss Internationalisierung unterstützen (z.B. mittels `qsTr()`).15
- **`toolTip` (QString):** Eine erweiterte Beschreibung, die z.B. beim Überfahren mit der Maus angezeigt wird.15
- **`icon` (QIcon/QString):** Ein Icon zur visuellen Repräsentation der Aktion. Kann als Pfad zu einer Bilddatei oder als Name eines Theme-Icons angegeben werden.15
- **`shortcut` (QKeySequence):** Das Standard-Tastaturkürzel für diese Aktion.15

### Zustandsverwaltung

Der Zustand einer Aktion ist dynamisch und kontextabhängig:

- **`enabled` (bool):** Gibt an, ob die Aktion aktuell ausgeführt werden kann. Wird vom `ActionManager` basierend auf dem Kontext verwaltet.15
- **`checkable` (bool):** Gibt an, ob die Aktion einen An/Aus-Zustand repräsentiert (z.B. Fettdruck ein/aus).15
- **`checked` (bool):** Der aktuelle Zustand (an/aus), falls `checkable` true ist. Muss mit dem Backend-Zustand synchronisiert werden.15

### Kontextdefinition und -verwaltung

Der "Kontext" umfasst alle relevanten Zustandsinformationen der Anwendung (z.B. aktives Fenster, Selektionsstatus, Dokumentzustand), die bestimmen, ob eine Aktion verfügbar ist und wie sie sich verhält. Jede `VivoXAction` benötigt daher assoziierte Regeln oder eine Logik (`contextEvaluator`), anhand derer der `ActionManager` entscheiden kann, ob die Aktion im aktuellen Kontext aktiviert (`enabled`) sein soll. Diese Zustände müssen dynamisch aktualisiert werden, wenn sich der Anwendungskontext ändert.

### Tabelle: `VivoXAction` Eigenschaftsspezifikation

Die folgende Tabelle fasst die wesentlichen Eigenschaften einer `VivoXAction` zusammen und dient als Referenz für die Implementierung:

|   |   |   |   |   |   |
|---|---|---|---|---|---|
|**Eigenschaft**|**C++ Typ**|**QML Äquivalent (ca.)**|**Beschreibung**|**Management**|**Anmerkungen**|
|`id`|`QString` oder Enum|`string`|Eindeutiger Bezeichner der Aktion.|Read-only (nach Reg.)|Schlüssel für Registrierung, Lookup, Triggering.|
|`text`|`QString`|`string`|Benutzerlesbarer Text für UI-Elemente.|Read-only (nach Reg.)|Sollte `qsTr()` für I18N verwenden.|
|`toolTip`|`QString`|`string`|Erweiterte Beschreibung (optional).|Read-only (nach Reg.)|Für Hover-Infos.|
|`icon`|`QIcon` / `QString`|`url` / `string`|Visuelles Icon (Pfad oder Theme-Name).|Read-only (nach Reg.)|Siehe `QAction::icon`.|
|`shortcut`|`QKeySequence`|`keysequence`|Standard-Tastaturkürzel.|Read-only (nach Reg.)|Kann durch Benutzereinstellungen überschrieben werden.|
|`enabled`|`bool`|`bool`|Gibt an, ob die Aktion aktuell ausführbar ist.|Read-only (UI)|Wird vom `ActionManager` basierend auf Kontext gesetzt.|
|`checkable`|`bool`|`bool`|Gibt an, ob die Aktion einen An/Aus-Zustand hat.|Read-only (nach Reg.)|Siehe `QAction::checkable`.|
|`checked`|`bool`|`bool`|Aktueller An/Aus-Zustand (wenn `checkable`).|Read-only (UI)|Synchronisiert mit Backend-Zustand.|
|`contextEvaluator`|Funktion/Objekt|-|Logik zur Bestimmung des `enabled`-Status basierend auf dem Kontext.|Read-only (nach Reg.)|Wird vom `ActionManager` aufgerufen.|

Die eindeutige `id` ist fundamental für die Entkopplung im System.1 Sie erlaubt es verschiedenen Teilen der Anwendung (UI, Logik), sich auf dieselbe abstrakte Aktion zu beziehen, ohne direkte Abhängigkeiten von konkreten Objektinstanzen zu haben. Der `ActionManager` nutzt die ID zur Verwaltung. Eine klare Konvention für die Benennung von IDs (z.B. hierarchisch wie `"Edit.Cut"`) ist empfehlenswert.

Die Logik zur Kontextbewertung (`contextEvaluator`) ist entscheidend für die dynamische Aktivierung von Aktionen [Anforderung (4)]. Diese Logik muss mit der Aktionsdefinition verbunden sein, aber zentral vom `ActionManager` ausgeführt werden. Mögliche Implementierungen sind Lambda-Funktionen, Funktionszeiger oder dedizierte Evaluator-Objekte, die bei der Aktionsregistrierung mitgegeben oder über die ID verknüpft werden.

## 6. C++ Kern: `ActionManager`-Architektur

Der `ActionManager` ist die zentrale C++ Komponente für die Verwaltung des Lebenszyklus und Zustands aller Aktionen in VivoX.

### Kernverantwortlichkeiten

- **Registrierung:** Bietet eine Schnittstelle (z.B. `registerAction(VivoXAction*)` oder `registerAction(ActionDefinition)`) an, um Aktionen anhand ihrer eindeutigen `id` zu registrieren. Die registrierten Aktionen werden intern gespeichert, typischerweise in einer `QHash<ActionID, VivoXAction*>` oder ähnlichen Struktur.
- **Lookup:** Ermöglicht das Abrufen einer registrierten Aktion oder ihres Zustands anhand der `id` (z.B. `getAction(actionId)`, `isActionEnabled(actionId)`).
- **Kontextbewertung:** Überwacht relevante Änderungen im Anwendungszustand (Kontext). Bei einer Kontextänderung wertet der `ActionManager` die `contextEvaluator`-Logik der betroffenen Aktionen neu aus, aktualisiert deren `enabled`- (und ggf. `checked`-) Zustand und sendet Signale (z.B. `actionStateChanged(actionId)`), um Listener über die Änderungen zu informieren.
- **Auslösung:** Stellt eine zentrale Methode zum Auslösen von Aktionen bereit (z.B. `triggerAction(actionId)`). Diese Methode schlägt die Aktion nach ID nach, prüft, ob sie `enabled` ist, führt die zugehörige Logik aus (dies beinhaltet oft das Erstellen und Ausführen eines `QUndoCommand`) und kümmert sich um eventuelles Logging oder andere übergreifende Aspekte.

### Handhabung von Kontextänderungen

Der `ActionManager` muss über Änderungen im Anwendungskontext informiert werden. Dies kann durch explizite Aufrufe von anderen Systemteilen geschehen (z.B. `actionManager->updateContext(newContext)`) oder, bevorzugt, durch Verbinden von Signalen relevanter Zustandsmanager (z.B. `QItemSelectionModel::selectionChanged`, `Document::modifiedChanged`) mit Slots im `ActionManager`. Bei einer Änderung muss der Manager effizient ermitteln, welche Aktionen potenziell betroffen sind, deren Zustand neu bewerten und über Signale wie `actionStateChanged(actionId, propertyName)` gezielt über Änderungen informieren. Dies stellt sicher, dass die UI (über die `ActionRegistry`) immer den korrekten Aktionszustand widerspiegelt.

### Implementierung von Aktionsketten/Sequenzen 26

Falls VivoX Aktionen benötigt, die eine Sequenz anderer Aktionen auslösen, kann dies unterstützt werden. Eine Möglichkeit ist, dass die `triggerAction`-Methode eine vordefinierte Kette von Action-IDs abarbeitet. Eine andere, oft sauberere Methode ist die Nutzung von `QUndoCommand`-Makros. Entweder durch explizite Eltern-Kind-Beziehungen zwischen `QUndoCommand`-Objekten oder durch Nutzung von `QUndoStack::beginMacro()` und `endMacro()`.26 Der `ActionManager` könnte das Starten und Beenden von Makros für komplexe, zusammengesetzte Aktionen initiieren.

### Vorgeschlagenes C++ `ActionManager` Klassendesign

- **Implementierungsmuster:** Aufgrund seiner zentralen Rolle bietet sich die Implementierung als C++ Singleton an, um globalen Zugriff aus dem C++ Backend zu ermöglichen.
- **Wichtige Methoden (Beispiele):**
    - `static ActionManager* instance();`
    - `void registerAction(VivoXAction* action);` (oder eine Definition)
    - `VivoXAction* getAction(const ActionID& id) const;`
    - `bool isActionEnabled(const ActionID& id) const;`
    - `bool isActionChecked(const ActionID& id) const;`
    - `void triggerAction(const ActionID& id);`
    - Slots zur Entgegennahme von Kontextänderungs-Signalen (z.B. `onSelectionChanged(...)`).
- **Wichtige Signale (Beispiele):**
    - `void actionRegistered(const ActionID& id);`
    - `void actionStateChanged(const ActionID& id, const QByteArray& propertyName);` (z.B. für "enabled", "checked")
    - `void preActionTriggered(const ActionID& id);` (für Logging/Hooks)
    - `void postActionTriggered(const ActionID& id);` (für Logging/Hooks)
- **Interne Speicherung:** Eine `QHash<ActionID, VivoXAction*>` oder `QMap` zur Speicherung der registrierten Aktionen.

### Integration mit Undo/Redo 27

Der `ActionManager` spielt eine Schlüsselrolle bei der Integration mit dem Undo/Redo-System. Wenn `triggerAction` für eine rückgängig machbare Aktion aufgerufen wird, muss sichergestellt werden, dass der entsprechende `QUndoCommand` erzeugt wird (diese Aufgabe wird typischerweise an die Implementierung der `VivoXAction` delegiert) und auf den zentralen `QUndoStack` der Anwendung mittels `push()` gelegt wird.27

Die Zentralisierung der Kontextbewertung im `ActionManager` ist ein Kernaspekt des Designs. Anstatt jedes UI-Element den Kontext prüfen zu lassen, überwacht der Manager den Zustand und aktualisiert proaktiv den Status (`enabled`, `checked`) der Aktionen. UI-Elemente reagieren dann nur noch auf die `actionStateChanged`-Signale. Dies ist effizient und stellt Konsistenz sicher.

Ebenso zentralisiert die `triggerAction(actionId)`-Methode die Aktionsausführung.1 UI-Elemente und andere Auslöser (Shortcuts, Befehlspalette) müssen nur die ID kennen und diese Methode aufrufen. Der Manager kümmert sich um Nachschlagen, Zustandsprüfung, Ausführung und Undo/Redo-Integration, was die UI-Komponenten vereinfacht und die Logik entkoppelt.

## 7. QML-Integrationsschicht: `ActionRegistry`-Design

Die `ActionRegistry` dient als Brücke zwischen dem C++ `ActionManager` und der QML-basierten Benutzeroberfläche. Sie macht Aktionen in QML auffindbar, deren Zustand beobachtbar und deren Ausführung aus QML heraus möglich.

### Mechanismus zur Bereitstellung von Aktionen 44

Mehrere Ansätze sind denkbar, um Aktionen in QML verfügbar zu machen:

- **Singleton:** Die `ActionRegistry` (oder eine Fassade des `ActionManager`) wird als QML-Singleton-Typ registriert (`qmlRegisterSingletonType` oder CMake `QML_SINGLETON_TYPE`).45 Dies ermöglicht globalen Zugriff in QML über eine `import`-Anweisung und ist für anwendungsweite Dienste oft der bevorzugte Ansatz.
- **Context Property:** Die Registry wird als Eigenschaft des Wurzelkontexts gesetzt (`QQmlContext::setContextProperty`).47 Dies ist einfacher, kann aber bei komplexen Anwendungen zu einem überladenen globalen Kontext führen.
- **Model-Ansatz:** Aktionen werden als `QAbstractListModel` (oder `QAbstractTableModel`) bereitgestellt.44 Dies ist ideal für dynamische Menüs, Werkzeugleisten oder die Befehlspalette, die eine Liste von Aktionen anzeigen müssen. Das Modell würde Aktions-Eigenschaften (`id`, `text`, `icon`, `enabled`, `checked`, `shortcut`) als Rollen (`roleNames`) verfügbar machen.44

### Binding von Aktionseigenschaften an QML-Controls 15

QML-UI-Elemente wie `Button`, `MenuItem` oder `ToolButton` müssen ihre relevanten Eigenschaften (`enabled`, `checked`, `text`, `icon`, `shortcut`) an die entsprechenden Eigenschaften der Aktion binden, die sie repräsentieren.

- **QML `Action` Typ:** `MenuItem` und `ToolButton` in Qt Quick Controls haben eine `action`-Eigenschaft, die an eine QML `Action`-Instanz gebunden werden kann.15 Die `ActionRegistry` müsste diese QML `Action`-Instanzen bereitstellen oder verwalten, synchronisiert mit den C++-Aktionen.
- **Direktes Binding (Singleton/Context Property):** Wenn C++-Aktionseigenschaften direkt über die Registry verfügbar sind, kann QML direkt binden: `enabled: ActionRegistry.isActionEnabled("File.Save")`. Dies erfordert, dass die `ActionRegistry` `Q_PROPERTY`-Deklarationen mit `NOTIFY`-Signalen für die relevanten Zustände bereitstellt, damit die QML-Bindings automatisch aktualisiert werden.70 Das `Binding`-Element in QML kann verwendet werden, um explizit an Eigenschaften von Objekten zu binden, die nicht direkt in QML instanziiert wurden.67
- **Model Binding:** Bei Verwendung eines Modells binden die Delegaten in `ListView` oder `Repeater` an die Rollen des Modells: `text: model.text`, `enabled: model.enabled`.44

### Auslösen von Aktionen aus QML 65

Wenn ein UI-Element in QML aktiviert wird (z.B. `Button.clicked`, `MenuItem.triggered`), muss die entsprechende C++-Aktion ausgelöst werden.

- **Implementierung:** Das Signal des QML-Elements (z.B. `onClicked`) wird mit einer JavaScript-Funktion verbunden, die eine `Q_INVOKABLE`-Methode der `ActionRegistry` aufruft, z.B. `ActionRegistry.triggerAction("File.Save")`.65 Alternativ kann das `triggered()`-Signal einer QML `Action` direkt mit einem C++-Slot verbunden werden, wenn die `ActionRegistry` dies ermöglicht oder wenn die QML `Action` korrekt an das C++-Backend gebunden ist. Die `Connections`-Komponente in QML kann ebenfalls verwendet werden, um QML-Signale mit C++-Slots zu verbinden.88

### Vorgeschlagene QML-C++ Interaktionsstrategie

1. **`ActionRegistry` als C++ Singleton:** Eine C++ Klasse `ActionRegistry` wird implementiert und als QML-Singleton-Typ (z.B. unter `VivoX.Actions 1.0`) registriert.
2. **Zustands-Exposition:** Die `ActionRegistry` beobachtet den C++ `ActionManager` (verbindet sich mit dessen `actionStateChanged`-Signal). Sie macht die relevanten Zustände der Aktionen (mindestens `enabled` und `checked` für jede bekannte `actionId`) über `Q_PROPERTY`-Deklarationen mit `NOTIFY`-Signalen für QML zugänglich. Methoden wie `isActionEnabled(actionId)` und `isActionChecked(actionId)` werden bereitgestellt.
3. **Auslöse-Methode:** Eine `Q_INVOKABLE`-Methode `triggerAction(const QString& actionId)` wird implementiert, die intern `ActionManager::triggerAction` aufruft.
4. **Aktions-Modell:** Die `ActionRegistry` implementiert oder stellt ein `QAbstractListModel` bereit, das die Liste der _aktuell verfügbaren_ (kontextuell relevanten und aktivierten) Aktionen enthält. Dieses Modell wird dynamisch aktualisiert, wenn sich der Kontext ändert. Die Rollen des Modells umfassen mindestens `actionId`, `text`, `iconSource`, `shortcutText`, `enabled`, `checkable`, `checked`. Dieses Modell dient als Datenquelle für die Befehlspalette und potenziell für dynamische Menüs/Toolbars.

Die Verwendung eines Singletons 45 bietet einen klaren, global zugänglichen Endpunkt für QML, um mit dem Aktionssystem zu interagieren, ohne die Registry durch die Komponenten-Hierarchie reichen zu müssen.

Die Bereitstellung eines `QAbstractListModel` 51 ist besonders vorteilhaft für dynamische UI-Elemente wie die Befehlspalette oder Menüs, die auf der Liste der verfügbaren Aktionen basieren.92 Die Delegaten dieser Views können direkt an die über Rollen bereitgestellten Aktionseigenschaften binden. Es ist entscheidend, dass dieses Modell dynamisch auf Kontextänderungen reagiert und die View korrekt über Änderungen mittels `dataChanged`, `beginInsertRows`, `endRemoveRows` etc. informiert.44

## 8. Eingabebindung: Shortcuts und Gesten

Die Aktionsbibliothek muss es ermöglichen, Aktionen nicht nur über explizite UI-Elemente, sondern auch über Tastaturkürzel und potenziell Gesten auszulösen.

### Mapping von Tastaturkürzeln 15

Der primäre Mechanismus zur Definition von Standard-Shortcuts ist die `shortcut`-Eigenschaft der `VivoXAction`. Bei der Registrierung einer Aktion liest der `ActionManager` diese Eigenschaft aus und erstellt ein `QShortcut`-Objekt.20 Dieses Objekt wird mit dem passenden Eltern-Kontext (meist das Fenster, also `Qt::WindowShortcut` 20) initialisiert. Das `activated()`-Signal des `QShortcut` wird mit einem Slot oder Lambda verbunden, das `ActionManager::triggerAction(actionId)` aufruft. Die QML `Action` hat ebenfalls eine `shortcut`-Eigenschaft 15; die Definition im C++-Backend sollte als maßgeblich betrachtet werden, um Konsistenz zu gewährleisten.

### Handhabung potenzieller Gesteneingabe 94

Falls Gestensteuerung in VivoX benötigt wird, bietet QML leistungsfähige Input Handler. Klassen wie `TapHandler` 94, `DragHandler` 101, `PinchHandler` etc. können in QML-Komponenten verwendet werden, um spezifische Gesten zu erkennen. Wenn eine Geste erkannt wird (z.B. `TapHandler::longPressed`), kann das entsprechende Signal des Handlers (`onLongPressed`) genutzt werden, um `ActionRegistry.triggerAction(actionId)` mit der ID der Aktion aufzurufen, die dieser Geste in diesem spezifischen UI-Kontext zugeordnet ist. Im Gegensatz zu globalen Shortcuts wird die Zuordnung von Gesten zu Aktionen typischerweise innerhalb der QML-Komponente definiert, in der die Geste relevant ist.

### Konflikterkennung und -lösung

- **Shortcuts:** Qt signalisiert mehrdeutige Shortcut-Sequenzen (wenn eine Sequenz der Anfang einer anderen ist) mit `QShortcut::activatedAmbiguously()`.21 Der `ActionManager` benötigt eine Strategie dafür, z.B. eine Warnung loggen oder basierend auf Prioritäten entscheiden. Konflikte zwischen `ApplicationShortcut` und System-Shortcuts müssen im Design vermieden werden.
- **Gesten:** Konflikte zwischen verschiedenen QML Input Handlers auf demselben Element müssen durch Konfiguration der Handler-Eigenschaften (z.B. `acceptedButtons`, `acceptedModifiers`, `grabPermissions`) und der Ereignisweiterleitung in QML gelöst werden.96

### Strategie zur Benutzeranpassung

Es sollte vorgesehen werden, dass Benutzer Tastaturkürzel anpassen können. Dies erfordert:

1. **Persistente Speicherung:** Benutzerdefinierte Shortcuts müssen gespeichert werden (z.B. mittels `QSettings`).
2. **Laden und Anwenden:** Der `ActionManager` muss beim Start die benutzerdefinierten Shortcuts laden und diese anstelle der Standard-Shortcuts aus der Aktionsdefinition verwenden, wenn er die `QShortcut`-Objekte erstellt.
3. **UI zur Anpassung:** Eine Benutzeroberfläche (z.B. ein Einstellungsdialog) muss bereitgestellt werden, die dem Benutzer erlaubt, die Liste aller Aktionen (abrufbar über den `ActionManager`/`ActionRegistry`) anzuzeigen und deren zugeordnete Shortcuts zu ändern. Diese UI interagiert mit dem `ActionManager`, um die benutzerdefinierten Mappings zu speichern.

Sowohl Tastaturkürzel 20 als auch Gesten 94 sind Eingabeereignisse, die als alternative Auslöser (Invoker) für Aktionen dienen. Die Aktionsbibliothek muss diese Ereignisse auf die abstrakten Action-IDs abbilden. Der `ActionManager` (für Shortcuts) und spezifische QML-Komponenten (für Gesten) fungieren als Adapter, die diese Übersetzung vornehmen und `ActionManager::triggerAction(actionId)` aufrufen, wodurch die Entkopplung gewahrt bleibt.

Die Notwendigkeit der Benutzeranpassung von Shortcuts impliziert, dass der `ActionManager` von Anfang an flexibel gestaltet sein muss. Er muss die Standard-Shortcuts kennen, aber in der Lage sein, diese durch benutzerdefinierte Werte aus einer persistenten Konfiguration zu überschreiben, bevor die `QShortcut`-Objekte tatsächlich instanziiert werden.

## 9. Undo/Redo-Integrationsstrategie

Eine robuste Undo/Redo-Funktionalität ist eine Kernanforderung und wird durch die Integration mit Qt's Undo Framework realisiert, das eng mit dem Command Pattern verknüpft ist.

### Verbindung von Aktionen zu `QUndoCommand` 25

Jede Aktion, die den Zustand der Anwendung modifiziert und rückgängig gemacht werden soll, muss einem konkreten `QUndoCommand` entsprechen.27 Diese von `QUndoCommand` abgeleitete Klasse implementiert die Methoden `redo()` (führt die Aktion aus) und `undo()` (macht die Aktion rückgängig).41 Sie muss alle Zustandsinformationen speichern, die für beide Operationen notwendig sind. Die Ausführungslogik einer rückgängig machbaren `VivoXAction`, die vom `ActionManager::triggerAction` aufgerufen wird, ist dafür verantwortlich, die passende `QUndoCommand`-Instanz mit dem aktuellen Kontext und Parametern zu erzeugen.

### Rolle des `ActionManager` beim Pushen von Befehlen 26

Es wird ein zentraler `QUndoStack` für die Anwendung (oder pro Dokument) angenommen. Nachdem eine Aktion ausgelöst und der zugehörige `QUndoCommand` erstellt wurde, muss der `ActionManager` (oder die von ihm aufgerufene Aktionslogik) diesen Befehl mittels `QUndoStack::push()` auf den Stack legen.27 Der Aufruf von `push()` führt automatisch die `redo()`-Methode des Befehls zum ersten Mal aus.43

### Design rückgängig machbarer `VivoXAction`-Implementierungen

Es muss eine Konvention oder eine Erweiterung der `VivoXAction`-Schnittstelle definiert werden, die angibt, ob eine Aktion rückgängig machbar ist und wie sie ihren `QUndoCommand` erzeugt. Beispielsweise könnte eine `UndoableVivoXAction`-Unterklasse eine Methode `createUndoCommand(context)` bereitstellen, die von `ActionManager::triggerAction` aufgerufen wird.

### Integration mit UI (Undo/Redo Actions) 25

Qt's `QUndoStack` bietet praktische Methoden zur Erzeugung von `QAction`-Objekten für Undo und Redo: `createUndoAction()` und `createRedoAction()`.25 Diese Aktionen aktualisieren automatisch ihren Text (z.B. "Undo Typing") und ihren Aktivierungszustand (`enabled`) basierend auf dem Zustand des Stacks. Diese generierten `QAction`s sollten in das Bearbeiten-Menü ("Edit") der Anwendung und optional in die Hauptwerkzeugleiste eingefügt werden. Wenn die UI in QML realisiert wird, muss die `ActionRegistry` den Zustand (Text, enabled) dieser C++-generierten Undo/Redo-Aktionen für QML verfügbar machen, z.B. durch das Bereitstellen entsprechender QML `Action`-Objekte, deren Eigenschaften gebunden werden.

### Command Compression und Macros 26

- **Kompression:** Die Methode `QUndoCommand::mergeWith()` kann implementiert werden, um aufeinanderfolgende, ähnliche Befehle zu einem einzigen zusammenzufassen (z.B. einzelne Tastatureingaben zu einem Wort).26 Dies sollte in relevanten `QUndoCommand`-Unterklassen erfolgen.
- **Makros:** Sequenzen von Befehlen, die als eine Einheit rückgängig gemacht werden sollen, können als Makros implementiert werden. Dies geschieht entweder durch explizite Eltern-Kind-Beziehungen zwischen `QUndoCommand`-Objekten oder durch die Verwendung von `QUndoStack::beginMacro()` und `endMacro()`.26 Der `ActionManager` kann das Starten und Beenden von Makros für komplexe Aktionen koordinieren.

Qt's Undo Framework basiert explizit auf dem Command Pattern.42 Jede `QUndoCommand`-Instanz _ist_ ein konkretes Command-Objekt, das eine Aktion und deren Umkehrung kapselt.41 Der `QUndoStack` verwaltet die Historie dieser Befehle.25 Die Integration mit diesem Framework bedeutet daher, das Command Pattern für alle zustandsändernden, rückgängig machbaren Aktionen zu übernehmen.

Da der `ActionManager` der zentrale Punkt zum Auslösen von Aktionen ist, ist er auch die logische Stelle, um die Interaktion mit dem `QUndoStack` zu orchestrieren.27 Er muss sicherstellen, dass für rückgängig machbare Aktionen der korrekte `QUndoCommand` instanziiert und auf den Stack gepusht wird. Dies erfordert eine Unterscheidung zwischen undoable und non-undoable Aktionen innerhalb der `triggerAction`-Logik.

## 10. Zusammenfassung des Bauplans und Implementierungsempfehlungen

### Architektur-Rekapitulation

Die vorgeschlagene Architektur für die VivoX-Aktionsbibliothek basiert auf der Kapselung von Aktionen im `VivoXAction`-Konzept, das durch Qt's `QAction` unterstützt wird. Ein zentraler C++ `ActionManager` (als Singleton implementiert) verwaltet die Registrierung, das Nachschlagen, die kontextabhängige Zustandsbewertung (`enabled`, `checked`), die Auslösung von Aktionen (inkl. `QUndoCommand`-Erzeugung und `QUndoStack`-Interaktion) sowie die Verwaltung von `QShortcut`-Objekten. Eine QML `ActionRegistry` (ebenfalls als Singleton) dient als Brücke zur QML-UI, macht Aktionszustände über Properties mit NOTIFY-Signalen verfügbar, bietet eine invokierbare Methode zum Auslösen von Aktionen aus QML und stellt potenziell ein `QAbstractListModel` der verfügbaren Aktionen für dynamische UI-Elemente wie die Befehlspalette bereit. UI-Elemente (Menüs, Toolbars, Befehlspalette) binden an die `ActionRegistry` und lösen Aktionen über deren Schnittstelle aus.

### Wichtige Designentscheidungen

- **Nutzung von Qt-Infrastruktur:** Starkes Aufsetzen auf `QAction`, `QShortcut`, `QActionGroup` und `QUndoStack`/`QUndoCommand`.
- **Entkopplung durch IDs:** Verwendung eindeutiger, stabiler IDs zur Referenzierung von Aktionen.
- **Zentralisierte Kontextbewertung:** Der `ActionManager` ist die einzige Quelle der Wahrheit für den Zustand (`enabled`, `checked`) von Aktionen.
- **QML-Integration über Singleton:** Globale Verfügbarkeit der `ActionRegistry` in QML.
- **Model für dynamische Listen:** Bereitstellung eines `QAbstractListModel` für die Befehlspalette und dynamische Menüs.
- **Undo/Redo über `QUndoCommand`:** Implementierung rückgängig machbarer Aktionen durch Ableitung von `QUndoCommand`.

### Implementierungsphasen

Eine schrittweise Implementierung wird empfohlen:

1. Definition der `VivoXAction`-Struktur und Kerneigenschaften.
2. Implementierung des Basis-`ActionManager` (Registrierung, Lookup, einfache Auslösung).
3. Implementierung der Basis-`ActionRegistry` (Singleton, einfache Zustands-Exposition, Auslösung).
4. Integration mit statischen UI-Elementen (z.B. einfache Menüs/Buttons in QML oder Widgets).
5. Implementierung der Kontextbewertung und dynamischen Zustandsaktualisierung im `ActionManager` und Propagierung zur `ActionRegistry`.
6. Hinzufügen der Shortcut-Verwaltung (`QShortcut`).
7. Implementierung der Undo/Redo-Funktionalität (`QUndoCommand`, `QUndoStack`-Integration im `ActionManager`).
8. Entwicklung der Befehlspaletten-UI (unter Verwendung des Aktions-Modells).
9. Integration von Gestenerkennung und -auslösung (falls erforderlich).
10. Implementierung der Benutzeranpassung für Shortcuts.

### Teststrategie

- **Unit-Tests:** Für jede konkrete `VivoXAction`-Implementierung (insbesondere deren `execute`/`redo`/`undo`-Logik in `QUndoCommand`).
- **Integrationstests:** Für den `ActionManager` (Kontextbewertung, Zustandsänderungen, Triggering, Undo/Redo-Interaktion) und die `ActionRegistry` (Signalweiterleitung, Modellaktualisierung, Binding-Verhalten).
- **UI-Tests:** Überprüfung der korrekten Zustandsanzeige in Menüs, Toolbars, Befehlspalette und der korrekten Auslösung durch UI-Interaktion und Shortcuts.

### Zukünftige Überlegungen

Mögliche zukünftige Erweiterungen umfassen die Integration mit Skripting-Schnittstellen, die Unterstützung komplexerer Aktionsketten oder Workflows und die Einführung eines fortgeschritteneren Regelwerks für die Kontextbewertung, falls die Komplexität der Anwendung dies erfordert. Die hier vorgestellte Architektur bietet eine solide Grundlage für solche Erweiterungen.