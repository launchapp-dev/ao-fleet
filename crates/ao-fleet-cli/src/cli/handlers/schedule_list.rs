use anyhow::Result;

use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::schedule_list_command::ScheduleListCommand;

pub fn schedule_list(db_path: &str, _command: ScheduleListCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let schedules = store.list_schedules(None)?;
    print_json(&schedules)
}
