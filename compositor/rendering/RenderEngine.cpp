#include "RenderEngine.h"
#include "RenderSurface.h"
#include "RenderTexture.h"
#include "RenderShader.h"
#include "RenderTarget.h"

#include <iostream>
#include <chrono>
#include <thread>
#include <stdexcept>
#include <cmath>
#include <fstream>
#include <algorithm>
#include <sstream>

#include <GL/gl.h>
#include <GL/glext.h>
#include <EGL/egl.h>
#include <EGL/eglext.h>

namespace VivoX {
namespace Compositor {
namespace Rendering {

// Standard shaders that will be used by the rendering engine
namespace Shaders {
    // Basic vertex shader that transforms vertices
    const std::string basicVertexShader = R"(
        #version 300 es
        precision highp float;
        
        layout(location = 0) in vec3 a_position;
        layout(location = 1) in vec2 a_texCoord;
        
        out vec2 v_texCoord;
        
        uniform mat4 u_mvpMatrix;
        
        void main() {
            gl_Position = u_mvpMatrix * vec4(a_position, 1.0);
            v_texCoord = a_texCoord;
        }
    )";
    
    // Basic fragment shader that samples a texture
    const std::string basicFragmentShader = R"(
        #version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        out vec4 fragColor;
        
        uniform sampler2D u_texture;
        
        void main() {
            fragColor = texture(u_texture, v_texCoord);
        }
    )";
    
    // Blur shader for applying gaussian blur
    const std::string blurFragmentShader = R"(
        #version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        out vec4 fragColor;
        
        uniform sampler2D u_texture;
        uniform vec2 u_resolution;
        uniform float u_radius;
        uniform vec2 u_direction;
        
        void main() {
            vec4 color = vec4(0.0);
            vec2 off1 = vec2(1.3333333333333333) * u_direction;
            vec2 off2 = vec2(3.1111111111111112) * u_direction;
            vec2 off3 = vec2(5.0) * u_direction;
            
            float sigma = u_radius * 0.5;
            float s = sigma * sigma;
            
            vec2 texelSize = 1.0 / u_resolution;
            off1 *= texelSize * sigma;
            off2 *= texelSize * sigma;
            off3 *= texelSize * sigma;
            
            color += texture(u_texture, v_texCoord) * 0.2270270270;
            color += texture(u_texture, v_texCoord + off1) * 0.3162162162;
            color += texture(u_texture, v_texCoord - off1) * 0.3162162162;
            color += texture(u_texture, v_texCoord + off2) * 0.0702702703;
            color += texture(u_texture, v_texCoord - off2) * 0.0702702703;
            
            fragColor = color;
        }
    )";
    
    // Shadow shader for applying drop shadows
    const std::string shadowFragmentShader = R"(
        #version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        out vec4 fragColor;
        
        uniform sampler2D u_texture;
        uniform vec2 u_resolution;
        uniform vec2 u_shadowOffset;
        uniform vec4 u_shadowColor;
        uniform float u_shadowBlur;
        
        void main() {
            vec2 texelSize = 1.0 / u_resolution;
            vec2 shadowTexCoord = v_texCoord - u_shadowOffset * texelSize;
            
            float alpha = 0.0;
            float totalWeight = 0.0;
            float sigma = u_shadowBlur * 0.5;
            float s = sigma * sigma;
            
            for (float y = -u_shadowBlur; y <= u_shadowBlur; y += 1.0) {
                for (float x = -u_shadowBlur; x <= u_shadowBlur; x += 1.0) {
                    float weight = exp(-(x*x + y*y) / (2.0 * 3.14159265359 * s)) / (2.0 * 3.14159265359 * s);
                    vec2 offset = vec2(x, y) * texelSize;
                    alpha += texture(u_texture, shadowTexCoord + offset).a * weight;
                    totalWeight += weight;
                }
            }
            
            alpha /= totalWeight;
            
            vec4 originalColor = texture(u_texture, v_texCoord);
            vec4 shadowColor = vec4(u_shadowColor.rgb, u_shadowColor.a * alpha);
            
            // Mix the shadow and original color based on original alpha
            fragColor = mix(shadowColor, originalColor, originalColor.a);
        }
    )";
    
    // Neon glow effect shader
    const std::string neonFragmentShader = R"(
        #version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        out vec4 fragColor;
        
        uniform sampler2D u_texture;
        uniform vec2 u_resolution;
        uniform float u_radius;
        uniform vec4 u_glowColor;
        uniform float u_intensity;
        
        void main() {
            vec4 originalColor = texture(u_texture, v_texCoord);
            vec2 texelSize = 1.0 / u_resolution;
            
            float alpha = 0.0;
            float totalWeight = 0.0;
            float sigma = u_radius * 0.5;
            float s = sigma * sigma;
            
            for (float y = -u_radius; y <= u_radius; y += 1.0) {
                for (float x = -u_radius; x <= u_radius; x += 1.0) {
                    float weight = exp(-(x*x + y*y) / (2.0 * s)) / (2.0 * 3.14159265359 * s);
                    vec2 offset = vec2(x, y) * texelSize;
                    alpha += texture(u_texture, v_texCoord + offset).a * weight;
                    totalWeight += weight;
                }
            }
            
            alpha /= totalWeight;
            alpha = min(alpha * u_intensity, 1.0);
            
            vec4 glowColor = vec4(u_glowColor.rgb, u_glowColor.a * alpha);
            
            // Add glow to original color
            fragColor = vec4(originalColor.rgb + glowColor.rgb * glowColor.a * (1.0 - originalColor.a), 
                           max(originalColor.a, glowColor.a * (1.0 - originalColor.a)));
        }
    )";
    
    // Reflection effect shader
    const std::string reflectionFragmentShader = R"(
        #version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        out vec4 fragColor;
        
        uniform sampler2D u_texture;
        uniform float u_intensity;
        uniform float u_fadeDistance;
        
        void main() {
            vec4 originalColor = texture(u_texture, v_texCoord);
            
            // Calculate reflection coord (flipped vertically)
            vec2 reflectionCoord = vec2(v_texCoord.x, 2.0 - v_texCoord.y);
            
            // Sample the reflection
            vec4 reflectionColor = texture(u_texture, reflectionCoord);
            
            // Calculate fade factor based on distance from center
            float fadeFactor = clamp(1.0 - (v_texCoord.y / u_fadeDistance), 0.0, 1.0);
            
            // Mix the original color with the reflection
            fragColor = originalColor;
            
            // Add the reflection if we're in the bottom half
            if (v_texCoord.y > 1.0) {
                fragColor = mix(originalColor, reflectionColor, u_intensity * fadeFactor);
            }
        }
    )";
    
    // Distortion effect shader
    const std::string distortionFragmentShader = R"(
        #version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        out vec4 fragColor;
        
        uniform sampler2D u_texture;
        uniform vec2 u_resolution;
        uniform float u_time;
        uniform float u_intensity;
        
        void main() {
            vec2 uv = v_texCoord;
            
            // Create some distortion
            float distortionX = sin(uv.y * 10.0 + u_time) * 0.01 * u_intensity;
            float distortionY = cos(uv.x * 10.0 + u_time) * 0.01 * u_intensity;
            
            vec2 distortedUV = vec2(uv.x + distortionX, uv.y + distortionY);
            
            // Sample with distorted coordinates
            fragColor = texture(u_texture, distortedUV);
        }
    )";
    
    // Color adjustment effect shader
    const std::string colorAdjustmentFragmentShader = R"(
        #version 300 es
        precision highp float;
        
        in vec2 v_texCoord;
        out vec4 fragColor;
        
        uniform sampler2D u_texture;
        uniform float u_brightness;
        uniform float u_contrast;
        uniform float u_saturation;
        uniform float u_hue;
        
        // Function to adjust hue of a color
        vec3 adjustHue(vec3 color, float hue) {
            const vec3 k = vec3(0.57735, 0.57735, 0.57735);
            float cosAngle = cos(hue);
            return vec3(color * cosAngle + cross(k, color) * sin(hue) + k * dot(k, color) * (1.0 - cosAngle));
        }
        
        // Function to adjust saturation of a color
        vec3 adjustSaturation(vec3 color, float saturation) {
            float luminance = dot(color, vec3(0.2126, 0.7152, 0.0722));
            return mix(vec3(luminance), color, saturation);
        }
        
        void main() {
            // Sample the texture
            vec4 color = texture(u_texture, v_texCoord);
            
            // Adjust brightness
            vec3 adjusted = color.rgb * u_brightness;
            
            // Adjust contrast
            adjusted = (adjusted - 0.5) * u_contrast + 0.5;
            
            // Adjust saturation
            adjusted = adjustSaturation(adjusted, u_saturation);
            
            // Adjust hue
            adjusted = adjustHue(adjusted, u_hue);
            
            // Output the final color
            fragColor = vec4(adjusted, color.a);
        }
    )";
}

