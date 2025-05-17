# **Domänenschicht: Theming-Engine – Ultra-Feinspezifikation (Teil 1/4)**

## **1\. Einleitung zum Modul domain::theming**

Das Modul domain::theming ist eine Kernkomponente der Domänenschicht und trägt die Verantwortung für die gesamte Logik des Erscheinungsbilds (Theming) der Desktop-Umgebung. Seine Hauptaufgabe besteht darin, Design-Tokens zu verwalten, Theme-Definitionen zu interpretieren, Benutzereinstellungen für das Theming zu berücksichtigen und den finalen, aufgelösten Theme-Zustand für die Benutzeroberflächenschicht bereitzustellen. Dieses Modul ermöglicht dynamische Theme-Wechsel zur Laufzeit, einschließlich Änderungen des Farbschemas (Hell/Dunkel) und der Akzentfarben, basierend auf einem robusten, Token-basierten System. Es ist so konzipiert, dass es unabhängig von spezifischen UI-Toolkits oder Systemdetails agiert und eine klare Trennung zwischen der Logik des Erscheinungsbilds und dessen Darstellung gewährleistet. Diese Spezifikation dient als direkter Implementierungsleitfaden für Entwickler.

## **2\. Datenstrukturen (domain::theming::types)**

Die folgenden Datenstrukturen definieren die Entitäten und Wertobjekte, die für die Verwaltung und Anwendung von Themes und Design-Tokens notwendig sind. Sie sind für die Serialisierung und Deserialisierung mittels serde vorbereitet, um das Laden von Konfigurationen und Definitionen aus Dateien (z.B. JSON) zu ermöglichen.

### **2.1. Token-bezogene Datenstrukturen**

Diese Strukturen repräsentieren einzelne Design-Tokens und deren Werte.

* TokenIdentifier (Wertobjekt):  
  Ein eindeutiger, hierarchischer Bezeichner für ein Design-Token (z.B. "color.background.primary", "font.size.default"). Die hierarchische Struktur erleichtert die Organisation und das Verständnis der Tokens.  
  Rust  
  \#  
  pub struct TokenIdentifier(String);

  impl TokenIdentifier {  
      pub fn new(id: impl Into\<String\>) \-\> Self {  
          Self(id.into())  
      }  
      pub fn as\_str(\&self) \-\> \&str {  
          \&self.0  
      }  
  }

  impl std::fmt::Display for TokenIdentifier {  
      fn fmt(\&self, f: \&mut std::fmt::Formatter\<'\_\>) \-\> std::fmt::Result {  
          write\!(f, "{}", self.0)  
      }  
  }

* TokenValue (Enum):  
  Repräsentiert die möglichen Wertetypen eines Design-Tokens. Die String-Werte für Farben, Dimensionen etc. sind so gestaltet, dass sie direkt CSS-kompatibel sind. Die Variante Reference ermöglicht die Erstellung von Alias-Tokens, die auf andere Tokens verweisen, was die Wiederverwendbarkeit und Konsistenz fördert.  
  Rust  
  \#  
  \#\[serde(rename\_all \= "kebab-case")\]  
  pub enum TokenValue {  
      Color(String),      // z.B., "\#FF0000", "rgba(255,0,0,0.5)", "transparent"  
      Dimension(String),  // z.B., "16px", "2rem", "100%"  
      FontSize(String),   // z.B., "12pt", "1.5em"  
      FontFamily(String), // z.B., "Inter, sans-serif"  
      FontWeight(String), // z.B., "normal", "bold", "700"  
      LineHeight(String), // z.B., "1.5", "150%"  
      LetterSpacing(String),// z.B., "0.5px", "0.05em"  
      Border(String),     // z.B., "1px solid \#CCCCCC"  
      Shadow(String),     // z.B., "2px 2px 5px rgba(0,0,0,0.3)"  
      Radius(String),     // z.B., "4px", "50%"  
      Spacing(String),    // z.B., "8px" (generische Abstände für padding, margin)  
      ZIndex(i32),  
      Opacity(f64),       // 0.0 bis 1.0  
      Text(String),       // Für beliebige String-Werte  
      Reference(TokenIdentifier), // Alias zu einem anderen Token  
  }

* RawToken (Struct):  
  Repräsentiert ein einzelnes Design-Token, wie es typischerweise aus einer Konfigurationsdatei (z.B. JSON) geladen wird. Enthält den Identifikator, den Wert und optionale Metadaten wie Beschreibung und Gruppierung.  
  Rust  
  \#  
  pub struct RawToken {  
      pub id: TokenIdentifier,  
      pub value: TokenValue,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub description: Option\<String\>,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub group: Option\<String\>, // z.B., "colors", "spacing", "typography"  
  }

* TokenSet (Typalias):  
  Eine Sammlung von RawTokens, die für eine effiziente Suche und Verwaltung als HashMap implementiert ist, wobei der TokenIdentifier als Schlüssel dient.  
  Rust  
  pub type TokenSet \= std::collections::HashMap\<TokenIdentifier, RawToken\>;

### **2.2. Theme-Definitionsstrukturen**

Diese Strukturen definieren ein vollständiges Theme, seine Varianten (z.B. Hell/Dunkel) und unterstützte Anpassungen.

* ThemeIdentifier (Wertobjekt):  
  Ein eindeutiger Bezeichner für ein Theme (z.B. "adwaita-ng", "material-you-like").  
  Rust  
  \#  
  pub struct ThemeIdentifier(String);

  impl ThemeIdentifier {  
      pub fn new(id: impl Into\<String\>) \-\> Self {  
          Self(id.into())  
      }  
      pub fn as\_str(\&self) \-\> \&str {  
          \&self.0  
      }  
  }  
  impl std::fmt::Display for ThemeIdentifier {  
      fn fmt(\&self, f: \&mut std::fmt::Formatter\<'\_\>) \-\> std::fmt::Result {  
          write\!(f, "{}", self.0)  
      }  
  }

* ColorSchemeType (Enum):  
  Definiert die grundlegenden Farbschemata, die ein Theme unterstützen kann.  
  Rust  
  \#  
  pub enum ColorSchemeType {  
      Light,  
      Dark,  
  }

* AccentColor (Struct / Wertobjekt):  
  Repräsentiert eine Akzentfarbe, die entweder einen vordefinierten Namen oder einen direkten Farbwert haben kann.  
  Rust  
  \#  
  pub struct AccentColor {  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub name: Option\<String\>, // z.B., "Blue", "ForestGreen"  
      pub value: String,        // z.B., "\#3498db" (tatsächlicher CSS-Farbwert)  
  }

* ThemeVariantDefinition (Struct):  
  Definiert die spezifischen Token-Werte oder Überschreibungen für eine bestimmte Variante eines Themes (z.B. das Dunkel-Schema). Der TokenSet hier enthält nur die Tokens, die sich von den base\_tokens des Themes unterscheiden oder spezifisch für diese Variante sind.  
  Rust  
  \#  
  pub struct ThemeVariantDefinition {  
      pub applies\_to\_scheme: ColorSchemeType,  
      pub tokens: TokenSet, // Token-Überschreibungen oder spezifische Definitionen für diese Variante  
  }

* ThemeDefinition (Struct):  
  Die vollständige Definition eines Themes, inklusive Metadaten, Basis-Tokens, Varianten und unterstützten Akzentfarben.  
  Rust  
  \#  
  pub struct ThemeDefinition {  
      pub id: ThemeIdentifier,  
      pub name: String, // Anzeigename, z.B. "Adwaita Next Generation"  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub description: Option\<String\>,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub author: Option\<String\>,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub version: Option\<String\>,  
      pub base\_tokens: TokenSet, // Grundlegende Tokens, die für alle Varianten gelten  
      \#\[serde(default, skip\_serializing\_if \= "Vec::is\_empty")\]  
      pub variants: Vec\<ThemeVariantDefinition\>, // Definitionen für Hell, Dunkel etc.  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub supported\_accent\_colors: Option\<Vec\<AccentColor\>\>, // Vordefinierte Akzentfarben  
  }

### **2.3. Konfigurations- und Zustandsstrukturen**

Diese Strukturen repräsentieren die vom Benutzer gewählten Theming-Einstellungen und den daraus resultierenden, angewendeten Theme-Zustand.

* AppliedThemeState (Struct):  
  Repräsentiert den aktuell im System aktiven Theme-Zustand. Entscheidend ist hier das Feld resolved\_tokens, welches alle Design-Tokens auf ihre endgültigen, CSS-kompatiblen String-Werte abbildet. Diese Struktur ist das primäre Ergebnis der Theming-Logik und wird von der UI-Schicht konsumiert.  
  Eine wichtige Invariante ist, dass resolved\_tokens keine TokenValue::Reference mehr enthalten darf; alle Werte müssen endgültig aufgelöst sein.  
  Rust  
  \# // Deserialize ist hier nicht zwingend nötig  
  pub struct AppliedThemeState {  
      pub theme\_id: ThemeIdentifier,  
      pub color\_scheme: ColorSchemeType,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub active\_accent\_color: Option\<AccentColor\>,  
      // Schlüssel: TokenIdentifier (z.B., "color.background.default")  
      // Wert: Final aufgelöster CSS-String (z.B., "\#FFFFFF")  
      pub resolved\_tokens: std::collections::HashMap\<TokenIdentifier, String\>,  
  }

* ThemingConfiguration (Struct):  
  Speichert die benutzerspezifischen Einstellungen für das Theming. Diese Konfiguration wird typischerweise von einer übergeordneten Einstellungsverwaltung (domain::settings) bereitgestellt und dient als Eingabe für die ThemingEngine. Sie ermöglicht es Benutzern, ihr bevorzugtes Theme, Farbschema, Akzentfarbe und sogar einzelne Tokens global zu überschreiben.  
  Rust  
  \#  
  pub struct ThemingConfiguration {  
      pub selected\_theme\_id: ThemeIdentifier,  
      pub preferred\_color\_scheme: ColorSchemeType, // Präferenz des Benutzers  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      pub selected\_accent\_color: Option\<AccentColor\>,  
      \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\]  
      // Ermöglicht Power-Usern, spezifische Tokens für jedes Theme zu überschreiben  
      pub custom\_user\_token\_overrides: Option\<TokenSet\>,  
  }

### **2.4. Tabellen für Datenstrukturen**

Die folgenden Tabellen fassen die Schlüsseleigenschaften der wichtigsten Datenstrukturen zusammen und dienen als schnelle Referenz für Entwickler. Sie verdeutlichen die Struktur und die Bedeutung der einzelnen Felder, was für die korrekte Implementierung und Nutzung dieser Typen unerlässlich ist. Die explizite Angabe von serde-Attributen und abgeleiteten Traits stellt sicher, dass die Strukturen direkt für die Datenpersistenz und den internen Gebrauch geeignet sind.

