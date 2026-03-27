use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScheduleListInput {
    pub team_id: Option<String>,
    pub enabled_only: bool,
}
