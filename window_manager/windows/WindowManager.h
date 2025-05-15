#pragma once

#include <memory>
#include <vector>
#include <string>
#include <map>
#include <functional>
#include <mutex>

namespace VivoX {
namespace WindowManager {
namespace Windows {

class Window;
class WindowGroup;
class WindowLayout;
class WindowWorkspace;

/**
 * @brief Window state enumeration
 */
enum class WindowState {
    Normal,
    Maximized,
    Minimized,
    Fullscreen,
    Tiled,
    Snapped
};

/**
 * @brief Window type enumeration
 */
enum class WindowType {
    Normal,
    Dialog,
    Popup,
    Utility,
    Splash,
    Notification,
    Dock,
    Desktop,
    Menu
};

/**
 * @brief Window snap position enumeration
 */
enum class SnapPosition {
    None,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center
};

/**
 * @brief Window Manager for the VivoX Desktop Environment
 * 
 * This class is responsible for managing windows in the VivoX Desktop Environment.
 * It handles window creation, destruction, positioning, sizing, and stacking.
 * 
 * Features:
 * - Window lifecycle management
 * - Window state management (normal, maximized, minimized, fullscreen)
 * - Window positioning and sizing
 * - Window stacking order
 * - Window grouping
 * - Window tiling and snapping
 * - Window workspaces
 * - Window focus management
 * - Window event handling
 */
class WindowManager {
public:
    /**
     * @brief Get the singleton instance of the WindowManager
     * @return Shared pointer to the WindowManager instance
     */
    static std::shared_ptr<WindowManager> getInstance();
    
    /**
     * @brief Initialize the window manager
     * @return True if initialization was successful, false otherwise
     */
    bool initialize();
    
    /**
     * @brief Shutdown the window manager
     */
    void shutdown();
    
    /**
     * @brief Check if the window manager is initialized
     * @return True if the window manager is initialized, false otherwise
     */
    bool isInitialized() const;
    
    /**
     * @brief Create a new window
     * @param title Window title
     * @param x X position
     * @param y Y position
     * @param width Window width
     * @param height Window height
     * @param type Window type
     * @return Shared pointer to the created window, or nullptr if creation failed
     */
    std::shared_ptr<Window> createWindow(const std::string& title, int x, int y, int width, int height, WindowType type = WindowType::Normal);
    
    /**
     * @brief Destroy a window
     * @param window Window to destroy
     * @return True if the operation was successful, false otherwise
     */
    bool destroyWindow(std::shared_ptr<Window> window);
    
    /**
     * @brief Get all windows
     * @return Vector of all windows
     */
    std::vector<std::shared_ptr<Window>> getWindows() const;
    
    /**
     * @brief Get a window by ID
     * @param id Window ID
     * @return Shared pointer to the window, or nullptr if it wasn't found
     */
    std::shared_ptr<Window> getWindow(uint32_t id) const;
    
    /**
     * @brief Get windows by type
     * @param type Window type
     * @return Vector of windows of the specified type
     */
    std::vector<std::shared_ptr<Window>> getWindowsByType(WindowType type) const;
    
    /**
     * @brief Get windows by workspace
     * @param workspaceId Workspace ID
     * @return Vector of windows in the specified workspace
     */
    std::vector<std::shared_ptr<Window>> getWindowsByWorkspace(uint32_t workspaceId) const;
    
    /**
     * @brief Get the active window
     * @return Shared pointer to the active window, or nullptr if no window is active
     */
    std::shared_ptr<Window> getActiveWindow() const;
    
    /**
     * @brief Activate a window
     * @param window Window to activate
     * @return True if the operation was successful, false otherwise
     */
    bool activateWindow(std::shared_ptr<Window> window);
    
    /**
     * @brief Maximize a window
     * @param window Window to maximize
     * @return True if the operation was successful, false otherwise
     */
    bool maximizeWindow(std::shared_ptr<Window> window);
    
    /**
     * @brief Minimize a window
     * @param window Window to minimize
     * @return True if the operation was successful, false otherwise
     */
    bool minimizeWindow(std::shared_ptr<Window> window);
    
    /**
     * @brief Restore a window
     * @param window Window to restore
     * @return True if the operation was successful, false otherwise
     */
    bool restoreWindow(std::shared_ptr<Window> window);
    
    /**
     * @brief Make a window fullscreen
     * @param window Window to make fullscreen
     * @param fullscreen Whether the window should be fullscreen
     * @return True if the operation was successful, false otherwise
     */
    bool setFullscreen(std::shared_ptr<Window> window, bool fullscreen);
    
    /**
     * @brief Move a window
     * @param window Window to move
     * @param x New X position
     * @param y New Y position
     * @return True if the operation was successful, false otherwise
     */
    bool moveWindow(std::shared_ptr<Window> window, int x, int y);
    
    /**
     * @brief Resize a window
     * @param window Window to resize
     * @param width New width
     * @param height New height
     * @return True if the operation was successful, false otherwise
     */
    bool resizeWindow(std::shared_ptr<Window> window, int width, int height);
    
    /**
     * @brief Raise a window to the top of the stack
     * @param window Window to raise
     * @return True if the operation was successful, false otherwise
     */
    bool raiseWindow(std::shared_ptr<Window> window);
    
    /**
     * @brief Lower a window to the bottom of the stack
     * @param window Window to lower
     * @return True if the operation was successful, false otherwise
     */
    bool lowerWindow(std::shared_ptr<Window> window);
    
