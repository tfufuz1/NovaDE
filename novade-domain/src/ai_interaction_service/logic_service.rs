// novade-domain/src/ai_interaction_service/logic_service.rs
use novade_core::types::assistant::{AssistantCommand, ContextInfo, AssistantPreferences};
use crate::error::DomainError;
use super::nlp_processor::{NlpProcessor, BasicNlpProcessor};
use super::context_manager::{ContextManager, DefaultContextManager, PartialContextUpdate};
use super::skills_executor::{SkillsExecutor, DefaultSkillsExecutor};
use std::sync::Arc;

// Import System Layer components
use novade_system::application_manager::{ApplicationManager, DefaultApplicationManager as SystemDefaultApplicationManager};

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessOutput {
    CommandToExecute(AssistantCommand),
    DirectResponse(String),
    NeedsClarification { // Made struct-like for clarity
        query: String,
        options: Vec<String>
    },
    NoActionableIntent(String),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExecutionResult {
    pub success: bool,
    pub message_to_user: Option<String>,
}

pub trait AIInteractionLogicService: Send + Sync {
    fn process_text_input(
        &self,
        input_text: String,
        current_context: &ContextInfo,
        preferences: &AssistantPreferences,
    ) -> Result<ProcessOutput, DomainError>;

    // Note: execute_command is now async to accommodate potential async calls to system layer
    // However, for this prototype, ApplicationManager::launch_application is sync.
    // If launch_application were async, this would need to be:
    // async fn execute_command(&self, command: AssistantCommand, current_context: &ContextInfo) -> Result<ExecutionResult, DomainError>;
    // For now, keeping it sync as per ApplicationManager's current definition.
    fn execute_command(
        &self,
        command: AssistantCommand,
        current_context: &ContextInfo,
    ) -> Result<ExecutionResult, DomainError>;
}

pub struct DefaultAIInteractionLogicService {
    nlp_processor: Arc<dyn NlpProcessor>,
    context_manager: Arc<dyn ContextManager>,
    skills_executor: Arc<dyn SkillsExecutor>,
    app_manager: Arc<dyn ApplicationManager>, // System layer service
}

impl DefaultAIInteractionLogicService {
    pub fn new(
        nlp_processor: Arc<dyn NlpProcessor>,
        context_manager: Arc<dyn ContextManager>,
        skills_executor: Arc<dyn SkillsExecutor>,
        app_manager: Arc<dyn ApplicationManager>,
    ) -> Self {
        DefaultAIInteractionLogicService {
            nlp_processor,
            context_manager,
            skills_executor,
            app_manager,
        }
    }

    pub fn new_default() -> Self {
        Self::new(
            Arc::new(BasicNlpProcessor::new()),
            Arc::new(DefaultContextManager::new()),
            Arc::new(DefaultSkillsExecutor::new()),
            Arc::new(SystemDefaultApplicationManager::new()), // Use system's default
        )
    }
}

impl AIInteractionLogicService for DefaultAIInteractionLogicService {
    fn process_text_input(
        &self,
        input_text: String,
        current_context: &ContextInfo,
        _preferences: &AssistantPreferences,
    ) -> Result<ProcessOutput, DomainError> {
        println!("[Domain Layer] Processing text input: '{}'", input_text);
        
        // Update context (e.g., add to history)
        let _ = self.context_manager.update_context(PartialContextUpdate {
            new_interaction: Some(input_text.clone()),
            ..Default::default()
        });

        let user_intent = match self.nlp_processor.parse_input(&input_text, current_context) {
            Ok(intent) => intent,
            Err(DomainError::NlpError(msg)) => {
                return Ok(ProcessOutput::NoActionableIntent(msg));
            }
            Err(e) => return Err(e),
        };

        match user_intent.primary_intent_name.as_str() {
            "open_app" => Ok(ProcessOutput::CommandToExecute(AssistantCommand {
                intent: user_intent.primary_intent_name,
                parameters: Some(user_intent.entities),
                confidence: None,
            })),
            "tell_joke" => Ok(ProcessOutput::DirectResponse(
                "Why don't scientists trust atoms? Because they make up everything!".to_string(),
            )),
            _ => Ok(ProcessOutput::DirectResponse(format!(
                "Intent: {}. Entities: {:?}",
                user_intent.primary_intent_name, user_intent.entities
            ))),
        }
    }

    fn execute_command(
        &self,
        command: AssistantCommand,
        _current_context: &ContextInfo, // Context might be used by other commands or skills
    ) -> Result<ExecutionResult, DomainError> {
        println!("[Domain Layer] Executing command: intent='{}'", command.intent);
        if command.intent == "open_app" {
            if let Some(params) = &command.parameters {
                if let Some(app_name) = params.get("app_name") {
                    // Directly call the system layer's ApplicationManager
                    self.app_manager
                        .launch_application(app_name)
                        .map_err(|e| {
                            // Convert SystemError to DomainError
                            DomainError::SkillError(format!( // Using SkillError as a general "action failed" error
                                "Failed to launch app '{}': {}",
                                app_name, e
                            ))
                        })?;
                    return Ok(ExecutionResult {
                        success: true,
                        message_to_user: Some(format!(
                            "Application '{}' launched (attempted).",
                            app_name
                        )),
                    });
                }
            }
            Err(DomainError::SkillError(
                "Missing 'app_name' parameter for open_app intent".to_string(),
            ))
        } else {
            // For other intents, delegate to the skills_executor
            // This part remains unchanged from the previous step's placeholder
            println!("[Domain Layer] Delegating command '{}' to skills executor.", command.intent);
            self.skills_executor
                .execute_skill_command(&command, _current_context)
        }
    }
}
