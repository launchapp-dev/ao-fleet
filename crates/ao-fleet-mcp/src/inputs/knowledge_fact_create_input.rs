use ao_fleet_core::{KnowledgeFactKind, KnowledgeScope, KnowledgeSourceKind};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeFactCreateInput {
    pub id: Option<String>,
    pub scope: KnowledgeScope,
    pub scope_ref: Option<String>,
    pub kind: KnowledgeFactKind,
    pub statement: String,
    pub confidence: u8,
    pub source_id: Option<String>,
    pub source_kind: Option<KnowledgeSourceKind>,
    pub tags: Vec<String>,
    pub observed_at: Option<DateTime<Utc>>,
}
