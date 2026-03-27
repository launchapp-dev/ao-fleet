use ao_fleet_core::{Project, Schedule, Team};
use serde::{Deserialize, Serialize};

use crate::models::fleet_reconcile_preview_item::FleetReconcilePreviewItem;
use crate::models::fleet_team_summary::FleetTeamSummary;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetTeamOverview {
    pub team: Team,
    pub summary: FleetTeamSummary,
    pub projects: Vec<Project>,
    pub schedules: Vec<Schedule>,
    pub reconcile_preview: FleetReconcilePreviewItem,
}
