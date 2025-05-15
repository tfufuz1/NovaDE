#pragma once

#include <QObject>
#include <QString>
#include <QList>
#include <QPoint>
#include <QKeySequence>

namespace VivoX {
namespace Input {

class Gesture;
class Shortcut;

/**
 * @brief Interface for the input manager
 * 
 * This interface defines the functionality that the input manager must implement.
 * It is responsible for handling input events from various sources and routing them
 * to the appropriate components.
 */
class InputManagerInterface {
public:
    virtual ~InputManagerInterface() = default;

    /**
     * @brief Initialize the input manager
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Process a key press event
     * 
     * @param key The key code
     * @param modifiers The keyboard modifiers
     * @return bool True if the event was handled, false otherwise
     */
    virtual bool processKeyPress(int key, int modifiers) = 0;

    /**
     * @brief Process a key release event
     * 
     * @param key The key code
     * @param modifiers The keyboard modifiers
     * @return bool True if the event was handled, false otherwise
     */
    virtual bool processKeyRelease(int key, int modifiers) = 0;

    /**
     * @brief Process a mouse press event
     * 
     * @param button The mouse button
     * @param pos The mouse position
     * @param modifiers The keyboard modifiers
     * @return bool True if the event was handled, false otherwise
     */
    virtual bool processMousePress(int button, const QPoint& pos, int modifiers) = 0;

    /**
     * @brief Process a mouse release event
     * 
     * @param button The mouse button
     * @param pos The mouse position
     * @param modifiers The keyboard modifiers
     * @return bool True if the event was handled, false otherwise
     */
    virtual bool processMouseRelease(int button, const QPoint& pos, int modifiers) = 0;

    /**
     * @brief Process a mouse move event
     * 
     * @param pos The mouse position
     * @param buttons The pressed mouse buttons
     * @param modifiers The keyboard modifiers
     * @return bool True if the event was handled, false otherwise
     */
    virtual bool processMouseMove(const QPoint& pos, int buttons, int modifiers) = 0;

    /**
     * @brief Process a touch begin event
     * 
     * @param id The touch point ID
     * @param pos The touch position
     * @return bool True if the event was handled, false otherwise
     */
    virtual bool processTouchBegin(int id, const QPoint& pos) = 0;

    /**
     * @brief Process a touch update event
     * 
     * @param id The touch point ID
     * @param pos The touch position
     * @return bool True if the event was handled, false otherwise
     */
    virtual bool processTouchUpdate(int id, const QPoint& pos) = 0;

    /**
     * @brief Process a touch end event
     * 
     * @param id The touch point ID
     * @param pos The touch position
     * @return bool True if the event was handled, false otherwise
     */
    virtual bool processTouchEnd(int id, const QPoint& pos) = 0;

    /**
     * @brief Register a shortcut
     * 
     * @param sequence The key sequence
     * @param context The context in which the shortcut is active
     * @param action The action to trigger when the shortcut is activated
     * @return Shortcut* The registered shortcut
     */
    virtual Shortcut* registerShortcut(const QKeySequence& sequence, int context, const QString& action) = 0;

    /**
     * @brief Unregister a shortcut
     * 
     * @param shortcut The shortcut to unregister
     */
    virtual void unregisterShortcut(Shortcut* shortcut) = 0;

    /**
     * @brief Get all registered shortcuts
     * 
     * @return QList<Shortcut*> The list of shortcuts
     */
    virtual QList<Shortcut*> shortcuts() const = 0;

    /**
     * @brief Register a gesture
     * 
     * @param type The gesture type
     * @param action The action to trigger when the gesture is recognized
     * @return Gesture* The registered gesture
     */
    virtual Gesture* registerGesture(int type, const QString& action) = 0;

    /**
     * @brief Unregister a gesture
     * 
     * @param gesture The gesture to unregister
     */
    virtual void unregisterGesture(Gesture* gesture) = 0;

    /**
     * @brief Get all registered gestures
     * 
     * @return QList<Gesture*> The list of gestures
     */
    virtual QList<Gesture*> gestures() const = 0;

    /**
     * @brief Set the input focus to a specific surface
     * 
     * @param surface The surface to focus
     */
    virtual void setFocus(QObject* surface) = 0;

    /**
     * @brief Get the current focused surface
     * 
     * @return QObject* The focused surface
     */
    virtual QObject* focusedSurface() const = 0;
};

} // namespace Input
} // namespace VivoX
