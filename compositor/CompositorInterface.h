#pragma once

#include <QObject>
#include <QString>
#include <QRect>
#include <QWaylandSurface>
#include <QWaylandXdgToplevel>

namespace VivoX {
namespace Compositor {

/**
 * @brief Interface for the Wayland compositor
 * 
 * This interface defines the functionality that the Wayland compositor must implement.
 * It is responsible for managing surfaces, handling input, and rendering the desktop.
 */
class CompositorInterface {
public:
    virtual ~CompositorInterface() = default;

    /**
     * @brief Initialize the compositor
     * 
     * @param renderEngine The rendering engine to use
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Get the Wayland display
     * 
     * @return QWaylandCompositor* The Wayland compositor instance
     */
    virtual QObject* compositor() const = 0;

    /**
     * @brief Create a new surface
     * 
     * @return QWaylandSurface* The created surface
     */
    virtual QWaylandSurface* createSurface() = 0;

    /**
     * @brief Handle a new XDG toplevel surface
     * 
     * @param toplevel The XDG toplevel surface
     * @param surface The Wayland surface
     */
    virtual void handleXdgToplevel(QWaylandXdgToplevel* toplevel, QWaylandSurface* surface) = 0;

    /**
     * @brief Get the current active surface
     * 
     * @return QWaylandSurface* The active surface
     */
    virtual QWaylandSurface* activeSurface() const = 0;

    /**
     * @brief Set the active surface
     * 
     * @param surface The surface to activate
     */
    virtual void setActiveSurface(QWaylandSurface* surface) = 0;

    /**
     * @brief Get the geometry of a surface
     * 
     * @param surface The surface
     * @return QRect The geometry of the surface
     */
    virtual QRect surfaceGeometry(QWaylandSurface* surface) const = 0;

    /**
     * @brief Set the geometry of a surface
     * 
     * @param surface The surface
     * @param geometry The new geometry
     */
    virtual void setSurfaceGeometry(QWaylandSurface* surface, const QRect& geometry) = 0;

    /**
     * @brief Raise a surface to the top
     * 
     * @param surface The surface to raise
     */
    virtual void raiseSurface(QWaylandSurface* surface) = 0;

    /**
     * @brief Lower a surface to the bottom
     * 
     * @param surface The surface to lower
     */
    virtual void lowerSurface(QWaylandSurface* surface) = 0;

    /**
     * @brief Close a surface
     * 
     * @param surface The surface to close
     */
    virtual void closeSurface(QWaylandSurface* surface) = 0;

    /**
     * @brief Maximize a surface
     * 
     * @param surface The surface to maximize
     * @param maximized Whether the surface should be maximized
     */
    virtual void maximizeSurface(QWaylandSurface* surface, bool maximized) = 0;

    /**
     * @brief Minimize a surface
     * 
     * @param surface The surface to minimize
     * @param minimized Whether the surface should be minimized
     */
    virtual void minimizeSurface(QWaylandSurface* surface, bool minimized) = 0;

    /**
     * @brief Make a surface fullscreen
     * 
     * @param surface The surface to make fullscreen
     * @param fullscreen Whether the surface should be fullscreen
     */
    virtual void fullscreenSurface(QWaylandSurface* surface, bool fullscreen) = 0;
};

} // namespace Compositor
} // namespace VivoX
