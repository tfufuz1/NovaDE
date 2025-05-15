#pragma once

#include <QObject>
#include <QString>
#include <QList>
#include <QVariant>

namespace VivoX {
namespace System {

/**
 * @brief Interface for system services
 * 
 * This interface defines the functionality that system services must implement.
 * System services provide access to system resources and functionality.
 */
class SystemServiceInterface {
public:
    virtual ~SystemServiceInterface() = default;

    /**
     * @brief Initialize the system service
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Get the service ID
     * 
     * @return QString The service ID
     */
    virtual QString serviceId() const = 0;

    /**
     * @brief Get the service name
     * 
     * @return QString The service name
     */
    virtual QString serviceName() const = 0;

    /**
     * @brief Get the service description
     * 
     * @return QString The service description
     */
    virtual QString serviceDescription() const = 0;

    /**
     * @brief Check if the service is available
     * 
     * @return bool True if the service is available, false otherwise
     */
    virtual bool isAvailable() const = 0;

    /**
     * @brief Get a property of the service
     * 
     * @param name The property name
     * @return QVariant The property value
     */
    virtual QVariant property(const QString& name) const = 0;

    /**
     * @brief Set a property of the service
     * 
     * @param name The property name
     * @param value The property value
     */
    virtual void setProperty(const QString& name, const QVariant& value) = 0;

    /**
     * @brief Get the list of available properties
     * 
     * @return QStringList The list of property names
     */
    virtual QStringList availableProperties() const = 0;

    /**
     * @brief Execute a command on the service
     * 
     * @param command The command to execute
     * @param args The command arguments
     * @return QVariant The command result
     */
    virtual QVariant executeCommand(const QString& command, const QVariantMap& args = QVariantMap()) = 0;

    /**
     * @brief Get the list of available commands
     * 
     * @return QStringList The list of command names
     */
    virtual QStringList availableCommands() const = 0;
};

} // namespace System
} // namespace VivoX
