use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::knowledge_document_kind::KnowledgeDocumentKind;
use crate::models::knowledge_scope::KnowledgeScope;
use crate::models::knowledge_source_kind::KnowledgeSourceKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeDocument {
    pub id: String,
    pub scope: KnowledgeScope,
    pub scope_ref: Option<String>,
    pub kind: KnowledgeDocumentKind,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub source_id: Option<String>,
    pub source_kind: Option<KnowledgeSourceKind>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
