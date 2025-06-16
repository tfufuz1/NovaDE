// Copyright 2024 NovaDE Contributors
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

//! Defines traits for services that the workspace domain logic depends on.

use async_trait::async_trait;
use crate::workspaces::core::{Window as DomainWindow, WindowId, WindowState as DomainWindowState};
use novade_core::types::geometry::{Point, Size};

// Placeholder for SystemResult if not defined in novade-domain directly.
// Ideally, this would come from a shared error module or be defined here.
// For now, let's assume a basic Result type for the trait definition.
// In `novade-system`, this will map to its `SystemResult`.
pub type DomainResult<T> = Result<T, String>; // Placeholder Error Type

/// Window manager interface required by the WorkspaceManager.
#[async_trait]
pub trait WindowManager: Send + Sync {
    async fn get_windows(&self) -> DomainResult<Vec<DomainWindow>>;
    async fn get_window(&self, id: WindowId) -> DomainResult<DomainWindow>;
    async fn focus_window(&self, id: WindowId) -> DomainResult<()>;
    async fn move_window(&self, id: WindowId, position: Point) -> DomainResult<()>;
    async fn resize_window(&self, id: WindowId, size: Size) -> DomainResult<()>;
    async fn set_window_state(&self, id: WindowId, state: DomainWindowState) -> DomainResult<()>;
    async fn close_window(&self, id: WindowId) -> DomainResult<()>;

    // Methods needed by WorkspaceManager for switching workspaces (show/hide)
    async fn hide_window_for_workspace(&self, id: WindowId) -> DomainResult<()>;
    async fn show_window_for_workspace(&self, id: WindowId) -> DomainResult<()>;

    // Methods for multi-monitor support
    async fn get_primary_output_id(&self) -> DomainResult<Option<String>>;
    async fn get_output_work_area(&self, output_id: &str) -> DomainResult<Rect>; // Rect from novade_core::types::geometry
    async fn get_focused_output_id(&self) -> DomainResult<Option<String>>;
    // ANCHOR: May need a method like `get_outputs_info() -> DomainResult<Vec<OutputInfo>>`
    // if WorkspaceManager needs to list available monitors to assign workspaces.
}
