use anyhow::{Result, anyhow};
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_get_command::ProjectGetCommand;

pub fn project_get(db_path: &str, command: ProjectGetCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let project = store
        .get_project(&command.id)?
        .ok_or_else(|| anyhow!("project not found: {}", command.id))?;
    print_json(&project)
}
