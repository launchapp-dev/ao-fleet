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

        validate_windows_for_policy(self.policy_kind, &self.windows)?;

        for window in &self.windows {
            window.validate()?;
        }

        Ok(())
    }
}

fn validate_windows_for_policy(
    policy_kind: SchedulePolicyKind,
    windows: &[WeekdayWindow],
) -> Result<(), FleetError> {
    match policy_kind {
        SchedulePolicyKind::BusinessHours => {
            if windows.is_empty() {
                return Err(FleetError::Validation {
                    message: "business_hours schedules require at least one window".to_string(),
                });
            }

            if windows.iter().any(|window| window.weekdays.is_empty()) {
                return Err(FleetError::Validation {
                    message: "business_hours windows require at least one weekday".to_string(),
                });
            }

            if windows.iter().any(|window| window.start_hour > window.end_hour) {
                return Err(FleetError::Validation {
                    message: "business_hours windows cannot wrap past midnight".to_string(),
                });
            }
        }
        SchedulePolicyKind::Nightly => {
            if windows.is_empty() {
                return Err(FleetError::Validation {
                    message: "nightly schedules require at least one window".to_string(),
                });
            }
        }
        SchedulePolicyKind::AlwaysOn
        | SchedulePolicyKind::ManualOnly
        | SchedulePolicyKind::BurstOnBacklog => {}
    }

    Ok(())
}
