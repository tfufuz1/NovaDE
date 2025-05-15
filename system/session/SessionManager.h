#pragma once

#include <QObject>
#include <QString>
#include <QVariantMap>

namespace VivoX::System {

/**
 * @brief The SessionManager class manages the user session.
 * 
 * It provides an interface to control session-related functionality such as
 * startup applications, session saving/restoring, and session control.
 */
class SessionManager : public QObject {
    Q_OBJECT

public:
    explicit SessionManager(QObject *parent = nullptr);
    ~SessionManager();

    /**
     * @brief Initialize the session manager
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Get the current user name
     * @return The user name
     */
    QString getUserName() const;

    /**
     * @brief Get the current session ID
     * @return The session ID
     */
    QString getSessionId() const;

    /**
     * @brief Get the session start time
     * @return The start time as a Unix timestamp
     */
    qint64 getSessionStartTime() const;

    /**
     * @brief Get the list of startup applications
     * @return List of application IDs
     */
    QStringList getStartupApplications() const;

    /**
     * @brief Add an application to the startup list
     * @param appId The application ID
     * @return True if successful
     */
    bool addStartupApplication(const QString &appId);

    /**
     * @brief Remove an application from the startup list
     * @param appId The application ID
     * @return True if successful
     */
    bool removeStartupApplication(const QString &appId);

    /**
     * @brief Save the current session state
     * @return True if successful
     */
    bool saveSession();

    /**
     * @brief Restore the previous session state
     * @return True if successful
     */
    bool restoreSession();

    /**
     * @brief Log out the current user
     * @param saveSession Whether to save the session before logging out
     * @return True if successful
     */
    bool logOut(bool saveSession = true);

    /**
     * @brief Lock the screen
     * @return True if successful
     */
    bool lockScreen();

    /**
     * @brief Switch to another user
     * @param userName The user name to switch to
     * @param saveSession Whether to save the current session
     * @return True if successful
     */
    bool switchUser(const QString &userName, bool saveSession = true);

signals:
    /**
     * @brief Signal emitted when the session is about to end
     * @param reason The reason for ending the session
     */
    void sessionEnding(const QString &reason);

    /**
     * @brief Signal emitted when the screen is locked
     */
    void screenLocked();

    /**
     * @brief Signal emitted when the screen is unlocked
     */
    void screenUnlocked();

    /**
     * @brief Signal emitted when the startup applications list changes
     */
    void startupApplicationsChanged();

private:
    // Current user name
    QString m_userName;
    
    // Current session ID
    QString m_sessionId;
    
    // Session start time
    qint64 m_sessionStartTime;
    
    // List of startup applications
    QStringList m_startupApplications;
    
    // Session state
    QVariantMap m_sessionState;
    
    // Load startup applications
    void loadStartupApplications();
    
    // Save startup applications
    void saveStartupApplications();
};

} // namespace VivoX::System
