#pragma once

#include <QObject>
#include <QString>
#include <QVariant>
#include <QMap>
#include <QVector>
#include <functional>

namespace VivoX {
namespace Widget {

/**
 * @brief Die WidgetInterface-Klasse definiert die Schnittstelle für Widgets im VivoX-System.
 * 
 * Diese Klasse muss von allen Widget-Implementierungen implementiert werden, um mit dem
 * VivoX-Widget-System zu interagieren. Sie definiert die Eigenschaften und Methoden eines Widgets.
 */
class WidgetInterface : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString widgetId READ widgetId CONSTANT)
    Q_PROPERTY(QString widgetType READ widgetType CONSTANT)
    Q_PROPERTY(QString widgetName READ widgetName WRITE setWidgetName NOTIFY widgetNameChanged)
    Q_PROPERTY(QString widgetDescription READ widgetDescription WRITE setWidgetDescription NOTIFY widgetDescriptionChanged)
    Q_PROPERTY(QVariantMap widgetSettings READ widgetSettings WRITE setWidgetSettings NOTIFY widgetSettingsChanged)
    Q_PROPERTY(bool enabled READ isEnabled WRITE setEnabled NOTIFY enabledChanged)

public:
    /**
     * @brief Konstruktor für das Widget-Interface.
     * @param parent Elternobjekt
     */
    explicit WidgetInterface(QObject* parent = nullptr) : QObject(parent) {}
    
    /**
     * @brief Virtueller Destruktor.
     */
    virtual ~WidgetInterface() {}

    /**
     * @brief Gibt die ID des Widgets zurück.
     * @return Widget-ID
     */
    virtual QString widgetId() const = 0;

    /**
     * @brief Gibt den Typ des Widgets zurück.
     * @return Widget-Typ
     */
    virtual QString widgetType() const = 0;

    /**
     * @brief Gibt den Namen des Widgets zurück.
     * @return Widget-Name
     */
    virtual QString widgetName() const = 0;

    /**
     * @brief Setzt den Namen des Widgets.
     * @param name Neuer Name
     */
    virtual void setWidgetName(const QString& name) = 0;

    /**
     * @brief Gibt die Beschreibung des Widgets zurück.
     * @return Widget-Beschreibung
     */
    virtual QString widgetDescription() const = 0;

    /**
     * @brief Setzt die Beschreibung des Widgets.
     * @param description Neue Beschreibung
     */
    virtual void setWidgetDescription(const QString& description) = 0;

    /**
     * @brief Gibt die Einstellungen des Widgets zurück.
     * @return Widget-Einstellungen
     */
    virtual QVariantMap widgetSettings() const = 0;

    /**
     * @brief Setzt die Einstellungen des Widgets.
     * @param settings Neue Einstellungen
     */
    virtual void setWidgetSettings(const QVariantMap& settings) = 0;

    /**
     * @brief Gibt an, ob das Widget aktiviert ist.
     * @return true wenn aktiviert, false wenn deaktiviert
     */
    virtual bool isEnabled() const = 0;

    /**
     * @brief Aktiviert oder deaktiviert das Widget.
     * @param enabled Neuer Aktivierungsstatus
     */
    virtual void setEnabled(bool enabled) = 0;

    /**
     * @brief Initialisiert das Widget.
     * @return true wenn erfolgreich, false bei Fehler
     */
    virtual bool initialize() = 0;

    /**
     * @brief Deinitialisiert das Widget.
     * @return true wenn erfolgreich, false bei Fehler
     */
    virtual bool deinitialize() = 0;

    /**
     * @brief Aktualisiert das Widget.
     */
    virtual void update() = 0;

    /**
     * @brief Öffnet den Einstellungsdialog des Widgets.
     */
    virtual void openSettings() = 0;

    /**
     * @brief Gibt die Metadaten des Widgets zurück.
     * @return Widget-Metadaten
     */
    virtual QVariantMap metadata() const = 0;

signals:
    /**
     * @brief Signal, das ausgelöst wird, wenn sich der Name des Widgets ändert.
     * @param name Neuer Name
     */
    void widgetNameChanged(const QString& name);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich die Beschreibung des Widgets ändert.
     * @param description Neue Beschreibung
     */
    void widgetDescriptionChanged(const QString& description);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich die Einstellungen des Widgets ändern.
     * @param settings Neue Einstellungen
     */
    void widgetSettingsChanged(const QVariantMap& settings);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich der Aktivierungsstatus des Widgets ändert.
     * @param enabled Neuer Aktivierungsstatus
     */
    void enabledChanged(bool enabled);

    /**
     * @brief Signal, das ausgelöst wird, wenn das Widget ein Ereignis auslöst.
     * @param eventType Typ des Ereignisses
     * @param data Ereignisdaten
     */
    void widgetEvent(const QString& eventType, const QVariant& data);
};

} // namespace Widget
} // namespace VivoX
