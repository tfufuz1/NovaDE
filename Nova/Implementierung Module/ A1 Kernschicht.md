# **Nova A1 Kernschicht Implementierungsleitfaden: Modul 1 \- Fundamentale Datentypen (core::types)**

## **1\. Modulübersicht: core::types**

### **1.1. Zweck und Verantwortlichkeit**

Dieses Modul, core::types, bildet das Fundament der Kernschicht (core) und somit des gesamten Systems. Seine primäre Verantwortung liegt in der Definition grundlegender, universell einsetzbarer Datentypen, die von allen anderen Schichten und Modulen der Desktop-Umgebung benötigt werden. Dazu gehören geometrische Primitive (wie Punkte, Größen, Rechtecke), Farbdarstellungen und allgemeine Enumerationen (wie Orientierungen).  
Die in diesem Modul definierten Typen sind bewusst einfach gehalten und repräsentieren reine Datenstrukturen ohne komplexe Geschäftslogik oder Abhängigkeiten zu höheren Schichten oder externen Systemen. Sie dienen als Bausteine für komplexere Operationen und Zustandsrepräsentationen in den Domänen-, System- und Benutzeroberflächenschichten.

### **1.2. Designphilosophie**

Das Design von core::types folgt den Prinzipien der Modularität, Wiederverwendbarkeit und minimalen Kopplung. Die Typen sind generisch gehalten (wo sinnvoll, z.B. bei geometrischen Primitiven), um Flexibilität für verschiedene numerische Darstellungen (z.B. i32 für Koordinaten, f32 für Skalierungsfaktoren) zu ermöglichen.  
Ein wesentlicher Aspekt ist die klare Trennung von Datenrepräsentation (in core::types) und Fehlerbehandlung. Während dieses Modul die Datenstrukturen definiert, werden die spezifischen Fehler, die bei Operationen mit diesen Typen auftreten können (z.B. durch ungültige Werte), in den Modulen definiert, die diese Operationen durchführen (typischerweise in core::errors oder modulspezifischen Fehler-Enums höherer Schichten).

### **1.3. Zusammenspiel mit Fehlerbehandlung**

Obwohl core::types selbst keine Error-Typen definiert, ist das Design der hier enthaltenen Typen entscheidend für eine robuste und konsistente Fehlerbehandlungsstrategie im gesamten Projekt. Die übergeordnete Richtlinie sieht die Verwendung des thiserror-Crates vor, um spezifische Fehler-Enums pro Modul zu definieren. Dies ermöglicht eine granulare Fehlerbehandlung, ohne die Komplexität übermäßig zu erhöhen.  
Die Typen in core::types unterstützen diese Strategie, indem sie:

1. **Standard-Traits implementieren:** Alle Typen implementieren grundlegende Traits wie Debug und Display. Dies ist essenziell, damit Instanzen dieser Typen effektiv in Fehlermeldungen und Log-Ausgaben eingebettet werden können, die von höheren Schichten unter Verwendung von thiserror generiert werden. Eine gute Fehlerdarstellung ist entscheidend für die Fehlersuche und das Verständnis von Problemen im Laufzeitbetrieb.  
2. **Invarianten dokumentieren:** Für Typen wie Rect\<T\> existieren logische Invarianten (z.B. nicht-negative Breite und Höhe). Diese Invarianten werden klar dokumentiert.  
3. **Validierung ermöglichen:** Wo sinnvoll, werden Methoden zur Überprüfung der Gültigkeit bereitgestellt (z.B. Rect::is\_valid()). Diese Methoden erlauben es aufrufendem Code in höheren Schichten, Zustände zu überprüfen, *bevor* Operationen ausgeführt werden, die fehlschlagen könnten.  
4. **Keine Panics in Kernfunktionen:** Konstruktoren und einfache Zugriffsmethoden in core::types lösen keine Panics aus und geben keine Result-Typen zurück, um die API auf dieser fundamentalen Ebene einfach und vorhersagbar zu halten. Die Verantwortung für die Handhabung potenziell ungültiger Zustände (z.B. ein Rect mit negativer Breite, das an eine Rendering-Funktion übergeben wird) liegt bei den konsumierenden Funktionen, die dann die definierten Fehlerpfade (mittels Result\<T, E\> 3 und den thiserror-basierten E-Typen) nutzen.

