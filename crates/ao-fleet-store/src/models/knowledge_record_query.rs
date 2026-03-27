use ao_fleet_core::KnowledgeScope;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KnowledgeRecordQuery {
    pub scope: Option<KnowledgeScope>,
    pub scope_ref: Option<String>,
    pub limit: usize,
}

impl Default for KnowledgeRecordQuery {
    fn default() -> Self {
        Self { scope: None, scope_ref: None, limit: 100 }
    }
}