    /**
     * @brief Snap a window to a predefined position
     * @param window Window to snap
     * @param position Snap position
     * @return True if the operation was successful, false otherwise
     */
    bool snapWindow(std::shared_ptr<Window> window, SnapPosition position);
    
    /**
     * @brief Tile windows according to a layout
     * @param layout Layout name (grid, horizontal, vertical, etc.)
     * @param workspaceId Workspace ID (0 for current workspace)
     * @return True if the operation was successful, false otherwise
     */
    bool tileWindows(const std::string& layout, uint32_t workspaceId = 0);
    
    /**
     * @brief Create a window group
     * @param name Group name
     * @param windows Windows to add to the group
     * @return Shared pointer to the created group, or nullptr if creation failed
     */
    std::shared_ptr<WindowGroup> createWindowGroup(const std::string& name, const std::vector<std::shared_ptr<Window>>& windows = {});
    
    /**
     * @brief Destroy a window group
     * @param group Group to destroy
     * @return True if the operation was successful, false otherwise
     */
    bool destroyWindowGroup(std::shared_ptr<WindowGroup> group);
    
    /**
     * @brief Get all window groups
     * @return Vector of all window groups
     */
    std::vector<std::shared_ptr<WindowGroup>> getWindowGroups() const;
    
    /**
     * @brief Get a window group by ID
     * @param id Group ID
     * @return Shared pointer to the group, or nullptr if it wasn't found
     */
    std::shared_ptr<WindowGroup> getWindowGroup(uint32_t id) const;
    
    /**
     * @brief Get a window group by name
     * @param name Group name
     * @return Shared pointer to the group, or nullptr if it wasn't found
     */
    std::shared_ptr<WindowGroup> getWindowGroupByName(const std::string& name) const;
    
    /**
     * @brief Create a workspace
     * @param name Workspace name
     * @return Shared pointer to the created workspace, or nullptr if creation failed
     */
    std::shared_ptr<WindowWorkspace> createWorkspace(const std::string& name);
    
    /**
     * @brief Destroy a workspace
     * @param workspace Workspace to destroy
     * @return True if the operation was successful, false otherwise
     */
    bool destroyWorkspace(std::shared_ptr<WindowWorkspace> workspace);
    
    /**
     * @brief Get all workspaces
     * @return Vector of all workspaces
     */
    std::vector<std::shared_ptr<WindowWorkspace>> getWorkspaces() const;
    
    /**
     * @brief Get a workspace by ID
     * @param id Workspace ID
     * @return Shared pointer to the workspace, or nullptr if it wasn't found
     */
    std::shared_ptr<WindowWorkspace> getWorkspace(uint32_t id) const;
    
    /**
     * @brief Get a workspace by name
     * @param name Workspace name
     * @return Shared pointer to the workspace, or nullptr if it wasn't found
     */
    std::shared_ptr<WindowWorkspace> getWorkspaceByName(const std::string& name) const;
    
    /**
     * @brief Get the current workspace
     * @return Shared pointer to the current workspace, or nullptr if no workspace is active
     */
    std::shared_ptr<WindowWorkspace> getCurrentWorkspace() const;
    
    /**
     * @brief Switch to a workspace
     * @param workspace Workspace to switch to
     * @return True if the operation was successful, false otherwise
     */
    bool switchToWorkspace(std::shared_ptr<WindowWorkspace> workspace);
    
    /**
     * @brief Move a window to a workspace
     * @param window Window to move
     * @param workspace Target workspace
     * @return True if the operation was successful, false otherwise
     */
    bool moveWindowToWorkspace(std::shared_ptr<Window> window, std::shared_ptr<WindowWorkspace> workspace);
    
    /**
     * @brief Register a callback for window events
     * @param callback Function to call when a window event occurs
     * @return ID that can be used to unregister the callback
     */
    int registerWindowEventCallback(std::function<void(const std::string&, std::shared_ptr<Window>)> callback);
    
    /**
     * @brief Unregister a window event callback
     * @param callbackId ID returned by registerWindowEventCallback
     * @return True if the callback was unregistered, false if the ID was invalid
     */
    bool unregisterWindowEventCallback(int callbackId);
    
    /**
     * @brief Get available window layouts
     * @return Vector of available layout names
     */
    std::vector<std::string> getAvailableLayouts() const;
    
    /**
     * @brief Get the current window layout
     * @return Name of the current layout
     */
    std::string getCurrentLayout() const;
    
    /**
     * @brief Set the current window layout
     * @param layout Layout name
     * @return True if the operation was successful, false otherwise
     */
    bool setCurrentLayout(const std::string& layout);
    
    /**
     * @brief Save the current window layout
     * @param name Layout name
     * @return True if the operation was successful, false otherwise
     */
    bool saveLayout(const std::string& name);
    
    /**
     * @brief Load a saved window layout
     * @param name Layout name
     * @return True if the operation was successful, false otherwise
     */
    bool loadLayout(const std::string& name);
    
    /**
     * @brief Delete a saved window layout
     * @param name Layout name
     * @return True if the operation was successful, false otherwise
     */
    bool deleteLayout(const std::string& name);

private:
    WindowManager();
    ~WindowManager();
    
    static std::shared_ptr<WindowManager> s_instance;
    static std::mutex s_instanceMutex;
    
    class Impl;
    std::unique_ptr<Impl> m_impl;
    
    // Window event callback management
    std::map<int, std::function<void(const std::string&, std::shared_ptr<Window>)>> m_windowEventCallbacks;
    int m_nextCallbackId;
    mutable std::mutex m_callbackMutex;
};

} // namespace Windows
} // namespace WindowManager
} // namespace VivoX