// Simple vertex data for a fullscreen quad
struct Vertex {
    float x, y, z;    // Position
    float u, v;       // Texture coordinates
};

const Vertex quadVertices[] = {
    { -1.0f, -1.0f, 0.0f, 0.0f, 0.0f },  // Bottom-left
    {  1.0f, -1.0f, 0.0f, 1.0f, 0.0f },  // Bottom-right
    {  1.0f,  1.0f, 0.0f, 1.0f, 1.0f },  // Top-right
    { -1.0f,  1.0f, 0.0f, 0.0f, 1.0f }   // Top-left
};

const uint16_t quadIndices[] = {
    0, 1, 2,  // First triangle
    0, 2, 3   // Second triangle
};

// Implementation of the RenderEngine class
class RenderEngine::Impl {
public:
    Impl()
        : m_initialized(false)
        , m_hardwareAccelerationEnabled(false)
        , m_hardwareAccelerationAvailable(false)
        , m_vSync(true)
        , m_maxFrameRate(60)
        , m_currentFrameRate(0)
        , m_frameTime(0.0f)
        , m_frameCount(0)
        , m_drawCalls(0)
        , m_eglDisplay(EGL_NO_DISPLAY)
        , m_eglContext(EGL_NO_CONTEXT)
        , m_eglSurface(EGL_NO_SURFACE)
        , m_vao(0)
        , m_vbo(0)
        , m_ibo(0)
        , m_currentBackend("none")
        , m_isRunning(false)
        , m_lastFrameTime(std::chrono::high_resolution_clock::now())
        , m_gpuMemoryUsage(0) {
        
        // Initialize available backends
        m_availableBackends.push_back("opengl");
        m_availableBackends.push_back("vulkan");
        m_availableBackends.push_back("software");
    }
    
    ~Impl() {
        shutdown();
    }
    
    bool initialize(bool useHardwareAcceleration, const std::string& preferredBackend) {
        // Check if already initialized
        if (m_initialized) {
            return true;
        }
        
        // Check hardware acceleration availability
        m_hardwareAccelerationAvailable = checkHardwareAcceleration();
        
        // Set hardware acceleration based on availability and user preference
        m_hardwareAccelerationEnabled = useHardwareAcceleration && m_hardwareAccelerationAvailable;
        
        // Choose backend
        if (preferredBackend == "opengl" && m_hardwareAccelerationEnabled) {
            m_currentBackend = "opengl";
            if (!initializeOpenGL()) {
                std::cerr << "Failed to initialize OpenGL backend" << std::endl;
                return false;
            }
        } else if (preferredBackend == "vulkan" && m_hardwareAccelerationEnabled) {
            m_currentBackend = "vulkan";
            if (!initializeVulkan()) {
                std::cerr << "Failed to initialize Vulkan backend" << std::endl;
                return false;
            }
        } else {
            m_currentBackend = "software";
            m_hardwareAccelerationEnabled = false;
            if (!initializeSoftwareRendering()) {
                std::cerr << "Failed to initialize software rendering backend" << std::endl;
                return false;
            }
        }
        
        // Create basic shaders
        createShaders();
        
        m_initialized = true;
        return true;
    }
    
    void shutdown() {
        if (!m_initialized) {
            return;
        }
        
        // Stop rendering if running
        stop();
        
        // Destroy shaders
        m_shaders.clear();
        m_shadersByName.clear();
        
        // Cleanup based on backend
        if (m_currentBackend == "opengl") {
            cleanupOpenGL();
        } else if (m_currentBackend == "vulkan") {
            cleanupVulkan();
        } else if (m_currentBackend == "software") {
            cleanupSoftwareRendering();
        }
        
        m_initialized = false;
    }
    
    bool isInitialized() const {
        return m_initialized;
    }
    
    bool isHardwareAccelerationAvailable() const {
        return m_hardwareAccelerationAvailable;
    }
    
    bool isHardwareAccelerationEnabled() const {
        return m_hardwareAccelerationEnabled;
    }
    
    bool setHardwareAcceleration(bool enable) {
        if (m_initialized) {
            // Cannot change hardware acceleration while initialized
            std::cerr << "Cannot change hardware acceleration while engine is initialized" << std::endl;
            return false;
        }
        
        if (enable && !m_hardwareAccelerationAvailable) {
            std::cerr << "Hardware acceleration not available" << std::endl;
            return false;
        }
        
        m_hardwareAccelerationEnabled = enable;
        return true;
    }
    
