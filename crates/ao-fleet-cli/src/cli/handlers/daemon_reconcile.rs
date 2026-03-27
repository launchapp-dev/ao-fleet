use std::collections::BTreeMap;

use anyhow::Result;
use chrono::{DateTime, Utc};

use ao_fleet_core::DaemonDesiredState;
use ao_fleet_scheduler::schedule_evaluator::ScheduleEvaluator;
use ao_fleet_store::FleetStore;

use crate::cli::handlers::daemon_reconcile_command::DaemonReconcileCommand;
use crate::cli::handlers::daemon_reconcile_result::DaemonReconcileResult;
use crate::cli::handlers::json_printer::print_json;

pub fn daemon_reconcile(db_path: &str, command: DaemonReconcileCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let schedules = store.list_schedules(None)?;
    let backlog_map = parse_backlog_map(command.backlog)?;
    let at = match command.at {
        Some(value) => DateTime::parse_from_rfc3339(&value)?.with_timezone(&Utc),
        None => Utc::now(),
    };

    let mut by_team = BTreeMap::<String, DaemonReconcileResult>::new();

    for schedule in schedules {
        let backlog_count = backlog_map.get(&schedule.team_id).copied().unwrap_or(0);
        let desired_state = ScheduleEvaluator::evaluate(&schedule, at, backlog_count);
        let result =
            by_team.entry(schedule.team_id.clone()).or_insert_with(|| DaemonReconcileResult {
                team_id: schedule.team_id.clone(),
                desired_state,
                backlog_count,
                schedule_ids: Vec::new(),
            });

        result.desired_state = merge_desired_state(result.desired_state, desired_state);
        result.schedule_ids.push(schedule.id);
    }

    let results = by_team.into_values().collect::<Vec<_>>();
    print_json(&serde_json::json!({
        "evaluated_at": at.to_rfc3339(),
        "results": results
    }))
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
