#include "Logger.h"
#include <iostream>
#include <iomanip>
#include <thread>
#include <filesystem>
#include <algorithm>
#include <chrono>
#include <ctime>
#include <sstream>

namespace VivoX {
namespace Core {
namespace Logging {

std::map<std::string, std::weak_ptr<Logger>> Logger::s_instances;
std::mutex Logger::s_instancesMutex;

std::shared_ptr<Logger> Logger::getInstance(const std::string& component) {
    std::lock_guard<std::mutex> lock(s_instancesMutex);
    
    auto it = s_instances.find(component);
    if (it != s_instances.end()) {
        if (auto instance = it->second.lock()) {
            return instance;
        }
    }
    
    auto instance = std::shared_ptr<Logger>(new Logger(component));
    s_instances[component] = instance;
    return instance;
}

Logger::Logger(const std::string& component) 
    : m_component(component), m_running(false) {
    // Default configuration
    LogConfig config;
    configure(config);
}

Logger::~Logger() {
    // Ensure all pending messages are processed
    flush();
    
    // Stop async logging thread if running
    if (m_running && m_asyncThread) {
        m_running = false;
        if (m_asyncThread->joinable()) {
            m_asyncThread->join();
        }
    }
    
    // Close file stream if open
    if (m_fileStream.is_open()) {
        m_fileStream.close();
    }
}

void Logger::configure(const LogConfig& config) {
    std::lock_guard<std::mutex> lock(m_logMutex);
    
    // Store new configuration
    m_config = config;
    
    // Close existing file stream if open
    if (m_fileStream.is_open()) {
        m_fileStream.close();
    }
    
    // Open file stream if logging to file is enabled
    if (m_config.getLogToFile()) {
        // Create directory if it doesn't exist
        std::filesystem::path logFilePath(m_config.getLogFile());
        std::filesystem::create_directories(logFilePath.parent_path());
        
        // Open file stream in append mode
        m_fileStream.open(m_config.getLogFile(), std::ios::app);
        
        // Check if file was opened successfully
        if (!m_fileStream.is_open()) {
            std::cerr << "Failed to open log file: " << m_config.getLogFile() << std::endl;
            m_config.setLogToFile(false);
        }
    }
    
    // Start or stop async logging thread
    if (m_config.getAsyncLogging() && !m_running) {
        m_running = true;
        m_asyncThread = std::make_unique<std::thread>(&Logger::processMessageQueue, this);
    } else if (!m_config.getAsyncLogging() && m_running) {
        m_running = false;
        if (m_asyncThread && m_asyncThread->joinable()) {
            m_asyncThread->join();
        }
        m_asyncThread.reset();
        
        // Process any remaining messages in the queue
        flush();
    }
}

void Logger::flush() {
    // Process any remaining messages in the queue
    std::vector<std::string> remainingMessages;
    {
        std::lock_guard<std::mutex> queueLock(m_queueMutex);
        remainingMessages.swap(m_messageQueue);
    }
    
    for (const auto& message : remainingMessages) {
        writeLogEntry(message);
    }
}

std::string Logger::formatLogEntry(LogLevel level, const std::string& message) {
    // Get current time in ISO 8601 format with milliseconds
    auto now = std::chrono::system_clock::now();
    auto time = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
        now.time_since_epoch()) % 1000;
    
    std::stringstream ss;
    ss << std::put_time(std::localtime(&time), "%Y-%m-%dT%H:%M:%S");
    ss << '.' << std::setfill('0') << std::setw(3) << ms.count() << "Z";
    
    // Add log level
    ss << " [";
    switch (level) {
        case LogLevel::TRACE:
            ss << "TRACE";
            break;
        case LogLevel::DEBUG:
            ss << "DEBUG";
            break;
        case LogLevel::INFO:
            ss << "INFO";
            break;
        case LogLevel::WARNING:
            ss << "WARNING";
            break;
        case LogLevel::ERROR:
            ss << "ERROR";
            break;
        case LogLevel::CRITICAL:
            ss << "CRITICAL";
            break;
    }
    ss << "] ";
    
    // Add component name
    ss << "[" << m_component << "] ";
    
    // Add thread ID if enabled
    if (m_config.getIncludeThreadId()) {
        ss << "[Thread:" << std::this_thread::get_id() << "] ";
    }
    
    // Add message
    ss << message;
    
    return ss.str();
}

void Logger::writeLogEntry(const std::string& entry) {
    std::lock_guard<std::mutex> lock(m_logMutex);
    
    // Write to console if enabled
    if (m_config.getLogToConsole()) {
        std::cout << entry << std::endl;
    }
    
    // Write to file if enabled
    if (m_config.getLogToFile() && m_fileStream.is_open()) {
        // Check if rotation is needed
        checkRotation();
        
        // Write log entry
        m_fileStream << entry << std::endl;
        m_fileStream.flush();
    }
}

void Logger::checkRotation() {
    if (!m_config.getLogToFile() || !m_fileStream.is_open()) {
        return;
    }
    
    // Get current file size
    std::filesystem::path logFilePath(m_config.getLogFile());
    if (!std::filesystem::exists(logFilePath)) {
        return;
    }
    
    size_t currentSize = std::filesystem::file_size(logFilePath);
    if (currentSize >= m_config.getMaxFileSize()) {
        // Close current file
        m_fileStream.close();
        
        // Rotate backup files
        for (int i = m_config.getMaxBackupFiles(); i > 0; --i) {
            std::filesystem::path oldPath = logFilePath;
            oldPath += "." + std::to_string(i);
            
            std::filesystem::path newPath = logFilePath;
            if (i < m_config.getMaxBackupFiles()) {
                newPath += "." + std::to_string(i + 1);
            } else {
                // Delete oldest backup file
                if (std::filesystem::exists(oldPath)) {
                    std::filesystem::remove(oldPath);
                }
                continue;
            }
            
            // Rename backup file
            if (std::filesystem::exists(oldPath)) {
                std::filesystem::rename(oldPath, newPath);
            }
        }
        
        // Rename current file to .1
        std::filesystem::path backupPath = logFilePath;
        backupPath += ".1";
        std::filesystem::rename(logFilePath, backupPath);
        
        // Open new file
        m_fileStream.open(m_config.getLogFile(), std::ios::app);
        if (!m_fileStream.is_open()) {
            std::cerr << "Failed to open log file after rotation: " << m_config.getLogFile() << std::endl;
            m_config.setLogToFile(false);
        }
    }
}

void Logger::processMessageQueue() {
    while (m_running) {
        std::vector<std::string> messages;
        
        // Get messages from queue
        {
            std::lock_guard<std::mutex> lock(m_queueMutex);
            if (!m_messageQueue.empty()) {
                messages.swap(m_messageQueue);
            }
        }
        
        // Process messages
        for (const auto& message : messages) {
            writeLogEntry(message);
        }
        
        // Sleep for a short time
        std::this_thread::sleep_for(std::chrono::milliseconds(50));
    }
}

} // namespace Logging
} // namespace Core
} // namespace VivoX
