#include "WindowManager.h"
#include "Window.h"
#include "../workspaces/WindowWorkspace.h"
#include "../groups/WindowGroup.h"
#include "../layouts/WindowLayout.h"
#include "../layouts/LayoutEngine.h"

#include <iostream>
#include <algorithm>
#include <sstream>

namespace VivoX {
namespace WindowManager {
namespace Windows {

// Implementation class
class WindowManager::Impl {
public:
    Impl() 
        : m_initialized(false)
        , m_nextWindowId(1)
        , m_nextGroupId(1)
        , m_nextWorkspaceId(1)
        , m_currentLayout("default")
    {
    }
    
    ~Impl() {
        shutdown();
    }
    
    bool initialize() {
        if (m_initialized) {
            return true;
        }
        
        // Create default workspace
        auto defaultWorkspace = std::make_shared<WindowWorkspace>(m_nextWorkspaceId++, "Default");
        m_workspaces.push_back(defaultWorkspace);
        m_currentWorkspace = defaultWorkspace;
        
        m_initialized = true;
        return true;
    }
    
    void shutdown() {
        if (!m_initialized) {
            return;
        }
        
        // Clear all windows, groups, and workspaces
        m_windows.clear();
        m_windowsById.clear();
        m_groups.clear();
        m_groupsById.clear();
        m_groupsByName.clear();
        m_workspaces.clear();
        m_workspacesById.clear();
        m_workspacesByName.clear();
        m_currentWorkspace = nullptr;
        m_activeWindow = nullptr;
        
        m_initialized = false;
    }
    
    bool isInitialized() const {
        return m_initialized;
    }
    
    std::shared_ptr<Window> createWindow(const std::string& title, int x, int y, int width, int height, WindowType type) {
        if (!m_initialized) {
            std::cerr << "WindowManager not initialized" << std::endl;
            return nullptr;
        }
        
        // Create window
        auto window = std::make_shared<Window>(m_nextWindowId++, title, x, y, width, height, type);
        
        // Add to lists
        m_windows.push_back(window);
        m_windowsById[window->getId()] = window;
        
        // Add to current workspace
        if (m_currentWorkspace) {
            m_currentWorkspace->addWindow(window);
        }
        
        // If this is the first window, activate it
        if (m_windows.size() == 1 && !m_activeWindow) {
            activateWindow(window);
        }
        
        std::cout << "Created window: " << window->getId() << " - " << title << std::endl;
        
        return window;
    }
    
    bool destroyWindow(std::shared_ptr<Window> window) {
        if (!m_initialized || !window) {
            return false;
        }
        
        uint32_t windowId = window->getId();
        
        // Check if window exists
        auto it = m_windowsById.find(windowId);
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << windowId << std::endl;
            return false;
        }
        
        // If this is the active window, clear it
        if (m_activeWindow == window) {
            m_activeWindow = nullptr;
            
            // Activate another window if available
            if (!m_windows.empty()) {
                // Find the next window to activate
                auto it = std::find(m_windows.begin(), m_windows.end(), window);
                if (it != m_windows.end() && it + 1 != m_windows.end()) {
                    // Activate the next window
                    activateWindow(*(it + 1));
                } else if (!m_windows.empty()) {
                    // Activate the first window
                    activateWindow(m_windows.front());
                }
            }
        }
        
        // Remove from workspace
        for (auto& workspace : m_workspaces) {
            workspace->removeWindow(window);
        }
        
        // Remove from groups
        for (auto& group : m_groups) {
            group->removeWindow(window);
        }
        
        // Remove from lists
        m_windows.erase(std::remove(m_windows.begin(), m_windows.end(), window), m_windows.end());
        m_windowsById.erase(windowId);
        
        std::cout << "Destroyed window: " << windowId << std::endl;
        
        return true;
    }
    
    std::vector<std::shared_ptr<Window>> getWindows() const {
        return m_windows;
    }
    
    std::shared_ptr<Window> getWindow(uint32_t id) const {
        auto it = m_windowsById.find(id);
        if (it == m_windowsById.end()) {
            return nullptr;
        }
        
        return it->second;
    }
    
    std::vector<std::shared_ptr<Window>> getWindowsByType(WindowType type) const {
        std::vector<std::shared_ptr<Window>> result;
        
        for (const auto& window : m_windows) {
            if (window->getType() == type) {
                result.push_back(window);
            }
        }
        
        return result;
    }
    
