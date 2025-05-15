# Detaillierter Bauplan: VivoX UI-Bibliothek (VivoXLib)

## 1. Einleitung

Dieses Dokument beschreibt den detaillierten Bauplan für die UI-Bibliothek des VivoX-Projekts, nachfolgend als "VivoXLib" bezeichnet. VivoXLib dient als Grundlage für die Benutzeroberfläche der VivoX-Desktop-Umgebung und zielt darauf ab, eine konsistente, moderne, effiziente und ästhetisch ansprechende Benutzererfahrung zu schaffen.

**Zielsetzung:**

Das primäre Ziel von VivoXLib ist die Bereitstellung eines wiederverwendbaren Satzes von UI-Komponenten und Richtlinien, die speziell auf die Anforderungen von VivoX zugeschnitten sind. Dies umfasst:

- **Konsistenz:** Sicherstellung eines einheitlichen Erscheinungsbilds und Verhaltens über die gesamte VivoX-Umgebung hinweg.
- **Effizienz:** Ermöglichung einer schnellen und ressourcenschonenden Entwicklung von UI-Elementen.
- **Anpassbarkeit:** Bereitstellung eines flexiblen Theming-Systems, das einfache Anpassungen ermöglicht, insbesondere im Hinblick auf das primäre Dark Theme.
- **Modernität:** Schaffung einer visuell ansprechenden und zeitgemäßen Ästhetik.
- **Wartbarkeit:** Strukturierung der Bibliothek nach bewährten Mustern (Atomic Design) zur Vereinfachung von Wartung und Erweiterung.

**Inspiration und Adaption:**

VivoXLib lässt sich stark von den Prinzipien und Komponenten etablierter Designsysteme inspirieren, insbesondere von Googles Material Design (mit Fokus auf Material 3). Material Design bietet eine umfassende Grundlage für Designphilosophie, Komponenten, Theming (Farben, Typografie, Formen, Bewegung), Layout und Interaktion.1

Die Herausforderung und Kernaufgabe besteht jedoch darin, diese Prinzipien sinnvoll für eine **Desktop-Umgebung** unter **Linux/Wayland** zu adaptieren. Dies erfordert spezifische Anpassungen:

- **Dark Theme First:** Im Gegensatz zu vielen Designsystemen, die primär für helle Themes konzipiert sind, ist VivoX dunkel-fokussiert. VivoXLib muss daher von Grund auf für optimale Lesbarkeit und Ästhetik in dunklen Umgebungen ausgelegt sein, unter Berücksichtigung der Material Design Richtlinien für Dark Themes (z.B. Nutzung von dunklem Grau statt Schwarz, angepasste Elevation durch Oberflächen-Overlays, desaturierte Akzentfarben).4
- **Desktop-Interaktion:** Interaktionsmuster müssen für Maus und Tastatur optimiert werden, während Material Design oft einen Mobile-First-Ansatz verfolgt. Hover-Zustände, Tastaturnavigation und Fokus-Indikatoren sind auf dem Desktop von zentraler Bedeutung.
- **Technologie-Stack:** Die Implementierung erfolgt in Qt 6/QML mit einem C++20 Backend [User Query]. Dies beeinflusst die technische Umsetzung von Komponenten, Theming und Layout.

**Technologie-Stack:**

- **UI-Framework:** Qt 6 / QML [User Query]
- **Programmiersprache:** C++20 (Backend), QML/JavaScript (Frontend) [User Query]
- **Grafik-Backend:** Wayland [User Query]
- **Architekturprinzip:** Atomic Design [User Query]
- **Primäres Theme:** Dark Theme [User Query]

Dieses Dokument dient als Leitfaden für Designer und Entwickler, die an der VivoX-Benutzeroberfläche arbeiten, und legt die Struktur, die Komponenten, das Theming und die Implementierungsstrategie für VivoXLib fest.

## 2. Designphilosophie

Die Designphilosophie von VivoXLib bildet das Fundament für alle Gestaltungsentscheidungen und leitet sich aus den Projektzielen und der Adaption von Material Design ab. Sie umfasst folgende Kernprinzipien:

- **Klarheit (Clarity):** Die Benutzeroberfläche muss intuitiv und leicht verständlich sein. Informationen sollen klar strukturiert und Aktionen eindeutig erkennbar sein. Dies wird erreicht durch logische Layouts, klare visuelle Hierarchien (unterstützt durch Typografie und Elevation) und verständliche Beschriftungen.1
- **Effizienz (Efficiency):** Die Interaktion mit der VivoX-Umgebung soll schnell und reibungslos erfolgen. Dies betrifft sowohl die Performance der UI-Komponenten als auch die Gestaltung von Workflows, die Nutzer dabei unterstützen, ihre Ziele mit minimalem Aufwand zu erreichen. Komponenten müssen performant sein, insbesondere unter Qt 6/QML.5
- **Konsistenz (Consistency):** Ein einheitliches Erscheinungsbild und Verhalten über alle Teile der Anwendung hinweg ist entscheidend für die Benutzerfreundlichkeit und Markenidentität. VivoXLib stellt sicher, dass gleiche Elemente immer gleich aussehen und funktionieren. Dies wird durch die zentrale Definition von Komponenten und Theming erreicht.2
- **Anpassbarkeit (Customizability):** Während Konsistenz wichtig ist, muss das System auch Flexibilität bieten. Das Theming-System soll einfache Anpassungen von Farben, Schriftarten und Formen ermöglichen, um zukünftigen Anforderungen oder Benutzerpräferenzen gerecht zu werden, ohne die Kernkonsistenz zu brechen.2
- **Visuelle Ästhetik (Visual Aesthetics):** Die Benutzeroberfläche soll modern, ansprechend und augenschonend sein. Der Fokus liegt auf einem eleganten Dark Theme, das durchdachte Farbpaletten, klare Typografie und subtile Animationen nutzt, um eine hochwertige Anmutung zu erzeugen.4 Die Ästhetik orientiert sich an Material Designs Prinzipien von "Bold, graphic, intentional" 2, adaptiert für eine dunkle Desktop-Umgebung.
- **Fokus auf Dark Theme:** Das Design ist primär für dunkle Umgebungen optimiert. Kontraste, Farbsättigung und Helligkeitsabstufungen werden sorgfältig gewählt, um Blendung zu vermeiden und Lesbarkeit zu maximieren, basierend auf Material Design Empfehlungen für Dark Themes.4
- **Desktop-Orientierung:** Alle Designentscheidungen berücksichtigen die spezifischen Anforderungen und Interaktionsmuster einer Desktop-Umgebung (Maus, Tastatur, größere Bildschirme, Fensterverwaltung).

Die Adaption von Material Design für den Desktop erfordert eine bewusste Abwägung. Während die Kernprinzipien von Material (Metapher, Intentionalität, Bewegung) 2 übernommen werden, müssen Implementierungsdetails wie die Ausprägung von Schatten, die Intensität von Animationen oder die Größe von Touch-Targets für den Desktop-Kontext angepasst werden. Beispielsweise werden Elevation und Hierarchie im Dark Theme nicht nur durch Schatten, sondern auch durch subtile Helligkeitsanpassungen der Oberflächen erreicht, wie in Material Design beschrieben.4 Die Notwendigkeit dieser Anpassung ergibt sich direkt aus der Kombination der Material Design Richtlinien für dunkle Themes und dem primären Fokus von VivoX auf ein Dark Theme. Die Designphilosophie muss Entwicklern daher konkrete Anleitungen geben, _wie_ diese Adaption erfolgen soll.

## 3. Komponentenarchitektur (Atomic Design)

VivoXLib folgt dem Atomic Design-Prinzip, um eine modulare, wiederverwendbare und wartbare Komponentenstruktur zu schaffen.9 Diese Methodik gliedert die UI in hierarchische Ebenen: Atome, Moleküle, Organismen, Templates und Views/Pages.

### 3.1 Abbildung von Atomic Design auf QML

