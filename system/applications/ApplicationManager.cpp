#include "ApplicationManager.h"

#include <QDebug>
#include <QDir>
#include <QFile>
#include <QSettings>
#include <QStandardPaths>
#include <QProcess>
#include <QRegularExpression>

namespace VivoX::System {

ApplicationManager::ApplicationManager(QObject *parent)
    : QObject(parent)
{
    qDebug() << "ApplicationManager created";
}

ApplicationManager::~ApplicationManager()
{
    // Save application usage statistics
    saveApplicationUsage();
    
    qDebug() << "ApplicationManager destroyed";
}

bool ApplicationManager::initialize()
{
    // Discover installed applications
    discoverApplications();
    
    // Load application usage statistics
    loadApplicationUsage();
    
    qDebug() << "ApplicationManager initialized with" << m_applications.size() << "applications";
    return true;
}

QList<ApplicationInfo> ApplicationManager::getAllApplications() const
{
    return m_applications.values();
}

QList<ApplicationInfo> ApplicationManager::getApplicationsByCategory(const QString &category) const
{
    QList<ApplicationInfo> result;
    
    for (const ApplicationInfo &info : m_applications.values()) {
        if (info.categories.contains(category, Qt::CaseInsensitive)) {
            result.append(info);
        }
    }
    
    return result;
}

ApplicationInfo ApplicationManager::getApplicationInfo(const QString &id) const
{
    return m_applications.value(id, ApplicationInfo());
}

qint64 ApplicationManager::launchApplication(const QString &id, const QStringList &args)
{
    if (!m_applications.contains(id)) {
        qWarning() << "Application not found:" << id;
        return 0;
    }
    
    const ApplicationInfo &info = m_applications[id];
    
    // Combine default arguments with additional arguments
    QStringList allArgs = info.args;
    allArgs.append(args);
    
    qint64 pid = 0;
    
    if (info.terminal) {
        // Launch in terminal
        QString terminal = "x-terminal-emulator";
        QStringList terminalArgs;
        terminalArgs << "-e" << info.executable;
        terminalArgs.append(allArgs);
        
        if (!QProcess::startDetached(terminal, terminalArgs, QString(), &pid)) {
            qWarning() << "Failed to launch application in terminal:" << id;
            return 0;
        }
    } else {
        // Launch directly
        if (!QProcess::startDetached(info.executable, allArgs, QString(), &pid)) {
            qWarning() << "Failed to launch application:" << id;
            return 0;
        }
    }
    
    // Update application usage statistics
    updateApplicationUsage(id);
    
    // Add to running applications
    m_runningApplications[pid] = id;
    
    // Emit signal
    emit applicationLaunched(id, pid);
    
    qDebug() << "Launched application:" << id << "with PID:" << pid;
    
    return pid;
}

QList<ApplicationInfo> ApplicationManager::searchApplications(const QString &query) const
{
    QList<ApplicationInfo> result;
    
    if (query.isEmpty()) {
        return result;
    }
    
    QRegularExpression regex(query, QRegularExpression::CaseInsensitiveOption);
    
    for (const ApplicationInfo &info : m_applications.values()) {
        if (info.name.contains(regex) || info.description.contains(regex)) {
            result.append(info);
        }
    }
    
    return result;
}

QList<ApplicationInfo> ApplicationManager::getRecentApplications(int count) const
{
    QList<ApplicationInfo> result;
    
    for (const QString &id : m_recentApplications) {
        if (m_applications.contains(id)) {
            result.append(m_applications[id]);
            
            if (result.size() >= count) {
                break;
            }
        }
    }
    
    return result;
}

QList<ApplicationInfo> ApplicationManager::getFrequentApplications(int count) const
{
    // Create a list of application IDs sorted by usage count
    QList<QPair<QString, int>> sortedUsage;
    
    for (auto it = m_applicationUsage.begin(); it != m_applicationUsage.end(); ++it) {
        if (m_applications.contains(it.key())) {
            sortedUsage.append(qMakePair(it.key(), it.value()));
        }
    }
    
    // Sort by usage count (descending)
    std::sort(sortedUsage.begin(), sortedUsage.end(), 
        [](const QPair<QString, int> &a, const QPair<QString, int> &b) {
            return a.second > b.second;
        });
    
    // Create result list
    QList<ApplicationInfo> result;
    
    for (const auto &pair : sortedUsage) {
        result.append(m_applications[pair.first]);
        
        if (result.size() >= count) {
            break;
        }
    }
    
    return result;
}

void ApplicationManager::discoverApplications()
{
    // Clear existing applications
    m_applications.clear();
    
    // Get application directories
    QStringList appDirs = QStandardPaths::standardLocations(QStandardPaths::ApplicationsLocation);
    
    // Add system application directories
    appDirs << "/usr/share/applications" << "/usr/local/share/applications";
    
    // Find .desktop files
    for (const QString &dirPath : appDirs) {
        QDir dir(dirPath);
        
        if (!dir.exists()) {
            continue;
        }
        
        QStringList desktopFiles = dir.entryList(QStringList() << "*.desktop", QDir::Files);
        
        for (const QString &file : desktopFiles) {
            QString filePath = dir.filePath(file);
            
            ApplicationInfo info;
            if (parseDesktopFile(filePath, info)) {
                m_applications[info.id] = info;
            }
        }
    }
    
    qDebug() << "Discovered" << m_applications.size() << "applications";
    
    emit applicationsChanged();
}

bool ApplicationManager::parseDesktopFile(const QString &path, ApplicationInfo &info)
{
    QSettings settings(path, QSettings::IniFormat);
    settings.beginGroup("Desktop Entry");
    
    // Check if this is an application
    QString type = settings.value("Type").toString();
    if (type != "Application") {
        return false;
    }
    
    // Check if the application should be hidden
    bool noDisplay = settings.value("NoDisplay", false).toBool();
    if (noDisplay) {
        return false;
    }
    
    // Get application information
    info.name = settings.value("Name").toString();
    info.description = settings.value("Comment").toString();
    info.executable = settings.value("Exec").toString();
    info.iconName = settings.value("Icon").toString();
    info.categories = settings.value("Categories").toString().split(';', Qt::SkipEmptyParts);
    info.terminal = settings.value("Terminal", false).toBool();
    info.desktopFile = path;
    
    // Generate ID from desktop file name
    QFileInfo fileInfo(path);
    info.id = fileInfo.completeBaseName();
    
    // Parse executable and arguments
    if (!info.executable.isEmpty()) {
        // Remove field codes (%f, %F, %u, %U, etc.)
        QRegularExpression fieldCodeRegex("%[fFuUdDnNickvm]");
        info.executable = info.executable.remove(fieldCodeRegex);
        
        // Split into executable and arguments
        QStringList parts = info.executable.split(' ', Qt::SkipEmptyParts);
        if (!parts.isEmpty()) {
            info.executable = parts.takeFirst();
            info.args = parts;
        }
    }
    
    // Load icon
    if (!info.iconName.isEmpty()) {
        info.icon = QIcon::fromTheme(info.iconName);
    }
    
    settings.endGroup();
    
    return !info.name.isEmpty() && !info.executable.isEmpty();
}

void ApplicationManager::updateApplicationUsage(const QString &id)
{
    // Update usage count
    m_applicationUsage[id] = m_applicationUsage.value(id, 0) + 1;
    
    // Update recent applications list
    m_recentApplications.removeAll(id);
    m_recentApplications.prepend(id);
    
    // Limit recent applications list to 100 items
    while (m_recentApplications.size() > 100) {
        m_recentApplications.removeLast();
    }
}

void ApplicationManager::loadApplicationUsage()
{
    // Clear existing data
    m_recentApplications.clear();
    m_applicationUsage.clear();
    
    // Load from settings
    QSettings settings("VivoX", "ApplicationManager");
    
    // Load recent applications
    m_recentApplications = settings.value("RecentApplications").toStringList();
    
    // Load application usage
    settings.beginGroup("ApplicationUsage");
    QStringList keys = settings.childKeys();
    for (const QString &key : keys) {
        m_applicationUsage[key] = settings.value(key).toInt();
    }
    settings.endGroup();
    
    qDebug() << "Loaded application usage statistics:" 
             << m_recentApplications.size() << "recent," 
             << m_applicationUsage.size() << "usage counts";
}

void ApplicationManager::saveApplicationUsage()
{
    // Save to settings
    QSettings settings("VivoX", "ApplicationManager");
    
    // Save recent applications
    settings.setValue("RecentApplications", m_recentApplications);
    
    // Save application usage
    settings.beginGroup("ApplicationUsage");
    for (auto it = m_applicationUsage.begin(); it != m_applicationUsage.end(); ++it) {
        settings.setValue(it.key(), it.value());
    }
    settings.endGroup();
    
    qDebug() << "Saved application usage statistics";
}

} // namespace VivoX::System
