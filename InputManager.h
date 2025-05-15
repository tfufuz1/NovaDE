     * @param position The mouse position
     * @return True if simulation was successful
     */
    bool simulateMouseMove(const QPoint &position);
    
    /**
     * @brief Simulate mouse wheel scrolling
     * @param delta The scroll delta
     * @param orientation The scroll orientation
     * @return True if simulation was successful
     */
    bool simulateMouseWheel(qreal delta, ScrollOrientation orientation);
    
    /**
     * @brief Simulate a touch press
     * @param id The touch point ID
     * @param position The touch position
     * @return True if simulation was successful
     */
    bool simulateTouchPress(qint32 id, const QPoint &position);
    
    /**
     * @brief Simulate a touch release
     * @param id The touch point ID
     * @param position The touch position
     * @return True if simulation was successful
     */
    bool simulateTouchRelease(qint32 id, const QPoint &position);
    
    /**
     * @brief Simulate touch movement
     * @param id The touch point ID
     * @param position The touch position
     * @return True if simulation was successful
     */
    bool simulateTouchMove(qint32 id, const QPoint &position);
    
    /**
     * @brief Lock the pointer to a specific region
     * @param rect The region to lock the pointer to
     * @return True if locking was successful
     */
    bool lockPointerToRegion(const QRect &rect);
    
    /**
     * @brief Unlock the pointer
     * @return True if unlocking was successful
     */
    bool unlockPointer();
    
    /**
     * @brief Check if the pointer is locked
     * @return True if the pointer is locked
     */
    bool isPointerLocked() const;
    
    /**
     * @brief Warp the pointer to a specific position
     * @param position The position to warp to
     * @return True if warping was successful
     */
    bool warpPointer(const QPoint &position);
    
    /**
     * @brief Get the current pointer position
     * @return The current pointer position
     */
    QPoint pointerPosition() const;
    
    /**
     * @brief Get the current pressed keys
     * @return Set of currently pressed key codes
     */
    QSet<quint32> pressedKeys() const;
    
    /**
     * @brief Get the current pressed mouse buttons
     * @return Set of currently pressed mouse buttons
     */
    QSet<MouseButton> pressedButtons() const;
    
    /**
     * @brief Get the current active touch points
     * @return Map of touch point IDs to positions
     */
    QMap<qint32, QPoint> activeTouchPoints() const;
    
    /**
     * @brief Get the current keyboard modifiers
     * @return The current keyboard modifiers
     */
    KeyboardModifiers keyboardModifiers() const;
    
    /**
     * @brief Save the current input configuration
     * @param name Configuration name
     * @return True if saving was successful
     */
    bool saveConfiguration(const QString &name);
    
    /**
     * @brief Load an input configuration
     * @param name Configuration name
     * @return True if loading was successful
     */
    bool loadConfiguration(const QString &name);
    
    /**
     * @brief Get all saved input configurations
     * @return List of configuration names
     */
    QStringList savedConfigurations() const;
    
    /**
     * @brief Delete a saved input configuration
     * @param name Configuration name
     * @return True if deletion was successful
     */
    bool deleteConfiguration(const QString &name);

signals:
    /**
     * @brief Signal emitted when a keyboard event should be forwarded to the client
     * @param keyCode The key code
     * @param pressed Whether the key is pressed or released
     * @param modifiers The active keyboard modifiers
     */
    void keyboardEventForClient(quint32 keyCode, bool pressed, quint32 modifiers);

    /**
     * @brief Signal emitted when a pointer event should be forwarded to the client
     * @param position The pointer position
     * @param button The button that was pressed or released
     * @param pressed Whether the button is pressed or released
     */
    void pointerEventForClient(const QPoint &position, quint32 button, bool pressed);

    /**
     * @brief Signal emitted when a touch event should be forwarded to the client
     * @param id The touch point ID
     * @param position The touch position
     * @param pressed Whether the touch point is pressed or released
     */
    void touchEventForClient(qint32 id, const QPoint &position, bool pressed);

    /**
     * @brief Signal emitted when a scroll event should be forwarded to the client
     * @param delta The scroll delta
     * @param orientation The scroll orientation (1 for horizontal, 2 for vertical)
     */
    void scrollEventForClient(qreal delta, quint32 orientation);
    
    /**
     * @brief Signal emitted when keyboard input is enabled or disabled
     * @param enabled Whether keyboard input is enabled
     */
    void keyboardEnabledChanged(bool enabled);
    
    /**
     * @brief Signal emitted when pointer input is enabled or disabled
     * @param enabled Whether pointer input is enabled
     */
    void pointerEnabledChanged(bool enabled);
    
    /**
     * @brief Signal emitted when touch input is enabled or disabled
     * @param enabled Whether touch input is enabled
     */
    void touchEnabledChanged(bool enabled);
    
    /**
     * @brief Signal emitted when tablet input is enabled or disabled
     * @param enabled Whether tablet input is enabled
     */
    void tabletEnabledChanged(bool enabled);
    
    /**
     * @brief Signal emitted when the current keyboard layout changes
     * @param layout The new keyboard layout
     */
    void currentKeyboardLayoutChanged(const QString &layout);
    
    /**
     * @brief Signal emitted when the current input profile changes
     * @param profile The new input profile
     */
    void currentInputProfileChanged(const QString &profile);
    
    /**
     * @brief Signal emitted when accessibility features are enabled or disabled
     * @param enabled Whether accessibility features are enabled
     */
    void accessibilityEnabledChanged(bool enabled);
    
    /**
     * @brief Signal emitted when accessibility features change
     * @param features The new accessibility features
     */
    void accessibilityFeaturesChanged(AccessibilityFeatures features);
    
    /**
     * @brief Signal emitted when an input device is added
     * @param device The added device
     */
    void inputDeviceAdded(InputDevice *device);
    
    /**
     * @brief Signal emitted when an input device is removed
     * @param device The removed device
     */
    void inputDeviceRemoved(InputDevice *device);
    
    /**
     * @brief Signal emitted when recording starts
     * @param fileName The recording file name
     */
    void recordingStarted(const QString &fileName);
    
    /**
     * @brief Signal emitted when recording stops
     */
    void recordingStopped();
    
    /**
     * @brief Signal emitted when playback starts
     * @param fileName The playback file name
     * @param loop Whether playback is looping
     */
    void playbackStarted(const QString &fileName, bool loop);
    
    /**
     * @brief Signal emitted when playback stops
     */
    void playbackStopped();
    
    /**
     * @brief Signal emitted when the pointer is locked
     * @param rect The region the pointer is locked to
     */
    void pointerLocked(const QRect &rect);
    
    /**
     * @brief Signal emitted when the pointer is unlocked
     */
    void pointerUnlocked();
    
    /**
     * @brief Signal emitted when the pointer position changes
     * @param position The new pointer position
     */
    void pointerPositionChanged(const QPoint &position);
    
    /**
     * @brief Signal emitted when keyboard modifiers change
     * @param modifiers The new keyboard modifiers
     */
    void keyboardModifiersChanged(KeyboardModifiers modifiers);