    std::string getCurrentBackend() const {
        return m_currentBackend;
    }
    
    bool setBackend(const std::string& backend) {
        if (m_initialized) {
            // Cannot change backend while initialized
            std::cerr << "Cannot change backend while engine is initialized" << std::endl;
            return false;
        }
        
        // Check if backend is available
        if (std::find(m_availableBackends.begin(), m_availableBackends.end(), backend) == m_availableBackends.end()) {
            std::cerr << "Backend not available: " << backend << std::endl;
            return false;
        }
        
        // Check if hardware acceleration is required but not available
        if ((backend == "opengl" || backend == "vulkan") && !m_hardwareAccelerationAvailable) {
            std::cerr << "Hardware acceleration not available for backend: " << backend << std::endl;
            return false;
        }
        
        m_currentBackend = backend;
        return true;
    }
    
    std::vector<std::string> getAvailableBackends() const {
        return m_availableBackends;
    }
    
    void start() {
        if (!m_initialized) {
            std::cerr << "Engine not initialized" << std::endl;
            return;
        }
        
        if (m_isRunning) {
            std::cerr << "Engine already running" << std::endl;
            return;
        }
        
        m_isRunning = true;
        m_lastFrameTime = std::chrono::high_resolution_clock::now();
    }
    
    void stop() {
        m_isRunning = false;
    }
    
    bool isRunning() const {
        return m_isRunning;
    }
    
    bool beginFrame() {
        if (!m_initialized || !m_isRunning) {
            return false;
        }
        
        // Calculate frame time
        auto currentTime = std::chrono::high_resolution_clock::now();
        auto delta = currentTime - m_lastFrameTime;
        m_frameTime = std::chrono::duration<float>(delta).count();
        m_lastFrameTime = currentTime;
        
        // Limit frame rate if needed
        if (m_maxFrameRate > 0 && !m_vSync) {
            float targetFrameTime = 1.0f / static_cast<float>(m_maxFrameRate);
            if (m_frameTime < targetFrameTime) {
                std::this_thread::sleep_for(std::chrono::duration<float>(targetFrameTime - m_frameTime));
                
                // Recalculate frame time
                currentTime = std::chrono::high_resolution_clock::now();
                delta = currentTime - m_lastFrameTime;
                m_frameTime = std::chrono::duration<float>(delta).count();
                m_lastFrameTime = currentTime;
            }
        }
        
        // Calculate current frame rate
        if (m_frameTime > 0.0f) {
            m_currentFrameRate = static_cast<int>(1.0f / m_frameTime);
        }
        
        // Reset draw calls counter
        m_drawCalls = 0;
        
        // Clear the frame
        glClearColor(0.0f, 0.0f, 0.0f, 1.0f);
        glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        
        return true;
    }
    
    bool endFrame() {
        if (!m_initialized || !m_isRunning) {
            return false;
        }
        
        // Increment frame counter
        m_frameCount++;
        
        // Update stats
        updateStats();
        
        // Swap buffers based on backend
        if (m_currentBackend == "opengl") {
            eglSwapBuffers(m_eglDisplay, m_eglSurface);
        } else if (m_currentBackend == "vulkan") {
            // Vulkan present
            presentVulkanFrame();
        } else if (m_currentBackend == "software") {
            // Software present
            presentSoftwareFrame();
        }
        
        return true;
    }
    
    std::shared_ptr<RenderSurface> createSurface(int width, int height, const std::string& format) {
        if (!m_initialized) {
            std::cerr << "Engine not initialized" << std::endl;
            return nullptr;
        }
        
        auto surface = std::make_shared<RenderSurface>(width, height, format);
        m_surfaces.push_back(surface);
        
        // Update GPU memory usage
        m_gpuMemoryUsage += calculateTextureMemoryUsage(width, height, format);
        
        return surface;
    }
    
    std::shared_ptr<RenderTexture> createTexture(int width, int height, const std::string& format) {
        if (!m_initialized) {
            std::cerr << "Engine not initialized" << std::endl;
            return nullptr;
        }
        
        auto texture = std::make_shared<RenderTexture>(width, height, format);
        m_textures.push_back(texture);
        
        // Update GPU memory usage
        m_gpuMemoryUsage += calculateTextureMemoryUsage(width, height, format);
        
        return texture;
    }
    
    std::shared_ptr<RenderShader> createShader(const std::string& vertexShader, const std::string& fragmentShader) {
        if (!m_initialized) {
            std::cerr << "Engine not initialized" << std::endl;
            return nullptr;
        }
        
        auto shader = std::make_shared<RenderShader>(vertexShader, fragmentShader);
        if (shader->isCompiled()) {
            m_shaders.push_back(shader);
            return shader;
        }
        
        return nullptr;
    }
    
    std::shared_ptr<RenderTarget> createRenderTarget(int width, int height, const std::string& format) {
        if (!m_initialized) {
            std::cerr << "Engine not initialized" << std::endl;
            return nullptr;
        }
        
        auto target = std::make_shared<RenderTarget>(width, height, format);
        m_renderTargets.push_back(target);
        
        // Update GPU memory usage
        m_gpuMemoryUsage += calculateTextureMemoryUsage(width, height, format);
        m_gpuMemoryUsage += calculateTextureMemoryUsage(width, height, "depth24");
        
        return target;
    }
    
    bool applyEffect(std::shared_ptr<RenderSurface> surface, const EffectParams& effect) {
        if (!m_initialized || !surface) {
            return false;
        }
        
        switch (effect.type) {
            case EffectType::Blur:
                return applyBlurEffect(surface, effect.radius);
            case EffectType::Shadow:
                return applyShadowEffect(surface, effect.offsetX, effect.offsetY, effect.radius, effect.color);
            case EffectType::Neon:
                return applyNeonEffect(surface, effect.radius, effect.color, effect.intensity);
            case EffectType::Reflection:
                return applyReflectionEffect(surface, effect.intensity, 
                    effect.customParams.count("fadeDistance") ? effect.customParams.at("fadeDistance") : 0.5f);
            case EffectType::Distortion:
                // Implement distortion effect
                return applyDistortionEffect(surface, effect.intensity, 
                    effect.customParams.count("time") ? effect.customParams.at("time") : 0.0f);
            case EffectType::ColorAdjustment:
                // Implement color adjustment effect
                float brightness = 1.0f;
                float contrast = 1.0f;
                float saturation = 1.0f;
                float hue = 0.0f;
                
                if (effect.customParams.count("brightness")) brightness = effect.customParams.at("brightness");
                if (effect.customParams.count("contrast")) contrast = effect.customParams.at("contrast");
                if (effect.customParams.count("saturation")) saturation = effect.customParams.at("saturation");
                if (effect.customParams.count("hue")) hue = effect.customParams.at("hue");
                
                return applyColorAdjustmentEffect(surface, brightness, contrast, saturation, hue);
            default:
                return false;
        }
    }
    
