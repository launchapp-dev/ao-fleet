use anyhow::{Result, anyhow};

use ao_fleet_core::{
    KnowledgeDocument, KnowledgeDocumentKind, KnowledgeScope, KnowledgeSourceKind,
};
use ao_fleet_store::FleetStore;
use chrono::Utc;

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::knowledge_document_create_command::KnowledgeDocumentCreateCommand;

pub fn knowledge_document_create(
    db_path: &str,
    command: KnowledgeDocumentCreateCommand,
) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let now = Utc::now();
    let document = store.create_knowledge_document(KnowledgeDocument {
        id: command.id.unwrap_or_default(),
        scope: parse_scope(&command.scope)?,
        scope_ref: command.scope_ref,
        kind: parse_document_kind(&command.kind)?,
        title: command.title,
        summary: command.summary,
        body: command.body,
        source_id: command.source_id,
        source_kind: command.source_kind.as_deref().map(parse_source_kind).transpose()?,
        tags: command.tags,
        created_at: now,
        updated_at: now,
    })?;
    print_json(&document)
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

fn parse_document_kind(value: &str) -> Result<KnowledgeDocumentKind> {
    match value {
        "brief" => Ok(KnowledgeDocumentKind::Brief),
        "decision" => Ok(KnowledgeDocumentKind::Decision),
        "runbook" => Ok(KnowledgeDocumentKind::Runbook),
        "research_note" => Ok(KnowledgeDocumentKind::ResearchNote),
        "team_profile" => Ok(KnowledgeDocumentKind::TeamProfile),
        "project_profile" => Ok(KnowledgeDocumentKind::ProjectProfile),
        "incident_report" => Ok(KnowledgeDocumentKind::IncidentReport),
        "policy_note" => Ok(KnowledgeDocumentKind::PolicyNote),
        other => Err(anyhow!("unsupported knowledge document kind '{other}'")),
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
