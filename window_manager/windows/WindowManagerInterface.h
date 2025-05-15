#pragma once

#include <QObject>
#include <QString>
#include <QList>
#include <QRect>

namespace VivoX {
namespace WindowManager {

class Window;
class Workspace;

/**
 * @brief Interface for the window manager
 * 
 * This interface defines the functionality that the window manager must implement.
 * It is responsible for managing windows, workspaces, and layouts.
 */
class WindowManagerInterface {
public:
    virtual ~WindowManagerInterface() = default;

    /**
     * @brief Initialize the window manager
     * 
     * @return bool True if initialization was successful, false otherwise
     */
    virtual bool initialize() = 0;

    /**
     * @brief Add a window to the window manager
     * 
     * @param window The window to add
     */
    virtual void addWindow(Window* window) = 0;

    /**
     * @brief Remove a window from the window manager
     * 
     * @param window The window to remove
     */
    virtual void removeWindow(Window* window) = 0;

    /**
     * @brief Get all windows managed by the window manager
     * 
     * @return QList<Window*> The list of windows
     */
    virtual QList<Window*> windows() const = 0;

    /**
     * @brief Get the active window
     * 
     * @return Window* The active window
     */
    virtual Window* activeWindow() const = 0;

    /**
     * @brief Set the active window
     * 
     * @param window The window to activate
     */
    virtual void setActiveWindow(Window* window) = 0;

    /**
     * @brief Get the window at the specified position
     * 
     * @param pos The position
     * @return Window* The window at the position, or nullptr if none
     */
    virtual Window* windowAt(const QPoint& pos) const = 0;

    /**
     * @brief Move a window to a new position
     * 
     * @param window The window to move
     * @param pos The new position
     */
    virtual void moveWindow(Window* window, const QPoint& pos) = 0;

    /**
     * @brief Resize a window to a new size
     * 
     * @param window The window to resize
     * @param size The new size
     */
    virtual void resizeWindow(Window* window, const QSize& size) = 0;

    /**
     * @brief Set the geometry of a window
     * 
     * @param window The window
     * @param geometry The new geometry
     */
    virtual void setWindowGeometry(Window* window, const QRect& geometry) = 0;

    /**
     * @brief Maximize a window
     * 
     * @param window The window to maximize
     * @param maximized Whether the window should be maximized
     */
    virtual void maximizeWindow(Window* window, bool maximized) = 0;

    /**
     * @brief Minimize a window
     * 
     * @param window The window to minimize
     * @param minimized Whether the window should be minimized
     */
    virtual void minimizeWindow(Window* window, bool minimized) = 0;

    /**
     * @brief Make a window fullscreen
     * 
     * @param window The window to make fullscreen
     * @param fullscreen Whether the window should be fullscreen
     */
    virtual void fullscreenWindow(Window* window, bool fullscreen) = 0;

    /**
     * @brief Close a window
     * 
     * @param window The window to close
     */
    virtual void closeWindow(Window* window) = 0;

    /**
     * @brief Add a workspace to the window manager
     * 
     * @param workspace The workspace to add
     */
    virtual void addWorkspace(Workspace* workspace) = 0;

    /**
     * @brief Remove a workspace from the window manager
     * 
     * @param workspace The workspace to remove
     */
    virtual void removeWorkspace(Workspace* workspace) = 0;

    /**
     * @brief Get all workspaces managed by the window manager
     * 
     * @return QList<Workspace*> The list of workspaces
     */
    virtual QList<Workspace*> workspaces() const = 0;

    /**
     * @brief Get the active workspace
     * 
     * @return Workspace* The active workspace
     */
    virtual Workspace* activeWorkspace() const = 0;

    /**
     * @brief Set the active workspace
     * 
     * @param workspace The workspace to activate
     */
    virtual void setActiveWorkspace(Workspace* workspace) = 0;

    /**
     * @brief Move a window to a workspace
     * 
     * @param window The window to move
     * @param workspace The target workspace
     */
    virtual void moveWindowToWorkspace(Window* window, Workspace* workspace) = 0;

    /**
     * @brief Set the layout for a workspace
     * 
     * @param workspace The workspace
     * @param layoutName The name of the layout to use
     */
    virtual void setWorkspaceLayout(Workspace* workspace, const QString& layoutName) = 0;

    /**
     * @brief Apply the current layout to a workspace
     * 
     * @param workspace The workspace to apply the layout to
     */
    virtual void applyLayout(Workspace* workspace) = 0;
};

} // namespace WindowManager
} // namespace VivoX
