use ao_fleet_core::{Project, Schedule, Team};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetConfigSnapshot {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub teams: Vec<Team>,
    pub projects: Vec<Project>,
    pub schedules: Vec<Schedule>,
}
