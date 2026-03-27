use crate::FleetError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekdayWindow {
    pub weekdays: Vec<u8>,
    pub start_hour: u8,
    pub end_hour: u8,
}

impl WeekdayWindow {
    pub fn validate(&self) -> Result<(), FleetError> {
        if self.weekdays.iter().any(|weekday| *weekday > 6) {
            return Err(FleetError::Validation {
                message: "weekday values must be between 0 and 6".to_string(),
            });
        }

        if self.start_hour > 23 || self.end_hour > 24 || self.start_hour == self.end_hour {
            return Err(FleetError::Validation {
                message: "weekday window hours must satisfy 0 <= start <= 23, 0 <= end <= 24, and start != end".to_string(),
            });
        }

        Ok(())
    }
}
