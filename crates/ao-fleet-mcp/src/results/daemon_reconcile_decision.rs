use ao_fleet_core::{DaemonDesiredState, DaemonOverride};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonReconcileDecision {
    pub team_id: String,
    pub desired_state: DaemonDesiredState,
    pub backlog_count: usize,
    pub schedule_ids: Vec<String>,
    pub reason: String,
    pub override_applied: Option<DaemonOverride>,
}