Diese Designentscheidungen stellen sicher, dass die fundamentalen Typen nahtlos in das übergeordnete Fehlerbehandlungskonzept integriert werden können, ohne selbst die Komplexität der Fehlerdefinition tragen zu müssen. Die gewählte Fehlerstrategie mit thiserror pro Modul wird als ausreichend für die Bedürfnisse der Kernschicht erachtet, auch wenn alternative Ansätze wie snafu für komplexere Szenarien existieren, in denen z.B. die Unterscheidung von Fehlern aus derselben Quelle kritisch ist. Für die Kernschicht wird die Einfachheit und Direktheit von thiserror bevorzugt.

### **1.4. Modulabhängigkeiten**

Dieses Modul ist darauf ausgelegt, minimale externe Abhängigkeiten zu haben, um seine grundlegende Natur und breite Anwendbarkeit zu gewährleisten.

* **Erlaubte Abhängigkeiten:**  
  * std (Rust Standardbibliothek)  
* **Optionale Abhängigkeiten (derzeit nicht verwendet):**  
  * num-traits: Nur hinzufügen, falls generische numerische Operationen benötigt werden, die über std::ops hinausgehen.  
  * serde (mit derive-Feature): Nur hinzufügen, wenn Serialisierung/Deserialisierung dieser Basistypen *direkt auf dieser Ebene* zwingend erforderlich ist (z.B. für Konfigurationsdateien, die diese Typen direkt verwenden). Aktuell wird davon ausgegangen, dass Serialisierungslogik in höheren Schichten implementiert wird, um unnötige Abhängigkeiten zu vermeiden.

### **1.5. Ziel-Dateistruktur**

Die Implementierung dieses Moduls erfolgt innerhalb des core-Crates mit folgender Verzeichnisstruktur:

src/  
└── core/  
    ├── Cargo.toml         \# (Definiert das 'core' Crate)  
    └── src/  
        ├── lib.rs             \# (Deklariert Kernmodule: pub mod types; pub mod errors;...)  
        └── types/  
            ├── mod.rs         \# (Deklariert und re-exportiert Typen: pub mod geometry; pub mod color;...)  
            ├── geometry.rs    \# (Enthält Point\<T\>, Size\<T\>, Rect\<T\>)  
            ├── color.rs       \# (Enthält Color)  
            └── enums.rs       \# (Enthält Orientation, etc.)

## **2\. Spezifikation: Geometrische Primitive (geometry.rs)**

Diese Datei definiert grundlegende 2D-Geometrietypen, die für Layout, Positionierung und Rendering unerlässlich sind.

### **2.1. Struct: Point\<T\>**

* **2.1.1. Definition und Zweck:** Repräsentiert einen Punkt im 2D-Raum mit x- und y-Koordinaten. Generisch über den Typ T.  
* **2.1.2. Felder:**  
  * pub x: T  
  * pub y: T  
* **2.1.3. Assoziierte Konstanten:**  
  * pub const ZERO\_I32: Point\<i32\> \= Point { x: 0, y: 0 };  
  * pub const ZERO\_U32: Point\<u32\> \= Point { x: 0, y: 0 };  
  * pub const ZERO\_F32: Point\<f32\> \= Point { x: 0.0, y: 0.0 };  
  * pub const ZERO\_F64: Point\<f64\> \= Point { x: 0.0, y: 0.0 };  
