use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::daemon_desired_state::DaemonDesiredState;
use crate::models::daemon_override_mode::DaemonOverrideMode;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaemonOverride {
    pub id: String,
    pub team_id: String,
    pub mode: DaemonOverrideMode,
    pub forced_state: Option<DaemonDesiredState>,
    pub pause_until: Option<DateTime<Utc>>,
    pub note: Option<String>,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DaemonOverride {
    pub fn is_active(&self, at: DateTime<Utc>) -> bool {
        match self.mode {
            DaemonOverrideMode::ForceDesiredState => true,
            DaemonOverrideMode::FreezeUntil => {
                self.pause_until.map(|pause_until| pause_until > at).unwrap_or(false)
            }
        }
    }
}
