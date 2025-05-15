#pragma once

#include <QObject>
#include <QWaylandCompositor>
#include <QWaylandSurface>
#include <QWaylandOutput>
#include <QWaylandXdgShell>
#include <QWaylandSeat>
#include <QVector>
#include <memory>

namespace VivoX::Compositor {

class WaylandProtocols;
class RenderEngine;

/**
 * @brief The WaylandCompositor class is the core of the Wayland compositor implementation.
 * 
 * It manages the central Wayland objects: the QWaylandCompositor instance, the QWaylandSeat,
 * the QWaylandOutput objects, and the basic lifecycle of Wayland surfaces.
 * 
 * Features:
 * - Multiple output support
 * - XDG shell protocol implementation
 * - Wayland protocol extensions
 * - Integration with the rendering engine
 * - Input device management
 */
class WaylandCompositor : public QObject {
    Q_OBJECT

public:
    explicit WaylandCompositor(QObject *parent = nullptr);
    ~WaylandCompositor();

    /**
     * @brief Initialize the Wayland compositor
     * @param renderEngine The rendering engine to use
     * @param socketName Optional socket name (defaults to "vivox")
     * @return True if initialization was successful
     */
    bool initialize(RenderEngine *renderEngine, const QString &socketName = "vivox");

    /**
     * @brief Get the underlying QWaylandCompositor instance
     * @return The QWaylandCompositor instance
     */
    QWaylandCompositor *compositor() const;

    /**
     * @brief Get the Wayland seat
     * @return The QWaylandSeat instance
     */
    QWaylandSeat *seat() const;

    /**
     * @brief Get the list of outputs
     * @return The list of QWaylandOutput instances
     */
    QVector<QWaylandOutput *> outputs() const;

    /**
     * @brief Get the XDG shell
     * @return The QWaylandXdgShell instance
     */
    QWaylandXdgShell *xdgShell() const;

    /**
     * @brief Get the Wayland protocols manager
     * @return The WaylandProtocols instance
     */
    WaylandProtocols *protocols() const;

    /**
     * @brief Add an output to the compositor
     * @param output The output to add
     */
    void addOutput(QWaylandOutput *output);

    /**
     * @brief Remove an output from the compositor
     * @param output The output to remove
     */
    void removeOutput(QWaylandOutput *output);
    
    /**
     * @brief Get the primary output
     * @return The primary QWaylandOutput instance, or nullptr if none
     */
    QWaylandOutput *primaryOutput() const;
    
    /**
     * @brief Set the primary output
     * @param output The output to set as primary
     */
    void setPrimaryOutput(QWaylandOutput *output);
    
    /**
     * @brief Get the output at the given position
     * @param pos The position to check
     * @return The QWaylandOutput at the position, or nullptr if none
     */
    QWaylandOutput *outputAt(const QPoint &pos) const;
    
    /**
     * @brief Get the output containing the given surface
     * @param surface The surface to check
     * @return The QWaylandOutput containing the surface, or nullptr if none
     */
    QWaylandOutput *outputForSurface(QWaylandSurface *surface) const;
    
    /**
     * @brief Get the rendering engine
     * @return The RenderEngine instance
     */
    RenderEngine *renderEngine() const;
    
    /**
     * @brief Set keyboard focus to the given surface
     * @param surface The surface to focus
     */
    void setKeyboardFocus(QWaylandSurface *surface);
    
    /**
     * @brief Get the surface with keyboard focus
     * @return The QWaylandSurface with keyboard focus, or nullptr if none
     */
    QWaylandSurface *keyboardFocus() const;
    
    /**
     * @brief Send a key event to the focused surface
     * @param pressed True if the key is pressed, false if released
     * @param keyCode The key code
     * @param modifiers The keyboard modifiers
     */
    void sendKeyEvent(bool pressed, int keyCode, int modifiers);
    
    /**
     * @brief Send a pointer event to the surface at the given position
     * @param pos The position of the pointer
     * @param pressed True if a button is pressed, false if released
     * @param button The button (if any)
     * @param modifiers The keyboard modifiers
     */
    void sendPointerEvent(const QPointF &pos, bool pressed, int button, int modifiers);
    
