#pragma once

#include <QObject>
#include <QPoint>
#include <QHash>
#include <QVariantMap>

namespace VivoX::Input {

class GestureEngine;

/**
 * @brief The GestureRecognizer class is the base class for all gesture recognizers.
 * 
 * It provides the interface for recognizing specific types of gestures from
 * input events and notifying the GestureEngine when a gesture is recognized.
 */
class GestureRecognizer : public QObject {
    Q_OBJECT

public:
    explicit GestureRecognizer(GestureEngine *engine, QObject *parent = nullptr);
    virtual ~GestureRecognizer();

    /**
     * @brief Handle a pointer button event
     * @param position The pointer position
     * @param button The button that was pressed or released
     * @param pressed Whether the button is pressed or released
     * @return True if the event was handled, false otherwise
     */
    virtual bool handlePointerButton(const QPoint &position, quint32 button, bool pressed);

    /**
     * @brief Handle a pointer motion event
     * @param position The new pointer position
     * @return True if the event was handled, false otherwise
     */
    virtual bool handlePointerMotion(const QPoint &position);

    /**
     * @brief Handle a touch point event
     * @param id The touch point ID
     * @param position The touch position
     * @param pressed Whether the touch point is pressed or released
     * @return True if the event was handled, false otherwise
     */
    virtual bool handleTouchPoint(qint32 id, const QPoint &position, bool pressed);

    /**
     * @brief Handle a touch motion event
     * @param id The touch point ID
     * @param position The new touch position
     * @return True if the event was handled, false otherwise
     */
    virtual bool handleTouchMotion(qint32 id, const QPoint &position);

    /**
     * @brief Handle a touch frame event
     * @param touchPoints The current touch points
     * @return True if the event was handled, false otherwise
     */
    virtual bool handleTouchFrame(const QHash<int, QPoint> &touchPoints);

    /**
     * @brief Handle a scroll event
     * @param delta The scroll delta
     * @param orientation The scroll orientation (1 for horizontal, 2 for vertical)
     * @return True if the event was handled, false otherwise
     */
    virtual bool handleScroll(qreal delta, quint32 orientation);

signals:
    /**
     * @brief Signal emitted when a gesture is recognized
     * @param type The recognized gesture type
     * @param parameters Additional parameters for the gesture
     */
    void gestureRecognized(int type, const QVariantMap &parameters);

    /**
     * @brief Signal emitted when visual feedback should be shown
     * @param type The gesture type
     * @param progress The progress of the gesture (0.0 to 1.0)
     * @param position The position of the gesture
     */
    void gestureFeedback(int type, qreal progress, const QPoint &position);

protected:
    GestureEngine *m_engine;
};

} // namespace VivoX::Input
