# Abläufe und Anwendungsfälle

**Grundphilosophie der Benutzererfahrung:** Unsere Desktop-Umgebung soll sich anfühlen wie eine natürliche Erweiterung der Denk- und Arbeitsweise des Nutzers. Jede Interaktion ist darauf ausgelegt, Reibung zu minimieren, Kontextwechsel intelligent zu gestalten und proaktive Unterstützung anzubieten, ohne bevormundend zu wirken. Die dunkle, elegante Ästhetik mit gezielten Akzentfarben schafft eine fokussierte und angenehme Arbeitsatmosphäre. Flüssige Animationen sind nicht nur schmückendes Beiwerk, sondern visuelles Feedback, das Aktionen verständlich macht und die wahrgenommene Geschwindigkeit erhöht.

**Typische Nutzerabläufe und das Zusammenspiel der Komponenten:**

**Szenario 1: Der Entwickler startet in seinen Tag**

1. **Systemstart und Login:** Der Entwickler startet sein System. Der Login-Bildschirm präsentiert sich im gleichen dunklen, eleganten Stil wie der Desktop selbst. Nach erfolgreichem Login (unterstützt durch die Systemsicherheitsmechanismen) wird der Desktop blitzschnell geladen.
2. **Der erste Blick – Der "Space" für Entwicklung:**
    - Der Entwickler landet standardmäßig in seinem "Development-Space", den er gestern so verlassen hat. Dieser "Space" hat ein von ihm gewähltes Icon (z.B. ein Zahnrad) und eine dezente blaue Akzentfarbe, die ihm hilft, ihn sofort von seinem "Kommunikations-Space" (mit einer Sprechblase und grüner Akzentfarbe) zu unterscheiden.
    - **Intelligente Tab-Leiste:** Oben im "Development-Space" zeigt die Tab-Leiste prominent seine "angepinnte" Hauptanwendung: seinen Code-Editor (z.B. VS Code). Daneben, ebenfalls als "gepinnt" definiert, ein Terminal in einer Split-View. Diese Tabs sind klar beschriftet und der aktive Tab (Code-Editor) ist mit der primären Systemakzentfarbe (Korallrot) hervorgehoben.
    - **Linke Seitenleiste (Workspace-Switcher):** Im eingeklappten Zustand sieht er die Icons seiner Spaces: das Zahnrad für "Development", die Sprechblase für "Kommunikation" und ein Icon für seinen "Design-Space". Das Zahnrad-Icon ist hervorgehoben, da dies der aktive Space ist.
3. **Projekt öffnen und Arbeitsumgebung einrichten:**
    - **Kontextuelle Befehlspalette (`Super+Space`):** Der Entwickler drückt `Super+Space`. Die Befehlspalette erscheint als dunkles Overlay. Er tippt "projekt anaconda". Dank Fuzzy-Matching und Kenntnis seines aktuellen "Development-Space" (Kontext!) schlägt die Palette "Projekt 'Anaconda' in VS Code öffnen" und "Terminal in Projekt 'Anaconda' öffnen" vor. Er wählt die erste Option.
    - VS Code (als XWayland-Anwendung oder native Wayland-Anwendung, gemanagt durch `xdg-shell`) startet und wird automatisch dem "Development-Space" zugeordnet, da er von dort aus initiiert wurde. Die **Intelligente Tab-Leiste** könnte jetzt, falls VS Code Fokus hat, kontextrelevante Aktionen oder Informationen aus VS Code anzeigen (zukünftige Erweiterung, erfordert Plugin-System oder standardisierte App-Integration).
    - **Automatisches Tiling:** Der Entwickler öffnet ein zweites Terminal. Da er für diesen "Development-Space" ein Spalten-Tiling-Layout konfiguriert hat, ordnet sich das neue Terminalfenster automatisch rechts neben dem bestehenden Split-View (Code-Editor + erstes Terminal) an. Die konfigurierbaren Gaps zwischen den Fenstern sorgen für eine klare visuelle Trennung.
4. **Recherche und Dokumentation:**
    - **Rechte Seitenleiste (Widgets):** Er hat ein Widget für Notizen und eines für Web-Lesezeichen in der rechten Seitenleiste. Er zieht per Drag & Drop einen Link aus seinem Browser in das Lesezeichen-Widget. Das Widget zeigt eine Vorschau und speichert den Link.
    - **KI-Unterstützung (Einwilligung vorausgesetzt):** Er markiert einen komplexen Code-Abschnitt in VS Code, öffnet die Befehlspalette und tippt "code erklären". Die KI-Integration (via MCP-Client) sendet den Code-Schnipsel (nach expliziter Zustimmung, die er zuvor für "Code-Analyse durch lokales KI-Modell" erteilt hat) an ein lokales Sprachmodell. Die Erklärung erscheint in einem unaufdringlichen Pop-up oder direkt in der Seitenleiste in einem KI-Widget.