    /**
     * @brief Send a touch event to the surface at the given position
     * @param pos The position of the touch
     * @param id The touch point ID
     * @param state The touch state (pressed, moved, released)
     */
    void sendTouchEvent(const QPointF &pos, int id, int state);
    
    /**
     * @brief Close all surfaces
     */
    void closeAllSurfaces();
    
    /**
     * @brief Get all active surfaces
     * @return List of all active surfaces
     */
    QVector<QWaylandSurface *> surfaces() const;

signals:
    /**
     * @brief Signal emitted when a new surface is created
     * @param surface The new surface
     */
    void surfaceCreated(QWaylandSurface *surface);

    /**
     * @brief Signal emitted when a surface is about to be destroyed
     * @param surface The surface being destroyed
     */
    void surfaceAboutToBeDestroyed(QWaylandSurface *surface);

    /**
     * @brief Signal emitted when an XDG toplevel surface is created
     * @param toplevel The new toplevel surface
     * @param surface The underlying Wayland surface
     */
    void xdgToplevelCreated(QWaylandXdgToplevel *toplevel, QWaylandSurface *surface);

    /**
     * @brief Signal emitted when an XDG popup surface is created
     * @param popup The new popup surface
     * @param surface The underlying Wayland surface
     */
    void xdgPopupCreated(QWaylandXdgPopup *popup, QWaylandSurface *surface);
    
    /**
     * @brief Signal emitted when the primary output changes
     * @param output The new primary output
     */
    void primaryOutputChanged(QWaylandOutput *output);
    
    /**
     * @brief Signal emitted when the keyboard focus changes
     * @param surface The new focused surface
     */
    void keyboardFocusChanged(QWaylandSurface *surface);
    
    /**
     * @brief Signal emitted when a key event is sent
     * @param surface The target surface
     * @param pressed True if the key is pressed, false if released
     * @param keyCode The key code
     * @param modifiers The keyboard modifiers
     */
    void keyEvent(QWaylandSurface *surface, bool pressed, int keyCode, int modifiers);
    
    /**
     * @brief Signal emitted when a pointer event is sent
     * @param surface The target surface
     * @param pos The position of the pointer
     * @param pressed True if a button is pressed, false if released
     * @param button The button (if any)
     * @param modifiers The keyboard modifiers
     */
    void pointerEvent(QWaylandSurface *surface, const QPointF &pos, bool pressed, int button, int modifiers);
    
    /**
     * @brief Signal emitted when a touch event is sent
     * @param surface The target surface
     * @param pos The position of the touch
     * @param id The touch point ID
     * @param state The touch state (pressed, moved, released)
     */
    void touchEvent(QWaylandSurface *surface, const QPointF &pos, int id, int state);

private:
    // The underlying QWaylandCompositor instance
    QWaylandCompositor *m_compositor;
    
    // The Wayland seat
    QWaylandSeat *m_seat;
    
    // The XDG shell
    QWaylandXdgShell *m_xdgShell;
    
    // The Wayland protocols manager
    WaylandProtocols *m_protocols;
    
    // The rendering engine
    RenderEngine *m_renderEngine;
    
    // List of outputs
    QVector<QWaylandOutput *> m_outputs;
    
    // Primary output
    QWaylandOutput *m_primaryOutput;
    
    // List of active surfaces
    QVector<QWaylandSurface *> m_surfaces;
    
    // Connect signals from the compositor
    void connectSignals();
    
    // Handle XDG shell events
    void handleXdgToplevelCreated(QWaylandXdgToplevel *toplevel, QWaylandXdgSurface *xdgSurface);
    void handleXdgPopupCreated(QWaylandXdgPopup *popup, QWaylandXdgSurface *xdgSurface);
    
    // Handle surface events
    void handleSurfaceCreated(QWaylandSurface *surface);
    void handleSurfaceDestroyed(QWaylandSurface *surface);
};

} // namespace VivoX::Compositor
