pub mod backend;
pub mod core;
pub mod cursor_manager; // Add this line
pub mod display_loop;
pub mod input;
pub mod renderer_interface;
pub mod render;
pub mod renderers; // Ensure this line is present and public
pub mod shm;
pub mod surface_management;
pub mod wayland_server;
pub mod shell; // Changed from xdg_shell
pub mod nova_compositor_logic;
pub mod composition_engine;
pub mod scene_graph;
pub mod texture_manager; // Added texture_manager module

// Optionally, re-export TextureManager if it's commonly used.
pub use texture_manager::TextureManager;
