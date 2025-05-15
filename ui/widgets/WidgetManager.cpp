#include "WidgetManager.h"
#include <QDebug>
#include <QUuid>
#include "../core/ConfigManager.h"
#include "../core/Logger.h"

namespace VivoX {
namespace Widget {

WidgetManager::WidgetManager() : QObject(nullptr)
{
    qDebug() << "WidgetManager initialized";
}

WidgetManager::~WidgetManager()
{
    // Entferne alle Widgets
    QStringList widgetIds = m_widgets.keys();
    for (const QString& widgetId : widgetIds) {
        removeWidget(widgetId);
    }
    
    qDebug() << "WidgetManager destroyed";
}

WidgetManager& WidgetManager::instance()
{
    static WidgetManager instance;
    return instance;
}

void WidgetManager::initialize()
{
    // Lade die Widget-Konfiguration
    loadWidgetConfiguration();
    
    qDebug() << "WidgetManager initialized";
}

bool WidgetManager::registerWidgetType(const QString& widgetType, 
                                      std::function<WidgetInterface*()> factory,
                                      const QVariantMap& metadata)
{
    if (m_widgetFactories.contains(widgetType)) {
        Core::Logger::instance().warning("Widget type already registered: " + widgetType, "WidgetManager");
        return false;
    }
    
    m_widgetFactories[widgetType] = factory;
    m_widgetTypeMetadata[widgetType] = metadata;
    
    emit widgetTypeRegistered(widgetType);
    Core::Logger::instance().info("Widget type registered: " + widgetType, "WidgetManager");
    
    return true;
}

QStringList WidgetManager::getAvailableWidgetTypes() const
{
    return m_widgetFactories.keys();
}

QVariantMap WidgetManager::getWidgetTypeMetadata(const QString& widgetType) const
{
    return m_widgetTypeMetadata.value(widgetType, QVariantMap());
}

WidgetInterface* WidgetManager::createWidget(const QString& widgetType, const QString& widgetId)
{
    if (!m_widgetFactories.contains(widgetType)) {
        Core::Logger::instance().error("Widget type not registered: " + widgetType, "WidgetManager");
        return nullptr;
    }
    
    // Erstelle das Widget mit der Factory-Funktion
    WidgetInterface* widget = m_widgetFactories[widgetType]();
    if (!widget) {
        Core::Logger::instance().error("Failed to create widget of type: " + widgetType, "WidgetManager");
        return nullptr;
    }
    
    // Generiere eine eindeutige ID, wenn keine angegeben wurde
    QString actualWidgetId = widgetId.isEmpty() ? generateWidgetId() : widgetId;
    
    // Verbinde Signale
    setupWidgetConnections(widget);
    
    // Initialisiere das Widget
    if (!widget->initialize()) {
        Core::Logger::instance().error("Failed to initialize widget of type: " + widgetType, "WidgetManager");
        delete widget;
        return nullptr;
    }
    
    // Speichere das Widget
    m_widgets[actualWidgetId] = widget;
    
    emit widgetCreated(actualWidgetId);
    Core::Logger::instance().info("Widget created: " + actualWidgetId + " of type: " + widgetType, "WidgetManager");
    
    return widget;
}

bool WidgetManager::removeWidget(const QString& widgetId)
{
    if (!m_widgets.contains(widgetId)) {
        Core::Logger::instance().warning("Widget not found: " + widgetId, "WidgetManager");
        return false;
    }
    
    WidgetInterface* widget = m_widgets[widgetId];
    
    // Deinitialisiere das Widget
    if (!widget->deinitialize()) {
        Core::Logger::instance().warning("Failed to deinitialize widget: " + widgetId, "WidgetManager");
        // Wir entfernen das Widget trotzdem
    }
    
    // Trenne Signale
    disconnect(widget, nullptr, this, nullptr);
    
    // Entferne das Widget
    m_widgets.remove(widgetId);
    
    // Lösche das Widget-Objekt
    delete widget;
    
    emit widgetRemoved(widgetId);
    Core::Logger::instance().info("Widget removed: " + widgetId, "WidgetManager");
    
    return true;
}

WidgetInterface* WidgetManager::getWidget(const QString& widgetId) const
{
    return m_widgets.value(widgetId, nullptr);
}

QMap<QString, WidgetInterface*> WidgetManager::getWidgets() const
{
    return m_widgets;
}

QStringList WidgetManager::getWidgetIds() const
{
    return m_widgets.keys();
}

QStringList WidgetManager::getWidgetsByType(const QString& widgetType) const
{
    QStringList result;
    
    for (auto it = m_widgets.begin(); it != m_widgets.end(); ++it) {
        if (it.value()->widgetType() == widgetType) {
            result << it.key();
        }
    }
    
    return result;
}

bool WidgetManager::saveWidgetConfiguration()
{
    Core::ConfigManager& configManager = Core::ConfigManager::instance();
    
    // Speichere die Anzahl der Widgets
    configManager.setValue("widgets/count", m_widgets.size());
    
    // Speichere die Widget-Typen
    int index = 0;
    for (auto it = m_widgetTypeMetadata.begin(); it != m_widgetTypeMetadata.end(); ++it) {
        configManager.setValue("widgets/types/" + QString::number(index) + "/name", it.key());
        configManager.setValue("widgets/types/" + QString::number(index) + "/metadata", it.value());
        index++;
    }
    configManager.setValue("widgets/types/count", index);
    
    // Speichere die Widget-Konfigurationen
    index = 0;
    for (auto it = m_widgets.begin(); it != m_widgets.end(); ++it) {
        QString widgetId = it.key();
        WidgetInterface* widget = it.value();
        
        configManager.setValue("widgets/widget" + QString::number(index) + "/id", widgetId);
        configManager.setValue("widgets/widget" + QString::number(index) + "/type", widget->widgetType());
        configManager.setValue("widgets/widget" + QString::number(index) + "/name", widget->widgetName());
        configManager.setValue("widgets/widget" + QString::number(index) + "/description", widget->widgetDescription());
        configManager.setValue("widgets/widget" + QString::number(index) + "/settings", widget->widgetSettings());
        configManager.setValue("widgets/widget" + QString::number(index) + "/enabled", widget->isEnabled());
        
        index++;
    }
    
    Core::Logger::instance().info("Widget configuration saved", "WidgetManager");
    return true;
}

bool WidgetManager::loadWidgetConfiguration()
{
    Core::ConfigManager& configManager = Core::ConfigManager::instance();
    
    // Entferne alle bestehenden Widgets
    QStringList widgetIds = m_widgets.keys();
    for (const QString& widgetId : widgetIds) {
        removeWidget(widgetId);
    }
    
    // Lade die Widget-Konfigurationen
    int widgetCount = configManager.getValue("widgets/count", 0).toInt();
    for (int i = 0; i < widgetCount; ++i) {
        QString widgetId = configManager.getValue("widgets/widget" + QString::number(i) + "/id").toString();
        QString widgetType = configManager.getValue("widgets/widget" + QString::number(i) + "/type").toString();
        QString widgetName = configManager.getValue("widgets/widget" + QString::number(i) + "/name").toString();
        QString widgetDescription = configManager.getValue("widgets/widget" + QString::number(i) + "/description").toString();
        QVariantMap widgetSettings = configManager.getValue("widgets/widget" + QString::number(i) + "/settings").toMap();
        bool enabled = configManager.getValue("widgets/widget" + QString::number(i) + "/enabled", true).toBool();
        
        // Erstelle das Widget
        WidgetInterface* widget = createWidget(widgetType, widgetId);
        if (widget) {
            widget->setWidgetName(widgetName);
            widget->setWidgetDescription(widgetDescription);
            widget->setWidgetSettings(widgetSettings);
            widget->setEnabled(enabled);
        }
    }
    
    Core::Logger::instance().info("Widget configuration loaded", "WidgetManager");
    return true;
}

bool WidgetManager::resetWidgetConfiguration()
{
    // Entferne alle bestehenden Widgets
    QStringList widgetIds = m_widgets.keys();
    for (const QString& widgetId : widgetIds) {
        removeWidget(widgetId);
    }
    
    // Erstelle Standard-Widgets
    // In einer vollständigen Implementierung würden wir hier Standard-Widgets erstellen
    
    Core::Logger::instance().info("Widget configuration reset to defaults", "WidgetManager");
    return true;
}

QString WidgetManager::generateWidgetId() const
{
    return "widget_" + QUuid::createUuid().toString(QUuid::WithoutBraces).left(8);
}

void WidgetManager::setupWidgetConnections(WidgetInterface* widget)
{
    if (!widget) {
        return;
    }
    
    connect(widget, &WidgetInterface::widgetEvent, [this, widget](const QString& eventType, const QVariant& data) {
        QString widgetId = widget->widgetId();
        emit widgetEvent(widgetId, eventType, data);
    });
}

} // namespace Widget
} // namespace VivoX
