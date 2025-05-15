// RenderTexture.h
#pragma once

#include <cstdint>
#include <string>

namespace VivoX {
namespace Compositor {
namespace Rendering {

class RenderTexture {
public:
    RenderTexture(int width, int height, const std::string& format = "rgba8");
    ~RenderTexture();

    int getWidth() const { return m_width; }
    int getHeight() const { return m_height; }
    uint32_t getTextureId() const { return m_textureId; }

    void bind(int textureUnit = 0);
    void unbind(int textureUnit = 0);
    void upload(const void* data, const std::string& format = "rgba");
    void resize(int width, int height);

private:
    int m_width;
    int m_height;
    uint32_t m_textureId;
    std::string m_format;
};


} // namespace Rendering
} // namespace Compositor
} // namespace VivoX
