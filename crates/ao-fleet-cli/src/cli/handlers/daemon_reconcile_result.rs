use ao_fleet_core::DaemonDesiredState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DaemonReconcileResult {
    pub team_id: String,
    pub desired_state: DaemonDesiredState,
    pub backlog_count: usize,
    pub schedule_ids: Vec<String>,
}
