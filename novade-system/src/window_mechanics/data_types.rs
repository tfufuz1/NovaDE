// novade-system/src/window_mechanics/data_types.rs

use novade_core::types::geometry::{Point, Size};
use uuid::Uuid;

/// Unique identifier for a window.
///
/// Wraps a `uuid::Uuid` to provide strong typing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WindowId(Uuid);

impl WindowId {
    /// Creates a new, unique `WindowId`.
    pub fn new_v4() -> Self {
        WindowId(Uuid::new_v4())
    }
}

impl Default for WindowId {
    fn default() -> Self {
        Self::new_v4()
    }
}

impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the geometry of a window using floating-point coordinates and dimensions.
///
/// This uses `Point<f64>` for origin and `Size<f64>` for dimensions from `novade_core`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WindowRect {
    /// The top-left corner of the window.
    pub origin: Point<f64>,
    /// The width and height of the window.
    pub size: Size<f64>,
}

impl WindowRect {
    /// Creates a new `WindowRect`.
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        WindowRect {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }
}

/// Represents the various states a window can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowState {
    /// The window is managed by a tiling algorithm.
    Tiled,
    /// The window is floating freely, not managed by tiling.
    Floating,
    /// The window is minimized (not visible).
    Minimized,
    /// The window is maximized to fill the workspace.
    Maximized,
    /// The window is in fullscreen mode.
    Fullscreen,
}

impl Default for WindowState {
    fn default() -> Self {
        WindowState::Tiled
    }
}

/// Contains information about a specific window.
///
/// This structure holds metadata like the window's ID, title, geometry,
/// and its current state (e.g., tiled, floating).
#[derive(Debug, Clone, PartialEq)]
pub struct WindowInfo {
    /// Unique identifier for this window.
    pub id: WindowId,
    /// The title of the window.
    pub title: String,
    /// The current geometry (position and size) of the window.
    pub rect: WindowRect,
    /// The current state of the window (e.g., Tiled, Floating).
    pub state: WindowState,
    // pub xdg_surface_handle: Option<SmithayXdgSurfaceHandleType>, // To be added later
}

impl WindowInfo {
    /// Creates a new `WindowInfo`.
    pub fn new(id: WindowId, title: String, rect: WindowRect, state: WindowState) -> Self {
        WindowInfo { id, title, rect, state }
    }
}

/// Unique identifier for a workspace.
///
/// Wraps a `uuid::Uuid` for type safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WorkspaceId(Uuid);

impl WorkspaceId {
    /// Creates a new, unique `WorkspaceId`.
    pub fn new_v4() -> Self {
        WorkspaceId(Uuid::new_v4())
    }
}

impl Default for WorkspaceId {
    fn default() -> Self {
        Self::new_v4()
    }
}

impl std::fmt::Display for WorkspaceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents the available tiling layout algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TilingLayout {
    /// A layout that typically has a main "master" area and a stacking area.
    Tall,
    /// A layout that arranges windows in a grid.
    Grid,
    /// A layout that arranges windows in a Fibonacci spiral (placeholder).
    Fibonacci,
    // Floating is handled by WindowState::Floating, not as a layout itself here.
}

impl Default for TilingLayout {
    fn default() -> Self {
        TilingLayout::Tall
    }
}

/// Contains information about a workspace.
///
/// This includes its ID, a human-readable name, the list of windows it contains,
/// and the current tiling layout algorithm being applied.
#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceInfo {
    /// Unique identifier for this workspace.
    pub id: WorkspaceId,
    /// A human-readable name for the workspace (e.g., "Workspace 1", "Web").
    pub name: String,
    /// A list of `WindowId`s representing the windows present in this workspace.
    /// The order might be significant depending on the layout algorithm.
    pub windows: Vec<WindowId>,
    /// The current tiling layout algorithm active for this workspace.
    pub current_layout: TilingLayout,
    // pub screen_id: Option<ScreenId>, // If workspaces are tied to specific screens
}

impl WorkspaceInfo {
    /// Creates a new `WorkspaceInfo`.
    pub fn new(id: WorkspaceId, name: String, current_layout: TilingLayout) -> Self {
        WorkspaceInfo {
            id,
            name,
            windows: Vec::new(),
            current_layout,
        }
    }
}

