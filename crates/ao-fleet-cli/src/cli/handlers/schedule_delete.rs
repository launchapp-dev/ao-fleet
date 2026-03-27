use anyhow::Result;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::schedule_delete_command::ScheduleDeleteCommand;

pub fn schedule_delete(db_path: &str, command: ScheduleDeleteCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let deleted = store.delete_schedule(&command.id)?;
    print_json(&serde_json::json!({ "id": command.id, "deleted": deleted }))
}
