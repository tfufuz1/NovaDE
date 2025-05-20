//! Error handling for the NovaDE system layer.
//!
//! This module defines system-specific errors for the NovaDE desktop environment.

use thiserror::Error;
use novade_core::error::CoreError;
use novade_domain::error::DomainError;

/// System error type.
#[derive(Debug, Error)]
pub enum SystemError {
    /// Core error.
    #[error(transparent)]
    Core(#[from] CoreError),
    
    /// Domain error.
    #[error(transparent)]
    Domain(#[from] DomainError),
    
    /// Compositor error.
    #[error(transparent)]
    Compositor(#[from] CompositorError),
    
    /// Input error.
    #[error(transparent)]
    Input(#[from] InputError),
    
    /// D-Bus error.
    #[error(transparent)]
    DBus(#[from] DBusError),
    
    /// Audio error.
    #[error(transparent)]
    Audio(#[from] AudioError),
    
    /// MCP error.
    #[error(transparent)]
    Mcp(#[from] McpError),
    
    /// Portals error.
    #[error(transparent)]
    Portals(#[from] PortalsError),
    
    /// Power management error.
    #[error(transparent)]
    PowerManagement(#[from] PowerManagementError),
    
    /// Other error.
    #[error("System error: {0}")]
    Other(String),
}

/// Compositor error type.
#[derive(Debug, Error)]
pub enum CompositorError {
    /// Smithay error.
    #[error("Smithay error: {0}")]
    Smithay(String),
    
    /// Surface error.
    #[error("Surface error: {0}")]
    Surface(String),
    
    /// XDG shell error.
    #[error("XDG shell error: {0}")]
    XdgShell(String),
    
    /// Layer shell error.
    #[error("Layer shell error: {0}")]
    LayerShell(String),
    
    /// Renderer error.
    #[error("Renderer error: {0}")]
    Renderer(String),
    
    /// Output error.
    #[error("Output error: {0}")]
    Output(String),
    
    /// Other error.
    #[error("Compositor error: {0}")]
    Other(String),
}

/// Input error type.
#[derive(Debug, Error)]
pub enum InputError {
    /// Libinput error.
    #[error("Libinput error: {0}")]
    Libinput(String),
    
    /// XKB error.
    #[error("XKB error: {0}")]
    Xkb(String),
    
    /// Seat error.
    #[error("Seat error: {0}")]
    Seat(String),
    
    /// Device error.
    #[error("Device error: {0}")]
    Device(String),
    
    /// Focus error.
    #[error("Focus error: {0}")]
    Focus(String),
    
    /// Other error.
    #[error("Input error: {0}")]
    Other(String),
}

/// D-Bus error type.
#[derive(Debug, Error)]
pub enum DBusError {
    /// Connection error.
    #[error("D-Bus connection error: {0}")]
    Connection(String),
    
    /// Method call error.
    #[error("D-Bus method call error: {0}")]
    MethodCall(String),
    
    /// Signal error.
    #[error("D-Bus signal error: {0}")]
    Signal(String),
    
    /// Property error.
    #[error("D-Bus property error: {0}")]
    Property(String),
    
    /// Interface error.
    #[error("D-Bus interface error: {0}")]
    Interface(String),
    
    /// Other error.
    #[error("D-Bus error: {0}")]
    Other(String),
}

/// Audio error type.
#[derive(Debug, Error)]
pub enum AudioError {
    /// PipeWire error.
    #[error("PipeWire error: {0}")]
    PipeWire(String),
    
    /// Device error.
    #[error("Audio device error: {0}")]
    Device(String),
    
    /// Stream error.
    #[error("Audio stream error: {0}")]
    Stream(String),
    
    /// Volume error.
    #[error("Volume error: {0}")]
    Volume(String),
    
    /// Other error.
    #[error("Audio error: {0}")]
    Other(String),
}

/// MCP error type.
#[derive(Debug, Error)]
pub enum McpError {
    /// Connection error.
    #[error("MCP connection error: {0}")]
    Connection(String),
    
    /// Protocol error.
    #[error("MCP protocol error: {0}")]
    Protocol(String),
    
    /// Authentication error.
    #[error("MCP authentication error: {0}")]
    Authentication(String),
    
    /// Other error.
    #[error("MCP error: {0}")]
    Other(String),
}

/// Portals error type.
#[derive(Debug, Error)]
pub enum PortalsError {
    /// D-Bus error.
    #[error("Portals D-Bus error: {0}")]
    DBus(String),
    
    /// File chooser error.
    #[error("File chooser error: {0}")]
    FileChooser(String),
    
    /// Screenshot error.
    #[error("Screenshot error: {0}")]
    Screenshot(String),
    
    /// Other error.
    #[error("Portals error: {0}")]
    Other(String),
}

/// Power management error type.
#[derive(Debug, Error)]
pub enum PowerManagementError {
    /// D-Bus error.
    #[error("Power management D-Bus error: {0}")]
    DBus(String),
    
    /// UPower error.
    #[error("UPower error: {0}")]
    UPower(String),
    
    /// Logind error.
    #[error("Logind error: {0}")]
    Logind(String),
    
    /// Other error.
    #[error("Power management error: {0}")]
    Other(String),
}
