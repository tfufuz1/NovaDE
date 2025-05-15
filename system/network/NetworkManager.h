#pragma once

#include <QObject>
#include <QString>
#include <QStringList>
#include <QVariantMap>

namespace VivoX::System {

/**
 * @brief The NetworkInfo class contains information about a network connection.
 */
class NetworkInfo {
public:
    QString id;           ///< Unique identifier for the network
    QString name;         ///< Display name of the network
    QString type;         ///< Type of network (wifi, ethernet, vpn, etc.)
    QString state;        ///< Current state (connected, disconnected, connecting)
    bool isDefault;       ///< Whether this is the default connection
    QVariantMap details;  ///< Additional details about the connection
};

/**
 * @brief The NetworkManager class manages network connections.
 * 
 * It provides an interface to NetworkManager via D-Bus for monitoring and
 * controlling network connections.
 */
class NetworkManager : public QObject {
    Q_OBJECT

public:
    explicit NetworkManager(QObject *parent = nullptr);
    ~NetworkManager();

    /**
     * @brief Initialize the network manager
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Get a list of all network connections
     * @return List of network info objects
     */
    QList<NetworkInfo> getAllNetworks() const;

    /**
     * @brief Get information about a specific network
     * @param id The network ID
     * @return The network info, or an empty object if not found
     */
    NetworkInfo getNetworkInfo(const QString &id) const;

    /**
     * @brief Get the currently active network connections
     * @return List of active network info objects
     */
    QList<NetworkInfo> getActiveNetworks() const;

    /**
     * @brief Get available wireless networks
     * @return List of available wireless network info objects
     */
    QList<NetworkInfo> getAvailableWirelessNetworks() const;

    /**
     * @brief Connect to a network
     * @param id The network ID
     * @param password The password (if required)
     * @return True if connection was initiated successfully
     */
    bool connectToNetwork(const QString &id, const QString &password = QString());

    /**
     * @brief Disconnect from a network
     * @param id The network ID
     * @return True if disconnection was initiated successfully
     */
    bool disconnectFromNetwork(const QString &id);

    /**
     * @brief Enable or disable networking
     * @param enable Whether to enable or disable networking
     * @return True if successful
     */
    bool setNetworkingEnabled(bool enable);

    /**
     * @brief Enable or disable wireless networking
     * @param enable Whether to enable or disable wireless networking
     * @return True if successful
     */
    bool setWirelessEnabled(bool enable);

    /**
     * @brief Check if networking is enabled
     * @return True if networking is enabled
     */
    bool isNetworkingEnabled() const;

    /**
     * @brief Check if wireless networking is enabled
     * @return True if wireless networking is enabled
     */
    bool isWirelessEnabled() const;

signals:
    /**
     * @brief Signal emitted when a network connection state changes
     * @param info The updated network info
     */
    void networkStateChanged(const NetworkInfo &info);

    /**
     * @brief Signal emitted when a new network connection is added
     * @param info The new network info
     */
    void networkAdded(const NetworkInfo &info);

    /**
     * @brief Signal emitted when a network connection is removed
     * @param id The network ID
     */
    void networkRemoved(const QString &id);

    /**
     * @brief Signal emitted when the list of available wireless networks changes
     */
    void availableWirelessNetworksChanged();

    /**
     * @brief Signal emitted when the networking enabled state changes
     * @param enabled The new enabled state
     */
    void networkingEnabledChanged(bool enabled);

    /**
     * @brief Signal emitted when the wireless enabled state changes
     * @param enabled The new enabled state
     */
    void wirelessEnabledChanged(bool enabled);

private:
    // Map of network ID to network info
    QHash<QString, NetworkInfo> m_networks;
    
    // List of available wireless networks
    QList<NetworkInfo> m_availableWirelessNetworks;
    
    // Networking enabled state
    bool m_networkingEnabled;
    
    // Wireless enabled state
    bool m_wirelessEnabled;
    
    // Update network information from D-Bus
    void updateNetworkInfo();
    
    // Update available wireless networks from D-Bus
    void updateAvailableWirelessNetworks();
};

} // namespace VivoX::System
