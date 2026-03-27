use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::knowledge_scope::KnowledgeScope;
use crate::models::knowledge_source_kind::KnowledgeSourceKind;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    pub kind: KnowledgeSourceKind,
    pub label: String,
    pub uri: Option<String>,
    pub scope: KnowledgeScope,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
