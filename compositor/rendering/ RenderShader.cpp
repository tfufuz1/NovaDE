// RenderShader.cpp
#include "RenderShader.h"
#include <GL/gl.h>
#include <iostream>
#include <vector>

namespace VivoX {
namespace Compositor {
namespace Rendering {

RenderShader::RenderShader(const std::string& vertexShader, const std::string& fragmentShader)
    : m_programId(0)
    , m_vertexShaderId(0)
    , m_fragmentShaderId(0)
    , m_isCompiled(false) {
    
    // Create shader program
    m_programId = glCreateProgram();
    
    // Compile vertex shader
    m_vertexShaderId = compileShader(GL_VERTEX_SHADER, vertexShader);
    if (m_vertexShaderId == 0) {
        std::cerr << "Failed to compile vertex shader" << std::endl;
        return;
    }
    
    // Compile fragment shader
    m_fragmentShaderId = compileShader(GL_FRAGMENT_SHADER, fragmentShader);
    if (m_fragmentShaderId == 0) {
        std::cerr << "Failed to compile fragment shader" << std::endl;
        glDeleteShader(m_vertexShaderId);
        m_vertexShaderId = 0;
        return;
    }
    
    // Attach shaders to program
    glAttachShader(m_programId, m_vertexShaderId);
    glAttachShader(m_programId, m_fragmentShaderId);
    
    // Link program
    glLinkProgram(m_programId);
    
    // Check link status
    GLint linkStatus = GL_FALSE;
    glGetProgramiv(m_programId, GL_LINK_STATUS, &linkStatus);
    if (linkStatus != GL_TRUE) {
        GLint infoLogLength = 0;
        glGetProgramiv(m_programId, GL_INFO_LOG_LENGTH, &infoLogLength);
        
        if (infoLogLength > 0) {
            std::vector<char> infoLog(infoLogLength);
            glGetProgramInfoLog(m_programId, infoLogLength, nullptr, infoLog.data());
            std::cerr << "Shader program link error: " << infoLog.data() << std::endl;
        }
        
        glDeleteShader(m_vertexShaderId);
        glDeleteShader(m_fragmentShaderId);
        glDeleteProgram(m_programId);
        
        m_vertexShaderId = 0;
        m_fragmentShaderId = 0;
        m_programId = 0;
        
        return;
    }
    
    // Detach shaders after linking
    glDetachShader(m_programId, m_vertexShaderId);
    glDetachShader(m_programId, m_fragmentShaderId);
    
    m_isCompiled = true;
}

RenderShader::~RenderShader() {
    if (m_vertexShaderId) {
        glDeleteShader(m_vertexShaderId);
        m_vertexShaderId = 0;
    }
    
    if (m_fragmentShaderId) {
        glDeleteShader(m_fragmentShaderId);
        m_fragmentShaderId = 0;
    }
    
    if (m_programId) {
        glDeleteProgram(m_programId);
        m_programId = 0;
    }
}

void RenderShader::bind() {
    if (m_isCompiled) {
        glUseProgram(m_programId);
    }
}

void RenderShader::unbind() {
    glUseProgram(0);
}

bool RenderShader::isCompiled() const {
    return m_isCompiled;
}

void RenderShader::setUniformInt(const std::string& name, int value) {
    if (!m_isCompiled) {
        return;
    }
    
    GLint location = glGetUniformLocation(m_programId, name.c_str());
    if (location != -1) {
        glUniform1i(location, value);
    }
}

void RenderShader::setUniformFloat(const std::string& name, float value) {
    if (!m_isCompiled) {
        return;
    }
    
    GLint location = glGetUniformLocation(m_programId, name.c_str());
    if (location != -1) {
        glUniform1f(location, value);
    }
}

void RenderShader::setUniformVec2(const std::string& name, float x, float y) {
    if (!m_isCompiled) {
        return;
    }
    
    GLint location = glGetUniformLocation(m_programId, name.c_str());
    if (location != -1) {
        glUniform2f(location, x, y);
    }
}

void RenderShader::setUniformVec3(const std::string& name, float x, float y, float z) {
    if (!m_isCompiled) {
        return;
    }
    
    GLint location = glGetUniformLocation(m_programId, name.c_str());
    if (location != -1) {
        glUniform3f(location, x, y, z);
    }
}

void RenderShader::setUniformVec4(const std::string& name, float x, float y, float z, float w) {
    if (!m_isCompiled) {
        return;
    }
    
    GLint location = glGetUniformLocation(m_programId, name.c_str());
    if (location != -1) {
        glUniform4f(location, x, y, z, w);
    }
}

void RenderShader::setUniformMat4(const std::string& name, const float* matrix) {
    if (!m_isCompiled) {
        return;
    }
    
    GLint location = glGetUniformLocation(m_programId, name.c_str());
    if (location != -1) {
        glUniformMatrix4fv(location, 1, GL_FALSE, matrix);
    }
}

GLuint RenderShader::compileShader(GLenum type, const std::string& source) {
    GLuint shader = glCreateShader(type);
    if (shader == 0) {
        std::cerr << "Failed to create shader" << std::endl;
        return 0;
    }
    
    const char* sourceCStr = source.c_str();
    glShaderSource(shader, 1, &sourceCStr, nullptr);
    glCompileShader(shader);
    
    GLint compileStatus = GL_FALSE;
    glGetShaderiv(shader, GL_COMPILE_STATUS, &compileStatus);
    if (compileStatus != GL_TRUE) {
        GLint infoLogLength = 0;
        glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &infoLogLength);
        
        if (infoLogLength > 0) {
            std::vector<char> infoLog(infoLogLength);
            glGetShaderInfoLog(shader, infoLogLength, nullptr, infoLog.data());
            std::cerr << "Shader compile error: " << infoLog.data() << std::endl;
        }
        
        glDeleteShader(shader);
        return 0;
    }
    
    return shader;
}

} // namespace Rendering
} // namespace Compositor
} // namespace VivoX
