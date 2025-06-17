// Declare the modules within the wayland directory
pub mod buffer;
pub mod client;
pub mod event_loop;
pub mod object;
pub mod seat;          // Added seat module
pub mod server;
pub mod shm;
pub mod subcompositor;
pub mod subsurface;
pub mod surface;
pub mod wire;

// Re-export key public types for easier access from outside `wayland` module
pub use buffer::{WlBuffer, BufferId};
pub use client::{Client, ClientId, ClientManager, ClientError};
pub use event_loop::{Event, EventLoop, EventLoopContext, EventTypeDiscriminant, Callback, EventLoopError};
pub use object::{WaylandObject, ObjectId, ObjectManager, ObjectError, ProtocolError, RequestContext};
pub use seat::{WlSeat, WlKeyboard, SeatCapability}; // Re-export seat types
pub use server::{Server, ServerError, UnixStream};
pub use shm::{WlShm, WlShmPool, ShmFormat};
pub use subcompositor::WlSubcompositor;
pub use subsurface::{WlSubsurface, SubsurfaceSyncMode, SubsurfacePendingState, SubsurfaceCommittedState};
pub use surface::{Surface, SurfaceId, Rect, SurfaceRole, SurfacePendingState as SurfaceCorePendingState, SurfaceCommittedState as SurfaceCoreCommittedState};
pub use wire::{MessageHeader, WlArgument, ArgType, serialize_message, deserialize_message, SerializationError, DeserializationError};
