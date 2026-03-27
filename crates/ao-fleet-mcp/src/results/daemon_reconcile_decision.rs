use ao_fleet_core::DaemonDesiredState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonReconcileDecision {
    pub team_id: String,
    pub desired_state: DaemonDesiredState,
    pub backlog_count: usize,
    pub schedule_ids: Vec<String>,
}
