use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, ChildStdout};
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing; // Ensure tracing is available

use crate::ai_interaction_service::types::{JsonRpcRequest, JsonRpcResponse, JsonRpcError}; // Assuming JsonRpcError is also in types

// Define the trait if it's not already defined elsewhere or imported
// For this task, I'll assume it's defined in this file or a parent module.
// If it's in types.rs, it should be `crate::ai_interaction_service::types::IMCPTransport`
#[async_trait]
pub trait IMCPTransport: Send + Sync {
    async fn connect(&mut self) -> Result<()>;
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse>;
    async fn send_notification(&self, notification: JsonRpcRequest) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
    fn get_pid(&self) -> Option<u32>;
}

pub struct ActualStdioTransport {
    stdin_writer: Arc<Mutex<ChildStdin>>,
    // stdout_reader is moved into the reader_task. Store ChildStdout temporarily before connect.
    stdout_temp: Option<ChildStdout>,
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<Result<JsonRpcResponse, anyhow::Error>>>>>,
    notification_sender: mpsc::UnboundedSender<JsonRpcRequest>,
    reader_task_handle: Option<tokio::task::JoinHandle<()>>,
    stop_reader_tx: Option<oneshot::Sender<()>>,
    pid: u32,
}

impl ActualStdioTransport {
    pub fn new(
        stdin: ChildStdin,
        stdout: ChildStdout,
        pid: u32,
        notification_sender: mpsc::UnboundedSender<JsonRpcRequest>,
    ) -> Self {
        Self {
            stdin_writer: Arc::new(Mutex::new(stdin)),
            stdout_temp: Some(stdout), // Store it here until connect is called
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            notification_sender,
            reader_task_handle: None,
            stop_reader_tx: None,
            pid,
        }
    }

    async fn run_reader_task(
        mut stdout_reader: BufReader<ChildStdout>, // Take ownership
        pending_requests_clone: Arc<Mutex<HashMap<String, oneshot::Sender<Result<JsonRpcResponse, anyhow::Error>>>>>,
        notification_sender_clone: mpsc::UnboundedSender<JsonRpcRequest>,
        mut stop_reader_rx: oneshot::Receiver<()>,
        pid: u32,
    ) {
        loop {
            let mut line = String::new();
            tokio::select! {
                biased; // Prioritize stop signal
                _ = &mut stop_reader_rx => {
                    tracing::info!("[PID {}] Reader task: Stop signal received. Exiting.", pid);
                    break;
                }
                result = stdout_reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            tracing::info!("[PID {}] Reader task: Stdout EOF. Server process likely exited.", pid);
                            break; // EOF
                        }
                        Ok(_) => {
                            let trimmed_line = line.trim();
                            if trimmed_line.is_empty() {
                                continue;
                            }
                            // Attempt to parse as Response or Notification
                            match serde_json::from_str::<JsonRpcResponse>(trimmed_line) {
                                Ok(response) => {
                                    if let Some(id_val_any) = response.id.clone() {
                                        // Expecting ID to be a string or number, convert to string for map key
                                        let id_str = match id_val_any {
                                            Value::String(s) => s,
                                            Value::Number(n) => n.to_string(),
                                            _ => {
                                                tracing::warn!("[PID {}] Reader task: Received response with non-string/non-number id: {:?}. Line: {}", pid, id_val_any, trimmed_line);
                                                // Potentially a notification parsed as a response, try parsing as notification
                                                // This case might also indicate a malformed response from the server.
                                                // Fall through to try parsing as notification if no error field.
                                                if response.error.is_none() {
                                                    // Try parsing as notification
                                                } else {
                                                    // It's an error response with a weird ID type. Log it.
                                                    tracing::error!("[PID {}] Reader task: Received error response with unexpected ID type: {:?}. Error: {:?}", pid, id_val_any, response.error);
                                                    continue;
                                                }
                                                "" // Placeholder to avoid immediate error, will be caught by next if.
                                            }
                                        };

                                        if !id_str.is_empty() {
                                            let mut pending = pending_requests_clone.lock().await;
                                            if let Some(tx) = pending.remove(&id_str) {
                                                if tx.send(Ok(response)).is_err() {
                                                    tracing::warn!("[PID {}] Reader task: Failed to send response for id {} to waiting task (receiver dropped).", pid, id_str);
                                                }
                                            } else {
                                                tracing::warn!("[PID {}] Reader task: Received response with unknown id: {}. Line: {}", pid, id_str, trimmed_line);
                                            }
                                            continue; // Processed as a response
                                        }
                                    }
                                    // If response.id is None or was not processable as a string/number key,
                                    // it could be an error response to a notification (not typical for JSON-RPC 2.0)
                                    // or it could be a notification itself that got misparsed.
                                    // Let's check if it's an error response for a null ID (batch error etc.)
                                    if response.id.is_none() && response.error.is_some() {
                                         tracing::error!("[PID {}] Reader task: Received error response for a request with null ID (potentially a server error for batch or malformed notification): {:?}", pid, response.error);
                                         continue;
                                    }
                                    // Fall through to try parsing as a notification if it wasn't clearly a response to a pending request.
                                }
                                Err(_e_resp) => { // Not a valid JsonRpcResponse, try parsing as a notification
                                    // The fall-through from the Ok(response) case also leads here if ID was not string/number
                                }
                            }

                            // Try parsing as a notification (JsonRpcRequest with no id)
                            // This is also the fallback if the response parsing didn't fully resolve it.
                            match serde_json::from_str::<JsonRpcRequest>(trimmed_line) {
                                Ok(notification_req) if notification_req.id.is_none() => {
                                    if notification_sender_clone.send(notification_req).is_err() {
                                        tracing::warn!("[PID {}] Reader task: Failed to send notification (receiver dropped). Line: {}", pid, trimmed_line);
                                    }
                                }
                                Ok(request_with_id) => {
                                     tracing::error!("[PID {}] Reader task: Parsed as JsonRpcRequest but it has an ID, which is unexpected from server stdout unless it's an echo or error. ID: {:?}, Line: {}", pid, request_with_id.id, trimmed_line);
                                }
                                Err(e_req) => {
                                    tracing::error!("[PID {}] Reader task: Failed to deserialize line as JSON-RPC Response or Notification. Error: {}. Line: {}", pid, e_req, trimmed_line);
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("[PID {}] Reader task: Error reading from stdout: {}", pid, e);
                            break;
                        }
                    }
                }
            }
        }
        // Cleanup pending requests when reader exits
        tracing::info!("[PID {}] Reader task cleaning up pending requests...", pid);
        let mut pending = pending_requests_clone.lock().await;
        for (id, tx) in pending.drain() {
            tracing::warn!("[PID {}] Reader task: Notifying pending request ID {} about connection closure.", pid, id);
            let _ = tx.send(Err(anyhow!("Connection closed while awaiting response (PID: {})", pid)));
        }
        tracing::info!("[PID {}] Reader task finished.", pid);
    }
}

