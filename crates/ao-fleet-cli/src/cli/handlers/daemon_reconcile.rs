use std::collections::BTreeMap;

use anyhow::Result;
use chrono::{DateTime, Utc};

use ao_fleet_ao::{AoDaemonClient, DaemonCommandResult, DaemonStartOptions, DaemonState};
use ao_fleet_core::DaemonDesiredState;
use ao_fleet_scheduler::schedule_evaluator::ScheduleEvaluator;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::daemon_reconcile_command::DaemonReconcileCommand;
use crate::cli::handlers::daemon_reconcile_result::DaemonReconcileResult;
use crate::cli::handlers::json_printer::print_json;

pub fn daemon_reconcile(db_path: &str, command: DaemonReconcileCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let schedules = store.list_schedules(None)?;
    let projects = store.list_projects(None)?;
    let backlog_map = parse_backlog_map(command.backlog)?;
    let ao = AoDaemonClient::new();
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

        let observed_state = ao
            .daemon_status(&project.ao_project_root)
            .ok()
            .or_else(|| status_to_daemon_state(&ao, &project.ao_project_root));
        let action = planned_action(team_state.desired_state, observed_state.clone());
        let command_result = if command.apply {
            execute_action(&ao, &project.ao_project_root, action.as_deref())?
        } else {
            None
        };

        results.push(DaemonReconcileResult {
            team_id: project.team_id,
            project_id: project.id,
            project_root: project.ao_project_root,
            desired_state: team_state.desired_state,
            observed_state,
            backlog_count: team_state.backlog_count,
            schedule_ids: team_state.schedule_ids.clone(),
            action,
            command_result,
        });
    }

    print_json(&serde_json::json!({
        "evaluated_at": at.to_rfc3339(),
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

fn status_to_daemon_state(ao: &AoDaemonClient, project_root: &str) -> Option<DaemonState> {
    ao.project_status(project_root).ok().map(|report| report.daemon_state)
}

fn planned_action(
    desired_state: DaemonDesiredState,
    observed_state: Option<DaemonState>,
) -> Option<String> {
    match (desired_state, observed_state) {
        (DaemonDesiredState::Running, Some(DaemonState::Running)) => None,
        (DaemonDesiredState::Running, Some(DaemonState::Paused)) => Some("resume".to_string()),
        (DaemonDesiredState::Running, _) => Some("start".to_string()),
        (DaemonDesiredState::Paused, Some(DaemonState::Running)) => Some("pause".to_string()),
        (DaemonDesiredState::Paused, _) => None,
        (DaemonDesiredState::Stopped, Some(DaemonState::Running | DaemonState::Paused)) => {
            Some("stop".to_string())
        }
        (DaemonDesiredState::Stopped, _) => None,
    }
}

fn execute_action(
    ao: &AoDaemonClient,
    project_root: &str,
    action: Option<&str>,
) -> Result<Option<DaemonCommandResult>> {
    let result = match action {
        Some("start") => Some(ao.start(project_root, &DaemonStartOptions::default())?),
        Some("resume") => Some(ao.resume(project_root)?),
        Some("pause") => Some(ao.pause(project_root)?),
        Some("stop") => Some(ao.stop(project_root, None)?),
        Some(_) | None => None,
    };

    Ok(result)
}
