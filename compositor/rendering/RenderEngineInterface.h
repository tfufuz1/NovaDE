#pragma once

#include <QObject>
#include <QString>
#include <QList>
#include <QVariant>

namespace VivoX {
namespace Rendering {

/**
 * @brief Interface for the rendering engine
 * 
 * This interface defines the functionality that the rendering engine must implement.
 * It is responsible for rendering the desktop and applying visual effects.
 */
class RenderEngineInterface {
public:
    virtual ~RenderEngineInterface() = default;

    /**
     * @brief Initialize the rendering engine
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Start the rendering loop
     */
    virtual void start() = 0;

    /**
     * @brief Stop the rendering loop
     */
    virtual void stop() = 0;

    /**
     * @brief Check if the rendering engine is running
     * 
     * @return bool True if the rendering engine is running, false otherwise
     */
    virtual bool isRunning() const = 0;

    /**
     * @brief Set the target frame rate
     * 
     * @param fps The target frames per second
     */
    virtual void setTargetFrameRate(int fps) = 0;

    /**
     * @brief Get the current frame rate
     * 
     * @return int The current frames per second
     */
    virtual int currentFrameRate() const = 0;

    /**
     * @brief Enable or disable vertical sync
     * 
     * @param enable Whether to enable vertical sync
     */
    virtual void setVSync(bool enable) = 0;

    /**
     * @brief Check if vertical sync is enabled
     * 
     * @return bool True if vertical sync is enabled, false otherwise
     */
    virtual bool isVSyncEnabled() const = 0;

    /**
     * @brief Enable or disable a visual effect
     * 
     * @param effect The effect name
     * @param enable Whether to enable the effect
     */
    virtual void setEffectEnabled(const QString& effect, bool enable) = 0;

    /**
     * @brief Check if a visual effect is enabled
     * 
     * @param effect The effect name
     * @return bool True if the effect is enabled, false otherwise
     */
    virtual bool isEffectEnabled(const QString& effect) const = 0;

    /**
     * @brief Get the list of available visual effects
     * 
     * @return QStringList The list of effect names
     */
    virtual QStringList availableEffects() const = 0;

    /**
     * @brief Set a property of a visual effect
     * 
     * @param effect The effect name
     * @param property The property name
     * @param value The property value
     */
    virtual void setEffectProperty(const QString& effect, const QString& property, const QVariant& value) = 0;

    /**
     * @brief Get a property of a visual effect
     * 
     * @param effect The effect name
     * @param property The property name
     * @return QVariant The property value
     */
    virtual QVariant effectProperty(const QString& effect, const QString& property) const = 0;

    /**
     * @brief Check if hardware acceleration is available
     * 
     * @return bool True if hardware acceleration is available, false otherwise
     */
    virtual bool isHardwareAccelerationAvailable() const = 0;

    /**
     * @brief Enable or disable hardware acceleration
     * 
     * @param enable Whether to enable hardware acceleration
     */
    virtual void setHardwareAcceleration(bool enable) = 0;

    /**
     * @brief Check if hardware acceleration is enabled
     * 
     * @return bool True if hardware acceleration is enabled, false otherwise
     */
    virtual bool isHardwareAccelerationEnabled() const = 0;

    /**
     * @brief Get information about the graphics hardware
     * 
     * @return QVariantMap A map containing information about the graphics hardware
     */
    virtual QVariantMap hardwareInfo() const = 0;
};

} // namespace Rendering
} // namespace VivoX