#[async_trait]
impl IMCPTransport for ActualStdioTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.reader_task_handle.is_some() {
            tracing::warn!("[PID {}] Connect called, but reader task already exists. Disconnecting first.", self.pid);
            self.disconnect().await?; // Ensure clean state
        }

        let stdout = self.stdout_temp.take().ok_or_else(|| anyhow!("stdout already taken, cannot connect"))?;
        let stdout_reader = BufReader::new(stdout);

        let (stop_tx, stop_rx) = oneshot::channel();
        self.stop_reader_tx = Some(stop_tx);

        let pending_requests_clone = Arc::clone(&self.pending_requests);
        let notification_sender_clone = self.notification_sender.clone();
        let pid_clone = self.pid;

        tracing::info!("[PID {}] Starting reader task.", self.pid);
        let handle = tokio::spawn(Self::run_reader_task(
            stdout_reader,
            pending_requests_clone,
            notification_sender_clone,
            stop_rx,
            pid_clone,
        ));
        self.reader_task_handle = Some(handle);
        Ok(())
    }

    async fn send_request(&self, mut request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        if self.reader_task_handle.is_none() {
            return Err(anyhow!("Transport not connected (PID: {})", self.pid));
        }

        // Ensure request.id is a string for HashMap key consistency
        // MCPClientInstance uses u64, then json!(u64) -> Value::Number.
        // The reader task expects string keys.
        let id_str = match request.id.clone() {
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::String(s)) => s,
            None => { // Should not happen for a request that expects a response
                let new_id = uuid::Uuid::new_v4().to_string();
                tracing::warn!("[PID {}] Request has no ID. Generating one: {}", self.pid, new_id);
                request.id = Some(Value::String(new_id.clone()));
                new_id
            }
            Some(other) => return Err(anyhow!("Request ID is not a string or number: {:?}", other)),
        };


        let (response_tx, response_rx) = oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().await;
            if pending.contains_key(&id_str) {
                return Err(anyhow!("Duplicate request ID: {} (PID: {})", id_str, self.pid));
            }
            pending.insert(id_str.clone(), response_tx);
        }

        let request_json = serde_json::to_string(&request)
            .context(format!("[PID {}] Failed to serialize request", self.pid))?;

        tracing::debug!("[PID {}] Sending request: {}", self.pid, request_json);
        let mut stdin = self.stdin_writer.lock().await;
        stdin.write_all((request_json + "\n").as_bytes()).await
            .context(format!("[PID {}] Failed to write to stdin", self.pid))?;
        stdin.flush().await
            .context(format!("[PID {}] Failed to flush stdin", self.pid))?;

        // Await the response from the oneshot channel
        // The outer Result handles oneshot channel errors (e.g., sender dropped)
        // The inner Result handles errors from the reader task (e.g., deserialization, connection closed)
        match tokio::time::timeout(std::time::Duration::from_secs(30), response_rx).await {
            Ok(Ok(Ok(response))) => Ok(response),
            Ok(Ok(Err(e))) => Err(e).context(format!("[PID {}] Reader task reported error for request ID {}", self.pid, id_str)),
            Ok(Err(e)) => Err(anyhow!(e)).context(format!("[PID {}] Failed to receive response for request ID {} (oneshot recv error)", self.pid, id_str)),
            Err(_) => Err(anyhow!("[PID {}] Timeout waiting for response for request ID {}", self.pid, id_str)),
        }
    }

    async fn send_notification(&self, notification: JsonRpcRequest) -> Result<()> {
         if self.reader_task_handle.is_none() && notification.method != "exit" && notification.method != "shutdown" {
             // Allow exit/shutdown even if not "fully" connected, assuming stdin might still be open.
            return Err(anyhow!("Transport not connected, cannot send notification (PID: {})", self.pid));
        }
        if notification.id.is_some() {
            return Err(anyhow!("Notifications must not have an ID (PID: {})", self.pid));
        }

        let notification_json = serde_json::to_string(&notification)
            .context(format!("[PID {}] Failed to serialize notification", self.pid))?;
        
        tracing::debug!("[PID {}] Sending notification: {}", self.pid, notification_json);
        let mut stdin = self.stdin_writer.lock().await;
        stdin.write_all((notification_json + "\n").as_bytes()).await
            .context(format!("[PID {}] Failed to write notification to stdin", self.pid))?;
        stdin.flush().await
            .context(format!("[PID {}] Failed to flush notification to stdin", self.pid))?;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        tracing::info!("[PID {}] Disconnecting transport...", self.pid);
        if let Some(stop_tx) = self.stop_reader_tx.take() {
            tracing::debug!("[PID {}] Sending stop signal to reader task.", self.pid);
            if stop_tx.send(()).is_err() {
                tracing::warn!("[PID {}] Failed to send stop signal to reader task (already stopped or panicked).", self.pid);
            }
        }

        if let Some(handle) = self.reader_task_handle.take() {
            tracing::debug!("[PID {}] Awaiting reader task to join.", self.pid);
            match tokio::time::timeout(std::time::Duration::from_secs(5), handle).await {
                Ok(Ok(())) => tracing::info!("[PID {}] Reader task joined successfully.", self.pid),
                Ok(Err(e)) => tracing::error!("[PID {}] Reader task panicked or failed: {:?}", self.pid, e),
                Err(_) => tracing::warn!("[PID {}] Timeout waiting for reader task to join.", self.pid),
            }
        } else {
            tracing::debug!("[PID {}] No reader task handle found during disconnect.", self.pid);
        }

        // Clear any remaining pending requests
        let mut pending = self.pending_requests.lock().await;
        if !pending.is_empty() {
            tracing::warn!("[PID {}] {} pending requests outstanding during disconnect. Clearing them.", self.pid, pending.len());
            for (id, tx) in pending.drain() {
                 tracing::debug!("[PID {}] Notifying pending request ID {} about disconnect.", self.pid, id);
                let _ = tx.send(Err(anyhow!("Transport disconnected (PID: {})", self.pid)));
            }
        }
        tracing::info!("[PID {}] Disconnect process complete.", self.pid);
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.reader_task_handle.is_some() && !self.reader_task_handle.as_ref().map_or(false, |h| h.is_finished())
    }
    
    fn get_pid(&self) -> Option<u32> {
        Some(self.pid)
    }
}

