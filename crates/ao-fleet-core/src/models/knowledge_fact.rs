use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::knowledge_fact_kind::KnowledgeFactKind;
use crate::models::knowledge_scope::KnowledgeScope;
use crate::models::knowledge_source_kind::KnowledgeSourceKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeFact {
    pub id: String,
    pub scope: KnowledgeScope,
    pub scope_ref: Option<String>,
    pub kind: KnowledgeFactKind,
    pub statement: String,
    pub confidence: u8,
    pub source_id: Option<String>,
    pub source_kind: Option<KnowledgeSourceKind>,
    pub tags: Vec<String>,
    pub observed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
