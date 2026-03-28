use ao_fleet_core::{DaemonDesiredState, DaemonOverrideMode};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonOverrideUpsertInput {
    pub team_id: String,
    pub mode: DaemonOverrideMode,
    pub forced_state: Option<DaemonDesiredState>,
    pub pause_until: Option<DateTime<Utc>>,
    pub note: Option<String>,
    pub source: String,
}
