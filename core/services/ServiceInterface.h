#pragma once

#include <QString>
#include <QObject>

namespace VivoX {
namespace Core {

/**
 * @brief Interface for all services in the VivoX system
 * 
 * This interface defines the common functionality that all services must implement.
 * Services are registered with the ServiceRegistry and can be accessed by other components.
 */
class ServiceInterface {
public:
    virtual ~ServiceInterface() = default;

    /**
     * @brief Get the unique identifier for this service
     * 
     * @return QString The service identifier
     */
    virtual QString serviceId() const = 0;

    /**
     * @brief Initialize the service
     * 
     * This method is called during system startup to initialize the service.
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Shutdown the service
     * 
     * This method is called during system shutdown to clean up resources.
     */
    virtual void shutdown() = 0;

    /**
     * @brief Check if the service is initialized
     * 
     * @return bool True if the service is initialized, false otherwise
     */
    virtual bool isInitialized() const = 0;
};

} // namespace Core
} // namespace VivoX
