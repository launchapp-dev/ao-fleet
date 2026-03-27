use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use ao_fleet_core::KnowledgeScope;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeCollection {
    pub id: String,
    pub manifest_id: String,
    pub scope: KnowledgeScope,
    pub name: String,
    pub description: String,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
