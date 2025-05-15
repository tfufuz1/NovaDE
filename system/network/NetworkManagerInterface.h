#pragma once

#include <QObject>
#include <QString>
#include <QList>
#include <QNetworkInterface>

namespace VivoX {
namespace System {

/**
 * @brief Interface for the network manager
 * 
 * This interface defines the functionality that the network manager must implement.
 * It is responsible for managing network connections and providing network information.
 */
class NetworkManagerInterface {
public:
    virtual ~NetworkManagerInterface() = default;

    /**
     * @brief Initialize the network manager
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Get all network interfaces
     * 
     * @return QList<QNetworkInterface> The list of network interfaces
     */
    virtual QList<QNetworkInterface> networkInterfaces() const = 0;

    /**
     * @brief Get the active network interface
     * 
     * @return QNetworkInterface The active network interface
     */
    virtual QNetworkInterface activeInterface() const = 0;

    /**
     * @brief Check if the system is connected to a network
     * 
     * @return bool True if connected, false otherwise
     */
    virtual bool isConnected() const = 0;

    /**
     * @brief Check if the system is connected to the internet
     * 
     * @return bool True if connected to the internet, false otherwise
     */
    virtual bool hasInternetConnection() const = 0;

    /**
     * @brief Get the current connection type
     * 
     * @return QString The connection type (e.g., "wifi", "ethernet", "vpn")
     */
    virtual QString connectionType() const = 0;

    /**
     * @brief Get the current connection strength (for wireless connections)
     * 
     * @return int The connection strength as a percentage (0-100)
     */
    virtual int connectionStrength() const = 0;

    /**
     * @brief Get the current connection speed
     * 
     * @return qint64 The connection speed in bits per second
     */
    virtual qint64 connectionSpeed() const = 0;

    /**
     * @brief Get the current IP address
     * 
     * @return QString The IP address
     */
    virtual QString ipAddress() const = 0;

    /**
     * @brief Get the current MAC address
     * 
     * @return QString The MAC address
     */
    virtual QString macAddress() const = 0;

    /**
     * @brief Get the current gateway
     * 
     * @return QString The gateway address
     */
    virtual QString gateway() const = 0;

    /**
     * @brief Get the current DNS servers
     * 
     * @return QStringList The list of DNS server addresses
     */
    virtual QStringList dnsServers() const = 0;

    /**
     * @brief Connect to a wireless network
     * 
     * @param ssid The network SSID
     * @param password The network password
     * @return bool True if connection was successful, false otherwise
     */
    virtual bool connectToWirelessNetwork(const QString& ssid, const QString& password) = 0;

    /**
     * @brief Disconnect from the current network
     * 
     * @return bool True if disconnection was successful, false otherwise
     */
    virtual bool disconnect() = 0;

    /**
     * @brief Enable or disable networking
     * 
     * @param enable Whether to enable networking
     * @return bool True if the operation was successful, false otherwise
     */
    virtual bool setNetworkingEnabled(bool enable) = 0;

    /**
     * @brief Check if networking is enabled
     * 
     * @return bool True if networking is enabled, false otherwise
     */
    virtual bool isNetworkingEnabled() const = 0;

    /**
     * @brief Enable or disable wireless networking
     * 
     * @param enable Whether to enable wireless networking
     * @return bool True if the operation was successful, false otherwise
     */
    virtual bool setWirelessEnabled(bool enable) = 0;

    /**
     * @brief Check if wireless networking is enabled
     * 
     * @return bool True if wireless networking is enabled, false otherwise
     */
    virtual bool isWirelessEnabled() const = 0;

    /**
     * @brief Scan for wireless networks
     * 
     * @return QStringList The list of available wireless networks
     */
    virtual QStringList scanWirelessNetworks() = 0;
};

} // namespace System
} // namespace VivoX
