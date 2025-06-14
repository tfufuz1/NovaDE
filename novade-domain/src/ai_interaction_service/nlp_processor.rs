// novade-domain/src/ai_interaction_service/nlp_processor.rs
use novade_core::types::assistant::{UserIntent, ContextInfo};
use crate::error::DomainError; // Assuming this path is correct
use std::collections::HashMap;

pub trait NlpProcessor: Send + Sync {
    fn parse_input(&self, input_text: &str, context: &ContextInfo) -> Result<UserIntent, DomainError>;
}

pub struct BasicNlpProcessor;

impl BasicNlpProcessor {
    pub fn new() -> Self { BasicNlpProcessor }
}

impl Default for BasicNlpProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl NlpProcessor for BasicNlpProcessor {
    fn parse_input(&self, input_text: &str, _context: &ContextInfo) -> Result<UserIntent, DomainError> {
        // Extremely basic parsing: if "open app_name", intent is "open_app"
        let lower_input = input_text.to_lowercase(); // Basic case-insensitivity
        if lower_input.starts_with("open ") {
            let parts: Vec<&str> = input_text.splitn(2, ' ').collect();
            if parts.len() == 2 && !parts[1].trim().is_empty() {
                let app_name = parts[1].trim().to_string();
                let mut entities = HashMap::new();
                entities.insert("app_name".to_string(), app_name);
                return Ok(UserIntent {
                    raw_input: input_text.to_string(),
                    primary_intent_name: "open_app".to_string(),
                    entities,
                    alternative_intents: None,
                });
            }
        }
        // Add another basic intent for testing
        if lower_input.starts_with("tell me a joke") {
            return Ok(UserIntent {
                raw_input: input_text.to_string(),
                primary_intent_name: "tell_joke".to_string(),
                entities: HashMap::new(),
                alternative_intents: None,
            });
        }

        Err(DomainError::NlpError(format!("Could not understand: '{}'", input_text)))
    }
}
