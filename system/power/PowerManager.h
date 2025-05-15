#pragma once

#include <QObject>
#include <QString>
#include <QVariantMap>

namespace VivoX::System {

/**
 * @brief The PowerManager class manages system power states and settings.
 * 
 * It provides an interface to control power-related functionality such as
 * suspend, hibernate, shutdown, and power settings.
 */
class PowerManager : public QObject {
    Q_OBJECT

public:
    /**
     * @brief Enum defining the possible power states
     */
    enum PowerState {
        OnAC,           ///< Running on AC power
        OnBattery,      ///< Running on battery power
        LowBattery,     ///< Running on low battery
        CriticalBattery ///< Running on critically low battery
    };

    /**
     * @brief Enum defining the possible power actions
     */
    enum PowerAction {
        Suspend,        ///< Suspend to RAM
        Hibernate,      ///< Suspend to disk
        Shutdown,       ///< Shut down the system
        Reboot,         ///< Reboot the system
        LogOut          ///< Log out the current user
    };

    explicit PowerManager(QObject *parent = nullptr);
    ~PowerManager();

    /**
     * @brief Initialize the power manager
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Get the current power state
     * @return The current power state
     */
    PowerState getPowerState() const;

    /**
     * @brief Get the current battery level
     * @return The battery level as a percentage (0-100), or -1 if no battery
     */
    int getBatteryLevel() const;

    /**
     * @brief Get the estimated battery time remaining
     * @return The time remaining in seconds, or -1 if unknown
     */
    int getBatteryTimeRemaining() const;

    /**
     * @brief Check if the system can perform a power action
     * @param action The power action to check
     * @return True if the action can be performed
     */
    bool canPerformAction(PowerAction action) const;

    /**
     * @brief Perform a power action
     * @param action The power action to perform
     * @return True if the action was initiated successfully
     */
    bool performAction(PowerAction action);

    /**
     * @brief Get the current power settings
     * @return Map of power settings
     */
    QVariantMap getPowerSettings() const;

    /**
     * @brief Set a power setting
     * @param key The setting key
     * @param value The setting value
     * @return True if successful
     */
    bool setPowerSetting(const QString &key, const QVariant &value);

signals:
    /**
     * @brief Signal emitted when the power state changes
     * @param state The new power state
     */
    void powerStateChanged(PowerState state);

    /**
     * @brief Signal emitted when the battery level changes
     * @param level The new battery level
     */
    void batteryLevelChanged(int level);

    /**
     * @brief Signal emitted when a power setting changes
     * @param key The setting key
     * @param value The new setting value
     */
    void powerSettingChanged(const QString &key, const QVariant &value);

private:
    // Current power state
    PowerState m_powerState;
    
    // Current battery level
    int m_batteryLevel;
    
    // Current battery time remaining
    int m_batteryTimeRemaining;
    
    // Power settings
    QVariantMap m_powerSettings;
    
    // Update power information
    void updatePowerInfo();
};

} // namespace VivoX::System
