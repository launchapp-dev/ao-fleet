use anyhow::Result;

use ao_fleet_core::NewSchedule;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::schedule_create_command::ScheduleCreateCommand;
use crate::cli::handlers::schedule_policy_kind_parser::parse_schedule_policy_kind;
use crate::cli::handlers::window_parser::parse_window;

pub fn schedule_create(db_path: &str, command: ScheduleCreateCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let policy_kind = parse_schedule_policy_kind(&command.policy_kind)?;
    let mut windows = Vec::new();

    for window in command.windows {
        windows.push(parse_window(&window)?);
    }

    let schedule = store.create_schedule(NewSchedule {
        team_id: command.team_id,
        timezone: command.timezone,
        policy_kind,
        windows,
        enabled: command.enabled,
    })?;

    print_json(&schedule)
}
