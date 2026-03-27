use anyhow::{Result, anyhow};
use ao_fleet_store::FleetStore;
use chrono::Utc;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::schedule_policy_kind_parser::parse_schedule_policy_kind;
use crate::cli::handlers::schedule_update_command::ScheduleUpdateCommand;
use crate::cli::handlers::window_parser::parse_window;

pub fn schedule_update(db_path: &str, command: ScheduleUpdateCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let mut schedule = store
        .get_schedule(&command.id)?
        .ok_or_else(|| anyhow!("schedule not found: {}", command.id))?;

    if let Some(value) = command.team_id {
        schedule.team_id = value;
    }
    if let Some(value) = command.timezone {
        schedule.timezone = value;
    }
    if let Some(value) = command.policy_kind {
        schedule.policy_kind = parse_schedule_policy_kind(&value)?;
    }
    if !command.windows.is_empty() {
        let mut windows = Vec::new();
        for window in command.windows {
            windows.push(parse_window(&window)?);
        }
        schedule.windows = windows;
    }
    if let Some(value) = command.enabled {
        schedule.enabled = value;
    }
    schedule.updated_at = Utc::now();

    let schedule = store.update_schedule(schedule)?;
    print_json(&schedule)
}
