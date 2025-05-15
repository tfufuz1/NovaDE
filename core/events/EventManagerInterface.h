#pragma once

#include <QObject>
#include <QString>
#include <QList>
#include <QVariantMap>

namespace VivoX {
namespace Core {

/**
 * @brief Interface for the event manager
 * 
 * This interface defines the functionality that the event manager must implement.
 * It is responsible for dispatching events between components in the system.
 */
class EventManagerInterface {
public:
    virtual ~EventManagerInterface() = default;

    /**
     * @brief Initialize the event manager
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Register an event handler
     * 
     * @param eventType The type of event to handle
     * @param handler The object that will handle the event
     * @param method The method to call when the event occurs
     * @return bool True if registration was successful, false otherwise
     */
    virtual bool registerEventHandler(const QString& eventType, QObject* handler, const char* method) = 0;

    /**
     * @brief Unregister an event handler
     * 
     * @param eventType The type of event
     * @param handler The handler object
     * @param method The handler method
     * @return bool True if unregistration was successful, false otherwise
     */
    virtual bool unregisterEventHandler(const QString& eventType, QObject* handler, const char* method = nullptr) = 0;

    /**
     * @brief Emit an event
     * 
     * @param eventType The type of event to emit
     * @param data The event data
     * @return bool True if the event was successfully emitted, false otherwise
     */
    virtual bool emitEvent(const QString& eventType, const QVariantMap& data = QVariantMap()) = 0;

    /**
     * @brief Get all registered event types
     * 
     * @return QStringList The list of registered event types
     */
    virtual QStringList registeredEventTypes() const = 0;

    /**
     * @brief Get all handlers for a specific event type
     * 
     * @param eventType The event type
     * @return QList<QObject*> The list of handler objects
     */
    virtual QList<QObject*> handlersForEventType(const QString& eventType) const = 0;
};

} // namespace Core
} // namespace VivoX
