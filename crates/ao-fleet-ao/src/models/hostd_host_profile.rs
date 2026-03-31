use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostdHostProfile {
    pub slug: String,
    pub name: String,
    pub address: String,
    pub platform: String,
    pub status: String,
    pub capacity_slots: i32,
    pub fleet_url: Option<String>,
    pub project_count: usize,
}
