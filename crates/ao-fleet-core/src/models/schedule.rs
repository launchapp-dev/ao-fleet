use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::SchedulePolicyKind;
use crate::WeekdayWindow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schedule {
    pub id: String,
    pub team_id: String,
    pub timezone: String,
    pub policy_kind: SchedulePolicyKind,
    pub windows: Vec<WeekdayWindow>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
