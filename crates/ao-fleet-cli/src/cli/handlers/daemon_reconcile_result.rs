use ao_fleet_ao::{DaemonCommandResult, DaemonState};
use ao_fleet_core::DaemonDesiredState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DaemonReconcileResult {
    pub team_id: String,
    pub project_id: String,
    pub project_root: String,
    pub target: serde_json::Value,
    pub desired_state: DaemonDesiredState,
    pub observed_state: Option<DaemonState>,
    pub backlog_count: usize,
    pub schedule_ids: Vec<String>,
    pub action: Option<String>,
    pub command_result: Option<DaemonCommandResult>,
}
