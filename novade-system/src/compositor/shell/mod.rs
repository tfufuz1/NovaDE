// novade-system/src/compositor/shell/mod.rs
pub mod xdg; // For the new XDG shell implementation (xdg.rs and xdg/handlers.rs)
pub mod xdg_decoration; // For the XDG decoration protocol
pub mod xdg_shell; // Existing module, ensure it doesn't conflict or is phased out.
// Potentially other shell modules like layer_shell will be added here later.