5. **Schneller Wechsel zum Kommunikations-Space:**
    - Eine **Benachrichtigung** erscheint dezent (gestaltet im Dark Mode Stil, mit korallrotem Akzent für die Dringlichkeit "wichtig"): "Neue Nachricht von Team-Chat". Ein Klick auf die Benachrichtigung oder eine definierte Geste (z.B. Drei-Finger-Wisch nach links auf dem Touchpad) wechselt zum "Kommunikations-Space".
    - **Linke Seitenleiste (Workspace-Switcher):** Das Sprechblasen-Icon wird hervorgehoben.
    - **Intelligente Tab-Leiste:** Zeigt nun den "angepinnten" Team-Chat-Client und daneben seinen E-Mail-Client.
6. **Meeting und Bildschirmfreigabe:**
    - Er startet einen Videoanruf. Die Anfrage zur Bildschirmfreigabe wird über ein **XDG Desktop Portal** sicher gehandhabt. Er wählt aus, nur das Anwendungsfenster des Team-Chats freizugeben.
    - **Quick-Settings-Panel:** Während des Anrufs passt er schnell die Mikrofonlautstärke über das Quick-Settings-Panel in der Systemleiste an.

**Szenario 2: Die Content Creatorin gestaltet und publiziert**

1. **Kreativer Arbeitsplatz – Der "Design-Space":**
    - Die Creatorin wechselt in ihren "Design-Space". Dieser hat ein Pinsel-Icon und eine violette Akzentfarbe. Ihr Grafikprogramm und ein Ordner mit Projektdateien sind hier als Tabs "angepinnt". Der Space hat ein individuelles, inspirierendes Hintergrundbild.
2. **Arbeiten mit mehreren Monitoren und Farbkonsistenz:**
    - Sie arbeitet mit zwei Monitoren. Die Desktop-Umgebung erkennt die unterschiedlichen Auflösungen und wendet die korrekten Skalierungsfaktoren an. Das **Token-basierte Theming** sorgt dafür, dass die Farben auf beiden Bildschirmen konsistent dargestellt werden (unter der Annahme korrekter Farbprofile auf Systemebene).
    - Das Grafikprogramm läuft auf dem Hauptmonitor, während sie auf dem zweiten Monitor Referenzbilder und Notizen geöffnet hat, die sich dank der flexiblen Fensterverwaltung (Floating-Fenster im selben Space) frei anordnen lassen.
3. **Widgets als Inspirationsquelle:**
    - In ihrer rechten Seitenleiste hat sie ein Widget, das Bilder von einer von ihr abonnierten Design-Plattform anzeigt. Ein anderes Widget zeigt aktuelle Farbpaletten-Trends (könnte KI-gestützt sein).
4. **Übersichtsmodus für Projektmanagement:**
    - Sie aktiviert den **Übersichtsmodus**, um alle geöffneten Dateien, Programmfenster und Notizen ihres aktuellen Projekts auf einen Blick zu sehen. Sie gruppiert per Drag & Drop einige Referenzbilder zu einer temporären visuellen Gruppe direkt im Übersichtsmodus.
5. **Veröffentlichung und Sprachbefehle:**
    - Nach Fertigstellung eines Designs sagt sie: "Neuer Entwurf 'Frühlingskampagne' in Cloud-Ordner 'Abgeschlossen' verschieben und Benachrichtigung an Projektleiter senden."
    - Die KI (nach Zustimmung für Dateizugriff und E-Mail-Versand) führt die Dateioperation aus und öffnet einen E-Mail-Entwurf mit einer vordefinierten Nachricht.
    - Der Fortschritt der Datei-Synchronisation wird dezent in einem Widget oder der Systemleiste angezeigt.

**Szenario 3: Der Alltagsnutzer surft, organisiert und personalisiert**

1. **Intuitive Ersteinrichtung und Anpassung:**
    - Der Nutzer startet die Umgebung zum ersten Mal. Ein kurzer, interaktiver Willkommens-Guide erklärt die wichtigsten Konzepte wie "Spaces", die Befehlspalette und die Seitenleisten.
    - **Control Center:** Er öffnet das Control Center. Die Module sind klar benannt. Er möchte das Erscheinungsbild anpassen. Unter "Erscheinungsbild" wählt er den "Dark Mode" (der bereits Standard ist) und als Akzentfarbe ein leuchtendes Grün. Die Änderungen werden sofort live in der Vorschau des Control Centers und auf dem gesamten Desktop sichtbar.
    - **Schnellaktionsdock:** Er zieht seine Lieblingsanwendungen (Browser, E-Mail, Musik-Player) per Drag & Drop in das Schnellaktionsdock am unteren Bildschirmrand.