    std::vector<std::shared_ptr<Window>> getWindowsByWorkspace(uint32_t workspaceId) const {
        auto workspace = getWorkspace(workspaceId);
        if (!workspace) {
            return {};
        }
        
        return workspace->getWindows();
    }
    
    std::shared_ptr<Window> getActiveWindow() const {
        return m_activeWindow;
    }
    
    bool activateWindow(std::shared_ptr<Window> window) {
        if (!m_initialized || !window) {
            return false;
        }
        
        // Check if window exists
        auto it = m_windowsById.find(window->getId());
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << window->getId() << std::endl;
            return false;
        }
        
        // If already active, do nothing
        if (m_activeWindow == window) {
            return true;
        }
        
        // Deactivate current active window
        if (m_activeWindow) {
            m_activeWindow->setState(WindowState::Normal);
        }
        
        // Set new active window
        m_activeWindow = window;
        
        // Raise the window to the top
        m_windows.erase(std::remove(m_windows.begin(), m_windows.end(), window), m_windows.end());
        m_windows.push_back(window);
        
        // Switch to the workspace containing the window
        for (auto& workspace : m_workspaces) {
            if (workspace->containsWindow(window)) {
                switchToWorkspace(workspace);
                break;
            }
        }
        
        std::cout << "Activated window: " << window->getId() << std::endl;
        
        return true;
    }
    
    bool maximizeWindow(std::shared_ptr<Window> window) {
        if (!m_initialized || !window) {
            return false;
        }
        
        // Check if window exists
        auto it = m_windowsById.find(window->getId());
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << window->getId() << std::endl;
            return false;
        }
        
        // Set window state
        window->setState(WindowState::Maximized);
        
        std::cout << "Maximized window: " << window->getId() << std::endl;
        
        return true;
    }
    
    bool minimizeWindow(std::shared_ptr<Window> window) {
        if (!m_initialized || !window) {
            return false;
        }
        
        // Check if window exists
        auto it = m_windowsById.find(window->getId());
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << window->getId() << std::endl;
            return false;
        }
        
        // Set window state
        window->setState(WindowState::Minimized);
        
        // If this is the active window, activate another window
        if (m_activeWindow == window) {
            m_activeWindow = nullptr;
            
            // Find another window to activate
            for (auto& w : m_windows) {
                if (w != window && w->getState() != WindowState::Minimized) {
                    activateWindow(w);
                    break;
                }
            }
        }
        
        std::cout << "Minimized window: " << window->getId() << std::endl;
        
