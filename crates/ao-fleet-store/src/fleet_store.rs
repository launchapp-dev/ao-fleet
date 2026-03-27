use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ao_fleet_core::{
    AuditEvent, DaemonDesiredState, NewAuditEvent, NewProject, NewSchedule, NewTeam, Project,
    Schedule, SchedulePolicyKind, Team, WeekdayWindow,
};
use ao_fleet_scheduler::schedule_evaluator::ScheduleEvaluator;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Row, params, types::Type};
use uuid::Uuid;

use crate::errors::store_error::StoreError;
use crate::models::fleet_overview::FleetOverview;
use crate::models::fleet_overview_query::FleetOverviewQuery;
use crate::models::fleet_overview_summary::FleetOverviewSummary;
use crate::models::fleet_reconcile_action::FleetReconcileAction;
use crate::models::fleet_reconcile_preview::FleetReconcilePreview;
use crate::models::fleet_reconcile_preview_item::FleetReconcilePreviewItem;
use crate::models::fleet_team_overview::FleetTeamOverview;
use crate::models::fleet_team_summary::FleetTeamSummary;

const MIGRATION_SQL: &[&str] = &[
    include_str!("../sql/migrations/001_enable_foreign_keys.sql"),
    include_str!("../sql/migrations/002_create_schema.sql"),
    include_str!("../sql/migrations/003_create_audit_events.sql"),
];

#[derive(Debug, Clone)]
pub struct FleetStore {
    conn: Arc<Mutex<Connection>>,
}

