#include "SessionManager.h"

#include <QDebug>
#include <QProcess>
#include <QDateTime>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QSettings>
#include <QStandardPaths>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QDBusConnection>
#include <QDBusInterface>
#include <QDBusReply>
#include <QCryptographicHash>
#include <QUuid>
#include <unistd.h>
#include <pwd.h>

namespace VivoX::System {

SessionManager::SessionManager(QObject *parent)
    : QObject(parent)
    , m_sessionStartTime(QDateTime::currentSecsSinceEpoch())
{
    // Get current user name
    uid_t uid = getuid();
    struct passwd *pw = getpwuid(uid);
    if (pw) {
        m_userName = QString::fromLocal8Bit(pw->pw_name);
    } else {
        m_userName = qgetenv("USER");
        if (m_userName.isEmpty()) {
            m_userName = qgetenv("USERNAME");
        }
    }
    
    // Generate a unique session ID
    QUuid sessionUuid = QUuid::createUuid();
    m_sessionId = sessionUuid.toString(QUuid::WithoutBraces);
    
    qDebug() << "SessionManager created for user:" << m_userName << "with session ID:" << m_sessionId;
}

SessionManager::~SessionManager()
{
    qDebug() << "SessionManager destroyed";
}

bool SessionManager::initialize()
{
    // Load startup applications
    loadStartupApplications();
    
    // Connect to session DBus signals if available
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    
    if (sessionBus.isConnected()) {
        // Connect to screen lock signals
        sessionBus.connect(
            "org.freedesktop.ScreenSaver", 
            "/org/freedesktop/ScreenSaver", 
            "org.freedesktop.ScreenSaver", 
            "ActiveChanged", 
            this, 
            SLOT([this](bool active) {
                if (active) {
                    emit screenLocked();
                } else {
                    emit screenUnlocked();
                }
            })
        );
        
        // Connect to session signals
        sessionBus.connect(
            "org.gnome.SessionManager", 
            "/org/gnome/SessionManager", 
            "org.gnome.SessionManager", 
            "SessionRunning", 
            this, 
            SLOT([this]() {
                qDebug() << "Session is now running";
            })
        );
        
        sessionBus.connect(
            "org.gnome.SessionManager", 
            "/org/gnome/SessionManager", 
            "org.gnome.SessionManager", 
            "SessionOver", 
            this, 
            SLOT([this]() {
                emit sessionEnding("logout");
            })
        );
    }
    
    qDebug() << "SessionManager initialized";
    return true;
}

QString SessionManager::getUserName() const
{
    return m_userName;
}

QString SessionManager::getSessionId() const
{
    return m_sessionId;
}

qint64 SessionManager::getSessionStartTime() const
{
    return m_sessionStartTime;
}

QStringList SessionManager::getStartupApplications() const
{
    return m_startupApplications;
}

bool SessionManager::addStartupApplication(const QString &appId)
{
    if (m_startupApplications.contains(appId)) {
        qDebug() << "Application already in startup list:" << appId;
        return true;
    }
    
    // Verify that the application exists
    QString desktopFilePath = findDesktopFile(appId);
    
    if (desktopFilePath.isEmpty()) {
        qWarning() << "Could not find desktop file for application:" << appId;
        return false;
    }
    
    m_startupApplications.append(appId);
    saveStartupApplications();
    
    emit startupApplicationsChanged();
    
    qDebug() << "Added application to startup list:" << appId;
    return true;
}

bool SessionManager::removeStartupApplication(const QString &appId)
{
    if (!m_startupApplications.contains(appId)) {
        qDebug() << "Application not in startup list:" << appId;
        return true;
    }
    
    m_startupApplications.removeAll(appId);
    saveStartupApplications();
    
    emit startupApplicationsChanged();
    
    qDebug() << "Removed application from startup list:" << appId;
    return true;
}

bool SessionManager::saveSession()
{
    // Create session state
    QVariantMap sessionState;
    
    // Add basic session info
    sessionState["sessionId"] = m_sessionId;
    sessionState["userName"] = m_userName;
    sessionState["timestamp"] = QDateTime::currentDateTime().toString(Qt::ISODate);
    
    // Add running applications
    // In a real implementation, we would query the window manager for this information
    // For now, we'll just save a placeholder
    QVariantList runningApps;
    // TODO: Get actual running applications from window manager
    sessionState["runningApplications"] = runningApps;
    
    // Add window positions and states
    // In a real implementation, we would query the window manager for this information
    // For now, we'll just save a placeholder
    QVariantList windowStates;
    // TODO: Get actual window states from window manager
    sessionState["windowStates"] = windowStates;
    
    // Add workspace information
    // In a real implementation, we would query the workspace manager for this information
    // For now, we'll just save a placeholder
    QVariantList workspaces;
    // TODO: Get actual workspace information from workspace manager
    sessionState["workspaces"] = workspaces;
    
    // Save session state to file
    QString sessionDir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation) + "/sessions";
    QDir().mkpath(sessionDir);
    
