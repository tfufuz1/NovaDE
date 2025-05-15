#pragma once

#include <memory>
#include <vector>
#include <string>
#include <map>

namespace VivoX {
namespace Compositor {
namespace XWayland {

class WaylandCompositor;
class XWaylandSurface;

/**
 * @brief XWayland integration for the VivoX Desktop Environment
 * 
 * This class is responsible for integrating XWayland with the Wayland compositor
 * to support legacy X11 applications.
 */
class XWaylandIntegration {
public:
    /**
     * @brief Get the singleton instance of the XWaylandIntegration
     * @return Shared pointer to the XWaylandIntegration instance
     */
    static std::shared_ptr<XWaylandIntegration> getInstance();
    
    /**
     * @brief Initialize the XWayland integration
     * @param compositor Wayland compositor instance
     * @return True if initialization was successful, false otherwise
     */
    bool initialize(std::shared_ptr<WaylandCompositor> compositor);
    
    /**
     * @brief Shutdown the XWayland integration
     */
    void shutdown();
    
    /**
     * @brief Check if the XWayland integration is initialized
     * @return True if the XWayland integration is initialized, false otherwise
     */
    bool isInitialized() const;
    
    /**
     * @brief Check if XWayland is running
     * @return True if XWayland is running, false otherwise
     */
    bool isRunning() const;
    
    /**
     * @brief Get all XWayland surfaces
     * @return Vector of all XWayland surfaces
     */
    std::vector<std::shared_ptr<XWaylandSurface>> getSurfaces() const;
    
    /**
     * @brief Get an XWayland surface by window ID
     * @param windowId X11 window ID
     * @return Shared pointer to the XWayland surface, or nullptr if it wasn't found
     */
    std::shared_ptr<XWaylandSurface> getSurface(uint32_t windowId) const;
    
    /**
     * @brief Map an X11 window to a Wayland surface
     * @param windowId X11 window ID
     * @return True if the mapping was successful, false otherwise
     */
    bool mapWindow(uint32_t windowId);
    
    /**
     * @brief Unmap an X11 window
     * @param windowId X11 window ID
     * @return True if the unmapping was successful, false otherwise
     */
    bool unmapWindow(uint32_t windowId);
    
    /**
     * @brief Set the position of an X11 window
     * @param windowId X11 window ID
     * @param x X position
     * @param y Y position
     * @return True if the operation was successful, false otherwise
     */
    bool setWindowPosition(uint32_t windowId, int x, int y);
    
    /**
     * @brief Set the size of an X11 window
     * @param windowId X11 window ID
     * @param width Window width
     * @param height Window height
     * @return True if the operation was successful, false otherwise
     */
    bool setWindowSize(uint32_t windowId, int width, int height);
    
    /**
     * @brief Focus an X11 window
     * @param windowId X11 window ID
     * @return True if the operation was successful, false otherwise
     */
    bool focusWindow(uint32_t windowId);

private:
    XWaylandIntegration();
    ~XWaylandIntegration();
    
    static std::shared_ptr<XWaylandIntegration> s_instance;
    static std::mutex s_instanceMutex;
    
    class Impl;
    std::unique_ptr<Impl> m_impl;
};

} // namespace XWayland
} // namespace Compositor
} // namespace VivoX
