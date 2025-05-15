// RenderSurface.h
#pragma once

#include <cstdint>
#include <string>

namespace VivoX {
namespace Compositor {
namespace Rendering {

class RenderSurface {
public:
    RenderSurface(int width, int height, const std::string& format = "rgba8");
    ~RenderSurface();
    
    int getWidth() const { return m_width; }
    int getHeight() const { return m_height; }
    uint32_t getTextureId() const;
    uint32_t getFramebufferId() const;
    
    void bind();
    void unbind();
    void clear(float r = 0.0f, float g = 0.0f, float b = 0.0f, float a = 1.0f);
    void resize(int width, int height);
    
private:
    int m_width;
    int m_height;
    uint32_t m_textureId;
    uint32_t m_framebufferId;
    std::string m_format;
};

} // namespace Rendering
} // namespace Compositor
} // namespace VivoX
