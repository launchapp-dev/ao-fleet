use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::host_delete_command::HostDeleteCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn host_delete(db_path: &str, command: HostDeleteCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let deleted = store.delete_host(&command.id)?;
    print_json(&serde_json::json!({ "id": command.id, "deleted": deleted }))
}
