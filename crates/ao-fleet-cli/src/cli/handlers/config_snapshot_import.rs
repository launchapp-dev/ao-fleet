use std::collections::BTreeMap;

use anyhow::Result;
use ao_fleet_core::{NewProject, NewSchedule, NewTeam, Project, Schedule, Team};
use ao_fleet_store::FleetStore;
use chrono::Utc;
use serde::Serialize;

use crate::cli::handlers::config_snapshot_import_command::ConfigSnapshotImportCommand;
use crate::cli::handlers::fleet_config_snapshot::FleetConfigSnapshot;
use crate::cli::handlers::json_printer::print_json;

pub fn config_snapshot_import(db_path: &str, command: ConfigSnapshotImportCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let snapshot: FleetConfigSnapshot =
        serde_json::from_str(&std::fs::read_to_string(command.input)?)?;

    let team_map = import_teams(&store, &snapshot.teams)?;
    let project_map = import_projects(&store, &snapshot.projects, &team_map)?;
    let schedule_count = import_schedules(&store, &snapshot.schedules, &team_map)?;

    print_json(&ConfigSnapshotImportResult {
        version: snapshot.version,
        team_count: team_map.len(),
        project_count: project_map.len(),
        schedule_count,
    })
}

#[derive(Debug, Serialize)]
struct ConfigSnapshotImportResult {
    version: String,
    team_count: usize,
    project_count: usize,
    schedule_count: usize,
}

fn import_teams(store: &FleetStore, snapshot_teams: &[Team]) -> Result<BTreeMap<String, Team>> {
    let existing_by_slug = store
        .list_teams()?
        .into_iter()
        .map(|team| (team.slug.clone(), team))
        .collect::<BTreeMap<_, _>>();
    let now = Utc::now();
    let mut imported = BTreeMap::new();

    for snapshot_team in snapshot_teams {
        let team = match existing_by_slug.get(&snapshot_team.slug) {
            Some(existing) => store.update_team(Team {
                id: existing.id.clone(),
                slug: snapshot_team.slug.clone(),
                name: snapshot_team.name.clone(),
                mission: snapshot_team.mission.clone(),
                ownership: snapshot_team.ownership.clone(),
                business_priority: snapshot_team.business_priority,
                created_at: existing.created_at,
                updated_at: now,
            })?,
            None => store.create_team(NewTeam {
                slug: snapshot_team.slug.clone(),
                name: snapshot_team.name.clone(),
                mission: snapshot_team.mission.clone(),
                ownership: snapshot_team.ownership.clone(),
                business_priority: snapshot_team.business_priority,
            })?,
        };

        imported.insert(snapshot_team.id.clone(), team);
    }

    Ok(imported)
}

fn import_projects(
    store: &FleetStore,
    snapshot_projects: &[Project],
    team_map: &BTreeMap<String, Team>,
) -> Result<BTreeMap<String, Project>> {
    let existing_by_slug = store
        .list_projects(None)?
        .into_iter()
        .map(|project| (project.slug.clone(), project))
        .collect::<BTreeMap<_, _>>();
    let now = Utc::now();
    let mut imported = BTreeMap::new();

    for snapshot_project in snapshot_projects {
        let mapped_team_id = team_map
            .get(&snapshot_project.team_id)
            .map(|team| team.id.clone())
            .unwrap_or_else(|| snapshot_project.team_id.clone());

        let project = match existing_by_slug.get(&snapshot_project.slug) {
            Some(existing) => store.update_project(Project {
                id: existing.id.clone(),
                team_id: mapped_team_id,
                slug: snapshot_project.slug.clone(),
                root_path: snapshot_project.root_path.clone(),
                ao_project_root: snapshot_project.ao_project_root.clone(),
                default_branch: snapshot_project.default_branch.clone(),
                remote_url: snapshot_project.remote_url.clone(),
                enabled: snapshot_project.enabled,
                created_at: existing.created_at,
                updated_at: now,
            })?,
            None => store.create_project(NewProject {
                team_id: mapped_team_id,
                slug: snapshot_project.slug.clone(),
                root_path: snapshot_project.root_path.clone(),
                ao_project_root: snapshot_project.ao_project_root.clone(),
                default_branch: snapshot_project.default_branch.clone(),
                remote_url: snapshot_project.remote_url.clone(),
                enabled: snapshot_project.enabled,
            })?,
        };

        imported.insert(snapshot_project.id.clone(), project);
    }

    Ok(imported)
}

fn import_schedules(
    store: &FleetStore,
    snapshot_schedules: &[Schedule],
    team_map: &BTreeMap<String, Team>,
) -> Result<usize> {
    for team in team_map.values() {
        for schedule in store.list_schedules(Some(&team.id))? {
            store.delete_schedule(&schedule.id)?;
        }
    }

    let mut imported = 0_usize;
    for snapshot_schedule in snapshot_schedules {
        let mapped_team_id = team_map
            .get(&snapshot_schedule.team_id)
            .map(|team| team.id.clone())
            .unwrap_or_else(|| snapshot_schedule.team_id.clone());

        store.create_schedule(NewSchedule {
            team_id: mapped_team_id,
            timezone: snapshot_schedule.timezone.clone(),
            policy_kind: snapshot_schedule.policy_kind,
            windows: snapshot_schedule.windows.clone(),
            enabled: snapshot_schedule.enabled,
        })?;
        imported += 1;
    }

    Ok(imported)
}
