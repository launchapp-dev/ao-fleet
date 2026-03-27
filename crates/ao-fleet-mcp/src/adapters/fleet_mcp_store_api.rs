use std::collections::BTreeMap;

use ao_fleet_core::{
    DaemonDesiredState, NewProject, NewSchedule, NewTeam, Project, Schedule, Team,
};
use ao_fleet_scheduler::schedule_evaluator::ScheduleEvaluator;
use ao_fleet_store::{FleetOverview, FleetOverviewQuery, FleetStore};
use chrono::Utc;

use crate::api::fleet_mcp_api::FleetMcpApi;
use crate::error::fleet_mcp_error::FleetMcpError;
use crate::inputs::daemon_reconcile_input::DaemonReconcileInput;
use crate::inputs::project_create_input::ProjectCreateInput;
use crate::inputs::project_list_input::ProjectListInput;
use crate::inputs::schedule_create_input::ScheduleCreateInput;
use crate::inputs::schedule_list_input::ScheduleListInput;
use crate::inputs::team_create_input::TeamCreateInput;
use crate::inputs::team_list_input::TeamListInput;
use crate::results::daemon_reconcile_decision::DaemonReconcileDecision;
use crate::results::daemon_reconcile_result::DaemonReconcileResult;

pub struct FleetMcpStoreApi {
    store: FleetStore,
}

impl FleetMcpStoreApi {
    pub fn new(store: FleetStore) -> Self {
        Self { store }
    }
}

impl FleetMcpApi for FleetMcpStoreApi {
    fn fleet_overview(&self, input: FleetOverviewQuery) -> Result<FleetOverview, FleetMcpError> {
        self.store.fleet_overview(input).map_err(Into::into)
    }

    fn list_teams(&self, _input: TeamListInput) -> Result<Vec<Team>, FleetMcpError> {
        self.store.list_teams().map_err(Into::into)
    }

    fn create_team(&self, input: TeamCreateInput) -> Result<Team, FleetMcpError> {
        self.store
            .create_team(NewTeam {
                slug: input.slug,
                name: input.name,
                mission: input.mission,
                ownership: input.ownership,
                business_priority: input.business_priority,
            })
            .map_err(Into::into)
    }

    fn list_projects(&self, input: ProjectListInput) -> Result<Vec<Project>, FleetMcpError> {
        let projects = self.store.list_projects(input.team_id.as_deref())?;
        if input.enabled_only {
            Ok(projects.into_iter().filter(|project| project.enabled).collect())
        } else {
            Ok(projects)
        }
    }

    fn create_project(&self, input: ProjectCreateInput) -> Result<Project, FleetMcpError> {
        self.store
            .create_project(NewProject {
                team_id: input.team_id,
                slug: input.slug,
                root_path: input.root_path,
                ao_project_root: input.ao_project_root,
                default_branch: input.default_branch,
                enabled: input.enabled,
            })
            .map_err(Into::into)
    }

    fn list_schedules(&self, input: ScheduleListInput) -> Result<Vec<Schedule>, FleetMcpError> {
        let schedules = self.store.list_schedules(input.team_id.as_deref())?;
        if input.enabled_only {
            Ok(schedules.into_iter().filter(|schedule| schedule.enabled).collect())
        } else {
            Ok(schedules)
        }
    }

    fn create_schedule(&self, input: ScheduleCreateInput) -> Result<Schedule, FleetMcpError> {
        self.store
            .create_schedule(NewSchedule {
                team_id: input.team_id,
                timezone: input.timezone,
                policy_kind: input.policy_kind,
                windows: input.windows.into_iter().map(Into::into).collect(),
                enabled: input.enabled,
            })
            .map_err(Into::into)
    }

    fn reconcile_daemons(
        &self,
        input: DaemonReconcileInput,
    ) -> Result<DaemonReconcileResult, FleetMcpError> {
        let schedules = self.store.list_schedules(None)?;
        let evaluated_at = input.at.unwrap_or_else(Utc::now);

        let mut per_team: BTreeMap<String, DaemonReconcileDecision> = BTreeMap::new();
        for schedule in schedules {
            let backlog_count = input.backlog_by_team.get(&schedule.team_id).copied().unwrap_or(0);
            let desired_state = ScheduleEvaluator::evaluate(&schedule, evaluated_at, backlog_count);

            let entry = per_team.entry(schedule.team_id.clone()).or_insert_with(|| {
                DaemonReconcileDecision {
                    team_id: schedule.team_id.clone(),
                    desired_state,
                    backlog_count,
                    schedule_ids: Vec::new(),
                }
            });

            entry.desired_state = merge_desired_state(entry.desired_state, desired_state);
            entry.backlog_count = entry.backlog_count.max(backlog_count);
            entry.schedule_ids.push(schedule.id);
        }

        Ok(DaemonReconcileResult {
            evaluated_at,
            applied: input.apply,
            decisions: per_team.into_values().collect(),
        })
    }
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
