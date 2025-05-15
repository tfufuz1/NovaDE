#include "InputManager.h"
#include "shortcuts/ShortcutManager.h"
#include "gestures/GestureEngine.h"
#include "InputDevice.h"
#include "InputProfile.h"
#include "KeyboardLayout.h"
#include "MouseSettings.h"
#include "TouchSettings.h"
#include "TabletSettings.h"

#include <QDebug>
#include <QDir>
#include <QSettings>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QDataStream>
#include <QDateTime>

namespace VivoX::Input {

InputManager::InputManager(QObject *parent)
    : QObject(parent)
    , m_seat(nullptr)
    , m_keyboard(nullptr)
    , m_pointer(nullptr)
    , m_touch(nullptr)
    , m_keyboardEnabled(true)
    , m_pointerEnabled(true)
    , m_touchEnabled(true)
    , m_tabletEnabled(true)
    , m_currentKeyboardLayout("us")
    , m_currentInputProfile("default")
    , m_accessibilityEnabled(false)
    , m_accessibilityFeatures(NoAccessibilityFeatures)
    , m_recording(false)
    , m_playingBack(false)
    , m_playbackLoop(false)
    , m_pointerLocked(false)
    , m_currentModifiers(NoModifier)
    , m_nextCallbackId(1)
    , m_shortcutManager(nullptr)
    , m_gestureEngine(nullptr)
{
    // Initialize settings
    m_mouseSettings = std::make_unique<MouseSettings>();
    m_touchSettings = std::make_unique<TouchSettings>();
    m_tabletSettings = std::make_unique<TabletSettings>();
    
    // Connect playback timer
    connect(&m_playbackTimer, &QTimer::timeout, this, &InputManager::playNextEvent);
    
    qDebug() << "InputManager created";
}

InputManager::~InputManager()
{
    shutdown();
    qDebug() << "InputManager destroyed";
}

bool InputManager::initialize(QWaylandSeat *seat)
{
    if (!seat) {
        qWarning() << "Cannot initialize InputManager with null seat";
        return false;
    }

    m_seat = seat;
    
    // Connect to keyboard capability
    if (m_seat->keyboard()) {
        m_keyboard = m_seat->keyboard();
        
        // Connect keyboard signals
        connect(m_keyboard, &QWaylandKeyboard::keyPressed, this, 
            [this](quint32 keyCode, quint32 modifiers) {
                handleKeyboardEvent(keyCode, true, modifiers);
            });
            
        connect(m_keyboard, &QWaylandKeyboard::keyReleased, this, 
            [this](quint32 keyCode, quint32 modifiers) {
                handleKeyboardEvent(keyCode, false, modifiers);
            });
            
        connect(m_keyboard, &QWaylandKeyboard::modifiersChanged, this,
            [this](quint32 depressed, quint32 latched, quint32 locked, quint32 group) {
                // Update current modifiers
                KeyboardModifiers mods = NoModifier;
                
                if (depressed & 0x01) mods |= ShiftModifier;
                if (depressed & 0x04) mods |= ControlModifier;
                if (depressed & 0x08) mods |= AltModifier;
                if (depressed & 0x40) mods |= MetaModifier;
                if (depressed & 0x80) mods |= SuperModifier;
                
                if (m_currentModifiers != mods) {
                    m_currentModifiers = mods;
                    emit keyboardModifiersChanged(mods);
                }
            });
            
        connect(m_keyboard, &QWaylandKeyboard::keymapChanged, this,
            [this](const QString &keymap) {
                // Parse keymap to determine layout
                QString layout = "us"; // Default to US layout
                
                // Extract layout from keymap string
                QRegExp layoutRegex("layout\\(([^)]+)\\)");
                if (layoutRegex.indexIn(keymap) != -1) {
                    layout = layoutRegex.cap(1);
                }
                
                // Update layout if changed
                if (m_currentKeyboardLayout != layout) {
                    setCurrentKeyboardLayout(layout);
                }
            });
    }
    
    // Connect to pointer capability
    if (m_seat->pointer()) {
        m_pointer = m_seat->pointer();
        
        // Connect pointer signals
        connect(m_pointer, &QWaylandPointer::buttonPressed, this,
            [this](quint32 button, const QPointF &position) {
                handlePointerEvent(position.toPoint(), button, true);
                
                // Update pressed buttons
                MouseButton mb = static_cast<MouseButton>(button);
                m_pressedButtons.insert(mb);
            });
            
        connect(m_pointer, &QWaylandPointer::buttonReleased, this,
            [this](quint32 button, const QPointF &position) {
                handlePointerEvent(position.toPoint(), button, false);
                
                // Update pressed buttons
                MouseButton mb = static_cast<MouseButton>(button);
                m_pressedButtons.remove(mb);
            });
            
        connect(m_pointer, &QWaylandPointer::motion, this,
            [this](const QPointF &position) {
                QPoint pos = position.toPoint();
                
                // Handle pointer locking
                if (m_pointerLocked && !m_pointerLockRegion.contains(pos)) {
                    pos = QPoint(
                        qBound(m_pointerLockRegion.left(), pos.x(), m_pointerLockRegion.right()),
                        qBound(m_pointerLockRegion.top(), pos.y(), m_pointerLockRegion.bottom())
                    );
                    
                    // Warp pointer back to lock region
                    warpPointer(pos);
                }
                
                // Update pointer position
                if (m_pointerPosition != pos) {
                    m_pointerPosition = pos;
                    emit pointerPositionChanged(pos);
                }
                
                // Forward to gesture engine if registered
                if (m_gestureEngine) {
                    m_gestureEngine->handlePointerMotion(pos);
                }
                
                // Record event if recording
                if (m_recording) {
                    QByteArray eventData;
                    QDataStream stream(&eventData, QIODevice::WriteOnly);
                    stream << QString("pointer_motion") << m_recordingTimer.elapsed() << pos;
                    recordEvent(eventData);
                }
            });
            
        connect(m_pointer, &QWaylandPointer::axis, this,
            [this](Qt::Orientation orientation, qreal delta) {
                ScrollOrientation so = orientation == Qt::Horizontal ? Horizontal : Vertical;
                handleScrollEvent(delta, static_cast<quint32>(so));
                
                // Record event if recording
                if (m_recording) {
                    QByteArray eventData;
                    QDataStream stream(&eventData, QIODevice::WriteOnly);
                    stream << QString("scroll") << m_recordingTimer.elapsed() << delta << static_cast<int>(so);
                    recordEvent(eventData);
                }
            });
    }
    
    // Connect to touch capability
    if (m_seat->touch()) {
        m_touch = m_seat->touch();
        
        // Connect touch signals
        connect(m_touch, &QWaylandTouch::touchPointPressed, this,
            [this](int id, const QPointF &position) {
                QPoint pos = position.toPoint();
                handleTouchEvent(id, pos, true);
                
                // Update active touch points
                m_activeTouchPoints[id] = pos;
                
                // Record event if recording
                if (m_recording) {
                    QByteArray eventData;
                    QDataStream stream(&eventData, QIODevice::WriteOnly);
                    stream << QString("touch_press") << m_recordingTimer.elapsed() << id << pos;
                    recordEvent(eventData);
                }
            });
            
        connect(m_touch, &QWaylandTouch::touchPointReleased, this,
            [this](int id, const QPointF &position) {
                QPoint pos = position.toPoint();
                handleTouchEvent(id, pos, false);
                
                // Update active touch points
                m_activeTouchPoints.remove(id);
                
                // Record event if recording
                if (m_recording) {
                    QByteArray eventData;
                    QDataStream stream(&eventData, QIODevice::WriteOnly);
                    stream << QString("touch_release") << m_recordingTimer.elapsed() << id << pos;
                    recordEvent(eventData);
                }
            });
            
        connect(m_touch, &QWaylandTouch::touchPointMoved, this,
            [this](int id, const QPointF &position) {
                QPoint pos = position.toPoint();
                
                // Update active touch points
                m_activeTouchPoints[id] = pos;
                
                // Forward to gesture engine if registered
                if (m_gestureEngine) {
                    m_gestureEngine->handleTouchMotion(id, pos);
                }
                
                // Record event if recording
                if (m_recording) {
                    QByteArray eventData;
                    QDataStream stream(&eventData, QIODevice::WriteOnly);
                    stream << QString("touch_move") << m_recordingTimer.elapsed() << id << pos;
                    recordEvent(eventData);
                }
            });
            
        connect(m_touch, &QWaylandTouch::touchFrameFinished, this,
            [this]() {
                // Handle touch frame finished
                if (m_gestureEngine) {
                    m_gestureEngine->handleTouchFrame();
                }
                
                // Record event if recording
                if (m_recording) {
                    QByteArray eventData;
                    QDataStream stream(&eventData, QIODevice::WriteOnly);
                    stream << QString("touch_frame") << m_recordingTimer.elapsed();
                    recordEvent(eventData);
                }
            });
    }
    
    // Load settings
    QSettings settings("VivoX", "Input");
    
    // Load input device settings
    m_keyboardEnabled = settings.value("keyboard/enabled", true).toBool();
    m_pointerEnabled = settings.value("pointer/enabled", true).toBool();
    m_touchEnabled = settings.value("touch/enabled", true).toBool();
    m_tabletEnabled = settings.value("tablet/enabled", true).toBool();
    
    // Load keyboard layout
    setCurrentKeyboardLayout(settings.value("keyboard/layout", "us").toString());
    
    // Load input profile
    setCurrentInputProfile(settings.value("profile", "default").toString());
    
    // Load accessibility settings
    m_accessibilityEnabled = settings.value("accessibility/enabled", false).toBool();
    m_accessibilityFeatures = static_cast<AccessibilityFeatures>(settings.value("accessibility/features", 0).toInt());
    
    // Scan for input devices
    scanInputDevices();
    
    qDebug() << "InputManager initialized with Wayland seat";
    return true;
}

void InputManager::shutdown()
{
    // Stop recording or playback if active
    if (m_recording) {
        stopRecording();
    }
    
    if (m_playingBack) {
        stopPlayback();
    }
    
    // Save settings
    QSettings settings("VivoX", "Input");
    
    settings.setValue("keyboard/enabled", m_keyboardEnabled);
    settings.setValue("pointer/enabled", m_pointerEnabled);
    settings.setValue("touch/enabled", m_touchEnabled);
    settings.setValue("tablet/enabled", m_tabletEnabled);
    settings.setValue("keyboard/layout", m_currentKeyboardLayout);
    settings.setValue("profile", m_currentInputProfile);
    settings.setValue("accessibility/enabled", m_accessibilityEnabled);
    settings.setValue("accessibility/features", static_cast<int>(m_accessibilityFeatures));
    
    // Clear device callbacks
    m_deviceCallbacks.clear();
    
    // Clear input devices
    qDeleteAll(m_inputDevices);
    m_inputDevices.clear();
    
    // Clear input state
    m_pressedKeys.clear();
    m_pressedButtons.clear();
    m_activeTouchPoints.clear();
    
    // Clear subsystems
    m_shortcutManager = nullptr;
    m_gestureEngine = nullptr;
    
    // Clear Wayland references
    m_keyboard = nullptr;
    m_pointer = nullptr;
    m_touch = nullptr;
    m_seat = nullptr;
}

void InputManager::registerShortcutManager(ShortcutManager *manager)
{
    m_shortcutManager = manager;
    qDebug() << "ShortcutManager registered with InputManager";
}

void InputManager::registerGestureEngine(GestureEngine *engine)
{
    m_gestureEngine = engine;
    qDebug() << "GestureEngine registered with InputManager";
}

bool InputManager::isKeyboardEnabled() const
{
    return m_keyboardEnabled;
}

void InputManager::setKeyboardEnabled(bool enabled)
{
    if (m_keyboardEnabled == enabled) {
        return;
    }
    
    m_keyboardEnabled = enabled;
    emit keyboardEnabledChanged(enabled);
    
    qDebug() << "Keyboard input" << (enabled ? "enabled" : "disabled");
}

bool InputManager::isPointerEnabled() const
{
    return m_pointerEnabled;
}

void InputManager::setPointerEnabled(bool enabled)
{
    if (m_pointerEnabled == enabled) {
        return;
    }
    
    m_pointerEnabled = enabled;
    emit pointerEnabledChanged(enabled);
    
    qDebug() << "Pointer input" << (enabled ? "enabled" : "disabled");
}

bool InputManager::isTouchEnabled() const
{
    return m_touchEnabled;
}

void InputManager::setTouchEnabled(bool enabled)
{
    if (m_touchEnabled == enabled) {
        return;
    }
    
    m_touchEnabled = enabled;
    emit touchEnabledChanged(enabled);
    
    qDebug() << "Touch input" << (enabled ? "enabled" : "disabled");
}

bool InputManager::isTabletEnabled() const
{
    return m_tabletEnabled;
}

void InputManager::setTabletEnabled(bool enabled)
{
    if (m_tabletEnabled == enabled) {
        return;
    }
    
    m_tabletEnabled = enabled;
    emit tabletEnabledChanged(enabled);
    
    qDebug() << "Tablet input" << (enabled ? "enabled" : "disabled");
}

QString InputManager::currentKeyboardLayout() const
{
    return m_currentKeyboardLayout;
}

void InputManager::setCurrentKeyboardLayout(const QString &layout)
{
    if (m_currentKeyboardLayout == layout) {
        return;
    }
    
    // Check if layout exists
    if (!availableKeyboardLayouts().contains(layout)) {
        qWarning() << "Keyboard layout not found:" << layout;
        return;
    }
    
    // Load keyboard layout
    m_keyboardLayout = std::make_shared<KeyboardLayout>(layout);
    if (!m_keyboardLayout->load()) {
        qWarning() << "Failed to load keyboard layout:" << layout;
        return;
    }
    
    m_currentKeyboardLayout = layout;
    emit currentKeyboardLayoutChanged(layout);
    
    qDebug() << "Set keyboard layout:" << layout;
}

QStringList InputManager::availableKeyboardLayouts() const
{
    QStringList layouts;
    
    // Add standard layouts
    layouts << "us" << "de" << "fr" << "gb" << "es" << "it" << "ru" << "jp";
    
    // Add custom layouts
    QDir layoutsDir(QDir::homePath() + "/.config/vivox/keyboard_layouts");
    if (layoutsDir.exists()) {
        QStringList entries = layoutsDir.entryList(QStringList() << "*.json", QDir::Files);
        
        for (const QString &entry : entries) {
            layouts << entry.left(entry.length() - 5); // Remove .json extension
        }
    }
    
    return layouts;
}

QString InputManager::currentInputProfile() const
{
    return m_currentInputProfile;
}

void InputManager::setCurrentInputProfile(const QString &profile)
{
    if (m_currentInputProfile == profile) {
        return;
    }
    
    // Check if profile exists
    if (!availableInputProfiles().contains(profile)) {
        qWarning() << "Input profile not found:" << profile;
        return;
    }
    
    // Load input profile
    m_inputProfile = std::make_shared<InputProfile>(pro
(Content truncated due to size limit. Use line ranges to read in chunks)