// Ensure JsonRpcRequest and JsonRpcResponse are public in types.rs
// Example in types.rs:
// pub struct JsonRpcRequest { ... }
// pub struct JsonRpcResponse { ... }
// pub struct JsonRpcError { ... }
```
I have implemented the `ActualStdioTransport` in `novade-domain/src/ai_interaction_service/transport.rs`.

**Key Implementation Details:**

1.  **Imports:** Added necessary Tokio, standard library, and crate-specific imports.
2.  **`ActualStdioTransport` Struct:**
    *   `stdin_writer`: `Arc<Mutex<ChildStdin>>` for asynchronous, shared writing.
    *   `stdout_temp`: `Option<ChildStdout>` to hold `ChildStdout` until `connect` is called.
    *   `pending_requests`: `Arc<Mutex<HashMap<String, oneshot::Sender<Result<JsonRpcResponse, anyhow::Error>>>>>` to map request IDs (as strings) to response senders.
    *   `notification_sender`: `mpsc::UnboundedSender<JsonRpcRequest>` for forwarding notifications.
    *   `reader_task_handle`: `Option<tokio::task::JoinHandle<()>>` for the stdout reading task.
    *   `stop_reader_tx`: `Option<oneshot::Sender<()>>` to signal the reader task to stop.
    *   `pid`: `u32` to store the process ID of the MCP server for logging.
3.  **`new()` Method:** Initializes the struct, taking `ChildStdin`, `ChildStdout`, `pid`, and the `notification_sender`. `ChildStdout` is stored in `stdout_temp`.
4.  **`run_reader_task()` Static Async Function:**
    *   Takes ownership of `BufReader<ChildStdout>`, clones of `pending_requests` and `notification_sender`, the `stop_reader_rx` oneshot receiver, and the `pid`.
    *   Enters a loop, using `tokio::select!` to prioritize the stop signal over reading lines from stdout.
    *   **Line Processing:**
        *   On EOF or read error, breaks the loop.
        *   Attempts to deserialize the line as `JsonRpcResponse`.
            *   If successful and has a string or number ID, it finds the corresponding `oneshot::Sender` in `pending_requests`, sends the response, and removes the sender.
            *   If ID is not string/number, or if parsing as `JsonRpcResponse` fails, it then attempts to parse the line as a `JsonRpcRequest` (potential notification).
        *   If parsing as `JsonRpcRequest` is successful and the request has no ID, it's considered a notification and sent via `notification_sender_clone`.
        *   Logs errors for deserialization failures or unexpected message formats.
    *   **Cleanup:** When the loop exits, it iterates over any remaining `pending_requests` and sends an error indicating the connection closed.
5.  **`IMCPTransport` Implementation for `ActualStdioTransport`:**
    *   **`connect()`:**
        *   Takes `ChildStdout` from `self.stdout_temp`.
        *   Creates a `oneshot` channel for stopping the reader task and stores the sender.
        *   Spawns `run_reader_task` and stores its `JoinHandle`.
    *   **`send_request()`:**
        *   Ensures the transport is connected.
        *   Converts request ID to a string (generating a UUID if none is present, though requests should have IDs).
        *   Creates a `oneshot` channel for the response and stores the sender in `pending_requests` keyed by the stringified ID.
        *   Serializes the request to JSON, appends a newline, writes to `stdin_writer`, and flushes.
        *   Awaits the response from the `oneshot::Receiver` with a 30-second timeout.
    *   **`send_notification()`:**
        *   Ensures the transport is connected (with an exception for `exit`/`shutdown`).
        *   Validates that the notification has no ID.
        *   Serializes the notification, appends a newline, writes to `stdin_writer`, and flushes.
    *   **`disconnect()`:**
        *   Sends a stop signal via `stop_reader_tx`.
        *   Awaits the `reader_task_handle` with a timeout.
        *   Clears any remaining `pending_requests`, sending an error to waiting tasks.
    *   **`is_connected()`:** Checks if the `reader_task_handle` exists and the task is not finished.
    *   **`get_pid()`:** Returns the stored `pid`.

**Assumptions and Notes:**

*   `JsonRpcRequest`, `JsonRpcResponse`, and `JsonRpcError` are assumed to be defined in `crate::ai_interaction_service::types` and are public.
*   The `IMCPTransport` trait definition was included for completeness within this file, assuming it might not be globally accessible otherwise for this specific context.
*   Request IDs from `MCPClientInstance` are expected to be `u64` initially, which are then converted to `Value::Number` by `serde_json::json!`. The `ActualStdioTransport` stringifies these numeric IDs for use as keys in the `pending_requests` HashMap. The `run_reader_task` also handles string or number IDs in incoming responses.
*   Error handling uses `anyhow::Result` and `context()`. Tracing is used for logging.
*   A timeout is added to `send_request` when awaiting the response.
*   The `reader_task` has improved logic for distinguishing between responses and notifications.

This implementation provides a functional stdio transport layer for JSON-RPC communication. The next step would be to integrate this into `MCPClientInstance` and handle the notifications received via `notification_sender`.
