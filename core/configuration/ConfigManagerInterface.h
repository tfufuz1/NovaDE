#pragma once

#include <QObject>
#include <QString>
#include <QVariantMap>

namespace VivoX {
namespace Core {

/**
 * @brief Interface for the configuration manager
 * 
 * This interface defines the functionality that the configuration manager must implement.
 * It is responsible for loading, saving, and managing configuration settings.
 */
class ConfigManagerInterface {
public:
    virtual ~ConfigManagerInterface() = default;

    /**
     * @brief Initialize the configuration manager
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Load configuration from a file
     * 
     * @param filePath The path to the configuration file
     * @return bool True if loading was successful, false otherwise
     */
    virtual bool loadConfig(const QString& filePath) = 0;

    /**
     * @brief Save configuration to a file
     * 
     * @param filePath The path to the configuration file
     * @return bool True if saving was successful, false otherwise
     */
    virtual bool saveConfig(const QString& filePath) = 0;

    /**
     * @brief Get a configuration value
     * 
     * @param key The configuration key
     * @param defaultValue The default value to return if the key is not found
     * @return QVariant The configuration value
     */
    virtual QVariant getValue(const QString& key, const QVariant& defaultValue = QVariant()) const = 0;

    /**
     * @brief Set a configuration value
     * 
     * @param key The configuration key
     * @param value The configuration value
     */
    virtual void setValue(const QString& key, const QVariant& value) = 0;

    /**
     * @brief Check if a configuration key exists
     * 
     * @param key The configuration key
     * @return bool True if the key exists, false otherwise
     */
    virtual bool hasKey(const QString& key) const = 0;

    /**
     * @brief Remove a configuration key
     * 
     * @param key The configuration key
     * @return bool True if the key was removed, false otherwise
     */
    virtual bool removeKey(const QString& key) = 0;

    /**
     * @brief Get all configuration keys
     * 
     * @return QStringList The list of configuration keys
     */
    virtual QStringList keys() const = 0;

    /**
     * @brief Get all configuration values
     * 
     * @return QVariantMap The map of configuration keys and values
     */
    virtual QVariantMap allValues() const = 0;

    /**
     * @brief Clear all configuration values
     */
    virtual void clear() = 0;

    /**
     * @brief Watch a configuration file for changes
     * 
     * @param filePath The path to the configuration file
     * @return bool True if watching was successful, false otherwise
     */
    virtual bool watchConfigFile(const QString& filePath) = 0;

    /**
     * @brief Stop watching a configuration file
     * 
     * @param filePath The path to the configuration file
     * @return bool True if stopping was successful, false otherwise
     */
    virtual bool unwatchConfigFile(const QString& filePath) = 0;
};

} // namespace Core
} // namespace VivoX
