#pragma once

#include <string>
#include <map>
#include <any>
#include <memory>
#include <mutex>
#include <functional>
#include <filesystem>
#include <vector>

namespace VivoX {
namespace Core {
namespace Configuration {

/**
 * @brief Configuration manager for the VivoX Desktop Environment
 * 
 * This class provides a centralized way to manage configuration settings
 * for all components of the VivoX Desktop Environment. It supports:
 * - Loading/saving configuration from/to JSON files
 * - Watching configuration files for changes
 * - Hierarchical configuration with system defaults and user overrides
 * - Type-safe access to configuration values
 * - Thread-safe operations
 */
class ConfigManager {
public:
    /**
     * @brief Get the singleton instance of the ConfigManager
     * @return Shared pointer to the ConfigManager instance
     */
    static std::shared_ptr<ConfigManager> getInstance();

    /**
     * @brief Load configuration from a file
     * @param filePath Path to the configuration file
     * @return True if the configuration was loaded successfully, false otherwise
     */
    bool loadFromFile(const std::filesystem::path& filePath);

    /**
     * @brief Load configuration from multiple files with override hierarchy
     * 
     * Later files in the list override settings from earlier files.
     * This allows for a hierarchy of configuration files (e.g., system defaults,
     * user overrides).
     * 
     * @param filePaths List of paths to configuration files
     * @return True if at least one configuration file was loaded successfully
     */
    bool loadFromFiles(const std::vector<std::filesystem::path>& filePaths);

    /**
     * @brief Save configuration to a file
     * @param filePath Path to the configuration file
     * @return True if the configuration was saved successfully, false otherwise
     */
    bool saveToFile(const std::filesystem::path& filePath);

    /**
     * @brief Watch a configuration file for changes
     * @param filePath Path to the configuration file
     * @param callback Function to call when the file changes
     * @return True if the file is being watched successfully, false otherwise
     */
    bool watchConfigFile(const std::filesystem::path& filePath, 
                         std::function<void()> callback);

    /**
     * @brief Stop watching a configuration file
     * @param filePath Path to the configuration file
     */
    void unwatchConfigFile(const std::filesystem::path& filePath);

    /**
     * @brief Get a configuration value
     * @param key Configuration key (can be hierarchical with dot notation, e.g., "ui.theme.color")
     * @param defaultValue Default value to return if the key is not found
     * @return Configuration value, or defaultValue if the key is not found
     */
    template<typename T>
    T getValue(const std::string& key, const T& defaultValue) const {
        std::lock_guard<std::mutex> lock(m_mutex);
        
        auto it = m_config.find(key);
        if (it != m_config.end()) {
            try {
                return std::any_cast<T>(it->second);
            } catch (const std::bad_any_cast&) {
                return defaultValue;
            }
        }
        
        return defaultValue;
    }

    /**
     * @brief Set a configuration value
     * @param key Configuration key (can be hierarchical with dot notation)
     * @param value Configuration value
     */
    template<typename T>
    void setValue(const std::string& key, const T& value) {
        std::lock_guard<std::mutex> lock(m_mutex);
        m_config[key] = value;
    }

    /**
     * @brief Check if a configuration key exists
     * @param key Configuration key
     * @return True if the key exists, false otherwise
     */
    bool hasKey(const std::string& key) const {
        std::lock_guard<std::mutex> lock(m_mutex);
        return m_config.find(key) != m_config.end();
    }

    /**
     * @brief Remove a configuration key
     * @param key Configuration key
     * @return True if the key was removed, false if it didn't exist
     */
    bool removeKey(const std::string& key) {
        std::lock_guard<std::mutex> lock(m_mutex);
        return m_config.erase(key) > 0;
    }

    /**
     * @brief Clear all configuration values
     */
    void clear() {
        std::lock_guard<std::mutex> lock(m_mutex);
        m_config.clear();
    }

    /**
     * @brief Get all keys with a specific prefix
     * @param prefix Key prefix to search for
     * @return Vector of keys that start with the given prefix
     */
    std::vector<std::string> getKeysWithPrefix(const std::string& prefix) const;

private:
    ConfigManager();
    ~ConfigManager();

    static std::shared_ptr<ConfigManager> s_instance;
    static std::mutex s_instanceMutex;

    std::map<std::string, std::any> m_config;
    mutable std::mutex m_mutex;
    
    // File watcher related members
    class FileWatcher;
    std::unique_ptr<FileWatcher> m_fileWatcher;
};

} // namespace Configuration
} // namespace Core
} // namespace VivoX
