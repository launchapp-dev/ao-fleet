use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ao_fleet_core::{
    AuditEvent, DaemonDesiredState, DaemonOverride, DaemonOverrideMode, Host, KnowledgeDocument,
    KnowledgeFact, KnowledgeScope, KnowledgeSource, NewAuditEvent, NewDaemonOverride, NewHost,
    NewProject, NewSchedule, NewTeam, ObservedDaemonStatus, Project, ProjectHostPlacement,
    Schedule, SchedulePolicyKind, Team, WeekdayWindow,
};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Row, params, types::Type};
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::errors::store_error::StoreError;
use crate::models::fleet_daemon_status::FleetDaemonStatus;
use crate::models::fleet_overview::FleetOverview;
use crate::models::fleet_overview_query::FleetOverviewQuery;
use crate::models::fleet_overview_summary::FleetOverviewSummary;
use crate::models::fleet_reconcile_action::FleetReconcileAction;
use crate::models::fleet_reconcile_preview::FleetReconcilePreview;
use crate::models::fleet_reconcile_preview_item::FleetReconcilePreviewItem;
use crate::models::fleet_team_overview::FleetTeamOverview;
use crate::models::fleet_team_summary::FleetTeamSummary;
use crate::models::founder_overview::FounderOverview;
use crate::models::founder_overview_summary::FounderOverviewSummary;
use crate::models::founder_team_overview::FounderTeamOverview;
use crate::models::knowledge_record_query::KnowledgeRecordQuery;
use crate::models::team_reconcile_evaluation::TeamReconcileEvaluation;

const MIGRATION_SQL: &[&str] = &[
    include_str!("../sql/migrations/001_enable_foreign_keys.sql"),
    include_str!("../sql/migrations/002_create_schema.sql"),
    include_str!("../sql/migrations/003_create_audit_events.sql"),
    include_str!("../sql/migrations/004_create_knowledge_tables.sql"),
    include_str!("../sql/migrations/005_create_observed_daemon_statuses.sql"),
    include_str!("../sql/migrations/006_create_hosts_and_placements.sql"),
    include_str!("../sql/migrations/007_add_project_remote_url.sql"),
    include_str!("../sql/migrations/008_create_daemon_overrides.sql"),
];

