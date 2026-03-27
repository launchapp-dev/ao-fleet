use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub team_id: Option<String>,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub actor_type: String,
    pub actor_id: Option<String>,
    pub summary: String,
    pub details: Value,
    pub occurred_at: DateTime<Utc>,
}
