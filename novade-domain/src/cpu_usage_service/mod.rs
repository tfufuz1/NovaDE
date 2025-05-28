use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, oneshot}; // Added oneshot
use uuid::Uuid;
use async_trait::async_trait;
use serde_json::Value as JsonValue; // Renamed to avoid conflict if types::Value is used

// Assuming these types are correctly exposed from ai_interaction_service module
use crate::ai_interaction_service::client_instance::MCPClientInstance;
use crate::ai_interaction_service::connection_service::MCPConnectionService;
use crate::ai_interaction_service::types::{JsonRpcRequest}; // Removed unused: MCPServerConfig, ServerId
use crate::error::DomainError; // Assuming this is the main error enum from novade-domain/src/error.rs

// Define a type alias for ServerId if it's commonly used string, or import if it's a distinct type
type ServerId = String;


// --- Constants ---
const CPU_SERVER_ID_DEFAULT: &str = "cpu_mcp_server"; // Default ServerId for the CPU server
const CPU_RESOURCE_URI_DEFAULT: &str = "cpu/usage_percent";

// --- Public Types ---
pub type SubscriptionId = Uuid;

// Temporary struct for deserializing the content from the server.
// Ideally, this would be a shared type.
#[derive(serde::Deserialize, Debug, Clone)]
struct CpuUsageContentFromServer {
    percentage: f64,
}

// --- Service Trait ---
#[async_trait]
pub trait ICpuUsageService: Send + Sync {
    async fn get_current_cpu_percentage(&self) -> Result<f64, DomainError>;
    async fn subscribe_to_cpu_updates(
        &self,
        sender: mpsc::UnboundedSender<Result<f64, DomainError>>,
    ) -> Result<SubscriptionId, DomainError>;
    async fn unsubscribe_from_cpu_updates(&self, id: SubscriptionId) -> Result<(), DomainError>;
}

// --- Service Implementation ---

// State for the single, shared MCP subscription to the CPU server's resource
struct ActiveMcpSubscription {
    #[allow(dead_code)] // May be used later for re-establishing connection or debugging
    notification_rx_from_client: mpsc::UnboundedReceiver<JsonRpcRequest>,
    processing_task_handle: tokio::task::JoinHandle<()>,
    stop_processing_tx: oneshot::Sender<()>, // To stop the processing task
}

pub struct DefaultCpuUsageService {
    connection_service: Arc<Mutex<MCPConnectionService>>,
    cpu_server_id: ServerId,
    // Stores senders for each active subscriber to this DefaultCpuUsageService
    subscribers: Arc<Mutex<HashMap<SubscriptionId, mpsc::UnboundedSender<Result<f64, DomainError>>>>>,
    // Holds the active subscription to the underlying MCP server resource
    active_mcp_sub: Arc<Mutex<Option<ActiveMcpSubscription>>>,
}

