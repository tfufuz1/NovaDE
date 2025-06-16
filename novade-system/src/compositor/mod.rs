pub mod backend;
pub mod config;
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
pub mod display_management; // Added display_management module
pub mod animations; // Added animations module
pub mod workspaces; // ANCHOR: AddWorkspacesModule
pub mod tiling; // ANCHOR: AddTilingModule
pub mod outputs; // ANCHOR: AddOutputConfigModule
#[cfg(test)]
mod tiling_tests; // ANCHOR: AddTilingTestsModule
