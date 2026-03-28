use anyhow::{Result, anyhow};
use ao_fleet_core::{DaemonDesiredState, DaemonOverrideMode, NewDaemonOverride};
use ao_fleet_store::FleetStore;
use chrono::{DateTime, Utc};

use crate::cli::handlers::daemon_override_upsert_command::DaemonOverrideUpsertCommand;
use crate::cli::handlers::json_printer::print_json;

pub fn daemon_override_upsert(db_path: &str, command: DaemonOverrideUpsertCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let override_record = store.upsert_daemon_override(NewDaemonOverride {
        team_id: command.team_id,
        mode: parse_mode(&command.mode)?,
        forced_state: command.forced_state.as_deref().map(parse_forced_state).transpose()?,
        pause_until: command.pause_until.as_deref().map(parse_pause_until).transpose()?,
        note: command.note,
        source: command.source,
    })?;
    print_json(&override_record)
}

fn parse_mode(value: &str) -> Result<DaemonOverrideMode> {
    match value {
        "force_desired_state" => Ok(DaemonOverrideMode::ForceDesiredState),
        "freeze_until" => Ok(DaemonOverrideMode::FreezeUntil),
        other => Err(anyhow!("unsupported override mode '{other}'")),
    }
}

fn parse_forced_state(value: &str) -> Result<DaemonDesiredState> {
    match value {
        "running" => Ok(DaemonDesiredState::Running),
        "paused" => Ok(DaemonDesiredState::Paused),
        "stopped" => Ok(DaemonDesiredState::Stopped),
        other => Err(anyhow!("unsupported forced state '{other}'")),
    }
}

fn parse_pause_until(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc))
}
