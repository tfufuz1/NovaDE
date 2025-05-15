#pragma once

#include <QObject>
#include <QHash>
#include <QList>
#include <QString>
#include <QIcon>
#include <QProcess>

namespace VivoX::System {

/**
 * @brief The ApplicationInfo class contains information about an application.
 */
class ApplicationInfo {
public:
    QString id;           ///< Unique identifier for the application
    QString name;         ///< Display name of the application
    QString description;  ///< Description of the application
    QString executable;   ///< Path to the executable
    QStringList args;     ///< Arguments to pass to the executable
    QString iconName;     ///< Name of the icon
    QIcon icon;           ///< Icon of the application
    QStringList categories; ///< Categories the application belongs to
    bool terminal;        ///< Whether the application should be run in a terminal
    QString desktopFile;  ///< Path to the .desktop file
};

/**
 * @brief The ApplicationManager class manages applications and their launching.
 * 
 * It is responsible for discovering installed applications, providing information
 * about them, and launching them.
 */
class ApplicationManager : public QObject {
    Q_OBJECT

public:
    explicit ApplicationManager(QObject *parent = nullptr);
    ~ApplicationManager();

    /**
     * @brief Initialize the application manager
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Get a list of all applications
     * @return List of application info objects
     */
    QList<ApplicationInfo> getAllApplications() const;

    /**
     * @brief Get applications in a specific category
     * @param category The category to filter by
     * @return List of application info objects in the category
     */
    QList<ApplicationInfo> getApplicationsByCategory(const QString &category) const;

    /**
     * @brief Get information about a specific application
     * @param id The application ID
     * @return The application info, or an empty object if not found
     */
    ApplicationInfo getApplicationInfo(const QString &id) const;

    /**
     * @brief Launch an application
     * @param id The application ID
     * @param args Additional arguments to pass to the application
     * @return The process ID if successful, 0 otherwise
     */
    qint64 launchApplication(const QString &id, const QStringList &args = QStringList());

    /**
     * @brief Search for applications
     * @param query The search query
     * @return List of matching application info objects
     */
    QList<ApplicationInfo> searchApplications(const QString &query) const;

    /**
     * @brief Get recently used applications
     * @param count Maximum number of applications to return
     * @return List of recently used application info objects
     */
    QList<ApplicationInfo> getRecentApplications(int count = 10) const;

    /**
     * @brief Get frequently used applications
     * @param count Maximum number of applications to return
     * @return List of frequently used application info objects
     */
    QList<ApplicationInfo> getFrequentApplications(int count = 10) const;

signals:
    /**
     * @brief Signal emitted when the list of applications has changed
     */
    void applicationsChanged();

    /**
     * @brief Signal emitted when an application has been launched
     * @param id The application ID
     * @param pid The process ID
     */
    void applicationLaunched(const QString &id, qint64 pid);

    /**
     * @brief Signal emitted when an application has finished
     * @param id The application ID
     * @param exitCode The exit code of the process
     */
    void applicationFinished(const QString &id, int exitCode);

private:
    // Map of application ID to application info
    QHash<QString, ApplicationInfo> m_applications;
    
    // List of recently used application IDs
    QList<QString> m_recentApplications;
    
    // Map of application ID to usage count
    QHash<QString, int> m_applicationUsage;
    
    // Map of running application processes
    QHash<qint64, QString> m_runningApplications;
    
    // Discover installed applications
    void discoverApplications();
    
    // Parse a .desktop file
    bool parseDesktopFile(const QString &path, ApplicationInfo &info);
    
    // Update application usage statistics
    void updateApplicationUsage(const QString &id);
    
    // Load application usage statistics
    void loadApplicationUsage();
    
    // Save application usage statistics
    void saveApplicationUsage();
};

} // namespace VivoX::System
