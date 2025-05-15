#include "WidgetRegistry.h"
#include <QDebug>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQmlContext>
#include <QUrl>
#include "../core/Logger.h"

namespace VivoX {
namespace Widget {

WidgetRegistry::WidgetRegistry() : QObject(nullptr)
{
    qDebug() << "WidgetRegistry initialized";
}

WidgetRegistry::~WidgetRegistry()
{
    qDebug() << "WidgetRegistry destroyed";
}

WidgetRegistry& WidgetRegistry::instance()
{
    static WidgetRegistry instance;
    return instance;
}

void WidgetRegistry::initialize()
{
    qDebug() << "WidgetRegistry initialized";
}

bool WidgetRegistry::registerWidgetType(const QString& widgetType, 
                                       const QString& qmlFile,
                                       const QVariantMap& metadata)
{
    if (m_widgetQmlPaths.contains(widgetType)) {
        Core::Logger::instance().warning("Widget type already registered: " + widgetType, "WidgetRegistry");
        return false;
    }
    
    m_widgetQmlPaths[widgetType] = qmlFile;
    m_widgetTypeMetadata[widgetType] = metadata;
    
    emit widgetTypeRegistered(widgetType);
    Core::Logger::instance().info("Widget type registered: " + widgetType + " with QML file: " + qmlFile, "WidgetRegistry");
    
    return true;
}

QStringList WidgetRegistry::getAvailableWidgetTypes() const
{
    return m_widgetQmlPaths.keys();
}

QVariantMap WidgetRegistry::getWidgetTypeMetadata(const QString& widgetType) const
{
    return m_widgetTypeMetadata.value(widgetType, QVariantMap());
}

QString WidgetRegistry::getWidgetQmlPath(const QString& widgetType) const
{
    return m_widgetQmlPaths.value(widgetType, QString());
}

QObject* WidgetRegistry::createWidgetObject(const QString& widgetType, QObject* parent)
{
    if (!m_widgetQmlPaths.contains(widgetType)) {
        Core::Logger::instance().error("Widget type not registered: " + widgetType, "WidgetRegistry");
        return nullptr;
    }
    
    QString qmlFile = m_widgetQmlPaths[widgetType];
    
    QQmlEngine* engine = qmlEngine(this);
    if (!engine) {
        engine = new QQmlEngine(this);
    }
    
    QQmlComponent component(engine, QUrl(qmlFile));
    if (component.status() != QQmlComponent::Ready) {
        Core::Logger::instance().error("Failed to create widget component: " + component.errorString(), "WidgetRegistry");
        return nullptr;
    }
    
    QObject* object = component.create(engine->rootContext());
    if (!object) {
        Core::Logger::instance().error("Failed to create widget object", "WidgetRegistry");
        return nullptr;
    }
    
    if (parent) {
        object->setParent(parent);
    }
    
    return object;
}

QObject* WidgetRegistry::createWidgetComponent(const QString& widgetType)
{
    if (!m_widgetQmlPaths.contains(widgetType)) {
        Core::Logger::instance().error("Widget type not registered: " + widgetType, "WidgetRegistry");
        return nullptr;
    }
    
    QString qmlFile = m_widgetQmlPaths[widgetType];
    
    QQmlEngine* engine = qmlEngine(this);
    if (!engine) {
        engine = new QQmlEngine(this);
    }
    
    QQmlComponent* component = new QQmlComponent(engine, QUrl(qmlFile));
    if (component->status() != QQmlComponent::Ready) {
        Core::Logger::instance().error("Failed to create widget component: " + component->errorString(), "WidgetRegistry");
        delete component;
        return nullptr;
    }
    
    return component;
}

QObject* WidgetRegistry::qmlInstance(QQmlEngine* engine, QJSEngine* scriptEngine)
{
    Q_UNUSED(scriptEngine)
    
    WidgetRegistry* instance = &WidgetRegistry::instance();
    QQmlEngine::setObjectOwnership(instance, QQmlEngine::CppOwnership);
    return instance;
}

} // namespace Widget
} // namespace VivoX