* **2.1.4. Methoden:**  
  * pub const fn new(x: T, y: T) \-\> Self  
    * Erstellt einen neuen Punkt.  
  * pub fn distance\_squared(\&self, other: \&Point\<T\>) \-\> T  
    * Berechnet das Quadrat der euklidischen Distanz.  
    * *Constraints:* T:Copy+std::ops::Add\<Output=T\>+std::ops::Sub\<Output=T\>+std::ops::Mul\<Output=T\>  
  * pub fn distance(\&self, other: \&Point\<T\>) \-\> T  
    * Berechnet die euklidische Distanz.  
    * *Constraints:* T:Copy+std::ops::Add\<Output=T\>+std::ops::Sub\<Output=T\>+std::ops::Mul\<Output=T\>+numt​raits::Float (Implementierung nur für Float-Typen sinnvoll oder über sqrt-Funktion). Vorerst nur für f32,f64 implementieren.  
  * pub fn manhattan\_distance(\&self, other: \&Point\<T\>) \-\> T  
    * Berechnet die Manhattan-Distanz (∣x1​−x2​∣+∣y1​−y2​∣).  
    * *Constraints:* T:Copy+std::ops::Add\<Output=T\>+std::ops::Sub\<Output=T\>+numt​raits::Signed (Benötigt abs()).  
* **2.1.5. Trait Implementierungen:**  
  * \#  
    * *Bedingung:* T muss die jeweiligen Traits ebenfalls implementieren. Default setzt x und y auf T::default().  
  * impl\<T: Send \+ 'static\> Send for Point\<T\> {}  
  * impl\<T: Sync \+ 'static\> Sync for Point\<T\> {}  
  * impl\<T: std::ops::Add\<Output \= T\>\> std::ops::Add for Point\<T\>  
  * impl\<T: std::ops::Sub\<Output \= T\>\> std::ops::Sub for Point\<T\>  
* **2.1.6. Generische Constraints (Basis):** T:Copy+Debug+PartialEq+Default+Send+Sync+′static. Weitere Constraints werden pro Methode spezifiziert.

### **2.2. Struct: Size\<T\>**

* **2.2.1. Definition und Zweck:** Repräsentiert eine 2D-Dimension (Breite und Höhe). Generisch über den Typ T.  
* **2.2.2. Felder:**  
  * pub width: T  
  * pub height: T  
* **2.2.3. Assoziierte Konstanten:**  
  * pub const ZERO\_I32: Size\<i32\> \= Size { width: 0, height: 0 };  
  * pub const ZERO\_U32: Size\<u32\> \= Size { width: 0, height: 0 };  
  * pub const ZERO\_F32: Size\<f32\> \= Size { width: 0.0, height: 0.0 };  
  * pub const ZERO\_F64: Size\<f64\> \= Size { width: 0.0, height: 0.0 };  
* **2.2.4. Methoden:**  
  * pub const fn new(width: T, height: T) \-\> Self  
    * Erstellt eine neue Größe.  
  * pub fn area(\&self) \-\> T  
    * Berechnet die Fläche (width×height).  
    * *Constraints:* T:Copy+std::ops::Mul\<Output=T\>  
  * pub fn is\_empty(\&self) \-\> bool  
    * Prüft, ob Breite oder Höhe null ist.  
    * *Constraints:* T:PartialEq+numt​raits::Zero  
  * pub fn is\_valid(\&self) \-\> bool  
    * Prüft, ob Breite und Höhe nicht-negativ sind. Nützlich für Typen wie i32.  
    * *Constraints:* T:PartialOrd+numt​raits::Zero  
* **2.2.5. Trait Implementierungen:**  
  * \#  
    * *Bedingung:* T muss die jeweiligen Traits ebenfalls implementieren. Default setzt width und height auf T::default().  
  * impl\<T: Send \+ 'static\> Send for Size\<T\> {}  
  * impl\<T: Sync \+ 'static\> Sync for Size\<T\> {}  
* **2.2.6. Generische Constraints (Basis):** T:Copy+Debug+PartialEq+Default+Send+Sync+′static. Weitere Constraints werden pro Methode spezifiziert. Die Invariante nicht-negativer Dimensionen wird durch is\_valid prüfbar gemacht, aber nicht durch den Typ erzwungen.

### **2.3. Struct: Rect\<T\>**

* **2.3.1. Definition und Zweck:** Repräsentiert ein 2D-Rechteck, definiert durch einen Ursprungspunkt (oben-links) und eine Größe. Generisch über den Typ T.  
* **2.3.2. Felder:**  
  * pub origin: Point\<T\>  
  * pub size: Size\<T\>  
