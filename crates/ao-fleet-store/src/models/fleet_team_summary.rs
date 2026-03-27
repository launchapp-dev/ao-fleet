use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetTeamSummary {
    pub project_count: usize,
    pub enabled_project_count: usize,
    pub schedule_count: usize,
    pub enabled_schedule_count: usize,
    pub backlog_count: usize,
}