    QString sessionFile = sessionDir + "/session-" + m_sessionId + ".json";
    QFile file(sessionFile);
    
    if (!file.open(QIODevice::WriteOnly)) {
        qWarning() << "Failed to open session file for writing:" << sessionFile;
        return false;
    }
    
    QJsonDocument doc = QJsonDocument::fromVariant(sessionState);
    file.write(doc.toJson());
    file.close();
    
    // Save as the latest session
    QString latestSessionFile = sessionDir + "/latest-session.json";
    QFile latestFile(latestSessionFile);
    
    if (latestFile.open(QIODevice::WriteOnly)) {
        latestFile.write(doc.toJson());
        latestFile.close();
    }
    
    m_sessionState = sessionState;
    
    qDebug() << "Session saved to:" << sessionFile;
    return true;
}

bool SessionManager::restoreSession()
{
    // Load the latest session
    QString sessionDir = QStandardPaths::writableLocation(QStandardPaths::AppDataLocation) + "/sessions";
    QString latestSessionFile = sessionDir + "/latest-session.json";
    
    QFile file(latestSessionFile);
    
    if (!file.exists()) {
        qWarning() << "No saved session found";
        return false;
    }
    
    if (!file.open(QIODevice::ReadOnly)) {
        qWarning() << "Failed to open session file for reading:" << latestSessionFile;
        return false;
    }
    
    QJsonDocument doc = QJsonDocument::fromJson(file.readAll());
    file.close();
    
    if (doc.isNull() || !doc.isObject()) {
        qWarning() << "Invalid session file format:" << latestSessionFile;
        return false;
    }
    
    QVariantMap sessionState = doc.object().toVariantMap();
    
    // Validate session state
    if (!sessionState.contains("sessionId") || !sessionState.contains("userName")) {
        qWarning() << "Invalid session state: missing required fields";
        return false;
    }
    
    // Check if the session is for the current user
    if (sessionState["userName"].toString() != m_userName) {
        qWarning() << "Session is for a different user:" << sessionState["userName"].toString();
        return false;
    }
    
    // Restore running applications
    if (sessionState.contains("runningApplications")) {
        QVariantList runningApps = sessionState["runningApplications"].toList();
        
        for (const QVariant &app : runningApps) {
            QVariantMap appInfo = app.toMap();
            
            if (appInfo.contains("appId")) {
                QString appId = appInfo["appId"].toString();
                
                // In a real implementation, we would launch the application
                // For now, we'll just log it
                qDebug() << "Would restore application:" << appId;
                
                // TODO: Launch application with appropriate parameters
            }
        }
    }
    
    // Restore window positions and states
    if (sessionState.contains("windowStates")) {
        QVariantList windowStates = sessionState["windowStates"].toList();
        
        for (const QVariant &window : windowStates) {
            QVariantMap windowInfo = window.toMap();
            
            // In a real implementation, we would restore the window state
            // For now, we'll just log it
            qDebug() << "Would restore window:" << windowInfo;
            
            // TODO: Restore window state
        }
    }
    
    // Restore workspace information
    if (sessionState.contains("workspaces")) {
        QVariantList workspaces = sessionState["workspaces"].toList();
        
        for (const QVariant &workspace : workspaces) {
            QVariantMap workspaceInfo = workspace.toMap();
            
            // In a real implementation, we would restore the workspace
            // For now, we'll just log it
            qDebug() << "Would restore workspace:" << workspaceInfo;
            
            // TODO: Restore workspace
        }
    }
    
    m_sessionState = sessionState;
    
    qDebug() << "Session restored from:" << latestSessionFile;
    return true;
}

