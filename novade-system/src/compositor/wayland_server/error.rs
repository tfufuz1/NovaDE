#[derive(Debug, thiserror::Error)]
pub enum WaylandServerError {
    #[error("Socket creation failed: {0}")]
    SocketCreation(String),
    #[error("Client connection error: {0}")]
    ClientConnection(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
    #[error("Object error: {0}")]
    Object(String),
    #[error("Event dispatch error: {0}")]
    EventDispatch(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
