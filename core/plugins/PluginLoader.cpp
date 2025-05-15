#include "PluginLoader.h"
#include <iostream>
#include <dlfcn.h>
#include <filesystem>
#include <algorithm>
#include <queue>

namespace VivoX {
namespace Core {
namespace Plugins {

std::shared_ptr<PluginLoader> PluginLoader::s_instance = nullptr;
std::mutex PluginLoader::s_instanceMutex;

std::shared_ptr<PluginLoader> PluginLoader::getInstance() {
    std::lock_guard<std::mutex> lock(s_instanceMutex);
    
    if (!s_instance) {
        s_instance = std::shared_ptr<PluginLoader>(new PluginLoader());
    }
    
    return s_instance;
}

PluginLoader::PluginLoader() : m_nextCallbackId(1) {
    // Initialize empty plugin loader
}

PluginLoader::~PluginLoader() {
    // Shutdown and unload all plugins
    shutdownAll();
    
    std::lock_guard<std::mutex> lock(m_mutex);
    
    // Close all plugin handles
    for (auto& [path, handle] : m_pluginHandles) {
        if (handle) {
            dlclose(handle);
        }
    }
    
    m_pluginHandles.clear();
    m_plugins.clear();
    m_pluginLoadedCallbacks.clear();
}

std::shared_ptr<IPlugin> PluginLoader::loadPlugin(const std::string& pluginPath) {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    // Check if file exists
    if (!std::filesystem::exists(pluginPath)) {
        std::cerr << "Plugin file does not exist: " << pluginPath << std::endl;
        return nullptr;
    }
    
    // Open the shared library
    void* handle = dlopen(pluginPath.c_str(), RTLD_LAZY);
    if (!handle) {
        std::cerr << "Failed to open plugin: " << dlerror() << std::endl;
        return nullptr;
    }
    
    // Reset errors
    dlerror();
    
    // Load the create function
    using CreatePluginFunc = IPlugin* (*)();
    CreatePluginFunc createPlugin = reinterpret_cast<CreatePluginFunc>(dlsym(handle, "createPlugin"));
    
    // Check for errors
    const char* dlsym_error = dlerror();
    if (dlsym_error) {
        std::cerr << "Failed to load createPlugin symbol: " << dlsym_error << std::endl;
        dlclose(handle);
        return nullptr;
    }
    
    // Create the plugin
    IPlugin* plugin = createPlugin();
    if (!plugin) {
        std::cerr << "Failed to create plugin instance" << std::endl;
        dlclose(handle);
        return nullptr;
    }
    
    // Wrap in shared_ptr with custom deleter
    std::shared_ptr<IPlugin> pluginPtr(plugin, [](IPlugin* p) {
        delete p;
    });
    
    // Validate the plugin
    if (!validatePlugin(pluginPtr)) {
        std::cerr << "Plugin validation failed" << std::endl;
        dlclose(handle);
        return nullptr;
    }
    
    // Store the plugin and handle
    const std::string& pluginId = pluginPtr->getPluginId();
    m_plugins[pluginId] = pluginPtr;
    m_pluginHandles[pluginId] = handle;
    
    // Notify callbacks
    for (const auto& [id, callback] : m_pluginLoadedCallbacks) {
        try {
            callback(pluginId);
        } catch (const std::exception& e) {
            std::cerr << "Exception in plugin loaded callback: " << e.what() << std::endl;
        }
    }
    
    return pluginPtr;
}

int PluginLoader::loadPluginsFromDirectory(const std::string& pluginDir, bool recursive) {
    // Check if directory exists
    if (!std::filesystem::exists(pluginDir) || !std::filesystem::is_directory(pluginDir)) {
        std::cerr << "Plugin directory does not exist: " << pluginDir << std::endl;
        return 0;
    }
    
    int loadedCount = 0;
    
    // Determine the iterator type based on recursive flag
    auto dirIterator = recursive 
        ? std::filesystem::recursive_directory_iterator(pluginDir)
        : std::filesystem::directory_iterator(pluginDir);
    
    // Iterate over all files in the directory
    for (const auto& entry : dirIterator) {
        if (entry.is_regular_file()) {
            const std::string& path = entry.path().string();
            
            // Check if file is a shared library
            if (path.ends_with(".so") || path.ends_with(".dll") || path.ends_with(".dylib")) {
                // Load the plugin
                std::shared_ptr<IPlugin> plugin = loadPlugin(path);
                if (plugin) {
                    loadedCount++;
                }
            }
        }
    }
    
    // Get initialization order
    auto initOrder = getInitializationOrder();
    
    // Initialize plugins in the correct order
    for (const auto& pluginId : initOrder) {
        if (!initializePlugin(pluginId, false)) { // Don't resolve dependencies again
            std::cerr << "Failed to initialize plugin: " << pluginId << std::endl;
        }
    }
    
    return loadedCount;
}

bool PluginLoader::unloadPlugin(const std::string& pluginId) {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    auto pluginIt = m_plugins.find(pluginId);
    if (pluginIt == m_plugins.end()) {
        return false;
    }
    
    // Shutdown the plugin if it's initialized
    if (pluginIt->second->isInitialized()) {
        pluginIt->second->shutdown();
    }
    
    // Get the handle
    auto handleIt = m_pluginHandles.find(pluginId);
    if (handleIt != m_pluginHandles.end()) {
        // Close the handle
        if (handleIt->second) {
            dlclose(handleIt->second);
        }
        
        // Remove the handle
        m_pluginHandles.erase(handleIt);
    }
    
    // Remove the plugin
    m_plugins.erase(pluginIt);
    
    return true;
}

std::shared_ptr<IPlugin> PluginLoader::getPlugin(const std::string& pluginId) const {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    auto it = m_plugins.find(pluginId);
    if (it == m_plugins.end()) {
        return nullptr;
    }
    
    return it->second;
}

bool PluginLoader::isPluginLoaded(const std::string& pluginId) const {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    return m_plugins.find(pluginId) != m_plugins.end();
}

const std::map<std::string, std::shared_ptr<IPlugin>>& PluginLoader::getLoadedPlugins() const {
    return m_plugins;
}

bool PluginLoader::initializeAll(bool resolveDependencies) {
    if (resolveDependencies) {
        // Get initialization order
        auto initOrder = getInitializationOrder();
        
        // Initialize plugins in the correct order
        for (const auto& pluginId : initOrder) {
            if (!initializePlugin(pluginId, false)) { // Don't resolve dependencies again
                return false;
            }
        }
        
        return true;
    } else {
        std::lock_guard<std::mutex> lock(m_mutex);
        
        bool allInitialized = true;
        
        // Initialize all plugins
        for (auto& [id, plugin] : m_plugins) {
            if (!plugin->isInitialized()) {
                if (!plugin->initialize()) {
                    std::cerr << "Failed to initialize plugin: " << id << std::endl;
                    allInitialized = false;
                }
            }
        }
        
        return allInitialized;
    }
}

bool PluginLoader::initializePlugin(const std::string& pluginId, bool resolveDependencies) {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    auto it = m_plugins.find(pluginId);
    if (it == m_plugins.end()) {
        std::cerr << "Plugin not found: " << pluginId << std::endl;
        return false;
    }
    
    auto plugin = it->second;
    
    // Check if already initialized
    if (plugin->isInitialized()) {
        return true;
    }
    
    // Resolve dependencies if requested
    if (resolveDependencies && !resolvePluginDependencies(plugin)) {
        std::cerr << "Failed to resolve dependencies for plugin: " << pluginId << std::endl;
        return false;
    }
    
    // Initialize dependencies first
    for (const auto& dependencyId : plugin->getDependencies()) {
        if (!initializePlugin(dependencyId, false)) { // Don't resolve dependencies again
            std::cerr << "Failed to initialize dependency " << dependencyId << " for plugin " << pluginId << std::endl;
            return false;
        }
    }
    
    // Initialize the plugin
    return plugin->initialize();
}

void PluginLoader::shutdownAll() {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    // Get initialization order and reverse it for shutdown
    auto initOrder = getInitializationOrder();
    std::reverse(initOrder.begin(), initOrder.end());
    
    // Shutdown plugins in the reverse order of initialization
    for (const auto& pluginId : initOrder) {
        shutdownPlugin(pluginId);
    }
}

bool PluginLoader::shutdownPlugin(const std::string& pluginId) {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    auto it = m_plugins.find(pluginId);
    if (it == m_plugins.end()) {
        return false;
    }
    
    auto plugin = it->second;
    
    // Check if initialized
    if (!plugin->isInitialized()) {
        return true;
    }
    
    // Shutdown the plugin
    plugin->shutdown();
    return true;
}

bool PluginLoader::validatePlugin(const std::shared_ptr<IPlugin>& plugin) {
    if (!plugin) {
        return false;
    }
    
    // Check if plugin ID is valid
    const std::string& pluginId = plugin->getPluginId();
    if (pluginId.empty()) {
        std::cerr << "Plugin ID is empty" << std::endl;
        return false;
    }
    
    // Check if plugin with the same ID is already loaded
    if (m_plugins.find(pluginId) != m_plugins.end()) {
        std::cerr << "Plugin with ID " << pluginId << " is already loaded" << std::endl;
        return false;
    }
    
    // Check if plugin name is valid
    if (plugin->getName().empty()) {
        std::cerr << "Plugin name is empty for plugin ID: " << pluginId << std::endl;
        return false;
    }
    
    // Check if plugin version is valid
    if (plugin->getVersion().empty()) {
        std::cerr << "Plugin version is empty for plugin ID: " << pluginId << std::endl;
        return false;
    }
    
    return true;
}

bool PluginLoader::resolvePluginDependencies(const std::shared_ptr<IPlugin>& plugin) {
    if (!plugin) {
        return false;
    }
    
    const std::string& pluginId = plugin->getPluginId();
    const std::vector<std::string>& dependencies = plugin->getDependencies();
    
    // Check for self-dependency
    if (std::find(dependencies.begin(), dependencies.end(), pluginId) != dependencies.end()) {
        std::cerr << "Plugin " << pluginId << " depends on itself" << std::endl;
        return false;
    }
    
    // Check for dependency cycles
    std::set<std::string> visited;
    std::set<std::string> recursionStack;
    if (detectCycle(pluginId, visited, recursionStack)) {
        std::cerr << "Cyclic dependency detected for plugin " << pluginId << std::endl;
        return false;
    }
    
    // Check if all dependencies are loaded
    for (const std::string& dependencyId : dependencies) {
        if (!isPluginLoaded(dependencyId)) {
            std::cerr << "Plugin " << pluginId << " depends on plugin " << dependencyId << " which is not loaded" << std::endl;
            return false;
        }
    }
    
    return true;
}

std::vector<std::string> PluginLoader::getInitializationOrder() {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    std::vector<std::string> sortedPlugins;
    std::set<std::string> visited;
    
    // Perform topological sort for each plugin
    for (const auto& [id, _] : m_plugins) {
        if (visited.find(id) == visited.end()) {
            topologicalSort(id, visited, sortedPlugins);
        }
    }
    
    return sortedPlugins;
}

int PluginLoader::registerPluginLoadedCallback(std::function<void(const std::string&)> callback) {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    int callbackId = m_nextCallbackId++;
    m_pluginLoadedCallbacks[callbackId] = callback;
    
    return callbackId;
}

bool PluginLoader::unregisterPluginLoadedCallback(int callbackId) {
    std::lock_guard<std::mutex> lock(m_mutex);
    
    auto it = m_pluginLoadedCallbacks.find(callbackId);
    if (it == m_pluginLoadedCallbacks.end()) {
        return false;
    }
    
    m_pluginLoadedCallbacks.erase(it);
    return true;
}

bool PluginLoader::detectCycle(const std::string& pluginId, std::set<std::string>& visited, std::set<std::string>& recursionStack) {
    // Mark the current node as visited and part of recursion stack
    visited.insert(pluginId);
    recursionStack.insert(pluginId);
    
    // Get the plugin
    auto it = m_plugins.find(pluginId);
    if (it == m_plugins.end()) {
        // Plugin not found, no cycle
        recursionStack.erase(pluginId);
        return false;
    }
    
    // Recur for all dependencies
    for (const auto& dependencyId : it->second->getDependencies()) {
        // If dependency is not visited, check for cycle
        if (visited.find(dependencyId) == visited.end()) {
            if (detectCycle(dependencyId, visited, recursionStack)) {
                return true;
            }
        }
        // If dependency is in recursion stack, there is a cycle
        else if (recursionStack.find(dependencyId) != recursionStack.end()) {
            return true;
        }
    }
    
    // Remove the current node from recursion stack
    recursionStack.erase(pluginId);
    return false;
}

void PluginLoader::topologicalSort(const std::string& pluginId, std::set<std::string>& visited, std::vector<std::string>& sortedPlugins) {
    // Mark the current node as visited
    visited.insert(pluginId);
    
    // Get the plugin
    auto it = m_plugins.find(pluginId);
    if (it == m_plugins.end()) {
        return;
    }
    
    // Recur for all dependencies
    for (const auto& dependencyId : it->second->getDependencies()) {
        if (visited.find(dependencyId) == visited.end()) {
            topologicalSort(dependencyId, visited, sortedPlugins);
        }
    }
    
    // Add the current node to the sorted list
    sortedPlugins.push_back(pluginId);
}

} // namespace Plugins
} // namespace Core
} // namespace VivoX