    bool applyBlurEffect(std::shared_ptr<RenderSurface> surface, float radius) {
        if (!m_initialized || !surface) {
            return false;
        }
        
        // Get blur shader
        auto shaderIt = m_shadersByName.find("blur");
        if (shaderIt == m_shadersByName.end()) {
            std::cerr << "Blur shader not found" << std::endl;
            return false;
        }
        
        auto shader = shaderIt->second;
        
        // Create temporary render target for ping-pong rendering
        auto tempTarget = createRenderTarget(surface->getWidth(), surface->getHeight());
        if (!tempTarget) {
            std::cerr << "Failed to create temporary render target" << std::endl;
            return false;
        }
        
        // Bind the shader
        shader->bind();
        
        // Set common uniforms
        shader->setUniformVec2("u_resolution", static_cast<float>(surface->getWidth()), 
                             static_cast<float>(surface->getHeight()));
        shader->setUniformFloat("u_radius", radius);
        
        // First pass: horizontal blur
        tempTarget->bind();
        glClear(GL_COLOR_BUFFER_BIT);
        
        // Bind the surface texture
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, surface->getTextureId());
        shader->setUniformInt("u_texture", 0);
        shader->setUniformVec2("u_direction", 1.0f, 0.0f);
        
        // Draw fullscreen quad
        drawFullscreenQuad();
        
        // Second pass: vertical blur
        surface->bind();
        glClear(GL_COLOR_BUFFER_BIT);
        
