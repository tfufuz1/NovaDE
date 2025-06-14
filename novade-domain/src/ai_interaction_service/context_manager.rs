// novade-domain/src/ai_interaction_service/context_manager.rs
use novade_core::types::assistant::ContextInfo;
use crate::error::DomainError;

#[derive(Debug, Clone, Default)] // Added Default
pub struct PartialContextUpdate {
    pub new_interaction: Option<String>, // Example field
    // pub updated_active_app: Option<String>,
}

pub trait ContextManager: Send + Sync {
    fn get_current_context(&self) -> Result<ContextInfo, DomainError>;
    fn update_context(&self, new_info: PartialContextUpdate) -> Result<(), DomainError>;
}

#[derive(Default)] // Added Default for easier construction
pub struct DefaultContextManager {
    // In a real scenario, this might hold:
    // - A handle to a service providing active window info (e.g., from novade-system)
    // - A handle to user preferences service
    // - Internal state for interaction history
    // For now, it's stateless.
}

impl DefaultContextManager {
    pub fn new() -> Self {
        DefaultContextManager {}
    }
}

// Default trait implementation already provided by derive for the struct itself.
// No need for:
// impl Default for DefaultContextManager {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl ContextManager for DefaultContextManager {
    fn get_current_context(&self) -> Result<ContextInfo, DomainError> {
        // Return a default empty context for now.
        // In a real implementation, this would gather data from various sources.
        let mut context = ContextInfo::default();
        context.current_time = Some(chrono::Utc::now().to_rfc3339()); // Add current time as an example
        println!("DefaultContextManager: Providing current context: {:?}", context);
        Ok(context)
    }

    fn update_context(&self, new_info: PartialContextUpdate) -> Result<(), DomainError> {
        // Placeholder: In a real implementation, this might update a stored context,
        // like adding to interaction_history in ContextInfo.
        println!("DefaultContextManager: Context update called with: {:?}", new_info);
        // Example: if let Some(interaction) = new_info.new_interaction {
        //     // self.history.push(interaction); // If DefaultContextManager had internal state
        // }
        Ok(())
    }
}
