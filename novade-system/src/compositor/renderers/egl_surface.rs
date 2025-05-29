use std::rc::Rc;
use std::os::raw::c_void;

use khronos_egl as egl;
// Correct type for wl_surface pointer when interacting with wayland-egl
use wayland_client::sys::client::wl_proxy;
use wayland_egl; // For WlEglWindow

// Assuming egl_context.rs is in the same module directory (super::egl_context)
// and OpenGLError is public.
use super::egl_context::{GlContext, OpenGLError};

// Helper function from egl_context, needs to be accessible.
// If not pub in egl_context, it might need to be duplicated or made pub.
// For now, assume it can be called if GlContext provides necessary EGL error info.
// Let's define a local one for now if it's simple, or rely on GlContext exposing error details.
fn egl_error_string() -> String {
    // This is a simplified placeholder. Ideally, use the one from egl_context or similar.
    // The egl::Instance should be used to get the error.
    // This function might not be needed if error methods on GlContext provide enough detail.
    format!("EGL error code: {}", khronos_egl::Instance::current_platform_instance().get_error())
}


#[derive(Debug)]
pub enum EGLSurfaceError {
    CreationFailed(String),
    WlEglWindowCreationFailed(String),
    MakeCurrentFailed(String),
    SwapBuffersFailed(String),
    SwapIntervalFailed(String),
    ResizeNotPossible(String),
    UnderlyingOpenGLError(OpenGLError), // If GlContext methods return OpenGLError
    Misc(String),
}

impl From<OpenGLError> for EGLSurfaceError {
    fn from(e: OpenGLError) -> Self {
        EGLSurfaceError::UnderlyingOpenGLError(e)
    }
}


pub struct EGLSurfaceWrapper {
    gl_context: Rc<GlContext>,
    egl_surface: egl::Surface,
    wl_egl_window: wayland_egl::WlEglWindow,
    width: i32,
    height: i32,
}

impl EGLSurfaceWrapper {
    /// Creates a new EGLSurfaceWrapper for a given Wayland surface proxy.
    ///
    /// # Safety
    ///
    /// The `wl_surface_proxy_ptr` must be a valid pointer to a Wayland client `wl_proxy`
    /// that represents a `wl_surface`. This pointer is typically obtained from a
    /// `wayland_client::protocol::wl_surface::WlSurface` object via its `.c_ptr()` method.
    /// The caller is responsible for ensuring the `wl_surface` outlives this `EGLSurfaceWrapper`.
    pub unsafe fn new(
        gl_context: Rc<GlContext>,
        wl_surface_proxy_ptr: *mut wl_proxy, // Pointer to wl_proxy (client-side surface)
        width: i32,
        height: i32,
    ) -> Result<Self, EGLSurfaceError> {
        if wl_surface_proxy_ptr.is_null() {
            return Err(EGLSurfaceError::WlEglWindowCreationFailed("Provided wl_surface_proxy_ptr was null".to_string()));
        }
        if width <= 0 || height <= 0 {
            return Err(EGLSurfaceError::WlEglWindowCreationFailed("Invalid width or height".to_string()));
        }

        // 1. Wayland EGL Window Creation
        let wl_egl_window = wayland_egl::WlEglWindow::new(wl_surface_proxy_ptr, width, height)
            .or_else(|e_primary| {
                eprintln!("WlEglWindow::new failed (libwayland-egl.so.1?): {:?}. Trying fallback.", e_primary);
                wayland_egl::WlEglWindow::new_fallback(wl_surface_proxy_ptr, width, height)
            })
            .map_err(|e_fallback| EGLSurfaceError::WlEglWindowCreationFailed(format!("Primary and fallback WlEglWindow creation failed: {:?}", e_fallback)))?;
        
        let egl_instance = gl_context.egl_instance();

        // 2. EGL Surface Creation
        let surface_attribs = [
            // egl::RENDER_BUFFER, egl::BACK_BUFFER, // Usually default
            // egl::GL_COLORSPACE_KHR, egl::GL_COLORSPACE_SRGB_KHR, // Example for SRGB if needed and supported
            egl::NONE,
        ];

        let egl_surface = egl_instance
            .create_window_surface(
                gl_context.display(),
                gl_context.config(),
                wl_egl_window.ptr() as egl::NativeWindowType, // wl_egl_window.ptr() is *mut c_void
                Some(&surface_attribs),
            )
            .map_err(|e| EGLSurfaceError::CreationFailed(format!("eglCreateWindowSurface: {} (EGL error code: {})", e, egl_instance.get_error())))?;
            // Ensure egl_error_string() or similar is used if `e` itself is not descriptive enough.
            // The `e.code()` from khronos_egl::Error can be used.

        Ok(Self {
            gl_context,
            egl_surface,
            wl_egl_window,
            width,
            height,
        })
    }

    pub fn make_current(&self) -> Result<(), EGLSurfaceError> {
        self.gl_context.make_current_with_surface(Some(self.egl_surface), Some(self.egl_surface))?;
        Ok(())
    }
    
    pub fn release_current(&self) -> Result<(), EGLSurfaceError> {
        self.gl_context.release_current()?;
        Ok(())
    }

    pub fn swap_buffers(&self) -> Result<(), EGLSurfaceError> {
        let egl_instance = self.gl_context.egl_instance();
        egl_instance
            .swap_buffers(self.gl_context.display(), self.egl_surface)
            .map_err(|e| EGLSurfaceError::SwapBuffersFailed(format!("eglSwapBuffers: {} (EGL error code: {})", e, egl_instance.get_error())))
    }

