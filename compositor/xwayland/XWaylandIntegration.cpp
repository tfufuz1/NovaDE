#include "XWaylandIntegration.h"
#include "../wayland/WaylandCompositor.h"
#include <iostream>
#include <mutex>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/wait.h>
#include <fcntl.h>

namespace VivoX {
namespace Compositor {
namespace XWayland {

// Forward declaration for XWaylandSurface
class XWaylandSurface {
public:
    XWaylandSurface(uint32_t windowId, int width, int height);
    ~XWaylandSurface();
    
    uint32_t getWindowId() const { return m_windowId; }
    int getWidth() const { return m_width; }
    int getHeight() const { return m_height; }
    int getX() const { return m_x; }
    int getY() const { return m_y; }
    bool isMapped() const { return m_mapped; }
    bool isFocused() const { return m_focused; }
    
    void setPosition(int x, int y) { m_x = x; m_y = y; }
    void setSize(int width, int height) { m_width = width; m_height = height; }
    void setMapped(bool mapped) { m_mapped = mapped; }
    void setFocused(bool focused) { m_focused = focused; }
    
private:
    uint32_t m_windowId;
    int m_width;
    int m_height;
    int m_x;
    int m_y;
    bool m_mapped;
    bool m_focused;
};

// Implementation of XWaylandSurface
XWaylandSurface::XWaylandSurface(uint32_t windowId, int width, int height)
    : m_windowId(windowId)
    , m_width(width)
    , m_height(height)
    , m_x(0)
    , m_y(0)
    , m_mapped(false)
    , m_focused(false) {
}

XWaylandSurface::~XWaylandSurface() {
}

// Implementation of XWaylandIntegration
class XWaylandIntegration::Impl {
public:
    Impl()
        : m_initialized(false)
        , m_running(false)
        , m_compositor(nullptr)
        , m_xwaylandPid(-1)
        , m_displayFd(-1) {
    }
    
    ~Impl() {
        shutdown();
    }
    
    bool initialize(std::shared_ptr<WaylandCompositor> compositor) {
        if (!compositor) {
            std::cerr << "Invalid compositor instance" << std::endl;
            return false;
        }
        
        if (m_initialized) {
            return true;
        }
        
        m_compositor = compositor;
        
        // Create socket pair for X display number
        int displaySockets[2];
        if (socketpair(AF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0, displaySockets) < 0) {
            std::cerr << "Failed to create socket pair for X display" << std::endl;
            return false;
        }
        
        // Fork XWayland process
        m_xwaylandPid = fork();
        if (m_xwaylandPid < 0) {
            std::cerr << "Failed to fork XWayland process" << std::endl;
            close(displaySockets[0]);
            close(displaySockets[1]);
            return false;
        }
        
        if (m_xwaylandPid == 0) {
            // Child process (XWayland)
            close(displaySockets[0]);
            
            // Set environment variables
            setenv("WAYLAND_SOCKET", std::to_string(displaySockets[1]).c_str(), 1);
            
            // Execute XWayland
            execl("/usr/bin/Xwayland", "Xwayland", "-rootless", "-terminate", NULL);
            
            // If we get here, execl failed
            std::cerr << "Failed to execute XWayland" << std::endl;
            _exit(EXIT_FAILURE);
        }
        
        // Parent process
        close(displaySockets[1]);
        m_displayFd = displaySockets[0];
        
        // Set non-blocking
        int flags = fcntl(m_displayFd, F_GETFL, 0);
        fcntl(m_displayFd, F_SETFL, flags | O_NONBLOCK);
        
        m_initialized = true;
        m_running = true;
        
        return true;
    }
    
    void shutdown() {
        if (!m_initialized) {
            return;
        }
        
        // Kill XWayland process
        if (m_xwaylandPid > 0) {
            kill(m_xwaylandPid, SIGTERM);
            waitpid(m_xwaylandPid, NULL, 0);
            m_xwaylandPid = -1;
        }
        
        // Close display socket
        if (m_displayFd >= 0) {
            close(m_displayFd);
            m_displayFd = -1;
        }
        
        // Clear surfaces
        m_surfaces.clear();
        
        m_compositor = nullptr;
        m_initialized = false;
        m_running = false;
    }
    
    bool isInitialized() const {
        return m_initialized;
    }
    
