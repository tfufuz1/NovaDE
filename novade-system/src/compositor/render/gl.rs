// novade-system/src/compositor/render/gl.rs

use smithay::backend::renderer::{
    gles2::{Gles2Renderer, Gles2Texture},
    Bind,
    Renderer,
    Error as RendererError, // Explicitly import Smithay Renderer Error
};
use smithay::reexports::{
    wayland_server::DisplayHandle,
    calloop::LoopHandle,
};
use smithay::utils::{Buffer, Transform, Size};
use tracing::{info, debug, error, warn};

// Importiere notwendige GL/EGL-Abhängigkeiten
// Annahme: `libegl_sys` oder `egl` crate wird verwendet.
// Für Smithay GLES2Renderer wird typischerweise `smithay::backend::allocator::dmabuf` und `smithay::backend::egl` verwendet.
use smithay::backend::egl::{Egl, EglDisplay, Error as EglError}; // Explicitly import EglError
// use smithay::backend::allocator::dmabuf::Dmabuf; // Not used in the provided snippet directly, but good to keep in mind for future DMABUF integration.

/// Errors that can occur during the initialization of the OpenGL (GLES2) renderer.
#[derive(Debug, thiserror::Error)]
pub enum GlInitError {
    /// An error occurred within the EGL library.
    /// This typically relates to display, context, or surface management.
    #[error("EGL error: {0}")]
    EglError(#[from] EglError),

    /// An error occurred within Smithay's renderer abstraction.
    /// This could indicate issues with shader compilation, buffer allocation, or other OpenGL operations.
    #[error("Smithay renderer error: {0}")]
    SmithayRendererError(#[from] RendererError),

    /// No suitable EGL context or EGL display was found or could be configured.
    /// This may happen if the underlying graphics hardware or drivers are not properly set up
    /// or do not support the required GLES2 features.
    #[error("No suitable EGL context or EGL display available.")]
    NoSuitableEglContext,

    /// An error occurred during the initialization of a DRM (Direct Rendering Manager) backend.
    /// This variant is a placeholder for more specific DRM errors if a DRM backend is used.
    #[error("DRM backend initialization error: {0}")]
    DrmBackendError(String),

    /// An unknown or unspecified error occurred during renderer initialization.
    #[error("An unknown error occurred during GL renderer initialization.")]
    Unknown,
}

/// Initializes a Smithay GLES2 renderer using an existing EGL context.
///
/// This function is responsible for creating and configuring an OpenGL (GLES2)
/// renderer instance that Smithay can use for rendering operations within the compositor.
/// It leverages an `Egl` instance, which encapsulates the EGL display and context.
///
/// # Arguments
///
/// * `egl`: An initialized `smithay::backend::egl::Egl` instance. This instance
///   provides the necessary EGL display and context for the renderer. The caller
///   is responsible for ensuring this EGL instance is properly set up, for example,
///   by a graphics backend like DRM or by a headless setup.
///
/// # Returns
///
/// * `Ok(Gles2Renderer)`: A `smithay::backend::renderer::gles2::Gles2Renderer` instance
///   on successful initialization.
/// * `Err(GlInitError)`: An error of type `GlInitError` if any step of the initialization fails.
///   This includes EGL errors or errors from within Smithay's renderer creation process.
///
/// # Errors
///
/// This function can fail if:
/// - The EGL context provided by the `Egl` instance is invalid or not GLES2 compatible.
/// - Smithay's `Gles2Renderer::new()` fails for internal reasons (e.g., driver issues,
///   unsupported OpenGL features).
///
/// # Safety
///
/// The caller must ensure that the EGL context encapsulated by the provided `Egl` instance
/// remains valid for the entire lifetime of the `Gles2Renderer`. Dropping or invalidating
/// the EGL context while the renderer is still in use will lead to undefined behavior.
pub fn init_gl_renderer(egl: Egl) -> Result<Gles2Renderer, GlInitError> {
    info!("Initialisiere Smithay GLES2 Renderer...");

    // Versuche, einen GLES2 Renderer aus dem EGL-Kontext zu erstellen
    // Die `Egl` Instanz selbst wird hier für die Renderer-Erstellung benötigt,
    // da sie den EGL-Display und den Kontext verwaltet.
    let renderer = Gles2Renderer::new(egl)?; // `?` Operator leitet EglError oder RendererError weiter

    info!("Smithay GLES2 Renderer erfolgreich initialisiert.");
    Ok(renderer)
}

// Hier würden später weitere Funktionen für das Rendern spezifischer Oberflächen hinzukommen,
// z.B. `render_xdg_surface`, `render_layer_surface` etc. Diese Funktionen würden
// die `Gles2Renderer`-Instanz verwenden, um die tatsächliche Zeichenlogik auszuführen.
// Das wird in einem späteren Prompt detailliert.

#[cfg(test)]
mod tests {
    use super::*;
    use smithay::backend::egl::Egl;

    // Helper function to attempt EGL initialization for tests.
    // This is a simplified version and might need adjustment based on the test environment.
    fn try_init_egl_for_test() -> Option<Egl> {
        // Try to initialize EGL. This might fail in environments without a display server
        // or proper EGL setup (e.g., in some CI environments).
        match Egl::new() {
            Ok(egl) => Some(egl),
            Err(e) => {
                warn!("EGL-Initialisierung für Test fehlgeschlagen: {}. Überspringe abhängige Tests.", e);
                None
            }
        }
    }

    #[test]
    fn test_init_gl_renderer_success() {
        // This test depends on a functional EGL environment.
        if let Some(egl_instance) = try_init_egl_for_test() {
            match init_gl_renderer(egl_instance) {
                Ok(renderer) => {
                    info!("Gles2Renderer erfolgreich im Test initialisiert: {:?}", renderer);
                    // Further checks could be added here if Gles2Renderer had inspectable properties
                }
                Err(e) => {
                    // If EGL was available but renderer init failed, that's a specific test failure.
                    error!("init_gl_renderer fehlgeschlagen, obwohl EGL verfügbar war: {}", e);
                    panic!("init_gl_renderer fehlgeschlagen mit EGL: {}", e);
                }
            }
        } else {
            warn!("Überspringe test_init_gl_renderer_success aufgrund fehlender EGL-Umgebung.");
        }
    }

    #[test]
    fn test_init_gl_renderer_error_propagation() {
        // This test is tricky because causing Gles2Renderer::new() to fail
        // without a valid EGL instance (which itself would be an EglError)
        // is hard to simulate directly.
        // Smithay's Gles2Renderer::new itself primarily returns errors originating from EGL context creation/binding.
        // If Egl::new() fails, that's an EglError, which is covered by the GlInitError::EglError variant.
        // If Gles2Renderer::new() itself had other specific error conditions beyond EGL,
        // we would mock them here.

        // For now, we mostly rely on the from implementations for EglError and RendererError
        // in GlInitError. A direct test for SmithayRendererError without a valid EGL context
        // is difficult.

        // A conceptual test for NoSuitableEglContext would involve a scenario
        // where EGL is present but no compatible configuration is found.
        // This is hard to mock at this level without deeper EGL control.
        warn!("test_init_gl_renderer_error_propagation ist derzeit auf die From-Implementierungen angewiesen.");
    }
}
