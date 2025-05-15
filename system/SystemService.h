#pragma once

#include <QObject>
#include <QString>
#include <QProcess>

namespace VivoX::System {

/**
 * @brief The SystemService class provides basic system interactions.
 * 
 * It offers functionality for starting external processes, opening URLs,
 * and performing simple file operations with proper permission handling.
 */
class SystemService : public QObject {
    Q_OBJECT

public:
    explicit SystemService(QObject *parent = nullptr);
    ~SystemService();

    /**
     * @brief Initialize the system service
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Start an external process
     * @param program The program to start
     * @param arguments The arguments to pass to the program
     * @param detached Whether to start the process detached
     * @return The process ID if successful, 0 otherwise
     */
    qint64 startProcess(const QString &program, const QStringList &arguments, bool detached = true);

    /**
     * @brief Open a URL in the default application
     * @param url The URL to open
     * @return True if successful
     */
    bool openUrl(const QString &url);

    /**
     * @brief Copy a file
     * @param source The source file
     * @param destination The destination file
     * @param overwrite Whether to overwrite the destination if it exists
     * @return True if successful
     */
    bool copyFile(const QString &source, const QString &destination, bool overwrite = false);

    /**
     * @brief Move a file
     * @param source The source file
     * @param destination The destination file
     * @param overwrite Whether to overwrite the destination if it exists
     * @return True if successful
     */
    bool moveFile(const QString &source, const QString &destination, bool overwrite = false);

    /**
     * @brief Delete a file
     * @param file The file to delete
     * @return True if successful
     */
    bool deleteFile(const QString &file);

    /**
     * @brief Create a directory
     * @param path The directory path
     * @return True if successful
     */
    bool createDirectory(const QString &path);

signals:
    /**
     * @brief Signal emitted when a process has finished
     * @param exitCode The exit code of the process
     * @param exitStatus The exit status of the process
     */
    void processFinished(int exitCode, QProcess::ExitStatus exitStatus);

    /**
     * @brief Signal emitted when a process has encountered an error
     * @param error The error that occurred
     */
    void processError(QProcess::ProcessError error);

private:
    // Check if the user has permission to access a file
    bool checkPermission(const QString &file, QFile::Permissions permissions);
};

} // namespace VivoX::System
