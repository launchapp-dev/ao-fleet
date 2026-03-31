use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostdWsHeartbeat {
    pub ts: DateTime<Utc>,
    pub host_slug: String,
}
