
// RenderTarget.cpp
#include "RenderTarget.h"
#include <GL/gl.h>
#include <iostream>

namespace VivoX {
    namespace Compositor {
        namespace Rendering {

            RenderTarget::RenderTarget(int width, int height, const std::string& format)
            : m_width(width)
            , m_height(height)
            , m_colorTextureId(0)
            , m_depthTextureId(0)
            , m_framebufferId(0)
            , m_format(format) {

                // Create framebuffer
                glGenFramebuffers(1, &m_framebufferId);
                glBindFramebuffer(GL_FRAMEBUFFER, m_framebufferId);

                // Create color texture
                glGenTextures(1, &m_colorTextureId);
                glBindTexture(GL_TEXTURE_2D, m_colorTextureId);

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

                // Attach color texture to framebuffer
                glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, m_colorTextureId, 0);

                // Create depth texture
                glGenTextures(1, &m_depthTextureId);
                glBindTexture(GL_TEXTURE_2D, m_depthTextureId);

                // Set texture parameters
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);

                // Allocate depth texture storage
                glTexImage2D(GL_TEXTURE_2D, 0, GL_DEPTH_COMPONENT24, width, height, 0, GL_DEPTH_COMPONENT, GL_FLOAT, nullptr);

                // Attach depth texture to framebuffer
                glFramebufferTexture2D(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_TEXTURE_2D, m_depthTextureId, 0);

                // Set draw buffers
                GLenum drawBuffers[1] = { GL_COLOR_ATTACHMENT0 };
                glDrawBuffers(1, drawBuffers);

                // Check framebuffer status
                GLenum status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
                if (status != GL_FRAMEBUFFER_COMPLETE) {
                    std::cerr << "Framebuffer incomplete: " << status << std::endl;
                }

                // Unbind framebuffer
                glBindFramebuffer(GL_FRAMEBUFFER, 0);
            }

            RenderTarget::~RenderTarget() {
                if (m_framebufferId) {
                    glDeleteFramebuffers(1, &m_framebufferId);
                    m_framebufferId = 0;
                }

                if (m_colorTextureId) {
                    glDeleteTextures(1, &m_colorTextureId);
                    m_colorTextureId = 0;
                }

                if (m_depthTextureId) {
                    glDeleteTextures(1, &m_depthTextureId);
                    m_depthTextureId = 0;
                }
            }

            void RenderTarget::bind() {
                glBindFramebuffer(GL_FRAMEBUFFER, m_framebufferId);
                glViewport(0, 0, m_width, m_height);
            }

            void RenderTarget::unbind() {
                glBindFramebuffer(GL_FRAMEBUFFER, 0);
            }

            void RenderTarget::clear(float r, float g, float b, float a) {
                bind();
                glClearColor(r, g, b, a);
                glClear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
                unbind();
            }

            void RenderTarget::resize(int width, int height) {
                if (width == m_width && height == m_height) {
                    return;
                }

                m_width = width;
                m_height = height;

                // Resize color texture
                glBindTexture(GL_TEXTURE_2D, m_colorTextureId);

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

                // Resize depth texture
                glBindTexture(GL_TEXTURE_2D, m_depthTextureId);
                glTexImage2D(GL_TEXTURE_2D, 0, GL_DEPTH_COMPONENT24, width, height, 0, GL_DEPTH_COMPONENT, GL_FLOAT, nullptr);

                // Ensure framebuffer is complete
                glBindFramebuffer(GL_FRAMEBUFFER, m_framebufferId);
                glFramebufferTexture2D(GL_FRAMEBUFFER, GL_COLOR_ATTACHMENT0, GL_TEXTURE_2D, m_colorTextureId, 0);
                glFramebufferTexture2D(GL_FRAMEBUFFER, GL_DEPTH_ATTACHMENT, GL_TEXTURE_2D, m_depthTextureId, 0);

                GLenum status = glCheckFramebufferStatus(GL_FRAMEBUFFER);
                if (status != GL_FRAMEBUFFER_COMPLETE) {
                    std::cerr << "Framebuffer incomplete after resize: " << status << std::endl;
                }

                glBindFramebuffer(GL_FRAMEBUFFER, 0);
            }

            uint32_t RenderTarget::getColorTextureId() const {
                return m_colorTextureId;
            }

            uint32_t RenderTarget::getDepthTextureId() const {
                return m_depthTextureId;
            }

            uint32_t RenderTarget::getFramebufferId() const {
                return m_framebufferId;
            }

            void RenderTarget::bindColorTexture(int textureUnit) {
                glActiveTexture(GL_TEXTURE0 + textureUnit);
                glBindTexture(GL_TEXTURE_2D, m_colorTextureId);
            }

            void RenderTarget::bindDepthTexture(int textureUnit) {
                glActiveTexture(GL_TEXTURE0 + textureUnit);
                glBindTexture(GL_TEXTURE_2D, m_depthTextureId);
            }

            void RenderTarget::unbindTexture(int textureUnit) {
                glActiveTexture(GL_TEXTURE0 + textureUnit);
                glBindTexture(GL_TEXTURE_2D, 0);
            }

        } // namespace Rendering
    } // namespace Compositor
} // namespace VivoX
