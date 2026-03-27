use ao_fleet_core::SchedulePolicyKind;
use serde::{Deserialize, Serialize};

use crate::inputs::schedule_window_input::ScheduleWindowInput;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleCreateInput {
    pub team_id: String,
    pub timezone: String,
    pub policy_kind: SchedulePolicyKind,
    pub windows: Vec<ScheduleWindowInput>,
    pub enabled: bool,
}
