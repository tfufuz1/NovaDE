#pragma once

#include <QObject>
#include <QPoint>
#include <QWaylandSeat>

namespace VivoX::Input {

class ShortcutManager;
class GestureEngine;

/**
 * @brief The InputManager class is the central instance for input event handling.
 * 
 * It receives raw input events from the Wayland compositor and processes them,
 * forwarding them to the appropriate subsystems (ShortcutManager, GestureEngine)
 * or to the focused client window.
 */
class InputManager : public QObject {
    Q_OBJECT

public:
    explicit InputManager(QObject *parent = nullptr);
    ~InputManager();

    /**
     * @brief Initialize the input manager with the Wayland seat
     * @param seat The Wayland seat that provides input capabilities
     */
    void initialize(QWaylandSeat *seat);

    /**
     * @brief Register the shortcut manager for keyboard event processing
     * @param manager The shortcut manager instance
     */
    void registerShortcutManager(ShortcutManager *manager);

    /**
     * @brief Register the gesture engine for touch and pointer event processing
     * @param engine The gesture engine instance
     */
    void registerGestureEngine(GestureEngine *engine);

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

private:
    // Wayland input handlers
    void handleKeyboardEvent(quint32 keyCode, bool pressed, quint32 modifiers);
    void handlePointerEvent(const QPoint &position, quint32 button, bool pressed);
    void handleTouchEvent(qint32 id, const QPoint &position, bool pressed);
    void handleScrollEvent(qreal delta, quint32 orientation);

    // Wayland seat and its capabilities
    QWaylandSeat *m_seat = nullptr;
    QWaylandKeyboard *m_keyboard = nullptr;
    QWaylandPointer *m_pointer = nullptr;
    QWaylandTouch *m_touch = nullptr;

    // Registered subsystems
    ShortcutManager *m_shortcutManager = nullptr;
    GestureEngine *m_gestureEngine = nullptr;
};

} // namespace VivoX::Input