2. **Web-Browse und Organisation in "Spaces":**
    - Er erstellt einen neuen "Space" namens "Urlaubsplanung" und weist ihm ein Palmen-Icon zu.
    - In diesem Space "pinnt" er seinen Webbrowser an die **Intelligente Tab-Leiste**. Er öffnet mehrere Tabs für Hotels, Flüge und Aktivitäten. Die Tab-Leiste zeigt den aktiven Webseiten-Tab deutlich an.
3. **Interaktive Widgets und schnelle Informationen:**
    - Er fügt der rechten Seitenleiste ein Wetter-Widget und ein Kalender-Widget hinzu. Das Wetter-Widget zeigt die aktuelle Temperatur und eine Vorhersage. Ein Klick auf einen Tag im Kalender-Widget könnte eine detailliertere Tagesansicht oder eine Verknüpfung zu seiner Kalenderanwendung öffnen.
4. **Multitasking und der Übersichtsmodus:**
    - Er hat mehrere Fenster geöffnet. Mit einer Geste auf dem Touchpad (z.B. Vier-Finger-Wisch nach oben) aktiviert er den **Übersichtsmodus**. Alle Fenster seines "Urlaubsplanungs-Space" werden übersichtlich als Kacheln angezeigt. Er klickt auf das Fenster mit den Hotelbuchungen, um es in den Vordergrund zu holen.
5. **Befehlspalette für schnelle Aktionen:**
    - Er möchte die Bildschirmhelligkeit reduzieren. Statt ins Control Center zu gehen, drückt er `Super+Space` und tippt "helligkeit". Die Befehlspalette schlägt "Bildschirmhelligkeit anpassen" vor. Mit den Pfeiltasten kann er direkt in der Palette einen Schieberegler bedienen oder einen Prozentwert eingeben.
6. **Benachrichtigungen im Blick:**
    - Eine Benachrichtigung über ein abgeschlossenes Software-Update erscheint. Er kann sie direkt mit einem "OK"-Button in der Benachrichtigung schließen oder später im **Benachrichtigungszentrum** in der Systemleiste nachlesen.

**Zusammenspiel und Intuition – Das "Wie" und "Warum":**

- **Kontextsensitivität:** Die "Intelligente Tab-Leiste" und die "Kontextuelle Befehlspalette" sind Schlüsselelemente, die dem Nutzer basierend auf dem aktuellen "Space" oder der aktiven Anwendung relevante Werkzeuge und Aktionen anbieten. Dies reduziert die kognitive Last und macht häufige Aktionen schneller zugänglich.
- **Visuelle Hierarchie und Konsistenz:** Die dunkle Grundästhetik lenkt den Fokus auf den Inhalt. Akzentfarben werden gezielt eingesetzt, um aktive Zustände, wichtige Informationen oder Benutzerinteraktionspunkte hervorzuheben. Die Formensprache (scharfe Kanten für Hauptbereiche, abgerundete Ecken für interaktive Elemente) ist durchgängig und schafft ein harmonisches Gesamtbild.
- **Direkte Manipulation:** Features wie Drag & Drop (für Widgets, Fenster zwischen Spaces im Übersichtsmodus, Dateien ins Dock) und interaktive Widgets vermitteln ein Gefühl direkter Kontrolle und machen die Bedienung greifbar.
- **Nahtlose Übergänge:** Flüssige Animationen beim Öffnen des Übersichtsmodus, beim Wechseln von "Spaces" oder beim Ein-/Ausklappen von Seitenleisten sind nicht nur optisch ansprechend, sondern helfen dem Nutzer auch, die räumlichen Beziehungen und Zustandsänderungen im System besser zu verstehen.
- **Anpassung als Kernprinzip:** Von der Wahl der Akzentfarbe über die Konfiguration der "Spaces" (Icons, gepinnte Apps, Hintergründe) bis hin zur Bestückung der Seitenleisten und des Docks – der Nutzer kann die Umgebung an seine individuellen Bedürfnisse und Vorlieben anpassen. Das Token-basierte Theming ist die technische Grundlage dafür.
- **KI als unaufdringlicher Helfer:** Die KI drängt sich nicht in den Vordergrund, sondern bietet Unterstützung an, wo sie sinnvoll ist (z.B. Vorschläge im Speed-Dial, Code-Erklärung, Sprachbefehle). Das explizite Einwilligungsmanagement stellt sicher, dass der Nutzer immer die Kontrolle behält.
