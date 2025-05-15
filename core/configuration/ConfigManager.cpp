#include "ConfigManager.h"
#include <fstream>
#include <iostream>
#include <thread>
#include <chrono>
#include <atomic>
#include <unordered_map>
#include <nlohmann/json.hpp>
#include <algorithm>

namespace VivoX {
namespace Core {
namespace Configuration {

// FileWatcher implementation
class ConfigManager::FileWatcher {
public:
    FileWatcher() : m_running(false) {}
    
    ~FileWatcher() {
        stop();
    }
    
    void start() {
        if (m_running) {
            return;
        }
        
        m_running = true;
        m_watcherThread = std::thread(&FileWatcher::watchLoop, this);
    }
    
    void stop() {
        if (!m_running) {
            return;
        }
        
        m_running = false;
        if (m_watcherThread.joinable()) {
            m_watcherThread.join();
        }
    }
    
    void addWatch(const std::filesystem::path& filePath, std::function<void()> callback) {
        std::lock_guard<std::mutex> lock(m_watchMutex);
        
        // Store last modification time
        if (std::filesystem::exists(filePath)) {
            m_watchedFiles[filePath.string()] = {
                std::filesystem::last_write_time(filePath),
                callback
            };
        } else {
            // File doesn't exist yet, store current time
            m_watchedFiles[filePath.string()] = {
                std::filesystem::file_time_type::clock::now(),
                callback
            };
        }
    }
    
    void removeWatch(const std::filesystem::path& filePath) {
        std::lock_guard<std::mutex> lock(m_watchMutex);
        m_watchedFiles.erase(filePath.string());
    }
    
private:
    void watchLoop() {
        while (m_running) {
            std::this_thread::sleep_for(std::chrono::milliseconds(500));
            
            std::lock_guard<std::mutex> lock(m_watchMutex);
            
            for (auto& [path, info] : m_watchedFiles) {
                std::filesystem::path filePath(path);
                
                if (!std::filesystem::exists(filePath)) {
                    continue;
                }
                
                auto lastWriteTime = std::filesystem::last_write_time(filePath);
                
                if (lastWriteTime != info.lastModified) {
                    info.lastModified = lastWriteTime;
                    
                    // Call the callback
                    if (info.callback) {
                        info.callback();
                    }
                }
            }
        }
    }
    
    struct FileInfo {
        std::filesystem::file_time_type lastModified;
        std::function<void()> callback;
    };
    
