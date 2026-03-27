use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectHostPlacement {
    pub project_id: String,
    pub host_id: String,
    pub assignment_source: String,
    pub assigned_at: DateTime<Utc>,
}
