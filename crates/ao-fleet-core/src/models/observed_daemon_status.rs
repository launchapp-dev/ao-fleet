use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::daemon_desired_state::DaemonDesiredState;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservedDaemonStatus {
    pub project_id: String,
    pub team_id: String,
    pub observed_state: DaemonDesiredState,
    pub source: String,
    pub checked_at: DateTime<Utc>,
    pub details: Value,
}