* **2.3.3. Assoziierte Konstanten:**  
  * pub const ZERO\_I32: Rect\<i32\> \= Rect { origin: Point::ZERO\_I32, size: Size::ZERO\_I32 };  
  * pub const ZERO\_U32: Rect\<u32\> \= Rect { origin: Point::ZERO\_U32, size: Size::ZERO\_U32 };  
  * pub const ZERO\_F32: Rect\<f32\> \= Rect { origin: Point::ZERO\_F32, size: Size::ZERO\_F32 };  
  * pub const ZERO\_F64: Rect\<f64\> \= Rect { origin: Point::ZERO\_F64, size: Size::ZERO\_F64 };  
* **2.3.4. Methoden:**  
  * pub const fn new(origin: Point\<T\>, size: Size\<T\>) \-\> Self  
  * pub fn from\_coords(x: T, y: T, width: T, height: T) \-\> Self  
    * *Constraints:* T muss die Constraints von Point::new und Size::new erfüllen.  
  * pub fn x(\&self) \-\> T (*Constraints:* T:Copy)  
  * pub fn y(\&self) \-\> T (*Constraints:* T:Copy)  
  * pub fn width(\&self) \-\> T (*Constraints:* T:Copy)  
  * pub fn height(\&self) \-\> T (*Constraints:* T:Copy)  
  * pub fn top(\&self) \-\> T (Alias für y, *Constraints:* T:Copy)  
  * pub fn left(\&self) \-\> T (Alias für x, *Constraints:* T:Copy)  
  * pub fn bottom(\&self) \-\> T (y+height, *Constraints:* T:Copy+std::ops::Add\<Output=T\>)  
  * pub fn right(\&self) \-\> T (x+width, *Constraints:* T:Copy+std::ops::Add\<Output=T\>)  
  * pub fn center(\&self) \-\> Point\<T\>  
    * Berechnet den Mittelpunkt.  
    * *Constraints:* T:Copy+std::ops::Add\<Output=T\>+std::ops::Div\<Output=T\>+numt​raits::FromPrimitive (Benötigt Division durch 2).  
  * pub fn contains\_point(\&self, point: \&Point\<T\>) \-\> bool  
    * Prüft, ob der Punkt innerhalb des Rechtecks liegt (Grenzen inklusiv für top/left, exklusiv für bottom/right).  
    * *Constraints:* T:Copy+PartialOrd+std::ops::Add\<Output=T\>  
  * pub fn intersects(\&self, other: \&Rect\<T\>) \-\> bool  
    * Prüft, ob sich dieses Rechteck mit einem anderen überschneidet.  
    * *Constraints:* T:Copy+PartialOrd+std::ops::Add\<Output=T\>  
  * pub fn intersection(\&self, other: \&Rect\<T\>) \-\> Option\<Rect\<T\>\>  
    * Berechnet das Schnittrechteck. Gibt None zurück, wenn keine Überschneidung vorliegt.  
    * *Constraints:* T:Copy+Ord+std::ops::Add\<Output=T\>+std::ops::Sub\<Output=T\>+numt​raits::Zero  
  * pub fn union(\&self, other: \&Rect\<T\>) \-\> Rect\<T\>  
    * Berechnet das umschließende Rechteck beider Rechtecke.  
    * *Constraints:* T:Copy+Ord+std::ops::Add\<Output=T\>+std::ops::Sub\<Output=T\>  
  * pub fn translated(\&self, dx: T, dy: T) \-\> Rect\<T\>  
    * Verschiebt das Rechteck um (dx,dy).  
    * *Constraints:* T:Copy+std::ops::Add\<Output=T\>  
  * pub fn scaled(\&self, sx: T, sy: T) \-\> Rect\<T\>  
    * Skaliert das Rechteck relativ zum Ursprung (0,0). Beachtet, dass dies Ursprung und Größe skaliert.  
    * *Constraints:* T:Copy+std::ops::Mul\<Output=T\>  
  * pub fn is\_valid(\&self) \-\> bool  
    * Prüft, ob size.is\_valid() wahr ist.  
    * *Constraints:* T:PartialOrd+numt​raits::Zero  
