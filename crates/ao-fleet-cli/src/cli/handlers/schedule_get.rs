use anyhow::{Result, anyhow};
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::schedule_get_command::ScheduleGetCommand;

pub fn schedule_get(db_path: &str, command: ScheduleGetCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let schedule = store
        .get_schedule(&command.id)?
        .ok_or_else(|| anyhow!("schedule not found: {}", command.id))?;
    print_json(&schedule)
}
