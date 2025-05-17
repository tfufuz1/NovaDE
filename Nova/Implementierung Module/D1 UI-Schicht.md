# **NovaDE UI-Schicht: Implementierungsleitfaden – Teil 1: ui::shell::PanelWidget und AppMenuButton**

## **1\. Einleitung**

### **1.1. Zweck des Dokuments**

Dieses Dokument dient als detaillierter Implementierungsleitfaden für ausgewählte Module der UI-Schicht der Nova Desktop Environment (NovaDE). Es spezifiziert die Architektur, das Design, die Datenstrukturen, Schnittstellen und Implementierungsdetails auf einer ultrafeinen Ebene, sodass Entwicklerteams diese Spezifikationen direkt für die Codierung verwenden können, ohne grundlegende Designentscheidungen treffen oder Kernlogiken selbst entwerfen zu müssen. Dieses erste Teildokument fokussiert sich auf die Kernkomponente ui::shell::PanelWidget und dessen Submodul ui::shell::panel\_widget::AppMenuButton.

### **1.2. Zielgruppe**

Dieses Dokument richtet sich an Softwareentwickler und \-architekten, die an der Implementierung der NovaDE UI-Schicht beteiligt sind. Es wird ein Verständnis von Rust, GTK4 und den gtk4-rs Bindings sowie grundlegenden Konzepten der Softwarearchitektur und des UI-Designs vorausgesetzt.

### **1.3. Umfang (Teil 1: ui::shell::PanelWidget und AppMenuButton)**

Dieser erste Teil des Implementierungsleitfadens für die UI-Schicht behandelt die folgenden Module:

* **ui::shell::PanelWidget**: Die Haupt-Panel-Komponente der Desktop-Shell, verantwortlich für die Aufnahme und Anordnung verschiedener Panel-Module.  
* **ui::shell::panel\_widget::AppMenuButton**: Ein spezifisches Panel-Modul innerhalb des PanelWidget, das das globale Anwendungsmenü der aktiven Applikation anzeigt.

Nachfolgende Teildokumente werden weitere Module der UI-Schicht detaillieren.

### **1.4. Technologie-Stack (Verbindlich)**

Die Implementierung der UI-Schicht erfolgt unter strikter Verwendung des folgenden Technologie-Stacks:

* **GUI-Toolkit**: GTK4 1  
* **Rust-Bindings**: gtk4-rs 1  
* **Programmiersprache**: Rust 4  
* **Asynchrone Operationen**: Integration mit Rusts async/await über glib::MainContext::spawn\_local 7  
* **Theming**: Anwendung von CSS-Stilen über gtk::CssProvider, generiert durch domain::theming 9  
* **D-Bus-Kommunikation**: zbus Crate für Interaktionen mit Systemdiensten und anderen Anwendungen 12

### **1.5. Allgemeine UI/UX-Prinzipien (Wiederholung)**

Die Entwicklung der UI-Schicht orientiert sich an den folgenden übergeordneten UI/UX-Prinzipien, die eine visionstreue Umsetzung gewährleisten:

* **Konsistenz**: Einheitliches Erscheinungsbild und Verhalten über alle UI-Komponenten hinweg.  
* **Feedback**: Klares visuelles (und ggf. haptisches) Feedback auf Benutzeraktionen.  
* **Effizienz**: Minimierung der notwendigen Schritte zur Erledigung häufiger Aufgaben.  
* **Zugänglichkeit (Accessibility)**: Einhaltung der a11y-Standards (ATK/AT-SPI).23  
* **Performance**: Flüssige Animationen, schnelle Reaktionszeiten und geringer Ressourcenverbrauch.24  
* **Anpassbarkeit**: Ermöglichung benutzerdefinierter Konfigurationen von Layouts, Widgets und Verhalten.

## **2\. Modul: ui::shell::PanelWidget (Haupt-Panel-Implementierung)**

### **2.1.1. Übersicht und Verantwortlichkeiten**

Das PanelWidget ist die zentrale Komponente der ui::shell, die als primäre(s) Kontroll- und Systemleiste(n) der NovaDE dient. Es ist verantwortlich für:

* Die Bereitstellung einer oder mehrerer horizontaler Leisten am Bildschirmrand (oben oder unten, konfigurierbar).  
* Die Aufnahme, Anordnung und Verwaltung verschiedener, modularer Panel-Elemente (Submodule wie AppMenuButton, ClockDateTimeWidget, etc.).  
* Die Implementierung grundlegender Panel-Eigenschaften wie Höhe, Transparenz und eines visuellen "Leuchtakzent"-Effekts.  
* Die Interaktion mit dem gtk4-layer-shell-Protokoll, um sich korrekt in Wayland-Compositors zu integrieren, die dieses Protokoll unterstützen (z.B. wlroots-basierte wie Sway, Mir, KDE Plasma).26  
* Das dynamische Laden und Anwenden von Theming-Informationen, insbesondere für den "Leuchtakzent" und Hintergrundstile.

### **2.1.2. Visuelles Design und Theming**

* **Positionierung**: Konfigurierbar am oberen oder unteren Bildschirmrand.  
* **Höhe**: Konfigurierbare Höhe, z.B. zwischen 24px und 128px.  
* **Transparenz**: Optionale Transparenz des Panel-Hintergrunds. Dies wird durch Setzen der Opazität des Hauptfensters und/oder durch Verwendung von RGBA-Farben im CSS und im benutzerdefinierten Zeichencode erreicht. Für Wayland-Compositors, die transparente Oberflächen unterstützen, muss das zugrundeliegende GdkSurface entsprechend konfiguriert werden. Die gtk4-layer-shell kann hierbei relevant sein, um sicherzustellen, dass der Compositor die Transparenz korrekt handhabt.26  
* **"Leuchtakzent"-Effekt**: Ein subtiler Leuchteffekt entlang einer Kante des Panels (z.B. die dem Bildschirmzentrum zugewandte Kante), dessen Farbe und Intensität durch das Theming-System (domain::theming) gesteuert wird. Die Implementierung erfolgt entweder durch CSS (box-shadow mit entsprechenden Offsets und Blur-Radien 36) oder durch benutzerdefiniertes Zeichnen mit Cairo auf einem gtk::DrawingArea.37  
* **CSS-Styling**:  
  * **CSS-Knoten**: Das PanelWidget selbst (als GtkApplicationWindow) hat den CSS-Knoten window. Wenn es einen internen Hauptcontainer (z.B. GtkBox) verwendet, hat dieser den Knoten box.42 Spezifische CSS-Klassen werden zugewiesen, um das Styling zu erleichtern.  
  * **CSS-Klassen**:  
    * .nova-panel: Allgemeine Klasse für das Panel.  
    * .panel-top, .panel-bottom: Je nach Positionierung.  
    * .transparent-panel: Wenn Transparenz aktiviert ist.  
  * Die Anwendung von CSS erfolgt über einen globalen gtk::CssProvider, der durch ui::theming\_gtk verwaltet wird.10 Das Panel reagiert auf ThemeChangedEvents, um dynamische Stiländerungen zu übernehmen.

### **2.1.3. Datenstrukturen, Eigenschaften und Zustand**

Das PanelWidget wird als benutzerdefiniertes GObject-Widget implementiert, das von gtk::ApplicationWindow erbt, um die Integration mit gtk4-layer-shell zu ermöglichen.27

