use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use chrono::{DateTime, Utc};

use ao_fleet_ao::DaemonState;
use ao_fleet_core::{DaemonDesiredState, DaemonOverride, ObservedDaemonStatus, Schedule};
use ao_fleet_store::{FleetStore, TeamReconcileEvaluation};

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
    let overrides = store
        .list_daemon_overrides()?
        .into_iter()
        .filter(|override_record| {
            team_filter.map_or(true, |team_id| override_record.team_id == team_id)
        })
        .collect::<Vec<_>>();
    let placement_map = build_project_host_placement_map(store.list_project_host_placements()?);
    let host_map = build_host_map(store.list_hosts()?);
    let backlog_map = parse_backlog_map(command.backlog)?;
    let at = match command.at {
        Some(value) => DateTime::parse_from_rfc3339(&value)?.with_timezone(&Utc),
        None => Utc::now(),
    };

    let schedules_by_team = group_schedules_by_team(schedules);
    let overrides_by_team = group_overrides_by_team(overrides);
    let mut team_ids = BTreeSet::new();
    for project in &projects {
        team_ids.insert(project.team_id.clone());
    }
    for team_id in schedules_by_team.keys() {
        team_ids.insert(team_id.clone());
    }
    for team_id in overrides_by_team.keys() {
        team_ids.insert(team_id.clone());
    }

    let mut by_team = BTreeMap::new();
    for team_id in team_ids {
        let team_schedules = schedules_by_team.get(&team_id).cloned().unwrap_or_default();
        let backlog_count = backlog_map.get(&team_id).copied().unwrap_or(0);
        let evaluation = TeamReconcileEvaluation::evaluate(
            team_id.clone(),
            &team_schedules,
            overrides_by_team.get(&team_id),
            at,
            backlog_count,
        );
        by_team.insert(team_id, evaluation);
    }

    let mut results = Vec::new();
    for project in projects {
        let Some(team_state) = by_team.get(&project.team_id) else {
            continue;
        };
        let target = resolve_project_daemon_target(&project, &placement_map, &host_map);
        let desired_state =
            if project.enabled { team_state.desired_state } else { DaemonDesiredState::Stopped };
        let reason = if project.enabled {
            team_state.reason.clone()
        } else {
            format!("project disabled in fleet registry; team reason: {}", team_state.reason)
        };

        let result = reconcile_project(
            target.controller(),
            project.team_id.clone(),
            project.id.clone(),
            project.ao_project_root.clone(),
            target.details(),
            desired_state,
            reason,
            team_state.backlog_count,
            team_state.schedule_ids.clone(),
            team_state.override_applied.clone(),
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

fn group_schedules_by_team(schedules: Vec<Schedule>) -> BTreeMap<String, Vec<Schedule>> {
    let mut schedules_by_team: BTreeMap<String, Vec<Schedule>> = BTreeMap::new();

    for schedule in schedules {
        schedules_by_team.entry(schedule.team_id.clone()).or_default().push(schedule);
    }

    schedules_by_team
}

fn group_overrides_by_team(overrides: Vec<DaemonOverride>) -> BTreeMap<String, DaemonOverride> {
    overrides
        .into_iter()
        .map(|override_record| (override_record.team_id.clone(), override_record))
        .collect()
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
