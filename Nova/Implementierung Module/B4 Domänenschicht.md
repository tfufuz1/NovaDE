# **B4 Domänenschicht (Domain Layer) – Teil 4/4: Einstellungs- und Benachrichtigungs-Subsysteme**

Dieser Teil der Spezifikation widmet sich den verbleibenden Kernkomponenten der Domänenschicht: dem Subsystem für die Verwaltung von Einstellungen (domain::settings\_core und domain::settings\_persistence\_iface) sowie dem Subsystem für die Verarbeitung und Regelung von Benachrichtigungen (domain::notifications\_core und domain::notifications\_rules). Diese Module sind entscheidend für die Konfigurierbarkeit und das reaktive Verhalten der Desktop-Umgebung.

## **4.1. Entwicklungsmodul: Kernlogik für Einstellungen (domain::settings\_core)**

Dieses Modul bildet das Herzstück der Einstellungsverwaltung innerhalb der Domänenschicht. Es ist verantwortlich für die Definition, Validierung, Speicherung (über eine Abstraktionsschicht) und den Zugriff auf alle Konfigurationseinstellungen der Desktop-Umgebung.

### **4.1.1. Detaillierte Verantwortlichkeiten und Ziele**

* **Verantwortlichkeiten:**  
  * Definition der Struktur und Typen von Einstellungen (SettingKey, SettingValue, SettingMetadata, Setting).  
  * Bereitstellung einer zentralen Logik (SettingsCoreManager) zur Verwaltung dieser Einstellungen.  
  * Validierung von Einstellungswerten gegen definierte Metadaten (Typ, Bereich, erlaubte Werte).  
  * Koordination des Ladens und Speicherns von Einstellungen über eine abstrakte Persistenzschnittstelle (SettingsProvider).  
  * Benachrichtigung anderer Systemteile über Einstellungsänderungen mittels interner Events (SettingChangedEvent).  
* **Ziele:**  
  * Schaffung einer typsicheren und validierten Verwaltung von Einstellungen.  
  * Entkopplung der Einstellungslogik von der konkreten Speicherung und der Benutzeroberfläche.  
  * Ermöglichung einer reaktiven Anpassung des Systemverhaltens basierend auf Konfigurationsänderungen.  
  * Sicherstellung der Konsistenz und Integrität der Einstellungen.

### **4.1.2. Entitäten und Wertobjekte**

Die folgenden Datenstrukturen sind in domain/src/settings\_core/types.rs zu definieren. Sie müssen Debug, Clone und PartialEq implementieren. Für die Persistenz und den Datenaustausch ist zudem die Implementierung von serde::Serialize und serde::Deserialize für SettingValue und die darin enthaltenen Typen essenziell. Die uuid Crate wird für eindeutige IDs verwendet, wobei die Features v4 (zur Generierung) und serde (zur Serialisierung) aktiviert sein müssen.1 Für Zeitstempel wird chrono mit dem aktivierten serde-Feature eingesetzt.3

* **SettingKey (Newtype für String)**  
  * **Zweck:** Ein typsicherer Wrapper für den Schlüssel einer Einstellung (z.B. "appearance.theme.name", "notifications.do\_not\_disturb.enabled").  
  * **Warum wertvoll:** Erhöht die Typsicherheit und verhindert die versehentliche Verwendung beliebiger Strings als Einstellungsschlüssel. Fördert Klarheit im Code.  
  * **Implementierungsdetails:**  
    * Interner Typ: String.  
    * Sollte Display, Hash, Eq, PartialEq, Ord, PartialOrd, From\<String\>, AsRef\<str\> implementieren.  
    * Konstruktion z.B. über SettingKey::new("my.setting.key") oder From::from("my.setting.key").  
* **SettingValue (Enum)**  
  * **Zweck:** Ein Enum, das alle möglichen Typen von Einstellungswerten repräsentiert.  
  * **Warum wertvoll:** Ermöglicht eine flexible, aber dennoch typsichere Behandlung unterschiedlicher Einstellungsdatentypen an einer zentralen Stelle.  
  * **Varianten:**  
    * Boolean(bool)  
    * Integer(i64)  
    * Float(f64)  
    * String(String)  
    * Color(String): Hex-Farbcode (z.B. "\#RRGGBBAA").  
    * FilePath(String): Ein Pfad zu einer Datei oder einem Verzeichnis.  
    * List(Vec\<SettingValue\>): Eine geordnete Liste von SettingValue.  
    * Map(std::collections::HashMap\<String, SettingValue\>): Eine Schlüssel-Wert-Map.  
  * **Methoden (Beispiele):**  
    * pub fn as\_bool(\&self) \-\> Option\<bool\>  
    * pub fn as\_str(\&self) \-\> Option\<\&str\>  
    * Weitere as\_TYPE und try\_into\_TYPE Methoden für bequemen Zugriff und Konvertierung.  
* **SettingMetadata Struktur**  
  * **Zweck:** Enthält Metadaten zu einer Einstellung, wie Beschreibung, Standardwert, mögliche Werte (für Enums), Validierungsregeln.  
  * **Warum wertvoll:** Ermöglicht eine deklarative Definition von Einstellungen und deren Eigenschaften. Dies ist fundamental, um die Verwaltung, die automatische Generierung von Benutzeroberflächen für Einstellungen und die Validierung zu vereinfachen. Ohne Metadaten wäre jede Einstellungslogik ad-hoc und schwer zu warten.

| Attribut | Typ | Sichtbarkeit | Beschreibung |
| :---- | :---- | :---- | :---- |
| description | Option\<String\> | pub | Menschenlesbare Beschreibung der Einstellung. |
| default\_value | SettingValue | pub | Der Standardwert, der verwendet wird, wenn kein Wert gesetzt ist. |
| value\_type\_hint | String | pub | Hinweis auf den erwarteten SettingValue-Typ (z.B. "Boolean", "Integer"). |
| possible\_values | Option\<Vec\<SettingValue\>\> | pub | Für Enum-Typen: eine Liste der erlaubten Werte. |
| validation\_regex | Option\<String\> | pub | Für String-Typen: ein regulärer Ausdruck zur Validierung. |
| min\_value | Option\<SettingValue\> | pub | Für numerische Typen: der minimale erlaubte Wert. |
| max\_value | Option\<SettingValue\> | pub | Für numerische Typen: der maximale erlaubte Wert. |
| is\_sensitive | bool | pub | Gibt an, ob der Wert sensibel ist (z.B. Passwort, nicht loggen). Default: false. |
| requires\_restart | Option\<String\> | pub | Wenn Some(app\_id\_or\_service\_name), deutet an, dass eine Änderung einen Neustart der genannten Komponente erfordert. None bedeutet keinen Neustart. |

* **Setting Struktur (Entität)**  
  * **Zweck:** Repräsentiert eine einzelne, konkrete Einstellung mit ihrem aktuellen Wert und Metadaten.  
  * **Warum wertvoll:** Das zentrale Objekt, das eine Einstellung im System darstellt und deren Zustand und Verhalten kapselt.

| Attribut | Typ | Sichtbarkeit | Beschreibung | Invarianten |
| :---- | :---- | :---- | :---- | :---- |
| id | uuid::Uuid | pub | Eindeutige ID der Einstellung (intern verwendet). | Muss eindeutig sein. Generiert via Uuid::new\_v4(). |
| key | SettingKey | pub | Der eindeutige Schlüssel der Einstellung. | Muss eindeutig sein. |
| current\_value | SettingValue | pub(crate) | Der aktuell gesetzte Wert der Einstellung. | Muss den Validierungsregeln in metadata entsprechen, falls gesetzt. |
| metadata | SettingMetadata | pub | Metadaten, die diese Einstellung beschreiben. |  |
| last\_modified | chrono::DateTime\<Utc\> | pub(crate) | Zeitstempel der letzten Änderung. | Wird bei jeder erfolgreichen Wertänderung aktualisiert. |
| is\_dirty | bool | pub(crate) | true, wenn current\_value geändert wurde, aber noch nicht persistiert ist. |  |

