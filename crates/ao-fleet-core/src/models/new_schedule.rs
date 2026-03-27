use crate::{FleetError, SchedulePolicyKind, WeekdayWindow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewSchedule {
    pub team_id: String,
    pub timezone: String,
    pub policy_kind: SchedulePolicyKind,
    pub windows: Vec<WeekdayWindow>,
    pub enabled: bool,
}

impl NewSchedule {
    pub fn validate(&self) -> Result<(), FleetError> {
        if self.team_id.trim().is_empty() || self.timezone.trim().is_empty() {
            return Err(FleetError::Validation {
                message: "schedule team_id and timezone are required".to_string(),
            });
        }

        for window in &self.windows {
            window.validate()?;
        }

        Ok(())
    }
}
