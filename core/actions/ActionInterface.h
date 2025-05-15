#pragma once

#include <QObject>
#include <QString>
#include <QVariant>
#include <QMap>
#include <functional>

namespace VivoX {
namespace Action {

/**
 * @brief Die ActionInterface-Klasse definiert die Schnittstelle für Aktionen im VivoX-System.
 * 
 * Diese Klasse muss von allen Aktions-Implementierungen implementiert werden, um mit dem
 * VivoX-Action-Framework zu interagieren. Sie definiert die Eigenschaften und Methoden einer Aktion.
 */
class ActionInterface : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString actionId READ actionId CONSTANT)
    Q_PROPERTY(QString actionName READ actionName WRITE setActionName NOTIFY actionNameChanged)
    Q_PROPERTY(QString actionDescription READ actionDescription WRITE setActionDescription NOTIFY actionDescriptionChanged)
    Q_PROPERTY(bool enabled READ isEnabled WRITE setEnabled NOTIFY enabledChanged)
    Q_PROPERTY(QVariantMap actionData READ actionData WRITE setActionData NOTIFY actionDataChanged)

public:
    /**
     * @brief Konstruktor für das Action-Interface.
     * @param parent Elternobjekt
     */
    explicit ActionInterface(QObject* parent = nullptr) : QObject(parent) {}
    
    /**
     * @brief Virtueller Destruktor.
     */
    virtual ~ActionInterface() {}

    /**
     * @brief Gibt die ID der Aktion zurück.
     * @return Aktions-ID
     */
    virtual QString actionId() const = 0;

    /**
     * @brief Gibt den Namen der Aktion zurück.
     * @return Aktions-Name
     */
    virtual QString actionName() const = 0;

    /**
     * @brief Setzt den Namen der Aktion.
     * @param name Neuer Name
     */
    virtual void setActionName(const QString& name) = 0;

    /**
     * @brief Gibt die Beschreibung der Aktion zurück.
     * @return Aktions-Beschreibung
     */
    virtual QString actionDescription() const = 0;

    /**
     * @brief Setzt die Beschreibung der Aktion.
     * @param description Neue Beschreibung
     */
    virtual void setActionDescription(const QString& description) = 0;

    /**
     * @brief Gibt an, ob die Aktion aktiviert ist.
     * @return true wenn aktiviert, false wenn deaktiviert
     */
    virtual bool isEnabled() const = 0;

    /**
     * @brief Aktiviert oder deaktiviert die Aktion.
     * @param enabled Neuer Aktivierungsstatus
     */
    virtual void setEnabled(bool enabled) = 0;

    /**
     * @brief Gibt die Daten der Aktion zurück.
     * @return Aktions-Daten
     */
    virtual QVariantMap actionData() const = 0;

    /**
     * @brief Setzt die Daten der Aktion.
     * @param data Neue Daten
     */
    virtual void setActionData(const QVariantMap& data) = 0;

    /**
     * @brief Führt die Aktion aus.
     * @param parameters Parameter für die Ausführung
     * @return Ergebnis der Ausführung
     */
    virtual QVariant execute(const QVariantMap& parameters = QVariantMap()) = 0;

    /**
     * @brief Prüft, ob die Aktion mit den angegebenen Parametern ausgeführt werden kann.
     * @param parameters Parameter für die Ausführung
     * @return true wenn die Aktion ausgeführt werden kann, false sonst
     */
    virtual bool canExecute(const QVariantMap& parameters = QVariantMap()) const = 0;

    /**
     * @brief Gibt die Metadaten der Aktion zurück.
     * @return Aktions-Metadaten
     */
    virtual QVariantMap metadata() const = 0;

signals:
    /**
     * @brief Signal, das ausgelöst wird, wenn sich der Name der Aktion ändert.
     * @param name Neuer Name
     */
    void actionNameChanged(const QString& name);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich die Beschreibung der Aktion ändert.
     * @param description Neue Beschreibung
     */
    void actionDescriptionChanged(const QString& description);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich der Aktivierungsstatus der Aktion ändert.
     * @param enabled Neuer Aktivierungsstatus
     */
    void enabledChanged(bool enabled);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich die Daten der Aktion ändern.
     * @param data Neue Daten
     */
    void actionDataChanged(const QVariantMap& data);

    /**
     * @brief Signal, das ausgelöst wird, wenn die Aktion ausgeführt wurde.
     * @param result Ergebnis der Ausführung
     */
    void executed(const QVariant& result);
};

} // namespace Action
} // namespace VivoX