        // Bind the temp texture
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, tempTarget->getColorTextureId());
        shader->setUniformInt("u_texture", 0);
        shader->setUniformVec2("u_direction", 0.0f, 1.0f);
        
        // Draw fullscreen quad
        drawFullscreenQuad();
        
        // Unbind
        shader->unbind();
        surface->unbind();
        
        m_drawCalls += 2;
        
        return true;
    }
    
    bool applyShadowEffect(std::shared_ptr<RenderSurface> surface, 
                          float offsetX, float offsetY, 
                          float blur, 
                          uint32_t color) {
        if (!m_initialized || !surface) {
            return false;
        }
        
        // Get shadow shader
        auto shaderIt = m_shadersByName.find("shadow");
        if (shaderIt == m_shadersByName.end()) {
            std::cerr << "Shadow shader not found" << std::endl;
            return false;
        }
        
        auto shader = shaderIt->second;
        
        // Create temporary render target
        auto tempTarget = createRenderTarget(surface->getWidth(), surface->getHeight());
        if (!tempTarget) {
            std::cerr << "Failed to create temporary render target" << std::endl;
            return false;
        }
        
        // Bind the shader
        shader->bind();
        
        // Set uniforms
        shader->setUniformVec2("u_resolution", static_cast<float>(surface->getWidth()), 
                             static_cast<float>(surface->getHeight()));
        shader->setUniformVec2("u_shadowOffset", offsetX, offsetY);
        
        // Convert color from RGBA8 to float [0,1]
        float r = ((color >> 24) & 0xFF) / 255.0f;
        float g = ((color >> 16) & 0xFF) / 255.0f;
        float b = ((color >> 8) & 0xFF) / 255.0f;
        float a = (color & 0xFF) / 255.0f;
        shader->setUniformVec4("u_shadowColor", r, g, b, a);
        
        shader->setUniformFloat("u_shadowBlur", blur);
        
        // Render to temp target
        tempTarget->bind();
        glClear(GL_COLOR_BUFFER_BIT);
        
        // Bind the surface texture
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, surface->getTextureId());
        shader->setUniformInt("u_texture", 0);
        
        // Draw fullscreen quad
        drawFullscreenQuad();
        
        // Copy result back to original surface
        surface->bind();
        
        // Use basic shader for simple copy
        auto basicShaderIt = m_shadersByName.find("basic");
        if (basicShaderIt != m_shadersByName.end()) {
            auto basicShader = basicShaderIt->second;
            basicShader->bind();
            
            // Bind the temp texture
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, tempTarget->getColorTextureId());
            basicShader->setUniformInt("u_texture", 0);
            
            // Draw fullscreen quad
            drawFullscreenQuad();
            
            basicShader->unbind();
        }
        
        shader->unbind();
        surface->unbind();
        
        m_drawCalls += 2;
        
        return true;
    }
    
    bool applyNeonEffect(std::shared_ptr<RenderSurface> surface, float radius, uint32_t color, float intensity) {
        if (!m_initialized || !surface) {
            return false;
        }
        
        // Get neon shader
        auto shaderIt = m_shadersByName.find("neon");
        if (shaderIt == m_shadersByName.end()) {
            std::cerr << "Neon shader not found" << std::endl;
            return false;
        }
        
        auto shader = shaderIt->second;

        // Create temporary render target
        auto tempTarget = createRenderTarget(surface->getWidth(), surface->getHeight(), "rgba8");
        if (!tempTarget) {
            std::cerr << "Failed to create temporary render target" << std::endl;
            return false;
        }

        // Bind the shader
        shader->bind();

        // Set uniforms
        shader->setUniformVec2("u_resolution", static_cast<float>(surface->getWidth()),
                               static_cast<float>(surface->getHeight()));
        shader->setUniformFloat("u_radius", radius);

        // Convert color from RGBA8 to float [0,1]
        float r = ((color >> 24) & 0xFF) / 255.0f;
        float g = ((color >> 16) & 0xFF) / 255.0f;
        float b = ((color >> 8) & 0xFF) / 255.0f;
        float a = (color & 0xFF) / 255.0f;
        shader->setUniformVec4("u_glowColor", r, g, b, a);
        shader->setUniformFloat("u_intensity", intensity);

        // Render to temp target
        tempTarget->bind();
        glClear(GL_COLOR_BUFFER_BIT);

        // Bind the surface texture
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, surface->getTextureId());
        shader->setUniformInt("u_texture", 0);

        // Draw fullscreen quad
        drawFullscreenQuad();

        // Copy result back to original surface
        surface->bind();

        // Use basic shader for simple copy
        auto basicShaderIt = m_shadersByName.find("basic");
        if (basicShaderIt != m_shadersByName.end()) {
            auto basicShader = basicShaderIt->second;
            basicShader->bind();

            // Bind the temp texture
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, tempTarget->getColorTextureId());
            basicShader->setUniformInt("u_texture", 0);

            // Draw fullscreen quad
            drawFullscreenQuad();

            basicShader->unbind();
        }

        shader->unbind();
        surface->unbind();

        m_drawCalls += 2;

        return true;
    }

    bool applyReflectionEffect(std::shared_ptr<RenderSurface> surface, float intensity, float fadeDistance) {
        if (!m_initialized || !surface) {
            return false;
        }

        // Get reflection shader
        auto shaderIt = m_shadersByName.find("reflection");
        if (shaderIt == m_shadersByName.end()) {
            std::cerr << "Reflection shader not found" << std::endl;
            return false;
        }

        auto shader = shaderIt->second;

        // Create temporary render target
        auto tempTarget = createRenderTarget(surface->getWidth(), surface->getHeight(), "rgba8");
        if (!tempTarget) {
            std::cerr << "Failed to create temporary render target" << std::endl;
            return false;
        }

        // Bind the shader
        shader->bind();

        // Set uniforms
        shader->setUniformFloat("u_intensity", intensity);
        shader->setUniformFloat("u_fadeDistance", fadeDistance);

        // Render to temp target
        tempTarget->bind();
        glClear(GL_COLOR_BUFFER_BIT);

        // Bind the surface texture
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, surface->getTextureId());
        shader->setUniformInt("u_texture", 0);

        // Draw fullscreen quad
        drawFullscreenQuad();

        // Copy result back to original surface
        surface->bind();

        // Use basic shader for simple copy
        auto basicShaderIt = m_shadersByName.find("basic");
        if (basicShaderIt != m_shadersByName.end()) {
            auto basicShader = basicShaderIt->second;
            basicShader->bind();

            // Bind the temp texture
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, tempTarget->getColorTextureId());
            basicShader->setUniformInt("u_texture", 0);

            // Draw fullscreen quad
            drawFullscreenQuad();

            basicShader->unbind();
        }

        shader->unbind();
        surface->unbind();

        m_drawCalls += 2;

        return true;
    }

    bool applyDistortionEffect(std::shared_ptr<RenderSurface> surface, float intensity, float time) {
        if (!m_initialized || !surface) {
            return false;
        }

        // Get distortion shader
        auto shaderIt = m_shadersByName.find("distortion");
        if (shaderIt == m_shadersByName.end()) {
            std::cerr << "Distortion shader not found" << std::endl;
            return false;
        }

        auto shader = shaderIt->second;

        // Create temporary render target
        auto tempTarget = createRenderTarget(surface->getWidth(), surface->getHeight(), "rgba8");
        if (!tempTarget) {
            std::cerr << "Failed to create temporary render target" << std::endl;
            return false;
        }

        // Bind the shader
        shader->bind();

        // Set uniforms
        shader->setUniformVec2("u_resolution", static_cast<float>(surface->getWidth()),
                               static_cast<float>(surface->getHeight()));
        shader->setUniformFloat("u_intensity", intensity);
        shader->setUniformFloat("u_time", time);

        // Render to temp target
        tempTarget->bind();
        glClear(GL_COLOR_BUFFER_BIT);

        // Bind the surface texture
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, surface->getTextureId());
        shader->setUniformInt("u_texture", 0);

        // Draw fullscreen quad
        drawFullscreenQuad();

        // Copy result back to original surface
        surface->bind();

        // Use basic shader for simple copy
        auto basicShaderIt = m_shadersByName.find("basic");
        if (basicShaderIt != m_shadersByName.end()) {
            auto basicShader = basicShaderIt->second;
            basicShader->bind();

            // Bind the temp texture
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, tempTarget->getColorTextureId());
            basicShader->setUniformInt("u_texture", 0);

            // Draw fullscreen quad
            drawFullscreenQuad();

            basicShader->unbind();
        }

        shader->unbind();
        surface->unbind();

        m_drawCalls += 2;

        return true;
    }

    bool applyColorAdjustmentEffect(std::shared_ptr<RenderSurface> surface,
                                    float brightness, float contrast,
                                    float saturation, float hue) {
        if (!m_initialized || !surface) {
            return false;
        }

        // Get color adjustment shader
        auto shaderIt = m_shadersByName.find("colorAdjustment");
        if (shaderIt == m_shadersByName.end()) {
            std::cerr << "Color adjustment shader not found" << std::endl;
            return false;
        }

        auto shader = shaderIt->second;

        // Create temporary render target
        auto tempTarget = createRenderTarget(surface->getWidth(), surface->getHeight(), "rgba8");
        if (!tempTarget) {
            std::cerr << "Failed to create temporary render target" << std::endl;
            return false;
        }

        // Bind the shader
        shader->bind();

        // Set uniforms
        shader->setUniformFloat("u_brightness", brightness);
        shader->setUniformFloat("u_contrast", contrast);
        shader->setUniformFloat("u_saturation", saturation);
        shader->setUniformFloat("u_hue", hue);

        // Render to temp target
        tempTarget->bind();
        glClear(GL_COLOR_BUFFER_BIT);

        // Bind the surface texture
        glActiveTexture(GL_TEXTURE0);
        glBindTexture(GL_TEXTURE_2D, surface->getTextureId());
        shader->setUniformInt("u_texture", 0);

        // Draw fullscreen quad
        drawFullscreenQuad();

        // Copy result back to original surface
        surface->bind();

        // Use basic shader for simple copy
        auto basicShaderIt = m_shadersByName.find("basic");
        if (basicShaderIt != m_shadersByName.end()) {
            auto basicShader = basicShaderIt->second;
            basicShader->bind();

            // Bind the temp texture
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, tempTarget->getColorTextureId());
            basicShader->setUniformInt("u_texture", 0);

            // Draw fullscreen quad
            drawFullscreenQuad();

            basicShader->unbind();
        }

        shader->unbind();
        surface->unbind();

        m_drawCalls += 2;

        return true;
                                    }

                                    // Render statistics and performance methods
                                    int getFrameRate() const {
                                        return m_currentFrameRate;
                                    }

                                    float getFrameTime() const {
                                        return m_frameTime;
                                    }

                                    int getDrawCalls() const {
                                        return m_drawCalls;
                                    }

                                    uint64_t getGPUMemoryUsage() const {
                                        return m_gpuMemoryUsage;
                                    }

                                    void setVSync(bool enable) {
                                        m_vSync = enable;

                                        // Apply vsync settings based on backend
                                        if (m_currentBackend == "opengl") {
                                            // For EGL
                                            eglSwapInterval(m_eglDisplay, enable ? 1 : 0);
                                        } else if (m_currentBackend == "vulkan") {
                                            // Vulkan needs to recreate the swapchain
                                            // This is a placeholder for actual implementation
                                        }
                                    }

                                    bool isVSyncEnabled() const {
                                        return m_vSync;
                                    }

                                    void setMaxFrameRate(int frameRate) {
                                        m_maxFrameRate = frameRate;
                                    }

                                    int getMaxFrameRate() const {
                                        return m_maxFrameRate;
                                    }