impl FleetStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                StoreError::validation(format!("failed to create database directory: {error}"))
            })?;
        }

        let connection = Connection::open(path.as_ref())?;
        connection.busy_timeout(Duration::from_secs(5))?;

        let store = Self { conn: Arc::new(Mutex::new(connection)) };
        store.run_migrations()?;
        Ok(store)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let connection = Connection::open_in_memory()?;
        connection.busy_timeout(Duration::from_secs(5))?;

        let store = Self { conn: Arc::new(Mutex::new(connection)) };
        store.run_migrations()?;
        Ok(store)
    }

    pub fn desired_state_from_enabled(enabled: bool) -> DaemonDesiredState {
        if enabled { DaemonDesiredState::Running } else { DaemonDesiredState::Stopped }
    }

    pub fn append_audit_event(&self, input: NewAuditEvent) -> Result<AuditEvent, StoreError> {
        input.validate()?;
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let audit_event = append_audit_event_with_connection(&tx, input)?;
        tx.commit()?;
        Ok(audit_event)
    }

    pub fn list_audit_events(
        &self,
        team_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<AuditEvent>, StoreError> {
        let conn = self.connection()?;
        let limit = limit.unwrap_or(100);
        let mut stmt = conn.prepare(include_str!("../sql/audit_event/list.sql"))?;
        let rows = stmt.query_map(params![team_id, limit as i64], audit_event_from_row)?;
        collect_rows(rows)
    }

    pub fn create_team(&self, input: NewTeam) -> Result<Team, StoreError> {
        input.validate()?;
        let now = Utc::now();
        let team = Team {
            id: new_id("team"),
            slug: input.slug,
            name: input.name,
            mission: input.mission,
            ownership: input.ownership,
            business_priority: input.business_priority,
            created_at: now,
            updated_at: now,
        };

        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let result = tx.execute(
            include_str!("../sql/team/insert.sql"),
            params![
                team.id,
                team.slug,
                team.name,
                team.mission,
                team.ownership,
                team.business_priority,
                team.created_at.to_rfc3339(),
                team.updated_at.to_rfc3339(),
            ],
        );

        match result {
            Ok(_) => {
                record_audit_event(
                    &tx,
                    NewAuditEvent {
                        team_id: Some(team.id.clone()),
                        entity_type: "team".to_string(),
                        entity_id: team.id.clone(),
                        action: "created".to_string(),
                        actor_type: "system".to_string(),
                        actor_id: None,
                        summary: format!("Created team {}", team.slug),
                        details: serde_json::json!({
                            "slug": team.slug,
                            "name": team.name,
                            "ownership": team.ownership,
                            "business_priority": team.business_priority,
                        }),
                    },
                )?;
                tx.commit()?;
                Ok(team)
            }
            Err(error) if is_unique_constraint(&error) => {
                Err(StoreError::already_exists("team", team.slug))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn list_teams(&self) -> Result<Vec<Team>, StoreError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(include_str!("../sql/team/list.sql"))?;
        let rows = stmt.query_map([], team_from_row)?;
        collect_rows(rows)
    }

    pub fn get_team(&self, id: &str) -> Result<Option<Team>, StoreError> {
        let conn = self.connection()?;
        conn.query_row(include_str!("../sql/team/get.sql"), params![id], team_from_row)
            .optional()
            .map_err(Into::into)
    }

    pub fn update_team(&self, team: Team) -> Result<Team, StoreError> {
        validate_team(&team)?;
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(
            include_str!("../sql/team/update.sql"),
            params![
                team.slug,
                team.name,
                team.mission,
                team.ownership,
                team.business_priority,
                team.updated_at.to_rfc3339(),
                team.id,
            ],
        )?;

        if changed == 0 {
            return Err(StoreError::not_found("team", team.id));
        }

        record_audit_event(
            &tx,
            NewAuditEvent {
                team_id: Some(team.id.clone()),
                entity_type: "team".to_string(),
                entity_id: team.id.clone(),
                action: "updated".to_string(),
                actor_type: "system".to_string(),
                actor_id: None,
                summary: format!("Updated team {}", team.slug),
                details: serde_json::json!({
                    "slug": team.slug,
                    "name": team.name,
                    "ownership": team.ownership,
                    "business_priority": team.business_priority,
                }),
            },
        )?;
        tx.commit()?;

        Ok(team)
    }

    pub fn delete_team(&self, id: &str) -> Result<bool, StoreError> {
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(include_str!("../sql/team/delete.sql"), params![id])?;
        if changed > 0 {
            record_audit_event(
                &tx,
                NewAuditEvent {
                    team_id: Some(id.to_string()),
                    entity_type: "team".to_string(),
                    entity_id: id.to_string(),
                    action: "deleted".to_string(),
                    actor_type: "system".to_string(),
                    actor_id: None,
                    summary: format!("Deleted team {id}"),
                    details: serde_json::json!({}),
                },
            )?;
            tx.commit()?;
        }
        Ok(changed > 0)
    }

    pub fn create_project(&self, input: NewProject) -> Result<Project, StoreError> {
        input.validate()?;
        let now = Utc::now();
        let project = Project {
            id: new_id("project"),
            team_id: input.team_id,
            slug: input.slug,
            root_path: input.root_path,
            ao_project_root: input.ao_project_root,
            default_branch: input.default_branch,
            enabled: input.enabled,
            created_at: now,
            updated_at: now,
        };

        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let result = tx.execute(
            include_str!("../sql/project/insert.sql"),
            params![
                project.id,
                project.team_id,
                project.slug,
                project.root_path,
                project.ao_project_root,
                project.default_branch,
                i64::from(project.enabled),
                project.created_at.to_rfc3339(),
                project.updated_at.to_rfc3339(),
            ],
        );

        match result {
            Ok(_) => {
                record_audit_event(
                    &tx,
                    NewAuditEvent {
                        team_id: Some(project.team_id.clone()),
                        entity_type: "project".to_string(),
                        entity_id: project.id.clone(),
                        action: "created".to_string(),
                        actor_type: "system".to_string(),
                        actor_id: None,
                        summary: format!("Created project {}", project.slug),
                        details: serde_json::json!({
                            "slug": project.slug,
                            "root_path": project.root_path,
                            "ao_project_root": project.ao_project_root,
                            "default_branch": project.default_branch,
                            "enabled": project.enabled,
                        }),
                    },
                )?;
                tx.commit()?;
                Ok(project)
            }
            Err(error) if is_unique_constraint(&error) => {
                Err(StoreError::already_exists("project", project.slug))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn list_projects(&self, team_id: Option<&str>) -> Result<Vec<Project>, StoreError> {
        let conn = self.connection()?;
        let sql = match team_id {
            Some(_) => include_str!("../sql/project/list_by_team.sql"),
            None => include_str!("../sql/project/list.sql"),
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = match team_id {
            Some(team_id) => stmt.query_map(params![team_id], project_from_row)?,
            None => stmt.query_map([], project_from_row)?,
        };
        collect_rows(rows)
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>, StoreError> {
        let conn = self.connection()?;
        conn.query_row(include_str!("../sql/project/get.sql"), params![id], project_from_row)
            .optional()
            .map_err(Into::into)
    }

    pub fn update_project(&self, project: Project) -> Result<Project, StoreError> {
        validate_project(&project)?;
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(
            include_str!("../sql/project/update.sql"),
            params![
                project.team_id,
                project.slug,
                project.root_path,
                project.ao_project_root,
                project.default_branch,
                i64::from(project.enabled),
                project.updated_at.to_rfc3339(),
                project.id,
            ],
        )?;

        if changed == 0 {
            return Err(StoreError::not_found("project", project.id));
        }

        record_audit_event(
            &tx,
            NewAuditEvent {
                team_id: Some(project.team_id.clone()),
                entity_type: "project".to_string(),
                entity_id: project.id.clone(),
                action: "updated".to_string(),
                actor_type: "system".to_string(),
                actor_id: None,
                summary: format!("Updated project {}", project.slug),
                details: serde_json::json!({
                    "slug": project.slug,
                    "root_path": project.root_path,
                    "ao_project_root": project.ao_project_root,
                    "default_branch": project.default_branch,
                    "enabled": project.enabled,
                }),
            },
        )?;
        tx.commit()?;

        Ok(project)
    }

    pub fn delete_project(&self, id: &str) -> Result<bool, StoreError> {
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(include_str!("../sql/project/delete.sql"), params![id])?;
        if changed > 0 {
            record_audit_event(
                &tx,
                NewAuditEvent {
                    team_id: None,
                    entity_type: "project".to_string(),
                    entity_id: id.to_string(),
                    action: "deleted".to_string(),
                    actor_type: "system".to_string(),
                    actor_id: None,
                    summary: format!("Deleted project {id}"),
                    details: serde_json::json!({}),
                },
            )?;
            tx.commit()?;
        }
        Ok(changed > 0)
    }

    pub fn create_schedule(&self, input: NewSchedule) -> Result<Schedule, StoreError> {
        input.validate()?;
        let now = Utc::now();
        let schedule = Schedule {
            id: new_id("schedule"),
            team_id: input.team_id,
            timezone: input.timezone,
            policy_kind: input.policy_kind,
            windows: input.windows,
            enabled: input.enabled,
            created_at: now,
            updated_at: now,
        };

        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let windows_json = serde_json::to_string(&schedule.windows)?;
        let result = tx.execute(
            include_str!("../sql/schedule/insert.sql"),
            params![
                schedule.id,
                schedule.team_id,
                schedule.timezone,
                policy_kind_to_text(schedule.policy_kind),
                windows_json,
                i64::from(schedule.enabled),
                schedule.created_at.to_rfc3339(),
                schedule.updated_at.to_rfc3339(),
            ],
        );

        match result {
            Ok(_) => {
                record_audit_event(
                    &tx,
                    NewAuditEvent {
                        team_id: Some(schedule.team_id.clone()),
                        entity_type: "schedule".to_string(),
                        entity_id: schedule.id.clone(),
                        action: "created".to_string(),
                        actor_type: "system".to_string(),
                        actor_id: None,
                        summary: format!("Created schedule {}", schedule.id),
                        details: serde_json::json!({
                            "policy_kind": policy_kind_to_text(schedule.policy_kind),
                            "timezone": schedule.timezone,
                            "enabled": schedule.enabled,
                            "window_count": schedule.windows.len(),
                        }),
                    },
                )?;
                tx.commit()?;
                Ok(schedule)
            }
            Err(error) if is_unique_constraint(&error) => {
                Err(StoreError::already_exists("schedule", schedule.id))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn list_schedules(&self, team_id: Option<&str>) -> Result<Vec<Schedule>, StoreError> {
        let conn = self.connection()?;
        let sql = match team_id {
            Some(_) => include_str!("../sql/schedule/list_by_team.sql"),
            None => include_str!("../sql/schedule/list.sql"),
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = match team_id {
            Some(team_id) => stmt.query_map(params![team_id], schedule_from_row)?,
            None => stmt.query_map([], schedule_from_row)?,
        };
        collect_rows(rows)
    }

    pub fn get_schedule(&self, id: &str) -> Result<Option<Schedule>, StoreError> {
        let conn = self.connection()?;
        conn.query_row(include_str!("../sql/schedule/get.sql"), params![id], schedule_from_row)
            .optional()
            .map_err(Into::into)
    }

    pub fn update_schedule(&self, schedule: Schedule) -> Result<Schedule, StoreError> {
        validate_schedule(&schedule)?;
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let windows_json = serde_json::to_string(&schedule.windows)?;
        let changed = tx.execute(
            include_str!("../sql/schedule/update.sql"),
            params![
                schedule.team_id,
                schedule.timezone,
                policy_kind_to_text(schedule.policy_kind),
                windows_json,
                i64::from(schedule.enabled),
                schedule.updated_at.to_rfc3339(),
                schedule.id,
            ],
        )?;

        if changed == 0 {
            return Err(StoreError::not_found("schedule", schedule.id));
        }

        record_audit_event(
            &tx,
            NewAuditEvent {
                team_id: Some(schedule.team_id.clone()),
                entity_type: "schedule".to_string(),
                entity_id: schedule.id.clone(),
                action: "updated".to_string(),
                actor_type: "system".to_string(),
                actor_id: None,
                summary: format!("Updated schedule {}", schedule.id),
                details: serde_json::json!({
                    "policy_kind": policy_kind_to_text(schedule.policy_kind),
                    "timezone": schedule.timezone,
                    "enabled": schedule.enabled,
                    "window_count": schedule.windows.len(),
                }),
            },
        )?;
        tx.commit()?;

        Ok(schedule)
    }

    pub fn delete_schedule(&self, id: &str) -> Result<bool, StoreError> {
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(include_str!("../sql/schedule/delete.sql"), params![id])?;
        if changed > 0 {
            record_audit_event(
                &tx,
                NewAuditEvent {
                    team_id: None,
                    entity_type: "schedule".to_string(),
                    entity_id: id.to_string(),
                    action: "deleted".to_string(),
                    actor_type: "system".to_string(),
                    actor_id: None,
                    summary: format!("Deleted schedule {id}"),
                    details: serde_json::json!({}),
                },
            )?;
            tx.commit()?;
        }
        Ok(changed > 0)
    }

    pub fn fleet_overview(&self, query: FleetOverviewQuery) -> Result<FleetOverview, StoreError> {
        let evaluated_at = query.at.unwrap_or_else(Utc::now);
        let team_filter = query.team_id.as_deref();

        let teams = self
            .list_teams()?
            .into_iter()
            .filter(|team| team_filter.map_or(true, |team_id| team.id == team_id))
            .collect::<Vec<_>>();
        let projects = self.list_projects(team_filter)?;
        let schedules = self.list_schedules(team_filter)?;

        let mut projects_by_team: BTreeMap<String, Vec<Project>> = BTreeMap::new();
        for project in projects {
            projects_by_team.entry(project.team_id.clone()).or_default().push(project);
        }

        let mut schedules_by_team: BTreeMap<String, Vec<Schedule>> = BTreeMap::new();
        for schedule in schedules {
            schedules_by_team.entry(schedule.team_id.clone()).or_default().push(schedule);
        }

        let mut team_overviews = Vec::with_capacity(teams.len());
        let mut preview_items = Vec::with_capacity(teams.len());
        let mut team_count = 0_usize;
        let mut project_count = 0_usize;
        let mut schedule_count = 0_usize;
        let mut enabled_project_count = 0_usize;
        let mut enabled_schedule_count = 0_usize;

        for team in teams {
            team_count += 1;
            let team_projects = projects_by_team.remove(&team.id).unwrap_or_default();
            let team_schedules = schedules_by_team.remove(&team.id).unwrap_or_default();
            let backlog_count = query.backlog_by_team.get(&team.id).copied().unwrap_or(0);
            let desired_state =
                reconcile_desired_state(&team_schedules, evaluated_at, backlog_count);
            let observed_state = query
                .observed_state_by_team
                .get(&team.id)
                .copied()
                .unwrap_or_else(|| infer_observed_state(&team_projects));

            let reconcile_preview = FleetReconcilePreviewItem {
                team_id: team.id.clone(),
                team_slug: team.slug.clone(),
                desired_state,
                observed_state,
                action: reconcile_action(desired_state, observed_state),
                backlog_count,
                schedule_ids: team_schedules.iter().map(|schedule| schedule.id.clone()).collect(),
            };

            let summary = FleetTeamSummary {
                project_count: team_projects.len(),
                enabled_project_count: team_projects
                    .iter()
                    .filter(|project| project.enabled)
                    .count(),
                schedule_count: team_schedules.len(),
                enabled_schedule_count: team_schedules
                    .iter()
                    .filter(|schedule| schedule.enabled)
                    .count(),
                backlog_count,
            };

            project_count += summary.project_count;
            schedule_count += summary.schedule_count;
            enabled_project_count += summary.enabled_project_count;
            enabled_schedule_count += summary.enabled_schedule_count;

            preview_items.push(reconcile_preview.clone());
            team_overviews.push(FleetTeamOverview {
                team,
                summary,
                projects: team_projects,
                schedules: team_schedules,
                reconcile_preview,
            });
        }

        Ok(FleetOverview {
            evaluated_at: evaluated_at.clone(),
            summary: FleetOverviewSummary {
                team_count,
                project_count,
                schedule_count,
                enabled_project_count,
                enabled_schedule_count,
            },
            teams: team_overviews,
            preview: FleetReconcilePreview { evaluated_at, items: preview_items },
        })
    }

    fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>, StoreError> {
        self.conn.lock().map_err(|_| StoreError::validation("database connection lock poisoned"))
    }

    fn run_migrations(&self) -> Result<(), StoreError> {
        let conn = self.connection()?;
        for migration in MIGRATION_SQL {
            conn.execute_batch(migration)?;
        }
        Ok(())
    }
}

fn new_id(prefix: &str) -> String {
    format!("{prefix}_{}", Uuid::now_v7())
}

fn is_unique_constraint(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(code, _) if matches!(code.code, rusqlite::ErrorCode::ConstraintViolation)
    )
}

fn collect_rows<T, I>(rows: I) -> Result<Vec<T>, StoreError>
where
    I: Iterator<Item = Result<T, rusqlite::Error>>,
{
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

fn parse_datetime_sql(column: usize, value: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value).map(|value| value.with_timezone(&Utc)).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(error))
    })
}

fn bool_from_i64(value: i64) -> bool {
    value != 0
}

fn reconcile_desired_state(
    schedules: &[Schedule],
    evaluated_at: DateTime<Utc>,
    backlog_count: usize,
) -> DaemonDesiredState {
    schedules.iter().fold(DaemonDesiredState::Stopped, |current, schedule| {
        let desired_state = ScheduleEvaluator::evaluate(schedule, evaluated_at, backlog_count);
        merge_desired_state(current, desired_state)
    })
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

fn infer_observed_state(projects: &[Project]) -> DaemonDesiredState {
    if projects.is_empty() {
        return DaemonDesiredState::Stopped;
    }

    let any_enabled = projects.iter().any(|project| project.enabled);
    let any_disabled = projects.iter().any(|project| !project.enabled);

    match (any_enabled, any_disabled) {
        (true, false) => DaemonDesiredState::Running,
        (false, true) => DaemonDesiredState::Stopped,
        (true, true) => DaemonDesiredState::Paused,
        (false, false) => DaemonDesiredState::Stopped,
    }
}

fn reconcile_action(
    desired_state: DaemonDesiredState,
    observed_state: DaemonDesiredState,
) -> FleetReconcileAction {
    match (desired_state, observed_state) {
        (DaemonDesiredState::Running, DaemonDesiredState::Running)
        | (DaemonDesiredState::Paused, DaemonDesiredState::Paused)
        | (DaemonDesiredState::Stopped, DaemonDesiredState::Stopped) => FleetReconcileAction::Keep,
        (DaemonDesiredState::Running, DaemonDesiredState::Paused) => FleetReconcileAction::Resume,
        (DaemonDesiredState::Running, DaemonDesiredState::Stopped) => FleetReconcileAction::Start,
        (DaemonDesiredState::Paused, DaemonDesiredState::Running) => FleetReconcileAction::Pause,
        (DaemonDesiredState::Paused, DaemonDesiredState::Stopped) => {
            FleetReconcileAction::StartPaused
        }
        (DaemonDesiredState::Stopped, DaemonDesiredState::Running)
        | (DaemonDesiredState::Stopped, DaemonDesiredState::Paused) => FleetReconcileAction::Stop,
    }
}

fn audit_event_from_row(row: &Row<'_>) -> Result<AuditEvent, rusqlite::Error> {
    let details_json: String = row.get(8)?;
    let details: serde_json::Value = serde_json::from_str(&details_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(8, Type::Text, Box::new(error))
    })?;

    Ok(AuditEvent {
        id: row.get(0)?,
        team_id: row.get(1)?,
        entity_type: row.get(2)?,
        entity_id: row.get(3)?,
        action: row.get(4)?,
        actor_type: row.get(5)?,
        actor_id: row.get(6)?,
        summary: row.get(7)?,
        details,
        occurred_at: parse_datetime_sql(9, row.get::<_, String>(9)?)?,
    })
}

fn policy_kind_to_text(policy_kind: SchedulePolicyKind) -> &'static str {
    match policy_kind {
        SchedulePolicyKind::AlwaysOn => "always_on",
        SchedulePolicyKind::BusinessHours => "business_hours",
        SchedulePolicyKind::Nightly => "nightly",
        SchedulePolicyKind::ManualOnly => "manual_only",
        SchedulePolicyKind::BurstOnBacklog => "burst_on_backlog",
    }
}

fn policy_kind_from_text_sql(value: String) -> rusqlite::Result<SchedulePolicyKind> {
    match value.as_str() {
        "always_on" => Ok(SchedulePolicyKind::AlwaysOn),
        "business_hours" => Ok(SchedulePolicyKind::BusinessHours),
        "nightly" => Ok(SchedulePolicyKind::Nightly),
        "manual_only" => Ok(SchedulePolicyKind::ManualOnly),
        "burst_on_backlog" => Ok(SchedulePolicyKind::BurstOnBacklog),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            3,
            Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown schedule policy kind: {other}"),
            )),
        )),
    }
}

fn validate_team(team: &Team) -> Result<(), StoreError> {
    if team.id.trim().is_empty()
        || team.slug.trim().is_empty()
        || team.name.trim().is_empty()
        || team.mission.trim().is_empty()
        || team.ownership.trim().is_empty()
    {
        return Err(StoreError::validation("team fields cannot be empty"));
    }

    Ok(())
}

fn validate_project(project: &Project) -> Result<(), StoreError> {
    if project.id.trim().is_empty()
        || project.team_id.trim().is_empty()
        || project.slug.trim().is_empty()
        || project.root_path.trim().is_empty()
        || project.ao_project_root.trim().is_empty()
        || project.default_branch.trim().is_empty()
    {
        return Err(StoreError::validation("project fields cannot be empty"));
    }

    Ok(())
}

fn validate_schedule(schedule: &Schedule) -> Result<(), StoreError> {
    if schedule.id.trim().is_empty()
        || schedule.team_id.trim().is_empty()
        || schedule.timezone.trim().is_empty()
    {
        return Err(StoreError::validation("schedule fields cannot be empty"));
    }

    match schedule.policy_kind {
        SchedulePolicyKind::BusinessHours => {
            if schedule.windows.is_empty() {
                return Err(StoreError::validation(
                    "business_hours schedules require at least one window",
                ));
            }

            if schedule.windows.iter().any(|window| window.weekdays.is_empty()) {
                return Err(StoreError::validation(
                    "business_hours windows require at least one weekday",
                ));
            }

            if schedule.windows.iter().any(|window| window.start_hour > window.end_hour) {
                return Err(StoreError::validation(
                    "business_hours windows cannot wrap past midnight",
                ));
            }
        }
        SchedulePolicyKind::Nightly => {
            if schedule.windows.is_empty() {
                return Err(StoreError::validation(
                    "nightly schedules require at least one window",
                ));
            }
        }
        SchedulePolicyKind::AlwaysOn
        | SchedulePolicyKind::ManualOnly
        | SchedulePolicyKind::BurstOnBacklog => {}
    }

    for window in &schedule.windows {
        validate_window(window)?;
    }

    Ok(())
}

fn validate_window(window: &WeekdayWindow) -> Result<(), StoreError> {
    if window.weekdays.iter().any(|weekday| *weekday > 6) {
        return Err(StoreError::validation("weekday window weekdays must be in the range 0..=6"));
    }

    if window.start_hour > 23 || window.end_hour > 24 || window.start_hour == window.end_hour {
        return Err(StoreError::validation(
            "weekday window hours must satisfy 0 <= start <= 23, 0 <= end <= 24, and start != end",
        ));
    }

    Ok(())
}

fn team_from_row(row: &Row<'_>) -> Result<Team, rusqlite::Error> {
    Ok(Team {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        mission: row.get(3)?,
        ownership: row.get(4)?,
        business_priority: row.get(5)?,
        created_at: parse_datetime_sql(6, row.get::<_, String>(6)?)?,
        updated_at: parse_datetime_sql(7, row.get::<_, String>(7)?)?,
    })
}

fn project_from_row(row: &Row<'_>) -> Result<Project, rusqlite::Error> {
    Ok(Project {
        id: row.get(0)?,
        team_id: row.get(1)?,
        slug: row.get(2)?,
        root_path: row.get(3)?,
        ao_project_root: row.get(4)?,
        default_branch: row.get(5)?,
        enabled: bool_from_i64(row.get(6)?),
        created_at: parse_datetime_sql(7, row.get::<_, String>(7)?)?,
        updated_at: parse_datetime_sql(8, row.get::<_, String>(8)?)?,
    })
}

fn schedule_from_row(row: &Row<'_>) -> Result<Schedule, rusqlite::Error> {
    let windows_json: String = row.get(4)?;
    let windows: Vec<WeekdayWindow> = serde_json::from_str(&windows_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, Type::Text, Box::new(error))
    })?;

    Ok(Schedule {
        id: row.get(0)?,
        team_id: row.get(1)?,
        timezone: row.get(2)?,
        policy_kind: policy_kind_from_text_sql(row.get::<_, String>(3)?)?,
        windows,
        enabled: bool_from_i64(row.get(5)?),
        created_at: parse_datetime_sql(6, row.get::<_, String>(6)?)?,
        updated_at: parse_datetime_sql(7, row.get::<_, String>(7)?)?,
    })
}