const FOUNDER_OVERVIEW_ACTIVITY_LIMIT: usize = 250;
const FOUNDER_OVERVIEW_KNOWLEDGE_LIMIT: usize = 250;

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

    pub fn upsert_daemon_override(
        &self,
        input: NewDaemonOverride,
    ) -> Result<DaemonOverride, StoreError> {
        input.validate()?;
        let existing = self.get_daemon_override(&input.team_id)?;
        let was_existing = existing.is_some();
        let now = Utc::now();
        let override_record = DaemonOverride {
            id: existing
                .as_ref()
                .map(|record| record.id.clone())
                .unwrap_or_else(|| new_id("daemon_override")),
            team_id: input.team_id,
            mode: input.mode,
            forced_state: input.forced_state,
            pause_until: input.pause_until,
            note: input.note,
            source: input.source,
            created_at: existing.as_ref().map(|record| record.created_at).unwrap_or(now),
            updated_at: now,
        };
        validate_daemon_override(&override_record)?;
        let mode_text = override_mode_to_text(override_record.mode).to_string();
        let forced_state_text =
            override_record.forced_state.map(desired_state_to_text).map(String::from);
        let pause_until_text = override_record.pause_until.map(|value| value.to_rfc3339());
        let note = override_record.note.clone();
        let source = override_record.source.clone();
        let id = override_record.id.clone();
        let team_id = override_record.team_id.clone();

        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        tx.execute(
            include_str!("../sql/daemon_override/upsert.sql"),
            params![
                id.clone(),
                team_id.clone(),
                mode_text.clone(),
                forced_state_text.clone(),
                pause_until_text.clone(),
                note.clone(),
                source.clone(),
                override_record.created_at.to_rfc3339(),
                override_record.updated_at.to_rfc3339(),
            ],
        )?;

        record_audit_event(
            &tx,
            NewAuditEvent {
                team_id: Some(team_id.clone()),
                entity_type: "daemon_override".to_string(),
                entity_id: id.clone(),
                action: if was_existing { "updated" } else { "created" }.to_string(),
                actor_type: "system".to_string(),
                actor_id: None,
                summary: format!(
                    "{} founder override for team {team_id}",
                    if was_existing { "Updated" } else { "Created" }
                ),
                details: serde_json::json!({
                    "mode": mode_text,
                    "forced_state": forced_state_text,
                    "pause_until": pause_until_text,
                    "note": note,
                    "source": source,
                }),
            },
        )?;
        tx.commit()?;

        Ok(override_record)
    }

    pub fn list_daemon_overrides(&self) -> Result<Vec<DaemonOverride>, StoreError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(include_str!("../sql/daemon_override/list.sql"))?;
        let rows = stmt.query_map([], daemon_override_from_row)?;
        collect_rows(rows)
    }

    pub fn get_daemon_override(&self, team_id: &str) -> Result<Option<DaemonOverride>, StoreError> {
        let conn = self.connection()?;
        conn.query_row(
            include_str!("../sql/daemon_override/get_by_team.sql"),
            params![team_id],
            daemon_override_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn clear_daemon_override(&self, team_id: &str) -> Result<bool, StoreError> {
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let changed = tx
            .execute(include_str!("../sql/daemon_override/delete_by_team.sql"), params![team_id])?;
        if changed > 0 {
            record_audit_event(
                &tx,
                NewAuditEvent {
                    team_id: Some(team_id.to_string()),
                    entity_type: "daemon_override".to_string(),
                    entity_id: team_id.to_string(),
                    action: "cleared".to_string(),
                    actor_type: "system".to_string(),
                    actor_id: None,
                    summary: format!("Cleared founder override for team {team_id}"),
                    details: serde_json::json!({}),
                },
            )?;
            tx.commit()?;
        }
        Ok(changed > 0)
    }

    pub fn create_host(&self, input: NewHost) -> Result<Host, StoreError> {
        input.validate()?;
        let now = Utc::now();
        let host = Host {
            id: new_id("host"),
            slug: input.slug,
            name: input.name,
            address: input.address,
            platform: input.platform,
            status: input.status,
            capacity_slots: input.capacity_slots,
            created_at: now,
            updated_at: now,
        };

        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let result = tx.execute(
            include_str!("../sql/host/insert.sql"),
            params![
                host.id,
                host.slug,
                host.name,
                host.address,
                host.platform,
                host.status,
                host.capacity_slots,
                host.created_at.to_rfc3339(),
                host.updated_at.to_rfc3339(),
            ],
        );

        match result {
            Ok(_) => {
                record_audit_event(
                    &tx,
                    NewAuditEvent {
                        team_id: None,
                        entity_type: "host".to_string(),
                        entity_id: host.id.clone(),
                        action: "created".to_string(),
                        actor_type: "system".to_string(),
                        actor_id: None,
                        summary: format!("Created host {}", host.slug),
                        details: serde_json::json!({
                            "slug": host.slug,
                            "address": host.address,
                            "platform": host.platform,
                            "status": host.status,
                            "capacity_slots": host.capacity_slots,
                        }),
                    },
                )?;
                tx.commit()?;
                Ok(host)
            }
            Err(error) if is_unique_constraint(&error) => {
                Err(StoreError::already_exists("host", host.slug))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn list_hosts(&self) -> Result<Vec<Host>, StoreError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(include_str!("../sql/host/list.sql"))?;
        let rows = stmt.query_map([], host_from_row)?;
        collect_rows(rows)
    }

    pub fn get_host(&self, id: &str) -> Result<Option<Host>, StoreError> {
        let conn = self.connection()?;
        conn.query_row(include_str!("../sql/host/get.sql"), params![id], host_from_row)
            .optional()
            .map_err(Into::into)
    }

    pub fn update_host(&self, host: Host) -> Result<Host, StoreError> {
        validate_host(&host)?;
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(
            include_str!("../sql/host/update.sql"),
            params![
                host.slug,
                host.name,
                host.address,
                host.platform,
                host.status,
                host.capacity_slots,
                host.updated_at.to_rfc3339(),
                host.id,
            ],
        )?;

        if changed == 0 {
            return Err(StoreError::not_found("host", host.id));
        }

        record_audit_event(
            &tx,
            NewAuditEvent {
                team_id: None,
                entity_type: "host".to_string(),
                entity_id: host.id.clone(),
                action: "updated".to_string(),
                actor_type: "system".to_string(),
                actor_id: None,
                summary: format!("Updated host {}", host.slug),
                details: serde_json::json!({
                    "slug": host.slug,
                    "address": host.address,
                    "platform": host.platform,
                    "status": host.status,
                    "capacity_slots": host.capacity_slots,
                }),
            },
        )?;
        tx.commit()?;
        Ok(host)
    }

    pub fn delete_host(&self, id: &str) -> Result<bool, StoreError> {
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(include_str!("../sql/host/delete.sql"), params![id])?;
        if changed > 0 {
            record_audit_event(
                &tx,
                NewAuditEvent {
                    team_id: None,
                    entity_type: "host".to_string(),
                    entity_id: id.to_string(),
                    action: "deleted".to_string(),
                    actor_type: "system".to_string(),
                    actor_id: None,
                    summary: format!("Deleted host {id}"),
                    details: serde_json::json!({}),
                },
            )?;
            tx.commit()?;
        }
        Ok(changed > 0)
    }

    pub fn upsert_project_host_placement(
        &self,
        placement: ProjectHostPlacement,
    ) -> Result<ProjectHostPlacement, StoreError> {
        validate_project_host_placement(&placement)?;
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        tx.execute(
            include_str!("../sql/project_host_placement/upsert.sql"),
            params![
                placement.project_id,
                placement.host_id,
                placement.assignment_source,
                placement.assigned_at.to_rfc3339(),
            ],
        )?;
        record_audit_event(
            &tx,
            NewAuditEvent {
                team_id: None,
                entity_type: "project_host_placement".to_string(),
                entity_id: placement.project_id.clone(),
                action: "upserted".to_string(),
                actor_type: "system".to_string(),
                actor_id: None,
                summary: format!(
                    "Assigned project {} to host {}",
                    placement.project_id, placement.host_id
                ),
                details: serde_json::json!({
                    "project_id": placement.project_id,
                    "host_id": placement.host_id,
                    "assignment_source": placement.assignment_source,
                }),
            },
        )?;
        tx.commit()?;
        Ok(placement)
    }

    pub fn clear_project_host_placement(&self, project_id: &str) -> Result<bool, StoreError> {
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let changed = tx.execute(
            include_str!("../sql/project_host_placement/delete.sql"),
            params![project_id],
        )?;
        if changed > 0 {
            record_audit_event(
                &tx,
                NewAuditEvent {
                    team_id: None,
                    entity_type: "project_host_placement".to_string(),
                    entity_id: project_id.to_string(),
                    action: "cleared".to_string(),
                    actor_type: "system".to_string(),
                    actor_id: None,
                    summary: format!("Cleared host placement for project {project_id}"),
                    details: serde_json::json!({ "project_id": project_id }),
                },
            )?;
            tx.commit()?;
        }
        Ok(changed > 0)
    }

    pub fn list_project_host_placements(&self) -> Result<Vec<ProjectHostPlacement>, StoreError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(include_str!("../sql/project_host_placement/list.sql"))?;
        let rows = stmt.query_map([], project_host_placement_from_row)?;
        collect_rows(rows)
    }

    pub fn upsert_observed_daemon_status(
        &self,
        status: ObservedDaemonStatus,
    ) -> Result<ObservedDaemonStatus, StoreError> {
        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        tx.execute(
            include_str!("../sql/observed_daemon_status/upsert.sql"),
            params![
                status.project_id,
                status.team_id,
                enum_to_text(&status.observed_state)?,
                status.source,
                status.checked_at.to_rfc3339(),
                status.details.to_string(),
            ],
        )?;
        tx.commit()?;
        Ok(status)
    }

    pub fn get_observed_daemon_status(
        &self,
        project_id: &str,
    ) -> Result<Option<ObservedDaemonStatus>, StoreError> {
        let conn = self.connection()?;
        conn.query_row(
            include_str!("../sql/observed_daemon_status/get.sql"),
            params![project_id],
            observed_daemon_status_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_observed_daemon_statuses(
        &self,
        team_id: Option<&str>,
    ) -> Result<Vec<ObservedDaemonStatus>, StoreError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(include_str!("../sql/observed_daemon_status/list.sql"))?;
        let rows = stmt.query_map(params![team_id], observed_daemon_status_from_row)?;
        collect_rows(rows)
    }

    pub fn fleet_daemon_statuses(
        &self,
        team_id: Option<&str>,
    ) -> Result<Vec<FleetDaemonStatus>, StoreError> {
        let evaluated_at = Utc::now();
        let teams = self
            .list_teams()?
            .into_iter()
            .filter(|team| team_id.map_or(true, |value| team.id == value))
            .map(|team| (team.id.clone(), team))
            .collect::<BTreeMap<_, _>>();
        let projects = self.list_projects(team_id)?;
        let schedules = self.list_schedules(team_id)?;
        let overrides = self
            .list_daemon_overrides()?
            .into_iter()
            .filter(|override_record| {
                team_id.map_or(true, |value| override_record.team_id == value)
                    && override_record.is_active(evaluated_at)
            })
            .collect::<Vec<_>>();
        let observed_statuses = self.list_observed_daemon_statuses(team_id)?;

        let desired_by_team =
            desired_state_by_team(schedules, overrides, evaluated_at, BTreeMap::new());
        let observed_by_project = observed_statuses
            .into_iter()
            .map(|status| (status.project_id.clone(), status))
            .collect::<BTreeMap<_, _>>();

        let mut rows = Vec::with_capacity(projects.len());
        for project in projects {
            let observed_status = observed_by_project.get(&project.id);
            rows.push(FleetDaemonStatus {
                team_id: project.team_id.clone(),
                team_slug: teams
                    .get(&project.team_id)
                    .map(|team| team.slug.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                project_id: project.id.clone(),
                project_slug: project.slug.clone(),
                project_root: project.ao_project_root.clone(),
                desired_state: if project.enabled {
                    desired_by_team
                        .get(&project.team_id)
                        .map(|evaluation| evaluation.desired_state)
                        .unwrap_or(DaemonDesiredState::Stopped)
                } else {
                    DaemonDesiredState::Stopped
                },
                observed_state: observed_status.map(|status| status.observed_state),
                checked_at: observed_status.map(|status| status.checked_at),
                source: observed_status.map(|status| status.source.clone()),
                details: observed_status.map(|status| status.details.clone()),
            });
        }

        rows.sort_by(|left, right| {
            left.team_slug.cmp(&right.team_slug).then(left.project_slug.cmp(&right.project_slug))
        });
        Ok(rows)
    }

    pub fn upsert_knowledge_source(
        &self,
        mut source: KnowledgeSource,
    ) -> Result<KnowledgeSource, StoreError> {
        validate_knowledge_source(&source)?;
        if source.id.trim().is_empty() {
            source.id = new_id("knowledge_source");
        }

        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let scope = enum_to_text(&source.scope)?;
        let kind = enum_to_text(&source.kind)?;
        let sync_state = enum_to_text(&source.sync_state)?;

        tx.execute(
            include_str!("../sql/knowledge_source/upsert.sql"),
            params![
                source.id,
                kind,
                source.label,
                source.uri,
                scope,
                source.scope_ref,
                sync_state,
                source.last_synced_at.map(|value| value.to_rfc3339()),
                source.metadata.to_string(),
                source.created_at.to_rfc3339(),
                source.updated_at.to_rfc3339(),
            ],
        )?;

        record_audit_event(
            &tx,
            NewAuditEvent {
                team_id: knowledge_team_id(&source.scope, source.scope_ref.as_deref()),
                entity_type: "knowledge_source".to_string(),
                entity_id: source.id.clone(),
                action: "upserted".to_string(),
                actor_type: "system".to_string(),
                actor_id: None,
                summary: format!("Upserted knowledge source {}", source.label),
                details: serde_json::json!({
                    "kind": enum_to_text(&source.kind)?,
                    "scope": enum_to_text(&source.scope)?,
                    "scope_ref": source.scope_ref,
                    "sync_state": enum_to_text(&source.sync_state)?,
                }),
            },
        )?;
        tx.commit()?;

        Ok(source)
    }

    pub fn get_knowledge_source(&self, id: &str) -> Result<Option<KnowledgeSource>, StoreError> {
        let conn = self.connection()?;
        conn.query_row(
            include_str!("../sql/knowledge_source/get.sql"),
            params![id],
            knowledge_source_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_knowledge_sources(
        &self,
        query: KnowledgeRecordQuery,
    ) -> Result<Vec<KnowledgeSource>, StoreError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(include_str!("../sql/knowledge_source/list.sql"))?;
        let scope = query.scope.as_ref().map(enum_to_text).transpose()?;
        let rows = stmt.query_map(
            params![scope, query.scope_ref, query.limit as i64],
            knowledge_source_from_row,
        )?;
        collect_rows(rows)
    }

    pub fn create_knowledge_document(
        &self,
        mut document: KnowledgeDocument,
    ) -> Result<KnowledgeDocument, StoreError> {
        validate_knowledge_document(&document)?;
        if document.id.trim().is_empty() {
            document.id = new_id("knowledge_document");
        }

        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let scope = enum_to_text(&document.scope)?;
        let kind = enum_to_text(&document.kind)?;
        let source_kind = document.source_kind.as_ref().map(enum_to_text).transpose()?;

        let result = tx.execute(
            include_str!("../sql/knowledge_document/insert.sql"),
            params![
                document.id,
                scope,
                document.scope_ref,
                kind,
                document.title,
                document.summary,
                document.body,
                document.source_id,
                source_kind,
                serde_json::to_string(&document.tags)?,
                document.created_at.to_rfc3339(),
                document.updated_at.to_rfc3339(),
            ],
        );

        match result {
            Ok(_) => {
                record_audit_event(
                    &tx,
                    NewAuditEvent {
                        team_id: knowledge_team_id(&document.scope, document.scope_ref.as_deref()),
                        entity_type: "knowledge_document".to_string(),
                        entity_id: document.id.clone(),
                        action: "created".to_string(),
                        actor_type: "system".to_string(),
                        actor_id: None,
                        summary: format!("Created knowledge document {}", document.title),
                        details: serde_json::json!({
                            "kind": enum_to_text(&document.kind)?,
                            "scope": enum_to_text(&document.scope)?,
                            "scope_ref": document.scope_ref,
                            "source_id": document.source_id,
                        }),
                    },
                )?;
                tx.commit()?;
                Ok(document)
            }
            Err(error) if is_unique_constraint(&error) => {
                Err(StoreError::already_exists("knowledge_document", document.id))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_knowledge_document(
        &self,
        id: &str,
    ) -> Result<Option<KnowledgeDocument>, StoreError> {
        let conn = self.connection()?;
        conn.query_row(
            include_str!("../sql/knowledge_document/get.sql"),
            params![id],
            knowledge_document_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_knowledge_documents(
        &self,
        query: KnowledgeRecordQuery,
    ) -> Result<Vec<KnowledgeDocument>, StoreError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(include_str!("../sql/knowledge_document/list.sql"))?;
        let scope = query.scope.as_ref().map(enum_to_text).transpose()?;
        let rows = stmt.query_map(
            params![scope, query.scope_ref, query.limit as i64],
            knowledge_document_from_row,
        )?;
        collect_rows(rows)
    }

    pub fn create_knowledge_fact(
        &self,
        mut fact: KnowledgeFact,
    ) -> Result<KnowledgeFact, StoreError> {
        validate_knowledge_fact(&fact)?;
        if fact.id.trim().is_empty() {
            fact.id = new_id("knowledge_fact");
        }

        let mut conn = self.connection()?;
        let tx = conn.transaction()?;
        let scope = enum_to_text(&fact.scope)?;
        let kind = enum_to_text(&fact.kind)?;
        let source_kind = fact.source_kind.as_ref().map(enum_to_text).transpose()?;

        let result = tx.execute(
            include_str!("../sql/knowledge_fact/insert.sql"),
            params![
                fact.id,
                scope,
                fact.scope_ref,
                kind,
                fact.statement,
                i64::from(fact.confidence),
                fact.source_id,
                source_kind,
                serde_json::to_string(&fact.tags)?,
                fact.observed_at.to_rfc3339(),
                fact.created_at.to_rfc3339(),
            ],
        );

        match result {
            Ok(_) => {
                record_audit_event(
                    &tx,
                    NewAuditEvent {
                        team_id: knowledge_team_id(&fact.scope, fact.scope_ref.as_deref()),
                        entity_type: "knowledge_fact".to_string(),
                        entity_id: fact.id.clone(),
                        action: "created".to_string(),
                        actor_type: "system".to_string(),
                        actor_id: None,
                        summary: format!("Created knowledge fact {}", fact.id),
                        details: serde_json::json!({
                            "kind": enum_to_text(&fact.kind)?,
                            "scope": enum_to_text(&fact.scope)?,
                            "scope_ref": fact.scope_ref,
                            "confidence": fact.confidence,
                            "source_id": fact.source_id,
                        }),
                    },
                )?;
                tx.commit()?;
                Ok(fact)
            }
            Err(error) if is_unique_constraint(&error) => {
                Err(StoreError::already_exists("knowledge_fact", fact.id))
            }
            Err(error) => Err(error.into()),
        }
    }

    pub fn get_knowledge_fact(&self, id: &str) -> Result<Option<KnowledgeFact>, StoreError> {
        let conn = self.connection()?;
        conn.query_row(
            include_str!("../sql/knowledge_fact/get.sql"),
            params![id],
            knowledge_fact_from_row,
        )
        .optional()
        .map_err(Into::into)
    }

    pub fn list_knowledge_facts(
        &self,
        query: KnowledgeRecordQuery,
    ) -> Result<Vec<KnowledgeFact>, StoreError> {
        let conn = self.connection()?;
        let mut stmt = conn.prepare(include_str!("../sql/knowledge_fact/list.sql"))?;
        let scope = query.scope.as_ref().map(enum_to_text).transpose()?;
        let rows = stmt.query_map(
            params![scope, query.scope_ref, query.limit as i64],
            knowledge_fact_from_row,
        )?;
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
            remote_url: input.remote_url,
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
                project.remote_url,
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
                            "remote_url": project.remote_url,
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

    pub fn get_project_by_ao_project_root(
        &self,
        ao_project_root: &str,
    ) -> Result<Option<Project>, StoreError> {
        let conn = self.connection()?;
        conn.query_row(
            include_str!("../sql/project/get_by_ao_project_root.sql"),
            params![ao_project_root],
            project_from_row,
        )
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
                project.remote_url,
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
                    "remote_url": project.remote_url,
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
        let placements = self.list_project_host_placements()?;
        let hosts = self.list_hosts()?;
        let daemon_statuses = self.fleet_daemon_statuses(team_filter)?;
        let overrides = self
            .list_daemon_overrides()?
            .into_iter()
            .filter(|override_record| {
                team_filter.map_or(true, |team_id| override_record.team_id == team_id)
                    && override_record.is_active(evaluated_at)
            })
            .collect::<Vec<_>>();

        let mut projects_by_team: BTreeMap<String, Vec<Project>> = BTreeMap::new();
        let mut project_team_by_id: BTreeMap<String, String> = BTreeMap::new();
        for project in projects {
            project_team_by_id.insert(project.id.clone(), project.team_id.clone());
            projects_by_team.entry(project.team_id.clone()).or_default().push(project);
        }

        let mut schedules_by_team: BTreeMap<String, Vec<Schedule>> = BTreeMap::new();
        for schedule in &schedules {
            schedules_by_team.entry(schedule.team_id.clone()).or_default().push(schedule.clone());
        }

        let mut placements_by_team: BTreeMap<String, Vec<ProjectHostPlacement>> = BTreeMap::new();
        for placement in placements {
            if let Some(team_id) = project_team_by_id.get(&placement.project_id) {
                placements_by_team.entry(team_id.clone()).or_default().push(placement);
            }
        }

        let host_by_id =
            hosts.iter().cloned().map(|host| (host.id.clone(), host)).collect::<BTreeMap<_, _>>();

        let mut hosts_by_team: BTreeMap<String, Vec<Host>> = BTreeMap::new();
        for (team_id, team_placements) in &placements_by_team {
            let mut team_hosts = Vec::new();
            let mut seen_host_ids = std::collections::BTreeSet::new();
            for placement in team_placements {
                if !seen_host_ids.insert(placement.host_id.clone()) {
                    continue;
                }
                if let Some(host) = host_by_id.get(&placement.host_id) {
                    team_hosts.push(host.clone());
                }
            }
            hosts_by_team.insert(team_id.clone(), team_hosts);
        }

        let mut daemon_statuses_by_team: BTreeMap<String, Vec<FleetDaemonStatus>> = BTreeMap::new();
        for status in daemon_statuses {
            daemon_statuses_by_team.entry(status.team_id.clone()).or_default().push(status);
        }

        let stored_observed_state_by_team = if query.observed_state_by_team.is_empty() {
            observed_state_by_team(self.list_observed_daemon_statuses(team_filter)?)
        } else {
            BTreeMap::new()
        };

        let mut team_overviews = Vec::with_capacity(teams.len());
        let mut preview_items = Vec::with_capacity(teams.len());
        let mut team_count = 0_usize;
        let mut project_count = 0_usize;
        let mut schedule_count = 0_usize;
        let mut enabled_project_count = 0_usize;
        let mut enabled_schedule_count = 0_usize;

        let desired_by_team = desired_state_by_team(
            schedules.clone(),
            overrides.clone(),
            evaluated_at,
            query.backlog_by_team.clone(),
        );
        let override_by_team = overrides
            .into_iter()
            .map(|override_record| (override_record.team_id.clone(), override_record))
            .collect::<BTreeMap<_, _>>();

        for team in teams {
            team_count += 1;
            let team_projects = projects_by_team.remove(&team.id).unwrap_or_default();
            let team_schedules = schedules_by_team.remove(&team.id).unwrap_or_default();
            let team_placements = placements_by_team.remove(&team.id).unwrap_or_default();
            let team_hosts = hosts_by_team.remove(&team.id).unwrap_or_default();
            let team_daemon_statuses = daemon_statuses_by_team.remove(&team.id).unwrap_or_default();
            let backlog_count = query.backlog_by_team.get(&team.id).copied().unwrap_or(0);
            let reconcile_evaluation =
                desired_by_team.get(&team.id).cloned().unwrap_or_else(|| {
                    TeamReconcileEvaluation::evaluate(
                        team.id.clone(),
                        &team_schedules,
                        None,
                        evaluated_at,
                        backlog_count,
                    )
                });
            let desired_state = reconcile_evaluation.desired_state;
            let observed_state = query
                .observed_state_by_team
                .get(&team.id)
                .copied()
                .or_else(|| stored_observed_state_by_team.get(&team.id).copied())
                .unwrap_or_else(|| infer_observed_state(&team_projects));

            let reconcile_preview = FleetReconcilePreviewItem {
                team_id: team.id.clone(),
                team_slug: team.slug.clone(),
                desired_state,
                observed_state,
                action: reconcile_action(desired_state, observed_state),
                backlog_count,
                schedule_ids: reconcile_evaluation.schedule_ids.clone(),
                reason: reconcile_evaluation.reason.clone(),
                override_applied: reconcile_evaluation.override_applied.clone(),
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

            let audit_events = self.list_audit_events(Some(&team.id), Some(8))?;
            let knowledge_documents = self.list_knowledge_documents(KnowledgeRecordQuery {
                scope: Some(KnowledgeScope::Team),
                scope_ref: Some(team.id.clone()),
                limit: 8,
            })?;
            let knowledge_facts = self.list_knowledge_facts(KnowledgeRecordQuery {
                scope: Some(KnowledgeScope::Team),
                scope_ref: Some(team.id.clone()),
                limit: 12,
            })?;

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
                placements: team_placements,
                hosts: team_hosts,
                daemon_statuses: team_daemon_statuses,
                audit_events,
                knowledge_documents,
                knowledge_facts,
                daemon_override: override_by_team.get(&reconcile_preview.team_id).cloned(),
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

    pub fn founder_overview(
        &self,
        query: FleetOverviewQuery,
    ) -> Result<FounderOverview, StoreError> {
        let fleet = self.fleet_overview(query.clone())?;
        let evaluated_at = fleet.evaluated_at;
        let team_filter = query.team_id.as_deref();
        let fleet_teams = fleet.teams.clone();
        let project_team_by_id = project_team_map(&fleet_teams);

        let hosts = self.list_hosts()?;
        let project_host_placements = self.list_project_host_placements()?;
        let daemon_statuses = self.fleet_daemon_statuses(team_filter)?;
        let audit_events =
            self.list_audit_events(team_filter, Some(FOUNDER_OVERVIEW_ACTIVITY_LIMIT))?;
        let knowledge_query = KnowledgeRecordQuery {
            scope: None,
            scope_ref: None,
            limit: FOUNDER_OVERVIEW_KNOWLEDGE_LIMIT,
        };
        let knowledge_documents = self.list_knowledge_documents(knowledge_query.clone())?;
        let knowledge_facts = self.list_knowledge_facts(knowledge_query)?;

        let project_host_placements = project_host_placements
            .into_iter()
            .filter(|placement| {
                project_team_by_id.get(&placement.project_id).map_or(false, |team_id| {
                    team_filter.map_or(true, |filter| filter == team_id.as_str())
                })
            })
            .collect::<Vec<_>>();
        let placement_count = project_host_placements.len();

        let host_ids = project_host_placements
            .iter()
            .map(|placement| placement.host_id.clone())
            .collect::<BTreeSet<_>>();
        let hosts = hosts
            .into_iter()
            .filter(|host| team_filter.is_none() || host_ids.contains(&host.id))
            .collect::<Vec<_>>();

        let knowledge_documents = knowledge_documents
            .into_iter()
            .filter(|document| {
                knowledge_record_team_id(
                    &document.scope,
                    document.scope_ref.as_deref(),
                    &project_team_by_id,
                )
                .map_or(team_filter.is_none(), |team_id| {
                    team_filter.map_or(true, |filter| filter == team_id.as_str())
                })
            })
            .collect::<Vec<_>>();
        let knowledge_facts = knowledge_facts
            .into_iter()
            .filter(|fact| {
                knowledge_record_team_id(
                    &fact.scope,
                    fact.scope_ref.as_deref(),
                    &project_team_by_id,
                )
                .map_or(team_filter.is_none(), |team_id| {
                    team_filter.map_or(true, |filter| filter == team_id.as_str())
                })
            })
            .collect::<Vec<_>>();

        let team_project_host_placements =
            group_project_host_placements_by_team(&project_host_placements, &project_team_by_id);
        let team_daemon_statuses = group_daemon_statuses_by_team(daemon_statuses.clone());
        let team_audit_events = group_audit_events_by_team(audit_events.clone());
        let team_knowledge_documents =
            group_knowledge_documents_by_team(knowledge_documents.clone(), &project_team_by_id);
        let team_knowledge_facts =
            group_knowledge_facts_by_team(knowledge_facts.clone(), &project_team_by_id);

        let teams = fleet_teams
            .into_iter()
            .filter(|team| team_filter.map_or(true, |filter| filter == team.team.id))
            .map(|team| {
                let team_id = team.team.id.clone();
                FounderTeamOverview {
                    fleet: team.clone(),
                    project_host_placements: team_project_host_placements
                        .get(&team_id)
                        .cloned()
                        .unwrap_or_default(),
                    daemon_statuses: team_daemon_statuses
                        .get(&team_id)
                        .cloned()
                        .unwrap_or_default(),
                    audit_events: team_audit_events.get(&team_id).cloned().unwrap_or_default(),
                    knowledge_documents: team_knowledge_documents
                        .get(&team_id)
                        .cloned()
                        .unwrap_or_default(),
                    knowledge_facts: team_knowledge_facts
                        .get(&team_id)
                        .cloned()
                        .unwrap_or_default(),
                }
            })
            .collect::<Vec<_>>();

        Ok(FounderOverview {
            evaluated_at,
            summary: FounderOverviewSummary {
                fleet: fleet.summary.clone(),
                host_count: hosts.len(),
                placement_count,
                daemon_status_count: daemon_statuses.len(),
                audit_event_count: audit_events.len(),
                knowledge_document_count: knowledge_documents.len(),
                knowledge_fact_count: knowledge_facts.len(),
            },
            fleet,
            hosts,
            project_host_placements,
            daemon_statuses,
            audit_events,
            knowledge_documents,
            knowledge_facts,
            teams,
        })
    }

    fn connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>, StoreError> {
        self.conn.lock().map_err(|_| StoreError::validation("database connection lock poisoned"))
    }

    fn run_migrations(&self) -> Result<(), StoreError> {
        let conn = self.connection()?;
        for (index, migration) in MIGRATION_SQL.iter().enumerate() {
            if let Err(error) = conn.execute_batch(migration) {
                if should_ignore_migration_error(index, &error) {
                    continue;
                }
                return Err(error.into());
            }
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

fn should_ignore_migration_error(index: usize, error: &rusqlite::Error) -> bool {
    const REMOTE_URL_MIGRATION_INDEX: usize = 6;

    index == REMOTE_URL_MIGRATION_INDEX
        && matches!(
            error,
            rusqlite::Error::SqliteFailure(_, Some(message))
                if message.contains("duplicate column name: remote_url")
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

fn parse_optional_datetime_sql(
    column: usize,
    value: Option<String>,
) -> rusqlite::Result<Option<DateTime<Utc>>> {
    value.map(|value| parse_datetime_sql(column, value)).transpose()
}

fn bool_from_i64(value: i64) -> bool {
    value != 0
}

fn enum_to_text<T: Serialize>(value: &T) -> Result<String, StoreError> {
    let json = serde_json::to_value(value)?;
    json.as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| StoreError::validation("enum did not serialize to a string"))
}

fn enum_from_text_sql<T: DeserializeOwned>(column: usize, value: String) -> rusqlite::Result<T> {
    serde_json::from_value(serde_json::Value::String(value)).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(error))
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

fn observed_state_by_team(
    statuses: Vec<ObservedDaemonStatus>,
) -> BTreeMap<String, DaemonDesiredState> {
    let mut states = BTreeMap::new();

    for status in statuses {
        states
            .entry(status.team_id)
            .and_modify(|current| *current = merge_desired_state(*current, status.observed_state))
            .or_insert(status.observed_state);
    }

    states
}

fn project_team_map(teams: &[FleetTeamOverview]) -> BTreeMap<String, String> {
    let mut project_team_by_id = BTreeMap::new();

    for team in teams {
        for project in &team.projects {
            project_team_by_id.insert(project.id.clone(), team.team.id.clone());
        }
    }

    project_team_by_id
}

fn group_project_host_placements_by_team(
    placements: &[ProjectHostPlacement],
    project_team_by_id: &BTreeMap<String, String>,
) -> BTreeMap<String, Vec<ProjectHostPlacement>> {
    let mut grouped: BTreeMap<String, Vec<ProjectHostPlacement>> = BTreeMap::new();

    for placement in placements {
        if let Some(team_id) = project_team_by_id.get(&placement.project_id) {
            grouped.entry(team_id.clone()).or_default().push(placement.clone());
        }
    }

    grouped
}

fn group_daemon_statuses_by_team(
    statuses: Vec<FleetDaemonStatus>,
) -> BTreeMap<String, Vec<FleetDaemonStatus>> {
    let mut grouped: BTreeMap<String, Vec<FleetDaemonStatus>> = BTreeMap::new();

    for status in statuses {
        grouped.entry(status.team_id.clone()).or_default().push(status);
    }

    grouped
}

fn group_audit_events_by_team(events: Vec<AuditEvent>) -> BTreeMap<String, Vec<AuditEvent>> {
    let mut grouped: BTreeMap<String, Vec<AuditEvent>> = BTreeMap::new();

    for event in events {
        if let Some(team_id) = event.team_id.clone() {
            grouped.entry(team_id).or_default().push(event);
        }
    }

    grouped
}

fn group_knowledge_documents_by_team(
    documents: Vec<KnowledgeDocument>,
    project_team_by_id: &BTreeMap<String, String>,
) -> BTreeMap<String, Vec<KnowledgeDocument>> {
    let mut grouped: BTreeMap<String, Vec<KnowledgeDocument>> = BTreeMap::new();

    for document in documents {
        if let Some(team_id) = knowledge_record_team_id(
            &document.scope,
            document.scope_ref.as_deref(),
            project_team_by_id,
        ) {
            grouped.entry(team_id).or_default().push(document);
        }
    }

    grouped
}

fn group_knowledge_facts_by_team(
    facts: Vec<KnowledgeFact>,
    project_team_by_id: &BTreeMap<String, String>,
) -> BTreeMap<String, Vec<KnowledgeFact>> {
    let mut grouped: BTreeMap<String, Vec<KnowledgeFact>> = BTreeMap::new();

    for fact in facts {
        if let Some(team_id) =
            knowledge_record_team_id(&fact.scope, fact.scope_ref.as_deref(), project_team_by_id)
        {
            grouped.entry(team_id).or_default().push(fact);
        }
    }

    grouped
}

fn knowledge_record_team_id(
    scope: &KnowledgeScope,
    scope_ref: Option<&str>,
    project_team_by_id: &BTreeMap<String, String>,
) -> Option<String> {
    match scope {
        KnowledgeScope::Team => scope_ref.map(ToOwned::to_owned),
        KnowledgeScope::Project => {
            scope_ref.and_then(|project_id| project_team_by_id.get(project_id).cloned())
        }
        KnowledgeScope::Global | KnowledgeScope::Operational => None,
    }
}

fn desired_state_by_team(
    schedules: Vec<Schedule>,
    overrides: Vec<DaemonOverride>,
    evaluated_at: DateTime<Utc>,
    backlog_by_team: BTreeMap<String, usize>,
) -> BTreeMap<String, TeamReconcileEvaluation> {
    let mut schedules_by_team: BTreeMap<String, Vec<Schedule>> = BTreeMap::new();
    for schedule in schedules {
        schedules_by_team.entry(schedule.team_id.clone()).or_default().push(schedule);
    }

    let mut overrides_by_team = BTreeMap::new();
    for override_record in overrides {
        overrides_by_team.insert(override_record.team_id.clone(), override_record);
    }

    let mut desired_by_team = BTreeMap::new();
    for (team_id, team_schedules) in schedules_by_team {
        let backlog_count = backlog_by_team.get(&team_id).copied().unwrap_or(0);
        let evaluation = TeamReconcileEvaluation::evaluate(
            team_id.clone(),
            &team_schedules,
            overrides_by_team.get(&team_id),
            evaluated_at,
            backlog_count,
        );
        desired_by_team.insert(team_id, evaluation);
    }

    for (team_id, override_record) in overrides_by_team {
        desired_by_team.entry(team_id.clone()).or_insert_with(|| {
            let backlog_count = backlog_by_team.get(&team_id).copied().unwrap_or(0);
            TeamReconcileEvaluation::evaluate(
                team_id,
                &[],
                Some(&override_record),
                evaluated_at,
                backlog_count,
            )
        });
    }

    desired_by_team
}

fn host_from_row(row: &Row<'_>) -> Result<Host, rusqlite::Error> {
    Ok(Host {
        id: row.get(0)?,
        slug: row.get(1)?,
        name: row.get(2)?,
        address: row.get(3)?,
        platform: row.get(4)?,
        status: row.get(5)?,
        capacity_slots: row.get(6)?,
        created_at: parse_datetime_sql(7, row.get::<_, String>(7)?)?,
        updated_at: parse_datetime_sql(8, row.get::<_, String>(8)?)?,
    })
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

fn observed_daemon_status_from_row(row: &Row<'_>) -> Result<ObservedDaemonStatus, rusqlite::Error> {
    let details_json: String = row.get(5)?;
    let details = serde_json::from_str(&details_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, Type::Text, Box::new(error))
    })?;

    Ok(ObservedDaemonStatus {
        project_id: row.get(0)?,
        team_id: row.get(1)?,
        observed_state: enum_from_text_sql(2, row.get::<_, String>(2)?)?,
        source: row.get(3)?,
        checked_at: parse_datetime_sql(4, row.get::<_, String>(4)?)?,
        details,
    })
}

fn daemon_override_from_row(row: &Row<'_>) -> Result<DaemonOverride, rusqlite::Error> {
    Ok(DaemonOverride {
        id: row.get(0)?,
        team_id: row.get(1)?,
        mode: override_mode_from_text_sql(row.get::<_, String>(2)?)?,
        forced_state: row
            .get::<_, Option<String>>(3)?
            .map(desired_state_from_text_sql)
            .transpose()?,
        pause_until: parse_optional_datetime_sql(4, row.get::<_, Option<String>>(4)?)?,
        note: row.get(5)?,
        source: row.get(6)?,
        created_at: parse_datetime_sql(7, row.get::<_, String>(7)?)?,
        updated_at: parse_datetime_sql(8, row.get::<_, String>(8)?)?,
    })
}

fn project_host_placement_from_row(row: &Row<'_>) -> Result<ProjectHostPlacement, rusqlite::Error> {
    Ok(ProjectHostPlacement {
        project_id: row.get(0)?,
        host_id: row.get(1)?,
        assignment_source: row.get(2)?,
        assigned_at: parse_datetime_sql(3, row.get::<_, String>(3)?)?,
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

fn override_mode_to_text(mode: DaemonOverrideMode) -> &'static str {
    match mode {
        DaemonOverrideMode::ForceDesiredState => "force_desired_state",
        DaemonOverrideMode::FreezeUntil => "freeze_until",
    }
}

fn override_mode_from_text_sql(value: String) -> rusqlite::Result<DaemonOverrideMode> {
    match value.as_str() {
        "force_desired_state" => Ok(DaemonOverrideMode::ForceDesiredState),
        "freeze_until" => Ok(DaemonOverrideMode::FreezeUntil),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            2,
            Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown daemon override mode: {other}"),
            )),
        )),
    }
}

fn desired_state_to_text(state: DaemonDesiredState) -> &'static str {
    match state {
        DaemonDesiredState::Running => "running",
        DaemonDesiredState::Paused => "paused",
        DaemonDesiredState::Stopped => "stopped",
    }
}

fn desired_state_from_text_sql(value: String) -> rusqlite::Result<DaemonDesiredState> {
    enum_from_text_sql(3, value)
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

fn validate_daemon_override(override_record: &DaemonOverride) -> Result<(), StoreError> {
    if override_record.id.trim().is_empty()
        || override_record.team_id.trim().is_empty()
        || override_record.source.trim().is_empty()
    {
        return Err(StoreError::validation("daemon override fields cannot be empty"));
    }

    Ok(())
}

fn validate_host(host: &Host) -> Result<(), StoreError> {
    if host.id.trim().is_empty()
        || host.slug.trim().is_empty()
        || host.name.trim().is_empty()
        || host.address.trim().is_empty()
        || host.platform.trim().is_empty()
        || host.status.trim().is_empty()
    {
        return Err(StoreError::validation("host fields cannot be empty"));
    }

    if host.capacity_slots < 0 {
        return Err(StoreError::validation("host capacity must be non-negative"));
    }

    Ok(())
}

fn validate_project_host_placement(placement: &ProjectHostPlacement) -> Result<(), StoreError> {
    if placement.project_id.trim().is_empty()
        || placement.host_id.trim().is_empty()
        || placement.assignment_source.trim().is_empty()
    {
        return Err(StoreError::validation("project host placement fields cannot be empty"));
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

fn validate_knowledge_source(source: &KnowledgeSource) -> Result<(), StoreError> {
    if source.label.trim().is_empty() {
        return Err(StoreError::validation("knowledge source label cannot be empty"));
    }

    validate_knowledge_scope(&source.scope, source.scope_ref.as_deref())?;

    Ok(())
}

fn validate_knowledge_document(document: &KnowledgeDocument) -> Result<(), StoreError> {
    if document.title.trim().is_empty()
        || document.summary.trim().is_empty()
        || document.body.trim().is_empty()
    {
        return Err(StoreError::validation("knowledge document fields cannot be empty"));
    }

    validate_knowledge_scope(&document.scope, document.scope_ref.as_deref())?;

    Ok(())
}

fn validate_knowledge_fact(fact: &KnowledgeFact) -> Result<(), StoreError> {
    if fact.statement.trim().is_empty() {
        return Err(StoreError::validation("knowledge fact statement cannot be empty"));
    }

    if fact.confidence > 100 {
        return Err(StoreError::validation("knowledge fact confidence must be between 0 and 100"));
    }

    validate_knowledge_scope(&fact.scope, fact.scope_ref.as_deref())?;

    Ok(())
}

fn validate_knowledge_scope(
    scope: &KnowledgeScope,
    scope_ref: Option<&str>,
) -> Result<(), StoreError> {
    match scope {
        KnowledgeScope::Global => {
            if scope_ref.is_some() {
                return Err(StoreError::validation(
                    "global knowledge records must not have a scope_ref",
                ));
            }
        }
        KnowledgeScope::Team | KnowledgeScope::Project => {
            let Some(scope_ref) = scope_ref else {
                return Err(StoreError::validation(
                    "team and project knowledge records require a scope_ref",
                ));
            };
            if scope_ref.trim().is_empty() {
                return Err(StoreError::validation("knowledge scope_ref cannot be empty"));
            }
        }
        KnowledgeScope::Operational => {}
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
        remote_url: row.get(6)?,
        enabled: bool_from_i64(row.get(7)?),
        created_at: parse_datetime_sql(8, row.get::<_, String>(8)?)?,
        updated_at: parse_datetime_sql(9, row.get::<_, String>(9)?)?,
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

fn knowledge_source_from_row(row: &Row<'_>) -> Result<KnowledgeSource, rusqlite::Error> {
    let metadata_json: String = row.get(8)?;
    let metadata: serde_json::Value = serde_json::from_str(&metadata_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(8, Type::Text, Box::new(error))
    })?;

    Ok(KnowledgeSource {
        id: row.get(0)?,
        kind: enum_from_text_sql(1, row.get::<_, String>(1)?)?,
        label: row.get(2)?,
        uri: row.get(3)?,
        scope: enum_from_text_sql(4, row.get::<_, String>(4)?)?,
        scope_ref: row.get(5)?,
        sync_state: enum_from_text_sql(6, row.get::<_, String>(6)?)?,
        last_synced_at: parse_optional_datetime_sql(7, row.get::<_, Option<String>>(7)?)?,
        metadata,
        created_at: parse_datetime_sql(9, row.get::<_, String>(9)?)?,
        updated_at: parse_datetime_sql(10, row.get::<_, String>(10)?)?,
    })
}

fn knowledge_document_from_row(row: &Row<'_>) -> Result<KnowledgeDocument, rusqlite::Error> {
    let tags_json: String = row.get(9)?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(9, Type::Text, Box::new(error))
    })?;

    Ok(KnowledgeDocument {
        id: row.get(0)?,
        scope: enum_from_text_sql(1, row.get::<_, String>(1)?)?,
        scope_ref: row.get(2)?,
        kind: enum_from_text_sql(3, row.get::<_, String>(3)?)?,
        title: row.get(4)?,
        summary: row.get(5)?,
        body: row.get(6)?,
        source_id: row.get(7)?,
        source_kind: row
            .get::<_, Option<String>>(8)?
            .map(|value| enum_from_text_sql(8, value))
            .transpose()?,
        tags,
        created_at: parse_datetime_sql(10, row.get::<_, String>(10)?)?,
        updated_at: parse_datetime_sql(11, row.get::<_, String>(11)?)?,
    })
}

fn knowledge_fact_from_row(row: &Row<'_>) -> Result<KnowledgeFact, rusqlite::Error> {
    let tags_json: String = row.get(8)?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(8, Type::Text, Box::new(error))
    })?;

    Ok(KnowledgeFact {
        id: row.get(0)?,
        scope: enum_from_text_sql(1, row.get::<_, String>(1)?)?,
        scope_ref: row.get(2)?,
        kind: enum_from_text_sql(3, row.get::<_, String>(3)?)?,
        statement: row.get(4)?,
        confidence: row.get(5)?,
        source_id: row.get(6)?,
        source_kind: row
            .get::<_, Option<String>>(7)?
            .map(|value| enum_from_text_sql(7, value))
            .transpose()?,
        tags,
        observed_at: parse_datetime_sql(9, row.get::<_, String>(9)?)?,
        created_at: parse_datetime_sql(10, row.get::<_, String>(10)?)?,
    })
}

fn knowledge_team_id(scope: &KnowledgeScope, scope_ref: Option<&str>) -> Option<String> {
    match scope {
        KnowledgeScope::Team => scope_ref.map(ToOwned::to_owned),
        _ => None,
    }
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
        DaemonDesiredState, KnowledgeDocument, KnowledgeDocumentKind, KnowledgeFact,
        KnowledgeFactKind, KnowledgeScope, KnowledgeSource, KnowledgeSourceKind,
        KnowledgeSyncState, NewHost, NewProject, NewSchedule, NewTeam, ObservedDaemonStatus,
        ProjectHostPlacement, SchedulePolicyKind, WeekdayWindow,
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
                remote_url: None,
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
                remote_url: None,
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
                remote_url: None,
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

    #[test]
    fn founder_overview_aggregates_company_surface() {
        let store = FleetStore::open_in_memory().expect("store opens");

        let team = store
            .create_team(NewTeam {
                slug: "platform".to_string(),
                name: "Platform".to_string(),
                mission: "Own company systems".to_string(),
                ownership: "engineering".to_string(),
                business_priority: 9,
            })
            .expect("team created");

        let project = store
            .create_project(NewProject {
                team_id: team.id.clone(),
                slug: "ao-fleet".to_string(),
                root_path: "/tmp/ao-fleet".to_string(),
                ao_project_root: "/tmp/ao-fleet".to_string(),
                default_branch: "main".to_string(),
                remote_url: Some("https://example.com/ao-fleet.git".to_string()),
                enabled: true,
            })
            .expect("project created");

        store
            .create_schedule(NewSchedule {
                team_id: team.id.clone(),
                timezone: "UTC".to_string(),
                policy_kind: SchedulePolicyKind::AlwaysOn,
                windows: vec![],
                enabled: true,
            })
            .expect("schedule created");

        let host = store
            .create_host(NewHost {
                slug: "local".to_string(),
                name: "Local Host".to_string(),
                address: "http://localhost:3000".to_string(),
                platform: "linux".to_string(),
                status: "healthy".to_string(),
                capacity_slots: 8,
            })
            .expect("host created");

        store
            .upsert_project_host_placement(ProjectHostPlacement {
                project_id: project.id.clone(),
                host_id: host.id.clone(),
                assignment_source: "manual".to_string(),
                assigned_at: Utc::now(),
            })
            .expect("placement stored");

        store
            .upsert_observed_daemon_status(ObservedDaemonStatus {
                project_id: project.id.clone(),
                team_id: team.id.clone(),
                observed_state: DaemonDesiredState::Running,
                source: "test".to_string(),
                checked_at: Utc::now(),
                details: json!({"state": "running"}),
            })
            .expect("daemon status stored");

        let source = store
            .upsert_knowledge_source(KnowledgeSource {
                id: String::new(),
                kind: KnowledgeSourceKind::ManualNote,
                label: "Operator note".to_string(),
                uri: Some("file:///tmp/operator-note.md".to_string()),
                scope: KnowledgeScope::Team,
                scope_ref: Some(team.id.clone()),
                sync_state: KnowledgeSyncState::Ready,
                last_synced_at: Some(Utc::now()),
                metadata: json!({"author": "ops"}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .expect("knowledge source created");

        store
            .create_knowledge_document(KnowledgeDocument {
                id: String::new(),
                scope: KnowledgeScope::Team,
                scope_ref: Some(team.id.clone()),
                kind: KnowledgeDocumentKind::Runbook,
                title: "Founders runbook".to_string(),
                summary: "How to operate the company".to_string(),
                body: "Keep the fleet stable.".to_string(),
                source_id: Some(source.id.clone()),
                source_kind: Some(KnowledgeSourceKind::ManualNote),
                tags: vec!["ops".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .expect("knowledge document created");

        store
            .create_knowledge_fact(KnowledgeFact {
                id: String::new(),
                scope: KnowledgeScope::Team,
                scope_ref: Some(team.id.clone()),
                kind: KnowledgeFactKind::Policy,
                statement: "Platform owns the fleet control plane".to_string(),
                confidence: 99,
                source_id: Some(source.id.clone()),
                source_kind: Some(KnowledgeSourceKind::ManualNote),
                tags: vec!["policy".to_string()],
                observed_at: Utc::now(),
                created_at: Utc::now(),
            })
            .expect("knowledge fact created");

        let overview = store
            .founder_overview(FleetOverviewQuery {
                team_id: None,
                at: Some(Utc.with_ymd_and_hms(2025, 3, 3, 10, 0, 0).unwrap()),
                backlog_by_team: std::collections::BTreeMap::new(),
                observed_state_by_team: std::collections::BTreeMap::new(),
            })
            .expect("founder overview built");

        assert_eq!(overview.summary.fleet.team_count, 1);
        assert_eq!(overview.summary.fleet.project_count, 1);
        assert_eq!(overview.summary.fleet.schedule_count, 1);
        assert_eq!(overview.summary.fleet.enabled_project_count, 1);
        assert_eq!(overview.summary.fleet.enabled_schedule_count, 1);
        assert_eq!(overview.summary.host_count, 1);
        assert_eq!(overview.summary.placement_count, 1);
        assert_eq!(overview.summary.daemon_status_count, 1);
        assert_eq!(overview.summary.audit_event_count, 8);
        assert_eq!(overview.summary.knowledge_document_count, 1);
        assert_eq!(overview.summary.knowledge_fact_count, 1);
        assert_eq!(overview.hosts.len(), 1);
        assert_eq!(overview.project_host_placements.len(), 1);
        assert_eq!(overview.daemon_statuses.len(), 1);
        assert_eq!(overview.audit_events.len(), 8);
        assert_eq!(overview.knowledge_documents.len(), 1);
        assert_eq!(overview.knowledge_facts.len(), 1);
        assert_eq!(overview.teams.len(), 1);

        let founder_team = &overview.teams[0];
        assert_eq!(founder_team.fleet.team.id, team.id);
        assert_eq!(founder_team.project_host_placements.len(), 1);
        assert_eq!(founder_team.daemon_statuses.len(), 1);
        assert_eq!(founder_team.audit_events.len(), 6);
        assert_eq!(founder_team.knowledge_documents.len(), 1);
        assert_eq!(founder_team.knowledge_facts.len(), 1);
    }

    #[test]
    fn knowledge_records_round_trip() {
        let store = FleetStore::open_in_memory().expect("store opens");

        let team = store
            .create_team(NewTeam {
                slug: "platform".to_string(),
                name: "Platform".to_string(),
                mission: "Own company systems".to_string(),
                ownership: "engineering".to_string(),
                business_priority: 9,
            })
            .expect("team created");

        let source = store
            .upsert_knowledge_source(KnowledgeSource {
                id: String::new(),
                kind: KnowledgeSourceKind::ManualNote,
                label: "Operator note".to_string(),
                uri: Some("file:///tmp/operator-note.md".to_string()),
                scope: KnowledgeScope::Team,
                scope_ref: Some(team.id.clone()),
                sync_state: KnowledgeSyncState::Ready,
                last_synced_at: Some(Utc::now()),
                metadata: json!({"author": "ops"}),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .expect("source upserted");

        let document = store
            .create_knowledge_document(KnowledgeDocument {
                id: String::new(),
                scope: KnowledgeScope::Team,
                scope_ref: Some(team.id.clone()),
                kind: KnowledgeDocumentKind::Runbook,
                title: "On-call runbook".to_string(),
                summary: "How to restart the fleet".to_string(),
                body: "Steps for handling fleet incidents".to_string(),
                source_id: Some(source.id.clone()),
                source_kind: Some(KnowledgeSourceKind::ManualNote),
                tags: vec!["ops".to_string(), "runbook".to_string()],
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
            .expect("document created");

        let fact = store
            .create_knowledge_fact(KnowledgeFact {
                id: String::new(),
                scope: KnowledgeScope::Team,
                scope_ref: Some(team.id.clone()),
                kind: KnowledgeFactKind::Policy,
                statement: "Platform owns production daemon policy".to_string(),
                confidence: 95,
                source_id: Some(source.id.clone()),
                source_kind: Some(KnowledgeSourceKind::ManualNote),
                tags: vec!["policy".to_string()],
                observed_at: Utc::now(),
                created_at: Utc::now(),
            })
            .expect("fact created");

        let query = KnowledgeRecordQuery {
            scope: Some(KnowledgeScope::Team),
            scope_ref: Some(team.id.clone()),
            limit: 10,
        };

        let sources = store.list_knowledge_sources(query.clone()).expect("sources listed");
        let documents = store.list_knowledge_documents(query.clone()).expect("documents listed");
        let facts = store.list_knowledge_facts(query).expect("facts listed");

        assert_eq!(sources.len(), 1);
        assert_eq!(documents.len(), 1);
        assert_eq!(facts.len(), 1);
        assert_eq!(sources[0].sync_state, KnowledgeSyncState::Ready);
        assert_eq!(documents[0].source_id.as_deref(), Some(source.id.as_str()));
        assert_eq!(facts[0].source_id.as_deref(), Some(source.id.as_str()));
        assert_eq!(
            store
                .get_knowledge_document(&document.id)
                .expect("document fetched")
                .expect("document exists")
                .title,
            "On-call runbook"
        );
        assert_eq!(
            store
                .get_knowledge_fact(&fact.id)
                .expect("fact fetched")
                .expect("fact exists")
                .confidence,
            95
        );

        assert_eq!(
            store.list_audit_events(Some(&team.id), None).expect("audit events listed").len(),
            4
        );
    }
}