private:
    // Private helper methods for initialization and rendering
    bool checkHardwareAcceleration() {
        // Check for OpenGL support
        EGLDisplay testDisplay = eglGetDisplay(EGL_DEFAULT_DISPLAY);
        if (testDisplay == EGL_NO_DISPLAY) {
            return false;
        }

        EGLint major, minor;
        if (eglInitialize(testDisplay, &major, &minor) != EGL_TRUE) {
            return false;
        }

        // Get number of configs
        EGLint configCount;
        if (eglGetConfigs(testDisplay, nullptr, 0, &configCount) != EGL_TRUE || configCount == 0) {
            eglTerminate(testDisplay);
            return false;
        }

        eglTerminate(testDisplay);
        return true;
    }

    bool initializeOpenGL() {
        // Initialize EGL
        m_eglDisplay = eglGetDisplay(EGL_DEFAULT_DISPLAY);
        if (m_eglDisplay == EGL_NO_DISPLAY) {
            std::cerr << "Could not get EGL display" << std::endl;
            return false;
        }

        EGLint major, minor;
        if (eglInitialize(m_eglDisplay, &major, &minor) != EGL_TRUE) {
            std::cerr << "Could not initialize EGL display" << std::endl;
            return false;
        }

        std::cout << "EGL version: " << major << "." << minor << std::endl;

        // Choose configuration
        eglBindAPI(EGL_OPENGL_ES_API);

        EGLint configAttributes[] = {
            EGL_SURFACE_TYPE, EGL_PBUFFER_BIT,
            EGL_RENDERABLE_TYPE, EGL_OPENGL_ES3_BIT,
            EGL_RED_SIZE, 8,
            EGL_GREEN_SIZE, 8,
            EGL_BLUE_SIZE, 8,
            EGL_ALPHA_SIZE, 8,
            EGL_DEPTH_SIZE, 24,
            EGL_STENCIL_SIZE, 8,
            EGL_NONE
        };

        EGLConfig config;
        EGLint numConfigs;
        if (eglChooseConfig(m_eglDisplay, configAttributes, &config, 1, &numConfigs) != EGL_TRUE || numConfigs == 0) {
            std::cerr << "Could not choose EGL configuration" << std::endl;
            eglTerminate(m_eglDisplay);
            return false;
        }

        // Create a surface (for offscreen rendering, we use a PBuffer)
        EGLint surfaceAttributes[] = {
            EGL_WIDTH, 1,
            EGL_HEIGHT, 1,
            EGL_NONE
        };

        m_eglSurface = eglCreatePbufferSurface(m_eglDisplay, config, surfaceAttributes);
        if (m_eglSurface == EGL_NO_SURFACE) {
            std::cerr << "Could not create EGL surface" << std::endl;
            eglTerminate(m_eglDisplay);
            return false;
        }

        // Create OpenGL ES 3.0 context
        EGLint contextAttributes[] = {
            EGL_CONTEXT_CLIENT_VERSION, 3,
            EGL_NONE
        };

        m_eglContext = eglCreateContext(m_eglDisplay, config, EGL_NO_CONTEXT, contextAttributes);
        if (m_eglContext == EGL_NO_CONTEXT) {
            std::cerr << "Could not create EGL context" << std::endl;
            eglDestroySurface(m_eglDisplay, m_eglSurface);
            eglTerminate(m_eglDisplay);
            return false;
        }

        // Make the context current
        if (eglMakeCurrent(m_eglDisplay, m_eglSurface, m_eglSurface, m_eglContext) != EGL_TRUE) {
            std::cerr << "Could not make EGL context current" << std::endl;
            eglDestroyContext(m_eglDisplay, m_eglContext);
            eglDestroySurface(m_eglDisplay, m_eglSurface);
            eglTerminate(m_eglDisplay);
            return false;
        }

        // Set vsync
        eglSwapInterval(m_eglDisplay, m_vSync ? 1 : 0);

        // Initialize OpenGL resources
        initializeOpenGLResources();

        return true;
    }

    void initializeOpenGLResources() {
        // Generate VAO
        glGenVertexArrays(1, &m_vao);
        glBindVertexArray(m_vao);

        // Generate VBO and upload vertex data
        glGenBuffers(1, &m_vbo);
        glBindBuffer(GL_ARRAY_BUFFER, m_vbo);
        glBufferData(GL_ARRAY_BUFFER, sizeof(quadVertices), quadVertices, GL_STATIC_DRAW);

        // Vertex position attribute
        glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, sizeof(Vertex), (void*)offsetof(Vertex, x));
        glEnableVertexAttribArray(0);

        // Texture coordinate attribute
        glVertexAttribPointer(1, 2, GL_FLOAT, GL_FALSE, sizeof(Vertex), (void*)offsetof(Vertex, u));
        glEnableVertexAttribArray(1);

        // Generate IBO and upload index data
        glGenBuffers(1, &m_ibo);
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, m_ibo);
        glBufferData(GL_ELEMENT_ARRAY_BUFFER, sizeof(quadIndices), quadIndices, GL_STATIC_DRAW);

        // Reset state
        glBindVertexArray(0);
        glBindBuffer(GL_ARRAY_BUFFER, 0);
        glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, 0);

        // Enable blending
        glEnable(GL_BLEND);
        glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
    }

    void drawFullscreenQuad() {
        // Bind VAO
        glBindVertexArray(m_vao);

        // Draw the quad
        glDrawElements(GL_TRIANGLES, 6, GL_UNSIGNED_SHORT, nullptr);

        // Unbind VAO
        glBindVertexArray(0);
    }

    bool initializeVulkan() {
        // This is a placeholder for the actual Vulkan initialization
        // In a real implementation, we would:
        // 1. Create a Vulkan instance
        // 2. Select a physical device
        // 3. Create a logical device with appropriate queues
        // 4. Create a swapchain
        // 5. Create render passes, framebuffers, etc.

        std::cerr << "Vulkan backend not fully implemented yet" << std::endl;
        return false;
    }

    bool initializeSoftwareRendering() {
        // Create a software rendering context
        // This implementation uses a simple buffer for software rendering

        // Create an EGL context with software rendering
        m_eglDisplay = eglGetDisplay(EGL_DEFAULT_DISPLAY);
        if (m_eglDisplay == EGL_NO_DISPLAY) {
            std::cerr << "Could not get EGL display" << std::endl;
            return false;
        }

        EGLint major, minor;
        if (eglInitialize(m_eglDisplay, &major, &minor) != EGL_TRUE) {
            std::cerr << "Could not initialize EGL display" << std::endl;
            return false;
        }

        // Choose configuration with software rendering
        eglBindAPI(EGL_OPENGL_ES_API);

        EGLint configAttributes[] = {
            EGL_SURFACE_TYPE, EGL_PBUFFER_BIT,
            EGL_RENDERABLE_TYPE, EGL_OPENGL_ES3_BIT,
            EGL_RED_SIZE, 8,
            EGL_GREEN_SIZE, 8,
            EGL_BLUE_SIZE, 8,
            EGL_ALPHA_SIZE, 8,
            EGL_NONE
        };

        EGLConfig config;
        EGLint numConfigs;
        if (eglChooseConfig(m_eglDisplay, configAttributes, &config, 1, &numConfigs) != EGL_TRUE || numConfigs == 0) {
            std::cerr << "Could not choose EGL configuration" << std::endl;
            eglTerminate(m_eglDisplay);
            return false;
        }

        // Create a surface
        EGLint surfaceAttributes[] = {
            EGL_WIDTH, 1,
            EGL_HEIGHT, 1,
            EGL_NONE
        };

        m_eglSurface = eglCreatePbufferSurface(m_eglDisplay, config, surfaceAttributes);
        if (m_eglSurface == EGL_NO_SURFACE) {
            std::cerr << "Could not create EGL surface" << std::endl;
            eglTerminate(m_eglDisplay);
            return false;
        }

        // Create software context
        EGLint contextAttributes[] = {
            EGL_CONTEXT_CLIENT_VERSION, 3,
            EGL_NONE
        };

        m_eglContext = eglCreateContext(m_eglDisplay, config, EGL_NO_CONTEXT, contextAttributes);
        if (m_eglContext == EGL_NO_CONTEXT) {
            std::cerr << "Could not create EGL context" << std::endl;
            eglDestroySurface(m_eglDisplay, m_eglSurface);
            eglTerminate(m_eglDisplay);
            return false;
        }

        // Make the context current
        if (eglMakeCurrent(m_eglDisplay, m_eglSurface, m_eglSurface, m_eglContext) != EGL_TRUE) {
            std::cerr << "Could not make EGL context current" << std::endl;
            eglDestroyContext(m_eglDisplay, m_eglContext);
            eglDestroySurface(m_eglDisplay, m_eglSurface);
            eglTerminate(m_eglDisplay);
            return false;
        }

        // Initialize OpenGL resources (same as for hardware rendering)
        initializeOpenGLResources();

        return true;
    }

    void cleanupOpenGL() {
        if (m_eglDisplay != EGL_NO_DISPLAY) {
            // Delete OpenGL resources
            if (m_vao) {
                glDeleteVertexArrays(1, &m_vao);
                m_vao = 0;
            }

            if (m_vbo) {
                glDeleteBuffers(1, &m_vbo);
                m_vbo = 0;
            }

            if (m_ibo) {
                glDeleteBuffers(1, &m_ibo);
                m_ibo = 0;
            }

            // Release EGL context
            eglMakeCurrent(m_eglDisplay, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);

            if (m_eglContext != EGL_NO_CONTEXT) {
                eglDestroyContext(m_eglDisplay, m_eglContext);
                m_eglContext = EGL_NO_CONTEXT;
            }

            if (m_eglSurface != EGL_NO_SURFACE) {
                eglDestroySurface(m_eglDisplay, m_eglSurface);
                m_eglSurface = EGL_NO_SURFACE;
            }

            eglTerminate(m_eglDisplay);
            m_eglDisplay = EGL_NO_DISPLAY;
        }
    }

    void cleanupVulkan() {
        // Cleanup Vulkan resources
        // This is a placeholder for actual Vulkan cleanup
    }

    void cleanupSoftwareRendering() {
        // Software rendering uses the same EGL context, so we reuse the OpenGL cleanup
        cleanupOpenGL();
    }

    void createShaders() {
        // Create basic shader
        auto basicVertexShader = Shaders::basicVertexShader;
        auto basicFragmentShader = Shaders::basicFragmentShader;
        auto basicShader = createShader(basicVertexShader, basicFragmentShader);
        if (basicShader) {
            m_shadersByName["basic"] = basicShader;
        }

        // Create blur shader
        auto blurShader = createShader(basicVertexShader, Shaders::blurFragmentShader);
        if (blurShader) {
            m_shadersByName["blur"] = blurShader;
        }

        // Create shadow shader
        auto shadowShader = createShader(basicVertexShader, Shaders::shadowFragmentShader);
        if (shadowShader) {
            m_shadersByName["shadow"] = shadowShader;
        }

        // Create neon shader
        auto neonShader = createShader(basicVertexShader, Shaders::neonFragmentShader);
        if (neonShader) {
            m_shadersByName["neon"] = neonShader;
        }

        // Create reflection shader
        auto reflectionShader = createShader(basicVertexShader, Shaders::reflectionFragmentShader);
        if (reflectionShader) {
            m_shadersByName["reflection"] = reflectionShader;
        }

        // Create distortion shader
        auto distortionShader = createShader(basicVertexShader, Shaders::distortionFragmentShader);
        if (distortionShader) {
            m_shadersByName["distortion"] = distortionShader;
        }

        // Create color adjustment shader
        auto colorAdjustmentShader = createShader(basicVertexShader, Shaders::colorAdjustmentFragmentShader);
        if (colorAdjustmentShader) {
            m_shadersByName["colorAdjustment"] = colorAdjustmentShader;
        }
    }

    void presentVulkanFrame() {
        // Vulkan frame presentation logic
        // This is a placeholder for actual Vulkan present implementation
    }

    void presentSoftwareFrame() {
        // Software rendering frame presentation
        // In a real implementation, this would copy the software-rendered buffer to a display
        // or window system

        // For now, we just use EGL swapbuffers
        eglSwapBuffers(m_eglDisplay, m_eglSurface);
    }

    void updateStats() {
        // Update performance statistics
        // This is called every frame to collect metrics

        // Calculate GPU memory usage from surfaces, textures, and render targets
        m_gpuMemoryUsage = 0;

        // Calculate memory for surfaces
        for (const auto& surface : m_surfaces) {
            if (surface) {
                m_gpuMemoryUsage += calculateTextureMemoryUsage(
                    surface->getWidth(), surface->getHeight(), surface->getFormat());
            }
        }

        // Calculate memory for textures
        for (const auto& texture : m_textures) {
            if (texture) {
                m_gpuMemoryUsage += calculateTextureMemoryUsage(
                    texture->getWidth(), texture->getHeight(), texture->getFormat());
            }
        }

        // Calculate memory for render targets (color + depth)
        for (const auto& target : m_renderTargets) {
            if (target) {
                m_gpuMemoryUsage += calculateTextureMemoryUsage(
                    target->getWidth(), target->getHeight(), target->getColorFormat());
                m_gpuMemoryUsage += calculateTextureMemoryUsage(
                    target->getWidth(), target->getHeight(), "depth24");
            }
        }
    }

    uint64_t calculateTextureMemoryUsage(int width, int height, const std::string& format) {
        // Calculate memory usage for a texture based on its format
        uint64_t bytesPerPixel = 4; // Default to 4 bytes per pixel (RGBA8)

        if (format == "rgb8") {
            bytesPerPixel = 3;
        } else if (format == "rgba8") {
            bytesPerPixel = 4;
        } else if (format == "rgb16f") {
            bytesPerPixel = 6;
        } else if (format == "rgba16f") {
            bytesPerPixel = 8;
        } else if (format == "r8") {
            bytesPerPixel = 1;
        } else if (format == "rg8") {
            bytesPerPixel = 2;
        } else if (format == "depth24") {
            bytesPerPixel = 3;
        } else if (format == "depth32f") {
            bytesPerPixel = 4;
        }

        return static_cast<uint64_t>(width) * static_cast<uint64_t>(height) * bytesPerPixel;
    }

    // Member variables
    bool m_initialized;
    bool m_hardwareAccelerationEnabled;
    bool m_hardwareAccelerationAvailable;
    bool m_vSync;
    int m_maxFrameRate;
    int m_currentFrameRate;
    float m_frameTime;
    uint64_t m_frameCount;
    int m_drawCalls;

    // EGL variables
    EGLDisplay m_eglDisplay;
    EGLContext m_eglContext;
    EGLSurface m_eglSurface;

    // OpenGL resources
    GLuint m_vao;
    GLuint m_vbo;
    GLuint m_ibo;

    // Backend info
    std::string m_currentBackend;
    std::vector<std::string> m_availableBackends;

    // Rendering state
    bool m_isRunning;
    std::chrono::high_resolution_clock::time_point m_lastFrameTime;
    uint64_t m_gpuMemoryUsage;

    // Resources
    std::vector<std::shared_ptr<RenderSurface>> m_surfaces;
    std::vector<std::shared_ptr<RenderTexture>> m_textures;
    std::vector<std::shared_ptr<RenderShader>> m_shaders;
    std::vector<std::shared_ptr<RenderTarget>> m_renderTargets;
    std::map<std::string, std::shared_ptr<RenderShader>> m_shadersByName;
};