private:
    // Wayland input handlers
    void handleKeyboardEvent(quint32 keyCode, bool pressed, quint32 modifiers);
    void handlePointerEvent(const QPoint &position, quint32 button, bool pressed);
    void handleTouchEvent(qint32 id, const QPoint &position, bool pressed);
    void handleScrollEvent(qreal delta, quint32 orientation);
    
    // Device hotplug handlers
    void handleDeviceAdded(InputDevice *device);
    void handleDeviceRemoved(InputDevice *device);
    
    // Accessibility feature handlers
    void handleStickyKeys(quint32 keyCode, bool pressed);
    void handleSlowKeys(quint32 keyCode, bool pressed);
    void handleBounceKeys(quint32 keyCode, bool pressed);
    void handleMouseKeys(quint32 keyCode, bool pressed);
    
    // Recording and playback handlers
    void recordEvent(const QByteArray &eventData);
    void playNextEvent();
    
    // Wayland seat and its capabilities
    QWaylandSeat *m_seat = nullptr;
    QWaylandKeyboard *m_keyboard = nullptr;
    QWaylandPointer *m_pointer = nullptr;
    QWaylandTouch *m_touch = nullptr;
    
    // Input state
    bool m_keyboardEnabled = true;
    bool m_pointerEnabled = true;
    bool m_touchEnabled = true;
    bool m_tabletEnabled = true;
    
    // Current keyboard layout
    QString m_currentKeyboardLayout = "us";
    std::shared_ptr<KeyboardLayout> m_keyboardLayout;
    
    // Current input profile
    QString m_currentInputProfile = "default";
    std::shared_ptr<InputProfile> m_inputProfile;
    
    // Accessibility features
    bool m_accessibilityEnabled = false;
    AccessibilityFeatures m_accessibilityFeatures = NoAccessibilityFeatures;
    
    // Input devices
    QMap<QString, InputDevice*> m_inputDevices;
    
    // Input settings
    std::unique_ptr<MouseSettings> m_mouseSettings;
    std::unique_ptr<TouchSettings> m_touchSettings;
    std::unique_ptr<TabletSettings> m_tabletSettings;
    
    // Recording and playback
    bool m_recording = false;
    bool m_playingBack = false;
    QString m_recordingFileName;
    QString m_playbackFileName;
    bool m_playbackLoop = false;
    QFile m_recordingFile;
    QFile m_playbackFile;
    QTimer m_playbackTimer;
    QElapsedTimer m_recordingTimer;
    
    // Pointer locking
    bool m_pointerLocked = false;
    QRect m_pointerLockRegion;
    
    // Input state tracking
    QPoint m_pointerPosition;
    QSet<quint32> m_pressedKeys;
    QSet<MouseButton> m_pressedButtons;
    QMap<qint32, QPoint> m_activeTouchPoints;
    KeyboardModifiers m_currentModifiers = NoModifier;
    
    // Device callbacks
    QMap<int, std::function<void(InputDevice*, bool)>> m_deviceCallbacks;
    int m_nextCallbackId = 1;

    // Registered subsystems
    ShortcutManager *m_shortcutManager = nullptr;
    GestureEngine *m_gestureEngine = nullptr;
};

Q_DECLARE_OPERATORS_FOR_FLAGS(InputManager::KeyboardModifiers)
Q_DECLARE_OPERATORS_FOR_FLAGS(InputManager::AccessibilityFeatures)

} // namespace VivoX::Input
