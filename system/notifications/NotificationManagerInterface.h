#pragma once

#include <QObject>
#include <QString>
#include <QList>

namespace VivoX {
namespace System {

class Notification;

/**
 * @brief Interface for the notification manager
 * 
 * This interface defines the functionality that the notification manager must implement.
 * It is responsible for managing system notifications.
 */
class NotificationManagerInterface {
public:
    virtual ~NotificationManagerInterface() = default;

    /**
     * @brief Initialize the notification manager
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Create a new notification
     * 
     * @param appName The application name
     * @param summary The notification summary
     * @param body The notification body
     * @param icon The notification icon
     * @param timeout The notification timeout in milliseconds
     * @return Notification* The created notification
     */
    virtual Notification* createNotification(const QString& appName, const QString& summary, 
                                            const QString& body, const QString& icon, 
                                            int timeout = -1) = 0;

    /**
     * @brief Show a notification
     * 
     * @param notification The notification to show
     * @return uint32_t The notification ID
     */
    virtual uint32_t showNotification(Notification* notification) = 0;

    /**
     * @brief Close a notification
     * 
     * @param id The notification ID
     * @return bool True if the notification was closed, false otherwise
     */
    virtual bool closeNotification(uint32_t id) = 0;

    /**
     * @brief Get a notification by ID
     * 
     * @param id The notification ID
     * @return Notification* The notification, or nullptr if not found
     */
    virtual Notification* notificationById(uint32_t id) const = 0;

    /**
     * @brief Get all active notifications
     * 
     * @return QList<Notification*> The list of active notifications
     */
    virtual QList<Notification*> activeNotifications() const = 0;

    /**
     * @brief Get the notification history
     * 
     * @param limit The maximum number of notifications to return
     * @return QList<Notification*> The list of historical notifications
     */
    virtual QList<Notification*> notificationHistory(int limit = -1) const = 0;

    /**
     * @brief Clear the notification history
     */
    virtual void clearHistory() = 0;

    /**
     * @brief Get the server capabilities
     * 
     * @return QStringList The list of server capabilities
     */
    virtual QStringList serverCapabilities() const = 0;
};

} // namespace System
} // namespace VivoX
