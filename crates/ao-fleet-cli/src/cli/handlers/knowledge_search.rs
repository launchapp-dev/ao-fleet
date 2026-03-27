use anyhow::{Result, anyhow};

use ao_fleet_core::{
    KnowledgeDocumentKind, KnowledgeFactKind, KnowledgeScope, KnowledgeSourceKind,
};
use ao_fleet_knowledge::{KnowledgeQuery, KnowledgeSearchService};
use ao_fleet_store::{FleetStore, KnowledgeRecordQuery};

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::knowledge_search_command::KnowledgeSearchCommand;

pub fn knowledge_search(db_path: &str, command: KnowledgeSearchCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let query = KnowledgeQuery {
        scope: command.scope.as_deref().map(parse_scope).transpose()?,
        scope_ref: command.scope_ref.clone(),
        document_kinds: command
            .document_kinds
            .iter()
            .map(|value| parse_document_kind(value))
            .collect::<Result<Vec<_>>>()?,
        fact_kinds: command
            .fact_kinds
            .iter()
            .map(|value| parse_fact_kind(value))
            .collect::<Result<Vec<_>>>()?,
        source_kinds: command
            .source_kinds
            .iter()
            .map(|value| parse_source_kind(value))
            .collect::<Result<Vec<_>>>()?,
        tags: command.tags,
        text: command.text,
        limit: command.limit,
    };
    let record_query = KnowledgeRecordQuery {
        scope: query.scope.clone(),
        scope_ref: query.scope_ref.clone(),
        limit: query.limit,
    };
    let documents = store.list_knowledge_documents(record_query.clone())?;
    let facts = store.list_knowledge_facts(record_query)?;
    let result = KnowledgeSearchService::default().search(&query, &documents, &facts);

    print_json(&result)
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