* **2.3.5. Trait Implementierungen:**  
  * \#  
    * *Bedingung:* T muss die jeweiligen Traits ebenfalls implementieren. Default verwendet Point::default() und Size::default().  
  * impl\<T: Send \+ 'static\> Send for Rect\<T\> {}  
  * impl\<T: Sync \+ 'static\> Sync for Rect\<T\> {}  
* **2.3.6. Generische Constraints (Basis):** T:Copy+Debug+PartialEq+Default+Send+Sync+′static. Weitere Constraints werden pro Methode spezifiziert.  
* **2.3.7. Invarianten und Validierung (Verbindung zur Fehlerbehandlung):**  
  * **Invariante:** Logisch sollten width und height der size-Komponente nicht-negativ sein.  
  * **Kontext:** Die Verwendung von vorzeichenbehafteten Typen wie i32 für Koordinaten ist üblich, erlaubt aber technisch negative Dimensionen. Eine Erzwingung nicht-negativer Dimensionen auf Typebene (z.B. durch u32) wäre zu restriktiv für Koordinatensysteme.  
  * **Konsequenz:** Die Flexibilität, Rect\<i32\> zu verwenden, verlagert die Verantwortung für die Validierung auf die Nutzer des Rect-Typs. Funktionen in höheren Schichten (z.B. Layout-Algorithmen, Rendering-Engines), die ein Rect konsumieren, müssen potenziell ungültige Rechtecke (mit negativer Breite oder Höhe) behandeln. Solche Fälle stellen Laufzeitfehler dar, die über das etablierte Fehlerbehandlungssystem (basierend auf Result\<T, E\> und thiserror-definierten E-Typen) signalisiert werden müssen.  
  * **Implementierung in core::types:** Das Modul erzwingt die Invariante nicht zur Compilezeit oder in Konstruktoren. Stattdessen wird die Methode pub fn is\_valid(\&self) \-\> bool bereitgestellt. Nutzer von Rect\<T\> (insbesondere mit T=i32) *sollten* diese Methode aufrufen, um die Gültigkeit sicherzustellen, bevor Operationen durchgeführt werden, die eine positive Breite und Höhe voraussetzen. Die Dokumentation des Rect-Typs muss explizit auf diese Invariante und die Notwendigkeit der Validierung durch den Aufrufer hinweisen. Die Verantwortung für das *Melden* eines Fehlers bei Verwendung eines ungültigen Rect liegt beim Aufrufer, der dafür die Fehlerinfrastruktur (z.B. core::errors oder modulspezifische Fehler) nutzt.

## **3\. Spezifikation: Farbdarstellung (color.rs)**

Diese Datei definiert einen Standard-Farbtyp für die Verwendung im gesamten System.

### **3.1. Struct: Color (RGBA)**

* **3.1.1. Definition und Zweck:** Repräsentiert eine Farbe mit Rot-, Grün-, Blau- und Alpha-Komponenten. Verwendet f32-Komponenten im Bereich \[0.0,1.0\] für hohe Präzision und Flexibilität bei Farboperationen wie Mischen und Transformationen.  
* **3.1.2. Felder:**  
  * pub r: f32 (Rotkomponente, 0.0 bis 1.0)  
  * pub g: f32 (Grünkomponente, 0.0 bis 1.0)  
  * pub b: f32 (Blaukomponente, 0.0 bis 1.0)  
  * pub a: f32 (Alphakomponente, 0.0=transparent bis 1.0=opak)  
* **3.1.3. Assoziierte Konstanten:**  
  * pub const TRANSPARENT: Color \= Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };  
  * pub const BLACK: Color \= Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };  
  * pub const WHITE: Color \= Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };  
  * pub const RED: Color \= Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };  
  * pub const GREEN: Color \= Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };  
  * pub const BLUE: Color \= Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };  
  * *(Weitere Standardfarben nach Bedarf hinzufügen)*  
