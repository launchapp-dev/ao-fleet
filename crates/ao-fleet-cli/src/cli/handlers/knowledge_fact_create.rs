use anyhow::{Context, Result, anyhow};

use ao_fleet_core::{KnowledgeFact, KnowledgeFactKind, KnowledgeScope, KnowledgeSourceKind};
use ao_fleet_store::FleetStore;
use chrono::{DateTime, Utc};

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::knowledge_fact_create_command::KnowledgeFactCreateCommand;

pub fn knowledge_fact_create(db_path: &str, command: KnowledgeFactCreateCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let now = Utc::now();
    let fact = store.create_knowledge_fact(KnowledgeFact {
        id: command.id.unwrap_or_default(),
        scope: parse_scope(&command.scope)?,
        scope_ref: command.scope_ref,
        kind: parse_fact_kind(&command.kind)?,
        statement: command.statement,
        confidence: command.confidence,
        source_id: command.source_id,
        source_kind: command.source_kind.as_deref().map(parse_source_kind).transpose()?,
        tags: command.tags,
        observed_at: parse_optional_datetime(command.observed_at)?.unwrap_or(now),
        created_at: now,
    })?;
    print_json(&fact)
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

fn parse_fact_kind(value: &str) -> Result<KnowledgeFactKind> {
    match value {
        "policy" => Ok(KnowledgeFactKind::Policy),
        "decision" => Ok(KnowledgeFactKind::Decision),
        "risk" => Ok(KnowledgeFactKind::Risk),
        "incident" => Ok(KnowledgeFactKind::Incident),
        "workflow_outcome" => Ok(KnowledgeFactKind::WorkflowOutcome),
        "schedule_observation" => Ok(KnowledgeFactKind::ScheduleObservation),
        other => Err(anyhow!("unsupported knowledge fact kind '{other}'")),
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

fn parse_optional_datetime(value: Option<String>) -> Result<Option<DateTime<Utc>>> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|parsed| parsed.with_timezone(&Utc))
                .with_context(|| format!("invalid RFC 3339 timestamp '{value}'"))
        })
        .transpose()
}
