use anyhow::{Result, anyhow};

use ao_fleet_core::KnowledgeScope;
use ao_fleet_store::{FleetStore, KnowledgeRecordQuery};

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::knowledge_document_list_command::KnowledgeDocumentListCommand;

pub fn knowledge_document_list(db_path: &str, command: KnowledgeDocumentListCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let documents = store.list_knowledge_documents(KnowledgeRecordQuery {
        scope: command.scope.as_deref().map(parse_scope).transpose()?,
        scope_ref: command.scope_ref,
        limit: command.limit,
    })?;
    print_json(&documents)
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