Atomic Design wird in VivoXLib wie folgt auf die QML-Entwicklung abgebildet:

- **Implementierung:** Atome, Moleküle und Organismen werden als wiederverwendbare QML-Komponenten implementiert, wobei jede Komponente in einer eigenen `.qml`-Datei definiert wird.13
- **Struktur:** Die Komponenten werden in einer Verzeichnisstruktur organisiert, die die atomare Hierarchie widerspiegelt (z.B. `VivoXLib/Atoms/Button.qml`, `VivoXLib/Molecules/SearchField.qml`).
- **Modul:** Alle Komponenten werden über ein zentrales QML-Modul (z.B. `VivoXLib 1.0`) bereitgestellt. Dieses Modul wird mithilfe von CMake's `qt_add_qml_module` Funktion definiert, welche automatisch die notwendige `qmldir`-Datei generiert und die Komponenten für den Import (`import VivoXLib 1.0`) verfügbar macht.13

Ein grundlegendes Prinzip des Atomic Design ist die Kapselung und die unidirektionale Abhängigkeit: Atome kennen keine Moleküle, Moleküle keine Organismen usw..11 Dies fördert die Entkopplung, erfordert aber klare Schnittstellen. In der QML-Implementierung bedeutet dies, dass die Kommunikation und Datenweitergabe primär über Properties, Signale und nach unten gereichte Modelle erfolgen muss. Komponenten dürfen nicht auf übergeordnete Elemente oder Elemente auf derselben Ebene zugreifen, die nicht explizit über ihre API bereitgestellt werden.

### 3.2 Atome (Basiselemente)

Atome sind die kleinsten, nicht weiter teilbaren Bausteine der UI. Sie sind generisch, über das Theming-System stark anpassbar und bilden die Grundlage für komplexere Komponenten.9 Die Granularität der Atome ist entscheidend: Sie sollten funktional sinnvoll und wiederverwendbar sein, inspiriert von grundlegenden Material Design Komponenten 19, aber nicht zu primitiv (wie ein einfaches `Rectangle`).

- **`VxButton`**: Basis-Button-Komponente.
    - _Inspiration:_ Material Design Buttons.19
    - _Properties:_ `text` (QString), `iconSource` (QUrl/QString), `role` (enum: Primary, Secondary, Tertiary, Text), `enabled` (bool).
    - _Zustände:_ Default, Hover, Pressed, Disabled, Focused (visuell über Theme definiert).
    - _Beispiel QML (Atom):_ `Atoms/VxButton.qml`
- **`VxIcon`**: Zeigt Icons aus dem definierten Icon-Set an.
    - _Properties:_ `source` (QString - Icon-Name/Key), `color` (QColor - erbt vom Theme, überschreibbar), `size` (QSize/int).
    - _Beispiel QML (Atom):_ `Atoms/VxIcon.qml`
- **`VxTextField`**: Einzeiliges Texteingabefeld.
    - _Inspiration:_ Material Design Text Fields.19
    - _Properties:_ `text` (QString), `placeholderText` (QString), `readOnly` (bool), `validator` (QValidator), `echoMode` (enum). Enthält intern ggf. ein kleines "Clear"-Button-Atom.
    - _Zustände:_ Default, Focused, Error, Disabled (visuell über Theme definiert).
    - _Beispiel QML (Atom):_ `Atoms/VxTextField.qml`
- **`VxLabel`**: Zeigt Text an.
    - _Properties:_ `text` (QString), `fontRole` (QString - mapst auf Typografie-Skala im Theme), `colorRole` (QString - mapst auf Farbpalette im Theme), `wrapMode` (enum), `elide` (enum).
    - _Beispiel QML (Atom):_ `Atoms/VxLabel.qml`
- **`VxSwitch`**: Umschalter (Toggle).
    - _Inspiration:_ Material Design Switch.19
    - _Properties:_ `checked` (bool), `text` (QString - optional), `enabled` (bool).
    - _Beispiel QML (Atom):_ `Atoms/VxSwitch.qml`
- **`VxCheckbox`**: Kontrollkästchen.
    - _Inspiration:_ Material Design Checkbox.19
    - _Properties:_ `checked` (bool/int - für tristate), `text` (QString - optional), `tristate` (bool), `enabled` (bool).
    - _Beispiel QML (Atom):_ `Atoms/VxCheckbox.qml`
- **`VxRadioButton`**: Radio-Button.
    - _Inspiration:_ Material Design RadioButton.19
    - _Properties:_ `checked` (bool), `text` (QString - optional), `enabled` (bool). Gehört typischerweise zu einer `ButtonGroup`.
    - _Beispiel QML (Atom):_ `Atoms/VxRadioButton.qml`
- **`VxAvatar`**: Zeigt Benutzerbilder oder Initialen an.
    - _Properties:_ `source` (QUrl/QString), `initials` (QString), `size` (enum/int), `shape` (enum: Circle, Square).
    - _Beispiel QML (Atom):_ `Atoms/VxAvatar.qml`
- **`VxBadge`**: Kleiner Statusindikator.
    - _Inspiration:_ Material Design Badge.19
    - _Properties:_ `text` (QString), `count` (int), `colorRole` (QString), `visible` (bool).
    - _Beispiel QML (Atom):_ `Atoms/VxBadge.qml`
- **`VxDivider`**: Visuelle Trennlinie.
    - _Inspiration:_ Material Design Divider.19
    - _Properties:_ `orientation` (enum: Horizontal, Vertical).
    - _Beispiel QML (Atom):_ `Atoms/VxDivider.qml`
- **`VxProgressIndicator`**: Linearer oder zirkulärer Fortschrittsindikator.
    - _Inspiration:_ Material Design ProgressIndicator.19
    - _Properties:_ `value` (real - 0.0 bis 1.0), `indeterminate` (bool), `visible` (bool).
    - _Beispiel QML (Atom):_ `Atoms/VxProgressIndicator.qml`
- **`VxSurface`**: Basis-Oberflächenelement, das Theme-Farbe und Elevation anwendet.
    - _Hinweis:_ Dies ist eher ein internes Atom, das von anderen Komponenten zur Darstellung von Hintergründen und Containern verwendet wird. Es kapselt die Logik zur Anwendung der Oberflächenfarbe basierend auf der Elevation im Dark Theme.4
    - _Beispiel QML (Atom):_ `Atoms/VxSurface.qml`

### 3.3 Moleküle (Einfache zusammengesetzte Elemente)

Moleküle sind funktionale Einheiten, die aus der Kombination mehrerer Atome entstehen.9 Sie erfüllen spezifischere Aufgaben als einzelne Atome.

- **`VxSearchField`**: Suchfeld-Komponente.
    - _Zusammensetzung:_ Kombiniert `VxTextField` (Atom) und `VxIcon` (Atom, Such-Icon), optional `VxButton` (Atom, Löschen-Icon).
    - _Properties:_ `text` (QString), `placeholderText` (QString).
    - _Signale:_ `searchTriggered(string text)`.
    - _Beispiel QML (Molekül):_ `Molecules/VxSearchField.qml`
- **`VxListItem`**: Repräsentiert einen Eintrag in einer Liste.
    - _Zusammensetzung:_ Kombiniert optional `VxAvatar` (Atom), `VxLabel` (Atom, primärer/sekundärer Text), optional `VxIcon` oder `VxButton` (Atome) für Aktionen.
    - _Properties:_ `primaryText` (QString), `secondaryText` (QString), `avatarSource` (QUrl/QString), `clickable` (bool).
    - _Zustände:_ Default, Hover, Selected (visuell über Theme definiert).
    - _Beispiel QML (Molekül):_ `Molecules/VxListItem.qml`
- **`VxChip`**: Kleines interaktives Element.
    - _Inspiration:_ Material Design Chip.19
    - _Zusammensetzung:_ Kombiniert `VxLabel` (Atom), optional `VxIcon` (Atom), optional `VxButton` (Atom, Schließen-Icon).
    - _Properties:_ `text` (QString), `iconSource` (QUrl/QString), `removable` (bool), `checkable` (bool), `checked` (bool).
    - _Beispiel QML (Molekül):_ `Molecules/VxChip.qml`
