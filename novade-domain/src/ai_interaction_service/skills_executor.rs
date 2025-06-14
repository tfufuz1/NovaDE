// novade-domain/src/ai_interaction_service/skills_executor.rs
use novade_core::types::assistant::{AssistantCommand, SkillDefinition, ContextInfo};
use crate::error::DomainError;
// Corrected path to logic_service.rs where ExecutionResult is now defined (as per previous steps)
use super::logic_service::ExecutionResult;
use std::collections::HashMap;

pub trait SkillsExecutor: Send + Sync {
    fn register_skill(&mut self, definition: SkillDefinition) -> Result<(), DomainError>;
    fn unregister_skill(&mut self, skill_id: &str) -> Result<(), DomainError>;
    fn execute_skill_command(&self, command: &AssistantCommand, context: &ContextInfo) -> Result<ExecutionResult, DomainError>;
}

pub struct DefaultSkillsExecutor {
    skills: HashMap<String, SkillDefinition>,
    // May need handles to various domain/system services to pass to skills in a real implementation
    // For example: app_launcher: Arc<dyn ApplicationManagerSystemTrait>
}

impl DefaultSkillsExecutor {
    pub fn new() -> Self {
        DefaultSkillsExecutor {
            skills: HashMap::new(),
        }
    }
}

impl Default for DefaultSkillsExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillsExecutor for DefaultSkillsExecutor {
    fn register_skill(&mut self, definition: SkillDefinition) -> Result<(), DomainError> {
        if definition.id.is_empty() {
            return Err(DomainError::SkillError("Skill ID cannot be empty".to_string()));
        }
        if definition.name.is_empty() {
            return Err(DomainError::SkillError(format!("Skill '{}' name cannot be empty", definition.id)));
        }
        if definition.supported_intents.is_empty() {
            return Err(DomainError::SkillError(format!("Skill '{}' must support at least one intent", definition.id)));
        }
        println!("SkillsExecutor: Registering skill: id='{}', name='{}', intents='{:?}'", definition.id, definition.name, definition.supported_intents);
        self.skills.insert(definition.id.clone(), definition);
        Ok(())
    }

    fn unregister_skill(&mut self, skill_id: &str) -> Result<(), DomainError> {
        println!("SkillsExecutor: Unregistering skill: {}", skill_id);
        if self.skills.remove(skill_id).is_some() {
            Ok(())
        } else {
            Err(DomainError::SkillError(format!("Skill not found for unregistration: {}", skill_id)))
        }
    }

    fn execute_skill_command(&self, command: &AssistantCommand, _context: &ContextInfo) -> Result<ExecutionResult, DomainError> {
        println!("SkillsExecutor: Attempting to execute skill command: intent='{}', params='{:?}'", command.intent, command.parameters);

        // Find a skill that supports the intent
        let handling_skill = self.skills.values().find(|skill_def| {
            skill_def.supported_intents.contains(&command.intent)
        });

        if let Some(skill) = handling_skill {
            println!("SkillsExecutor: Found skill '{}' to handle intent '{}'", skill.name, command.intent);
            // Placeholder: Actual skill invocation would happen here.
            // This might involve dynamically calling a method on a skill object,
            // or sending a message to a skill process/thread.
            // For now, just acknowledge that a skill *could* handle it.

            // Example: Specific handling for "open_app" intent if no real skill execution framework exists yet
            if command.intent == "open_app" {
                if let Some(params) = &command.parameters {
                    if let Some(app_name) = params.get("app_name") {
                        // Here, you would ideally call a system service to launch the app.
                        // e.g., self.app_launcher.launch_application(app_name)?;
                        println!("SkillsExecutor: Placeholder - would launch application '{}'", app_name);
                        return Ok(ExecutionResult {
                            success: true,
                            message_to_user: Some(format!("Application '{}' would be opened.", app_name)),
                        });
                    }
                }
                return Err(DomainError::SkillError(format!("Missing 'app_name' parameter for 'open_app' intent for skill '{}'", skill.name)));
            }

            // Generic success for other intents handled by registered skills
            Ok(ExecutionResult {
                success: true,
                message_to_user: Some(format!("Skill '{}' would execute for intent '{}'.", skill.name, command.intent)),
            })
        } else {
            println!("SkillsExecutor: No skill registered for intent: {}", command.intent);
            Err(DomainError::SkillError(format!("No skill registered for intent: {}", command.intent)))
        }
    }
}
