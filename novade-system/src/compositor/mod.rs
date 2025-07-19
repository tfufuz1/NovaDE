// NovaDE Compositor Module
// Main entry point for compositor related functionalities.

// Core compositor logic
pub mod core;
// State management
pub mod state;
// Rendering abstraction and backends
pub mod render;
// Wayland protocol handlers
pub mod handlers;
// XDG Shell specific logic
pub mod xdg_shell;
// Layer Shell specific logic
pub mod layer_shell;
// Output management (including zxdg_output_manager_v1)
pub mod output_manager; // Renamed from outputs for clarity
// Input handling and integration
pub mod input;
// XWayland integration
pub mod xwayland;
// Error types for the compositor
pub mod errors;
pub mod window_info_provider;


// Existing modules (to be reviewed and integrated/removed as needed)
pub mod backend; // E.g. DRM, Winit backends
pub mod config;
// pub mod cursor_manager; // Cursor management might be part of input.rs or a dedicated module
pub mod display_loop;     // Potentially part of core.rs or backend management
pub mod renderer_interface; // Might be superseded by render.rs
pub mod renderers;        // Specific renderer implementations (e.g., GL, Vulkan folders within)
pub mod shm;              // SHM buffer handling, likely integrated into core Smithay states
pub mod surface_management; // Surface roles and state, integrated into shell handlers
pub mod wayland_server;   // General Wayland server setup, part of core.rs
pub mod shell;            // Generic shell module, now split into xdg_shell.rs, layer_shell.rs
// pub mod nova_compositor_logic; // High-level logic, to be integrated
// pub mod composition_engine;    // Scene graph / composition logic, part of rendering
// pub mod scene_graph;
pub mod display_management; // Domain level display management, may differ from output_manager.rs
pub mod animations;
pub mod workspaces;
pub mod tiling;

// Remove if outputs module is fully replaced by output_manager
// pub mod outputs;

#[cfg(test)]
mod tiling_tests;
