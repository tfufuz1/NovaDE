#pragma once

#include <string>
#include <memory>
#include <map>
#include <mutex>
#include <functional>
#include <any>
#include <vector>
#include <set>
#include <stdexcept>

namespace VivoX {
namespace Core {
namespace Services {

/**
 * @brief Exception thrown when a service dependency cannot be resolved
 */
class ServiceDependencyException : public std::runtime_error {
public:
    explicit ServiceDependencyException(const std::string& message) 
        : std::runtime_error(message) {}
};

/**
 * @brief Interface for all services
 */
class IService {
public:
    virtual ~IService() = default;
    
    /**
     * @brief Get the name of the service
     * @return Service name
     */
    virtual std::string getName() const = 0;
    
    /**
     * @brief Get the dependencies of the service
     * @return Vector of service names that this service depends on
     */
    virtual std::vector<std::string> getDependencies() const = 0;
    
    /**
     * @brief Initialize the service
     * @return True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;
    
    /**
     * @brief Shutdown the service
     */
    virtual void shutdown() = 0;
    
    /**
     * @brief Check if the service is initialized
     * @return True if the service is initialized, false otherwise
     */
    virtual bool isInitialized() const = 0;
};

/**
 * @brief Service registry for the VivoX Desktop Environment
 * 
 * This class provides a centralized way to register and access services
 * for all components of the VivoX Desktop Environment.
 * Features:
 * - Dependency injection for services
 * - Automatic dependency resolution
 * - Service lifecycle management
 * - Type-safe service access
 */
class ServiceRegistry {
public:
    /**
     * @brief Get the singleton instance of the ServiceRegistry
     * @return Shared pointer to the ServiceRegistry instance
     */
    static std::shared_ptr<ServiceRegistry> getInstance();
    
    /**
     * @brief Register a service
     * @param service Service to register
     * @return True if the service was registered, false if a service with the same name already exists
     */
    bool registerService(std::shared_ptr<IService> service);
    
    /**
     * @brief Unregister a service
     * @param serviceName Name of the service to unregister
     * @return True if the service was unregistered, false if it wasn't registered
     */
    bool unregisterService(const std::string& serviceName);
    
    /**
     * @brief Get a service by name
     * @param serviceName Name of the service to get
     * @return Shared pointer to the service, or nullptr if it wasn't found
     */
    std::shared_ptr<IService> getService(const std::string& serviceName) const;
    
    /**
     * @brief Get a service by name and cast it to a specific type
     * @param serviceName Name of the service to get
     * @return Shared pointer to the service cast to the specified type, or nullptr if it wasn't found or couldn't be cast
     */
    template<typename T>
    std::shared_ptr<T> getService(const std::string& serviceName) const {
        auto service = getService(serviceName);
        if (!service) {
            return nullptr;
        }
        
        return std::dynamic_pointer_cast<T>(service);
    }
    
    /**
     * @brief Get a service by type
     * @return Shared pointer to the first service of the specified type, or nullptr if none was found
     */
    template<typename T>
    std::shared_ptr<T> getServiceByType() const {
        std::lock_guard<std::mutex> lock(m_mutex);
        
        for (const auto& [name, service] : m_services) {
            auto typedService = std::dynamic_pointer_cast<T>(service);
            if (typedService) {
                return typedService;
            }
        }
        
        return nullptr;
    }
    
    /**
     * @brief Check if a service exists
     * @param serviceName Name of the service to check
     * @return True if the service exists, false otherwise
     */
    bool hasService(const std::string& serviceName) const;
    
    /**
     * @brief Initialize all registered services
     * @param resolveDependencies Whether to resolve dependencies before initialization
     * @return True if all services were initialized successfully, false otherwise
     */
    bool initializeAll(bool resolveDependencies = true);
    
    /**
     * @brief Initialize a specific service
     * @param serviceName Name of the service to initialize
     * @param resolveDependencies Whether to resolve dependencies before initialization
     * @return True if the service was initialized successfully, false otherwise
     */
    bool initializeService(const std::string& serviceName, bool resolveDependencies = true);
    
    /**
     * @brief Shutdown all registered services
     */
    void shutdownAll();
    
    /**
     * @brief Shutdown a specific service
     * @param serviceName Name of the service to shutdown
     * @return True if the service was shutdown successfully, false otherwise
     */
    bool shutdownService(const std::string& serviceName);
    
    /**
     * @brief Get all registered services
     * @return Map of service names to service instances
     */
    const std::map<std::string, std::shared_ptr<IService>>& getServices() const;
    
    /**
     * @brief Get the initialization order for services
     * @return Vector of service names in the order they should be initialized
     */
    std::vector<std::string> getInitializationOrder();

private:
    ServiceRegistry();
    ~ServiceRegistry();
    
    /**
     * @brief Detect cycles in the service dependency graph
     * @param serviceName Name of the service to check
     * @param visited Set of visited service names
     * @param recursionStack Set of service names in the current recursion stack
     * @return True if a cycle was detected, false otherwise
     */
    bool detectCycle(const std::string& serviceName, std::set<std::string>& visited, std::set<std::string>& recursionStack);
    
    /**
     * @brief Topologically sort services based on dependencies
     * @param serviceName Name of the service to sort
     * @param visited Set of visited service names
     * @param sortedServices Vector to store the sorted service names
     */
    void topologicalSort(const std::string& serviceName, std::set<std::string>& visited, std::vector<std::string>& sortedServices);
    
    static std::shared_ptr<ServiceRegistry> s_instance;
    static std::mutex s_instanceMutex;
    
    std::map<std::string, std::shared_ptr<IService>> m_services;
    mutable std::mutex m_mutex;
};

} // namespace Services
} // namespace Core
} // namespace VivoX
