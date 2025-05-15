#include "OutputManager.h"
#include "WaylandCompositor.h"
#include <iostream>
#include <mutex>
#include <algorithm>

namespace VivoX {
namespace Compositor {
namespace Wayland {

class OutputManager::Impl {
public:
    Impl()
        : m_initialized(false)
        , m_compositor(nullptr) {}
    
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
        
        // Detect outputs
        detectOutputs();
        
        m_initialized = true;
        return true;
    }
    
    void shutdown() {
        if (!m_initialized) {
            return;
        }
        
        m_outputs.clear();
        m_compositor = nullptr;
        m_initialized = false;
    }
    
    bool isInitialized() const {
        return m_initialized;
    }
    
    std::vector<OutputInfo> getOutputs() const {
        std::vector<OutputInfo> outputs;
        
        for (const auto& pair : m_outputs) {
            outputs.push_back(pair.second);
        }
        
        return outputs;
    }
    
    const OutputInfo* getOutput(const std::string& name) const {
        auto it = m_outputs.find(name);
        if (it == m_outputs.end()) {
            return nullptr;
        }
        
        return &it->second;
    }
    
    const OutputInfo* getPrimaryOutput() const {
        for (const auto& pair : m_outputs) {
            if (pair.second.primary) {
                return &pair.second;
            }
        }
        
        return nullptr;
    }
    
    bool setPrimaryOutput(const std::string& name) {
        auto it = m_outputs.find(name);
        if (it == m_outputs.end()) {
            return false;
        }
        
        // Reset primary flag for all outputs
        for (auto& pair : m_outputs) {
            pair.second.primary = false;
        }
        
        // Set primary flag for the specified output
        it->second.primary = true;
        
        return true;
    }
    
    bool setOutputEnabled(const std::string& name, bool enable) {
        auto it = m_outputs.find(name);
        if (it == m_outputs.end()) {
            return false;
        }
        
        it->second.enabled = enable;
        
        return true;
    }
    
    bool setOutputPosition(const std::string& name, int x, int y) {
        auto it = m_outputs.find(name);
        if (it == m_outputs.end()) {
            return false;
        }
        
        it->second.x = x;
        it->second.y = y;
        
        return true;
    }
    
    bool setOutputScale(const std::string& name, float scale) {
        auto it = m_outputs.find(name);
        if (it == m_outputs.end()) {
            return false;
        }
        
        it->second.scale = scale;
        
        return true;
    }
    
    bool setOutputMode(const std::string& name, int width, int height, int refreshRate) {
        auto it = m_outputs.find(name);
        if (it == m_outputs.end()) {
            return false;
        }
        
        it->second.width = width;
        it->second.height = height;
        it->second.refreshRate = refreshRate;
        
        return true;
    }
    
    bool applyConfiguration() {
        if (!m_initialized) {
            return false;
        }
        
        // Apply output configuration
        // Implementation details would go here
        
        return true;
    }
    
private:
    void detectOutputs() {
        // Clear existing outputs
        m_outputs.clear();
        
        // Detect outputs
        // Implementation details would go here
        
        // For now, add a dummy output
        OutputInfo output;
        output.name = "HDMI-1";
        output.model = "Generic Monitor";
        output.manufacturer = "Generic";
        output.x = 0;
        output.y = 0;
        output.width = 1920;
        output.height = 1080;
        output.physicalWidth = 476;
        output.physicalHeight = 268;
        output.scale = 1.0f;
        output.refreshRate = 60000;
        output.enabled = true;
        output.primary = true;
        
        m_outputs[output.name] = output;
    }
    
    bool m_initialized;
    std::shared_ptr<WaylandCompositor> m_compositor;
    std::map<std::string, OutputInfo> m_outputs;
};

std::shared_ptr<OutputManager> OutputManager::s_instance = nullptr;
std::mutex OutputManager::s_instanceMutex;

std::shared_ptr<OutputManager> OutputManager::getInstance() {
    std::lock_guard<std::mutex> lock(s_instanceMutex);
    
    if (!s_instance) {
        s_instance = std::shared_ptr<OutputManager>(new OutputManager());
    }
    
    return s_instance;
}

OutputManager::OutputManager() : m_impl(std::make_unique<Impl>()) {
}

OutputManager::~OutputManager() {
}

bool OutputManager::initialize(std::shared_ptr<WaylandCompositor> compositor) {
    return m_impl->initialize(compositor);
}

void OutputManager::shutdown() {
    m_impl->shutdown();
}

bool OutputManager::isInitialized() const {
    return m_impl->isInitialized();
}

std::vector<OutputManager::OutputInfo> OutputManager::getOutputs() const {
    return m_impl->getOutputs();
}

const OutputManager::OutputInfo* OutputManager::getOutput(const std::string& name) const {
    return m_impl->getOutput(name);
}

const OutputManager::OutputInfo* OutputManager::getPrimaryOutput() const {
    return m_impl->getPrimaryOutput();
}

bool OutputManager::setPrimaryOutput(const std::string& name) {
    return m_impl->setPrimaryOutput(name);
}

bool OutputManager::setOutputEnabled(const std::string& name, bool enable) {
    return m_impl->setOutputEnabled(name, enable);
}

bool OutputManager::setOutputPosition(const std::string& name, int x, int y) {
    return m_impl->setOutputPosition(name, x, y);
}

bool OutputManager::setOutputScale(const std::string& name, float scale) {
    return m_impl->setOutputScale(name, scale);
}

bool OutputManager::setOutputMode(const std::string& name, int width, int height, int refreshRate) {
    return m_impl->setOutputMode(name, width, height, refreshRate);
}

bool OutputManager::applyConfiguration() {
    return m_impl->applyConfiguration();
}

} // namespace Wayland
} // namespace Compositor
} // namespace VivoX
