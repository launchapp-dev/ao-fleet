use anyhow::Result;
use ao_fleet_ao::DaemonState;
use ao_fleet_core::{DaemonDesiredState, ObservedDaemonStatus};
use ao_fleet_store::FleetStore;
use chrono::Utc;

use crate::cli::handlers::daemon_reconcile_support::DaemonController;
use crate::cli::handlers::daemon_status_command::DaemonStatusCommand;
use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_daemon_target::{
    build_host_map, build_project_host_placement_map, resolve_project_daemon_target,
};

pub fn daemon_status(db_path: &str, command: DaemonStatusCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;

    if command.refresh {
        refresh_observed_statuses(&store, command.team_id.as_deref())?;
    }

    print_json(&store.fleet_daemon_statuses(command.team_id.as_deref())?)
}

fn refresh_observed_statuses(store: &FleetStore, team_id: Option<&str>) -> Result<()> {
    let placement_map = build_project_host_placement_map(store.list_project_host_placements()?);
    let host_map = build_host_map(store.list_hosts()?);

    for project in store.list_projects(team_id)? {
        let target = resolve_project_daemon_target(&project, &placement_map, &host_map);
        let observed_state = target
            .controller()
            .daemon_status(&project.ao_project_root)
            .ok()
            .or_else(|| target.controller().project_status(&project.ao_project_root).ok());

        if let Some(observed_state) = observed_state {
            store.upsert_observed_daemon_status(ObservedDaemonStatus {
                project_id: project.id.clone(),
                team_id: project.team_id.clone(),
                observed_state: daemon_state_to_desired_state(observed_state.clone()),
                source: target.source_name().to_string(),
                checked_at: Utc::now(),
                details: serde_json::json!({
                    "project_root": project.ao_project_root,
                    "raw_state": String::from(observed_state),
                    "target": target.details(),
                }),
            })?;
        }
    }

    Ok(())
}

fn daemon_state_to_desired_state(state: DaemonState) -> DaemonDesiredState {
    match state {
        DaemonState::Running => DaemonDesiredState::Running,
        DaemonState::Paused => DaemonDesiredState::Paused,
        DaemonState::Stopped | DaemonState::Crashed | DaemonState::Unknown(_) => {
            DaemonDesiredState::Stopped
        }
    }
}
