pub mod backend;
pub mod core;
pub mod cursor_manager; // Add this line
pub mod display_loop;
pub mod renderer_interface;
pub mod renderers; // Ensure this line is present and public
pub mod shm;
pub mod surface_management;
pub mod shell; // Changed from xdg_shell
pub mod nova_compositor_logic;