* **3.1.4. Methoden:**  
  * pub const fn new(r: f32, g: f32, b: f32, a: f32) \-\> Self  
    * Erstellt eine neue Farbe. Werte außerhalb \[0.0,1.0\] werden nicht automatisch geklemmt, dies liegt in der Verantwortung des Aufrufers oder nachfolgender Operationen. debug\_assert\! kann zur Laufzeitprüfung in Debug-Builds verwendet werden.  
  * pub fn from\_rgba8(r: u8, g: u8, b: u8, a: u8) \-\> Self  
    * Konvertiert von 8-Bit-Ganzzahlkomponenten (0−255) zu f32 (0.0−1.0). value/255.0.  
  * pub fn to\_rgba8(\&self) \-\> (u8, u8, u8, u8)  
    * Konvertiert von f32 zu 8-Bit-Ganzzahlkomponenten. Klemmt Werte auf \[0.0,1.0\] und skaliert dann auf $$. (value.clamp(0.0,1.0)∗255.0).round()asu8.  
  * pub fn with\_alpha(\&self, alpha: f32) \-\> Self  
    * Erstellt eine neue Farbe mit dem angegebenen Alpha-Wert, wobei RGB beibehalten wird. Klemmt Alpha auf \[0.0,1.0\].  
  * pub fn blend(\&self, background: \&Color) \-\> Color  
    * Führt Alpha-Blending ("source-over") dieser Farbe über einer Hintergrundfarbe durch. Formel: Cout​=Cfg​×αfg​+Cbg​×αbg​×(1−αfg​). αout​=αfg​+αbg​×(1−αfg​). Annahme: Farben sind nicht vormultipliziert.  
  * pub fn lighten(\&self, amount: f32) \-\> Color  
    * Hellt die Farbe um einen Faktor amount auf (z.B. durch lineare Interpolation zu Weiß). Klemmt das Ergebnis auf gültige Farbwerte. amount im Bereich \[0.0,1.0\].  
  * pub fn darken(\&self, amount: f32) \-\> Color  
    * Dunkelt die Farbe um einen Faktor amount ab (z.B. durch lineare Interpolation zu Schwarz). Klemmt das Ergebnis. amount im Bereich \[0.0,1.0\].  
* **3.1.5. Trait Implementierungen:**  
  * \#  
    * PartialEq: Verwendet den Standard-Float-Vergleich. Für präzisere Vergleiche könnten benutzerdefinierte Implementierungen mit Epsilon erforderlich sein, dies wird jedoch für die Kernschicht als unnötige Komplexität betrachtet.  
    * Default: Implementiert Default manuell, um Color::TRANSPARENT zurückzugeben.  
  * impl Send for Color {}  
  * impl Sync for Color {}

## **4\. Spezifikation: Allgemeine Enumerationen (enums.rs)**

Diese Datei enthält häufig verwendete, einfache Enumerationen.

### **4.1. Enum: Orientation**

* **4.1.1. Definition und Zweck:** Repräsentiert eine horizontale oder vertikale Ausrichtung, häufig verwendet in UI-Layouts und Widgets.  
* **4.1.2. Varianten:**  
  * Horizontal  
  * Vertical  
* **4.1.3. Methoden:**  
  * pub fn toggle(\&self) \-\> Self  
    * Gibt die jeweils andere Orientierung zurück (Horizontal \-\> Vertical, Vertical \-\> Horizontal).  
* **4.1.4. Trait Implementierungen:**  
  * \#  
  * impl Default for Orientation { fn default() \-\> Self { Orientation::Horizontal } } (Standard ist Horizontal).  
  * impl Send for Orientation {}  
  * impl Sync for Orientation {}

## **5\. Zusammenfassung: Standard Trait Implementierungen**

Die folgende Tabelle gibt einen Überblick über die Implementierung gängiger Standard-Traits für die in diesem Modul definierten Typen. Dies dient als schnelle Referenz für Entwickler.

| Typ | Debug | Clone | Copy | PartialEq | Eq | Default | Hash | Send | Sync |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| Point\<T\> | Ja | Ja | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) |
| Size\<T\> | Ja | Ja | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) |
| Rect\<T\> | Ja | Ja | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) | Ja (wenn T) |
| Color | Ja | Ja | Ja | Ja | Nein | Ja | Nein | Ja | Ja |
| Orientation | Ja | Ja | Ja | Ja | Ja | Ja | Ja | Ja | Ja |

