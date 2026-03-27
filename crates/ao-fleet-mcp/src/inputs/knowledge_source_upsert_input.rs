use ao_fleet_core::{KnowledgeScope, KnowledgeSourceKind, KnowledgeSyncState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeSourceUpsertInput {
    pub id: Option<String>,
    pub kind: KnowledgeSourceKind,
    pub label: String,
    pub uri: Option<String>,
    pub scope: KnowledgeScope,
    pub scope_ref: Option<String>,
    pub sync_state: KnowledgeSyncState,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}
