// Copyright 2024 Novade Co. Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A basic file-system based implementation of `ConfigServiceAsync` for theming defaults.

use crate::ConfigServiceAsync;
use novade_core::CoreError;
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

/// A simple config service that reads files directly from the filesystem.
/// It assumes that the paths provided to its methods are either absolute
/// or relative to the current working directory of the application.
/// For the theming engine, this means paths like "src/theming/default_themes/base.tokens.json"
/// are expected to be resolvable from where the application is run.
#[derive(Debug, Clone, Default)]
pub struct DefaultFileSystemConfigService;

impl DefaultFileSystemConfigService {
    pub fn new() -> Self {
        DefaultFileSystemConfigService {}
    }
}

#[async_trait]
impl ConfigServiceAsync for DefaultFileSystemConfigService {
    async fn read_config_file_string(&self, file_path: &str) -> Result<String, CoreError> {
        fs::read_to_string(file_path)
            .await
            .map_err(|e| CoreError::Io(format!("Failed to read file '{}': {}", file_path, e)))
    }

    // --- The following methods are not strictly needed for ThemingEngine's current default loading ---
    // --- but are part of the ConfigServiceAsync trait. We provide minimal/dummy implementations. ---

    async fn write_config_file_string(&self, file_path: &str, content: String) -> Result<(), CoreError> {
        fs::write(file_path, content)
            .await
            .map_err(|e| CoreError::Io(format!("Failed to write file '{}': {}", file_path, e)))
    }
    
    async fn read_file_to_string(&self, path: &Path) -> Result<String, CoreError> {
        fs::read_to_string(path)
            .await
            .map_err(|e| CoreError::Io(format!("Failed to read file '{:?}': {}", path, e)))
    }

    async fn list_files_in_dir(&self, dir_path: &Path, extension: Option<&str>) -> Result<Vec<PathBuf>, CoreError> {
        let mut entries = fs::read_dir(dir_path).await
            .map_err(|e| CoreError::Io(format!("Failed to read directory '{:?}': {}", dir_path, e)))?;
        
        let mut files = Vec::new();
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| CoreError::Io(format!("Failed to read directory entry in '{:?}': {}", dir_path, e)))? {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = extension {
                    if path.extension().map_or(false, |os_str| os_str == ext) {
                        files.push(path);
                    }
                } else {
                    files.push(path);
                }
            }
        }
        Ok(files)
    }

    async fn get_config_dir(&self) -> Result<PathBuf, CoreError> {
        // This basic implementation doesn't manage specific config directories.
        // It would typically return a path like ".config/novade/" in user's home.
        // For now, let's return current dir + ".config" as a placeholder.
        // A real implementation would use crates like `dirs` or `directories`.
        let mut path = std::env::current_dir().map_err(|e| CoreError::Io(e.to_string()))?;
        path.push(".config"); // Placeholder
        Ok(path)
    }

    async fn get_data_dir(&self) -> Result<PathBuf, CoreError> {
        // Similar to get_config_dir, this is a placeholder.
        // Would typically be ".local/share/novade/"
        let mut path = std::env::current_dir().map_err(|e| CoreError::Io(e.to_string()))?;
        path.push(".local/share"); // Placeholder
        Ok(path)
    }
}