Anmerkungen:  
Eq und Hash sind aufgrund von Präzisionsproblemen generell nicht für Fließkommazahlen geeignet.  
Default::default() ergibt Color::TRANSPARENT.  
Default::default() ergibt Orientation::Horizontal.

## **6\. Schritt-für-Schritt Implementierungsplan**

Die Implementierung des core::types-Moduls folgt diesen Schritten:

* **6.1. Setup: Verzeichnis- und Dateierstellung:**  
  * Sicherstellen, dass das core-Crate existiert (ggf. cargo new core \--lib ausführen).  
  * Erstellen des Verzeichnisses src/core/src/types.  
  * Erstellen der Dateien:  
    * src/core/src/types/mod.rs  
    * src/core/src/types/geometry.rs  
    * src/core/src/types/color.rs  
    * src/core/src/types/enums.rs  
* **6.2. Implementierung geometry.rs: Point\<T\>, Size\<T\>, Rect\<T\>:**  
  * Definieren der Point\<T\>-Struktur mit Feldern x, y. Hinzufügen der spezifizierten generischen Basis-Constraints (T:Copy+Debug+PartialEq+Default+Send+Sync+′static). Implementieren von new, Konstanten (ZERO\_I32 etc.), Methoden (distance\_squared, distance (für Floats), manhattan\_distance) mit ihren spezifischen Constraints und Ableiten/Implementieren der spezifizierten Traits (Add, Sub).  
  * Definieren der Size\<T\>-Struktur mit Feldern width, height. Hinzufügen der Basis-Constraints. Implementieren von new, Konstanten (ZERO\_I32 etc.), Methoden (area, is\_empty, is\_valid) mit ihren Constraints und Ableiten/Implementieren der Traits.  
  * Definieren der Rect\<T\>-Struktur mit Feldern origin, size. Hinzufügen der Basis-Constraints. Implementieren von new, from\_coords, Konstanten (ZERO\_I32 etc.), Zugriffsmethoden (x, y, width, height, top, left, bottom, right), geometrischen Methoden (center, contains\_point, intersects, intersection, union, translated, scaled), Validierungsmethode (is\_valid) mit ihren Constraints und Ableiten/Implementieren der Traits.  
  * Hinzufügen notwendiger use-Anweisungen (z.B. std::ops, num\_traits).  
* **6.3. Implementierung color.rs: Color:**  
  * Definieren der Color-Struktur mit Feldern r, g, b, a (alle f32).  
  * Implementieren von new, Konstanten (TRANSPARENT, BLACK, WHITE, etc.), Konvertierungsmethoden (from\_rgba8, to\_rgba8), Hilfsmethoden (with\_alpha, blend, lighten, darken) und Ableiten/Implementieren der Traits (Default manuell).  
* **6.4. Implementierung enums.rs: Orientation:**  
  * Definieren des Orientation-Enums mit Varianten Horizontal, Vertical.  
  * Implementieren der toggle-Methode.  
  * Ableiten/Implementieren der spezifizierten Traits (Default manuell).  
* **6.5. Implementierung Moduldeklaration (mod.rs):**  
  * In src/core/src/types/mod.rs:  
    Rust  
    // src/core/src/types/mod.rs  
    pub mod color;  
    pub mod enums;  
    pub mod geometry;

    // Re-exportiere die primären Typen für einfacheren Zugriff  
    pub use color::Color;  
    pub use enums::Orientation;  
    pub use geometry::{Point, Rect, Size};

  * In src/core/src/lib.rs:  
    Rust  
    // src/core/src/lib.rs  
    // Deklariere das types-Modul  
    pub mod types;

    // Deklariere andere Kernmodule (werden später hinzugefügt)  
    // pub mod errors;  
    // pub mod logging;  
    // pub mod config;  
    // pub mod utils;

