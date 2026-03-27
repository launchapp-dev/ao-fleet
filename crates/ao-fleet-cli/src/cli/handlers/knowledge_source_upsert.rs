use anyhow::{Context, Result, anyhow};

use ao_fleet_core::{KnowledgeScope, KnowledgeSource, KnowledgeSourceKind, KnowledgeSyncState};
use ao_fleet_store::FleetStore;
use chrono::{DateTime, Utc};

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::knowledge_source_upsert_command::KnowledgeSourceUpsertCommand;

pub fn knowledge_source_upsert(db_path: &str, command: KnowledgeSourceUpsertCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let now = Utc::now();
    let source = store.upsert_knowledge_source(KnowledgeSource {
        id: command.id.unwrap_or_default(),
        kind: parse_source_kind(&command.kind)?,
        label: command.label,
        uri: command.uri,
        scope: parse_scope(&command.scope)?,
        scope_ref: command.scope_ref,
        sync_state: parse_sync_state(&command.sync_state)?,
        last_synced_at: parse_optional_datetime(command.last_synced_at)?,
        metadata: serde_json::from_str(&command.metadata_json)
            .context("invalid --metadata-json payload")?,
        created_at: now,
        updated_at: now,
    })?;
    print_json(&source)
}

fn parse_scope(value: &str) -> Result<KnowledgeScope> {
    match value {
        "global" => Ok(KnowledgeScope::Global),
        "team" => Ok(KnowledgeScope::Team),
        "project" => Ok(KnowledgeScope::Project),
        "operational" => Ok(KnowledgeScope::Operational),
        other => Err(anyhow!("unsupported knowledge scope '{other}'")),
    }
}

fn parse_source_kind(value: &str) -> Result<KnowledgeSourceKind> {
    match value {
        "ao_event" => Ok(KnowledgeSourceKind::AoEvent),
        "git_commit" => Ok(KnowledgeSourceKind::GitCommit),
        "github_issue" => Ok(KnowledgeSourceKind::GitHubIssue),
        "github_pull_request" => Ok(KnowledgeSourceKind::GitHubPullRequest),
        "manual_note" => Ok(KnowledgeSourceKind::ManualNote),
        "incident" => Ok(KnowledgeSourceKind::Incident),
        "schedule_change" => Ok(KnowledgeSourceKind::ScheduleChange),
        "workflow_run" => Ok(KnowledgeSourceKind::WorkflowRun),
        other => Err(anyhow!("unsupported knowledge source kind '{other}'")),
    }
}

fn parse_sync_state(value: &str) -> Result<KnowledgeSyncState> {
    match value {
        "pending" => Ok(KnowledgeSyncState::Pending),
        "ready" => Ok(KnowledgeSyncState::Ready),
        "stale" => Ok(KnowledgeSyncState::Stale),
        "failed" => Ok(KnowledgeSyncState::Failed),
        other => Err(anyhow!("unsupported knowledge sync state '{other}'")),
    }
}

fn parse_optional_datetime(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|parsed| parsed.with_timezone(&Utc))
                .with_context(|| format!("invalid RFC 3339 timestamp '{value}'"))
        })
        .transpose()
}