impl DefaultCpuUsageService {
    pub fn new(
        connection_service: Arc<Mutex<MCPConnectionService>>,
        cpu_server_id: Option<ServerId>, // Allow overriding the default server ID
    ) -> Self {
        DefaultCpuUsageService {
            connection_service,
            cpu_server_id: cpu_server_id.unwrap_or_else(|| CPU_SERVER_ID_DEFAULT.to_string()),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            active_mcp_sub: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl ICpuUsageService for DefaultCpuUsageService {
    async fn get_current_cpu_percentage(&self) -> Result<f64, DomainError> {
        let conn_service_guard = self.connection_service.lock().await;
        let client_instance_arc = conn_service_guard.get_client_instance(&self.cpu_server_id)
            .ok_or_else(|| DomainError::ServiceMisconfigured(format!("CPU MCP server instance '{}' not found.", self.cpu_server_id)))?;
        
        let mut client_instance = client_instance_arc.lock().await;

        // Ensure client is connected (connect_and_initialize should have been called previously by ConnectionService)
        if *client_instance.get_connection_status() != crate::ai_interaction_service::types::ConnectionStatus::Connected {
            return Err(DomainError::MCPConnection(format!("CPU MCP server '{}' is not connected.", self.cpu_server_id)));
        }

        let params = serde_json::json!({
            "uri": CPU_RESOURCE_URI_DEFAULT,
        });

        match client_instance.send_request_internal("resources/read".to_string(), params).await {
            Ok(response) => {
                if let Some(err) = response.error {
                    return Err(DomainError::MCPRequest(format!("Error from server: code={}, message={}", err.code, err.message)));
                }
                let result = response.result.ok_or(DomainError::MCPRequest("Response missing result".to_string()))?;
                // Assuming CpuUsageContent { percentage: f64 } is the structure from cpu_mcp_server
                let cpu_content: CpuUsageContentFromServer = serde_json::from_value(result)
                    .map_err(|e| DomainError::SerdeError(format!("Failed to deserialize CpuUsageContent: {}", e)))?;
                Ok(cpu_content.percentage)
            }
            Err(e) => Err(DomainError::MCPRequest(format!("Failed to send resources/read request: {}", e))),
        }
    }

    async fn subscribe_to_cpu_updates(
        &self,
        sender: mpsc::UnboundedSender<Result<f64, DomainError>>,
    ) -> Result<SubscriptionId, DomainError> {
        let subscription_id = Uuid::new_v4();
        // Add to local subscribers first
        self.subscribers.lock().await.insert(subscription_id, sender);
        
        let mut active_sub_guard = self.active_mcp_sub.lock().await;

        if active_sub_guard.is_none() {
            // No active MCP subscription, need to create one
            let conn_service_guard = self.connection_service.lock().await;
            let client_instance_arc = conn_service_guard.get_client_instance(&self.cpu_server_id)
                .ok_or_else(|| DomainError::ServiceMisconfigured(format!("CPU MCP server instance '{}' not found for subscription.", self.cpu_server_id)))?;
            
            let mut client_instance = client_instance_arc.lock().await;

            if *client_instance.get_connection_status() != crate::ai_interaction_service::types::ConnectionStatus::Connected {
                 // Attempt to connect and initialize if not connected. This might be redundant if ConnectionService guarantees it.
                // For now, assume ConnectionService handles initial connection. If not, this would be the place.
                // However, connect_and_initialize also re-initializes, which might not be desired here.
                // This indicates a potential design consideration: should this service try to manage connections
                // or assume ConnectionService has already established it?
                // For now, returning an error if not connected.
                return Err(DomainError::MCPConnection(format!("CPU MCP server '{}' is not connected for subscription.", self.cpu_server_id)));
            }

            let notification_rx_from_client = client_instance.take_notification_receiver()
                .ok_or_else(|| DomainError::ServiceMisconfigured("Notification channel unavailable from MCPClientInstance.".to_string()))?;

            let subscribe_params = serde_json::json!({
                "uri": CPU_RESOURCE_URI_DEFAULT,
            });
            
            match client_instance.send_request_internal("resources/subscribe".to_string(), subscribe_params).await {
                Ok(response) => {
                    if let Some(err) = response.error {
                        // Remove the subscriber we just added if MCP subscription failed
                        self.subscribers.lock().await.remove(&subscription_id);
                        return Err(DomainError::SubscriptionFailed(format!("MCP subscribe error: code={}, message={}", err.code, err.message)));
                    }
                    // Check response.result for success if needed by the specific MCP server.
                    // For example, some servers might return a subscription ID or status.
                    // Assuming a successful response (no error) means subscribed.
                    tracing::info!("[CpuUsageService][{}] Successfully subscribed to MCP resource URI {}", self.cpu_server_id, CPU_RESOURCE_URI_DEFAULT);
                }
                Err(e) => {
                    self.subscribers.lock().await.remove(&subscription_id);
                    return Err(DomainError::SubscriptionFailed(format!("Failed to send resources/subscribe request: {}", e)));
                }
            }

            let (stop_tx, stop_rx) = oneshot::channel();
            let task_subscribers_ref = Arc::clone(&self.subscribers);
            let server_id_clone = self.cpu_server_id.clone(); // Clone for the task
            let resource_uri_clone = CPU_RESOURCE_URI_DEFAULT.to_string();

            let processing_task_handle = tokio::spawn(process_cpu_notifications_task(
                notification_rx_from_client,
                task_subscribers_ref,
                stop_rx,
                server_id_clone,
                resource_uri_clone,
            ));

            *active_sub_guard = Some(ActiveMcpSubscription {
                notification_rx_from_client, // This has been moved into the task
                processing_task_handle,
                stop_processing_tx: stop_tx,
            });
        }
        // Else, active_mcp_sub is Some, meaning we are already subscribed to the MCP server.
        // The new subscriber will receive updates via the existing process_cpu_notifications_task.
        Ok(subscription_id)
    }

    async fn unsubscribe_from_cpu_updates(&self, id: SubscriptionId) -> Result<(), DomainError> {
        let mut subscribers_map = self.subscribers.lock().await;
        if subscribers_map.remove(&id).is_none() {
            tracing::warn!("[CpuUsageService] Attempted to unsubscribe with unknown SubscriptionId: {}", id);
            // Depending on strictness, could return an error:
            // return Err(DomainError::Internal("SubscriptionId not found".to_string()));
        }

        if subscribers_map.is_empty() {
            // Last local subscriber unsubscribed, so unsubscribe from MCP server
            let mut active_sub_guard = self.active_mcp_sub.lock().await;
            if let Some(active_sub) = active_sub_guard.take() { // take() removes it and gives ownership
                tracing::info!("[CpuUsageService][{}] All local subscribers gone. Unsubscribing from MCP resource URI {}.", self.cpu_server_id, CPU_RESOURCE_URI_DEFAULT);
                // Send stop signal to the processing task
                if active_sub.stop_processing_tx.send(()).is_err() {
                    tracing::error!("[CpuUsageService][{}] Failed to send stop signal to notification processing task for URI {}.", self.cpu_server_id, CPU_RESOURCE_URI_DEFAULT);
                }
                // Wait for the processing task to finish
                if let Err(e) = tokio::time::timeout(tokio::time::Duration::from_secs(5), active_sub.processing_task_handle).await {
                    tracing::error!("[CpuUsageService][{}] Timeout or error waiting for notification processing task to finish for URI {}: {:?}", self.cpu_server_id, CPU_RESOURCE_URI_DEFAULT, e);
                }

                // Get client instance to send unsubscribe request
                let conn_service_guard = self.connection_service.lock().await;
                if let Some(client_instance_arc) = conn_service_guard.get_client_instance(&self.cpu_server_id) {
                    let mut client_instance = client_instance_arc.lock().await;
                    let unsubscribe_params = serde_json::json!({
                        "uri": CPU_RESOURCE_URI_DEFAULT,
                    });
                    if let Err(e) = client_instance.send_request_internal("resources/unsubscribe".to_string(), unsubscribe_params).await {
                        // Log error, but proceed with cleanup. The service is no longer "actively" subscribed.
                        tracing::error!("[CpuUsageService][{}] Error sending resources/unsubscribe to MCP server for URI {}: {:?}", self.cpu_server_id, CPU_RESOURCE_URI_DEFAULT, e);
                        // Potentially return an error or handle it based on desired service guarantees
                        // For now, we consider the local state (no active_mcp_sub) as "unsubscribed" from the service's perspective.
                    } else {
                        tracing::info!("[CpuUsageService][{}] Successfully sent unsubscribe request to MCP server for URI {}.", self.cpu_server_id, CPU_RESOURCE_URI_DEFAULT);
                    }
                } else {
                    tracing::error!("[CpuUsageService][{}] CPU MCP server instance '{}' not found during unsubscribe.", self.cpu_server_id, self.cpu_server_id);
                }
            }
        }
        Ok(())
    }
}

// Implementation of process_cpu_notifications_task will be in subsequent steps

// Placeholder for process_cpu_notifications_task to be implemented later
async fn process_cpu_notifications_task(
    mut notification_rx: mpsc::UnboundedReceiver<JsonRpcRequest>,
    subscribers_ref: Arc<Mutex<HashMap<SubscriptionId, mpsc::UnboundedSender<Result<f64, DomainError>>>>>,
    mut stop_rx: oneshot::Receiver<()>,
    server_id_for_log: ServerId, // For logging context
    resource_uri_for_log: String, // For logging context
) {
    loop {
        tokio::select! {
            biased; // Prioritize stop signal
            _ = &mut stop_rx => {
                tracing::info!("[CpuUsageService][{}] Notification processor: Stop signal received for URI {}.", server_id_for_log, resource_uri_for_log);
                break;
            }
            maybe_notification = notification_rx.recv() => {
                match maybe_notification {
                    Some(notification) => {
                        tracing::debug!("[CpuUsageService][{}] Received notification: {:?}", server_id_for_log, notification);
                        if notification.method == "notifications/resources/updated" {
                            // Assuming notification.params is a Value, not Option<Value>
                            // If it's Option<Value>, unwrap or handle None case.
                            // For this example, assuming params is always present for this notification type.
                            match notification.params.as_object() {
                                Some(params_obj) => {
                                    if params_obj.get("uri").and_then(|v| v.as_str()) == Some(&resource_uri_for_log) {
                                        if let Some(content_val) = params_obj.get("content") {
                                            match serde_json::from_value::<CpuUsageContentFromServer>(content_val.clone()) {
                                                Ok(cpu_content) => {
                                                    let subscribers = subscribers_ref.lock().await;
                                                    if subscribers.is_empty() {
                                                         tracing::debug!("[CpuUsageService][{}] No active subscribers for CPU updates from URI {}", server_id_for_log, resource_uri_for_log);
                                                    }
                                                    for (id, sender) in subscribers.iter() {
                                                        if sender.send(Ok(cpu_content.percentage)).is_err() {
                                                            tracing::warn!("[CpuUsageService][{}] Failed to send CPU update to subscriber {}, channel closed. URI: {}", server_id_for_log, id, resource_uri_for_log);
                                                            // Consider removing the subscriber here if the channel is closed.
                                                        } else {
                                                            tracing::trace!("[CpuUsageService][{}] Sent CPU update to subscriber {}. URI: {}", server_id_for_log, id, resource_uri_for_log);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::error!("[CpuUsageService][{}] Failed to deserialize CpuUsageContentFromServer from notification for URI {}: {:?}. Content: {:?}", server_id_for_log, resource_uri_for_log, e, content_val);
                                                     let subscribers = subscribers_ref.lock().await;
                                                     for sender in subscribers.values() {
                                                         let _ = sender.send(Err(DomainError::NotificationProcessingError(format!("Deserialization error: {}", e))));
                                                     }
                                                }
                                            }
                                        } else {
                                             tracing::warn!("[CpuUsageService][{}] Notification for URI {} missing 'content' field. Params: {:?}", server_id_for_log, resource_uri_for_log, params_obj);
                                        }
                                    } // else, notification for a different URI, ignore.
                                }
                                None => {
                                     tracing::warn!("[CpuUsageService][{}] Notification params is not an object: {:?}", server_id_for_log, notification.params);
                                }
                            }
                        } // else, notification for a different method, ignore.
                    }
                    None => { // Channel closed by MCPClientInstance (e.g. transport died)
                        tracing::info!("[CpuUsageService][{}] Notification processor: Notification channel closed for URI {}.", server_id_for_log, resource_uri_for_log);
                         let subscribers = subscribers_ref.lock().await;
                         for sender in subscribers.values() {
                             let _ = sender.send(Err(DomainError::NotSubscribed)); // Or a more specific error
                         }
                        break;
                    }
                }
            }
        }
    }
    tracing::info!("[CpuUsageService][{}] Notification processor task finished for URI {}.", server_id_for_log, resource_uri_for_log);
}

// Ensure this module is registered in novade-domain/src/lib.rs
// pub mod cpu_usage_service;

#[cfg(test)]
mod tests;
```

**Manual step required by user (or in a later agent turn):**
Add `pub mod cpu_usage_service;` to `novade-domain/src/lib.rs`.

I have created the `novade-domain/src/cpu_usage_service/mod.rs` file with the definitions for `SubscriptionId`, `ICpuUsageService`, `DefaultCpuUsageService`, `ActiveMcpSubscription`, and a skeleton for `process_cpu_notifications_task`. I've also included the `CpuUsageContentFromServer` temporary struct.

Next, I will implement **Step 4: Implement `DefaultCpuUsageService::new()`**.Okay, I have reviewed the existing `novade-domain/src/error.rs` and `novade-domain/Cargo.toml`.
The existing `DomainError` enum is quite comprehensive. I will use `DomainError::Other(String)` for simplicity if a very specific error is not available, or map to `DomainError::Internal(String)` when converting from `anyhow::Error`. The specific errors like `MCPConnection`, `MCPRequest`, etc., can be added as variants to the main `DomainError` if desired, or nested. For now, I'll rely on `Other` or `Internal` if a direct match isn't obvious in the existing structure.

The `Cargo.toml` already has `thiserror`, `uuid`, and `async-trait`.

I have also created the `novade-domain/src/cpu_usage_service/mod.rs` file with the initial structure in the previous turn.

Now, I will proceed with **Step 4: Implement `DefaultCpuUsageService::new()`**.
This involves defining the `new` function for `DefaultCpuUsageService` to initialize its fields.
