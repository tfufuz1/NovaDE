#pragma once

#include <string>
#include <memory>
#include <vector>
#include <mutex>
#include <fstream>
#include <sstream>
#include <chrono>
#include <map>
#include <functional>

namespace VivoX {
namespace Core {
namespace Logging {

/**
 * @brief Enumeration of log levels
 */
enum class LogLevel {
    DEBUG,
    INFO,
    WARNING,
    ERROR,
    CRITICAL
};

/**
 * @brief Configuration for the logger
 */
class LogConfig {
public:
    LogConfig() : 
        logLevel(LogLevel::INFO),
        logToConsole(true),
        logToFile(false),
        logFile(""),
        maxFileSize(10 * 1024 * 1024), // 10 MB
        maxBackupFiles(5),
        asyncLogging(false) {}

    LogLevel getLogLevel() const { return logLevel; }
    bool getLogToConsole() const { return logToConsole; }
    bool getLogToFile() const { return logToFile; }
    std::string getLogFile() const { return logFile; }
    size_t getMaxFileSize() const { return maxFileSize; }
    int getMaxBackupFiles() const { return maxBackupFiles; }
    bool getAsyncLogging() const { return asyncLogging; }

    void setLogLevel(LogLevel level) { logLevel = level; }
    void setLogToConsole(bool enable) { logToConsole = enable; }
    void setLogToFile(bool enable) { logToFile = enable; }
    void setLogFile(const std::string& file) { logFile = file; logToFile = true; }
    void setMaxFileSize(size_t size) { maxFileSize = size; }
    void setMaxBackupFiles(int count) { maxBackupFiles = count; }
    void setAsyncLogging(bool enable) { asyncLogging = enable; }

private:
    LogLevel logLevel;
    bool logToConsole;
    bool logToFile;
    std::string logFile;
    size_t maxFileSize;
    int maxBackupFiles;
    bool asyncLogging;
};

/**
 * @brief Main logger class
 */
class Logger {
public:
    /**
     * @brief Get a logger instance for a specific component
     * @param component Name of the component
     * @return Shared pointer to a logger instance
     */
    static std::shared_ptr<Logger> getInstance(const std::string& component);

    /**
     * @brief Configure the logger
     * @param config Configuration object
     */
    void configure(const LogConfig& config);

    /**
     * @brief Log a debug message
     * @param message Message to log
     * @param args Format arguments
     */
    template<typename... Args>
    void debug(const std::string& message, Args&&... args) {
        log(LogLevel::DEBUG, message, std::forward<Args>(args)...);
    }

    /**
     * @brief Log an info message
     * @param message Message to log
     * @param args Format arguments
     */
    template<typename... Args>
    void info(const std::string& message, Args&&... args) {
        log(LogLevel::INFO, message, std::forward<Args>(args)...);
    }

    /**
     * @brief Log a warning message
     * @param message Message to log
     * @param args Format arguments
     */
    template<typename... Args>
    void warning(const std::string& message, Args&&... args) {
        log(LogLevel::WARNING, message, std::forward<Args>(args)...);
    }

    /**
     * @brief Log an error message
     * @param message Message to log
     * @param args Format arguments
     */
    template<typename... Args>
    void error(const std::string& message, Args&&... args) {
        log(LogLevel::ERROR, message, std::forward<Args>(args)...);
    }

    /**
     * @brief Log a critical message
     * @param message Message to log
     * @param args Format arguments
     */
    template<typename... Args>
    void critical(const std::string& message, Args&&... args) {
        log(LogLevel::CRITICAL, message, std::forward<Args>(args)...);
    }

private:
    Logger(const std::string& component);
    ~Logger();

    /**
     * @brief Log a message with a specific level
     * @param level Log level
     * @param message Message to log
     * @param args Format arguments
     */
    template<typename... Args>
    void log(LogLevel level, const std::string& message, Args&&... args) {
        if (level < m_config.getLogLevel()) {
            return;
        }

        std::string formattedMessage = formatMessage(message, std::forward<Args>(args)...);
        std::string logEntry = formatLogEntry(level, formattedMessage);

        if (m_config.getAsyncLogging()) {
            std::lock_guard<std::mutex> lock(m_queueMutex);
            m_messageQueue.push_back(logEntry);
        } else {
            writeLogEntry(logEntry);
        }
    }

    /**
     * @brief Format a message with arguments
     * @param message Message format
     * @param args Format arguments
     * @return Formatted message
     */
    template<typename... Args>
    std::string formatMessage(const std::string& message, Args&&... args) {
        // Simple implementation for demonstration
        // In a real implementation, use a proper formatting library like fmt
        return message; // Placeholder
    }

    /**
     * @brief Format a log entry with timestamp, level, component, etc.
     * @param level Log level
     * @param message Message to log
     * @return Formatted log entry
     */
    std::string formatLogEntry(LogLevel level, const std::string& message);

    /**
     * @brief Write a log entry to the configured outputs
     * @param entry Log entry to write
     */
    void writeLogEntry(const std::string& entry);

    /**
     * @brief Check if log rotation is needed and perform it if necessary
     */
    void checkRotation();

    /**
     * @brief Process the message queue for async logging
     */
    void processMessageQueue();

    static std::map<std::string, std::weak_ptr<Logger>> s_instances;
    static std::mutex s_instancesMutex;

    std::string m_component;
    LogConfig m_config;
    std::ofstream m_fileStream;
    std::mutex m_logMutex;
    std::mutex m_queueMutex;
    std::vector<std::string> m_messageQueue;
    bool m_running;
    std::unique_ptr<std::thread> m_asyncThread;
};

} // namespace Logging
} // namespace Core
} // namespace VivoX
