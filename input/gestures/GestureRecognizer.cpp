#include "GestureRecognizer.h"
#include "GestureEngine.h"

#include <QDebug>

namespace VivoX::Input {

GestureRecognizer::GestureRecognizer(GestureEngine *engine, QObject *parent)
    : QObject(parent)
    , m_engine(engine)
{
    qDebug() << "GestureRecognizer created";
}

GestureRecognizer::~GestureRecognizer()
{
    qDebug() << "GestureRecognizer destroyed";
}

bool GestureRecognizer::handlePointerButton(const QPoint &position, quint32 button, bool pressed)
{
    // Base implementation does nothing
    Q_UNUSED(position);
    Q_UNUSED(button);
    Q_UNUSED(pressed);
    return false;
}

bool GestureRecognizer::handlePointerMotion(const QPoint &position)
{
    // Base implementation does nothing
    Q_UNUSED(position);
    return false;
}

bool GestureRecognizer::handleTouchPoint(qint32 id, const QPoint &position, bool pressed)
{
    // Base implementation does nothing
    Q_UNUSED(id);
    Q_UNUSED(position);
    Q_UNUSED(pressed);
    return false;
}

bool GestureRecognizer::handleTouchMotion(qint32 id, const QPoint &position)
{
    // Base implementation does nothing
    Q_UNUSED(id);
    Q_UNUSED(position);
    return false;
}

bool GestureRecognizer::handleTouchFrame(const QHash<int, QPoint> &touchPoints)
{
    // Base implementation does nothing
    Q_UNUSED(touchPoints);
    return false;
}

bool GestureRecognizer::handleScroll(qreal delta, quint32 orientation)
{
    // Base implementation does nothing
    Q_UNUSED(delta);
    Q_UNUSED(orientation);
    return false;
}

} // namespace VivoX::Input
