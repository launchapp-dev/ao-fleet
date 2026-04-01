use anyhow::Result;
use ao_fleet_core::DaemonDesiredState;
use ao_fleet_store::FleetStore;
use serde::Serialize;

use crate::cli::handlers::daemon_health_rollup_command::DaemonHealthRollupCommand;
use crate::cli::handlers::json_printer::print_json;

#[derive(Debug, Serialize)]
pub struct DaemonHealthRollup {
    pub total: usize,
    pub desired_running: usize,
    pub observed_running: usize,
    pub aligned: usize,
    pub degraded: usize,
    pub unobserved: usize,
}

pub fn daemon_health_rollup(db_path: &str, command: DaemonHealthRollupCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let statuses = store.fleet_daemon_statuses(command.team_id.as_deref())?;

    let total = statuses.len();
    let desired_running = statuses
        .iter()
        .filter(|s| s.desired_state == DaemonDesiredState::Running)
        .count();
    let observed_running = statuses
        .iter()
        .filter(|s| s.observed_state == Some(DaemonDesiredState::Running))
        .count();
    let aligned = statuses
        .iter()
        .filter(|s| s.observed_state.as_ref().map_or(false, |obs| obs == &s.desired_state))
        .count();
    let unobserved = statuses.iter().filter(|s| s.observed_state.is_none()).count();
    let degraded = total - aligned - unobserved;

    print_json(&DaemonHealthRollup {
        total,
        desired_running,
        observed_running,
        aligned,
        degraded,
        unobserved,
    })
}
