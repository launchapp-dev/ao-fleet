use anyhow::{Result, anyhow};

use ao_fleet_core::KnowledgeScope;
use ao_fleet_store::{FleetStore, KnowledgeRecordQuery};

use crate::cli::handlers::json_printer::print_json;
use crate::cli::handlers::knowledge_source_list_command::KnowledgeSourceListCommand;

pub fn knowledge_source_list(db_path: &str, command: KnowledgeSourceListCommand) -> Result<()> {
    let store = FleetStore::open(db_path)?;
    let sources = store.list_knowledge_sources(KnowledgeRecordQuery {
        scope: command.scope.as_deref().map(parse_scope).transpose()?,
        scope_ref: command.scope_ref,
        limit: command.limit,
    })?;
    print_json(&sources)
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
