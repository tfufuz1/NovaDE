use tokio::process::{ChildStdin, ChildStdout};
use std::fmt;

// StdioProcess will hold the communication handles and PID for a spawned MCP server.
// ChildStdin and ChildStdout are owned by this struct and will be passed to
// the StdioTransportHandler in the domain layer.
pub struct StdioProcess {
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
    pub pid: u32,
}

// Custom Debug implementation because ChildStdin and ChildStdout don't implement Debug
impl fmt::Debug for StdioProcess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StdioProcess")
         .field("pid", &self.pid)
         .field("stdin", &"Opaque<ChildStdin>") // Indicate presence without trying to format
         .field("stdout", &"Opaque<ChildStdout>") // Indicate presence without trying to format
         .finish()
    }
}
