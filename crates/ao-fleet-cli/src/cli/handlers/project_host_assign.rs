use anyhow::Result;
use ao_fleet_core::ProjectHostPlacement;
use ao_fleet_store::FleetStore;
use chrono::Utc;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_host_assign_command::ProjectHostAssignCommand;

pub fn project_host_assign(db_path: &str, command: ProjectHostAssignCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let placement = store.upsert_project_host_placement(ProjectHostPlacement {
        project_id: command.project_id,
        host_id: command.host_id,
        assignment_source: command.assignment_source,
        assigned_at: Utc::now(),
    })?;
    print_json(&placement)
}
