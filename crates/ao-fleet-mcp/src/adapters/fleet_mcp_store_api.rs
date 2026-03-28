use std::collections::BTreeMap;

use ao_fleet_core::{
    DaemonOverride, Host, KnowledgeDocument, KnowledgeFact, KnowledgeSource, NewDaemonOverride,
    NewProject, NewSchedule, NewTeam, Project, ProjectHostPlacement, Schedule, Team,
};
use ao_fleet_knowledge::{KnowledgeQuery, KnowledgeSearchResult, KnowledgeSearchService};
use ao_fleet_store::{
    FleetDaemonStatus, FleetOverview, FleetOverviewQuery, FleetStore, KnowledgeRecordQuery,
    TeamReconcileEvaluation,
};
use chrono::Utc;

use crate::api::fleet_mcp_api::FleetMcpApi;
use crate::error::fleet_mcp_error::FleetMcpError;
use crate::inputs::daemon_override_clear_input::DaemonOverrideClearInput;
use crate::inputs::daemon_override_list_input::DaemonOverrideListInput;
use crate::inputs::daemon_override_upsert_input::DaemonOverrideUpsertInput;
use crate::inputs::daemon_reconcile_input::DaemonReconcileInput;
use crate::inputs::daemon_status_input::DaemonStatusInput;
use crate::inputs::host_get_input::HostGetInput;
use crate::inputs::host_list_input::HostListInput;
use crate::inputs::knowledge_document_create_input::KnowledgeDocumentCreateInput;
use crate::inputs::knowledge_fact_create_input::KnowledgeFactCreateInput;
use crate::inputs::knowledge_record_list_input::KnowledgeRecordListInput;
use crate::inputs::knowledge_search_input::KnowledgeSearchInput;
use crate::inputs::knowledge_source_upsert_input::KnowledgeSourceUpsertInput;
use crate::inputs::project_create_input::ProjectCreateInput;
use crate::inputs::project_host_placement_assign_input::ProjectHostPlacementAssignInput;
use crate::inputs::project_host_placement_clear_input::ProjectHostPlacementClearInput;
use crate::inputs::project_host_placement_list_input::ProjectHostPlacementListInput;
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

    fn daemon_statuses(
        &self,
        input: DaemonStatusInput,
    ) -> Result<Vec<FleetDaemonStatus>, FleetMcpError> {
        self.store.fleet_daemon_statuses(input.team_id.as_deref()).map_err(Into::into)
    }

    fn list_hosts(&self, _input: HostListInput) -> Result<Vec<Host>, FleetMcpError> {
        self.store.list_hosts().map_err(Into::into)
    }

    fn get_host(&self, input: HostGetInput) -> Result<Option<Host>, FleetMcpError> {
        self.store.get_host(&input.id).map_err(Into::into)
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
                remote_url: input.remote_url,
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

    fn list_project_host_placements(
        &self,
        input: ProjectHostPlacementListInput,
    ) -> Result<Vec<ProjectHostPlacement>, FleetMcpError> {
        let placements = self.store.list_project_host_placements()?;
        if let Some(team_id) = input.team_id.as_deref() {
            let project_ids = self
                .store
                .list_projects(Some(team_id))?
                .into_iter()
                .map(|project| project.id)
                .collect::<std::collections::BTreeSet<_>>();
            Ok(placements
                .into_iter()
                .filter(|placement| project_ids.contains(&placement.project_id))
                .collect())
        } else {
            Ok(placements)
        }
    }

    fn assign_project_host_placement(
        &self,
        input: ProjectHostPlacementAssignInput,
    ) -> Result<ProjectHostPlacement, FleetMcpError> {
        self.store
            .upsert_project_host_placement(ProjectHostPlacement {
                project_id: input.project_id,
                host_id: input.host_id,
                assignment_source: input.assignment_source,
                assigned_at: Utc::now(),
            })
            .map_err(Into::into)
    }

    fn clear_project_host_placement(
        &self,
        input: ProjectHostPlacementClearInput,
    ) -> Result<bool, FleetMcpError> {
        self.store.clear_project_host_placement(&input.project_id).map_err(Into::into)
    }

    fn list_daemon_overrides(
        &self,
        input: DaemonOverrideListInput,
    ) -> Result<Vec<DaemonOverride>, FleetMcpError> {
        let overrides = self.store.list_daemon_overrides()?;
        Ok(match input.team_id {
            Some(team_id) => overrides
                .into_iter()
                .filter(|override_record| override_record.team_id == team_id)
                .collect(),
            None => overrides,
        })
    }

    fn upsert_daemon_override(
        &self,
        input: DaemonOverrideUpsertInput,
    ) -> Result<DaemonOverride, FleetMcpError> {
        self.store
            .upsert_daemon_override(NewDaemonOverride {
                team_id: input.team_id,
                mode: input.mode,
                forced_state: input.forced_state,
                pause_until: input.pause_until,
                note: input.note,
                source: input.source,
            })
            .map_err(Into::into)
    }

    fn clear_daemon_override(
        &self,
        input: DaemonOverrideClearInput,
    ) -> Result<bool, FleetMcpError> {
        self.store.clear_daemon_override(&input.team_id).map_err(Into::into)
    }

    fn list_knowledge_sources(
        &self,
        input: KnowledgeRecordListInput,
    ) -> Result<Vec<KnowledgeSource>, FleetMcpError> {
        self.store.list_knowledge_sources(record_query_from_input(input)).map_err(Into::into)
    }

    fn list_knowledge_documents(
        &self,
        input: KnowledgeRecordListInput,
    ) -> Result<Vec<KnowledgeDocument>, FleetMcpError> {
        self.store.list_knowledge_documents(record_query_from_input(input)).map_err(Into::into)
    }

    fn list_knowledge_facts(
        &self,
        input: KnowledgeRecordListInput,
    ) -> Result<Vec<KnowledgeFact>, FleetMcpError> {
        self.store.list_knowledge_facts(record_query_from_input(input)).map_err(Into::into)
    }

    fn search_knowledge(
        &self,
        input: KnowledgeSearchInput,
    ) -> Result<KnowledgeSearchResult, FleetMcpError> {
        let query = KnowledgeQuery {
            scope: input.scope,
            scope_ref: input.scope_ref.clone(),
            document_kinds: input.document_kinds,
            fact_kinds: input.fact_kinds,
            source_kinds: input.source_kinds,
            tags: input.tags,
            text: input.text,
            limit: input.limit,
        };
        let records = KnowledgeRecordQuery {
            scope: query.scope.clone(),
            scope_ref: query.scope_ref.clone(),
            limit: query.limit,
        };
        let documents = self.store.list_knowledge_documents(records.clone())?;
        let facts = self.store.list_knowledge_facts(records)?;

        Ok(KnowledgeSearchService::default().search(&query, &documents, &facts))
    }

    fn upsert_knowledge_source(
        &self,
        input: KnowledgeSourceUpsertInput,
    ) -> Result<KnowledgeSource, FleetMcpError> {
        let now = Utc::now();
        self.store
            .upsert_knowledge_source(KnowledgeSource {
                id: input.id.unwrap_or_default(),
                kind: input.kind,
                label: input.label,
                uri: input.uri,
                scope: input.scope,
                scope_ref: input.scope_ref,
                sync_state: input.sync_state,
                last_synced_at: input.last_synced_at,
                metadata: input.metadata,
                created_at: now,
                updated_at: now,
            })
            .map_err(Into::into)
    }

    fn create_knowledge_document(
        &self,
        input: KnowledgeDocumentCreateInput,
    ) -> Result<KnowledgeDocument, FleetMcpError> {
        let now = Utc::now();
        self.store
            .create_knowledge_document(KnowledgeDocument {
                id: input.id.unwrap_or_default(),
                scope: input.scope,
                scope_ref: input.scope_ref,
                kind: input.kind,
                title: input.title,
                summary: input.summary,
                body: input.body,
                source_id: input.source_id,
                source_kind: input.source_kind,
                tags: input.tags,
                created_at: now,
                updated_at: now,
            })
            .map_err(Into::into)
    }

    fn create_knowledge_fact(
        &self,
        input: KnowledgeFactCreateInput,
    ) -> Result<KnowledgeFact, FleetMcpError> {
        let now = Utc::now();
        self.store
            .create_knowledge_fact(KnowledgeFact {
                id: input.id.unwrap_or_default(),
                scope: input.scope,
                scope_ref: input.scope_ref,
                kind: input.kind,
                statement: input.statement,
                confidence: input.confidence,
                source_id: input.source_id,
                source_kind: input.source_kind,
                tags: input.tags,
                observed_at: input.observed_at.unwrap_or(now),
                created_at: now,
            })
            .map_err(Into::into)
    }

    fn reconcile_daemons(
        &self,
        input: DaemonReconcileInput,
    ) -> Result<DaemonReconcileResult, FleetMcpError> {
        let schedules = self.store.list_schedules(None)?;
        let overrides = self.store.list_daemon_overrides()?;
        let evaluated_at = input.at.unwrap_or_else(Utc::now);
        let schedules_by_team = group_schedules_by_team(schedules);
        let overrides_by_team = group_overrides_by_team(overrides);
        let mut team_ids = std::collections::BTreeSet::new();
        for team_id in schedules_by_team.keys() {
            team_ids.insert(team_id.clone());
        }
        for team_id in overrides_by_team.keys() {
            team_ids.insert(team_id.clone());
        }

        let mut per_team: BTreeMap<String, DaemonReconcileDecision> = BTreeMap::new();
        for team_id in team_ids {
            let team_schedules = schedules_by_team.get(&team_id).cloned().unwrap_or_default();
            let backlog_count = input.backlog_by_team.get(&team_id).copied().unwrap_or(0);
            let evaluation = TeamReconcileEvaluation::evaluate(
                team_id.clone(),
                &team_schedules,
                overrides_by_team.get(&team_id),
                evaluated_at,
                backlog_count,
            );

            per_team.insert(
                team_id.clone(),
                DaemonReconcileDecision {
                    team_id,
                    desired_state: evaluation.desired_state,
                    backlog_count,
                    schedule_ids: evaluation.schedule_ids,
                    reason: evaluation.reason,
                    override_applied: evaluation.override_applied,
                },
            );
        }

        Ok(DaemonReconcileResult {
            evaluated_at,
            applied: input.apply,
            decisions: per_team.into_values().collect(),
        })
    }
}

fn record_query_from_input(input: KnowledgeRecordListInput) -> KnowledgeRecordQuery {
    KnowledgeRecordQuery { scope: input.scope, scope_ref: input.scope_ref, limit: input.limit }
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