* GObject-Definition (PanelWidget):  
  Die Definition erfolgt in zwei Hauptdateien: mod.rs für die öffentliche API und imp.rs für die private GObject-Implementierung.  
  *Auszug aus src/ui/shell/panel\_widget/mod.rs (vereinfacht):*  
  Rust  
  use gtk::glib;  
  use gtk::subclass::prelude::\*;  
  use std::cell::{Cell, RefCell};

  mod imp;

  glib::wrapper\! {  
      pub struct PanelWidget(ObjectSubclass\<imp::PanelWidget\>)  
          @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,  
          @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;  
  }

  impl PanelWidget {  
      pub fn new(app: \&gtk::Application) \-\> Self {  
          glib::Object::builder::\<Self\>()  
             .property("application", app)  
             .build()  
      }

      // Öffentliche Methoden hier definieren, z.B.:  
      pub fn add\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>, position: imp::ModulePosition, order: i32) {  
          self.imp().add\_module(module, position, order);  
      }

      pub fn remove\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>) {  
          self.imp().remove\_module(module);  
      }  
  }

  *Auszug aus src/ui/shell/panel\_widget/imp.rs (vereinfacht):*  
  Rust  
  use gtk::glib;  
  use gtk::subclass::prelude::\*;  
  use gtk::{CompositeTemplate, Align};  
  use std::cell::{Cell, RefCell};  
  use std::collections::HashMap;  
  use once\_cell::sync::Lazy; // \[123\]

  // Enum für PanelPosition  
  \#  
  \#  
  pub enum PanelPosition {  
      \#\[default\]  
      Top,  
      Bottom,  
  }

  \#  
  \#  
  pub enum ModulePosition {  
      Start,  
      Center,  
      End,  
  }

  static PANEL\_PROPERTIES: Lazy\<Vec\<glib::ParamSpec\>\> \= Lazy::new(|| {  
      vec\!  
  });

  // Hier könnten benutzerdefinierte Signale definiert werden, falls benötigt.  
  // static PANEL\_SIGNALS: Lazy\<HashMap\<String, glib::subclass::Signal\>\> \= Lazy::new(|| HashMap::new());

  \#  
  \#\[template(resource \= "/org/nova\_de/ui/shell/panel\_widget.ui")\] // Pfad zur UI-Datei  
  pub struct PanelWidget {  
      \#\[template\_child\]  
      pub(super) main\_box: TemplateChild\<gtk::Box\>,  
      \#\[template\_child\]  
      pub(super) start\_box: TemplateChild\<gtk::Box\>,  
      \#\[template\_child\]  
      pub(super) center\_box: TemplateChild\<gtk::Box\>,  
      \#\[template\_child\]  
      pub(super) end\_box: TemplateChild\<gtk::Box\>,

      // Für benutzerdefiniertes Zeichnen, falls CSS nicht ausreicht  
      drawing\_area: RefCell\<Option\<gtk::DrawingArea\>\>,

      \#\[property(get, set, explicit\_notify)\]  
      position: RefCell\<PanelPosition\>,  
      \#\[property(get, set, explicit\_notify)\]  
      panel\_height: Cell\<i32\>,  
      \#\[property(get, set, explicit\_notify)\]  
      transparency\_enabled: Cell\<bool\>,  
      \#\[property(get, set, explicit\_notify)\]  
      leuchtakzent\_color: RefCell\<Option\<gdk::RGBA\>\>,  
      \#\[property(get, set, explicit\_notify)\]  
      leuchtakzent\_intensity: Cell\<f64\>,

      // Interne Verwaltung der Module  
      modules\_start: RefCell\<Vec\<gtk::Widget\>\>,  
      modules\_center: RefCell\<Vec\<gtk::Widget\>\>,  
      modules\_end: RefCell\<Vec\<gtk::Widget\>\>,  
  }

  \#\[glib::object\_subclass\]  
  impl ObjectSubclass for PanelWidget {  
      const NAME: &'static str \= "NovaDEPanelWidget";  
      type Type \= super::PanelWidget;  
      type ParentType \= gtk::ApplicationWindow;

      fn class\_init(klass: \&mut Self::Class) {  
          klass.bind\_template();  
          klass.install\_properties(\&PANEL\_PROPERTIES);  
          // klass.install\_signals(\&PANEL\_SIGNALS, false); // Falls Signale vorhanden

          // CSS-Name für das Widget setzen, falls nicht über UI-Datei  
          klass.set\_css\_name("panelwidget");  
      }

      fn instance\_init(obj: \&glib::subclass::InitializingObject\<Self\>) {  
          obj.init\_template();  
      }  
  }

  impl ObjectImpl for PanelWidget {  
      fn constructed(\&self) {  
          self.parent\_constructed();  
          let obj \= self.obj();

          // Standardwerte setzen, falls nicht durch Properties initialisiert  
          if self.position.borrow().eq(\&PanelPosition::default()) {  
               self.position.replace(PanelPosition::Top);  
          }  
          if self.panel\_height.get() \== 0 { // GObject Int default ist 0  
              self.panel\_height.set(36); // Expliziter Standardwert  
          }  
           if self.leuchtakzent\_intensity.get() \== 0.0 { // GObject Double default ist 0.0  
              self.leuchtakzent\_intensity.set(0.5);  
          }

          // Layer Shell initialisieren  
          obj.setup\_layer\_shell();  
          obj.update\_layout(); // Erstes Layout anwenden

          // Eventuell DrawingArea initialisieren und verbinden  
          // let drawing\_area \= gtk::DrawingArea::new();  
          // drawing\_area.set\_content\_width(obj.width\_request()); // Beispiel  
          // drawing\_area.set\_content\_height(self.panel\_height.get());  
          // self.main\_box.prepend(\&drawing\_area); // Oder als Hintergrund  
          // self.drawing\_area.replace(Some(drawing\_area));  
          // self.obj().connect\_draw\_signal();  
      }

      fn properties() \-\> &'static {  
          PANEL\_PROPERTIES.as\_ref()  
      }

      fn set\_property(\&self, \_id: usize, value: \&glib::Value, pspec: \&glib::ParamSpec) {  
          match pspec.name() {  
              "position" \=\> {  
                  let position: PanelPosition \= value.get().expect("Value must be PanelPosition");  
                  self.position.replace(position);  
                  self.obj().setup\_layer\_shell(); // Layer Shell neu konfigurieren bei Positionsänderung  
                  self.obj().notify\_position();   
              }  
              "panel-height" \=\> {  
                  let height: i32 \= value.get().expect("Value must be i32");  
                  self.panel\_height.set(height);  
                  self.obj().set\_default\_height(height); // Fensterhöhe anpassen  
                  self.main\_box.set\_height\_request(height);  
                  // Ggf. DrawingArea Höhe anpassen  
                  // if let Some(da) \= self.drawing\_area.borrow().as\_ref() {  
                  //    da.set\_content\_height(height);  
                  // }  
                  self.obj().queue\_draw(); // Neuzeichnen anfordern  
                  self.obj().notify\_panel\_height();  
              }  
              "transparency-enabled" \=\> {  
                  let enabled: bool \= value.get().expect("Value must be bool");  
                  self.transparency\_enabled.set(enabled);  
                  self.obj().update\_transparency();  
                  self.obj().notify\_transparency\_enabled();  
              }  
              "leuchtakzent-color" \=\> {  
                  let color: Option\<gdk::RGBA\> \= value.get().expect("Value must be Option\<gdk::RGBA\>");  
                  self.leuchtakzent\_color.replace(color);  
                  self.obj().queue\_draw();  
                  self.obj().notify\_leuchtakzent\_color();  
              }  
              "leuchtakzent-intensity" \=\> {  
                  let intensity: f64 \= value.get().expect("Value must be f64");  
                  self.leuchtakzent\_intensity.set(intensity);  
                  self.obj().queue\_draw();  
                  self.obj().notify\_leuchtakzent\_intensity();  
              }  
              \_ \=\> unimplemented\!(),  
          }  
      }

      fn property(\&self, \_id: usize, pspec: \&glib::ParamSpec) \-\> glib::Value {  
          match pspec.name() {  
              "position" \=\> self.position.borrow().to\_value(),  
              "panel-height" \=\> self.panel\_height.get().to\_value(),  
              "transparency-enabled" \=\> self.transparency\_enabled.get().to\_value(),  
              "leuchtakzent-color" \=\> self.leuchtakzent\_color.borrow().to\_value(),  
              "leuchtakzent-intensity" \=\> self.leuchtakzent\_intensity.get().to\_value(),  
              \_ \=\> unimplemented\!(),  
          }  
      }  
  }  
  impl WidgetImpl for PanelWidget {  
      fn map(\&self) {  
          self.parent\_map();  
          // Sicherstellen, dass Layer Shell korrekt initialisiert ist, bevor das Fenster angezeigt wird  
          self.obj().setup\_layer\_shell();  
      }  
       fn size\_allocate(\&self, width: i32, height: i32, baseline: i32) {  
          self.parent\_size\_allocate(width, height, baseline);  
          // Ggf. Layout der internen Boxen hier anpassen oder DrawingArea Größe  
      }  
  }  
  impl WindowImpl for PanelWidget {  
      // Fenster-spezifische Implementierungen, z.B. Schließen-Verhalten  
  }  
  impl ApplicationWindowImpl for PanelWidget {}

  // Implementierung der öffentlichen und privaten Methoden für PanelWidget  
  impl super::PanelWidget {  
      fn setup\_layer\_shell(\&self) {  
          let imp \= self.imp();  
          gtk\_layer\_shell::init\_for\_window(self);  
          gtk\_layer\_shell::set\_layer(self, gtk\_layer\_shell::Layer::Top);  
          gtk\_layer\_shell::set\_keyboard\_mode(self, gtk\_layer\_shell::KeyboardMode::None); // Panels benötigen i.d.R. keinen direkten Fokus  
          gtk\_layer\_shell::auto\_exclusive\_zone\_enable(self); // Platz reservieren  
          gtk\_layer\_shell::set\_namespace(self, "NovaDEPanel");

          let position \= \*imp.position.borrow();  
          match position {  
              PanelPosition::Top \=\> {  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Top, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Left, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Right, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Bottom, false);  
              }  
              PanelPosition::Bottom \=\> {  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Bottom, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Left, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Right, true);  
                  gtk\_layer\_shell::set\_anchor(self, gtk\_layer\_shell::Edge::Top, false);  
              }  
          }  
          self.set\_default\_height(imp.panel\_height.get());  
          // Margins könnten hier auch gesetzt werden, falls gewünscht  
          // gtk\_layer\_shell::set\_margin(self, gtk\_layer\_shell::Edge::Top, 5);  
      }

      fn update\_layout(\&self) {  
          let imp \= self.imp();  
          // Entferne alle Kinder aus start\_box, center\_box, end\_box  
          while let Some(child) \= imp.start\_box.first\_child() {  
              imp.start\_box.remove(\&child);  
          }  
          while let Some(child) \= imp.center\_box.first\_child() {  
              imp.center\_box.remove(\&child);  
          }  
          while let Some(child) \= imp.end\_box.first\_child() {  
              imp.end\_box.remove(\&child);  
          }

          // Füge Module entsprechend ihrer Reihenfolge und Position hinzu  
          // Diese Logik muss verfeinert werden, um die \`order\` Eigenschaft zu berücksichtigen  
          for widget in imp.modules\_start.borrow().iter() {  
              imp.start\_box.append(widget);  
          }  
          for widget in imp.modules\_center.borrow().iter() {  
              imp.center\_box.append(widget);  
          }  
          for widget in imp.modules\_end.borrow().iter() {  
              imp.end\_box.append(widget);  
          }  
      }

      fn update\_transparency(\&self) {  
          let imp \= self.imp();  
          let visual \= if imp.transparency\_enabled.get() {  
              self.display().rgba\_visual()  
          } else {  
              None // Oder Standard-Visual  
          };  
          self.set\_visual(visual.as\_ref()); // Benötigt GdkDisplay

          // Für echte Transparenz unter Wayland muss der Compositor dies unterstützen  
          // und das Fenster muss ggf. mit einem Alpha-Kanal gezeichnet werden.  
          // CSS kann auch für Hintergrundtransparenz verwendet werden.  
          self.queue\_draw();  
      }

      // Beispiel für das Verbinden des Draw-Signals, falls benutzerdefiniertes Zeichnen  
      // fn connect\_draw\_signal(\&self) {  
      //     if let Some(da) \= self.imp().drawing\_area.borrow().as\_ref() {  
      //        da.set\_draw\_func(glib::clone\!(@weak self as panel \=\> move |\_, cr, width, height| {  
      //            panel.imp().draw\_background\_and\_accent(cr, width, height);  
      //        }));  
      //    } else { // Wenn das PanelWindow selbst zeichnet (komplexer wegen Layer Shell)  
      //        self.connect\_realize(|widget| { // Realize statt draw für Fensterhintergrund  
      //            widget.set\_decorated(false); // Wichtig für custom drawing  
      //            if widget.imp().transparency\_enabled.get() {  
      //                 if let Some(surface) \= widget.surface() {  
      //                    surface.set\_opaque\_region(None); // Versuch für Transparenz  
      //                 }  
      //            }  
      //        });  
      //        // Das direkte Zeichnen auf einem GtkApplicationWindow ist nicht trivial.  
      //        // Besser ist ein Kind-Widget (GtkDrawingArea) zu verwenden.  
      //    }  
      // }  
  }

  // Private Implementierungsmethoden  
  impl PanelWidget {  
      fn add\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>, position: ModulePosition, \_order: i32) {  
          // TODO: Ordnung berücksichtigen  
          match position {  
              ModulePosition::Start \=\> {  
                  self.imp().modules\_start.borrow\_mut().push(module.clone().upcast());  
                  self.imp().start\_box.append(module);  
              }  
              ModulePosition::Center \=\> {  
                  self.imp().modules\_center.borrow\_mut().push(module.clone().upcast());  
                  self.imp().center\_box.append(module);  
              }  
              ModulePosition::End \=\> {  
                  self.imp().modules\_end.borrow\_mut().push(module.clone().upcast());  
                  self.imp().end\_box.append(module);  
              }  
          }  
          // Signal 'module-layout-changed' emittieren  
      }

      fn remove\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>) {  
          let widget\_ptr \= module.as\_ref().to\_glib\_none().0;  
          if self.imp().modules\_start.borrow\_mut().retain(|m| m.to\_glib\_none().0\!= widget\_ptr).len() \< self.imp().modules\_start.borrow().len() {  
               self.imp().start\_box.remove(module);  
          } else if self.imp().modules\_center.borrow\_mut().retain(|m| m.to\_glib\_none().0\!= widget\_ptr).len() \< self.imp().modules\_center.borrow().len() {  
               self.imp().center\_box.remove(module);  
          } else if self.imp().modules\_end.borrow\_mut().retain(|m| m.to\_glib\_none().0\!= widget\_ptr).len() \< self.imp().modules\_end.borrow().len() {  
               self.imp().end\_box.remove(module);  
          }  
          // Signal 'module-layout-changed' emittieren  
      }  
  }

* Eigenschaften (Properties):  
  Die GObject-Eigenschaften ermöglichen die Konfiguration und Zustandsabfrage des PanelWidget. Sie werden über das glib::Properties-Makro und die install\_properties-Methode im ObjectSubclass-Trait deklariert.47  
  **Tabelle: PanelWidget Eigenschaften**

| Eigenschaftsname | Typ | Zugriff | Standardwert | Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| position | PanelPosition | Lesen/Schreiben | Top | Bildschirmkante, an der das Panel verankert ist (Oben, Unten). |
| panel-height | i32 | Lesen/Schreiben | 36 | Höhe des Panels in Pixeln (Min: 24, Max: 128). |
| transparency-enabled | bool | Lesen/Schreiben | false | Gibt an, ob Transparenzeffekte für das Panel aktiv sind. |
| leuchtakzent-color | Option\<gdk::RGBA\> | Lesen/Schreiben | None | Farbe des Leuchtakzents. Wird typischerweise vom Theming-System aktualisiert. |
| leuchtakzent-intensity | f64 | Lesen/Schreiben | 0.5 | Intensität/Opazität des Leuchtakzents (Bereich: 0.0 bis 1.0). |

\*Bedeutung der Tabelle:\* Diese Tabelle bietet eine klare, strukturierte Definition der konfigurierbaren Aspekte des Panels. Sie ist essentiell für Entwickler, um die API des Widgets zu verstehen und es in Einstellungssysteme zu integrieren. Sie adressiert direkt die Anforderung der Anfrage nach der Definition von Eigenschaften mit exakten Typen und Initialwerten.

* **Interner Zustand:**  
  * modules\_start: RefCell\<Vec\<gtk::Widget\>\>: Speichert Referenzen auf die Panel-Module im Startbereich.  
  * modules\_center: RefCell\<Vec\<gtk::Widget\>\>: Speichert Referenzen auf die Panel-Module im Mittelbereich.  
  * modules\_end: RefCell\<Vec\<gtk::Widget\>\>: Speichert Referenzen auf die Panel-Module im Endbereich.  
  * Die Verwendung von RefCell ist notwendig für die innere Veränderlichkeit innerhalb des GObject-Systems, da GObject-Methoden typischerweise \&self erhalten.51

### **2.1.4. GTK-Widget-Implementierungsstrategie**

* **Basis-Widget**: Das PanelWidget erbt von gtk::ApplicationWindow.43 Diese Wahl ist entscheidend für die Integration mit gtk4-layer-shell, da dessen Funktionen wie init\_for\_window, set\_layer, set\_anchor und set\_margin auf einem gtk::Window operieren.26  
  * Die Initialisierung der Layer-Shell-Eigenschaften (gtk\_layer\_shell::init\_for\_window(self), etc.) muss erfolgen, bevor das Fenster zum ersten Mal realisiert (mapped) wird.29  
  * gtk\_layer\_shell::set\_layer(self.as\_ref(), gtk\_layer\_shell::Layer::Top) positioniert das Panel über normalen Anwendungsfenstern.  
  * gtk\_layer\_shell::set\_keyboard\_mode(self.as\_ref(), gtk\_layer\_shell::KeyboardMode::None) ist typisch für Panels, da sie selten direkten Tastaturfokus benötigen; dieser wird von den einzelnen Modulen gehandhabt.  
  * gtk\_layer\_shell::auto\_exclusive\_zone\_enable(self.as\_ref()) sorgt dafür, dass das Panel Platz auf dem Bildschirm reserviert und andere Fenster nicht verdeckt.  
* **Internes Layout**:  
  * Das PanelWidget verwendet eine panel\_widget.ui-Datei (Composite Template 55) oder definiert sein internes Layout programmatisch.  
  * Eine Haupt-gtk::Box (main\_box) mit horizontaler Orientierung dient als primärer Container.  
  * Innerhalb dieser main\_box befinden sich drei weitere gtk::Box-Widgets: start\_box, center\_box, und end\_box.42 Diese dienen zur Aufnahme der jeweiligen Panel-Module. start\_box und end\_box haben eine feste Größe basierend auf ihrem Inhalt, während center\_box den verbleibenden Raum einnimmt und sich horizontal ausdehnt (hexpand \= true).  
  * Alternativ kann gtk::CenterBox verwendet werden, wenn die UI-Definition dies unterstützt und die Anforderungen an die Ausrichtung der Kindelemente erfüllt.63  
* **Benutzerdefiniertes Zeichnen für "Leuchtakzent" und Hintergrund**:  
  * Falls CSS (box-shadow 36) für den "Leuchtakzent" oder komplexe Hintergründe nicht ausreicht oder die gewünschte Performance nicht liefert, wird ein gtk::DrawingArea eingesetzt.34  
  * Diese DrawingArea würde als unterste Ebene im PanelWidget platziert, oder das PanelWidget (als ApplicationWindow) muss seine Hintergrundzeichnung sorgfältig handhaben. Dies kann erreicht werden, indem das Fenster selbst transparent gemacht wird (widget.set\_visual(Some(\&display.rgba\_visual())) 34) und auf einer Kind-DrawingArea gezeichnet wird.  
  * Das draw-Signal der DrawingArea wird mit cairo-rs verwendet, um den Akzent und den Hintergrund zu zeichnen. Die Transparenz wird durch cairo::Context::set\_source\_rgba und die opacity-Eigenschaft von GtkWidget gesteuert.68

### **2.1.5. Methoden und Funktionssignaturen**

Die Methoden des PanelWidget definieren seine öffentliche API und interne Logik.

* **Öffentliche API (Auszug)**:  
  * pub fn add\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>, position: ModulePosition, order: i32) noexcept;  
    * Fügt ein gtk::Widget-basiertes Modul dem Panel hinzu.  
    * position: Enum (Start, Center, End), das den Bereich im Panel angibt.  
    * order: Ein i32-Wert, der die Reihenfolge innerhalb des Bereichs bestimmt (niedrigere Werte zuerst).  
  * pub fn remove\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>) noexcept;  
    * Entfernt ein zuvor hinzugefügtes Modul aus dem Panel.  
* **Interne Methoden (Auszug)**:  
  * fn setup\_layer\_shell(\&self) noexcept;  
    * Initialisiert und konfiguriert die gtk4-layer-shell-Eigenschaften basierend auf den aktuellen Panel-Einstellungen (Position, Höhe).  
  * fn update\_layout(\&self) noexcept;  
    * Ordnet die Module innerhalb der start\_box, center\_box und end\_box neu an, basierend auf ihrer order-Eigenschaft und aktuellen Konfiguration.  
  * fn draw\_background\_and\_accent(\&self, cr: \&cairo::Context, width: i32, height: i32) noexcept;  
    * Wird von der draw-Signal-Callback-Funktion der DrawingArea aufgerufen, um den benutzerdefinierten Hintergrund und den Leuchtakzent zu zeichnen. Verwendet leuchtakzent-color und leuchtakzent-intensity.  
  * fn update\_transparency(\&self) noexcept;  
    * Passt die Visuals des Fensters an, um Transparenz zu (de-)aktivieren.

**Tabelle: PanelWidget Methoden (Auswahl)**

