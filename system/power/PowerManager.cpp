#include "PowerManager.h"

#include <QDebug>
#include <QProcess>
#include <QTimer>
#include <QDBusConnection>
#include <QDBusInterface>
#include <QDBusReply>
#include <QFile>
#include <QSettings>

namespace VivoX::System {

PowerManager::PowerManager(QObject *parent)
    : QObject(parent)
    , m_powerState(OnAC)
    , m_batteryLevel(100)
    , m_batteryTimeRemaining(-1)
{
    // Initialize default power settings
    m_powerSettings["display_timeout_ac"] = 15 * 60;     // 15 minutes on AC
    m_powerSettings["display_timeout_battery"] = 5 * 60; // 5 minutes on battery
    m_powerSettings["sleep_timeout_ac"] = 30 * 60;       // 30 minutes on AC
    m_powerSettings["sleep_timeout_battery"] = 15 * 60;  // 15 minutes on battery
    m_powerSettings["performance_mode_ac"] = "balanced"; // balanced, performance, powersave
    m_powerSettings["performance_mode_battery"] = "powersave";
    m_powerSettings["lid_close_action"] = "suspend";     // suspend, hibernate, shutdown, nothing
    m_powerSettings["power_button_action"] = "suspend";  // suspend, hibernate, shutdown, nothing
    m_powerSettings["critical_battery_action"] = "hibernate"; // suspend, hibernate, shutdown
    m_powerSettings["critical_battery_level"] = 5;       // 5% is critical
    m_powerSettings["low_battery_level"] = 15;           // 15% is low
    m_powerSettings["dim_display_on_battery"] = true;
    m_powerSettings["reduce_performance_on_battery"] = true;
    
    qDebug() << "PowerManager created";
}

PowerManager::~PowerManager()
{
    qDebug() << "PowerManager destroyed";
}

bool PowerManager::initialize()
{
    // Connect to UPower via DBus if available
    QDBusConnection systemBus = QDBusConnection::systemBus();
    
    if (systemBus.isConnected()) {
        // Connect to UPower signals
        systemBus.connect("org.freedesktop.UPower", 
                         "/org/freedesktop/UPower",
                         "org.freedesktop.UPower",
                         "DeviceChanged",
                         this, SLOT(updatePowerInfo()));
        
        systemBus.connect("org.freedesktop.UPower", 
                         "/org/freedesktop/UPower",
                         "org.freedesktop.UPower",
                         "Changed",
                         this, SLOT(updatePowerInfo()));
    }
    
    // Load power settings from configuration
    QSettings settings("VivoX", "PowerManager");
    
    for (auto it = m_powerSettings.begin(); it != m_powerSettings.end(); ++it) {
        if (settings.contains(it.key())) {
            m_powerSettings[it.key()] = settings.value(it.key());
        }
    }
    
    // Start a timer to periodically update power information
    QTimer *timer = new QTimer(this);
    connect(timer, &QTimer::timeout, this, &PowerManager::updatePowerInfo);
    timer->start(30000); // Update every 30 seconds
    
    // Initial update
    updatePowerInfo();
    
    qDebug() << "PowerManager initialized";
    return true;
}

PowerManager::PowerState PowerManager::getPowerState() const
{
    return m_powerState;
}

int PowerManager::getBatteryLevel() const
{
    return m_batteryLevel;
}

int PowerManager::getBatteryTimeRemaining() const
{
    return m_batteryTimeRemaining;
}

bool PowerManager::canPerformAction(PowerAction action) const
{
    // Check if the system can perform the requested power action
    switch (action) {
        case Suspend:
            // Check if suspend is supported
            return QFile::exists("/sys/power/state") && 
                   QFile("/sys/power/state").open(QIODevice::ReadOnly) && 
                   QFile("/sys/power/state").readAll().contains("mem");
            
        case Hibernate:
            // Check if hibernate is supported
            return QFile::exists("/sys/power/state") && 
                   QFile("/sys/power/state").open(QIODevice::ReadOnly) && 
                   QFile("/sys/power/state").readAll().contains("disk");
            
        case Shutdown:
        case Reboot:
        case LogOut:
            // These actions are generally always available
            return true;
            
        default:
            return false;
    }
}

bool PowerManager::performAction(PowerAction action)
{
    if (!canPerformAction(action)) {
        qWarning() << "Cannot perform power action:" << action;
        return false;
    }
    
    // Perform the requested power action
    switch (action) {
        case Suspend:
            // Use systemd if available, otherwise use direct method
            if (QFile::exists("/usr/bin/systemctl")) {
                return QProcess::startDetached("systemctl", QStringList() << "suspend");
            } else {
                QFile file("/sys/power/state");
                if (file.open(QIODevice::WriteOnly)) {
                    file.write("mem");
                    file.close();
                    return true;
                }
                return false;
            }
            
        case Hibernate:
            // Use systemd if available, otherwise use direct method
            if (QFile::exists("/usr/bin/systemctl")) {
                return QProcess::startDetached("systemctl", QStringList() << "hibernate");
            } else {
                QFile file("/sys/power/state");
                if (file.open(QIODevice::WriteOnly)) {
                    file.write("disk");
                    file.close();
                    return true;
                }
                return false;
            }
            
        case Shutdown:
            // Use systemd if available, otherwise use direct method
            if (QFile::exists("/usr/bin/systemctl")) {
                return QProcess::startDetached("systemctl", QStringList() << "poweroff");
            } else {
                return QProcess::startDetached("shutdown", QStringList() << "-h" << "now");
            }
            
        case Reboot:
            // Use systemd if available, otherwise use direct method
            if (QFile::exists("/usr/bin/systemctl")) {
                return QProcess::startDetached("systemctl", QStringList() << "reboot");
            } else {
                return QProcess::startDetached("shutdown", QStringList() << "-r" << "now");
            }
            
        case LogOut:
            // This depends on the session manager
            // For systemd:
            if (QFile::exists("/usr/bin/loginctl")) {
                return QProcess::startDetached("loginctl", QStringList() << "terminate-session" << qgetenv("XDG_SESSION_ID"));
            }
            // For other session managers, we would need to implement specific methods
            return false;
            
        default:
            return false;
    }
}

QVariantMap PowerManager::getPowerSettings() const
{
    return m_powerSettings;
}

bool PowerManager::setPowerSetting(const QString &key, const QVariant &value)
{
    if (!m_powerSettings.contains(key)) {
        qWarning() << "Unknown power setting:" << key;
        return false;
    }
    
    // Validate the value based on the setting type
    if (key == "display_timeout_ac" || key == "display_timeout_battery" || 
        key == "sleep_timeout_ac" || key == "sleep_timeout_battery") {
        // These should be integers (seconds)
        if (!value.canConvert<int>()) {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
        int seconds = value.toInt();
        if (seconds < 0) {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
    } else if (key == "performance_mode_ac" || key == "performance_mode_battery") {
        // These should be strings from a predefined set
        if (!value.canConvert<QString>()) {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
        QString mode = value.toString();
        if (mode != "balanced" && mode != "performance" && mode != "powersave") {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
    } else if (key == "lid_close_action" || key == "power_button_action") {
        // These should be strings from a predefined set
        if (!value.canConvert<QString>()) {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
        QString action = value.toString();
        if (action != "suspend" && action != "hibernate" && action != "shutdown" && action != "nothing") {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
    } else if (key == "critical_battery_action") {
        // These should be strings from a predefined set
        if (!value.canConvert<QString>()) {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
        QString action = value.toString();
        if (action != "suspend" && action != "hibernate" && action != "shutdown") {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
    } else if (key == "critical_battery_level" || key == "low_battery_level") {
        // These should be integers (percentage)
        if (!value.canConvert<int>()) {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
        int level = value.toInt();
        if (level < 0 || level > 100) {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
        // Ensure critical level is lower than low level
        if (key == "critical_battery_level" && level >= m_powerSettings["low_battery_level"].toInt()) {
            qWarning() << "Critical battery level must be lower than low battery level";
            return false;
        }
        if (key == "low_battery_level" && level <= m_powerSettings["critical_battery_level"].toInt()) {
            qWarning() << "Low battery level must be higher than critical battery level";
            return false;
        }
    } else if (key == "dim_display_on_battery" || key == "reduce_performance_on_battery") {
        // These should be booleans
        if (!value.canConvert<bool>()) {
            qWarning() << "Invalid value for" << key << ":" << value;
            return false;
        }
    }
    
    // Update the setting
    m_powerSettings[key] = value;
    
    // Save to configuration
    QSettings settings("VivoX", "PowerManager");
    settings.setValue(key, value);
    
    // Emit signal
    emit powerSettingChanged(key, value);
    
    qDebug() << "Power setting changed:" << key << "=" << value;
    
    // Apply the setting if necessary
    if (key == "performance_mode_ac" && m_powerState == OnAC) {
        applyPerformanceMode(value.toString());
    } else if (key == "performance_mode_battery" && (m_powerState == OnBattery || m_powerState == LowBattery || m_powerState == CriticalBattery)) {
        applyPerformanceMode(value.toString());
    }
    
    return true;
}

void PowerManager::updatePowerInfo()
{
    // Try to get power information from UPower via DBus
    QDBusConnection systemBus = QDBusConnection::systemBus();
    
    if (systemBus.isConnected()) {
        QDBusInterface upower("org.freedesktop.UPower", 
                             "/org/freedesktop/UPower",
                             "org.freedesktop.UPower",
                             systemBus);
        
        if (upower.isValid()) {
            // Get the primary battery device
            QDBusReply<QDBusObjectPath> reply = upower.call("GetDisplayDevice");
            
            if (reply.isValid()) {
                QDBusObjectPath path = reply.value();
                
                QDBusInterface device("org.freedesktop.UPower", 
                                     path.path(),
                                     "org.freedesktop.UPower.Device",
                                     systemBus);
                
                if (device.isValid()) {
                    // Get battery level
                    QDBusReply<double> levelReply = device.call("GetPercentage");
                    if (levelReply.isValid()) {
                        int newLevel = qRound(levelReply.value());
                        
                        if (newLevel != m_batteryLevel) {
                            m_batteryLevel = newLevel;
                            emit batteryLevelChanged(m_batteryLevel);
                        }
                    }
                    
                    // Get battery time remaining
                    QDBusReply<qlonglong> timeReply = device.call("GetTimeToEmpty");
                    if (timeReply.isValid()) {
                        int newTime = timeReply.value();
                        
                        if (newTime != m_batteryTimeRemaining) {
                            m_batteryTimeRemaining = newTime;
                        }
                    }
                    
                    // Get power state
                    QDBusReply<uint> stateReply = device.call("GetState");
                    if (stateReply.isValid()) {
                        uint state = stateReply.value();
                        PowerState newState;
                        
                        // UPower states: 0=unknown, 1=charging, 2=discharging, 3=empty, 4=fully charged, 5=pending charge, 6=pending discharge
                        if (state == 1 || state == 4 || state == 5) {
                            newState = OnAC;
                        } else if (state == 2 || state == 6) {
                            if (m_batteryLevel <= m_powerSettings["critical_battery_level"].toInt()) {
                                newState = CriticalBattery;
                            } else if (m_batteryLevel <= m_powerSettings["low_battery_level"].toInt()) {
                                newState = LowBattery;
                            } else {
                                newState = OnBattery;
                            }
                        } else {
                            // Default to AC if unknown
                            newState = OnAC;
                        }
                        
                        if (newState != m_powerState) {
                            m_powerState = newState;
                            emit powerStateChanged(m_powerState);
                            
                            // Apply appropriate performance mode
                            if (m_powerState == OnAC) {
                                applyPerformanceMode(m_powerSettings["performance_mode_ac"].toString());
                            } else {
                                applyPerformanceMode(m_powerSettings["performance_mode_battery"].toString());
                                
                                // Check if we need to take action for critical battery
                                if (m_powerState == CriticalBattery) {
                                    QString action = m_powerSettings["critical_battery_action"].toString();
                                    
                                    if (action == "suspend") {
                                        performAction(Suspend);
                                    } else if (action == "hibernate") {
                                        performAction(Hibernate);
                                    } else if (action == "shutdown") {
                                        performAction(Shutdown);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            return;
        }
    }
    
    // Fallback: Try to get power information from sysfs
    if (QFile::exists("/sys/class/power_supply/BAT0") || QFile::exists("/sys/class/power_supply/BAT1")) {
        // We have a battery
        bool onAC = false;
        
        // Check if we're on AC power
        if (QFile::exists("/sys/class/power_supply/AC/online")) {
            QFile acFile("/sys/class/power_supply/AC/online");
            if (acFile.open(QIODevice::ReadOnly)) {
                QString acStatus = acFile.readAll().trimmed();
                acFile.close();
                
                onAC = (acStatus == "1");
            }
        }
        
        // Get battery level
        int newLevel = -1;
        QString batteryPath = QFile::exists("/sys/class/power_supply/BAT0") ? "/sys/class/power_supply/BAT0" : "/sys/class/power_supply/BAT1";
        
        if (QFile::exists(batteryPath + "/capacity")) {
            QFile capacityFile(batteryPath + "/capacity");
            if (capacityFile.open(QIODevice::ReadOnly)) {
                QString capacityStr = capacityFile.readAll().trimmed();
                capacityFile.close();
                
                bool ok;
                newLevel = capacityStr.toInt(&ok);
                
                if (!ok) {
                    newLevel = -1;
                }
            }
        }
        
        if (newLevel != m_batteryLevel && newLevel != -1) {
            m_batteryLevel = newLevel;
            emit batteryLevelChanged(m_batteryLevel);
        }
        
        // Determine power state
        PowerState newState;
        
        if (onAC) {
            newState = OnAC;
        } else {
            if (m_batteryLevel <= m_powerSettings["critical_battery_level"].toInt()) {
                newState = CriticalBattery;
            } else if (m_batteryLevel <= m_powerSettings["low_battery_level"].toInt()) {
                newState = LowBattery;
            } else {
                newState = OnBattery;
            }
        }
        
        if (newState != m_powerState) {
            m_powerState = newState;
            emit powerStateChanged(m_powerState);
            
            // Apply appropriate performance mode
            if (m_powerState == OnAC) {
                applyPerformanceMode(m_powerSettings["performance_mode_ac"].toString());
            } else {
                applyPerformanceMode(m_powerSettings["performance_mode_battery"].toString());
                
                // Check if we need to take action for critical battery
                if (m_powerState == CriticalBattery) {
                    QString action = m_powerSettings["critical_battery_action"].toString();
                    
                    if (action == "suspend") {
                        performAction(Suspend);
                    } else if (action == "hibernate") {
                        performAction(Hibernate);
                    } else if (action == "shutdown") {
                        performAction(Shutdown);
                    }
                }
            }
        }
    } else {
        // No battery, assume we're on AC
        if (m_powerState != OnAC) {
            m_powerState = OnAC;
            m_batteryLevel = -1;
            m_batteryTimeRemaining = -1;
            
            emit powerStateChanged(m_powerState);
            emit batteryLevelChanged(m_batteryLevel);
            
            // Apply AC performance mode
            applyPerformanceMode(m_powerSettings["performance_mode_ac"].toString());
        }
    }
}

void PowerManager::applyPerformanceMode(const QString &mode)
{
    // Apply the specified performance mode
    // This is system-dependent and might involve:
    // - Setting CPU governor
    // - Adjusting screen brightness
    // - Configuring other power-saving features
    
    if (mode == "performance") {
        // Set CPU governor to performance if possible
        if (QFile::exists("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")) {
            for (int i = 0; i < 16; i++) { // Try up to 16 CPUs
                QString path = QString("/sys/devices/system/cpu/cpu%1/cpufreq/scaling_governor").arg(i);
                
                if (!QFile::exists(path)) {
                    break;
                }
                
                QFile file(path);
                if (file.open(QIODevice::WriteOnly)) {
                    file.write("performance");
                    file.close();
                }
            }
        }
        
        // Set maximum brightness if on AC
        if (m_powerState == OnAC) {
            if (QFile::exists("/sys/class/backlight/acpi_video0/brightness")) {
                QFile maxBrightFile("/sys/class/backlight/acpi_video0/max_brightness");
                if (maxBrightFile.open(QIODevice::ReadOnly)) {
                    QString maxBrightStr = maxBrightFile.readAll().trimmed();
                    maxBrightFile.close();
                    
                    QFile brightFile("/sys/class/backlight/acpi_video0/brightness");
                    if (brightFile.open(QIODevice::WriteOnly)) {
                        brightFile.write(maxBrightStr.toUtf8());
                        brightFile.close();
                    }
                }
            }
        }
    } else if (mode == "powersave") {
        // Set CPU governor to powersave if possible
        if (QFile::exists("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")) {
            for (int i = 0; i < 16; i++) { // Try up to 16 CPUs
                QString path = QString("/sys/devices/system/cpu/cpu%1/cpufreq/scaling_governor").arg(i);
                
                if (!QFile::exists(path)) {
                    break;
                }
                
                QFile file(path);
                if (file.open(QIODevice::WriteOnly)) {
                    file.write("powersave");
                    file.close();
                }
            }
        }
        
        // Reduce brightness if on battery and dim_display_on_battery is enabled
        if (m_powerState != OnAC && m_powerSettings["dim_display_on_battery"].toBool()) {
            if (QFile::exists("/sys/class/backlight/acpi_video0/brightness")) {
                QFile maxBrightFile("/sys/class/backlight/acpi_video0/max_brightness");
                if (maxBrightFile.open(QIODevice::ReadOnly)) {
                    QString maxBrightStr = maxBrightFile.readAll().trimmed();
                    maxBrightFile.close();
                    
                    bool ok;
                    int maxBright = maxBrightStr.toInt(&ok);
                    
                    if (ok) {
                        // Set to 50% of max brightness
                        int newBright = maxBright / 2;
                        
                        QFile brightFile("/sys/class/backlight/acpi_video0/brightness");
                        if (brightFile.open(QIODevice::WriteOnly)) {
                            brightFile.write(QString::number(newBright).toUtf8());
                            brightFile.close();
                        }
                    }
                }
            }
        }
    } else if (mode == "balanced") {
        // Set CPU governor to ondemand if possible
        if (QFile::exists("/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor")) {
            for (int i = 0; i < 16; i++) { // Try up to 16 CPUs
                QString path = QString("/sys/devices/system/cpu/cpu%1/cpufreq/scaling_governor").arg(i);
                
                if (!QFile::exists(path)) {
                    break;
                }
                
                QFile file(path);
                if (file.open(QIODevice::WriteOnly)) {
                    file.write("ondemand");
                    file.close();
                }
            }
        }
        
        // Set appropriate brightness based on power state
        if (m_powerState == OnAC) {
            if (QFile::exists("/sys/class/backlight/acpi_video0/brightness")) {
                QFile maxBrightFile("/sys/class/backlight/acpi_video0/max_brightness");
                if (maxBrightFile.open(QIODevice::ReadOnly)) {
                    QString maxBrightStr = maxBrightFile.readAll().trimmed();
                    maxBrightFile.close();
                    
                    QFile brightFile("/sys/class/backlight/acpi_video0/brightness");
                    if (brightFile.open(QIODevice::WriteOnly)) {
                        brightFile.write(maxBrightStr.toUtf8());
                        brightFile.close();
                    }
                }
            }
        } else if (m_powerSettings["dim_display_on_battery"].toBool()) {
            if (QFile::exists("/sys/class/backlight/acpi_video0/brightness")) {
                QFile maxBrightFile("/sys/class/backlight/acpi_video0/max_brightness");
                if (maxBrightFile.open(QIODevice::ReadOnly)) {
                    QString maxBrightStr = maxBrightFile.readAll().trimmed();
                    maxBrightFile.close();
                    
                    bool ok;
                    int maxBright = maxBrightStr.toInt(&ok);
                    
                    if (ok) {
                        // Set to 70% of max brightness
                        int newBright = (maxBright * 7) / 10;
                        
                        QFile brightFile("/sys/class/backlight/acpi_video0/brightness");
                        if (brightFile.open(QIODevice::WriteOnly)) {
                            brightFile.write(QString::number(newBright).toUtf8());
                            brightFile.close();
                        }
                    }
                }
            }
        }
    }
}

} // namespace VivoX::System
