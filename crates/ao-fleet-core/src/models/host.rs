use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Host {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub address: String,
    pub platform: String,
    pub status: String,
    pub capacity_slots: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