    bool isRunning() const {
        return m_running;
    }
    
    std::vector<std::shared_ptr<XWaylandSurface>> getSurfaces() const {
        std::vector<std::shared_ptr<XWaylandSurface>> surfaces;
        
        for (const auto& pair : m_surfaces) {
            surfaces.push_back(pair.second);
        }
        
        return surfaces;
    }
    
    std::shared_ptr<XWaylandSurface> getSurface(uint32_t windowId) const {
        auto it = m_surfaces.find(windowId);
        if (it == m_surfaces.end()) {
            return nullptr;
        }
        
        return it->second;
    }
    
    bool mapWindow(uint32_t windowId) {
        auto surface = getSurface(windowId);
        if (!surface) {
            // Create new surface
            surface = std::make_shared<XWaylandSurface>(windowId, 800, 600);
            m_surfaces[windowId] = surface;
        }
        
        surface->setMapped(true);
        return true;
    }
    
    bool unmapWindow(uint32_t windowId) {
        auto surface = getSurface(windowId);
        if (!surface) {
            return false;
        }
        
        surface->setMapped(false);
        return true;
    }
    
    bool setWindowPosition(uint32_t windowId, int x, int y) {
        auto surface = getSurface(windowId);
        if (!surface) {
            return false;
        }
        
        surface->setPosition(x, y);
        return true;
    }
    
    bool setWindowSize(uint32_t windowId, int width, int height) {
        auto surface = getSurface(windowId);
        if (!surface) {
            return false;
        }
        
        surface->setSize(width, height);
        return true;
    }
    
    bool focusWindow(uint32_t windowId) {
        // Unfocus all windows
        for (auto& pair : m_surfaces) {
            pair.second->setFocused(false);
        }
        
        auto surface = getSurface(windowId);
        if (!surface) {
            return false;
        }
        
        surface->setFocused(true);
        return true;
    }
    
private:
    bool m_initialized;
    bool m_running;
    std::shared_ptr<WaylandCompositor> m_compositor;
    pid_t m_xwaylandPid;
    int m_displayFd;
    std::map<uint32_t, std::shared_ptr<XWaylandSurface>> m_surfaces;
};

std::shared_ptr<XWaylandIntegration> XWaylandIntegration::s_instance = nullptr;
std::mutex XWaylandIntegration::s_instanceMutex;

std::shared_ptr<XWaylandIntegration> XWaylandIntegration::getInstance() {
    std::lock_guard<std::mutex> lock(s_instanceMutex);
    
    if (!s_instance) {
        s_instance = std::shared_ptr<XWaylandIntegration>(new XWaylandIntegration());
    }
    
    return s_instance;
}

XWaylandIntegration::XWaylandIntegration() : m_impl(std::make_unique<Impl>()) {
}

XWaylandIntegration::~XWaylandIntegration() {
}

bool XWaylandIntegration::initialize(std::shared_ptr<WaylandCompositor> compositor) {
    return m_impl->initialize(compositor);
}

void XWaylandIntegration::shutdown() {
    m_impl->shutdown();
}

bool XWaylandIntegration::isInitialized() const {
    return m_impl->isInitialized();
}

bool XWaylandIntegration::isRunning() const {
    return m_impl->isRunning();
}

std::vector<std::shared_ptr<XWaylandSurface>> XWaylandIntegration::getSurfaces() const {
    return m_impl->getSurfaces();
}

std::shared_ptr<XWaylandSurface> XWaylandIntegration::getSurface(uint32_t windowId) const {
    return m_impl->getSurface(windowId);
}

bool XWaylandIntegration::mapWindow(uint32_t windowId) {
    return m_impl->mapWindow(windowId);
}

bool XWaylandIntegration::unmapWindow(uint32_t windowId) {
    return m_impl->unmapWindow(windowId);
}

bool XWaylandIntegration::setWindowPosition(uint32_t windowId, int x, int y) {
    return m_impl->setWindowPosition(windowId, x, y);
}

bool XWaylandIntegration::setWindowSize(uint32_t windowId, int width, int height) {
    return m_impl->setWindowSize(windowId, width, height);
}

bool XWaylandIntegration::focusWindow(uint32_t windowId) {
    return m_impl->focusWindow(windowId);
}

} // namespace XWayland
} // namespace Compositor
} // namespace VivoX