* **6.6. Unit-Testing Anforderungen:**  
  * Erstellen eines \#\[cfg(test)\]-Moduls innerhalb jeder Implementierungsdatei (geometry.rs, color.rs, enums.rs).  
  * Schreiben von Unit-Tests, die Folgendes abdecken:  
    * Konstruktorfunktionen (new, from\_coords, from\_rgba8).  
    * Konstantenwerte (deren Eigenschaften überprüfen).  
    * Methodenlogik (z.B. distance\_squared, area, is\_empty, bottom, right, contains\_point, intersects, intersection, union, toggle, blend). Testen von Grenzfällen (Nullwerte, überlappende/nicht überlappende Rechtecke, identische Punkte, Farbblending mit transparent/opak).  
    * Trait-Implementierungen (insbesondere Default, PartialEq, Add/Sub, wo zutreffend).  
    * Invariantenprüfungen (z.B. is\_valid für Rect und Size testen).  
  * Anstreben einer hohen Testabdeckung für diesen fundamentalen Code.  
* **6.7. Dokumentationsanforderungen (rustdoc):**  
  * Hinzufügen von ///-Dokumentationskommentaren zu *allen* öffentlichen Elementen: Module (mod.rs-Dateien), Structs, Enums, Felder, Konstanten, Methoden, Typ-Aliase.  
  * Modul-Level-Kommentare sollen den Zweck des Moduls erklären (geometry.rs, color.rs, etc.).  
  * Typ-Level-Kommentare sollen den Zweck und die Invarianten der Struktur/des Enums erklären (besonders wichtig für Rect-Invarianten).  
  * Feld-Level-Kommentare sollen die Bedeutung des Feldes erklären (z.B. Wertebereich für Color-Komponenten).  
  * Methoden-Level-Kommentare sollen erklären, was die Methode tut, ihre Parameter, Rückgabewerte, mögliche Panics (sollten hier idealerweise keine auftreten, außer bei unwrap/expect in Tests), relevante Vor-/Nachbedingungen oder verwendete Algorithmen (z.B. Alpha-Blending-Formel). \# Examples-Abschnitte verwenden, wo sinnvoll.  
  * Strikte Einhaltung der Rust API Guidelines für Dokumentation.  
  * Ausführen von cargo doc \--open zur Überprüfung der generierten Dokumentation.

## **7\. Schlussfolgerung**

Dieses Dokument spezifiziert das Modul core::types, welches die grundlegendsten Datentypen für die neue Linux-Desktop-Umgebung bereitstellt. Die definierten Typen (Point\<T\>, Size\<T\>, Rect\<T\>, Color, Orientation) sind mit Fokus auf Einfachheit, Wiederverwendbarkeit und minimalen Abhängigkeiten entworfen. Besonderes Augenmerk wurde auf die klare Trennung zwischen Datenrepräsentation und Fehlerbehandlung gelegt, wobei die Typen so gestaltet sind, dass sie die übergeordnete, auf thiserror basierende Fehlerstrategie des Projekts unterstützen, ohne selbst Fehlerdefinitionen zu enthalten. Die Bereitstellung von Validierungsfunktionen wie Rect::is\_valid und die klare Dokumentation von Invarianten sind entscheidend, um Robustheit in den konsumierenden Schichten zu ermöglichen. Der detaillierte Implementierungsplan inklusive Test- und Dokumentationsanforderungen stellt sicher, dass dieses fundamentale Modul mit hoher Qualität und Konsistenz entwickelt werden kann.

#### **Referenzen**

1. Error handling \- good/best practices : r/rust \- Reddit, Zugriff am Mai 13, 2025, [https://www.reddit.com/r/rust/comments/1bb7dco/error\_handling\_goodbest\_practices/](https://www.reddit.com/r/rust/comments/1bb7dco/error_handling_goodbest_practices/)  
2. Error in std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/trait.Error.html](https://doc.rust-lang.org/std/error/trait.Error.html)  
3. Error Handling for Large Rust Projects \- Best Practice in GreptimeDB, Zugriff am Mai 13, 2025, [https://greptime.com/blogs/2024-05-07-error-rust](https://greptime.com/blogs/2024-05-07-error-rust)  
4. std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/index.html](https://doc.rust-lang.org/std/error/index.html)