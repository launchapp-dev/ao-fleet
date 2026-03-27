use crate::FleetError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewProject {
    pub team_id: String,
    pub slug: String,
    pub root_path: String,
    pub ao_project_root: String,
    pub default_branch: String,
    pub remote_url: Option<String>,
    pub enabled: bool,
}

impl NewProject {
    pub fn validate(&self) -> Result<(), FleetError> {
        if self.team_id.trim().is_empty() || self.slug.trim().is_empty() {
            return Err(FleetError::Validation {
                message: "project team_id and slug are required".to_string(),
            });
        }

        if self.root_path.trim().is_empty() || self.ao_project_root.trim().is_empty() {
            return Err(FleetError::Validation {
                message: "project root paths are required".to_string(),
            });
        }

        if self.default_branch.trim().is_empty() {
            return Err(FleetError::Validation {
                message: "default branch is required".to_string(),
            });
        }

        Ok(())
    }
}
