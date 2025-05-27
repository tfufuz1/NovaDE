// Exports from the xdg_shell module

pub mod types;
pub mod errors;
pub mod handlers;

// Re-export key items for easier use by the parent `compositor` module
// or other parts of the system.

pub use self::types::{ManagedWindow, DomainWindowIdentifier, XdgSurfaceVariant};
pub use self::errors::XdgShellError;
pub use self::handlers::{create_xdg_shell_globals}; // XdgShellHandler is implemented on DesktopState

// The DesktopState itself will have XdgShellState and XdgActivationState fields,
// and the necessary GlobalDispatch and Handler trait implementations are in
// handlers.rs (for XDG Shell) and core/state.rs (for XDG Activation).

// Smithay's delegation macros (delegate_xdg_shell!, delegate_xdg_activation!)
// will be called on DesktopState in core/state.rs or directly where DesktopState is defined
// to ensure the handler methods are correctly wired up.
// We added delegate_xdg_shell! in handlers.rs.
// delegate_xdg_activation! is likely in core/state.rs or needs to be added there.

// We need to ensure that DesktopState in core/state.rs has the xdg_shell delegate:
// smithay::delegate_xdg_shell!(DesktopState);
// This was added at the end of handlers.rs.

// And for activation (if not already present in core/state.rs where XdgActivationHandler is impl'd):
// smithay::delegate_xdg_activation!(DesktopState);
// This is already present in core/state.rs from the looks of the previous changes.

// This mod.rs primarily serves to structure the xdg_shell related code
// and re-export its public interface. The actual integration into DesktopState
// happens via direct field additions and trait implementations.