bool SessionManager::logOut(bool saveSession)
{
    // Save session if requested
    if (saveSession) {
        this->saveSession();
    }
    
    // Emit signal that session is ending
    emit sessionEnding("logout");
    
    // Try to log out using DBus
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    
    if (sessionBus.isConnected()) {
        // Try GNOME session manager
        QDBusInterface gnomeSessionManager(
            "org.gnome.SessionManager", 
            "/org/gnome/SessionManager", 
            "org.gnome.SessionManager", 
            sessionBus
        );
        
        if (gnomeSessionManager.isValid()) {
            QDBusReply<void> reply = gnomeSessionManager.call("Logout", 0); // 0 = normal logout
            
            if (reply.isValid()) {
                return true;
            }
        }
        
        // Try KDE session manager
        QDBusInterface kdeSessionManager(
            "org.kde.ksmserver", 
            "/KSMServer", 
            "org.kde.KSMServerInterface", 
            sessionBus
        );
        
        if (kdeSessionManager.isValid()) {
            QDBusReply<void> reply = kdeSessionManager.call("logout", 0, 3, 3); // 0 = confirm, 3 = normal, 3 = normal
            
            if (reply.isValid()) {
                return true;
            }
        }
        
        // Try systemd
        QDBusInterface systemd(
            "org.freedesktop.login1", 
            "/org/freedesktop/login1", 
            "org.freedesktop.login1.Manager", 
            QDBusConnection::systemBus()
        );
        
        if (systemd.isValid()) {
            QDBusReply<void> reply = systemd.call("TerminateUser", getuid());
            
            if (reply.isValid()) {
                return true;
            }
        }
    }
    
    // Fallback: Use command-line tools
    return QProcess::startDetached("logout");
}

bool SessionManager::lockScreen()
{
    // Try to lock screen using DBus
    QDBusConnection sessionBus = QDBusConnection::sessionBus();
    
    if (sessionBus.isConnected()) {
        // Try GNOME screen saver
        QDBusInterface gnomeScreenSaver(
            "org.gnome.ScreenSaver", 
            "/org/gnome/ScreenSaver", 
            "org.gnome.ScreenSaver", 
            sessionBus
        );
        
        if (gnomeScreenSaver.isValid()) {
            QDBusReply<void> reply = gnomeScreenSaver.call("Lock");
            
            if (reply.isValid()) {
                emit screenLocked();
                return true;
            }
        }
        
        // Try KDE screen locker
        QDBusInterface kdeScreenLocker(
            "org.freedesktop.ScreenSaver", 
            "/ScreenSaver", 
            "org.freedesktop.ScreenSaver", 
            sessionBus
        );
        
        if (kdeScreenLocker.isValid()) {
            QDBusReply<uint> reply = kdeScreenLocker.call("Lock");
            
            if (reply.isValid()) {
                emit screenLocked();
                return true;
            }
        }
        
        // Try XDG screen saver
        QDBusInterface xdgScreenSaver(
            "org.freedesktop.ScreenSaver", 
            "/org/freedesktop/ScreenSaver", 
            "org.freedesktop.ScreenSaver", 
            sessionBus
        );
        
        if (xdgScreenSaver.isValid()) {
            QDBusReply<uint> reply = xdgScreenSaver.call("Lock");
            
            if (reply.isValid()) {
                emit screenLocked();
                return true;
            }
        }
    }
    
    // Fallback: Use command-line tools
    bool success = false;
    
    // Try various screen locking commands
    if (QFile::exists("/usr/bin/gnome-screensaver-command")) {
        success = QProcess::startDetached("gnome-screensaver-command", QStringList() << "--lock");
    } else if (QFile::exists("/usr/bin/xdg-screensaver")) {
        success = QProcess::startDetached("xdg-screensaver", QStringList() << "lock");
    } else if (QFile::exists("/usr/bin/xscreensaver-command")) {
        success = QProcess::startDetached("xscreensaver-command", QStringList() << "-lock");
    } else if (QFile::exists("/usr/bin/i3lock")) {
        success = QProcess::startDetached("i3lock");
    }
    
    if (success) {
        emit screenLocked();
    }
    
    return success;
}

bool SessionManager::switchUser(const QString &userName, bool saveSession)
{
    // Save session if requested
    if (saveSession) {
        this->saveSession();
    }
    
    // Emit signal that session is ending
    emit sessionEnding("switch-user");
    
    // Try to switch user using DBus
    QDBusConnection systemBus = QDBusConnection::systemBus();
    
    if (systemBus.isConnected()) {
        // Try systemd
        QDBusInterface systemd(
            "org.freedesktop.login1", 
            "/org/freedesktop/login1", 
            "org.freedesktop.login1.Manager", 
            systemBus
        );
        
        if (systemd.isValid()) {
            QDBusReply<void> reply = systemd.call("SwitchToUser", userName, "");
            
            if (reply.isValid()) {
                return true;
            }
        }
    }
    
    // Fallback: Use command-line tools
    // This is highly dependent on the display manager
    // For now, we'll just return false
    qWarning() << "User switching not supported without systemd";
    return false;
}

