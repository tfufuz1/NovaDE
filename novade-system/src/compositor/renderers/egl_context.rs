use khronos_egl as egl;
use glow;
use std::rc::Rc;
use std::os::raw::c_void;

// This file was potentially modified by an earlier step and might contain EGLContextManager
// and GlContext. The goal is to have only GlContext with the necessary updates.

use khronos_egl as egl;
use glow;
use std::rc::Rc;
use std::os::raw::c_void;
use std::ffi::CStr;

// Placeholder for Smithay's wl_display opaque type, if not directly available from a crate.
// For actual integration, this would come from `smithay::reexports::wayland_server::sys::wl_display`
// or `wayland_client::sys::client::wl_display`.
pub enum WlDisplay {} // Keep as opaque placeholder as per original.
type WlDisplayPtr = *mut c_void; // Matches original use as a generic pointer.


#[derive(Debug)]
pub enum OpenGLError {
    EglGetDisplayFailed(String),
    EglInitializeFailed(String),
    EglChooseConfigFailed(String),
    EglCreateContextFailed(String),
    EglMakeCurrentFailed(String),
    EglGetCurrentContextFailed(String), // Keep if GlContext::is_current needs it
    EglQueryStringFailed(String),
    EglCreatePbufferSurfaceFailed(String),
    EglDestroySurfaceFailed(String),
    EglDestroyContextFailed(String),
    EglTerminateFailed(String),
    LoadFunctionsFailed(String),
    Other(String), // General fallback
}

// Helper to get a String from egl::Instance::get_error()
// Made public to be potentially usable by egl_surface.rs if needed, or keep internal.
pub fn egl_error_string_from_instance(instance: &egl::Instance) -> String {
    let error_code = instance.get_error();
    match error_code {
        egl::SUCCESS => "EGL_SUCCESS".to_string(),
        egl::NOT_INITIALIZED => "EGL_NOT_INITIALIZED".to_string(),
        egl::BAD_ACCESS => "EGL_BAD_ACCESS".to_string(),
        egl::BAD_ALLOC => "EGL_BAD_ALLOC".to_string(),
        egl::BAD_ATTRIBUTE => "EGL_BAD_ATTRIBUTE".to_string(),
        egl::BAD_CONTEXT => "EGL_BAD_CONTEXT".to_string(),
        egl::BAD_CONFIG => "EGL_BAD_CONFIG".to_string(),
        egl::BAD_CURRENT_SURFACE => "EGL_BAD_CURRENT_SURFACE".to_string(),
        egl::BAD_DISPLAY => "EGL_BAD_DISPLAY".to_string(),
        egl::BAD_SURFACE => "EGL_BAD_SURFACE".to_string(),
        egl::BAD_MATCH => "EGL_BAD_MATCH".to_string(),
        egl::BAD_PARAMETER => "EGL_BAD_PARAMETER".to_string(),
        egl::BAD_NATIVE_PIXMAP => "EGL_BAD_NATIVE_PIXMAP".to_string(),
        egl::BAD_NATIVE_WINDOW => "EGL_BAD_NATIVE_WINDOW".to_string(),
        egl::CONTEXT_LOST => "EGL_CONTEXT_LOST".to_string(),
        _ => format!("Unknown EGL error code: {}", error_code),
    }
}


pub struct GlContext {
    instance: egl::Instance, // Store the EGL instance
    display: egl::Display,  // Renamed from _display for clarity and direct access via pub method
    config: egl::Config,    // Renamed from _config
    context: egl::Context,
    gl: Rc<glow::Context>,
    // Temporary Pbuffer surface for headless initialization.
    // This surface is owned by GlContext and used to make the context current initially.
    // It's important that this surface is cleaned up.
    init_pbuffer_surface: Option<egl::Surface>,
    pub egl_extensions: Vec<String>, // Store EGL extensions
    pub supports_dma_buf_import: bool, // DMA-BUF import capability
}