    pub fn set_swap_interval(&self, interval: i32) -> Result<(), EGLSurfaceError> {
        let egl_instance = self.gl_context.egl_instance();
        egl_instance
            .swap_interval(self.gl_context.display(), interval)
            .map_err(|e| EGLSurfaceError::SwapIntervalFailed(format!("eglSwapInterval: {} (EGL error code: {})",e, egl_instance.get_error())))
    }

    pub fn resize(&mut self, new_width: i32, new_height: i32) -> Result<(), EGLSurfaceError> {
        if new_width <= 0 || new_height <= 0 {
            return Err(EGLSurfaceError::ResizeNotPossible("Invalid dimensions for resize".to_string()));
        }
        // dx, dy are destination x, y offsets, usually 0 for a simple resize.
        self.wl_egl_window.resize(new_width, new_height, 0, 0);
        self.width = new_width;
        self.height = new_height;
        
        // The EGL surface is implicitly resized by EGL on next swap_buffers.
        // No explicit EGL surface recreation is usually needed.
        Ok(())
    }
    
    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }
    
    pub fn gl(&self) -> Rc<glow::Context> {
        self.gl_context.gl()
    }

    // Helper to get the EGLSurface, might be needed by renderer internals
    pub fn egl_surface(&self) -> egl::Surface {
        self.egl_surface
    }
}

impl Drop for EGLSurfaceWrapper {
    fn drop(&mut self) {
        // Ensure context is not current with this surface before destroying.
        // This might involve making a NULL context current or another context current.
        // If this surface's context is current on this thread:
        if self.gl_context.is_current_for_surface_and_context(self.egl_surface) {
             let _ = self.gl_context.release_current().map_err(|e| {
                eprintln!("Error releasing context during EGLSurfaceWrapper drop: {:?}", e);
             }); // Log error if release fails
        }

        if self.egl_surface != egl::NO_SURFACE {
            let egl_instance = self.gl_context.egl_instance();
            if egl_instance
                .destroy_surface(self.gl_context.display(), self.egl_surface)
                .is_err()
            {
                eprintln!("Error destroying EGL surface: {} (EGL error code: {})", egl_error_string(), egl_instance.get_error());
            }
            self.egl_surface = egl::NO_SURFACE;
        }
        // `self.wl_egl_window` is dropped automatically. Its Drop impl handles cleanup.
    }
}

// To make this compile, `GlContext` needs to be updated:
// 1. `display()` and `config()` methods to access `_display` and `_config`.
// 2. `egl_instance()` method to get the `egl::Instance`. (Store it or fetch it).
// 3. `make_current_with_surface()` method.
// 4. `is_current_for_surface_and_context()` method.
// Example additions to `novade-system/src/compositor/renderers/egl_context.rs`
/*
// In GlContext:

// Store the EGL instance instead of fetching it repeatedly.
// pub struct GlContext {
//     instance: egl::Instance, // Or some EGL instance wrapper from khronos_egl
//     _display: egl::Display,
// ...
// }
// impl GlContext {
//     pub fn new(...) -> Result<Self, OpenGLError> {
//         let instance = unsafe { egl::Instance::current_platform_instance() };
//         // ... use this instance ...
//         Ok(Self { instance, ... })
//     }
//     pub fn egl_instance(&self) -> &egl::Instance { &self.instance }
//     pub fn display(&self) -> egl::Display { self._display }
//     pub fn config(&self) -> egl::Config { self._config }
//
//     pub fn make_current_with_surface(&self, draw: Option<egl::Surface>, read: Option<egl::Surface>) -> Result<(), OpenGLError> {
//         self.instance.make_current(self._display, draw, read, Some(self.context))
//             .map_err(|e| OpenGLError::EglMakeCurrentFailed) // Add more detail
//     }
//
//     pub fn is_current_for_surface_and_context(&self, surface: egl::Surface) -> bool {
//         let current_draw = self.instance.get_current_surface(egl::DRAW);
//         let current_read = self.instance.get_current_surface(egl::READ);
//         let current_ctx = self.instance.get_current_context();
//         current_ctx == Some(self.context) && (current_draw == Some(surface) || current_read == Some(surface))
//     }
// }
*/

// Note on `wl_surface_proxy_ptr`:
// This pointer must correspond to a client-side Wayland surface.
// In a Wayland compositor (like one built with Smithay), the compositor manages server-side
// `wl_surface` objects. If this EGL surface is for rendering *to* a client's window *from the client side*,
// this code is appropriate for a client application.
// If this is for the compositor to render to a Wayland surface (e.g., using hardware acceleration
// via EGL on the compositor side), then the `wl_surface` is server-side.
// `wayland-egl` is typically used client-side. For server-side EGL on Wayland,
// different mechanisms like `EGL_PLATFORM_WAYLAND_KHR` with `eglGetPlatformDisplay`
// and then creating surfaces for server-side `wl_resource`s might be used, often involving
// `EGL_WL_bind_wayland_display` and custom EGL extensions if not using `wayland-egl` directly.
// Given the use of `wayland-client` types, this code assumes a client-side context.
// Smithay's `Client` type or similar constructs would manage client connections and their surfaces.
// If Smithay itself is the compositor, and it needs to render using EGL to an internal representation
// of a client window, it would use its server-side `wl_surface` resource, get its `wl_resource` pointer,
// and potentially use extensions like `EGL_KHR_platform_gbm` (if rendering headless to a GBM buffer to be
// composited) or `EGL_MESA_platform_surfaceless` or `EGL_KHR_surfaceless_context` if no actual window system
// surface is directly used by EGL.
// This specific code using `WlEglWindow` is for a classic client-side EGL setup.
