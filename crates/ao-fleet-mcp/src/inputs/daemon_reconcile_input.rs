use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonReconcileInput {
    pub at: Option<DateTime<Utc>>,
    pub backlog_by_team: HashMap<String, usize>,
    pub apply: bool,
}
