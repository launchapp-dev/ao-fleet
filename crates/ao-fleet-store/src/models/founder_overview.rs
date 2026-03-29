use ao_fleet_core::{AuditEvent, Host, KnowledgeDocument, KnowledgeFact, ProjectHostPlacement};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::fleet_overview::FleetOverview;
use crate::models::founder_overview_summary::FounderOverviewSummary;
use crate::models::founder_team_overview::FounderTeamOverview;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FounderOverview {
    pub evaluated_at: DateTime<Utc>,
    pub summary: FounderOverviewSummary,
    pub fleet: FleetOverview,
    pub hosts: Vec<Host>,
    pub project_host_placements: Vec<ProjectHostPlacement>,
    pub daemon_statuses: Vec<crate::models::fleet_daemon_status::FleetDaemonStatus>,
    pub audit_events: Vec<AuditEvent>,
    pub knowledge_documents: Vec<KnowledgeDocument>,
    pub knowledge_facts: Vec<KnowledgeFact>,
    pub teams: Vec<FounderTeamOverview>,
}