void SessionManager::loadStartupApplications()
{
    m_startupApplications.clear();
    
    // Load from settings
    QSettings settings("VivoX", "SessionManager");
    m_startupApplications = settings.value("startupApplications").toStringList();
    
    // Also check XDG autostart directories
    QStringList autostartDirs;
    
    // System-wide autostart directory
    autostartDirs << "/etc/xdg/autostart";
    
    // User-specific autostart directory
    QString userAutostartDir = QStandardPaths::writableLocation(QStandardPaths::ConfigLocation) + "/autostart";
    autostartDirs << userAutostartDir;
    
    for (const QString &dir : autostartDirs) {
        QDir autostartDir(dir);
        
        if (!autostartDir.exists()) {
            continue;
        }
        
        QStringList desktopFiles = autostartDir.entryList(QStringList() << "*.desktop", QDir::Files);
        
        for (const QString &file : desktopFiles) {
            QString desktopFilePath = autostartDir.filePath(file);
            QSettings desktopEntry(desktopFilePath, QSettings::IniFormat);
            desktopEntry.beginGroup("Desktop Entry");
            
            // Check if the application should be started
            if (desktopEntry.value("Hidden", false).toBool()) {
                continue;
            }
            
            // Check if the application is compatible with the current desktop environment
            QString onlyShowIn = desktopEntry.value("OnlyShowIn").toString();
            QString notShowIn = desktopEntry.value("NotShowIn").toString();
            
            if (!onlyShowIn.isEmpty() && !onlyShowIn.split(';').contains("VivoX")) {
                continue;
            }
            
            if (!notShowIn.isEmpty() && notShowIn.split(';').contains("VivoX")) {
                continue;
            }
            
            // Get the application ID
            QString appId = desktopEntry.value("X-GNOME-Autostart-enabled", true).toBool() ? 
                            QFileInfo(file).baseName() : 
                            QString();
            
            if (!appId.isEmpty() && !m_startupApplications.contains(appId)) {
                m_startupApplications.append(appId);
            }
            
            desktopEntry.endGroup();
        }
    }
    
    qDebug() << "Loaded startup applications:" << m_startupApplications;
}

void SessionManager::saveStartupApplications()
{
    // Save to settings
    QSettings settings("VivoX", "SessionManager");
    settings.setValue("startupApplications", m_startupApplications);
    
    // Also update XDG autostart directory
    QString userAutostartDir = QStandardPaths::writableLocation(QStandardPaths::ConfigLocation) + "/autostart";
    QDir().mkpath(userAutostartDir);
    
    // Remove all VivoX-specific autostart entries
    QDir autostartDir(userAutostartDir);
    QStringList desktopFiles = autostartDir.entryList(QStringList() << "*.desktop", QDir::Files);
    
    for (const QString &file : desktopFiles) {
        QString desktopFilePath = autostartDir.filePath(file);
        QSettings desktopEntry(desktopFilePath, QSettings::IniFormat);
        desktopEntry.beginGroup("Desktop Entry");
        
        if (desktopEntry.value("X-VivoX-Autostart", false).toBool()) {
            QFile::remove(desktopFilePath);
        }
        
        desktopEntry.endGroup();
    }
    
    // Create new autostart entries
    for (const QString &appId : m_startupApplications) {
        QString desktopFilePath = findDesktopFile(appId);
        
        if (desktopFilePath.isEmpty()) {
            continue;
        }
        
        // Copy the desktop file to the autostart directory
        QString autostartFilePath = userAutostartDir + "/" + QFileInfo(desktopFilePath).fileName();
        QFile::copy(desktopFilePath, autostartFilePath);
        
        // Update the autostart file
        QSettings desktopEntry(autostartFilePath, QSettings::IniFormat);
        desktopEntry.beginGroup("Desktop Entry");
        desktopEntry.setValue("X-GNOME-Autostart-enabled", true);
        desktopEntry.setValue("X-VivoX-Autostart", true);
        desktopEntry.endGroup();
    }
}

QString SessionManager::findDesktopFile(const QString &appId)
{
    // Check if the appId is a path
    if (QFile::exists(appId) && appId.endsWith(".desktop")) {
        return appId;
    }
    
    // Check if the appId is a desktop file name
    if (!appId.contains('/') && !appId.endsWith(".desktop")) {
        // Search in standard application directories
        QStringList appDirs = QStandardPaths::standardLocations(QStandardPaths::ApplicationsLocation);
        
        for (const QString &dir : appDirs) {
            QString desktopFilePath = dir + "/" + appId + ".desktop";
            
            if (QFile::exists(desktopFilePath)) {
                return desktopFilePath;
            }
        }
    }
    
    return QString();
}

} // namespace VivoX::System
