//! Workspace configuration module for the NovaDE domain layer.
//!
//! This module provides functionality for loading and managing
//! workspace configuration in the NovaDE desktop environment.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{DomainResult, WorkspaceError};
use crate::workspaces::core::{WorkspaceId, WorkspaceType};

/// Workspace configuration for the NovaDE desktop environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// The default workspaces to create.
    pub default_workspaces: Vec<DefaultWorkspaceConfig>,
    /// The path to the workspace configuration file.
    #[serde(skip)]
    pub config_path: Option<PathBuf>,
}

impl WorkspaceConfig {
    /// Creates a new workspace configuration.
    ///
    /// # Returns
    ///
    /// A new empty `WorkspaceConfig`.
    pub fn new() -> Self {
        WorkspaceConfig {
            default_workspaces: Vec::new(),
            config_path: None,
        }
    }

    /// Adds a default workspace to the configuration.
    ///
    /// # Arguments
    ///
    /// * `workspace` - The default workspace configuration to add
    ///
    /// # Returns
    ///
    /// The modified `WorkspaceConfig`.
    pub fn add_default_workspace(&mut self, workspace: DefaultWorkspaceConfig) {
        self.default_workspaces.push(workspace);
    }

    /// Loads the workspace configuration from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file
    ///
    /// # Returns
    ///
    /// The loaded `WorkspaceConfig`, or an error if loading failed.
    pub fn load_from_file(path: PathBuf) -> DomainResult<Self> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| WorkspaceError::ConfigurationLoadFailed(e.to_string()))?;
        
        let mut config: WorkspaceConfig = serde_json::from_str(&content)
            .map_err(|e| WorkspaceError::ConfigurationLoadFailed(e.to_string()))?;
        
        config.config_path = Some(path);
        
        Ok(config)
    }

    /// Saves the workspace configuration to a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to save the configuration to, or None to use the path it was loaded from
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration was saved, or an error if saving failed.
    pub fn save_to_file(&self, path: Option<PathBuf>) -> DomainResult<()> {
        let path = path.or_else(|| self.config_path.clone())
            .ok_or_else(|| WorkspaceError::ConfigurationSaveFailed("No path specified".to_string()))?;
        
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| WorkspaceError::ConfigurationSaveFailed(e.to_string()))?;
        
        std::fs::write(&path, content)
            .map_err(|e| WorkspaceError::ConfigurationSaveFailed(e.to_string()))?;
        
        Ok(())
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        let mut config = Self::new();
        
        // Add default workspaces
        config.add_default_workspace(DefaultWorkspaceConfig {
            id: None,
            name: "Main".to_string(),
            workspace_type: WorkspaceType::Standard,
            auto_activate: true,
        });
        
        config.add_default_workspace(DefaultWorkspaceConfig {
            id: None,
            name: "Work".to_string(),
            workspace_type: WorkspaceType::Standard,
            auto_activate: false,
        });
        
        config.add_default_workspace(DefaultWorkspaceConfig {
            id: None,
            name: "Communication".to_string(),
            workspace_type: WorkspaceType::Specialized,
            auto_activate: false,
        });
        
        config
    }
}

/// Configuration for a default workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultWorkspaceConfig {
    /// The ID of the workspace, if predefined.
    pub id: Option<WorkspaceId>,
    /// The name of the workspace.
    pub name: String,
    /// The type of the workspace.
    pub workspace_type: WorkspaceType,
    /// Whether to automatically activate this workspace.
    pub auto_activate: bool,
}

/// Interface for providing workspace configuration.
pub trait WorkspaceConfigProvider: Send + Sync {
    /// Gets the workspace configuration.
    ///
    /// # Returns
    ///
    /// The workspace configuration.
    fn get_config(&self) -> DomainResult<WorkspaceConfig>;
    
    /// Saves the workspace configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration to save
    ///
    /// # Returns
    ///
    /// `Ok(())` if the configuration was saved, or an error if saving failed.
    fn save_config(&self, config: &WorkspaceConfig) -> DomainResult<()>;
}

/// File-based workspace configuration provider.
pub struct FileWorkspaceConfigProvider {
    /// The path to the configuration file.
    config_path: PathBuf,
}

impl FileWorkspaceConfigProvider {
    /// Creates a new file-based workspace configuration provider.
    ///
    /// # Arguments
    ///
    /// * `config_path` - The path to the configuration file
    ///
    /// # Returns
    ///
    /// A new `FileWorkspaceConfigProvider`.
    pub fn new(config_path: PathBuf) -> Self {
        FileWorkspaceConfigProvider { config_path }
    }
}

impl WorkspaceConfigProvider for FileWorkspaceConfigProvider {
    fn get_config(&self) -> DomainResult<WorkspaceConfig> {
        if self.config_path.exists() {
            WorkspaceConfig::load_from_file(self.config_path.clone())
        } else {
            let mut config = WorkspaceConfig::default();
            config.config_path = Some(self.config_path.clone());
            Ok(config)
        }
    }
    
    fn save_config(&self, config: &WorkspaceConfig) -> DomainResult<()> {
        config.save_to_file(Some(self.config_path.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_workspace_config_new() {
        let config = WorkspaceConfig::new();
        
        assert!(config.default_workspaces.is_empty());
        assert!(config.config_path.is_none());
    }
    
    #[test]
    fn test_workspace_config_add_default_workspace() {
        let mut config = WorkspaceConfig::new();
        
        config.add_default_workspace(DefaultWorkspaceConfig {
            id: None,
            name: "Test".to_string(),
            workspace_type: WorkspaceType::Standard,
            auto_activate: true,
        });
        
        assert_eq!(config.default_workspaces.len(), 1);
        assert_eq!(config.default_workspaces[0].name, "Test");
    }
    
    #[test]
    fn test_workspace_config_default() {
        let config = WorkspaceConfig::default();
        
        assert_eq!(config.default_workspaces.len(), 3);
        assert_eq!(config.default_workspaces[0].name, "Main");
        assert_eq!(config.default_workspaces[1].name, "Work");
        assert_eq!(config.default_workspaces[2].name, "Communication");
    }
    
    #[test]
    fn test_workspace_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workspaces.json");
        
        let mut config = WorkspaceConfig::default();
        config.config_path = Some(config_path.clone());
        
        // Save the configuration
        config.save_to_file(None).unwrap();
        
        // Load the configuration
        let loaded_config = WorkspaceConfig::load_from_file(config_path).unwrap();
        
        assert_eq!(loaded_config.default_workspaces.len(), config.default_workspaces.len());
        assert_eq!(loaded_config.default_workspaces[0].name, config.default_workspaces[0].name);
    }
    
    #[test]
    fn test_file_workspace_config_provider() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workspaces.json");
        
        let provider = FileWorkspaceConfigProvider::new(config_path.clone());
        
        // Get the default configuration
        let config = provider.get_config().unwrap();
        assert_eq!(config.default_workspaces.len(), 3);
        
        // Modify and save the configuration
        let mut modified_config = config.clone();
        modified_config.default_workspaces.clear();
        modified_config.add_default_workspace(DefaultWorkspaceConfig {
            id: None,
            name: "Modified".to_string(),
            workspace_type: WorkspaceType::Standard,
            auto_activate: true,
        });
        
        provider.save_config(&modified_config).unwrap();
        
        // Load the modified configuration
        let loaded_config = provider.get_config().unwrap();
        assert_eq!(loaded_config.default_workspaces.len(), 1);
        assert_eq!(loaded_config.default_workspaces[0].name, "Modified");
    }
}
