#pragma once

#include <QObject>
#include <QPoint>
#include <QHash>
#include <QString>
#include <QVector>

namespace VivoX::Input {

class GestureRecognizer;

/**
 * @brief The GestureEngine class recognizes and processes mouse and touch gestures.
 * 
 * It is responsible for detecting various gestures like taps, swipes, pinches, etc.
 * and triggering the associated actions via the ActionManager.
 */
class GestureEngine : public QObject {
    Q_OBJECT

public:
    /**
     * @brief Enum defining the supported gesture types
     */
    enum GestureType {
        Tap,
        DoubleTap,
        LongPress,
        Swipe,
        Pinch,
        Rotate,
        EdgeSwipe,
        HoverButton,
        ActiveCorner,
        Rocker,
        RightButtonDrag,
        RightButtonScroll
    };

    explicit GestureEngine(QObject *parent = nullptr);
    ~GestureEngine();

    /**
     * @brief Initialize the gesture engine
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Handle a pointer button event from the InputManager
     * @param position The pointer position
     * @param button The button that was pressed or released
     * @param pressed Whether the button is pressed or released
     * @return True if the event was handled, false otherwise
     */
    bool handlePointerButton(const QPoint &position, quint32 button, bool pressed);

    /**
     * @brief Handle a pointer motion event from the InputManager
     * @param position The new pointer position
     * @return True if the event was handled, false otherwise
     */
    bool handlePointerMotion(const QPoint &position);

    /**
     * @brief Handle a touch point event from the InputManager
     * @param id The touch point ID
     * @param position The touch position
     * @param pressed Whether the touch point is pressed or released
     * @return True if the event was handled, false otherwise
     */
    bool handleTouchPoint(qint32 id, const QPoint &position, bool pressed);

    /**
     * @brief Handle a touch motion event from the InputManager
     * @param id The touch point ID
     * @param position The new touch position
     * @return True if the event was handled, false otherwise
     */
    bool handleTouchMotion(qint32 id, const QPoint &position);

    /**
     * @brief Handle a touch frame event from the InputManager
     * @return True if the event was handled, false otherwise
     */
    bool handleTouchFrame();

    /**
     * @brief Handle a scroll event from the InputManager
     * @param delta The scroll delta
     * @param orientation The scroll orientation (1 for horizontal, 2 for vertical)
     * @return True if the event was handled, false otherwise
     */
    bool handleScroll(qreal delta, quint32 orientation);

    /**
     * @brief Register a gesture with an action
     * @param type The gesture type
     * @param parameters Additional parameters for the gesture
     * @param actionId The ID of the action to trigger
     * @return True if registration was successful
     */
    bool registerGesture(GestureType type, const QVariantMap &parameters, const QString &actionId);

    /**
     * @brief Unregister a gesture
     * @param type The gesture type
     * @param parameters Additional parameters for the gesture
     * @return True if unregistration was successful
     */
    bool unregisterGesture(GestureType type, const QVariantMap &parameters);

    /**
     * @brief Load gestures from configuration
     * @return True if loading was successful
     */
    bool loadGestures();

    /**
     * @brief Save gestures to configuration
     * @return True if saving was successful
     */
    bool saveGestures();

signals:
    /**
     * @brief Signal emitted when a gesture is recognized
     * @param type The recognized gesture type
     * @param parameters Additional parameters for the gesture
     */
    void gestureRecognized(GestureType type, const QVariantMap &parameters);

    /**
     * @brief Signal emitted when visual feedback should be shown
     * @param type The gesture type
     * @param progress The progress of the gesture (0.0 to 1.0)
     * @param position The position of the gesture
     */
    void gestureFeedback(GestureType type, qreal progress, const QPoint &position);

private:
    // Structure to store gesture configuration
    struct GestureConfig {
        GestureType type;
        QVariantMap parameters;
        QString actionId;
    };

    // List of registered gesture recognizers
    QVector<GestureRecognizer*> m_recognizers;
    
    // List of registered gesture configurations
    QVector<GestureConfig> m_gestureConfigs;
    
    // Current touch points
    QHash<int, QPoint> m_touchPoints;
    
    // Initialize recognizers
    void initializeRecognizers();
    
    // Process a recognized gesture
    void processGesture(GestureType type, const QVariantMap &parameters);
};

} // namespace VivoX::Input
