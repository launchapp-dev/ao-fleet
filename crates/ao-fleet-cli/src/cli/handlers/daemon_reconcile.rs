use std::collections::BTreeMap;

use anyhow::Result;
use chrono::{DateTime, Utc};

use ao_fleet_ao::DaemonState;
use ao_fleet_core::{DaemonDesiredState, ObservedDaemonStatus};
use ao_fleet_scheduler::schedule_evaluator::ScheduleEvaluator;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::daemon_reconcile_command::DaemonReconcileCommand;
use crate::cli::handlers::daemon_reconcile_support::reconcile_project;
use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::project_daemon_target::{
    build_host_map, build_project_host_placement_map, resolve_project_daemon_target,
};

pub fn daemon_reconcile(db_path: &str, command: DaemonReconcileCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let team_filter = command.team_id.as_deref();
    let schedules = store.list_schedules(team_filter)?;
    let projects = store.list_projects(team_filter)?;
    let placement_map = build_project_host_placement_map(store.list_project_host_placements()?);
    let host_map = build_host_map(store.list_hosts()?);
    let backlog_map = parse_backlog_map(command.backlog)?;
    let at = match command.at {
        Some(value) => DateTime::parse_from_rfc3339(&value)?.with_timezone(&Utc),
        None => Utc::now(),
    };

    let mut by_team = BTreeMap::<String, TeamReconcileState>::new();

    for schedule in schedules {
        let backlog_count = backlog_map.get(&schedule.team_id).copied().unwrap_or(0);
        let desired_state = ScheduleEvaluator::evaluate(&schedule, at, backlog_count);
        let result = by_team.entry(schedule.team_id.clone()).or_insert_with(|| {
            TeamReconcileState { desired_state, backlog_count, schedule_ids: Vec::new() }
        });

        result.desired_state = merge_desired_state(result.desired_state, desired_state);
        result.schedule_ids.push(schedule.id);
    }

    let mut results = Vec::new();
    for project in projects {
        let Some(team_state) = by_team.get(&project.team_id) else {
            continue;
        };
        let target = resolve_project_daemon_target(&project, &placement_map, &host_map);

        let result = reconcile_project(
            target.controller(),
            project.team_id.clone(),
            project.id.clone(),
            project.ao_project_root.clone(),
            target.details(),
            team_state.desired_state,
            team_state.backlog_count,
            team_state.schedule_ids.clone(),
            command.apply,
        )?;

        if let Some(stored_state) = result.observed_state.clone() {
            store.upsert_observed_daemon_status(ObservedDaemonStatus {
                project_id: project.id.clone(),
                team_id: project.team_id.clone(),
                observed_state: daemon_state_to_desired_state(stored_state.clone()),
                source: "daemon_reconcile".to_string(),
                checked_at: Utc::now(),
                details: serde_json::json!({
                    "project_root": project.ao_project_root,
                    "raw_state": String::from(stored_state),
                    "action": result.action,
                    "command_result": result.command_result,
                    "apply": command.apply,
                    "target": result.target,
                }),
            })?;
        }

        results.push(result);
    }

    print_json(&serde_json::json!({
        "evaluated_at": at.to_rfc3339(),
        "team_id": command.team_id,
        "apply": command.apply,
        "results": results
    }))
}

#[derive(Debug, Clone)]
struct TeamReconcileState {
    desired_state: DaemonDesiredState,
    backlog_count: usize,
    schedule_ids: Vec<String>,
}

fn parse_backlog_map(values: Vec<String>) -> Result<BTreeMap<String, usize>> {
    let mut backlog_map = BTreeMap::new();

    for value in values {
        let Some((team_id, count)) = value.split_once('=') else {
            anyhow::bail!("backlog values must be formatted as team_id=count");
        };
        backlog_map.insert(team_id.to_string(), count.parse::<usize>()?);
    }

    Ok(backlog_map)
}

fn merge_desired_state(
    current: DaemonDesiredState,
    candidate: DaemonDesiredState,
) -> DaemonDesiredState {
    match (current, candidate) {
        (DaemonDesiredState::Running, _) | (_, DaemonDesiredState::Running) => {
            DaemonDesiredState::Running
        }
        (DaemonDesiredState::Paused, _) | (_, DaemonDesiredState::Paused) => {
            DaemonDesiredState::Paused
        }
        _ => DaemonDesiredState::Stopped,
    }
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
