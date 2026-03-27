use std::collections::BTreeMap;

use ao_fleet_core::DaemonDesiredState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct FleetOverviewQuery {
    pub team_id: Option<String>,
    pub at: Option<DateTime<Utc>>,
    pub backlog_by_team: BTreeMap<String, usize>,
    pub observed_state_by_team: BTreeMap<String, DaemonDesiredState>,
}
