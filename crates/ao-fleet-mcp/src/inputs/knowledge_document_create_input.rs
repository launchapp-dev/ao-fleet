use ao_fleet_core::{KnowledgeDocumentKind, KnowledgeScope, KnowledgeSourceKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeDocumentCreateInput {
    pub id: Option<String>,
    pub scope: KnowledgeScope,
    pub scope_ref: Option<String>,
    pub kind: KnowledgeDocumentKind,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub source_id: Option<String>,
    pub source_kind: Option<KnowledgeSourceKind>,
    pub tags: Vec<String>,
}
