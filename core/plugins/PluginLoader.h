#pragma once

#include <string>
#include <memory>
#include <map>
#include <vector>
#include <functional>
#include <mutex>
#include <set>

namespace VivoX {
namespace Core {
namespace Plugins {

/**
 * @brief Interface for all plugins
 */
class IPlugin {
public:
    virtual ~IPlugin() = default;
    
    /**
     * @brief Get the ID of the plugin
     * @return Plugin ID
     */
    virtual std::string getPluginId() const = 0;
    
    /**
     * @brief Get the name of the plugin
     * @return Plugin name
     */
    virtual std::string getName() const = 0;
    
    /**
     * @brief Get the version of the plugin
     * @return Plugin version
     */
    virtual std::string getVersion() const = 0;
    
    /**
     * @brief Get the author of the plugin
     * @return Plugin author
     */
    virtual std::string getAuthor() const = 0;
    
    /**
     * @brief Get the description of the plugin
     * @return Plugin description
     */
    virtual std::string getDescription() const = 0;
    
    /**
     * @brief Get the dependencies of the plugin
     * @return Vector of plugin IDs that this plugin depends on
     */
    virtual std::vector<std::string> getDependencies() const = 0;
    
    /**
     * @brief Initialize the plugin
     * @return True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;
    
    /**
     * @brief Shutdown the plugin
     */
    virtual void shutdown() = 0;
    
    /**
     * @brief Check if the plugin is initialized
     * @return True if the plugin is initialized, false otherwise
     */
    virtual bool isInitialized() const = 0;
    
    /**
     * @brief Get a configuration value
     * @param key Configuration key
     * @param defaultValue Default value to return if key not found
     * @return Configuration value, or defaultValue if key not found
     */
    template<typename T>
    T getConfigValue(const std::string& key, const T& defaultValue) const {
        return defaultValue; // Default implementation, should be overridden
    }
    
    /**
     * @brief Set a configuration value
     * @param key Configuration key
     * @param value Configuration value
     */
    template<typename T>
    void setConfigValue(const std::string& key, const T& value) {
        // Default implementation, should be overridden
    }
};

/**
 * @brief Plugin metadata
 */
struct PluginMetadata {
    std::string id;
    std::string name;
    std::string version;
    std::string author;
    std::string description;
    std::vector<std::string> dependencies;
    std::string apiVersion;
    bool enabled = true;
};

/**
 * @brief Plugin loader for the VivoX Desktop Environment
 * 
 * This class provides functionality to load, validate, and manage plugins.
 * Features:
 * - Dynamic loading of plugins from shared libraries
 * - Plugin dependency resolution
 * - Plugin validation
 * - Plugin lifecycle management (initialization, shutdown)
 * - Plugin configuration
 */
class PluginLoader {
public:
    /**
     * @brief Get the singleton instance of the PluginLoader
     * @return Shared pointer to the PluginLoader instance
     */
    static std::shared_ptr<PluginLoader> getInstance();
    
    /**
     * @brief Load a plugin from a shared library
     * @param pluginPath Path to the plugin shared library
     * @return Shared pointer to the loaded plugin, or nullptr if loading failed
     */
    std::shared_ptr<IPlugin> loadPlugin(const std::string& pluginPath);
    
    /**
     * @brief Load all plugins from a directory
     * @param pluginDir Directory containing plugin shared libraries
     * @param recursive Whether to search subdirectories recursively
     * @return Number of successfully loaded plugins
     */
    int loadPluginsFromDirectory(const std::string& pluginDir, bool recursive = false);
    
    /**
     * @brief Unload a plugin
     * @param pluginId ID of the plugin to unload
     * @return True if the plugin was unloaded, false if it wasn't loaded
     */
    bool unloadPlugin(const std::string& pluginId);
    
    /**
     * @brief Get a plugin by ID
     * @param pluginId ID of the plugin to get
     * @return Shared pointer to the plugin, or nullptr if it wasn't found
     */
    std::shared_ptr<IPlugin> getPlugin(const std::string& pluginId) const;
    
