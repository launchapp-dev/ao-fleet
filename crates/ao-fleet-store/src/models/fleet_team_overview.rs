use ao_fleet_core::{
    AuditEvent, DaemonOverride, Host, KnowledgeDocument, KnowledgeFact, Project,
    ProjectHostPlacement, Schedule, Team,
};
use serde::{Deserialize, Serialize};

use crate::models::fleet_daemon_status::FleetDaemonStatus;
use crate::models::fleet_reconcile_preview_item::FleetReconcilePreviewItem;
use crate::models::fleet_team_summary::FleetTeamSummary;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetTeamOverview {
    pub team: Team,
    pub summary: FleetTeamSummary,
    pub projects: Vec<Project>,
    pub schedules: Vec<Schedule>,
    pub placements: Vec<ProjectHostPlacement>,
    pub hosts: Vec<Host>,
    pub daemon_statuses: Vec<FleetDaemonStatus>,
    pub audit_events: Vec<AuditEvent>,
    pub knowledge_documents: Vec<KnowledgeDocument>,
    pub knowledge_facts: Vec<KnowledgeFact>,
    pub daemon_override: Option<DaemonOverride>,
    pub reconcile_preview: FleetReconcilePreviewItem,
}