impl GlContext {
    pub fn new(wl_display_ptr: Option<WlDisplayPtr>) -> Result<Self, OpenGLError> {
        let instance = unsafe { egl::Instance::current_platform_instance() };

        let egl_display = match wl_display_ptr {
            Some(ptr) if !ptr.is_null() => unsafe { instance.get_display(ptr as egl::NativeDisplayType) },
            _ => unsafe { instance.get_display(egl::DEFAULT_DISPLAY as egl::NativeDisplayType) },
        }.map_err(|e| OpenGLError::EglGetDisplayFailed(format!("EGL error {}: {}", e.code(), egl_error_string_from_instance(&instance))))?;

        if egl_display == egl::NO_DISPLAY {
            return Err(OpenGLError::EglGetDisplayFailed("eglGetDisplay returned EGL_NO_DISPLAY".to_string()));
        }

        let (mut major, mut minor) = (0, 0);
        instance.initialize(egl_display, &mut major, &mut minor)
            .map_err(|e| OpenGLError::EglInitializeFailed(format!("EGL error {}: {}", e.code(), egl_error_string_from_instance(&instance))))?;
        println!("EGL Initialized. Version: {}.{}", major, minor);

        // Query, parse, and store EGL extensions
        let egl_extensions_c_str = unsafe {
            instance.query_string(Some(egl_display), egl::EXTENSIONS)
                .map_err(|e| OpenGLError::EglQueryStringFailed(format!("EGL QueryString for EGL_EXTENSIONS failed. EGL error {}: {}", e.code(), egl_error_string_from_instance(&instance))))?
        };
        let egl_extensions_str = unsafe { CStr::from_ptr(egl_extensions_c_str).to_string_lossy().into_owned() };
        
        let egl_extensions_vec: Vec<String> = egl_extensions_str.split(' ').map(|s| s.to_string()).collect();
        println!("Available EGL Extensions: {:?}", egl_extensions_vec);

        let supports_dma_buf_import = egl_extensions_vec.iter().any(|s| s == "EGL_EXT_image_dma_buf_import") &&
                                      egl_extensions_vec.iter().any(|s| s == "EGL_KHR_image_base");
        if supports_dma_buf_import {
            println!("EGL_EXT_image_dma_buf_import and EGL_KHR_image_base are supported.");
        } else {
            println!("EGL_EXT_image_dma_buf_import or EGL_KHR_image_base NOT supported. DMA-BUF import will not be available.");
        }


        let config_attribs = [
            egl::SURFACE_TYPE, egl::PBUFFER_BIT | egl::WINDOW_BIT,
            egl::RENDERABLE_TYPE, egl::OPENGL_ES3_BIT,
            egl::RED_SIZE, 8, egl::GREEN_SIZE, 8, egl::BLUE_SIZE, 8, egl::ALPHA_SIZE, 8,
            egl::DEPTH_SIZE, 24,
            egl::NONE,
        ];
        let mut configs = Vec::with_capacity(10);
        let mut num_configs = 0;
        instance.choose_config(egl_display, &config_attribs, &mut configs, &mut num_configs)
            .map_err(|e| OpenGLError::EglChooseConfigFailed(format!("EGL error {}: {}", e.code(), egl_error_string_from_instance(&instance))))?;

        if num_configs == 0 {
            return Err(OpenGLError::EglChooseConfigFailed("No suitable EGL configs found".to_string()));
        }
        let egl_config = configs.get(0).copied().ok_or_else(|| OpenGLError::EglChooseConfigFailed("No EGL config found in returned list".to_string()))?;

        let mut context_attribs = vec![egl::CONTEXT_CLIENT_VERSION, 3];
        if extensions_str.contains("EGL_KHR_create_context") {
            // Add debug context flags or specific version requests if needed
            // context_attribs.extend([egl::CONTEXT_FLAGS_KHR, egl::CONTEXT_OPENGL_DEBUG_BIT_KHR]);
        }
        context_attribs.push(egl::NONE);

        let egl_context = instance.create_context(egl_display, egl_config, None, &context_attribs)
            .map_err(|e| OpenGLError::EglCreateContextFailed(format!("EGL error {}: {}", e.code(), egl_error_string_from_instance(&instance))))?;

        if egl_context == egl::NO_CONTEXT {
             return Err(OpenGLError::EglCreateContextFailed("eglCreateContext returned EGL_NO_CONTEXT".to_string()));
        }

        let pbuffer_attribs = [egl::WIDTH, 1, egl::HEIGHT, 1, egl::NONE];
        let temp_surface = instance.create_pbuffer_surface(egl_display, egl_config, &pbuffer_attribs)
            .map_err(|e| OpenGLError::EglCreatePbufferSurfaceFailed(format!("EGL error {}: {}", e.code(), egl_error_string_from_instance(&instance))))?;

        instance.make_current(egl_display, Some(temp_surface), Some(temp_surface), Some(egl_context))
            .map_err(|e| {
                let err_str = format!("EGL error {}: {}", e.code(), egl_error_string_from_instance(&instance));
                // Cleanup temp_surface before returning error
                let _ = instance.destroy_surface(egl_display, temp_surface);
                OpenGLError::EglMakeCurrentFailed(err_str)
            })?;

        let gl_functions = unsafe {
            glow::Context::from_loader_function_with_version_parse(
                |s| instance.get_proc_address(s) as *const _,
            ).map_err(|e_str| {
                // Cleanup before returning
                let _ = instance.make_current(egl_display, None, None, None);
                let _ = instance.destroy_surface(egl_display, temp_surface);
                OpenGLError::LoadFunctionsFailed(e_str.to_string())
            })?
        };
        
        println!("OpenGL ES context created successfully with glow.");

        // Setup GL Debug Callback if KHR_debug is supported
        // This needs to be done when the context is current.
        let gl_ptr = Rc::new(gl_functions); // Rc for the glow::Context

        if gl_ptr.supported_extensions().contains("GL_KHR_debug") {
            unsafe {
                gl_ptr.debug_message_callback(|source, gl_type, id, severity, message| {
                    let source_str = match source {
                        glow::DEBUG_SOURCE_API => "API",
                        glow::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
                        glow::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
                        glow::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
                        glow::DEBUG_SOURCE_APPLICATION => "Application",
                        glow::DEBUG_SOURCE_OTHER => "Other",
                        _ => "Unknown Source",
                    };
                    let type_str = match gl_type {
                        glow::DEBUG_TYPE_ERROR => "Error",
                        glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
                        glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior",
                        glow::DEBUG_TYPE_PORTABILITY => "Portability",
                        glow::DEBUG_TYPE_PERFORMANCE => "Performance",
                        glow::DEBUG_TYPE_MARKER => "Marker",
                        glow::DEBUG_TYPE_PUSH_GROUP => "Push Group",
                        glow::DEBUG_TYPE_POP_GROUP => "Pop Group",
                        glow::DEBUG_TYPE_OTHER => "Other",
                        _ => "Unknown Type",
                    };
                    let severity_str = match severity {
                        glow::DEBUG_SEVERITY_HIGH => "High",
                        glow::DEBUG_SEVERITY_MEDIUM => "Medium",
                        glow::DEBUG_SEVERITY_LOW => "Low",
                        glow::DEBUG_SEVERITY_NOTIFICATION => "Notification",
                        _ => "Unknown Severity",
                    };
                    // TODO: Replace eprintln! with tracing crate macros, e.g.,
                    // tracing::error!(target: "opengl", "[{}][{}][{}][{}]: {}", source_str, type_str, id, severity_str, message);
                    eprintln!(
                        "[GL DEBUG][{}][{}][{}][{}]: {}",
                        source_str, type_str, id, severity_str, message.trim_end()
                    );
                });
                // Enable all debug messages. Adjust filter as needed.
                gl_ptr.debug_message_control(glow::DONT_CARE, glow::DONT_CARE, glow::DONT_CARE, &[], true);
                // For development, synchronous output can be helpful.
                // Consider making this configurable.
                // gl_ptr.enable(glow::DEBUG_OUTPUT_SYNCHRONOUS); 
                println!("OpenGL Debug Output (GL_KHR_debug) enabled.");
            }
        } else {
            println!("GL_KHR_debug extension not supported.");
        }

        Ok(Self {
            instance,
            display: egl_display,
            config: egl_config,
            context: egl_context,
            gl: gl_ptr, // Use the Rc created above
            init_pbuffer_surface: Some(temp_surface),
            egl_extensions: egl_extensions_vec,
            supports_dma_buf_import,
        })
    }

