use anyhow::{Result, anyhow};
use ao_fleet_store::FleetStore;
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::cli::handlers::daemon_status::refresh_observed_statuses;
use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_status_command::ProjectStatusCommand;

#[derive(Debug, Serialize)]
pub struct ProjectStatus {
    pub project_id: String,
    pub project_slug: String,
    pub team_id: String,
    pub root_path: String,
    pub ao_project_root: String,
    pub enabled: bool,
    pub desired_state: String,
    pub observed_state: Option<String>,
    pub checked_at: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub details: Option<Value>,
}

pub fn project_status(db_path: &str, command: ProjectStatusCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;

    let project = store
        .get_project(&command.id)?
        .ok_or_else(|| anyhow!("project not found: {}", command.id))?;

    if command.refresh {
        refresh_observed_statuses(&store, Some(&project.team_id))?;
    }

    let statuses = store.fleet_daemon_statuses(None)?;
    let status = statuses
        .into_iter()
        .find(|s| s.project_id == command.id)
        .ok_or_else(|| anyhow!("project not found in fleet status: {}", command.id))?;

    print_json(&ProjectStatus {
        project_id: status.project_id,
        project_slug: status.project_slug,
        team_id: status.team_id,
        root_path: project.root_path,
        ao_project_root: project.ao_project_root,
        enabled: project.enabled,
        desired_state: format!("{:?}", status.desired_state).to_lowercase(),
        observed_state: status.observed_state.map(|s| format!("{:?}", s).to_lowercase()),
        checked_at: status.checked_at,
        source: status.source,
        details: status.details,
    })
}