- **`VxLabeledControl`**: Kombiniert ein `VxLabel` (Atom) mit einem anderen Kontroll-Atom (z.B. `VxTextField`, `VxSwitch`, `VxCheckbox`). Stellt konsistentes Layout und Accessibility-Verbindung sicher.
    - _Properties:_ `labelText` (QString), `control` (Item - Referenz auf das eingebettete Kontroll-Atom).
    - _Beispiel QML (Molekül):_ `Molecules/VxLabeledControl.qml`
- **`VxToolbar`**: Eine einfache Leiste, die `VxButton` oder `VxIcon` Atome für Aktionen enthält.
    - _Properties:_ `contentData` (list/model - Liste von Aktionen/Buttons).
    - _Beispiel QML (Molekül):_ `Molecules/VxToolbar.qml`

### 3.4 Organismen (Komplexe UI-Abschnitte)

Organismen sind komplexere, eigenständige UI-Abschnitte, die aus Atomen und Molekülen zusammengesetzt sind.9 Sie bilden erkennbare Bereiche der Benutzeroberfläche.

- **`VxSidebar`**: Anwendungs-Seitenleiste.
    - _Zusammensetzung:_ Enthält `VxListItems` (Moleküle) für Navigation, gruppiert durch `VxLabel` (Atom, Überschriften) oder `VxDivider` (Atom). Kann ein `VxSearchField` (Molekül) enthalten.
    - _Properties:_ `model` (QObject* - Navigationsstruktur), `selectedItem` (QVariant).
    - _Beispiel QML (Organismus):_ `Organisms/VxSidebar.qml`
- **`VxTitleBar`**: Fenster-Titelleiste.
    - _Zusammensetzung:_ Enthält Fenster-Steuerelemente (Atome, plattformspezifisch oder benutzerdefiniert), `VxLabel` (Atom, Fenstertitel), optional `VxTabs` (Organismus) oder `VxToolbar` (Molekül) für Aktionen.
    - _Properties:_ `windowTitle` (QString), `actionsModel` (list/model).
    - _Beispiel QML (Organismus):_ `Organisms/VxTitleBar.qml`
- **`VxSpeedDialWidget`**: Ein Widget für die Speed Dial-Ansicht.
    - _Zusammensetzung:_ Kombiniert `VxIcon` (Atom), `VxLabel` (Atom), optional `VxProgressIndicator` (Atom) oder `VxBadge` (Atom). Könnte auf `VxCard` (Organismus) basieren.
    - _Properties:_ `title` (QString), `iconSource` (QUrl/QString), `statusText` (QString), `progressValue` (real).
    - _Beispiel QML (Organismus):_ `Organisms/VxSpeedDialWidget.qml`
- **`VxCard`**: Inhaltscontainer mit Elevation.
    - _Inspiration:_ Material Design Card.19
    - _Zusammensetzung:_ Enthält Kombinationen von Atomen und Molekülen (z.B. Kopfzeile, Text, Bild, Aktionsbuttons).
    - _Properties:_ `headerText` (QString), `contentItem` (Item), `actions` (list<Item>), `elevation` (int - mapst auf Theme-Elevation).
    - _Beispiel QML (Organismus):_ `Organisms/VxCard.qml`
- **`VxDialog`**: Modaler oder nicht-modaler Dialog.
    - _Inspiration:_ Material Design Dialog.19
    - _Zusammensetzung:_ Enthält `VxLabel` (Atom, Titel), Inhaltsbereich (Molekül/Organismus), `VxToolbar` (Molekül, Aktionsbuttons).
    - _Properties:_ `title` (QString), `contentItem` (Item), `actions` (list<QObject>), `modal` (bool), `visible` (bool).
    - _Beispiel QML (Organismus):_ `Organisms/VxDialog.qml`
- **`VxSettingsSection`**: Ein abgegrenzter Abschnitt innerhalb der Einstellungen.
    - _Zusammensetzung:_ Enthält `VxLabel` (Atom, Überschrift) und mehrere `VxLabeledControl` Moleküle oder andere relevante Steuerelemente.
    - _Properties:_ `title` (QString), `contentItems` (list<Item>).
    - _Beispiel QML (Organismus):_ `Organisms/VxSettingsSection.qml`
- **`VxTabs`**: Tab-Leiste zur Organisation von Inhalten.
    - _Inspiration:_ Material Design Tabs.19
    - _Zusammensetzung:_ Enthält mehrere Tab-Reiter (Moleküle, bestehend aus `VxLabel` und optional `VxIcon` Atomen) und verwaltet den zugehörigen Inhaltsbereich.
    - _Properties:_ `model` (list/model - Tab-Daten), `currentIndex` (int).
    - _Beispiel QML (Organismus):_ `Organisms/VxTabs.qml`

### 3.5 Templates & Views/Pages (Layout-Strukturen)

Templates definieren abstrakte Seitenlayouts, die die Anordnung von Organismen und Molekülen vorgeben. Views (oder Pages) sind konkrete Implementierungen dieser Templates mit spezifischem Inhalt und Logik.9

- **`SpeedDialTemplate`**: Definiert das Rasterlayout für `VxSpeedDialWidget`-Organismen. Legt fest, wie Widgets angeordnet und skaliert werden.
    - _Beispiel QML (Template):_ `Templates/SpeedDialTemplate.qml`
- **`SettingsTemplate`**: Definiert ein typisches zweispaltiges Layout für Einstellungen: links eine Navigationsliste (`VxSidebar` oder `VxListItems`-Molekül) und rechts der Inhaltsbereich, der dynamisch `VxSettingsSection`-Organismen lädt.
    - _Beispiel QML (Template):_ `Templates/SettingsTemplate.qml`
- **`DashboardTemplate`**: Definiert die Struktur für das Haupt-Dashboard/Übersicht. Ordnet verschiedene Informations-Organismen (z.B. `VxCard`s) in einem flexiblen Layout an.
    - _Beispiel QML (Template):_ `Templates/DashboardTemplate.qml`
- **`VivoXMainWindowView`**: Die konkrete Implementierung des Hauptfensters der Anwendung. Nutzt ein Template, das `VxTitleBar`, `VxSidebar` und den Hauptinhaltsbereich (der andere Views basierend auf der Navigation lädt) anordnet.
    - _Beispiel QML (View/Page):_ `Pages/VivoXMainWindowView.qml`

Die strikte Anwendung von Atomic Design erfordert sorgfältige Planung der Komponentengrenzen und Verantwortlichkeiten. Die QML-Implementierung muss robuste Property-Schnittstellen und Signal-Emissionen für jede Komponente bereitstellen, um Komposition und Kommunikation zu ermöglichen, ohne die Kapselung zu verletzen. Dabei sind Best Practices für die Erstellung benutzerdefinierter QML-Komponenten zu befolgen.13

## 4. Theming-System Spezifikation

Das Theming-System ist entscheidend für die Konsistenz und Anpassbarkeit von VivoXLib. Es definiert das visuelle Erscheinungsbild aller Komponenten und muss leicht zugänglich und modifizierbar sein.

### 4.1 QML Theming-Architektur

