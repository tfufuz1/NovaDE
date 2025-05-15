#pragma once

#include <QObject>
#include <QString>
#include <QVariant>
#include <QQmlEngine>
#include <QJSEngine>
#include "WidgetInterface.h"

namespace VivoX {
namespace Widget {

/**
 * @brief Die WidgetRegistry-Klasse registriert Widgets im QML-System.
 * 
 * Diese Klasse ist verantwortlich für die Registrierung von Widget-Typen im QML-System
 * und stellt eine Brücke zwischen dem C++-Widget-System und dem QML-Widget-System dar.
 * Sie folgt dem Singleton-Muster.
 */
class WidgetRegistry : public QObject
{
    Q_OBJECT

public:
    /**
     * @brief Gibt die Singleton-Instanz der WidgetRegistry zurück.
     * @return Referenz auf die WidgetRegistry-Instanz
     */
    static WidgetRegistry& instance();

    /**
     * @brief Initialisiert die WidgetRegistry.
     */
    void initialize();

    /**
     * @brief Registriert einen Widget-Typ im QML-System.
     * @param widgetType Typ des zu registrierenden Widgets
     * @param qmlFile Pfad zur QML-Datei des Widgets
     * @param metadata Metadaten des Widget-Typs
     * @return true wenn erfolgreich, false bei Fehler
     */
    Q_INVOKABLE bool registerWidgetType(const QString& widgetType, 
                                       const QString& qmlFile,
                                       const QVariantMap& metadata = QVariantMap());

    /**
     * @brief Gibt die verfügbaren Widget-Typen zurück.
     * @return Liste der verfügbaren Widget-Typen
     */
    Q_INVOKABLE QStringList getAvailableWidgetTypes() const;

    /**
     * @brief Gibt die Metadaten eines Widget-Typs zurück.
     * @param widgetType Typ des abzufragenden Widgets
     * @return Metadaten des Widget-Typs
     */
    Q_INVOKABLE QVariantMap getWidgetTypeMetadata(const QString& widgetType) const;

    /**
     * @brief Gibt den QML-Pfad eines Widget-Typs zurück.
     * @param widgetType Typ des abzufragenden Widgets
     * @return QML-Pfad des Widget-Typs
     */
    Q_INVOKABLE QString getWidgetQmlPath(const QString& widgetType) const;

    /**
     * @brief Erstellt ein QML-Widget-Objekt.
     * @param widgetType Typ des zu erstellenden Widgets
     * @param parent Elternobjekt
     * @return Erstelltes QML-Objekt oder nullptr bei Fehler
     */
    Q_INVOKABLE QObject* createWidgetObject(const QString& widgetType, QObject* parent = nullptr);

    /**
     * @brief Erstellt eine QML-Widget-Komponente.
     * @param widgetType Typ des zu erstellenden Widgets
     * @return Erstellte QML-Komponente oder nullptr bei Fehler
     */
    Q_INVOKABLE QObject* createWidgetComponent(const QString& widgetType);

    /**
     * @brief Registriert die WidgetRegistry als QML-Singleton.
     * @param engine QML-Engine
     */
    static QObject* qmlInstance(QQmlEngine* engine, QJSEngine* scriptEngine);

signals:
    /**
     * @brief Signal, das ausgelöst wird, wenn ein Widget-Typ registriert wurde.
     * @param widgetType Typ des registrierten Widgets
     */
    void widgetTypeRegistered(const QString& widgetType);

private:
    // Private Konstruktoren für Singleton-Muster
    WidgetRegistry();
    ~WidgetRegistry();
    WidgetRegistry(const WidgetRegistry&) = delete;
    WidgetRegistry& operator=(const WidgetRegistry&) = delete;

    // Interne Variablen
    QMap<QString, QString> m_widgetQmlPaths;
    QMap<QString, QVariantMap> m_widgetTypeMetadata;
};

} // namespace Widget
} // namespace VivoX