* **Tabelle 2.1: RawToken Felder**

| Feldname | Rust-Typ | Sichtbarkeit | Initialwert (JSON Default) | Invarianten/Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| id | TokenIdentifier | pub | N/A (erforderlich) | Eindeutiger, hierarchischer Bezeichner des Tokens. |
| value | TokenValue | pub | N/A (erforderlich) | Der Wert des Tokens, kann ein primitiver Typ oder eine Referenz auf ein anderes Token sein. |
| description | Option\<String\> | pub | None | Optionale Beschreibung des Tokens und seines Verwendungszwecks. |
| group | Option\<String\> | pub | None | Optionale Gruppierung (z.B. "Farben", "Typografie") zur besseren Organisation. |

* **Tabelle 2.2: ThemeDefinition Felder**

| Feldname | Rust-Typ | Sichtbarkeit | Initialwert (JSON Default) | Invarianten/Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| id | ThemeIdentifier | pub | N/A (erforderlich) | Eindeutiger Bezeichner des Themes. |
| name | String | pub | N/A (erforderlich) | Menschenlesbarer Name des Themes. |
| description | Option\<String\> | pub | None | Optionale Beschreibung des Themes. |
| author | Option\<String\> | pub | None | Optionaler Autor des Themes. |
| version | Option\<String\> | pub | None | Optionale Version des Themes. |
| base\_tokens | TokenSet | pub | N/A (erforderlich, kann leer sein) | Set von Basis-Tokens, die für alle Varianten gelten, falls nicht spezifisch überschrieben. |
| variants | Vec\<ThemeVariantDefinition\> | pub | \`\` (leerer Vektor) | Definitionen für spezifische Varianten (z.B. Hell, Dunkel). |
| supported\_accent\_colors | Option\<Vec\<AccentColor\>\> | pub | None | Optionale Liste vordefinierter Akzentfarben, die gut mit diesem Theme harmonieren. |

* **Tabelle 2.3: AppliedThemeState Felder**

| Feldname | Rust-Typ | Sichtbarkeit | Beschreibung |
| :---- | :---- | :---- | :---- |
| theme\_id | ThemeIdentifier | pub | ID des aktuell angewendeten Themes. |
| color\_scheme | ColorSchemeType | pub | Das aktuell angewendete Farbschema (Hell/Dunkel). |
| active\_accent\_color | Option\<AccentColor\> | pub | Die aktuell angewendete Akzentfarbe, falls eine ausgewählt wurde. |
| resolved\_tokens | std::collections::HashMap\<TokenIdentifier, String\> | pub | Eine Map aller Design-Tokens, aufgelöst zu ihren finalen, CSS-kompatiblen String-Werten. Enthält keine Referenzen. |

## **3\. Kernlogik und Geschäftsregeln (domain::theming::logic)**

Dieser Abschnitt beschreibt die internen Algorithmen und Regeln, die das Verhalten der Theming-Engine steuern. Diese Logik wird in privaten (priv) oder modul-internen (pub(crate)) Funktionen und Untermodulen innerhalb von domain::theming implementiert und von der in Abschnitt 4 definierten öffentlichen API genutzt.

### **3.1. Laden, Parsen und Validieren von Token- und Theme-Definitionen**

Die Theming-Engine muss in der Lage sein, Token- und Theme-Definitionen aus externen Quellen, typischerweise JSON-Dateien, zu laden, zu parsen und auf ihre Gültigkeit zu überprüfen.

* **Token-Dateien (\*.tokens.json):**  
  * **Ladepfade:** Token-Definitionen werden von standardisierten Pfaden geladen. Systemweite Tokens befinden sich beispielsweise unter /usr/share/desktop-environment/themes/tokens/, während benutzerspezifische Tokens unter $XDG\_CONFIG\_HOME/desktop-environment/themes/tokens/ (gemäß XDG Base Directory Specification) abgelegt werden können. Benutzerspezifische Dateien haben Vorrang und können systemweite Tokens überschreiben oder ergänzen.  
  * **Einlesen und Parsen:** Es wird eine Logik implementiert, die JSON-Dateien einliest, welche entweder ein Vec\<RawToken\> oder direkt ein TokenSet (als JSON-Objekt, bei dem Schlüssel Token-IDs sind) enthalten. Für das Parsen wird die serde\_json-Bibliothek verwendet.  
  * **Validierung:**  
    * **Eindeutigkeit der TokenIdentifier:** Beim Laden mehrerer Token-Dateien muss sichergestellt werden, dass Token-Identifier eindeutig sind. Bei Konflikten (gleiche ID aus verschiedenen Quellen) wird eine klare Strategie verfolgt: Benutzerspezifische Tokens haben Vorrang vor systemweiten Tokens. Bei gleichrangigen Konflikten wird eine Warnung geloggt, und das zuletzt geladene Token überschreibt das vorherige.  
    * **Zyklische Referenzen:** Es muss geprüft werden, ob TokenValue::Reference-Abhängigkeiten Zyklen bilden (z.B. Token A verweist auf B, B verweist auf A). Dies erfordert einen Graphenalgorithmus, wie z.B. eine Tiefensuche (DFS), um solche Zyklen zu erkennen. Ein erkannter Zyklus führt zu einem ThemingError::CyclicTokenReference.  
    * **Fehlerbehandlung:** Parse-Fehler (ungültiges JSON) oder ungültige Werte innerhalb der Tokens (z.B. ein fehlerhaftes Farbformat, das nicht CSS-kompatibel ist) führen zu einem ThemingError::TokenFileParseError bzw. ThemingError::InvalidTokenData.  
* **Theme-Definitionsdateien (\*.theme.json):**  
  * **Ladepfade:** Analog zu Token-Dateien, z.B. /usr/share/desktop-environment/themes/\[theme\_id\]/\[theme\_id\].theme.json für systemweite Themes und $XDG\_CONFIG\_HOME/desktop-environment/themes/\[theme\_id\]/\[theme\_id\].theme.json für benutzerspezifische Themes.  
  * **Einlesen und Parsen:** Es wird eine Logik implementiert, die JSON-Dateien einliest, die eine ThemeDefinition-Struktur repräsentieren. Auch hier kommt serde\_json zum Einsatz.  
  * **Validierung:**  
    * **Referenzierte Tokens:** Es muss sichergestellt werden, dass Tokens, die in base\_tokens oder variants\[\*\].tokens als TokenValue::Reference definiert sind, entweder auf bekannte globale Tokens (aus den geladenen \*.tokens.json-Dateien) verweisen oder innerhalb derselben ThemeDefinition (z.B. in base\_tokens) definiert sind. Fehlende Referenzen führen zu einem Fehler.  
    * **Vollständigkeit der Varianten:** Es sollte geprüft werden, ob für gängige ColorSchemeType-Werte (insbesondere Light und Dark) entsprechende ThemeVariantDefinitions existieren oder ob die base\_tokens als ausreichend für alle Schemata betrachtet werden können. Fehlende, aber erwartete Varianten könnten zu Warnungen führen.  
    * **Fehlerbehandlung:** Fehler beim Parsen oder ungültige Datenstrukturen führen zu ThemingError::ThemeFileLoadError oder ThemingError::InvalidThemeData.  
* **Logging:** Während des Lade-, Parse- und Validierungsprozesses wird das tracing-Framework intensiv genutzt:  
  * tracing::debug\!: Für Informationen über geladene Dateien und erfolgreich geparste Definitionen.  
  * tracing::warn\!: Für nicht-kritische Probleme, wie das Überschreiben von Tokens durch benutzerspezifische Definitionen oder kleinere Validierungsfehler, die nicht das Laden des gesamten Themes verhindern.  
  * tracing::error\!: Für kritische Fehler, die das Laden oder die Verwendung eines Tokensets oder einer Theme-Definition unmöglich machen (z.B. Parse-Fehler, zyklische Referenzen).

### **3.2. Mechanismus zur Auflösung und Vererbung von Tokens (Token Resolution Pipeline)**

Dies ist die zentrale Logikkomponente der Theming-Engine. Sie ist dafür verantwortlich, aus den rohen RawTokens, der ausgewählten ThemeDefinition und der aktuellen ThemingConfiguration die endgültigen, anwendbaren Token-Werte zu berechnen, die im AppliedThemeState.resolved\_tokens gespeichert werden. Dieser Prozess stellt sicher, dass alle Referenzen aufgelöst, Überschreibungen korrekt angewendet und spezifische Anpassungen (wie Akzentfarben) berücksichtigt werden.  
Die Auflösung erfolgt in einer klar definierten Reihenfolge von Schritten für eine gegebene ThemingConfiguration:

1. **Basissatz globaler Tokens bestimmen:**  
   * Lade alle RawTokens aus den systemweiten und benutzerspezifischen Token-Dateien (\*.tokens.json).  
   * Diese Sammlung bildet den "Foundation Layer" oder den globalen Token-Pool, auf den sich Themes beziehen können. Bei Namenskonflikten haben benutzerspezifische Tokens Vorrang.  
2. **Theme-spezifische Tokens laden und anwenden:**  
   * Identifiziere und lade die ThemeDefinition für die in ThemingConfiguration.selected\_theme\_id angegebene ID.  
   * Beginne mit einer Kopie der base\_tokens aus dieser ThemeDefinition. Diese Tokens können entweder eigenständige Werte definieren oder Referenzen auf Tokens im globalen Pool (aus Schritt 1\) sein.  
3. **Varianten-spezifische Tokens anwenden:**  
   * Ermittle die preferred\_color\_scheme (z.B. Light oder Dark) aus der ThemingConfiguration.  
   * Suche in der ThemeDefinition.variants nach einer ThemeVariantDefinition, deren applies\_to\_scheme mit der bevorzugten Einstellung übereinstimmt.  
   * Wenn eine passende Variante gefunden wird, merge deren tokens über das bisherige Set (aus Schritt 2). "Merging" bedeutet hier, dass Tokens aus der Variante gleichnamige Tokens aus den base\_tokens (oder dem globalen Pool, falls die Basis-Tokens Referenzen waren) überschreiben.  
4. **Akzentfarben-Logik anwenden (falls ThemingConfiguration.selected\_accent\_color vorhanden ist):**  
   * Dieser Schritt ist komplex und hängt stark davon ab, wie ein Theme die Integration von Akzentfarben definiert.  
   * **Ansatz 1: Direkte Ersetzung über spezielle Token-IDs:** Das Theme definiert Tokens mit speziellen, reservierten IDs (z.B. color.accent.primary.value, color.accent.secondary.value). Die Werte dieser Tokens werden direkt durch den value-Teil der selected\_accent\_color (z.B. "\#3498db") ersetzt. Das Theme kann auch Tokens definieren, die auf diese Akzent-Tokens verweisen (z.B. button.background.active verweist auf color.accent.primary.value).  
   * **Ansatz 2: Farbmanipulation (fortgeschritten):** Basierend auf der selected\_accent\_color.value könnten andere verwandte Farben dynamisch generiert werden (z.B. hellere/dunklere Schattierungen für Hover/Active-Zustände, kontrastierende Textfarben). Dies würde eine Farbmanipulationsbibliothek erfordern. Für die Erstimplementierung wird die direkte Ersetzung (Ansatz 1\) bevorzugt, da sie einfacher umzusetzen ist und weniger Abhängigkeiten erfordert.  
   * Die ThemeDefinition könnte ein Feld enthalten, das auflistet, welche ihrer Tokens als "akzentfähig" gelten und wie sie von der selected\_accent\_color beeinflusst werden.  
5. **Benutzerdefinierte globale Token-Overrides anwenden:**  
   * Wenn in der ThemingConfiguration ein custom\_user\_token\_overrides-Set vorhanden ist, merge diese Tokens über das bisherige, aus den vorherigen Schritten resultierende Set. Diese benutzerdefinierten Überschreibungen haben die höchste Priorität und überschreiben jeden zuvor festgelegten Wert für ein Token mit derselben ID.  
6. **Referenzen auflösen (rekursiv):**  
   * Nachdem alle Überschreibungen angewendet wurden, iteriere durch alle Tokens im aktuellen Set.  
   * Wenn ein Token den Wert TokenValue::Reference(target\_id) hat:  
     * Suche das Token mit der target\_id im aktuellen Set.  
     * **Erfolgreiche Auflösung:** Wenn target\_id gefunden wird und dessen Wert *kein* weiterer Reference ist (d.h., es ist ein konkreter Wert wie Color, Dimension etc.), ersetze den Wert des ursprünglichen Tokens (das die Referenz enthielt) durch den aufgelösten Wert des Ziel-Tokens.  
     * **Kaskadierte Referenz:** Wenn target\_id gefunden wird, aber dessen Wert ebenfalls ein Reference ist, muss diese Referenz ebenfalls aufgelöst werden. Dieser Prozess wird rekursiv fortgesetzt.  
     * **Fehlende Referenz:** Wenn target\_id nicht im aktuellen Set gefunden wird, ist dies ein Fehler, der als ThemingError::MissingTokenReference behandelt wird. Das referencing Token kann nicht aufgelöst werden.  
     * **Zyklenerkennung:** Während der rekursiven Auflösung muss ein Mechanismus zur Erkennung von Zyklen aktiv sein (z.B. durch Verfolgung des Auflösungspfads). Ein Zyklus (z.B. A → B → C → A) würde zu einer Endlosschleife führen und muss als ThemingError::CyclicTokenReference abgefangen werden. Die Validierung in Schritt 3.1 sollte Zyklen bereits erkennen, aber eine zusätzliche Prüfung hier dient als Sicherheitsnetz.  
     * **Maximale Rekursionstiefe:** Eine maximale Tiefe für die Auflösung von Referenzen (z.B. 10-20 Ebenen) sollte festgelegt werden, um bei unentdeckten Fehlern oder extrem verschachtelten (aber gültigen) Strukturen eine Endlosschleife zu verhindern und einen ThemingError::MaxReferenceDepthExceeded auszulösen.  
7. **Finale Wertkonvertierung und Erstellung des AppliedThemeState:**  
   * Nachdem alle Referenzen erfolgreich aufgelöst wurden, enthält das Token-Set nur noch konkrete TokenValue-Varianten (außer Reference).  
   * Konvertiere alle diese TokenValues in ihre finalen String-Repräsentationen, die direkt von der UI-Schicht (z.B. als CSS-Werte) verwendet werden können. Beispielsweise wird TokenValue::Color("\#aabbcc".to\_string()) zu String::from("\#aabbcc").  
   * Das Ergebnis dieser Konvertierung ist eine HashMap\<TokenIdentifier, String\>, die zusammen mit der theme\_id, color\_scheme und active\_accent\_color aus der ThemingConfiguration den neuen AppliedThemeState bildet.  
* **Caching:** Da die Token-Auflösung potenziell rechenintensiv sein kann (insbesondere bei vielen Tokens, komplexen Referenzen und häufigen Theme-Wechseln), sollte ein Caching-Mechanismus in Betracht gezogen werden.  
  * Ein aufgelöstes AppliedThemeState (oder zumindest das resolved\_tokens-Set) kann für eine gegebene Kombination aus (ThemeIdentifier, ColorSchemeType, Option\<AccentColor\>, HashOfUserOverrides) gecacht werden.  
  * Der Cache muss invalidiert werden, wenn sich zugrundeliegende Token-Dateien (\*.tokens.json) oder Theme-Definitionen (\*.theme.json) ändern (z.B. durch Aufruf von reload\_themes\_and\_tokens() in der ThemingEngine) oder wenn sich die custom\_user\_token\_overrides ändern.

### **3.3. Regeln für dynamische Theme-Wechsel und Aktualisierung des Theme-Zustands**

Die Theming-Engine muss in der Lage sein, auf Änderungen der ThemingConfiguration (z.B. durch Benutzereingaben in den Einstellungen) dynamisch zur Laufzeit zu reagieren.

1. **Benachrichtigung über Konfigurationsänderung:** Die ThemingEngine wird über eine Änderung der ThemingConfiguration informiert, typischerweise durch einen Methodenaufruf ihrer öffentlichen API (z.B. update\_configuration(new\_config)).  
2. **Neuberechnung des Theme-Zustands:** Nach Erhalt der neuen Konfiguration führt die ThemingEngine die vollständige Token Resolution Pipeline (wie in Abschnitt 3.2 beschrieben) erneut aus, unter Verwendung der new\_config.  
3. **Aktualisierung des internen Zustands:** Der resultierende AppliedThemeState wird zum neuen internen aktuellen Zustand der ThemingEngine.  
4. **Event-Benachrichtigung:** Wenn sich der neu berechnete AppliedThemeState vom vorherigen Zustand unterscheidet, emittiert die ThemingEngine ein ThemeChangedEvent. Dieses Event enthält den neuen AppliedThemeState und ermöglicht es anderen Teilen des Systems (insbesondere der UI-Schicht), auf die Änderung zu reagieren und ihr Erscheinungsbild entsprechend zu aktualisieren.

### **3.4. Invarianten und Konsistenzprüfungen**

Um die Stabilität und Korrektheit des Theming-Systems zu gewährleisten, müssen bestimmte Invarianten jederzeit gelten:

* **Keine Referenzen im AppliedThemeState:** Das Feld resolved\_tokens eines AppliedThemeState-Objekts darf unter keinen Umständen TokenValue::Reference-Typen (oder deren String-Äquivalente, falls die Auflösung fehlschlägt) enthalten. Alle Werte müssen endgültig und direkt verwendbar sein.  
* **Gültiger Fallback-Zustand:** Die ThemingEngine muss auch dann einen gültigen (wenn auch möglicherweise minimalen) AppliedThemeState bereitstellen können, wenn Konfigurationsdateien fehlerhaft, unvollständig oder nicht vorhanden sind. Hierfür ist ein Default-Fallback-Theme erforderlich. Dieses Fallback-Theme sollte entweder fest im Code einkompiliert sein (z.B. über include\_str\! aus eingebetteten JSON-Ressourcen) oder aus einer garantierten, immer verfügbaren Quelle geladen werden können. Ein Fehlschlagen beim Laden des Fallback-Themes ist ein kritischer Fehler (ThemingError::FallbackThemeLoadError).  
* **Zuverlässige Zyklenerkennung:** Zyklische Abhängigkeiten in Token-Referenzen müssen bei der Validierung (3.1) und spätestens bei der Auflösung (3.2) zuverlässig erkannt und als Fehler (ThemingError::CyclicTokenReference) behandelt werden, um Endlosschleifen und Systeminstabilität zu verhindern.  
* **Konsistenz der ThemeIdentifier:** Alle in ThemingConfiguration oder intern verwendeten ThemeIdentifier müssen auf tatsächlich geladene und validierte ThemeDefinitions verweisen, es sei denn, es handelt sich um den expliziten Fallback-Zustand.

## **4\. Öffentliche API-Spezifikation (domain::theming::api)**

Dieser Abschnitt definiert die öffentliche Schnittstelle des domain::theming-Moduls. Die Interaktion mit der Theming-Logik erfolgt primär über den ThemingEngine-Service. Diese API ist so gestaltet, dass sie klar, robust und einfach von anderen Modulen, insbesondere der UI-Schicht und der Einstellungsverwaltung, genutzt werden kann.

### **4.1. Haupt-Service: ThemingEngine**

Der ThemingEngine-Service ist die zentrale Struktur, die die gesamte Theming-Logik kapselt, den aktuellen Theme-Zustand verwaltet und als Schnittstelle für andere Systemteile dient. Er wird typischerweise als eine gemeinsam genutzte, langlebige Instanz im System existieren (z.B. als Singleton oder über Dependency Injection bereitgestellt).  
Die Implementierung muss Thread-Sicherheit gewährleisten (Send \+ Sync), da von verschiedenen Threads (z.B. UI-Thread, Hintergrund-Threads für Konfigurationsaktualisierungen) darauf zugegriffen werden könnte. Dies wird üblicherweise durch die Verwendung von Arc\<Mutex\<ThemingEngineInternalState\>\> für den internen, veränderlichen Zustand erreicht.

Rust

// Angenommen in domain::theming::mod.rs oder domain::theming::api.rs

use crate::core::errors::CoreError; // Basis-Fehlertyp, falls benötigt  
use super::types::\*;  
use super::errors::ThemingError;  
use std::sync::{Arc, Mutex};  
use std::path::PathBuf;  
// Für Eventing wird eine robuste Multi-Producer, Multi-Consumer (MPMC) Broadcast-Lösung  
// oder eine sorgfältig verwaltete Liste von mpsc-Sendern empfohlen.  
// Hier als Beispiel mit einer Liste von mpsc::Sendern für Einfachheit,  
// aber tokio::sync::broadcast oder crossbeam\_channel::Sender (cloneable) wären bessere Optionen.  
use std::sync::mpsc;

pub struct ThemingEngine {  
    internal\_state: Arc\<Mutex\<ThemingEngineInternalState\>\>,  
    // Hält Sender-Enden für alle Subscriber.  
    event\_subscribers: Arc\<Mutex\<Vec\<mpsc::Sender\<ThemeChangedEvent\>\>\>\>,  
}

struct ThemingEngineInternalState {  
    current\_config: ThemingConfiguration,  
    available\_themes: Vec\<ThemeDefinition\>, // Geladen beim Start/Refresh  
    global\_raw\_tokens: TokenSet, // Globale Tokens, nicht Teil eines Themes  
    applied\_state: AppliedThemeState,  
    // Pfade, von denen Tokens und Themes geladen wurden, für \`reload\_themes\_and\_tokens\`  
    theme\_load\_paths: Vec\<PathBuf\>,  
    token\_load\_paths: Vec\<PathBuf\>,  
    // Optional: Cache für aufgelöste Token-Sets  
    // resolved\_state\_cache: HashMap\<CacheKey, AppliedThemeState\>,  
}

impl ThemingEngine {  
    // Konstruktor und Methoden werden unten definiert  
}

#### **4.1.1. Deklarierte Eigenschaften (Properties)**

Diese Eigenschaften repräsentieren den Kernzustand der ThemingEngine. Der Zugriff erfolgt ausschließlich über die unten definierten Methoden, um Kapselung und kontrollierte Zustandsänderungen zu gewährleisten.

* **Aktueller AppliedThemeState:** Der vollständig aufgelöste und angewendete Theme-Zustand. Zugänglich über get\_current\_theme\_state().  
* **Liste der verfügbaren Themes (Vec\<ThemeDefinition\>):** Eine Liste aller erfolgreich geladenen und validierten Theme-Definitionen. Zugänglich über get\_available\_themes().  
* **Aktuelle ThemingConfiguration:** Die derzeit von der Engine verwendete Benutzerkonfiguration. Zugänglich über get\_current\_configuration().

#### **4.1.2. Methoden**

Die Methoden der ThemingEngine ermöglichen die Initialisierung, Abfrage des Zustands, Aktualisierung der Konfiguration und die Registrierung für Benachrichtigungen über Zustandsänderungen.

* **Konstruktoren/Builder:**  
  * pub fn new(initial\_config: ThemingConfiguration, theme\_load\_paths: Vec\<PathBuf\>, token\_load\_paths: Vec\<PathBuf\>) \-\> Result\<Self, ThemingError\>  
    * **Beschreibung:** Initialisiert die ThemingEngine. Lädt alle verfügbaren Themes und Tokens von den angegebenen theme\_load\_paths und token\_load\_paths. Wendet die initial\_config an, um den ersten AppliedThemeState zu berechnen. Wenn dieser Prozess fehlschlägt, wird versucht, ein Fallback-Theme zu laden.  
    * **Parameter:**  
      * initial\_config: ThemingConfiguration: Die anfängliche Benutzerkonfiguration für das Theming.  
      * theme\_load\_paths: Vec\<PathBuf\>: Eine Liste von Verzeichnispfaden, in denen nach Theme-Definitionen (\*.theme.json) gesucht wird.  
      * token\_load\_paths: Vec\<PathBuf\>: Eine Liste von Verzeichnispfaden, in denen nach globalen Token-Dateien (\*.tokens.json) gesucht wird.  
    * **Rückgabe:** Result\<Self, ThemingError\>. Gibt die initialisierte ThemingEngine oder einen Fehler zurück.  
    * **Vorbedingungen:** initial\_config sollte semantisch valide sein (obwohl die Engine dies prüft). Die angegebenen Pfade müssen für das Programm lesbar sein.  
    * **Nachbedingungen:** Bei Erfolg ist die Engine initialisiert, verfügt über einen gültigen applied\_state (entweder basierend auf initial\_config oder einem Fallback) und hat alle verfügbaren Themes/Tokens geladen. event\_subscribers ist initialisiert (leer).  
    * **Mögliche Fehler:** ThemingError::TokenFileParseError, ThemingError::ThemeFileLoadError, ThemingError::CyclicTokenReference, ThemingError::InitialConfigurationError (wenn initial\_config zu einem unauflösbaren Zustand führt), ThemingError::FallbackThemeLoadError (wenn selbst das Laden des Fallback-Themes fehlschlägt).  
* **Zustandsabfrage:**  
  * pub fn get\_current\_theme\_state(\&self) \-\> Result\<AppliedThemeState, ThemingError\>  
    * **Beschreibung:** Gibt eine Kopie (Clone) des aktuellen AppliedThemeState zurück. Dies ist der primäre Weg für die UI-Schicht, die aktuellen Theme-Werte abzurufen.  
    * **Rückgabe:** Result\<AppliedThemeState, ThemingError\>. Ein Fehler ist hier unwahrscheinlich, könnte aber bei schwerwiegenden internen Inkonsistenzen auftreten (z.B. ThemingError::InternalStateError).  
    * **Thread-Sicherheit:** Diese Methode ist lesend und greift auf den internen Zustand über einen Mutex zu.  
  * pub fn get\_available\_themes(\&self) \-\> Result\<Vec\<ThemeDefinition\>, ThemingError\>  
    * **Beschreibung:** Gibt eine Kopie (Clone) der Liste aller geladenen und validierten ThemeDefinitions zurück. Nützlich für UI-Elemente, die eine Theme-Auswahl anbieten.  
    * **Rückgabe:** Result\<Vec\<ThemeDefinition\>, ThemingError\>. Fehler wie bei get\_current\_theme\_state().  
  * pub fn get\_current\_configuration(\&self) \-\> Result\<ThemingConfiguration, ThemingError\>  
    * **Beschreibung:** Gibt eine Kopie (Clone) der aktuell von der Engine verwendeten ThemingConfiguration zurück.  
    * **Rückgabe:** Result\<ThemingConfiguration, ThemingError\>. Fehler wie bei get\_current\_theme\_state().  
* **Zustandsänderung:**  
  * pub fn update\_configuration(\&self, new\_config: ThemingConfiguration) \-\> Result\<(), ThemingError\>  
    * **Beschreibung:** Aktualisiert die Konfiguration der ThemingEngine mit der new\_config. Dies löst die Token Resolution Pipeline (Abschnitt 3.2) neu aus. Der interne applied\_state wird aktualisiert. Wenn sich der applied\_state dadurch tatsächlich ändert, wird ein ThemeChangedEvent an alle registrierten Subscriber gesendet.  
    * **Parameter:**  
      * new\_config: ThemingConfiguration: Die neue anzuwendende Benutzerkonfiguration.  
    * **Rückgabe:** Result\<(), ThemingError\>.  
    * **Vorbedingungen:** new\_config sollte semantisch valide sein.  
    * **Nachbedingungen:** Der interne Zustand (current\_config, applied\_state) ist aktualisiert. Bei einer relevanten Änderung wurde ein ThemeChangedEvent gesendet.  
    * **Mögliche Fehler:** ThemingError::ThemeNotFound (wenn new\_config.selected\_theme\_id ungültig ist), ThemingError::TokenResolutionError (z.B. MissingTokenReference, CyclicTokenReference während der Anwendung der neuen Konfiguration), ThemingError::ThemeApplicationError für allgemeinere Probleme.  
  * pub fn reload\_themes\_and\_tokens(\&self) \-\> Result\<(), ThemingError\>  
    * **Beschreibung:** Veranlasst die ThemingEngine, alle Theme-Definitionen und Token-Dateien von den beim Konstruktor angegebenen Pfaden neu zu laden. Dies ist nützlich, wenn der Benutzer z.B. neue Themes manuell installiert oder bestehende Token-Dateien extern bearbeitet hat. Nach dem Neuladen wird die *aktuell gespeicherte* ThemingConfiguration auf die neu geladenen Daten angewendet. Wenn sich der applied\_state dadurch ändert, wird ein ThemeChangedEvent gesendet.  
    * **Rückgabe:** Result\<(), ThemingError\>.  
    * **Nachbedingungen:** Der interne Bestand an available\_themes und global\_raw\_tokens ist aktualisiert. Der applied\_state ist basierend auf der aktuellen Konfiguration und den neuen Daten neu berechnet. Ein Event wurde ggf. gesendet.  
    * **Mögliche Fehler:** ThemingError::TokenFileIoError, ThemingError::TokenFileParseError, ThemingError::ThemeFileIoError, ThemingError::ThemeFileLoadError (beim Neuladen), sowie Fehler, die auch bei update\_configuration auftreten können, da der Zustand neu angewendet wird.  
* **Event-Handling (Subscription):**  
  * pub fn subscribe\_to\_theme\_changes(\&self) \-\> Result\<mpsc::Receiver\<ThemeChangedEvent\>, ThemingError\>  
    * **Beschreibung:** Ermöglicht anderen Teilen des Systems (Subscriber), sich für Benachrichtigungen über Änderungen am AppliedThemeState zu registrieren. Jeder Aufruf dieser Methode erstellt einen neuen Kommunikationskanal.  
    * **Rückgabe:** Result\<mpsc::Receiver\<ThemeChangedEvent\>, ThemingError\>. Der zurückgegebene Receiver kann verwendet werden, um ThemeChangedEvents asynchron zu empfangen.  
    * **Implementierungsdetails:** Die ThemingEngine hält eine Liste von mpsc::Sender\<ThemeChangedEvent\>-Enden (in event\_subscribers). Diese Methode erstellt ein neues mpsc::channel(), fügt den Sender-Teil zur Liste hinzu und gibt den Receiver-Teil zurück. Beim Senden eines Events iteriert die Engine über alle gespeicherten Sender und versucht, das Event zu senden. Sender, deren korrespondierender Receiver nicht mehr existiert (Kanal geschlossen), werden aus der Liste entfernt.  
    * **Mögliche Fehler:** ThemingError::EventSubscriptionError (z.B. bei Problemen mit der internen Verwaltung der Subscriber-Liste, obwohl dies bei korrekter Implementierung selten sein sollte).

#### **4.1.3. Signale/Events**

Die ThemingEngine verwendet Events, um andere Systemkomponenten über relevante Zustandsänderungen zu informieren, ohne eine enge Kopplung zu erfordern.

* **ThemeChangedEvent (Struct):**  
  * **Beschreibung:** Dieses Event wird von der ThemingEngine immer dann gesendet, wenn sich der AppliedThemeState erfolgreich geändert hat, sei es durch eine neue Benutzerkonfiguration oder durch das Neuladen von Theme-Daten.  
  * **Payload:**  
    Rust  
    \# // Clone ist wichtig, damit das Event an mehrere Subscriber gesendet werden kann.  
                            // Serialize ist nicht unbedingt nötig für interne Events.  
    pub struct ThemeChangedEvent {  
        pub new\_state: AppliedThemeState,  
        // Optional könnte hier auch der alte Zustand für Vergleiche mitgesendet werden:  
        // pub old\_state: Option\<AppliedThemeState\>,  
    }

  * **Typischer Publisher:** Die ThemingEngine selbst, innerhalb der Methoden update\_configuration und reload\_themes\_and\_tokens.  
  * **Typische Subscriber:**  
    * ui::theming\_gtk (oder ein äquivalentes Modul in der UI-Schicht): Um die GTK4-CSS-Provider mit den neuen, in new\_state.resolved\_tokens enthaltenen Werten zu aktualisieren.  
    * Andere UI-Komponenten oder Widgets, die direkt auf spezifische Token-Werte reagieren müssen, ohne den Umweg über CSS (obwohl dies seltener sein sollte).  
    * Potenziell andere Domänen- oder Systemdienste, die ihr Verhalten an das aktuelle Theme anpassen müssen.

### **4.2. Tabellen für API-Spezifikation**

Diese Tabellen bieten eine kompakte Übersicht über die Methoden der ThemingEngine und die von ihr emittierten Events. Sie sind entscheidend für Entwickler, die die Engine nutzen, da sie klare Erwartungen an Signaturen, Verhalten und Fehlerfälle setzen.

* **Tabelle 4.1: ThemingEngine-Methoden**

| Name | Signatur | Zugriff | Kurzbeschreibung | Vorbedingungen | Nachbedingungen | ThemingError-Varianten (Beispiele) |
| :---- | :---- | :---- | :---- | :---- | :---- | :---- |
| new | (initial\_config: ThemingConfiguration, theme\_load\_paths: Vec\<PathBuf\>, token\_load\_paths: Vec\<PathBuf\>) \-\> Result\<Self, ThemingError\> | pub | Konstruktor. Initialisiert Engine, lädt Themes/Tokens, wendet Erstkonfiguration an, richtet Fallback ein. | initial\_config valide, Pfade lesbar. | Engine initialisiert, applied\_state gültig. | ThemeLoadError, TokenParseError, InitialConfigurationError, FallbackThemeLoadError |
| get\_current\_theme\_state | (\&self) \-\> Result\<AppliedThemeState, ThemingError\> | pub | Gibt den aktuell angewendeten AppliedThemeState zurück. | Engine muss initialisiert sein. | Eine Kopie des Zustands wird zurückgegeben. | InternalStateError (selten) |
| get\_available\_themes | (\&self) \-\> Result\<Vec\<ThemeDefinition\>, ThemingError\> | pub | Gibt eine Liste aller verfügbaren, geladenen Theme-Definitionen zurück. | Engine muss initialisiert sein. | Eine Kopie der Liste wird zurückgegeben. | InternalStateError (selten) |
| get\_current\_configuration | (\&self) \-\> Result\<ThemingConfiguration, ThemingError\> | pub | Gibt die aktuell verwendete ThemingConfiguration zurück. | Engine muss initialisiert sein. | Eine Kopie der Konfiguration wird zurückgegeben. | InternalStateError (selten) |
| update\_configuration | (\&self, new\_config: ThemingConfiguration) \-\> Result\<(), ThemingError\> | pub | Aktualisiert Konfiguration, berechnet neuen Zustand und sendet ggf. ThemeChangedEvent. | new\_config valide. | Interner Zustand aktualisiert, ThemeChangedEvent ggf. gesendet. | ThemeApplicationError, TokenResolutionError, ThemeNotFound |
| reload\_themes\_and\_tokens | (\&self) \-\> Result\<(), ThemingError\> | pub | Lädt alle Theme- und Token-Dateien neu und wendet aktuelle Konfiguration an. Sendet ggf. ThemeChangedEvent. | Konfigurierte Pfade müssen weiterhin zugänglich sein. | Interner Datenbestand aktualisiert, ThemeChangedEvent ggf. gesendet. | ThemeLoadError, TokenParseError, ThemeApplicationError |
| subscribe\_to\_theme\_changes | (\&self) \-\> Result\<mpsc::Receiver\<ThemeChangedEvent\>, ThemingError\> | pub | Registriert einen Listener für ThemeChangedEvent und gibt einen Receiver zurück. | Engine muss initialisiert sein. | Ein mpsc::Receiver wird zurückgegeben, Sender intern registriert. | EventSubscriptionError |

* **Tabelle 4.2: ThemeChangedEvent**

| Event-Name/Typ | Payload-Struktur (pub fields: Type) | Typische Publisher | Typische Subscriber | Beschreibung |
| :---- | :---- | :---- | :---- | :---- |
| ThemeChangedEvent | new\_state: AppliedThemeState | ThemingEngine | ui::theming\_gtk (und Äquivalente), UI-Komponenten, die direkt auf Tokens reagieren | Wird ausgelöst, nachdem der AppliedThemeState der ThemingEngine erfolgreich aktualisiert und geändert wurde. |

## **5\. Fehlerbehandlung (domain::theming::errors)**

Eine robuste und aussagekräftige Fehlerbehandlung ist entscheidend für die Stabilität und Wartbarkeit des domain::theming-Moduls. Gemäß den übergeordneten Entwicklungsrichtlinien (Abschnitt 4.3 der Gesamtspezifikation) wird das thiserror-Crate verwendet, um spezifische, benutzerdefinierte Fehler-Enums pro Modul zu definieren. Dies ermöglicht eine klare Kommunikation von Fehlerzuständen sowohl innerhalb des Moduls als auch an dessen Aufrufer.  
Die Fehlerbehandlung in Rust, die sich um das Result\<T, E\>-Enum dreht 1, erfordert eine sorgfältige Definition der Fehlertypen E. Während std::error::Error eine Basistrait ist 2, bieten Crates wie thiserror erhebliche Erleichterungen bei der Erstellung benutzerdefinierter Fehlertypen, die diesen Trait implementieren.1

### **5.1. Definition des ThemingError Enums**

Das ThemingError-Enum fasst alle spezifischen Fehler zusammen, die innerhalb des domain::theming-Moduls auftreten können. Jede Variante des Enums repräsentiert einen distinkten Fehlerfall und ist mit einer aussagekräftigen Fehlermeldung versehen, die Kontextinformationen für Entwickler bereitstellt. Die Verwendung von \#\[from\] für Fehler aus tieferliegenden Bibliotheken (wie std::io::Error oder serde\_json::Error) ermöglicht eine einfache Fehlerkonvertierung und erhält die Kausalkette (source()).

Rust

// In domain::theming::errors.rs  
use thiserror::Error;  
use super::types::{TokenIdentifier, ThemeIdentifier}; // Annahme: types.rs ist im selben Modul  
use std::path::PathBuf;

\#  
pub enum ThemingError {  
    \#\[error("Failed to parse token file '{path}': {source}")\]  
    TokenFileParseError {  
        path: PathBuf,  
        \#\[source\]  
        source: serde\_json::Error,  
    },

    \#\[error("I/O error while processing token file '{path}': {source}")\]  
    TokenFileIoError {  
        path: PathBuf,  
        \#\[source\]  
        source: std::io::Error,  
    },

    \#\[error("Invalid token data in file '{path}': {message}")\]  
    InvalidTokenData {  
        path: PathBuf,  
        message: String,  
    },

    \#\[error("Cyclic dependency detected involving token '{token\_id}' during token validation or resolution")\]  
    CyclicTokenReference {  
        token\_id: TokenIdentifier,  
        // Optional: path\_to\_cycle: Vec\<TokenIdentifier\> // Zur besseren Diagnose  
    },

    \#\[error("Failed to load theme definition '{theme\_id}' from file '{path}': {source}")\]  
    ThemeFileLoadError {  
        theme\_id: ThemeIdentifier,  
        path: PathBuf,  
        \#\[source\]  
        source: serde\_json::Error,  
    },

    \#\[error("I/O error while loading theme definition '{theme\_id}' from file '{path}': {source}")\]  
    ThemeFileIoError {  
        theme\_id: ThemeIdentifier,  
        path: PathBuf,  
        \#\[source\]  
        source: std::io::Error,  
    },

    \#\[error("Invalid theme data for theme '{theme\_id}' in file '{path}': {message}")\]  
    InvalidThemeData {  
        theme\_id: ThemeIdentifier,  
        path: PathBuf,  
        message: String,  
    },

    \#  
    ThemeNotFound {  
        theme\_id: ThemeIdentifier,  
    },

    \#  
    MissingTokenReference {  
        referencing\_token\_id: TokenIdentifier,  
        target\_token\_id: TokenIdentifier,  
    },

    \#  
    MaxReferenceDepthExceeded {  
        token\_id: TokenIdentifier,  
    },

    \#\[error("Failed to apply theming configuration: {message}")\]  
    ThemeApplicationError {  
        message: String,  
        // Optional: \#\[source\] source: Option\<Box\<dyn std::error::Error \+ Send \+ Sync \+ 'static\>\>,  
    },

    \#\[error("Critical error: Failed to initialize theming engine because no suitable fallback theme could be loaded.")\]  
    FallbackThemeLoadError,

    \#  
    InitialConfigurationError(String),  
      
    \#  
    InternalStateError(String),

    \#\[error("Failed to subscribe to theme change events: {0}")\]  
    EventSubscriptionError(String),

    // Beispiel für einen Wrapper für Core-Fehler, falls das Projekt einen zentralen CoreError hat.  
    // Dies ist oft weniger spezifisch als dedizierte Fehler, kann aber für die Integration nützlich sein.  
    // \#\[error("Core system error: {source}")\]  
    // CoreError(\#\[from\] crate::core::errors::CoreError),  
}

Die gewählte Granularität – ein Fehler-Enum pro Modul (ThemingError) mit spezifischen Varianten – stellt einen guten Kompromiss dar. Es vermeidet eine übermäßige Anzahl von Fehlertypen über das gesamte Projekt hinweg, bietet aber dennoch genügend Spezifität, um Fehlerquellen innerhalb des Moduls klar zu identifizieren und darauf reagieren zu können.4 Die Fehlermeldungen sind so gestaltet, dass sie möglichst viel Kontext liefern (z.B. Dateipfade, Token-IDs), was die Fehlersuche erheblich erleichtert und der Anforderung nach aussagekräftigen Fehlerberichten entspricht.1  
Die \#\[from\]-Annotation von thiserror wird genutzt, um Fehler von Abhängigkeiten wie serde\_json::Error und std::io::Error nahtlos in spezifische ThemingError-Varianten zu überführen. Dies vereinfacht den Code, da der ?-Operator direkt verwendet werden kann, und stellt sicher, dass die ursprüngliche Fehlerquelle (source) erhalten bleibt.1 Die Unterscheidung zwischen TokenFileIoError und ThemeFileIoError, obwohl beide potenziell von std::io::Error stammen, ist hier gerechtfertigt, da sie unterschiedliche logische Operationen (Lesen einer Token-Datei vs. Lesen einer Theme-Datei) und unterschiedliche Kontextinformationen (nur path vs. theme\_id und path) repräsentieren. Dies vermeidet die in 1 erwähnte Problematik, dass der Kontext bei der reinen Verwendung von \#\[from\] für denselben Quelltyp verschwimmen kann, wenn nicht genügend differenzierende Felder vorhanden sind.

### **5.2. Richtlinien zur Fehlerbehandlung und \-weitergabe innerhalb des Moduls**

* **Fehlerkonvertierung:** Innerhalb der privaten Logikfunktionen des domain::theming-Moduls (Abschnitt 3\) werden auftretende Fehler (z.B. I/O-Fehler beim Dateizugriff, Parsing-Fehler von serde\_json) systematisch in die entsprechenden Varianten von ThemingError umgewandelt. Dies geschieht häufig automatisch durch die Verwendung des ?-Operators in Verbindung mit den \#\[from\]-Annotationen im ThemingError-Enum oder, falls notwendig, manuell durch Aufrufe von .map\_err().  
* **Vermeidung von Panics:** Panics, ausgelöst durch unwrap() oder expect(), sind im Code des domain::theming-Moduls strikt zu vermeiden. Die einzige Ausnahme bilden potenziell Situationen, in denen ein absolut inkonsistenter Zustand eine sichere Fortführung des Programms unmöglich macht (z.B. ein kritischer, nicht behebbarer Fehler beim Laden des essentiellen Fallback-Themes während der Initialisierung der ThemingEngine). Solche Fälle müssen extrem selten sein, sorgfältig dokumentiert und begründet werden. Falls ein expect() in einer solchen Ausnahmesituation verwendet wird, sollte die Nachricht dem "expect as precondition"-Stil folgen, der beschreibt, warum der Entwickler erwartet hat, dass die Operation erfolgreich sein würde.2  
* **Fehlerweitergabe durch die API:** Alle öffentlichen Methoden der ThemingEngine (Abschnitt 4), die fehlschlagen können, geben Result\<T, ThemingError\> zurück. Dies zwingt den aufrufenden Code, Fehler explizit zu behandeln und ermöglicht eine differenzierte Reaktion auf verschiedene Fehlerzustände.  
* **Nutzung der source()-Kette:** Durch die korrekte Verwendung von \#\[source\] in den thiserror-Definitionen wird die Kausalkette von Fehlern bewahrt. Dies ist besonders nützlich für das Debugging, da es ermöglicht, einen Fehler bis zu seiner ursprünglichen Ursache zurückzuverfolgen, auch über Modul- oder Bibliotheksgrenzen hinweg.3

### **5.3. Tabelle für Fehlerbehandlung**

Die folgende Tabelle listet eine Auswahl der wichtigsten ThemingError-Varianten auf, beschreibt ihre Bedeutung und die typischen Umstände ihres Auftretens. Dies dient Entwicklern als Referenz für die Implementierung der Fehlerbehandlung im aufrufenden Code und für das Debugging.

* **Tabelle 5.1: ThemingError-Varianten (Auswahl)**

| Variante | \#\[error("...")\] String (Beispiel) | Gekapselter Quellfehler (via \#\[from\] oder Feld) | Beschreibung des Fehlerfalls |
| :---- | :---- | :---- | :---- |
| TokenFileParseError | "Failed to parse token file '{path}': {source}" | path: PathBuf, source: serde\_json::Error | Fehler beim Parsen einer JSON-Datei, die Tokens enthält (z.B. Syntaxfehler im JSON). |
| TokenFileIoError | "I/O error while processing token file '{path}': {source}" | path: PathBuf, source: std::io::Error | Ein-/Ausgabefehler beim Lesen oder Schreiben einer Token-Datei (z.B. Datei nicht gefunden, keine Leserechte). |
| CyclicTokenReference | "Cyclic dependency detected involving token '{token\_id}'..." | token\_id: TokenIdentifier | Eine zirkuläre Referenz zwischen Tokens wurde gefunden (z.B. Token A verweist auf B, und B verweist zurück auf A). |
| ThemeNotFound | "Theme with ID '{theme\_id}' not found among available themes" | theme\_id: ThemeIdentifier | Ein angefordertes Theme (z.B. in ThemingConfiguration) konnte nicht in den geladenen Definitionen gefunden werden. |
| MissingTokenReference | "Token resolution failed: Referenced token '{target\_token\_id}' not found (referenced by '{referencing\_token\_id}')" | referencing\_token\_id: TokenIdentifier, target\_token\_id: TokenIdentifier | Ein Token verweist auf ein anderes Token (target\_token\_id), das jedoch nicht im aktuellen Auflösungskontext existiert. |
| ThemeApplicationError | "Failed to apply theming configuration: {message}" | message: String | Allgemeiner Fehler während des Prozesses, eine neue ThemingConfiguration anzuwenden und den AppliedThemeState zu generieren. |
| FallbackThemeLoadError | "Critical error: Failed to initialize theming engine because no suitable fallback theme could be loaded." | \- | Kritischer Initialisierungsfehler: Das essentielle Fallback-Theme konnte nicht geladen oder verarbeitet werden. |
| InternalStateError | "An internal, unrecoverable error occurred in the ThemingEngine: {0}" | String (Fehlermeldung) | Ein unerwarteter, interner Fehler in der Engine, der auf einen Programmierfehler oder eine Datenkorruption hindeutet. |

## **6\. Vorgeschlagene Dateistruktur für das Modul domain::theming**

Eine klare und logische Dateistruktur ist entscheidend für die Wartbarkeit und Verständlichkeit eines Moduls. Für domain::theming wird folgende Struktur vorgeschlagen:

domain/  
└── theming/  
    ├── mod.rs           // Hauptmoduldatei (public API: ThemingEngine, Re-Exports)  
    ├── types.rs         // Definition aller Datenstrukturen (Token\*, Theme\*, Config\*, Event\*)  
    ├── errors.rs        // Definition des ThemingError Enums und zugehöriger Typen  
    ├── logic.rs         // Interne Implementierung der Kernlogik (Token-Laden, \-Auflösung etc.)  
    │                    // Kann bei Bedarf in Untermodule aufgeteilt werden:  
    │                    //   logic/token\_parser.rs  
    │                    //   logic/theme\_loader.rs  
    │                    //   logic/token\_resolver.rs  
    │                    //   logic/accent\_color\_processor.rs  
    ├── default\_themes/  // Verzeichnis für eingebettete Fallback-Theme-Dateien (JSON)  
    │   └── fallback.theme.json  
    │   └── base.tokens.json // Minimale Basis-Tokens für das Fallback-Theme  
    └──Cargo.toml        // Falls domain::theming als eigenes Crate innerhalb eines Workspace konzipiert ist

* **Begründung der Struktur:**  
  * mod.rs: Dient als Fassade des Moduls. Es deklariert die ThemingEngine-Struktur und re-exportiert die öffentlich zugänglichen Typen aus types.rs und errors.rs. Hier wird die öffentliche API des Moduls definiert und zugänglich gemacht.  
  * types.rs: Zentralisiert alle theming-spezifischen Datenstrukturen (wie RawToken, ThemeDefinition, AppliedThemeState etc.). Dies verbessert die Übersichtlichkeit und hilft, zyklische Abhängigkeiten zu vermeiden, da diese Typen sowohl von der API (mod.rs) als auch von der internen Logik (logic.rs) benötigt werden.  
  * errors.rs: Enthält ausschließlich die Definition des ThemingError-Enums und eventuell zugehöriger Hilfstypen für Fehler. Dies entspricht der Richtlinie, Fehlerdefinitionen pro Modul zu gruppieren.  
  * logic.rs: Kapselt die gesamte interne Implementierungslogik der Theming-Engine. Dazu gehören das Laden, Parsen und Validieren von Token- und Theme-Dateien, die komplexe Token Resolution Pipeline und die Handhabung von dynamischen Theme-Wechseln. Um die Komplexität zu bewältigen, kann logic.rs selbst wiederum in spezialisierte Untermodule (z.B. token\_parser.rs, token\_resolver.rs) aufgeteilt werden, die jeweils einen spezifischen Teilaspekt der Logik behandeln. Diese internen Module und Funktionen sind nicht Teil der öffentlichen API (pub(crate)).  
  * default\_themes/: Dieses Verzeichnis enthält die JSON-Dateien für das Fallback-Theme und die dafür notwendigen Basis-Tokens. Diese Dateien können zur Kompilierzeit mittels include\_str\! direkt in die Binärdatei eingebettet werden, um sicherzustellen, dass das Fallback-Theme immer verfügbar ist, selbst wenn externe Konfigurationsdateien fehlen oder beschädigt sind.  
  * Cargo.toml: Wäre vorhanden, wenn domain::theming als separates Crate innerhalb eines Rust-Workspace verwaltet wird. In diesem Fall würde es die Abhängigkeiten (wie serde, serde\_json, thiserror, tracing) und Metadaten spezifisch für dieses Crate deklarieren.

Diese Struktur fördert eine klare Trennung der Belange ("Separation of Concerns"): Die API-Definition ist von der Implementierungslogik getrennt, Datentypen sind zentralisiert, und Fehlerbehandlung sowie Ressourcen sind ebenfalls in eigenen Bereichen organisiert. Dies erleichtert neuen Entwicklern den Einstieg und vereinfacht die Wartung und Weiterentwicklung des Moduls.

## **7\. Detaillierter Implementierungsleitfaden (Schritt-für-Schritt)**

Dieser Leitfaden beschreibt die empfohlene Reihenfolge und die Details für die Implementierung des domain::theming-Moduls. Jeder Schritt sollte von umfassenden Unit-Tests begleitet werden, um die Korrektheit der Implementierung sicherzustellen.

### **7.1. Schrittweise Implementierung der Datenstrukturen (Abschnitt 2\)**

1. **Datei erstellen:** domain/theming/types.rs.  
2. **TokenIdentifier implementieren:**  
   * Struct-Definition mit String-Feld.  
   * new()-Methode, as\_str()-Methode.  
   * Ableitungen: Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize.  
   * Implementierung von std::fmt::Display.  
3. **TokenValue implementieren:**  
   * Enum-Definition mit allen Varianten (Color, Dimension,..., Reference(TokenIdentifier)).  
   * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
   * \#\[serde(rename\_all \= "kebab-case")\] Attribut für konsistente JSON-Serialisierung.  
4. **RawToken implementieren:**  
   * Struct-Definition mit Feldern id: TokenIdentifier, value: TokenValue, description: Option\<String\>, group: Option\<String\>.  
   * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
   * \#\[serde(default, skip\_serializing\_if \= "Option::is\_none")\] für optionale Felder.  
5. **TokenSet Typalias definieren:**  
   * pub type TokenSet \= std::collections::HashMap\<TokenIdentifier, RawToken\>;  
6. **ThemeIdentifier implementieren:** Analog zu TokenIdentifier.  
7. **ColorSchemeType implementieren:**  
   * Enum-Definition mit Varianten Light, Dark.  
   * Ableitungen: Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize.  
8. **AccentColor implementieren:**  
   * Struct-Definition mit Feldern name: Option\<String\>, value: String.  
   * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
9. **ThemeVariantDefinition implementieren:**  
   * Struct-Definition mit Feldern applies\_to\_scheme: ColorSchemeType, tokens: TokenSet.  
   * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
10. **ThemeDefinition implementieren:**  
    * Struct-Definition mit allen Feldern (id, name, description, author, version, base\_tokens, variants, supported\_accent\_colors).  
    * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
    * \#\[serde(default,...)\] für optionale Felder und Vektoren.  
11. **AppliedThemeState implementieren:**  
    * Struct-Definition mit Feldern theme\_id, color\_scheme, active\_accent\_color, resolved\_tokens: std::collections::HashMap\<TokenIdentifier, String\>.  
    * Ableitungen: Debug, Clone, PartialEq, serde::Serialize. Deserialize ist hier optional, da dieser Zustand typischerweise von der Engine konstruiert wird.  
12. **ThemingConfiguration implementieren:**  
    * Struct-Definition mit Feldern selected\_theme\_id, preferred\_color\_scheme, selected\_accent\_color, custom\_user\_token\_overrides.  
    * Ableitungen: Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize.  
13. **Unit-Tests für Datenstrukturen:**  
    * Für jede serialisierbare Struktur Tests schreiben, die die korrekte Serialisierung zu JSON und Deserialisierung von JSON überprüfen.  
    * Beispieldaten für JSON-Strings verwenden, die alle Felder und Varianten abdecken.  
    * Korrektheit der serde-Attribute (rename\_all, default, skip\_serializing\_if) verifizieren.  
    * Die Display-Implementierung für TokenIdentifier und ThemeIdentifier testen.

### **7.2. Implementierung des ThemingError Enums (Abschnitt 5\)**

1. **Datei erstellen:** domain/theming/errors.rs.  
2. **Abhängigkeit hinzufügen:** thiserror zur Cargo.toml des domain-Crates (oder des Workspace-Root, falls domain::theming ein eigenes Crate wird, bzw. zum Projekt-Crate).  
   Ini, TOML  
   \[dependencies\]  
   thiserror \= "1.0"  
   serde\_json \= "1.0" // Bereits für Typen benötigt, aber auch für Fehlerquellen relevant  
   \# weitere Abhängigkeiten

3. **ThemingError Enum definieren:**  
   * Das Enum wie in Abschnitt 5.1 spezifiziert implementieren.  
   * Alle Varianten mit den entsprechenden Feldern für Kontextinformationen (Pfade, IDs etc.) definieren.  
   * \#\[error("...")\] Attribute für jede Variante mit aussagekräftigen Fehlermeldungen versehen.  
   * \#\[source\] für gekapselte Fehler und \#\[from\] für automatische Konvertierung von std::io::Error und serde\_json::Error verwenden, wo passend.  
4. **Unit-Tests für ThemingError:**  
   * Tests schreiben, die sicherstellen, dass die Display-Implementierung (generiert durch \#\[error("...")\]) die erwarteten, formatierten Fehlermeldungen erzeugt.  
   * Für Fehler-Varianten, die einen \#\[source\]-Fehler kapseln, testen, ob die source()-Methode den korrekten zugrundeliegenden Fehler zurückgibt.  
   * Testen der From-Implementierungen (generiert durch \#\[from\]), indem Quellfehler manuell erzeugt und in ThemingError konvertiert werden.

### **7.3. Implementierung der Kernlogik-Funktionen und Geschäftsregeln (Abschnitt 3\)**

Diese Funktionen werden typischerweise in domain/theming/logic.rs oder dessen Untermodulen implementiert und als pub(crate) deklariert.

* **7.3.1. Token- und Theme-Definitionen laden, parsen und validieren:**  
  1. **Funktion pub(crate) fn load\_raw\_tokens\_from\_file(path: \&std::path::Path) \-\> Result\<TokenSet, ThemingError\>:**  
     * Datei öffnen und Inhalt lesen (std::fs::read\_to\_string). Fehlerbehandlung für I/O (ThemingError::TokenFileIoError).  
     * JSON-Inhalt parsen (serde\_json::from\_str) zu Vec\<RawToken\>. Fehlerbehandlung für Parsing (ThemingError::TokenFileParseError).  
     * Vec\<RawToken\> in TokenSet (HashMap) konvertieren. Dabei auf doppelte TokenIdentifier prüfen. Bei Duplikaten eine Warnung loggen (tracing::warn\!) und das zuletzt gelesene Token verwenden oder einen Fehler (ThemingError::InvalidTokenData) auslösen, je nach definierter Strategie (z.B. Duplikate innerhalb einer Datei sind ein Fehler).  
     * tracing::debug\! für erfolgreiches Laden verwenden.  
  2. **Funktion pub(crate) fn load\_theme\_definition\_from\_file(path: \&std::path::Path, theme\_id\_from\_path: ThemeIdentifier) \-\> Result\<ThemeDefinition, ThemingError\>:**  
     * Datei öffnen und Inhalt lesen. Fehlerbehandlung (ThemingError::ThemeFileIoError mit theme\_id und path).  
     * JSON-Inhalt parsen zu ThemeDefinition. Fehlerbehandlung (ThemingError::ThemeFileLoadError mit theme\_id und path).  
     * Validieren, ob theme\_def.id mit theme\_id\_from\_path (abgeleitet vom Dateinamen/Pfad) übereinstimmt. Bei Diskrepanz ThemingError::InvalidThemeData.  
  3. **Funktion pub(crate) fn validate\_tokenset\_for\_cycles(tokens: \&TokenSet) \-\> Result\<(), ThemingError\>:**  
     * Implementiert einen Algorithmus zur Zyklenerkennung (z.B. Tiefensuche) für TokenValue::Reference-Beziehungen.  
     * Hält eine Liste der besuchten Tokens während eines Auflösungspfads, um Zyklen zu erkennen.  
     * Gibt bei Zykluserkennung ThemingError::CyclicTokenReference { token\_id } zurück (wobei token\_id das erste im Zyklus erkannte Token ist oder ein Token, das Teil des Zyklus ist).  
  4. **Funktion pub(crate) fn validate\_theme\_definition\_references(theme\_def: \&ThemeDefinition, global\_tokens: \&TokenSet) \-\> Result\<(), ThemingError\>:**  
     * Iteriert durch alle Tokens in theme\_def.base\_tokens und in allen theme\_def.variants\[\*\].tokens.  
     * Für jedes Token, das ein TokenValue::Reference(target\_id) ist, prüfen, ob target\_id entweder in global\_tokens oder in theme\_def.base\_tokens (falls das aktuelle Token aus einer Variante stammt und sich auf ein Basistoken des Themes bezieht) existiert.  
     * Gibt bei einer fehlenden Referenz ThemingError::InvalidThemeData (oder einen spezifischeren Fehler wie MissingThemeTokenReference) zurück.  
  5. **Unit-Tests für Lade- und Validierungsfunktionen:**  
     * Tests mit gültigen JSON-Dateien für Tokens und Themes.  
     * Tests mit fehlerhaften JSON-Dateien (Syntaxfehler, falsche Typen).  
     * Tests mit semantisch ungültigen Daten (z.B. doppelte Token-IDs in einer Datei, zyklische Referenzen in einem TokenSet, fehlende Referenzen in einer ThemeDefinition).  
     * Sicherstellen, dass die korrekten ThemingError-Varianten zurückgegeben werden.  
* **7.3.2. Token Resolution Pipeline implementieren:**  
  1. **Hauptfunktion pub(crate) fn resolve\_applied\_state(config: \&ThemingConfiguration, available\_themes: &, global\_tokens: \&TokenSet) \-\> Result\<AppliedThemeState, ThemingError\>:**  
     * Implementiere die in Abschnitt 3.2 detailliert beschriebenen Schritte:  
       * **Theme auswählen:** Finde die ThemeDefinition für config.selected\_theme\_id in available\_themes. Bei Nichtauffinden ThemingError::ThemeNotFound.  
       * **Initiales Token-Set:** Beginne mit einer Kopie von global\_tokens. Merge (überschreibe) mit selected\_theme.base\_tokens.  
       * **Variante anwenden:** Finde die passende ThemeVariantDefinition für config.preferred\_color\_scheme. Merge deren tokens.  
       * **Akzentfarbe anwenden:** Implementiere die Logik zur Verarbeitung von config.selected\_accent\_color. Für die Erstimplementierung: Ersetze spezielle Token-IDs (z.B. {{ACCENT\_COLOR\_VALUE}} oder token.system.accent) durch accent\_color.value.  
       * **Benutzer-Overrides anwenden:** Merge config.custom\_user\_token\_overrides.  
       * **Referenzen auflösen:** Implementiere eine rekursive Funktion resolve\_references(current\_tokens: \&mut TokenSet, max\_depth: u8) \-\> Result\<(), ThemingError\>. Diese Funktion iteriert, bis keine TokenValue::Reference mehr vorhanden sind oder max\_depth erreicht ist. Sie muss Zyklenerkennung beinhalten (kann validate\_tokenset\_for\_cycles nutzen oder eine eigene Implementierung haben) und Fehler wie ThemingError::MissingTokenReference und ThemingError::MaxReferenceDepthExceeded behandeln.  
       * **Finale Werte konvertieren:** Konvertiere die nun aufgelösten TokenValues in String-Werte für das resolved\_tokens-Feld des AppliedThemeState.  
     * Konstruiere und gib den AppliedThemeState zurück.  
  2. **Hilfsfunktionen:**  
     * merge\_token\_sets(base: \&mut TokenSet, overrides: \&TokenSet): Fügt Tokens aus overrides zu base hinzu, wobei bestehende Tokens in base überschrieben werden.  
  3. **Unit-Tests für die Resolution Pipeline:**  
     * Szenarien mit einfachen Themes ohne Varianten oder Overrides.  
     * Szenarien mit Hell/Dunkel-Varianten.  
     * Szenarien mit Akzentfarben (einfache Ersetzung testen).  
     * Szenarien mit Benutzer-Overrides.  
     * Tests für mehrstufige Token-Referenzen (Aliase).  
     * Explizite Tests für Fehlerfälle: fehlende Referenzen, zyklische Referenzen während der Auflösung, Überschreitung der maximalen Tiefe.  
* **7.3.3. Fallback-Theme Logik:**  
  1. **Fallback-Ressourcen erstellen:** Erstelle domain/theming/default\_themes/fallback.theme.json und domain/theming/default\_themes/base.tokens.json mit minimalen, aber funktionsfähigen Werten. Diese sollten keine externen Referenzen enthalten und in sich geschlossen sein.  
  2. **Funktion pub(crate) fn load\_fallback\_applied\_state() \-\> Result\<AppliedThemeState, ThemingError\>:**  
     * Verwende include\_str\! Makros, um den Inhalt der JSON-Dateien zur Kompilierzeit einzubetten.  
     * Parse die eingebetteten Strings zu ThemeDefinition und TokenSet.  
     * Erzeuge einen AppliedThemeState direkt aus diesen Fallback-Daten (die Auflösung sollte hier trivial sein, da keine komplexen Referenzen erwartet werden).  
     * Diese Funktion sollte robust sein und nur im äußersten Notfall fehlschlagen (z.B. wenn die eingebetteten JSONs fehlerhaft sind, was ein Build-Problem wäre). Ein Fehler hier wäre ThemingError::FallbackThemeLoadError.

### **7.4. Implementierung des ThemingEngine-Service und seiner API (Abschnitt 4\)**

1. **Datei anpassen/erstellen:** domain/theming/mod.rs.  
2. **Strukturen definieren:**  
   * pub struct ThemingEngine { internal\_state: Arc\<Mutex\<ThemingEngineInternalState\>\>, event\_subscribers: Arc\<Mutex\<Vec\<mpsc::Sender\<ThemeChangedEvent\>\>\>\> }  
   * struct ThemingEngineInternalState {... } (Felder wie in 4.1 definiert, inklusive theme\_load\_paths, token\_load\_paths für reload).  
3. **Event-Struktur ThemeChangedEvent in types.rs definieren** (bereits in 7.1, hier nur zur Erinnerung).  
4. **Konstruktor ThemingEngine::new(...) implementieren:**  
   * Initialisiere event\_subscribers mit Arc::new(Mutex::new(Vec::new())).  
   * Initialisiere internal\_state.theme\_load\_paths und internal\_state.token\_load\_paths mit den übergebenen Pfaden.  
   * **Laden der globalen Tokens:** Iteriere über token\_load\_paths, rufe logic::load\_raw\_tokens\_from\_file für jede Datei auf und merge die Ergebnisse in internal\_state.global\_raw\_tokens. Führe logic::validate\_tokenset\_for\_cycles für das finale Set aus.  
   * **Laden der verfügbaren Themes:** Iteriere über theme\_load\_paths, finde \*.theme.json-Dateien, lade sie mit logic::load\_theme\_definition\_from\_file. Validiere jede ThemeDefinition mit logic::validate\_theme\_definition\_references gegen die global\_raw\_tokens. Sammle gültige Themes in internal\_state.available\_themes.  
   * **Anfänglichen Zustand anwenden:**  
     * Versuche, logic::resolve\_applied\_state mit initial\_config, internal\_state.available\_themes und internal\_state.global\_raw\_tokens aufzurufen.  
     * Bei Erfolg: Speichere initial\_config als internal\_state.current\_config und das Ergebnis als internal\_state.applied\_state.  
     * Bei Fehler: Logge den Fehler (tracing::warn\!). Versuche, logic::load\_fallback\_applied\_state() aufzurufen.  
       * Wenn Fallback erfolgreich: Speichere eine entsprechende Fallback-ThemingConfiguration (z.B. mit der ID des Fallback-Themes) und den Fallback-AppliedThemeState.  
       * Wenn Fallback fehlschlägt: Gib ThemingError::FallbackThemeLoadError zurück.  
   * Konstruiere und gib Ok(Self) zurück.  
5. **Implementiere get\_current\_theme\_state():** Sperre internal\_state-Mutex, klone internal\_state.applied\_state, gib Ok(cloned\_state) zurück.  
6. **Implementiere get\_available\_themes():** Sperre Mutex, klone internal\_state.available\_themes, gib Ok(cloned\_list) zurück.  
7. **Implementiere get\_current\_configuration():** Sperre Mutex, klone internal\_state.current\_config, gib Ok(cloned\_config) zurück.  
8. **Implementiere update\_configuration(new\_config: ThemingConfiguration):**  
   * Sperre internal\_state-Mutex.  
   * Speichere den alten applied\_state (für späteren Vergleich).  
   * Rufe logic::resolve\_applied\_state mit new\_config, \&self.internal\_state.available\_themes und \&self.internal\_state.global\_raw\_tokens auf.  
   * Bei Erfolg (Ok(new\_applied\_state)):  
     * Aktualisiere self.internal\_state.current\_config \= new\_config.  
     * Aktualisiere self.internal\_state.applied\_state \= new\_applied\_state.  
     * Wenn self.internal\_state.applied\_state sich vom alten applied\_state unterscheidet:  
       * Erzeuge ThemeChangedEvent { new\_state: self.internal\_state.applied\_state.clone() }.  
       * Sperre event\_subscribers-Mutex. Iteriere über die Sender und sende das geklonte Event. Entferne Sender, bei denen send() fehlschlägt (Kanal geschlossen).  
     * Gib Ok(()) zurück.  
   * Bei Fehler (Err(e)): Gib Err(e) zurück.  
9. **Implementiere reload\_themes\_and\_tokens():**  
   * Sperre internal\_state-Mutex.  
   * Lade globale Tokens und verfügbare Themes neu (wie im Konstruktor, unter Verwendung der gespeicherten theme\_load\_paths und token\_load\_paths). Aktualisiere internal\_state.global\_raw\_tokens und internal\_state.available\_themes. Fehler hierbei sollten geloggt und ggf. zurückgegeben werden.  
   * Speichere den alten applied\_state.  
   * Rufe logic::resolve\_applied\_state mit der *aktuellen* self.internal\_state.current\_config (die nicht geändert wurde) und den neu geladenen Daten auf.  
   * Aktualisiere self.internal\_state.applied\_state und sende Event wie bei update\_configuration, falls eine Änderung vorliegt.  
   * Gib Ok(()) oder den entsprechenden Lade-/Anwendungsfehler zurück.  
10. **Implementiere subscribe\_to\_theme\_changes():**  
    * Erzeuge ein neues mpsc::channel().  
    * Sperre event\_subscribers-Mutex. Füge den sender-Teil des Kanals zur Liste self.event\_subscribers hinzu.  
    * Gib Ok(receiver) zurück.  
11. **Unit-Tests für ThemingEngine:**  
    * **new():** Teste erfolgreiche Initialisierung mit gültigen Konfigurationen und Pfaden. Teste das Fallback-Verhalten, wenn initiale Konfigurationen fehlerhaft sind oder Pfade ungültig. Teste kritischen Fehler, wenn selbst Fallback fehlschlägt.  
    * **get\_\*() Methoden:** Teste, ob die korrekten Daten (Klone des internen Zustands) zurückgegeben werden.  
    * **update\_configuration():** Teste erfolgreiche Zustandsänderungen. Verifiziere, dass der applied\_state korrekt aktualisiert wird. Teste, dass ThemeChangedEvent nur gesendet wird, wenn sich der applied\_state tatsächlich ändert. Teste Fehlerfälle (z.B. ungültige ThemeIdentifier in new\_config).  
    * **reload\_themes\_and\_tokens():** Erstelle temporäre Theme-/Token-Dateien, modifiziere sie und teste, ob reload die Änderungen korrekt aufnimmt und den Zustand aktualisiert. Teste Event-Auslösung.  
    * **Event-System (subscribe\_to\_theme\_changes und Senden):** Registriere mehrere Subscriber. Löse eine Zustandsänderung aus und verifiziere, dass alle aktiven Subscriber das Event empfangen. Teste, dass Subscriber, deren Receiver fallengelassen wurde, korrekt aus der internen Liste entfernt werden und keine Fehler verursachen.  
    * **Thread-Sicherheit (konzeptionell):** Obwohl direkte Unit-Tests für Thread-Sicherheit komplex sind, stelle sicher, dass alle Zugriffe auf internal\_state und event\_subscribers korrekt durch Mutexe geschützt sind. Integrationstests könnten parallele Aufrufe simulieren.

### **7.5. Richtlinien für Unit-Tests (Zusammenfassung)**

* **Hohe Codeabdeckung:** Strebe eine hohe Testabdeckung für alle Logik-Komponenten in logic.rs und alle öffentlichen API-Methoden der ThemingEngine in mod.rs an.  
* **Fokus der Testfälle:**  
  * **Parsing und Validierung:** Korrekte Verarbeitung gültiger und ungültiger Eingabedaten (JSON-Dateien, Token-Strukturen).  
  * **Token-Auflösung:** Korrekte Auflösung von einfachen und komplexen Token-Referenzen (Aliase, Vererbung). Explizite Tests für Fehlerfälle wie fehlende Referenzen und zyklische Abhängigkeiten.  
  * **Theme-Anwendung:** Korrekte Anwendung von Basis-Themes, Varianten (Hell/Dunkel), Akzentfarben und Benutzer-Overrides.  
  * **ThemingEngine-Verhalten:** Korrekte Zustandsübergänge, Event-Auslösung und Fehlerbehandlung für alle API-Methoden.  
  * **Grenzwertanalyse:** Teste Randbedingungen (z.B. leere Token-Sets, Themes ohne Varianten, maximale Rekursionstiefe bei Referenzen).  
* **Testdaten und Fixtures:** Verwende kleine, fokussierte JSON-Beispieldateien für Tokens und Themes als Test-Fixtures. Diese können als Strings direkt in die Testfunktionen eingebettet oder aus einem Test-Ressourcenverzeichnis geladen werden.  
* **Mocking:** Für dieses Modul der Domänenschicht ist Mocking von externen Abhängigkeiten (hauptsächlich das Dateisystem) in der Regel nicht notwendig für Unit-Tests der Kernlogik. Die Ladefunktionen können mit temporären Dateien oder In-Memory-Daten getestet werden. Der Fokus liegt auf der internen Verarbeitungslogik.  
* **Testorganisation:** Unit-Tests sollten direkt neben dem zu testenden Code in Untermodulen tests liegen (\#\[cfg(test)\] mod tests {... }).

Durch die konsequente Befolgung dieses Implementierungsleitfadens und die sorgfältige Erstellung von Unit-Tests kann ein robustes, korrekt funktionierendes und wartbares domain::theming-Modul entwickelt werden.

#### **Referenzen**

1. Error Handling for Large Rust Projects \- Best Practice in GreptimeDB, Zugriff am Mai 13, 2025, [https://greptime.com/blogs/2024-05-07-error-rust](https://greptime.com/blogs/2024-05-07-error-rust)  
2. std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/index.html](https://doc.rust-lang.org/std/error/index.html)  
3. Error in std::error \- Rust, Zugriff am Mai 13, 2025, [https://doc.rust-lang.org/std/error/trait.Error.html](https://doc.rust-lang.org/std/error/trait.Error.html)  
4. Error handling \- good/best practices : r/rust \- Reddit, Zugriff am Mai 13, 2025, [https://www.reddit.com/r/rust/comments/1bb7dco/error\_handling\_goodbest\_practices/](https://www.reddit.com/r/rust/comments/1bb7dco/error_handling_goodbest_practices/)