- **Ansatz:** Ein **Singleton QML-Objekt** (z.B. `Theme.qml`) wird verwendet, um alle Theme-Eigenschaften zentral zu verwalten. Dies umfasst Farben, Typografie, Abstände, Formen, Icons und Animationswerte.17 Singletons bieten globalen Zugriff innerhalb der QML-Engine.
- **Struktur:** Um die Komplexität zu managen, wird das Singleton intern strukturiert, z.B. durch verschachtelte `QtObject`-Instanzen für verschiedene Kategorien (`Theme.color`, `Theme.typography`, `Theme.spacing`, `Theme.shape`, `Theme.icon`, `Theme.motion`).21
- **Registrierung:** `Theme.qml` wird als Singleton-Typ innerhalb des `VivoXLib`-QML-Moduls registriert. Dies geschieht über CMake durch Setzen der `QT_QML_SINGLETON_TYPE` Property auf `TRUE` für die `Theme.qml`-Datei vor dem Aufruf von `qt_add_qml_module`.17 Alternativ kann dies manuell in der `qmldir`-Datei erfolgen: `singleton Theme 1.0 Theme.qml`.17
- **Zugriff:** Komponenten greifen über die Singleton-ID auf Theme-Eigenschaften zu. Beispiel: `Rectangle { color: Theme.color.surface }`, `Text { font: Theme.typography.body1.font }`.21
- **Dynamische Anpassung (Dark/Light):** Das Singleton kann eine `darkMode` Property (bool) bereitstellen. Andere Eigenschaften (z.B. Farben) können intern Logik enthalten, um basierend auf dem Wert von `darkMode` die entsprechenden Farbwerte für das dunkle oder helle Theme zurückzugeben. Änderungen an `darkMode` propagieren sich automatisch durch QMLs Property Binding.21

### 4.2 Farbsystem

Das Farbsystem ist auf das primäre Dark Theme von VivoX optimiert, definiert aber auch ein entsprechendes Light Theme für Vollständigkeit.

- **Semantische Namen:** Es werden semantische Farbrollen verwendet, die sich an Material Design orientieren: `primary`, `onPrimary`, `secondary`, `onSecondary`, `tertiary`, `onTertiary`, `error`, `onError`, `background`, `onBackground`, `surface`, `onSurface`, `outline`.2 Zusätzliche Rollen können bei Bedarf definiert werden (z.B. `surfaceDim`, `surfaceBright`, `accentHover`, `textPrimary`, `textSecondary`, `textDisabled`). "On-" Farben werden für Inhalte (Text, Icons) verwendet, die auf einer Fläche mit der entsprechenden Basis-Farbe platziert werden.2
- **Dark Theme Optimierung:** Die Farbwerte für das Dark Theme werden sorgfältig ausgewählt, um Blendung zu reduzieren und hohe Lesbarkeit zu gewährleisten. Dies beinhaltet die Verwendung eines dunklen Grautons (z.B. `#121212`) als Basisfarbe für Oberflächen (`surface`) anstelle von reinem Schwarz und die Verwendung von desaturierten Akzentfarben (`primary`, `secondary`), um visuelle Vibrationen zu vermeiden.4
- **Elevation im Dark Theme:** Höhenunterschiede zwischen Oberflächen werden im Dark Theme nicht nur durch Schatten, sondern primär durch subtile, helle Overlays auf der Oberflächenfarbe dargestellt. Je höher die Elevation, desto heller die Oberfläche.4 Das `Theme`-Singleton stellt hierfür spezifische Farben bereit (z.B. `Theme.color.surface_dp0`, `Theme.color.surface_dp1`, `Theme.color.surface_dp4`, `Theme.color.surface_dp8`, etc.).
- **Zustände:** Farben oder Overlays für Komponenten-Zustände (Hover, Pressed, Focused, Disabled, Selected) werden definiert. Für den Disabled-Zustand im Dark Theme werden spezifische Opazitätswerte verwendet (z.B. 38% für Text/Icons, 12% für Container-Fills/Outlines).4
- **Kontrast:** Alle Farbkombinationen, insbesondere Text auf Hintergrund, müssen ausreichende Kontrastverhältnisse gemäß WCAG AA-Standards erfüllen (mindestens 4.5:1 für normalen Text).4 Im Dark Theme muss heller Text auf dunklen Oberflächen gut lesbar sein.
- **Dynamische Farben (Ausblick):** Material 3 führt dynamische, benutzergenerierte oder inhaltsbasierte Farbpaletten ein.7 Die Implementierung in Qt/QML ist jedoch komplex und erfordert plattformspezifische Integrationen oder Inhaltsanalysen, die über den initialen Umfang von VivoXLib hinausgehen. Das semantische Farbsystem legt jedoch eine Grundlage für eine mögliche spätere Implementierung.

**Tabelle: VivoX Farbpaletten-Definition (Auszug - Dark Theme Fokus)**

|   |   |   |   |   |   |   |
|---|---|---|---|---|---|---|
|**Semantischer Name**|**Dark Theme HEX**|**Light Theme HEX**|**Dark Hover Overlay/Color**|**Dark Pressed Overlay/Color**|**Dark Disabled Opacity/Color**|**Beschreibung/Verwendung**|
|`primary`|#BB86FC|#6200EE|+4% White Overlay|+8% White Overlay|---|Hauptakzentfarbe (Buttons, aktive Zustände)|
|`onPrimary`|#000000|#FFFFFF|---|---|38%|Text/Icons auf `primary`|
|`secondary`|#03DAC6|#03DAC6|+4% White Overlay|+8% White Overlay|---|Sekundäre Akzentfarbe (Filter Chips, Fortschrittsbalken)|
|`onSecondary`|#000000|#000000|---|---|38%|Text/Icons auf `secondary`|
|`background`|#121212|#FFFFFF|---|---|---|Hauptfensterhintergrund|
|`onBackground`|#FFFFFF|#000000|---|---|38%|Text/Icons auf `background`|
|`surface` (dp0)|#121212|#FFFFFF|+4% White Overlay|+8% White Overlay|12% (Container)|Basisfarbe für Komponenten (Karten, Dialoge, Menüs)|
|`surface_dp1`|`#121212` + 5% Overlay|`#FFFFFF`|+4% White Overlay|+8% White Overlay|12% (Container)|Oberfläche mit 1dp Elevation|
|`surface_dp4`|`#121212` + 9% Overlay|`#FFFFFF`|+4% White Overlay|+8% White Overlay|12% (Container)|Oberfläche mit 4dp Elevation|
|`onSurface`|#FFFFFF|#000000|---|---|38%|Text/Icons auf `surface`|
|`error`|#CF6679|#B00020|+4% White Overlay|+8% White Overlay|---|Fehlerindikation|
|`onError`|#000000|#FFFFFF|---|---|38%|Text/Icons auf `error`|
|`outline`|`onSurface` @ 12%|`onSurface` @ 12%|`onSurface` @ 16%|`onSurface` @ 20%|12%|Ränder von Komponenten (z.B. Textfelder, inaktive Buttons)|
|`textPrimary`|`onSurface` @ 87%|`onSurface` @ 87%|---|---|38%|Primärer Text|
|`textSecondary`|`onSurface` @ 60%|`onSurface` @ 60%|---|---|38%|Sekundärer Text, Hinweistext|
|`textDisabled`|`onSurface` @ 38%|`onSurface` @ 38%|---|---|38%|Deaktivierter Text|
|`scrim`|#000000 @ 32%|#000000 @ 32%|---|---|---|Overlay zur Abdunkelung (z.B. hinter Dialogen)|

_(Hinweis: Die Overlay-Prozentsätze für Dark Theme Elevation sind Richtwerte basierend auf M2/M3 und müssen ggf. angepasst werden. Die exakten HEX-Werte sind Platzhalter und müssen definiert werden.)_

### 4.3 Typografie-System

- **Typografie-Skala:** Eine definierte Skala legt Schriftgrößen, -gewichte und Zeilenhöhen für verschiedene semantische Rollen fest, inspiriert von Material Design 3 (z.B. Display, Headline, Title, Body, Label - jeweils Large/Medium/Small).2 Die Skala wird für optimale Lesbarkeit auf Desktop-Monitoren angepasst.
- **Schriftarten:** Primäre (und ggf. sekundäre) Schriftfamilien werden festgelegt. Die Auswahl berücksichtigt Lesbarkeit unter Linux/Wayland und Verfügbarkeit. Gewichte (z.B. Regular, Medium, Bold) werden den Skalen-Einträgen zugeordnet.
- **Implementierung:** Das `Theme`-Singleton stellt Font-Objekte bereit (z.B. `Theme.typography.headlineLarge.font`). Jedes Font-Objekt kapselt `family`, `pointSize`/`pixelSize`, `weight`, `letterSpacing` etc. QML `Text`-Elemente (oder `VxLabel`) binden ihre `font`-Property an diese Objekte.