/// Contains information about a connected display screen.
///
/// This primarily includes its resolution.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ScreenInfo {
    /// The width of the screen in pixels (or logical units if scaled).
    pub width: f64,
    /// The height of the screen in pixels (or logical units if scaled).
    pub height: f64,
    // pub id: ScreenId, // A unique identifier for the screen/output
    // pub workspaces: Vec<WorkspaceId>, // Workspaces currently displayed on this screen
}

impl ScreenInfo {
    /// Creates a new `ScreenInfo`.
    pub fn new(width: f64, height: f64) -> Self {
        ScreenInfo { width, height }
    }
}


// Unit tests for data types
#[cfg(test)]
mod tests {
    use super::*;
    use novade_core::types::geometry::{Point, Size}; // For direct use in tests

    #[test]
    fn window_id_creation_and_default() {
        let id1 = WindowId::new_v4();
        let id2 = WindowId::new_v4();
        let default_id = WindowId::default();
        assert_ne!(id1, id2);
        assert_ne!(id1, default_id);
        assert_ne!(id2, default_id); // Ensure default also generates a new one
        println!("Window ID 1: {}", id1); // For manual inspection if needed
        println!("Window ID 2: {}", id2);
    }

    #[test]
    fn window_rect_creation() {
        let rect = WindowRect::new(10.0, 20.0, 300.0, 200.0);
        assert_eq!(rect.origin.x, 10.0);
        assert_eq!(rect.origin.y, 20.0);
        assert_eq!(rect.size.width, 300.0);
        assert_eq!(rect.size.height, 200.0);

        let default_rect = WindowRect::default();
        assert_eq!(default_rect.origin.x, 0.0);
        assert_eq!(default_rect.origin.y, 0.0);
        assert_eq!(default_rect.size.width, 0.0);
        assert_eq!(default_rect.size.height, 0.0);
    }

    #[test]
    fn window_state_default() {
        assert_eq!(WindowState::default(), WindowState::Tiled);
    }

    #[test]
    fn window_info_creation() {
        let id = WindowId::new_v4();
        let rect = WindowRect::new(0.0, 0.0, 800.0, 600.0);
        let info = WindowInfo::new(id, "Test Window".to_string(), rect, WindowState::Floating);
        assert_eq!(info.id, id);
        assert_eq!(info.title, "Test Window");
        assert_eq!(info.rect, rect);
        assert_eq!(info.state, WindowState::Floating);
    }

    #[test]
    fn workspace_id_creation_and_default() {
        let id1 = WorkspaceId::new_v4();
        let id2 = WorkspaceId::new_v4();
        let default_id = WorkspaceId::default();
        assert_ne!(id1, id2);
        assert_ne!(id1, default_id);
        assert_ne!(id2, default_id);
        println!("Workspace ID 1: {}", id1);
        println!("Workspace ID 2: {}", id2);
    }

    #[test]
    fn tiling_layout_default() {
        assert_eq!(TilingLayout::default(), TilingLayout::Tall);
    }

    #[test]
    fn workspace_info_creation() {
        let id = WorkspaceId::new_v4();
        let info = WorkspaceInfo::new(id, "Primary".to_string(), TilingLayout::Grid);
        assert_eq!(info.id, id);
        assert_eq!(info.name, "Primary");
        assert!(info.windows.is_empty());
        assert_eq!(info.current_layout, TilingLayout::Grid);
    }

    #[test]
    fn workspace_info_add_window() {
        let id = WorkspaceId::new_v4();
        let mut info = WorkspaceInfo::new(id, "Primary".to_string(), TilingLayout::Grid);
        let window_id = WindowId::new_v4();
        info.windows.push(window_id);
        assert_eq!(info.windows.len(), 1);
        assert_eq!(info.windows[0], window_id);
    }

    #[test]
    fn screen_info_creation() {
        let screen = ScreenInfo::new(1920.0, 1080.0);
        assert_eq!(screen.width, 1920.0);
        assert_eq!(screen.height, 1080.0);

        let default_screen = ScreenInfo::default();
        assert_eq!(default_screen.width, 0.0);
        assert_eq!(default_screen.height, 0.0);
    }
}
