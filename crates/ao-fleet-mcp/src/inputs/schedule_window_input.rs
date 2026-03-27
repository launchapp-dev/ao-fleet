use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleWindowInput {
    pub weekdays: Vec<u8>,
    pub start_hour: u8,
    pub end_hour: u8,
}

impl From<ScheduleWindowInput> for ao_fleet_core::WeekdayWindow {
    fn from(value: ScheduleWindowInput) -> Self {
        Self { weekdays: value.weekdays, start_hour: value.start_hour, end_hour: value.end_hour }
    }
}