**Tabelle: VivoX Typografie-Skala (Beispiel)**

|   |   |   |   |   |   |   |
|---|---|---|---|---|---|---|
|**Stil-Name**|**Font Family**|**Font Size (pt)**|**Font Weight**|**Line Height (faktor)**|**Letter Spacing**|**Verwendungskontext**|
|`headlineLarge`|"Noto Sans"|32|400|1.25|0|Große Überschriften (selten)|
|`headlineMedium`|"Noto Sans"|28|400|1.25|0|Standard-Seitenüberschriften|
|`titleLarge`|"Noto Sans"|22|500|1.25|0.1|Wichtige Titel unterhalb von Headlines|
|`titleMedium`|"Noto Sans"|16|500|1.5|0.15|Untertitel, Karten-Header, Dialog-Titel|
|`titleSmall`|"Noto Sans"|14|500|1.5|0.1|Kleine Titel, Listenelement-Überschriften|
|`bodyLarge`|"Noto Sans"|16|400|1.5|0.5|Langer Fließtext|
|`bodyMedium`|"Noto Sans"|14|400|1.4|0.25|Standard-Text, Listenelement-Sekundärtext|
|`bodySmall`|"Noto Sans"|12|400|1.3|0.4|Kleingedrucktes, Beschriftungen|
|`labelLarge`|"Noto Sans"|14|500|1.4|0.1|Button-Text, Tab-Beschriftungen|
|`labelMedium`|"Noto Sans"|12|500|1.3|0.5|Kleine Buttons, Chip-Text|
|`labelSmall`|"Noto Sans"|11|500|1.3|0.5|Überstriche (Overline), sehr kleine Beschriftungen|

_(Hinweis: Schriftfamilie, Größen und Gewichte sind Beispiele und müssen final festgelegt werden.)_

### 4.4 Abstandssystem (Spacing)

- **Basis-Einheit:** Eine Basis-Abstandseinheit wird definiert (z.B. 8dp, angepasst an Desktop-Skalierung).
- **Skala:** Alle Ränder (Margins), Innenabstände (Paddings) und Lücken zwischen Elementen sollten Vielfache dieser Basis-Einheit sein (z.B. 4dp, 8dp, 12dp, 16dp, 24dp, 32dp).2
- **Implementierung:** Das `Theme`-Singleton stellt benannte Abstandswerte bereit (z.B. `Theme.spacing.base` (8), `Theme.spacing.small` (4), `Theme.spacing.medium` (16), `Theme.spacing.large` (24)). QML-Layouts (`RowLayout`, `ColumnLayout`, `GridLayout`) und Anchors verwenden diese Werte für `spacing`, `margins`, `padding`, `anchors.margins` etc.

### 4.5 Formensystem (Shape)

- **Eckenradien:** Eine Skala für Eckenradien wird definiert, um die Form von Containern zu steuern (z.B. `none` (0), `extraSmall` (2), `small` (4), `medium` (8), `large` (16), `extraLarge` (28), `full` (basierend auf Höhe/Breite für Kreis/Pille)).2
- **Implementierung:** Das `Theme`-Singleton stellt Radiuswerte bereit (z.B. `Theme.shape.radiusMedium`). Komponenten wie `VxButton`, `VxCard`, `VxDialog`, `VxTextField` verwenden diese Werte für ihre `radius`-Property (oft auf einem internen `Rectangle`).

### 4.6 Ikonografie (Icons)

- **Icon-Set:** Die Quelle der Icons wird festgelegt (z.B. Material Symbols 3, Font Awesome oder ein benutzerdefiniertes SVG-Set). Das Set muss für Dark Themes geeignet sein (klare Linien, ausreichender Kontrast) und gut skalieren.
- **Größen:** Standardgrößen für Icons werden definiert (z.B. 16px, 20px, 24px).
- **Farben:** Die Standardfarbe für Icons ist typischerweise eine "On-"-Farbe (z.B. `Theme.color.onSurface` oder `Theme.color.onPrimary`), kann aber überschrieben werden.
- **Implementierung:** Das `Theme`-Singleton kann Standardgrößen bereitstellen (z.B. `Theme.icon.sizeMedium`). Das `VxIcon`-Atom ist für die Darstellung verantwortlich und nimmt `source`, `size` und `color` als Properties entgegen.

### 4.7 Bewegung & Animation (Motion)

- **Prinzipien:** Animationen sollen informativ, fokussiert und expressiv sein, um die Benutzerführung zu unterstützen und Feedback zu geben.2 Sie sollen natürlich wirken und nicht störend sein.
- **Easing & Dauer:** Standardisierte Easing-Kurven (z.B. `emphasized`, `standard`, `accelerate`, `decelerate` - adaptiert von M3 29) und Dauer-Token (z.B. `short1` (100ms), `medium2` (300ms), `long1` (500ms) - M3 Skala 29) werden definiert. Die Wahl hängt von der Art der Transition ab (z.B. Eintreten vs. Verlassen, Größe der Bewegung).30 Kürzere Dauern für Ausblendungen, längere für Einblendungen oder persistente Elemente.30
- **Standard-Übergänge:** Konsistente Übergänge für häufige Aktionen werden definiert (z.B. Fade-In/Out für Dialoge, Slide-Up/Down für Bottom Sheets (falls verwendet), dezenter Container-Transform für Listen-Detail-Ansichten 31).
- **Barrierefreiheit:** Animationen müssen die Systemeinstellungen für reduzierte Bewegung respektieren. Bei aktivierter Reduzierung sollten intensive Animationen durch subtile Fades ersetzt oder ganz deaktiviert werden.31
- **Implementierung:** Das `Theme`-Singleton stellt Easing- und Dauerwerte bereit (z.B. `Theme.motion.easing.emphasized`, `Theme.motion.duration.medium2`). QML `Behavior`, `Transition`, `PropertyAnimation`, `SequentialAnimation`, `ParallelAnimation` etc. nutzen diese Werte.6

Die konsequente Anwendung des Theming-Systems auf alle Komponenten ist entscheidend. Jegliche hartcodierten Werte für Farben, Schriftarten, Abstände etc. in Komponenten untergraben das System und müssen vermieden werden. Code-Reviews sind hierfür unerlässlich.

## 5. Layout-System Spezifikation

Das Layout-System definiert, wie Komponenten und Inhalte auf dem Bildschirm angeordnet werden, um eine konsistente und anpassungsfähige Struktur zu gewährleisten, die für Desktop-Umgebungen optimiert ist.

### 5.1 Desktop-Layout-Prinzipien

- **Responsivität/Adaptivität:** Layouts müssen sich fließend an Änderungen der Fenstergröße anpassen. Die Lesbarkeit und Bedienbarkeit von Inhalten muss über einen typischen Bereich von Desktop-Fenstergrößen erhalten bleiben. Material Designs Fenstergrößenklassen (Compact, Medium, Expanded, Large, Extra-Large) 33 dienen als konzeptionelle Grundlage, die spezifischen Breakpoints und Verhaltensweisen werden jedoch für den Desktop-Kontext optimiert. Der Fokus liegt weniger auf festen Breakpoints für Geräteklassen als auf flexibler Anpassung.
- **Kanonische Layouts (Adaptiert):** Inspiration wird aus M3's kanonischen Layouts (List-Detail, Supporting Pane, Feed) 35 gezogen, insbesondere für gängige Desktop-Muster wie Einstellungsdialoge (oft List-Detail oder Supporting Pane) oder Übersichtsseiten (oft Feed-ähnlich). Die konkrete Umsetzung erfolgt jedoch mit den Mitteln von Qt Quick.
- **Pane-Management:** Für Layouts mit mehreren Bereichen (Panes) werden Standardverhaltensweisen definiert. Dies umfasst die Unterscheidung zwischen fixierten und flexiblen Panes sowie Split-Panes.34 Das Verhalten bei Größenänderung (z.B. Ein-/Ausblenden von Panes, Umfließen von Inhalten bei geringer Breite) wird festgelegt.34 Es wird zwischen persistenter und temporärer Größenänderung durch den Benutzer unterschieden (z.B. über einen Drag Handle).34