fn record_audit_event(conn: &Connection, input: NewAuditEvent) -> Result<AuditEvent, StoreError> {
    append_audit_event_with_connection(conn, input)
}

fn append_audit_event_with_connection(
    conn: &Connection,
    input: NewAuditEvent,
) -> Result<AuditEvent, StoreError> {
    input.validate()?;
    let audit_event = AuditEvent {
        id: new_id("audit_event"),
        team_id: input.team_id,
        entity_type: input.entity_type,
        entity_id: input.entity_id,
        action: input.action,
        actor_type: input.actor_type,
        actor_id: input.actor_id,
        summary: input.summary,
        details: input.details,
        occurred_at: Utc::now(),
    };

    conn.execute(
        include_str!("../sql/audit_event/insert.sql"),
        params![
            audit_event.id,
            audit_event.team_id,
            audit_event.entity_type,
            audit_event.entity_id,
            audit_event.action,
            audit_event.actor_type,
            audit_event.actor_id,
            audit_event.summary,
            audit_event.details.to_string(),
            audit_event.occurred_at.to_rfc3339(),
        ],
    )?;

    Ok(audit_event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ao_fleet_core::{
        NewAuditEvent, NewProject, NewSchedule, NewTeam, SchedulePolicyKind, WeekdayWindow,
    };
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    #[test]
    fn store_crud_round_trip() {
        let store = FleetStore::open_in_memory().expect("store opens");

        let team = store
            .create_team(NewTeam {
                slug: "marketing".to_string(),
                name: "Marketing".to_string(),
                mission: "Own campaigns and launches".to_string(),
                ownership: "growth".to_string(),
                business_priority: 10,
            })
            .expect("team created");

        assert_eq!(store.list_teams().expect("teams listed").len(), 1);
        assert_eq!(
            store.get_team(&team.id).expect("team fetched").expect("team exists").slug,
            "marketing"
        );

        let project = store
            .create_project(NewProject {
                team_id: team.id.clone(),
                slug: "launch-site".to_string(),
                root_path: "/tmp/launch-site".to_string(),
                ao_project_root: "/tmp/launch-site".to_string(),
                default_branch: "main".to_string(),
                enabled: true,
            })
            .expect("project created");

        assert_eq!(store.list_projects(None).expect("projects listed").len(), 1);
        assert_eq!(store.list_projects(Some(&team.id)).expect("projects by team listed").len(), 1);
        assert!(store.get_project(&project.id).expect("project fetched").is_some());

        let schedule = store
            .create_schedule(NewSchedule {
                team_id: team.id.clone(),
                timezone: "UTC".to_string(),
                policy_kind: SchedulePolicyKind::BusinessHours,
                windows: vec![WeekdayWindow {
                    weekdays: vec![1, 2, 3, 4, 5],
                    start_hour: 9,
                    end_hour: 17,
                }],
                enabled: true,
            })
            .expect("schedule created");

        assert_eq!(store.list_schedules(None).expect("schedules listed").len(), 1);
        assert_eq!(
            store.list_schedules(Some(&team.id)).expect("schedules by team listed").len(),
            1
        );
        assert_eq!(
            store
                .get_schedule(&schedule.id)
                .expect("schedule fetched")
                .expect("schedule exists")
                .timezone,
            "UTC"
        );

        let mut updated_team = team.clone();
        updated_team.name = "Growth Marketing".to_string();
        updated_team.updated_at = Utc::now();
        store.update_team(updated_team.clone()).expect("team updated");
        assert_eq!(
            store.get_team(&team.id).expect("team fetched").expect("team exists").name,
            "Growth Marketing"
        );

        let mut updated_project = project.clone();
        updated_project.enabled = false;
        updated_project.updated_at = Utc::now();
        store.update_project(updated_project.clone()).expect("project updated");
        assert!(
            !store
                .get_project(&project.id)
                .expect("project fetched")
                .expect("project exists")
                .enabled
        );

        let mut updated_schedule = schedule.clone();
        updated_schedule.enabled = false;
        updated_schedule.updated_at = Utc::now();
        store.update_schedule(updated_schedule.clone()).expect("schedule updated");
        assert!(
            !store
                .get_schedule(&schedule.id)
                .expect("schedule fetched")
                .expect("schedule exists")
                .enabled
        );

        assert!(store.delete_project(&project.id).expect("project deleted"));
        assert!(store.get_project(&project.id).expect("project fetched").is_none());

        assert!(store.delete_schedule(&schedule.id).expect("schedule deleted"));
        assert!(store.get_schedule(&schedule.id).expect("schedule fetched").is_none());

        assert!(store.delete_team(&team.id).expect("team deleted"));
        assert!(store.get_team(&team.id).expect("team fetched").is_none());

        assert_eq!(
            store.list_audit_events(Some(&team.id), None).expect("audit events listed").len(),
            7
        );
    }

    #[test]
    fn audit_event_append_and_list_round_trip() {
        let store = FleetStore::open_in_memory().expect("store opens");

        let event = store
            .append_audit_event(NewAuditEvent {
                team_id: Some("team-123".to_string()),
                entity_type: "team".to_string(),
                entity_id: "team-123".to_string(),
                action: "created".to_string(),
                actor_type: "system".to_string(),
                actor_id: None,
                summary: "Created team".to_string(),
                details: json!({"source": "test"}),
            })
            .expect("event appended");

        assert_eq!(event.team_id.as_deref(), Some("team-123"));
        assert_eq!(
            store.list_audit_events(Some("team-123"), None).expect("audit events listed").len(),
            1
        );
    }

    #[test]
    fn fleet_overview_summarizes_inventory_and_preview() {
        let store = FleetStore::open_in_memory().expect("store opens");

        let team = store
            .create_team(NewTeam {
                slug: "marketing".to_string(),
                name: "Marketing".to_string(),
                mission: "Own campaigns and launches".to_string(),
                ownership: "growth".to_string(),
                business_priority: 10,
            })
            .expect("team created");

        store
            .create_project(NewProject {
                team_id: team.id.clone(),
                slug: "launch-site".to_string(),
                root_path: "/tmp/launch-site".to_string(),
                ao_project_root: "/tmp/launch-site".to_string(),
                default_branch: "main".to_string(),
                enabled: true,
            })
            .expect("first project created");

        store
            .create_project(NewProject {
                team_id: team.id.clone(),
                slug: "campaigns".to_string(),
                root_path: "/tmp/campaigns".to_string(),
                ao_project_root: "/tmp/campaigns".to_string(),
                default_branch: "main".to_string(),
                enabled: false,
            })
            .expect("second project created");

        let schedule = store
            .create_schedule(NewSchedule {
                team_id: team.id.clone(),
                timezone: "UTC".to_string(),
                policy_kind: SchedulePolicyKind::BusinessHours,
                windows: vec![WeekdayWindow {
                    weekdays: vec![0, 1, 2, 3, 4],
                    start_hour: 9,
                    end_hour: 17,
                }],
                enabled: true,
            })
            .expect("schedule created");

        let mut backlog_by_team = std::collections::BTreeMap::new();
        backlog_by_team.insert(team.id.clone(), 3);

        let overview = store
            .fleet_overview(FleetOverviewQuery {
                team_id: Some(team.id.clone()),
                at: Some(Utc.with_ymd_and_hms(2025, 3, 3, 10, 0, 0).unwrap()),
                backlog_by_team,
                observed_state_by_team: std::collections::BTreeMap::new(),
            })
            .expect("overview built");

        assert_eq!(overview.summary.team_count, 1);
        assert_eq!(overview.summary.project_count, 2);
        assert_eq!(overview.summary.schedule_count, 1);
        assert_eq!(overview.summary.enabled_project_count, 1);
        assert_eq!(overview.summary.enabled_schedule_count, 1);
        assert_eq!(overview.teams.len(), 1);

        let team_overview = &overview.teams[0];
        assert_eq!(team_overview.team.id, team.id);
        assert_eq!(team_overview.summary.project_count, 2);
        assert_eq!(team_overview.summary.enabled_project_count, 1);
        assert_eq!(team_overview.summary.backlog_count, 3);
        assert_eq!(team_overview.reconcile_preview.desired_state, DaemonDesiredState::Running);
        assert_eq!(team_overview.reconcile_preview.observed_state, DaemonDesiredState::Paused);
        assert_eq!(team_overview.reconcile_preview.action, FleetReconcileAction::Resume);
        assert_eq!(team_overview.reconcile_preview.schedule_ids, vec![schedule.id]);

        assert_eq!(overview.preview.items.len(), 1);
        assert_eq!(overview.preview.items[0].team_id, team.id);
        assert_eq!(overview.preview.items[0].action, FleetReconcileAction::Resume);
    }
}
