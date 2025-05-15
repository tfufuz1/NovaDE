#include "SystemService.h"

#include <QDebug>
#include <QFile>
#include <QDir>
#include <QUrl>
#include <QDesktopServices>

namespace VivoX::System {

SystemService::SystemService(QObject *parent)
    : QObject(parent)
{
    qDebug() << "SystemService created";
}

SystemService::~SystemService()
{
    qDebug() << "SystemService destroyed";
}

bool SystemService::initialize()
{
    qDebug() << "SystemService initialized";
    return true;
}

qint64 SystemService::startProcess(const QString &program, const QStringList &arguments, bool detached)
{
    qint64 pid = 0;
    
    if (detached) {
        // Start process detached
        if (!QProcess::startDetached(program, arguments, QString(), &pid)) {
            qWarning() << "Failed to start process:" << program << arguments;
            return 0;
        }
    } else {
        // Start process attached
        QProcess *process = new QProcess(this);
        
        // Connect signals
        connect(process, static_cast<void(QProcess::*)(int, QProcess::ExitStatus)>(&QProcess::finished),
                this, &SystemService::processFinished);
        connect(process, &QProcess::errorOccurred,
                this, &SystemService::processError);
        connect(process, &QProcess::finished, process, &QProcess::deleteLater);
        
        process->start(program, arguments);
        
        if (!process->waitForStarted()) {
            qWarning() << "Failed to start process:" << program << arguments;
            process->deleteLater();
            return 0;
        }
        
        pid = process->processId();
    }
    
    qDebug() << "Started process:" << program << arguments << "with PID:" << pid;
    return pid;
}

bool SystemService::openUrl(const QString &url)
{
    QUrl qurl(url);
    
    if (!qurl.isValid()) {
        qWarning() << "Invalid URL:" << url;
        return false;
    }
    
    bool success = QDesktopServices::openUrl(qurl);
    
    if (success) {
        qDebug() << "Opened URL:" << url;
    } else {
        qWarning() << "Failed to open URL:" << url;
    }
    
    return success;
}

bool SystemService::copyFile(const QString &source, const QString &destination, bool overwrite)
{
    // Check if source file exists
    QFile sourceFile(source);
    if (!sourceFile.exists()) {
        qWarning() << "Source file does not exist:" << source;
        return false;
    }
    
    // Check if we have permission to read the source file
    if (!checkPermission(source, QFile::ReadUser)) {
        qWarning() << "No permission to read source file:" << source;
        return false;
    }
    
    // Check if destination file exists
    QFile destinationFile(destination);
    if (destinationFile.exists()) {
        if (!overwrite) {
            qWarning() << "Destination file already exists and overwrite is false:" << destination;
            return false;
        }
        
        // Check if we have permission to write to the destination file
        if (!checkPermission(destination, QFile::WriteUser)) {
            qWarning() << "No permission to write to destination file:" << destination;
            return false;
        }
        
        // Remove the destination file
        if (!destinationFile.remove()) {
            qWarning() << "Failed to remove existing destination file:" << destination;
            return false;
        }
    }
    
    // Create destination directory if it doesn't exist
    QDir destinationDir = QFileInfo(destination).dir();
    if (!destinationDir.exists()) {
        if (!destinationDir.mkpath(".")) {
            qWarning() << "Failed to create destination directory:" << destinationDir.path();
            return false;
        }
    }
    
    // Copy the file
    bool success = sourceFile.copy(destination);
    
    if (success) {
        qDebug() << "Copied file from" << source << "to" << destination;
    } else {
        qWarning() << "Failed to copy file from" << source << "to" << destination;
    }
    
    return success;
}

bool SystemService::moveFile(const QString &source, const QString &destination, bool overwrite)
{
    // First copy the file
    if (!copyFile(source, destination, overwrite)) {
        return false;
    }
    
    // Then delete the source file
    return deleteFile(source);
}

bool SystemService::deleteFile(const QString &file)
{
    // Check if file exists
    QFile fileObj(file);
    if (!fileObj.exists()) {
        qWarning() << "File does not exist:" << file;
        return false;
    }
    
    // Check if we have permission to write to the file
    if (!checkPermission(file, QFile::WriteUser)) {
        qWarning() << "No permission to delete file:" << file;
        return false;
    }
    
    // Delete the file
    bool success = fileObj.remove();
    
    if (success) {
        qDebug() << "Deleted file:" << file;
    } else {
        qWarning() << "Failed to delete file:" << file;
    }
    
    return success;
}

bool SystemService::createDirectory(const QString &path)
{
    QDir dir(path);
    
    if (dir.exists()) {
        qDebug() << "Directory already exists:" << path;
        return true;
    }
    
    bool success = dir.mkpath(".");
    
    if (success) {
        qDebug() << "Created directory:" << path;
    } else {
        qWarning() << "Failed to create directory:" << path;
    }
    
    return success;
}

bool SystemService::checkPermission(const QString &file, QFile::Permissions permissions)
{
    QFile fileObj(file);
    return (fileObj.permissions() & permissions) == permissions;
}

} // namespace VivoX::System
