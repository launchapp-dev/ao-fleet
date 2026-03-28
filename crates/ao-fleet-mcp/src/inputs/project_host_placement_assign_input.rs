use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectHostPlacementAssignInput {
    pub project_id: String,
    pub host_id: String,
    pub assignment_source: String,
}
