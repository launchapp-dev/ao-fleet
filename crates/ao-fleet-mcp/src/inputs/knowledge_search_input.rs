use ao_fleet_core::{
    KnowledgeDocumentKind, KnowledgeFactKind, KnowledgeScope, KnowledgeSourceKind,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct KnowledgeSearchInput {
    pub scope: Option<KnowledgeScope>,
    pub scope_ref: Option<String>,
    pub document_kinds: Vec<KnowledgeDocumentKind>,
    pub fact_kinds: Vec<KnowledgeFactKind>,
    pub source_kinds: Vec<KnowledgeSourceKind>,
    pub tags: Vec<String>,
    pub text: Option<String>,
    pub limit: usize,
}

impl Default for KnowledgeSearchInput {
    fn default() -> Self {
        Self {
            scope: None,
            scope_ref: None,
            document_kinds: Vec::new(),
            fact_kinds: Vec::new(),
            source_kinds: Vec::new(),
            tags: Vec::new(),
            text: None,
            limit: 50,
        }
    }
}
