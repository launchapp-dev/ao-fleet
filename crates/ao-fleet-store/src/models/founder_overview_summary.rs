use serde::{Deserialize, Serialize};

use crate::models::fleet_overview_summary::FleetOverviewSummary;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FounderOverviewSummary {
    pub fleet: FleetOverviewSummary,
    pub host_count: usize,
    pub placement_count: usize,
    pub daemon_status_count: usize,
    pub audit_event_count: usize,
    pub knowledge_document_count: usize,
    pub knowledge_fact_count: usize,
}
