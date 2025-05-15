#pragma once

#include <QObject>
#include <QString>
#include <QVariantMap>
#include <QDateTime>
#include <QIcon>

namespace VivoX::System {

/**
 * @brief The NotificationInfo class contains information about a notification.
 */
class NotificationInfo {
public:
    QString id;           ///< Unique identifier for the notification
    QString appName;      ///< Name of the application sending the notification
    QString appIcon;      ///< Icon of the application
    QString summary;      ///< Summary/title of the notification
    QString body;         ///< Body text of the notification
    QStringList actions;  ///< List of action IDs
    QStringList actionLabels; ///< List of action labels
    QVariantMap hints;    ///< Additional hints
    int timeout;          ///< Timeout in milliseconds
    QDateTime timestamp;  ///< When the notification was created
    bool resident;        ///< Whether the notification should remain after being displayed
    int urgency;          ///< Urgency level (0=low, 1=normal, 2=critical)
    QString category;     ///< Category of the notification
};

/**
 * @brief The NotificationManager class manages system notifications.
 * 
 * It implements the Freedesktop.org Notifications specification and provides
 * an interface for applications to send notifications and for the UI to display them.
 */
class NotificationManager : public QObject {
    Q_OBJECT

public:
    explicit NotificationManager(QObject *parent = nullptr);
    ~NotificationManager();

    /**
     * @brief Initialize the notification manager
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Create a new notification
     * @param appName Name of the application sending the notification
     * @param replacesId ID of the notification this replaces (0 for new)
     * @param appIcon Icon of the application
     * @param summary Summary/title of the notification
     * @param body Body text of the notification
     * @param actions List of action IDs and labels (alternating)
     * @param hints Additional hints
     * @param timeout Timeout in milliseconds
     * @return The notification ID
     */
    uint createNotification(const QString &appName, uint replacesId, const QString &appIcon,
                           const QString &summary, const QString &body,
                           const QStringList &actions, const QVariantMap &hints,
                           int timeout);

    /**
     * @brief Close a notification
     * @param id The notification ID
     * @param reason The reason for closing (1=expired, 2=dismissed, 3=closed by app)
     * @return True if successful
     */
    bool closeNotification(uint id, uint reason = 3);

    /**
     * @brief Get a list of all active notifications
     * @return List of notification info objects
     */
    QList<NotificationInfo> getActiveNotifications() const;

    /**
     * @brief Get information about a specific notification
     * @param id The notification ID
     * @return The notification info, or an empty object if not found
     */
    NotificationInfo getNotificationInfo(uint id) const;

    /**
     * @brief Get the capabilities of the notification manager
     * @return List of capability strings
     */
    QStringList getCapabilities() const;

signals:
    /**
     * @brief Signal emitted when a new notification is created
     * @param info The notification info
     */
    void notificationCreated(const NotificationInfo &info);

    /**
     * @brief Signal emitted when a notification is closed
     * @param id The notification ID
     * @param reason The reason for closing (1=expired, 2=dismissed, 3=closed by app)
     */
    void notificationClosed(uint id, uint reason);

    /**
     * @brief Signal emitted when a notification action is invoked
     * @param id The notification ID
     * @param actionId The action ID
     */
    void actionInvoked(uint id, const QString &actionId);

public slots:
    /**
     * @brief Invoke an action on a notification
     * @param id The notification ID
     * @param actionId The action ID
     * @return True if successful
     */
    bool invokeAction(uint id, const QString &actionId);

private:
    // Map of notification ID to notification info
    QHash<uint, NotificationInfo> m_notifications;
    
    // Next notification ID
    uint m_nextId;
    
    // Generate a unique notification ID
    uint generateId();
    
    // Check if a notification has expired
    void checkExpiry(uint id);
};

} // namespace VivoX::System
