use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::fleet_overview_summary::FleetOverviewSummary;
use crate::models::fleet_reconcile_preview::FleetReconcilePreview;
use crate::models::fleet_team_overview::FleetTeamOverview;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetOverview {
    pub evaluated_at: DateTime<Utc>,
    pub summary: FleetOverviewSummary,
    pub teams: Vec<FleetTeamOverview>,
    pub preview: FleetReconcilePreview,
}
