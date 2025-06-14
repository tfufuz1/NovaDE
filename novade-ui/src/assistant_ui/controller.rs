// novade-ui/src/assistant_ui/controller.rs
use std::sync::Arc;
use tokio::sync::mpsc;
use novade_core::types::assistant::{ContextInfo, AssistantPreferences};
use novade_domain::ai_interaction_service::{
    AIInteractionLogicService, ProcessOutput, ExecutionResult, DefaultAIInteractionLogicService
};

#[derive(Debug, Clone)]
pub enum AssistantUIUpdate {
    DisplayProcessOutput(ProcessOutput),
    DisplayExecutionResult(ExecutionResult),
    DisplayError(String),
    ClearInput,
}

pub struct AssistantUIController {
    domain_service: Arc<dyn AIInteractionLogicService>,
    // This sender is used to send updates from async tasks back to the Iced application loop
    pub result_sender: mpsc::Sender<AssistantUIUpdate>,
}

impl AssistantUIController {
    pub fn new(
        domain_service: Arc<dyn AIInteractionLogicService>,
        result_sender: mpsc::Sender<AssistantUIUpdate>,
    ) -> Self {
        Self { domain_service, result_sender }
    }

    pub fn on_text_input_submit(&self, text: String) {
        let domain_service = self.domain_service.clone(); // Arc clone
        let result_sender = self.result_sender.clone(); // mpsc::Sender clone

        // Use default context and preferences for this prototype
        let context = ContextInfo::default();
        let prefs = AssistantPreferences::default();

        println!("[UI Controller] Spawning task to process text: {}", text);
        tokio::spawn(async move {
            match domain_service.process_text_input(text, &context, &prefs) {
                Ok(output) => {
                    println!("[UI Controller] Processed output: {:?}", output);
                    match output {
                        ProcessOutput::CommandToExecute(cmd) => {
                            // If a command needs execution, execute it and then send the final result
                            // Note: If launch_application was async, execute_command would need .await
                            // For this prototype, launch_application is sync, so execute_command is sync.
                            match domain_service.execute_command(cmd, &context) {
                                Ok(exec_result) => {
                                    println!("[UI Controller] Execution result: {:?}", exec_result);
                                    if result_sender.send(AssistantUIUpdate::DisplayExecutionResult(exec_result)).await.is_err() {
                                        eprintln!("[UI Controller] Failed to send execution result to UI");
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[UI Controller] Error executing command: {}", e);
                                    if result_sender.send(AssistantUIUpdate::DisplayError(e.to_string())).await.is_err() {
                                        eprintln!("[UI Controller] Failed to send execution error to UI");
                                    }
                                }
                            }
                        }
                        _ => { // DirectResponse, NeedsClarification, NoActionableIntent
                            if result_sender.send(AssistantUIUpdate::DisplayProcessOutput(output)).await.is_err() {
                                eprintln!("[UI Controller] Failed to send process output to UI");
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[UI Controller] Error processing text input: {}", e);
                    if result_sender.send(AssistantUIUpdate::DisplayError(e.to_string())).await.is_err() {
                        eprintln!("[UI Controller] Failed to send processing error to UI");
                    }
                }
            };
            // Always attempt to send ClearInput after processing.
            if result_sender.send(AssistantUIUpdate::ClearInput).await.is_err() {
                eprintln!("[UI Controller] Failed to send ClearInput to UI");
            }
        });
    }
}

/// Helper function to create a default `AssistantUIController`.
/// This is useful for initializing the application.
pub fn create_default_assistant_controller(result_sender: mpsc::Sender<AssistantUIUpdate>) -> AssistantUIController {
    // For the prototype, we directly instantiate DefaultAIInteractionLogicService with its own defaults.
    // In a real app, domain_service might come from a service locator or DI.
    let domain_service = Arc::new(DefaultAIInteractionLogicService::new_default());
    AssistantUIController::new(domain_service, result_sender)
}
