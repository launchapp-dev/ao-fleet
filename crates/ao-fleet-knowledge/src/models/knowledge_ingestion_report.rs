use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeIngestionReport {
    pub job_id: String,
    pub manifest_id: String,
    pub source_id: String,
    pub documents_created: u32,
    pub documents_updated: u32,
    pub facts_created: u32,
    pub facts_updated: u32,
    pub notes: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
}
