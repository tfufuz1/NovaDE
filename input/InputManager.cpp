#include "InputManager.h"
#include "shortcuts/ShortcutManager.h"
#include "gestures/GestureEngine.h"

#include <QDebug>

namespace VivoX::Input {

InputManager::InputManager(QObject *parent)
    : QObject(parent)
    , m_seat(nullptr)
    , m_keyboard(nullptr)
    , m_pointer(nullptr)
    , m_touch(nullptr)
    , m_shortcutManager(nullptr)
    , m_gestureEngine(nullptr)
{
    qDebug() << "InputManager created";
}

InputManager::~InputManager()
{
    qDebug() << "InputManager destroyed";
}

void InputManager::initialize(QWaylandSeat *seat)
{
    if (!seat) {
        qWarning() << "Cannot initialize InputManager with null seat";
        return;
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
                // Handle modifier changes if needed
                Q_UNUSED(depressed);
                Q_UNUSED(latched);
                Q_UNUSED(locked);
                Q_UNUSED(group);
            });
            
        connect(m_keyboard, &QWaylandKeyboard::keymapChanged, this,
            [this](const QString &keymap) {
                // Handle keymap changes if needed
                Q_UNUSED(keymap);
            });
    }
    
    // Connect to pointer capability
    if (m_seat->pointer()) {
        m_pointer = m_seat->pointer();
        
        // Connect pointer signals
        connect(m_pointer, &QWaylandPointer::buttonPressed, this,
            [this](quint32 button, const QPointF &position) {
                handlePointerEvent(position.toPoint(), button, true);
            });
            
        connect(m_pointer, &QWaylandPointer::buttonReleased, this,
            [this](quint32 button, const QPointF &position) {
                handlePointerEvent(position.toPoint(), button, false);
            });
            
        connect(m_pointer, &QWaylandPointer::motion, this,
            [this](const QPointF &position) {
                // Handle pointer motion
                Q_UNUSED(position);
                // Forward to gesture engine if registered
                if (m_gestureEngine) {
                    m_gestureEngine->handlePointerMotion(position.toPoint());
                }
            });
            
        connect(m_pointer, &QWaylandPointer::axis, this,
            [this](Qt::Orientation orientation, qreal delta) {
                handleScrollEvent(delta, orientation == Qt::Horizontal ? 1 : 2);
            });
    }
    
    // Connect to touch capability
    if (m_seat->touch()) {
        m_touch = m_seat->touch();
        
        // Connect touch signals
        connect(m_touch, &QWaylandTouch::touchPointPressed, this,
            [this](int id, const QPointF &position) {
                handleTouchEvent(id, position.toPoint(), true);
            });
            
        connect(m_touch, &QWaylandTouch::touchPointReleased, this,
            [this](int id, const QPointF &position) {
                handleTouchEvent(id, position.toPoint(), false);
            });
            
        connect(m_touch, &QWaylandTouch::touchPointMoved, this,
            [this](int id, const QPointF &position) {
                // Handle touch point movement
                Q_UNUSED(id);
                Q_UNUSED(position);
                // Forward to gesture engine if registered
                if (m_gestureEngine) {
                    m_gestureEngine->handleTouchMotion(id, position.toPoint());
                }
            });
            
        connect(m_touch, &QWaylandTouch::touchFrameFinished, this,
            [this]() {
                // Handle touch frame finished
                if (m_gestureEngine) {
                    m_gestureEngine->handleTouchFrame();
                }
            });
    }
    
    qDebug() << "InputManager initialized with Wayland seat";
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

void InputManager::handleKeyboardEvent(quint32 keyCode, bool pressed, quint32 modifiers)
{
    // First, check if the shortcut manager wants to handle this event
    bool handled = false;
    
    if (m_shortcutManager) {
        handled = m_shortcutManager->handleKeyEvent(keyCode, pressed, modifiers);
    }
    
    // If not handled by shortcut manager, forward to client
    if (!handled) {
        emit keyboardEventForClient(keyCode, pressed, modifiers);
    }
}

void InputManager::handlePointerEvent(const QPoint &position, quint32 button, bool pressed)
{
    // First, check if the gesture engine wants to handle this event
    bool handled = false;
    
    if (m_gestureEngine) {
        handled = m_gestureEngine->handlePointerButton(position, button, pressed);
    }
    
    // If not handled by gesture engine, forward to client
    if (!handled) {
        emit pointerEventForClient(position, button, pressed);
    }
}

void InputManager::handleTouchEvent(qint32 id, const QPoint &position, bool pressed)
{
    // First, check if the gesture engine wants to handle this event
    bool handled = false;
    
    if (m_gestureEngine) {
        handled = m_gestureEngine->handleTouchPoint(id, position, pressed);
    }
    
    // If not handled by gesture engine, forward to client
    if (!handled) {
        emit touchEventForClient(id, position, pressed);
    }
}

void InputManager::handleScrollEvent(qreal delta, quint32 orientation)
{
    // First, check if the gesture engine wants to handle this event
    bool handled = false;
    
    if (m_gestureEngine) {
        handled = m_gestureEngine->handleScroll(delta, orientation);
    }
    
    // If not handled by gesture engine, forward to client
    if (!handled) {
        emit scrollEventForClient(delta, orientation);
    }
}

} // namespace VivoX::Input