        return true;
    }
    
    bool restoreWindow(std::shared_ptr<Window> window) {
        if (!m_initialized || !window) {
            return false;
        }
        
        // Check if window exists
        auto it = m_windowsById.find(window->getId());
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << window->getId() << std::endl;
            return false;
        }
        
        // Set window state
        window->setState(WindowState::Normal);
        
        std::cout << "Restored window: " << window->getId() << std::endl;
        
        return true;
    }
    
    bool setFullscreen(std::shared_ptr<Window> window, bool fullscreen) {
        if (!m_initialized || !window) {
            return false;
        }
        
        // Check if window exists
        auto it = m_windowsById.find(window->getId());
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << window->getId() << std::endl;
            return false;
        }
        
        // Set window state
        window->setState(fullscreen ? WindowState::Fullscreen : WindowState::Normal);
        
        std::cout << "Set fullscreen for window: " << window->getId() << " - " << (fullscreen ? "true" : "false") << std::endl;
        
        return true;
    }
    
    bool moveWindow(std::shared_ptr<Window> window, int x, int y) {
        if (!m_initialized || !window) {
            return false;
        }
        
        // Check if window exists
        auto it = m_windowsById.find(window->getId());
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << window->getId() << std::endl;
            return false;
        }
        
        // Move window
        window->setPosition(x, y);
        
        std::cout << "Moved window: " << window->getId() << " to (" << x << ", " << y << ")" << std::endl;
        
        return true;
    }
    
    bool resizeWindow(std::shared_ptr<Window> window, int width, int height) {
        if (!m_initialized || !window) {
            return false;
        }
        
        // Check if window exists
        auto it = m_windowsById.find(window->getId());
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << window->getId() << std::endl;
            return false;
        }
        
        // Resize window
        window->setSize(width, height);
        
        std::cout << "Resized window: " << window->getId() << " to " << width << "x" << height << std::endl;
        
        return true;
    }
    
    bool raiseWindow(std::shared_ptr<Window> window) {
        if (!m_initialized || !window) {
            return false;
        }
        
        // Check if window exists
        auto it = m_windowsById.find(window->getId());
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << window->getId() << std::endl;
            return false;
        }
        
        // Raise window
        m_windows.erase(std::remove(m_windows.begin(), m_windows.end(), window), m_windows.end());
        m_windows.push_back(window);
        
        std::cout << "Raised window: " << window->getId() << std::endl;
        
        return true;
    }
    
    bool lowerWindow(std::shared_ptr<Window> window) {
        if (!m_initialized || !window) {
            return false;
        }
        
        // Check if window exists
        auto it = m_windowsById.find(window->getId());
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << window->getId() << std::endl;
            return false;
        }
        
        // Lower window
        m_windows.erase(std::remove(m_windows.begin(), m_windows.end(), window), m_windows.end());
        m_windows.insert(m_windows.begin(), window);
        
        std::cout << "Lowered window: " << window->getId() << std::endl;
        
        return true;
    }
    
    bool snapWindow(std::shared_ptr<Window> window, SnapPosition position) {
        if (!m_initialized || !window) {
            return false;
        }
        
        // Check if window exists
        auto it = m_windowsById.find(window->getId());
        if (it == m_windowsById.end()) {
            std::cerr << "Window not found: " << window->getId() << std::endl;
            return false;
        }
        
        // Get screen dimensions (for now, hardcoded)
        int screenWidth = 1920;
        int screenHeight = 1080;
        
        // Calculate new position and size based on snap position
        int x = 0, y = 0, width = 0, height = 0;
        
        switch (position) {
            case SnapPosition::Left:
                x = 0;
                y = 0;
                width = screenWidth / 2;
                height = screenHeight;
                break;
            case SnapPosition::Right:
                x = screenWidth / 2;
                y = 0;
                width = screenWidth / 2;
                height = screenHeight;
                break;
            case SnapPosition::Top:
                x = 0;
                y = 0;
                width = screenWidth;
                height = screenHeight / 2;
                break;
            case SnapPosition::Bottom:
                x = 0;
                y = screenHeight / 2;
                width = screenWidth;
                height = screenHeight / 2;
                break;
            case SnapPosition::TopLeft:
                x = 0;
                y = 0;
                width = screenWidth / 2;
                height = screenHeight / 2;
                break;
            case SnapPosition::TopRight:
                x = screenWidth / 2;
                y = 0;
                width = screenWidth / 2;
                height = screenHeight / 2;
                break;
            case SnapPosition::BottomLeft:
                x = 0;
                y = screenHeight / 2;
                width = screenWidth / 2;
                height = screenHeight / 2;
                break;
            case SnapPosition::BottomRight:
                x = screenWidth / 2;
                y = screenHeight / 2;
                width = screenWidth / 2;
                height = screenHeight / 2;
                break;
            case SnapPosition::Center:
                x = screenWidth / 4;
                y = screenHeight / 4;
                width = screenWidth / 2;
                height = screenHeight / 2;
                break;
            default:
                return false;
        }
        
        // Set window position and size
        window->setPosition(x, y);
        window->setSize(width, height);
        window->setState(WindowState::Snapped);
        
        std::cout << "Snapped window: " << window->getId() << " to position " << static_cast<int>(position) << std::endl;
        
        return true;
    }
    
    bool tileWindows(const std::string& layout, uint32_t workspaceId) {
        if (!m_initialized) {
            return false;
        }
        
        // Get workspace
        std::shared_ptr<WindowWorkspace> workspace;
        if (workspaceId == 0) {
            workspace = m_currentWorkspace;
        } else {
            workspace = getWorkspace(workspaceId);
        }
        
        if (!workspace) {
            std::cerr << "Workspace not found: " << workspaceId << std::endl;
            return false;
        }
        
        // Get windows in workspace
        auto windows = workspace->getWindows();
        
        // Skip if no windows
        if (windows.empty()) {
            return true;
        }
        
        // Get screen dimensions (for now, hardcoded)
        int screenWidth = 1920;
        int screenHeight = 108
(Content truncated due to size limit. Use line ranges to read in chunks)