### 5.2 QML Grid- und Layout-System

- **Konzeptionelles Raster:** Ein konzeptionelles Raster (z.B. basierend auf 8dp-Einheiten für Spalten und Abstände, ähnlich M3 28) dient als Designrichtlinie für Ausrichtung und konsistente Abstände. Es wird jedoch nicht als starre Komponente implementiert.
- **Qt Quick Layouts:** Die primären Werkzeuge zur Anordnung von Elementen sind die `QtQuick.Layouts` (`RowLayout`, `ColumnLayout`, `GridLayout`).23 Diese bieten eine effiziente und idiomatische Möglichkeit, flexible und adaptive Layouts in QML zu erstellen. Sie verwalten die Größenanpassung und Positionierung von Kindelementen basierend auf verfügbarem Platz und den Eigenschaften der Kinder (z.B. `Layout.fillWidth`, `Layout.preferredHeight`). Die Verwendung dieser eingebauten Layouts ist der Neuerstellung eines komplexen Grid-Systems vorzuziehen.
- **Anchors:** QML Anchors (`anchors.*`) werden für relative Positionierungen verwendet, wo Layouts ungeeignet sind (z.B. Überlagerungen, feste Positionierung relativ zum Parent) oder für einfachere Fälle. Komplexe, verschachtelte Anchor-Beziehungen sollten vermieden werden, da sie schwerer zu warten sind.23
- **Abstände:** Innerhalb von Layouts (`spacing`) und bei der Verwendung von Anchors (`anchors.margins`, `anchors.leftMargin` etc.) werden konsistent die Abstandswerte aus dem `Theme`-Singleton (`Theme.spacing.*`) verwendet.28

### 5.3 Standard-Container-Komponenten

Standardisierte Container helfen, Konsistenz in der Darstellung von UI-Bereichen zu gewährleisten.

- **`VxPanel` / `VxSurface`**: Ein einfacher Container, der die Oberflächenfarbe und Elevation aus dem Theme verwendet (`Theme.color.surface_dpX`). Dient oft als Basis für komplexere Organismen oder als Hintergrund für UI-Abschnitte.
- **`VxCard`**: Wie in Abschnitt 3.4 definiert. Ein stilierter Container mit Elevation für gruppierte Inhalte und Aktionen.
- **`VxDialog`**: Wie in Abschnitt 3.4 definiert. Standardcontainer für modale und nicht-modale Dialoge mit Titel, Inhalt und Aktionsbereich.
- **`VxScrollablePage`**: Ein Container, der vertikales (und ggf. horizontales) Scrollen ermöglicht, wenn der Inhalt den sichtbaren Bereich überschreitet. Kann thematisierte Scrollbalken integrieren. Basiert oft auf `ScrollView` oder `Flickable`.

### 5.4 Richtlinien für Kern-Anwendungslayout

- **Hauptfensterstruktur (`VivoXMainWindowView`):** Definition der typischen Anordnung der Kern-Organismen: `VxTitleBar` oben, `VxSidebar` links (oder rechts, konfigurierbar), und der zentrale Hauptinhaltsbereich, der dynamisch verschiedene Views lädt.
- **Platzierung von Kontrollleisten/Toolbars:** Standardpositionen für primäre Aktionsleisten (`VxToolbar`) festlegen, z.B. direkt unter der `VxTitleBar` oder am unteren Rand des Inhaltsbereichs, je nach Kontext.
- **Verhalten der Seitenleiste (`VxSidebar`):** Standardbreite, optionales Einklappen (z.B. zu einer Icon-Leiste bei geringer Fensterbreite) und Interaktion mit dem Hauptinhaltsbereich definieren (z.B. Überlagerung oder Verdrängung des Inhalts).
- **Speed Dial Layout:** Verweis auf das `SpeedDialTemplate` (Abschnitt 3.5) für die Anordnung der `VxSpeedDialWidget`-Organismen in einem Raster.

Entwickler benötigen klare Richtlinien und Beispiele, wann Qt Quick Layouts und wann Anchors zu verwenden sind und wie die Theme-Abstände in beiden Systemen konsistent angewendet werden. Die Anpassung von M3-Layoutprinzipien an den Desktop-Fokus von VivoX bedeutet, Flexibilität und fließende Größenänderung über starre Breakpoints zu stellen.

## 6. Interaktionsmuster & Feedback

Dieses Kapitel definiert Standard-Interaktionsmuster und Feedback-Mechanismen für VivoXLib, inspiriert von Material Design, aber angepasst an eine primäre Desktop-Nutzung mit Maus und Tastatur.

### 6.1 Desktop-Interaktionsprimitive

- **Maus:**
    - _Hover:_ Visuelles Feedback (Farbänderung, leichte Skalierung oder Elevation) bei Überfahren interaktiver Elemente, definiert im Theme.2
    - _Klick:_ Standardverhalten für Links- (Aktion auslösen) und Rechtsklick (Kontextmenü). Klick-Feedback durch Zustandsänderung (Pressed-Farbe) und ggf. adaptierte Ripple-Effekte.
    - _Drag-and-Drop:_ Visuelle Hinweise während des Ziehens (z.B. Schatten, Platzhalter) und klare Zielbereiche.
    - _Scrollrad:_ Standard-Scrollverhalten in scrollbaren Bereichen (`VxScrollablePage`).
    - _Trefferflächen:_ Ausreichend große Klickbereiche auch für Mausbedienung sicherstellen.
- **Tastatur:**
    - _Navigation:_ Logische und vorhersagbare Fokusreihenfolge mit Tab/Shift+Tab. Pfeiltasten für Navigation innerhalb von Listen, Menüs oder Grids.
    - _Aktivierung:_ Standardaktivierung von Buttons und Listenelementen mit Enter/Leertaste.
    - _Shortcuts:_ Definition gängiger Shortcuts (z.B. Strg+C, Strg+V, Esc zum Schließen von Dialogen).
    - _Fokus-Indikation:_ Deutliche visuelle Hervorhebung des fokussierten Elements mittels Theme-Farben (z.B. `Theme.color.primary` oder `Theme.color.outline` mit erhöhter Dicke).
- **Touch/Gesten (Sekundär):** Grundlegende Touch-Interaktionen (Tap entspricht Klick, Swipe in scrollbaren Bereichen) sollten funktionieren, sind aber nicht primärer Fokus. M3 bietet detaillierte Gestenrichtlinien 2, die bei Bedarf konsultiert werden können.

### 6.2 Material-inspiriertes Interaktionsfeedback

- **Zustände:** Komponenten müssen ihre Zustände (Hover, Fokus, Pressed, Selected, Disabled) klar visuell kommunizieren. Dies geschieht primär durch die im Theme definierten Farben und Opazitätswerte.2 Subtile Skalierungs- oder Elevationsänderungen können unterstützend eingesetzt werden.
- **Ripple-Effekte (Adaptiert):** Als direktes Feedback auf Klick/Tap wird ein dezenter, zentrierter oder vom Klickpunkt ausgehender Ripple-Effekt auf aktivierbaren Oberflächen (`VxButton`, `VxListItem`) implementiert. Die Ausbreitung und Intensität wird für den Desktop reduziert (weniger dominant als oft auf Mobilgeräten) und die Animation muss performant sein. Der Effekt muss die Einstellung für reduzierte Bewegung respektieren.31

### 6.3 Visuelle und Bewegungs-Feedback-Richtlinien

