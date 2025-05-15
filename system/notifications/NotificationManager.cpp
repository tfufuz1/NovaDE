#include "NotificationManager.h"

#include <QDebug>
#include <QTimer>
#include <QUuid>

namespace VivoX::System {

NotificationManager::NotificationManager(QObject *parent)
    : QObject(parent)
    , m_nextId(1)
{
    qDebug() << "NotificationManager created";
}

NotificationManager::~NotificationManager()
{
    qDebug() << "NotificationManager destroyed";
}

bool NotificationManager::initialize()
{
    qDebug() << "NotificationManager initialized";
    return true;
}

uint NotificationManager::createNotification(const QString &appName, uint replacesId, const QString &appIcon,
                                           const QString &summary, const QString &body,
                                           const QStringList &actions, const QVariantMap &hints,
                                           int timeout)
{
    uint id;
    
    // Check if this is replacing an existing notification
    if (replacesId > 0 && m_notifications.contains(replacesId)) {
        id = replacesId;
    } else {
        id = generateId();
    }
    
    // Create notification info
    NotificationInfo info;
    info.id = QString::number(id);
    info.appName = appName;
    info.appIcon = appIcon;
    info.summary = summary;
    info.body = body;
    
    // Parse actions (alternating ID and label)
    for (int i = 0; i < actions.size(); i += 2) {
        if (i + 1 < actions.size()) {
            info.actions.append(actions[i]);
            info.actionLabels.append(actions[i + 1]);
        }
    }
    
    info.hints = hints;
    info.timeout = timeout;
    info.timestamp = QDateTime::currentDateTime();
    
    // Parse hints
    info.resident = hints.value("resident", false).toBool();
    info.urgency = hints.value("urgency", 1).toInt();
    info.category = hints.value("category", "").toString();
    
    // Add to map
    m_notifications[id] = info;
    
    // Emit signal
    emit notificationCreated(info);
    
    qDebug() << "Created notification:" << id << summary;
    
    // Set up expiry timer if timeout is not 0
    if (timeout > 0) {
        QTimer::singleShot(timeout, this, [this, id]() {
            checkExpiry(id);
        });
    }
    
    return id;
}

bool NotificationManager::closeNotification(uint id, uint reason)
{
    if (!m_notifications.contains(id)) {
        qWarning() << "Notification not found:" << id;
        return false;
    }
    
    // Remove from map
    m_notifications.remove(id);
    
    // Emit signal
    emit notificationClosed(id, reason);
    
    qDebug() << "Closed notification:" << id << "with reason:" << reason;
    
    return true;
}

QList<NotificationInfo> NotificationManager::getActiveNotifications() const
{
    return m_notifications.values();
}

NotificationInfo NotificationManager::getNotificationInfo(uint id) const
{
    return m_notifications.value(id, NotificationInfo());
}

QStringList NotificationManager::getCapabilities() const
{
    // Return the capabilities of this notification manager
    return QStringList() 
        << "actions"
        << "body"
        << "body-hyperlinks"
        << "body-markup"
        << "icon-static"
        << "persistence"
        << "sound";
}

bool NotificationManager::invokeAction(uint id, const QString &actionId)
{
    if (!m_notifications.contains(id)) {
        qWarning() << "Notification not found:" << id;
        return false;
    }
    
    const NotificationInfo &info = m_notifications[id];
    
    // Check if the action exists
    if (!info.actions.contains(actionId)) {
        qWarning() << "Action not found:" << actionId << "for notification:" << id;
        return false;
    }
    
    // Emit signal
    emit actionInvoked(id, actionId);
    
    qDebug() << "Invoked action:" << actionId << "for notification:" << id;
    
    // Close the notification if it's not resident
    if (!info.resident) {
        closeNotification(id, 2); // Dismissed by user
    }
    
    return true;
}

uint NotificationManager::generateId()
{
    return m_nextId++;
}

void NotificationManager::checkExpiry(uint id)
{
    if (m_notifications.contains(id)) {
        closeNotification(id, 1); // Expired
    }
}

} // namespace VivoX::System
