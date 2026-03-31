use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostdLogEntry {
    pub seq: Option<u64>,
    pub ts: DateTime<Utc>,
    pub host_slug: String,
    pub project_id: Option<String>,
    pub stream: String,
    pub level: String,
    pub message: String,
    pub details: serde_json::Value,
}
