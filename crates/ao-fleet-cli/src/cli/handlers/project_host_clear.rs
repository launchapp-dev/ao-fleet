use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_host_clear_command::ProjectHostClearCommand;

pub fn project_host_clear(db_path: &str, command: ProjectHostClearCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let cleared = store.clear_project_host_placement(&command.project_id)?;
    print_json(&serde_json::json!({ "project_id": command.project_id, "cleared": cleared }))
}
