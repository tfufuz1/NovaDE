#include "NetworkManager.h"

#include <QDebug>
#include <QDBusConnection>
#include <QDBusInterface>
#include <QDBusReply>
#include <QDBusMessage>
#include <QDBusArgument>
#include <QNetworkInterface>
#include <QProcess>

namespace VivoX::System {

NetworkManager::NetworkManager(QObject *parent)
    : QObject(parent)
    , m_networkingEnabled(false)
    , m_wirelessEnabled(false)
{
    qDebug() << "NetworkManager created";
}

NetworkManager::~NetworkManager()
{
    qDebug() << "NetworkManager destroyed";
}

bool NetworkManager::initialize()
{
    // Connect to NetworkManager via D-Bus
    QDBusConnection systemBus = QDBusConnection::systemBus();
    
    if (!systemBus.isConnected()) {
        qWarning() << "Cannot connect to system D-Bus";
        
        // Fallback: Use QNetworkInterface to get basic network information
        updateNetworkInfo();
        
        return false;
    }
    
    // Connect to NetworkManager signals
    systemBus.connect("org.freedesktop.NetworkManager", 
                     "/org/freedesktop/NetworkManager",
                     "org.freedesktop.NetworkManager",
                     "PropertiesChanged",
                     this, SLOT(updateNetworkInfo()));
    
    systemBus.connect("org.freedesktop.NetworkManager", 
                     "/org/freedesktop/NetworkManager",
                     "org.freedesktop.NetworkManager",
                     "DeviceAdded",
                     this, SLOT(updateNetworkInfo()));
    
    systemBus.connect("org.freedesktop.NetworkManager", 
                     "/org/freedesktop/NetworkManager",
                     "org.freedesktop.NetworkManager",
                     "DeviceRemoved",
                     this, SLOT(updateNetworkInfo()));
    
    // Get initial network information
    updateNetworkInfo();
    updateAvailableWirelessNetworks();
    
    qDebug() << "NetworkManager initialized";
    return true;
}

QList<NetworkInfo> NetworkManager::getAllNetworks() const
{
    return m_networks.values();
}

NetworkInfo NetworkManager::getNetworkInfo(const QString &id) const
{
    return m_networks.value(id, NetworkInfo());
}

QList<NetworkInfo> NetworkManager::getActiveNetworks() const
{
    QList<NetworkInfo> result;
    
    for (const NetworkInfo &info : m_networks.values()) {
        if (info.state == "connected") {
            result.append(info);
        }
    }
    
    return result;
}

QList<NetworkInfo> NetworkManager::getAvailableWirelessNetworks() const
{
    return m_availableWirelessNetworks;
}

bool NetworkManager::connectToNetwork(const QString &id, const QString &password)
{
    // Check if the network exists
    if (!m_networks.contains(id) && !id.startsWith("wifi-")) {
        qWarning() << "Network not found:" << id;
        return false;
    }
    
    // For a real implementation, we would use NetworkManager D-Bus API to connect
    // For now, we'll simulate the connection process
    
    // If this is a WiFi network that's not in our known networks
    if (id.startsWith("wifi-") && !m_networks.contains(id)) {
        // Create a new network info
        NetworkInfo info;
        info.id = id;
        info.name = id.mid(5); // Remove "wifi-" prefix
        info.type = "wifi";
        info.state = "connecting";
        info.isDefault = false;
        
        // Add to networks
        m_networks[id] = info;
        
        // Emit signal
        emit networkAdded(info);
    } else {
        // Update state to connecting
        NetworkInfo &info = m_networks[id];
        info.state = "connecting";
        
        // Emit signal
        emit networkStateChanged(info);
    }
    
    // Simulate connection process
    QTimer::singleShot(2000, this, [this, id]() {
        if (m_networks.contains(id)) {
            NetworkInfo &info = m_networks[id];
            info.state = "connected";
            
            // If this is the first connected network, make it default
            if (getActiveNetworks().size() == 1) {
                info.isDefault = true;
            }
            
            // Emit signal
            emit networkStateChanged(info);
            
            qDebug() << "Connected to network:" << id;
        }
    });
    
    return true;
}

bool NetworkManager::disconnectFromNetwork(const QString &id)
{
    // Check if the network exists
    if (!m_networks.contains(id)) {
        qWarning() << "Network not found:" << id;
        return false;
    }
    
    // Check if the network is connected
    NetworkInfo &info = m_networks[id];
    if (info.state != "connected") {
        qWarning() << "Network is not connected:" << id;
        return false;
    }
    
    // Update state to disconnecting
    info.state = "disconnecting";
    
    // Emit signal
    emit networkStateChanged(info);
    
    // Simulate disconnection process
    QTimer::singleShot(1000, this, [this, id]() {
        if (m_networks.contains(id)) {
            NetworkInfo &info = m_networks[id];
            info.state = "disconnected";
            
            // If this was the default network, find a new default
            if (info.isDefault) {
                info.isDefault = false;
                
                // Find another connected network to make default
                for (auto it = m_networks.begin(); it != m_networks.end(); ++it) {
                    if (it.value().state == "connected") {
                        it.value().isDefault = true;
                        emit networkStateChanged(it.value());
                        break;
                    }
                }
            }
            
            // Emit signal
            emit networkStateChanged(info);
            
            qDebug() << "Disconnected from network:" << id;
        }
    });
    
    return true;
}

bool NetworkManager::setNetworkingEnabled(bool enable)
{
    if (m_networkingEnabled == enable) {
        return true;
    }
    
    // For a real implementation, we would use NetworkManager D-Bus API to enable/disable networking
    // For now, we'll simulate the process
    
    m_networkingEnabled = enable;
    
    // Emit signal
    emit networkingEnabledChanged(m_networkingEnabled);
    
    // If disabling networking, disconnect all networks
    if (!m_networkingEnabled) {
        for (auto it = m_networks.begin(); it != m_networks.end(); ++it) {
            if (it.value().state == "connected") {
                disconnectFromNetwork(it.key());
            }
        }
        
        // Also disable wireless
        if (m_wirelessEnabled) {
            setWirelessEnabled(false);
        }
    }
    
    qDebug() << "Networking enabled:" << m_networkingEnabled;
    
    return true;
}

bool NetworkManager::setWirelessEnabled(bool enable)
{
    if (m_wirelessEnabled == enable) {
        return true;
    }
    
    // Cannot enable wireless if networking is disabled
    if (enable && !m_networkingEnabled) {
        qWarning() << "Cannot enable wireless when networking is disabled";
        return false;
    }
    
    // For a real implementation, we would use NetworkManager D-Bus API to enable/disable wireless
    // For now, we'll simulate the process
    
    m_wirelessEnabled = enable;
    
    // Emit signal
    emit wirelessEnabledChanged(m_wirelessEnabled);
    
    // If disabling wireless, disconnect all wireless networks
    if (!m_wirelessEnabled) {
        for (auto it = m_networks.begin(); it != m_networks.end(); ++it) {
            if (it.value().type == "wifi" && it.value().state == "connected") {
                disconnectFromNetwork(it.key());
            }
        }
        
        // Clear available wireless networks
        m_availableWirelessNetworks.clear();
        emit availableWirelessNetworksChanged();
    } else {
        // If enabling wireless, update available networks
        updateAvailableWirelessNetworks();
    }
    
    qDebug() << "Wireless enabled:" << m_wirelessEnabled;
    
    return true;
}

bool NetworkManager::isNetworkingEnabled() const
{
    return m_networkingEnabled;
}

bool NetworkManager::isWirelessEnabled() const
{
    return m_wirelessEnabled;
}

void NetworkManager::updateNetworkInfo()
{
    // In a real implementation, we would use NetworkManager D-Bus API to get network information
    // For now, we'll use QNetworkInterface to get basic information
    
    QList<QNetworkInterface> interfaces = QNetworkInterface::allInterfaces();
    
    // Create a set of existing network IDs to track removed networks
    QSet<QString> existingIds = QSet<QString>::fromList(m_networks.keys());
    
    // Process each interface
    for (const QNetworkInterface &interface : interfaces) {
        // Skip loopback and non-active interfaces
        if (interface.flags().testFlag(QNetworkInterface::IsLoopBack) ||
            !interface.flags().testFlag(QNetworkInterface::IsUp) ||
            !interface.flags().testFlag(QNetworkInterface::IsRunning)) {
            continue;
        }
        
        QString id = interface.name();
        
        // Determine network type
        QString type;
        if (id.startsWith("wlan") || id.startsWith("wlp")) {
            type = "wifi";
        } else if (id.startsWith("eth") || id.startsWith("enp")) {
            type = "ethernet";
        } else if (id.startsWith("tun") || id.startsWith("tap")) {
            type = "vpn";
        } else {
            type = "other";
        }
        
        // Determine state
        QString state;
        if (interface.flags().testFlag(QNetworkInterface::IsRunning)) {
            bool hasAddress = false;
            for (const QNetworkAddressEntry &entry : interface.addressEntries()) {
                if (entry.ip() != QHostAddress::LocalHost && !entry.ip().isNull()) {
                    hasAddress = true;
                    break;
                }
            }
            
            state = hasAddress ? "connected" : "disconnected";
        } else {
            state = "disconnected";
        }
        
        // Create or update network info
        if (m_networks.contains(id)) {
            NetworkInfo &info = m_networks[id];
            
            // Check if state changed
            if (info.state != state) {
                info.state = state;
                emit networkStateChanged(info);
            }
            
            // Remove from existing IDs
            existingIds.remove(id);
        } else {
            NetworkInfo info;
            info.id = id;
            info.name = interface.humanReadableName();
            info.type = type;
            info.state = state;
            info.isDefault = false;
            
            // Add details
            QVariantMap details;
            details["hardwareAddress"] = interface.hardwareAddress();
            details["mtu"] = interface.maximumTransmissionUnit();
            
            QStringList addressList;
            for (const QNetworkAddressEntry &entry : interface.addressEntries()) {
                if (entry.ip() != QHostAddress::LocalHost && !entry.ip().isNull()) {
                    addressList.append(entry.ip().toString());
                }
            }
            details["addresses"] = addressList;
            
            info.details = details;
            
            // Add to networks
            m_networks[id] = info;
            
            // Emit signal
            emit networkAdded(info);
        }
    }
    
    // Remove networks that no longer exist
    for (const QString &id : existingIds) {
        // Skip wireless networks that we're managing ourselves
        if (id.startsWith("wifi-")) {
            continue;
        }
        
        // Remove network
        NetworkInfo info = m_networks.take(id);
        
        // Emit signal
        emit networkRemoved(id);
        
        qDebug() << "Removed network:" << id;
    }
    
    // Update networking enabled state
    bool networking = !interfaces.isEmpty();
    if (m_networkingEnabled != networking) {
        m_networkingEnabled = networking;
        emit networkingEnabledChanged(m_networkingEnabled);
    }
    
    // Update wireless enabled state
    bool wireless = false;
    for (const QNetworkInterface &interface : interfaces) {
        if ((interface.name().startsWith("wlan") || interface.name().startsWith("wlp")) &&
            interface.flags().testFlag(QNetworkInterface::IsUp)) {
            wireless = true;
            break;
        }
    }
    
    if (m_wirelessEnabled != wireless) {
        m_wirelessEnabled = wireless;
        emit wirelessEnabledChanged(m_wirelessEnabled);
    }
}

void NetworkManager::updateAvailableWirelessNetworks()
{
    // In a real implementation, we would use NetworkManager D-Bus API to get available wireless networks
    // For now, we'll simulate some networks
    
    // Clear existing networks
    m_availableWirelessNetworks.clear();
    
    // Only show networks if wireless is enabled
    if (!m_wirelessEnabled) {
        emit availableWirelessNetworksChanged();
        return;
    }
    
    // Create some simulated networks
    QStringList ssids = {"HomeNetwork", "GuestWiFi", "CoffeeShop", "Library", "FreeWiFi"};
    
    for (const QString &ssid : ssids) {
        NetworkInfo info;
        info.id = "wifi-" + ssid;
        info.name = ssid;
        info.type = "wifi";
        info.state = "available";
        info.isDefault = false;
        
        // Add details
        QVariantMap details;
        details["ssid"] = ssid;
        details["strength"] = qrand() % 100;
        details["security"] = (qrand() % 2 == 0) ? "WPA2" : "Open";
        
        info.details = details;
        
        // Add to available networks
        m_availableWirelessNetworks.append(info);
    }
    
    // Emit signal
    emit availableWirelessNetworksChanged();
    
    qDebug() << "Updated available wireless networks:" << m_availableWirelessNetworks.size();
}

} // namespace VivoX::System