\*   \*\*Methoden:\*\*  
    \*   \`pub fn new(key: SettingKey, metadata: SettingMetadata) \-\> Self\`: Erstellt eine neue Einstellung. Der \`current\_value\` wird initial auf \`metadata.default\_value\` gesetzt. \`id\` wird generiert. \`last\_modified\` wird auf \`Utc::now()\` gesetzt.  
    \*   \`pub fn value(\&self) \-\> \&SettingValue\`: Gibt eine Referenz auf den aktuellen Wert zurück.  
    \*   \`pub(crate) fn set\_value(\&mut self, new\_value: SettingValue, timestamp: chrono::DateTime\<Utc\>) \-\> Result\<(), SettingsCoreError\>\`: Setzt einen neuen Wert, nachdem dieser erfolgreich gegen \`self.metadata\` validiert wurde (interner Aufruf von \`validate\_value\`). Aktualisiert \`current\_value\` und \`last\_modified\`.  
    \*   \`pub fn validate\_value(value: \&SettingValue, metadata: \&SettingMetadata) \-\> Result\<(), SettingsCoreError\>\`: Statische Methode zur Validierung eines Wertes gegen die gegebenen Metadaten. Diese Methode ist separat, um auch externe Validierung zu ermöglichen, bevor \`set\_value\` aufgerufen wird.  
    \*   \`pub fn reset\_to\_default(\&mut self, timestamp: chrono::DateTime\<Utc\>)\`: Setzt den \`current\_value\` auf \`self.metadata.default\_value\` zurück und aktualisiert \`last\_modified\`.

### **4.1.3. Öffentliche API des Moduls (SettingsCoreManager)**

Der SettingsCoreManager, definiert in domain/src/settings\_core/mod.rs, ist die zentrale Schnittstelle zur Einstellungslogik. Er kapselt die Verwaltung der Setting-Objekte und die Interaktion mit dem Persistenz-Provider.  
Die Operationen zum Laden und Speichern von Einstellungen können I/O-intensiv sein. Um die Domänenschicht nicht zu blockieren, werden diese Methoden als async deklariert. Dies erfordert, dass der SettingsProvider ebenfalls asynchrone Methoden anbietet und als Arc\<dyn SettingsProvider \+ Send \+ Sync\> übergeben wird, um Thread-Sicherheit in asynchronen Kontexten zu gewährleisten.5  
Wenn Einstellungen geändert werden, müssen andere Teile der Domänenschicht (z.B. die NotificationRulesEngine oder das Theming-System) potenziell darüber informiert werden. Um eine lose Kopplung zu erreichen, sendet der SettingsCoreManager interne Events (SettingChangedEvent) über einen tokio::sync::broadcast::Sender.6 Interessierte Module können einen broadcast::Receiver abonnieren und auf diese Events reagieren, ohne dass der SettingsCoreManager explizite Kenntnis von ihnen haben muss. Dieser Mechanismus ist entscheidend für eine reaktive Architektur.

Rust

// domain/src/settings\_core/mod.rs  
use crate::settings\_persistence\_iface::{SettingsProvider, SettingsPersistenceError};  
use crate::settings\_core::types::{Setting, SettingKey, SettingValue, SettingMetadata};  
use crate::settings\_core::error::SettingsCoreError;  
use std::collections::HashMap;  
use std::sync::Arc;  
use tokio::sync::{RwLock, broadcast};  
use chrono::Utc;

\# // Clone ist wichtig für den broadcast::Sender  
pub struct SettingChangedEvent {  
    pub key: SettingKey,  
    pub new\_value: SettingValue,  
    pub old\_value: Option\<SettingValue\>,  
}

pub struct SettingsCoreManager {  
    settings: RwLock\<HashMap\<SettingKey, Setting\>\>,  
    provider: Arc\<dyn SettingsProvider \+ Send \+ Sync\>,  
    event\_sender: broadcast::Sender\<SettingChangedEvent\>,  
    registered\_metadata: RwLock\<HashMap\<SettingKey, SettingMetadata\>\>, // RwLock auch hier für dynamische Registrierung  
}

impl SettingsCoreManager {  
    pub fn new(  
        provider: Arc\<dyn SettingsProvider \+ Send \+ Sync\>,  
        initial\_metadata: Vec\<(SettingKey, SettingMetadata)\>,  
        event\_channel\_capacity: usize  
    ) \-\> Self {  
        let (event\_sender, \_) \= broadcast::channel(event\_channel\_capacity);  
        let mut metadata\_map \= HashMap::new();  
        for (key, meta) in initial\_metadata {  
            metadata\_map.insert(key, meta);  
        }

        SettingsCoreManager {  
            settings: RwLock::new(HashMap::new()),  
            provider,  
            event\_sender,  
            registered\_metadata: RwLock::new(metadata\_map),  
        }  
    }

    // Weitere Methoden folgen  
}

* **Tabelle: Methoden des SettingsCoreManager**

| Methode | Signatur | Kurzbeschreibung | Vorbedingungen | Nachbedingungen (Erfolg) | Nachbedingungen (Fehler) |
| :---- | :---- | :---- | :---- | :---- | :---- |
| new | pub fn new(provider: Arc\<dyn SettingsProvider \+ Send \+ Sync\>, initial\_metadata: Vec\<(SettingKey, SettingMetadata)\>, event\_channel\_capacity: usize) \-\> Self | Konstruktor. Initialisiert den Manager mit Provider, Metadaten und Event-Kanal-Kapazität. | provider ist valide. event\_channel\_capacity \> 0\. | SettingsCoreManager ist initialisiert. event\_sender ist erstellt. settings ist leer. registered\_metadata ist gefüllt. | \- |
| register\_setting\_metadata | pub async fn register\_setting\_metadata(\&self, key: SettingKey, metadata: SettingMetadata) \-\> Result\<(), SettingsCoreError\> | Registriert Metadaten für eine neue Einstellung zur Laufzeit. | key ist noch nicht registriert. | Metadaten für key sind in registered\_metadata gespeichert. | SettingsCoreError::SettingKeyAlreadyExists |
| load\_all\_settings | pub async fn load\_all\_settings(\&self) \-\> Result\<(), SettingsCoreError\> | Lädt alle Einstellungen, für die Metadaten registriert sind, vom SettingsProvider. | provider ist erreichbar. | Interne settings-Map ist mit geladenen Werten (oder Defaults aus Metadaten) gefüllt. | SettingsCoreError::PersistenceError, SettingsCoreError::ValidationError |
| get\_setting\_value | pub async fn get\_setting\_value(\&self, key: \&SettingKey) \-\> Result\<SettingValue, SettingsCoreError\> | Ruft den aktuellen Wert einer Einstellung ab. Lädt ggf. nach, falls nicht im Speicher. | key muss registriert sein. | SettingValue des Schlüssels wird zurückgegeben. | SettingsCoreError::SettingNotFound, SettingsCoreError::UnregisteredKey, SettingsCoreError::PersistenceError |
| set\_setting\_value | pub async fn set\_setting\_value(\&self, key: \&SettingKey, value: SettingValue) \-\> Result\<(), SettingsCoreError\> | Setzt den Wert einer Einstellung. Validiert und persistiert den Wert. Sendet ein Event. | key muss registriert sein. value muss valide sein gemäß Metadaten. | Wert ist intern gesetzt, persistiert via provider. SettingChangedEvent wird gesendet. last\_modified im Setting aktualisiert. | SettingsCoreError::SettingNotFound, SettingsCoreError::UnregisteredKey, SettingsCoreError::ValidationError, SettingsCoreError::PersistenceError |
| reset\_setting\_to\_default | pub async fn reset\_setting\_to\_default(\&self, key: \&SettingKey) \-\> Result\<(), SettingsCoreError\> | Setzt eine Einstellung auf ihren Standardwert (aus Metadaten) zurück. Persistiert und sendet Event. | key muss registriert sein. | Wert ist intern auf Default gesetzt, persistiert. SettingChangedEvent wird gesendet. last\_modified aktualisiert. | SettingsCoreError::SettingNotFound, SettingsCoreError::UnregisteredKey, SettingsCoreError::PersistenceError |
| get\_all\_settings\_with\_metadata | pub async fn get\_all\_settings\_with\_metadata(\&self) \-\> Result\<Vec\<Setting\>, SettingsCoreError\> | Gibt eine Liste aller aktuell verwalteten Einstellungen (inkl. ihrer Werte und Metadaten) zurück. | \- | Eine Vec\<Setting\> mit Klonen aller Einstellungen. | SettingsCoreError::PersistenceError (falls Nachladen nötig und fehlschlägt) |
| subscribe\_to\_changes | pub fn subscribe\_to\_changes(\&self) \-\> broadcast::Receiver\<SettingChangedEvent\> | Gibt einen Receiver für SettingChangedEvents zurück, um auf Einstellungsänderungen zu reagieren. | \- | Ein neuer broadcast::Receiver\<SettingChangedEvent\>. | \- |

### **4.1.4. Interne Events (SettingChangedEvent)**

Definiert in domain/src/settings\_core/mod.rs (siehe oben).

* **Zweck:** Entkoppelte Benachrichtigung anderer Domänenkomponenten über Einstellungsänderungen.  
* **Warum wertvoll:** Ermöglicht eine reaktive Architektur innerhalb der Domänenschicht. Module können auf Änderungen reagieren, ohne dass der SettingsCoreManager sie kennen muss. Dies reduziert die Kopplung und erhöht die Wartbarkeit und Erweiterbarkeit des Systems. Beispielsweise kann das Theming-Modul auf Änderungen der Akzentfarbe reagieren, ohne dass der SettingsCoreManager das Theming-Modul explizit aufrufen muss.  
* **Struktur SettingChangedEvent:**

| Feld | Typ | Beschreibung |
| :---- | :---- | :---- |
| key | SettingKey | Der Schlüssel der geänderten Einstellung. |
| new\_value | SettingValue | Der neue Wert der Einstellung. |
| old\_value | Option\<SettingValue\> | Der vorherige Wert der Einstellung (falls vorhanden oder nicht Standardwert). |

* **Typische Publisher:** SettingsCoreManager (nach erfolgreichem set\_setting\_value oder reset\_setting\_to\_default).  
* **Typische Subscriber (intern in Domänenschicht):** NotificationRulesEngine (um z.B. auf "Nicht stören"-Modus zu reagieren), ThemingEngine (um auf Theme- oder Akzentfarbänderungen zu reagieren), potenziell andere Domänenmodule, die einstellungsabhängige Logik haben.

### **4.1.5. Fehlerbehandlung (SettingsCoreError)**

Definiert in domain/src/settings\_core/error.rs unter Verwendung der thiserror-Crate, gemäß Richtlinie 4.3 der Gesamtspezifikation. Die Verwendung von thiserror für Bibliotheks-Code ist vorteilhaft, da sie spezifische, typisierte Fehler ermöglicht, die von Aufrufern explizit behandelt werden können, im Gegensatz zu generischen Fehlertypen wie anyhow::Error oder Box\<dyn std::error::Error\>.8 Die \#\[from\]-Annotation erleichtert die Konvertierung von Fehlern aus anderen Modulen (z.B. SettingsPersistenceError) in Varianten von SettingsCoreError.10

Rust

// domain/src/settings\_core/error.rs  
use thiserror::Error;  
use crate::settings\_core::types::SettingKey;  
use crate::settings\_persistence\_iface::SettingsPersistenceError;

\#  
pub enum SettingsCoreError {  
    \#  
    SettingNotFound { key: SettingKey },

    \#  
    SettingKeyAlreadyExists { key: SettingKey },

    \#\[error("Validation failed for setting '{key}': {message}")\]  
    ValidationError { key: SettingKey, message: String },

    \#\[error("Persistence operation failed for setting '{key\_str}': {source}")\]  
    PersistenceError {  
        key\_str: String, // String, da SettingKey nicht immer verfügbar oder relevant für globalen Fehler  
        \#\[source\]  
        source: SettingsPersistenceError,  
    },

    \#\[error("Attempted to operate on an unregistered setting key: '{key}'")\]  
    UnregisteredKey { key: SettingKey },

    \#\[error("An underlying I/O error occurred: {0}")\]  
    IoError(\#\[from\] std::io::Error), // Für den Fall, dass das Modul selbst I/O machen würde (selten)

    \#\[error("Event channel error while processing key '{key\_str}': {message}")\]  
    EventChannelError{ key\_str: String, message: String },  
}

// Konvertierung von SettingsPersistenceError zu SettingsCoreError  
// Dies ist nützlich, wenn ein Persistenzfehler auftritt, der nicht direkt einem Schlüssel zugeordnet ist.  
impl From\<SettingsPersistenceError\> for SettingsCoreError {  
    fn from(err: SettingsPersistenceError) \-\> Self {  
        SettingsCoreError::PersistenceError {  
            key\_str: err.get\_key().map\_or\_else(|| "global".to\_string(), |k| k.as\_str().to\_string()),  
            source: err,  
        }  
    }  
}

(Hinweis: SettingsPersistenceError müsste eine Methode get\_key() \-\> Option\<\&SettingKey\> haben, um dies sauber zu implementieren.)

* **Tabelle: SettingsCoreError Varianten**

| Variante | Beschreibung | Kontext/Ursache |
| :---- | :---- | :---- |
| SettingNotFound | Eine angeforderte Einstellung existiert nicht in der internen settings-Map. | get\_setting\_value, set\_setting\_value für einen Schlüssel, der zwar registriert, aber nicht geladen ist. |
| SettingKeyAlreadyExists | Versuch, Metadaten für einen bereits existierenden Schlüssel zu registrieren. | register\_setting\_metadata. |
| ValidationError | Ein neuer Wert für eine Einstellung entspricht nicht den Validierungsregeln. | set\_setting\_value, interne Validierung durch Setting::validate\_value. |
| PersistenceError | Fehler bei der Interaktion mit dem SettingsProvider. | Wrappt Fehler vom SettingsProvider (z.B. SettingsPersistenceError::StorageError). Verwendet \#\[source\]. |
| UnregisteredKey | Operation auf einem Schlüssel ohne registrierte Metadaten. | Wenn eine Operation Metadaten erfordert (z.B. set\_setting\_value), diese aber für den Schlüssel fehlen. |
| IoError | Generischer I/O-Fehler (eher selten direkt hier, mehr für SettingsProvider). | Beispiel für \#\[from\] std::io::Error. |
| EventChannelError | Fehler beim Senden eines SettingChangedEvent über den broadcast::Sender. | Wenn der broadcast::Sender::send() einen Fehler zurückgibt (z.B. keine aktiven Receiver und Puffer voll). |

### **4.1.6. Detaillierte Implementierungsschritte und Algorithmen**

1. **Initialisierung (SettingsCoreManager::new):**  
   * Speichere den übergebenen provider und die initial\_metadata (in RwLock\<HashMap\<...\>\>).  
   * Erstelle den broadcast::channel für SettingChangedEvent mit der spezifizierten Kapazität.  
   * Die settings-Map (RwLock\<HashMap\<SettingKey, Setting\>\>) bleibt initial leer. Einstellungen werden lazy oder durch load\_all\_settings geladen.  
2. **SettingsCoreManager::register\_setting\_metadata:**  
   * Erwirb Schreibsperre für registered\_metadata.  
   * Prüfe, ob key bereits existiert. Wenn ja, Err(SettingsCoreError::SettingKeyAlreadyExists).  
   * Füge (key, metadata) zu registered\_metadata hinzu. Ok(()).  
3. **SettingsCoreManager::load\_all\_settings:**  
   * Erwirb Lesesperre für registered\_metadata und Schreibsperre für settings.  
   * Iteriere über alle (key, metadata) in registered\_metadata.  
   * Für jeden key:  
     * Rufe self.provider.load\_setting(\&key).await auf.  
     * Bei Ok(Some(loaded\_value)):  
       * Validiere loaded\_value gegen metadata mittels Setting::validate\_value(\&loaded\_value, \&metadata). Bei Fehler: Err(SettingsCoreError::ValidationError).  
       * Erstelle ein Setting-Objekt: let setting \= Setting { id: uuid::Uuid::new\_v4(), key: key.clone(), current\_value: loaded\_value, metadata: metadata.clone(), last\_modified: Utc::now(), is\_dirty: false };  
       * Füge (key.clone(), setting) zur settings-Map hinzu.  
     * Bei Ok(None) (kein Wert persistiert):  
       * Verwende metadata.default\_value. Erstelle Setting-Objekt wie oben, aber mit metadata.default\_value.clone().  
       * Füge zur settings-Map hinzu.  
     * Bei Err(persistence\_error): Konvertiere zu SettingsCoreError::PersistenceError und gib Fehler zurück. Breche den Ladevorgang ab.  
   * Ok(()) bei Erfolg.  
4. **SettingsCoreManager::get\_setting\_value:**  
   * Erwirb Lesesperre für registered\_metadata. Prüfe, ob key registriert ist. Wenn nein, Err(SettingsCoreError::UnregisteredKey).  
   * Erwirb Lesesperre für settings.  
   * Wenn key in settings vorhanden ist, gib settings.get(key).unwrap().value().clone() zurück.  
   * Wenn key nicht in settings vorhanden (nicht geladen):  
     * Gib Lesesperre für settings frei.  
     * Rufe self.provider.load\_setting(key).await auf.  
     * Erwirb Schreibsperre für settings.  
     * Bei Ok(Some(loaded\_value)):  
       * Hole metadata aus registered\_metadata.  
       * Validiere loaded\_value. Bei Fehler: Err(SettingsCoreError::ValidationError).  
       * Erstelle Setting-Objekt, füge zu settings hinzu. Gib loaded\_value.clone() zurück.  
     * Bei Ok(None):  
       * Hole metadata aus registered\_metadata.  
       * Erstelle Setting-Objekt mit metadata.default\_value. Füge zu settings hinzu. Gib metadata.default\_value.clone() zurück.  
     * Bei Err(persistence\_error): Err(SettingsCoreError::from(persistence\_error)).  
   * Stelle sicher, dass Sperren korrekt freigegeben werden, besonders bei frühen Returns.  
5. **SettingsCoreManager::set\_setting\_value:**  
   * Erwirb Lesesperre für registered\_metadata. Hole metadata für key. Wenn nicht gefunden: Err(SettingsCoreError::UnregisteredKey).  
   * Validiere value gegen metadata mittels Setting::validate\_value(\&value, \&retrieved\_metadata). Bei Fehler: Err(SettingsCoreError::ValidationError).  
   * Erwirb Schreibsperre für settings.  
   * Hole das (mutable) Setting-Objekt für key. Wenn nicht gefunden (sollte nach get\_setting\_value-Logik oder load\_all\_settings existieren, aber zur Sicherheit prüfen oder entry() API verwenden): Err(SettingsCoreError::SettingNotFound).  
   * Speichere old\_value \= current\_setting.value().clone().  
   * Rufe current\_setting.set\_value(value.clone(), Utc::now()) auf (dies validiert intern nicht erneut, da bereits geschehen).  
   * Setze current\_setting.is\_dirty \= true.  
   * Rufe self.provider.save\_setting(key, \&value).await auf.  
     * Bei Err(persistence\_error):  
       * Setze current\_setting.set\_value(old\_value, Utc::now()) (Rollback der In-Memory-Änderung).  
       * Setze current\_setting.is\_dirty \= false.  
       * Err(SettingsCoreError::from(persistence\_error)).  
   * Setze current\_setting.is\_dirty \= false.  
   * Erstelle SettingChangedEvent { key: key.clone(), new\_value: value, old\_value: Some(old\_value) }.  
   * Sende das Event via self.event\_sender.send(). Bei Fehler (z.B. wenn keine Subscriber da sind und der Kanal voll ist, was bei broadcast selten zu einem harten Fehler führt, aber Err zurückgeben kann): Err(SettingsCoreError::EventChannelError).  
   * Ok(()).  
6. **Validierungslogik (Setting::validate\_value):**  
   * Prüfe Typkompatibilität von value mit metadata.value\_type\_hint (z.B. SettingValue::Integer mit "Integer").  
   * Wenn metadata.possible\_values Some(list) ist, prüfe, ob value in list enthalten ist.  
   * Wenn metadata.validation\_regex Some(regex\_str) ist und value ein SettingValue::String(s) ist, kompiliere Regex und prüfe s dagegen.  
   * Prüfe metadata.min\_value / metadata.max\_value für numerische Typen (Integer, Float).  
   * Bei Verletzung: Err(SettingsCoreError::ValidationError) mit passender Nachricht.

### **4.1.7. Überlegungen zur Nebenläufigkeit und Zustandssynchronisierung**

* Die internen Zustände settings und registered\_metadata werden mit tokio::sync::RwLock geschützt. Dies erlaubt parallele Lesezugriffe, während Schreibzugriffe exklusiv sind, was für typische Einstellungs-Workloads (viele Lesezugriffe, wenige Schreibzugriffe) performant ist.  
* Der SettingsProvider wird als Arc\<dyn SettingsProvider \+ Send \+ Sync\> gehalten. Send und Sync sind notwendig, da die async-Methoden des SettingsCoreManager potenziell auf verschiedenen Threads durch den Tokio-Executor ausgeführt werden können und der Provider über Thread-Grenzen hinweg sicher geteilt werden muss.5  
* Der broadcast::Sender für SettingChangedEvent ist Thread-sicher und für die Verwendung in asynchronen Kontexten konzipiert.6

## **4.2. Entwicklungsmodul: Persistenzabstraktion und Schema für Einstellungen (domain::settings\_persistence\_iface)**

Dieses Modul definiert die Schnittstelle, über die der SettingsCoreManager Einstellungen lädt und speichert, ohne die konkrete Implementierung der Persistenz zu kennen.

### **4.2.1. Detaillierte Verantwortlichkeiten und Ziele**

* **Verantwortlichkeiten:**  
  * Definition eines abstrakten Traits (SettingsProvider), der die Operationen zum Laden und Speichern von Einstellungen vorschreibt.  
  * Definition der Fehlertypen (SettingsPersistenceError), die bei Persistenzoperationen auftreten können.  
* **Ziele:**  
  * Vollständige Entkopplung der Domänenlogik (domain::settings\_core) von spezifischen Speichertechnologien (z.B. GSettings, Konfigurationsdateien im TOML/JSON-Format, Datenbank).  
  * Ermöglichung der Testbarkeit des SettingsCoreManager durch Mocking des SettingsProvider.  
  * Flexibilität bei der Auswahl oder dem Wechsel der Speichertechnologie, ohne dass Änderungen an der Domänenschicht erforderlich sind.

Die Verwendung eines Trait-Objekts (Arc\<dyn SettingsProvider \+ Send \+ Sync\>) ist hier entscheidend. Die Send \+ Sync-Bounds sind unerlässlich, da der Provider in async-Funktionen verwendet wird, die von einem Multi-Threaded-Executor wie Tokio ausgeführt werden können. Ohne diese Bounds könnte der Compiler die Thread-Sicherheit nicht garantieren.5

### **4.2.2. Trait-Definitionen (SettingsProvider)**

Definiert in domain/src/settings\_persistence\_iface/mod.rs. Die Verwendung von async\_trait ist notwendig, um async fn in Traits zu deklarieren, solange dies nicht nativ in stabilem Rust unterstützt wird.

Rust

// domain/src/settings\_persistence\_iface/mod.rs  
use async\_trait::async\_trait;  
use crate::settings\_core::types::{SettingKey, SettingValue};  
use crate::settings\_persistence\_iface::error::SettingsPersistenceError; // Eigener Fehlertyp

\#\[async\_trait\]  
pub trait SettingsProvider {  
    async fn load\_setting(\&self, key: \&SettingKey) \-\> Result\<Option\<SettingValue\>, SettingsPersistenceError\>;  
    async fn save\_setting(\&self, key: \&SettingKey, value: \&SettingValue) \-\> Result\<(), SettingsPersistenceError\>;  
    async fn load\_all\_settings(\&self) \-\> Result\<Vec\<(SettingKey, SettingValue)\>, SettingsPersistenceError\>;  
    async fn delete\_setting(\&self, key: \&SettingKey) \-\> Result\<(), SettingsPersistenceError\>;  
    async fn setting\_exists(\&self, key: \&SettingKey) \-\> Result\<bool, SettingsPersistenceError\>;  
}

* **Tabelle: Methoden des SettingsProvider Traits**

| Methode | Signatur | Kurzbeschreibung |
| :---- | :---- | :---- |
| load\_setting | async fn load\_setting(\&self, key: \&SettingKey) \-\> Result\<Option\<SettingValue\>, SettingsPersistenceError\> | Lädt den Wert für einen Schlüssel. Ok(None) wenn nicht vorhanden. |
| save\_setting | async fn save\_setting(\&self, key: \&SettingKey, value: \&SettingValue) \-\> Result\<(), SettingsPersistenceError\> | Speichert einen Wert für einen Schlüssel. Überschreibt, falls existent. |
| load\_all\_settings | async fn load\_all\_settings(\&self) \-\> Result\<Vec\<(SettingKey, SettingValue)\>, SettingsPersistenceError\> | Lädt alle Einstellungen, die dieser Provider verwaltet (z.B. unter einem Schema). |
| delete\_setting | async fn delete\_setting(\&self, key: \&SettingKey) \-\> Result\<(), SettingsPersistenceError\> | Löscht eine Einstellung aus dem persistenten Speicher. |
| setting\_exists | async fn setting\_exists(\&self, key: \&SettingKey) \-\> Result\<bool, SettingsPersistenceError\> | Prüft, ob eine Einstellung im persistenten Speicher existiert. |

### **4.2.3. Datenstrukturen für die Persistenzschnittstelle**

Die primären Datenstrukturen, die über diese Schnittstelle ausgetauscht werden, sind SettingKey und SettingValue aus dem Modul domain::settings\_core::types. Es wird implizit erwartet, dass Implementierungen des SettingsProvider-Traits mit serialisierbaren Formen von SettingValue arbeiten können. Daher müssen SettingValue und die darin enthaltenen Typen serde::Serialize und serde::Deserialize implementieren. Die konkrete Serialisierungslogik (z.B. zu JSON, GVariant für GSettings, etc.) ist Aufgabe der jeweiligen Provider-Implementierung in der Systemschicht, nicht der Domänenschicht.

### **4.2.4. Fehlerbehandlung (SettingsPersistenceError)**

Definiert in domain/src/settings\_persistence\_iface/error.rs unter Verwendung von thiserror. Diese Fehler sind spezifisch für Persistenzoperationen und werden vom SettingsCoreManager in SettingsCoreError::PersistenceError gewrappt.

Rust

// domain/src/settings\_persistence\_iface/error.rs  
use thiserror::Error;  
use crate::settings\_core::types::SettingKey;

\#  
pub enum SettingsPersistenceError {  
    \#  
    BackendUnavailable { message: String },

    \#\[error("Failed to access storage for key '{key}': {message}")\]  
    StorageAccessError { key: SettingKey, message: String },

    \#\[error("Failed to serialize setting '{key}': {message}")\]  
    SerializationError { key: SettingKey, message: String },

    \#\[error("Failed to deserialize setting '{key}': {message}")\]  
    DeserializationError { key: SettingKey, message: String },

    \#  
    SettingNotFoundInStorage { key: SettingKey }, // Eindeutiger als der allgemeine SettingNotFound

    \#\[error("An I/O error occurred while accessing storage for key '{key\_opt:?}': {source}")\]  
    IoError {  
        key\_opt: Option\<SettingKey\>,  
        \#\[source\]  
        source: std::io::Error,  
    },

    \#\[error("An unknown persistence error occurred for key '{key\_opt:?}': {message}")\]  
    UnknownError { key\_opt: Option\<SettingKey\>, message: String },  
}

impl SettingsPersistenceError {  
    /// Hilfsmethode, um den Schlüssel aus dem Fehler zu extrahieren, falls vorhanden.  
    pub fn get\_key(\&self) \-\> Option\<\&SettingKey\> {  
        match self {  
            SettingsPersistenceError::StorageAccessError { key,.. } \=\> Some(key),  
            SettingsPersistenceError::SerializationError { key,.. } \=\> Some(key),  
            SettingsPersistenceError::DeserializationError { key,.. } \=\> Some(key),  
            SettingsPersistenceError::SettingNotFoundInStorage { key,.. } \=\> Some(key),  
            SettingsPersistenceError::IoError { key\_opt,.. } \=\> key\_opt.as\_ref(),  
            SettingsPersistenceError::UnknownError { key\_opt,.. } \=\> key\_opt.as\_ref(),  
            \_ \=\> None,  
        }  
    }  
}

* **Tabelle: SettingsPersistenceError Varianten**

| Variante | Beschreibung |
| :---- | :---- |
| BackendUnavailable | Das Speichersystem (z.B. D-Bus Dienst, Datenbankverbindung) ist nicht erreichbar. |
| StorageAccessError | Allgemeiner Fehler beim Zugriff auf den Speicher für einen bestimmten Schlüssel. |
| SerializationError | Fehler beim Serialisieren eines SettingValue für die Speicherung. |
| DeserializationError | Fehler beim Deserialisieren eines Wertes aus dem Speicher in ein SettingValue. |
| SettingNotFoundInStorage | Spezifischer Fehler, wenn ein Schlüssel im Persistenzlayer nicht existiert. |
| IoError | Wrappt std::io::Error für dateibasierte Provider. Enthält optional den Schlüssel. |
| UnknownError | Ein anderer, nicht spezifisch klassifizierter Fehler. Enthält optional den Schlüssel. |

### **4.2.5. Detaillierte Implementierungsschritte für die Interaktion mit der Schnittstelle**

Die konkreten Implementierungen des SettingsProvider-Traits (z.B. GSettingsProvider, FileConfigProvider) befinden sich typischerweise in der Systemschicht oder einer dedizierten Infrastrukturschicht, da sie systemspezifische Details oder externe Bibliotheken involvieren.  
Der SettingsCoreManager interagiert wie folgt mit dem Provider:

1. Der SettingsCoreManager hält eine Instanz von Arc\<dyn SettingsProvider \+ Send \+ Sync\>.  
2. Bei Operationen wie set\_setting\_value ruft der SettingsCoreManager die entsprechende Methode des Providers auf, z.B. provider.save\_setting(\&key, \&value).await.  
3. Gibt die Provider-Methode Ok(...) zurück, fährt der SettingsCoreManager mit seiner Logik fort (internen Zustand aktualisieren, Event senden).  
4. Gibt die Provider-Methode Err(SettingsPersistenceError) zurück, konvertiert der SettingsCoreManager diesen Fehler in eine SettingsCoreError::PersistenceError-Variante (unter Beibehaltung des ursprünglichen Fehlers als source mittels \#\[from\] oder manueller Implementierung) und gibt diesen an seinen Aufrufer weiter. Der interne Zustand des SettingsCoreManager wird gegebenenfalls auf den Stand vor dem fehlgeschlagenen Persistenzversuch zurückgesetzt (Rollback).

Diese klare Trennung stellt sicher, dass die Domänenlogik agnostisch gegenüber der Persistenztechnologie bleibt und erleichtert das Testen erheblich, da der Provider durch einen Mock ersetzt werden kann.

## **4.3. Entwicklungsmodul: Kernlogik der Benachrichtigungsverwaltung (domain::notifications\_core)**

Dieses Modul ist für die zentrale Logik der Verwaltung von Desktop-Benachrichtigungen zuständig. Es definiert, was eine Benachrichtigung ist, wie sie verarbeitet, gespeichert und ihr Lebenszyklus verwaltet wird.

### **4.3.1. Detaillierte Verantwortlichkeiten und Ziele**

* **Verantwortlichkeiten:**  
  * Definition der Datenstruktur einer Benachrichtigung (Notification) und zugehöriger Typen (NotificationId, NotificationAction, NotificationUrgency).  
  * Verwaltung des Lebenszyklus von Benachrichtigungen: Erstellung, Anzeige (konzeptionell, die Darstellung erfolgt in der UI-Schicht), Aktualisierung, Schließen.  
  * Bereitstellung einer API (NotificationCoreManager) zum programmatischen Hinzufügen und Verwalten von Benachrichtigungen.  
  * Führung einer Liste aktiver Benachrichtigungen.  
  * Verwaltung einer Benachrichtigungshistorie mit konfigurierbarer Größe und Persistenzlogik (FIFO).  
  * Unterstützung für interaktive Benachrichtigungen durch NotificationAction.  
  * Implementierung von Logik zur Deduplizierung oder zum Ersetzen von Benachrichtigungen (z.B. basierend auf replaces\_id).  
  * Interaktion mit der NotificationRulesEngine (domain::notifications\_rules) zur Anwendung von Filter-, Priorisierungs- und Modifikationsregeln.  
  * Versenden interner Events (NotificationEvent) über Zustandsänderungen von Benachrichtigungen.  
* **Ziele:**  
  * Schaffung einer zentralen, konsistenten und robusten Logik für das gesamte Benachrichtigungssystem.  
  * Strikte Trennung der Benachrichtigungslogik von der UI-Darstellung und den Transportmechanismen (wie D-Bus). Die Domänenschicht definiert *was* eine Benachrichtigung ist und *wie* sie verwaltet wird, nicht wie sie konkret aussieht oder über welche Kanäle sie empfangen/gesendet wird.  
  * Ermöglichung eines flexiblen und durch Regeln steuerbaren Benachrichtigungsflusses.

### **4.3.2. Entitäten und Wertobjekte**

Alle Typen sind in domain/src/notifications\_core/types.rs zu definieren. Sie benötigen standardmäßig Debug, Clone, PartialEq. Für die Persistenz der Historie und die Verwendung in Events ist auch serde::Serialize und serde::Deserialize für die Hauptstrukturen (Notification, NotificationAction etc.) erforderlich. uuid::Uuid (mit Features v4, serde) 1 und chrono::DateTime\<Utc\> (mit Feature serde) 3 werden für IDs bzw. Zeitstempel verwendet.

* **NotificationId (Newtype für uuid::Uuid)**  
  * **Zweck:** Eine typsichere ID für Benachrichtigungen.  
  * **Warum wertvoll:** Verhindert Verwechslungen mit anderen Uuid-basierten IDs im System und macht die API expliziter.  
  * **Implementierungsdetails:**  
    * Interner Typ: uuid::Uuid.  
    * Sollte Display, Hash, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize, Copy (da Uuid Copy ist) implementieren.  
    * Methoden: pub fn new() \-\> Self { Self(uuid::Uuid::new\_v4()) }, pub fn as\_uuid(\&self) \-\> \&uuid::Uuid { \&self.0 }, From\<uuid::Uuid\>, Into\<uuid\_Uuid\>.  
* **NotificationUrgency (Enum)**  
  * **Zweck:** Definiert die Dringlichkeitsstufe einer Benachrichtigung.  
  * **Warum wertvoll:** Standardisiert die Dringlichkeit und ermöglicht darauf basierende Logik in Regeln und UI (z.B. Sortierung, Hervorhebung, unterschiedliche Töne).

| Variante | Wert (intern, z.B. u8) | Beschreibung |
| :---- | :---- | :---- |
| Low | 0 | Niedrige Dringlichkeit (z.B. informative Updates). |
| Normal | 1 | Normale Dringlichkeit (Standard). |
| Critical | 2 | Hohe Dringlichkeit (z.B. Fehler, wichtige Alarme). |

\*   Sollte \`serde::Serialize\`, \`serde::Deserialize\`, \`Copy\` implementieren.

* **NotificationAction (Struktur, Wertobjekt)**  
  * **Zweck:** Repräsentiert eine Aktion, die der Benutzer im Kontext einer Benachrichtigung ausführen kann.  
  * **Warum wertvoll:** Ermöglicht interaktive Benachrichtigungen, die über reine Informationsanzeige hinausgehen.

| Attribut | Typ | Sichtbarkeit | Beschreibung |
| :---- | :---- | :---- | :---- |
| key | String | pub | Eindeutiger Schlüssel der Aktion innerhalb der Benachrichtigung (z.B. "reply", "archive"). |
| label | String | pub | Menschenlesbare Beschriftung für den Button/Menüeintrag (z.B. "Antworten"). |
| icon\_name | Option\<String\> | pub | Optionaler Name eines Icons für die Aktion (gemäß Freedesktop Icon Naming Spec). |

\*   Sollte \`serde::Serialize\`, \`serde::Deserialize\` implementieren.

* **Notification (Struktur, Entität)**  
  * **Zweck:** Das zentrale Objekt, das eine einzelne Benachrichtigung mit all ihren Attributen darstellt.  
  * **Warum wertvoll:** Kapselt alle Informationen einer Benachrichtigung und dient als Hauptdatentyp für die Benachrichtigungslogik.

| Attribut | Typ | Sichtbarkeit | Beschreibung | Invarianten |
| :---- | :---- | :---- | :---- | :---- |
| id | NotificationId | pub | Eindeutige ID der Benachrichtigung. | Muss eindeutig sein. Wird bei Erstellung generiert. |
| app\_name | String | pub | Name der Anwendung, die die Benachrichtigung gesendet hat (z.B. "Thunderbird", "System Update"). | Nicht leer. |
| app\_icon | Option\<String\> | pub | Pfad oder Name des Icons der Anwendung (gemäß Freedesktop Icon Naming Spec). |  |
| summary | String | pub | Kurze Zusammenfassung/Titel der Benachrichtigung. | Nicht leer. |
| body | Option\<String\> | pub | Ausführlicherer Text der Benachrichtigung. Kann Markup enthalten (abhängig von UI-Interpretation). |  |
| actions | Vec\<NotificationAction\> | pub | Liste von Aktionen, die mit der Benachrichtigung verbunden sind. | Schlüssel (key) jeder Aktion müssen innerhalb dieser Liste eindeutig sein. |
| hints | HashMap\<String, SettingValue\> | pub | Zusätzliche, anwendungsspezifische Daten oder UI-Hinweise (z.B. "image-path", "progress", "resident"). |  |
| urgency | NotificationUrgency | pub | Dringlichkeitsstufe. Default: Normal. |  |
| timestamp\_created | chrono::DateTime\<Utc\> | pub | Zeitstempel der Erstellung der Benachrichtigung *in der Domänenschicht*. | Wird bei Instanziierung gesetzt. |
| timestamp\_displayed | Option\<chrono::DateTime\<Utc\>\> | pub(crate) | Zeitstempel, wann die Benachrichtigung (potenziell) dem Benutzer angezeigt wurde (von NotificationCoreManager gesetzt). |  |
| expires\_at | Option\<chrono::DateTime\<Utc\>\> | pub | Zeitstempel, wann die Benachrichtigung automatisch geschlossen werden soll (None \= kein Timeout). |  |
| is\_persistent | bool | pub | true, wenn die Benachrichtigung nach dem Schließen in der Historie verbleiben soll. Default: false. |  |
| replaces\_id | Option\<NotificationId\> | pub | ID der Benachrichtigung, die durch diese ersetzt werden soll. |  |
| category | Option\<String\> | pub | Kategorie der Benachrichtigung (z.B. "email.new", "download.complete", "chat.incoming\_message"). Standardisierte Kategorien können für Regeln nützlich sein. |  |

\*   Sollte \`serde::Serialize\`, \`serde::Deserialize\` implementieren.

* **NotificationHistory (Struktur, Aggregatwurzel)**  
  * **Zweck:** Verwaltet die Sammlung der vergangenen (geschlossenen, persistenten) Benachrichtigungen.  
  * **Warum wertvoll:** Stellt die Logik für die Historie bereit, insbesondere die Begrenzung der Größe und den Zugriff.  
  * **Implementierungsdetails:**  
    * notifications: VecDeque\<Notification\>: Eine VecDeque ist geeignet, da sie effizientes Hinzufügen am einen Ende und Entfernen am anderen Ende (für die Größenbeschränkung) ermöglicht.  
    * max\_size: usize: Maximale Anzahl an Benachrichtigungen in der Historie.  
    * Methoden:  
      * pub fn new(max\_size: usize) \-\> Self  
      * pub fn add(\&mut self, notification: Notification): Fügt eine Benachrichtigung hinzu. Wenn max\_size überschritten wird, wird die älteste entfernt (pop\_front).  
      * pub fn get\_all(\&self) \-\> Vec\<Notification\>: Gibt eine Kopie aller historischen Benachrichtigungen zurück (neueste zuerst oder älteste zuerst, je nach Anforderung).  
      * pub fn get\_paged(\&self, limit: usize, offset: usize) \-\> Vec\<Notification\>: Gibt eine Seite der Historie zurück.  
      * pub fn clear(\&mut self): Leert die Historie.  
      * pub fn current\_size(\&self) \-\> usize.  
  * Sollte serde::Serialize, serde::Deserialize implementieren, um die gesamte Historie persistieren zu können (optional, aber nützlich).

### **4.3.3. Öffentliche API des Moduls (NotificationCoreManager)**

Definiert in domain/src/notifications\_core/mod.rs. Der NotificationCoreManager ist die Fassade für die Benachrichtigungslogik. Er verwaltet intern Listen für aktive Benachrichtigungen und eine Instanz von NotificationHistory. Er interagiert eng mit der NotificationRulesEngine.

Rust

// domain/src/notifications\_core/mod.rs  
use crate::notifications\_core::types::{Notification, NotificationId, NotificationAction, NotificationHistory, NotificationUrgency}; // NotificationUrgency für Defaults  
use crate::notifications\_core::error::NotificationCoreError;  
use crate::notifications\_core::events::{NotificationEvent, CloseReason};  
use crate::notifications\_rules::{NotificationRulesEngine, RuleProcessingResult}; // Abhängigkeit  
use std::collections::{HashMap, VecDeque};  
use std::sync::Arc;  
use tokio::sync::{RwLock, broadcast};  
use chrono::Utc;

pub struct NotificationCoreManager {  
    active\_notifications: RwLock\<HashMap\<NotificationId, Notification\>\>,  
    history: RwLock\<NotificationHistory\>,  
    rules\_engine: Arc\<NotificationRulesEngine\>,  
    event\_sender: broadcast::Sender\<NotificationEvent\>,  
    // next\_internal\_id: RwLock\<u32\>, // Für Freedesktop Notification Spec Server ID, falls benötigt  
}

impl NotificationCoreManager {  
    pub fn new(  
        rules\_engine: Arc\<NotificationRulesEngine\>,  
        history\_max\_size: usize,  
        event\_channel\_capacity: usize  
    ) \-\> Self {  
        let (event\_sender, \_) \= broadcast::channel(event\_channel\_capacity);  
        NotificationCoreManager {  
            active\_notifications: RwLock::new(HashMap::new()),  
            history: RwLock::new(NotificationHistory::new(history\_max\_size)),  
            rules\_engine,  
            event\_sender,  
        }  
    }

    // Weitere Methoden folgen  
}

* **Tabelle: Methoden des NotificationCoreManager**

| Methode | Signatur | Kurzbeschreibung |
| :---- | :---- | :---- |
| new | pub fn new(rules\_engine: Arc\<NotificationRulesEngine\>, history\_max\_size: usize, event\_channel\_capacity: usize) \-\> Self | Konstruktor. Initialisiert den Manager mit der Regel-Engine, maximaler Historiengröße und Event-Kanal-Kapazität. |
| add\_notification | pub async fn add\_notification(\&self, mut new\_notification: Notification) \-\> Result\<NotificationId, NotificationCoreError\> | Fügt eine neue Benachrichtigung hinzu. Wendet Regeln an, prüft auf Ersetzung. Sendet NotificationAdded oder NotificationSuppressedByRule Event. Gibt die ID der (ggf. modifizierten) Benachrichtigung zurück. |
| get\_active\_notification | pub async fn get\_active\_notification(\&self, id: \&NotificationId) \-\> Result\<Option\<Notification\>, NotificationCoreError\> | Ruft eine aktive Benachrichtigung anhand ihrer ID ab (als Klon). |
| get\_all\_active\_notifications | pub async fn get\_all\_active\_notifications(\&self) \-\> Result\<Vec\<Notification\>, NotificationCoreError\> | Ruft eine Liste aller derzeit aktiven Benachrichtigungen ab (als Klone). |
| close\_notification | pub async fn close\_notification(\&self, id: \&NotificationId, reason: CloseReason) \-\> Result\<(), NotificationCoreError\> | Schließt eine aktive Benachrichtigung. Verschiebt sie ggf. in die Historie (basierend auf is\_persistent und reason). Sendet NotificationClosed Event. |
| invoke\_action | pub async fn invoke\_action(\&self, notification\_id: \&NotificationId, action\_key: \&str) \-\> Result\<(), NotificationCoreError\> | Löst eine Aktion für eine Benachrichtigung aus. Sendet NotificationActionInvoked Event. Die eigentliche Ausführung der Aktion ist nicht Teil dieser Domänenlogik. |
| get\_history | pub async fn get\_history(\&self, limit: Option\<usize\>, offset: Option\<usize\>) \-\> Result\<Vec\<Notification\>, NotificationCoreError\> | Ruft Benachrichtigungen aus der Historie ab (paginiert). |
| clear\_history | pub async fn clear\_history(\&self) \-\> Result\<(), NotificationCoreError\> | Leert die Benachrichtigungshistorie. Sendet NotificationHistoryCleared Event. |
| clear\_app\_notifications | pub async fn clear\_app\_notifications(\&self, app\_name: \&str, reason: CloseReason) \-\> Result\<usize, NotificationCoreError\> | Schließt alle aktiven Benachrichtigungen einer bestimmten App. Gibt Anzahl geschlossener Benachrichtigungen zurück. |
| subscribe\_to\_events | pub fn subscribe\_to\_events(\&self) \-\> broadcast::Receiver\<NotificationEvent\> | Gibt einen Receiver für NotificationEvents zurück, um auf Benachrichtigungs-Events zu reagieren. |

### **4.3.4. Interne Events (NotificationEvent)**

Definiert in domain/src/notifications\_core/events.rs. Diese Events werden über tokio::sync::broadcast 6 verteilt, was eine entkoppelte Kommunikation innerhalb des Systems ermöglicht.

* **Zweck:** Andere Teile des Systems (primär die UI-Schicht über Adaptoren in der Systemschicht, aber auch andere Domänenmodule oder Logging-Dienste) über signifikante Änderungen im Benachrichtigungssystem zu informieren.  
* **Warum wertvoll:** Entkoppelte Kommunikation ist ein Schlüsselprinzip für modulare und wartbare Systeme. Die UI muss nicht direkt vom NotificationCoreManager aufgerufen werden; sie reagiert stattdessen auf Events.

Rust

// domain/src/notifications\_core/events.rs  
use crate::notifications\_core::types::{Notification, NotificationId};  
use chrono::{DateTime, Utc}; // Für Zeitstempel in Events

\# // Clone für Sender, PartialEq für Tests, Serde für ggf. externe Weiterleitung  
pub enum CloseReason {  
    DismissedByUser,  
    Expired,  
    Replaced,  
    AppClosed,      // App hat explizit CloseNotification gerufen  
    SystemShutdown,  
    AppScopeClear,  // Durch clear\_app\_notifications  
    Other(String),  
}

\# // Clone für Sender, Serde für ggf. externe Weiterleitung  
pub enum NotificationEvent {  
    NotificationAdded {  
        notification: Notification, // Die tatsächlich hinzugefügte (ggf. modifizierte) Notification  
        timestamp: DateTime\<Utc\>,  
    },  
    NotificationUpdated { // Falls Benachrichtigungen aktualisiert werden können (z.B. Fortschritt)  
        notification: Notification, // Die aktualisierte Notification  
        timestamp: DateTime\<Utc\>,  
    },  
    NotificationClosed {  
        notification\_id: NotificationId,  
        app\_name: String, // Nützlich für UI, um schnell zuordnen zu können  
        summary: String,  // Nützlich für UI  
        reason: CloseReason,  
        timestamp: DateTime\<Utc\>,  
    },  
    NotificationActionInvoked {  
        notification\_id: NotificationId,  
        action\_key: String,  
        timestamp: DateTime\<Utc\>,  
    },  
    NotificationHistoryCleared {  
        timestamp: DateTime\<Utc\>,  
    },  
    NotificationSuppressedByRule {  
        original\_summary: String, // Nur einige Infos, nicht die ganze Notification  
        app\_name: String,  
        rule\_id: String, // ID der verantwortlichen Regel  
        timestamp: DateTime\<Utc\>,  
    }  
}

* **Tabelle: NotificationEvent Varianten**

| Variante | Payload-Felder | Beschreibung |
| :---- | :---- | :---- |
| NotificationAdded | notification: Notification, timestamp | Eine neue Benachrichtigung wurde dem System hinzugefügt und ist (nach Regelprüfung) aktiv. |
| NotificationUpdated | notification: Notification, timestamp | Eine bestehende aktive Benachrichtigung wurde aktualisiert (z.B. Fortschrittsbalken). |
| NotificationClosed | notification\_id: NotificationId, app\_name, summary, reason: CloseReason, timestamp | Eine aktive Benachrichtigung wurde geschlossen. app\_name und summary für leichtere UI-Verarbeitung. |
| NotificationActionInvoked | notification\_id: NotificationId, action\_key: String, timestamp | Eine Aktion einer Benachrichtigung wurde ausgelöst. |
| NotificationHistoryCleared | timestamp | Die Benachrichtigungshistorie wurde geleert. |
| NotificationSuppressedByRule | original\_summary: String, app\_name: String, rule\_id: String, timestamp | Eine eingehende Benachrichtigung wurde aufgrund einer Regel unterdrückt und nicht aktiv angezeigt. |

* **Typische Publisher:** NotificationCoreManager.  
* **Typische Subscriber:** Die UI-Schicht (über einen Adapter in der Systemschicht, der D-Bus-Signale oder Wayland-Events generiert), Logging-Dienste, potenziell andere Domänenmodule, die auf Benachrichtigungsstatus reagieren müssen.

### **4.3.5. Fehlerbehandlung (NotificationCoreError)**

Definiert in domain/src/notifications\_core/error.rs mit thiserror.9

Rust

// domain/src/notifications\_core/error.rs  
use thiserror::Error;  
use crate::notifications\_core::types::NotificationId;  
use crate::notifications\_rules::error::NotificationRulesError;

\#  
pub enum NotificationCoreError {  
    \#  
    NotificationNotFound(NotificationId),

    \#\[error("Action '{action\_key}' not found for notification '{notification\_id}'.")\]  
    ActionNotFound {  
        notification\_id: NotificationId,  
        action\_key: String,  
    },

    \#\[error("Failed to apply notification rules: {source}")\]  
    RuleApplicationError {  
        \#\[from\] // Direkte Konvertierung von NotificationRulesError  
        source: NotificationRulesError  
    },

    \#\[error("Notification history is full (max size: {max\_size}). Cannot add notification '{summary}'.")\]  
    HistoryFull { max\_size: usize, summary: String },

    \#\[error("Invalid notification data: {message}")\]  
    InvalidNotificationData { message: String },

    \#\[error("Event channel error: {message}")\]  
    EventChannelError { message: String },

    \#  
    DuplicateNotificationId(NotificationId),

    \#  
    ReplacedNotificationNotFound(NotificationId),  
}

* **Tabelle: NotificationCoreError Varianten**

| Variante | Beschreibung |
| :---- | :---- |
| NotificationNotFound | Eine angeforderte Benachrichtigung (aktiv) wurde nicht gefunden. |
| ActionNotFound | Eine angeforderte Aktion für eine Benachrichtigung existiert nicht. |
| RuleApplicationError | Fehler bei der Anwendung von Regeln aus NotificationRulesEngine. Nutzt \#\[from\] für direkte Konvertierung. |
| HistoryFull | Die Benachrichtigungshistorie hat ihre maximale Kapazität erreicht und eine weitere kann nicht hinzugefügt werden. |
| InvalidNotificationData | Die Daten der hinzuzufügenden Benachrichtigung sind ungültig (z.B. fehlender summary). |
| EventChannelError | Fehler beim Senden eines NotificationEvent über den broadcast::Sender. |
| DuplicateNotificationId | Versuch, eine Benachrichtigung mit einer bereits existierenden ID zu den aktiven Benachrichtigungen hinzuzufügen. |
| ReplacedNotificationNotFound | Die in replaces\_id angegebene Benachrichtigung wurde nicht gefunden. |

### **4.3.6. Detaillierte Implementierungsschritte und Algorithmen**

1. **NotificationCoreManager::add\_notification:**  
   * Validiere new\_notification (z.B. app\_name, summary nicht leer, id muss gesetzt sein). Bei Fehler: Err(NotificationCoreError::InvalidNotificationData).  
   * Erwirb Schreibsperre für active\_notifications.  
   * Wenn new\_notification.id bereits in active\_notifications existiert: Err(NotificationCoreError::DuplicateNotificationId).  
   * **Regelanwendung:** Rufe self.rules\_engine.process\_notification(\&new\_notification).await auf.  
     * Bei Err(rules\_error): Err(NotificationCoreError::from(rules\_error)).  
     * Bei Ok(RuleProcessingResult::Suppress(rule\_id)):  
       * Sende NotificationSuppressedByRule Event.  
       * Die Benachrichtigung wird nicht aktiv. Ggf. zur Historie hinzufügen, falls die Regel dies impliziert oder new\_notification.is\_persistent ist (abhängig von Designentscheidung).  
       * Ok(new\_notification.id) zurückgeben (die ID der ursprünglichen, nun unterdrückten Benachrichtigung).  
     * Bei Ok(RuleProcessingResult::Allow(mut processed\_notification)):  
       * processed\_notification.timestamp\_displayed \= Some(Utc::now()).  
       * **Ersetzungslogik:** Wenn processed\_notification.replaces\_id ein Some(id\_to\_replace) ist:  
         * Versuche, die Benachrichtigung mit id\_to\_replace aus active\_notifications zu entfernen.  
         * Wenn erfolgreich entfernt, sende NotificationClosed Event für id\_to\_replace mit CloseReason::Replaced.  
         * Wenn nicht gefunden: Err(NotificationCoreError::ReplacedNotificationNotFound(id\_to\_replace)).  
       * Füge processed\_notification.clone() zu active\_notifications hinzu (mit ihrer eigenen id).  
       * Sende NotificationAdded { notification: processed\_notification.clone(),... } Event.  
       * Ok(processed\_notification.id).  
2. **NotificationCoreManager::close\_notification:**  
   * Erwirb Schreibsperren für active\_notifications und history.  
   * Entferne Benachrichtigung mit id aus active\_notifications. Wenn nicht gefunden: Err(NotificationCoreError::NotificationNotFound(id)).  
   * Sei closed\_notification die entfernte Benachrichtigung.  
   * Wenn closed\_notification.is\_persistent oder reason dies nahelegt (z.B. DismissedByUser, aber nicht Expired wenn nicht persistent):  
     * Rufe history.write().await.add(closed\_notification.clone()) auf. Handle HistoryFull Fehler, falls add diesen zurückgeben kann (oder logge es).  
   * Sende NotificationClosed { notification\_id: id, app\_name: closed\_notification.app\_name, summary: closed\_notification.summary, reason,... } Event.  
   * Ok(()).  
3. **NotificationHistory::add (interne Methode von NotificationHistory):**  
   * Wenn self.notifications.len() \>= self.max\_size und self.max\_size \> 0:  
     * self.notifications.pop\_front() (entferne die älteste).  
   * self.notifications.push\_back(notification).  
4. **NotificationCoreManager::invoke\_action:**  
   * Erwirb Lesesperre für active\_notifications.  
   * Hole Benachrichtigung mit notification\_id. Wenn nicht gefunden: Err(NotificationCoreError::NotificationNotFound).  
   * Prüfe, ob die Aktion mit action\_key in notification.actions existiert. Wenn nicht: Err(NotificationCoreError::ActionNotFound).  
   * Sende NotificationActionInvoked { notification\_id, action\_key: action\_key.to\_string(),... } Event.  
   * Ok(()). (Die Domänenschicht löst nur das Event aus; die tatsächliche Aktionsausführung erfolgt in höheren Schichten oder der Anwendung selbst).

### **4.3.7. Überlegungen zur Nebenläufigkeit und Zustandssynchronisierung**

* active\_notifications und history (bzw. dessen interne VecDeque) benötigen tokio::sync::RwLock für Thread-sicheren Lese- und Schreibzugriff, da mehrere Tasks (z.B. durch D-Bus-Aufrufe oder interne Timer) gleichzeitig auf Benachrichtigungen zugreifen könnten.  
* Die rules\_engine wird als Arc\<NotificationRulesEngine\> übergeben, da sie von mehreren Aufrufen (z.B. für jede neue Benachrichtigung) nebenläufig genutzt werden kann und ihr Zustand (die Regeln) ebenfalls Thread-sicher sein muss.  
* Der broadcast::Sender für NotificationEvent ist inhärent Thread-sicher.13

## **4.4. Entwicklungsmodul: Priorisierung und Regel-Engine für Benachrichtigungen (domain::notifications\_rules)**

Dieses Modul implementiert die Logik zur dynamischen Verarbeitung von Benachrichtigungen basierend auf einem Satz von konfigurierbaren Regeln.

### **4.4.1. Detaillierte Verantwortlichkeiten und Ziele**

* **Verantwortlichkeiten:**  
  * Definition der Struktur von Benachrichtigungsregeln (NotificationRule), deren Bedingungen (RuleCondition) und Aktionen (RuleAction).  
  * Bereitstellung einer Engine (NotificationRulesEngine), die eingehende Benachrichtigungen anhand dieser Regeln bewertet.  
  * Ermöglichung von Modifikationen an Benachrichtigungen durch Regeln (z.B. Dringlichkeit ändern, Ton festlegen, Aktionen hinzufügen).  
  * Ermöglichung der Unterdrückung von Benachrichtigungen basierend auf Regelbedingungen.  
  * Interaktion mit domain::settings\_core (durch Empfang von SettingChangedEvents und Abfrage von Einstellungswerten), um kontextsensitive Regeln zu ermöglichen (z.B. "Nicht stören"-Modus, anwendungsspezifische Stummschaltungen).  
  * Laden und Verwalten von Regeldefinitionen. Diese können initial fest kodiert sein, sollten aber idealerweise aus einer externen Konfiguration (z.B. via SettingsProvider) geladen werden können, um Flexibilität zu gewährleisten.  
* **Ziele:**  
  * Schaffung einer flexiblen und erweiterbaren Logik zur dynamischen Anpassung des Benachrichtigungsverhaltens.  
  * Ermöglichung einer feingranularen Steuerung des Benachrichtigungsflusses durch den Benutzer (implizit über Systemeinstellungen) oder durch Systemadministratoren.  
  * Reduzierung von "Notification Fatigue" durch intelligente Filterung und Priorisierung.

Ein wichtiger Aspekt beim Design der Regel-Engine ist die Frage, ob Regeln fest im Code verankert oder datengetrieben (z.B. aus einer Konfigurationsdatei) sind. Ein datengetriebener Ansatz erhöht die Flexibilität und Wartbarkeit erheblich, da Regeln ohne Neukompilierung des Systems geändert oder hinzugefügt werden können. Dies erfordert, dass die Regelstrukturen (NotificationRule, RuleCondition, RuleAction) serde::Serialize und serde::Deserialize implementieren. Selbst wenn die erste Version mit fest kodierten Regeln startet, sollte das Design eine spätere Umstellung ermöglichen.

### **4.4.2. Entitäten und Wertobjekte**

Alle Typen sind in domain/src/notifications\_rules/types.rs zu definieren. Sie benötigen Debug, Clone, PartialEq und, für datengetriebene Regeln, serde::Serialize und serde::Deserialize.

* **RuleCondition (Enum, Wertobjekt)**  
  * **Zweck:** Definiert die Bedingungen, die erfüllt sein müssen, damit eine Regel ausgelöst wird.  
  * **Warum wertvoll:** Ermöglicht die flexible und kompositorische Definition von Kriterien für Regeln, von einfachen Vergleichen bis zu komplexen logischen Verknüpfungen.

| Variante | Assoziierte Daten | Beschreibung |
| :---- | :---- | :---- |
| AppNameIs | String | Der app\_name der Benachrichtigung entspricht exakt dem Wert (case-sensitive). |
| AppNameMatches | String (als Regex-Pattern zu interpretieren) | Der app\_name der Benachrichtigung entspricht dem regulären Ausdruck. |
| SummaryContains | String | Der summary der Benachrichtigung enthält den Text (case-insensitive). |
| SummaryMatches | String (Regex-Pattern) | Der summary der Benachrichtigung entspricht dem regulären Ausdruck. |
| BodyContains | String | Der body der Benachrichtigung (falls vorhanden) enthält den Text (case-insensitive). |
| UrgencyIs | NotificationUrgency | Die urgency der Benachrichtigung entspricht dem Wert. |
| CategoryIs | String | Die category der Benachrichtigung (falls vorhanden) entspricht exakt dem Wert. |
| HintExists | String (Schlüssel des Hints) | Ein bestimmter Schlüssel existiert in den hints der Benachrichtigung. |
| HintValueIs | (String (Hint-Schlüssel), SettingValue (erwarteter Wert)) | Ein bestimmter Hint-Schlüssel existiert und sein Wert entspricht dem SettingValue. |
| SettingIsTrue | SettingKey (Schlüssel zu einer Boolean-Einstellung) | Eine globale Systemeinstellung (aus SettingsCoreManager) ist auf true gesetzt. |
| SettingIsFalse | SettingKey (Schlüssel zu einer Boolean-Einstellung) | Eine globale Systemeinstellung ist auf false gesetzt. |
| SettingValueEquals | (SettingKey, SettingValue) | Eine globale Systemeinstellung hat exakt den spezifizierten Wert. |
| LogicalAnd | Vec\<RuleCondition\> | Alle Unterbedingungen in der Liste müssen wahr sein. |
| LogicalOr | Vec\<RuleCondition\> | Mindestens eine der Unterbedingungen in der Liste muss wahr sein. |
| LogicalNot | Box\<RuleCondition\> | Die umschlossene Unterbedingung muss falsch sein. |

* **RuleAction (Enum, Wertobjekt)**  
  * **Zweck:** Definiert die Aktionen, die ausgeführt werden, wenn die Bedingungen einer Regel erfüllt sind.  
  * **Warum wertvoll:** Beschreibt, wie eine Benachrichtigung als Reaktion auf eine Regel modifiziert oder behandelt wird.

| Variante | Assoziierte Daten | Beschreibung |
| :---- | :---- | :---- |
| SuppressNotification | \- | Unterdrückt die Benachrichtigung vollständig. Sie wird nicht aktiv und typischerweise auch nicht in der Historie gespeichert. |
| SetUrgency | NotificationUrgency | Ändert die urgency der Benachrichtigung auf den neuen Wert. |
| AddAction | NotificationAction | Fügt eine zusätzliche NotificationAction zur Liste der Aktionen der Benachrichtigung hinzu. |
| SetHint | (String (Hint-Schlüssel), SettingValue (Wert)) | Setzt oder überschreibt einen Wert in den hints der Benachrichtigung. |
| PlaySound | Option\<String\> (Sound-Datei/Name oder Event-Name) | Signalisiert, dass ein Ton abgespielt werden soll. None für einen Standard-Benachrichtigungston, Some(name) für einen spezifischen Ton. Die Implementierung des Abspielens erfolgt in der System- oder UI-Schicht. |
| MarkAsPersistent | bool | Setzt das is\_persistent-Flag der Benachrichtigung. |
| SetExpiration | Option\<i64\> (Millisekunden relativ zu jetzt) | Setzt oder ändert die Ablaufzeit der Benachrichtigung. None entfernt eine existierende Ablaufzeit. Ein positiver Wert gibt die Dauer in ms an. |
| LogMessage | (String (Level: "info", "warn", "debug"), String (Nachricht)) | Schreibt eine Nachricht ins System-Log (über das tracing-Framework). Nützlich für das Debugging von Regeln. |

* **NotificationRule (Struktur, Entität)**  
  * **Zweck:** Repräsentiert eine einzelne, vollständige Regel mit Bedingungen und Aktionen.  
  * **Warum wertvoll:** Die atomaren Bausteine der Regel-Engine. Eine Sammlung dieser Regeln definiert das Verhalten des Benachrichtigungssystems.

| Attribut | Typ | Sichtbarkeit | Beschreibung |
| :---- | :---- | :---- | :---- |
| id | String | pub | Eindeutige, menschenlesbare ID der Regel (z.B. "suppress-low-priority-chat", "urgentify-calendar-reminders"). |
| description | Option\<String\> | pub | Optionale, menschenlesbare Beschreibung des Zwecks der Regel. |
| conditions | RuleCondition | pub | Die Bedingung(en), die erfüllt sein müssen, damit die Regel angewendet wird. Oft eine LogicalAnd oder LogicalOr. |
| actions | Vec\<RuleAction\> | pub | Die Liste der Aktionen, die ausgeführt werden, wenn die conditions zutreffen. Die Reihenfolge kann relevant sein. |
| is\_enabled | bool | pub | Gibt an, ob die Regel aktiv ist und ausgewertet werden soll. Default: true. |
| priority | i32 | pub | Priorität der Regel. Regeln mit höherem Wert werden typischerweise früher ausgewertet. Default: 0\. |
| stop\_after | bool | pub | Wenn true und diese Regel zutrifft und Aktionen ausführt, werden keine weiteren (niedriger priorisierten) Regeln für diese Benachrichtigung mehr ausgewertet. Default: false. |

### **4.4.3. Öffentliche API des Moduls (NotificationRulesEngine)**

Definiert in domain/src/notifications\_rules/mod.rs.

Rust

// domain/src/notifications\_rules/mod.rs  
use crate::notifications\_core::types::{Notification, NotificationUrgency, SettingValue as NotificationSettingValue}; // SettingValue hier umbenannt zur Klarheit  
use crate::notifications\_rules::types::{NotificationRule, RuleCondition, RuleAction, NotificationAction as RuleNotificationAction};  
use crate::notifications\_rules::error::NotificationRulesError;  
use crate::settings\_core::{SettingsCoreManager, SettingChangedEvent, SettingKey, SettingValue};  
use std::sync::Arc;  
use tokio::sync::{RwLock, broadcast::Receiver as BroadcastReceiver}; // Receiver explizit benannt  
use tracing; // Für LogMessage Aktion

\#  
pub enum RuleProcessingResult {  
    Allow(Notification),  
    Suppress(String), // Enthält die ID der Regel, die zur Unterdrückung geführt hat  
}

pub struct NotificationRulesEngine {  
    rules: RwLock\<Vec\<NotificationRule\>\>,  
    settings\_manager: Arc\<SettingsCoreManager\>,  
    // settings\_update\_receiver: RwLock\<Option\<BroadcastReceiver\<SettingChangedEvent\>\>\>, // Für das Lauschen auf Einstellungsänderungen  
}

impl NotificationRulesEngine {  
    pub fn new(  
        settings\_manager: Arc\<SettingsCoreManager\>,  
        initial\_rules: Vec\<NotificationRule\>,  
        // mut settings\_event\_receiver: BroadcastReceiver\<SettingChangedEvent\> // Wird übergeben  
    ) \-\> Arc\<Self\> { // Gibt Arc\<Self\> zurück, um das Klonen für den Listener-Task zu erleichtern  
        let mut sorted\_rules \= initial\_rules;  
        sorted\_rules.sort\_by\_key(|r| \-r.priority); // Höchste Priorität zuerst

        let engine \= Arc::new(NotificationRulesEngine {  
            rules: RwLock::new(sorted\_rules),  
            settings\_manager,  
            // settings\_update\_receiver: RwLock::new(Some(settings\_event\_receiver)),  
        });

        // Hier könnte ein Task gestartet werden, der auf settings\_event\_receiver lauscht  
        // und self.handle\_setting\_changed aufruft.  
        // let engine\_clone \= Arc::clone(\&engine);  
        // tokio::spawn(async move {  
        //     if let Some(mut rx) \= engine\_clone.settings\_update\_receiver.write().await.take() {  
        //         while let Ok(event) \= rx.recv().await {  
        //             engine\_clone.handle\_setting\_changed(\&event).await;  
        //         }  
        //     }  
        // });

        engine  
    }

    pub async fn load\_rules(\&self, new\_rules: Vec\<NotificationRule\>) {  
        let mut rules\_guard \= self.rules.write().await;  
        \*rules\_guard \= new\_rules;  
        rules\_guard.sort\_by\_key(|r| \-r.priority); // Höchste Priorität zuerst  
        tracing::info\!("Notification rules reloaded. {} rules active.", rules\_guard.len());  
    }

    pub async fn process\_notification(  
        \&self,  
        notification: \&Notification,  
    ) \-\> Result\<RuleProcessingResult, NotificationRulesError\> {  
        let rules\_guard \= self.rules.read().await;  
        let mut current\_notification \= notification.clone();  
        let mut suppressed\_by\_rule\_id: Option\<String\> \= None;

        for rule in rules\_guard.iter().filter(|r| r.is\_enabled) {  
            if self.evaluate\_condition(\&rule.conditions, \&current\_notification, rule).await? {  
                tracing::debug\!("Rule '{}' matched for notification '{}'", rule.id, notification.summary);  
                for action in \&rule.actions {  
                    match self.apply\_action(action, \&mut current\_notification, rule).await? {  
                        RuleProcessingResult::Suppress(\_) \=\> {  
                            suppressed\_by\_rule\_id \= Some(rule.id.clone());  
                            break; // Aktion "Suppress" beendet Aktionsschleife für diese Regel  
                        }  
                        RuleProcessingResult::Allow(modified\_notification) \=\> {  
                            current\_notification \= modified\_notification;  
                        }  
                    }  
                }  
                if suppressed\_by\_rule\_id.is\_some() |  
| rule.stop\_after {  
                    break; // Regelverarbeitung für diese Benachrichtigung beenden  
                }  
            }  
        }

        if let Some(rule\_id) \= suppressed\_by\_rule\_id {  
            Ok(RuleProcessingResult::Suppress(rule\_id))  
        } else {  
            Ok(RuleProcessingResult::Allow(current\_notification))  
        }  
    }

    async fn evaluate\_condition(  
        \&self,  
        condition: \&RuleCondition,  
        notification: \&Notification,  
        rule: \&NotificationRule, // Für Kontext in Fehlermeldungen  
    ) \-\> Result\<bool, NotificationRulesError\> {  
        match condition {  
            RuleCondition::AppNameIs(name) \=\> Ok(\&notification.app\_name \== name),  
            RuleCondition::AppNameMatches(pattern) \=\> {  
                // Hier Regex-Implementierung, z.B. mit \`regex\` Crate  
                // Für dieses Beispiel: einfache Prüfung  
                match regex::Regex::new(pattern) {  
                    Ok(re) \=\> Ok(re.is\_match(\&notification.app\_name)),  
                    Err(e) \=\> Err(NotificationRulesError::ConditionEvaluationError{ rule\_id: Some(rule.id.clone()), message: format\!("Invalid regex pattern '{}': {}", pattern, e) })  
                }  
            }  
            RuleCondition::SummaryContains(text) \=\> Ok(notification.summary.to\_lowercase().contains(\&text.to\_lowercase())),  
            //... Implementierung für alle RuleCondition-Varianten...  
            RuleCondition::SettingIsTrue(key) \=\> {  
                match self.settings\_manager.get\_setting\_value(key).await {  
                    Ok(SettingValue::Boolean(b)) \=\> Ok(b),  
                    Ok(other\_type) \=\> {  
                        tracing::warn\!("Rule '{}' expected boolean for setting '{}', got {:?}", rule.id, key.as\_str(), other\_type);  
                        Ok(false) // Falscher Typ, als false bewerten  
                    }  
                    Err(SettingsCoreError::SettingNotFound{..}) | Err(SettingsCoreError::UnregisteredKey{..}) \=\> {  
                        tracing::debug\!("Rule '{}': Setting '{}' not found or unregistered, condition evaluates to false.", rule.id, key.as\_str());  
                        Ok(false) // Einstellung nicht gefunden, als false bewerten  
                    }  
                    Err(e) \=\> Err(NotificationRulesError::SettingsAccessError(e)) // Anderer Fehler beim Holen  
                }  
            }  
            RuleCondition::LogicalAnd(sub\_conditions) \=\> {  
                for sub\_cond in sub\_conditions {  
                    if\!self.evaluate\_condition(sub\_cond, notification, rule).await? {  
                        return Ok(false);  
                    }  
                }  
                Ok(true)  
            }  
            RuleCondition::LogicalOr(sub\_conditions) \=\> {  
                for sub\_cond in sub\_conditions {  
                    if self.evaluate\_condition(sub\_cond, notification, rule).await? {  
                        return Ok(true);  
                    }  
                }  
                Ok(false)  
            }  
            RuleCondition::LogicalNot(sub\_condition) \=\> {  
                Ok(\!self.evaluate\_condition(sub\_condition, notification, rule).await?)  
            }  
            // Standard-Fallback für nicht implementierte Bedingungen (sollte nicht passieren bei vollständiger Impl.)  
            \_ \=\> {  
                tracing::warn\!("Unimplemented condition met in rule '{}': {:?}", rule.id, condition);  
                Ok(false)  
            }  
        }  
    }

    async fn apply\_action(  
        \&self,  
        action: \&RuleAction,  
        notification: \&mut Notification,  
        rule: \&NotificationRule, // Für Kontext  
    ) \-\> Result\<RuleProcessingResult, NotificationRulesError\> {  
        tracing::debug\!("Applying action {:?} from rule '{}' to notification '{}'", action, rule.id, notification.summary);  
        match action {  
            RuleAction::SuppressNotification \=\> return Ok(RuleProcessingResult::Suppress(rule.id.clone())),  
            RuleAction::SetUrgency(new\_urgency) \=\> notification.urgency \= \*new\_urgency,  
            RuleAction::AddAction(new\_action) \=\> {  
                // Prüfen, ob Aktion mit gleichem Key schon existiert, um Duplikate zu vermeiden  
                if\!notification.actions.iter().any(|a| a.key \== new\_action.key) {  
                    notification.actions.push(new\_action.clone());  
                }  
            }  
            RuleAction::SetHint((key, value)) \=\> {  
                notification.hints.insert(key.clone(), value.clone().into\_setting\_value()); // Annahme: value ist hier ein Domänen-SettingValue  
            }  
            RuleAction::PlaySound(sound\_name\_opt) \=\> {  
                // Diese Aktion setzt typischerweise einen Hint, den die UI/Systemschicht interpretiert  
                let hint\_key \= "sound-name".to\_string();  
                if let Some(sound\_name) \= sound\_name\_opt {  
                    notification.hints.insert(hint\_key, NotificationSettingValue::String(sound\_name.clone()));  
                } else {  
                    // Signal für Standardton, z.B. spezieller Wert oder Entfernen des Hints  
                    notification.hints.remove(\&hint\_key);  
                }  
            }  
            RuleAction::MarkAsPersistent(is\_persistent) \=\> notification.is\_persistent \= \*is\_persistent,  
            RuleAction::SetExpiration(duration\_ms\_opt) \=\> {  
                if let Some(duration\_ms) \= duration\_ms\_opt {  
                    if \*duration\_ms \> 0 {  
                        notification.expires\_at \= Some(Utc::now() \+ chrono::Duration::milliseconds(\*duration\_ms));  
                    } else {  
                        notification.expires\_at \= None; // Negative oder Null-Dauer entfernt Expiration  
                    }  
                } else {  
                    notification.expires\_at \= None;  
                }  
            }  
            RuleAction::LogMessage((level, message)) \=\> {  
                let full\_message \= format\!(" {}", rule.id, message);  
                match level.as\_str() {  
                    "info" \=\> tracing::info\!("{}", full\_message),  
                    "warn" \=\> tracing::warn\!("{}", full\_message),  
                    "debug" \=\> tracing::debug\!("{}", full\_message),  
                    \_ \=\> tracing::trace\!("{}", full\_message), // Default zu trace  
                }  
            }  
        }  
        Ok(RuleProcessingResult::Allow(notification.clone()))  
    }

    // Diese Methode wird aufgerufen, wenn ein SettingChangedEvent empfangen wird.  
    // Sie könnte z.B. einen internen Cache für Settings aktualisieren, falls verwendet,  
    // oder Regeln neu bewerten, die von dieser Einstellung abhängen (komplexer).  
    // Für eine einfache Implementierung ohne Cache ist diese Methode ggf. leer  
    // oder löst nur einen Log-Eintrag aus.  
    pub async fn handle\_setting\_changed(\&self, event: \&SettingChangedEvent) {  
        tracing::debug\!("NotificationRulesEngine received SettingChangedEvent for key: {}", event.key.as\_str());  
        // Hier könnte Logik stehen, um z.B. interne Caches zu invalidieren,  
        // falls die Performance der direkten Abfrage des SettingsCoreManager ein Problem darstellt.  
        // Für die meisten Fälle sollte die direkte Abfrage bei Bedarf ausreichend sein.  
    }  
}

// Hilfskonvertierung für RuleAction::SetHint, falls SettingValue aus notifications\_rules::types  
// und settings\_core::types nicht identisch sind (sollten sie aber sein).  
// Hier wird angenommen, dass SettingValue aus settings\_core verwendet wird.  
trait IntoSettingValue {  
    fn into\_setting\_value(self) \-\> SettingValue;  
}  
impl IntoSettingValue for NotificationSettingValue { // Hier NotificationSettingValue ist Alias für settings\_core::SettingValue  
    fn into\_setting\_value(self) \-\> SettingValue {  
        self // Direkte Konvertierung, da Typen identisch sein sollten  
    }  
}

(Hinweis: Die regex-Crate müsste als Abhängigkeit hinzugefügt werden. Der Listener-Task für Einstellungsänderungen ist auskommentiert, da seine Implementierung von der genauen Architektur des Event-Handlings abhängt und den Rahmen sprengen könnte, aber das Prinzip ist wichtig.)

* **Tabelle: Methoden der NotificationRulesEngine**

| Methode | Signatur | Kurzbeschreibung |
| :---- | :---- | :---- |
| new | pub fn new(settings\_manager: Arc\<SettingsCoreManager\>, initial\_rules: Vec\<NotificationRule\>/\*, settings\_event\_receiver: BroadcastReceiver\<SettingChangedEvent\>\*/) \-\> Arc\<Self\> | Konstruktor. Lädt initiale Regeln, sortiert sie nach Priorität. Speichert Referenz auf SettingsCoreManager. Startet optional einen Task, um auf SettingChangedEvents zu lauschen. Gibt Arc\<Self\> zurück. |
| load\_rules | pub async fn load\_rules(\&self, new\_rules: Vec\<NotificationRule\>) | Lädt einen neuen Satz von Regeln, ersetzt die alten und sortiert sie neu nach Priorität. |
| process\_notification | pub async fn process\_notification(\&self, notification: \&Notification) \-\> Result\<RuleProcessingResult, NotificationRulesError\> | Verarbeitet eine eingehende Benachrichtigung anhand der geladenen, aktivierten Regeln. Gibt entweder eine (potenziell modifizierte) Benachrichtigung (Allow) oder ein Signal zur Unterdrückung (Suppress) mit der verantwortlichen Regel-ID zurück. |
| handle\_setting\_changed | pub async fn handle\_setting\_changed(\&self, event: \&SettingChangedEvent) | Wird (intern, z.B. durch einen dedizierten Task) aufgerufen, wenn sich eine für Regeln relevante Systemeinstellung ändert. Ermöglicht der Engine, ihren Zustand oder ihr Verhalten anzupassen (z.B. Cache-Invalidierung). |

Die Entscheidung, SettingValue aus settings\_core auch in den RuleCondition und RuleAction zu verwenden, vereinfacht die Typisierung und vermeidet unnötige Konvertierungen.

### **4.4.4. Fehlerbehandlung (NotificationRulesError)**

Definiert in domain/src/notifications\_rules/error.rs mit thiserror.

Rust

// domain/src/notifications\_rules/error.rs  
use thiserror::Error;  
use crate::settings\_core::error::SettingsCoreError; // Für Fehler beim Zugriff auf Settings

\#  
pub enum NotificationRulesError {  
    \#  
    InvalidRuleDefinition {  
        rule\_id: Option\<String\>,  
        message: String,  
    },

    \#\[error("Failed to evaluate condition for rule '{rule\_id:?}': {message}")\]  
    ConditionEvaluationError {  
        rule\_id: Option\<String\>,  
        message: String,  
    },

    \#\[error("Failed to apply action for rule '{rule\_id:?}': {message}")\]  
    ActionApplicationError {  
        rule\_id: Option\<String\>,  
        message: String,  
    },

    \#\[error("Error accessing settings for rule evaluation: {source}")\]  
    SettingsAccessError{  
        \#\[from\] // Direkte Konvertierung von SettingsCoreError  
        source: SettingsCoreError  
    },

    \# // Wird intern verwendet, falls Regeln auf andere verweisen  
    RuleNotFound(String),  
}

* **Tabelle: NotificationRulesError Varianten**

| Variante | Beschreibung |
| :---- | :---- |
| InvalidRuleDefinition | Eine geladene Regel ist ungültig (z.B. fehlerhaftes Regex-Pattern in AppNameMatches, widersprüchliche Bedingungen, unbekannte Aktionstypen). |
| ConditionEvaluationError | Ein Fehler trat während der Auswertung einer Bedingung auf (z.B. Regex-Kompilierungsfehler, interner Logikfehler). |
| ActionApplicationError | Ein Fehler trat während der Anwendung einer Aktion auf (z.B. ungültige Parameter für eine Aktion). |
| SettingsAccessError | Fehler beim Zugriff auf SettingsCoreManager für die Auswertung von Bedingungen, die auf Systemeinstellungen basieren. Nutzt \#\[from\]. |
| RuleNotFound | Eine referenzierte Regel-ID (z.B. in einer komplexen Regelstruktur) existiert nicht. |

### **4.4.5. Detaillierte Implementierungsschritte und Algorithmen**

1. **Initialisierung (NotificationRulesEngine::new):**  
   * Speichere den Arc\<SettingsCoreManager\>.  
   * Lade die initial\_rules.  
   * Sortiere die Regeln nach priority (absteigend, d.h. höhere numerische Werte zuerst) und dann ggf. nach id für deterministische Reihenfolge bei gleicher Priorität.  
   * **Abonnement von Einstellungsänderungen:** Es ist entscheidend, dass die Regel-Engine auf Änderungen von Systemeinstellungen reagieren kann, die in RuleConditions verwendet werden (z.B. "Nicht stören"-Modus).  
     * Der NotificationRulesEngine sollte beim Erstellen einen broadcast::Receiver\<SettingChangedEvent\> vom SettingsCoreManager erhalten (oder der SettingsCoreManager registriert die Engine als Listener).  
     * Ein dedizierter tokio::task sollte gestartet werden, der diesen Receiver konsumiert. Bei Empfang eines SettingChangedEvent ruft dieser Task engine.handle\_setting\_changed(\&event).await auf.  
     * handle\_setting\_changed kann dann z.B. einen internen Cache von oft benötigten Einstellungswerten invalidieren oder aktualisieren, um zu vermeiden, dass für jede Regelauswertung der SettingsCoreManager abgefragt werden muss (Performance-Optimierung, falls nötig). Für den Anfang kann es ausreichen, dass evaluate\_condition immer live den SettingsCoreManager abfragt.  
2. **NotificationRulesEngine::process\_notification:**  
   * Erwirb eine Lesesperre auf self.rules.  
   * Klone die eingehende notification, um Modifikationen zu ermöglichen (current\_notification).  
   * Iteriere durch die sortierten, aktivierten (rule.is\_enabled) Regeln.  
   * Für jede Regel:  
     * Evaluiere rule.conditions rekursiv mittels self.evaluate\_condition(\&rule.conditions, \&current\_notification, \&rule).await.  
     * Wenn die Bedingungen erfüllt sind (true):  
       * Iteriere durch rule.actions.  
       * Wende jede Aktion auf current\_notification an mittels self.apply\_action(action, \&mut current\_notification, \&rule).await.  
       * Wenn eine Aktion RuleProcessingResult::Suppress zurückgibt (z.B. RuleAction::SuppressNotification), speichere die rule.id und brich die Verarbeitung der Aktionen *dieser Regel* ab.  
       * Wenn RuleProcessingResult::Allow(modified\_notification) zurückgegeben wird, aktualisiere current\_notification \= modified\_notification.  
       * Wenn suppressed\_by\_rule\_id gesetzt wurde oder rule.stop\_after \== true ist, brich die Iteration über *weitere Regeln* ab.  
   * Wenn am Ende suppressed\_by\_rule\_id gesetzt ist, gib Ok(RuleProcessingResult::Suppress(rule\_id)) zurück.  
   * Andernfalls gib Ok(RuleProcessingResult::Allow(current\_notification)) zurück.  
3. **NotificationRulesEngine::evaluate\_condition (rekursiv):**  
   * Implementiere die Logik für jede RuleCondition-Variante:  
     * Einfache Vergleiche (AppNameIs, SummaryContains, UrgencyIs, etc.) sind direkte Vergleiche der Felder der notification.  
     * Regex-basierte Vergleiche (AppNameMatches, SummaryMatches) verwenden die regex-Crate. Fehler bei der Regex-Kompilierung (sollten idealerweise beim Laden der Regeln abgefangen werden) führen zu Err(NotificationRulesError::ConditionEvaluationError).  
     * HintExists, HintValueIs: Zugriff auf notification.hints.  
     * SettingIsTrue, SettingIsFalse, SettingValueEquals: Asynchroner Aufruf von self.settings\_manager.get\_setting\_value(\&key).await.  
       * Fehler wie SettingsCoreError::SettingNotFound oder UnregisteredKey sollten die Bedingung typischerweise als false bewerten lassen, anstatt einen harten Fehler in der Regel-Engine auszulösen, um die Robustheit zu erhöhen. Ein Log-Eintrag (Warnung oder Debug) ist hier angebracht. Andere SettingsCoreError (z.B. PersistenceError) sollten als Err(NotificationRulesError::SettingsAccessError) propagiert werden.  
     * LogicalAnd: Gibt true zurück, wenn alle Unterbedingungen true sind (Kurzschlussauswertung).  
     * LogicalOr: Gibt true zurück, wenn mindestens eine Unterbedingung true ist (Kurzschlussauswertung).  
     * LogicalNot: Negiert das Ergebnis der Unterbedingung.  
   * Alle Pfade müssen Result\<bool, NotificationRulesError\> zurückgeben.  
4. **NotificationRulesEngine::apply\_action:**  
   * Implementiere die Logik für jede RuleAction-Variante.  
   * Die meisten Aktionen modifizieren die übergebene \&mut Notification direkt (z.B. SetUrgency, AddAction, SetHint, MarkAsPersistent, SetExpiration).  
   * SuppressNotification gibt Ok(RuleProcessingResult::Suppress(...)) zurück.  
   * PlaySound könnte einen speziellen Hint setzen (z.B. sound-event: "message-new-instant"), den die UI-Schicht interpretiert.  
   * LogMessage verwendet das tracing-Makro (z.B. tracing::info\!).  
   * Alle Pfade geben Result\<RuleProcessingResult, NotificationRulesError\> zurück (meist Ok(RuleProcessingResult::Allow(notification.clone())) nach Modifikation).

### **4.4.6. Erweiterbarkeit und Konfiguration der Regeln**

* Um Regeln dynamisch (z.B. aus Konfigurationsdateien) laden zu können, müssen NotificationRule und alle eingebetteten Typen (RuleCondition, RuleAction) serde::Serialize und serde::Deserialize implementieren.  
* Die NotificationRulesEngine könnte eine Methode async fn load\_rules\_from\_provider(\&self, settings\_provider: Arc\<dyn SettingsProvider\>, config\_key: \&SettingKey) anbieten. Diese Methode würde:  
  1. Den settings\_provider verwenden, um eine serialisierte Regelmenge (z.B. als JSON-String oder eine Liste von serialisierten Regelobjekten) unter config\_key zu laden.  
  2. Die geladenen Daten deserialisieren in Vec\<NotificationRule\>.  
  3. Diese neuen Regeln über self.load\_rules(...) aktivieren.  
* Das Format der Serialisierung (z.B. JSON, YAML, TOML) muss sorgfältig entworfen werden, um sowohl menschenlesbar als auch maschinell verarbeitbar zu sein. Validierungsschemata (z.B. JSON Schema) können helfen, die Korrektheit der Regeldefinitionen sicherzustellen, bevor sie geladen werden.  
* Die Fehlerbehandlung beim Laden und Deserialisieren von Regeln muss robust sein (InvalidRuleDefinition).

Diese detaillierte Ausarbeitung der Einstellungs- und Benachrichtigungs-Subsysteme vervollständigt die Spezifikation der Domänenschicht und legt eine solide Grundlage für deren Implementierung. Die Betonung von klar definierten Schnittstellen, Typsicherheit, Fehlerbehandlung und Entkopplung durch Events und Abstraktionen ist entscheidend für die Entwicklung einer modernen, wartbaren und erweiterbaren Desktop-Umgebung.

#### **Referenzen**

1. uuid \- Rust, Zugriff am Mai 14, 2025, [https://messense.github.io/bosonnlp-rs/uuid/index.html](https://messense.github.io/bosonnlp-rs/uuid/index.html)  
2. Uuid in rocket::serde::uuid \- Rust, Zugriff am Mai 14, 2025, [https://api.rocket.rs/v0.5/rocket/serde/uuid/struct.Uuid](https://api.rocket.rs/v0.5/rocket/serde/uuid/struct.Uuid)  
3. chrono::serde \- Rust, Zugriff am Mai 14, 2025, [https://prisma.github.io/prisma-engines/doc/chrono/serde/index.html](https://prisma.github.io/prisma-engines/doc/chrono/serde/index.html)  
4. chrono \- crates.io: Rust Package Registry, Zugriff am Mai 14, 2025, [https://crates.io/crates/chrono](https://crates.io/crates/chrono)  
5. Arc in std::sync \- Rust Documentation, Zugriff am Mai 14, 2025, [https://doc.rust-lang.org/std/sync/struct.Arc.html](https://doc.rust-lang.org/std/sync/struct.Arc.html)  
6. Tokio: Channels \- oida.dev, Zugriff am Mai 14, 2025, [https://oida.dev/rust-tokio-guide/channels/](https://oida.dev/rust-tokio-guide/channels/)  
7. tokio::sync \- Rust, Zugriff am Mai 14, 2025, [https://docs.rs/tokio/latest/tokio/sync/index.html](https://docs.rs/tokio/latest/tokio/sync/index.html)  
8. Rust Error Handling: thiserror, anyhow, and When to Use Each | Momori Nakano, Zugriff am Mai 14, 2025, [https://momori.dev/posts/rust-error-handling-thiserror-anyhow/](https://momori.dev/posts/rust-error-handling-thiserror-anyhow/)  
9. Error Handling for Large Rust Projects \- Best Practice in GreptimeDB, Zugriff am Mai 13, 2025, [https://greptime.com/blogs/2024-05-07-error-rust](https://greptime.com/blogs/2024-05-07-error-rust)  
10. Simplify Error Handling in Rust with thiserror Crate \- w3resource, Zugriff am Mai 14, 2025, [https://www.w3resource.com/rust-tutorial/simplify-error-handling-rust-thiserror-crate.php](https://www.w3resource.com/rust-tutorial/simplify-error-handling-rust-thiserror-crate.php)  
11. as\_dyn\_trait \- Rust \- Docs.rs, Zugriff am Mai 14, 2025, [https://docs.rs/as-dyn-trait](https://docs.rs/as-dyn-trait)  
12. Graceful Shutdown | Will Cygan, Zugriff am Mai 14, 2025, [https://www.wcygan.io/post/tokio-graceful-shutdown/](https://www.wcygan.io/post/tokio-graceful-shutdown/)  
13. tokio::sync \- Rust \- People @EECS, Zugriff am Mai 14, 2025, [https://people.eecs.berkeley.edu/\~pschafhalter/pub/erdos/doc/tokio/sync/](https://people.eecs.berkeley.edu/~pschafhalter/pub/erdos/doc/tokio/sync/)