| Signatur | Beschreibung | const | noexcept |
| :---- | :---- | :---- | :---- |
| pub fn new(app: \&gtk::Application) \-\> Self | Konstruktor, erstellt eine neue Instanz des PanelWidget. | Nein | Nein |
| pub fn add\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>, position: ModulePosition, order: i32) | Fügt ein Widget-Modul einem bestimmten Bereich (Start, Center, End) des Panels hinzu, unter Berücksichtigung der order. | Nein | Ja |
| pub fn remove\_module(\&self, module: \&impl glib::IsA\<gtk::Widget\>) | Entfernt das angegebene Widget-Modul aus dem Panel. | Nein | Ja |
| fn setup\_layer\_shell(\&self) | Interne Methode zur Konfiguration der gtk4-layer-shell-Parameter (Anker, Layer, Exklusivzone etc.) basierend auf den Panel-Eigenschaften wie position und panel-height. | Nein | Ja |
| fn update\_layout(\&self) | Interne Methode, die das Layout der Module in den Start-, Mittel- und Endbereichen aktualisiert, z.B. nach Hinzufügen/Entfernen eines Moduls oder einer Konfigurationsänderung. | Nein | Ja |

\*Bedeutung der Tabelle:\* Diese Tabelle ist entscheidend für Entwickler, die das \`PanelWidget\` verwenden oder erweitern, da sie einen klaren API-Vertrag bereitstellt und die Kernfunktionalitäten dokumentiert. Sie erfüllt die Anforderung der Anfrage nach exakten Methodensignaturen.

### **2.1.6. Signale**

Signale ermöglichen die Kommunikation von Zustandsänderungen oder Ereignissen des PanelWidget.

* **Benutzerdefinierte Signale**:  
  * module-layout-changed:  
    * Parameter: Keine.  
    * Emission: Wird emittiert, nachdem Module hinzugefügt, entfernt oder neu angeordnet wurden.  
    * Zweck: Ermöglicht anderen UI-Komponenten oder Logikmodulen, auf Änderungen im Panel-Layout zu reagieren.  
* **Verbundene Signale**:  
  * Lauscht auf ThemeChangedEvent von domain::theming::ThemingEngine:  
    * Handler-Aktion: Aktualisiert die Eigenschaft leuchtakzent-color und andere themenabhängige visuelle Aspekte. Fordert ein Neuzeichnen des Panels an (self.queue\_draw()).  
  * Verbindet sich mit notify::gtk-theme-name und notify::gtk-application-prefer-dark-theme von gtk::Settings::default() 10:  
    * Handler-Aktion: Lädt bei Bedarf Panel-spezifisches CSS neu oder passt Stile an, um Änderungen im System-Theme oder Dark-Mode-Präferenzen Rechnung zu tragen.

**Tabelle: PanelWidget emittierte Signale**

| Signalname | Parameter | Beschreibung |
| :---- | :---- | :---- |
| module-layout-changed | Keine | Wird emittiert, wenn sich die Anordnung oder der Satz der Module im Panel ändert. |

\*\*Tabelle: \`PanelWidget\` verbundene Signale\*\*

| Quelle | Signal | Handler-Aktion |
| :---- | :---- | :---- |
| domain::theming::ThemingEngine | ThemeChangedEvent | Aktualisiert leuchtakzent-color, fordert Neuzeichnen an. |
| gtk::Settings::default() | notify::gtk-theme-name | Lädt bei Bedarf panel-spezifisches CSS neu oder passt Stile an. |
| gtk::Settings::default() | notify::gtk-application-prefer-dark-theme | Passt Stile für Dark Mode an, lädt ggf. spezifisches CSS. |

\*Bedeutung der Tabellen:\* Diese Tabellen verdeutlichen die ereignisgesteuerten Interaktionen des \`PanelWidget\`. Dies ist entscheidend für das Verständnis seines dynamischen Verhaltens und für das Debugging.

### **2.1.7. Ereignisbehandlung**

* Das PanelWidget behandelt primär interne Layout-Aktualisierungen, die durch Eigenschaftsänderungen oder das Hinzufügen/Entfernen von Modulen ausgelöst werden.  
* Mausereignisse (z.B. enter-notify-event, leave-notify-event für Tooltips auf dem Panel selbst, falls vorhanden) werden über gtk::EventControllerMotion gehandhabt.70 Das Panel selbst wird jedoch in der Regel keinen komplexen Mausinteraktionen ausgesetzt sein; diese werden von den einzelnen Modulen übernommen.  
* Tastaturereignisse werden nicht direkt vom PanelWidget verarbeitet. Der Tastaturfokus wird von den einzelnen, fokussierbaren Panel-Modulen verwaltet.

### **2.1.8. Interaktionen**

* **domain::global\_settings\_and\_state\_management**:  
  * Liest die Panel-Konfiguration (Position, Höhe, Transparenzoptionen, Liste und Reihenfolge der Module) beim Start.  
  * Beobachtet Änderungen an diesen Einstellungen (z.B. über gio::Settings 21 oder ein anwendungsspezifisches Event-System), um das Panel dynamisch zu aktualisieren. Änderungen an Eigenschaften wie position oder panel-height führen zu Aufrufen von setup\_layer\_shell und update\_layout.  
* **system::compositor**:  
  * Die Interaktion erfolgt indirekt über die gtk4-layer-shell-Bibliothek.26 Das PanelWidget deklariert sich als Layer Surface (z.B. Layer::Top), setzt Anker und Margins, um seine Position und Größe relativ zum Output zu definieren.  
* **domain::theming::ThemingEngine**:  
  * Abonniert das ThemeChangedEvent, um Design-Tokens (insbesondere für leuchtakzent-color und Hintergrund) zu erhalten und anzuwenden. Dies löst ein Neuzeichnen des Panels aus.

### **2.1.9. Ausnahmebehandlung**

Zur robusten Fehlerbehandlung wird ein spezifischer Fehlertyp für das PanelWidget definiert.

* **enum PanelWidgetError** (definiert mit thiserror 72):  
  * LayerShellInitializationFailed(String): Wird zurückgegeben oder geloggt, wenn die Initialisierung mit gtk4-layer-shell fehlschlägt (z.B. wenn der Compositor das Protokoll nicht unterstützt).  
  * SettingsReadError(String): Wenn die Panel-Konfiguration nicht gelesen werden kann.  
  * InvalidModulePosition(String): Wenn versucht wird, ein Modul an einer ungültigen Position hinzuzufügen.  
* Fehler werden über das tracing-Crate geloggt 73, um Diagnose und Debugging zu erleichtern. Kritische Fehler, die die Funktionalität des Panels verhindern (z.B. LayerShellInitializationFailed), können dazu führen, dass das Panel nicht angezeigt wird, mit einer entsprechenden Log-Meldung.

### **2.1.10. Auflösung "Untersuchungsbedarf"**

* **Best Practices für gtk4-layer-shell-Integration**:  
  * Die Initialisierung der Layer-Shell-Eigenschaften (gtk\_layer\_shell::init\_for\_window, set\_layer, set\_anchor, set\_margin, auto\_exclusive\_zone\_enable) muss erfolgen, *bevor* das Panel-Fenster zum ersten Mal realisiert/gemappt wird. Dies geschieht typischerweise im constructed-Handler oder kurz vor dem ersten present()-Aufruf.26  
  * Der Tastaturinteraktivitätsmodus sollte sorgfältig gewählt werden. Für ein typisches Panel ist gtk\_layer\_shell::KeyboardMode::None oft angemessen, da die Panel-Module selbst den Fokus handhaben. KeyboardMode::OnDemand könnte relevant sein, wenn das Panel selbst oder bestimmte nicht-interaktive Bereiche des Panels temporär Fokus benötigen könnten.29  
  * Ein eindeutiger Namespace (z.B. "novade-panel") sollte mittels gtk\_layer\_shell::set\_namespace gesetzt werden. Dies hilft dem Compositor, verschiedene Layer-Shell-Clients zu identifizieren.29  
  * Für Multi-Monitor-Setups: Das Panel kann über gtk\_layer\_shell::set\_monitor einem spezifischen Monitor zugewiesen werden. Um Panels auf allen Monitoren darzustellen, müsste für jeden Monitor eine eigene PanelWidget-Instanz erstellt und konfiguriert werden. Die Liste der Monitore ist über gdk::Display::monitors() zugänglich.74 Änderungen in der Monitorkonfiguration (An-/Abstecken) können über Signale von gdk::Display (monitor-added, monitor-removed) überwacht werden.  
* **Implementierung des konfigurierbaren "Leuchtakzents" mit Cairo/GSK**:  
  * Das PanelWidget (oder eine dedizierte Kind-gtk::DrawingArea, die unter den Modul-Containern liegt) verbindet sich mit dem draw-Signal.  
  * Im Draw-Handler (fn draw\_background\_and\_accent):  
    1. Die aktuellen Werte der Eigenschaften leuchtakzent-color (ein gdk::RGBA) und leuchtakzent-intensity (ein f64 zwischen 0.0 und 1.0) werden abgerufen.  
    2. Der cairo::Context (cr) wird verwendet.  
    3. **Hintergrund zeichnen**: Zuerst wird der Panel-Hintergrund gezeichnet. Wenn Transparenz (transparency-enabled) aktiv ist, wird cr.set\_source\_rgba() mit einem Alpha-Wert \< 1.0 verwendet. Ansonsten eine deckende Farbe gemäß Theme. Abgerundete Ecken, falls spezifiziert, werden hier berücksichtigt (z.B. mit arc\_to und line\_to Pfaden).  
    4. **Leuchtakzent-Pfad definieren**: Ein Pfad wird für den Leuchteffekt erstellt. Dies könnte eine Linie oder ein schmales Rechteck entlang der Kante des Panels sein, die dem Bildschirmzentrum zugewandt ist. Die Position hängt von der position-Eigenschaft des Panels ab (oben oder unten).  
    5. **Leuchtakzent zeichnen**:  
       * **Farbe und Intensität**: cr.set\_source\_rgba() wird mit der leuchtakzent-color und einer durch leuchtakzent-intensity modulierten Alpha-Komponente aufgerufen.  
       * **Weicher Effekt**: Um einen weichen "Glow"-Effekt zu erzielen, können verschiedene Cairo-Techniken verwendet werden:  
         * **Gradienten**: Ein cairo::LinearGradient kann erstellt werden, der von der Akzentfarbe zu einer transparenten Version derselben Farbe oder zur Hintergrundfarbe übergeht. Der Gradient wird so ausgerichtet, dass er senkrecht zur Panelkante verläuft und nach außen hin ausblendet.41  
         * **Mehrfaches Zeichnen mit Unschärfe (simuliert)**: Da Cairo keine direkte Gausssche Unschärfe für Pfade bietet, kann ein ähnlicher Effekt durch mehrfaches Zeichnen des Akzentpfades mit leicht variierenden Offsets, Größen und abnehmender Deckkraft erzielt werden. Dies ist rechenintensiv und sollte mit Bedacht eingesetzt werden.  
         * **Schatten-API (falls anwendbar)**: Obwohl Cairo keine direkte box-shadow-Entsprechung für Pfade hat, könnte man einen Schatten simulieren, indem man eine versetzte, gefärbte und leicht transparente Version des Panelrands zeichnet und darüber den eigentlichen Panelinhalt.  
       * Die gezeichneten Elemente müssen die panel-height und die Gesamtbreite des Panels berücksichtigen.  
    6. Die GSK-Rendering-Pipeline von GTK4 wird diese Cairo-Operationen effizient auf die GPU übertragen.64 Es ist wichtig, queue\_draw() nur dann aufzurufen, wenn sich visuelle Aspekte tatsächlich ändern, um unnötiges Neuzeichnen zu vermeiden.  
  * Die Transparenz des Panel-Fensters selbst wird über gtk\_widget\_set\_opacity() 68 und die korrekte Konfiguration des GDK-Visuals für RGBA-Unterstützung gehandhabt, falls der Compositor dies erfordert und unterstützt.34

### **2.1.11. Dateistruktur**

Die Implementierung des PanelWidget wird in folgendem Verzeichnisbaum organisiert:

src/  
└── ui/  
    └── shell/  
        └── panel\_widget/  
            ├── mod.rs              // Öffentliche API, GObject Wrapper (PanelWidget struct)  
            ├── imp.rs              // Private GObject Implementierung (Subclass-Logik)  
            ├── panel\_widget.ui     // (Optional) XML-Definition für Composite Template  
            └── error.rs            // (Optional) Definition von PanelWidgetError

* mod.rs: Enthält die glib::wrapper\! Makrodefinition und öffentliche Methoden, die an die imp-Struktur delegieren.  
* imp.rs: Beinhaltet die \# Struktur, die \#\[glib::object\_subclass\] Implementierung und die Implementierungen für ObjectImpl, WidgetImpl, WindowImpl, und ApplicationWindowImpl. Hier werden Eigenschaften und Signale definiert und die Kernlogik des Widgets implementiert.  
* panel\_widget.ui: Falls Composite Templates für das interne Layout des Panels (z.B. die Anordnung von start\_box, center\_box, end\_box) verwendet werden, wird die XML-Struktur hier definiert.56  
* error.rs: Definiert PanelWidgetError unter Verwendung von thiserror.

Diese Struktur fördert die Modularität und Trennung von öffentlicher Schnittstelle und Implementierungsdetails, wie es in der gtk-rs Community üblich ist.5

### ---

**2.2. Sub-Modul: ui::shell::panel\_widget::AppMenuButton**

#### **2.2.1. Übersicht und Verantwortlichkeiten**

Das AppMenuButton ist ein spezialisiertes Panel-Modul, das als gtk::MenuButton (oder eine benutzerdefinierte Ableitung davon) implementiert wird. Seine Hauptverantwortung ist die Darstellung des globalen Anwendungsmenüs der aktuell fokussierten Applikation. Hierzu muss es:

1. Den app\_id (oder eine äquivalente Kennung) des aktiven Fensters ermitteln.  
2. Basierend auf dem app\_id das gio::MenuModel der aktiven Anwendung über D-Bus abrufen.  
3. Das abgerufene Menümodell in einem gtk::PopoverMenu darstellen, das beim Klick auf den Button erscheint.  
4. Das Aussehen des Buttons dynamisch an die aktive Anwendung anpassen (z.B. Icon und/oder Name anzeigen).

Die Komplexität dieser Komponente ergibt sich aus der Notwendigkeit, mit externen Systemkomponenten (Wayland Compositor für Fensterinformationen, D-Bus für Menüdaten) zu interagieren und auf Änderungen des Fensterfokus zu reagieren.

#### **2.2.2. Visuelles Design und Theming**

* **Anzeige**: Zeigt typischerweise das Icon der aktiven Anwendung. Falls kein Icon verfügbar ist oder keine Anwendung ein Menü bereitstellt, wird ein generisches "Anwendungsmenü"-Icon oder ein Platzhaltertext angezeigt.  
* **Beschriftung**: Kann optional den Namen der aktiven Anwendung neben dem Icon anzeigen, abhängig von der Konfiguration und dem verfügbaren Platz im Panel.  
* **Styling**:  
  * Als Instanz von gtk::MenuButton oder einer benutzerdefinierten, von gtk::Button abgeleiteten Klasse, die ein Popover öffnet. Es kann als ui::components::StyledButtonWidget implementiert werden, um ein konsistentes Erscheinungsbild mit anderen Buttons im Panel zu gewährleisten.  
  * **CSS-Knoten**: button (wenn von gtk::Button abgeleitet) oder menubutton (wenn von gtk::MenuButton).  
  * **CSS-Klassen**:  
    * .app-menu-button: Allgemeine Klasse für spezifisches Styling.  
    * .active-app: Wenn ein Anwendungsmenü erfolgreich geladen wurde.  
    * .no-app-menu: Wenn kein Menü für die aktive Anwendung verfügbar ist oder keine Anwendung fokussiert ist.  
* Der Tooltip des Buttons zeigt den Namen der aktiven Anwendung an, falls nicht bereits als Label sichtbar.76

#### **2.2.3. Datenstrukturen, Eigenschaften und Zustand**

Das AppMenuButton wird als GObject-Widget implementiert.

* **GObject-Definition (AppMenuButton)**:  
  *Auszug aus src/ui/shell/panel\_widget/app\_menu\_button/imp.rs (vereinfacht):*  
  Rust  
  use gtk::glib;  
  use gtk::subclass::prelude::\*;  
  use gtk::{gio, CompositeTemplate};  
  use std::cell::{Cell, RefCell};  
  use once\_cell::sync::Lazy;  
  use zbus::Connection; // \[12\]

  // Enum für den Status der Menüabfrage  
  \#  
  pub enum MenuFetchStatus {  
      \#\[default\]  
      Idle,  
      Loading,  
      Success,  
      Error(String), // Enthält Fehlermeldung  
  }

  static APP\_MENU\_BUTTON\_PROPERTIES: Lazy\<Vec\<glib::ParamSpec\>\> \= Lazy::new(|| {  
      vec\!  
  });

  \#  
  pub struct AppMenuButton {  
      // Eigenschaften  
      active\_app\_id: RefCell\<Option\<String\>\>,  
      active\_app\_name: RefCell\<Option\<String\>\>,  
      active\_app\_icon\_name: RefCell\<Option\<String\>\>,  
      has\_menu: Cell\<bool\>,  
      menu\_fetch\_status: RefCell\<MenuFetchStatus\>,

      // Interner Zustand  
      current\_menu\_model: RefCell\<Option\<gio::MenuModel\>\>,  
      dbus\_connection: RefCell\<Option\<Connection\>\>, // Zbus-Verbindung \[12\]

      // Referenz auf das GtkMenuButton-Widget selbst (oder das Popover, falls custom)  
      menu\_button\_widget: RefCell\<Option\<gtk::MenuButton\>\>, // Wird in constructed gesetzt  
  }

  \#\[glib::object\_subclass\]  
  impl ObjectSubclass for AppMenuButton {  
      const NAME: &'static str \= "NovaDEAppMenuButton";  
      type Type \= super::AppMenuButton;  
      type ParentType \= gtk::MenuButton; // Oder gtk::Button, wenn ein Popover manuell verwaltet wird

      fn new() \-\> Self {  
          Self {  
              active\_app\_id: RefCell::new(None),  
              active\_app\_name: RefCell::new(None),  
              active\_app\_icon\_name: RefCell::new(None),  
              has\_menu: Cell::new(false),  
              menu\_fetch\_status: RefCell::new(MenuFetchStatus::Idle),  
              current\_menu\_model: RefCell::new(None),  
              dbus\_connection: RefCell::new(None),  
              menu\_button\_widget: RefCell::new(None),  
          }  
      }

      fn class\_init(klass: \&mut Self::Class) {  
          klass.install\_properties(\&APP\_MENU\_BUTTON\_PROPERTIES);  
          // CSS-Name setzen  
          klass.set\_css\_name("appmenubutton");  
      }  
  }

  impl ObjectImpl for AppMenuButton {  
      fn constructed(\&self) {  
          self.parent\_constructed();  
          let obj \= self.obj();  
          // Speichere eine Referenz auf das Widget selbst für einfachen Zugriff  
          // self.menu\_button\_widget.replace(Some(obj.clone()));

          // Initialisiere D-Bus Verbindung und abonniere aktive Fensteränderungen  
          // Dies sollte idealerweise asynchron geschehen.  
          let widget \= obj.clone();  
          glib::MainContext::default().spawn\_local(async move {  
              match Connection::session().await { // \[12\]  
                  Ok(conn) \=\> {  
                      widget.imp().dbus\_connection.replace(Some(conn));  
                      // Hier Logik zum Abonnieren von Änderungen des aktiven Fensters einfügen  
                      // z.B. über einen internen Service, der Wayland-Events verarbeitet  
                      // widget.subscribe\_to\_active\_window\_changes();  
                  }  
                  Err(e) \=\> {  
                      tracing::error\!("Failed to connect to D-Bus for AppMenuButton: {}", e);  
                      widget.imp().menu\_fetch\_status.replace(MenuFetchStatus::Error(format\!("D-Bus connection failed: {}", e)));  
                      widget.update\_button\_appearance\_and\_state();  
                  }  
              }  
          });  
          obj.update\_button\_appearance\_and\_state(); // Initiales Aussehen  
      }

      fn properties() \-\> &'static {  
          APP\_MENU\_BUTTON\_PROPERTIES.as\_ref()  
      }

      fn property(\&self, \_id: usize, pspec: \&glib::ParamSpec) \-\> glib::Value {  
          match pspec.name() {  
              "active-app-name" \=\> self.active\_app\_name.borrow().to\_value(),  
              "active-app-icon-name" \=\> self.active\_app\_icon\_name.borrow().to\_value(),  
              "has-menu" \=\> self.has\_menu.get().to\_value(),  
              \_ \=\> unimplemented\!(),  
          }  
      }  
      // set\_property ist hier nicht nötig, da die Eigenschaften Read-only sind und intern gesetzt werden.  
  }  
  impl WidgetImpl for AppMenuButton {  
      fn map(\&self) {  
          self.parent\_map();  
          // Beim Sichtbarwerden ggf. aktuellen Status neu abfragen  
          self.obj().trigger\_menu\_update\_for\_current\_app();  
      }  
  }  
  impl ButtonImpl for AppMenuButton {} // Falls ParentType gtk::Button  
  impl MenuButtonImpl for AppMenuButton {} // Falls ParentType gtk::MenuButton

* **Eigenschaften (Properties)**:  
  **Tabelle: AppMenuButton Eigenschaften**

| Eigenschaftsname | Typ | Zugriff | Standardwert | Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| active-app-name | Option\<String\> | Nur Lesen | None | Name der Anwendung, deren Menü aktuell angezeigt wird oder angezielt ist. |
| active-app-icon-name | Option\<String\> | Nur Lesen | None | Icon-Name (für Theming) der Anwendung, deren Menü angezielt ist. |
| has-menu | bool | Nur Lesen | false | true, wenn ein Menü für die aktive Anwendung verfügbar und geladen ist. |

\*Bedeutung der Tabelle:\* Definiert den beobachtbaren Zustand des \`AppMenuButton\`, nützlich für Binding oder um auf Änderungen im Menü der aktiven Anwendung zu reagieren.

* **Interner Zustand**:  
  * active\_app\_id: RefCell\<Option\<String\>\>: Speichert die ID der aktuell fokussierten Anwendung.  
  * menu\_fetch\_status: RefCell\<MenuFetchStatus\>: Verfolgt den Zustand des Menüabrufs.  
  * current\_menu\_model: RefCell\<Option\<gio::MenuModel\>\>: Hält das aktuell geladene Menümodell.  
  * dbus\_connection: RefCell\<Option\<zbus::Connection\>\>: Die D-Bus-Verbindung für Abfragen.

#### **2.2.4. GTK-Widget-Implementierung**

* Das AppMenuButton erbt von gtk::MenuButton.77 Diese Klasse bietet bereits die Funktionalität, ein Popover beim Klick anzuzeigen.  
* Das Popover wird ein gtk::PopoverMenu sein.79  
* Die Eigenschaft menu-model des gtk::MenuButton (oder des internen gtk::PopoverMenu) wird dynamisch mit dem über D-Bus abgerufenen gio::MenuModel aktualisiert.  
  * gtk::MenuButton::set\_menu\_model(Some(menu\_model))  
  * Wenn kein Menü verfügbar ist, wird gtk::MenuButton::set\_menu\_model(None) gesetzt oder das Popover deaktiviert.

#### **2.2.5. Methoden und Funktionssignaturen**

* **Öffentliche Methoden (vom Panel oder einem Dienst für aktive Fenster aufgerufen)**:  
  * pub fn update\_active\_window\_info(\&self, app\_id: Option\<String\>, window\_title: Option\<String\>, icon\_name: Option\<String\>) noexcept;  
    * Wird aufgerufen, wenn sich das aktive Fenster *oder* dessen Metadaten ändern.  
    * Speichert app\_id, window\_title, icon\_name intern.  
    * Löst trigger\_menu\_update\_for\_current\_app aus.  
    * Aktualisiert sofort das Aussehen des Buttons (Icon/Label) basierend auf icon\_name und window\_title/app\_id.  
* **Interne Methoden**:  
  * fn trigger\_menu\_update\_for\_current\_app(\&self) noexcept;  
    * Prüft, ob active\_app\_id gesetzt ist.  
    * Wenn ja, startet die asynchrone fetch\_menu\_for\_app-Operation.  
    * Setzt menu\_fetch\_status auf Loading.  
    * Aktualisiert das Button-Aussehen (z.B. Ladeindikator).  
  * async fn fetch\_menu\_for\_app(dbus\_conn: Connection, app\_id: String) \-\> Result\<gio::MenuModel, AppMenuError\>;  
    * Diese Funktion ist async und wird mit glib::MainContext::spawn\_local ausgeführt.7  
    * Versucht, das gio::MenuModel für den gegebenen app\_id über D-Bus zu beziehen (siehe 2.2.8 Interaktionen).  
    * Gibt das gio::MenuModel oder einen AppMenuError zurück.  
  * fn handle\_menu\_fetch\_result(\&self, result: Result\<gio::MenuModel, AppMenuError\>) noexcept;  
    * Wird im glib::MainContext aufgerufen, nachdem fetch\_menu\_for\_app abgeschlossen ist.  
    * Aktualisiert current\_menu\_model, has\_menu, und menu\_fetch\_status.  
    * Ruft display\_menu und update\_button\_appearance\_and\_state auf.  
  * fn display\_menu(\&self) noexcept;  
    * Setzt das current\_menu\_model auf den gtk::MenuButton.  
  * fn update\_button\_appearance\_and\_state(\&self) noexcept;  
    * Aktualisiert Icon (z.B. gtk::Image::set\_from\_icon\_name 84) und Label des gtk::MenuButton basierend auf active\_app\_icon\_name, active\_app\_name und menu\_fetch\_status.  
    * Setzt den sensitive-Zustand des Buttons (z.B. deaktiviert, wenn kein Menü geladen werden kann oder Loading).  
    * Aktualisiert die GObject-Properties (active-app-name, active-app-icon-name, has-menu) und emittiert notify Signale.

**Tabelle: AppMenuButton Methoden (Auswahl)**

| Signatur | Beschreibung | async | noexcept |
| :---- | :---- | :---- | :---- |
| pub fn update\_active\_window\_info(\&self, app\_id: Option\<String\>, window\_title: Option\<String\>, icon\_name: Option\<String\>) | Aktualisiert die Informationen über das aktive Fenster und löst ggf. eine Menüaktualisierung aus. | Nein | Ja |
| fn trigger\_menu\_update\_for\_current\_app(\&self) | Startet den Prozess zum Abrufen und Anzeigen des Menüs für die aktuell zwischengespeicherte app\_id. | Nein | Ja |
| async fn fetch\_menu\_for\_app(dbus\_conn: Connection, app\_id: String) \-\> Result\<gio::MenuModel, AppMenuError\> | Ruft asynchron das GMenuModel für die gegebene app\_id über D-Bus ab. | Ja | Nein |
| fn handle\_menu\_fetch\_result(\&self, result: Result\<gio::MenuModel, AppMenuError\>) | Verarbeitet das Ergebnis von fetch\_menu\_for\_app, aktualisiert den internen Zustand und die UI. | Nein | Ja |

#### **2.2.6. Signale**

* **Benutzerdefinierte Signale**: Keine spezifischen benutzerdefinierten Signale für diese Komponente vorgesehen. Es erbt die Signale von gtk::MenuButton (z.B. clicked, activate).  
* **Verbundene Signale**:  
  * Intern: Lauscht auf ein Signal von einem übergeordneten Dienst (z.B. innerhalb von ui::shell), das Änderungen des aktiven Fensters (app\_id, Titel, Icon) meldet.

#### **2.2.7. Ereignisbehandlung**

* Die Hauptinteraktion ist der Klick auf den Button, der durch die gtk::MenuButton-Basisklasse gehandhabt wird und das Popover mit dem Menü anzeigt.  
* Interne Reaktionen auf die Ergebnisse der asynchronen D-Bus-Menüabfrage und auf Änderungen des aktiven Fensters sind entscheidend für die dynamische Aktualisierung.

#### **2.2.8. Interaktionen**

* **system::compositor (Fensterinformationen)**:  
  * Das AppMenuButton selbst interagiert nicht direkt mit dem Compositor. Es ist auf einen Dienst innerhalb der ui::shell angewiesen, der Informationen über das aktive Fenster bereitstellt. Dieser Dienst nutzt Wayland-Protokolle.  
  * **Wayland-Protokolle**:  
    * wlr-foreign-toplevel-management-unstable-v1: Dieses Protokoll ermöglicht es einem Client (dem NovaDE-Shell-Dienst), eine Liste von Toplevel-Fenstern zu erhalten und deren Zustände (inkl. app\_id, title, state) zu überwachen. Der Dienst würde das activated-Ereignis nutzen, um das aktuell fokussierte Fenster zu identifizieren.85  
    * ext-foreign-toplevel-list-v1: Ein alternatives oder ergänzendes Protokoll, das ebenfalls zur Auflistung von Toplevel-Fenstern dient.85  
  * Die Implementierung dieser Wayland-Client-Logik sollte zentral in einem ui::shell-Modul erfolgen (z.B. ui::shell::active\_window\_service) und nicht im AppMenuButton selbst, um Redundanz zu vermeiden und die Komplexität zu kapseln. Dieser Dienst würde dann ein internes Signal oder einen Event für das AppMenuButton bereitstellen.  
* **D-Bus (Menüabruf)**:  
  * Sobald der app\_id des aktiven Fensters bekannt ist, wird versucht, dessen Menümodell über D-Bus abzurufen.  
  * **Primärer Mechanismus (org.gtk.Menus)**:  
    * GTK4-Anwendungen, die GApplication verwenden, exportieren ihre Menüs (typischerweise GMenuModel für Anwendungsmenü und Menüleiste) oft über D-Bus unter ihrem eigenen Bus-Namen (welcher dem app\_id entspricht, z.B. org.gnome.TextEditor).  
    * Der Standard-Objektpfad für das Menü ist oft /org/gtk/menus/menubar oder ein ähnlicher, durch GApplication festgelegter Pfad.91  
    * Die Schnittstelle ist org.gtk.Menus.  
    * gio::DBusMenuModel::new(bus\_name, object\_path) kann verwendet werden, um ein GMenuModel direkt von einem D-Bus-Dienst zu erstellen, was die Details der Methodenaufrufe abstrahiert.21  
  * **Fallback-Mechanismus (com.canonical.AppMenu.Registrar)**:  
    * Ein älterer Mechanismus, der von Unity verwendet wurde. Anwendungen registrieren ihre Fenster-ID und den D-Bus-Pfad zu ihrem Menü bei diesem Dienst.15  
    * Dienstname: com.canonical.AppMenu.Registrar  
    * Objektpfad: /com/canonical/AppMenu/Registrar  
    * Schnittstelle: com.canonical.AppMenu.Registrar  
    * Methode: GetMenuForWindow(window\_id\_uint32). Dies ist problematisch in einer reinen Wayland-Umgebung, da X11-Fenster-IDs nicht direkt verfügbar oder relevant sind. Eine Wayland-kompatible Anwendung müsste ihren Menüpfad auf andere Weise bekannt geben.  
  * **D-Bus-Client-Implementierung**: Das zbus-Crate wird verwendet, um D-Bus-Proxies zu erstellen und Methoden aufzurufen.12  
    * Ein zbus::Proxy wird für den Zieldienst erstellt (entweder der app\_id oder com.canonical.AppMenu.Registrar).  
    * Die entsprechenden Methoden werden asynchron aufgerufen.  
    * Das Ergebnis (oft ein Pfad zu einem DBusMenu-Objekt) wird verwendet, um ein gio::MenuModel zu instanziieren, typischerweise mit gio::DBusMenuModel.

**Tabelle: AppMenuButton D-Bus Interaktionen**

| Interaktion | Zieldienst (Primär) | Objektpfad (Primär) | Schnittstelle (Primär) | Methode/Eigenschaft (Primär) | Zieldienst (Fallback) | Objektpfad (Fallback) | Schnittstelle (Fallback) | Methode (Fallback) |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| GMenuModel für aktive Anwendung abrufen | \[app\_id\_der\_aktiven\_Anwendung\] | /org/gtk/menus/menubar (oder Konvention) | org.gtk.Menus (oder org.freedesktop.DBus.Properties) | gio::DBusMenuModel::new(bus\_name, object\_path) (abstrahiert Methodenaufrufe) | com.canonical.AppMenu.Registrar | /com/canonical/AppMenu/Registrar | com.canonical.AppMenu.Registrar | GetMenuForWindow (XID-abhängig) oder App registriert Menüpfad |

\*Bedeutung der Tabelle:\* Verdeutlicht die komplexen D-Bus-Interaktionen, die für das Abrufen von Anwendungsmenüs erforderlich sind. Dies ist entscheidend für die Implementierung und das Debugging, insbesondere angesichts der verschiedenen Mechanismen, über die Menüs bereitgestellt werden können.

#### **2.2.9. Ausnahmebehandlung**

* **enum AppMenuError** (definiert mit thiserror 72):  
  * WaylandError(String): Fehler beim Abrufen von Informationen zum aktiven Fenster.  
  * DBusConnectionError(zbus::Error): Fehler bei der D-Bus-Kommunikation.  
  * MenuServiceUnavailable(String): Der D-Bus-Dienst für die Anwendung (z.B. app\_id oder AppMenu.Registrar) ist nicht erreichbar.  
  * MenuNotFound(String): Die Anwendung (app\_id) exportiert kein bekanntes Menü oder das Menü ist leer.  
  * MenuModelParseError(String): Fehler beim Parsen oder Interpretieren der Menüdaten.  
* Im Fehlerfall zeigt der AppMenuButton einen deaktivierten Zustand oder ein generisches Icon an. Fehlerdetails werden über tracing geloggt.73

#### **2.2.10. Auflösung "Untersuchungsbedarf"**

* **Zuverlässige Methode zur Ermittlung des aktiven Fensters/app\_id unter Wayland**:  
  * Die bevorzugte Methode ist die Verwendung des wlr-foreign-toplevel-management-unstable-v1-Protokolls.86 Ein zentraler Shell-Dienst (nicht der AppMenuButton selbst) agiert als Client dieses Protokolls.  
  * Der Dienst bindet sich an den globalen zwlr\_foreign\_toplevel\_manager\_v1.  
  * Für jedes gemeldete Toplevel (zwlr\_foreign\_toplevel\_handle\_v1) lauscht der Dienst auf die Ereignisse app\_id, title und state.  
  * Das state-Ereignis enthält Flags, darunter activated. Das Toplevel mit dem activated-Flag ist das aktuell fokussierte Fenster.  
  * Die smithay-client-toolkit 85 könnte Rust-Abstraktionen für dieses Protokoll bereitstellen. Falls nicht, ist die direkte Verwendung von wayland-client mit den wayland-protocols-Bindings (speziell wlr-protocols) notwendig.89  
  * Dieser zentrale Dienst stellt dann die Informationen über das aktive Fenster (insbesondere app\_id, title, icon\_name) dem AppMenuButton und anderen interessierten UI-Komponenten über ein internes Event-System oder Signale zur Verfügung.  
* **Ermittlung und Konsumierung von GMenuModel via D-Bus**:  
  * **Primärer Pfad (für GTK4-Anwendungen)**: Moderne GTK-Anwendungen, die GApplication verwenden, exportieren ihr Hauptmenü (GMenuModel) typischerweise über D-Bus auf ihrem eigenen, durch den app\_id bestimmten Bus-Namen. Der Objektpfad ist oft standardisiert, z.B. /org/gtk/menus/menubar oder ein anderer Pfad, den GApplication für diesen Zweck nutzt.91 Ein gio::DBusMenuModel wird dann mit diesem Bus-Namen und Objektpfad instanziiert, um das Menümodell zu erhalten.21  
  * **Fallback (StatusNotifierItem)**: Falls eine Anwendung ein StatusNotifierItem bereitstellt, kann dessen Menu-Eigenschaft einen D-Bus-Objektpfad zu einem Menü (oft im com.canonical.dbusmenu-Format) enthalten.102 Dies ist relevant, wenn die Anwendung primär über ein Tray-Icon interagiert.  
  * **Fallback (AppMenu Registrar)**: Der com.canonical.AppMenu.Registrar D-Bus-Dienst ist ein älterer Mechanismus.15 Seine Verwendung in einer reinen Wayland-Umgebung ist aufgrund der Abhängigkeit von X11-Fenster-IDs problematisch und sollte nur als letzte Option in Betracht gezogen werden, falls Anwendungen keine anderen Mechanismen anbieten.  
  * **Implementierungsentscheidung**: Die Strategie sollte sein, zuerst den primären Pfad (org.gtk.Menus auf dem app\_id-Bus) zu versuchen. Schlägt dies fehl und ist ein StatusNotifierItem für die App vorhanden, kann dessen Menüpfad versucht werden. Der AppMenuRegistrar wird aufgrund seiner X11-Lastigkeit tendenziell vermieden.  
  * Das AppMenuButton verwendet das erhaltene gio::MenuModel, um seinen internen gtk::PopoverMenu zu füllen.79

Die Implementierung des AppMenuButton erfordert eine sorgfältige Orchestrierung asynchroner Operationen für Wayland-Events und D-Bus-Aufrufe, um die UI reaktionsfähig zu halten (glib::MainContext::spawn\_local 7). Fehlerzustände (z.B. keine aktive Anwendung, keine Menüdaten, D-Bus-Fehler) müssen robust gehandhabt und dem Benutzer klar signalisiert werden (z.B. durch ein deaktiviertes oder generisches Icon).

#### **2.2.11. Dateistruktur**

src/  
└── ui/  
    └── shell/  
        └── panel\_widget/  
            └── app\_menu\_button/  
                ├── mod.rs          // Öffentliche API, GObject Wrapper (AppMenuButton struct)  
                ├── imp.rs          // Private GObject Implementierung  
                ├── dbus.rs         // Logik für D-Bus Interaktionen (Menüabruf)  
                └── error.rs        // Definition von AppMenuError

Diese Struktur kapselt die Komplexität des AppMenuButton und trennt die D-Bus-Logik klar ab.  
---

**(Hinweis: Die detaillierte Ausarbeitung weiterer Submodule des PanelWidget wie WorkspaceIndicatorWidget, ClockDateTimeWidget, SystemTrayEquivalentWidget etc. würde einem ähnlichen Detaillierungsgrad folgen und die spezifischen "Untersuchungsbedarfe" adressieren. Insbesondere das SystemTrayEquivalentWidget erfordert eine tiefgreifende Auseinandersetzung mit der StatusNotifierItem-Spezifikation und deren D-Bus-Implementierung mittels zbus, wie in der Gliederung angedeutet.102)**  
Die Implementierung eines SystemTrayEquivalentWidget ist ein komplexes Unterfangen, da Wayland selbst kein natives "System Tray"-Protokoll definiert. Die De-facto-Standardlösung ist die StatusNotifierItem (SNI) Spezifikation von Freedesktop.org, die auf D-Bus basiert.102  
Ein SystemTrayEquivalentWidget müsste folgende Kernkomponenten umfassen:

1. **StatusNotifierHost-Registrierung**: Das Panel (oder dieses Widget) muss sich als org.freedesktop.StatusNotifierHost auf dem Session-Bus registrieren. Dies signalisiert dem StatusNotifierWatcher, dass ein Host für Items vorhanden ist.106 Die Registrierung erfolgt typischerweise durch das Anfordern eines eindeutigen Bus-Namens (z.B. org.freedesktop.StatusNotifierHost-PID oder org.freedesktop.StatusNotifierHost-NovaDE).  
2. **Interaktion mit StatusNotifierWatcher**:  
   * Der StatusNotifierWatcher (org.freedesktop.StatusNotifierWatcher) ist der zentrale Dienst zur Verwaltung von SNIs.102  
   * Das Widget muss diesen Watcher auf dem D-Bus finden (Standardname org.freedesktop.StatusNotifierWatcher, Pfad /org/freedesktop/StatusNotifierWatcher).  
   * Es muss die Methode RegisterStatusNotifierHost am Watcher aufrufen, um sich selbst als Host zu registrieren.  
   * Es muss die Eigenschaft RegisteredStatusNotifierItems des Watchers abfragen, um eine initiale Liste aller bereits vorhandenen SNIs zu erhalten.  
   * Es muss die Signale StatusNotifierItemRegistered und StatusNotifierItemUnregistered des Watchers abonnieren, um dynamisch auf neue oder entfernte SNIs zu reagieren. zbus wird hierfür verwendet, um Signal-Handler einzurichten.109  
3. **Interaktion mit einzelnen StatusNotifierItems**:  
   * Für jeden von StatusNotifierWatcher gemeldeten Dienstnamen eines SNI (z.B. org.freedesktop.StatusNotifierItem-PID-ID) muss ein zbus::Proxy erstellt werden.17  
   * Über diesen Proxy werden die Eigenschaften des SNI ausgelesen: Category, Id, Title, Status, WindowId, IconName, IconPixmap, OverlayIconName, OverlayIconPixmap, AttentionIconName, AttentionIconPixmap, AttentionMovieName, ToolTip, ItemIsMenu, Menu.102  
   * Signale des SNI (z.B. NewIcon, NewStatus, NewToolTip, NewMenu) müssen abonniert werden, um auf Änderungen zu reagieren und die Darstellung des entsprechenden Indikator-Widgets im Panel zu aktualisieren.16  
4. **Darstellung der Indikatoren**:  
   * Für jedes aktive SNI wird ein kleines Widget im SystemTrayEquivalentWidget (das selbst eine gtk::Box oder ein ähnlicher Container ist) angezeigt.  
   * **Icon**: IconName wird verwendet, um ein themenbasiertes Icon über gtk::Image::from\_icon\_name zu laden.84 Falls IconPixmap bereitgestellt wird, müssen die Rohpixeldaten (oft ein Array von Tupeln (width, height, data)) in ein gdk\_pixbuf::Pixbuf konvertiert werden (z.B. mit Pixbuf::from\_mut\_slice oder PixbufLoader, falls die Daten gestreamt ankommen, was hier aber unwahrscheinlich ist) und dann in einem gtk::Image angezeigt werden.117  
   * **Tooltip**: Die ToolTip-Eigenschaft des SNI (eine Struktur mit Titel, Text, Icon) wird verwendet, um einen Tooltip für das Indikator-Widget mittels gtk::Widget::set\_tooltip\_markup oder gtk::Widget::set\_tooltip\_text zu setzen.76  
   * **Status**: Die Status-Eigenschaft (Passive, Active, NeedsAttention) kann verwendet werden, um das Aussehen des Indikators anzupassen (z.B. Hervorhebung bei NeedsAttention).  
5. **Interaktion mit den Indikatoren**:  
   * **Linksklick (Activate)**: Ein Klick auf das Indikator-Widget ruft die Activate(x, y)-Methode des SNI über D-Bus auf.103  
   * **Rechtsklick (ContextMenu)**: Ein Rechtsklick ruft die ContextMenu(x, y)-Methode des SNI auf. Wenn die ItemIsMenu-Eigenschaft true ist und die Menu-Eigenschaft einen gültigen D-Bus-Pfad zu einem com.canonical.dbusmenu-Objekt enthält, wird dieses Menü abgerufen (mittels gio::DBusMenuModel 96) und als gtk::PopoverMenu angezeigt.79  
   * **Scrollen**: Mausrad-Events über dem Indikator rufen die Scroll(delta, orientation)-Methode des SNI auf.  
6. **Asynchronität**: Alle D-Bus-Interaktionen (Methodenaufrufe, Signal-Handling) müssen asynchron mit glib::MainContext::spawn\_local erfolgen, um die UI nicht zu blockieren.7

Die "Alternativen unter Wayland" 104 beziehen sich darauf, dass Wayland selbst kein Tray-Protokoll spezifiziert. StatusNotifierItem ist die etablierte D-Bus-basierte Lösung. Einige Desktop-Umgebungen könnten eigene Protokolle haben, aber für eine breite Kompatibilität ist SNI der Standard. Die Herausforderung besteht darin, dass nicht alle Anwendungen SNI korrekt oder vollständig implementieren.

## ---

**3\. Übergreifende Belange – Initiale Spezifikationen**

Dieser Abschnitt definiert initiale Strategien für Aspekte, die mehrere UI-Module betreffen und eine konsistente Handhabung erfordern.

### **3.1. UI-Zustandsverwaltungsstrategie**

Die Verwaltung des UI-Zustands ist entscheidend für eine reaktive und wartbare Benutzeroberfläche. In NovaDE wird ein mehrschichtiger Ansatz verfolgt, der die Stärken von GObject mit Rust-Idiomen kombiniert:

* **GObject-Eigenschaften für Widget-Zustand**:  
  * Der primäre Mechanismus zur Verwaltung des Zustands einzelner Widgets sind GObject-Eigenschaften. Diese werden mit dem glib::Properties-Derive-Makro und klass.install\_properties() in der ObjectSubclass-Implementierung definiert.47  
  * Beispiel: Die panel-height-Eigenschaft des PanelWidget.  
  * Änderungen an diesen Eigenschaften lösen automatisch "notify::property-name"-Signale aus, auf die andere Teile der UI oder die interne Logik des Widgets reagieren können. Explizite Benachrichtigung kann mit self.obj().notify\_propertyName() erzwungen werden, falls die automatische Benachrichtigung nicht ausreicht oder benutzerdefinierte Logik vor der Benachrichtigung ausgeführt werden muss.  
* **Benutzerdefinierte GObject-Signale**:  
  * Für komplexere Zustandsänderungen oder Ereignisse, die nicht direkt durch eine einzelne Eigenschaftsänderung abgebildet werden, werden benutzerdefinierte GObject-Signale definiert.115  
  * Beispiel: Das module-layout-changed-Signal des PanelWidget.  
  * Signale werden in ObjectImpl::signals() definiert und können mit self.obj().emit\_by\_name::\<()\>("signal-name", &\[\&param1, \&param2\]) ausgelöst werden.  
* **Rc\<RefCell\<T\>\> für gemeinsam genutzten UI-Zustand**:  
  * Für UI-Zustände, die von mehreren Widgets gemeinsam genutzt werden und nicht in einer direkten GObject-Eltern-Kind-Beziehung stehen oder nicht sinnvoll als globale GSettings abgebildet werden können, wird das Rust-Idiom Rc\<RefCell\<T\>\> verwendet.51  
  * Rc ermöglicht das Teilen des Besitzes im Single-Threaded-Kontext des GTK-Mainloops.  
  * RefCell ermöglicht die innere Veränderlichkeit (mutable borrows zur Laufzeit geprüft).  
  * Dies ist nützlich für z.B. einen gemeinsam genutzten D-Bus-Verbindungsmanager, der von mehreren UI-Komponenten verwendet wird, oder für View-Modelle, die Daten für mehrere, lose gekoppelte Widgets halten.  
  * Vorsicht ist geboten, um Zyklen von Rc-Referenzen zu vermeiden, die zu Speicherlecks führen können. Weak\<RefCell\<T\>\> kann hier Abhilfe schaffen.  
* **Datenbindung (Property Binding)**:  
  * GObject-Eigenschaftsbindungen (GObject.bind\_property()) werden intensiv genutzt, um UI-Elemente direkt an Zustandseigenschaften zu koppeln. Dies reduziert manuellen Synchronisationscode und fördert eine deklarative UI-Logik.  
  * Beispiel: Die label-Eigenschaft eines gtk::Label könnte an eine String-Eigenschaft eines View-Modell-Objekts gebunden werden.  
* **Adaption von MVVM/MVC-Mustern**:  
  * Obwohl GTK nicht explizit für ein bestimmtes UI-Architekturmuster wie MVVM oder MVC ausgelegt ist, können deren Prinzipien adaptiert werden:  
    * **Model**: Repräsentiert die Anwendungsdaten und Geschäftslogik (primär in der domain-Schicht, aber auch UI-spezifische Zustandsmodelle).  
    * **View**: Die GTK-Widgets selbst.  
    * **ViewModel/Controller**: GObject-Instanzen, die UI-spezifische Logik und Zustand halten (ViewModel-Aspekt) und Benutzerinteraktionen verarbeiten (Controller-Aspekt). GObject-Eigenschaften des ViewModels werden an die View (Widgets) gebunden. Methoden im ViewModel/Controller reagieren auf UI-Events und interagieren mit dem Model.  
* **Kommunikation mit unteren Schichten**:  
  * Zustandsänderungen, die von der domain- oder system-Schicht ausgehen (z.B. durch Ereignisse oder Callbacks von asynchronen Operationen), werden in UI-Zustandsaktualisierungen übersetzt.  
  * Dies geschieht typischerweise innerhalb von Closures, die mit glib::MainContext::spawn\_local auf dem UI-Thread ausgeführt werden, um Thread-Sicherheit zu gewährleisten.7  
  * Beispiel: Ein NetworkStatusChangedEvent aus der system-Schicht könnte die icon-name-Eigenschaft eines NetworkIndicatorWidget aktualisieren.

Dieser Ansatz ermöglicht eine klare Trennung der Belange, nutzt die Stärken des GObject-Systems für Widget-spezifischen Zustand und bietet gleichzeitig flexible Rust-basierte Lösungen für komplexere oder gemeinsam genutzte UI-Zustände.

### **3.2. Fehlerbehandlungs-Framework für die UI-Schicht**

Eine konsistente und benutzerfreundliche Fehlerbehandlung ist unerlässlich.

* **Fehlerdefinition mit thiserror**:  
  * Für jedes Hauptmodul der UI-Schicht (z.B. ui::shell, ui::control\_center) und ggf. für komplexe Submodule (z.B. AppMenuButton) werden spezifische Error-Enums mit thiserror::Error definiert.72 Beispiel: PanelWidgetError, AppMenuError.  
  * Diese modul-spezifischen Fehler werden in einem übergeordneten UI-Fehler-Enum (z.B. NovaUiError) zusammengefasst, ebenfalls unter Verwendung von \#\[from\]-Attributen in thiserror für eine einfache Konvertierung.  
    Rust  
    // Beispiel: src/ui/error.rs  
    use thiserror::Error;

    \#  
    pub enum PanelWidgetError {  
        \#\[error("Layer shell initialization failed: {0}")\]  
        LayerShellInitializationFailed(String),  
        // Weitere Panel-spezifische Fehler  
    }

    \#  
    pub enum AppMenuError {  
        \#\[error("Failed to get active window info from Wayland: {0}")\]  
        WaylandError(String),  
        \#  
        DBusConnectionError(\#\[from\] zbus::Error),  
        \#\[error("Menu service for app '{0}' unavailable")\]  
        MenuServiceUnavailable(String),  
        \#\[error("Menu not found for app '{0}'")\]  
        MenuNotFound(String),  
    }

    \#  
    pub enum NovaUiError {  
        \#\[error("Panel widget error: {0}")\]  
        Panel(\#\[from\] PanelWidgetError),  
        \#\[error("AppMenu button error: {0}")\]  
        AppMenu(\#\[from\] AppMenuError),  
        \#  
        Theming(String), // Fehler von ui::theming\_gtk  
        \#\[error("I/O error: {0}")\]  
        Io(\#\[from\] std::io::Error),  
        // Weitere Fehlerkategorien  
    }

* **Fehlerdarstellung**:  
  * **Kritische Fehler**: Fehler, die die grundlegende Funktionalität einer Komponente oder der UI stark beeinträchtigen (z.B. D-Bus-Verbindung nicht möglich, Layer-Shell-Initialisierung fehlgeschlagen), werden dem Benutzer über ein gtk::AlertDialog mitgeteilt. Der Dialog sollte eine klare Fehlermeldung und ggf. Vorschläge zur Fehlerbehebung oder einen Hinweis auf Log-Dateien enthalten.  
  * **Nicht-kritische Fehler**: Weniger schwerwiegende Fehler (z.B. ein einzelnes Panel-Modul kann nicht geladen werden, eine Einstellung kann nicht gelesen werden) werden als NotificationPopupWidget (siehe ui::notifications\_frontend) oder durch eine Zustandsänderung im Widget selbst (z.B. ausgegrautes Icon, Fehlermeldung im Tooltip) angezeigt.  
  * Fehlermeldungen für den Benutzer werden internationalisiert (i18n).  
* **Fehlerpropagation**: Fehler aus unteren Schichten (domain, system) werden in entsprechende NovaUiError-Varianten umgewandelt und nach oben propagiert oder an der Stelle behandelt, an der sie für die UI relevant werden.

### **3.3. Logging-Strategie**

Strukturiertes Logging ist für Diagnose und Debugging unerlässlich.

* **Bibliothek**: Das tracing-Crate wird für alle Logging-Aufgaben in der UI-Schicht verwendet.73  
* **Log-Level**:  
  * trace\!: Sehr detaillierte Informationen für tiefgreifendes Debugging (z.B. einzelne D-Bus-Nachrichten, detaillierte Widget-Zustandsänderungen). Standardmäßig deaktiviert.  
  * debug\!: Informationen, die für das Debugging nützlich sind (z.B. Erstellung von Widgets, Aufruf wichtiger interner Methoden, empfangene Ereignisse).  
  * info\!: Allgemeine Informationen über den Betrieb (z.B. Modul geladen, Einstellung geändert).  
  * warn\!: Unerwartete, aber nicht unbedingt fehlerhafte Zustände (z.B. optionale Konfigurationsdatei nicht gefunden, Fallback-Verhalten aktiviert).  
  * error\!: Fehlerzustände, die die Funktionalität beeinträchtigen (z.B. D-Bus-Aufruf fehlgeschlagen, Widget konnte nicht erstellt werden). Details zum Fehlerobjekt werden mitgeloggt.  
* **Strukturierte Felder**: Log-Nachrichten sollen relevante Kontextinformationen als strukturierte Felder enthalten.  
  * Beispiel: tracing::debug\!(widget\_name \= %self.widget\_name(), event \=?event\_type, "Event received");  
* **Span-Nutzung**: tracing::span\! wird verwendet, um wichtige Operationen oder Lebenszyklen von Komponenten zu umfassen, insbesondere bei asynchronen Abläufen.  
* **Konfiguration**:  
  * Die Konfiguration des tracing-Subscribers (z.B. tracing\_subscriber::fmt für Konsolenausgabe oder tracing\_journald für systemd-journal-Integration) erfolgt im Hauptanwendungseinstiegspunkt (main.rs).  
  * Die Standard-Logstufe für Entwicklungs-Builds ist DEBUG, für Release-Builds INFO. Die Logstufe kann zur Laufzeit über Umgebungsvariablen (z.B. RUST\_LOG) angepasst werden.

### **3.4. Initiales Teststrategie-Framework**

Eine mehrschichtige Teststrategie stellt die Qualität und Korrektheit der UI-Schicht sicher.

* **Unit-Tests**:  
  * Fokus: Testen von isolierter Logik innerhalb von UI-Komponenten, die nicht direkt vom GTK-Rendering oder \-Eventloop abhängt (z.B. Hilfsfunktionen, Datenkonvertierungslogik, Zustandsmanagement-Helfer).  
  * Werkzeuge: Standard Rust \#\[test\], Mocking-Bibliotheken (z.B. mockall) für Abhängigkeiten zu unteren Schichten oder externen Diensten.  
* **Widget-Tests**:  
  * Fokus: Testen des Verhaltens und Zustands einzelner GTK-Widgets und benutzerdefinierter GObject-Komponenten.  
  * Werkzeuge:  
    * gtk::test Namespace: Bietet Funktionen zum Initialisieren von GTK in Testumgebungen.  
    * Programmatische Interaktion: Simulieren von Signalen (z.B. widget.emit\_by\_name::\<()\>("clicked", &)), Setzen und Abfragen von GObject-Eigenschaften.  
    * Inspektion: Überprüfung von Widget-Zuständen (z.B. label.text(), button.is\_sensitive()).  
    * GTK-Inspektionswerkzeuge und Accessibility-APIs (ATK) können programmatisch genutzt werden, um Widget-Zustände und \-Eigenschaften zu überprüfen.122 Die Evaluierung von Frameworks wie gtk4-rs-test-utils (falls existent und passend) oder ähnlichen Ansätzen ist Teil des Untersuchungsbedarfs.  
* **Accessibility-Tests**:  
  * Fokus: Sicherstellen, dass UI-Komponenten für assistive Technologien zugänglich sind.  
  * Werkzeuge: Überprüfung von ATK-Eigenschaften (Rolle, Name, Beschreibung, Zustand) der Widgets. Manuelle Tests mit Screenreadern (z.B. Orca) sind ebenfalls notwendig. gtk::Accessible.23  
* **Visuelle Regressionstests**: (Zur Evaluierung)  
  * Fokus: Erkennen von unbeabsichtigten visuellen Änderungen in der UI.  
  * Werkzeuge: Evaluierung von Werkzeugen für den visuellen Vergleich von UI-Zuständen (Screenshots). Dies ist oft aufwendig und wird initial möglicherweise zurückgestellt.  
* **Integrations-/End-to-End-Tests**: (Herausfordernd, für kritische Pfade)  
  * Fokus: Testen des Zusammenspiels mehrerer UI-Komponenten und deren Interaktion mit unteren Schichten.  
  * Werkzeuge: Simulation von Benutzerinteraktionen auf Wayland-Ebene (z.B. mit Tools wie ydotool oder spezialisierten Test-Frameworks, falls verfügbar und integrierbar). Überprüfung des Systemverhaltens. Dies ist sehr komplex und wird nur für kritische User Journeys in Betracht gezogen.

### **3.5. Richtlinien für Performance-Optimierung und Profiling**

Die Sicherstellung einer performanten UI ist ein Kernziel.

* **Profiling-Werkzeuge**:  
  * **Rust-spezifisch**: perf unter Linux, cargo flamegraph, tracing mit tracing-flame für CPU-Profiling. Speicher-Profiler wie heaptrack oder Valgrind (mit Massif) können zur Analyse des Speicherverbrauchs herangezogen werden.  
  * **GTK4-spezifisch**: Der GTK Inspector enthält einen Profiler, der Rendering-Zeiten und Widget-Updates visualisiert. GSK-spezifische Debug-Flags (GSK\_DEBUG) können Aufschluss über Rendering-Pfade geben.  
* **Optimierungsbereiche**:  
  * **Widget-Zeichnung**: Bei benutzerdefinierten Zeichnungen mit Cairo (GtkDrawingArea) darauf achten, nur die notwendigen Bereiche neu zu zeichnen (gtk\_widget\_queue\_draw\_area). Komplexität der Zeichenoperationen minimieren.  
  * **CSS-Anwendung**: CSS-Selektoren einfach halten. Komplexe Selektoren und Regeln können die Performance beeinträchtigen. Effiziente Aktualisierung von CSS bei Theme-Wechseln.10  
  * **Datenbindung**: Übermäßige Nutzung von GObject-Property-Bindings oder zu häufige Benachrichtigungen bei kleinen Änderungen können zu Performance-Engpässen führen. Änderungen ggf. bündeln.  
  * **Layout-Performance**: Vermeidung unnötig tiefer Widget-Hierarchien. Effiziente Nutzung von Layout-Managern wie GtkBox und GtkGrid.  
  * **Asynchrone Operationen**: Konsequente Nutzung von glib::MainContext::spawn\_local für alle potenziell blockierenden Operationen (Netzwerk, Datei-I/O, aufwändige Berechnungen in der Domänenschicht), um UI-Blockaden zu verhindern.7 Visuelles Feedback (Spinner, Fortschrittsbalken) für laufende Operationen bereitstellen.  
* **Allgemeine Rust-Optimierungen**: Zero-Cost-Abstraktionen nutzen, unnötige Allokationen vermeiden, effiziente Datenstrukturen wählen.24

Performance-Messungen und \-Optimierungen sollten ein integraler Bestandteil des Entwicklungsprozesses sein, nicht eine nachträgliche Maßnahme.

## **4\. Plan für nachfolgende UI-Layer-Module**

### **4.1. Priorisierung für nächste Module**

Nach der initialen Implementierung und Stabilisierung des PanelWidget und des AppMenuButton sowie der grundlegenden übergreifenden Frameworks (Theming, State Management, Error Handling, Logging) werden die UI-Module in folgender logischer Reihenfolge priorisiert:

1. **Weitere Kern-Panel-Module (ui::shell)**:  
   * WorkspaceIndicatorWidget: Essentiell für die Workspace-Navigation.  
   * ClockDateTimeWidget: Grundlegende Benutzerinformation.  
   * SystemTrayEquivalentWidget: Kritisch für die Integration von Drittanbieter-Anwendungen. Aufgrund seiner Komplexität (siehe oben) wird hierfür frühzeitig mit der Detailplanung und Prototyping begonnen.  
   * QuickSettingsButtonWidget und NotificationCenterButtonWidget: Wichtige Zugriffspunkte für Systemfunktionen.  
   * Weitere Indikatoren (NetworkIndicatorWidget, PowerIndicatorWidget, AudioIndicatorWidget).  
2. **Ausklappbare Panel-Inhalte (ui::shell)**:  
   * QuickSettingsPanelWidget: Wird vom QuickSettingsButtonWidget geöffnet.  
   * NotificationCenterPanelWidget: Wird vom NotificationCenterButtonWidget geöffnet und interagiert mit ui::notifications\_frontend.  
3. **Weitere Shell-Komponenten (ui::shell)**:  
   * SmartTabBarWidget  
   * WorkspaceSwitcherWidget  
   * QuickActionDockWidget  
4. **Systemeinstellungsanwendung (ui::control\_center)**:  
   * Dies ist eine größere, eigenständige Anwendung und wird parallel zu weniger kritischen Shell-Komponenten entwickelt, sobald die Kern-Shell-Interaktionen stabil sind.  
5. **Spezifische UI-Frontends und Widgets**:  
   * ui::notifications\_frontend (Popups)  
   * ui::widgets (Sidebar-Widgets)  
   * ui::window\_manager\_frontend  
   * ui::speed\_dial  
   * ui::command\_palette

Diese Priorisierung zielt darauf ab, schnell einen funktionalen Kern der Desktop-Shell zu etablieren und dann schrittweise weitere Funktionen und Anwendungen hinzuzufügen.

### **4.2. Identifizierte Abhängigkeiten und Parallelisierungsmöglichkeiten**

* **Abhängigkeiten**:  
  * Alle Panel-Module hängen von einem stabilen PanelWidget und dessen API ab.  
  * AppMenuButton und SystemTrayEquivalentWidget haben starke Abhängigkeiten von D-Bus-Interaktionen und Wayland-Protokollen (bzw. den Abstraktionsdiensten dafür).  
  * NetworkIndicatorWidget, PowerIndicatorWidget, AudioIndicatorWidget hängen von den entsprechenden D-Bus-Schnittstellen der system-Schicht ab.  
  * NotificationCenterPanelWidget hängt von domain::user\_centric\_services::NotificationService und ui::notifications\_frontend::NotificationPopupWidget ab.  
  * ui::control\_center Module hängen stark von domain::global\_settings\_and\_state\_management::GlobalSettingsService ab.  
* **Parallelisierung**:  
  * Sobald die API des PanelWidget definiert ist, können viele der darin enthaltenen Module (ClockDateTimeWidget, WorkspaceIndicatorWidget, einzelne Indikatoren) parallel entwickelt werden.  
  * Die Entwicklung des ui::control\_center kann weitgehend parallel zur Verfeinerung der ui::shell erfolgen, sobald die GlobalSettingsService-Schnittstelle stabil ist.  
  * Wiederverwendbare Komponenten in ui::components können frühzeitig parallel entwickelt und in anderen Modulen eingesetzt werden.  
  * Die Implementierung der D-Bus-Clients für verschiedene Systemdienste (NetworkManager, UPower, etc.) kann parallelisiert werden.

Eine enge Abstimmung zwischen den Teams, die an abhängigen Modulen arbeiten, ist entscheidend. Die Definition klarer Schnittstellen (GObject-Properties und \-Signale, Rust-Traits) für Module und Dienste erleichtert die parallele Entwicklung und spätere Integration.

#### **Referenzen**

1. gtk4-rs/README.md at main \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/gtk-rs/gtk4-rs/blob/main/README.md](https://github.com/gtk-rs/gtk4-rs/blob/main/README.md)  
2. GTK and Rust \- The GTK Project \- A free and open-source cross-platform widget toolkit, Zugriff am Mai 14, 2025, [https://www.gtk.org/docs/language-bindings/rust](https://www.gtk.org/docs/language-bindings/rust)  
3. gtk4 \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/gtk4](https://docs.rs/gtk4)  
4. Patterns \- The Rust Reference, Zugriff am Mai 14, 2025, [https://doc.rust-lang.org/stable/reference/patterns.html?highlight=Patterns](https://doc.rust-lang.org/stable/reference/patterns.html?highlight=Patterns)  
5. Rust: Project structure example step by step \- DEV Community, Zugriff am Mai 14, 2025, [https://dev.to/ghost/rust-project-structure-example-step-by-step-3ee](https://dev.to/ghost/rust-project-structure-example-step-by-step-3ee)  
6. How to structure files in rust projects? \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/rust/comments/6qbi91/how\_to\_structure\_files\_in\_rust\_projects/](https://www.reddit.com/r/rust/comments/6qbi91/how_to_structure_files_in_rust_projects/)  
7. MainContext in glib \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/glib/latest/glib/struct.MainContext.html](https://docs.rs/glib/latest/glib/struct.MainContext.html)  
8. MainContext in glib \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/struct.MainContext.html](https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/struct.MainContext.html)  
9. How to force custom GTK4 theme to follow preferred theme? : r/gnome \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/gnome/comments/1kasp2c/how\_to\_force\_custom\_gtk4\_theme\_to\_follow/](https://www.reddit.com/r/gnome/comments/1kasp2c/how_to_force_custom_gtk4_theme_to_follow/)  
10. jbenner-radham/rust-gtk4-css-styling: How to style your ... \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/jbenner-radham/rust-gtk4-css-styling](https://github.com/jbenner-radham/rust-gtk4-css-styling)  
11. Gtk.CssProvider, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.CssProvider.html](https://docs.gtk.org/gtk4/class.CssProvider.html)  
12. zbus \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/](https://docs.rs/zbus/latest/zbus/)  
13. dbus2/zbus: Rust D-Bus crate. \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/dbus2/zbus](https://github.com/dbus2/zbus)  
14. zbus \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/zbus](https://crates.io/crates/zbus)  
15. zbus \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/zbus/3.15.2](https://crates.io/crates/zbus/3.15.2)  
16. Writing a service interface \- zbus: D-Bus for Rust made easy, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/service.html](https://dbus2.github.io/zbus/service.html)  
17. "dbus\_proxy" Search \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/?search=dbus\_proxy](https://docs.rs/zbus/latest/zbus/?search=dbus_proxy)  
18. proxy in zbus \- Rust \- openrr.github.io, Zugriff am Mai 14, 2025, [https://openrr.github.io/openrr/zbus/attr.proxy.html](https://openrr.github.io/openrr/zbus/attr.proxy.html)  
19. Introduction \- zbus: D-Bus for Rust made easy, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/](https://dbus2.github.io/zbus/)  
20. Fighting with zbus implementation \- help \- The Rust Programming Language Forum, Zugriff am Mai 14, 2025, [https://users.rust-lang.org/t/fighting-with-zbus-implementation/106696](https://users.rust-lang.org/t/fighting-with-zbus-implementation/106696)  
21. Gio – 2.0 \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gio/index.html](https://docs.gtk.org/gio/index.html)  
22. Zugriff am Januar 1, 1970, [https://github.com/dbus2/zbus/tree/main/zbus/examples](https://github.com/dbus2/zbus/tree/main/zbus/examples)  
23. Gtk – 4.0: Accessibility, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/section-accessibility.html](https://docs.gtk.org/gtk4/section-accessibility.html)  
24. Ultimate Rust Performance Optimization Guide 2024: Basics to Advanced \- Rapid Innovation, Zugriff am Mai 14, 2025, [https://www.rapidinnovation.io/post/performance-optimization-techniques-in-rust](https://www.rapidinnovation.io/post/performance-optimization-techniques-in-rust)  
25. Ultimate Rust FPS Optimization Guide ✔️ \- YouTube, Zugriff am Mai 14, 2025, [https://m.youtube.com/watch?v=lQxeBhTgPEQ](https://m.youtube.com/watch?v=lQxeBhTgPEQ)  
26. wmww/gtk4-layer-shell: A library to create panels and other ... \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/wmww/gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell)  
27. gtk4-layer-shell \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/gtk4-layer-shell](https://crates.io/crates/gtk4-layer-shell)  
28. gtk4-layer-shell/examples/simple-example.c at main \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/wmww/gtk4-layer-shell/blob/main/examples/simple-example.c](https://github.com/wmww/gtk4-layer-shell/blob/main/examples/simple-example.c)  
29. LayerShell in gtk\_layer\_shell \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/gtk-layer-shell/latest/gtk\_layer\_shell/trait.LayerShell.html](https://docs.rs/gtk-layer-shell/latest/gtk_layer_shell/trait.LayerShell.html)  
30. drkrssll/chunks-rs: Simplifies the process of making GTK4 widgets for wayland compositors. \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/drkrssll/chunks-rs/](https://github.com/drkrssll/chunks-rs/)  
31. Zugriff am Januar 1, 1970, [https://github.com/pentamassiv/gtk4-layer-shell-gir/tree/main/gtk4-layer-shell/examples](https://github.com/pentamassiv/gtk4-layer-shell-gir/tree/main/gtk4-layer-shell/examples)  
32. Zugriff am Januar 1, 1970, [https://github.com/pentamassiv/gtk4-layer-shell-gir/tree/main/examples](https://github.com/pentamassiv/gtk4-layer-shell-gir/tree/main/examples)  
33. Zugriff am Januar 1, 1970, [https://docs.rs/gtk4-layer-shell/latest/gtk\_layer\_shell/trait.LayerShell.html](https://docs.rs/gtk4-layer-shell/latest/gtk_layer_shell/trait.LayerShell.html)  
34. DrawingArea in gtk4 \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/0.8/docs/gtk4/struct.DrawingArea.html](https://gtk-rs.org/gtk4-rs/stable/0.8/docs/gtk4/struct.DrawingArea.html)  
35. gtk4/NEWS.pre-4.0 at layer-shell\_impish \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/pop-os/gtk4/blob/layer-shell\_impish/NEWS.pre-4.0](https://github.com/pop-os/gtk4/blob/layer-shell_impish/NEWS.pre-4.0)  
36. Gtk – 4.0: GTK CSS Properties, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/css-properties.html](https://docs.gtk.org/gtk4/css-properties.html)  
37. DrawingArea in gtk4 \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/0.5/docs/gtk4/struct.DrawingArea.html](https://gtk-rs.org/gtk4-rs/stable/0.5/docs/gtk4/struct.DrawingArea.html)  
38. DrawingArea in gtk4 \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.DrawingArea.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.DrawingArea.html)  
39. How do I create a glow effect like this? I tried gaussian blur, but it doesn't really look like this : r/krita \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/krita/comments/1j7innz/how\_do\_i\_create\_a\_glow\_effect\_like\_this\_i\_tried/](https://www.reddit.com/r/krita/comments/1j7innz/how_do_i_create_a_glow_effect_like_this_i_tried/)  
40. Creating a highlight effect with the cairo library \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/9955964/creating-a-highlight-effect-with-the-cairo-library](https://stackoverflow.com/questions/9955964/creating-a-highlight-effect-with-the-cairo-library)  
41. Cooking with Cairo \- Cairo graphics library, Zugriff am Mai 14, 2025, [https://www.cairographics.org/cookbook/](https://www.cairographics.org/cookbook/)  
42. GtkBox, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Box.html](https://docs.gtk.org/gtk4/class.Box.html)  
43. IsA for subclasses in GTK Rust \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/75623312/isa-for-subclasses-in-gtk-rust](https://stackoverflow.com/questions/75623312/isa-for-subclasses-in-gtk-rust)  
44. gtk4-rs/book/listings/actions/6/resources/window.ui at main \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/gtk-rs/gtk4-rs/blob/main/book/listings/actions/6/resources/window.ui](https://github.com/gtk-rs/gtk4-rs/blob/main/book/listings/actions/6/resources/window.ui)  
45. How to make gtk4-rs use native windows decorations? : r/rust \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/rust/comments/1jdm50v/how\_to\_make\_gtk4rs\_use\_native\_windows\_decorations/](https://www.reddit.com/r/rust/comments/1jdm50v/how_to_make_gtk4rs_use_native_windows_decorations/)  
46. Gtk.ApplicationWindow, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.ApplicationWindow.html](https://docs.gtk.org/gtk4/class.ApplicationWindow.html)  
47. Properties \- GUI development with Rust and GTK 4, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/g\_object\_properties.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/g_object_properties.html)  
48. Zugriff am Januar 1, 1970, [https://gtk-rs.org/gtk4-rs/stable/latest/book/gobject\_properties.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/gobject_properties.html)  
49. GObject – 2.0 \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gobject/index.html](https://docs.gtk.org/gobject/index.html)  
50. Zugriff am Januar 1, 1970, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/glib/index.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/glib/index.html)  
51. Memory Management \- GUI development with Rust and GTK 4, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/g\_object\_memory\_management.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/g_object_memory_management.html)  
52. glib \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/](https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/)  
53. Zugriff am Januar 1, 1970, [https://gtk-rs.org/gtk4-rs/stable/latest/book/gobject\_memory\_management.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/gobject_memory_management.html)  
54. GLib – 2.0 \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/glib/index.html](https://docs.gtk.org/glib/index.html)  
55. Counter App with GTK4 CompositeTemplate and Rust \- DEV Community, Zugriff am Mai 14, 2025, [https://dev.to/kashifsoofi/counter-app-with-gtk4-compositetemplate-and-rust-h0p](https://dev.to/kashifsoofi/counter-app-with-gtk4-compositetemplate-and-rust-h0p)  
56. Composite Templates \- GUI development with Rust and GTK 4, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/composite\_templates.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/composite_templates.html)  
57. Writing a Custom GTK widget with template UI files \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/77768792/writing-a-custom-gtk-widget-with-template-ui-files](https://stackoverflow.com/questions/77768792/writing-a-custom-gtk-widget-with-template-ui-files)  
58. Counter App with GTK4 CompositeTemplate and Rust \- Kashif Soofi, Zugriff am Mai 14, 2025, [https://kashifsoofi.github.io/gtk4/rust/gtk4-rust-counter-app-with-template/](https://kashifsoofi.github.io/gtk4/rust/gtk4-rust-counter-app-with-template/)  
59. Building a Simple To-Do App \- GUI development with Rust and GTK 4, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/todo\_1.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/todo_1.html)  
60. Introduction \- GUI development with Rust and GTK 4 \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/introduction.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/introduction.html)  
61. Gtk.Builder \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Builder.html](https://docs.gtk.org/gtk4/class.Builder.html)  
62. Box in gtk4 \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Box.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.Box.html)  
63. Gtk.CenterBox, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.CenterBox.html](https://docs.gtk.org/gtk4/class.CenterBox.html)  
64. Zugriff am Januar 1, 1970, [https://docs.gtk.org/gtk4/section-drawing-model.html](https://docs.gtk.org/gtk4/section-drawing-model.html)  
65. Zugriff am Januar 1, 1970, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/cairo/index.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/cairo/index.html)  
66. Pango – 1.0 \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/Pango/index.html](https://docs.gtk.org/Pango/index.html)  
67. cairo-rs 0.20.7 \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/cairo-rs/latest/cairo/struct.ImageSurface.html](https://docs.rs/cairo-rs/latest/cairo/struct.ImageSurface.html)  
68. Gtk.Widget, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Widget.html](https://docs.gtk.org/gtk4/class.Widget.html)  
69. Gtk.Settings, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Settings.html](https://docs.gtk.org/gtk4/class.Settings.html)  
70. geting the coordinates of the user's mouse pointer in GTK4 in C \- Stack Overflow, Zugriff am Mai 14, 2025, [https://stackoverflow.com/questions/79016771/geting-the-coordinates-of-the-users-mouse-pointer-in-gtk4-in-c](https://stackoverflow.com/questions/79016771/geting-the-coordinates-of-the-users-mouse-pointer-in-gtk4-in-c)  
71. Gtk.EventControllerMotion, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.EventControllerMotion.html](https://docs.gtk.org/gtk4/class.EventControllerMotion.html)  
72. thiserror \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/thiserror/latest/thiserror/](https://docs.rs/thiserror/latest/thiserror/)  
73. tracing \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/tracing/latest/tracing/](https://docs.rs/tracing/latest/tracing/)  
74. Gdk.Display, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gdk4/class.Display.html](https://docs.gtk.org/gdk4/class.Display.html)  
75. Gdk.Monitor, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gdk4/class.Monitor.html](https://docs.gtk.org/gdk4/class.Monitor.html)  
76. Gtk.Widget:tooltip-text, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/property.Widget.tooltip-text.html](https://docs.gtk.org/gtk4/property.Widget.tooltip-text.html)  
77. Gtk.Button, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Button.html](https://docs.gtk.org/gtk4/class.Button.html)  
78. Gtk.MenuButton, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.MenuButton.html](https://docs.gtk.org/gtk4/class.MenuButton.html)  
79. Gtk.PopoverMenu, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.PopoverMenu.html](https://docs.gtk.org/gtk4/class.PopoverMenu.html)  
80. PopoverMenu in gtk4 \- Rust \- gtk-rs, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.PopoverMenu.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/struct.PopoverMenu.html)  
81. API changes in GTK4: removal of GtkMenu : r/GTK \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/GTK/comments/xdfgjr/api\_changes\_in\_gtk4\_removal\_of\_gtkmenu/](https://www.reddit.com/r/GTK/comments/xdfgjr/api_changes_in_gtk4_removal_of_gtkmenu/)  
82. Zugriff am Januar 1, 1970, [https://docs.gtk.org/gtk4/method.PopoverMenu.new\_from\_model.html](https://docs.gtk.org/gtk4/method.PopoverMenu.new_from_model.html)  
83. Gtk.Popover, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Popover.html](https://docs.gtk.org/gtk4/class.Popover.html)  
84. Gtk.Image \- GTK Documentation, Zugriff am Mai 14, 2025, [https://docs.gtk.org/gtk4/class.Image.html](https://docs.gtk.org/gtk4/class.Image.html)  
85. I want to create a window switcher for Linux. Is a Wayland client the correct approach?, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/linux/comments/1juzcto/i\_want\_to\_create\_a\_window\_switcher\_for\_linux\_is\_a/](https://www.reddit.com/r/linux/comments/1juzcto/i_want_to_create_a_window_switcher_for_linux_is_a/)  
86. wlr foreign toplevel management protocol \- Wayland Explorer, Zugriff am Mai 14, 2025, [https://wayland.app/protocols/wlr-foreign-toplevel-management-unstable-v1](https://wayland.app/protocols/wlr-foreign-toplevel-management-unstable-v1)  
87. Zugriff am Januar 1, 1970, [https://wayland.freedesktop.org/protocols/wlr-foreign-toplevel-management-unstable-v1.html](https://wayland.freedesktop.org/protocols/wlr-foreign-toplevel-management-unstable-v1.html)  
88. Zugriff am Januar 1, 1970, [https://docs.rs/smithay-client-toolkit/latest/smithay\_client\_toolkit/shell/foreign\_toplevel/index.html](https://docs.rs/smithay-client-toolkit/latest/smithay_client_toolkit/shell/foreign_toplevel/index.html)  
89. Zugriff am Januar 1, 1970, [https://docs.rs/wayland-protocols/latest/wayland\_protocols/wlr/unstable/foreign\_toplevel\_management/v1/client/index.html](https://docs.rs/wayland-protocols/latest/wayland_protocols/wlr/unstable/foreign_toplevel_management/v1/client/index.html)  
90. wayland\_protocols::ext::foreign\_toplevel\_list::v1::client::ext\_foreign\_toplevel\_list\_v1 \- Rust, Zugriff am Mai 14, 2025, [https://pop-os.github.io/libcosmic/wayland\_protocols/ext/foreign\_toplevel\_list/v1/client/ext\_foreign\_toplevel\_list\_v1/index.html](https://pop-os.github.io/libcosmic/wayland_protocols/ext/foreign_toplevel_list/v1/client/ext_foreign_toplevel_list_v1/index.html)  
91. Projects/GLib/GApplication/DBusAPI – GNOME Wiki Archive, Zugriff am Mai 14, 2025, [https://wiki.gnome.org/Projects/GLib/GApplication/DBusAPI](https://wiki.gnome.org/Projects/GLib/GApplication/DBusAPI)  
92. gtk/gtk/gtkapplication-dbus.c at main · GNOME/gtk \- GitHub, Zugriff am Mai 14, 2025, [https://github.com/GNOME/gtk/blob/master/gtk/gtkapplication-dbus.c](https://github.com/GNOME/gtk/blob/master/gtk/gtkapplication-dbus.c)  
93. D-Bus \- GNOME JavaScript, Zugriff am Mai 14, 2025, [https://gjs.guide/guides/gio/dbus.html](https://gjs.guide/guides/gio/dbus.html)  
94. Menus \- GNOME Developer Documentation, Zugriff am Mai 14, 2025, [https://developer.gnome.org/documentation/tutorials/menus.html](https://developer.gnome.org/documentation/tutorials/menus.html)  
95. Application menu / Actions \- GNOME Wiki, Zugriff am Mai 14, 2025, [https://wiki.gnome.org/ThreePointThree(2f)Features(2f)ApplicationMenu.html](https://wiki.gnome.org/ThreePointThree\(2f\)Features\(2f\)ApplicationMenu.html)  
96. Zugriff am Januar 1, 1970, [https://docs.gtk.org/gio/struct.DBusMenuModel.html](https://docs.gtk.org/gio/struct.DBusMenuModel.html)  
97. rust-zbus package : Ubuntu \- Launchpad, Zugriff am Mai 14, 2025, [https://launchpad.net/ubuntu/+source/rust-zbus](https://launchpad.net/ubuntu/+source/rust-zbus)  
98. Global Menu — helloSystem documentation, Zugriff am Mai 14, 2025, [https://hellosystem.github.io/docs/developer/menu.html](https://hellosystem.github.io/docs/developer/menu.html)  
99. Zugriff am Januar 1, 1970, [https://docs.rs/zbus/latest/zbus/struct.Proxy.html](https://docs.rs/zbus/latest/zbus/struct.Proxy.html)  
100. smithay\_client\_toolkit::foreign\_toplevel\_list \- Rust, Zugriff am Mai 14, 2025, [https://smithay.github.io/client-toolkit/smithay\_client\_toolkit/foreign\_toplevel\_list/index.html](https://smithay.github.io/client-toolkit/smithay_client_toolkit/foreign_toplevel_list/index.html)  
101. Zugriff am Januar 1, 1970, [https://github.com/smithay/smithay-client-toolkit/tree/master/examples](https://github.com/smithay/smithay-client-toolkit/tree/master/examples)  
102. StatusNotifierItem \- Freedesktop.org, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/)  
103. StatusNotifierItem \- Freedesktop.org, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierItem/](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierItem/)  
104. Where are status icons in Gnome 40/GTK 4? \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/gnome/comments/u9tq1k/where\_are\_status\_icons\_in\_gnome\_40gtk\_4/](https://www.reddit.com/r/gnome/comments/u9tq1k/where_are_status_icons_in_gnome_40gtk_4/)  
105. Introduce support for GTK 4 · Issue \#22 · AyatanaIndicators/libayatana-appindicator-glib, Zugriff am Mai 14, 2025, [https://github.com/AyatanaIndicators/libayatana-appindicator-glib/issues/22](https://github.com/AyatanaIndicators/libayatana-appindicator-glib/issues/22)  
106. StatusNotifierHost \- Freedesktop.org, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierHost/](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierHost/)  
107. StatusNotifierWatcher \- Freedesktop.org, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierWatcher/](https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/StatusNotifierWatcher/)  
108. Zugriff am Januar 1, 1970, [https://www.freedesktop.org/wiki/Specifications/StatusNotifierHost/](https://www.freedesktop.org/wiki/Specifications/StatusNotifierHost/)  
109. Writing a client proxy \- zbus: D-Bus for Rust made easy \- GitHub Pages, Zugriff am Mai 14, 2025, [https://dbus2.github.io/zbus/client.html](https://dbus2.github.io/zbus/client.html)  
110. NameOwnerChanged in zbus::fdo \- Rust \- openrr.github.io, Zugriff am Mai 14, 2025, [https://openrr.github.io/openrr/zbus/fdo/struct.NameOwnerChanged.html](https://openrr.github.io/openrr/zbus/fdo/struct.NameOwnerChanged.html)  
111. zbus::fdo \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/zbus/latest/zbus/fdo/index.html](https://docs.rs/zbus/latest/zbus/fdo/index.html)  
112. Zbus: catching signal with a destination \- help \- The Rust Programming Language Forum, Zugriff am Mai 14, 2025, [https://users.rust-lang.org/t/zbus-catching-signal-with-a-destination/125709](https://users.rust-lang.org/t/zbus-catching-signal-with-a-destination/125709)  
113. Zugriff am Januar 1, 1970, [https://docs.rs/zbus/latest/zbus/struct.Connection.html](https://docs.rs/zbus/latest/zbus/struct.Connection.html)  
114. Zugriff am Januar 1, 1970, [https://docs.rs/zbus/latest/zbus/fdo/struct.NameOwnerChangedArgs.html](https://docs.rs/zbus/latest/zbus/fdo/struct.NameOwnerChangedArgs.html)  
115. Signals \- GUI development with Rust and GTK 4, Zugriff am Mai 14, 2025, [https://gtk-rs.org/gtk4-rs/stable/latest/book/g\_object\_signals.html](https://gtk-rs.org/gtk4-rs/stable/latest/book/g_object_signals.html)  
116. icon-theme-spec \- Freedesktop.org, Zugriff am Mai 14, 2025, [https://www.freedesktop.org/wiki/Specifications/icon-theme-spec/](https://www.freedesktop.org/wiki/Specifications/icon-theme-spec/)  
117. Zugriff am Januar 1, 1970, [https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk\_pixbuf/index.html](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gdk_pixbuf/index.html)  
118. Zugriff am Januar 1, 1970, [https://docs.rs/gdk-pixbuf/latest/gdk\_pixbuf/struct.PixbufLoader.html](https://docs.rs/gdk-pixbuf/latest/gdk_pixbuf/struct.PixbufLoader.html)  
119. dbusmenu\_glib \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/dbusmenu-glib](https://docs.rs/dbusmenu-glib)  
120. System Tray Icons in GTK4? \- Platform \- GNOME Discourse, Zugriff am Mai 14, 2025, [https://discourse.gnome.org/t/system-tray-icons-in-gtk4/22615](https://discourse.gnome.org/t/system-tray-icons-in-gtk4/22615)  
121. Correct way of state management for gtk-rs? : r/rust \- Reddit, Zugriff am Mai 14, 2025, [https://www.reddit.com/r/rust/comments/1czodm3/correct\_way\_of\_state\_management\_for\_gtkrs/](https://www.reddit.com/r/rust/comments/1czodm3/correct_way_of_state_management_for_gtkrs/)  
122. Gtk4 \- how to deploy complex event handlers \- help \- Rust Users Forum, Zugriff am Mai 14, 2025, [https://users.rust-lang.org/t/gtk4-how-to-deploy-complex-event-handlers/88170](https://users.rust-lang.org/t/gtk4-how-to-deploy-complex-event-handlers/88170)  
123. once\_cell \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/once\_cell/latest/once\_cell/](https://docs.rs/once_cell/latest/once_cell/)