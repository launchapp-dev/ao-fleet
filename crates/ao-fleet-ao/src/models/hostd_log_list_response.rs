use serde::{Deserialize, Serialize};

use crate::models::hostd_log_entry::HostdLogEntry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostdLogListResponse {
    pub entries: Vec<HostdLogEntry>,
    pub next_seq: Option<u64>,
}