    std::unordered_map<std::string, FileInfo> m_watchedFiles;
    std::mutex m_watchMutex;
    std::thread m_watcherThread;
    std::atomic<bool> m_running;
};

// ConfigManager implementation
std::shared_ptr<ConfigManager> ConfigManager::s_instance = nullptr;
std::mutex ConfigManager::s_instanceMutex;

std::shared_ptr<ConfigManager> ConfigManager::getInstance() {
    std::lock_guard<std::mutex> lock(s_instanceMutex);
    
    if (!s_instance) {
        s_instance = std::shared_ptr<ConfigManager>(new ConfigManager());
    }
    
    return s_instance;
}

ConfigManager::ConfigManager() : m_fileWatcher(std::make_unique<FileWatcher>()) {
    m_fileWatcher->start();
}

ConfigManager::~ConfigManager() {
    if (m_fileWatcher) {
        m_fileWatcher->stop();
    }
}

bool ConfigManager::loadFromFile(const std::filesystem::path& filePath) {
    if (!std::filesystem::exists(filePath)) {
        return false;
    }
    
    try {
        std::ifstream file(filePath);
        if (!file.is_open()) {
            return false;
        }
        
        nlohmann::json jsonConfig;
        file >> jsonConfig;
        
        std::lock_guard<std::mutex> lock(m_mutex);
        
        // Parse JSON into config map
        parseJsonToConfig(jsonConfig, "");
        
        return true;
    } catch (const std::exception& e) {
        std::cerr << "Error loading configuration: " << e.what() << std::endl;
        return false;
    }
}

bool ConfigManager::loadFromFiles(const std::vector<std::filesystem::path>& filePaths) {
    bool atLeastOneLoaded = false;
    
    for (const auto& filePath : filePaths) {
        if (loadFromFile(filePath)) {
            atLeastOneLoaded = true;
        }
    }
    
    return atLeastOneLoaded;
}

bool ConfigManager::saveToFile(const std::filesystem::path& filePath) {
    try {
        // Create directory if it doesn't exist
        std::filesystem::create_directories(filePath.parent_path());
        
        nlohmann::json jsonConfig;
        
        {
            std::lock_guard<std::mutex> lock(m_mutex);
            
            // Convert config map to JSON
            for (const auto& [key, value] : m_config) {
                // Parse hierarchical keys (e.g., "ui.theme.color")
                std::vector<std::string> keyParts;
                std::string currentPart;
                std::istringstream keyStream(key);
                
                while (std::getline(keyStream, currentPart, '.')) {
                    keyParts.push_back(currentPart);
                }
                
                // Build nested JSON structure
                nlohmann::json* currentJson = &jsonConfig;
                for (size_t i = 0; i < keyParts.size() - 1; ++i) {
                    if (!currentJson->contains(keyParts[i])) {
                        (*currentJson)[keyParts[i]] = nlohmann::json::object();
                    }
                    currentJson = &(*currentJson)[keyParts[i]];
                }
                
                // Set the value
                const std::string& lastKey = keyParts.back();
                if (value.type() == typeid(std::string)) {
                    (*currentJson)[lastKey] = std::any_cast<std::string>(value);
                } else if (value.type() == typeid(int)) {
                    (*currentJson)[lastKey] = std::any_cast<int>(value);
                } else if (value.type() == typeid(double)) {
                    (*currentJson)[lastKey] = std::any_cast<double>(value);
                } else if (value.type() == typeid(bool)) {
                    (*currentJson)[lastKey] = std::any_cast<bool>(value);
                }
                // Other types are not supported for simplicity
            }
        }
        
        std::ofstream file(filePath);
        if (!file.is_open()) {
            return false;
        }
        
        file << jsonConfig.dump(4); // Pretty print with 4 spaces
        return true;
    } catch (const std::exception& e) {
        std::cerr << "Error saving configuration: " << e.what() << std::endl;
        return false;
    }
}

bool ConfigManager::watchConfigFile(const std::filesystem::path& filePath, 
                                   std::function<void()> callback) {
    if (!m_fileWatcher) {
        return false;
    }
    
    m_fileWatcher->addWatch(filePath, callback);
    return true;
}

void ConfigManager::unwatchConfigFile(const std::filesystem::path& filePath) {
    if (m_fileWatcher) {
        m_fileWatcher->removeWatch(filePath);
    }
}

std::vector<std::string> ConfigManager::getKeysWithPrefix(const std::string& prefix) const {
    std::vector<std::string> result;
    std::lock_guard<std::mutex> lock(m_mutex);
    
    for (const auto& [key, _] : m_config) {
        if (key.find(prefix) == 0) {
            result.push_back(key);
        }
    }
    
    return result;
}

// Helper method to parse JSON into config map
void ConfigManager::parseJsonToConfig(const nlohmann::json& json, const std::string& prefix) {
    for (auto it = json.begin(); it != json.end(); ++it) {
        std::string key = prefix.empty() ? it.key() : prefix + "." + it.key();
        
        if (it->is_object()) {
            // Recursively parse nested objects
            parseJsonToConfig(*it, key);
        } else if (it->is_string()) {
            m_config[key] = it->get<std::string>();
        } else if (it->is_number_integer()) {
            m_config[key] = it->get<int>();
        } else if (it->is_number_float()) {
            m_config[key] = it->get<double>();
        } else if (it->is_boolean()) {
            m_config[key] = it->get<bool>();
        } else if (it->is_array()) {
            // For simplicity, store arrays as strings
            m_config[key] = it->dump();
        }
    }
}

} // namespace Configuration
} // namespace Core
} // namespace VivoX
