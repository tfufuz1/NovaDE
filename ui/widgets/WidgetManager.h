#pragma once

#include <QObject>
#include <QString>
#include <QVariant>
#include <QMap>
#include <QVector>
#include <functional>
#include "WidgetInterface.h"

namespace VivoX {
namespace Widget {

/**
 * @brief Die WidgetManager-Klasse verwaltet alle Widgets im VivoX Desktop Environment.
 * 
 * Diese Klasse ist verantwortlich für die Registrierung, Erstellung und Verwaltung
 * von Widgets. Sie folgt dem Singleton-Muster.
 */
class WidgetManager : public QObject
{
    Q_OBJECT

public:
    /**
     * @brief Gibt die Singleton-Instanz des WidgetManagers zurück.
     * @return Referenz auf die WidgetManager-Instanz
     */
    static WidgetManager& instance();

    /**
     * @brief Initialisiert den WidgetManager.
     */
    void initialize();

    /**
     * @brief Registriert einen Widget-Typ.
     * @param widgetType Typ des zu registrierenden Widgets
     * @param factory Factory-Funktion zur Erstellung von Widgets dieses Typs
     * @param metadata Metadaten des Widget-Typs
     * @return true wenn erfolgreich, false bei Fehler
     */
    bool registerWidgetType(const QString& widgetType, 
                           std::function<WidgetInterface*()> factory,
                           const QVariantMap& metadata = QVariantMap());

    /**
     * @brief Gibt die verfügbaren Widget-Typen zurück.
     * @return Liste der verfügbaren Widget-Typen
     */
    QStringList getAvailableWidgetTypes() const;

    /**
     * @brief Gibt die Metadaten eines Widget-Typs zurück.
     * @param widgetType Typ des abzufragenden Widgets
     * @return Metadaten des Widget-Typs
     */
    QVariantMap getWidgetTypeMetadata(const QString& widgetType) const;

    /**
     * @brief Erstellt ein neues Widget.
     * @param widgetType Typ des zu erstellenden Widgets
     * @param widgetId ID des zu erstellenden Widgets (optional, wird automatisch generiert wenn leer)
     * @return Zeiger auf das erstellte Widget oder nullptr bei Fehler
     */
    WidgetInterface* createWidget(const QString& widgetType, const QString& widgetId = QString());

    /**
     * @brief Entfernt ein Widget.
     * @param widgetId ID des zu entfernenden Widgets
     * @return true wenn erfolgreich, false bei Fehler
     */
    bool removeWidget(const QString& widgetId);

    /**
     * @brief Gibt ein Widget anhand seiner ID zurück.
     * @param widgetId ID des abzufragenden Widgets
     * @return Zeiger auf das Widget oder nullptr, wenn nicht gefunden
     */
    WidgetInterface* getWidget(const QString& widgetId) const;

    /**
     * @brief Gibt alle Widgets zurück.
     * @return Map mit Widget-IDs und Zeigern auf die Widgets
     */
    QMap<QString, WidgetInterface*> getWidgets() const;

    /**
     * @brief Gibt die IDs aller Widgets zurück.
     * @return Liste der Widget-IDs
     */
    QStringList getWidgetIds() const;

    /**
     * @brief Gibt die Widgets eines bestimmten Typs zurück.
     * @param widgetType Typ der abzufragenden Widgets
     * @return Liste der Widget-IDs des angegebenen Typs
     */
    QStringList getWidgetsByType(const QString& widgetType) const;

    /**
     * @brief Speichert die Widget-Konfiguration.
     * @return true wenn erfolgreich, false bei Fehler
     */
    bool saveWidgetConfiguration();

    /**
     * @brief Lädt die Widget-Konfiguration.
     * @return true wenn erfolgreich, false bei Fehler
     */
    bool loadWidgetConfiguration();

    /**
     * @brief Setzt die Widget-Konfiguration zurück.
     * @return true wenn erfolgreich, false bei Fehler
     */
    bool resetWidgetConfiguration();

signals:
    /**
     * @brief Signal, das ausgelöst wird, wenn ein Widget-Typ registriert wurde.
     * @param widgetType Typ des registrierten Widgets
     */
    void widgetTypeRegistered(const QString& widgetType);

    /**
     * @brief Signal, das ausgelöst wird, wenn ein Widget erstellt wurde.
     * @param widgetId ID des erstellten Widgets
     */
    void widgetCreated(const QString& widgetId);

    /**
     * @brief Signal, das ausgelöst wird, wenn ein Widget entfernt wurde.
     * @param widgetId ID des entfernten Widgets
     */
    void widgetRemoved(const QString& widgetId);

    /**
     * @brief Signal, das ausgelöst wird, wenn ein Widget ein Ereignis auslöst.
     * @param widgetId ID des Widgets
     * @param eventType Typ des Ereignisses
     * @param data Ereignisdaten
     */
    void widgetEvent(const QString& widgetId, const QString& eventType, const QVariant& data);

private:
    // Private Konstruktoren für Singleton-Muster
    WidgetManager();
    ~WidgetManager();
    WidgetManager(const WidgetManager&) = delete;
    WidgetManager& operator=(const WidgetManager&) = delete;

    // Interne Methoden
    QString generateWidgetId() const;
    void setupWidgetConnections(WidgetInterface* widget);

    // Interne Variablen
    QMap<QString, WidgetInterface*> m_widgets;
    QMap<QString, std::function<WidgetInterface*()>> m_widgetFactories;
    QMap<QString, QVariantMap> m_widgetTypeMetadata;
};

} // namespace Widget
} // namespace VivoX
