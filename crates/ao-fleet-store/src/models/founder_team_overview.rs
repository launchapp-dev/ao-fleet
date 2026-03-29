use ao_fleet_core::{AuditEvent, KnowledgeDocument, KnowledgeFact, ProjectHostPlacement};
use serde::{Deserialize, Serialize};

use crate::models::fleet_daemon_status::FleetDaemonStatus;
use crate::models::fleet_team_overview::FleetTeamOverview;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FounderTeamOverview {
    pub fleet: FleetTeamOverview,
    pub project_host_placements: Vec<ProjectHostPlacement>,
    pub daemon_statuses: Vec<FleetDaemonStatus>,
    pub audit_events: Vec<AuditEvent>,
    pub knowledge_documents: Vec<KnowledgeDocument>,
    pub knowledge_facts: Vec<KnowledgeFact>,
}
