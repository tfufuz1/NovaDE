use crate::ai_interaction_service::types::{JsonRpcRequest, JsonRpcResponse, MCPError};
use novade_system::mcp_client_service::StdioProcess; // From novade-system
use async_trait::async_trait;
use serde_json::Value;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout},
    sync::Mutex, // Using Tokio's Mutex for stdout_reader if shared access is complex
};

#[async_trait]
pub trait IMCPTransport: Send + Sync {
    async fn connect(&mut self, process: StdioProcess) -> Result<(), MCPError>;
    // send_request now expects the MCPClientInstance to handle matching request ID to response.
    // Thus, it doesn't directly return JsonRpcResponse. Responses will come via message_handler.
    async fn send_request(&mut self, request: JsonRpcRequest) -> Result<(), MCPError>;
    async fn send_notification(&mut self, notification: JsonRpcRequest) -> Result<(), MCPError>;
    async fn disconnect(&mut self) -> Result<(), MCPError>;
    async fn register_message_handler(
        &mut self,
        handler: Arc<dyn Fn(Result<Value, MCPError>) + Send + Sync>,
    );
    // No direct receive_message; messages are pushed via the handler.
}

pub struct StdioTransportHandler {
    stdin: Option<ChildStdin>,
    // stdout_reader is now an Option<Arc<Mutex<...>>> to allow it to be moved into the reading task
    stdout_reader: Option<Arc<Mutex<BufReader<ChildStdout>>>>,
    message_handler: Option<Arc<dyn Fn(Result<Value, MCPError>) + Send + Sync>>,
    // For signalling the reading task to stop
    should_terminate_reader: Arc<AtomicBool>,
    reader_task_handle: Option<tokio::task::JoinHandle<()>>,
}

impl StdioTransportHandler {
    pub fn new() -> Self {
        StdioTransportHandler {
            stdin: None,
            stdout_reader: None,
            message_handler: None,
            should_terminate_reader: Arc::new(AtomicBool::new(false)),
            reader_task_handle: None,
        }
    }
}

