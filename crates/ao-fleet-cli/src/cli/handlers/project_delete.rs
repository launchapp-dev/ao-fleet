use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_delete_command::ProjectDeleteCommand;

pub fn project_delete(db_path: &str, command: ProjectDeleteCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let deleted = store.delete_project(&command.id)?;
    print_json(&serde_json::json!({ "id": command.id, "deleted": deleted }))
}