// Public methods implementation that delegate to the impl
RenderEngine::RenderEngine()
: m_pImpl(new Impl()) {
}

RenderEngine::~RenderEngine() {
    delete m_pImpl;
}

bool RenderEngine::initialize(bool useHardwareAcceleration, const std::string& preferredBackend) {
    return m_pImpl->initialize(useHardwareAcceleration, preferredBackend);
}

void RenderEngine::shutdown() {
    m_pImpl->shutdown();
}

bool RenderEngine::isInitialized() const {
    return m_pImpl->isInitialized();
}

bool RenderEngine::isHardwareAccelerationAvailable() const {
    return m_pImpl->isHardwareAccelerationAvailable();
}

bool RenderEngine::isHardwareAccelerationEnabled() const {
    return m_pImpl->isHardwareAccelerationEnabled();
}

bool RenderEngine::setHardwareAcceleration(bool enable) {
    return m_pImpl->setHardwareAcceleration(enable);
}

std::string RenderEngine::getCurrentBackend() const {
    return m_pImpl->getCurrentBackend();
}

bool RenderEngine::setBackend(const std::string& backend) {
    return m_pImpl->setBackend(backend);
}

std::vector<std::string> RenderEngine::getAvailableBackends() const {
    return m_pImpl->getAvailableBackends();
}

