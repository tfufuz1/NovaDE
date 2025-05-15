// RenderEngine.h
#pragma once

#include <memory>
#include <string>
#include <vector>
#include <cstdint>

namespace VivoX {
namespace Compositor {
namespace Rendering {

// Forward declarations
class RenderSurface;
class RenderTexture;
class RenderShader;
class RenderTarget;

// Effect types for visual effects
enum class EffectType {
    None,
    Blur,
    Shadow,
    Neon,
    Reflection,
    Distortion,
    ColorAdjustment
};

// Parameters for visual effects
struct EffectParams {
    EffectType type = EffectType::None;
    float radius = 0.0f;       // For blur, shadow, neon effects
    float offsetX = 0.0f;      // For shadow effect
    float offsetY = 0.0f;      // For shadow effect
    uint32_t color = 0;        // For shadow, neon effects (RGBA8 format)
    float intensity = 1.0f;    // For neon, reflection, distortion effects
    std::map<std::string, float> customParams; // For additional effect parameters
};

/**
 * @class RenderEngine
 * @brief Main rendering engine class responsible for handling all rendering operations
 * 
 * The RenderEngine provides rendering capabilities for the compositing system.
 * It supports hardware acceleration through OpenGL ES or Vulkan, with a fallback
 * to software rendering when hardware acceleration is unavailable.
 */
class RenderEngine {
public:
    /**
     * Constructor
     */
    RenderEngine();
    
    /**
     * Destructor
     */
    ~RenderEngine();
    
    /**
     * Initialize the rendering engine
     * 
     * @param useHardwareAcceleration Whether to use hardware acceleration if available
     * @param preferredBackend Preferred rendering backend ("opengl", "vulkan", or "software")
     * @return True if initialization was successful, false otherwise
     */
    bool initialize(bool useHardwareAcceleration = true, const std::string& preferredBackend = "opengl");
    
    /**
     * Shutdown the rendering engine and release all resources
     */
    void shutdown();
    
    /**
     * Check if the engine is initialized
     * 
     * @return True if initialized, false otherwise
     */
    bool isInitialized() const;
    
    /**
     * Check if hardware acceleration is available
     * 
     * @return True if hardware acceleration is available, false otherwise
     */
    bool isHardwareAccelerationAvailable() const;
    
    /**
     * Check if hardware acceleration is enabled
     * 
     * @return True if hardware acceleration is enabled, false otherwise
     */
    bool isHardwareAccelerationEnabled() const;
    
    /**
     * Enable or disable hardware acceleration
     * 
     * @param enable Whether to enable hardware acceleration
     * @return True if the operation was successful, false otherwise
     * @note Cannot be changed while the engine is initialized
     */
    bool setHardwareAcceleration(bool enable);
    
    /**
     * Get the current rendering backend
     * 
     * @return The name of the current backend ("opengl", "vulkan", or "software")
     */
    std::string getCurrentBackend() const;
    
    /**
     * Set the rendering backend
     * 
     * @param backend The name of the backend to use ("opengl", "vulkan", or "software")
     * @return True if the operation was successful, false otherwise
     * @note Cannot be changed while the engine is initialized
     */
    bool setBackend(const std::string& backend);
    
    /**
     * Get the list of available rendering backends
     * 
     * @return A vector of available backend names
     */
    std::vector<std::string> getAvailableBackends() const;
    
    /**
     * Start the rendering engine
     */
    void start();
    
    /**
     * Stop the rendering engine
     */
    void stop();
    
    /**
     * Check if the engine is running
     * 
     * @return True if running, false otherwise
     */
    bool isRunning() const;
    
    /**
     * Begin a new frame
     * 
     * @return True if successful, false otherwise
     */
    bool beginFrame();
    
    /**
     * End the current frame and present it
     * 
     * @return True if successful, false otherwise
     */
    bool endFrame();
    
    /**
     * Create a new render surface
     * 
     * @param width The width of the surface
     * @param height The height of the surface
     * @param format The pixel format of the surface (e.g., "rgba8")
     * @return A shared pointer to the created surface, or nullptr on failure
     */
    std::shared_ptr<RenderSurface> createSurface(int width, int height, const std::string& format = "rgba8");
    
    /**
     * Create a new render texture
     * 
     * @param width The width of the texture
     * @param height The height of the texture
     * @param format The pixel format of the texture (e.g., "rgba8")
     * @return A shared pointer to the created texture, or nullptr on failure
     */
    std::shared_ptr<RenderTexture> createTexture(int width, int height, const std::string& format = "rgba8");
    
    /**
     * Create a new shader program
     * 
     * @param vertexShader The vertex shader source code
     * @param fragmentShader The fragment shader source code
     * @return A shared pointer to the created shader, or nullptr on failure
     */
    std::shared_ptr<RenderShader> createShader(const std::string& vertexShader, const std::string& fragmentShader);
    
    /**
     * Create a new render target for offscreen rendering
     * 
     * @param width The width of the render target
     * @param height The height of the render target
     * @param format The pixel format of the render target (e.g., "rgba8")
     * @return A shared pointer to the created render target, or nullptr on failure
     */
    std::shared_ptr<RenderTarget> createRenderTarget(int width, int height, const std::string& format = "rgba8");
    
    /**
     * Apply a visual effect to a render surface
     * 
     * @param surface The surface to apply the effect to
     * @param effect The effect parameters
     * @return True if successful, false otherwise
     */
    bool applyEffect(std::shared_ptr<RenderSurface> surface, const EffectParams& effect);
    
    /**
     * Get the current frame rate
     * 
     * @return The current frame rate in frames per second
     */
    int getFrameRate() const;
    
    /**
     * Get the time taken to render the last frame
     * 
     * @return The frame time in seconds
     */
    float getFrameTime() const;
    
    /**
     * Get the number of draw calls in the last frame
     * 
     * @return The number of draw calls
     */
    int getDrawCalls() const;
    
    /**
     * Get the current GPU memory usage
     * 
     * @return The GPU memory usage in bytes
     */
    uint64_t getGPUMemoryUsage() const;
    
    /**
     * Enable or disable vertical synchronization
     * 
     * @param enable Whether to enable vsync
     */
    void setVSync(bool enable);
    
    /**
     * Check if vertical synchronization is enabled
     * 
     * @return True if vsync is enabled, false otherwise
     */
    bool isVSyncEnabled() const;
    
    /**
     * Set the maximum frame rate
     * 
     * @param frameRate The maximum frame rate in frames per second (0 for unlimited)
     * @note This is only used if vsync is disabled
     */
    void setMaxFrameRate(int frameRate);
    
    /**
     * Get the maximum frame rate
     * 
     * @return The maximum frame rate in frames per second (0 for unlimited)
     */
    int getMaxFrameRate() const;
    
private:
    // Implementation using the PIMPL idiom
    class Impl;
    Impl* m_pImpl;
    
    // Disable copy and assignment
    RenderEngine(const RenderEngine&) = delete;
    RenderEngine& operator=(const RenderEngine&) = delete;
};

} // namespace Rendering
} // namespace Compositor
} // namespace VivoX