    /**
     * @brief Check if a plugin is loaded
     * @param pluginId ID of the plugin to check
     * @return True if the plugin is loaded, false otherwise
     */
    bool isPluginLoaded(const std::string& pluginId) const;
    
    /**
     * @brief Get all loaded plugins
     * @return Map of plugin IDs to plugin instances
     */
    const std::map<std::string, std::shared_ptr<IPlugin>>& getLoadedPlugins() const;
    
    /**
     * @brief Initialize all loaded plugins
     * @param resolveDependencies Whether to resolve dependencies before initialization
     * @return True if all plugins were initialized successfully, false otherwise
     */
    bool initializeAll(bool resolveDependencies = true);
    
    /**
     * @brief Initialize a specific plugin
     * @param pluginId ID of the plugin to initialize
     * @param resolveDependencies Whether to resolve dependencies before initialization
     * @return True if the plugin was initialized successfully, false otherwise
     */
    bool initializePlugin(const std::string& pluginId, bool resolveDependencies = true);
    
    /**
     * @brief Shutdown all loaded plugins
     */
    void shutdownAll();
    
    /**
     * @brief Shutdown a specific plugin
     * @param pluginId ID of the plugin to shutdown
     * @return True if the plugin was shutdown successfully, false otherwise
     */
    bool shutdownPlugin(const std::string& pluginId);
    
    /**
     * @brief Validate a plugin
     * @param plugin Plugin to validate
     * @return True if the plugin is valid, false otherwise
     */
    bool validatePlugin(const std::shared_ptr<IPlugin>& plugin);
    
    /**
     * @brief Resolve plugin dependencies
     * @param plugin Plugin to resolve dependencies for
     * @return True if all dependencies were resolved, false otherwise
     */
    bool resolvePluginDependencies(const std::shared_ptr<IPlugin>& plugin);
    
    /**
     * @brief Get the initialization order for plugins
     * @return Vector of plugin IDs in the order they should be initialized
     */
    std::vector<std::string> getInitializationOrder();
    
    /**
     * @brief Register a callback to be called when a plugin is loaded
     * @param callback Function to call when a plugin is loaded
     * @return ID that can be used to unregister the callback
     */
    int registerPluginLoadedCallback(std::function<void(const std::string&)> callback);
    
    /**
     * @brief Unregister a plugin loaded callback
     * @param callbackId ID returned by registerPluginLoadedCallback
     * @return True if the callback was unregistered, false if the ID was invalid
     */
    bool unregisterPluginLoadedCallback(int callbackId);

private:
    PluginLoader();
    ~PluginLoader();
    
    /**
     * @brief Detect cycles in the plugin dependency graph
     * @param pluginId ID of the plugin to check
     * @param visited Set of visited plugin IDs
     * @param recursionStack Set of plugin IDs in the current recursion stack
     * @return True if a cycle was detected, false otherwise
     */
    bool detectCycle(const std::string& pluginId, std::set<std::string>& visited, std::set<std::string>& recursionStack);
    
    /**
     * @brief Topologically sort plugins based on dependencies
     * @param pluginId ID of the plugin to sort
     * @param visited Set of visited plugin IDs
     * @param sortedPlugins Vector to store the sorted plugin IDs
     */
    void topologicalSort(const std::string& pluginId, std::set<std::string>& visited, std::vector<std::string>& sortedPlugins);
    
    static std::shared_ptr<PluginLoader> s_instance;
    static std::mutex s_instanceMutex;
    
    std::map<std::string, std::shared_ptr<IPlugin>> m_plugins;
    std::map<std::string, void*> m_pluginHandles;
    std::map<int, std::function<void(const std::string&)>> m_pluginLoadedCallbacks;
    int m_nextCallbackId;
    mutable std::mutex m_mutex;
};

} // namespace Plugins
} // namespace Core
} // namespace VivoX
