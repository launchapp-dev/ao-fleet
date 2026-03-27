use ao_fleet_core::DaemonDesiredState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetDaemonStatus {
    pub team_id: String,
    pub team_slug: String,
    pub project_id: String,
    pub project_slug: String,
    pub project_root: String,
    pub desired_state: DaemonDesiredState,
    pub observed_state: Option<DaemonDesiredState>,
    pub checked_at: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub details: Option<Value>,
}