#[async_trait]
impl IMCPTransport for StdioTransportHandler {
    async fn connect(&mut self, process: StdioProcess) -> Result<(), MCPError> {
        println!("[StdioTransportHandler] Connecting with StdioProcess PID: {}", process.pid);
        self.stdin = Some(process.stdin);
        let stdout_buf_reader = BufReader::new(process.stdout);
        self.stdout_reader = Some(Arc::new(Mutex::new(stdout_buf_reader)));
        self.should_terminate_reader.store(false, Ordering::SeqCst); // Reset termination flag

        if let Some(handler) = self.message_handler.clone() {
            if let Some(stdout_reader_arc) = self.stdout_reader.clone() {
                let terminate_flag = self.should_terminate_reader.clone();
                
                let task = tokio::spawn(async move {
                    loop {
                        if terminate_flag.load(Ordering::SeqCst) {
                            println!("[StdioTransportHandler ReaderTask] Termination signal received.");
                            handler(Err(MCPError::ConnectionClosed)); // Notify handler about termination
                            break;
                        }

                        let mut line_buf = String::new();
                        let mut reader_guard = stdout_reader_arc.lock().await;
                        
                        // Use select for non-blocking check of terminate_flag and read_line
                        tokio::select! {
                            _ = tokio::time::sleep(Duration::from_millis(100)) => { // Check termination periodically
                                if terminate_flag.load(Ordering::SeqCst) {
                                    println!("[StdioTransportHandler ReaderTask] Terminating due to flag check.");
                                    handler(Err(MCPError::ConnectionClosed));
                                    break;
                                }
                                continue; // Continue to next iteration of select
                            }
                            read_result = reader_guard.read_line(&mut line_buf) => {
                                match read_result {
                                    Ok(0) => { // EOF
                                        println!("[StdioTransportHandler ReaderTask] EOF received. Connection closed by server.");
                                        handler(Err(MCPError::ConnectionClosed));
                                        break;
                                    }
                                    Ok(_) => {
                                        // Trim whitespace, especially newline characters
                                        let trimmed_line = line_buf.trim();
                                        if trimmed_line.is_empty() {
                                            continue; // Skip empty lines if any
                                        }
                                        println!("[StdioTransportHandler ReaderTask] Received line: {}", trimmed_line);
                                        match serde_json::from_str::<Value>(trimmed_line) {
                                            Ok(json_value) => {
                                                handler(Ok(json_value));
                                            }
                                            Err(e) => {
                                                eprintln!("[StdioTransportHandler ReaderTask] JSON parse error: {} for line: {}", e, trimmed_line);
                                                handler(Err(MCPError::JsonRpcParseError(e.to_string())));
                                            }
                                        }
                                    }
                                    Err(e) => { // IO error
                                        eprintln!("[StdioTransportHandler ReaderTask] Error reading line: {}", e);
                                        if !terminate_flag.load(Ordering::SeqCst) { // Avoid double error if already terminating
                                             handler(Err(MCPError::TransportIOError(e.to_string())));
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    println!("[StdioTransportHandler ReaderTask] Exiting normally.");
                });
                self.reader_task_handle = Some(task);
                println!("[StdioTransportHandler] Reader task spawned.");
            } else {
                 println!("[StdioTransportHandler] Cannot start reader task: stdout_reader not available.");
                 return Err(MCPError::InternalError); // Or some other appropriate error
            }
        } else {
            println!("[StdioTransportHandler] No message handler registered. Incoming messages will not be processed.");
            // This is not necessarily an error for connect, but operations requiring responses will fail.
        }
        
        println!("[StdioTransportHandler] Connected successfully.");
        Ok(())
    }

    async fn send_request(&mut self, request: JsonRpcRequest) -> Result<(), MCPError> {
        if let Some(stdin) = self.stdin.as_mut() {
            let request_str = serde_json::to_string(&request)
                .map_err(|e| MCPError::JsonRpcParseError(e.to_string()))?; // Should be very rare for outgoing
            
            // Append newline for framing
            let framed_request = format!("{}\n", request_str);
            
            println!("[StdioTransportHandler] Sending request (ID: {:?}): {}", request.id, framed_request.trim());
            
            stdin.write_all(framed_request.as_bytes()).await
                .map_err(|e| MCPError::TransportIOError(e.to_string()))?;
            stdin.flush().await.map_err(|e| MCPError::TransportIOError(e.to_string()))?;
            Ok(())
        } else {
            Err(MCPError::ConnectionClosed) // Or NotConnected if that's more appropriate
        }
    }

    async fn send_notification(&mut self, notification: JsonRpcRequest) -> Result<(), MCPError> {
         if let Some(stdin) = self.stdin.as_mut() {
            let notification_str = serde_json::to_string(&notification)
                .map_err(|e| MCPError::JsonRpcParseError(e.to_string()))?;
            
            let framed_notification = format!("{}\n", notification_str);

            println!("[StdioTransportHandler] Sending notification: {}", framed_notification.trim());

            stdin.write_all(framed_notification.as_bytes()).await
                .map_err(|e| MCPError::TransportIOError(e.to_string()))?;
            stdin.flush().await.map_err(|e| MCPError::TransportIOError(e.to_string()))?;
            Ok(())
        } else {
            Err(MCPError::ConnectionClosed)
        }
    }
    
    async fn register_message_handler(
        &mut self,
        handler: Arc<dyn Fn(Result<Value, MCPError>) + Send + Sync>,
    ) {
        println!("[StdioTransportHandler] Message handler registered.");
        self.message_handler = Some(handler);
        // If already connected, and reader task was not started due to no handler,
        // it might be desirable to start it now. However, current logic starts it in connect().
        // This implies handler should be registered *before* connect for messages to be processed.
    }

    async fn disconnect(&mut self) -> Result<(), MCPError> {
        println!("[StdioTransportHandler] Disconnecting...");
        self.should_terminate_reader.store(true, Ordering::SeqCst);
        
        if let Some(handle) = self.reader_task_handle.take() {
            println!("[StdioTransportHandler] Waiting for reader task to complete...");
            if let Err(e) = tokio::time::timeout(std::time::Duration::from_secs(2), handle).await {
                 eprintln!("[StdioTransportHandler] Timeout waiting for reader task to join: {:?}", e);
                 // The task might be stuck in a read; it should eventually exit due to the flag or I/O error.
            } else {
                println!("[StdioTransportHandler] Reader task joined.");
            }
        }

        // Drop stdin to close the pipe, which might signal the child process
        if let Some(mut stdin) = self.stdin.take() {
            if let Err(e) = stdin.shutdown().await {
                 eprintln!("[StdioTransportHandler] Error shutting down stdin: {:?}", e);
            }
        }
        self.stdout_reader.take(); // Drop stdout reader

        println!("[StdioTransportHandler] Disconnected.");
        Ok(())
    }
}

impl Default for StdioTransportHandler {
    fn default() -> Self {
        Self::new()
    }
}

// Added for the reader task loop, replace with std::time::Duration if not already used
use std::time::Duration;
