use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::team_delete_command::TeamDeleteCommand;

pub fn team_delete(db_path: &str, command: TeamDeleteCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let deleted = store.delete_team(&command.id)?;
    print_json(&serde_json::json!({ "id": command.id, "deleted": deleted }))
}