- **Übergänge (Transitions):** Zustandsänderungen (z.B. Auswahl, Auf-/Zuklappen von Bereichen, Ein-/Ausblenden von Elementen) werden durch die im Theme definierten Standardanimationen (Easing, Duration - siehe Abschnitt 4.7) begleitet, um dem Nutzer Orientierung und ein Gefühl von Fluss zu geben.29 Harte Schnitte ("Jump Cuts") sollten vermieden werden, außer bei sehr einfachen, hochfrequenten Aktionen wie dem Öffnen eines Menüs.31
- **Mikrointeraktionen:** Subtile Animationen (z.B. Icon-Transformationen beim Umschalten, fließende Fortschrittsanzeigen) werden encouraged, um Aktionen zu bestätigen und die Oberfläche lebendiger wirken zu lassen.26 Sie dürfen jedoch nicht ablenken oder die wahrgenommene Performance beeinträchtigen.
- **Ladezustände:** Während Daten geladen werden oder längere Operationen stattfinden, wird dem Nutzer Feedback gegeben. Dies kann durch `VxProgressIndicator` (Atom) oder durch Platzhalter-UI (Skeleton Loader), die die Struktur des zu ladenden Inhalts andeuten, geschehen.31
- **Fehlerzustände:** Fehler werden klar kommuniziert, z.B. durch rote Hervorhebung (Farbe `Theme.color.error`) und erklärenden Text in betroffenen Komponenten wie `VxTextField` oder durch dedizierte Fehler-Dialoge/Banner.2

Die direkte Übernahme von touch-zentrierten M3-Feedbackmustern wie sehr prominenten Ripples kann sich auf dem Desktop unpassend anfühlen. Eine sorgfältige Adaption an Maus- und Tastaturinteraktion ist notwendig. Hover- und Fokus-Zustände sind hierbei besonders wichtig. Die Implementierung von Animationen in QML muss performant sein, insbesondere auf Wayland, wo die Grafikleistung variieren kann.36 Einfache, aber aussagekräftige Animationen sind komplexen, potenziell ruckelnden Effekten vorzuziehen. VivoXLib muss Komponenten bereitstellen, die diese Feedback-Mechanismen integriert haben, um Entwicklern die Arbeit zu erleichtern.

## 7. Implementierungsstrategie

Diese Sektion beschreibt die technische Umsetzung von VivoXLib, einschließlich der Implementierung von QML-Komponenten, des Theming-Systems und der Integration mit dem C++ Backend.

### 7.1 QML-Komponenten-Implementierungsrichtlinien

- **Wiederverwendbarkeit & Kapselung:** Jedes Atom, Molekül und Organismus wird als eigenständige QML-Datei implementiert (`.qml`). Komponenten müssen in sich geschlossen sein und ihre Funktionalität über eine klar definierte Schnittstelle (Properties, Signale, Methoden) nach außen anbieten. Interne Implementierungsdetails sollten verborgen bleiben. QML Best Practices sind zu befolgen.23
- **API-Design:** Die QML-API jeder Komponente muss klar, konsistent und gut dokumentiert sein.
    - `property` für konfigurierbare Attribute.
    - `readonly property` für interne Konstanten oder abgeleitete Werte.
    - `property alias` um Eigenschaften von Kind-Elementen direkt zugänglich zu machen, wo sinnvoll.
    - `signal` für Ereignisse, auf die von außen reagiert werden soll.42
    - Methoden (JavaScript-Funktionen innerhalb der Komponente) für interne Logik oder einfache Aktionen.
- **Performance:** Performance ist ein kritischer Aspekt.
    - Minimierung komplexer Bindungen und JavaScript-Logik, insbesondere in Delegaten von Listen/Views.
    - Vermeidung unnötig tiefer Verschachtelung von Elementen.
    - Einsatz von `Loader` zur verzögerten Instanziierung von UI-Teilen, die nicht sofort sichtbar sind.
    - Optimierung der Grafik-Renderer-Nutzung durch QML.6 Qt 6 bietet generelle Performance-Verbesserungen gegenüber Qt 5.5
    - Vermeidung imperativer Zuweisungen in `Component.onCompleted`, wo deklarative Bindungen möglich sind, da dies zu mehrfachen Evaluierungen führen kann.23
- **Struktur & Registrierung:** Komponenten werden gemäß der Atomic Design-Hierarchie in Verzeichnissen organisiert. Sie werden über die `qmldir`-Datei des `VivoXLib`-Moduls oder via CMake (`qt_add_qml_module`) registriert, um per `import VivoXLib 1.0` verfügbar zu sein.13 Richtlinien zur Erstellung von Custom Components sind zu beachten.13

### 7.2 Theming-Implementierung in QML

- **Singleton `Theme.qml`:** Das Theming wird über ein zentrales Singleton-Objekt `Theme.qml` realisiert.17
    - _Struktur:_ Das Singleton enthält verschachtelte `QtObject`s für eine klare Organisation: `Theme.color`, `Theme.typography`, `Theme.spacing`, `Theme.shape`, `Theme.icon`, `Theme.motion`.
    - _Beispiel Zugriff:_ `VxButton { background.color: Theme.color.primary; label.font: Theme.typography.button.font }`.
    - _Registrierung:_ Erfolgt wie in 4.1 beschrieben über CMake oder `qmldir`.
- **Anwendung:** Alle VivoXLib-Komponenten (Atome, Moleküle, Organismen) MÜSSEN ihre visuellen Eigenschaften (Farben, Schriftarten, Abstände, Radien, Animationsparameter) ausschließlich aus dem `Theme`-Singleton beziehen. Hartcodierte Werte sind nicht zulässig.
- **Anpassung:** Die einfachste Form der Anpassung besteht darin, die Werte innerhalb von `Theme.qml` zu ändern. Für komplexere Szenarien könnten alternative Singleton-Implementierungen bereitgestellt werden, die zur Laufzeit ausgewählt werden (obwohl dies die Vorteile der Compile-Zeit-Optimierung von Bindungen beeinträchtigen kann). Die Mechanismen von Qt Quick Controls zur Stil-Anpassung 45 (z.B. `qtquickcontrols2.conf`, `-style` Argument) werden für VivoXLib _nicht_ primär genutzt, um die Kontrolle über das einheitliche Theme zu behalten; der Singleton-Ansatz wird bevorzugt.

### 7.3 C++ Backend-Integrationsstrategie

Die Interaktion zwischen der QML-basierten UI (VivoXLib) und dem C++ Backend von VivoX erfolgt über etablierte Qt-Mechanismen.49 Die Trennung von UI (QML) und Logik/Daten (C++) ist dabei ein zentrales Prinzip.23

- **Datenbindung (Data Binding):**
    - _Modelle bereitstellen:_ C++ Datenmodelle werden QML zugänglich gemacht.
        - Für Listen/Tabellen: Subklassen von `QAbstractItemModel` (z.B. `QAbstractListModel`) verwenden. Rollen müssen über `roleNames()` für QML definiert werden.18
        - Für einfache Objekte/Werte: `QObject`-basierte Klassen mit `Q_PROPERTY` verwenden.49
    - _Registrierung/Exposition:_ C++ Typen werden mit `qmlRegisterType` im QML-System registriert oder Instanzen werden über `QQmlContext::setContextProperty` in den QML-Kontext injiziert.49
    - _Properties Binden:_ `Q_PROPERTY` in C++ benötigt `READ`, `WRITE` (optional) und `NOTIFY` (essentiell für Updates in QML). QML-Properties können direkt an diese C++ Properties gebunden werden (`text: cppBackend.someProperty`). Änderungen in C++ lösen das `NOTIFY`-Signal aus, was die QML-Bindung aktualisiert.49
    - _Änderungen aus QML:_ Wenn QML eine gebundene C++ Property ändert (z.B. `TextField { text: cppBackend.value; onTextChanged: cppBackend.value = text }`), muss die C++ `WRITE`-Methode (Setter) das `NOTIFY`-Signal nur emittieren, wenn sich der Wert tatsächlich geändert hat, um Bindungsschleifen zu vermeiden.52 Für Modelle müssen `setData()` implementiert und `dataChanged()` emittiert werden.18 Strukturelle Änderungen erfordern `begin/endInsertRows` bzw. `begin/endRemoveRows`.18
