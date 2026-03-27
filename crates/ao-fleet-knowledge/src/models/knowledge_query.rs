use serde::{Deserialize, Serialize};

use ao_fleet_core::{
    KnowledgeDocumentKind, KnowledgeFactKind, KnowledgeScope, KnowledgeSourceKind,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeQuery {
    pub scope: Option<KnowledgeScope>,
    pub document_kinds: Vec<KnowledgeDocumentKind>,
    pub fact_kinds: Vec<KnowledgeFactKind>,
    pub source_kinds: Vec<KnowledgeSourceKind>,
    pub tags: Vec<String>,
    pub text: Option<String>,
    pub limit: usize,
}

impl KnowledgeQuery {
    pub fn new() -> Self {
        Self {
            scope: None,
            document_kinds: Vec::new(),
            fact_kinds: Vec::new(),
            source_kinds: Vec::new(),
            tags: Vec::new(),
            text: None,
            limit: 50,
        }
    }
}

impl Default for KnowledgeQuery {
    fn default() -> Self {
        Self::new()
    }
}
