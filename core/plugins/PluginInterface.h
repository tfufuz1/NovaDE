#pragma once

#include <QObject>
#include <QString>
#include <QVariant>

namespace VivoX {
namespace Core {

/**
 * @brief Interface for all plugins in the VivoX system
 * 
 * This interface defines the common functionality that all plugins must implement.
 * Plugins are loaded by the PluginLoader and can extend the functionality of the system.
 */
class PluginInterface {
public:
    virtual ~PluginInterface() = default;

    /**
     * @brief Get the unique identifier for this plugin
     * 
     * @return QString The plugin identifier
     */
    virtual QString pluginId() const = 0;

    /**
     * @brief Get the version of this plugin
     * 
     * @return QString The plugin version
     */
    virtual QString version() const = 0;

    /**
     * @brief Get the name of this plugin
     * 
     * @return QString The plugin name
     */
    virtual QString name() const = 0;

    /**
     * @brief Get the description of this plugin
     * 
     * @return QString The plugin description
     */
    virtual QString description() const = 0;

    /**
     * @brief Get the author of this plugin
     * 
     * @return QString The plugin author
     */
    virtual QString author() const = 0;

    /**
     * @brief Get the dependencies of this plugin
     * 
     * @return QStringList The list of plugin IDs that this plugin depends on
     */
    virtual QStringList dependencies() const = 0;

    /**
     * @brief Initialize the plugin
     * 
     * This method is called during system startup to initialize the plugin.
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Shutdown the plugin
     * 
     * This method is called during system shutdown to clean up resources.
     */
    virtual void shutdown() = 0;

    /**
     * @brief Get a configuration value
     * 
     * @param key The configuration key
     * @param defaultValue The default value to return if the key is not found
     * @return QVariant The configuration value
     */
    virtual QVariant getConfigValue(const QString& key, const QVariant& defaultValue = QVariant()) const = 0;

    /**
     * @brief Set a configuration value
     * 
     * @param key The configuration key
     * @param value The configuration value
     */
    virtual void setConfigValue(const QString& key, const QVariant& value) = 0;
};

} // namespace Core
} // namespace VivoX
