#include "WaylandCompositor.h"
#include "WaylandProtocols.h"
#include "../rendering/RenderEngine.h"

#include <QDebug>
#include <QWaylandQuickOutput>
#include <QWaylandQuickCompositor>

namespace VivoX::Compositor {

WaylandCompositor::WaylandCompositor(QObject *parent)
    : QObject(parent)
    , m_compositor(nullptr)
    , m_seat(nullptr)
    , m_xdgShell(nullptr)
    , m_protocols(nullptr)
    , m_renderEngine(nullptr)
    , m_primaryOutput(nullptr)
{
    qDebug() << "WaylandCompositor created";
}

WaylandCompositor::~WaylandCompositor()
{
    qDebug() << "WaylandCompositor destroyed";
    closeAllSurfaces();
}

bool WaylandCompositor::initialize(RenderEngine *renderEngine, const QString &socketName)
{
    if (!renderEngine) {
        qWarning() << "Cannot initialize WaylandCompositor with null RenderEngine";
        return false;
    }

    m_renderEngine = renderEngine;

    // Create the Wayland compositor
    m_compositor = new QWaylandQuickCompositor(this);
    m_compositor->setSocketName(socketName);
    
    // Create the Wayland seat
    m_seat = new QWaylandSeat(m_compositor);
    m_seat->setName(socketName + "-seat");
    
    // Create keyboard, pointer, and touch capabilities
    m_seat->setHasKeyboard(true);
    m_seat->setHasPointer(true);
    m_seat->setHasTouch(true);
    
    // Create the XDG shell
    m_xdgShell = new QWaylandXdgShell(m_compositor);
    
    // Create and initialize the Wayland protocols manager
    m_protocols = new WaylandProtocols(m_compositor, this);
    if (!m_protocols->initialize()) {
        qWarning() << "Failed to initialize Wayland protocols";
        return false;
    }
    
    // Connect signals
    connectSignals();
    
    // Create the compositor
    m_compositor->create();
    
    qDebug() << "WaylandCompositor initialized with socket name:" << socketName;
    
    return true;
}

QWaylandCompositor *WaylandCompositor::compositor() const
{
    return m_compositor;
}

QWaylandSeat *WaylandCompositor::seat() const
{
    return m_seat;
}

QVector<QWaylandOutput *> WaylandCompositor::outputs() const
{
    return m_outputs;
}

QWaylandXdgShell *WaylandCompositor::xdgShell() const
{
    return m_xdgShell;
}

WaylandProtocols *WaylandCompositor::protocols() const
{
    return m_protocols;
}

void WaylandCompositor::addOutput(QWaylandOutput *output)
{
    if (!output) {
        qWarning() << "Cannot add null output to WaylandCompositor";
        return;
    }
    
    if (m_outputs.contains(output)) {
        qWarning() << "Output already added to WaylandCompositor";
        return;
    }
    
    m_outputs.append(output);
    
    // Set up the output
    output->setCompositor(m_compositor);
    
    // If this is the first output, set it as primary
    if (m_outputs.size() == 1 && !m_primaryOutput) {
        setPrimaryOutput(output);
    }
    
    qDebug() << "Added output to WaylandCompositor:" << output->manufacturer() << output->model();
}

void WaylandCompositor::removeOutput(QWaylandOutput *output)
{
    if (!output) {
        qWarning() << "Cannot remove null output from WaylandCompositor";
        return;
    }
    
    if (!m_outputs.contains(output)) {
        qWarning() << "Output not found in WaylandCompositor";
        return;
    }
    
    // If this is the primary output, clear it
    if (m_primaryOutput == output) {
        m_primaryOutput = nullptr;
        
        // Set a new primary output if available
        if (!m_outputs.isEmpty() && m_outputs.size() > 1) {
            for (QWaylandOutput *newPrimary : m_outputs) {
                if (newPrimary != output) {
                    setPrimaryOutput(newPrimary);
                    break;
                }
            }
        }
    }
    
    m_outputs.removeOne(output);
    
    qDebug() << "Removed output from WaylandCompositor";
}

QWaylandOutput *WaylandCompositor::primaryOutput() const
{
    return m_primaryOutput;
}

void WaylandCompositor::setPrimaryOutput(QWaylandOutput *output)
{
    if (!output || !m_outputs.contains(output)) {
        qWarning() << "Cannot set invalid output as primary";
        return;
    }
    
    if (m_primaryOutput != output) {
        m_primaryOutput = output;
        emit primaryOutputChanged(m_primaryOutput);
        qDebug() << "Set primary output:" << output->manufacturer() << output->model();
    }
}

QWaylandOutput *WaylandCompositor::outputAt(const QPoint &pos) const
{
    for (QWaylandOutput *output : m_outputs) {
        QRect geometry = output->geometry();
        if (geometry.contains(pos)) {
            return output;
        }
    }
    
    // If no output contains the position, return the primary output
    return m_primaryOutput;
}

QWaylandOutput *WaylandCompositor::outputForSurface(QWaylandSurface *surface) const
{
    if (!surface) {
        return nullptr;
    }
    
    // Get the surface geometry
    QPoint surfacePos = surface->client()->positionForOutput(surface, m_primaryOutput);
    QSize surfaceSize = surface->size();
    QRect surfaceGeometry(surfacePos, surfaceSize);
    
    // Find the output with the largest intersection
    QWaylandOutput *bestOutput = nullptr;
    int largestArea = 0;
    
    for (QWaylandOutput *output : m_outputs) {
        QRect outputGeometry = output->geometry();
        QRect intersection = outputGeometry.intersected(surfaceGeometry);
        
        if (!intersection.isEmpty()) {
            int area = intersection.width() * intersection.height();
            if (area > largestArea) {
                largestArea = area;
                bestOutput = output;
            }
        }
    }
    
    // If no output contains the surface, return the primary output
    return bestOutput ? bestOutput : m_primaryOutput;
}

RenderEngine *WaylandCompositor::renderEngine() const
{
    return m_renderEngine;
}

void WaylandCompositor::setKeyboardFocus(QWaylandSurface *surface)
{
    if (!m_seat) {
        qWarning() << "Cannot set keyboard focus: no seat available";
        return;
    }
    
    QWaylandSurface *oldFocus = m_seat->keyboardFocus();
    if (oldFocus != surface) {
        m_seat->setKeyboardFocus(surface);
        emit keyboardFocusChanged(surface);
    }
}

QWaylandSurface *WaylandCompositor::keyboardFocus() const
{
    return m_seat ? m_seat->keyboardFocus() : nullptr;
}

void WaylandCompositor::sendKeyEvent(bool pressed, int keyCode, int modifiers)
{
    if (!m_seat || !m_seat->keyboard()) {
        qWarning() << "Cannot send key event: no keyboard available";
        return;
    }
    
    QWaylandSurface *focus = m_seat->keyboardFocus();
    if (!focus) {
        qWarning() << "Cannot send key event: no surface has keyboard focus";
        return;
    }
    
    if (pressed) {
        m_seat->keyboard()->sendKeyPressEvent(keyCode);
    } else {
        m_seat->keyboard()->sendKeyReleaseEvent(keyCode);
    }
    
    emit keyEvent(focus, pressed, keyCode, modifiers);
}

void WaylandCompositor::sendPointerEvent(const QPointF &pos, bool pressed, int button, int modifiers)
{
    if (!m_seat || !m_seat->pointer()) {
        qWarning() << "Cannot send pointer event: no pointer available";
        return;
    }
    
    // Find the surface at the position
    QWaylandSurface *targetSurface = nullptr;
    QPointF localPos;
    
    for (QWaylandSurface *surface : m_surfaces) {
        QPoint surfacePos = surface->client()->positionForOutput(surface, m_primaryOutput);
        QRectF surfaceGeometry(surfacePos, surface->size());
        
        if (surfaceGeometry.contains(pos)) {
            targetSurface = surface;
            localPos = pos - surfacePos;
            break;
        }
    }
    
    if (!targetSurface) {
        return;
    }
    
    // Set pointer focus
    m_seat->pointer()->setFocus(targetSurface, localPos);
    
    // Send button event if needed
    if (pressed) {
        m_seat->pointer()->sendButtonPressEvent(button);
    } else {
        m_seat->pointer()->sendButtonReleaseEvent(button);
    }
    
    emit pointerEvent(targetSurface, localPos, pressed, button, modifiers);
}

void WaylandCompositor::sendTouchEvent(const QPointF &pos, int id, int state)
{
    if (!m_seat || !m_seat->touch()) {
        qWarning() << "Cannot send touch event: no touch device available";
        return;
    }
    
    // Find the surface at the position
    QWaylandSurface *targetSurface = nullptr;
    QPointF localPos;
    
    for (QWaylandSurface *surface : m_surfaces) {
        QPoint surfacePos = surface->client()->positionForOutput(surface, m_primaryOutput);
        QRectF surfaceGeometry(surfacePos, surface->size());
        
        if (surfaceGeometry.contains(pos)) {
            targetSurface = surface;
            localPos = pos - surfacePos;
            break;
        }
    }
    
    if (!targetSurface) {
        return;
    }
    
    // Send touch event
    switch (state) {
        case 0: // pressed
            m_seat->touch()->sendTouchPointPressed(id, localPos);
            break;
        case 1: // moved
            m_seat->touch()->sendTouchPointMoved(id, localPos);
            break;
        case 2: // released
            m_seat->touch()->sendTouchPointReleased(id, localPos);
            break;
    }
    
    emit touchEvent(targetSurface, localPos, id, state);
}

void WaylandCompositor::closeAllSurfaces()
{
    for (QWaylandSurface *surface : m_surfaces) {
        surface->client()->close();
    }
    
    m_surfaces.clear();
}

QVector<QWaylandSurface *> WaylandCompositor::surfaces() const
{
    return m_surfaces;
}

void WaylandCompositor::connectSignals()
{
    // Connect signals from the compositor
    connect(m_compositor, &QWaylandCompositor::surfaceCreated,
            this, &WaylandCompositor::handleSurfaceCreated);
    connect(m_compositor, &QWaylandCompositor::surfaceAboutToBeDestroyed,
            this, &WaylandCompositor::handleSurfaceDestroyed);
    
    // Connect signals from the XDG shell
    connect(m_xdgShell, &QWaylandXdgShell::toplevelCreated,
            this, &WaylandCompositor::handleXdgToplevelCreated);
    connect(m_xdgShell, &QWaylandXdgShell::popupCreated,
            this, &WaylandCompositor::handleXdgPopupCreated);
}

void WaylandCompositor::handleXdgToplevelCreated(QWaylandXdgToplevel *toplevel, QWaylandXdgSurface *xdgSurface)
{
    emit xdgToplevelCreated(toplevel, xdgSurface->surface());
}

void WaylandCompositor::handleXdgPopupCreated(QWaylandXdgPopup *popup, QWaylandXdgSurface *xdgSurface)
{
    emit xdgPopupCreated(popup, xdgSurface->surface());
}

void WaylandCompositor::handleSurfaceCreated(QWaylandSurface *surface)
{
    if (!surface) {
        return;
    }
    
    m_surfaces.append(surface);
    emit surfaceCreated(surface);
}

void WaylandCompositor::handleSurfaceDestroyed(QWaylandSurface *surface)
{
    if (!surface) {
        return;
    }
    
    m_surfaces.removeOne(surface);
    emit surfaceAboutToBeDestroyed(surface);
}

} // namespace VivoX::Compositor
