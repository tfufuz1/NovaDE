#include "ActionRegistry.h"
#include <QDebug>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQmlContext>
#include <QUrl>
#include "../core/Logger.h"
#include "ActionManager.h"

namespace VivoX {
namespace Action {

ActionRegistry::ActionRegistry() : QObject(nullptr)
{
    qDebug() << "ActionRegistry initialized";
}

ActionRegistry::~ActionRegistry()
{
    qDebug() << "ActionRegistry destroyed";
}

ActionRegistry& ActionRegistry::instance()
{
    static ActionRegistry instance;
    return instance;
}

void ActionRegistry::initialize()
{
    qDebug() << "ActionRegistry initialized";
}

bool ActionRegistry::registerActionType(const QString& actionType, 
                                       const QString& qmlFile,
                                       const QVariantMap& metadata)
{
    if (m_actionQmlPaths.contains(actionType)) {
        Core::Logger::instance().warning("Action type already registered: " + actionType, "ActionRegistry");
        return false;
    }
    
    m_actionQmlPaths[actionType] = qmlFile;
    m_actionTypeMetadata[actionType] = metadata;
    
    emit actionTypeRegistered(actionType);
    Core::Logger::instance().info("Action type registered: " + actionType + " with QML file: " + qmlFile, "ActionRegistry");
    
    return true;
}

QStringList ActionRegistry::getAvailableActionTypes() const
{
    return m_actionQmlPaths.keys();
}

QVariantMap ActionRegistry::getActionTypeMetadata(const QString& actionType) const
{
    return m_actionTypeMetadata.value(actionType, QVariantMap());
}

QString ActionRegistry::getActionQmlPath(const QString& actionType) const
{
    return m_actionQmlPaths.value(actionType, QString());
}

QObject* ActionRegistry::createActionObject(const QString& actionType, QObject* parent)
{
    if (!m_actionQmlPaths.contains(actionType)) {
        Core::Logger::instance().error("Action type not registered: " + actionType, "ActionRegistry");
        return nullptr;
    }
    
    QString qmlFile = m_actionQmlPaths[actionType];
    
    QQmlEngine* engine = qmlEngine(this);
    if (!engine) {
        engine = new QQmlEngine(this);
    }
    
    QQmlComponent component(engine, QUrl(qmlFile));
    if (component.status() != QQmlComponent::Ready) {
        Core::Logger::instance().error("Failed to create action component: " + component.errorString(), "ActionRegistry");
        return nullptr;
    }
    
    QObject* object = component.create(engine->rootContext());
    if (!object) {
        Core::Logger::instance().error("Failed to create action object", "ActionRegistry");
        return nullptr;
    }
    
    if (parent) {
        object->setParent(parent);
    }
    
    return object;
}

QObject* ActionRegistry::createActionComponent(const QString& actionType)
{
    if (!m_actionQmlPaths.contains(actionType)) {
        Core::Logger::instance().error("Action type not registered: " + actionType, "ActionRegistry");
        return nullptr;
    }
    
    QString qmlFile = m_actionQmlPaths[actionType];
    
    QQmlEngine* engine = qmlEngine(this);
    if (!engine) {
        engine = new QQmlEngine(this);
    }
    
    QQmlComponent* component = new QQmlComponent(engine, QUrl(qmlFile));
    if (component->status() != QQmlComponent::Ready) {
        Core::Logger::instance().error("Failed to create action component: " + component->errorString(), "ActionRegistry");
        delete component;
        return nullptr;
    }
    
    return component;
}

QVariant ActionRegistry::executeAction(const QString& actionId, const QVariantMap& parameters)
{
    // Delegiere die Ausführung an den ActionManager
    return ActionManager::instance().executeAction(actionId, parameters);
}

QObject* ActionRegistry::qmlInstance(QQmlEngine* engine, QJSEngine* scriptEngine)
{
    Q_UNUSED(scriptEngine)
    
    ActionRegistry* instance = &ActionRegistry::instance();
    QQmlEngine::setObjectOwnership(instance, QQmlEngine::CppOwnership);
    return instance;
}

} // namespace Action
} // namespace VivoX