    pub fn egl_instance(&self) -> &egl::Instance {
        &self.instance
    }

    pub fn display(&self) -> egl::Display {
        self.display
    }

    pub fn config(&self) -> egl::Config {
        self.config
    }
    
    pub fn context(&self) -> egl::Context {
        self.context
    }

    pub fn gl(&self) -> Rc<glow::Context> {
        Rc::clone(&self.gl)
    }

    /// Makes the EGL context current on the calling thread with the specified draw and read surfaces.
    /// If `draw_surface` and `read_surface` are None, the context is made current with no surface,
    /// or with the initial pbuffer surface if that's desired for some operations.
    /// For making current with a *window* surface, EGLSurfaceWrapper.make_current should be used.
    /// This method is more general. If both are None, it uses the init_pbuffer_surface.
    pub fn make_current_with_surface(&self, draw_surface: Option<egl::Surface>, read_surface: Option<egl::Surface>) -> Result<(), OpenGLError> {
        let draw = draw_surface.or(self.init_pbuffer_surface);
        let read = read_surface.or(self.init_pbuffer_surface);

        self.instance.make_current(self.display, draw, read, Some(self.context))
            .map_err(|e| OpenGLError::EglMakeCurrentFailed(format!("EGL error {}: {}", e.code(), egl_error_string_from_instance(&self.instance))))
    }
    
