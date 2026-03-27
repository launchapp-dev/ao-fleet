use anyhow::{Result, anyhow};
use ao_fleet_store::FleetStore;
use chrono::Utc;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_update_command::ProjectUpdateCommand;

pub fn project_update(db_path: &str, command: ProjectUpdateCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let mut project = store
        .get_project(&command.id)?
        .ok_or_else(|| anyhow!("project not found: {}", command.id))?;

    if let Some(value) = command.team_id {
        project.team_id = value;
    }
    if let Some(value) = command.slug {
        project.slug = value;
    }
    if let Some(value) = command.root_path {
        project.root_path = value;
    }
    if let Some(value) = command.ao_project_root {
        project.ao_project_root = value;
    }
    if let Some(value) = command.default_branch {
        project.default_branch = value;
    }
    if let Some(value) = command.enabled {
        project.enabled = value;
    }
    project.updated_at = Utc::now();

    let project = store.update_project(project)?;
    print_json(&project)
}
