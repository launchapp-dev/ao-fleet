use ao_fleet_core::DaemonDesiredState;
use serde::{Deserialize, Serialize};

use crate::models::fleet_reconcile_action::FleetReconcileAction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetReconcilePreviewItem {
    pub team_id: String,
    pub team_slug: String,
    pub desired_state: DaemonDesiredState,
    pub observed_state: DaemonDesiredState,
    pub action: FleetReconcileAction,
    pub backlog_count: usize,
    pub schedule_ids: Vec<String>,
}