    /// Makes the EGL context current on the calling thread using its internal pbuffer surface.
    /// Useful for headless operations or before a window surface is available.
    pub fn make_current_headless(&self) -> Result<(), OpenGLError> {
        match self.init_pbuffer_surface {
            Some(pbuffer) => self.make_current_with_surface(Some(pbuffer), Some(pbuffer)),
            None => Err(OpenGLError::EglMakeCurrentFailed("No pbuffer surface available for headless make_current".to_string())),
        }
    }

    pub fn release_current(&self) -> Result<(), OpenGLError> {
        self.instance.make_current(self.display, None, None, None)
            .map_err(|e| OpenGLError::EglMakeCurrentFailed(format!("Release current EGL error {}: {}", e.code(), egl_error_string_from_instance(&self.instance))))
    }

    /// Checks if this EGL context is current on the calling thread and optionally bound to specific draw/read surfaces.
    pub fn is_current_for_surface_and_context(&self, surface: egl::Surface) -> bool {
        let current_ctx = self.instance.get_current_context();
        if current_ctx != Some(self.context) {
            return false;
        }
        // If context is correct, check surfaces
        let current_draw_surface = self.instance.get_current_surface(egl::DRAW);
        let current_read_surface = self.instance.get_current_surface(egl::READ);
        
        current_draw_surface == Some(surface) && current_read_surface == Some(surface)
    }
}

impl Drop for GlContext {
    fn drop(&mut self) {
        // Ensure the context is not current, or make a null context current.
        if self.instance.get_current_context() == Some(self.context) {
            if self.instance.make_current(self.display, None, None, None).is_err() {
                eprintln!("Error releasing context during GlContext drop: {}", egl_error_string_from_instance(&self.instance));
            }
        }

        // Destroy the GL resources, temp_surface first, then context.
        if let Some(surface) = self.init_pbuffer_surface.take() {
            if self.instance.destroy_surface(self.display, surface).is_err() {
                eprintln!("Error destroying EGL temporary pbuffer surface: {}", egl_error_string_from_instance(&self.instance));
            }
        }

        if self.context != egl::NO_CONTEXT {
            if self.instance.destroy_context(self.display, self.context).is_err() {
                eprintln!("Error destroying EGL context: {}", egl_error_string_from_instance(&self.instance));
            }
            self.context = egl::NO_CONTEXT; // Mark as destroyed
        }

        if self.display != egl::NO_DISPLAY {
            if self.instance.terminate(self.display).is_err() {
                eprintln!("Error terminating EGL display: {}", egl_error_string_from_instance(&self.instance));
            }
            self.display = egl::NO_DISPLAY; // Mark as terminated
        }
    }
}