- **Aktionsauslösung (Action Invocation):**
    - _QML ruft C++ auf:_ C++ Funktionen/Methoden werden für QML über das `Q_INVOKABLE` Makro oder durch Deklaration als `public slot` verfügbar gemacht.50 QML kann diese direkt aufrufen, z.B. in Signalhandlern: `VxButton { onClicked: cppBackend.performAction(someArgument) }`. Die Wahl zwischen `Q_INVOKABLE` und `slot` ist oft stilistisch; `Q_INVOKABLE` kann semantisch klarer sein für Methoden, die primär für QML-Aufrufe gedacht sind.
    - _C++ benachrichtigt QML:_ C++ Objekte können Signale (`Q_SIGNAL`) aussenden, auf die in QML reagiert werden kann. Dies geschieht entweder durch `on<SignalName>`-Handler in QML oder durch explizite Verbindungen mittels des `Connections`-Elements.42
- **Integration mit VivoX-Frameworks:**
    - _Action Framework:_ QML-Benutzeraktionen (z.B. `VxButton.onClicked`) rufen die entsprechenden `Q_INVOKABLE`-Methoden oder Slots im C++ Backend auf, die dann Aktionen über das Action Framework auslösen.
    - _Widget System:_ VivoX Widgets, die als Organismen in VivoXLib implementiert werden (z.B. `VxSpeedDialWidget`), binden ihre darzustellenden Eigenschaften an Daten aus C++ Modellen, die vom Widget System bereitgestellt werden. Interaktionen innerhalb der Widgets rufen wiederum C++ Methoden auf.
    - _ConfigManager:_ Relevante Konfigurationseinstellungen aus dem `ConfigManager` werden über `Q_PROPERTY` (mit `READ`, `WRITE`, `NOTIFY`) eines C++ Singletons oder Kontextobjekts an QML exponiert. UI-Elemente in den Einstellungen (z.B. `VxSwitch`, `VxTextField` in `VxSettingsSection`) können direkt an diese Properties binden, um Werte anzuzeigen und zu ändern.
- **QWebChannel (für WebEngine-Interaktion):**
    - _Anwendungsfall:_ Falls eine tiefe, bidirektionale Kommunikation zwischen QML/C++ und dem Inhalt einer `WebEngineView` (z.B. für spezielle web-basierte Widgets im Speed Dial) erforderlich ist, wird `QWebChannel` eingesetzt.59
    - _Funktionsweise:_ C++ `QObject`-Instanzen werden auf einem `QWebChannel`-Objekt registriert (`channel.registerObject("backendId", myCppObject)`). JavaScript im Web-Inhalt kann über die `qwebchannel.js`-Bibliothek auf diese Objekte zugreifen (`channel.objects.backendId`).61 Dies ermöglicht JS, C++ Methoden/Slots aufzurufen (`backendId.doSomething()`) und auf C++ Signale zu reagieren (`backendId.someSignal.connect(...)`).62
    - _Transport:_ Die Kommunikation benötigt einen Transportmechanismus. Innerhalb von `WebEngineView` kann dies direkt erfolgen, oder es wird ein `WebSocket`-basierter Transport aufgesetzt (z.B. mit `QWebSocketServer` in C++).59
    - _Komplexität:_ Die Integration von `QtWebEngine` 66 und `QWebChannel` fügt signifikante Komplexität und Abhängigkeiten (Chromium 69) hinzu. Performance 68 und Wayland-Kompatibilität 36 sind sorgfältig zu prüfen. Die Verwendung sollte auf notwendige Fälle beschränkt werden. Die Unterstützung für Progressive Web Apps (PWA) in QtWebEngine scheint begrenzt oder nicht vorhanden zu sein.74

Die klare Trennung der Verantwortlichkeiten zwischen C++ und QML ist für die Wartbarkeit entscheidend. Direkte Manipulation von QML-Objekten aus C++ sollte vermieden werden; der Datenfluss sollte primär über Modelle und Properties, der Aktionsfluss über Signale, Slots und Invokables erfolgen.50

## 8. Schlussfolgerung und Empfehlungen

Dieser Bauplan legt eine solide Grundlage für die Entwicklung der VivoX UI-Bibliothek (VivoXLib). Durch die Kombination der bewährten Prinzipien von Material Design 3, angepasst an die spezifischen Anforderungen einer Dark-Theme-fokussierten Desktop-Umgebung unter Linux/Wayland, und der strukturierten Herangehensweise des Atomic Design in QML, zielt VivoXLib darauf ab, eine konsistente, moderne, performante und wartbare Benutzeroberfläche für das VivoX-Projekt zu ermöglichen.

**Zusammenfassung der Kernpunkte:**

- **Design:** Material 3-inspiriert, aber für Dark Theme und Desktop (Maus/Tastatur) adaptiert.
- **Architektur:** Strikte Anwendung von Atomic Design (Atome, Moleküle, Organismen, Templates, Views) implementiert als wiederverwendbare QML-Komponenten in einem dedizierten Modul (`VivoXLib`).
- **Theming:** Zentralisiert über ein QML-Singleton (`Theme.qml`), das Farben (Dark Theme optimiert, mit Elevation-Overlays), Typografie, Abstände, Formen, Icons und Animationsparameter definiert.
- **Layout:** Nutzung von Qt Quick Layouts und Anchors für responsive Desktop-Layouts, inspiriert von M3-Konzepten, aber mit Fokus auf fließende Größenänderung.
- **Interaktion:** Definition von Desktop-Interaktionsmustern (Maus, Tastatur) mit adaptiertem Material-Feedback (Zustände, dezente Ripples) und klaren visuellen/Bewegungs-Rückmeldungen.
- **Implementierung:** Klare Richtlinien für QML-Komponentenentwicklung, Anwendung des Singleton-Theming und Integration mit dem C++ Backend über Qt's Datenbindungs- und Signal/Slot/Invokable-Mechanismen.

**Wichtige Empfehlungen:**

1. **Strikte Einhaltung:** Die konsequente Anwendung der Atomic Design-Struktur und des zentralen Theming-Systems ist entscheidend für den Erfolg. Abweichungen sollten gut begründet und dokumentiert sein.
2. **Wayland-Tests:** Aufgrund potenzieller Performance-Variabilität und spezifischer Probleme von Qt/QtWebEngine unter Wayland 36 sind gründliche Tests der Komponenten und insbesondere der Animationen auf den Zielplattformen unerlässlich.
3. **C++/QML-Trennung:** Die klare Trennung zwischen QML (Präsentation) und C++ (Logik, Daten) muss während der gesamten Entwicklung aufrechterhalten werden, um die Wartbarkeit zu gewährleisten.23
4. **Iterative Entwicklung:** Beginnen Sie mit der Implementierung der grundlegenden Atome und des Theming-Singletons. Bauen Sie darauf aufbauend Moleküle und dann Organismen.
5. **Code-Reviews:** Etablieren Sie robuste Code-Review-Prozesse, um die Einhaltung der Richtlinien dieses Bauplans sicherzustellen, insbesondere bezüglich Theming und Komponentendesign.
6. **QtWebEngine mit Bedacht:** Minimieren Sie die Abhängigkeit von `QtWebEngine` in Kernkomponenten der Bibliothek aufgrund der zusätzlichen Komplexität, Abhängigkeiten und potenziellen Performance-/Kompatibilitätsprobleme.68 Kapseln Sie die Verwendung, wo sie unvermeidbar ist.

**Nächste Schritte:**

- Detaillierte Ausarbeitung des Designs einzelner Schlüsselkomponenten (Atome, Moleküle).
- Prototypische Implementierung des `Theme.qml` Singletons mit den finalen Farb- und Typografiewerten.
- Aufsetzen der Projektstruktur und des `VivoXLib` QML-Moduls mittels CMake.
- Beginn der Implementierung der Basis-Atome gemäß den Spezifikationen.