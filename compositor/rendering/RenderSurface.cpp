// RenderSurface.cpp
#include "RenderSurface.h"
#include <GL/gl.h>
#include <iostream>

namespace VivoX {
namespace Compositor {
namespace Rendering {

RenderSurface::RenderSurface(int width, int height, const std::string& format)
    : m_width(width)
    , m_height(height)
    , m_textureId(0)
    , m_framebufferId(0)
    , m_format(format) {
    
    // Create texture
    glGenTextures(1, &m_textureId);
    glBindTexture(GL_TEXTURE_2D, m_textureId);
    
    // Set texture parameters
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_LINEAR);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
    
    // Allocate texture storage based on format
    if (format == "rgba8") {
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);
    } else if (format == "rgb8") {
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB8, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, nullptr);
    } else if (format == "rgba16f") {
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA16F, width, height, 0, GL_RGBA, GL_FLOAT, nullptr);
    } else if (format == "r8") {
        glTexImage2D(GL_TEXTURE_2D, 0, GL_R8, width, height, 0, GL_RED, GL_UNSIGNED_BYTE, nullptr);
    } else {
        // Default to RGBA8
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);
    }
    
    // Create framebuffer
    glGenFramebuffers(1, &m_framebufferId);
    glBindFramebuffer(GL_FRAMEBUFFER, m_framebufferId);
    
    // Attach texture to framebuffer
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, m_textureId, 0);
    
    // Check framebuffer status
    GLenum status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
    if (status != GL_FRAMEBUFFER_COMPLETE) {
        std::cerr << "Framebuffer incomplete: " << status << std::endl;
    }
    
    // Unbind framebuffer
    glBindFramebuffer(GL_FRAMEBUFFER, 0);
}

RenderSurface::~RenderSurface() {
    // Cleanup resources
    if (m_framebufferId) {
        glDeleteFramebuffers(1, &m_framebufferId);
        m_framebufferId = 0;
    }
    
    if (m_textureId) {
        glDeleteTextures(1, &m_textureId);
        m_textureId = 0;
    }
}

void RenderSurface::bind() {
    glBindFramebuffer(GL_FRAMEBUFFER, m_framebufferId);
    glViewport(0, 0, m_width, m_height);
}

void RenderSurface::unbind() {
    glBindFramebuffer(GL_FRAMEBUFFER, 0);
}

void RenderSurface::clear(float r, float g, float b, float a) {
    bind();
    glClearColor(r, g, b, a);
    glClear(GL_COLOR_BUFFER_BIT);
    unbind();
}

void RenderSurface::resize(int width, int height) {
    if (width == m_width && height == m_height) {
        return;
    }
    
    m_width = width;
    m_height = height;
    
    // Resize texture
    glBindTexture(GL_TEXTURE_2D, m_textureId);
    
    // Reallocate texture storage based on format
    if (m_format == "rgba8") {
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);
    } else if (m_format == "rgb8") {
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB8, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, nullptr);
    } else if (m_format == "rgba16f") {
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA16F, width, height, 0, GL_RGBA, GL_FLOAT, nullptr);
    } else if (m_format == "r8") {
        glTexImage2D(GL_TEXTURE_2D, 0, GL_R8, width, height, 0, GL_RED, GL_UNSIGNED_BYTE, nullptr);
    } else {
        // Default to RGBA8
        glTexImage2D(GL_TEXTURE_2D, 0, GL_RGBA8, width, height, 0, GL_RGBA, GL_UNSIGNED_BYTE, nullptr);
    }
    
    // Ensure framebuffer is complete
    glBindFramebuffer(GL_FRAMEBUFFER, m_framebufferId);
    glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, m_textureId, 0);
    
    GLenum status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
    if (status != GL_FRAMEBUFFER_COMPLETE) {
        std::cerr << "Framebuffer incomplete after resize: " << status << std::endl;
    }
    
    glBindFramebuffer(GL_FRAMEBUFFER, 0);
}

uint32_t RenderSurface::getTextureId() const {
    return m_textureId;
}

uint32_t RenderSurface::getFramebufferId() const {
    return m_framebufferId;
}


} // namespace Rendering
} // namespace Compositor
} // namespace VivoX
