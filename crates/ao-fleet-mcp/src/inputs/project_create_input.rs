use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCreateInput {
    pub team_id: String,
    pub slug: String,
    pub root_path: String,
    pub ao_project_root: String,
    pub default_branch: String,
    pub enabled: bool,
}