void RenderEngine::start() {
    m_pImpl->start();
}
void RenderEngine::stop() {
    m_pImpl->stop();
}

bool RenderEngine::isRunning() const {
    return m_pImpl->isRunning();
}

bool RenderEngine::beginFrame() {
    return m_pImpl->beginFrame();
}

bool RenderEngine::endFrame() {
    return m_pImpl->endFrame();
}

std::shared_ptr<RenderSurface> RenderEngine::createSurface(int width, int height, const std::string& format) {
    return m_pImpl->createSurface(width, height, format);
}

std::shared_ptr<RenderTexture> RenderEngine::createTexture(int width, int height, const std::string& format) {
    return m_pImpl->createTexture(width, height, format);
}

std::shared_ptr<RenderShader> RenderEngine::createShader(const std::string& vertexShader, const std::string& fragmentShader) {
    return m_pImpl->createShader(vertexShader, fragmentShader);
}

std::shared_ptr<RenderTarget> RenderEngine::createRenderTarget(int width, int height, const std::string& format) {
    return m_pImpl->createRenderTarget(width, height, format);
}

bool RenderEngine::applyEffect(std::shared_ptr<RenderSurface> surface, const EffectParams& effect) {
    return m_pImpl->applyEffect(surface, effect);
}

int RenderEngine::getFrameRate() const {
    return m_pImpl->getFrameRate();
}

float RenderEngine::getFrameTime() const {
    return m_pImpl->getFrameTime();
}

int RenderEngine::getDrawCalls() const {
    return m_pImpl->getDrawCalls();
}

uint64_t RenderEngine::getGPUMemoryUsage() const {
    return m_pImpl->getGPUMemoryUsage();
}

void RenderEngine::setVSync(bool enable) {
    m_pImpl->setVSync(enable);
}

bool RenderEngine::isVSyncEnabled() const {
    return m_pImpl->isVSyncEnabled();
}

void RenderEngine::setMaxFrameRate(int frameRate) {
    m_pImpl->setMaxFrameRate(frameRate);
}

int RenderEngine::getMaxFrameRate() const {
    return m_pImpl->getMaxFrameRate();
}

} // namespace Rendering
} // namespace Compositor
} // namespace VivoX
