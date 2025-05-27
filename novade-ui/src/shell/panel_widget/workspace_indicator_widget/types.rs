#[derive(Clone, Debug)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
    pub icon_name: Option<String>,
    pub number: usize,
    pub is_active: bool,
    pub is_occupied: bool, // For future styling
}
