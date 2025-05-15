// RenderTexture.cpp
#include "RenderTexture.h"
#include <GL/gl.h>
#include <iostream>

namespace VivoX {
    namespace Compositor {
        namespace Rendering {

            RenderTexture::RenderTexture(int width, int height, const std::string& format)
            : m_width(width)
            , m_height(height)
            , m_textureId(0)
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
            }

            RenderTexture::~RenderTexture() {
                if (m_textureId) {
                    glDeleteTextures(1, &m_textureId);
                    m_textureId = 0;
                }
            }

            void RenderTexture::bind(int textureUnit) {
                glActiveTexture(GL_TEXTURE0 + textureUnit);
                glBindTexture(GL_TEXTURE_2D, m_textureId);
            }

            void RenderTexture::unbind(int textureUnit) {
                glActiveTexture(GL_TEXTURE0 + textureUnit);
                glBindTexture(GL_TEXTURE_2D, 0);
            }

            void RenderTexture::upload(const void* data, const std::string& format) {
                glBindTexture(GL_TEXTURE_2D, m_textureId);

                if (format == "rgba" || m_format == "rgba8") {
                    glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, m_width, m_height, GL_RGBA, GL_UNSIGNED_BYTE, data);
                } else if (format == "rgb" || m_format == "rgb8") {
                    glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, m_width, m_height, GL_RGB, GL_UNSIGNED_BYTE, data);
                } else if (format == "r" || m_format == "r8") {
                    glTexSubImage2D(GL_TEXTURE_2D, 0, 0, 0, m_width, m_height, GL_RED, GL_UNSIGNED_BYTE, data);
                } else {
                    std::cerr << "Unsupported texture format for upload: " << format << std::endl;
                }
            }

            void RenderTexture::resize(int width, int height) {
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
            }

        } // namespace Rendering
    } // namespace Compositor
} // namespace VivoX
