// RenderShader.h
#pragma once

#include <cstdint>
#include <string>

namespace VivoX {
    namespace Compositor {
        namespace Rendering {

            class RenderShader {
            public:
                RenderShader(const std::string& vertexShader, const std::string& fragmentShader);
                ~RenderShader();

                void bind();
                void unbind();
                bool isCompiled() const;

                void setUniformInt(const std::string& name, int value);
                void setUniformFloat(const std::string& name, float value);
                void setUniformVec2(const std::string& name, float x, float y);
                void setUniformVec3(const std::string& name, float x, float y, float z);
                void setUniformVec4(const std::string& name, float x, float y, float z, float w);
                void setUniformMat4(const std::string& name, const float* matrix);

            private:
                uint32_t compileShader(uint32_t type, const std::string& source);

                uint32_t m_programId;
                uint32_t m_vertexShaderId;
                uint32_t m_fragmentShaderId;
                bool m_isCompiled;
            };

                    } // namespace Rendering
                } // namespace Compositor
            } // namespace VivoX
